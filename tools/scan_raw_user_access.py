#!/usr/bin/env python3
"""
scan_raw_user_access.py — Nt* syscalls that load or store through a caller's pointer.

THE POINT. An Nt* syscall's pointer arguments are user addresses for a user-mode
caller. The C probes them and then touches them inside `_SEH2_TRY`, so a page that
is unmapped (or freed by another thread after the probe) becomes a returned
STATUS_ACCESS_VIOLATION. This kernel has no SEH, so in Rust every such touch has to
go through a fault-tolerant primitive (`ntkrnl_sys::probe::write_back`,
`write_user`, `probe_read_generic`, `safe_copy_raw`, `OutputBounce`, ...). A plain
`*Ptr = x`, `(*Ptr).Field`, or `copy_nonoverlapping` on one is a bugcheck any
process can trigger: the probe proves where the buffer IS, never that it stays
mapped. `scan_raising_probes` sees the probe; this sees what comes after it.

WHAT IS FLAGGED. In the body of each `pub unsafe extern "system" fn Nt*`, a raw
access through one of its pointer-typed parameters:

    *P = ..   (*P).f   *(P as T)   = *P   P.add(..) deref   copy/move/fill/volatile
    and slice::from_raw_parts with P as an argument

`addr_of!((*P).f)` is address arithmetic, not an access, and is not flagged. A
parameter re-bound or re-assigned to something other than `opaque_ptr(P)` (a
capture, a pool bounce) stops being tracked from that line on, and so does the
kernel-mode arm of an explicit `if <mode> != KernelMode { .. } else { .. }` (or the
`==` form): the C trusts a kernel caller's pointers.

CALLEES. A tracked pointer passed straight on as a call argument (bare, cast with
`as`, `.cast()`, or `opaque_ptr(..)`) is followed into the callee's matching
parameter, transitively, and a raw access there is reported against the syscall
with the chain that reached it (`NtFoo P -> helper(q) -> inner(r)`). The callee is
resolved by name: a function in the same crate first, then a name defined exactly
once in the tree (the `#[no_mangle]` exports crates call each other through).
Ambiguous names are not followed.

EMBEDDED POINTERS. A value bound from `read_back(<tracked ptr>)` is a CAPTURED user
struct: reading its own fields is a kernel-local load, but a POINTER field of it
(`UNICODE_STRING.Buffer`, `OBJECT_ATTRIBUTES.ObjectName`) still addresses user
memory. A raw deref or copy of that field is reported just like the parameter, and
a field lifted into a local (`let b = cap.Buffer`) or handed straight to a callee
(`f(cap.ObjectName)`) carries the taint onward. `read_back` is the one load that is
user-safe by construction, so it marks the capture unambiguously; a scalar field
passed as a primitive's length argument (`RtlZeroMemory(pool, cap.MaximumLength)`)
is not a buffer and is not flagged.

WHAT IT CANNOT SEE. Whether the access sits in a kernel-mode-only branch (where the
pointer is trusted); a user struct captured by a primitive OTHER than `read_back`
(`safe_copy_from_user` into a local, then its buffer field used raw); an array of
user pointers indexed element by element; and pointers reached through method calls.
That is why this gates against a BASELINE
(`tools/lint-baseline/scan_raw_user_access.txt`) and not at zero: rows in the
baseline were read and are either kernel-mode branches or owed. A NEW row fails
`vibeos-cli check`. Suppress a reviewed site with `// nolint: raw-user-access
<reason>` on the line; in a callee shared with kernel callers, say which branch
makes it safe.

Usage:
    python3 tools/scan_raw_user_access.py            # rows: path: NtFoo Param
    python3 tools/scan_raw_user_access.py --lines    # also print each line
    python3 tools/scan_raw_user_access.py --lint     # terse rows (baseline gate)
    python3 tools/scan_raw_user_access.py <path ...>
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from source_walk import source_files

REPO = Path(__file__).resolve().parent.parent
DEFAULT_ROOT = REPO / "sdk" / "lib" / "rust"

SYSCALL_RE = re.compile(r'pub\s+unsafe\s+extern\s+"system"\s+fn\s+(Nt\w+)\s*\(')
NOLINT_RE = re.compile(r"nolint:\s*(raw-user-access|raw_user_access|scan_raw_user_access|all)\b")
# Pointer-typed parameters: raw pointers and the Windows P* aliases.
POINTER_TYPE_RE = re.compile(r"^\s*(\*\s*(mut|const)\b|P[A-Z0-9_]\w*$|LP[A-Z]\w*$)")
RAW_PRIMITIVES = (
    "copy_nonoverlapping",
    "copy",
    "write_bytes",
    "write_volatile",
    "read_volatile",
    "write_unaligned",
    "read_unaligned",
    "RtlCopyMemory",
    "RtlMoveMemory",
    "RtlZeroMemory",
    "RtlFillMemory",
    "from_raw_parts",
    "from_raw_parts_mut",
    "slice_from_raw_parts",
    "slice_from_raw_parts_mut",
)
SKIP_DIR_PARTS = ("-sys", "/target/", "rtl-strings", "ntdll")


def _strip(text: str) -> list[str]:
    """Code per line with // and /* */ comments and string contents blanked."""
    out: list[str] = []
    in_block = False
    for raw in text.splitlines():
        if not in_block and '"' not in raw and "/" not in raw:
            out.append(raw)  # nothing to blank; the per-char walk is the cost
            continue
        code: list[str] = []
        i = 0
        in_str = False
        while i < len(raw):
            if in_block:
                end = raw.find("*/", i)
                if end < 0:
                    i = len(raw)
                    break
                in_block = False
                i = end + 2
                continue
            ch = raw[i]
            if in_str:
                if ch == "\\":
                    i += 2
                    continue
                if ch == '"':
                    in_str = False
                    code.append('"')
                i += 1
                continue
            if raw.startswith("//", i):
                break
            if raw.startswith("/*", i):
                in_block = True
                i += 2
                continue
            if ch == '"':
                in_str = True
                code.append('"')
                i += 1
                continue
            code.append(ch)
            i += 1
        out.append("".join(code))
    return out


