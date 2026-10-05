"""Kernel-mode test runner: sends test commands over serial, reads results.

Requires --test-kernel flag (boots with /TESTKERNEL, enters dispatch loop).
The vm fixture boots to "TESTKERNEL READY" and saves a snapshot.
Each test restores that snapshot, sends RUN <name>, and checks for PASS/FAIL.
"""

from __future__ import annotations

import pytest

from tests.qemu.vm import QemuVM

# Tests compiled into ntoskrnl when REACTOS_TEST_KERNEL=ON
KERNEL_TESTS = [
    "MmPool",
    "ObHandle",
    "KeEvent",
    "Dpc",
    "Timer",
    "Vmem",
    "Registry",
    "WorkItem",
    "Irql",
    "Thread",
    "Mutex",
    "Semaphore",
    "MultiWait",
    "ObRef",
    "RtlStr",
    "SList",
    "Mdl",
    "FileIo",
    "Lookaside",
    "FastMutex",
    "Bitmap",
    "Capture",
    "MemOps",
    "SysTime",
    "ProcInfo",
    "ObjDir",
    "SymLink",
    "InterlockedList",
]


def run_kernel_test(vm: QemuVM, name: str, timeout: float = 30.0) -> str:
    """Send RUN <name> to the kernel dispatch loop and wait for result.

    Returns the full output captured. Raises AssertionError on failure/timeout.
    """
    # Send the command
    vm.serial.send(f"RUN {name}\n".encode("ascii"))

    # Read until we see [TEST PASS] or [TEST FAIL]
    pass_marker = f"[TEST PASS] {name}"
    fail_marker = f"[TEST FAIL] {name}"

    buf = ""
    try:
        # Read line by line until we get a result
        buf = vm.serial.read_until_idle(
            idle_seconds=2.0,
            timeout=timeout,
            min_bytes=len(f"[TEST START] {name}"),
        )
    except TimeoutError:
        pytest.fail(f"Kernel test {name!r} timed out after {timeout}s. Captured output:\n{buf}")

    if pass_marker in buf:
        return buf
    if fail_marker in buf:
        pytest.fail(f"Kernel test {name!r} failed. Output:\n{buf}")

    pytest.fail(f"Kernel test {name!r}: no result marker found. Output:\n{buf}")


def test_kernel_list(vm: QemuVM) -> None:
    """Verify LIST command returns all registered tests."""
    vm.serial.send(b"LIST\n")
    output = vm.serial.read_until_idle(
        idle_seconds=2.0,
        timeout=10.0,
        min_bytes=10,
    )
    assert "[TEST LIST]" in output, f"LIST response missing marker. Got: {output!r}"
    for name in KERNEL_TESTS:
        assert name in output, f"Test {name!r} not in LIST response: {output!r}"


@pytest.mark.parametrize("test_name", KERNEL_TESTS)
def test_kernel(vm: QemuVM, test_name: str) -> None:
    """Run a single kernel-mode test via serial dispatch."""
    output = run_kernel_test(vm, test_name)
    # If we get here, the test passed (run_kernel_test asserts on failure)
    assert f"[TEST PASS] {test_name}" in output
