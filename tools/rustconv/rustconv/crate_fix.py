"""Compiler-in-the-loop repair: each function rustc rejects goes back to the model, errors attached.

WHY THIS EXISTS
---------------
The per-function translation is bought BLIND -- the model never sees a compiler -- and
`crate-repair` then fixes what comes back one hand-written rule per error class. On setuplib
(21K lines of C, 2026-10-05) that plateaued at ~300 errors per arch: the last four repair
passes moved the count by 11, and every class left needed a new rule written, tested and
landed before it moved again. An agent with `cargo check` in its loop does not have that
problem, and neither does this: the residue goes back to the model per function, errors
attached, at thinking-off prices (~$0.001 a call).

What keeps it honest is the same discipline as translate:
  * every reply is banked under a key of (function text, its errors, prompt version), so a
    regenerated crate replays its fixes for $0 and a paid reply is never lost;
  * a reply is ACCEPTED only past deterministic guards -- it parses, keeps the signature,
    keeps every sealed constant and every call the C made, adds no item outside the function,
    and uses none of the panicking conversions crate-repair refuses by design;
  * a fix that leaves its function with MORE errors on any arch is reverted (the bank keeps
    it), and a round that raises the crate's halting (parse/expansion) errors is reverted
    whole -- those hide every other error, so a falling total would be a lie.

The deterministic rules stay where a model's fix is unsafe or the class is huge: pointer
widths (a model bakes one arch's width), `.try_into().unwrap()`, truthiness. This stage is
for the long tail after them.
"""

from __future__ import annotations

import hashlib
import json
import re
from collections import Counter
from dataclasses import dataclass, field
from pathlib import Path

# The bank key's salt. NOT a prompt version: replies are re-judged by the guards on every
# replay, so a reply bought under older prompt prose is kept exactly when it still passes.
# Changing this re-buys the whole bank.
_KEY_SALT = "v1"

# --------------------------------------------------------------------------------------
# locating top-level functions in a Rust file
# --------------------------------------------------------------------------------------

_ITEM_FN = re.compile(
    r"(?m)^(?:pub(?:\([^)]*\))?\s+)?(?:const\s+)?(?:unsafe\s+)?"
    r"(?:extern\s+\"[^\"]+\"\s+)?fn\s+([A-Za-z_]\w*)"
)
_ATTR_OR_DOC = re.compile(r"^(?:#\[|///|//!)")


@dataclass(frozen=True)
class FnSpan:
    name: str
    start: int  # byte offset of the first attribute / doc line
    end: int  # byte offset one past the closing brace


def _skip_literal(text: str, i: int) -> int:
    """`text[i]` opens a comment, string or char literal: the offset just past it, else i."""
    n = len(text)
    two = text[i : i + 2]
    if two == "//":
        j = text.find("\n", i)
        return n if j < 0 else j
    if two == "/*":
        depth, j = 1, i + 2
        while j < n and depth:
            if text.startswith("/*", j):
                depth, j = depth + 1, j + 2
            elif text.startswith("*/", j):
                depth, j = depth - 1, j + 2
            else:
                j += 1
        return j
    m = re.compile(r'b?r(#*)"').match(text, i)
    if m and (i == 0 or not (text[i - 1].isalnum() or text[i - 1] == "_")):
        close = '"' + m.group(1)
        j = text.find(close, m.end())
        return n if j < 0 else j + len(close)
    if text[i] == '"' or two == 'b"':
        j = i + (2 if two == 'b"' else 1)
        while j < n and text[j] != '"':
            j += 2 if text[j] == "\\" else 1
        return j + 1
    if text[i] == "'" or two == "b'":
        j = i + (2 if two == "b'" else 1)
        # A char literal closes within a few chars; a lifetime / loop label does not.
        m = re.compile(r"(?:\\(?:x[0-9a-fA-F]{2}|u\{[0-9a-fA-F]+\}|.)|[^\\'\n])'").match(text, j)
        return m.end() if m else i
    return i


