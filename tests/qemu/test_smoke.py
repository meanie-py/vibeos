"""Smoke tests: verify QEMU boots to desktop, serial works, QMP works, snapshots work."""

from __future__ import annotations

import os
import re
import shutil
import subprocess
import time
from pathlib import Path

import pytest

from tests.qemu.cli import rustlints
from tests.qemu.cli.archcfg import base_arch
from tests.qemu.vm import PROJECT_DIR, QemuVM, process_cpu_pct


def _qcode_key_event(k, d):
    """A QMP input-send-event key event for qcode `k`, down if `d`."""
    return {"type": "key", "data": {"down": d, "key": {"type": "qcode", "data": k}}}


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
        r"\*\*\*\s+(?:BUGCHECK|STOP):\s+(0x[0-9A-Fa-f]+)\s*"
        r"\((0x[0-9A-Fa-f]+)[,\s]+(0x[0-9A-Fa-f]+)[,\s]+(0x[0-9A-Fa-f]+)[,\s]+(0x[0-9A-Fa-f]+)\)",
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
                    triple = {"i386": "i686", "amd64": "x86_64", "arm64": "aarch64"}
                    names = [f"{triple[base_arch(arch)]}-w64-mingw32-objdump", "llvm-objdump"]
                    candidates = names + [
                        str(PROJECT_DIR / ".build-tools/llvm-mingw/bin" / name) for name in names
                    ]
                    objdump = next((path for path in candidates if shutil.which(path)), None)
                    if objdump:
                        result = subprocess.run(
                            [objdump, "-d", str(ntoskrnl)],
                            capture_output=True,
                            text=True,
                            timeout=120,
                        )
                        # Find which function contains the crash address
                        last_func = None
                        for line in result.stdout.splitlines():
                            func_match = re.match(r"^([0-9a-f]+)\s+<(.+)>:", line)
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
        f"Too little boot output: {len(vm._boot_output)} bytes (expected >= {threshold}){hint}"
    )
    # Detect crashes in serial output (catches BSODs that idle detection misses)
    for crash_sig in ["*** BUGCHECK:", "*** STOP:"]:
        if crash_sig in vm._boot_output:
            sym = _symbolicate_bugcheck(vm._boot_output, arch)
            raise AssertionError(f"Crash detected in serial output: {crash_sig!r}{sym}{hint}")


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

    # -smp N boots (VIBEOS_QEMU_SMP > 1, SMP bring-up): additionally require
    # proof that every AP actually scheduled a non-idle thread — the kernel
    # prints a one-shot "[SMP] CPUn scheduling live" marker per AP
    # (ke/amd64/ipi.rs KiSmpNoteApSwitch, called from KiSwapContextResume).
    smp = os.environ.get("VIBEOS_QEMU_SMP", "").strip()
    if smp.isdigit() and int(smp) > 1:
        milestones = milestones + [f"CPU{n} scheduling live" for n in range(1, int(smp))]

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
    "*** BUGCHECK",
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
        pytest.skip(
            "arm64 uses idle-detect boot + serial-hack input; "
            "post-input gate is i386/amd64 (QMP input-send-event)"
        )

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

    key = _qcode_key_event
    for k in ("esc", "a", "spc", "ret", "esc"):
        _send([key(k, True)])
        _send([key(k, False)])
    for _ in range(6):
        _send(
            [
                {"type": "rel", "data": {"axis": "x", "value": 40}},
                {"type": "rel", "data": {"axis": "y", "value": 30}},
            ]
        )
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
        pytest.fail(
            f"win32k died handling injected input — markers {hit}\n"
            f"{sym}\nserial tail: {settle[-800:]!r}"
        )

    # HARD: VM still running (not paused/shutdown/guest-panicked).
    status = vm.qmp.execute("query-status")
    assert status["return"]["running"] is True, (
        f"VM not running after input: {status.get('return')}"
    )

    # HARD (i386/amd64): the injected keystroke must actually REACH the
    # keyboard class driver — delivery, not just "win32k didn't crash".
    # kbdclass.c ClassCallback emits this one-shot marker on the first real
    # key (i8042prt/usbkbd -> kbdclass). Boot markers structurally cannot
    # cover this: the raw-input path only runs once a key arrives after the
    # desktop is up. If this fails the input chain is broken upstream of
    # win32k — fix the OS, not the gate (smoke is the source of truth).
    KBD_DELIVERED = "[KBDCLASS] first key delivered"
    if KBD_DELIVERED not in settle:
        for _ in range(10):
            settle += vm.serial_output(timeout=1.0)
            if KBD_DELIVERED in settle:
                break
    assert KBD_DELIVERED in settle, (
        f"injected keystroke never reached kbdclass on {arch} — input is "
        f"not being delivered to the class driver (chain broken upstream of "
        f"win32k: HAL/i8042prt/kbdclass).\nserial tail: {settle[-1000:]!r}"
    )
    print(
        f"\n[post-input] kbd delivery confirmed — keystroke reached "
        f"kbdclass ({KBD_DELIVERED!r} observed)"
    )

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
    print(f"\n[post-input] survived input dispatch; screen={delta}; saved {after_png}")


