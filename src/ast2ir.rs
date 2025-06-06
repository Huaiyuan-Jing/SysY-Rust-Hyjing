use crate::ast::{self, BlockItem, FuncType, Stmt};
use lazy_static::lazy_static;
use std::{collections::HashMap, sync::Mutex};
lazy_static! {
    static ref COUNTER: Mutex<i32> = Mutex::new(-1);
}
enum IdElement {
    Const(i32),
    Var(String),
}
struct IdTable {
    table:HashMap<String, IdElement>,
    father: Option<Box<IpTable>>, 
}
impl IdTable {
    pub fn new(father: Option<Box<IpTable>>) -> Self {
        IdTable {
            table: HashMap::new(),
            father: father,
        }
    }
    pub fn insert(k: String, v: IdElement) -> bool {
        self.table.insert(k, v).is_none()
    }
    pub fn get(k: &String) -> Option<&IdElement> {
        if self.table.contains_key(k) {
            Some(&self.table[k])
        } else if self.father.is_some() {
            self.father.as_ref().unwrap().get(k)
        } else {
            None
        }
    }
}
pub fn ast2ir(ast: &ast::CompUnit) -> String {
    let func_type = match ast.func_def.func_type {
        FuncType::Int => "i32",
    };
    let mut out = format!(
        "fun @{}(): {} {{\n%entry:\n",
        &ast.func_def.ident, func_type
    );
    out += &block2ir(&ast.func_def.block, &IdTable::new(None));
    out += "}\n";
    out
}
fn block2ir(block: &ast::Block, id_table: &IdTable) -> String {
    let mut out = String::new();
    for item in &block.items {
        match item {
            BlockItem::ConstDecl(clist) => {
                for c in clist {
                    let id = c.id.clone();
                    let val = compute_expr(&c.value);
                    id_table.insert(id, IdElement::Const(val));
                }
            }
            BlockItem::VarDecl(vlsit) => {
                for v in vlsit {
                    let id = v.id.clone();
                    out += &format!("@{} = alloc {}\n", id, "i32");
                    if !v.value.is_none() {
                        let tmp = expr2ir(&(v.value.as_ref()).unwrap());
                        let pos = if tmp.0 == String::new() {
                            tmp.1.to_string()
                        } else {
                            format!("%{}", tmp.1)
                        };
                        out += &tmp.0;
                        out += &format!("store {}, @{}\n", pos, id);
                    }
                    id_table.insert(id, IdElement::Var(String::from("i32")));
                }
            }
            BlockItem::Stmt(s) => match s {
                Stmt::Ret(e) => match e {
                    Some(e) => {
                        let tmp = expr2ir(e);
                        let pos = if tmp.0 == String::new() {
                            tmp.1.to_string()
                        } else {
                            format!("%{}", tmp.1)
                        };
                        out += &tmp.0;
                        out += &format!("ret {}\n", pos);
                    }
                    None => {
                        out += "ret\n";
                    }
                },
                Stmt::Assign(id, e) => {
                    let tmp = expr2ir(e);
                    let pos = if tmp.0 == String::new() {
                        tmp.1.to_string()
                    } else {
                        format!("%{}", tmp.1)
                    };
                    out += &tmp.0;
                    out += &format!("store {}, @{}\n", pos, id);
                },
                Stmt::Expr(e) => match e {
                    Some(e) => {
                        out += &expr2ir(exp).0;
                    }
                    None => {}
                }
                Stmt::Block(b) => {
                    out += &block2ir(b, &IdTable::new(Some(id_table)));
                } 
                _ => todo!()
            },
            _ => unreachable!(),
        }
    }
    out
}
fn compute_expr(expr: &ast::Expr) -> i32 {
    match expr {
        ast::Expr::Number(n) => *n,
        ast::Expr::UnaryExpr(op, expr) => {
            let out = compute_expr(expr.as_ref());
            match op {
                ast::UnaryOp::Not => {
                    if out == 0 {
                        1
                    } else {
                        0
                    }
                }
                ast::UnaryOp::Plus => out,
                ast::UnaryOp::Minus => -out,
            }
        }
        ast::Expr::BinaryExpr(lhs, op, rhs) => {
            let lhs_val = compute_expr(lhs.as_ref());
            let rhs_val = compute_expr(rhs.as_ref());
            match op {
                ast::BinaryOp::Plus => lhs_val + rhs_val,
                ast::BinaryOp::Minus => lhs_val - rhs_val,
                ast::BinaryOp::Multiply => lhs_val * rhs_val,
                ast::BinaryOp::Divide => lhs_val / rhs_val,
                ast::BinaryOp::Modulo => lhs_val % rhs_val,
                ast::BinaryOp::Less => {
                    if lhs_val < rhs_val {
                        1
                    } else {
                        0
                    }
                }
                ast::BinaryOp::Greater => {
                    if lhs_val > rhs_val {
                        1
                    } else {
                        0
                    }
                }
                ast::BinaryOp::LessOrEqual => {
                    if lhs_val <= rhs_val {
                        1
                    } else {
                        0
                    }
                }
                ast::BinaryOp::GreaterOrEqual => {
                    if lhs_val >= rhs_val {
                        1
                    } else {
                        0
                    }
                }
                ast::BinaryOp::Eq => {
                    if lhs_val == rhs_val {
                        1
                    } else {
                        0
                    }
                }
                ast::BinaryOp::Neq => {
                    if lhs_val != rhs_val {
                        1
                    } else {
                        0
                    }
                }
                ast::BinaryOp::And => {
                    if lhs_val != 0 && rhs_val != 0 {
                        1
                    } else {
                        0
                    }
                }
                ast::BinaryOp::Or => {
                    if lhs_val != 0 || rhs_val != 0 {
                        1
                    } else {
                        0
                    }
                }
            }
        }
        ast::Expr::LVal(lval) => {
            match id_table.get(lval).unwrap() {
                IdElement::Const(v) => *v,
                _ => unreachable!(),
            }
        }
    }
}
fn expr2ir(exp: &ast::Expr) -> (String, i32) {
    match exp {
        ast::Expr::Number(n) => (String::new(), *n),
        ast::Expr::UnaryExpr(op, expr) => {
            let out = expr2ir(expr);
            let mut counter = COUNTER.lock().unwrap();
            let pos = if out.0 == "".to_string() {
                out.1.to_string()
            } else {
                format!("%{}", out.1)
            };
            match op {
                ast::UnaryOp::Not => {
                    *counter += 1;
                    (
                        format!("{}%{} = eq 0, {}\n", out.0, *counter, pos),
                        *counter,
                    )
                }
                ast::UnaryOp::Minus => {
                    *counter += 1;
                    (
                        format!("{}%{} = sub 0, {}\n", out.0, *counter, pos),
                        *counter,
                    )
                }
                ast::UnaryOp::Plus => (out.0, out.1),
            }
        }
        ast::Expr::BinaryExpr(lhs, op, rhs) => {
            let lout = expr2ir(lhs);
            let lpos = if lout.0 == String::new() {
                lout.1.to_string()
            } else {
                format!("%{}", lout.1)
            };
            let rout = expr2ir(rhs);
            let rpos = if rout.0 == String::new() {
                rout.1.to_string()
            } else {
                format!("%{}", rout.1)
            };
            let mut out = format!("{}{}", lout.0, rout.0);
            let mut counter = COUNTER.lock().unwrap();
            *counter += 1;
            match op {
                ast::BinaryOp::Plus => {
                    out += &format!("%{} = add {}, {}\n", counter, lpos, rpos);
                    (out, *counter)
                }
                ast::BinaryOp::Minus => {
                    out += &format!("%{} = sub {}, {}\n", counter, lpos, rpos);
                    (out, *counter)
                }
                ast::BinaryOp::Multiply => {
                    out += &format!("%{} = mul {}, {}\n", counter, lpos, rpos);
                    (out, *counter)
                }
                ast::BinaryOp::Divide => {
                    out += &format!("%{} = div {}, {}\n", counter, lpos, rpos);
                    (out, *counter)
                }
                ast::BinaryOp::Modulo => {
                    out += &format!("%{} = mod {}, {}\n", counter, lpos, rpos);
                    (out, *counter)
                }
                ast::BinaryOp::Eq => {
                    out += &format!("%{} = eq {}, {}\n", counter, lpos, rpos);
                    (out, *counter)
                }
                ast::BinaryOp::Neq => {
                    out += &format!("%{} = ne {}, {}\n", counter, lpos, rpos);
                    (out, *counter)
                }
                ast::BinaryOp::Less => {
                    out += &format!("%{} = lt {}, {}\n", counter, lpos, rpos);
                    (out, *counter)
                }
                ast::BinaryOp::LessOrEqual => {
                    out += &format!("%{} = le {}, {}\n", counter, lpos, rpos);
                    (out, *counter)
                }
                ast::BinaryOp::Greater => {
                    out += &format!("%{} = lt {}, {}\n", counter, rpos, lpos);
                    (out, *counter)
                }
                ast::BinaryOp::GreaterOrEqual => {
                    out += &format!("%{} = le {}, {}\n", counter, rpos, lpos);
                    (out, *counter)
                }
                ast::BinaryOp::And => {
                    out += &format!("%{} = ne {}, {}\n", counter, 0, lpos);
                    let lpos = format!("%{}", *counter);
                    *counter += 1;
                    out += &format!("%{} = ne {}, {}\n", counter, 0, rpos);
                    let rpos = format!("%{}", *counter);
                    *counter += 1;
                    out += &format!("%{} = and {}, {}\n", counter, lpos, rpos);
                    (out, *counter)
                }
                ast::BinaryOp::Or => {
                    out += &format!("%{} = ne {}, {}\n", counter, 0, lpos);
                    let lpos = format!("%{}", *counter);
                    *counter += 1;
                    out += &format!("%{} = ne {}, {}\n", counter, 0, rpos);
                    let rpos = format!("%{}", *counter);
                    *counter += 1;
                    out += &format!("%{} = or {}, {}\n", counter, lpos, rpos);
                    (out, *counter)
                } // _ => unreachable!(),
            }
        }
        ast::Expr::LVal(lval) => {
            let element = id_table.get(lval).unwrap();
            match element {
                IdElement::Const(val) => (String::new(), *val),
                IdElement::Var(_) => {
                    let mut counter = COUNTER.lock().unwrap();
                    *counter += 1;
                    let out = format!("%{} = load @{}\n", *counter, lval);
                    (out, *counter)
                }
            }
        } // _ => unreachable!(),
    }
}