def _split_params(sig: str) -> list[tuple[str, str]]:
    params: list[str] = []
    depth = 0
    cur = ""
    for ch in sig:
        if ch in "(<[":
            depth += 1
        elif ch in ")>]":
            depth -= 1
        if ch == "," and depth == 0:
            params.append(cur)
            cur = ""
        else:
            cur += ch
    if cur.strip():
        params.append(cur)
    out = []
    for p in params:
        if ":" not in p:
            continue
        name, ty = p.split(":", 1)
        name = name.replace("mut ", "").strip()
        out.append((name, ty.strip()))
    return out


# Pointer-arithmetic methods: `P.offset(i)` addresses the same user memory as
# `P.add(i)`, so a deref of either is an access. Missing `.offset` hid a raw loop
# store `*access_status_list.offset(i) = ..` (se/audit.c result write-back).
PTR_ARITH = r"(?:add|offset|sub|wrapping_add|wrapping_offset|wrapping_sub|byte_add|byte_offset)"


def _access_patterns(name: str) -> list[re.Pattern[str]]:
    n = re.escape(name)
    prims = "|".join(RAW_PRIMITIVES)
    return [
        re.compile(rf"(?<![\w.])\*\s*{n}\b\s*(\.{PTR_ARITH}\([^)]*\)\s*)?=(?!=)"),  # *P[.add(i)] =
        re.compile(rf"(?<![\w.])\*\s*{n}\s*\.\s*{PTR_ARITH}\s*\("),  # *P.offset(i) (read or store)
        re.compile(rf"\(\*{n}\)\s*\."),  # (*P).f
        re.compile(rf"\*\(\s*{n}\s+as\b"),  # *(P as T)
        re.compile(rf"=\s*\*\s*{n}\b"),  # = *P, = *P as T
        re.compile(
            rf"(?<![\w:])(?:core::ptr::|ptr::|core::slice::|slice::)?(?:{prims})"
            rf"\s*\((?:[^;]*?[(,\s])?{n}\b"
        ),
    ]


ADDR_OF_RE = re.compile(r"\baddr_of(?:_mut)?!\s*\(")


