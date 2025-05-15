use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use rocket::serde::Serialize;
use rocket::Route;
use rocket::State;

use std::collections::BTreeMap;

use crate::app_state::AppState;
use crate::app_state::ShowMappings;
use crate::app_state::TVModeStatus;

type ApiResponse<T> = Result<Json<T>, Custom<Json<StatusResponse>>>;

#[derive(Debug, Serialize)]
pub struct UsersResponse {
    show_mappings: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    status: String,
    message: String,
    tv_mode: Option<TVModeStatus>,
}

#[get("/api/users")]
pub async fn get_users(app_state: &State<AppState>) -> Json<UsersResponse> {
    let users = app_state.show_mappings.read().await.sorted_shows();

    Json(UsersResponse {
        show_mappings: users,
    })
}

#[post("/api/play/<user>")]
pub async fn play_random_show(app_state: &State<AppState>, user: &str) -> Json<StatusResponse> {
    info!("[!] route for user: {}", user);

    let mut tv_mode = app_state.tv_mode.write().await;

    // enable TV mode for User=$user
    tv_mode.active = true;
    tv_mode.user = Some(user.to_string());

    Json(StatusResponse {
        status: "0".to_string(),
        message: format!("enabled tv-mode for {}", user.to_string()),
        tv_mode: Some(tv_mode.clone()),
    })
}

#[post("/api/stop")]
pub async fn stop_tv_mode(app_state: &State<AppState>) -> Json<StatusResponse> {
    let mut tv_mode = app_state.tv_mode.write().await;

    // disable tv-mode and clear user
    tv_mode.active = false;
    tv_mode.user = None;

    Json(StatusResponse {
        status: "0".to_string(),
        message: "disabled tv-mode".to_string(),
        tv_mode: Some(tv_mode.clone()),
    })
}

#[get("/api/status")]
pub async fn get_status(app_state: &State<AppState>) -> ApiResponse<StatusResponse> {
    // Get TV mode status - unlock immediately to avoid holding mutex across await
    let tv_mode_status = app_state.tv_mode.read().await.clone();

    // This scope ensures the lock is dropped after we call is_active()
    let client = app_state.rpc_client.read().await;

    // Call is_active() and handle potential errors
    let active_result = match client.is_active().await {
        Ok(result) => result,
        Err(err) => {
            error!("Failed to connect to RPC server: {}", err);
            return Err(Custom(
                Status::Ok, // Still return HTTP 200 to the client
                Json(StatusResponse {
                    status: "error".to_string(),
                    message: "Unable to connect to media server".to_string(),
                    tv_mode: Some(tv_mode_status),
                }),
            ));
        }
    };

    // Lock is automatically released here when client goes out of scope

    if active_result {
        Ok(Json(StatusResponse {
            status: "active".to_string(),
            message: "Media is currently playing".to_string(),
            tv_mode: Some(tv_mode_status),
        }))
    } else {
        Ok(Json(StatusResponse {
            status: "inactive".to_string(),
            message: "No media is currently playing".to_string(),
            tv_mode: Some(tv_mode_status),
        }))
    }
}

// Return routes defined in this module
pub fn routes() -> Vec<Route> {
    routes![get_users, get_status, play_random_show, stop_tv_mode]
}
