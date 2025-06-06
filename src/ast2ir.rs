use crate::ast::{self, BlockItem, FuncType, Stmt};
use lazy_static::lazy_static;
use std::{collections::HashMap, fmt::format, sync::Mutex};
lazy_static! {
    static ref COUNTER: Mutex<i32> = Mutex::new(-1);
    static ref BLOCK_COUNTER: Mutex<i32> = Mutex::new(-1);
}
enum IdElement {
    Const(i32),
    Var(String),
}
struct IdTable<'a> {
    table: HashMap<String, IdElement>,
    father: Option<&'a IdTable<'a>>,
    offset: i32,
}
impl<'a> IdTable<'a> {
    pub fn new(father: Option<&'a IdTable<'a>>, offset: i32) -> Self {
        IdTable {
            table: HashMap::new(),
            father: father,
            offset: offset,
        }
    }
    pub fn insert(&mut self, k: String, v: IdElement) -> bool {
        self.table
            .insert(format!("{}_{}", k, self.offset), v)
            .is_none()
    }
    pub fn get(&self, k: &String) -> (Option<&IdElement>, String) {
        if self.table.contains_key(&format!("{}_{}", k, self.offset)) {
            (Some(&self.table[&format!("{}_{}", k, self.offset)]), format!("{}_{}", k, self.offset))
        } else if self.father.is_some() {
            self.father.as_ref().unwrap().get(k)
        } else {
            (None, String::new())
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
    let id = {
        let mut counter_guard = BLOCK_COUNTER.lock().unwrap();
        *counter_guard += 1;
        counter_guard.clone()
    };
    let mut table = IdTable::new(None, id);
    out += &block2ir(&ast.func_def.block, &mut table);
    out += "}\n";
    out
}
fn block2ir(block: &ast::Block, id_table: &mut IdTable) -> String {
    println!("Block: {}\n", id_table.offset);
    let mut out = String::new();
    for item in &block.items {
        println!("{:#?}", item);
        match item {
            BlockItem::ConstDecl(clist) => {
                for c in clist {
                    let id = c.id.clone();
                    let val = compute_expr(&c.value, id_table);
                    id_table.insert(id, IdElement::Const(val));
                }
            }
            BlockItem::VarDecl(vlsit) => {
                for v in vlsit {
                    let id = v.id.clone();
                    out += &format!("@{}_{} = alloc {}\n", id, id_table.offset, "i32");
                    if !v.value.is_none() {
                        let tmp = expr2ir(&(v.value.as_ref()).unwrap(), id_table);
                        let pos = if tmp.0 == String::new() {
                            tmp.1.to_string()
                        } else {
                            format!("%{}", tmp.1)
                        };
                        out += &tmp.0;
                        out += &format!("store {}, @{}_{}\n", pos, id, id_table.offset);
                    }
                    id_table.insert(id, IdElement::Var(String::from("i32")));
                }
            }
            BlockItem::Stmt(s) => match s {
                Stmt::Ret(e) => match e {
                    Some(e) => {
                        let tmp = expr2ir(e, id_table);
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
                    let id = id_table.get(id);
                    if id.0.is_none() {
                        panic!("Unable to Find Variable");
                    }
                    match id.0.unwrap() {
                        IdElement::Var(_) => {},
                        _ => panic!("assign to non-variable"),
                    }
                    let id = id.1;
                    let tmp = expr2ir(e, id_table);
                    let pos = if tmp.0 == String::new() {
                        tmp.1.to_string()
                    } else {
                        format!("%{}", tmp.1)
                    };
                    out += &tmp.0;
                    out += &format!("store {}, @{}\n", pos, id);
                }
                Stmt::Expr(e) => match e {
                    Some(e) => {
                        out += &expr2ir(e, id_table).0;
                    }
                    None => {}
                },
                Stmt::Block(b) => {
                    let id = {
                        let mut counter_guard = BLOCK_COUNTER.lock().unwrap();
                        *counter_guard += 1;
                        counter_guard.clone()
                    };
                    let mut table = IdTable::new(Some(id_table), id);
                    out += &block2ir(b, &mut table);
                }
                _ => todo!(),
            },
            _ => unreachable!(),
        }
    }
    out
}
fn compute_expr(expr: &ast::Expr, id_table: &IdTable) -> i32 {
    match expr {
        ast::Expr::Number(n) => *n,
        ast::Expr::UnaryExpr(op, expr) => {
            let out = compute_expr(expr.as_ref(), id_table);
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
            let lhs_val = compute_expr(lhs.as_ref(), id_table);
            let rhs_val = compute_expr(rhs.as_ref(), id_table);
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
            let out = id_table.get(lval);
            if out.0.is_none() {
                panic!("Unable to Find Value")
            }
            match out.0.unwrap() {
                IdElement::Var(_) => panic!("Unable to calculate"),
                IdElement::Const(c) => *c,
            }
        }
    }
}
fn expr2ir(exp: &ast::Expr, id_table: &IdTable) -> (String, i32) {
    // println!("{:#?}", exp);
    match exp {
        ast::Expr::Number(n) => (String::new(), *n),
        ast::Expr::UnaryExpr(op, expr) => {
            let out = expr2ir(expr, id_table);
            let mut counter = COUNTER.lock().unwrap();
            *counter += 1;
            let counter = counter.clone();
            let pos = if out.0 == "".to_string() {
                out.1.to_string()
            } else {
                format!("%{}", out.1)
            };
            match op {
                ast::UnaryOp::Not => (format!("{}%{} = eq 0, {}\n", out.0, counter, pos), counter),
                ast::UnaryOp::Minus => {
                    (format!("{}%{} = sub 0, {}\n", out.0, counter, pos), counter)
                }
                ast::UnaryOp::Plus => (out.0, out.1),
            }
        }
        ast::Expr::BinaryExpr(lhs, op, rhs) => {
            let lout = expr2ir(lhs, id_table);
            let lpos = if lout.0 == String::new() {
                lout.1.to_string()
            } else {
                format!("%{}", lout.1)
            };
            let rout = expr2ir(rhs, id_table);
            let rpos = if rout.0 == String::new() {
                rout.1.to_string()
            } else {
                format!("%{}", rout.1)
            };
            let mut out = format!("{}{}", lout.0, rout.0);
            let mut counter = COUNTER.lock().unwrap();
            *counter += 1;
            let counter = counter.clone();
            match op {
                ast::BinaryOp::Plus => {
                    out += &format!("%{} = add {}, {}\n", counter, lpos, rpos);
                    (out, counter)
                }
                ast::BinaryOp::Minus => {
                    out += &format!("%{} = sub {}, {}\n", counter, lpos, rpos);
                    (out, counter)
                }
                ast::BinaryOp::Multiply => {
                    out += &format!("%{} = mul {}, {}\n", counter, lpos, rpos);
                    (out, counter)
                }
                ast::BinaryOp::Divide => {
                    out += &format!("%{} = div {}, {}\n", counter, lpos, rpos);
                    (out, counter)
                }
                ast::BinaryOp::Modulo => {
                    out += &format!("%{} = mod {}, {}\n", counter, lpos, rpos);
                    (out, counter)
                }
                ast::BinaryOp::Eq => {
                    out += &format!("%{} = eq {}, {}\n", counter, lpos, rpos);
                    (out, counter)
                }
                ast::BinaryOp::Neq => {
                    out += &format!("%{} = ne {}, {}\n", counter, lpos, rpos);
                    (out, counter)
                }
                ast::BinaryOp::Less => {
                    out += &format!("%{} = lt {}, {}\n", counter, lpos, rpos);
                    (out, counter)
                }
                ast::BinaryOp::LessOrEqual => {
                    out += &format!("%{} = le {}, {}\n", counter, lpos, rpos);
                    (out, counter)
                }
                ast::BinaryOp::Greater => {
                    out += &format!("%{} = lt {}, {}\n", counter, rpos, lpos);
                    (out, counter)
                }
                ast::BinaryOp::GreaterOrEqual => {
                    out += &format!("%{} = le {}, {}\n", counter, rpos, lpos);
                    (out, counter)
                }
                ast::BinaryOp::And => {
                    out += &format!("%{} = ne {}, {}\n", counter, 0, lpos);
                    let mut counter = COUNTER.lock().unwrap();
                    let lpos = format!("%{}", counter);
                    *counter += 1;
                    out += &format!("%{} = ne {}, {}\n", counter, 0, rpos);
                    let rpos = format!("%{}", counter);
                    *counter += 1;
                    out += &format!("%{} = and {}, {}\n", counter, lpos, rpos);
                    (out, counter.clone())
                }
                ast::BinaryOp::Or => {
                    out += &format!("%{} = ne {}, {}\n", counter, 0, lpos);
                    let mut counter = COUNTER.lock().unwrap();
                    let lpos = format!("%{}", counter);
                    *counter += 1;
                    out += &format!("%{} = ne {}, {}\n", counter, 0, rpos);
                    let rpos = format!("%{}", counter);
                    *counter += 1;
                    out += &format!("%{} = or {}, {}\n", counter, lpos, rpos);
                    (out, counter.clone())
                } // _ => unreachable!(),
            }
        }
        ast::Expr::LVal(lval) => {
            let element = id_table.get(lval);
            if element.0.is_none() {
                panic!("Unable to Find Value")
            }
            match element.0.unwrap() {
                IdElement::Const(val) => (String::new(), *val),
                IdElement::Var(_) => {
                    let mut counter = COUNTER.lock().unwrap();
                    *counter += 1;
                    let out = format!("%{} = load @{}\n", *counter, element.1);
                    (out, *counter)
                }
            }
        } // _ => unreachable!(),
    }
}
