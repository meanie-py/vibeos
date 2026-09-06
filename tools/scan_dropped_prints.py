#!/usr/bin/env python3
"""
scan_dropped_prints.py — the C told us why it failed. The translation kept that
sentence as a comment and shipped silence.

THE POINT. Same oracle as `scan_mask_drift`, same family: rustconv keeps the
original C as trailing comments in the same `.rs`. So a function whose commented
C prints a diagnostic on a failure path, and whose live Rust returns from that
same path saying nothing, has stated its own intent and dropped it — visible in
one file, with no header catalog, no C build, and no cross-file reasoning.

WHY IT MATTERS, from the incident that opened this. `CmpLinkKeyToHive` has three
failure returns and upstream prints on each ("couldn't create %S, Status = ...",
"%S already exists!", "couldn't create symbolic link for %S, ..."). All three
were dropped. When `\\Registry\\User\\S-1-5-18` did not appear, there was no
signal anywhere — the key simply was not there. Restoring the prints, and then
getting SILENCE back from them, is what proved the create half was fine and
moved the hunt to the reparse. That cost two build+boot cycles which one grep
would have given for free.

WHY NOTHING ELSE SEES THIS CLASS. `scan_silent_failures` covers `loop {}`,
`.unwrap()`, `Err(_) => {}` — SHAPES, things that are present and wrong. A branch
that returns a correct error code without printing is not a shape, it is an
ABSENCE, and absence is only visible against an oracle that says what should
have been there. The commented C is that oracle.

WHICH MACROS COUNT, and why the line is drawn there. Only the ones upstream
actually emits on a default build:

    DPRINT1  DbgPrint  DbgPrintEx  vDbgPrintEx      (ReactOS kernel: unconditional)
    ERR  FIXME                                      (wine channels: on by default)

Deliberately NOT counted: DPRINT (3579 sites), TRACE (3442), WARN (247). Those
are compiled out or off by default upstream, so dropping them loses nothing that
upstream would have printed either. They are the larger population and they are
excluded on purpose, not by oversight — if we ever ship DBG=1 media that wants
them, widen ALWAYS_ON and re-seed.

Anything that looks like a print but prints nothing does NOT count as live. The
concrete case is `dprint_noop!`, which takes the format string, drops it, and
returns 0. A no-op print is exactly the silence this scanner exists to find, so
it is scored as absent no matter how much it reads like a call.

PAIRING is by function NAME, exact first and case-insensitively second, and only
when BOTH sides exist. A C function with no live Rust counterpart is untranslated
— not this scanner's business, and reporting it would bury the real findings
under every file's still-C helpers.

THREE KINDS.

  ALL      the C function prints and the live one never does. High confidence:
           the whole function's diagnostics are gone. Burn these down first.
  PARTIAL  live prints, but fewer times than the C does. ADVISORY — read it
           before believing it. `scan_mask_drift` learned the hard way that
           counting occurrences is a trap ("the C masks it 4 times, the Rust 3"),
           and the same trap is here: a translation may legitimately fold two
           error returns into one path with one print. PARTIAL is kept anyway
           because dropping it would hide a function that lost eight prints and
           kept one, which is most of them.
  NOOP     this file DEFINES the macro as a no-op — either `fn ERR(..) {}` (an
           out-of-line empty function) or `macro_rules! DPRINT1 { ($($t:tt)*) => {}; }`
           (an empty expansion). Every call to it in the file reads like a diagnostic
           and emits nothing, which is why NOOP also makes those callers score as
           silent. The `fn` form is independently forbidden: AGENTS.md says never
           translate a C debug macro into a no-op `fn`, because an out-of-line no-op is
           an optimization barrier that uncloaks latent ABI mismatches into a `ud2`
           (field manual §9.65). Use a `macro_rules!` — but an EMPTY `macro_rules!` is
           still silence, and this scanner missed that half for a long time: acpi-rust
           and pci-rust each opened nine files with
           `// DPRINT/DPRINT1 are no-ops under the driver's NDEBUG build` over an empty
           pair, and every one of the 56 `DPRINT1!(..)` call sites under them scored as
           a LIVE print. NDEBUG silences `DPRINT`; `DPRINT1` sits outside that guard in
           debug.h and is live whenever DBG is set, so the comment was half true and
           the code was wholly quiet.

           The same shim written in C is invisible from here and has to be
           grepped for separately — `dll/win32/cabinet/rust_cohab_stub.c` and
           `dll/win32/mlang/rust_cohab_stub.c` define no-op `ERR`/`FIXME`, so
           those two crates' always-on diagnostics go nowhere.

STRUCTURE, so nobody re-derives the parsing. The Rust side is tree-sitter, so a
`//` inside a string literal is not a comment. The C side is then recovered from
the comment nodes, its OWN comments and string literals blanked (a GUID format
like `L"{%08lx-..."` otherwise unbalances every brace after it), and functions
found by brace DEPTH rather than indentation — the `//` prefix shifts every
column, so indent proves nothing and depth is exact. A function's name is the
LAST identifier followed by `(` at paren depth 0 in its signature block, which
steps over SAL annotations (`_IRQL_requires_max_(PASSIVE_LEVEL)`) for free.

Usage:
    python3 tools/scan_dropped_prints.py                 # full report
    python3 tools/scan_dropped_prints.py --lint          # terse (CLI baseline gate)
    python3 tools/scan_dropped_prints.py --self-test     # prove the scanner can SEE
    python3 tools/scan_dropped_prints.py <file.rs ...>   # just these files

Suppress a verified false positive on the function's signature line:

    pub unsafe extern "system" fn Foo() {  // nolint: dropped-print (caller prints)

The `--lint` line carries no line numbers and no counts, so restoring one print
of eight does not churn the baseline — only reaching parity, or losing the last
print in a function, moves a row.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
RUST_ROOT = REPO / "sdk" / "lib" / "rust"

SKIP_FILE_RE = re.compile(r"(bindgen_[a-z0-9_]+\.rs$|/target/|/\.responses/)")
GENERATED_RE = re.compile(r"automatically generated by rust-bindgen|@generated")

# Upstream macros that print on a DEFAULT build. See the docstring for why
# DPRINT/TRACE/WARN are excluded — it is a decision, not an omission.
ALWAYS_ON = ("DPRINT1", "DbgPrintEx", "vDbgPrintEx", "DbgPrint", "ERR", "FIXME")
C_PRINT_RE = re.compile(r"(?<![A-Za-z0-9_])(" + "|".join(ALWAYS_ON) + r")\s*\(")

# Cheap byte prefilter: no always-on print anywhere means nothing to compare. `!` and `{`
# are in the class alongside `(` because a file whose only mention is `macro_rules! ERR {`
# over `ERR!(..)` call sites has no `ERR(` in it at all, and skipping it here is what let
# every empty `macro_rules!` debug macro in the tree go unexamined.
PREFILTER_RE = re.compile(rb"(?:DPRINT1|DbgPrint|ERR|FIXME)\s*[({!]")

NOLINT_RE = re.compile(r"\bnolint\b\s*[:(\[]?\s*([a-z0-9_-]*)", re.I)
NOLINT_IDS = {"", "all", "dropped-print", "dropped_print", "scan_dropped_prints"}

# Live Rust macro invocations that actually reach the serial port. A name is a
# print if it is listed here or simply contains "print" — crates spell their own
# (`dbg_print!`, `debug_print!`) — but never if it contains "noop".
#
# `err` earns its place the same way `fixme` does: both are ALWAYS_ON channels on
# the C side, and a usermode translation spells them either as a bare `ERR(...)`
# call (LIVE_PRINT_FN_RE) or as an `ERR!(...)` macro. FIXME was covered in both
# forms and ERR only in the first, so restoring a dropped ERR *as a macro* — the
# form every wine-derived crate actually uses — could never clear its own finding.
# A lint you cannot satisfy by doing the right thing reports noise, not silence.
LIVE_PRINT_NAMES = {
    "dbg_print",
    "dbg_print_ex",
    "debug_print",
    "dbgprint",
    "dprintf",
    "err",
    "trace",
    "error",
    "errors",
    "warn",
    "info",
    "fixme",
    "println",
    "eprintln",
}
# Substrings that disqualify a "…print…" name from counting as one. These
# format INTO A BUFFER and emit nothing: `rtl_string_cb_printf_w!` scored as a
# live print and hid three genuinely dropped diagnostics in CmpCreateControlSet
# before this list existed. A name in LIVE_PRINT_NAMES is exempt, which is how
# `dprintf` (KdpDprintf, a real print) survives the "printf" entry.
NOT_A_PRINT = ("noop", "sprint", "printf", "format", "_str", "str_")

MACRO_CALL_RE = re.compile(r"(?<![A-Za-z0-9_])([A-Za-z_][A-Za-z0-9_]*)\s*!")
# ...plus plain calls to the upstream names themselves. Usermode translations
# keep `FIXME(...)`/`ERR(...)` and bind them to the wine debug channel through
# an `extern` declaration, so the call reads exactly like the C did.
# ...and the plain-call printers a crate provides because the upstream spelling
# is unavailable to it. freeldr's `DbgPrintSz` is the whole reason this list is
# not just ALWAYS_ON: freeldr-rust builds on STABLE, where a C-variadic `DbgPrint`
# definition needs the nightly `c_variadic` feature, so its DbgPrint takes the
# format string ALONE and the crate prints formatted text as sprintf-into-a-buffer
# followed by DbgPrintSz. That reaches the serial port exactly as DbgPrint does.
# Missing it scored freeldr's RtlAssert -- which prints on every assertion -- as
# having dropped both of its diagnostics.
LIVE_PRINT_EXTRA = ("DbgPrintSz",)
LIVE_PRINT_FN_RE = re.compile(
    r"(?<![A-Za-z0-9_])(" + "|".join(ALWAYS_ON + LIVE_PRINT_EXTRA) + r")\s*\(",
)
DECL_BEFORE_RE = re.compile(r"\bfn\s+$")

# Translation routinely splits one C function into an exported wrapper plus the
# real body next to it — 910 `_body`, 580 `_impl` tree-wide, and a CamelCase
# `Impl` variant besides. The prints live in the body, so a scanner that pairs
# only on the exact name reports the wrapper as silent and is wrong every time.
IMPL_SUFFIXES = (
    "_impl",
    "_body",
    "_inner",
    "_core",
    "_helper",
    "_worker",
    "Impl",
    "Body",
    "Inner",
    "Core",
    "Helper",
    "Worker",
)

C_SIG_NAME_RE = re.compile(r"([A-Za-z_][A-Za-z0-9_]*)\s*\(")
C_SIG_REJECT_RE = re.compile(r"^\s*(typedef|struct|enum|union|extern\s*\"C\")\b")


def suppressed(text: str) -> bool:
    """True if `text` carries a nolint marker this scanner answers to."""
    m = NOLINT_RE.search(text)
    return bool(m) and m.group(1).lower() in NOLINT_IDS


def blank_c_noise(text: str) -> str:
    """Blank C comment bodies and string/char literal bodies, preserving length.

    Length and newlines are preserved so every offset still maps back to a line.
    The quotes and the `(`/`)` survive; only what is BETWEEN them is erased. That
    is what keeps `DPRINT1("{%08lx-%04x-...}")` from contributing a `{` to the
    brace depth that finds function bodies.
    """
    out = list(text)
    i, n = 0, len(text)
    while i < n:
        c = text[i]
        nxt = text[i + 1] if i + 1 < n else ""
        if c == "/" and nxt == "/":
            j = text.find("\n", i)
            j = n if j < 0 else j
            for k in range(i, j):
                out[k] = " "
            i = j
        elif c == "/" and nxt == "*":
            j = text.find("*/", i + 2)
            j = n if j < 0 else j + 2
            for k in range(i, j):
                if out[k] != "\n":
                    out[k] = " "
            i = j
        elif c in "\"'":
            j = i + 1
            while j < n and text[j] != c and text[j] != "\n":
                j += 2 if text[j] == "\\" else 1
            for k in range(i + 1, min(j, n)):
                out[k] = " "
            # An unterminated quote is prose ("couldn't"), not a literal: stop at
            # the newline rather than swallowing the rest of the file.
            i = j + 1 if j < n and text[j] == c else j
        else:
            i += 1
    return "".join(out)


_PARSER = None
_QUERY = None


def split_regions(src: bytes) -> tuple[str, str]:
    """(live_code, commented_c) — same length, comments blanked out of one and
    code blanked out of the other, so byte offsets mean the same thing in both.

    tree-sitter, not a line test: `//` inside a string literal is not a comment,
    and the whole premise here is that the two regions are told apart correctly.
    """
    import tree_sitter_rust as tsr
    from tree_sitter import Language, Parser, Query, QueryCursor

    global _PARSER, _QUERY
    if _PARSER is None:
        lang = Language(tsr.language())
        _PARSER = Parser(lang)
        _QUERY = Query(lang, "(line_comment) @c (block_comment) @c")

    tree = _PARSER.parse(src)
    nodes = sorted(
        QueryCursor(_QUERY).captures(tree.root_node).get("c", []), key=lambda n: n.start_byte
    )

    live = bytearray(src)
    craw = bytearray(b" " * len(src))
    for i, b in enumerate(src):
        if b == 0x0A:
            craw[i] = 0x0A
    for n in nodes:
        s, e = n.start_byte, n.end_byte
        for i in range(s, e):
            if live[i] != 0x0A:
                live[i] = 0x20
            craw[i] = src[i]
        # Erase the Rust comment opener/closer so what remains is plain C.
        for i in range(s, min(s + 2, e)):
            craw[i] = 0x20
        if src[s : s + 2] == b"/*":
            for i in range(max(s, e - 2), e):
                craw[i] = 0x20
    return live.decode("utf8", "replace"), craw.decode("utf8", "replace")


def c_func_name(sig: str) -> str | None:
    """The declared name in a C signature block, or None if it is not one."""
    # The signature is the last block of non-blank lines. Trailing blanks come
    # from the `{` sitting on its own line, which is the house style on both
    # sides — drop them before the block boundary is looked for.
    src_lines = sig.splitlines()
    while src_lines and not src_lines[-1].strip():
        src_lines.pop()
    lines: list[str] = []
    for ln in reversed(src_lines):
        if not ln.strip():
            break
        lines.append(ln)
    s = " ".join(reversed(lines)).strip()
    if not s or s.endswith(("=", ",")) or C_SIG_REJECT_RE.match(s):
        return None
    # The LAST identifier followed by `(` at paren depth 0. SAL annotations
    # (`_IRQL_requires_max_(PASSIVE_LEVEL)`) sit at depth 0 too but come first,
    # and every parameter annotation is inside the parameter list at depth 1.
    name = None
    for m in C_SIG_NAME_RE.finditer(s):
        depth = s.count("(", 0, m.start()) - s.count(")", 0, m.start())
        if depth == 0:
            name = m.group(1)
    return name


def c_functions(craw: str) -> dict[str, int]:
    """{function name: count of always-on print calls in its body}."""
    clean = blank_c_noise(craw)
    out: dict[str, int] = {}
    depth = 0
    seg_start = 0
    body_start = None
    sig = ""
    for i, ch in enumerate(clean):
        if ch == "{":
            if depth == 0:
                sig = clean[seg_start:i]
                body_start = i
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth <= 0:
                if depth == 0 and body_start is not None:
                    name = c_func_name(sig)
                    if name:
                        n = len(C_PRINT_RE.findall(clean[body_start:i]))
                        out[name] = out.get(name, 0) + n
                depth = 0
                body_start = None
                seg_start = i + 1
        elif ch == ";" and depth == 0:
            seg_start = i + 1
    return out


def _walk(node):
    stack = [node]
    while stack:
        n = stack.pop()
        yield n
        stack.extend(n.children)


def live_prints(text: str, noop: frozenset[str]) -> int:
    """Count of live Rust statements that actually reach the serial port.

    `noop` names print-shaped functions this same file DEFINES with an empty
    body, whose calls therefore print nothing. That resolution is file-local on
    purpose: an `extern` declaration could bind to a real channel or to a no-op
    C shim and this scanner cannot tell, so it assumes the honest one and the
    shims are reported separately (see NOOP).
    """
    n = 0
    for m in LIVE_PRINT_FN_RE.finditer(text):
        if m.group(1) in noop or DECL_BEFORE_RE.search(text[max(0, m.start() - 8) : m.start()]):
            continue
        n += 1
    for m in MACRO_CALL_RE.finditer(text):
        name = m.group(1).lower()
        # An explicit name always counts; otherwise it has to look like a print
        # and not be one of the buffer formatters.
        if name in LIVE_PRINT_NAMES or (
            "print" in name and not any(bad in name for bad in NOT_A_PRINT)
        ):
            n += 1
    return n


# The transcriber of a no-op arm: `=> {}`, `=> {{}}`, `=> { () }`, `=> {;}`. Anything
# else — an identifier, a literal, a call — means the arm expands to real code. Judged on
# the TOKENS rather than on tree-sitter's named children, because a macro's right-hand
# side is an unparsed `token_tree` either way.
_MACRO_SILENT_RE = re.compile(r"^[\s(){}\[\];,]*$")


def _macro_is_silent(src: bytes, node) -> bool:
    """True if every arm of this `macro_rules!` transcribes to nothing."""
    arms = [c for c in node.children if c.type == "macro_rule"]
    if not arms:
        return False
    for arm in arms:
        # The arm is `token_tree_pattern => token_tree`; only the transcriber on the
        # right is a plain `token_tree`, so it is the sole one to read.
        trees = [c for c in arm.children if c.type == "token_tree"]
        if not trees:
            return False
        body = src[trees[-1].start_byte : trees[-1].end_byte].decode("utf8", "replace")
        if not _MACRO_SILENT_RE.match(body):
            return False
    return True


def rust_functions(
    src: bytes, live: str
) -> tuple[dict[str, tuple[int, int, int]], dict[str, tuple[int, int]]]:
    """({fn name: (live print count, decl_start, body_end)}, {noop name: span}).

    The second dict is the print-shaped definitions in this file that emit nothing:
    a C debug macro lowered to an out-of-line `fn` with an empty body — which AGENTS.md
    forbids outright, an out-of-line no-op being an optimization barrier that uncloaks
    latent ABI mismatches into a `ud2` — and the `macro_rules!` spelling of the same
    silence, whose arms all transcribe to nothing. Both quiet every call site below.
    """
    import tree_sitter_rust as tsr
    from tree_sitter import Language, Parser

    global _PARSER
    if _PARSER is None:
        _PARSER = Parser(Language(tsr.language()))

    found = []
    noop: dict[str, tuple[int, int]] = {}
    for node in _walk(_PARSER.parse(src).root_node):
        if node.type != "function_item":
            continue
        nm = node.child_by_field_name("name")
        body = node.child_by_field_name("body")
        if nm is None or body is None:
            continue
        name = src[nm.start_byte : nm.end_byte].decode("utf8", "replace")
        found.append((name, node, body))
        if name in ALWAYS_ON and not body.named_children:
            noop[name] = (node.start_byte, body.end_byte)

    for node in _walk(_PARSER.parse(src).root_node):
        if node.type != "macro_definition":
            continue
        nm = node.child_by_field_name("name")
        if nm is None:
            continue
        name = src[nm.start_byte : nm.end_byte].decode("utf8", "replace")
        if name in ALWAYS_ON and _macro_is_silent(src, node):
            noop.setdefault(name, (node.start_byte, node.end_byte))

    quiet = frozenset(noop)
    out: dict[str, tuple[int, int, int]] = {}
    for name, node, body in found:
        n = live_prints(live[body.start_byte : body.end_byte], quiet)
        for key in base_names(name):
            prev = out.get(key)
            if prev is None:
                out[key] = (n, node.start_byte, body.end_byte)
            else:
                out[key] = (
                    prev[0] + n,
                    min(prev[1], node.start_byte),
                    max(prev[2], body.end_byte),
                )
    return out, noop


def base_names(name: str) -> set[str]:
    """`Foo_impl` also answers for `Foo` — see IMPL_SUFFIXES."""
    out = {name}
    for suf in IMPL_SUFFIXES:
        if name.endswith(suf) and len(name) > len(suf):
            out.add(name[: -len(suf)])
    return out


def scan_file(path: Path, rel: str) -> list[tuple[str, str, str, str]]:
    """[(rel, fn name, kind, detail)] for one translated file."""
    src = path.read_bytes()
    if not PREFILTER_RE.search(src):
        return []
    text = src.decode("utf8", "replace")
    if GENERATED_RE.search(text[:4096]):
        return []

    live, craw = split_regions(src)
    rfuncs, noop = rust_functions(src, live)

    out = []
    for name, (decl, end) in sorted(noop.items()):
        if not suppressed(text[max(0, decl - 240) : end]):
            out.append((rel, name, "NOOP", "expands to nothing — the call site prints nothing"))

    cfuncs = c_functions(craw)
    if not any(cfuncs.values()):
        return out

    # Exact name first; a case-insensitive fallback for translations that
    # re-cased, but only when it is unambiguous on both sides.
    ci_r: dict[str, list[str]] = {}
    for k in rfuncs:
        ci_r.setdefault(k.lower(), []).append(k)
    ci_c: dict[str, list[str]] = {}
    for k in cfuncs:
        ci_c.setdefault(k.lower(), []).append(k)

    for cname, cn in sorted(cfuncs.items()):
        if cn == 0:
            continue
        rname = cname
        if rname not in rfuncs:
            cand = ci_r.get(cname.lower(), [])
            if len(cand) != 1 or len(ci_c.get(cname.lower(), [])) != 1:
                continue
            rname = cand[0]
        rn, decl, end = rfuncs[rname]
        if rn >= cn:
            continue
        # A nolint anywhere in the function, including the attributes above it.
        if suppressed(text[max(0, decl - 240) : end]):
            continue
        kind = "ALL" if rn == 0 else "PARTIAL"
        out.append((rel, rname, kind, f"C prints {cn}x, live {rn}x"))
    return out


def rust_files() -> list[Path]:
    return [p for p in sorted(RUST_ROOT.rglob("*.rs")) if not SKIP_FILE_RE.search(p.as_posix())]


SELF_TEST: list[tuple[str, str, list[str]]] = [
    (
        "the motivating bug: every failure return printed in C, none in Rust",
        """
