#!/usr/bin/env python3
"""Line out the Windows 10 export surface as dormant stubs, and rank what real programs need.

WHY THIS EXISTS
---------------
We report NT 5.2 and build exports for `DLL_EXPORT_VERSION=0x502`. Moving toward Win10 is
not a version-number change: a program built for Win10 fails on IMPORTS it cannot resolve,
and a version bump alone turns well-behaved programs' XP fallbacks into NULL calls. The work
is the export surface, and its size had only ever been guessed. This tool measures it from
two things that cannot drift: the export tables of a real Windows install, and our `.spec`s.

    reference   what a Windows install exports, per module we build (x64 System32, x86
                SysWOW64, kernel-mode from System32/drivers). Cached as JSON.
    stubs       write each module's missing names into its `.spec` as `@ stub` lines tagged
                `-version=0xA00+`, inside one generated block the tool owns and rewrites.
    verify      prove the block is dormant: spec2def's output at the tree's default version
                is byte-identical with and without it, and at 0xA00 it still generates.
    gap         rank what a corpus of real programs imports that we don't export.
    gated       for every row Windows exports that our spec gates behind a newer
                `-version`: is its body already LINKED into the built image (`--apply`
                exports it), imported from another DLL (a `forwards` provider), or absent?
                "absent" means absent from the IMAGE: a body in a static library the
                module links but nothing references yet is pulled in only by exporting it
                (ntoskrnl's RtlUTF8ToUnicodeN, in the utf8 crate via ntoskrnl_vista).

DORMANT ON PURPOSE
------------------
Every generated line is `-version=0xA00+`, so the shipping build (0x502) never sees one —
`verify` checks exactly that rather than trusting it. A stub only does something once a build
raises DLL_EXPORT_VERSION, and then it is a LOUD stub: spec2def's stub prints
`WARNING: calling stub X` and raises the noncontinuable EXCEPTION_WINE_STUB. So a program
that merely imports a name loads; one that calls it fails at the call, naming it. Nothing here
writes a body.

THE TIER IS "PRESENT IN THE REFERENCE", NOT "INTRODUCED IN WIN10"
-----------------------------------------------------------------
A name missing from our spec may be NT4-era and simply never declared by ReactOS. Nothing in
an export table dates it, so every generated line carries the same 0xA00 tier and the block
header names the reference build it was measured against.

WHAT IS LEFT OUT, AND SAID SO
-----------------------------
  * DATA exports (the RVA lands in a non-executable section): a code stub there is wrong, not
    dormant, so they are listed as comments for a human to give an `extern`.
  * Ordinal-only exports: no name to declare.
  * Forwarders become ordinary stubs; the Windows target rides along as a trailing comment
    (`# -> NTDLL.RtlFoo`) because it is the best hint for whoever implements it.
"""

from __future__ import annotations

import argparse
import json
import mmap
import os
import re
import struct
import subprocess
import sys
import tempfile
from collections import Counter, defaultdict
from collections.abc import Iterable, Iterator
from dataclasses import dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "tools" / "payloads"))
from pecheck import export_version, spec_exports  # noqa: E402
from source_walk import source_walk  # noqa: E402

TIER = "0xA00"
BEGIN = "# BEGIN win10-surface stubs"
END = "# END win10-surface stubs"
FWD_BEGIN = "# BEGIN vista forwards"
FWD_END = "# END vista forwards"
# A Windows module whose newer exports our tree implements in a SEPARATE DLL, built and staged
# on every arch: the 2003-era module gates them behind -version=0x600+, and the *_vista DLL
# exports them unconditionally. A forward makes the real export reachable from the module
# programs import it from. (uxtheme_vista exists but is not staged on the media.)
VISTA = {
    "kernel32.dll": "kernel32_vista",
    "ntdll.dll": "ntdll_vista",
    "advapi32.dll": "advapi32_vista",
    "gdi32.dll": "gdi32_vista",
    "user32.dll": "user32_vista",
}
# Who may implement a module's missing exports, in order of preference. Windows'
# kernelbase.dll hosts most of kernel32's and much of advapi32's surface, and programs
# (and api sets) import from it directly; ours is a minimal shim, while kernel32 and
# advapi32 (with their vista forwards) have the bodies.
PROVIDERS: dict[str, list[str]] = {
    **{m: [v] for m, v in VISTA.items()},
    "kernelbase.dll": ["kernel32", "advapi32"],
    # Vista moved the Winsock helper API from ws2help into ws2_32; ws2help keeps it here.
    "ws2_32.dll": ["ws2help"],
}
# Windows' kernel32 forwards the registry, token, SID and ETW calls to advapi32 (by way
# of kernelbase); programs import them from kernel32. Ours has them in advapi32 only.
PROVIDERS["kernel32.dll"] = [*PROVIDERS["kernel32.dll"], "advapi32"]
# Providers that themselves link the module's import library: a forward to one is
# `-private`, kept out of that import library, or the provider's own definitions collide
# with the import entries for them (lld: "WahCloseNotificationHandleHelper was replaced").
LINKS_BACK = {("ws2_32.dll", "ws2help"), ("kernel32.dll", "advapi32")}
ARCHES = ("i386", "x86_64", "arm64")
CACHE = Path.home() / ".cache" / "vibeos" / "win10-surface"
# The trees that build shipping modules. modules/ builds test DLLs. sdk/lib is IN: wdfldr.sys,
# the KMDF loader 227 driver packages import, builds from sdk/lib/drivers, and leaving it out
# reported it as a Windows module we do not build.
MODULE_TREES = ("dll", "win32ss", "ntoskrnl", "hal", "drivers", "base", "subsystems", "sdk/lib")
# spec2def splits a line on whitespace and `,`-and-below punctuation; a name with anything
# outside this set would not survive as one token.
NAME_OK = re.compile(r"^[A-Za-z_?$@][A-Za-z0-9_?$@<>~.-]*$")
SCN_MEM_EXECUTE = 0x20000000
MACHINES = {0x14C: "i386", 0x8664: "x86_64", 0xAA64: "arm64"}


# --- PE --------------------------------------------------------------------------------------


class PEError(Exception):
    pass


@dataclass
class Export:
    name: str
    data: bool = False
    forward: str | None = None


@dataclass
class PE:
    machine: str
    exports: dict[str, Export] = field(default_factory=dict)
    noname: int = 0
    imports: dict[str, list[str]] = field(default_factory=dict)  # dll (lower) -> names


def read_pe(data: bytes | mmap.mmap) -> PE:
    """Exports (with data/forwarder classification) and imports (incl. delay-load).

    Its own reader rather than pecheck.parse because it needs three things pecheck does
    not: the export table, which section an RVA lands in, and the delay-import table —
    a program that delay-loads a function needs it exactly as much once it calls it.
    """
    try:
        return _read_pe(data)
    except (struct.error, ValueError, IndexError) as e:
        raise PEError(f"truncated or malformed image: {e}") from e


