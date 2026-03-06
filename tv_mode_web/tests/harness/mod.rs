use mockito::{Server, Mock};
use serde_json::json;
use std::time::Duration;

use mockito::ServerGuard;

pub struct KodiMock {
    server: ServerGuard,
}

#[allow(dead_code)]
impl KodiMock {
    pub async fn new() -> Self {
        Self {
            server: Server::new_async().await,
        }
    }

    pub fn url(&self) -> String {
        self.server.url()
    }

    pub async fn mock_get_tv_shows(&mut self) -> Mock {
        self.server.mock("POST", "/jsonrpc")
            .match_body(mockito::Matcher::PartialJson(json!({"method": "VideoLibrary.GetTVShows"})))
            .with_header("content-type", "application/json")
            .with_body(json!({
                "id": 1,
                "jsonrpc": "2.0",
                "result": {
                    "tvshows": [
                        { "tvshowid": 1, "title": "The Office" },
                        { "tvshowid": 2, "title": "Breaking Bad" }
                    ],
                    "limits": { "end": 2, "start": 0, "total": 2 }
                }
            }).to_string())
            .create_async()
            .await
    }

    pub async fn mock_get_artists(&mut self) -> Mock {
        self.server.mock("POST", "/jsonrpc")
            .match_body(mockito::Matcher::PartialJson(json!({"method": "AudioLibrary.GetArtists"})))
            .with_header("content-type", "application/json")
            .with_body(json!({
                "id": 1,
                "jsonrpc": "2.0",
                "result": {
                    "artists": [
                        { "artistid": 1, "artist": "Daft Punk" },
                        { "artistid": 2, "artist": "Radiohead" }
                    ],
                    "limits": { "end": 2, "start": 0, "total": 2 }
                }
            }).to_string())
            .create_async()
            .await
    }

    pub async fn mock_get_albums(&mut self) -> Mock {
        self.server.mock("POST", "/jsonrpc")
            .match_body(mockito::Matcher::PartialJson(json!({"method": "AudioLibrary.GetAlbums"})))
            .with_header("content-type", "application/json")
            .with_body(json!({
                "id": 1,
                "jsonrpc": "2.0",
                "result": {
                    "albums": [
                        { "albumid": 1, "label": "Discovery" },
                        { "albumid": 2, "label": "Homework" }
                    ],
                    "limits": { "end": 2, "start": 0, "total": 2 }
                }
            }).to_string())
            .create_async()
            .await
    }

    pub async fn mock_get_songs(&mut self) -> Mock {
        self.server.mock("POST", "/jsonrpc")
            .match_body(mockito::Matcher::PartialJson(json!({"method": "AudioLibrary.GetSongs"})))
            .with_header("content-type", "application/json")
            .with_body(json!({
                "id": 1,
                "jsonrpc": "2.0",
                "result": {
                    "songs": [
                        { "songid": 1, "label": "One More Time" },
                        { "songid": 2, "label": "Digital Love" }
                    ],
                    "limits": { "end": 2, "start": 0, "total": 2 }
                }
            }).to_string())
            .create_async()
            .await
    }


    pub async fn mock_get_episodes(&mut self) -> Mock {
        self.server.mock("POST", "/jsonrpc")
            .match_body(mockito::Matcher::PartialJson(json!({"method": "VideoLibrary.GetEpisodes"})))
            .with_header("content-type", "application/json")
            .with_body(json!({
                "id": 1,
                "jsonrpc": "2.0",
                "result": {
                    "episodes": [
                        { "episodeid": 101, "title": "Pilot", "season": 1, "episode": 1 },
                        { "episodeid": 102, "title": "Diversity Day", "season": 1, "episode": 2 }
                    ],
                    "limits": { "end": 2, "start": 0, "total": 12 }
                }
            }).to_string())
            .create_async()
            .await
    }

    pub async fn mock_get_episode_details(&mut self) -> Mock {
        self.server.mock("POST", "/jsonrpc")
            .match_body(mockito::Matcher::PartialJson(json!({"method": "VideoLibrary.GetEpisodeDetails"})))
            .with_header("content-type", "application/json")
            .with_body(json!({
                "id": 1,
                "jsonrpc": "2.0",
                "result": {
                    "episodedetails": {
                        "file": "/media/tv/The Office/S01E01.mkv",
                        "label": "Pilot"
                    }
                }
            }).to_string())
            .create_async()
            .await
    }

    pub async fn mock_get_active_players_none(&mut self) -> Mock {
        self.server.mock("POST", "/jsonrpc")
            .match_body(mockito::Matcher::PartialJson(json!({"method": "Player.GetActivePlayers"})))
            .with_header("content-type", "application/json")
            .with_body(json!({
                "id": 1,
                "jsonrpc": "2.0",
                "result": []
            }).to_string())
            .create_async()
            .await
    }

    pub async fn mock_get_active_players_active(&mut self) -> Mock {
        self.server.mock("POST", "/jsonrpc")
            .match_body(mockito::Matcher::PartialJson(json!({"method": "Player.GetActivePlayers"})))
            .with_header("content-type", "application/json")
            .with_body(json!({
                "id": 1,
                "jsonrpc": "2.0",
                "result": [
                    { "playerid": 1, "type": "video" }
                ]
            }).to_string())
            .create_async()
            .await
    }

    pub async fn mock_player_open(&mut self) -> Mock {
        self.server.mock("POST", "/jsonrpc")
            .match_body(mockito::Matcher::PartialJson(json!({"method": "Player.Open"})))
            .with_header("content-type", "application/json")
            .with_body(json!({
                "id": 1,
                "jsonrpc": "2.0",
                "result": "OK"
            }).to_string())
            .create_async()
            .await
    }

    pub async fn mock_player_stop(&mut self) -> Mock {
        self.server.mock("POST", "/jsonrpc")
            .match_body(mockito::Matcher::PartialJson(json!({"method": "Player.Stop"})))
            .with_header("content-type", "application/json")
            .with_body(json!({
                "id": 1,
                "jsonrpc": "2.0",
                "result": "OK"
            }).to_string())
            .create_async()
            .await
    }

    pub async fn mock_timeout(&mut self, delay: Duration) -> Mock {
        self.server.mock("POST", "/jsonrpc")
            .with_chunked_body(move |w| {
                std::thread::sleep(delay);
                let _ = w.write_all(json!({
                    "id": 1,
                    "jsonrpc": "2.0",
                    "result": "OK"
                }).to_string().as_bytes());
                Ok(())
            })
            .with_header("content-type", "application/json")
            .create_async()
            .await
    }

    pub async fn mock_malformed(&mut self) -> Mock {
        self.server.mock("POST", "/jsonrpc")
            .with_header("content-type", "application/json")
            .with_body("invalid json { [")
            .create_async()
            .await
    }

    pub async fn mock_http_error(&mut self, status: usize) -> Mock {
        self.server.mock("POST", "/jsonrpc")
            .with_status(status)
            .create_async()
            .await
    }
}