def _matching_brace(text: str, open_idx: int) -> int | None:
    depth, i, n = 0, open_idx, len(text)
    while i < n:
        j = _skip_literal(text, i)
        if j != i:
            i = j
            continue
        c = text[i]
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return i
        i += 1
    return None


def _body_open(text: str, start: int) -> int | None:
    """The `{` that opens the body of the fn whose name ends at `start`; None for a `;` decl."""
    depth, i, n = 0, start, len(text)
    while i < n:
        j = _skip_literal(text, i)
        if j != i:
            i = j
            continue
        c = text[i]
        if c in "([<":
            depth += 1
        elif c in ")]>" and not (c == ">" and text[i - 1] == "-"):
            depth -= 1
        elif c == ";" and depth <= 0:
            return None
        elif c == "{" and depth <= 0:
            return i
        i += 1
    return None


def function_spans(text: str) -> list[FnSpan]:
    """Every top-level (column-0) function definition, attributes included."""
    out: list[FnSpan] = []
    for m in _ITEM_FN.finditer(text):
        if out and m.start() < out[-1].end:
            continue  # inside the previous item (only column-0 matches, but be safe)
        open_idx = _body_open(text, m.end())
        if open_idx is None:
            continue
        close = _matching_brace(text, open_idx)
        if close is None:
            continue
        start = m.start()
        while start > 0:
            prev_end = start - 1
            prev_start = text.rfind("\n", 0, prev_end) + 1
            line = text[prev_start:prev_end]
            if _ATTR_OR_DOC.match(line):
                start = prev_start
            else:
                break
        out.append(FnSpan(m.group(1), start, close + 1))
    return out


def span_at(spans: list[FnSpan], offset: int) -> FnSpan | None:
    for s in spans:
        if s.start <= offset < s.end:
            return s
    return None


# --------------------------------------------------------------------------------------
# errors -> functions
# --------------------------------------------------------------------------------------


@dataclass
class FnErrors:
    file: str  # path relative to the workspace root, as rustc reports it
    span: FnSpan
    # rendered diagnostic -> the arches that reported it
    errors: dict[str, set[str]] = field(default_factory=dict)

    def count(self, arch: str) -> int:
        return sum(1 for arches in self.errors.values() if arch in arches)


def _strip_location(rendered: str) -> str:
    """Rendered diagnostic without the `--> file:line:col` noise that differs per arch only
    by nothing -- kept for the model, but used as-is for dedupe since it is identical."""
    return rendered.rstrip()


def group_by_function(
    diagnostics_by_arch: dict[str, list[dict]],
    root: Path,
    *,
    is_error,
) -> tuple[dict[tuple[str, str], FnErrors], dict[str, int]]:
    """Map every error to the top-level function enclosing its primary span.

    Returns (per-function errors, per-arch count of errors OUTSIDE any function -- prelude
    items, statics, imports -- which this stage does not touch).
    """
    spans_cache: dict[str, list[FnSpan]] = {}
    out: dict[tuple[str, str], FnErrors] = {}
    outside: dict[str, int] = {}
    for arch, diags in diagnostics_by_arch.items():
        outside.setdefault(arch, 0)
        for d in diags:
            if not is_error(d):
                continue
            primary = next((s for s in d.get("spans") or [] if s.get("is_primary")), None)
            if primary is None:
                outside[arch] += 1
                continue
            primary = _in_crate_call_site(primary, root)
            fname = primary["file_name"]
            if fname not in spans_cache:
                p = root / fname
                spans_cache[fname] = (
                    function_spans(p.read_bytes().decode("utf-8", "replace")) if p.is_file() else []
                )
            # rustc's byte offsets are into the UTF-8 file; our spans index the decoded str.
            # The crate is ASCII apart from comments, so map via the line instead.
            text_path = root / fname
            offset = _offset_of_line(text_path, primary["line_start"], primary["column_start"])
            sp = span_at(spans_cache[fname], offset) if offset is not None else None
            if sp is None:
                outside[arch] += 1
                continue
            key = (fname, sp.name)
            fe = out.setdefault(key, FnErrors(fname, sp))
            rendered = _strip_location(d.get("rendered") or d.get("message") or "")
            fe.errors.setdefault(rendered, set()).add(arch)
    return out, outside


