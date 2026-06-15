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

  * **NTAPI / WINAPI / stdcall** → `#[no_mangle] pub unsafe extern "system"`.
    `extern "system"` is stdcall on i386 and cdecl on amd64/arm64 *by
    definition*, so a single declaration is correct on all three arches —
    no `#[cfg]` split needed (this is why it's simpler than the win32k
    fastcall path).
  * **FASTCALL** → dual-cfg: `extern "fastcall"` on x86, `extern "C"`
    elsewhere (there is no `extern "system"` equivalent that means
    fastcall, and fastcall only deviates on i386).
  * **cdecl / no qualifier** → `#[no_mangle] pub unsafe extern "C"`.
  * **static** in C → file-private: strip any `#[no_mangle]` / `pub` /
    `extern "…"`, leave a plain `unsafe fn` (the symbol must NOT be
    exported; it has internal linkage in C).

Idempotent on its own output. Consumed by `postprocess` (optionally) and
the `abi-enforce` CLI verb.
"""

from __future__ import annotations

import re
from functools import lru_cache
from pathlib import Path

from .scanner import scan_file

# C calling-convention macro → normalized convention.
_CONV_NORM = {
    "NTAPI": "system",
    "WINAPI": "system",
    "STDCALL": "system",
    "__stdcall": "system",
    "FASTCALL": "fastcall",
    "__fastcall": "fastcall",
    "CDECL": "cdecl",
    "__cdecl": "cdecl",
}


# Convention macros as they appear in the C text. Order = lookup priority.
_CONV_KEYWORDS = ("NTAPI", "WINAPI", "APIENTRY", "STDMETHODCALLTYPE", "FASTCALL", "CDECL")
_KW_TO_CONV = {
    "NTAPI": "system",
    "WINAPI": "system",
    "APIENTRY": "system",
    "STDMETHODCALLTYPE": "system",
    "STDCALL": "system",
    "FASTCALL": "fastcall",
    "CDECL": "cdecl",
}


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
        # Skip whitespace; a definition has '{' next (a call/decl has ';' or ',').
        j = i + 1
        while j < len(c_text) and c_text[j] in " \t\r\n":
            j += 1
        if j >= len(c_text) or c_text[j] != "{":
            continue

        # Bound the qualifier window: back to prev '}' / ';' / blank line.
        win_start = m.start()
        lo = max(0, win_start - 400)
        seg = c_text[lo:win_start]
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
            conv = _CONV_NORM.get(f.calling_convention or "", "cdecl")
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


def _find_defs(code: str, name: str):
    """Yield (span_start, body_end, params, ret_suffix, body, indent) for
    every definition of `fn name(` in `code`, absorbing preceding
    attribute/blank scaffolding lines into span_start."""
    head_re = re.compile(
        rf'(?:pub\s+)?(?:unsafe\s+)?(?:extern\s+"[^"]+"\s+)?fn\s+{re.escape(name)}\s*\(',
    )
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


def _canonical_block(
    name: str, conv: str, params: str, ret_suffix: str, body: str, indent: str, is_static: bool
) -> str:
    """Render the canonical definition for one function."""
    if is_static:
        # Internal linkage in C — keep file-private, no export, no extern.
        return f"{indent}unsafe fn {name}{params}{ret_suffix} {body}"
    if conv == "fastcall":
        return (
            f'{indent}#[cfg(target_arch = "x86")]\n'
            f"{indent}#[no_mangle]\n"
            f'{indent}pub unsafe extern "fastcall" fn {name}{params}{ret_suffix} {body}\n\n'
            f'{indent}#[cfg(not(target_arch = "x86"))]\n'
            f"{indent}#[no_mangle]\n"
            f'{indent}pub unsafe extern "C" fn {name}{params}{ret_suffix} {body}'
        )
    extern = "system" if conv == "system" else "C"
    return (
        f"{indent}#[no_mangle]\n"
        f'{indent}pub unsafe extern "{extern}" fn {name}{params}{ret_suffix} {body}'
    )


def enforce_abi_from_c(code: str, c_path: Path) -> str:
    """Rewrite every fn in `code` that `c_path` defines to its canonical
    ABI + visibility (see module docstring). Target-agnostic.

    Only touches functions the C source actually defines (so externs,
    helpers, and Rust-only glue are left alone). Idempotent.
    """
    abi = build_abi_map_from_c(c_path)
    return _rewrite(code, abi)


def _rewrite(code: str, abi: dict[str, tuple[str, bool]]) -> str:
    for name, (conv, is_static) in abi.items():
        defs = list(_find_defs(code, name))
        if not defs:
            continue
        # Use the first def's params/ret/body/indent as canonical.
        params, ret_suffix, body, indent = defs[0][2], defs[0][3], defs[0][4], defs[0][5]
        block = _canonical_block(name, conv, params, ret_suffix, body, indent, is_static)

        # Idempotency: if the contiguous region spanning all this fn's defs
        # already equals the canonical block, skip (handles both the 1-def
        # and the 2-def fastcall dual-cfg case).
        spans = [(d[0], d[1]) for d in defs]
        spans.sort()
        region_start, region_end = spans[0][0], spans[-1][1]
        if code[region_start:region_end].strip() == block.strip():
            continue

        first_start = spans[0][0]
        for s, e in sorted(spans, reverse=True):
            code = code[:s] + code[e:]
        code = code[:first_start] + block + code[first_start:]
    return code
