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


def test_distinct_cold_fills_overlap(tmp_path, monkeypatch):
    """A barrier inside the actual copy path would deadlock under one cache lock."""
    import threading
    from concurrent.futures import ThreadPoolExecutor

    monkeypatch.setenv("VIBEOS_HOSTLOCK_DIR", str(tmp_path / "locks"))
    monkeypatch.setattr(backend, "win_path", str)
    sources = [tmp_path / "one.iso", tmp_path / "two.iso"]
    for i, source in enumerate(sources):
        source.write_bytes(bytes([i]) * 4096)
    barrier = threading.Barrier(2)
    copy = backend.stage_file

    def overlap(source, staging):
        barrier.wait(timeout=5)
        return copy(source, staging)

    monkeypatch.setattr(backend, "stage_file", overlap)
    with ThreadPoolExecutor(max_workers=2) as pool:
        futures = [
            pool.submit(backend.stage_readonly_file, s, tmp_path / "windows" / str(i))
            for i, s in enumerate(sources)
        ]
        results = [Path(f.result(timeout=10)) for f in futures]
    assert [p.read_bytes() for p in results] == [s.read_bytes() for s in sources]


def test_warm_hit_and_cleanup_do_not_wait_for_cold_copy(tmp_path, monkeypatch):
    import threading
    from concurrent.futures import ThreadPoolExecutor

    monkeypatch.setenv("VIBEOS_HOSTLOCK_DIR", str(tmp_path / "locks"))
    monkeypatch.setattr(backend, "win_path", str)
    root = tmp_path / "windows"
    warm, cold = tmp_path / "warm.iso", tmp_path / "cold.iso"
    warm.write_bytes(b"warm")
    cold.write_bytes(b"cold")
    backend.stage_readonly_file(warm, root / "first")
    entered, finish = threading.Event(), threading.Event()
    copy = backend.stage_file

    def slow(source, staging):
        entered.set()
        assert finish.wait(timeout=10)
        return copy(source, staging)

    monkeypatch.setattr(backend, "stage_file", slow)
    with ThreadPoolExecutor(max_workers=2) as pool:
        future = pool.submit(backend.stage_readonly_file, cold, root / "cold-run")
        try:
            assert entered.wait(timeout=5)
            hit = pool.submit(backend.stage_readonly_file, warm, root / "warm-run")
            assert Path(hit.result(timeout=5)).read_bytes() == b"warm"
            sweep = pool.submit(windows_staging.cleanup_locked, root, min_age=0)
            result = sweep.result(timeout=5)
            assert any(".populate-v2-" in p for p in result["protected"])
        finally:
            finish.set()
        assert Path(future.result(timeout=5)).read_bytes() == b"cold"


def test_same_entry_is_copied_once(tmp_path, monkeypatch):
    import threading
    from concurrent.futures import ThreadPoolExecutor

    monkeypatch.setenv("VIBEOS_HOSTLOCK_DIR", str(tmp_path / "locks"))
    monkeypatch.setattr(backend, "win_path", str)
    source = tmp_path / "disk.iso"
    source.write_bytes(b"shared")
    barrier = threading.Barrier(4)
    copies = []
    copy = backend.stage_file

    def counted(src, staging):
        copies.append(src)
        return copy(src, staging)

    def stage(i):
        barrier.wait(timeout=5)
        return Path(backend.stage_readonly_file(source, tmp_path / "windows" / str(i)))

    monkeypatch.setattr(backend, "stage_file", counted)
    with ThreadPoolExecutor(max_workers=4) as pool:
        outputs = list(pool.map(stage, range(4)))
    assert copies == [source]
    assert all(p.samefile(outputs[0]) for p in outputs)
    assert outputs[0].read_bytes() == b"shared"


def test_legacy_publication_is_never_overwritten(tmp_path, monkeypatch):
    import hashlib

    monkeypatch.setenv("VIBEOS_HOSTLOCK_DIR", str(tmp_path / "locks"))
    monkeypatch.setattr(backend, "win_path", str)
    source = tmp_path / "disk.iso"
    source.write_bytes(b"immutable")
    root = tmp_path / "windows"
    entry = root / "media-cache-v1" / f"{hashlib.sha256(b'immutable').hexdigest()}-disk.iso"
    pin = root / "old-launcher-pin"
    copy = backend.stage_file

    def legacy_wins(src, staging):
        result = copy(src, staging)
        with backend.media_cache_lock():
            entry.write_bytes(b"immutable")
            os.link(entry, pin)
        return result

    monkeypatch.setattr(backend, "stage_file", legacy_wins)
    result = Path(backend.stage_readonly_file(source, root / "new-run"))
    assert result.samefile(pin)


def test_failed_fill_does_not_publish_partial_media(tmp_path, monkeypatch):
    monkeypatch.setenv("VIBEOS_HOSTLOCK_DIR", str(tmp_path / "locks"))
    monkeypatch.setattr(backend, "win_path", str)
    source = tmp_path / "disk.iso"
    source.write_bytes(b"complete")
    root = tmp_path / "windows"

    def fail(src, staging):
        (staging / src.name).write_bytes(b"partial")
        raise OSError("copy failed")

    monkeypatch.setattr(backend, "stage_file", fail)
    with pytest.raises(OSError, match="copy failed"):
        backend.stage_readonly_file(source, root / "run")
    assert list((root / "media-cache-v1").iterdir()) == []
    assert not (root / "run" / source.name).exists()


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


def test_crashed_fill_releases_lock_and_can_be_reclaimed(tmp_path, monkeypatch):
    import subprocess
    import sys

    monkeypatch.setenv("VIBEOS_HOSTLOCK_DIR", str(tmp_path / "locks"))
    monkeypatch.setattr(backend, "win_path", str)
    source = tmp_path / "disk.iso"
    source.write_bytes(b"complete")
    root = tmp_path / "windows"
    code = """
import os, sys
from pathlib import Path
from tests.qemu import backend
backend.win_path = str
def crash(src, staging):
    (staging / src.name).write_bytes(b"partial")
    os._exit(9)
backend.stage_file = crash
backend.stage_readonly_file(Path(sys.argv[1]), Path(sys.argv[2]))
"""
    result = subprocess.run(
        [sys.executable, "-c", code, str(source), str(root / "dead")], timeout=10, check=False
    )
    assert result.returncode == 9
    fills = list((root / "media-cache-v1").glob(".populate-v2-*"))
    assert len(fills) == 1
    removed = windows_staging.cleanup_locked(root, min_age=0)["removed"]
    assert str(fills[0]) in removed
    output = Path(backend.stage_readonly_file(source, root / "retry"))
    assert output.read_bytes() == b"complete"


def test_source_change_during_fill_is_not_published(tmp_path, monkeypatch):
    monkeypatch.setenv("VIBEOS_HOSTLOCK_DIR", str(tmp_path / "locks"))
    monkeypatch.setattr(backend, "win_path", str)
    source = tmp_path / "disk.iso"
    source.write_bytes(b"original")
    stamp = source.stat()
    root = tmp_path / "windows"
    copy = backend.stage_file

    def changed(src, staging):
        result = copy(src, staging)
        src.write_bytes(b"mutation")
        os.utime(src, ns=(stamp.st_atime_ns, stamp.st_mtime_ns))
        return result

    monkeypatch.setattr(backend, "stage_file", changed)
    with pytest.raises(RuntimeError, match="changed before publication"):
        backend.stage_readonly_file(source, root / "run")
    assert list((root / "media-cache-v1").iterdir()) == []
    assert not (root / "run" / source.name).exists()
