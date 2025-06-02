use crate::ast::{self, FuncType};
use koopa;
use lazy_static::lazy_static;
use std::{collections::HashMap, sync::Mutex};
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
    out += &format!("{}ret %{}\n", tmp.0, tmp.1);
    out += "}\n";
    out
}
fn expr2ir(exp: &ast::Expr) -> (String, i32) {
    match exp {
        ast::Expr::Number(n) => {
            let mut counter = COUNTER.lock().unwrap();
            *counter += 1;
            (format!("%{} = add 0, {}\n", *counter, n), *counter)
        }
        ast::Expr::UnaryExpr(op, expr) => {
            let out = expr2ir(expr);
            let mut counter = COUNTER.lock().unwrap();
            match op {
                ast::UnaryOp::Not => {
                    *counter += 1;
                    (
                        format!("{}%{} = eq 0, %{}\n", out.0, *counter, out.1),
                        *counter,
                    )
                }
                ast::UnaryOp::Minus => {
                    *counter += 1;
                    (
                        format!("{}%{} = sub 0, %{}\n", out.0, *counter, out.1),
                        *counter,
                    )
                }
                ast::UnaryOp::Plus => (out.0, out.1),
            }
        }
    }
}
pub fn ir2riscv(ir: String) -> String {
    let mut out = String::new();
    let driver = koopa::front::Driver::from(ir);
    let program = driver.generate_program().unwrap();
    out += ".text\n";

    for &func in program.func_layout() {
        let func_data = program.func(func);
        out += &format!(".globl {}\n", &func_data.name()[1..]);
        out += &format!("{}:\n", &func_data.name()[1..]);
        let mut reg_map: HashMap<koopa::ir::Value, String> = HashMap::new();
        let mut reg_count = 0;
        for (&_, node) in func_data.layout().bbs() {
            for &inst in node.insts().keys() {
                let code = stmt2str(func_data, &inst, &mut reg_map, &mut reg_count);
                out += &code;
            }
        }
    }
    out
}
fn stmt2str(
    func_data: &koopa::ir::FunctionData,
    value: &koopa::ir::Value,
    reg_map: &mut HashMap<koopa::ir::Value, String>,
    reg_count: &mut usize,
) -> String {
    match func_data.dfg().value(*value).kind() {
        koopa::ir::ValueKind::Integer(int) => {
            if int.value() == 0 {
                reg_map.insert(*value, "x0".to_string());
                return String::new();
            }
            let reg = *reg_count;
            *reg_count += 1;
            reg_map.insert(*value, format!("t{}", reg));
            format!("li t{}, {}\n", reg, int.value())
        }
        koopa::ir::ValueKind::Return(ret) => {
            let mut out = String::new();
            if !reg_map.contains_key(&ret.value().unwrap()) {
                out += &stmt2str(func_data, &ret.value().unwrap(), reg_map, reg_count);
            }
            let val_reg = reg_map.get(&ret.value().unwrap()).unwrap();
            out += &format!("mv a0, {}\nret\n", val_reg);
            out
        }
        koopa::ir::ValueKind::Binary(bin) => {
            let mut out = String::new();
            if !reg_map.contains_key(&bin.lhs()) {
                out += &stmt2str(func_data, &bin.lhs(), reg_map, reg_count);
            }
            if !reg_map.contains_key(&bin.rhs()) {
                out += &stmt2str(func_data, &bin.rhs(), reg_map, reg_count);
            }
            let dest_reg = *reg_count;
            *reg_count += 1;
            reg_map.insert(*value, format!("t{}", dest_reg));
            let dest_reg = reg_map.get(value).unwrap();
            let lhs_reg = reg_map.get(&bin.lhs()).unwrap();
            let rhs_reg = reg_map.get(&bin.rhs()).unwrap();
            match bin.op() {
                koopa::ir::BinaryOp::Add => {
                    out += &format!("add {}, {}, {}\n", dest_reg, lhs_reg, rhs_reg);
                    out
                }
                koopa::ir::BinaryOp::Sub => {
                    out += &format!("sub {}, {}, {}\n", dest_reg, lhs_reg, rhs_reg);
                    out
                }
                koopa::ir::BinaryOp::Eq => {
                    out += &format!(
                        "xor {}, {}, {}\nseqz {}, {}\n",
                        dest_reg, lhs_reg, rhs_reg, dest_reg, dest_reg
                    );
                    out
                }
                _ => unreachable!(),
            }
        }
        _ => "".to_string(),
    }
}
