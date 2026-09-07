"""No post-filter can make entering generated output safe."""

import os
import shutil
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


def test_old_rglob_spelling_actually_fails_this_race(tmp_path):
    """The control this module rests on: prove the spelling it replaced really breaks.

    Without this, every test above could pass against an unfixed tree and prove
    nothing. It is worth stating how easily this class is dismissed: a casual
    delete-a-directory-mid-walk test PASSES against `rglob`, because the window is
    narrow and easy to miss entirely. It has to be forced — start the walk, advance it
    by exactly one file so it is mid-flight, then delete every directory it has not yet
    reached. `pathlib._RecursiveWildcardSelector` catches only `PermissionError`, so
    the `FileNotFoundError` comes straight out of the generator and kills the caller.
    That is the gate reporting `lint:source-early: NEW lint findings (exit 1)` with no
    findings in the artifact.
    """
    for i in range(200):
        sub = tmp_path / f"a{i:03d}"
        sub.mkdir()
        (sub / "x.rs").write_text("x")

    stale = tmp_path.rglob("*.rs")  # unpruned-walk: intentional
    first = next(stale)
    for i in range(200):
        sub = tmp_path / f"a{i:03d}"
        if sub != first.parent and sub.exists():
            shutil.rmtree(sub)

    with pytest.raises(OSError):
        list(stale)

    # And the replacement is immune to the race that actually happens, which is the
    # point of pruning rather than filtering: the churn is cargo's, inside target/, and
    # a walk that never enters target/ cannot observe it. Note this walker does NOT
    # swallow a vanishing SOURCE directory — `onerror=_raise` is deliberate ("missing
    # source is an incomplete check, not an empty corpus"), and that is the right
    # trade only because the mutable tree is excluded before scandir. Both halves of
    # that bargain are asserted here so neither can be relaxed alone.
    churn = tmp_path / "target" / "debug" / "build"
    churn.mkdir(parents=True)
    for i in range(200):
        sub = churn / f"c{i:03d}"
        sub.mkdir()
        (sub / "out.rs").write_text("generated")

    fresh = source_files(tmp_path)
    first = next(fresh)
    shutil.rmtree(tmp_path / "target")
    rest = list(fresh)  # must not raise: the walk never entered target/
    assert all("target" not in p.parts for p in [first, *rest])
