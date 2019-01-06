use tokentype::TokenType;
use token::Token;
use grammar::gram::Gram;
use grammar::expression::Expression;
use failure::Error;

#[derive(PartialEq,Clone,Debug)]
pub struct Binary {
    left_expr : Expression,
    operator : Token,
    right_expr : Expression,
}

impl Binary {

    // order of operation constants
    // taken from https://www.lua.org/pil/3.5.html
    const ORDER_TIER_1 : [TokenType; 1] = [ TokenType::Carrot ];
    const ORDER_TIER_3 : [TokenType; 2] = [ TokenType::Star, TokenType::Slash ];
    const ORDER_TIER_4 : [TokenType; 2] = [ TokenType::Plus, TokenType::Minus ];
    const ORDER_TIER_5 : [TokenType; 1] = [ TokenType::DoublePeriod ];
    const ORDER_TIER_6 : [TokenType; 6] = [ 
        TokenType::GreaterThan, TokenType::LessThan,
        TokenType::GreaterEqual, TokenType::LessEqual,
        TokenType::NotEqual, TokenType::EqualEqual
    ];
    const ORDER_TIER_7 : [TokenType; 1] = [ TokenType::And ];
    const ORDER_TIER_8 : [TokenType; 1] = [ TokenType::Or ];

    const OPERATION_ORDER : [ &'static [TokenType]; 7] = [
        &Binary::ORDER_TIER_1,
        &Binary::ORDER_TIER_3,
        &Binary::ORDER_TIER_4,
        &Binary::ORDER_TIER_5,
        &Binary::ORDER_TIER_6,
        &Binary::ORDER_TIER_7,
        &Binary::ORDER_TIER_8
    ];
    
    pub fn create_from(left_token : &Gram, operator: &Gram, right_token : &Gram) -> Option<Gram> {
        match (left_token, operator, right_token) {
            (Gram::Expression(left_expr), Gram::Token(token), Gram::Expression(right_expr)) => {
                match token.get_type() {
                    TokenType::Carrot |
                    TokenType::Star | 
                    TokenType::Slash | 
                    TokenType::Plus |
                    TokenType::Minus | 
                    TokenType::DoublePeriod |
                    TokenType::LessThan |
                    TokenType::GreaterThan |
                    TokenType::GreaterEqual |
                    TokenType::LessEqual |
                    TokenType::NotEqual |
                    TokenType::EqualEqual |
                    TokenType::And |
                    TokenType::Or => Some(Gram::Binary(Box::new(Binary{
                        left_expr : *left_expr.clone(),
                        operator : token.clone(),
                        right_expr : *right_expr.clone(),
                    }))),
                    _ => None,
                }
            }
            (_, _, _) => None,
        }
    }

