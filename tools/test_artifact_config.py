"""Bounded ambient Cargo audits reject unmodeled original compiler inputs."""

import json
import re
from pathlib import Path

import pytest

from tools import artifact_config as guard
from tools import artifact_runtime as runtime


@pytest.fixture
def config_repo(tmp_path):
    root = tmp_path / "repo"
    (root / "sdk/lib/rust/body").mkdir(parents=True)
    return root


def write_config(directory, text, name="config.toml"):
    directory.mkdir(parents=True, exist_ok=True)
    path = directory / name
    path.write_text(text)
    return path


@pytest.mark.parametrize(
    "text",
    [
        "[profile.release]\nopt-level=3\n",
        "[target.x86_64-pc-windows-gnu]\nlinker=1979-05-27T07:32:00Z\n",
        '[build]\nrustflags=["--cfg=changed"]\n',
        '[env]\nOBSERVED="changed"\n',
        '[source.crates-io]\nreplace-with="elsewhere"\n',
        '[target.x86_64-pc-windows-gnu]\nrustflags=["--cfg=changed"]\n',
        '[target.\'cfg(target_arch = "x86_64")\']\nrustflags=["--cfg=changed"]\n',
    ],
)
def test_global_or_relevant_target_config_is_rejected(config_repo, text):
    write_config(config_repo / ".cargo", text)
    with pytest.raises(guard.ConfigError):
        guard.audit_scope(config_repo, environment={})


def test_inert_target_configs_are_permitted_and_only_files_are_watched(config_repo):
    path = write_config(
        config_repo / "sdk/lib/rust/.cargo",
        '[target.x86_64-pc-windows-gnu]\nlinker="unused-for-staticlib"\n'
        '[target.i686-pc-windows-gnu]\nrustflags=["-C","target-feature=-sse,-sse2"]\n'
        '[target.i686-kernel]\nlinker="other-target"\n'
        '[target.aarch64-pc-windows-gnullvm]\nlinker="other-target"\n',
    )
    report = guard.audit_scope(config_repo, environment={})
    assert str(path) in report["watches"]
    assert all(Path(watch).is_file() for watch in report["watches"])
    assert "/" not in report["watches"]
    assert len(report["checked_paths"]) == len(set(report["checked_paths"]))


@pytest.mark.parametrize(
    "key",
    [
        "CARGO_PROFILE_RELEASE_OPT_LEVEL",
        "CARGO_PROFILE_DEV_DEBUG",
        "CARGO_BUILD_INCREMENTAL",
        "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER",
        "CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS",
        "CARGO_TARGET_I686_PC_WINDOWS_GNU_RUSTFLAGS",
        "RUSTC_BOOTSTRAP",
    ],
)
def test_unscrubbed_code_environment_is_rejected(config_repo, key):
    with pytest.raises(guard.ConfigError):
        guard.audit_scope(config_repo, environment={key: "1"})


def test_scrubbed_flags_and_controlled_scheduler_wrapper_are_not_rejected(config_repo):
    broker = config_repo / "broker"
    broker.write_text("known broker")
    environment = {
        "RUSTFLAGS": "outer-only",
        "CARGO_ENCODED_RUSTFLAGS": "outer-only",
        "CARGO_BUILD_TARGET": "outer-only",
        "CARGO_BUILD_RUSTFLAGS": "outer-only",
        "CARGO_BUILD_RUSTC": "outer-only",
        "CARGO_UNSTABLE_BUILD_STD": "outer-only",
        "CARGO_BUILD_JOBS": "4",
        "CARGO_BUILD_RUSTC_WRAPPER": str(broker),
    }
    guard.audit_scope(config_repo, environment=environment, broker=broker)
    environment["CARGO_BUILD_RUSTC_WRAPPER"] = "unknown-wrapper"
    with pytest.raises(guard.ConfigError):
        guard.audit_scope(config_repo, environment=environment, broker=broker)


def test_effective_cargo_home_and_legacy_config_are_checked(config_repo, tmp_path):
    home = tmp_path / "cargo-home"
    path = write_config(home, '[env]\nOBSERVED="changed"\n', name="config")
    with pytest.raises(guard.ConfigError, match=str(path)):
        guard.audit_scope(config_repo, environment={"CARGO_HOME": str(home)})
    with pytest.raises(guard.ConfigError, match="absolute"):
        guard.audit_scope(config_repo, environment={"CARGO_HOME": "relative"})


