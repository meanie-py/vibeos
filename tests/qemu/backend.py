"""QEMU launch backends: `native` (Linux/Unix-socket/KVM) vs `windows-host`.

The `windows-host` backend runs the *Windows* QEMU.exe via WSL interop to escape WSL2
nested-virt contention (docs/engineering/windows-host-qemu-whpx-spike.md). This module holds the
pure, testable helpers for that backend — WSL↔Windows path translation, media staging, QEMU
arg rewriting (accel + staged paths), and uid-scoped process kill. The QemuVM wiring lives in
vm.py and only branches on the backend name; native behavior is left byte-for-byte unchanged.
"""

from __future__ import annotations

import fcntl
import hashlib
import os
import re
import shutil
import subprocess
import tempfile
from contextlib import contextmanager
from pathlib import Path

# A token like `file=/home/.../livecd.iso` inside a -drive arg value.
_FILE_TOKEN = re.compile(r"(file=)(/[^,]+)")
_COPY_BUFFER_SIZE = 16 * 1024
_MEDIA_CACHE_LIMIT = 2 * 1024**3


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
        # DrvFS/9p sendfile failed with ENOMEM under concurrent gate load even
        # with ample reclaimable RAM (order-4 kernel allocation). Even buffered
        # 256 KiB writes reproduced it. Keep writes small; cache read-only media
        # to avoid paying for this transfer on every boot.
        before = src.stat()
        fd, temporary = tempfile.mkstemp(prefix=f".{src.name}-", dir=staging)
        temp = Path(temporary)
        try:
            with (
                os.fdopen(fd, "wb", buffering=_COPY_BUFFER_SIZE) as output,
                src.open("rb") as source,
            ):
                shutil.copyfileobj(source, output, length=_COPY_BUFFER_SIZE)
            after = src.stat()

            def identity(s):
                return s.st_dev, s.st_ino, s.st_size, s.st_mtime_ns, s.st_ctime_ns

            if identity(before) != identity(after) or temp.stat().st_size != before.st_size:
                raise RuntimeError(f"QEMU staging input changed while copying: {src}")
            shutil.copystat(src, temp)
            os.replace(temp, dst)
        finally:
            temp.unlink(missing_ok=True)
    return win_path(dst)


@contextmanager
def media_cache_lock(name: str = "cache.lock"):
    """Linux flock shared across worktrees; never unlink a lock's inode."""
    locks = Path(os.environ.get("VIBEOS_HOSTLOCK_DIR", "/tmp/vibeos-qemu-locks/host")) / "media"
    locks.mkdir(parents=True, exist_ok=True)
    with (locks / name).open("a") as lock:
        fcntl.flock(lock.fileno(), fcntl.LOCK_EX)
        yield


def stage_readonly_file(src: Path, staging: Path) -> str:
    """Hash every input; fill different images concurrently, then publish and pin.

    The global lock still coordinates publication, hard links and pruning with
    older launchers. Slow copies hold only a per-entry lock. A warm hit never
    waits for that lock, and a second cold caller rechecks after acquiring it.
    """
    before = src.stat()
    digest = hashlib.sha256()
    with src.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)

    def identity(st):
        return st.st_dev, st.st_ino, st.st_size, st.st_mtime_ns, st.st_ctime_ns

    if identity(before) != identity(src.stat()):
        raise RuntimeError(f"QEMU staging input changed while hashing: {src}")
    cache = staging.parent / "media-cache-v1"
    entry = cache / f"{digest.hexdigest()}-{src.name}"
    pinned = staging / src.name

    def pin_if_present() -> bool:
        # Caller holds the global lock through lookup, link and pruning. In
        # particular an entry cannot disappear between the existence test and pin.
        if not entry.exists():
            return False
        if entry.stat().st_size != before.st_size:
            raise RuntimeError(f"QEMU read-only media cache is damaged: {entry}")
        if identity(src.stat()) != identity(before):
            raise RuntimeError(f"QEMU staging input changed before pinning: {src}")
        staging.mkdir(parents=True, exist_ok=True)
        if pinned.exists():
            if not pinned.samefile(entry):
                raise RuntimeError(f"Conflicting read-only staging basename: {pinned}")
        else:
            os.link(entry, pinned)
        entry.touch()
        _prune_media_cache(cache, keep=entry)
        return True

    with media_cache_lock():
        cache.mkdir(parents=True, exist_ok=True)
        if pin_if_present():
            return win_path(pinned)

    key = hashlib.sha256(str(entry.resolve()).encode()).hexdigest()
    with media_cache_lock(f"fill-{key}.lock"):
        with media_cache_lock():
            if pin_if_present():
                return win_path(pinned)
        # Older cleanup knows .fill-* but does not know that copies now run
        # outside the global lock. A distinct prefix keeps it from deleting our
        # in-progress files. New cleanup also checks the Linux owner's lifetime.
        with tempfile.TemporaryDirectory(prefix=f".populate-v2-{os.getpid()}-", dir=cache) as tmp:
            private = Path(tmp)
            stage_file(src, private)
            with media_cache_lock():
                if identity(src.stat()) != identity(before):
                    raise RuntimeError(f"QEMU staging input changed before publication: {src}")
                # An older launcher may have filled this entry while we copied.
                # Never replace an existing immutable inode (runs may pin it).
                if not entry.exists():
                    os.replace(private / src.name, entry)
                if not pin_if_present():
                    raise RuntimeError(f"QEMU cache entry disappeared under lock: {entry}")
    return win_path(pinned)


