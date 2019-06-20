use std::fs;
use std::collections::HashMap;
use std::process::Command;
use std::path::Path;
use rocket_contrib::json::Json;
use regex::{Regex, Captures};
use crate::generation_request::GenerationRequest;
use uuid::Uuid;

struct ForEach {
    single_var: String,
    collection_var: String,
    current_pos: usize,
    done: bool,
}

impl ForEach {
    fn new(single_var: String, collection_var: String) -> ForEach {
        ForEach {
            single_var,
            collection_var,
            current_pos: 0,
            done: false,
        }
    }
}

fn evaluate(part: &str, gen_req: &Json<GenerationRequest>, keys: &HashMap<String, String>, collections: &HashMap<String, Vec<HashMap<String, String>>>, foreaches: &mut Vec<ForEach>) -> Option<String> {
    let mut new_part = String::new();

    for line in part.lines() {
        let line = line.trim();

        // Replace all normal keys
        let re = Regex::new(r"#\[(\S+)\]").unwrap();
        new_part += &re.replace_all(line, |caps: &Captures| keys.get(&caps[1]).expect("Key not found"));
    }

    Some(new_part)
}

pub fn generate_latex(gen_req: &Json<GenerationRequest>, keys: &HashMap<String, String>, collections: &HashMap<String, Vec<HashMap<String, String>>>) -> Option<String> {
    let id = Uuid::new_v4().to_string(); // generate random id

    // Create temp directory for output of this job
    let temp_dir_path = Path::new("pdf").join(format!("temp-{}", id));
    fs::create_dir(temp_dir_path.as_path()).expect("Could not create temp dir");

    // Create empty list of foreach objects
    let mut foreaches: Vec<ForEach> = Vec::new();

    // Read template file and replace the keys
    let template_path = Path::new("templates").join("test.tex");
    let file = fs::read_to_string(template_path).expect("Could not read template file");
    
    let new_file = evaluate(&file, gen_req, keys, collections, &mut foreaches).expect("Error while evaluating");

    // Write new file to temp directory
    let tex_output_path = temp_dir_path.join("new.tex");
    fs::write(tex_output_path.as_path(), new_file).expect("Could not write new file");

    /*let output = Command::new("cmd")
        .args(&["/C", &format!("pdflatex -output-directory=pdf\\temp{} pdf\\temp{}\\new.tex", id, id)])
        .output()
        .expect("Failed to run pdflatex");

    // Rename pdf output file in temp directory and move it to the main directory
    fs::rename(format!("pdf\\temp{}\\new.pdf", id), format!("pdf\\output{}.pdf", id));

    // Delete temp directory
    fs::remove_dir_all(format!("pdf\\temp{}", id));

    println!("{:?}", output);*/

    Some(id)
}