def test_start_menu_opens(vm: QemuVM, arch: str) -> None:
    """Start-menu gate: press Ctrl+Esc on the settled desktop and assert the
    start menu actually renders.

    This is an end-to-end *effect* assertion, one level above
    test_post_shell_ready_input's delivery gate: the keystroke must traverse
    i8042prt -> kbdclass -> win32k RIT -> VkToVsc (keyboard layout!) ->
    hotkey dispatch -> explorer's tray wndproc -> menu window creation ->
    paint. A dead keyboard layout (gspklBaseLayout null) or a broken hotkey
    path passes the delivery gate but fails here.

    Saves qemu/<arch>_startmenu.png for visual inspection.
    """
    # arm64 runs this gate too since 2026-07-10: QMP input-send-event routes
    # to the bespoke arm64input virtio-mmio keyboard (test_arm64input_delivery
    # proves the QMP -> virtqueue -> KeyboardClass0 path end-to-end), so the
    # same Ctrl+Esc chord exercises win32k hotkey dispatch there as well.

    # amd64 history (field manual §9.41, ROOT-CAUSED + FIXED 2026-07-10): the
    # Ctrl+Esc explorer death AND the blank-labels/missing-items menu were one
    # bug — KiSystemCallHandler read TEB.GdiBatchCount at a hand-rolled wrong
    # offset (0x74 vs real 0x1740), so the GDI TEB batch never flushed:
    # batched ExtTextOut label text silently vanished and the batch buffer
    # overflowed into CLIENTINFO, AV'ing explorer. If a regression here
    # recurs, xfail with `arch.startswith("amd64")` (fixture arch is
    # "amd64-uefi", NOT "amd64") rather than deleting the gate.

    try:
        from PIL import Image, ImageChops
    except ImportError:
        pytest.skip("PIL not available for pixel-delta assertion")

    vm.serial_output(timeout=1.0)  # clear backlog
    before_ppm = vm.work_dir / "pre_startmenu.ppm"
    vm.screendump(before_ppm)

    def _send(events: list) -> None:
        r = vm.qmp.execute("input-send-event", events=events)
        if "error" in r:
            pytest.fail(f"QMP input-send-event failed: {r['error']}")

    key = _qcode_key_event

    def _ctrl_esc() -> None:
        _send([key("ctrl", True)])
        _send([key("esc", True)])
        _send([key("esc", False)])
        _send([key("ctrl", False)])

    # Prime the input path: the first keystroke after a snapshot restore is
    # sometimes swallowed (PS/2 device state right after loadvm). A bare Esc
    # tap is a no-op on the desktop and also closes any stray menu.
    _send([key("esc", True)])
    _send([key("esc", False)])
    time.sleep(1.0)

    # Poll for the menu to actually RENDER. Don't gate on a raw pixel-delta
    # bbox: a post-snapshot-restore full-screen repaint produces a huge delta
    # with no menu in it (observed on amd64 — solid-background transient).
    # Instead test content: the classic start menu is a light-gray panel
    # (~RGB 212,208,200) anchored above the Start button; sample the panel
    # area to the right of the vertical "ReactOS" sidebar. On the blue
    # wallpaper this region is dark; menu open flips it predominantly light.
    # Give explorer up to 20s — first-open loads shell links under TCG.
    def _menu_panel_lit(img) -> float:
        """Fraction of light-gray pixels in the start-menu panel region."""
        w, h = img.size
        # Region scaled off the 640x480 classic layout (menu ≈ x 5..170,
        # y 181..453; sidebar occupies the left ~35px of it).
        x0, x1 = int(w * 0.075), int(w * 0.25)
        y0, y1 = int(h * 0.42), int(h * 0.90)
        px = img.crop((x0, y0, x1, y1)).getdata()
        lit = sum(1 for (r, g, b) in px if min(r, g, b) > 150)
        return lit / max(1, len(px))

    before = Image.open(before_ppm).convert("RGB")
    lit_before = _menu_panel_lit(before)
    lit = lit_before
    bbox = None
    serial = ""
    after_ppm = vm.work_dir / "post_startmenu.ppm"
    after = before

    # Up to 3 Ctrl+Esc attempts, each with an ~8s render window — first-open
    # loads shell links from disk under TCG, and an occasional swallowed
    # chord (see priming note above) should not fail the gate.
    def _region(img):
        w, h = img.size
        return img.crop((int(w * 0.075), int(h * 0.42), int(w * 0.25), int(h * 0.90)))

    def _region_changed(a, b) -> bool:
        return ImageChops.difference(_region(a), _region(b)).getbbox() is not None

    # The lit-fraction alone can't prove a menu when the sample region is
    # ALREADY light before the chord (arm64: the winlogon notepad demo's white
    # client covers it — the gate passed once with NO menu on screen). Require
    # the region to also CHANGE relative to the pre-chord frame: an opening
    # menu always repaints its pixels, whatever was underneath.
    region_delta = False
    for _attempt in range(3):
        _ctrl_esc()
        # monotonic, not time.time(): a backward wall-clock step LOWERS
        # time.time(), so `while time.time() < deadline` stays true longer and
        # silently hands a slow render extra grace. Lenient is the false-green
        # direction, and this is the gate the whole smoke verdict rests on.
        deadline = time.monotonic() + 8.0
        while time.monotonic() < deadline:
            serial += vm.serial_output(timeout=1.0)
            vm.screendump(after_ppm)
            after = Image.open(after_ppm).convert("RGB")
            bbox = ImageChops.difference(before, after).getbbox()
            lit = _menu_panel_lit(after)
            region_delta = _region_changed(before, after)
            if lit >= 0.60 and region_delta:
                break
        if lit >= 0.60 and region_delta:
            break

    # The lit-threshold can trip mid-paint (panel background up, labels not
    # yet drawn — seen on amd64 under TCG). Once the gate condition is met,
    # give the menu a settle window and re-capture so the saved artifact is
    # the FINISHED render, not the first frame past the threshold. The
    # assertion below still uses the settled frame (it can only get lighter).
    if lit >= 0.60 and region_delta:
        settle_deadline = time.monotonic() + 6.0
        stable = 0
        prev = after
        while time.monotonic() < settle_deadline and stable < 2:
            time.sleep(1.0)
            vm.screendump(after_ppm)
            cur = Image.open(after_ppm).convert("RGB")
            stable = stable + 1 if ImageChops.difference(prev, cur).getbbox() is None else 0
            prev = cur
        after = prev
        lit = _menu_panel_lit(after)

    # Save the after-shot regardless of verdict — it's the artifact to eyeball.
    png_path = PROJECT_DIR / "qemu" / f"{arch}_startmenu.png"
    after.save(png_path, "PNG")
    print(
        f"\n[startmenu] Ctrl+Esc: panel-lit {lit_before:.2f} -> {lit:.2f}; "
        f"region-delta={region_delta}; delta bbox={bbox}; saved {png_path}"
    )

    # HARD: the input must not have killed anything.
    hit = [m for m in _FATAL_MARKERS if m in serial]
    if hit:
        sym = _symbolicate_bugcheck(serial, arch)
        pytest.fail(
            f"OS died opening the start menu — markers {hit}\n{sym}\nserial tail: {serial[-800:]!r}"
        )

    # HARD: the menu panel must be rendered. Both halves are required — the
    # lit fraction alone is fooled by a pre-existing light window under the
    # sample region (arm64 notepad demo), and a raw pixel-delta alone is
    # fooled by clock repaints / post-restore full-screen invalidations.
    assert lit >= 0.60 and region_delta, (
        f"start menu never rendered after Ctrl+Esc on {arch} — panel region "
        f"lit {lit:.0%} (needs ≥60%; was {lit_before:.0%} before), "
        f"region-changed={region_delta} (needs True). Keyboard layout dead? "
        f"Hotkey path broken? Explorer menu paint broken? "
        f"Screenshot: {png_path}"
    )


