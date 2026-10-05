"""Tier-3 perf telemetry: name the runaway IRQ behind the idle-storm.

Zero guest perturbation: QEMU HMP `info irq` carries cumulative per-IRQ
counts; diffing over a wall-clock window = interrupts/sec/IRQ. The IRQ
with a multi-kHz rate at an *idle* desktop is the storm source.

Usage:  python -m tests.qemu.experiments.irq_stat <run-id>
(<run-id> from qemu/start.sh; reads qemu/runs/<run-id>/qmp.sock)
"""

from __future__ import annotations

import re
import sys
import time

from tests.qemu.qmp_client import QmpClient
from tests.qemu.vm import PROJECT_DIR

# QEMU `info irq` format: per-controller blocks, then ` <irq>: <count>` lines.
#   IRQ statistics for kvm-i8259:
#    15: 8365
# Same IRQ appears under multiple controllers (ioapic + i8259) with the same
# underlying count — take the MAX per IRQ across controllers, don't sum.
_IRQ_LINE = re.compile(r"^\s*(\d+):\s+(\d+)\s*$", re.M)


def _irq_counts(q: QmpClient) -> dict[int, int]:
    txt = q.hmp("info irq")
    counts: dict[int, int] = {}
    for n, c in _IRQ_LINE.findall(txt):
        n, c = int(n), int(c)
        counts[n] = max(counts.get(n, 0), c)
    return counts


def main() -> None:
    run = sys.argv[1]
    sock = PROJECT_DIR / "qemu" / "runs" / run / "qmp.sock"
    if not sock.exists():
        sys.exit(f"no qmp sock at {sock}")

    q = QmpClient()
    q.connect(str(sock), timeout=10.0)

    print(f"=== irq-stat {run} ===")
    print("--- raw `info pic` ---")
    print(q.hmp("info pic").strip()[:900])
    print("\n--- raw `info irq` ---")
    print(q.hmp("info irq").strip()[:900])
    print("\n--- raw `info lapic` ---")
    lapic = q.hmp("info lapic")
    print(lapic.strip()[:1600])

    # LAPIC timer is internal to the vCPU (not an `info irq` line). If it is
    # periodic with a tiny initial count, it IS the idle-storm: it wakes the
    # `hlt` every iteration. Parse LVT Timer mode + DCR (divide) + ICT (init).
    lvt = re.search(
        r"(?im)^.*timer.*?(0x[0-9a-f]+).*?(periodic|one[- ]?shot|tsc).*?"
        r"(masked)?.*vec\s*[:=]?\s*(\d+)",
        lapic,
    )
    ict = re.search(r"(?i)(?:timer\s*)?ICT\s*[:=]?\s*(0x[0-9a-f]+|\d+)", lapic)
    dcr = re.search(r"(?i)(?:timer\s*)?DCR\s*[:=]?\s*(0x[0-9a-f]+|\d+)", lapic)
    cct = re.search(r"(?i)(?:timer\s*)?CCT\s*[:=]?\s*(0x[0-9a-f]+|\d+)", lapic)
    print("\n--- LAPIC timer parse ---")
    print(f"  LVT Timer line : {lvt.group(0).strip() if lvt else 'NOT FOUND'}")
    print(f"  ICT (init cnt) : {ict.group(1) if ict else '?'}")
    print(f"  DCR (divide)   : {dcr.group(1) if dcr else '?'}")
    print(f"  CCT (cur cnt)  : {cct.group(1) if cct else '?'}")
    if ict:
        ictv = int(ict.group(1), 0)
        periodic = bool(lvt and re.search(r"periodic", lvt.group(0), re.I))
        masked = bool(lvt and re.search(r"masked", lvt.group(0), re.I))
        if not masked and periodic and 0 < ictv < 100000:
            print(
                f"\nVERDICT: LAPIC TIMER STORM — periodic, unmasked, "
                f"initial-count={ictv} (tiny). This wakes the idle `hlt` "
                f"every iteration ⇒ ~88% host CPU. Root cause = HAL (Rust) "
                f"APIC timer programming: initial-count/divide set far too "
                f"small (or periodic where it should be one-shot)."
            )
        else:
            print(
                f"\nLAPIC timer: periodic={periodic} masked={masked} "
                f"ict={ictv} — inspect raw dump above for the real driver."
            )

    # Three 1s windows for stability.
    windows = []
    prev = _irq_counts(q)
    for _ in range(3):
        time.sleep(1.0)
        cur = _irq_counts(q)
        delta = {k: cur.get(k, 0) - prev.get(k, 0) for k in cur}
        windows.append(delta)
        prev = cur

    # Average rate per IRQ across windows.
    irqs = sorted({k for d in windows for k in d})
    print("\n--- interrupts/sec per IRQ (3x 1s windows) ---")
    rates: dict[int, float] = {}
    for irq in irqs:
        vals = [d.get(irq, 0) for d in windows]
        avg = sum(vals) / len(vals)
        rates[irq] = avg
        flag = "  <-- STORM" if avg > 1000 else ("  (normal tick)" if 50 <= avg <= 250 else "")
        print(f"  IRQ {irq:2d}: {vals}  avg={avg:8.1f}/s{flag}")

    top = max(rates, key=rates.get) if rates else None
    if top is not None and rates[top] > 1000:
        hint = {
            0: "PIT/system timer (IRQ0) — HAL clock programming: PIT divisor "
            "too small, or HalpClockInterrupt not re-arming/acking correctly",
            8: "RTC (IRQ8) — RTC periodic rate misprogrammed in HAL",
        }.get(
            top,
            f"device on IRQ{top} — its handler likely never EOIs/masks, "
            f"so the line re-asserts immediately (level-triggered storm)",
        )
        print(f"\nVERDICT: runaway IRQ{top} @ ~{rates[top]:.0f}/s — {hint}")
    else:
        print(
            "\nVERDICT: no single IRQ > 1000/s — storm may be NMI/IPI/exception, "
            "not a maskable line; re-check with bt at a non-idle sample."
        )

    q.close()


if __name__ == "__main__":
    main()
