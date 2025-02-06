use std::{
    fs::{File, OpenOptions},
    path::PathBuf,
};

use crate::constants::PKG_NAME;

/// Creates a unique temporary file in the system's temporary directory.
/// Returns the file handle and the file path.
pub fn create_unique_temp_file() -> (File, PathBuf) {
    let temp_dir = std::env::temp_dir();
    let suffix = format!("{}tmp", PKG_NAME);

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

pub fn append_str_to_file(file_path: &PathBuf, s: &str) -> std::io::Result<usize> {
    use std::fs::OpenOptions;
    use std::io;

    let mut file = OpenOptions::new()
        .write(true) // Enable writing
        .append(true) // Enable appending
        .open(file_path)?; // Open the file

    io::Write::write(&mut file, s.as_bytes()) // Write the line with a newline at the end
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
}
