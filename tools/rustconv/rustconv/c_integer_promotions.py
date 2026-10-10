"""Locate comparisons and apply the bounded C int-promotion subset (8/16/32 bits)."""

import tree_sitter_rust
from tree_sitter import Language, Parser


def comparison_operands(data: bytes, start: int, end: int):
    tree = Parser(Language(tree_sitter_rust.language())).parse(data)
    node = tree.root_node.descendant_for_byte_range(start, max(start, end - 1))
    while node is not None:
        # A call's argument list (or a block) bounds the operand: `GetKeyState(VK_SHIFT) < 0`
        # compares the CALL, and VK_SHIFT's mismatch is with GetKeyState's parameter.
        if node.type in ("arguments", "block", "index_expression"):
            return None
        if node.type == "binary_expression":
            op = node.child_by_field_name("operator")
            if op is not None and data[op.start_byte : op.end_byte] in (
                b"==",
                b"!=",
                b"<",
                b">",
                b"<=",
                b">=",
            ):
                left = node.child_by_field_name("left")
                right = node.child_by_field_name("right")
                if left is not None and right is not None:
                    return (left.start_byte, left.end_byte), (right.start_byte, right.end_byte)
        node = node.parent
    return None


def promoted_type(left: str, right: str) -> str | None:
    supported = {"u8", "i8", "u16", "i16", "u32", "i32"}
    if left not in supported or right not in supported:
        return None
    return "u32" if "u32" in (left, right) else "i32"
