"""Standalone input-delivery verifier — boot, inject PS/2 kbd+mouse, grep markers.

Bypasses the savevm snapshot harness (which flakes on amd64 QMP) to directly
answer the only question that matters: does an injected keystroke / mouse event
reach kbdclass / mouclass. Usage: python verify_input_direct.py <arch>
  arch = i386-uefi | amd64-uefi
"""

import sys
import time
from pathlib import Path

from tests.qemu.vm import QemuVM, get_arch_config

KBD_MARK = "[KBDCLASS] first key delivered"
FATAL = ("BUGCHECK", "*** Fatal", "KMODE_EXCEPTION")


def main() -> int:
    arch = sys.argv[1] if len(sys.argv) > 1 else "amd64-uefi"
    cfg = get_arch_config(arch)
    work = Path("/tmp") / f"verify-input-{arch}"
    work.mkdir(parents=True, exist_ok=True)
    vm = QemuVM(cfg, work)
    vm.start()
    print(f"[verify] live serial: {vm.live_log_path}", flush=True)
    try:
        vm.wait_for_boot()
        print("[verify] boot reached marker; settling 8s", flush=True)
        time.sleep(8)
        vm.serial_output(timeout=1.0)  # clear backlog

        def send(events):
            r = vm.qmp.execute("input-send-event", events=events)
            if "error" in r:
                raise RuntimeError(f"input-send-event: {r['error']}")

        def key(k, d):
            return {"type": "key", "data": {"down": d, "key": {"type": "qcode", "data": k}}}

        for k in ("esc", "a", "spc", "ret", "esc"):
            send([key(k, True)])
            send([key(k, False)])
        for _ in range(6):
            send(
                [
                    {"type": "rel", "data": {"axis": "x", "value": 40}},
                    {"type": "rel", "data": {"axis": "y", "value": 30}},
                ]
            )
        send([{"type": "btn", "data": {"button": "left", "down": True}}])
        send([{"type": "btn", "data": {"button": "left", "down": False}}])

        settle = ""
        for _ in range(15):
            settle += vm.serial_output(timeout=1.0)
            if KBD_MARK in settle:
                break
        hit = [m for m in FATAL if m in settle]
        kbd_ok = KBD_MARK in settle
        # Mouse gate mirrors test_post_shell_ready_mouse: no fatal + VM running
        # (no per-event mouclass marker exists).
        st = vm.qmp.execute("query-status")
        running = st.get("return", {}).get("running") is True
        print(
            f"[verify] {arch}: kbd_delivered={kbd_ok} vm_running_after_mouse={running} fatal={hit}",
            flush=True,
        )
        return 0 if (kbd_ok and running and not hit) else 1
    finally:
        vm.stop()


if __name__ == "__main__":
    sys.exit(main())
