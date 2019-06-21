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
    let id = Uuid::new_v4().to_string(); // generate random id
    println!("using id: {:?}", id);

    // Create temp directory for output of this job
    let temp_dir_path = Path::new("pdf").join(format!("temp-{}", id));
    fs::create_dir(temp_dir_path.as_path()).expect("Could not create temp dir");

    // Read template file and replace the keys
    let template_path = Path::new("templates").join(format!("{}.tex", gen_req.template)); // TODO: Take care of UNIX path in gen_req.template
    let file = fs::read_to_string(template_path).expect("Could not read template file");
    
    let new_file = evaluate(&file, gen_req, keys, collections).expect("Error while evaluating");

    // Write new file to temp directory
    let tex_output_path = temp_dir_path.join("new.tex");
    fs::write(tex_output_path.as_path(), new_file).expect("Could not write new file");

    let command = format!("pdflatex -output-directory={} {}", temp_dir_path.to_str().expect("Failed"), tex_output_path.to_str().expect("Failed")).to_string();
    println!("command: {}", command);
    let output = Command::new("cmd")
        .args(&["/C", &format!("{}", command)])
        .output()
        .expect("Failed to run pdflatex");

    // Rename pdf output file in temp directory and move it to the main directory
    let pdf_temp_path = tex_output_path.with_extension("pdf");
    let pdf_output_path = Path::new("pdf").join(format!("output-{}.pdf", id));
    fs::rename(pdf_temp_path.to_str().expect("Failed"), pdf_output_path.to_str().expect("Failed")).expect("Failed to rename and move output pdf file");

    // Delete temp directory
    fs::remove_dir_all(temp_dir_path).expect("Failed to remove temp dir");

    println!("{:?}", output);

    Some(id)
}