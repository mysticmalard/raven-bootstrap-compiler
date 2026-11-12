use itertools::{PutBackN, put_back_n};
use logos::Logos;
use token_macros::define_tokens;

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

pub const CTRL_FLOW_GEN: &'static [Token] = &[
    Token::FOR, 
    Token::MATCH,
    Token::ATTEMPT, Token::IF, Token::WITH,
    Token::TRY, Token::WHILE, Token::UNTIL,
];

pub const CTRL_FLOW_KWDS: &'static [Token] = &[
    Token::CONTINUE, Token::RETRY,
    Token::YIELD, Token::RETURN,
    Token::BREAK,

    Token::USE, Token::LET
];

pub const ACCESSORS: &'static [Token] = &[

];

pub const PREFIXES: &'static [Token] = &[

];

pub const INFIXES: &'static [Token] = &[

];

pub const POSTFIXES: &'static [Token] = &[

];

impl<'src> Token<'src> {
    pub fn get_closer(&self) -> Option<Self> {
        OPEN_BRACKETS.iter()
            .zip(CLOSE_BRACKETS)
            .find_map(
                |(
                    open,
                    close
                )|
                { (self == open).then_some(*close) }
            )
    }
}

pub struct Lexer<'src> {
    tokens: PutBackN<logos::Lexer<'src, Token<'src>>>,
    index: usize
}

impl<'src> Lexer<'src> {
    pub fn new(src: &'src str) -> Self {
        Self { tokens: put_back_n(Token::lexer(src) ), index: 0 }
    }
}

impl<'src> Lexer<'src> {
    pub fn peeking_next(&mut self, accept: impl FnOnce(&Token<'src>) -> bool) -> Option<Token<'src>> {
        match self.tokens.next()? {
            Ok(token) => accept(&token).then_some(token).or_else(|| { self.tokens.put_back(Ok(token)); None }),
            _ => unreachable!()
        }
    }

    pub fn forgive(&mut self, token: Token<'src>) {
        self.tokens.put_back(Ok(token));
        self.index -= 1;
    }

    pub fn peek(&mut self) -> Option<Token<'src>> {
        let token = self.next()?;
        self.forgive(token);
        Some(token)
    }

    pub fn get_index(&self) -> usize {
        self.index - 1
    }
}

impl<'src> Iterator for Lexer<'src> {
    type Item = Token<'src>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.tokens.next() {
            Some(Ok(token)) => { self.index += 1; Some(token) },
            Some(Err(())) | None => None
        }
    }
}