def _in_crate_call_site(span: dict, root: Path) -> dict:
    """A span inside a MACRO body (a shared crate's, or the prelude's) is a misuse at the
    call site: walk the expansion chain out to the outermost call, so the function that
    called the macro gets the error. Without this, ~25 of setuplib's errors sat "outside any
    function" -- win32macros-rust lib.rs and prelude.rs -- where `port fix` never looked."""
    best = span
    cur = span
    while isinstance(cur.get("expansion"), dict) and cur["expansion"].get("span"):
        cur = cur["expansion"]["span"]
        best = cur
    # Only adopt the call site if it is a real file in this workspace.
    return best if (root / best.get("file_name", "")).is_file() else span


_LINE_STARTS: dict[Path, list[int]] = {}


def _offset_of_line(path: Path, line: int, col: int) -> int | None:
    if path not in _LINE_STARTS:
        try:
            text = path.read_bytes().decode("utf-8", "replace")
        except OSError:
            return None
        starts = [0]
        for i, c in enumerate(text):
            if c == "\n":
                starts.append(i + 1)
        _LINE_STARTS[path] = starts
    starts = _LINE_STARTS[path]
    if not 1 <= line <= len(starts):
        return None
    return starts[line - 1] + max(0, col - 1)


def forget_file_cache() -> None:
    _LINE_STARTS.clear()


# --------------------------------------------------------------------------------------
# prompt
# --------------------------------------------------------------------------------------

SYSTEM = """\
You fix rustc compile errors in Rust that was translated function-by-function from ReactOS C.
The crate is #![no_std], builds for i686, x86_64 and aarch64 Windows, and must stay a faithful
translation of the C shown.

Return ONLY the complete corrected function -- attributes, signature and body -- in a single
```rust fenced block. No prose.

Rules:
1. Change only what the errors require. Keep the control flow, every call, every constant
   name and every comment of the input. Do not restyle.
2. Keep the signature exactly (name, attributes, ABI, parameter and return types).
3. Add nothing outside the function: no `use`, `extern`, `struct`, `const`, `static`,
   `macro_rules!`, and no stub or forward declaration of a missing name.
4. A name rustc cannot find (constant, function, macro such as ASSERT or _SEH2_LEAVE) is
   supplied elsewhere later. Leave it EXACTLY as written: never substitute a different name,
   a number, or inline code for it, and never delete or comment out its statement. The only
   exception is C's min/max, which you may write inline. A TYPE or FIELD name rustc suggests
   a correction for may take that correction.
5. Never `.unwrap()`, `.expect()`, `try_into`, `try_from`, `panic!`, `todo!`,
   `unimplemented!`. C integer conversions are `as` casts: C truncates silently.
6. The same source compiles for 32- and 64-bit targets. Where a pointer-sized integer is
   needed, cast to the type the callee declares (SIZE_T, ULONG_PTR, usize, ...) -- never to
   u32 or u64 for that purpose. An error reported only on some targets is width-dependent.
7. C truthiness is explicit in Rust: `x != 0`, `!p.is_null()`, `b != FALSE as BOOLEAN`.
8. Fields of a #[repr(packed)] struct must not be borrowed: read them by value or use
   `core::ptr::addr_of!` / `&raw const`.
"""


