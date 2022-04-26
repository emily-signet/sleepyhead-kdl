use sleepyhead_kdl::assembler::*;
use sleepyhead_kdl::parser::Parser;

fn main() {
    fn print_node(ident: usize, node: KdlNode<'_>) {
        print!("{}<{}", " ".repeat(ident), node.name);

        for attr in node.attrs {
            print!(" {}={}", attr.key, attr.value);
        }

        for value in node.values {
            print!(" {}", value);
        }

        print!(">");

        let has_children = !node.children.is_empty();

        if has_children {
            println!("");
        } else {
            println!("</{}>", node.name);
        }

        for child in node.children {
            print_node(ident + 2, child);
        }

        if has_children {
            println!("{}</{}>", " ".repeat(ident), node.name);
        }
    }

    let mut ident = 0;
    let mut parser = Parser::from_str(include_str!("./schema.kdl"));
    for root_node in parse_document(&mut parser).unwrap() {
        print_node(0, root_node);
    }
}