pub unsafe fn CmpLinkKeyToHive(a: u32) -> u8 {
    if a == 0 { return 0u8; }
    if a == 1 { return 0u8; }
    1u8
}
// BOOLEAN
// CmpLinkKeyToHive(
//     PCWSTR LinkKeyName)
// {
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("couldn't create %S, Status = 0x%lx\\n", LinkKeyName, Status);
//         return FALSE;
//     }
//     if (Disposition != REG_CREATED_NEW_KEY)
//     {
//         DPRINT1("%S already exists!\\n", LinkKeyName);
//         return FALSE;
//     }
//     return TRUE;
// }
""",
        ["ALL"],
    ),
    (
        "restored: live prints as often as the C does",
        """
pub unsafe fn Foo(a: u32) -> u8 {
    if a == 0 { crate::dbg_print!(b"nope\\0"); return 0u8; }
    1u8
}
// BOOLEAN
// Foo(ULONG a)
// {
//     if (!a) { DPRINT1("nope\\n"); return FALSE; }
//     return TRUE;
// }
""",
        [],
    ),
    (
        "one of three restored is PARTIAL, not silence",
        """
pub unsafe fn Foo(a: u32) -> u8 {
    if a == 0 { crate::dbg_print!(b"a\\0"); return 0u8; }
    if a == 1 { return 0u8; }
    if a == 2 { return 0u8; }
    1u8
}
// BOOLEAN
// Foo(ULONG a)
// {
//     if (a == 0) { DPRINT1("a\\n"); return FALSE; }
//     if (a == 1) { DPRINT1("b\\n"); return FALSE; }
//     if (a == 2) { DPRINT1("c\\n"); return FALSE; }
//     return TRUE;
// }
""",
        ["PARTIAL"],
    ),
    (
        "DPRINT/TRACE/WARN are out of scope — upstream does not print them either",
        """
