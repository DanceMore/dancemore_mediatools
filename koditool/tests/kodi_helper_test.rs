use koditool::{Authorization, Config, RpcClient, SelectedEpisode};

use mockito::{mock, server_url, Mock};
use rand::prelude::IndexedMutRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha12Rng;
use serde_json::{json, Value};
use std::error::Error;

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a test configuration
    fn test_config() -> Config {
        Config {
            url: server_url(),
            username: "test_user".to_string(),
            password: "test_pass".to_string(),
        }
    }

    // Helper to create a client with mocked config
    fn test_client() -> RpcClient {
        let config = test_config();
        RpcClient::new(config).unwrap()
    }

    #[tokio::test]
    async fn test_authorization_header() {
        let auth = Authorization::new("test_user", "test_pass");
        let header_value = auth.auth_header_value().to_str().unwrap();
        
        // The expected header value is "Basic " + base64("test_user:test_pass")
        let expected = format!("Basic {}", base64::encode("test_user:test_pass"));
        assert_eq!(header_value, expected);
    }

    #[tokio::test]
    async fn test_rpc_call_success() {
        let _mock = mock("POST", "/jsonrpc")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"jsonrpc": "2.0", "id": 1, "result": {"success": true}}"#)
            .create();

        let client = test_client();
        let params = json!({"jsonrpc": "2.0", "method": "test", "id": 1});
        
        let result = client.rpc_call(&params).await.unwrap();
        assert_eq!(result["result"]["success"], json!(true));
    }

    #[tokio::test]
    async fn test_rpc_call_error() {
        let _mock = mock("POST", "/jsonrpc")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"jsonrpc": "2.0", "id": 1, "error": {"code": -32601, "message": "Method not found"}}"#)
            .create();

        let client = test_client();
        let params = json!({"jsonrpc": "2.0", "method": "invalid_method", "id": 1});
        
        let result = client.rpc_call(&params).await.unwrap();
        assert_eq!(result["error"]["code"], json!(-32601));
        assert_eq!(result["error"]["message"], json!("Method not found"));
    }

    #[tokio::test]
    async fn test_select_random_episode_by_title() {
        // Mock for GetTVShows
        let tv_shows_mock = mock("POST", "/jsonrpc")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "tvshows": [
                        {"tvshowid": 1, "title": "Friends"},
                        {"tvshowid": 2, "title": "Breaking Bad"}
                    ],
                    "limits": {"start": 0, "end": 2, "total": 2}
                }
            }"#)
            .expect(1)
            .create();

        // Mock for GetEpisodes
        let episodes_mock = mock("POST", "/jsonrpc")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "episodes": [
                        {"episodeid": 101, "title": "Pilot", "season": 1, "episode": 1},
                        {"episodeid": 102, "title": "Second Episode", "season": 1, "episode": 2}
                    ],
                    "limits": {"start": 0, "end": 2, "total": 2}
                }
            }"#)
            .expect(1)
            .create();

        // Mock for GetEpisodeDetails
        let episode_details_mock = mock("POST", "/jsonrpc")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "episodedetails": {
                        "file": "/path/to/episode.mp4"
                    }
                }
            }"#)
            .expect(1)
            .create();

        let client = test_client();
        
        // Note: This test will always select episode 101 with our fixed seed
        let result = client.select_random_episode_by_title("Friends").await.unwrap();
        
        assert_eq!(result.episode_id, 101);
        assert_eq!(result.episode_file_path, "/path/to/episode.mp4");
        
        // Verify all mocks were called
        tv_shows_mock.assert();
        episodes_mock.assert();
        episode_details_mock.assert();
    }

    #[tokio::test]
    async fn test_random_episode_selection() {
        // This test verifies that the random selection works properly
        // We'll create a fixed seed for reproducibility
        
        // Create a vec of episode IDs that matches the one in the code
        let mut episode_ids: Vec<u64> = vec![101, 102, 103, 104];
        
        // Create a fixed RNG with the same seed as in the code
        let mut rng = ChaCha12Rng::from_seed(Default::default());
        
        // Select a random episode ID
        let random_episode_id = episode_ids.choose_mut(&mut rng).unwrap();
        
        // With our fixed seed, we should consistently get the same episode
        assert_eq!(*random_episode_id, 102);
    }

    #[tokio::test]
    async fn test_play_episode() {
        let _mock = mock("POST", "/jsonrpc")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"jsonrpc": "2.0", "id": 1, "result": "OK"}"#)
            .create();

        let client = test_client();
        let episode = SelectedEpisode {
            episode_id: 101,
            episode_file_path: "/path/to/test_episode.mp4".to_string(),
        };
        
        let result = client.rpc_play(&episode).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_stop_playback() {
        let _mock = mock("POST", "/jsonrpc")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"jsonrpc": "2.0", "id": 1, "result": "OK"}"#)
            .create();

        let client = test_client();
        let result = client.rpc_stop().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_is_active_with_players() {
        let _mock = mock("POST", "/jsonrpc")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "jsonrpc": "2.0",
                "id": 1,
                "result": [{"playerid": 1, "type": "video"}]
            }"#)
            .create();

        let client = test_client();
        let result = client.is_active().await.unwrap();
        assert_eq!(result, true);
    }

    #[tokio::test]
    async fn test_is_active_without_players() {
        let _mock = mock("POST", "/jsonrpc")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"jsonrpc": "2.0", "id": 1, "result": []}"#)
            .create();

        let client = test_client();
        let result = client.is_active().await.unwrap();
        assert_eq!(result, false);
    }

    #[tokio::test]
    async fn test_tv_show_not_found() {
        let _mock = mock("POST", "/jsonrpc")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "tvshows": [
                        {"tvshowid": 1, "title": "Friends"}
                    ],
                    "limits": {"start": 0, "end": 1, "total": 1}
                }
            }"#)
            .create();

        let client = test_client();
        let result = client.select_random_episode_by_title("Game of Thrones").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("TV show Game of Thrones not found"));
    }

    #[tokio::test]
    async fn test_no_episodes_available() {
        // Mock for GetTVShows
        let _tv_shows_mock = mock("POST", "/jsonrpc")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "tvshows": [
                        {"tvshowid": 1, "title": "Friends"}
                    ],
                    "limits": {"start": 0, "end": 1, "total": 1}
                }
            }"#)
            .create();

        // Mock for GetEpisodes with empty result
        let _episodes_mock = mock("POST", "/jsonrpc")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "episodes": [],
                    "limits": {"start": 0, "end": 0, "total": 0}
                }
            }"#)
            .create();

        let client = test_client();
        let result = client.select_random_episode_by_title("Friends").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No episodes available"));
    }

    #[tokio::test]
    async fn test_http_error() {
        let _mock = mock("POST", "/jsonrpc")
            .with_status(500)
            .with_header("content-type", "application/json")
            .with_body(r#"{"error": "Internal Server Error"}"#)
            .create();

        let client = test_client();
        let params = json!({"jsonrpc": "2.0", "method": "test", "id": 1});
        
        let result = client.rpc_call(&params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invalid_json_response() {
        let _mock = mock("POST", "/jsonrpc")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("This is not valid JSON")
            .create();

        let client = test_client();
        let params = json!({"jsonrpc": "2.0", "method": "test", "id": 1});
        
        let result = client.rpc_call(&params).await;
        assert!(result.is_err());
    }
}
