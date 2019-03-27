use super::{bytecode, types, Object};
#[derive(Debug)]
pub struct Closure {
    pub func_proto: Object,
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
