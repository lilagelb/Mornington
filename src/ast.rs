use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use crate::error::Error;
use crate::error::ErrorKind::{Break, Continue, Return};
use crate::lexer::{Position, Token, TokenKind};
use crate::runtime::Runtime;
use crate::value::Value;

// TODO: implement PartialEq for all node types to allow full test coverage


pub trait Evaluable: Debug {
    fn evaluate(&self, runtime: &Runtime) -> Result<Value, Error>;
}

pub type Expression = Box<dyn Evaluable>;

pub trait Executable: Debug {
    fn execute(&self, runtime: &mut Runtime) -> Result<(), Error>;
}
type Statement = Box<dyn Executable>;


#[derive(Debug, PartialEq)]
pub struct ConstantNode {
    value: Value,
}
impl ConstantNode {
    pub fn new(value: Value) -> ConstantNode {
        ConstantNode { value }
    }
}
impl Evaluable for ConstantNode {
    fn evaluate(&self, _: &Runtime) -> Result<Value, Error> {
        Ok(self.value.clone())
    }
}


#[derive(Debug)]
pub struct ListNode {
    list: Vec<Expression>,
}
impl ListNode {
    pub fn new(list: Vec<Expression>) -> ListNode {
        ListNode { list }
    }
}
impl Evaluable for ListNode {
    fn evaluate(&self, runtime: &Runtime) -> Result<Value, Error> {
        let mut evaluated_list = Vec::new();
        for element in &self.list {
            evaluated_list.push(element.evaluate(runtime)?);
        }
        Ok(Value::List(evaluated_list))
    }
}


#[derive(Debug)]
pub struct OperatorNode {
    lhs: Expression,
    rhs: Expression,
    operator: Operator,
}
impl OperatorNode {
    pub fn new(lhs: Expression, rhs: Expression, operator: Operator) -> OperatorNode {
        OperatorNode { lhs, rhs, operator }
    }
}

impl Evaluable for OperatorNode {
    fn evaluate(&self, runtime: &Runtime) -> Result<Value, Error> {
        let lhs = self.lhs.evaluate(runtime)?;
        let rhs = self.rhs.evaluate(runtime)?;

        use Operator::*;
        Ok(match &self.operator {
            Add => lhs.add(&rhs),
            Sub => lhs.sub(&rhs),
            Mul => lhs.mul(&rhs),
            Div => lhs.div(&rhs),
            Mod => lhs.modulus(&rhs)?,
            Seq => lhs.seq(&rhs),
            Sne => lhs.sne(&rhs),
            Eq => lhs.eq(&rhs),
            Ne => lhs.ne(&rhs),
            Gt => lhs.gt(&rhs),
            Lt => lhs.lt(&rhs),
            Ge => lhs.ge(&rhs),
            Le => lhs.le(&rhs),
        })
    }
}

#[derive(Debug)]
pub enum Operator {
    Add, Sub, Mul, Div, Mod,
    Seq, Sne, Eq, Ne, Gt, Lt, Ge, Le,
}
impl Operator {
    pub fn from_token(token: &Token) -> Operator {
        match token.kind {
            TokenKind::Plus => Operator::Add,
            TokenKind::Minus => Operator::Sub,
            TokenKind::Mul => Operator::Mul,
            TokenKind::Div => Operator::Div,
            TokenKind::Mod => Operator::Mod,
            TokenKind::Seq => Operator::Seq,
            TokenKind::Sne => Operator::Sne,
            TokenKind::Eq => Operator::Eq,
            TokenKind::Ne => Operator::Ne,
            TokenKind::Gt => Operator::Gt,
            TokenKind::Lt => Operator::Lt,
            TokenKind::Ge => Operator::Ge,
            TokenKind::Le => Operator::Le,
            _ => panic!()
        }
    }

    pub fn precedence(&self) -> u32 {
        use Operator::*;
        match self {
            Seq | Sne | Eq | Ne | Gt | Lt | Ge | Le => 10,
            Add | Sub => 20,
            Mul | Div | Mod => 30,
        }
    }
}


#[derive(Debug)]
pub struct VariableNode {
    name: String,
}
impl VariableNode {
    pub fn new(name: String) -> VariableNode {
        VariableNode { name }
    }
}

impl Evaluable for VariableNode {
    fn evaluate(&self, runtime: &Runtime) -> Result<Value, Error> {
        match runtime.get_variable(&self.name) {
            Ok(value_ref) => Ok(value_ref.clone()),
            Err(error) => Err(error),
        }
    }
}


#[derive(Debug)]
pub struct FunctionCallNode {
    name: String,
    args: ListNode,
}
impl FunctionCallNode {
    pub fn new(name: String, args: ListNode) -> FunctionCallNode {
        FunctionCallNode { name, args }
    }
}

impl Evaluable for FunctionCallNode {
    fn evaluate(&self, runtime: &Runtime) -> Result<Value, Error> {
        // TODO: this is a TEMPORARY HACK to get some output working for some basic testing
        if self.name == "prointl" {
            println!("{}", self.args.list[0].evaluate(runtime)?.coerce_to_string());
            Ok(Value::Number(0.0))
        } else {
            todo!("function call node")
        }
    }
}
impl Executable for FunctionCallNode {
    fn execute(&self, runtime: &mut Runtime) -> Result<(), Error> {
        self.evaluate(runtime)?;
        Ok(())
    }
}


