use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use rocket::serde::Serialize;
use rocket::tokio::time::{Duration, Instant};
use rocket::Route;
use rocket::State;

use std::collections::BTreeMap;

use crate::app_state::AppState;
use crate::app_state::ShowMappings;
use crate::app_state::TVModeStatus;

type ApiResponse<T> = Result<Json<T>, Custom<Json<StatusResponse>>>;

// Timeout for RPC calls
const RPC_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Serialize)]
pub struct UsersResponse {
    show_mappings: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    status: String,
    message: String,
    tv_mode: Option<TVModeStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_details: Option<String>,
}

impl StatusResponse {
    fn success(message: String, tv_mode: Option<TVModeStatus>) -> Self {
        Self {
            status: "success".to_string(),
            message,
            tv_mode,
            error_details: None,
        }
    }

    fn error(
        message: String,
        tv_mode: Option<TVModeStatus>,
        error_details: Option<String>,
    ) -> Self {
        Self {
            status: "error".to_string(),
            message,
            tv_mode,
            error_details,
        }
    }

    fn media_active(tv_mode: TVModeStatus) -> Self {
        Self {
            status: "active".to_string(),
            message: "Media is currently playing".to_string(),
            tv_mode: Some(tv_mode),
            error_details: None,
        }
    }

    fn media_inactive(tv_mode: TVModeStatus) -> Self {
        Self {
            status: "inactive".to_string(),
            message: "No media is currently playing".to_string(),
            tv_mode: Some(tv_mode),
            error_details: None,
        }
    }
}

#[get("/api/users")]
pub async fn get_users(app_state: &State<AppState>) -> ApiResponse<UsersResponse> {
    let users = app_state.show_mappings.read().await.sorted_shows();

    Ok(Json(UsersResponse {
        show_mappings: users,
    }))
}

#[post("/api/play/<user>")]
pub async fn play_random_show(
    app_state: &State<AppState>,
    user: &str,
) -> ApiResponse<StatusResponse> {
    // Validate user exists in mappings first
    let shows = app_state.show_mappings.read().await.sorted_shows();
    if !shows.contains_key(user) {
        warn!("Attempt to enable TV mode for unknown user: {}", user);
        return Err(Custom(
            Status::BadRequest,
            Json(StatusResponse::error(
                format!("User '{}' not found in show mappings", user),
                None,
                Some("Check available users via /api/users endpoint".to_string()),
            )),
        ));
    }

    let user_shows = shows.get(user).unwrap();
    if user_shows.is_empty() {
        warn!("Attempt to enable TV mode for user with no shows: {}", user);
        return Err(Custom(
            Status::BadRequest,
            Json(StatusResponse::error(
                format!("User '{}' has no shows configured", user),
                None,
                Some("Add shows to the show_mappings.yml file for this user".to_string()),
            )),
        ));
    }

    info!("Enabling TV mode for user: {}", user);

    let mut tv_mode = app_state.tv_mode.write().await;
    tv_mode.active = true;
    tv_mode.user = Some(user.to_string());

    Ok(Json(StatusResponse::success(
        format!(
            "Enabled TV mode for user '{}' with {} shows available",
            user,
            user_shows.len()
        ),
        Some(tv_mode.clone()),
    )))
}

#[post("/api/stop")]
pub async fn stop_tv_mode(app_state: &State<AppState>) -> ApiResponse<StatusResponse> {
    let mut tv_mode = app_state.tv_mode.write().await;

    let was_active = tv_mode.active;
    let previous_user = tv_mode.user.clone();

    tv_mode.active = false;
    tv_mode.user = None;

    let message = if was_active {
        match previous_user {
            Some(user) => {
                info!("Disabled TV mode (was active for user: {})", user);
                format!("Disabled TV mode (was active for user '{}')", user)
            }
            None => {
                info!("Disabled TV mode (was active with no user)");
                "Disabled TV mode (was active with no user)".to_string()
            }
        }
    } else {
        debug!("TV mode stop requested but was already inactive");
        "TV mode was already inactive".to_string()
    };

    Ok(Json(StatusResponse::success(
        message,
        Some(tv_mode.clone()),
    )))
}

#[get("/api/status")]
pub async fn get_status(app_state: &State<AppState>) -> ApiResponse<StatusResponse> {
    let tv_mode_status = app_state.tv_mode.read().await.clone();

    // Use a timeout wrapper for the RPC call
    let active_result = {
        let client = app_state.rpc_client.read().await;

        match rocket::tokio::time::timeout(RPC_TIMEOUT, client.is_active()).await {
            Ok(Ok(result)) => Ok(result),
            Ok(Err(rpc_error)) => Err(format!("RPC error: {}", rpc_error)),
            Err(_) => Err("RPC call timed out after 5 seconds".to_string()),
        }
    };

    match active_result {
        Ok(is_active) => {
            if is_active {
                Ok(Json(StatusResponse::media_active(tv_mode_status)))
            } else {
                Ok(Json(StatusResponse::media_inactive(tv_mode_status)))
            }
        }
        Err(error_msg) => {
            // Log the error but don't spam - use warn level
            warn!("Media server connectivity issue: {}", error_msg);

            // Return HTTP 200 with error status to indicate API is working
            // but media server is unreachable
            Ok(Json(StatusResponse::error(
                "Unable to connect to media server".to_string(),
                Some(tv_mode_status),
                Some(error_msg),
            )))
        }
    }
}

// Health check endpoint
#[get("/api/health")]
pub async fn health_check() -> Json<StatusResponse> {
    Json(StatusResponse::success("API is healthy".to_string(), None))
}

// Return routes defined in this module
pub fn routes() -> Vec<Route> {
    routes![
        get_users,
        get_status,
        play_random_show,
        stop_tv_mode,
        health_check
    ]
}
