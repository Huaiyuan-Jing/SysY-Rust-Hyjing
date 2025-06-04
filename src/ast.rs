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
    pub items: Vec<BlockItem>,
}

#[derive(Debug)]
pub enum BlockItem {
    ConstDecl(Vec<ConstDef>),
    Stmt(Stmt),
}

#[derive(Debug)]
pub struct Stmt {
    pub exp: Expr,
}

#[derive(Debug)]
pub enum Expr {
    Number(i32),
    UnaryExpr(UnaryOp, Box<Expr>),
    BinaryExpr(Box<Expr>, BinaryOp, Box<Expr>),
    LVal(String),
}

#[derive(Debug)]
pub enum BinaryOp {
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
    Eq,
    Neq,
    And,
    Or,
}

#[derive(Debug)]
pub enum UnaryOp {
    Plus,
    Minus,
    Not,
}

#[derive(Debug)]
pub struct ConstDef {
    pub kind: String,
    pub id: String,
    pub value: Expr,
}
