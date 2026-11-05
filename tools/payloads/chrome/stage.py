"""chrome's staging: a Chrome for Testing folder from the host, copied whole."""

from __future__ import annotations

import os
import sys
from pathlib import Path


def stage_entries(arch: str) -> list[dict]:
    root = os.environ.get("VIBEOS_CHROME_DIR")
    if not root:
        sys.exit("chrome: set VIBEOS_CHROME_DIR to a Chrome for Testing folder (the one holding chrome.exe)")
    src = Path(root).expanduser()
    if not (src / "chrome.exe").is_file():
        sys.exit(f"chrome: {src} has no chrome.exe")
    # Chrome finds its .pak, locales and the versioned manifest beside chrome.exe, so the
    # whole tree goes over with its layout intact.
    return [
        {"src": str(p), "dest": f"bin/chrome/{p.relative_to(src).as_posix()}"}
        for p in sorted(src.rglob("*"))
        if p.is_file()
    ]
