use std::{env, iter};
use std::path::PathBuf;

use tokio::fs;

use crate::backends::run_command;
use crate::gui::Dir;
use crate::{col, GuiError};

const TYPST_START: &str = r##"#set page(width: auto, height: auto, margin: 0pt)
#set text(fill: "##;

enum Image {
    Svg,
    Png(usize),
}

async fn gen_image(eq: String, dir: Dir, color: String, image: Image) -> Result<(), GuiError> {
    // using my vendored typst for the --background option for pngs
    const TYPST: &'static str = r#"C:\Users\andre\CLionProjects\typst\target\release\typst.exe"#;

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
            ]
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

#[derive(Debug)]
pub enum Never {}

pub async fn watch(dir: PathBuf) -> Result<Never, GuiError> {
    env::set_current_dir(dir)
        .map_err(|_| GuiError::GetSetCurrentDir)?;

    fs::write("eq.typ", format!("{TYPST_START}white)\n$$"))
        .await
        .map_err(|_| GuiError::WriteFile("eq.typ".into()))?;

    let output = run_command("typst", [
        "watch",
        "eq.typ",
        "eq.svg"
    ]).await?;

    todo!("shouldn't finish? here's the output: {output}")
}