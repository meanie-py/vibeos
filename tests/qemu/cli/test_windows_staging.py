"""Immutable media reuse and conservative Windows staging reclamation."""

import os
import shutil
import time
from pathlib import Path

import pytest

from .. import backend, windows_staging


@pytest.fixture
def staging(tmp_path, monkeypatch):
    monkeypatch.setattr(backend, "win_path", str)
    monkeypatch.setenv("VIBEOS_HOSTLOCK_DIR", str(tmp_path / "locks"))
    return tmp_path / "windows" / "run1"


def test_readonly_cache_pins_content_and_detects_same_metadata_rebuild(staging, tmp_path):
    source = tmp_path / "livecd.iso"
    source.write_bytes(b"first image")
    before = source.stat()
    first = Path(backend.stage_readonly_file(source, staging))
    second = Path(backend.stage_readonly_file(source, staging.parent / "run2"))
    assert first.samefile(second)
    source.write_bytes(b"other image")
    os.utime(source, ns=(before.st_atime_ns, before.st_mtime_ns))
    third = Path(backend.stage_readonly_file(source, staging.parent / "run3"))
    assert first.read_bytes() == b"first image"
    assert third.read_bytes() == b"other image"
    assert not first.samefile(third)


def test_writable_media_remains_private(staging, tmp_path):
    source = tmp_path / "vars.qcow2"
    source.write_bytes(b"firmware variables")
    args = [f"file={source},format=qcow2"]
    backend.stage_media_paths(args, staging)
    backend.stage_media_paths(args, staging.parent / "run2")
    first = staging / source.name
    second = staging.parent / "run2" / source.name
    assert not first.samefile(second)
    first.write_bytes(b"guest changes")
    assert second.read_bytes() == source.read_bytes()
    assert not (staging.parent / "media-cache-v1").exists()


def test_cache_hit_does_not_copy_again(staging, tmp_path, monkeypatch):
    source = tmp_path / "code.fd"
    source.write_bytes(b"firmware")
    backend.stage_media_paths([f"file={source},readonly=on"], staging)
    monkeypatch.setattr(backend, "stage_file", lambda *a: pytest.fail("warm cache copied media"))
    result = backend.stage_media_paths([f"file={source},readonly=on"], staging.parent / "run2")
    assert "run2/code.fd" in result[0]


def test_cache_prunes_only_unpinned_media(staging, tmp_path, monkeypatch):
    monkeypatch.setattr(backend, "_MEDIA_CACHE_LIMIT", 8)
    source = tmp_path / "boot.img"
    source.write_bytes(b"oldimage")
    first = Path(backend.stage_readonly_file(source, staging))
    cache = staging.parent / "media-cache-v1"
    source.write_bytes(b"newimage")
    backend.stage_readonly_file(source, staging.parent / "run2")
    assert len(list(cache.iterdir())) == 2  # both live runs pin their own bytes
    assert first.read_bytes() == b"oldimage"
    shutil.rmtree(staging)
    backend._prune_media_cache(cache, keep=cache)
    assert len(list(cache.iterdir())) == 1
    assert next(cache.iterdir()).read_bytes() == b"newimage"


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
