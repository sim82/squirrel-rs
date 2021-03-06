#![allow(dead_code)]
use crate::bytecode::{AppendArrayType, CompOp, NewObjectType, Opcode};
use crate::{bytecode, object, types, Object};
use crate::{Error, Result};
use core::ops::Range;
use num_traits::FromPrimitive;
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;

#[derive(Copy, Clone, Debug)]
struct StackFrame {
    base: types::Integer,
    top: types::Integer,
}

#[derive(Debug)]
pub struct Stack {
    stack: Vec<RefCell<Object>>,
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
            stack: vec![RefCell::new(Object::Null); 1024 * 100],
            frame: StackFrame { base: 1, top: 1 },
        }
    }

    pub fn up(&mut self, pos: isize) -> RefMut<Object> {
        self.stack[(self.frame.top as isize + pos) as usize].borrow_mut()
    }
    pub fn top(&mut self) -> RefMut<Object> {
        // &mut self.stack[self.frame.top - 1]
        self.up(-1)
    }

    fn value(&self, pos: types::Integer) -> Ref<Object> {
        self.stack[(self.frame.base + pos) as usize].borrow()
    }

    fn value_mut(&mut self, pos: types::Integer) -> RefMut<Object> {
        self.stack[(self.frame.base + pos) as usize].borrow_mut()
        // self.stack
        //         .get_mut((self.frame.base + pos) as usize)
        // unsafe {
        //     self.stack
        //         .get_unchecked_mut((self.frame.base + pos) as usize)
        // }
    }
    fn swap(&mut self, pos1: types::Integer, pos2: types::Integer) {
        self.stack.swap(
            (pos1 + self.frame.base) as usize,
            (pos2 + self.frame.base) as usize,
        );
    }
    fn get_frame(&self) -> StackFrame {
        self.frame.clone()
    }
    fn set_frame(&mut self, frame: StackFrame) {
        self.frame = frame;
    }

    pub fn pop(&mut self, num: types::Integer) {
        self.frame.top -= num;
    }

    pub fn push(&mut self, obj: Object) {
        self.stack[self.frame.top as usize].swap(&mut RefCell::new(obj));
        self.frame.top += 1;
    }

    fn set_arg0(&mut self, instr: &bytecode::Instruction, value: Object) {
        *self.value_mut(instr.arg0 as types::Integer) = value;
    }
    fn set_arg1(&mut self, instr: &bytecode::Instruction, value: Object) {
        *self.value_mut(instr.arg1 as types::Integer) = value;
    }
    fn set_arg2(&mut self, instr: &bytecode::Instruction, value: Object) {
        *self.value_mut(instr.arg2 as types::Integer) = value;
    }
    fn set_arg3(&mut self, instr: &bytecode::Instruction, value: Object) {
        *self.value_mut(instr.arg3 as types::Integer) = value;
    }
    fn set_target(&mut self, instr: &bytecode::Instruction, value: Object) {
        self.set_arg0(instr, value);
    }
    fn get_arg0(&self, instr: &bytecode::Instruction) -> Ref<Object> {
        self.value(instr.arg0 as types::Integer)
    }
    fn get_arg1(&self, instr: &bytecode::Instruction) -> Ref<Object> {
        self.value(instr.arg1 as types::Integer)
    }
    fn get_arg2(&self, instr: &bytecode::Instruction) -> Ref<Object> {
        self.value(instr.arg2 as types::Integer)
    }
    fn get_arg3(&self, instr: &bytecode::Instruction) -> Ref<Object> {
        self.value(instr.arg3 as types::Integer)
    }

    fn get_arg0_mut(&mut self, instr: &bytecode::Instruction) -> RefMut<Object> {
        self.value_mut(instr.arg0 as types::Integer)
    }
    fn get_arg1_mut(&mut self, instr: &bytecode::Instruction) -> RefMut<Object> {
        self.value_mut(instr.arg1 as types::Integer)
    }
    fn get_arg2_mut(&mut self, instr: &bytecode::Instruction) -> RefMut<Object> {
        self.value_mut(instr.arg2 as types::Integer)
    }
    fn get_arg3_mut(&mut self, instr: &bytecode::Instruction) -> RefMut<Object> {
        self.value_mut(instr.arg3 as types::Integer)
    }

    pub fn print_compact(&self, info: &str) {
        println!(" --- {}", info);
        for i in 0..=self.frame.top {
            let extra = if i == self.frame.top {
                " <- top"
            } else if i == self.frame.base {
                " <- base"
            } else {
                ""
            };

            println!("{}: {}{}", i, self.stack[i as usize].borrow(), extra);
        }
        println!(" ---");
    }

    pub fn slice_mut(&mut self, r: Range<types::Integer>) -> &[RefCell<Object>] {
        &mut self.stack
            [((r.start + self.frame.base) as usize)..((r.end + self.frame.base) as usize)]
    }
}

