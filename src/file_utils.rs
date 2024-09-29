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

pub fn create_log_file_if_not_exists(log_file_path: &PathBuf, verbose: bool) {
    if log_file_path.exists() {
        if verbose {
            println!("Found log file '{}'", log_file_path.display());
        }
        return;
    }

    if verbose {
        println!("Creating log file");
    }

    let result = File::create(log_file_path);
    if result.is_err() {
        println!("Can't create the file. \n{}", result.err().unwrap());
    }
}

pub fn append_line_to_file(file_path: &PathBuf, line: &str) -> std::io::Result<usize> {
    use std::fs::OpenOptions;
    use std::io;

    let mut file = OpenOptions::new()
        .write(true) // Enable writing
        .append(true) // Enable appending
        .open(file_path)?; // Open the file

    io::Write::write(&mut file, line.as_bytes()) // Write the line with a newline at the end
}
