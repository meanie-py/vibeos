"""Real native process/socket controls for serial evidence and snapshot gaps."""

import copy
import json
import os
import socket
import time
from pathlib import Path

import pytest

from tests.qemu.cli import artifacts, smoke
from tests.qemu.serial_client import SerialClient
from tools.qemu_infra_compat import DedicatedSession, NativeError


@pytest.fixture
def serial_config(tmp_path):
    binary = tmp_path / "serial-qemu"
    binary.write_text(Path(__file__).with_name("fake_serial_qemu.py").read_text())
    binary.chmod(0o700)
    env = {
        **os.environ,
        "SERIAL_EVENTS": str(tmp_path / "events"),
        "VIBEOS_QEMU_LOCK_DIR": str(tmp_path / "locks"),
        "VIBEOS_HOSTLOCK_DIR": str(tmp_path / "host"),
        "VIBEOS_BUILD_POLICY_DIR": str(tmp_path / "policy"),
        "VIBEOS_HOST_BROKER": "0",
    }
    for key in ("VIBEOS_QEMU_RR", "VIBEOS_QEMU_RR_SAMPLE", "VIBEOS_QEMU_PC_SAMPLE"):
        env.pop(key, None)
    return {
        "arch": "arm64",
        "qemu_bin": str(binary),
        "work_dir": str(tmp_path / "work"),
        "boot_marker": "READY",
        "boot_timeout": 2,
        "desktop_settle_seconds": 0,
        "min_boot_bytes": 0,
        "max_lifetime": 30,
        "env": env,
    }


def start(session, config):
    vm = session.call("vm_create", {"config": config})["id"]
    handles = session.call("vm_start", {"id": vm})
    return vm, handles


def exit_vm(session, vm):
    session.call("vm_qmp_execute", {"id": vm, "command": "fixture-exit", "arguments": {}})
    return session.call("vm_wait_exit", {"id": vm, "timeout": 5})


def test_boot_eof_preserves_serial_and_reports_process_exit(serial_config):
    serial_config["env"].update(
        SERIAL_INITIAL="firmware breadcrumb", SERIAL_STDERR="WHPX fatal error\n"
    )
    with DedicatedSession() as session:
        vm, _ = start(session, serial_config)
        assert exit_vm(session, vm) == 1
        with pytest.raises(NativeError, match="QEMU exited with code 1") as error:
            session.call("vm_wait_for_boot", {"id": vm})
        assert "WHPX fatal error" in str(error.value)
        handles = session.call("vm_handles", {"id": vm})
        assert handles["boot_output"] == "firmware breadcrumb"
        assert "firmware breadcrumb" in Path(handles["live_log"]).read_text()


def test_dead_qemu_does_not_drain_serial_for_another_hour(serial_config):
    serial_config["env"]["SERIAL_INITIAL"] = ""
    with DedicatedSession() as session:
        vm, _ = start(session, serial_config)
        assert exit_vm(session, vm) == 1
        began = time.monotonic()
        with pytest.raises(NativeError, match="QEMU exited with code 1"):
            session.call("vm_serial_output", {"id": vm, "timeout": 0.01})
        assert time.monotonic() - began < 2


def test_qemu_stderr_cannot_fill_a_pipe_and_stall(serial_config):
    serial_config["env"].update(SERIAL_STDERR_BYTES="131072", SERIAL_EXIT_CODE="0")
    with DedicatedSession() as session:
        vm, handles = start(session, serial_config)
        # Handshake completed after the child wrote 128 KiB to stderr.
        assert handles["qemu_pid"] > 0
        assert exit_vm(session, vm) == 0
        path = Path(handles["live_log"]).with_name("arm64_qemu.stderr.log")
        assert path.stat().st_size == 131072
        assert session.call("vm_stderr_tail", {"id": vm}) == "x" * 65536


def test_serial_eof_retains_partial_output(tmp_path):
    client = SerialClient()
    path = tmp_path / "serial.sock"
    try:
        client.listen(path)
        writer = socket.socket(socket.AF_UNIX)
        writer.connect(str(path))
        client.accept(timeout=2)
        writer.sendall(b"firmware reached the loader")
        writer.close()
        with pytest.raises(ConnectionError):
            client.read_until("shell ready", timeout=1)
        assert client.partial_output == "firmware reached the loader"
    finally:
        client.close()


