#![windows_subsystem = "windows"]

use std::borrow::Cow;
use std::env;
use std::ffi::OsStr;
use std::fmt::Debug;
use std::path::PathBuf;
use std::process::{ExitStatus, Output};

use iced::{Application, Settings};
use itertools::Itertools;
use tempdir::TempDir;
use thiserror::Error;
use tokio::fs::{self};
use tokio::process::Command;

mod gui;
mod utils;
mod style;
mod easing;
mod circular;

// #[derive(Parser, Debug)]
// struct CliArgs {
//     latex: String,
//     #[arg(short, long)]
//     name: Option<String>,
//     color: Option<String>,
// }

#[derive(Debug, Error, Clone)]
pub enum GuiError {
    #[error("Enter a LaTeX expression!")]
    NoLatex,
    #[error("could not create temporary directory")]
    TempDir,
    #[error("could not get/set the current directory")]
    GetSetCurrentDir,
    #[error("could not write to `{0}`")]
    WriteFile(Cow<'static, str>),
    #[error("could not read from `{0}`")]
    ReadFile(String),
    #[error("could not copy file from `{0}` to `{1}`")]
    CopyFile(String, String),
    #[error(transparent)]
    Command(#[from] CommandError),
}

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

const LATEX_START: &str = r#"\documentclass[12pt]{article}
\usepackage{amsmath}
\usepackage{amssymb}
\usepackage{amsfonts}
\usepackage{xcolor}
\usepackage[utf8]{inputenc}
\thispagestyle{empty}
\begin{document}
\begin{align*}
    "#;
const LATEX_END: &str = r#"
\end{align*}
\end{document}"#;

fn main() {
    gui::Gui::run(Settings {
        antialiasing: true,
        ..Settings::default()
    }).unwrap();
    // if let Err(e) = main2() {
    //     println!("{e}");
    // }
}

// fn main2() -> Result<(), Box<dyn Error>> {
//     let CliArgs { latex, name, color } = CliArgs::parse();
//     let file_name = match name {
//         None => "eq.svg".to_string(),
//         Some(name) => if name.ends_with(".svg") {
//             name
//         } else {
//             format!("{name}.svg")
//         }
//     };
//     let color = color.as_deref().unwrap_or("black");
//
//     gen_svg(&latex, color, &file_name)?;
//
//     Ok(())
// }

// pub fn gen_svg2(latex: &str, color: &str, file_name: &str) -> Result<(), Box<dyn Error>> {
//     let mut status = status::NoopStatusBackend::default();
//     let config = config::PersistentConfig::open(false)?;
//     let bundle = config.default_bundle(false, &mut status)?;
//
//     let start = Instant::now();
//     let mut builder = driver::ProcessingSessionBuilder::default();
//     builder.bundle(bundle)
//         .primary_input_buffer(format!("{LATEX_START}{latex}{LATEX_END}").as_bytes())
//         .tex_input_name("texput.tex")
//         .format_name("latex")
//         .keep_logs(false)
//         .keep_intermediates(false)
//         .print_stdout(false)
//         .output_format(driver::OutputFormat::Xdv)
//         // .do_not_write_output_files()
//     ;
//
//     let mut session = builder.create(&mut status)?;
//     session.run(&mut status)?;
//     println!("start = {:?}", start.elapsed());
//
//     let files = session.into_file_data();
//     println!("files = {:?}", files);
//
//     let start = Instant::now();
//     let _output = run_command("dvisvgm", [
//         "--no-fonts",
//         "--scale=1",
//         "--exact",
//         &format!("-o {file_name}"),
//         "texput.xdv"
//     ])?;
//     println!("start.elapsed() = {:?}", start.elapsed());
//
//     Ok(())
// }

pub async fn gen_svg(latex: String, color: String, file_name: String) -> Result<PathBuf, GuiError> {
    let tmp_dir = TempDir::new("svg_latex")
        .map_err(|_| GuiError::TempDir)?;
    let initial_dir = env::current_dir()
        .map_err(|_| GuiError::GetSetCurrentDir)?;
    env::set_current_dir(tmp_dir.path())
        .map_err(|_| GuiError::GetSetCurrentDir)?;

    fs::write("eq.tex", format!("{LATEX_START}{latex}{LATEX_END}"))
        .await
        .map_err(|_| GuiError::WriteFile("eq.tex".into()))?;

    // let start = Instant::now();
    let _output = run_command("latex", [
        "-no-shell-escape",
        "-interaction=nonstopmode",
        "-halt-on-error",
        "eq.tex"
    ]).await?;
    // println!("start.elapsed() = {:?}", start.elapsed());

    // let start = Instant::now();
    let _output = run_command("dvisvgm", [
        "--no-fonts",
        "--scale=1",
        "--exact",
        &format!("-o {file_name}"),
        "eq.dvi"
    ]).await?;
    // println!("start.elapsed() = {:?}", start.elapsed());

    // let start = Instant::now();
    let mut in_page = false;
    let contents = fs::read_to_string(&file_name)
        .await
        .map_err(|_| GuiError::ReadFile(file_name.clone()))?
        .lines()
        .map(|line| {
            let mut line = line.to_string();
            if line == "</g>" {
                in_page = false;
            }
            if in_page {
                let tag = &line[..line.len() - 2];
                line = format!("{tag} fill='{color}'/>");
            }
            if line.starts_with("<g id=") {
                in_page = true;
            }
            line
        })
        .join("\n");
    fs::write(&file_name, contents)
        .await
        .map_err(|_| GuiError::WriteFile(file_name.clone().into()))?;

    let svg = initial_dir.join(&file_name);
    // println!("svg = {}", svg.to_str().unwrap());
    fs::copy(&file_name, &svg)
        .await
        .map_err(|_| GuiError::CopyFile(file_name.clone(), svg.to_str().unwrap().to_string()))?;
    // println!("start.elapsed() = {:?}", start.elapsed());

    env::set_current_dir(initial_dir)
        .map_err(|_| GuiError::GetSetCurrentDir)?;

    Ok(svg)
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
        let message = utf8_to_string(&stdout);
        println!("stdout = {}", message);
        let message = if let Some(idx) = message.find('!') {
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
