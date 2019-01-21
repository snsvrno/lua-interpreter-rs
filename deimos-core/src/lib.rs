#[cfg(test)]
#[macro_use] mod test_crate;

use failure::Error;

mod elements;
mod scanner; use crate::scanner::Scanner;
mod parser; use crate::parser::Parser;
mod eval; use crate::eval::Eval;

pub fn evaluate(code : &str) -> Result<Eval,Error> {
    let scanner = Scanner::init(code).scan()?;
    let parser = Parser::from_scanner(scanner)?;
    let evaluated = Eval::from_parser(parser)?;
    Ok(evaluated)
}