def build_prompt(
    rust_fn: str,
    errors: dict[str, set[str]],
    c_fn: str | None,
    facts: str,
    arches: list[str],
    refused: str = "",
) -> str:
    lines = ["## rustc errors in this function"]
    for rendered, where in sorted(errors.items()):
        tag = "all targets" if set(where) >= set(arches) else "only on " + ", ".join(sorted(where))
        lines.append(f"[{tag}]\n{rendered}")
    if facts:
        lines += ["", "## Declarations the errors mention (from the crate)"]
        lines += ["```rust", facts, "```"]
    if c_fn:
        lines += ["", "## The original C", "```c", c_fn.rstrip(), "```"]
    if refused:
        # The cheap retry. Escalating to thinking ON bought the SAME wrong answer at ~10x the
        # price on setuplib (InstallFat12BootCode -> InstallFat32BootCode, iswspace ->
        # isspace, twice each); telling the model what was refused is the missing input.
        lines += [
            "",
            "## Your previous fix of this function was REFUSED",
            refused,
            "Do not repeat it. Every name listed there must appear in your reply unchanged.",
        ]
    lines += ["", "## The Rust function to fix", "```rust", rust_fn.rstrip(), "```"]
    return "\n".join(lines) + "\n"


def bank_key(rust_fn: str, errors: dict[str, set[str]], thinking: str, refused: str = "") -> str:
    h = hashlib.sha256()
    h.update(f"{_KEY_SALT}|{thinking}|".encode())
    if refused:
        # A retry carrying the refusal is a different question; without this it would
        # replay the refused reply forever.
        h.update(b"refused\0" + refused.encode())
    h.update(rust_fn.encode())
    for rendered in sorted(errors):
        h.update(b"\0" + rendered.encode() + b"\1" + ",".join(sorted(errors[rendered])).encode())
    return h.hexdigest()[:20]


# --------------------------------------------------------------------------------------
# declarations the errors mention
# --------------------------------------------------------------------------------------

_BACKTICK = re.compile(r"`([^`\n]{1,200})`")
_IDENT = re.compile(r"\b[A-Za-z_]\w*\b")
_RUST_WORDS = frozenset(
    [
        "as",
        "fn",
        "mut",
        "const",
        "unsafe",
        "extern",
        "pub",
        "struct",
        "union",
        "enum",
        "type",
        "impl",
        "for",
        "in",
        "if",
        "else",
        "let",
        "match",
        "return",
        "while",
        "loop",
        "break",
        "continue",
        "self",
        "Self",
        "crate",
        "super",
        "use",
        "mod",
        "where",
        "dyn",
        "ref",
        "move",
        "static",
        "true",
        "false",
        "Option",
        "Some",
        "None",
        "core",
        "ptr",
        "mem",
        "c_void",
        "u8",
        "u16",
        "u32",
        "u64",
        "u128",
        "usize",
        "i8",
        "i16",
        "i32",
        "i64",
        "i128",
        "isize",
        "bool",
        "char",
        "str",
        "f32",
        "f64",
        "Default",
        "default",
        "Ok",
        "Err",
    ]
)


def mentioned_names(errors: dict[str, set[str]]) -> set[str]:
    names: set[str] = set()
    for rendered in errors:
        for m in _BACKTICK.finditer(rendered):
            names.update(w for w in _IDENT.findall(m.group(1)) if w not in _RUST_WORDS)
    return names


def declarations(names: set[str], sources: list[Path], limit: int = 6000) -> str:
    """`type`/`struct`/`union`/`const`/fn-signature lines that define the mentioned names."""
    found: dict[str, str] = {}
    for src in sources:
        try:
            text = src.read_text(errors="replace")
        except OSError:
            continue
        for name in sorted(names - set(found)):
            n = re.escape(name)
            m = re.search(
                rf"(?m)^[ \t]*pub (?:type {n}\b[^;]*;|const {n}\b[^;]*;"
                rf"|static (?:mut )?{n}\b[^;]*;)",
                text,
            )
            if m:
                found[name] = m.group(0).strip()
                continue
            m = re.search(rf"(?m)^[ \t]*pub (?:struct|union) {n}\b", text)
            if m:
                open_idx = text.find("{", m.end())
                semi = text.find(";", m.end())
                if open_idx < 0 or (0 <= semi < open_idx):
                    found[name] = text[m.start() : semi + 1].strip()
                else:
                    close = _matching_brace(text, open_idx)
                    if close is not None and close - m.start() < 2500:
                        found[name] = text[m.start() : close + 1].strip()
                continue
            m = re.search(rf"(?m)^[ \t]*pub (?:unsafe )?(?:extern \"[^\"]+\" )?fn {n}\s*\(", text)
            if m:
                close = text.find("{", m.end())
                semi = text.find(";", m.end())
                stop = min(x for x in (close, semi) if x >= 0) if max(close, semi) >= 0 else -1
                if stop > 0 and stop - m.start() < 1200:
                    found[name] = text[m.start() : stop].strip().rstrip("{").strip() + ";"
    out, size = [], 0
    for name in sorted(found):
        if size + len(found[name]) > limit:
            break
        out.append(found[name])
        size += len(found[name])
    return "\n".join(out)


