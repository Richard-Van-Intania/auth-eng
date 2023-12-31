use axum::{extract::State, response::IntoResponse, routing::get, Router};
use elasticsearch::{Elasticsearch, SearchParts};
use serde_json::json;

#[tokio::main]
async fn main() {
    let client = Elasticsearch::default();
    let app = Router::new()
        .route("/health", get(|| async {}))
        .route("/testelasticsearch", get(test_elasticsearch))
        .with_state(client);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn test() {}

async fn test_elasticsearch(State(client): State<Elasticsearch>) -> impl IntoResponse {
    let response = client
        .search(SearchParts::Index(&["books"]))
        .from(0)
        .size(10)
        .body(json!({
            "query": {
                "match": {
                    "name": "brave"
                }
            }
        }))
        .send()
        .await;
    match response {
        Ok(_) => "ok".to_string(),
        Err(e) => e.to_string(),
    }
}
