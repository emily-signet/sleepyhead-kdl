use sleepyhead_kdl::parser::Parser;
use sleepyhead_kdl::KdlEvent;

fn main() {
    let mut ident = 0;
    let mut parser = Parser::from_str(include_str!("./schema.kdl"));
    while let Some(node) = parser.next() {
        let node = node.unwrap();

        match node {
            KdlEvent::NodeOpen {
                name,
                attrs,
                values,
                has_children,
            } => {
                print!(
                    "{}<{} {} {}>",
                    " ".repeat(ident),
                    name,
                    attrs
                        .into_iter()
                        .map(|p| format!("{}={}", p.key, p.value))
                        .collect::<Vec<String>>()
                        .join(" "),
                    values
                        .into_iter()
                        .map(|v| format!("{v}"))
                        .collect::<Vec<String>>()
                        .join(" ")
                );

                if has_children {
                    ident += 2;
                    print!("\n");
                } else {
                    use std::io::Write;
                    std::io::stdout().flush();
                }
            }
            KdlEvent::BracketedNodeClose(n) => {
                ident -= 2;
                println!("{}</{}>", " ".repeat(ident), n);
            }
            KdlEvent::NodeClose(n) => {
                println!("</{}>", n);
            }
        }
    }
}