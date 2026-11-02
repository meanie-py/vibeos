"""Native repair's Python CLI admission and real isolated reproduction contracts.

Missing/corrupt/key-change controls use the native RuntimeHost tests; see the
migration handback mapping. No Python compiler/store injection remains.
"""

import argparse
import json
from contextlib import contextmanager
from pathlib import Path
from types import SimpleNamespace

import pytest

from tests.qemu.cli import artifact_store as cli
from tools import artifact_runtime as runtime
from tools import artifact_store as storage
from tools.artifact_executor import PIN
from tools.test_artifact_runtime_real import source_tree


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
    assert command[:4] == [
        str(tmp_path / "tools/build-infra"),
        "artifact-runtime",
        "repair",
        "--supervisor",
    ]
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
