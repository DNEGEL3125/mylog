use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum Command {
    ShowDate,
    SetDate(String),
    None,
}

impl FromStr for Command {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(Command::None);
        }
        Ok(if parts[0] == "date" {
            match parts.len() {
                1 => Command::ShowDate,
                2 => Command::SetDate(parts[1].to_owned()),
                _ => Command::None,
            }
        } else {
            Command::None
        })
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::Command;
    #[test]
    fn test_command_from_str() {
        assert_eq!(
            Command::from_str("date 2025-2-15"),
            Ok(Command::SetDate("2025-2-15".to_owned()))
        );
        assert_eq!(
            Command::from_str("date 2021-12-5"),
            Ok(Command::SetDate("2021-12-5".to_owned()))
        );
        assert_eq!(Command::from_str("date"), Ok(Command::ShowDate));
    }
}