def _read_pe(data: bytes | mmap.mmap) -> PE:
    if data[:2] != b"MZ":
        raise PEError("not an MZ image")
    pe = struct.unpack_from("<I", data, 0x3C)[0]
    if data[pe : pe + 4] != b"PE\0\0":
        raise PEError("no PE signature")
    machine, nsect = struct.unpack_from("<HH", data, pe + 4)
    optsize = struct.unpack_from("<H", data, pe + 20)[0]
    opt = pe + 24
    magic = struct.unpack_from("<H", data, opt)[0]
    if magic not in (0x10B, 0x20B):
        raise PEError(f"unknown optional-header magic {magic:#x}")
    wide = magic == 0x20B
    image_base = struct.unpack_from("<Q" if wide else "<I", data, opt + (24 if wide else 28))[0]
    ndirs = struct.unpack_from("<I", data, opt + (108 if wide else 92))[0]
    ddir = opt + (112 if wide else 96)

    def directory(i: int) -> tuple[int, int]:
        if i >= ndirs:
            return 0, 0
        return struct.unpack_from("<II", data, ddir + 8 * i)

    sections = []
    for i in range(nsect):
        sh = opt + optsize + 40 * i
        vsize, va, rawsize, rawptr = struct.unpack_from("<IIII", data, sh + 8)
        chars = struct.unpack_from("<I", data, sh + 36)[0]
        sections.append((va, max(vsize, rawsize), rawptr, chars))

    def section(rva: int) -> tuple[int, int, int, int]:
        for s in sections:
            if s[0] <= rva < s[0] + s[1]:
                return s
        raise PEError(f"RVA {rva:#x} is in no section")

    def off(rva: int) -> int:
        va, _, raw, _ = section(rva)
        return raw + (rva - va)

    def cstr(rva: int) -> str:
        o = off(rva)
        end = data.find(b"\0", o)
        if end < 0:
            raise PEError(f"unterminated string at RVA {rva:#x}")
        return data[o:end].decode("latin-1")

    out = PE(machine=MACHINES.get(machine, f"{machine:#x}"))

    exp_rva, exp_size = directory(0)
    if exp_rva:
        e = off(exp_rva)
        nfuncs, nnames, funcs, names, ords = struct.unpack_from("<IIIII", data, e + 20)
        named: set[int] = set()
        for i in range(nnames):
            name = cstr(struct.unpack_from("<I", data, off(names) + 4 * i)[0])
            idx = struct.unpack_from("<H", data, off(ords) + 2 * i)[0]
            named.add(idx)
            rva = struct.unpack_from("<I", data, off(funcs) + 4 * idx)[0]
            if exp_rva <= rva < exp_rva + exp_size:
                out.exports[name] = Export(name, forward=cstr(rva))
            else:
                try:
                    chars = section(rva)[3]
                except PEError:
                    chars = 0
                out.exports[name] = Export(name, data=not chars & SCN_MEM_EXECUTE)
        out.noname = sum(
            1
            for i in range(nfuncs)
            if i not in named and struct.unpack_from("<I", data, off(funcs) + 4 * i)[0]
        )

    step, fmt = (8, "<Q") if wide else (4, "<I")
    ordinal_bit = 1 << (63 if wide else 31)

    def thunks(rva: int) -> list[str]:
        names: list[str] = []
        t = off(rva)
        while True:
            v = struct.unpack_from(fmt, data, t)[0]
            if not v:
                return names
            names.append(f"#{v & 0xFFFF}" if v & ordinal_bit else cstr((v & 0x7FFFFFFF) + 2))
            t += step

    imp_rva, _ = directory(1)
    if imp_rva:
        d = off(imp_rva)
        while True:
            oft, _, _, name_rva, ft = struct.unpack_from("<IIIII", data, d)
            if not (oft or name_rva or ft):
                break
            out.imports.setdefault(cstr(name_rva).lower(), []).extend(thunks(oft or ft))
            d += 20

    dly_rva, _ = directory(13)
    if dly_rva:
        d = off(dly_rva)
        while True:
            attrs, name_rva, _, _, int_rva = struct.unpack_from("<IIIII", data, d)
            if not (name_rva or int_rva):
                break
            # Attribute bit 0 clear is the VC6-era layout: the fields are VAs, not RVAs.
            rebase = 0 if attrs & 1 else image_base
            out.imports.setdefault(cstr(name_rva - rebase).lower(), []).extend(
                thunks(int_rva - rebase)
            )
            d += 32
    return out


def read_pe_file(path: Path) -> PE:
    """mmap, not read_bytes: over /mnt/c (9p) a full read of every image made a corpus scan
    I/O-bound at 3% CPU for 12+ minutes (chrome.dll alone is ~250MB). The headers and the
    export/import tables are a few pages, and a mapping faults in only those."""
    with path.open("rb") as fh:
        try:
            m = mmap.mmap(fh.fileno(), 0, access=mmap.ACCESS_READ)
        except (ValueError, OSError):  # empty file, or a filesystem without mmap
            return read_pe(fh.read())
        with m:
            return read_pe(m)


# --- API sets --------------------------------------------------------------------------------


def read_apiset_schema(data: bytes) -> dict[str, str]:
    """{api set name prefix (lower, no `.dll`): default host dll (lower)} from the schema v6.

    The v6 namespace (Win10+) lives in apisetschema.dll's `.apiset` section. Each entry's
    name is matched on its first HashedLength bytes — `api-ms-win-core-file-l1-2` — so
    every minor revision (`-l1-2-0`, `-l1-2-1`) resolves through the same entry.
    """
    pe = struct.unpack_from("<I", data, 0x3C)[0]
    nsect = struct.unpack_from("<H", data, pe + 6)[0]
    optsize = struct.unpack_from("<H", data, pe + 20)[0]
    base = None
    for i in range(nsect):
        sh = pe + 24 + optsize + 40 * i
        if data[sh : sh + 8].rstrip(b"\0") == b".apiset":
            base = struct.unpack_from("<I", data, sh + 20)[0]
    if base is None:
        raise PEError("no .apiset section")
    version, _, _, count, entry_off = struct.unpack_from("<IIIII", data, base)
    if version != 6:
        raise PEError(f"api set schema v{version}; only v6 (Win10+) is understood")

    def wstr(o: int, n: int) -> str:
        return data[base + o : base + o + n].decode("utf-16-le")

    out: dict[str, str] = {}
    for i in range(count):
        _, name_off, _, hashed, value_off, value_count = struct.unpack_from(
            "<IIIIII", data, base + entry_off + 24 * i
        )
        host = ""
        for j in range(value_count):
            _, _vn_off, vn_len, vv_off, vv_len = struct.unpack_from(
                "<IIIII", data, base + value_off + 20 * j
            )
            if vn_len == 0 or not host:  # the default (importer-independent) host wins
                host = wstr(vv_off, vv_len)
        if host:
            out[wstr(name_off, hashed).lower()] = host.lower()
    return out


def resolve_apiset(dll: str, schema: dict[str, str]) -> str | None:
    """The host DLL behind `api-ms-win-…` / `ext-ms-…`, or None if it is not an api set."""
    stem = dll.lower().removesuffix(".dll")
    if not stem.startswith(("api-", "ext-")):
        return None
    prefix = stem.rsplit("-", 1)[0]
    return schema.get(prefix, "")


ROS_APISETS = ROOT / "sdk" / "lib" / "apisets" / "apisets.table.c"
_ROS_APISET_ROW = re.compile(
    r'^\s*\{\s*RTL_CONSTANT_STRING\(L"([^"]+)"\),\s*RTL_CONSTANT_STRING\(L"([^"]*)"\)', re.M
)


