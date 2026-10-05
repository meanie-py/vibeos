"""Tier-A input bisection: does injected input even raise a guest IRQ?

QEMU `info irq` counts IRQ1 (PS/2 keyboard) and IRQ12 (PS/2 aux mouse).
Zero guest perturbation, no gdb/kdb/symbols. Boot healthy i386 (serial
fix in place), inject a burst of keys+mouse, diff the IRQ counts.

  IRQ1/12 jump  => interrupt FIRES; i8042 delivery OK; break is higher
                   (i8042prt -> kbdclass -> win32k RIT -> input desktop)
  IRQ1/12 flat  => QEMU input-send-event isn't reaching the guest i8042
                   (input routing / device config) — the break is BELOW
                   the driver stack

    python -m tests.qemu.experiments.input_irq
"""

from __future__ import annotations

import re
import time
from pathlib import Path

from tests.qemu.vm import QemuVM, get_arch_config

_IRQ_LINE = re.compile(r"^\s*(\d+):\s+(\d+)\s*$", re.M)


def _irq(vm: QemuVM) -> dict[int, int]:
    txt = vm.qmp.hmp("info irq")
    c: dict[int, int] = {}
    for n, v in _IRQ_LINE.findall(txt):
        n, v = int(n), int(v)
        c[n] = max(c.get(n, 0), v)
    return c


def _key(vm: QemuVM, q: str, down: bool) -> None:
    vm.qmp.execute(
        "input-send-event",
        events=[{"type": "key", "data": {"down": down, "key": {"type": "qcode", "data": q}}}],
    )


def main() -> None:
    cfg = get_arch_config("i386")
    cfg.desktop_settle_seconds = 20.0
    vm = QemuVM(cfg, Path("/tmp/input_irq_work"))
    print("=== boot i386 (serial-fixed, healthy) ===")
    vm.start()
    vm.wait_for_boot()
    print("  booted")

    try:
        time.sleep(3)
        before = _irq(vm)

        # Burst: 20 key taps + 40 mouse moves + 4 clicks.
        for _ in range(20):
            for k in ("esc", "a", "r", "ret"):
                _key(vm, k, True)
                _key(vm, k, False)
        for _ in range(40):
            vm.qmp.execute(
                "input-send-event",
                events=[
                    {"type": "rel", "data": {"axis": "x", "value": 25}},
                    {"type": "rel", "data": {"axis": "y", "value": 18}},
                ],
            )
        for _ in range(4):
            vm.qmp.execute(
                "input-send-event",
                events=[{"type": "btn", "data": {"button": "left", "down": True}}],
            )
            vm.qmp.execute(
                "input-send-event",
                events=[{"type": "btn", "data": {"button": "left", "down": False}}],
            )
        time.sleep(2)
        after = _irq(vm)

        kbd = after.get(1, 0) - before.get(1, 0)
        mouse = after.get(12, 0) - before.get(12, 0)
        print(f"\n  IRQ1  (PS/2 keyboard) delta = {kbd}")
        print(f"  IRQ12 (PS/2 mouse)    delta = {mouse}")
        deltas = {
            k: after.get(k, 0) - before.get(k, 0)
            for k in sorted(set(before) | set(after))
            if after.get(k, 0) - before.get(k, 0)
        }
        print(f"  all IRQ deltas: {deltas}")

        if kbd > 0 or mouse > 0:
            print(
                "\n  VERDICT: input IRQ FIRES in the guest — QEMU->i8042 "
                "delivery is OK. The break is ABOVE the controller: "
                "i8042prt/kbdclass/mouclass ISR, win32k RawInputThread, or "
                "the input desktop (Tier-B: kdb BP on the ISR / dump "
                "grpdeskRitInput vs the rendered desktop)."
            )
        else:
            print(
                "\n  VERDICT: NO input IRQ — QEMU input-send-event is not "
                "reaching the guest i8042 at all (input routing / no PS2 "
                "consumer with -display none). The break is BELOW the "
                "driver stack; fix is QEMU-side (e.g. add an explicit "
                "kbd/mouse device or input route), not win32k."
            )
    finally:
        vm.stop()


if __name__ == "__main__":
    main()
