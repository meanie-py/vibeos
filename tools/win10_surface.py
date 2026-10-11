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

TIER = "0xA00"
BEGIN = "# BEGIN win10-surface stubs"
END = "# END win10-surface stubs"
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


def strip_block(text: str) -> str:
    return re.sub(rf"\n?{re.escape(BEGIN)}.*?{re.escape(END)}\n?", "\n", text, flags=re.S)


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


def apply_block(spec: Path, block: str | None) -> bool:
    """Write (or remove, when block is None) the generated block. True if the file changed.

    Byte-preserving outside the block: latin-1 round-trips any byte, and the hand-written
    part is never re-flowed. A first version normalized trailing blank lines and a missing
    final newline, which put a spurious hunk in 231 specs that the block never touched.
    """
    old = spec.read_bytes().decode("latin-1")
    base = re.sub(rf"\n?{re.escape(BEGIN)}.*?{re.escape(END)}\n", "", old, flags=re.S)
    if block is None:
        new = base
    else:
        new = base + ("\n" if base and not base.endswith("\n") else "") + "\n" + block
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


def gap(corpus: list[Path], ref: dict, schema: dict[str, str], root: Path = ROOT) -> dict:
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
            host = resolve_apiset(dll, schema)
            if host is not None:
                apisets[dll].add(app)
                if not host:
                    continue
                dll = host
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
        "missing_modules": ranked(missing_module),
        "redistributables": ranked(redist),
        "apiset_imports": ranked(apisets),
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
        changed = apply_block(spec, block) if args.apply else False
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
    for title, key in (
        ("Windows modules we do not build", "missing_modules"),
        ("redistributables (VC++/DirectX runtimes, not OS surface)", "redistributables"),
        ("api-set names imported (we ship no api-set DLLs)", "apiset_imports"),
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
    s.add_argument("--top", type=int, default=40)
    s.set_defaults(fn=cmd_stubs)
    v = sub.add_parser("verify", help="prove every generated block is dormant and generates")
    v.add_argument("--spec2def", default=str(ROOT / "build-amd64/host-tools/bin/spec2def"))
    v.set_defaults(fn=cmd_verify)
    g = sub.add_parser("gap", help="rank what a corpus of programs imports that we lack")
    g.add_argument("corpus", nargs="+", help="directories of installed programs")
    g.add_argument("--json", help="write the full result here")
    g.add_argument("--top", type=int, default=40)
    g.set_defaults(fn=cmd_gap)
    args = ap.parse_args(argv)
    return args.fn(args)


if __name__ == "__main__":
    sys.exit(main())
