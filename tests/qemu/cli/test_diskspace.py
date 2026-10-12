"""Disk capacity is admission state, not a build correctness verdict."""

import json
import select
import subprocess
import sys
from pathlib import Path

import pytest

from . import __main__ as cli
from . import diskspace as disk
from . import gate, hostlock, land, queue
from .artifacts import Run

#: Room for exactly ONE build (plus the check-tooling job these tests nest inside it) and
#: not two. DERIVED from the policy rather than hardcoded, because the tests assert "the
#: SECOND build is refused" and a bare constant stops expressing that the moment
#: REQUESTS["build"] is re-measured: 60 GiB said "one build" against a 32 GiB estimate and
#: "four builds" against a 12 GiB one, so the tests failed for a reason that was not a
#: regression. The assert pins the property the number has to keep having.
ONE_BUILD_FREE = (20 + disk.REQUESTS["build"] + disk.REQUESTS["check-tooling"] + 1) * disk.GIB
assert (20 + 2 * disk.REQUESTS["build"]) * disk.GIB > ONE_BUILD_FREE, (
    "free space must admit one build and refuse two; re-check REQUESTS"
)


@pytest.fixture
def isolated(tmp_path, monkeypatch):
    monkeypatch.setattr(hostlock, "HOSTLOCK_DIR", tmp_path / "locks")
    # Contention refuses at once here; the waiting tests below opt back in.
    monkeypatch.setenv("VIBEOS_DISK_WAIT_SECONDS", "0")
    monkeypatch.setattr(
        disk,
        "filesystems",
        lambda repo: [{"device": "1", "path": str(repo), "free_bytes": ONE_BUILD_FREE}],
    )
    return tmp_path


def test_reserve_and_release_never_deletes_outputs(isolated):
    paid = isolated / "paid.rs"
    paid.write_text("paid translation")
    with disk.reserve(isolated, "build"):
        assert (
            disk.status(isolated)["filesystems"][0]["reserved_bytes"]
            == disk.REQUESTS["build"] * disk.GIB
        )
        with (
            pytest.raises(disk.AdmissionError, match="live reservations"),
            disk.reserve(isolated, "build"),
        ):
            pytest.fail("overcommitted disk")
        with disk.reserve(isolated, "check-tooling"):
            assert len(disk.status(isolated)["leases"]) == 2
    assert not disk.status(isolated)["leases"]
    assert paid.read_text() == "paid translation"


def test_reserve_releases_on_body_failure(isolated):
    with pytest.raises(FileNotFoundError), disk.reserve(isolated, "build"):
        raise FileNotFoundError("not an admission failure")
    assert not disk.status(isolated)["leases"]


@pytest.mark.parametrize(
    "bad",
    [
        "{}",
        "null",
        "{",
        '{"version":2}',
        '{"version":1,"min_free_gib":0,"requests_gib":{"build":32,"check-tooling":4,"gate":0,"land":0}}',
    ],
)
def test_invalid_policy_fails_closed(isolated, bad):
    disk.root().mkdir(parents=True)
    (disk.root() / "policy.json").write_text(bad)
    with pytest.raises(disk.AdmissionError), disk.reserve(isolated, "build"):
        pytest.fail("invalid policy accepted")


def test_policy_is_host_wide_and_reloaded(isolated):
    disk.root().mkdir(parents=True)
    config = disk.policy()
    config["min_free_gib"] = 50
    (disk.root() / "policy.json").write_text(json.dumps(config))
    with pytest.raises(disk.AdmissionError), disk.reserve(isolated, "build"):
        pytest.fail("policy ignored")
    config["min_free_gib"] = 20
    (disk.root() / "policy.json").write_text(json.dumps(config))
    with disk.reserve(isolated, "build"):
        pass


def test_cli_refuses_before_build_or_artifact_creation(isolated, monkeypatch, capsys):
    monkeypatch.setattr(cli, "REPO", isolated)
    monkeypatch.setattr(
        disk, "filesystems", lambda repo: [{"device": "1", "path": str(repo), "free_bytes": 0}]
    )
    monkeypatch.setattr(cli.build_mod, "build", lambda *a: pytest.fail("build ran"))
    assert cli.main(["build", "--arch", "arm64"]) == 2
    message = capsys.readouterr().out
    assert gate._child_classification(message) == disk.FAILURE
    assert disk.FAILURE in queue.INCONCLUSIVE


