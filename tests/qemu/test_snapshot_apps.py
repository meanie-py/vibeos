"""Snapshot-harness PoC: branch per-app sub-tests off the steady-state snapshot.

This extends the smoke harness with the pattern brainstormed in
`tests/qemu/experiments/snapshot_bench.py` (validated ~80-140x faster than
reboot, robust over 50 cycles, byte-deterministic restore):

    boot once  ->  conftest saves `ready` at steady state (screenshot moment)
    each case  ->  autouse `restore_vm` rewinds to pristine `ready`
                ->  drive a GUI app launch host-side (QMP input-send-event)
                ->  probe + assert (per-case expectation)

No in-guest agent yet: we drive the GUI from the host exactly like
`test_post_shell_ready_input` does. The agent is the robust follow-up — it
slots in behind the same `AppCase.probe` seam without changing the harness.

Run (boots once, both cases reuse the snapshot):

    .venv/bin/python -m pytest tests/qemu/test_snapshot_apps.py \
        -v --arch amd64 --skip-clean-build --timeout 900
"""

from __future__ import annotations

from dataclasses import dataclass

import pytest

from tests.qemu.experiments.input_liveness import cpu_pct
from tests.qemu.test_smoke import _FATAL_MARKERS, _symbolicate_bugcheck
from tests.qemu.vm import PROJECT_DIR, QemuVM

# ---------------------------------------------------------------------------
# Host-side input primitives (PS/2 qcode kbd — same model as
# test_post_shell_ready_input; `-M pc` has no usb-tablet).
# ---------------------------------------------------------------------------

# qcode names for the characters we type into the Run box.
_QCODE = {
    **{c: c for c in "abcdefghijklmnopqrstuvwxyz0123456789"},
    ".": "dot",
    "-": "minus",
    " ": "spc",
}


def _key(vm: QemuVM, qcode: str, down: bool) -> None:
    r = vm.qmp.execute(
        "input-send-event",
        events=[{"type": "key", "data": {"down": down, "key": {"type": "qcode", "data": qcode}}}],
    )
    if "error" in r:
        pytest.fail(f"QMP input-send-event failed for {qcode!r}: {r['error']}")


def _tap(vm: QemuVM, qcode: str) -> None:
    _key(vm, qcode, True)
    _key(vm, qcode, False)


def _chord(vm: QemuVM, mod: str, qcode: str) -> None:
    """Hold `mod`, tap `qcode`, release `mod` (e.g. Win+R)."""
    _key(vm, mod, True)
    _tap(vm, qcode)
    _key(vm, mod, False)


def _type(vm: QemuVM, text: str) -> None:
    for ch in text.lower():
        q = _QCODE.get(ch)
        if q is None:
            pytest.fail(f"no qcode mapping for {ch!r} in {text!r}")
        _tap(vm, q)


def _step_shot(vm: QemuVM, arch: str, tag: str) -> None:
    """Save a per-step screenshot to qemu/ for trigger debugging.

    The launch path is GUI-driven and shell-version sensitive; capturing each
    step makes one (expensive) boot tell us exactly where it breaks instead of
    bisecting across reboots. Cheap (a screendump); on by default during the
    PoC, drop once the trigger is locked in.
    """
    try:
        from PIL import Image

        ppm = vm.work_dir / f"_step_{tag}.ppm"
        vm.screendump(ppm)
        Image.open(ppm).save(PROJECT_DIR / "qemu" / f"{arch}_step_{tag}.png", "PNG")
    except Exception:
        pass


def _launch_via_run(vm: QemuVM, exe: str, arch: str) -> None:
    """Launch `exe` keyboard-only: Ctrl+Esc -> type-ahead 'r' (Run...) ->
    Enter -> type `exe` -> Enter.

    Ctrl+Esc is a win32k *global* hotkey (no focused window required, unlike
    Win+R which this build ignored), and the classic Start menu supports
    first-letter type-ahead, so 'r' selects "Run...". If the shell changes and
    this stops working, the per-case screen-delta expectation fails loudly
    (no silent pass) and the _step_*.png trail shows which step died.
    """
    _chord(vm, "ctrl", "esc")  # open Start menu
    _drain(vm, 1.5)
    _step_shot(vm, arch, f"{exe}_1_startmenu")
    _tap(vm, "r")  # type-ahead -> "Run..."
    _drain(vm, 0.5)
    _tap(vm, "ret")  # open Run dialog
    _drain(vm, 1.5)
    _step_shot(vm, arch, f"{exe}_2_rundialog")
    _type(vm, exe)
    _drain(vm, 0.5)
    _step_shot(vm, arch, f"{exe}_3_typed")
    _tap(vm, "ret")  # execute


def _drain(vm: QemuVM, secs: float) -> str:
    out = ""
    n = max(1, round(secs))
    for _ in range(n):
        out += vm.serial_output(timeout=1.0)
    return out


# ---------------------------------------------------------------------------
# Observation
# ---------------------------------------------------------------------------


def _changed_fraction(ppm_a, ppm_b) -> float:
    """Fraction of pixels that differ between two screendumps (0.0-1.0).

    Returns -1.0 if PIL is unavailable (caller treats as soft-skip of the
    delta expectation; the crash/liveness gate still applies).
    """
    try:
        from PIL import Image, ImageChops
    except ImportError:
        return -1.0
    a = Image.open(ppm_a).convert("RGB")
    b = Image.open(ppm_b).convert("RGB")
    diff = ImageChops.difference(a, b).convert("L")
    # Threshold out codec/dither noise; count pixels that really moved.
    moved = sum(1 for px in diff.getdata() if px > 16)
    return moved / (a.width * a.height)


