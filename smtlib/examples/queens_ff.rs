extern crate itertools;
extern crate miette;
extern crate smtlib;

use itertools::Itertools;
use miette::IntoDiagnostic;
use smtlib::{
    and,
    backend::{Backend, Cvc5Binary, Z3Binary, Z3Static},
    distinct, or,
    terms::Sort,
    Logic, SatResultWithModel, Solver,
};
use smtlib::theories::fieldelements::FieldElement;


/* 
fn queens_ff<B: Backend>(backend: B) -> miette::Result<()> {
    let x0 = FieldElement::from_name("x0");
    let x1 = FieldElement::from_name("x1");
    let x2 = FieldElement::from_name("x2");
    let x3 = FieldElement::from_name("x3");
    let x4 = FieldElement::from_name("x4");
    let x5 = FieldElement::from_name("x5");
    let x6 = FieldElement::from_name("x6");
    let x7 = FieldElement::from_name("x7");
    let xs = [x0, x1, x2, x3, x4, x5, x6, x7];

    let n = FieldElement::from_name("N");

    let mut solver = Solver::new(backend)?;

    solver.set_logic(Logic::QF_FF)?;

    solver.assert(n._eq(8))?;

    solver.assert(distinct(xs))?;

    solver.assert(distinct([
        x0 - 0,
        x1 - 1,
        x2 - 2,
        x3 - 3,
        x4 - 4,
        x5 - 5,
        x6 - 6,
        x7 - 7,
    ]))?;

    for i in 1.. {
        match solver.check_sat_with_model()? {
            SatResultWithModel::Unsat => {
                eprintln!("No more solutions!");
                break;
            }
            SatResultWithModel::Sat(model) => {
                println!(
                    "{i:5}: {}",
                    xs.map(|x| model.eval(x).unwrap()).iter().format(",")
                );

                solver.assert(or(xs.map(|x| distinct([x.into(), model.eval(x).unwrap()]))))?;
            }
            SatResultWithModel::Unknown => todo!(),
        }
    }

    Ok(())
}
*/

fn main() -> miette::Result<()> {
    miette::set_panic_hook();
    Ok(())
}

    /* 
    match std::env::args().nth(1).as_deref().unwrap_or("cvc5") {
        "z3" => queens_ff(Z3Binary::new("z3").into_diagnostic()?)?,
        "z3-static" => queens_ff(Z3Static::new(&None).into_diagnostic()?)?,
        "cvc5" => queens_ff(Cvc5Binary::new("cvc5").into_diagnostic()?)?,
        given => miette::bail!(
            "Invalid backend: '{}'. Available backends are: 'z3', 'z3-static', 'cvc5'",
            given
        ),
    }

    Ok(())
}

*/