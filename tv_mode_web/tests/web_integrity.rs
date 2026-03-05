mod harness;

use harness::KodiMock;
use rocket::local::asynchronous::Client;
use rocket::http::Status;
use std::fs;
use std::env;
use std::time::Duration;
use tempfile::tempdir;

#[rocket::async_test]
async fn test_health_check() {
    let client = create_test_client(None).await;
    let response = client.get("/api/health").dispatch().await;
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.unwrap();
    assert!(body.contains("\"status\":\"success\""));
    assert!(body.contains("\"message\":\"API is healthy\""));
}

#[rocket::async_test]
async fn test_index_page() {
    let client = create_test_client(None).await;
    let response = client.get("/").dispatch().await;
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.unwrap();
    // Check for some content that should be in index.html.j2
    assert!(body.contains("<title>"));
    assert!(body.contains("TV Mode"));
}

#[rocket::async_test]
async fn test_jukectl_page() {
    let client = create_test_client(None).await;
    let response = client.get("/jukectl").dispatch().await;
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.unwrap();
    // Check for some content that should be in jukectl.html.j2
    assert!(body.contains("Jukebox"));
}

#[rocket::async_test]
async fn test_resilience_kodi_unreachable() {
    // Use an unreachable port
    let client = create_test_client(Some("http://127.0.0.1:1")).await;
    let response = client.get("/api/status").dispatch().await;

    assert_eq!(response.status(), Status::Ok);
    let body: serde_json::Value = response.into_json().await.unwrap();
    assert_eq!(body["status"], "error");
    // The error message might vary based on the OS/environment,
    // but it should contain something about the connection failure or RPC error.
    assert!(body["error_details"].as_str().is_some());
}

#[rocket::async_test]
async fn test_resilience_kodi_timeout() {
    let mut mock = KodiMock::new().await;
    // RPC_TIMEOUT is 5s, so we delay for 6s
    let _m = mock.mock_timeout(Duration::from_secs(6)).await;

    let client = create_test_client(Some(&mock.url())).await;
    let response = client.get("/api/status").dispatch().await;

    assert_eq!(response.status(), Status::Ok);
    let body: serde_json::Value = response.into_json().await.unwrap();
    assert_eq!(body["status"], "error");
    assert!(body["error_details"].as_str().unwrap().contains("timed out"));
}

#[rocket::async_test]
async fn test_resilience_kodi_malformed_json() {
    let mut mock = KodiMock::new().await;
    let _m = mock.mock_malformed().await;

    let client = create_test_client(Some(&mock.url())).await;
    let response = client.get("/api/status").dispatch().await;

    assert_eq!(response.status(), Status::Ok);
    let body: serde_json::Value = response.into_json().await.unwrap();
    assert_eq!(body["status"], "error");
    // Should be an error message about parsing
    assert!(body["error_details"].as_str().is_some());
}

async fn create_test_client(kodi_url: Option<&str>) -> Client {
    // Set up a temporary config directory
    let tmp_dir = tempdir().expect("Failed to create temp dir");
    let config_dir = tmp_dir.path();
    
    // Create dummy config files
    let url = kodi_url.unwrap_or("http://localhost:8080");
    let config_yml = format!("url: {}\nusername: user\npassword: pass\n", url);
    let show_mappings_yml = "user1:\n  - Show 1\n  - Show 2\n";
    let jukectl_channels_yml = "channels:\n  - name: Channel 1\n    any: [\"tag1\"]\n";
    
    fs::write(config_dir.join("config.yml"), config_yml).unwrap();
    fs::write(config_dir.join("show_mappings.yml"), show_mappings_yml).unwrap();
    fs::write(config_dir.join("jukectl_channels.yml"), jukectl_channels_yml).unwrap();
    
    // Set CONFIG_DIR so app_state::initialize() finds the files
    env::set_var("CONFIG_DIR", config_dir.to_str().unwrap());
    // Also need JUKECTL_API_URL for jukectl route
    env::set_var("JUKECTL_API_URL", "http://localhost:8000");

    let rocket = tv_mode_web::build_rocket();
    Client::tracked(rocket).await.expect("valid rocket instance")
}
