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
