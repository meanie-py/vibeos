"""Smoke tests: verify QEMU boots to desktop, serial works, QMP works, snapshots work."""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

import pytest

from tests.qemu.vm import QemuVM, process_cpu_pct, PROJECT_DIR

# Add tools/ to path for scan_ud2
sys.path.insert(0, str(PROJECT_DIR / "tools"))


def _screenshot_path(arch: str) -> Path:
    """Return the path where the smoke screenshot will be saved."""
    return PROJECT_DIR / "qemu" / f"{arch}_smoke.png"


def _debugcon_hint(vm: QemuVM) -> str:
    """Get debugcon triage report + screenshot path for assertion failure messages."""
    parts = []
    report = vm.debugcon_report()
    if report:
        parts.append(report)
    parts.append(f"Screenshot: {_screenshot_path(vm.config.arch)}")
    return "\n\n" + "\n".join(parts)


def _symbolicate_bugcheck(serial_output: str, arch: str) -> str:
    """Extract BUGCHECK info from serial output and symbolicate crash address."""
    # Match: *** BUGCHECK: 0x0000001E (0xC000001D, 0x804C4213, 0xF7BBB8C8, 0x00000000)
    # Or:    *** STOP: 0x0000001E (0xC000001D,0x804C4213,0xF7BBB8C8,0x00000000)
    match = re.search(
        r'\*\*\*\s+(?:BUGCHECK|STOP):\s+(0x[0-9A-Fa-f]+)\s*'
        r'\((0x[0-9A-Fa-f]+)[,\s]+(0x[0-9A-Fa-f]+)[,\s]+(0x[0-9A-Fa-f]+)[,\s]+(0x[0-9A-Fa-f]+)\)',
        serial_output,
    )
    if not match:
        return ""

    bugcode = match.group(1)
    p1, p2, p3, p4 = match.group(2), match.group(3), match.group(4), match.group(5)

    # Known bugcheck codes
    bugcheck_names = {
        "0x0000001E": "KMODE_EXCEPTION_NOT_HANDLED",
        "0x0000007E": "SYSTEM_THREAD_EXCEPTION_NOT_HANDLED",
        "0x0000000A": "IRQL_NOT_LESS_OR_EQUAL",
        "0x00000050": "PAGE_FAULT_IN_NONPAGED_AREA",
        "0x0000007F": "UNEXPECTED_KERNEL_MODE_TRAP",
        "0x0000006A": "LPC_INITIALIZATION_FAILED",
    }
    exception_codes = {
        "0xC000001D": "STATUS_ILLEGAL_INSTRUCTION (ud2 — likely LLVM miscompilation)",
        "0xC0000005": "STATUS_ACCESS_VIOLATION",
        "0xC000001E": "STATUS_INVALID_LOCK_SEQUENCE",
    }

    name = bugcheck_names.get(bugcode.upper(), "UNKNOWN")
    parts = [f"\n  BUGCHECK {bugcode} ({name})"]
    parts.append(f"    P1={p1} P2={p2} P3={p3} P4={p4}")

    exc_name = exception_codes.get(p1.upper(), "")
    if exc_name:
        parts.append(f"    Exception: {exc_name}")

    # Try to symbolicate crash address (P2 for 0x1E)
    crash_addr = int(p2, 16) if bugcode.upper() == "0X0000001E" else None
    if crash_addr:
        # PE ImageBase: i386=0x80400000, amd64=varies
        image_bases = {"i386": 0x80400000, "amd64": 0xFFFFF80000000000}
        base = image_bases.get(arch, 0)
        if crash_addr > base:
            rva = crash_addr - base
            parts.append(f"    Crash RVA: 0x{rva:X} (base 0x{base:X})")

            # Try objdump to find function
            ntoskrnl = PROJECT_DIR / f"build-{arch}" / "ntoskrnl" / "ntoskrnl.exe"
            if ntoskrnl.exists():
                try:
                    from scan_ud2 import find_objdump
                    objdump = find_objdump(arch)
                    if objdump:
                        result = subprocess.run(
                            [objdump, "-d", str(ntoskrnl)],
                            capture_output=True, text=True, timeout=120,
                        )
                        # Find which function contains the crash address
                        last_func = None
                        for line in result.stdout.splitlines():
                            func_match = re.match(r'^([0-9a-f]+)\s+<(.+)>:', line)
                            if func_match:
                                func_addr = int(func_match.group(1), 16)
                                if func_addr <= rva:
                                    last_func = func_match.group(2)
                                else:
                                    break
                        if last_func:
                            parts.append(f"    Function: {last_func}")
                except Exception:
                    pass

    return "\n".join(parts)