# --------------------------------------------------------------------------------------
# accepting a reply
# --------------------------------------------------------------------------------------

_FENCE = re.compile(r"```(?:rust|rs)?\s*\n(.*?)```", re.S)
# transmute: UpdateFreeLoaderIni "fixed" an ABI mismatch by transmuting an extern "C" fn into
# the extern "system" slot -- cdecl into stdcall, a stack imbalance on i386. intrinsics:
# `core::intrinsics::va_arg` is nightly-only.
_FORBIDDEN = re.compile(
    r"\.unwrap\(\)|\.expect\(|\btry_into\b|\btry_from\b|\bpanic!|\btodo!|\bunimplemented!|"
    r"\bgoto\s+\w+\s*;|\btransmute\b|\bintrinsics::"
)
_OUTER_ITEM = re.compile(
    r"(?m)^[ \t]*(?:pub(?:\([^)]*\))?\s+)?(?:use\s|extern\s+(?:crate|\"[^\"]*\"\s*\{)|"
    r"struct\s|union\s|enum\s|const\s+[A-Z_]\w*\s*:|static\s|macro_rules!|type\s|mod\s|impl\b)"
)
_CODE_NOISE = re.compile(r"//[^\n]*|/\*.*?\*/|b?\"(?:\\.|[^\"\\])*\"", re.S)


def extract_reply(reply: str) -> str | None:
    blocks = _FENCE.findall(reply)
    if len(blocks) == 1:
        return blocks[0].strip() + "\n"
    if not blocks and _ITEM_FN.search(reply):
        return reply.strip() + "\n"
    return None


def _name_end(fn_text: str) -> int:
    m = _ITEM_FN.search(fn_text)
    return m.end() if m else 0


def _signature(fn_text: str) -> str:
    open_idx = _body_open(fn_text, _name_end(fn_text))
    head = fn_text[: open_idx if open_idx is not None else len(fn_text)]
    head = _CODE_NOISE.sub(" ", head)
    return re.sub(r"\s+", "", head)


def _code(text: str) -> str:
    return _CODE_NOISE.sub(" ", text)


