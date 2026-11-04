"""Native media work overlaps; only port selection through handshake is exclusive."""

import fcntl
import hashlib
import os
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import pytest

from tools.qemu_infra_compat import DedicatedSession, NativeError, call

from .. import vm
from ..portlease import launch_lock
from .native_preparation_fixture import preparation_tree as _preparation_tree
from .native_preparation_fixture import request
from .test_checkpoint_native import config as _checkpoint_config

preparation_tree = _preparation_tree
checkpoint_config = _checkpoint_config


def prepare_plan(repo, work, arch):
    return call(
        "vm_prepare_config", request(repo, work, arch=arch, backend="windows-host"), root=repo
    )


def test_media_preparation_overlaps_but_launch_is_exclusive(
    preparation_tree, tmp_path, monkeypatch
):
    repo, trace = preparation_tree
    monkeypatch.setenv("FAKE_PREPARATION", "barrier")

    def boot(index):
        with DedicatedSession(root=repo) as session:
            identifier = session.call("vm_create", request(repo, tmp_path / f"work-{index}"))["id"]
            session.call("vm_start", {"id": identifier})

    with ThreadPoolExecutor(max_workers=2) as pool:
        futures = [pool.submit(boot, index) for index in range(2)]
        for future in futures:
            future.result(timeout=30)
    # The external conversion tool takes a shared probe on the real handshake inode
    # and rendezvous before either conversion completes. This proves actual overlap.
    assert len(list(trace.glob("prepare-*"))) == 2
    starts = list(trace.glob("launch-start-*"))
    assert len(starts) == 2
    intervals = sorted(
        (int(path.read_text()), int(path.with_name(path.name.replace("start", "end")).read_text()))
        for path in starts
    )
    assert intervals[0][1] <= intervals[1][0]


def test_preparation_failure_releases_resources(preparation_tree, tmp_path, monkeypatch):
    repo, _trace = preparation_tree
    monkeypatch.setenv("FAKE_PREPARATION", "fail")
    with DedicatedSession(root=repo) as session:
        identifier = session.call("vm_create", request(repo, tmp_path / "failed"))["id"]
        with pytest.raises(NativeError, match="fixture media staging failed"):
            session.call("vm_start", {"id": identifier})
        handles = session.call("vm_handles", {"id": identifier})
        assert handles["qemu_pid"] is None and handles["admission"] is None
        slot = Path(os.environ["VIBEOS_QEMU_LOCK_DIR"]) / "slots/slot-00"
        with slot.open("r+") as stream:
            fcntl.flock(stream, fcntl.LOCK_EX | fcntl.LOCK_NB)
        with launch_lock(timeout=0):
            pass


@pytest.mark.parametrize("arch", ["amd64-uefi-net", "i386-uefi-net", "arm64-net"])
def test_media_preparation_does_not_select_ports(preparation_tree, tmp_path, arch):
    repo, trace = preparation_tree
    plan = prepare_plan(repo, tmp_path / arch, arch)
    assert plan["network_args"] == []
    assert not any("hostfwd=" in argument for argument in plan["arch_args"])
    # A Windows port selection would execute the fixture PowerShell and fail.
    assert not (trace / "unexpected-port-selection").exists()


def test_checkpoint_media_failure_precedes_selecting_gdb_port(
    preparation_tree, checkpoint_config, tmp_path
):
    _repo, trace = preparation_tree
    bundle = tmp_path / "bundle"
    bundle.mkdir()
    identity = call("checkpoint_emulator_identity", {"executable": checkpoint_config["qemu_bin"]})
    manifest = {
        "schema": 1,
        "accel": "tcg",
        "backend": "windows-host",
        "emulator": identity,
        "media": [
            {
                "name": "missing.qcow2",
                "readonly": False,
                "format": "qcow2",
                "sha256": hashlib.sha256(b"missing").hexdigest(),
            }
        ],
        "argv": ["-drive", "file=@missing.qcow2,format=qcow2"],
    }
    checkpoint_config.update(
        backend="windows-host",
        gdb=True,
        windows={"accel": "tcg", "staging_dir": str(tmp_path / "win-staging")},
        checkpoint={"bundle": str(bundle), "manifest": manifest},
    )
    with DedicatedSession() as session:
        identifier = session.call("vm_create", {"config": checkpoint_config})["id"]
        with pytest.raises(NativeError):
            session.call("vm_start", {"id": identifier})
        assert session.call("vm_handles", {"id": identifier})["qemu_pid"] is None
    assert not (trace / "unexpected-port-selection").exists()


