#!/usr/bin/env python3
"""Boot-phase timing report from a timestamped smoke live log.

Feed it a `qemu/<arch>_smoke_live.log` produced with VIBEOS_SERIAL_TIMESTAMPS=1
(see tests/qemu/serial_client.py). It reports:
  1. first-hit time for each boot-phase marker (firmware -> shell ready)
  2. the largest inter-line gaps (where the boot silently stalled)

Usage:
  VIBEOS_SERIAL_TIMESTAMPS=1 ./vibeos-cli smoke --arch amd64
  python3 tools/boot_phase_report.py qemu/amd64-uefi_smoke_live.log
  python3 tools/boot_phase_report.py --gaps 30 <log> [<log> ...]
"""

from __future__ import annotations

import argparse
import re
import sys

STAMP_RE = re.compile(r"^\[\+\s*([0-9]+\.[0-9]+)\] ?(.*)$")

# Ordered boot-phase markers (substring match, first hit wins). Spans both the
# x86 UEFI livecd path and the arm64 ramdisk path; missing markers are skipped.
PHASE_MARKERS: list[tuple[str, str]] = [
    ("firmware->freeldr", "Starting freeldr"),
    ("freeldr MachInit", "[freeldr] MachInit done"),
    ("freeldr stack-switch", "Switching stack"),
    ("kernel entry (ACPI tables)", "ACPI v"),
    ("kernel: IoInitSystem done", "IoInitSystem returning TRUE"),
    ("SMSS start", "SMSS:"),
    ("csrss start", "CSRSS"),
    ("win32k ready", "win32k"),
    ("winlogon start", "WINLOGON"),
    ("explorer entry", "[EXPLORER] _tWinMain entered"),
    ("shell ready", "shell ready"),
]


def parse(path: str) -> list[tuple[float, str]]:
    lines: list[tuple[float, str]] = []
    with open(path, encoding="latin-1", errors="replace") as fh:
        for raw in fh:
            m = STAMP_RE.match(raw.rstrip("\n"))
            if m:
                lines.append((float(m.group(1)), m.group(2)))
    return lines


def report(path: str, n_gaps: int) -> None:
    lines = parse(path)
    print(f"=== {path} ({len(lines)} stamped lines)")
    if not lines:
        print("  no [+elapsed] stamps found — was VIBEOS_SERIAL_TIMESTAMPS=1 set?")
        return

    print("-- phase markers (first hit, elapsed s, delta from previous hit)")
    prev = 0.0
    for name, marker in PHASE_MARKERS:
        hit = next(((t, txt) for t, txt in lines if marker in txt), None)
        if hit is None:
            continue
        t = hit[0]
        print(f"  {t:9.3f}  (+{t - prev:8.3f})  {name}")
        prev = t

    total = lines[-1][0]
    print(f"  {total:9.3f}  end of log")

    print(f"-- top {n_gaps} inter-line gaps")
    gaps = []
    for i in range(1, len(lines)):
        dt = lines[i][0] - lines[i - 1][0]
        if dt >= 0.5:
            gaps.append((dt, lines[i - 1], lines[i]))
    gaps.sort(reverse=True, key=lambda g: g[0])
    for dt, (t0, before), (_, after) in gaps[:n_gaps]:
        print(f"  {dt:8.3f}s at +{t0:9.3f}  after: {before[:70]!r}")
        print(f"             {'':>10}  next : {after[:70]!r}")


def fldr_report(path: str, build_dir: str | None) -> None:
    """Per-file freeldr load timing from [FLDR] stamps; with --sizes <build-dir>
    joins file sizes to compute MB/s so I/O-bound (uniform slow MB/s) can be
    told apart from per-call overhead (small files disproportionately slow)."""
    import os

    lines = parse(path)
    loads = [
        (t, txt.split("[FLDR] load ", 1)[1].strip()) for t, txt in lines if "[FLDR] load " in txt
    ]
    if not loads:
        return
    sizes: dict[str, int] = {}
    if build_dir:
        for root, _dirs, files in os.walk(build_dir):
            for f in files:
                sizes.setdefault(f.lower(), os.path.getsize(os.path.join(root, f)))
    print(f"-- [FLDR] per-file loads ({len(loads)} files)")
    prev_t = loads[0][0]
    for i, (t, name) in enumerate(loads):
        # duration of THIS load = next stamp (load or phase) minus this one
        nxt = (
            loads[i + 1][0]
            if i + 1 < len(loads)
            else next((tt for tt, txt in lines if tt > t and "[FLDR] phase" in txt), t)
        )
        dur = max(0.0, nxt - t)
        base = name.rsplit("\\", 1)[-1].lower()
        sz = sizes.get(base)
        rate = f"{sz / dur / 1e6:6.2f} MB/s" if sz and dur > 0.05 else "      -    "
        szs = f"{sz / 1024:8.0f}K" if sz else "       ?"
        print(f"  {t:9.3f}  {dur:7.3f}s {szs} {rate}  {name}")
        prev_t = t
    _ = prev_t


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("logs", nargs="+")
    ap.add_argument("--gaps", type=int, default=15, help="how many gaps to show")
    ap.add_argument(
        "--sizes",
        default=None,
        metavar="BUILD_DIR",
        help="join [FLDR] load stamps with file sizes from this build dir",
    )
    args = ap.parse_args()
    for p in args.logs:
        report(p, args.gaps)
        fldr_report(p, args.sizes)
        print()
    return 0


if __name__ == "__main__":
    sys.exit(main())
