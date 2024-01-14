use axum::{
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use elasticsearch::{
    auth::Credentials,
    cert::{Certificate, CertificateValidation},
    http::{
        transport::{SingleNodeConnectionPool, TransportBuilder},
        Url,
    },
    Elasticsearch, SearchParts,
};
use serde_json::{json, Value};
use std::{collections::HashMap, fs::File, io::Read};

#[tokio::main]
async fn main() {
    let url = Url::parse("https://localhost:9200").unwrap();
    let pool = SingleNodeConnectionPool::new(url);
    let credentials = Credentials::Basic("elastic".into(), "Yxp=9DLAR_kXWedXdejI".into());
    let mut buf = Vec::new();
    File::open("http_ca.crt")
        .unwrap()
        .read_to_end(&mut buf)
        .unwrap();
    let certificate = Certificate::from_pem(&buf).unwrap();
    let validation: CertificateValidation = CertificateValidation::Full(certificate);
    let transport = TransportBuilder::new(pool)
        .auth(credentials)
        .cert_validation(validation)
        .build()
        .unwrap();
    let client = Elasticsearch::new(transport);

    let app = Router::new()
        .route("/health", get(|| async {}))
        .route("/testelasticsearch", get(test_elasticsearch))
        .route("/testejson", get(test_json))
        .with_state(client);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3315").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn test() {}

async fn test_json() -> Json<Value> {
    Json(json!({ "data": 42 }))
}

async fn test_elasticsearch(
    Query(params): Query<HashMap<String, String>>,
    State(client): State<Elasticsearch>,
) -> impl IntoResponse {
    let search = params.get("search").unwrap();
    let response = client
        .search(SearchParts::Index(&["dbdrev1"]))
        .from(0)
        .size(10)
        .body(json!({
            "query": {
                "match": {
                    "legal_entity_registration_number": search
                }
            }
        }))
        .send()
        .await
        .unwrap();

    let body = response.json::<Value>().await.unwrap();
    Json(body)
}
