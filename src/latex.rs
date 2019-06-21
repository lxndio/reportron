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
    fn append_content(&mut self, content: String) { self.content += &content; }
}

fn eval_foreach(foreach: &ForEach, collections: &HashMap<String, Vec<HashMap<String, String>>>) -> Result<String, String> {
    Ok("hi".to_string())
}

fn evaluate(file: &str, gen_req: &Json<GenerationRequest>, keys: &HashMap<String, String>, collections: &HashMap<String, Vec<HashMap<String, String>>>) -> Result<String, String> {
    let mut new_file = String::new();
    let mut foreaches: Vec<ForEach> = Vec::new();

    // Replace all keys
    for line in file.lines() {
        let line = line.trim();

        let re = Regex::new(r"#\[(\S+)\]").unwrap();
        new_file += &format!("{}\n", re.replace_all(line, |caps: &Captures| keys.get(&caps[1]).expect("Key not found")));
    }

    // Search for line that matches foreach pattern
    let re_begin = Regex::new(r"#\[foreach (\S+) in (\S+)\]").unwrap();
    let re_end = Regex::new(r"#\[end foreach\]").unwrap();
    let file = new_file;
    let mut new_file = String::new();
    let mut foreach_stack: Vec<String> = Vec::new();

    for line in file.lines() {
        if re_begin.is_match(line) {
            // Store variables read from regex capture groups
            let caps = re_begin.captures(line).unwrap();
            let (single_var, collection_var) = (
                caps.get(1).map_or("", |m| m.as_str()),
                caps.get(2).map_or("", |m| m.as_str()),
            );

            // Fail if any of the capture groups was empty (TODO can this happen?)
            if single_var == "" || collection_var == "" {
                return Err("Syntax error in foreach".to_string());
            }

            // Create ForEach object and place marker in new_file or outer ForEach's content
            foreaches.push(ForEach::new(single_var.to_string(), collection_var.to_string()));
            foreach_stack.push(single_var.to_string());

            if foreach_stack.last().is_none() {
                new_file += &format!("#![FE {}]\n", single_var);
            } else {
                for foreach in foreaches.iter_mut() {
                    if foreach.single_var() == foreach_stack.last().unwrap() {
                        foreach.append_content(format!("#![FE {}]\n", single_var));
                        break;
                    }
                }
            }
        } else if re_end.is_match(line) {
            // Fail if there is no ForEach to end
            if foreach_stack.pop().is_none() {
                return Err("end foreach without foreach".to_string());
            }
            foreach_stack.pop();
        } else {
            if foreach_stack.last().is_none() {
                // Add line to new_file if it didn't contain a ForEach header
                new_file += &format!("{}\n", line);
            } else {
                // Add it to the innermost ForEach's content instead if necessary
                for foreach in foreaches.iter_mut() {
                    if foreach.single_var() == foreach_stack.last().unwrap() {
                        foreach.append_content(format!("{}\n", line));
                        break;
                    }
                }
            }
        }
    }

    // Evaluate ForEaches
    let re = Regex::new(r"#!\[FE (\S+)\]").unwrap();
    let mut lines: Vec<&str> = new_file.lines().collect::<Vec<&str>>();
    for i in 0..lines.len() {
        if re.is_match(lines[i]) {
            // Remove marker
            lines.remove(i);

            // Find corresponding ForEach
            for foreach in foreaches.iter() {
                if foreach.single_var() == re.captures(lines[i]).unwrap().get(1).map_or("", |m| m.as_str()) {

                }
            }
            lines.insert(i, eval_foreach(foreach, collections));
        }
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