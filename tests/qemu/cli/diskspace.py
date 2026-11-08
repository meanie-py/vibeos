"""Host-wide, crash-released disk admission. Never deletes artifacts.

Reservations bound estimated additional writes by cooperating CLI processes, not
physical allocation. Legacy lanes and external writers remain unaccounted for.
Policy is one host file, not a per-lane environment override.
"""

from __future__ import annotations

import contextlib
import fcntl
import json
import os
import subprocess
import tempfile
import time
import uuid
from collections.abc import Iterator
from pathlib import Path
from typing import TextIO

from tools import build_infra_compat as native

from . import cachecfg, hostlock

# The decisions are native: tools/vibeos-build/src/diskspace.rs (policy, volumes, live leases,
# the hopeless/contention verdicts, status). This module holds the two flocks that must live
# as long as the command -- admission.lock around one claim, and the lease it leaves -- and
# measures the volumes it hands the native side, so a test can stand in for the host.

GIB = 1024**3
# Per-kind write estimates in GiB, the defaults pinned into the host policy. MEASURED, and
# owned by diskspace.rs `REQUESTS` (with the measurement and why 12 not 32): one copy, so a
# re-measure cannot leave this side disagreeing.
REQUESTS: dict[str, int] = native.call("disk_requests", {})
FAILURE = "DISK_ADMISSION_FAIL"

# Kinds that claim their budget inside the command rather than at process start, so a
# build that never runs is never billed. The original wedge: a gate launched one build
# per arch, the cpuheavy pool admitted two, and the third took an instant refusal for
# space nothing would ever use -- 3x32 + 4 + 20 = 120 GiB needed against 109 free, no
# lane at fault, and because DISK_ADMISSION_FAIL is INCONCLUSIVE the conductor bisected
# for a culprit that did not exist while the drain stalled for every queued lane.
#
# Two later fixes ate most of that: the estimate is a measured 12 now, not 32, and CPU
# admission moved to the per-compiler-job broker, so whole builds are no longer gated on
# a pool of two. What survives is the invariant, which is worth keeping on its own terms:
# reserve for what actually runs, not for what was launched. reserve() may wait for other
# jobs' reservations to clear, but never inside the admission lock and never holding a CPU
# permit, so a waiting claim blocks nobody else's.
DEFERRED = frozenset({"build"})


class AdmissionError(RuntimeError):
    pass


class ContentionError(AdmissionError):
    """Short of space only because other jobs hold live reservations: space that frees
    itself when they finish, unlike a full disk."""


def _call(operation: str, **arguments):
    try:
        return native.call("disk_" + operation, arguments)
    except native.NativeError as exc:
        cls = ContentionError if exc.kind == "contention" else AdmissionError
        raise cls(str(exc)) from exc


def root() -> Path:
    return hostlock.HOSTLOCK_DIR / "disk"


def policy() -> dict:
    return _call("policy", root=root())


# Host volumes BACKING our filesystem (a dynamic WSL .vhdx's Windows volume), which `df /`
# cannot see. One path per line; the file wins over VIBEOS_DISK_BACKING_MOUNTS. See
# diskspace.rs `backing_mounts` for the 2026-09-13 loss that put it here.
BACKING_FILE = Path("/etc/vibeos/disk-backing-mounts")


def backing_mounts() -> list[Path]:
    return [
        Path(p)
        for p in _call("backing_mounts", backing_file=BACKING_FILE, environment=dict(os.environ))
    ]


def filesystems(repo: Path) -> list[dict]:
    """Account each device once, including separate cache/temp/artifact/backing volumes."""
    return _call(
        "filesystems",
        repo=repo,
        environment=dict(os.environ),
        tmpdir=tempfile.gettempdir(),
        backing_file=BACKING_FILE,
    )


@contextlib.contextmanager
def _locked() -> Iterator[None]:
    root().mkdir(parents=True, exist_ok=True)
    with (root() / "admission.lock").open("a+") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        yield


