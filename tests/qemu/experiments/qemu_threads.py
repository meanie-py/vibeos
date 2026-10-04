"""Tier-3b telemetry: per-QEMU-thread host-CPU breakdown.

cpu_pct() sums ALL qemu threads, which can't tell "guest vCPU is busy"
(guest/hlt bug) from "a device-emulation thread is hot" (QEMU-side) from
"vCPU exits constantly on PIO/MMIO" (vmexit storm shows as the KVM vCPU
thread hot even though the guest looks idle).

QEMU names its threads in /proc/<pid>/task/<tid>/comm:
  "CPU 0/KVM"  = the vCPU thread (guest execution + vmexits)
  "qemu-system-*" = main loop / device emulation / iothread

Usage:  python -m tests.qemu.experiments.qemu_threads <run-id> [window_s]
"""

from __future__ import annotations

import json
import os
import sys
import time
from pathlib import Path

from tests.qemu.vm import PROJECT_DIR

_CLK = os.sysconf("SC_CLK_TCK")


def _qemu_pid(run: str) -> int:
    mf = PROJECT_DIR / "qemu" / "runs" / run / "manifest.json"
    data = json.loads(mf.read_text())
    for k in ("qemu_pid", "pid", "qemuPid"):
        if k in data:
            return int(data[k])
    raise SystemExit(f"no qemu pid in {mf}: keys={list(data)}")


def _threads(pid: int) -> dict[int, tuple[str, int]]:
    """tid -> (comm, utime+stime jiffies)."""
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
    run = sys.argv[1]
    window = float(sys.argv[2]) if len(sys.argv) > 2 else 3.0
    pid = _qemu_pid(run)

    a = _threads(pid)
    time.sleep(window)
    b = _threads(pid)

    print(f"=== qemu-threads {run}  pid={pid}  window={window}s ===")
    rows = []
    total = 0.0
    for tid, (comm, j1) in b.items():
        j0 = a.get(tid, (comm, j1))[1]
        pct = (j1 - j0) / _CLK / window * 100.0
        total += pct
        rows.append((pct, tid, comm))
    rows.sort(reverse=True)
    for pct, tid, comm in rows:
        bar = "#" * int(pct / 2)
        print(f"  tid={tid:<7} {comm:<18} {pct:6.1f}% {bar}")
    print(f"  {'TOTAL':<27}{total:6.1f}% of one core")

    hot = rows[0] if rows else None
    if hot:
        _, _, comm = hot
        if "KVM" in comm or "CPU" in comm:
            print(
                f"\nVERDICT: the vCPU thread ({comm}) is hot ⇒ guest-side: "
                f"either hlt is being woken (re-check exit reasons / pending "
                f"vector) OR a guest PIO/MMIO poll loop causes a vmexit storm "
                f"(KVM vCPU thread burns host CPU even with guest 'idle')."
            )
        else:
            print(
                f"\nVERDICT: a non-vCPU QEMU thread ({comm}) is hot ⇒ "
                f"QEMU device-emulation hot loop, NOT a guest bug. The guest "
                f"is genuinely idle; the host burn is QEMU emulating some "
                f"device (likely whatever the guest left actively polling)."
            )

    # KVM exit-reason stats if debugfs is readable (often needs privilege).
    kdir = Path("/sys/kernel/debug/kvm")
    if kdir.exists() and os.access(kdir, os.R_OK):
        try:
            stats = {}
            for f in (
                "halt_exits",
                "io_exits",
                "mmio_exits",
                "irq_exits",
                "halt_poll_success_ns",
                "halt_wakeup",
            ):
                p = kdir / f
                if p.exists():
                    stats[f] = p.read_text().strip()
            if stats:
                print(f"\n[kvm debugfs] {stats}")
        except Exception:
            pass


if __name__ == "__main__":
    main()