#[derive(Debug)]
pub struct Block {
    statements: Vec<Statement>,
}
impl Block {
    pub fn new() -> Block {
        Block { statements: Vec::new() }
    }

    pub fn add_statement(&mut self, statement: Statement) {
        self.statements.push(statement);
    }

    fn execute_in_new_scope(&self, runtime: &mut Runtime) -> Result<(), Error> {
        runtime.begin_scope();
        self.execute(runtime)?;
        runtime.end_scope();
        Ok(())
    }
}

impl Executable for Block {
    fn execute(&self, runtime: &mut Runtime) -> Result<(), Error> {
        for statement in &self.statements {
            statement.execute(runtime)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct AssignNode {
    target: String,
    expression: Expression,
}
impl AssignNode {
    pub fn new(target: String, expression: Expression) -> AssignNode {
        AssignNode { target, expression }
    }
}

impl Executable for AssignNode {
    fn execute(&self, runtime: &mut Runtime) -> Result<(), Error> {
        runtime.set_variable(&self.target, self.expression.evaluate(runtime)?);
        Ok(())
    }
}


#[derive(Debug)]
pub struct ConditionalNode {
    conditional_paths: Vec<ConditionalPath>,
    else_block: Option<Block>,
}
impl ConditionalNode {
    pub fn new(conditional_paths: Vec<ConditionalPath>, else_block: Option<Block>) -> ConditionalNode {
        ConditionalNode { conditional_paths, else_block }
    }
}

impl Executable for ConditionalNode {
    fn execute(&self, runtime: &mut Runtime) -> Result<(), Error> {
        for ConditionalPath { condition, block: path } in &self.conditional_paths {
            if condition.evaluate(runtime)?.coerce_to_bool() {
                path.execute_in_new_scope(runtime)?;
                return Ok(());
            }
        }
        if let Some(block) = &self.else_block {
            block.execute_in_new_scope(runtime)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct ConditionalPath {
    condition: Expression,
    block: Block,
}
impl ConditionalPath {
    pub fn new(condition: Expression, block: Block) -> ConditionalPath {
        ConditionalPath { condition, block }
    }
}


#[derive(Debug)]
pub struct WhileLoopNode {
    condition: Expression,
    block: Block,
}
impl WhileLoopNode {
    pub fn new(condition: Expression, block: Block) -> WhileLoopNode {
        WhileLoopNode { condition, block }
    }
}
impl Executable for WhileLoopNode {
    fn execute(&self, runtime: &mut Runtime) -> Result<(), Error> {
        runtime.begin_scope();
        while self.condition.evaluate(runtime)?.coerce_to_bool() {
            // execute the loop block, catching any propagated breaks or continues
            match self.block.execute(runtime) {
                Ok(_) => {},
                Err(Error { kind: Continue, ..}) => continue,
                Err(Error { kind: Break, ..}) => break,
                Err(other_error) => return Err(other_error),
            }
        }
        runtime.end_scope();
        Ok(())
    }
}

#[derive(Debug)]
pub struct ForLoopNode {
    iterable: Expression,
    loop_variable: String,
    block: Block,
}
impl ForLoopNode {
    pub fn new(iterable: Expression, loop_variable: String, block: Block) -> ForLoopNode {
        ForLoopNode { iterable, loop_variable, block }
    }
}
impl Executable for ForLoopNode {
    fn execute(&self, runtime: &mut Runtime) -> Result<(), Error> {
        let iterable = self.iterable.evaluate(runtime)?.coerce_to_list();
        if iterable.is_empty() {
            return Ok(());
        }
        runtime.begin_scope();
        for element in &iterable {
            runtime.set_variable(&self.loop_variable, element.clone());
            // execute the loop block, catching any propagated breaks or continues
            match self.block.execute(runtime) {
                Ok(_) => {},
                Err(Error { kind: Continue, ..}) => continue,
                Err(Error { kind: Break, ..}) => break,
                Err(other_error) => return Err(other_error),
            }
        }
        runtime.end_scope();
        Ok(())
    }
}


#[derive(Debug)]
pub struct BreakNode;
impl Executable for BreakNode {
    fn execute(&self, _runtime: &mut Runtime) -> Result<(), Error> {
        Err(Error::new(Break, Position::new(0, 0, 0)))
    }
}


#[derive(Debug)]
pub struct ContinueNode;
impl Executable for ContinueNode {
    fn execute(&self, _runtime: &mut Runtime) -> Result<(), Error> {
        Err(Error::new(Continue, Position::new(0, 0, 0)))
    }
}


#[derive(Debug)]
pub struct ReturnNode {
    return_value: Expression,
}
impl ReturnNode {
    pub fn new(return_value: Expression) -> ReturnNode {
        ReturnNode { return_value }
    }
}
impl Executable for ReturnNode {
    fn execute(&self, runtime: &mut Runtime) -> Result<(), Error> {
        Err(Error::new(Return(self.return_value.evaluate(runtime)?), Position::new(0, 0, 0)))
    }
}


// TODO: function definition node
#[derive(Debug)]
pub struct FunctionDefinitionNode {
    name: String,
    parameters: Vec<String>,
    block: Block,
}
impl FunctionDefinitionNode {
    pub fn new(name: String, parameters: Vec<String>, block: Block) -> FunctionDefinitionNode {
        FunctionDefinitionNode {
            name, parameters, block,
        }
    }
}
impl Executable for FunctionDefinitionNode {
    fn execute(&self, runtime: &mut Runtime) -> Result<(), Error> {
        todo!()
    }
}