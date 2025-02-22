use std::{
    borrow::Cow,
    cmp::min,
    io::{stdout, Stdout, Write},
    path::PathBuf,
    str::FromStr,
};

use chrono::NaiveDate;
use crossterm::{
    cursor, execute, queue,
    style::{Print, Stylize},
    terminal::{disable_raw_mode, enable_raw_mode, Clear},
};

use crate::{
    log_config::{construct_log_file_path, get_date_from_log_file_name},
    log_item::{LogItem, LogItemList},
    log_pager::search::mark_search_result,
};

use super::{
    events::{search_event::SearchEvent, view_event::ViewEvent},
    pager::Pager,
    pager_mode::PagerMode,
    range::Range,
    search::Search,
    utils::{get_char_index_by_line_index, get_line_index_by_char_index},
};

pub struct PagingAllPager {
    log_dir_path: PathBuf,
    mode: PagerMode,
    /// The index of the first character of the current page in the log file.
    /// White space characters are ignored when calculating the index.
    begin_char_index: usize,
    log_item_list: LogItemList,
    terminal_total_rows: u16,
    terminal_total_cols: u16,
    colored_lines: Vec<String>,
    is_exit: bool,
    search_pattern: Option<regex::Regex>,
    search_pattern_input: String,
}

impl PagingAllPager {
    pub fn new(log_dir_path: PathBuf) -> Self {
        use crate::utils::terminal::{get_terminal_total_cols, get_terminal_total_rows};
        let terminal_total_rows = get_terminal_total_rows();
        let terminal_total_cols = get_terminal_total_cols();
        let mut ret = Self {
            log_dir_path,
            mode: PagerMode::View,
            begin_char_index: 0,
            log_item_list: LogItemList::new(),
            terminal_total_rows,
            terminal_total_cols,
            colored_lines: Vec::new(),
            is_exit: false,
            search_pattern: None,
            search_pattern_input: String::new(),
        };

        ret.update_log_items();
        ret.resize(terminal_total_cols, terminal_total_rows);

        ret
    }

    fn resize(&mut self, columns: u16, rows: u16) {
        self.terminal_total_cols = columns;
        self.terminal_total_rows = rows;
        self.update_colored_lines();
    }

    fn goto_page_begin(&mut self) {
        self.set_begin_line_index(0);
    }

    fn goto_page_end(&mut self) {
        let original_page_range = self.page_range();
        let diff = self.total_content_lines() - original_page_range.end;
        self.set_begin_line_index(original_page_range.begin + diff);
    }

    fn highlight_log_item<'h>(&self, log_item: &'h LogItem) -> String {
        let date_str = format!("[{}]", log_item.date_time().format("%Y-%m-%d %H:%M"));
        let content: &str = log_item.content();
        let new_content: Cow<'h, str>;
        if let Some(regex) = &self.search_pattern {
            new_content = mark_search_result(regex, content);
        } else {
            new_content = Cow::Borrowed(content);
        }
        format!("{} {}", date_str.green(), new_content)
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

    fn all_date(&self) -> std::io::Result<Vec<NaiveDate>> {
        let mut ret = Vec::new();
        let log_dir_path = &self.log_dir_path;
        for entry in std::fs::read_dir(log_dir_path)? {
            let entry = entry?;
            let file_name = entry.file_name();
            if let Some(date) = get_date_from_log_file_name(file_name.to_str().unwrap()) {
                ret.push(date);
            }
        }

        Ok(ret)
    }

    fn confirm_search(&mut self) {
        let search_pattern_input = &mut self.search_pattern_input;
        self.search_pattern = match regex::Regex::new(search_pattern_input) {
            Ok(result) => Some(result),
            Err(_) => None,
        };
        search_pattern_input.clear();
        self.update_colored_lines();
        self.enter_view_mode();
        self.search_next(false);
    }

    fn content(&self) -> String {
        let mut ret = String::new();
        let mut all_date = self.all_date().unwrap();
        all_date.sort();
        for date in all_date {
            let file_path = construct_log_file_path(&self.log_dir_path, &date);
            let file_content: String = std::fs::read_to_string(&file_path).unwrap_or_default();
            ret += &file_content;
        }
        ret
    }

    fn enter_search_mode(&mut self) {
        self.mode = PagerMode::Search;
    }

    fn enter_view_mode(&mut self) {
        self.mode = PagerMode::View;
    }

    fn exit(&mut self) {
        self.is_exit = true;
    }

    fn handle_search_event(&mut self, event: SearchEvent) {
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

    fn handle_view_event(&mut self, event: ViewEvent) {
        match event {
            ViewEvent::NextLine => self.next_line(),
            ViewEvent::PrevLine => self.prev_line(),
            ViewEvent::Quit => self.exit(),
            ViewEvent::Resize(columns, rows) => self.resize(columns, rows),
            ViewEvent::GotoPageBegin => self.goto_page_begin(),
            ViewEvent::GotoPageEnd => self.goto_page_end(),
            ViewEvent::EnterSearchMode => self.enter_search_mode(),
            ViewEvent::SearchNext => self.search_next(true),
            ViewEvent::SearchPrev => self.search_prev(),
            _ => {}
        }

        self.print_pager().expect("Unable to print the pager");
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

    fn update_log_items(&mut self) {
        let content = self.content();

        self.log_item_list = LogItemList::from_str(&content).expect("Invalid log file");
        self.update_colored_lines();
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

    fn prepare_run(&self) {
        enable_raw_mode().expect("Failed to enable raw mode");
        execute!(stdout(), crossterm::terminal::EnterAlternateScreen)
            .expect("Unable to enter alternate screen");
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

    pub fn print_pager(&self) -> Result<(), std::io::Error> {
        let mut stdout = stdout();
        crossterm::queue!(
            stdout,
            Clear(crossterm::terminal::ClearType::All),
            cursor::MoveTo(0, 0),
            cursor::Hide
        )?;
        self.print_colored_file_content(&mut stdout)?;

        match self.mode {
            PagerMode::Search => {
                self.print_search_pattern_input(&mut stdout)?;
            }
            _ => {}
        }

        stdout.flush()?;
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

    pub fn run(&mut self) {
        self.prepare_run();
        self.print_pager().expect("Print pager");

        while !self.is_exit {
            let crossterm_event = crossterm::event::read().expect("Unable to read the event");
            match self.mode {
                PagerMode::View => {
                    let event = ViewEvent::from_crossterm_event(crossterm_event);
                    self.handle_view_event(event);
                }
                PagerMode::Command => {}
                PagerMode::Search => {
                    let event = SearchEvent::from_crossterm_event(crossterm_event);
                    self.handle_search_event(event);
                }
            }
        }

        crate::utils::terminal::restore_terminal().expect("Unable to restore the terminal");

        disable_raw_mode().expect("Unable to diable raw mode");
    }

    pub fn total_content_lines(&self) -> usize {
        self.colored_lines.len()
    }
}

impl Pager for PagingAllPager {
    fn begin_line_index(&self) -> usize {
        get_line_index_by_char_index(&self.colored_lines, self.begin_char_index).unwrap()
    }

    fn colored_lines(&self) -> &Vec<String> {
        &self.colored_lines
    }

    fn set_begin_line_index(&mut self, line_index: usize) {
        self.begin_char_index = get_char_index_by_line_index(&self.colored_lines, line_index);
    }
}
