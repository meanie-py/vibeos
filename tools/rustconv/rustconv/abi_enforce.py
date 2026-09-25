"""Target-agnostic ABI / `#[no_mangle]` enforcer driven by the C source.

Calling convention is the project's historical #1 i386 crash cause, and
the second-most-common rustconv defect (after dropped fns) is forgetting
`#[no_mangle]` on a translated non-static C function — the symbol then
silently vanishes when the `.c` is commented out of its CMake, breaking
the cohabitation link.

Unlike `abi_map.py`/`enforce_win32k_abi` (which fuse win32k-sys bindgen +
win32k.spec — bound to one subsystem), this module takes the **C source
file as the ground truth** and derives, per function:

    name -> (convention, is_static)

via the existing `scanner.py` (which already records
`calling_convention` ∈ {NTAPI, FASTCALL, WINAPI, …} and `is_static`).
That makes it point-anywhere: any `dll/`, `sdk/lib/`, `drivers/` C file
you're cohabiting to Rust. No per-target map to maintain.

Rewrite rules (applied to every fn DEFINED in the .rs that the C scan
also defines):

  * **NTAPI / WINAPI / stdcall**, *exported* → `#[no_mangle] pub unsafe
    extern "system"`. `extern "system"` is stdcall on i386 and cdecl on
    amd64/arm64 *by definition*, so one declaration is right on all three
    arches — no `#[cfg]` split needed.
  * **NTAPI / WINAPI / stdcall**, *static* → dual-cfg `extern "stdcall"` /
    `extern "C"`, file-private (see "Why `static` + system is a `#[cfg]`
    dual" below).
  * **FASTCALL** → dual-cfg: `extern "fastcall"` on x86, `extern "C"`
    elsewhere (there is no `extern "system"` equivalent that means
    fastcall, and fastcall only deviates on i386).
  * **cdecl / no qualifier** → `#[no_mangle] pub unsafe extern "C"`.

`static` in C controls **linkage, not calling convention**. It therefore
only decides *visibility* here: a `static` C fn gets **no `#[no_mangle]`
and no `pub`** (it must not be exported — it has internal linkage in C),
but it keeps the convention the C declared. This is the whole point:
`static HRESULT WINAPI Foo(...)` is internal-linkage AND `__stdcall`.
Dropping the stdcall and emitting a plain `unsafe fn` gives the function
the **Rust ABI**; parked in a C vtbl slot (every COM method in ole32 /
oleaut32 is exactly this shape) it corrupts the stack on i386 at the
first call — and links silently on amd64/arm64, where the Rust ABI and
stdcall happen to agree on argument passing. That defect shipped ~130
ole32 methods before it was caught; `audit_abi()` below is the detector.

**Why `static` + system is a `#[cfg]` dual, not `extern "system"`:** rustc
does not unify `extern "system"` with `extern "stdcall"` at the *type*
level on i686, even though they are the same ABI there — `expected
"stdcall" fn, found "system" fn`. Our COM bindgen spells vtbl slots
concretely (`com-sys/src/bindgen_i386.rs`: `Option<unsafe extern "stdcall"
fn(…)>`; `extern "C"` on amd64), so an `extern "system"` method literally
cannot be stored in its own vtbl slot on i386. Internal-linkage system fns
are therefore rendered like `fastcall` already is:

    #[cfg(target_arch = "x86")]      unsafe extern "stdcall" fn F(..) {..}
    #[cfg(not(target_arch = "x86"))] unsafe extern "C"        fn F(..) {..}

The body is duplicated under both cfgs — source bloat only (rustc compiles
one branch), and it keeps the DEFAULT rendering macro-free and
target-agnostic. The *exported* (non-static) case keeps plain
`extern "system"`: an export is not a vtbl slot, and `#[no_mangle] extern
"system"` decorates correctly as `_Name@N` on i386.

**`com_method_macro="stdmethod"`** switches that dual to a
`stdmethod! { fn NAME(..) -> R {..} }` invocation, which expands to exactly
the same dual but writes the body once. Use it for the COM DLL crates that
already define the macro (`ole32-rust/src/winemacros.rs:99`,
`oleaut32-rust/src/oleaut.rs:10`); leave it `None` for kernel/HAL/driver
targets, which have no such macro.

**Expect churn when re-running on an already-translated file.** Every
`static` C fn with no convention macro is plain cdecl, so it is now rendered
`unsafe extern "C" fn` where it used to be a bare `unsafe fn` (Rust ABI).
That is 127 internal helpers in ole32's storage32.rs alone. The rewrite is
correct and faithful to the C — a static helper's address can be handed to C
at any time (a vtbl, a callback table, a qsort comparator), and the Rust ABI
is not a stable one — but it is a large, benign diff on files that were
already reviewed. `audit_abi(strict_cdecl=False)` deliberately does NOT
report this class, precisely because it is latent rather than a live crash.
Functions already wrapped in a `stdmethod!` are left alone in both modes, so
hand-fixed COM files do not churn.

Idempotent on its own output, in both rendering modes. Consumed by
`postprocess` and the `abi-enforce` CLI verb.
"""

