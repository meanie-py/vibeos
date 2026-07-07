"""Opt-in Doom foreign-binary smoke suite (`--doom`).

Boots the arch's normal smoke media and asserts the VIBEOS_DOOM=1 demo
actually works end-to-end: doom.exe launched, the game is rendering frames,
and (where the input chain works) a keypress REACHES the game. This is the
adversarial-compat gate the foreign-binary quest bought us — a win32k/GDI/
msvcrt regression that only bites real Windows software turns this red while
the normal smoke stays green.

OFF by default: requires `--doom` AND media built with `VIBEOS_DOOM=1`
(tools/doom/build.sh <arch> && VIBEOS_DOOM=1 ./vibeos-cli build --arch <arch>).
Full bring-up story: docs/engineering/doom-on-vibeos.md.

Run: pytest tests/qemu/test_doom.py --arch amd64-uefi --doom
Front door: ./vibeos-cli smoke --arch amd64 --doom
"""

from __future__ import annotations

import time

import pytest

from .vm import QemuVM

DOOM_LAUNCHED = "doom.exe created"  # [WINLOGON] (arm64) / [USERINIT] (x86)

# Fraction of screen pixels that must change between two frames ~2s apart for
# the game to count as "rendering". The desktop alone only animates the tray
# clock (<<0.1%); the Doom attract demo animates most of its viewport.
RENDER_DELTA_MIN = 0.02
# Fraction allowed to still change on an idle in-game view (the status-bar
# face animates ~0.4% of the screen; tolerance for palette dither on top).
STATIC_DELTA_MAX = 0.012


def _require_doom(request: pytest.FixtureRequest) -> None:
    if not request.config.getoption("--doom"):
        pytest.skip("doom suite is opt-in (pass --doom)")


def _frame_delta(vm: QemuVM, gap: float) -> float:
    """Fraction of pixels differing between two screendumps `gap` seconds apart."""
    from PIL import Image, ImageChops

    a_p = vm.work_dir / "doom_a.ppm"
    b_p = vm.work_dir / "doom_b.ppm"
    vm.screendump(a_p)
    time.sleep(gap)
    vm.screendump(b_p)
    a = Image.open(a_p).convert("RGB")
    b = Image.open(b_p).convert("RGB")
    diff = ImageChops.difference(a, b).convert("L").point(lambda v: 255 if v > 8 else 0)
    hist = diff.histogram()
    changed = sum(hist[1:])
    return changed / float(a.size[0] * a.size[1])


def _click_into_doom(vm: QemuVM) -> None:
    """Click into the Doom window to give it foreground focus — exactly what a
    human does. Doom's window is not foreground-activated at spawn (winlogon
    spawns it without SetForegroundWindow); a left-click both activates it and
    exercises the mouse chain (mouclass -> win32k hit-test -> activation). The
    window is maximized-ish at the top-left, so sweep the PS/2 relative mouse
    far up-left, then nudge into the client area and click."""

    def _send(events: list) -> None:
        r = vm.qmp.execute("input-send-event", events=events)
        if "error" in r:
            pytest.fail(f"QMP input-send-event(mouse) failed: {r['error']}")

    for _ in range(25):  # pin to top-left corner (rel mouse: saturate)
        _send(
            [
                {"type": "rel", "data": {"axis": "x", "value": -50}},
                {"type": "rel", "data": {"axis": "y", "value": -40}},
            ]
        )
    for _ in range(10):  # move to ~screen centre — well inside the maximized client area
        _send(
            [
                {"type": "rel", "data": {"axis": "x", "value": 40}},
                {"type": "rel", "data": {"axis": "y", "value": 28}},
            ]
        )
    _send([{"type": "btn", "data": {"button": "left", "down": True}}])
    _send([{"type": "btn", "data": {"button": "left", "down": False}}])
    time.sleep(1.5)  # let activation settle


def _send_key(vm: QemuVM, qcode: str) -> None:
    for down in (True, False):
        r = vm.qmp.execute(
            "input-send-event",
            events=[
                {"type": "key", "data": {"down": down, "key": {"type": "qcode", "data": qcode}}}
            ],
        )
        if "error" in r:
            pytest.fail(f"QMP input-send-event({qcode}) failed: {r['error']}")


def test_doom_launched(vm: QemuVM, arch: str, request: pytest.FixtureRequest) -> None:
    """doom.exe was spawned (image was VIBEOS_DOOM-staged and the launcher ran)."""
    _require_doom(request)
    output = vm._boot_output or ""
    if DOOM_LAUNCHED not in output:
        # Launcher may fire slightly after the boot marker — drain a little more.
        for _ in range(15):
            output += vm.serial_output(timeout=1.0)
            if DOOM_LAUNCHED in output:
                break
    assert DOOM_LAUNCHED in output, (
        "--doom was passed but doom.exe never launched. Either the media was "
        "not built with VIBEOS_DOOM=1 (fix: tools/doom/build.sh <arch> && "
        "VIBEOS_DOOM=1 ./vibeos-cli build --arch <arch>) or the launcher "
        f"regressed (grep serial for CreateProcessW errors).\n"
        f"serial tail: {output[-800:]!r}"
    )