def test_boot_and_serial(vm: QemuVM, arch: str) -> None:
    """Verify QEMU boots and produces serial output without crashing."""
    hint = _debugcon_hint(vm)
    assert vm._boot_output, f"No boot output captured{hint}"
    threshold = vm.config.min_boot_bytes
    assert len(vm._boot_output) >= threshold, (
        f"Too little boot output: {len(vm._boot_output)} bytes "
        f"(expected >= {threshold}){hint}"
    )
    # Detect crashes in serial output (catches BSODs that idle detection misses)
    for crash_sig in ["*** BUGCHECK:", "*** STOP:"]:
        if crash_sig in vm._boot_output:
            sym = _symbolicate_bugcheck(vm._boot_output, arch)
            assert False, (
                f"Crash detected in serial output: {crash_sig!r}{sym}{hint}"
            )


def test_milestones(vm: QemuVM, arch: str) -> None:
    """Verify all required boot milestones appear in serial output.

    Each arch declares a list of substrings that must appear in the boot log
    (e.g., '[SMSS] _main entered', 'shell ready'). This catches regressions
    where the kernel boots silently or stops short of user mode without
    triggering an explicit BUGCHECK.

    Reports per-milestone reached/missed status — useful for tracking how far
    a regressed boot makes it before going silent.
    """
    milestones = vm.config.required_milestones
    if not milestones:
        pytest.skip(f"no milestones configured for {arch}")

    output = vm._boot_output or ""
    missing = [m for m in milestones if m not in output]
    reached = [m for m in milestones if m in output]

    if missing:
        report = ["Missing required boot milestones:"]
        for m in milestones:
            mark = "✓" if m in reached else "✗"
            report.append(f"  {mark} {m}")
        report.append(f"\nLast 500 chars of serial: {output[-500:]!r}")
        pytest.fail("\n".join(report))


def test_qmp_status(vm: QemuVM) -> None:
    """Verify QMP reports VM as running."""
    status = vm.qmp.execute("query-status")
    assert status["return"]["running"] is True


def test_snapshot_restore(vm: QemuVM) -> None:
    """Verify snapshot save/restore cycle works."""
    vm.save_snapshot("test_snap")
    vm.restore_snapshot("test_snap")

    status = vm.qmp.execute("query-status")
    assert status["return"]["running"] is True


def test_screenshot(vm: QemuVM, arch: str) -> None:
    """Capture a screenshot and verify it shows a desktop.

    Saves as qemu/<arch>_smoke.png for visual inspection.
    """
    ppm_path = vm.work_dir / "screen.ppm"
    vm.screendump(ppm_path)
    assert ppm_path.exists(), "screendump did not create file"
    assert ppm_path.stat().st_size > 1000, "screendump file too small"

    # Convert PPM to PNG via PIL and save to qemu/ directory.  Do NOT use
    # ImageMagick `convert` — IM 6.x writes a 1-bit colormap PNG when the
    # source PPM is mostly one color (e.g. a blue desktop background), which
    # most viewers then render as solid black, hiding real desktop renders.
    png_path = PROJECT_DIR / "qemu" / f"{arch}_smoke.png"
    try:
        from PIL import Image
        Image.open(ppm_path).save(png_path, "PNG")
    except ImportError:
        # PIL not available — fall back to keeping the raw PPM
        png_path = PROJECT_DIR / "qemu" / f"{arch}_smoke.ppm"
        import shutil
        shutil.copy2(ppm_path, png_path)

    print(f"\n[screenshot] Saved to {png_path}")


# Fatal markers that mean win32k (or anything it called) died handling the
# input we injected. `shell ready` fires at desktop *render*; the raw-input
# thread → message queue → focus-window → DefWndProc → hook paths only run
# once real input arrives — exactly the ntuser surface the win32k
# translation touches, and exactly what a boot-only marker cannot catch.
_FATAL_MARKERS = (
    "*** Fatal System Error",
    "KeBugCheckEx",
    "BugCheck 0x",
    "Unhandled exception",
    "KMODE_EXCEPTION_NOT_HANDLED",
    "UNEXPECTED_KERNEL_MODE_TRAP",
    "Assertion '",
    "Assertion failed",
    "Triple fault",
)