from __future__ import annotations

import bisect
import re
from functools import lru_cache
from pathlib import Path

# The convention-alias table lives in `conventions.py` — the SINGLE source of truth,
# shared with scanner.py (which needs the same names for its CALLING_CONVENTIONS) and
# verified against the real headers by tools/verify_conv_macros.py. Aliased to the old
# private names so the rest of this module is unchanged.
#
# This also absorbed a THIRD copy that used to live here as `_CONV_NORM` (8 entries, used
# as the fallback when _derive_convention comes up empty). Three lists, three different
# contents — so the same macro could be a known convention to one and invisible to another.
# One copy now.
from .conventions import CONV_KEYWORDS as _CONV_KEYWORDS
from .conventions import KW_TO_CONV as _KW_TO_CONV
from .scanner import scan_file

_COMMENT_RE = re.compile(r"/\*.*?\*/|//[^\n]*", re.S)


def _skip_ws_and_comments(text: str, i: int) -> int:
    """Advance past whitespace **and C comments** starting at `i`.

    The comment part is load-bearing: wine/ReactOS routinely annotate the last
    parameter, so the definition reads

        IEnumSTATSTG**  ppenum)    /* [out] */
        {

    A whitespace-only skip stops on the `/`, concludes "not a definition", and
    `_derive_convention` then returns None — whereupon the caller silently
    falls back to **cdecl**. That is how 17 `static … WINAPI` COM methods in
    storage32.c were mis-derived as cdecl (found by `audit_abi`, 2026-07): the
    same i386 callee-pops stack bug as the Rust-ABI one, one layer deeper.
    """
    n = len(text)
    while i < n:
        if text[i] in " \t\r\n":
            i += 1
        elif text.startswith("/*", i):
            end = text.find("*/", i + 2)
            if end == -1:
                return n
            i = end + 2
        elif text.startswith("//", i):
            end = text.find("\n", i)
            if end == -1:
                return n
            i = end + 1
        else:
            break
    return i


