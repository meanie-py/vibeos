"""Receipts must prove identical inputs, not timestamps or a convenient green tree."""

import json
import os
import subprocess
import time
from concurrent.futures import ThreadPoolExecutor

import pytest

from tests.qemu.cli import tooling_receipts as receipts

LABEL = "ruff:tools/*.py"


@pytest.fixture
def repo(tmp_path, monkeypatch):
    subprocess.run(["git", "init", "-q", str(tmp_path)], check=True)
    (tmp_path / "source.py").write_text("answer = 41\n")
    subprocess.run(["git", "add", "source.py"], cwd=tmp_path, check=True)
    monkeypatch.setattr(receipts, "runtime_identity", lambda _: "runtime")
    return tmp_path


def invoke(repo, compute=lambda: (LABEL, True, "passed")):
    with receipts.ReceiptSession(repo) as session:
        result = session.run(LABEL, ["ruff", "check", "."], compute)
        hits = set(session.hits)
        session.finish()
    return result, hits


def test_success_reused_and_failures_never_published(repo):
    assert not invoke(repo)[1]
    assert invoke(repo, lambda: pytest.fail("receipt missed"))[1] == {LABEL}
    (repo / "source.py").write_text("broken\n")
    assert not invoke(repo, lambda: (LABEL, False, "bad"))[0][1]
    assert invoke(repo)[0][1]
    assert invoke(repo)[1] == {LABEL}


@pytest.mark.parametrize("mutation", ["content", "new", "deleted", "mode"])
def test_source_changes_invalidate_even_with_restored_mtime(repo, mutation):
    invoke(repo)
    source = repo / "source.py"
    before = source.stat()
    if mutation == "content":
        source.write_text("answer = 42\n")
        os.utime(source, ns=(before.st_atime_ns, before.st_mtime_ns))
    elif mutation == "new":
        (repo / "new_test.py").write_text("def test_new(): assert False\n")
    elif mutation == "deleted":
        source.unlink()
    else:
        source.chmod(0o755)
    assert not invoke(repo)[1]


def test_notes_do_not_hide_new_source_inputs(repo):
    invoke(repo)
    (repo / "handoff.md").write_text("user notes")
    assert invoke(repo)[1] == {LABEL}
    (repo / "config.toml").write_text("enabled = false")
    assert not invoke(repo)[1]


def test_changed_during_run_never_publishes(repo):
    def change():
        (repo / "source.py").write_text("changed")
        return LABEL, True, "passed old source"

    with pytest.raises(RuntimeError, match="inputs changed"):
        invoke(repo, change)
    assert not list((repo / ".git/vibeos-tooling-receipts").glob("*.json"))


def test_corruption_is_loud(repo):
    invoke(repo)
    path = next((repo / ".git/vibeos-tooling-receipts").glob("*.json"))
    path.write_text("{broken")
    with pytest.raises(RuntimeError, match="Corrupt tooling receipt"):
        invoke(repo)


def test_expired_receipt_reruns(repo):
    invoke(repo)
    path = next((repo / ".git/vibeos-tooling-receipts").glob("*.json"))
    data = json.loads(path.read_text())
    data["created"] = 0
    path.write_text(json.dumps(data))
    assert not invoke(repo)[1]


def test_finish_prunes_receipts_past_max_age(repo):
    """Every commit mints a fresh receipt set (the key hashes every tracked file) and
    nothing ever removed the old ones: 90,882 files / 358 MB on 2026-10-09."""
    invoke(repo)
    store = repo / ".git/vibeos-tooling-receipts"
    fresh = next(store.glob("*.json"))
    stale = store / ("0" * 64 + ".json")
    stale.write_text(fresh.read_text())
    old = time.time() - receipts._MAX_AGE - 60
    os.utime(stale, (old, old))
    invoke(repo)
    assert not stale.exists()
    assert fresh.exists()


def test_concurrent_identical_invocations_compute_once(repo):
    calls = []

    def compute():
        calls.append(1)
        return LABEL, True, "passed"

    with ThreadPoolExecutor(max_workers=2) as pool:
        results = list(pool.map(lambda _: invoke(repo, compute), range(2)))
    assert len(calls) == 1
    assert sum(bool(hits) for _, hits in results) == 1


def test_shared_receipts_normalize_worktree_paths(repo, tmp_path):
    subprocess.run(
        ["git", "-c", "user.name=T", "-c", "user.email=t@t", "commit", "-qm", "fixture"],
        cwd=repo,
        check=True,
    )
    other = tmp_path.parent / f"{tmp_path.name}-other"
    subprocess.run(["git", "worktree", "add", "-q", "--detach", str(other)], cwd=repo, check=True)
    invoke(repo)
    assert invoke(other, lambda: pytest.fail("identical worktree missed"))[1] == {LABEL}


