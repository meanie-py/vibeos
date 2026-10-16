"""Bounded ambient Cargo audits reject unmodeled original compiler inputs."""

import json
from pathlib import Path

import pytest

from tools import artifact_config as guard
from tools import test_artifact_runtime as adapter_tests

harness = adapter_tests.harness


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
    real_samefile = guard.os.path.samefile
    monkeypatch.setattr(
        guard.os.path,
        "samefile",
        lambda a, b: True if Path(a) == alias and Path(b) == config_repo else real_samefile(a, b),
    )
    assert (
        guard.mapped_cwd(config_repo, alias / "build-amd64", kernel=False)
        == config_repo / "build-amd64"
    )


@pytest.fixture
def guarded_api(harness):
    (harness.repo / "tools/artifact_config.py").write_text("# guarded helper\n")
    return harness


def test_added_forbidden_config_blocks_fresh_no_request_supervisor(guarded_api):
    api = guarded_api
    with adapter_tests.supervise(api):
        pass
    before = api.executor.provisions
    write_config(api.repo / "sdk/lib/rust/beep-rust/.cargo", "[profile.release]\nopt-level=3\n")
    with pytest.raises(guard.ConfigError), adapter_tests.supervise(api):
        pytest.fail("fresh Cargo bypassed supervisor ambient configuration audit")
    assert api.executor.provisions == before
    summary = json.loads((api.context / "summary.json").read_text())
    assert summary["config_preflight"] == "failed" and summary["status"] == "failed"


def test_supervisor_postcheck_detects_added_config_without_request(guarded_api):
    api = guarded_api
    with pytest.raises(guard.ConfigError), adapter_tests.supervise(api):
        write_config(api.repo / "build-amd64/.cargo", '[env]\nOBSERVED="changed"\n')
    summary = json.loads((api.context / "summary.json").read_text())
    assert summary["config_postcheck"] == "failed"


def test_request_rechecks_config_after_hit_before_binding(guarded_api):
    api = guarded_api
    with pytest.raises(guard.ConfigError), adapter_tests.supervise(api) as supervisor:
        adapter_tests.request(api, supervisor)
        api.store.before_lookup = lambda: write_config(
            api.repo / "build-amd64/.cargo", '[build]\nrustflags=["--cfg=changed"]\n'
        )
        adapter_tests.request(api, supervisor, "cargo:must-not-bind")
    assert (str(api.repo), "cargo:must-not-bind") not in api.store.refs


def test_inert_config_edit_keeps_hit_and_registers_existing_file_watch(guarded_api):
    api = guarded_api
    path = write_config(
        api.repo / "sdk/lib/rust/.cargo", '[target.x86_64-pc-windows-gnu]\nlinker="first"\n'
    )
    with adapter_tests.supervise(api) as supervisor:
        first = adapter_tests.request(api, supervisor, cargo_cwd=api.repo / "sdk/lib/rust")
        path.write_text('[target.x86_64-pc-windows-gnu]\nlinker="second"\n')
        second = adapter_tests.request(
            api, supervisor, "cargo:second", cargo_cwd=api.repo / "sdk/lib/rust"
        )
        assert first["action_key"] == second["action_key"]
        assert second["hit"] and second["builder_calls"] == 0
        assert str(path) in second["watches"]
        assert str(api.repo / "tools/artifact_config.py") in second["watches"]
