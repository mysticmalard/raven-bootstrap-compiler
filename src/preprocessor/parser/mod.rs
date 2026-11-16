use crate::preprocessor::lexer::{ACCESS_MODIFIERS, ACCESSORS, CLOSE_BRACKETS, CTRL_FLOW_GEN, CTRL_FLOW_KWDS, CTRL_FLOW_LONE, INFIXES, Lexer, OPEN_BRACKETS, POSTFIXES, PREFIXES, Token};

pub mod ast;
use ast::*;

#[derive(Debug, Clone)]
enum ParseErrorKind<'src> {
    ENDOFFILE,
    EXPECTEDIDENT,
    EXPECTEDATOM,
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

    fn expected_atom_error(&self, token: Token<'src>) -> ParseError<'src> {
        self.emit_error(ParseErrorKind::EXPECTEDATOM, Some(token))
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

    fn expect_atoms_or_else(&mut self, tokens: &[Token<'src>]) -> PR<'src, Option<Token<'src>>> {
        Ok(match self.expect_tokens_or_else(tokens)? {
            token @ Some(_) => token,
            None => self.peeking_next_token(|token| token.is_atom())
        })
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

        if check_start.then(|| cont_cond(self)).transpose()?.unwrap_or(true) {
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
    fn build_call_args(&mut self, closer: Token<'src>) -> PR<'src, Vec<Expr<'src>>> {
        let mut args = Vec::new();
        if !self.expect_tokens_consume(closer, &[])? {
            self.loop_over(
                |me| me.continue_list(Token::COMMA, closer),
                false,
                &mut args,
                |me, args| {
                    // let expr = me.build_expr(&[Token::COMMA, closer])?;
                    // dbg!(&expr);
                    // args.push(expr);
                    args.push(me.build_expr(&[Token::COMMA, closer])?);
                    Ok(())
                }
            )?;
        }
        Ok(args)
    }

    fn build_set(&mut self) -> PR<'src, Expr<'src>> {
        let closers = &[Token::COMMA, Token::SEMI, Token::RBRACE];
        let expr = self.build_expr(closers)?;

        let expr = match self.expect_tokens(closers)? {
            Token::SEMI => {
                let CodeBlock(mut stmts1, exp) = self.build_code_block()?;
                let mut stmts2 = Vec::with_capacity(stmts1.capacity() + 1);
                stmts2.push(Stmt::Expr(expr));
                stmts2.append(&mut stmts1);
                Expr::CodeBlock(CodeBlock(stmts2, exp))
            },
            Token::RBRACE => Expr::CodeBlock(CodeBlock(Vec::new(), Some(Box::new(expr)))),
            Token::COMMA => {
                let mut args = self.build_call_args(Token::RBRACE)?;
                let mut exprs = Vec::with_capacity(args.capacity() + 1);
                exprs.push(expr);
                exprs.append(&mut args);
                Expr::List(Token::RBRACE, exprs)
            },
            _ => unreachable!()
        };

        Ok(expr)
    }

    fn build_group_or_list(&mut self) -> PR<'src, Expr<'src>> {
        let list = self.build_call_args(Token::RPAREN)?;

        Ok(if list.len() == 1 {
            // * There is some unsafe wizardry that can be done to
            // * convert the Vec directly into a Box without cloning
            // * the expression
            Expr::Group(Token::LPAREN, Box::new(list[0].clone()))
        } else {
            Expr::List(Token::LPAREN, list)
        })
    }

    fn build_match_arm(&mut self) -> PR<'src, Expr<'src>> {
        let binding = Box::new(self.build_expr(&[Token::AT, Token::IF, Token::BARW])?);

        let pattern = self.expect_tokens_consume(Token::AT, &[Token::IF, Token::BARW])?
            .then(|| self.build_expr(&[Token::IF, Token::BARW]))
            .transpose()?
            .map(|expr| Box::new(expr));

        let condition = self.expect_tokens_consume(Token::IF, &[Token::BARW])?
            .then(|| self.build_expr(&[Token::BARW]))
            .transpose()?
            .map(|expr| Box::new(expr));

        self.expect_token(Token::BARW)?;
        let expr = Box::new(self.build_expr(&[Token::COMMA, Token::RBRACE])?);

        Ok(Expr::Arm { binding, pattern, condition, expr })
    }

    fn build_match_arms(&mut self) -> PR<'src, Vec<Expr<'src>>> {
        let mut arms = Vec::new();

        self.loop_over(
            |me| me.continue_list(Token::COMMA, Token::RBRACE),
            false,
            &mut arms,
            |me, arms| {
                arms.push(me.build_match_arm()?);
                Ok(())
            }
        )?;

        Ok(arms)
    }

    fn build_phrase(&mut self, kwd: Token<'src>) -> PR<'src, Phrase<'src>> {
        Ok(match kwd {
            Token::FOR => {
                let binding = self.build_expr(&[Token::IN])?;
                self.expect_token(Token::IN)?;
                let expr = self.build_expr(&[Token::LBRACE])?;
                Phrase::For(Box::new(binding), Box::new(expr))
            },
            Token::MATCH => {
                let expr = self.build_expr(&[Token::LBRACE])?;
                self.expect_token(Token::LBRACE)?;
                // TODO
                let arms = self.build_match_arms()?;
                Phrase::Match(Box::new(expr), arms)
            }
            kwd if CTRL_FLOW_GEN.contains(&kwd) => {
                Phrase::Generic(kwd, Box::new(self.build_expr(&[Token::LBRACE])?))
            }
            _ => unreachable!()
        })
    }

    fn build_clause(&mut self, kwd: Token<'src>) -> PR<'src, Phrase<'src>> {
        let generic = self.build_phrase(kwd)?;
        self.expect_token(Token::LBRACE)?;
        let mut block = Phrase::Block(Box::new(generic), self.build_code_block()?);

        loop {
            block = if self.peek_token().is_ok() && let Some(lone) = self.expect_tokens_or_else(CTRL_FLOW_LONE)? {
                Phrase::Chain(Box::new(block), Box::new(if self.expect_tokens_consume(Token::LBRACE, CTRL_FLOW_GEN)? {
                    Phrase::Block(Box::new(Phrase::Lone(lone, None)), self.build_code_block()?)
                } else {
                    let kwd = self.next_token()?;
                    Phrase::Lone(lone, Some(Box::new(self.build_clause(kwd)?)))
                }))
            } else {
                break Ok(block);
            }
        }
    }

    fn build_expr_leaf(&mut self) -> PR<'src, Expr<'src>> {
        Ok(match self.expect_atoms_or_else(&[OPEN_BRACKETS, CTRL_FLOW_GEN, CTRL_FLOW_KWDS].concat())? {
            Some(atom @
                ( Token::Ident(_)
                | Token::Char(_)
                | Token::Literal(_)
                | Token::Number(_))) => Expr::Atom(atom),
            Some(bracket @ (Token::LANGLE | Token::LBRACKET)) => {
                Expr::List(bracket, self.build_call_args(bracket.get_closer().unwrap())?)
            },
            Some(Token::LBRACE) => self.build_set()?,
            Some(Token::LPAREN) => self.build_group_or_list()?,
            Some(kwd) if CTRL_FLOW_GEN.contains(&kwd) => Expr::Phrase(self.build_clause(kwd)?),
            Some(kwd) if CTRL_FLOW_KWDS.contains(&kwd) => Expr::Ctrl(kwd),
            None => todo!(),
            _ => unreachable!()
        })
    }

    fn append_postfix(&mut self, expr: Box<Expr<'src>>) -> PR<'src, Expr<'src>> {
        Ok(match self.expect_tokens(&[
            OPEN_BRACKETS, INFIXES, ACCESSORS, POSTFIXES
        ].concat())? {
            bracket if OPEN_BRACKETS.contains(&bracket) => Expr::FnCall(
                expr,
                bracket,
                self.build_call_args(bracket.get_closer().unwrap())?
            ),
            token if [INFIXES, POSTFIXES].concat().contains(&token) => Expr::Postfix(expr, token),
            accessor if ACCESSORS.contains(&accessor) => {
                let modifier = self.expect_tokens_or_else(ACCESS_MODIFIERS)?;
                let name = self.expect_ident()?;
                Expr::Access { expr, accessor, modifier, name }
            }
            _ => unreachable!()
        })
    }

    fn build_expr(&mut self, closers: &[Token<'src>]) -> PR<'src, Expr<'src>> {
        let mut expr = Box::new(match self.expect_tokens_or_else(PREFIXES)? {
            Some(prefix) => Expr::Prefix(prefix, Box::new(self.build_expr(closers)?)),
            None => self.build_expr_leaf()?,
            // _ => unreachable!()
        });

        self.loop_over(
            |me| Ok(!me.check_for_tokens(closers)?),
            true,
            &mut expr,
            |me, expr| {
                // let exp = match expr.clone() {
                **expr = match *expr.clone() {
                    Expr::Postfix(exp, infix) if INFIXES.contains(&infix) => {
                        Expr::Binary(exp, infix, Box::new(me.build_expr(closers)?))
                    },
                    exp => me.append_postfix(Box::new(exp))?
                };
                // **expr = exp;
                Ok(())
            }
        )?;

        if let Expr::Postfix(_, ref token) = *expr && !POSTFIXES.contains(token) {
            return Err( self.expected_tokens_error(POSTFIXES, *token))
        }

        Ok(*expr)
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

    fn build_param(&mut self, closer: Token<'src>) -> PR<'src, Param<'src>> {
        let binding = self.build_expr(&[Token::COMMA, Token::COLON, closer])?;

        let dtype = self.expect_tokens_consume(Token::COLON, &[Token::COMMA, closer])?
            .then(|| self.build_expr(&[Token::COMMA, closer]))
            .transpose()?;

        Ok(Param(binding, dtype))
    }

    fn build_params(&mut self, closer: Token<'src>) -> PR<'src, Vec<Param<'src>>> {
        let mut params = Vec::new();

        if !self.expect_tokens_consume(closer, &[])? {
            self.loop_over(
                |me| me.continue_list(Token::COMMA, closer),
                false,
                &mut params,
                |me, params| {
                    params.push(me.build_param(closer)?);
                    Ok(())
                }
            )?;
        }
        Ok(params)
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
            Some(kwd) if CTRL_FLOW_GEN.contains(&kwd) => Ok(Stmt::Expr(Expr::Phrase(self.build_clause(kwd)?))),
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