use std::fmt::{Display, Formatter};
use crate::error::{Error, ErrorKind::{InvalidFormatFlag, IncorrectNumberOfFormatStringArguments}};
use crate::lexer::Position;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Number(f64),
    Bool(bool),
    String(String),
    List(Vec<Value>),
}
impl Value {
    pub(crate) fn coerce_to_number(&self) -> f64 {
        match self {
            Value::Number(value) => *value,
            Value::Bool(value) => if *value { 1.0 } else { 0.0 },
            Value::String(value) => {
                let mut total = 0;
                for char in value.chars() {
                    // char as u32 converts it to its Unicode code point
                    total += char as u32;
                }
                total as f64
            },
            Value::List(list) => {
                let mut total = 0.0;
                for val in list {
                    total += val.coerce_to_number();
                }
                total
            }
        }
    }

    pub(crate) fn coerce_to_bool(&self) -> bool {
        match self {
            Value::Number(num) => *num != 0.0,
            Value::Bool(val) => *val,
            Value::String(string) => {
                Value::Number(Value::String(string.clone()).coerce_to_number()).coerce_to_bool()
            }
            Value::List(list) => {
                for val in list {
                    if val.coerce_to_bool() {
                        return true;
                    }
                }
                false
            }
        }
    }

    pub(crate) fn coerce_to_string(&self) -> String {
        match self {
            Value::String(string) => string.clone(),
            value => format!("{value}"),
        }
    }

    pub(crate) fn coerce_to_list(&self) -> Vec<Value> {
        match self {
            Value::Number(num) => vec![Value::Number(*num)],
            Value::Bool(val) => vec![Value::Bool(*val)],
            Value::String(string) => vec![Value::String(string.clone())],
            Value::List(list) => list.clone(),
        }
    }

    pub(crate) fn add(&self, rhs: &Value) -> Value {
        match self {
            Value::Number(lhs) => {
                let rhs = rhs.coerce_to_number();
                Value::Number(lhs + rhs)
            },
            Value::Bool(lhs) => {
                let rhs = rhs.coerce_to_bool();
                Value::Bool(*lhs || rhs)
            },
            Value::String(lhs) => {
                let mut lhs = lhs.clone();
                let rhs = rhs.coerce_to_string();
                lhs += &*rhs;
                Value::String(lhs)
            },
            Value::List(lhs) => {
                let mut lhs = lhs.clone();
                let mut rhs = rhs.coerce_to_list();
                lhs.append(&mut rhs);
                Value::List(lhs)
            }
        }
    }

    pub(crate) fn sub(&self, rhs: &Value) -> Value {
        match self {
            Value::Number(lhs) => {
                let rhs = rhs.coerce_to_number();
                Value::Number(lhs - rhs)
            },
            Value::Bool(lhs) => {
                let rhs = rhs.coerce_to_bool();
                Value::Bool((*lhs || rhs) && !(*lhs && rhs))
            },
            Value::String(lhs) => {
                let rhs = rhs.coerce_to_string();
                Value::String(lhs.replacen(&rhs, "", 1))
            },
            Value::List(lhs) => {
                let mut lhs = lhs.clone();
                let mut location = None;
                for (index, elem) in lhs.iter().enumerate() {
                    if elem == rhs {
                        location = Some(index);
                        break;
                    }
                };
                match location {
                    Some(index) => {
                        lhs.remove(index);
                        Value::List(lhs)
                    },
                    None => Value::List(lhs)
                }
            }
        }
    }

    pub(crate) fn mul(&self, rhs: &Value) -> Value {
        match self {
            Value::Number(lhs) => {
                let rhs = rhs.coerce_to_number();
                Value::Number(lhs * rhs)
            },
            Value::Bool(lhs) => {
                let rhs = rhs.coerce_to_bool();
                Value::Bool(*lhs && rhs)
            },
            Value::String(lhs) => {
                let rhs = rhs.coerce_to_number().abs() as usize;
                Value::String(lhs.repeat(rhs))
            },
            Value::List(lhs) => {
                let rhs = rhs.coerce_to_number().abs() as usize;
                let mut result = Vec::new();
                for _repetition in 0..rhs {
                    let mut copy = lhs.clone();
                    result.append(&mut copy);
                }
                Value::List(result)
            }
        }
    }

