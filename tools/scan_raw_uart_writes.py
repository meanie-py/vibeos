#!/usr/bin/env python3
"""scan_raw_uart_writes.py — a debug byte stored straight into the PL011 data register.

THE INCIDENT
    On a physical Raspberry Pi 4 the kernel's serial output went quiet in Phase 1, and
    for a whole session the last readable line was taken to be where the kernel stopped.
    It was not. The raw bytes read

        [PHASE1] ExI[[[[[[[MM1] MiCreateMemoryEvent LowMem

    — seven trace lines each cut down to their first `[`. `ARM64_UART_TRACE`, which every
    ex-rust `uart_trace!` goes through, wrote each byte with a bare store to the data
    register. A PL011's transmit FIFO holds 16-32 bytes and drains at 115200 baud
    (~87 us per byte), so a CPU storing faster than that fills it within a line, and the
    hardware silently discards every byte after. QEMU's PL011 model drains instantly, so
    none of this ever showed there. 87 such stores lived in 13 files across six crates,
    each a private copy of "poke the UART".

THE RULE
    Every arm64 debug byte goes through `crate::arm64_dbg::uart_putc*` (inside
    ntoskrnl-rust) or the exported `Arm64UartPutChar` (from any other kernel crate),
    which wait on UARTFR.TXFF with a bounded spin.

WHAT COUNTS AS A FINDING
    A store through a pointer derived from `Arm64UartKernelBase`:

      * `*u = ...;` or `write_volatile(u, ...)` where `u` was bound by a `let` whose
        initialiser names `Arm64UartKernelBase` — the nearest preceding binding of that
        name wins, so an unrelated `u` elsewhere in the file is not confused with it;
      * `write_volatile(Arm64UartKernelBase ...` / `*(Arm64UartKernelBase as ...) =`.

    `ntoskrnl-rust/src/arm64_dbg.rs` is exempt: it is the one sanctioned writer.
    Comments and string literals are stripped first, so C kept in `//` comments for
    reference is not reported. Baseline is EMPTY.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.scan_octal_literals import strip_noise
from tools.source_walk import source_files

BASE = "Arm64UartKernelBase"
SANCTIONED = "sdk/lib/rust/ntoskrnl-rust/src/arm64_dbg.rs"
# How far back a store may be from the binding it uses. Longer than any debug helper in
# the tree; a store further away than this is reported as unbound, i.e. not at all.
WINDOW = 80

BIND = re.compile(r"\blet\s+(?:mut\s+)?(\w+)\s*(?::[^=;]+)?=(?!=)")
STORE = re.compile(r"(?:^|[;{}])\s*\*(\w+)\s*=(?!=)|\bwrite_volatile\(\s*(\w+)\s*,")
DIRECT = re.compile(rf"\bwrite_volatile\(\s*\(?\s*{BASE}\b|\*\s*\(\s*{BASE}\s+as\b[^)]*\)\s*=(?!=)")


def scan_text(text: str) -> list[tuple[int, str]]:
    lines = strip_noise(text).split("\n")
    # name -> line index of its most recent `let`, and whether that `let` was the UART.
    latest: dict[str, tuple[int, bool]] = {}
    out: list[tuple[int, str]] = []
    for i, line in enumerate(lines):
        for m in BIND.finditer(line):
            # The initialiser may continue onto the next line (`= core::ptr::read_volatile(`).
            init = line[m.end() :] + " " + (lines[i + 1] if i + 1 < len(lines) else "")
            latest[m.group(1)] = (i, BASE in init)
        if DIRECT.search(line):
            out.append((i + 1, BASE))
            continue
        for m in STORE.finditer(line):
            name = m.group(1) or m.group(2)
            bound = latest.get(name)
            if bound and bound[1] and i - bound[0] <= WINDOW:
                out.append((i + 1, name))
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--lint", action="store_true", help="findings only on stdout")
    ap.add_argument(
        "--root",
        action="append",
        default=None,
        help="directory to scan (repeatable); defaults to sdk/",
    )
    args = ap.parse_args()

    repo = Path(__file__).resolve().parent.parent
    roots = [repo / r for r in (args.root or ["sdk"])]
    findings = files = 0
    for root in roots:
        for path in sorted(source_files(root)):
            rel = path.relative_to(repo).as_posix()
            if rel == SANCTIONED:
                continue
            text = path.read_text(encoding="utf-8", errors="replace")
            if BASE not in text:
                continue
            files += 1
            for lineno, name in scan_text(text):
                print(
                    f"{rel}:{lineno}: RAW_UART_STORE through `{name}` — a bare store "
                    f"to the PL011 data register drops bytes once the FIFO is full; "
                    f"use Arm64UartPutChar / crate::arm64_dbg::uart_putc"
                )
                findings += 1
    print(f"scanned {files} files naming {BASE}, {findings} findings", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
