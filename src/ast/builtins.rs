use std::io;
use crate::ast::{Evaluable, ListNode};
use crate::error::Error;
use crate::error::ErrorKind::{Input, Signature};
use crate::lexer::Position;
use crate::runtime::Runtime;
use crate::value::Value;

pub fn print(runtime: &mut Runtime, args: &ListNode) -> Result<Value, Error> {
    for arg in &args.list {
        print!("{}", arg.evaluate(runtime)?.coerce_to_string());
    }
    Ok(Value::List(vec![]))
}

pub fn println(runtime: &mut Runtime, args: &ListNode) -> Result<Value, Error> {
    for arg in &args.list {
        print!("{}", arg.evaluate(runtime)?.coerce_to_string());
    }
    println!();
    Ok(Value::List(vec![]))
}

pub fn printerr(runtime: &mut Runtime, args: &ListNode) -> Result<Value, Error> {
    for arg in &args.list {
        eprint!("{}", arg.evaluate(runtime)?.coerce_to_string());
    }
    Ok(Value::List(vec![]))
}

pub fn printlnerr(runtime: &mut Runtime, args: &ListNode) -> Result<Value, Error> {
    for arg in &args.list {
        eprint!("{}", arg.evaluate(runtime)?.coerce_to_string());
    }
    eprintln!();
    Ok(Value::List(vec![]))
}

pub fn input() -> Result<Value, Error> {
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => Ok(Value::String(input)),
        Err(_) => Err(Error::new(Input, Position::new(0, 0, 0)))
    }
}


pub fn range(runtime: &mut Runtime, args: &ListNode) -> Result<Value, Error> {
    let num_args = args.list.len();
    if num_args == 0 || num_args > 3 {
        return Err(Error::new(
            Signature { 
                function_name: "arnge".to_string(), 
                expected_args: 3, 
                passed_args: num_args 
            },
            Position::new(0, 0, 0),
        ))
    }

    let finish = args.list.last().unwrap().evaluate(runtime)?.coerce_to_number();
    let start = if num_args == 1 {
        0.0
    } else {
        args.list[0].evaluate(runtime)?.coerce_to_number()
    };
    let step = if num_args == 3 {
        args.list[1].evaluate(runtime)?.coerce_to_number()
    } else {
        1.0
    };

    let mut sequence = Vec::new();
    let mut current = start;
    while current < finish {
        sequence.push(Value::Number(current));
        current += step;
    }
    Ok(Value::List(sequence))
}