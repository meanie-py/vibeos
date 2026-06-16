"""Authoritative win32k calling-convention map — deterministic, LLM-free.

Calling convention is the project's historical #1 i386 crash cause
(NTAPI=stdcall `_X@N`, FASTCALL=`@X@N`, cdecl=`_X`; amd64/arm64 are
uniformly cdecl — the variance is x86-only). It must NOT be left to the
LLM to infer from `APIENTRY`/`FASTCALL`/`NTAPI` macros. We already have
the ground truth: `win32k-sys/src/bindgen_i386.rs` encodes every win32k
function's exact i386 convention (bindgen cross-compiled it from the
real headers' decoration), and `win32k.spec` is definitive for exports.

This module fuses both into `name -> i386_convention` where convention
∈ {"stdcall","fastcall","cdecl"}. amd64/arm64 are always cdecl for
win32k, so only the i386 deviation is recorded; callers emit a dual-cfg
block (`#[cfg(target_arch="x86")] extern "<conv>"` / else `extern "C"`).

Consumed by `postprocess.enforce_win32k_abi` (rewrites the LLM's
`extern "..."` to ground truth) and `verify_abi` (the post-pass gate).
"""

from __future__ import annotations

from functools import lru_cache

from .config import WIN32K_SYS_DIR, WIN32SS_DIR
from .verify_abi import parse_bindgen_conventions, parse_spec

# bindgen extern-string → our normalized convention.
_NORM = {
    "C": "cdecl",
    "cdecl": "cdecl",
    "system": "stdcall",  # extern "system" == stdcall on i386 (NTAPI)
    "stdcall": "stdcall",
    "fastcall": "fastcall",
}


@lru_cache(maxsize=1)
def build_win32k_abi_map() -> dict[str, str]:
    """name → i386 calling convention ("stdcall"|"fastcall"|"cdecl").

    Priority: win32k-sys bindgen_i386 (the rich superset — ~2,860 fns
    incl the 411 FASTCALL ones), then `win32k.spec` overlaid on top
    (definitive for the names it lists — the exported syscall/Eng/GDI
    stubs). Only non-cdecl entries matter to the enforcer (cdecl is the
    implicit default and what the LLM usually emits anyway), but we keep
    cdecl entries too so the enforcer can *correct* a wrong stdcall→cdecl.
    """
    m: dict[str, str] = {}

    bindgen_i386 = WIN32K_SYS_DIR / "bindgen_i386.rs"
    for name, conv in parse_bindgen_conventions(bindgen_i386).items():
        norm = _NORM.get(conv)
        if norm:
            m[name] = norm

    # win32k.spec is definitive for exports — overlay (spec wins).
    spec = parse_spec(WIN32SS_DIR / "win32k.spec")
    for name, conv in spec.items():
        norm = _NORM.get(conv)
        if norm:
            m[name] = norm

    return m


def i386_convention(name: str) -> str | None:
    """i386 convention for `name`, or None if unknown (leave LLM output)."""
    return build_win32k_abi_map().get(name)
