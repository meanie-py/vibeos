#!/usr/bin/env python3
"""Are two PE images the same MODULE? The check for moving a module's link cmake -> cargo.

A cargo-linked module cannot be byte-identical to the cmake-linked one it replaces: the
same objects reach lld in a different order, so code and data land at different RVAs. What
must not change is everything the loader and every caller can see:

  header   machine, image base, alignments, OS/subsystem versions, subsystem, file and
           DLL characteristics, stack/heap sizes, which data directories are present
  exports  every ordinal, name and forwarder (never the RVA)
  imports  every DLL and every function or ordinal taken from it, normal and delay-load
  rsrc     every resource type/name/language and the SHA-256 of its bytes
  sections names and characteristics (never sizes or addresses)

    python3 tools/pe_equiv.py old.exe new.exe        # exit 0 equivalent, 1 not, 2 unreadable
    python3 tools/pe_equiv.py --code old.exe new.exe # and the same bytes at the same RVAs

`--code` is for a change that should not move a byte the loader maps, such as a library
now built by a different tool from the same source: every section but DWARF must have
the same address, size and content. Only debug info (object paths, line tables) may
differ (and `.buildid`, a hash over the whole file), and lld is deterministic, so this
proves the link chose the same code and data at the same addresses.
    python3 tools/pe_equiv.py --dump some.dll

Pure Python, no toolchain needed. Plan: docs/engineering/cargo-drive-plan.md.
"""

from __future__ import annotations

import hashlib
import struct
import sys
from pathlib import Path

_DIRS = (
    "export",
    "import",
    "resource",
    "exception",
    "certificate",
    "basereloc",
    "debug",
    "architecture",
    "globalptr",
    "tls",
    "loadconfig",
    "boundimport",
    "iat",
    "delayimport",
    "clr",
    "reserved",
)
# Directories whose PRESENCE is part of the module contract. `debug` and `iat` are link
# products, and `exception`/`basereloc` follow from the code the same objects produce.
_CONTRACT_DIRS = ("export", "import", "resource", "tls", "loadconfig", "delayimport")


class PEError(Exception):
    pass


