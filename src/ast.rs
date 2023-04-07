use std::any::Any;
use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use crate::error::Error;
use crate::error::ErrorKind::{Break, Continue, Return};
use crate::lexer::{Position, Token, TokenKind};
use crate::runtime::Runtime;
use crate::value::Value;


pub trait Evaluable: Debug {
    fn evaluate(&self, runtime: &Runtime) -> Result<Value, Error>;

    fn to_expression(self) -> ExpressionNode;
}

#[derive(Debug, PartialEq)]
pub enum ExpressionNode {
    Constant(ConstantNode),
    List(ListNode),
    Operator(Box<OperatorNode>),
    Variable(VariableNode),
    FunctionCall(FunctionCallNode),
}
impl Evaluable for ExpressionNode {
    fn evaluate(&self, runtime: &Runtime) -> Result<Value, Error> {
        match self {
            ExpressionNode::Constant(node) => node.evaluate(runtime),
            ExpressionNode::List(node) => node.evaluate(runtime),
            ExpressionNode::Operator(node) => node.evaluate(runtime),
            ExpressionNode::Variable(node) => node.evaluate(runtime),
            ExpressionNode::FunctionCall(node) => node.evaluate(runtime),
        }
    }

    fn to_expression(self) -> ExpressionNode {
        self
    }
}

pub trait Executable: Debug {
    fn execute(&self, runtime: &mut Runtime) -> Result<(), Error>;

    fn to_statement(self) -> StatementNode;
}

#[derive(Debug, PartialEq)]
pub enum StatementNode {
    Block(Block),
    Assign(AssignNode),
    FunctionCall(FunctionCallNode),
    Conditional(ConditionalNode),
    ForLoop(ForLoopNode),
    WhileLoop(WhileLoopNode),
    Break(BreakNode),
    Continue(ContinueNode),
    Return(ReturnNode),
    FunctionDefinition(FunctionDefinitionNode),
}
impl Executable for StatementNode {
    fn execute(&self, runtime: &mut Runtime) -> Result<(), Error> {
        match self {
            StatementNode::Block(node) => node.execute(runtime),
            StatementNode::Assign(node) => node.execute(runtime),
            StatementNode::FunctionCall(node) => node.execute(runtime),
            StatementNode::Conditional(node) => node.execute(runtime),
            StatementNode::ForLoop(node) => node.execute(runtime),
            StatementNode::WhileLoop(node) => node.execute(runtime),
            StatementNode::Break(node) => node.execute(runtime),
            StatementNode::Continue(node) => node.execute(runtime),
            StatementNode::Return(node) => node.execute(runtime),
            StatementNode::FunctionDefinition(node) => node.execute(runtime),
        }
    }

    fn to_statement(self) -> StatementNode {
        self
    }
}


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

    fn to_expression(self) -> ExpressionNode {
        ExpressionNode::Constant(self)
    }
}


#[derive(Debug, PartialEq)]
pub struct ListNode {
    list: Vec<ExpressionNode>,
}
impl ListNode {
    pub fn new(list: Vec<ExpressionNode>) -> ListNode {
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

    fn to_expression(self) -> ExpressionNode {
        ExpressionNode::List(self)
    }
}


#[derive(Debug, PartialEq)]
pub struct OperatorNode {
    lhs: ExpressionNode,
    rhs: ExpressionNode,
    operator: Operator,
}
impl OperatorNode {
    pub fn new(lhs: ExpressionNode, rhs: ExpressionNode, operator: Operator) -> OperatorNode {
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

    fn to_expression(self) -> ExpressionNode {
        ExpressionNode::Operator(Box::new(self))
    }
}

#[derive(Debug, PartialEq)]
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


#[derive(Debug, PartialEq)]
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

    fn to_expression(self) -> ExpressionNode {
        ExpressionNode::Variable(self)
    }
}


#[derive(Debug, PartialEq)]
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

