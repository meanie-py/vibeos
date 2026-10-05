"""Does host-side QMP MOUSE injection drive this win32k? (keyboard didn't)

-M pc has a *relative* PS/2 mouse (no usb-tablet / no abs axes). Trick:
spam large-negative rel moves -> cursor clamps at top-left (0,0); then a
large +dy with small +dx -> clamps at the BOTTOM-LEFT corner, which is
exactly the Start button regardless of resolution. Click there; if the
Start menu opens the screen changes massively -> host mouse-drive WORKS
(different win32k path than the dead keyboard-hotkey path).

Boots i386 once (serial-fix => clean idle), no snapshot needed.
    python -m tests.qemu.experiments.mouse_start
"""

from __future__ import annotations

import time
from pathlib import Path

from tests.qemu.vm import PROJECT_DIR, QemuVM, get_arch_config


def _rel(vm: QemuVM, dx: int, dy: int) -> None:
    vm.qmp.execute(
        "input-send-event",
        events=[
            {"type": "rel", "data": {"axis": "x", "value": dx}},
            {"type": "rel", "data": {"axis": "y", "value": dy}},
        ],
    )


def _click(vm: QemuVM) -> None:
    vm.qmp.execute(
        "input-send-event", events=[{"type": "btn", "data": {"button": "left", "down": True}}]
    )
    vm.qmp.execute(
        "input-send-event", events=[{"type": "btn", "data": {"button": "left", "down": False}}]
    )


def _shot(vm: QemuVM, tag: str) -> Path:
    ppm = vm.work_dir / f"_ms_{tag}.ppm"
    vm.screendump(ppm)
    png = PROJECT_DIR / "qemu" / f"i386_ms_{tag}.png"
    try:
        from PIL import Image

        Image.open(ppm).save(png, "PNG")
    except Exception:
        pass
    return ppm


def _delta_pct(a: Path, b: Path) -> float:
    from PIL import Image, ImageChops

    ia = Image.open(a).convert("RGB")
    ib = Image.open(b).convert("RGB")
    d = ImageChops.difference(ia, ib).convert("L")
    moved = sum(1 for px in d.getdata() if px > 16)
    return moved / (ia.width * ia.height) * 100.0


def main() -> None:
    cfg = get_arch_config("i386")
    cfg.desktop_settle_seconds = 25.0
    vm = QemuVM(cfg, Path("/tmp/mouse_start_work"))
    print("=== boot i386 (serial-fixed; clean idle) ===")
    vm.start()
    vm.wait_for_boot()
    print("  booted")

    try:
        time.sleep(3)
        base = _shot(vm, "0_baseline")

        # 1. Clamp to top-left: many big negative rel moves.
        for _ in range(15):
            _rel(vm, -400, -400)
        time.sleep(0.3)
        _shot(vm, "1_topleft")

        # 2. Drive to BOTTOM-LEFT corner = Start button (res-independent):
        #    huge +dy clamps at bottom; small +dx keeps us at the far left.
        for _ in range(15):
            _rel(vm, 6, 400)
        time.sleep(0.5)
        atstart = _shot(vm, "2_at_start")

        # 3. Click the Start button.
        _click(vm)
        time.sleep(1.5)
        clicked = _shot(vm, "3_after_click")

        d_move = _delta_pct(base, atstart)
        d_click = _delta_pct(atstart, clicked)
        d_total = _delta_pct(base, clicked)
        print(f"\n  screenΔ cursor-moved-to-Start : {d_move:.3f}%")
        print(f"  screenΔ after Start click     : {d_click:.3f}%")
        print(f"  screenΔ total vs baseline     : {d_total:.3f}%")
        if d_click > 1.0:
            print(
                "\n  VERDICT: Start menu OPENED — host MOUSE-drive WORKS on "
                "this win32k (keyboard hotkeys don't, mouse does). Next: "
                "navigate the menu to launch notepad/calc."
            )
        elif d_move > 0.3:
            print(
                "\n  PARTIAL: cursor moved (mouse input IS delivered) but "
                "the click didn't open Start — try click timing / position."
            )
        else:
            print(
                "\n  VERDICT: no screen change at all — host mouse injection "
                "also doesn't drive this win32k. Pivot to in-guest agent."
            )
        print("  screenshots: qemu/i386_ms_*.png")
    finally:
        vm.stop()


if __name__ == "__main__":
    main()
