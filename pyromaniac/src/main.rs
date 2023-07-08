use pyrod_service::Language;
use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;

mod api;
mod config;
mod runner;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt::fmt()
            .with_span_events(FmtSpan::ACTIVE)
            .with_max_level(Level::DEBUG)
            .finish(),
    )?;

    let output = runner::run_code(
        Language::Python,
        "print('hello world!')".to_owned(),
        "".to_owned(),
    )
    .await?;
    dbg!(output);
    Ok(())
}
