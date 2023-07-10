use std::{ffi::OsString, time::Duration};

use crate::run::RunError;

use tarpc::context;
use tokio::{task::spawn_blocking, time::timeout};
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
        timeouts: (Duration, Duration),
    ) -> Result<(OsString, OsString), RunError>;
}

#[derive(Clone, Debug)]
pub struct PyrodServer;

#[tarpc::server]
impl Pyrod for PyrodServer {
    #[tracing::instrument(skip(self, _ctx))]
    async fn ping(self, _ctx: context::Context) -> String {
        "Pong!".to_owned()
    }

    // #[tracing::instrument(skip(self, code, input))]
    async fn run_code(
        self,
        _: context::Context,
        lang: super::run::Language,
        code: String,
        input: String,
        (compile_timeout, run_timeout): (Duration, Duration),
    ) -> Result<(OsString, OsString), RunError> {
        //it's a zero-sized type, and this process is
        let runner = lang.get_runner();

        //there's no point making these async, because all they're doing
        //is a bit of filesystem stuff and calling other processes
        //which is not something we need to do asynchronously
        //but we do need to spawn_blocking because function colours

        let compile_task = timeout(compile_timeout, spawn_blocking(|| runner.compile(code)))
            .await
            //handle compile timeout
            //other errors handled for us
            .map_err(|_| RunError::CompileTimeout(compile_timeout))??;

        //if we get a compile error can return early with an okay (skill issue error)
        if let Err(RunError::CompileError(out, err)) = compile_task {
            tracing::info!("Compilation error: stdout: {:?}, stderr: {:?}", out, err);
            return Ok((out, err));
        }
        //return any unexpected errors we got
        compile_task?;

        timeout(run_timeout, spawn_blocking(|| runner.run(input)))
            .await
            .map_err(|_| RunError::RunTimeout(run_timeout))??
    }
}
