#!/usr/bin/env python3
"""Byte- and relocation-compare DATA symbols between two COFF objects.

Built for replacing a C object's statics with Rust ones (the ucrt locale
takeover): the exported symbol's memory must be exactly the C object's, and a
table that is merely "equal when printed" is not good enough.

    compare_coff_data.py C.obj RUST.{o,obj,a,rlib} SYM [SYM...]
    compare_coff_data.py C.obj RUST.rlib --all            # every data symbol C defines
    compare_coff_data.py C.obj RUST.rlib --map 'CSYM=rust_substring[:SIZE]' ...
    compare_coff_data.py X.obj --list                     # what a file defines

Either side may be a COFF object or an ar archive (`.a`/`.rlib`); every COFF
member is searched. Names are C names: on i386 the leading `_` is added when
the bare name is absent. `--map C=R` pairs a C symbol with a differently named
Rust one (e.g. a file-local table whose Rust name is mangled): R is an exact
name, or else a substring that must match exactly one symbol.

Per symbol, compared:
  * extent: the distance to the next symbol / section end, per side. The
    common prefix is compared; the longer side's excess must be zero padding
    with no relocations, else it is data the other side lacks. `SYM:SIZE`
    pins the size instead.
  * bytes, with relocation sites masked out (COFF addends are stored inline);
  * relocations: same offsets, same type, and the same TARGET: a global
    symbol by name plus addend; an anonymous/local target (a string literal)
    by its CONTENT, read as a NUL-terminated narrow or UTF-16 string;
  * writability (.data/.bss vs .rdata): a const C table must not become
    writable, and vice versa.

Exit status: 0 all MATCH, 1 any DIFF, 2 usage / missing symbol.
"""

from __future__ import annotations

import argparse
import re
import struct
import sys
from dataclasses import dataclass, field

IMAGE_SCN_CNT_UNINITIALIZED_DATA = 0x80
IMAGE_SCN_MEM_WRITE = 0x80000000
IMAGE_SCN_MEM_EXECUTE = 0x20000000
IMAGE_SCN_LNK_NRELOC_OVFL = 0x01000000

MACHINES = {0x14C: "i386", 0x8664: "amd64", 0xAA64: "arm64"}
# Relocation types that carry an inline addend of this many bytes.
RELOC = {
    "i386": {
        0x06: ("DIR32", 4),
        0x07: ("DIR32NB", 4),
        0x14: ("REL32", 4),
        0x0B: ("SECREL", 4),
        0x0A: ("SECTION", 2),
    },
    "amd64": {
        0x01: ("ADDR64", 8),
        0x02: ("ADDR32", 4),
        0x03: ("ADDR32NB", 4),
        0x04: ("REL32", 4),
        0x0B: ("SECREL", 4),
        0x0A: ("SECTION", 2),
    },
    "arm64": {
        0x01: ("ADDR32", 4),
        0x02: ("ADDR32NB", 4),
        0x0E: ("ADDR64", 8),
        0x08: ("SECREL", 4),
        0x0D: ("SECTION", 2),
    },
}
BIGOBJ_CLASSID = bytes.fromhex("c7a1bad1eebaa9 4baf20faf66aa4dcb8".replace(" ", ""))


@dataclass
class Section:
    name: str
    size: int
    flags: int
    data: bytes  # zero-filled for .bss
    relocs: list = field(default_factory=list)  # (offset, symidx, type)


@dataclass
class Symbol:
    name: str
    value: int
    secnum: int  # 1-based; <=0 undefined/abs/debug
    storage: int
    index: int