class PE:
    def __init__(self, data: bytes):
        self.data = data
        if data[:2] != b"MZ":
            raise PEError("no MZ header")
        pe = struct.unpack_from("<I", data, 0x3C)[0]
        if data[pe : pe + 4] != b"PE\0\0":
            raise PEError("no PE signature")
        (self.machine, nsect, _ts, _, _, opt_size, self.characteristics) = struct.unpack_from(
            "<HHIIIHH", data, pe + 4
        )
        opt = pe + 24
        magic = struct.unpack_from("<H", data, opt)[0]
        self.pe64 = magic == 0x20B
        if self.pe64:
            self.image_base = struct.unpack_from("<Q", data, opt + 24)[0]
            o = opt + 32
        else:
            self.image_base = struct.unpack_from("<I", data, opt + 28)[0]
            o = opt + 32
        (
            self.sect_align,
            self.file_align,
            self.os_major,
            self.os_minor,
            _,
            _,
            self.sub_major,
            self.sub_minor,
        ) = struct.unpack_from("<IIHHHHHH", data, o)
        self.subsystem, self.dll_characteristics = struct.unpack_from("<HH", data, o + 36)
        if self.pe64:
            self.stack = struct.unpack_from("<QQQQ", data, o + 40)
            ndir_off = o + 40 + 32 + 4
        else:
            self.stack = struct.unpack_from("<IIII", data, o + 40)
            ndir_off = o + 40 + 16 + 4
        ndirs = struct.unpack_from("<I", data, ndir_off)[0]
        self.dirs = {}
        for i in range(min(ndirs, 16)):
            rva, size = struct.unpack_from("<II", data, ndir_off + 4 + 8 * i)
            self.dirs[_DIRS[i]] = (rva, size)
        sect = opt + opt_size
        self.sections = []
        for i in range(nsect):
            s = sect + 40 * i
            name = data[s : s + 8].rstrip(b"\0").decode("latin-1")
            vsize, va, rsize, rptr = struct.unpack_from("<IIII", data, s + 8)
            chars = struct.unpack_from("<I", data, s + 36)[0]
            self.sections.append((name, va, vsize, rptr, rsize, chars))

    def off(self, rva: int) -> int:
        for _n, va, vsize, rptr, rsize, _c in self.sections:
            if va <= rva < va + max(vsize, rsize):
                return rptr + (rva - va)
        raise PEError(f"RVA {rva:#x} is in no section")

    def cstr(self, rva: int) -> str:
        o = self.off(rva)
        end = self.data.index(b"\0", o)
        return self.data[o:end].decode("latin-1")

    def exports(self) -> list[str]:
        rva, size = self.dirs.get("export", (0, 0))
        if not rva:
            return []
        o = self.off(rva)
        (_, _, _, _, _name, base, nfun, nnames, afun, anames, aords) = struct.unpack_from(
            "<IIHHIIIIIII", self.data, o
        )
        names = {}
        for i in range(nnames):
            nrva = struct.unpack_from("<I", self.data, self.off(anames) + 4 * i)[0]
            idx = struct.unpack_from("<H", self.data, self.off(aords) + 2 * i)[0]
            names[idx] = self.cstr(nrva)
        out = []
        for i in range(nfun):
            frva = struct.unpack_from("<I", self.data, self.off(afun) + 4 * i)[0]
            if not frva:
                continue
            fwd = f" -> {self.cstr(frva)}" if rva <= frva < rva + size else ""
            out.append(f"@{base + i} {names.get(i, '(noname)')}{fwd}")
        return out

    def _thunks(self, rva: int) -> list[str]:
        out, width = [], 8 if self.pe64 else 4
        flag = 1 << (63 if self.pe64 else 31)
        o = self.off(rva)
        while True:
            v = struct.unpack_from("<Q" if self.pe64 else "<I", self.data, o)[0]
            if not v:
                return out
            out.append(f"#{v & 0xFFFF}" if v & flag else self.cstr((v & 0x7FFFFFFF) + 2))
            o += width

    def imports(self) -> list[str]:
        out = []
        rva, _ = self.dirs.get("import", (0, 0))
        if rva:
            o = self.off(rva)
            while True:
                ilt, _, _, name, iat = struct.unpack_from("<IIIII", self.data, o)
                if not name:
                    break
                dll = self.cstr(name).lower()
                out += [f"{dll}!{f}" for f in self._thunks(ilt or iat)]
                o += 20
        rva, _ = self.dirs.get("delayimport", (0, 0))
        if rva:
            o = self.off(rva)
            while True:
                (_attr, name, _h, _iat, int_rva, *_rest) = struct.unpack_from(
                    "<IIIIIIII", self.data, o
                )
                if not name:
                    break
                dll = self.cstr(name).lower()
                out += [f"delay {dll}!{f}" for f in self._thunks(int_rva)]
                o += 32
        return sorted(out)

    def resources(self) -> list[str]:
        rva, _ = self.dirs.get("resource", (0, 0))
        if not rva:
            return []
        root = self.off(rva)
        out: list[str] = []

        def name_of(entry_name: int) -> str:
            if entry_name & 0x80000000:
                o = root + (entry_name & 0x7FFFFFFF)
                n = struct.unpack_from("<H", self.data, o)[0]
                return self.data[o + 2 : o + 2 + 2 * n].decode("utf-16-le")
            return f"#{entry_name}"

        def walk(dir_off: int, path: list[str]) -> None:
            _, _, _, _, nnamed, nid = struct.unpack_from("<IIHHHH", self.data, dir_off)
            for i in range(nnamed + nid):
                ename, target = struct.unpack_from("<II", self.data, dir_off + 16 + 8 * i)
                p = [*path, name_of(ename)]
                if target & 0x80000000:
                    walk(root + (target & 0x7FFFFFFF), p)
                else:
                    drva, dsize = struct.unpack_from("<II", self.data, root + target)
                    blob = self.data[self.off(drva) : self.off(drva) + dsize]
                    out.append(f"{'/'.join(p)} {dsize} {hashlib.sha256(blob).hexdigest()[:16]}")

        walk(root, [])
        return sorted(out)

    def mapped_size(self) -> int:
        """Bytes the loader maps: every section but the discardable (debug) ones."""
        return sum(max(vs, rs) for _n, _va, vs, _rp, rs, c in self.sections if not c & 0x02000000)

    def loaded(self) -> list[str]:
        """Every section but DWARF, as name, address, size and the hash of its bytes.

        `.buildid` is left out too: lld hashes the whole file into it, DWARF included."""
        out = []
        for n, va, vs, rp, rs, c in self.sections:
            dwarf = c & 0x02000000 and (n.startswith("/") or n.startswith(".debug"))
            if dwarf or n == ".buildid":
                continue
            blob = self.data[rp : rp + min(vs, rs)] if rp else b""
            out.append(f"{n} {va:#x}+{vs:#x} {hashlib.sha256(blob).hexdigest()[:16]}")
        return out

    def contract(self) -> dict[str, list[str]]:
        hdr = [
            f"machine {self.machine:#x}",
            f"image_base {self.image_base:#x}",
            f"alignment section={self.sect_align:#x} file={self.file_align:#x}",
            f"os {self.os_major}.{self.os_minor}",
            f"subsystem_version {self.sub_major}.{self.sub_minor}",
            f"subsystem {self.subsystem}",
            f"characteristics {self.characteristics:#x}",
            f"dll_characteristics {self.dll_characteristics:#x}",
            f"stack/heap {' '.join(hex(x) for x in self.stack)}",
            "directories " + " ".join(d for d in _CONTRACT_DIRS if self.dirs.get(d, (0, 0))[0]),
        ]
        # Discardable sections are never mapped: DWARF, named `.debug_*` or `/<n>` (an
        # offset into the COFF string table, so the number moves with every link).
        secs = sorted(
            {
                f"{n} {c:#x}"
                for n, _va, _vs, _rp, _rs, c in self.sections
                if not (c & 0x02000000 and (n.startswith("/") or n.startswith(".debug")))
            }
        )
        return {
            "header": hdr,
            "exports": self.exports(),
            "imports": self.imports(),
            "rsrc": self.resources(),
            "sections": secs,
        }