def _prune_media_cache(cache: Path, *, keep: Path) -> None:
    """Bound cached media to 2 GiB plus entries pinned by run directories."""
    entries = []
    for path in cache.iterdir():
        if re.fullmatch(r"[0-9a-f]{64}-.+", path.name):
            try:
                st = path.stat()
                entries.append((st.st_mtime_ns, st.st_size, st.st_nlink, path))
            except FileNotFoundError:
                continue  # another launch pruned it
    total = sum(size for _, size, _, _ in entries)
    for _, size, links, path in sorted(entries):
        if total <= _MEDIA_CACHE_LIMIT:
            break
        if path == keep or links > 1:
            continue
        try:
            path.unlink()
        except (FileNotFoundError, PermissionError):
            continue
        total -= size


def windows_qemu_bin(native_bin: str, qemu_dir: str, accel: str) -> str:
    """Resolve the Windows QEMU exe for a native binary name. WHPX is compiled only into
    `qemu-system-x86_64.exe`, so an x86 guest under whpx uses the 64-bit emulator (it boots
    32-bit guests fine); under tcg we keep the matching per-arch exe."""
    name = native_bin
    if accel == "whpx" and native_bin == "qemu-system-i386":
        name = "qemu-system-x86_64"
    return str(Path(qemu_dir) / f"{name}.exe")


