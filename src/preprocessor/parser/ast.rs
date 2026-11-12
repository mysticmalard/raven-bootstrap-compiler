use crate::preprocessor::lexer::Token;

#[derive(Debug, Clone)]
pub enum Phrase<'src> {
    For(Expr<'src>, Expr<'src>),
    Generic(Token<'src>, Expr<'src>),
    Standalone(Token<'src>),
    Chain(Box<Self>, Box<Self>),
    Block(Box<Self>, CodeBlock<'src>)
}

#[derive(Debug, Clone)]
pub enum Expr<'src> {
    Atom(Token<'src>),
    Access(Box<Self>, Token<'src>, Token<'src>),
    Prefix(Token<'src>, Box<Self>),
    Postfix(Box<Self>, Token<'src>),
    Binary(Box<Self>, Token<'src>, Box<Self>),
    Group(Token<'src>, Box<Self>),
    PrefixCall(Box<Self>, Box<Self>),
    List(Token<'src>, Vec<Self>),
    Phrase(Box<Phrase<'src>>)
}

#[derive(Debug, Clone)]
pub struct Param<'src>(Token<'src>, Expr<'src>);

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