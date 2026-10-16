"""Filesystem/race tests for the physical artifact inventory."""

import errno
import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path

import pytest
import rust_storage_report as inventory

SHARD = "body-storage-v1-amd64-0123456789abcdef-0-rustlib"


def put(root, relative, data=b"archive"):
    path = root / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
    return path


def regular(report):
    return [r for r in report["inventory"] if r["kind"] == "regular"]


def test_hardlinks_count_once_keep_all_roles_and_aliases(tmp_path):
    root = tmp_path / "target"
    source = put(root, f"{SHARD}/triple/release/libbody.a")
    hashed = source.parent / "deps/libbody-aaaaaaaaaaaaaaaa.a"
    hashed.parent.mkdir()
    os.link(source, hashed)
    snapshot = (
        root / "foo-cargo-amd64-cargo-link/triple/release/build/foo/out/body-storage/id/libbody.a"
    )
    snapshot.parent.mkdir(parents=True)
    os.link(source, snapshot)
    report = inventory.measure(root, detailed=True)
    files = regular(report)
    assert len(files) == 1
    assert len(files[0]["paths"]) == 3
    assert files[0]["roles"] == ["body_primary_archive", "consumer_snapshot"]
    assert report["roles"]["ambiguous"]["allocated_bytes"] == source.stat().st_blocks * 512
    assert report["content"]["duplicate_inodes"] == 0
    assert report["content"]["hashed_bytes_read"] == len(b"archive")
    assert report["coverage"]["complete"]
    assert sum(r["allocated_bytes"] for r in report["roles"].values()) == report["allocated_bytes"]


def test_equal_names_are_not_identity_different_names_are(tmp_path):
    root = tmp_path / "target"
    put(root, f"{SHARD}/triple/release/deps/libx-aaaaaaaaaaaaaaaa.rlib", b"aaa")
    put(
        root,
        "body-storage-v1-amd64-0123456789abcdef-1-rustlib/triple/release/deps/"
        "libx-aaaaaaaaaaaaaaaa.rlib",
        b"bbb",
    )
    put(root, "foo-cargo-amd64-cargo-link/release/deps/libdifferent.rmeta", b"aaa")
    report = inventory.measure(root, detailed=True)
    assert report["content"]["verified_inodes"] == 3
    assert report["content"]["distinct_contents"] == 2
    assert report["content"]["duplicate_inodes"] == 1
    (group,) = report["content_groups"]
    assert group["sha256"] == hashlib.sha256(b"aaa").hexdigest()
    assert group["roles"] == ["body_dependencies", "outer_host_dependencies"]
    assert group["cross_role"]
    assert report["configurations"][0]["observed_shards"] == [0, 1]


def test_snapshot_bytes_and_old_private_layout_separate(tmp_path):
    root = tmp_path / "target"
    put(root, f"{SHARD}/triple/release/libfoo.a", b"archive")
    snapshot = put(
        root,
        "foo-cargo-amd64-cargo-link/triple/release/build/foo/out/body-storage/namespace/libfoo.a",
        b"archive",
    )
    old = put(root, "foo-cargo-foo_rust-rustlib/triple/release/libfoo.a", b"old")
    put(root, "foo-cargo-amd64-cargo-link/release/deps/libhost.rlib", b"host")
    report = inventory.measure(root, detailed=True)
    assert (
        report["roles"]["consumer_snapshot"]["allocated_bytes"] == snapshot.stat().st_blocks * 512
    )
    assert report["roles"]["legacy_private_body"]["allocated_bytes"] >= old.stat().st_blocks * 512
    assert report["roles"]["outer_host_dependencies"]["regular_inodes"] == 1
    assert (
        report["content_groups"][0]["snapshot_allocated_bytes"] == snapshot.stat().st_blocks * 512
    )
    assert report["layouts"]["legacy_private_body"]["regular_inodes"] == 1


def test_sparse_duplicate_bound_uses_allocated_blocks_not_sizes(tmp_path):
    root = tmp_path / "target"
    dense = put(root, "dense.a", bytes(1024 * 1024))
    sparse = root / "sparse.rlib"
    with sparse.open("wb") as stream:
        stream.truncate(dense.stat().st_size)
    report = inventory.measure(root, detailed=True)
    expected = min(dense.stat().st_blocks, sparse.stat().st_blocks) * 512
    assert report["content"]["duplicate_allocated_bytes"] == expected
    assert report["content"]["duplicate_inodes"] == 1
    assert (
        report["content"]["unique_content_allocated_bytes"]
        == max(dense.stat().st_blocks, sparse.stat().st_blocks) * 512
    )


