use squirrel_rs::io::read_closure;

use squirrel_rs::object;
use squirrel_rs::vm::Executor;
use squirrel_rs::Object;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::rc::Rc;

fn main() {
    if let Some(filename) = env::args().nth(1) {
        // println!("The first argument is {}", arg1);

        println!(
            "object size: {} {} {}",
            std::mem::size_of::<Rc<object::FuncProto>>(),
            std::mem::size_of::<squirrel_rs::Object>(),
            std::mem::size_of::<Box<str>>(),
        );
        let mut file = File::open(filename).unwrap();
        //        let mut bc = &include_bytes!("out.cnut")[..];
        let closure = read_closure(&mut file).unwrap();

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
        exec.add_native_func(
            "print",
            squirrel_rs::native_closure(
                Box::new(|stack| {
                    println!("print\n");
                    stack.print_compact("print");
                }),
                1,
            ),
        )
        .unwrap();
        // #[cfg(debug)]
        {
            exec.instr_profiling = false;
            exec.trace_call_return = false;
        }

        exec.stack().push(closure);
        exec.push_roottable();
        let mut num_args = 1;

        exec.stack().print_compact("initial");

        exec.call(num_args, false).unwrap();
        let ret = exec.execute();
        if ret.is_err() {
            exec.print_state();
        }

        let retval = ret.unwrap();
        //let ret = exec.stack.pop();
        println!("{:?}", retval);
        assert_eq!(retval.integer().unwrap(), 111)
    }
}
