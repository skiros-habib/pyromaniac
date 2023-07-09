use anyhow::Result;
use pyrod_service::{Pyrod, PyrodServer};
use tarpc::tokio_serde::formats::Bincode;
use tarpc::{
    server::{self, Channel},
    tokio_util::codec::length_delimited::LengthDelimitedCodec,
};
use tokio_vsock::VsockStream;
use tracing::{Instrument, Level};
use tracing_subscriber::fmt::format::FmtSpan;

const PORT: u32 = 5000;

mod init;

#[tokio::main]
async fn main() -> Result<()> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt::fmt()
            .with_span_events(FmtSpan::ACTIVE)
            .with_max_level(Level::TRACE)
            .finish(),
    )?;

    //linux system init stuff
    init::init();

    //create a new vsock connection
    //we initiate the connection when we're ready
    let stream = VsockStream::connect(2, PORT).await?;
    tracing::info!("Vsock connection opened on port {}", PORT);

    //create the serde-based transport layer from the stream
    //build framed stream from raw one using length delimited codec
    let transport = tarpc::serde_transport::new(
        LengthDelimitedCodec::builder().new_framed(stream),
        Bincode::default(),
    );

    server::BaseChannel::with_defaults(transport)
        .execute(PyrodServer.serve())
        .instrument(tracing::info_span!(
            "Spawning RPC server on vsock client connection"
        ))
        .await;

    Ok(())
}
