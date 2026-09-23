#!/usr/bin/env python3
"""Run the Tier 2 ucrt difftest as N sharded boots and merge the result.

Why this exists rather than one long boot: the harness used to grade all 220
exports in a single process, and whatever ended a long run cost every export
after it. The stopping point moved with emission order -- 114 exports forward,
53 reversed -- so every table we produced was some PREFIX of the truth and no
two prefixes agreed. That is not a flaky test, it is an unusable oracle.

Sharding sidesteps the cause instead of waiting to find it. Shard k of N grades
exports k, k+N, k+2N..., which is a run small enough to finish inside any wall
we have observed, and N boots cover the whole set. The interleave is deliberate:
a contiguous block would put a whole family, and so a whole failure mode, in one
shard.

Each shard's serial log is parsed for `[UCRTDIFF] EXPORT <name> <verdict>` rows
and for the `done` line's `partial=` field. A shard whose done line is missing,
or which reports partial>0, is reported as INCOMPLETE and its exports are listed
as ungraded rather than silently dropped -- the whole point is that a short run
must never read as a clean one.
"""

from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

EXPORT_RE = re.compile(r"\[UCRTDIFF\] EXPORT (\S+) (\S+)(.*)")
DONE_RE = re.compile(r"\[UCRTDIFF\] done (.*)")


def parse_serial(text: str) -> tuple[dict[str, str], dict[str, str] | None]:
    """Return (export -> verdict line tail, done-fields or None)."""
    rows: dict[str, str] = {}
    done: dict[str, str] = {}
    for line in text.splitlines():
        m = EXPORT_RE.search(line)
        if m:
            rows[m.group(1)] = (m.group(2) + m.group(3)).strip()
            continue
        m = DONE_RE.search(line)
        if m:
            for tok in m.group(1).split():
                if "=" in tok:
                    k, _, v = tok.partition("=")
                    done[k] = v
    return rows, (done or None)


def run_shard(arch: str, k: int, n: int, budget_ms: int, dry_run: bool) -> str:
    """Build a shard-k harness + media and boot it; return the serial text."""
    env = dict(os.environ, UCRTDIFF_SHARD=f"{k}/{n}", UCRTDIFF_BUDGET_MS=str(budget_ms))
    build = ["bash", "tools/payloads/ucrtdiff/build.sh", arch]
    media = ["./vibeos-cli", "build", "--arch", arch]
    smoke = ["./vibeos-cli", "smoke", "--arch", arch, "--payload", "ucrtdiff"]
    if dry_run:
        print(f"  would run: UCRTDIFF_SHARD={k}/{n} {' '.join(build)}")
        return ""
    for cmd in (build, media, smoke):
        e = dict(env, VIBEOS_PAYLOAD="ucrtdiff") if cmd is media else env
        # The smoke verb exits non-zero on a payload FAIL, which for a sharded
        # sweep is information, not a reason to stop: collect the log either way.
        subprocess.run(cmd, cwd=ROOT, env=e, check=(cmd is not smoke))
    runs = sorted(Path("/tmp/vibeos-cli").glob(f"*-smoke-{arch}"), key=lambda p: p.stat().st_mtime)
    if not runs:
        raise SystemExit(f"shard {k}/{n}: no artifact dir under /tmp/vibeos-cli")
    log = runs[-1] / "full.log"
    return log.read_text(errors="replace") if log.exists() else ""


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument(
        "--arch",
        default="arm64",
        help="Tier 2 only really runs on arm64: VEH first-chance dispatch "
        "is dead code on i386/amd64 (rtl-except), and the fault guard "
        "refuses to grade without it.",
    )
    ap.add_argument("--shards", type=int, default=8)
    ap.add_argument("--budget-ms", type=int, default=420000)
    ap.add_argument(
        "--only",
        type=int,
        action="append",
        help="run only this shard index (repeatable); default is all",
    )
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args(argv)

    wanted = args.only if args.only else list(range(args.shards))
    merged: dict[str, str] = {}
    incomplete: list[int] = []
    for k in wanted:
        print(f"[sweep] shard {k}/{args.shards}")
        text = run_shard(args.arch, k, args.shards, args.budget_ms, args.dry_run)
        rows, done = parse_serial(text)
        merged.update(rows)
        if done is None or done.get("partial", "0") != "0":
            incomplete.append(k)
            why = "no done line" if done is None else f"partial={done['partial']}"
            print(f"[sweep]   INCOMPLETE ({why}), {len(rows)} exports graded")
        else:
            print(f"[sweep]   ok, {len(rows)} exports graded")

    if args.dry_run:
        return 0

    tally: dict[str, int] = {}
    for verdict in merged.values():
        tally[verdict.split()[0]] = tally.get(verdict.split()[0], 0) + 1
    print(f"\n[sweep] {len(merged)} exports graded across {len(wanted)} shard(s)")
    for verdict, count in sorted(tally.items(), key=lambda kv: -kv[1]):
        print(f"  {verdict:<8} {count}")
    if incomplete:
        print(f"[sweep] INCOMPLETE shards: {incomplete} -- their exports are NOT all graded")
    return 1 if incomplete else 0


if __name__ == "__main__":
    sys.exit(main())