def _derive_convention(c_text: str, name: str) -> str | None:
    """Scan the C *text* for `name`'s definition and read the calling-
    convention macro from the qualifiers preceding the name.

    tree-sitter's `calling_convention` field is lossy on ReactOS K&R style
    (the macro sits on its own line, e.g. `BOOLEAN\\nNTAPI\\nRtlFoo(VOID)`),
    so we go to the text. We bound the search to the declaration's own
    return-type/qualifier run: from the name back to the previous `}`,
    `;`, or blank line. Returns the normalized convention, or None if the
    name's definition site can't be located (caller defaults to cdecl).
    """
    # Definition site: the `name(` whose matching ')' is followed by '{'.
    for m in re.finditer(rf"\b{re.escape(name)}\s*\(", c_text):
        paren_open = c_text.index("(", m.end() - 1)
        depth, i = 0, paren_open
        while i < len(c_text):
            if c_text[i] == "(":
                depth += 1
            elif c_text[i] == ")":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        # A definition has '{' next (a call/decl has ';' or ','). Trailing
        # per-parameter comments may sit in between — skip them too.
        j = _skip_ws_and_comments(c_text, i + 1)
        if j >= len(c_text) or c_text[j] != "{":
            continue

        # Bound the qualifier window: back to prev '}' / ';' / blank line.
        win_start = m.start()
        lo = max(0, win_start - 400)
        seg = c_text[lo:win_start]
        # Drop comments first: a doc block that merely *mentions* WINAPI must
        # not be mistaken for the qualifier (and its punctuation must not
        # truncate the window below).
        seg = _COMMENT_RE.sub(" ", seg)
        for sep in ("}", ";"):
            k = seg.rfind(sep)
            if k != -1:
                seg = seg[k + 1 :]
        # also cut at a blank line
        bl = seg.rfind("\n\n")
        if bl != -1:
            seg = seg[bl + 2 :]

        for kw in _CONV_KEYWORDS:
            if re.search(rf"\b{kw}\b", seg):
                return _KW_TO_CONV[kw]
        return "cdecl"
    return None


@lru_cache(maxsize=64)
def build_abi_map_from_c(c_path: Path) -> dict[str, tuple[str, bool]]:
    """name -> (convention, is_static) from a C source file.

    convention ∈ {"system", "fastcall", "cdecl"}. The authoritative name
    list + `is_static` come from the tree-sitter scanner; the convention
    is re-derived from the C *text* (the scanner field is lossy on K&R
    own-line macros — see `_derive_convention`). No explicit macro → cdecl.
    """
    scan = scan_file(c_path)
    c_text = c_path.read_text(errors="replace")
    out: dict[str, tuple[str, bool]] = {}
    for f in scan.functions:
        conv = _derive_convention(c_text, f.name)
        if conv is None:
            # Fall back to the (lossy) tree-sitter field, then cdecl.
            conv = _KW_TO_CONV.get(f.calling_convention or "", "cdecl")
        out[f.name] = (conv, f.is_static)
    return out


def _match(s: str, open_idx: int, opc: str, clc: str) -> int:
    """Index just past the balancer matching s[open_idx] == opc (or -1)."""
    depth = 0
    i = open_idx
    while i < len(s):
        c = s[i]
        if c == opc:
            depth += 1
        elif c == clc:
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    return -1


_ATTR_OR_BLANK = re.compile(r"^[ \t]*(#\[[^\n]*\])?[ \t]*$")

_HEAD_RE_TMPL = r'(?:pub\s+)?(?:unsafe\s+)?(?:extern\s+"(?P<abi>[^"]+)"\s+)?fn\s+{name}\s*\('


def _is_def_gap(gap: str) -> bool:
    """True if `gap` (the text between a fn's `)` and the next `{`) is a
    plausible *definition* tail — i.e. empty or just a return type.

    A `;` or `}` at bracket depth 0 means we ran past the end of a mere
    *declaration* (`extern "C" { fn Foo(x: u32); }`) and latched onto some
    unrelated later brace. Depth-tracking keeps array return types
    (`-> [u8; 8]`) from tripping the `;` check.
    """
    depth = 0
    for ch in gap:
        if ch in "[(<":
            depth += 1
        elif ch in "])>":
            depth -= 1
        elif ch in ";}" and depth <= 0:
            return False
    return True