def rewrite_accel(args: list[str], accel: str, arch: str = "amd64") -> list[str]:
    """Swap the KVM accelerator for the Windows one and drop the paired `-cpu host`.
    `-accel kvm` -> `-accel <accel>` (whpx adds a PER-ARCH `kernel-irqchip` — see below).
    `-cpu host` is dropped for BOTH tcg and whpx: it's a KVM host-passthrough concept, and
    under WHPX it actively breaks boot — `-cpu host` makes OVMF fault on FXSAVE in early
    firmware (the host feature set WHPX exposes isn't what the 32/64-bit firmware expects),
    so WHPX falls back to qemu's default model. Operates on a copy.

    kernel-irqchip is PER-ARCH (measured 2026-07-03, qemu v11-dev):

      - amd64 needs `on` (Hyper-V's in-kernel APIC): with `off`, QEMU's userspace APIC
        round-trips the TPR through a lazy cache (`vcpu->tpr` / `cpu_set_apic_tpr` in
        qemu whpx-all.c) and the guest's CR8 is reset to the cached (stale, usually 0)
        value on EVERY VM exit — measured in-guest as 1000/1000 CR8 writes lost across a
        single port-I/O exit. amd64 NT keeps IRQL in CR8, so irqchip=off collapses every
        raised IRQL and bugchecks 0x9 at the first asserting spinlock release
        (historically "the uniata DMA bugcheck"). With `on`: 0/1000 lost, boots to
        `shell ready`.
      - i386 needs `off` (QEMU userspace PIC+APIC): our i386 UP kernel drives the legacy
        8259 PIC and keeps IRQL in memory (immune to the CR8 bug); under irqchip=on it
        wedges mid-boot (last serial: NETSTACK self-tests), under irqchip=off it boots
        to explorer.

    VIBEOS_WHPX_IRQCHIP overrides both for experiments.
    See docs/engineering/windows-host-qemu-whpx-spike.md."""
    out: list[str] = []
    i = 0
    while i < len(args):
        a = args[i]
        if a == "-accel" and i + 1 < len(args):
            if accel == "whpx":
                default_irqchip = "off" if arch.startswith("i386") else "on"
                irqchip = os.environ.get("VIBEOS_WHPX_IRQCHIP", default_irqchip)
                out += ["-accel", f"whpx,kernel-irqchip={irqchip}"]
            else:
                out += ["-accel", accel]
            i += 2
            continue
        if a == "-cpu" and i + 1 < len(args) and args[i + 1] == "host":
            i += 2  # drop `-cpu host` (KVM-only passthrough; faults OVMF under whpx)
            continue
        if (
            a == "-device"
            and i + 1 < len(args)
            and args[i + 1].startswith("ide-cd,")
            and (accel == "whpx" or os.environ.get("VIBEOS_AHCI_CD") == "1")
        ):
            # (VIBEOS_AHCI_CD=1 forces the AHCI attach under TCG too — debug
            # discriminator: exercises the guest AHCI driver path with WHPX's
            # interrupt-delivery quirks out of the picture.)
            # WHPX firmware CD I/O is glacial: OVMF's AtaAtapi driver reads the livecd
            # via IDE PIO (port 0x170/0x177), and every 2-byte read is a VM exit → the
            # uefildr file-load phase takes MINUTES (vs seconds under TCG's fast port
            # I/O). Put the CD on an AHCI controller instead: OVMF's AtaAtapi in AHCI
            # mode DMAs it (no per-word exit). The guest still reads it post-firmware —
            # uniata claims ich9-ahci (8086:2922, UNIATA_AHCI; bm_devs.h) and the
            # CC_0106 CriticalDeviceDatabase entry (hivesys.inf) binds uniata to the
            # AHCI controller during boot. TCG keeps plain IDE — its PIO is a cheap
            # host memcpy, no exit, so AHCI would only add enumeration cost there.
            out += ["-device", "ich9-ahci,id=ahci0"]
            out += ["-device", args[i + 1].replace("ide-cd,", "ide-cd,bus=ahci0.0,", 1)]
            i += 2
            continue
        out.append(a)
        i += 1
    return out


def resolve_accel(arch: str, configured: str, whpx_ok: bool | None = None) -> str:
    """Per-arch accelerator for the windows-host backend. WHPX is x86-ONLY (it lives only in
    `qemu-system-x86_64.exe` and runs an x86 guest on the host's Hyper-V platform); an arm64
    guest can't use it and boots via a real `-cpu cortex-a72` model that `rewrite_accel` would
    strip. So the configured value is interpreted per-arch:

      - arm64 → always "tcg" (the configured value is ignored — whpx can't run an aarch64 guest)
      - x86 + configured "whpx" + probe says DEFINITELY absent (`whpx_ok is False`) → "tcg"
      - else → the configured value verbatim

    The degrade is fail-soft by design: an x86 host without the Hyper-V platform enabled still
    boots, just on TCG. `whpx_ok is None` means "unknown/unprobed" → trust the configured value
    (don't degrade on an inconclusive probe). Pure + side-effect-free for unit testing."""
    if arch.startswith("arm64"):
        return "tcg"
    if configured == "whpx" and whpx_ok is False:
        return "tcg"
    return configured


