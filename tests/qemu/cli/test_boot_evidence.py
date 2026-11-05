from ..vm import _write_sparse_image
from . import artifacts, smoke


def test_launch_failure_never_borrows_another_boot_log(tmp_path, monkeypatch):
    monkeypatch.setattr(artifacts, "ARTIFACT_ROOT", tmp_path / "runs")
    run = artifacts.Run("smoke", "arm64")
    smoke._boot_evidence(
        run,
        [
            {"ok": True, "serial_log": "another-boot.log"},
            {"ok": False, "classification": "LAUNCH_FAIL"},
        ],
    )
    assert "live_log" not in run.extra


def test_sparse_media_is_byte_exact_including_trailing_holes(tmp_path):
    path = tmp_path / "boot.img"
    for data in (b"", bytes(17), bytes(131072), b"head" + bytes(196608) + b"tail" + bytes(65537)):
        path.write_bytes(b"old longer contents" * 20000)
        _write_sparse_image(path, data)
        assert path.read_bytes() == data
