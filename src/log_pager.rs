use std::cmp::min;
use std::io::{stdout, Stdout, Write};
use std::path::PathBuf;
use std::str::FromStr;

use chrono::{Datelike, Days, NaiveDate};
use crossterm::style::{ContentStyle, Print, PrintStyledContent, StyledContent, Stylize};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear};
use crossterm::{cursor, execute, queue};

use crate::log_config::construct_log_file_path;
use crate::log_item::LogItemList;
use crate::user_event::{get_user_event, UserEvent};
use crate::utils::fs::get_file_content_by_path;
use crate::utils::time::get_today_date;

pub struct LogPager {
    date: NaiveDate,
    log_dir_path: PathBuf,
    verbose: bool,
    /// The row index at which the current page starts.
    page_log_line_range_begin: usize,
    bottom_message: StyledContent<String>,
    log_item_list: LogItemList,
    terminal_total_rows: u16,
    terminal_total_cols: u16,
}

impl LogPager {
    pub fn new(date: NaiveDate, log_dir_path: PathBuf) -> Self {
        use crate::utils::terminal::{get_terminal_total_cols, get_terminal_total_rows};
        let terminal_total_rows = get_terminal_total_rows();
        let terminal_total_cols = get_terminal_total_cols();
        let message = StyledContent::new(ContentStyle::new(), String::new());
        let mut ret = LogPager {
            date,
            log_dir_path,
            verbose: false,
            page_log_line_range_begin: 0,
            bottom_message: message,
            log_item_list: LogItemList::new(),
            terminal_total_rows,
            terminal_total_cols,
        };

        ret.update_pager();

        ret
    }

    pub fn set_verbose(&mut self, value: bool) {
        self.verbose = value;
    }

    pub fn total_content_lines(&self) -> usize {
        self.split_colored_log_content_to_lines().len()
    }

    pub fn page_log_line_range_end(&self) -> usize {
        let terminal_total_rows = self.terminal_total_rows;
        if terminal_total_rows <= 2 {
            self.page_log_line_range_begin + 1
        } else {
            min(
                self.total_content_lines(),
                self.page_log_line_range_begin + terminal_total_rows as usize - 2,
            )
        }
    }

    pub fn next_day(&mut self) {
        if self.date == get_today_date() {
            let err_msg = "This is already today's log";
            self.show_error_message(&err_msg);
            return;
        }
        self.date = self
            .date
            .checked_add_days(Days::new(1))
            .expect("Date out of range");

        self.update_pager();
        self.page_log_line_range_begin = 0;
    }

    pub fn prev_day(&mut self) {
        self.date = self
            .date
            .checked_sub_days(Days::new(1))
            .expect("Date out of range");

        self.update_pager();
        self.page_log_line_range_begin = 0;
    }

    pub fn next_line(&mut self) {
        if self.page_log_line_range_end() >= self.total_content_lines() {
            return;
        }
        self.page_log_line_range_begin += 1;
    }

    pub fn prev_line(&mut self) {
        if self.page_log_line_range_begin == 0 {
            return;
        }
        self.page_log_line_range_begin -= 1;
    }

    fn update_pager(&mut self) {
        let file_path = construct_log_file_path(&self.log_dir_path, self.date);

        let file_content = if file_path.exists() {
            get_file_content_by_path(&file_path)
        } else {
            if self.verbose {
                self.show_error_message(&format!("'{}' doesn't exist", file_path.display()));
            }
            String::new()
        };

        self.log_item_list = LogItemList::from_str(&file_content).expect("Invalid log file");
        // let _ = self
        //     .pager
        //     .set_prompt(format!("{} {}", self.date, self.date.weekday()));
    }

    fn print_colored_file_content(&self, stdout: &mut Stdout) -> Result<(), std::io::Error> {
        let terminal_total_rows = self.terminal_total_rows;
        if terminal_total_rows == 0 {
            return Ok(());
        }
        let begin_index: usize = self.page_log_line_range_begin;
        let end_index: usize = self.page_log_line_range_end();

        let colored_lines = self.split_colored_log_content_to_lines();

        for i in begin_index..end_index {
            if i != begin_index {
                queue!(stdout, cursor::MoveToNextLine(1))?;
            }
            queue!(stdout, Print(&colored_lines[i]))?;
        }

        Ok(())
    }