def test_doom_renders(vm: QemuVM, arch: str, request: pytest.FixtureRequest) -> None:
    """The game is actually drawing frames (attract demo animates the screen)."""
    _require_doom(request)
    pytest.importorskip("PIL")
    time.sleep(3)  # let the window come up / demo start after restore
    delta = _frame_delta(vm, gap=2.0)
    assert delta >= RENDER_DELTA_MIN, (
        f"screen delta {delta:.4f} < {RENDER_DELTA_MIN} — doom.exe launched but "
        "nothing is animating (GDI blit path / game loop wedged?). Check "
        f"qemu/{arch}_smoke.png and the serial log."
    )
    print(f"\n[doom] rendering confirmed — frame delta {delta:.1%}")


def test_doom_input_reaches_game(vm: QemuVM, arch: str, request: pytest.FixtureRequest) -> None:
    """PLAYABLE gate: start a NEW GAME of Doom with injected keys.

    The probe is the strongest e2e proof available: drive Doom's menu with
    ESC -> ENTER -> ENTER -> ENTER (menu / New Game / episode / skill) and the
    game starts at E1M1 with an IDLE player. An idle first-person view is
    near-static (<1% frame delta: just the status-bar face), while the attract
    demo animates ~50% of the screen — a clean binary signal with no pixel-color
    heuristics. (An earlier version probed F1=help, but doomgeneric's
    convertToDoomKey doesn't map VK_F1, so the game legitimately ignored it —
    the oracle was broken, not the input chain. ESC/ENTER/arrows ARE mapped.)

    A mouse click into the window first gives it foreground focus, covering the
    mouse chain too (click -> activation); keys then retry a couple of times to
    absorb post-restore focus settling.

    Real catches so far (all found by this gate, 2026-07-06/07, amd64):
      - acpi-rust's invented CmResourceType table typed IRQ descriptors as
        Null -> i8042prt never connected its ISRs -> PS/2 kbd+mouse dead.
      - i8042prt mouse reset state machine used transmuted invalid enum
        discriminants (1001..) -> UB -> LLVM deleted the match arms -> mouse
        stalled at wheel-detect forever.
      - winlogon only loaded keyboard layouts on arm64 -> win32k dropped every
        key at "NO keyboard layout" on x86.
    """
    _require_doom(request)
    pytest.importorskip("PIL")
    if arch.startswith("arm64"):
        pytest.xfail(
            "arm64 kbd/mouse -> foreground-focus routing is an open win32k gap "
            "(doom-on-vibeos.md bug #4); flip this to a hard assert when fixed"
        )
    if arch.startswith("i386"):
        pytest.xfail(
            "i386 can't load a keyboard layout at boot: LoadKeyboardLayoutW -> "
            "NtCreateSection(SEC_IMAGE) deadlocks in the i386 Rust MM, so "
            "winlogon skips it (would wedge before explorer). Keys reach "
            "kbdclass but win32k drops them at 'NO keyboard layout'. Flip to a "
            "hard assert once the i386 SEC_IMAGE MM deadlock is fixed. "
            "(amd64 IS a hard assert — the layout loads there.)"
        )

    time.sleep(3)
    _click_into_doom(vm)
    pre = _frame_delta(vm, gap=1.5)
    assert pre >= RENDER_DELTA_MIN, (
        f"pre-input frame delta {pre:.4f} too low — game not animating, "
        "input test would be meaningless"
    )

    # Menu-drive a new game, with retries (foreground can lag right after the
    # snapshot restore while explorer settles; ESC also closes a stray menu).
    post = 1.0
    for attempt in range(3):
        for key in ("esc", "ret", "ret", "ret"):
            _send_key(vm, key)
            time.sleep(0.7)
        time.sleep(2.0)  # level load + first frame
        post = _frame_delta(vm, gap=1.5)
        if post <= STATIC_DELTA_MAX:
            break
        print(f"[doom] new-game attempt {attempt + 1}: delta {post:.4f} — refocus + retry")
        _send_key(vm, "esc")
        time.sleep(0.7)
        _click_into_doom(vm)

    if post > STATIC_DELTA_MAX:
        # Persist a screenshot for postmortem (who actually has foreground?).
        from .vm import PROJECT_DIR

        shot = PROJECT_DIR / "qemu" / f"{arch}_doom_input_fail.ppm"
        vm.screendump(shot)
        pytest.fail(
            f"post-newgame frame delta {post:.4f} > {STATIC_DELTA_MAX} — the demo "
            "kept animating, so the menu keys never reached doom.exe (3 attempts, "
            "click-refocus between). Input chain is broken between the class "
            f"driver and the foreground window (win32k routing/focus). "
            f"Screenshot: {shot}"
        )

    # And the OS survived the whole exchange.
    status = vm.qmp.execute("query-status")
    assert status["return"]["running"] is True
    print(f"\n[doom] PLAYABLE — new game started via menu keys (delta {pre:.1%} -> {post:.1%})")