def refuse_if_hopeless(repo: Path, kind: str) -> None:
    """Fail-fast half of a DEFERRED kind's admission: is the host too full for this at ALL?
    Ignores other lanes' live reservations -- contention is settled once the CPU broker says
    this build may actually run."""
    _call("refuse_if_hopeless", root=root(), kind=kind, volumes=filesystems(repo))


# How long a contended claim waits for other jobs' reservations to clear. A reservation is
# held for the life of a build (minutes), so a refusal was almost always premature: on
# 2026-10-11 every build in one lane was refused for an hour with 99 GiB free and 72 GiB
# claimed by six other lanes' builds, and the only recourse was to re-run by hand.
WAIT_SECONDS = 1800
POLL_SECONDS = 15


def _wait_seconds() -> float:
    raw = os.environ.get("VIBEOS_DISK_WAIT_SECONDS")
    try:
        return max(0.0, float(raw)) if raw is not None else WAIT_SECONDS
    except ValueError as exc:
        raise AdmissionError(f"VIBEOS_DISK_WAIT_SECONDS={raw!r} is not a number") from exc


def _claim(repo: Path, kind: str) -> tuple[dict, Path, TextIO]:
    """One admission attempt under the host lock: the lease, or ContentionError/AdmissionError."""
    with _locked():
        granted = _call("claim", root=root(), kind=kind, volumes=filesystems(repo))
        meta = {
            "version": 1,
            "pid": os.getpid(),
            "kind": kind,
            "worktree": str(repo.resolve()),
            "reservations": granted["reservations"],
        }
        path = root() / f"lease-{uuid.uuid4().hex}.json"
        lease = path.open("x+")
        fcntl.flock(lease, fcntl.LOCK_EX)
        json.dump(meta, lease)
        lease.flush()
        return meta, path, lease


@contextlib.contextmanager
def reserve(repo: Path, kind: str) -> Iterator[dict]:
    """Check + claim under one host lock. No CPU permits held while doing this.

    The full estimate remains reserved until exit (conservative as writes accrue). A disk
    too full for this job even with nobody else's claim fails at once. One that is short
    only because of other jobs' live reservations waits, with the lock released, for up to
    VIBEOS_DISK_WAIT_SECONDS (default 1800), then fails the same way. Never runs the
    command unprotected.
    """
    lease = None
    path = None
    admitted = False
    try:
        deadline = time.monotonic() + _wait_seconds()
        said = False
        while True:
            try:
                meta, path, lease = _claim(repo, kind)
                break
            except ContentionError as exc:
                if time.monotonic() >= deadline:
                    raise
                if not said:
                    print(
                        f"disk admission: waiting for other jobs' reservations ({exc})", flush=True
                    )
                    said = True
                time.sleep(POLL_SECONDS)
        admitted = True
        yield meta
    except OSError as exc:
        if admitted:
            raise
        raise AdmissionError(f"disk reservation unavailable: {exc}") from exc
    finally:
        if lease is not None:
            # Unlink while held, so a reaper cannot mistake a live reservation
            # for an orphan. A killed process instead releases its flock.
            try:
                if path is not None:
                    path.unlink(missing_ok=True)
            finally:
                lease.close()


def status(repo: Path, *, sizes: bool = False) -> dict:
    footprints = None
    if sizes:
        cache = cachecfg.make(repo)
        footprints = [
            cache.ccache_dir,
            cache.sccache_dir,
            repo / "sdk/lib/rust/target",
            *(repo / f"build-{a}" for a in ("i386", "amd64", "arm64")),
        ]
    return _call(
        "status",
        root=root(),
        hostlock=hostlock.HOSTLOCK_DIR,
        volumes=filesystems(repo),
        backing_file=BACKING_FILE,
        environment=dict(os.environ),
        footprints=footprints,
    )


def command(args, repo: Path) -> int:
    try:
        print(json.dumps(status(repo, sizes=args.sizes), indent=2))
        return 0
    except (AdmissionError, OSError, subprocess.SubprocessError) as exc:
        print(f"{FAILURE}: {exc}")
        return 2
