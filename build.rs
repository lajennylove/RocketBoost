//! Compiles the views in `resources/views` into Rust, so a release build renders
//! them without a template interpreter. The generated module is included in
//! `views.rs`. Every view here takes the same context, `views::Page`.

fn main() {
    const PAGE: &str = "crate::views::Page";
    propel_compiler::build::Views::new("resources/views")
        .view("home", PAGE)
        .view("about", PAGE)
        .view("docs.index", PAGE)
        .view("errors.not_found", PAGE)
        .build();
}
