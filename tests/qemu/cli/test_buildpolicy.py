import fcntl
import json
from types import SimpleNamespace

import pytest

from . import artifacts, build, buildpolicy, hostlock, queue


@pytest.fixture
def trial(tmp_path, monkeypatch):
    monkeypatch.setattr(hostlock, "HOSTLOCK_DIR", tmp_path)
    # POLICY_DIR is a SEPARATE dir now (durable config vs ephemeral locks), so it needs
    # its own patch -- without it these tests read the real host policy and assert
    # against whatever capacity this box happens to be running.
    monkeypatch.setattr(hostlock, "POLICY_DIR", tmp_path)
    monkeypatch.setenv("VIBEOS_HOSTLOCK_FAIR", "0")
    monkeypatch.setenv("VIBEOS_HOST_BROKER", "1")
    (tmp_path / "build-policy.json").write_text(json.dumps(buildpolicy.TRIAL))
    return tmp_path


def test_the_trial_policy_sets_capacity_and_ninja_fanout(trial):
    # Build admission itself is host-jobs' (tools/test_host_jobs.py); what reads the
    # policy here is host-status's capacity and ninja's job count.
    assert hostlock.status()["cpu_leases_total"] == 6
    assert build._ninja_jobs() == ["-j", "6"]


def _hold(path, *, shared: bool = False):
    """A live client as native admission holds it: a cpuheavy permit while running, the
    policy lock shared while queued."""
    stream = path.open("a+")
    fcntl.flock(stream, (fcntl.LOCK_SH if shared else fcntl.LOCK_EX) | fcntl.LOCK_NB)
    return stream


def test_live_holder_prevents_cutover(trial, monkeypatch):
    monkeypatch.setattr(buildpolicy, "blockers", lambda repo: [])
    with _hold(trial / "cpuheavy-00"), pytest.raises(RuntimeError, match="active or queued"):
        buildpolicy.activate(trial, trial, rollback=True)
    monkeypatch.setattr(buildpolicy.os, "cpu_count", lambda: 24)
    buildpolicy.activate(trial, trial, rollback=True)
    policy = buildpolicy.read(trial)
    assert policy is not None and policy["leases"] == 4


def test_mixed_fleet_refused_without_changing_policy(trial, monkeypatch):
    monkeypatch.setattr(buildpolicy, "blockers", lambda repo: ["old client"])
    before = (trial / "build-policy.json").read_bytes()
    with pytest.raises(RuntimeError, match="old client"):
        buildpolicy.activate(trial, trial)
    assert (trial / "build-policy.json").read_bytes() == before


def test_invalid_policy_fails_loudly(trial):
    (trial / "build-policy.json").write_text('{"version": 1}')
    with pytest.raises(RuntimeError, match="invalid host"):
        hostlock.status()


@pytest.mark.parametrize("env", [None, {"CARGO_BUILD_JOBS": "24", "RUSTC_WRAPPER": "cache"}])
def test_cargo_fanout_bound_with_or_without_cache(trial, monkeypatch, env):
    monkeypatch.setenv("CARGO_BUILD_JOBS", "24")
    configured = build._policy_build_env(env)
    assert configured is not None and configured["CARGO_BUILD_JOBS"] == "1"
    if env is not None:
        assert configured["RUSTC_WRAPPER"] == "cache"
        assert env["CARGO_BUILD_JOBS"] == "24"


def test_queued_client_pins_policy_before_getting_a_permit(trial, monkeypatch):
    # A queued native client holds the policy lock shared from before it takes a ticket
    # until its permit is released (vibeos-test admission.rs), so a cutover cannot
    # change the capacity it is waiting against.
    monkeypatch.setattr(buildpolicy, "blockers", lambda repo: [])
    with (
        _hold(trial / "build-policy.lock", shared=True),
        pytest.raises(RuntimeError, match="active or queued"),
    ):
        buildpolicy.activate(trial, trial)


def test_active_policy_cannot_disable_broker(trial, monkeypatch):
    monkeypatch.setenv("VIBEOS_HOST_BROKER", "0")
    with pytest.raises(RuntimeError, match="requires the CPU broker"):
        hostlock.enabled()


def test_pause_refusal_is_infrastructure_not_a_lane_regression(trial, monkeypatch):
    monkeypatch.setattr(artifacts, "ARTIFACT_ROOT", trial / "runs")
    (trial / "build-arm64").mkdir()
    (trial / "build-policy.paused").touch()
    assert build.build(SimpleNamespace(arch="arm64", clean=False), trial) == 1
    summary = json.loads(next((trial / "runs").glob("*/summary.json")).read_text())
    assert summary["classification"] == "BUILD_POLICY_REFUSED"
    assert summary["classification"] in queue.INCONCLUSIVE


