use crate::error::codeinfo::CodeInformation;
use crate::element::{Element, CodeElement};
use crate::scanner::Scanner;
use crate::token::{CodeToken, Token};
use crate::error::parser::ParserError;
use crate::coderef::CodeRef::CodeRef;

use failure::Error;

pub struct Parser<'a> {
    pub file_name : String,
    pub raw_code : &'a str, 
    pub blocks : Option<CodeElement>, 

    // private things
    tokens : Vec<CodeToken>,

}

impl<'a> CodeInformation for Parser<'a> {
    fn raw_code(&self) -> String { self.raw_code.to_string() }
    fn file_name(&self) -> String { self.file_name.to_string() }
}

impl<'a> std::default::Default for Parser<'a> {
    fn default() -> Parser<'a> {
        Parser {
            raw_code : "",
            file_name : String::from("buffer"),
            blocks : None,

            tokens : Vec::new(),
        }
    }
}

impl<'a> Parser<'a> {
    pub fn from_scanner(scanner : Scanner<'a>) -> Result<Parser<'a>,Error> {
        //! creates a parser object from a scanner object. this
        //! will consume the scanner.
        
        let parser = Parser {
            file_name : scanner.file_name,
            raw_code : scanner.raw_code,
            tokens : scanner.tokens,

            .. Parser::default()
        };

        parser.parse()
    }

    // PRIVATE FUNCTIONS /////////////////////////////////////
    //////////////////////////////////////////////////////////
    //////////////////////////////////////////////////////////
    //////////////////////////////////////////////////////////

    fn parse(mut self) -> Result<Parser<'a>, Error> {
        //! will attempt to parse the object


        // checks to see if we already assigned the blocks, if there 
        // is then something is wrong? you shouldn't be calling
        // this thing twice on the same object.
        if self.blocks.is_some() {
            return Err(ParserError::general("can't run parse more than once."));
        }

        let mut working_phrase : Vec<CodeElement> = Vec::new();

        loop {

            // the next statement of code, using LUA's statement rules
            match self.get_next_statement() {
                None => break,
                Some(mut statement) => {

                    // now we try and match that statement to something
                    // from the lua syntax

                    loop {

                        println!("=====");
                        for s in statement.iter() {
                            println!("{}:{}:{}",s,s.code_start(), s.code_end());
                        }

                        // stat ::=  varlist `=´ explist | 
                        if Parser::statement_assignment(&mut statement)? { continue; }
                        
                        // stat ::=  functioncall | 
                        // stat ::=  do block end | 
                        // stat ::=  while exp do block end | 
                        // stat ::=  repeat block until exp | 
                        // stat ::=  if exp then block {elseif exp then block} [else block] end | 
                        // stat ::=  for Name `=´ exp `,´ exp [`,´ exp] do block end | 
                        // stat ::=  for namelist in explist do block end | 
                        // stat ::=  function funcname funcbody | 
                        // stat ::=  local function Name funcbody | 
                        // stat ::=  local namelist [`=´ explist] 

                        // laststat ::= return [explist] | break
                        
                        // funcname ::= Name {`.´ Name} [`:´ Name]

                        // varlist ::= var {`,´ var}

                        // var ::=  Name | prefixexp `[´ exp `]´ | prefixexp `.´ Name 

                        // namelist ::= Name {`,´ Name}

                        // explist ::= {exp `,´} exp

                        // exp ::=  nil | false | true | Number | String | `...´ | function | prefixexp | tableconstructor | 
                        
                        // exp ::=  exp binop exp
                        if Parser::check_for_binop(&mut statement)? { continue; }

                        // exp ::=  unop exp

                        // prefixexp ::= var | functioncall | `(´ exp `)´

                        // functioncall ::=  prefixexp args | prefixexp `:´ Name args 
                        /*
                        args ::=  `(´ [explist] `)´ | tableconstructor | String 

                        function ::= function funcbody

                        funcbody ::= `(´ [parlist] `)´ block end

                        parlist ::= namelist [`,´ `...´] | `...´

                        tableconstructor ::= `{´ [fieldlist] `}´

                        fieldlist ::= field {fieldsep field} [fieldsep]
                        */

                        break;
                    }

                    // checks if we reduced it down to a single element, if so 
                    // then we can add it to the working_phrase and move on.
                    match statement.len() {
                        1 => working_phrase.push(statement.remove(0)),
                        0 => return Err(ParserError::general("parser found an empty statement?")),
                        _ => return Err(ParserError::not_a_statement(&self,
                            statement[0].line_number(), statement[0].code_start(),
                            statement[statement.len()-1].code_end()))
                    }
                }
            }
        }

        if working_phrase.len() == 0 {
            return Err(ParserError::general("parser found empty `working_phrase`?"))
        }

        // now we need to check if we can build a chunk and block out of the working_phrase
        for i in 0 .. working_phrase.len() - 1 {
            // checking all but the last element if it is a statement.
            if !working_phrase[i].i().is_statement() {
                return Err(ParserError::not_a_statement(&self,
                    working_phrase[i].line_number(), 
                    working_phrase[i].code_start(),
                    working_phrase[i].code_end()
                ));
            }
        }

        // assuming that all passed, lets check if the last elemnet is a statement or last_statemnet
        if !(working_phrase[working_phrase.len()-1].i().is_statement() 
            || working_phrase[working_phrase.len()-1].i().is_last_statement()
        ) {
            let pos = working_phrase.len()-1;
            return Err(ParserError::not_a_statement(&self,
                working_phrase[pos].line_number(),
                working_phrase[pos].code_start(),
                working_phrase[pos].code_end()
            ));
        }

        // everything check'd out, so lets build that chunk / block
        
        let chunk = {
            let code_start : usize = working_phrase[0].code_start();
            let line_number : usize = working_phrase[0].line_number();
            let code_end : usize = working_phrase[working_phrase.len()-1].code_end();

            CodeRef {
                item : Element::create(vec![], working_phrase)?, 
                code_start, 
                code_end,
                line_number,

            }
        };

        self.blocks = Some(chunk);

        Ok(self)
    }