def read_ros_apisets(text: str) -> dict[str, str]:
    """{api set name (lower, no `.dll`): host dll (lower)} from OUR loader's table.

    The loader (ntdll's LdrpApplyFileNameRedirection -> ApiSetResolveToHost) redirects an
    imported `api-ms-win-…` name through this table (see `ros_apiset_host`), and that decides
    whether an import loads here -- not Windows' schema, which maps most core sets to
    kernelbase.dll where ours maps them to kernel32.dll. Commented-out rows are not entries.
    """
    return {m.group(1).lower(): m.group(2).lower() for m in _ROS_APISET_ROW.finditer(text)}


def ros_apiset_host(name: str, ros: dict[str, str]) -> str | None:
    """The host our loader redirects api set `name` to: the exact row, else -- as Windows
    does, hashing a name only up to its last '-' -- the last row differing only in the
    trailing minor version (`realtime-l1-1-1` -> the `realtime-l1-1-2` row)."""
    name = name.lower().removesuffix(".dll")
    if name in ros:
        return ros[name]
    stem, _, minor = name.rpartition("-")
    if not minor.isdigit():
        return None
    rows = sorted(k for k in ros if k.rpartition("-")[0] == stem and k.rpartition("-")[2].isdigit())
    return ros[rows[-1]] if rows else None


# --- the tree --------------------------------------------------------------------------------


def module_specs(root: Path = ROOT) -> dict[str, list[Path]]:
    """{module file name (lower): [spec]} from every literal `spec2def(<name> <spec>)` call.

    Read from CMake rather than guessed from the spec's stem, because the stem lies for
    kernel modules (`ntoskrnl.exe` builds from `ntoskrnl.spec`, a driver's spec is often
    named for its directory) and the module name is what a Windows install calls it.
    """
    out: dict[str, list[Path]] = defaultdict(list)
    for top in MODULE_TREES:
        for cm in (root / top).rglob("CMakeLists.txt"):
            for m in re.finditer(r"\bspec2def\(\s*([^\s$)]+)\s+([^\s$)]+\.spec)", cm.read_text()):
                spec = (cm.parent / m.group(2)).resolve()
                # hal and win32k declare their module twice (per-arch CMake branches).
                if spec.is_file() and spec not in out[m.group(1).lower()]:
                    out[m.group(1).lower()].append(spec)
    # A module built through a CMake function has no literal call to find
    # (d3dcompiler_47 is `add_d3dcompiler_target(47)`, and gap reported it missing):
    # a directory whose CMakeLists has no spec2def of its own but sits beside
    # `<dir>.spec` builds `<dir>.dll`.
    known = {p for specs in out.values() for p in specs}
    for top in MODULE_TREES:
        for cm in (root / top).rglob("CMakeLists.txt"):
            spec = (cm.parent / f"{cm.parent.name}.spec").resolve()
            if spec.is_file() and spec not in known and "spec2def" not in cm.read_text():
                out[f"{cm.parent.name.lower()}.dll"].append(spec)
    return dict(out)


def declared_names(spec: Path) -> set[str]:
    """Every name the spec declares under ANY version or arch, generated block excluded.

    The generated block is excluded so a re-run measures against the hand-maintained spec;
    otherwise it would see its own previous output as "already declared" and never shrink
    the block when a name gets a real line.
    """
    text = strip_block(spec.read_text(errors="replace"))
    with tempfile.NamedTemporaryFile("w", suffix=".spec", delete=False) as fh:
        fh.write(text)
        tmp = Path(fh.name)
    try:
        names: set[str] = set()
        for arch in ("i386", "x86_64", "arm64"):
            names |= set(spec_exports(tmp, arch, 0xFFF))
        return names
    finally:
        tmp.unlink()


def strip_block(text: str, begin: str = BEGIN, end: str = END) -> str:
    return re.sub(rf"\n?{re.escape(begin)}.*?{re.escape(end)}\n?", "\n", text, flags=re.S)


# --- reference -------------------------------------------------------------------------------


def _windows_files(win: Path, module: str) -> tuple[Path | None, Path | None]:
    """(x64 image, x86 image) for one module name in a Windows install."""
    sys32, wow = win / "System32", win / "SysWOW64"
    x64 = next((p for p in (sys32 / module, sys32 / "drivers" / module) if p.is_file()), None)
    x86 = wow / module if (wow / module).is_file() else None
    return x64, x86


def windows_build(win: Path) -> str:
    """The reference's build, read from ntoskrnl's version resource (no registry needed)."""
    data = (win / "System32" / "ntoskrnl.exe").read_bytes()
    sig = data.find(struct.pack("<I", 0xFEEF04BD))
    if sig < 0:
        return "unknown"
    ms, ls = struct.unpack_from("<II", data, sig + 8)
    return f"{ms >> 16}.{ms & 0xFFFF}.{ls >> 16}.{ls & 0xFFFF}"


def build_reference(win: Path, root: Path = ROOT) -> dict:
    mods: dict[str, dict] = {}
    for module in sorted(module_specs(root)):
        x64, x86 = _windows_files(win, module)
        if not x64 and not x86:
            continue
        entry: dict[str, dict] = {}
        for tag, path in (("x64", x64), ("x86", x86)):
            if not path:
                continue
            try:
                pe = read_pe_file(path)
            except (PEError, OSError) as e:
                print(f"win10_surface: skip {path}: {e}", file=sys.stderr)
                continue
            entry[tag] = {
                "exports": {
                    n: {"data": e.data, "forward": e.forward} for n, e in pe.exports.items()
                },
                "noname": pe.noname,
            }
        if entry:
            mods[module] = entry
    return {"windows": str(win), "build": windows_build(win), "modules": mods}


def load_reference(path: Path) -> dict:
    if not path.is_file():
        sys.exit(
            f"win10_surface: no reference at {path}\n"
            "  fix: python3 tools/win10_surface.py reference --windows /mnt/c/Windows"
        )
    return json.loads(path.read_text())


# --- stubs -----------------------------------------------------------------------------------


@dataclass
class Plan:
    module: str
    spec: Path
    lines: list[str]
    data: list[str]
    skipped: list[str]


def plan_module(module: str, spec: Path, ref: dict) -> Plan:
    declared = declared_names(spec)
    x64 = ref.get("x64", {}).get("exports", {})
    x86 = ref.get("x86", {}).get("exports", {})
    have_x86 = "x86" in ref
    lines, data, skipped = [], [], []
    for name in sorted(set(x64) | set(x86)):
        if name in declared:
            continue
        if not NAME_OK.match(name):
            skipped.append(name)
            continue
        info = x64.get(name) or x86[name]
        if info["data"]:
            data.append(name)
            continue
        # Only an x86 reference can say a name is 64-bit-only; a kernel module has none,
        # and then the line is unrestricted rather than guessed.
        arch = ""
        if have_x86 and name in x64 and name not in x86:
            arch = " -arch=win64"
        elif name in x86 and name not in x64:
            arch = " -arch=win32"
        tail = f" # -> {info['forward']}" if info["forward"] else ""
        lines.append(f"@ stub{arch} -version={TIER}+ {name}{tail}")
    return Plan(module, spec, lines, data, skipped)


def render_block(plan: Plan, build: str) -> str:
    head = [
        BEGIN,
        f"# Generated by tools/win10_surface.py from Windows {build}; do not edit by hand.",
        "# Names that build exports and this spec does not declare. Dormant below",
        f"# DLL_EXPORT_VERSION {TIER}; give a name a real line above and re-run to drop it.",
    ]
    body = list(plan.lines)
    if plan.data:
        body.append("# Data exports (need an `extern`, not a code stub):")
        body += [f"#   {n}" for n in plan.data]
    return "\n".join([*head, *body, END]) + "\n"


