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
}