def test_cleanup_protects_preparing_vm_without_a_qemu_process(preparation_tree, tmp_path):
    repo, trace = preparation_tree
    with DedicatedSession(root=repo) as session:
        identifier = session.call(
            "vm_create", request(repo, tmp_path / "preparing", backend="windows-host")
        )["id"]
        session.call("vm_admit", {"id": identifier})
        owner = session._session.process.pid
        root = tmp_path / "windows-staging"
        staging = root / f"{owner}-preparing"
        staging.mkdir(parents=True)
        (staging / "private.qcow2").write_bytes(b"preparing")
        old = time.time() - 7200
        os.utime(staging, (old, old))
        with launch_lock():
            result = call("backend_cleanup_staging", {"root": str(root), "min_age": 3600})
        assert str(staging) in result["protected"]
        assert (staging / "private.qcow2").read_bytes() == b"preparing"
        assert session.call("vm_handles", {"id": identifier})["qemu_pid"] is None
    assert not (trace / "unexpected-port-selection").exists()


@pytest.mark.parametrize(
    "arch,enabled,attached",
    [
        ("amd64-uefi", None, True),
        ("amd64-uefi", "0", False),
        ("i386-uefi", None, True),
        ("i386-uefi", "0", False),
        ("arm64", "1", False),
    ],
)
def test_temp_disk_policy(preparation_tree, tmp_path, monkeypatch, arch, enabled, attached):
    repo, _trace = preparation_tree
    if enabled is not None:
        monkeypatch.setenv("VIBEOS_QEMU_TEMP_DISK", enabled)
    work = tmp_path / arch
    work.mkdir()
    args = call(
        "vm_temp_disk_args",
        {"arch": arch, "work_dir": str(work), "env": dict(os.environ)},
        root=repo,
    )
    assert bool(args) == attached and (work / "tempdisk.qcow2").is_file() == attached
    if attached:
        assert "ide-hd,drive=tempdisk" in args
        assert not any("bootindex" in argument for argument in args)


@pytest.mark.parametrize("arch", ["i386", "i386-uefi", "amd64", "amd64-uefi"])
def test_gdb_launches_share_the_smoke_disk_policy(preparation_tree, arch, tmp_path):
    _repo, _trace = preparation_tree
    assert "ide-hd,drive=tempdisk" in vm.temp_disk_args(arch, tmp_path)
    assert (tmp_path / "tempdisk.qcow2").is_file()


def test_the_bios_smoke_attaches_no_mbr_disk(tmp_path):
    assert vm.temp_disk_args("amd64-bios", tmp_path) == []


def test_the_bios_config_knows_its_own_name():
    assert vm.get_arch_config("amd64-bios").arch == "amd64-bios"


def test_the_bios_smoke_boots_the_iso_from_a_usb_stick(monkeypatch):
    monkeypatch.delenv("VIBEOS_QEMU_BOOT_TRANSPORT", raising=False)
    assert any(
        arg.startswith("usb-storage,")
        for arg in vm.boot_media_device_args("x.iso", default_transport="usb-storage")
    )
    assert any(arg.startswith("ide-cd,") for arg in vm.boot_media_device_args("x.iso"))
    monkeypatch.setenv("VIBEOS_QEMU_BOOT_TRANSPORT", "ide-cd")
    assert any(
        arg.startswith("ide-cd,")
        for arg in vm.boot_media_device_args("x.iso", default_transport="usb-storage")
    )


@pytest.mark.parametrize("arch", ["amd64-uefi", "i386-uefi", "arm64"])
def test_the_nvme_rig_attaches_one_controller_on_every_arch(
    preparation_tree, tmp_path, monkeypatch, arch
):
    repo, _trace = preparation_tree
    plain = prepare_plan(repo, tmp_path / "plain", arch)
    assert not any("drive=nvme0" in argument for argument in plain["common_args"])
    monkeypatch.setenv("VIBEOS_QEMU_NVME", "1")
    work = tmp_path / "nvme"
    plan = prepare_plan(repo, work, arch)
    controllers = [
        argument
        for argument in plan["common_args"]
        if argument.startswith("nvme,") and "drive=nvme0" in argument
    ]
    assert len(controllers) == 1 and "addr=0x9" in controllers[0]
    assert "bootindex" not in controllers[0]
    assert sum("id=nvme0" in argument for argument in plan["disk_args"]) == 1
    assert (work / "nvmedisk.qcow2").is_file()
