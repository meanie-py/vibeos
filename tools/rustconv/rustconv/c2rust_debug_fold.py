"""Fold c2rust's expansion of wine/debug.h's TRACE/WARN/FIXME/ERR back into the macro.

c2rust sees the preprocessed TU, so every `WARN("header too small\\n")` arrives as ~18 lines:
a test of the channel's flag bits, two locals, and a `ros_dbg_log(...)` call carrying the
builder's absolute source path and line. A c2rust body banked by hand (one too large for the
model's output budget -- winhlp32's HLPFILE_BrowseParagraph, 1,569 lines, 26 such blocks)
should read like the rest of the crate, which calls win32macros-rust's `WINE_WARN!(..)`:

    if (*__wine_dbch___default).flags as c_int & (1 as c_int) << __WINE_DBCL_WARN as c_int != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_WARN;
        ros_dbg_log(__dbcl, __dbch, b"/abs/path.c\\0" .., b"Func\\0" .., 1236 as c_int,
                    b"header too small\\n\\0" as *const u8 as *const c_char, args..);
    }
  ->
    WINE_WARN!("header too small\\n", args..);

Only the exact shape is folded; anything else is left alone. The default channel only: a
named channel's block is left as c2rust wrote it.
"""

from __future__ import annotations

import re

# Written with single spaces; each space then matches any run of whitespace, since rustfmt
# breaks c2rust's long lines wherever the width runs out (`let __dbcl_7: __wine_debug_class =`
# with its value on the next line).
#
# A body the MODEL wrote from the anchor keeps the block but respells it: extra parentheses,
# `core::ffi` without the leading `::`, `addr_of!(__wine_dbch___default)`, `(__dbcl) as u32`
# (26 of dsound's 77 blocks). So the condition is taken as text and checked for its parts
# in `_cond_ok`, and only the spellings below are tolerated in the rest -- the call's shape
# (class, channel, file, function, line, literal format) is still required exactly.
_C = r"(?:::)?core::ffi::c_\w+"
_CSTR = rf"b\"[^\"]*\\0\" as \*const u8 as \*const {_C}"
_TEMPLATE = (
    r"\bif (?P<cond>[^{};]*?) \{ "
    # c2rust numbers the locals of the second and later blocks (`__dbch_0`, `__dbcl_1`).
    r"let (?P<ch>__dbch(?:_\d+)?): \*mut __wine_debug_channel = [^;]*?__wine_dbch___default[^;]*?; "
    r"let (?P<cl>__dbcl(?:_\d+)?): __wine_debug_class = __WINE_DBCL_(?P<cls>TRACE|WARN|FIXME|ERR); "
    r"ros_dbg_log\( \(?(?P=cl)\)?(?: as \w+)?, (?P=ch), "
    rf"{_CSTR}, "
    rf"{_CSTR}, "
    rf"\d+ as {_C}, "
    rf"b\"(?P<fmt>(?:\\.|[^\"\\])*)\\0\" as \*const u8 as \*const {_C},?"
    r"(?P<args>[^;]*?)\); \}"
)
_BLOCK = re.compile(_TEMPLATE.replace(" ", r"\s*"), re.S)


def _cond_ok(cond: str, cls: str) -> bool:
    """The `if` really is `channel.flags & (1 << class) != 0` for THIS block's class."""
    flat = re.sub(r"\s+", "", cond)
    return (
        "__wine_dbch___default" in flat
        and ").flags" in flat
        and f"<<(__WINE_DBCL_{cls}" in flat.replace("<<__WINE", "<<(__WINE")
        and flat.endswith("!=0")
        and "&&" not in flat
        and "||" not in flat
    )


def fold(text: str) -> tuple[str, int]:
    """(new text, number of blocks folded)."""

    count = 0

    def one(m: re.Match) -> str:
        nonlocal count
        if not _cond_ok(m.group("cond"), m.group("cls")):
            return m.group(0)
        count += 1
        args = m.group("args").strip().rstrip(",").strip()
        args = re.sub(r"\s*\n\s*", " ", args)
        tail = f", {args}" if args else ""
        return f'WINE_{m.group("cls")}!("{m.group("fmt")}"{tail});'

    return _BLOCK.sub(one, text), count


# The scaffolding a translated module declares for the expanded blocks: the enum value as a
# local const and the logger as an extern. Once the blocks are folded nothing names them, and
# the const is worse than dead: dsound's mixer.rs came back with `__WINE_DBCL_TRACE = 4`
# (wine/debug.h says 3), which tested the wrong channel bit in every block it guarded.
_LOCAL_CLASS_CONST = re.compile(
    r"(?m)^[ \t]*(?:pub(?:\([^)]*\))?\s+)?const\s+(?P<name>__WINE_DBCL_\w+)"
    r"\s*:[^=;]*=[^;]*;[ \t]*\n"
)
_LOGGER_DECL = re.compile(
    r"(?m)^[ \t]*(?:pub\s+)?fn\s+ros_dbg_log\s*\((?:[^;]*)\)\s*(?:->[^;]*)?;[ \t]*\n"
)
_EMPTY_EXTERN = re.compile(r'(?m)^[ \t]*(?:unsafe\s+)?extern\s+"C"\s*\{\s*\}[ \t]*\n')


def _named(text: str, name: str) -> bool:
    return re.search(rf"\b{re.escape(name)}\b", text) is not None


def fold_module(text: str) -> tuple[str, int, list[str]]:
    """Fold every expanded block, then drop the scaffolding nothing references any more.

    Returns (new text, blocks folded, scaffolding names dropped). A local const or the logger
    declaration is kept while anything still names it -- a block outside the exact shape
    `fold` handles (a named channel) still needs both."""
    new, n = fold(text)
    dropped: list[str] = []
    for m in list(_LOCAL_CLASS_CONST.finditer(new)):
        rest = new[: m.start()] + new[m.end() :]
        if not _named(rest, m.group("name")):
            new = rest
            dropped.append(m.group("name"))
    for m in list(_LOGGER_DECL.finditer(new)):
        rest = new[: m.start()] + new[m.end() :]
        if not _named(rest, "ros_dbg_log"):
            new = _EMPTY_EXTERN.sub("", rest)
            dropped.append("ros_dbg_log")
    return new, n, dropped
