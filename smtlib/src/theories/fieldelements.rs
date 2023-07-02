#![doc = concat!("```ignore\n", include_str!("./FieldElements.smt2"), "```")]

use smtlib_lowlevel::{
    ast::{self, Identifier, Term},
    lexicon::Symbol,
};

use crate::{
    impl_op,
    terms::{fun, qual_ident, Const, Dynamic, Sort},
    Bool,
};

/// A [`Int`] is a term containing a
/// [integer](https://en.wikipedia.org/wiki/Integer). You can [read more
/// here.](https://smtlib.cs.uiowa.edu/theories-Ints.shtml).
#[derive(Debug, Clone, Copy)]
pub struct FieldElement(&'static Term);
impl From<Const<FieldElement>> for FieldElement {
    fn from(c: Const<FieldElement>) -> Self {
        c.1
    }
}
impl std::fmt::Display for FieldElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Term::from(*self).fmt(f)
    }
}

impl From<FieldElement> for Dynamic {
    fn from(i: FieldElement) -> Self {
        Term::from(i).into()
    }
}

impl From<FieldElement> for Term {
    fn from(i: FieldElement) -> Self {
        i.0.clone()
    }
}
impl From<Term> for FieldElement {
    fn from(t: Term) -> Self {
        FieldElement(Box::leak(Box::new(t)))
    }
}
impl Sort for FieldElement {
    type Inner = Self;
    fn sort() -> ast::Sort {
        ast::Sort::Sort(Identifier::Simple(Symbol("FieldElement".into())))
    }
}
impl From<i64> for FieldElement {
    fn from(i: i64) -> Self {
        Term::Identifier(qual_ident(i.to_string(), None)).into()
    }
}
impl FieldElement {
    fn binop<T: From<Term>>(self, op: &str, other: FieldElement) -> T {
        fun(op, vec![self.into(), other.into()]).into()
    }
    /// Construct the term expressing `(> self other)`
    pub fn gt(self, other: impl Into<Self>) -> Bool {
        self.binop(">", other.into())
    }
    /// Construct the term expressing `(>= self other)`
    pub fn ge(self, other: impl Into<Self>) -> Bool {
        self.binop(">=", other.into())
    }
    /// Construct the term expressing `(< self other)`
    pub fn lt(self, other: impl Into<Self>) -> Bool {
        self.binop("<", other.into())
    }
    /// Construct the term expressing `(<= self other)`
    pub fn le(self, other: impl Into<Self>) -> Bool {
        self.binop("<=", other.into())
    }
}

impl std::ops::Neg for FieldElement {
    type Output = Self;
    fn neg(self) -> Self::Output {
        fun("-", vec![self.into()]).into()
    }
}

impl_op!(FieldElement, i64, Add, add, "+", AddAssign, add_assign, +);
impl_op!(FieldElement, i64, Sub, sub, "-", SubAssign, sub_assign, -);
impl_op!(FieldElement, i64, Mul, mul, "*", MulAssign, mul_assign, *);
impl_op!(FieldElement, i64, Div, div, "div", DivAssign, div_assign, /);
