"""Checkpoint guards: no fresh-boot fallback, shared writable files or false pass."""

import json
from types import SimpleNamespace
from typing import Any

import pytest

from . import checkpoint as cp


def test_template_keeps_topology_clock_and_devices_but_replaces_transports():
    args = [
        "qemu",
        "-S",
        "-name",
        "old",
        "-qmp",
        "old.sock",
        "-smp",
        "4",
        "-rtc",
        "base=utc,clock=vm",
        "-accel",
        "tcg,thread=multi",
        "-device",
        "virtio-blk-device,drive=scratch",
        "-drive",
        "file=C:/temp/scratch.qcow2,format=qcow2,if=none,id=scratch",
    ]
    template, media = cp.guest_template(args)
    assert "old.sock" not in template
    assert template[:6] == ["-smp", "4", "-rtc", "base=utc,clock=vm", "-accel", "tcg,thread=multi"]
    assert media[0]["source"] == "/mnt/c/temp/scratch.qcow2"
    assert "@media-00.qcow2" in template[-1]


@pytest.mark.parametrize(
    "extra",
    [
        ["-netdev", "user,id=n"],
        ["-accel", "whpx"],
        ["-fsdev", "local,path=/host"],
        ["-fw_cfg", "name=test,file=/hidden"],
        ["-device", "usb-storage,drive=x"],
        ["-drive", "file=/writable.raw,format=raw"],
    ],
)
def test_unknown_external_state_and_accelerators_are_refused(extra):
    with pytest.raises(cp.CheckpointError):
        cp.guest_template(["qemu", "-drive", "file=/scratch,format=qcow2", *extra])


def test_hmp_lowercase_error_is_fatal():
    qmp: Any = SimpleNamespace(
        execute=lambda *a: {"return": {"running": False, "status": "paused"}},
        hmp=lambda _: "Device cannot be saved",
    )
    with pytest.raises(cp.CheckpointError, match="cannot be saved"):
        vm: Any = SimpleNamespace(qmp=qmp)
        cp.snapshot(vm, "loadvm")


def test_restore_checks_registers_before_allowing_execution(monkeypatch):
    monkeypatch.setattr(cp, "snapshot", lambda *a: None)
    monkeypatch.setattr(cp, "registers", lambda _: "PC=2222")
    with pytest.raises(cp.CheckpointError, match="registers differ"):
        vm: Any = SimpleNamespace(manifest={"registers": "PC=1111"})
        cp.restore(vm)


@pytest.mark.parametrize("state", ["paused", "prelaunch", "debug"])
def test_capture_accepts_a_real_debugger_stop(state):
    vm: Any = SimpleNamespace(
        qmp=SimpleNamespace(
            execute=lambda *a: {"return": {"running": False, "status": state}},
        )
    )
    cp.pause(vm)


def test_capture_refuses_internal_error_state():
    vm: Any = SimpleNamespace(
        qmp=SimpleNamespace(
            execute=lambda *a: {"return": {"running": False, "status": "internal-error"}},
        )
    )
    with pytest.raises(cp.CheckpointError, match="did not stop cleanly"):
        cp.pause(vm)


def test_failure_wins_over_success_in_same_chunk(tmp_path):
    vm: Any = SimpleNamespace(serial_output=lambda **kw: "SUCCESS\nBUGCHECK 0x1e")
    assert cp.watch(vm, "SUCCESS", cp.FAIL_PATTERN, 1, tmp_path / "log") == "failure-pattern"


def test_timeout_uses_monotonic_not_wall_clock(tmp_path, monkeypatch):
    times = iter([10, 10, 10, 12])
    monkeypatch.setattr(cp.time, "monotonic", lambda: next(times))
    monkeypatch.setattr(cp.time, "time", lambda: pytest.fail("wall clock used for deadline"))
    vm: Any = SimpleNamespace(serial_output=lambda **kw: "")
    assert cp.watch(vm, "SUCCESS", cp.FAIL_PATTERN, 1, tmp_path / "log") == "timeout"


def test_writable_attempt_is_private_and_seed_remains_unchanged(tmp_path):
    bundle = tmp_path / "bundle"
    bundle.mkdir()
    seed = bundle / "media-00.qcow2"
    seed.write_bytes(b"saved")
    work = tmp_path / "attempt"
    work.mkdir()
    vm = object.__new__(cp.RestoredVM)
    vm.bundle = bundle
    vm.manifest = {
        "media": [{"name": seed.name, "sha256": cp.digest(seed), "readonly": False}],
        "argv": ["-drive", "file=@media-00.qcow2,format=qcow2"],
    }
    argv = vm._guest_args(work, False)
    assert str(work / seed.name) in argv[1]
    (work / seed.name).write_bytes(b"changed")
    assert seed.read_bytes() == b"saved"


def test_corruption_refused_before_emulator_or_launch(tmp_path, monkeypatch):
    (tmp_path / "media-00.qcow2").write_bytes(b"damaged")
    (tmp_path / "manifest.json").write_text(
        json.dumps(
            {
                "schema": 1,
                "accel": "tcg",
                "backend": "windows-host",
                "media": [{"name": "media-00.qcow2", "sha256": "wrong"}],
            }
        )
    )
    monkeypatch.setattr(cp, "emulator_identity", lambda _: pytest.fail("guard bypassed"))
    with pytest.raises(cp.CheckpointError, match="damaged"):
        cp.load(tmp_path)
