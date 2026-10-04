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
_FOUR_CHAR_TAG = re.compile(r"'[^'\\\n,:;>]{4}'")


def opens_a_literal(text: str, index: int) -> bool:
    """False for a Rust label/lifetime (`'cleanup:`, `'static`, `&'a str`).

    Shared so every masker in the tree applies the same rule; `fact_seal` keeps its
    own blanking semantics (it also blanks the quotes and any `L`/`u`/`U` prefix)
    but must not disagree with this one about what a quote even opens.
    """
    match = _LABEL_OR_LIFETIME.match(text, index)
    if match is None:
        return True  # `'x'`, `'\n'`, `'\''` — an escape or a bare quote
    if text[match.end() : match.end() + 1] == "'":
        return True
    # A four-character pool tag that leaves the identifier alphabet: freeldr heap.c's
    # `'dnE#'`, `'SFI '`. Read as a label, the quote stayed open and the masker swallowed
    # the next 460 lines. Exactly four and no `,:;>`, so `<'ab, 'c>` stays two lifetimes.
    return _FOUR_CHAR_TAG.match(text, index) is not None


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


def directive_spans(text: str) -> list[bool]:
    """Per-character "is inside a preprocessor directive" flags.

    Nothing inside a directive can be sealed. A `#define`/`#undef` NAME is not a value and
    not a callee, and the halves of a `##` paste are not identifiers at all — sealing any of
    them produces an envelope no correct translation can satisfy, because the token has no
    place to go in the Rust output. shell32's `changenotify.cpp` hit all three at once: its
    `DumpEvent` defines `DUMPEV(x)` in the function body, pastes `SHCNE_##x`, and `#undef`s it
    at the end, which sealed as `#define __RUSTCONV_CALL_0000__(x)`, `__RUSTCONV_FACT_0001__##x`
    and `#undef __RUSTCONV_FACT_0003__`. Four paid retries, all rejected, none satisfiable.

    A directive runs to end of line and a trailing backslash continues it onto the next, which
    is what keeps a multi-line `#define` body covered. The `#` is located in MASKED text so a
    `#` inside a string or comment never starts one.
    """
    masked = mask_code(text)
    inside = [False] * len(text)
    length = len(text)
    pos = 0
    while pos < length:
        end = masked.find("\n", pos)
        end = length if end < 0 else end
        if masked[pos:end].lstrip(" \t").startswith("#"):
            while True:
                for i in range(pos, end):
                    inside[i] = True
                line = masked[pos:end]
                if end >= length:
                    pos = length
                    break
                pos = end + 1
                if not line.rstrip().endswith("\\"):
                    break
                end = masked.find("\n", pos)
                end = length if end < 0 else end
        else:
            pos = end + 1
    return inside


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
