pub fn get_terminal_total_rows() -> u16 {
    let terminal_size = crossterm::terminal::size().expect("Doesn't support get height");
    return terminal_size.1;
}

pub fn get_terminal_total_cols() -> u16 {
    let terminal_size = crossterm::terminal::size().expect("Doesn't support get width");
    return terminal_size.0;
}
