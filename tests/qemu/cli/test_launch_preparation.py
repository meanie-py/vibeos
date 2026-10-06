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


@pytest.mark.parametrize(
    ("config", "env", "attached"),
    [
        (vm.Amd64UefiConfig, None, True),  # default ON: the only disk any smoke boots with
        (vm.Amd64UefiConfig, "0", False),
        (vm.I386UefiConfig, None, True),  # default ON since fastfat's i386 DriverEntry fix
        (vm.I386UefiConfig, "0", False),
        (vm.Arm64Config, "1", False),  # no IDE on arm64
    ],
)
def test_temp_disk_policy(config, env, attached, tmp_path, monkeypatch):
    """The scratch-TEMP disk is also the smoke's only hard disk, and the disk path
    stayed broken for months (classpnp hung every boot; i386 fastfat returned into
    its caller's stack) because nothing attached one. The default is the regression
    guard, so pin it."""
    if env is None:
        monkeypatch.delenv("VIBEOS_QEMU_TEMP_DISK", raising=False)
    else:
        monkeypatch.setenv("VIBEOS_QEMU_TEMP_DISK", env)
    made = []
    monkeypatch.setattr(vm, "make_temp_disk", lambda p: made.append(p))
    args = vm.QemuVM(config(), tmp_path)._temp_disk_args()
    assert bool(args) == attached and bool(made) == attached
    if attached:
        assert "ide-hd,drive=tempdisk" in args
        assert not any("bootindex" in a for a in args), "must never boot ahead of the CD"


@pytest.mark.parametrize("arch", ["i386", "i386-uefi", "amd64", "amd64-uefi"])
def test_gdb_launches_share_the_smoke_disk_policy(arch, tmp_path, monkeypatch):
    """`gdb start` builds its own QEMU command line. It once omitted this disk, so a
    debug boot could not reproduce the disk-only i386 crash it was started to catch.
    Both launchers now call vm.temp_disk_args with the arch spelling they hold."""
    monkeypatch.delenv("VIBEOS_QEMU_TEMP_DISK", raising=False)
    monkeypatch.setattr(vm, "make_temp_disk", lambda p: None)
    assert "ide-hd,drive=tempdisk" in vm.temp_disk_args(arch, tmp_path)


def test_the_bios_smoke_attaches_no_mbr_disk(tmp_path):
    """isoboot boots drive 80h after a 5 s prompt when it carries a valid MBR, so with
    the temp disk attached an unattended BIOS smoke never reached freeldr (2026-10-05)."""
    assert vm.temp_disk_args("amd64-bios", tmp_path) == []


def test_the_bios_config_knows_its_own_name():
    """The arch configs are dataclasses: a subclass without its own @dataclass keeps
    the PARENT's field default, so Amd64BiosConfig().arch was "amd64" and every
    per-arch decision keyed on the name (the temp disk above) silently missed it."""
    assert vm.get_arch_config("amd64-bios").arch == "amd64-bios"


def test_the_bios_smoke_boots_the_iso_from_a_usb_stick(monkeypatch):
    """The BIOS lane is the Libreboot OptiPlex's configuration: the hybrid ISO written
    to a USB stick, through isombr and usbstor. Every other x86 lane keeps ide-cd."""
    monkeypatch.delenv("VIBEOS_QEMU_BOOT_TRANSPORT", raising=False)
    bios = vm.boot_media_device_args("x.iso", default_transport="usb-storage")
    assert any(a.startswith("usb-storage,") for a in bios), bios
    assert any(a.startswith("ide-cd,") for a in vm.boot_media_device_args("x.iso"))
    monkeypatch.setenv("VIBEOS_QEMU_BOOT_TRANSPORT", "ide-cd")
    cd = vm.boot_media_device_args("x.iso", default_transport="usb-storage")
    assert any(a.startswith("ide-cd,") for a in cd), cd


@pytest.mark.parametrize("config", [vm.Amd64UefiConfig, vm.I386UefiConfig, vm.Arm64Config])
def test_the_nvme_rig_attaches_one_controller_on_every_arch(config, tmp_path, monkeypatch):
    """VIBEOS_QEMU_NVME=1 is the nvme.sys rig on all three arches. The namespace
    must never get a bootindex: firmware would then try it ahead of the CD."""
    made = []
    monkeypatch.setattr(vm, "make_usb_stick_image", lambda p: made.append(p))
    monkeypatch.delenv("VIBEOS_QEMU_NVME", raising=False)
    machine = vm.QemuVM(config(), tmp_path)
    assert machine._nvme_args() == [] and made == []
    monkeypatch.setenv("VIBEOS_QEMU_NVME", "1")
    args = machine._nvme_args()
    assert any(a.startswith("nvme,") and "drive=nvme0" in a and "addr=0x9" in a for a in args), args
    assert not any("bootindex" in a for a in args)
    assert made == [machine.scratch_path.parent / "nvmedisk.qcow2"]
