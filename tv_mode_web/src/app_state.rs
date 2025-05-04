// Import the koditool library from the workspace
use koditool::Config;
use koditool::RpcClient;

use rocket::tokio::sync::RwLock;
use std::sync::Arc;
//use std::sync::RwLock;

use rocket::serde::Deserialize;
use rocket::serde::Serialize;
use std::collections::BTreeMap;
use std::collections::HashMap;
//use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
pub struct ShowMappings {
    #[serde(flatten)]
    shows: HashMap<String, Vec<String>>,
}

impl ShowMappings {
    // Add a method to get alphabetically sorted shows
    pub fn sorted_shows(&self) -> BTreeMap<String, Vec<String>> {
        let mut sorted_map = BTreeMap::new();

        for (key, mut values) in self.shows.clone() {
            // Sort the values (show names) alphabetically
            values.sort();
            sorted_map.insert(key, values);
        }

        sorted_map
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TVModeStatus {
    pub active: bool,
    pub user: Option<String>,
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub rpc_client: Arc<RwLock<RpcClient>>,
    pub show_mappings: Arc<RwLock<ShowMappings>>,
    pub tv_mode: Arc<RwLock<TVModeStatus>>,
}

pub fn initialize() -> Result<AppState, std::io::Error> {
    // Load config
    let config = match Config::load("config.yml") {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load config: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ));
        }
    };

    // Create RPC client
    let rpc_client = match RpcClient::new(config) {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to create RPC client: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ));
        }
    };

    // Load show mappings
    let show_mappings = match load_show_mappings() {
        Ok(mappings) => mappings,
        Err(e) => {
            eprintln!("Failed to load show mappings: {}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, e));
        }
    };

    // Create app state with mutexes and Arc
    let app_state = AppState {
        rpc_client: Arc::new(RwLock::new(rpc_client)),
        show_mappings: Arc::new(RwLock::new(show_mappings)),
        tv_mode: Arc::new(RwLock::new(TVModeStatus {
            active: false,
            user: None,
        })),
    };

    Ok(app_state)
}

fn load_show_mappings() -> Result<ShowMappings, String> {
    std::fs::read_to_string("show_mappings.yml")
        .map_err(|e| format!("Failed to read show_mappings.yml: {}", e))
        .and_then(|content| {
            let mut mappings: ShowMappings = serde_yaml::from_str(&content)
                .map_err(|e| format!("Failed to parse show_mappings.yml: {}", e))?;

            // Sort each vector of show names for consistency
            for values in mappings.shows.values_mut() {
                values.sort();
            }

            Ok(mappings)
        })
}
