use crate::{
    SyntaxKind::{EOF, LET_DECL, NESTED_MODULE},
    T,
    grammar::name,
    parser::{Marker, Parser},
};

pub(super) fn module_contents(p: &mut Parser<'_>) {
    p.new_indent_block();

    p.many_same(item);

    p.drop_indent_block();
}

fn item(p: &mut Parser<'_>) {
    let m = p.start();

    match p.current().kind {
        T![let] => let_(p, m),
        T![module] => module(p, m),
        _ => {
            m.abandon(p);
            p.error("expected item");
            p.bump_any();
        },
    }
}

fn let_(p: &mut Parser<'_>, m: Marker) {
    p.bump(T![let]);

    name(p);

    // parens(p);

    if p.eat(T![=]) {
        // parse expr
    }

    m.complete(p, LET_DECL);
}

fn module(p: &mut Parser<'_>, m: Marker) {
    p.new_indent_block();
    p.bump(T![module]);

    // TODO: >= relation for terminals' indentation??
    name(p);

    p.expect(T![=]);

    let (block, _) = p.get_current_indent_block();
    if p.current().col < block {
        p.error("module should contain at least one item");
        return;
    }

    module_contents(p);

    m.complete(p, NESTED_MODULE);
    p.drop_indent_block();
}
