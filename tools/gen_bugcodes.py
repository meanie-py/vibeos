#!/usr/bin/env python3
"""
gen_bugcodes.py — generate `ntkrnl_sys::bugcodes` from the ReactOS bugcode list.

WHY THIS EXISTS (2026-07-24). Every translated `KeBugCheck(SYMBOL)` call came
back from the LLM as `KeBugCheck(0xNN)` with an INVENTED number: a sweep of the
kernel crates found 61 of ~115 translated stop codes wrong — internally
consistent, plausible-looking, and pointing at the wrong subsystem
(`INVALID_AFFINITY_SET` -> 0x48 CANCEL_STATE_IN_COMPLETED_IRP,
`MAXIMUM_WAIT_OBJECTS_EXCEEDED` -> 0x36, the whole ex/init.c
`*_INITIALIZATION_FAILED` family shifted). A wrong stop code costs hours: the
bugcheck fires, the screen names a subsystem nobody touched, and the hunt starts
in the wrong place.

The durable fix is the substrate-closure rule (AGENTS.md): constants never go
through the LLM. This generator emits every bugcheck code as a named Rust
constant so call sites read `bugcodes::INVALID_AFFINITY_SET`, and
`tools/scan_bugcheck_codes.py` keeps bare numbers out.

Source of truth: sdk/include/reactos/mc/bugcodes.mc (the message-compiler input
that generates bugcodes.h at build time). We cross-check every value against the
committed header copy and HARD FAIL on any disagreement rather than silently
preferring one — a mismatch means the committed copy has drifted and the
generated Rust would be wrong in a way nothing else would catch.

Usage:
    python3 tools/gen_bugcodes.py            # write the .rs
    python3 tools/gen_bugcodes.py --check    # exit 1 if the .rs is stale
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
MC = REPO / "sdk" / "include" / "reactos" / "mc" / "bugcodes.mc"
COMMITTED_H = REPO / "sdk" / "lib" / "rust" / "reactos-link" / "committed-headers" / "bugcodes.h"
OUT = REPO / "sdk" / "lib" / "rust" / "ntkrnl-sys" / "src" / "bugcodes.rs"


# Message-compiler severity bits (bits 31..30 of the composed message ID).
# Only Severity=Success entries are real stop codes; the file also carries the
# kernel's bugcheck-screen STRINGS (WINDOWS_NT_BANNER, BUGCHECK_MESSAGE_INTRO,
# ...) as Informational/Warning messages, and those must keep their severity
# bits or they no longer match the generated bugcodes.h.
SEVERITY = {"Success": 0, "Informational": 1, "Warning": 2, "Error": 3}


def parse_mc(path: Path) -> tuple[dict[str, int], set[str]]:
    """(name -> composed value, names that are real stop codes)."""
    codes: dict[str, int] = {}
    stop_codes: set[str] = set()
    msg_id: int | None = None
    severity = "Success"
    for line in path.read_text(errors="replace").splitlines():
        s = line.strip()
        m = re.match(r"^MessageId\s*=\s*(0x[0-9A-Fa-f]+|\d+)\s*$", s)
        if m:
            msg_id = int(m.group(1), 0)
            severity = "Success"
            continue
        m = re.match(r"^Severity\s*=\s*(\w+)\s*$", s)
        if m:
            severity = m.group(1)
            continue
        m = re.match(r"^SymbolicName\s*=\s*(\w+)\s*$", s)
        if m and msg_id is not None:
            codes[m.group(1)] = (SEVERITY.get(severity, 0) << 30) | msg_id
            if severity == "Success":
                stop_codes.add(m.group(1))
            msg_id = None
    return codes, stop_codes


def parse_header(path: Path) -> dict[str, int]:
    codes: dict[str, int] = {}
    for line in path.read_text(errors="replace").splitlines():
        m = re.match(r"^#define\s+(\w+)\s+\(ULONG\)\s*(0x[0-9A-Fa-f]+|\d+)", line.strip())
        if m:
            codes[m.group(1)] = int(m.group(2), 0)
    return codes


def render(codes: dict[str, int], stop_codes: set[str]) -> str:
    width = max(len(n) for n in codes)
    rows = []
    for name, val in sorted(codes.items(), key=lambda kv: (kv[1], kv[0])):
        tail = "" if name in stop_codes else "  // message string, NOT a stop code"
        rows.append(f"pub const {name:<{width}} : ULONG = 0x{val:X};{tail}")
    body = "\n".join(rows)
    n_stop, n_msg = len(stop_codes), len(codes) - len(stop_codes)
    return f"""//! Kernel bugcheck (stop) codes — GENERATED, do not edit by hand.
//!
//! Regenerate with `python3 tools/gen_bugcodes.py` after touching
//! `sdk/include/reactos/mc/bugcodes.mc`.
//!
//! Pass these to `KeBugCheck`/`KeBugCheckEx` instead of a numeric literal. A
//! hand-written number is how 61 translated stop codes ended up naming the
//! wrong subsystem (field manual §9.113); `tools/scan_bugcheck_codes.py` keeps
//! new ones out.
//!
//! Source: sdk/include/reactos/mc/bugcodes.mc ({n_stop} stop codes + {n_msg}
//! bugcheck-screen message strings, which keep their message-compiler severity
//! bits), cross-verified against
//! sdk/lib/rust/reactos-link/committed-headers/bugcodes.h.

#![allow(dead_code)]

use crate::ddk::ULONG;

{body}

/// Real-NT stop code that ReactOS's bugcodes.mc does not define. Used by our own
/// IPI/spinlock stall guards, which have no C original to inherit a code from.
/// Kept here so those call sites stay symbolic like every other one.
pub const CLOCK_WATCHDOG_TIMEOUT: ULONG = 0x101;
"""


def main() -> int:
    check = "--check" in sys.argv
    mc_codes, stop_codes = parse_mc(MC)
    h_codes = parse_header(COMMITTED_H)
    if not mc_codes:
        print(f"gen_bugcodes: parsed no codes from {MC}", file=sys.stderr)
        return 2

    # Fail loud on drift instead of silently picking a winner.
    problems = []
    for name, val in sorted(mc_codes.items()):
        if name in h_codes and h_codes[name] != val:
            problems.append(f"  {name}: bugcodes.mc=0x{val:X} committed-header=0x{h_codes[name]:X}")
    if problems:
        print("gen_bugcodes: SOURCES DISAGREE — refusing to generate:", file=sys.stderr)
        print("\n".join(problems), file=sys.stderr)
        return 2

    text = render(mc_codes, stop_codes)
    if check:
        current = OUT.read_text(errors="replace") if OUT.is_file() else ""
        if current != text:
            print(f"gen_bugcodes: {OUT.relative_to(REPO)} is STALE — run tools/gen_bugcodes.py")
            return 1
        print(f"gen_bugcodes: {OUT.relative_to(REPO)} up to date ({len(mc_codes)} codes)")
        return 0

    OUT.write_text(text)
    print(f"gen_bugcodes: wrote {OUT.relative_to(REPO)} ({len(mc_codes)} codes)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
