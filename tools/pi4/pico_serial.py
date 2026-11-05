"""Run the native Pi serial capture service."""

import os
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]


def main() -> None:
    if len(sys.argv) not in (2, 3):
        raise SystemExit("usage: pico_serial.py <bench> [picousb-host]")
    tool = ROOT / "tools" / "bench-infra"
    os.execv(
        tool,
        [
            str(tool),
            "--root",
            str(ROOT),
            "serial",
            sys.argv[1],
            sys.argv[2] if len(sys.argv) == 3 else "picousb",
        ],
    )


if __name__ == "__main__":
    main()
