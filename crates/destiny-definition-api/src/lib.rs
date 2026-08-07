//! Async HTTP adapter around `destiny-runtime-core`.
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use destiny_runtime_core::Runtime;
use serde_json::json;
use std::sync::Arc;
use tokio::net::TcpListener;

pub type AppState = Arc<Runtime>;

pub fn router(runtime: AppState) -> Router {
    Router::new()
        .route("/definition/:hash", get(definition))
        .route("/references/:hash", get(references))
        .route("/stats", get(stats))
        .with_state(runtime)
}

pub async fn serve(runtime: AppState, address: &str) -> anyhow::Result<()> {
    let listener = TcpListener::bind(address).await?;
    let bound = listener.local_addr()?;
    println!("destiny-definition-api listening on http://{bound}");
    axum::serve(listener, router(runtime))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
            println!("shutdown signal received");
        })
        .await?;
    Ok(())
}

async fn definition(
    State(runtime): State<AppState>,
    Path(hash): Path<String>,
) -> impl IntoResponse {
    match runtime.get_definition(&hash) {
        Ok(Some(value)) => (StatusCode::OK, Json(json!(value))),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error":"definition_not_found"})),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error":e.to_string()})),
        ),
    }
}
async fn references(
    State(runtime): State<AppState>,
    Path(hash): Path<String>,
) -> impl IntoResponse {
    match runtime.get_references(&hash) {
        Ok(value) => (StatusCode::OK, Json(json!(value))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error":e.to_string()})),
        ),
    }
}
async fn stats(State(runtime): State<AppState>) -> impl IntoResponse {
    match runtime.stats() {
        Ok((files, fields, references, strings)) => (
            StatusCode::OK,
            Json(json!({"files":files,"fields":fields,"references":references,"strings":strings})),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error":e.to_string()})),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };

    #[tokio::test]
    async fn starts_on_ephemeral_port_and_serves_stats() {
        let db = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../destiny.db");
        let runtime = Arc::new(Runtime::open(db).unwrap());
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let app = router(runtime);
        let task = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        let mut client = tokio::net::TcpStream::connect(address).await.unwrap();
        client
            .write_all(b"GET /stats HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
            .await
            .unwrap();
        let mut response = String::new();
        client.read_to_string(&mut response).await.unwrap();
        assert!(response.starts_with("HTTP/1.1 200 OK"));
        assert!(response.contains("\"files\":12192"));
        task.abort();
    }
}