def test_tooling_propagates_capacity_classification(isolated, monkeypatch):
    monkeypatch.setattr(
        gate.subprocess,
        "run",
        lambda *a, **kw: subprocess.CompletedProcess(a[0], 2, "DISK_ADMISSION_FAIL: full\n", ""),
    )
    result = gate._tooling_once(isolated, isolated / "artifacts")
    assert not result["ok"]
    assert result["classification"] == disk.FAILURE


def test_land_preflight_keeps_actionable_disk_reason(isolated, monkeypatch):
    from argparse import Namespace

    monkeypatch.setattr(
        land.subprocess,
        "run",
        lambda *a, **kw: subprocess.CompletedProcess(
            a[0], 2, "DISK_ADMISSION_FAIL: needs headroom\n", ""
        ),
    )
    run = Run("test-disk-land", "all")
    with pytest.raises(land.LandAbort, match="needs headroom"):
        land._lint_preflight(isolated, run, Namespace(skip_lint=False))
    assert run.classification == disk.FAILURE


def test_filesystems_count_same_device_once(tmp_path):
    volumes = disk.filesystems(tmp_path)
    assert len({v["device"] for v in volumes}) == len(volumes)
    assert all(v["free_bytes"] >= 0 for v in volumes)


def test_status_is_read_only_and_foreign_live_metadata_fails_closed(isolated):
    assert disk.status(isolated)["leases"] == []
    assert not disk.root().exists()
    disk.root().mkdir(parents=True)
    import fcntl

    path = disk.root() / "lease-foreign.json"
    with path.open("w+") as lease:
        fcntl.flock(lease, fcntl.LOCK_EX)
        lease.write('{"version":2,"reservations":{}}')
        lease.flush()
        with (
            pytest.raises(disk.AdmissionError, match="unreadable live"),
            disk.reserve(isolated, "build"),
        ):
            pytest.fail("unknown live protocol ignored")
    # After the owner exits even corrupt metadata is an orphan, not a blocker.
    with disk.reserve(isolated, "build"):
        assert not path.exists()