def _find_defs(code: str, name: str):
    """Yield (span_start, body_end, params, ret_suffix, body, indent) for
    every definition of `fn name(` in `code`, absorbing preceding
    attribute/blank scaffolding lines into span_start."""
    head_re = re.compile(_HEAD_RE_TMPL.format(name=re.escape(name)))
    for hm in head_re.finditer(code):
        try:
            paren_open = code.index("(", hm.end() - 1)
        except ValueError:
            continue
        paren_end = _match(code, paren_open, "(", ")")
        if paren_end < 0:
            continue
        try:
            brace_open = code.index("{", paren_end)
        except ValueError:
            continue
        if not _is_def_gap(code[paren_end:brace_open]):
            continue  # a declaration, not a definition
        body_end = _match(code, brace_open, "{", "}")
        if body_end < 0:
            continue
        params = code[paren_open:paren_end]
        ret = code[paren_end:brace_open].strip()
        ret_suffix = f" {ret}" if ret else ""
        body = code[brace_open:body_end]

        line_start = code.rfind("\n", 0, hm.start()) + 1
        indent = code[line_start : hm.start()]
        span_start = line_start
        while span_start > 0:
            prev_nl = span_start - 1
            prev_start = code.rfind("\n", 0, prev_nl) + 1
            prev_line = code[prev_start:prev_nl]
            if _ATTR_OR_BLANK.match(prev_line):
                span_start = prev_start
            else:
                break
        yield (span_start, body_end, params, ret_suffix, body, indent)


# A macro-form parameter list must be exactly `$($arg:ident : $ty:ty),*` — the
# fragment shape BOTH `stdmethod!` definitions match. `mut x: T`, a bare `_: T`
# (the `_` token is not an `ident` fragment), and `self` all fail to match, and
# would be a confusing macro-expansion error rather than a clear one. When the
# params don't fit we fall back to the inline cfg dual, which is ABI-identical
# — a formatting fallback, not a correctness one.
_MACRO_PARAM_RE = re.compile(r"^[A-Za-z][A-Za-z0-9_]*\s*:\s*\S.*$", re.S)


def _params_fit_macro(params: str) -> bool:
    """True if `params` (including the outer parens) is a plain
    `name: Type, name: Type` list that `$($arg:ident : $ty:ty),*` will match."""
    inner = params.strip()[1:-1].strip()  # drop the parens
    if not inner:
        return True
    depth = 0
    parts, cur = [], []
    for ch in inner:  # split on top-level commas (types may contain commas)
        if ch in "<([":
            depth += 1
        elif ch in ">)]":
            depth -= 1
        if ch == "," and depth == 0:
            parts.append("".join(cur))
            cur = []
        else:
            cur.append(ch)
    parts.append("".join(cur))
    return all(_MACRO_PARAM_RE.match(p.strip()) for p in parts if p.strip())


def _macro_block(
    macro: str, name: str, params: str, ret_suffix: str, body: str, indent: str
) -> str:
    """Render a static+system fn as a `stdmethod! { fn NAME(..) -> R {..} }`.

    Single fn per invocation, bare `fn`, no attributes, and an explicit return
    type — the intersection of the two macro definitions this can target:
    `ole32-rust/src/winemacros.rs:99` (accepts attrs and a no-return arm) and
    `oleaut32-rust/src/oleaut.rs:10` (accepts NEITHER). A void fn therefore
    gets an explicit `-> ()`, which matches `$ret:ty` in both.
    """
    ret = ret_suffix if ret_suffix.strip() else " -> ()"
    return f"{indent}{macro}! {{\n{indent}fn {name}{params}{ret} {body}\n{indent}}}"


