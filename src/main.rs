use axum::{http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, FromRow, Row};

const DB_URL: &'static str = "postgres://postgres:mysecretpassword@localhost/testuser";

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/health", get(|| async { StatusCode::OK }))
        .route("/testdb", get(test_db_connect))
        .route("/testdbticket", get(test_db_connect_ticket));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn test_db_connect() -> impl IntoResponse {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(DB_URL)
        .await
        .unwrap();
    let rows = sqlx::query("SELECT * FROM public.users")
        .execute(&pool)
        .await
        .unwrap();
    let local: DateTime<Local> = Local::now();
    local.to_string()
}

async fn test_db_connect_ticket(Json(payload): Json<Ticket>) -> impl IntoResponse {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(DB_URL)
        .await
        .unwrap();
    //
    sqlx::query(
        r#"
    CREATE TABLE IF NOT EXISTS ticket (
      id bigserial,
      name text
    );"#,
    )
    .execute(&pool)
    .await
    .unwrap();
    //
    let row: (i64,) = sqlx::query_as("insert into ticket (name) values ($1) returning id")
        .bind(payload.name)
        .fetch_one(&pool)
        .await
        .unwrap();

    //
    let rows = sqlx::query("SELECT * FROM ticket")
        .fetch_all(&pool)
        .await
        .unwrap();
    let str_result = rows
        .iter()
        .map(|r| format!("{} - {}", r.get::<i64, _>("id"), r.get::<String, _>("name")))
        .collect::<Vec<String>>()
        .join(", ");
    println!("\n== select tickets with PgRows:\n{}", str_result);
    //
    let select_query = sqlx::query_as::<_, Ticket>("SELECT id, name FROM ticket");
    let tickets: Vec<Ticket> = select_query.fetch_all(&pool).await.unwrap();
    println!("\n=== select tickets with query.map...: \n{:?}", tickets);
    StatusCode::OK
}

// StatusCode::OK

#[derive(Debug, FromRow, Deserialize, Serialize)]
struct Ticket {
    id: i64,
    name: String,
}
