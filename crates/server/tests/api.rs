use axum_test::TestServer;
use serde_json::json;
use tracker_core::Dataset;

fn server() -> (tempfile::TempDir, TestServer) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.json");
    let app = tracker_server::build_router(path, None);
    (dir, TestServer::new(app).unwrap())
}

#[tokio::test]
async fn empty_dataset_is_served() {
    let (_d, server) = server();
    let res = server.get("/api/data").await;
    res.assert_status_ok();
    let data: Dataset = res.json();
    assert!(data.items.is_empty());
}

#[tokio::test]
async fn create_then_list_item() {
    let (_d, server) = server();
    let created = server.post("/api/items").json(&json!({
        "name": "Camera", "category": "Electronics", "details": "", "condition": "Good",
        "currency": "GBP", "purchasePrice": 900.0, "purchaseDate": "2024-02-02", "notes": ""
    })).await;
    created.assert_status_ok();
    let data: Dataset = server.get("/api/data").await.json();
    assert_eq!(data.items.len(), 1);
    assert_eq!(data.items[0].name, "Camera");
    assert!(!data.items[0].id.is_empty());
}

#[tokio::test]
async fn create_rejects_blank_name() {
    let (_d, server) = server();
    let res = server.post("/api/items").json(&json!({
        "name": "", "category": "", "details": "", "condition": "",
        "currency": "GBP", "purchasePrice": 1.0, "purchaseDate": "2024-01-01", "notes": ""
    })).await;
    res.assert_status(axum::http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn add_valuation_to_item() {
    let (_d, server) = server();
    let created: tracker_core::Item = server.post("/api/items").json(&json!({
        "name": "Bike", "category": "Vehicles", "details": "", "condition": "Fair",
        "currency": "GBP", "purchasePrice": 1200.0, "purchaseDate": "2021-06-01", "notes": ""
    })).await.json();
    let res = server.post(&format!("/api/items/{}/valuations", created.id)).json(&json!({
        "date": "2026-05-29", "low": 600.0, "high": 800.0, "best": 700.0,
        "confidence": "medium", "sources": [], "comment": ""
    })).await;
    res.assert_status_ok();
    let data: Dataset = server.get("/api/data").await.json();
    assert_eq!(data.items[0].valuations.len(), 1);
    assert_eq!(data.items[0].valuations[0].best, 700.0);
}

#[tokio::test]
async fn delete_item() {
    let (_d, server) = server();
    let created: tracker_core::Item = server.post("/api/items").json(&json!({
        "name": "Ring", "category": "Luxury", "details": "", "condition": "Good",
        "currency": "GBP", "purchasePrice": 300.0, "purchaseDate": "2020-01-01", "notes": ""
    })).await.json();
    server.delete(&format!("/api/items/{}", created.id)).await.assert_status_ok();
    let data: Dataset = server.get("/api/data").await.json();
    assert!(data.items.is_empty());
}
