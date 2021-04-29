use actix_web::{web, Result, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;
use tracing_futures::Instrument;

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, HttpResponse> {
    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!("Adding a new subscriber",
        %request_id,
        email = %form.email,
        name = %form.name,
    );

    let _request_span_guard = request_span.enter();

    let query_span = tracing::info_span!("Saving new subscriber details to the database");

    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(pool.as_ref())
    .instrument(query_span)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        eprintln!("Failed to execute query: {}", e);
        HttpResponse::InternalServerError().finish()
    })?;
    Ok(HttpResponse::Ok().finish())
}
