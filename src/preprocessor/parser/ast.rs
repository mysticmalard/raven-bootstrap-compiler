use crate::preprocessor::lexer::Token;

#[derive(Debug, Clone)]
pub enum Phrase<'src> {
    For(Box<Expr<'src>>, Box<Expr<'src>>),
    Match(Box<Expr<'src>>, Vec<Expr<'src>>),
    Generic(Token<'src>, Box<Expr<'src>>),
    Block(Box<Self>, CodeBlock<'src>),
    Lone(Token<'src>, Option<Box<Self>>),
    Chain(Box<Self>, Box<Self>),
}

#[derive(Debug, Clone)]
pub enum Expr<'src> {
    Atom(Token<'src>),
    Access {
        expr: Box<Self>,
        accessor: Token<'src>,
        modifier: Option<Token<'src>>,
        name: Token<'src>
    },
    Prefix(Token<'src>, Box<Self>),
    Binary(Box<Self>, Token<'src>, Box<Self>),
    Postfix(Box<Self>, Token<'src>),
    Group(Token<'src>, Box<Self>),
    List(Token<'src>, Vec<Self>),
    CodeBlock(CodeBlock<'src>),
    FnCall(Box<Self>, Token<'src>, Vec<Self>),
    Phrase(Phrase<'src>),
    Ctrl(Token<'src>),
    Arm {
        binding: Box<Self>,
        pattern: Option<Box<Self>>,
        condition: Option<Box<Self>>,
        expr: Box<Self>
    },
    VarDecl {
        binding: Box<Self>,
        dtype: Option<Box<Self>>,
        expr: Option<Box<Self>>
    }
}

#[derive(Debug, Clone)]
pub struct Param<'src>(pub Expr<'src>, pub Option<Expr<'src>>);

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