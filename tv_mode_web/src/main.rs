#[macro_use]
extern crate rocket;
use rocket_dyn_templates::Template;

// local imports
mod app_state;
use app_state::AppState;
mod routes;
mod scheduler;
use scheduler::start_scheduler;

#[launch]
fn rocket() -> _ {
    // Initialize the pp state
    let app_state = match app_state::initialize() {
        Ok(state) => state,
        Err(e) => {
            eprintln!("Failed to initialize application: {}", e);
            std::process::exit(1);
        }
    };

    // Build the rocket instance with routes and scheduler
    rocket::build()
        .manage(app_state.clone())
        .mount("/", routes::all_routes())
        .attach(Template::fairing())
        .attach(rocket::fairing::AdHoc::on_liftoff(
            "Initialize Scheduler",
            |rocket| {
                Box::pin(async move {
                    //let state = rocket.state::<AppState>().unwrap();
                    start_scheduler(app_state).await;
                })
            },
        ))
}
