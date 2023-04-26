use std::collections::HashMap;
use crate::ast::{Block, FunctionDefinitionNode};
use crate::error::{Error, ErrorKind::Name};
use crate::lexer::Position;
use crate::value::Value;


#[derive(Debug, Default, PartialEq)]
pub struct Runtime<'a> {
    stack: Vec<Scope<'a>>,
}

#[derive(Debug, Default, PartialEq)]
pub struct Scope<'a> {
    variables: HashMap<String, Value>,
    functions: HashMap<String, &'a FunctionDefinitionNode>
}


impl<'a> Runtime<'a> {
    pub fn new() -> Runtime<'a> {
        Runtime {
            stack: vec![Scope::new()],
        }
    }

    pub fn begin_scope(&mut self) {
        self.stack.push(Scope::new());
    }
    pub fn end_scope(&mut self) {
        self.stack.pop();
    }

    pub fn get_variable(&self, name: &str) -> Result<&Value, Error> {
        for scope in self.stack.iter().rev() {
            if let Some(value) = scope.get_variable(name) {
                return Ok(value);
            }
        }
        Err(Error::new(Name(name.to_string()), Position::new(0, 0, 0)))
    }

    pub fn set_variable(&mut self, name: &str, value: Value) {
        for scope in self.stack.iter_mut().rev() {
            if scope.get_variable(name).is_some() {
                scope.set_variable(name, value);
                return;
            }
        }
        self.stack.last_mut().unwrap().set_variable(name, value);
    }

    pub fn get_function_definition(&self, name: &str) -> Result<&&FunctionDefinitionNode, Error> {
        for scope in self.stack.iter().rev() {
            if let Some(definition) = scope.get_function_definition(name) {
                return Ok(definition)
            }
        }
        Err(Error::new(Name(name.to_string()), Position::new(0, 0, 0)))
    }

    pub fn set_function_definition(&mut self, name: &str, definition: &'a FunctionDefinitionNode) {
        let top_scope = self.stack.last_mut().expect("`set_function_definition()` called after last scope closed");
        top_scope.set_function_definition(name, definition);
    }
}


