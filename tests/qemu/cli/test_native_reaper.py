"""Reaper guards over real, exclusively fixture-owned process identities."""

import json
import os
import signal
import subprocess
import sys
import threading
import time
from pathlib import Path

import pytest

from tools.qemu_infra_compat import DedicatedSession


def tick(pid):
    return int(Path(f"/proc/{pid}/stat").read_text().rsplit(")", 1)[1].split()[19])


class Owned:
    def __init__(self):
        # Private process group excludes the test runner, API server and siblings.
        self.proc = subprocess.Popen(
            [sys.executable, "-c", "import time;time.sleep(120)"], start_new_session=True
        )
        self.pid = self.proc.pid
        self.start = tick(self.pid)
        self.done = threading.Event()

        def collect():
            self.proc.wait()
            self.done.set()

        self.thread = threading.Thread(target=collect, daemon=True)
        self.thread.start()

    def stop(self):
        if not self.done.is_set():
            try:
                os.kill(self.pid, signal.SIGTERM)
            except ProcessLookupError:
                pass
        assert self.done.wait(5), f"fixture process {self.pid} did not stop"
        self.thread.join(timeout=1)


class Reaper:
    def __init__(self, tmp, monkeypatch):
        self.locks = tmp / "locks"
        self.runs = tmp / "runs"
        self.locks.mkdir()
        self.runs.mkdir()
        self.owned = []
        self.uid = "reaper-fixture-" + str(os.getpid())
        binary = tmp / "bin"
        binary.mkdir()
        ps = binary / "powershell.exe"
        ps.write_text(Path(__file__).with_name("fake_reaper_powershell.py").read_text())
        ps.chmod(0o700)
        self.calls = tmp / "uid.calls"
        monkeypatch.setenv("PATH", str(binary) + os.pathsep + os.environ.get("PATH", ""))
        monkeypatch.setenv("REAPER_UID", self.uid)
        monkeypatch.setenv("REAPER_CALLS", str(self.calls))
        monkeypatch.setenv("REAPER_LEFT", "0")
        self.env = {
            **os.environ,
            "VIBEOS_QEMU_LOCK_DIR": str(self.locks),
            "RUNS_DIR": str(self.runs),
        }

    def process(self):
        item = Owned()
        self.owned.append(item)
        return item

    def dead_owner(self):
        item = self.process()
        item.stop()
        return item.pid

    def lock(self, name, **info):
        path = self.locks / (name + ".json")
        path.write_text(json.dumps(info))
        return path

    def manifest(self, name, **info):
        path = self.runs / name / "manifest.json"
        path.parent.mkdir()
        path.write_text(json.dumps(info))
        return path

    def reap(self, dry=False):
        # A fresh native owner inherits the private PowerShell stub PATH, and
        # receives only these private ledgers. No ambient fleet scan can occur.
        with DedicatedSession() as session:
            return session.call("vm_reap_orphan_qemus", {"env": self.env, "dry_run": dry})

    def close(self):
        for item in self.owned:
            item.stop()


@pytest.fixture
def reaper(tmp_path, monkeypatch):
    fixture = Reaper(tmp_path, monkeypatch)
    try:
        yield fixture
    finally:
        fixture.close()


def test_reaper_never_touches_a_live_owner_within_its_cap(reaper):
    qemu = reaper.process()
    path = reaper.lock(
        "live",
        launcher_pid=os.getpid(),
        launcher_start_tick=tick(os.getpid()),
        qemu_pid=qemu.pid,
        qemu_start_tick=qemu.start,
        deadline=time.time() + 600,
    )
    assert reaper.reap() == []
    assert path.exists() and not qemu.done.is_set()
    assert not reaper.calls.exists()


def test_reaper_kills_past_the_cap_even_with_a_live_owner(reaper):
    qemu = reaper.process()
    path = reaper.lock(
        "overdue",
        launcher_pid=os.getpid(),
        launcher_start_tick=tick(os.getpid()),
        qemu_pid=qemu.pid,
        qemu_start_tick=qemu.start,
        deadline=time.time() - 1,
    )
    actions = reaper.reap()
    assert qemu.done.wait(5)
    assert not path.exists() and any("exceeded lifetime cap" in line for line in actions)


