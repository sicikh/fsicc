mod ast_src;

use std::{
    collections::{BTreeSet, HashSet},
    fmt::Write,
    fs,
};

use itertools::{Either, Itertools};
use proc_macro2::{Punct, Spacing};
use quote::{format_ident, quote};
use ungrammar::{Grammar, Rule};

use self::ast_src::{AstEnumSrc, AstNodeSrc, AstSrc, Cardinality, Field, KindsSrc};
use crate::{
    codegen::{add_preamble, ensure_file_contents, grammar::ast_src::generate_kind_src, reformat},
    project_root,
};

pub(crate) fn generate(check: bool) {
    let grammar = fs::read_to_string(project_root().join("crates/syntax/fsic.ungram"))
        .unwrap()
        .parse()
        .unwrap();
    let ast = lower(&grammar);
    let kinds_src = generate_kind_src(&ast.nodes, &ast.enums, &grammar);

    let syntax_kinds = generate_syntax_kinds(kinds_src);
    let syntax_kinds_file = project_root().join("crates/parser/src/syntax_kind/generated.rs");
    ensure_file_contents(
        crate::cli::CodegenMode::Grammar,
        syntax_kinds_file.as_path(),
        &syntax_kinds,
        check,
    );

    let ast_tokens = generate_tokens(&ast);
    let ast_tokens_file = project_root().join("crates/syntax/src/ast/generated/tokens.rs");
    ensure_file_contents(
        crate::cli::CodegenMode::Grammar,
        ast_tokens_file.as_path(),
        &ast_tokens,
        check,
    );

    let ast_nodes = generate_nodes(kinds_src, &ast);
    let ast_nodes_file = project_root().join("crates/syntax/src/ast/generated/nodes.rs");
    ensure_file_contents(
        crate::cli::CodegenMode::Grammar,
        ast_nodes_file.as_path(),
        &ast_nodes,
        check,
    );
}

fn generate_tokens(grammar: &AstSrc) -> String {
    let tokens = grammar.tokens.iter().map(|token| {
        let name = format_ident!("{}", token);
        let kind = format_ident!("{}", to_upper_snake_case(token));
        quote! {
            #[derive(Debug, Clone, PartialEq, Eq, Hash)]
            pub struct #name {
                pub(crate) syntax: SyntaxToken,
            }
            impl std::fmt::Display for #name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    std::fmt::Display::fmt(&self.syntax, f)
                }
            }
            impl AstToken for #name {
                fn can_cast(kind: SyntaxKind) -> bool { kind == #kind }
                fn cast(syntax: SyntaxToken) -> Option<Self> {
                    if Self::can_cast(syntax.kind()) { Some(Self { syntax }) } else { None }
                }
                fn syntax(&self) -> &SyntaxToken { &self.syntax }
            }
        }
    });

    add_preamble(
        crate::cli::CodegenMode::Grammar,
        reformat(
            quote! {
                use crate::{SyntaxKind::{self, *}, SyntaxToken, ast::AstToken};
                #(#tokens)*
            }
            .to_string(),
        ),
    )
    .replace("#[derive", "\n#[derive")
}