impl<'a> Scope<'a> {
    pub fn new() -> Scope<'a> {
        Scope {
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    pub fn set_variable(&mut self, name: &str, value: Value) {
        if let Some(stored_value) = self.variables.get_mut(name) {
            *stored_value = value;
        } else {
            self.variables.insert(name.to_string(), value);
        }
    }

    pub fn get_function_definition(&self, name: &str) -> Option<&&FunctionDefinitionNode> {
        self.functions.get(name)
    }

    pub fn set_function_definition(&mut self, name: &str, definition: &'a FunctionDefinitionNode) {
        if let Some(existing_definition) = self.functions.get_mut(name) {
            *existing_definition = definition;
        } else {
            self.functions.insert(name.to_string(), definition);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    mod runtime_tests {
        use super::*;

        #[test]
        fn get_variable_takes_uppermost_value() {
            let runtime = Runtime {
                stack: vec![
                    {
                        let mut scope = Scope::new();
                        scope.set_variable("a", Value::Bool(false));
                        scope
                    }, {
                        let mut scope = Scope::new();
                        scope.set_variable("a", Value::Bool(true));
                        scope
                    }
                ]
            };
            assert_eq!(
                Value::Bool(true),
                *runtime.get_variable("a").unwrap()
            );
        }

        #[test]
        fn get_variable_digs_stack_if_necessary() {
            let runtime = Runtime {
                stack: vec![
                    {
                        let mut scope = Scope::new();
                        scope.set_variable("b", Value::Bool(false));
                        scope
                    }, {
                        let mut scope = Scope::new();
                        scope.set_variable("a", Value::Bool(true));
                        scope
                    }
                ]
            };
            assert_eq!(
                Value::Bool(false),
                *runtime.get_variable("b").unwrap(),
            )
        }

        #[test]
        fn get_variable_throws_name_error_if_variable_not_found() {
            let runtime = Runtime::new();
            assert_eq!(
                Err(Error::new(Name("test".to_string()), Position::new(0, 0, 0))),
                runtime.get_variable("test"),
            )
        }

        #[test]
        fn set_variable_sets_uppermost_value() {
            let mut runtime = Runtime {
                stack: vec![
                    {
                        let mut scope = Scope::new();
                        scope.set_variable("a", Value::Bool(false));
                        scope
                    }, {
                        let mut scope = Scope::new();
                        scope.set_variable("a", Value::Bool(true));
                        scope
                    }
                ]
            };
            runtime.set_variable("a", Value::Number(3.0));
            assert_eq!(
                Runtime {
                    stack: vec![
                        {
                            let mut scope = Scope::new();
                            scope.set_variable("a", Value::Bool(false));
                            scope
                        }, {
                            let mut scope = Scope::new();
                            scope.set_variable("a", Value::Number(3.0));
                            scope
                        }
                    ]
                },
                runtime,
            );
        }

        #[test]
        fn set_variable_digs_stack_in_preference_to_creating_new_variable() {
            let mut runtime = Runtime {
                stack: vec![
                    {
                        let mut scope = Scope::new();
                        scope.set_variable("a", Value::Bool(false));
                        scope
                    }, {
                        let mut scope = Scope::new();
                        scope.set_variable("b", Value::Bool(true));
                        scope
                    }
                ]
            };
            runtime.set_variable("a", Value::Number(3.0));
            assert_eq!(
                Runtime {
                    stack: vec![
                        {
                            let mut scope = Scope::new();
                            scope.set_variable("a", Value::Number(3.0));
                            scope
                        }, {
                            let mut scope = Scope::new();
                            scope.set_variable("b", Value::Bool(true));
                            scope
                        }
                    ]
                },
                runtime,
            );
        }

        #[test]
        fn set_variable_creates_new_variable_in_highest_scope_if_none_of_name_exist() {
            let mut runtime = Runtime {
                stack: vec![
                    {
                        let mut scope = Scope::new();
                        scope.set_variable("a", Value::Bool(false));
                        scope
                    }, {
                        let mut scope = Scope::new();
                        scope.set_variable("b", Value::Bool(true));
                        scope
                    }
                ]
            };
            runtime.set_variable("c", Value::Number(3.0));
            assert_eq!(
                Runtime {
                    stack: vec![
                        {
                            let mut scope = Scope::new();
                            scope.set_variable("a", Value::Bool(false));
                            scope
                        }, {
                            let mut scope = Scope::new();
                            scope.set_variable("b", Value::Bool(true));
                            scope.set_variable("c", Value::Number(3.0));
                            scope
                        },
                    ]
                },
                runtime,
            );
        }

        #[test]
        fn get_function_definition_takes_uppermost_definition() {
            let lower_definition = generic_function_definition_returning(Value::Bool(false));
            let upper_definition = generic_function_definition_returning(Value::Bool(true));

            let runtime = Runtime {
                stack: vec![
                    {
                        let mut scope = Scope::new();
                        scope.set_function_definition("a", &lower_definition);
                        scope
                    }, {
                        let mut scope = Scope::new();
                        scope.set_function_definition("a", &upper_definition);
                        scope
                    }
                ]
            };
            assert_eq!(
                upper_definition,
                **runtime.get_function_definition("a").unwrap()
            );
        }

        #[test]
        fn get_function_definition_digs_stack_if_necessary() {
            let b_definition = generic_function_definition_returning(Value::Bool(false));
            let a_definition = generic_function_definition_returning(Value::Bool(true));

            let runtime = Runtime {
                stack: vec![
                    {
                        let mut scope = Scope::new();
                        scope.set_function_definition("b", &b_definition);
                        scope
                    }, {
                        let mut scope = Scope::new();
                        scope.set_function_definition("a", &a_definition);
                        scope
                    }
                ]
            };
            assert_eq!(
                b_definition,
                **runtime.get_function_definition("b").unwrap(),
            );
        }

        #[test]
        fn get_function_definition_throws_name_error_if_definition_not_found() {
            let runtime = Runtime::new();
            assert_eq!(
                Err(Error::new(Name("test".to_string()), Position::new(0, 0, 0))),
                runtime.get_function_definition("test"),
            );
        }

        #[test]
        fn set_function_defines_new_function_in_highest_scope_if_no_existing_definition() {
            let definition = generic_function_definition_returning(Value::Bool(false));
            let mut runtime = Runtime::new();
            runtime.set_function_definition("test", &definition);
            assert_eq!(
                Runtime {
                    stack: vec![
                        {
                            let mut scope = Scope::new();
                            scope.set_function_definition("test", &definition);
                            scope
                        },
                    ]
                },
                runtime,
            );
        }

        #[test]
        fn set_function_defines_new_function_in_highest_scope_if_there_are_no_definitions_in_the_highest_scope() {
            let lower_definition = generic_function_definition_returning(Value::Bool(false));
            let upper_definition = generic_function_definition_returning(Value::Bool(true));
            let mut runtime = Runtime {
                stack: vec![
                    {
                        let mut scope = Scope::new();
                        scope.set_function_definition("test", &lower_definition);
                        scope
                    },
                    Scope::new(),
                ]
            };
            runtime.set_function_definition("test", &upper_definition);
            assert_eq!(
                Runtime {
                    stack: vec![
                        {
                            let mut scope = Scope::new();
                            scope.set_function_definition("test", &lower_definition);
                            scope
                        },
                        {
                            let mut scope = Scope::new();
                            scope.set_function_definition("test", &upper_definition);
                            scope
                        },
                    ]
                },
                runtime,
            );
        }

        #[test]
        fn set_function_overwrites_function_in_highest_scope_if_existing_definition() {
            let lower_definition = generic_function_definition_returning(Value::Bool(true));
            let upper_definition = generic_function_definition_returning(Value::Bool(false));
            let mut runtime = Runtime {
                stack: vec![
                    {
                        let mut scope = Scope::new();
                        scope.set_function_definition("a", &lower_definition);
                        scope
                    }, {
                        let mut scope = Scope::new();
                        scope.set_function_definition("a", &upper_definition);
                        scope
                    }
                ]
            };
            let replacement_definition = generic_function_definition_returning(Value::Number(3.0));
            runtime.set_function_definition("a", &replacement_definition);
            assert_eq!(
                Runtime {
                    stack: vec![
                        {
                            let mut scope = Scope::new();
                            scope.set_function_definition("a", &lower_definition);
                            scope
                        }, {
                            let mut scope = Scope::new();
                            scope.set_function_definition("a", &replacement_definition);
                            scope
                        }
                    ]
                },
                runtime,
            );
        }
    }

    mod scope_tests {
        use super::*;

        #[test]
        fn defined_variable_get_yields_value() {
            let scope = Scope {
                variables: HashMap::from([
                    ("a".to_string(), Value::Number(1.0)),
                    ("b".to_string(), Value::Bool(false)),
                    ("c".to_string(), Value::String("test".to_string())),
                ]),
                functions: HashMap::new(),
            };
            assert_eq!(
                Value::Number(1.0),
                *scope.get_variable("a").unwrap(),
            );
            assert_eq!(
                Value::String("test".to_string()),
                *scope.get_variable("c").unwrap(),
            );
            assert_eq!(
                Value::Bool(false),
                *scope.get_variable("b").unwrap(),
            );
        }

        #[test]
        fn undefined_variable_get_yields_none() {
            let scope = Scope {
                variables: HashMap::from([
                    ("a".to_string(), Value::Number(1.0)),
                ]),
                functions: HashMap::new(),
            };
            assert_eq!(
                None,
                scope.get_variable("test"),
            )
        }

        #[test]
        fn set_variable_creates_variable_if_not_already_defined() {
            let mut scope = Scope::new();
            scope.set_variable("a", Value::Number(2.0));
            assert_eq!(
                Value::Number(2.0),
                *scope.get_variable("a").unwrap(),
            );
        }

        #[test]
        fn set_variable_overwrites_existing_data() {
            let mut scope = Scope::new();
            scope.set_variable("a", Value::Bool(false));
            scope.set_variable("a", Value::Bool(true));
            assert_eq!(
                Value::Bool(true),
                *scope.get_variable("a").unwrap(),
            );
        }

        #[test]
        fn defined_function_get_yields_definition() {
            let definition = generic_function_definition_returning(Value::Bool(true));
            let scope = Scope {
                variables: HashMap::new(),
                functions: HashMap::from([
                    ("test".to_string(), &definition)
                ]),
            };
            assert_eq!(
                definition,
                **scope.get_function_definition("test").unwrap(),
            );
        }

        #[test]
        fn undefined_function_get_yields_none() {
            let scope = Scope {
                variables: HashMap::new(),
                functions: HashMap::new(),
            };
            assert_eq!(
                None,
                scope.get_function_definition("test"),
            );
        }

        #[test]
        fn set_function_creates_function_definition_if_not_already_defined() {
            let definition = generic_function_definition_returning(Value::Bool(true));

            let mut scope = Scope::new();
            scope.set_function_definition("test", &definition);

            assert_eq!(
                definition,
                **scope.get_function_definition("test").unwrap(),
            );
        }

        #[test]
        fn set_function_overwrites_existing_definitions() {
            let definition_old = generic_function_definition_returning(Value::Bool(true));
            let definition_new = generic_function_definition_returning(Value::Bool(false));

            let mut scope = Scope::new();
            scope.set_function_definition("a", &definition_old);
            scope.set_function_definition("a", &definition_new);
            assert_eq!(
                definition_new,
                **scope.get_function_definition("a").unwrap(),
            );
        }
    }

    fn generic_function_definition_returning(return_value: Value) -> FunctionDefinitionNode {
        use crate::ast::{ConstantNode, ExpressionNode, ReturnNode, StatementNode};

        let mut function_block = Block::new();
        function_block.add_statement(StatementNode::Return(ReturnNode::new(
            ExpressionNode::Constant(ConstantNode::new(
                return_value
            ))
        )));
        FunctionDefinitionNode::new("test".to_string(), vec![], function_block)
    }
}