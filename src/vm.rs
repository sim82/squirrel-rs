#![allow(dead_code)]
use crate::bytecode::{CompOp, Opcode};
use crate::{bytecode, object, types, Object};
use crate::{Error, Result};
use num_traits::FromPrimitive;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;

#[derive(Copy, Clone, Debug)]
struct StackFrame {
    base: types::Integer,
    top: types::Integer,
}

#[derive(Debug)]
struct Stack {
    stack: Vec<Object>,
    frame: StackFrame,
}

impl Display for Stack {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "stack base: {} top: {}", self.frame.base, self.frame.top)?;
        for i in self.frame.base..self.frame.top {
            writeln!(f, "{}: {:?}", i, self.stack[i as usize])?;
        }
        Ok(())
    }
}

impl Stack {
    fn new() -> Stack {
        Stack {
            stack: vec![Object::Null; 1024],
            frame: StackFrame { base: 1, top: 1 },
        }
    }

    fn up(&mut self, pos: isize) -> &mut Object {
        &mut self.stack[(self.frame.top as isize + pos) as usize]
    }
    fn top(&mut self) -> &mut Object {
        // &mut self.stack[self.frame.top - 1]
        self.up(-1)
    }

    fn value(&self, pos: types::Integer) -> &Object {
        &self.stack[(self.frame.base + pos) as usize]
    }

    fn value_mut(&mut self, pos: types::Integer) -> &mut Object {
        &mut self.stack[(self.frame.base + pos) as usize]
    }

    fn get_frame(&self) -> StackFrame {
        self.frame.clone()
    }
    fn set_frame(&mut self, frame: StackFrame) {
        self.frame = frame;
    }

    fn pop(&mut self, num: types::Integer) {
        self.frame.top -= num;
    }

    fn push(&mut self, obj: Object) {
        self.stack[self.frame.top as usize] = obj;
        self.frame.top += 1;
    }

    fn set_arg3(&mut self, instr: &bytecode::Instruction, value: Object) {
        *self.value_mut(instr.arg3 as types::Integer) = value;
    }
    fn set_target(&mut self, instr: &bytecode::Instruction, value: Object) {
        *self.value_mut(instr.arg0 as types::Integer) = value;
    }
    fn get_arg0(&self, instr: &bytecode::Instruction) -> &Object {
        self.value(instr.arg0 as types::Integer)
    }
    fn get_arg1(&self, instr: &bytecode::Instruction) -> &Object {
        self.value(instr.arg1 as types::Integer)
    }
    fn get_arg2(&self, instr: &bytecode::Instruction) -> &Object {
        self.value(instr.arg2 as types::Integer)
    }
    fn get_arg3(&self, instr: &bytecode::Instruction) -> &Object {
        self.value(instr.arg3 as types::Integer)
    }

    fn get_arg0_mut(&mut self, instr: &bytecode::Instruction) -> &mut Object {
        self.value_mut(instr.arg0 as types::Integer)
    }
    fn get_arg1_mut(&mut self, instr: &bytecode::Instruction) -> &mut Object {
        self.value_mut(instr.arg1 as types::Integer)
    }
    fn get_arg2_mut(&mut self, instr: &bytecode::Instruction) -> &mut Object {
        self.value_mut(instr.arg2 as types::Integer)
    }
    fn get_arg3_mut(&mut self, instr: &bytecode::Instruction) -> &mut Object {
        self.value_mut(instr.arg3 as types::Integer)
    }

    fn print_compact(&self) {
        for i in 0..=self.frame.top {
            let extra = if i == self.frame.top {
                " <- top"
            } else if i == self.frame.base {
                " <- base"
            } else {
                ""
            };

            println!("{}: {}{}", i, self.stack[i as usize], extra);
        }
    }
}

#[derive(Clone)]
struct CallInfo {
    prevframe: StackFrame,

    closure: Object,
    ip: types::Integer,
    root: bool,

    target: types::Integer,
}

struct Profiling {
    op_count: HashMap<bytecode::Opcode, usize>,
}