def _blank_addr_of(lines: list[str], start: int, end: int) -> list[str]:
    """Blank every addr_of!(..)/addr_of_mut!(..) span in lines[start..=end], across
    line breaks: computing an address through a pointer is not an access."""
    text = "\n".join(lines[start : end + 1])
    chars = list(text)
    for m in ADDR_OF_RE.finditer(text):
        depth = 0
        for k in range(m.end() - 1, len(text)):
            if text[k] == "(":
                depth += 1
            elif text[k] == ")":
                depth -= 1
                if depth == 0:
                    for q in range(m.start(), k + 1):
                        if chars[q] != "\n":
                            chars[q] = " "
                    break
    return "".join(chars).split("\n")


def _body_end(lines: list[str], start: int) -> int:
    depth = 0
    seen = False
    for i in range(start, len(lines)):
        for ch in lines[i]:
            if ch == "{":
                depth += 1
                seen = True
            elif ch == "}":
                depth -= 1
                if seen and depth == 0:
                    return i
    return len(lines) - 1


FN_RE = re.compile(
    r"^\s*(?:pub(?:\([\w:]+\))?\s+)?(?:const\s+)?(?:unsafe\s+)?"
    r'(?:extern\s+"[^"]+"\s+)?fn\s+(\w+)\s*(?:<[^(]*>)?\s*\('
)
CALL_RE = re.compile(r"(?<![\w.!])((?:\w+::)*)(\w+)\s*(?:::<[^(]*>)?\s*$")
KEYWORDS = frozenset({"if", "while", "match", "for", "return", "loop", "Some", "Ok", "Err"})


class FnDef:
    """One function with a body: where its signature ends and its body does."""

    def __init__(self, path: Path, rel: str, name: str, params, body0: int, end: int, raw, lines):
        self.path, self.rel, self.name = path, rel, name
        self.params: list[tuple[str, str]] = params
        self.body0, self.end = body0, end
        self.raw_lines, self.lines = raw, lines
        self.crate = rel.split("/src/", 1)[0]


def _signature(raw_lines: list[str], lines: list[str], i: int, col: int) -> tuple[str, int]:
    """Text between the '(' at (i, col) and its match, and the line after it closes."""
    sig = ""
    depth = 0
    j = i
    done = False
    while j < len(lines) and not done:
        seg = raw_lines[j][col:] if j == i else lines[j]
        for ch in seg:
            if ch == "(":
                depth += 1
                if depth == 1:
                    continue
            elif ch == ")":
                depth -= 1
                if depth == 0:
                    done = True
                    break
            sig += ch
        sig += " "
        j += 1
    return sig, j


def parse_file(path: Path, rel: str) -> list[FnDef]:
    try:
        text = path.read_text(errors="replace")
    except OSError:
        return []
    raw_lines = text.splitlines()
    lines = _strip(text)
    fns: list[FnDef] = []
    i = 0
    while i < len(lines):
        # Signature detection reads the raw line for the ABI string (_strip blanks it).
        m = FN_RE.match(raw_lines[i]) if i < len(raw_lines) else None
        if not m or raw_lines[i].lstrip().startswith("//"):
            i += 1
            continue
        sig, j = _signature(raw_lines, lines, i, m.end() - 1)
        # A declaration (extern block, trait item) ends in ';' before any '{'.
        k = j - 1
        tail = lines[k][lines[k].find(")") + 1 :] if k == i else lines[k]
        while k < len(lines) and "{" not in tail and ";" not in tail:
            k += 1
            tail = lines[k] if k < len(lines) else "{"
        if ";" in tail.split("{", 1)[0]:
            i = k + 1
            continue
        end = _body_end(lines, j - 1)
        fns.append(FnDef(path, rel, m.group(1), _split_params(sig), j - 1, end, raw_lines, lines))
        i = j
    return fns


def _forwarded_arg(text: str, name: str) -> bool:
    """`name` IS the argument, not something computed from it."""
    n = re.escape(name)
    return bool(
        re.fullmatch(
            rf"\s*(?:{n}(?:\s+as\s+[^,]+)?|{n}\s*\.\s*cast(?:_mut|_const)?(?:::<[^>]*>)?\(\)"
            rf"|(?:\w+::)*opaque_ptr(?:_const)?\(\s*{n}\s*\))\s*",
            text,
            re.S,
        )
    )


