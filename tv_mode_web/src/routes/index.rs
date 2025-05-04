use rocket::Route;
use rocket::State;
use rocket_dyn_templates::Template;

use crate::app_state::AppState;

#[get("/")]
pub async fn index(app_state: &State<AppState>) -> Template {
    let context = "";
    Template::render("index", &context)
}

// Return routes defined in this module
pub fn routes() -> Vec<Route> {
    routes![index,]
}
