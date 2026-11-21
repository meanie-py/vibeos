"""C identifiers that are Rust's RESERVED (unused) keywords, made raw identifiers.

Wine names a pointer `priv` everywhere (`struct wined3d_sm4_data *priv`,
`struct shader_glsl_priv *priv`); `box`, `final`, `override`, `virtual` and `macro` are
ordinary C names too. Rust reserves them without giving them any meaning, so a body the
model writes with `priv: *mut wined3d_sm4_data` does not parse, and translate rejected nine
of wined3d's bodies for it. A bare use of one of these tokens can only be the C identifier,
so `r#priv` is exact -- unlike a strict keyword (`type`, `match`, `ref`), which the model
already renames because it has a Rust meaning.

Field access (`.priv`) is left alone: bindgen spells a keyword FIELD `priv_`, and
`postprocess.fix_keyword_field_names` maps every access onto that. Strings, chars and
comments are skipped.
"""

from __future__ import annotations

import re

#: Rust 2021's reserved-for-future-use keywords. `try` is omitted: it is reserved too, but
#: the C almost never uses it as a name and `r#try` would misread a stray `try!` macro.
RESERVED = (
    "abstract",
    "become",
    "box",
    "do",
    "final",
    "macro",
    "override",
    "priv",
    "typeof",
    "unsized",
    "virtual",
    "yield",
)

_TOKEN = re.compile(
    r"//[^\n]*"  # line comment
    r"|/\*.*?\*/"  # block comment
    r'|b?"(?:[^"\\]|\\.)*"'  # string / byte string
    r"|b?'(?:[^'\\\n]|\\.){1,10}'"  # char / byte char (a lifetime has no closing quote)
    r"|(?P<pre>r#|[.#'])?\b(?P<kw>" + "|".join(RESERVED) + r")\b(?!!)",
    re.S,
)


def raw_reserved_keywords(rust: str) -> str:
    """`priv` -> `r#priv` wherever a reserved keyword stands as an identifier."""

    def sub(m: re.Match[str]) -> str:
        if m.group("kw") is None or m.group("pre"):
            return m.group(0)
        return "r#" + m.group("kw")

    return _TOKEN.sub(sub, rust)
