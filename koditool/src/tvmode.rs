mod kodi_helper;
use kodi_helper::Config;
use kodi_helper::RpcClient;

use rand::prelude::IndexedRandom;
use std::collections::HashMap;
use std::env;
use std::io::{self, Write};
use std::time::Duration;
use tokio::time::sleep;

use serde::Deserialize;
use serde_yaml;

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

fn select_random_show_name<'a>(shows: &'a [String]) -> Option<&'a String> {
    shows.choose(&mut rand::rng())
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
    let user_shows = show_mappings
        .shows
        .get(user)
        .ok_or_else(|| "User not found")?;

    if user_shows.is_empty() {
        eprintln!("No shows available for this user.");
        std::process::exit(1);
    }

    let spinner_chars = "|/-\\";
    let mut spinner_index = 0;

    loop {
        if !rpc_client.is_active().await? {
            let selected_show_name = select_random_show_name(user_shows).expect("No show available");
            println!("[-] selected show => {:?}", selected_show_name);

            println!("\n[!] no show playing, calling other Rust binary...\n");

            let selected_episode = rpc_client
                .select_random_episode_by_title(&selected_show_name)
                .await?;
            rpc_client.rpc_play(&selected_episode).await?;

            // sleep for a moment after playing a new show to let Physics resolve
            sleep(Duration::from_secs(3)).await;
        } else {
            // Print the spinner character and move to the next one
            print!("{}", spinner_chars.chars().nth(spinner_index).unwrap());
            io::stdout().flush()?; // Make sure the spinner is immediately printed

            // Move to the next spinner character, wrapping around if necessary
            spinner_index = (spinner_index + 1) % spinner_chars.len();

            // Backspace to overwrite the previous spinner character
            print!("\x08");
        }

        sleep(Duration::from_secs(1)).await;
    }
}
