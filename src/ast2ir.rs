use crate::ast::{self, FuncType};
use lazy_static::lazy_static;
use std::sync::Mutex;
lazy_static! {
    static ref COUNTER: Mutex<i32> = Mutex::new(-1);
}
pub fn ast2ir(ast: &ast::CompUnit) -> String {
    let func_type = match ast.func_def.func_type {
        FuncType::Int => "i32",
    };
    let mut out = format!(
        "fun @{}(): {} {{\n%entry:\n",
        &ast.func_def.ident, func_type
    );
    let tmp = expr2ir(&ast.func_def.block.stmt.exp);
    let pos = if tmp.0 == String::new() {
        tmp.1.to_string()
    } else {
        format!("%{}", tmp.1)
    };
    out += &format!("{}ret {}\n", tmp.0, pos);
    out += "}\n";
    out
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
                }
                // _ => unreachable!(),
            }
        }
        // _ => unreachable!(),
    }
}
