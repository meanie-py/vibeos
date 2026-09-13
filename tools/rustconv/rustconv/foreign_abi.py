"""Move variadic declarations without changing their fixed-arity neighbors' ABI."""

import tree_sitter_rust
from tree_sitter import Language, Parser


def split_variadic_system_blocks(code: str) -> str:
    source = code.encode()
    tree = Parser(Language(tree_sitter_rust.language())).parse(source)
    edits = []

    def walk(node):
        if node.type == "foreign_mod_item":
            modifier = next((c for c in node.named_children if c.type == "extern_modifier"), None)
            body = next((c for c in node.named_children if c.type == "declaration_list"), None)
            if modifier is None or body is None:
                return
            if b'"system"' not in source[modifier.start_byte : modifier.end_byte]:
                return
            moved = []
            for item in body.named_children:
                params = item.child_by_field_name("parameters")
                if item.type != "function_signature_item" or params is None:
                    continue
                if not any(c.type == "variadic_parameter" for c in params.named_children):
                    continue
                start = item.start_byte
                previous = item.prev_named_sibling
                while previous is not None and previous.type in (
                    "attribute_item",
                    "line_comment",
                    "block_comment",
                ):
                    start = previous.start_byte
                    previous = previous.prev_named_sibling
                moved.append((start, item.end_byte))
            if not moved:
                return
            fixed = source[node.start_byte : node.end_byte]
            for start, end in reversed(moved):
                fixed = fixed[: start - node.start_byte] + fixed[end - node.start_byte :]
            # An outer #[cfg]/#[link] applies to BOTH resulting foreign blocks.
            attrs_start = node.start_byte
            previous = node.prev_named_sibling
            while previous is not None and previous.type == "attribute_item":
                attrs_start = previous.start_byte
                previous = previous.prev_named_sibling
            attrs = source[attrs_start : node.start_byte]
            unsafe = b"unsafe " if source[node.start_byte : modifier.start_byte].strip() else b""
            foreign = b"\n".join(source[a:b] for a, b in moved)
            replacement = fixed + b"\n" + attrs + unsafe + b'extern "C" {\n' + foreign + b"\n}"
            edits.append((node.start_byte, node.end_byte, replacement))
            return
        if node.type not in ("macro_definition", "macro_invocation"):
            for child in node.named_children:
                walk(child)

    walk(tree.root_node)
    for start, end, replacement in sorted(edits, reverse=True):
        source = source[:start] + replacement + source[end:]
    return source.decode()