# Errors about a TYPE or FIELD name: rustc's suggested rename (`FAT32_BOOTSECTOR` ->
# `_FAT32_BOOTSECTOR`, field `DeleteFile` -> `DeleteFileA`) names the same thing, so the
# reply may drop the name. Never for a value or function (E0425): on setuplib the model
# "fixed" an unresolved ERROR_SYSTEM_PARTITION_NOT_FOUND into ERROR_SYSTEM_DEVICE_NOT_FOUND,
# STATUS_NOT_SAME_DEVICE into the Win32 ERROR_NOT_SAME_DEVICE, two NTSTATUS names into bare
# hex, and iswspace into isspace -- rustc's own "similar name" suggestion, and wrong.
_RENAMABLE_CODES = frozenset({"E0412", "E0422", "E0573", "E0574", "E0609", "E0560", "E0026"})
_ERROR_CODE = re.compile(r"^error\[(E\d{4})\]")
_INT_LITERAL = re.compile(r"\b(0x[0-9A-Fa-f_]+|\d[\d_]*)(?:[iu](?:8|16|32|64|128|size))?\b")
# Capitalised names, and lowercase names CALLED as functions or macros -- but not METHOD
# calls (`.as_ptr()`, `.is_null()`): those are the Rust spelling the fix may change.
_KEEP_IDENT = re.compile(r"(?<![.\w])(?:_*[A-Z]\w*|[a-z_]\w*(?=\s*!?\s*\())")
# C's min/max are the one idiom the prompt lets the model write inline.
_INLINE_OK = frozenset({"min", "max"})
# Control-flow MACROS whose Rust spelling is a construct, not a name: the dummy-PSEH
# `_SEH2_LEAVE` is a jump to the end of the try block (a labeled `break`), `_SEH2_END` closes
# it, ntbasedef's `DEFAULT_UNREACHABLE` is `default: __builtin_unreachable()`. Dropping one is
# allowed only when the reply contains the construct that replaces it.
_REPLACED_BY = {
    "_SEH2_LEAVE": re.compile(r"\bbreak\s+'\w+"),
    "_SEH2_END": re.compile(r""),
    "_SEH2_TRY": re.compile(r"'\w+\s*:\s*\{"),
    "_SEH2_FINALLY": re.compile(r""),
    "DEFAULT_UNREACHABLE": re.compile(r"\bunreachable(?:_unchecked)?\b"),
}


def _renamable(errors: dict[str, set[str]]) -> set[str]:
    out: set[str] = set()
    for rendered in errors:
        m = _ERROR_CODE.match(rendered)
        if m and m.group(1) in _RENAMABLE_CODES:
            out |= mentioned_names({rendered: set()})
    return out


def _literal_values(code: str) -> set[int]:
    vals = set()
    for m in _INT_LITERAL.finditer(code):
        raw = m.group(1).replace("_", "")
        try:
            vals.add(int(raw, 16) if raw.lower().startswith("0x") else int(raw))
        except ValueError:
            continue
    return vals


_ITEM_NAME = re.compile(r"\b(?:struct|union|enum|type|const|static(?:\s+mut)?|fn)\s+([A-Za-z_]\w*)")


def _items_c_defines(new_code: str, old_code: str, c_fn: str | None) -> bool:
    """Every nested item the reply ADDS names a type the C function itself defines locally
    (bootsup.c declares `struct _DISK_GEOMETRY_EX_INTERNAL` inside its function)."""
    if not c_fn:
        return False
    added = set(_ITEM_NAME.findall(new_code)) - set(_ITEM_NAME.findall(old_code))
    return bool(added) and all(re.search(rf"\b{re.escape(n)}\b", c_fn) for n in added)


