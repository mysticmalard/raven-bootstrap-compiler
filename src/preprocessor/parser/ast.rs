use crate::preprocessor::lexer::Token;

#[derive(Debug, Clone)]
pub enum Expr<'src> {
    Atom(Token<'src>)
}

#[derive(Debug, Clone)]
pub struct Param<'src>(Token<'src>, Expr<'src>);

#[derive(Debug, Clone)]
pub struct CodeBlock<'src>(Vec<Stmt<'src>>, Option<Box<Expr<'src>>>);

#[derive(Debug, Clone)]
pub enum Stmt<'src> {
    Expr(Expr<'src>),
    FnDef {
        name: Token<'src>,
        params: Vec<Param<'src>>,
        body: Option<CodeBlock<'src>>,
    }
}