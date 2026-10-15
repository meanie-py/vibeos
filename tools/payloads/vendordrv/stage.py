"""vendordrv's staging: every file in $VIBEOS_VENDOR_DRIVERS, as payload.yaml describes."""

from __future__ import annotations

import os
import re
import sys
from pathlib import Path

# VIBEOS_VENDOR_PREFER=1: the vendor INF beats an inbox INF that claims the same device, as
# Windows' signature rule would make it (an unsigned driver ranks behind every signed one, and
# ours are all unsigned). setupapi has no such rule: it ranks by where the ID matched, then by
# DriverVer date (driver.c). So the staged copy, never the vendor's files, gets both:
#   * a model line `section,,ID` (Intel's style: the ID in the compatible-ID slot, rank
#     0x3000) becomes `section,ID` (hardware-ID slot, 0x2000, as our INFs write it);
#   * DriverVer moves to 2099, which settles the remaining tie.
PREFERRED_DATE = "12/31/2099"


def _preferred_inf(src: Path) -> Path:
    raw = src.read_bytes()
    utf16 = raw[:2] in (b"\xff\xfe", b"\xfe\xff")
    text = raw.decode("utf-16" if utf16 else "latin-1")
    new, n = re.subn(r"(?im)^(\s*DriverVer\s*=\s*)[0-9/]+", lambda m: m.group(1) + PREFERRED_DATE, text)
    if n == 0:
        sys.exit(f"vendordrv: VIBEOS_VENDOR_PREFER set but {src.name} has no DriverVer line")
    new = re.sub(r"(?m)^(\s*%[^%\r\n]+%\s*=\s*[^,;\r\n]+?)\s*,\s*,", r"\1,", new)
    out_dir = Path(os.environ.get("XDG_CACHE_HOME", Path.home() / ".cache")) / "vibeos" / "vendordrv"
    out_dir.mkdir(parents=True, exist_ok=True)
    out = out_dir / src.name.lower()
    out.write_bytes(new.encode("utf-16") if utf16 else new.encode("latin-1"))
    return out


def stage_entries(arch: str) -> list[dict]:
    src = os.environ.get("VIBEOS_VENDOR_DRIVERS")
    if not src:
        sys.exit("vendordrv: set VIBEOS_VENDOR_DRIVERS to the driver package directory (the .inf and its files)")
    root = Path(src).expanduser().resolve()
    files = sorted(p for p in root.iterdir() if p.is_file()) if root.is_dir() else []
    if not any(p.suffix.lower() == ".inf" for p in files):
        sys.exit(f"vendordrv: no .inf in {root}")
    prefer = os.environ.get("VIBEOS_VENDOR_PREFER") == "1"
    out = []
    for p in files:
        if p.name.lower() == "ndisprobe.txt":
            out.append({"src": str(p), "dest": "bin/ndisprobe.txt"})
        else:
            # Lower-case names: the INF names its files in whatever case it likes, and the
            # ISO's names are matched case-insensitively anyway.
            src_inf = _preferred_inf(p) if prefer and p.suffix.lower() == ".inf" else p
            out.append({"src": str(src_inf), "dest": f"inf/{p.name.lower()}"})
            # Also pre-placed where the INF copies them. The live image's system volume
            # is read-only (the RAM disk is exported as a CD), so setupapi cannot copy
            # them there itself; it fails creating a temp file in X:\reactos\TEMP.
            if p.suffix.lower() == ".sys":
                out.append({"src": str(p), "dest": f"system32/drivers/{p.name.lower()}"})
            elif p.suffix.lower() == ".dll":
                out.append({"src": str(p), "dest": f"system32/{p.name.lower()}"})
    # VIBEOS_WIFI_CONFIG: the bench network for a Wi-Fi driver's WDI layer (ssid=, psk=, ...),
    # read from \\SystemRoot\\bin\\wifi.txt. A path, never the values: they stay in that file.
    wifi = os.environ.get("VIBEOS_WIFI_CONFIG")
    if wifi:
        cfg = Path(wifi).expanduser()
        if not cfg.is_file():
            sys.exit(f"vendordrv: VIBEOS_WIFI_CONFIG={wifi} is not a file")
        out.append({"src": str(cfg), "dest": "bin/wifi.txt"})
    return out
