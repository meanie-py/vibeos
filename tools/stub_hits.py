#!/usr/bin/env python3
"""Rank the stubs real runs call, and the GetProcAddress misses beside them.

    python3 tools/stub_hits.py [PATH ...] [--top N] [--json]

PATH is a run directory, a tree of them, or a log file; the default is every run under
/tmp/vibeos-cli. Each boot's serial capture is read once (a run's full.log only when it
has none), so a boot counts once however many copies of its output it keeps.

Two lists come out, and together they are the implementation queue for the Windows 10
export surface (docs/engineering/modern-windows-compat-roadmap.md, Rung 0):

  stubs   `WARNING: calling stub <module>!<name>(` -- a stub some run reached. Its KIND
          is read from the module's spec: `raises` (a bare `@ stub`: noncontinuable
          EXCEPTION_WINE_STUB, the caller dies), `returns 0` (`-stub` with arguments) or
          `returns V` (`-stub=V`, logged once per process). A line from before stubs
          named their module is matched to the one spec stubbing that name, or listed
          as ambiguous. DIED counts the runs where a process died of it: kernel32's
          unhandled-exception filter prints `Missing function: <module>!<name>` for an
          EXCEPTION_WINE_STUB nobody caught.
  misses  `LDR: GetProcAddress miss <module>!<name>` -- a function a program LOOKED FOR
          and did not find. A program that probes has a fallback, so a miss is usually
          the right answer and a stub there would break it; a miss that IS a Windows
          export and keeps recurring is a body to write, not a stub to add.

Ranked by the number of boots that hit each one, then by line count: one boot calling a
stub a thousand times is one program, a stub ten boots reach is ten.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from collections import defaultdict
from dataclasses import dataclass, field
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from win10_surface import module_specs, spec_rows

DEFAULT_ROOT = Path("/tmp/vibeos-cli")
STUB = re.compile(r"WARNING: calling stub (?:([\w.$-]+)!)?([^\s(]+)\(")
MISS = re.compile(r"LDR: GetProcAddress miss ([^\s!]+)!(\S+)")
DIED = re.compile(r"Missing function: ([^\s!]+)!(\S+)")


@dataclass
class Hit:
    module: str  # "" when the log line predates module-qualified stub messages
    name: str
    runs: set[str] = field(default_factory=set)
    lines: int = 0
    died: set[str] = field(default_factory=set)


def run_logs(paths: list[Path]) -> list[tuple[str, Path]]:
    """[(boot id, the one log to read for it)].

    Every serial capture is one boot: `serial.log` (smoke keeps one per boot, under
    `boot0/`, `boot1/`, ..., and its net and payload boots as `<arch>_serial.log`) or
    `<arch>_serial.log` (conformance). A run's `full.log` repeats its serial, so it is
    read only for a run directory with no capture of its own anywhere beneath it; a
    capture copied elsewhere in the run is dropped by `scan` as identical bytes.
    """
    out: list[tuple[str, Path]] = []
    for p in paths:
        if p.is_file():
            out.append((str(p), p))
            continue
        serials = sorted({*p.rglob("serial.log"), *p.rglob("*_serial.log")})
        out += [(str(f), f) for f in serials]
        for full in sorted(p.rglob("full.log")):
            if not any(f.is_relative_to(full.parent) for f in serials):
                out.append((str(full.parent), full))
    return out


def scan(
    logs: list[tuple[str, Path]],
) -> tuple[dict[tuple[str, str], Hit], dict[tuple[str, str], Hit], int]:
    """(stubs, misses, boots read)."""
    stubs: dict[tuple[str, str], Hit] = {}
    misses: dict[tuple[str, str], Hit] = {}
    seen: set[str] = set()
    for run, log in logs:
        try:
            data = log.read_bytes()
        except OSError:
            continue
        # smoke copies boot0/serial.log up to the run directory: same boot, same bytes.
        digest = hashlib.sha1(data).hexdigest()
        if digest in seen:
            continue
        seen.add(digest)
        text = data.decode(errors="replace")
        for table, rx in ((stubs, STUB), (misses, MISS)):
            for m in rx.finditer(text):
                key = ((m.group(1) or "").lower(), m.group(2))
                hit = table.setdefault(key, Hit(*key))
                hit.runs.add(run)
                hit.lines += 1
        for m in DIED.finditer(text):
            key = (m.group(1).lower(), m.group(2))
            hit = stubs.get(key) or stubs.setdefault(key, Hit(*key))
            hit.runs.add(run)
            hit.died.add(run)
    return stubs, misses, len(seen)


class SpecIndex:
    """What each module's spec says a stubbed name does when called."""

    def __init__(self, specs: dict[str, list[Path]]):
        self.kinds: dict[str, dict[str, str]] = {}
        self.stubbed_by: dict[str, set[str]] = defaultdict(set)
        for module, paths in specs.items():
            kinds: dict[str, str] = {}
            for spec in paths:
                for name, rows in spec_rows(spec.read_text(errors="replace")).items():
                    for row in rows:
                        kind = self.row_kind(row.kind, row.opts, row.args)
                        if kind:
                            kinds.setdefault(name, kind)
            self.kinds[module] = kinds
            for name in kinds:
                self.stubbed_by[name].add(module)

    @staticmethod
    def row_kind(conv: str, opts: list[str], args: str) -> str | None:
        for o in opts:
            if o.startswith("-stub="):
                return "returns " + o[len("-stub=") :]
        if "-stub" in opts or (conv == "stub" and args.strip("() ")):
            return "returns 0"
        if conv == "stub":
            return "raises"
        return None

    def resolve(self, module: str, name: str) -> tuple[str, str]:
        """(module, kind); module comes back "a|b" when an old line cannot say which."""
        if module:
            return module, self.kinds.get(module, {}).get(name, "?")
        owners = sorted(self.stubbed_by.get(name, ()))
        if len(owners) == 1:
            return owners[0], self.kinds[owners[0]][name]
        return "|".join(owners) or "?", "ambiguous" if owners else "?"