def test_post_shell_ready_input(vm: QemuVM, arch: str) -> None:
    """Post-`shell ready` regression gate: inject real keyboard+mouse input
    and assert win32k survives dispatching it.

    The session reached `shell ready` (desktop rendered) and the autouse
    fixture restored that snapshot. We now drive input via QMP
    `input-send-event` (host-side, no in-guest agent) and assert the OS
    keeps running with no fatal marker — catching translation regressions
    that only fire on the first input event after the desktop is up, which
    boot markers structurally miss (audit open question #6).
    """
    if arch == "arm64":
        pytest.skip("arm64 uses idle-detect boot + serial-hack input; "
                    "post-input gate is i386/amd64 (QMP input-send-event)")

    # Clear serial backlog so the post-injection scan only sees input fallout.
    vm.serial_output(timeout=1.0)
    before_ppm = vm.work_dir / "pre_input.ppm"
    vm.screendump(before_ppm)

    # PS/2 keyboard (qcode) + PS/2 relative mouse — the default `-M pc`
    # devices; no usb-tablet, so use rel moves + button, not abs axes.
    def _send(events: list) -> None:
        r = vm.qmp.execute("input-send-event", events=events)
        if "error" in r:
            pytest.fail(f"QMP input-send-event failed: {r['error']}")

    key = lambda k, d: {"type": "key",
                        "data": {"down": d, "key": {"type": "qcode", "data": k}}}
    for k in ("esc", "a", "spc", "ret", "esc"):
        _send([key(k, True)])
        _send([key(k, False)])
    for _ in range(6):
        _send([{"type": "rel", "data": {"axis": "x", "value": 40}},
               {"type": "rel", "data": {"axis": "y", "value": 30}}])
    _send([{"type": "btn", "data": {"button": "left", "down": True}}])
    _send([{"type": "btn", "data": {"button": "left", "down": False}}])

    # Let win32k dispatch it; accumulate any fault output.
    settle = ""
    for _ in range(5):
        settle += vm.serial_output(timeout=1.0)

    # HARD: no fatal marker provoked by the input.
    hit = [m for m in _FATAL_MARKERS if m in settle]
    if hit:
        sym = _symbolicate_bugcheck(settle, arch)
        pytest.fail(f"win32k died handling injected input — markers {hit}\n"
                    f"{sym}\nserial tail: {settle[-800:]!r}")

    # HARD: VM still running (not paused/shutdown/guest-panicked).
    status = vm.qmp.execute("query-status")
    assert status["return"]["running"] is True, \
        f"VM not running after input: {status.get('return')}"

    # Diagnostic: after-shot for visual inspection + pixel-delta signal
    # (soft — PS/2 relative mouse + theme variance make a hard threshold
    # flaky; the crash/liveness asserts above are the real gate).
    after_png = PROJECT_DIR / "qemu" / f"{arch}_post_input.png"
    after_ppm = vm.work_dir / "post_input.ppm"
    vm.screendump(after_ppm)
    delta = "n/a"
    try:
        from PIL import Image, ImageChops
        a = Image.open(before_ppm).convert("RGB")
        b = Image.open(after_ppm).convert("RGB")
        bbox = ImageChops.difference(a, b).getbbox()
        delta = "changed" if bbox else "identical"
        b.save(after_png, "PNG")
    except ImportError:
        after_png = PROJECT_DIR / "qemu" / f"{arch}_post_input.ppm"
        import shutil
        shutil.copy2(after_ppm, after_png)
    print(f"\n[post-input] survived input dispatch; screen={delta}; "
          f"saved {after_png}")


