use std::{path::PathBuf, process::exit};

use chrono::Datelike;
use chrono::NaiveDate;
use clap::Parser;
use log_config::{construct_log_file_path, LogConfig};
use log_item::LogItem;
use log_pager::LogPager;
use utils::time::{date_time_now, get_today_date};

pub mod cli;
pub mod constants;
pub mod log_config;
pub mod log_item;
pub mod log_pager;
pub mod user_event;
pub mod utils;

fn paging_log_file_by_date(log_dir_path: &PathBuf, date: NaiveDate, verbose: bool) {
    let mut log_pager = LogPager::new(date, log_dir_path.to_owned());
    log_pager.set_verbose(verbose);
    log_pager.run();
}

fn parse_date_from_str(date_str: &str) -> Result<NaiveDate, String> {
    let err_msg_template = format!("Invalid date '{}'.", date_str);
    let date_str_parts: Vec<&str> = date_str.split('-').collect();
    let parts_count = date_str_parts.len();
    let date = match parts_count {
        2 => {
            let today = get_today_date();
            let month: u32 = date_str_parts[0].parse().map_err(|_| &err_msg_template)?;
            let day: u32 = date_str_parts[1].parse().map_err(|_| &err_msg_template)?;
            today
                .with_month(month)
                .ok_or(&err_msg_template)?
                .with_day(day)
                .ok_or(&err_msg_template)?
        }
        3 => {
            let date_fmt = "%Y-%m-%d";
            NaiveDate::parse_from_str(date_str, date_fmt).map_err(|_| &err_msg_template)?
        }
        _ => return Err(err_msg_template),
    };
    Ok(date)
}

fn view_logs(date_str: Option<String>, verbose: bool, log_dir_path: &PathBuf) {
    let today_date = get_today_date();

    let date_result = match date_str {
        Some(date_str) => parse_date_from_str(&date_str),
        // Default date is today
        None => Ok(today_date),
    };

    let date = match date_result {
        Ok(date) => date,
        Err(msg) => {
            println!("{}", msg);
            exit(-966);
        }
    };

    paging_log_file_by_date(log_dir_path, date, verbose);
}

fn write_log(log_content: &str, verbose: bool, log_dir_path: &PathBuf) {
    let date_time_now = date_time_now();
    let today_date = date_time_now.date();

    let log_file_path = construct_log_file_path(log_dir_path, today_date);

    // If the log file does not exist, create it
    let _ = std::fs::File::create_new(&log_file_path);

    let log_item = LogItem::new(date_time_now, log_content);
    if verbose {
        println!("Log info: {:#?}\nWriting the log message...", log_item);
    }

    LogItem::append_to_file(&log_item, &log_file_path, verbose);
}

fn edit_logs(date_str: Option<String>, verbose: bool, log_dir_path: &PathBuf) {
    let today_date = get_today_date();
    let date_result = match date_str {
        Some(date_str) => parse_date_from_str(&date_str),
        // Default date is today
        None => Ok(today_date),
    };

    let date = match date_result {
        Ok(date) => date,
        Err(msg) => {
            println!("{}", msg);
            exit(-966);
        }
    };

    let log_file_path = construct_log_file_path(log_dir_path, date);

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

    let res = edit::edit_file(log_file_path);
    if res.is_err() {
        println!("Can't edit the file");
    }
}

fn main() {
    LogConfig::create_config_file_if_not_exists();
    let config_file_path = &crate::constants::CONFIG_FILE_PATH;
    let log_config = log_config::LogConfig::from_config_file(config_file_path.as_path());
    let log_dir_path = log_config.log_dir_path;

    if !log_dir_path.exists() {
        println!(
            "The log directory '{}' doesn't exist.\nYou may want to configure it in '{}'",
            log_dir_path.display(),
            config_file_path.display()
        );
        exit(1);
    }

    // Command line parameters
    let cli = cli::Cli::parse();
    match cli.command {
        cli::Commands::View { date, verbose } => {
            view_logs(date, verbose, &log_dir_path);
        }
        cli::Commands::Write { message, verbose } => {
            let message_string = if let Some(message_string) = message {
                message_string
            } else {
                input_log_message()
            };

            if message_string.trim().is_empty() {
                println!("Aborting due to empty log message.");
                exit(-9320);
            }
            write_log(&message_string, verbose, &log_dir_path);
        }
        cli::Commands::Config { .. } => todo!(),
        cli::Commands::Edit { date, verbose } => {
            edit_logs(date, verbose, &log_dir_path);
        }
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
