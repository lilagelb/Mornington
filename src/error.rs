use crate::lexer::{TokenKind};
use crate::lexer::Position;
use crate::value::Value;

#[derive(Clone, Debug, PartialEq)]
pub struct Error {
    pub kind: ErrorKind,
    // TODO: replace this with Option<Position>
    pub pos: Position,
}
impl Error {
    pub fn new(kind: ErrorKind, position: Position) -> Error {
        Error {
            pos: position,
            kind,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ErrorKind {
    Balance { opener: String, closer: String },
    UnexpectedToken(TokenKind),
    UnexpectedEOF,
    MissingToken(TokenKind),
    MissingExpression,
    InvalidFormatFlag { flag: String, specifier_num: usize },
    IncorrectNumberOfFormatStringArguments { expected: usize, received: usize },
    Name(String),
    ConsistentIndentation { previous_indentation: usize },
    Signature { function_name: String, expected_args: usize, passed_args: usize },
    Break,
    Continue,
    Return(Value),
    Input,
}