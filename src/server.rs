use crate::AmperageData;
use axum::{Json, Router, extract::State, routing::get};
use std::sync::{Arc, RwLock};

type SharedState = Arc<RwLock<AmperageData>>;

pub async fn run(state: SharedState) {
    let app = Router::new()
        .route("/metrics", get(get_metrics))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    tracing::info!("HTTP server listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn get_metrics(State(state): State<SharedState>) -> Json<AmperageData> {
    let data = state.read().unwrap();
    Json(data.clone())
}
