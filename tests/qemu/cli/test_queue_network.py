"""A DNS blip is not the remote's answer: the queue's git retries it.

The stderr below is the conductor's verbatim death on 2026-10-15, which twice left the
merge queue holding a submission with no conductor.
"""

import subprocess

from tests.qemu.cli import queue

DNS = (
    "ssh: Could not resolve hostname github.com: Temporary failure in name resolution\n"
    "fatal: Could not read from remote repository."
)
REJECTED = " ! [rejected]        HEAD -> master (non-fast-forward)"


def _fake_run(results):
    calls = []

    def run(argv, **_kw):
        calls.append(argv)
        rc, err = results[min(len(calls), len(results)) - 1]
        return subprocess.CompletedProcess(argv, rc, stdout="", stderr=err)

    return run, calls


def test_a_transient_network_failure_is_retried(monkeypatch, tmp_path):
    run, calls = _fake_run([(128, DNS), (128, DNS), (0, "")])
    monkeypatch.setattr(queue.subprocess, "run", run)
    monkeypatch.setattr(queue, "NETWORK_RETRY_DELAYS", (0, 0, 0))
    assert queue.git(tmp_path, "fetch", "origin") == ""
    assert len(calls) == 3


def test_what_the_remote_said_is_not_retried(monkeypatch, tmp_path):
    run, calls = _fake_run([(1, REJECTED), (0, "")])
    monkeypatch.setattr(queue.subprocess, "run", run)
    monkeypatch.setattr(queue, "NETWORK_RETRY_DELAYS", (0, 0, 0))
    assert queue.run_git(tmp_path, ("push", "origin", "HEAD:master"), 60).returncode == 1
    assert len(calls) == 1


def test_a_local_command_is_never_retried(monkeypatch, tmp_path):
    run, calls = _fake_run([(128, DNS), (0, "")])
    monkeypatch.setattr(queue.subprocess, "run", run)
    monkeypatch.setattr(queue, "NETWORK_RETRY_DELAYS", (0, 0, 0))
    assert queue.run_git(tmp_path, ("merge", "x"), 60).returncode == 128
    assert len(calls) == 1


def test_the_retries_run_out(monkeypatch, tmp_path):
    run, calls = _fake_run([(128, DNS)])
    monkeypatch.setattr(queue.subprocess, "run", run)
    monkeypatch.setattr(queue, "NETWORK_RETRY_DELAYS", (0, 0))
    assert queue.run_git(tmp_path, ("fetch", "origin"), 60).returncode == 128
    assert len(calls) == 3
