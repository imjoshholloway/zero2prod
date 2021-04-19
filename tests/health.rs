use std::net::TcpListener;


#[actix_rt::test]
async fn health_check_works() {
    let addr = spawn_app();

    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health", &addr))
        .send()
        .await
        .expect("Failed to send request!");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

fn spawn_app() -> String {
    // bind to a randomly assigned port
    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind random port");

    // extract the port from the listener
    let port = listener.local_addr().unwrap().port();

    let server = zero2prod::run(listener)
        .expect("Failed to start server!");

    // start as background task
    let _ = tokio::spawn(server);

    // return the address to the caller
    format!("http://127.0.0.1:{}", port)
}
