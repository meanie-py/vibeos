"""Host CPU broker — cross-worktree advisory locks so CPU-heavy work doesn't starve,
thrash, or corrupt itself on the shared dev host.

The contended resource on this box is **CPU**, not the QEMU accelerator: a cold build
(`ninja -j` over all cores) and a TCG/arm64 boot are both CPU-bound, so running them
together starves the boot into a false `HANG_TIMEOUT`, and two `build --clean` on the
SAME build dir corrupt each other. The per-arch QEMU slot cap in `vm.py` only covers
*accelerated* same-arch boots; this module covers the CPU-bound class that otherwise
has to be arbitrated by hand ("taking the host" / "host free").

Two flock-based locks, both under a shared `/tmp` dir so every worktree sees them and the
kernel auto-releases them when the holding process dies (no stale-lock cleanup needed):

- **`cpuheavy`** — a counting pool of `VIBEOS_HOST_CPU_LEASES` permits (default 4):
    * a cold **build** / `repro verify` grabs a **ceil(K/max_builds) share** (default 2 of 4,
      `VIBEOS_MAX_CONCURRENT_BUILDS=2`) → up to two builds coexist instead of fleet-wide
      build serialization; TWO running builds exhaust the pool, so native-TCG boots are
      still excluded exactly when the host is build-saturated (the false-`HANG_TIMEOUT`
      starvation incident). `VIBEOS_MAX_CONCURRENT_BUILDS=1` restores writer-takes-all.
      Shares are acquired with NO hold-and-wait (collect free slots nonblocking; on a
      shortfall release all + retry) so partial acquirers cannot deadlock;
    * a **TCG boot** grabs **1** permit → up to K TCG boots run in parallel (a ×3 gate keeps
      its boot parallelism); one running build leaves K-share permits of boot room.
  A boot takes a permit iff its EFFECTIVE accelerator is `tcg` (CPU-bound emulation). KVM /
  WHPX boots are hardware-accelerated → they take none and stay on the per-arch QEMU cap.
  **Only NATIVE-backend TCG boots use `cpuheavy`** (arm64, or x86-degraded) — those run in
  WSL and genuinely share cores with a WSL build, so a build must still exclude them.
- **`bootcpu`** — a SEPARATE pool of `VIBEOS_BOOT_CPU_LEASES` permits for **windows-host**
  QEMU boots (`qemu.exe` on the Windows side). Because that's a different scheduling domain
  from the WSL build, a build's exclusive `cpuheavy` grant must NOT block it — the build
  instead leaves physical-core headroom via its `-j` cap (`VIBEOS_BUILD_CORE_RESERVE`), so
  build ∥ boot. This pool throttles boot-vs-boot only.
- **`builddir:<canonical-path>`** — exclusive per-build-dir lock. Cheap insurance that two
  builds never land on the same dir (the determinism-verify-vs-gate corruption class), held
  *under* the cpuheavy grant.

Acquire order is always `cpuheavy` → `builddir`, and a build grabs the K permits IN ORDER
(0,1,…) so two builds can't deadlock. Timeouts fail SOFT (warn + proceed unlocked) rather
than wedge a build forever — a generous default ceiling means hitting it implies a stuck
holder, not real contention.
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
# Pool size = max concurrent TCG boots (a build grabs ALL of them → exclusive). 4 suits a
# ~32-core box (≈4×4 vCPU boots leaves headroom); raise/lower via the env.
DEFAULT_CPU_LEASES = max(1, int(os.environ.get("VIBEOS_HOST_CPU_LEASES", "4")))
# SEPARATE pool for windows-host QEMU boots. Those run on the WINDOWS side (qemu.exe) — a
# different scheduling domain from the WSL-side build — so a build's writer-takes-all
# `cpuheavy` grant must NOT exclude them; the build leaves physical-core headroom via its
# `-j` cap instead (VIBEOS_BUILD_CORE_RESERVE), and this pool throttles boot-vs-boot only.
# native TCG boots (arm64) stay on `cpuheavy` because they DO share WSL cores with the build.
DEFAULT_BOOT_LEASES = max(1, int(os.environ.get("VIBEOS_BOOT_CPU_LEASES", str(DEFAULT_CPU_LEASES))))
# Generous ceiling: a real cold build + queue is minutes; hitting an hour = a stuck holder.
ACQUIRE_TIMEOUT = float(os.environ.get("VIBEOS_HOST_LOCK_TIMEOUT", "3600"))
_POLL = 0.5


def enabled() -> bool:
    """The broker is on by default; `VIBEOS_HOST_BROKER=0` disables it (uncontended single
    agent / CI, where the locks are a no-op anyway, or to debug without arbitration)."""
    return os.environ.get("VIBEOS_HOST_BROKER", "1").lower() not in ("0", "false", "no", "")


def _slug(key: str) -> str:
    return hashlib.sha1(key.encode()).hexdigest()[:16]


# Shared-protocol version (T2, fleet-isolation program 2026-07-04): worktrees run
# DIVERGENT tooling versions against the same lease files, so cross-version
# behavior must be explicit. Readers treat a missing field as v0 and still
# cooperate (never brick another lane), but WARN loudly — the dedup incident was
# exactly old-code-in-another-lane operating on shared state with obsolete
# semantics. Bump when lease semantics change incompatibly.
PROTOCOL_VERSION = 1


def _self_meta(kind: str, **extra: object) -> dict:
    """Holder identity written into the lock file so `host-status` can show who has it."""
    meta = {
        "v": PROTOCOL_VERSION,
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


# The queue/acquire announcement pair. A captured transcript containing WAIT_ANNOUNCE with
# no ACQUIRED_ANNOUNCE means the process spent its whole budget QUEUED on the broker and
# never ran the work it was queueing for (field manual §9.84) — `looks_starved` below is
# the classifier both gate.py and smoke.py use to tell that apart from a real test failure.
WAIT_ANNOUNCE = "[hostlock] waiting for host CPU lease"
ACQUIRED_ANNOUNCE = "[hostlock] lease acquired"


def looks_starved(output: str) -> bool:
    """True when `output` (a captured pytest/boot transcript) announced queueing on the
    lease broker but never printed the matching acquisition line. ACQUIRED_ANNOUNCE is
    printed ONLY after an announced wait, so the pair-absence is exact: the run's flat
    timeouts are host starvation (another lane owns the leases), not a code regression."""
    return WAIT_ANNOUNCE in output and ACQUIRED_ANNOUNCE not in output


def _acquire_one(files: list[Path], timeout: float, meta: dict) -> int | None:
    """Try to grab ANY of `files` (an exclusive flock). Returns the held fd, or None on
    timeout. Loops the candidate set so a lease of N permits picks the first free slot."""
    HOSTLOCK_DIR.mkdir(parents=True, exist_ok=True)
    t0 = time.monotonic()
    deadline = t0 + timeout
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
            if announced:
                print(
                    f"{ACQUIRED_ANNOUNCE} ({meta.get('kind', '?')}) "
                    f"after {time.monotonic() - t0:.0f}s queued",
                    flush=True,
                )
            return fd
        if time.monotonic() >= deadline:
            return None
        if not announced:
            print(f"{WAIT_ANNOUNCE} ({meta.get('kind', '?')})...", flush=True)
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


def _boot_lease_files(leases: int) -> list[Path]:
    return [HOSTLOCK_DIR / f"bootcpu-{i:02d}" for i in range(leases)]


def acquire_boot_lease(
    kind: str,
    *,
    leases: int = DEFAULT_BOOT_LEASES,
    timeout: float = ACQUIRE_TIMEOUT,
    **extra: object,
) -> int | None:
    """Acquire one BOOT-pool permit (for a windows-host qemu.exe boot). SEPARATE from the
    `cpuheavy` build pool, so a cold build's exclusive grant never excludes a windows-host
    boot (they're different scheduling domains — Windows vs WSL). Throttles boot-vs-boot
    only. Returns the held fd or None on timeout."""
    return _acquire_one(
        _boot_lease_files(leases), timeout, _self_meta(kind, boot_pool=True, **extra)
    )


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


def _max_builds() -> int:
    """Max concurrent builds host-wide (VIBEOS_MAX_CONCURRENT_BUILDS, default 2).
    Read at CALL time (not import) so tests and per-invocation env overrides work.
    Set to 1 to restore the original exclusive-host build behavior."""
    return max(1, int(os.environ.get("VIBEOS_MAX_CONCURRENT_BUILDS", "2")))


def build_permits(leases: int = DEFAULT_CPU_LEASES) -> int:
    """Permits a build takes from the cpuheavy pool: ceil(K / max_builds). With the
    defaults (K=4, max 2 builds) that's 2 — so two builds coexist, a third queues, and
    while TWO builds run the pool is exhausted (native-TCG boots excluded, preserving
    the starvation protection the old writer-takes-all grant existed for)."""
    return max(1, -(-leases // _max_builds()))


def acquire_cpu_share(
    kind: str,
    *,
    n: int,
    leases: int = DEFAULT_CPU_LEASES,
    timeout: float = ACQUIRE_TIMEOUT,
    **extra: object,
) -> list[int] | None:
    """Grab `n` of the `leases` CPU permits WITHOUT hold-and-wait: scan all slots
    nonblocking, and on a shortfall release everything collected and retry. No partial
    holder ever blocks, so two share-acquirers can't deadlock each other (and an
    old-protocol exclusive acquirer, which DOES hold-and-wait in slot order, still
    completes because share holders always finish and release). Returns the held fds,
    or None on timeout."""
    meta = _self_meta(kind, share=n, **extra)
    HOSTLOCK_DIR.mkdir(parents=True, exist_ok=True)
    deadline = time.monotonic() + timeout
    announced = False
    while True:
        held: list[int] = []
        for p in _cpu_lease_files(leases):
            fd = os.open(str(p), os.O_RDWR | os.O_CREAT, 0o644)
            try:
                fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
            except OSError:
                os.close(fd)
                continue
            _write_meta(fd, meta)
            held.append(fd)
            if len(held) == n:
                return held
        # Shortfall: release ALL (no hold-and-wait) and retry after a poll.
        for h in held:
            release(h)
        if time.monotonic() >= deadline:
            return None
        if not announced:
            print(f"[hostlock] waiting for {n} host CPU lease(s) ({kind})...")
            announced = True
        time.sleep(_POLL)


def acquire_cpu_exclusive(
    kind: str,
    *,
    leases: int = DEFAULT_CPU_LEASES,
    timeout: float = ACQUIRE_TIMEOUT,
    **extra: object,
) -> list[int] | None:
    """Grab ALL `leases` CPU permits (a build → exclusive host). Permits are taken IN ORDER
    (0,1,…) so two concurrent builds can't deadlock. Returns the list of held fds, or None on
    timeout (releasing whatever it had partially acquired)."""
    meta = _self_meta(kind, exclusive=True, **extra)
    deadline = time.monotonic() + timeout
    held: list[int] = []
    HOSTLOCK_DIR.mkdir(parents=True, exist_ok=True)
    announced = False
    for p in _cpu_lease_files(leases):
        while True:
            fd = os.open(str(p), os.O_RDWR | os.O_CREAT, 0o644)
            try:
                fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
            except OSError:
                os.close(fd)
                if time.monotonic() >= deadline:
                    for h in held:
                        release(h)
                    return None
                if not announced:
                    print(f"[hostlock] waiting for exclusive host ({kind})...")
                    announced = True
                time.sleep(_POLL)
                continue
            _write_meta(fd, meta)
            held.append(fd)
            break
    return held


def release_all(fds: list[int] | None) -> None:
    """Release every fd from an exclusive grant."""
    for fd in fds or []:
        release(fd)


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
def cpu_exclusive(kind: str, **kw: object) -> Iterator[list[int] | None]:
    """Context manager: grab the WHOLE CPU pool (a build → exclusive host). No-op when the
    broker is off; warn + proceed UNLOCKED on timeout."""
    if not enabled():
        yield None
        return
    fds = acquire_cpu_exclusive(kind, **kw)  # type: ignore[arg-type]
    if fds is None:
        print(
            f"[hostlock] WARNING: exclusive-host timeout ({kind}) — proceeding UNLOCKED "
            "(a holder looks stuck; check `vibeos-cli host-status`)",
            file=sys.stderr,
        )
    try:
        yield fds
    finally:
        release_all(fds)


@contextmanager
def cpu_share(kind: str, n: int, **kw: object) -> Iterator[list[int] | None]:
    """Context manager around an n-permit share of the CPU pool. No-op when the broker
    is off; warn + proceed UNLOCKED on timeout."""
    if not enabled():
        yield None
        return
    fds = acquire_cpu_share(kind, n=n, **kw)  # type: ignore[arg-type]
    if fds is None:
        print(
            f"[hostlock] WARNING: CPU share timeout ({kind}, n={n}) — proceeding UNLOCKED "
            "(a holder looks stuck; check `vibeos-cli host-status`)",
            file=sys.stderr,
        )
    try:
        yield fds
    finally:
        release_all(fds)


@contextmanager
def build_lock(build_dir: str | Path, arch: str, **kw: object) -> Iterator[None]:
    """The build front-door grant: a ceil(K/max_builds) SHARE of the CPU pool (default
    2 of 4 → up to VIBEOS_MAX_CONCURRENT_BUILDS=2 builds coexist; two running builds
    exhaust the pool, so native-TCG boots stay starvation-protected exactly when the
    host is build-saturated) AND the per-build-dir lock (same-dir corruption guard),
    acquired cpuheavy→builddir. VIBEOS_MAX_CONCURRENT_BUILDS=1 restores the original
    exclusive-host grant. No-op when the broker is disabled."""
    if not enabled():
        yield None
        return
    n = build_permits()
    with cpu_share(f"build:{arch}", n, **kw), build_dir_lock(build_dir, arch=arch):
        yield None


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
    out: dict = {
        "dir": str(HOSTLOCK_DIR),
        "enabled": enabled(),
        "cpuheavy": [],
        "bootcpu": [],
        "builddir": [],
        "cpu_leases_total": DEFAULT_CPU_LEASES,
        "cpu_leases_held": 0,
        "boot_leases_total": DEFAULT_BOOT_LEASES,
        "boot_leases_held": 0,
    }
    if not HOSTLOCK_DIR.is_dir():
        return out  # no lock dir yet → nothing held (keys above stay at their defaults)
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
            # protocol-version skew (T2): a live holder speaking a different lease
            # protocol means another lane runs divergent tooling against this
            # shared state — cooperate anyway, but surface it loudly.
            "protocol_skew": bool(held and meta and meta.get("v", 0) != PROTOCOL_VERSION),
        }
        if p.name.startswith("cpuheavy-"):
            out["cpuheavy"].append(entry)
        elif p.name.startswith("bootcpu-"):
            out["bootcpu"].append(entry)
        elif p.name.startswith("builddir-"):
            out["builddir"].append(entry)
    out["cpu_leases_total"] = DEFAULT_CPU_LEASES
    out["cpu_leases_held"] = sum(1 for e in out["cpuheavy"] if e["held"])
    out["boot_leases_total"] = DEFAULT_BOOT_LEASES
    out["boot_leases_held"] = sum(1 for e in out["bootcpu"] if e["held"])
    return out


def render_status(st: dict) -> str:
    """Human-readable host-broker summary (the `vibeos-cli host-status` body)."""
    lines = [
        f"host broker: {'ON' if st['enabled'] else 'OFF (VIBEOS_HOST_BROKER=0)'}  —  "
        f"build(cpuheavy) {st['cpu_leases_held']}/{st['cpu_leases_total']} · "
        f"boot(windows-host) {st.get('boot_leases_held', 0)}/{st.get('boot_leases_total', 0)}"
    ]
    held_cpu = [e for e in st["cpuheavy"] if e["held"]]
    if not held_cpu:
        lines.append("  cpuheavy : FREE — no cold build / TCG boot running")
    else:
        # Group permits by holder pid so an exclusive build (holds ALL K) shows one line.
        by_pid: dict = {}
        for e in held_cpu:
            by_pid.setdefault(e["holder"].get("pid"), []).append(e)
        for pid, entries in by_pid.items():
            h = entries[0]["holder"]
            kind = str(h.get("kind", "?"))
            n = len(entries)
            tag = "EXCLUSIVE" if h.get("exclusive") else f"{n} permit{'s' if n > 1 else ''}"
            dead = "" if entries[0]["holder_alive"] else "  ⚠STALE(holder gone)"
            lines.append(f"  cpuheavy : {kind:<14} pid={pid} ({tag}){dead}  {h.get('cwd', '')}")
    for e in (e for e in st.get("bootcpu", []) if e["held"]):
        h = e["holder"]
        dead = "" if e["holder_alive"] else "  ⚠STALE(holder gone)"
        lines.append(f"  bootcpu  : {h.get('kind', '?')!s:<14} pid={h.get('pid', '?')}{dead}")
    for e in (e for e in st["builddir"] if e["held"]):
        h = e["holder"]
        lines.append(f"  builddir : {h.get('build_dir', '?')}  (pid={h.get('pid', '?')})")
    skewed = [
        e
        for pool in ("cpuheavy", "bootcpu", "builddir")
        for e in st.get(pool, [])
        if e.get("protocol_skew")
    ]
    if skewed:
        lines.append(
            f"  ⚠ PROTOCOL SKEW: {len(skewed)} live holder(s) speak a different lease "
            f"protocol (v{PROTOCOL_VERSION} here) — another lane runs divergent tooling; "
            f"cooperate but expect surprises until it merges"
        )
    lines.append("  (QEMU boots: `vibeos-cli ps`)")
    return "\n".join(lines)


def cmd_status(args, repo: Path) -> int:
    """`vibeos-cli host-status` — who holds the CPU lease / build-dir locks. `--json`
    feeds the wait-for-quiet poll guards agents have been hand-rolling."""
    st = status()
    if getattr(args, "json", False):
        print(json.dumps(st, indent=2))
    else:
        print(render_status(st))
    return 0
