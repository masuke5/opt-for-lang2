use std::fmt;

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
    Label(usize),
    Jump(usize),
    JumpIfZero(Expr, usize),
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Stmt::Store(loc, expr) => write!(f, "v{} <- {}", loc, expr),
            Stmt::Expr(expr) => write!(f, "{};", expr),
            Stmt::Label(name) => write!(f, "L{}:", name),
            Stmt::Jump(name) => write!(f, "jump L{}", name),
            Stmt::JumpIfZero(expr, name) => write!(f, "jump_if_zero {} -> L{}", expr, name),
        }
    }
}

impl fmt::Debug for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}
