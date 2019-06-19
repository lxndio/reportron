#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;

mod latex;
mod generation_request;

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
    json!({ "status": "ok", "id": id})
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![index])
        .mount("/generate", routes![generate])
}

fn main() {
    rocket().launch();
}
