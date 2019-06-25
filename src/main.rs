#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;

mod latex;
mod generation_request;

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
    let id = generate_latex(&gen_req);
    json!({ "status": "ok", "id": id })
}

#[get("/")]
fn list() -> JsonValue {
    let mut templates: Vec<String> = Vec::new();

    for path in glob("templates/**/*.tex").unwrap().filter_map(Result::ok) {
        // templates.push(path.file_stem().expect("Failed").to_str().expect("Failed").to_string())
        let dirpath =  path.strip_prefix("templates").expect("Failed").parent().expect("Failed");
        let mut dirpathstr = String::new();
        if !dirpath.to_str().expect("Failed").to_string().is_empty() { // dirpath not empty

            for component in dirpath.components() {
                dirpathstr += component.as_os_str().to_str().expect("Failed");
                dirpathstr += "/";
            }
        }

        let filename = path.file_stem().expect("Failed").to_str().expect("Failed");

        templates.push(format!("{}{}", dirpathstr, filename));
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
