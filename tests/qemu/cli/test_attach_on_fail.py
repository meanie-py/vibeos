"""`smoke --attach-on-fail`: a failed boot's VM is handed to the debug toolkit, not killed.

The flag was parsed and read by nothing; a QEMU merely left running would also have been
lost (its lock unlinked at exit) or reaped (its owner, the smoke process, dead). These pin
the hand-over: who keeps the VM, under which owner, and that every OTHER path still tears
it down.
"""

from __future__ import annotations

import json
import time
from pathlib import Path
from types import SimpleNamespace
from typing import Any, cast

import pytest

from tests.qemu import vm as vm_mod
from tests.qemu.cli import gdbengine, smoke


class _Closable:
    def __init__(self) -> None:
        self.closed = False

    def close(self) -> None:
        self.closed = True

    def cancel(self) -> None:
        self.closed = True


class _Proc:
    def __init__(self, pid: int, alive: bool = True) -> None:
        self.pid = pid
        self._alive = alive

    def poll(self):
        return None if self._alive else 0


def _bare_vm(tmp_path: Path, monkeypatch, *, alive: bool = True) -> vm_mod.QemuVM:
    """A QemuVM with only the state detach_for_debug/stop touch -- no QEMU launched."""
    monkeypatch.setattr(vm_mod, "LOCK_DIR", tmp_path / "locks")
    (tmp_path / "locks").mkdir()
    vm = object.__new__(vm_mod.QemuVM)
    v = cast("Any", vm)
    v._proc = _Proc(4242, alive)
    v._watchdog = _Closable()
    v.serial = _Closable()
    v.qmp = _Closable()
    v._echo_servers = []
    v.config = SimpleNamespace(arch="amd64-bios")
    v._backend = "native"
    v.gdb_port = 1234
    v._win_uid = ""
    v._win_staging = None
    old = tmp_path / "locks" / "777-4242.json"
    old.write_text(json.dumps({"launcher_pid": 777, "qemu_pid": 4242, "cmd": ["qemu", "-x"]}))
    v._lock_path = old
    v._lock_atexit = lambda: None
    return vm


def test_detach_reregisters_the_lock_under_the_debugging_session(tmp_path, monkeypatch):
    unregistered: list[object] = []
    monkeypatch.setattr(vm_mod.atexit, "unregister", unregistered.append)
    vm = _bare_vm(tmp_path, monkeypatch)
    hook = cast("Any", vm)._lock_atexit
    before = time.time()
    h = vm.detach_for_debug(owner_pid=31337, run_id="RID")

    new = tmp_path / "locks" / "31337-4242.json"
    info = json.loads(new.read_text())
    assert info["launcher_pid"] == 31337 and info["qemu_pid"] == 4242
    # Every field the launcher wrote survives: the reaper finds a windows-host VM's
    # uid in `cmd`, so dropping it would make the kept VM unkillable by the reaper.
    assert info["cmd"] == ["qemu", "-x"]
    assert info["run_id"] == "RID" and info["source"] == "smoke --attach-on-fail"
    assert info["deadline"] >= int(before + vm_mod.MAX_VM_LIFETIME_SECONDS)
    assert not (tmp_path / "locks" / "777-4242.json").exists()
    assert unregistered == [hook], "the old atexit unlink must not fire at exit"
    assert h["lock"] == str(new) and h["gdb_port"] == 1234 and h["qemu_pid"] == 4242


def test_detach_stops_the_watchdog_and_closes_our_sockets(tmp_path, monkeypatch):
    monkeypatch.setattr(vm_mod.atexit, "unregister", lambda f: None)
    vm = _bare_vm(tmp_path, monkeypatch)
    v = cast("Any", vm)
    wd, ser, qmp = v._watchdog, v.serial, v.qmp
    vm.detach_for_debug(owner_pid=1, run_id="R")
    assert wd.closed and ser.closed and qmp.closed and v._watchdog is None


def test_after_detach_stop_cannot_kill_the_kept_vm(tmp_path, monkeypatch):
    """A stray stop() must be a no-op on the process -- including the windows-host
    kill-by-uid, which does not look at _proc at all."""
    monkeypatch.setattr(vm_mod.atexit, "unregister", lambda f: None)
    vm = _bare_vm(tmp_path, monkeypatch)
    v = cast("Any", vm)
    v._backend = "windows-host"
    v._win_uid = "vibeos-1-2"
    v._win_staging = tmp_path / "staging"
    vm.detach_for_debug(owner_pid=1, run_id="R")
    assert v._proc is None and v._win_uid == "" and v._win_staging is None