def load(path: Path) -> PE:
    try:
        return PE(path.read_bytes())
    except (OSError, struct.error, ValueError, PEError) as exc:
        raise PEError(f"{path}: {exc}") from exc


def compare(a: PE, b: PE, code: bool = False) -> list[str]:
    """Human-readable differences between two modules' contracts; empty when equivalent.

    With `code`, also every mapped section's address, size and bytes (see `--code`)."""
    out = []
    ca, cb = a.contract(), b.contract()
    if code:
        ca["loaded"], cb["loaded"] = a.loaded(), b.loaded()
    for key in ca:
        sa, sb = set(ca[key]), set(cb[key])
        for x in sorted(sa - sb):
            out.append(f"{key}: - {x}")
        for x in sorted(sb - sa):
            out.append(f"{key}: + {x}")
    return out


def main(argv: list[str] | None = None) -> int:
    args = sys.argv[1:] if argv is None else argv
    try:
        if len(args) == 2 and args[0] == "--dump":
            for key, lines in load(Path(args[1])).contract().items():
                for line in lines:
                    print(f"{key}: {line}")
            return 0
        code = bool(args) and args[0] == "--code"
        if code:
            args = args[1:]
        if len(args) != 2:
            print(
                __doc__.strip().splitlines()[0],
                "\nusage: pe_equiv.py [--code] OLD NEW | --dump PE",
            )
            return 2
        diffs = compare(load(Path(args[0])), load(Path(args[1])), code)
    except PEError as exc:
        print(f"pe_equiv: {exc}")
        return 2
    for d in diffs:
        print(d)
    print("EQUIVALENT" if not diffs else f"DIFFERENT ({len(diffs)} lines)")
    return 0 if not diffs else 1


if __name__ == "__main__":
    sys.exit(main())
