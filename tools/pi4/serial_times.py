"""Put arrival times on a bench serial capture and find its silences.

The capture writes serial.bin raw and, beside it, serial.times: one
'<byte offset> <unix ms>' line per chunk read. This joins them.

  serial_times.py <run dir> [--from REGEX] [--gaps SECONDS] [--lines]

--from starts at the LAST line matching REGEX (default: the last coreboot or
freeldr banner, i.e. the latest boot). --gaps lists every silence at least that
long, with the line before and after it. --lines prints every line with its time
since the start.
"""
from __future__ import annotations

import argparse
import bisect
import pathlib
import re
import sys

BOOT = r"bootblock starting|FreeLoader v"


def load(run: pathlib.Path) -> list[tuple[float, str]]:
    data = (run / "serial.bin").read_bytes()
    times = run / "serial.times"
    if not times.exists():
        sys.exit(f"serial_times: {times} missing (capture predates timing); restart the listener")
    offs: list[int] = []
    stamps: list[int] = []
    for row in times.read_text().split("\n"):
        parts = row.split()
        if len(parts) == 2:
            offs.append(int(parts[0]))
            stamps.append(int(parts[1]))
    if not offs:
        sys.exit("serial_times: serial.times is empty")
    out: list[tuple[float, str]] = []
    start = 0
    for m in re.finditer(rb"\n", data):
        line = data[start : m.start()].replace(b"\r", b"").decode("latin-1")
        # A line is stamped by the chunk holding its first byte.
        i = max(bisect.bisect_right(offs, start) - 1, 0)
        out.append((stamps[i] / 1000.0, line))
        start = m.end()
    return out


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("run", type=pathlib.Path)
    ap.add_argument("--from", dest="start", default=BOOT)
    ap.add_argument("--gaps", type=float, default=10.0)
    ap.add_argument("--lines", action="store_true")
    a = ap.parse_args()
    lines = load(a.run)
    rx = re.compile(a.start)
    first = max((i for i, (_, s) in enumerate(lines) if rx.search(s)), default=0)
    lines = lines[first:]
    if not lines:
        return
    t0 = lines[0][0]
    if a.lines:
        for t, s in lines:
            print(f"{t - t0:8.2f}  {s}")
        return
    print(f"{len(lines)} lines over {lines[-1][0] - t0:.1f} s from: {lines[0][1][:100]}")
    for (ta, sa), (tb, sb) in zip(lines, lines[1:]):
        if tb - ta >= a.gaps:
            print(f"\n{tb - ta:7.1f} s silent at +{ta - t0:.1f} s")
            print(f"   before: {sa[:150]}")
            print(f"   after:  {sb[:150]}")


if __name__ == "__main__":
    main()
