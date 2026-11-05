"""Native checkpoint controls over real owned QMP/serial fixture processes."""

import hashlib
import os
from pathlib import Path

import pytest

from tools.qemu_infra_compat import DedicatedSession, NativeError, call


@pytest.fixture
def config(tmp_path):
    binary = tmp_path / "fake-checkpoint-qemu"
    binary.write_text(Path(__file__).with_name("fake_checkpoint_qemu.py").read_text())
    binary.chmod(0o700)
    return {
        "arch": "arm64",
        "qemu_bin": str(binary),
        "work_dir": str(tmp_path / "work"),
        "env": {
            **os.environ,
            "CP_EVENTS": str(tmp_path / "events"),
            "VIBEOS_QEMU_LOCK_DIR": str(tmp_path / "locks"),
            "VIBEOS_HOSTLOCK_DIR": str(tmp_path / "host"),
            "VIBEOS_BUILD_POLICY_DIR": str(tmp_path / "policy"),
            "VIBEOS_HOST_BROKER": "0",
        },
        "max_lifetime": 30,
        "boot_timeout": 2,
        "desktop_settle_seconds": 0,
        "min_boot_bytes": 0,
    }


def start(session, config):
    vm = session.call("vm_create", {"config": config})["id"]
    session.call("vm_start", {"id": vm})
    return vm


@pytest.mark.parametrize("state", ["paused", "prelaunch", "debug"])
def test_native_checkpoint_accepts_real_stopped_states(config, state):
    config["env"]["CP_STOP_STATE"] = state
    with DedicatedSession() as session:
        vm = start(session, config)
        session.call("vm_checkpoint_pause", {"id": vm})


def test_native_checkpoint_refuses_internal_error_stop(config):
    config["env"]["CP_STOP_STATE"] = "internal-error"
    with DedicatedSession() as session:
        vm = start(session, config)
        with pytest.raises(NativeError, match="did not stop cleanly"):
            session.call("vm_checkpoint_pause", {"id": vm})


def test_native_checkpoint_any_hmp_error_is_fatal(config):
    config["env"]["CP_SNAPSHOT_ERROR"] = "device cannot be saved"
    with DedicatedSession() as session:
        vm = start(session, config)
        with pytest.raises(NativeError, match="cannot be saved"):
            session.call("vm_checkpoint_snapshot", {"id": vm, "operation": "loadvm"})


def test_native_checkpoint_restore_verifies_registers_before_continuing(config):
    with DedicatedSession() as session:
        vm = start(session, config)
        with pytest.raises(NativeError, match="registers differ"):
            session.call(
                "vm_checkpoint_restore",
                {"id": vm, "manifest": {"registers": "PC=wrong", "vcpus": 1}},
            )
        events = Path(config["env"]["CP_EVENTS"]).read_text().splitlines()
        assert "loadvm checkpoint" in events
        assert "cont" not in events


def test_native_checkpoint_restore_verifies_cpu_topology(config):
    with DedicatedSession() as session:
        vm = start(session, config)
        witness = session.call("vm_checkpoint_registers", {"id": vm})
        with pytest.raises(NativeError, match="CPU topology differs"):
            session.call(
                "vm_checkpoint_restore", {"id": vm, "manifest": {"registers": witness, "vcpus": 2}}
            )


def test_native_checkpoint_failure_wins_in_same_serial_chunk(config, tmp_path):
    config["env"]["CP_SERIAL"] = "SUCCESS\nBUGCHECK 0x1e"
    with DedicatedSession() as session:
        vm = start(session, config)
        result = session.call(
            "vm_checkpoint_watch",
            {
                "id": vm,
                "until": "SUCCESS",
                "fail": "(?i)BUGCHECK",
                "timeout": 1,
                "log": str(tmp_path / "window.log"),
            },
        )
        assert result == "failure-pattern"
        assert "BUGCHECK" in (tmp_path / "window.log").read_text()


def test_native_checkpoint_timeout_retains_its_serial_evidence(config, tmp_path):
    config["env"]["CP_SERIAL"] = "partial progress"
    with DedicatedSession() as session:
        vm = start(session, config)
        result = session.call(
            "vm_checkpoint_watch",
            {
                "id": vm,
                "until": "UNREACHED",
                "fail": "BUGCHECK",
                "timeout": 0.1,
                "log": str(tmp_path / "window.log"),
            },
        )
        assert result == "timeout"
        assert (tmp_path / "window.log").read_text() == "partial progress"


def test_native_checkpoint_staging_is_private(config, tmp_path):
    bundle = tmp_path / "bundle"
    bundle.mkdir()
    seed = bundle / "media-00.qcow2"
    seed.write_bytes(b"saved")
    identity = call("checkpoint_emulator_identity", {"executable": config["qemu_bin"]})
    manifest = {
        "schema": 1,
        "accel": "tcg",
        "backend": "native",
        "emulator": identity,
        "media": [
            {
                "name": seed.name,
                "sha256": hashlib.sha256(b"saved").hexdigest(),
                "readonly": False,
                "format": "qcow2",
            }
        ],
        "argv": ["-drive", "file=@media-00.qcow2,format=qcow2"],
    }
    work = tmp_path / "attempt"
    args = call(
        "checkpoint_prepare_guest",
        {"bundle": str(bundle), "manifest": manifest, "staging": str(work), "windows": False},
    )
    assert str(work / seed.name) in args[1]
    (work / seed.name).write_bytes(b"changed")
    assert seed.read_bytes() == b"saved"
