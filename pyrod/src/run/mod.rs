mod python;

pub use python::PythonRunner;

use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum Language {
    Python,
    Rust,
    Java,
}

pub trait Runner {
    fn compile(&self, code: String) -> Result<PathBuf, std::io::Error>;
    fn run(&self, path: &Path, stdin: String) -> Result<(OsString, OsString), std::io::Error>;
}

impl Language {
    pub fn get_runner(self) -> impl Runner {
        match self {
            Language::Python => PythonRunner,
            Language::Rust => todo!(),
            Language::Java => todo!(),
        }
    }
}

#[derive(Debug, Error, serde::Deserialize, serde::Serialize)]
pub enum RunnerError {
    #[error("Thread panicked during execution: {0}")]
    ThreadPanicked(String),
    #[error("I/O error while running code: {0}")]
    IOError(String),
    #[error("Output data from program was not valid UTF-8")]
    OutputUtf8Error,
}
