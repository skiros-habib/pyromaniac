use std::process::Command;
use std::{ffi::OsString, io::Write, os::unix::prelude::OsStringExt};
use std::{
    path::{Path, PathBuf},
    process::Stdio,
};

#[derive(Debug)]
pub struct PythonRunner;

impl super::Runner for PythonRunner {
    ///For python, all we need to do is write the code to a file somewhere
    #[tracing::instrument]
    fn compile(&self, code: String) -> Result<PathBuf, std::io::Error> {
        let path = PathBuf::from("/tmp/code.py");
        std::fs::write(&path, code)?;
        tracing::debug!("Code written out to {path:?}");
        Ok(path)
    }

    #[tracing::instrument(skip(self, stdin))]
    fn run(&self, path: &Path, stdin: String) -> Result<(OsString, OsString), std::io::Error> {
        //spawn child process
        let mut child = Command::new("/usr/bin/python")
            .arg(path.as_os_str())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        tracing::debug!("Child process spawned");

        //pipe input
        child.stdin.take().unwrap().write_all(stdin.as_bytes())?;
        //collect output
        let output = child.wait_with_output()?;

        tracing::debug!("Output collected, process joined");

        Ok((
            OsString::from_vec(output.stdout),
            OsString::from_vec(output.stderr),
        ))
    }
}
