use odysseus_engine::{api, extractor};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("odysseus_engine=info,tower_http=info")
        .init();

    // Initialize proxy configuration from environment variable
    if let Ok(proxies_env) = std::env::var("PROXIES") {
        let proxies: Vec<String> = proxies_env
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        if !proxies.is_empty() {
            extractor::init_proxy_config(proxies.clone());
            tracing::info!("🔄 Proxy rotation enabled with {} proxies", proxies.len());
        }
    }

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    tracing::info!("🚀 Odysseus API iniciando en http://{}", addr);

    let router = api::create_router();
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, router).await.unwrap();
}