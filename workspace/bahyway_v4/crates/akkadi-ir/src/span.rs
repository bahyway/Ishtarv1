//! 𒁾 Source Span — position in .akk source for error reporting

/// Location in a `.akk` source file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub source: String,
    pub line: u32,
    pub col: u32,
    pub offset: usize,
    pub len: usize,
}

impl Span {
    pub fn new(source: &str, line: u32, col: u32, offset: usize, len: usize) -> Self {
        Self {
            source: source.to_string(),
            line,
            col,
            offset,
            len,
        }
    }

    pub fn generated() -> Self {
        Self {
            source: "<generated>".into(),
            line: 0,
            col: 0,
            offset: 0,
            len: 0,
        }
    }

    pub fn is_generated(&self) -> bool {
        self.line == 0
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_generated() {
            write!(f, "<generated>")
        } else {
            write!(f, "{}:{}:{}", self.source, self.line, self.col)
        }
    }
}

impl Default for Span {
    fn default() -> Self {
        Self::generated()
    }
}
