#!/usr/bin/env python3
"""Fetch and verify the Pi Wi-Fi firmware using the native firmware mechanism."""

import argparse
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
if str(REPO) not in sys.path:
    sys.path.insert(0, str(REPO))
from tools import build_infra_compat, qemu_infra_compat  # noqa: E402

_metadata = build_infra_compat.call("payload_firmware_metadata", {}, REPO)
COMMIT = _metadata["commit"]
BASE = _metadata["base"]
FILES = [tuple(row) for row in _metadata["files"]]


def default_dest():
    return Path(build_infra_compat.call("payload_firmware_default_dest", {}, REPO))


def sha256(path):
    return build_infra_compat.call("payload_firmware_sha256", {"path": path}, REPO)


def download(url):
    try:
        return bytes(build_infra_compat.call("payload_firmware_download", {"url": url}, REPO))
    except build_infra_compat.NativeError as e:
        raise RuntimeError(str(e)) from e


def fetch(dest, get=download):
    args = {"dest": dest, "files": FILES}
    try:
        if get is download:
            return build_infra_compat.call("payload_firmware_fetch", args, REPO)
        with qemu_infra_compat.DedicatedSession(root=REPO) as session:
            return session.call(
                "payload_firmware_fetch",
                args,
                callbacks={"payload_download": lambda a: get(a["url"])},
            )
    except build_infra_compat.NativeError as e:
        raise RuntimeError(str(e)) from e


def main():
    ap = argparse.ArgumentParser(description=__doc__)
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
