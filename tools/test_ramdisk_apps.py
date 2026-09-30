"""ramdisk_apps: the arm64 ramdisk's stock apps, resolved through the livecd manifest."""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import ramdisk_apps as ra


def _tree(tmp_path: Path, lines: list[str]) -> Path:
    build = tmp_path / "build-arm64"
    (build / "boot").mkdir(parents=True)
    (build / "boot" / "livecd.cmake.lst").write_text("unevaluated=$<TARGET_FILE:x>\n")
    (build / "boot" / "livecd.Debug.lst").write_text("\n".join(lines) + "\n")
    return build


def test_manifest_prefers_system32_and_maps_canonical_paths(tmp_path):
    build = _tree(
        tmp_path,
        [
            "reactos/notepad.exe=/tmp/vibeos/build-arm64/base/applications/notepad/notepad.exe",
            "reactos/system32/notepad.exe=/tmp/vibeos/build-arm64/base/applications/notepad/notepad.exe",
            "reactos/iexplore.exe=/tmp/vibeos/build-arm64/base/applications/iexplore/iexplore.exe",
            "reactos/inf/foo.inf=/tmp/vibeos/build-arm64/media/inf/foo.inf",
            "reactos/TEMP=/tmp/vibeos/build-arm64/boot/empty",
        ],
    )
    mods = ra.load_manifest(build)
    assert mods["notepad.exe"][0] == "reactos/system32/notepad.exe"
    assert mods["iexplore.exe"] == (
        "reactos/iexplore.exe",
        tmp_path / "build-arm64/base/applications/iexplore/iexplore.exe",
    )
    assert "foo.inf" not in mods  # not a PE module


def test_unbuilt_root_is_a_target_not_a_failure(tmp_path, monkeypatch):
    build = _tree(
        tmp_path,
        ["reactos/system32/calc.exe=/tmp/vibeos/build-arm64/base/applications/calc/calc.exe"],
    )
    roots = tmp_path / "apps.txt"
    roots.write_text("# comment\nreactos/system32/calc.exe\n")
    monkeypatch.setattr(ra, "REPO", tmp_path)
    monkeypatch.setattr(ra.closure, "find_objdump", lambda _repo: "objdump")
    out, unbuilt, unknown = ra.compute(build, roots)
    assert not out and not unknown
    assert list(unbuilt) == ["calc.exe"]


def test_root_missing_from_manifest_is_unknown(tmp_path, monkeypatch):
    build = _tree(tmp_path, [])
    roots = tmp_path / "apps.txt"
    roots.write_text("reactos/system32/nosuch.exe\n")
    monkeypatch.setattr(ra, "REPO", tmp_path)
    monkeypatch.setattr(ra.closure, "find_objdump", lambda _repo: "objdump")
    _out, _unbuilt, unknown = ra.compute(build, roots)
    assert "nosuch.exe" in unknown


def test_hand_staged_reads_mkramdisk_system32_dests():
    staged = ra.hand_staged(ra.REPO)
    assert {"notepad.exe", "shell32.dll", "comctl32.dll"} <= staged
