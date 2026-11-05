"""Source capture tests; these never build an OS body or activate store hits."""

import os
import stat
from pathlib import Path

import pytest

from tools.artifact_inputs import discover
from tools.artifact_worker import SnapshotError, capture, executor_requirements
from tools.test_artifact_inputs import CONTEXT, project, write
from tools.test_artifact_inputs import fixture as fixture


def writable(root):
    for current, _directories, files in os.walk(root):
        os.chmod(current, 0o755)
        for name in files:
            os.chmod(Path(current) / name, 0o644)


def test_capture_derives_key_from_independent_readonly_copy(fixture, tmp_path):
    expected = project(fixture)
    target = tmp_path / "captured"
    report = capture(fixture, "body", CONTEXT, target, workspace=fixture)
    assert report["source_projection_key"] == expected["source_projection_key"]
    copied = discover(target, "body", CONTEXT, workspace=target)
    assert copied["source_projection_key"] == report["source_projection_key"]
    assert not (target / "dep/src/lib.rs").stat().st_mode & stat.S_IWUSR
    assert (target / "dep/src/lib.rs").stat().st_ino != (fixture / "dep/src/lib.rs").stat().st_ino
    write(fixture, "dep/src/lib.rs", "pub fn value() -> u32 { 2 }\n")
    assert (target / "dep/src/lib.rs").read_text() == "pub fn value() -> u32 { 1 }\n"
    assert project(fixture)["source_projection_key"] != report["source_projection_key"]
    assert not executor_requirements()["build_supported"]
    writable(target)


def test_expected_projection_change_fails_before_snapshot_write(fixture, tmp_path):
    expected = project(fixture)["source_projection_key"]
    write(fixture, "dep/src/lib.rs", "pub fn changed() {}")
    target = tmp_path / "captured"
    with pytest.raises(SnapshotError, match="projection changed"):
        capture(
            fixture, "body", CONTEXT, target, workspace=fixture, expected_projection_key=expected
        )
    assert not target.exists()


def test_capture_copies_inactive_manifests_but_no_inactive_sources(fixture, tmp_path):
    target = tmp_path / "captured"
    capture(fixture, "body", CONTEXT, target, workspace=fixture)
    assert (target / "optional/Cargo.toml").is_file()
    assert not (target / "optional/src/lib.rs").exists()
    assert not (target / "unrelated").exists()
    writable(target)


def test_capture_rejects_live_repository_destination_and_ineligible(fixture, tmp_path):
    with pytest.raises(SnapshotError, match="outside"):
        capture(fixture, "body", CONTEXT, fixture / "snap", workspace=fixture)
    write(fixture, "body/build.rs", "fn main() {}")
    with pytest.raises(SnapshotError, match="ineligible"):
        capture(fixture, "body", CONTEXT, tmp_path / "captured", workspace=fixture)


# Source/directory mutations during native capture are exercised at the actual
# copy boundary in artifact_worker.rs::tests::capture_rejects_membership_changes.
