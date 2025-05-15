use rocket::tokio;
use rocket::tokio::time::Duration;

use rand::prelude::*;

use crate::app_state::AppState;

pub async fn start_scheduler(app_state: AppState) {
    info!("[+] Starting scheduler...");
    tokio::spawn(scheduler_mainbody(app_state));
}

async fn scheduler_mainbody(app_state: AppState) {
    loop {
        debug!("[-] scheduler firing");

        // Get TV mode status
        let tv_mode_status = app_state.tv_mode.read().await.clone();

        // Check if media is active, with error handling
        let active_result = {
            let client = app_state.rpc_client.read().await;
            match client.is_active().await {
                Ok(result) => result,
                Err(err) => {
                    error!("[-] scheduler failed to connect to RPC server: {}", err);
                    continue; // Skip this iteration and try again
                }
            }
        };

        debug!("[-] scheduler sees {} playing", active_result);

        if tv_mode_status.active && !active_result {
            info!("[!] tv-mode is enabled but nothing is playing, we should DO WORK !!!");

            // Get the user (with error handling)
            let user = match tv_mode_status.user {
                Some(user) => user,
                None => {
                    warn!("[!] tv-mode is enabled with None $user. this should never happen. disabling tv-mode.");
                    // Instead of exiting, disable TV mode and continue
                    let mut tv_mode = app_state.tv_mode.write().await;
                    tv_mode.active = false;
                    tv_mode.user = None;
                    continue;
                }
            };

            info!("[!] would play for user: {}", user);

            let shows = app_state.show_mappings.read().await.sorted_shows().clone();

            // Get the user's shows (with error handling)
            let user_shows = match shows.get(&user) {
                Some(shows) => shows,
                None => {
                    error!("[!] User '{}' not found in show mappings", user);
                    // Disable TV mode for this non-existent user
                    let mut tv_mode = app_state.tv_mode.write().await;
                    tv_mode.active = false;
                    tv_mode.user = None;
                    continue;
                }
            };

            // Select a random show (with error handling)
            let selected_show_name = match select_random_show_name(user_shows) {
                Some(show) => show,
                None => {
                    error!("[!] No shows available for user '{}'", user);
                    continue;
                }
            };

            info!("[-] selected show => {:?}", selected_show_name);

            let rpc_client = app_state.rpc_client.read().await;

            // Try to select a random episode (with error handling)
            let selected_episode = match rpc_client
                .select_random_episode_by_title(selected_show_name)
                .await
            {
                Ok(episode) => episode,
                Err(err) => {
                    error!(
                        "[!] Failed to select episode for show '{}': {}",
                        selected_show_name, err
                    );
                    continue;
                }
            };

            // Try to play the episode (with error handling)
            match rpc_client.rpc_play(&selected_episode).await {
                Ok(result) => info!("[+] Successfully started playing: {:?}", result),
                Err(err) => error!("[!] Failed to play episode: {}", err),
            }
        }

        // Non-blocking sleep using tokio
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

fn select_random_show_name<'a>(shows: &'a [String]) -> Option<&'a String> {
    if shows.is_empty() {
        return None;
    }
    shows.choose(&mut rand::thread_rng())
}
