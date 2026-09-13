"""Real process/thread exclusion and crash recovery for QEMU port handshakes."""

import itertools
import json
import os
import subprocess
import sys
import threading
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import pytest

from ..portlease import launch_lock


@pytest.fixture(autouse=True)
def isolated_lock(tmp_path, monkeypatch):
    monkeypatch.setenv("VIBEOS_HOSTLOCK_DIR", str(tmp_path))


def child(code):
    return subprocess.Popen(
        [sys.executable, "-c", "from tests.qemu.portlease import launch_lock\n" + code],
        cwd=Path(__file__).resolve().parents[3],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )


def test_competing_processes_do_not_overlap():
    code = (
        "import json, time\n"
        "with launch_lock(timeout=10):\n"
        " start = time.monotonic_ns()\n"
        " time.sleep(.025)\n"
        " end = time.monotonic_ns()\n"
        "print(json.dumps([start, end]))\n"
    )
    processes = [child(code) for _ in range(6)]
    intervals = []
    try:
        for proc in processes:
            out, err = proc.communicate(timeout=15)
            assert proc.returncode == 0, err
            intervals.append(json.loads(out))
        intervals.sort()
        assert all(left[1] <= right[0] for left, right in itertools.pairwise(intervals))
    finally:
        for proc in processes:
            if proc.poll() is None:
                proc.kill()
            proc.wait()


def test_threads_are_excluded_and_timeout_names_holder():
    barrier = threading.Barrier(2)

    def compete():
        barrier.wait(timeout=5)
        with pytest.raises(TimeoutError, match=f'"pid": {os.getpid()}'), launch_lock(timeout=0.05):
            pytest.fail("entered another thread's critical section")

    with ThreadPoolExecutor(max_workers=1) as pool, launch_lock():
        future = pool.submit(compete)
        barrier.wait(timeout=5)
        future.result(timeout=5)
    with launch_lock(timeout=0):
        pass


def test_killed_owner_releases_lock(tmp_path):
    proc = child("with launch_lock():\n print('held', flush=True)\n input()\n")
    try:
        assert proc.stdout is not None
        assert proc.stdout.readline() == "held\n"
        with pytest.raises(TimeoutError, match="port handshake timed out"), launch_lock(timeout=0):
            pytest.fail("entered live owner's critical section")
        proc.kill()
        proc.wait(timeout=5)
        with launch_lock(timeout=1):
            assert (tmp_path / "qemu-port-handshake.lock").is_file()
    finally:
        if proc.poll() is None:
            proc.kill()
        proc.communicate(timeout=5)


def test_exception_releases_lock():
    with pytest.raises(RuntimeError, match="fixture"), launch_lock():
        raise RuntimeError("fixture")
    with launch_lock(timeout=0):
        pass


@pytest.mark.parametrize("timeout", [-1, float("inf"), float("nan")])
def test_invalid_timeout_fails(timeout):
    with pytest.raises(ValueError, match="finite and nonnegative"), launch_lock(timeout=timeout):
        pytest.fail("accepted invalid timeout")


def test_one_launch_never_assigns_same_port_twice(monkeypatch):
    from tests.qemu import vm

    candidates = iter([45000, 45000, 45001])
    monkeypatch.setattr(vm, "_selected_launch_ports", set())
    assert vm._unique_launch_port(lambda: next(candidates)) == 45000
    assert vm._unique_launch_port(lambda: next(candidates)) == 45001


def test_duplicate_port_allocator_fails_loudly(monkeypatch):
    from tests.qemu import vm

    monkeypatch.setattr(vm, "_selected_launch_ports", {45000})
    with pytest.raises(RuntimeError, match="already assigned"):
        vm._unique_launch_port(lambda: 45000)
