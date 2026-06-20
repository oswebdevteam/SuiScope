use axum::{
    extract::State,
    routing::get,
    Json, Router,
};
use serde_json::{json, Value};
use suiscope_core::{Registry, SuiScopeConfig};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};
use tracing::info;

struct AppState {
    registry: Mutex<Registry>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // 1. Setup DB Connection
    let db_path = SuiScopeConfig::db_path().expect("Failed to locate .suiscope directory");
    let registry = Registry::open(&db_path).expect("Failed to open SQLite registry");
    let state = Arc::new(AppState {
        registry: Mutex::new(registry),
    });

    // 2. Setup CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 3. Build Router
    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/objects", get(list_objects))
        .route("/api/transactions", get(list_transactions))
        .route("/api/errors", get(list_errors))
        // Serve static files from the React frontend (Engineer 3 / Enginneer 2 to build this)
        .fallback_service(ServeDir::new("frontend/dist"))
        .layer(cors)
        .with_state(state);

    // 4. Start Server
    let port = 7731;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!("Starting SuiScope Dashboard server on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> Json<Value> {
    Json(json!({ "status": "ok", "service": "suiscope-dashboard" }))
}

// Returns all tracked objects across every network; the frontend filters by
// network and search text client-side.
async fn list_objects(
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    let registry = state.registry.lock().unwrap();
    let objects = registry.list_all_objects().unwrap_or_default();
    Json(json!(objects))
}

async fn list_transactions(
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    let registry = state.registry.lock().unwrap();
    let txs = registry.list_all_transactions(200).unwrap_or_default();
    Json(json!(txs))
}

async fn list_errors(
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    let registry = state.registry.lock().unwrap();
    let errors = registry.list_all_errors(200).unwrap_or_default();
    Json(json!(errors))
}
