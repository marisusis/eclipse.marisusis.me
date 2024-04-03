pub mod config;

use anyhow::Result;
use axum::extract::{Path, State};
use axum::response::Html;
use axum::{http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use config::ConfigFile;
use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tower_http::services::ServeDir;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPointFlags {
    has_gps_fix: bool,
    is_clipping: bool,
}

impl Default for DataPointFlags {
    fn default() -> DataPointFlags {
        DataPointFlags {
            has_gps_fix: false,
            is_clipping: false,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DataPoint {
    timestamp: Option<i64>,
    sample_rate: f32,
    flags: DataPointFlags,
    latitude: f32,
    longitude: f32,
    elevation: f32,
    speed: f32,
    angle: f32,
    fix: u16,
    data: Vec<f64>,
}

#[derive(Deserialize, Serialize, Debug)]
struct LastDataResponse {
    data: DataPoint,
}

#[derive(Deserialize, Serialize, Debug)]
struct TimeDataResponse {
    data: Vec<NodeEntry>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
enum NodeStatus {
    #[serde(rename = "online")]
    Online,

    #[serde(rename = "timeout")]
    Timeout,

    #[serde(rename = "nogpsfix")]
    NoGpsFix,

    #[serde(rename = "offline")]
    Offline,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct NodeEntry {
    node_id: String,
    status: NodeStatus,
    location: String,
    last_update: i64,
    data: Option<DataPoint>,
}

struct NodeInfo {
    location: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Params {
    node: String,
}

struct AppState {
    data_cache: Arc<RwLock<BTreeMap<String, Option<DataPoint>>>>,
    node_infos: Arc<BTreeMap<String, NodeInfo>>,
}

async fn all_data_handler(
    State(app_state): State<Arc<AppState>>,
) -> Result<Json<TimeDataResponse>, StatusCode> {
    let data_cache = app_state.data_cache.read().await;
    let node_infos = app_state.node_infos.clone();

    let mut nodes = Vec::new();
    for (node_id, data) in data_cache.iter() {
        match data {
            Some(data) => {
                nodes.push(NodeEntry {
                    node_id: node_id.clone(),
                    location: node_infos.get(node_id).unwrap().location.clone(),
                    last_update: data.timestamp.unwrap_or(0),
                    status: NodeStatus::Online,
                    data: Some(data.clone()),
                });
            }
            None => {
                nodes.push(NodeEntry {
                    node_id: node_id.clone(),
                    location: node_infos.get(node_id).unwrap().location.clone(),
                    last_update: 0,
                    status: NodeStatus::Offline,
                    data: None,
                });
            }
        }
    }

    if !nodes.is_empty() {
        return Ok(Json(TimeDataResponse { data: nodes }));
    }

    return Err(StatusCode::IM_A_TEAPOT);
}

async fn node_handler(
    Path(params): Path<Params>,
    State(app_state): State<Arc<AppState>>,
) -> Result<Json<NodeEntry>, StatusCode> {
    let data_cache = app_state.data_cache.read().await;
    let node_infos = app_state.node_infos.clone();

    let mut node_id = params.node.clone();
    node_id.make_ascii_uppercase();
    if let Some(data) = data_cache.get(&node_id) {
        if let Some(data) = data {
            return Ok(Json(NodeEntry {
                node_id: node_id.clone(),
                location: node_infos.get(&node_id.clone()).unwrap().location.clone(),
                status: NodeStatus::Online,
                last_update: data.timestamp.unwrap_or(0),
                data: Some(data.clone()),
            }));
        }
    }

    return Err(StatusCode::IM_A_TEAPOT);
}

#[tokio::main(flavor = "multi_thread", worker_threads = 16)]
async fn main() -> Result<()> {
    // Initialize logger
    simple_logger::init_with_level(log::Level::Info).unwrap();

    log::info!("Starting ET Live Data Server");

    // Load config file
    let node_file_contents = fs::read_to_string("config.toml")?;
    let config: ConfigFile = toml::from_str(&node_file_contents)?;

    // Create ET data cache
    let mut data_cache = BTreeMap::<String, Option<DataPoint>>::new();
    let mut node_infos = BTreeMap::<String, NodeInfo>::new();

    // initialize cache with None
    for node in config.nodes.iter() {
        data_cache.insert(node.node_id.clone(), None);
        node_infos.insert(
            node.node_id.clone(),
            NodeInfo {
                location: node.location.clone(),
            },
        );
    }

    let node_infos = node_infos;

    // Move to multi-thread/access capable storage
    let data_cache = std::sync::Arc::new(tokio::sync::RwLock::new(data_cache));
    let node_infos = std::sync::Arc::new(node_infos);

    // Create app state
    let app_state = AppState {
        data_cache: data_cache.clone(),
        node_infos: node_infos.clone(),
    };

    let app_state = Arc::new(app_state);

    // Create axum router
    let router = Router::new()
        .nest_service("/", ServeDir::new("app/build"))
        .route("/api/data/all", get(all_data_handler))
        .route("/api/data/:node", get(node_handler))
        .with_state(app_state)
        .layer(tower_http::cors::CorsLayer::permissive())
        .layer((
            TraceLayer::new_for_http(),
            // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
            // requests don't hang forever.
            TimeoutLayer::new(std::time::Duration::from_secs(10)),
        ));

    // Create axum listener
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();

    let cancellation_token = tokio_util::sync::CancellationToken::new();

    // Clone for collection task
    let data_cache_clone = data_cache.clone();
    let cancellation_token_clone = cancellation_token.clone();
    let handle = tokio::spawn(async move {
        let cancellation_token = cancellation_token_clone;
        let nodes = config.nodes;
        let mut semaphores = HashMap::<String, Arc<tokio::sync::Semaphore>>::new();
        for node in nodes.iter() {
            semaphores.insert(
                node.node_id.clone(),
                Arc::new(tokio::sync::Semaphore::const_new(1)),
            );
        }

        let data_cache = data_cache_clone;
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
        for node in nodes.iter() {
            let data_cache = data_cache.clone();
            let node = node.clone();
            let cancellation_token = cancellation_token.clone();
            tokio::spawn(async move {
                loop {
                    if cancellation_token.is_cancelled() {
                        log::info!("Collection task cancelled");
                        break;
                    }

                    let client = Client::new();
                    let response = client
                        .get(node.data_endpoint.clone())
                        .timeout(std::time::Duration::from_millis(10000))
                        .send()
                        .await;

                    match response {
                        Ok(response) => {
                            if response.status() == 200 {
                                log::debug!("Got response from {}", node.node_id);
                                match response.json::<LastDataResponse>().await {
                                    Ok(json) => {
                                        let mut data_cache = data_cache.write().await;
                                        data_cache.insert(node.node_id.clone(), Some(json.data));
                                    }
                                    Err(e) => {
                                        let mut data_cache = data_cache.write().await;
                                        data_cache.insert(node.node_id.clone(), None);
                                        log::error!(
                                            "Failed to parse response from {}: {}",
                                            node.node_id,
                                            e
                                        );
                                    }
                                }
                            } else {
                                let mut data_cache = data_cache.write().await;
                                data_cache.insert(node.node_id.clone(), None);
                            }
                        }
                        Err(e) => {
                            let mut data_cache = data_cache.write().await;
                            data_cache.insert(node.node_id.clone(), None);
                            log::debug!("Failed to get response from {}: {}", node.node_id, e);
                        }
                    }
                }
            });
        }
    });

    // Serve the app
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal(cancellation_token.clone()))
        .await
        .unwrap();

    // Wait for collection task to finish
    tokio::select! {
        _ = handle => {},
        _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
            log::error!("Collection task did not finish in time");
        }
    };

    log::info!("Server is shutting down");

    Ok(())
}

async fn shutdown_signal(token: CancellationToken) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    log::info!("Received shutdown signal");

    token.cancel();
}
