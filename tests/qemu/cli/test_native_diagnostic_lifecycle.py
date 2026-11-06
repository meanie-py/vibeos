"""Real native ownership tests with external diagnostic-process authorities."""

import json
import os
import time
from pathlib import Path

import pytest

from tools.qemu_infra_compat import DedicatedSession, NativeError


@pytest.fixture
def diag_config(tmp_path):
    binaries = {}
    for name in ("fake_diagnostic_qemu", "fake_diagnostic_authority"):
        target = tmp_path / name
        target.write_text(Path(__file__).with_name(name + ".py").read_text())
        target.chmod(0o700)
        binaries[name] = str(target)
    env = {
        **os.environ,
        "VIBEOS_TEST_DIAGNOSTIC_AUTHORITY": binaries["fake_diagnostic_authority"],
        "VIBEOS_QEMU_PC_SAMPLE": "1",
        "DIAG_MODE": "none",
        "DIAG_PID": str(tmp_path / "sidecar.pid"),
        "DIAG_QEMU_PID": str(tmp_path / "qemu.pid"),
        "DIAG_EVENTS": str(tmp_path / "events"),
        "VIBEOS_QEMU_LOCK_DIR": str(tmp_path / "locks"),
        "VIBEOS_HOSTLOCK_DIR": str(tmp_path / "host"),
        "VIBEOS_BUILD_POLICY_DIR": str(tmp_path / "policy"),
        "VIBEOS_HOST_BROKER": "0",
    }
    for key in ("VIBEOS_QEMU_RR", "VIBEOS_QEMU_RR_SAMPLE", "VIBEOS_QEMU_RR_BREAK"):
        env.pop(key, None)
    return {
        "arch": "arm64",
        "qemu_bin": binaries["fake_diagnostic_qemu"],
        "work_dir": str(tmp_path / "work"),
        "env": env,
        "boot_timeout": 540,
        "hang_silence_seconds": 120,
        "max_lifetime": 30,
    }


def create(session, config):
    return session.call("vm_create", {"config": config})["id"]


def wait_file(path):
    deadline = time.monotonic() + 5
    while not path.is_file() and time.monotonic() < deadline:
        time.sleep(0.02)
    assert path.is_file(), f"missing fixture witness {path}"
    return path.read_text()


def wait_gone(pid):
    deadline = time.monotonic() + 10
    while time.monotonic() < deadline:
        path = Path(f"/proc/{pid}/stat")
        if not path.exists() or path.read_text().rsplit(")", 1)[1].split()[0] == "Z":
            return
        time.sleep(0.02)
    pytest.fail(f"owned diagnostic fixture {pid} survived owner cleanup")


@pytest.mark.parametrize("mode,fired", [("break", False), ("breakfire", True)])
def test_native_rr_break_is_sent_immediately_when_qmp_ready(diag_config, mode, fired):
    diag_config["env"]["DIAG_MODE"] = mode
    with DedicatedSession() as session:
        vm = create(session, diag_config)
        session.call("vm_start", {"id": vm})
        events = [
            json.loads(line)
            for line in Path(diag_config["env"]["DIAG_EVENTS"]).read_text().splitlines()
        ]
        assert [event["execute"] for event in events][:2] == ["qmp_capabilities", "replay-break"]
        assert events[1]["arguments"] == {"icount": 200}
        report = session.call("vm_report_replay_break", {"id": vm})
        assert report["icount"] == 200 and report["fired"] == fired
        # Serial deliberately contains no boot marker: arming did not await boot.


def test_native_ordinary_boot_reports_no_replay_break(diag_config):
    diag_config["env"].pop("VIBEOS_QEMU_PC_SAMPLE")
    with DedicatedSession() as session:
        vm = create(session, diag_config)
        session.call("vm_start", {"id": vm})
        assert session.call("vm_report_replay_break", {"id": vm}) is None


