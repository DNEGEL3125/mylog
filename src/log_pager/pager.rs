pub trait Pager {
    fn begin_line_index(&self) -> usize;
    fn colored_lines(&self) -> &Vec<String>;
    fn set_begin_line_index(&mut self, line_index: usize);
}
