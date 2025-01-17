use crate::{ast::Script, lexicon::SmtlibParse, parse::Parser};

#[test]
fn escaped_string() {
    insta::assert_ron_snapshot!(String::parse(&mut Parser::new(
        r#""Hello ""world"" this is cool!""#
    )));
}

#[test]
fn bubble_sort() {
    insta::assert_ron_snapshot!(Script::parse(include_str!("../examples/bubble_sort.smt2")));
}

#[cfg(feature = "z3")]
mod z3 {
    use crate::{ast::Command, backend::Z3Binary, Driver};

    macro_rules! cmd {
        ($d:expr, $cmd:literal) => {
            insta::assert_ron_snapshot!($d.exec(&Command::parse($cmd)?)?);
        };
    }

    #[test]
    fn echo_test() -> Result<(), Box<dyn std::error::Error>> {
        let mut d = Driver::new(Z3Binary::new("z3")?, false)?;

        cmd!(d, r#"(echo "Hello, world!")"#);
        cmd!(d, r#"(echo "Hello, unmatched paren! :)")"#);

        Ok(())
    }
}

#[cfg(feature = "z3-static")]
mod z3_static {
    use crate::{ast::Command, backend::Z3Static, Driver};

    macro_rules! cmd {
        ($d:expr, $cmd:literal) => {
            insta::assert_ron_snapshot!($d.exec(&Command::parse($cmd)?)?);
        };
    }

    #[test]
    fn echo_test() -> Result<(), Box<dyn std::error::Error>> {
        // Use dummy timeout of 60s
        let timeout = Some(60);
        let mut d = Driver::new(Z3Static::new(&timeout)?, false)?;

        cmd!(d, r#"(echo "Hello, world!")"#);
        cmd!(d, r#"(echo "Hello, unmatched paren! :)")"#);

        Ok(())
    }
}
