"""hostapps' staging: the folders apps.txt names, copied from the host's Program Files."""

from __future__ import annotations

import os
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
DEFAULT_ROOT = "/mnt/c/Program Files"


def stage_entries(arch: str) -> list[dict]:
    root = Path(os.environ.get("VIBEOS_HOST_APPS_ROOT", DEFAULT_ROOT)).expanduser()
    if not root.is_dir():
        sys.exit(f"hostapps: {root} is not a directory (set VIBEOS_HOST_APPS_ROOT)")
    out = []
    for line in (HERE / "apps.txt").read_text().splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        guest, host, globs = line.split("|")
        src_dir = root / host
        for pat in globs.split(","):
            hits = sorted(p for p in src_dir.glob(pat.strip()) if p.is_file())
            if not hits:
                sys.exit(f"hostapps: {src_dir}/{pat.strip()} matched nothing (apps.txt: {guest})")
            out += [{"src": str(p), "dest": f"bin/{guest}/{p.name}"} for p in hits]
    return out
