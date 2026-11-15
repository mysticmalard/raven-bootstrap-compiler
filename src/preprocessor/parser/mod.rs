use crate::preprocessor::lexer::{ACCESSORS, CLOSE_BRACKETS, CTRL_FLOW_GEN, INFIXES, Lexer, OPEN_BRACKETS, POSTFIXES, PREFIXES, Token};

pub mod ast;
use ast::*;

#[derive(Debug, Clone)]
enum ParseErrorKind<'src> {
    ENDOFFILE,
    EXPECTEDIDENT,
    ExpectedToken(Token<'src>),
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
    fn emit_error(&self, kind: ParseErrorKind<'src>, offender: Option<Token<'src>>) -> ParseError<'src> {
        ParseError(kind, offender, self.lexer.get_index())
    }

    fn emit_end_of_file(&self) -> ParseError<'src> {
        self.emit_error(ParseErrorKind::ENDOFFILE, None)
    }

    fn expected_ident_error(&self, token: Token<'src>) -> ParseError<'src> {
        self.emit_error(ParseErrorKind::EXPECTEDIDENT, Some(token))
    }

    fn expected_token_error(&self, expectation: Token<'src>, token: Token<'src>) -> ParseError<'src> {
        self.emit_error(ParseErrorKind::ExpectedToken(expectation), Some(token))
    }

    fn expected_tokens_error(&self, acceptable: &[Token<'src>], token: Token<'src>) -> ParseError<'src> {
        self.emit_error(ParseErrorKind::ExpectedMulti(acceptable.to_vec()), Some(token))
    }
}

impl<'src> Parser<'src> {
    fn next_token(&mut self) -> PR<'src, Token<'src>> {
        self.lexer.next().ok_or_else(|| self.emit_end_of_file())
    }

    fn peek_token(&mut self) -> PR<'src, Token<'src>> {
        self.lexer.peek().ok_or_else(|| self.emit_end_of_file())
    }

