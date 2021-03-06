use crate::routes::{health, subscribe};
use actix_web::{dev::Server, web, App, HttpServer};
use tracing_actix_web::TracingLogger;
use sqlx::PgPool;
use std::net::TcpListener;

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    // web::Data is essentially an Arc smart pointer
    let db_pool = web::Data::new(db_pool);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger)
            .route("/health", web::get().to(health))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
