//! The same error-tolerant Rust grammar used by the historical Python scanners.
use tree_sitter::{Parser, Tree};

pub fn parse(text: &str) -> Result<Tree, String> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .map_err(|e| format!("Rust grammar: {e}"))?;
    parser
        .parse(text, None)
        .ok_or_else(|| "Rust syntax parser cancelled".to_owned())
}

/// Visit a subtree in preorder with one cursor, including unnamed nodes.
/// The cursor is rooted at `root`, so traversal cannot escape to its siblings.
pub fn preorder<'tree>(
    root: tree_sitter::Node<'tree>,
) -> impl Iterator<Item = tree_sitter::Node<'tree>> {
    let mut cursor = root.walk();
    let mut first = true;
    let mut done = false;
    std::iter::from_fn(move || {
        if done {
            return None;
        }
        if first {
            first = false;
            return Some(cursor.node());
        }
        if !cursor.goto_first_child() {
            while !cursor.goto_next_sibling() {
                if !cursor.goto_parent() {
                    done = true;
                    return None;
                }
            }
        }
        Some(cursor.node())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preorder_keeps_unnamed_nodes_and_stays_inside_its_root() {
        let tree = parse("fn first() { let x = 1; } fn second() {}").unwrap();
        let root = tree.root_node().named_child(0).unwrap();
        fn reference<'a>(node: tree_sitter::Node<'a>, out: &mut Vec<usize>) {
            out.push(node.id());
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                reference(child, out);
            }
        }
        let mut expected = Vec::new();
        reference(root, &mut expected);
        assert_eq!(preorder(root).map(|n| n.id()).collect::<Vec<_>>(), expected);
        assert!(preorder(root).any(|n| n.kind() == "("));
        assert!(!preorder(root)
            .any(|n| &"fn first() { let x = 1; } fn second() {}"[n.byte_range()] == "second"));
    }
}
