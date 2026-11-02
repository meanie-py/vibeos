"""Shipping facade contracts; controlled compiler failures live in native RuntimeHost tests.

See docs/engineering/artifact-native-test-coverage.md for the old-to-native control mapping.
These tests invoke native mechanisms rather than injecting Python stores.
"""

import fcntl
import json
import os
import subprocess
import sys
import time
from pathlib import Path

import pytest

from tools import artifact_runtime as runtime
from tools import artifact_store as storage

ROOT = Path(__file__).resolve().parents[1]


@pytest.fixture
def supervisor_state(tmp_path):
    repo = tmp_path / "repo"
    repo.mkdir()
    activity = tmp_path / "activity"
    activity.mkdir()
    provisioned = {"compiler": "fixture descriptor"}
    state = {
        "schema": runtime.SCHEMA,
        "repo": str(repo),
        "store_root": str(tmp_path / "store"),
        "toolchain": str(tmp_path / "toolchain"),
        "activity": str(activity),
        "broker": str(repo / "broker"),
        "provisioned": provisioned,
        "toolchain_key": runtime._key(provisioned),
        "clean": False,
        "verify": False,
        "repair_action": None,
        "outer_result": str(tmp_path / "outer.json"),
    }
    path = tmp_path / "supervisor.json"
    path.write_text(json.dumps(state))
    return path, state


def test_supervisor_config_reads_native_schema_and_identity(supervisor_state):
    path, state = supervisor_state
    assert runtime.supervisor_config(path) == state


@pytest.mark.parametrize(
    "field,value,message",
    [
        ("schema", "unknown", "schema"),
        ("repo", "relative", "absolute"),
        ("clean", 1, "booleans"),
        ("verify", "false", "booleans"),
        ("toolchain_key", "0" * 64, "descriptor changed"),
        ("repair_action", "A" * 64, "SHA-256"),
    ],
)
def test_supervisor_config_rejects_invalid_private_context(supervisor_state, field, value, message):
    path, state = supervisor_state
    state[field] = value
    path.write_text(json.dumps(state))
    with pytest.raises(runtime.RuntimeError_, match=message):
        runtime.supervisor_config(path)


def test_supervisor_config_refuses_symlink_and_non_object_json(supervisor_state):
    path, _state = supervisor_state
    alias = path.with_name("alias.json")
    alias.symlink_to(path)
    with pytest.raises((runtime.RuntimeError_, OSError)):
        runtime.supervisor_config(alias)
    path.write_text("[]")
    with pytest.raises(runtime.RuntimeError_, match="object"):
        runtime.supervisor_config(path)


def test_record_outer_result_attaches_actual_native_session(supervisor_state):
    path, state = supervisor_state
    store = storage.Store(Path(state["store_root"]))
    with store.session() as session:
        state["session_token"] = session.token
        path.write_text(json.dumps(state))
        runtime.record_outer_result(path, 7)
        assert json.loads(Path(state["outer_result"]).read_text()) == {"exit_code": 7}
        with pytest.raises(runtime.RuntimeError_, match="clean supervisor"):
            runtime.retire_build_refs(path)
        state["clean"] = True
        path.write_text(json.dumps(state))
        assert runtime.retire_build_refs(path) == []
    Path(state["outer_result"]).unlink()
    with pytest.raises(storage.SessionError):
        runtime.record_outer_result(path, 0)
    assert not Path(state["outer_result"]).exists()


def test_native_receipt_aggregation_does_not_count_failure_as_success(tmp_path):
    action = "a" * 64
    receipts = [
        {
            "body": "body-rust",
            "action_key": action,
            "hit": False,
            "builder_calls": 2,
            "verification_compared": True,
            "verified_rebuild": True,
            "sampled_worker_target_peak_allocated_bytes": 123,
            "timings": {"compile_seconds": 1.25},
        },
        {"body": "bad-rust", "status": "failed", "error": {"message": "compile failed"}},
        {"body": "other-rust", "status": "ineligible", "reasons": ["unsupported graph"]},
    ]
    for index, receipt in enumerate(receipts):
        (tmp_path / f"{index}.json").write_text(json.dumps(receipt))
    result = runtime._receipt_stats(tmp_path)
    assert (result["requested"], result["successful_requests"], result["failed_requests"]) == (
        3,
        1,
        1,
    )
    assert result["ineligible_bodies"] == {"other-rust": "unsupported graph"}
    assert result["hits"] == 0 and result["misses"] == 1
    assert result["builder_calls"] == 2 and result["failed_request_builder_calls"] == "unknown"
    assert result["unique_actions"] == [action]
    assert result["request_timings"]["compile_seconds"] == 1.25
    assert result["sampled_worker_target_peak_allocated_bytes"] == 123


