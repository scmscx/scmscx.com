use tracing_log::LogTracer;
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

mod actix;
mod api;
mod db;
mod hacks;
mod middleware;
// mod search;
mod search2;
// mod ssr;
mod static_pages;
mod tests;
mod uiv2;
mod util;

// #[actix_web::main]

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // enable console_subcriber only in debug build because it consumes so much memory it breaks the server
    if cfg!(debug_assertions) {
        //console_subscriber::init();
    }

    LogTracer::init().expect("Failed to set logger");

    // let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    // let formatting_layer = BunyanFormattingLayer::new(
    //     "zero2prod".into(),
    //     // Output the formatted spans to stdout.
    //     std::io::stdout,
    // );
    // The `with` method is provided by `SubscriberExt`, an extension
    // trait for `Subscriber` exposed by `tracing_subscriber`
    // let subscriber = Registry::default()
    //     .with(env_filter)
    //     .with(JsonStorageLayer)
    //     .with(formatting_layer);
    // // `set_global_default` can be used by applications to specify
    // // what subscriber should be used to process spans.
    // set_global_default(subscriber).expect("Failed to set subscriber");

    let filter = EnvFilter::from_default_env();
    let subscriber = tracing_subscriber::fmt()
        // filter spans/events with level TRACE or higher.
        .with_env_filter(filter)
        .with_span_events(FmtSpan::CLOSE)
        .with_file(true)
        .with_target(false)
        .with_line_number(true)
        // build but do not install the subscriber.
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    //tracing_subscriber::fmt::init();

    anyhow::Ok(actix::start().await?)

    // let mut runner = actix_web::rt::System::new();
    // runner.block_on(async_main()).unwrap();

    // let mut runner = tokio::runtime::Builder::new_multi_thread()
    // .worker_threads(8)
    // .enable_all()
    // .build()
    // .unwrap();

    // runner.block_on(async_main()).unwrap();
}
