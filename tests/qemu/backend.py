"""QEMU launch backends: `native` (Linux/Unix-socket/KVM) vs `windows-host`.

The `windows-host` backend runs the *Windows* QEMU.exe via WSL interop to escape WSL2
nested-virt contention (docs/engineering/windows-host-qemu-spike.md). This module holds the
pure, testable helpers for that backend — WSL↔Windows path translation, media staging, QEMU
arg rewriting (accel + staged paths), and uid-scoped process kill. The QemuVM wiring lives in
vm.py and only branches on the backend name; native behavior is left byte-for-byte unchanged.
"""

from __future__ import annotations

import re
import shutil
import subprocess
from pathlib import Path

# A token like `file=/home/.../livecd.iso` inside a -drive arg value.
_FILE_TOKEN = re.compile(r"(file=)(/[^,]+)")


def win_path(linux_path: str | Path) -> str:
    """Translate a WSL `/mnt/<drive>/rest` path to the Windows `<DRIVE>:/rest` form the
    Windows QEMU expects (forward slashes, which QEMU accepts on Windows). Raises ValueError
    for paths not under /mnt (those aren't reachable by a Windows process)."""
    p = str(linux_path)
    m = re.match(r"^/mnt/([a-zA-Z])/(.*)$", p)
    if not m:
        raise ValueError(f"path {p!r} is not under /mnt/<drive>; not reachable by Windows QEMU")
    return f"{m.group(1).upper()}:/{m.group(2)}"


def stage_file(src: Path, staging: Path) -> str:
    """Copy `src` into the Windows-visible `staging` dir; return its Windows path.
    Idempotent: a file already IN staging (e.g. the pre-created scratch disk) is not
    re-copied onto itself (which would be a shutil SameFileError)."""
    staging.mkdir(parents=True, exist_ok=True)
    dst = staging / src.name
    if src.resolve() != dst.resolve():
        shutil.copy2(src, dst)
    return win_path(dst)


def windows_qemu_bin(native_bin: str, qemu_dir: str, accel: str) -> str:
    """Resolve the Windows QEMU exe for a native binary name. WHPX is compiled only into
    `qemu-system-x86_64.exe`, so an x86 guest under whpx uses the 64-bit emulator (it boots
    32-bit guests fine); under tcg we keep the matching per-arch exe."""
    name = native_bin
    if accel == "whpx" and native_bin == "qemu-system-i386":
        name = "qemu-system-x86_64"
    return str(Path(qemu_dir) / f"{name}.exe")


def rewrite_accel(args: list[str], accel: str) -> list[str]:
    """Swap the KVM accelerator for the Windows one. `-accel kvm` -> `-accel <accel>`; for
    tcg drop the paired `-cpu host` (host-passthrough is a KVM/WHPX concept), for whpx keep
    it (WHPX wants host CPU). Operates on a copy."""
    out: list[str] = []
    i = 0
    while i < len(args):
        a = args[i]
        if a == "-accel" and i + 1 < len(args):
            out += ["-accel", "whpx,kernel-irqchip=off" if accel == "whpx" else accel]
            i += 2
            continue
        if a == "-cpu" and i + 1 < len(args) and accel != "whpx":
            i += 2  # drop `-cpu host` under tcg
            continue
        out.append(a)
        i += 1
    return out


def stage_media_paths(args: list[str], staging: Path) -> list[str]:
    """Stage every `file=/abs` referenced in the arg list into the Windows staging dir and
    rewrite the path to its Windows form. Generic over arches (catches ISO, OVMF code/vars,
    scratch). Non-/mnt absolute files are staged by copy; already-/mnt files are staged too
    so the Windows QEMU reads a stable local copy."""
    out: list[str] = []
    for a in args:
        m = _FILE_TOKEN.search(a)
        if not m:
            out.append(a)
            continue
        src = Path(m.group(2))
        win = stage_file(src, staging) if src.is_file() else win_path(src)
        out.append(a[: m.start(2)] + win + a[m.end(2) :])
    return out


