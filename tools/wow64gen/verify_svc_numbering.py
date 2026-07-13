#!/usr/bin/env python3
"""Prove the WoW64 win32k thunk table is indexed by the GUEST's syscall numbers.

Why this exists
---------------
The thunk table maps a service number the 32-bit guest puts in EAX to a host
function. So the table's INDICES must be the numbers the i386 user32/gdi32
syscall stubs actually use — not the host's.

`win32ss/w32ksvc32.h` gates one entry on the host's word size:

    SVC_(UserSetWindowLong, 4)
  #ifdef _WIN64
    SVC_(UserSetWindowLongPtr, 4)     <- 64-bit ONLY
  #endif
    SVC_(UserSetWindowPlacement, 2)

Until 2026-07-13 wow64gen parsed that file as flat text, so on a 64-bit host it
included the _WIN64-only entry and shifted EVERY service number after it by one.
195 syscalls were misrouted. Nothing faulted — the neighbours have plausible
signatures, so the calls just silently did the wrong thing. The guest's
NtUserShowWindow(0x1227) landed on the host's NtUserShowScrollBar; x86 Doom's
window therefore never appeared while it happily rendered thousands of frames
into a DC nobody could see.

This check compares the generated table against the REAL i386 stubs (disassembled
out of the built user32.dll/gdi32.dll — the ground truth the guest executes), so
the class cannot come back silently. Any divergence is a hard failure.

Usage:
    verify_svc_numbering.py [--gen <wow64_win32k_thunks.gen.c>] [--build-i386 <dir>]
"""

import argparse
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
OBJDUMP = ROOT / ".build-tools/llvm-mingw/bin/llvm-objdump"

# `{ "NtUserFoo", wow64_thunk_NtUserFoo, 2 }, /* [551] */`  and the MANUAL form
# `{ "NtUserBar", 0, 15 }, /* [342] MANUAL: ... */`
_TABLE_ENTRY = re.compile(r'\s*\{\s*"([^"]+)",\s*(?:\S+|0),\s*\d+\s*\},\s*/\* \[(\d+)\]')
# i386 syscall stub:  `<_ZwUserShowWindow@8>:` then `movl $0x1227, %eax`
_STUB_LABEL = re.compile(r"^[0-9a-f]+ <_?(?:Nt|Zw)((?:User|Gdi)\w+?)(?:@\d+)?>:")
_STUB_EAX = re.compile(r"movl\s+\$0x(1[0-9a-f]{3}),\s*%eax")


def load_table(gen: Path) -> dict:
    table = {}
    for line in gen.read_text(encoding="utf-8", errors="replace").splitlines():
        m = _TABLE_ENTRY.match(line)
        if m:
            table[int(m.group(2))] = m.group(1)
    return table


def load_guest_stubs(dlls) -> dict:
    stubs = {}
    for dll in dlls:
        out = subprocess.run(
            [str(OBJDUMP), "-d", str(dll)], capture_output=True, text=True
        ).stdout
        cur = None
        for line in out.splitlines():
            m = _STUB_LABEL.match(line)
            if m:
                cur = "Nt" + m.group(1)
                continue
            if cur:
                m2 = _STUB_EAX.search(line)
                if m2:
                    stubs[cur] = int(m2.group(1), 16) - 0x1000
                    cur = None
    return stubs


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--gen", default="build-arm64/dll/win32/wow64/wow64_win32k_thunks.gen.c")
    ap.add_argument("--build-i386", default="build-i386")
    args = ap.parse_args()

    gen = Path(args.gen)
    if not gen.is_absolute():
        gen = ROOT / gen
    b32 = Path(args.build_i386)
    if not b32.is_absolute():
        b32 = ROOT / b32

    dlls = [
        b32 / "win32ss/user/user32/user32.dll",
        b32 / "win32ss/gdi/gdi32/gdi32.dll",
    ]
    missing = [p for p in [gen, *dlls] if not p.exists()]
    if missing:
        # Nothing to check against — not a failure, just nothing built yet.
        print(f"verify_svc_numbering: SKIP (not built: {chr(44).join(p.name for p in missing)})", file=sys.stderr)
        return 0
    if not OBJDUMP.exists():
        print("verify_svc_numbering: SKIP (no llvm-objdump)", file=sys.stderr)
        return 0

    table = load_table(gen)
    stubs = load_guest_stubs(dlls)
    if not table or not stubs:
        print("verify_svc_numbering: SKIP (empty table or no stubs found)", file=sys.stderr)
        return 0

    top = max(table)
    bad = sorted(
        (idx, name, table.get(idx))
        for name, idx in stubs.items()
        if idx <= top and table.get(idx) != name
    )

    if bad:
        print(
            f"verify_svc_numbering: FAIL — {len(bad)} of {len(stubs)} guest syscalls "
            f"are misrouted by the WoW64 win32k thunk table.\n"
            f"The table is indexed by the HOST's numbering; it must use the GUEST's "
            f"(i386) numbering. See parse_sysfuncs() in wow64gen.py.\n",
            file=sys.stderr,
        )
        for idx, gname, hname in bad[:12]:
            print(
                f"   svc 0x{idx + 0x1000:04x} (idx {idx:4d}): guest calls {gname:34s} "
                f"-> host table dispatches {hname}",
                file=sys.stderr,
            )
        if len(bad) > 12:
            print(f"   ... and {len(bad) - 12} more", file=sys.stderr)
        return 1

    # Silent on success: the check harness treats every stdout line as a finding
    # to be gated against the baseline, so a clean run must print nothing there.
    print(
        f"verify_svc_numbering: OK — all {len([i for i in stubs.values() if i <= top])} "
        f"guest win32k syscalls map to the right host thunk",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
