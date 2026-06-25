use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use suiscope_core::{Registry, SuiRpcClient, SuiScopeConfig, WalrusClient};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tower_http::{
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
};
use tracing::info;


struct AppState {
    registry: Mutex<Registry>,
}

/// Start the SuiScope dashboard server on the given port.
///
/// This function blocks until the server is shut down.
pub async fn start(port: u16) -> anyhow::Result<()> {
    // Setup DB Connection
    let db_path = SuiScopeConfig::db_path()?;
    let registry = Registry::open(&db_path)?;
    let state = Arc::new(AppState {
        registry: Mutex::new(registry),
    });

    // Setup CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build Router
    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/objects", get(list_objects))
        .route("/api/transactions", get(list_transactions))
        .route("/api/errors", get(list_errors))
        .route("/api/inspect/:id", get(inspect_object))
        .route("/api/graph", get(get_object_graph))
        .route("/api/walrus/upload", post(walrus_upload))
        .route("/api/walrus/import/:blob_id", get(walrus_import))

        // Serve the statically exported Next.js frontend
        .fallback_service(
            ServeDir::new("frontend/out")
                .not_found_service(ServeFile::new("frontend/out/404.html")),
        )
        .layer(cors)
        .with_state(state);

    // Start Server
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!("Starting SuiScope Dashboard server on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
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

async fn inspect_object(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    let config = match SuiScopeConfig::load() {
        Ok(c) => c,
        Err(e) => return Json(json!({ "error": e.to_string() })),
    };
    let rpc_client = SuiRpcClient::new(&config.rpc_url());
    let network = config.sui_network();

    match rpc_client.get_object(&id).await {
        Ok(result) => {
            // Upsert into registry so other tabs reflect the inspected object
            let tracked = suiscope_core::types::TrackedObject {
                id: None,
                object_id: result.object_id.clone(),
                object_type: result.object_type.clone(),
                alias: None,
                owner: result.owner.clone(),
                package_id: None,
                version: result.version.clone(),
                digest: result.digest.clone(),
                tx_digest: result.previous_transaction.clone(),
                network: network.as_str().to_string(),
                created_at: None,
                updated_at: None,
            };
            if let Ok(registry) = state.registry.lock() {
                let _ = registry.upsert_object(&tracked);
            }

            let tx_url = result
                .previous_transaction
                .as_deref()
                .map(|tx| network.explorer_tx_url(tx));
            let obj_url = network.explorer_object_url(&result.object_id);

            Json(json!({
                "object_id": result.object_id,
                "version": result.version,
                "digest": result.digest,
                "object_type": result.object_type,
                "owner": result.owner,
                "previous_transaction": result.previous_transaction,
                "storage_rebate": result.storage_rebate,
                "content": result.content,
                "explorer_tx_url": tx_url,
                "explorer_object_url": obj_url,
                "network": network.as_str(),
            }))
        }
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

async fn get_object_graph(
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    let registry = state.registry.lock().unwrap();
    let objects = registry.list_all_objects().unwrap_or_default();
    
    // Build nodes and edges for graph visualization
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    
    for obj in &objects {
        nodes.push(json!({
            "id": obj.object_id,
            "label": obj.alias.as_deref().unwrap_or(&obj.object_id[..8]),
            "type": if obj.object_type.is_some() { "object" } else { "package" },
            "network": obj.network,
            "package_id": obj.package_id,
        }));
        
        // Create edges based on package relationships
        if let Some(pkg_id) = &obj.package_id {
            if objects.iter().any(|o| &o.object_id == pkg_id) {
                edges.push(json!({
                    "from": obj.object_id,
                    "to": pkg_id,
                    "label": "depends_on",
                }));
            }
        }
    }
    
    Json(json!({
        "nodes": nodes,
        "edges": edges,
    }))
}

async fn walrus_upload(
    State(state): State<Arc<AppState>>,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    use axum::http::StatusCode;
    
    let config = match SuiScopeConfig::load() {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    };
    
    let db_path = match SuiScopeConfig::db_path() {
        Ok(p) => p,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    };
    
    if !db_path.exists() {
        return (StatusCode::BAD_REQUEST, Json(json!({ "error": "No registry database found" }))).into_response();
    }
    
    let temp_upload_db = db_path.parent()
        .map(|p| p.join("temp_upload.db"))
        .unwrap_or_else(|| std::path::PathBuf::from("temp_upload.db"));

    // Create a fully-consolidated copy of the DB
    if let Ok(registry) = state.registry.lock() {
        let _ = std::fs::remove_file(&temp_upload_db);
        let vacuum_query = format!("VACUUM INTO '{}';", temp_upload_db.to_string_lossy().replace('\'', "''"));
        if let Err(e) = registry.connection().execute_batch(&vacuum_query) {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("Failed to create database snapshot: {}", e) }))).into_response();
        }
    }

    let db_bytes = match tokio::fs::read(&temp_upload_db).await {
        Ok(b) => {
            let _ = tokio::fs::remove_file(&temp_upload_db).await;
            b
        },
        Err(e) => {
            let _ = tokio::fs::remove_file(&temp_upload_db).await;
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("Failed to read database snapshot: {}", e) }))).into_response();
        }
    };
    
    let walrus_client = WalrusClient::new(&config.walrus_publisher, &config.walrus_aggregator);
    
    let blob_id = match walrus_client.upload_blob(&db_bytes).await {
        Ok(id) => id,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("Walrus upload failed: {}", e) }))).into_response(),
    };
    
    Json(json!({
        "blob_id": blob_id,
        "message": "Registry uploaded successfully",
    })).into_response()
}

async fn walrus_import(
    Path(blob_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    use axum::http::StatusCode;

    let config = match SuiScopeConfig::load() {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    };
    
    let db_path = match SuiScopeConfig::db_path() {
        Ok(p) => p,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    };
    
    let walrus_client = WalrusClient::new(&config.walrus_publisher, &config.walrus_aggregator);
    
    let bytes = match walrus_client.download_blob(&blob_id).await {
        Ok(b) => b,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(json!({ "error": format!("Walrus download failed: {}", e) }))).into_response(),
    };
    
    let temp_db_path = db_path.parent()
        .map(|p| p.join("temp_import.db"))
        .unwrap_or_else(|| std::path::PathBuf::from("temp_import.db"));
    
    if let Err(e) = tokio::fs::write(&temp_db_path, bytes).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("Failed to write temp file: {}", e) }))).into_response();
    }
    
    let merge_res = {
        let registry = state.registry.lock().unwrap();
        registry.merge_from_db_file(&temp_db_path)
    };
    
    let _ = tokio::fs::remove_file(&temp_db_path).await;
    
    if let Err(e) = merge_res {
        return (StatusCode::BAD_REQUEST, Json(json!({ "error": format!("Merge failed: file is likely not a valid SuiScope registry database. Error: {}", e) }))).into_response();
    }
    
    Json(json!({
        "message": "Registry imported and merged successfully",
    })).into_response()
}
