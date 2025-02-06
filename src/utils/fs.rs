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
        .append(true) // Enable appending
        .open(file_path)?; // Open the file

    io::Write::write(&mut file, s.as_bytes()) // Write the line with a newline at the end
}

#[cfg(test)]
mod test {
    use std::{fs::read_to_string, path::PathBuf};

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

    struct TestAppendStrToFileConfig {
        file_path: PathBuf,
    }

    impl Drop for TestAppendStrToFileConfig {
        fn drop(&mut self) {
            std::fs::remove_file(&self.file_path).unwrap_or_else(|err| eprintln!("{}", err));
        }
    }

    #[test]
    fn test_append_str_to_file() {
        use super::append_str_to_file;

        let append_lines_count = 100;
        let init_file_content = r#"道 可 道 ， 非 常 道 。 名 可 名 ， 非 常 名 。
无 名 天 地 之 始 ﹔ 有 名 万 物 之 母 。
故 常 无 ， 欲 以 观 其 妙 ﹔ 常 有 ， 欲 以 观 其 徼 。
此 两 者 ， 同 出 而 异 名 ， 同 谓 之 玄 。
玄 之 又 玄 ， 众 妙 之 门 。"#;

        let string_to_append = "I am the string to append";
        let (_, file_path) = super::create_unique_temp_file();
        let test_config = TestAppendStrToFileConfig { file_path };
        let file_path = &test_config.file_path;
        let mut expected_file_content: String = init_file_content.to_owned();
        append_str_to_file(file_path, init_file_content).expect("Fail to write the file");
        for _ in 0..append_lines_count {
            append_str_to_file(file_path, string_to_append).expect("Fail to write the file");
            expected_file_content.push_str(string_to_append);
        }

        let final_file_content = read_to_string(file_path).expect("Fail to read the file");
        assert_eq!(final_file_content, expected_file_content);
    }
}
