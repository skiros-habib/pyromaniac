mod python;
mod rust;

use std::ffi::OsString;
use thiserror::Error;

#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum Language {
    Python,
    Rust,
    Java,
}

pub trait Runner {
    fn compile(&self, code: String) -> Result<(), RunError>;
    fn run(&self, stdin: String) -> Result<(OsString, OsString), RunError>;
}

impl Language {
    pub fn get_runner(self) -> Box<dyn Runner + Send + Sync> {
        match self {
            Language::Python => Box::new(python::PythonRunner),
            Language::Rust => Box::new(rust::RustRunner),
            Language::Java => todo!(),
        }
    }
}

#[derive(Debug, Error, serde::Deserialize, serde::Serialize)]
pub enum RunError {
    #[error("Thread panicked during execution: {0}")]
    ThreadPanicked(String),
    #[error("I/O error while running code: {0}")]
    IOError(String),
    #[error(
        "File not found error while running code. Fuck you if you wanted to know what file though."
    )]
    FileNotFoundError,
    #[error("Output data from program was not valid UTF-8")]
    OutputUtf8Error,
    #[error("Code failed to compile: stdout: {0:?}, stderr: {1:?}")]
    CompileError(OsString, OsString),
}

impl From<std::io::Error> for RunError {
    fn from(value: std::io::Error) -> Self {
        match value.kind() {
            std::io::ErrorKind::NotFound => Self::FileNotFoundError,
            _ => Self::IOError(format!("{value:?}")),
        }
    }
}
