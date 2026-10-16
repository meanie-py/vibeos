"""Adapter contract tests: real files, fake compiler/store, no OS/toolchain builds."""

import copy
import json
import shutil
import uuid
from contextlib import contextmanager
from pathlib import Path
from types import SimpleNamespace

import pytest
import tomli

from tools import artifact_runtime as runtime


class FakeStore:
    def __init__(self, root):
        self.root = root
        self.entries = {}
        self.refs = {}
        self.sessions = {}
        self.events = []
        self.locked = False
        self.before_lookup = None
        self.fail_bind = False

    @contextmanager
    def session(self):
        token = uuid.uuid4().hex
        state = SimpleNamespace(token=token, active=True, forced={}, pinned=set())
        self.sessions[token] = state
        self.events.append("owner-enter")
        try:
            yield state
        finally:
            state.active = False
            self.events.append("owner-exit")

    @contextmanager
    def attach_session(self, token):
        state = self.sessions[token]
        if not state.active:
            raise RuntimeError("dead supervisor session")
        self.events.append("child-enter")
        try:
            yield state
        finally:
            self.events.append("child-exit")

    def restore_refs(self, lane, *, session):
        assert session.active
        self.events.append("restore")
        handles = []
        for (owner, _), key in self.refs.items():
            if owner == lane:
                handle = self.entries[key]
                if not handle.blob_path.exists():
                    raise RuntimeError("missing store blob")
                handles.append(handle)
                session.pinned.add(key)
        return handles

    def manifest_path(self, key):
        return self.root / "actions" / (key + ".json")

    def lookup_or_build(
        self,
        descriptor,
        builder,
        *,
        session,
        verify=False,
        verify_once_per_session=False,
        require_verified=False,
    ):
        assert session.active
        self.events.append("action-lock")
        self.locked = True
        try:
            key = runtime._key(descriptor)
            if self.before_lookup:
                callback, self.before_lookup = self.before_lookup, None
                callback()
            previous = self.entries.get(key)
            satisfied = key in session.forced and (session.forced[key] or not require_verified)
            if previous and (not verify or (verify_once_per_session and satisfied)):
                return SimpleNamespace(
                    **(
                        vars(previous)
                        | {
                            "hit": True,
                            "verified_rebuild": False,
                            "session_verified": session.forced.get(key, False),
                        }
                    )
                )
            scratch = self.root / "staging" / uuid.uuid4().hex
            scratch.mkdir(parents=True)
            built = builder(scratch)
            payload = built.outputs["body.a"].path.read_bytes()
            if previous and payload != previous.blob_path.read_bytes():
                raise RuntimeError("independent output conflict")
            digest = __import__("hashlib").sha256(payload).hexdigest()
            blob = self.root / "blobs" / digest
            blob.parent.mkdir(exist_ok=True)
            if not blob.exists():
                blob.write_bytes(payload)
            manifest = {"descriptor": descriptor, "provenance": built.provenance}
            destination = self.manifest_path(key)
            destination.parent.mkdir(exist_ok=True)
            destination.write_text(json.dumps(manifest))
            compared = previous is not None
            if verify:
                session.forced[key] = compared
            session.pinned.add(key)
            handle = SimpleNamespace(
                action_key=key,
                blob_path=blob,
                manifest=manifest,
                hit=False,
                verified_rebuild=compared,
                session_verified=session.forced.get(key, False),
            )
            self.entries[key] = handle
            shutil.rmtree(scratch)
            return handle
        finally:
            self.locked = False

    def bind(self, lane, consumer, key, *, session):
        assert session.active and key in session.pinned
        if self.fail_bind:
            raise RuntimeError("controlled durable bind failure")
        self.events.append("bind")
        self.refs[lane, consumer] = key
        reference = self.root / "refs" / runtime._key([lane, consumer])
        reference.parent.mkdir(exist_ok=True)
        reference.write_text(key)
        return reference

    def list_refs(self, lane):
        return [
            {"lane": owner, "consumer": consumer, "action_key": key}
            for (owner, consumer), key in self.refs.items()
            if owner == lane
        ]

    @contextmanager
    def consumer_guard(self, lane, consumer):
        yield (lane, consumer)

    def release_ref(self, lane, consumer, *, guard):
        assert guard == (lane, consumer)
        del self.refs[lane, consumer]


