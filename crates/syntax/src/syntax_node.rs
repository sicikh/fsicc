pub(crate) use rowan::GreenNode;
use rowan::{GreenNodeBuilder, Language};

use crate::{Parse, SyntaxError, SyntaxKind, TextSize};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum FsicLanguage {}

impl Language for FsicLanguage {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        SyntaxKind::from(raw.0)
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        rowan::SyntaxKind(kind.into())
    }
}

pub type SyntaxNode = rowan::SyntaxNode<FsicLanguage>;
pub type SyntaxToken = rowan::SyntaxToken<FsicLanguage>;
pub type SyntaxElement = rowan::SyntaxElement<FsicLanguage>;
pub type SyntaxNodeChildren = rowan::SyntaxNodeChildren<FsicLanguage>;
pub type SyntaxElementChildren = rowan::SyntaxElementChildren<FsicLanguage>;
pub type PreorderWithTokens = rowan::api::PreorderWithTokens<FsicLanguage>;

#[derive(Default)]
pub struct SyntaxTreeBuilder {
    errors: Vec<SyntaxError>,
    inner: GreenNodeBuilder<'static>,
}

impl SyntaxTreeBuilder {
    pub(crate) fn finish_raw(self) -> (GreenNode, Vec<SyntaxError>) {
        let green = self.inner.finish();
        (green, self.errors)
    }

    pub fn finish(self) -> Parse<SyntaxNode> {
        let (green, errors) = self.finish_raw();
        Parse::new(green, errors)
    }

    pub fn token(&mut self, kind: SyntaxKind, text: &str) {
        let kind = FsicLanguage::kind_to_raw(kind);
        self.inner.token(kind, text);
    }

    pub fn start_node(&mut self, kind: SyntaxKind) {
        let kind = FsicLanguage::kind_to_raw(kind);
        self.inner.start_node(kind);
    }

    pub fn finish_node(&mut self) {
        self.inner.finish_node();
    }

    pub fn error(&mut self, error: String, text_pos: TextSize) {
        self.errors
            .push(SyntaxError::new_at_offset(error, text_pos));
    }
}
