use crate::preprocessor::lexer::{Lexer, Token};

pub mod ast;
use ast::*;

#[derive(Debug, Clone)]
enum ParseError<'src> {
    ENDOFFILE,
    ExpectedIdent(Token<'src>),
    ExpectedMulti(Vec<Token<'src>>)
}

pub struct Parser<'src> {
    lexer: Lexer<'src>,
    bracket_stack: Vec<(Token<'src>, usize)>
}

type PR<'src, T> = Result<T, ParseError<'src>>;

impl<'src> Parser<'src> {
    pub fn new(src: &'src str) -> Self {
        Self { lexer: Lexer::new(src), bracket_stack: Vec::new() }
    }
}