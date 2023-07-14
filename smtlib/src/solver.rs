use std::collections::{hash_map::Entry, HashMap};

use smtlib_lowlevel::{
    ast::{self, Identifier, QualIdentifier},
    backend,
    lexicon::Symbol,
    Driver,
};

use num_bigint::BigUint;

use crate::{Bool, Error, Logic, Model, SatResult, SatResultWithModel};

/// The [`Solver`] type is the primary entrypoint to interaction with the
/// solver. Checking for validity of a set of assertions requires:
/// ```
/// # use smtlib::{Int, Sort};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // 1. Set up the backend (in this case z3)
/// let backend = smtlib::backend::Z3Binary::new("z3")?;
/// // 2. Set up the solver
/// let mut solver = smtlib::Solver::new(backend)?;
/// // 3. Declare the necessary constants
/// let x = Int::from_name("x");
/// // 4. Add assertions to the solver
/// solver.assert(x._eq(12))?;
/// // 5. Check for validity, and optionally construct a model
/// let sat_res = solver.check_sat_with_model()?;
/// // 6. In this case we expect sat, and thus want to extract the model
/// let model = sat_res.expect_sat()?;
/// // 7. Interpret the result by extract the values of constants which
/// //    satisfy the assertions.
/// match model.eval(x) {
///     Some(x) => println!("This is the value of x: {x}"),
///     None => panic!("Oh no! This should never happen, as x was part of an assert"),
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Solver<B> {
    pub driver: Driver<B>,
    decls: HashMap<Identifier, ast::Sort>,
}

impl<B> Solver<B>
where
    B: backend::Backend,
{
    /// Construct a new solver provided with the backend to use.
    ///
    /// The read more about which backends are available, check out the
    /// documentation of the [`backend`] module.
    pub fn new(backend: B) -> Result<Self, Error> {
        Ok(Self {
            driver: Driver::new(backend, false)?,
            decls: Default::default(),
        })
    }
    /// Explicitly sets the logic for the solver. For some backends this is not
    /// required, as they will infer what ever logic fits the current program.
    ///
    /// To read more about logics read the documentation of [`Logic`].
    pub fn set_logic(&mut self, logic: Logic) -> Result<(), Error> {
        let cmd = ast::Command::SetLogic(Symbol(logic.to_string()));
        println!("Setting logic to {logic}");
        match self.driver.exec(&cmd)? {
            ast::GeneralResponse::Success => Ok(()),
            ast::GeneralResponse::SpecificSuccessResponse(_) => panic!("Failed to set logic"),
            ast::GeneralResponse::Unsupported => panic!("Failed to set logic - Unsupported"),
            ast::GeneralResponse::Error(_) => panic!("Failed to set logic - Err"),
        }
    }

    /// Explicitly sets the field order for the solver in FiniteField theory. For some backends this is not
    /// required, as they will infer what ever logic fits the current program.
    ///
    /// To read more about logics read the documentation of [`Logic`].
    pub fn set_field_order(&mut self, prime: &BigUint) -> Result<(), Error> {
        let ff_sort = ast::Sort::Sort(Identifier::Simple(Symbol(format!("(_ FiniteField {prime})").into())));
        let sort_command = ast::Command::DefineSort(Symbol("F".into()), vec![], ff_sort);
        
        match self.driver.exec(&sort_command)? {
            ast::GeneralResponse::Success => Ok(()),
            ast::GeneralResponse::SpecificSuccessResponse(_) => panic!("Failed to set field order"),
            ast::GeneralResponse::Unsupported => panic!("Failed to set logic - Unsupported"),
            ast::GeneralResponse::Error(_) => panic!("Failed to set logic - Err"),
        }
    }

    /// Adds the constraint of `b` as an assertion to the solver. To check for
    /// satisfiability call [`Solver::check_sat`] or
    /// [`Solver::check_sat_with_model`].
    pub fn assert(&mut self, b: Bool) -> Result<(), Error> {
        let term = ast::Term::from(b);
        for q in term.all_consts() {
            match q {
                QualIdentifier::Identifier(_) => {}
                QualIdentifier::Sorted(i, s) => match self.decls.entry(i.clone()) {
                    Entry::Occupied(stored) => assert_eq!(s, stored.get()),
                    Entry::Vacant(v) => {
                        v.insert(s.clone());
                        match i {
                            Identifier::Simple(sym) => {
                                // Want this to be declare-fun
                                self.driver
                                    .exec(&ast::Command::DeclareConst(sym.clone(), s.clone()))?;
                            }
                            Identifier::Indexed(_, _) => todo!(),
                        }
                    }
                },
            }
        }
        let cmd = ast::Command::Assert(term);
        match self.driver.exec(&cmd)? {
            ast::GeneralResponse::Success => Ok(()),
            ast::GeneralResponse::Error(e) => Err(Error::Smt(e, cmd.to_string())),
            _ => todo!(),
        }
    }
    /// Checks for satisfiability of the assertions sent to the solver using
    /// [`Solver::assert`].
    ///
    /// If you are interested in producing a model satisfying the assertions
    /// check out [`Solver::check_sat`].
    pub fn check_sat(&mut self) -> Result<SatResult, Error> {
        let cmd = ast::Command::CheckSat;
        match self.driver.exec(&cmd)? {
            ast::GeneralResponse::SpecificSuccessResponse(
                ast::SpecificSuccessResponse::CheckSatResponse(res),
            ) => Ok(match res {
                ast::CheckSatResponse::Sat => SatResult::Sat,
                ast::CheckSatResponse::Unsat => SatResult::Unsat,
                ast::CheckSatResponse::Unknown => SatResult::Unknown,
            }),
            ast::GeneralResponse::Error(msg) => Err(Error::Smt(msg, format!("{cmd}"))),
            res => todo!("{res:?}"),
        }
    }
    /// Checks for satisfiability of the assertions sent to the solver using
    /// [`Solver::assert`], and produces a [model](Model) in case of `sat`.
    ///
    /// If you are not interested in the produced model, check out
    /// [`Solver::check_sat`].
    pub fn check_sat_with_model(&mut self) -> Result<SatResultWithModel, Error> {
        match self.check_sat()? {
            SatResult::Unsat => Ok(SatResultWithModel::Unsat),
            SatResult::Sat => Ok(SatResultWithModel::Sat(self.get_model()?)),
            SatResult::Unknown => Ok(SatResultWithModel::Unknown),
        }
    }
    /// Produces the model for satisfying the assertions. If you are looking to
    /// retrieve a model after calling [`Solver::check_sat`], consider using
    /// [`Solver::check_sat_with_model`] instead.
    ///
    /// > **NOTE:** This must only be called after having called
    /// > [`Solver::check_sat`] and it returning [`SatResult::Sat`].
    pub fn get_model(&mut self) -> Result<Model, Error> {
        match self.driver.exec(&ast::Command::GetModel)? {
            ast::GeneralResponse::SpecificSuccessResponse(
                ast::SpecificSuccessResponse::GetModelResponse(model),
            ) => Ok(Model::new(model)),
            res => todo!("{res:?}"),
        }
    }
}
