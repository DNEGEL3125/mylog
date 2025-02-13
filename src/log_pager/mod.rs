use std::cmp::min;
use std::io::{stdout, Stdout, Write};
use std::path::PathBuf;
use std::str::FromStr;

use chrono::{Datelike, Days, NaiveDate};
use crossterm::style::{ContentStyle, Print, PrintStyledContent, StyledContent, Stylize};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear};
use crossterm::{cursor, execute, queue};
use events::command_event::CommandEvent;
use events::search_event::SearchEvent;
use events::view_event::ViewEvent;

use crate::log_config::construct_log_file_path;
use crate::log_item::LogItemList;
use crate::utils::time::get_today_date;

pub mod command;
pub mod events;

/// Compute the index in `lines` of the first character in `line` at `line_index`.
/// # Example
/// ```rust
/// let lines: Vec<String> = vec!["qwq".to_owned(), "This game".to_owned(), "Hello World".to_owned()];
/// let mut current_char_index = 0;
/// assert_eq!(get_char_index_by_line_index(lines, 0), 0);
/// assert_eq!(get_char_index_by_line_index(lines, 1), 3);
/// assert_eq!(get_char_index_by_line_index(lines, 2), 13);
/// ```
fn get_char_index_by_line_index(lines: &[String], line_index: usize) -> usize {
    let mut current_char_index: usize = 0;
    for line in lines.iter().take(line_index) {
        current_char_index += line.chars().filter(|c| !c.is_whitespace()).count();
    }

    current_char_index
}

/// Calculate the line index of the `char_index + 1`th character in `lines`.
fn get_line_index_by_char_index(lines: &[String], char_index: usize) -> Option<usize> {
    let mut current_char_index: usize = 0;
    for (line_index, line) in lines.iter().enumerate() {
        current_char_index += line.chars().filter(|c| !c.is_whitespace()).count();
        if current_char_index > char_index {
            return Some(line_index);
        }
    }
    None
}

struct Range {
    begin: usize,
    end: usize,
}

impl Range {
    fn new(begin: usize, end: usize) -> Self {
        assert!(begin <= end, "begin > end");
        Self { begin, end }
    }
}

#[derive(PartialEq)]
enum LogPagerMode {
    View,
    Command,
    Search,
}

