"""Preparation must overlap without opening the port selection race again."""

import threading
from concurrent.futures import ThreadPoolExecutor

import pytest

from .. import vm
from ..portlease import launch_lock


def test_media_preparation_overlaps_but_launch_is_exclusive(tmp_path, monkeypatch):
    monkeypatch.setenv("VIBEOS_HOSTLOCK_DIR", str(tmp_path / "locks"))
    ready = threading.Barrier(2)
    launched = []

    def prepare():
        # If preparation still holds the launch lock, the second VM cannot arrive.
        ready.wait(timeout=5)

    def launch():
        with pytest.raises(TimeoutError), launch_lock(timeout=0):
            pytest.fail("port handshake is unprotected")
        launched.append(True)

    machines = [vm.QemuVM(vm.Arm64Config(), tmp_path / str(i)) for i in range(2)]
    for machine in machines:
        monkeypatch.setattr(machine, "_acquire_slot", lambda: None)
        monkeypatch.setattr(machine, "_acquire_cpu_lease", lambda: None)
        monkeypatch.setattr(machine, "_prepare_launch", prepare)
        monkeypatch.setattr(machine, "_start_with_ports", launch)
    with ThreadPoolExecutor(max_workers=2) as pool:
        futures = [pool.submit(machine.start) for machine in machines]
        for future in futures:
            future.result(timeout=10)
    assert len(launched) == 2


def test_preparation_failure_releases_resources(tmp_path, monkeypatch):
    machine = vm.QemuVM(vm.Arm64Config(), tmp_path)
    stopped = []
    monkeypatch.setattr(machine, "_acquire_slot", lambda: None)
    monkeypatch.setattr(machine, "_acquire_cpu_lease", lambda: None)

    def fail():
        raise OSError("staging failed")

    monkeypatch.setattr(machine, "_prepare_launch", fail)
    monkeypatch.setattr(machine, "stop", lambda: stopped.append(True))
    with pytest.raises(OSError, match="staging failed"):
        machine.start()
    assert stopped == [True]


@pytest.mark.parametrize(
    "config,base",
    [
        (vm.Amd64UefiNetConfig, vm.Amd64UefiConfig),
        (vm.I386UefiNetConfig, vm.I386UefiConfig),
        (vm.Arm64NetConfig, vm.Arm64Config),
    ],
)
def test_media_preparation_does_not_select_ports(config, base, tmp_path, monkeypatch):
    machine = config()
    monkeypatch.setattr(base, "qemu_args", lambda self, work: ["prepared-disk"])
    monkeypatch.setattr(vm, "_free_tcp_port", lambda: pytest.fail("early port selection"))
    assert machine.prepare_args(tmp_path) == ["prepared-disk"]
    ports = iter([41001, 41002, 41003])
    monkeypatch.setattr(vm, "_free_tcp_port", lambda: next(ports))
    network = machine.network_args()
    assert machine.echo_hostfwd_port == 41001
    assert machine.tdi_srv_hostfwd_port == 41002
    assert machine.payload_hostfwd_port == 41003
    assert all(str(port) in network[-1] for port in (41001, 41002, 41003))


def test_checkpoint_stages_media_before_selecting_gdb_port(tmp_path, monkeypatch):
    from . import checkpoint

    machine = checkpoint.RestoredVM(
        tmp_path / "bundle",
        {
            "config": {
                "arch": "arm64",
                "qemu_bin": "qemu-system-aarch64",
                "machine_args": [],
                "memory": "1G",
                "boot_marker": "ready",
            },
            "backend": "native",
            "emulator": {"path": "qemu-system-aarch64"},
        },
        tmp_path / "work",
        {},
        True,
    )
    operations = []

    def guest(staging, windows):
        operations.append("media")
        return ["-drive", "file=prepared.qcow2"]

    monkeypatch.setattr(machine, "_guest_args", guest)
    monkeypatch.setattr(machine, "_debugcon_args", lambda *args: [])
    monkeypatch.setattr(machine, "_set_live_tee", lambda: None)
    monkeypatch.setattr(machine.serial, "listen", lambda *args: None)

    def gdb(**kwargs):
        operations.append("port")
        return ["-gdb", "tcp::41001"]

    monkeypatch.setattr(machine, "_gdb_args", gdb)
    machine._prepare_launch()
    assert operations == ["media"]
    argv = machine._build_native_cmd()
    assert operations == ["media", "port"]
    assert "file=prepared.qcow2" in argv


def test_cleanup_protects_preparing_vm_without_a_qemu_process(tmp_path, monkeypatch):
    import os
    import time

    from .. import windows_staging

    monkeypatch.setenv("VIBEOS_HOSTLOCK_DIR", str(tmp_path / "locks"))
    root = tmp_path / "staging"
    machine = vm.QemuVM(
        vm.Arm64Config(),
        tmp_path / "work",
        backend="windows-host",
        windows={"staging_dir": str(root)},
    )
    staging = machine._prepare_staging()
    (staging / "private.qcow2").write_bytes(b"preparing")
    old = time.time() - 7200
    os.utime(staging, (old, old))
    monkeypatch.setattr(
        windows_staging,
        "windows_qemu_commands",
        lambda: pytest.fail("live Linux owner must protect unlaunched media"),
    )
    with launch_lock():
        result = windows_staging.cleanup_locked(root)
    assert str(staging) in result["protected"]
    assert (staging / "private.qcow2").read_bytes() == b"preparing"
    assert machine._proc is None
