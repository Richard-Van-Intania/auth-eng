use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use serde::Deserialize;
use sqlx::{FromRow, PgPool};
use std::time::Duration;
use tower_http::timeout::TimeoutLayer;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    let pool = PgPool::connect("postgres://postgres:mysecretpassword@localhost:5432/app789plates")
        .await
        .expect("Failed to connect to database");
    let app = Router::new()
        .route("/health", get(get_health))
        .route("/testquery/:user_id", get(get_test_query))
        .route("/adduser", post(post_test_query))
        .route("/deleteallusers", delete(delete_all_users))
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .with_state(pool);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_health(State(pool): State<PgPool>) -> StatusCode {
    let rows = sqlx::query("SELECT * FROM public.users")
        .fetch_one(&pool)
        .await;
    match rows {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn get_test_query(Path(user_id): Path<i64>, State(pool): State<PgPool>) -> impl IntoResponse {
}

async fn post_test_query(
    State(pool): State<PgPool>,
    Json(payload): Json<Users>,
) -> Result<String, StatusCode> {
    let row: Result<(i64,), sqlx::Error> = sqlx::query_as(
        "INSERT INTO users (user_uuid, email, password) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(Uuid::new_v4().to_string())
    .bind(payload.email)
    .bind(payload.password)
    .fetch_one(&pool)
    .await;

    match row {
        Ok(id) => Ok(id.0.to_string()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn delete_all_users(
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    State(pool): State<PgPool>,
) -> StatusCode {
    let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
    if jwt.eq(bearer.to_owned().token()) {
        let rows = sqlx::query("DELETE FROM public.users").execute(&pool).await;
        match rows {
            Ok(_) => StatusCode::OK,
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    } else {
        StatusCode::UNAUTHORIZED
    }
}

#[derive(FromRow, Deserialize)]
struct Users {
    email: String,
    password: String,
}
