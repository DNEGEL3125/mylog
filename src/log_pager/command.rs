#[derive(Debug, PartialEq)]
pub enum Command {
    ShowDate,
    SetDate(String),
    None,
}

impl Command {
    pub fn from_str(s: &str) -> Self {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.is_empty() {
            return Command::None;
        }
        if parts[0] == "date" {
            return match parts.len() {
                1 => Command::ShowDate,
                2 => Command::SetDate(parts[1].to_owned()),
                _ => Command::None,
            };
        }
        Command::None
    }
}

#[cfg(test)]
mod test {
    use super::Command;
    #[test]
    fn test_command_from_str() {
        assert_eq!(
            Command::from_str("date 2025-2-15"),
            Command::SetDate("2025-2-15".to_owned())
        );
        assert_eq!(
            Command::from_str("date 2021-12-5"),
            Command::SetDate("2021-12-5".to_owned())
        );
        assert_eq!(Command::from_str("date"), Command::ShowDate);
    }
}