#[derive(Clone)]
struct CallInfo {
    prevframe: StackFrame,

    closure: Object,
    ip: types::Integer,
    root: bool,

    target: Option<types::Integer>,
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

pub struct Executor {
    stack: Stack,
    callstack: Vec<CallInfo>,
    roottable: Object,
    profiling: Profiling,
    pub trace_call_return: bool,
    pub instr_profiling: bool,
}

#[derive(Debug)]
enum LoopState {
    Continue,
    LeaveFrame(Object),
    Call {
        closure: Object,
        target: Option<types::Integer>,
        num_args: types::Integer,
        stack_inc: types::Integer,
    },
    TailCall {
        closure: Object,
        num_args: types::Integer,
        arg_offset: types::Integer,
    },
}

macro_rules! arith {
    ($op:tt, $self:expr, $instr:expr) => {
        {
            let res = {
                let op1 = $self.stack.get_arg2($instr);
                let op2 = $self.stack.get_arg1($instr);

                match (&*op1, &*op2) {
                    (Object::Integer(int1), Object::Integer(int2)) => Object::Integer(int1 $op int2),
                    (Object::String(str1), _) => Object::new_string(&format!("{}{}",str1, op2)), // FIXME: this is crappy
                    // (Object::String(str1), _) => match "$op" {
                    //     "+" => Object::new_string(&format!("{}{}",str1, op2)),
                    //     _ => return Err(Error::RuntimeError(format!(
                    //         "unhandled operands {:?} {} {:?}",
                    //         op1, "$op", op2
                    //     ))),
                    // }
                    _ => {
                        return Err(Error::RuntimeError(format!(
                            "unhandled operands {:?} {:?}",
                            op1, op2
                        )))
                    }
                }
            };
        $self.stack.set_target($instr, res);
        LoopState::Continue
    }};
}

impl Executor {
    pub fn new() -> Executor {
        Executor {
            stack: Stack::new(),
            callstack: Vec::new(),
            profiling: Profiling::new(),
            roottable: Object::new_table(),
            trace_call_return: false,
            instr_profiling: false,
        }
    }
    pub fn stack(&mut self) -> &mut Stack {
        &mut self.stack
    }
    pub fn call(&mut self, num_params: types::Integer, retval: bool) -> Result<()> {
        let top = self.stack.frame.top;

        let closure = self.stack.up(-((num_params + 1) as isize)).clone();
        self.start_call(
            closure,
            Some(top - num_params),
            num_params,
            top - num_params,
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
        target: Option<types::Integer>,
        num_params: types::Integer,
        stackbase: types::Integer,
    ) -> Result<()> {
        let func = closure.closure_ref()?.func_proto.func_proto_ref()?;
        let newtop = stackbase + func.stacksize;

        self.callstack.push(CallInfo {
            prevframe: self.stack.get_frame(),
            closure: closure,
            ip: 0,
            root: false,
            target: target,
        });

        self.stack.set_frame(StackFrame {
            base: stackbase,
            top: newtop,
        });

        Ok(())
    }

    pub fn execute(&mut self) -> Result<Object> {
        let mut ci = self
            .callstack
            .last_mut()
            .ok_or_else(|| Error::RuntimeError("callstack empty".to_string()))?;

        let mut func = ci.closure.closure_ref()?.func_proto.func_proto()?;

        loop {
            let instr = &func.instructions[ci.ip as usize];
            ci.ip += 1;

            let opcode = <Opcode as num_traits::FromPrimitive>::from_u8(instr.opcode)
                .ok_or_else(|| Error::RuntimeError(format!("unhandled opcode: {:?}", instr)))?;

            if self.instr_profiling {
                self.profiling.instruction(instr);
            }
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
                Opcode::DLOAD => {
                    // ARGET = ci->_literals[arg1]; STK(arg2) = func.literals[arg3]

                    self.stack
                        .set_target(instr, func.literals[instr.arg1 as usize].clone());
                    self.stack
                        .set_arg2(instr, func.literals[instr.arg3 as usize].clone());
                    LoopState::Continue
                }
                Opcode::TYPEOF => {
                    // dest = SQString::Create(_ss(this),GetTypeName(obj1));
                    let name = Object::new_string(self.stack.get_arg1(instr).typesystem_name());
                    self.stack.set_target(instr, name);
                    LoopState::Continue
                }
                Opcode::MOVE => {
                    let src = self.stack.get_arg1(instr).clone();
                    self.stack.set_target(instr, src);
                    LoopState::Continue
                }
                Opcode::ADD => arith!(+,self, instr),
                Opcode::SUB => arith!(-,self, instr),
                Opcode::MUL => arith!(*,self, instr),
                Opcode::DIV => arith!(/,self, instr),
                Opcode::MOD => arith!(%,self, instr),

                Opcode::EQ => {
                    // if instr.arg3 != 0 {
                    //     return Err(Error::RuntimeError(
                    //         "literal compare not implemented".to_string(),
                    //     ));
                    // }
                    let res = {
                        let op1 = self.stack.get_arg2(instr);
                        let op2 = if instr.arg3 != 0 {
                            func.literals[instr.arg1 as usize].clone()
                        } else {
                            let x = self.stack.get_arg1(instr);
                            x.clone()
                        };
                        match (&*op1, &op2) {
                            (Object::Integer(int1), Object::Integer(int2)) => {
                                Object::Bool(*int1 == *int2)
                            }
                            (Object::String(op1), Object::String(op2)) => {
                                Object::Bool(*op1 == *op2)
                            }
                            _ => {
                                return Err(Error::RuntimeError(format!(
                                    "unhandled operands {} {}",
                                    op1, op2
                                )))
                            }
                        }
                    };
                    self.stack.set_target(instr, res);

                    LoopState::Continue
                }
                Opcode::JZ => {
                    let cond = self.stack.get_arg0(instr);
                    let b = match *cond {
                        Object::Bool(b) => b,
                        Object::Integer(i) => i != 0 as types::Integer,
                        Object::Null => false,
                        Object::String(_) => true,
                        _ => {
                            return Err(Error::RuntimeError(format!(
                                "unsopported condition in JZ {:?}",
                                cond
                            )))
                        }
                    };

                    if !b {
                        ci.ip += instr.arg1 as types::Integer;
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

                    let r = match (&*op1, &*op2) {
                        (Object::Integer(int1), Object::Integer(int2)) => {
                            if *int1 == *int2 {
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
                    let mut table = self.stack.get_arg1_mut(instr);
                    let mut table = table.table_mut()?;
                    table.map.insert(key, value);

                    LoopState::Continue
                }
                Opcode::PREPCALLK | Opcode::PREPCALL => {
                    // self.stack.print_compact(&format!("{:?} begin", opcode));
                    let key = if opcode == Opcode::PREPCALLK {
                        func.literals[instr.arg1 as usize].clone()
                    } else {
                        self.stack.get_arg1(instr).clone()
                    };
                    // let o = self.stack.get_arg2(instr).clone();
                    // {
                    //     println!("get {:?} {:?}", o, key);
                    //     let table = o.table()?;

                    //     let tmp = table.map.get(key).ok_or_else(|| {
                    //         Error::RuntimeError(format!("key {:?} not found in table", key))
                    //     })?;
                    //     self.stack.set_target(instr, tmp.clone());
                    // }
                    // self.stack.set_arg3(instr, o);
                    let obj = self.stack.get_arg2(instr).clone();
                    let res = get(&obj, &key)?;
                    self.stack.set_arg3(instr, obj);
                    self.stack.set_target(instr, res);
                    // self.stack.print_compact();
                    LoopState::Continue
                }
                Opcode::CALL => LoopState::Call {
                    closure: self.stack.get_arg1(instr).clone(),
                    target: if instr.arg0 != 255 {
                        Some(instr.arg0 as types::Integer)
                    } else {
                        None
                    },
                    num_args: instr.arg3 as types::Integer,
                    stack_inc: instr.arg2 as types::Integer,
                },
                Opcode::TAILCALL => LoopState::TailCall {
                    closure: self.stack.get_arg1(instr).clone(),
                    num_args: instr.arg3 as types::Integer,
                    arg_offset: instr.arg2 as types::Integer,
                },
                Opcode::RETURN => {
                    let retval = if instr.arg0 == 0xff {
                        Object::Null
                    } else {
                        self.stack.get_arg1(instr).clone()
                    };
                    // println!("return: {}", instr.arg1);
                    // self.stack.print_compact();
                    LoopState::LeaveFrame(retval)
                }
                Opcode::NEWOBJ => {
                    match <NewObjectType as FromPrimitive>::from_u8(instr.arg3) {
                        Some(NewObjectType::ARRAY) => {
                            self.stack
                                .set_target(instr, Object::new_array(instr.arg1 as types::Integer));
                        }
                        Some(NewObjectType::TABLE) => {
                            self.stack.set_target(instr, Object::new_table())
                        }
                        _ => {
                            return Err(Error::RuntimeError(format!(
                                "unhandled NEWOBJ type {:?}",
                                instr.arg3
                            )))
                        }
                    }
                    LoopState::Continue
                }
                Opcode::APPENDARRAY => {
                    let val = match <AppendArrayType as FromPrimitive>::from_u8(instr.arg2) {
                        Some(AppendArrayType::STACK) => self.stack.get_arg1(instr).clone(),
                        Some(AppendArrayType::LITERAL) => {
                            func.literals[instr.arg1 as usize].clone()
                        }
                        Some(AppendArrayType::INT) => Object::Integer(instr.arg1 as types::Integer),
                        // Some(AppendArrayType::FLOAT) => Object::Null,
                        Some(AppendArrayType::BOOL) => Object::Bool(instr.arg1 != 0),
                        _ => {
                            return Err(Error::RuntimeError(format!(
                                "unhandled APPENDARRAY type {:?}",
                                instr.arg2
                            )))
                        }
                    };
                    // self.stack.set_target(instr, val);
                    let mut array = self.stack.get_arg0_mut(instr);
                    array.array_mut()?.array.push(val);
                    LoopState::Continue
                }
                Opcode::LOADROOT => {
                    self.stack.set_target(instr, self.roottable.clone());
                    LoopState::Continue
                }
                Opcode::LOADNULLS => {
                    // self.stack.print_compact("before loadnulls");
                    let first = instr.arg0 as types::Integer;
                    let last = first + instr.arg1 as types::Integer;
                    for v in self.stack.slice_mut(first..last) {
                        v.swap(&RefCell::new(Object::Null));
                    }
                    // self.stack.print_compact("after loadnulls");

                    LoopState::Continue
                    // for(SQInt32 n=0; n < arg1; n++) STK(arg0+n).Null();
                }
                Opcode::FOREACH => {
                    let container = self.stack.get_arg0(instr).clone();
                    let outkey = instr.arg2 as types::Integer;
                    let outvalue = instr.arg2 as types::Integer + 1;
                    let index_pos = instr.arg2 as types::Integer + 2;
                    let index = match *self.stack.value(index_pos) {
                        Object::Null => 0,
                        Object::Integer(i) => i as usize,
                        _ => {
                            return Err(Error::RuntimeError(format!(
                                "unexpected iterator index: {:?}",
                                self.stack.value(instr.arg2 as types::Integer + 2),
                            )))
                        }
                    };
                    let exitpos = instr.arg1 as types::Integer;

                    match &container {
                        Object::Array(array) => {
                            if index < array.borrow().array.len() {
                                let out = array.borrow().array[index].clone(); // end borrowing array so we can modify the stack

                                *self.stack.value_mut(outkey) =
                                    Object::Integer(index as types::Integer);
                                *self.stack.value_mut(outvalue) = out;
                                *self.stack.value_mut(index_pos) =
                                    Object::Integer(index as types::Integer + 1);
                                ci.ip += 1;
                            } else {
                                ci.ip += exitpos; // exit loop
                            }
                        }
                        _ => {
                            return Err(Error::RuntimeError(format!(
                                "cannot iterate over {:?}",
                                container
                            )))
                        }
                    }

                    // STK(arg0),STK(arg2),STK(arg2+1),STK(arg2+2),arg2,sarg1,tojump

                    LoopState::Continue
                }
                Opcode::GETK => {
                    let key = &func.literals[instr.arg1 as usize];
                    let v = get(&*self.stack.get_arg2(instr), key)?;
                    self.stack.set_target(instr, v);
                    LoopState::Continue
                    // Get(STK(arg2), ci->_literals[arg1], temp_reg, 0,arg2)
                }
                Opcode::CLONE => {
                    let obj = self.stack.get_arg1(instr).clone_object()?;
                    // println!("clone: {:?}", obj);
                    self.stack.set_target(instr, obj);
                    LoopState::Continue
                }
                Opcode::DMOVE => {
                    let obj1 = self.stack.get_arg1(instr).clone();
                    self.stack.set_arg0(instr, obj1);
                    let obj3 = self.stack.get_arg3(instr).clone();
                    self.stack.set_arg2(instr, obj3);

                    LoopState::Continue
                }
                // Opcode::SET => {
                //     let obj = self.stack.get_arg1(instr);

                //     LoopState::Continue
                //     //  if (!Set(STK(arg1), STK(arg2), STK(arg3),arg1)) { SQ_THROW(); }
                //     // if (arg0 != 0xFF) TARGET = STK(arg3);
                // }
                _ => {
                    return Err(Error::RuntimeError(format!(
                        "unhandled opcode: {:?}",
                        instr
                    )))
                }
            };

            match state {
                LoopState::Call {
                    closure,
                    target,
                    num_args,
                    stack_inc,
                } => {
                    if self.trace_call_return {
                        println!(
                            "call {} {} {} {}",
                            closure,
                            match target {
                                Some(target) => format!("{}", self.stack.frame.base + target),
                                None => "none".into(),
                            },
                            num_args,
                            self.stack.frame.base + stack_inc
                        );
                        self.stack.print_compact("before call");
                    }
                    let new_base = self.stack.frame.base + stack_inc;

                    match closure {
                        Object::Closure(_) => {
                            self.start_call(closure, target, num_args, new_base)?;
                            ci = self.callstack.last_mut().ok_or_else(|| {
                                Error::RuntimeError("callstack empty".to_string())
                            })?;
                            func = ci.closure.closure_ref()?.func_proto.func_proto()?;
                        }
                        Object::NativeClosure(native_closure) => {
                            let new_top = new_base + native_closure.nargs;
                            let last_frame = self.stack.get_frame();
                            self.stack.set_frame(StackFrame {
                                base: new_base,
                                top: new_top,
                            });

                            (native_closure.func)(&mut self.stack);
                            self.stack.set_frame(last_frame);
                        }
                        _ => {
                            return Err(Error::RuntimeError(format!(
                                "expected Closure or NativeClosure. found {}",
                                closure
                            )))
                        }
                    }
                    if self.trace_call_return {
                        self.stack.print_compact("after call");
                    }
                }
                LoopState::TailCall {
                    closure,
                    num_args,
                    arg_offset,
                } => {
                    if self.trace_call_return {
                        println!("tailcall {} {} {}", closure, num_args, arg_offset);
                        self.stack.print_compact("before tailcall");
                    }

                    for i in 0..num_args {
                        // println!(
                        //     "{} <- {} {}",
                        //     i,
                        //     arg_offset + i,
                        //     self.stack.value(arg_offset + i),
                        // );
                        // *self.stack.value_mut(i) = self.stack.value(arg_offset + i).clone();
                        // self.stack.stack.swap(i as usize, (arg_offset + i) as usize);
                        self.stack.swap(i, arg_offset + i);
                    }

                    ci.closure = closure;
                    ci.ip = 0;
                    func = ci.closure.closure_ref()?.func_proto.func_proto()?;
                    if self.trace_call_return {
                        self.stack.print_compact("after tailcall");
                    }
                }
                LoopState::LeaveFrame(retval) => {
                    if self.trace_call_return {
                        match ci.target {
                            Some(target) => println!("LeaveFrame {:?} -> {}", retval, target),
                            None => println!("LeaveFrame noreturn"),
                        }
                    }
                    let root = ci.root;
                    if !root {
                        let target = ci.target;

                        if self.trace_call_return {
                            self.stack.print_compact("before return");
                        }
                        self.stack.set_frame(ci.prevframe);

                        self.callstack.pop();
                        ci = self
                            .callstack
                            .last_mut()
                            .ok_or_else(|| Error::RuntimeError("callstack empty".to_string()))?;

                        if let Some(target) = target {
                            *self.stack.value_mut(target) = retval;
                        }
                        if self.trace_call_return {
                            self.stack.print_compact("after return");
                        }

                        func = ci.closure.closure_ref()?.func_proto.func_proto()?;
                    } else {
                        return Ok(retval);
                    }
                }

                _ => (),
            }
        }

        self.profiling.print();
    }

    pub fn push_roottable(&mut self) {
        self.stack.push(self.roottable.clone());
    }
    pub fn print_state(&self) -> Result<()> {
        let ci = self
            .callstack
            .last()
            .ok_or_else(|| Error::RuntimeError("callstack empty".to_string()))?;

        let func = ci.closure.closure()?.func_proto.func_proto()?;
        println!(
            "function: {} {}\nip: {}",
            func.source_name, func.name, ci.ip
        );
        Ok(())
    }

    pub fn add_native_func(&mut self, name: &str, closure: Object) -> Result<()> {
        self.roottable
            .table_mut()?
            .map
            .insert(Object::String(name.into()), closure);
        Ok(())
    }
}

fn get(obj: &Object, key: &Object) -> Result<Object> {
    match obj {
        Object::Table(table) => table
            .borrow()
            .map
            .get(key)
            .cloned()
            .ok_or_else(|| Error::RuntimeError(format!("key {:?} not found in table", key))),

        Object::Array(array) => match key {
            Object::Integer(i) => array
                .borrow()
                .array
                .get(*i as usize)
                .cloned()
                .ok_or_else(|| Error::RuntimeError(format!("array access error {:?}", key))),

            Object::String(s) => match &**s {
                // hack array default delegation
                "len" => Ok(Object::Integer(array.borrow().array.len() as types::Integer)),
                _ => Err(Error::RuntimeError(format!(
                    "array delegate failed {:?}",
                    key
                ))),
            },

            _ => Err(Error::RuntimeError(format!(
                "unsupported array key {:?}",
                key
            ))),
        },
        _ => Err(Error::RuntimeError(format!(
            "key {:?} not found in table",
            key
        ))),
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

        closure
            .closure()
            .unwrap()
            .func_proto
            .func_proto()
            .unwrap()
            .print_disassembly("");
        // println!("{:?}", closure);
        // assert!(false);

        let mut exec = Executor::new();
        // #[cfg(debug)]
        {
            exec.instr_profiling = true;
        }

        exec.stack.push(closure);
        exec.push_roottable();
        let mut num_args = 1;

        exec.stack.print_compact("initial");

        exec.call(num_args, false).unwrap();
        let retval = exec.execute().unwrap();
        //let ret = exec.stack.pop();
        println!("{:?}", retval);
        exec.profiling.print();
        assert_eq!(retval.integer().unwrap(), 4091140000);
    }
}
