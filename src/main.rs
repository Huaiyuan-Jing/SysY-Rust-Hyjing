use lalrpop_util::lalrpop_mod;
use std::env::args;
use std::fs::{self, read_to_string};
use std::io::Result;

mod ast;
mod ast2ir;
mod ir2riscv;

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
    let mut koopa_ir;
    if mode == "-koopa" {
        koopa_ir = ast2ir::ast2ir(&ast);
        fs::write(outfile, koopa_ir)?;
        return Ok(());
    }
    let mut riscv;
    if mode == "-riscv" {
        koopa_ir = ast2ir::ast2ir(&ast);
        riscv = ir2riscv::ir2riscv(koopa_ir);
        fs::write(outfile, riscv)?;
        return Ok(());
    }
    Ok(())
}