def reply_findings(
    original: str,
    fixed: str,
    errors: dict[str, set[str]],
    parse_check,
    type_names: frozenset[str] | set[str] = frozenset(),
    c_fn: str | None = None,
    dead_names: frozenset[str] = frozenset(),
) -> list[str]:
    """Why `fixed` must not replace `original`; empty when it may.

    The model is asked to change only what the errors require; these check it did, on the
    things a compile cannot: WHICH constant, WHICH callee, WHICH statements.
    """
    out: list[str] = []
    spans = function_spans(fixed)
    if len(spans) != 1 or spans[0].start > 0 or fixed[spans[0].end :].strip():
        return ["the reply is not exactly one function"]
    orig_name = function_spans(original)[0].name
    if spans[0].name != orig_name:
        return [f"the reply defines `{spans[0].name}`, not `{orig_name}`"]
    if _signature(fixed) != _signature(original):
        # The signature may change only when an error is IN it.
        sig_end = _body_open(original, _name_end(original)) or 0
        if not any("fn " + orig_name in r and "(" in r for r in errors) or sig_end == 0:
            out.append("the signature changed")
    new_code, old_code = _code(fixed), _code(original)
    # Only a construct the reply ADDED: one already in the input is not the reply's doing.
    added = sorted(
        {m.group(0) for m in _FORBIDDEN.finditer(new_code)}
        - {m.group(0) for m in _FORBIDDEN.finditer(old_code)}
    )
    if added:
        out.append("forbidden construct " + ", ".join(f"`{a}`" for a in added))
    if len(_OUTER_ITEM.findall(new_code)) > len(
        _OUTER_ITEM.findall(old_code)
    ) and not _items_c_defines(new_code, old_code, c_fn):
        out.append("the reply declares an item inside the function (stub / import / type)")
    # Every capitalised name (constants, types, C callees, locals) and every lowercase
    # callee must survive. A name rustc cannot find is NOT exempt: it is resolve's job to
    # supply it, and the model's substitute is how a wrong constant gets in.
    # A TYPE name may go (`f as PPROCESS_ENTRY_ROUTINE` -> `Some(f)`): which type a cast
    # names is checked by the compiler, unlike which constant or callee a body uses.
    keep = (
        set(_KEEP_IDENT.findall(old_code))
        - _RUST_WORDS
        - _renamable(errors)
        - _INLINE_OK
        - set(type_names)
        - dead_names  # the C has them only on lines the build never compiles
    )
    # By COUNT, not presence: UpdateFreeLoaderIni hand-expanded one `MAKESTRKEY!(..)` and
    # passed because a second call kept the name in the function.
    old_uses = Counter(n for n in _KEEP_IDENT.findall(old_code) if n in keep)
    new_uses = Counter(re.findall(r"\b\w+\b", new_code))
    lost = {n for n, k in old_uses.items() if new_uses[n] < k}
    lost = {n for n in lost if not (n in _REPLACED_BY and _REPLACED_BY[n].search(new_code))}
    if lost:
        out.append("names dropped: " + ", ".join(sorted(lost)[:8]))
    magic = {v for v in _literal_values(new_code) - _literal_values(old_code) if v >= 0x100}
    if magic:
        out.append("new numeric literals: " + ", ".join(hex(v) for v in sorted(magic)[:6]))
    out += parse_check(fixed)
    return out


# --------------------------------------------------------------------------------------
# banking
# --------------------------------------------------------------------------------------


def bank_read(bank: Path, key: str) -> dict | None:
    p = bank / f"{key}.json"
    try:
        return json.loads(p.read_text())
    except (OSError, ValueError):
        return None


def bank_write(bank: Path, key: str, record: dict) -> None:
    bank.mkdir(parents=True, exist_ok=True)
    tmp = bank / f"{key}.tmp"
    tmp.write_text(json.dumps(record, indent=1))
    tmp.replace(bank / f"{key}.json")


# --------------------------------------------------------------------------------------
# the loop
# --------------------------------------------------------------------------------------


@dataclass
class Attempt:
    file: str
    name: str
    key: str
    original: str
    fixed: str | None = None
    status: str = ""  # replayed / bought / rejected:<why> / failed:<why>
    usd: float = 0.0


@dataclass
class RoundReport:
    errors_before: dict[str, int]
    errors_after: dict[str, int] = field(default_factory=dict)
    attempts: list[Attempt] = field(default_factory=list)
    kept: list[str] = field(default_factory=list)
    reverted: list[str] = field(default_factory=list)
    usd: float = 0.0


