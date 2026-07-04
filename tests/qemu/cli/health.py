"""`vibeos-cli health` — trailing pass rates from the durable outcome ledger (T3).

The gate is a point sample against a stochastic system: an arch at 20% health
still rolls a green gate 1 time in 5, and that is exactly how a broken arm64
landed and sat on master for a day. The ledger (vibeos-ledger.jsonl under
VIBEOS_ROOT, appended by every finalized Run across all worktrees) turns those
samples into rates: `vibeos-cli health` shows, per arch, the smoke pass rate
over a trailing window and over the last 10 attempts — a 100%→20% drop is
visible data within a day regardless of individual gate luck.

Scope note: this owns outcome RECORDING/trending. Verdict quality (evidence-
based retries, panic-aware triage) is the ghosts-writeup lane's program — the
two compose, they don't overlap.
"""

from __future__ import annotations

import json
import time
from pathlib import Path

from . import cachecfg

WINDOW_DAYS_DEFAULT = 7
RECENT_N = 10
ALERT_RATE = 0.8  # last-10 rate below this flags the arch


def _load(repo: Path, days: float) -> list[dict]:
    path = cachecfg.vibeos_root(repo) / "vibeos-ledger.jsonl"
    if not path.is_file():
        return []
    cutoff = time.time() - days * 86400
    rows = []
    for line in path.read_text(errors="replace").splitlines():
        try:
            row = json.loads(line)
        except ValueError:
            continue  # torn/foreign line — ledger is append-only telemetry
        if row.get("ts", 0) >= cutoff:
            rows.append(row)
    return rows


def _rate(rows: list[dict]) -> tuple[int, int, float]:
    ok = sum(1 for r in rows if r.get("verdict") in ("OK", "WARN"))
    n = len(rows)
    return ok, n, (ok / n if n else 1.0)


def health(args, repo: Path) -> int:
    days = float(getattr(args, "days", WINDOW_DAYS_DEFAULT) or WINDOW_DAYS_DEFAULT)
    rows = _load(repo, days)
    if getattr(args, "json", False):
        print(json.dumps(rows))
        return 0
    if not rows:
        print(
            f"health: no ledger rows in the last {days:g}d "
            f"({cachecfg.vibeos_root(repo) / 'vibeos-ledger.jsonl'})"
        )
        return 0

    smokes = [r for r in rows if r.get("cmd") == "smoke"]
    builds = [r for r in rows if r.get("cmd") == "build"]
    gates = [r for r in rows if r.get("cmd") == "gate"]

    print(f"health (trailing {days:g}d, {len(rows)} runs across all worktrees)")
    alerts: list[str] = []
    arches = sorted({r.get("arch", "?") for r in smokes})
    for arch in arches:
        arows = sorted((r for r in smokes if r.get("arch") == arch), key=lambda r: r.get("ts", 0))
        ok, n, rate = _rate(arows)
        rok, rn, rrate = _rate(arows[-RECENT_N:])
        fails = [
            r.get("classification") or "FAIL"
            for r in arows
            if r.get("verdict") not in ("OK", "WARN")
        ]
        top_fail = max(set(fails), key=fails.count) if fails else ""
        flag = ""
        if rn >= 3 and rrate < ALERT_RATE:
            flag = "  ⚠ UNHEALTHY"
            alerts.append(
                f"{arch}: last-{rn} smoke rate {rok}/{rn} "
                f"({rrate * 100:.0f}%) — dominant failure: {top_fail or 'n/a'}"
            )
        extra = f"  dominant-fail={top_fail}" if top_fail else ""
        print(
            f"  smoke {arch:<14} {ok:>3}/{n:<3} ({rate * 100:3.0f}%)  "
            f"last-{min(rn, RECENT_N)}: {rok}/{rn} ({rrate * 100:3.0f}%){extra}{flag}"
        )
    if builds:
        bok, bn, brate = _rate(builds)
        print(f"  build (all arches)  {bok:>3}/{bn:<3} ({brate * 100:3.0f}%)")
    if gates:
        gok, gn, grate = _rate(gates)
        print(f"  gate  (full)        {gok:>3}/{gn:<3} ({grate * 100:3.0f}%)")
    if alerts:
        print("alerts:")
        for a in alerts:
            print(f"  ⚠ {a}")
        print(
            "  (an arch can still roll a green gate at these rates — treat the RATE "
            "as the truth, not the last gate)"
        )
    return 0
