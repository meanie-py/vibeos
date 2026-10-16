"""Semantic worker projection and real namespace negative controls, no compiles."""

import json
import os
import re
import shutil
import subprocess
from pathlib import Path

import pytest

from tools import artifact_executor as executor
from tools.artifact_executor import WORKSPACE, file_digest, launch, normalize, run


def write(root, relative, text):
    path = root / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)


def test_projection_preserves_dependency_features_without_dev_or_other_members(tmp_path):
    snapshot = tmp_path / "snapshot"
    for name in ("body", "dep", "inactive"):
        manifest = f'[package]\nname="{name}"\nversion="0.1.0"\nedition="2021"\n'
        if name == "body":
            manifest += (
                '[dependencies]\ndep={path="../dep",default-features=false,features=["base"]}\n'
                'inactive={path="../inactive",optional=true}\n[dev-dependencies]\n'
                'missing-dev={path="../missing-dev"}\n[features]\ndefault=["enabled"]\n'
                'enabled=["dep/extra"]\n'
            )
        else:
            manifest += '[features]\nbase=[]\nextra=[]\ndefault=["extra"]\n'
        write(snapshot, WORKSPACE / name / "Cargo.toml", manifest)
        if name != "inactive":
            write(snapshot, WORKSPACE / name / "src/lib.rs", "pub fn example() {}\n")
    body = (WORKSPACE / "body").as_posix()
    dep = (WORKSPACE / "dep").as_posix()
    projection = {
        "descriptor": {
            "request": {"package": body, "profile": "release"},
            "profiles": {
                "opt-level": 1,
                "panic": "abort",
                "package": {"dep": {"codegen-units": 17}},
            },
            "packages": {body: {}, dep: {}},
        }
    }
    destination = tmp_path / "projection"
    result = normalize(snapshot, projection, destination)
    toolchain = Path.home() / ".rustup/toolchains/1.92.0-x86_64-unknown-linux-gnu"
    environment = {
        "RUSTC": str(toolchain / "bin/rustc"),
        "HOME": str(tmp_path),
        "CARGO_HOME": str(tmp_path / "cargo-home"),
        "PATH": os.environ["PATH"],
    }
    command = subprocess.run(
        [
            str(toolchain / "bin/cargo"),
            "metadata",
            "--format-version",
            "1",
            "--offline",
            "--locked",
            "--manifest-path",
            str(destination / WORKSPACE / "Cargo.toml"),
        ],
        cwd=destination,
        env=environment,
        capture_output=True,
        text=True,
        check=False,
    )
    assert command.returncode == 0, command.stderr
    metadata = json.loads(command.stdout)
    by_name = {package["id"]: package["name"] for package in metadata["packages"]}
    features = {by_name[node["id"]]: node["features"] for node in metadata["resolve"]["nodes"]}
    assert features == {"body": ["default", "enabled"], "dep": ["base", "extra"]}
    assert metadata["workspace_members"] == [
        next(package["id"] for package in metadata["packages"] if package["name"] == "body")
    ]
    assert result["active_packages"] == [body, dep]
    assert "dev-dependencies" not in (destination / WORKSPACE / "body/Cargo.toml").read_text()
    assert (destination / WORKSPACE / "inactive/__artifact_inactive.rs").is_file()
    assert "codegen-units" in (destination / WORKSPACE / "Cargo.toml").read_text()


def test_real_namespace_hides_host_and_rejects_source_writes_and_network(tmp_path):
    config = {
        "repo": str(Path(__file__).resolve().parents[0]),
        "env": {"ONLY_EXPLICIT": "1"},
        "argv": [],
    }
    # Bootstrap needs repository Python modules before chroot; they are never
    # mounted into the worker. Derive their repository path from loaded module.
    import tools.artifact_inputs

    config["repo"] = str(Path(tools.artifact_inputs.__file__).resolve().parents[1])
    for name in ("root", "projection", "toolchain", "target", "cargo-home", "home", "scratch"):
        path = tmp_path / name
        path.mkdir()
        config[name] = str(path)
    config["runtime_libraries"] = []
    write(
        Path(config["projection"]), WORKSPACE / "Cargo.toml", "source read-only negative control\n"
    )
    config_path = tmp_path / "worker.json"
    config_path.write_text(json.dumps(config))
    result = subprocess.run(launch(config_path, True), capture_output=True, text=True, check=False)
    assert result.returncode == 0, result.stderr
    evidence = json.loads(result.stdout)
    assert evidence["host_checkout_hidden"] and evidence["host_home_hidden"]
    assert evidence["readonly_source_errno"] == 30
    assert evidence["network_connect_errno"] == 101
    assert evidence["environment_keys"] == ["ONLY_EXPLICIT"]


