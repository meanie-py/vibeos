#!/usr/bin/env python3
"""Two definitions of one exported symbol, live on the SAME arch, in the SAME link.

WHY THIS EXISTS
---------------
Every PE is linked with `-Wl,--allow-multiple-definition` (reactos-link, "common to
every module kind"), which is load-bearing: each independent Rust staticlib graph
emits its own panic / compiler-builtins runtime. The price is that ANY duplicate
resolves first-match, silently. Two incidents, both invisible to the old
crate-name census this replaces:

  * i386 `RtlPrefetchMemoryNonTemporal` (88b04bf7f86): reviving an orphan-cfg'd
    empty Rust stub put it in the same crate as `prefetch_asm.rs`'s `global_asm!`
    label. rustc cannot see into the assembler, the stub won the link, and the
    kiinit NOP patch fell through it — BUGCHECK 0x1E in kmtest RtlMemoryKM.
  * arm64 `RtlUshortByteSwap` in `arm64/stubs.rs` duplicated the byteswap crate,
    which the old hard-coded crate list did not include.

WHAT IT MEASURES
----------------
`tools/rust_link_model.py` builds a source model of every crate: only files
reachable from the crate root through `mod`/`#[path]`/`include!`, every
definition's full cfg conjunction (item + inline modules + mod declarations +
`#![cfg]`), `#[no_mangle]`/`#[export_name]` items and `global_asm!` `.globl` labels.

A LINK UNIT is the path-dependency closure of the Rust libraries one binary links:
every `rust_staticlib("X")` / `rust_lib("X", …)` in a `*-cargo/build.rs`, plus each
`crate-type = staticlib|cdylib` crate on its own. A finding is one undecorated
name (i386 `_X`, `_X@N`, `@X@N` → `X`) defined at two places in one unit whose
cfgs are jointly satisfiable on one arch — arch is fixed per build, every other
atom (`feature = "smp"`, `debug_assertions`) is free, so `feature="smp"` /
`not(feature="smp")` twins are correctly apart.

Out of scope: C objects in the link, and symbols a `macro_rules!` expands into.

FIXING A FINDING
----------------
Decide which definition is the real one and DELETE the other. Never "fix" it by
narrowing a cfg until the scanner is quiet without knowing which body the link
actually took — that is how the prefetch stub came back. When both are
deliberate (a cohab override), say so on the losing definition with
`// nolint: duplicate-symbol (<which one wins and why>)`.

USAGE
-----
  scan_duplicate_symbols.py            # human-readable report
  scan_duplicate_symbols.py --lint     # one finding per line (vibeos-cli check)
  scan_duplicate_symbols.py ROOT       # a fixture tree (crates found by Cargo.toml)
"""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from itertools import combinations
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from rust_link_model import (
    ARCHES,
    REPO,
    Def,
    crate_defs,
    crates,
    feature_atom,
    free_atoms,
    resolve_features,
    unknown_build_cfgs,
)

NOLINT = "nolint: duplicate-symbol"

# Symbols each independent Rust staticlib graph legitimately defines its own copy
# of (the panic/eh runtime). Irreducible while N>1 staticlibs feed one link.
STRUCTURAL = re.compile(
    r"^(rust_eh_personality|rust_begin_unwind|rust_oom|__rust_[a-z_]+|_fltused)$"
)

_LINK_CALL = re.compile(r'\.rust_(?:staticlib|lib)\s*\(\s*"([^"]+)"')
_LIB = re.compile(r'\.rust_lib\s*\(\s*"([^"]+)"\s*,\s*(&\[[^\]]*\]|\w+)\s*,\s*(true|false)')
_STATICLIB = re.compile(r'\.rust_staticlib\s*\(\s*"([^"]+)"')
_FEATS = re.compile(r"\.rust_features\s*\(\s*&\[([^\]]*)\]")
_HELPER = re.compile(r"\b(\w+)_link::")


@dataclass
class Unit:
    crates: list[str]
    pinned: frozenset[tuple[str, bool]]  # feature atoms this binary decides


def _strip_comments(src: str) -> str:
    return re.sub(r"//[^\n]*", "", src)


