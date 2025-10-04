mod api;
mod index;
mod jukectl;

pub fn all_routes() -> Vec<rocket::Route> {
    // Combine routes from all modules
    let mut routes = Vec::new();
    routes.extend(index::routes());
    routes.extend(api::routes());
    routes.extend(jukectl::routes());
    routes
}