class Coff:
    def __init__(self, blob: bytes, label: str):
        self.label = label
        sig1, sig2 = struct.unpack_from("<HH", blob, 0)
        if sig1 == 0 and sig2 == 0xFFFF and blob[12:28] == BIGOBJ_CLASSID:
            machine = struct.unpack_from("<H", blob, 6)[0]
            nsec, symptr, nsym = struct.unpack_from("<III", blob, 44)
            sechdr, symsize, secfmt = 56, 20, "<i"
        else:
            machine, nsec, _, symptr, nsym, opt, _ = struct.unpack_from("<HHIIIHH", blob, 0)
            sechdr, symsize, secfmt = 20 + opt, 18, "<h"
        if machine not in MACHINES:
            raise ValueError(f"{label}: not a COFF object (machine {machine:#x})")
        self.arch = MACHINES[machine]
        strtab = symptr + nsym * symsize
        self.strtab = blob[strtab:]

        def cstr(raw: bytes) -> str:
            return raw.split(b"\0", 1)[0].decode("latin-1")

        self.sections: list[Section] = []
        for i in range(nsec):
            h = sechdr + 40 * i
            raw_name = blob[h : h + 8]
            name = cstr(raw_name)
            if name.startswith("/"):
                name = self._str(int(name[1:]))
            (_vsz, _va, size, rawptr, relptr, _ln, nrel, _nln, flags) = struct.unpack_from(
                "<IIIIIIHHI", blob, h + 8
            )
            if flags & IMAGE_SCN_CNT_UNINITIALIZED_DATA or rawptr == 0:
                data = bytes(size)
            else:
                data = blob[rawptr : rawptr + size]
            relocs = []
            if flags & IMAGE_SCN_LNK_NRELOC_OVFL and nrel == 0xFFFF:
                nrel = struct.unpack_from("<I", blob, relptr)[0]
                relptr += 10
                nrel -= 1
            for r in range(nrel):
                off, sidx, typ = struct.unpack_from("<IIH", blob, relptr + 10 * r)
                relocs.append((off, sidx, typ))
            self.sections.append(Section(name, size, flags, data, relocs))

        self.symbols: dict[int, Symbol] = {}
        i = 0
        while i < nsym:
            e = symptr + i * symsize
            raw = blob[e : e + 8]
            if raw[:4] == b"\0\0\0\0":
                name = self._str(struct.unpack_from("<I", raw, 4)[0])
            else:
                name = cstr(raw)
            value = struct.unpack_from("<I", blob, e + 8)[0]
            secnum = struct.unpack_from(secfmt, blob, e + 12)[0]
            off = e + 12 + (4 if symsize == 20 else 2)
            _typ, storage, naux = struct.unpack_from("<HBB", blob, off)
            self.symbols[i] = Symbol(name, value, secnum, storage, i)
            i += 1 + naux

    def _str(self, off: int) -> str:
        return self.strtab[off:].split(b"\0", 1)[0].decode("latin-1")

    def defined(self):
        for s in self.symbols.values():
            if s.secnum > 0 and s.storage in (2, 3) and not self._is_section_sym(s):
                yield s

    def _is_section_sym(self, s: Symbol) -> bool:
        return s.storage == 3 and s.value == 0 and s.name == self.sections[s.secnum - 1].name

    def extent(self, sym: Symbol) -> int:
        """Distance from sym to the next symbol start in its section, or the end."""
        sec = self.sections[sym.secnum - 1]
        end = sec.size
        for s in self.symbols.values():
            if s.secnum == sym.secnum and sym.value < s.value < end and not s.name.startswith("$"):
                end = s.value
        return end - sym.value


def load(path: str) -> list[Coff]:
    with open(path, "rb") as f:
        blob = f.read()
    if blob[:8] != b"!<arch>\n":
        return [Coff(blob, path)]
    out, pos, longnames = [], 8, b""
    while pos + 60 <= len(blob):
        hdr = blob[pos : pos + 60]
        name = hdr[:16].decode("latin-1").rstrip()
        size = int(hdr[48:58].decode().strip())
        body = blob[pos + 60 : pos + 60 + size]
        if name == "//":
            longnames = body
        elif name.startswith("/") and name[1:].isdigit():
            n = int(name[1:])
            name = longnames[n:].split(b"\n", 1)[0].decode("latin-1").rstrip("/")
        if name not in ("/", "//", "/SYM64/") and len(body) >= 20:
            try:
                out.append(Coff(body, f"{path}({name})"))
            except (ValueError, struct.error):
                pass
        pos += 60 + size + (size & 1)
    if not out:
        raise SystemExit(f"{path}: archive holds no COFF members")
    return out


def find(objs: list[Coff], name: str, substring_ok: bool):
    hits = []
    for o in objs:
        cands = [name, "_" + name] if o.arch == "i386" else [name]
        for s in o.defined():
            if s.name in cands:
                hits.append((o, s))
    if not hits and substring_ok:
        for o in objs:
            hits += [(o, s) for s in o.defined() if name in s.name]
        # A Rust path: prefer the item whose LAST segment is the name
        # (`...<len><name>17h<hash>E`), not the statics nested inside it.
        last = [
            h
            for h in hits
            if re.search(rf"{len(name)}{re.escape(name)}17h[0-9a-f]{{16}}E$", h[1].name)
        ]
        hits = last or hits
    # Prefer an external definition over a local of the same name.
    ext = [h for h in hits if h[1].storage == 2]
    hits = ext or hits
    if len(hits) != 1:
        return None, f"{len(hits)} matches" if hits else "not defined"
    return hits[0], None


