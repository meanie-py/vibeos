"""Actual FAT fixtures prove native TESTKERNEL caching and immutable publication."""

import os
import subprocess

from tests.qemu.vm import testkernel_boot_img as kernel_image


def fat_fixture(tmp_path, marker="initial"):
    image = tmp_path / "boot.img"
    image.write_bytes(bytes(2 * 1024 * 1024))
    subprocess.run(["mformat", "-i", str(image), "::"], check=True, capture_output=True)
    write_ini(tmp_path, image, marker)
    return image


def write_ini(work, image, marker):
    ini = work / "input.ini"
    ini.write_text(f"[FREELOADER]\n/SOS /RDPATH=x\n; {marker}\n")
    subprocess.run(
        ["mcopy", "-o", "-i", str(image), str(ini), "::/freeldr.ini"],
        check=True,
        capture_output=True,
    )


def test_testkernel_cache_reuses_an_unchanged_source(tmp_path):
    source = fat_fixture(tmp_path)
    original = source.read_bytes()
    first = kernel_image(source, tmp_path / "first")
    before = first.stat()
    second = kernel_image(source, tmp_path / "second")
    after = second.stat()
    assert first == second and (before.st_ino, before.st_mtime_ns) == (
        after.st_ino,
        after.st_mtime_ns,
    )
    assert source.read_bytes() == original
    ini = subprocess.run(
        ["mcopy", "-i", str(first), "::/freeldr.ini", "-"], check=True, capture_output=True
    ).stdout
    assert b"/TESTKERNEL" in ini


def test_testkernel_cache_rekeys_after_source_change(tmp_path):
    source = fat_fixture(tmp_path)
    first = kernel_image(source, tmp_path / "first")
    write_ini(tmp_path, source, "a rebuilt source")
    before = source.stat()
    os.utime(source, ns=(before.st_atime_ns, before.st_mtime_ns + 1_000_000))
    second = kernel_image(source, tmp_path / "second")
    assert first != second
    ini = subprocess.run(
        ["mcopy", "-i", str(second), "::/freeldr.ini", "-"], check=True, capture_output=True
    ).stdout
    assert b"a rebuilt source" in ini and b"/TESTKERNEL" in ini


def test_testkernel_publication_preserves_an_open_old_inode(tmp_path):
    source = fat_fixture(tmp_path)
    first = kernel_image(source, tmp_path / "first")
    with first.open("rb") as existing:
        old_bytes = existing.read()
        write_ini(tmp_path, source, "new bytes after the first boot opened its media")
        before = source.stat()
        os.utime(source, ns=(before.st_atime_ns, before.st_mtime_ns + 1_000_000))
        second = kernel_image(source, tmp_path / "second")
        assert first != second
        existing.seek(0)
        assert existing.read() == old_bytes
        assert second.read_bytes() != old_bytes
