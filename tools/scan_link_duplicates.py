#!/usr/bin/env python3
"""Every duplicate symbol lld ACTUALLY resolved in a cargo-linked PE — C, asm and Rust.

WHY THIS EXISTS
---------------
Every cargo-linked PE passes `-Wl,--allow-multiple-definition` (reactos-link), so a
duplicate resolves first-match and nothing fails. `scan_duplicate_symbols.py` models
the RUST side from source; it cannot see C shims or `.s` objects in the same link,
nor which archive members lld really extracted. The linker can: COFF lld still
WARNS `duplicate symbol` under that flag, naming both definers (with file:line when
there is debug info), and only for members it pulled in. rustc hides linker stderr
behind the allow-by-default `linker_messages` lint, so every `*-cargo/Cargo.toml`
sets it to warn, and cargo caches the diagnostic in
`.fingerprint/<pkg>-<hash>/output-bin-<name>` — readable after ANY build, warm or
cold, without relinking.

Not findings: the panic/eh runtime (STRUCTURAL), one crate reached through two
staticlibs (both definers at the same source line), and compiler_builtins' mem*
against the CRT's.

Out of scope: the two CMake-linked PEs that pass the flag themselves (explorer,
umpnpmgr); every other CMake link fails on a duplicate already.

FIXING A FINDING
----------------
Decide which body is the real one and delete the other. When both are deliberate,
mark the LOSING definition `nolint: duplicate-symbol (<why>)` within the 6 lines
above it, exactly as for scan_duplicate_symbols.py.

USAGE
-----
  scan_link_duplicates.py build-amd64            # report
  scan_link_duplicates.py --check build-amd64    # exit 1 on any finding
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
NOLINT = "nolint: duplicate-symbol"

# The panic / eh runtime every independent Rust staticlib graph carries
# (same set as scan_duplicate_symbols.STRUCTURAL).
STRUCTURAL = re.compile(
    r"^(rust_eh_personality|rust_begin_unwind|rust_oom|__rust_[a-z_]+|_fltused)$"
)

_DUP = re.compile(r"duplicate symbol: (.+?)\s*$")
_AT = re.compile(r"^>>> defined at (.+?)\s*$")
_CONT = re.compile(r"^>>>\s+(\S.*?)\s*$")
_SRCLINE = re.compile(r"^(.+):(\d+)$")


@dataclass(frozen=True)
class Definer:
    where: str  # "path:line" when lld had debug info, else the object
    obj: str

    def source(self) -> tuple[Path, int] | None:
        m = _SRCLINE.match(self.where)
        if not m:
            return None
        raw = m.group(1)
        for prefix in ("/tmp/vibeos/", "/vibeos/"):
            if raw.startswith(prefix):
                raw = raw[len(prefix) :]
                break
        for base in (REPO, REPO / "sdk/lib/rust"):
            p = base / raw
            if p.is_file():
                return p, int(m.group(2))
        return None


@dataclass(frozen=True)
class Finding:
    pe: str
    symbol: str
    a: Definer
    b: Definer


def structural(sym: str) -> bool:
    """Runtime symbol, judged undecorated (i386 adds a leading `_`/`@` and `@N`)."""
    s = re.sub(r"@\d+$", "", sym.removeprefix("__rustc::"))
    return bool(STRUCTURAL.match(s) or (s[:1] in "_@" and STRUCTURAL.match(s[1:])))


def parse(stderr: str) -> list[tuple[str, list[Definer]]]:
    out: list[tuple[str, list[Definer]]] = []
    lines = [ln.strip() for ln in stderr.splitlines()]
    i = 0
    while i < len(lines):
        m = _DUP.search(lines[i])
        if not m:
            i += 1
            continue
        sym, defs = m.group(1), []
        i += 1
        while i < len(lines) and lines[i].startswith(">>>"):
            at = _AT.match(lines[i])
            if at:
                where = at.group(1)
                obj = where
                if i + 1 < len(lines) and not _AT.match(lines[i + 1]):
                    cont = _CONT.match(lines[i + 1])
                    if cont:
                        obj = cont.group(1)
                        i += 1
                defs.append(Definer(where, obj))
            i += 1
        out.append((sym, defs))
    return out


def _current_outputs(release: Path) -> dict[str, Path | None]:
    """bin name -> its cached diagnostics (None = the last link said nothing)."""
    newest: dict[str, tuple[float, Path]] = {}
    for fp in (release / ".fingerprint").glob("*/bin-*"):
        if fp.suffix == ".json":
            continue
        name = fp.name.removeprefix("bin-")
        t = fp.stat().st_mtime
        if name not in newest or t > newest[name][0]:
            newest[name] = (t, fp.parent)
    return {
        name: (d / f"output-bin-{name}") if (d / f"output-bin-{name}").is_file() else None
        for name, (_, d) in newest.items()
    }


_ARCHIVE = re.compile(r"^lib([A-Za-z0-9_]+)\.a\(")


def _marked(path: Path, line: int) -> bool:
    lines = path.read_text(errors="replace").splitlines()
    return any(NOLINT in ln for ln in lines[max(0, line - 6) : line])


def _nolint(d: Definer, symbol: str) -> bool:
    src = d.source()
    if src:
        return _marked(*src)
    # No line info (arm64 objects built without it): find the definition in the
    # archive's crate. `libcrt_dllmain_rust.a(...)` is crate `crt-dllmain-rust`.
    m = _ARCHIVE.match(d.obj)
    if not m:
        return False
    crate = REPO / "sdk/lib/rust" / m.group(1).replace("_", "-") / "src"
    name = re.sub(r"@\d+$", "", symbol).lstrip("_@") or symbol
    defn = re.compile(rf"\bfn\s+_?{re.escape(name)}\s*[(<]|\b{re.escape(name)}:")
    for path in crate.rglob("*.rs") if crate.is_dir() else ():
        for i, ln in enumerate(path.read_text(errors="replace").splitlines(), 1):
            if defn.search(ln) and _marked(path, i):
                return True
    return False


def findings(arch: str, root: Path = REPO) -> tuple[list[Finding], int]:
    """(findings, number of PE links read)."""
    out: set[Finding] = set()
    links = 0
    for tdir in sorted((root / "sdk/lib/rust/target").glob(f"*-{arch}-cargo-link")):
        for release in tdir.glob("*/release"):
            for name, cached in _current_outputs(release).items():
                links += 1
                if cached is None:
                    continue
                pe = f"{tdir.name.removesuffix(f'-{arch}-cargo-link')}:{name}"
                for raw in cached.read_text(errors="replace").splitlines():
                    try:
                        msg = json.loads(raw).get("message", "")
                    except json.JSONDecodeError:
                        continue
                    if "linker stderr" not in msg:
                        continue
                    for sym, defs in parse(msg):
                        if structural(sym) or len(defs) < 2:
                            continue
                        a, b = defs[0], defs[1]
                        # One crate reached through two staticlibs: the same body twice.
                        if a.where == b.where and _SRCLINE.match(a.where):
                            continue
                        # compiler_builtins (build-std `mem`) vs the CRT's mem*: both are
                        # the toolchain/CRT primitive, and AGENTS.md forbids a Rust one.
                        if any("compiler_builtins" in d.obj for d in (a, b)):
                            continue
                        if _nolint(a, sym) or _nolint(b, sym):
                            continue
                        out.add(Finding(pe, sym, a, b))
    return sorted(out, key=lambda f: (f.pe, f.symbol)), links


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=(__doc__ or "").splitlines()[1])
    ap.add_argument("build_dir", type=Path, help="build-<arch>")
    ap.add_argument("--check", action="store_true", help="exit 1 on any finding")
    args = ap.parse_args(argv)
    m = re.search(r"build-(i386|amd64|arm64)$", str(args.build_dir.resolve()))
    if not m:
        print(f"scan_link_duplicates: cannot tell the arch of {args.build_dir}", file=sys.stderr)
        return 2
    arch = m.group(1)
    found, links = findings(arch)
    if links == 0:
        print(
            f"scan_link_duplicates: no cargo link diagnostics for {arch}; build first",
            file=sys.stderr,
        )
        return 2
    for f in found:
        print(f"{arch} {f.pe}: duplicate {f.symbol}\n    {f.a.where}\n    {f.b.where}")
    print(f"{len(found)} duplicate(s) across {links} {arch} PE link(s)")
    return 1 if (args.check and found) else 0


if __name__ == "__main__":
    sys.exit(main())
