#!/usr/bin/env python3
"""ld.lld front end that refuses a duplicate symbol resolved by link ORDER.

WHY THIS EXISTS
---------------
Every PE links with `--allow-multiple-definition`, because some duplicates are
irreducible: each Rust staticlib graph carries its own panic runtime, and Rust's
`compiler_builtins` overlaps clang's `libclang_rt.builtins`. But the flag is
global, so EVERY duplicate resolved silently, keeping whichever definition lld
met first. lld does report each one, as a warning nobody read. The first census
(2026-10-04) found the C glue beating the Rust translation in cdrom
(`DeviceEvtIoInCallerContext` — the Rust body had never run), fastfat and disk;
C stubs beating Rust in crtdll and ucrtbase; two Rust copies of one kernel32
export; and the Rust "can never run" `_Unwind_Resume` stub beating libunwind in a
C++ module that throws.

So the flag stays (it turns lld's duplicate ERROR into a warning) and this
wrapper reads the warnings and decides. Allowed:

  * both definitions are toolchain runtime (compiler_builtins, compiler-rt,
    libunwind) — interchangeable implementations of the same ABI;
  * the Rust panic runtime (`rust_begin_unwind`, `rust_eh_personality`, ...)
    defined by two of the module's own Rust staticlibs;
  * an import (import-library member or delay-load stub) on either side, as
    long as a LOCAL definition, if there is one, is the one kept. Two import
    libraries exporting one name (shell32 forwarding shlwapi's) is ordinary.

Everything else — C vs Rust, Rust vs Rust, C vs C, an import kept over a local
body, a local body kept over the toolchain runtime — fails the link. Fix it by
deleting the copy that should not exist; never by reordering the link.

Every CMake and cargo link reaches lld through here: clang's `--ld-path` names
`tools/link_guard/ld.lld` (gcc.cmake for CMake links, reactos-link for cargo).

  VIBEOS_LINK_GUARD=report   record findings, never fail (census mode)
  VIBEOS_LINK_GUARD_LOG=F    append one JSON line per duplicate to F
  VIBEOS_REAL_LLD=PATH       the real ld.lld (default: the pinned llvm-mingw one)
"""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

REPO = Path(os.path.abspath(__file__)).parent.parent
DEFAULT_LLD = REPO / ".build-tools" / "llvm-mingw" / "bin" / "ld.lld"

DUP_RE = re.compile(r"warning: duplicate symbol: (.+)$")
# The Rust panic runtime: every staticlib graph emits its own copy, and which
# one is kept does not matter (panic=abort; each just reports and halts).
# panic-common's _Unwind_Resume fallback is one body compiled into each graph.
RUST_RUNTIME_SYMBOL = re.compile(
    r"^_?(__rustc::)?(rust_begin_unwind|rust_eh_personality|rust_oom|__rust_[a-z_]+|_fltused"
    r"|vibeos_unwind_resume_fallback)$"
)
# compiler_builtins' `mem` feature defines these too; on COFF they are strong,
# so they collide with the module's own CRT copies (i386 kernel: libcntpr's
# memcpy.c). Both are correct implementations of the same contract.
MEM_SYMBOL = re.compile(r"^_?(memcpy|memmove|memset|memcmp|bcmp|strlen)$")
# Toolchain runtime providers, recognised by where the definition came from.
RUNTIME_ORIGIN = re.compile(
    r"compiler_builtins-|compiler-builtins|\.[/\\]lib/builtins[/\\]|libclang_rt\.builtins|libunwind\.a\("
)
# An import: a delay-load stub object, an import-library member named after
# its DLL, or the bare DLL name lld prints for a short-import symbol.
IMPORT_ORIGIN = re.compile(
    r"\.delay\.o\)?$|\([^()/]+\.(dll|exe|sys|drv|ocx|cpl|acm|efi|tsp)\)$|^[^()/\s]+\.(dll|exe|sys|drv|ocx|cpl|acm|efi|tsp)$",
    re.IGNORECASE,
)


@dataclass
class Dup:
    symbol: str
    kept: str
    dropped: str
    kept_kind: str
    dropped_kind: str
    verdict: str  # "ok:<why>" or "error:<why>"


