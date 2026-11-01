"""Native compiler generators preserve headers and refuse incomplete or foreign inputs."""

import json
import subprocess
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[1]
FIXTURES = REPO / "tools/vibeos-codegen/tests/compiler"


def run(*args, env=None):
    return subprocess.run(
        [str(REPO / "tools/codegen"), *map(str, args)],
        capture_output=True,
        text=True,
        check=False,
        env=env,
        timeout=180,
    )


@pytest.mark.parametrize("command", ["msvcrt-consts", "seh-layout"])
def test_live_header_probe_matches_committed_outputs(command):
    result = run(command, "--check")
    assert result.returncode == 0, result.stderr


@pytest.mark.parametrize(
    "command,arch,count", [("msvcrt-consts", "amd64", 421), ("seh-layout", "i386", 134)]
)
def test_probe_recipes_are_complete_ordered_and_read_only(tmp_path, command, arch, count):
    result = run("--root", tmp_path, command, "--recipe-json")
    assert result.returncode == 0, result.stderr
    recipe = json.loads(result.stdout)
    assert recipe["arch"] == arch
    assert len(recipe["expressions"]) == count
    assert len({row[0] for row in recipe["expressions"]}) == count
    assert recipe["defines"][0] == "_MSVCR_VER=0"
    assert not list(tmp_path.iterdir())


def test_missing_or_foreign_generated_headers_leave_output_untouched(tmp_path):
    output = tmp_path / "keep.h"
    output.write_text("keep")
    result = run("--root", tmp_path, "seh-layout", "--output", output)
    assert result.returncode != 0 and "--headers-only" in result.stderr
    assert output.read_text() == "keep"
    other = tmp_path / "outside"
    other.mkdir()
    own = tmp_path / "own"
    own.mkdir()
    (own / "build-i386").symlink_to(other, target_is_directory=True)
    (other / "sdk/include/reactos").mkdir(parents=True)
    result = run("--root", own, "seh-layout", "--output", output)
    assert result.returncode != 0 and "escapes checkout" in result.stderr
    assert output.read_text() == "keep"


def ucrt_fixture(root):
    files = json.loads((FIXTURES / "ucrt-files.json").read_text())
    for name, text in files.items():
        path = root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)
        if name.startswith(".build-tools/"):
            path.chmod(0o755)


def test_nonempty_ucrt_selection_and_both_outputs_match_python(tmp_path):
    ucrt_fixture(tmp_path)
    output = tmp_path / "generated.inc"
    result = run("--root", tmp_path, "ucrt-cases", "--arch=amd64", "--out", output)
    assert result.returncode == 0, result.stderr
    assert output.read_bytes() == (FIXTURES / "ucrt-python.inc").read_bytes()
    assert (tmp_path / "generated-refs.h").read_bytes() == (FIXTURES / "ucrt-python.h").read_bytes()
    assert "QUARANTINED 1" in result.stderr and "strcoll" in result.stderr
    assert (
        run(
            "--root", tmp_path, "ucrt-cases", "--arch", "amd64", "--out", output, "--check"
        ).returncode
        == 0
    )
    reverse = run("--root", tmp_path, "ucrt-cases", "--arch", "amd64", "--reverse", "--stdout")
    assert reverse.returncode == 0, reverse.stderr
    assert reverse.stdout.find("run_strcmp") < reverse.stdout.find("run_isalpha")
    stats = run("--root", tmp_path, "ucrt-cases", "--arch", "amd64", "--stats")
    assert stats.returncode == 0 and "2 gradable exports" in stats.stdout
    before = output.read_bytes()
    (tmp_path / "build-amd64/libucrt_ref.a").unlink()
    assert run("--root", tmp_path, "ucrt-cases", "--arch", "amd64", "--out", output).returncode != 0
    assert output.read_bytes() == before


def test_ucrt_unkeyed_ast_cache_is_regenerated_and_keyed(tmp_path):
    ucrt_fixture(tmp_path)
    cache = tmp_path / "ast.json"
    cache.write_text('{"bogus": "void (void)"}')
    args = ["--root", tmp_path, "ucrt-cases", "--arch", "amd64", "--ast-json", "--cache", cache]
    result = run(*args)
    assert result.returncode == 0, result.stderr
    assert json.loads(result.stdout)["isalpha"] == "int (int)"
    assert "bogus" not in json.loads(result.stdout)
    stored = json.loads(cache.read_text())
    assert "identity" in stored and "functions" in stored
    assert run(*args).stdout == result.stdout
    assert run(*args, "--check").returncode != 0


def test_ucrt_foreign_build_archives_are_refused(tmp_path):
    ucrt_fixture(tmp_path)
    archive = tmp_path / "build-amd64/libucrt_ref.a"
    elsewhere = tmp_path.parent / (tmp_path.name + "-foreign.a")
    archive.replace(elsewhere)
    archive.symlink_to(elsewhere)
    result = run("--root", tmp_path, "ucrt-cases", "--arch", "amd64", "--stdout")
    assert result.returncode != 0 and "escapes checkout" in result.stderr
