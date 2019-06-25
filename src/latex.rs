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
    objects: Vec<HashMap<String, String>>,
    current_obj: usize,
}

impl ForEach {
    fn new(single_var: String, collection_var: String) -> ForEach {
        ForEach {
            single_var,
            collection_var,
            content: String::new(),
            objects: Vec::new(),
            current_obj: 0,
        }
    }

    fn single_var(&self) -> &String { &self.single_var }

    fn collection_var(&self) -> &String { &self.collection_var }

    fn content(&self) -> &String { &self.content }
    fn append_content(&mut self, content: String) { self.content += &content; }

    fn gen_objects(&mut self, collections: &HashMap<String, Vec<HashMap<String, String>>>) -> Result<(), String> {
        self.objects = match collections.get(self.collection_var()) {
            Some(obj) => obj.to_vec(),
            None => return Err(format!("Collection variable {} of foreach not found", self.collection_var())),
        };

        Ok(())
    }

    fn has_next_obj(&self) -> bool {
        (0..self.objects.len()-1).contains(&self.current_obj)
    }

    fn next_obj(&mut self) -> Option<&HashMap<String, String>> {
        if self.has_next_obj() {
            self.current_obj += 1;
            Some(&self.objects[self.current_obj-1])
        } else {
            None
        }
    }

    fn current_obj(&self) -> &HashMap<String, String> {
        &self.objects[self.current_obj]
    }

    // Get a ForEach from a &Vec<ForEach> by single_var
    fn get_from(foreaches: &Vec<ForEach>, single_var: String) -> Option<&ForEach> {
        for foreach in foreaches.iter() {
            if foreach.single_var() == &single_var {
                return Some(&foreach);
            }
        }

        None
    }

    fn get_from_mut(foreaches: &mut Vec<ForEach>, single_var: String) -> Option<&mut ForEach> {
        for foreach in foreaches.iter_mut() {
            if foreach.single_var() == &single_var {
                return Some(foreach);
            }
        }

        None
    }
}

struct Evaluator {
    foreaches: Vec<ForEach>,
}

impl Evaluator {
    fn new() -> Evaluator {
        Evaluator {
            foreaches: Vec::new(),
        }
    }

    fn eval_foreach(&mut self, foreach: String, collections: &HashMap<String, Vec<HashMap<String, String>>>) -> Result<String, String> {
        let re_fe = Regex::new(r"#!\[FE (\S+)\]").unwrap();
        let re_of = Regex::new(r"#\[(\S+) of (\S+)\]").unwrap();

        //let foreach: &mut ForEach = ForEach::get_from_mut(&mut self.foreaches, foreach).unwrap();
        let foreach: &str = &foreach;
        let mut lines: Vec<String> = ForEach::get_from(&self.foreaches, foreach.to_string()).unwrap().content().lines().map(|l| l.to_string()).collect::<Vec<String>>();
        let mut new_lines: Vec<String> = lines.clone(); // TODO probably undo by replacing new_lines with lines everywhere and deleting doubles

        match ForEach::get_from_mut(&mut self.foreaches, foreach.to_string()).unwrap().gen_objects(collections) {
            Ok(()) => (),
            Err(e) => return Err(e),
        };

        while ForEach::get_from(&self.foreaches, foreach.to_string()).unwrap().has_next_obj() {
            for i in 0..lines.len() {
                // If there is a ForEach nested in this one, evaluate it recursively
                if re_fe.is_match(&lines[i]) {
                    // Remove marker
                    //lines.remove(i);
                    new_lines.remove(i);

                    // Find corresponding ForEach
                    let mut found = false;
                    // Loop of the foreaches using single var as the key to prevent direct referencing
                    for foreach in self.foreaches.iter().map(|fe| fe.single_var().to_string()) {
                        if foreach == re_fe.captures(&lines[i]).unwrap().get(1).map_or("", |m| m.as_str()) {
                            new_lines.insert(i, match self.eval_foreach(foreach, collections) {
                                Ok(l) => l,
                                Err(e) => return Err(e),
                            });
                            found = true;
                            break;
                        }
                    }

                    if !found {
                        return Err("Internal error: invalid foreach marker".to_string());
                    }
                } else {
                    ForEach::get_from_mut(&mut self.foreaches, foreach.to_string()).unwrap().next_obj();
                    // TODO make safe (remove unwraps)
                    new_lines[i] = re_of.replace_all(
                        &lines[i],
                        |caps: &Captures|
                            ForEach::get_from(
                                &self.foreaches,
                                caps.get(2).unwrap().as_str().to_string())
                                .expect("Couldn't find element in foreaches")
                                .current_obj().get(&caps[1])
                                .unwrap()).to_string();
                }
            }
        }

        let mut res = String::new();
        for line in new_lines.iter() {
            res += &format!("\n{}", line);
        }

        Ok(res)
    }

