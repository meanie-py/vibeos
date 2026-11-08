"""Keep the file-scope DATA c2rust already supplies out of the paid preamble prompt.

c2rust's file-scope statics and tables are compiler-exact, and `translate` prepends them and
drops any same-named item from the model's preamble. Sending those definitions to the model
anyway paid for a translation that was always discarded, and on a table-heavy TU it does
worse: wined3d directx.c's preamble is 52,093 C chars, 51K of them the eleven tables c2rust
emits (gpu_description_table alone is 42K), and the reply truncated at 32,768 output tokens,
so the TU shipped no body at all. Each such definition is replaced by a one-line note naming
it, so the model still knows the name exists.

Only the PROMPT changes: the preamble's resume-cache key stays the full file-scope text, so
no preamble already banked anywhere in the tree is re-keyed.
"""

from __future__ import annotations

import re


def _definition_span(src: str, name: str) -> tuple[int, int] | None:
    """[start, end) of the top-level `... NAME[..] = ...;` definition, or None."""
    m = re.search(rf"(?m)^[^\n;{{}}#]*\b{re.escape(name)}\s*(?:\[[^\]]*\]\s*)*=", src)
    if m is None:
        return None
    depth, i = 0, m.end()
    while i < len(src):
        ch = src[i]
        if ch == '"' or ch == "'":
            q, i = ch, i + 1
            while i < len(src) and src[i] != q:
                i += 2 if src[i] == "\\" else 1
        elif ch in "{([":
            depth += 1
        elif ch in "})]":
            depth -= 1
        elif ch == ";" and depth == 0:
            return m.start(), i + 1
        i += 1
    return None


def omit_c2rust_data(src: str, names: set[str]) -> tuple[str, list[str]]:
    """`src` with each c2rust-supplied definition replaced by a note; the names omitted."""
    omitted: list[str] = []
    for name in sorted(names):
        span = _definition_span(src, name)
        if span is None:
            continue
        start, end = span
        note = f"/* {name}: defined exactly by c2rust's file-scope data -- do not emit it */"
        src = src[:start] + note + src[end:]
        omitted.append(name)
    return src, omitted