class HandEditedBlock(Exception):
    """A generated block holds a row the regeneration would drop with nothing replacing it."""


def hand_edits(old: str, block: str | None, begin: str, end: str) -> list[str]:
    """Rows of the generated block on disk that the new block drops while no row outside
    the block declares the name: an edit made INSIDE the block, which regenerating would
    silently revert. A row that leaves because the name got a real line elsewhere (the
    documented way to take one over) is not one of these.

    The incident: an agent turned advapi32's dormant `@ stub EventRegister` rows into
    forwards in place, `stubs --apply` restored the stubs, and the build shipped without
    the eight exports while every spec diff looked finished.
    """
    m = re.search(rf"{re.escape(begin)}(.*?){re.escape(end)}", old, flags=re.S)
    if m is None:
        return []
    kept = set((block or "").splitlines())
    outside = spec_rows(strip_block(strip_block(old), FWD_BEGIN, FWD_END))
    edits = []
    for line in m.group(1).splitlines():
        if line in kept or not line.startswith("@") or _generated_row(line, begin):
            continue
        if not any(name in outside for name in spec_rows(line)):
            edits.append(line)
    return edits


def _generated_row(line: str, begin: str) -> bool:
    """A row in the form the generator writes: dormant at the tier (stubs block), or a
    same-name forward (forwards block). Those come and go with the reference and the
    providers; anything else in the block was typed there."""
    rows = [r for rs in spec_rows(line).values() for r in rs]
    if not rows:
        return True
    if begin == BEGIN:
        return f"-version={TIER}+" in rows[0].opts
    name = next(iter(spec_rows(line)))
    return bool(re.search(rf"\s\w+\.{re.escape(name)}\s*(#.*)?$", line))


def apply_block(
    spec: Path, block: str | None, begin: str = BEGIN, end: str = END, force: bool = False
) -> bool:
    """Write (or remove, when block is None) the generated block. True if the file changed.

    Byte-preserving outside the block: latin-1 round-trips any byte, and the hand-written
    part is never re-flowed. A first version normalized trailing blank lines and a missing
    final newline, which put a spurious hunk in 231 specs that the block never touched.

    Refuses (HandEditedBlock) when the block on disk carries a hand edit the rewrite would
    discard, unless `force`: move the row above the block, then re-run.
    """
    old = spec.read_bytes().decode("latin-1")
    if not force and (edits := hand_edits(old, block, begin, end)):
        raise HandEditedBlock(
            f"{spec}: {len(edits)} row(s) inside the generated block would be discarded:\n"
            + "\n".join(f"    {e}" for e in edits)
            + "\n  Move each above the block as a real line (or pass --force to drop them)."
        )
    pat = re.compile(rf"\n?{re.escape(begin)}.*?{re.escape(end)}\n", flags=re.S)
    if block is None:
        new = pat.sub("", old)
    elif pat.search(old):
        # In place: a spec carries two generated blocks (stubs, vista forwards), and
        # re-appending moved each to the end, so running one tool after the other swapped
        # them and rewrote specs whose content had not changed -- new mtimes, stale media.
        new = pat.sub(lambda _: "\n" + block, old, count=1)
    else:
        new = old + ("\n" if old and not old.endswith("\n") else "") + "\n" + block
    if new == old:
        return False
    spec.write_bytes(new.encode("latin-1"))
    return True


def specs_for(modules: dict[str, list[Path]], ref: dict) -> Iterator[tuple[str, Path, dict]]:
    """(module, spec, its reference) — skipping names two modules share a spec for twice."""
    seen: set[Path] = set()
    for module, specs in sorted(modules.items()):
        if module not in ref["modules"]:
            continue
        for spec in specs:
            if spec in seen:
                continue
            seen.add(spec)
            yield module, spec, ref["modules"][module]


# --- vista forwards --------------------------------------------------------------------------


@dataclass
class SpecRow:
    kind: str  # stdcall / cdecl / stub / ...
    opts: list[str]
    name: str
    args: str  # "(ptr long)" or ""
    target: str  # forward target, or ""


def spec_rows(text: str) -> dict[str, list[SpecRow]]:
    """{name: every row declaring it}, from spec text (comments stripped)."""
    out: dict[str, list[SpecRow]] = defaultdict(list)
    for line in text.splitlines():
        toks = line.split("#", 1)[0].split()
        if len(toks) < 3 or not (toks[0] == "@" or toks[0].isdigit()):
            continue
        kind, rest = toks[1], toks[2:]
        opts = []
        while rest and rest[0].startswith("-"):
            opts.append(rest.pop(0))
        if not rest:
            continue
        m = re.match(r"([^(\s]+)(\([^)]*\))?", " ".join(rest))
        if not m:
            continue
        after = " ".join(rest)[m.end() :].split()
        out[m.group(1)].append(
            SpecRow(kind, opts, m.group(1), m.group(2) or "", after[0] if after else "")
        )
    return out


def _stub_flag(opts: list[str]) -> bool:
    """`-stub`, or `-stub=VALUE` (the stub that returns VALUE instead of raising)."""
    return any(o == "-stub" or o.startswith("-stub=") for o in opts)


# Flags that describe the export itself and carry over to a forward of it; the rest
# (-version, -arch, -stub, -i386) say WHEN a row applies, which the forward sets itself.
KEEP_FLAGS = ("-ret64", "-noname", "-private", "-norelay", "-register")


def plan_forwards(
    module: str, spec: Path, providers: list[tuple[str, Path]], ref: dict, version: int
) -> list[str]:
    """Forward lines for every name Windows' `module` exports, our default build does not,
    and a provider does (the first that has it): on exactly the arches where that holds,
    and below the version where the module's own row (if any) takes over, so the two never
    both apply."""
    text = strip_block(strip_block(spec.read_bytes().decode("latin-1")), FWD_BEGIN, FWD_END)
    rows = spec_rows(text)
    with tempfile.NamedTemporaryFile("w", suffix=".spec", delete=False) as fh:
        fh.write(text)
        tmp = Path(fh.name)
    try:
        mine = {a: spec_exports(tmp, a, version) for a in ARCHES}
    finally:
        tmp.unlink()
    prov = [
        (
            pmod,
            {a: spec_exports(pspec, a, version) for a in ARCHES},
            spec_rows(pspec.read_bytes().decode("latin-1")),
        )
        for pmod, pspec in providers
    ]
    win = {**ref.get("x86", {}).get("exports", {}), **ref.get("x64", {}).get("exports", {})}
    lines = []
    for name in sorted(win):
        if win[name]["data"]:
            continue
        # The first provider that exports the name, on the arches where the module does not.
        hit = next(
            (
                (pmod, arches, prows)
                for pmod, theirs, prows in prov
                if (
                    arches := [
                        a
                        for a in ARCHES
                        if mine[a].get(name) != "ok" and theirs[a].get(name) == "ok"
                    ]
                )
            ),
            None,
        )
        if hit is None:
            continue
        vista_mod, arches, theirs_rows = hit
        # The signature comes from the module's own row when it has a real one, else from
        # the provider's (a `stub` row carries no calling convention).
        own = [r for r in rows.get(name, []) if r.kind != "stub" and not _stub_flag(r.opts)]
        theirs_real = [
            r for r in theirs_rows.get(name, []) if r.kind != "stub" and not _stub_flag(r.opts)
        ]
        sig = (own or theirs_real or theirs_rows[name])[0]
        starts = [
            int(r.split("-")[0].rstrip("+"), 16)
            for row in rows.get(name, [])
            for o in row.opts
            if o.startswith("-version=")
            for r in o[9:].split(",")
        ]
        until = min(starts) - 1 if starts else None
        if until is not None and until < version:
            continue  # the module's own row already covers the default version
        opt = "" if len(arches) == len(ARCHES) else f" -arch={','.join(arches)}"
        if until is not None:
            opt += f" -version=0x400-{until:#x}"
        flags = "".join(f" {o}" for o in sig.opts if o in KEEP_FLAGS)
        if (module, vista_mod) in LINKS_BACK and "-private" not in flags:
            flags += " -private"
        lines.append(f"@ {sig.kind}{opt}{flags} {name}{sig.args} {vista_mod}.{name}")
    return lines