pub unsafe fn Foo(a: u32) -> u8 { if a == 0 { return 0u8; } 1u8 }
// BOOLEAN
// Foo(ULONG a)
// {
//     if (!a) { DPRINT("quiet\\n"); TRACE("quiet\\n"); WARN("quiet\\n"); return FALSE; }
//     return TRUE;
// }
""",
        [],
    ),
    (
        "wine ERR and FIXME are on by default, so they count",
        """
pub unsafe fn Foo(a: u32) -> u8 { if a == 0 { return 0u8; } 1u8 }
// BOOL Foo(ULONG a)
// {
//     if (!a) { ERR("bad\\n"); FIXME("stub\\n"); return FALSE; }
//     return TRUE;
// }
""",
        ["ALL"],
    ),
    (
        "untranslated C function must NOT be reported — no live counterpart",
        """
pub unsafe fn Bar(a: u32) -> u8 { a as u8 }
// BOOLEAN
// Foo(ULONG a)
// {
//     if (!a) { DPRINT1("bad\\n"); return FALSE; }
//     return TRUE;
// }
""",
        [],
    ),
    (
        "a print in the C's OWN comment is not a print",
        """
pub unsafe fn Foo(a: u32) -> u8 { if a == 0 { return 0u8; } 1u8 }
// BOOLEAN
// Foo(ULONG a)
// {
//     /* upstream used to DPRINT1("bad\\n") here */
//     // DPRINT1("bad\\n");
//     return TRUE;
// }
""",
        [],
    ),
    (
        "a GUID format string must not unbalance the brace depth that finds bodies",
        """
