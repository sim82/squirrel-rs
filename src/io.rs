use super::{object, Error, FileTags, Object, ObjectType, Result};

use byteorder::{LittleEndian, ReadBytesExt};
use num_traits::{FromPrimitive, ToPrimitive};
use std::io::Read;
use std::rc::Rc;

fn read_string(rdr: &mut dyn Read) -> Result<Object> {
    let len = rdr.read_u64::<LittleEndian>()? as usize;
    let mut buf = vec![0; len];
    match rdr.read(&mut buf) {
        Ok(rlen) if rlen == len => {
            Ok(Object::String(String::from_utf8(buf).map_err(|x| {
                Error::RuntimeError(format!("failed to decode utf8: {}", x))
            })?))
        }
        Ok(rlen) => Err(Error::RuntimeError(format!(
            "could not read {} bytes for string. Got {}",
            len, rlen
        ))),
        Err(e) => Err(e.into()),
    }
}

fn read_object(rdr: &mut dyn Read) -> Result<Object> {
    let obj_type = rdr.read_u32::<LittleEndian>()?;

    match FromPrimitive::from_u32(obj_type) {
        Some(ObjectType::Integer) => Ok(Object::Integer(rdr.read_i64::<LittleEndian>()?)),
        Some(ObjectType::Float) => Ok(Object::Float(rdr.read_f32::<LittleEndian>()?)),
        Some(ObjectType::String) => read_string(rdr),
        Some(_) => panic!("unhandled object type"),
        None => Err(Error::RuntimeError(format!(
            "failed to decode object type: {}",
            obj_type,
        ))),
    }
}

fn read_tag(rdr: &mut dyn Read) -> Result<FileTags> {
    let tag = rdr.read_u32::<LittleEndian>()?;
    FromPrimitive::from_u32(tag).ok_or(Error::RuntimeError(format!("failed to map tag {}", tag)))
}

fn expect_tag(rdr: &mut dyn Read, rtag: FileTags) -> Result<()> {
    let tag = read_tag(rdr)?;

    if tag == rtag {
        Ok(())
    } else {
        Err(Error::RuntimeError(format!(
            "unexpected tag: {:?} vs {:?}",
            tag, rtag
        )))
    }
}

pub fn read_closure(rdr: &mut dyn Read) -> Result<Object> {
    let file_tag = rdr.read_u16::<LittleEndian>()?;
    match FromPrimitive::from_u16(file_tag) {
        Some(FileTags::BytecodeStreamTag) => (),
        _ => {
            return Err(Error::RuntimeError(
                "missing bytecode stream tag".to_string(),
            ))
        }
    };

    expect_tag(rdr, FileTags::ClosurestreamHead)?;
    expect_tag(rdr, FileTags::SizeChar)?;
    expect_tag(rdr, FileTags::SizeInteger)?;
    expect_tag(rdr, FileTags::SizeFloat)?;
    let func_proto = read_funcproto(rdr)?;
    expect_tag(rdr, FileTags::ClosurestreamTail)?;

    let closure = object::Closure {
        func_proto: func_proto,
    };
    Ok(Object::Closure(Rc::new(closure)))
}