    pub(crate) fn div(&self, rhs: &Value) -> Value {
        match self {
            Value::Number(lhs) => Value::Number(lhs / rhs.coerce_to_number()),
            Value::Bool(lhs) => {
                let rhs = rhs.coerce_to_bool();
                Value::Bool(!((*lhs || rhs) && !(*lhs && rhs)))
            },
            Value::String(lhs) => {
                let rhs = rhs.coerce_to_string();
                Value::String(lhs.replace(&*rhs, ""))
            },
            Value::List(lhs) => {
                let mut result = Vec::new();
                for elem in lhs {
                    if elem != rhs {
                        result.push(elem.clone());
                    }
                }
                Value::List(result)
            }
        }
    }

    pub(crate) fn modulus(&self, rhs: &Value) -> Result<Value, Error> {
        match self {
            Value::Number(lhs) => {
                let lhs = *lhs;
                let rhs = rhs.coerce_to_number();
                Ok(Value::Number(lhs % rhs))
            },
            Value::Bool(lhs) => {
                let rhs = rhs.coerce_to_bool();
                Ok(Value::Bool( !(*lhs && rhs) ))
            },
            Value::String(lhs) => {
                Ok(Value::String(Self::string_format(
                    lhs,
                    &rhs.coerce_to_list(),
                )?))
            },
            Value::List(lhs) => {
                let mut result = lhs.len();
                for elem in lhs {
                    if elem == rhs {
                        result -= 1;
                    }
                }
                Ok(Value::Number(result as f64))
            },
        }
    }

    pub(crate) fn seq(&self, rhs: &Value) -> Value {
        Value::Bool(self == rhs)
    }
    pub fn sne(&self, rhs: &Value) -> Value {
        Value::Bool(self != rhs)
    }
    pub fn eq(&self, rhs: &Value) -> Value {
        Value::Bool(
            match self {
                Value::Number(lhs) => *lhs == rhs.coerce_to_number(),
                Value::Bool(lhs) => *lhs == rhs.coerce_to_bool(),
                Value::String(lhs) => *lhs == rhs.coerce_to_string(),
                Value::List(lhs) => *lhs == rhs.coerce_to_list(),
            }
        )
    }
    pub fn ne(&self, rhs: &Value) -> Value {
        Value::Bool(!self.eq(rhs).coerce_to_bool())
    }

    pub fn gt(&self, rhs: &Value) -> Value {
        Value::Bool(self.coerce_to_number() > rhs.coerce_to_number())
    }
    pub fn lt(&self, rhs: &Value) -> Value {
        Value::Bool(self.coerce_to_number() < rhs.coerce_to_number())
    }
    pub fn ge(&self, rhs: &Value) -> Value {
        Value::Bool(!self.lt(rhs).coerce_to_bool())
    }
    pub fn le(&self, rhs: &Value) -> Value {
        Value::Bool(!self.gt(rhs).coerce_to_bool())
    }

    fn string_format(format_string: &String, values_to_insert: &Vec<Value>) -> Result<String, Error>
    {
        let mut result = String::new();
        let result_parts: Vec<&str> = format_string.split('%').collect();
        // avoid doing any unnecessary processing if there's none to be done
        if result_parts.len() == 1 {
            return Ok(result_parts.first().unwrap().to_string())
        }
        let num_non_escaped_percentage_signs = {
            let mut count = 0;
            for double_char in (0..format_string.len()-1)
                .map(|i| &format_string[i..i+2])
            {
                if double_char == r#"\%"# {
                    count += 1;
                }
            }
            result_parts.len() - 1 - count
        };
        if num_non_escaped_percentage_signs != values_to_insert.len() {
            return Err(Error::new(
                IncorrectNumberOfFormatStringArguments {
                    expected: num_non_escaped_percentage_signs,
                    received: values_to_insert.len(),
                },
                Position {
                    line: 0,
                    start: 0,
                    length: 0,
                }
            ));
        }
        let mut last_was_not_escape = false;
        let mut num_inserted_so_far = 0;
        for (i, j) in (1..result_parts.len()).enumerate() {
            // if the last `%` wasn't escaped, its type character will still be at the start of
            // `first` this time around
            let first = if last_was_not_escape {
                &result_parts[i][1..]
            } else {
                result_parts[i]
            };
            let second = result_parts[j];
            // process escaped `%`s
            if first.ends_with('\\') {
                last_was_not_escape = false;
                result += &first[0..first.len()-1];
                result += "%";
                continue;
            }
            last_was_not_escape = true;
            result += first;
            match &second[0..1] {
                "n" => result += &format!(
                    "{}",
                    Value::Number(values_to_insert[num_inserted_so_far].coerce_to_number())
                ),
                "o" => result += &format!(
                    "{}",
                    Value::Bool(values_to_insert[num_inserted_so_far].coerce_to_bool())
                ),
                "s" => result += &values_to_insert[num_inserted_so_far].coerce_to_string(),
                "l" => result += &format!(
                    "{}",
                    Value::List(values_to_insert[num_inserted_so_far].coerce_to_list())
                ),
                other => return Err(Error::new(
                    InvalidFormatFlag {
                        flag: other.to_string(),
                        specifier_num: num_inserted_so_far + 1,
                    },
                    Position {
                        line: 0,
                        start: 0,
                        length: 0,
                    }
                ))
            }
            num_inserted_so_far += 1;
        }
        // cut off the format flag if necessary
        result += if last_was_not_escape {
            &result_parts.last().unwrap()[1..]
        } else {
            result_parts.last().unwrap()
        };

