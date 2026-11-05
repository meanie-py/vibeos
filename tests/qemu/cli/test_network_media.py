"""Networking launched by smoke must not rebuild the media it just validated."""

from tools import qemu_infra_compat as native

from ..vm import QemuVM, get_arch_config


def test_network_stage_preserves_validated_media(tmp_path):
    test = tmp_path / "tests/qemu/test_net.py"
    test.parent.mkdir(parents=True)
    test.write_text("")
    media = tmp_path / "livecd.iso"
    media.write_bytes(b"validated media")
    out = native.call(
        "smoke_fixture",
        {
            "mode": "operation",
            "operation": "smoke_run_net_stage",
            "options": {
                "run_dir": str(tmp_path),
                "arch": "amd64",
                "env": {"VIBEOS_QEMU_NVME": "1"},
            },
            "responses": {
                "command": [
                    {"returncode": 0, "stdout": "15 passed in 1s", "stderr": "", "timed_out": False}
                ]
            },
        },
        root=tmp_path,
    )
    command = next(c["kwargs"] for c in out["calls"] if c["op"] == "command")
    assert command["env"]["VIBEOS_ARTIFACT_DIR"] == str(tmp_path / "net")
    assert "VIBEOS_QEMU_NVME" not in command["env"]
    assert "--skip-clean-build" in command["argv"]
    assert media.read_bytes() == b"validated media"
    assert out["run"]["stages"][0]["status"] == "ok"
    assert any("15 passed" in c["kwargs"].get("text", "") for c in out["calls"] if c["op"] == "log")


def test_same_arch_vms_keep_independent_serial_logs(tmp_path, monkeypatch):
    monkeypatch.delenv("VIBEOS_ARTIFACT_DIR", raising=False)
    first = QemuVM(get_arch_config("amd64-uefi"), tmp_path / "first")
    second = QemuVM(get_arch_config("amd64-uefi"), tmp_path / "second")
    first.live_log_path.write_text("first boot")
    second.live_log_path.write_text("second boot")
    assert first.live_log_path.read_text() == "first boot"
    monkeypatch.setenv("VIBEOS_ARTIFACT_DIR", str(tmp_path / "durable"))
    durable = QemuVM(get_arch_config("amd64-uefi"), tmp_path / "third")
    assert durable.live_log_path == tmp_path / "durable/amd64-uefi_serial.log"