def gateway_ip() -> str:
    """WSL's default-gateway IP = the Windows host as seen from WSL. A Windows-side
    service bound to `0.0.0.0` (e.g. a slirp hostfwd on the Windows QEMU) is reachable
    from WSL via THIS address, NOT `127.0.0.1` — WSL's loopback is the WSL VM's own,
    separate from the Windows host's. Mirror of the inbound rule (where the WSL listener
    must bind `0.0.0.0` for the Windows QEMU to dial it back)."""
    out = subprocess.run(
        ["ip", "route", "show", "default"], capture_output=True, text=True, check=True
    ).stdout
    m = re.search(r"default via (\d+\.\d+\.\d+\.\d+)", out)
    if not m:
        raise RuntimeError(f"could not parse WSL default gateway from {out!r}")
    return m.group(1)


def windows_free_tcp_port() -> int:
    """Pick a TCP port free ON THE WINDOWS HOST — the stack where a windows-host
    slirp hostfwd actually binds (`0.0.0.0:<port>`).

    The native `_free_tcp_port` binds on the WSL TCP stack, which is a DIFFERENT
    stack from Windows under NAT: a WSL-free port says nothing about Windows-side
    bindability, so QEMU.exe's hostfwd bind fails with `Could not set up host
    forwarding rule 'tcp:0.0.0.0:<port>'` (exit 1, before any guest code). A
    Windows `TcpListener` on port 0 lets the Windows OS assign a port free on its
    own stack, eliminating the cross-stack collision (a small bind-time TOCTOU
    window remains, as with any pick-then-bind scheme)."""
    ps = (
        "$l=[System.Net.Sockets.TcpListener]::new([System.Net.IPAddress]::Any,0);"
        "$l.Start();$p=$l.LocalEndpoint.Port;$l.Stop();$p"
    )
    out = subprocess.run(
        ["powershell.exe", "-NoProfile", "-Command", ps],
        capture_output=True,
        text=True,
        timeout=30,
        check=True,
    ).stdout
    digits = re.sub(r"\D", "", out)
    if not digits:
        raise RuntimeError(f"windows_free_tcp_port: no port in powershell output {out!r}")
    return int(digits)


def rewrite_netdev_for_windows(args: list[str], chardev_id: str) -> tuple[list[str], bool]:
    """Adapt the slirp `-netdev user,...` for the windows-host boundary. Two rewrites,
    mirror images of each other:

    - **hostfwd** (host->guest, the test dials in): `127.0.0.1` -> `0.0.0.0`. A
      127.0.0.1 hostfwd binds only the Windows loopback, invisible from WSL; bound to
      the wildcard it's reachable via `gateway_ip()` (what `QemuVM.net_host` returns).
    - **guestfwd** (guest->host echo): `-cmd:cat` runs a *Unix* program on the QEMU host
      — which here is Windows (no `cat`). Redirect it to `-chardev:<id>`; the caller wires
      that chardev to a harness-run echo server. Returns (new_args, needs_echo) where
      needs_echo signals the caller to start the echo server + add the chardev arg.

    No-op on a netdev without hostfwd/guestfwd; native never calls this (native keeps
    `127.0.0.1` + `cmd:cat` unchanged)."""
    out: list[str] = []
    needs_echo = False
    for a in args:
        if a.startswith("user,") and ("hostfwd=" in a or "guestfwd=" in a):
            a = a.replace("hostfwd=tcp:127.0.0.1:", "hostfwd=tcp:0.0.0.0:")
            new_a = re.sub(r"(guestfwd=tcp:[\d.]+:\d+)-cmd:cat", rf"\1-chardev:{chardev_id}", a)
            if new_a != a:
                needs_echo = True
            a = new_a
        out.append(a)
    return out, needs_echo


def kill_by_uid(uid: str) -> None:
    """Kill the Windows QEMU process(es) whose command line carries our unique `-name <uid>`
    tag — precise, so a concurrent agent's QEMU is never touched. Best-effort."""
    ps = (
        "Get-CimInstance Win32_Process -Filter \"Name like 'qemu-system-%'\" "
        f"| Where-Object {{ $_.CommandLine -like '*{uid}*' }} "
        "| ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }"
    )
    try:
        subprocess.run(
            ["powershell.exe", "-NoProfile", "-Command", ps],
            capture_output=True,
            timeout=30,
        )
    except (OSError, subprocess.SubprocessError):
        pass
