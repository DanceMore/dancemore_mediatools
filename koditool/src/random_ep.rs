mod kodi_helper;
use kodi_helper::Config;
use kodi_helper::RpcClient;

use std::error::Error;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load configuration from YAML
    let config = Config::load("config.yml")?;

    // Get the TV show name from the command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
	    eprintln!("Usage: {} <TV Show Name>", args[0]);
	    return Ok(());
    }

    let tv_show_name = &args[1];
    println!("[-] target => {:?}", tv_show_name);

    // build RPC client and run
    let rpc_client = RpcClient::new(config)?;

    // Call the function to select a random episode by title
    let selected_episode = rpc_client.select_random_episode_by_title(tv_show_name).await?;

    // Call the function to play the selected episode
    rpc_client.rpc_play(&selected_episode).await?;

    Ok(())
}
