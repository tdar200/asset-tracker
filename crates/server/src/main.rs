use std::path::PathBuf;

#[tokio::main]
async fn main() {
    let data_path = std::env::var("ASSET_TRACKER_DATA").unwrap_or_else(|_| "data.json".into());
    let dist = std::env::var("ASSET_TRACKER_DIST").unwrap_or_else(|_| "dist".into());
    let port: u16 = std::env::var("ASSET_TRACKER_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(4000);

    let app = tracker_server::build_router(data_path, Some(PathBuf::from(dist)));
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port)).await.unwrap();
    println!("asset-tracker listening on http://127.0.0.1:{port}");
    axum::serve(listener, app).await.unwrap();
}
