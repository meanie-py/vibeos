"""No post-filter can make entering generated output safe."""

import os
from pathlib import Path

import pytest

from tools.source_walk import source_files, source_walk


def test_prunes_before_scandir_and_preserves_static_inventory(tmp_path, monkeypatch):
    for rel in ("one/src/a.rs", "two/src/b.rs", "target/nested/generated.rs"):
        path = tmp_path / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("fn source() {}")
    expected = sorted(p for p in tmp_path.rglob("*.rs") if "target" not in p.parts)
    original = os.scandir
    visited = []

    def scan(path):
        visited.append(Path(path))
        if "target" in Path(path).parts:
            raise AssertionError("entered excluded output")
        return original(path)

    monkeypatch.setattr(os, "scandir", scan)
    assert sorted(source_files(tmp_path)) == expected
    assert tmp_path / "one/src" in visited  # cannot pass by walking nothing
    with pytest.raises(AssertionError, match="excluded"):
        # Filtering yielded paths cannot stop scandir from entering target.
        [parent for parent, _, _ in os.walk(tmp_path) if "target" not in Path(parent).parts]


@pytest.mark.parametrize("error", [FileNotFoundError, PermissionError])
def test_source_walk_errors_are_not_silently_skipped(tmp_path, monkeypatch, error):
    (tmp_path / "src").mkdir()
    original = os.scandir
    reached = []

    def scan(path):
        if Path(path).name == "src":
            reached.append(path)
            raise error("source cannot be scanned")
        return original(path)

    monkeypatch.setattr(os, "scandir", scan)
    with pytest.raises(error, match="source cannot"):
        list(source_files(tmp_path))
    assert reached


def test_oracle_prefix_exclusions_match_legacy_selection(tmp_path):
    for rel in (
        "drivers/probe",
        "sdk/lib/probe",
        "build-amd64/drivers/probe",
        "modules/rostests/probe",
        "sdk/lib/rust/probe",
        ".git/probe",
    ):
        (tmp_path / rel).mkdir(parents=True)
    skip = ("build-", "modules/rostests", "sdk/lib/rust", ".git")
    expected = sorted(
        p for p in tmp_path.rglob("probe") if not str(p.relative_to(tmp_path)).startswith(skip)
    )
    actual = sorted(
        parent / d
        for parent, dirs, _ in source_walk(tmp_path, exclude_prefixes=skip)
        for d in dirs
        if d == "probe"
    )
    assert actual == expected == [tmp_path / "drivers/probe", tmp_path / "sdk/lib/probe"]


def test_missing_root_fails(tmp_path):
    with pytest.raises(FileNotFoundError):
        list(source_files(tmp_path / "absent"))
