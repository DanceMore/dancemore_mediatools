use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use rocket::Route;
use rocket::State;
use rocket_dyn_templates::Template;
use serde::Serialize;
use std::env;

use crate::app_state::{AppState, JukectlChannel};

#[derive(Serialize)]
struct JukectlContext {
    jukectl_api_url: String,
    channels: Vec<JukectlChannel>,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

type ApiResponse<T> = Result<Json<T>, Custom<Json<ErrorResponse>>>;

#[get("/jukectl")]
pub async fn jukectl_page(app_state: &State<AppState>) -> Template {
    // Get the jukectl API URL from environment variable
    let jukectl_api_url = env::var("JUKECTL_API_URL")
        .unwrap_or_else(|_| "http://localhost:8000".to_string());
    
    // Get the channels from app state
    let channels = app_state.jukectl_channels.read().await.clone();
    
    let context = JukectlContext { 
        jukectl_api_url,
        channels,
    };
    Template::render("jukectl", &context)
}

// Proxy: Get tags/status
#[get("/jukectl/proxy/tags")]
pub async fn proxy_get_tags() -> ApiResponse<serde_json::Value> {
    let jukectl_url = env::var("JUKECTL_API_URL")
        .unwrap_or_else(|_| "http://localhost:8000".to_string());
    
    match reqwest::get(format!("{}/tags", jukectl_url)).await {
        Ok(resp) if resp.status().is_success() => {
            match resp.json().await {
                Ok(data) => Ok(Json(data)),
                Err(e) => Err(Custom(Status::InternalServerError, 
                    Json(ErrorResponse { error: format!("Parse error: {}", e) }))),
            }
        }
        Ok(resp) => Err(Custom(Status::BadGateway,
            Json(ErrorResponse { error: format!("Backend error: {}", resp.status()) }))),
        Err(e) => Err(Custom(Status::ServiceUnavailable,
            Json(ErrorResponse { error: format!("Connection error: {}", e) }))),
    }
}

// Proxy: Get current queue/now playing
#[get("/jukectl/proxy/queue")]
pub async fn proxy_get_queue() -> ApiResponse<serde_json::Value> {
    let jukectl_url = env::var("JUKECTL_API_URL")
        .unwrap_or_else(|_| "http://localhost:8000".to_string());
    
    // Use the /queue endpoint which gives us head, tail, and length
    match reqwest::get(format!("{}/queue", jukectl_url)).await {
        Ok(resp) if resp.status().is_success() => {
            match resp.json().await {
                Ok(data) => Ok(Json(data)),
                Err(e) => Err(Custom(Status::InternalServerError, 
                    Json(ErrorResponse { error: format!("Parse error: {}", e) }))),
            }
        }
        Ok(resp) => Err(Custom(Status::BadGateway,
            Json(ErrorResponse { error: format!("Backend error: {}", resp.status()) }))),
        Err(e) => Err(Custom(Status::ServiceUnavailable,
            Json(ErrorResponse { error: format!("Connection error: {}", e) }))),
    }
}

// Proxy: Skip song
#[post("/jukectl/proxy/skip")]
pub async fn proxy_skip() -> ApiResponse<serde_json::Value> {
    let jukectl_url = env::var("JUKECTL_API_URL")
        .unwrap_or_else(|_| "http://localhost:8000".to_string());
    
    let client = reqwest::Client::new();
    match client.post(format!("{}/skip", jukectl_url)).send().await {
        Ok(resp) if resp.status().is_success() => {
            match resp.json().await {
                Ok(data) => Ok(Json(data)),
                Err(e) => Err(Custom(Status::InternalServerError,
                    Json(ErrorResponse { error: format!("Parse error: {}", e) }))),
            }
        }
        Ok(resp) => Err(Custom(Status::BadGateway,
            Json(ErrorResponse { error: format!("Backend error: {}", resp.status()) }))),
        Err(e) => Err(Custom(Status::ServiceUnavailable,
            Json(ErrorResponse { error: format!("Connection error: {}", e) }))),
    }
}

// Proxy: Toggle album mode
#[post("/jukectl/proxy/album-mode/toggle")]
pub async fn proxy_toggle_album() -> ApiResponse<serde_json::Value> {
    let jukectl_url = env::var("JUKECTL_API_URL")
        .unwrap_or_else(|_| "http://localhost:8000".to_string());
    
    let client = reqwest::Client::new();
    match client.post(format!("{}/album-mode/toggle", jukectl_url)).send().await {
        Ok(resp) if resp.status().is_success() => {
            match resp.json().await {
                Ok(data) => Ok(Json(data)),
                Err(e) => Err(Custom(Status::InternalServerError,
                    Json(ErrorResponse { error: format!("Parse error: {}", e) }))),
            }
        }
        Ok(resp) => Err(Custom(Status::BadGateway,
            Json(ErrorResponse { error: format!("Backend error: {}", resp.status()) }))),
        Err(e) => Err(Custom(Status::ServiceUnavailable,
            Json(ErrorResponse { error: format!("Connection error: {}", e) }))),
    }
}

// Proxy: Update tags
#[post("/jukectl/proxy/tags", data = "<tags>")]
pub async fn proxy_update_tags(tags: Json<serde_json::Value>) -> ApiResponse<serde_json::Value> {
    let jukectl_url = env::var("JUKECTL_API_URL")
        .unwrap_or_else(|_| "http://localhost:8000".to_string());
    
    let client = reqwest::Client::new();
    match client.post(format!("{}/tags", jukectl_url))
        .json(&tags.into_inner())
        .send().await {
        Ok(resp) if resp.status().is_success() => {
            match resp.json().await {
                Ok(data) => Ok(Json(data)),
                Err(e) => Err(Custom(Status::InternalServerError,
                    Json(ErrorResponse { error: format!("Parse error: {}", e) }))),
            }
        }
        Ok(resp) => Err(Custom(Status::BadGateway,
            Json(ErrorResponse { error: format!("Backend error: {}", resp.status()) }))),
        Err(e) => Err(Custom(Status::ServiceUnavailable,
            Json(ErrorResponse { error: format!("Connection error: {}", e) }))),
    }
}

// Return routes defined in this module
pub fn routes() -> Vec<Route> {
    routes![
        jukectl_page,
        proxy_get_tags,
        proxy_skip,
        proxy_toggle_album,
        proxy_update_tags
    ]
}