pub unsafe fn Foo(a: u32) -> u8 { if a == 0 { return 0u8; } 1u8 }
pub unsafe fn Bar(a: u32) -> u8 { a as u8 }
// VOID Foo(ULONG a)
// {
//     DPRINT1("guid {%08lx-%04x-%04x}\\n", a);
// }
//
// VOID Bar(ULONG a)
// {
//     DPRINT1("second\\n");
// }
""",
        ["ALL", "ALL"],
    ),
    (
        "dprint_noop! prints nothing, so it does not count as live",
        """
pub unsafe fn Foo(a: u32) -> u8 {
    if a == 0 { crate::glue::dprint_noop!(b"bad\\0"); return 0u8; }
    1u8
}
// BOOLEAN Foo(ULONG a)
// {
//     if (!a) { DPRINT1("bad\\n"); return FALSE; }
//     return TRUE;
// }
""",
        ["ALL"],
    ),
    (
        "SAL annotations must not be mistaken for the function name",
        """
pub unsafe fn IopBuildSymbolicLink(a: u32) -> u32 { a }
// _IRQL_requires_max_(PASSIVE_LEVEL)
// static
// NTSTATUS
// IopBuildSymbolicLink(
//     _In_ PCUNICODE_STRING DeviceString,
//     _Out_ PUNICODE_STRING SymbolicLinkName)
// {
//     DPRINT1("RtlDuplicateUnicodeString() failed, Status 0x%08lx\\n", Status);
//     return Status;
// }
""",
        ["ALL"],
    ),
    (
        "a struct initializer is not a function, even with a call inside it",
        """
