#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;

mod latex;
mod generation_request;

use std::collections::HashMap;
use std::path::Path;
use std::result::Result;
use glob::glob;
use rocket::response::NamedFile;
use rocket_contrib::json::{Json, JsonValue};
use crate::generation_request::GenerationRequest;
use crate::latex::generate_latex;

#[get("/")]
fn index() -> &'static str {
    "Hello, World!"
}

#[post("/", format = "json", data = "<gen_req>")]
fn generate(gen_req: Json<GenerationRequest>) -> JsonValue {
    // TODO Replace test data with the corresponding data from the given JSON object
    let mut article1: HashMap<String, String> = HashMap::new();
    article1.insert("Test 1".to_string(), "13,37 €".to_string());
    let mut article2: HashMap<String, String> = HashMap::new();
    article2.insert("Test 2".to_string(), "42,42 €".to_string());

    let mut collections: HashMap<String, Vec<HashMap<String, String>>> = HashMap::new();
    collections.insert("articles".to_string(), vec![article1, article2]);

    let mut keys: HashMap<String, String> = HashMap::new();
    keys.insert("date".to_string(), "20.06.2019".to_string());

    let id = generate_latex(&gen_req, &keys, &collections);
    json!({ "status": "ok", "id": id})
}

#[get("/")]
fn list() -> JsonValue {
    let mut templates: Vec<String> = Vec::new();

    for path in glob("./templates/*.tex").unwrap().filter_map(Result::ok) {
        templates.push(path.file_stem().expect("Failed").to_str().expect("Failed").to_string())
    }
    json!({ "templates": templates})
}

#[get("/<id>")]
fn get_pdf(id: usize) -> Option<NamedFile> {
    NamedFile::open(Path::new(&format!("pdf/output{}.pdf", id))).ok()
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
    .mount("/", routes![index])
    .mount("/templates", routes![list])
    .mount("/generate", routes![generate])
    .mount("/pdf", routes![get_pdf])
}

fn main() {
    rocket().launch();
}
