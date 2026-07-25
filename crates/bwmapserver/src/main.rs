mod access;
mod api;
mod db;
mod middleware;
mod pumpers;
mod ratelimit;
mod search2;
mod server;
mod static_pages;
mod tests;
mod uiv2;
mod util;
mod webutil;

use tracing_log::LogTracer;
use tracing_subscriber::{fmt::format::FmtSpan, layer::SubscriberExt, EnvFilter, Layer};

fn main() -> anyhow::Result<()> {
    // Build the runtime explicitly (rather than `#[tokio::main]`) so the tokio
    // poll-time histogram can be enabled — see `common::telemetry::build_runtime`.
    common::telemetry::build_runtime()?.block_on(run())
}

async fn run() -> anyhow::Result<()> {
    LogTracer::init().expect("Failed to set logger");

    tracing::subscriber::set_global_default(
        tracing_subscriber::registry().with(
            tracing_subscriber::fmt::layer()
                .with_span_events(FmtSpan::CLOSE)
                .with_file(true)
                .with_target(false)
                .with_line_number(true)
                .with_filter(EnvFilter::from_default_env()),
        ),
    )?;

    anyhow::Ok(server::start().await?)
}