def resolved(
    stubs: dict[tuple[str, str], Hit], index: SpecIndex, kinds: dict[tuple[str, str], str]
) -> list[Hit]:
    """`stubs` re-keyed by resolved module, ranked: an unqualified line and a qualified
    one for the same stub are one stub. `kinds` is filled with each one's kind."""
    merged: dict[tuple[str, str], Hit] = {}
    for h in stubs.values():
        module, kind = index.resolve(h.module, h.name)
        key = (module, h.name)
        kinds.setdefault(key, kind)
        m = merged.setdefault(key, Hit(module, h.name))
        m.runs |= h.runs
        m.died |= h.died
        m.lines += h.lines
    return ranked(merged)


def ranked(table: dict[tuple[str, str], Hit]) -> list[Hit]:
    return sorted(table.values(), key=lambda h: (-len(h.runs), -h.lines, h.module, h.name))


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("paths", nargs="*", type=Path, default=[DEFAULT_ROOT])
    ap.add_argument("--top", type=int, default=40)
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args(argv)

    logs = run_logs(args.paths)
    stubs, misses, boots = scan(logs)
    index = SpecIndex(module_specs())
    rows = []
    kinds: dict[tuple[str, str], str] = {}
    for h in resolved(stubs, index, kinds):
        rows.append(
            {
                "module": h.module,
                "name": h.name,
                "kind": kinds[(h.module, h.name)],
                "runs": len(h.runs),
                "lines": h.lines,
                "died": len(h.died),
                "example": min(h.runs),
            }
        )
    miss_rows = [
        {
            "module": h.module,
            "name": h.name,
            "runs": len(h.runs),
            "lines": h.lines,
            "example": min(h.runs),
        }
        for h in ranked(misses)
    ]

    if args.json:
        json.dump({"boots": boots, "stubs": rows, "misses": miss_rows}, sys.stdout, indent=1)
        print()
        return 0

    print(f"{boots} boots; {len(rows)} stubs called, {len(miss_rows)} GetProcAddress misses")
    print(f"\n{'boots':>5} {'lines':>7} {'died':>5}  {'kind':<14} stub")
    for r in rows[: args.top]:
        where = f"{r['module']}!{r['name']}"
        print(f"{r['runs']:>5} {r['lines']:>7} {r['died']:>5}  {r['kind']:<14} {where}")
    print(f"\n{'boots':>5} {'lines':>7}  GetProcAddress miss")
    for r in miss_rows[: args.top]:
        print(f"{r['runs']:>5} {r['lines']:>7}  {r['module']}!{r['name']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