def test_compile_refuses_missing_broker_admission(tmp_path, monkeypatch):
    monkeypatch.delenv("VIBEOS_HOST_JOB_HELD", raising=False)
    with pytest.raises(RuntimeError, match="host-jobs"):
        run(tmp_path)


def test_cold_compile_refuses_existing_evidence(tmp_path, monkeypatch):
    monkeypatch.setenv("VIBEOS_HOST_JOB_HELD", "1")
    report = {
        "compile_status": "success",
        "context": {
            "execution_policy": {"executor_source": file_digest(Path(run.__code__.co_filename))}
        },
    }
    (tmp_path / "report.json").write_text(json.dumps(report))
    with pytest.raises(RuntimeError, match="refusing to overwrite evidence"):
        run(tmp_path)


def test_executor_change_refuses_prepared_recipe(tmp_path, monkeypatch):
    monkeypatch.setenv("VIBEOS_HOST_JOB_HELD", "1")
    (tmp_path / "target").mkdir()
    report = {
        "compile_status": "not run",
        "context": {"execution_policy": {"executor_source": {"sha256": "wrong"}}},
    }
    (tmp_path / "report.json").write_text(json.dumps(report))
    with pytest.raises(RuntimeError, match="executor changed"):
        run(tmp_path)


def test_executed_child_receives_explicit_environment_only(tmp_path, monkeypatch):
    import tools.artifact_inputs

    config = {
        "repo": str(Path(tools.artifact_inputs.__file__).resolve().parents[1]),
        "env": {"EXPECTED_ONLY": "declared-value"},
        "argv": ["/toolchain/bin/cargo"],
    }
    for name in ("root", "projection", "toolchain", "target", "cargo-home", "home", "scratch"):
        path = tmp_path / name
        path.mkdir()
        config[name] = str(path)
    write(Path(config["projection"]), WORKSPACE / "Cargo.toml", "probe-only\n")
    (Path(config["toolchain"]) / "bin").mkdir()
    shutil.copy2("/usr/bin/env", Path(config["toolchain"]) / "bin/cargo")
    linkage = subprocess.run(["ldd", "/usr/bin/env"], capture_output=True, text=True, check=True)
    config["runtime_libraries"] = sorted(
        {match.group(1) for match in re.finditer(r"(?:=>\s*)?(/\S+)", linkage.stdout)}
    )
    config_path = tmp_path / "worker.json"
    config_path.write_text(json.dumps(config))
    monkeypatch.setenv("UNDECLARED_HOST_SENTINEL", "must-not-reach-executed-child")
    result = subprocess.run(launch(config_path), capture_output=True, text=True, check=False)
    assert result.returncode == 0, result.stderr
    assert result.stdout == "EXPECTED_ONLY=declared-value\n"


@pytest.fixture(scope="module")
def provisioned():
    toolchain = Path.home() / ".rustup/toolchains" / executor.PIN
    return toolchain, executor.provision(toolchain)


@pytest.fixture
def prepared(tmp_path, provisioned):
    toolchain, tool = provisioned
    repo = tmp_path / "source-repo-without-tools"
    workspace = repo / WORKSPACE
    write(
        workspace,
        "Cargo.toml",
        '[workspace]\nresolver="2"\nmembers=["body"]\n[profile.release]\npanic="abort"\n',
    )
    write(workspace, "Cargo.lock", 'version=4\n[[package]]\nname="body"\nversion="0.1.0"\n')
    write(
        workspace,
        "body/Cargo.toml",
        '[package]\nname="body"\nversion="0.1.0"\nedition="2021"\n[lib]\ncrate-type=["staticlib"]\n',
    )
    write(workspace, "body/src/lib.rs", "pub fn value() -> u32 { 1 }\n")
    output = tmp_path / "worker"
    report = executor.prepare(repo, "body", toolchain, output, provisioned=tool)
    return output, report, toolchain, tool


def test_provisioned_prepare_matches_lookup_context_and_does_not_rehash(prepared, monkeypatch):
    output, report, toolchain, tool = prepared
    monkeypatch.setattr(executor, "provision", lambda _: pytest.fail("duplicate full provision"))
    assert executor.request_context(toolchain, tool, "body") == report["context"]
    assert report["toolchain_hash_mode"] == "supervisor-provided"
    assert executor.validate_prepared(output, report)
    original = tool["rustc"]
    try:
        tool["rustc"] = "changed outside receipt"
        assert report["context"]["toolchain"]["rustc"] == original
    finally:
        tool["rustc"] = original


