#!/usr/bin/env python3
"""A kernel-mode driver that DEFINES a function ntoskrnl or the HAL exports.

WHY THIS EXISTS (2026-10-07)
----------------------------
classpnp's Rust carried `#[no_mangle] RtlInterlockedPushEntrySList` /
`RtlInterlockedPopEntrySList` bodies that returned NULL. They were written to make
the link succeed: the bindings name the kernel-INTERNAL symbol, which no driver can
import (the export is `ExpInterlocked*EntrySList` on 64-bit, fastcall
`InterlockedPush/PopEntrySList` on i386). A local definition beats the import
library, so every transfer-packet free list dropped its entries for four months and
amd64-bios failed one gate boot in five. Nothing complained: the Rust compiled, the
link was clean, and the driver ran — just not the kernel's code.

The shape is general. A driver that defines `X` where the kernel exports `X` (or
where `X` is the internal name a kernel export points at) runs its own `X` instead
of the kernel's, and the kernel's semantics are only as good as whoever wrote the
local copy.

WHAT IT MEASURES
----------------
Every link unit from `scan_duplicate_symbols.link_units` that pulls in `ntkrnl-sys`
(i.e. kernel-mode code) other than the kernel and HAL images themselves and the boot
loaders, on each arch — skipping crates those images link too, since a copy of the
exporter's own code is the real implementation — against the export names in
ntoskrnl.spec and hal/hal.spec for that arch PLUS the internal names those exports
forward to (the trailing column:
`ExpInterlockedPopEntrySList(ptr ptr) RtlInterlockedPopEntrySList`). That second set
is what catches classpnp: its stubs were named after the forward target, not the
export.

Many hits are deliberate and correct: header FORCEINLINEs the DDK expects every driver
to carry (`KeInitializeSpinLock`, `Interlocked*` on i386, `READ_PORT_*` on arm64) and
real algorithms (`_stricmp`). An EMPTY body never is: four drivers' `RtlAssert` no-ops
hid every failed ASSERT until this scanner found them. Mark a correct one with
`// nolint: export-shadow (<why the local copy is right>)` within 5 lines above the
definition. tools/lint-baseline/scan_export_shadows.txt holds the set reviewed on
2026-10-08 (every entry read: header inlines, real algorithms, the classpnp
forwarders); a NEW finding fails `check`.

FIXING A FINDING
----------------
If the kernel export works for this arch, delete the local body and import it (on
i386 mind the decoration: fastcall exports need `extern "fastcall"` + `link_name`).
If it must stay local, make it the real algorithm, never a stub that returns
nothing, and say why with the nolint marker.

USAGE
-----
  scan_export_shadows.py            # human-readable report
  scan_export_shadows.py --lint     # one finding per line (vibeos-cli check)
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from rust_link_model import ARCHES, REPO, Def, crate_defs
from scan_duplicate_symbols import STRUCTURAL, link_units

NOLINT = "nolint: export-shadow"

SPECS = ("ntoskrnl/ntoskrnl.spec", "hal/hal.spec")

# Units that ARE the exporter (or are not loaded by it at all).
EXPORTERS = {
    "ntoskrnl-cargo",
    "ntkrnlmp-cargo",
    "kernel-rust",
    "hal-cargo",
    "halmp-cargo",
    "hal-rust",
}
# Boot loaders run before the kernel exists; ntdll is user mode (its crate links
# ntkrnl-sys for shared types, not to import from the kernel).
NOT_DRIVERS = {"rosload-cargo", "uefildr-cargo", "ntdll-cargo", "ntdll-rust", "ntdll_vista-cargo"}

_SPEC_ARCH = {"i386": "x86", "x86_64": "x86_64", "amd64": "x86_64", "arm64": "aarch64"}
_SPEC_LINE = re.compile(r"^\s*@\s+(\w+)\s+(.*)$")


def _spec_arches(flags: list[str]) -> set[str]:
    for f in flags:
        if f.startswith("-arch="):
            want = f[len("-arch=") :].split(",")
            neg = {_SPEC_ARCH[a[1:]] for a in want if a.startswith("!") and a[1:] in _SPEC_ARCH}
            pos = {_SPEC_ARCH[a] for a in want if a in _SPEC_ARCH}
            if neg:
                return set(ARCHES) - neg
            return pos
    return set(ARCHES)


# spec2def's `--version` for the kernel and HAL: DLL_EXPORT_VERSION, which config.cmake defaults
# to 0x502 and neither spec2def() call overrides.
EXPORT_VERSION = 0x502


def _version_included(flags: list[str], version: int = EXPORT_VERSION) -> bool:
    """spec2def's `-version=` test: comma-separated `a`, `a+` or `a-b` ranges, hex.

    A `-version=0x602+` line is not in a 5.2 kernel's export table, so a driver defining that
    name shadows nothing -- it is the only copy. ntoskrnl_vista exists to be that copy (its
    ten PoFx* functions are 0x602+ in ntoskrnl.spec).
    """
    for f in flags:
        if not f.startswith("-version="):
            continue
        for rng in f[len("-version=") :].split(","):
            m = re.fullmatch(r"(?:0x)?([0-9A-Fa-f]+)(?:(\+)|-(?:0x)?([0-9A-Fa-f]+))?", rng)
            if not m:
                raise SystemExit(f"error: unparsed spec version range {f!r}")
            lo = int(m.group(1), 16)
            hi = 0xFFF if m.group(2) else int(m.group(3) or m.group(1), 16)
            if lo <= version <= hi:
                return True
        return False
    return True


def kernel_names(root: Path = REPO) -> dict[str, dict[str, str]]:
    """arch -> {symbol: "<spec>: <export>"} for every export and forward target."""
    out: dict[str, dict[str, str]] = {a: {} for a in ARCHES}
    for spec in SPECS:
        p = root / spec
        if not p.is_file():
            raise SystemExit(f"error: {spec} missing")
        for raw in p.read_text(errors="replace").splitlines():
            line = raw.split("#", 1)[0]
            m = _SPEC_LINE.match(line)
            if not m:
                continue
            toks = m.group(2).split()
            flags = [t for t in toks if t.startswith("-")]
            rest = [t for t in toks if not t.startswith("-")]
            if not rest:
                continue
            # A stub has no body in the kernel, so a crate defining the name shadows
            # nothing. Both spellings: the `stub` directive (what spec2def generates,
            # and every win10_surface line) and the `-stub` flag on a typed line.
            if "-stub" in flags or m.group(1) == "stub":
                continue
            if not _version_included(flags):
                continue
            export = re.split(r"[(\s]", rest[0], maxsplit=1)[0]
            names = {export}
            # the forward target follows the closing paren of the argument list
            tail = m.group(2).split(")", 1)[1].split() if ")" in m.group(2) else rest[1:]
            for t in tail:
                if not t.startswith("-"):
                    names.add(t.rsplit(".", 1)[-1])
                    break
            for a in _spec_arches(flags):
                for n in names:
                    if re.fullmatch(r"\w+", n):
                        out[a].setdefault(n, f"{Path(spec).name}: {export}")
    return out


def _nolint(d: Def) -> bool:
    try:
        lines = d.path.read_text(errors="replace").splitlines()
    except OSError:
        return False
    return any(NOLINT in ln for ln in lines[max(0, d.line - 6) : d.line])


def findings(root: Path | None = None) -> dict[Def, tuple[set[str], set[str], str]]:
    """def -> (arches, units, which export it shadows)."""
    names = kernel_names(root or REPO)
    units_all = link_units(root)
    # A crate the kernel or HAL image itself links is the exporter's OWN code: a
    # driver that links it carries a copy of the real implementation (the
    # libcntpr/rtl static-lib model), not a stand-in for it.
    exporter_crates = {
        c for n in EXPORTERS | NOT_DRIVERS for u in units_all.get(n, {}).values() for c in u.crates
    }
    out: dict[Def, tuple[set[str], set[str], str]] = {}
    for unit, per_arch in units_all.items():
        if unit in EXPORTERS or unit in NOT_DRIVERS:
            continue
        for arch, u in per_arch.items():
            if "ntkrnl-sys" not in u.crates:
                continue
            for c in u.crates:
                if c in exporter_crates:
                    continue
                for d in crate_defs(c, root):
                    if d.kind != "rust" or STRUCTURAL.match(d.base):
                        continue
                    what = names[arch].get(d.base)
                    if what is None or not d.cond.satisfiable_with(d.cond, arch, u.pinned):
                        continue
                    if _nolint(d):
                        continue
                    arches, units, _ = out.setdefault(d, (set(), set(), what))
                    arches.add(arch)
                    units.add(unit)
    return out


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--lint", action="store_true", help="one finding per line")
    ap.add_argument("root", nargs="?", type=Path, help="fixture tree instead of the repo")
    args = ap.parse_args(argv)
    root = args.root.resolve() if args.root else None

    res = findings(root)
    rows = sorted(res.items(), key=lambda kv: (kv[0].loc(), kv[0].name))
    for d, (arches, units, what) in rows:
        ar = ",".join(a for a in ARCHES if a in arches)
        if args.lint:
            print(f"{d.loc()}: {d.crate} defines {d.name}, a kernel export ({what}) [{ar}]")
        else:
            print(f"{d.name}  [{ar}]  shadows {what}")
            print(f"    {d.loc()} ({d.crate}; linked into {', '.join(sorted(units))})")
    if not args.lint:
        print(f"\n{len(rows)} local definition(s) of kernel exports in drivers")
    return 0


if __name__ == "__main__":
    sys.exit(main())
