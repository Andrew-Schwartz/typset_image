use std::env;

use tokio::fs;

use crate::GuiError;
use crate::backends::run_command;
use crate::gui::Dir;

const TYPST_START: &str = r##"
#import "@preview/physica:0.8.1": *
#set page(width: auto, height: auto, margin: 0pt)
#set text(11pt, font: "New Computer Modern", lang: "en", fill: "##;

// using my vendored typst for the --background option for pngs
const TYPST: &str = r"C:\Users\andre\CLionProjects\typst\target\release\typst.exe";

enum Image {
    Svg,
    Png(usize),
}

async fn gen_image(eq: String, dir: Dir, color: String, image: Image) -> Result<(), GuiError> {

    // println!("dir = {:?}", dir);

    let initial_dir = env::current_dir()
        .map_err(|_| GuiError::GetSetCurrentDir)?;

    env::set_current_dir(&dir)
        .map_err(|_| GuiError::GetSetCurrentDir)?;

    fs::write("eq.typ", format!("{TYPST_START}{color})\n$ {eq} $"))
        .await
        .map_err(|_| GuiError::WriteFile("eq.typ".into()))?;

    let _output = match image {
        Image::Svg => run_command(TYPST, [
            "compile",
            "eq.typ",
            &format!("{color}_eq.svg"),
            "--diagnostic-format",
            "short",
        ],
        ).await?,
        Image::Png(dpi) => run_command(TYPST, [
            "compile",
            "eq.typ",
            &format!("{color}_eq.png"),
            "--diagnostic-format",
            "short",
            "--ppi",
            &dpi.to_string(),
            "--background",
            "#00000000",
        ]).await?,
    };

    env::set_current_dir(initial_dir)
        .map_err(|_| GuiError::GetSetCurrentDir)?;

    Ok(())
}

pub async fn gen_svg(eq: String, dir: Dir, color: String) -> Result<(), GuiError> {
    // println!("GENERATE SVG from Typst");
    gen_image(eq, dir, color, Image::Svg).await
}

pub async fn gen_png(eq: String, dir: Dir, color: String, density: usize) -> Result<(), GuiError> {
    // println!("GENERATE PNG from Typst");
    gen_image(eq, dir, color, Image::Png(density)).await
}
