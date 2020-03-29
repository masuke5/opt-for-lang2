#![feature(box_syntax)]

use opt_for_lang2::{ir, ir_to_insts, print_code, print_insts, Optimizer, VM};

fn main() {
    use ir::{Expr::*, Stmt::*};

    let code = vec![
        Store(0, Int(30)),
        Jump(0),
        Label(1),
        Expr(Add(box LoadCopy(0), box LoadCopy(1))),
        Store(0, Int(5)),
        Label(0),
        Store(1, Int(50)),
        Print(LoadCopy(1)),
    ];

    /*
    let code = vec![
        Store(0, Int(10)),
        Store(0, Int(20)),
        Store(1, LoadCopy(0)),
        // Store(0, Int(20)),
        Store(2, Add(box LoadCopy(1), box Int(5))),
    ];
    */

    let optimizer = Optimizer::new(code);
    let code = optimizer.optimize();

    println!("----------------------------------------");

    print_code(&code);

    println!("------------------------------------");

    let insts = ir_to_insts(&code);
    print_insts(&insts);

    println!("------------------------------------");

    let vm = VM::new();
    vm.run(&insts);
}
