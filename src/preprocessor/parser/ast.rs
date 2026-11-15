use crate::preprocessor::lexer::Token;

#[derive(Debug, Clone)]
pub enum Expr<'src> {
    Atom(Token<'src>),
    Prefix(Token<'src>, Box<Self>),
    Group(Token<'src>, Box<Self>),
    List(Token<'src>, Vec<Self>),
    CodeBlock(CodeBlock<'src>),
    FnCall(Box<Self>, Token<'src>, Vec<Self>),
}

#[derive(Debug, Clone)]
pub struct Param<'src>(pub Token<'src>, pub Expr<'src>);

#[derive(Debug, Clone)]
pub struct CodeBlock<'src>(pub Vec<Stmt<'src>>, pub Option<Box<Expr<'src>>>);

#[derive(Debug, Clone)]
pub enum Stmt<'src> {
    Expr(Expr<'src>),
    FnDef {
        ftype: Token<'src>,
        name: Token<'src>,
        params: Vec<Param<'src>>,
        body: Option<CodeBlock<'src>>,
    }
}