def test_post_shell_ready_mouse(vm: QemuVM, arch: str) -> None:
    """Mouse-specific dispatch gate: drive a sustained PS/2 mouse storm
    (multi-direction sweep + button chatter) and assert the
    `mouclass -> win32k` pointer path survives it.

    Complements `test_post_shell_ready_input` (keyboard-weighted): the
    pointer path (RawInputThread -> cursor move -> hit-test -> hover/track
    -> SetCursor) is a distinct slice of the win32k Rust translation, and a
    burst is what shakes out reentrancy/lock regressions there.

    Why liveness-only and not "cursor moved": QEMU `screendump` does not
    composite the PS/2 cursor from the restored `ready` snapshot state
    (every dump is byte-identical even though input is dispatched), so a
    pixel-diff effect assertion is structurally flaky here. Robust effect
    verification is the in-guest agent (AppCase.probe seam). This gate is
    the non-flaky half: a mouse storm must not kill win32k. It is a real
    regression gate independent of input *reaching userland* — on i386
    (PS/2 dead until the ACPI-HAL backport) the events are dropped at the
    bus, so the OS still must not fault handling the empty path.
    """
    if arch == "arm64":
        pytest.skip(
            "arm64 uses idle-detect boot + serial-hack input; "
            "mouse-dispatch gate is i386/amd64 (QMP input-send-event)"
        )

    vm.serial_output(timeout=1.0)  # clear backlog
    before_ppm = vm.work_dir / "pre_mouse.ppm"
    vm.screendump(before_ppm)

    def _send(events: list) -> None:
        r = vm.qmp.execute("input-send-event", events=events)
        if "error" in r:
            pytest.fail(f"QMP input-send-event failed: {r['error']}")

    # Sustained sweep: box pattern + diagonals, with button chatter — the
    # kind of burst that surfaces pointer-path reentrancy regressions.
    legs = [(45, 0), (0, 35), (-45, 0), (0, -35), (40, 30), (-40, -30)]
    for dx, dy in legs:
        for _ in range(8):
            _send(
                [
                    {"type": "rel", "data": {"axis": "x", "value": dx}},
                    {"type": "rel", "data": {"axis": "y", "value": dy}},
                ]
            )
        _send([{"type": "btn", "data": {"button": "left", "down": True}}])
        _send([{"type": "btn", "data": {"button": "left", "down": False}}])
    _send([{"type": "btn", "data": {"button": "right", "down": True}}])
    _send([{"type": "btn", "data": {"button": "right", "down": False}}])

    settle = ""
    for _ in range(5):
        settle += vm.serial_output(timeout=1.0)

    # HARD: mouse storm did not crash win32k / the pointer path.
    hit = [m for m in _FATAL_MARKERS if m in settle]
    if hit:
        sym = _symbolicate_bugcheck(settle, arch)
        pytest.fail(
            f"win32k died handling mouse storm — markers {hit}\n"
            f"{sym}\nserial tail: {settle[-800:]!r}"
        )

    # HARD: VM still alive.
    status = vm.qmp.execute("query-status")
    assert status["return"]["running"] is True, (
        f"VM not running after mouse storm: {status.get('return')}"
    )

    # Diagnostic only (cursor not reliably composited from snapshot state).
    after_png = PROJECT_DIR / "qemu" / f"{arch}_post_mouse.png"
    after_ppm = vm.work_dir / "post_mouse.ppm"
    vm.screendump(after_ppm)
    delta = "n/a"
    try:
        from PIL import Image, ImageChops

        a = Image.open(before_ppm).convert("RGB")
        b = Image.open(after_ppm).convert("RGB")
        delta = "changed" if ImageChops.difference(a, b).getbbox() else "identical"
        b.save(after_png, "PNG")
    except ImportError:
        after_png = PROJECT_DIR / "qemu" / f"{arch}_post_mouse.ppm"
        import shutil

        shutil.copy2(after_ppm, after_png)
    print(f"\n[post-mouse] survived mouse storm; screen={delta}; saved {after_png}")


