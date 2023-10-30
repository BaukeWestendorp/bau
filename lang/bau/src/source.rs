#[derive(Debug, Clone, PartialEq)]
pub struct Source<'text> {
    text: &'text str,
    lines: Vec<&'text str>,
}

impl<'text> Source<'text> {
    pub fn new(text: &'text str) -> Self {
        let lines = text.lines().collect();
        Self { text, lines }
    }

    pub fn text(&self) -> &'text str {
        self.text
    }

    pub fn lines(&self) -> &[&'text str] {
        &self.lines
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CodeRange {
    pub span: Span,
    pub coords: SourceCoords,
}

impl CodeRange {
    pub fn new(span: Span, coords: SourceCoords) -> Self {
        Self { span, coords }
    }

    pub fn from_ranges(start: CodeRange, end: CodeRange) -> Self {
        Self {
            span: Span::new(start.span.start, end.span.end),
            coords: start.coords,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceCoords {
    pub line: usize,
    pub column: usize,
}

impl SourceCoords {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}
