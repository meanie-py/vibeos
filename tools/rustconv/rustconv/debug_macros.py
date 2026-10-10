"""Preserve source diagnostics without deleting statements or introducing Rust panics."""

import tree_sitter_rust
from tree_sitter import Language, Parser

_NAMES = {"DPRINT", "DPRINT1", "ERR", "WARN", "TRACE", "FIXME", "MESSAGE"}


def preserve_debug_macros(code: str) -> str:
    data = code.encode()
    tree = Parser(Language(tree_sitter_rust.language())).parse(data)
    edits = []

    def walk(node):
        if node.type in ("macro_definition", "macro_invocation"):
            return
        if node.type == "call_expression":
            callee = node.child_by_field_name("function")
            if callee is not None and callee.type == "identifier":
                name = data[callee.start_byte : callee.end_byte].decode()
                if name in _NAMES:
                    edits.append((callee.end_byte, b"!"))
        elif node.type == "expression_statement" and node.named_children:
            child = node.named_children[0]
            # Bare statement macros: `UNIMPLEMENTED;`, and `UNREACHABLE;` (ntbasedef.h's
            # `__builtin_unreachable()`), which the new ATA stack uses after exhaustive switches.
            if child.type == "identifier" and data[child.start_byte : child.end_byte] in (
                b"UNIMPLEMENTED",
                b"UNIMPLEMENTED_ONCE",  # user32_vista's dpi.c
                b"UNREACHABLE",
            ):
                edits.append((child.end_byte, b"!()"))
        for child in node.named_children:
            walk(child)

    walk(tree.root_node)
    for position, addition in sorted(edits, reverse=True):
        data = data[:position] + addition + data[position:]
    return data.decode()
