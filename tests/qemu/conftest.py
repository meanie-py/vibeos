"""Pytest fixtures for QEMU-based ReactOS kernel tests."""

from __future__ import annotations

from collections.abc import Iterator

import pytest

from .vm import QemuVM, get_arch_config


def pytest_addoption(parser: pytest.Parser) -> None:
    parser.addoption(
        "--arch",
        default="arm64",
        choices=["arm64", "arm64-test", "i386", "amd64"],
        help="Target architecture (default: arm64)",
    )
    parser.addoption(
        "--test-kernel",
        action="store_true",
        default=False,
        help="Boot with /TESTKERNEL (overrides --arch to arm64-test)",
    )


@pytest.fixture(scope="session")
def arch(request: pytest.FixtureRequest) -> str:
    if request.config.getoption("--test-kernel"):
        return "arm64-test"
    return request.config.getoption("--arch")


@pytest.fixture(scope="session")
def vm(arch: str, tmp_path_factory: pytest.TempPathFactory) -> Iterator[QemuVM]:
    """Boot QEMU once per test session, save 'ready' snapshot."""
    config = get_arch_config(arch)
    work_dir = tmp_path_factory.mktemp(f"qemu-{arch}")

    vm = QemuVM(config, work_dir)
    vm.start()
    boot_output = vm.wait_for_boot()
    print(f"\n[pytest] Boot complete ({len(boot_output)} bytes of serial output)")
    vm.save_snapshot("ready")

    yield vm
    vm.stop()


@pytest.fixture(autouse=True)
def restore_vm(vm: QemuVM) -> Iterator[None]:
    """Restore to 'ready' snapshot before each test."""
    vm.restore_snapshot("ready")
    yield
