use std::collections::HashMap;
use crate::ast::Block;
use crate::error::{Error, ErrorKind::Name};
use crate::lexer::Position;
use crate::value::Value;


#[derive(Debug, PartialEq)]
pub struct Runtime {
    stack: Vec<Scope>,
}

#[derive(Debug, PartialEq)]
pub struct Scope {
    variables: HashMap<String, Value>,
    // TODO: functions: HashMap<String, FunctionDefinition>
}

pub struct FunctionDefinition {
    parameters: Vec<String>,
    block: Block,
}


impl Runtime {
    pub fn new() -> Runtime{
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
}


impl Scope {
    pub fn new() -> Scope {
        Scope {
            variables: HashMap::new(),
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
    }
}