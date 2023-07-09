use anyhow::{Context, Result};
use futures::StreamExt as _;
use pyrod_service::{Pyrod, PyrodServer};
use tarpc::tokio_serde::formats::Bincode;
use tarpc::{
    server::{self, Channel},
    tokio_util::codec::length_delimited::LengthDelimitedCodec,
};
use tokio_vsock::VsockListener;
use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;

const PORT: u32 = 5000;

mod init;

#[tokio::main]
async fn main() -> Result<()> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt::fmt()
            .with_span_events(FmtSpan::ACTIVE)
            .with_max_level(Level::DEBUG)
            .finish(),
    )?;

    //linux system init stuff
    init::init();

    //create a new vsock connection
    let mut incoming = VsockListener::bind(libc::VMADDR_CID_ANY, PORT)
        .context(format!(
            "Failed to open vsock listener cid={} port={}",
            libc::VMADDR_CID_ANY,
            PORT
        ))?
        .incoming();
    tracing::info!("Vsock listener opened on port {}", PORT);

    while let Some(result) = incoming.next().await {
        let stream = result.context("Failed to get vsock stream")?;
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
