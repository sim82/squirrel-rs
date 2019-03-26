use crate::{object, Object};
use crate::{Error, Result};

#[derive(Copy, Clone)]
struct StackFrame {
    base: usize,
    top: usize,
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

    fn value(&mut self, pos: usize) -> &mut Object {
        &mut self.stack[self.frame.base + pos]
    }

    fn get_frame(&self) -> StackFrame {
        self.frame.clone()
    }
    fn set_frame(&mut self, frame: StackFrame) {
        self.frame = frame;
    }

    fn pop(&mut self, num: usize) {
        self.frame.top -= num;
    }

    fn push(&mut self, obj: Object) {
        self.stack[self.frame.top] = obj;
        self.frame.top += 1;
    }
}

struct CallInfo {}

struct Executor {
    stack: Stack,
}

impl Executor {
    fn new() -> Executor {
        Executor {
            stack: Stack::new(),
        }
    }

    fn call(&mut self, num_params: usize, retval: bool) -> Result<()> {
        if let Object::Closure(closure) = self.stack.up(-((num_params + 1) as isize)) {
            let closure = closure.clone();
            self.execute_closure(&closure, num_params, self.stack.frame.top - num_params);
            self.stack.pop(num_params);
            Ok(())
        } else {
            Err(Error::RuntimeError(format!(
                "expected closure. Found {:?}",
                self.stack.up(-((num_params + 1) as isize))
            )))
        }
    }

    fn execute_closure(
        &mut self,
        closure: &object::Closure,
        num_params: usize,
        stackbase: usize,
    ) -> Result<()> {
        if let Object::FuncProto(func) = &closure.func_proto {}
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
