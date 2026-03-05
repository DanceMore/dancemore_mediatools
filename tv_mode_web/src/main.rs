#[macro_use]
extern crate rocket;

use tv_mode_web::build_rocket;

#[launch]
fn rocket() -> _ {
    build_rocket()
}