def c_function_text(c_path: Path, name: str, cache: dict) -> str | None:
    """The C definition of `name` as written in the file -- return type through closing brace.

    Located in the RAW text rather than by the scanner's line numbers: the scanner works on a
    preprocessed copy (`#if 0` arms dropped, includes spliced), so its lines drift and the
    first try cut ArcGetNextTokenA off before its last statements.
    """
    if c_path not in cache:
        try:
            cache[c_path] = c_path.read_text(errors="replace")
        except OSError:
            cache[c_path] = ""
    text = cache[c_path]
    for m in re.finditer(rf"\b{re.escape(name)}\s*\(", text):
        line_start = text.rfind("\n", 0, m.start()) + 1
        if text[line_start : m.start()].strip().startswith(("#", "//")):
            continue
        close = _matching_paren(text, m.end() - 1)
        if close is None:
            continue
        rest = text[close + 1 :]
        stripped = rest.lstrip()
        if not stripped.startswith("{"):
            continue
        open_idx = close + 1 + (len(rest) - len(stripped))
        end = _matching_brace(text, open_idx)
        if end is None:
            continue
        # Back up over the return type / qualifiers to the previous item's end.
        head = line_start
        while head > 0:
            prev_start = text.rfind("\n", 0, head - 1) + 1
            prev = text[prev_start : head - 1].strip()
            if not prev or prev.endswith((";", "}", "*/")) or prev.startswith(("#", "//")):
                break
            head = prev_start
        return _live_text(c_path, text, head, end + 1, name, cache)
    return None


def _live_text(c_path: Path, text: str, start: int, end: int, name: str, cache: dict) -> str:
    """The function with the lines this build never compiles taken out, and the names that
    appeared ONLY on those lines recorded for the guard (see :func:`c_dead_names`).

    osdetect.c `#define`s NDEBUG and then calls `DumpNTOSInstalls(List)` under `#ifndef
    NDEBUG`; the dumper's own body was rightly never translated, but the call was, and the
    guard then refused every fix that dropped it.
    """
    from .dead_regions import never_compiled_lines

    key = ("dead-lines", c_path)
    if key not in cache:
        cache[key] = never_compiled_lines(text)
    dead = cache[key]
    first = text.count("\n", 0, start) + 1
    live, gone = [], []
    for n, line in enumerate(text[start:end].split("\n"), first):
        (gone if n in dead else live).append(line)
    if gone:
        live_words = set(re.findall(r"\b[A-Za-z_]\w*\b", "\n".join(live)))
        cache[("dead-names", c_path, name)] = frozenset(
            set(re.findall(r"\b[A-Za-z_]\w*\b", "\n".join(gone))) - live_words
        )
    return "\n".join(live)


def c_dead_names(c_path: Path, name: str, cache: dict) -> frozenset[str]:
    """Names the C function mentions only on lines this build never compiles."""
    c_function_text(c_path, name, cache)
    return cache.get(("dead-names", c_path, name), frozenset())


def _matching_paren(text: str, open_idx: int) -> int | None:
    depth, i, n = 0, open_idx, len(text)
    while i < n:
        j = _skip_literal(text, i)
        if j != i:
            i = j
            continue
        if text[i] == "(":
            depth += 1
        elif text[i] == ")":
            depth -= 1
            if depth == 0:
                return i
        i += 1
    return None


def _apply(root: Path, replacements: dict[str, list[tuple[FnSpan, str, str]]]) -> None:
    """Splice fixed functions in, bottom-up, refusing if the file moved under us."""
    for fname, reps in replacements.items():
        p = root / fname
        text = p.read_bytes().decode("utf-8")
        for span, original, fixed in sorted(reps, key=lambda r: r[0].start, reverse=True):
            if text[span.start : span.end] != original:
                raise RuntimeError(f"crate_fix: {fname}:{span.name} changed under the fix")
            text = text[: span.start] + fixed.rstrip("\n") + text[span.end :]
        tmp = p.with_suffix(".rs.fixtmp")
        tmp.write_bytes(text.encode("utf-8"))
        tmp.replace(p)
    forget_file_cache()


def _replace_named(root: Path, fname: str, name: str, new_text: str) -> bool:
    p = root / fname
    text = p.read_bytes().decode("utf-8")
    for sp in function_spans(text):
        if sp.name == name:
            text = text[: sp.start] + new_text.rstrip("\n") + text[sp.end :]
            tmp = p.with_suffix(".rs.fixtmp")
            tmp.write_bytes(text.encode("utf-8"))
            tmp.replace(p)
            forget_file_cache()
            return True
    return False
