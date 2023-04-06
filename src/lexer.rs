use regex::Regex;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub text: &'a str,
    pub(crate) position: Position,
}
impl<'a> Token<'a> {
    pub(crate) fn new(kind: TokenKind, text: &str, line: usize, start: usize, length: usize) -> Token {
        Token {
            kind,
            text,
            position: Position::new(line, start, length),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TokenKind {
    Newline,
    LParen, RParen, LBrack, RBrack,
    Comma, FullStop,
    Plus, Minus, Mul, Div, Mod,
    Eq, Ne, Seq, Sne, Gt, Lt, Ge, Le,
    Assign,
    If, Elif, Else,
    While, For, In, Break, Continue,
    Funcdef, Return,
    BoolTrue, BoolFalse, Number, String,
    Name,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Position {
    pub line: usize,
    pub start: usize,
    pub length: usize,
}
impl Position {
    pub fn new(line: usize, start: usize, length: usize) -> Position {
        Position { line, start, length }
    }

    pub(crate) fn one_past(&self) -> Position {
        Position {
            line: self.line,
            start: self.start + self.length,
            length: 1,
        }
    }
}


pub struct Lexer<'a> {
    source: &'a str,
    token_vec: Vec<Token<'a>>,
    current_line: usize,
    current_column: usize,
    current_position: usize,
    current_token_length: usize,
    remaining_source: &'a str,
}
impl<'a> Lexer<'a> {
    pub fn new(source: &str) -> Lexer {
        Lexer {
            source,
            token_vec: Vec::new(),
            current_line: 1,
            current_column: 0,
            current_position: 0,
            current_token_length: 0,
            remaining_source: source,
        }
    }

    pub fn lex(&mut self) -> &Vec<Token> {
        use TokenKind::*;

        // whitespace
        let re_whitespace = Regex::new(r"^([ \t])+").unwrap();
        let re_newline = Regex::new(r"^\n").unwrap();
        // wrappers
        let re_lparen = Regex::new(r"^\(+").unwrap();
        let re_rparen = Regex::new(r"^\)+").unwrap();
        let re_lbrack = Regex::new(r"^\[+").unwrap();
        let re_rbrack = Regex::new(r"^]+").unwrap();
        // punctuation
        let re_comma = Regex::new(r"^,").unwrap();
        let re_full_stop = Regex::new(r"^\.").unwrap();
        // operators
        let re_plus = Regex::new(r"^\+").unwrap();
        let re_minus = Regex::new(r"^-").unwrap();
        let re_mul = Regex::new(r"^\*").unwrap();
        let re_div = Regex::new(r"^/").unwrap();
        let re_mod = Regex::new(r"^%").unwrap();
        let re_eq = Regex::new(r"^==").unwrap();
        let re_ne = Regex::new(r"^!=").unwrap();
        let re_seq = Regex::new(r"^===").unwrap();
        let re_sne = Regex::new(r"^!==").unwrap();
        let re_gt = Regex::new(r"^>").unwrap();
        let re_lt = Regex::new(r"^<").unwrap();
        let re_ge = Regex::new(r"^>=").unwrap();
        let re_le = Regex::new(r"^<=").unwrap();
        let re_assign = Regex::new(r"^=").unwrap();
        // name and datatypes
        let re_name = Regex::new(r"^[a-zA-Z_][a-zA-Z_0-9]*").unwrap();
        let re_bool_true = Regex::new(r"^rtue").unwrap();
        let re_bool_false = Regex::new(r"^flase").unwrap();
        let re_number = Regex::new(r"^[0-9]+(?:\.[0-9]+)?").unwrap();
        let re_string = Regex::new("^\"+[\\S\\s]+?\"+").unwrap();
        let re_empty_string_1 = Regex::new("^\"'").unwrap();
        let re_empty_string_2 = Regex::new("^'\"").unwrap();
        // control flow
        let re_if = Regex::new(r"^fi\s").unwrap();
        let re_elif = Regex::new(r"^lefi\s").unwrap();
        let re_else = Regex::new(r"^sele\s").unwrap();
        // loops
        let re_while = Regex::new(r"^whitl\s").unwrap();
        let re_for = Regex::new(r"^fir\s").unwrap();
        let re_in = Regex::new(r"^ni\s").unwrap();
        let re_break = Regex::new(r"^brek\s").unwrap();
        let re_continue = Regex::new(r"^cnotineu\s").unwrap();
        // functions
        let re_funcdef = Regex::new(r"^fnuc\s").unwrap();
        let re_return = Regex::new(r"^retrun\s").unwrap();


        // allows all the empty `else if`s below, necessary because they *must* be checked in that order
        #[allow(clippy::if_same_then_else)]
        while !self.remaining_source.is_empty() {
            // work out what the token at current_position is

            // whitespace
            if let Some(mat) = re_whitespace.find(self.remaining_source) {
                self.current_token_length = mat.end();
            }
            else if self.try_token_fixed_length(Newline, &re_newline, 1) {
                // as this is specifically a newline, increment the current line and reset the current
                // column count
                self.current_line += 1;
                self.current_column = 0;
                // additionally, set the current token length to zero to stop columns on the next line
                // getting thrown off in self.update_positions()
                self.current_token_length = 0;
                // because of the above adjustment, the current position has to be updated manually
                self.current_position += 1;
            }
            // brackets
            else if self.try_token_variable_length(LParen, &re_lparen) {}
            else if self.try_token_variable_length(RParen, &re_rparen) {}
            else if self.try_token_variable_length(LBrack, &re_lbrack) {}
            else if self.try_token_variable_length(RBrack, &re_rbrack) {}
            // misc. punctuation
            else if self.try_token_fixed_length(Comma, &re_comma, 1) {}
            else if self.try_token_fixed_length(FullStop, &re_full_stop, 1) {}
            // arithmetic operators
            else if self.try_token_fixed_length(Plus, &re_plus, 1) {}
            else if self.try_token_fixed_length(Minus, &re_minus, 1) {}
            else if self.try_token_fixed_length(Mul, &re_mul, 1) {}
            else if self.try_token_fixed_length(Div, &re_div, 1) {}
            else if self.try_token_fixed_length(Mod, &re_mod, 1) {}
            // relational operators
            else if self.try_token_fixed_length(Seq, &re_seq, 3) {}
            else if self.try_token_fixed_length(Sne, &re_sne, 3) {}
            else if self.try_token_fixed_length(Eq, &re_eq, 2) {}
            else if self.try_token_fixed_length(Ne, &re_ne, 2) {}
            else if self.try_token_fixed_length(Ge, &re_ge, 2) {}
            else if self.try_token_fixed_length(Le, &re_le, 2) {}
            else if self.try_token_fixed_length(Gt, &re_gt, 1) {}
            else if self.try_token_fixed_length(Lt, &re_lt, 1) {}
            // misc. operators
            else if self.try_token_fixed_length(Assign, &re_assign, 1) {}
            // keywords - control flow
            else if self.try_token_keyword(If, &re_if, "fi", 2) {}
            else if self.try_token_keyword(Elif, &re_elif, "lefi", 4) {}
            else if self.try_token_keyword(Else, &re_else, "sele", 4) {}
            // keywords - loops
            else if self.try_token_keyword(While, &re_while, "whitl", 5) {}
            else if self.try_token_keyword(For, &re_for, "fir", 3) {}
            else if self.try_token_keyword(In, &re_in, "ni", 2) {}
            else if self.try_token_keyword(Break, &re_break, "brek", 4) {}
            else if self.try_token_keyword(Continue, &re_continue, "cnotineu", 8) {}
            // keywords - functions
            else if self.try_token_keyword(Funcdef, &re_funcdef, "fnuc", 4) {}
            else if self.try_token_keyword(Return, &re_return, "retrun", 6) {}
            // datatypes
            else if self.try_token_keyword(BoolTrue, &re_bool_true, "rtue", 4) {}
            else if self.try_token_keyword(BoolFalse, &re_bool_false, "flase", 5) {}
            else if self.try_token_variable_length(Number, &re_number) {}
            else if self.try_token_fixed_length(String, &re_empty_string_1, 2) {}
            else if self.try_token_fixed_length(String, &re_empty_string_2, 2) {}
            else if self.try_token_variable_length(String, &re_string) {}
            // name
            else if self.try_token_variable_length(Name, &re_name) {}
            else {
                panic!()
            }

            self.update_positions();
        }

        &self.token_vec
    }

    fn try_token_fixed_length(&mut self, token: TokenKind, regex: &Regex, length: usize) -> bool {
        if let Some(mat) = regex.find(self.remaining_source) {
            self.push_token(token, mat.as_str(), length);
            true
        } else {
            false
        }
    }
    fn try_token_variable_length(&mut self, token: TokenKind, regex: &Regex) -> bool {
        if let Some(mat) = regex.find(self.remaining_source) {
            self.push_token(token, mat.as_str(), mat.end());
            true
        } else {
            false
        }
    }
    /// Since keywords only have special meanings when alone, a whitespace character is required to
    /// follow them. Since this throws off the newline parsing by prematurely consuming newlines,
    /// the length of this whitespace character is not included in the length of the token (i.e. the
    /// If token 'fi\s' has length 2 still) to prevent the lexer advancing too far too quickly. The
    /// extra character must be chopped off in the token text.
    /// To perform this, `try_token_keyword()` takes manual input of the text and length, rather
    /// than using the regex input to calculate it.
    fn try_token_keyword(&mut self,
                         token: TokenKind,
                         regex: &Regex,
                         token_text: &'a str,
                         length: usize)
                         -> bool
    {
        if regex.find(self.remaining_source).is_some() {
            self.push_token(token, token_text, length);
            true
        } else {
            false
        }
    }

    fn push_token(&mut self, token: TokenKind, token_text: &'a str, length: usize) {
        self.current_token_length = length;
        self.token_vec.push(Token::new(
            token,
            token_text,
            self.current_line,
            self.current_column,
            length,
        ))
    }
    fn update_positions(&mut self) {
        self.current_position += self.current_token_length;
        self.current_column += self.current_token_length;
        self.remaining_source = &self.source[self.current_position..];
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use TokenKind::*;

    #[test]
    fn newline() {
        assert_eq!(
            vec![
                Token::new(Newline, "\n", 1, 0, 1),
                Token::new(Newline, "\n", 2, 0, 1),
            ],
            *Lexer::new("\n\n").lex(),
        )
    }

    #[test]
    fn lparen() {
        assert_eq!(
            vec![
                Token::new(LParen, "((", 1, 0, 2),
                Token::new(LParen, "(", 1, 3, 1),
            ],
            *Lexer::new("(( (").lex(),
        )
    }
    #[test]
    fn rparen() {
        assert_eq!(
            vec![
                Token::new(RParen, "))", 1, 0, 2),
                Token::new(RParen, ")", 1, 3, 1),
            ],
            *Lexer::new(")) )").lex(),
        )
    }
    #[test]
    fn lbrack() {
        assert_eq!(
            vec![
                Token::new(LBrack, "[[", 1, 0, 2),
                Token::new(LBrack, "[", 1, 3, 1),
            ],
            *Lexer::new("[[ [").lex(),
        )
    }
    #[test]
    fn rbrack() {
        assert_eq!(
            vec![
                Token::new(RBrack, "]]", 1, 0, 2),
                Token::new(RBrack, "]", 1, 3, 1),
            ],
            *Lexer::new("]] ]").lex(),
        )
    }

    fn standard_symbol_test(token: TokenKind, token_text: &str, length: usize) {
        assert_eq!(
            vec![
                Token::new(token, token_text, 1, 0, length),
                Token::new(token, token_text, 1, length, length),
                Token::new(token, token_text, 1, 2*length + 1, length),
            ],
            *Lexer::new(&format!("{token_text}{token_text} {token_text} ")).lex(),
        )
    }
    /// Adapted symbol test, for when the standard symbol test doesn't work properly due to the
    /// repeat of a symbol being another symbol, e.g. 2 `=`s next to one another should be parsed as
    /// one `==`, but the standard symbol test will mark that incorrectly as a failure.
    fn adapted_symbol_test(token: TokenKind, token_text: &str, length: usize) {
        assert_eq!(
            vec![
                Token::new(token, token_text, 1, 0, length),
                Token::new(token, token_text, 1, length + 1, length),
            ],
            *Lexer::new(&format!("{token_text} {token_text} ")).lex(),
        )
    }

    #[test]
    fn comma() {
        standard_symbol_test(Comma, ",", 1);
    }
    #[test]
    fn full_stop() {
        standard_symbol_test(FullStop, ".", 1);
    }
    #[test]
    fn plus() {
        standard_symbol_test(Plus, "+", 1);
    }
    #[test]
    fn minus() {
        standard_symbol_test(Minus, "-", 1);
    }
    #[test]
    fn mul() {
        standard_symbol_test(Mul, "*", 1);
    }
    #[test]
    fn div() {
        standard_symbol_test(Div, "/", 1);
    }
    #[test]
    fn modulus() {
        standard_symbol_test(Mod, "%", 1);
    }
    #[test]
    fn eq() {
        adapted_symbol_test(Eq, "==", 2);
    }
    #[test]
    fn ne() {
        standard_symbol_test(Ne, "!=", 2);
    }
    #[test]
    fn gt() {
        standard_symbol_test(Gt, ">", 1);
    }
    #[test]
    fn lt() {
        standard_symbol_test(Lt, "<", 1);
    }
    #[test]
    fn ge() {
        standard_symbol_test(Ge, ">=", 2);
    }
    #[test]
    fn le() {
        standard_symbol_test(Le, "<=", 2);
    }
    #[test]
    fn seq() {
        standard_symbol_test(Seq, "===", 3);
    }
    #[test]
    fn sne() {
        standard_symbol_test(Sne, "!==", 3);
    }
    #[test]
    fn assign() {
        adapted_symbol_test(Assign, "=", 1);
    }

    #[test]
    fn name() {
        assert_eq!(
            vec![
                Token::new(Name, "m0r_nIngton_rul3z", 1, 0, 17),
                Token::new(Name, "_h3lloWorld", 1, 19, 11),
            ],
            *Lexer::new("m0r_nIngton_rul3z  _h3lloWorld").lex(),
        )
    }
    #[test]
    fn bool_true() {
        adapted_symbol_test(BoolTrue, "rtue", 4);
    }
    #[test]
    fn bool_false() {
        adapted_symbol_test(BoolFalse, "flase", 5);
    }
    #[test]
    fn number() {
        adapted_symbol_test(Number, "1", 1);
        adapted_symbol_test(Number, "12", 2);
        adapted_symbol_test(Number, "1.0", 3);
        adapted_symbol_test(Number, "4.234", 5);
    }
    #[test]
    fn string() {
        adapted_symbol_test(String, "\"Hello, Mornington!\"\"\"", 22);
        adapted_symbol_test(String, "\"\"\"Hello, Mornington!\"", 22);
    }
    #[test]
    fn empty_string_type_1() {
        adapted_symbol_test(String, "\"'", 2);
    }
    #[test]
    fn empty_string_type_2() {
        adapted_symbol_test(String, "'\"", 2);
    }

    #[test]
    fn if_keyword() {
        adapted_symbol_test(If, "fi", 2);
    }
    #[test]
    fn elif_statement() {
        adapted_symbol_test(Elif, "lefi", 4);
    }
    #[test]
    fn else_keyword() {
        adapted_symbol_test(Else, "sele", 4);
    }

    #[test]
    fn while_keyword() {
        adapted_symbol_test(While, "whitl", 5);
    }
    #[test]
    fn for_keyword() {
        adapted_symbol_test(For, "fir", 3);
    }
    #[test]
    fn in_keyword() {
        adapted_symbol_test(In, "ni", 2);
    }
    #[test]
    fn break_keyword() {
        adapted_symbol_test(Break, "brek", 4);
    }
    #[test]
    fn continue_keyword() {
        adapted_symbol_test(Continue, "cnotineu", 8);
    }
    #[test]
    fn funcdef_keyword() {
        adapted_symbol_test(Funcdef, "fnuc", 4);
    }
    #[test]
    fn return_keyword() {
        adapted_symbol_test(Return, "retrun", 6);
    }
}