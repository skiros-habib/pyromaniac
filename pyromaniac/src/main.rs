use tracing::{Instrument, Level};
use tracing_subscriber::fmt::format::FmtSpan;
mod api;
mod config;
mod runner;

#[tokio::main]
async fn main() {
    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt::fmt()
            .with_span_events(FmtSpan::ACTIVE)
            .pretty()
            .with_max_level(Level::DEBUG)
            .finish(),
    )
    .expect("Could not install tracing subscriber");

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(api::app().into_make_service())
        .instrument(tracing::info_span!("Web server"))
        .await
        .expect("Could not start server");
}
