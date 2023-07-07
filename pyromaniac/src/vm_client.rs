use anyhow::Result;
use pyrod_service::PyrodClient;
use std::path::Path;
use tarpc::context;
use tarpc::serde_transport::unix;
use tarpc::tokio_serde::formats::Bincode;

pub async fn send_request(sock: impl AsRef<Path>) -> Result<()> {
    let connection = unix::connect(sock.as_ref(), Bincode::default).await?;
    let client = PyrodClient::new(Default::default(), connection).spawn();

    let response = client.echo(context::current(), "Boo!".into()).await?;
    dbg!(response);

    Ok(())
}
