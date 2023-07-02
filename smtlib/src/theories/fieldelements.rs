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
        ast::Sort::Sort(Identifier::Simple(Symbol("F".into())))
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

#[cfg(test)]
mod tests {
    use smtlib_lowlevel::backend::{Z3Binary, Z3Static};

    use crate::{terms::Sort, Solver};

    use super::FieldElement;
    use std::ops::Mul;

    #[test]
    fn finite_field_element_assertions() -> Result<(), Box<dyn std::error::Error>> {
        let a = FieldElement::from(2);
        let b = FieldElement::from(5);
        let c = FieldElement::from(0);

        let mut solver = Solver::new(Z3Static::new(&None)?)?;

        solver.assert(a.mul(b)._eq(c));
        let model = solver.check_sat_with_model()?.expect_sat()?;
        /* 
        solver.assert(a._eq(!d))?;
        solver.assert(b._eq(a.extract::<5, 2>()))?;
        solver.assert(c._eq(a.concat(b)))?;

        let model = solver.check_sat_with_model()?.expect_sat()?;

        let a: [bool; 6] = model.eval(a).unwrap().try_into()?;
        let b: [bool; 4] = model.eval(b).unwrap().try_into()?;
        let c: [bool; 10] = model.eval(c).unwrap().try_into()?;
        insta::assert_ron_snapshot!(a, @"(false, true, false, false, true, false)");
        insta::assert_ron_snapshot!(b, @"(false, true, false, false)");
        insta::assert_ron_snapshot!(c, @"(false, true, false, false, true, false, false, true, false, false)");
        */

        Ok(())
    }
}
