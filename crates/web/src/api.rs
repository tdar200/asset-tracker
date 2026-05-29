use gloo_net::http::Request;
use tracker_core::{Dataset, Item, NewItem, Valuation};

pub async fn fetch_data() -> Result<Dataset, String> {
    let resp = Request::get("/api/data").send().await.map_err(|e| e.to_string())?;
    resp.json::<Dataset>().await.map_err(|e| e.to_string())
}

pub async fn create_item(payload: &NewItem) -> Result<Item, String> {
    let resp = Request::post("/api/items")
        .json(payload).map_err(|e| e.to_string())?
        .send().await.map_err(|e| e.to_string())?;
    if !resp.ok() { return Err(error_body(resp).await); }
    resp.json::<Item>().await.map_err(|e| e.to_string())
}

pub async fn update_item(item: &Item) -> Result<(), String> {
    let resp = Request::put(&format!("/api/items/{}", item.id))
        .json(item).map_err(|e| e.to_string())?
        .send().await.map_err(|e| e.to_string())?;
    if resp.ok() { Ok(()) } else { Err(error_body(resp).await) }
}

pub async fn delete_item(id: &str) -> Result<(), String> {
    let resp = Request::delete(&format!("/api/items/{}", id))
        .send().await.map_err(|e| e.to_string())?;
    if resp.ok() { Ok(()) } else { Err(error_body(resp).await) }
}

pub async fn add_valuation(id: &str, valuation: &Valuation) -> Result<(), String> {
    let resp = Request::post(&format!("/api/items/{}/valuations", id))
        .json(valuation).map_err(|e| e.to_string())?
        .send().await.map_err(|e| e.to_string())?;
    if resp.ok() { Ok(()) } else { Err(error_body(resp).await) }
}

async fn error_body(resp: gloo_net::http::Response) -> String {
    resp.text().await.unwrap_or_else(|_| "request failed".into())
}
