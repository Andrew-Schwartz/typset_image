#![windows_subsystem = "windows"]
#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::too_many_lines,
    clippy::default_trait_access,
    clippy::unreadable_literal,
    clippy::wildcard_imports,
)]

use std::borrow::Cow;
use std::fmt::Debug;
use iced::{Application, Font, Settings};
use thiserror::Error;
use latex::CommandError;

mod gui;
mod utils;
mod style;
mod easing;
mod circular;
mod latex;
mod icons;

pub const ICON_FONT_BYTES: &[u8] = include_bytes!("../resources/latex-image-icons.ttf");

pub const ICON_FONT: Font = Font::with_name("latex-image-icons");

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
}

