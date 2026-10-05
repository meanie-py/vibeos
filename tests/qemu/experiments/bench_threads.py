"""Isolate the vm.py-config 88% artifact: boot the FAT (vm.py/BenchConfig)
QEMU config and break host CPU down per thread + per vm.py-only flag class.

start.sh (lean) idles at 3.4%; BenchConfig (scratch qcow2 + -vga cirrus +
-debugcon + reconnect socket-serial) idles at ~88%. Per-thread split says
which subsystem: a hot main-loop/IO thread => chardev/display/blockdev;
a hot vCPU thread => vmexit storm.

    python -m tests.qemu.experiments.bench_threads
"""

from __future__ import annotations

import os
import threading
import time
from pathlib import Path

from tests.qemu.vm import QemuVM, get_arch_config

_CLK = os.sysconf("SC_CLK_TCK")


def _threads(pid: int) -> dict[int, tuple[str, int]]:
    out: dict[int, tuple[str, int]] = {}
    for t in Path(f"/proc/{pid}/task").iterdir():
        try:
            comm = (t / "comm").read_text().strip()
            parts = (t / "stat").read_text().rsplit(")", 1)[1].split()
            out[int(t.name)] = (comm, int(parts[11]) + int(parts[12]))
        except (FileNotFoundError, ProcessLookupError, IndexError):
            pass
    return out


def main() -> None:
    # The REAL smoke i386 config (the one that measured 87.5% via
    # test_snapshot_apps pristine_ref). vm.py's start() wraps qemu_args()
    # with the FAT extras: reconnect socket-serial, scratch qcow2, debugcon.
    cfg = get_arch_config("i386")
    cfg.desktop_settle_seconds = 20.0
    cfg.boot_timeout = 900.0  # heavy external QEMU contention on this host

    work = Path("/tmp/bench_threads_work")
    work.mkdir(parents=True, exist_ok=True)
    vm = QemuVM(cfg, work)

    print("=== boot i386 via FAT vm.py/BenchConfig config ===")
    t0 = time.monotonic()
    vm.start()
    vm.wait_for_boot()
    pid = vm._proc.pid
    print(f"  booted {time.monotonic() - t0:.0f}s; pid={pid}")
    print(f"  cmdline: {' '.join(vm._proc.args)}\n")

    try:
        vcpu = vm.qmp.execute("query-cpus-fast")["return"][0]["thread-id"]
        print(f"  vCPU host thread-id = {vcpu}")

        def vcpu_pct(window: float) -> float:
            def j():
                p = Path(f"/proc/{pid}/task/{vcpu}/stat").read_text().rsplit(")", 1)[1].split()
                return int(p[11]) + int(p[12])

            a = j()
            time.sleep(window)
            b = j()
            return (b - a) / _CLK / window * 100.0

        # CONFIRMING A/B on the REAL smoke config — only var: drain serial?
        # Hypothesis: ReactOS polled serial + DbgPrint spew + undrained
        # socket chardev backpressure => guest busy-waits on UART THRE.
        print("\n  [A] serial NOT drained (harness post-boot state):")
        a_pct = vcpu_pct(6.0)
        print(f"      vCPU = {a_pct:.1f}%")

        print("  [B] continuously draining serial socket:")
        stop = threading.Event()

        def drain():
            while not stop.is_set():
                try:
                    vm.serial.read_available(timeout=0.2)
                except Exception:
                    time.sleep(0.1)

        dt = threading.Thread(target=drain, daemon=True)
        dt.start()
        time.sleep(2)  # let the backlog flush
        b_pct = vcpu_pct(6.0)
        stop.set()
        dt.join(timeout=2)
        print(f"      vCPU = {b_pct:.1f}%")

        print(
            f"\n  VERDICT: undrained={a_pct:.1f}%  drained={b_pct:.1f}%  "
            f"(lean start.sh baseline 3.4%)"
        )
        if a_pct > 50 and b_pct < 20:
            print(
                "  => CONFIRMED: the 'idle storm' is SERIAL BACKPRESSURE — "
                "ReactOS polled-serial DbgPrint spinning on UART THRE because "
                "the harness stops draining the socket chardev post-boot. "
                "NOT an OS bug. Fix: drain serial continuously, or use "
                "-serial file:, or unblock the chardev."
            )
        elif a_pct > 50 and b_pct > 50:
            print(
                "  => NOT (only) serial; vCPU still hot when drained — "
                "another vmexit source remains."
            )
        else:
            print("  => inconclusive (undrained not hot this sample).")
    finally:
        vm.stop()


if __name__ == "__main__":
    main()
