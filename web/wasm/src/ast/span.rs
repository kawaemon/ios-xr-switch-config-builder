/// Represents a line number in the source file (1-indexed)
/// Line number wrapper to avoid mixing with other numeric types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LineNumber(pub u32);

impl LineNumber {
    /// Get the underlying line number.
    pub fn get(&self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for LineNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents a span (position range) in the source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// Line number associated with the span.
    pub line: LineNumber,
    /// Zero-based column where the token starts.
    pub col_start: u32,
    /// Zero-based column where the token ends.
    pub col_end: u32,
}

impl Span {
    /// Construct a span that covers only a line (columns default to 0).
    pub fn line_only(line: u32) -> Self {
        Span {
            line: LineNumber(line),
            col_start: 0,
            col_end: 0,
        }
    }
}

/// Value annotated with its source span for diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spanned<T> {
    /// Underlying parsed value.
    pub value: T,
    /// Source position for the value.
    pub span: Span,
}

impl<T> Spanned<T> {
    /// Create a new spanned value.
    pub fn new(value: T, span: Span) -> Self {
        Spanned { value, span }
    }

    /// Transform the inner value while preserving the span.
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Spanned<U> {
        Spanned {
            value: f(self.value),
            span: self.span,
        }
    }

    /// Borrow the inner value while keeping the span reference.
    pub fn as_ref(&self) -> Spanned<&T> {
        Spanned {
            value: &self.value,
            span: self.span,
        }
    }

    /// Split into owned value and span.
    pub fn into_parts(self) -> (T, Span) {
        (self.value, self.span)
    }
}
