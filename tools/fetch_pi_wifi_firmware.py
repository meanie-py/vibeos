#!/usr/bin/env python3
"""Fetch the Pis' Wi-Fi firmware (CYW43455) at build time, never from this repository.

The chip runs a closed program Cypress/Infineon licenses for redistribution only in
narrow terms, so VibeOS neither commits nor hosts it, and the published images ship
without it (docs/engineering/pi-wifi-design.md). A build that wants Wi-Fi on a Pi
fetches the three files from Raspberry Pi's own firmware-nonfree repository, at a
pinned commit, and checks each against the SHA-256 the Pi 5's Raspberry Pi OS runs:

    python3 tools/fetch_pi_wifi_firmware.py [--dest DIR]

DIR defaults to $VIBEOS_PI_WIFI_FIRMWARE, else ~/.cache/vibeos/cyw43455. A file
already there with the right hash is kept; a wrong one is replaced. Any mismatch
after download fails: a changed upstream file is a decision for a person, not a
silent update.
"""

from __future__ import annotations

import argparse
import hashlib
import os
import sys
import tempfile
import urllib.request
from collections.abc import Callable
from pathlib import Path

# RPi-Distro/firmware-nonfree, "Update changelog for 1:20260519-1~bpo13+1+rpt1 release".
COMMIT = "3bab0f823f5b53150b76aab77093adef6655b920"
BASE = (
    f"https://raw.githubusercontent.com/RPi-Distro/firmware-nonfree/{COMMIT}/debian/added-firmware/"
)

# (upstream path, name the driver loads, sha256). The hashes are those of the files
# Raspberry Pi OS's firmware-brcm80211 1:20241210-1+rpt4 installs on a Pi 5, which the
# pinned commit's are byte for byte.
FILES = [
    (
        "cypress/cyfmac43455-sdio-standard.bin",
        "cyfmac43455-sdio.bin",
        "d608f866582519c0a28d86db43040f4f1b98dd1d153e72e9752586546b4a36c3",
    ),
    (
        "cypress/cyfmac43455-sdio.clm_blob",
        "cyfmac43455-sdio.clm_blob",
        "9823842cae9fb9a5dd1e5fb31f595516ec7deee341354bef30bb3026eee29cc1",
    ),
    (
        "brcm/brcmfmac43455-sdio.txt",
        "brcmfmac43455-sdio.txt",
        "ca709be81a78bdb6932936374f39943acbd7af07fae6151011127599a3ce9e3d",
    ),
]


def default_dest() -> Path:
    env = os.environ.get("VIBEOS_PI_WIFI_FIRMWARE")
    return Path(env) if env else Path.home() / ".cache" / "vibeos" / "cyw43455"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def download(url: str) -> bytes:
    with urllib.request.urlopen(url, timeout=60) as r:
        return r.read()


def fetch(dest: Path, get: Callable[[str], bytes] = download) -> list[str]:
    """Make `dest` hold the three files. Returns what it did, one line per file.
    Raises RuntimeError on a hash mismatch, leaving any good file already there alone."""
    dest.mkdir(parents=True, exist_ok=True)
    done = []
    for upstream, name, want in FILES:
        target = dest / name
        if target.is_file() and sha256(target) == want:
            done.append(f"{name}: present")
            continue
        data = get(BASE + upstream)
        got = hashlib.sha256(data).hexdigest()
        if got != want:
            raise RuntimeError(
                f"{upstream}: sha256 {got}, pinned {want}. Upstream changed under the pinned "
                f"commit, or the download was altered; nothing written for it."
            )
        with tempfile.NamedTemporaryFile(dir=dest, delete=False) as tmp:
            tmp.write(data)
        Path(tmp.name).replace(target)
        done.append(f"{name}: fetched ({len(data)} bytes)")
    return done


def main() -> int:
    ap = argparse.ArgumentParser(description="Fetch the Pis' Wi-Fi firmware at build time.")
    ap.add_argument("--dest", type=Path, default=None)
    args = ap.parse_args()
    dest = args.dest or default_dest()
    try:
        for line in fetch(dest):
            print(line)
    except (RuntimeError, OSError) as e:
        print(f"fetch_pi_wifi_firmware: {e}", file=sys.stderr)
        return 1
    print(f"CYW43455 firmware in {dest}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
