#[derive(Debug)]
pub struct CompUnit {
    pub func_defs: Vec<FuncDef>,
}

#[derive(Debug)]
pub struct FuncDef {
    pub func_type: FuncType,
    pub ident: String,
    pub block: Block,
    pub params: Vec<FuncParam>,
}

#[derive(Debug)]
pub struct FuncParam {
    pub ident: String,
    pub kind: String,
}

#[derive(Debug)]
pub enum FuncType {
    Int,
    Void,
}

#[derive(Debug)]
pub struct Block {
    pub items: Vec<BlockItem>,
}

#[derive(Debug)]
pub enum BlockItem {
    ConstDecl(Vec<ConstDef>),
    VarDecl(Vec<VarDef>),
    Stmt(Stmt),
}

#[derive(Debug)]
pub enum Stmt {
    Ret(Option<Expr>),
    Assign(String, Expr),
    Block(Box<Block>),
    Expr(Option<Expr>),
    IfElse(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
    Break,
    Continue,
}

#[derive(Debug)]
pub enum Expr {
    Number(i32),
    UnaryExpr(UnaryOp, Box<Expr>),
    BinaryExpr(Box<Expr>, BinaryOp, Box<Expr>),
    LVal(String),
    Func(String, Vec<Expr>),
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

#[derive(Debug)]
pub struct VarDef {
    pub kind: String,
    pub id: String,
    pub value: Option<Expr>,
}

