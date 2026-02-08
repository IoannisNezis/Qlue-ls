mod generated;
pub use generated::SyntaxKind::{self};

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}

impl SyntaxKind {
    pub fn is_trivia(&self) -> bool {
        matches!(self, SyntaxKind::WHITESPACE | SyntaxKind::Comment)
    }
}
