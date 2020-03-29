use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};

pub static NEXT_LABEL: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Label(usize);

impl Label {
    pub fn new() -> Self {
        Self(NEXT_LABEL.fetch_add(1, Ordering::AcqRel))
    }

    pub fn as_usize(&self) -> usize {
        self.0
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum Expr {
    Int(i64),
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    LoadCopy(isize),
}

impl Expr {
    pub fn is_const(&self) -> bool {
        match self {
            Self::Int(_) => true,
            Self::Add(lhs, rhs) | Self::Mul(lhs, rhs) => lhs.is_const() && rhs.is_const(),
            _ => false,
        }
    }

    pub fn to_value(&self) -> i64 {
        match self {
            Self::Int(n) => *n,
            Self::Add(lhs, rhs) => lhs.to_value() + rhs.to_value(),
            Self::Mul(lhs, rhs) => lhs.to_value() * rhs.to_value(),
            _ => panic!("`{}` is not constant", self),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expr::Int(n) => write!(f, "{}", n),
            Expr::LoadCopy(loc) => write!(f, "v{}", loc),
            Expr::Add(lhs, rhs) => write!(f, "{} + {}", lhs, rhs),
            Expr::Mul(lhs, rhs) => write!(f, "{} * {}", lhs, rhs),
        }
    }
}

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum Stmt {
    Store(isize, Expr),
    Expr(Expr),
    Label(Label),
    Jump(Label),
    JumpIfZero(Expr, Label),
    Print(Expr),
}

impl Stmt {
    pub fn is_label(&self) -> bool {
        match self {
            Self::Label(_) => true,
            _ => false,
        }
    }

    pub fn is_jump(&self) -> bool {
        match self {
            Self::Jump(_) | Self::JumpIfZero(_, _) => true,
            _ => false,
        }
    }
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Stmt::Store(loc, expr) => write!(f, "v{} <- {}", loc, expr),
            Stmt::Expr(expr) => write!(f, "{};", expr),
            Stmt::Label(label) => write!(f, "L{}:", label.as_usize()),
            Stmt::Jump(label) => write!(f, "jump L{}", label.as_usize()),
            Stmt::JumpIfZero(expr, label) => {
                write!(f, "jump_if_zero {} -> L{}", expr, label.as_usize())
            }
            Stmt::Print(expr) => write!(f, "print ({})", expr),
        }
    }
}

impl fmt::Debug for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct BasicBlock {
    pub stmts: Vec<Stmt>,
}

impl BasicBlock {
    pub fn new() -> Self {
        Self { stmts: Vec::new() }
    }
}

pub fn stmts_to_bbs(stmts: Vec<Stmt>) -> Vec<BasicBlock> {
    let mut bbs: Vec<BasicBlock> = Vec::new();
    let mut curr_bb: Option<BasicBlock> = None;

    for stmt in stmts {
        if let Some(curr) = &mut curr_bb {
            let is_jump = stmt.is_jump();

            match stmt {
                Stmt::Label(label) if !is_jump => {
                    curr.stmts.push(Stmt::Jump(label));
                    bbs.push(curr_bb.take().unwrap());

                    let mut next_bb = BasicBlock::new();
                    next_bb.stmts.push(stmt);
                    curr_bb = Some(next_bb);
                }
                _ => {
                    curr.stmts.push(stmt);
                    if is_jump {
                        bbs.push(curr_bb.take().unwrap());
                    }
                }
            }
        } else {
            curr_bb = Some(BasicBlock::new());
            let curr_bb = curr_bb.as_mut().unwrap();

            if !stmt.is_label() {
                curr_bb.stmts.push(Stmt::Label(Label::new()));
            }

            curr_bb.stmts.push(stmt);
        }
    }

    if let Some(last_bb) = curr_bb.take() {
        bbs.push(last_bb);
    }

    bbs
}
