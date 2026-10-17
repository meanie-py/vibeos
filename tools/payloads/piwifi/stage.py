"""piwifi's staging: the CYW43455 firmware, fetched and verified at build time."""

from __future__ import annotations

import importlib.util
import os
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
GUEST_DIR = "system32/drivers/cyw43455"


def _fetcher():
    path = REPO / "tools" / "fetch_pi_wifi_firmware.py"
    spec = importlib.util.spec_from_file_location("_piwifi_fetch", path)
    if spec is None or spec.loader is None:
        sys.exit(f"piwifi: cannot load {path}")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def stage_entries(arch: str) -> list[dict]:
    fw = _fetcher()
    dest = fw.default_dest()
    try:
        for line in fw.fetch(dest):
            print(f"piwifi: {line}", file=sys.stderr)
    except (RuntimeError, OSError) as e:
        sys.exit(f"piwifi: the firmware fetch failed, so this image would have no Pi Wi-Fi: {e}")
    out = [{"src": str(dest / name), "dest": f"{GUEST_DIR}/{name}"} for _, name, _ in fw.FILES]
    # A path, never the values: the passphrase stays in that file.
    wifi = os.environ.get("VIBEOS_WIFI_CONFIG")
    if wifi:
        cfg = Path(wifi).expanduser()
        if not cfg.is_file():
            sys.exit(f"piwifi: VIBEOS_WIFI_CONFIG={wifi} is not a file")
        out.append({"src": str(cfg), "dest": "bin/wifi.txt"})
    return out
