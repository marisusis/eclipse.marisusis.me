use anyhow::Result;
use axum::extract::Path;
use axum::response::Html;
use axum::{http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fs;
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

async fn root() -> Html<String> {
    let file_content = fs::read_to_string("app/build/index.html")
        .unwrap_or_else(|_| String::from("Failed to read file"));

    Html(file_content)
}

async fn handler(Path(params): Path<Params>) -> impl IntoResponse {
    log::info!("Request for {}", params.node);

    let now = std::time::Instant::now();

    let client = reqwest::Client::new();
    match client
        .get(format!("http://{}:8003/last_data", params.node))
        .timeout(std::time::Duration::from_millis(1000))
        .send()
        .await
    {
        Ok(response) => {
            if response.status() == 200 {
                let json = response.json::<LastDataResponse>().await.unwrap();
                log::info!("Request for {} took {:?}", params.node, now.elapsed());
                return (StatusCode::OK, Json(Some(json.data)));
            }
        }
        Err(e) => {
            log::error!("Error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(None));
        }
    };

    println!("http://{}", params.node);
    return (StatusCode::INTERNAL_SERVER_ERROR, Json(None));
}

#[tokio::main(flavor = "multi_thread", worker_threads = 16)]
async fn main() -> Result<()> {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let router = Router::new()
        .nest_service("/", ServeDir::new("app/build"))
        .route("/api/data/:node", get(handler))
        .layer(tower_http::cors::CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, router).await.unwrap();

    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
struct Params {
    node: String,
}
