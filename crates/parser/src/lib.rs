#![allow(unused)]

mod event;
mod grammar;
mod input;
mod lexed_str;
mod output;
mod parser;
mod shortcuts;
mod syntax_kind;

pub use crate::{
    input::Input,
    lexed_str::LexedStr,
    output::{Output, Step},
    shortcuts::StrStep,
    syntax_kind::SyntaxKind,
};

pub enum TopEntryPoint {
    SourceFile,
}

impl TopEntryPoint {
    pub fn parse(&self, input: &Input) -> Output {
        let entry_point: fn(&'_ mut parser::Parser<'_>) = match self {
            TopEntryPoint::SourceFile => grammar::entry::top::source_file,
        };

        let mut p = parser::Parser::new(input);
        entry_point(&mut p);
        let events = p.finish();
        let res = event::process(events);

        if cfg!(debug_assertions) {
            let mut depth = 0;
            let mut first = true;
            for step in res.iter() {
                assert!(depth > 0 || first);
                first = false;
                match step {
                    Step::Enter { .. } => depth += 1,
                    Step::Exit => depth -= 1,
                    Step::Token { .. } | Step::Error { .. } => (),
                }
            }
            assert!(!first, "no tree at all");
            assert_eq!(depth, 0, "unbalanced tree");
        }

        res
    }
}
