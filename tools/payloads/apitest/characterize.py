#!/usr/bin/env python3
"""Drive the apitest sweep to completion, quarantining whatever stops it.

    python3 tools/payloads/apitest/characterize.py --arch amd64 [--rounds 12]

A boot of this harness ends when some subtest takes the guest with it: a
BUGCHECK, or a crash-loop that kills services.exe and never returns. The fix
each time is the same three steps -- work out which subtest was in flight, add
it to quarantine-<arch>.txt, rebuild and boot again -- and doing that by hand is
a recipe someone has to remember, which is the kind of thing that belongs in a
tool rather than in a doc.

Each round: regenerate the suite/skip lists, build media with
VIBEOS_PAYLOAD=apitest, boot, parse. If the sweep reached
`[APITEST-SWEEP] done`, stop. Otherwise the last subtest with no `exit` line is
the culprit; append it to the quarantine with the round's evidence and go again.
It stops early rather than looping forever if a round makes no progress, because
a round that quarantines something and still dies in the same place means the
attribution is wrong and more rounds will not fix it.

Results are accumulated ACROSS rounds -- no single boot sees every subtest while
crashers remain -- and written as one merged table. Note what that means for a
baseline seeded from it: the rows come from different boots, so treat a count
as commentary and the assertion identities as the floor, exactly as ratchet.py
already says.
"""

from __future__ import annotations

import argparse
import datetime
import importlib.util
import os
import shutil
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
VENV_PY = REPO.parent / "vibeos" / ".venv" / "bin" / "python"

PYTEST_ARCH = {"i386": "i386-uefi", "amd64": "amd64-uefi", "arm64": "arm64"}


