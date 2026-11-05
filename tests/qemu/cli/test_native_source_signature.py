"""Source edits must invalidate transport connections without repeated metadata reads."""

import importlib
import os

import pytest

from tools import build_infra_compat as transport


def legacy(paths, roots):
    paths = list(paths)
    for root in roots:
        paths.extend(path for path in root.rglob("*") if path.is_file())
    return tuple(
        (
            str(path),
            *(
                getattr(path.stat(), field)
                for field in ("st_dev", "st_ino", "st_size", "st_mtime_ns", "st_ctime_ns")
            ),
        )
        for path in sorted(paths)
    )


def test_nested_paths_and_file_symlinks_keep_the_same_identity(tmp_path):
    source = tmp_path / "src"
    (source / "alpha").mkdir(parents=True)
    for name in ("alpha.rs", "alpha/child.rs", ".hidden"):
        (source / name).write_text(name)
    (source / "linked.rs").symlink_to(source / "alpha.rs")
    (source / "directory-alias").symlink_to(source / "alpha", target_is_directory=True)
    (source / "missing").symlink_to(source / "does-not-exist")
    lock = tmp_path / "Cargo.lock"
    lock.write_text("lock")
    assert transport._signature_records([lock], [source]) == legacy([lock], [source])


@pytest.mark.parametrize(
    "module_name", ["build_infra_compat", "qemu_infra_compat", "bench_infra_compat"]
)
def test_source_edits_additions_and_replacement_change_each_bridge_token(
    tmp_path, monkeypatch, module_name
):
    module = importlib.import_module("tools." + module_name)
    for name in ("vibeos-build", "vibeos-test", "vibeos-bench"):
        crate = tmp_path / "tools" / name
        (crate / "src").mkdir(parents=True)
        for filename in ("Cargo.toml", "Cargo.lock", "src/lib.rs"):
            (crate / filename).write_text(filename)
    policy = tmp_path / "tools/host-jobs/src/build_policy.rs"
    policy.parent.mkdir(parents=True)
    policy.write_text("policy")
    for name in (
        "rust-toolchain.toml",
        "tools/build-infra",
        "tools/test-infra",
        "tools/bench-infra",
    ):
        (tmp_path / name).write_text(name)
    monkeypatch.setattr(module, "REPO", tmp_path)
    target = (
        tmp_path
        / "tools"
        / ("vibeos-bench" if module_name == "bench_infra_compat" else "vibeos-build")
        / "src/lib.rs"
    )
    before = module._source_signature()
    previous = target.stat()
    target.write_text("x" * len(target.read_text()))
    os.utime(target, ns=(previous.st_atime_ns, previous.st_mtime_ns))
    edited = module._source_signature()
    assert edited != before
    other = target.with_name("other.rs")
    other.write_text("new")
    added = module._source_signature()
    assert added != edited
    other.unlink()
    removed = module._source_signature()
    assert removed != added
    replacement = target.with_name("replacement.tmp")
    replacement.write_bytes(target.read_bytes())
    os.utime(replacement, ns=(previous.st_atime_ns, previous.st_mtime_ns))
    replacement.replace(target)
    assert module._source_signature() != removed
