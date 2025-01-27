use std::{
    fs::{File, OpenOptions},
    io::Read,
    path::PathBuf,
    process::exit,
};

/// Creates a unique temporary file in the system's temporary directory.
/// Returns the file handle and the file path.
pub fn create_unique_temp_file() -> (File, PathBuf) {
    let temp_dir = std::env::temp_dir();
    let suffix = "mylogtmp";

    for i in 0.. {
        let filename = format!("{}{}", suffix, i);
        let temp_file_path = temp_dir.join(&filename);

        // Attempt to create the file atomically
        match OpenOptions::new()
            .create_new(true) // Ensures atomic file creation
            .write(true)
            .open(&temp_file_path)
        {
            Ok(file) => return (file, temp_file_path), // Return the file and its path
            Err(_) => continue, // File exists or other error, try the next iteration
        }
    }

    unreachable!("Ran out of unique temporary file names");
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

#[cfg(test)]
mod test {
    #[test]
    fn test_create_unique_temp_file() {
        use super::create_unique_temp_file;
        use std::collections::HashSet;
        use std::path::PathBuf;

        let num_files_gen = 1000;
        let mut file_path_set: HashSet<PathBuf> = HashSet::new();

        // Generate a large number of temporary file paths and check for uniqueness.
        for _ in 0..num_files_gen {
            let (_, file_path) = create_unique_temp_file();
            assert!(file_path.exists());
            // Check if the generated path is unique. The insert method returns true if the
            // element was not already present in the set.
            assert!(
                file_path_set.insert(file_path.clone()),
                "Generated duplicate file path: {:?}",
                file_path
            );
        }

        // Clean up the created temporary files.
        for file_path in file_path_set {
            std::fs::remove_file(file_path).expect("Unable to delete the created temporary files");
        }
    }

    #[test]
    fn test_get_file_content_by_path() {
        use super::{create_unique_temp_file, get_file_content_by_path};
        use std::io::Write;
        let (mut output_file, file_path) = create_unique_temp_file();

        let file_content = r#"The darkest valley
            The highest mountain
            We walk in the name of our brave
            The rushing river, the blooming flower
            Descended from heaven we embrace"#;

        output_file
            .write_all(file_content.as_bytes())
            .expect(&format!("Unable to write the file {:?}", file_path));
        std::mem::drop(output_file);

        assert_eq!(get_file_content_by_path(&file_path), file_content);

        // Clean up the created temporary file.
        std::fs::remove_file(file_path).expect("Unable to delete the created temporary files");
    }
}