fn generate_nodes(kinds: KindsSrc, grammar: &AstSrc) -> String {
    let (node_defs, node_boilerplate_impls): (Vec<_>, Vec<_>) = grammar
        .nodes
        .iter()
        .map(|node| {
            let name = format_ident!("{}", node.name);
            let kind = format_ident!("{}", to_upper_snake_case(&node.name));
            let traits = node
                .traits
                .iter()
                .filter(|trait_name| {
                    // TODO:
                    //  Loops have two expressions so this might collide, therefore manual impl it
                    node.name != "ForExpr" && node.name != "WhileExpr"
                        || trait_name.as_str() != "HasLoopBody"
                })
                .map(|trait_name| {
                    let trait_name = format_ident!("{}", trait_name);
                    quote!(impl ast::#trait_name for #name {})
                });

            let methods = node.fields.iter().map(|field| {
                let method_name = format_ident!("{}", field.method_name());
                let ty = field.ty();

                if field.is_many() {
                    quote! {
                        pub fn #method_name(&self) -> AstChildren<#ty> {
                            support::children(&self.syntax)
                        }
                    }
                } else if let Some(token_kind) = field.token_kind() {
                    quote! {
                        pub fn #method_name(&self) -> Option<#ty> {
                            support::token(&self.syntax, #token_kind)
                        }
                    }
                } else {
                    quote! {
                        pub fn #method_name(&self) -> Option<#ty> {
                            support::child(&self.syntax)
                        }
                    }
                }
            });
            (
                quote! {
                    #[pretty_doc_comment_placeholder_workaround]
                    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
                    pub struct #name {
                        pub(crate) syntax: SyntaxNode,
                    }

                    #(#traits)*

                    impl #name {
                        #(#methods)*
                    }
                },
                quote! {
                    impl AstNode for #name {
                        fn can_cast(kind: SyntaxKind) -> bool {
                            kind == #kind
                        }
                        fn cast(syntax: SyntaxNode) -> Option<Self> {
                            if Self::can_cast(syntax.kind()) { Some(Self { syntax }) } else { None }
                        }
                        fn syntax(&self) -> &SyntaxNode { &self.syntax }
                    }
                },
            )
        })
        .unzip();

    let (enum_defs, enum_boilerplate_impls): (Vec<_>, Vec<_>) = grammar
        .enums
        .iter()
        .map(|en| {
            let variants: Vec<_> = en
                .variants
                .iter()
                .map(|var| format_ident!("{}", var))
                .sorted()
                .collect();
            let name = format_ident!("{}", en.name);
            let kinds: Vec<_> = variants
                .iter()
                .map(|name| format_ident!("{}", to_upper_snake_case(&name.to_string())))
                .collect();
            let traits = en.traits.iter().sorted().map(|trait_name| {
                let trait_name = format_ident!("{}", trait_name);
                quote!(impl ast::#trait_name for #name {})
            });

            let ast_node = if en.name == "Stmt" {
                quote! {}
            } else {
                quote! {
                    impl AstNode for #name {
                        fn can_cast(kind: SyntaxKind) -> bool {
                            matches!(kind, #(#kinds)|*)
                        }
                        fn cast(syntax: SyntaxNode) -> Option<Self> {
                            let res = match syntax.kind() {
                                #(
                                #kinds => #name::#variants(#variants { syntax }),
                                )*
                                _ => return None,
                            };
                            Some(res)
                        }
                        fn syntax(&self) -> &SyntaxNode {
                            match self {
                                #(
                                #name::#variants(it) => &it.syntax,
                                )*
                            }
                        }
                    }
                }
            };

            (
                quote! {
                    #[pretty_doc_comment_placeholder_workaround]
                    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
                    pub enum #name {
                        #(#variants(#variants),)*
                    }

                    #(#traits)*
                },
                quote! {
                    #(
                        impl From<#variants> for #name {
                            fn from(node: #variants) -> #name {
                                #name::#variants(node)
                            }
                        }
                    )*
                    #ast_node
                },
            )
        })
        .unzip();
    let (any_node_defs, any_node_boilerplate_impls): (Vec<_>, Vec<_>) = grammar
        .nodes
        .iter()
        .flat_map(|node| node.traits.iter().map(move |t| (t, node)))
        .into_group_map()
        .into_iter()
        .sorted_by_key(|(name, _)| *name)
        .map(|(trait_name, nodes)| {
            let name = format_ident!("Any{}", trait_name);
            let trait_name = format_ident!("{}", trait_name);
            let kinds: Vec<_> = nodes
                .iter()
                .map(|name| format_ident!("{}", to_upper_snake_case(&name.name.to_string())))
                .collect();
            let nodes = nodes.iter().map(|node| format_ident!("{}", node.name));
            (
                quote! {
                    #[pretty_doc_comment_placeholder_workaround]
                    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
                    pub struct #name {
                        pub(crate) syntax: SyntaxNode,
                    }
                    impl ast::#trait_name for #name {}
                },
                quote! {
                    impl #name {
                        #[inline]
                        pub fn new<T: ast::#trait_name>(node: T) -> #name {
                            #name {
                                syntax: node.syntax().clone()
                            }
                        }
                    }
                    impl AstNode for #name {
                        #[inline]
                        fn can_cast(kind: SyntaxKind) -> bool {
                            matches!(kind, #(#kinds)|*)
                        }
                        #[inline]
                        fn cast(syntax: SyntaxNode) -> Option<Self> {
                            Self::can_cast(syntax.kind()).then_some(#name { syntax })
                        }
                        #[inline]
                        fn syntax(&self) -> &SyntaxNode {
                            &self.syntax
                        }
                    }

                    #(
                        impl From<#nodes> for #name {
                            #[inline]
                            fn from(node: #nodes) -> #name {
                                #name { syntax: node.syntax }
                            }
                        }
                    )*
                },
            )
        })
        .unzip();

    let enum_names = grammar.enums.iter().map(|it| &it.name);
    let node_names = grammar.nodes.iter().map(|it| &it.name);

    let display_impls = enum_names
        .chain(node_names.clone())
        .map(|it| format_ident!("{}", it))
        .map(|name| {
            quote! {
                impl std::fmt::Display for #name {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        std::fmt::Display::fmt(self.syntax(), f)
                    }
                }
            }
        });

    let defined_nodes: HashSet<_> = node_names.collect();

    for node in kinds
        .nodes
        .iter()
        .map(|kind| to_pascal_case(kind))
        .filter(|name| !defined_nodes.iter().any(|&it| it == name))
    {
        drop(node);
        // eprintln!("Warning: node {} not defined in ast source", node);
    }

    let ast = quote! {
        #![allow(non_snake_case)]
        use crate::{
            SyntaxNode, SyntaxToken, SyntaxKind::{self, *},
            ast::{self, AstNode, AstChildren, support},
            T,
        };

        #(#node_defs)*
        #(#enum_defs)*
        #(#any_node_defs)*
        #(#node_boilerplate_impls)*
        #(#enum_boilerplate_impls)*
        #(#any_node_boilerplate_impls)*
        #(#display_impls)*
    };

    let ast = ast.to_string().replace("T ! [", "T![");

    let mut res = String::with_capacity(ast.len() * 2);

    let mut docs = grammar
        .nodes
        .iter()
        .map(|it| &it.doc)
        .chain(grammar.enums.iter().map(|it| &it.doc));

    for chunk in ast.split("# [pretty_doc_comment_placeholder_workaround] ") {
        res.push_str(chunk);
        if let Some(doc) = docs.next() {
            write_doc_comment(doc, &mut res);
        }
    }

    let res = add_preamble(crate::cli::CodegenMode::Grammar, reformat(res));
    res.replace("#[derive", "\n#[derive")
}

