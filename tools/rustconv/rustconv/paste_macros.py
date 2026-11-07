"""Call sites of a module's MEMBER-PASTE macro, expanded as the C preprocessor would.

wined3d_gl.h: `#define GL_EXTCALL(f) (gl_info->gl_ops.ext.p_##f)`, used as
`GL_EXTCALL(glActiveTexture(GL_TEXTURE0 + i))` at 610 sites. The model keeps the macro call,
and no `macro_rules!` can stand in for it: `p_##f` pastes an identifier (nothing in std does
that) and the body names the CALLER's `gl_info`, which macro hygiene hides. resolve then
reported every GL entry point as a missing import. So each call site is expanded here, from
the macro's own definition:

    GL_EXTCALL(glFoo(a, b))  ->  ((*gl_info).gl_ops.ext.p_glFoo.unwrap_unchecked())(a, b)
    GL_EXTCALL(glFoo)        ->  (*gl_info).gl_ops.ext.p_glFoo

The field is bindgen's `Option<fn>`; calling it is `unwrap_unchecked`, as crate-repair's
`option_fn_call` does -- calling a NULL entry point is undefined in the C too.

Only a one-parameter macro whose whole body is a member chain pasted with its parameter
(`base->a.b.prefix_##f`, optionally parenthesised) qualifies; anything else is left alone.
"""

from __future__ import annotations

import re

from .local_macros import CMacro

_CHAIN = re.compile(r"\s*(->|\.)\s*([A-Za-z_]\w*)")


def member_paste_macros(defs: dict[str, CMacro]) -> dict[str, str]:
    """{macro name: the Rust field-path prefix its pasted argument completes}."""
    out: dict[str, str] = {}
    for name, m in defs.items():
        if m.kind != "macro" or m.params is None or len(m.params) != 1:
            continue
        body = m.body.strip()
        while body.startswith("(") and body.endswith(")"):
            body = body[1:-1].strip()
        p = re.escape(m.params[0])
        hit = re.fullmatch(rf"([A-Za-z_]\w*)((?:\s*(?:->|\.)\s*[A-Za-z_]\w*)+)\s*##\s*{p}", body)
        if hit is None:
            continue
        expr = hit.group(1)
        for op, seg in _CHAIN.findall(hit.group(2)):
            expr = f"(*{expr}).{seg}" if op == "->" else f"{expr}.{seg}"
        out[name] = expr
    return out


def _close(text: str, open_at: int) -> int | None:
    """Index of the `)` matching the `(` at `open_at` (strings and chars skipped)."""
    depth, i = 0, open_at
    while i < len(text):
        c = text[i]
        if c == '"':
            i += 1
            while i < len(text) and text[i] != '"':
                i += 2 if text[i] == "\\" else 1
        elif c == "(":
            depth += 1
        elif c == ")":
            depth -= 1
            if depth == 0:
                return i
        i += 1
    return None


def expand_member_paste_calls(rust: str, macros: dict[str, str]) -> tuple[str, int, list[str]]:
    """(rewritten text, sites expanded, sites left alone because their argument has no shape)."""
    if not macros:
        return rust, 0, []
    use = re.compile(r"\b(" + "|".join(map(re.escape, sorted(macros))) + r")!?\s*\(")
    out: list[str] = []
    pos, count, odd = 0, 0, []
    while (m := use.search(rust, pos)) is not None:
        open_at = m.end() - 1
        close = _close(rust, open_at)
        if close is None:
            break
        arg = rust[open_at + 1 : close].strip()
        field = macros[m.group(1)]
        call = re.match(r"([A-Za-z_]\w*)\s*\(", arg)
        if call is not None and _close(arg, call.end() - 1) == len(arg) - 1:
            args = arg[call.end() : -1]
            new = f"({field}{call.group(1)}.unwrap_unchecked())({args})"
        elif re.fullmatch(r"[A-Za-z_]\w*", arg):
            new = f"{field}{arg}"
        else:
            odd.append(rust[m.start() : close + 1][:80])
            out.append(rust[pos : close + 1])
            pos = close + 1
            continue
        out.append(rust[pos : m.start()] + new)
        pos = close + 1
        count += 1
    out.append(rust[pos:])
    return "".join(out), count, odd
