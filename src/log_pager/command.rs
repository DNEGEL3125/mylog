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
