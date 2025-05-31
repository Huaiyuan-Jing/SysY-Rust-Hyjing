use crate::ast::{self, FuncType};
use koopa;
pub fn ast2ir(ast: &ast::CompUnit) -> String {
    // 生成 IR 代码
    let func_type = match ast.func_def.func_type {
        FuncType::Int => "i32",
    };
    let out = format!(
        "fun @{}(): {} {{\n%entry:\nret {}\n}}",
        &ast.func_def.ident, func_type, ast.func_def.block.stmt.num
    );
    out
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
