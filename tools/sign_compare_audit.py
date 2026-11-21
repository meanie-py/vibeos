#!/usr/bin/env python3
"""Audit the inert (cohab-flipped) C for mixed-sign min/max calls.

C's usual arithmetic conversions make `min(int, UINT)` an UNSIGNED compare;
a Rust translation that casts one side so `min` type-checks can flip it to a
signed compare (rebar's 8 px taskbar, user32 listbox's LB_SETCOUNT clamp).
No source lint can see this, because the C operand types are needed. This
recovers them: every flipped `.c` that calls min/max is parsed by clang with
`-Wsign-compare`, borrowing the compile flags of the nearest live TU in
`build-<arch>/compile_commands.json` (flipped modules usually have none of
their own), and the warnings that fall on a min/max are printed for review.

It is an AUDIT, not a gate: each hit must be read against its Rust port.
Most are loop bounds or operands guarded non-negative. Run all three arches,
since SIZE_T/LONG widths move which compares are mixed-sign.

    python3 tools/sign_compare_audit.py --arch amd64 [--jobs 8]

`#ERR n file` lines are TUs that only partly parsed under borrowed flags:
functions after the error may be unchecked, so treat them as reduced coverage.
"""

from __future__ import annotations

import argparse
import concurrent.futures as cf
import json
import os
import re
import shlex
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CANON = "/tmp/vibeos"  # builds run at the canonical bind-mount path
AREAS = ["ntoskrnl", "hal", "win32ss", "dll", "base", "drivers", "boot", "sdk/lib", "subsystems"]
MINMAX = re.compile(r"\b(min|max|MIN|MAX|min_c|max_c)\s*\(")
MACRO = re.compile(r"expanded from macro '(min|max|MIN|MAX|min_c|max_c)'")
DROP_WITH_VALUE = {"-o", "-MF", "-MT", "-MQ"}
DROP = {"-c", "-MD", "-MMD"}


def _real(p: str) -> str:
    return p.replace(CANON, str(ROOT), 1)


def _live_entries(arch: str) -> tuple[set[str], dict[str, dict]]:
    db_path = ROOT / f"build-{arch}" / "compile_commands.json"
    if not db_path.is_file():
        sys.exit(f"sign_compare_audit: no {db_path}\n  fix: ./vibeos-cli build --arch {arch}")
    live: set[str] = set()
    by_dir: dict[str, dict] = {}
    for e in json.loads(db_path.read_text()):
        f = _real(e["file"])
        live.add(f)
        if f.endswith(".c"):
            by_dir.setdefault(os.path.dirname(f), e)
    return live, by_dir


def _donor(d: str, by_dir: dict[str, dict]) -> dict | None:
    """The nearest live TU: same dir, then an ancestor, then the same area."""
    x = d
    while len(x) > len(str(ROOT)):
        if x in by_dir:
            return by_dir[x]
        x = os.path.dirname(x)
    rel = Path(d).relative_to(ROOT).parts
    for k in (2, 1):
        pre = str(ROOT.joinpath(*rel[:k]))
        for dd in sorted(by_dir):
            if dd == pre or dd.startswith(pre + "/"):
                return by_dir[dd]
    return None


def _candidates(live: set[str], by_dir: dict[str, dict]) -> tuple[list[tuple[str, dict]], list[str]]:
    found: list[tuple[str, dict]] = []
    no_donor: list[str] = []
    for area in AREAS:
        for dp, _dn, fn in os.walk(ROOT / area):
            for n in fn:
                p = os.path.join(dp, n)
                if not n.endswith(".c") or p in live:
                    continue
                if not MINMAX.search(Path(p).read_text(errors="replace")):
                    continue
                donor = _donor(dp, by_dir)
                if donor is None:
                    no_donor.append(p)
                else:
                    found.append((p, donor))
    return found, no_donor


def _command(p: str, donor: dict, arch: str) -> list[str]:
    args = donor.get("arguments") or shlex.split(donor["command"])
    donor_file = _real(donor["file"])
    out: list[str] = []
    skip = False
    for a in (_real(x) for x in args):
        if skip:
            skip = False
            continue
        if a in DROP_WITH_VALUE:
            skip = True
            continue
        if a in DROP or a == donor_file or (a.startswith("-W") and "error" in a):
            continue
        out.append(a)
    inc: list[str] = []
    x = os.path.dirname(p)
    for _ in range(4):
        rel = os.path.relpath(x, ROOT)
        inc += [f"-I{x}", f"-I{x}/include", f"-I{x}/include/internal", f"-I{ROOT / f'build-{arch}' / rel}"]
        x = os.path.dirname(x)
        if x == str(ROOT):
            break
    if os.path.relpath(p, ROOT).startswith("ntoskrnl"):
        inc += ["-D_NTOSKRNL_", "-D__NTOSKRNL__", "-D_NTSYSTEM_", "-include", "ntoskrnl.h"]
    return out + inc + ["-fsyntax-only", "-Wno-everything", "-Wsign-compare", "-ferror-limit=0", p]


def _scan(p: str, donor: dict, arch: str) -> tuple[str, list[str], int]:
    r = subprocess.run(_command(p, donor, arch), cwd=_real(donor["directory"]),
                       capture_output=True, text=True, check=False)
    lines = r.stderr.splitlines()
    src = Path(p).read_text(errors="replace").splitlines()
    rel = os.path.relpath(p, ROOT)
    hits: list[str] = []
    for i, line in enumerate(lines):
        if "-Wsign-compare" not in line or not line.startswith(p + ":"):
            continue
        ln = int(line[len(p) + 1:].split(":", 1)[0])
        text = src[ln - 1].strip() if ln <= len(src) else ""
        notes: list[str] = []  # this diagnostic's own "expanded from" notes, up to the next one
        for nxt in lines[i + 1:]:
            if ": warning: " in nxt or ": error: " in nxt:
                break
            notes.append(nxt)
        if MINMAX.search(text) or MACRO.search("\n".join(notes)):
            why = line.split("warning: ", 1)[-1].replace(" [-Wsign-compare]", "")
            hits.append(f"{rel}:{ln}: {text}  || {why}")
    return rel, hits, sum(1 for line in lines if " error: " in line)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--arch", required=True, choices=["i386", "amd64", "arm64"])
    ap.add_argument("--jobs", type=int, default=8)
    a = ap.parse_args()
    live, by_dir = _live_entries(a.arch)
    found, no_donor = _candidates(live, by_dir)
    n_hits = n_err = 0
    with cf.ThreadPoolExecutor(a.jobs) as ex:
        for rel, hits, errs in ex.map(lambda t: _scan(t[0], t[1], a.arch), found):
            if errs:
                n_err += 1
                print(f"#ERR {errs} {rel}")
            for h in hits:
                n_hits += 1
                print(h)
    for p in no_donor:
        print(f"#NODONOR {os.path.relpath(p, ROOT)}")
    print(f"# {a.arch}: {len(found)} flipped .c scanned ({n_err} partly parsed), "
          f"{len(no_donor)} without a donor TU, {n_hits} mixed-sign min/max sites", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