    pub fn process_set(grams : &mut Vec<Gram>) -> Result<(),Error> {

        // needs at least Grams in order to match a binary, since the binary 
        // is 3 Expr (op) Expr, else it will just return.
        if grams.len() < 3 { return Ok(()); }

        // goes through the order of operations, for all operations
        let mut tier : Option<usize> = Some(0);
        loop {
            
            let ops = match tier {
                Some(t) => {
                    match Binary::OPERATION_ORDER.len() > t {
                        true => Binary::OPERATION_ORDER[t],
                        false => break,
                    }
                },
                None => return Err(format_err!("Tier is None!! Shouldn't have happened.")),
            };

            // decided to put a loop in here so once we get a match we will start 
            // over again with that operator in case we were chaining that operator
            // for example : 2 + 3 + 4 + 5, would ignore (2+3) + 4 because of the 
            // way the for loop works, and in a case where there was some other operation, 
            // it could possibly perform that grouping before causing the order to not
            // be correct.
            loop {

                // used to go through this loop again if we found a match.
                // the usize is the position of the matching set of Grams
                let mut reset_loop : Option<usize> = None;

                // get a group of 3 grams and check it against all of the operators in the group
                for i in 0 .. (grams.len()-2) {
                    // first we check if it matches the general patter for a binary,
                    // if the 1st and 3rd grams aren't expressions we move on to the next
                    // group of grams
                    if !grams[i].is_expression() || !grams[i+2].is_expression() { continue; }
                    
                    // goes through each operator
                    for op in ops.iter() {
                        if let Gram::Token(ref token) = grams[i+1] {
                            if token.get_type() == op {
                                // found a match!

                                // resetting the loop
                                reset_loop = Some(i);
                                break;
                            }
                        }
                    }

                    // continuing to break the loop from a positive operator match
                    if reset_loop.is_some() { break; }
                }

                // modifying the gram vec if we found a match in the above loop
                if let Some(i) = reset_loop {

                    // removing the 3 Grams and putting them in a format that can be used.
                    let mut removed_tokens : Vec<Gram> = grams.drain(i .. i + 3).collect();

                    let right : Gram = if let Some(gram) = removed_tokens.pop() { gram } else { 
                        return Err(format_err!("Failed to build Binary, tried to remove 1/3 Grams but failed.")); };
                    let middle : Gram = if let Some(gram) = removed_tokens.pop() { gram } else { 
                        return Err(format_err!("Failed to build Binary, tried to remove 2/3 Grams but failed.")); };
                    let left : Gram = if let Some(gram) = removed_tokens.pop() { gram } else { 
                        return Err(format_err!("Failed to build Binary, tried to remove 3/3 Grams but failed.")); };

                    // creates the new gram, needs to unwrap the pieces, they will error
                    // if somehow we got mismatched types, but this shouldn't happen
                    // because we previously check these when we were checking the operator.
                    let new_gram = Gram::Binary(Box::new(Binary{
                        left_expr : left.unwrap_expr()?,
                        operator : middle.unwrap_token()?,
                        right_expr : right.unwrap_expr()?,
                    }));

                    match Expression::create_into_gram(new_gram) {
                        None => return Err(format_err!("You shouldn't ever see this error!")), 
                        Some(expr_gram) => { grams.insert(i,expr_gram); }
                    }

                    // need to check if we have enough Grams to actually continue, if we get less than 3 there is 
                    // no way to match anything anymore so we should finish.
                    if grams.len() < 3 { return Ok(()); }

                    // counts as a reset for the tier, we need to do this because we just matched an operation,
                    // maybe there was another operation further up the stack that we didn't match because it
                    // couldn't have matched, and we would now miss it.
                    // example : 
                    // tier = None;

                } else {

                    // should be that we looked at all of the tokens and didn't find what we 
                    // were looking for, so lets move on. 
                    //
                    // we will only be here (and always be here) when the inner loop doesn't foind a match, meaning
                    // the reset_loop var will be none, and we will be in this part. This means we went through the
                    // inner loop completely and didn't find anything, so we should break and go to the next operator
                    // set (tier)
                    break;
                }
            }
            // increment the operator tier.
            tier = match tier {
                None => Some(0),
                Some(t) => Some(t+1),
            };
        }

        Ok(())
    }

/*
    fn collect_in_tier(tier : usize) -> Vec<&'static TokenType> {
        //! returns a list of operators in the desired tier,
        //! 
        //! use for order of operations.

        let mut tiers : Vec<&TokenType> = Vec::new();

        for (token,t) in Binary::operation_order.iter() {
            if t == &tier {
                tiers.push(&token);
            }
        }

        tiers
    }*/

}

impl std::fmt::Display for Binary {
    fn fmt(&self, f:&mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,"({} {} {})",self.operator,self.left_expr,self.right_expr)
    }
}

mod tests {