def _canonical_block(
    name: str,
    conv: str,
    params: str,
    ret_suffix: str,
    body: str,
    indent: str,
    is_static: bool,
    com_method_macro: str | None = None,
    concrete_stdcall_slots: bool = True,
) -> str:
    """Render the canonical definition for one function.

    `is_static` controls **visibility only** (`static` is C *linkage*): a
    static fn is file-private — no `#[no_mangle]`, no `pub`. The calling
    convention ALWAYS comes from the C, static or not.

    Shapes:
      * `fastcall` (any linkage) → `#[cfg]` dual: `fastcall` on x86,
        `extern "C"` elsewhere.
      * `system` + **static** → `#[cfg]` dual: `stdcall` on x86, `extern "C"`
        elsewhere. It must be spelled `stdcall` (not `system`) because rustc
        won't coerce a `"system" fn` into the `"stdcall" fn` slot our COM
        vtbl bindgen declares — see the module docstring.
        …unless `com_method_macro` is set, in which case the same thing is
        emitted as a `stdmethod!` invocation (the macro expands to exactly
        that dual, but writes the body once).
      * everything else → a single `extern "system"` / `extern "C"`.
    """
    if com_method_macro and conv == "system" and is_static and _params_fit_macro(params):
        return _macro_block(com_method_macro, name, params, ret_suffix, body, indent)

    # Visibility scaffolding: exported iff the C fn has external linkage.
    mangle = "" if is_static else f"{indent}#[no_mangle]\n"
    vis = "" if is_static else "pub "

    x86_abi = None
    if conv == "fastcall":
        x86_abi = "fastcall"
    elif conv == "thiscall":
        # Same shape as fastcall, and it has to be HERE rather than only in
        # postprocess.fix_fastcall_exports: this pass runs after that one and is the last
        # word on a fn the C defines, so without this arm `thiscall` fell through to the
        # `extern = "C"` default below and quietly flattened the dual back to one ABI.
        # msvcrt's cpp.c is the first file to reach it — 40-odd `__thiscall` C++ methods
        # that read `this` from ECX on i386 and from the first stack slot everywhere else.
        x86_abi = "thiscall"
    elif conv == "system" and is_static and concrete_stdcall_slots:
        # Only when the crate's substrate still spells slots per-arch. Once the island is
        # `extern "system"` throughout there is nothing for the twin to satisfy, and the
        # pair actively breaks assignment into a fn-pointer typedef the SAME generator
        # emits as `extern "system"` -- dde.cpp's `pfnCommandHandler` handler table, where
        # it was E0308 on amd64/arm64 (Rust types `extern "C"` distinctly even where the
        # ABI coincides) and redundant on i386.
        x86_abi = "stdcall"

    if x86_abi is not None:
        return (
            f'{indent}#[cfg(target_arch = "x86")]\n'
            f"{mangle}"
            f'{indent}{vis}unsafe extern "{x86_abi}" fn {name}{params}{ret_suffix} {body}\n\n'
            f'{indent}#[cfg(not(target_arch = "x86"))]\n'
            f"{mangle}"
            f'{indent}{vis}unsafe extern "C" fn {name}{params}{ret_suffix} {body}'
        )

    extern = "system" if conv == "system" else "C"
    return f'{mangle}{indent}{vis}unsafe extern "{extern}" fn {name}{params}{ret_suffix} {body}'


def enforce_abi_from_c(
    code: str,
    c_path: Path,
    *,
    com_method_macro: str | None = None,
    concrete_stdcall_slots: bool = True,
) -> str:
    """Rewrite every fn in `code` that `c_path` defines to its canonical
    ABI + visibility (see module docstring). Target-agnostic.

    Only touches functions the C source actually defines (so externs,
    helpers, and Rust-only glue are left alone). Idempotent.

    `com_method_macro` (e.g. `"stdmethod"`) renders `static` + system fns as
    `stdmethod! { fn NAME(..) -> R {..} }` instead of the inline cfg dual — the
    macro expands to exactly that dual, but writes the body once, and matches
    the style ole32-rust/oleaut32-rust are already hand-written in. Leave it
    `None` for targets with no such macro (kernel, HAL, drivers).
    """
    abi = build_abi_map_from_c(c_path)
    return _rewrite(
        code,
        abi,
        com_method_macro=com_method_macro,
        concrete_stdcall_slots=concrete_stdcall_slots,
    )