class FakeExecutor:
    TARGETS = runtime._apis().executor.TARGETS
    FLAGS = runtime._apis().executor.FLAGS

    def __init__(self, api):
        self.api = api
        self.provisions = self.preparations = self.executions = 0
        self.changed = False
        self.mismatch_prepare = False
        self.invalid_report = False
        self.nondeterministic = False
        self.pending = {}

    def provision(self, toolchain):
        self.provisions += 1
        return {"rustc": "fake pinned compiler", "tree": "changed" if self.changed else "fixed"}

    def request_context(
        self,
        toolchain,
        provisioned,
        body,
        *,
        features=(),
        default_features=True,
        kernel=True,
        target=runtime.TARGET,
        flags=None,
    ):
        if flags is None:
            flags = runtime.KERNEL_FLAGS if kernel else ""
        return {
            "toolchain": copy.deepcopy(provisioned),
            "target": {"builtin": target},
            "effective_flags": (flags + " " if flags else "") + runtime._DLLTOOL_FLAG,
            "body": body,
            "features": list(features),
            "default_features": default_features,
            "environment": {"RUSTFLAGS": runtime.KERNEL_FLAGS + " " if kernel else ""},
            "execution_policy": "fake isolated compiler",
        }

    def prepare(self, repo, body, toolchain, output, *, provisioned, **options):
        assert self.api.store.locked, "compile preparation escaped the action lock"
        self.preparations += 1
        output.mkdir()
        context = self.request_context(toolchain, provisioned, body, **options)
        projected = self.api.discover(
            repo,
            body,
            context,
            features=options["features"],
            default_features=options["default_features"],
        )
        report = {"action_key": projected["action_key"], "context": context}
        self.pending[output] = (report, projected["source_projection_key"])
        return report | {"action_key": "bad"} if self.mismatch_prepare else report

    def run(self, output):
        assert self.api.store.locked, "compiler escaped the action lock"
        self.executions += 1
        report, source = self.pending[output]
        archive = output / "body.a"
        archive.write_bytes(
            (source + (str(self.executions) if self.nondeterministic else "")).encode()
        )
        result = report | {
            "archive": {"path": str(archive)},
            "compile_status": "success",
            "lane_input_revalidation": "identical",
            "toolchain_postcheck": "identical",
        }
        if self.invalid_report:
            result["lane_input_revalidation"] = "changed"
        (output / "report.json").write_text(json.dumps(result))
        return result


