use std::fmt;
#[derive(Clone)]
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
            // Self::Add(_, _) | Self::Mul(_, _) => true,
            _ => false,
        }
    }

    pub fn to_value(&self) -> i64 {
        match self {
            Self::Int(n) => *n,
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

#[derive(Clone)]
pub enum Stmt {
    Store(isize, Expr),
    Expr(Expr),
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Stmt::Store(loc, expr) => write!(f, "v{} = {}", loc, expr),
            Stmt::Expr(expr) => write!(f, "{};", expr),
        }
    }
}
