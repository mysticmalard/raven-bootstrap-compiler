use crate::preprocessor::lexer::{Lexer, Token};

pub mod ast;
use ast::*;

#[derive(Debug, Clone)]
enum ParseErrorKind<'src> {
    ENDOFFILE,
    ExpectedIdent(Token<'src>),
    ExpectedMulti(Vec<Token<'src>>)
}

#[derive(Debug, Clone)]
pub struct ParseError<'src>(ParseErrorKind<'src>, Option<Token<'src>>);

pub struct Parser<'src> {
    lexer: Lexer<'src>,
    bracket_stack: Vec<(Token<'src>, usize)>,
    errored: bool,
}

type PR<'src, T> = Result<T, ParseError<'src>>;

impl<'src> Parser<'src> {
    pub fn new(src: &'src str) -> Self {
        Self { lexer: Lexer::new(src), bracket_stack: Vec::new(), errored: false }
    }
}

impl<'src> Parser<'src> {
    fn build_stmt(&mut self) -> PR<'src, Stmt<'src>> {
        todo!()
    }
}

impl<'src> Iterator for Parser<'src> {
    type Item = PR<'src, Stmt<'src>>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.errored {
            match self.build_stmt() {
                Err(ParseError(ParseErrorKind::ENDOFFILE, _)) => { self.errored = true; None },
                Err(e) => { self.errored = true; Some(Err(e)) }
                Ok(stmt) => Some(Ok(stmt)),
            }
        } else {
            None
        }
    }
}