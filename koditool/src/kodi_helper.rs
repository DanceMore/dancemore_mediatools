use rand::prelude::IndexedMutRandom;
use rand::rng;
use rand::SeedableRng;
use rand::TryRngCore;
use rand_chacha::ChaCha12Rng;
use reqwest::header::AUTHORIZATION;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use serde_json::Value;
use std::error::Error;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub url: String,
    pub username: String,
    pub password: String,
}

impl Config {
    pub fn load(filename: &str) -> Result<Self, Box<dyn Error>> {
        let config_content = fs::read_to_string(filename)?;
        let config: Config = serde_yaml::from_str(&config_content)?;
        Ok(config)
    }
}

#[derive(Debug)]
pub struct Authorization {
    value: HeaderValue,
}

impl Authorization {
    pub fn new(username: &str, password: &str) -> Self {
        let auth_header_value = format!(
            "Basic {}",
            base64::encode(format!("{}:{}", username, password))
        )
        .parse()
        .expect("failed to create Authorization header");

        Authorization {
            value: auth_header_value,
        }
    }

    pub fn auth_header_value(&self) -> &HeaderValue {
        &self.value
    }
}

// individual found Episode Struct
#[derive(Clone, Debug)]
pub struct SelectedEpisode {
    pub episode_id: u64,
    pub episode_file_path: String,
}

// Implement Display for default {} formatting
impl std::fmt::Display for SelectedEpisode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.episode_file_path)
    }
}

// Define a struct to be the basis of our RPC Client re-use
pub struct RpcClient {
    pub config: Config,
    pub auth: Authorization,
}

impl RpcClient {
    // Create a new instance of RpcClient
    pub fn new(config: Config) -> Result<Self, Box<dyn Error>> {
        let auth = Authorization::new(&config.username, &config.password);
        Ok(RpcClient { auth, config })
    }

    pub async fn select_random_episode_by_title(
        &self,
        tv_show_name: &str,
    ) -> Result<SelectedEpisode, Box<dyn Error>> {
        // Fetch the list of TV shows
        let tv_shows_request_params = json!({
            "jsonrpc": "2.0",
            "method": "VideoLibrary.GetTVShows",
            "params": {
                "properties": ["title"],
                "limits": { "start": 0, "end": 1000 }
            },
            "id": 1
        });

        let tv_shows_response_json = self.rpc_call(&tv_shows_request_params).await?;

        // Extract the "tvshows" array from the "result" field
        let tv_shows = tv_shows_response_json["result"]["tvshows"]
            .as_array()
            .ok_or("TV shows not found in response")?;

        // Find the TV show with the given name
        let tv_show = tv_shows
            .iter()
            .find(|show| show["title"].as_str() == Some(tv_show_name))
            .ok_or_else(|| format!("TV show {} not found", tv_show_name))?;

        println!("Selected TV Show: {:?}", tv_show);

        let tv_show_id = tv_show["tvshowid"].as_u64().ok_or("TV show ID not found")?;
        println!("Selected TV Show ID: {}", tv_show_id);

        // Fetch the list of episodes
        let episodes_request_params = json!({
            "jsonrpc": "2.0",
            "method": "VideoLibrary.GetEpisodes",
            "params": {
                "tvshowid": tv_show_id, // Use the TV show ID you obtained earlier
                "properties": ["title", "season", "episode"],
                "limits": { "start": 0, "end": 1000 }
            },
            "id": 1
        });

        let episodes_response_json = self.rpc_call(&episodes_request_params).await?;

        //println!("Episodes Response: {:?}", episodes_response_json);

        // Extract the "episodes" array from the "result" field
        let episodes = episodes_response_json["result"]["episodes"]
            .as_array()
            .ok_or("Episodes not found in response")?;

        //for episode in episodes {
        //        let episode_id = episode["episodeid"].as_u64().ok_or("Episode ID not found")?;
        //        let episode_title = episode["title"].as_str().ok_or("Episode title not found")?;
        //        let season_number = episode["season"].as_u64().ok_or("Season number not found")?;
        //        let episode_number = episode["episode"].as_u64().ok_or("Episode number not found")?;

        //        println!(
        //    	    "Episode ID: {}, Title: {}, Season: {}, Episode: {}",
        //    	    episode_id, episode_title, season_number, episode_number
        //    	    );
        //}

        // Extract the episode IDs from the episodes array
        let mut episode_ids: Vec<u64> = episodes
            .iter()
            .map(|episode| episode["episodeid"].as_u64().unwrap())
            .collect();

        // Randomly select an episode ID

        let mut seed_array = [0u8; 32];
        let _ = rng().try_fill_bytes(&mut seed_array);

        let mut rng = ChaCha12Rng::from_seed(seed_array);
        let random_episode_id = episode_ids
            .choose_mut(&mut rng)
            .ok_or("No episodes available")?;

        //println!("Randomly selected episode ID: {:?}", random_episode_id);

        // Prepare the request parameters
        let episode_details_request_params = json!({
            "jsonrpc": "2.0",
            "method": "VideoLibrary.GetEpisodeDetails",
            "params": {
                "episodeid": random_episode_id,
                "properties": ["file"] // You can also include other properties you need
            },
            "id": 1
        });

        // Make the RPC call
        let episode_details_response_json = self.rpc_call(&episode_details_request_params).await?;

        // Extract the episode file path from the response
        let episode_file_path = episode_details_response_json["result"]["episodedetails"]["file"]
            .as_str()
            .ok_or("Episode file path not found in response")?
            .to_string(); // Convert to String

        //println!("[!] file path => {:?}", episode_file_path);

        let selected_episode = SelectedEpisode {
            episode_id: *random_episode_id,
            episode_file_path,
        };

        Ok(selected_episode)
    }