def _forwarded_field_arg(text: str, name: str) -> bool:
    """A POINTER field of the capture `name` IS the argument (`cap.Buffer`,
    `cap.ObjectName as PUNICODE_STRING`). A scalar field passed this way reaches a
    non-pointer parameter, which the caller filters out, so no field whitelist is
    needed here."""
    n = re.escape(name)
    return bool(re.fullmatch(rf"\s*{n}\s*\.\s*\w+\s*(?:as\s+[^,]+)?\s*", text, re.S))


def _call_sites(body: str, name: str, is_fwd=_forwarded_arg) -> list[tuple[str, int, int, int]]:
    """(callee, argument index, offset, arg count) per call taking `name` as an arg."""
    out = []
    for m in re.finditer(rf"(?<![\w.]){re.escape(name)}\b", body):
        # The innermost unclosed '(' before the token.
        depth = 0
        k = m.start() - 1
        while k >= 0:
            c = body[k]
            if c in ")]}":
                depth += 1
            elif c in "([{":
                if depth == 0:
                    break
                depth -= 1
            k -= 1
        if k < 0 or body[k] != "(":
            continue
        call = CALL_RE.search(body[max(0, k - 160) : k])  # end-anchored: a window, not the prefix
        if not call or call.group(2) in KEYWORDS:
            continue
        # Split the argument list at top-level commas up to its ')'.
        args, cur, depth, q = [], "", 0, k + 1
        idx = None
        while q < len(body):
            c = body[q]
            if q == m.start():
                idx = len(args)
            if c in "([{":
                depth += 1
            elif c in ")]}":
                if depth == 0:
                    break
                depth -= 1
            if c == "," and depth == 0:
                args.append(cur)
                cur = ""
            else:
                cur += c
            q += 1
        args.append(cur)
        if idx is not None and idx < len(args) and is_fwd(args[idx], name):
            out.append((call.group(2), idx, m.start(), len(args)))
    return out


# A value bound from `read_back(<user ptr>)` is a CAPTURED user struct: reading its
# own fields is now a kernel-local load, but a POINTER field of it
# (`UNICODE_STRING.Buffer`, `OBJECT_ATTRIBUTES.ObjectName`) still addresses user
# memory. Dereferencing or copying from that field raw -- not through a
# fault-tolerant primitive -- is the same any-process bugcheck as touching the
# original pointer, one level removed, and the parameter walk above cannot see it
# because the field pointer is a fresh local. `read_back` is the one load that is
# user-safe by construction, so it marks the capture unambiguously; a pointer field
# stored into a local (`let b = cap.Buffer`) carries the taint onward like a param.
READBACK_RE = re.compile(r"\bread_back\s*(?:::<[^>]*>)?\s*\(")
FIELD_SINK_PRIMS = RAW_PRIMITIVES  # the slice/copy primitives are a raw read of a field
ASSIGN_RE = re.compile(r"(?:\blet\s+(?:mut\s+)?)?(\w+)\s*(?::[^=;{}]*)?=(?!=)")


def _stmt_end(text: str, start: int) -> int:
    """Offset of the `;` (or unbalanced close) that ends the statement at `start`."""
    depth = 0
    for k in range(start, len(text)):
        c = text[k]
        if c in "([{":
            depth += 1
        elif c in ")]}":
            if depth == 0:
                return k
            depth -= 1
        elif c == ";" and depth == 0:
            return k
    return len(text)


def _first_arg(text: str, lpar: int) -> str:
    """The first top-level argument of the call whose `(` is at `lpar`."""
    depth = 0
    out: list[str] = []
    for k in range(lpar, len(text)):
        c = text[k]
        if c in "([{":
            depth += 1
            if depth == 1:
                continue
        elif c in ")]}":
            depth -= 1
            if depth == 0:
                break
        if depth == 1 and c == ",":
            break
        out.append(c)
    return "".join(out)