pub unsafe fn Foo(a: u32) -> u8 { a as u8 }
// static const THING Table[] =
// {
//     ENTRY(1), ENTRY(2),
// };
//
// VOID Foo(ULONG a)
// {
//     DPRINT1("real\\n");
// }
""",
        ["ALL"],
    ),
    (
        "nolint on the signature suppresses it",
        """
pub unsafe fn Foo(a: u32) -> u8 {  // nolint: dropped-print (caller prints)
    if a == 0 { return 0u8; }
    1u8
}
// BOOLEAN Foo(ULONG a)
// {
//     if (!a) { DPRINT1("bad\\n"); return FALSE; }
//     return TRUE;
// }
""",
        [],
    ),
    (
        "a DPRINT1 mentioned in LIVE Rust prose is not a live print",
        """
// The DPRINT1("bad") upstream has here was dropped on purpose.
pub unsafe fn Foo(a: u32) -> u8 { if a == 0 { return 0u8; } 1u8 }
// BOOLEAN Foo(ULONG a)
// {
//     if (!a) { DPRINT1("bad\\n"); return FALSE; }
//     return TRUE;
// }
""",
        ["ALL"],
    ),
    (
        "a direct DbgPrint() FFI call counts as live",
        """
pub unsafe fn Foo(a: u32) -> u8 {
    if a == 0 { DbgPrint(b"bad\\0".as_ptr()); return 0u8; }
    1u8
}
// BOOLEAN Foo(ULONG a)
// {
//     if (!a) { DbgPrint("bad\\n"); return FALSE; }
//     return TRUE;
// }
""",
        [],
    ),
    (
        "a bare FIXME()/ERR() call is the wine channel, and counts as live",
        """
