use rocket::tokio;
use rocket::tokio::time::Duration;
use std::io::Write;

use rand::prelude::IndexedRandom;

use crate::app_state::AppState;

pub async fn start_scheduler(app_state: AppState) {
    println!("[+] Starting scheduler...");
    tokio::spawn(scheduler_mainbody(app_state));
}

async fn scheduler_mainbody(app_state: AppState) {
    loop {
        debug!("[-] scheduler firing");

        let tv_mode_status = app_state.tv_mode.read().await
        .clone();

        let active_result = {
            let client = app_state.rpc_client.read().await;
            client.is_active().await.expect("")
        };
        debug!("[-] scheduler sees {} playing", active_result);

        if tv_mode_status.active && (active_result == false) {
            info!("[!] tv-mode is enabled but nothing is playing, we should DO WORK !!!");

            let result = match tv_mode_status.user {
                Some(user) => user,
                None => {
                    warn!("[!] tv-mode is enabled with None $user. this should never happen. disabling tv-mode.");
                    std::process::exit(1)
                }
            };
            info!("[!] would play {}", result);

            let shows = app_state.show_mappings.read().await.shows.clone();
            // Get the user's shows
            let user_shows = match shows.get(&result) {
                Some(shows) => shows,
                None => { std::process::exit(1) }
            };

            let selected_show_name =
                select_random_show_name(user_shows).expect("No show available");
            println!("[-] selected show => {:?}", selected_show_name);

            let rpc_client = app_state.rpc_client.read().await;

            let selected_episode = rpc_client
                .select_random_episode_by_title(&selected_show_name)
                .await.unwrap();
            let _result = rpc_client.rpc_play(&selected_episode).await;
        }

        // drop locks until the next loop...
        //drop(locked_song_queue);
        //drop(locked_tags_data);
        //drop(locked_mpd_conn);

        // Non-blocking sleep using tokio
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

fn select_random_show_name<'a>(shows: &'a [String]) -> Option<&'a String> {
    shows.choose(&mut rand::rng())
}