"""Host-wide, crash-released disk admission. Never deletes artifacts.

Reservations bound estimated additional writes by cooperating CLI processes, not
physical allocation. Legacy lanes and external writers remain unaccounted for.
Policy is one host file, not a per-lane environment override.
"""

from __future__ import annotations

import contextlib
import fcntl
import json
import math
import os
import shutil
import subprocess
import tempfile
import uuid
from collections.abc import Iterator
from pathlib import Path

from . import cachecfg, fsutil, hostlock

GIB = 1024**3
# MEASURED, not guessed (2026-09-11, fleet stopped, sampled every 20s across a cold
# build): i386 and amd64 both PEAK at 7.44 GiB, and the ten arm64 build dirs on disk are
# 2.8-3.0 GiB. `build` was 32 -- 4.3x the real peak -- and that over-estimate halts the
# merge train, because a 3-arch gate submits three `build` reservations concurrently while
# the CPU broker only ever admits two: 20 floor + 3x32 = 116 GiB had to be free or the
# third arch took an instant DISK_ADMISSION_FAIL, which is INCONCLUSIVE, which stops the
# drain for every queued lane. At 12 the same gate needs 56 GiB.
#
# 12 rather than 8 because a reservation bounds estimated additional WRITES, and a
# cache-miss-heavy build also writes into the shared ccache/sccache on this same
# filesystem; the cold build measured here grew them by 0 (every object was already
# cached), so that component is unmeasured and 12 carries ~60% headroom over the observed
# peak to cover it.
#
# The 3-vs-2 asymmetry itself is deliberate and stays: `reserve()` takes disk while
# holding NO CPU permit, so a build queued for CPU holds its disk reservation. Inverting
# that would mean sitting on a scarce CPU permit while blocked on disk. Accuracy is what
# makes the asymmetry harmless, so if these numbers drift, re-measure -- don't re-order.
REQUESTS = {"build": 12, "check-tooling": 4, "gate": 0, "land": 0}
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
# reserve for what actually runs, not for what was launched. Safe to claim under a lock
# only because reserve() never BLOCKS -- it admits or raises immediately.
DEFERRED = frozenset({"build"})


class AdmissionError(RuntimeError):
    pass


def root() -> Path:
    return hostlock.HOSTLOCK_DIR / "disk"


def policy(*, initialize: bool = False) -> dict:
    path = root() / "policy.json"
    default = {"version": 1, "min_free_gib": 20, "requests_gib": dict(REQUESTS)}
    if not path.exists():
        if not initialize:
            return default
        # Called under admission.lock: the first writer pins a host policy;
        # later lane versions read it rather than choosing different defaults.
        with path.open("x") as stream:
            json.dump(default, stream)
            stream.flush()
            os.fsync(stream.fileno())
    try:
        value = json.loads(path.read_text())
        if not isinstance(value, dict) or set(value) != set(default) or value["version"] != 1:
            raise ValueError("expected version, min_free_gib, requests_gib")
        amounts = value["requests_gib"]
        if not isinstance(amounts, dict) or set(amounts) != set(REQUESTS):
            raise ValueError("requests_gib must name build, check-tooling, gate, land")
        for amount in [value["min_free_gib"], *amounts.values()]:
            if isinstance(amount, bool) or not isinstance(amount, (int, float)):
                raise ValueError("budgets must be finite numbers")
            if not math.isfinite(amount) or amount < 0:
                raise ValueError("budgets must be nonnegative and finite")
        if value["min_free_gib"] < 1 or any(amounts[k] <= 0 for k in ("build", "check-tooling")):
            raise ValueError("keep at least 1 GiB headroom and positive writer reservations")
        return value
    except (OSError, ValueError, TypeError, OverflowError) as exc:
        raise AdmissionError(f"invalid host disk policy {path}: {exc}") from exc


