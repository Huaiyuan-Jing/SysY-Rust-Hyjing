use crate::ast::{self, FuncType};
use koopa;
use lazy_static::lazy_static;
use std::sync::Mutex;
lazy_static! {
    static ref COUNTER: Mutex<i32> = Mutex::new(-1);
}
pub fn ast2ir(ast: &ast::CompUnit) -> String {
    let func_type = match ast.func_def.func_type {
        FuncType::Int => "i32",
    };
    let mut out = format!("fun @{}(): {} {{\n%entry:\n", &ast.func_def.ident, func_type);
    let tmp = expr2ir(&ast.func_def.block.stmt.exp);
    out += &format!("{}ret %{}\n", tmp.0, tmp.1);
    out += "}\n";
    out
}
fn expr2ir(exp: &ast::Expr) -> (String, i32) {
    println!("{:?}", exp);
    match exp {
        ast::Expr::Number(n) => {
            let mut counter = COUNTER.lock().unwrap();
            *counter += 1;
            (format!("%{} = {}\n", *counter, n), *counter)
        }
        ast::Expr::UnaryExpr(op, expr) => {
            let out = expr2ir(expr);
            let mut counter = COUNTER.lock().unwrap();
            match op {
                ast::UnaryOp::Not => {
                    *counter += 1;
                    (
                        format!("{}%{} = eq %{}, 0\n", out.0, *counter, out.1),
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
        out += &format!(".global {}\n", &func_data.name().to_string()[1..]);
        out += &format!("{}:\n", &func_data.name().to_string()[1..]);
        for (&_, node) in func_data.layout().bbs() {
            for &inst in node.insts().keys() {
                let value_data = func_data.dfg().value(inst);
                out += &stmt2str(func_data, value_data);
            }
        }
    }
    out
}
fn stmt2str(
    func_data: &koopa::ir::FunctionData,
    value_data: &koopa::ir::entities::ValueData,
) -> String {
    match value_data.kind() {
        koopa::ir::ValueKind::Integer(int) => int.value().to_string(),
        koopa::ir::ValueKind::Return(ret) => {
            let tmp = (*func_data.dfg().value(ret.value().unwrap())).clone();
            format!("li a0, {}\nret\n", stmt2str(func_data, &tmp))
        }
        _ => unreachable!(),
    }
}