def test_arm64input_delivery(vm: QemuVM, arch: str) -> None:
    """arm64 input *delivery* gate (not just liveness): inject keyboard via
    QMP and assert the arm64input.rs Rust driver actually delivered it up
    the class-driver stack.

    First functional coverage for the bespoke arm64input virtio-mmio
    driver. Boot only proves DriverEntry ran + virtio devices enumerated;
    the one-shot `[ARM64INPUT] KBD-DELIVERED` serial marker proves the full
    QMP -> virtio-mmio -> poll thread -> scancode xlate -> KeyboardClass0
    ring -> read-IRP path completed end-to-end (the marker is emitted
    exactly once by the driver via AtomicBool, so re-injecting across the
    read window is safe). This gate caught three real pre-existing bugs:
    a wide/narrow device-name string (OBJECT_PATH_SYNTAX_BAD), a missing
    legacy QueueAlign write, and — the root cause — non-volatile reads of
    the device-DMA'd virtqueue used ring (LLVM constant-folded used.idx to
    0 so the poll never saw QEMU's completions).

    KEYBOARD is the hard gate. MOUSE delivery (`MOU-DELIVERED`) is reported
    but non-fatal — the pointer accumulate+EV_SYN-flush path is a distinct
    secondary slice with a tracked remaining arm64 wrinkle.

    i386/amd64 have no arm64input driver — covered by
    test_post_shell_ready_input / _mouse instead.
    """
    if arch != "arm64":
        pytest.skip(
            "arm64input is the arm64-only bespoke virtio-mmio "
            "driver; i386/amd64 use test_post_shell_ready_input"
        )

    vm.serial_output(timeout=2.0)  # drain backlog so the scan sees fallout

    # Default routing (no `device=`: QMP input-send-event with a `device`
    # arg aborts QEMU for virtio-mmio input — they have no console/head
    # property, object_property_find_err). QEMU's input subsystem routes
    # typed events to the matching virtio-input handler.
    def _send(events: list) -> None:
        r = vm.qmp.execute("input-send-event", events=events)
        if "error" in r:
            pytest.fail(f"QMP input-send-event failed: {r['error']}")

    key = _qcode_key_event

    def _inject() -> None:
        for k in ("a", "spc", "ret", "esc"):
            _send([key(k, True)])
            _send([key(k, False)])
        for _ in range(6):
            _send(
                [
                    {"type": "rel", "data": {"axis": "x", "value": 40}},
                    {"type": "rel", "data": {"axis": "y", "value": 30}},
                ]
            )
        _send([{"type": "btn", "data": {"button": "left", "down": True}}])
        _send([{"type": "btn", "data": {"button": "left", "down": False}}])

    # KEYBOARD delivery is the HARD gate (the bespoke arm64input virtio-mmio
    # kbd path, end-to-end). MOUSE delivery is reported but non-fatal: the
    # pointer accumulate+EV_SYN-flush path is a distinct secondary slice with
    # a known remaining arm64 wrinkle (tracked follow-up); the keyboard gate
    # is the regression-critical one and proves the DMA/virtqueue path works.
    # TCG is slow + the poll thread sleeps when idle; re-inject across a
    # generous window and stop as soon as the keyboard marker lands.
    KBD = "[ARM64INPUT] KBD-DELIVERED"
    MOU = "[ARM64INPUT] MOU-DELIVERED"
    acc = ""
    for _ in range(30):
        _inject()
        acc += vm.serial_output(timeout=2.0)
        hit = [m for m in _FATAL_MARKERS if m in acc]
        if hit:
            sym = _symbolicate_bugcheck(acc, arch)
            pytest.fail(
                f"arm64input died handling injected input — "
                f"markers {hit}\n{sym}\nserial tail: {acc[-800:]!r}"
            )
        if KBD in acc:
            break

    missing = [] if KBD in acc else [KBD]
    if missing:
        # Self-diagnose: the driver's boot-time enumeration ([VIRTIO-INPUT]
        # Found N device(s) / version / FEATURES_OK / Queue setup) lives in
        # the boot serial, not in `acc`. Surface it + persist full serial so
        # the failure is actionable without a blind re-run.
        boot = vm._boot_output or ""
        drv = [
            ln
            for ln in (boot + "\n" + acc).splitlines()
            if "VIRTIO-INPUT" in ln or "ARM64INPUT" in ln
        ]
        # QEMU's authoritative virtqueue view at injection time — the only
        # thing that can settle "guest ring correct but QEMU sees it empty":
        # compare evtq desc/avail/used addr + avail/used idx vs the guest's
        # [ARM64INPUT] QSETUP line. info virtio-queue-status needs the qom
        # path of the virtio-input device.
        virtio = ""
        try:

            def _hmp(c: str) -> str:
                r = vm.qmp.execute("human-monitor-command", **{"command-line": c})
                return r.get("return", "") if isinstance(r, dict) else str(r)

            listing = _hmp("info virtio")
            virtio += "=== info virtio ===\n" + listing + "\n"
            # Each line: "<qom-path> [<type>]" — grab the path token; keep
            # only the virtio-input devices (the ones under test).
            paths = re.findall(r"^\s*(\S+)\s+\[virtio-input\]", listing, re.M)
            for path in paths:
                virtio += f"=== info virtio-status {path} ===\n"
                virtio += _hmp(f"info virtio-status {path}") + "\n"
                for q in (0, 1):
                    virtio += f"=== virtio-queue-status {path} {q} ===\n"
                    virtio += _hmp(f"info virtio-queue-status {path} {q}") + "\n"
        except Exception as e:  # diagnostic only — never mask the real fail
            virtio = f"(info virtio capture failed: {e})"
        dbg = PROJECT_DIR / "qemu" / "arm64_input_debug.log"
        try:
            dbg.write_text(
                boot + "\n==== POST-INJECT ====\n" + acc + "\n==== QEMU VIRTIO ====\n" + virtio
            )
        except OSError:
            pass
        assert not missing, (
            f"arm64input did not deliver keyboard input within the window — "
            f"the bespoke virtio-mmio kbd path is broken end-to-end.\n"
            f"driver boot markers ({len(drv)}):\n  "
            + (
                "\n  ".join(drv[-25:])
                if drv
                else "(NONE — driver never logged "
                "[VIRTIO-INPUT]/[ARM64INPUT]; not loaded or DriverEntry failed)"
            )
            + f"\n=== QEMU virtio (authoritative vq view) ===\n{virtio[-2000:]}"
            f"\nfull serial saved: {dbg}\nserial tail: {acc[-400:]!r}"
        )

    status = vm.qmp.execute("query-status")
    assert status["return"]["running"] is True, (
        f"VM not running after arm64 input delivery: {status.get('return')}"
    )

    if MOU in acc:
        print(
            "\n[arm64input] delivery confirmed — KBD + MOU reached the class-driver ring end-to-end"
        )
    else:
        # Non-fatal: keyboard (the hard gate) is green. arm64 mouse
        # accumulate+EV_SYN-flush is a tracked secondary follow-up.
        print(
            "\n[arm64input] KEYBOARD delivery confirmed (hard gate green). "
            "MOUSE marker not observed — known arm64 pointer-path follow-up, "
            "non-fatal (kbd path proves the DMA/virtqueue chain works)."
        )


