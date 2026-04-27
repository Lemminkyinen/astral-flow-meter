use crate::AmperageData;
use axum::{Json, Router, extract::State, routing::get};
use std::sync::{Arc, RwLock};

type SharedState = Arc<RwLock<AmperageData>>;

pub async fn run(state: SharedState, port: u16) {
    let app = Router::new()
        .route("/metrics", get(get_metrics))
        .with_state(state);
    let listener = match tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("Failed to bind server to port {}: {}", port, e);
            return;
        }
    };

    tracing::info!("HTTP server listening on http://0.0.0.0:{}", port);

    if let Err(e) = axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.ok();
        })
        .await
    {
        tracing::error!("Server error: {}", e);
    };

    tracing::info!("HTTP server stopped");
}

async fn get_metrics(State(state): State<SharedState>) -> Json<AmperageData> {
    let data = state.read().unwrap();
    Json(data.clone())
}