@pytest.mark.parametrize("silence,expected", [(0, 0), (120, 600)])
def test_native_rr_watchdog_raises_enabled_budget_and_preserves_disabled(
    diag_config, silence, expected
):
    diag_config["env"]["DIAG_MODE"] = "budgets"
    diag_config["hang_silence_seconds"] = silence
    with DedicatedSession() as session:
        vm = create(session, diag_config)
        handles = session.call("vm_start", {"id": vm})
        assert handles["config"]["boot_timeout"] >= 1800
        assert handles["config"]["hang_silence_seconds"] == expected


def test_native_x86_rr_demotes_accel_and_host_cpu_before_diagnostic_authority(diag_config):
    diag_config["arch"] = "amd64"
    diag_config["arch_args"] = ["-accel", "kvm", "-cpu", "host"]
    diag_config["env"]["VIBEOS_QEMU_RR"] = "record:/unused-by-fixture-authority"
    with DedicatedSession() as session:
        vm = create(session, diag_config)
        session.call("vm_start", {"id": vm})
        launch = json.loads(Path(diag_config["work_dir"]).joinpath("amd64_launch.json").read_text())
        argv = launch["argv"]
        assert "kvm" not in argv and "host" not in argv
        assert any("tcg" in arg for arg in argv)


def test_native_sidecar_failure_is_a_loud_diagnostic_error(diag_config):
    diag_config["env"]["DIAG_MODE"] = "fail"
    with DedicatedSession() as session:
        vm = create(session, diag_config)
        session.call("vm_start", {"id": vm})
        wait_file(Path(diag_config["env"]["DIAG_PID"]))
        deadline = time.monotonic() + 5
        while True:
            try:
                session.call("vm_poll_exit", {"id": vm})
            except NativeError as error:
                assert "requested VM diagnostic failed" in str(error)
                break
            assert time.monotonic() < deadline, "diagnostic failure was silently ignored"
            time.sleep(0.02)


def test_native_owner_death_reaps_diagnostic_and_qemu(diag_config):
    diag_config["env"]["DIAG_MODE"] = "hold"
    session = DedicatedSession()
    try:
        vm = create(session, diag_config)
        qemu = session.call("vm_start", {"id": vm})["qemu_pid"]
        sidecar = int(wait_file(Path(diag_config["env"]["DIAG_PID"])))
        server = session._session.process
        server.kill()
        server.wait(timeout=5)
        wait_gone(sidecar)
        wait_gone(qemu)
    finally:
        session.close()


def test_real_trace_guard_refuses_changed_media_before_qemu_launch(
    diag_config, tmp_path, monkeypatch
):
    from tests.qemu.cli import recordreplay as rr

    from .test_diagnostic_authorities import recorded

    trace, argv, _ = recorded(tmp_path, monkeypatch)
    (tmp_path / "boot.img").write_bytes(b"rebuilt after record")
    diag_config["arch_args"] = argv[1:]
    diag_config["env"].pop("VIBEOS_TEST_DIAGNOSTIC_AUTHORITY", None)
    diag_config["env"][rr.ENV_VAR] = f"replay:{trace}"
    diag_config["env"].pop("VIBEOS_QEMU_PC_SAMPLE", None)
    with DedicatedSession() as session:
        vm = create(session, diag_config)
        with pytest.raises(NativeError, match="CHANGED"):
            session.call("vm_start", {"id": vm})
    assert not Path(diag_config["env"]["DIAG_QEMU_PID"]).exists()
    assert not Path(diag_config["env"]["DIAG_EVENTS"]).exists()


def test_real_host_shape_guard_refuses_before_qemu_launch(diag_config, tmp_path, monkeypatch):
    from tests.qemu.cli import recordreplay as rr

    from .test_diagnostic_authorities import recorded

    trace, argv, _ = recorded(tmp_path, monkeypatch)
    # Recorded authority argv lacks native harness QMP/chardev transports. A
    # reconstructed launch cannot bypass that deliberately mismatched shape.
    diag_config["arch_args"] = argv[1:]
    diag_config["env"].pop("VIBEOS_TEST_DIAGNOSTIC_AUTHORITY", None)
    diag_config["env"][rr.ENV_VAR] = f"replay:{trace}"
    diag_config["env"].pop("VIBEOS_QEMU_PC_SAMPLE", None)
    with DedicatedSession() as session:
        vm = create(session, diag_config)
        with pytest.raises(NativeError, match="HOST-SIDE shape"):
            session.call("vm_start", {"id": vm})
    assert not Path(diag_config["env"]["DIAG_QEMU_PID"]).exists()


