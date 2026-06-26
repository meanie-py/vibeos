"""Host CPU broker — cross-worktree advisory locks so CPU-heavy work doesn't starve,
thrash, or corrupt itself on the shared dev host.

The contended resource on this box is **CPU**, not the QEMU accelerator: a cold build
(`ninja -j` over all cores) and a TCG/arm64 boot are both CPU-bound, so running them
together starves the boot into a false `HANG_TIMEOUT`, and two `build --clean` on the
SAME build dir corrupt each other. The per-arch QEMU slot cap in `vm.py` only covers
*accelerated* same-arch boots; this module covers the CPU-bound class the swarm has been
arbitrating by hand in #swarm ("taking the host" / "host free").

Two flock-based locks, both under a shared `/tmp` dir so every worktree sees them and the
kernel auto-releases them when the holding process dies (no stale-lock cleanup needed):

- **`cpuheavy`** — a counting LEASE (default 1 permit). Held for the duration of a cold
  build / `repro verify`, and by TCG (arm64) boots. Accelerated x86 boots do NOT take it
  (they keep the per-arch QEMU cap, so cross-arch boot parallelism is preserved). At the
  default 1 permit this serializes {build, build} and {build, TCG-boot} and {TCG-boot,
  TCG-boot} — every CPU-starvation/thrash class we hit.
- **`builddir:<canonical-path>`** — exclusive per-build-dir lock. Redundant while
  `cpuheavy` is a 1-permit mutex, but makes raising `VIBEOS_HOST_CPU_LEASES` corruption-safe
  (two concurrent builds then can't land on the same dir).

Acquire order is always `cpuheavy` → `builddir` (cpuheavy outermost) so there is no lock-
ordering deadlock. Timeouts fail SOFT (warn + proceed unlocked) rather than wedge a build
forever — a generous default ceiling means hitting it implies a stuck holder, not real
contention.
"""

from __future__ import annotations

import fcntl
import hashlib
import json
import os
import sys
import time
from collections.abc import Iterator
from contextlib import contextmanager
from pathlib import Path

# Shared with the qemu slot locks (vm.py LOCK_DIR) so `reap.sh`/`ps` live in one tree.
HOSTLOCK_DIR = Path(os.environ.get("VIBEOS_HOSTLOCK_DIR", "/tmp/vibeos-qemu-locks/host"))
# Default 1 = full serialize (matches the manual "one heavy thing at a time"). Raise on a
# beefy host to allow N concurrent CPU-heavy ops; the builddir lock keeps that safe.
DEFAULT_CPU_LEASES = max(1, int(os.environ.get("VIBEOS_HOST_CPU_LEASES", "1")))
# Generous ceiling: a real cold build + queue is minutes; hitting an hour = a stuck holder.
ACQUIRE_TIMEOUT = float(os.environ.get("VIBEOS_HOST_LOCK_TIMEOUT", "3600"))
_POLL = 0.5


def enabled() -> bool:
    """The broker is on by default; `VIBEOS_HOST_BROKER=0` disables it (uncontended single
    agent / CI, where the locks are a no-op anyway, or to debug without arbitration)."""
    return os.environ.get("VIBEOS_HOST_BROKER", "1").lower() not in ("0", "false", "no", "")


def _slug(key: str) -> str:
    return hashlib.sha1(key.encode()).hexdigest()[:16]


def _self_meta(kind: str, **extra: object) -> dict:
    """Holder identity written into the lock file so `host-status` can show who has it."""
    meta = {
        "kind": kind,
        "pid": os.getpid(),
        "argv": " ".join(sys.argv)[:200],
        "cwd": os.getcwd(),
        # NB: NO wall-clock acquire time — Date.now()-style values would make the file's
        # content nondeterministic, but more importantly the reader derives liveness from
        # the pid, not a timestamp. Callers may add their own via **extra.
    }
    meta.update(extra)
    return meta


def _write_meta(fd: int, meta: dict) -> None:
    """Best-effort: stamp the locked file with holder metadata (never fails the acquire)."""
    try:
        os.ftruncate(fd, 0)
        os.lseek(fd, 0, os.SEEK_SET)
        os.write(fd, json.dumps(meta).encode())
        os.fsync(fd)
    except OSError:
        pass


def _acquire_one(files: list[Path], timeout: float, meta: dict) -> int | None:
    """Try to grab ANY of `files` (an exclusive flock). Returns the held fd, or None on
    timeout. Loops the candidate set so a lease of N permits picks the first free slot."""
    HOSTLOCK_DIR.mkdir(parents=True, exist_ok=True)
    deadline = time.monotonic() + timeout
    announced = False
    while True:
        for p in files:
            fd = os.open(str(p), os.O_RDWR | os.O_CREAT, 0o644)
            try:
                fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
            except OSError:
                os.close(fd)
                continue
            _write_meta(fd, meta)
            return fd
        if time.monotonic() >= deadline:
            return None
        if not announced:
            print(f"[hostlock] waiting for host CPU lease ({meta.get('kind', '?')})...")
            announced = True
        time.sleep(_POLL)


