use std::{ffi::OsString, io::Write, os::unix::prelude::OsStringExt};
use std::{os::unix::process::CommandExt, process::Command};
use std::{path::PathBuf, process::Stdio};

use super::RunError;
#[derive(Debug)]
pub struct RustRunner;

impl super::Runner for RustRunner {
    //root image should already have a cargo project in it with the dependencies that we promised
    //this will be located at /cargo_project
    //we need to our file to be /cargo_project/src/main.rs
    //and to then run cargo build (--release)
    #[tracing::instrument]
    fn compile(&self, code: String) -> Result<(), RunError> {
        let path = PathBuf::from("/cargo_project/src/main.rs");
        std::fs::write(&path, code)?;
        tracing::debug!("Code written out to {path:?}");

        std::env::set_var("RUSTUP_HOME", "/usr/local/rustup");
        std::env::set_var("CARGO_HOME", "/usr/local/cargo");
        std::env::set_var(
            "PATH",
            "/usr/local/cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
        );

        let child = Command::new("/usr/local/cargo/bin/cargo") // where it's installed in alpine
            .arg("build")
            .arg("--release")
            .arg("--offline")
            .arg("--quiet")
            .current_dir("/cargo_project")
            .env(
                "RUSTFLAGS",
                "--sysroot=/usr/local/rustup/toolchains/1.70.0-x86_64-unknown-linux-musl",
            )
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .uid(111)
            .gid(111)
            .spawn()?;

        //collect output
        let output = child.wait_with_output()?;

        if output.status.success() {
            tracing::info!("Code compiled succesfully");
            Ok(())
        } else {
            tracing::error!("Code failed to compile");
            Err(RunError::CompileError(
                OsString::from_vec(output.stdout),
                OsString::from_vec(output.stderr),
            ))
        }
    }

    #[tracing::instrument(skip(self, stdin))]
    fn run(&self, stdin: String) -> Result<(OsString, OsString), RunError> {
        //spawn child process
        let mut child = Command::new("/cargo_project/target/release/cargo_project")
            .uid(111) //non-root uids
            .gid(111)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        tracing::debug!("Child process spawned");

        // pipe input
        child.stdin.take().unwrap().write_all(stdin.as_bytes())?;
        // collect output
        let output = child.wait_with_output()?;

        tracing::debug!("Output collected, process joined");

        Ok((
            OsString::from_vec(output.stdout),
            OsString::from_vec(output.stderr),
        ))
    }
}
