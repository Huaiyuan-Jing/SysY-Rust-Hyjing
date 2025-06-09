use crate::ast::*;
use lazy_static::lazy_static;
use std::{collections::HashMap, sync::Mutex};
lazy_static! {
    static ref COUNTER: Mutex<i32> = Mutex::new(-1);
    static ref BLOCK_COUNTER: Mutex<i32> = Mutex::new(-1);
    static ref IF_COUNTER: Mutex<i32> = Mutex::new(-1);
    static ref WHILE_COUNTER: Mutex<i32> = Mutex::new(-1);
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
            (
                Some(&self.table[&format!("{}_{}", k, self.offset)]),
                format!("{}_{}", k, self.offset),
            )
        } else if self.father.is_some() {
            self.father.as_ref().unwrap().get(k)
        } else {
            (None, String::new())
        }
    }
}
pub fn ast2ir(ast: &mut CompUnit) -> String {
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
    let (st, _) = &block2ir(&mut ast.func_def.block, &mut table, -1);
    out += st;
    out += "}\n";
    out
}
fn stmt2ir(stmt: &mut Stmt, id_table: &mut IdTable, cur_while_id: i32) -> (String, bool) {
    let mut out = String::new();
    let mut is_exit = false;
    match stmt {
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
                is_exit = true;
            }
            None => {
                out += "ret\n";
                is_exit = true;
            }
        },
        Stmt::Assign(id, e) => {
            let id = id_table.get(id);
            if id.0.is_none() {
                panic!("Unable to Find Variable");
            }
            match id.0.unwrap() {
                IdElement::Var(_) => {}
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
            let (st, is_exit_block) = &block2ir(b, &mut table, cur_while_id);
            out += st;
            if *is_exit_block {
                is_exit = true;
            }
        }
        Stmt::IfElse(cond, if_then, else_then) => {
            let tmp = expr2ir(cond, id_table);
            out += &tmp.0;
            let cond = if tmp.0 == String::new() {
                tmp.1.to_string()
            } else {
                format!("%{}", tmp.1)
            };
            let id = {
                let mut counter_guard = IF_COUNTER.lock().unwrap();
                *counter_guard += 1;
                counter_guard.clone()
            };
            match else_then {
                None => {
                    out += &format!("br {}, %then_{}, %end_{}\n", cond, id, id);
                    out += &format!("%then_{}:\n", id);
                    let (st, then_is_ret) = &stmt2ir(if_then, id_table, cur_while_id);
                    out += st;
                    if !then_is_ret {
                        out += &format!("jump %end_{}\n", id);
                    }
                    out += &format!("%end_{}:\n", id);
                }
                Some(else_then) => {
                    out += &format!("br {}, %then_{}, %else_{}\n", cond, id, id);
                    out += &format!("%then_{}:\n", id);
                    let (st, then_is_ret) = &stmt2ir(if_then, id_table, cur_while_id);
                    out += st;
                    if !then_is_ret {
                        out += &format!("jump %end_{}\n", id);
                    }
                    out += &format!("%else_{}:\n", id);
                    let (st, else_is_ret) = &stmt2ir(else_then.as_mut(), id_table, cur_while_id);
                    out += st;
                    if !else_is_ret {
                        out += &format!("jump %end_{}\n", id);
                    }
                    out += &format!("%end_{}:\n", id);
                }
            }
        }
        Stmt::While(cond, body) => {
            let while_id = {
                let mut lock = WHILE_COUNTER.lock().unwrap();
                *lock += 1;
                *lock
            };
            out += &format!("jump %while_entry{}\n", while_id);
            out += &format!("%while_entry{}:\n", while_id);
            let tmp = expr2ir(cond, id_table);
            out += &tmp.0;
            let pos = if tmp.0 == "" {
                tmp.1.to_string()
            } else {
                format!("%{}", tmp.1)
            };
            out += &format!(
                "br {}, %while_body{}, %while_end{}\n",
                pos, while_id, while_id
            );
            out += &format!("%while_body{}:\n", while_id);
            let (st, body_is_ret) = &stmt2ir(body, id_table, while_id);
            out += st;
            if !body_is_ret {
                out += &format!("jump %while_entry{}\n", while_id);
            }
            out += &format!("%while_end{}:\n", while_id);
        }
        Stmt::Break => {
            if cur_while_id == -1 {
                panic!("Break outside of while loop");
            }
            out += &format!("jump %while_end{}\n", cur_while_id);
            is_exit = true;
        }
        Stmt::Continue => {
            if cur_while_id == -1 {
                panic!("Continue outside of while loop");
            }
            out += &format!("jump %while_entry{}\n", cur_while_id);
            is_exit = true;
        }
        _ => todo!(),
    }
    (out, is_exit)
}
fn block2ir(block: &mut Block, id_table: &mut IdTable, cur_while_id: i32) -> (String, bool) {
    // println!("Block: {}\n", id_table.offset);
    let mut out = String::new();
    // let mut if_else_stack = Vec::new();
    for item in &mut block.items {
        // println!("{:#?}", item);
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
            BlockItem::Stmt(s) => {
                let (st, is_exit_st) = &stmt2ir(s, id_table, cur_while_id);
                out += st;
                if *is_exit_st {
                    return (out, true);
                }
            }
            _ => unreachable!(),
        }
    }
    (out, false)
}
fn compute_expr(expr: &Expr, id_table: &IdTable) -> i32 {
    match expr {
        Expr::Number(n) => *n,
        Expr::UnaryExpr(op, expr) => {
            let out = compute_expr(expr.as_ref(), id_table);
            match op {
                UnaryOp::Not => {
                    if out == 0 {
                        1
                    } else {
                        0
                    }
                }
                UnaryOp::Plus => out,
                UnaryOp::Minus => -out,
            }
        }
        Expr::BinaryExpr(lhs, op, rhs) => {
            let lhs_val = compute_expr(lhs.as_ref(), id_table);
            let rhs_val = compute_expr(rhs.as_ref(), id_table);
            match op {
                BinaryOp::Plus => lhs_val + rhs_val,
                BinaryOp::Minus => lhs_val - rhs_val,
                BinaryOp::Multiply => lhs_val * rhs_val,
                BinaryOp::Divide => lhs_val / rhs_val,
                BinaryOp::Modulo => lhs_val % rhs_val,
                BinaryOp::Less => {
                    if lhs_val < rhs_val {
                        1
                    } else {
                        0
                    }
                }
                BinaryOp::Greater => {
                    if lhs_val > rhs_val {
                        1
                    } else {
                        0
                    }
                }
                BinaryOp::LessOrEqual => {
                    if lhs_val <= rhs_val {
                        1
                    } else {
                        0
                    }
                }
                BinaryOp::GreaterOrEqual => {
                    if lhs_val >= rhs_val {
                        1
                    } else {
                        0
                    }
                }
                BinaryOp::Eq => {
                    if lhs_val == rhs_val {
                        1
                    } else {
                        0
                    }
                }
                BinaryOp::Neq => {
                    if lhs_val != rhs_val {
                        1
                    } else {
                        0
                    }
                }
                BinaryOp::And => {
                    if lhs_val != 0 && rhs_val != 0 {
                        1
                    } else {
                        0
                    }
                }
                BinaryOp::Or => {
                    if lhs_val != 0 || rhs_val != 0 {
                        1
                    } else {
                        0
                    }
                }
            }
        }
        Expr::LVal(lval) => {
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
fn expr2ir(exp: &Expr, id_table: &IdTable) -> (String, i32) {
    // println!("{:#?}", exp);
    match exp {
        Expr::Number(n) => (String::new(), *n),
        Expr::UnaryExpr(op, expr) => {
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
                UnaryOp::Not => (format!("{}%{} = eq 0, {}\n", out.0, counter, pos), counter),
                UnaryOp::Minus => (format!("{}%{} = sub 0, {}\n", out.0, counter, pos), counter),
                UnaryOp::Plus => (out.0, out.1),
            }
        }
        Expr::BinaryExpr(lhs, op, rhs) => {
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
            let generate_lhs = lout.0;
            let generate_rhs = rout.0;
            let mut out = String::new();
            let mut counter = COUNTER.lock().unwrap();
            *counter += 1;
            match op {
                BinaryOp::Plus => {
                    out += &generate_lhs;
                    out += &generate_rhs;
                    out += &format!("%{} = add {}, {}\n", counter, lpos, rpos);
                    (out, *counter)
                }
                BinaryOp::Minus => {
                    out += &generate_lhs;
                    out += &generate_rhs;
                    out += &format!("%{} = sub {}, {}\n", counter, lpos, rpos);
                    (out, *counter)
                }
                BinaryOp::Multiply => {
                    out += &generate_lhs;
                    out += &generate_rhs;
                    out += &format!("%{} = mul {}, {}\n", counter, lpos, rpos);
                    (out, *counter)
                }
                BinaryOp::Divide => {
                    out += &generate_lhs;
                    out += &generate_rhs;
                    out += &format!("%{} = div {}, {}\n", counter, lpos, rpos);
                    (out, *counter)
                }
                BinaryOp::Modulo => {
                    out += &generate_lhs;
                    out += &generate_rhs;
                    out += &format!("%{} = mod {}, {}\n", counter, lpos, rpos);
                    (out, *counter)
                }
                BinaryOp::Eq => {
                    out += &generate_lhs;
                    out += &generate_rhs;
                    out += &format!("%{} = eq {}, {}\n", counter, lpos, rpos);
                    (out, *counter)
                }
                BinaryOp::Neq => {
                    out += &generate_lhs;
                    out += &generate_rhs;
                    out += &format!("%{} = ne {}, {}\n", counter, lpos, rpos);
                    (out, *counter)
                }
                BinaryOp::Less => {
                    out += &generate_lhs;
                    out += &generate_rhs;
                    out += &format!("%{} = lt {}, {}\n", counter, lpos, rpos);
                    (out, *counter)
                }
                BinaryOp::LessOrEqual => {
                    out += &generate_lhs;
                    out += &generate_rhs;
                    out += &format!("%{} = le {}, {}\n", counter, lpos, rpos);
                    (out, *counter)
                }
                BinaryOp::Greater => {
                    out += &generate_lhs;
                    out += &generate_rhs;
                    out += &format!("%{} = lt {}, {}\n", counter, rpos, lpos);
                    (out, *counter)
                }
                BinaryOp::GreaterOrEqual => {
                    out += &generate_lhs;
                    out += &generate_rhs;
                    out += &format!("%{} = le {}, {}\n", counter, rpos, lpos);
                    (out, *counter)
                }
                BinaryOp::And => {
                    out += &generate_lhs;
                    out += &format!("%{} = ne {}, {}\n", counter, 0, lpos);
                    let lpos = *counter;
                    *counter += 1;
                    let dest = *counter;
                    out += &format!("@tmp{} = alloc i32\n", dest);
                    out += &format!("store %{}, @tmp{}\n", lpos, dest);
                    let if_id = {
                        let mut counter = IF_COUNTER.lock().unwrap();
                        *counter += 1;
                        *counter
                    };
                    out += &format!("br %{}, %and_if{}, %and_end{}\n", lpos, if_id, if_id);
                    out += &format!("%and_if{}:\n", if_id);
                    out += &generate_rhs;
                    *counter += 1;
                    out += &format!("%{} = ne {}, {}\n", counter, 0, rpos);
                    let rpos = *counter;
                    out += &format!("store %{}, @tmp{}\n", rpos, dest);
                    out += &format!("jump %and_end{}\n", if_id);
                    out += &format!("%and_end{}:\n", if_id);
                    *counter += 1;
                    out += &format!("%{} = load @tmp{}\n", *counter, dest);
                    (out, *counter)
                }
                BinaryOp::Or => {
                    out += &generate_lhs;
                    out += &format!("%{} = ne {}, {}\n", counter, 0, lpos);
                    let lpos = *counter;
                    *counter += 1;
                    let dest = *counter;
                    out += &format!("@tmp{} = alloc i32\n", dest);
                    out += &format!("store %{}, @tmp{}\n", lpos, dest);
                    let if_id = {
                        let mut counter = IF_COUNTER.lock().unwrap();
                        *counter += 1;
                        *counter
                    };
                    *counter += 1;
                    out += &format!("%{} = eq {}, %{}\n", counter, 0, lpos);
                    out += &format!("br %{}, %or_if{}, %or_end{}\n", counter, if_id, if_id);
                    out += &format!("%or_if{}:\n", if_id);
                    out += &generate_rhs;
                    *counter += 1;
                    out += &format!("%{} = ne {}, {}\n", counter, 0, rpos);
                    let rpos = *counter;
                    out += &format!("store %{}, @tmp{}\n", rpos, dest);
                    out += &format!("jump %or_end{}\n", if_id);
                    out += &format!("%or_end{}:\n", if_id);
                    *counter += 1;
                    out += &format!("%{} = load @tmp{}\n", *counter, dest);
                    (out, *counter)
                } // _ => unreachable!(),
            }
        }
        Expr::LVal(lval) => {
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
