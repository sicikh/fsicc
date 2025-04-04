//! Generated by `cargo codegen grammar`, do not edit by hand.

#![allow(bad_style, missing_docs, unreachable_pub)]
/// The kind of syntax node, e.g. `IDENT`, `VOID_KW`, or `EXPR`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u16)]
pub enum SyntaxKind {
    #[doc(hidden)]
    TOMBSTONE,
    #[doc(hidden)]
    EOF,
    SEMICOLON,
    COMMA,
    L_PAREN,
    R_PAREN,
    L_CURLY,
    R_CURLY,
    L_BRACKET,
    R_BRACKET,
    PIPE,
    STAR,
    UNDERSCORE,
    DOT,
    COLON,
    COLON2,
    EQ,
    ARROW,
    ALIAS_KW,
    AND_KW,
    AS_KW,
    ATTRIBUTE_KW,
    CLASS_KW,
    ELSE_KW,
    FALSE_KW,
    IF_KW,
    IMPORT_KW,
    IN_KW,
    LET_KW,
    MATCH_KW,
    MODULE_KW,
    REC_KW,
    THEN_KW,
    TRUE_KW,
    UNION_KW,
    VALUE_KW,
    WHEN_KW,
    WHERE_KW,
    WITH_KW,
    CHAR,
    FLOAT_NUMBER,
    F_STRING,
    INT_NUMBER,
    STRING,
    ATTRIBUTE,
    COMMENT,
    ERROR,
    IDENT,
    NEWLINE,
    WHITESPACE,
    ADT,
    ADT_LIST,
    ALIAS,
    APP_EXPR,
    ASC_EXPR,
    ASC_PAT,
    ATTR,
    ATTR_DEF,
    BIN_EXPR,
    CLASS,
    CONSTRAINT,
    CONSTRAINT_LIST,
    CONSTRAINT_TYPE,
    CONS_PAT,
    EXPR,
    FN_TYPE,
    IDENT_PAT,
    IF_EXPR,
    IMPORT,
    INFER_TYPE,
    ITEM,
    LET_DECL,
    LET_EXPR,
    LET_OP,
    LITERAL,
    LITERAL_PAT,
    MATCH_CASE,
    MATCH_EXPR,
    MATCH_GUARD,
    MODULE,
    NAME,
    OPERATOR,
    OR_PAT,
    PARAM,
    PARAM_LIST,
    PAREN_EXPR,
    PAREN_PAT,
    PAREN_TYPE,
    PAT,
    PATH,
    PATH_SEGMENT,
    PATH_TYPE,
    PREAMBLE,
    RECORD_PAT,
    RECORD_PAT_FIELD,
    SEQ_EXPR,
    SOURCE_FILE,
    TUPLE_EXPR,
    TUPLE_PAT,
    TUPLE_TYPE,
    TUPLE_VALUE_PAT,
    TYPE,
    TYPE_ARG_LIST,
    TYPE_VAR,
    UNARY_EXPR,
    UNION,
    VALUE,
    VALUE_FIELD,
    WHERE,
    WILDCARD_PAT,
    #[doc(hidden)]
    __LAST,
}
use self::SyntaxKind::*;
impl SyntaxKind {
    pub fn is_keyword(self) -> bool {
        matches!(
            self,
            ALIAS_KW
                | AND_KW
                | AS_KW
                | ATTRIBUTE_KW
                | CLASS_KW
                | ELSE_KW
                | FALSE_KW
                | IF_KW
                | IMPORT_KW
                | IN_KW
                | LET_KW
                | MATCH_KW
                | MODULE_KW
                | REC_KW
                | THEN_KW
                | TRUE_KW
                | UNION_KW
                | VALUE_KW
                | WHEN_KW
                | WHERE_KW
                | WITH_KW
        )
    }
    pub fn is_punct(self) -> bool {
        matches!(
            self,
            SEMICOLON
                | COMMA
                | L_PAREN
                | R_PAREN
                | L_CURLY
                | R_CURLY
                | L_BRACKET
                | R_BRACKET
                | PIPE
                | STAR
                | UNDERSCORE
                | DOT
                | COLON
                | COLON2
                | EQ
                | ARROW
        )
    }
    pub fn is_literal(self) -> bool {
        matches!(self, CHAR | FLOAT_NUMBER | F_STRING | INT_NUMBER | STRING)
    }
    pub fn from_keyword(ident: &str) -> Option<SyntaxKind> {
        let kw = match ident {
            "alias" => ALIAS_KW,
            "and" => AND_KW,
            "as" => AS_KW,
            "attribute" => ATTRIBUTE_KW,
            "class" => CLASS_KW,
            "else" => ELSE_KW,
            "false" => FALSE_KW,
            "if" => IF_KW,
            "import" => IMPORT_KW,
            "in" => IN_KW,
            "let" => LET_KW,
            "match" => MATCH_KW,
            "module" => MODULE_KW,
            "rec" => REC_KW,
            "then" => THEN_KW,
            "true" => TRUE_KW,
            "union" => UNION_KW,
            "value" => VALUE_KW,
            "when" => WHEN_KW,
            "where" => WHERE_KW,
            "with" => WITH_KW,
            _ => return None,
        };
        Some(kw)
    }
    pub fn from_char(c: char) -> Option<SyntaxKind> {
        let tok = match c {
            ';' => SEMICOLON,
            ',' => COMMA,
            '(' => L_PAREN,
            ')' => R_PAREN,
            '{' => L_CURLY,
            '}' => R_CURLY,
            '[' => L_BRACKET,
            ']' => R_BRACKET,
            '|' => PIPE,
            '*' => STAR,
            '_' => UNDERSCORE,
            '.' => DOT,
            ':' => COLON,
            '=' => EQ,
            _ => return None,
        };
        Some(tok)
    }
}
#[macro_export]
macro_rules ! T { [;] => { $ crate :: SyntaxKind :: SEMICOLON } ; [,] => { $ crate :: SyntaxKind :: COMMA } ; ['('] => { $ crate :: SyntaxKind :: L_PAREN } ; [')'] => { $ crate :: SyntaxKind :: R_PAREN } ; ['{'] => { $ crate :: SyntaxKind :: L_CURLY } ; ['}'] => { $ crate :: SyntaxKind :: R_CURLY } ; ['['] => { $ crate :: SyntaxKind :: L_BRACKET } ; [']'] => { $ crate :: SyntaxKind :: R_BRACKET } ; [|] => { $ crate :: SyntaxKind :: PIPE } ; [*] => { $ crate :: SyntaxKind :: STAR } ; [_] => { $ crate :: SyntaxKind :: UNDERSCORE } ; [.] => { $ crate :: SyntaxKind :: DOT } ; [:] => { $ crate :: SyntaxKind :: COLON } ; [::] => { $ crate :: SyntaxKind :: COLON2 } ; [=] => { $ crate :: SyntaxKind :: EQ } ; [->] => { $ crate :: SyntaxKind :: ARROW } ; [alias] => { $ crate :: SyntaxKind :: ALIAS_KW } ; [and] => { $ crate :: SyntaxKind :: AND_KW } ; [as] => { $ crate :: SyntaxKind :: AS_KW } ; [attribute] => { $ crate :: SyntaxKind :: ATTRIBUTE_KW } ; [class] => { $ crate :: SyntaxKind :: CLASS_KW } ; [else] => { $ crate :: SyntaxKind :: ELSE_KW } ; [false] => { $ crate :: SyntaxKind :: FALSE_KW } ; [if] => { $ crate :: SyntaxKind :: IF_KW } ; [import] => { $ crate :: SyntaxKind :: IMPORT_KW } ; [in] => { $ crate :: SyntaxKind :: IN_KW } ; [let] => { $ crate :: SyntaxKind :: LET_KW } ; [match] => { $ crate :: SyntaxKind :: MATCH_KW } ; [module] => { $ crate :: SyntaxKind :: MODULE_KW } ; [rec] => { $ crate :: SyntaxKind :: REC_KW } ; [then] => { $ crate :: SyntaxKind :: THEN_KW } ; [true] => { $ crate :: SyntaxKind :: TRUE_KW } ; [union] => { $ crate :: SyntaxKind :: UNION_KW } ; [value] => { $ crate :: SyntaxKind :: VALUE_KW } ; [when] => { $ crate :: SyntaxKind :: WHEN_KW } ; [where] => { $ crate :: SyntaxKind :: WHERE_KW } ; [with] => { $ crate :: SyntaxKind :: WITH_KW } ; [ident] => { $ crate :: SyntaxKind :: IDENT } ; }
