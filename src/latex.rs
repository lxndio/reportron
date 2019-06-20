use std::fs;
use std::collections::HashMap;
use std::process::Command;
use rocket_contrib::json::Json;
use crate::generation_request::GenerationRequest;

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

fn evaluate(part: &str, gen_req: &Json<GenerationRequest>, foreaches: &mut Vec<ForEach>) -> Option<String> {
    let mut new_part = String::new();
    let mut status = 0;
    let mut key = String::new();

    for c in part.chars() {
        match c {
            '#' => {
                if status == 0 { status = 1 }
            },
            '[' => {
                if status == 1 { status = 2 }
                else { new_part.push(c) }
            },
            ']' => {
                if status == 2 { status = 3 }
                else { new_part.push(c) }
            },
            _   => {
                if status == 0 {
                    new_part.push(c);
                } else if status == 1 {
                    status = 0;
                    new_part.push('#');
                    new_part.push(c);
                } else if status == 2 {
                    key.push(c);
                } else if status == 3 {
                    // If there are any spaces, the key has to be a command and not a simple key
                    if key.contains(' ') {
                        let parts: Vec<&str> = key.trim().split_whitespace().collect();

                        if parts.len() == 4 && parts.get(0).unwrap() == &"foreach" {
                            let foreach_part = String::new();
                            // read all chars until #[end foreach]

                            new_part += &match evaluate(&foreach_part, gen_req, foreaches) {
                                Some(part) => part,
                                None => return None,
                            };
                            /*if parts.get(2).unwrap() == &"in" {
                                // TODO
                            } else {
                                return None; // Invalid syntax
                            }*/
                        } else if parts.len() == 2 && parts.get(0).unwrap() == &"end" {
                            match parts.get(1).unwrap() {
                                &"foreach" => foreaches.pop(),
                                _ => return None, // Invalid keyword used
                            };
                        }
                    } else {
                        // TODO make not hardcoded
                        if key == "date" {
                            new_part += &gen_req.date;
                        } else {
                            return None; // invalid key used
                        }
                    }

                    status = 0;
                    new_part.push(c);
                }
            }
        }
    }

    Some(new_part)
}

pub fn generate_latex(gen_req: &Json<GenerationRequest>, collections: HashMap<String, Vec<HashMap<String, String>>>) -> Option<String> {
    let id = "12345".to_string(); // generate random id

    // Create temp directory for output of this job
    fs::create_dir(format!("pdf\\temp{}", id));

    // Create empty list of foreach objects
    let mut foreaches: Vec<ForEach> = Vec::new();

    // Read template file and replace the keys
    let file = fs::read_to_string("templates\\test.tex").expect("Could not read template file");
    let mut new_file = String::new();
    let mut status = 0;
    let mut key = String::new();
    for c in file.chars() {
        match c {
            '#' => {
                if status == 0 { status = 1 }
            },
            '[' => {
                if status == 1 { status = 2 }
                else { new_file.push(c) }
            },
            ']' => {
                if status == 2 { status = 3 }
                else { new_file.push(c) }
            },
            _   => {
                if status == 0 {
                    new_file.push(c);
                } else if status == 1 {
                    status = 0;
                    new_file.push('#');
                    new_file.push(c);
                } else if status == 2 {
                    key.push(c);
                } else if status == 3 {
                    // If there are any spaces, the key has to be a command and not a simple key
                    if key.contains(' ') {
                        let parts: Vec<&str> = key.trim().split_whitespace().collect();

                        if parts.len() == 4 && parts.get(0).unwrap() == &"foreach" {
                            if parts.get(2).unwrap() == &"in" {
                                // TODO recursively run a function for validating the part inside of the foreach loop
                                // e.g. validate(part, &mut foreaches) // Pass &mut foreaches to have access to nested foreaches
                            } else {
                                return None; // Invalid syntax
                            }
                        } else if parts.len() == 2 && parts.get(0).unwrap() == &"end" {
                            match parts.get(1).unwrap() {
                                &"foreach" => foreaches.pop(),
                                _ => return None, // Invalid keyword used
                            };
                        }
                    } else {
                        // TODO make not hardcoded
                        if key == "date" {
                            new_file += &gen_req.date;
                        } else {
                            return None; // invalid key used
                        }
                    }

                    status = 0;
                    new_file.push(c);
                }
            }
        }
    }

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