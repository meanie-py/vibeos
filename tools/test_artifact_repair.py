"""Explicit missing-only runtime repair, including one real isolated rebuild."""

import argparse
import json
from contextlib import contextmanager
from pathlib import Path
from types import SimpleNamespace

import pytest

from tests.qemu.cli import artifact_store as cli
from tools import artifact_runtime as runtime
from tools import artifact_store as storage
from tools import test_artifact_runtime as adapter_tests
from tools.artifact_executor import PIN
from tools.test_artifact_runtime_real import source_tree

harness = adapter_tests.harness


@pytest.fixture
def indexed(harness):
    """Actual Store/leases/manifests/refs, deterministic fake compiler bytes."""
    api = harness
    api.store = storage.Store(api.store.root)
    # FakeExecutor assertion; the real Store owns its OS locks independently.
    api.store.locked = True  # pyright: ignore[reportAttributeAccessIssue]
    api.storage = storage
    original = api.discover

    def discover(root, body, context, **options):
        result = original(root, body, context, **options)
        result["descriptor"]["request"].update(
            features=sorted(options.get("features", ())),
            default_features=options.get("default_features", True),
            profile="release",
            invocation="cargo build --lib",
        )
        result["action_key"] = runtime._key(result["descriptor"])
        return result

    api.discover = discover
    api.inputs.discover = discover
    with adapter_tests.supervise(api) as supervisor:
        receipt = adapter_tests.request(api, supervisor)
    api.initial = receipt
    api.blob = Path(receipt["archive"])
    api.manifest = Path(receipt["manifest"])
    api.original_blob = api.blob.read_bytes()
    api.original_manifest = api.manifest.read_bytes()
    return api


def repair_supervisor(api):
    return adapter_tests.supervise(api, repair_action=api.initial["action_key"])


def refs(api):
    return {path.name: path.read_bytes() for path in (api.store.base / "refs").glob("*.json")}


def test_default_preflight_still_fails_then_explicit_repair_restores_exact_blob(indexed):
    api = indexed
    original_refs = refs(api)
    api.blob.unlink()
    with pytest.raises(storage.CorruptStore), adapter_tests.supervise(api):
        pytest.fail("normal supervisor bypassed missing durable input")
    before = api.executor.executions
    with repair_supervisor(api) as supervisor:
        receipt = runtime.repair(supervisor, api.initial["action_key"], _api=api)
        assert receipt["repaired"] and receipt["reference"] is None
        assert receipt["builder_calls"] == 1
    assert api.executor.executions == before + 1
    assert api.blob.read_bytes() == api.original_blob
    assert api.manifest.read_bytes() == api.original_manifest
    assert refs(api) == original_refs
    summary = json.loads((api.context / "summary.json").read_text())
    assert summary["status"] == "success" and summary["repair_success"]
    assert summary["requests"]["repaired_actions"] == [api.initial["action_key"]]
    assert summary["preflight_mode"] == "pin repair metadata only"
    assert summary["restored_actions"] == [api.initial["action_key"]]
    with adapter_tests.supervise(api) as supervisor:
        assert adapter_tests.request(api, supervisor)["builder_calls"] == 0


@pytest.mark.parametrize("change", ["source", "toolchain"])
def test_repair_rejects_changed_current_key_before_builder(indexed, change):
    api = indexed
    api.blob.unlink()
    if change == "source":
        api.source.write_text("pub fn changed() {}\n")
    else:
        api.executor.changed = True
    before = api.executor.preparations
    with (
        pytest.raises(runtime.RuntimeError_, match="current"),
        repair_supervisor(api) as supervisor,
    ):
        runtime.repair(supervisor, api.initial["action_key"], _api=api)
    assert api.executor.preparations == before
    assert not api.blob.exists()
    assert api.manifest.read_bytes() == api.original_manifest


def test_present_corrupt_blob_is_never_repaired_or_replaced(indexed):
    api = indexed
    api.blob.chmod(0o644)
    api.blob.write_bytes(b"corrupted existing bytes")
    before = api.executor.preparations
    with pytest.raises(storage.CorruptStore), repair_supervisor(api) as supervisor:
        runtime.repair(supervisor, api.initial["action_key"], _api=api)
    assert api.executor.preparations == before
    assert api.blob.read_bytes() == b"corrupted existing bytes"
    assert api.manifest.read_bytes() == api.original_manifest


def test_repair_requires_absent_blob_and_never_adds_reference(indexed):
    api = indexed
    original_refs = refs(api)
    before = api.executor.preparations
    with (
        pytest.raises(storage.StoreError, match="at least one absent"),
        repair_supervisor(api) as supervisor,
    ):
        runtime.repair(supervisor, api.initial["action_key"], _api=api)
    assert api.executor.preparations == before
    assert refs(api) == original_refs


