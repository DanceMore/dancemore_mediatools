mod harness;

use rocket::local::asynchronous::Client;
use rocket::http::Status;
use std::fs;
use std::env;
use tempfile::tempdir;

async fn setup_config_dir(config_dir: &std::path::Path) {
    let config_yml = "url: http://localhost:8080\nusername: user\npassword: pass\n";
    let show_mappings_yml = "user1:\n  - Show 1\n";

    fs::write(config_dir.join("config.yml"), config_yml).unwrap();
    fs::write(config_dir.join("show_mappings.yml"), show_mappings_yml).unwrap();
    // jukectl_channels.yml is optional
}

fn set_config_dir(path: &std::path::Path) {
    env::set_var("CONFIG_DIR", path.to_str().unwrap());
}

#[rocket::async_test]
async fn test_state_persistence_across_restarts() {
    let tmp_dir = tempdir().expect("Failed to create temp dir");
    let config_dir = tmp_dir.path();
    setup_config_dir(config_dir).await;
    set_config_dir(config_dir);

    // 1. Start first server instance and enable TV mode
    {
        let rocket = tv_mode_web::build_rocket();
        let client = Client::tracked(rocket).await.expect("valid rocket instance");

        let response = client.post("/api/play/user1").dispatch().await;
        assert_eq!(response.status(), Status::Ok);

        // Give it a moment to ensure file is written if it's async (though we await it)
        rocket::tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let body: serde_json::Value = response.into_json().await.unwrap();
        assert_eq!(body["status"], "success");
        assert_eq!(body["tv_mode"]["active"], true);
        assert_eq!(body["tv_mode"]["user"], "user1");

        // Verify persistent_state.json exists
        assert!(config_dir.join("persistent_state.json").exists());
    }

    // 2. Start second server instance and verify state is restored
    {
        let rocket = tv_mode_web::build_rocket();
        let client = Client::tracked(rocket).await.expect("valid rocket instance");

        let response = client.get("/api/status").dispatch().await;
        assert_eq!(response.status(), Status::Ok);

        let body: serde_json::Value = response.into_json().await.unwrap();
        // Even if Kodi is unreachable, tv_mode state should be restored
        assert_eq!(body["tv_mode"]["active"], true, "TV mode should be active after restart. Body: {:?}", body);
        assert_eq!(body["tv_mode"]["user"], "user1");
    }
}

#[rocket::async_test]
async fn test_state_persistence_stop_tv_mode() {
    let tmp_dir = tempdir().expect("Failed to create temp dir");
    let config_dir = tmp_dir.path();
    setup_config_dir(config_dir).await;
    set_config_dir(config_dir);

    // 1. Enable TV mode
    {
        let rocket = tv_mode_web::build_rocket();
        let client = Client::tracked(rocket).await.expect("valid rocket instance");
        client.post("/api/play/user1").dispatch().await;
    }

    // 2. Stop TV mode in a new instance
    {
        let rocket = tv_mode_web::build_rocket();
        let client = Client::tracked(rocket).await.expect("valid rocket instance");

        let response = client.post("/api/stop").dispatch().await;
        assert_eq!(response.status(), Status::Ok);
    }

    // 3. Verify it stays stopped in a third instance
    {
        let rocket = tv_mode_web::build_rocket();
        let client = Client::tracked(rocket).await.expect("valid rocket instance");

        let response = client.get("/api/status").dispatch().await;
        let body: serde_json::Value = response.into_json().await.unwrap();
        assert_eq!(body["tv_mode"]["active"], false);
        assert_eq!(body["tv_mode"]["user"], serde_json::Value::Null);
    }
}
