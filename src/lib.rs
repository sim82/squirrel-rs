use num_derive::{FromPrimitive, ToPrimitive};
use std::rc::Rc;
// use num_traits::FromPrimitive;

pub mod bytecode;
pub mod io;
pub mod vm;

pub mod object;
#[derive(Debug)]
pub enum Error {
    RuntimeError(String),
    IoError(std::io::Error),
}

impl std::convert::From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IoError(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub mod raw_type {
    pub const NULL: isize = 0x00000001;
    pub const INTEGER: isize = 0x00000002;
    pub const FLOAT: isize = 0x00000004;
    pub const BOOL: isize = 0x00000008;
    pub const STRING: isize = 0x00000010;
    pub const TABLE: isize = 0x00000020;
    pub const ARRAY: isize = 0x00000040;
    pub const USERDATA: isize = 0x00000080;
    pub const CLOSURE: isize = 0x00000100;
    pub const NATIVECLOSURE: isize = 0x00000200;
    pub const GENERATOR: isize = 0x00000400;
    pub const USERPOINTER: isize = 0x00000800;
    pub const THREAD: isize = 0x00001000;
    pub const FUNCPROTO: isize = 0x00002000;
    pub const CLASS: isize = 0x00004000;
    pub const INSTANCE: isize = 0x00008000;
    pub const WEAKREF: isize = 0x00010000;
    pub const OUTER: isize = 0x00020000;
}

pub mod obj_flags {
    pub const REF_COUNTED: isize = 0x08000000;
    pub const NUMERIC: isize = 0x04000000;
    pub const DELEGABLE: isize = 0x02000000;
    pub const CANBEFALSE: isize = 0x01000000;
}

#[derive(FromPrimitive, ToPrimitive)]
enum ObjectType {
    Null = (raw_type::NULL | obj_flags::CANBEFALSE),
    Integer = (raw_type::INTEGER | obj_flags::NUMERIC | obj_flags::CANBEFALSE),
    Float = (raw_type::FLOAT | obj_flags::NUMERIC | obj_flags::CANBEFALSE),
    Bool = (raw_type::BOOL | obj_flags::CANBEFALSE),
    String = (raw_type::STRING | obj_flags::REF_COUNTED),
    Table = (raw_type::TABLE | obj_flags::REF_COUNTED | obj_flags::DELEGABLE),
    Array = (raw_type::ARRAY | obj_flags::REF_COUNTED),
    Userdata = (raw_type::USERDATA | obj_flags::REF_COUNTED | obj_flags::DELEGABLE),
    Closure = (raw_type::CLOSURE | obj_flags::REF_COUNTED),
    Nativeclosure = (raw_type::NATIVECLOSURE | obj_flags::REF_COUNTED),
    Generator = (raw_type::GENERATOR | obj_flags::REF_COUNTED),
    Userpointer = raw_type::USERPOINTER,
    Thread = (raw_type::THREAD | obj_flags::REF_COUNTED),
    Funcproto = (raw_type::FUNCPROTO | obj_flags::REF_COUNTED), //internal usage only
    Class = (raw_type::CLASS | obj_flags::REF_COUNTED),
    Instance = (raw_type::INSTANCE | obj_flags::REF_COUNTED | obj_flags::DELEGABLE),
    Weakref = (raw_type::WEAKREF | obj_flags::REF_COUNTED),
    Outer = (raw_type::OUTER | obj_flags::REF_COUNTED), //internal usage only
}

#[derive(FromPrimitive, ToPrimitive, PartialEq, Debug)]
enum FileTags {
    BytecodeStreamTag = 0xFAFA,

    ClosurestreamHead =
        ((('S' as isize) << 24) | (('Q' as isize) << 16) | (('I' as isize) << 8) | ('R' as isize)),
    ClosurestreamPart =
        ((('P' as isize) << 24) | (('A' as isize) << 16) | (('R' as isize) << 8) | ('T' as isize)),
    ClosurestreamTail =
        ((('T' as isize) << 24) | (('A' as isize) << 16) | (('I' as isize) << 8) | ('L' as isize)),

    SizeChar = 1,
    SizeInteger = 8,
    SizeFloat = 4,
}

pub mod types {
    pub type Integer = i64;
    pub type Float = f32;
}

#[derive(Debug, Clone)]
pub enum Object {
    Integer(types::Integer),
    Float(types::Float),
    String(String),
    FuncProto(Rc<object::FuncProto>),
    Closure(Rc<object::Closure>),
    Null,
}

impl Object {
    fn closure(&self) -> Result<Rc<object::Closure>> {
        match self {
            Object::Closure(closure) => Ok(closure.clone()),
            _ => Err(Error::RuntimeError(format!(
                "expected closure. found {:?}",
                self
            ))),
        }
    }
    fn func_proto(&self) -> Result<Rc<object::FuncProto>> {
        match self {
            Object::FuncProto(fp) => Ok(fp.clone()),
            _ => Err(Error::RuntimeError(format!(
                "expected FuncProto. found {:?}",
                self
            ))),
        }
    }
}
