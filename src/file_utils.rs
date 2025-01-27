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

#[cfg(test)]
mod test {
    #[test]
    fn test_gen_temp_file_path() {
        use super::gen_temp_file_path;
        use std::collections::HashSet;
        use std::path::PathBuf;

        let num_files_gen = 1000;
        let mut file_path_set: HashSet<PathBuf> = HashSet::new();

        // Generate a large number of temporary file paths and check for uniqueness.
        for _ in 0..num_files_gen {
            let file_path = gen_temp_file_path();
            // Attempt to create the file. This will fail if the path already exists,
            // which would indicate a collision in the generated paths.
            std::fs::File::create_new(&file_path)
                .expect("The file already exists, indicating a path collision.");
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
        use super::{gen_temp_file_path, get_file_content_by_path};
        use std::io::Write;
        let file_path = gen_temp_file_path();
        let mut output_file = std::fs::File::create_new(&file_path)
            .expect("The file already exists, indicating a path collision.");
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
