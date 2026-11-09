use itertools::{PutBackN, put_back_n};
use logos::Logos;
use token_macros::define_tokens;

use crate::preprocessor::collector::Collector;

define_tokens! {
    AND, OR, NOT,
    AS, IN,
    FN, MACRO, SHADER, API,
    STRUCT, ENUM, TRAIT,
    IMPL, DYN,

    MUT, PUB, STATIC, CONST,

    IMPORT, USE, MOD,
    LET,

    BREAK, CONTINUE, RETRY,
    RETURN, YIELD,

    FOR,
    MATCH,
    ATTEMPT, IF, WITH, TRY, WHILE, UNTIL,
    ELSE, DO, DEFER, FINALLY, EXCEPT,

    COMMA -> ",",
    COLON -> ":",
    SEMI -> ";",
    DOT -> ".",
    DCOLON -> "::",
    BARW -> "=>",
    MARW -> "-mut>",
    ARW["<-", "->"],

    PAREN["(", ")"],
    BRACKET["[", "]"],
    BRACE["{", "}"],
    ANGLE["<", ">"],

    Char(r"'(\\)?.'"),
    Lifetime(r"'[a-zA-Z_][a-zA-Z_0-9]*\s"),
    Number(r"[0-9]+"),
    Ident(r"[a-zA-Z_][a-zA-Z_0-9]*"),
    Literal(r#"".*?\""#),
    FStringLiteral(r#"f".*?""#),
    Unknown(".")
}

pub const OPEN_BRACKETS: &'static [Token] = &[Token::LANGLE, Token::LPAREN, Token::LBRACE, Token::LBRACKET];
pub const CLOSE_BRACKETS: &'static [Token] = &[Token::RANGLE, Token::RPAREN, Token::RBRACE, Token::RBRACKET];

pub struct Lexer<'src> {
    tokens: PutBackN<logos::Lexer<'src, Token<'src>>>,
}

impl<'src> Lexer<'src> {
    pub fn new(src: &'src Collector) -> Self {
        Self { tokens: put_back_n(Token::lexer(src.get_body()) ) }
    }
}

impl<'src> Iterator for Lexer<'src> {
    type Item = Token<'src>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.tokens.next() {
            Some(Ok(token)) => Some(token),
            Some(Err(())) | None => None
        }
    }
}