def test_no_unexpected_ud2(arch: str) -> None:
    """Scan ntoskrnl.exe for ud2 instructions in named functions.

    ud2 in a named function means LLVM proved the function body has undefined
    behavior and replaced it with an illegal instruction. This catches
    captures(none) miscompilation and ABI mismatches WITHOUT needing to boot.

    Expected ud2 in core::option::unwrap_unchecked etc. are excluded.
    """
    build_dir = PROJECT_DIR / f"build-{base_arch(arch)}"
    ntoskrnl = build_dir / "ntoskrnl" / "ntoskrnl.exe"
    if not ntoskrnl.exists():
        pytest.skip(f"ntoskrnl.exe not found at {ntoskrnl}")

    result = rustlints.run_binary(
        PROJECT_DIR,
        base_arch(arch),
        ["scan_ud2"],
        lambda cmd: subprocess.run(cmd, cwd=PROJECT_DIR, capture_output=True, text=True),
    )["scan_ud2"]
    assert result.returncode == 0, result.stdout + result.stderr


def test_sacred_primitives(arch: str) -> None:
    """Verify that LLVM has not eliminated sacred IRQL/spinlock primitives.

    Static binary analysis — no QEMU needed. Checks that KfRaiseIrql,
    KfLowerIrql, KeAcquireSpinLock, etc. survive optimization in the
    final ntoskrnl binary.
    """
    build_dir = PROJECT_DIR / f"build-{base_arch(arch)}"
    ntoskrnl = build_dir / "ntoskrnl" / "ntoskrnl.exe"
    if not ntoskrnl.exists():
        pytest.skip(f"ntoskrnl.exe not found at {ntoskrnl}")

    result = rustlints.run_binary(
        PROJECT_DIR,
        base_arch(arch),
        ["sacred_primitives"],
        lambda cmd: subprocess.run(cmd, cwd=PROJECT_DIR, capture_output=True, text=True),
    )["sacred_primitives"]
    assert result.returncode == 0, result.stdout + result.stderr