def target_desc(o: Coff, sec: Section, off: int, symidx: int, typ: int):
    """What a relocation at `off` points at, independent of object layout:
    (type name, width, target name or None, addend, content at the target)."""
    tname, width = RELOC[o.arch].get(typ, (f"type{typ:#x}", 4))
    addend = int.from_bytes(sec.data[off : off + width], "little", signed=True)
    tsym = o.symbols[symidx]
    if tsym.secnum <= 0:  # undefined here: only the name can be compared
        return tname, width, tsym.name, addend, None
    tsec = o.sections[tsym.secnum - 1]
    content = read_string(tsec.data, tsym.value + addend)
    # A C-named global is compared by name. A section symbol, a private label,
    # or a compiler-named global (a Rust `static` inside an initializer is
    # emitted global with a mangled name) is an anonymous literal: by content.
    anonymous = (
        tsym.storage != 2
        or o._is_section_sym(tsym)
        or re.match(r"_{1,2}(ZN|R[NICM])", tsym.name) is not None  # Rust legacy / v0 mangling
    )
    return tname, width, None if anonymous else tsym.name, addend, content


def same_target(c, r) -> bool:
    (_, _, cname, cadd, ccont), (_, _, rname, radd, rcont) = c, r
    if cname is not None and rname is not None:
        return cname == rname and cadd == radd
    if cname is None and rname is None:
        return ccont is not None and ccont == rcont
    return False  # a named object on one side, an anonymous one on the other


def show(d) -> str:
    _, _, name, addend, content = d
    if name is not None:
        return f"{name}{addend:+#x}"
    return f"<anon {content[0]} {content[1]!r}>"


def read_string(data: bytes, at: int):
    """A NUL-terminated string at `at`: UTF-16 when the second byte is 0 and the
    first is not (every ASCII wide string), else narrow. "" if it starts at 0."""
    if at >= len(data) or data[at] == 0:
        return ("str", "")
    if at + 1 < len(data) and data[at + 1] == 0:
        units = []
        i = at
        while i + 1 < len(data):
            u = data[i] | data[i + 1] << 8
            if u == 0:
                return ("wstr", "".join(map(chr, units)))
            units.append(u)
            i += 2
        return ("wstr-unterminated", "".join(map(chr, units)))
    end = data.find(b"\0", at)
    if end < 0:
        return ("str-unterminated", data[at:].decode("latin-1"))
    return ("str", data[at:end].decode("latin-1"))


def compare(c: tuple, r: tuple, size: int | None, verbose: bool = False):
    (co, cs), (ro, rs) = c, r
    csec, rsec = co.sections[cs.secnum - 1], ro.sections[rs.secnum - 1]
    problems = []
    cw, rw = bool(csec.flags & IMAGE_SCN_MEM_WRITE), bool(rsec.flags & IMAGE_SCN_MEM_WRITE)
    if cw != rw:
        problems.append(
            f"writability: C {'rw' if cw else 'ro'} ({csec.name}) vs "
            f"Rust {'rw' if rw else 'ro'} ({rsec.name})"
        )
    cext, rext = co.extent(cs), ro.extent(rs)
    n = size if size is not None else min(cext, rext)
    if size is not None and (cext < size or rext < size):
        problems.append(f"extent: C {cext}, Rust {rext}, both must be >= {size}")
        n = min(cext, rext, size)

    def relmap(o, s, sec):
        return {
            off - s.value: (off, idx, typ)
            for off, idx, typ in sec.relocs
            if s.value <= off < s.value + max(o.extent(s), n)
        }

    crel, rrel = relmap(co, cs, csec), relmap(ro, rs, rsec)
    masked = set()
    matched = []
    for rel_off in sorted(set(crel) | set(rrel)):
        if rel_off >= n:
            side = "C" if rel_off in crel else "Rust"
            problems.append(f"+{rel_off:#x}: relocation beyond the compared extent ({side})")
            continue
        if rel_off not in crel or rel_off not in rrel:
            side = "C only" if rel_off in crel else "Rust only"
            problems.append(f"+{rel_off:#x}: relocation {side}")
            continue
        cd = target_desc(co, csec, *crel[rel_off])
        rd = target_desc(ro, rsec, *rrel[rel_off])
        masked.update(range(rel_off, rel_off + max(cd[1], rd[1])))
        if cd[0] != rd[0]:
            problems.append(f"+{rel_off:#x}: reloc type C {cd[0]} vs Rust {rd[0]}")
        if not same_target(cd, rd):
            problems.append(f"+{rel_off:#x}: target C {show(cd)} vs Rust {show(rd)}")
        else:
            matched.append(f"+{rel_off:#x} {cd[0]} -> {show(cd)}")
    cb = csec.data[cs.value : cs.value + n]
    rb = rsec.data[rs.value : rs.value + n]
    diffs = [i for i in range(n) if i not in masked and cb[i] != rb[i]]
    if diffs:
        first = diffs[0]
        problems.append(
            f"{len(diffs)} byte(s) differ, first at +{first:#x}: "
            f"C {cb[first : first + 8].hex()} vs Rust {rb[first : first + 8].hex()}"
        )
    if size is None:
        for label, s, sec, ext in (("C", cs, csec, cext), ("Rust", rs, rsec, rext)):
            tail = sec.data[s.value + n : s.value + ext]
            if any(tail):
                problems.append(
                    f"extent: {label} has {ext - n} more byte(s) than the other "
                    f"side and they are not zero padding (C {cext}, Rust {rext})"
                )
    return problems, n, len(crel), matched if verbose else []