def test_relocated_source_repo_does_not_supply_bootstrap_imports(prepared):
    output, report, _, _ = prepared
    analyzed = Path(json.loads((output / "worker.json").read_text())["repo"])
    assert not (analyzed / "tools").exists()
    assert report["graph_validation"]["status"] == "identical"
    command = executor.launch(output / "worker.json", True)
    assert str(executor.IMPLEMENTATION_REPO) in command
    assert str(analyzed) not in command


@pytest.mark.parametrize(
    "mutation",
    [
        "config",
        "source",
        "same-mtime",
        "new-file",
        "new-dir",
        "manifest",
        "snapshot",
        "symlink",
        "action-key",
        "context",
        "cargo-config",
        "root-scaffold",
    ],
)
def test_prepared_inputs_reject_mutation_before_launch(prepared, monkeypatch, mutation):
    output, report, _, _ = prepared
    source = output / "projection" / WORKSPACE / "body/src/lib.rs"
    if mutation == "config":
        config = json.loads((output / "worker.json").read_text())
        config["env"]["UNDECLARED"] = "changed"
        (output / "worker.json").write_text(json.dumps(config))
    elif mutation in {"source", "same-mtime"}:
        old_stat = source.stat()
        source.write_text("pub fn value() -> u32 { 2 }\n")
        if mutation == "same-mtime":
            os.utime(source, ns=(old_stat.st_atime_ns, old_stat.st_mtime_ns))
    elif mutation == "new-file":
        (source.parent / "untracked.txt").write_text("added")
    elif mutation == "new-dir":
        (source.parent / "new-empty-directory").mkdir()
    elif mutation == "manifest":
        (output / "projection" / WORKSPACE / "Cargo.toml").write_text("changed")
    elif mutation == "snapshot":
        path = output / "snapshot" / WORKSPACE / "body/src/lib.rs"
        path.chmod(0o644)
        path.write_text("changed captured bytes")
    elif mutation == "symlink":
        (source.parent / "escaping-link").symlink_to("/etc/passwd")
    elif mutation == "cargo-config":
        (output / "cargo-home/config.toml").write_text('[build]\nrustflags=["--cfg=unexpected"]\n')
    elif mutation == "root-scaffold":
        (output / "root/etc").mkdir()
        (output / "root/etc/injected-config").write_text("must not be visible")
    elif mutation == "action-key":
        report["action_key"] = "wrong"
        (output / "report.json").write_text(json.dumps(report))
    else:
        report["context"]["effective_config"]["argv"].append("--features=changed")
        (output / "report.json").write_text(json.dumps(report))
    monkeypatch.setenv("VIBEOS_HOST_JOB_HELD", "1")
    monkeypatch.setattr(executor, "launch", lambda *_: pytest.fail("changed inputs reached worker"))
    with pytest.raises(RuntimeError):
        executor.run(output)
    failed = json.loads((output / "report.json").read_text())
    assert failed["compile_status"] == "failed"
    assert failed["failure_stage"] == "prepared-input-validation"


def fake_worker(output, *, exit_code=0, sleep=False):
    archive = output / "target/x86_64-pc-windows-gnu/release/libbody.a"
    script = (
        "from pathlib import Path; import sys,time; "
        "p=Path(sys.argv[1]); p.parent.mkdir(parents=True); "
        "p.write_bytes(b'!<arch>\\nfixture'); "
        + ("time.sleep(30); " if sleep else "")
        + f"sys.exit({exit_code})"
    )
    return [executor.sys.executable, "-c", script, str(archive)]


@pytest.mark.parametrize(
    "failure", ["probe", "compile-launch", "compile", "tool-postcheck", "sampling", "interrupt"]
)
def test_run_failure_is_final_and_retains_stage_evidence(prepared, monkeypatch, failure):
    output, _, _, _ = prepared
    monkeypatch.setenv("VIBEOS_HOST_JOB_HELD", "1")

    def worker_launch(_, probe=False):
        if probe:
            if failure == "probe":
                return [
                    executor.sys.executable,
                    "-c",
                    "import sys; sys.stderr.write('probe failed');sys.exit(17)",
                ]
            return [executor.sys.executable, "-c", "print('{}')"]
        if failure == "compile-launch":
            return [str(output / "does-not-exist")]
        return fake_worker(
            output,
            exit_code=13 if failure == "compile" else 0,
            sleep=failure in {"sampling", "interrupt"},
        )

    monkeypatch.setattr(executor, "launch", worker_launch)
    if failure == "tool-postcheck":

        def broken_provision(_):
            raise RuntimeError("provision postcheck failed")

        monkeypatch.setattr(executor, "provision", broken_provision)
    if failure in {"sampling", "interrupt"}:

        def broken_accounting(_):
            if failure == "interrupt":
                raise KeyboardInterrupt()
            raise RuntimeError("sampling failed")

        monkeypatch.setattr(executor, "allocated", broken_accounting)
    expected = KeyboardInterrupt if failure == "interrupt" else Exception
    with pytest.raises(expected):
        executor.run(output)
    failed = json.loads((output / "report.json").read_text())
    assert failed["compile_status"] == ("interrupted" if failure == "interrupt" else "failed")
    stages = {
        "probe": "isolation-probe",
        "compile-launch": "compile-launch",
        "compile": "compile",
        "tool-postcheck": "toolchain-postcheck",
        "sampling": "compile",
        "interrupt": "compile",
    }
    assert failed["failure_stage"] == stages[failure]
    if failure == "probe":
        assert failed["failure"]["stderr"] == "probe failed"
    if failure in {"sampling", "interrupt"}:
        assert failed["exit_code"] is not None


