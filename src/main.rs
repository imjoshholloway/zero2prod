use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::{configuration, startup, telemetry};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = telemetry::get_subscriber("zero2prod".into(), "info".into());
    telemetry::init_subscriber(subscriber);

    let config = configuration::get_configuration().expect("Failed to load configuration");
    let conn = PgPool::connect(&config.database.connection_string())
        .await
        .expect("Failed to connect to Postgres");

    let addr = format!("127.0.0.1:{}", config.app_port);
    let listener = TcpListener::bind(addr)?;
    startup::run(listener, conn)?.await?;
    Ok(())
}
