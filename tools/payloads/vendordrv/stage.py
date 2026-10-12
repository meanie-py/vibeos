"""vendordrv's staging: every file in $VIBEOS_VENDOR_DRIVERS, as payload.yaml describes."""

from __future__ import annotations

import os
import sys
from pathlib import Path


def stage_entries(arch: str) -> list[dict]:
    src = os.environ.get("VIBEOS_VENDOR_DRIVERS")
    if not src:
        sys.exit("vendordrv: set VIBEOS_VENDOR_DRIVERS to the driver package directory (the .inf and its files)")
    root = Path(src).expanduser().resolve()
    files = sorted(p for p in root.iterdir() if p.is_file()) if root.is_dir() else []
    if not any(p.suffix.lower() == ".inf" for p in files):
        sys.exit(f"vendordrv: no .inf in {root}")
    out = []
    for p in files:
        if p.name.lower() == "ndisprobe.txt":
            out.append({"src": str(p), "dest": "bin/ndisprobe.txt"})
        else:
            # Lower-case names: the INF names its files in whatever case it likes, and the
            # ISO's names are matched case-insensitively anyway.
            out.append({"src": str(p), "dest": f"inf/{p.name.lower()}"})
            # Also pre-placed where the INF copies them. The live image's system volume
            # is read-only (the RAM disk is exported as a CD), so setupapi cannot copy
            # them there itself; it fails creating a temp file in X:\reactos\TEMP.
            if p.suffix.lower() == ".sys":
                out.append({"src": str(p), "dest": f"system32/drivers/{p.name.lower()}"})
            elif p.suffix.lower() == ".dll":
                out.append({"src": str(p), "dest": f"system32/{p.name.lower()}"})
    return out
