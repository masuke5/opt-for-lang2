#![feature(box_syntax)]

mod ir;

use std::collections::{HashMap, HashSet};
use std::fmt;

use ir::{Expr, Stmt};
use maplit::hashset as hs;

struct FormatIter<'a, I, V>(I, &'a str)
where
    I: IntoIterator<Item = V> + Clone,
    V: fmt::Display;

impl<'a, I, V> fmt::Display for FormatIter<'a, I, V>
where
    I: IntoIterator<Item = V> + Clone,
    V: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(iter, delimiter) = self;

        let mut iter = iter.clone().into_iter();
        let first = iter.next();

        if let Some(first) = first {
            write!(f, "{}", first)?;
            for value in iter {
                write!(f, "{}{}", delimiter, value)?;
            }
        }

        Ok(())
    }
}

struct Optimizer {
    // 到達定義
    in_defs: Vec<HashSet<usize>>,
    out_defs: Vec<HashSet<usize>>,
    // 変数ごとの定義の集合
    defs: HashMap<isize, HashSet<usize>>,
    def_exprs: HashMap<usize, Expr>,
    code: Vec<Stmt>,
}

impl Optimizer {
    fn new(code: Vec<Stmt>) -> Self {
        Self {
            in_defs: vec![HashSet::new(); code.len()],
            out_defs: vec![HashSet::new(); code.len()],
            defs: HashMap::new(),
            def_exprs: HashMap::new(),
            code,
        }
    }

    fn optimize_expr(&self, i: usize, expr: Expr) -> Expr {
        match expr {
            Expr::LoadCopy(loc) => {
                let defs = match self.defs.get(&loc) {
                    Some(defs) => defs,
                    None => return Expr::LoadCopy(loc),
                };
                let in_defs = &self.in_defs[i]; // in[i]
                let reached_defs = defs & in_defs;

                if reached_defs.len() == 1 {
                    let only_one_def = reached_defs.into_iter().next().unwrap();
                    let expr = self.def_exprs[&only_one_def].clone();

                    if expr.is_const() {
                        Expr::Int(expr.to_value())
                    } else {
                        Expr::LoadCopy(loc)
                    }
                } else {
                    Expr::LoadCopy(loc)
                }
            }
            Expr::Add(lhs, rhs) => Expr::Add(
                box self.optimize_expr(i, *lhs),
                box self.optimize_expr(i, *rhs),
            ),
            Expr::Mul(lhs, rhs) => Expr::Mul(
                box self.optimize_expr(i, *lhs),
                box self.optimize_expr(i, *rhs),
            ),
            expr => expr,
        }
    }

    fn optimize_stmt(&self, i: usize, stmt: Stmt) -> Option<Stmt> {
        match stmt {
            Stmt::Store(loc, expr) => {
                let expr = self.optimize_expr(i, expr);
                Some(Stmt::Store(loc, expr))
            }
            Stmt::Expr(expr) => Some(Stmt::Expr(self.optimize_expr(i, expr))),
        }
    }

    fn optimize(mut self) -> Vec<Stmt> {
        for (i, ir) in self.code.iter().enumerate() {
            match ir {
                Stmt::Store(loc, expr) => {
                    self.defs.entry(*loc).or_insert(HashSet::new()).insert(i);
                    self.def_exprs.insert(i, expr.clone());
                }
                _ => {}
            }
        }

        loop {
            let prev_in = self.in_defs.clone();
            let prev_out = self.out_defs.clone();

            for i in 0..self.code.len() {
                if let Some(prev) = i.checked_sub(1) {
                    // in[i] = out[i - 1]
                    self.in_defs[i] = self.out_defs[prev].clone();
                }

                let (gen, kill) = match self.code[i] {
                    Stmt::Store(loc, _) => (hs!(i), &self.defs[&loc] - &hs!(i)),
                    _ => (hs!(), hs!()),
                };

                // out[i] = gen[i] U (in[i] - kill[i])
                self.out_defs[i] = &gen | &(&self.in_defs[i] - &kill);
            }

            if self.in_defs == prev_in && self.out_defs == prev_out {
                break;
            }
        }

        // 計算した到達定義を表示する
        for i in 0..self.code.len() {
            println!(
                "{:<3} {:<15} in={} out={}",
                i,
                format!("{}", self.code[i]),
                FormatIter(&self.in_defs[i], ","),
                FormatIter(&self.out_defs[i], ",")
            );
        }

        // 到達定義情報を元に最適化する
        // let mut stmts_should_remove = Vec::new();
        let mut new_code = Vec::with_capacity(self.code.len());
        for (i, stmt) in self.code.clone().into_iter().enumerate() {
            if let Some(stmt) = self.optimize_stmt(i, stmt) {
                new_code.push(stmt);
            }
        }

        new_code
    }
}

fn main() {
    use crate::ir::Expr::*;
    use Stmt::*;

    let code = vec![
        // v0 = 10
        Store(0, Int(10)),
        // v0 = 40
        Store(0, Int(40)),
        // v1 = a0
        Store(1, LoadCopy(-1)),
        // v2 = v0
        Store(2, LoadCopy(0)),
        // v2 + 20 * v1
        Expr(Add(box LoadCopy(2), box Mul(box Int(20), box LoadCopy(1)))),
    ];

    let optimizer = Optimizer::new(code);
    let optimized_code = optimizer.optimize();

    println!("----------------------------------------");

    // 最適化したコードを表示する
    for (i, stmt) in optimized_code.into_iter().enumerate() {
        println!("{:<3} {}", i, stmt);
    }
}
