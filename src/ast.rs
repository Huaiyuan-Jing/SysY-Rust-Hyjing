#[derive(Debug)]
pub struct CompUnit {
    pub func_def: FuncDef,
}

#[derive(Debug)]
pub struct FuncDef {
    pub func_type: FuncType,
    pub ident: String,
    pub block: Block,
}

#[derive(Debug)]
pub enum FuncType {
    Int,
}

#[derive(Debug)]
pub struct Block {
    pub stmt: Stmt,
}

#[derive(Debug)]
pub struct Stmt {
    pub exp: Expr,
}

#[derive(Debug)]
pub enum Expr {
    Number(i32),
    UnaryExpr(UnaryOp, Box<Expr>),
    AddExpr(Box<Expr>, AddOp, Box<Expr>),
    MulExpr(Box<Expr>, MulOp, Box<Expr>),
    RelExpr(Box<Expr>, RelOp, Box<Expr>),
    EqExpr(Box<Expr>, EqOp, Box<Expr>),
    LogicExpr(Box<Expr>, LogicOp, Box<Expr>),
}

#[derive(Debug)]
pub enum UnaryOp {
    Plus,
    Minus,
    Not,
}

#[derive(Debug)]
pub enum AddOp {
    Plus,
    Minus,
}

#[derive(Debug)]
pub enum MulOp {
    Multiply,
    Divide,
    Modulo,
}

#[derive(Debug)]
pub enum RelOp {
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
}

#[derive(Debug)]
pub enum EqOp {
    Eq,
    Neq,
}

#[derive(Debug)]
pub enum LogicOp {
    And,
    Or,
}