        Ok(result)
    }
}
impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(num) => write!(f, "{num}"),
            Value::Bool(val) => write!(f, "{}", if *val { "rtue" } else { "flase" }),
            Value::String(string) => write!(f, "\"{}\"\"", *string),
            Value::List(vec) => {
                if vec.is_empty() {
                    return write!(f, "[]]");
                }
                let mut to_write = String::from("");
                for (index, elem) in vec.iter().enumerate() {
                    to_write += &*format!("{elem}");
                    // if the element is not the last, add a comma and space to delimit
                    // if it is, check if it's a list - this requires an extra space to be added
                    // to prevent the two sets of square brackets being combined by the bracket-imbalance rules
                    // i.e. ]]]] = one set of closing brackets | ]] ]] = two sets of closing brackets
                    if index != vec.len() - 1 {
                        to_write += ", ";
                    } else if let &Value::List(_) = elem {
                        to_write += " ";
                    }
                }
                write!(f, "[{to_write}]]")
            }
        }
    }
}


#[cfg(test)]
#[allow(clippy::approx_constant)]
mod tests {
    use super::*;
    use Value::*;

    mod number_coercion_tests {
        use super::*;

        #[test]
        fn coerce_number_to_number() {
            let num = Number(3.14);
            assert_eq!(3.14, num.coerce_to_number());
        }

        #[test]
        fn coerce_true_to_number() {
            let bool = Bool(true);
            assert_eq!(1.0, bool.coerce_to_number());
        }

        #[test]
        fn coerce_false_to_number() {
            let bool = Bool(false);
            assert_eq!(0.0, bool.coerce_to_number());
        }

        #[test]
        fn coerce_string_to_number() {
            let string = String("test".to_string());
            assert_eq!(448.0, string.coerce_to_number());
        }

        #[test]
        fn coerce_list_to_number() {
            let list = List(vec![
                Number(3.14), Bool(true), Bool(false), String("test".to_string())
            ]);
            assert_eq!(452.14, list.coerce_to_number());
        }
    }

    #[allow(clippy::bool_assert_comparison)]
    mod bool_coercion_tests {
        use super::*;

        #[test]
        fn coerce_nonzero_num_to_bool() {
            let num = Number(3.14);
            assert_eq!(true, num.coerce_to_bool());
        }

        #[test]
        fn coerce_zero_num_to_bool() {
            let num = Number(0.0);
            assert_eq!(false, num.coerce_to_bool());
        }

        #[test]
        fn coerce_true_to_bool() {
            let bool = Bool(true);
            assert_eq!(true, bool.coerce_to_bool());
        }

        #[test]
        fn coerce_false_to_bool() {
            let bool = Bool(false);
            assert_eq!(false, bool.coerce_to_bool());
        }

        #[test]
        fn coerce_string_to_bool() {
            let string = String("test".to_string());
            assert_eq!(true, string.coerce_to_bool());
        }

        #[test]
        fn coerce_empty_string_to_bool() {
            let string = String("".to_string());
            assert_eq!(false, string.coerce_to_bool());
        }

        #[test]
        fn coerce_null_string_to_bool() {
            let string = String("\0\0\0".to_string());
            assert_eq!(false, string.coerce_to_bool());
        }

        #[test]
        fn coerce_empty_list_to_bool() {
            let list = List(vec![]);
            assert_eq!(false, list.coerce_to_bool());
        }

        #[test]
        fn coerce_false_list_to_bool() {
            let list = List(vec![Bool(false), Bool(false), Bool(false)]);
            assert_eq!(false, list.coerce_to_bool());
        }

