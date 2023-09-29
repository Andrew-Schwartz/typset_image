use std::env;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::{ExitStatus, Output};

use itertools::Itertools;
use once_cell::sync::Lazy;
use thiserror::Error;
use tokio::fs;
use tokio::process::Command;
use crate::gui::Dir;

use crate::GuiError;

#[derive(Debug, Error, Clone)]
pub enum CommandError {
    #[error("could not start command `{0}`")]
    ErrorSpawning(String),
    #[error("{command} returned {status}:\n{message}")]
    Error {
        status: ExitStatus,
        command: String,
        message: String,
    },
}

const LATEX_START: &str = r"\documentclass[12pt]{article}
\usepackage{amsmath}
\usepackage{amssymb}
\usepackage{amsfonts}
\usepackage[usenames,dvipsnames]{color}
\usepackage[utf8]{inputenc}
\thispagestyle{empty}
\begin{document}
\color{white}
\begin{align*}
    ";

const LATEX_END: &str = r"
\end{align*}
\end{document}";

static CACHE_DIR: Lazy<PathBuf> = Lazy::new(|| {
    let path = dirs::data_local_dir()
        .expect("unsupported os?")
        .join("latex_image");
    std::fs::create_dir_all(&path).unwrap();
    path
});

pub fn get_dir(hash: u64) -> Dir {
    let hash_dir = format!("latex_{hash}");
    CACHE_DIR.join(hash_dir)
}

pub async fn gen_svg(latex: String, hash: u64, color: String) -> Result<Dir, GuiError> {
    let initial_dir = env::current_dir()
        .map_err(|_| GuiError::GetSetCurrentDir)?;

    let dir = get_dir(hash);
    fs::create_dir(&dir).await
        .map_err(|_| GuiError::TempDir)?;

    // println!("dir = {:?}", dir);
    env::set_current_dir(&dir)
        .map_err(|_| GuiError::GetSetCurrentDir)?;

    fs::write("eq.tex", format!("{LATEX_START}{latex}{LATEX_END}"))
        .await
        .map_err(|_| GuiError::WriteFile("eq.tex".into()))?;

    let _output = run_command("latex", [
        "-no-shell-escape",
        "-interaction=nonstopmode",
        "-halt-on-error",
        "eq.tex"
    ]).await?;

    let _output = run_command("dvisvgm", [
        "--no-fonts",
        "--scale=1",
        "--exact",
        // &format!("-o {file_name}"),
        "-o eq.svg",
        "eq.dvi"
    ]).await?;

    set_color(hash, color)
        .await?;

    env::set_current_dir(initial_dir)
        .map_err(|_| GuiError::GetSetCurrentDir)?;

    Ok(dir)
}

pub async fn gen_png(dir: Dir, color: String, density: usize) -> Result<Dir, GuiError> {
    let initial_dir = env::current_dir()
        .map_err(|_| GuiError::GetSetCurrentDir)?;

    env::set_current_dir(&dir)
        .map_err(|_| GuiError::GetSetCurrentDir)?;

    let _output = run_command("magick.exe", [
        "convert",
        "-background", "none",
        "-density", &density.to_string(),
        &format!("{color}_eq.svg"),
        &format!("{color}_eq.png"),
    ]).await?;

    env::set_current_dir(initial_dir)
        .map_err(|_| GuiError::GetSetCurrentDir)?;

    Ok(dir)
}

/// copies `eq.svg` to `{color}_eq.svg` and changes the fill color
pub async fn set_color(hash: u64, color: String) -> Result<(), GuiError> {
    let dir = get_dir(hash);
    let svg = fs::read_to_string(dir.join("eq.svg"))
        .await
        .map_err(|_| GuiError::ReadFile("eq.svg".to_string()))?;

    let svg = svg.replace(
        "#fff",
        &color,
    );

    let path_colored = dir.join(format!("{color}_eq.svg"));
    fs::write(&path_colored, svg)
        .await
        .map_err(|_| GuiError::WriteFile(path_colored.to_string_lossy().to_string().into()))
}

async fn run_command<I, S>(command: &str, args: I) -> Result<String, CommandError>
    where
        I: IntoIterator<Item=S>,
        S: AsRef<OsStr>,
{
    fn utf8_to_string(utf8: &[u8]) -> String {
        std::str::from_utf8(utf8)
            .map(str::to_string)
            .expect("latex & dvisvgm always have utf8 outputs")
    }
    // Constant can be found in `winapi` or `windows` crates as well
    //
    // List of all process creation flags:
    // https://learn.microsoft.com/en-us/windows/win32/procthread/process-creation-flags
    const CREATE_NO_WINDOW: u32 = 0x0800_0000; // Or `134217728u32`

    let Output { status, stdout, stderr } = Command::new(command)
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .await
        .map_err(|_| CommandError::ErrorSpawning(command.to_string()))?;
    if status.success() {
        Ok(utf8_to_string(&stdout))
    } else {
        println!("stderr = {}", utf8_to_string(&stderr));
        println!("stdout = {}", utf8_to_string(&stdout));
        let message = utf8_to_string(&stdout);
        let message = if message.is_empty() {
            utf8_to_string(&stderr)
        } else if let Some(idx) = message.find('!') {
            message[idx..].lines()
                .take_while(|l| l.chars().any(|c| !c.is_ascii_whitespace()))
                .join("\n")
        } else {
            message
        };
        Err(CommandError::Error {
            status,
            command: command.to_string(),
            message,
        })
    }
}
