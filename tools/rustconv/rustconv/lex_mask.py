"""Offset-preserving comment/literal mask, for C++ source AND Rust output.

Three byte-identical copies of this lived in `call_seal`, `fact_seal` and
`cpp_com`, and every one of them was a C lexer — so every one of them treated a
Rust LABEL or LIFETIME as an opening char literal and blanked the source from
there to the next apostrophe:

    'cleanup: loop {          // <- masker opens a "char literal" here
        ...                   //    everything here becomes invisible
    }                         //
    if x { helper() }         //
    'cleanup: loop {          // <- ... and closes it here

That silently voided a faithful translation of `SHFileOperationA`: the C body has
three `goto cleanup`, the model lowered them to three `'cleanup: loop` blocks, and
two call tokens fell inside the blanked regions and were reported "missing". The
model had written them. We could not see them.

The discriminator is exact in both dialects: `'` + identifier + `'` is a character
constant (C multi-character constants like `'MPEG'` included); `'` + identifier
followed by anything else is a Rust label or lifetime and opens nothing. Valid C
cannot produce the second form, so one masker serves both.

Quotes themselves survive; only literal CONTENT and comment bodies are blanked, and
newlines are always kept so every offset and line number is preserved.
"""

from __future__ import annotations

import re

_LABEL_OR_LIFETIME = re.compile(r"'[A-Za-z_]\w*")


def opens_a_literal(text: str, index: int) -> bool:
    """False for a Rust label/lifetime (`'cleanup:`, `'static`, `&'a str`).

    Shared so every masker in the tree applies the same rule; `fact_seal` keeps its
    own blanking semantics (it also blanks the quotes and any `L`/`u`/`U` prefix)
    but must not disagree with this one about what a quote even opens.
    """
    match = _LABEL_OR_LIFETIME.match(text, index)
    if match is None:
        return True  # `'x'`, `'\n'`, `'\''` — an escape or a bare quote
    return text[match.end() : match.end() + 1] == "'"


_opens_a_literal = opens_a_literal


def mask_code(text: str) -> str:
    """Blank comments and literal contents, preserving every byte offset."""
    out = list(text)
    i = 0
    while i < len(text):
        if text.startswith("//", i):
            end = text.find("\n", i + 2)
            end = len(text) if end < 0 else end
            for pos in range(i, end):
                out[pos] = " "
            i = end
        elif text.startswith("/*", i):
            end = text.find("*/", i + 2)
            end = len(text) if end < 0 else end + 2
            for pos in range(i, end):
                if out[pos] != "\n":
                    out[pos] = " "
            i = end
        elif text[i] == "'" and not _opens_a_literal(text, i):
            i += 1  # a label/lifetime tick opens nothing
        elif text[i] in ('"', "'"):
            quote = text[i]
            i += 1
            while i < len(text):
                if text[i] == "\\":
                    out[i] = " "
                    if i + 1 < len(text):
                        out[i + 1] = " "
                    i += 2
                    continue
                if text[i] == quote:
                    i += 1
                    break
                if out[i] != "\n":
                    out[i] = " "
                i += 1
        else:
            i += 1
    return "".join(out)


def comment_spans(text: str) -> list[bool]:
    """Per-character "is inside a comment" flags, with the same literal rules."""
    inside = [False] * len(text)
    i = 0
    while i < len(text):
        if text.startswith("//", i):
            end = text.find("\n", i + 2)
            end = len(text) if end < 0 else end
            for pos in range(i, end):
                inside[pos] = True
            i = end
        elif text.startswith("/*", i):
            end = text.find("*/", i + 2)
            end = len(text) if end < 0 else end + 2
            for pos in range(i, end):
                inside[pos] = True
            i = end
        elif text[i] == "'" and not _opens_a_literal(text, i):
            i += 1
        elif text[i] in ('"', "'"):
            quote = text[i]
            i += 1
            while i < len(text):
                if text[i] == "\\":
                    i += 2
                    continue
                i += 1
                if text[i - 1] == quote:
                    break
        else:
            i += 1
    return inside
