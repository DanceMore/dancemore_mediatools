// Import the koditool library from the workspace
use koditool::Config;
use koditool::RpcClient;

use std::collections::HashMap;
use std::sync::Mutex;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_files as fs;
use serde::{Deserialize, Serialize};
use rand::prelude::SliceRandom;

#[derive(Debug, Deserialize, Clone)]
struct ShowMappings {
    #[serde(flatten)]
    shows: HashMap<String, Vec<String>>,
}

#[derive(Debug, Serialize)]
struct UsersList {
    users: Vec<String>,
}

#[derive(Debug, Serialize)]
struct StatusResponse {
    status: String,
    message: String,
}

struct AppState {
    rpc_client: Mutex<RpcClient>,
    show_mappings: ShowMappings,
}

fn load_show_mappings() -> Result<ShowMappings, String> {
    std::fs::read_to_string("show_mappings.yml")
        .map_err(|e| format!("Failed to read show_mappings.yml: {}", e))
        .and_then(|content| {
            serde_yaml::from_str(&content)
                .map_err(|e| format!("Failed to parse show_mappings.yml: {}", e))
        })
}

fn select_random_show_name(shows: &Vec<String>) -> Option<&String> {
    let mut rng = rand::thread_rng();
    shows.choose(&mut rng)
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

#[post("/api/play/{user}")]
async fn play_random_show(
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let user = path.into_inner();
    
    // Get the user's shows
    let user_shows = match data.show_mappings.shows.get(&user) {
        Some(shows) => shows,
        None => {
            return HttpResponse::NotFound().json(StatusResponse {
                status: "error".to_string(),
                message: format!("User '{}' not found", user),
            });
        }
    };

    if user_shows.is_empty() {
        return HttpResponse::BadRequest().json(StatusResponse {
            status: "error".to_string(),
            message: "No shows available for this user".to_string(),
        });
    }

    // Select a random show
    let selected_show_name = match select_random_show_name(user_shows) {
        Some(show) => show,
        None => {
            return HttpResponse::InternalServerError().json(StatusResponse {
                status: "error".to_string(),
                message: "Failed to select a random show".to_string(),
            });
        }
    };

    // Get a lock on the RPC client and play the show
    let mut rpc_client = match data.rpc_client.lock() {
        Ok(client) => client,
        Err(_) => {
            return HttpResponse::InternalServerError().json(StatusResponse {
                status: "error".to_string(),
                message: "Failed to acquire lock on RPC client".to_string(),
            });
        }
    };

    // Select a random episode
    let selected_episode = match rpc_client.select_random_episode_by_title(selected_show_name).await {
        Ok(episode) => episode,
        Err(e) => {
            return HttpResponse::InternalServerError().json(StatusResponse {
                status: "error".to_string(),
                message: format!("Failed to select episode: {}", e),
            });
        }
    };

    // Play the selected episode
    match rpc_client.rpc_play(&selected_episode).await {
        Ok(_) => {
            HttpResponse::Ok().json(StatusResponse {
                status: "success".to_string(),
                message: format!("Now playing {} - {}", selected_show_name, selected_episode),
            })
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(StatusResponse {
                status: "error".to_string(),
                message: format!("Failed to play episode: {}", e),
            })
        }
    }
}

#[get("/api/status")]
async fn get_status(data: web::Data<AppState>) -> impl Responder {
    let rpc_client = match data.rpc_client.lock() {
        Ok(client) => client,
        Err(_) => {
            return HttpResponse::InternalServerError().json(StatusResponse {
                status: "error".to_string(),
                message: "Failed to acquire lock on RPC client".to_string(),
            });
        }
    };

    match rpc_client.is_active().await {
        Ok(active) => {
            if active {
                HttpResponse::Ok().json(StatusResponse {
                    status: "active".to_string(),
                    message: "Media is currently playing".to_string(),
                })
            } else {
                HttpResponse::Ok().json(StatusResponse {
                    status: "inactive".to_string(),
                    message: "No media is currently playing".to_string(),
                })
            }
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(StatusResponse {
                status: "error".to_string(),
                message: format!("Failed to check status: {}", e),
            })
        }
    }
}

#[post("/api/stop")]
async fn stop_playback(data: web::Data<AppState>) -> impl Responder {
    let rpc_client = match data.rpc_client.lock() {
        Ok(client) => client,
        Err(_) => {
            return HttpResponse::InternalServerError().json(StatusResponse {
                status: "error".to_string(),
                message: "Failed to acquire lock on RPC client".to_string(),
            });
        }
    };

    match rpc_client.rpc_stop().await {
        Ok(_) => {
            HttpResponse::Ok().json(StatusResponse {
                status: "success".to_string(),
                message: "Playback stopped".to_string(),
            })
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(StatusResponse {
                status: "error".to_string(),
                message: format!("Failed to stop playback: {}", e),
            })
        }
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
            return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()));
        }
    };
    
    // Create RPC client
    let rpc_client = match RpcClient::new(config) {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to create RPC client: {}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()));
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
    
    // Create app state
    let app_state = web::Data::new(AppState {
        rpc_client: Mutex::new(rpc_client),
        show_mappings,
    });
    
    // Create server
    println!("Starting server on 0.0.0.0:8080");
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(index)
            .service(get_users)
            .service(play_random_show)
            .service(get_status)
            .service(stop_playback)
            .service(fs::Files::new("/static", "./static").show_files_listing())
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
