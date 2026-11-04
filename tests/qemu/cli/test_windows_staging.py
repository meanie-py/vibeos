"""Local publication controls and conservative Windows-only path/reclamation guards.

Concurrency, interruption, mutation and pruning controls live beside the same Rust
publication implementation; these API tests verify Python DTO bindings and bytes.
"""

import os
import time
from pathlib import Path

import pytest

from tools.qemu_infra_compat import NativeError, call

from .. import backend, windows_staging


def publish(source, staging, *, readonly=False):
    operation = "backend_publish_readonly_file" if readonly else "backend_publish_file"
    return Path(call(operation, {"src": str(source), "staging": str(staging)}))


@pytest.fixture
def staging(tmp_path, monkeypatch):
    monkeypatch.setenv("VIBEOS_HOSTLOCK_DIR", str(tmp_path / "locks"))
    return tmp_path / "windows/run1"


def test_readonly_cache_pins_content_and_detects_same_metadata_rebuild(staging, tmp_path):
    source = tmp_path / "livecd.iso"
    source.write_bytes(b"first image")
    before = source.stat()
    first = publish(source, staging, readonly=True)
    second = publish(source, staging.parent / "run2", readonly=True)
    assert first.samefile(second)
    source.write_bytes(b"other image")
    os.utime(source, ns=(before.st_atime_ns, before.st_mtime_ns))
    third = publish(source, staging.parent / "run3", readonly=True)
    assert first.read_bytes() == b"first image" and third.read_bytes() == b"other image"
    assert not first.samefile(third)


def test_writable_media_remains_private(staging, tmp_path):
    source = tmp_path / "vars.qcow2"
    source.write_bytes(b"firmware variables")
    first = publish(source, staging)
    second = publish(source, staging.parent / "run2")
    assert not first.samefile(second)
    first.write_bytes(b"guest changes")
    assert second.read_bytes() == source.read_bytes()
    assert not (staging.parent / "media-cache-v1").exists()


def test_local_publication_is_byte_exact_and_preserves_an_open_old_inode(staging, tmp_path):
    source = tmp_path / "boot.img"
    data = bytes(range(256)) * 4096 + b"partial final block"
    source.write_bytes(data)
    target = publish(source, staging)
    with target.open("rb") as old:
        source.write_bytes(b"new image")
        assert publish(source, staging) == target
        assert old.read() == data
    assert target.read_bytes() == b"new image"
    assert list(staging.iterdir()) == [target]


def test_windows_stage_still_refuses_a_linux_only_path(staging, tmp_path):
    source = tmp_path / "code.fd"
    source.write_bytes(b"firmware")
    with pytest.raises(NativeError, match="not under /mnt"):
        backend.stage_file(source, staging)
    with pytest.raises(NativeError, match="not under /mnt"):
        backend.stage_readonly_file(source, staging.parent / "run2")
    assert backend.win_path("/mnt/c/temp/run/code.fd") == "C:/temp/run/code.fd"


def _old(path):
    path.mkdir(parents=True)
    (path / "scratch.qcow2").write_bytes(b"scratch")
    old = time.time() - 7200
    os.utime(path, (old, old))
    return path


def test_cleanup_protects_linux_owner_windows_vm_recent_and_unknown(tmp_path, monkeypatch):
    root = tmp_path / "windows"
    dead = _old(root / "123-dead-boot0")
    linux = _old(root / "456-live-boot0")
    windows = _old(root / "dbg-live")
    recent = _old(root / "789-recent-boot0")
    recent.touch()
    unknown = _old(root / "human-files")
    outside = _old(tmp_path / "outside")
    (root / "999-symlink").symlink_to(outside, target_is_directory=True)
    monkeypatch.setattr(windows_staging, "_owner_alive", lambda pid: pid == 456)
    monkeypatch.setattr(backend, "win_path", lambda p: "C:/temp/" + p.name)
    monkeypatch.setattr(
        windows_staging,
        "windows_qemu_commands",
        lambda: ["qemu.exe -drive file=C:\\temp\\dbg-live\\scratch.qcow2"],
    )
    dry = windows_staging.cleanup_locked(root, dry_run=True)
    assert dry["eligible"] == [str(dead)]
    assert dead.exists()
    result = windows_staging.cleanup_locked(root)
    assert result["removed"] == [str(dead)]
    assert all(p.exists() for p in (linux, windows, recent, unknown, outside))


def test_cleanup_refuses_unverifiable_windows_processes(tmp_path, monkeypatch):
    root = tmp_path / "windows"
    dead = _old(root / "123-dead-boot0")
    monkeypatch.setattr(windows_staging, "_owner_alive", lambda pid: False)

    def fail():
        raise OSError("Windows process query failed")

    monkeypatch.setattr(windows_staging, "windows_qemu_commands", fail)
    with pytest.raises(OSError, match="process query"):
        windows_staging.cleanup_locked(root)
    assert dead.exists()


def test_cleanup_removes_dead_population_but_preserves_live_owner(tmp_path, monkeypatch):
    monkeypatch.setenv("VIBEOS_HOSTLOCK_DIR", str(tmp_path / "locks"))
    root = tmp_path / "windows"
    cache = root / "media-cache-v1"
    cache.mkdir(parents=True)
    live, dead, legacy = [
        cache / n for n in (".populate-v2-101-live", ".populate-v2-102-dead", ".fill-old")
    ]
    for path in (live, dead, legacy):
        path.mkdir()
        (path / "partial").write_bytes(b"bytes")
    monkeypatch.setattr(windows_staging, "_owner_alive", lambda pid: pid == 101)
    result = windows_staging.cleanup_locked(root, min_age=0)
    assert live.exists()
    assert str(live) in result["protected"]
    assert not dead.exists() and not legacy.exists()
