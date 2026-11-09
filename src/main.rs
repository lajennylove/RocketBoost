#[macro_use]
extern crate rocket;

mod views;

use propel_rocket::PropelView;
use rocket::fs::FileServer;
use rocket::http::{ContentType, Status};
use views::{render_page, Page};

/// The site's address, with no trailing slash. The canonical links, the Open Graph
/// URLs and the sitemap are built from it.
const SITE_URL: &str = "https://rocketboost.rs";

/// The public pages, in the order the sitemap lists them. The nav in
/// `resources/views/layouts/app.propel` links to the same pages. The error pages
/// are left out.
const PUBLIC_PAGES: &[&str] = &["/", "/about", "/docs"];

/// The date the sitemap gives for each page.
const LAST_MODIFIED: &str = "2026-11-09";

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

#[get("/robots.txt")]
fn robots() -> (ContentType, String) {
    let body = format!("User-agent: *\nAllow: /\n\nSitemap: {SITE_URL}/sitemap.xml\n");
    (ContentType::Text, body)
}

#[get("/sitemap.xml")]
fn sitemap() -> (ContentType, String) {
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
    );
    for page in PUBLIC_PAGES {
        // The root has no trailing slash, as in the canonical links.
        let loc = if *page == "/" {
            SITE_URL.to_string()
        } else {
            format!("{SITE_URL}{page}")
        };
        xml.push_str(&format!(
            "  <url>\n    <loc>{loc}</loc>\n    <lastmod>{LAST_MODIFIED}</lastmod>\n  </url>\n"
        ));
    }
    xml.push_str("</urlset>\n");
    (ContentType::new("application", "xml"), xml)
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
        .mount("/", routes![home, about, docs, robots, sitemap])
        // Built assets: hashed files and copied images and fonts.
        .mount("/build", FileServer::new("public/build"))
        .register("/", catchers![not_found])
}
