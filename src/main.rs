#![feature(box_syntax)]

use opt_for_lang2::{ir, print_code, Optimizer};

fn main() {
    use ir::{Expr::*, Stmt::*};

    let code = vec![
        // v0 = 10
        Store(0, Int(10)),
        // v0 = 40 + 5
        Store(0, Add(box Int(40), box Int(5))),
        // v1 = 90
        Store(1, Int(90)),
        // v2 = v0
        Store(2, LoadCopy(0)),
        // v2 + 20 * v1
        Expr(Add(box LoadCopy(2), box Mul(box Int(20), box LoadCopy(1)))),
    ];

    let optimizer = Optimizer::new(code);
    let optimized_code = optimizer.optimize();

    println!("----------------------------------------");

    print_code(&optimized_code);
}