def test_multiple_roots_globally_dedup_inodes_and_distinguish_direct_cmake(tmp_path):
    root = tmp_path / "target"
    source = put(root, f"{SHARD}/triple/release/deps/libshared.rlib")
    build = tmp_path / "build"
    destination = build / "rust-shared-2-target/triple/release/deps/libshared.rlib"
    destination.parent.mkdir(parents=True)
    os.link(source, destination)
    direct = put(build, "sdk/rust-private-target/triple/release/libdirect.a", b"direct")
    put(build, "CMakeFiles/foo.o", b"object")
    report = inventory.measure(root, build_roots=[build], detailed=True)
    files = regular(report)
    shared = next(r for r in files if len(r["paths"]) == 2)
    assert shared["roles"] == ["body_dependencies", "direct_cmake_dependencies"]
    assert (
        report["roles"]["direct_cmake_primary_archive"]["allocated_bytes"]
        == direct.stat().st_blocks * 512
    )
    assert report["roles"]["build_other"]["regular_inodes"] == 1
    assert report["content"]["verified_inodes"] == 2
    assert report["content"]["duplicate_inodes"] == 0


def test_overlapping_roots_do_not_double_count_same_paths(tmp_path):
    root = tmp_path / "target"
    source = put(root, "foo.a", b"data")
    report = inventory.measure(root, build_roots=[root], detailed=True)
    (file,) = regular(report)
    assert len(file["paths"]) == 2  # two scope/role annotations on one path
    assert report["path_count"] == 2  # root directory and regular file
    assert report["content"]["hashed_bytes_read"] == source.stat().st_size
    assert report["coverage"]["regular_inodes_with_unobserved_hardlinks"] == 0


def test_no_symlinks_followed_including_root_ancestors(tmp_path):
    outside = tmp_path / "outside"
    put(outside, "secret.a", b"outside")
    root = tmp_path / "target"
    root.mkdir()
    (root / "file.a").symlink_to(outside / "secret.a")
    (root / "directory").symlink_to(outside, target_is_directory=True)
    report = inventory.measure(root, detailed=True)
    assert not regular(report)
    assert report["coverage"]["symlink_inodes_skipped"] == 2
    assert report["content"]["verified_inodes"] == 0
    alias = tmp_path / "aliased"
    alias.symlink_to(tmp_path, target_is_directory=True)
    bad = inventory.measure(alias / "target")
    assert not bad["coverage"]["complete"]
    assert bad["coverage"]["errors"][0]["stage"] == "root"


def test_directory_swap_to_symlink_cannot_escape_scope(tmp_path, monkeypatch):
    root = tmp_path / "target"
    put(root, "child/inside.a", b"inside")
    outside = tmp_path / "outside"
    put(outside, "secret.a", b"outside")
    original = os.open
    swapped = False

    def intercepted(path, flags, mode=0o777, *, dir_fd=None):
        nonlocal swapped
        if path == "child" and not swapped:
            swapped = True
            (root / "child").rename(root / "moved")
            (root / "child").symlink_to(outside, target_is_directory=True)
        return original(path, flags, mode, dir_fd=dir_fd)

    monkeypatch.setattr(os, "open", intercepted)
    report = inventory.measure(root, detailed=True)
    assert not report["coverage"]["complete"]
    assert not any(p["path"].endswith("secret.a") for r in report["inventory"] for p in r["paths"])


def test_same_size_same_mtime_rewrite_during_hash_is_rejected(tmp_path, monkeypatch):
    root = tmp_path / "target"
    source = put(root, "body.a", b"before")
    original = os.read
    changed = False

    def interrupted(fd, length):
        nonlocal changed
        result = original(fd, length)
        if result and not changed:
            changed = True
            old = source.stat()
            source.write_bytes(b"after!")
            os.utime(source, ns=(old.st_atime_ns, old.st_mtime_ns))
        return result

    monkeypatch.setattr(os, "read", interrupted)
    report = inventory.measure(root, detailed=True)
    assert report["content"]["verified_inodes"] == 0
    assert report["coverage"]["candidate_status_counts"] == {"raced": 1}
    assert regular(report)[0]["sha256"] is None


def test_path_replacement_after_hash_invalidates_content(tmp_path, monkeypatch):
    root = tmp_path / "target"
    source = put(root, "body.a", b"before")
    original = inventory._hash_file

    def changed(parent_fd, name, expected):
        result = original(parent_fd, name, expected)
        replacement = root / "new"
        replacement.write_bytes(b"other!")
        replacement.replace(source)
        return result

    monkeypatch.setattr(inventory, "_hash_file", changed)
    report = inventory.measure(root, detailed=True)
    assert report["content"]["verified_inodes"] == 0
    assert not report["coverage"]["complete"]
    assert any(e["stage"] == "revalidate" for e in report["coverage"]["errors"])


