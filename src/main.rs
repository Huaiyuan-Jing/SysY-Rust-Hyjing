use lalrpop_util::lalrpop_mod;
use std::env::args;
use std::fs::{self, read_to_string};
use std::io::Result;
mod ast;
mod ir;

// 引用 lalrpop 生成的解析器
lalrpop_mod!(sysy);

fn main() -> Result<()> {
    // 解析命令行参数
    let mut args = args();
    args.next();
    let mode = args.next().unwrap();
    let input = args.next().unwrap();
    args.next();
    let outfile = args.next().unwrap();

    // 读取输入文件
    let input = read_to_string(input)?;

    // 调用 lalrpop 生成的 parser 解析输入文件
    let ast = sysy::CompUnitParser::new().parse(&input).unwrap();

    // 输出解析得到的 AST
    // println!("{:#?}", ast);
    let koopa_ir = ir::ast2ir(&ast);
    if mode == "-koopa" {
        fs::write(outfile, koopa_ir)?;
        return Ok(());
    }
    let riscv = ir::ir2riscv(koopa_ir);
    if mode == "-riscv" {
        fs::write(outfile, riscv)?;
        return Ok(());
    }
    Ok(())
}
