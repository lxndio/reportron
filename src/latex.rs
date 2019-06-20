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
    id: usize,
    current_pos: usize,
    done: bool,
}

impl ForEach {
    fn new(single_var: String, collection_var: String, id: usize) -> ForEach {
        ForEach {
            single_var,
            collection_var,
            content: String::new(),
            id,
            current_pos: 0,
            done: false,
        }
    }

    fn single_var(&self) -> &String { &self.single_var }

    fn set_content(&mut self, content: String) { self.content = content; }
    fn content(&self) -> &String { &self.content }

    fn id(&self) -> &usize { &self.id }

    fn set_done(&mut self) { self.done = true }

    fn done(&self) -> &bool { &self.done }
}

fn foreach_eval(foreach: &ForEach, collections: &HashMap<String, Vec<HashMap<String, String>>>) -> String {
    let mut new_content = String::new();

    for line in foreach.content().lines() {
        let re = Regex::new(r"#\[(\S+) of (\S+)\]").unwrap();
        // TODO replace with field from collections here and increase current_pos in ForEach object by one
        new_content += &format!("{}{}", re.replace_all(line, |caps: &Captures| keys.get(&caps[1]).expect("Key not found")), "\n");
    }

    new_content
}

fn evaluate(part: &str, gen_req: &Json<GenerationRequest>, keys: &HashMap<String, String>, collections: &HashMap<String, Vec<HashMap<String, String>>>, foreaches: &mut Vec<ForEach>) -> Option<String> {
    let mut new_part = String::new();

    let mut foreach_id = 0;
    let mut skip;

    for line in part.lines() {
        let line = line.trim();
        skip = false;

        let re = Regex::new(r"#\[foreach (\S+) in (\S+)\]").unwrap();
        if let Some(cap) = re.captures(line) {
            foreaches.push(ForEach::new(cap[1].to_string(), cap[2].to_string(), foreach_id));
            foreach_id += 1;
            skip = true;
            new_part += &format!("{}\n", line);
        }

        let re = Regex::new(r"#\[end foreach (\S+)\]").unwrap();
        if let Some(cap) = re.captures(line) {
            for foreach in foreaches.iter_mut() {
                if foreach.single_var() == &cap[1] {
                    foreach.set_done();
                }
            }
            continue;
        }

        // Replace all normal keys
        let re = Regex::new(r"#\[(\S+)\]").unwrap();
        let new_line = &format!("{}{}", re.replace_all(line, |caps: &Captures| keys.get(&caps[1]).expect("Key not found")), "\n");

        let mut line_in_foreach = false;

        for foreach in foreaches.iter_mut() {
            if !foreach.done() && !skip {
                foreach.set_content(format!("{}{}", foreach.content(), new_line));
                line_in_foreach = true;
            }
        }

        if !line_in_foreach {
            new_part += new_line;
        }
    }

    let mut new_new_part = String::new();

    // Evaluate each for each loop's content
    for line in new_part.lines() {
        let re = Regex::new(r"#\[foreach (\S+) in (\S+)\]").unwrap();
        if let Some(cap) = re.captures(line) {
            for foreach in foreaches.iter() {
                if foreach.single_var == cap[1] {
                    new_new_part += &foreach_eval(foreach, collections);
                }
            }
        } else {
            new_new_part += &format!("{}\n", line);
        }
    }

    Some(new_new_part)
}

pub fn generate_latex(gen_req: &Json<GenerationRequest>, keys: &HashMap<String, String>, collections: &HashMap<String, Vec<HashMap<String, String>>>) -> Option<String> {
    let id = "12345".to_string(); // generate random id

    // Create temp directory for output of this job
    fs::create_dir(format!("pdf\\temp{}", id));

    // Create empty list of foreach objects
    let mut foreaches: Vec<ForEach> = Vec::new();

    // Read template file and replace the keys
    let file = fs::read_to_string("templates\\test.tex").expect("Could not read template file");
    
    let new_file = evaluate(&file, gen_req, keys, collections, &mut foreaches).expect("Error while evaluating");

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