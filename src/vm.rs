use crate::bytecode::Opcode;
use crate::{object, types, Object};
use crate::{Error, Result};
use num_traits;
use std::rc::Rc;

#[derive(Copy, Clone)]
struct StackFrame {
    base: types::Integer,
    top: types::Integer,
}

struct Stack {
    stack: Vec<Object>,
    frame: StackFrame,
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
}

#[derive(Clone)]
struct CallInfo {
    prevframe: StackFrame,

    closure: Object,
    ip: types::Integer,
}

struct Executor {
    stack: Stack,
    callstack: Vec<CallInfo>,
}

impl Executor {
    fn new() -> Executor {
        Executor {
            stack: Stack::new(),
            callstack: Vec::new(),
        }
    }

    fn call(&mut self, num_params: types::Integer, retval: bool) -> Result<()> {
        let closure = self.stack.up(-((num_params + 1) as isize)).clone();
        self.start_call(closure, num_params, self.stack.frame.top - num_params);
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
        num_params: types::Integer,
        stackbase: types::Integer,
    ) -> Result<()> {
        self.callstack.push(CallInfo {
            prevframe: self.stack.get_frame(),
            closure: closure.clone(),
            ip: 0,
        });

        let func = closure.closure()?.func_proto.func_proto()?;

        let newtop = stackbase + func.stacksize;
        self.stack.set_frame(StackFrame {
            base: stackbase,
            top: newtop,
        });

        Ok(())
    }

    fn execute(&mut self) -> Result<()> {
        let ci = self
            .callstack
            .last_mut()
            .ok_or(Error::RuntimeError("callstack empty".to_string()))?;

        let func = ci.closure.closure()?.func_proto.func_proto()?;
        loop {
            let instr = &func.instructions[ci.ip as usize];
            ci.ip += 1;

            let opcode = <Opcode as num_traits::FromPrimitive>::from_u8(instr.opcode).ok_or(
                Error::RuntimeError(format!("unhandled opcode: {:?}", instr)),
            )?;

            match opcode {
                Opcode::LOADINT => {
                    *self.stack.value_mut(instr.arg0 as types::Integer) =
                        Object::Integer(instr.arg1 as types::Integer);
                }
                Opcode::ADD => {
                    // case _OP_ADD: _ARITH_(+,TARGET,STK(arg2),STK(arg1)); continue;

                    let op1 = self.stack.value(instr.arg2 as types::Integer);
                    let op2 = self.stack.value(instr.arg1 as types::Integer);

                    let res = match (op1, op2) {
                        (Object::Integer(int1), Object::Integer(int2)) => {
                            Object::Integer(*int1 + *int2)
                        }
                        _ => {
                            return Err(Error::RuntimeError(format!(
                                "unhandled operands {:?} {:?}",
                                op1, op2
                            )))
                        }
                    };
                    *self.stack.value_mut(instr.arg0 as types::Integer) = res;
                }
                _ => {
                    return Err(Error::RuntimeError(format!(
                        "unhandled opcode: {:?}",
                        instr
                    )))
                }
            }
        }
        Ok(())
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
    }
}