impl Profiling {
    fn new() -> Profiling {
        Profiling {
            op_count: HashMap::new(),
        }
    }
    fn instruction(&mut self, instr: &bytecode::Instruction) {
        if let Some(opcode) = <bytecode::Opcode as FromPrimitive>::from_u8(instr.opcode) {
            *self.op_count.entry(opcode).or_insert(0) += 1;
        }
    }
    fn print(&self) {
        for (key, val) in self.op_count.iter() {
            println!("{:?}: {}", key, val);
        }
    }
}

struct Executor {
    stack: Stack,
    callstack: Vec<CallInfo>,
    roottable: Object,
    profiling: Profiling,
}

#[derive(Debug)]
enum LoopState {
    Continue,
    LeaveFrame(Object),
    Call(Object, types::Integer, types::Integer, types::Integer),
}

macro_rules! arith {
    ($op:tt, $self:expr, $instr:expr) => {
        {
        let op1 = $self.stack.get_arg2($instr);
        let op2 = $self.stack.get_arg1($instr);

        let res = match (op1, op2) {
            (Object::Integer(int1), Object::Integer(int2)) => Object::Integer(*int1 $op *int2),
            _ => {
                return Err(Error::RuntimeError(format!(
                    "unhandled operands {:?} {:?}",
                    op1, op2
                )))
            }
        };
        $self.stack.set_target($instr, res);
        LoopState::Continue
    }};
}

impl Executor {
    fn new() -> Executor {
        Executor {
            stack: Stack::new(),
            callstack: Vec::new(),
            profiling: Profiling::new(),
            roottable: Object::Table(Rc::new(RefCell::new(object::Table::new()))),
        }
    }

    fn call(&mut self, num_params: types::Integer, retval: bool) -> Result<()> {
        let closure = self.stack.up(-((num_params + 1) as isize)).clone();
        self.start_call(
            closure,
            self.stack.frame.top - num_params,
            num_params,
            self.stack.frame.top - num_params,
        )?;
        self.callstack
            .last_mut()
            .ok_or_else(|| Error::RuntimeError("empty callstack".to_string()))?
            .root = true;

        self.stack.pop(num_params);
        Ok(())
        // } else {
        //     Err(Error::RuntimeError(format!(
        //         "expected closure. Found {:?}",
        //         self.stack.up(-((num_params + 1) as isize))
        //     )))
        // }
    }

    fn start_call(
        &mut self,
        closure: Object,
        target: types::Integer,
        num_params: types::Integer,
        stackbase: types::Integer,
    ) -> Result<()> {
        self.callstack.push(CallInfo {
            prevframe: self.stack.get_frame(),
            closure: closure.clone(),
            ip: 0,
            root: false,
            target: target,
        });

        let func = closure.closure()?.func_proto.func_proto()?;

        let newtop = stackbase + func.stacksize;
        self.stack.set_frame(StackFrame {
            base: stackbase,
            top: newtop,
        });

        Ok(())
    }