def test_live_suites_are_never_reused(repo):
    for label in ("pytest:tools/test_host_jobs.py", "pytest:tools/test_verify_ucrtbase_cohab.py"):
        assert not receipts.eligibility(label)
        for _ in range(2):
            with receipts.ReceiptSession(repo) as session:
                assert (
                    session.run(label, [], lambda label=label: (label, True, "live"))[2] == "live"
                )
                assert not session.hits
                session.finish()


def test_generated_headers_affect_only_declared_suite(repo):
    label = "pytest:tools/rustconv/tests"

    def run():
        with receipts.ReceiptSession(repo) as session:
            session.run(label, [], lambda: (label, True, "passed"))
            hit = label in session.hits
            session.finish()
            return hit

    invoke(repo)
    assert not run()
    assert run()
    generated = repo / "build-amd64/sdk/include/psdk"
    generated.mkdir(parents=True)
    # As in the real checkout, generated build inputs are not source inputs.
    (repo / ".git/info/exclude").write_text("build-*/\n")
    (generated / "oleidl.h").write_text("new generated interface")
    assert not run()
    assert invoke(repo)[1] == {LABEL}


def test_headers_regenerated_under_a_reused_result_are_rerun_not_failed(repo):
    # A gate's builds rewrite build-*/sdk/include while check-tooling runs. A receipt
    # reused at the start is then stale: the caller reruns it instead of the gate
    # failing the lane (shell32-com b20d7379364c), and without the rerun it still fails.
    label = "pytest:tools/rustconv/tests"
    (repo / ".git/info/exclude").write_text("build-*/\n")
    header = repo / "build-amd64/sdk/include/psdk/oleidl.h"
    header.parent.mkdir(parents=True)
    header.write_text("v1")
    with receipts.ReceiptSession(repo) as session:
        session.run(label, [], lambda: (label, True, "passed"))
        session.finish()

    def session_with_headers_moving():
        session = receipts.ReceiptSession(repo)
        session.__enter__()
        session.run(label, [], lambda: pytest.fail("receipt missed"))
        assert label in session.hits
        header.write_text("v2")
        return session

    session = session_with_headers_moving()
    assert session.stale_generated_hits() == {label}
    session.mark_rerun(label)
    session.finish()
    session.__exit__(None, None, None)

    header.write_text("v1")
    session = session_with_headers_moving()
    with pytest.raises(RuntimeError, match="after receipt reuse"):
        session.finish()
    session.__exit__(None, None, None)


def test_runtime_environment_and_actual_tool_bytes(tmp_path, monkeypatch):
    tool = tmp_path / "compiler"
    tool.write_text("compiler one")
    monkeypatch.setattr(receipts.importlib.metadata, "distributions", lambda: [])
    monkeypatch.setattr(
        receipts.shutil, "which", lambda name: None if name == "rustup" else str(tool)
    )
    first = receipts.runtime_identity(tmp_path)
    old = tool.stat()
    tool.write_text("compiler two")
    os.utime(tool, ns=(old.st_atime_ns, old.st_mtime_ns))
    second = receipts.runtime_identity(tmp_path)
    assert second != first
    monkeypatch.setenv("PYTHONWARNINGS", "error")
    assert receipts.runtime_identity(tmp_path) != second


def test_new_registry_suite_is_not_automatically_cacheable(monkeypatch):
    from tests.qemu.cli import tooling

    monkeypatch.setattr(
        tooling, "_PYTEST_SUITES", [*tooling._PYTEST_SUITES, (".", ".", "tools/test_live_new.py")]
    )
    assert not receipts.eligibility("pytest:tools/test_live_new.py")
    assert receipts.eligibility("pytest:tools/rustconv/tests")


@pytest.mark.parametrize("at_root", [False, True])
def test_generated_directory_symlinks_fail_loudly(tmp_path, at_root):
    generated = tmp_path / "generated"
    foreign = tmp_path / "foreign"
    foreign.mkdir()
    (foreign / "layout.h").write_text("mutable layout")
    if at_root:
        generated.symlink_to(foreign, target_is_directory=True)
    else:
        generated.mkdir()
        (generated / "headers").symlink_to(foreign, target_is_directory=True)
    with pytest.raises(RuntimeError, match="directory symlink"):
        receipts._tree(generated)


def test_generated_symlinked_build_parent_is_rejected(tmp_path):
    foreign = tmp_path / "other" / "sdk" / "include"
    foreign.mkdir(parents=True)
    (tmp_path / "build-amd64").symlink_to(tmp_path / "other", target_is_directory=True)
    with pytest.raises(RuntimeError, match="directory symlink"):
        receipts._tree(tmp_path / "build-amd64" / "sdk" / "include")