def backing_mounts() -> list[Path]:
    """Host volumes BACKING our filesystem, which `df /` cannot see.

    A WSL ext4 lives in a sparse .vhdx on a Windows volume, so the guest reports the
    VHDX CEILING while the real constraint is free space on the host volume underneath.
    On 2026-09-13 that gap ate the fleet: a 1 TiB ceiling over a C: with 620 GiB free,
    so seven concurrent lane builds filled the HOST while `df /` still showed room, and
    the disk did not survive it. Admission was not wrong, it was answering a different
    question than the one that mattered.

    Colon-separated in VIBEOS_DISK_BACKING_MOUNTS; unset means unarmed, which is the
    correct default for a bare-metal host that has no backing store. Deliberately NOT a
    policy.json key: that file's loader rejects an unknown key set, so adding one there
    would fail admission closed in every lane still running an older commit.
    """
    raw = os.environ.get("VIBEOS_DISK_BACKING_MOUNTS", "")
    return [Path(p) for p in raw.split(":") if p]


def filesystems(repo: Path) -> list[dict]:
    """Account each device once, including separate cache/temp/artifact volumes."""
    cache = cachecfg.make(repo)
    paths = [
        repo,
        cache.ccache_dir,
        cache.sccache_dir,
        Path(tempfile.gettempdir()),
        Path(os.environ.get("VIBEOS_ARTIFACT_ROOT", "/tmp/vibeos-cli")),
        # Each is a distinct st_dev, so the dedup below folds it in as one more volume
        # and every reservation/floor check then applies to it unchanged.
        *backing_mounts(),
    ]
    devices = {}
    for path in paths:
        path = path.resolve()
        while not path.exists():
            path = path.parent
        dev = str(path.stat().st_dev)
        devices[dev] = {
            "device": dev,
            "path": str(path),
            "free_bytes": shutil.disk_usage(path).free,
        }
    return list(devices.values())


@contextlib.contextmanager
def _locked() -> Iterator[None]:
    root().mkdir(parents=True, exist_ok=True)
    with (root() / "admission.lock").open("a+") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        yield


def _live(*, reap: bool) -> list[dict]:
    leases = []
    for path in sorted(root().glob("lease-*.json")):
        try:
            stream = path.open("r+")
        except FileNotFoundError:
            continue
        with stream:
            try:
                fcntl.flock(stream, fcntl.LOCK_EX | fcntl.LOCK_NB)
            except BlockingIOError:
                try:
                    meta = json.load(stream)
                    if (
                        not isinstance(meta, dict)
                        or meta.get("version") != 1
                        or not isinstance(meta["reservations"], dict)
                    ):
                        raise ValueError("invalid reservation metadata")
                    if any(type(n) is not int or n < 0 for n in meta["reservations"].values()):
                        raise ValueError("invalid reserved byte count")
                except (ValueError, KeyError, TypeError) as exc:
                    raise AdmissionError(f"unreadable live disk lease {path}: {exc}") from exc
                leases.append(meta)
            else:
                if reap:
                    path.unlink(missing_ok=True)
    return leases


def refuse_if_hopeless(repo: Path, kind: str) -> None:
    """Fail-fast half of a DEFERRED kind's admission: is the host too full for this at ALL?

    Deliberately ignores other lanes' live reservations, which is what makes it different
    from reserve(): contention is settled later, once the CPU broker says this build may
    actually run. Counting contention here is precisely the bug -- it refuses a build for
    space that a concurrent build is merely holding a claim on.

    Without this, deferring the reservation would mean a genuinely full host queues every
    build for a CPU permit just to refuse it after, turning an instant honest error into a
    slow one. The ENOSPC afternoon is why the floor exists; keep failing fast on it.
    """
    config = policy()
    budget = int(config["requests_gib"][kind] * GIB)
    floor = int(config["min_free_gib"] * GIB)
    for volume in filesystems(repo):
        if volume["free_bytes"] < floor + budget:
            raise AdmissionError(
                f"{kind}: {volume['path']} has {volume['free_bytes'] / GIB:.1f} GiB free; "
                f"need {floor / GIB:.1f} headroom + {budget / GIB:.1f} requested, before any "
                "other lane is counted. Use vibeos-cli disk-status; reclaim completed "
                "disposable outputs."
            )


