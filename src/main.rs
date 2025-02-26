use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;
use std::str::FromStr;

use crate::error::Error;
use chrono::NaiveDate;
use clap::Parser;
use config::{construct_log_file_path, Config};
use constants::CONFIG_FILE_PATH;
use log_item::LogItem;
use log_pager::paging_all_pager::PagingAllPager;
use log_pager::single_date_pager::SingleDatePager;
use utils::fs::append_str_to_file;
use utils::time::{date_time_now, get_today_date};

pub mod cli;
pub mod config;
pub mod constants;
pub mod error;
pub mod log_item;
pub mod log_pager;
pub mod utils;

fn paging_log_file_by_date(log_dir_path: &PathBuf, date: NaiveDate, verbose: bool) {
    let mut log_pager = SingleDatePager::new(date, log_dir_path.to_owned());
    log_pager.set_verbose(verbose);
    log_pager.run();
}

fn parse_date_from_str(date_str: &str) -> Result<NaiveDate, chrono::ParseError> {
    let today = get_today_date();
    let date_fmt = "%Y-%m-%d";
    NaiveDate::parse_from_str(date_str, date_fmt).or(NaiveDate::parse_from_str(
        &format!("{}-{}", today.format("%Y"), date_str),
        date_fmt,
    ))
}

fn view_logs<P: AsRef<Path>>(
    date_str: Option<String>,
    all: bool,
    verbose: bool,
    log_dir_path: P,
) -> Result<(), Error> {
    let today_date = get_today_date();
    if !log_dir_path.as_ref().exists() {
        return Err(Error::LogDirNotFound(log_dir_path.as_ref().into()));
    }

    if all {
        PagingAllPager::new(log_dir_path.as_ref().to_path_buf()).run();
        return Ok(());
    }

    let date = match date_str {
        Some(date_str) => parse_date_from_str(&date_str).map_err(|_| Error::DateParse(date_str))?,
        // Default date is today
        None => today_date,
    };

    paging_log_file_by_date(&log_dir_path.as_ref().to_path_buf(), date, verbose);
    Ok(())
}

fn write_log(log_content: &str, verbose: bool, log_dir_path: &Path) -> Result<(), String> {
    let date_time_now = date_time_now();
    let today_date = date_time_now.date();

    if !log_dir_path.exists() {
        let config_file_path = &CONFIG_FILE_PATH;
        return Err(format!(
            "The log directory '{}' doesn't exist.\nYou can configure it in '{}'",
            log_dir_path.display(),
            config_file_path.display()
        ));
    }

    let log_file_path = construct_log_file_path(log_dir_path, &today_date);

    // If the log file does not exist, create it
    let _ = std::fs::File::create_new(&log_file_path);

    let log_item = LogItem::new(date_time_now, log_content);
    if verbose {
        println!("Log info: {:#?}\nWriting the log message...", log_item);
    }

    append_str_to_file(&log_file_path, &log_item.to_string()).map_err(|error| error.to_string())?;

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
    Ok(())
}

fn edit_logs(date_str: Option<String>, verbose: bool, log_dir_path: &Path) -> Result<(), String> {
    let today_date = get_today_date();

    let date = match date_str {
        Some(date_str) => parse_date_from_str(&date_str).map_err(|error| error.to_string())?,
        // Default date is today
        None => today_date,
    };

    if !log_dir_path.exists() {
        let config_file_path = &CONFIG_FILE_PATH;
        return Err(format!(
            "The log directory '{}' doesn't exist.\nYou can configure it in '{}'",
            log_dir_path.display(),
            config_file_path.display()
        ));
    }

    let log_file_path = construct_log_file_path(log_dir_path, &date);

    // If the log file does not exist, create it
    let _ = std::fs::File::create_new(&log_file_path);

    if verbose {
        let editor_path_res = edit::get_editor();
        match editor_path_res {
            Ok(editor_path) => {
                println!("Opening editor: {}", editor_path.display());
            }
            Err(_) => {
                println!("Can't find the editor");
            }
        }
    }

    edit::edit_file(log_file_path).or(Err(String::from("Unable to edit the file")))
}

fn run() -> Result<(), String> {
    // Command line parameters
    let cli = cli::Cli::parse();

    Config::create_config_file_if_not_exists();
    let config_file_path = &crate::constants::CONFIG_FILE_PATH;
    let config = config::Config::from_config_file(config_file_path.as_path())?;
    let log_dir_path = PathBuf::from_str(&config.log.dir).expect("Incorrect path");

    match cli.command {
        cli::Commands::View { date, verbose, all } => {
            view_logs(date, all, verbose, &log_dir_path).map_err(|error| error.to_string())?;
        }
        cli::Commands::Write { message, verbose } => {
            let message_string = if let Some(message_string) = message {
                message_string
            } else {
                input_log_message()
            };

            if message_string.trim().is_empty() {
                return Err(String::from("Aborting due to empty log message."));
            }
            write_log(&message_string, verbose, &log_dir_path)?;
        }
        cli::Commands::Config { key, value } => match value {
            Some(value) => {
                config::set_by_key(config_file_path, &key, value)?;
            }
            None => {
                if let Some(value) = config.get_by_key(&key) {
                    println!("{}", value)
                } else {
                    Err(format!("invalid key: {}", key))?
                }
            }
        },
        cli::Commands::Edit { date, verbose } => {
            edit_logs(date, verbose, &log_dir_path)?;
        }
    };
    Ok(())
}

fn main() -> ExitCode {
    if let Err(error_message) = run() {
        eprintln!("error: {}", error_message);
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// Opens a temporary file in the user's default editor, waits for editing to complete,
/// reads the edited content, deletes the temporary file, and returns the content.
///
/// # Returns
/// A `String` containing the content of the temporary file after the user finishes editing.
///
/// # Errors
/// Panics if there is an issue creating, reading, or editing the temporary file.
fn input_log_message() -> String {
    use std::fs::{self, File};
    use std::io::{Read, Write};

    use edit::edit_file;

    // Create a temporary file
    let (mut temp_file, temp_file_path) = crate::utils::fs::create_unique_temp_file();

    // Optionally add an initial message
    writeln!(
        temp_file,
        "\n# Enter your log message here.\n# Lines starting with '#' will be ignored."
    )
    .expect("Failed to write initial content to the temporary file");
    drop(temp_file); // Close the file so it can be opened by the editor

    // Open the file in the user's default editor
    edit_file(&temp_file_path).expect("Failed to open the file in the editor");

    // Read the edited content
    let mut edited_content = String::new();
    let mut temp_file = File::open(&temp_file_path).expect("Failed to open the temporary file");
    temp_file
        .read_to_string(&mut edited_content)
        .expect("Failed to read the content from the temporary file");

    // Delete the temporary file
    fs::remove_file(&temp_file_path).expect("Failed to delete the temporary file");

    // Filter out comment lines
    let cleaned_content: String = edited_content
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");

    cleaned_content
}

#[cfg(test)]
mod test {
    use chrono::{Datelike, NaiveDate};

    use crate::utils::time::get_today_date;

    #[test]
    fn test_parse_date_from_str() {
        let today = get_today_date();
        assert_eq!(
            super::parse_date_from_str("2024-5-12"),
            Ok(NaiveDate::from_ymd_opt(2024, 5, 12).unwrap())
        );
        assert_eq!(
            super::parse_date_from_str("12-02"),
            Ok(today.with_day(2).unwrap().with_month(12).unwrap())
        );
    }
}