@pytest.mark.parametrize("left", [None, 1])
def test_stop_keeps_the_lock_when_the_windows_kill_is_unconfirmed(tmp_path, monkeypatch, left):
    """stop() dropped the lock after a kill that PowerShell never confirmed. When the
    interop launch of powershell.exe failed, qemu.exe kept running with no record
    anywhere; nine such VMs were found on 2026-10-11, two running for eight days."""
    from tests.qemu import backend as be

    unregistered: list[object] = []
    monkeypatch.setattr(vm_mod.atexit, "unregister", unregistered.append)
    monkeypatch.setattr(be, "kill_by_uid", lambda uid: left)
    vm = _bare_vm(tmp_path, monkeypatch, alive=False)
    v = cast("Any", vm)
    v._backend = "windows-host"
    v._win_uid = "vibeos-1-2"
    v._proc = None
    v._sock_dir = tmp_path / "socks"
    v._release_slot = lambda: None
    lock, hook = v._lock_path, v._lock_atexit
    vm.stop()
    assert lock.exists(), "the lock is the reaper's only record of the qemu.exe"
    assert unregistered == [hook], "its atexit unlink must not fire either"


def test_stop_drops_the_lock_once_the_windows_kill_is_confirmed(tmp_path, monkeypatch):
    from tests.qemu import backend as be

    monkeypatch.setattr(be, "kill_by_uid", lambda uid: 0)
    vm = _bare_vm(tmp_path, monkeypatch, alive=False)
    v = cast("Any", vm)
    v._backend = "windows-host"
    v._win_uid = "vibeos-1-2"
    v._proc = None
    v._sock_dir = tmp_path / "socks"
    v._release_slot = lambda: None
    lock = v._lock_path
    vm.stop()
    assert not lock.exists()


def test_a_dead_qemu_is_not_kept(tmp_path, monkeypatch):
    vm = _bare_vm(tmp_path, monkeypatch, alive=False)
    with pytest.raises(RuntimeError, match="already exited"):
        vm.detach_for_debug(owner_pid=1, run_id="R")


# ---- _boot_vm: which outcomes keep the VM ----------------------------------------


class _FakeVM:
    def __init__(self) -> None:
        self.stopped = 0

    def stop(self) -> None:
        self.stopped += 1

    def read_debugcon(self) -> bytes:
        return b""

    def debugcon_report(self):
        return None

    def report_replay_break(self) -> str:
        return ""


def _drive(tmp_path, monkeypatch, *, attach: bool, watch, classify=None, fail_on="BOOM"):
    fake = _FakeVM()
    kept: list[str] = []
    monkeypatch.setattr(smoke, "launch_with_retry", lambda *a, **k: (fake, 1, None))
    monkeypatch.setattr(smoke, "_watch", watch)
    monkeypatch.setattr(smoke, "_screenshot", lambda *a, **k: "")
    monkeypatch.setattr(
        smoke.triage,
        "classify",
        classify or (lambda *a, **k: {"classification": "X", "salient": []}),
    )
    monkeypatch.setattr(smoke.triage, "symbolicate_pc", lambda *a, **k: "")
    monkeypatch.setattr(smoke, "_keep_for_debug", lambda run, vm, *a: kept.append("kept"))
    run = SimpleNamespace(dir=tmp_path, log=lambda *a, **k: None, extra={}, add=lambda st: None)
    repo = Path(__file__).resolve().parents[3]
    res = smoke._boot_vm(
        cast("Any", run), repo, "amd64", until="", fail_on=fail_on, idx=0, total=1, attach=attach
    )
    return res, fake, kept


def test_a_failed_verdict_keeps_the_vm_with_the_flag(tmp_path, monkeypatch):
    res, fake, kept = _drive(
        tmp_path, monkeypatch, attach=True, watch=lambda *a: ("failmarker", "BOOM", {})
    )
    assert not res["ok"] and kept == ["kept"] and fake.stopped == 0


def test_without_the_flag_a_failure_still_tears_down(tmp_path, monkeypatch):
    res, fake, kept = _drive(
        tmp_path, monkeypatch, attach=False, watch=lambda *a: ("failmarker", "BOOM", {})
    )
    assert not res["ok"] and kept == [] and fake.stopped == 1


def test_a_harness_exception_is_not_a_guest_state_worth_keeping(tmp_path, monkeypatch):
    fake = _FakeVM()
    monkeypatch.setattr(smoke, "launch_with_retry", lambda *a, **k: (fake, 1, None))

    def boom(*a):
        raise ValueError("harness bug")

    monkeypatch.setattr(smoke, "_watch", boom)
    kept: list[str] = []
    monkeypatch.setattr(smoke, "_keep_for_debug", lambda *a: kept.append("kept"))
    run = SimpleNamespace(dir=tmp_path, log=lambda *a, **k: None, extra={}, add=lambda st: None)
    with pytest.raises(ValueError):
        smoke._boot_vm(
            cast("Any", run),
            Path(__file__).resolve().parents[3],
            "amd64",
            until="",
            fail_on=None,
            idx=0,
            total=1,
            attach=True,
        )
    assert kept == [] and fake.stopped == 1


# ---- the hand-over record ---------------------------------------------------------