    fn get_next_statement(&mut self) -> Option<Vec<CodeElement>> {
        //! gets the next state of tokens that makes as statement. there are a few
        //! cases where this won't be accurate (such as table definitions using ';')
        //! because it looks for EOL and ';' characters to draw the statement line

        let mut phrase : Vec<CodeElement> = Vec::new();
        loop {
            // makes sure we still have tokens to work with.
            if self.tokens.len() == 0 { break; }

            let token = self.tokens.remove(0);

            // we reached the end and have some tokens in the phrase,
            // lets send that back
            if (token == Token::EOL || token == Token::SemiColon) && phrase.len() > 0 { break; }
            // we reached a line break with nothing on it, but we
            // don't have anything to send back, we aren't necessarily
            // at the end of the token list (since that would break us
            // above) so lets just keep trying
            else if token == Token::EOL || token == Token::SemiColon || token == Token::WhiteSpace { continue; }
            // the default action, send it to the phrase
            else { 
                let new_element = Element::codeelement_from_token(token);
                phrase.push(new_element);
            }

        }

        match phrase.len() {
            0 => None,
            _ => Some(phrase)
        }
    }

    // checking functions
    
    fn check_for_binop(statement : &mut Vec<CodeElement>) -> Result<bool,Error> {
        if statement.len() >= 3 { for i in 0 .. statement.len() - 2 {
            // checks the standard format of `EXP (binop) EXP`

            if statement[i].i().is_exp() && statement[i+1].i().is_binop_token() && statement[i+2].i().is_exp() {
                // remove the pieces we care about
                let exp1 = statement.remove(i);
                let op = statement.remove(i);
                let exp2 = statement.remove(i);

                let code_start = exp1.code_start();
                let line_number = exp1.line_number();
                let code_end = exp2.code_end();

                let item = Element::create(vec![op],vec![exp1, exp2])?;

                statement.insert(i, CodeRef { item, code_end, code_start, line_number });

                return Ok(true);
            }
        }}

        Ok(false)
    }

    fn statement_assignment(statement: &mut Vec<CodeElement>) -> Result<bool,Error> {
        //! varlist `=´ explist

        if statement.len() == 3 {
            if let Some(ref token) = statement[1].i().get_token() {
                if token.i() == Token::Equal 
                && statement[0].i().is_var_list()
                && statement[2].i().is_exp_list() {
                    
                    let vars = statement.remove(0);
                    let op = statement.remove(0);
                    let exp = statement.remove(0);

                    let code_start = vars.code_start();
                    let code_end = exp.code_end();
                    let line_number = vars.line_number();

                    let new_element = Element::create(
                        vec![op],
                        vec![vars, exp])?;

                    statement.insert(0,CodeRef{
                        item : new_element,
                        code_start, code_end, line_number
                    });

                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

}

#[cfg(test)]
mod tests {

    #[test]
    // #[ignore]
    pub fn test_failure() {
        use crate::scanner::Scanner;
        use crate::parser::Parser;

        let code : &str = r#"
        x = 1 + 2
        "#;

        let scanner = Scanner::from_str(code,Some("testfile.lua")).unwrap();
        let parser = Parser::from_scanner(scanner);

        match parser {
            Ok(_) => { },
            Err(error) => { println!("{}",error); assert!(false); },
        }
    }

}