def _field_patterns(s: str) -> list[re.Pattern[str]]:
    e = re.escape(s)
    prims = "|".join(re.escape(p) for p in FIELD_SINK_PRIMS)
    return [
        re.compile(rf"(?<![\w.])\*\s*{e}\s*\.\s*\w+"),  # *cap.field  (deref/store)
        # a raw copy/slice primitive reading a POINTER field of the capture: the
        # field must be followed by a comma with no intervening ')', so a scalar
        # length argument (`cap.Length)`) at the tail of the call is not matched.
        re.compile(
            rf"(?<![\w:])(?:core::ptr::|ptr::|core::slice::|slice::)?(?:{prims})\s*\([^;]*?"
            rf"{e}\s*\.\s*\w+\b[^;)]*,"
        ),
    ]


def _derived_patterns(name: str) -> list[re.Pattern[str]]:
    """Accesses through a pointer LIFTED from a capture (`let b = cap.Buffer`). Like
    `_access_patterns`, but the raw-primitive rule requires the pointer in a non-final
    argument position, so a scalar field (`let n = cap.MaximumLength`) passed as a
    length -- the last argument -- is not mistaken for a buffer."""
    n = re.escape(name)
    prims = "|".join(re.escape(p) for p in FIELD_SINK_PRIMS)
    return [
        re.compile(rf"(?<![\w.])\*\s*{n}\b"),  # *b  (deref read/store, *b.add(i)/.offset(i))
        re.compile(rf"\(\*{n}\)\s*\."),  # (*b).f
        re.compile(rf"\*\(\s*{n}\s+as\b"),  # *(b as T)
        re.compile(rf"=\s*\*\s*{n}\b"),  # = *b
        re.compile(
            rf"(?<![\w:])(?:core::ptr::|ptr::|core::slice::|slice::)?(?:{prims})"
            rf"\s*\([^;]*?{n}\b[^;)]*,"
        ),
    ]


def _captures(text: str, seed: set[str]) -> tuple[dict[str, int], list[tuple[str, int]]]:
    """Resolve, to a fixpoint, which locals hold a user-struct capture reachable from
    `seed` (the tracked pointers), and which locals are pointer fields lifted out of
    one. Returns {capture -> first def offset} and [(derived pointer, def offset)]."""
    captured: dict[str, int] = {}
    derived: list[tuple[str, int]] = []
    if "read_back" not in text:  # no capture possible; skip the fixpoint entirely
        return captured, derived
    tracked = set(seed)
    changed = True
    while changed:
        changed = False
        for m in ASSIGN_RE.finditer(text):
            lhs = m.group(1)
            init = text[m.end() : _stmt_end(text, m.end())]
            rb = READBACK_RE.search(init)
            if rb and lhs not in captured:
                arg = _first_arg(init, rb.end() - 1)
                # read_back of a tracked pointer, or of a pointer field of an
                # existing capture (`read_back(oa.ObjectName)`): still a capture.
                if any(_forwarded_arg(arg, t) for t in tracked) or any(
                    _forwarded_field_arg(arg, c) for c in captured
                ):
                    captured[lhs] = m.start()
                    changed = True
                continue
            if lhs in tracked:
                continue
            fld = re.match(r"\s*(\w+)\s*\.\s*\w+\s*(?:as\b|$)", init)
            if fld and fld.group(1) in captured and lhs not in tracked:
                derived.append((lhs, m.start()))
                tracked.add(lhs)
                changed = True
    return captured, derived


# `if <mode> != KernelMode { user } else { kernel }` and the `==` form. Only an
# explicit comparison against kernel mode counts; anything subtler stays flagged.
KERNEL_IF_RE = re.compile(
    r"\bif\s+\(?\s*(\w*(?:mode|Mode)\w*)\s*(==|!=)\s*"
    r"(?:0(?:i8|u8)?|KernelMode|KPROCESSOR_MODE::KernelMode)(?:\s+as\s+\w+)?\s*\)?\s*\{"
)


def _block_end(text: str, open_at: int) -> int:
    depth = 0
    for k in range(open_at, len(text)):
        if text[k] == "{":
            depth += 1
        elif text[k] == "}":
            depth -= 1
            if depth == 0:
                return k
    return len(text) - 1


