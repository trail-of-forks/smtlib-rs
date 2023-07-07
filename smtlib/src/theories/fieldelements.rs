#![doc = concat!("```ignore\n", include_str!("./FieldElements.smt2"), "```")]

use smtlib_lowlevel::{
    ast::{self, Identifier, Term, Command},
    lexicon::Symbol,
};

use crate::{
    impl_op,
    terms::{fun, qual_ident, Const, Dynamic, Sort},
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
        Term::Identifier(qual_ident(format!("(as ff{i} F)"), None)).into()
    }
}
impl FieldElement {
    fn binop<T: From<Term>>(self, op: &str, other: FieldElement) -> T {
        fun(op, vec![self.into(), other.into()]).into()
    }
    fn unop<T: From<Term>>(self, op: &str) -> T {
        fun(op, vec![self.into()]).into()
    }
}

impl std::ops::Neg for FieldElement {
    type Output = Self;
    fn neg(self) -> Self::Output {
        fun("ff.neg", vec![self.into()]).into()
    }
}

impl_op!(FieldElement, i64, Add, add, "ff.add", AddAssign, add_assign, +);
impl_op!(FieldElement, i64, Mul, mul, "ff.mul", MulAssign, mul_assign, *);

#[cfg(test)]
mod tests {
    use num_bigint::BigUint;
    use smtlib_lowlevel::{backend::{Cvc5Binary}, ast::{self, Command, Identifier, Term, SpecConstant}, lexicon::{Symbol,Numeral}};

    use crate::{terms::Sort, Solver, SatResult};

    use super::FieldElement;
    use std::ops::Mul;

    #[test]
    fn finite_field_element_assertions() -> Result<(), Box<dyn std::error::Error>> {
        // Reimplement finite_field.smt2 example file tests
        // Use CVC5 solver
        let mut backend = Cvc5Binary::new("src/theories/cvc5")?;
        // Use solver's exec method to set the sort of finite field
        // Find a way to incorporate this into the start (or maybe able to stick with driver being public exposed)
        let mut solver = Solver::new(backend)?;
        solver.set_logic(crate::Logic::QF_FF)?;
        // Let prime be 5
        let prime = BigUint::from(5u32);
        solver.set_field_order(&prime)?;
        
        // "a" and "b" are not constants, but fun
        let a = FieldElement::from_name("a");
        let b = FieldElement::from_name("b");
        let one = FieldElement::from(1);
        let two = FieldElement::from(2);

        // SAT
        solver.assert(a.mul(b)._eq(one))?;
        solver.assert(a._eq(two))?;
        
        // let model = solver.check_sat_with_model()?.expect_sat()?;
        let sat_result = solver.check_sat()?;
        println!("Debug sat {:?}", sat_result);
        let sat_string = format!("{:?}", sat_result);
        let sat_expected = format!("{:?}", SatResult::Sat);
        assert!(sat_expected == sat_string);    
    
        // Now, assert for UnSat
        solver.assert(b._eq(two))?;
        let sat_result = solver.check_sat()?;
        println!("Debug sat {:?}", sat_result);
        let sat_string = format!("{:?}", sat_result);
        let sat_expected = format!("{:?}", SatResult::Unsat);
        assert!(sat_expected == sat_string);  
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
