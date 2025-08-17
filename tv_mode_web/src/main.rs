#[macro_use]
extern crate rocket;

use rocket_dyn_templates::Template;
use std::env;

// local imports
mod app_state;
use app_state::AppState;
mod routes;
mod scheduler;
use scheduler::start_scheduler;

fn init_logging() {
    // Get log level from environment variable, default to "info"
    let log_level = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(&log_level))
        .format_timestamp_millis()
        .init();

    info!("Logging initialized with level: {}", log_level);
}

#[launch]
fn rocket() -> _ {
    // Initialize logging first
    init_logging();

    // Initialize the app state
    let app_state = match app_state::initialize() {
        Ok(state) => {
            info!("Application state initialized successfully");
            state
        }
        Err(e) => {
            error!("Failed to initialize application: {}", e);
            std::process::exit(1);
        }
    };

    info!("Starting Rocket web server...");

    // Build the rocket instance with routes and scheduler
    rocket::build()
        .manage(app_state.clone())
        .mount("/", routes::all_routes())
        .attach(Template::fairing())
        .attach(rocket::fairing::AdHoc::on_liftoff(
            "Initialize Scheduler",
            |_rocket| {
                Box::pin(async move {
                    start_scheduler(app_state).await;
                })
            },
        ))
}
