use rocket::tokio;
use rocket::tokio::time::Duration;
use std::io::Write;

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


            //user_shows = match app_state.show_mappings.shows.get(&user) {

            //fn select_random_show_name(shows: &Vec<String>) -> Option<&String> {
            //    let mut rng = rand::thread_rng();
            //    shows.choose(&mut rng)
            //}
            
            
            
            
            //    let selected_show_name = match select_random_show_name(user_shows) {


        }

        // drop locks until the next loop...
        //drop(locked_song_queue);
        //drop(locked_tags_data);
        //drop(locked_mpd_conn);

        // Non-blocking sleep using tokio
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