@pytest.fixture
def harness(tmp_path, monkeypatch):
    repo = tmp_path / "repo"
    package = repo / "sdk/lib/rust/beep-rust"
    (package / "src").mkdir(parents=True)
    (package / "Cargo.toml").write_text(
        '[package]\nname="beep-rust"\nversion="0.1.0"\n[lib]\ncrate-type=["staticlib"]\n'
    )
    (package / "src/lib.rs").write_text("pub fn body() {}\n")
    (repo / "sdk/lib/rust/Cargo.toml").write_text('[workspace]\nresolver="2"\n')
    (repo / "sdk/lib/rust/Cargo.lock").write_text("version=4\n")
    (repo / "rust-toolchain.toml").write_text('[toolchain]\nchannel="1.92.0"\n')
    (repo / "tools").mkdir()
    for module in (
        "artifact_runtime",
        "artifact_inputs",
        "artifact_executor",
        "artifact_worker",
        "artifact_store",
        "artifact_config",
    ):
        (repo / "tools" / (module + ".py")).write_text("# implementation\n")
    broker = repo / ".cache/host-jobs/host-jobs"
    broker.parent.mkdir(parents=True)
    broker.write_text("#!/bin/sh\nexit 99\n")
    broker.chmod(0o755)
    store = FakeStore(tmp_path / "store")
    api = SimpleNamespace(
        repo=repo,
        store=store,
        source=package / "src/lib.rs",
        toolchain=tmp_path / "toolchain",
        context=tmp_path / "contexts",
        ineligible=False,
        discovers=0,
    )

    def discover(root, body, context, **options):
        api.discovers += 1
        source = runtime._key({"source": api.source.read_text()})
        descriptor = {
            "context": context,
            "request": {"package": "sdk/lib/rust/" + body},
            "source": source,
            "packages": {"sdk/lib/rust/" + body: {"inventory": {"directories": ["", "src"]}}},
        }
        reads = {
            p.relative_to(repo).as_posix(): {}
            for p in [
                api.source,
                package / "Cargo.toml",
                repo / "sdk/lib/rust/Cargo.toml",
                repo / "sdk/lib/rust/Cargo.lock",
            ]
        }
        return {
            "eligible": not api.ineligible,
            "action_key": runtime._key(descriptor),
            "source_projection_key": source,
            "descriptor": descriptor,
            "provenance": {"reads": reads},
            "reasons": ["unsupported fixture input"],
        }

    api.discover = discover
    api.inputs = SimpleNamespace(discover=discover, tomllib=tomli)
    api.executor = FakeExecutor(api)
    api.storage = SimpleNamespace(
        Store=lambda root: store,
        Output=lambda path, **kwargs: SimpleNamespace(path=path, **kwargs),
        BuildResult=SimpleNamespace,
    )
    # Fake run still asserts store ownership; a separate test covers broker argv.
    monkeypatch.setattr(
        runtime, "_execute", lambda repo, broker, output, executor: executor.run(output)
    )
    return api


def supervise(api, **options):
    return runtime.supervise(
        api.repo, api.store.root, api.toolchain, api.context, _api=api, **options
    )


def request(api, path, consumer="cargo:first", **options):
    return runtime.request(path, "beep-rust", consumer, _api=api, **options)


def test_miss_hit_shared_owner_lease_direct_archive_and_complete_watches(harness):
    api = harness
    with supervise(api) as supervisor:
        state = runtime.supervisor_config(supervisor)
        assert state["toolchain_key"] == runtime._key(state["provisioned"])
        assert supervisor.stat().st_mode & 0o777 == 0o600
        assert api.executor.provisions == 1
        miss = request(api, supervisor)
        hit = request(api, supervisor, "cargo:second")
        assert miss["builder_calls"] == 1 and hit["builder_calls"] == 0
        assert not miss["hit"] and hit["hit"]
        assert miss["archive"] == hit["archive"]
        assert Path(miss["archive"]).parent.name == "blobs"
        assert not list((api.store.root / "staging").iterdir())
        assert str(api.source) in hit["watches"]
        assert str(api.source.parent) in hit["watches"]
        assert str(api.repo / "sdk/lib/rust/beep-rust") in hit["watches"]
        assert str(api.repo / "sdk/lib/rust/Cargo.lock") in hit["watches"]
        assert str(api.repo / "tools/artifact_store.py") in hit["watches"]
        assert str(supervisor) not in hit["watches"]
        assert str(api.toolchain) not in hit["watches"]
        assert api.store.sessions[state["session_token"]].active
        assert Path(hit["reference"]).is_file()
        assert api.store.events.index("action-lock") < api.store.events.index("bind")
    assert not api.store.sessions[state["session_token"]].active
    assert len(api.store.refs) == 2
    assert api.executor.provisions == 2


