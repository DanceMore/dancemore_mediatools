use rocket::Route;
use rocket::State;
use rocket_dyn_templates::Template;
use serde::Serialize;
use std::env;

use crate::app_state::{AppState, JukectlChannel};

#[derive(Serialize)]
struct JukectlContext {
    jukectl_api_url: String,
    channels: Vec<JukectlChannel>,
}

#[get("/jukectl")]
pub async fn jukectl_page(app_state: &State<AppState>) -> Template {
    // Get the jukectl API URL from environment variable
    let jukectl_api_url = env::var("JUKECTL_API_URL")
        .unwrap_or_else(|_| "http://localhost:8000".to_string());
    
    // Get the channels from app state
    let channels = app_state.jukectl_channels.read().await.clone();
    
    let context = JukectlContext { 
        jukectl_api_url,
        channels,
    };
    Template::render("jukectl", &context)
}

// Return routes defined in this module
pub fn routes() -> Vec<Route> {
    routes![jukectl_page]
}
