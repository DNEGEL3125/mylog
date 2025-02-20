use std::borrow::Cow;

use crossterm::style::Stylize;

pub fn mark_search_result<'h>(
    search_pattern: &str,
    s: &'h str,
) -> Result<Cow<'h, str>, regex::Error> {
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