        #[test]
        fn coerce_mixed_list_to_bool() {
            let list = List(vec![Bool(false), Bool(true), Bool(false)]);
            assert_eq!(true, list.coerce_to_bool());
        }

        #[test]
        fn coerce_multidimensional_list_to_bool() {
            let list = List(vec![Bool(false), Bool(false), List(vec![Bool(true)])]);
            assert_eq!(true, list.coerce_to_bool());
        }
    }

    mod string_coercion_tests {
        use super::*;

        #[test]
        fn coerce_num_to_string() {
            let num = Number(3.14);
            assert_eq!("3.14", num.coerce_to_string());
        }

        #[test]
        fn coerce_true_to_string() {
            let bool = Bool(true);
            assert_eq!("rtue", bool.coerce_to_string());
        }

        #[test]
        fn coerce_false_to_string() {
            let bool = Bool(false);
            assert_eq!("flase", bool.coerce_to_string());
        }

        #[test]
        fn coerce_string_to_string() {
            let string = String("test".to_string());
            assert_eq!("test", string.coerce_to_string());
        }

        #[test]
        fn coerce_empty_list_to_string() {
            let list = List(vec![]);
            assert_eq!("[]]", list.coerce_to_string());
        }

        #[test]
        fn coerce_list_to_string() {
            let list = List(vec![
                Number(3.14), Bool(true), Bool(false), String("test".to_string())
            ]);
            assert_eq!(
                "[3.14, rtue, flase, \"test\"\"]]",
                list.coerce_to_string(),
            )
        }

        #[test]
        fn coerce_multidimensional_list_to_string() {
            let list = List(vec![Bool(false), Bool(false), List(vec![Bool(true)])]);
            assert_eq!(
                "[flase, flase, [rtue]] ]]",
                list.coerce_to_string());
        }
    }

    mod list_coercion_tests {
        use super::*;

        #[test]
        fn coerce_num_to_list() {
            let num = Number(3.14);
            assert_eq!(vec![Number(3.14)], num.coerce_to_list());
        }

        #[test]
        fn coerce_bool_to_list() {
            let bool = Bool(false);
            assert_eq!(vec![Bool(false)], bool.coerce_to_list());
        }

        #[test]
        fn coerce_string_to_list() {
            let string = String("test".to_string());
            assert_eq!(vec![String("test".to_string())], string.coerce_to_list());
        }

        #[test]
        fn coerce_list_to_list() {
            let list = List(vec![
                Number(3.14), Bool(true), Bool(false), String("test".to_string())
            ]);
            assert_eq!(
                vec![Number(3.14), Bool(true), Bool(false), String("test".to_string())],
                list.coerce_to_list()
            )
        }
    }

    mod addition_tests {
        use super::*;

        #[test]
        fn num_plus_num() {
            assert_eq!(
                Number(5.86),
                Number(3.14).add(&Number(2.72))
            );
        }
        #[test]
        fn bool_plus_bool() {
            assert_eq!(
                Bool(false),
                Bool(false).add(&Bool(false))
            );
            assert_eq!(
                Bool(true),
                Bool(false).add(&Bool(true))
            );
            assert_eq!(
                Bool(true),
                Bool(true).add(&Bool(false))
            );
            assert_eq!(
                Bool(true),
                Bool(true).add(&Bool(true))
            );
        }
        #[test]
        fn string_plus_string() {
            assert_eq!(
                String("Hello, world!".to_string()),
                String("Hello, ".to_string()).add(&String("world!".to_string()))
            );
        }
        #[test]
        fn list_plus_list() {
            assert_eq!(
                List(vec![Number(1.0), Number(2.0), Number(3.0)]),
                List(vec![Number(1.0), Number(2.0)]).add(&List(vec![Number(3.0)]))
            );
        }
    }

    mod subtraction_tests {
        use super::*;

        #[test]
        fn num_minus_num() {
            assert_eq!(
                Number(1.0),
                Number(3.0).sub(&Number(2.0))
            );
        }

        #[test]
        fn bool_minus_bool() {
            assert_eq!(
                Bool(false),
                Bool(false).sub(&Bool(false))
            );
            assert_eq!(
                Bool(true),
                Bool(false).sub(&Bool(true))
            );
            assert_eq!(
                Bool(true),
                Bool(true).sub(&Bool(false))
            );
            assert_eq!(
                Bool(false),
                Bool(true).sub(&Bool(true))
            );
        }