def test_protocol_has_exactly_one_archive_and_rejects_control_paths():
    assert (
        runtime.protocol_lines({"archive": "/store/blob", "watches": ["/repo/a", "/repo/b"]})
        == "archive\t/store/blob\nwatch\t/repo/a\nwatch\t/repo/b\n"
    )
    for path in ["relative", "/store/bad\nblob", "/store/bad\tblob", "/store/bad\0blob"]:
        with pytest.raises(runtime.RuntimeError_):
            runtime.protocol_lines({"archive": path, "watches": []})
    assert (
        runtime.protocol_lines(
            {"status": "ineligible", "reasons": ["unsupported\ngraph", "private\tpath"]}
        )
        == "ineligible\tunsupported graph; private path\n"
    )


def test_python_host_injection_is_explicitly_retired(tmp_path):
    with (
        pytest.raises(runtime.RuntimeError_, match="native RuntimeHost"),
        runtime.supervise(tmp_path, tmp_path, tmp_path, tmp_path, _api=object()),
    ):
        pytest.fail("Python host injection yielded")
    operations = [
        lambda: runtime.request(tmp_path, "body-rust", "cargo:body", _api=object()),
        lambda: runtime.repair(tmp_path, "a" * 64, _api=object()),
        lambda: runtime.record_outer_result(tmp_path, 0, _api=object()),
        lambda: runtime.retire_build_refs(tmp_path, _api=object()),
    ]
    for operation in operations:
        with pytest.raises(runtime.RuntimeError_, match="native RuntimeHost"):
            operation()


def test_cli_wrapper_forwards_raw_options_to_native_parser(monkeypatch):
    arguments = [
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
    ]
    observed = []
    monkeypatch.setattr(runtime.sys, "argv", ["artifact_runtime.py", *arguments])
    monkeypatch.setattr(runtime.os, "execv", lambda path, argv: observed.append((path, argv)))
    runtime.main()
    tool = str(runtime.bridge.REPO / "tools/build-infra")
    assert observed == [(tool, [tool, "artifact-runtime", *arguments])]


def test_cli_failure_is_stderr_and_nonzero_without_fallback(tmp_path):
    result = subprocess.run(
        [
            sys.executable,
            "-m",
            "tools.artifact_runtime",
            "request",
            "--supervisor",
            str(tmp_path / "missing-supervisor.json"),
            "--body",
            "beep-rust",
            "--consumer",
            "cargo:test",
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
        timeout=60,
        check=False,
    )
    assert result.returncode == 2
    assert result.stdout == ""
    assert result.stderr
    assert not list(tmp_path.iterdir())


def test_direct_native_request_dies_with_cargo_owner_while_waiting_for_store_lock(
    supervisor_state,
):
    path, state = supervisor_state
    store = storage.Store(Path(state["store_root"]))
    # Provision before the synthetic Cargo owner starts: the process under test
    # must reach the real native lock, rather than waiting for a bootstrap build.
    subprocess.run([str(ROOT / "tools/build-infra"), "--help"], check=True, capture_output=True)
    with store.session() as session:
        state["session_token"] = session.token
        path.write_text(json.dumps(state))
        with (Path(state["store_root"]) / "v1/locks/gc.lock").open("rb") as lock:
            fcntl.flock(lock, fcntl.LOCK_EX)
            command = [
                str(ROOT / "tools/build-infra"),
                "artifact-runtime",
                "request",
                "--supervisor",
                str(path),
                "--body",
                "beep-rust",
                "--consumer",
                "fixture:owner",
            ]
            program = (
                "import subprocess,sys; "
                "p=subprocess.Popen(sys.argv[1:],stdout=subprocess.DEVNULL,"
                "stderr=subprocess.DEVNULL); "
                "print(p.pid,flush=True); sys.stdin.read()"
            )
            owner = subprocess.Popen(
                [sys.executable, "-c", program, *command],
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                text=True,
            )
            assert owner.stdout is not None
            pid = int(owner.stdout.readline())

            def alive():
                try:
                    return Path(f"/proc/{pid}/stat").read_text().rsplit(")", 1)[1].split()[0] != "Z"
                except FileNotFoundError:
                    return False

            try:
                deadline = time.monotonic() + 10
                while not any(
                    "->" in line and str(pid) in line.split()
                    for line in Path("/proc/locks").read_text().splitlines()
                ):
                    assert alive(), "native request exited before reaching the held GC lock"
                    assert time.monotonic() < deadline, "native request never waited on the store"
                    time.sleep(0.01)
                owner.kill()
                owner.wait(timeout=5)
                deadline = time.monotonic() + 5
                while alive() and time.monotonic() < deadline:
                    time.sleep(0.01)
                assert not alive(), "orphaned native request retained its resources"
            finally:
                if owner.poll() is None:
                    owner.kill()
                    owner.wait(timeout=5)
                if alive():
                    os.kill(pid, 9)  # Exact fixture child, never a process-name selector.
                owner.stdout.close()
                if owner.stdin is not None:
                    owner.stdin.close()
                fcntl.flock(lock, fcntl.LOCK_UN)
