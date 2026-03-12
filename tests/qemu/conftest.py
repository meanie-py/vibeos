"""Pytest fixtures for QEMU-based ReactOS kernel tests."""

from __future__ import annotations

from collections.abc import Iterator

import pytest

from .vm import QemuVM, get_arch_config, PROJECT_DIR


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
    try:
        boot_output = vm.wait_for_boot()
    except (TimeoutError, ConnectionError, RuntimeError) as e:
        # Boot failed — capture screenshot + debugcon for diagnosis
        screenshot_msg = ""
        try:
            ppm = work_dir / "fail_screen.ppm"
            vm.screendump(ppm)
            png = PROJECT_DIR / "qemu" / f"{arch}_smoke.png"
            import subprocess as _sp
            try:
                _sp.run(["convert", str(ppm), str(png)],
                        check=True, capture_output=True, timeout=10)
            except (FileNotFoundError, _sp.CalledProcessError):
                import shutil as _sh
                png = PROJECT_DIR / "qemu" / f"{arch}_smoke.ppm"
                _sh.copy2(ppm, png)
            screenshot_msg = f"Screenshot: {png}"
        except Exception:
            screenshot_msg = "Screenshot: failed to capture"

        debugcon = vm.debugcon_report() or "no debugcon output"
        serial_len = len(vm._boot_output)
        vm.stop()
        pytest.fail(
            f"Boot failed: {e}\n"
            f"Serial output: {serial_len} bytes\n"
            f"{screenshot_msg}\n"
            f"\n{debugcon}"
        )
    print(f"\n[pytest] Boot complete ({len(boot_output)} bytes of serial output)")

    vm.save_snapshot("ready")

    yield vm
    vm.stop()


@pytest.fixture(autouse=True)
def restore_vm(vm: QemuVM) -> Iterator[None]:
    """Restore to 'ready' snapshot before each test."""
    try:
        vm.restore_snapshot("ready")
    except (RuntimeError, BrokenPipeError, ConnectionError, OSError):
        pass
    yield