def test_the_kept_vm_gets_a_manifest_every_debug_tool_reads(tmp_path, monkeypatch):
    monkeypatch.setattr(gdbengine, "RUNS_DIR", tmp_path / "runs")
    monkeypatch.setenv("VIBEOS_QEMU_OWNER_PID", "999")
    handles = {
        "qemu_pid": 4242,
        "deadline": 2_000_000_000,
        "lock": str(tmp_path / "999-4242.json"),
        "gdb_host": "127.0.0.1",
        "gdb_port": 1234,
        "win_uid": "",
        "staging": "",
    }
    vm = SimpleNamespace(_backend="native", detach_for_debug=lambda owner, rid: handles)
    added: list = []
    run = SimpleNamespace(
        dir=tmp_path / "20261009-x-smoke-amd64",
        log=lambda *a, **k: None,
        extra={},
        add=added.append,
    )
    smoke._keep_for_debug(cast("Any", run), cast("Any", vm), "amd64", tmp_path, 0, 1, "boot")

    rid = "20261009-x-smoke-amd64"
    loaded = gdbengine.DebugRun.load(rid)  # the gdbengine schema: `gdb <verb> <run-id>`
    assert loaded.gdb_port == 1234 and loaded.qemu_pid == 4242
    raw = json.loads((tmp_path / "runs" / rid / "manifest.json").read_text())
    # The reaper's manifest walk keys on these two.
    assert raw["owner_pid"] == 999 and raw["deadline"] == 2_000_000_000
    assert run.extra["attach"]["kept"] and run.extra["attach"]["run_id"] == rid
    # ...and the bounded summary says so: a kept VM nobody notices runs to its deadline.
    assert [st.name for st in added] == ["attach"] and rid in added[0].detail


def test_repeat_runs_name_each_kept_boot(tmp_path, monkeypatch):
    monkeypatch.setattr(gdbengine, "RUNS_DIR", tmp_path / "runs")
    monkeypatch.setenv("VIBEOS_QEMU_OWNER_PID", "1")
    handles = dict.fromkeys(("lock", "gdb_host", "win_uid", "staging"), "") | {
        "qemu_pid": 1,
        "deadline": 1,
        "gdb_port": 0,
    }
    vm = SimpleNamespace(_backend="native", detach_for_debug=lambda o, r: handles)
    run = SimpleNamespace(
        dir=tmp_path / "R", log=lambda *a, **k: None, extra={}, add=lambda st: None
    )
    smoke._keep_for_debug(cast("Any", run), cast("Any", vm), "amd64", tmp_path, 2, 5, "b")
    assert run.extra["attach"]["run_id"] == "R-boot2"


def test_a_vm_that_cannot_be_kept_is_stopped_and_said_so(tmp_path, monkeypatch):
    monkeypatch.setenv("VIBEOS_QEMU_OWNER_PID", "1")
    stopped: list[int] = []

    def refuse(owner, rid):
        raise RuntimeError("cannot keep a VM whose QEMU has already exited")

    vm = SimpleNamespace(detach_for_debug=refuse, stop=lambda: stopped.append(1))
    run = SimpleNamespace(
        dir=tmp_path / "R", log=lambda *a, **k: None, extra={}, add=lambda st: None
    )
    smoke._keep_for_debug(cast("Any", run), cast("Any", vm), "amd64", tmp_path, 0, 1, "b")
    assert stopped == [1] and run.extra["attach"]["kept"] is False


def test_gdb_stop_unregisters_the_kept_vms_lock(tmp_path, monkeypatch):
    """Read before run.save(): save rewrites the manifest with the dataclass fields
    only, which drops "lock" -- reading it afterwards found nothing."""
    monkeypatch.setattr(gdbengine, "RUNS_DIR", tmp_path / "runs")
    lock = tmp_path / "1-2.json"
    lock.write_text("{}")
    serial = tmp_path / "serial.log"
    serial.write_text("x")
    run = gdbengine.DebugRun(
        run_id="R",
        arch="amd64",
        backend="native",
        gdb_host="127.0.0.1",
        gdb_port=1,
        serial_log=str(serial),
        kernel_image="k",
    )
    run.dir.mkdir(parents=True)
    run.manifest_path().write_text(json.dumps({**gdbengine.asdict(run), "lock": str(lock)}))
    gdbengine.stop(run)
    assert not lock.exists()


def test_the_fail_marker_named_is_the_pattern_that_fired(tmp_path, monkeypatch):
    """It named the BOOT marker: `--fail-on 'nvme\\.sys'` reported "hit fail marker:
    [EXPLORER] _tWinMain entered" for a run that never got near explorer."""
    seen: dict = {}

    def classify(*a, **k):
        seen.update(k)
        return {"classification": "FAIL_MARKER", "salient": []}

    _drive(
        tmp_path,
        monkeypatch,
        attach=False,
        watch=lambda *a: ("failmarker", "BOOM", {}),
        classify=classify,
        fail_on=r"nvme\.sys",
    )
    assert seen["fail_marker"] == r"nvme\.sys"
