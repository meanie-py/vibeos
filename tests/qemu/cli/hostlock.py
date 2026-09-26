"""Host ownership and correctness locks shared across worktrees.

New builds hold only their build-directory lock. The Rust host-jobs runner
admits individual compiler/linker processes with priority and aging. Cargo,
Ninja and directory waiters reserve no CPU capacity. The legacy cpuheavy pool
remains for older lanes and native TCG; the runner discounts its live holders
while those lanes migrate. Windows-host boots retain their separate boot pool.

Build-directory locks never fail open, including when CPU throttling is disabled.
Legacy admission helpers remain available to existing non-build consumers.
"""

from __future__ import annotations

import contextlib
import fcntl
import hashlib
import json
import math
import os
import sys
import time
from collections.abc import Iterator
from contextlib import contextmanager
from pathlib import Path

from . import buildpolicy

# Bumped to 2 when the build policy moved out of HOSTLOCK_DIR. `buildpolicy.blockers()`
# reads this literal out of every registered worktree, so a lane still on 1 is NAMED as
# policy-unaware instead of quietly reading the old /tmp path, finding nothing after a
# reboot, and building uncapped while everyone assumes the cap is on.
HOST_POLICY_VERSION = 2

# Shared with the qemu slot locks (vm.py LOCK_DIR) so `reap.sh`/`ps` live in one tree.
# EPHEMERAL BY DESIGN: everything here is flock state, whose meaning dies with the
# process holding it, so /tmp losing it on reboot is correct.
HOSTLOCK_DIR = Path(os.environ.get("VIBEOS_HOSTLOCK_DIR", "/tmp/vibeos-qemu-locks/host"))
# DURABLE config, which is a different lifetime entirely -- the capacity of this host is
# not runtime state and must outlive a reboot. Kept separate rather than moving the whole
# lock tree: stale lock FILES are harmless, but a stale *pause* or capacity would not be.
POLICY_DIR = buildpolicy.durable_dir()
# Pool size = max concurrent TCG boots (a build grabs ALL of them → exclusive). 4 suits a
# ~32-core box (≈4×4 vCPU boots leaves headroom); raise/lower via the env.
DEFAULT_CPU_LEASES = max(1, int(os.environ.get("VIBEOS_HOST_CPU_LEASES", "4")))


