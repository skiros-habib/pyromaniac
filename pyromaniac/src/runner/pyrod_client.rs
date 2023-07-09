use anyhow::{Context, Result};
use pyrod_service::PyrodClient;
use std::fmt::Debug;
use std::{path::Path, time::SystemTime};
use tarpc::context;
use tarpc::tokio_serde::formats::Bincode;
use tarpc::tokio_util::codec::length_delimited::LengthDelimitedCodec;
use tokio::net::UnixListener;

#[tracing::instrument]
async fn connect(sock: impl AsRef<Path> + Debug) -> Result<(PyrodClient, UnixListener)> {
    let sock = sock.as_ref();

    //we can't just use tarpc::unix::connect because we need to establish the connection with the port number over the raw stream first
    //this is also confusing, because we are the *server* here as far as the vsock layer is concerned
    //so we need to keep the connection open
    let listener =
        UnixListener::bind(sock).context(format!("Failed to open unix socket {sock:?}"))?;

    tracing::debug!(
        "Started listening for pyrod process on unix socket {:?}",
        sock
    );

    let (stream, addr) = listener.accept().await?;
    tracing::info!(
        "Accepted connection on socket {:?} with addr {:?}",
        sock,
        addr
    );

    //we can hand over the stream to tarpc now
    //build transport layer using serde bincode and length-delimited frames
    let transport = tarpc::serde_transport::new(
        LengthDelimitedCodec::builder().new_framed(stream),
        Bincode::default(),
    );

    //we create a tarpc *client* on this end, because we're the one making RPC calls
    let client = PyrodClient::new(Default::default(), transport).spawn();
    tracing::debug!("Connection to pyrod established on socket {:?}", sock);
    Ok((client, listener))
}

#[tracing::instrument(skip(code, input, lang))]
#[must_use]
pub async fn run_code(
    sock: impl AsRef<Path> + Debug,
    lang: pyrod_service::Language,
    code: String,
    input: String,
) -> Result<(String, String)> {
    let (client, _l) = connect(sock).await.context("Failed to create RPC client")?;

    // ping commented out for speed
    // client.ping(context::current()).await?;
    // tracing::info!("Got Pong from VM");

    let mut ctx = context::current();
    ctx.deadline = SystemTime::now() + std::time::Duration::from_secs(60);
    client
        .run_code(ctx, lang, code, input)
        .await?
        .map_err(Into::into)
}
