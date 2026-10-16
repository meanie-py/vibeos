"""Discovery commands preserve evidence and expose incomplete measurements."""

import argparse
import json
import subprocess
from pathlib import Path
from types import SimpleNamespace

from . import artifact_store


def test_build_rejects_unsupported_store_modes_before_build(tmp_path, monkeypatch):
    from . import build

    class Run:
        classification = ""

        def __init__(self):
            self.stages = []

        def add(self, stage):
            self.stages.append(stage)

        def finalize(self, repo):
            return 2

    def unexpected(*args, **kwargs):
        raise AssertionError("unsupported store request reached the build")

    monkeypatch.setattr(build, "_build_locked", unexpected)
    for root, verify, arch, incremental in (
        (None, True, "amd64", False),
        (tmp_path.__class__("relative"), False, "amd64", False),
        (tmp_path / "store", False, "amd64", True),
    ):
        args = SimpleNamespace(
            artifact_store=root, artifact_store_verify=verify, incremental=incremental
        )
        run = Run()
        assert build._build_supervised(args, tmp_path, run, arch, None, tmp_path, False) == 2
        assert run.classification == "ARTIFACT_STORE_FAIL"


def test_build_uses_the_shared_store_by_default_on_every_arch(tmp_path, monkeypatch):
    from . import build

    supervised = []

    def supervise(repo, store_root, *args, **kwargs):
        supervised.append(store_root)
        raise ValueError("stop before building")

    monkeypatch.setenv("VIBEOS_ARTIFACT_STORE_ROOT", str(tmp_path / "shared"))
    monkeypatch.setattr("tools.artifact_runtime.supervise", supervise)
    monkeypatch.setattr(build, "_build_locked", lambda *a, **k: 0)
    for arch in ("i386", "amd64", "arm64"):
        args = SimpleNamespace(artifact_store=None, artifact_store_verify=False, incremental=False)
        build._build_supervised(args, tmp_path, _Run(), arch, None, tmp_path, False)
    assert supervised == [tmp_path / "shared"] * 3
    # Opted out, or incremental: the private path, and no store.
    for opt in ({"no_artifact_store": True}, {"incremental": True}):
        args = SimpleNamespace(
            artifact_store=None, artifact_store_verify=False, **{"incremental": False, **opt}
        )
        assert build._build_supervised(args, tmp_path, _Run(), "amd64", None, tmp_path, False) == 0
    assert len(supervised) == 3


class _Run:
    classification = ""
    dir = Path("/nonexistent-run")

    def __init__(self):
        self.extra = {}

    def add(self, stage):
        pass

    def finalize(self, repo):
        return 2


def arguments(*argv):
    parser = argparse.ArgumentParser()
    artifact_store.add_parser(parser.add_subparsers())
    return parser.parse_args(["artifact-store", *argv])


def test_inventory_incomplete_is_failure_with_report(tmp_path, monkeypatch):
    from tools import rust_storage_report

    monkeypatch.setattr(
        rust_storage_report,
        "measure",
        lambda *a, **kw: {"allocated_bytes": 42, "errors": ["changed during read"]},
    )
    output = tmp_path / "evidence.json"
    args = arguments("inventory", "--output", str(output))
    assert args.fn(args, tmp_path) == 2
    assert json.loads(output.read_text())["errors"] == ["changed during read"]
    output.write_text("prior evidence")
    assert args.fn(args, tmp_path) == 2
    assert output.read_text() == "prior evidence"


def test_plan_records_ineligible_without_claiming_hit(tmp_path, monkeypatch):
    from tools import artifact_inputs

    monkeypatch.setattr(
        artifact_inputs,
        "discover",
        lambda *a, **kw: {"eligible": False, "reasons": ["build script unsupported"]},
    )
    monkeypatch.setattr(artifact_store, "_revision", lambda _: "revision")
    output = tmp_path / "plan.json"
    args = arguments("plan", "--body", "example", "--output", str(output))
    assert args.fn(args, tmp_path) == 0
    report = json.loads(output.read_text())
    assert report["kind"] == "input-projection-only"
    assert report["records"][0]["result"]["eligible"] is False
    assert "not observed artifact hits" in report["note"]


def test_plan_bad_context_fails(tmp_path):
    context = tmp_path / "context.json"
    context.write_text("[]")
    output = tmp_path / "plan.json"
    args = arguments(
        "plan", "--body", "example", "--context", str(context), "--output", str(output)
    )
    assert args.fn(args, tmp_path) == 2
    assert not output.exists()


def test_historical_capture_uses_revision_not_dirty_files(tmp_path):
    from tools.artifact_history import capture

    repo = tmp_path / "repo"
    workspace = repo / "sdk/lib/rust"
    body = workspace / "body"
    body.mkdir(parents=True)
    (workspace / "Cargo.toml").write_text('[workspace]\nmembers=["body"]\n')
    (workspace / "Cargo.lock").write_text("version=4\n")
    (body / "Cargo.toml").write_text('[package]\nname="body"\nversion="0.1.0"\n')
    source = body / "lib.rs"
    source.write_text("original")
    for args in (
        ["init", "-q"],
        ["add", "."],
        [
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "commit",
            "-qm",
            "fixture",
        ],
    ):
        subprocess.run(["git", "-C", str(repo), *args], check=True)
    source.write_text("dirty")
    output = tmp_path / "snapshot"
    revision = capture(repo, "HEAD", "body", output)
    assert len(revision) == 40
    assert (output / "sdk/lib/rust/body/lib.rs").read_text() == "original"
    assert source.read_text() == "dirty"
