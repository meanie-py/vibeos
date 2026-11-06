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
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
if str(REPO) not in sys.path:
    sys.path.insert(0, str(REPO))
from tools import qemu_infra_compat as native  # noqa: E402
from tools.qemu_infra_runner import run_controller  # noqa: E402

GUEST_WENT_SILENT = "[payload] drain-end: idle"
SAME_SUITE_LIMIT = 3
PYTEST_ARCH = {"i386": "i386-uefi", "amd64": "amd64-uefi", "arm64": "arm64"}


def should_quarantine(bugcheck: str | None, harness_out: str) -> bool:
    return native.call(
        "characterize_should_quarantine", {"bugcheck": bugcheck, "harness": harness_out}, REPO
    )


def archive(serial: Path, run_dir: Path, rnd: int) -> Path:
    return Path(
        native.call(
            "characterize_archive", {"serial": serial, "run_dir": run_dir, "round": rnd}, REPO
        )
    )


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--arch", required=True, choices=tuple(PYTEST_ARCH))
    ap.add_argument("--rounds", type=int, default=12)
    ap.add_argument(
        "--artifact-dir", type=Path, help="parent directory for unique per-run evidence"
    )
    ap.add_argument("--out", type=Path, help="merged results table")
    ap.add_argument("--only", action="append", metavar="SUITE.exe")
    ap.add_argument("--ignore-quarantine", action="store_true")
    ap.add_argument("--no-quarantine", action="store_true")
    args = ap.parse_args(argv)
    return run_controller("characterize_run", args, REPO, command="characterize", arch=args.arch)


if __name__ == "__main__":
    raise SystemExit(main())