@pytest.mark.parametrize(
    "options",
    [
        {"kernel": True},
        {"flags": runtime.KERNEL_FLAGS},
        {"flags": "-C opt-level=0"},
        {"architecture": "arm64"},
        {"target": "custom-kernel.json"},
        {"build_std": True},
        {"emit_ir": True},
        {"force_staticlib": True},
        {"features": "feature"},
        {"features": ("",)},
        {"default_features": 1},
    ],
)
def test_selected_unsupported_requests_fail_without_lookup(harness, options):
    api = harness
    with supervise(api) as supervisor:
        with pytest.raises(runtime.RuntimeError_):
            request(api, supervisor, **options)
        assert api.executor.preparations == 0
        assert not api.store.entries


def test_clean_bypasses_existing_once_per_action_per_session(harness):
    api = harness
    with supervise(api) as path:
        request(api, path)
    with supervise(api, clean=True) as path:
        first = request(api, path)
        second = request(api, path, "cargo:second")
        assert first["builder_calls"] == 1 and second["builder_calls"] == 0
        assert first["verified_rebuild"]
    assert api.executor.executions == 2


def test_verify_cold_builds_baseline_and_independent_comparison_then_dedups(harness):
    api = harness
    with supervise(api, verify=True) as path:
        first = request(api, path)
        second = request(api, path, "cargo:second")
        assert first["builder_calls"] == 2
        assert first["verification_compared"]
        assert second["builder_calls"] == 0
        assert second["verification_compared"]
    assert api.executor.executions == 2


def test_verify_existing_rebuilds_once_and_zero_request_is_error(harness):
    api = harness
    with supervise(api) as path:
        request(api, path)
    with supervise(api, verify=True) as path:
        assert request(api, path)["builder_calls"] == 1
    with (
        pytest.raises(runtime.RuntimeError_, match="no selected action compared"),
        supervise(api, verify=True),
    ):
        pass


def test_verify_nondeterminism_fails_no_success_receipt(harness):
    api = harness
    api.executor.nondeterministic = True
    with (
        pytest.raises(RuntimeError, match="independent output conflict"),
        supervise(api, verify=True) as path,
    ):
        request(api, path)
    assert not any(
        json.loads(p.read_text()).get("status", "success") == "success"
        for p in Path(runtime.supervisor_config(path)["activity"]).iterdir()
    )


@pytest.mark.parametrize("failure", ["mismatch_prepare", "invalid_report"])
def test_worker_context_or_acceptance_failure_cannot_publish(harness, failure):
    api = harness
    setattr(api.executor, failure, True)
    with supervise(api) as path, pytest.raises(runtime.RuntimeError_):
        request(api, path)
    assert not api.store.entries and not api.store.refs


def test_source_edit_while_waiting_rejects_hit_before_bind(harness):
    api = harness
    with supervise(api) as path:
        request(api, path)
        api.store.before_lookup = lambda: api.source.write_text("pub fn edited() {}\n")
        with pytest.raises(runtime.RuntimeError_, match="lane inputs changed"):
            request(api, path, "cargo:changed")
    assert (str(api.repo), "cargo:changed") not in api.store.refs


def test_dead_supervisor_rejected_before_source_discovery(harness):
    api = harness
    with supervise(api) as path:
        pass
    before = api.discovers
    with pytest.raises(RuntimeError, match="dead supervisor"):
        request(api, path)
    assert api.discovers == before and api.executor.executions == 0


def test_missing_durable_blob_fails_preflight_before_ninja(harness):
    api = harness
    with supervise(api) as path:
        handle = request(api, path)
    Path(handle["archive"]).unlink()
    with pytest.raises(RuntimeError, match="missing store blob"), supervise(api):
        pytest.fail("yielded before validating durable refs")


def test_changed_supervisor_toolchain_rejects_outer_build(harness):
    with (
        pytest.raises(runtime.RuntimeError_, match="toolchain/runtime changed"),
        supervise(harness),
    ):
        harness.executor.changed = True


