use super::{Element, Node};

pub struct AnalyzeError {
    message: String,
    line: usize,
    col: usize,
}

fn analyze_node(node: &Node, buf: &mut String, depth: &mut usize) {
    let spaces = (0..*depth).fold(String::new(), |mut s, _| {
        s.push_str("    ");
        s
    });

    match node {
        Node::Element(Element {
            kind,
            attributes,
            children,
        }) => {
            let attrs = attributes.iter().fold(String::new(), |mut s, a| {
                s.push_str(&format!("{}: {:?} ", a.name, a.value));
                s
            });

            buf.push_str(&format!("{spaces}{kind:?}"));

            if !attrs.is_empty() {
                buf.push_str(&format!(" | {attrs}"));
            }

            buf.push('\n');

            *depth += 1;

            for child in children {
                analyze_node(child, buf, depth);
            }

            *depth -= 1;
        }
        Node::Text(text) => buf.push_str(&format!("{spaces}{text}\n")),
    }
}

pub fn analyze_ast() {}
