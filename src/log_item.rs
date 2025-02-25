use std::{path::PathBuf, str::FromStr};

use chrono::NaiveDateTime;

use crate::utils::fs::append_str_to_file;

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
                Ok(LogItem::new(date_time_result, &log_content))
            }
            Err(_) => Err(ParseError::DateNotFound),
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

    pub fn date_time(&self) -> &NaiveDateTime {
        &self.date_time
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn append_to_file(&self, log_file_path: &PathBuf) -> Result<(), String> {
        append_str_to_file(log_file_path, &self.to_string())
            .map_err(|_| {
                format!(
                    "Unable to write the message to `{}`",
                    log_file_path.display()
                )
            })
            .map(|_| ())
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
            if current_log.is_empty() {
                // Do nothing
            } else if LogItem::from_str(line).is_err() {
                current_log.push('\n');
            } else if let Ok(parsed_item) = LogItem::from_str(&current_log) {
                log_items.push(parsed_item);
                current_log.clear();
            }
            current_log.push_str(line);
        }

        if current_log.is_empty() {
            // Do nothing
        } else if let Ok(parsed_item) = LogItem::from_str(&current_log) {
            log_items.push(parsed_item);
            current_log.clear();
        }

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

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use chrono::NaiveDateTime;

    use super::LogItemList;

    #[test]
    fn test_log_item_list_from_str() {
        let s = r#"[2023-5-23 01:33] qwq
[2023-12-1 11:22] mylog
[2024-1-2 14:59] test"#;

        let dates = ["2023-5-23 01:33", "2023-12-1 11:22", "2024-1-2 14:59"]
            .map(|x| NaiveDateTime::parse_from_str(x, "%Y-%m-%d %H:%M").unwrap());
        let contetns = ["qwq", "mylog", "test"];

        let log_item_list = LogItemList::from_str(s).unwrap();
        for (i, item) in log_item_list.iter().enumerate() {
            assert_eq!(item.date_time(), dates.get(i).unwrap());
            assert_eq!(item.content(), contetns[i])
        }
    }
}
