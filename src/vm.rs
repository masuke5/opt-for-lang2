use crate::ir::{Expr, Stmt};
use std::collections::HashMap;

#[derive(Debug)]
#[non_exhaustive]
pub enum Inst {
    Int(i64),
    Add,
    Mul,
    Store(isize),
    LoadCopy(isize),
    Jump(usize),
    JumpIfZero(usize),
    Call(usize),
}

fn expr_to_insts(insts: &mut Vec<Inst>, expr: &Expr) {
    match expr {
        Expr::Int(n) => insts.push(Inst::Int(*n)),
        Expr::Add(lhs, rhs) => {
            expr_to_insts(insts, lhs);
            expr_to_insts(insts, rhs);
            insts.push(Inst::Add);
        }
        Expr::Mul(lhs, rhs) => {
            expr_to_insts(insts, lhs);
            expr_to_insts(insts, rhs);
            insts.push(Inst::Mul);
        }
        Expr::LoadCopy(loc) => insts.push(Inst::LoadCopy(*loc)),
    }
}

fn stmt_to_insts(insts: &mut Vec<Inst>, labels: &mut HashMap<usize, usize>, stmt: &Stmt) {
    match stmt {
        Stmt::Expr(expr) => expr_to_insts(insts, expr),
        Stmt::Store(loc, expr) => {
            expr_to_insts(insts, expr);
            insts.push(Inst::Store(*loc));
        }
        Stmt::Label(name) => {
            labels.insert(*name, insts.len());
        }
        Stmt::Jump(name) => {
            insts.push(Inst::Jump(*name));
        }
        Stmt::JumpIfZero(expr, name) => {
            expr_to_insts(insts, expr);
            insts.push(Inst::JumpIfZero(*name));
        }
        Stmt::Print(expr) => {
            expr_to_insts(insts, expr);
            insts.push(Inst::Call(0));
        }
    }
}

pub fn ir_to_insts(stmts: &[Stmt]) -> Vec<Inst> {
    let mut labels = HashMap::new();
    let mut insts = Vec::new();
    for stmt in stmts {
        stmt_to_insts(&mut insts, &mut labels, stmt);
    }

    for inst in &mut insts {
        match inst {
            Inst::Jump(loc) | Inst::JumpIfZero(loc) => {
                let label_loc = labels[loc];
                *loc = label_loc;
            }
            _ => {}
        }
    }

    insts
}

pub fn print_insts(insts: &[Inst]) {
    let width = format!("{}", insts.len()).len();

    for (i, inst) in insts.iter().enumerate() {
        print!("{:<width$}  ", i, width = width);
        match inst {
            Inst::Int(n) => println!("INT {}", n),
            Inst::Add => println!("ADD"),
            Inst::Mul => println!("MUL"),
            Inst::Store(loc) => println!("STORE {}", loc),
            Inst::LoadCopy(loc) => println!("LOAD_COPY {}", loc),
            Inst::Jump(loc) => println!("JUMP {}", loc),
            Inst::JumpIfZero(loc) => println!("JUMP_IF_ZERO {}", loc),
            Inst::Call(id) => match id {
                0 => println!("PRINT"),
                _ => println!("CALL {} (unknown)", id),
            },
        }
    }
}

const STACK_SIZE: usize = 500;
const MAX_VARIABLES: usize = 50;

pub struct VM {
    variables: [i64; MAX_VARIABLES],
    stack: [i64; STACK_SIZE],
}

impl VM {
    pub fn new() -> Self {
        Self {
            variables: [0; MAX_VARIABLES],
            stack: [0; STACK_SIZE],
        }
    }

    pub fn run(mut self, code: &[Inst]) {
        let mut ip = 0;
        let mut sp = 0;

        while ip < code.len() {
            match &code[ip] {
                Inst::Int(n) => {
                    sp += 1;
                    self.stack[sp] = *n;
                }
                Inst::Add => {
                    self.stack[sp - 1] += self.stack[sp];
                    sp -= 1;
                }
                Inst::Mul => {
                    self.stack[sp - 1] *= self.stack[sp];
                    sp -= 1;
                }
                Inst::Store(loc) => {
                    self.variables[*loc as usize] = self.stack[sp];
                    sp -= 1;
                }
                Inst::LoadCopy(loc) => {
                    sp += 1;
                    self.stack[sp] = self.variables[*loc as usize];
                }
                Inst::Jump(loc) => {
                    ip = *loc;
                    continue;
                }
                Inst::JumpIfZero(loc) => {
                    if self.stack[sp] == 0 {
                        ip = *loc;
                    }
                    sp -= 1;
                    continue;
                }
                Inst::Call(id) => match *id {
                    0 => {
                        let value = self.stack[sp];
                        sp -= 1;
                        println!("{}", value);
                    }
                    _ => panic!("Unknown function id: {}", *id),
                },
                // #[non_exhaustive]を指定しているのに警告が出る
                #[allow(unreachable_patterns)]
                inst => panic!("Unknown inst: {:?}", inst),
            }

            ip += 1;
        }
    }
}
