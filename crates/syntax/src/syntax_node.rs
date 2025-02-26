pub(crate) use rowan::{GreenNode, GreenToken, NodeOrToken};
use rowan::{GreenNodeBuilder, Language};

use crate::SyntaxKind;

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
    // errors: Vec<SyntaxError>,
    inner: GreenNodeBuilder<'static>,
}
