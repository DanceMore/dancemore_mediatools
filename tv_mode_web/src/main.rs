// Import the koditool library from the workspace
use koditool::Config;
use koditool::RpcClient;
use log::info;

use actix_files as fs;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use rand::seq::IteratorRandom; // Use iterator random instead of slice
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time;

#[derive(Debug, Deserialize, Clone)]
struct ShowMappings {
    #[serde(flatten)]
    shows: HashMap<String, Vec<String>>,
}

#[derive(Debug, Serialize)]
struct UsersList {
    users: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TVModeStatus {
    active: bool,
    user: Option<String>,
}

#[derive(Debug, Serialize)]
struct StatusResponse {
    status: String,
    message: String,
    tv_mode: Option<TVModeStatus>,
}

#[derive(Debug)]
struct AppState {
    rpc_client: Arc<Mutex<RpcClient>>,
    show_mappings: Arc<ShowMappings>,
    tv_mode: Arc<Mutex<TVModeStatus>>,
}

fn load_show_mappings() -> Result<ShowMappings, String> {
    std::fs::read_to_string("show_mappings.yml")
        .map_err(|e| format!("Failed to read show_mappings.yml: {}", e))
        .and_then(|content| {
            serde_yaml::from_str(&content)
                .map_err(|e| format!("Failed to parse show_mappings.yml: {}", e))
        })
}

// IMPORTANT: This is a completely separate function that will be called
// synchronously to avoid ThreadRng being used in async context
fn sync_select_random_show_name(shows: &[String]) -> Option<String> {
    use rand::thread_rng;
    shows.iter().choose(&mut thread_rng()).cloned()
}

// Function to play a random show for a given user
async fn play_random_show_for_user(
    rpc_client: &Arc<Mutex<RpcClient>>,
    shows: &HashMap<String, Vec<String>>,
    user: &str,
) -> Result<String, String> {
    // Get the user's shows
    let user_shows = match shows.get(user) {
        Some(shows) => shows,
        None => return Err(format!("User '{}' not found", user)),
    };

    if user_shows.is_empty() {
        return Err("No shows available for this user".to_string());
    }

    // First select a random show name (synchronously, not holding any async state)
    // IMPORTANT: We call this outside of async context to avoid Send issues with ThreadRng
    let selected_show_name = match sync_select_random_show_name(user_shows) {
        Some(show) => show,
        None => return Err("Failed to select a random show".to_string()),
    };

    // Acquire the mutex lock in a small scope
    let selected_episode = {
        let client = rpc_client.lock().await;
        // We need to call a separate function to avoid the ThreadRng Send issue
        match client
            .select_random_episode_by_title(&selected_show_name)
            .await
        {
            Ok(episode) => episode,
            Err(e) => return Err(format!("Failed to select episode: {}", e)),
        }
    };

    // Play the selected episode (in another small scope)
    {
        let client = rpc_client.lock().await;
        match client.rpc_play(&selected_episode).await {
            Ok(_) => Ok(format!(
                "Now playing {} - {}",
                selected_show_name, selected_episode
            )),
            Err(e) => Err(format!("Failed to play episode: {}", e)),
        }
    }
}

#[get("/")]
async fn index() -> impl Responder {
    fs::NamedFile::open_async("./static/index.html").await
}

#[get("/api/users")]
async fn get_users(data: web::Data<AppState>) -> impl Responder {
    let users: Vec<String> = data.show_mappings.shows.keys().cloned().collect();
    let response = UsersList { users };
    HttpResponse::Ok().json(response)
}

// Start TV mode for a specific user
#[post("/api/tv-mode/{user}")]
async fn start_tv_mode(data: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let user = path.into_inner();
    info!("!!!!! inside start_tv_mode for {}", user);

    // Validate the user exists
    if !data.show_mappings.shows.contains_key(&user) {
        return HttpResponse::NotFound().json(StatusResponse {
            status: "error".to_string(),
            message: format!("User '{}' not found", user),
            tv_mode: None,
        });
    }

    // Stop any current playback
    {
        let client = data.rpc_client.lock().await;
        let _ = client.rpc_stop().await;
    }

    // Update TV mode status
    {
        let mut tv_mode = data.tv_mode.lock().await;
        tv_mode.active = true;
        tv_mode.user = Some(user.clone());
    }
    info!("!!!!! {:?}", data.tv_mode);

    // Play a random show for the user
    let play_result =
        play_random_show_for_user(&data.rpc_client, &data.show_mappings.shows, &user).await;

    match play_result {
        Ok(message) => HttpResponse::Ok().json(StatusResponse {
            status: "success".to_string(),
            message: format!("TV Mode started for '{}'. {}", user, message),
            tv_mode: Some(TVModeStatus {
                active: true,
                user: Some(user),
            }),
        }),
        Err(e) => {
            // If we failed to play, disable TV mode
            let mut tv_mode = data.tv_mode.lock().await;
            tv_mode.active = false;
            tv_mode.user = None;

            HttpResponse::InternalServerError().json(StatusResponse {
                status: "error".to_string(),
                message: format!("Failed to start TV Mode: {}", e),
                tv_mode: None,
            })
        }
    }
}

// Legacy endpoint for backward compatibility
#[post("/api/play/{user}")]
async fn play_random_show(data: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let user = path.into_inner();

    // Get the current TV mode status
    let tv_mode_status = {
        let tv_mode = data.tv_mode.lock().await;
        tv_mode.clone()
    };

    // Play a random show for the user without enabling TV mode
    let play_result =
        play_random_show_for_user(&data.rpc_client, &data.show_mappings.shows, &user).await;

    match play_result {
        Ok(message) => HttpResponse::Ok().json(StatusResponse {
            status: "success".to_string(),
            message,
            tv_mode: Some(tv_mode_status),
        }),
        Err(e) => HttpResponse::InternalServerError().json(StatusResponse {
            status: "error".to_string(),
            message: format!("Failed to play episode: {}", e),
            tv_mode: Some(tv_mode_status),
        }),
    }
}

#[get("/api/status")]
async fn get_status(data: web::Data<AppState>) -> impl Responder {
    // Get TV mode status
    let tv_mode_status = {
        let tv_mode = data.tv_mode.lock().await;
        tv_mode.clone()
    };

    // Check playback status
    let is_active = {
        let client = data.rpc_client.lock().await;
        match client.is_active().await {
            Ok(active) => active,
            Err(e) => {
                return HttpResponse::InternalServerError().json(StatusResponse {
                    status: "error".to_string(),
                    message: format!("Failed to check status: {}", e),
                    tv_mode: Some(tv_mode_status),
                });
            }
        }
    };

    if is_active {
        HttpResponse::Ok().json(StatusResponse {
            status: "active".to_string(),
            message: "Media is currently playing".to_string(),
            tv_mode: Some(tv_mode_status),
        })
    } else {
        HttpResponse::Ok().json(StatusResponse {
            status: "inactive".to_string(),
            message: "No media is currently playing".to_string(),
            tv_mode: Some(tv_mode_status),
        })
    }
}

#[post("/api/stop")]
async fn stop_tv_mode(data: web::Data<AppState>) -> impl Responder {
    // Stop playback
    {
        let client = data.rpc_client.lock().await;
        if let Err(e) = client.rpc_stop().await {
            return HttpResponse::InternalServerError().json(StatusResponse {
                status: "error".to_string(),
                message: format!("Failed to stop playback: {}", e),
                tv_mode: None,
            });
        }
    }

    // Disable TV mode
    let response = {
        let mut tv_mode = data.tv_mode.lock().await;
        let was_active = tv_mode.active;
        let user = tv_mode.user.clone();

        tv_mode.active = false;
        tv_mode.user = None;

        StatusResponse {
            status: "success".to_string(),
            message: if was_active {
                format!("TV Mode stopped for user '{}'", user.unwrap_or_default())
            } else {
                "Playback stopped".to_string()
            },
            tv_mode: Some(TVModeStatus {
                active: false,
                user: None,
            }),
        }
    };

    HttpResponse::Ok().json(response)
}

// Start a background task to monitor playback status
async fn tv_mode_monitor(app_state: Arc<web::Data<AppState>>) {
    let mut interval = time::interval(Duration::from_secs(2));

    loop {
        println!("{:?}", app_state);
        interval.tick().await;

        // Check if TV mode is active
        let user_to_play = {
            let tv_mode = app_state.tv_mode.lock().await;

            if !tv_mode.active || tv_mode.user.is_none() {
                continue; // Skip if TV mode is not active
            }

            tv_mode.user.clone().unwrap()
        };

        println!("{}", user_to_play);

        // Check if media is currently playing
        let is_playing = {
            let client = app_state.rpc_client.lock().await;

            match client.is_active().await {
                Ok(active) => active,
                Err(_) => continue, // Skip this iteration if check failed
            }
        };

        // If TV mode is active but nothing is playing, start a new show
        if !is_playing {
            // Try to play a new show
            let _ = play_random_show_for_user(
                &app_state.rpc_client,
                &app_state.show_mappings.shows,
                &user_to_play,
            )
            .await;
        }

        // Non-blocking sleep using tokio
        //tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    println!("Loading configuration...");

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

    // Create app state with tokio mutexes and Arc
    let app_state = web::Data::new(AppState {
        rpc_client: Arc::new(Mutex::new(rpc_client)),
        show_mappings: Arc::new(show_mappings),
        tv_mode: Arc::new(Mutex::new(TVModeStatus {
            active: false,
            user: None,
        })),
    });

    // Start the TV mode monitor task - IMPORTANT: Use Arc to avoid Send issues
    let monitor_state = Arc::new(app_state.clone());
    tokio::spawn(async move {
        tv_mode_monitor(monitor_state).await;
    });

    // Create server
    println!("Starting server on 0.0.0.0:8080");
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(index)
            .service(get_users)
            .service(play_random_show)
            .service(start_tv_mode)
            .service(get_status)
            .service(stop_tv_mode)
            .service(fs::Files::new("/static", "./static").show_files_listing())
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
