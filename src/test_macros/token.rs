#[macro_export]
macro_rules! token {
    ($t:expr) => ({

        let tt = if $t == "\n" { crate::elements::TokenType::EOL } else {
            match crate::elements::TokenType::match_symbol($t) {
                Some(tt) => tt,
                None => match crate::elements::TokenType::match_keyword($t) {
                    Some(tt) => tt,
                    None => match $t.parse::<f32>() {
                        Ok(tt) => crate::elements::TokenType::Number(tt),
                        Err(_) => crate::elements::TokenType::Identifier($t.to_string())
                    }
                }
            }
        };
        crate::elements::Token::simple(tt)
    });
}