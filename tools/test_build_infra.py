"""Exercise native build commands and their refusal/write boundaries."""

import json
import subprocess
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[1]


def run(*args):
    return subprocess.run(
        [str(REPO / "tools/build-infra"), *map(str, args)],
        capture_output=True,
        text=True,
        timeout=120,
        check=False,
    )


def graph(root):
    from tools.test_cell_manifest import COMMAND

    build = root / "build-amd64"
    build.mkdir()
    command = COMMAND.replace("build-i386", "build-amd64")
    (build / "build.ninja").write_text(
        "\nbuild drivers/beep/beep.sys: CUSTOM_COMMAND\n"
        f"  COMMAND = {command}\n  DESC = Cargo-linking beep\n"
    )


def test_manifest_roundtrip_and_check_preserves_timestamp(tmp_path):
    graph(tmp_path)
    result = run("--root", tmp_path, "cell-manifest", "--write", "--arch", "amd64")
    assert result.returncode == 0, result.stderr
    path = tmp_path / "sdk/lib/rust/cells/amd64.jsonl"
    assert json.loads(path.read_text())["crate"] == "beep-cargo"
    before = path.stat().st_mtime_ns
    result = run("--root", tmp_path, "cell-manifest", "--check", "--arch", "amd64")
    assert result.returncode == 0, result.stderr
    assert path.stat().st_mtime_ns == before
    path.write_text("stale")
    assert run("--root", tmp_path, "cell-manifest", "--check", "--arch", "amd64").returncode == 1
    assert path.read_text() == "stale"


def test_manifest_refusal_plans_all_arches_before_writes(tmp_path):
    graph(tmp_path)
    result = run("--root", tmp_path, "cell-manifest", "--write", "--arch", "amd64,arm64")
    assert result.returncode == 2
    assert "build-arm64" in result.stderr
    assert not (tmp_path / "sdk/lib/rust/cells/amd64.jsonl").exists()


def test_depfile_refusal_preserves_previous_output(tmp_path):
    source, output = tmp_path / "input.d", tmp_path / "output.d"
    source.write_text("wrong: input.rs\n")
    output.write_text("previous")
    result = run(
        "retarget-depfile",
        "--source",
        source,
        "--cargo-output",
        "cargo.exe",
        "--target",
        "app.exe",
        "--output",
        output,
    )
    assert result.returncode == 2
    assert output.read_text() == "previous"


def test_unknown_cleanup_target_deletes_nothing(tmp_path):
    (tmp_path / "build.ninja").write_text("build live: phony\nbuild dead: phony\n")
    (tmp_path / "dead").write_text("keep")
    (tmp_path / ".ninja_log").write_text("1\t2\t3\tdead\thash\n")
    result = run("ninja-prune", tmp_path, "missing", "--delete")
    assert result.returncode == 2
    assert (tmp_path / "dead").read_text() == "keep"


@pytest.mark.parametrize(
    "args",
    [
        ["cell-manifest"],
        ["sdk-recipe", "--run"],
        ["media-manifest", "--assemble"],
        ["cargo-depgraph", "--delete"],
        ["retarget-depfile", "--write"],
        ["cell-manifest", "--write", "--check"],
    ],
)
def test_incomplete_or_conflicting_modes_fail(args):
    result = run(*args)
    assert result.returncode == 2


def test_launcher_caches_provisioning_and_preserves_inherited_fds(tmp_path):
    """The broker deliberately inherits lease fds into build tools."""
    tools = tmp_path / "tools"
    src = tools / "vibeos-build/src"
    src.mkdir(parents=True)
    launcher = tools / "build-infra"
    launcher.write_bytes((REPO / "tools/build-infra").read_bytes())
    launcher.chmod(0o755)
    (tmp_path / "rust-toolchain.toml").write_text('[toolchain]\nchannel="stable"\n')
    for name in ("Cargo.toml", "Cargo.lock"):
        (src.parent / name).write_text("fixture")
    source = src / "main.rs"
    source.write_text("fixture")
    shared = tools / "host-jobs/src/build_policy.rs"
    shared.parent.mkdir(parents=True)
    shared.write_text("capacity fixture")
    cargo = tmp_path / "bin/cargo"
    cargo.parent.mkdir()
    cargo.write_text(
        '#!/bin/sh\nwhile [ "$#" -gt 0 ]; do '
        'if [ "$1" = --target-dir ]; then shift; target=$1; fi; shift; done\n'
        'mkdir -p "$target/release"\n'
        'printf x >> "$target/builds"\n'
        "printf '#!/bin/sh\\ncat <&9\\n' > \"$target/release/vibeos-build\"\n"
        'chmod +x "$target/release/vibeos-build"\n'
    )
    cargo.chmod(0o755)
    token = tmp_path / "lease"
    token.write_text("lease survives")
    import os

    env = {**os.environ, "PATH": f"{cargo.parent}:{os.environ['PATH']}"}

    def invoke():
        result = subprocess.run(
            ["bash", "-c", 'exec 9<"$1"; exec "$2" --help', "fixture", str(token), str(launcher)],
            env=env,
            capture_output=True,
            text=True,
            check=False,
        )
        assert result.returncode == 0, result.stderr
        assert result.stdout == "lease survives"

    invoke()
    invoke()
    builds = tmp_path / ".cache/vibeos-build/target/builds"
    assert builds.read_text() == "x"
    source.write_text("changed")
    invoke()
    assert builds.read_text() == "xx"
    shared.write_text("updated shared capacity contract")
    invoke()
    assert builds.read_text() == "xxx"


def test_native_broker_bootstrap_keeps_compiler_output_out_of_json_protocol(tmp_path, monkeypatch):
    from tools import build_infra_compat as native

    assert run("--help").returncode == 0
    source = tmp_path / "tools/host-jobs/src"
    source.mkdir(parents=True)
    (source / "main.rs").write_text("fixture source")
    (tmp_path / "rust-toolchain.toml").write_text('[toolchain]\nchannel="stable"\n')
    fakebin = tmp_path / "bin"
    fakebin.mkdir()
    compiler = fakebin / "rustc"
    compiler.write_text(
        "#!/bin/sh\necho compiler-progress\n"
        'while [ "$#" -gt 0 ]; do if [ "$1" = -o ]; then shift; out=$1; fi; shift; done\n'
        'printf "#!/bin/sh\\nexit 0\\n" > "$out"\nchmod +x "$out"\n'
    )
    compiler.chmod(0o755)
    import os

    monkeypatch.setenv("PATH", str(fakebin) + os.pathsep + os.environ["PATH"])
    produced = Path(native.call("build_policy_ensure_host_jobs", {"repo": tmp_path}))
    assert produced == tmp_path / ".cache/host-jobs/host-jobs"
    assert produced.read_text() == "#!/bin/sh\nexit 0\n"
