pub struct Range {
    pub begin: usize,
    pub end: usize,
}

impl Range {
    pub fn new(begin: usize, end: usize) -> Self {
        assert!(begin <= end, "begin > end");
        Self { begin, end }
    }
}