unsafe extern "C" { fn FIXME(fmt: *const i8, ...); }
pub unsafe fn Foo(a: u32) -> u8 {
    if a == 0 { FIXME(b"stub\\0".as_ptr() as *const i8); return 0u8; }
    1u8
}
// BOOL Foo(ULONG a)
// {
//     if (!a) { FIXME("stub\\n"); return FALSE; }
//     return TRUE;
// }
""",
        [],
    ),
    (
        "wine ERR!/FIXME! in MACRO form are the same two channels as the bare calls",
        """
pub unsafe fn Foo(a: u32) -> u8 {
    if a == 0 { ERR!("bad\\n"); return 0u8; }
    if a == 1 { FIXME!("stub\\n"); return 0u8; }
    1u8
}
// BOOL Foo(ULONG a)
// {
//     if (!a) { ERR("bad\\n"); return FALSE; }
//     if (a == 1) { FIXME("stub\\n"); return FALSE; }
//     return TRUE;
// }
""",
        [],
    ),
    (
        "a debug macro lowered to an empty fn is a NOOP, and quiets its callers",
        """
unsafe fn ERR(_msg: *const i8) {}
pub unsafe fn Foo(a: u32) -> u8 {
    if a == 0 { ERR(b"bad\\0".as_ptr() as *const i8); return 0u8; }
    1u8
}
// BOOL Foo(ULONG a)
// {
//     if (!a) { ERR("bad\\n"); return FALSE; }
//     return TRUE;
// }
""",
        ["NOOP", "ALL"],
    ),
    (
        "formatting into a buffer is not printing",
        """
