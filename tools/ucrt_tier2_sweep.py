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

#: The durable verdict table. Before this existed the table lived only in
#: /tmp/vibeos-cli run dirs, which the reaper deletes -- the first full 188-export
#: sweep was gone within two days and the next step (reusing the MATCH bodies)
#: had nothing to read. Rows are a UNION keyed by export, each stamped with the
#: commit it was graded at, so a partial re-run refreshes its rows without
#: pretending the rest were re-graded.
TABLE = ROOT / "docs" / "engineering" / "active-lanes" / "ucrt-tier2-table.tsv"

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


#: The kernel's thread-exit trace for the harness process. A run with no `done`
#: line either hung or DIED, and only this says which -- and with what status.
EXIT_RE = re.compile(
    r"ExitStatus=0x(?:FFFFFFFF)?([0-9A-Fa-f]{8})\S*?\[PS\] PspExitThread: Teb=\S+?"
    r"\[PS\] PspExitThread: StartAddress=\S+?\[PS\] PspExitThread: Image=ucrtdiff\.exe"
)


def harness_exit_status(text: str) -> str | None:
    """`0xC0000194`-style status of the harness's last non-zero thread exit, if any."""
    hits = [h for h in EXIT_RE.findall(text) if int(h, 16) != 0]
    return f"0x{hits[-1].upper()}" if hits else None


def load_table(path: Path) -> dict[str, tuple[str, str]]:
    """export -> (verdict tail, commit) from a table written by write_table."""
    rows: dict[str, tuple[str, str]] = {}
    if not path.exists():
        return rows
    for line in path.read_text().splitlines():
        if not line or line.startswith("#"):
            continue
        name, verdict, commit = line.split("\t")
        rows[name] = (verdict, commit)
    return rows


def write_table(path: Path, rows: dict[str, tuple[str, str]]) -> None:
    lines = [
        "# ucrt Tier 2 verdicts -- written by tools/ucrt_tier2_sweep.py, do not hand-edit.",
        "# export\tverdict (+ counts)\tcommit graded at",
    ]
    lines += [f"{n}\t{v}\t{c}" for n, (v, c) in sorted(rows.items())]
    path.write_text("\n".join(lines) + "\n")