def test_actual_cwd_requires_audited_scope_and_canonical_same_checkout(
    config_repo, tmp_path, monkeypatch
):
    with pytest.raises(guard.ConfigError, match="outside"):
        guard.audit_request(
            config_repo, cargo_cwd=config_repo / "sdk/lib/rust/body/src", environment={}
        )
    alias = tmp_path / "canonical-alias"
    alias.mkdir()
    monkeypatch.setattr(guard, "CANONICAL_ROOT", alias)
    with pytest.raises(guard.ConfigError, match="does not mount"):
        guard.mapped_cwd(config_repo, alias / "build-amd64", kernel=False)
    alias.rmdir()
    alias.symlink_to(config_repo, target_is_directory=True)
    assert (
        guard.mapped_cwd(config_repo, alias / "build-amd64", kernel=False)
        == config_repo / "build-amd64"
    )


def test_added_forbidden_config_blocks_fresh_no_request_supervisor(config_repo, tmp_path):
    broker = config_repo / ".cache/host-jobs/host-jobs"
    broker.parent.mkdir(parents=True)
    broker.write_text("#!/bin/sh\nexit 99\n")
    broker.chmod(0o755)
    write_config(config_repo / "sdk/lib/rust/body/.cargo", "[profile.release]\nopt-level=3\n")
    context = tmp_path / "context"
    with (
        pytest.raises(guard.ConfigError),
        runtime.supervise(config_repo, tmp_path / "store", tmp_path / "absent-toolchain", context),
    ):
        pytest.fail("fresh Cargo bypassed supervisor ambient configuration audit")
    summary = json.loads((context / "summary.json").read_text())
    assert summary["config_preflight"] == "failed" and summary["status"] == "failed"
    assert "provision_seconds" not in summary["timings"]


def test_supervisor_postcheck_observes_new_parent_environment_without_request(
    tmp_path, monkeypatch
):
    # Exercise the shipping Python -> native callback after the service has started.
    # Remaining config/compiler fault combinations use native RuntimeHost controls.
    from tools.artifact_executor import PIN
    from tools.test_artifact_runtime_real import source_tree

    repo = tmp_path / "lane"
    source_tree(repo)
    context = tmp_path / "context"
    toolchain = Path.home() / ".rustup/toolchains" / PIN
    with (
        pytest.raises(guard.ConfigError),
        runtime.supervise(repo, tmp_path / "store", toolchain, context),
    ):
        monkeypatch.setenv("CARGO_PROFILE_RELEASE_OPT_LEVEL", "1")
    summary = json.loads((context / "summary.json").read_text())
    assert summary["config_preflight"] == "valid"
    assert summary["config_postcheck"] == "failed" and summary["status"] == "failed"
    assert summary["requests"]["requested"] == 0


def test_the_engine_gate_audits_the_flags_the_worker_compiles_with():
    # reactos-link keeps a copy of the recipe's RUSTFLAGS table so a body built with
    # anything else takes the private path instead of failing the build at the store.
    from tools import artifact_executor

    rs = (
        Path(__file__).parent.parent / "sdk/lib/rust/reactos-link/src/artifact_store.rs"
    ).read_text()
    start = rs.index("fn audited_flags")
    body = rs[start : rs.index("\n}\n", start)]
    table = {}
    for arches, kernel, flags in re.findall(
        r'\("([a-z0-9" |]+)", (true|false)\) => "([^"]*)"', body
    ):
        for arch in re.findall(r"[a-z0-9]+", arches):
            table[(arch, kernel == "true")] = flags
    assert table == artifact_executor.FLAGS
    targets = dict(re.findall(r'"([a-z0-9]+)" => Some\("([a-z0-9_-]+)"\)', rs))
    assert targets == artifact_executor.TARGETS


def test_a_requested_target_s_config_rustflags_are_refused_unless_the_request_replaces_them(
    config_repo,
):
    # Cargo uses [target.<triple>].rustflags only when RUSTFLAGS is unset; the isolated
    # worker reads no config, so an applying entry would compile something else.
    write_config(
        config_repo / "sdk/lib/rust/.cargo",
        '[target.i686-pc-windows-gnu]\nrustflags=["-C","target-feature=-sse,-sse2"]\n',
    )
    cwd = config_repo / "sdk/lib/rust/body"
    request = dict(cargo_cwd=cwd, environment={}, target="i686-pc-windows-gnu")
    with pytest.raises(guard.ConfigError, match="i686-pc-windows-gnu"):
        guard.audit_request(config_repo, **request)
    assert guard.audit_request(config_repo, rustflags_set=True, **request)["watches"]
    # Building another target, the entry cannot apply.
    guard.audit_request(config_repo, cargo_cwd=cwd, environment={})
