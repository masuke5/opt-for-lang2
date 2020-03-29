#![feature(box_syntax)]

use opt_for_lang2::{ir, print_code, Optimizer};

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
        Jump(1),
    ];

    let optimizer = Optimizer::new(code);
    let optimized_code = optimizer.optimize();

    println!("----------------------------------------");

    print_code(&optimized_code);
}
