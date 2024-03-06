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
use tokio::sync::RwLock;
use tower_http::services::ServeDir;

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
struct NodeEntry {
    node_id: String,
    location: String,
    last_update: i64,
    data: Option<DataPoint>,
}

struct AppState {
    data_cache: Arc<RwLock<BTreeMap<String, Option<DataPoint>>>>,
}

async fn all_data_handler(
    State(app_state): State<Arc<AppState>>,
) -> Result<Json<TimeDataResponse>, StatusCode> {
    let data_cache = app_state.data_cache.read().await;

    let mut nodes = Vec::new();
    for (node_id, data) in data_cache.iter() {
        match data {
            Some(data) => {
                nodes.push(NodeEntry {
                    node_id: node_id.clone(),
                    location: "Unknown".to_string(),
                    last_update: data.timestamp.unwrap_or(0),
                    data: Some(data.clone()),
                });
            }
            None => {
                nodes.push(NodeEntry {
                    node_id: node_id.clone(),
                    location: "Unknown".to_string(),
                    last_update: 0,
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

    let mut node_id = params.node.clone();
    node_id.make_ascii_uppercase();
    if let Some(data) = data_cache.get(&node_id) {
        if let Some(data) = data {
            return Ok(Json(NodeEntry {
                node_id: node_id,
                location: "Earth".to_string(),
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

    // Load config file
    let node_file_contents = fs::read_to_string("config.toml")?;
    let config: ConfigFile = toml::from_str(&node_file_contents)?;

    // Create ET data cache
    let mut data_cache = BTreeMap::<String, Option<DataPoint>>::new();

    // initialize cache with None
    for node in config.nodes.iter() {
        data_cache.insert(node.node_id.clone(), None);
    }

    // Move to multi-thread/access capable storage
    let data_cache = std::sync::Arc::new(tokio::sync::RwLock::new(data_cache));

    // Create app state
    let app_state = AppState {
        data_cache: data_cache.clone(),
    };

    let app_state = Arc::new(app_state);

    // Create axum router
    let router = Router::new()
        .nest_service("/", ServeDir::new("app/build"))
        .route("/api/data/all", get(all_data_handler))
        .route("/api/data/:node", get(node_handler))
        .with_state(app_state)
        .layer(tower_http::cors::CorsLayer::permissive());

    // Create axum listener
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();

    // Clone for collection task
    let data_cache_clone = data_cache.clone();
    tokio::spawn(async move {
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
        loop {
            for node in nodes.iter() {
                let semaphore = semaphores.get(&node.node_id).unwrap();
                let data_cache = data_cache.clone();
                let node = node.clone();
                match semaphore.try_acquire() {
                    Ok(_) => {
                        let semaphore = semaphore.clone();
                        tokio::spawn(async move {
                            let _permit = semaphore.clone().acquire_owned().await.unwrap();
                            let client = Client::new();
                            let response = client
                                .get(node.data_endpoint.clone())
                                .timeout(std::time::Duration::from_millis(800))
                                .send()
                                .await;
                            match response {
                                Ok(response) => {
                                    if response.status() == 200 {
                                        log::debug!("Got response from {}", node.node_id);
                                        let json =
                                            response.json::<LastDataResponse>().await.unwrap();
                                        let mut data_cache = data_cache.write().await;
                                        data_cache.insert(node.node_id.clone(), Some(json.data));
                                    } else {
                                        let mut data_cache = data_cache.write().await;
                                        data_cache.insert(node.node_id.clone(), None);
                                    }
                                }
                                Err(e) => {
                                    let mut data_cache = data_cache.write().await;
                                    data_cache.insert(node.node_id.clone(), None);
                                }
                            }
                        });
                    }
                    Err(_) => {}
                }
            }

            // Wait for a bit before pinging again
            interval.tick().await;
        }
    });

    // Serve the app
    axum::serve(listener, router).await.unwrap();

    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
struct Params {
    node: String,
}