@contextlib.contextmanager
def reserve(repo: Path, kind: str) -> Iterator[dict]:
    """Check + claim under one host lock. No CPU permits held while doing this.

    The full estimate remains reserved until exit (conservative as writes accrue).
    Failure returns immediately, never runs the requested command unprotected.
    """
    lease = None
    path = None
    admitted = False
    try:
        with _locked():
            config = policy(initialize=True)
            budget = int(config["requests_gib"][kind] * GIB)
            floor = int(config["min_free_gib"] * GIB)
            live = _live(reap=True)
            volumes = filesystems(repo)
            for volume in volumes:
                reserved = sum(item["reservations"].get(volume["device"], 0) for item in live)
                if volume["free_bytes"] < floor + reserved + budget:
                    raise AdmissionError(
                        f"{kind}: {volume['path']} has {volume['free_bytes'] / GIB:.1f} GiB free; "
                        f"need {floor / GIB:.1f} headroom + {reserved / GIB:.1f} live reservations "
                        f"+ {budget / GIB:.1f} requested. Use vibeos-cli disk-status; "
                        "reclaim completed disposable outputs or wait for reservations to finish."
                    )
            meta = {
                "version": 1,
                "pid": os.getpid(),
                "kind": kind,
                "worktree": str(repo.resolve()),
                "reservations": {v["device"]: budget for v in volumes},
            }
            path = root() / f"lease-{uuid.uuid4().hex}.json"
            lease = path.open("x+")
            fcntl.flock(lease, fcntl.LOCK_EX)
            json.dump(meta, lease)
            lease.flush()
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
    config = policy()
    live = _live(reap=False)
    volumes = filesystems(repo)
    for volume in volumes:
        volume["reserved_bytes"] = sum(x["reservations"].get(volume["device"], 0) for x in live)
        volume["unreserved_above_floor_bytes"] = max(
            0, volume["free_bytes"] - volume["reserved_bytes"] - int(config["min_free_gib"] * GIB)
        )
    retention_policy = hostlock.HOSTLOCK_DIR / "retention.json"
    timer_active = False
    if retention_policy.exists() and not (hostlock.HOSTLOCK_DIR / "maintenance-disabled").exists():
        timer_active = (
            subprocess.run(
                ["systemctl", "--user", "is-active", "--quiet", "vibeos-retention.timer"],
                capture_output=True,
                timeout=5,
            ).returncode
            == 0
        )
    result = {
        "policy_path": str(root() / "policy.json"),
        "policy": config,
        "filesystems": volumes,
        "leases": live,
        "coverage": "cooperating CLI build/tooling jobs only; not a physical quota",
        # Say ARMED/UNARMED positively. An absent guard that reports nothing reads
        # identically to a guard that found nothing to worry about.
        "backing_mounts": [str(p) for p in backing_mounts()]
        or "UNARMED: set VIBEOS_DISK_BACKING_MOUNTS",
        "automatic_retention": timer_active,
        "retention_policy_path": str(retention_policy),
    }
    if sizes:
        cache = cachecfg.make(repo)
        paths = [
            cache.ccache_dir,
            cache.sccache_dir,
            repo / "sdk/lib/rust/target",
            *(repo / f"build-{a}" for a in ("i386", "amd64", "arm64")),
        ]
        footprints = []
        for path in paths:
            if path.is_dir():
                footprints.append({"path": str(path), "bytes": fsutil.disk_usage_bytes(path)})
        result["footprints"] = footprints
    return result


def command(args, repo: Path) -> int:
    try:
        print(json.dumps(status(repo, sizes=args.sizes), indent=2))
        return 0
    except (AdmissionError, OSError, subprocess.SubprocessError) as exc:
        print(f"{FAILURE}: {exc}")
        return 2