def test_policy_survives_the_lock_tree_being_wiped(trial, monkeypatch, tmp_path):
    """The reboot bug: /tmp is cleared, and the cap silently stopped existing."""
    (trial / "build-policy.json").write_text(json.dumps(buildpolicy.TRIAL))
    assert build._ninja_jobs() == ["-j", "6"]
    # Simulate the reboot: the EPHEMERAL lock tree goes away, config does not.
    wiped = tmp_path / "wiped-locks"
    wiped.mkdir()
    monkeypatch.setattr(hostlock, "HOSTLOCK_DIR", wiped)
    assert build._ninja_jobs() == ["-j", "6"], "capacity must not depend on the lock tree"


def test_heal_seeds_the_conservative_example_not_the_trial(tmp_path):
    buildpolicy.heal(tmp_path)
    seeded = buildpolicy.read(tmp_path)
    assert seeded is not None
    # An unattended box that lost its policy comes back SLOW, never fast.
    assert seeded != buildpolicy.TRIAL
    assert seeded["max_builds"] * seeded["ninja_jobs"] < (
        buildpolicy.TRIAL["max_builds"] * buildpolicy.TRIAL["ninja_jobs"]
    )


def test_heal_never_overwrites_an_applied_policy(tmp_path):
    (tmp_path / "build-policy.json").write_text(json.dumps(buildpolicy.TRIAL))
    buildpolicy.heal(tmp_path)
    assert buildpolicy.read(tmp_path) == buildpolicy.TRIAL


def test_the_tracked_example_passes_the_validator():
    """A malformed example would fail admission closed on every fresh clone."""
    example = buildpolicy.repo_root() / buildpolicy.EXAMPLE_NAME
    assert example.is_file(), "the example is the seed for self-heal; it must be tracked"
    assert buildpolicy.read(example.parent.resolve() / "does-not-exist") is None
    parsed = json.loads(example.read_text())
    assert parsed["max_builds"] * parsed["ninja_jobs"] <= 4, "example must be safe on a small host"


def test_every_worktree_resolves_to_one_shared_policy(tmp_path):
    """Eight lanes with eight capacities is not a cap."""
    main = tmp_path / "main"
    (main / ".git" / "worktrees" / "lane").mkdir(parents=True)
    lane = tmp_path / "lane"
    lane.mkdir()
    (lane / ".git").write_text(f"gitdir: {main / '.git' / 'worktrees' / 'lane'}\n")
    assert buildpolicy.main_worktree(lane) == main
    assert buildpolicy.main_worktree(main) == main


def test_mirror_keeps_v1_lanes_capped_during_the_transition(tmp_path, monkeypatch):
    """A lane on old tooling reads HOSTLOCK_DIR and nothing else.

    If moving the file simply stopped writing there, every un-merged lane would find no
    policy and take the UNCAPPED branch -- this change would have caused the very crash
    it prevents.
    """
    locks, durable = tmp_path / "locks", tmp_path / "durable"
    locks.mkdir()
    durable.mkdir()
    monkeypatch.setattr(hostlock, "HOSTLOCK_DIR", locks)
    monkeypatch.setattr(hostlock, "POLICY_DIR", durable)
    buildpolicy.heal(durable)
    assert buildpolicy.read(durable) is not None
    # What a v1 lane would read:
    assert buildpolicy.read(locks) == buildpolicy.read(durable)


def test_stale_worktrees_do_not_block_the_cutover(trial, monkeypatch):
    """Advisory, not a blocker: a paused un-merged lane must not pin the whole host
    to the conservative seed, because mirror() keeps that lane capped anyway."""
    monkeypatch.setattr(buildpolicy, "stale_worktrees", lambda repo: ["/some/old/lane"])
    monkeypatch.setattr(buildpolicy, "blockers", lambda repo: [])
    monkeypatch.setattr(buildpolicy.os, "cpu_count", lambda: 24)
    buildpolicy.activate(trial, trial)
    assert buildpolicy.read(trial) == buildpolicy.TRIAL


def test_native_cache_discovery_uses_callers_execute_permission(tmp_path):
    import os
    import shutil

    from tools import build_infra_compat as native

    tool = tmp_path / "ccache"
    tool.write_text("#!/bin/sh\nexit 0\n")
    # An execute bit for another permission class is insufficient for this owner.
    tool.chmod(0o010)
    plan = native.call(
        "build_policy_cache_make", {"repo": tmp_path, "environment": {"PATH": str(tmp_path)}}
    )
    assert plan["ccache_bin"] == shutil.which("ccache", path=str(tmp_path))
    tool.chmod(0o700)
    plan = native.call(
        "build_policy_cache_make", {"repo": tmp_path, "environment": {"PATH": str(tmp_path)}}
    )
    assert plan["ccache_bin"] == str(tool) and os.access(tool, os.X_OK)
