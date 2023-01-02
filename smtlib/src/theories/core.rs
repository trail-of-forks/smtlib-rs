#![doc = concat!("```ignore\n", include_str!("./Core.smt2"), "```")]

use smtlib_lowlevel::{
    ast::{self, Identifier, Term},
    lexicon::Symbol,
};

use crate::{
    impl_op,
    terms::{fun, qual_ident, Const, Dynamic, Sort},
};

/// A [`Bool`] is a term containing a
/// [boolean](https://en.wikipedia.org/wiki/Boolean_data_type). You can [read more
/// here.](https://smtlib.cs.uiowa.edu/theories-Core.shtml).
#[derive(Clone, Copy)]
pub struct Bool(BoolImpl);

impl std::fmt::Debug for Bool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Copy)]
enum BoolImpl {
    #[allow(unused)]
    Const(&'static str),
    Term(&'static Term),
}
impl From<Const<Bool>> for Bool {
    fn from(c: Const<Bool>) -> Self {
        c.1
    }
}
impl std::fmt::Display for Bool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Term::from(*self).fmt(f)
    }
}
impl From<Bool> for Dynamic {
    fn from(b: Bool) -> Self {
        Term::from(b).into()
    }
}
impl From<bool> for Bool {
    fn from(b: bool) -> Self {
        let s = match b {
            true => "true",
            false => "false",
        };

        Term::Identifier(qual_ident(s.into(), None)).into()
    }
}
impl From<Bool> for Term {
    fn from(b: Bool) -> Self {
        match b.0 {
            BoolImpl::Const(name) => Term::Identifier(qual_ident(name.into(), None)),
            BoolImpl::Term(t) => t.clone(),
        }
    }
}
impl From<Term> for Bool {
    fn from(t: Term) -> Self {
        Bool(BoolImpl::Term(Box::leak(Box::new(t))))
    }
}
impl Sort for Bool {
    type Inner = Self;
    fn sort() -> ast::Sort {
        ast::Sort::Sort(Identifier::Simple(Symbol("Bool".into())))
    }
}
impl Bool {
    fn binop(self, op: &str, other: Bool) -> Self {
        fun(op, vec![self.into(), other.into()]).into()
    }
    /// Construct the term expressing `(==> self other)`.
    ///
    /// The value of the returned boolean is true if:
    /// - `self` is false
    /// - or `other` is true
    pub fn implies(self, other: Bool) -> Bool {
        self.binop("=>", other)
    }
    /// Construct the term expressing `(ite self then otherwise)`.
    ///
    /// This is similar to the [ternary condition
    /// operator](https://en.wikipedia.org/wiki/Ternary_conditional_operator),
    /// and an if statement:
    /// - **C-style notation:** `self ? then : otherwise`
    /// - **Rust notation:**  `if self { then } else { otherwise }`
    pub fn ite(self, then: Bool, otherwise: Bool) -> Bool {
        fun("ite", vec![self.into(), then.into(), otherwise.into()]).into()
    }
}

impl_op!(Bool, bool, BitAnd, bitand, "and", BitAndAssign, bitand_assign, &);
impl_op!(Bool, bool, BitOr,  bitor,  "or", BitOrAssign,  bitor_assign,  |);
impl_op!(Bool, bool, BitXor, bitxor, "xor",  BitXorAssign, bitxor_assign, ^);

impl std::ops::Not for Bool {
    type Output = Bool;

    fn not(self) -> Self::Output {
        fun("not", vec![self.into()]).into()
    }
}

/// Construct the term expressing `(and ...terms)` representing the conjunction
/// of all of the terms. That is to say, the result is true iff all terms in
/// `terms` is true.
pub fn and<const N: usize>(terms: [Bool; N]) -> Bool {
    fun("and", terms.map(Into::into).to_vec()).into()
}
/// Construct the term expressing `(or ...terms)` representing the disjunction
/// of all of the terms. That is to say, the result is true iff any of the terms in
/// `terms` is true.
pub fn or<const N: usize>(terms: [Bool; N]) -> Bool {
    fun("or", terms.map(Into::into).to_vec()).into()
}
/// Construct the term expressing `(xor ...terms)`.
pub fn xor<const N: usize>(terms: [Bool; N]) -> Bool {
    fun("xor", terms.map(Into::into).to_vec()).into()
}

/// Construct the term expressing `(equal terms)` representing that all of the
/// terms in `terms` are equal.
pub fn equal<T, const N: usize>(terms: [T; N]) -> Bool
where
    T: Into<ast::Term>,
{
    fun("=", terms.map(Into::into).to_vec()).into()
}
/// Construct the term expressing `(distinct terms)` representing that all of the
/// terms in `terms` are distinct (or not-equal).
pub fn distinct<T, const N: usize>(terms: [T; N]) -> Bool
where
    T: Into<ast::Term>,
{
    fun("distinct", terms.map(Into::into).to_vec()).into()
}
