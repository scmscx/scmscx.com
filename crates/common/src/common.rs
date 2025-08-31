use anyhow::Result;
use bb8_postgres::{bb8::Pool, tokio_postgres::NoTls, PostgresConnectionManager};
use tracing_subscriber::{
    fmt::format::FmtSpan, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt,
    EnvFilter, Layer,
};

pub fn setup_logging() -> Result<()> {
    let filter_layer = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;

    // let console_layer = console_subscriber::ConsoleLayer::builder()
    //     .retention(Duration::from_secs(60))
    //     .server_addr(([0, 0, 0, 0], 5555))
    //     .spawn();

    let subscriber_layer = tracing_subscriber::fmt::layer()
        .with_span_events(FmtSpan::CLOSE)
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .with_filter(filter_layer);

    tracing_subscriber::registry()
        .with(subscriber_layer)
        //.with(console_layer)
        .init();

    anyhow::Ok(())
}

pub async fn setup_db() -> Result<Pool<PostgresConnectionManager<NoTls>>> {
    let connection_string = format!(
        "host={} port={} user={} password={} dbname={}",
        std::env::var("DB_HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_string())
            .as_str(),
        std::env::var("DB_PORT").unwrap().as_str(),
        std::env::var("DB_USER").unwrap().as_str(),
        std::env::var("DB_PASSWORD").unwrap().as_str(),
        std::env::var("DB_DATABASE")
            .unwrap_or_else(|_| std::env::var("DB_USER").unwrap())
            .as_str(),
    );
    let manager = PostgresConnectionManager::new(connection_string.parse()?, NoTls);

    let pool = Pool::builder()
        .max_size(
            std::env::var("DB_CONNECTIONS")
                .unwrap_or_else(|_| "16".to_string())
                .parse::<u32>()?,
        )
        .min_idle(Some(1))
        .max_lifetime(Some(std::time::Duration::from_secs(60)))
        .idle_timeout(Some(std::time::Duration::from_secs(30)))
        .test_on_check_out(true)
        .build(manager)
        .await?;

    anyhow::Ok(pool)
}