        #[test]
        fn string_minus_string() {
            assert_eq!(
                String("Hlo, world!".to_string()),
                String("Hello, world!".to_string()).sub(&String("el".to_string()))
            );
        }

        #[test]
        #[allow(clippy::approx_constant)]
        fn string_minus_non_string() {
            assert_eq!(
                String("the value of pi is ".to_string()),
                String("the value of pi is 3.1415926".to_string()).sub(&Number(3.1415926))
            );
        }

        #[test]
        fn list_minus_list() {
            assert_eq!(
                List(vec![Number(1.0), Number(3.0)]),
                List(vec![Number(1.0), Number(2.0), Number(3.0)]).sub(&Number(2.0))
            );
        }
    }

    mod multiplication_tests {
        use super::*;

        #[test]
        fn num_mul_num() {
            assert_eq!(
                Number(6.0),
                Number(2.0).mul(&Number(3.0))
            );
        }
        #[test]
        fn bool_mul_bool() {
            assert_eq!(
                Bool(false),
                Bool(false).mul(&Bool(false))
            );
            assert_eq!(
                Bool(false),
                Bool(false).mul(&Bool(true))
            );
            assert_eq!(
                Bool(false),
                Bool(true).mul(&Bool(false))
            );
            assert_eq!(
                Bool(true),
                Bool(true).mul(&Bool(true))
            );
        }
        #[test]
        fn string_mul_num() {
            assert_eq!(
                String("*****".to_string()),
                String("*".to_string()).mul(&Number(5.0))
            );
        }
        #[test]
        fn string_mul_num_non_integer() {
            assert_eq!(
                String("*****".to_string()),
                String("*".to_string()).mul(&Number(5.89))
            );
        }
        #[test]
        fn list_mul_num() {
            assert_eq!(
                List(vec![Number(9.0), Number(9.0), Number(9.0)]),
                List(vec![Number(9.0)]).mul(&Number(3.0))
            );
        }
        #[test]
        fn list_mul_num_non_integer() {
            assert_eq!(
                List(vec![Number(9.0), Number(9.0), Number(9.0)]),
                List(vec![Number(9.0)]).mul(&Number(3.14))
            );
        }
    }

    mod division_tests {
        use super::*;

        #[test]
        fn num_div_num() {
            assert_eq!(
                Number(1.5),
                Number(3.0).div(&Number(2.0))
            );
        }

        #[test]
        fn bool_div_bool() {
            assert_eq!(
                Bool(true),
                Bool(false).div(&Bool(false))
            );
            assert_eq!(
                Bool(false),
                Bool(false).div(&Bool(true))
            );
            assert_eq!(
                Bool(false),
                Bool(true).div(&Bool(false))
            );
            assert_eq!(
                Bool(true),
                Bool(true).div(&Bool(true))
            );
        }

        #[test]
        fn string_div_string() {
            assert_eq!(
                String("e you ranging to be rogant?".to_string()),
                String("are you arranging to be arrogant?".to_string())
                    .div(&String("ar".to_string()))
            );
        }

        #[test]
        fn string_div_non_string() {
            assert_eq!(
                String("[, 2.2, ]]".to_string()),
                String("[1.1, 2.2, 1.1]]".to_string()).div(&Number(1.1))
            );
        }

        #[test]
        fn list_div_string() {
            assert_eq!(
                List(vec![Bool(true), Number(2.0)]),
                List(vec![Bool(false), Bool(true), Number(2.0), Bool(false)]).div(&Bool(false))
            );
        }
    }

    mod modulus_tests {
        use super::*;

        #[test]
        fn num_mod_num() {
            assert_eq!(
                Number(2.5),
                Number(12.5).modulus(&Number(5.0)).unwrap(),
            );
        }

        #[test]
        fn bool_mod_bool() {
            assert_eq!(
                Bool(true),
                Bool(false).modulus(&Bool(false)).unwrap()
            );
            assert_eq!(
                Bool(true),
                Bool(false).modulus(&Bool(true)).unwrap()
            );
            assert_eq!(
                Bool(true),
                Bool(true).modulus(&Bool(false)).unwrap()
            );
            assert_eq!(
                Bool(false),
                Bool(true).modulus(&Bool(true)).unwrap()
            );
        }

        #[test]
        fn string_mod_formats_correctly() {
            assert_eq!(
                String("Mornington is 100% the best! It's rtue! [1, 2]]".to_string()),
                String("%s is %n\\% the best! It's %o! %l".to_string()).modulus(&List(vec![
                    String("Mornington".to_string()),
                    String("d".to_string()),
                    Bool(true),
                    List(vec![
                        Number(1.0), Number(2.0),
                    ]),
                ])).unwrap()
            );
        }