def test_two_processes_cannot_overreserve_and_crash_releases(isolated):
    # Independent interpreters race the real host lock on a private test root.
    # The winner stays alive until explicitly killed; no external process touched.
    code = r"""
import sys
from pathlib import Path
from tests.qemu.cli import diskspace as d, hostlock
hostlock.HOSTLOCK_DIR = Path(sys.argv[1]) / 'locks'
d.filesystems = lambda repo: [{'device':'1', 'path':str(repo),
                               'free_bytes':(20+d.REQUESTS['build']+d.REQUESTS['check-tooling']+1)*d.GIB}]
try:
    with d.reserve(Path(sys.argv[1]), 'build'):
        print('ADMITTED', flush=True)
        sys.stdin.readline()
except d.AdmissionError:
    print('REFUSED', flush=True)
"""
    children = [
        subprocess.Popen(
            [sys.executable, "-c", code, str(isolated)],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        for _ in range(2)
    ]
    try:
        outcomes = []
        for child in children:
            assert child.stdout is not None
            assert select.select([child.stdout], [], [], 15)[0], "reservation deadlocked"
            outcomes.append(child.stdout.readline().strip())
        assert sorted(outcomes) == ["ADMITTED", "REFUSED"]
        winner = children[outcomes.index("ADMITTED")]
        winner.kill()
        winner.wait(timeout=10)
        with disk.reserve(isolated, "build"):
            assert len(disk.status(isolated)["leases"]) == 1
    finally:
        for child in children:
            if child.poll() is None:
                child.kill()
            child.communicate(timeout=10)


def test_a_build_is_not_billed_for_concurrency_the_broker_will_never_allow(isolated, monkeypatch):
    """The wedge: three launched builds, two permits, and a refusal for unusable space."""
    monkeypatch.setattr(cli, "REPO", isolated)
    # One build already holds a claim, so a second cannot fit floor + live + budget.
    with disk.reserve(isolated, "build"):
        # Start-of-process admission must NOT count it: whether this build may run at all
        # is the CPU broker's call, and reserving at launch is what billed three builds
        # for a pool that admits two.
        disk.refuse_if_hopeless(isolated, "build")
        # Contention is still enforced -- just later, when the permit is actually held.
        with (
            pytest.raises(disk.AdmissionError, match="live reservations"),
            disk.reserve(isolated, "build"),
        ):
            pytest.fail("the second claim must not be granted")


def test_a_hopeless_host_still_refuses_before_the_build_runs(isolated, monkeypatch):
    """Deferring must not turn an instant honest refusal into a slow one."""
    monkeypatch.setattr(cli, "REPO", isolated)
    monkeypatch.setattr(
        disk, "filesystems", lambda repo: [{"device": "1", "path": str(repo), "free_bytes": 0}]
    )
    monkeypatch.setattr(cli.build_mod, "build", lambda *a: pytest.fail("build ran"))
    assert cli.main(["build", "--arch", "arm64"]) == 2


def test_an_admission_refusal_raised_deep_inside_a_command_still_classifies(isolated, monkeypatch):
    """build claims disk after the CPU permit, so its AdmissionError starts inside args.fn."""
    monkeypatch.setattr(cli, "REPO", isolated)

    def refuse(*_a, **_kw):
        raise disk.AdmissionError("build: no room at permit time")

    monkeypatch.setattr(cli.build_mod, "build", refuse)
    assert cli.main(["build", "--arch", "arm64"]) == 2


def test_build_defers_and_the_other_kinds_do_not(isolated):
    assert "build" in disk.DEFERRED
    assert set(disk.REQUESTS) >= disk.DEFERRED
    assert "check-tooling" not in disk.DEFERRED, "only a kind that waits on a CPU permit defers"


def test_backing_store_is_admitted_as_its_own_volume(tmp_path, monkeypatch):
    """A WSL guest reports its VHDX CEILING, not the host free space behind it.

    The 2026-09-13 loss: a 1 TiB ext4 ceiling over a C: with 620 GiB free, so concurrent
    lane builds filled the HOST while `df /` still showed room. Admission never saw the
    volume that ran out. Guard is opt-in because a bare-metal host has no backing store,
    so the test pins both states — unarmed must not invent a volume, armed must add one.
    """
    monkeypatch.delenv("VIBEOS_DISK_BACKING_MOUNTS", raising=False)
    assert disk.backing_mounts() == []
    before = {v["device"] for v in disk.filesystems(tmp_path)}

    monkeypatch.setenv("VIBEOS_DISK_BACKING_MOUNTS", f"{tmp_path}:/nonexistent-drive")
    assert disk.backing_mounts() == [Path(tmp_path), Path("/nonexistent-drive")]
    after = disk.filesystems(tmp_path)
    # Deduped by st_dev like every other path, and a floor/reservation applies to each.
    assert {v["device"] for v in after} >= before
    assert all(v["free_bytes"] >= 0 for v in after)


def test_a_contended_claim_waits_for_the_other_lease_to_release(isolated, monkeypatch):
    """Short only because another job holds a claim: wait for it, don't refuse."""
    import threading

    monkeypatch.setenv("VIBEOS_DISK_WAIT_SECONDS", "10")
    monkeypatch.setattr(disk, "POLL_SECONDS", 0.05)
    holder_in, release = threading.Event(), threading.Event()

    def hold():
        with disk.reserve(isolated, "build"):
            holder_in.set()
            release.wait(10)

    holder = threading.Thread(target=hold)
    holder.start()
    assert holder_in.wait(10)
    threading.Timer(0.3, release.set).start()
    with disk.reserve(isolated, "build"):
        assert release.is_set(), "admitted while the other claim was still live"
    holder.join(10)


def test_a_full_disk_refuses_at_once_even_when_waiting_is_allowed(isolated, monkeypatch):
    """Nobody's claim to wait for: the space is simply not there."""
    import time

    monkeypatch.setenv("VIBEOS_DISK_WAIT_SECONDS", "600")
    monkeypatch.setattr(
        disk,
        "filesystems",
        lambda repo: [{"device": "1", "path": str(repo), "free_bytes": 5 * disk.GIB}],
    )
    started = time.monotonic()
    with pytest.raises(disk.AdmissionError) as info, disk.reserve(isolated, "build"):
        pytest.fail("a full disk admitted a build")
    assert not isinstance(info.value, disk.ContentionError)
    assert time.monotonic() - started < 5
