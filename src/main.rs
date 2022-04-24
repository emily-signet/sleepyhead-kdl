use sleepyhead_kdl::ast::*;
use sleepyhead_kdl::lex::Token;
use easybench::bench;
use logos::Logos;

fn main() {
    let input = include_str!("../to_parse.kdl");
    println!(
        "self: {}",
        easybench::bench(|| {
            let mut parser = Parser::new(Token::lexer(input));
            while let Some(node) = parser.next() {
                let node = node.unwrap();
            }
        })
    );

    println!(
        "kdl-rs: {}",
        easybench::bench(|| {
            let _: kdl::KdlDocument = input.parse().unwrap();
        })
    );
    let mut ident = 0;
    let mut parser = Parser::new(Token::lexer(input));
    while let Some(node) = parser.next() {
        let node = node.unwrap();
    
        match node {
            KdlEvent::NodeOpen {
                name,
                attrs,
                values,
                has_children
            } => {
                println!(
                    "{}<{} {} {}>",
                    " ".repeat(ident),
                    name,
                    attrs
                        .into_iter()
                        .map(|p| format!("{}={:?}", p.key, p.value))
                        .collect::<Vec<String>>()
                        .join(" "),
                    values
                        .into_iter()
                        .map(|v| format!("{v:?}"))
                        .collect::<Vec<String>>()
                        .join(" ")
                );

                ident += 2;
            }
            KdlEvent::NodeClose(n) => {
                ident -= 2;
                println!("{}</{}>", " ".repeat(ident), n);
            }
        }
    }
}