def render_forwards(module: str, providers: list[str], lines: list[str]) -> str:
    head = [
        FWD_BEGIN,
        "# Generated by tools/win10_surface.py forwards; do not edit by hand. Exports Windows'",
        f"# {module} has and this build would not, forwarded to the implementation in "
        f"{' / '.join(providers)}.",
        "# Each switches off where this spec's own row for the name takes over.",
    ]
    return "\n".join([*head, *lines, FWD_END]) + "\n"


# --- gated rows ------------------------------------------------------------------------------

LLVM_NM = ROOT / ".build-tools" / "llvm-mingw" / "bin" / "llvm-nm"
BUILD_ARCH = {"build-i386": "i386", "build-amd64": "x86_64", "build-arm64": "arm64"}
# Copies of an image staged onto boot media, never the image the link wrote.
_STAGED = {"boot", "bootcd", "livecd", "reactos", "syswow64", "sysamd64"}

# Gated rows whose LINKED body does not do the job: exporting one would turn a program's
# working fallback (no export, so it takes its XP path) into a call that lies or fails.
# They stay gated until the body is real; `gated` lists them so they are not forgotten.
NOT_A_BODY = {
    "D3DKMTOpenAdapterFromHdc": "returns success without opening an adapter",
    "D3DKMTOpenAdapterFromDeviceName": "returns success without opening an adapter",
    "D3DKMTGetProcessSchedulingPriorityClass": "returns 1 and writes nothing",
    "D3DKMTSetProcessSchedulingPriorityClass": "returns 1, a success status, and does nothing",
    "D3DKMTSharedPrimaryLockNotification": "returns 1, a success status, and does nothing",
    "D3DKMTSharedPrimaryUnLockNotification": "returns 1, a success status, and does nothing",
    "RtlConvertLCIDToString": "returns STATUS_NOT_IMPLEMENTED",
    "RtlIsValidLocaleName": "answers TRUE for every name",
    "SHCreateItemInKnownFolder": "returns E_NOTIMPL",
    "AddClipboardFormatListener": "fails without setting a last error",
    "RemoveClipboardFormatListener": "fails without setting a last error",
    "ConvertTrusteeToSid": "the spec row's prototype is unknown",
}


def built_image(build: Path, module: str) -> Path | None:
    """The image the link wrote for `module` in a build dir (not a staged copy). The walk
    prunes cargo targets and the staged media trees before entering them: a build dir is
    rewritten under a running walk (tools/scan_unpruned_walk.py)."""
    staged = tuple(sorted(_STAGED))
    hits = sorted(
        parent / module
        for parent, _, files in source_walk(build, exclude_prefixes=staged)
        if module in files and not _STAGED & {p.lower() for p in parent.relative_to(build).parts}
    )
    return hits[0] if hits else None


def defined_symbols(image: Path) -> dict[str, set[int | None]]:
    """{undecorated name: the stdcall/fastcall argument sizes it is defined with} for every
    symbol the image defines (`_F@8` -> {"F": {8}}; an undecorated `F` adds None)."""
    if not LLVM_NM.is_file():
        raise SystemExit(f"win10_surface: llvm-nm not found at {LLVM_NM}")
    out = subprocess.run(
        [str(LLVM_NM), "--defined-only", str(image)], capture_output=True, text=True, check=True
    ).stdout
    names: dict[str, set[int | None]] = defaultdict(set)
    for ln in out.splitlines():
        parts = ln.split()
        if len(parts) >= 3:
            n = parts[-1]
            names[n].add(None)
            m = re.fullmatch(r"[_@]?(.+?)(?:@(\d+))?", n)
            if m:
                names[m.group(1)].add(int(m.group(2)) if m.group(2) else None)
    return names


_ARG_BYTES_I386 = {"int64": 8, "double": 8, "int128": 16}


def stdcall_bytes(args: str) -> int:
    """The i386 `@N` a spec argument list decorates to."""
    return sum(_ARG_BYTES_I386.get(a, 4) for a in args.strip("()").split())


@dataclass
class GatedRow:
    module: str
    spec: Path
    name: str
    row: SpecRow
    # arch -> "linked" | "imported:<dll>" | "absent" | "noimage", or for a forwarding row
    # "forward" (its target exports the name at this version) | "forward-dead"
    arches: dict[str, str]
    excluded: str | None = None


def survey_gated(builds: list[Path], ref: dict, version: int) -> list[GatedRow]:
    """Every row Windows exports (code, not data) that the spec gates out of `version`."""
    images: dict[tuple[str, str], tuple[dict[str, set[int | None]], dict[str, str]] | None] = {}

    def image_facts(
        module: str, build: Path
    ) -> tuple[dict[str, set[int | None]], dict[str, str]] | None:
        key = (module, str(build))
        if key not in images:
            img = built_image(build, module)
            if img is None:
                images[key] = None
            else:
                pe = read_pe_file(img)
                imported = {n: d.lower() for d, ns in pe.imports.items() for n in ns}
                images[key] = (defined_symbols(img), imported)
        return images[key]

    modules = module_specs()
    target_states: dict[tuple[str, str], dict[str, str]] = {}

    def forward_live(target: str, arch: str) -> bool:
        tmod, _, tname = target.partition(".")
        specs = modules.get(tmod.lower() + ".dll", [])
        if not specs:
            return False
        key = (str(specs[0]), arch)
        if key not in target_states:
            target_states[key] = spec_exports(specs[0], arch, version)
        return target_states[key].get(tname) == "ok"

    out: list[GatedRow] = []
    for module, spec, mref in specs_for(modules, ref):
        win = {**mref.get("x86", {}).get("exports", {}), **mref.get("x64", {}).get("exports", {})}
        text = strip_block(strip_block(spec.read_bytes().decode("latin-1")), FWD_BEGIN, FWD_END)
        rows = spec_rows(text)
        states = {a: spec_exports(spec, a, version) for a in ARCHES}
        for name, rs in sorted(rows.items()):
            if name not in win or win[name]["data"]:
                continue
            real = [
                r
                for r in rs
                if r.kind != "stub"
                and not _stub_flag(r.opts)
                and any(o.startswith("-version=") for o in r.opts)
            ]
            if not real:
                continue
            row = real[0]
            arches: dict[str, str] = {}
            for build in builds:
                arch = BUILD_ARCH.get(build.name)
                if arch is None or not states[arch].get(name, "").startswith("exported only"):
                    continue
                target = row.target or name
                facts = image_facts(module, build)
                if "." in target:
                    arches[arch] = "forward" if forward_live(target, arch) else "forward-dead"
                elif facts is None:
                    arches[arch] = "noimage"
                elif target in facts[1]:
                    arches[arch] = f"imported:{facts[1][target]}"
                elif target in facts[0]:
                    # The body under this name may be ANOTHER function: ReactOS kept
                    # shell32's 4-argument internal SHBindToObject where Windows exports
                    # a 5-argument one. i386's stdcall decoration is the size on record.
                    sizes = {n for n in facts[0][target] if n is not None}
                    want = stdcall_bytes(row.args)
                    if arch == "i386" and row.kind == "stdcall" and sizes and want not in sizes:
                        arches[arch] = f"signature:@{'/'.join(map(str, sorted(sizes)))}!=@{want}"
                    else:
                        arches[arch] = "linked"
                else:
                    arches[arch] = "absent"
            if arches:
                out.append(GatedRow(module, spec, name, row, arches, NOT_A_BODY.get(name)))
    return out