def _unit(roots: dict[str, tuple[set[str], bool]], maybe: set[str], root: Path | None) -> Unit:
    on, unknown = resolve_features(roots, maybe, root)
    allc = crates(root)
    pinned = []
    for c in on:
        for f in allc[c].features:
            if f != "default" and f not in unknown[c]:
                pinned.append((feature_atom(c, f), f in on[c]))
    return Unit(sorted(on), frozenset(pinned))


def _arch_guard(cond: str) -> set[str] | None:
    """Arches an `if` header over `arch == "x"` / `arch != "x"` (joined by || or &&)
    selects; None when it tests anything else."""
    cond = cond.strip().strip("()")
    for op, join in (("||", set.union), ("&&", set.intersection)):
        if op in cond:
            parts = [_arch_guard(p) for p in cond.split(op)]
            return None if None in parts else join(*parts)  # type: ignore[arg-type]
    m = re.fullmatch(r'arch(?:\.as_str\(\))?\s*(==|!=)\s*"(\w+)"', cond)
    if not m:
        return None
    return {m.group(2)} if m.group(1) == "==" else set(ARCHES) - {m.group(2)}


def _arches_at(src: str) -> list[set[str]]:
    """For every offset, the arches whose build reaches it through `if arch == …`
    blocks (and their `else`). Other conditions don't restrict."""
    out: list[set[str]] = []
    stack: list[set[str]] = [set(ARCHES)]
    last_if: set[str] | None = None
    header_start = 0
    for i, c in enumerate(src):
        out.append(stack[-1])
        if c == "{":
            header = src[header_start:i]
            cur = stack[-1]
            m = re.search(r"(?:^|\s)(else\s+)?if\s+(.*)$", header, re.S)
            if m and not re.search(r"\bmatch\b|=>|\blet\b", m.group(2)):
                g = _arch_guard(m.group(2))
                sel = set(ARCHES) if g is None else g
                if m.group(1) and last_if is not None:
                    sel &= set(ARCHES) - last_if
                last_if = sel if g is not None else None
                stack.append(cur & sel)
            elif re.search(r"(?:^|\s)else\s*$", header) and last_if is not None:
                stack.append(cur - last_if)
                last_if = None
            else:
                stack.append(cur)
            header_start = i + 1
        elif c == "}":
            if len(stack) > 1:
                stack.pop()
            header_start = i + 1
        elif c == ";":
            header_start = i + 1
            last_if = None
    return out


def link_units(root: Path | None) -> dict[str, dict[str, Unit]]:
    """unit -> arch -> Unit. One unit per binary: each *-cargo build.rs (plus the
    `<x>-link` helper crate it delegates to), and each staticlib crate on its own. A
    crate a build.rs links only under `if arch == "x86"` is in the x86 unit only."""
    allc = crates(root)
    base = root or REPO
    units: dict[str, dict[str, Unit]] = {}
    for br in sorted(base.rglob("*-cargo/build.rs")):
        if root is None and "sdk/lib/rust" not in br.as_posix():
            continue
        src = _strip_comments(br.read_text(errors="replace"))
        for helper in set(_HELPER.findall(src)) - {"reactos"}:
            h = allc.get(helper.replace("_", "-") + "-link")
            if h and (h.dir / "src" / "lib.rs").is_file():
                src += "\n" + _strip_comments(
                    (h.dir / "src" / "lib.rs").read_text(errors="replace")
                )
        at = _arches_at(src)
        maybe = set(re.findall(r'"([\w-]+)"', src))  # any name spelled may be env/arch-switched
        per: dict[str, Unit] = {}
        for arch in ARCHES:
            roots: dict[str, tuple[set[str], bool]] = {}
            for m in _LIB.finditer(src):
                if arch in at[m.start()]:
                    crate, feats, dflt = m.groups()
                    lit = set(re.findall(r'"([^"]+)"', feats)) if feats.startswith("&") else set()
                    roots[crate] = (roots.get(crate, (set(), False))[0] | lit, dflt == "true")
            off = 0
            for stmt in src.split(";"):
                if (m := _STATICLIB.search(stmt)) and arch in at[off + m.start()]:
                    lit = {f for x in _FEATS.findall(stmt) for f in re.findall(r'"([^"]+)"', x)}
                    roots[m.group(1)] = (lit, True)
                off += len(stmt) + 1
            for m in _LINK_CALL.finditer(src):
                if arch in at[m.start()]:
                    roots.setdefault(m.group(1), (set(), True))
            roots = {k: v for k, v in roots.items() if k in allc}
            if roots:
                per[arch] = _unit(roots, maybe, root)
        if per:
            units[br.parent.name] = per
    for c in allc.values():
        if c.staticlib and c.name not in units:
            u = _unit({c.name: (set(), True)}, set(), root)
            units[c.name] = dict.fromkeys(ARCHES, u)
    return units