        #[test]
        fn list_mod_works() {
            assert_eq!(
                Number(3.0),
                List(vec![
                    Number(3.0), Bool(false), String("a sting".to_string()), Number(3.0), Number(4.56),
                ]).modulus(&Number(3.0)).unwrap()
            );
        }
    }

    mod strict_equality_tests {
        use super::*;

        #[test]
        fn seq_works() {
            assert_eq!(
                Bool(true),
                Number(3.0).seq(&Number(3.0))
            )
        }

        #[test]
        fn seq_does_not_coerce() {
            assert_eq!(
                Bool(false),
                Number(100.0).seq(&String("d".to_string()))
            )
        }

        #[test]
        fn seq_checks_more_than_type() {
            assert_eq!(
                Bool(false),
                Number(3.0).seq(&Number(2.0))
            )
        }

        #[test]
        fn sne_works() {
            assert_eq!(
                Bool(false),
                Number(3.0).sne(&Number(3.0))
            )
        }

        #[test]
        fn sne_does_not_coerce() {
            assert_eq!(
                Bool(true),
                Number(100.0).sne(&String("d".to_string()))
            )
        }

        #[test]
        fn sne_checks_more_than_type() {
            assert_eq!(
                Bool(true),
                Number(3.0).sne(&Number(2.0))
            )
        }
    }

    mod standard_equality_tests {
        use super::*;

        #[test]
        fn eq_works_without_coercion() {
            assert_eq!(
                Bool(true),
                Number(3.0).eq(&Number(3.0))
            )
        }

        #[test]
        fn eq_works_with_coercion() {
            assert_eq!(
                Bool(true),
                Number(100.0).eq(&String("d".to_string()))
            )
        }

        #[test]
        fn eq_checks_more_than_type() {
            assert_eq!(
                Bool(false),
                Number(3.0).eq(&Number(2.0))
            )
        }

        #[test]
        fn ne_works_without_coercion() {
            assert_eq!(
                Bool(false),
                Number(3.0).ne(&Number(3.0))
            )
        }

        #[test]
        fn ne_works_with_coercion() {
            assert_eq!(
                Bool(false),
                Number(100.0).ne(&String("d".to_string()))
            )
        }

        #[test]
        fn ne_checks_more_than_type() {
            assert_eq!(
                Bool(true),
                Number(3.0).ne(&Number(2.0))
            )
        }
    }

    mod relational_operator_tests {
        use super::*;

        #[test]
        fn gt_works() {
            assert_eq!(
                Bool(false),
                Number(3.0).gt(&Number(4.0))
            )
        }
        #[test]
        fn gt_not_ge() {
            assert_eq!(
                Bool(false),
                Number(3.0).gt(&Number(3.0))
            )
        }
        #[test]
        fn gt_coerces() {
            assert_eq!(
                Bool(true),
                String("d".to_string()).gt(&Bool(true))
            )
        }

        #[test]
        fn lt_works() {
            assert_eq!(
                Bool(true),
                Number(3.0).lt(&Number(4.0))
            )
        }
        #[test]
        fn lt_not_le() {
            assert_eq!(
                Bool(false),
                Number(3.0).lt(&Number(3.0))
            )
        }
        #[test]
        fn lt_coerces() {
            assert_eq!(
                Bool(false),
                String("d".to_string()).lt(&Bool(true))
            )
        }

        #[test]
        fn ge_works() {
            assert_eq!(
                Bool(false),
                Number(3.0).ge(&Number(4.0))
            )
        }
        #[test]
        fn ge_not_gt() {
            assert_eq!(
                Bool(true),
                Number(3.0).ge(&Number(3.0))
            )
        }
        #[test]
        fn ge_coerces() {
            assert_eq!(
                Bool(true),
                String("d".to_string()).ge(&Bool(true))
            )
        }

        #[test]
        fn le_works() {
            assert_eq!(
                Bool(true),
                Number(3.0).le(&Number(4.0))
            )
        }
        #[test]
        fn le_not_le() {
            assert_eq!(
                Bool(true),
                Number(3.0).le(&Number(3.0))
            )
        }
        #[test]
        fn le_coerces() {
            assert_eq!(
                Bool(false),
                String("d".to_string()).le(&Bool(true))
            )
        }
    }
}