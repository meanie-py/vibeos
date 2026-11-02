"""Module generators retain Python bytes and reject partial/empty input failures."""

import hashlib
import json
import subprocess
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[1]
FIXTURES = REPO / "tools/vibeos-codegen/tests/modules"


def run(*args, cwd=None):
    return subprocess.run(
        [str(REPO / "tools/codegen"), *map(str, args)],
        cwd=cwd,
        capture_output=True,
        text=True,
        check=False,
        timeout=120,
    )


def populate(root, fixture):
    for name, text in json.loads((FIXTURES / fixture).read_text()).items():
        path = root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)


def test_cnt_shipping_outputs_match_and_fixture_covers_future_symbols(tmp_path):
    result = run("cnt", "--check")
    assert result.returncode == 0, result.stderr
    manifest = tmp_path / "sdk/lib/rust/cnt-rust/cnt-manifest.toml"
    manifest.parent.mkdir(parents=True)
    manifest.write_bytes((FIXTURES / "cnt-manifest.toml").read_bytes())
    output = tmp_path / "out"
    result = run("--root", tmp_path, "cnt", "--output-dir", output)
    assert result.returncode == 0, result.stderr
    for name, expected in json.loads((FIXTURES / "cnt-hashes.json").read_text()).items():
        assert hashlib.sha256((output / name).read_bytes()).hexdigest() == expected
    generated = (output / "generated.rs").read_text()
    assert "mod mfuture_" in generated
    assert "super::super::" in generated
    assert "raw_move" in (output / "mod.rs").read_text()
    assert run("--root", tmp_path, "cnt", "--output-dir", output, "--check").returncode == 0
    before = {p.name: p.read_bytes() for p in output.iterdir()}
    manifest.write_text('[symbols.bad]\nc_signature="float bad(float x)"\n')
    assert run("--root", tmp_path, "cnt", "--output-dir", output).returncode != 0
    assert before == {p.name: p.read_bytes() for p in output.iterdir()}


def test_registry_matches_legacy_and_report_keeps_stdout_clean(tmp_path):
    populate(tmp_path, "cohab-files.json")
    nested = tmp_path / ".claude/worktrees/other/CMakeLists.txt"
    nested.parent.mkdir(parents=True)
    nested.write_text("# ghost.c (foo-rust src/one.rs)\n")
    args = ["--root", tmp_path, "cohab-registry"]
    result = run(*args)
    assert result.returncode == 0, result.stderr
    assert result.stdout == (FIXTURES / "cohab.json").read_text()
    entries = json.loads(result.stdout)
    assert entries["src/mod/a.c"]["also_compiled_uncommented"]
    assert entries["src/mod/b.c"]["confidence"] == "LOW"
    assert "src/mod/prose.c" not in entries and "src/mod/x.c" not in entries
    report = run(*args, "--report")
    assert report.returncode == 0 and report.stdout == ""
    assert "disabled-with-Rust-signal TUs" in report.stderr
    output = tmp_path / "registry.json"
    assert run(*args, "--output", output).returncode == 0
    assert run(*args, "--output", output, "--check").returncode == 0
    output.write_text("stale")
    assert run(*args, "--output", output, "--check").returncode == 1


def test_registry_basename_fallback_excludes_generated_copies(tmp_path):
    populate(tmp_path, "cohab-files.json")
    with (tmp_path / "src/mod/CMakeLists.txt").open("a") as f:
        f.write("# c.c (foo-rust src/deeper/one.rs)\n")
    result = run("--root", tmp_path, "cohab-registry")
    assert result.returncode == 0, result.stderr
    entry = json.loads(result.stdout)["src/mod/c.c"]
    assert entry["confidence"] == "HIGH"
    assert entry["rust_files"] == ["sdk/lib/rust/foo-rust/src/one.rs"]


def test_barrier_preserves_legacy_architecture_blocks_and_dedup(tmp_path):
    populate(tmp_path, "barrier-files.json")
    # Build output and generated barriers must never expand coverage or duplicate names.
    (tmp_path / "target").mkdir()
    (tmp_path / "target/generated.rs").write_text("pub static mut Wrong: u32 = 0;")
    (tmp_path / "global_barrier.rs").write_text("pub static mut SelfInput: u32 = 0;")
    args = ["global-barrier", "--source-dir", tmp_path]
    result = run(*args)
    assert result.returncode == 0, result.stderr
    assert result.stdout == (FIXTURES / "global-barrier.rs.txt").read_text()
    output = tmp_path / "global_barrier.rs"
    assert run(*args, "--output", output).returncode == 0
    before = output.stat().st_mtime_ns
    assert run(*args, "--output", output).returncode == 0
    assert output.stat().st_mtime_ns == before
    assert run(*args, "--check").returncode == 0
    output.write_text("stale")
    assert run(*args, "--check").returncode == 1


@pytest.mark.parametrize("name", ["cnt", "cohab-registry", "global-barrier"])
def test_missing_root_fails_without_writing(tmp_path, name):
    result = run("--root", tmp_path / "missing", name, "--check")
    assert result.returncode != 0
    assert not (tmp_path / "missing").exists()


def test_legacy_stdout_entry_points_delegate_to_native(tmp_path):
    populate(tmp_path / "sdk/lib/rust/ntoskrnl-rust/src", "barrier-files.json")
    native = run("--root", tmp_path, "global-barrier")
    import sys

    legacy = subprocess.run(
        [sys.executable, str(REPO / "tools/gen_global_barrier.py")],
        cwd=tmp_path,
        capture_output=True,
        text=True,
        check=False,
    )
    assert legacy.returncode == 0, legacy.stderr
    assert legacy.stdout == native.stdout


def test_registry_source_root_paths_normalize_aliases(tmp_path):
    populate(tmp_path, "cohab-files.json")
    cmake = tmp_path / "src/mod/CMakeLists.txt"
    with cmake.open("a") as f:
        f.write("# ${REACTOS_SOURCE_DIR}/src//mod/./a.c (foo-rust src/one.rs)\n")
    result = run("--root", tmp_path, "cohab-registry")
    assert result.returncode == 0, result.stderr
    entries = json.loads(result.stdout)
    assert "src//mod/./a.c" not in entries
    assert len(entries["src/mod/a.c"]["disabled_at"]) == 3