def test_root_replacement_invalidates_content(tmp_path, monkeypatch):
    root = tmp_path / "target"
    put(root, "body.a", b"original")
    original = inventory._hash_file

    def changed(parent_fd, name, expected):
        result = original(parent_fd, name, expected)
        root.rename(tmp_path / "moved")
        root.mkdir()
        return result

    monkeypatch.setattr(inventory, "_hash_file", changed)
    report = inventory.measure(root, detailed=True)
    assert report["content"]["verified_inodes"] == 0
    assert not report["coverage"]["complete"]
    assert any(e["stage"] == "root_revalidate" for e in report["coverage"]["errors"])


def test_nested_generated_archive_is_not_a_body_primary(tmp_path):
    path = f"{SHARD}/triple/release/build/helper/out/libnative.a"
    assert inventory.classify(Path(path))["role"] == "body_other"


def test_unreadable_artifact_is_not_zero_or_verified(tmp_path, monkeypatch):
    root = tmp_path / "target"
    source = put(root, "body.a")

    def unavailable(*args):
        raise PermissionError(errno.EACCES, "controlled unreadable fixture")

    monkeypatch.setattr(inventory, "_hash_file", unavailable)
    report = inventory.measure(root, detailed=True)
    assert not report["coverage"]["complete"]
    assert report["coverage"]["candidate_status_counts"] == {"unreadable": 1}
    assert report["allocated_bytes"] >= source.stat().st_blocks * 512
    assert report["content"]["duplicate_allocated_bytes"] == 0


def test_disabled_hashing_never_claims_content_equality(tmp_path):
    put(tmp_path, "a.a", b"equal")
    put(tmp_path, "b.a", b"equal")
    report = inventory.measure(tmp_path, hash_content=False)
    assert report["duplicates"] == {}
    assert not report["content_hashed"]
    assert report["content"]["verified_inodes"] == 0
    assert report["coverage"]["candidate_status_counts"] == {"disabled": 2}
    assert report["coverage"]["complete"]
    assert not report["coverage"]["hash_complete"]


def test_missing_root_fails_coverage(tmp_path):
    report = inventory.measure(tmp_path / "missing")
    assert not report["coverage"]["complete"]
    assert report["coverage"]["errors"][0]["stage"] == "root"


def test_external_hardlinks_are_visible(tmp_path):
    root = tmp_path / "target"
    source = put(root, "a.a")
    os.link(source, tmp_path / "external.a")
    report = inventory.measure(root)
    assert report["coverage"]["regular_inodes_with_unobserved_hardlinks"] == 1


def test_cli_detailed_output_is_opt_in_file_and_summary_is_bounded(tmp_path):
    root = tmp_path / "target"
    put(root, "a.a")
    output = tmp_path / "report.json"
    result = subprocess.run(
        [
            sys.executable,
            inventory.__file__,
            "--root",
            str(root),
            "--json",
            str(output),
            "--detailed",
        ],
        capture_output=True,
        text=True,
    )
    assert result.returncode == 0, result.stderr
    assert "inventory" not in json.loads(result.stdout)
    assert json.loads(output.read_text())["inventory"]
    assert len(result.stdout) < 15000
    bad = subprocess.run(
        [sys.executable, inventory.__file__, "--root", str(root), "--json", str(root / "bad.json")],
        capture_output=True,
        text=True,
    )
    assert bad.returncode != 0
    assert not (root / "bad.json").exists()
    missing = subprocess.run(
        [sys.executable, inventory.__file__, "--root", str(root / "missing")],
        capture_output=True,
        text=True,
    )
    assert missing.returncode == 2


@pytest.mark.parametrize(
    ("path", "role"),
    [
        (f"{SHARD}/release/deps/libhost.rlib", "body_dependencies"),
        (f"{SHARD}/triple/release/libmain.a", "body_primary_archive"),
        ("foo-cargo-arm64-cargo-link/release/deps/libhost.so", "outer_host_dependencies"),
        ("foo-cargo-arm64-cargo-link/triple/release/foo.exe", "outer_target_other"),
        ("foo-cargo-foo_rust-rustlib/triple/release/deps/libdep.rlib", "legacy_private_body"),
    ],
)
def test_role_classification(path, role):
    assert inventory.classify(Path(path))["role"] == role
