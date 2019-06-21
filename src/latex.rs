use std::fs;
use std::collections::HashMap;
use std::process::Command;
use rocket_contrib::json::Json;
use regex::{Regex, Captures};
use crate::generation_request::GenerationRequest;

struct ForEach {
    single_var: String,
    collection_var: String,
    content: String,
}

impl ForEach {
    fn new(single_var: String, collection_var: String) -> ForEach {
        ForEach {
            single_var,
            collection_var,
            content: String::new(),
        }
    }

    fn single_var(&self) -> &String { &self.single_var }

    fn collection_var(&self) -> &String { &self.collection_var }

    fn set_content(&mut self, content: String) { self.content = content; }
    fn content(&self) -> &String { &self.content }
}

fn evaluate(file: &str, gen_req: &Json<GenerationRequest>, keys: &HashMap<String, String>, collections: &HashMap<String, Vec<HashMap<String, String>>>) -> Result<String, String> {
    let mut new_file = String::new();

    for line in file.lines() {
        let line = line.trim();

        // Replace all normal keys
        let re = Regex::new(r"#\[(\S+)\]").unwrap();
        new_file += &format!("{}{}", re.replace_all(line, |caps: &Captures| keys.get(&caps[1]).expect("Key not found")), "\n");
    }

    Ok(new_file)
}

pub fn generate_latex(gen_req: &Json<GenerationRequest>, keys: &HashMap<String, String>, collections: &HashMap<String, Vec<HashMap<String, String>>>) -> Option<String> {
    let id = "12345".to_string(); // generate random id

    // Create temp directory for output of this job
    fs::create_dir(format!("pdf\\temp{}", id));

    // Read template file and replace the keys
    let file = fs::read_to_string("templates\\test.tex").expect("Could not read template file");
    
    let new_file = evaluate(&file, gen_req, keys, collections).expect("Error while evaluating");

    // Write new file to temp directory
    fs::write(format!("pdf\\temp{}\\new.tex", id), new_file).expect("Could not write new file");

    /*let output = Command::new("cmd")
        .args(&["/C", &format!("pdflatex -output-directory=pdf\\temp{} pdf\\temp{}\\new.tex", id, id)])
        .output()
        .expect("Failed to run pdflatex");

    // Rename pdf output file in temp directory and move it to the main directory
    fs::rename(format!("pdf\\temp{}\\new.pdf", id), format!("pdf\\output{}.pdf", id));

    // Delete temp directory
    fs::remove_dir_all(format!("pdf\\temp{}", id));

    println!("{:?}", output);*/

    Some("12345".to_string())
}