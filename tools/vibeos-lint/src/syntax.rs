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
