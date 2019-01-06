use scanner::Scanner;
use failure::Error;

use token::Token;
use grammar::gram::Gram;
use tokentype::TokenType;

use grammar::binary::Binary;

pub struct Tree<'a> {
    raw_code : &'a str,
    tokens : Vec<Vec<Gram>>,
}

impl<'a> Tree<'a> {
    pub fn from_scanner(scanner : Scanner<'a>) -> Result<Tree,Error> {
        let (raw_code,mut raw_tokens) = scanner.explode();

        let mut tokens : Vec<Vec<Gram>> = Vec::new();
        let mut sub_tokens : Vec<Gram> = Vec::new();

        // loops through the linear list of tokens and groups them into statements.
        loop {
            let token : Option<Token> = if raw_tokens.len() > 0 { Some(raw_tokens.remove(0)) } else { None }; 

            match token {
                None => break,
                Some(token) => {
                    match token.get_type() {
                        TokenType::EOL => {
                            if sub_tokens.len() > 0 { 
                                tokens.push(sub_tokens);
                                sub_tokens = Vec::new();
                            }
                        },
                        _ => {
                            sub_tokens.push(Gram::create(token));
                        }
                    }
                }
            }

        }

        // catches the last tokens if there are any.
        if sub_tokens.len() > 0 {
            tokens.push(sub_tokens);
        }

        let tree = Tree {
            raw_code : raw_code,
            tokens : tokens,
        };

        Ok(tree)
    }

    pub fn create_tree(mut self) -> Result<Self,Error> {
        for mut line in self.tokens.iter_mut() {
            Binary::process_set(&mut line)?;
        }

        for line in self.tokens.iter() {
            println!("====");
            for token in line.iter() {
                println!("{}",token);
            }
        }

        Ok(self)
    }
}
