mod items;

use crate::{
    SyntaxKind::{self, *},
    T,
    parser::{Marker, Parser},
};

pub(crate) mod entry {
    use super::*;

    pub(crate) mod top {
        use super::*;

        pub(crate) fn source_file(p: &mut Parser<'_>) {
            let m = p.start();
            items::module_contents(p);
            m.complete(p, MODULE);
        }
    }
}

fn name(p: &mut Parser<'_>) {
    if p.at(IDENT) {
        let m = p.start();
        p.bump(IDENT);
        m.complete(p, NAME);
    } else {
        todo!("error recovery")
    }
}
