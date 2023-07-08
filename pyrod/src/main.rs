use anyhow::Result;
use futures::StreamExt as _;
use pyrod_service::{Pyrod, PyrodServer};
use tarpc::tokio_serde::formats::Bincode;
use tarpc::{
    server::{self, Channel},
    tokio_util::codec::length_delimited::LengthDelimitedCodec,
};
use tokio_vsock::VsockListener;

const PORT: u32 = 5000;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    //create a new vsock connection
    let mut incoming = VsockListener::bind(libc::VMADDR_CID_ANY, PORT)?.incoming();
    tracing::info!("Vsock listener opened on port {}", PORT);

    while let Some(result) = incoming.next().await {
        let stream = result?;
        tracing::info!(
            "Connection established. Local addr: {:?}, Remote addr {:?}",
            stream.local_addr(),
            stream.peer_addr()
        );

        //create the serde-based transport layer from the stream
        //build framed stream from raw one using length delimited codec
        let transport = tarpc::serde_transport::new(
            LengthDelimitedCodec::builder().new_framed(stream),
            Bincode::default(),
        );

        let fut = server::BaseChannel::with_defaults(transport).execute(PyrodServer.serve());
        tokio::spawn(fut);
        tracing::info!("Server spawned, listening for requests!");
    }

    Ok(())
}