    fn peeking_next_token(&mut self, accept: impl FnOnce(&Token<'src>) -> bool) -> Option<Token<'src>> {
        self.lexer.peeking_next(accept)
    }
}

impl<'src> Parser<'src> {
    fn expect_token(&mut self, expectation: Token<'src>) -> PR<'src, ()> {
        let token = self.next_token()?;
        if token == expectation {
            Ok(())
        } else {
            Err( self.expected_token_error(expectation, token) )
        }
    }

    fn expect_tokens(&mut self, acceptable: &[Token<'src>]) -> PR<'src, Token<'src>> {
        let token = self.next_token()?;
        acceptable.contains(&token).then_some(token).ok_or_else(|| self.expected_tokens_error(acceptable, token))
    }

    fn expect_ident(&mut self) -> PR<'src, Token<'src>> {
        let token = self.next_token()?;
        match token {
            t @ Token::Ident(_) => Ok(t),
            t => Err(self.expected_ident_error(t))
        }
    }

    fn expect_tokens_or_else<'a>(&'a mut self, tokens: &[Token<'src>]) -> PR<'src, Option<Token<'src>>> {
        self.peek_token()?;
        Ok(self.peeking_next_token(|token| tokens.contains(token)))
    }

    fn expect_tokens_consume(&mut self, expectation: Token<'src>, acceptable: &[Token<'src>]) -> PR<'src, bool> {
        self.peeking_next_token(|token| *token == expectation)
            .map_or_else(
                || {
                    let token = self.peek_token()?;
                    if acceptable.contains(&token) || acceptable.is_empty() {
                        Ok(false)
                    } else {
                        Err(self.expected_tokens_error(&[&[expectation], acceptable].concat(), token))
                    }
                },
                |_| Ok(true)
            )
    }

    fn check_for_tokens(&mut self, tokens: &[Token<'src>]) -> PR<'src, bool> {
        Ok(tokens.contains(&self.peek_token()?))
    }

    fn continue_list(&mut self, delimiter: Token<'src>, closer: Token<'src>) -> PR<'src, bool> {
        match self.next_token()? {
            token if token == delimiter => Ok(!self.expect_tokens_consume(closer, &[])?),
            token if token == closer => Ok(false),
            token => Err(self.expected_tokens_error(&[delimiter, closer], token))
        }
    }

    // fn parse_list(
    //     &mut self,
    //     delimiter: Option<Token<'src>>,
    //     closer: Token<'src>,
    //     mut foo: impl FnMut(&mut Self) -> PR<'src, ()>
    // ) -> PR<'src, ()> {
    //     let cont_cond = delimiter.map_or_else(
    //         || Box::new(
    //             |me: &mut Self| me.expect_tokens_consume(closer, &[]))
    //                 as Box<dyn Fn(&mut Self) -> PR<'src, bool>>,
    //         |token: Token<'src>| {
    //             Box::new(
    //                 move |me: &mut Self| me.continue_list(token, closer)
    //             ) as Box<dyn Fn(&mut Self) -> PR<'src, bool>>
    //         }
    //     );

    //     // if !self.expect_tokens_consume(closer, &[])? {
    //     if !self.check_for_tokens(&[closer])? {
    //         loop {
    //             foo(self)?;

    //             if !cont_cond(self)? {
    //                 break;
    //             }
    //         }
    //     }
    //     self.expect_token(closer)?;
    //     Ok(())
    // }

    fn loop_over<T>(
        &mut self,
        cont_cond: impl Fn(&mut Self) -> PR<'src, bool>,
        check_start: bool,
        args: &mut T,
        foo: impl Fn(&mut Self, &mut T) -> PR<'src, ()>
    ) -> PR<'src, ()> {
        // while cont_cond(self)? {
        //     foo(self, args)?;
        // }

        if check_start || cont_cond(self)? {
            loop {
                foo(self, args)?;
                if !cont_cond(self)? {
                    break;
                }
            }
        }
        Ok(())
    }
}

impl<'src> Parser<'src> {
    fn build_expr(&mut self, closers: &[Token<'src>]) -> PR<'src, Expr<'src>> {
        Ok(Expr::Atom(self.next_token()?))
    }

    fn build_code_block(&mut self) -> PR<'src, CodeBlock<'src>> {
        let mut stmts = Vec::new();
        let mut expr = None;

        self.loop_over(
            |me| Ok(!me.expect_tokens_consume(Token::RBRACE, &[])?),
            true,
            &mut (&mut stmts, &mut expr),
            |me, (stmts, expr)| {
                match me.build_stmt_or_expr(Some(Token::RBRACE))? {
                    Ok(stmt) => stmts.push(stmt),
                    Err(exp) => { expr.replace(Box::new(exp)); }
                }
                Ok(())
            }
        )?;

        Ok(CodeBlock(stmts, expr))
    }

    fn build_params(&mut self, closer: Token<'src>) -> PR<'src, Vec<Param<'src>>> {
        // TODO
        self.expect_token(closer)?;
        Ok(Vec::new())
    }

    fn build_fn_decl(&mut self, ftype: Token<'src>) -> PR<'src, Stmt<'src>> {
        let name = self.expect_ident()?;

        self.expect_token(Token::LPAREN)?;
        let params = self.build_params(Token::RPAREN)?;

        let body = self.expect_tokens_consume(Token::LBRACE, &[Token::SEMI])?
            .then(|| self.build_code_block())
            .transpose()?;

        Ok(Stmt::FnDef { ftype, name, params, body })
    }

    fn build_stmt_or_expr(&mut self, closer: Option<Token<'src>>) -> PR<'src, Result<Stmt<'src>, Expr<'src>>> {
        let closers = if let Some(closer) = closer {
            (&[Token::SEMI, closer]) as &[Token<'src>]
        } else {
            (&[Token::SEMI]) as &[Token<'src>]
        };

        let stmt_or_expr = match self.expect_tokens_or_else(CTRL_FLOW_GEN)? {
            Some(kwd) if CTRL_FLOW_GEN.contains(&kwd) => todo!(),
            None => {
                let expr = self.build_expr(closers)?;
                if self.expect_tokens_consume(Token::SEMI, &[])? {
                    Ok(Stmt::Expr(expr))
                } else {
                    Err(expr)
                }
            },
            _ => unreachable!()
        };

        Ok(stmt_or_expr)
    }

    fn build_root_stmt(&mut self) -> PR<'src, Stmt<'src>> {
        Ok(match self.build_stmt_or_expr(None)? {
            Ok(stmt) => stmt,
            Err(expr) => { self.expect_token(Token::SEMI)?; Stmt::Expr(expr) }
        })
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
        if !self.errored && self.peek_token().is_ok() {
            match self.build_stmt() {
                Err(e) => { self.errored = true; Some(Err(e)) }
                Ok(stmt) => Some(Ok(stmt)),
            }
        } else {
            None
        }
    }
}