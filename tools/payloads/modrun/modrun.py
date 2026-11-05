#!/usr/bin/env python3
"""Compatibility facade for the native modrun suite compiler and grader."""

import argparse
import re as re
import sys
from dataclasses import dataclass, field
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
SUITES = HERE / "suites"
if str(REPO) not in sys.path:
    sys.path.insert(0, str(REPO))
from tools.build_infra_compat import call  # noqa: E402


@dataclass
class Case:
    id: str
    suite: str
    variants: dict
    args: str = ""
    stdin: list = field(default_factory=list)
    expect: list = field(default_factory=list)
    timeout: int = 60
    normalize: list = field(default_factory=list)
    xfail: str = ""
    diverge: list = field(default_factory=list)


def _call(op, **kwargs):
    try:
        return call("payload_providers_" + op, {"suites": SUITES, **kwargs}, REPO)
    except ValueError as e:
        raise SystemExit(str(e)) from e


def load_suites():
    return dict(_call("load_suites"))


def cases(arch):
    out = []
    for c in _call("cases", arch=arch):
        c["variants"] = dict(c["variants"])
        c["normalize"] = [tuple(x) for x in c["normalize"]]
        c["diverge"] = [(why, pats) for why, pats in c["diverge"]]
        out.append(Case(**c))
    return out


def case_file(arch):
    return _call("case_file", arch=arch)


def stage_entries(arch):
    return _call("stage_entries", spec="modrun", arch=arch)


def ninja_targets(arch):
    return _call("ninja_targets", spec="modrun", arch=arch)


def checksum(text, seq):
    return _call("checksum", text=text, seq=seq)


def _wire_serial(serial):
    try:
        serial.encode("utf-8")
        return serial
    except UnicodeEncodeError:
        return serial.encode("utf-8", errors="surrogateescape").decode("latin-1")


def parse(serial):
    out, exits = _call("parse", serial=_wire_serial(serial))
    return ({(a, b): v for a, b, v in out}, {(a, b): tuple(v) for a, b, v in exits})


def grade(serial, arch):
    return tuple(_call("grade", serial=_wire_serial(serial), arch=arch))


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__)
    sub = ap.add_subparsers(dest="cmd", required=True)
    for name in ("gen", "ninja"):
        sub.add_parser(name).add_argument("--arch", required=True)
    g = sub.add_parser("grade")
    g.add_argument("log", type=Path)
    g.add_argument("--arch", required=True)
    args = ap.parse_args(argv)
    if args.cmd == "gen":
        path = HERE / f"modrun-{args.arch}.txt"
        path.write_text(case_file(args.arch))
        print(f"modrun: {path.name}: {len(cases(args.arch))} case(s)")
    elif args.cmd == "ninja":
        print(" ".join(ninja_targets(args.arch)))
    else:
        report, failures = grade(
            args.log.read_bytes().decode("utf-8", errors="surrogateescape"), args.arch
        )
        print("\n".join(report))
        count = sum(r.startswith(("PASS", "FAIL", "XFAIL", "XPASS")) for r in report)
        print(f"modrun: {count} case(s), {len(failures)} failed")
        return int(bool(failures))
    return 0


if __name__ == "__main__":
    sys.exit(main())
