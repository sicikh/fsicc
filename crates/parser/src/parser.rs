use std::cell::Cell;

use drop_bomb::DropBomb;

use crate::{
    SyntaxKind::{self, *},
    event::Event,
    input::{Input, Token},
};

pub struct Parser<'t> {
    input: &'t Input,
    pos: usize,
    events: Vec<Event>,
    steps: Cell<u32>,
    indentation_blocks: Vec<u32>,
}

const PARSER_STEP_LIMIT: usize = 15_000_000;

impl<'t> Parser<'t> {
    pub fn new(input: &'t Input) -> Parser<'t> {
        Parser {
            input,
            pos: 0,
            events: Vec::new(),
            steps: Cell::new(0),
            indentation_blocks: vec![0],
        }
    }

    pub fn finish(self) -> Vec<Event> {
        self.events
    }

    pub fn current(&self) -> Token {
        self.nth(0)
    }

    pub fn nth(&self, n: usize) -> Token {
        assert!(n <= 3);

        let steps = self.steps.get();
        assert!(
            (steps as usize) < PARSER_STEP_LIMIT,
            "the parser seems stuck"
        );
        self.steps.set(steps + 1);

        self.input.token(self.pos + n)
    }

    pub fn at(&self, kind: SyntaxKind) -> bool {
        self.nth_at(0, kind)
    }

    fn nth_at(&self, n: usize, kind: SyntaxKind) -> bool {
        self.input.token(self.pos + n).kind == kind
    }

    fn push_event(&mut self, event: Event) {
        self.events.push(event);
    }

    pub fn start(&mut self) -> Marker {
        let pos = self.events.len() as u32;
        self.push_event(Event::tombstone());
        Marker::new(pos)
    }

    pub fn bump(&mut self, kind: SyntaxKind) {
        assert!(self.eat(kind));
    }

    pub fn eat(&mut self, kind: SyntaxKind) -> bool {
        if !self.at(kind) {
            return false;
        }

        let n_raw_tokens = 1;

        self.do_bump(kind, n_raw_tokens);
        true
    }

    pub fn do_bump(&mut self, kind: SyntaxKind, n_raw_tokens: u8) {
        self.pos += n_raw_tokens as usize;
        self.steps.set(0);
        self.push_event(Event::Token { kind, n_raw_tokens });
    }

    /// Advances the parser by one token
    pub(crate) fn bump_any(&mut self) {
        let kind = self.nth(0).kind;
        if kind == EOF {
            return;
        }
        self.do_bump(kind, 1);
    }

    /// Emit error with the `message`
    /// FIXME: this should be much more fancy and support
    /// structured errors with spans and notes, like rustc
    /// does.
    pub(crate) fn error<T: Into<String>>(&mut self, message: T) {
        let msg = message.into();
        self.push_event(Event::Error { msg });
    }

    /// Consume the next token if it is `kind` or emit an error
    /// otherwise.
    pub(crate) fn expect(&mut self, kind: SyntaxKind) -> bool {
        if self.eat(kind) {
            return true;
        }
        self.error(format!("expected {kind:?}"));
        false
    }

    pub(crate) fn new_indent_block(&mut self) {
        self.indentation_blocks.push(self.current().col);
    }

    pub(crate) fn drop_indent_block(&mut self) {
        self.indentation_blocks.pop();
    }

    pub(crate) fn get_current_indent_block(&self) -> (u32, usize) {
        (
            *self.indentation_blocks.last().unwrap(),
            self.indentation_blocks.len() - 1,
        )
    }

    pub(crate) fn get_indent_block(&self, index: usize) -> u32 {
        *self.indentation_blocks.get(index).unwrap()
    }

    /// Zero or more at the same indentation level.
    pub(crate) fn many_same(&mut self, p: impl Fn(&mut Self)) {
        let (block, _) = self.get_current_indent_block();
        while !self.at(EOF) {
            if self.current().col != block {
                break;
            }
            p(self);
        }
    }
}

pub struct Marker {
    pos: u32,
    bomb: DropBomb,
}

impl Marker {
    fn new(pos: u32) -> Marker {
        Marker {
            pos,
            bomb: DropBomb::new("Marker must be either completed or abandoned"),
        }
    }

    pub fn complete(mut self, p: &mut Parser<'_>, kind: SyntaxKind) -> CompletedMarker {
        self.bomb.defuse();

        let idx = self.pos as usize;

        match &mut p.events[idx] {
            Event::Start { kind: slot, .. } => {
                *slot = kind;
            },
            _ => unreachable!(),
        }
        p.push_event(Event::Finish);
        let end_pos = p.events.len() as u32;
        CompletedMarker::new(self.pos, end_pos, kind)
    }

    /// Abandons the syntax tree node. All its children
    /// are attached to its parent instead.
    pub(crate) fn abandon(mut self, p: &mut Parser<'_>) {
        self.bomb.defuse();
        let idx = self.pos as usize;
        if idx == p.events.len() - 1 {
            assert!(matches!(
                p.events.pop(),
                Some(Event::Start {
                    kind: TOMBSTONE,
                    forward_parent: None
                })
            ));
        }
    }
}

pub struct CompletedMarker {
    start_pos: u32,
    end_pos: u32,
    kind: SyntaxKind,
}

impl CompletedMarker {
    fn new(start_pos: u32, end_pos: u32, kind: SyntaxKind) -> Self {
        CompletedMarker {
            start_pos,
            end_pos,
            kind,
        }
    }

    /// This method allows to create a new node which starts
    /// *before* the current one. That is, parser could start
    /// node `A`, then complete it, and then after parsing the
    /// whole `A`, decide that it should have started some node
    /// `B` before starting `A`. `precede` allows to do exactly
    /// that. See also docs about
    /// [`Event::Start::forward_parent`](crate::event::Event::Start::forward_parent).
    ///
    /// Given completed events `[START, FINISH]` and its corresponding
    /// `CompletedMarker(pos: 0, _)`.
    /// Append a new `START` events as `[START, FINISH, NEWSTART]`,
    /// then mark `NEWSTART` as `START`'s parent with saving its relative
    /// distance to `NEWSTART` into forward_parent(=2 in this case);
    pub(crate) fn precede(self, p: &mut Parser<'_>) -> Marker {
        let new_pos = p.start();
        let idx = self.start_pos as usize;
        match &mut p.events[idx] {
            Event::Start { forward_parent, .. } => {
                *forward_parent = Some(new_pos.pos - self.start_pos);
            },
            _ => unreachable!(),
        }
        new_pos
    }

    /// Extends this completed marker *to the left* up to `m`.
    pub(crate) fn extend_to(self, p: &mut Parser<'_>, mut m: Marker) -> CompletedMarker {
        m.bomb.defuse();
        let idx = m.pos as usize;
        match &mut p.events[idx] {
            Event::Start { forward_parent, .. } => {
                *forward_parent = Some(self.start_pos - m.pos);
            },
            _ => unreachable!(),
        }
        self
    }

    pub(crate) fn kind(&self) -> SyntaxKind {
        self.kind
    }
}
