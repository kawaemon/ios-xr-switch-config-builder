/// Represents a line number in the source file (1-indexed)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LineNumber(pub u32);

impl LineNumber {
    pub fn get(&self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for LineNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents a span (position range) in the source file
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub line: LineNumber,
    pub col_start: u32,
    pub col_end: u32,
}

impl Span {
    pub fn line_only(line: u32) -> Self {
        Span {
            line: LineNumber(line),
            col_start: 0,
            col_end: 0,
        }
    }
}
