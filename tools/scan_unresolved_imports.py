#!/usr/bin/env python3
"""scan_unresolved_imports.py — a shipped PE that imports a name its exporter does not export.

THE INCIDENT (2026-10-10)
    Binding pcix.sys as the PCI bus driver booted i386 green with NO PCI driver at all.
    FreeLDR refused the image:

        err: Did not find export 'RtlAssert@16'!
        err: Cannot load boot driver 'system32\\drivers\\pcix.sys'!
        [FLDR] boot driver not loaded, dropped from the boot driver list

    pcix-rust declared the kernel FUNCTION RtlAssert inside a
    `#[link(kind = "raw-dylib")] extern "system"` block. raw-dylib is for DATA exports;
    for a stdcall function on i386 it mints an import of the DECORATED name `RtlAssert@16`
    (the kernel exports `RtlAssert`), and because raw-dylib invents its own import
    descriptor the link has nothing unresolved to fail on. Every gate was green: build,
    check, link, and a smoke that never needed the driver. Only the loader looks, and the
    loader drops a boot driver and keeps going.

WHAT IT CHECKS
    Every PE on the livecd (the `livecd.Debug.lst` corpus: what actually ships) is read
    for its import table, and every by-name import is looked up in the export table of the
    module it names, resolved by filename on the same medium. A name the exporter does not
    have is a load failure waiting for the first boot that needs the module. Delay imports
    are not judged: they fail at call time, not load time, and are often deliberate.

    Kernel images are substituted at load time — the loader binds `ntoskrnl.exe` and
    `hal.dll` imports to whichever kernel/HAL it chose — so those names are checked
    against EVERY variant on the medium. A name that only `ntoskrnl.exe` exports works
    until the MP kernel is chosen.

    An import naming a module that is not on the medium at all is reported as MISSING_DLL.

`--check` fails on ANY finding: there is no baseline, because the tree had none when this
landed and an unresolved import has no legitimate reason to exist.

Usage:
    python3 tools/scan_unresolved_imports.py build-i386
    python3 tools/scan_unresolved_imports.py --check build-i386
"""

from __future__ import annotations

import argparse
import struct
import sys
from pathlib import Path

if __package__:
    from .binary_corpus import CONTAINERS
else:
    from binary_corpus import CONTAINERS

# The BIOS freeldr.sys is a real-mode stub with the PE body appended: not a PE image.
CONTAINER_NAMES = frozenset(Path(c).name.lower() for c in CONTAINERS)

# Modules whose imports the RUNNING loader satisfies: rosload.exe is loaded by freeldr
# and binds to the exports of the freeldr already in memory, not to a file on the medium.
LOADER_PROVIDED = frozenset({"freeldr.sys"})

# Import names the loader binds to whichever image it selected for that role.
KERNEL_ALIASES = {
    "ntoskrnl.exe": ("ntoskrnl.exe", "ntkrnlmp.exe"),
    "hal.dll": ("hal.dll", "halmp.dll"),
}


class PeError(ValueError):
    pass


class Pe:
    """Just enough of a PE reader for import and export name tables."""

    def __init__(self, data: bytes) -> None:
        self.data = data
        if data[:2] != b"MZ":
            raise PeError("no MZ header")
        pe = struct.unpack_from("<I", data, 0x3C)[0]
        if data[pe : pe + 4] != b"PE\0\0":
            raise PeError("no PE signature")
        (nsect,) = struct.unpack_from("<H", data, pe + 6)
        (optsize,) = struct.unpack_from("<H", data, pe + 20)
        opt = pe + 24
        (magic,) = struct.unpack_from("<H", data, opt)
        if magic == 0x10B:
            self.pe64 = False
            dd = opt + 96
        elif magic == 0x20B:
            self.pe64 = True
            dd = opt + 112
        else:
            raise PeError(f"bad optional header magic {magic:#x}")
        (ndd,) = struct.unpack_from("<I", data, dd - 4)
        self.dirs = [struct.unpack_from("<II", data, dd + 8 * i) for i in range(min(ndd, 16))]
        self.sections = []
        sect = opt + optsize
        for i in range(nsect):
            vsize, va, rsize, rptr = struct.unpack_from("<IIII", data, sect + 40 * i + 8)
            self.sections.append((va, max(vsize, rsize), rptr, rsize))

    def off(self, rva: int) -> int:
        for va, size, rptr, rsize in self.sections:
            if va <= rva < va + size and rva - va < rsize:
                return rptr + (rva - va)
        raise PeError(f"rva {rva:#x} not in any section")

    def cstr(self, rva: int) -> str:
        o = self.off(rva)
        end = self.data.index(b"\0", o)
        return self.data[o:end].decode("latin-1")

    def exports(self) -> set[str]:
        if len(self.dirs) < 1 or not self.dirs[0][0]:
            return set()
        o = self.off(self.dirs[0][0])
        (nnames,) = struct.unpack_from("<I", self.data, o + 24)
        (names_rva,) = struct.unpack_from("<I", self.data, o + 32)
        names_off = self.off(names_rva) if nnames else 0
        return {
            self.cstr(struct.unpack_from("<I", self.data, names_off + 4 * i)[0])
            for i in range(nnames)
        }

    def imports(self) -> dict[str, list[str]]:
        """{dll: [by-name imports]} from the regular (not delay) import directory."""
        out: dict[str, list[str]] = {}
        if len(self.dirs) < 2 or not self.dirs[1][0]:
            return out
        o = self.off(self.dirs[1][0])
        width = 8 if self.pe64 else 4
        ordflag = 1 << (63 if self.pe64 else 31)
        while True:
            ilt, _, _, name_rva, iat = struct.unpack_from("<IIIII", self.data, o)
            if not name_rva:
                break
            dll = self.cstr(name_rva)
            names = out.setdefault(dll, [])
            t = self.off(ilt or iat)
            while True:
                (thunk,) = struct.unpack_from("<Q" if self.pe64 else "<I", self.data, t)
                if not thunk:
                    break
                if not thunk & ordflag:
                    names.append(self.cstr((thunk & 0x7FFFFFFF) + 2))
                t += width
            o += 20
        return out


