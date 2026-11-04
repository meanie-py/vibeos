"""Checkpoint guards: no fresh-boot fallback, shared writable files or false pass."""

import json

import pytest

from . import checkpoint as cp


def test_template_keeps_topology_clock_and_devices_but_replaces_transports():
    args = [
        "qemu",
        "-S",
        "-name",
        "old",
        "-qmp",
        "old.sock",
        "-smp",
        "4",
        "-rtc",
        "base=utc,clock=vm",
        "-accel",
        "tcg,thread=multi",
        "-device",
        "virtio-blk-device,drive=scratch",
        "-drive",
        "file=C:/temp/scratch.qcow2,format=qcow2,if=none,id=scratch",
    ]
    template, media = cp.guest_template(args)
    assert "old.sock" not in template
    assert template[:6] == ["-smp", "4", "-rtc", "base=utc,clock=vm", "-accel", "tcg,thread=multi"]
    assert media[0]["source"] == "/mnt/c/temp/scratch.qcow2"
    assert "@media-00.qcow2" in template[-1]


@pytest.mark.parametrize(
    "extra",
    [
        ["-netdev", "user,id=n"],
        ["-accel", "whpx"],
        ["-fsdev", "local,path=/host"],
        ["-fw_cfg", "name=test,file=/hidden"],
        ["-device", "usb-storage,drive=x"],
        ["-drive", "file=/writable.raw,format=raw"],
    ],
)
def test_unknown_external_state_and_accelerators_are_refused(extra):
    with pytest.raises(cp.CheckpointError):
        cp.guest_template(["qemu", "-drive", "file=/scratch,format=qcow2", *extra])


def test_corruption_refused_before_emulator_or_launch(tmp_path, monkeypatch):
    (tmp_path / "media-00.qcow2").write_bytes(b"damaged")
    (tmp_path / "manifest.json").write_text(
        json.dumps(
            {
                "schema": 1,
                "accel": "tcg",
                "backend": "windows-host",
                "media": [{"name": "media-00.qcow2", "sha256": "wrong"}],
            }
        )
    )
    monkeypatch.setattr(cp, "emulator_identity", lambda _: pytest.fail("guard bypassed"))
    with pytest.raises(cp.CheckpointError, match="damaged"):
        cp.load(tmp_path)