def _rewrite(
    code: str,
    abi: dict[str, tuple[str, bool]],
    *,
    com_method_macro: str | None = None,
    concrete_stdcall_slots: bool = True,
) -> str:
    for name, (conv, is_static) in abi.items():
        defs = list(_find_defs(code, name))
        if not defs:
            continue

        spans = sorted((d[0], d[1]) for d in defs)

        # Is this fn already wrapped in a `stdmethod!`-style invocation? (The
        # `stdmethod! {` line is not an attr/blank, so `_find_defs`' span_start
        # stops just inside it — a def's span_start therefore lands within the
        # macro's span.) Recomputed per name because `code` mutates below.
        macro_spans = _system_abi_macro_spans(code)
        wrapping = [(ms, me) for (ms, me) in macro_spans if any(ms <= s < me for s, _ in spans)]

        if wrapping:
            if conv == "system" and is_static:
                # Already canonical: the macro expands to the exact cfg dual we
                # would emit. Never fight a hand-written `stdmethod!` — this is
                # also what makes the macro mode idempotent, and it is why a
                # default-mode (`com_method_macro=None`) re-run does NOT churn
                # ole32's hand-fixed files back into inline duals.
                continue
            # A non-static (exported) fn has no business in the macro — it needs
            # `#[no_mangle]`, which `stdmethod!` does not emit. Swallow the whole
            # invocation so we don't leave an empty `stdmethod! { }` behind.
            spans = sorted(set(spans) | set(wrapping))
            spans = [(min(s for s, _ in spans), max(e for _, e in spans))]

        # Use the first def's params/ret/body/indent as canonical.
        params, ret_suffix, body, indent = defs[0][2], defs[0][3], defs[0][4], defs[0][5]
        block = _canonical_block(
            name,
            conv,
            params,
            ret_suffix,
            body,
            indent,
            is_static,
            com_method_macro,
            concrete_stdcall_slots,
        )

        # Idempotency: if the contiguous region spanning all this fn's defs
        # already equals the canonical block, skip (handles both the 1-def
        # and the 2-def cfg-dual cases).
        region_start, region_end = spans[0][0], spans[-1][1]
        if code[region_start:region_end].strip() == block.strip():
            continue

        first_start = spans[0][0]
        for s, e in sorted(spans, reverse=True):
            code = code[:s] + code[e:]
        code = code[:first_start] + block + code[first_start:]
    return code


# ---------------------------------------------------------------------------
# audit_abi — the detector (read-only counterpart of enforce_abi_from_c)
# ---------------------------------------------------------------------------

# Rust `extern "…"` string → normalized convention. `stdcall` and `system` are
# the same ABI (they only differ as rustc *types*), so both normalize to
# "system"; `C` and `cdecl` both normalize to "cdecl". Anything unrecognized
# (`win64`, `sysv64`, …) is carried through verbatim and reported as-is.
_RUST_ABI_NORM = {
    "system": "system",
    "stdcall": "system",
    "C": "cdecl",
    "cdecl": "cdecl",
    "fastcall": "fastcall",
    "thiscall": "thiscall",
}

# Macros whose expansion supplies the stdcall/system ABI to the fn(s) they
# wrap — a bare `fn Foo(..)` inside one is NOT Rust-ABI. `stdmethod!` is the
# hand-written COM-vtbl helper (`ole32-rust/src/winemacros.rs:99`); it expands
# to exactly the canonical cfg dual (`extern "stdcall"` on x86, `extern "C"`
# elsewhere), so an fn inside it is *correct*, not a violation.
_SYSTEM_ABI_MACROS = ("stdmethod",)

_CFG_X86 = re.compile(r'cfg\s*\(\s*target_arch\s*=\s*"x86"\s*\)')
_CFG_NOT_X86 = re.compile(r'cfg\s*\(\s*not\s*\(\s*target_arch\s*=\s*"x86"\s*\)\s*\)')

_C_CONV_MACRO = {
    "system": "WINAPI/NTAPI/STDMETHODCALLTYPE (__stdcall)",
    "fastcall": "FASTCALL",
    "cdecl": "cdecl (no convention macro)",
}


def _line_indexer(code: str):
    """offset -> 1-based line number."""
    starts = [0]
    for i, ch in enumerate(code):
        if ch == "\n":
            starts.append(i + 1)

    def line_of(off: int) -> int:
        return bisect.bisect_right(starts, off)

    return line_of


