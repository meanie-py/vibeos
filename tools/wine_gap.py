#!/usr/bin/env python3
"""What upstream Wine has that this tree has never pulled in.

`port_audit.py` measures the Wine-derived C we carry. This measures the other side: modules in
an upstream Wine checkout with no module of the same name here, sized and sorted by what they
would be to us. Plan and decision: docs/engineering/wine-upstream-intake-plan.md.

    git clone --depth 1 --branch wine-11.19 https://gitlab.winehq.org/wine/wine.git ~/src/wine
    python3 tools/wine_gap.py --wine ~/src/wine           # area table
    python3 tools/wine_gap.py --wine ~/src/wine --list    # every absent module
    python3 tools/wine_gap.py --wine ~/src/wine --json

Matching is by MODULE NAME (a `.spec` basename or a CMake `add_library`/`add_executable`
target here), so a module ReactOS carries under another name reads as absent. "Imports
closed" is DLL-level only: every DLL in its `IMPORTS`/`DELAYIMPORTS` exists here. Whether the
FUNCTIONS it calls are exported is `tools/win10_surface.py gap`'s question, on built PEs.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
from collections import Counter, defaultdict
from dataclasses import asdict, dataclass, field
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent

_SOURCE_EXT = (".c", ".h", ".idl", ".y", ".l", ".cpp")
_SKIP_TOP = ("build-", "modules/rostests", "vendor/")

# First match wins; checked against the lower-cased Wine directory name.
AREAS: list[tuple[str, str, bool]] = [
    # (area, regex, useful on an NT kernel)
    ("16-bit", r"(16|\.exe16|\.dll16|\.drv16|\.mod16|\.vxd)$", False),
    (
        "Unix backend",
        r"^(winex11|winewayland|winemac|wineandroid|winealsa|winepulse|wineoss"
        r"|winecoreaudio|winebus|winegstreamer|winevulkan|wineusb|winedmo|opencl|nsiproxy|wpcap"
        r"|ctapi32|winebth)",
        False,
    ),
    ("WinRT (windows.*)", r"^windows\.", True),
    (
        "Media Foundation / DirectShow / audio",
        r"^(mf|evr|qasf|qdvd|qedit|dmsynth|xaudio|x3daudio"
        r"|xapofx|xactengine|colorcnv|dsdmo|resampledmo|msvproc|mp3dmod|wmphoto|wmvdecod|dsquery"
        r"|msvdsp|wmadmod|msmpeg2|vidreszr|msauddecmft|wmp$)",
        True,
    ),
    ("MSVC runtimes", r"^(msvcp|msvcr|vcomp|vcruntime|concrt|msvcirt|vccorlib)", True),
    ("Direct3D / input / play", r"^(d3d|dxgi|dcomp|d3dim|xinput|gameux|dplay)", True),
    (
        "Networking / web / security",
        r"^(webservices|wsdapi|kerberos|netprofm|ncrypt|cryptxml"
        r"|fwpuclnt|ktmw32|wsnmp|websocket|dhcpcsvc|wlan|httpapi|webauthn)",
        True,
    ),
]


@dataclass
class Module:
    kind: str  # dlls | programs
    name: str
    lines: int
    present: bool
    unix: bool
    area: str
    useful: bool
    imports: list[str] = field(default_factory=list)
    missing_imports: list[str] = field(default_factory=list)


def _norm(name: str) -> str:
    n = name.lower()
    return n.rsplit(".", 1)[0] if n.endswith((".dll", ".exe")) else n


def local_modules(repo: Path = REPO) -> set[str]:
    """Every module name this tree builds or describes: `.spec` basenames + CMake targets."""
    names: set[str] = set()
    for root, dirs, files in os.walk(repo):
        rel = os.path.relpath(root, repo).replace(os.sep, "/")
        if rel != "." and (rel.startswith(_SKIP_TOP) or "/." in "/" + rel):
            dirs[:] = []
            continue
        for f in files:
            if f.endswith(".spec"):
                names.add(f[:-5].lower())
            elif f == "CMakeLists.txt":
                text = Path(root, f).read_text(errors="replace")
                for m in re.finditer(r"add_(?:library|executable)\(\s*([A-Za-z0-9_.\-]+)", text):
                    names.add(m.group(1).lower())
    return names


def _lines(d: Path) -> int:
    n = 0
    for root, dirs, files in os.walk(d):
        dirs[:] = [x for x in dirs if x != "tests"]
        for f in files:
            if f.endswith(_SOURCE_EXT):
                with open(os.path.join(root, f), "rb") as fh:
                    n += sum(1 for _ in fh)
    return n


def makefile_imports(text: str) -> list[str]:
    out: set[str] = set()
    for key in ("IMPORTS", "DELAYIMPORTS"):
        m = re.search(rf"^{key}\s*=\s*((?:[^\n]*\\\n)*[^\n]*)", text, re.M)
        if m:
            for tok in m.group(1).replace("\\\n", " ").split():
                if not tok.startswith("$"):
                    out.add(tok.removeprefix("-l").lower())
    return sorted(out)


def classify(name: str) -> tuple[str, bool]:
    n = name.lower()
    for area, rx, useful in AREAS:
        if re.search(rx, n):
            return area, useful
    return "Other Win32", True


def survey(wine: Path, repo: Path = REPO) -> list[Module]:
    if not (wine / "dlls").is_dir():
        sys.exit(f"wine_gap: {wine} is not a Wine checkout (no dlls/)")
    here = local_modules(repo)
    mods: list[Module] = []
    for kind in ("dlls", "programs"):
        for d in sorted((wine / kind).iterdir()):
            if not d.is_dir():
                continue
            mk_path = d / "Makefile.in"
            mk = mk_path.read_text(errors="replace") if mk_path.exists() else ""
            name = _norm(d.name)
            present = name in here or d.name.lower() in here or name.split(".")[0] in here
            area, useful = classify(d.name)
            unix = bool(re.search(r"^UNIXLIB\s*=", mk, re.M))
            mods.append(
                Module(
                    kind,
                    d.name,
                    _lines(d),
                    present,
                    unix,
                    area,
                    useful and not unix,
                    makefile_imports(mk),
                )
            )
    have = here | {_norm(m.name) for m in mods if m.present}
    for m in mods:
        m.missing_imports = [i for i in m.imports if i not in have and not i.startswith("api-ms")]
    return mods


def report(mods: list[Module], list_all: bool) -> str:
    out = []
    absent = [m for m in mods if not m.present]
    for kind in ("dlls", "programs"):
        k = [m for m in mods if m.kind == kind]
        a = [m for m in k if not m.present]
        out.append(
            f"{kind}: {len(k)} upstream, {len(k) - len(a)} here by name, {len(a)} absent "
            f"({sum(m.lines for m in a):,} lines)"
        )
    out.append("")
    out.append("| Area (absent modules) | Modules | Lines | Useful on NT |")
    out.append("|---|---:|---:|---|")
    by = defaultdict(list)
    for m in absent:
        by[m.area].append(m)
    for area, rows in sorted(by.items(), key=lambda x: -sum(m.lines for m in x[1])):
        useful = "no" if not any(m.useful for m in rows) else "yes"
        out.append(f"| {area} | {len(rows)} | {sum(m.lines for m in rows):,} | {useful} |")
    pe = [m for m in absent if m.kind == "dlls" and m.useful]
    closed = [m for m in pe if not m.missing_imports]
    out.append("")
    out.append(
        f"Useful absent DLLs whose imports all exist here (DLL-level): {len(closed)} modules, "
        f"{sum(m.lines for m in closed):,} lines"
    )
    wanted = Counter(i for m in pe for i in m.missing_imports)
    out.append(
        "Most-wanted missing imports: " + ", ".join(f"{i} ({n})" for i, n in wanted.most_common(10))
    )
    if list_all:
        out.append("")
        for m in sorted(absent, key=lambda m: (m.area, -m.lines)):
            miss = f"  needs {','.join(m.missing_imports)}" if m.missing_imports else ""
            out.append(
                f"  {m.area:40s} {m.name:40s} {m.lines:>7,}{' unix' if m.unix else ''}{miss}"
            )
    return "\n".join(out)


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--wine", type=Path, default=Path.home() / "src" / "wine")
    ap.add_argument("--list", action="store_true", help="list every absent module")
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args(argv)
    mods = survey(args.wine.expanduser())
    if args.json:
        json.dump([asdict(m) for m in mods], sys.stdout, indent=1)
    else:
        print(report(mods, args.list))
    return 0


if __name__ == "__main__":
    sys.exit(main())
