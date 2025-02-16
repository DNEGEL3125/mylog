/// Compute the index in `lines` of the first character in `line` at `line_index`.
/// # Example
/// ```rust
/// let lines: Vec<String> = vec!["qwq".to_owned(), "This game".to_owned(), "Hello World".to_owned()];
/// let mut current_char_index = 0;
/// assert_eq!(get_char_index_by_line_index(lines, 0), 0);
/// assert_eq!(get_char_index_by_line_index(lines, 1), 3);
/// assert_eq!(get_char_index_by_line_index(lines, 2), 13);
/// ```
pub fn get_char_index_by_line_index(lines: &[String], line_index: usize) -> usize {
    let mut current_char_index: usize = 0;
    for line in lines.iter().take(line_index) {
        current_char_index += line.chars().filter(|c| !c.is_whitespace()).count();
    }

    current_char_index
}

/// Calculate the line index of the `char_index + 1`th character in `lines`.
pub fn get_line_index_by_char_index(lines: &[String], char_index: usize) -> Option<usize> {
    let mut current_char_index: usize = 0;
    for (line_index, line) in lines.iter().enumerate() {
        current_char_index += line.chars().filter(|c| !c.is_whitespace()).count();
        if current_char_index > char_index {
            return Some(line_index);
        }
    }
    None
}

#[cfg(test)]
mod test {
    use std::sync::LazyLock;

    use crate::log_pager::utils::{get_char_index_by_line_index, get_line_index_by_char_index};

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
}