def test_binding_failure_is_not_success(harness):
    api = harness
    api.store.fail_bind = True
    with supervise(api) as path, pytest.raises(RuntimeError, match="durable bind failure"):
        request(api, path)
    assert not any(
        json.loads(p.read_text()).get("status", "success") == "success"
        for p in Path(runtime.supervisor_config(path)["activity"]).iterdir()
    )


def test_public_supervisor_summary_counts_and_never_contains_capability(harness):
    api = harness
    with supervise(api, verify=True) as path:
        state = runtime.supervisor_config(path)
        request(api, path)
        request(api, path, "cargo:second")
        runtime.record_outer_result(path, 0, _api=api)
    summary = json.loads((api.context / "summary.json").read_text())
    assert summary["status"] == "success" and summary["verification_success"]
    assert summary["toolchain_postcheck"] == "identical"
    assert summary["outer_exit_code"] == 0
    assert summary["requests"]["requested"] == 2
    assert summary["requests"]["hits"] == 1 and summary["requests"]["misses"] == 1
    assert summary["requests"]["builder_calls"] == 2
    assert len(summary["requests"]["unique_actions"]) == 1
    assert summary["requests"]["sampled_worker_target_peak_allocated_bytes"] is None
    assert summary["requests"]["worker_requests_without_peak_measurement"] == 1
    assert all(value >= 0 for value in summary["timings"].values())
    assert summary["finished_ns"] >= summary["started_ns"]
    assert state["session_token"] not in json.dumps(summary)
    assert "session_token" not in json.dumps(summary)
    assert json.loads((path.parent / "summary.json").read_text()) == summary


def test_summary_outer_return_failure_preserves_caller_exit_code(harness):
    api = harness
    with supervise(api, verify=True) as path:
        runtime.record_outer_result(path, 7, _api=api)
    summary = json.loads((api.context / "summary.json").read_text())
    assert summary["status"] == "failed"
    assert summary["outer_exit_code"] == 7
    assert not summary["verification_success"]
    assert summary["error"]["type"] == "OuterBuildFailure"


def test_summary_failed_request_receipt_and_final_toolcheck_failure(harness):
    api = harness
    with (
        pytest.raises(runtime.RuntimeError_, match="toolchain/runtime changed"),
        supervise(api) as path,
    ):
        with pytest.raises(runtime.RuntimeError_, match="RUSTFLAGS"):
            request(api, path, flags="unsupported")
        api.executor.changed = True
    summary = json.loads((api.context / "summary.json").read_text())
    assert summary["status"] == "failed" and not summary["verification_success"]
    assert summary["toolchain_postcheck"] == "changed"
    assert summary["requests"]["failed_requests"] == 1
    assert summary["requests"]["requested"] == 1
    assert summary["requests"]["successful_requests"] == 0


def test_summary_initial_preflight_failure_still_persists(harness):
    api = harness
    (api.repo / ".cache/host-jobs/host-jobs").unlink()
    with pytest.raises(runtime.RuntimeError_, match="broker missing"), supervise(api):
        pass
    summary = json.loads((api.context / "summary.json").read_text())
    assert summary["status"] == "failed"
    assert summary["toolchain_postcheck"] == "not_run"
    assert summary["requests"]["requested"] == 0


def test_protocol_has_exactly_one_archive_and_rejects_control_paths():
    lines = runtime.protocol_lines({"archive": "/store/blob", "watches": ["/repo/a", "/repo/b"]})
    assert lines == "archive\t/store/blob\nwatch\t/repo/a\nwatch\t/repo/b\n"
    with pytest.raises(runtime.RuntimeError_):
        runtime.protocol_lines({"archive": "/store/bad\nblob", "watches": []})


def test_store_cannot_live_inside_checkout_or_contain_checkout(harness):
    api = harness
    with (
        pytest.raises(runtime.RuntimeError_, match="disjoint"),
        runtime.supervise(api.repo, api.repo / "target", api.toolchain, api.context, _api=api),
    ):
        pass
    with (
        pytest.raises(runtime.RuntimeError_, match="disjoint"),
        runtime.supervise(api.repo, api.repo.parent, api.toolchain, api.context, _api=api),
    ):
        pass


