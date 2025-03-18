use logos::Logos;

pub struct Token {
    pub kind: TokenKind,
    pub len: u32,
}

impl Token {
    fn new(kind: TokenKind, len: u32) -> Self {
        Self { kind, len }
    }
}

#[derive(Logos, Copy, Clone, Debug, PartialEq, Eq)]
pub enum TokenKind {
    #[token("let")]
    Let,
    #[token("module")]
    Module,

    #[token("=")]
    Eq,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident,

    #[regex("[ ]")]
    Whitespace,

    #[regex("(\n|\r\n)")]
    Newline,

    #[token("(")]
    OpenParen,

    #[token(")")]
    CloseParen,

    #[regex("[0-9]+")]
    Number,

    Unknown,

    Eof,
}

pub fn tokenize(input: &str) -> impl Iterator<Item = Token> {
    let mut lexer = TokenKind::lexer(input);
    std::iter::from_fn(move || {
        match lexer.next() {
            Some(Ok(token_kind)) => {
                let len = lexer.slice().len() as u32;
                Some(Token::new(token_kind, len))
            },
            Some(Err(_)) => Some(Token::new(TokenKind::Unknown, lexer.slice().len() as u32)),
            None => None,
        }
    })
}