def origin_kind(origin: str) -> str:
    if RUNTIME_ORIGIN.search(origin):
        return "runtime"
    if IMPORT_ORIGIN.search(origin.split(" | ")[-1].strip()):
        return "import"
    return "local"


def parse(stderr: str) -> list[tuple[str, str, str]]:
    """(symbol, kept origin, dropped origin) per lld duplicate warning.

    lld lists the definition it already had (and keeps) first, the newcomer
    second; a location may span two lines (source:line, then archive(member)).
    """
    out: list[tuple[str, str, str]] = []
    lines = stderr.splitlines()
    i = 0
    while i < len(lines):
        m = DUP_RE.search(lines[i])
        i += 1
        if not m:
            continue
        defs: list[list[str]] = []
        while i < len(lines) and lines[i].startswith(">>>"):
            body = lines[i][3:].strip()
            if body.startswith("defined at "):
                defs.append([body[len("defined at ") :]])
            elif defs:
                defs[-1].append(body)
            i += 1
        if len(defs) >= 2:
            out.append((m.group(1).strip(), " | ".join(defs[0]), " | ".join(defs[1])))
    return out


def judge(symbol: str, kept: str, dropped: str) -> Dup:
    k, d = origin_kind(kept), origin_kind(dropped)
    bare = symbol.removeprefix("__declspec(dllimport) ")

    def dup(verdict: str) -> Dup:
        return Dup(symbol, kept, dropped, k, d, verdict)

    if k == d == "runtime":
        return dup("ok:toolchain runtime twice")
    if RUST_RUNTIME_SYMBOL.match(bare) and "local" in (k, d):
        return dup("ok:Rust panic runtime")
    if "import" in (k, d):
        if k == "import" and d != "import":
            return dup(f"error:an import was kept over a {d} definition")
        return dup("ok:import")
    if k == "runtime":
        return dup("ok:toolchain runtime kept over a local copy")
    if d == "runtime" and MEM_SYMBOL.match(bare):
        return dup("ok:the CRT's own mem* over compiler_builtins' copy")
    if d == "runtime":
        return dup("error:a local copy was kept over the toolchain runtime")
    return dup("error:two local definitions, kept by link order")


def output_of(argv: list[str]) -> str:
    for i, a in enumerate(argv):
        if a == "-o" and i + 1 < len(argv):
            return argv[i + 1]
        if a.startswith("-o") and len(a) > 2:
            return a[2:]
    return "?"


def main(argv: list[str]) -> int:
    lld = os.environ.get("VIBEOS_REAL_LLD") or str(DEFAULT_LLD)
    if not os.access(lld, os.X_OK):
        sys.stderr.write(f"link-guard: real ld.lld not found at {lld} (set VIBEOS_REAL_LLD)\n")
        return 2
    proc = subprocess.run([lld, *argv], stderr=subprocess.PIPE, text=True, errors="replace")
    sys.stderr.write(proc.stderr)
    if proc.returncode != 0:
        return proc.returncode

    out = output_of(argv)
    dups = [judge(*t) for t in parse(proc.stderr)]
    log = os.environ.get("VIBEOS_LINK_GUARD_LOG")
    if log and dups:
        with open(log, "a", encoding="utf-8") as f:
            for d in dups:
                f.write(json.dumps({"output": out, **d.__dict__}) + "\n")

    bad = [d for d in dups if d.verdict.startswith("error:")]
    if not bad or os.environ.get("VIBEOS_LINK_GUARD") == "report":
        return 0
    sys.stderr.write(
        f"\nlink-guard: {out}: {len(bad)} duplicate symbol(s) resolved by link order "
        "(--allow-multiple-definition keeps the first). Delete the copy that should "
        "not exist; reordering the link is not a fix. See tools/link_guard.py.\n"
    )
    for d in bad:
        sys.stderr.write(
            f"  {d.symbol}: {d.verdict[len('error:') :]}\n"
            f"      kept    ({d.kept_kind}): {d.kept}\n"
            f"      dropped ({d.dropped_kind}): {d.dropped}\n"
        )
    try:
        os.unlink(out)
    except OSError:
        pass
    return 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