def main() -> int:
    ap = argparse.ArgumentParser(description=(__doc__ or "").split("\n\n")[0])
    ap.add_argument("c_obj")
    ap.add_argument("rust_obj", nargs="?")
    ap.add_argument("symbols", nargs="*", help="SYM or SYM:SIZE")
    ap.add_argument("--map", action="append", default=[], metavar="C=RUST[:SIZE]")
    ap.add_argument("--all", action="store_true", help="every data symbol the C object defines")
    ap.add_argument("--list", action="store_true", help="list data symbols and exit")
    ap.add_argument("-v", "--verbose", action="store_true", help="print every matched relocation")
    a = ap.parse_args()

    cobjs = load(a.c_obj)
    if a.list or a.rust_obj is None:
        for o in cobjs + (load(a.rust_obj) if a.rust_obj else []):
            for s in o.defined():
                sec = o.sections[s.secnum - 1]
                if sec.flags & IMAGE_SCN_MEM_EXECUTE:
                    continue
                print(
                    f"{o.label}: {s.name}  {sec.name}+{s.value:#x}  extent {o.extent(s)}"
                    f"  {'global' if s.storage == 2 else 'local'}"
                )
        return 0
    robjs = load(a.rust_obj)
    if cobjs[0].arch != robjs[0].arch:
        print(f"arch mismatch: C {cobjs[0].arch}, Rust {robjs[0].arch}")
        return 2

    work = []
    for spec in a.symbols:
        name, _, size = spec.partition(":")
        work.append((name, name, int(size, 0) if size else None))
    for m in a.map:
        cname, _, rname = m.partition("=")
        rname, _, size = rname.partition(":")
        work.append((cname, rname, int(size, 0) if size else None))
    if a.all:
        named = {w[0] for w in work}
        for o in cobjs:
            for s in o.defined():
                if s.storage != 2 or o.sections[s.secnum - 1].flags & IMAGE_SCN_MEM_EXECUTE:
                    continue
                if s.name.startswith(".refptr."):  # mingw's compiler-made indirection
                    continue
                cname = s.name[1:] if o.arch == "i386" and s.name.startswith("_") else s.name
                if cname not in named:
                    work.append((cname, cname, None))
    if not work:
        if a.all:
            print(f"{a.c_obj}: defines no data symbols")
            return 0
        ap.error("no symbols given")

    rc = 0
    for cname, rname, size in work:
        c, err = find(cobjs, cname, False)
        if c is None:
            print(f"MISSING  {cname}: C side {err}")
            rc = max(rc, 2)
            continue
        r, err = find(robjs, rname, rname != cname)
        if r is None:
            print(f"MISSING  {cname}: Rust side ({rname}) {err}")
            rc = max(rc, 2)
            continue
        problems, n, nrel, matched = compare(c, r, size, a.verbose)
        verdict = "MATCH" if not problems else "DIFF "
        print(f"{verdict}  {cname}  [{cobjs[0].arch}] {n} bytes, {nrel} relocs")
        for p in problems + matched:
            print(f"           {p}")
        if problems:
            rc = max(rc, 1)
    return rc


if __name__ == "__main__":
    sys.exit(main())
