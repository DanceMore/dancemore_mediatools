mod api;
mod index;

pub fn all_routes() -> Vec<rocket::Route> {
    // Combine routes from all modules
    let mut routes = Vec::new();
    routes.extend(index::routes());
    routes.extend(api::routes());
    routes
}