_VERSION_PLUS = re.compile(r"\s-version=0x([0-9A-Fa-f]+)\+(?=\s)")


def ungate_line(line: str) -> str | None:
    """`line` with its `-version=0xNNN+` removed, or None when the row is not that shape."""
    body, sep, comment = line.partition("#")
    if len(_VERSION_PLUS.findall(body)) != 1:
        return None
    return _VERSION_PLUS.sub("", body, count=1) + sep + comment


def apply_ungate(rows: list[GatedRow]) -> tuple[int, list[str]]:
    """Ungate every row whose body is linked, or whose forward target is exported, on ALL
    the arches it is gated on. A body another DLL provides is a forward instead
    (`forwards`, PROVIDERS). Returns (rows changed, rows left for a human: an unusual
    -version shape)."""
    by_spec: dict[Path, list[GatedRow]] = defaultdict(list)
    for g in rows:
        if not g.excluded and set(g.arches.values()) in ({"linked"}, {"forward"}):
            by_spec[g.spec].append(g)
    changed, manual = 0, []
    for spec, gs in by_spec.items():
        lines = spec.read_bytes().decode("latin-1").splitlines(keepends=True)
        for g in gs:
            pat = re.compile(
                rf"^\s*(@|\d+)\s+{re.escape(g.row.kind)}((\s+-\S+)*)\s+{re.escape(g.name)}(\(|\s|$)"
            )
            idx = [
                i
                for i, ln in enumerate(lines)
                if (m := pat.match(ln)) and "-version=" in m.group(2)
            ]
            new = ungate_line(lines[idx[0]]) if len(idx) == 1 else None
            if new is None:
                manual.append(f"{g.module}!{g.name}: {' '.join(g.row.opts)}")
                continue
            lines[idx[0]] = new
            changed += 1
        spec.write_bytes("".join(lines).encode("latin-1"))
    return changed, manual


def cmd_gated(args: argparse.Namespace) -> int:
    ref = load_reference(_ref_path(args))
    builds = [Path(b).resolve() for b in args.builds]
    rows = survey_gated(builds, ref, export_version())
    tally: Counter[str] = Counter()
    for g in rows:
        kinds = set(g.arches.values())
        verdict = (
            f"excluded ({g.excluded})"
            if g.excluded
            else kinds.pop().split(":")[0]
            if len(kinds) == 1
            else "mixed " + ",".join(f"{a}={k}" for a, k in sorted(g.arches.items()))
        )
        tally[verdict.split(" (")[0].split()[0]] += 1
        if args.verbose or not verdict.startswith("linked"):
            print(f"  {verdict:40s} {g.module}!{g.name}")
    print("gated rows Windows exports: " + ", ".join(f"{k} {v}" for k, v in tally.most_common()))
    if args.apply:
        changed, manual = apply_ungate(rows)
        print(f"ungated {changed} rows; left for a human: {len(manual)}")
        for m in manual:
            print(f"    {m}")
    return 0


# --- verify ----------------------------------------------------------------------------------


def _spec2def(binary: Path, spec_text: str, module: str, arch: str, version: str) -> bytes:
    with tempfile.TemporaryDirectory() as td:
        t = Path(td)
        (t / "in.spec").write_text(spec_text)
        proc = subprocess.run(
            [
                str(binary),
                f"-n={module}",
                f"-a={arch}",
                f"-d={t / 'out.def'}",
                f"-s={t / 'out.rs'}",
                f"--version={version}",
                str(t / "in.spec"),
            ],
            capture_output=True,
            text=True,
        )
        if proc.returncode != 0:
            raise RuntimeError(proc.stderr.strip() or f"spec2def rc={proc.returncode}")
        return (t / "out.def").read_bytes() + b"\0" + (t / "out.rs").read_bytes()


def verify_spec(binary: Path, module: str, spec: Path, default: str) -> list[str]:
    text = spec.read_text(errors="replace")
    if BEGIN not in text:
        return []
    problems = []
    for arch in ("i386", "x86_64", "arm64"):
        try:
            if _spec2def(binary, text, module, arch, default) != _spec2def(
                binary, strip_block(text), module, arch, default
            ):
                problems.append(f"{spec}: {arch} output at {default} CHANGES (block not dormant)")
            _spec2def(binary, text, module, arch, TIER)
        except RuntimeError as e:
            problems.append(f"{spec}: {arch}: spec2def failed: {e}")
    return problems


# --- gap -------------------------------------------------------------------------------------


IMAGE_SUFFIXES = (".exe", ".dll", ".sys")
# Shipped by an installer next to (or into System32 for) the programs that need them, not by
# Windows: the VC++ and DirectX redistributables. Reported apart, because they are a
# different job — Wine has most of them, and they are not OS surface.
REDIST = re.compile(
    r"^(vcruntime|msvcp\d|msvcr\d|concrt|vcomp|vccorlib|mfc|d3dx|xinput1_[0-3]|xaudio2_[0-7])"
)


def iter_images(roots: Iterable[Path]) -> Iterator[tuple[str, Path]]:
    """(app, image) for every .exe/.dll/.sys under each root; app = first path component.

    os.walk, not Path.rglob + is_file: over /mnt/c (9p) every stat is an RPC, and the walk
    gets each entry's type from the directory listing for free.
    """
    for root in roots:
        for dirpath, _, files in os.walk(root):
            for f in files:
                if f.lower().endswith(IMAGE_SUFFIXES):
                    p = Path(dirpath, f)
                    rel = p.relative_to(root).parts
                    yield (f"{root.name}/{rel[0]}" if len(rel) > 1 else root.name), p


def _classify(spec: Path, arch: str, version: int) -> dict[str, str]:
    """{name: why our default build does not export it}, for the HAND-WRITTEN spec.

    gated        a real line exists, behind a newer -version (the work may already be done)
    stub         declared, but only as a stub
    arch         declared for other arches only
    surface-only only in the generated win10-surface block: nobody has touched it yet
    """
    with tempfile.NamedTemporaryFile("w", suffix=".spec", delete=False) as fh:
        fh.write(strip_block(spec.read_bytes().decode("latin-1")))
        tmp = Path(fh.name)
    try:
        states = spec_exports(tmp, arch, version)
    finally:
        tmp.unlink()
    out: dict[str, str] = {}
    for name, st in states.items():
        if st == "ok":
            out[name] = "ok"
        elif st == "stub":
            out[name] = "stub"
        elif st.startswith("not exported on"):
            out[name] = "arch"
        else:
            out[name] = "gated"
    return out


