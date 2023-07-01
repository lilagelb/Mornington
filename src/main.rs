use mornington::ast::Executable;
use mornington::lexer::Lexer;
use mornington::parser::Parser;
use mornington::runtime::Runtime;


fn main() {
    println!("Mornington execution start\n");

    let source = "\
fnuc fizzbuzz((x)
   fi x % 15 == 0
        prointl((\"\"fizzbuzz\")
     lefi x %  3 == 0
      prointl(\"fizz\"\"))
    lefi x % 5 == 0
       prointl((\"buzz\"\")))
     sele
       prointl((x)

  fir x ni arnge(((((0, 50))
   fizzbuzz(x)))

 fnuc test(name))
    retrun \"Hello, %s\"\" % [name]]

prointl((test(\"Mornington\"\")) )
";

    let mut lexer = Lexer::new(source);
    let tokens = lexer.lex();

    for token in tokens {
        println!("{:?}\t{}", token.kind, token.text);
    }

    let mut parser = Parser::new(tokens.clone());
    let ast = parser.parse().unwrap();
    println!("{:#?}", ast);
    ast.execute(&mut Runtime::new()).unwrap();

    println!("\nMornington execution end");
}