def test_repair_supervisor_cannot_serve_consumers_or_succeed_without_repair(indexed):
    api = indexed
    api.blob.unlink()
    with (
        pytest.raises(runtime.RuntimeError_, match="repair-only"),
        repair_supervisor(api) as supervisor,
    ):
        adapter_tests.request(api, supervisor, "cargo:must-not-bind")
    assert len(refs(api)) == 1
    with (
        pytest.raises(runtime.RuntimeError_, match="no matching successful repair"),
        repair_supervisor(api),
    ):
        pass


def test_normal_supervisor_cannot_repair(indexed):
    api = indexed
    with (
        adapter_tests.supervise(api) as supervisor,
        pytest.raises(runtime.RuntimeError_, match="repair-only"),
    ):
        runtime.repair(supervisor, api.initial["action_key"], _api=api)
    assert not json.loads((api.context / "summary.json").read_text())["repair_success"]


def test_repair_revalidates_all_current_lane_refs(indexed):
    api = indexed
    api.source.write_text("pub fn another_action() {}\n")
    with adapter_tests.supervise(api) as supervisor:
        second = adapter_tests.request(api, supervisor, "cargo:second")
    Path(second["archive"]).unlink()
    api.source.write_text("pub fn body() {}\n")
    api.blob.unlink()
    with pytest.raises(storage.CorruptStore), repair_supervisor(api) as supervisor:
        runtime.repair(supervisor, api.initial["action_key"], _api=api)
    assert api.blob.read_bytes() == api.original_blob
    assert len(refs(api)) == 2
    assert not json.loads((api.context / "summary.json").read_text())["repair_success"]


def test_cli_parser_and_repair_route_keep_disk_admission_and_evidence(tmp_path, monkeypatch):
    from tests.qemu.cli import diskspace, hostjobs

    parser = argparse.ArgumentParser()
    cli.add_parser(parser.add_subparsers())
    store = tmp_path / "store"
    (store / "v1").mkdir(parents=True)
    key = "a" * 64
    output = tmp_path / "evidence"
    args = parser.parse_args(
        [
            "artifact-store",
            "repair",
            "--store",
            str(store),
            "--action",
            key,
            "--output",
            str(output),
        ]
    )
    observed = []

    @contextmanager
    def reserved(repo, kind):
        observed.append(("disk", kind))
        yield

    @contextmanager
    def supervised(*args, **options):
        assert observed == [("disk", "check-tooling")]
        assert options == {"repair_action": key}
        yield tmp_path / "supervisor.json"

    def child(command, **options):
        observed.append(("child", command))
        return SimpleNamespace(
            returncode=0, stdout=json.dumps({"action_key": key, "repaired": True}), stderr=""
        )

    monkeypatch.setattr(diskspace, "reserve", reserved)
    monkeypatch.setattr(runtime, "supervise", supervised)
    monkeypatch.setattr(hostjobs, "ensure", lambda repo: None)
    monkeypatch.setattr(hostjobs, "environment", lambda *args, **options: {})
    monkeypatch.setattr(cli.subprocess, "run", child)
    assert args.fn(args, tmp_path) == 0
    command = observed[-1][1]
    assert command[2:5] == ["tools.artifact_runtime", "repair", "--supervisor"]
    assert command[-2:] == ["--action", key]
    assert (output / "repair.stdout").is_file() and (output / "repair.stderr").is_file()


def test_real_isolated_missing_blob_repair_restores_indexed_bytes_and_refs(tmp_path):
    repo = tmp_path / "real-lane"
    source_tree(repo)
    root = tmp_path / "store"
    toolchain = Path.home() / ".rustup/toolchains" / PIN
    context = tmp_path / "context"
    with runtime.supervise(repo, root, toolchain, context) as supervisor:
        original = runtime.request(supervisor, "beep-rust", "fixture:real")
    archive = Path(original["archive"])
    payload = archive.read_bytes()
    manifest = Path(original["manifest"])
    metadata = manifest.read_bytes()
    store = storage.Store(root)
    references = store.list_refs(str(repo.resolve()))
    archive.unlink()
    with pytest.raises(storage.CorruptStore), runtime.supervise(repo, root, toolchain, context):
        pytest.fail("real normal preflight yielded despite missing blob")
    with runtime.supervise(
        repo, root, toolchain, context, repair_action=original["action_key"]
    ) as supervisor:
        repaired = runtime.repair(supervisor, original["action_key"])
        assert repaired["builder_calls"] == 1 and repaired["reference"] is None
    assert archive.read_bytes() == payload and manifest.read_bytes() == metadata
    assert store.list_refs(str(repo.resolve())) == references
    with runtime.supervise(repo, root, toolchain, context) as supervisor:
        hit = runtime.request(supervisor, "beep-rust", "fixture:real")
        assert hit["hit"] and hit["builder_calls"] == 0