def whpx_available() -> bool | None:
    """Best-effort probe: is a host hypervisor (the platform WHPX runs on) active on this
    Windows host? Reads `Win32_ComputerSystem.HypervisorPresent` via CIM — which, unlike
    `Get-WindowsOptionalFeature -Online` (elevation-gated, so it always came back unknown),
    works WITHOUT admin. Returns True (a hypervisor is running → whpx usable), False (none →
    whpx would fail, degrade to tcg), or None (couldn't determine → caller trusts config, does
    NOT degrade). HypervisorPresent is a slight over-approximation (it's also True under
    VBS/Hyper-V without the WHPX feature), but the only outcome it changes is detecting the
    genuinely-no-hypervisor host so it degrades to tcg instead of failing qemu's whpx init."""
    ps = "(Get-CimInstance -ClassName Win32_ComputerSystem).HypervisorPresent"
    try:
        proc = subprocess.run(
            ["powershell.exe", "-NoProfile", "-Command", ps],
            capture_output=True,
            text=True,
            timeout=30,
        )
    except (OSError, subprocess.SubprocessError):
        return None
    out = proc.stdout.strip().lower()
    if out == "true":
        return True
    if out == "false":
        return False
    return None


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
        if src.is_file():
            readonly = "readonly=on" in a.split(",")
            win = stage_readonly_file(src, staging) if readonly else stage_file(src, staging)
        else:
            win = win_path(src)
        out.append(a[: m.start(2)] + win + a[m.end(2) :])
    return out


def gateway_ip() -> str:
    """WSL's default-gateway IP = the Windows host as seen from WSL. A Windows-side
    service bound to `0.0.0.0` (e.g. a slirp hostfwd on the Windows QEMU) is reachable
    from WSL via THIS address, NOT `127.0.0.1` — WSL's loopback is the WSL VM's own,
    separate from the Windows host's. Mirror of the inbound rule (where the WSL listener
    must bind `0.0.0.0` for the Windows QEMU to dial it back)."""
    # /proc/net/route, not an `ip route` shell-out (iproute2 is one more host package to be
    # missing). Columns: Iface Destination Gateway Flags ...; hex, little-endian.
    table = Path("/proc/net/route").read_text()
    for line in table.splitlines()[1:]:
        cols = line.split()
        if len(cols) > 3 and cols[1] == "00000000" and int(cols[3], 16) & 0x2:  # RTF_GATEWAY
            return ".".join(str(b) for b in bytes.fromhex(cols[2])[::-1])
    raise RuntimeError(f"no default gateway in /proc/net/route:\n{table}")


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


def rewrite_netdev_for_windows(
    args: list[str], chardev_prefix: str = "gf"
) -> tuple[list[str], list[str]]:
    """Adapt the slirp `-netdev user,...` for the windows-host boundary. Two rewrites,
    mirror images of each other:

    - **hostfwd** (host->guest, the test dials in): `127.0.0.1` -> `0.0.0.0`. A
      127.0.0.1 hostfwd binds only the Windows loopback, invisible from WSL; bound to
      the wildcard it's reachable via `gateway_ip()` (what `QemuVM.net_host` returns).
    - **guestfwd** (guest->host echo): `-cmd:cat` runs a *Unix* program on the QEMU host
      — which here is Windows (no `cat`). Redirect it to `-chardev:<id>`; the caller wires
      each chardev to its own harness-run echo server.

    Returns (new_args, chardev_ids) — ONE id per guestfwd, in the order they appear.
    Every guestfwd must get its OWN id: a chardev is a single byte stream, so pointing
    two forwards at one chardev merges two guests' echo traffic (see vm.GUESTFWD_ECHO_PORTS
    for what that cost). `-cmd:cat` does not have that problem — slirp forks a fresh `cat`
    per CONNECTION — so the rewrite has to add back the separation the fork gave us.

    No-op on a netdev without hostfwd/guestfwd; native never calls this (native keeps
    `127.0.0.1` + `cmd:cat` unchanged)."""
    out: list[str] = []
    ids: list[str] = []
    for a in args:
        if a.startswith("user,") and ("hostfwd=" in a or "guestfwd=" in a):
            a = a.replace("hostfwd=tcp:127.0.0.1:", "hostfwd=tcp:0.0.0.0:")

            def _own_chardev(m: re.Match[str]) -> str:
                cid = f"{chardev_prefix}{len(ids)}"
                ids.append(cid)
                return f"{m.group(1)}-chardev:{cid}"

            a = re.sub(r"(guestfwd=tcp:[\d.]+:\d+)-cmd:cat", _own_chardev, a)
        out.append(a)
    return out, ids


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