    #[test]
    fn basic_parsing() {
        use tokentype::TokenType;
        use token::Token;
        use grammar::binary::Binary;
        use grammar::gram::Gram;

        let exp1 = Gram::Token(Token::simple(TokenType::Nil)).to_literal().unwrap().to_expr().unwrap();
        let exp2 = Gram::Token(Token::simple(TokenType::String("what".to_string()))).to_literal().unwrap().to_expr().unwrap();

        let carrot = Gram::Token(Token::simple(TokenType::Carrot)); 
        let star = Gram::Token(Token::simple(TokenType::Star)); 
        let slash = Gram::Token(Token::simple(TokenType::Slash)); 
        let plus = Gram::Token(Token::simple(TokenType::Plus));
        let minus = Gram::Token(Token::simple(TokenType::Minus)); 
        let double_period = Gram::Token(Token::simple(TokenType::DoublePeriod));
        let less_than = Gram::Token(Token::simple(TokenType::LessThan));
        let greater_than = Gram::Token(Token::simple(TokenType::GreaterThan));
        let greater_equal = Gram::Token(Token::simple(TokenType::GreaterEqual));
        let less_equal = Gram::Token(Token::simple(TokenType::LessEqual));
        let not_equal = Gram::Token(Token::simple(TokenType::NotEqual));
        let equal_equal = Gram::Token(Token::simple(TokenType::EqualEqual));
        let and = Gram::Token(Token::simple(TokenType::And));
        let or = Gram::Token(Token::simple(TokenType::Or));

        assert!(Binary::create_from(&exp1, &carrot, &exp2).is_some());
        assert!(Binary::create_from(&exp1, &star, &exp2).is_some());
        assert!(Binary::create_from(&exp1, &slash, &exp2).is_some());
        assert!(Binary::create_from(&exp1, &or, &exp2).is_some());
        assert!(Binary::create_from(&exp1, &double_period, &exp2).is_some());
        assert!(Binary::create_from(&exp1, &plus, &exp2).is_some());
        assert!(Binary::create_from(&exp1, &minus, &exp2).is_some());
        assert!(Binary::create_from(&exp1, &less_than, &exp2).is_some());
        assert!(Binary::create_from(&exp1, &and, &exp2).is_some());
        assert!(Binary::create_from(&exp1, &equal_equal, &exp2).is_some());
        assert!(Binary::create_from(&exp1, &greater_equal, &exp2).is_some());
        assert!(Binary::create_from(&exp1, &greater_than, &exp2).is_some());
        assert!(Binary::create_from(&exp1, &less_equal, &exp2).is_some());
        assert!(Binary::create_from(&exp1, &not_equal, &exp2).is_some());

        let left_paren = Gram::Token(Token::simple(TokenType::LeftParen));
        let not = Gram::Token(Token::simple(TokenType::Not));
        assert!(Binary::create_from(&exp1, &left_paren, &exp2).is_none());
        assert!(Binary::create_from(&exp1, &not, &exp2).is_none());
    }

    #[test]
    fn order_of_operations() {
        use tokentype::TokenType;
        use token::Token;
        use grammar::binary::Binary;
        use grammar::gram::Gram;
        
        // 5 + 6 * 2 - 3
        // should do the correct order of operations and create something that looks like
        // 5 + (6 * 2) - 3
        // (5 + (6*2)) - 3
        // ((5+(6*2))-3)

        let mut tokens = vec![
            Gram::Token(Token::simple(TokenType::Number(5.0))).to_literal().unwrap().to_expr().unwrap(),
            Gram::Token(Token::simple(TokenType::Plus)),
            Gram::Token(Token::simple(TokenType::Number(6.0))).to_literal().unwrap().to_expr().unwrap(),
            Gram::Token(Token::simple(TokenType::Star)),
            Gram::Token(Token::simple(TokenType::Number(2.0))).to_literal().unwrap().to_expr().unwrap(),
            Gram::Token(Token::simple(TokenType::Minus)),
            Gram::Token(Token::simple(TokenType::Number(3.0))).to_literal().unwrap().to_expr().unwrap()
        ];

        if let Err(error) = Binary::process_set(&mut tokens) {
            panic!("ERROR : {}",error);
        }

        assert_eq!(1, tokens.len());
    }
}