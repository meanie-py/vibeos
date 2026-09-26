"""Reclaim abandoned Windows QEMU staging while retaining live runs.

Callers hold portlease.launch_lock from inspection through deletion, the same
lock every harness launcher holds while registering PID-owned staging and
starting QEMU. Media copies may run outside it: live Linux owners protect their
staging directories even before the Windows QEMU process exists.
"""

from __future__ import annotations

import hashlib
import json
import os
import re
import shutil
import subprocess
import time
from pathlib import Path

from . import backend

# Only tool-owned names, including files left by the original flat staging layout.
_FLAT_MEDIA = {
    "debugcon.bin",
    "livecd.iso",
    "OVMF_CODE_4M.fd",
    "OVMF32_CODE.fd",
    "ovmf_vars.qcow2",
    "ovmf32_vars.qcow2",
    "scratch.qcow2",
    "virt.dtb",
    "QEMU_EFI.fd",
    "efivars.qcow2",
    "boot.img",
    "ramdisk.img",
}


def windows_qemu_commands() -> list[str]:
    """Fail closed if we cannot prove which Windows processes use staging."""
    script = (
        "$ErrorActionPreference='Stop';"
        "Get-CimInstance Win32_Process -Filter \"Name like 'qemu-system-%'\" | "
        "ForEach-Object {if (-not $_.CommandLine) {throw 'QEMU command line unavailable'};"
        "$_.CommandLine} | ConvertTo-Json -Compress"
    )
    result = subprocess.run(
        ["powershell.exe", "-NoProfile", "-Command", script],
        capture_output=True,
        text=True,
        check=True,
        timeout=30,
    )
    value = json.loads(result.stdout) if result.stdout.strip() else []
    commands = [value] if isinstance(value, str) else value
    if not isinstance(commands, list) or not all(isinstance(c, str) and c for c in commands):
        raise ValueError("Cannot verify live Windows QEMU command lines; cleanup refused")
    return commands


def _owner_alive(pid: int) -> bool:
    try:
        os.kill(pid, 0)
        return True
    except ProcessLookupError:
        return False
    except PermissionError:
        return True


def cleanup_locked(root: Path, *, dry_run: bool = False, min_age: float = 3600) -> dict:
    """Remove old owned paths only after Linux owners and Windows QEMUs are checked."""
    result: dict[str, list[str]] = {"removed": [], "eligible": [], "protected": []}
    if not root.exists():
        return result
    now = time.time()
    candidates = []
    for path in root.iterdir():
        if path.is_symlink():
            continue
        owner = re.match(r"^(\d+)-", path.name)
        known = (path.is_dir() and (owner or path.name.startswith("dbg-"))) or (
            path.is_file() and path.name in _FLAT_MEDIA
        )
        if not known:
            continue
        if now - path.stat().st_mtime < min_age or (owner and _owner_alive(int(owner[1]))):
            result["protected"].append(str(path))
            continue
        candidates.append(path)
    commands = [c.lower().replace("\\", "/") for c in windows_qemu_commands()] if candidates else []
    for path in candidates:
        windows = backend.win_path(path).lower()
        # Include the delimiter so run-1 cannot alias run-10. Flat media appears
        # as a complete filename; protecting a longer match is conservative.
        needle = windows + "/" if path.is_dir() else windows
        if any(needle in cmd for cmd in commands):
            result["protected"].append(str(path))
            continue
        result["eligible"].append(str(path))
        if not dry_run:
            if path.is_dir():
                shutil.rmtree(path)
            else:
                path.unlink()
            result["removed"].append(str(path))
    if not dry_run:
        # Cache pruning uses link counts: a staged run pins its immutable image.
        with backend.media_cache_lock():
            cache = root / "media-cache-v1"
            if cache.exists():
                for path in cache.iterdir():
                    if path.is_symlink() or not path.is_dir():
                        continue
                    owner = re.match(r"^\.populate-v2-(\d+)-", path.name)
                    if not owner and not path.name.startswith(".fill-"):
                        continue
                    if owner and _owner_alive(int(owner[1])):
                        result["protected"].append(str(path))
                        continue
                    # A live fill may finish between enumeration and stat.
                    try:
                        old = now - path.stat().st_mtime >= min_age
                    except FileNotFoundError:
                        continue
                    if old:
                        shutil.rmtree(path)
                        result["removed"].append(str(path))
                backend._prune_media_cache(cache, keep=cache)
    return result


def auto_cleanup_locked(root: Path) -> None:
    """Default on, at most once per five minutes across all worktrees."""
    controls = Path(os.environ.get("VIBEOS_HOSTLOCK_DIR", "/tmp/vibeos-qemu-locks/host"))
    controls.mkdir(parents=True, exist_ok=True)
    key = hashlib.sha256(str(root.resolve()).encode()).hexdigest()[:16]
    stamp = controls / f"windows-cleanup-{key}.stamp"
    if stamp.exists() and 0 <= time.time() - stamp.stat().st_mtime < 300:
        return
    result = cleanup_locked(root)
    stamp.touch()
    if result["removed"]:
        print(f"[QemuVM] cleaned {len(result['removed'])} abandoned Windows staging paths")