def _system_abi_macro_spans(code: str) -> list[tuple[int, int]]:
    """Byte spans of every `<macro>! { … }` invocation that confers the
    stdcall/system ABI on the fns inside it."""
    spans: list[tuple[int, int]] = []
    for mac in _SYSTEM_ABI_MACROS:
        for m in re.finditer(rf"\b{re.escape(mac)}\s*!\s*(?P<open>[{{(\[])", code):
            opc = m.group("open")
            clc = {"{": "}", "(": ")", "[": "]"}[opc]
            end = _match(code, m.start("open"), opc, clc)
            if end > 0:
                spans.append((m.start(), end))
    return spans


def _attr_prefix(code: str, head_start: int) -> str:
    """The contiguous run of attribute/blank lines directly above `head_start`
    (i.e. the `#[cfg(...)]` / `#[no_mangle]` scaffolding attached to this fn)."""
    line_start = code.rfind("\n", 0, head_start) + 1
    span_start = line_start
    while span_start > 0:
        prev_nl = span_start - 1
        prev_start = code.rfind("\n", 0, prev_nl) + 1
        if _ATTR_OR_BLANK.match(code[prev_start:prev_nl]):
            span_start = prev_start
        else:
            break
    return code[span_start:line_start]


class _DefSite:
    """One Rust definition of a fn: where it is, its ABI, and its cfg gate."""

    __slots__ = ("conv", "offset", "raw_abi", "x86_excluded", "x86_only")

    def __init__(self, offset: int, conv: str, raw_abi: str | None, attrs: str):
        self.offset = offset
        self.conv = conv
        self.raw_abi = raw_abi
        self.x86_excluded = bool(_CFG_NOT_X86.search(attrs))
        self.x86_only = bool(_CFG_X86.search(attrs)) and not self.x86_excluded

    def describe(self) -> str:
        return "the Rust ABI (no `extern`)" if self.raw_abi is None else f'extern "{self.raw_abi}"'


def _rust_def_sites(code: str, name: str) -> list[_DefSite]:
    """Every *definition* of `fn name` in `code`, with its ABI and cfg gate.

    A fn with no `extern "…"` is Rust-ABI (`conv == "rust"`) unless it sits
    inside a `stdmethod!`-style macro, which supplies the ABI for it.
    """
    macro_spans = _system_abi_macro_spans(code)
    out: list[_DefSite] = []
    head_re = re.compile(_HEAD_RE_TMPL.format(name=re.escape(name)))
    for hm in head_re.finditer(code):
        try:
            paren_open = code.index("(", hm.end() - 1)
        except ValueError:
            continue
        paren_end = _match(code, paren_open, "(", ")")
        if paren_end < 0:
            continue
        try:
            brace_open = code.index("{", paren_end)
        except ValueError:
            continue
        if not _is_def_gap(code[paren_end:brace_open]):
            continue  # a declaration, not a definition
        if _match(code, brace_open, "{", "}") < 0:
            continue

        raw = hm.group("abi")
        if raw is not None:
            conv = _RUST_ABI_NORM.get(raw, raw)
        elif any(s <= hm.start() < e for s, e in macro_spans):
            # Inside stdmethod! — the macro expands to the stdcall/C cfg dual.
            conv, raw = "system", "stdcall (via stdmethod!)"
        else:
            conv = "rust"
        out.append(_DefSite(hm.start(), conv, raw, _attr_prefix(code, hm.start())))
    return out


