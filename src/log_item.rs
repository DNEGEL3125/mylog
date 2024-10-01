use std::{path::PathBuf, process::exit};

use crate::file_utils::append_line_to_file;

#[derive(Debug)]
pub struct LogItem {
    pub date_time: chrono::NaiveDateTime,
    pub content: String,
}

impl std::fmt::Display for LogItem {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let data = format!(
            "[{}] {}\n",
            self.date_time.format("%Y-%m-%d %H:%M"),
            self.content
        );

        fmt.write_str(&data)
    }
}

impl LogItem {
    pub fn append_to_file(&self, log_file_path: &PathBuf, quiet: bool, verbose: bool) {
        let result = append_line_to_file(log_file_path, &self.to_string());
        if result.is_err() {
            println!("Can't write message to the log file");
            exit(3);
        }

        if quiet {
        } else if verbose {
            println!(
                r#"Written the log message to "{}""#,
                log_file_path.display()
            );
        } else {
            println!(
                r#"Written the log message to "{}""#,
                log_file_path
                    .file_name()
                    .expect("Isn't a filename")
                    .to_str()
                    .expect("Invalid Unicode")
            );
        }
    }
}
