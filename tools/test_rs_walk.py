"""rs_walk — the traversal six scanners share. Both bugs it exists to prevent are races or
performance cliffs, so both tests assert on what is VISITED, not on what comes back: filtering
after the walk returns the same files, which is precisely why the bug survived review.
"""

from __future__ import annotations

import os
from pathlib import Path

import rs_walk


def _tree(root: Path, rels: list[str]) -> None:
    for rel in rels:
        p = root / rel
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text("")


def test_pruned_trees_are_never_descended(monkeypatch, tmp_path):
    _tree(
        tmp_path,
        [
            "src/lib.rs",
            "target/x86_64-pc-windows-gnu/debug/build/foo/out/generated.rs",
            ".responses/20260906-file/body.rs",
            ".git/hooks/nothing.rs",
        ],
    )
    visited: list[str] = []
    real_walk = os.walk

    def spy(top, **kw):
        for root, dirs, files in real_walk(top, **kw):
            visited.append(str(Path(root).relative_to(tmp_path)))
            yield root, dirs, files

    monkeypatch.setattr(rs_walk.os, "walk", spy)
    got = sorted(str(p.relative_to(tmp_path)) for p in rs_walk.iter_rs_files(tmp_path))
    assert got == ["src/lib.rs"]
    # Anti-vacuity: an rglob implementation never calls os.walk, so `visited` would be empty
    # and the exclusion assertion below would pass by finding nothing at all.
    assert visited, "iter_rs_files must walk-and-prune"
    assert not [v for v in visited if v.split("/")[0] in ("target", ".responses", ".git")]


def test_a_directory_vanishing_mid_walk_is_not_fatal(monkeypatch, tmp_path):
    """A build deleting a directory under the walker must not kill the scan. A scanner that
    dies on a concurrent build reports host timing as a source defect — and the gate renders
    that crash as "NEW lint findings (exit 1)" with no findings in it."""
    _tree(tmp_path, ["src/lib.rs", "src/doomed/gone.rs"])
    doomed = tmp_path / "src" / "doomed"
    real_walk = os.walk

    def spy(top, **kw):
        for root, dirs, files in real_walk(top, **kw):
            if doomed.exists():
                (doomed / "gone.rs").unlink()
                doomed.rmdir()  # listed, then gone before it is descended
            yield root, dirs, files

    monkeypatch.setattr(rs_walk.os, "walk", spy)
    got = sorted(str(p.relative_to(tmp_path)) for p in rs_walk.iter_rs_files(tmp_path))
    assert got == ["src/lib.rs"]
    assert not doomed.exists(), "the race never happened, so this proved nothing"


def test_skip_generated_drops_machine_written_files_only():
    """`bindgen_*.rs` and `build.rs` hold no hand-translation bugs; nothing else is exempt."""
    import tempfile

    with tempfile.TemporaryDirectory() as td:
        root = Path(td)
        _tree(root, ["a/bindgen_i386.rs", "a/build.rs", "a/real.rs", "a/rebuild.rs"])
        assert sorted(p.name for p in rs_walk.iter_rs_files(root, skip_generated=True)) == [
            "real.rs",
            "rebuild.rs",
        ]
        assert len(list(rs_walk.iter_rs_files(root))) == 4
