use axum::{http::StatusCode, response::IntoResponse, routing::get, Extension, Json, Router};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Row};

const DATABASE_URL: &'static str = "postgres://postgres:mysecretpassword@localhost/testuser";

#[tokio::main]
async fn main() {
    let pool = PgPool::connect(DATABASE_URL)
        .await
        .expect("Failed to connect to database");

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/health", get(|| async { StatusCode::OK }))
        .route("/testdb", get(test_db_connect))
        .route("/testdbticket", get(test_db_connect_ticket))
        .route("/testdbticketout", get(get_data_ticket))
        .layer(Extension(pool));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn test_db_connect(Extension(pool): Extension<PgPool>) -> impl IntoResponse {
    let rows = sqlx::query("SELECT * FROM public.users")
        .execute(&pool)
        .await
        .unwrap();
    let local: DateTime<Local> = Local::now();
    local.to_string()
}

async fn test_db_connect_ticket(
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<Ticket>,
) -> impl IntoResponse {
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

async fn get_data_ticket(Extension(pool): Extension<PgPool>) -> impl IntoResponse {
    let select_query = sqlx::query_as::<_, Ticket>("SELECT id, name FROM ticket");
    let tickets: Vec<Ticket> = select_query
        .fetch_all(&pool)
        .await
        .expect("Failed to fetch data from the database");
    tickets.get(0).unwrap().name.to_owned()
}

async fn degug_app(Extension(pool): Extension<PgPool>) -> impl IntoResponse {}

// StatusCode::OK

#[derive(Debug, FromRow, Deserialize, Serialize)]
struct Ticket {
    id: i64,
    name: String,
}