def test_run_success_records_verified_inputs_and_actual_tool_postcheck(prepared, monkeypatch):
    output, _, _, tool = prepared
    monkeypatch.setenv("VIBEOS_HOST_JOB_HELD", "1")
    monkeypatch.setattr(
        executor,
        "launch",
        lambda _, probe=False: (
            [executor.sys.executable, "-c", "print('{}')"] if probe else fake_worker(output)
        ),
    )
    calls = []

    def verified_toolchain(_):
        calls.append(True)
        return tool

    monkeypatch.setattr(executor, "provision", verified_toolchain)
    result = executor.run(output)
    assert calls == [True]
    assert result["compile_status"] == "success"
    assert result["prepared_input_validation"] == "identical"
    assert result["toolchain_postcheck"] == "identical"
    assert result["archive"]["sha256"]


@pytest.mark.parametrize("changed", ["projection", "config", "toolchain"])
def test_changed_inputs_after_compile_are_rejected(prepared, monkeypatch, changed):
    output, _, _, tool = prepared
    monkeypatch.setenv("VIBEOS_HOST_JOB_HELD", "1")

    def worker_launch(_, probe=False):
        if probe:
            return [executor.sys.executable, "-c", "print('{}')"]
        command = fake_worker(output)
        if changed == "projection":
            source = output / "projection" / WORKSPACE / "body/src/lib.rs"
            command[2] = command[2].replace(
                "sys.exit(0)",
                f"Path({str(source)!r}).write_text('changed during build');sys.exit(0)",
            )
        elif changed == "config":
            config = output / "worker.json"
            command[2] = command[2].replace(
                "sys.exit(0)", f"Path({str(config)!r}).write_text('{{}}');sys.exit(0)"
            )
        return command

    monkeypatch.setattr(executor, "launch", worker_launch)
    if changed == "toolchain":
        altered = json.loads(json.dumps(tool))
        altered["rustc"] = "changed native toolchain identity"
        monkeypatch.setattr(executor, "provision", lambda _: altered)
    with pytest.raises(RuntimeError, match="changed"):
        executor.run(output)
    report = json.loads((output / "report.json").read_text())
    assert report["compile_status"] == "failed"
    assert report["failure_stage"] == (
        "toolchain-postcheck" if changed == "toolchain" else "prepared-input-postcheck"
    )


def test_each_architecture_recipe_compiles_its_own_triple_with_its_audited_flags(provisioned):
    toolchain, tool = provisioned
    amd64 = executor.request_context(toolchain, tool, "body", kernel=True)
    # The amd64 recipe is unchanged by the per-arch table, so its existing actions stay hits.
    assert amd64 == executor.request_context(
        toolchain,
        tool,
        "body",
        kernel=True,
        target=executor.TARGETS["amd64"],
        flags=executor.FLAGS[("amd64", True)],
    )
    i386 = executor.request_context(
        toolchain,
        tool,
        "body",
        kernel=False,
        target=executor.TARGETS["i386"],
        flags=executor.FLAGS[("i386", False)],
    )
    assert i386["target"] == {"builtin": "i686-pc-windows-gnu"}
    assert "i686-pc-windows-gnu" in i386["effective_config"]["argv"]
    assert i386["environment"]["RUSTFLAGS"].startswith("-C target-feature=-sse,-sse2 -C dlltool=")
    assert i386 != amd64
    # Flags from another arch's recipe, or none at all where SSE must be off, are refused.
    for target, flags in (("i686-pc-windows-gnu", ""), ("x86_64-pc-windows-gnu", "-C opt-level=3")):
        with pytest.raises(RuntimeError, match="unaudited RUSTFLAGS"):
            executor.request_context(toolchain, tool, "body", target=target, flags=flags)
    with pytest.raises(RuntimeError, match="builtin recipe triple"):
        executor.request_context(toolchain, tool, "body", target="i686-kernel", flags="")
