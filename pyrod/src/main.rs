use tarpc::{
    server::{self, Channel},
    tokio_util::codec::{length_delimited::LengthDelimitedCodec, Framed},
};

use pyrod_service::{Pyrod, PyrodServer};
use tokio_vsock::VsockStream;

const PORT: u32 = 5000;
const CID: u32 = 3;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    //create a new vsock connection
    let vsock_stream = VsockStream::connect(CID, PORT).await?;
    //delimit the AsyncRead/AsyncWrite into frames using a length header
    let framed_stream = Framed::new(vsock_stream, LengthDelimitedCodec::new());

    //create the serde-based transport layer from the stream
    let transport = tarpc::serde_transport::new(
        framed_stream,
        tarpc::tokio_serde::formats::Bincode::default(),
    );

    let server = server::BaseChannel::with_defaults(transport);
    server.execute(PyrodServer.serve()).await;

    Ok(())
}
