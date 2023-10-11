use std::ffi::OsStr;
use std::process::{ExitStatus, Output};

use itertools::Itertools;
use thiserror::Error;
use tokio::process::Command;

use crate::{GuiError, latex, typst};
use crate::gui::Dir;

#[derive(Default, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Backend {
    LaTeX,
    #[default]
    Typst,
}

impl Backend {
    pub const fn letter(self) -> &'static str {
        match self {
            Self::LaTeX => "L",
            Self::Typst => "T",
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::LaTeX => "latex",
            Self::Typst => "typst",
        }
    }

    pub const fn stylized(self) -> &'static str {
        match self {
            Self::LaTeX => "LaTeX",
            Self::Typst => "Typst",
        }
    }

    pub async fn gen_png(self, eq: String, dir: Dir, color: String, dpi: usize) -> Result<(), GuiError> {
        match self {
            Backend::LaTeX => latex::gen_png(dir, color, dpi).await,
            Backend::Typst => typst::gen_png(eq, dir, color, dpi).await,
        }
    }
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

pub async fn run_command<I, S>(command: &str, args: I) -> Result<String, CommandError>
    where
        I: IntoIterator<Item=S> + Send,
        S: AsRef<OsStr>,
{
    fn utf8_to_string(utf8: &[u8]) -> String {
        std::str::from_utf8(utf8)
            .map(str::to_string)
            .expect("latex, typst, dvisvgm always have utf8 outputs")
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
        let message = utf8_to_string(&stdout);
        println!("stdout = {}", message);
        println!("stderr = {}", utf8_to_string(&stderr));
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
