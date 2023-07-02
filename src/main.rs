use std::{env, fs};
use mornington::ast::Executable;
use mornington::error::{Error, ErrorKind};
use mornington::lexer::{Lexer, Position, TokenKind};
use mornington::parser::Parser;
use mornington::runtime::Runtime;


fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        eprintln!("Error: no file passed for execution. Supply one using\n\
            \tmornington <filename>\n\
            Terminating..."
        );
        return;
    } else if args.len() > 2 {
        println!("Warning: more than one file passed for execution. \
            All but the first will be disregarded."
        );
    }

    let source_filepath = &args[1];
    let source = match fs::read_to_string(source_filepath) {
        Ok(source) => source,
        Err(_) => {
            eprintln!("Error: unable to read file `{source_filepath}`.\nTerminating...");
            return;
        }
    };

    let mut lexer = Lexer::new(&source);
    let tokens = match lexer.lex() {
        Ok(tokens) => tokens,
        Err(position) => {
            eprintln!("Error: Unexpected Symbol");
            print_error_position(&source, position);
            return;
        }
    };

    if tokens.is_empty() {
        return;
    }

    let mut parser = Parser::new(tokens.clone());
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(Error { pos, kind}) => {
            eprintln!("Error: {}", error_kind_to_print_name(kind));
            if let Some(position) = pos {
                print_error_position(&source, position);
            }
            return;
        }
    };

    match ast.execute(&mut Runtime::new()) {
        Ok(_) => {},
        Err(Error { pos, kind}) => {
            eprintln!("Error: {}", error_kind_to_print_name(kind));
            if let Some(position) = pos {
                print_error_position(&source, position);
            }
        }
    };
}


fn print_error_position(source: &String, position: Position) {
    let margin_width = (source.len().ilog10() + 2) as usize;
    let source_line = source.lines().nth(position.line - 1).unwrap();
    println!("{line_number:>margin_width$} | {source_line}", line_number=position.line);
    println!("{0:>margin_width$} | {0:>start$}{indicator}",
             "",
             indicator="^".repeat(position.length),
             start=position.start);
    println!("{0:>margin_width$} | {0:>start$}here",
             "",
             start=position.start);
}

fn error_kind_to_print_name(kind: ErrorKind) -> String {
    match kind {
        ErrorKind::Balance { opener, closer } => {
            format!("Wrapper Balance: closing `{closer}` balances opening `{opener}`")
        }
        ErrorKind::UnexpectedToken(kind) => {
            format!("Unexpected Token: `{}`", token_kind_to_print_name(kind))
        }
        ErrorKind::UnexpectedEOF => {"Unexpected End Of File".to_string()}
        ErrorKind::MissingToken(kind) => {
            format!("Missing Token: expected `{}`", token_kind_to_print_name(kind))
        }
        ErrorKind::MissingExpression => {"Missing Expression".to_string()}
        ErrorKind::InvalidFormatFlag { flag, specifier_num } => {
            format!("Invalid Sting Format Flag: `{flag}` (flag number {specifier_num})")
        }
        ErrorKind::IncorrectNumberOfFormatStringArguments { expected, received } => {
            format!("Incorrect Number Of Format String Arguments: \
                     expected {expected}, got {received}")
        }
        ErrorKind::Name(name) => {
            format!("Name Not Found: `{name}`")
        }
        ErrorKind::ConsistentIndentation { previous_indentation } => {
            format!("Consistent Indentation: \
                     indentation consistent with previous line at depth {previous_indentation}")
        }
        ErrorKind::Signature { function_name, expected_args, passed_args } => {
            format!("Function Signature: function `{function_name}` \
                     takes {expected_args} arguments but {passed_args} were passed")
        }
        ErrorKind::Input => {"Could Not Read Stdin".to_string()}
        ErrorKind::Break | ErrorKind::Continue | ErrorKind::Return(_)
            => panic!("Non-error propagated to interface")
    }
}


fn token_kind_to_print_name(kind: TokenKind) -> String {
    match kind {
        TokenKind::Newline   => {"newline"}
        TokenKind::LParen    => {"left parenthesis"}
        TokenKind::RParen    => {"right parenthesis"}
        TokenKind::LBrack    => {"left bracket"}
        TokenKind::RBrack    => {"right bracket"}
        TokenKind::Comma     => {"comma"}
        TokenKind::FullStop  => {"full stop"}
        TokenKind::Plus      => {"plus"}
        TokenKind::Minus     => {"minus"}
        TokenKind::Mul       => {"star"}
        TokenKind::Div       => {"forward slash"}
        TokenKind::Mod       => {"percent sign"}
        TokenKind::Eq        => {"equal"}
        TokenKind::Ne        => {"not equal"}
        TokenKind::Seq       => {"strict equal"}
        TokenKind::Sne       => {"strict not equal"}
        TokenKind::Gt        => {"greater than"}
        TokenKind::Lt        => {"less than"}
        TokenKind::Ge        => {"greater than or equal to"}
        TokenKind::Le        => {"less than or equal to"}
        TokenKind::Assign    => {"assign"}
        TokenKind::If        => {"fi"}
        TokenKind::Elif      => {"lefi"}
        TokenKind::Else      => {"sele"}
        TokenKind::While     => {"whitl"}
        TokenKind::For       => {"fir"}
        TokenKind::In        => {"ni"}
        TokenKind::Break     => {"brek"}
        TokenKind::Continue  => {"cnotineu"}
        TokenKind::Funcdef   => {"fnuc"}
        TokenKind::Return    => {"retrun"}
        TokenKind::BoolTrue  => {"rtue"}
        TokenKind::BoolFalse => {"flase"}
        TokenKind::Number    => {"nmu"}
        TokenKind::String    => {"sting"}
        TokenKind::Name      => {"name"}
    }.to_string()
}