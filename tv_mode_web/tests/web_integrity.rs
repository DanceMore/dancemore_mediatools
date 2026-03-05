use rocket::local::asynchronous::Client;
use rocket::http::Status;
use std::fs;
use std::env;
use tempfile::tempdir;

#[rocket::async_test]
async fn test_health_check() {
    let client = create_test_client().await;
    let response = client.get("/api/health").dispatch().await;
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.unwrap();
    assert!(body.contains("\"status\":\"success\""));
    assert!(body.contains("\"message\":\"API is healthy\""));
}

#[rocket::async_test]
async fn test_index_page() {
    let client = create_test_client().await;
    let response = client.get("/").dispatch().await;
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.unwrap();
    // Check for some content that should be in index.html.j2
    assert!(body.contains("<title>"));
    assert!(body.contains("TV Mode"));
}

#[rocket::async_test]
async fn test_jukectl_page() {
    let client = create_test_client().await;
    let response = client.get("/jukectl").dispatch().await;
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.unwrap();
    // Check for some content that should be in jukectl.html.j2
    assert!(body.contains("Jukebox"));
}

async fn create_test_client() -> Client {
    // Set up a temporary config directory
    let tmp_dir = tempdir().expect("Failed to create temp dir");
    let config_dir = tmp_dir.path();
    
    // Create dummy config files
    let config_yml = "url: http://localhost:8080\nusername: user\npassword: pass\n";
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
