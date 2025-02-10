mod actix;
mod api;
mod db;
mod hacks;
mod middleware;
mod search2;
mod static_pages;
mod tests;
mod uiv2;
mod util;

use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use tracing_log::LogTracer;
use tracing_subscriber::{fmt::format::FmtSpan, layer::SubscriberExt, EnvFilter, Layer};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    LogTracer::init().expect("Failed to set logger");

    let mut builder = opentelemetry_sdk::trace::TracerProvider::builder();

    if let Ok(endpoint) = &std::env::var("JAEGER_ENDPOINT") {
        builder = builder.with_batch_exporter(
            // opentelemetry_otlp::SpanExporter::builder()
            //     .with_tonic()
            //     .with_endpoint(endpoint)
            //     .build()?,
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(endpoint)
                .build_span_exporter()?,
            opentelemetry_sdk::runtime::Tokio,
        );
    }

    // builder = builder.with_batch_exporter(
    //     opentelemetry_stdout::SpanExporter::default(),
    //     opentelemetry_sdk::runtime::Tokio,
    // );

    builder = builder.with_config(opentelemetry_sdk::trace::Config::default().with_resource(
        opentelemetry_sdk::Resource::new([
            opentelemetry::KeyValue::new("service.name", "scmscx.com"),
            opentelemetry::KeyValue::new("node", "scmscx.com"),
        ]),
    ));

    let tracer_provider = builder.build();
    let tracer = tracer_provider.tracer("scmscx.com");

    tracing::subscriber::set_global_default(
        tracing_subscriber::registry()
            .with(
                tracing_opentelemetry::layer()
                    .with_tracer(tracer)
                    .with_filter(tracing_subscriber::EnvFilter::new(
                        "trace,h2=info,scmscx_com=off,bwmap=off,bwmpq=off",
                    )),
            )
            .with(
                tracing_subscriber::fmt::layer()
                    .with_span_events(FmtSpan::CLOSE)
                    .with_file(true)
                    .with_target(false)
                    .with_line_number(true)
                    .with_filter(EnvFilter::from_default_env()),
            ),
    )?;

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
