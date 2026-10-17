"""Pytest fixtures for QEMU-based ReactOS kernel tests."""

from __future__ import annotations

import os
import shutil
import subprocess
import sys
from collections.abc import Iterator

import pytest

from .cli import config as cli_config
from .vm import PROJECT_DIR, QemuVM, get_arch_config, launch_with_retry, reap_orphan_qemus

# Add tools/ to path for shared tooling helpers
sys.path.insert(0, str(PROJECT_DIR / "tools"))


def pytest_addoption(parser: pytest.Parser) -> None:
    parser.addoption(
        "--arch",
        default="arm64",
        # BIOS boot is gone: x86 is UEFI-only (i386-uefi / amd64-uefi). The
        # legacy "i386"/"amd64" BIOS arches were dropped with the BIOS prune.
        choices=[
            "arm64",
            "arm64-test",
            "arm64-net",
            "i386-uefi",
            "i386-uefi-net",
            "amd64-uefi",
            "amd64-uefi-net",
            "amd64-bios",
        ],
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
    parser.addoption(
        "--payload",
        default="",
        help="Comma-separated payload names to gate (tests/qemu/test_payload.py; "
        "media must be built with VIBEOS_PAYLOAD=<names>). "
        "Design doc: docs/engineering/payload-harness-design.md",
    )
    parser.addoption(
        "--doom",
        action="store_true",
        default=False,
        help="Legacy alias for --payload doom",
    )


@pytest.fixture(scope="session")
def arch(request: pytest.FixtureRequest) -> str:
    if request.config.getoption("--test-kernel"):
        return "arm64-test"
    arch = request.config.getoption("--arch")
    assert isinstance(arch, str), "--arch is required"
    return arch


# base-arch mapping (smoke variant → build tree) lives in archcfg — the single
# build-side source of truth. conftest used to keep a private copy that drifted
# from archcfg's (archcfg didn't strip '-net'); reconciled + deduped 2026-07-05.
from .cli.archcfg import base_arch as _base_arch  # noqa: E402


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

    # Rebuild. Canonical build dirs (cli.canon) bake /tmp/vibeos paths, so ninja
    # must run under the canon-run bind mount; legacy dirs run ninja directly.
    from .cli import canon

    def _wrap(cmd: list[str]) -> list[str]:
        return canon.wrap(PROJECT_DIR, cmd) if canon.is_canonical(build_dir) else cmd

    build_target = "livecd" if _base_arch(arch) in ("i386", "amd64") else "ntoskrnl"
    ninja = subprocess.run(
        _wrap(["ninja", build_target]),
        cwd=build_dir,
        capture_output=True,
        text=True,
        timeout=600,
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
            _wrap(["ninja", "arm64input", "sysfb"]),
            cwd=build_dir,
            capture_output=True,
            text=True,
            timeout=600,
        )
        refresh = subprocess.run(
            ["bash", str(PROJECT_DIR / "qemu" / "mkramdisk.sh"), "--refresh"],
            capture_output=True,
            text=True,
            timeout=600,
        )
        if refresh.returncode != 0:
            pytest.fail(
                "arm64 ramdisk --refresh failed (smoke would boot stale "
                f"media):\n{refresh.stdout[-1500:]}\n{refresh.stderr[-1500:]}"
            )
        print("[pytest] arm64 ramdisk refreshed (boot.img re-staged)")


def _run_post_build_checks(arch: str) -> list[str]:
    """Run static analysis on the built binary. Returns list of warnings."""
    warnings = []
    build_dir = PROJECT_DIR / f"build-{_base_arch(arch)}"
    ntoskrnl = build_dir / "ntoskrnl" / "ntoskrnl.exe"
    if not ntoskrnl.exists():
        return warnings

    from tests.qemu.cli import rustlints

    result = rustlints.run_binary(
        PROJECT_DIR,
        _base_arch(arch),
        ["scan_ud2"],
        lambda cmd: subprocess.run(cmd, cwd=PROJECT_DIR, capture_output=True, text=True),
    )["scan_ud2"]
    if result.returncode:
        warnings.append(f"ud2 scan failed: {result.stdout}{result.stderr}")
        print(f"\n[pytest] WARNING: {warnings[-1]}")

    return warnings


@pytest.fixture(scope="session")
def vm(
    arch: str, tmp_path_factory: pytest.TempPathFactory, request: pytest.FixtureRequest
) -> Iterator[QemuVM]:
    """Boot QEMU once per test session, save 'ready' snapshot."""
    # Reap ONLY orphan QEMUs whose owner is dead (or that are past their lifetime cap):
    # the smoke harness's lock files AND the kernel-debug toolkit's run manifests. A QEMU
    # with a live owner -- possibly another agent's -- is never touched. In-process; see
    # vm.reap_orphan_qemus for why this is no longer a qemu/reap.sh shell-out.
    for line in reap_orphan_qemus():
        print(f"[pytest] [reap] {line}")
    # Surface a missing screenshot dependency up front (don't fail — boot is
    # still verifiable via serial markers — but tell the operator how to fix it,
    # since screenshots silently degrade to raw .ppm without it).
    try:
        import PIL  # noqa: F401
    except ImportError:
        print(
            "[pytest] WARNING: Pillow not installed — screenshots will save "
            "as raw .ppm. Fix: pip install -e '.[qemu]'"
        )

    if not request.config.getoption("--skip-clean-build"):
        _ensure_clean_rust_build(arch)
    _run_post_build_checks(arch)

    config = get_arch_config(arch)
    work_dir = tmp_path_factory.mktemp(f"qemu-{arch}")

    # Honor the same layered launch-backend config the cli smoke path reads
    # (.vibeos-cli.local.json / env), so EVERY pytest QEMU path — the net suite
    # folded into smoke, test_snapshot_apps, a direct test_smoke run — uses the
    # selected backend. Without this the net stage spawned a plain pytest that
    # always launched native KVM, so a WSL2-no-KVM user on the windows-host
    # backend got a working boot stage but a net stage that failed on KVM.
    cli_cfg = cli_config.load(PROJECT_DIR)
    backend = cli_cfg.get("qemu.backend", "native")
    # Through the shared retry, like smoke: a lost serial socket is the HOST, it resolves
    # on a fresh VM (new port), and without this it was graded a lost boot of the OS.
    # Each retry is printed, and a launch that fails every attempt raises as before.
    started, _attempts, launch_exc = launch_with_retry(
        lambda: QemuVM(config, work_dir, backend=backend, windows=cli_cfg.get("qemu.windows", {})),
        {},
        lambda msg: print(f"[pytest] {msg}"),
        "launch",
    )
    if launch_exc is not None:
        raise launch_exc
    assert started is not None
    vm = started
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
            f"Boot failed: {e}\nSerial output: {serial_len} bytes\n{screenshot_msg}\n\n{debugcon}"
        )
    print(f"\n[pytest] Boot complete ({len(boot_output)} bytes of serial output)")

    if os.environ.get("VIBEOS_QEMU_RR", "").strip():
        # Record/replay: a trace is ONE continuous execution. The per-test
        # `loadvm ready` below would rewind the whole machine at a moment the
        # harness picks by host time, which no log can reproduce — the replay's
        # loadvm lands at a different instruction than the recording's and swaps
        # the entire state. That was the arm64 kmtest divergence after the network
        # was recorded: `rr diverge` matched the recording through the desktop and
        # then met a STOP no break had asked for. Tests run sequentially in one VM,
        # the same isolation WHPX gets.
        print("[pytest] record/replay: no `ready` snapshot; tests share one live VM")
        yield vm
        vm.stop()
        return

    if vm._nvme_enabled():
        # QEMU's nvme device registers a migration blocker, so `savevm` fails with
        # "State blocked by non-migratable device". The NVMe rig's tests are written
        # to run in order in one live VM (read-only checks first, then the write).
        print(
            "[pytest] NVMe rig: no `ready` snapshot (nvme blocks savevm); tests share one live VM"
        )
        yield vm
        vm.stop()
        return

    try:
        vm.save_snapshot("ready")
    except RuntimeError as e:
        # WHPX cannot savevm — the accelerator registers migration blockers
        # ("State blocked due to non-migratable CPUID feature support, dirty
        # memory tracking support, and XSAVE/XRSTOR support"). Degrade LOUDLY
        # to no-per-test-snapshot isolation (restore_vm below already no-ops
        # when the snapshot doesn't exist) instead of erroring the whole
        # suite. Only for this known accelerator limitation; anything else
        # still hard-fails. See docs/engineering/windows-host-qemu-whpx-spike.md.
        if "State blocked" in str(e) and getattr(vm, "_win_accel", "") == "whpx":
            print(
                "[pytest] WARNING: savevm unsupported under WHPX (migration "
                "blockers) — running WITHOUT per-test snapshot isolation; "
                "tests share one live VM sequentially"
            )
        else:
            raise

    yield vm
    vm.stop()


@pytest.fixture(autouse=True)
def restore_vm(vm: QemuVM) -> Iterator[None]:
    """Restore to 'ready' snapshot before each test."""
    if os.environ.get("VIBEOS_QEMU_RR", "").strip() or vm._nvme_enabled():
        # No snapshot under record/replay or the NVMe rig (see the vm fixture), and
        # HMP loadvm stops and restarts the VM before it finds that out.
        yield
        return
    try:
        vm.restore_snapshot("ready")
    except (RuntimeError, BrokenPipeError, ConnectionError, OSError):
        pass
    yield