def gap(
    corpus: list[Path],
    ref: dict,
    schema: dict[str, str],
    root: Path = ROOT,
    ros: dict[str, str] | None = None,
) -> dict:
    """Rank every system import the corpus makes that our default build does not export."""
    version = export_version(root)
    modules = module_specs(root)
    win = Path(ref["windows"])
    images = list(iter_images(corpus))
    private: dict[str, set[str]] = defaultdict(set)  # app -> dll names it ships itself
    for app, p in images:
        private[app].add(p.name.lower())

    is_system: dict[str, bool] = {}

    def system(dll: str) -> bool:
        if dll not in is_system:
            is_system[dll] = any(f is not None for f in _windows_files(win, dll))
        return is_system[dll]

    need: dict[tuple[str, str], set[str]] = defaultdict(set)  # (dll, fn) -> apps
    why: dict[tuple[str, str], str] = {}
    missing_module: dict[str, set[str]] = defaultdict(set)
    redist: dict[str, set[str]] = defaultdict(set)
    apisets: dict[str, set[str]] = defaultdict(set)
    unmapped: dict[str, set[str]] = defaultdict(set)  # api sets our loader cannot resolve
    # (dll, fn) -> apps, for imports Windows' own host does not export either: a delay-load
    # or a binary that never loads on Windows. Reported apart, never as our gap.
    not_windows: dict[tuple[str, str], set[str]] = defaultdict(set)

    def windows_names(module: str) -> set[str] | None:
        entry = ref["modules"].get(module)
        if entry is None:
            return None
        return {n for t in entry.values() for n in t["exports"]}

    win_names: dict[str, set[str] | None] = {}
    if ros is None:
        ros = read_ros_apisets(ROS_APISETS.read_text()) if ROS_APISETS.is_file() else {}
    unparsed = 0
    exports_cache: dict[tuple[Path, str], dict[str, str]] = {}
    for app, p in images:
        try:
            pe = read_pe_file(p)
        except (PEError, OSError):
            unparsed += 1
            continue
        arch = pe.machine if pe.machine in ("i386", "x86_64", "arm64") else "x86_64"
        for dll, names in pe.imports.items():
            if dll in private[app]:
                continue
            if REDIST.match(dll):
                redist[dll].add(app)
                continue
            win_host = resolve_apiset(dll, schema) or dll
            win_host = win_host if win_host.endswith(".dll") else win_host + ".dll"
            if win_host not in win_names:
                win_names[win_host] = windows_names(win_host)
            on_windows = win_names[win_host]
            if resolve_apiset(dll, schema) is not None:
                apisets[dll].add(app)
                host = ros_apiset_host(dll, ros)
                if not host:
                    # Our loader has no row for this name: the import fails to load here
                    # whatever Windows maps it to.
                    unmapped[dll].add(app)
                    continue
                dll = host if host.endswith(".dll") else host + ".dll"
            if not system(dll):
                continue  # a third-party DLL the app expects from elsewhere: not ours
            specs = modules.get(dll)
            if not specs:
                missing_module[dll].add(app)
                continue
            key = (specs[0], arch)
            if key not in exports_cache:
                exports_cache[key] = _classify(specs[0], arch, version)
            have = exports_cache[key]
            for n in names:
                if n.startswith("#"):
                    continue
                if on_windows is not None and n not in on_windows:
                    if have.get(n) != "ok":
                        not_windows[(dll, n)].add(app)
                    continue
                state = have.get(n, "surface-only")
                if state != "ok":
                    need[(dll, n)].add(app)
                    why[(dll, n)] = state

    def ranked(d: dict[str, set[str]]) -> list[dict]:
        return sorted(
            ({"dll": k, "apps": sorted(a)} for k, a in d.items()),
            key=lambda r: (-len(r["apps"]), r["dll"]),
        )

    return {
        "images": len(images),
        "unparsed": unparsed,
        "apps": len(private),
        "missing_functions": sorted(
            (
                {"dll": d, "fn": f, "why": why[(d, f)], "apps": sorted(a)}
                for (d, f), a in need.items()
            ),
            key=lambda r: (-len(r["apps"]), r["dll"], r["fn"]),
        ),
        "not_on_windows": sorted(
            ({"dll": d, "fn": f, "apps": sorted(a)} for (d, f), a in not_windows.items()),
            key=lambda r: (-len(r["apps"]), r["dll"], r["fn"]),
        ),
        "missing_modules": ranked(missing_module),
        "redistributables": ranked(redist),
        "apiset_imports": ranked(apisets),
        "apiset_unmapped": [
            {**r, "windows_host": resolve_apiset(r["dll"], schema) or ""} for r in ranked(unmapped)
        ],
    }


# --- CLI -------------------------------------------------------------------------------------


def _ref_path(args: argparse.Namespace) -> Path:
    return Path(args.reference) if args.reference else CACHE / "reference.json"


def cmd_reference(args: argparse.Namespace) -> int:
    win = Path(args.windows)
    ref = build_reference(win)
    out = _ref_path(args)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(ref, indent=1, sort_keys=True))
    n = sum(len(m.get("x64", {}).get("exports", {})) for m in ref["modules"].values())
    print(
        f"reference: Windows {ref['build']}, {len(ref['modules'])} modules we build, "
        f"{n} x64 exports -> {out}"
    )
    return 0


def cmd_stubs(args: argparse.Namespace) -> int:
    ref = load_reference(_ref_path(args))
    modules = module_specs()
    totals = Counter()
    rows = []
    for module, spec, mref in specs_for(modules, ref):
        plan = plan_module(module, spec, mref)
        block = render_block(plan, ref["build"]) if plan.lines or plan.data else None
        changed = apply_block(spec, block, force=args.force) if args.apply else False
        totals["stubs"] += len(plan.lines)
        totals["data"] += len(plan.data)
        totals["skipped"] += len(plan.skipped)
        totals["specs"] += bool(block)
        totals["changed"] += changed
        rows.append((len(plan.lines), module, spec.relative_to(ROOT), len(plan.data)))
    for n, module, spec, nd in sorted(rows, reverse=True)[: args.top]:
        if n or nd:
            print(f"  {n:6d} stubs  {nd:4d} data  {module:24s} {spec}")
    verb = "wrote" if args.apply else "would write (dry run; --apply to write)"
    print(
        f"{verb}: {totals['stubs']} stubs + {totals['data']} data exports across "
        f"{totals['specs']} specs ({totals['changed']} changed); {totals['skipped']} names "
        f"spec2def cannot tokenize skipped; reference Windows {ref['build']}"
    )
    return 0


def cmd_forwards(args: argparse.Namespace) -> int:
    if args.check:
        return cmd_forwards_check(args.check)
    ref = load_reference(_ref_path(args))
    modules = module_specs()
    version = export_version()
    total = 0
    for module, names in PROVIDERS.items():
        spec = modules[module][0]
        providers = [(p, modules[f"{p}.dll"][0]) for p in names]
        lines = plan_forwards(module, spec, providers, ref["modules"][module], version)
        block = render_forwards(module, names, lines) if lines else None
        changed = (
            apply_block(spec, block, FWD_BEGIN, FWD_END, force=args.force) if args.apply else False
        )
        total += len(lines)
        print(
            f"  {len(lines):4d} forwards  {module:14s} -> {'/'.join(names):18s} "
            f"{spec.relative_to(ROOT)}" + ("  (changed)" if changed else "")
        )
        for ln in lines[: args.show]:
            print(f"        {ln}")
    verb = "wrote" if args.apply else "would write (dry run; --apply to write)"
    print(f"{verb}: {total} forwards. Then re-run `stubs --apply` so the dormant block drops them.")
    return 0


