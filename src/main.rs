#[macro_use]
extern crate rocket;

mod views;

use propel_rocket::PropelView;
use rocket::fs::FileServer;
use rocket::http::Status;
use views::{render_page, Page};

#[get("/")]
fn home() -> PropelView {
    render_page("home", &Page {})
}

#[get("/about")]
fn about() -> PropelView {
    render_page("about", &Page {})
}

#[get("/docs")]
fn docs() -> PropelView {
    render_page("docs.index", &Page {})
}

fn not_found_page() -> (Status, PropelView) {
    (Status::NotFound, render_page("errors.not_found", &Page {}))
}

#[catch(404)]
fn not_found() -> (Status, PropelView) {
    not_found_page()
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![home, about, docs])
        // Built assets: hashed files and copied images and fonts.
        .mount("/build", FileServer::new("public/build"))
        .register("/", catchers![not_found])
}
