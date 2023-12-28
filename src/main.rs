use axum::{extract::State, http::StatusCode, routing::get, Router};
use sqlx::PgPool;
use std::time::Duration;
use tower_http::timeout::TimeoutLayer;

#[tokio::main]
async fn main() {
    let pool = PgPool::connect("postgres://postgres:mysecretpassword@localhost:5432/app789plates")
        .await
        .expect("Failed to connect to database");
    let app = Router::new()
        .route("/health", get(get_health))
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .with_state(pool);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_health(State(pool): State<PgPool>) -> StatusCode {
    let rows = sqlx::query("SELECT * FROM public.users")
        .execute(&pool)
        .await;
    match rows {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