def _derive_boot_leases() -> int:
    """Default size of the `bootcpu` pool — how many windows-host QEMU boots run at once,
    fleet-wide. SEPARATE from `cpuheavy`: these run on the WINDOWS side (qemu.exe), a
    different scheduling domain from the WSL-side build, so a build's grant must NOT
    exclude them (the build leaves core headroom via VIBEOS_BUILD_CORE_RESERVE instead).
    native TCG boots (arm64 on the native backend) stay on `cpuheavy` — they DO share WSL
    cores with a build. This pool throttles boot-vs-boot only.

    SIZING (2026-07-25). This was a flat 4, inherited from the `cpuheavy` default — and 4
    is EXACTLY one full gate's boot fan-out (i386 + amd64 + arm64 + the amd64 net lane).
    So a single gating lane saturated the pool for the whole fleet and every other lane's
    boots queued behind it end-to-end; add the gate's up-to-2 concurrent flake-retries and
    even ONE lane oversubscribes its own pool. That queueing is a large part of what
    "agents are constantly waiting on the gate" actually was.

    What a boot really costs: the smoke guest is UNIPROCESSOR (`-smp` is passed only by the
    SMP tests), i.e. one TCG vCPU thread + an iothread ≈ ~1.3 busy host threads, with
    `-m 512M..1G`. On this box (16C/32T + 63 GB Windows-side; WSL sees 24 vCPU / 47 GB) the
    old cap spent ~15% of the CPU and ~6% of the RAM — a 3× under-subscription.

    So: cpu/3, floored at the historical 4 (never *fewer* boots than before on a small box)
    and ceilinged at 12 (beyond that TCG boots start stealing from each other and from the
    build, and `boot_timeout` headroom is finite). 24 vCPU → 8, which is also the design
    requirement: the pool must fit ≥2 concurrent full gates or lanes serialize by
    construction. Override with VIBEOS_BOOT_CPU_LEASES.
    """
    n = os.cpu_count() or 0
    return max(4, min(12, n // 3))


DEFAULT_BOOT_LEASES = max(
    1, int(os.environ.get("VIBEOS_BOOT_CPU_LEASES", str(_derive_boot_leases())))
)
# Generous ceiling: a real cold build + queue is minutes; hitting an hour = a stuck holder.
ACQUIRE_TIMEOUT = float(os.environ.get("VIBEOS_HOST_LOCK_TIMEOUT", "3600"))
_POLL = 0.5


def enabled() -> bool:
    """The broker is on by default; `VIBEOS_HOST_BROKER=0` disables it (uncontended single
    agent / CI, where the locks are a no-op anyway, or to debug without arbitration)."""
    on = os.environ.get("VIBEOS_HOST_BROKER", "1").lower() not in ("0", "false", "no", "")
    if not on and (buildpolicy.read(POLICY_DIR) or (POLICY_DIR / "build-policy.paused").exists()):
        raise buildpolicy.PolicyError("active host build policy requires the CPU broker")
    return on


def _slug(key: str) -> str:
    return hashlib.sha1(key.encode()).hexdigest()[:16]


# Shared-protocol version (T2, fleet-isolation program 2026-07-04): worktrees run
# DIVERGENT tooling versions against the same lease files, so cross-version
# behavior must be explicit. Readers treat a missing field as v0 and still
# cooperate (never brick another lane), but WARN loudly — the dedup incident was
# exactly old-code-in-another-lane operating on shared state with obsolete
# semantics. Bump when lease semantics change incompatibly.
PROTOCOL_VERSION = 1


def job_class() -> str:
    """The job's class, for tagging -- and for ONE admission decision: an `integration`
    job (a landing gate) may take the boot permits `land_boot_reserve` holds back from
    everyone else. Nothing else about admission reads it."""
    value = os.environ.get("VIBEOS_JOB_CLASS", "interactive")
    return value if value in ("interactive", "integration", "background") else "unknown"


def _self_meta(kind: str, **extra: object) -> dict:
    """Holder identity written into the lock file so `host-status` can show who has it."""
    meta = {
        "v": PROTOCOL_VERSION,
        "kind": kind,
        "pid": os.getpid(),
        "argv": " ".join(sys.argv)[:200],
        "cwd": os.getcwd(),
        "job_class": job_class(),
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

#: What each pool's wait actually MEANS, appended to WAIT_ANNOUNCE (whose text is kept
#: as-is: `looks_starved` and its tests match on it). Before this, every pool printed
#: "waiting for host CPU lease", so a conductor queued 73 minutes behind ANOTHER
#: conductor's land lock read like boot-slot starvation -- and was diagnosed as such.
POOL_WHAT = {
    "land": "the fleet LAND LOCK -- another conductor is draining the queue",
    "bootcpu": "a QEMU boot permit (the unreserved share; landing keeps the rest)",
    "bootcpu-land": "a QEMU boot permit for a LANDING gate (reserved share first)",
    "cpuheavy": "a build / native-TCG CPU permit",
}


def _wait_line(kind: object, pool: str | None) -> str:
    what = POOL_WHAT.get(pool or "", f"pool {pool}" if pool else "an unqueued lock")
    return f"{WAIT_ANNOUNCE} ({kind}; waiting on {what})..."


ACQUIRED_ANNOUNCE = "[hostlock] lease acquired"


def looks_starved(output: str) -> bool:
    """True when `output` (a captured pytest/boot transcript) announced queueing on the
    lease broker but never printed the matching acquisition line. ACQUIRED_ANNOUNCE is
    printed ONLY after an announced wait, so the pair-absence is exact: the run's flat
    timeouts are host starvation (another lane owns the leases), not a code regression."""
    return WAIT_ANNOUNCE in output and ACQUIRED_ANNOUNCE not in output


# ── FIFO admission (fairness) ───────────────────────────────────────────────
#
# WHY: permit acquisition is "scan nonblocking, and on a shortfall release everything and
# retry" (no hold-and-wait, so partial acquirers can't deadlock). That is deadlock-safe but
# ORDER-BLIND: a waiter has no claim on the permits it is waiting for, so a lane that
# arrives later can take them the instant a holder releases. A patient waiter can lose every
# race indefinitely. OBSERVED 2026-07-24: an arm64 build queued 23 MINUTES while the permits
# it needed changed hands between two other worktrees; it only got in when a lane finished
# outright. It presents as "the build is slow", not "you are starved" — which is why
# `looks_starved()` below exists at all (field manual §9.84 detects it AFTER the fact).
#
# FIX: an arrival-ordered ticket queue in front of the pool. A waiter registers a ticket and
# may only attempt to take permits when doing so cannot delay anyone who arrived earlier.
#
# CRASH-SAFETY (the hard part — waiters die: Ctrl-C, OOM, a killed agent): a ticket is a FILE
# ITS OWNER HOLDS AN FLOCK ON for the whole wait, so the kernel releases it on death exactly
# like the permits themselves. A ticket whose flock can be taken by somebody else is by
# definition an orphan, and the prober deletes it. No stale-lock sweeper, no pid liveness
# guessing, no timestamp heuristics. The ticket is created under a dotted temp name, flocked,
# and only THEN renamed into place, so a ticket is never visible before it is locked (which
# would otherwise race the orphan reaper into deleting a live waiter's ticket).
#
# DEADLOCK: none added. Tickets are advisory ORDERING only — they gate when you *attempt*,
# never what you hold. The permit scan underneath is still no-hold-and-wait.
#
# MIXED VERSIONS: every worktree runs its own copy of this file, so during a rollout an
# old-code lane takes no ticket and still barges. That degrades to exactly today's behavior
# (no deadlock, no lost permits) and self-corrects as lanes merge. Fairness holds among
# ticket-takers. `VIBEOS_HOSTLOCK_FAIR=0` opts out entirely.


def fair_enabled() -> bool:
    """FIFO admission is on by default; `VIBEOS_HOSTLOCK_FAIR=0` restores the old
    order-blind free-for-all (escape hatch if the queue itself ever misbehaves)."""
    return os.environ.get("VIBEOS_HOSTLOCK_FAIR", "1").lower() not in ("0", "false", "no", "")


def _queue_dir(pool: str) -> Path:
    # Derived at CALL time: the tests monkeypatch HOSTLOCK_DIR.
    return HOSTLOCK_DIR / f"queue-{pool}"


def _ticket_take(pool: str, need: int) -> tuple[int, Path] | None:
    """Register an arrival-ordered ticket for `need` permits. Returns (held fd, path), or
    None if fairness is off. The fd must be held for the whole wait."""
    if not fair_enabled():
        return None
    qd = _queue_dir(pool)
    try:
        qd.mkdir(parents=True, exist_ok=True)
        # Wall clock (not monotonic): the ordering is compared ACROSS processes.
        name = f"{time.time_ns():020d}-{os.getpid()}-{need}"
        tmp = qd / f".tmp-{os.getpid()}-{time.time_ns()}"
        fd = os.open(str(tmp), os.O_RDWR | os.O_CREAT, 0o644)
        try:
            fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
            _write_meta(fd, _self_meta("waiter", queued_monotonic=time.monotonic()))
            os.rename(str(tmp), str(qd / name))  # atomic: visible ⇒ already locked
        except OSError:
            os.close(fd)
            with contextlib.suppress(OSError):
                tmp.unlink()
            return None
        return fd, qd / name
    except OSError:
        return None  # queue unusable → fall back to order-blind (never fail an acquire)


def _ticket_drop(ticket: tuple[int, Path] | None) -> None:
    if ticket is None:
        return
    fd, path = ticket
    with contextlib.suppress(OSError):
        path.unlink()
    release(fd)


def _permits_claimed_ahead(pool: str, me: Path | None) -> int:
    """Permits claimed by live waiters that arrived BEFORE `me`, reaping orphans as it
    goes. Returns 0 when fairness is off or the queue is unreadable — i.e. every failure
    mode degrades to the old order-blind behavior rather than blocking an acquire."""
    if me is None:
        return 0
    qd = _queue_dir(pool)
    try:
        names = sorted(p.name for p in qd.iterdir() if not p.name.startswith("."))
    except OSError:
        return 0
    claimed = 0
    for name in names:
        if name >= me.name:  # sorted ⇒ everything from here on arrived after me
            break
        probe = qd / name
        try:
            pfd = os.open(str(probe), os.O_RDWR)
        except OSError:
            continue  # vanished under us — someone else reaped it
        try:
            try:
                fcntl.flock(pfd, fcntl.LOCK_EX | fcntl.LOCK_NB)
            except OSError:
                # Owner still holds it ⇒ a LIVE waiter ahead of us. Its `need` is the
                # trailing field; treat an unparseable name as 1 permit rather than 0, so
                # a malformed ticket can never be the reason we barge past someone.
                try:
                    claimed += int(name.rsplit("-", 1)[1])
                except (IndexError, ValueError):
                    claimed += 1
                continue
            # We took the lock ⇒ the owner is gone. Reap the orphan.
            with contextlib.suppress(OSError):
                probe.unlink()
        finally:
            os.close(pfd)
    return claimed


def _may_attempt(pool: str, me: Path | None, need: int, capacity: int) -> bool:
    """Only the ticket head may claim permits; active work remains concurrent.

    Comparing queued demand with TOTAL capacity allowed barging when running
    work already held some permits. Even a free-count snapshot would race other
    acquirers. Serialize admission, not execution: the successful acquirer drops
    its ticket immediately, letting the next waiter take any remaining permits.
    """
    if me is None:
        return True
    return _permits_claimed_ahead(pool, me) == 0


def _acquire_one(
    files: list[Path],
    timeout: float,
    meta: dict,
    *,
    pool: str | None = None,
) -> int | None:
    """Try to grab ANY of `files` (an exclusive flock). Returns the held fd, or None on
    timeout. Loops the candidate set so a lease of N permits picks the first free slot.

    `pool` names the FIFO queue to arrive in (None = no queueing: correct for the
    single-file `builddir` lock, which is mutual exclusion on one resource rather than a
    contended pool, so ordering buys nothing)."""
    HOSTLOCK_DIR.mkdir(parents=True, exist_ok=True)
    t0 = time.monotonic()
    deadline = t0 + timeout
    announced = False
    ticket = _ticket_take(pool, 1) if pool else None
    try:
        while True:
            if pool == "cpuheavy":
                buildpolicy.require_admission(POLICY_DIR)
            if _may_attempt(pool or "", ticket[1] if ticket else None, 1, len(files)):
                for p in files:
                    fd = os.open(str(p), os.O_RDWR | os.O_CREAT, 0o644)
                    try:
                        fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
                    except OSError:
                        os.close(fd)
                        continue
                    _write_meta(fd, dict(meta, wait_seconds=time.monotonic() - t0))
                    _report_shadow(pool)
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
                print(_wait_line(meta.get("kind", "?"), pool), flush=True)
                announced = True
            time.sleep(_POLL)
    finally:
        _ticket_drop(ticket)


def release(fd: int | None) -> None:
    """Release a lock (closing the fd also releases the flock; we unlock explicitly first)."""
    if fd is None:
        return
    policy_guard = buildpolicy.take_guard(fd)
    try:
        fcntl.flock(fd, fcntl.LOCK_UN)
    except OSError:
        pass
    try:
        os.close(fd)
    except OSError:
        pass
    if policy_guard is not None:
        policy_guard.__exit__(None, None, None)


def _cpu_lease_files(leases: int) -> list[Path]:
    return [HOSTLOCK_DIR / f"cpuheavy-{i:02d}" for i in range(leases)]


def _boot_lease_files(leases: int) -> list[Path]:
    return [HOSTLOCK_DIR / f"bootcpu-{i:02d}" for i in range(leases)]


def land_boot_reserve(leases: int = DEFAULT_BOOT_LEASES) -> int:
    """How many boot permits only a LANDING gate may take.

    Before this, a conductor's gate boots came out of the same pool as every lane's own
    smokes, payload runs and sweeps, so on a busy night the one job that unblocks the
    whole fleet queued behind the fleet. Half the pool by default -- one full gate's
    concurrent boots -- and never the whole of it, so lanes can always boot.
    VIBEOS_LAND_BOOT_RESERVE overrides (0 disables).
    """
    want = int(os.environ.get("VIBEOS_LAND_BOOT_RESERVE", str(leases // 2)))
    return max(0, min(want, leases - 1))


def is_landing_job() -> bool:
    """The conductor runs its gate with VIBEOS_JOB_CLASS=integration (queue.py), and every
    boot under it inherits that."""
    return job_class() == "integration"


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
    files = _boot_lease_files(leases)
    reserve = land_boot_reserve(leases)
    if is_landing_job():
        # Reserved permits first, then any other free one. Its OWN FIFO queue, so a
        # landing boot never waits for its turn behind lane boots.
        return _acquire_one(
            files[leases - reserve :] + files[: leases - reserve],
            timeout,
            _self_meta(kind, boot_pool=True, landing=True, **extra),
            pool="bootcpu-land",
        )
    return _acquire_one(
        files[: leases - reserve],
        timeout,
        _self_meta(kind, boot_pool=True, **extra),
        pool="bootcpu",
    )


@buildpolicy.guard
def acquire_cpu_lease(
    kind: str,
    *,
    leases: int = DEFAULT_CPU_LEASES,
    timeout: float = ACQUIRE_TIMEOUT,
    **extra: object,
) -> int | None:
    """Acquire one CPU-heavy lease permit (default pool size 1). Returns the held fd or
    None on timeout. `kind` is a short tag for host-status (e.g. 'build:i386', 'boot:arm64')."""
    buildpolicy.require_admission(POLICY_DIR)
    return _acquire_one(
        _cpu_lease_files(leases), timeout, _self_meta(kind, **extra), pool="cpuheavy"
    )


def _max_builds() -> int:
    """Host-sized build admission; retain explicit overrides and host policy."""
    cpus = len(os.sched_getaffinity(0)) if hasattr(os, "sched_getaffinity") else os.cpu_count()
    default = max(1, min(DEFAULT_CPU_LEASES, (cpus or 1) // 6))
    return max(1, int(os.environ.get("VIBEOS_MAX_CONCURRENT_BUILDS", str(default))))


def build_permits(leases: int = DEFAULT_CPU_LEASES) -> int:
    """Permits a build takes from the cpuheavy pool: ceil(K / max_builds). With the
    defaults (K=4, max 2 builds) that's 2 — so two builds coexist, a third queues, and
    while TWO builds run the pool is exhausted (native-TCG boots excluded, preserving
    the starvation protection the old writer-takes-all grant existed for)."""
    policy = buildpolicy.read(POLICY_DIR)
    if policy:
        return policy["leases"] // policy["max_builds"]
    return max(1, -(-leases // _max_builds()))


@buildpolicy.guard
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
    or None on timeout.

    Admission is FIFO (see "FIFO admission" above): we take an arrival-ordered ticket and
    only scan for permits once doing so cannot delay an earlier arrival. Without that, the
    release-all-and-retry loop below is order-blind and a patient waiter can lose every
    race — a build starved 23 minutes this way on 2026-07-24."""
    meta = _self_meta(kind, share=n, **extra)
    HOSTLOCK_DIR.mkdir(parents=True, exist_ok=True)
    t0 = time.monotonic()
    deadline = t0 + timeout
    announced = False
    ticket = _ticket_take("cpuheavy", n)
    try:
        while True:
            buildpolicy.require_admission(POLICY_DIR)
            if _may_attempt("cpuheavy", ticket[1] if ticket else None, n, leases):
                held: list[int] = []
                for p in _cpu_lease_files(leases):
                    fd = os.open(str(p), os.O_RDWR | os.O_CREAT, 0o644)
                    try:
                        fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
                    except OSError:
                        os.close(fd)
                        continue
                    _write_meta(fd, dict(meta, wait_seconds=time.monotonic() - t0))
                    held.append(fd)
                    if len(held) == n:
                        _report_shadow("cpuheavy")
                        if announced:
                            print(
                                f"{ACQUIRED_ANNOUNCE} ({kind}, n={n}) "
                                f"after {time.monotonic() - t0:.0f}s queued",
                                flush=True,
                            )
                        return held
                # Shortfall: release ALL (no hold-and-wait) and retry after a poll.
                for h in held:
                    release(h)
            if time.monotonic() >= deadline:
                return None
            if not announced:
                print(_wait_line(f"{kind}, n={n}", "cpuheavy"), flush=True)
                announced = True
            time.sleep(_POLL)
    finally:
        _ticket_drop(ticket)


@buildpolicy.guard
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
            try:
                buildpolicy.require_admission(POLICY_DIR)
            except RuntimeError:
                release_all(held)
                raise
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
def build_lock(build_dir: str | Path, arch: str, **kw: object) -> Iterator[dict]:
    """Serialize only this build directory; runnable tools acquire their own jobs.

    A caller waiting for another build of the same output takes no CPU capacity.
    The directory lock remains mandatory even when CPU throttling is disabled.
    """
    start = time.monotonic()
    with buildpolicy.session(POLICY_DIR), build_dir_lock(build_dir, arch=arch, **kw) as directory:
        buildpolicy.require_admission(POLICY_DIR)
        yield {
            "enabled": True,
            "scheduler": "host-jobs",
            "job_class": job_class(),
            "cpu_wait_s": 0.0,
            "builddir_wait_s": time.monotonic() - start,
            "cpu_granted": False,
            "builddir_granted": directory is not None,
        }


@contextmanager
def build_dir_lock(build_dir: str | Path, **kw: object) -> Iterator[int | None]:
    """Mandatory exclusive build-directory ownership, independent of CPU throttling."""
    fd = acquire_build_dir(build_dir, **kw)  # type: ignore[arg-type]
    if fd is None:
        raise buildpolicy.PolicyError("build-directory admission timed out; refusing unlocked work")
    try:
        yield fd
    finally:
        release(fd)


# ── the LAND lock (mutual exclusion over the whole land critical section) ────
#
# WHY (2026-07-25): the gate mints a push credential keyed to an exact TREE HASH, so the
# window between "gate green" and "push accepted" is a RACE. Whoever pushes first wins;
# every other in-flight lane's tree is now stale, its stamp is void, and it has to merge
# and RE-GATE. The loser didn't just lose wall-clock — its gate was 3 builds + 4 boots of
# fleet load that slowed everyone else's gate, which WIDENS the window that voided it. With
# more active lanes than the host can gate at once, that's a congestion collapse; it
# presents as "agents constantly waiting on the gate, racing to finish".
#
# FIX: landing is a critical section. `vibeos-cli land` holds this ONE lock across
# fetch → merge → gate → push, so master cannot move under a gating lane: a green gate
# ALWAYS lands, the fleet spends exactly ONE gate per landing instead of one per attempt,
# and the queue is FIFO + visible instead of a lottery a patient lane loses indefinitely.
#
# It reuses the pool machinery with capacity 1 (`_may_attempt` then admits only the queue
# head), so it inherits FIFO ordering and flock crash-safety for free.
#
# NOT gated on `enabled()`: the CPU pools are a *throughput* throttle you may switch off,
# but this is a *correctness* lock. VIBEOS_LAND_LOCK=0 opts out (single-agent / rescue).
LAND_LOCK_TIMEOUT = float(os.environ.get("VIBEOS_LAND_LOCK_TIMEOUT", "10800"))  # 3h


class LandLockBusy(RuntimeError):
    """Raised when the land lock can't be had inside the timeout. HARD failure by design:
    landing unlocked would reintroduce the exact race this lock exists to remove."""


def land_lock_enabled() -> bool:
    return os.environ.get("VIBEOS_LAND_LOCK", "1").lower() not in ("0", "false", "no", "")


def _land_lock_file() -> Path:
    return HOSTLOCK_DIR / "land"


def land_holder() -> dict:
    """Metadata of the current land-lock holder ({} when free) — for the 'who is landing
    right now' line, and to name the blocker when we give up waiting."""
    p = _land_lock_file()
    if not p.is_file() or not _is_held(p):
        return {}
    try:
        return json.loads(p.read_text() or "{}")
    except (OSError, ValueError):
        return {}


def land_queue() -> list[dict]:
    """Live landers queued for the lock, oldest arrival first."""
    return queue_status("land")


@contextmanager
def land_lock(kind: str, *, timeout: float = LAND_LOCK_TIMEOUT, **extra: object) -> Iterator[None]:
    """Hold the fleet-wide landing lock for the duration of the block.

    Raises LandLockBusy on timeout (naming the holder) — unlike the soft-failing CPU
    pools, this one must never degrade to "proceed unlocked". A timeout here means a LIVE
    holder has been landing for hours (the flock dies with its process, so there is no such
    thing as a stale one): look at `vibeos-cli host-status` and deal with that holder."""
    if not land_lock_enabled():
        yield None
        return
    HOSTLOCK_DIR.mkdir(parents=True, exist_ok=True)
    fd = _acquire_one(
        [_land_lock_file()], timeout, _self_meta(kind, land=True, **extra), pool="land"
    )
    if fd is None:
        h = land_holder()
        who = (
            f"pid={h.get('pid')} {h.get('kind', '?')} in {h.get('cwd', '?')}"
            if h
            else "unknown (it just released — retry)"
        )
        raise LandLockBusy(
            f"land lock busy for {timeout:.0f}s — holder: {who}. "
            "`vibeos-cli host-status` shows the queue; landing unlocked would re-open the "
            "gate→push race, so this is a hard stop."
        )
    try:
        yield None
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
        "land": [],
        "cpu_leases_total": DEFAULT_CPU_LEASES,
        "cpu_leases_held": 0,
        "boot_leases_total": DEFAULT_BOOT_LEASES,
        "boot_leases_held": 0,
    }
    if not HOSTLOCK_DIR.is_dir():
        return out  # no lock dir yet → nothing held (keys above stay at their defaults)
    for p in sorted(HOSTLOCK_DIR.glob("*")):
        if p.is_dir():
            continue  # queue-<pool>/ — reported separately via queue_status()
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
        elif p.name == "land":
            out["land"].append(entry)
    policy = buildpolicy.read(POLICY_DIR)
    out["build_policy"] = policy
    out["cpu_leases_total"] = policy["leases"] if policy else DEFAULT_CPU_LEASES
    out["cpu_leases_held"] = sum(1 for e in out["cpuheavy"] if e["held"])
    out["boot_leases_total"] = DEFAULT_BOOT_LEASES
    out["boot_leases_held"] = sum(1 for e in out["bootcpu"] if e["held"])
    out["queue"] = {p: queue_status(p) for p in ("land", "cpuheavy", "bootcpu")}
    out["admission_shadow"] = admission_shadow(out["queue"]["cpuheavy"])
    return out


def queue_status(pool: str) -> list[dict]:
    """Live waiters for `pool`, oldest arrival first. Read-only: unlike
    `_permits_claimed_ahead` this never reaps, so `host-status` can't perturb the queue
    it is reporting on. Orphan tickets (owner died) are dropped from the view."""
    qd = _queue_dir(pool)
    waiters: list[dict] = []
    try:
        names = sorted(p.name for p in qd.iterdir() if not p.name.startswith("."))
    except OSError:
        return waiters
    for name in names:
        if not _is_held(qd / name):
            continue  # orphan — the next acquirer reaps it
        parts = name.split("-")
        try:
            waited = max(0.0, time.time() - int(parts[0]) / 1e9)
        except (IndexError, ValueError):
            waited = 0.0
        meta = {}
        try:
            candidate = json.loads((qd / name).read_text())
            if isinstance(candidate, dict):
                meta = candidate
        except (OSError, ValueError):
            pass
        queued = meta.get("queued_monotonic")
        known_clock = False
        if isinstance(queued, float) and math.isfinite(queued) and 0 <= queued <= time.monotonic():
            known_clock = True
            waited = time.monotonic() - queued
        waiters.append(
            {
                "pid": parts[1] if len(parts) > 1 else "?",
                "need": int(parts[2]) if len(parts) > 2 and parts[2].isdigit() else 1,
                "waited_s": round(waited, 1),
                "job_class": meta.get("job_class", "unknown"),
                "clock": "monotonic" if known_clock else "legacy-wall",
            }
        )
    return waiters


def admission_shadow(waiters: list[dict]) -> dict:
    """Read-only candidate: give integration a bounded 120s head start.

    Older ordinary work wins once its age exceeds that head start. Unknown/legacy
    metadata suppresses the proposal, never silently classifies another lane.
    This is NOT admission policy; FIFO and every lock operation remain unchanged.
    """
    result = {"mode": "shadow", "enforced": False, "boost_s": 120, "changed": False}
    if (HOSTLOCK_DIR / "admission-shadow.disabled").exists():
        return dict(result, mode="disabled", reason="host-wide kill switch")
    if any(
        w.get("clock") != "monotonic"
        or w.get("job_class") not in ("interactive", "integration", "background")
        for w in waiters
    ):
        return dict(result, reason="unknown or legacy waiter metadata; FIFO only")
    order = sorted(
        range(len(waiters)),
        key=lambda i: (
            -(waiters[i]["waited_s"] + (120 if waiters[i]["job_class"] == "integration" else 0))
        ),
    )
    return dict(
        result,
        fifo=[w["pid"] for w in waiters],
        proposed=[waiters[i]["pid"] for i in order],
        changed=order != list(range(len(waiters))),
    )


def _report_shadow(pool: str | None) -> None:
    if pool == "cpuheavy":
        proposal = admission_shadow(queue_status(pool))
        print("[hostlock] admission-shadow " + json.dumps(proposal, sort_keys=True), flush=True)


def render_status(st: dict) -> str:
    """Human-readable host-broker summary (the `vibeos-cli host-status` body)."""
    lines = [
        f"host broker: {'ON' if st['enabled'] else 'OFF (VIBEOS_HOST_BROKER=0)'}  —  "
        f"legacy/native(cpuheavy) {st['cpu_leases_held']}/{st['cpu_leases_total']} · "
        f"boot(windows-host) {st.get('boot_leases_held', 0)}/{st.get('boot_leases_total', 0)}"
        + (
            f" ({land_boot_reserve(st.get('boot_leases_total', 0))} reserved for landing)"
            if st.get("boot_leases_total")
            else ""
        )
    ]
    held_cpu = [e for e in st["cpuheavy"] if e["held"]]
    if not held_cpu:
        lines.append("  cpuheavy : FREE — no legacy build / native TCG reservation")
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
    # The land lock: exactly one lane may be inside fetch→merge→gate→push at a time, so
    # "who is landing right now" answers most of "why is master moving under me".
    land_held = [e for e in st.get("land", []) if e["held"]]
    if land_held:
        h = land_held[0]["holder"]
        dead = "" if land_held[0]["holder_alive"] else "  ⚠STALE(holder gone)"
        lines.append(
            f"  LANDING  : {h.get('kind', '?')!s:<14} pid={h.get('pid', '?')}{dead}  "
            f"{h.get('cwd', '')}"
        )
    # The queue, oldest first. Starvation used to be invisible — it read as "my build is
    # slow", not "I am 3rd in line behind 20 minutes of waiting" (field manual §9.84).
    for pool, waiters in (st.get("queue") or {}).items():
        for i, w in enumerate(waiters):
            head = "NEXT" if i == 0 else f"#{i + 1}  "
            want = (
                "to land"
                if pool == "land"
                else f"needs {w['need']} permit{'s' if w['need'] > 1 else ''}"
            )
            lines.append(
                f"  queued   : {head} {pool:<9} pid={w['pid']} {want}, waiting {w['waited_s']:.0f}s"
            )
    skewed = [
        e
        for pool in ("cpuheavy", "bootcpu", "builddir")
        for e in st.get(pool, [])
        if e.get("protocol_skew")
    ]
    shadow = st.get("admission_shadow")
    if shadow:
        lines.append("  admission: FIFO active; " + json.dumps(shadow, sort_keys=True))
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
    from . import hostjobs

    st = status()
    st["jobs"] = hostjobs.status(repo)
    if getattr(args, "json", False):
        print(json.dumps(st, indent=2))
    else:
        print(render_status(st))
        print(hostjobs.render_status(st["jobs"]))
    return 0