    pub async fn rpc_call(&self, request_params: &Value) -> Result<Value, Box<dyn Error>> {
        let client = Client::new();

        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, self.auth.auth_header_value().clone());
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        let url = format!("{}/jsonrpc", self.config.url);

        // Serialize the request params to a JSON string
        let json_body = serde_json::to_string(request_params)?;

        let response = client
            .post(&url)
            .headers(headers)
            .body(json_body) // Use body with JSON string
            .send()
            .await?;

        // Check HTTP status code - return an error for non-2xx responses
        if !response.status().is_success() {
            let status = response.status();
            return Err(format!("HTTP error: {}", status).into());
        }

        // Read response body as bytes and deserialize using serde_json
        let response_bytes = response.bytes().await?;
        let response_str = String::from_utf8_lossy(&response_bytes);
        let response_json: Value = serde_json::from_str(&response_str)?;

        Ok(response_json)
    }

    // Method to make an RPC call for playing an episode
    pub async fn rpc_play(&self, selected_episode: &SelectedEpisode) -> Result<(), Box<dyn Error>> {
        let play_episode_request_params = json!({
            "jsonrpc": "2.0",
            "method": "Player.Open",
            "params": {
                "item": {
                    "file": &selected_episode.episode_file_path,
                }
            },
            "id": 1
        });

        // Make the RPC call to play the episode
        let _play_response = self.rpc_call(&play_episode_request_params).await?;
        println!("Play response: {:?}", _play_response);

        Ok(())
    }

    // method to stop playback
    pub async fn rpc_stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Create a JSON-RPC request to stop playback
        let params = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "Player.Stop",
            "params": {
                "playerid": 1
            },
            "id": 1
        });

        // Send the request
        let _response = self.rpc_call(&params).await?;

        Ok(())
    }

    pub async fn is_active(&self) -> Result<bool, Box<dyn Error>> {
        let active_players_request_params = json!({
            "jsonrpc": "2.0",
            "method": "Player.GetActivePlayers",
            "id": 1
        });

        let active_players_response_json = self.rpc_call(&active_players_request_params).await?;
        let active_players = active_players_response_json["result"]
            .as_array()
            .unwrap_or(&vec![])
            .to_owned(); // Clone the array

        Ok(!active_players.is_empty())
    }
}

impl std::fmt::Debug for RpcClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RpcClient {}", self.config.url)
    }
}
