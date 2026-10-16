from pathlib import Path

import pytest

from . import build, provenance


def test_clean_nested_targets_keeps_other_arch_and_host(tmp_path: Path):
    root = tmp_path / "sdk/lib/rust/target"
    for parent in (
        root,
        root / "beep-cargo-beep_rust-rustlib",
        root / "storage-pilot-v1-amd64-rustlib",
        root / "body-storage-v1-amd64-example-0-rustlib",
    ):
        for stem in ("release", "i686-kernel", "i686-pc-windows-gnu", "x86_64-pc-windows-gnu"):
            (parent / stem).mkdir(parents=True)
    amd64 = provenance.workspace_cargo_targets(tmp_path, "amd64")
    assert len(amd64) == 4
    assert all(path.name == "x86_64-pc-windows-gnu" for path in amd64)
    i386 = provenance.workspace_cargo_targets(tmp_path, "i386")
    assert len(i386) == 8
    assert all(path.name.startswith("i686-") for path in i386)
    assert provenance.workspace_cargo_targets(tmp_path, "arm64") == []


def test_clean_discovery_does_not_follow_foreign_target(tmp_path: Path):
    root = tmp_path / "sdk/lib/rust/target"
    root.mkdir(parents=True)
    foreign = tmp_path / "foreign"
    (foreign / "x86_64-pc-windows-gnu").mkdir(parents=True)
    (root / "foreign-rustlib").symlink_to(foreign, target_is_directory=True)
    with pytest.raises(ValueError, match="symlinked nested"):
        provenance.workspace_cargo_targets(tmp_path, "amd64")


def test_clean_refuses_symlinked_workspace_root(tmp_path: Path):
    root = tmp_path / "sdk/lib/rust/target"
    root.parent.mkdir(parents=True)
    foreign = tmp_path / "foreign"
    foreign.mkdir()
    root.symlink_to(foreign, target_is_directory=True)
    with pytest.raises(ValueError, match="symlinked workspace"):
        provenance.workspace_cargo_targets(tmp_path, "amd64")


def test_repeating_storage_settings_does_not_reconfigure(tmp_path: Path):
    (tmp_path / "CMakeCache.txt").write_text(
        "VIBEOS_RUST_STORAGE_SHARDS:STRING=4\nVIBEOS_RUST_BODY_SHARDS:STRING=0\n"
    )
    assert build._storage_options(tmp_path, 4, 0) == []
    assert build._storage_options(tmp_path, None, None) == []
    assert build._storage_options(tmp_path, 4, 2) == ["-DVIBEOS_RUST_BODY_SHARDS=2"]
    assert build._storage_options(tmp_path, 0, None) == ["-DVIBEOS_RUST_STORAGE_SHARDS=0"]