# ---------------------------------------------------------------------------
# Per-app cases — each is one parametrized sub-test branching off `ready`.
# `probe` is the extension seam: today it's screen-delta + liveness; an
# in-guest agent later returns structured pass/fail through this same hook.
# ---------------------------------------------------------------------------


@dataclass
class AppCase:
    name: str
    exe: str  # text typed into the Run dialog
    min_changed_frac: float  # window must repaint at least this much
    settle_secs: float = 6.0  # time for the app window to draw


APP_CASES = [
    AppCase("notepad", "notepad", min_changed_frac=0.01),
    AppCase("calc", "calc", min_changed_frac=0.01),
]


@pytest.fixture(scope="module")
def pristine_ref(vm: QemuVM):
    """Canonical pristine-desktop screendump, captured once.

    exp6 proved restore is byte-deterministic, so every post-restore desktop
    should match this. Each case asserts its post-restore / pre-launch frame
    ~= this reference — that is the proof the snapshot isolates sub-tests
    (the previous app is gone).
    """
    vm.restore_snapshot("ready")
    _drain(vm, 2.0)
    # Idle/spin discriminator at steady state (KVM HLT-idle ~0%, spin ~100%).
    # amd64 showed an 88% KiUnwaitThread storm here; this surfaces whether
    # i386's desktop is the clean idle baseline (and is the owed smoke-TODO).
    try:
        idle = cpu_pct(vm._proc.pid, 2.0)
        vm._idle_cpu_pct = idle
        print(
            f"\n[snap] steady-state qemu CPU = {idle}% "
            f"({'CLEAN idle' if idle < 15 else 'SPINNING — not idle'})"
        )
    except Exception as e:
        vm._idle_cpu_pct = None
        print(f"\n[snap] idle CPU probe failed: {e}")
    ref = vm.work_dir / "pristine_ref.ppm"
    vm.screendump(ref)
    return ref


@pytest.mark.parametrize("case", APP_CASES, ids=lambda c: c.name)
def test_snapshot_app_launch(vm: QemuVM, arch: str, pristine_ref, case: AppCase) -> None:
    """From the steady-state snapshot, launch one app and assert:

    isolation : post-restore frame ~= pristine reference (prev app gone)
    liveness  : no fatal marker, VM still running after the launch
    effect    : screen changed >= case.min_changed_frac (window appeared)
    """
    if arch == "arm64":
        pytest.skip(
            "arm64 uses idle-detect boot + serial-hack input; "
            "GUI-drive PoC is i386/amd64 (QMP input-send-event)"
        )

    # autouse `restore_vm` already rewound us to pristine `ready`.
    vm.serial_output(timeout=1.0)  # clear backlog
    pre = vm.work_dir / f"{case.name}_pre.ppm"
    vm.screendump(pre)

    # --- isolation: this run's pristine frame must match the reference,
    # i.e. the snapshot wiped whatever the previous case launched.
    drift = _changed_fraction(pristine_ref, pre)
    if drift >= 0.0:
        assert drift < 0.02, (
            f"post-restore desktop drifted {drift:.3%} from pristine reference "
            f"— snapshot is NOT isolating sub-tests (previous app leaked?)"
        )

    # --- trigger: launch the app host-side.
    _launch_via_run(vm, case.exe, arch)
    settle = _drain(vm, case.settle_secs)

    # --- liveness gate (reuse the smoke fatal-marker contract).
    hit = [m for m in _FATAL_MARKERS if m in settle]
    if hit:
        sym = _symbolicate_bugcheck(settle, arch)
        pytest.fail(
            f"OS died launching {case.exe!r} — markers {hit}\n{sym}\nserial tail: {settle[-800:]!r}"
        )
    status = vm.qmp.execute("query-status")
    assert status["return"]["running"] is True, (
        f"VM not running after launching {case.exe!r}: {status.get('return')}"
    )

    # --- effect: a window appeared (screen changed vs this run's baseline).
    post = vm.work_dir / f"{case.name}_post.ppm"
    vm.screendump(post)
    changed = _changed_fraction(pre, post)

    # Save the after-shot for visual inspection regardless of outcome.
    out_png = PROJECT_DIR / "qemu" / f"{arch}_snap_{case.name}.png"
    try:
        from PIL import Image

        Image.open(post).save(out_png, "PNG")
    except ImportError:
        import shutil

        out_png = PROJECT_DIR / "qemu" / f"{arch}_snap_{case.name}.ppm"
        shutil.copy2(post, out_png)

    if changed >= 0.0:
        assert changed >= case.min_changed_frac, (
            f"launching {case.exe!r} changed only {changed:.3%} of the screen "
            f"(< {case.min_changed_frac:.1%}) — app likely did not start; "
            f"see {out_png}. If Win+R is unsupported, switch _launch_via_run "
            f"to the Ctrl+Esc Start-menu path."
        )
        print(
            f"\n[snap:{case.name}] launched; screen changed {changed:.2%}; "
            f"isolation drift {drift:.3%}; saved {out_png}"
        )
    else:
        print(
            f"\n[snap:{case.name}] PIL unavailable — crash/liveness gate "
            f"passed; visual at {out_png}"
        )
