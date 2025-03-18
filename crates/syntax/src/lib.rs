#![allow(unused)]

pub mod ast;
mod parsing;
mod syntax_error;
mod syntax_node;

use std::marker::PhantomData;

pub use parser::{SyntaxKind, T};
pub use rowan::{
    Direction, GreenNode, NodeOrToken, SyntaxText, TextRange, TextSize, TokenAtOffset, WalkEvent,
    api::Preorder,
};
use triomphe::Arc;

pub use crate::{
    ast::{AstNode, AstToken},
    syntax_error::SyntaxError,
    syntax_node::{
        FsicLanguage, PreorderWithTokens, SyntaxElement, SyntaxElementChildren, SyntaxNode,
        SyntaxNodeChildren, SyntaxToken, SyntaxTreeBuilder,
    },
};

/// `Parse` is the result of the parsing: a syntax tree and a collection of
/// errors.
///
/// Note that we always produce a syntax tree, even for completely invalid
/// files.
#[derive(Debug, PartialEq, Eq)]
pub struct Parse<T> {
    green: GreenNode,
    errors: Option<Arc<[SyntaxError]>>,
    _ty: PhantomData<fn() -> T>,
}

impl<T> Clone for Parse<T> {
    fn clone(&self) -> Parse<T> {
        Parse {
            green: self.green.clone(),
            errors: self.errors.clone(),
            _ty: PhantomData,
        }
    }
}

impl<T> Parse<T> {
    fn new(green: GreenNode, errors: Vec<SyntaxError>) -> Parse<T> {
        Parse {
            green,
            errors: if errors.is_empty() {
                None
            } else {
                Some(errors.into())
            },
            _ty: PhantomData,
        }
    }

    pub fn syntax_node(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.green.clone())
    }

    pub fn errors(&self) -> Vec<SyntaxError> {
        let mut errors = if let Some(e) = self.errors.as_deref() {
            e.to_vec()
        } else {
            vec![]
        };
        // validation::validate(&self.syntax_node(), &mut errors);
        errors
    }
}

impl<T: AstNode> Parse<T> {
    /// Converts this parse result into a parse result for an untyped syntax
    /// tree.
    pub fn to_syntax(self) -> Parse<SyntaxNode> {
        Parse {
            green: self.green,
            errors: self.errors,
            _ty: PhantomData,
        }
    }

    /// Gets the parsed syntax tree as a typed ast node.
    ///
    /// # Panics
    ///
    /// Panics if the root node cannot be casted into the typed ast node
    /// (e.g. if it's an `ERROR` node).
    pub fn tree(&self) -> T {
        T::cast(self.syntax_node()).unwrap()
    }

    /// Converts from `Parse<T>` to [`Result<T, Vec<SyntaxError>>`].
    pub fn ok(self) -> Result<T, Vec<SyntaxError>> {
        match self.errors() {
            errors if !errors.is_empty() => Err(errors),
            _ => Ok(self.tree()),
        }
    }
}

impl Parse<SyntaxNode> {
    pub fn cast<N: AstNode>(self) -> Option<Parse<N>> {
        if N::cast(self.syntax_node()).is_some() {
            Some(Parse {
                green: self.green,
                errors: self.errors,
                _ty: PhantomData,
            })
        } else {
            None
        }
    }
}

pub use crate::ast::Module;

impl Module {
    pub fn parse(text: &str) -> Parse<Module> {
        let (green, errors) = parsing::parse_text(text);

        let root = SyntaxNode::new_root(green.clone());
        assert_eq!(root.kind(), SyntaxKind::MODULE);

        Parse::new(green, errors)
    }
}

#[test]
fn api_walkthrough() {
    let source_code = "let main\nmodule It =\n  let main\n  let main\nlet main";

    let parse = Module::parse(source_code);
    assert!(parse.errors().is_empty());

    let module: Module = parse.tree();

    let mut buf = String::new();
    let mut indent = 0;
    for event in module.syntax().preorder_with_tokens() {
        match event {
            WalkEvent::Enter(node) => {
                let text = match &node {
                    NodeOrToken::Node(it) => "".to_string(),
                    NodeOrToken::Token(it) => format!(" {:?}", it.text()),
                };
                use std::fmt::Write;
                writeln!(
                    buf,
                    "{:indent$}{:?}{}",
                    " ",
                    node.kind(),
                    text,
                    indent = indent
                )
                .unwrap();
                indent += 2;
            },
            WalkEvent::Leave(_) => indent -= 2,
        }
    }
    //     assert_eq!(indent, 0);
    //     assert_eq!(
    //         buf.trim(),
    //         r#"
    // "let main" MODULE
    //   "let main" LET_DECL
    //     "let" LET_KW
    //     " " WHITESPACE
    //     "main" NAME
    //       "main" IDENT
    // "#
    //         .trim()
    //     );
    assert_eq!(indent, 0);
    assert_eq!(
        buf.trim(),
        r#"
MODULE
  LET_DECL
    LET_KW "let"
    WHITESPACE " "
    NAME
      IDENT "main"
  NEWLINE "\n"
  NESTED_MODULE
    MODULE_KW "module"
    WHITESPACE " "
    NAME
      IDENT "It"
    WHITESPACE " "
    EQ "="
    NEWLINE "\n"
    WHITESPACE " "
    WHITESPACE " "
    LET_DECL
      LET_KW "let"
      WHITESPACE " "
      NAME
        IDENT "main"
    NEWLINE "\n"
    WHITESPACE " "
    WHITESPACE " "
    LET_DECL
      LET_KW "let"
      WHITESPACE " "
      NAME
        IDENT "main"
  NEWLINE "\n"
  LET_DECL
    LET_KW "let"
    WHITESPACE " "
    NAME
      IDENT "main"
"#
        .trim()
    );
}