def _violations_for(name: str, c_conv: str, is_static: bool, sites: list[_DefSite]):
    """Yield (offset, reason) for each way `sites` disagrees with `c_conv`.

    The accepted shapes mirror `_canonical_block`:
      * system, non-static  → a single `extern "system"`; the stdcall/C cfg
        dual is accepted too (same ABI, also correct).
      * system, static      → the cfg dual (`stdcall` on x86 / `C` elsewhere),
        or `stdmethod!`, or a plain `extern "system"`/`"stdcall"`.
      * fastcall            → the cfg dual (`fastcall` on x86 / `C` elsewhere).
      * cdecl               → `extern "C"`.

    An `extern "C"` arm only counts toward a dual when it is actually gated
    `#[cfg(not(target_arch = "x86"))]` — an *ungated* `extern "C"` for a
    WINAPI/FASTCALL fn is the real i386 stack bug and is always reported.
    """
    x86_kind = "fastcall" if c_conv == "fastcall" else "system"
    has_x86_arm = any(s.conv == x86_kind for s in sites)

    for s in sites:
        if s.conv == c_conv:
            continue  # exact match — always fine
        if c_conv in ("system", "fastcall"):
            # The non-x86 half of a legitimate cfg dual.
            if s.conv == "cdecl" and s.x86_excluded and has_x86_arm:
                continue
            yield (
                s.offset,
                f"C declares `{'static ' if is_static else ''}"
                f"{_C_CONV_MACRO[c_conv]}`, Rust has {s.describe()}",
            )
        else:  # c_conv == "cdecl"
            yield (s.offset, f"C declares `{_C_CONV_MACRO[c_conv]}`, Rust has {s.describe()}")

    # NB: a *missing* x86 arm needs no separate message — with `has_x86_arm`
    # false, the `extern "C"` arm above is never accepted, so the dual's broken
    # half is already reported (and reporting it twice would only add noise).


def audit_abi(rust_code: str, c_path: Path, *, strict_cdecl: bool = False) -> list[str]:
    """Report calling-convention disagreements between a translated Rust file
    and its C source. Read-only; no rewriting. Usable standalone on an
    already-translated crate.

    A violation is any fn **defined in both** whose Rust ABI disagrees with the
    convention derived from the C. The dominant real-world case (~130 ole32
    methods, 2026-07) is a `static <ret> WINAPI Foo(..)` COM vtbl method that
    came out as a plain `unsafe fn` — the **Rust ABI**, which on i386 does not
    pop the callee's arguments and so corrupts the stack at the first call
    through the vtbl, while linking and running fine on amd64/arm64. That
    silence is what makes this the pipeline's most expensive defect class.

    Reported by default:
      * C says system/fastcall, Rust is Rust-ABI (no `extern`).
      * C says system/fastcall, Rust says `extern "C"` *without* the
        `#[cfg(not(target_arch = "x86"))]` gate that would make it the non-x86
        half of the canonical dual.
      * C says system/fastcall and the x86 arm is missing entirely.
      * C says cdecl, Rust declares a *different* convention (`extern "system"`
        — the same stack bug, mirrored).

    `strict_cdecl=True` additionally reports plain-cdecl C fns left at the Rust
    ABI. Those are only *latent* (they misbehave the moment the fn's address
    crosses into C — a vtbl, a callback table), so they're off by default to
    keep the crash-class report's signal-to-noise high.

    `stdmethod!`-wrapped fns (the ole32 hand-fix; `winemacros.rs:99`) are
    recognized as system — the macro expands to the canonical cfg dual.

    Returns human-readable, line-numbered strings sorted by line.
    """
    abi = build_abi_map_from_c(c_path)
    line_of = _line_indexer(rust_code)
    tag = c_path.name
    out: list[tuple[int, str]] = []

    for name, (c_conv, is_static) in sorted(abi.items()):
        sites = _rust_def_sites(rust_code, name)
        if not sites:
            continue  # not translated at all — a different detector's problem
        if c_conv == "cdecl" and not strict_cdecl:
            # Rust-ABI-vs-cdecl is latent, not a crash. Still report an
            # actively *wrong* convention (e.g. extern "system").
            sites = [s for s in sites if s.conv != "rust"]
            if not sites:
                continue
        for off, reason in _violations_for(name, c_conv, is_static, sites):
            out.append((line_of(off), f"{tag}: fn {name} — {reason}"))

    out.sort()
    return [f"line {ln}: {msg}" for ln, msg in out]