    fn to_expression(self) -> ExpressionNode {
        ExpressionNode::FunctionCall(self)
    }
}
impl Executable for FunctionCallNode {
    fn execute(&self, runtime: &mut Runtime) -> Result<(), Error> {
        self.evaluate(runtime)?;
        Ok(())
    }

    fn to_statement(self) -> StatementNode {
        StatementNode::FunctionCall(self)
    }
}


#[derive(Debug, PartialEq)]
pub struct Block {
    statements: Vec<StatementNode>,
}
impl Block {
    pub fn new() -> Block {
        Block { statements: Vec::new() }
    }

    pub fn add_statement(&mut self, statement: StatementNode) {
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

    fn to_statement(self) -> StatementNode {
        StatementNode::Block(self)
    }
}

#[derive(Debug, PartialEq)]
pub struct AssignNode {
    target: String,
    expression: ExpressionNode,
}
impl AssignNode {
    pub fn new(target: String, expression: ExpressionNode) -> AssignNode {
        AssignNode { target, expression }
    }
}

impl Executable for AssignNode {
    fn execute(&self, runtime: &mut Runtime) -> Result<(), Error> {
        runtime.set_variable(&self.target, self.expression.evaluate(runtime)?);
        Ok(())
    }

    fn to_statement(self) -> StatementNode {
        StatementNode::Assign(self)
    }
}


#[derive(Debug, PartialEq)]
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

    fn to_statement(self) -> StatementNode {
        StatementNode::Conditional(self)
    }
}

#[derive(Debug, PartialEq)]
pub struct ConditionalPath {
    condition: ExpressionNode,
    block: Block,
}
impl ConditionalPath {
    pub fn new(condition: ExpressionNode, block: Block) -> ConditionalPath {
        ConditionalPath { condition, block }
    }
}


#[derive(Debug, PartialEq)]
pub struct WhileLoopNode {
    condition: ExpressionNode,
    block: Block,
}
impl WhileLoopNode {
    pub fn new(condition: ExpressionNode, block: Block) -> WhileLoopNode {
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

    fn to_statement(self) -> StatementNode {
        StatementNode::WhileLoop(self)
    }
}

#[derive(Debug, PartialEq)]
pub struct ForLoopNode {
    iterable: ExpressionNode,
    loop_variable: String,
    block: Block,
}
impl ForLoopNode {
    pub fn new(iterable: ExpressionNode, loop_variable: String, block: Block) -> ForLoopNode {
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

    fn to_statement(self) -> StatementNode {
        StatementNode::ForLoop(self)
    }
}


#[derive(Debug, PartialEq)]
pub struct BreakNode;
impl Executable for BreakNode {
    fn execute(&self, _runtime: &mut Runtime) -> Result<(), Error> {
        Err(Error::new(Break, Position::new(0, 0, 0)))
    }

    fn to_statement(self) -> StatementNode {
        StatementNode::Break(self)
    }
}


#[derive(Debug, PartialEq)]
pub struct ContinueNode;
impl Executable for ContinueNode {
    fn execute(&self, _runtime: &mut Runtime) -> Result<(), Error> {
        Err(Error::new(Continue, Position::new(0, 0, 0)))
    }

    fn to_statement(self) -> StatementNode {
        StatementNode::Continue(self)
    }
}


#[derive(Debug, PartialEq)]
pub struct ReturnNode {
    return_value: ExpressionNode,
}
impl ReturnNode {
    pub fn new(return_value: ExpressionNode) -> ReturnNode {
        ReturnNode { return_value }
    }
}
impl Executable for ReturnNode {
    fn execute(&self, runtime: &mut Runtime) -> Result<(), Error> {
        Err(Error::new(Return(self.return_value.evaluate(runtime)?), Position::new(0, 0, 0)))
    }

    fn to_statement(self) -> StatementNode {
        StatementNode::Return(self)
    }
}


// TODO: function definition node
#[derive(Debug, PartialEq)]
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

    fn to_statement(self) -> StatementNode {
        StatementNode::FunctionDefinition(self)
    }
}