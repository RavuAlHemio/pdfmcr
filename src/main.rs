mod file_to_pdf;
mod jpeg;
mod model;
mod pdf;


use std::sync::LazyLock;

use askama::Template;
use rocket;
use tokio::sync::RwLock;


static WEB_FILE: LazyLock<RwLock<crate::model::File>> = LazyLock::new(|| RwLock::new(crate::model::File::default()));


#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;


#[rocket::get("/")]
fn index() -> String {
    IndexTemplate.render().unwrap()
}


#[rocket::launch]
fn launch_rocket() -> _ {
    rocket::build()
        .mount("/", rocket::routes![index])
}