def test_no_unexpected_ud2(arch: str) -> None:
    """Scan ntoskrnl.exe for ud2 instructions in named functions.

    ud2 in a named function means LLVM proved the function body has undefined
    behavior and replaced it with an illegal instruction. This catches
    captures(none) miscompilation and ABI mismatches WITHOUT needing to boot.

    Expected ud2 in core::option::unwrap_unchecked etc. are excluded.
    """
    build_dir = PROJECT_DIR / f"build-{arch}"
    ntoskrnl = build_dir / "ntoskrnl" / "ntoskrnl.exe"
    if not ntoskrnl.exists():
        pytest.skip(f"ntoskrnl.exe not found at {ntoskrnl}")

    try:
        from scan_ud2 import scan_ud2, find_objdump
    except ImportError:
        pytest.skip("scan_ud2.py not found in tools/")

    objdump = find_objdump(arch)
    if objdump is None:
        pytest.skip("No objdump available")

    findings = scan_ud2(ntoskrnl, objdump)
    unexpected = [f for f in findings if not f["expected"] and not f.get("known_cgu")]

    if unexpected:
        lines = [f"Found {len(unexpected)} unexpected ud2 (LLVM miscompilation):"]
        for f in unexpected:
            lines.append(f"  0x{f['address']}: {f['function']}")
        assert False, "\n".join(lines)


def test_sacred_primitives(arch: str) -> None:
    """Verify that LLVM has not eliminated sacred IRQL/spinlock primitives.

    Static binary analysis — no QEMU needed. Checks that KfRaiseIrql,
    KfLowerIrql, KeAcquireSpinLock, etc. survive optimization in the
    final ntoskrnl binary.
    """
    build_dir = PROJECT_DIR / f"build-{arch}"
    ntoskrnl = build_dir / "ntoskrnl" / "ntoskrnl.exe"
    if not ntoskrnl.exists():
        pytest.skip(f"ntoskrnl.exe not found at {ntoskrnl}")

    # Find a working objdump
    objdump_candidates = {
        "arm64": ["aarch64-w64-mingw32-objdump", "llvm-objdump"],
        "amd64": ["x86_64-w64-mingw32-objdump", "llvm-objdump"],
        "i386": ["i686-w64-mingw32-objdump", "llvm-objdump"],
    }
    objdump = None
    for candidate in objdump_candidates.get(arch, ["llvm-objdump"]):
        try:
            subprocess.run([candidate, "--version"], capture_output=True, check=True)
            objdump = candidate
            break
        except (FileNotFoundError, subprocess.CalledProcessError, PermissionError):
            continue
    if objdump is None:
        pytest.skip("No objdump available")

    result = subprocess.run(
        [objdump, "-d", str(ntoskrnl)],
        capture_output=True, text=True, timeout=120,
    )
    disasm = result.stdout

    # amd64 uses direct CR8 access for IRQL, not KfRaiseIrql/KfLowerIrql
    if arch == "amd64":
        assert disasm.count("cr8") > 0, (
            "No CR8 accesses in ntoskrnl — IRQL primitives may be eliminated"
        )
        assert "KeAcquireSpinLock" in disasm, (
            "KeAcquireSpinLock not found in ntoskrnl — spinlock primitive eliminated"
        )
    else:
        for prim in ["KfRaiseIrql", "KfLowerIrql", "KeAcquireSpinLock"]:
            assert prim in disasm, (
                f"{prim} not found in ntoskrnl — sacred primitive eliminated by LLVM"
            )


def test_idle_cpu_not_spinning(vm: QemuVM) -> None:
    """At steady-state desktop the guest must actually be IDLE, not spinning.

    Under -accel kvm a healthy HLT-idle guest burns ~0-6% host CPU; a
    starved/livelocked guest pegs ~90%+. This gate catches "boots green
    but pathological" regressions that every other smoke signal misses —
    e.g. the serial-backpressure starvation (92% idle, smoke still green)
    that silently ran every post-boot test at ~1/20th speed. Threshold is
    deliberately generous (clean baseline measured 3-6%, starved 88-93%)
    so it flags only the catastrophic-spin class, never normal jitter.
    """
    pct = process_cpu_pct(vm._proc.pid, window=3.0)
    print(f"\n[idle-cpu] steady-state qemu host CPU = {pct}% "
          f"({'CLEAN idle' if pct < 30 else 'SPINNING/STARVED'})")
    assert pct < 40.0, (
        f"Guest is NOT idle at steady state: {pct}% host CPU (healthy < ~6%, "
        f"starved/spinning ~90%+). Something is busy-looping — likely serial/"
        f"IO backpressure or an interrupt/idle-loop regression. The OS may "
        f"still 'boot green' while running ~20x slow."
    )
