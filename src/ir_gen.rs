use crate::ast::{self, FuncType};
pub fn ast2ir(ast: &ast::CompUnit) -> String {
    // 生成 IR 代码
    let func_type = match ast.func_def.func_type {
        FuncType::Int => "i32",
        _ => "unknown",
    };
    let out = format!("fun @{}: {} {{\n%entry:\nret {}\n}}", &ast.func_def.ident, func_type, ast.func_def.block.stmt.num);
    out
}