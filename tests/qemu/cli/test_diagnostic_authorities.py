"""Real RR authority inputs replacing removed Python QemuVM hook tests."""

import json
import os
import struct

import pytest

from tests.qemu import native_diagnostics
from tests.qemu.cli import recordreplay as rr
from tests.qemu.cli.test_recordreplay import _base_cmd


@pytest.fixture(autouse=True)
def clean_diagnostics(monkeypatch):
    for key in (rr.ENV_VAR, rr.BREAK_ENV_VAR, "VIBEOS_QEMU_RR_SAMPLE", "VIBEOS_QEMU_PC_SAMPLE"):
        monkeypatch.delenv(key, raising=False)


def request(tmp_path, argv=None):
    return {
        "argv": argv or _base_cmd(tmp_path),
        "arch": "arm64",
        "backend": "native",
        "env": dict(os.environ),
        "sock_dir": str(tmp_path / "sockets"),
        "work_dir": str(tmp_path / "work"),
        "live_log": str(tmp_path / "serial.log"),
    }


def recorded(tmp_path, monkeypatch):
    trace = tmp_path / "trace"
    argv = _base_cmd(tmp_path)
    monkeypatch.setenv(rr.ENV_VAR, f"record:{trace}")
    result = native_diagnostics.prepare(request(tmp_path, argv))
    # The hook must not bank pre-boot media as pristine: finalization owns it.
    assert not (trace / "pristine").exists()
    manifest = json.loads((trace / "manifest.json").read_text())
    manifest["finalised"] = True
    (trace / "manifest.json").write_text(json.dumps(manifest))
    (trace / "replay.bin").write_bytes(
        struct.pack(">IQ", rr._SUPPORTED_REPLAY_VERSION, 0)
        + struct.pack(">BI", 0, 100) * 2
        + bytes([14])
    )
    return trace, argv, result


def test_diagnostics_off_leaves_argv_unchanged(tmp_path):
    argv = ["qemu-system-aarch64", "-M", "virt"]
    result = native_diagnostics.prepare(request(tmp_path, argv))
    assert result == {"argv": argv, "sidecars": []}


def test_rr_authority_never_freezes_pristine_before_recording_finishes(tmp_path, monkeypatch):
    trace, _, _ = recorded(tmp_path, monkeypatch)
    assert not (trace / "pristine").exists()
    rr.snapshot_pristine(trace)
    assert (trace / "pristine").is_dir()


def test_replay_check_trace_refuses_changed_media_before_noting_a_run(tmp_path, monkeypatch):
    trace, argv, _ = recorded(tmp_path, monkeypatch)
    before = (trace / "last-run.json").read_bytes() if (trace / "last-run.json").exists() else None
    (tmp_path / "boot.img").write_bytes(b"rebuilt source")
    monkeypatch.setenv(rr.ENV_VAR, f"replay:{trace}")
    with pytest.raises(rr.RecordReplayError, match="CHANGED"):
        native_diagnostics.prepare(request(tmp_path, argv))
    after = (trace / "last-run.json").read_bytes() if (trace / "last-run.json").exists() else None
    assert after == before


def test_replay_assert_same_shape_refuses_host_changes(tmp_path, monkeypatch):
    trace, argv, _ = recorded(tmp_path, monkeypatch)
    monkeypatch.setenv(rr.ENV_VAR, f"replay:{trace}")
    with pytest.raises(rr.RecordReplayError, match="HOST-SIDE shape"):
        native_diagnostics.prepare(
            request(tmp_path, [*argv, "-serial", f"file:{tmp_path}/different.log"])
        )


def test_break_is_snapped_by_real_trace_authority_and_absent_when_not_requested(
    tmp_path, monkeypatch, capsys
):
    trace, argv, _ = recorded(tmp_path, monkeypatch)
    monkeypatch.setenv(rr.ENV_VAR, f"replay:{trace}")
    assert "rr_break_icount" not in native_diagnostics.prepare(request(tmp_path, argv))
    monkeypatch.setenv(rr.BREAK_ENV_VAR, "150")
    assert native_diagnostics.prepare(request(tmp_path, argv))["rr_break_icount"] == 200
    assert "mid-chunk" in capsys.readouterr().err
    monkeypatch.setenv(rr.BREAK_ENV_VAR, "999999")
    with pytest.raises(rr.RecordReplayError, match="past the end"):
        native_diagnostics.prepare(request(tmp_path, argv))


def test_rr_watchdog_minima_are_authority_metadata(tmp_path, monkeypatch):
    _, _, result = recorded(tmp_path, monkeypatch)
    assert result["boot_timeout_min"] >= 1800
    assert result["hang_silence_min"] >= 600


def test_arm64_record_authority_keeps_cpu_model(tmp_path, monkeypatch):
    _, _, result = recorded(tmp_path, monkeypatch)
    assert "cortex-a72" in result["argv"]


def test_rr_sampler_requires_rr_and_the_matching_mode(tmp_path, monkeypatch):
    monkeypatch.setenv("VIBEOS_QEMU_RR_SAMPLE", "1")
    with pytest.raises(RuntimeError, match="requires VIBEOS_QEMU_RR"):
        native_diagnostics.prepare(request(tmp_path))
    trace = tmp_path / "trace"
    monkeypatch.setenv(rr.ENV_VAR, f"record:{trace}")
    monkeypatch.setenv("VIBEOS_QEMU_RR_SAMPLE", "check")
    with pytest.raises(RuntimeError, match="expects seconds for record"):
        native_diagnostics.prepare(request(tmp_path))


def test_pc_sampler_gets_a_dedicated_qmp_endpoint(tmp_path, monkeypatch):
    monkeypatch.setenv("VIBEOS_QEMU_PC_SAMPLE", ".05")
    r = native_diagnostics.prepare(request(tmp_path, ["qemu-system-aarch64", "-M", "virt"]))
    assert len(r["sidecars"]) == 1
    sidecar = r["sidecars"][0]
    assert sidecar["kind"] == "pc" and sidecar["period"] == 0.05
    assert sidecar["sock"] == str(tmp_path / "sockets/pcsample.sock")
    assert r["argv"][-2:] == ["-qmp", f"unix:{sidecar['sock']},server,nowait"]
