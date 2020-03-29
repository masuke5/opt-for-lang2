mod graph;
pub mod ir;

pub use graph::*;

use std::collections::{HashMap, HashSet};
use std::fmt;

use ir::{Expr, Stmt};
use maplit::hashset as hs;

use graph::DirectedGraph;

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
    code: DirectedGraph<Stmt>,
}

impl Optimizer {
    pub fn new(code: Vec<Stmt>) -> Self {
        Self {
            in_defs: vec![HashSet::new(); code.len()],
            out_defs: vec![HashSet::new(); code.len()],
            defs: HashMap::new(),
            def_exprs: HashMap::new(),
            code: code_to_graph(code),
        }
    }

    fn optimize_expr(&self, i: usize, expr: &mut Expr) {
        match expr {
            Expr::LoadCopy(loc) => {
                let defs = match self.defs.get(&loc) {
                    Some(defs) => defs,
                    None => return,
                };
                let in_defs = &self.in_defs[i];
                // iに到達したlocの定義
                let reached_defs = defs & in_defs;

                // 到達する定義が一つだけの場合
                if reached_defs.len() == 1 {
                    // 到達した唯一の定義とその式
                    let only_def = reached_defs.into_iter().next().unwrap();
                    let new_expr = self.def_exprs[&only_def].clone();

                    // 複写伝播
                    if let Expr::LoadCopy(loc) = new_expr {
                        // only_one_defからiに至るパスにlocの定義がなければ、locの複写に置き換える
                        let reached_defs = &self.defs[&loc] & &self.in_defs[only_def];
                        let defs = &(in_defs & &self.defs[&loc]) - &reached_defs;
                        if defs.is_empty() {
                            *expr = Expr::LoadCopy(loc);
                            return;
                        }
                    }

                    // 定数伝播
                    // 到達した唯一の定義の式が定数であれば、その式で置き換える
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
            Stmt::JumpIfZero(expr, _) => self.optimize_expr(i, expr),
            _ => {}
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
                self.in_defs[i] = self
                    .code
                    .pred_indexes(i)
                    .map(|index| &self.out_defs[index])
                    .fold(HashSet::new(), |acc, defs| &acc | defs);

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
        println!("-------------------");

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

        new_code.into_iter().collect()
    }
}

// コードを有向グラフに変換する
pub fn code_to_graph(code: Vec<Stmt>) -> DirectedGraph<Stmt> {
    let mut graph = DirectedGraph::new();
    let mut labels = HashMap::new();

    for stmt in code {
        if let Stmt::Label(name) = stmt {
            let index = graph.add(stmt);
            labels.insert(name, index);
        } else {
            graph.add(stmt);
        }
    }

    for index in 0..graph.len() {
        match &graph[index] {
            Stmt::Jump(name) | Stmt::JumpIfZero(_, name) => {
                let dest_index = labels[name];
                graph.add_edge(index, dest_index);
            }
            _ => {}
        }

        if let Some(prev) = index.checked_sub(1) {
            if let Stmt::Jump(_) = &graph[prev] {
            } else {
                graph.add_edge(prev, index);
            }
        }
    }

    graph
}

pub fn print_code(code: &[Stmt]) {
    for (i, stmt) in code.iter().enumerate() {
        println!("{:<3} {}", i, stmt);
    }
}
