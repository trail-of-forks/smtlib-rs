#![doc = concat!("```ignore\n", include_str!("./FieldElements.smt2"), "```")]

use std::str::FromStr;

use smtlib_lowlevel::{
    ast::{self, Identifier, Term},
    lexicon::Symbol,
};

use crate::{
    impl_op,
    terms::{fun, qual_ident, Const, Dynamic, Sort},
};

use num_bigint::BigUint;
use regex::Regex;
use num_traits::Num;

/// A [`FieldElement`] is a term containing a
/// [finite field element](https://mathworld.wolfram.com/FiniteField.html). You can [read more
/// here.](https://docs.circom.io/background/background/#arithmetic-circuits).
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
impl From<BigUint> for FieldElement {
    fn from(i: BigUint) -> Self {
        Term::Identifier(qual_ident(format!("(as ff{i} F)"), None)).into()
    }
}
impl FieldElement {
    fn binop<T: From<Term>>(self, op: &str, other: FieldElement) -> T {
        fun(op, vec![self.into(), other.into()]).into()
    }

    // Turn a FieldElement into a BigUint
    pub fn to_biguint(&self) -> BigUint {
        // Remove irrelevant parts using a regex to match BigUints
        let re = Regex::new(r"[0-9]+").unwrap();
        let s = self.to_string();
        let relevant_part = re.find(&s.as_str()).unwrap().as_str();
        
        // Convert to BigUint
        let big_u = BigUint::from_str_radix(relevant_part, 10).expect("Invalid number");
        big_u
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
    use smtlib_lowlevel::backend::Cvc5Binary;

    use crate::{terms::Sort, Solver, SatResult};

    use super::FieldElement;
    use std::ops::{Mul, Neg, Add};

    #[test]
    fn finite_field_element_assertions() -> Result<(), Box<dyn std::error::Error>> {
        // Reimplement finite_field.smt2 example file tests
        // Use CVC5 solver
        let backend = Cvc5Binary::new("src/theories/cvc5")?;
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
        
        Ok(())
    }

    #[test]
    fn finite_field_operation_test() -> Result<(), Box<dyn std::error::Error>> {
        let backend = Cvc5Binary::new("src/theories/cvc5")?;
        // Use solver's exec method to set the sort of finite field
        // Find a way to incorporate this into the start (or maybe able to stick with driver being public exposed)
        let mut solver = Solver::new(backend)?;
        solver.set_logic(crate::Logic::QF_FF)?;
        // Let prime be 5
        let prime = BigUint::from(5u32);
        solver.set_field_order(&prime)?;

        // Now, test + - * operations
        let a = FieldElement::from_name("a");
        let b = FieldElement::from_name("b");
        let c = FieldElement::from_name("c");

        let one = FieldElement::from(1);
        let two = FieldElement::from(2);
        let three = FieldElement::from(3);

        solver.assert(one.neg()._eq(FieldElement::from(4)))?; 

        // 1 * 2 + 2 * 2 = 1 * 1
        solver.assert(one.mul(two).add(two.mul(two))._eq(one.mul(one)))?;

        // 1 * a + 2 * b - 3 * c = 0
        solver.assert(one.mul(a).add(two.mul(b)).add(FieldElement::neg(three.mul(c)))._eq(FieldElement::from(0)))?;

        let sat_result = solver.check_sat()?;
        println!("Debug sat {:?}", sat_result);
        let sat_string = format!("{:?}", sat_result);
        let sat_expected = format!("{:?}", SatResult::Sat);
        assert!(sat_expected == sat_string); 

        // Test getmodel
        let model = solver.get_model()?;
        println!("Debug model {:?}", model); 

        Ok(())
    }

    #[test]
    fn from_biguint_test() -> Result<(), Box<dyn std::error::Error>> {
        let backend = Cvc5Binary::new("src/theories/cvc5")?;
        // Use solver's exec method to set the sort of finite field
        // Find a way to incorporate this into the start (or maybe able to stick with driver being public exposed)
        let mut solver = Solver::new(backend)?;
        solver.set_logic(crate::Logic::QF_FF)?;
        // Let prime be 5
        let prime = BigUint::from(5u32);
        solver.set_field_order(&prime)?;

        let a = FieldElement::from(BigUint::from(5u32));
        let b = FieldElement::from(BigUint::from(5u32));
        solver.assert(a._eq(b))?;

        let sat_result = solver.check_sat()?;
        println!("Debug Sat {:?}", sat_result);
        let sat_string = format!("{:?}", sat_result);
        let sat_expected = format!("{:?}", SatResult::Sat);
        assert!(sat_expected == sat_string);
        Ok(())
    }

    #[test]
    /// Same as the test above, but testing conversion back and forth from FieldElement to BigUint and back
    fn parse_to_string_test() -> Result<(), Box<dyn std::error::Error>> {
        let backend = Cvc5Binary::new("src/theories/cvc5")?;
        // Use solver's exec method to set the sort of finite field
        // Find a way to incorporate this into the start (or maybe able to stick with driver being public exposed)
        let mut solver = Solver::new(backend)?;
        solver.set_logic(crate::Logic::QF_FF)?;
        // Let prime be 5
        let prime = BigUint::from(5u32);
        solver.set_field_order(&prime)?;

        let a = FieldElement::from(BigUint::from(5u32));
        let b = FieldElement::from(BigUint::from(5u32));
        solver.assert(a._eq(b))?;

        let parsed_a = FieldElement::to_biguint(&a);
        let parsed_b = FieldElement::to_biguint(&b);

        println!("Debug parsed_a {:?}", parsed_a);
        println!("Debug parsed_b {:?}", parsed_b);
        assert!(parsed_a == parsed_b);

        let sat_result = solver.check_sat()?;
        println!("Debug Sat {:?}", sat_result);
        let sat_string = format!("{:?}", sat_result);
        let sat_expected = format!("{:?}", SatResult::Sat);
        assert!(sat_expected == sat_string);
        Ok(())
    }
}
