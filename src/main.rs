use lalrpop_util::lalrpop_mod;
use std::env::args;
use std::fs::{self, read_to_string};
use std::io::Result;
mod ast;
mod ir;

lalrpop_mod!(sysy);

fn main() -> Result<()> {
    let mut args = args();
    args.next();
    let mode = args.next().unwrap();
    let input = args.next().unwrap();
    args.next();
    let outfile = args.next().unwrap();

    let input = read_to_string(input)?;

    let ast = sysy::CompUnitParser::new().parse(&input).unwrap();
    println!("{:#?}", ast);
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
