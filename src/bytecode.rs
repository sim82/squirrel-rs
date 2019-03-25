use super::Result;
use byteorder::{LittleEndian, ReadBytesExt};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

use std::fmt::Formatter;
use std::io::Read;

#[derive(FromPrimitive, ToPrimitive, Debug)]
pub enum Opcode {
    LINE = 0x00,
    LOAD = 0x01,
    LOADINT = 0x02,
    LOADFLOAT = 0x03,
    DLOAD = 0x04,
    TAILCALL = 0x05,
    CALL = 0x06,
    PREPCALL = 0x07,
    PREPCALLK = 0x08,
    GETK = 0x09,
    MOVE = 0x0A,
    NEWSLOT = 0x0B,
    DELETE = 0x0C,
    SET = 0x0D,
    GET = 0x0E,
    EQ = 0x0F,
    NE = 0x10,
    ADD = 0x11,
    SUB = 0x12,
    MUL = 0x13,
    DIV = 0x14,
    MOD = 0x15,
    BITW = 0x16,
    RETURN = 0x17,
    LOADNULLS = 0x18,
    LOADROOT = 0x19,
    LOADBOOL = 0x1A,
    DMOVE = 0x1B,
    JMP = 0x1C,
    JCMP = 0x1D,
    JZ = 0x1E,
    SETOUTER = 0x1F,
    GETOUTER = 0x20,
    NEWOBJ = 0x21,
    APPENDARRAY = 0x22,
    COMPARITH = 0x23,
    INC = 0x24,
    INCL = 0x25,
    PINC = 0x26,
    PINCL = 0x27,
    CMP = 0x28,
    EXISTS = 0x29,
    INSTANCEOF = 0x2A,
    AND = 0x2B,
    OR = 0x2C,
    NEG = 0x2D,
    NOT = 0x2E,
    BWNOT = 0x2F,
    CLOSURE = 0x30,
    YIELD = 0x31,
    RESUME = 0x32,
    FOREACH = 0x33,
    POSTFOREACH = 0x34,
    CLONE = 0x35,
    TYPEOF = 0x36,
    PUSHTRAP = 0x37,
    POPTRAP = 0x38,
    THROW = 0x39,
    NEWSLOTA = 0x3A,
    GETBASE = 0x3B,
    CLOSE = 0x3C,
}

// #[derive(Debug)]
pub struct Instruction {
    arg1: u32,
    opcode: u8,
    arg0: u8,
    arg2: u8,
    arg3: u8,
}

impl Instruction {
    pub fn read(rdr: &mut dyn Read) -> Result<Instruction> {
        let arg1 = rdr.read_u32::<LittleEndian>()?;
        let mut buf = [0u8; 4];
        rdr.read(&mut buf)?;

        Ok(Instruction {
            arg1: arg1,
            opcode: buf[0],
            arg0: buf[1],
            arg2: buf[2],
            arg3: buf[3],
        })
    }
}

impl std::fmt::Debug for Instruction {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        // fmt.write_fmt(format_args!(
        write!(
            fmt,
            "{:?} {} {} {} {}",
            Opcode::from_u8(self.opcode).unwrap(),
            self.arg0,
            self.arg1 as u32,
            self.arg2 as u32,
            self.arg3 as u32
        )
    }
}
