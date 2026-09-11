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

from . import cachecfg, hostlock

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


def filesystems(repo: Path) -> list[dict]:
    """Account each device once, including separate cache/temp/artifact volumes."""
    cache = cachecfg.make(repo)
    paths = [
        repo,
        cache.ccache_dir,
        cache.sccache_dir,
        Path(tempfile.gettempdir()),
        Path(os.environ.get("VIBEOS_ARTIFACT_ROOT", "/tmp/vibeos-cli")),
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
    result = {
        "policy_path": str(root() / "policy.json"),
        "policy": config,
        "filesystems": volumes,
        "leases": live,
        "coverage": "cooperating CLI build/tooling jobs only; not a physical quota",
        "automatic_retention": False,
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
                cp = subprocess.run(
                    ["du", "-s", "-B1", str(path)], capture_output=True, text=True, check=True
                )
                footprints.append({"path": str(path), "bytes": int(cp.stdout.split()[0])})
        result["footprints"] = footprints
    return result


def command(args, repo: Path) -> int:
    try:
        print(json.dumps(status(repo, sizes=args.sizes), indent=2))
        return 0
    except (AdmissionError, OSError, subprocess.SubprocessError) as exc:
        print(f"{FAILURE}: {exc}")
        return 2
