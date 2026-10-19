"""Resource generation: frozen Python command oracles and real failing subprocesses."""

import hashlib
import json
import shlex
import subprocess
import sys
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[1]
ORACLES = json.loads((REPO / "tools/vibeos-codegen/tests/resource_commands.json").read_text())


def run(root, *args):
    return subprocess.run(
        [str(REPO / "tools/codegen"), "--root", str(root), *map(str, args)],
        capture_output=True,
        timeout=120,
    )


@pytest.mark.parametrize("oracle", ORACLES.values(), ids=ORACLES.keys())
def test_commands_match_frozen_python_recipes(oracle):
    args = [oracle["family"] + "-res", "--arch", oracle["arch"], "--jobs-json"]
    if oracle["family"] == "ntdll":
        args += ["--module", oracle["module"]]
    result = run(REPO, *args)
    assert result.returncode == 0, result.stderr
    argv = json.loads(result.stdout)["jobs"][0][2]
    normalized = [arg.replace(str(REPO), "{ROOT}") for arg in argv]
    digest = hashlib.sha256(json.dumps(normalized, separators=(",", ":")).encode()).hexdigest()
    assert digest == oracle["sha256"], argv


def put(root, relative, text):
    target = root / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(text)
    return target


@pytest.fixture
def project(tmp_path):
    root = tmp_path / "checkout with spaces"
    root.mkdir()
    compiler = (
        "#!/usr/bin/env python3\nimport sys\nfrom pathlib import Path\n"
        "args = sys.argv[1:]\n"
        "for arg in args:\n"
        " if arg.endswith('/mc-stage'):\n"
        "  assert (Path(arg) / 'bugcodes.bin').read_bytes() == b'message bytes'\n"
        "  assert (Path(arg) / 'bugcodes.rc').is_file()\n"
        "  assert not (Path(arg) / 'unrelated.bin').exists()\n"
        "Path(args[-1]).write_bytes(b'\\x00RESOURCE\\x00')\n"
    )
    for triple in ["i686", "x86_64", "aarch64"]:
        for tool in ["windres", "clang"]:
            path = put(root, f".build-tools/llvm-mingw/bin/{triple}-w64-mingw32-{tool}", compiler)
            path.chmod(0o755)
    for source in [
        "hal/halx86/pic/halpic.rc",
        "ntoskrnl/ntoskrnl.rc",
        "win32ss/win32k.rc",
        "dll/ntdll/def/ntdll.rc",
        "dll/ntdll/nt_0600/ntdll_vista.rc",
    ]:
        put(root, source, "resource source")
    put(root, "build-i386/sdk/include/reactos/mc/bugcodes.rc", "message resource")
    put(root, "build-i386/sdk/include/reactos/mc/bugcodes.bin", "message bytes")
    put(root, "build-i386/sdk/include/reactos/mc/unrelated.bin", "do not stage")
    return root


def test_binary_write_check_stale_and_stdout(project):
    target = project / "output.res"
    result = run(project, "hal-res", "--arch", "i386", "--output", target)
    assert result.returncode == 0, result.stderr
    assert target.read_bytes() == b"\0RESOURCE\0"
    stamp = target.stat().st_mtime_ns
    assert run(project, "hal-res", "--arch", "i386", "--output", target).returncode == 0
    assert target.stat().st_mtime_ns == stamp
    assert run(project, "hal-res", "--arch", "i386", "--output", target, "--check").returncode == 0
    target.write_bytes(b"stale")
    assert run(project, "hal-res", "--arch", "i386", "--output", target, "--check").returncode == 1
    assert target.read_bytes() == b"stale"
    result = run(project, "hal-res", "--arch", "i386", "--stdout")
    assert result.returncode == 0 and result.stdout == b"\0RESOURCE\0", result.stderr


@pytest.mark.parametrize(
    "body", ["raise SystemExit(7)", "pass", "Path(sys.argv[-1]).write_bytes(b'')"]
)
def test_failed_or_empty_generation_never_replaces_output(project, body):
    compiler = project / ".build-tools/llvm-mingw/bin/i686-w64-mingw32-windres"
    compiler.write_text(
        "#!/usr/bin/env python3\nimport sys\nfrom pathlib import Path\n" + body + "\n"
    )
    target = put(project, "precious.res", "precious")
    result = run(project, "hal-res", "--arch", "i386", "--output", target)
    assert result.returncode == 2, result.stderr
    assert target.read_text() == "precious"


def test_kernel_message_sidecars_are_staged(project):
    result = run(project, "ntoskrnl-res", "--arch", "i386", "--stdout")
    assert result.returncode == 0, result.stderr
    assert result.stdout == b"\0RESOURCE\0"
    (project / "build-i386/sdk/include/reactos/mc/bugcodes.rc").unlink()
    result = run(project, "ntoskrnl-res", "--arch", "i386", "--stdout")
    assert result.returncode == 2 and b"bugcodes.rc" in result.stderr


def test_later_arch_failure_leaves_all_destinations_untouched(project):
    first = put(project, "sdk/lib/rust/ntdll-cargo/data/ntdll-i386.rc.res", "precious")
    compiler = project / ".build-tools/llvm-mingw/bin/x86_64-w64-mingw32-windres"
    compiler.write_text("#!/usr/bin/env python3\nraise SystemExit(8)\n")
    result = run(project, "ntdll-res", "--arch", "i386,amd64", "--module", "ntdll")
    assert result.returncode == 2, result.stderr
    assert first.read_text() == "precious"
    assert not (first.parent / "ntdll-amd64.rc.res").exists()


