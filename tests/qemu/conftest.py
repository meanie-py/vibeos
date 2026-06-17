"""Pytest fixtures for QEMU-based ReactOS kernel tests."""

from __future__ import annotations

import shutil
import subprocess
import sys
from collections.abc import Iterator
from pathlib import Path

import pytest

from .vm import QemuVM, get_arch_config, reap_orphan_qemus, PROJECT_DIR

# Add tools/ to path for scan_ud2, lint_kernel
sys.path.insert(0, str(PROJECT_DIR / "tools"))


def pytest_addoption(parser: pytest.Parser) -> None:
    parser.addoption(
        "--arch",
        default="arm64",
        # BIOS boot is gone: x86 is UEFI-only (i386-uefi / amd64-uefi). The
        # legacy "i386"/"amd64" BIOS arches were dropped with the BIOS prune.
        choices=["arm64", "arm64-test", "i386-uefi", "amd64-uefi", "amd64-uefi-net"],
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


def _base_arch(arch: str) -> str:
    """Map a smoke variant to its build directory arch. Variants like
    'i386-uefi', 'arm64-test' and 'amd64-uefi-net' build from the base
    'build-i386'/'build-arm64'/'build-amd64' trees (same binaries, different
    boot media/firmware/devices). Strip '-net' BEFORE '-uefi' so
    'amd64-uefi-net' resolves to 'amd64', not 'amd64-net'."""
    return arch.replace("-net", "").replace("-uefi", "").replace("-test", "")


def _ensure_clean_rust_build(arch: str) -> None:
    """Force a clean Rust rebuild to prevent stale artifact false passes.

    Cargo's incremental compilation cache can mask regressions — a change that
    breaks boot may pass smoke tests if stale object files are linked.
    """
    build_dir = PROJECT_DIR / f"build-{_base_arch(arch)}"
    target_dir = build_dir / "ntoskrnl" / "rust-kernel-target"
    lib_file = build_dir / "ntoskrnl" / "libkernel_rust.a"

    if target_dir.exists():
        shutil.rmtree(target_dir, ignore_errors=True)
        print(f"\n[pytest] Removed {target_dir}")
    if lib_file.exists():
        lib_file.unlink()
        print(f"[pytest] Removed {lib_file}")

    # Rebuild
    build_target = "livecd" if _base_arch(arch) in ("i386", "amd64") else "ntoskrnl"
    ninja = subprocess.run(
        ["ninja", build_target],
        cwd=build_dir, capture_output=True, text=True, timeout=600,
    )
    if ninja.returncode != 0:
        pytest.fail(f"Rust rebuild failed:\n{ninja.stderr[-2000:]}")
    print(f"[pytest] Clean Rust rebuild complete for {arch}")

    # arm64 boots from qemu/boot.img ⊃ ramdisk.img, NOT build-{arch}/livecd.iso.
    # `ninja ntoskrnl` rebuilds the kernel but nothing re-stages it into the
    # ramdisk — so without this, arm64 smoke can boot STALE binaries (the
    # exact stale-artifact false-pass the clean rebuild exists to prevent on
    # x86, where `ninja livecd` makes the ISO the boot media). Rebuild the
    # arm64-specific drivers too, then fast-refresh the ramdisk. This is the
    # structural mirror of `ninja livecd`.
    if arch in ("arm64", "arm64-test"):
        subprocess.run(  # best-effort: targets absent on some branches
            ["ninja", "arm64input", "arm64vid"],
            cwd=build_dir, capture_output=True, text=True, timeout=600,
        )
        refresh = subprocess.run(
            ["bash", str(PROJECT_DIR / "qemu" / "mkramdisk.sh"), "--refresh"],
            capture_output=True, text=True, timeout=600,
        )
        if refresh.returncode != 0:
            pytest.fail(
                "arm64 ramdisk --refresh failed (smoke would boot stale "
                f"media):\n{refresh.stdout[-1500:]}\n{refresh.stderr[-1500:]}")
        print("[pytest] arm64 ramdisk refreshed (boot.img re-staged)")


def _run_post_build_checks(arch: str) -> list[str]:
    """Run static analysis on the built binary. Returns list of warnings."""
    warnings = []
    build_dir = PROJECT_DIR / f"build-{_base_arch(arch)}"
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
    # Reap ONLY orphan QEMUs whose launcher process is dead (crashed/timed-out
    # prior runs). A QEMU with a live launcher — possibly another agent's —
    # is never touched. Safe by construction; see vm.reap_orphan_qemus.
    for line in reap_orphan_qemus():
        print(f"[pytest] {line}")
    # Also sweep kernel-debug-toolkit orphans (qemu/start.sh manifests with
    # a dead owner_pid). The Python reaper above only knows the smoke
    # harness's /tmp/vibeos-qemu-locks/ lockfiles; reap.sh covers both halves.
    reap_sh = PROJECT_DIR / "qemu" / "reap.sh"
    if reap_sh.is_file():
        result = subprocess.run(
            [str(reap_sh)], capture_output=True, text=True, check=False
        )
        for line in result.stderr.splitlines():
            if line.startswith("[reap] killed") or line.startswith("[reap] FAILED"):
                print(f"[pytest] {line}")
    # Surface a missing screenshot dependency up front (don't fail — boot is
    # still verifiable via serial markers — but tell the operator how to fix it,
    # since screenshots silently degrade to raw .ppm without it).
    try:
        import PIL  # noqa: F401
    except ImportError:
        print("[pytest] WARNING: Pillow not installed — screenshots will save "
              "as raw .ppm. Fix: pip install -e '.[qemu]'")

    if not request.config.getoption("--skip-clean-build"):
        _ensure_clean_rust_build(arch)
    _run_post_build_checks(arch)

    config = get_arch_config(arch)
    work_dir = tmp_path_factory.mktemp(f"qemu-{arch}")

    vm = QemuVM(config, work_dir)
    vm.start()
    # Tell watchers where to follow boot progress live — a cold arm64 TCG boot
    # is silent for minutes, so this is how an agent distinguishes "working"
    # from "wedged" without restarting the run.
    print(f"\n[pytest] Live serial: tail -f {vm.live_log_path}")
    try:
        boot_output = vm.wait_for_boot()
    except (TimeoutError, ConnectionError, RuntimeError) as e:
        # Boot failed — capture screenshot + debugcon for diagnosis
        screenshot_msg = ""
        try:
            ppm = work_dir / "fail_screen.ppm"
            vm.screendump(ppm)
            png = PROJECT_DIR / "qemu" / f"{arch}_smoke.png"
            # Convert via PIL, NOT ImageMagick `convert`: IM 6.x writes a 1-bit
            # colormap PNG when the source PPM is mostly one color (e.g. a bare
            # blue desktop or a solid BSOD), which renders as solid black and
            # hides the very failure state we're trying to capture. Mirrors
            # test_smoke.py::test_screenshot.
            try:
                from PIL import Image
                Image.open(ppm).save(png, "PNG")
            except ImportError:
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