def release(fd: int | None) -> None:
    """Release a lock (closing the fd also releases the flock; we unlock explicitly first)."""
    if fd is None:
        return
    try:
        fcntl.flock(fd, fcntl.LOCK_UN)
    except OSError:
        pass
    try:
        os.close(fd)
    except OSError:
        pass


def _cpu_lease_files(leases: int) -> list[Path]:
    return [HOSTLOCK_DIR / f"cpuheavy-{i:02d}" for i in range(leases)]


def acquire_cpu_lease(
    kind: str,
    *,
    leases: int = DEFAULT_CPU_LEASES,
    timeout: float = ACQUIRE_TIMEOUT,
    **extra: object,
) -> int | None:
    """Acquire one CPU-heavy lease permit (default pool size 1). Returns the held fd or
    None on timeout. `kind` is a short tag for host-status (e.g. 'build:i386', 'boot:arm64')."""
    return _acquire_one(_cpu_lease_files(leases), timeout, _self_meta(kind, **extra))


def acquire_build_dir(
    build_dir: str | Path, *, timeout: float = ACQUIRE_TIMEOUT, **extra: object
) -> int | None:
    """Acquire the exclusive lock for a build dir (keyed by its real, canonical path so the
    SAME dir from any worktree maps to one lock). Returns the held fd or None on timeout."""
    key = str(Path(build_dir).resolve())
    f = HOSTLOCK_DIR / f"builddir-{_slug(key)}"
    return _acquire_one([f], timeout, _self_meta("builddir", build_dir=key, **extra))


@contextmanager
def cpu_lease(kind: str, **kw: object) -> Iterator[int | None]:
    """Context manager around a CPU-heavy lease. No-op (yields None) when the broker is
    disabled; warns + proceeds UNLOCKED on timeout (never wedges the caller forever)."""
    if not enabled():
        yield None
        return
    fd = acquire_cpu_lease(kind, **kw)  # type: ignore[arg-type]
    if fd is None:
        print(
            f"[hostlock] WARNING: CPU lease timeout ({kind}) — proceeding UNLOCKED "
            "(a holder looks stuck; check `vibeos-cli host-status`)",
            file=sys.stderr,
        )
    try:
        yield fd
    finally:
        release(fd)


@contextmanager
def build_dir_lock(build_dir: str | Path, **kw: object) -> Iterator[int | None]:
    """Context manager around the per-build-dir exclusive lock (no-op when disabled)."""
    if not enabled():
        yield None
        return
    fd = acquire_build_dir(build_dir, **kw)  # type: ignore[arg-type]
    if fd is None:
        print(
            f"[hostlock] WARNING: build-dir lock timeout ({build_dir}) — proceeding "
            "UNLOCKED (a holder looks stuck; check `vibeos-cli host-status`)",
            file=sys.stderr,
        )
    try:
        yield fd
    finally:
        release(fd)


def _pid_alive(pid: int) -> bool:
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        return True  # exists, owned by someone else
    except OSError:
        return False
    return True


def _is_held(p: Path) -> bool:
    """Probe whether a lock file is currently held by anyone (NB flock that we immediately
    drop if it was free). A held lock → EWOULDBLOCK → True."""
    try:
        fd = os.open(str(p), os.O_RDWR)
    except OSError:
        return False
    try:
        fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
        fcntl.flock(fd, fcntl.LOCK_UN)  # it was free — release immediately
        return False
    except OSError:
        return True
    finally:
        os.close(fd)


def status() -> dict:
    """Snapshot of the host broker for `host-status`: which cpuheavy permits + build-dir
    locks are held, by whom (holder metadata), and whether that holder is still alive."""
    out: dict = {"dir": str(HOSTLOCK_DIR), "enabled": enabled(), "cpuheavy": [], "builddir": []}
    if not HOSTLOCK_DIR.is_dir():
        return out
    for p in sorted(HOSTLOCK_DIR.glob("*")):
        held = _is_held(p)
        meta: dict = {}
        try:
            raw = p.read_text().strip()
            if raw:
                meta = json.loads(raw)
        except (OSError, ValueError):
            pass
        entry = {
            "name": p.name,
            "held": held,
            "holder": meta,
            "holder_alive": _pid_alive(int(meta["pid"])) if meta.get("pid") else None,
        }
        if p.name.startswith("cpuheavy-"):
            out["cpuheavy"].append(entry)
        elif p.name.startswith("builddir-"):
            out["builddir"].append(entry)
    out["cpu_leases_total"] = DEFAULT_CPU_LEASES
    out["cpu_leases_held"] = sum(1 for e in out["cpuheavy"] if e["held"])
    return out
