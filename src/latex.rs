use std::env;
use tokio::fs;
use crate::gui::Dir;

use crate::{backends, GuiError};

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

pub async fn gen_svg(latex: String, dir: Dir, color: String) -> Result<(), GuiError> {
    // println!("GENERATE SVG from LaTeX");

    let initial_dir = env::current_dir()
        .map_err(|_| GuiError::GetSetCurrentDir)?;

    // let dir = gui::get_dir(hash);
    fs::create_dir(&dir).await
        .map_err(|_| GuiError::TempDir)?;

    // println!("dir = {:?}", dir);
    env::set_current_dir(&dir)
        .map_err(|_| GuiError::GetSetCurrentDir)?;

    fs::write("eq.tex", format!("{LATEX_START}{latex}{LATEX_END}"))
        .await
        .map_err(|_| GuiError::WriteFile("eq.tex".into()))?;

    let _output = backends::run_command("latex", [
        "-no-shell-escape",
        "-interaction=nonstopmode",
        "-halt-on-error",
        "eq.tex"
    ]).await?;

    let _output = backends::run_command("dvisvgm", [
        "--no-fonts",
        "--scale=1",
        "--exact",
        // &format!("-o {file_name}"),
        "-o eq.svg",
        "eq.dvi"
    ]).await?;

    set_color(dir, color)
        .await?;

    env::set_current_dir(initial_dir)
        .map_err(|_| GuiError::GetSetCurrentDir)?;

    Ok(())
}

pub async fn gen_png(dir: Dir, color: String, density: usize) -> Result<(), GuiError> {
    // println!("GENERATE PNG from LaTeX");

    let initial_dir = env::current_dir()
        .map_err(|_| GuiError::GetSetCurrentDir)?;

    env::set_current_dir(&dir)
        .map_err(|_| GuiError::GetSetCurrentDir)?;

    let _output = backends::run_command("magick.exe", [
        "convert",
        "-background", "none",
        "-density", &density.to_string(),
        &format!("{color}_eq.svg"),
        &format!("{color}_eq.png"),
    ]).await?;

    env::set_current_dir(initial_dir)
        .map_err(|_| GuiError::GetSetCurrentDir)?;

    Ok(())
}

/// copies `eq.svg` to `{color}_eq.svg` and changes the fill color
pub async fn set_color(dir: Dir, color: String) -> Result<(), GuiError> {
    // println!("LATEX: SET COLOR");

    // let dir = gui::get_dir(hash);
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
