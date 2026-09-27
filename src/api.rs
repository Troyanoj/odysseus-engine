use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::env;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::warn;

use crate::extractor::{extract_multiple_page_data, extract_page_data, PageData};

#[derive(Deserialize)]
pub struct ExtractRequest {
    pub url: Option<String>,
    pub urls: Option<Vec<String>>,
}

#[derive(Serialize)]
pub struct ExtractResponse {
    pub success: bool,
    pub data: Option<Vec<Result<PageData, String>>>,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub cache_capacity: u64,
}

const DEFAULT_API_KEY: &str = "dev-key-123";

async fn require_api_key(
    State(_state): State<()>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let api_key = env::var("API_KEY").unwrap_or_else(|_| DEFAULT_API_KEY.to_string());
    
    let provided_key = headers
        .get("x-api-key")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    
    if provided_key != api_key {
        warn!("Unauthorized API access attempt");
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "success": false,
                "error": "Invalid or missing API key"
            })),
        ));
    }
    
    Ok(next.run(request).await)
}

async fn health_handler() -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: "1.0.0".to_string(),
        cache_capacity: 10_000,
    })
}

async fn extract_handler(
    Json(payload): Json<ExtractRequest>,
) -> impl IntoResponse {
    if payload.url.is_none() && payload.urls.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ExtractResponse {
                success: false,
                data: None,
                error: Some("Either 'url' or 'urls' must be provided".to_string()),
            }),
        )
            .into_response();
    }

    if let Some(url) = payload.url {
        match extract_page_data(&url).await {
            Ok(data) => (
                StatusCode::OK,
                Json(ExtractResponse {
                    success: true,
                    data: Some(vec![Ok(data)]),
                    error: None,
                }),
            )
                .into_response(),
            Err(e) => {
                let error_msg = e.to_string();
                let status = if error_msg.contains("404") || error_msg.contains("404 Not Found") {
                    StatusCode::NOT_FOUND
                } else if error_msg.contains("timeout") || error_msg.contains("timed out") {
                    StatusCode::GATEWAY_TIMEOUT
                } else {
                    StatusCode::BAD_GATEWAY
                };
                (
                    status,
                    Json(ExtractResponse {
                        success: false,
                        data: None,
                        error: Some(error_msg),
                    }),
                )
                    .into_response()
            }
        }
    } else if let Some(urls) = payload.urls {
        if urls.is_empty() {
            return (
                StatusCode::BAD_REQUEST,
                Json(ExtractResponse {
                    success: false,
                    data: None,
                    error: Some("URLs array cannot be empty".to_string()),
                }),
            )
                .into_response();
        }
        
        match extract_multiple_page_data(urls).await {
            Ok(results) => {
                let string_results: Vec<Result<PageData, String>> = results
                    .into_iter()
                    .map(|r| r.map_err(|e| e.to_string()))
                    .collect();
                (
                    StatusCode::OK,
                    Json(ExtractResponse {
                        success: true,
                        data: Some(string_results),
                        error: None,
                    }),
                )
                    .into_response()
            }
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ExtractResponse {
                    success: false,
                    data: None,
                    error: Some(format!("Batch extraction failed: {}", e)),
                }),
            )
                .into_response(),
        }
    } else {
        unreachable!()
    }
}

pub fn create_router() -> Router {
    let api_routes = Router::new()
        .route("/v1/extract", post(extract_handler))
        .layer(middleware::from_fn_with_state((), require_api_key));

    Router::new()
        .route("/health", get(health_handler))
        .merge(api_routes)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}