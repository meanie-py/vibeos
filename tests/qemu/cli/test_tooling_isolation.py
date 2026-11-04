"""Concurrent tooling suites own separate pytest temp roots."""

import subprocess
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from threading import Barrier

from tests.qemu.cli import tooling


def test_parallel_pytest_temp_roots_are_disjoint(tmp_path, monkeypatch):
    barrier = Barrier(2)
    roots = []

    def child(cmd, **kwargs):
        base = Path(cmd[cmd.index("--basetemp") + 1])
        assert base.parent.is_dir()
        assert not str(base).startswith("/tmp/vibeos")
        roots.append(base)
        barrier.wait(timeout=10)
        return subprocess.CompletedProcess(cmd, 0, "passed", "")

    monkeypatch.setattr(tooling.subprocess, "run", child)
    with ThreadPoolExecutor(max_workers=2) as pool:
        jobs = [
            pool.submit(tooling._run, f"pytest:{i}", ["python", "-m", "pytest", "tests"], tmp_path)
            for i in range(2)
        ]
        assert all(job.result()[1] for job in jobs)
    assert len(set(roots)) == 2
    assert all(not path.parent.exists() for path in roots)


def test_non_pytest_command_is_unchanged(tmp_path, monkeypatch):
    command = ["cargo", "test"]

    def child(cmd, **kwargs):
        assert cmd == command
        return subprocess.CompletedProcess(cmd, 0, "passed", "")

    monkeypatch.setattr(tooling.subprocess, "run", child)
    assert tooling._run("rust:vibeos-build", command, tmp_path)[1]