def _kernel_only_spans(text: str) -> list[tuple[int, int]]:
    """Character spans of `text` that run only for a kernel-mode caller, whose
    pointers the C trusts without probing."""
    spans = []
    for m in KERNEL_IF_RE.finditer(text):
        then_open = m.end() - 1
        then_close = _block_end(text, then_open)
        if m.group(2) == "==":
            spans.append((then_open, then_close))
            continue
        e = re.match(r"\s*else\s*\{", text[then_close + 1 :])
        if e:
            else_open = then_close + e.end()
            spans.append((else_open, _block_end(text, else_open)))
        elif re.search(r"\breturn\b[^;{}]*;\s*$", text[then_open + 1 : then_close]):
            # `if user { ..; return ..; }` with no else: what follows is the kernel arm.
            depth = 0
            for k in range(then_close + 1, len(text)):
                if text[k] == "{":
                    depth += 1
                elif text[k] == "}":
                    if depth == 0:
                        spans.append((then_close, k))
                        break
                    depth -= 1
    return spans


def _walk(fn: FnDef, names: set[str]):
    """Direct raw accesses through `names` in fn's body, and the calls that pass one on.
    Yields ("hit", name, line, text) and ("call", name, line, (callee, argidx))."""
    body = _blank_addr_of(fn.lines, fn.body0, fn.end)
    text = "\n".join(body)
    starts = [0]
    for ln in body[:-1]:
        starts.append(starts[-1] + len(ln) + 1)
    kernel_lines: set[int] = set()
    for a, b in _kernel_only_spans(text):
        first, last = text.count("\n", 0, a), text.count("\n", 0, b)
        # The lines strictly inside the braces; the `} else {` line is shared.
        kernel_lines.update(range(fn.body0 + first + 1, fn.body0 + last))

    def scope_end(k: int, name: str) -> int:
        """Last line of the block enclosing line k -- or of the block around THAT,
        when this one is `if !name.is_null() { .. }`: past it, the name is either
        null or what it was re-pointed at, so the capture covers the rest."""
        pos = starts[k - fn.body0]
        depth = 0
        open_at = -1
        for i in range(pos - 1, -1, -1):
            if text[i] == "}":
                depth += 1
            elif text[i] == "{":
                if depth == 0:
                    open_at = i
                    break
                depth -= 1
        n = re.escape(name)
        guard = re.search(
            rf"\bif\s+(?:!\s*{n}\s*\.is_null\(\)|{n}\s*!=\s*(?:(?:core::)?ptr::)?null(?:_mut)?\(\))\s*$",
            text[:open_at],
        )
        if open_at >= 0 and guard:
            pos = open_at
        depth = 0
        for i in range(pos + (1 if pos == open_at else 0), len(text)):
            if text[i] == "{":
                depth += 1
            elif text[i] == "}":
                if depth == 0:
                    if pos == open_at:  # skip our own block's close, then the outer one
                        pos, open_at = -1, -2
                        continue
                    return fn.body0 + text.count("\n", 0, i)
                depth -= 1
        return fn.end

    # A pointer field lifted out of a capture (`let b = cap.Buffer`) is tracked like
    # a parameter from its definition on; captured structs grow a field-deref check.
    captured, derived = _captures(text, names)
    derived_names = {b for b, _ in derived}
    names = set(names) | derived_names

    # A rebind -- `let P = ..` or a plain `P = ..`, e.g. `prcl = addr_of_mut!(local)`
    # after capturing into it -- ends tracking only until its enclosing block
    # closes. Function-wide, one reassignment in one `match` arm (IntDefWindowProc's
    # `lParam = ..` for WM_CONTEXTMENU) hid every other arm's raw access.
    dead: dict[str, list[tuple[int, int]]] = {n: [] for n in names}
    # A derived pointer is dead before its own binding (where `cap.Buffer` appears as
    # a value, not a deref) and anywhere before it is defined.
    for b, off in derived:
        dead[b].append((fn.body0, fn.body0 + text.count("\n", 0, off)))
    patterns = {
        n: (_derived_patterns(n) if n in derived_names else _access_patterns(n)) for n in names
    }
    field_pats = {s: _field_patterns(s) for s in captured}
    seen_hits: set[tuple[str, int]] = set()  # dedup per-line and cross-line prim hits
    for k in range(fn.body0, fn.end + 1):
        code = body[k - fn.body0]
        rebind = re.match(r"\s*(?:let\s+(?:mut\s+)?)?(\w+)\s*(?::[^=]*)?=(?!=)\s*(.*)", code)
        if rebind and rebind.group(1) in names:
            n = rebind.group(1)
            rhs = rebind.group(2)
            # `let b = cap.Buffer` is what MAKES b a tracked (derived) pointer -- it is
            # the definition, not a rebind away, so it must not kill b for the scope.
            fld = re.match(r"(\w+)\s*\.\s*\w+", rhs)
            is_derive = bool(fld and fld.group(1) in captured)
            if not is_derive and not re.match(
                rf"\s*opaque_ptr(_const)?\(\s*{re.escape(n)}\s*\)", rhs
            ):
                dead[n].append((k, scope_end(k, n)))
                continue
        if NOLINT_RE.search(fn.raw_lines[k]) or k in kernel_lines:
            continue
        for name, pats in patterns.items():
            if any(a <= k <= b for a, b in dead[name]):
                continue
            if any(p.search(code) for p in pats) and (name, k + 1) not in seen_hits:
                seen_hits.add((name, k + 1))
                yield ("hit", name, k + 1, fn.raw_lines[k].strip())
        for s, pats in field_pats.items():
            if any(p.search(code) for p in pats) and (s, k + 1) not in seen_hits:
                seen_hits.add((s, k + 1))
                yield ("hit", s, k + 1, fn.raw_lines[k].strip())
    # Calls carrying a tracked pointer. A raw PRIMITIVE (copy_nonoverlapping, ...) is
    # reported as a hit here, not just per line above: `_call_sites` parses the
    # argument list across line breaks, so it catches a primitive whose pointer
    # argument sits on a later line -- which the per-line regex misses entirely.
    for name in names:
        for callee, idx, off, nargs in _call_sites(text, name):
            line = fn.body0 + text.count("\n", 0, off)
            if line in kernel_lines or any(a <= line <= b for a, b in dead[name]):
                continue
            if NOLINT_RE.search(fn.raw_lines[line]):
                continue
            # A primitive's pointer argument is never its last (length) argument, so
            # a scalar lifted from a capture and passed as a length does not flag.
            if callee in RAW_PRIMITIVES and idx < nargs - 1:
                if (name, line + 1) not in seen_hits:
                    seen_hits.add((name, line + 1))
                    yield ("hit", name, line + 1, fn.raw_lines[line].strip())
            elif callee not in RAW_PRIMITIVES:
                yield ("call", name, line + 1, (callee, idx))
    # A pointer field handed straight to a callee (`f(oa.ObjectName)`) is followed
    # like a parameter; a scalar field reaches a non-pointer parameter, which the
    # caller's POINTER_TYPE_RE filter drops.
    for s in captured:
        for callee, idx, off, nargs in _call_sites(text, s, _forwarded_field_arg):
            line = fn.body0 + text.count("\n", 0, off)
            if line in kernel_lines or NOLINT_RE.search(fn.raw_lines[line]):
                continue
            if callee in RAW_PRIMITIVES and idx < nargs - 1:
                if (s, line + 1) not in seen_hits:
                    seen_hits.add((s, line + 1))
                    yield ("hit", s, line + 1, fn.raw_lines[line].strip())
            elif callee not in RAW_PRIMITIVES:
                yield ("call", s, line + 1, (callee, idx))