@pytest.mark.parametrize("empty", [False, True])
def test_shipping_native_pc_worker_samples_or_reports_failed_registers(diag_config, empty):
    """Exercise the default executable worker, including its second real QMP socket."""
    env = diag_config["env"]
    env.pop("VIBEOS_TEST_DIAGNOSTIC_AUTHORITY", None)
    # Any accidental return to the old Python worker fails before doing useful work.
    env["VIBEOS_TEST_PYTHON"] = "/no/python/diagnostic-worker"
    env["VIBEOS_QEMU_PC_SAMPLE"] = "0.05"
    if empty:
        env["DIAG_EMPTY_REGS"] = "1"
    with DedicatedSession() as session:
        vm = create(session, diag_config)
        session.call("vm_start", {"id": vm})
        work = Path(diag_config["work_dir"])
        if not empty:
            samples = work / "pcsamples.jsonl"
            deadline = time.monotonic() + 10
            while (
                not samples.is_file() or samples.stat().st_size == 0
            ) and time.monotonic() < deadline:
                session.call("vm_poll_exit", {"id": vm})
                time.sleep(0.02)
            rows = [json.loads(line) for line in samples.read_text().splitlines()]
            assert rows and rows[0]["pc"] == "0000000040001000"
        else:
            deadline = time.monotonic() + 10
            while True:
                try:
                    session.call("vm_poll_exit", {"id": vm})
                except NativeError as error:
                    assert "requested VM diagnostic failed" in str(error)
                    break
                assert time.monotonic() < deadline, "empty register sampler failed silently"
                time.sleep(0.02)
            reply = json.loads((work / "vm_diagnostic_sidecar-pc.json").read_text())
            assert "CPU" in reply["error"]["message"]


def test_diagnostic_workers_publish_separate_results(diag_config):
    diag_config["env"]["DIAG_MODE"] = "two-results"
    with DedicatedSession() as session:
        vm = create(session, diag_config)
        session.call("vm_start", {"id": vm})
        work = Path(diag_config["work_dir"])
        paths = [work / f"vm_diagnostic_sidecar-{kind}.json" for kind in ("pc", "rr")]
        deadline = time.monotonic() + 5
        while not all(p.is_file() and p.stat().st_size for p in paths):
            assert time.monotonic() < deadline, (
                "workers did not publish separate results; "
                f"files: {list(work.glob('vm_diagnostic_sidecar*.json'))}"
            )
            time.sleep(0.02)
        results = [json.loads(p.read_text())["ok"] for p in paths]
        assert results == [{"kind": "pc", "samples": 123456}, {"kind": "rr", "samples": 1}]


@pytest.mark.parametrize("failed", [False, True])
def test_finite_replay_checker_finishes_before_vm_teardown(diag_config, tmp_path, failed):
    finished = tmp_path / "comparison-finished"
    diag_config["env"].update(
        DIAG_MODE="finish-fail" if failed else "finish",
        DIAG_FINISHED=str(finished),
        VIBEOS_QEMU_RR_SAMPLE="check",
    )
    with DedicatedSession() as session:
        vm = create(session, diag_config)
        session.call("vm_start", {"id": vm})
        if failed:
            with pytest.raises(NativeError, match="requested replay comparison failed"):
                session.call("vm_finish_diagnostics", {"id": vm})
        else:
            assert session.call("vm_finish_diagnostics", {"id": vm}) == {"checkers": 1}
        assert finished.read_text() == "complete"
        # The continuous PC worker must not hold up finite checker completion.
        if not failed:
            assert session.call("vm_poll_exit", {"id": vm}) is None
