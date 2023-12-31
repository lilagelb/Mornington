use crate::ast::*;
use crate::error::{Error, ErrorKind::*};
use crate::lexer::{Position, Token, TokenKind};
use crate::value::Value;

#[derive(Debug)]
pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    current_token: Option<Token<'a>>,
    previous_token: Option<Token<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(mut tokens: Vec<Token<'a>>) -> Parser<'a> {
        if tokens.is_empty() {
            panic!("No tokens passed, cannot initialise parser");
        }
        // reverse so that elements can easily and efficiently be popped off the end
        tokens.reverse();
        Parser {
            tokens,
            current_token: None,
            previous_token: None,
        }
    }

    fn parse_constant(token: &Token<'a>) -> Result<ConstantNode, Error> {
        match token.kind {
            TokenKind::Number => {
                Ok(ConstantNode::new(
                    Value::Number(token.text.parse::<f64>().unwrap()),
                ))
            },
            TokenKind::BoolTrue => {
                Ok(ConstantNode::new(
                    Value::Bool(true),
                ))
            },
            TokenKind::BoolFalse => {
                Ok(ConstantNode::new(
                    Value::Bool(false),
                ))
            },
            TokenKind::String => {
                // check for empty strings
                if token.text == "\"'" || token.text == "'\"" {
                    return Ok(ConstantNode::new( Value::String("".to_string()) ));
                }

                let total_length = token.text.len();
                let mut string_contents = token.text.trim_start_matches("\"");
                let opener_length = total_length - string_contents.len();
                string_contents = string_contents.trim_end_matches("\"");
                let closer_length = total_length - opener_length - string_contents.len();

                // check for quote imbalance, throw Balance error if quotes are balanced
                if opener_length == closer_length {
                    return Err(Error::new(
                        Balance {
                            opener: "\"".repeat(opener_length),
                            closer: "\"".repeat(closer_length),
                        },
                        Some(token.position)
                    ));
                }

                Ok(ConstantNode::new( Value::String(string_contents.to_string()) ))
            }
            _ => Err(Error::new(UnexpectedToken(token.kind), Some(token.position))),
        }
    }

    fn parse_list(&mut self, opener: &str) -> Result<ListNode, Error> {
        self.parse_comma_separated_expressions(opener, TokenKind::RBrack)
    }

    fn parse_function_arguments(&mut self, opener: &str) -> Result<ListNode, Error> {
        self.parse_comma_separated_expressions(opener, TokenKind::RParen)
    }

    fn parse_comma_separated_expressions(&mut self,
                                         opener: &str,
                                         closing_wrapper: TokenKind)
        -> Result<ListNode, Error>
    {
        let mut list: Vec<ExpressionNode> = Vec::new();

        // check for empty list eventuality
        match self.peek() {
            Some(token) => {
                if token.kind == closing_wrapper {
                    self.advance();
                    self.check_wrapper_balance(opener.to_string())?;
                    return Ok(ListNode::new(list));
                }
            },
            None => return Err(Error::new(
                UnexpectedEOF, Some(self.previous_token.unwrap().position.one_past())
            )),
        }

        loop {
            list.push(self.parse_expr(0)?);
            self.advance();
            if self.current_token.is_none() {
                return Err(Error::new(
                    UnexpectedEOF,
                    Some(self.previous_token.unwrap().position.one_past()),
                ))
            }
            match self.current_token.unwrap().kind {
                TokenKind::Comma => continue,
                other_token => {
                    if other_token == closing_wrapper {
                        self.check_wrapper_balance(opener.to_string())?;
                        break;
                    } else {
                        return Err(Error::new(
                            UnexpectedToken(other_token),
                            Some(self.current_token.unwrap().position),
                        ));
                    }
                }
            }
        }
        Ok(ListNode::new(list))
    }

    fn parse_function_parameter_names(&mut self, opener: &str) -> Result<Vec<String>, Error> {
        // empty parentheses
        match self.peek() {
            Some(token) => {
                if token.kind == TokenKind::RParen {
                    self.advance();
                    self.check_wrapper_balance(opener.to_string())?;
                    return Ok(Vec::new());
                }
            },
            None => return Err(Error::new(
                UnexpectedEOF, Some(self.previous_token.unwrap().position.one_past())
            )),
        }

        let mut params = Vec::new();
        loop {
            params.push(self.eat_token(TokenKind::Name)?.text.to_string());
            self.advance();
            if self.current_token.is_none() {
                return Err(Error::new(
                    UnexpectedEOF,
                    Some(self.previous_token.unwrap().position.one_past())
                ));
            }
            match self.current_token.unwrap().kind {
                TokenKind::Comma => continue,
                TokenKind::RParen => {
                    self.check_wrapper_balance(opener.to_string())?;
                    break;
                },
                other_token => return Err(Error::new(
                    UnexpectedToken(other_token),
                    Some(self.current_token.unwrap().position),
                )),
            }
        }
        Ok(params)
    }

    fn parse_expr(&mut self, current_operator_precedence: u32) -> Result<ExpressionNode, Error> {
        // going token by token:
        // - if the token is an LParen, dig out the RParen, putting the intermediate tokens into
        //   a secondary token stream, then call parse_expr on that to get its value. Then,
        //   move on.
        //
        // - if the token is a value (a constant, variable, function call, or list), save its value
        //   as the lhs
        //
        // - if the token is an operator, compare precedences with the currently active operator
        //   - if the precedence is higher, call parse_expr again to collect the rhs, then combine
        //     with the lhs and operator to produce the rhs of the previous operator
        //   - if the precedence is lower, consume the next value as the rhs, and then collapse the
        //     lhs, rhs, and operator into the new lhs.
        use TokenKind::*;


        let mut lhs: Option<ExpressionNode> = None;

        while let Some(token) = self.peek() {
            match token.kind {
                LParen => {
                    // bracketed expression
                    let lparen_text = token.text.to_string();
                    self.advance();

                    // dig out the RParen, then parse the enclosed tokens separately, and stick
                    // the result in lhs
                    let mut lparen_count = 1;
                    let mut sub_expression = Vec::new();
                    while lparen_count > 0 {
                        self.advance();
                        let token = match self.current_token {
                            Some(token) => token,
                            None => {
                                // ran out of tokens before closing RParen
                                return Err(Error::new(
                                    MissingToken(RParen),
                                    Some(self.previous_token.unwrap().position.one_past()),
                                ))
                            }
                        };
                        match token.kind {
                            LParen => lparen_count += 1,
                            RParen => lparen_count -= 1,
                            _ => {},
                        }
                        sub_expression.push(token);
                    }
                    self.check_wrapper_balance(lparen_text)?;

                    let mut sub_parser = Parser::new(sub_expression);
                    lhs = Some(sub_parser.parse_expr(0)?);
                },
                Plus | Minus | Mul | Div | Mod | Seq | Sne | Eq | Ne | Gt | Lt | Ge | Le => {
                    // operator

                    // check that there is a value in lhs, and extract it if there is
                    // if there isn't, there's nothing for this operator to operate upon, so
                    // this is an invalid place for the operator
                    let lhs_unwrapped = match lhs {
                        Some(value) => value,
                        None => return Err(Error::new(
                            UnexpectedToken(token.kind),
                            Some(token.position)
                        )),
                    };

                    // compare precedence with the currently active operator (or 0 if there is
                    // none)
                    let operator = Operator::from_token(token);
                    let precedence = operator.precedence();
                    if precedence > current_operator_precedence {
                        // this operator has a higher precedence, so should consume both the lhs and
                        // parse the rhs, to collapse into the rhs of the previous operator
                        self.advance();
                        let rhs = self.parse_expr(precedence)?;


                        lhs = Some(OperatorNode::new(
                            lhs_unwrapped,
                            rhs,
                            operator,
                        ).to_expression());
                        continue;
                    } else {
                        // this operator has a lower precedence, so the previous operator should
                        // be allowed to consume the lhs (which is that operator's rhs)

                        // note: this is the only path where the peeked token isn't consumed
                        // since we don't want to consume this operator yet, the function one
                        // recursion layer up will consume it instead
                        return Ok(lhs_unwrapped);
                    }
                },
                Number | BoolTrue | BoolFalse | String => {
                    // constant
                    self.advance();
                    lhs = Some(Self::parse_constant(&self.current_token.unwrap())?.to_expression());
                },
                LBrack => {
                    // list
                    self.advance();
                    lhs = Some(self.parse_list(
                        self.previous_token.unwrap().text
                    )?.to_expression());
                },
                Name => {
                    // variable or function
                    let name = token.text.to_string();
                    self.advance();
                    if let Some(token) = self.peek() {
                        if token.kind == LParen {
                            let opener = token.text;
                            self.advance();
                            return Ok(FunctionCallNode::new(
                                name,
                                self.parse_function_arguments(opener)?
                            ).to_expression());
                        }
                    }
                    lhs = Some(VariableNode::new(name).to_expression());
                },
                _other_token_type => break,
            }
        }

        match lhs {
            Some(evaluable) => Ok(evaluable),
            None => Err(Error::new(
                MissingExpression,
                Some(self.current_token.unwrap().position.one_past())
            ))
        }
    }

    fn parse_expression(&mut self) -> Result<ExpressionNode, Error> {
        self.parse_expr(0)
    }

    fn parse_expression_and_block(&mut self, current_indentation_level: usize)
        -> Result<(ExpressionNode, Block), Error>
    {
        let expression = self.parse_expression()?;
        self.eat_token(TokenKind::Newline)?;
        let block = self.parse_block(current_indentation_level + 1)?;
        Ok((expression, block))
    }

    fn parse_block(&mut self, indentation_level: usize) -> Result<Block, Error> {
        use TokenKind::*;

        let mut block = Block::new();
        let mut start_of_line = true;
        let mut previous_indentation = usize::MAX;

        while let Some(token) = self.peek() {
            if start_of_line {
                // check indentation level is what this block requires
                let indentation = token.position.start;
                if Self::calculate_indentation_level(indentation) != indentation_level {
                    break;
                }
                // check for indentation consistency
                if indentation == previous_indentation {
                    return Err(Error::new(
                        ConsistentIndentation { previous_indentation },
                        Some(Position::new(token.position.line, 0, token.position.start))
                    ));
                } else {
                    previous_indentation = indentation;
                }
                start_of_line = false;
            }
            
            match token.kind {
                Name => {
                    // function call or assignment
                    let name = token.text.to_string();
                    self.advance();
                    self.advance();
                    let current_token = match self.current_token {
                        Some(token) => token,
                        None => return Err(Error::new(
                            UnexpectedEOF,
                            Some(self.previous_token.unwrap().position.one_past())
                        )),
                    };
                    match current_token.kind {
                        LParen => {
                            // function call
                            let opener = current_token.text;
                            let function_call = FunctionCallNode::new(
                                name,
                                self.parse_function_arguments(opener)?
                            );
                            block.add_statement(function_call.to_statement());
                        },
                        Assign => {
                            // assignment
                            let expression = self.parse_expression()?;
                            block.add_statement(AssignNode::new(
                                name,
                                expression,
                            ).to_statement());
                        },
                        other_token_kind => return Err(Error::new(
                            UnexpectedToken(other_token_kind),
                            Some(current_token.position),
                        )),
                    }
                },
                If => {
                    // conditional statement
                    self.advance();
                    let (condition, block_if_condition) =
                        self.parse_expression_and_block(indentation_level)?;

                    let mut conditional_paths = vec![ConditionalPath::new(
                        condition, block_if_condition
                    )];
                    let mut else_block = None;

                    while let Some(token) = self.peek() {
                        if token.kind == Elif {
                            self.advance();
                            let (condition, block_if_condition) =
                                self.parse_expression_and_block(indentation_level)?;
                            conditional_paths.push(ConditionalPath::new(
                                condition, block_if_condition
                            ));
                        }
                        else if token.kind == Else {
                            self.advance();
                            self.eat_token(Newline)?;
                            else_block = Some(self.parse_block(indentation_level + 1)?);
                            break;
                        }
                        else {
                            break;
                        }
                    }

                    block.add_statement(ConditionalNode::new(
                        conditional_paths, else_block,
                    ).to_statement());
                },
                For => {
                    // for loop
                    self.advance();
                    let loop_variable = self.eat_token(Name)?.text.to_string();
                    self.eat_token(In)?;
                    let iterable = self.parse_expression()?;
                    self.eat_token(Newline)?;
                    let for_block = self.parse_block(indentation_level + 1)?;
                    
                    block.add_statement(ForLoopNode::new(
                        iterable, loop_variable, for_block,
                    ).to_statement());
                },
                While => {
                    // while loop
                    self.advance();
                    let condition = self.parse_expression()?;
                    self.eat_token(Newline)?;
                    let while_block = self.parse_block(indentation_level + 1)?;
                    
                    block.add_statement(WhileLoopNode::new(
                        condition, while_block
                    ).to_statement());
                },
                Break => {
                    // break
                    self.advance();
                    block.add_statement(BreakNode.to_statement());
                },
                Continue => {
                    // break
                    self.advance();
                    block.add_statement(ContinueNode.to_statement());
                },
                Return => {
                    // return
                    self.advance();
                    let return_value = self.parse_expression()?;
                    block.add_statement(ReturnNode::new(
                        return_value
                    ).to_statement());
                },
                Funcdef => {
                    // function definition
                    self.advance();
                    // parse name, parameters, and block
                    let name = self.eat_token(Name)?.text.to_string();
                    let parentheses_opener = self.eat_token(LParen)?.text.to_string();
                    let parameters = self.parse_function_parameter_names(&parentheses_opener)?;
                    self.eat_token(Newline)?;
                    let function_block = self.parse_block(indentation_level + 1)?;
                    // wrap block into FunctionDefinitionNode and add to current block
                    block.add_statement(FunctionDefinitionNode::new(
                        name, parameters, function_block
                    ).to_statement());
                },
                Newline => {
                    self.advance();
                    start_of_line = true;
                }
                other_token_kind => return Err(Error::new(
                    UnexpectedToken(other_token_kind), Some(token.position),
                )),
            }
        }

        Ok(block)
    }

    pub fn parse(&mut self) -> Result<Block, Error> {
        self.parse_block(0)
    }

    fn advance(&mut self) {
        self.previous_token = self.current_token;
        self.current_token = self.tokens.pop();
    }

    fn peek(&self) -> Option<&Token<'a>>{
        self.tokens.last()
    }

    fn eat_token(&mut self, kind: TokenKind) -> Result<Token, Error> {
        self.advance();
        let token = match self.current_token {
            Some(token) => token,
            None => return Err(Error::new(
                UnexpectedEOF,
                Some(self.previous_token.unwrap().position.one_past()),
            )),
        };
        if token.kind != kind {
            return Err(Error::new(
                UnexpectedToken(token.kind),
                Some(token.position),
            ));
        }
        Ok(token)
    }

    /// Throws an error if wrapper imbalance is invalidated, otherwise does nothing
    fn check_wrapper_balance(&mut self, opener: String) -> Result<(), Error> {
        let token = self.current_token.unwrap();
        if opener.len() == token.position.length {
            Err(Error::new(
                Balance { opener, closer: token.text.to_string() },
                Some(token.position),
            ))
        } else {
            Ok(())
        }
    }

    fn calculate_indentation_level(start: usize) -> usize {
        start / 3
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::TokenKind::*;
    use crate::runtime::Runtime;

    mod parse_constant_tests {
        use super::*;
        #[test]
        #[allow(clippy::approx_constant)]
        fn number() {
            assert_eq!(
                Ok(ConstantNode::new(Value::Number(3.14))),
                Parser::parse_constant(&Token::new(Number, "3.14", 1, 2, 4)),
            );
        }

        #[test]
        fn bool() {
            assert_eq!(
                Ok(ConstantNode::new(Value::Bool(true))),
                Parser::parse_constant(&Token::new(BoolTrue, "rtue", 1, 0, 4)),
            );
            assert_eq!(
                Ok(ConstantNode::new(Value::Bool(false))),
                Parser::parse_constant(&Token::new(BoolFalse, "flase", 1, 0, 4)),
            );
        }

        #[test]
        fn string() {
            assert_eq!(
                Ok(ConstantNode::new( Value::String("a sting".to_string()) )),
                Parser::parse_constant(&Token::new(String, "\"a sting\"\"", 1, 0, 10)),
            );
        }

        #[test]
        fn empty_string() {
            assert_eq!(
                Ok(ConstantNode::new( Value::String("".to_string()) )),
                Parser::parse_constant(&Token::new(String, "\"'", 1, 0, 2)),
            );
            assert_eq!(
                Ok(ConstantNode::new(Value::String("".to_string()))),
                Parser::parse_constant(&Token::new(String, "'\"", 1, 0, 2)),
            )
        }

        #[test]
        fn string_throw_balance_error_on_balanced_strings() {
            match Parser::parse_constant(&Token::new(String, "\"balanced\"", 1, 0, 10)) {
                Ok(_) => panic!("Expected BalanceError due to balanced double quotes. No error indicated"),
                Err(Error {
                        pos: _,
                        kind: Balance { opener, closer }
                    }
                ) => {
                    assert_eq!("\"", opener);
                    assert_eq!("\"", closer);
                },
                Err(other_error) => {
                    panic!("Expected BalanceError due to balanced double quotes. Got {:?}",
                           other_error
                    );
                }
            }
        }
    }

    mod parse_list_tests {
        use super::*;

        fn parse_list_test(expected: Vec<Value>, source: Vec<Token>) {
            let opener = source[0].text;
            let mut parser = Parser::new(source);
            // the parser must be advanced one to keep with how parse_list is called from
            // parse_expr, since this will have consumed the left bracket before calling parse_list
            parser.advance();
            assert_eq!(
                Value::List(expected),
                parser.parse_list(opener).unwrap().evaluate(&mut Runtime::new()).unwrap(),
            );
        }

        #[test]
        fn empty_list() {
            parse_list_test(
                vec![],
                vec![
                    Token::new(LBrack, "[[", 1, 0, 2),
                    Token::new(RBrack, "]", 1, 2, 1),
                ],
            );
        }

        #[test]
        fn empty_list_bracket_balance_throws_error() {
            let mut parser = Parser::new(vec![
                Token::new(LBrack, "[", 1, 0, 1),
                Token::new(RBrack, "]", 1, 2, 1),
            ]);
            parser.advance();
            match parser.parse_list("[") {
                Ok(_) => panic!("Expected Balance error, none thrown"),
                Err(Error {
                    kind: Balance { opener, closer },
                    pos: _
                }) => {
                    assert_eq!(opener, "[".to_string());
                    assert_eq!(closer, "]".to_string());
                },
                Err(other_error) => panic!("Expected Balance error, got {:?}", other_error),
            }
        }

        #[test]
        fn one_element_list() {
            parse_list_test(
                vec![Value::Number(1.0)],
                vec![
                    Token::new(LBrack, "[[", 1, 0, 2),
                    Token::new(Number, "1", 1, 2, 1),
                    Token::new(RBrack, "]", 1, 5, 1),
                ],
            );
        }

        #[test]
        fn two_element_list() {
            parse_list_test(
                vec![Value::Number(1.0), Value::Number(2.0)],
                vec![
                    Token::new(LBrack, "[[", 1, 0, 2),
                    Token::new(Number, "1", 1, 2, 1),
                    Token::new(Comma, ",", 1, 3, 1),
                    Token::new(Number, "2", 1, 4, 1),
                    Token::new(RBrack, "]", 1, 5, 1),
                ],
            );
        }

        #[test]
        fn two_element_list_bracket_balance_throws_error() {
            let mut parser = Parser::new(vec![
                Token::new(LBrack, "[", 1, 0, 1),
                Token::new(Number, "1", 1, 2, 1),
                Token::new(Comma, ",", 1, 3, 1),
                Token::new(Number, "2", 1, 4, 1),
                Token::new(RBrack, "]", 1, 5, 1),
            ]);
            parser.advance();
            match parser.parse_list("[") {
                Ok(_) => panic!("Expected Balance error, none thrown"),
                Err(Error {
                        kind: Balance { opener, closer },
                        pos: _
                    }) => {
                    assert_eq!(opener, "[".to_string());
                    assert_eq!(closer, "]".to_string());
                },
                Err(other_error) => panic!("Expected Balance error, got {:?}", other_error),
            }
        }

        #[test]
        fn one_element_expression_list() {
            parse_list_test(
                vec![Value::Number(3.0)],
                vec![
                    Token::new(LBrack, "[[", 1, 0, 2),
                    Token::new(Number, "1", 1, 2, 1),
                    Token::new(Plus, "+", 1, 3, 1),
                    Token::new(Number, "2", 1, 4, 1),
                    Token::new(RBrack, "]", 1, 5, 1),
                ],
            );
        }

        #[test]
        fn two_element_expression_list() {
            parse_list_test(
                vec![Value::Number(7.0), Value::Number(0.0)],
                vec![
                    Token::new(LBrack, "[[", 1, 0, 2),
                    Token::new(Number, "1", 1, 2, 1),
                    Token::new(Plus, "+", 1, 3, 1),
                    Token::new(Number, "2", 1, 4, 1),
                    Token::new(Mul, "*", 1, 5, 1),
                    Token::new(Number, "3", 1, 6, 1),
                    Token::new(Comma, ",", 1, 7, 1),
                    Token::new(Number, "2", 1, 8, 1),
                    Token::new(Minus, "-", 1, 9, 1),
                    Token::new(Number, "2", 1, 10, 1),
                    Token::new(RBrack, "]", 1, 11, 1),
                ],
            );
        }

        #[test]
        fn nested_list() {
            parse_list_test(
                vec![
                    Value::List(vec![
                        Value::Number(1.0),
                        Value::Number(2.0),
                    ]),
                    Value::List(vec![
                        Value::Number(3.0),
                        Value::Number(4.0),
                    ]),
                ],
                vec![
                    Token::new(LBrack, "[[", 1, 0, 2),
                    Token::new(LBrack, "[[[", 1, 3, 3),
                    Token::new(Number, "1", 1, 6, 1),
                    Token::new(Comma, ",", 1, 7, 1),
                    Token::new(Number, "2", 1, 8, 1),
                    Token::new(RBrack, "]", 1, 9, 1),
                    Token::new(Comma, ",", 1, 10, 1),
                    Token::new(LBrack, "[", 1, 11, 1),
                    Token::new(Number, "3", 1, 12, 1),
                    Token::new(Comma, ",", 1, 13, 1),
                    Token::new(Number, "4", 1, 14, 1),
                    Token::new(RBrack, "]]", 1, 15, 2),
                    Token::new(RBrack, "]", 1, 18, 1),
                ],
            );
        }
    }

    mod parse_function_parameter_names_tests {
        use super::*;

        #[test]
        fn empty_parentheses_not_balanced() {
            let tokens = vec![
                Token::new(RParen, "))", 1, 0, 2),
            ];
            assert_eq!(
                Parser::new(tokens).parse_function_parameter_names("(").unwrap(),
                Vec::<std::string::String>::new(),
            )
        }

        #[test]
        fn empty_parentheses_balanced_throws_balance_error() {
            let tokens = vec![
                Token::new(RParen, ")", 1, 0, 1),
            ];
            let error = Parser::new(tokens).parse_function_parameter_names("(").unwrap_err();
            if let Balance {..} = error.kind
            {} else {
                panic!("Expected Balance error (got: {:?})", error.kind);
            }
        }

        #[test]
        fn only_open_parentheses_but_not_eof_throws_unexpected_token_error() {
            let tokens = vec![
                Token::new(Seq, "===", 1, 0, 3),
            ];
            let error = Parser::new(tokens).parse_function_parameter_names("(").unwrap_err();
            if error.kind == UnexpectedToken(Seq) {
            } else {
                panic!("Expected UnexpectedToken error (got: {:?})", error.kind);
            }
        }

        #[test]
        fn single_parameter_parentheses_not_balanced() {
            let tokens = vec![
                Token::new(TokenKind::Name, "param1", 1, 0, 6),
                Token::new(TokenKind::RParen, "))", 1, 7, 2),
            ];
            assert_eq!(
                Parser::new(tokens).parse_function_parameter_names("(").unwrap(),
                vec!["param1".to_string()],
            )
        }

        #[test]
        fn single_parameter_parentheses_balanced_throws_balance_error() {
            let tokens = vec![
                Token::new(TokenKind::Name, "param1", 1, 0, 6),
                Token::new(TokenKind::RParen, ")", 1, 7, 1),
            ];
            let error = Parser::new(tokens).parse_function_parameter_names("(").unwrap_err();
            if let Balance {..} = error.kind
            {} else {
                panic!("Expected Balance error (got: {:?})", error.kind);
            }
        }

        #[test]
        fn single_parameter_not_closed_throws_unexpected_eof_error() {
            let tokens = vec![
                Token::new(TokenKind::Name, "param1", 1, 0, 6),
            ];
            let error = Parser::new(tokens).parse_function_parameter_names("(").unwrap_err();
            if error.kind == UnexpectedEOF {
            } else {
                panic!("Expected UnexpectedEOF error (got: {:?})", error.kind);
            }
        }

        #[test]
        fn single_parameter_not_closed_and_not_eof_throws_unexpected_token_error() {
            let tokens = vec![
                Token::new(TokenKind::Name, "param1", 1, 0, 6),
                Token::new(Seq, "===", 1, 7, 3),
            ];
            let error = Parser::new(tokens).parse_function_parameter_names("(").unwrap_err();
            if error.kind == UnexpectedToken(Seq) {
            } else {
                panic!("Expected UnexpectedToken error (got: {:?})", error.kind);
            }
        }

        #[test]
        fn multi_parameter_parentheses_not_balanced() {
            let tokens = vec![
                Token::new(TokenKind::Name, "param1", 1, 0, 6),
                Token::new(TokenKind::Comma, ",", 1, 6, 1),
                Token::new(TokenKind::Name, "param2", 1, 8, 6),
                Token::new(TokenKind::Comma, ",", 1, 14, 1),
                Token::new(TokenKind::Name, "param3", 1, 16, 6),
                Token::new(TokenKind::RParen, "))", 1, 22, 2),
            ];
            assert_eq!(
                Parser::new(tokens).parse_function_parameter_names("(").unwrap(),
                vec!["param1".to_string(), "param2".to_string(), "param3".to_string()],
            )
        }

        #[test]
        fn multi_parameter_parentheses_balanced_throws_balance_error() {
            let tokens = vec![
                Token::new(TokenKind::Name, "param1", 1, 0, 6),
                Token::new(TokenKind::Comma, ",", 1, 6, 1),
                Token::new(TokenKind::Name, "param2", 1, 8, 6),
                Token::new(TokenKind::Comma, ",", 1, 14, 1),
                Token::new(TokenKind::Name, "param3", 1, 16, 6),
                Token::new(TokenKind::RParen, ")", 1, 22, 1),
            ];
            let error = Parser::new(tokens).parse_function_parameter_names("(").unwrap_err();
            if let Balance {..} = error.kind
            {} else {
                panic!("Expected Balance error (got: {:?})", error.kind);
            }
        }

        #[test]
        fn multi_parameter_not_closed_on_name_throws_unexpected_eof_error() {
            let tokens = vec![
                Token::new(TokenKind::Name, "param1", 1, 0, 6),
                Token::new(TokenKind::Comma, ",", 1, 6, 1),
                Token::new(TokenKind::Name, "param2", 1, 8, 6),
                Token::new(TokenKind::Comma, ",", 1, 14, 1),
                Token::new(TokenKind::Name, "param3", 1, 16, 6),
            ];
            let error = Parser::new(tokens).parse_function_parameter_names("(").unwrap_err();
            if error.kind == UnexpectedEOF {
            } else {
                panic!("Expected UnexpectedEOF error (got: {:?})", error.kind);
            }
        }

        #[test]
        fn multi_parameter_not_closed_on_name_and_not_eof_throws_unexpected_token_error() {
            let tokens = vec![
                Token::new(TokenKind::Name, "param1", 1, 0, 6),
                Token::new(TokenKind::Comma, ",", 1, 6, 1),
                Token::new(TokenKind::Name, "param2", 1, 8, 6),
                Token::new(TokenKind::Comma, ",", 1, 14, 1),
                Token::new(TokenKind::Name, "param3", 1, 16, 6),
                Token::new(Seq, "===", 1, 22, 3),
            ];
            let error = Parser::new(tokens).parse_function_parameter_names("(").unwrap_err();
            if error.kind == UnexpectedToken(Seq) {
            } else {
                panic!("Expected UnexpectedToken error (got: {:?})", error.kind);
            }
        }

        #[test]
        fn multi_parameter_not_closed_on_comma_throws_unexpected_eof_error() {
            let tokens = vec![
                Token::new(TokenKind::Name, "param1", 1, 0, 6),
                Token::new(TokenKind::Comma, ",", 1, 6, 1),
                Token::new(TokenKind::Name, "param2", 1, 8, 6),
                Token::new(TokenKind::Comma, ",", 1, 14, 1),
            ];
            let error = Parser::new(tokens).parse_function_parameter_names("(").unwrap_err();
            if error.kind == UnexpectedEOF {
            } else {
                panic!("Expected UnexpectedEOF error (got: {:?})", error.kind);
            }
        }

        #[test]
        fn multi_parameter_not_closed_on_comma_and_not_eof_throws_unexpected_token_error() {
            let tokens = vec![
                Token::new(TokenKind::Name, "param1", 1, 0, 6),
                Token::new(TokenKind::Comma, ",", 1, 6, 1),
                Token::new(TokenKind::Name, "param2", 1, 8, 6),
                Token::new(TokenKind::Comma, ",", 1, 14, 1),
                Token::new(Seq, "===", 1, 15, 3),
            ];
            let error = Parser::new(tokens).parse_function_parameter_names("(").unwrap_err();
            if error.kind == UnexpectedToken(Seq) {
            } else {
                panic!("Expected UnexpectedToken error (got: {:?})", error.kind);
            }
        }
    }

    mod parse_expr_tests {
        use super::*;

        #[test]
        fn brackets_take_priority() {
            // 3 * (2 + 4))
            let tokens = vec![
                Token::new(Number, "3", 1, 0, 1),
                Token::new(Mul, "*", 1, 2, 1),
                Token::new(LParen, "(", 1, 4, 1),
                Token::new(Number, "2", 1, 5, 1),
                Token::new(Plus, "+", 1, 7, 1),
                Token::new(Number, "4", 1, 9, 1),
                Token::new(RParen, "))", 1, 10, 2),
            ];
            let expected_ast = OperatorNode::new(
                ConstantNode::new(Value::Number(3.0)).to_expression(),
                OperatorNode::new(
                    ConstantNode::new(Value::Number(2.0)).to_expression(),
                    ConstantNode::new(Value::Number(4.0)).to_expression(),
                    Operator::Add,
                ).to_expression(),
                Operator::Mul,
            ).to_expression();
            assert_eq!(
                expected_ast,
                Parser::new(tokens).parse_expression().unwrap()
            );
        }

        #[test]
        fn mul_takes_priority_over_plus() {
            // 3 + 4 * 5
            let tokens = vec![
                Token::new(Number, "3", 1, 0, 1),
                Token::new(Plus, "+", 1, 2, 1),
                Token::new(Number, "4", 1, 4, 1),
                Token::new(Mul, "*", 1, 6, 1),
                Token::new(Number, "5", 1, 8, 1),
            ];
            let expected_ast = OperatorNode::new(
                ConstantNode::new(Value::Number(3.0)).to_expression(),
                OperatorNode::new(
                    ConstantNode::new(Value::Number(4.0)).to_expression(),
                    ConstantNode::new(Value::Number(5.0)).to_expression(),
                    Operator::Mul,
                ).to_expression(),
                Operator::Add,
            ).to_expression();
            assert_eq!(
                expected_ast,
                Parser::new(tokens).parse_expression().unwrap(),
            );
        }

        #[test]
        fn mul_takes_priority_over_minus() {
            // 3 - 4 * 5
            let tokens = vec![
                Token::new(Number, "3", 1, 0, 1),
                Token::new(Minus, "-", 1, 2, 1),
                Token::new(Number, "4", 1, 4, 1),
                Token::new(Mul, "*", 1, 6, 1),
                Token::new(Number, "5", 1, 8, 1),
            ];
            let expected_ast = OperatorNode::new(
                ConstantNode::new(Value::Number(3.0)).to_expression(),
                OperatorNode::new(
                    ConstantNode::new(Value::Number(4.0)).to_expression(),
                    ConstantNode::new(Value::Number(5.0)).to_expression(),
                    Operator::Mul,
                ).to_expression(),
                Operator::Sub,
            ).to_expression();
            assert_eq!(
                expected_ast,
                Parser::new(tokens).parse_expression().unwrap(),
            );
        }

        #[test]
        fn div_takes_priority_over_plus() {
            // 3 + 12 / 4
            let tokens = vec![
                Token::new(Number, "3", 1, 0, 1),
                Token::new(Plus, "+", 1, 2, 1),
                Token::new(Number, "12", 1, 4, 2),
                Token::new(Div, "/", 1, 7, 1),
                Token::new(Number, "4", 1, 9, 1),
            ];
            let expected_ast = OperatorNode::new(
                ConstantNode::new(Value::Number(3.0)).to_expression(),
                OperatorNode::new(
                    ConstantNode::new(Value::Number(12.0)).to_expression(),
                    ConstantNode::new(Value::Number(4.0)).to_expression(),
                    Operator::Div,
                ).to_expression(),
                Operator::Add,
            ).to_expression();
            assert_eq!(
                expected_ast,
                Parser::new(tokens).parse_expression().unwrap(),
            );
        }

        #[test]
        fn div_takes_priority_over_minus() {
            // 3 - 12 / 4
            let tokens = vec![
                Token::new(Number, "3", 1, 0, 1),
                Token::new(Minus, "-", 1, 2, 1),
                Token::new(Number, "12", 1, 4, 2),
                Token::new(Div, "/", 1, 7, 1),
                Token::new(Number, "4", 1, 9, 1),
            ];
            let expected_ast = OperatorNode::new(
                ConstantNode::new(Value::Number(3.0)).to_expression(),
                OperatorNode::new(
                    ConstantNode::new(Value::Number(12.0)).to_expression(),
                    ConstantNode::new(Value::Number(4.0)).to_expression(),
                    Operator::Div,
                ).to_expression(),
                Operator::Sub,
            ).to_expression();
            assert_eq!(
                expected_ast,
                Parser::new(tokens).parse_expression().unwrap(),
            );
        }

        #[test]
        fn mod_takes_priority_over_plus() {
            // 3 + 12 % 5
            let tokens = vec![
                Token::new(Number, "3", 1, 0, 1),
                Token::new(Plus, "+", 1, 2, 1),
                Token::new(Number, "12", 1, 4, 2),
                Token::new(Mod, "%", 1, 7, 1),
                Token::new(Number, "5", 1, 9, 1),
            ];
            let expected_ast = OperatorNode::new(
                ConstantNode::new(Value::Number(3.0)).to_expression(),
                OperatorNode::new(
                    ConstantNode::new(Value::Number(12.0)).to_expression(),
                    ConstantNode::new(Value::Number(5.0)).to_expression(),
                    Operator::Mod,
                ).to_expression(),
                Operator::Add,
            ).to_expression();
            assert_eq!(
                expected_ast,
                Parser::new(tokens).parse_expression().unwrap(),
            );
        }

        #[test]
        fn mod_takes_priority_over_minus() {
            // 3 - 12 % 5
            let tokens = vec![
                Token::new(Number, "3", 1, 0, 1),
                Token::new(Minus, "-", 1, 2, 1),
                Token::new(Number, "12", 1, 4, 2),
                Token::new(Mod, "%", 1, 7, 1),
                Token::new(Number, "5", 1, 9, 1),
            ];
            let expected_ast = OperatorNode::new(
                ConstantNode::new(Value::Number(3.0)).to_expression(),
                OperatorNode::new(
                    ConstantNode::new(Value::Number(12.0)).to_expression(),
                    ConstantNode::new(Value::Number(5.0)).to_expression(),
                    Operator::Mod,
                ).to_expression(),
                Operator::Sub,
            ).to_expression();
            assert_eq!(
                expected_ast,
                Parser::new(tokens).parse_expression().unwrap(),
            );
        }

        #[test]
        fn bidmas_complete() {
            // 7 - 5 % 2 + 3 * 4 / (2 + 4))
            let tokens = vec![
                Token::new(Number, "7", 1, 0, 1),
                Token::new(Minus, "-", 1, 2, 1),
                Token::new(Number, "5", 1, 4, 1),
                Token::new(Mod, "%", 1, 6, 1),
                Token::new(Number, "2", 1, 8, 1),
                Token::new(Plus, "+", 1, 10, 1),
                Token::new(Number, "3", 1, 12, 1),
                Token::new(Mul, "*", 1, 14, 1),
                Token::new(Number, "4", 1, 16, 1),
                Token::new(Div, "/", 1, 18, 1),
                Token::new(LParen, "(", 1, 20, 1),
                Token::new(Number, "2", 1, 21, 1),
                Token::new(Plus, "+", 1, 23, 1),
                Token::new(Number, "4", 1, 25, 1),
                Token::new(RParen, "))", 1, 26, 2),
            ];
            // expected tree:
            //           +
            //   ┌───────┴───────┐
            // ┌─-───┐       ┌───/────┐
            // 7   ┌─%─┐   ┌─*─┐    ┌─+─┐
            //     5   2   3   4    2   4
            let expected_ast = OperatorNode::new(
                OperatorNode::new(
                    ConstantNode::new(Value::Number(7.0)).to_expression(),
                    OperatorNode::new(
                        ConstantNode::new(Value::Number(5.0)).to_expression(),
                        ConstantNode::new(Value::Number(2.0)).to_expression(),
                        Operator::Mod,
                    ).to_expression(),
                    Operator::Sub,
                ).to_expression(),
                OperatorNode::new(
                    OperatorNode::new(
                        ConstantNode::new(Value::Number(3.0)).to_expression(),
                        ConstantNode::new(Value::Number(4.0)).to_expression(),
                        Operator::Mul,
                    ).to_expression(),
                    OperatorNode::new(
                        ConstantNode::new(Value::Number(2.0)).to_expression(),
                        ConstantNode::new(Value::Number(4.0)).to_expression(),
                        Operator::Add,
                    ).to_expression(),
                    Operator::Div,
                ).to_expression(),
                Operator::Add,
            ).to_expression();
            assert_eq!(
                expected_ast,
                Parser::new(tokens).parse_expression().unwrap(),
            )
        }

        #[test]
        fn balanced_parentheses_throw_error() {
            let tokens = vec![
                Token::new(LParen, "(", 1, 0, 1),
                Token::new(Number, "1", 1, 1, 1),
                Token::new(RParen, ")", 1, 2, 1),
            ];
            let result = Parser::new(tokens).parse_expression();
            match result {
                Ok(_) => panic!("Expected Balance error, got Ok()"),
                Err(Error { kind: Balance { opener, closer }, ..}) => {
                    assert_eq!("(", opener);
                    assert_eq!(")", closer);
                },
                Err(other_error) => panic!("Expected Balance error, got {:?}", other_error),
            }
        }

        #[test]
        fn single_variable_parsing() {
            let tokens = vec![
                Token::new(TokenKind::Name, "a", 1, 0, 1),
            ];
            let expected_ast = VariableNode::new("a".to_string()).to_expression();
            assert_eq!(
                expected_ast,
                Parser::new(tokens).parse_expression().unwrap(),
            );
        }

        #[test]
        fn bidmas_complete_with_variables() {
            // seven - five % 2 + three * four / (2 + four))
            let tokens = vec![
                Token::new(TokenKind::Name, "seven", 1, 0, 5),
                Token::new(Minus, "-", 1, 6, 1),
                Token::new(TokenKind::Name, "five", 1, 8, 4),
                Token::new(Mod, "%", 1, 13, 1),
                Token::new(Number, "2", 1, 15, 1),
                Token::new(Plus, "+", 1, 17, 1),
                Token::new(TokenKind::Name, "three", 1, 19, 5),
                Token::new(Mul, "*", 1, 25, 1),
                Token::new(TokenKind::Name, "four", 1, 27, 4),
                Token::new(Div, "/", 1, 32, 1),
                Token::new(LParen, "(", 1, 34, 1),
                Token::new(Number, "2", 1, 35, 1),
                Token::new(Plus, "+", 1, 37, 1),
                Token::new(TokenKind::Name, "four", 39, 25, 4),
                Token::new(RParen, "))", 1, 43, 2),
            ];
            // expected tree:
            //               +
            //       ┌───────┴──────────┐
            //   ┌───-───┐          ┌───/─────┐
            // seven   ┌─%─┐     ┌──*──┐    ┌─+─┐
            //       five  2   three  four  2  four
            let expected_ast = OperatorNode::new(
                OperatorNode::new(
                    VariableNode::new("seven".to_string()).to_expression(),
                    OperatorNode::new(
                        VariableNode::new("five".to_string()).to_expression(),
                        ConstantNode::new(Value::Number(2.0)).to_expression(),
                        Operator::Mod,
                    ).to_expression(),
                    Operator::Sub,
                ).to_expression(),
                OperatorNode::new(
                    OperatorNode::new(
                        VariableNode::new("three".to_string()).to_expression(),
                        VariableNode::new("four".to_string()).to_expression(),
                        Operator::Mul,
                    ).to_expression(),
                    OperatorNode::new(
                        ConstantNode::new(Value::Number(2.0)).to_expression(),
                        VariableNode::new("four".to_string()).to_expression(),
                        Operator::Add,
                    ).to_expression(),
                    Operator::Div,
                ).to_expression(),
                Operator::Add,
            ).to_expression();
            assert_eq!(
                expected_ast,
                Parser::new(tokens).parse_expression().unwrap(),
            )
        }
    }
}
