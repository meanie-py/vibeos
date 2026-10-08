"""Local conftest for the vibeos-cli unit tests.

The parent tests/qemu/conftest.py has an autouse `restore_vm` fixture that pulls
in the session `vm` fixture — which boots QEMU. These CLI unit tests are pure
(no hypervisor), so shadow `restore_vm` with a no-op autouse fixture (nearer
scope wins) to keep them fast and hermetic.
"""

import pytest


@pytest.fixture(autouse=True)
def restore_vm():
    yield


@pytest.fixture(autouse=True)
def _artifact_root_is_never_the_real_one(tmp_path, monkeypatch):
    """No unit test may write into the LIVE artifact tree.

    artifacts.ARTIFACT_ROOT is resolved from the environment at IMPORT time, so a
    test that does `monkeypatch.setenv("VIBEOS_ARTIFACT_ROOT", ...)` changes
    nothing and its Run() lands in the real root. Under `check-tooling` — which
    the gate runs with VIBEOS_ARTIFACT_ROOT pointed at the gate's own tooling dir
    — that meant test_run_summary_json deposited a synthetic
    `20231114-171320-smoke-i386/summary.json` carrying verdict FAIL /
    classification BUGCHECK directly into the gate's artifact tree. It is the
    first summary.json a `find` turns up, it looks exactly like a real verdict,
    and it has now twice been read as one, including in a report to the user.

    Pinning the root per-test kills the whole class instead of the four call
    sites that happened to hit it. Tests that patch ARTIFACT_ROOT themselves
    still win: their monkeypatch applies after this one."""
    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path / "artifacts")


@pytest.fixture(autouse=True)
def _conductors_drain_in_process(monkeypatch):
    """No unit test may pin a conductor checkout or spawn a drain child from one.

    An elected conductor hands its drain to a checkout of origin/master
    (`queue._conduct_pinned`): a `git worktree add` beside the repo plus a real
    `vibeos-cli land --drain` child. While it was being written, a land test that only
    meant to check `--foreground` did exactly that (inside its tmp dir, luckily).
    Tests that exercise the handoff unset this themselves."""
    monkeypatch.setenv("VIBEOS_CONDUCTOR_PIN", "0")


@pytest.fixture(autouse=True)
def _land_lock_opt_out_does_not_leak_in(monkeypatch):
    """The land-lock tests assert the lock EXCLUDES; they must not inherit the opt-out.

    VIBEOS_LAND_LOCK=0 is the documented single-agent/rescue switch, and it reaches every
    child of a land run with it -- including the gate's own check-tooling. There it made
    land_lock() a no-op, six "must raise LandLockBusy" tests failed, and the queue declared
    origin/master red (TOOLING_FAIL, 2026-10-03) and held the line on a tree that passes."""
    monkeypatch.delenv("VIBEOS_LAND_LOCK", raising=False)