def _nolint(d: Def) -> bool:
    try:
        lines = d.path.read_text(errors="replace").splitlines()
    except OSError:
        return False
    lo = max(0, d.line - 6)
    return any(NOLINT in ln for ln in lines[lo : d.line])


def findings(root: Path | None = None) -> dict[tuple[Def, Def], tuple[set[str], set[str]]]:
    """(def, def) -> (arches, units)."""
    out: dict[tuple[Def, Def], tuple[set[str], set[str]]] = {}
    for unit, per_arch in link_units(root).items():
        for arch, u in per_arch.items():
            fixed = u.pinned
            defs = [d for c in u.crates for d in crate_defs(c, root)]
            by_name: dict[str, list[Def]] = {}
            for d in defs:
                n = d.link_name(arch)
                if STRUCTURAL.match(d.base) or not d.cond.satisfiable_with(d.cond, arch, fixed):
                    continue
                by_name.setdefault(n, []).append(d)
            for group in by_name.values():
                for a, b in combinations(sorted(set(group), key=lambda d: (d.loc(), d.name)), 2):
                    if a.loc() == b.loc() or not a.cond.satisfiable_with(b.cond, arch, fixed):
                        continue
                    if _nolint(a) or _nolint(b):
                        continue
                    arches, units = out.setdefault((a, b), (set(), set()))
                    arches.add(arch)
                    units.add(unit)
    return out


def _fmt(d: Def) -> str:
    return f"{d.loc()} ({d.crate}, {d.kind} {d.name})"


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument(
        "--lint", action="store_true", help="one finding per line, for the baseline diff"
    )
    ap.add_argument("root", nargs="?", type=Path, help="fixture tree instead of the repo")
    args = ap.parse_args(argv)
    root = args.root.resolve() if args.root else None

    res = findings(root)
    # A build-script cfg the model can't evaluate reads as a free boolean, which makes
    # exclusive backends look co-linkable (or hides a real overlap). Refuse to guess.
    unknown = unknown_build_cfgs(root)
    used = {x for (a, b) in res for d in (a, b) for p in d.cond.preds for x in free_atoms(p)}
    linked = {c for per in link_units(root).values() for u in per.values() for c in u.crates}
    used |= {
        x for c in linked for d in crate_defs(c, root) for p in d.cond.preds for x in free_atoms(p)
    }
    if hit := sorted(set(unknown) & used):
        for n in hit:
            print(
                f"error: cfg `{n}` is set by {unknown[n]}; add what it means to "
                "BUILD_CFGS in tools/rust_link_model.py"
            )
        return 2
    rows = sorted(
        (
            (
                a.base if a.kind == "rust" else b.base,
                ",".join(x for x in ARCHES if x in ar),
                a,
                b,
                un,
            )
            for (a, b), (ar, un) in res.items()
        ),
        key=lambda r: (r[0], r[1], r[2].loc(), r[3].loc()),
    )
    for name, arches, a, b, units in rows:
        if args.lint:
            print(f"{_fmt(a)}: {name} [{arches}] also defined at {_fmt(b)}")
        else:
            print(f"{name}  [{arches}]  in {', '.join(sorted(units))}")
            print(f"    {_fmt(a)}")
            print(f"    {_fmt(b)}")
    if not args.lint:
        print(f"\n{len(rows)} duplicate definition pair(s)")
        if rows:
            print(
                "fix: decide which body the binary should run and DELETE the other; "
                "see the docstring."
            )
    return 0


if __name__ == "__main__":
    sys.exit(main())
