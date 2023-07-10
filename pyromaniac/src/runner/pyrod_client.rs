use anyhow::{anyhow, Context, Result};
use pyrod_service::PyrodClient;
use std::os::unix::fs::PermissionsExt;
use std::{ffi::OsString, fmt::Debug};
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

    //we need to chmod the port that is being listned on
    //so that the firecracker process can connect
    //as it runs under a different uid in jail.
    //again, chown probably better
    std::fs::set_permissions(sock, std::fs::Permissions::from_mode(0o777))
        .expect("Could not set perms for socket");

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
    let timeouts = (
        crate::config::get().runner_config.compile_timeout,
        crate::config::get().runner_config.run_timeout,
    );

    //include 5 seconds of slack
    ctx.deadline = SystemTime::now() + timeouts.0 + timeouts.1 + std::time::Duration::from_secs(5);
    let (stdout, stderr) = client
        .run_code(ctx, lang, code, input, timeouts)
        .await?
        .map_err(anyhow::Error::from)?;

    let convert = |s: OsString| {
        s.into_string()
            .map_err(|_| anyhow!("Output was not valid UTF8, could not convet to string"))
    };

    Ok((convert(stdout)?, convert(stderr)?))
}
