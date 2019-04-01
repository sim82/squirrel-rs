use squirrel_rs::io::read_closure;

use squirrel_rs::vm::Executor;
use std::env;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    if let Some(filename) = env::args().nth(1) {
        // println!("The first argument is {}", arg1);

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
        // #[cfg(debug)]
        {
            exec.instr_profiling = true;
            exec.trace_call_return = true;
        }

        exec.stack().push(closure);
        exec.push_roottable();
        let mut num_args = 1;

        exec.stack().print_compact("initial");

        exec.call(num_args, false).unwrap();
        let retval = exec.execute().unwrap();
        //let ret = exec.stack.pop();
        println!("{:?}", retval);
        assert_eq!(retval.integer().unwrap(), 111)
    }
}
