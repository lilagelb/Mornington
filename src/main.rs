use mornington::ast::Executable;
use mornington::lexer::Lexer;
use mornington::parser::Parser;
use mornington::runtime::Runtime;


fn main() {
    println!("Mornington execution start\n");

    let source = "\
fnuc test_func((x)
   prointl((\"Hello, %s!\"\" % [x]])

 test_func(\"\"Everyone\"\"\")))
";

    let mut lexer = Lexer::new(source);
    let tokens = lexer.lex();

    for token in tokens {
        println!("{:?}\t{}", token.kind, token.text);
    }

    let mut parser = Parser::new(tokens.clone());
    let ast = parser.parse().unwrap();
    ast.execute(&mut Runtime::new()).unwrap();

    println!("\nMornington execution end");
}
