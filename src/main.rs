#![windows_subsystem = "windows"]

use std::borrow::Cow;
use std::fmt::Debug;
use iced::{Application, Settings};
use thiserror::Error;
use latex::CommandError;

mod gui;
mod utils;
mod style;
mod easing;
mod circular;
mod latex;

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
    // todo rename
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
