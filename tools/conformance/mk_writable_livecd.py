#!/usr/bin/env python3
"""Derive livecd-rw.iso from an x86 livecd.iso: the same media, booted off a WRITABLE root.

Why: x86 boots the livecd with SystemRoot on the CD (`multi(0)disk(0)cdrom(0)`), which is
read-only CDFS, so every kmtest that creates a file under \\SystemRoot fails setup with
0xc0000022 and bails -- ZwCreateSection ran 9 assertions on x86 against 330 on arm64,
whose ramdisk is a writable FAT volume. x86 conformance was comparing a few percent of
those tests and calling the rest "passing".

How: the livecd's file tree goes into a FAT32 image, `ramdisk.img`, which is added to a
copy of the ISO; the boot entries' SystemPath becomes `ramdisk(0)\\reactos` with
`/RDPATH=\\ramdisk.img`. freeldr loads the image off the CD through the ordinary (proven)
CD boot path, exactly as upstream's hybridcd.ini LiveCD_RamDisk entry does, and the kernel
then runs from ramdisk.sys's FAT volume, as arm64 always does. Entry NAMES are untouched, so
vm.smp_selected_iso's DefaultOS poke selects UP/MP on this media like on the plain one.

The ISO is rebuilt only when livecd.iso changed (stamp beside it). Everything fails loud:
a missing tool or an entry this cannot rewrite is an error, never the plain livecd.
"""

from __future__ import annotations

import argparse
import fcntl
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

OUT_NAME = "livecd-rw.iso"
STAMP_NAME = "livecd-rw.json"
RDPATH = r"\ramdisk.img"
# Free space on the volume for what the tests write (kmtest's files are tiny; the
# registry hives and profiles grow a little at boot).
SLACK_MB = 64
# Boot-only parts of the ISO: the El Torito images and the loader's own files.
NOT_ON_RAMDISK = {"loader", "efi"}
# The entries this rewrites -- the ones vm.py's DefaultOS poke switches between.
ENTRIES = ("LiveCD_Debug", "LiveCD_SMP")


def _source_key(iso: Path) -> str:
    st = iso.stat()
    return hashlib.sha256(f"{iso.resolve()}:{st.st_size}:{st.st_mtime_ns}:v1".encode()).hexdigest()


def rewrite_ini(text: str) -> str:
    """Point ENTRIES at ramdisk(0). Line endings are preserved (freeldr.ini is CRLF)."""
    out, section, done = [], None, set()
    for line in text.splitlines(keepends=True):
        body = line.rstrip("\r\n")
        eol = line[len(body):]
        m = re.fullmatch(r"\[(.+)\]", body.strip())
        if m:
            section = m.group(1)
        elif section in ENTRIES and body.startswith("SystemPath="):
            if body != r"SystemPath=\reactos":
                raise SystemExit(
                    f"mk_writable_livecd: [{section}] {body!r} is not the expected \\reactos"
                )
            body = r"SystemPath=ramdisk(0)\reactos"
        elif section in ENTRIES and body.startswith("Options="):
            body = f"{body} /RDPATH={RDPATH}"
            done.add(section)
        out.append(body + eol)
    missing = set(ENTRIES) - done
    if missing:
        raise SystemExit(f"mk_writable_livecd: freeldr.ini has no Options= for {sorted(missing)}")
    return "".join(out)


def _run(cmd: list[str]) -> None:
    r = subprocess.run(cmd, capture_output=True, text=True)
    if r.returncode != 0:
        tail = r.stderr[-2000:]
        raise SystemExit(f"mk_writable_livecd: {cmd[0]} failed ({r.returncode}):\n{tail}")


def build(iso: Path, out: Path) -> None:
    for tool in ("xorriso", "mformat", "mcopy"):
        if shutil.which(tool) is None:
            raise SystemExit(f"mk_writable_livecd: {tool} not found")
    with tempfile.TemporaryDirectory(prefix="livecd-rw-", dir=out.parent) as td:
        tree, img, ini = Path(td) / "tree", Path(td) / "ramdisk.img", Path(td) / "freeldr.ini"
        _run(["xorriso", "-osirrox", "on", "-indev", str(iso), "-extract", "/", str(tree)])
        # Extracted files keep the ISO's read-only mode; nothing here needs that.
        _run(["chmod", "-R", "u+w", str(tree)])
        text = (tree / "freeldr.ini").read_bytes().decode("latin-1")
        ini.write_bytes(rewrite_ini(text).encode("latin-1"))
        payload = [p for p in sorted(tree.iterdir()) if p.name.lower() not in NOT_ON_RAMDISK]
        files = [f for p in payload for f in ([p] if p.is_file() else p.rglob("*"))]
        used = sum(f.stat().st_size for f in files if f.is_file())
        size_mb = used // (1 << 20) + used // (1 << 20) // 10 + SLACK_MB
        with open(img, "wb") as f:
            f.truncate(size_mb << 20)
        _run(["mformat", "-i", str(img), "-F", "-v", "ROSRAMDISK", "::"])
        _run(["mcopy", "-s", "-Q", "-i", str(img), *map(str, payload), "::/"])
        tmp_out = out.with_name(f"{out.name}.{os.getpid()}.partial")
        _run([
            "xorriso", "-indev", str(iso), "-outdev", str(tmp_out),
            "-boot_image", "any", "replay",
            "-map", str(img), "/ramdisk.img",
            "-map", str(ini), "/freeldr.ini",
            "-commit",
        ])
        tmp_out.replace(out)


def ensure(build_dir: Path) -> Path:
    """Return build_dir/livecd-rw.iso, (re)building it if livecd.iso changed."""
    iso = build_dir / "livecd.iso"
    if not iso.is_file():
        raise SystemExit(f"mk_writable_livecd: no {iso} (fix: ./vibeos-cli build --arch <arch>)")
    out, stamp = build_dir / OUT_NAME, build_dir / STAMP_NAME
    key = _source_key(iso)
    # Two boots of the same build (a 1G and a 2G run side by side) used to rebuild the ISO
    # concurrently and truncate each other's output; the second one waits here instead.
    with open(build_dir / "livecd-rw.lock", "w") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        try:
            if out.is_file() and json.loads(stamp.read_text()).get("source") == key:
                return out
        except (OSError, ValueError):
            pass
        build(iso, out)
        stamp.write_text(json.dumps({"source": key}))
        return out


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("build_dir", type=Path)
    print(ensure(ap.parse_args().build_dir))
    return 0


if __name__ == "__main__":
    sys.exit(main())
