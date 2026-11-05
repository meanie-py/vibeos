"""`smoke --attach-on-fail`: a failed boot's VM is handed to the debug toolkit, not killed.

The flag was parsed and read by nothing; a QEMU merely left running would also have been
lost (its lock unlinked at exit) or reaped (its owner, the smoke process, dead). These pin
the hand-over: who keeps the VM, under which owner, and that every OTHER path still tears
it down.
"""

from __future__ import annotations

import json

from tests.qemu.cli import gdbengine
from tools import qemu_infra_compat as native

# ---- _boot_vm: which outcomes keep the VM ----------------------------------------


def test_a_failed_verdict_keeps_the_vm_with_the_flag(tmp_path):
    out = _attach_fixture(tmp_path)
    assert not out["result"]["ok"]
    assert sum(c["op"] == "vm_detach" for c in out["calls"]) == 1
    assert not any(c["op"] == "vm_stop" for c in out["calls"])
    assert out["run"]["extra"]["attach"]["kept"] is True


def test_without_the_flag_a_failure_still_tears_down(tmp_path):
    out = _attach_fixture(tmp_path, attach=False)
    assert not out["result"]["ok"]
    assert not any(c["op"] == "vm_detach" for c in out["calls"])
    assert sum(c["op"] == "vm_stop" for c in out["calls"]) == 1


def test_a_harness_exception_is_not_a_guest_state_worth_keeping(tmp_path):
    # An invalid requested regex is a harness error after launch, not a guest verdict.
    out = _attach_fixture(tmp_path, fail_on="(", capture_error=True)
    assert "invalid fail-on expression" in out["error"]
    assert out["result"] is None
    assert not any(c["op"] == "vm_detach" for c in out["calls"])
    assert sum(c["op"] == "vm_stop" for c in out["calls"]) == 1


# ---- the hand-over record ---------------------------------------------------------


def test_the_kept_vm_gets_a_manifest_every_debug_tool_reads(tmp_path, monkeypatch):
    monkeypatch.setattr(gdbengine, "RUNS_DIR", tmp_path / "qemu/runs")
    handles = {
        "qemu_pid": 4242,
        "deadline": 2_000_000_000,
        "lock": str(tmp_path / "999-4242.json"),
        "gdb_host": "127.0.0.1",
        "gdb_port": 1234,
        "win_uid": "",
        "staging": "",
    }
    rid = "20261009-x-smoke-amd64"
    out = _attach_fixture(
        tmp_path,
        run_name=rid,
        responses={
            "debug_owner": [{"owner_pid": 999, "kernel_image": "fixture-kernel.exe"}],
            "vm_detach": [handles],
        },
    )
    loaded = gdbengine.DebugRun.load(rid)
    assert loaded.gdb_port == 1234 and loaded.qemu_pid == 4242
    raw = json.loads((tmp_path / "qemu/runs" / rid / "manifest.json").read_text())
    assert raw["owner_pid"] == 999 and raw["deadline"] == 2_000_000_000
    assert out["run"]["extra"]["attach"]["kept"]
    assert out["run"]["extra"]["attach"]["run_id"] == rid
    assert [s["name"] for s in out["run"]["stages"]] == ["attach"]
    assert rid in out["run"]["stages"][0]["detail"]


def test_repeat_runs_name_each_kept_boot(tmp_path):
    out = _attach_fixture(tmp_path, idx=2, total=5)
    assert out["run"]["extra"]["attach"]["run_id"] == "R-boot2"
    assert (tmp_path / "qemu/runs/R-boot2/manifest.json").is_file()


def test_a_vm_that_cannot_be_kept_is_stopped_and_said_so(tmp_path):
    out = _attach_fixture(
        tmp_path,
        responses={"vm_detach": [{"error": "cannot keep a VM whose QEMU has already exited"}]},
    )
    assert sum(c["op"] == "vm_stop" for c in out["calls"]) == 1
    assert out["run"]["extra"]["attach"]["kept"] is False
    assert "already exited" in out["run"]["extra"]["attach"]["reason"]


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


def test_the_fail_marker_named_is_the_pattern_that_fired(tmp_path):
    pattern = r"nvme\.sys"
    boot_marker = "[EXPLORER] _tWinMain entered"
    out = native.call(
        "smoke_fixture",
        {
            "mode": "one_boot",
            "chunks": ["loading nvme.sys\n"],
            "config": {
                "boot_marker": boot_marker,
                "boot_timeout": 1,
                "hang_silence_seconds": 0,
                "desktop_settle_seconds": 0,
                "required_milestones": [],
            },
            "options": {
                "arch": "amd64",
                "run_dir": str(tmp_path / "run"),
                "attach": False,
                "fail_on": pattern,
                "launch_retry_seconds": 0,
            },
        },
        root=tmp_path,
    )
    assert out["result"]["classification"] == "FAIL_MARKER"
    call = next(c for c in out["calls"] if c["op"] == "verdict_classify")
    assert call["kwargs"]["fail_marker"] == pattern
    assert any(pattern in line for line in out["result"]["salient"])
    assert not any(boot_marker in line for line in out["result"]["salient"])


def _attach_fixture(
    repo,
    *,
    attach=True,
    responses=None,
    run_name="R",
    idx=0,
    total=1,
    fail_on="BOOM",
    capture_error=False,
):
    return native.call(
        "smoke_fixture",
        {
            "mode": "one_boot",
            "capture_error": capture_error,
            "chunks": ["BOOM\n"],
            "options": {
                "arch": "amd64",
                "run_dir": str(repo / run_name),
                "attach": attach,
                "fail_on": fail_on,
                "idx": idx,
                "total": total,
                "launch_retry_seconds": 0,
            },
            "responses": responses or {},
        },
        root=repo,
    )
