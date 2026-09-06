from ..serial_client import SerialClient
from ..vm import QemuVM, _write_sparse_image, get_arch_config
from . import artifacts, smoke


def test_overlapping_boots_have_independent_serial_streams(tmp_path):
    clients = []
    paths = []
    for i in range(2):
        vm = QemuVM.__new__(QemuVM)
        vm.config = get_arch_config("arm64")
        vm.live_log_path = tmp_path / f"boot{i}.log"
        vm.serial = SerialClient()
        vm._set_live_tee()
        clients.append(vm.serial)
        paths.append(vm.live_log_path)
    try:
        for i, client in enumerate(clients):
            client._tee.write(f"unique-boot-{i}\n")
            client._tee.flush()
        for i, path in enumerate(paths):
            text = path.read_text()
            assert f"unique-boot-{i}" in text
            assert f"unique-boot-{1 - i}" not in text
    finally:
        for client in clients:
            client.close()


def test_repeat_summary_preserves_first_failure_and_all_logs(tmp_path, monkeypatch):
    monkeypatch.setattr(artifacts, "ARTIFACT_ROOT", tmp_path / "runs")
    run = artifacts.Run("smoke", "arm64")

    def boot(run, repo, arch, **kwargs):
        idx = kwargs["idx"]
        work = run.dir / f"boot{idx}"
        work.mkdir()
        (work / "serial.log").write_text(f"boot {idx} complete serial")
        return {"ok": idx != 1, "classification": "BUGCHECK" if idx == 1 else ""}

    monkeypatch.setattr(smoke, "_boot_vm", boot)
    results = [smoke._one_boot(run, tmp_path, "arm64", idx=i) for i in range(3)]
    smoke._boot_evidence(run, results)
    artifacts.archive_serial(run.dir, run.extra["live_log"], run.extra)
    assert (run.dir / "serial.log").read_text() == "boot 1 complete serial"
    assert len(run.extra["boots"]) == 3
    for i in range(3):
        assert (run.dir / f"boot{i}/serial.log").read_text() == f"boot {i} complete serial"


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