def corpus(build_dir: Path) -> dict[str, Path]:
    """Lower-cased medium filename -> built file, from the livecd list."""
    lst = build_dir / "boot" / "livecd.Debug.lst"
    if not lst.is_file():
        raise SystemExit(f"scan_unresolved_imports: {lst} missing (build livecd first)")
    canon = f"/tmp/vibeos/{build_dir.name}/"
    files: dict[str, Path] = {}
    for line in lst.read_text().splitlines():
        if "=" not in line:
            continue
        dest, src = line.split("=", 1)
        if not dest.lower().endswith((".sys", ".dll", ".exe", ".drv", ".cpl", ".ocx", ".acm")):
            continue
        if src.startswith(canon):
            src = str(build_dir / src[len(canon) :])
        p = Path(src)
        if p.is_file():
            files.setdefault(Path(dest).name.lower(), p)
    return files


def resolve(
    imports: dict[str, dict[str, list[str]] | None],
    exports: dict[str, set[str] | None],
) -> list[str]:
    """Judge every module's imports against the medium's export tables.

    `imports[name]` / `exports[name]` are None for an image the reader could not parse.
    """
    findings: list[str] = []
    for name in sorted(imports):
        table = imports[name]
        if table is None:
            # A reader that cannot see is never silently a pass.
            if name not in CONTAINER_NAMES:
                findings.append(f"UNREADABLE {name}")
            continue
        for dll, names in sorted(table.items()):
            key = dll.lower()
            if key in LOADER_PROVIDED:
                continue
            present = [v for v in KERNEL_ALIASES.get(key, (key,)) if v in exports]
            if not present:
                findings.append(f"MISSING_DLL {name} -> {dll}")
                continue
            for variant in present:
                exp = exports[variant]
                if exp is None:
                    findings.append(f"UNREADABLE {variant} (imported by {name})")
                    continue
                for sym in sorted(set(names) - exp):
                    findings.append(f"UNRESOLVED {name} -> {variant}!{sym}")
    return findings


def scan(build_dir: Path) -> list[str]:
    imports: dict[str, dict[str, list[str]] | None] = {}
    exports: dict[str, set[str] | None] = {}
    for name, path in corpus(build_dir).items():
        try:
            image = Pe(path.read_bytes())
        except (PeError, struct.error, ValueError):
            imports[name] = exports[name] = None
            continue
        try:
            imports[name] = image.imports()
        except (PeError, struct.error, ValueError):
            imports[name] = None
        try:
            exports[name] = image.exports()
        except (PeError, struct.error, ValueError):
            exports[name] = None
    return resolve(imports, exports)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("build_dir", type=Path)
    ap.add_argument("--check", action="store_true", help="exit 1 on any finding")
    args = ap.parse_args()
    build_dir = args.build_dir.resolve()
    findings = scan(build_dir)
    for f in findings:
        print(f)
    print(f"scan_unresolved_imports[{build_dir.name}]: {len(findings)} findings")
    return 1 if args.check and findings else 0


if __name__ == "__main__":
    sys.exit(main())
