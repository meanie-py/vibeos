"""The walk must survive a directory deleted underneath it, and never enter target/.

These are the two properties the `source-early` move turned from theoretical into
load-bearing. A plain `rglob("*.rs")` fails BOTH, and fails them silently until a
build happens to be running at the same time — so the test reproduces the race
deterministically instead of hoping to catch it.
"""

from __future__ import annotations

import shutil
import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent))

from rust_sources import walk_files


def _tree(root: Path) -> None:
    (root / "src").mkdir(parents=True)
    (root / "src" / "keep.rs").write_text("// keep\n")
    (root / "src" / "nested").mkdir()
    (root / "src" / "nested" / "deep.rs").write_text("// deep\n")
    (root / "target" / "debug" / "build").mkdir(parents=True)
    (root / "target" / "debug" / "build" / "out.rs").write_text("// generated\n")
    (root / "build-i386").mkdir()
    (root / "build-i386" / "gen.rs").write_text("// generated\n")
    (root / "src" / "notrust.txt").write_text("nope\n")


def test_prunes_build_output(tmp_path: Path) -> None:
    _tree(tmp_path)
    got = {p.relative_to(tmp_path).as_posix() for p in walk_files([tmp_path])}
    assert got == {"src/keep.rs", "src/nested/deep.rs"}


def test_survives_directory_vanishing_mid_walk(tmp_path: Path) -> None:
    """The race, reproduced deterministically rather than hoped for.

    The walk is started, advanced by exactly one file so it is mid-flight, and then
    every directory it has not yet reached is deleted — which is what cargo does to a
    lint running in `source-early`. The old `rglob("*.rs")` spelling raises
    FileNotFoundError straight out of the generator here (pathlib's recursive selector
    catches only PermissionError), killing the scanner and reporting to the gate as a
    source-lint failure with no findings in it. `walk_files` must simply finish.

    The companion assertion is the point of the test: a fixed walk that quietly
    yielded nothing would also "not raise".
    """
    _tree(tmp_path)
    for i in range(200):
        sub = tmp_path / "src" / f"doomed{i:03d}"
        sub.mkdir()
        (sub / "gone.rs").write_text("// about to vanish\n")

    it = walk_files([tmp_path])
    first = next(it)
    for i in range(200):
        sub = tmp_path / "src" / f"doomed{i:03d}"
        if sub != first.parent and sub.exists():
            shutil.rmtree(sub)

    rest = list(it)  # must not raise
    names = {p.name for p in [first, *rest]}
    assert "keep.rs" in names, "walk aborted early instead of surviving the race"
    assert "out.rs" not in names


def test_old_rglob_spelling_actually_fails_this_race(tmp_path: Path) -> None:
    """Proves the test above discriminates; without this it could pass vacuously."""
    for i in range(200):
        sub = tmp_path / f"a{i:03d}"
        sub.mkdir()
        (sub / "x.rs").write_text("x")

    it = tmp_path.rglob("*.rs")  # unpruned-walk: intentional
    first = next(it)
    for i in range(200):
        sub = tmp_path / f"a{i:03d}"
        if sub != first.parent and sub.exists():
            shutil.rmtree(sub)

    with pytest.raises(OSError):
        list(it)


def test_accepts_a_file_root(tmp_path: Path) -> None:
    _tree(tmp_path)
    f = tmp_path / "src" / "keep.rs"
    assert list(walk_files([f])) == [f]
    assert list(walk_files([tmp_path / "src" / "notrust.txt"])) == []


def test_missing_root_is_not_fatal(tmp_path: Path) -> None:
    assert list(walk_files([tmp_path / "does-not-exist"])) == []


def test_order_is_deterministic(tmp_path: Path) -> None:
    """A scanner's output feeds a checked-in baseline, so filesystem order would
    surface as phantom baseline churn on an unrelated commit."""
    _tree(tmp_path)
    assert list(walk_files([tmp_path])) == list(walk_files([tmp_path]))
