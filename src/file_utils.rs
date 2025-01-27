use std::{fs::File, io::Read, path::PathBuf, process::exit};

pub fn gen_temp_file_path() -> PathBuf {
    use std::env;
    let temp_dir = env::temp_dir();
    let mut i: usize = 0;
    let suffix = "mylogtmp";
    let mut temp_file_path: PathBuf = temp_dir.join(suffix);
    while temp_file_path.exists() {
        let filename = format!("{}{}", suffix, i);
        temp_file_path = temp_dir.join(filename);
        i += 1;
    }
    temp_file_path
}

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
        println!("Can't create the log file.");
        if verbose {
            println!("{}", result.err().unwrap());
        }
        exit(-3034);
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
