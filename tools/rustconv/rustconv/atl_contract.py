"""The ATL substrate contract: what a C++ worker is told about `atl-rust`.

A C++ body that names `CAtlStringW` or `CComPtr` needs a fixed Rust API to map onto, or the
model invents one -- a wrapper type, a drop it forgets, a `Format` that takes a Rust string.
`--local-raii` solves that for ONE file-local type by pasting its whole lowering into every
prompt that mentions it; for a shared 1,300-line crate that would be most of each prompt,
billed per function. So this hands over the API only: the public signatures of each type
the body actually names, extracted from `sdk/lib/rust/atl-rust/src/lib.rs` (the source of
truth, never a copy here), plus the C++-spelling mapping rules that no signature conveys.

The assembled module reaches these through the crate prelude (`pub use atl_rust::...`), which
`crate_integrate` wires when the crate depends on atl-rust.
"""

from __future__ import annotations

import re
from functools import cache
from pathlib import Path

ATL_RUST = Path(__file__).resolve().parents[3] / "sdk" / "lib" / "rust" / "atl-rust"

# C++ spelling -> atl-rust type. Every TU the cluster serves is a UNICODE build, so the
# TCHAR spellings (`CString`, `CAtlString`) are the W string; a non-UNICODE C++ module
# would make that wrong, and `rust_type_for` refuses rather than guess (see `unicode`).
_STRING_SPELLINGS = ("CAtlStringW", "CStringW", "CAtlString", "CString")
_TCHAR_SPELLINGS = frozenset({"CAtlString", "CString"})
_DIRECT = ("CComPtr", "CHeapPtr", "CComHeapPtr", "CSimpleArray", "CSimpleMap")


@cache
def _api() -> dict[str, list[str]]:
    """atl-rust type -> its public method signatures, bodies and docs stripped."""
    text = (ATL_RUST / "src" / "lib.rs").read_text()
    out: dict[str, list[str]] = {}
    for m in re.finditer(r"^impl(?:<[^>]*>)?\s+(\w+)(?:<[^>]*>)?\s*\{", text, re.M):
        depth, i = 1, m.end()
        while depth and i < len(text):
            depth += {"{": 1, "}": -1}.get(text[i], 0)
            i += 1
        block = text[m.end() : i - 1]
        for sig in re.finditer(
            r"^((?:\s*#\[[^\]]*\]\n)*)\s*(pub (?:const )?(?:unsafe )?fn [^{;]+)", block, re.M
        ):
            if "doc(hidden)" in sig.group(1):
                continue  # the atl_format! macro's internals, not API
            out.setdefault(m.group(1), []).append(" ".join(sig.group(2).split()))
    return out


def types_used(body: str) -> list[str]:
    """The atl-rust types a C++ body names, in a stable order."""
    used = []
    if any(re.search(rf"\b{s}\b", body) for s in _STRING_SPELLINGS):
        used.append("CStringW")
    used += [t for t in _DIRECT if re.search(rf"\b{t}\s*<", body)]
    if "CComHeapPtr" in used and "CHeapPtr" not in used:
        used.append("CHeapPtr")  # the alias's methods live on CHeapPtr
    return used


def contract(body: str, *, unicode: bool = True) -> str:
    """The deterministic-contract block for one C++ worker, or "" if it names no ATL type."""
    used = types_used(body)
    if not used:
        return ""
    if not unicode and any(re.search(rf"\b{s}\b", body) for s in _TCHAR_SPELLINGS):
        raise ValueError(
            "a non-UNICODE C++ module names CString/CAtlString: its TCHAR string is the ANSI "
            "CStringA, which atl-rust does not provide. Add it rather than map onto CStringW."
        )
    api = _api()
    lines = [
        "\n## ATL types (deterministic — provided by the `atl-rust` crate, in scope)\n",
        "These C++ ATL types are already lowered and imported. Use exactly this API. Do NOT "
        "define, wrap or re-implement them, and do NOT call a destructor: `Drop` runs at "
        "scope exit on every path.\n",
    ]
    for t in used:
        lines.append(f"\n`{t}`:\n```rust\n" + "\n".join(api.get(t, [])) + "\n```\n")
    lines.append("\nMapping (mandatory):\n")
    if "CStringW" in used:
        lines += [
            "- `CStringW`, `CAtlStringW`, `CString`, `CAtlString` are all `CStringW`.\n",
            "- `CStringW s;` → `let mut s = CStringW::new();` and "
            "`CStringW s(psz);` / `= psz;` → `CStringW::from_ptr(psz)` (an "
            "`MAKEINTRESOURCEW(id)` argument loads the resource, as in ATL).\n",
            "- Passing `s` where a `LPCWSTR`/`PCWSTR` is expected (implicit conversion, "
            "`(LPCWSTR)s`, `s.GetString()`) → `s.GetString()`.\n",
            "- `s.LoadString(id)` → `s.LoadStringW(id)`.\n",
            "- `s.Format(fmt, a, b, ..)` → `atl_format!(s, fmt, a, b, ..)` (any arity; each "
            "argument is evaluated once). A resource-ID format "
            "`s.Format(IDS_X, ..)` → "
            "`atl_format!(s, CStringW::from_resource(IDS_X).GetString(), ..)`.\n",
            "- `s += psz` → `s += psz` (`AddAssign` for `*const WCHAR` and `*mut WCHAR`); "
            "`s += t` with a CStringW `t` → `s += t.GetString()`.\n",
        ]
    if "CComPtr" in used:
        lines.append(
            "- `CComPtr<I> p;` → `let mut p: CComPtr<I> = CComPtr::new();`; `&p` as an out "
            "parameter → `p.GetAddressOf()`; `p->Method(..)` → "
            "`((*(*p.Get()).lpVtbl).Method)(p.Get(), ..)`.\n"
        )
    return "".join(lines)
