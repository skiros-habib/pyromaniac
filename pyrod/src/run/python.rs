use std::{ffi::OsString, io::Write, os::unix::prelude::OsStringExt};
use std::{os::unix::process::CommandExt, process::Command};
use std::{path::PathBuf, process::Stdio};

use super::RunError;

#[derive(Debug)]
pub struct PythonRunner;

impl super::Runner for PythonRunner {
    ///For python, all we need to do is write the code to a file somewhere
    #[tracing::instrument]
    fn compile(&self, code: String) -> Result<(), RunError> {
        let path = PathBuf::from("/tmp/code.py");
        std::fs::write(&path, code).map_err(RunError::from)?;
        tracing::debug!("Code written out to {path:?}");
        Ok(())
    }

    #[tracing::instrument(skip(self, stdin))]
    fn run(&self, stdin: String) -> Result<(OsString, OsString), RunError> {
        //spawn child process
        let mut child = Command::new("/usr/local/bin/python")
            .arg("/tmp/code.py")
            .uid(111) //service user id of untrusted process - don't want to run as root
            .gid(111) //set in the dockerfiles used to build rootfs images
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
