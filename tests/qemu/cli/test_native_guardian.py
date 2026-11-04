"""Owned-process guardian fixture controls, including uncertain Windows stop."""

import fcntl
import json
import os
import subprocess
import sys
import time
from pathlib import Path

import pytest

from tools.test_infra_compat import REPO, DedicatedSession

from .test_native_vm import create, gone, wait_gone
from .test_native_vm import fake_config as _fake_config

fake_config = _fake_config


def tick(pid):
    return int(Path(f"/proc/{pid}/stat").read_text().rsplit(")", 1)[1].split()[19])


def read_until(path, predicate, timeout=10):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            value = json.loads(path.read_text())
        except (FileNotFoundError, json.JSONDecodeError):
            value = None
        if value is not None and predicate(value):
            return value
        time.sleep(0.02)
    raise AssertionError(f"guardian did not publish expected state at {path}")


@pytest.mark.parametrize("left", ["unknown", "1"])
def test_guardian_retains_lease_and_evidence_until_windows_confirms(tmp_path, left):
    state = tmp_path / "windows-left"
    state.write_text(left)
    bin_dir = tmp_path / "bin"
    bin_dir.mkdir()
    powershell = bin_dir / "powershell.exe"
    powershell.write_text(
        f"#!{sys.executable}\nfrom pathlib import Path\n"
        f"value=Path({str(state)!r}).read_text()\n"
        "print('left='+value if value.isdigit() else 'unavailable')\n"
    )
    powershell.chmod(0o700)
    lease = tmp_path / "lease"
    descriptor = tmp_path / "guardian.json"
    status = tmp_path / "status.json"
    lock = tmp_path / "lock.json"
    staging = tmp_path / "staging"
    staging.mkdir()
    lock.write_text("owned ledger")
    owner = os.getpid()
    stream = lease.open("w+")
    fcntl.flock(stream, fcntl.LOCK_EX)
    request = {
        "argv": ["sleep", "60"],
        "cwd": str(tmp_path),
        "env": {},
        "owner_pid": owner,
        "owner_tick": tick(owner),
        "server_pid": owner,
        "server_tick": tick(owner),
        "deadline": time.time() + 60,
        "status": str(status),
        "stderr": str(tmp_path / "stderr"),
        "uid": "vibeos-fixture-123",
        "stop": False,
        "detached": False,
        "lease_fds": [stream.fileno()],
        "lock_path": str(lock),
        "staging": str(staging),
    }
    descriptor.write_text(json.dumps(request))
    guardian = subprocess.Popen(
        [str(REPO / "tools/test-infra"), "vm-guardian", str(descriptor)],
        pass_fds=(stream.fileno(),),
        env={**os.environ, "PATH": f"{bin_dir}:{os.environ['PATH']}"},
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE,
    )
    stream.close()
    try:
        running = read_until(status, lambda value: value["state"] == "running")
        request["stop"] = True
        temporary = descriptor.with_suffix(".tmp")
        temporary.write_text(json.dumps(request))
        temporary.replace(descriptor)
        read_until(status, lambda value: value["state"] == "stopping")
        assert guardian.poll() is None and lock.exists() and staging.exists()
        with lease.open("r+") as contender, pytest.raises(BlockingIOError):
            fcntl.flock(contender, fcntl.LOCK_EX | fcntl.LOCK_NB)
        state.write_text("0")
        assert guardian.wait(timeout=15) == 0
        stopped = read_until(status, lambda value: value["state"] == "stopped")
        assert stopped["confirmed"] and not lock.exists() and not staging.exists()
        wait_gone(running["pid"])
        with lease.open("r+") as contender:
            fcntl.flock(contender, fcntl.LOCK_EX | fcntl.LOCK_NB)
    finally:
        state.write_text("0")
        try:
            guardian.wait(timeout=15)
        except subprocess.TimeoutExpired:
            guardian.terminate()
            guardian.wait(timeout=5)


def test_detach_moves_owned_ledger_and_old_session_cannot_kill(fake_config, tmp_path):
    owner = subprocess.Popen(["sleep", "60"])
    session = DedicatedSession()
    try:
        vm = create(session, fake_config)
        pid = session.call("vm_start", {"id": vm})["qemu_pid"]
        result = session.call(
            "vm_detach", {"id": vm, "owner_pid": owner.pid, "run_id": "debug-fixture"}
        )
        assert result["qemu_pid"] == pid
        session.call("vm_stop", {"id": vm})
        session.close()
        time.sleep(0.2)
        assert not gone(pid)
        locks = list(Path(fake_config["env"]["VIBEOS_QEMU_LOCK_DIR"]).glob("*.json"))
        assert len(locks) == 1
        record = json.loads(locks[0].read_text())
        assert record["launcher_pid"] == owner.pid and record["run_id"] == "debug-fixture"
        assert record["qemu_start_tick"] == tick(pid)
        owner.terminate()
        owner.wait(timeout=5)
        wait_gone(pid)
    finally:
        if owner.poll() is None:
            owner.terminate()
            owner.wait(timeout=5)
        session.close()


def test_dead_vm_cannot_be_detached(fake_config):
    with DedicatedSession() as session:
        vm = create(session, fake_config)
        session.call("vm_start", {"id": vm})
        session.call("vm_stop", {"id": vm})
        with pytest.raises(ValueError, match="already exited"):
            session.call("vm_detach", {"id": vm, "owner_pid": os.getpid(), "run_id": "dead"})
