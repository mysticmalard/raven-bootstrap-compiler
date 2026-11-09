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
pub struct ParseError<'src>(ParseErrorKind<'src>, Option<Token<'src>>, usize);

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
    fn emit_end_of_file(&self) -> ParseError<'src> {
        ParseError(ParseErrorKind::ENDOFFILE, None, self.lexer.get_index())
    }
}

impl<'src> Parser<'src> {
    fn peek_token(&mut self) -> PR<'src, Token<'src>> {
        self.lexer.peek().ok_or_else(|| self.emit_end_of_file())
    }

    fn peeking_next_token(&mut self, accept: impl FnOnce(&Token<'src>) -> bool) -> Option<Token<'src>> {
        self.lexer.peeking_next(accept)
    }
}

impl<'src> Parser<'src> {
    fn expect_tokens_or_else<'a>(&'a mut self, tokens: &[Token<'src>]) -> PR<'src, Option<Token<'src>>> {
        self.peek_token()?;
        Ok(self.peeking_next_token(|token| tokens.contains(token)))
    }
}

impl<'src> Parser<'src> {
    fn build_root_stmt(&mut self) -> PR<'src, Stmt<'src>> {
        todo!()
    }

    fn build_fn_decl(&mut self, ftype: Token<'src>) -> PR<'src, Stmt<'src>> {
        todo!()
    }

    fn build_stmt(&mut self) -> PR<'src, Stmt<'src>> {
        match self.expect_tokens_or_else(&[
            Token::API, Token::FN, Token::MACRO, Token::SHADER
        ])? {
            Some(ftype @
                ( Token::API
                | Token::FN
                | Token::MACRO
                | Token::SHADER )
            ) => self.build_fn_decl(ftype),
            None => self.build_root_stmt(),
            _ => unreachable!()
        }
    }
}

impl<'src> Iterator for Parser<'src> {
    type Item = PR<'src, Stmt<'src>>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.errored {
            match self.build_stmt() {
                Err(ParseError(ParseErrorKind::ENDOFFILE, _, _)) => { self.errored = true; None },
                Err(e) => { self.errored = true; Some(Err(e)) }
                Ok(stmt) => Some(Ok(stmt)),
            }
        } else {
            None
        }
    }
}