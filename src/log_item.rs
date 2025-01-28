use std::{path::PathBuf, process::exit, str::FromStr};

use chrono::NaiveDateTime;
use crossterm::style::Stylize;

use crate::file_utils::append_line_to_file;

pub enum ParseError {
    DateNotFound,
}

#[derive(Debug, Default)]
pub struct LogItem {
    date_time: chrono::NaiveDateTime,
    content: String,
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

impl FromStr for LogItem {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let idx = match s.find("]") {
            Some(res) => res,
            None => {
                return Err(ParseError::DateNotFound);
            }
        };
        if !s.starts_with("[") {
            return Err(ParseError::DateNotFound);
        }

        let date_str = &s[1..idx];
        match chrono::NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M") {
            Ok(date_time_result) => {
                let log_content = s[idx + 1..].to_owned();
                return Ok(LogItem::new(date_time_result, &log_content));
            }
            Err(_) => {
                return Err(ParseError::DateNotFound);
            }
        }
    }
}

impl LogItem {
    pub fn new(date_time: NaiveDateTime, content: &str) -> Self {
        Self {
            date_time,
            content: content.trim().to_owned(),
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn append_to_file(&self, log_file_path: &PathBuf, verbose: bool) {
        let result = append_line_to_file(log_file_path, &self.to_string());
        if result.is_err() {
            println!("Can't write message to the log file");
            exit(3);
        }

        if verbose {
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

    pub fn to_colored_string(&self) -> String {
        let date_str = format!("[{}]", self.date_time.format("%Y-%m-%d %H:%M"));
        format!("{} {}", date_str.green(), self.content)
    }
}

pub struct LogItemList {
    items: Vec<LogItem>,
}

impl FromStr for LogItemList {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut log_items: Vec<LogItem> = Vec::new();
        let mut current_log = String::new();

        for line in s.lines() {
            if LogItem::from_str(line).is_err() {
                current_log.push('\n');
            } else {
                match LogItem::from_str(&current_log) {
                    Ok(parsed_item) => {
                        if !current_log.is_empty() {
                            log_items.push(parsed_item);
                            current_log.clear();
                        }
                    }
                    Err(_) => {}
                }
            }
            current_log.push_str(line);
        }

        match LogItem::from_str(&current_log) {
            Ok(parsed_item) => {
                if !current_log.is_empty() {
                    log_items.push(parsed_item);
                    current_log.clear();
                }
            }
            Err(_) => {}
        };

        Ok(LogItemList { items: log_items })
    }
}

impl LogItemList {
    pub fn iter(&self) -> std::slice::Iter<'_, LogItem> {
        self.items.iter()
    }

    pub(crate) fn new() -> Self {
        Self { items: Vec::new() }
    }
}
