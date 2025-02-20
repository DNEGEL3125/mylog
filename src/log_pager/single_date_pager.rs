use std::borrow::Cow;
use std::cmp::min;
use std::io::{stdout, Stdout, Write};
use std::path::PathBuf;
use std::str::FromStr;

use super::command;
use super::events::command_event::CommandEvent;
use super::events::search_event::SearchEvent;
use super::events::view_event::ViewEvent;
use super::pager_mode::PagerMode;
use super::range::Range;
use super::utils::{get_char_index_by_line_index, get_line_index_by_char_index};
use chrono::{Datelike, Days, NaiveDate};
use crossterm::style::{ContentStyle, Print, PrintStyledContent, StyledContent, Stylize};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear};
use crossterm::{cursor, execute, queue};

use crate::log_config::construct_log_file_path;
use crate::log_item::{LogItem, LogItemList};
use crate::utils::time::get_today_date;

pub struct SingleDatePager {
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
    mode: PagerMode,
    is_exit: bool,
    command_buffer: String,
    search_pattern: String,
    search_pattern_input: String,
}

impl SingleDatePager {
    pub fn new(date: NaiveDate, log_dir_path: PathBuf) -> Self {
        use crate::utils::terminal::{get_terminal_total_cols, get_terminal_total_rows};
        let terminal_total_rows = get_terminal_total_rows();
        let terminal_total_cols = get_terminal_total_cols();
        let message = StyledContent::new(ContentStyle::new(), String::new());
        let mut ret = SingleDatePager {
            date,
            log_dir_path,
            verbose: false,
            begin_char_index: 0,
            bottom_message: message,
            log_item_list: LogItemList::new(),
            terminal_total_rows,
            terminal_total_cols,
            colored_lines: Vec::new(),
            mode: PagerMode::View,
            is_exit: false,
            command_buffer: String::new(),
            search_pattern: String::new(),
            search_pattern_input: String::new(),
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

    fn begin_line_index(&self) -> usize {
        get_line_index_by_char_index(&self.colored_lines, self.begin_char_index).unwrap()
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
        self.update_colored_lines();
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

    fn print_search_pattern_input(&self, stdout: &mut Stdout) -> Result<(), std::io::Error> {
        let terminal_total_rows = self.terminal_total_rows;
        crossterm::queue!(
            stdout,
            cursor::MoveTo(0, terminal_total_rows - 1),
            Print('/'),
            Print(&self.search_pattern_input)
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
            PagerMode::Command => {
                self.print_command(&mut stdout)?;
            }
            PagerMode::Search => {
                self.print_search_pattern_input(&mut stdout)?;
            }
            _ => {}
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

    fn mark_search_result<'h>(&self, s: &'h str) -> Result<Cow<'h, str>, regex::Error> {
        let search_pattern = &self.search_pattern;
        let regex = regex::Regex::new(search_pattern)?;
        // Use regular expressions to replace matching parts
        let result = regex.replace_all(s, |caps: &regex::Captures| {
            // Get the matched text
            let matched_text = caps.get(0).map(|m| m.as_str()).unwrap_or("");
            // Highlight the matching text
            matched_text.black().on_white().to_string()
        });
        Ok(result)
    }

    fn highlight_log_item(&self, log_item: &LogItem) -> String {
        let date_str = format!("[{}]", log_item.date_time().format("%Y-%m-%d %H:%M"));
        let content = log_item.content();
        let content = self
            .mark_search_result(content)
            .unwrap_or(Cow::Borrowed(content));
        format!("{} {}", date_str.green(), content)
    }

    /// Splits the log content into lines that fit within the terminal width,
    /// while preserving any color formatting.
    ///
    /// - For each log item, it converts the log content into a colored string.
    /// - Each line is split into smaller lines if it exceeds the terminal's width.
    fn update_colored_lines(&mut self) {
        // Get the terminal's total column width.
        let terminal_total_cols = self.terminal_total_cols as usize;

        self.colored_lines.clear();
        for item in self.log_item_list.iter() {
            for line in self.highlight_log_item(item).lines() {
                self.colored_lines.extend(
                    textwrap::wrap(line, terminal_total_cols)
                        .iter()
                        .map(|x| x.to_string()),
                );
            }
        }
    }

    fn resize(&mut self, columns: u16, rows: u16) {
        self.terminal_total_cols = columns;
        self.terminal_total_rows = rows;
        self.update_colored_lines();
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
        self.mode = PagerMode::Command;
    }

    fn enter_search_mode(&mut self) {
        self.mode = PagerMode::Search;
    }

    fn exit(&mut self) {
        self.is_exit = true;
    }

    fn search_next(&mut self) {
        let target_str: String = "\0"
            .on_white()
            .to_string()
            .split_once('\0')
            .unwrap()
            .1
            .to_owned();
        let lines_to_skip = self.begin_line_index() + 1;
        for (line_index, line) in self.colored_lines.iter().enumerate().skip(lines_to_skip) {
            if line.contains(&target_str) {
                self.set_begin_line_index(line_index);
                break;
            }
        }
    }

    fn search_prev(&mut self) {
        let target_str: String = "\0"
            .on_white()
            .to_string()
            .split_once('\0')
            .unwrap()
            .1
            .to_owned();
        let lines_to_take: usize = self.begin_line_index();
        for (line_index, line) in self
            .colored_lines
            .iter()
            .enumerate()
            .take(lines_to_take)
            .rev()
        {
            if line.contains(&target_str) {
                self.set_begin_line_index(line_index);
                break;
            }
        }
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
            ViewEvent::SearchNext => self.search_next(),
            ViewEvent::SearchPrev => self.search_prev(),
            ViewEvent::Resize(columns, rows) => self.resize(columns, rows),
            ViewEvent::EnterCommandMode => self.enter_command_mode(),
            ViewEvent::EnterSearchMode => self.enter_search_mode(),
            ViewEvent::None => {}
        }

        self.print_pager().expect("Unable to print the pager");
    }

    fn enter_view_mode(&mut self) {
        self.command_buffer.clear();
        self.mode = PagerMode::View;
    }

    fn execute_command(&mut self) {
        let command_str = &self.command_buffer;
        let command = command::Command::from_str(command_str).unwrap();
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

    fn confirm_search(&mut self) {
        self.search_pattern = self.search_pattern_input.clone();
        self.search_pattern_input.clear();
        self.update_colored_lines();
        self.enter_view_mode();
        self.search_next();
    }

    fn handle_search_event(&mut self, event: SearchEvent) {
        self.clear_error_message();
        match event {
            SearchEvent::Confirm => self.confirm_search(),
            SearchEvent::Char(c) => self.search_pattern_input.push(c),
            SearchEvent::None => {}
            SearchEvent::Cancel => self.enter_view_mode(),
            SearchEvent::Backspace => {
                if self.search_pattern_input.is_empty() {
                    self.enter_view_mode();
                } else {
                    self.search_pattern_input.pop().unwrap();
                }
            }
            SearchEvent::ClearLine => self.search_pattern_input.clear(),
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
                PagerMode::View => {
                    let event = ViewEvent::from_crossterm_event(crossterm_event);
                    self.handle_view_event(event);
                }
                PagerMode::Command => {
                    let event = CommandEvent::from_crossterm_event(crossterm_event);
                    self.handle_command_event(event);
                }
                PagerMode::Search => {
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
    use std::path::PathBuf;

    use chrono::NaiveDate;

    use super::SingleDatePager;

    #[test]
    fn test_begin_line_index() {
        let mut pager = SingleDatePager::new(NaiveDate::default(), PathBuf::default());
        pager.colored_lines = ["qwq", "abc", "eee", "661", "sld", "934", "f8s"]
            .iter()
            .map(|x| x.to_string())
            .collect();
        pager.set_begin_line_index(5);
        assert_eq!(pager.begin_line_index(), 5);
        pager.set_begin_line_index(2);
        assert_eq!(pager.begin_line_index(), 2);
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