def test_broker_acquired_only_inside_store_builder(tmp_path, monkeypatch):
    # Test actual adapter launcher separately from the fake compiler backend.
    monkeypatch.delenv("VIBEOS_HOST_JOB_HELD", raising=False)
    output = tmp_path / "worker"
    output.mkdir()
    calls = []

    def run(command, **kwargs):
        calls.append((command, kwargs))
        (output / "report.json").write_text('{"compile_status":"success"}')
        return SimpleNamespace(returncode=0, stdout="", stderr="")

    monkeypatch.setattr(runtime.subprocess, "run", run)
    implementation = tmp_path / "implementation"
    executor = SimpleNamespace(__file__=str(implementation / "tools/artifact_executor.py"))
    result = runtime._execute(tmp_path, tmp_path / "host-jobs", output, executor)
    assert result["compile_status"] == "success"
    command, options = calls[0]
    assert command[:5] == [
        str(tmp_path / "host-jobs"),
        "--kind",
        "compile",
        "--",
        runtime.sys.executable,
    ]
    assert command[5:] == ["-m", "tools.artifact_executor", "run", str(output)]
    assert options["cwd"] == implementation


def test_held_compile_permit_avoids_nested_broker(monkeypatch):
    monkeypatch.setenv("VIBEOS_HOST_JOB_HELD", "1")
    executor = SimpleNamespace(run=lambda path: {"existing_permit": str(path)})
    assert runtime._execute(Path("/repo"), Path("/broker"), Path("/worker"), executor) == {
        "existing_permit": "/worker"
    }


def test_clean_ref_retirement_keeps_standalone_foreign_and_current_lease_pins(harness):
    api = harness
    with supervise(api) as path:
        original = request(api, path, "cargo:obsolete")
        request(api, path, "standalone:body")
    key = original["action_key"]
    api.store.refs["other-lane", "cargo:foreign"] = key
    with supervise(api, clean=True) as path:
        state = runtime.supervisor_config(path)
        owner = api.store.sessions[state["session_token"]]
        assert key in owner.pinned
        retired = runtime.retire_build_refs(path, _api=api)
        assert [ref["consumer"] for ref in retired] == ["cargo:obsolete"]
        assert (str(api.repo), "standalone:body") in api.store.refs
        assert ("other-lane", "cargo:foreign") in api.store.refs
        assert key in owner.pinned and owner.active
        assert Path(original["archive"]).is_file()
        assert runtime.retire_build_refs(path, _api=api) == []


def test_a_clean_build_retires_only_its_own_arch_refs(harness):
    # A clean i386 build removes i386's Cargo outputs only; amd64's fresh build scripts
    # still link their blobs, so their refs must survive it.
    api = harness
    i386 = "cargo:beep-cargo|/tmp/vibeos/sdk/lib/rust/target/beep-cargo-i386-cargo-link/out"
    amd64 = "cargo:beep-cargo|/tmp/vibeos/sdk/lib/rust/target/beep-cargo-amd64-cargo-link/out"
    with supervise(api) as path:
        request(api, path, i386)
        request(api, path, amd64)
    with supervise(api, clean=True, arch="i386") as path:
        retired = runtime.retire_build_refs(path, _api=api)
    assert [ref["consumer"] for ref in retired] == [i386]
    assert (str(api.repo), amd64) in api.store.refs


def test_warm_build_cannot_retire_cargo_references(harness):
    with supervise(harness) as path, pytest.raises(runtime.RuntimeError_, match="clean supervisor"):
        runtime.retire_build_refs(path, _api=harness)


