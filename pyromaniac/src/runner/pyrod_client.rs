use anyhow::{Context, Result};
use pyrod_service::PyrodClient;
use std::path::Path;
use tarpc::context;
use tarpc::tokio_serde::formats::Bincode;
use tarpc::tokio_util::codec::{length_delimited::LengthDelimitedCodec, Framed};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
const PORT: u16 = 5000;

async fn connect(sock: impl AsRef<Path>) -> Result<PyrodClient> {
    let sock = sock.as_ref();
    //we can't just use tarpc::unix::connect because we need to establish the connection with the port number over the raw stream first
    let mut stream = tokio::net::UnixStream::connect(sock)
        .await
        .context(format!("Could not open stream on Unix socket {sock:?}"))?;

    tracing::debug!("Opened stream on unix socket {:?}", sock);

    //write the connection message with port into the stream
    let conn_message = format!("CONNECT {}\n", PORT).into_bytes();
    stream.write_all(&conn_message).await?;
    tracing::debug!(
        "Connection message for port {} written into socket {:?}",
        PORT,
        sock
    );

    //read two chars from the buf
    //we want it to say "OK"
    let mut buf = vec![0u8; 20];
    let n_read = stream.read(&mut buf).await?;
    if &buf[0..1] != "OK".as_bytes() && buf[13] != b'\n' {
        anyhow::bail!(
            "Did not get OK message back from server: got {}",
            String::from_utf8_lossy(&buf)
        )
    }
    //until we get end of line
    tracing::debug!(
        "Got OK message back from socket {:?}, {} ({} bytes read)",
        sock,
        String::from_utf8_lossy(&buf),
        n_read
    );

    //we can hand over the stream to tarpc now
    //delimit the AsyncRead/AsyncWrite into frames using a length header
    let framed_stream = Framed::new(stream, LengthDelimitedCodec::new());

    //create the serde-based transport layer from the stream
    let transport = tarpc::serde_transport::new(framed_stream, Bincode::default());
    let client = PyrodClient::new(Default::default(), transport).spawn();
    tracing::debug!("Connection to server established on socket {:?}", sock);
    Ok(client)
}

pub async fn ping(sock: impl AsRef<Path>) -> Result<()> {
    let client = connect(sock).await.context("Failed to create RPC client")?;
    tracing::debug!("Sending Ping...");
    let response = client.ping(context::current()).await?;

    tracing::debug!("Ping response: {}", response);
    (response == "Pong!")
        .then_some(())
        .ok_or_else(|| anyhow::anyhow!("bad ping"))
}