    fn evaluate(&mut self, file: &str, gen_req: &Json<GenerationRequest>) -> Result<String, String> {
        let keys = &gen_req.keys;
        let collections = &gen_req.collections;

        let mut new_file = String::new();

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

                // Add marker for outermost ForEach to new_file or otherwise to outer ForEach loop
                if foreach_stack.last().is_none() {
                    new_file += &format!("#![FE {}]\n", single_var);
                } else {
                    for foreach in self.foreaches.iter_mut() {
                        if foreach.single_var() == foreach_stack.last().expect("Can't happen because of if condition") {
                            foreach.append_content(format!("#![FE {}]\n", single_var));
                            break;
                        }
                    }
                }

                // Create ForEach object and place marker in new_file or outer ForEach's content
                self.foreaches.push(ForEach::new(single_var.to_string(), collection_var.to_string()));
                foreach_stack.push(single_var.to_string());
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
                    for foreach in self.foreaches.iter_mut() {
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
        let mut lines: Vec<String> = new_file.lines().map(|l| l.to_string()).collect::<Vec<String>>();
        for i in 0..lines.len() {
            if re.is_match(&lines[i]) {
                // Find corresponding ForEach
                let mut found = false;
                for foreach in self.foreaches.iter().map(|fe| fe.single_var().to_string()) {
                    if foreach == re.captures(&lines[i]).unwrap().get(1).map_or("", |m| m.as_str()) {
                        // Remove marker and insert replacement
                        lines.remove(i);
                        lines.insert(i, match self.eval_foreach(foreach, collections) {
                            Ok(l) => l,
                            Err(e) => return Err(e),
                        });
                        found = true;
                        break;
                    }
                }

                if !found {
                    return Err("Internal error: invalid foreach marker".to_string());
                }
            }
        }

        let mut res = String::new();
        for line in lines.iter() {
            res += &format!("\n{}", line);
        }

        Ok(res)
    }
}

pub fn generate_latex(gen_req: &Json<GenerationRequest>) -> Option<String> {
    let id = Uuid::new_v4().to_string(); // generate random id
    println!("using id: {:?}", id);

    // Create temp directory for output of this job
    let temp_dir_path = Path::new("pdf").join(format!("temp-{}", id));
    fs::create_dir(temp_dir_path.as_path()).expect("Could not create temp dir");

    // Read template file and replace the keys
    let template_path = Path::new("templates").join(format!("{}.tex", gen_req.template)); // TODO: Take care of UNIX path in gen_req.template
    let file = fs::read_to_string(template_path).expect("Could not read template file");
    
    // Evaluate the template file using the data from gen_req
    let mut eval = Evaluator::new();
    let new_file = eval.evaluate(&file, gen_req).expect("Error while evaluating");

    // Write new file to temp directory
    let tex_output_path = temp_dir_path.join("new.tex");
    fs::write(tex_output_path.as_path(), new_file).expect("Could not write new file");

    /*let command = format!("pdflatex -output-directory={} {}", temp_dir_path.to_str().expect("Failed"), tex_output_path.to_str().expect("Failed")).to_string();
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

    println!("{:?}", output);*/

    Some(id)
}