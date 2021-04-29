use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::{configuration, startup};
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use tracing_log::LogTracer;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Redirect all logs to the subscriber
    LogTracer::init().expect("Failed to set logger");

    // env filter with a fallback to "info" level logs if the RUST_LOG env var isn't defined
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer = BunyanFormattingLayer::new(
        "zero2prod".into(),
        // Output the formatted spans to stdout.
        std::io::stdout
    );

    // create our subscriber and use the `with` method provided by `SubscrbierExt` to decorate it
    // with our various layers
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);


    set_global_default(subscriber).expect("Failed to set logging tracing subscriber");

    let config = configuration::get_configuration().expect("Failed to load configuration");
    let conn = PgPool::connect(&config.database.connection_string())
        .await
        .expect("Failed to connect to Postgres");

    let addr = format!("127.0.0.1:{}", config.app_port);
    let listener = TcpListener::bind(addr)?;
    startup::run(listener, conn)?.await?;
    Ok(())
}
