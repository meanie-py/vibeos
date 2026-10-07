"""Independent source inventory and Rust definition accounting, before and after payment.

This is a presence check, not proof of behavior or cfg activation. Foreign declarations,
comments and string contents never provide an implementation. Callers own explicit scope
selection (dead configurations, C trampolines, or a selected C++ class).
"""

import re
from collections.abc import Iterable

import tree_sitter_rust
from tree_sitter import Language, Parser

from .c_defs import c_functions


class CoverageError(ValueError):
    """A source or planned definition disappeared."""


def rust_definition_bodies(text: str) -> dict[str, str]:
    parser = Parser(Language(tree_sitter_rust.language()))
    source = text.encode()
    tree = parser.parse(source)
    definitions: dict[str, str] = {}

    def walk(node):
        if node.type == "function_item":
            name = node.child_by_field_name("name")
            body = node.child_by_field_name("body")
            if name is not None and body is not None:
                definitions[source[name.start_byte : name.end_byte].decode()] = source[
                    node.start_byte : node.end_byte
                ].decode()
        # Macro token trees are deliberately opaque: finding `fn X` inside a macro
        # argument does not prove that the macro expands to an implementation.
        if node.type in ("macro_definition", "macro_invocation"):
            return
        for child in node.named_children:
            walk(child)

    walk(tree.root_node)
    return definitions


def rust_definitions(text: str) -> set[str]:
    return set(rust_definition_bodies(text))


def restore_requested_name(rust: str, name: str) -> str:
    """Respell a definition whose name differs from the requested C name only in CASE.

    The model "corrects" a C name it finds oddly capitalized: identify_funcs.h's
    `AtaDevIsReadLookAHeadEnabled` came back as `AtaDevIsReadLookAheadEnabled`, a correct body
    the stitcher then refused as missing. Case is the only freedom taken: exactly one
    definition must match ignoring case, and none exactly. Every other spelling is still the
    coverage check's to reject.
    """
    defs = rust_definitions(rust)
    if name in defs:
        return rust
    twins = [d for d in defs if d.lower() == name.lower()]
    if len(twins) != 1:
        return rust
    return re.sub(rf"\b{re.escape(twins[0])}\b", name, rust)


def require_c_plan(source: str, planned: Iterable[str]) -> None:
    missing = c_functions(source) - set(planned)
    if missing:
        raise CoverageError(
            "C definitions absent from translation plan: " + ", ".join(sorted(missing))
        )


def require_rust_definitions(expected: Iterable[str], text: str) -> None:
    missing = set(expected) - rust_definitions(text)
    if missing:
        raise CoverageError(
            "Planned Rust definitions absent from output: " + ", ".join(sorted(missing))
        )
