use super::{bytecode, types, Object};
use std::collections::HashMap;
#[derive(Debug)]
pub struct Closure {
    pub func_proto: Object,
}

impl Closure {
    pub fn new(func_proto: Object) -> Self {
        Closure {
            func_proto: func_proto,
        }
    }
}

#[derive(Debug)]
pub struct FuncProto {
    pub source_name: Object,
    pub name: Object,

    pub literals: Vec<Object>,
    pub parameters: Vec<Object>,
    pub outervalues: Vec<(types::Integer, Object, Object)>,
    pub localvarinfos: Vec<(Object, types::Integer, types::Integer, types::Integer)>,
    pub lineinfos: Vec<(types::Integer, types::Integer)>,
    pub defaultparams: Vec<types::Integer>,
    pub instructions: Vec<bytecode::Instruction>,
    pub functions: Vec<Object>,

    pub stacksize: types::Integer,
}

impl FuncProto {
    pub fn print_disassembly(&self, indent: &str) {
        println!("{}function {} {}", indent, self.source_name, self.name);
        for (i, inst) in self.instructions.iter().enumerate() {
            println!("{}  {} {:?}", indent, i, inst);
        }

        for f in &self.functions {
            if let Object::FuncProto(func) = f {
                func.print_disassembly(&format!("{}  ", indent)[..]);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Table {
    pub map: HashMap<Object, Object>,
}

impl Table {
    pub fn new() -> Self {
        Table {
            map: HashMap::new(),
        }
    }
}
