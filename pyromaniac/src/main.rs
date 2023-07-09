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

    //     let script = r"
    // import sys
    // print('Hello, world!')
    // print('Hello, stderr!', file=sys.stderr)
    // print('Hello, stdin:')
    // for line in sys.stdin:
    //     line = line.rstrip()
    //     print(f'Message from stdin: {line}')
    //     ";

    let rust = "
fn main() {
    println!(\"Hello, world\");
}
    ";

    let output = runner::run_code(
        Language::Rust,
        rust.to_owned(),
        "line 1\n line2\n line3".to_owned(),
    )
    .await?;
    dbg!(output);
    Ok(())
}