pub struct LogPager {
    date: NaiveDate,
    log_dir_path: PathBuf,
    verbose: bool,
    /// The index of the first character of the current page in the log file.
    /// White space characters are ignored when calculating the index.
    begin_char_index: usize,
    bottom_message: StyledContent<String>,
    log_item_list: LogItemList,
    terminal_total_rows: u16,
    terminal_total_cols: u16,
    colored_lines: Vec<String>,
    mode: LogPagerMode,
    is_exit: bool,
    command_buffer: String,
    search_string: String,
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
            begin_char_index: 0,
            bottom_message: message,
            log_item_list: LogItemList::new(),
            terminal_total_rows,
            terminal_total_cols,
            colored_lines: Vec::new(),
            mode: LogPagerMode::View,
            is_exit: false,
            command_buffer: String::new(),
            search_string: String::new(),
        };

        ret.update_log_items();
        ret.resize(terminal_total_cols, terminal_total_rows);

        ret
    }

    pub fn set_verbose(&mut self, value: bool) {
        self.verbose = value;
    }

    pub fn total_content_lines(&self) -> usize {
        self.colored_lines.len()
    }

    fn set_begin_line_index(&mut self, line_index: usize) {
        self.begin_char_index = get_char_index_by_line_index(&self.colored_lines, line_index);
    }

    fn page_range(&self) -> Range {
        let terminal_total_rows = self.terminal_total_rows;
        let page_range_begin =
            get_line_index_by_char_index(&self.colored_lines, self.begin_char_index).unwrap_or(0);
        let page_range_end = if terminal_total_rows <= 2 {
            page_range_begin + 1
        } else {
            min(
                self.total_content_lines(),
                page_range_begin + terminal_total_rows as usize - 2,
            )
        };
        Range::new(page_range_begin, page_range_end)
    }

    pub fn next_day(&mut self) {
        if self.date == get_today_date() {
            let err_msg = "This is already today's log";
            self.show_error_message(err_msg);
            return;
        }
        self.date = self
            .date
            .checked_add_days(Days::new(1))
            .expect("Date out of range");

        self.update_log_items();
        self.begin_char_index = 0;
    }

    pub fn prev_day(&mut self) {
        self.date = self
            .date
            .checked_sub_days(Days::new(1))
            .expect("Date out of range");

        self.update_log_items();
        self.begin_char_index = 0;
    }

    pub fn next_line(&mut self) {
        let page_range = self.page_range();
        if page_range.end >= self.total_content_lines() {
            return;
        }

        self.set_begin_line_index(page_range.begin + 1);
    }

    pub fn prev_line(&mut self) {
        let page_range_begin = self.page_range().begin;
        if page_range_begin == 0 {
            return;
        }
        self.set_begin_line_index(page_range_begin - 1);
    }

    fn goto_page_begin(&mut self) {
        self.set_begin_line_index(0);
    }

    fn goto_page_end(&mut self) {
        let original_page_range = self.page_range();
        let diff = self.total_content_lines() - original_page_range.end;
        self.set_begin_line_index(original_page_range.begin + diff);
    }

    fn update_log_items(&mut self) {
        let file_path = construct_log_file_path(&self.log_dir_path, &self.date);

        let file_content = std::fs::read_to_string(&file_path).unwrap_or_else(|_err| {
            if self.verbose {
                self.show_error_message(&format!("'{}' doesn't exist", file_path.display()));
            }
            String::new()
        });

        self.log_item_list = LogItemList::from_str(&file_content).expect("Invalid log file");
        self.colored_lines = self.split_colored_log_content_to_lines();
        // let _ = self
        //     .pager
        //     .set_prompt(format!("{} {}", self.date, self.date.weekday()));
    }

    fn print_colored_file_content(&self, stdout: &mut Stdout) -> Result<(), std::io::Error> {
        let terminal_total_rows = self.terminal_total_rows;
        if terminal_total_rows == 0 {
            return Ok(());
        }

        let range = self.page_range();

        let colored_lines = &self.colored_lines;

        for i in range.begin..range.end {
            if i != range.begin {
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

    fn print_command(&self, stdout: &mut Stdout) -> Result<(), std::io::Error> {
        let terminal_total_rows = self.terminal_total_rows;
        crossterm::queue!(
            stdout,
            cursor::MoveTo(0, terminal_total_rows - 1),
            Print(':'),
            Print(&self.command_buffer)
        )?;

        Ok(())
    }

    fn print_search_string(&self, stdout: &mut Stdout) -> Result<(), std::io::Error> {
        let terminal_total_rows = self.terminal_total_rows;
        crossterm::queue!(
            stdout,
            cursor::MoveTo(0, terminal_total_rows - 1),
            Print('/'),
            Print(&self.search_string)
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
        match self.mode {
            LogPagerMode::Command => {
                self.print_command(&mut stdout)?;
            }
            LogPagerMode::Search => {
                self.print_search_string(&mut stdout)?;
            }
            _ => {}
        }
        if self.mode == LogPagerMode::Command {
            self.print_command(&mut stdout)?;
        }

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
        self.terminal_total_cols = columns;
        self.terminal_total_rows = rows;
        self.colored_lines = self.split_colored_log_content_to_lines();
    }

    fn edit(&mut self) -> Result<(), std::io::Error> {
        let log_dir_path = &self.log_dir_path;
        let date = &self.date;
        let file_path = construct_log_file_path(log_dir_path, date);
        crate::utils::terminal::restore_terminal().expect("Unable to restore the terminal");
        edit::edit_file(file_path)?;
        self.update_log_items();
        execute!(stdout(), crossterm::terminal::EnterAlternateScreen)
            .expect("Unable to enter alternate screen");
        Ok(())
    }

    fn enter_command_mode(&mut self) {
        self.mode = LogPagerMode::Command;
    }

    fn enter_search_mode(&mut self) {
        self.mode = LogPagerMode::Search;
    }

    fn exit(&mut self) {
        self.is_exit = true;
    }

    fn handle_view_event(&mut self, event: ViewEvent) {
        self.clear_error_message();
        match event {
            ViewEvent::NextDay => self.next_day(),
            ViewEvent::PrevDay => self.prev_day(),
            ViewEvent::NextLine => self.next_line(),
            ViewEvent::PrevLine => self.prev_line(),
            ViewEvent::GotoPageBegin => self.goto_page_begin(),
            ViewEvent::GotoPageEnd => self.goto_page_end(),
            ViewEvent::Quit => self.exit(),
            ViewEvent::Edit => self.edit().expect("Unable to edit the file"),
            ViewEvent::Resize(columns, rows) => self.resize(columns, rows),
            ViewEvent::EnterCommandMode => self.enter_command_mode(),
            ViewEvent::EnterSearchMode => self.enter_search_mode(),
            ViewEvent::None => {}
        }

        self.print_pager().expect("Unable to print the pager");
    }

    fn enter_view_mode(&mut self) {
        self.command_buffer.clear();
        self.search_string.clear();
        self.mode = LogPagerMode::View;
    }

    fn execute_command(&mut self) {
        let command_str = &self.command_buffer;
        let command = self::command::Command::from_str(command_str);
        match command {
            command::Command::None => {}
            command::Command::ShowDate => todo!(),
            command::Command::SetDate(date_str) => {
                if let Ok(date) = NaiveDate::from_str(&date_str) {
                    self.date = date;
                    self.update_log_items();
                }
            }
        }

        self.command_buffer.clear();
        self.enter_view_mode();
    }

    fn handle_command_event(&mut self, event: CommandEvent) {
        self.clear_error_message();
        match event {
            CommandEvent::Execute => self.execute_command(),
            CommandEvent::Char(c) => self.command_buffer.push(c),
            CommandEvent::None => {}
            CommandEvent::Cancel => self.enter_view_mode(),
            CommandEvent::Backspace => {
                if self.command_buffer.is_empty() {
                    self.enter_view_mode();
                } else {
                    self.command_buffer.pop().unwrap();
                }
            }
            CommandEvent::ClearLine => self.command_buffer.clear(),
        }
        self.print_pager().expect("Unable to print the pager");
    }

    fn handle_search_event(&mut self, event: SearchEvent) {
        self.clear_error_message();
        match event {
            SearchEvent::Confirm => self.execute_command(),
            SearchEvent::Char(c) => self.search_string.push(c),
            SearchEvent::None => {}
            SearchEvent::Cancel => self.enter_view_mode(),
            SearchEvent::Backspace => {
                if self.search_string.is_empty() {
                    self.enter_view_mode();
                } else {
                    self.search_string.pop().unwrap();
                }
            }
            SearchEvent::ClearLine => self.search_string.clear(),
        }
        self.print_pager().expect("Unable to print the pager");
    }

    pub fn run(&mut self) {
        enable_raw_mode().expect("Failed to enable raw mode");
        execute!(stdout(), crossterm::terminal::EnterAlternateScreen)
            .expect("Unable to enter alternate screen");
        self.print_pager().expect("Print pager");

        while !self.is_exit {
            let crossterm_event = crossterm::event::read().expect("Unable to read the event");
            match self.mode {
                LogPagerMode::View => {
                    let event = ViewEvent::from_crossterm_event(crossterm_event);
                    self.handle_view_event(event);
                }
                LogPagerMode::Command => {
                    let event = CommandEvent::from_crossterm_event(crossterm_event);
                    self.handle_command_event(event);
                }
                LogPagerMode::Search => {
                    let event = SearchEvent::from_crossterm_event(crossterm_event);
                    self.handle_search_event(event);
                }
            }
        }

        crate::utils::terminal::restore_terminal().expect("Unable to restore the terminal");

        disable_raw_mode().expect("Unable to diable raw mode");
    }
}

#[cfg(test)]
mod test {
    use std::sync::LazyLock;

    use crate::log_pager::get_char_index_by_line_index;

    use super::get_line_index_by_char_index;

    static TEST_LINES: LazyLock<Vec<String>> = LazyLock::new(|| {
        [
            "The darkest valley",
            "The highest mountain",
            "We walk in the name of our brave",
            "The rushing river",
            "The blooming flower",
            "Descended from heaven we embrace",
        ]
        .map(|x| x.to_string())
        .into()
    });

    #[test]
    fn test_get_line_index_by_char_index() {
        let lines: &Vec<String> = &TEST_LINES;
        let mut current_char_index = 0;
        for (expected_line_index, line) in lines.iter().enumerate() {
            for _ in line.chars().filter(|x| !x.is_whitespace()) {
                assert_eq!(
                    get_line_index_by_char_index(lines, current_char_index),
                    Some(expected_line_index)
                );
                current_char_index += 1;
            }
        }
    }

    #[test]
    fn test_get_char_index_by_line_index() {
        let lines: &Vec<String> = &TEST_LINES;
        for (line_index, _) in lines.iter().enumerate() {
            assert_eq!(
                get_line_index_by_char_index(
                    lines,
                    get_char_index_by_line_index(lines, line_index)
                ),
                Some(line_index)
            );
        }
    }

    // mod resize {
    //     struct TestConfig {
    //         log_dir: PathBuf,
    //         log_file_path: PathBuf,
    //         date: NaiveDate,
    //     }
    //     impl TestConfig {
    //         fn new() -> TestConfig {
    //             let log_dir = std::env::temp_dir().join("mylog");
    //             let date = NaiveDate::default();
    //             let log_file_path = log_dir.join(date.to_string());

    //             Self {
    //                 log_dir,
    //                 log_file_path,
    //                 date,
    //             }
    //         }

    //         fn _init(&self) -> Result<(), Box<dyn std::error::Error>> {
    //             std::fs::create_dir(&self.log_dir)?;
    //             std::fs::File::create(&self.log_file_path)?;
    //             Ok(())
    //         }
    //     }

    //     impl Drop for TestConfig {
    //         fn drop(&mut self) {
    //             if self.log_file_path.exists() {
    //                 std::fs::remove_file(&self.log_file_path).expect(&format!(
    //                     "Unable to remove file '{}'",
    //                     self.log_file_path.display()
    //                 ));
    //             }

    //             if self.log_dir.exists() {
    //                 std::fs::remove_dir(&self.log_dir).expect(&format!(
    //                     "Unable to remove directory '{}'",
    //                     self.log_dir.display()
    //                 ));
    //             }
    //         }
    //     }

    //     use crate::{
    //         log_item::{LogItem, LogItemList},
    //         log_pager::{get_char_index_by_line_index, get_line_by_char_index},
    //     };

    //     use super::super::LogPager;
    //     use chrono::{NaiveDate, NaiveDateTime};
    //     use rand::{seq::IndexedRandom, Rng};
    //     use std::path::PathBuf;
    //     #[test]
    //     fn test_resize() {
    //         let test_config = TestConfig::new();
    //         // if let Err(err) = test_config.init() {
    //         //     panic!("{}", err.to_string());
    //         // }

    //         let char_set: Vec<char> = [
    //             "我在哪（）合抱之木生于毫末 ()\"'\n闻道有先后如数家珍杠杆原理"
    //                 .chars()
    //                 .collect::<Vec<char>>(),
    //             ('A'..'Z').collect(),
    //             ('a'..'z').collect(),
    //             ('0'..'9').collect(),
    //         ]
    //         .concat();
    //         let mut log_item_list = LogItemList::new();
    //         let max_content_len: usize = 400;
    //         for _ in 1..200 {
    //             let content_len: usize = rand::rng().random_range(0..max_content_len) + 1;
    //             let mut content = String::new();
    //             for _ in 0..content_len {
    //                 let rand_char = char_set.choose(&mut rand::rng()).unwrap();
    //                 content.push(*rand_char);
    //             }
    //             log_item_list.push(LogItem::new(NaiveDateTime::default(), &content));
    //         }
    //         let mut log_pager = LogPager::new(test_config.date, test_config.log_dir.to_owned());
    //         log_pager.resize(8, 8);
    //         log_pager.log_item_list = log_item_list;
    //         let lines = log_pager.split_colored_log_content_to_lines();
    //         for (line_index, line) in lines.iter().enumerate() {
    //             let first_char = line.chars().next().unwrap();
    //             let char_index = get_char_index_by_line_index(&lines, line_index);
    //             for (columns, rows) in [(13, 14), (8, 8)] {
    //                 log_pager.resize(columns, rows);
    //                 let lines_after_resizing = log_pager.split_colored_log_content_to_lines();
    //                 let line_after_resizing =
    //                     get_line_by_char_index(&lines_after_resizing, char_index).unwrap();
    //                 assert!(
    //                     line_after_resizing.contains(first_char),
    //                     r#""{}" -> "{}", this resized line doesn't contain the first char '{}';
    //                     char_index = {}, line_index = {}, terminal_size = {:?}, line_char_count = {}"#,
    //                     line,
    //                     line_after_resizing,
    //                     first_char,
    //                     char_index,
    //                     line_index,
    //                     (columns, rows),
    //                     line.chars().count()
    //                 );
    //             }
    //         }
    //     }
    // }
}