@pytest.mark.parametrize("settle_times_out", [False, True])
def test_payload_marker_during_settle_survives_snapshot_restore(serial_config, settle_times_out):
    # A 20ms idle threshold races normal process/thread scheduling. Keep a
    # full second of tolerance while still requiring the real +10s timeout.
    serial_config["desktop_settle_seconds"] = 1 if settle_times_out else 0.02
    serial_config["max_lifetime"] = 120
    serial_config["env"].update(
        SERIAL_INITIAL="before READY\n[UCRTDIFF] done\n",
        SERIAL_CHATTY_UNTIL_STOP="1" if settle_times_out else "0",
    )
    with DedicatedSession() as session:
        vm, _ = start(session, serial_config)
        if settle_times_out:
            session.call(
                "vm_qmp_execute", {"id": vm, "command": "fixture-start-chat", "arguments": {}}
            )
        began = time.monotonic()
        session.call("vm_wait_for_boot", {"id": vm})
        # Exercise the actual idle timeout branch (settle budget + 10 seconds),
        # not a fabricated Python exception or private serial buffer.
        if settle_times_out:
            assert time.monotonic() - began >= 9
            session.call(
                "vm_qmp_execute", {"id": vm, "command": "fixture-stop-chat", "arguments": {}}
            )
        session.call("vm_save_snapshot", {"id": vm, "name": "ready"})
        session.call("vm_restore_snapshot", {"id": vm, "name": "ready"})
        assert "[UCRTDIFF] done" in session.call("vm_handles", {"id": vm})["boot_output"]


def test_marker_around_snapshot_reaches_the_test(serial_config):
    serial_config["env"].update(
        SERIAL_ON_STOP="[RPMPROBE] done\n", SERIAL_ON_LOAD="[RPMPROBE] done\n"
    )
    with DedicatedSession() as session:
        vm, _ = start(session, serial_config)
        session.call("vm_wait_for_boot", {"id": vm})
        session.call("vm_save_snapshot", {"id": vm, "name": "ready"})
        assert "[RPMPROBE] done" in session.call("vm_handles", {"id": vm})["boot_output"]
        session.call("vm_restore_snapshot", {"id": vm, "name": "ready"})
        output = session.call(
            "vm_serial_read_until", {"id": vm, "marker": "[RPMPROBE] done", "timeout": 2}
        )
        assert "[RPMPROBE] done" in output
        operations = Path(serial_config["env"]["SERIAL_EVENTS"]).read_text().splitlines()
        assert operations[-6:] == ["stop", "savevm ready", "cont", "stop", "loadvm ready", "cont"]


def test_overlapping_native_boots_have_independent_published_serial_streams(
    serial_config, tmp_path
):
    configs = []
    for index in range(2):
        config = copy.deepcopy(serial_config)
        config["work_dir"] = str(tmp_path / f"boot{index}")
        config["env"]["SERIAL_INITIAL"] = f"unique-boot-{index} READY\n"
        configs.append(config)
    with DedicatedSession() as first, DedicatedSession() as second:
        sessions = [first, second]
        boots = [start(session, config) for session, config in zip(sessions, configs, strict=True)]
        for session, (vm, _) in zip(sessions, boots, strict=True):
            session.call("vm_wait_for_boot", {"id": vm})
        for index, (_, handles) in enumerate(boots):
            text = Path(handles["live_log"]).read_text()
            assert f"unique-boot-{index}" in text
            assert f"unique-boot-{1 - index}" not in text
        assert boots[0][1]["live_log"] != boots[1][1]["live_log"]


def test_repeat_summary_selects_first_native_failure_and_keeps_all_logs(
    serial_config, tmp_path, monkeypatch
):
    from tools.qemu_infra_compat import call

    monkeypatch.setattr(artifacts, "ARTIFACT_ROOT", tmp_path / "runs")
    monkeypatch.setattr(artifacts, "FARM_DIR", tmp_path / "farm")
    monkeypatch.setenv("VIBEOS_ROOT", str(tmp_path / "ledger-root"))
    run = artifacts.Run("smoke", "arm64")
    results = []
    for index in range(3):
        config = copy.deepcopy(serial_config)
        config["work_dir"] = str(run.dir / f"boot{index}")
        serial = f"boot {index} complete serial\n" + (
            "*** BUGCHECK: 0x0000001E\n" if index == 1 else "READY\n"
        )
        config["env"]["SERIAL_INITIAL"] = serial
        with DedicatedSession() as session:
            vm, handles = start(session, config)
            text = session.call("vm_serial_output", {"id": vm, "timeout": 0.1})
            verdict = call(
                "verdict_classify",
                {
                    "serial": text,
                    "reached": "READY" in text,
                    "fail_marker": None,
                    "debugcon": None,
                    "known_flakes": [],
                },
            )
            reached = "READY" in text
            results.append(
                {
                    "ok": reached,
                    "classification": "" if reached else verdict["classification"],
                    "serial_log": handles["live_log"],
                    "run_dir": config["work_dir"],
                }
            )
    smoke._boot_evidence(run, results)
    run.add(artifacts.Stage("boot1", status="fail", detail=results[1]["classification"]))
    run.finalize()
    summary = json.loads((run.dir / "summary.json").read_text())
    assert "boot 1 complete serial" in (run.dir / "serial.log").read_text()
    assert summary["boots"][1]["classification"] == "BUGCHECK"
    assert len(summary["boots"]) == 3
    assert summary["serial_log"] == str(run.dir / "serial.log")
    for index, result in enumerate(results):
        assert f"boot {index} complete serial" in Path(result["serial_log"]).read_text()
