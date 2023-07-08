mod api;
mod runner;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    runner::run_code("".to_owned(), "".to_owned()).await?;
    Ok(())
}