FN_NAME_RE = re.compile(r"\bfn\s+(\w+)[^;{]*\{")


class Index:
    """Name -> candidate files from a cheap regex, parsed only when a call needs one.
    Parsing every file in the tree costs ~45s; the calls a user pointer reaches name
    a few dozen functions."""

    def __init__(self, files: list[tuple[Path, str]], texts: dict[Path, str]):
        self.files_for: dict[str, list[tuple[Path, str]]] = {}
        for path, rel in files:
            for name in set(FN_NAME_RE.findall(texts[path])):
                self.files_for.setdefault(name, []).append((path, rel))
        self.parsed: dict[Path, list[FnDef]] = {}

    def defs(self, path: Path, rel: str) -> list[FnDef]:
        if path not in self.parsed:
            self.parsed[path] = parse_file(path, rel)
        return self.parsed[path]

    def resolve(self, caller: FnDef, name: str) -> FnDef | None:
        """The one definition in the caller's crate, else the one in the tree. Narrowed
        on the regex index before parsing: helpers every crate defines its own copy of
        would otherwise parse most of the tree."""
        where = self.files_for.get(name, [])
        mine = [(p, r) for p, r in where if r.split("/src/", 1)[0] == caller.crate]
        if not mine and len(where) != 1:
            return None
        cands = [f for p, r in (mine or where) for f in self.defs(p, r) if f.name == name]
        return cands[0] if len(cands) == 1 else None


