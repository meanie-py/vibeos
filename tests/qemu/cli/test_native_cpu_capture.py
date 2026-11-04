"""Real QMP socket responses exercise native diagnostic capture and deadlines."""

import json
from pathlib import Path

import pytest

from tools.test_infra_compat import DedicatedSession

from .test_native_vm import create
from .test_native_vm import fake_config as _fake_config

fake_config = _fake_config

REGS = "CPU#0\n PC=ffff800000100000 X29=ffff800000200000 SP=ffff800000300000 PSTATE=000003c5\n"
BASE = 0xFFFFF78000000000


def scripted(config, tmp_path, values):
    path = tmp_path / "qmp-script.json"
    path.write_text(json.dumps(values))
    config["env"]["FAKE_QMP_SCRIPT"] = str(path)


def test_native_capture_reads_every_cpu_stack_and_frame_chain(fake_config, tmp_path):
    scripted(
        fake_config,
        tmp_path,
        {
            "info registers -a": REGS + "CPU#1\n PC=ffff800000400000 SP=ffff800000500000\n",
            "x /2gx 0xffff800000200000": "ffff800000200000: 0xffff800000200010 0xffff800000100044",
            "x /2gx 0xffff800000200010": "ffff800000200010: 0x0 0xffff800000100088",
        },
    )
    with DedicatedSession() as session:
        vm = create(session, fake_config)
        session.call("vm_start", {"id": vm})
        capture = session.call("vm_capture_cpu_state", {"id": vm})
        assert "=== info status ===" in capture
        assert "CPU#0 x /6i 0xffff800000100000" in capture
        assert "CPU#1 x /6i 0xffff800000400000" in capture
        assert "CPU#0 stack x /23gx 0xffff800000300000" in capture
        assert "CPU#1 stack x /23gx 0xffff800000500000" in capture
        assert "ret=ffff800000100044" in capture and "ret=ffff800000100088" in capture
        commands = Path(fake_config["env"]["FAKE_QEMU_EVENTS"]).read_text().splitlines()
        assert "cpu 0" in commands


@pytest.mark.parametrize(
    "clock2,changed,verdict",
    [(16, False, "FROZEN"), (32, False, "IDLE-BLOCKED"), (16, True, "EXECUTING")],
)
def test_native_hang_resamples_clock_and_pcs(fake_config, tmp_path, clock2, changed, verdict):
    artifact = tmp_path / "durable"
    fake_config["env"]["VIBEOS_ARTIFACT_DIR"] = str(artifact)
    next_regs = REGS.replace("00100000", "00100004") if changed else REGS
    scripted(
        fake_config,
        tmp_path,
        {
            "info registers -a": [REGS, next_regs],
            f"x /3wx 0x{BASE + 8:x}": ["address: 0x10 0x0 0x0", f"address: 0x{clock2:x} 0x0 0x0"],
            "x /2gx 0xffff800000200000": "address: 0x0 0x0",
        },
    )
    with DedicatedSession() as session:
        vm = create(session, fake_config)
        session.call("vm_start", {"id": vm})
        first = session.call("vm_save_hang_state", {"id": vm, "phase": "boot watchdog"})
        assert session.call("vm_get_hang_cpu_state", {"id": vm}) == first
        work = Path(fake_config["work_dir"]) / "hang-cpustate.txt"
        text = work.read_text()
        assert verdict in text and "boot watchdog" in text
        assert (artifact / "hang-cpustate.txt").read_text() == text
        commands = Path(fake_config["env"]["FAKE_QEMU_EVENTS"]).read_text().splitlines()
        assert commands.count("info registers -a") == 2, "deadline getter must not resample"


def test_native_dead_monitor_produces_unknown_not_frozen(fake_config):
    with DedicatedSession() as session:
        vm = create(session, fake_config)
        session.call("vm_start", {"id": vm})
        session.call("vm_stop", {"id": vm})
        capture = session.call("vm_save_hang_state", {"id": vm, "phase": "dead monitor"})
        assert "unavailable" in capture
        assert "UNKNOWN" in (Path(fake_config["work_dir"]) / "hang-cpustate.txt").read_text()


@pytest.mark.parametrize(
    "clock,expected",
    [
        ("address: 0x9502f900 0x2 0x2", "InterruptTime=0x29502f900"),
        ("address: 0x10 0x3 0x2", "torn read"),
    ],
)
def test_native_kuser_clock_handles_high_word_and_torn_update(
    fake_config, tmp_path, clock, expected
):
    scripted(
        fake_config,
        tmp_path,
        {
            "info registers -a": REGS,
            f"x /3wx 0x{BASE + 8:x}": clock,
            f"x /1wx 0x{BASE + 0x320:x}": "address: 0x42",
            "x /2gx 0xffff800000200000": "address: 0x0 0x0",
        },
    )
    with DedicatedSession() as session:
        vm = create(session, fake_config)
        session.call("vm_start", {"id": vm})
        capture = session.call("vm_capture_cpu_state", {"id": vm})
        assert expected in capture
        if expected.startswith("InterruptTime"):
            assert "TickCount=0x42" in capture


def test_native_kernel_at_user_pc_overrides_frozen_hang(fake_config, tmp_path):
    scripted(
        fake_config,
        tmp_path,
        {"info registers -a": "CPU#0\n PC=000000007fff1234 SP=ffff800000300000 PSTATE=000003c5\n"},
    )
    with DedicatedSession() as session:
        vm = create(session, fake_config)
        session.call("vm_start", {"id": vm})
        session.call("vm_save_hang_state", {"id": vm, "phase": "fault loop"})
        assert "FAULT-LOOP" in (Path(fake_config["work_dir"]) / "hang-cpustate.txt").read_text()