def test_reaper_kills_windows_host_vms_by_uid_even_when_the_shim_is_gone(reaper):
    shim = reaper.process()
    shim.stop()
    path = reaper.lock(
        "windows",
        launcher_pid=reaper.dead_owner(),
        qemu_pid=shim.pid,
        qemu_start_tick=shim.start,
        cmd=["qemu.exe", "-name", reaper.uid],
    )
    actions = reaper.reap()
    assert not path.exists() and actions
    assert reaper.calls.read_text().splitlines() == [reaper.uid]


def test_reaper_keeps_the_lock_when_the_kill_fails(reaper, monkeypatch):
    qemu = reaper.process()
    # Unknown Windows confirmation must retain ownership evidence and must not
    # proceed to kill even the fixture's Linux shim on an unproved host result.
    monkeypatch.setenv("REAPER_LEFT", "unknown")
    path = reaper.lock(
        "unconfirmed",
        launcher_pid=reaper.dead_owner(),
        qemu_pid=qemu.pid,
        qemu_start_tick=qemu.start,
        win_uid=reaper.uid,
    )
    actions = reaper.reap()
    assert path.exists() and not qemu.done.is_set()
    assert any(line.startswith("FAILED") and "unconfirmed" in line for line in actions)
    assert reaper.calls.read_text().splitlines() == [reaper.uid]


def test_reaper_leaves_a_fresh_unparseable_lock_alone(reaper):
    qemu = reaper.process()
    path = reaper.locks / "torn.json"
    path.write_text("{half a lo")
    assert reaper.reap() == []
    assert path.read_text() == "{half a lo" and not qemu.done.is_set()
    assert not reaper.calls.exists()


def test_reaper_dry_run_changes_nothing(reaper):
    qemu = reaper.process()
    path = reaper.lock(
        "dry",
        launcher_pid=reaper.dead_owner(),
        qemu_pid=qemu.pid,
        qemu_start_tick=qemu.start,
        win_uid=reaper.uid,
    )
    before = path.read_bytes()
    actions = reaper.reap(dry=True)
    assert actions and actions[0].startswith("DRY-RUN")
    assert path.read_bytes() == before and not qemu.done.is_set()
    assert not reaper.calls.exists()


def test_reaper_walks_debug_toolkit_manifests(reaper):
    orphan, owned, stopped, legacy = (reaper.process() for _ in range(4))
    dead = reaper.dead_owner()
    a = reaper.manifest("orphan", qemu_pid=orphan.pid, qemu_start_tick=orphan.start, owner_pid=dead)
    b = reaper.manifest(
        "owned",
        qemu_pid=owned.pid,
        qemu_start_tick=owned.start,
        owner_pid=os.getpid(),
        owner_start_tick=tick(os.getpid()),
    )
    c = reaper.manifest(
        "stopped",
        qemu_pid=stopped.pid,
        qemu_start_tick=stopped.start,
        owner_pid=dead,
        stopped_at="2026-09-26T00:00:00",
    )
    d = reaper.manifest(
        "legacy", qemu_pid=legacy.pid, qemu_start_tick=legacy.start, deadline=time.time() + 600
    )
    torn = reaper.runs / "torn"
    torn.mkdir()
    (torn / "manifest.json").write_text("{")
    unchanged = {p: p.read_bytes() for p in (b, c, d)}
    actions = reaper.reap()
    assert orphan.done.wait(5)
    assert json.loads(a.read_text())["stopped_at"]
    assert any("owner dead" in line for line in actions)
    assert all(not q.done.is_set() for q in (owned, stopped, legacy))
    assert all(p.read_bytes() == text for p, text in unchanged.items())
    assert (torn / "manifest.json").read_text() == "{"


def test_reaper_refuses_a_live_pid_without_its_start_tick_proof(reaper):
    qemu = reaper.process()
    path = reaper.lock("unproved", launcher_pid=reaper.dead_owner(), qemu_pid=qemu.pid)
    actions = reaper.reap()
    assert path.exists() and not qemu.done.is_set()
    assert any("lacks recorded start-tick proof" in line for line in actions)


def test_reaper_refuses_a_reused_pid_incarnation(reaper):
    qemu = reaper.process()
    path = reaper.lock(
        "wrong-incarnation",
        launcher_pid=reaper.dead_owner(),
        qemu_pid=qemu.pid,
        qemu_start_tick=qemu.start + 1,
    )
    actions = reaper.reap()
    assert path.exists() and not qemu.done.is_set()
    assert any("PID has been reused" in line for line in actions)
