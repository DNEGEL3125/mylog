use std::{fs::File, io::Read, path::PathBuf};

pub fn get_file_content_by_path(file_path: &PathBuf) -> String {
    // Open the file
    let mut file = File::open(file_path).expect("Can't open the file");
    // Read the content of the file into a string
    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Can't read the file");

    return content;
}