pub fn read_funcproto(rdr: &mut dyn Read) -> Result<Object> {
    expect_tag(rdr, FileTags::ClosurestreamPart)?;
    let source_name = read_object(rdr)?;
    let name = read_object(rdr)?;

    expect_tag(rdr, FileTags::ClosurestreamPart)?;

    let nliterals = rdr.read_u64::<LittleEndian>()?;
    let nparameters = rdr.read_u64::<LittleEndian>()?;
    let noutervalues = rdr.read_u64::<LittleEndian>()?;
    let nlocalvarinfos = rdr.read_u64::<LittleEndian>()?;
    let nlineinfos = rdr.read_u64::<LittleEndian>()?;
    let ndefaultparams = rdr.read_u64::<LittleEndian>()?;
    let ninstructions = rdr.read_u64::<LittleEndian>()?;
    let nfunctions = rdr.read_u64::<LittleEndian>()?;

    println!(
        "{} {} {} {} {} {} {} {}",
        nliterals,
        nparameters,
        noutervalues,
        nlocalvarinfos,
        nlineinfos,
        ndefaultparams,
        ninstructions,
        nfunctions
    );

    let obj = object::FuncProto {
        source_name: source_name,
        name: name,
    };
    expect_tag(rdr, FileTags::ClosurestreamPart)?;
    println!("literals: {}", nliterals);
    for i in 0..nliterals {
        let literal = read_object(rdr)?;
        println!("literal: {} {:?}", i, literal);
    }

    expect_tag(rdr, FileTags::ClosurestreamPart)?;
    println!("parameters: {}", nparameters);
    for i in 0..nparameters {
        let parameter = read_object(rdr)?;
        println!("parameter: {} {:?}", i, parameter);
    }

    expect_tag(rdr, FileTags::ClosurestreamPart)?;
    println!("outervalues: {}", noutervalues);
    for i in 0..noutervalues {
        let ovtype = rdr.read_u64::<LittleEndian>()?;
        let o = read_object(rdr)?;
        let name = read_object(rdr)?;

        println!("outervalue: {} {} {:?} {:?}", i, ovtype, o, name);
    }

    expect_tag(rdr, FileTags::ClosurestreamPart)?;
    println!("localvarinfos: {}", nlocalvarinfos);
    for i in 0..nlocalvarinfos {
        let name = read_object(rdr)?;
        let pos = rdr.read_u64::<LittleEndian>()?;
        let start_op = rdr.read_u64::<LittleEndian>()?;
        let end_op = rdr.read_u64::<LittleEndian>()?;

        println!("localvarinfos: {} {:?}", i, name);
    }

    expect_tag(rdr, FileTags::ClosurestreamPart)?;
    println!("lineinfos: {}", nlineinfos);
    for _i in 0..nlineinfos {
        let _line = rdr.read_u64::<LittleEndian>()?;
        let _op = rdr.read_u64::<LittleEndian>()?;
    }

    expect_tag(rdr, FileTags::ClosurestreamPart)?;
    println!("defaultpar: {}", ndefaultparams);
    for _i in 0..ndefaultparams {
        let _defaultparams = rdr.read_u64::<LittleEndian>()?;
    }

    expect_tag(rdr, FileTags::ClosurestreamPart)?;
    println!("instructions: {}", ninstructions);
    for _i in 0..ninstructions {
        let _arg1 = rdr.read_u32::<LittleEndian>()?;
        let mut buf = [0u8; 4];
        rdr.read(&mut buf)?;
    }

    expect_tag(rdr, FileTags::ClosurestreamPart)?;
    println!("functions: {}", nfunctions);
    for _i in 0..nfunctions {
        let _func = read_funcproto(rdr)?;
    }

    let stacksize = rdr.read_u64::<LittleEndian>()?;
    println!("stacksize: {}", stacksize);
    let mut bgenerator = [0u8; 1];
    println!("bgenerator: {}", bgenerator[0] as u32);

    rdr.read(&mut bgenerator)?;
    let varparams = rdr.read_u64::<LittleEndian>()?;
    println!("varparams: {}", varparams);

    // Ok(obj)
    Ok(Object::FuncProto(Rc::new(obj)))
}

#[cfg(test)]
mod tests {
    use super::read_closure;
    use super::Object;
    use std::io::Seek;

    fn read_cnut<R: std::io::Read + Seek>(rdr: &mut R) -> super::Result<Object> {
        let closure = read_closure(rdr);
        match closure {
            Ok(c) => Ok(c),
            Err(err) => {
                println!("reader pos: {:?}", rdr.seek(std::io::SeekFrom::Current(0)));
                Err(err)
            }
        }
    }

    #[test]
    fn load_closure() {
        let mut bc = &include_bytes!("out.cnut")[..];
        let closure = read_closure(&mut bc).unwrap();
        println!("{:?}", closure);
        // assert!(false);
        if let Object::Closure(closure) = &closure {
            if let Object::FuncProto(func_proto) = &closure.func_proto {
                assert_eq!(
                    format!("{:?}", func_proto.source_name),
                    "String(\"factorial.nut\")",
                );
                assert_eq!(format!("{:?}", func_proto.name), "String(\"main\")");
            }
        }
    }
}