def test_idle_cpu_not_spinning(vm: QemuVM, arch: str) -> None:
    """At steady-state desktop the guest must actually be IDLE, not spinning.

    Under -accel kvm a healthy HLT-idle guest burns ~0-6% host CPU; a
    starved/livelocked guest pegs ~90%+. This gate catches "boots green
    but pathological" regressions that every other smoke signal misses —
    e.g. the serial-backpressure starvation (92% idle, smoke still green)
    that silently ran every post-boot test at ~1/20th speed. Threshold is
    deliberately generous (clean baseline measured 3-6%, starved 88-93%)
    so it flags only the catastrophic-spin class, never normal jitter.

    KVM-only by construction: the baseline calibration above assumes the
    HLT-idle vCPU thread sleeps under -accel kvm (i386/amd64). arm64 has
    no KVM on an x86 host (pure TCG) and no GIC, so it busy-polls at idle
    and TCG pegs a host core regardless of guest idle health — the host-%
    signal is meaningless there. Skipped on arm64 for the same reason its
    sibling KVM/objdump-scoped gates (post_input, no_unexpected_ud2,
    sacred_primitives) skip arm64; the assertion is unchanged for the
    arches where it is valid.
    """
    if arch == "arm64":
        pytest.skip(
            "arm64 is TCG (no KVM on x86 host) + no GIC: busy-polls "
            "at idle so host-CPU%% is not a valid idle signal; this "
            "gate is -accel kvm-only (i386/amd64)"
        )

    if getattr(vm, "_backend", "native") == "windows-host":
        # Doubly invalid here: (1) vm._proc.pid is the WSL *interop shim* that
        # launched the .exe, not the Windows QEMU process, so /proc/<pid>/task
        # measures the shim (~0%), not the guest vCPU — the number is garbage;
        # (2) the backend runs TCG, not KVM, so the HLT-idle baseline this gate
        # is calibrated against (clean 3-6%, starved 90%+) doesn't apply. Same
        # reasoning as the arm64 skip. Skip honestly rather than read the shim
        # and report a meaningless "pass".
        pytest.skip(
            "windows-host backend: vm._proc.pid is the WSL interop shim "
            "(not the Windows QEMU) so /proc CPU% is unreadable, and it "
            "runs TCG not KVM — host-CPU%% is not a valid idle signal"
        )

    assert vm._proc is not None
    pct = process_cpu_pct(vm._proc.pid, window=3.0)
    print(
        f"\n[idle-cpu] steady-state qemu host CPU = {pct}% "
        f"({'CLEAN idle' if pct < 30 else 'SPINNING/STARVED'})"
    )
    assert pct < 40.0, (
        f"Guest is NOT idle at steady state: {pct}% host CPU (healthy < ~6%, "
        f"starved/spinning ~90%+). Something is busy-looping — likely serial/"
        f"IO backpressure or an interrupt/idle-loop regression. The OS may "
        f"still 'boot green' while running ~20x slow."
    )