fn write_doc_comment(contents: &[String], dest: &mut String) {
    for line in contents {
        writeln!(dest, "///{line}").unwrap();
    }
}

fn generate_syntax_kinds(grammar: KindsSrc) -> String {
    let (single_byte_tokens_values, single_byte_tokens): (Vec<_>, Vec<_>) = grammar
        .punct
        .iter()
        .filter(|(token, _name)| token.len() == 1)
        .map(|(token, name)| (token.chars().next().unwrap(), format_ident!("{}", name)))
        .unzip();

    let punctuation_values = grammar.punct.iter().map(|(token, _name)| {
        if "{}[]()".contains(token) {
            let c = token.chars().next().unwrap();
            quote! { #c }
        } else if *token == "_" {
            quote! { _ }
        } else {
            let cs = token.chars().map(|c| Punct::new(c, Spacing::Joint));
            quote! { #(#cs)* }
        }
    });
    let punctuation = grammar
        .punct
        .iter()
        .map(|(_token, name)| format_ident!("{}", name))
        .collect::<Vec<_>>();

    let x = |&name| format_ident!("{}_KW", to_upper_snake_case(name));
    let full_keywords_values = grammar.keywords;
    let full_keywords = full_keywords_values.iter().map(x);

    let all_keywords_values = grammar.keywords.to_vec();
    let all_keywords_idents = all_keywords_values.iter().map(|kw| format_ident!("{}", kw));
    let all_keywords = all_keywords_values.iter().map(x).collect::<Vec<_>>();

    let literals = grammar
        .literals
        .iter()
        .map(|name| format_ident!("{}", name))
        .collect::<Vec<_>>();

    let tokens = grammar
        .tokens
        .iter()
        .map(|name| format_ident!("{}", name))
        .collect::<Vec<_>>();

    let nodes = grammar
        .nodes
        .iter()
        .map(|name| format_ident!("{}", name))
        .collect::<Vec<_>>();

    let ast = quote! {
        #![allow(bad_style, missing_docs, unreachable_pub)]
        /// The kind of syntax node, e.g. `IDENT`, `VOID_KW`, or `EXPR`.
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
        #[repr(u16)]
        pub enum SyntaxKind {
            // Technical SyntaxKinds: they appear temporally during parsing,
            // but never end up in the final tree
            #[doc(hidden)]
            TOMBSTONE,
            #[doc(hidden)]
            EOF,
            #(#punctuation,)*
            #(#all_keywords,)*
            #(#literals,)*
            #(#tokens,)*
            #(#nodes,)*

            // Technical kind so that we can cast from u16 safely
            #[doc(hidden)]
            __LAST,
        }
        use self::SyntaxKind::*;

        impl SyntaxKind {
            pub fn is_keyword(self) -> bool {
                matches!(self, #(#all_keywords)|*)
            }

            pub fn is_punct(self) -> bool {

                matches!(self, #(#punctuation)|*)

            }

            pub fn is_literal(self) -> bool {
                matches!(self, #(#literals)|*)
            }

            pub fn from_keyword(ident: &str) -> Option<SyntaxKind> {
                let kw = match ident {
                    #(#full_keywords_values => #full_keywords,)*
                    _ => return None,
                };
                Some(kw)
            }

            pub fn from_char(c: char) -> Option<SyntaxKind> {
                let tok = match c {
                    #(#single_byte_tokens_values => #single_byte_tokens,)*
                    _ => return None,
                };
                Some(tok)
            }
        }

        #[macro_export]
        macro_rules! T {
            #([#punctuation_values] => { $crate::SyntaxKind::#punctuation };)*
            #([#all_keywords_idents] => { $crate::SyntaxKind::#all_keywords };)*
            [ident] => { $crate::SyntaxKind::IDENT };
        }
    };

    add_preamble(crate::cli::CodegenMode::Grammar, reformat(ast.to_string()))
}

fn to_upper_snake_case(s: &str) -> String {
    let mut buf = String::with_capacity(s.len());
    let mut prev = false;
    for c in s.chars() {
        if c.is_ascii_uppercase() && prev {
            buf.push('_')
        }
        prev = true;

        buf.push(c.to_ascii_uppercase());
    }
    buf
}

fn to_lower_snake_case(s: &str) -> String {
    let mut buf = String::with_capacity(s.len());
    let mut prev = false;
    for c in s.chars() {
        if c.is_ascii_uppercase() && prev {
            buf.push('_')
        }
        prev = true;

        buf.push(c.to_ascii_lowercase());
    }
    buf
}

fn to_pascal_case(s: &str) -> String {
    let mut buf = String::with_capacity(s.len());
    let mut prev_is_underscore = true;
    for c in s.chars() {
        if c == '_' {
            prev_is_underscore = true;
        } else if prev_is_underscore {
            buf.push(c.to_ascii_uppercase());
            prev_is_underscore = false;
        } else {
            buf.push(c.to_ascii_lowercase());
        }
    }
    buf
}

fn pluralize(s: &str) -> String {
    format!("{s}s")
}

impl Field {
    fn is_many(&self) -> bool {
        matches!(self, Field::Node {
            cardinality: Cardinality::Many,
            ..
        })
    }
    fn token_kind(&self) -> Option<proc_macro2::TokenStream> {
        match self {
            Field::Token(token) => {
                let token: proc_macro2::TokenStream = token.parse().unwrap();
                Some(quote! { T![#token] })
            },
            _ => None,
        }
    }
    fn method_name(&self) -> String {
        match self {
            Field::Token(name) => {
                let name = match name.as_str() {
                    ";" => "semicolon",
                    "->" => "arrow",
                    "'{'" => "l_curly",
                    "'}'" => "r_curly",
                    "'('" => "l_paren",
                    "')'" => "r_paren",
                    "'['" => "l_brack",
                    "']'" => "r_brack",
                    "<" => "l_angle",
                    ">" => "r_angle",
                    "=" => "eq",
                    "!" => "excl",
                    "*" => "star",
                    "&" => "amp",
                    "-" => "minus",
                    "_" => "underscore",
                    "." => "dot",
                    ":" => "colon",
                    "::" => "colon2",
                    "?" => "question_mark",
                    "," => "comma",
                    "|" => "pipe",
                    "~" => "tilde",
                    _ => name,
                };
                format!("{}_token", name)
            },
            Field::Node { name, .. } => {
                if name == "type" {
                    String::from("ty")
                } else {
                    name.to_string()
                }
            },
        }
    }
    fn ty(&self) -> proc_macro2::Ident {
        match self {
            Field::Token(_) => format_ident!("SyntaxToken"),
            Field::Node { ty, .. } => format_ident!("{}", ty),
        }
    }
}

fn clean_token_name(name: &str) -> String {
    let cleaned = name.trim_start_matches(['@', '#', '?']);
    if cleaned.is_empty() {
        name.to_owned()
    } else {
        cleaned.to_owned()
    }
}

fn lower(grammar: &Grammar) -> AstSrc {
    let mut res = AstSrc {
        tokens: "Whitespace Comment String FString IntNumber FloatNumber Char Ident"
            .split_ascii_whitespace()
            .map(|it| it.to_owned())
            .collect::<Vec<_>>(),
        ..Default::default()
    };

    let nodes = grammar.iter().collect::<Vec<_>>();

    for &node in &nodes {
        let name = grammar[node].name.clone();
        let rule = &grammar[node].rule;
        match lower_enum(grammar, rule) {
            Some(variants) => {
                let enum_src = AstEnumSrc {
                    doc: Vec::new(),
                    name,
                    traits: Vec::new(),
                    variants,
                };
                res.enums.push(enum_src);
            },
            None => {
                let mut fields = Vec::new();
                lower_rule(&mut fields, grammar, None, rule);
                res.nodes.push(AstNodeSrc {
                    doc: Vec::new(),
                    name,
                    traits: Vec::new(),
                    fields,
                });
            },
        }
    }

    deduplicate_fields(&mut res);
    extract_enums(&mut res);
    extract_struct_traits(&mut res);
    extract_enum_traits(&mut res);
    res.nodes.sort_by_key(|it| it.name.clone());
    res.enums.sort_by_key(|it| it.name.clone());
    res.tokens.sort();
    res.nodes.iter_mut().for_each(|it| {
        it.traits.sort();
        it.fields.sort_by_key(|it| {
            match it {
                Field::Token(name) => (true, name.clone()),
                Field::Node { name, .. } => (false, name.clone()),
            }
        });
    });
    res.enums.iter_mut().for_each(|it| {
        it.traits.sort();
        it.variants.sort();
    });
    res
}

fn lower_enum(grammar: &Grammar, rule: &Rule) -> Option<Vec<String>> {
    let alternatives = match rule {
        Rule::Alt(it) => it,
        _ => return None,
    };
    let mut variants = Vec::new();
    for alternative in alternatives {
        match alternative {
            Rule::Node(it) => variants.push(grammar[*it].name.clone()),
            Rule::Token(it) if grammar[*it].name == ";" => (),
            _ => return None,
        }
    }
    Some(variants)
}

fn lower_rule(acc: &mut Vec<Field>, grammar: &Grammar, label: Option<&String>, rule: &Rule) {
    if lower_list(acc, grammar, label, rule) {
        return;
    }

    match rule {
        Rule::Node(node) => {
            let ty = grammar[*node].name.clone();
            let name = label.cloned().unwrap_or_else(|| to_lower_snake_case(&ty));
            let field = Field::Node {
                name,
                ty,
                cardinality: Cardinality::Optional,
            };
            acc.push(field);
        },
        Rule::Token(token) => {
            if label.is_some() {
                println!("{label:?}")
            }
            assert!(label.is_none());
            let mut name = clean_token_name(&grammar[*token].name);
            if "[]{}()".contains(&name) {
                name = format!("'{name}'");
            }
            let field = Field::Token(name);
            acc.push(field);
        },
        Rule::Rep(inner) => {
            if let Rule::Node(node) = &**inner {
                let ty = grammar[*node].name.clone();
                let name = label
                    .cloned()
                    .unwrap_or_else(|| pluralize(&to_lower_snake_case(&ty)));
                let field = Field::Node {
                    name,
                    ty,
                    cardinality: Cardinality::Many,
                };
                acc.push(field);
                return;
            }

            panic!("unhandled rule: {rule:?}")
        },
        Rule::Labeled { label: l, rule } => {
            assert!(label.is_none());
            // TODO!
            let manually_implemented = matches!(
                l.as_str(),
                "lhs"
                    | "rhs"
                    | "then_branch"
                    | "else_branch"
                    | "start"
                    | "end"
                    | "op"
                    | "index"
                    | "base"
                    | "value"
                    | "trait"
                    | "self_ty"
                    | "iterable"
                    | "condition"
                    | "args"
                    | "body"
            );
            if manually_implemented {
                return;
            }
            lower_rule(acc, grammar, Some(l), rule);
        },
        Rule::Seq(rules) | Rule::Alt(rules) => {
            for rule in rules {
                lower_rule(acc, grammar, label, rule)
            }
        },
        Rule::Opt(rule) => lower_rule(acc, grammar, label, rule),
    }
}

// (T (',' T)* ','?)
// (T T*)
fn lower_list(
    acc: &mut Vec<Field>,
    grammar: &Grammar,
    label: Option<&String>,
    rule: &Rule,
) -> bool {
    let rule = match rule {
        Rule::Seq(it) => it,
        _ => return false,
    };

    let (nt, _repeat, _trailing_sep) = match rule.as_slice() {
        [Rule::Node(node), Rule::Rep(repeat), Rule::Opt(trailing_sep)] => {
            (Either::Left(node), repeat, Some(trailing_sep))
        },
        [Rule::Node(node), Rule::Rep(repeat)] => (Either::Left(node), repeat, None),
        [
            Rule::Token(token),
            Rule::Rep(repeat),
            Rule::Opt(trailing_sep),
        ] => (Either::Right(token), repeat, Some(trailing_sep)),
        [Rule::Token(token), Rule::Rep(repeat)] => (Either::Right(token), repeat, None),
        _ => return false,
    };
    // let repeat = match &**repeat {
    //     Rule::Seq(it) => it,
    //     _ => return false,
    // };
    // if !matches!(
    //     repeat.as_slice(),
    //     [comma, nt_]
    //         if trailing_sep.is_none_or(|it| comma == &**it) && match (nt, nt_) {
    //             (Either::Left(node), Rule::Node(nt_)) => node == nt_,
    //             (Either::Right(token), Rule::Token(nt_)) => token == nt_,
    //             _ => false,
    //         }
    // ) {
    //     return false;
    // }
    match nt {
        Either::Right(token) => {
            let name = clean_token_name(&grammar[*token].name);
            let field = Field::Token(name);
            acc.push(field);
        },
        Either::Left(node) => {
            let ty = grammar[*node].name.clone();
            let name = label
                .cloned()
                .unwrap_or_else(|| pluralize(&to_lower_snake_case(&ty)));
            let field = Field::Node {
                name,
                ty,
                cardinality: Cardinality::Many,
            };
            acc.push(field);
        },
    }
    true
}

fn deduplicate_fields(ast: &mut AstSrc) {
    for node in &mut ast.nodes {
        let mut i = 0;
        'outer: while i < node.fields.len() {
            for j in 0..i {
                let f1 = &node.fields[i];
                let f2 = &node.fields[j];
                if f1 == f2 {
                    node.fields.remove(i);
                    continue 'outer;
                }
            }
            i += 1;
        }
    }
}

fn extract_enums(ast: &mut AstSrc) {
    for node in &mut ast.nodes {
        for enm in &ast.enums {
            let mut to_remove = Vec::new();
            for (i, field) in node.fields.iter().enumerate() {
                let ty = field.ty().to_string();
                if enm.variants.iter().any(|it| it == &ty) {
                    to_remove.push(i);
                }
            }
            if to_remove.len() == enm.variants.len() {
                node.remove_field(to_remove);
                let ty = enm.name.clone();
                let name = to_lower_snake_case(&ty);
                node.fields.push(Field::Node {
                    name,
                    ty,
                    cardinality: Cardinality::Optional,
                });
            }
        }
    }
}

// TODO
const TRAITS: &[(&str, &[&str])] = &[
    // ("HasAttrs", &["attrs"]),
    // ("HasName", &["name"]),
    // ("HasVisibility", &["visibility"]),
    // ("HasGenericParams", &["generic_param_list", "where_clause"]),
    // ("HasTypeBounds", &["type_bound_list", "colon_token"]),
    // ("HasModuleItem", &["items"]),
    // ("HasLoopBody", &["label", "loop_body"]),
    // ("HasArgList", &["arg_list"]),
];

fn extract_struct_traits(ast: &mut AstSrc) {
    for node in &mut ast.nodes {
        for (name, methods) in TRAITS {
            extract_struct_trait(node, name, methods);
        }
    }

    // TODO
    let nodes_with_doc_comments = [];

    for node in &mut ast.nodes {
        if nodes_with_doc_comments.contains(&&*node.name) {
            node.traits.push("HasDocComments".into());
        }
    }
}

fn extract_struct_trait(node: &mut AstNodeSrc, trait_name: &str, methods: &[&str]) {
    let mut to_remove = Vec::new();
    for (i, field) in node.fields.iter().enumerate() {
        let method_name = field.method_name();
        if methods.iter().any(|&it| it == method_name) {
            to_remove.push(i);
        }
    }
    if to_remove.len() == methods.len() {
        node.traits.push(trait_name.to_owned());
        node.remove_field(to_remove);
    }
}

fn extract_enum_traits(ast: &mut AstSrc) {
    for enm in &mut ast.enums {
        if enm.name == "Stmt" {
            continue;
        }
        let nodes = &ast.nodes;
        let mut variant_traits = enm
            .variants
            .iter()
            .map(|var| nodes.iter().find(|it| &it.name == var).unwrap())
            .map(|node| node.traits.iter().cloned().collect::<BTreeSet<_>>());

        let mut enum_traits = match variant_traits.next() {
            Some(it) => it,
            None => continue,
        };
        for traits in variant_traits {
            enum_traits = enum_traits.intersection(&traits).cloned().collect();
        }
        enm.traits = enum_traits.into_iter().collect();
    }
}

impl AstNodeSrc {
    fn remove_field(&mut self, to_remove: Vec<usize>) {
        to_remove.into_iter().rev().for_each(|idx| {
            self.fields.remove(idx);
        });
    }
}

#[test]
fn test() {
    generate(true);
}
