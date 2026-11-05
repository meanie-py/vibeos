"""Put arrival times on a bench serial capture and find its silences."""

from __future__ import annotations

import argparse
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))
from tools.bench_infra_compat import call  # noqa: E402

BOOT = r"bootblock starting|FreeLoader v"


def load(run: pathlib.Path) -> list[tuple[float, str]]:
    return [tuple(row) for row in call("bench_serial_times_load", {"run": str(run)}, ROOT)]


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("run", type=pathlib.Path)
    ap.add_argument("--from", dest="start", default=BOOT)
    ap.add_argument("--gaps", type=float, default=10.0)
    ap.add_argument("--lines", action="store_true")
    args = ap.parse_args()
    result = call(
        "bench_serial_times_report",
        {"run": str(args.run), "from": args.start, "gaps": args.gaps, "lines": args.lines},
        ROOT,
    )
    print(result["output"], end="")


if __name__ == "__main__":
    main()
