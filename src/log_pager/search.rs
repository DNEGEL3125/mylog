use std::borrow::Cow;

use crossterm::style::Stylize;

use super::pager::Pager;

pub trait Search {
    fn search_next(&mut self);
}
impl Search for dyn Pager {
    fn search_next(&mut self) {
        let target_str: String = "\0"
            .on_white()
            .to_string()
            .split_once('\0')
            .unwrap()
            .1
            .to_owned();
        let lines_to_skip = self.begin_line_index() + 1;
        for (line_index, line) in self.colored_lines().iter().enumerate().skip(lines_to_skip) {
            if line.contains(&target_str) {
                self.set_begin_line_index(line_index);
                break;
            }
        }
    }
}

pub fn mark_search_result<'h>(regex: &regex::Regex, s: &'h str) -> Cow<'h, str> {
    // Use regular expressions to replace matching parts
    let result = regex.replace_all(s, |caps: &regex::Captures| {
        // Get the matched text
        let matched_text = caps.get(0).map(|m| m.as_str()).unwrap_or("");
        // Highlight the matching text
        matched_text.black().on_white().to_string()
    });
    result
}