pub unsafe fn Foo(a: u32) -> u8 {
    if a == 0 { crate::rtl_string_cb_printf_w!(buf, 8, w!("x")); return 0u8; }
    1u8
}
// BOOLEAN Foo(ULONG a)
// {
//     if (!a) { DPRINT1("bad\\n"); return FALSE; }
//     return TRUE;
// }
""",
        ["ALL"],
    ),
    (
        "a wrapper + _impl split keeps the prints in the body, and that counts",
        """
pub unsafe extern "C" fn GdiSelectVisRgn(a: u32) { GdiSelectVisRgnImpl(a) }
unsafe fn GdiSelectVisRgnImpl(a: u32) {
    if a == 0 { crate::dbg_print!(b"bad\\0"); }
}
// VOID GdiSelectVisRgn(ULONG a)
// {
//     if (!a) { DPRINT1("bad\\n"); }
// }
""",
        [],
    ),
    (
        "an extern DECLARATION of a print is not a call site",
        """
unsafe extern "C" { fn DbgPrint(fmt: *const i8, ...); }
pub unsafe fn Foo(a: u32) -> u8 { if a == 0 { return 0u8; } 1u8 }
// BOOLEAN Foo(ULONG a)
// {
//     if (!a) { DbgPrint("bad\\n"); return FALSE; }
//     return TRUE;
// }
""",
        ["ALL"],
    ),
]


def self_test() -> int:
    import tempfile

    ok = 0
    for desc, body, want in SELF_TEST:
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / "t.rs"
            p.write_text(body)
            got = sorted(h[2] for h in scan_file(p, "t.rs"))
        if got == sorted(want):
            ok += 1
        else:
            print(f"FAIL: {desc}\n  want {sorted(want)}\n  got  {got}")
    print(f"self-test: {ok}/{len(SELF_TEST)}")
    return 0 if ok == len(SELF_TEST) else 1


def main() -> int:
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    lint = "--lint" in sys.argv
    if "--self-test" in sys.argv:
        return self_test()

    if args:
        paths = [Path(a).resolve() for a in args]
    else:
        paths = rust_files()

    hits = []
    for p in paths:
        try:
            rel = p.relative_to(REPO).as_posix()
        except ValueError:
            rel = p.as_posix()
        hits.extend(scan_file(p, rel))

    hits.sort()
    for rel, name, kind, detail in hits:
        suffix = "" if lint else f"  ({detail})"
        print(f"{rel}: {name} [{kind}]{suffix}")
    if not lint and hits:
        by_kind: dict[str, int] = {}
        for h in hits:
            by_kind[h[2]] = by_kind.get(h[2], 0) + 1
        tally = ", ".join(f"{v} {k}" for k, v in sorted(by_kind.items()))
        print(f"\n{len(hits)} functions ({tally})")
    return 1 if hits else 0


if __name__ == "__main__":
    sys.exit(main())
