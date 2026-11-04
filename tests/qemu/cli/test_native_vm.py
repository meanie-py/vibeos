"""Native socket/lifecycle fixtures replacing Python-private VM monkeypatches."""

import json
import os
import socket
import stat
import threading
import time
from pathlib import Path

import pytest

from tests.qemu.qmp_client import QmpClient
from tests.qemu.serial_client import SerialClient
from tools.qemu_infra_compat import DedicatedSession


@pytest.fixture
def fake_config(tmp_path):
    # Root copies fake_qemu.py alongside this fixture module.
    executable = tmp_path / "fake-qemu"
    executable.write_text(Path(__file__).with_name("fake_qemu.py").read_text())
    executable.chmod(executable.stat().st_mode | stat.S_IXUSR)
    events = tmp_path / "events"
    return {
        "arch": "arm64",
        "qemu_bin": str(executable),
        "work_dir": str(tmp_path / "work"),
        "boot_marker": "READY",
        "boot_timeout": 2,
        "desktop_settle_seconds": 0,
        "min_boot_bytes": 0,
        "env": {
            **os.environ,
            "FAKE_QEMU_EVENTS": str(events),
            "VIBEOS_QEMU_LOCK_DIR": str(tmp_path / "locks"),
            "VIBEOS_HOSTLOCK_DIR": str(tmp_path / "host"),
            "VIBEOS_BUILD_POLICY_DIR": str(tmp_path / "policy"),
            "VIBEOS_HOST_BROKER": "0",
            "VIBEOS_QEMU_MAX_LIFETIME": "30",
        },
    }


def create(session, config):
    return session.call("vm_create", {"config": config})["id"]


def gone(pid):
    try:
        data = Path(f"/proc/{pid}/stat").read_text().rsplit(")", 1)[1].split()
    except FileNotFoundError:
        return True
    return data[0] == "Z"


def wait_gone(pid):
    deadline = time.monotonic() + 10
    while not gone(pid) and time.monotonic() < deadline:
        time.sleep(0.02)
    assert gone(pid), f"owned fixture process {pid} survived cleanup"


def test_native_vm_snapshot_preserves_restored_output(fake_config):
    with DedicatedSession() as session:
        vm = create(session, fake_config)
        handles = session.call("vm_start", {"id": vm})
        pid = handles["qemu_pid"]
        assert session.call("vm_wait_for_boot", {"id": vm}) == "before READY"
        session.call("vm_save_snapshot", {"id": vm, "name": "ready"})
        session.call("vm_restore_snapshot", {"id": vm, "name": "ready"})
        assert "RESTORED" in session.call(
            "vm_serial_read_until", {"id": vm, "marker": "RESTORED", "timeout": 2}
        )
        events = Path(fake_config["env"]["FAKE_QEMU_EVENTS"]).read_text().splitlines()
        assert events[events.index("savevm ready") - 1] == "stop"
        assert events[events.index("savevm ready") + 1] == "cont"
        assert events[events.index("loadvm ready") - 1] == "stop"
        assert events[events.index("loadvm ready") + 1] == "cont"
        session.call("vm_stop", {"id": vm})
        wait_gone(pid)
        assert not list(Path(fake_config["env"]["VIBEOS_QEMU_LOCK_DIR"]).glob("*.json"))


def test_api_server_death_cleans_vm_independently(fake_config):
    session = DedicatedSession()
    vm = create(session, fake_config)
    pid = session.call("vm_start", {"id": vm})["qemu_pid"]
    server = session._session.process
    server.kill()
    server.wait(timeout=5)
    wait_gone(pid)
    session.close()


def test_vm_lifetime_is_absolute(fake_config):
    fake_config["max_lifetime"] = 0.5
    with DedicatedSession() as session:
        vm = create(session, fake_config)
        pid = session.call("vm_start", {"id": vm})["qemu_pid"]
        wait_gone(pid)


def test_serial_native_lifetime_drain_and_latin1(tmp_path):
    serial = SerialClient()
    path = tmp_path / "serial.sock"
    serial.listen(path)
    stream = socket.socket(socket.AF_UNIX)
    stream.connect(str(path))
    try:
        serial.accept(timeout=1)
        stream.sendall(b"READY\xffafter")
        time.sleep(0.05)
        assert serial.read_until("READY", timeout=1) == "READY"
        assert serial.read_available(timeout=1) == "ÿafter"
        assert serial._pos == len("READYÿafter")
    finally:
        stream.close()
        serial.close()


def test_qmp_native_retains_event_between_greeting_and_reply(tmp_path):
    path = tmp_path / "qmp.sock"
    listener = socket.socket(socket.AF_UNIX)
    listener.bind(str(path))
    listener.listen()

    def server():
        connection, _ = listener.accept()
        with connection, connection.makefile("rwb", buffering=0) as stream:
            stream.write(b'{"QMP":{}}\n')
            for _ in range(2):
                assert stream.readline()
                stream.write(b'{"event":"STOP"}\n{"return":{}}\n')

    thread = threading.Thread(target=server)
    thread.start()
    qmp = QmpClient()
    try:
        qmp.connect(path, timeout=1)
        assert qmp.execute("query-status") == {"return": {}}
        assert qmp.saw_event("STOP")["event"] == "STOP"
    finally:
        qmp.close()
        thread.join(timeout=2)
        listener.close()


def test_reaper_refuses_live_pid_without_incarnation_proof(tmp_path):
    from tests.qemu.vm import reap_orphan_qemus

    # Caller is protected too: unknown incarnation must retain even an expired ledger.
    lock = tmp_path / f"999999-{os.getpid()}.json"
    lock.write_text(json.dumps({"launcher_pid": 999999, "qemu_pid": os.getpid(), "deadline": 1}))
    from unittest.mock import patch

    with patch.dict(
        os.environ, {"VIBEOS_QEMU_LOCK_DIR": str(tmp_path), "RUNS_DIR": str(tmp_path / "runs")}
    ):
        actions = reap_orphan_qemus()
    assert actions and actions[0].startswith("FAILED")
    assert lock.exists()


def test_default_windows_tcg_same_arch_admission_stays_parallel(fake_config, tmp_path):
    fake_config.update(backend="windows-host", arch="amd64-uefi", windows={"accel": "tcg"})
    fake_config["env"].update(VIBEOS_QEMU_CAP_AMD64="1", VIBEOS_QEMU_QUEUE_TIMEOUT="0.2")
    with DedicatedSession() as session:
        first = create(session, fake_config)
        fake_config["work_dir"] = str(tmp_path / "second")
        second = create(session, fake_config)
        session.call("vm_admit", {"id": first})
        session.call("vm_admit", {"id": second})
        one = session.call("vm_handles", {"id": first})["admission"]
        two = session.call("vm_handles", {"id": second})["admission"]
        assert one["arch_slot"] is None and two["arch_slot"] is None
        assert one["global_slot"] != two["global_slot"]