    fn print_colored_date(&self, stdout: &mut Stdout) -> Result<(), std::io::Error> {
        let terminal_total_rows = self.terminal_total_rows;
        if terminal_total_rows <= 1 {
            return Ok(());
        }
        let content_style = ContentStyle::new().dark_grey();
        let styled_content = StyledContent::new(
            content_style,
            format!("{} {}", self.date, self.date.weekday()),
        );
        let row_index = if terminal_total_rows == 2 {
            1
        } else {
            terminal_total_rows - 2
        };
        crossterm::queue!(
            stdout,
            cursor::MoveTo(0, row_index),
            PrintStyledContent(styled_content)
        )?;

        Ok(())
    }

    fn print_colored_message(&self, stdout: &mut Stdout) -> Result<(), std::io::Error> {
        let terminal_total_rows = self.terminal_total_rows;
        crossterm::queue!(
            stdout,
            cursor::MoveTo(0, terminal_total_rows - 1),
            PrintStyledContent(self.bottom_message.clone())
        )?;

        Ok(())
    }

    pub fn print_pager(&self) -> Result<(), std::io::Error> {
        let mut stdout = stdout();
        crossterm::queue!(
            stdout,
            Clear(crossterm::terminal::ClearType::All),
            cursor::MoveTo(0, 0),
            cursor::Hide
        )?;
        self.print_colored_file_content(&mut stdout)?;
        self.print_colored_date(&mut stdout)?;
        self.print_colored_message(&mut stdout)?;

        stdout.flush()?;
        Ok(())
    }

    fn show_error_message(&mut self, msg: &str) {
        let content_style = ContentStyle::new().white().on_red();
        self.bottom_message = StyledContent::new(content_style, msg.to_owned());
    }

    pub fn clear_error_message(&mut self) {
        self.bottom_message = StyledContent::new(ContentStyle::new(), String::new());
    }

    /// Splits the log content into lines that fit within the terminal width,
    /// while preserving any color formatting.
    ///
    /// - For each log item, it converts the log content into a colored string.
    /// - Each line is split into smaller lines if it exceeds the terminal's width.
    ///
    /// # Returns
    /// A vector of strings where each string is a single terminal-sized line.
    fn split_colored_log_content_to_lines(&self) -> Vec<String> {
        let mut ret: Vec<String> = Vec::new();
        // Get the terminal's total column width.
        let terminal_total_cols = self.terminal_total_cols as usize;

        for item in self.log_item_list.iter() {
            for line in item.to_colored_string().lines() {
                ret.extend(
                    textwrap::wrap(line, terminal_total_cols)
                        .iter()
                        .map(|x| x.to_string()),
                );
            }
        }
        ret
    }

    fn resize(&mut self, columns: u16, rows: u16) {
        let original_columns = self.terminal_total_cols as usize;
        self.page_log_line_range_begin =
            original_columns * self.page_log_line_range_begin / columns as usize;

        self.terminal_total_cols = columns;
        self.terminal_total_rows = rows;
        self.update_pager();
    }

    pub fn run(&mut self) {
        enable_raw_mode().expect("Failed to enable raw mode");
        execute!(stdout(), crossterm::terminal::EnterAlternateScreen)
            .expect("Unable to enter alternate screen");
        self.print_pager().expect("Print pager");

        let mut is_exit = false;
        while !is_exit {
            let user_event = get_user_event();

            self.clear_error_message();
            match user_event {
                UserEvent::NextDay => self.next_day(),
                UserEvent::PrevDay => self.prev_day(),
                UserEvent::NextLine => self.next_line(),
                UserEvent::PrevLine => self.prev_line(),
                UserEvent::Quit => is_exit = true,
                UserEvent::Search => todo!(),
                UserEvent::Resize(columns, rows) => self.resize(columns, rows),
                UserEvent::None => continue,
            }

            is_exit = is_exit || self.print_pager().is_err();
        }

        crate::utils::terminal::restore_terminal().expect("Unable to restore the terminal");

        disable_raw_mode().expect("Unable to diable raw mode");
    }
}
