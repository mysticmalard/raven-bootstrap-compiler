use crate::preprocessor::lexer::Token;

#[derive(Debug, Clone)]
pub enum Expr<'src> {
    Atom(Token<'src>)
}

#[derive(Debug, Clone)]
pub enum Stmt<'src> {
    Expr(Expr<'src>)
}