def test_cli_request_protocol_and_actual_options(monkeypatch, capsys):
    observed = []

    def fake(supervisor, body, consumer, **options):
        observed.append((supervisor, body, consumer, options))
        return {"archive": "/store/body", "watches": ["/repo/package"]}

    monkeypatch.setattr(runtime, "request", fake)
    monkeypatch.setattr(
        runtime.sys,
        "argv",
        [
            "artifact_runtime.py",
            "request",
            "--supervisor",
            "/private/supervisor.json",
            "--body",
            "kernel-rust",
            "--consumer",
            "cargo:kernel",
            "--features",
            "smp,lockdep",
            "--no-default-features",
            "--kernel",
            "--flags",
            runtime.KERNEL_FLAGS + " ",
        ],
    )
    assert runtime.main() == 0
    assert capsys.readouterr().out == "archive\t/store/body\nwatch\t/repo/package\n"
    assert observed == [
        (
            Path("/private/supervisor.json"),
            "kernel-rust",
            "cargo:kernel",
            {
                "features": ("smp", "lockdep"),
                "default_features": False,
                "kernel": True,
                "flags": runtime.KERNEL_FLAGS + " ",
                "architecture": "amd64",
                "target": runtime.TARGET,
            },
        )
    ]


def test_cli_failure_is_stderr_and_nonzero_without_fallback(monkeypatch, capsys):
    def failure(*args, **kwargs):
        raise runtime.RuntimeError_("selected body cannot execute")

    monkeypatch.setattr(runtime, "request", failure)
    monkeypatch.setattr(
        runtime.sys,
        "argv",
        [
            "artifact_runtime.py",
            "request",
            "--supervisor",
            "/private/supervisor.json",
            "--body",
            "beep-rust",
            "--consumer",
            "cargo:test",
        ],
    )
    assert runtime.main() == 2
    output = capsys.readouterr()
    assert output.out == ""
    assert "selected body cannot execute" in output.err


def test_an_ineligible_graph_builds_privately_and_is_counted_unless_strict(harness, monkeypatch):
    api = harness
    api.ineligible = True
    with supervise(api) as path:
        result = request(api, path)
        assert result["status"] == "ineligible"
        assert result["reasons"] == ["unsupported fixture input"]
        assert runtime.protocol_lines(result) == "ineligible\tunsupported fixture input\n"
        assert api.executor.preparations == 0
    summary = json.loads((api.context / "summary.json").read_text())
    assert summary["requests"]["ineligible_requests"] == 1
    assert summary["requests"]["ineligible_bodies"] == {"beep-rust": "unsupported fixture input"}
    assert summary["requests"]["hits"] == summary["requests"]["misses"] == 0
    monkeypatch.setenv("VIBEOS_ARTIFACT_STRICT", "1")
    with supervise(api) as path, pytest.raises(runtime.RuntimeError_, match="strict store"):
        request(api, path)


def test_any_workspace_staticlib_is_a_candidate_not_just_the_pilot_six(harness):
    api = harness
    package = api.repo / "sdk/lib/rust/other-rust"
    (package / "src").mkdir(parents=True)
    (package / "Cargo.toml").write_text(
        '[package]\nname="other-rust"\nversion="0.1.0"\n[lib]\ncrate-type=["staticlib"]\n'
    )
    with supervise(api) as path:
        assert runtime.request(path, "other-rust", "cargo:other", _api=api)["builder_calls"] == 1
        # A body that is also a Rust dependency (staticlib + rlib) is a candidate too.
        both = api.repo / "sdk/lib/rust/both-rust"
        (both / "src").mkdir(parents=True)
        (both / "Cargo.toml").write_text(
            '[package]\nname="both-rust"\nversion="0.1.0"\n[lib]\ncrate-type=["staticlib","rlib"]\n'
        )
        assert runtime.request(path, "both-rust", "cargo:both", _api=api)["builder_calls"] == 1
        # A package that is not a manifest staticlib is ineligible, not an error.
        missing = runtime.request(path, "no-such-rust", "cargo:none", _api=api)
        assert missing["status"] == "ineligible"
