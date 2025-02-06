pub fn get_terminal_total_rows() -> u16 {
    let terminal_size = crossterm::terminal::size().expect("Doesn't support get height");
    terminal_size.1
}

pub fn get_terminal_total_cols() -> u16 {
    let terminal_size = crossterm::terminal::size().expect("Doesn't support get width");
    terminal_size.0
}

pub fn restore_terminal() -> Result<(), std::io::Error> {
    use crossterm::*;
    execute!(
        std::io::stdout(),
        terminal::LeaveAlternateScreen,
        cursor::Show
    )?;
    Ok(())
}
