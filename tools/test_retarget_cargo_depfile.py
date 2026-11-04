"""Depfile retargeting must not reinterpret or discard emitted prerequisites."""

import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent))

from retarget_cargo_depfile import retarget


def test_preserves_dependency_body_and_phony_rules_verbatim():
    body = " a.rs space\\ name.bin \\\n+ b.rs\na.rs:\nb.rs:\n"
    assert retarget("/cargo/my\\ app:" + body, "/cargo/my app", "module/my app") == (
        "module/my\\ app:" + body
    )


@pytest.mark.parametrize("text", ["", "/wrong: a.rs\n", "/cargo/app2: a.rs\n", "/cargo/app:a.rs\n"])
def test_wrong_or_malformed_rule_is_rejected(text):
    with pytest.raises(ValueError):
        retarget(text, "/cargo/app", "app")


@pytest.mark.parametrize(
    "target", ["line\nbreak", "hash#name", "dollar$name", "colon:name", "back\\slash"]
)
def test_unsupported_target_spelling_fails_loudly(target):
    with pytest.raises(ValueError):
        retarget("/cargo/app: a.rs\n", "/cargo/app", target)


@pytest.mark.parametrize("failure", ["different", "missing", "unpaired"])
def test_native_snapshot_refusal_preserves_existing_depfile(tmp_path, failure):
    import subprocess

    tool = Path(__file__).resolve().parent / "build-infra"
    raw = tmp_path / "raw commands.json"
    snapshot = tmp_path / "stable commands.json"
    depfile = tmp_path / "cargo.d"
    output = tmp_path / "copied.d"
    raw.write_text("exact commands\n")
    snapshot.write_text(raw.read_text())
    escaped_raw = str(raw).replace(" ", "\\ ")
    escaped_snapshot = str(snapshot).replace(" ", "\\ ")
    depfile.write_text(f"/cargo/app: other.rs {escaped_raw}\n{escaped_raw}:\n")
    command = [
        str(tool),
        "retarget-depfile",
        "--source",
        str(depfile),
        "--cargo-output",
        "/cargo/app",
        "--target",
        "copied/app",
        "--output",
        str(output),
        "--dependency-source",
        str(raw),
        "--dependency-snapshot",
        str(snapshot),
    ]
    result = subprocess.run(command, capture_output=True, text=True, timeout=90)
    assert result.returncode == 0, result.stderr
    assert output.read_text() == f"copied/app: other.rs {escaped_snapshot}\n{escaped_raw}:\n"
    assert depfile.read_text() == f"/cargo/app: other.rs {escaped_raw}\n{escaped_raw}:\n"
    output.write_text("retain prior output\n")
    if failure == "different":
        snapshot.write_text("different commands\n")
    elif failure == "missing":
        snapshot.unlink()
    else:
        command = command[:-2]
    result = subprocess.run(command, capture_output=True, text=True, timeout=90)
    assert result.returncode != 0
    assert output.read_text() == "retain prior output\n"
