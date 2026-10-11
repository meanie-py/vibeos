"""mk_writable_livecd's freeldr.ini rewrite. Its failure mode is a FALSE writable root: an
entry left on \\reactos boots the read-only CD and every file kmtest bails exactly as before,
so the "fix" would re-seed the same partial rows. Pin that both selectable entries move."""
from __future__ import annotations

import importlib.util
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location(
    "mk_writable_livecd", ROOT / "tools/conformance/mk_writable_livecd.py"
)
assert spec is not None and spec.loader is not None
mk = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mk)


def test_rewrites_the_real_livecd_ini() -> None:
    text = (ROOT / "boot/bootdata/livecd.ini").read_bytes().decode("latin-1")
    out = mk.rewrite_ini(text)
    sections: dict[str, list[str]] = {}
    cur = None
    for line in out.splitlines():
        if line.startswith("["):
            cur = line.strip("[]")
        elif cur:
            sections.setdefault(cur, []).append(line)
    for entry in mk.ENTRIES:
        assert r"SystemPath=ramdisk(0)\reactos" in sections[entry]
        opts = [o for o in sections[entry] if o.startswith("Options=")]
        assert opts and all(o.endswith(r"/RDPATH=\ramdisk.img") for o in opts)
    # Entries this does not own stay on the CD, and CRLF survives.
    assert r"SystemPath=\reactos" in sections["LiveCD"]
    assert out.count("\r\n") == text.count("\r\n")


def test_refuses_an_unexpected_system_path() -> None:
    bad = (
        "[LiveCD_SMP]\r\nSystemPath=\\other\r\nOptions=/X\r\n"
        "[LiveCD_Debug]\r\nSystemPath=\\reactos\r\nOptions=/Y\r\n"
    )
    with pytest.raises(SystemExit):
        mk.rewrite_ini(bad)


def test_refuses_a_missing_entry() -> None:
    with pytest.raises(SystemExit):
        mk.rewrite_ini("[LiveCD_SMP]\r\nSystemPath=\\reactos\r\nOptions=/X\r\n")


def test_stages_outside_the_build_dir(tmp_path, monkeypatch) -> None:
    """The livecd tree copy must never appear inside build-<arch>: the amd64 WoW64 restage
    walks build-i386 and died on a file the staging cleanup removed mid-walk."""
    seen: dict[str, object] = {}

    class Stop(Exception):
        pass

    def recorder(*args, **kwargs):
        seen.update(kwargs)
        raise Stop

    monkeypatch.setattr(mk.shutil, "which", lambda _tool: "/bin/true")
    monkeypatch.setattr(mk.tempfile, "TemporaryDirectory", recorder)
    build_dir = tmp_path / "build-i386"
    build_dir.mkdir()
    with pytest.raises(Stop):
        mk.build(build_dir / "livecd.iso", build_dir / mk.OUT_NAME)
    staged = seen.get("dir")
    assert staged is None or build_dir not in Path(str(staged)).resolve().parents, seen
