use std::collections::HashMap;

pub fn ir2riscv(ir: String) -> String {
    let mut out = String::new();
    let driver = koopa::front::Driver::from(ir);
    let program = driver.generate_program().unwrap();
    out += ".data\n";
    for &inst in program.inst_layout() {
        println!("inst: {:?}", program.borrow_value(inst).kind());
        let name = &program.borrow_value(inst).name().clone().unwrap()[1..];
        out += &format!(".globl {}\n", name);
        out += &format!("{}:\n", name);
        
    }
    for &func in program.func_layout() {
        let func_data = program.func(func);
        if func_data.layout().entry_bb().is_none() {
            continue;
        }
        out += ".text\n";
        out += &format!(".globl {}\n", &func_data.name()[1..]);
        out += &format!("{}:\n", &func_data.name()[1..]);
        let mut stack_map: HashMap<koopa::ir::Value, String> = HashMap::new();
        let mut stack_offset = -4;
        let mut size = func_data.dfg().values().len() * 4 + 4;
        let mut max_arg_num = 0;
        for (_, node) in func_data.layout().bbs() {
            for &inst in node.insts().keys() {
                match func_data.dfg().value(inst).kind() {
                    koopa::ir::ValueKind::Call(call_inst) => {
                        max_arg_num =
                            max_arg_num.max(program.func(call_inst.callee()).params().len());
                    }
                    _ => {
                        continue;
                    }
                }
            }
        }
        size += max_arg_num * 4;
        stack_offset += max_arg_num as i32 * 4;
        if size % 16 != 0 {
            size += 16 - size % 16;
        }
        out += &format!("addi sp, sp, -{}\n", size);
        out += &format!("sw ra {}(sp)\n", size - 4);
        let mut arg_pos = size;
        for arg in func_data.params() {
            stack_map.insert(*arg, format!("{}(sp)", arg_pos));
            arg_pos += 4;
        }
        for (&bb, node) in func_data.layout().bbs() {
            // println!("Block: {:?}", func_data.dfg().bb(bb).name());
            if func_data.dfg().bb(bb).name().as_ref().unwrap() != "%entry" {
                out += &format!(
                    "{}:\n",
                    &func_data.dfg().bb(bb).name().as_ref().unwrap()[1..]
                );
            }
            for &inst in node.insts().keys() {
                let code = stmt2str(
                    &program,
                    func_data,
                    &inst,
                    0,
                    &mut stack_map,
                    &mut stack_offset,
                    size as i32,
                );
                out += &code;
            }
        }
        out += "\n";
    }
    out
}
fn stmt2str(
    program: &koopa::ir::Program,
    func_data: &koopa::ir::FunctionData,
    value: &koopa::ir::Value,
    reg_count: usize,
    stack_map: &mut HashMap<koopa::ir::Value, String>,
    stack_offset: &mut i32,
    size: i32,
) -> String {
    println!(
        "Processing statement: {:?}",
        func_data.dfg().value(*value).kind()
    );
    let mut out = String::new();
    match func_data.dfg().value(*value).kind() {
        koopa::ir::ValueKind::Integer(int) => out = format!("li t{}, {}\n", reg_count, int.value()),
        koopa::ir::ValueKind::Return(ret) => {
            if ret.value().is_none() {
                out += &format!("lw ra {}(sp)\n", size - 4);
                out += &format!("addi sp, sp, {}\n", size);
                out += "ret\n";
            } else {
                if stack_map.contains_key(&ret.value().unwrap()) {
                    out += &format!("lw a0, {}\n", stack_map.get(&ret.value().unwrap()).unwrap());
                } else {
                    out += &stmt2str(
                        program,
                        func_data,
                        &ret.value().unwrap(),
                        reg_count,
                        stack_map,
                        stack_offset,
                        size,
                    );
                    out += &format!("mv a0, t{}\n", reg_count);
                }
                out += &format!("lw ra {}(sp)\n", size - 4);
                out += &format!("addi sp, sp, {}\n", size);
                out += "ret\n";
            }
        }
        koopa::ir::ValueKind::Binary(bin) => {
            let dest_reg = format!("t{}", reg_count);
            let lhs_reg = format!("t{}", reg_count);
            let rhs_reg = format!("t{}", reg_count + 1);
            if stack_map.contains_key(&bin.lhs()) {
                out += &format!("lw {}, {}\n", lhs_reg, stack_map.get(&bin.lhs()).unwrap());
            } else {
                out += &stmt2str(
                    program,
                    func_data,
                    &bin.lhs(),
                    reg_count,
                    stack_map,
                    stack_offset,
                    size,
                );
            }
            if stack_map.contains_key(&bin.rhs()) {
                out += &format!("lw {}, {}\n", rhs_reg, stack_map.get(&bin.rhs()).unwrap());
            } else {
                out += &stmt2str(
                    program,
                    func_data,
                    &bin.rhs(),
                    reg_count + 1,
                    stack_map,
                    stack_offset,
                    size,
                );
            }
            match bin.op() {
                koopa::ir::BinaryOp::Add => {
                    out += &format!("add {}, {}, {}\n", dest_reg, lhs_reg, rhs_reg);
                }
                koopa::ir::BinaryOp::Sub => {
                    out += &format!("sub {}, {}, {}\n", dest_reg, lhs_reg, rhs_reg);
                }
                koopa::ir::BinaryOp::Eq => {
                    out += &format!(
                        "xor {}, {}, {}\nseqz {}, {}\n",
                        dest_reg, lhs_reg, rhs_reg, dest_reg, dest_reg
                    );
                }
                koopa::ir::BinaryOp::NotEq => {
                    out += &format!(
                        "xor {}, {}, {}\nsnez {}, {}\n",
                        dest_reg, lhs_reg, rhs_reg, dest_reg, dest_reg
                    );
                }
                koopa::ir::BinaryOp::Mul => {
                    out += &format!("mul {}, {}, {}\n", dest_reg, lhs_reg, rhs_reg);
                }
                koopa::ir::BinaryOp::Div => {
                    out += &format!("div {}, {}, {}\n", dest_reg, lhs_reg, rhs_reg);
                }
                koopa::ir::BinaryOp::Mod => {
                    out += &format!("rem {}, {}, {}\n", dest_reg, lhs_reg, rhs_reg);
                }
                koopa::ir::BinaryOp::Lt => {
                    out += &format!("slt {}, {}, {}\n", dest_reg, lhs_reg, rhs_reg);
                }
                koopa::ir::BinaryOp::And => {
                    out += &format!("and {}, {}, {}\n", dest_reg, lhs_reg, rhs_reg);
                }
                koopa::ir::BinaryOp::Or => {
                    out += &format!("or {}, {}, {}\n", dest_reg, lhs_reg, rhs_reg);
                }
                koopa::ir::BinaryOp::Le => {
                    out += &format!("slt {}, {}, {}\n", dest_reg, rhs_reg, lhs_reg);
                    out += &format!("xori {}, {}, 1\n", dest_reg, dest_reg);
                }
                _ => unreachable!(),
            }
            out += &format!("sw {}, {}(sp)\n", dest_reg, *stack_offset + 4);
            *stack_offset += 4;
            stack_map.insert(*value, format!("{}(sp)", stack_offset));
        }
        koopa::ir::ValueKind::Alloc(_) => {}
        koopa::ir::ValueKind::Store(store) => {
            if stack_map.contains_key(&store.value()) {
                out += &format!(
                    "lw t{}, {}\n",
                    reg_count,
                    stack_map.get(&store.value()).unwrap()
                );
            } else {
                out += &stmt2str(
                    program,
                    func_data,
                    &store.value(),
                    reg_count,
                    stack_map,
                    stack_offset,
                    size,
                );
            }
            if !stack_map.contains_key(&store.dest()) {
                *stack_offset += 4;
                stack_map.insert(store.dest(), format!("{}(sp)", stack_offset));
            }
            out += &format!(
                "sw t{}, {}\n",
                reg_count,
                stack_map.get(&store.dest()).unwrap()
            );
        }
        koopa::ir::ValueKind::Load(load) => {
            out += &format!(
                "lw t{}, {}\n",
                reg_count,
                stack_map.get(&load.src()).unwrap()
            );
            *stack_offset += 4;
            stack_map.insert(*value, format!("{}(sp)", stack_offset));
            out += &format!("sw t{}, {}(sp)\n", reg_count, stack_offset);
        }
        koopa::ir::ValueKind::Branch(branch) => {
            out += &stmt2str(
                program,
                func_data,
                &branch.cond(),
                reg_count,
                stack_map,
                stack_offset,
                size,
            );
            out += &format!(
                "bnez t{}, {}\n",
                reg_count,
                &func_data
                    .dfg()
                    .bb(branch.true_bb())
                    .name()
                    .as_ref()
                    .unwrap()[1..]
            );
            out += &format!(
                "j {}\n",
                &func_data
                    .dfg()
                    .bb(branch.false_bb())
                    .name()
                    .as_ref()
                    .unwrap()[1..]
            );
        }
        koopa::ir::ValueKind::Jump(jump) => {
            out += &format!(
                "j {}\n",
                &func_data.dfg().bb(jump.target()).name().as_ref().unwrap()[1..]
            );
        }
        koopa::ir::ValueKind::Call(call) => {
            let mut arg_pos = 0;
            for arg in call.args() {
                if stack_map.contains_key(arg) {
                    out += &format!("lw t{}, {}\n", reg_count, stack_map.get(arg).unwrap());
                } else {
                    out += &stmt2str(
                        program,
                        func_data,
                        arg,
                        reg_count,
                        stack_map,
                        stack_offset,
                        size,
                    );
                }
                out += &format!("sw t{}, {}(sp)\n", reg_count, arg_pos * 4);
                arg_pos += 1;
            }
            out += &format!("call {}\n", &program.func(call.callee()).name()[1..]);
            *stack_offset += 4;
            stack_map.insert(*value, format!("{}(sp)", *stack_offset));
            out += &format!("sw a0, {}\n", stack_map.get(value).unwrap());
            out += &format!("lw t{}, {}\n", reg_count, stack_map.get(value).unwrap());
        }
        _ => {}
    }
    out
}
