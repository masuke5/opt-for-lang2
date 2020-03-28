mod graph;
pub mod ir;

pub use graph::*;

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

pub struct Optimizer {
    // 到達定義
    in_defs: Vec<HashSet<usize>>,
    out_defs: Vec<HashSet<usize>>,
    // 変数ごとの定義の集合
    defs: HashMap<isize, HashSet<usize>>,
    def_exprs: HashMap<usize, Expr>,
    code: Vec<Stmt>,
}

impl Optimizer {
    pub fn new(code: Vec<Stmt>) -> Self {
        Self {
            in_defs: vec![HashSet::new(); code.len()],
            out_defs: vec![HashSet::new(); code.len()],
            defs: HashMap::new(),
            def_exprs: HashMap::new(),
            code,
        }
    }

    fn optimize_expr(&self, i: usize, expr: &mut Expr) {
        match expr {
            Expr::LoadCopy(loc) => {
                let defs = match self.defs.get(&loc) {
                    Some(defs) => defs,
                    None => return,
                };
                let in_defs = &self.in_defs[i]; // in[i]
                let reached_defs = defs & in_defs;

                // 到達する定義が一つだけの場合
                if reached_defs.len() == 1 {
                    let only_one_def = reached_defs.into_iter().next().unwrap();
                    let new_expr = self.def_exprs[&only_one_def].clone();

                    // 定数伝播
                    if new_expr.is_const() {
                        *expr = Expr::Int(new_expr.to_value())
                    }
                }
            }
            Expr::Add(lhs, rhs) | Expr::Mul(lhs, rhs) => {
                self.optimize_expr(i, lhs);
                self.optimize_expr(i, rhs);

                if expr.is_const() {
                    *expr = Expr::Int(expr.to_value());
                }
            }
            _ => {}
        }
    }

    fn optimize_stmt(&self, i: usize, stmt: &mut Stmt) {
        match stmt {
            Stmt::Store(_, expr) => self.optimize_expr(i, expr),
            Stmt::Expr(expr) => self.optimize_expr(i, expr),
        }
    }

    fn calc_reaching_definition(&mut self) {
        // 変数ごとの定義の集合を計算
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
    }

    pub fn optimize(mut self) -> Vec<Stmt> {
        self.calc_reaching_definition();

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
        let mut new_code = self.code.clone();
        for (i, stmt) in new_code.iter_mut().enumerate() {
            self.optimize_stmt(i, stmt)
        }

        new_code
    }
}

pub fn print_code(code: &[Stmt]) {
    for (i, stmt) in code.iter().enumerate() {
        println!("{:<3} {}", i, stmt);
    }
}