def _load(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def archive(serial: Path, run_dir: Path, rnd: int) -> Path:
    """Copy a round's serial somewhere it will still exist tomorrow.

    The obvious thing -- record the path pytest used -- is wrong twice over, and
    this tool shipped doing it. `/tmp/pytest-of-*/pytest-current` is a symlink
    that MOVES to the newest run, so by round 2 every earlier round's recorded
    "evidence" pointed at a different boot; pytest then prunes to the last three
    runs; and a reboot wipes /tmp entirely, which is how rounds 1-3 of the first
    real characterisation lost their serials for good. conformance.py already
    learned this and passes VIBEOS_ARTIFACT_DIR for the same reason.
    """
    run_dir.mkdir(parents=True, exist_ok=True)
    dest = run_dir / f"round{rnd:02d}-{serial.name}"
    try:
        shutil.copy2(serial, dest)
    except OSError as exc:  # pragma: no cover - disk full, permissions
        print(f"[characterize] WARNING: could not archive {serial}: {exc}", file=sys.stderr)
        return serial.resolve()
    return dest


def newest_serial(arch: str) -> Path | None:
    pat = f"qemu-{PYTEST_ARCH[arch]}*/{PYTEST_ARCH[arch]}_serial.log"
    logs = sorted(
        Path("/tmp").glob(f"pytest-of-*/pytest-*/{pat}"),
        key=lambda p: p.stat().st_mtime,
    )
    return logs[-1] if logs else None


def run(cmd: list[str], **kw) -> int:
    print(f"[characterize] $ {' '.join(str(c) for c in cmd)}", flush=True)
    return subprocess.run(cmd, cwd=str(REPO), **kw).returncode


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--arch", required=True, choices=tuple(PYTEST_ARCH))
    ap.add_argument("--rounds", type=int, default=12)
    ap.add_argument(
        "--artifact-dir",
        default=None,
        help="where each round's serial is archived (default: a timestamped dir "
        "under /tmp/vibeos-cli). Point this somewhere persistent if the run "
        "matters -- /tmp does not survive a reboot",
    )
    ap.add_argument(
        "--out",
        type=Path,
        default=None,
        help="merged results table (default: tools/conformance-baseline/<arch>/apitest.txt.new)",
    )
    args = ap.parse_args(argv)

    parser = _load(
        "parse_apitest_serial", REPO / "tools" / "conformance" / "parse_apitest_serial.py"
    )
    py = str(VENV_PY) if VENV_PY.exists() else sys.executable
    quarantine = HERE / f"quarantine-{args.arch}.txt"
    stamp = datetime.datetime.now().strftime("%Y%m%d-%H%M%S")
    run_dir = Path(args.artifact_dir or f"/tmp/vibeos-cli/{stamp}-characterize-{args.arch}")
    print(f"[characterize] artifacts: {run_dir}", flush=True)
    merged: dict[str, dict] = {}
    culprits: list[str] = []

    for rnd in range(1, args.rounds + 1):
        print(f"\n[characterize] ===== round {rnd}/{args.rounds} =====", flush=True)

        if run([py, str(HERE / "gen_suites.py"), "--arch", args.arch]) != 0:
            return 1

        env = dict(os.environ, VIBEOS_PAYLOAD="apitest")
        if run(["./vibeos-cli", "build", "--arch", args.arch, "--apitests"], env=env) != 0:
            print("[characterize] build failed — stopping", file=sys.stderr)
            return 1

        run(
            [
                py,
                "-m",
                "pytest",
                "tests/qemu/test_payload.py",
                "--arch",
                PYTEST_ARCH[args.arch],
                "--payload",
                "apitest",
                "--skip-clean-build",
                "--timeout",
                "7000",
                "-q",
            ]
        )

        serial = newest_serial(args.arch)
        if serial is None:
            print("[characterize] no serial log found — stopping", file=sys.stderr)
            return 1

        # Archive BEFORE parsing: everything below records this path as the
        # round's evidence, and it has to still be readable later.
        serial = archive(serial, run_dir, rnd)

        results, meta = parser.parse(serial.read_text(errors="replace"))
        for r in results:
            prev = merged.get(r["name"])
            # Keep the WORST verdict a subtest ever got: a later round where it
            # is skipped must not overwrite the round that caught it crashing.
            if prev is None or (r["status"] in parser.BAD and prev["status"] not in parser.BAD):
                merged[r["name"]] = r
        print(
            f"[characterize] round {rnd}: {len(results)} subtest(s), "
            f"{len(merged)} known; sweep {'done' if meta['done'] else 'INCOMPLETE'}",
            flush=True,
        )

        if meta["done"]:
            print("[characterize] sweep completed — characterisation finished.")
            break

        culprit = next(
            (r["name"] for r in reversed(results) if r["exit"] is None and r["status"] != "SKIP"),
            None,
        )
        if culprit is None:
            print("[characterize] sweep incomplete but no in-flight subtest — stopping.")
            break
        if culprit in culprits:
            print(
                f"[characterize] {culprit} was already quarantined and the sweep died "
                "in the same place; the attribution is wrong. Stopping rather than "
                "looping.",
                file=sys.stderr,
            )
            break

        culprits.append(culprit)
        why = f"BUGCHECK {meta['bugcheck']}" if meta["bugcheck"] else "guest wedged (no BUGCHECK)"
        with quarantine.open("a", encoding="utf-8") as fh:
            fh.write(
                f"\n# characterize.py round {rnd}: sweep stopped here — {why}.\n"
                f"# serial: {serial}\n{culprit}\n"
            )
        print(f"[characterize] quarantined {culprit!r} ({why})", flush=True)

    out = args.out or (REPO / "tools" / "conformance-baseline" / args.arch / "apitest.txt.new")
    out.parent.mkdir(parents=True, exist_ok=True)
    rows = [parser.fmt(merged[n]) for n in sorted(merged)]
    out.write_text(
        "# apitest conformance results — parse_apitest_serial.py format\n"
        "# MERGED ACROSS ROUNDS by characterize.py: no single boot sees every\n"
        "# subtest while crashers remain, so counts are commentary and the\n"
        "# assertion identities are the floor.\n" + "\n".join(rows) + "\n"
    )
    bad = sum(1 for r in merged.values() if r["status"] in parser.BAD)
    print(f"\n[characterize] wrote {out} — {len(merged)} subtest(s), {bad} bad")
    if culprits:
        print("[characterize] quarantined this session: " + ", ".join(culprits))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
