"""Pytest fixtures for QEMU-based ReactOS kernel tests."""

from __future__ import annotations

import shutil
import subprocess
import sys
from collections.abc import Iterator
from pathlib import Path

import pytest

from .vm import QemuVM, get_arch_config, PROJECT_DIR

# Add tools/ to path for scan_ud2, lint_kernel
sys.path.insert(0, str(PROJECT_DIR / "tools"))


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
    parser.addoption(
        "--skip-clean-build",
        action="store_true",
        default=False,
        help="Skip nuclear Rust rebuild (faster iteration, may mask bugs)",
    )


@pytest.fixture(scope="session")
def arch(request: pytest.FixtureRequest) -> str:
    if request.config.getoption("--test-kernel"):
        return "arm64-test"
    return request.config.getoption("--arch")


def _ensure_clean_rust_build(arch: str) -> None:
    """Force a clean Rust rebuild to prevent stale artifact false passes.

    Cargo's incremental compilation cache can mask regressions — a change that
    breaks boot may pass smoke tests if stale object files are linked.
    """
    build_dir = PROJECT_DIR / f"build-{arch}"
    target_dir = build_dir / "ntoskrnl" / "rust-kernel-target"
    lib_file = build_dir / "ntoskrnl" / "libkernel_rust.a"

    if target_dir.exists():
        shutil.rmtree(target_dir, ignore_errors=True)
        print(f"\n[pytest] Removed {target_dir}")
    if lib_file.exists():
        lib_file.unlink()
        print(f"[pytest] Removed {lib_file}")

    # Rebuild
    build_target = "livecd" if arch in ("i386", "amd64") else "ntoskrnl"
    ninja = subprocess.run(
        ["ninja", build_target],
        cwd=build_dir, capture_output=True, text=True, timeout=600,
    )
    if ninja.returncode != 0:
        pytest.fail(f"Rust rebuild failed:\n{ninja.stderr[-2000:]}")
    print(f"[pytest] Clean Rust rebuild complete for {arch}")


def _run_post_build_checks(arch: str) -> list[str]:
    """Run static analysis on the built binary. Returns list of warnings."""
    warnings = []
    build_dir = PROJECT_DIR / f"build-{arch}"
    ntoskrnl = build_dir / "ntoskrnl" / "ntoskrnl.exe"
    if not ntoskrnl.exists():
        return warnings

    # ud2 scan
    try:
        from scan_ud2 import scan_ud2, find_objdump
        objdump = find_objdump(arch)
        if objdump:
            findings = scan_ud2(ntoskrnl, objdump)
            unexpected = [f for f in findings if not f["expected"]]
            if unexpected:
                funcs = ", ".join(f["function"] for f in unexpected[:5])
                more = f" (+{len(unexpected)-5} more)" if len(unexpected) > 5 else ""
                warnings.append(
                    f"ud2 scanner: {len(unexpected)} LLVM miscompilations detected: {funcs}{more}"
                )
                print(f"\n[pytest] WARNING: {warnings[-1]}")
    except Exception as e:
        warnings.append(f"ud2 scan failed: {e}")

    return warnings


@pytest.fixture(scope="session")
def vm(arch: str, tmp_path_factory: pytest.TempPathFactory, request: pytest.FixtureRequest) -> Iterator[QemuVM]:
    """Boot QEMU once per test session, save 'ready' snapshot."""
    if not request.config.getoption("--skip-clean-build"):
        _ensure_clean_rust_build(arch)
    _run_post_build_checks(arch)

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
