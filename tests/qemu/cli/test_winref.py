"""winref's disk-reading arithmetic: both failures here are silent until 7-Zip says
"not an archive", 80 GB of copying later."""

from __future__ import annotations

import pytest

from tests.qemu.cli import winref

# `7z l -slt` of a Windows 11 install disk, as the host 7-Zip printed it (2026-10-11).
LISTING = """\
Path = D:\\vibeos-winref\\install.qcow2
Size = 85899345920

Path = install.gpt

Path = 0.EFI system partition.img
Size = 314572800
Offset = 1048576

Path = 1.Microsoft reserved partition.img
Size = 16777216
Offset = 315621376

Path = 2.Basic data partition.ntfs
Size = 84629520384
Offset = 332398592

Path = 3.ntfs
Size = 935329792
Offset = 84961918976
"""


def test_picks_the_windows_partition_not_the_recovery_one():
    assert winref.largest_ntfs(LISTING) == (332398592, 84629520384)


def test_handles_crlf_output():
    assert winref.largest_ntfs(LISTING.replace("\n", "\r\n")) == (332398592, 84629520384)


def test_no_ntfs_is_zero_size():
    assert winref.largest_ntfs("Path = x.img\nSize = 1\nOffset = 0\n")[1] == 0


def test_dd_count_includes_the_skip():
    # The partition is 80709 MiB at offset 317 MiB. A count of 80709 alone copied 317 MiB
    # short (qemu-img's dd counts skipped blocks) and lost NTFS's backup boot sector.
    assert winref.dd_window(332398592, 84629520384) == (317, 317 + 80709)


def test_dd_rounds_a_partial_last_mib_up():
    assert winref.dd_window(1 << 20, (1 << 20) + 1) == (1, 3)


def test_dd_refuses_an_unaligned_partition():
    with pytest.raises(ValueError):
        winref.dd_window(512, 1 << 20)


def _fake_base(d, content: bytes) -> None:
    d.mkdir(parents=True, exist_ok=True)
    (d / "base.qcow2").write_bytes(content)
    (d / "base-vars.fd").write_bytes(b"vars-" + content)
    (d / "state.json").write_text('{"setup": "build=26200"}')


def test_snapshot_survives_a_reinstall_and_restores(tmp_path, monkeypatch):
    """The point of the snapshot: a later `install --force` rewrites base.qcow2, and a
    ~2 h TCG install is too expensive to lose to that."""
    monkeypatch.setattr(winref, "WINREF_DIR", tmp_path)
    _fake_base(tmp_path, b"pristine")
    snap = winref._snapshot(winref.POST_INSTALL)
    assert (snap / "base.qcow2").read_bytes() == b"pristine"
    _fake_base(tmp_path, b"clobbered")  # what a reinstall or a base change does
    args = type("A", (), {"name": winref.POST_INSTALL})()
    assert winref.cmd_restore(args, tmp_path) == 0
    assert (tmp_path / "base.qcow2").read_bytes() == b"pristine"
    assert (tmp_path / "base-vars.fd").read_bytes() == b"vars-pristine"


def test_snapshot_never_overwrites_silently(tmp_path, monkeypatch):
    monkeypatch.setattr(winref, "WINREF_DIR", tmp_path)
    _fake_base(tmp_path, b"one")
    winref._snapshot("x")
    _fake_base(tmp_path, b"two")
    with pytest.raises(FileExistsError):
        winref._snapshot("x")
    assert (tmp_path / "snapshots" / "x" / "base.qcow2").read_bytes() == b"one"
    winref._snapshot("x", overwrite=True)
    assert (tmp_path / "snapshots" / "x" / "base.qcow2").read_bytes() == b"two"


def test_types_the_maint_command_exactly():
    """A wrong chord types a different path into Win+R and the maintenance silently never
    runs, so the drive colon and backslash are pinned."""
    chords = winref.keys_for(winref.MAINT_COMMAND)
    assert len(chords) == len(winref.MAINT_COMMAND)
    assert chords[winref.MAINT_COMMAND.index(":")] == ["shift", "semicolon"]
    assert chords[winref.MAINT_COMMAND.index("\\")] == ["backslash"]
    assert chords[winref.MAINT_COMMAND.index("E")] == ["shift", "e"]


def test_refuses_to_type_what_it_cannot():
    with pytest.raises(ValueError):
        winref.keys_for("é")
