from pathlib import Path

import pytest

from . import provenance


def test_clean_nested_targets_keeps_other_arch_and_host(tmp_path: Path):
    root = tmp_path / "sdk/lib/rust/target"
    for parent in (
        root,
        root / "beep-cargo-beep_rust-rustlib",
        root / "storage-pilot-v1-amd64-rustlib",
    ):
        for stem in ("release", "i686-kernel", "i686-pc-windows-gnu", "x86_64-pc-windows-gnu"):
            (parent / stem).mkdir(parents=True)
    amd64 = provenance.workspace_cargo_targets(tmp_path, "amd64")
    assert len(amd64) == 3
    assert all(path.name == "x86_64-pc-windows-gnu" for path in amd64)
    i386 = provenance.workspace_cargo_targets(tmp_path, "i386")
    assert len(i386) == 6
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