def database(root, *, arguments=True, canonical=True):
    rows = []
    for stem, source in [
        ("basesrv", "subsystems/win/basesrv/basesrv.rc"),
        ("services", "base/system/services/services.rc"),
        ("userinit", "base/system/userinit/userinit.rc"),
        ("winlogon", "base/system/winlogon/winlogon.rc"),
    ]:
        put(root, source, "resource source")
        prefix = "/tmp/vibeos" if canonical else str(root)
        command = [
            f"{prefix}/.build-tools/llvm-mingw/bin/i686-w64-mingw32-windres",
            f"{stem}_res.dir/unused",
            f"{prefix}/{source}",
            "old-output.res",
        ]
        row: dict[str, object] = {"file": source, "directory": prefix}
        row["arguments" if arguments else "command"] = command if arguments else shlex.join(command)
        rows.append(row)
    put(root, "build-i386/compile_commands.json", json.dumps(rows))
    return rows


@pytest.mark.parametrize("arguments,canonical", [(True, True), (False, True), (False, False)])
def test_committed_replay_rebases_to_local_checkout_and_preserves_argv(
    project, arguments, canonical
):
    database(project, arguments=arguments, canonical=canonical)
    result = run(project, "committed-res", "--arch", "i386")
    assert result.returncode == 0, result.stderr
    for stem in ["basesrv", "services", "userinit", "winlogon"]:
        target = project / f"sdk/lib/rust/{stem}-cargo/data/{stem}-i386.rc.res"
        assert target.read_bytes() == b"\0RESOURCE\0"


def test_incomplete_database_is_diagnostic_json_but_generation_fails(project):
    put(project, "build-i386/compile_commands.json", "[]")
    result = run(project, "committed-res", "--arch", "i386", "--jobs-json")
    assert result.returncode == 0, result.stderr
    data = json.loads(result.stdout)
    assert data["jobs"] == [] and data["messages"].count("NO compile_commands entry") == 4
    assert run(project, "committed-res", "--arch", "i386").returncode == 2


def test_absent_database_reports_null_and_generation_fails(project):
    result = run(project, "committed-res", "--arch", "i386", "--jobs-json")
    assert result.returncode == 0 and json.loads(result.stdout)["jobs"] is None
    assert run(project, "committed-res", "--arch", "i386").returncode == 2


def test_foreign_working_directory_is_rejected(project, tmp_path):
    rows = database(project)
    foreign = tmp_path / "sibling"
    foreign.mkdir()
    for row in rows:
        row["directory"] = str(foreign)
    put(project, "build-i386/compile_commands.json", json.dumps(rows))
    result = run(project, "committed-res", "--arch", "i386")
    assert result.returncode == 2 and b"not in this checkout" in result.stderr


def test_foreign_source_is_rejected(project, tmp_path):
    rows = database(project)
    foreign = put(tmp_path, "other-lane/source.rc", "foreign")
    for row in rows:
        argv = row["arguments"]
        assert isinstance(argv, list)
        argv[-2] = str(foreign)
    put(project, "build-i386/compile_commands.json", json.dumps(rows))
    result = run(project, "committed-res", "--arch", "i386")
    assert result.returncode == 2 and b"source resolves outside checkout" in result.stderr


def test_foreign_generated_include_is_rejected(project, tmp_path):
    build = project / "build-i386"
    foreign = tmp_path / "other-lane-build"
    build.rename(foreign)
    build.symlink_to(foreign, target_is_directory=True)
    result = run(project, "hal-res", "--arch", "i386", "--stdout")
    assert result.returncode == 2 and b"generated include resolves outside" in result.stderr


def test_foreign_message_sidecar_is_rejected(project, tmp_path):
    sidecar = project / "build-i386/sdk/include/reactos/mc/bugcodes.bin"
    sidecar.unlink()
    sidecar.symlink_to(put(tmp_path, "other-lane/bugcodes.bin", "foreign"))
    result = run(project, "ntoskrnl-res", "--arch", "i386", "--stdout")
    assert result.returncode == 2 and b"message sidecar resolves outside" in result.stderr


def test_legacy_committed_recipe_query_uses_callers_checkout(project):
    rows = database(project)
    result = subprocess.run(
        [sys.executable, str(REPO / "tools/gen_committed_res.py"), "--arch", "i386", "--jobs-json"],
        cwd=project,
        capture_output=True,
    )
    assert result.returncode == 0, result.stderr
    jobs = json.loads(result.stdout)["jobs"]
    assert len(jobs) == 4 and jobs[0][2] == rows[0]["arguments"]


@pytest.mark.parametrize(
    "args",
    [
        ["hal-res"],
        ["hal-res", "--arch", "arm64"],
        ["ntdll-res", "--arch", "invalid"],
        ["win32k-res", "--arch", "i386", "--module", "ntdll"],
        ["ntdll-res", "--arch", "i386", "--module", "invalid"],
        ["committed-res", "--arch", "i386", "--module", "ntdll"],
    ],
)
def test_invalid_resource_selection_fails(project, args):
    assert run(project, *args).returncode == 2