def check_forward_chains(dlls: dict[str, PE], start: str) -> list[str]:
    """Follow every forward `start` exports (one module's built PE) to a real body.

    A forward can only be judged on the BUILT images: the specs say what each module
    claims, not what the linker bound. Returns problems: a chain that comes back to an
    export it already passed through (the loader would loop), or one ending in a module or
    name the build does not have. A chain that leaves and re-enters a module under a
    DIFFERENT name is fine (kernel32_vista's GetSystemTimePreciseAsFileTime is
    kernel32.GetSystemTimeAsFileTime).
    """
    problems = []
    for name, exp in sorted(dlls[start].exports.items()):
        if not exp.forward:
            continue
        seen = {(start, name)}
        cur = exp.forward
        while True:
            mod, _, fn = cur.partition(".")
            key = (mod.lower() + ".dll", fn)
            pe = dlls.get(key[0])
            if pe is None or fn not in pe.exports:
                problems.append(f"{start}!{name}: forward chain ends at missing {cur}")
                break
            if key in seen:
                problems.append(f"{start}!{name}: forward CYCLE through {cur}")
                break
            seen.add(key)
            nxt = pe.exports[fn].forward
            if not nxt:
                break
            cur = nxt
    return problems


def cmd_forwards_check(builds: list[str]) -> int:
    bad = 0
    names = {m for mod, ps in PROVIDERS.items() for m in (mod, *(p + ".dll" for p in ps))}
    for build in builds:
        found: dict[str, PE] = {}
        for n in names:
            hits = [p for p in Path(build).rglob(n) if "CMakeFiles" not in p.parts]
            if not hits:
                sys.exit(f"win10_surface: no {n} under {build}; fix: ./vibeos-cli build")
            found[n] = read_pe_file(hits[0])
        problems = [p for m in PROVIDERS for p in check_forward_chains(found, m)]
        fwd = sum(1 for m in PROVIDERS for e in found[m].exports.values() if e.forward)
        for p in problems:
            print(f"  {build}: {p}")
        print(f"check {build}: {fwd} forwards followed to a body, {len(problems)} problems")
        bad += len(problems)
    return 1 if bad else 0


def cmd_verify(args: argparse.Namespace) -> int:
    binary = Path(args.spec2def)
    if not binary.is_file():
        sys.exit(f"win10_surface: no spec2def at {binary}\n  fix: ./vibeos-cli build --arch amd64")
    default = f"{export_version():#x}"
    problems: list[str] = []
    checked = 0
    seen: set[Path] = set()
    for module, specs in sorted(module_specs().items()):
        for spec in specs:
            if spec not in seen and BEGIN in spec.read_text(errors="replace"):
                seen.add(spec)
                checked += 1
                problems += verify_spec(binary, module, spec, default)
    for p in problems:
        print(p)
    print(
        f"verify: {checked} specs with a generated block, {len(problems)} problems "
        f"(dormant at {default}, generates at {TIER})"
    )
    return 1 if problems else 0


def cmd_gap(args: argparse.Namespace) -> int:
    ref = load_reference(_ref_path(args))
    win = Path(ref["windows"])
    schema = read_apiset_schema((win / "System32" / "apisetschema.dll").read_bytes())
    res = gap([Path(c) for c in args.corpus], ref, schema)
    if args.json:
        Path(args.json).write_text(json.dumps(res, indent=1))
    print(
        f"gap: {res['images']} images across {res['apps']} apps "
        f"({res['unparsed']} unparsed), reference Windows {ref['build']}"
    )
    fns = res["missing_functions"]
    by_why = Counter(r["why"] for r in fns)
    print(
        f"functions our default build does not export: {len(fns)} "
        f"({', '.join(f'{k} {v}' for k, v in by_why.most_common())}), by apps importing them:"
    )
    for r in fns[: args.top]:
        print(f"  {len(r['apps']):4d}  {r['why']:12s}  {r['dll']}!{r['fn']}")
    nw = res["not_on_windows"]
    print(
        f"imports Windows {ref['build']} does not export either (delay-loads or dead code; "
        f"not our gap): {len(nw)}"
    )
    for r in nw[: args.top]:
        print(f"  {len(r['apps']):4d}  {r['dll']}!{r['fn']}  ({', '.join(r['apps'])})")
    for title, key in (
        ("Windows modules we do not build", "missing_modules"),
        ("redistributables (VC++/DirectX runtimes, not OS surface)", "redistributables"),
        ("api-set names imported (our loader redirects them)", "apiset_imports"),
        ("api-set names our loader cannot resolve (no row in apisets.table.c)", "apiset_unmapped"),
    ):
        print(f"{title}: {len(res[key])}")
        for r in res[key][: args.top]:
            print(f"  {len(r['apps']):4d}  {r['dll']}")
    return 0


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--reference", help=f"reference JSON (default {CACHE}/reference.json)")
    sub = ap.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("reference", help="read a Windows install's export tables")
    r.add_argument("--windows", default="/mnt/c/Windows")
    r.set_defaults(fn=cmd_reference)
    s = sub.add_parser("stubs", help="line out missing exports as dormant spec stubs")
    s.add_argument("--apply", action="store_true", help="write the specs (default: dry run)")
    s.add_argument("--force", action="store_true", help="discard hand edits inside the block")
    s.add_argument("--top", type=int, default=40)
    s.set_defaults(fn=cmd_stubs)
    f = sub.add_parser("forwards", help="forward gated exports to the *_vista DLLs")
    f.add_argument("--apply", action="store_true", help="write the specs (default: dry run)")
    f.add_argument("--force", action="store_true", help="discard hand edits inside the block")
    f.add_argument("--show", type=int, default=0, help="print the first N lines per module")
    f.add_argument(
        "--check",
        nargs="+",
        metavar="BUILD_DIR",
        help="instead: follow every forward in the BUILT DLLs to a real body",
    )
    f.set_defaults(fn=cmd_forwards)
    v = sub.add_parser("verify", help="prove every generated block is dormant and generates")
    v.add_argument("--spec2def", default=str(ROOT / "build-amd64/host-tools/bin/spec2def"))
    v.set_defaults(fn=cmd_verify)
    g = sub.add_parser("gap", help="rank what a corpus of programs imports that we lack")
    g.add_argument("corpus", nargs="+", help="directories of installed programs")
    g.add_argument("--json", help="write the full result here")
    g.add_argument("--top", type=int, default=40)
    g.set_defaults(fn=cmd_gap)
    gt = sub.add_parser("gated", help="gated rows whose body is already linked: export them")
    gt.add_argument("builds", nargs="+", help="build dirs (build-i386 build-amd64 build-arm64)")
    gt.add_argument("--apply", action="store_true", help="ungate linked rows (default: report)")
    gt.add_argument("-v", "--verbose", action="store_true", help="list the linked rows too")
    gt.set_defaults(fn=cmd_gated)
    args = ap.parse_args(argv)
    try:
        return args.fn(args)
    except HandEditedBlock as e:
        print(f"error: {e}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
