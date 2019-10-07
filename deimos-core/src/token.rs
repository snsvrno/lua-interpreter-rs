use crate::coderef::CodeRef;

pub type CodeToken = CodeRef<Token>; 

#[derive(Debug,PartialEq)]
pub enum Token {
    
    // single-character tokens /////////////////////
    Plus,           Minus,          Star,
    Slash,          Percent,        Carrot,
    Pound,          LessThan,       GreaterThan,
    Equal,          LeftParen,      RightParen,
    LeftMoustache,  RightMoustache, LeftBracket,
    RightBracket,   SemiColon,      Colon,
    Comma,          Period,

    // double-character tokens ////////////////////
    DoublePeriod,    EqualEqual,    NotEqual,
    GreaterEqual,    LessEqual,

    // triple-character tokens ////////////////////
    TriplePeriod,

    // keywords ///////////////////////////////////
    And,    Break,    Do,    Else,      Elseif,
    End,    False,    For,   Function,  If,
    In,     Local,    Nil,   Not,       Or,
    Repeat, Return,   Then,  True,      Until,
    While,

    // literals ///////////////////////////////////
    Identifier(String),    String(String),
    Number(f32),           MultiLineString(String),

    // other /////////////////////////////////////
    Comment(String),
    WhiteSpace,

    // special characters ////////////////////////
    EOL,
    EOF,
}

impl PartialEq<Token> for &Token {
    // implemented so i can compare variables to raw tokens
    // (static tokens) without doing `&Token::Do`
    fn eq(&self, other: &Token) -> bool {
        self == &other
    }
}

impl Token {

    pub fn len(&self) -> usize {
        match self {
            Token::Plus  |          Token::Minus  |         Token::Star |
            Token::Slash |          Token::Percent |        Token::Carrot |
            Token::Pound |          Token::LessThan |       Token::GreaterThan |
            Token::Equal |          Token::LeftParen |      Token::RightParen |
            Token::LeftMoustache |  Token::RightMoustache | Token::LeftBracket |
            Token::RightBracket |   Token::SemiColon |      Token::Colon |
            Token::Comma |          Token::Period |  Token::WhiteSpace
                => 1,
            Token::DoublePeriod |   Token::EqualEqual |     Token::NotEqual |
            Token::GreaterEqual |   Token::LessEqual |      Token::Do |
            Token::In |             Token::If |             Token::Or
                => 2,
            Token::TriplePeriod | Token::And | Token::End | Token::For | 
            Token::Nil | Token::Not
                => 3,
            Token::Else | Token::Then | Token::True 
                => 4,
            Token::Break | Token::Until | Token::While | Token::False | Token::Local 
                => 5,
            Token::Elseif | Token::Repeat | Token::Return
                => 6,
            Token::Function
                => 8,
            Token::EOL | Token::EOF 
                => 0,

            Token::Identifier(string) => string.len(),
            Token::String(string) => string.len() + 2,
            Token::Number(number) => format!("{}",number).len(),
            Token::MultiLineString(string) => string.len() + 2, // TODO : FIX THIS THING

            Token::Comment(string) => string.len(),

        }
    }

    pub fn is_eol(char : &str) -> bool {
        //! checks if the string is an end of line character
        
        match char {
            "\n" | "\r" => true,
            _ => false,
        }
    }

    pub fn is_whitespace(char : &str) -> bool {
        //! checks if the string a valid whitespace character
        //! this is kind of lie since we just mean empty space,
        //! so we are checking for spaces and tabs
        
        match char {
            " " => true,
            _ => false,
        }
    }

    pub fn is_valid_number_char(char : &str) -> bool {
        //! checks if the single length character 
        //! is a valid character that couild be in a number
        
        let allowable_ranges = vec![
            // (u start, u end, can start)
            (48,57), // 0-9
            (46,46), // .
        ];

        if char.len() == 1 {
            if let Some(c) = char.chars().next(){
                let code = c as u32;
                for range in allowable_ranges {
                    if range.0 <= code && code <= range.1 {
                        return true;
                    }
                }
            }
        }
        
        false
    }

    pub fn is_valid_word_char(char : &str, first : bool) -> bool {
        //! checks if the single length character 
        //! is a valid character that couild be in a word
        
        let allowable_ranges = vec![
            // (u start, u end, can start)
            (65,90,true), // A-Z
            (97,122,true), // a-z
            (48,57,false), // 0-9
            (95,95,true) // _
        ];

        if char.len() == 1 {
            if let Some(c) = char.chars().next(){
                let code = c as u32;
                for range in allowable_ranges {
                    if range.0 <= code && code <= range.1 {
                        if first && range.2 == false {
                            return false;
                        } else {
                            return true
                        }
                    }
                }
            }
        }
        
        false
    }

    pub fn match_keyword(word : &str) -> Option<Token> {
        //! list of all the fixed keywords in lua.
        
        match word {
            "and" => Some(Token::And),
            "break" => Some(Token::Break),
            "do" => Some(Token::Do),
            "else" => Some(Token::Else),
            "elseif" => Some(Token::Elseif),
            "end" => Some(Token::End),
            "false" => Some(Token::False),
            "for" => Some(Token::For),
            "function" => Some(Token::Function),
            "if" => Some(Token::If),
            "in" => Some(Token::In),
            "local" => Some(Token::Local),
            "nil" => Some(Token::Nil),
            "not" => Some(Token::Not),
            "or" => Some(Token::Or),
            "repeat" => Some(Token::Repeat),
            "return" => Some(Token::Return),
            "then" => Some(Token::Then),
            "true" => Some(Token::True),
            "until" => Some(Token::Until),
            "while" => Some(Token::While),
            _ => None,
        }
    }
}