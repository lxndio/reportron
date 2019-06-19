use std::fs;
use std::fs::File;
use std::process::Command;
use rocket_contrib::json::Json;
use crate::generation_request::GenerationRequest;

pub fn generate_latex(gen_req: &Json<GenerationRequest>) -> Option<String> {
    let id = "12345".to_string(); // generate random id

    // Create temp directory for output of this job
    fs::create_dir(format!("pdf\\temp{}", id));

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
                    new_file.push('#'); // TODO necessary?
                    new_file.push(c);
                } else if status == 2 {
                    key.push(c);
                } else if status == 3 {
                    // TODO make not hardcoded
                    if key == "date" {
                        new_file += &gen_req.date;
                    } else {
                        return None; // invalid key used
                    }

                    status = 0;
                    new_file.push(c);
                }
            }
        }
    }

    // Write new file to temp directory
    fs::write(format!("pdf\\temp{}\\new.tex", id), new_file).expect("Could not write new file");

    let output = Command::new("cmd")
        .args(&["/C", &format!("pdflatex -output-directory=pdf\\temp{} pdf\\temp{}\\new.tex", id, id)])
        .output()
        .expect("Failed to run pdflatex");

    // Rename pdf output file in temp directory and move it to the main directory
    fs::rename(format!("pdf\\temp{}\\new.pdf", id), format!("pdf\\output{}.pdf", id));

    // Delete temp directory
    fs::remove_dir_all(format!("pdf\\temp{}", id));

    println!("{:?}", output);

    Some("12345".to_string())
}