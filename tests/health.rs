use sqlx::{PgPool, Executor, PgConnection, Connection};
use std::net::TcpListener;
use zero2prod::configuration::{self, DatabaseSettings};

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    // bind to a randomly assigned port
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");

    // extract the port from the listener
    let port = listener.local_addr().unwrap().port();

    let mut config = configuration::get_configuration().expect("Failed to load configuration file");

    config.database.database_name = uuid::Uuid::new_v4().to_string();

    let db_pool = configure_database(&config.database).await;

    let server = zero2prod::startup::run(listener, db_pool.clone()).expect("Failed to start server!");

    // start as background task
    let _ = tokio::spawn(server);

    TestApp{
        address: format!("http://127.0.0.1:{}", port),
        db_pool,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres");


    connection.execute(&*format!(r#"CREATE DATABASE "{}";"#, config.database_name))
        .await
        .expect("Failed to create database");

    let db_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres");

    sqlx::migrate!("./migrations").run(&db_pool)
        .await
        .expect("Failed to migrate database");

    db_pool
}

#[actix_rt::test]
async fn health_check_works() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health", &app.address))
        .send()
        .await
        .expect("Failed to send request!");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[actix_rt::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request!");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscriptions");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[actix_rt::test]
async fn subscribe_returns_400_when_data_is_missing() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request!");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 bad request when the payload was {}",
            error_message,
        )
    }
}
