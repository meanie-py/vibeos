"""Native launches expose actual stderr, GDB argv, and Rust-controlled retries."""

import json
from pathlib import Path

import pytest

from tests.qemu import vm as vm_module
from tools.qemu_infra_compat import DedicatedSession, NativeError

from .test_native_vm import create
from .test_native_vm import fake_config as _fake_config

fake_config = _fake_config


@pytest.mark.parametrize("gdb,wait", [(False, False), (True, False), (True, True)])
def test_native_gdb_is_loopback_and_wait_is_explicit(fake_config, gdb, wait):
    fake_config.update(gdb=gdb, gdb_wait=wait)
    with DedicatedSession() as session:
        vm = create(session, fake_config)
        result = session.call("vm_start", {"id": vm})
        launch = json.loads(Path(fake_config["work_dir"]).joinpath("arm64_launch.json").read_text())
        argv = launch["argv"]
        assert ("-gdb" in argv) == gdb
        if gdb:
            assert argv[argv.index("-gdb") + 1] == f"tcp:127.0.0.1:{result['gdb_port']}"
            assert result["gdb_host"] == "127.0.0.1" and result["gdb_port"] > 0
        assert ("-S" in argv) == (gdb and wait)


def test_native_early_exit_reports_real_stderr_without_full_accept_wait(fake_config, tmp_path):
    fake_config["env"].update(
        FAKE_QEMU_ATTEMPTS=str(tmp_path / "attempts"),
        FAKE_QEMU_FAILURES="1",
        FAKE_QEMU_FAILURE_MESSAGE="bad emulator configuration",
    )
    with DedicatedSession() as session:
        vm = create(session, fake_config)
        with pytest.raises(NativeError, match="bad emulator configuration"):
            session.call("vm_start", {"id": vm})
        assert not list(Path(fake_config["env"]["VIBEOS_QEMU_LOCK_DIR"]).glob("*.json"))


@pytest.mark.parametrize("failures,expected", [(0, 1), (1, 2), (3, 3)])
def test_native_retry_controller_uses_fresh_owned_sessions(
    fake_config, tmp_path, monkeypatch, failures, expected
):
    fake_config["env"].update(
        FAKE_QEMU_ATTEMPTS=str(tmp_path / "attempts"), FAKE_QEMU_FAILURES=str(failures)
    )
    monkeypatch.setattr(vm_module, "LAUNCH_RETRY_SECONDS", 0.0)
    instances = []

    class FixtureVm:
        def __init__(self):
            self.session = DedicatedSession()
            self.vm_id = create(self.session, fake_config)
            self.launch_timings = {}
            instances.append(self)

        def start(self):
            self.launch_timings = self.session.call("vm_start", {"id": self.vm_id})[
                "launch_timings"
            ]

        def stop(self):
            self.session.close()

    timings = {}
    logs = []
    _vm, attempts, error = vm_module.launch_with_retry(FixtureVm, timings, logs.append, "fixture")
    try:
        assert attempts == expected and len(instances) == expected
        assert (error is not None) == (failures >= 3)
        assert int((tmp_path / "attempts").read_text()) == expected
        if 0 < failures < 3:
            assert any("host-side flake" in line for line in logs)
            assert timings["launch_attempts"] == expected
    finally:
        for instance in instances:
            instance.stop()


def test_native_allocator_skips_duplicate_candidates_and_refuses_constant_picker():
    from tools.qemu_infra_compat import call

    assert call("vm_unique_ports", {"candidates": [45000, 45000, 45001], "count": 2}) == [
        45000,
        45001,
    ]
    with pytest.raises(NativeError, match="unique launch port"):
        call("vm_unique_ports", {"candidates": [45000], "count": 2})
