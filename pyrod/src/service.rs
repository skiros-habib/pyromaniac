use std::ffi::OsString;

use crate::run::RunError;

use tarpc::context;
use tracing::{span, Level};
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
    ) -> Result<(String, String), RunError>;
}

#[derive(Clone, Debug)]
pub struct PyrodServer;

#[tarpc::server]
impl Pyrod for PyrodServer {
    #[tracing::instrument(skip(self, _ctx))]
    async fn ping(self, _ctx: context::Context) -> String {
        "Pong!".to_owned()
    }

    #[tracing::instrument(skip(self, code, input))]
    async fn run_code(
        self,
        _: context::Context,
        lang: super::run::Language,
        code: String,
        input: String,
    ) -> Result<(String, String), RunError> {
        let runner = lang.get_runner();

        //there's no point making these async, because all they're doing
        //is a bit of filesystem stuff and calling other processes
        //my understanding is that using the tokio executor for this is not ideal
        //because the tokio executor is built for async stuff
        //so, we just spawn a regular thread.
        let result = std::thread::spawn(move || -> Result<_, _> {
            let _span = span!(Level::DEBUG, "Code runner on thread").entered();
            runner.run(&runner.compile(code)?, input)
        })
        .join();

        let output = match result {
            Ok(Ok(output)) => Ok(output),
            Ok(Err(e)) => Err(RunError::IOError(e.to_string())),
            Err(e) => Err(RunError::ThreadPanicked(format!("{e:?}"))),
        }
        .and_then(|(out, err)| {
            let convert = |s: OsString| s.into_string().map_err(|_| RunError::OutputUtf8Error);
            Ok((convert(out)?, convert(err)?))
        });

        match &output {
            Ok((out, err)) => tracing::info!(
                "Code executed succesfully!\n stdout:\n {} \n stderr: \n {}\n:",
                out,
                err
            ),
            Err(e) => tracing::info!("Error during execution: {}", e),
        }
        output
    }
}
