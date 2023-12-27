use std::collections::HashMap;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Extension, Json, Router,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Row};

const DATABASE_URL: &'static str =
    "postgres://postgres:mysecretpassword@localhost:5432/app789plates";
const MAX_CONNECTION: usize = 80;

#[tokio::main]
async fn main() {
    let pool = PgPool::connect(DATABASE_URL)
        .await
        .expect("Failed to connect to database");

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/health", get(|| async {}))
        .route("/testdb", get(test_db_connect))
        .route("/testdbticket", get(test_db_connect_ticket))
        .route("/dataticket/:user_id", post(data_ticket))
        .route("/dataticketone/:user_id", post(data_ticket_one))
        .route("/testjson", get(test_json))
        .route("/testquery", get(test_query))
        .route("/testpath/:user_id", get(test_path))
        .route("/foo", get(get_foo).post(post_foo))
        .route("/testusingstate", get(get_testusingstate))
        .route("/api/:version/users/:id/action", delete(do_users_action))
        .with_state(pool); // You should prefer using State if possible since it’s more type safe
                           // .layer(Extension(pool));

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

async fn data_ticket(
    Extension(pool): Extension<PgPool>,
    Path(user_id): Path<u32>,
) -> impl IntoResponse {
    let select_query = sqlx::query_as::<_, Ticket>("SELECT id, name FROM ticket");
    let tickets: Vec<Ticket> = select_query
        .fetch_all(&pool)
        .await
        .expect("Failed to fetch data from the database");
    tickets.get(user_id as usize).unwrap().name.to_owned()
}

async fn data_ticket_one(
    Extension(pool): Extension<PgPool>,
    Path(user_id): Path<usize>,
) -> impl IntoResponse {
    // let ticket = sqlx::query_as::<_, Ticket>("SELECT * FROM public.ticket WHERE id = ?")
    //     .bind(user_id)
    //     .fetch_one(&pool)
    //     .await
    //     .expect("Failed to fetch data from the database");
    // ticket.name
}
async fn test_path(Path(user_id): Path<usize>) -> String {
    user_id.to_string()
}

async fn test_query(Query(params): Query<HashMap<String, String>>) {
    println!("{}", params.len());
}

async fn test_json() -> Json<Ticket> {
    Json(Ticket {
        id: 10,
        name: "mytest".to_string(),
    })
}

async fn get_foo() {}
async fn post_foo() {}

async fn get_testusingstate(State(pool): State<PgPool>) -> String {
    let rows = sqlx::query("SELECT * FROM public.users")
        .execute(&pool)
        .await
        .unwrap();
    let local: DateTime<Local> = Local::now();
    local.to_string()
}

async fn authenticated_handler(
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> impl IntoResponse {
}

async fn do_users_action(Path((version, id)): Path<(String, usize)>) {}

// StatusCode::OK

#[derive(Debug, FromRow, Deserialize, Serialize)]
struct Ticket {
    id: i64,
    name: String,
}
