use std::ffi::OsString;

use crate::run::{Runner, RunnerError};

use tarpc::context;
// This is the service definition. It looks a lot like a trait definition.
// It defines one RPC, hello, which takes one arg, name, and returns a String.
#[tarpc::service]
pub trait Pyrod {
    //async fn execute(input: String, program: String) -> (String, String);
    async fn ping() -> String;

    async fn run_code(
        lang: super::run::Language,
        code: String,
        input: String,
    ) -> Result<(String, String), RunnerError>;
}

#[derive(Clone)]
pub struct PyrodServer;

#[tarpc::server]
impl Pyrod for PyrodServer {
    async fn ping(self, _: context::Context) -> String {
        "Pong!".to_owned()
    }

    async fn run_code(
        self,
        _: context::Context,
        lang: super::run::Language,
        code: String,
        input: String,
    ) -> Result<(String, String), RunnerError> {
        let runner = lang.get_runner();

        //there's no point making these async, because all they're doing
        //is a bit of filesystem stuff and calling other processes
        //my understanding is that using the tokio executor for this is not ideal
        //because the tokio executor is built for async stuff
        //so, we just spawn a regular thread.
        let result = std::thread::spawn(move || {
            runner
                .compile(code)
                .and_then(|path| runner.run(&path, input))
        })
        .join();

        match result {
            Ok(Ok(output)) => Ok(output),
            Ok(Err(e)) => Err(RunnerError::IOError(e.to_string())),
            Err(e) => Err(RunnerError::ThreadPanicked(format!("{e:?}"))),
        }
        .and_then(|(out, err)| {
            let convert = |s: OsString| s.into_string().map_err(|_| RunnerError::OutputUtf8Error);
            Ok((convert(out)?, convert(err)?))
        })
    }
}
