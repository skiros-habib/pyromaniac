use futures::future::{self, Ready};
use tarpc::context;
// This is the service definition. It looks a lot like a trait definition.
// It defines one RPC, hello, which takes one arg, name, and returns a String.
#[tarpc::service]
pub trait Pyrod {
    //async fn execute(input: String, program: String) -> (String, String);
    async fn ping() -> String;
    async fn echo(msg: String) -> String;
}

#[derive(Clone)]
pub struct PyrodServer;

impl Pyrod for PyrodServer {
    fn ping(self, _: context::Context) -> Self::PingFut {
        future::ready("Pong!".to_owned())
    }

    fn echo(self, _: context::Context, msg: String) -> Self::EchoFut {
        future::ready(msg)
    }

    type PingFut = Ready<String>;

    type EchoFut = Ready<String>;
}