    fn execute(&mut self) -> Result<Object> {
        let mut ci = self
            .callstack
            .last_mut()
            .ok_or_else(|| Error::RuntimeError("callstack empty".to_string()))?;

        let mut func = ci.closure.closure()?.func_proto.func_proto()?;

        loop {
            let instr = &func.instructions[ci.ip as usize];
            ci.ip += 1;

            let opcode = <Opcode as num_traits::FromPrimitive>::from_u8(instr.opcode)
                .ok_or_else(|| Error::RuntimeError(format!("unhandled opcode: {:?}", instr)))?;

            self.profiling.instruction(instr);

            let state = match opcode {
                Opcode::LOADINT => {
                    // *self.stack.value_mut(instr.arg0 as types::Integer) =
                    self.stack
                        .set_target(instr, Object::Integer(instr.arg1 as types::Integer));

                    LoopState::Continue
                }
                Opcode::LOAD => {
                    *self.stack.value_mut(instr.arg0 as types::Integer) =
                        func.literals[instr.arg1 as usize].clone();
                    LoopState::Continue
                }
                Opcode::ADD => arith!(+,self, instr),
                Opcode::SUB => arith!(-,self, instr),
                Opcode::MUL => arith!(*,self, instr),
                Opcode::DIV => arith!(/,self, instr),
                Opcode::MOD => arith!(%,self, instr),

                Opcode::EQ => {
                    if instr.arg3 != 0 {
                        return Err(Error::RuntimeError(
                            "literal compare not implemented".to_string(),
                        ));
                    }

                    let op1 = self.stack.get_arg2(instr);
                    let op2 = self.stack.get_arg1(instr);

                    let res = match (op1, op2) {
                        (Object::Integer(int1), Object::Integer(int2)) => {
                            Object::Bool(*int1 == *int2)
                        }
                        _ => {
                            return Err(Error::RuntimeError(format!(
                                "unhandled operands {:?} {:?}",
                                op1, op2
                            )))
                        }
                    };
                    self.stack.set_target(instr, res);

                    LoopState::Continue
                }
                Opcode::JZ => {
                    let cond = self.stack.get_arg0(instr);
                    if let Object::Bool(b) = cond {
                        if !*b {
                            ci.ip += instr.arg1 as types::Integer;
                        }
                    } else {
                        return Err(Error::RuntimeError(format!(
                            "unsopported condition in JZ {:?}",
                            cond
                        )));
                    }
                    LoopState::Continue
                }
                Opcode::JMP => {
                    ci.ip += instr.arg1 as types::Integer;
                    // println!("JMP {} {} -> {}", instr.arg1 as types::Integer, o, ci.ip);
                    LoopState::Continue
                }
                Opcode::JCMP => {
                    let op1 = self.stack.get_arg2(instr);
                    let op2 = self.stack.get_arg0(instr);

                    let r = match (op1, op2) {
                        (Object::Integer(int1), Object::Integer(int2)) => {
                            if int1 == int2 {
                                0
                            } else if int1 < int2 {
                                -1
                            } else {
                                1
                            }
                        }
                        _ => {
                            return Err(Error::RuntimeError(format!(
                                "unhandled operands {:?} {:?}",
                                op1, op2
                            )))
                        }
                    };

                    let res = match <CompOp as FromPrimitive>::from_u8(instr.arg3) {
                        Some(CompOp::G) => Object::Bool(r > 0),
                        Some(CompOp::GE) => Object::Bool(r >= 0),
                        Some(CompOp::L) => Object::Bool(r < 0),
                        Some(CompOp::LE) => Object::Bool(r <= 0),
                        Some(CompOp::_3W) => Object::Integer(r),
                        _ => {
                            return Err(Error::RuntimeError(format!(
                                "unhandled comparison op {:?}",
                                instr.arg3 as isize,
                            )))
                        }
                    };

                    if let Object::Bool(b) = &res {
                        if !*b {
                            ci.ip += instr.arg1 as types::Integer;
                        }
                    } else {
                        return Err(Error::RuntimeError(format!(
                            "unsopported condition in JZ {:?}",
                            res
                        )));
                    }
                    // _GUARD(CMP_OP((CmpOP)arg3,STK(arg2),STK(arg0),temp_reg));
                    // if(IsFalse(temp_reg)) ci->_ip+=(sarg1);

                    LoopState::Continue
                }

                Opcode::CLOSURE => {
                    let new_func = func.functions[instr.arg1 as usize].clone();
                    let new_closure = object::Closure::new(new_func);
                    self.stack
                        .set_target(instr, Object::Closure(Rc::new(new_closure)));
                    LoopState::Continue
                    // if(!CLOSURE_OP(TARGET,fp->_functions[arg1]._unVal.pFunctionProto)) { SQ_THROW(); }
                }
                Opcode::NEWSLOT => {
                    // println!(
                    //     "newslot: {:?} {:?} {:?} {}",
                    //     self.stack.get_arg1(instr),
                    //     self.stack.get_arg2(instr),
                    //     self.stack.get_arg3(instr),
                    //     instr.arg0 as usize
                    // );

                    // println!("{} {} {}", instr.arg1, instr.arg2, instr.arg3);
                    // println!("arg1: {}\n{:?}", instr.arg1, self.stack);

                    // self.stack.print_compact();

                    let key = self.stack.get_arg2(instr).clone();
                    let value = self.stack.get_arg3(instr).clone();

                    let mut table = self.stack.get_arg1_mut(instr).table_mut()?;
                    table.map.insert(key, value);

                    LoopState::Continue
                    // NewSlotA(STK(arg1),STK(arg2),STK(arg3),(arg0&NEW_SLOT_ATTRIBUTES_FLAG) ? STK(arg2-1) : SQObjectPtr(),(arg0&NEW_SLOT_STATIC_FLAG)?true:false,false));
                }
                Opcode::PREPCALLK | Opcode::PREPCALL => {
                    let key = if opcode == Opcode::PREPCALLK {
                        func.literals[instr.arg1 as usize].clone()
                    } else {
                        self.stack.get_arg1(instr).clone()
                    };
                    let o = self.stack.get_arg2(instr).clone();
                    let table = o.table()?;

                    let tmp = table.map.get(&key).ok_or_else(|| {
                        Error::RuntimeError(format!("key {:?} not found in table", key))
                    })?;
                    self.stack.set_arg3(instr, o.clone());
                    self.stack.set_target(instr, tmp.clone());

                    self.stack.print_compact();
                    LoopState::Continue
                }
                Opcode::CALL => LoopState::Call(
                    self.stack.get_arg1(instr).clone(),
                    instr.arg0 as types::Integer,
                    instr.arg3 as types::Integer,
                    instr.arg2 as types::Integer,
                ),
                Opcode::RETURN => {
                    let retval = if instr.arg0 == 0xff {
                        Object::Null
                    } else {
                        self.stack.get_arg1(instr).clone()
                    };
                    println!("return: {}", instr.arg1);
                    self.stack.print_compact();
                    LoopState::LeaveFrame(retval)
                }
                _ => {
                    return Err(Error::RuntimeError(format!(
                        "unhandled opcode: {:?}",
                        instr
                    )))
                }
            };

            match state {
                LoopState::LeaveFrame(retval) => {
                    println!("LeaveFrame {:?} -> {}", retval, ci.target);
                    let root = ci.root;
                    if !root {
                        let target = ci.target;

                        self.callstack.pop();
                        ci = self
                            .callstack
                            .last_mut()
                            .ok_or_else(|| Error::RuntimeError("callstack empty".to_string()))?;

                        self.stack.set_frame(ci.prevframe);
                        *self.stack.value_mut(target) = retval;

                        self.stack.print_compact();

                        func = ci.closure.closure()?.func_proto.func_proto()?;
                    } else {
                        return Ok(retval);
                    }
                }
                LoopState::Call(closure, target, num_args, stack_inc) => {
                    // println!(
                    //     "call {} {} {} {}",
                    //     closure.type_name(),
                    //     target,
                    //     num_args,
                    //     stack_inc
                    // );
                    // self.stack.print_compact();
                    let new_base = self.stack.frame.base + stack_inc;
                    self.start_call(closure, target, num_args, new_base)?;
                    ci = self
                        .callstack
                        .last_mut()
                        .ok_or_else(|| Error::RuntimeError("callstack empty".to_string()))?;
                    func = ci.closure.closure()?.func_proto.func_proto()?;
                }
                _ => (),
            }
        }
    }

    fn push_roottable(&mut self) {
        self.stack.push(self.roottable.clone());
    }
}

#[cfg(test)]
mod tests {
    // use super::read_closure;
    use super::*;
    use crate::io::*;
    use std::io::Seek;

    #[test]
    fn load_closure() {
        let mut bc = &include_bytes!("out.cnut")[..];
        let closure = read_closure(&mut bc).unwrap();
        println!("{:?}", closure);
        // assert!(false);

        let mut exec = Executor::new();
        exec.stack.push(closure);
        exec.push_roottable();
        let mut num_args = 1;

        exec.stack.print_compact();

        exec.call(num_args, false).unwrap();
        let retval = exec.execute().unwrap();
        //let ret = exec.stack.pop();
        println!("{:?}", retval);
        exec.profiling.print();
        assert_eq!(retval.integer().unwrap(), 111)
    }
}
