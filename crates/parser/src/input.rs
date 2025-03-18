use crate::SyntaxKind::{self, *};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    pub kind: SyntaxKind,
    pub col: u32,
}

#[derive(Default)]
pub struct Input {
    tokens: Vec<Token>,
}

impl Input {
    pub fn token(&self, idx: usize) -> Token {
        self.tokens
            .get(idx)
            .copied()
            .unwrap_or(Token { kind: EOF, col: 0 })
    }

    pub fn push(&mut self, kind: SyntaxKind, col: u32) {
        self.tokens.push(Token { kind, col });
    }
}