MAX_DEPTH = 6


def scan_syscall(fn: FnDef, index: Index) -> list[tuple[str, str, int, str, str]]:
    """(param, chain, line, text, rel) rows for one syscall, its callees included."""
    params = [n for n, ty in fn.params if POINTER_TYPE_RE.search(ty)]
    rows = []
    seen: set[tuple[str, str, str]] = set()
    # (function, its tracked param, the syscall param it came from, chain so far)
    work = [(fn, p, p, "") for p in params]
    while work:
        cur, name, origin, chain = work.pop()
        key = (cur.rel, cur.name, name)
        if key in seen:
            continue
        seen.add(key)
        for kind, _name, line, extra in _walk(cur, {name}):
            if kind == "hit":
                rows.append((origin, chain, line, extra, cur.rel))
            elif chain.count("->") < MAX_DEPTH:
                callee = index.resolve(cur, extra[0])
                if callee is None or callee is fn or extra[1] >= len(callee.params):
                    continue
                pname, pty = callee.params[extra[1]]
                if not POINTER_TYPE_RE.search(pty):
                    continue
                work.append((callee, pname, origin, f"{chain} -> {callee.name}({pname})"))
    return rows


def scan_tree(files: list[tuple[Path, str]]) -> dict[str, list[tuple[int, str]]]:
    texts: dict[Path, str] = {}
    for path, _ in files:
        try:
            texts[path] = path.read_text(errors="replace")
        except OSError:
            texts[path] = ""
    index = Index(files, texts)
    rows: dict[str, list[tuple[int, str]]] = {}
    for path, rel in files:
        if 'extern "system" fn Nt' not in texts[path]:
            continue
        for f in index.defs(path, rel):
            if not f.name.startswith("Nt") or not _is_syscall(f):
                continue
            for param, chain, line, text, where in scan_syscall(f, index):
                tail = "" if not chain else f" {chain} @{where}"
                rows.setdefault(f"{f.rel}: {f.name} {param}{tail}", []).append((line, text))
    return rows


def _is_syscall(f: FnDef) -> bool:
    for k in range(max(0, f.body0 - 40), f.body0 + 1):
        m = SYSCALL_RE.search(f.raw_lines[k])
        if m and m.group(1) == f.name and not f.raw_lines[k].lstrip().startswith("//"):
            return True
    return False


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--lint", action="store_true", help="terse rows for the baseline gate")
    ap.add_argument("--lines", action="store_true", help="print each flagged line")
    ap.add_argument("paths", nargs="*", type=Path)
    args = ap.parse_args(argv)

    files: list[Path] = []
    for p in args.paths or [DEFAULT_ROOT]:
        if p.is_dir():
            files.extend(sorted(source_files(p)))
        elif p.suffix == ".rs":
            files.append(p)
    picked: list[tuple[Path, str]] = []
    for f in files:
        try:
            rel = f.resolve().relative_to(REPO).as_posix()
        except ValueError:
            rel = f.as_posix()
        if any(part in rel for part in SKIP_DIR_PARTS) or "bindgen" in f.name:
            continue
        picked.append((f, rel))
    rows = scan_tree(picked)
    for row in sorted(rows):
        print(row)
        if args.lines and not args.lint:
            for line, text in rows[row]:
                print(f"    {line}: {text[:120]}")
    if not args.lint:
        print(f"\n{len(rows)} (syscall, pointer) pairs with a raw access", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
