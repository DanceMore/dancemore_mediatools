mod kodi_helper;
use kodi_helper::Config;
use kodi_helper::RpcClient;

use std::env;
use std::collections::HashMap;
use std::time::Duration;
use std::io::{self, Write};
use tokio::time::sleep;

use serde_yaml;
use serde::Deserialize;

use rand::prelude::SliceRandom;

#[derive(Debug, Deserialize)]
struct ShowMappings {
    #[serde(flatten)]
    shows: HashMap<String, Vec<String>>,
}

fn load_show_mappings() -> Result<ShowMappings, Box<dyn std::error::Error>> {
    let show_mappings_content = std::fs::read_to_string("show_mappings.yml")?;
    let show_mappings: ShowMappings = serde_yaml::from_str(&show_mappings_content)?;
    Ok(show_mappings)
}

fn select_random_show_name(shows: &Vec<String>) -> Option<&String> {
    let mut rng = rand::thread_rng();
    shows.choose(&mut rng)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load("config.yml")?;
    let rpc_client = RpcClient::new(config)?;

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
	    eprintln!("Usage: {} <User>", args[0]);
	    return Ok(());
    }
    let user = &args[1];

    let show_mappings = load_show_mappings()?;
    let user_shows = show_mappings.shows.get(user).ok_or_else(|| "User not found")?;

    if user_shows.is_empty() {
        eprintln!("No shows available for this user.");
        std::process::exit(1);
    }

    let selected_show_name = select_random_show_name(user_shows).expect("No show available");

    println!("[-] selected show => {:?}", selected_show_name);

    let mut i = 0;

    loop {
        if !rpc_client.is_active().await? {
            println!("\n[!] no show playing, calling other Rust binary...\n");

            let selected_episode = rpc_client.select_random_episode_by_title(&selected_show_name).await?;
            rpc_client.rpc_play(&selected_episode).await?;
        } else if i == 0 {
            print!("."); // Print a dot
            io::stdout().flush()?; // Make sure the dot is immediately printed
            i = 60; // Reset the counter
        }

        i -= 1;

        sleep(Duration::from_secs(1)).await;
    }
}