def run_shard(
    arch: str,
    k: int,
    n: int,
    budget_ms: int,
    dry_run: bool,
    takeover_check: bool = False,
    reuse_inputs: bool = False,
) -> str:
    """Build a shard-k harness + media and boot it; return the serial text.

    ORDER MATTERS. build.sh links the harness from whatever the build tree already
    holds -- the ref_ objects, the rust_ archive, the DLL's import lib -- and the
    media build after it is what runs ninja. In that order a Rust edit is graded
    one build LATE while the run reads as current. So by default ninja runs FIRST.

    `takeover_check` keeps the old order on purpose: build.sh then sees the tree as
    it was BEFORE the pending build, so its ref_ side still holds Microsoft's
    objects for the sources a takeover just deleted, while the DLL the media build
    then produces already answers those exports with our Rust. That is the one run
    that grades the SHIPPING takeover against the code it replaced.

    But only the FIRST shard of such a run sees that tree: its media build is what
    drops the deleted family's objects. So every later shard of a takeover check
    passes `reuse_inputs`, and build.sh keeps the first shard's reference set, rust
    set and case list -- none of which depends on the shard. Rebuilding them there
    loses Microsoft's bodies from ref_ and, the burn-down now calling the exports
    ours, drops them from the case list: the later shards grade less, silently.
    """
    env = dict(os.environ, UCRTDIFF_SHARD=f"{k}/{n}", UCRTDIFF_BUDGET_MS=str(budget_ms))
    if reuse_inputs:
        env["UCRTDIFF_REUSE_INPUTS"] = "1"
    build = ["bash", "tools/payloads/ucrtdiff/build.sh", arch]
    media = ["./vibeos-cli", "build", "--arch", arch]
    smoke = ["./vibeos-cli", "smoke", "--arch", arch, "--payload", "ucrtdiff"]
    if dry_run:
        print(f"  would run: UCRTDIFF_SHARD={k}/{n} {' '.join(build)}")
        return ""
    run_dir = None
    # The pre-build is a DISTINCT list object (so `is media` is false for it) and runs
    # WITHOUT VIBEOS_PAYLOAD: it only needs ninja. Staging the payload there demands
    # an exe that the harness build has not produced yet -- and a failed link deletes
    # the old one -- so it failed RESTAGE_FAIL whenever the previous attempt had.
    prebuild = list(media)
    steps = (build, media, smoke) if takeover_check else (prebuild, build, media, smoke)
    for cmd in steps:
        e = dict(env, VIBEOS_PAYLOAD="ucrtdiff") if cmd is media else env
        # The smoke verb exits non-zero on a payload FAIL, which for a sharded
        # sweep is information, not a reason to stop: collect the log either way.
        proc = subprocess.run(cmd, cwd=ROOT, env=e, capture_output=True, text=True)
        # stderr too, always: the generator announces the QUARANTINE list there, and
        # a failing stage explains itself there.
        out = (proc.stdout or "") + (proc.stderr or "")
        print(out, end="")
        # Raise only AFTER printing: check=True raised from inside run(), so a failed
        # media build died as a bare CalledProcessError with its reason never shown.
        if cmd is not smoke and proc.returncode != 0:
            raise SystemExit(f"shard {k}/{n}: {' '.join(cmd)} failed (rc={proc.returncode})")
        if cmd is smoke:
            # Take the artifact dir the CLI REPORTS, never the newest one matching a
            # glob. Other sessions run arm64 smokes concurrently -- three foreign dirs
            # appeared inside five minutes during one sweep -- and an mtime pick
            # silently attributes THEIR boot to THIS shard. That is how shard 4 came
            # back "0 exports graded" from a run it never launched.
            m = re.search(r"(/tmp/vibeos-cli/\S*-smoke-\S+)/full\.log", out)
            if m:
                run_dir = Path(m.group(1))
    if run_dir is None:
        raise SystemExit(
            f"shard {k}/{n}: the smoke verb never reported its artifact dir; refusing "
            f"to guess one by mtime (a foreign run would be scored as this shard)"
        )
    # The harness writes to the kernel debug serial, so its rows are in the SERIAL log,
    # never in full.log (which holds the CLI's own stage output). Reading full.log here
    # returned zero rows for every shard and would have reported a clean sweep as an
    # empty one. `serial.log` is the drained capture the payload gate asserts against;
    # `boot0/serial.log` is the live tail, which can be partial -- so prefer whichever
    # actually carries more rows rather than assuming either.
    #
    # `payload/<arch>_serial.log` is the payload boot's own capture and the ONLY one
    # that held the harness's death in the run that exposed this: the other two
    # stopped ~1.5KB short, before `ExitStatus=...C0000194`, so they reported a
    # silent stall where the process had in fact died in a critical section.
    best = ""
    for cand in (
        run_dir / "serial.log",
        run_dir / "boot0" / "serial.log",
        run_dir / "payload" / f"{arch}_serial.log",
    ):
        if not cand.exists():
            continue
        text = cand.read_text(errors="replace")
        if (text.count("[UCRTDIFF] EXPORT"), len(text)) >= (
            best.count("[UCRTDIFF] EXPORT"),
            len(best),
        ):
            best = text
    return best


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
    ap.add_argument(
        "--takeover-check",
        action="store_true",
        help="build the harness from the tree as it was BEFORE this build, so ref_ "
        "still has Microsoft's objects for sources a takeover just deleted (see run_shard)",
    )
    ap.add_argument(
        "--table",
        type=Path,
        default=TABLE,
        help="durable verdict table to merge this sweep's rows into",
    )
    args = ap.parse_args(argv)

    wanted = args.only if args.only else list(range(args.shards))
    merged: dict[str, str] = {}
    incomplete: list[int] = []
    for k in wanted:
        print(f"[sweep] shard {k}/{args.shards}")
        text = run_shard(
            args.arch,
            k,
            args.shards,
            args.budget_ms,
            args.dry_run,
            args.takeover_check,
            reuse_inputs=args.takeover_check and k != wanted[0],
        )
        rows, done = parse_serial(text)
        merged.update(rows)
        if done is None or done.get("partial", "0") != "0":
            incomplete.append(k)
            why = "no done line" if done is None else f"partial={done['partial']}"
            death = harness_exit_status(text)
            if death:
                why += f"; ucrtdiff.exe exited {death}"
            print(f"[sweep]   INCOMPLETE ({why}), {len(rows)} exports graded")
        else:
            print(f"[sweep]   ok, {len(rows)} exports graded")

    if args.dry_run:
        return 0

    commit = subprocess.run(
        ["git", "rev-parse", "--short=12", "HEAD"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    # A dirty tree is NOT that commit: stamping it bare would attribute rows to code
    # that never produced them. The table file itself is excluded, since writing it
    # is what this step does.
    dirty = subprocess.run(
        ["git", "status", "--porcelain", "--", ".", f":!{args.table.relative_to(ROOT)}"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if dirty:
        commit += "-dirty"
    table = load_table(args.table)
    table.update({name: (verdict, commit) for name, verdict in merged.items()})
    write_table(args.table, table)
    print(f"[sweep] {len(merged)} row(s) merged into {args.table} ({len(table)} total)")

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
