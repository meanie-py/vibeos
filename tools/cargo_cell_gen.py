#!/usr/bin/env python3
"""Move a fully-Rust module's link from cmake to cargo: generate its `<m>-cargo` cell.

A module whose C is all flipped still has cmake link it: `add_executable`/`add_library`
around a placeholder `rust_cohab_stub.c`, `set_module_type`, `add_importlibs`. This tool
reads what that link ACTUALLY does, from each `build-<arch>/build.ninja` (entry point,
subsystem, image base, the libraries, the `.def`, the resources), and writes the
reactos-link cell that does the same thing:

  sdk/lib/rust/<m>-cargo/{Cargo.toml, build.rs, src/main.rs}
  + the workspace member, + the module's CMakeLists rewritten to `add_cargo_module`.

    python3 tools/cargo_cell_gen.py winmine            # dry run: show what it would write
    python3 tools/cargo_cell_gen.py winmine --apply
    python3 tools/cargo_cell_gen.py --survey           # every fully-Rust cmake-linked module
    python3 tools/cargo_cell_gen.py --save-baseline DIR winmine   # before --apply
    python3 tools/cargo_cell_gen.py --check DIR winmine           # after the rebuild

It refuses, rather than guesses, any shape it does not model (delay imports, a C TU
that is not a placeholder, a statement on the target it does not know). The result is
checked with `tools/pe_equiv.py` against the cmake-built PE, on all three arches, before
the move lands. Plan: docs/engineering/cargo-drive-plan.md.
"""

from __future__ import annotations

import argparse
import functools
import json
import os
import re
import subprocess
import sys
import time
from dataclasses import dataclass, field
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO / "tools"))
from cohab_census import SRC_IN_COMMENT  # noqa: E402

RUST = REPO / "sdk/lib/rust"
ARCHES = ("i386", "amd64", "arm64")
TRIPLE = {"i386": "x86", "amd64": "x86_64", "arm64": "aarch64"}
TARGET = {
    "i386": "i686-pc-windows-gnu",
    "amd64": "x86_64-pc-windows-gnu",
    "arm64": "aarch64-pc-windows-gnullvm",
}
MSVCRTEX = {"wexe": "ExeWide", "exe": "ExeAnsi", "dll": "Dll"}
# What panic_common's user handler imports (its report, stack walk and exit).
PANIC_COMMON_IMPORTS = {
    "ntdll.dll!DbgPrint",
    "ntdll.dll!NtTerminateProcess",
    "ntdll.dll!RtlWalkFrameChain",
    "msvcrt.dll!memcmp",
    "msvcrt.dll!memset",
    "msvcrt.dll!memcpy",
    "msvcrt.dll!memmove",
    # A module that imports nothing from msvcrt (wlnotify) gets them from ntdll instead.
    "ntdll.dll!memcmp",
    "ntdll.dll!memset",
    "ntdll.dll!memcpy",
    "ntdll.dll!memmove",
}
# Per-module removals that were checked by hand and are dead code: the import's only user
# is a function nothing can call. Each entry names its evidence; never add one unread.
DEAD_IMPORTS = {
    # Only LoadAndFormatString (the varargs trampoline) reaches FormatMessageW, and nothing
    # calls it: its C callers are in the uncompiled misc.c and an exe exports nothing.
    # The engine archives the shim objects, so whether lld keeps it varies by arch.
    "sndvol32": {"kernel32.dll!FormatMessageW"},
}
# Modules whose cmake link bound an EXPORT to an import thunk of the same name, which the
# cell fixes by linking the body first. user32_vista (2026-10-17): its `dpi.c` was an object
# in the C link, so it won; the cohab flip moved it into an archive that loses to user32's
# import library, so user32_vista.dll exported user32!GetDpiFor* while user32.dll (export
# version 0x502) forwards those names to user32_vista: a forwarding cycle on every arch, and
# dpi.rs was never linked. The cell's only accepted difference is losing those imports.
REBOUND_EXPORTS = {
    "user32_vista": {"GetDpiForSystem", "GetDpiForWindow"},
}
# The engine builds a body with `-C force-frame-pointers=yes` (stack walks need them) where
# cmake's rust_staticlib_variant did not; a frame pointer can push a big frame over the
# probe threshold. ntdll exports __chkstk on amd64/arm64 and kernel32 forwards it there.
STACK_PROBE_IMPORTS = {"kernel32.dll!__chkstk", "ntdll.dll!__chkstk"}
# i386 only: the builtin i686-pc-windows-gnu triple links rsbegin.o/rsend.o, whose one
# `.ctors`/`.dtors` entry each (EH-frame registration, a no-op here: see reactos-link)
# lld's MinGW writer puts in sections of their own.
# A kernel-mode cell's sections are also IMAGE_SCN_MEM_NOT_PAGED (native-pefixup).
I386_RSBEGIN_SECTIONS = {
    f"sections: + {name} {flags:#x}"
    for name in (".ctors", ".dtors")
    for flags in (0x40000040, 0x48000040)
}
# rsbegin.o's own `.data` (its frame-object pointers, 0x28 bytes): a section of its own
# in a module that had none (hidusb, msfs, usbccgp, watchdog).
I386_RSBEGIN_DATA = 0x28
# How much more the loader may map for a cargo-linked module than for cmake's: the cell's
# own handler, startup and section padding. All of core is ~200 KB on amd64.
SIZE_SLACK = 16 * 1024
# Statements on the target that the cargo cell replaces outright.
REPLACED = (
    "add_executable",
    "add_library",
    "set_module_type",
    "add_importlibs",
    "add_delay_importlibs",
    "target_link_libraries",
    "add_pch",
    "add_cd_file",
    "add_dependencies",
)

# Statements that set how the target's sources compile. The rewrite re-aims each at the
# OBJECT libraries that now hold those compiles (`<m>_res`, `<m>_shims`), so the rc and C
# the cell replays are compiled exactly as cmake compiled them for the target.
TRANSFERRED = (
    "target_compile_definitions",
    "target_include_directories",
    "target_compile_options",
    "set_wine_module_fixme",
    "set_wine_module",
)
# Properties that only name the output file, which the cell already takes from the link
# edge (chcp.com, help.exe for cmd_help), and two the cell has no use for: spoolsv's
# ENABLE_EXPORTS (its exports come from the .def; its import lib from spec2def
# ADD_IMPORTLIB, which stays, and the link edge writes no --out-implib) and DEFINE_SYMBOL
# (a define for the target's own compiles, of which there are none left).
NAME_PROPERTIES = {"SUFFIX", "OUTPUT_NAME", "PREFIX", "ENABLE_EXPORTS", "DEFINE_SYMBOL"}
# set_module_type types that link as reactos-link's KernelDll, by add_cargo_module KIND.
KERNEL_KINDS = ("kernelmodedriver", "kerneldll")
# Statements naming the target that neither link nor compile it. add_driver_inf's first
# argument only names the `<m>_inf_files` target it makes for the INF.
KEPT = ("project", "add_driver_inf")

# cmake's build configurations, as they suffix a ninja link rule name.
CONFIGS = "(?:Debug|Release|RelWithDebInfo|MinSizeRel)"


class GenError(Exception):
    pass


@dataclass
class Link:
    arch: str
    output: str  # build-dir relative
    kind: str  # C_EXECUTABLE / C_MODULE_LIBRARY / C_SHARED_LIBRARY
    entry: str
    subsystem: str | None
    image_base: int | None
    stack: int | None
    inputs: list[str]
    libs: list[str]


@dataclass
class Cell:
    target: str
    cmake_dir: Path
    output_name: str
    module_kind: str  # Win32Exe | Win32Dll
    entries: dict[str, str]
    subsystem: str | None
    image_base: dict[str, int | None]
    stack: int | None
    body_crate: str
    body_features: list[str]
    body_defaults: bool
    import_libs: list[tuple[str, str, str]]  # (spec_rel, dll_name, basename)
    msvcrtex: str | None
    static_libs: dict[str, list[str]]  # per arch, build-dir relative .a, cmake-built for now
    spec: tuple[str, str] | None  # (spec_rel, dll_name) for exports
    spec_stubs: bool
    c_shims: list[str]
    rc: str | None  # rc source, repo relative
    rc_defines: list[str]
    destinations: list[str]  # CD directories, as the CMakeLists spells them
    name_on_cd: str | None
    # Per arch: did cmake's link carry the body's panic handler? Only then does the cell
    # take panic_common (and with it all of core); otherwise panic-lean.
    panics: dict[str, bool] = field(default_factory=lambda: dict.fromkeys(ARCHES, True))
    # The rust_staticlib_variant that builds the body; "" means `<target>_rust`.
    body_var: str = ""
    # Per arch, the static libs cmake linked AFTER the body (they lose a duplicate
    # definition to it); the rest of `static_libs` precede it.
    late_libs: dict[str, list[str]] = field(default_factory=dict)
    # add_delay_importlibs: same shape as import_libs, linked through gen-delaylib.py.
    delay_import_libs: list[tuple[str, str, str]] = field(default_factory=list)
    # The c_shims cmake generates into the build dir (widl's RPC stubs, flex output).
    generated: list[str] = field(default_factory=list)
    # The c_shims from OUTSIDE the module's directory (cscript compiles ../wscript's).
    foreign: list[str] = field(default_factory=list)
    # (basename, delayed) in cmake's link order, when there are delay imports: an
    # `__imp_` symbol two of the libs define binds to the first (Linker::delay_import_lib).
    import_order: list[tuple[str, bool]] = field(default_factory=list)
    notes: list[str] = field(default_factory=list)
    # Per arch, the RUSTFLAGS cmake built the body with (`ENV "RUSTFLAGS=..."` on its
    # rust_staticlib_variant): calc and friends pass floats by value, so i386 needs SSE.
    body_rustflags: dict[str, str] = field(default_factory=dict)
    # That `set(<VAR>) if(ARCH ...) set(<VAR> ENV ...) endif()` block, which the rewrite
    # drops with the variant.
    body_env_block: str = ""
    # Per arch, when the arches ship to different CD dirs (winprint's prtprocs/<arch>).
    arch_destinations: dict[str, list[str]] = field(default_factory=dict)
    # Per arch, cmake's order of "body", "import:<base>" and static libs; empty for
    # cells generated before Linker::link_order (they keep static_libs/late_libs).
    link_order: dict[str, list[str]] = field(default_factory=dict)
    # add_cd_file(... NO_CAB FOR all): not in reactos.cab, so setup does not install it.
    no_cab: bool = False
    # KernelDll: set_module_type's type (kernelmodedriver, kerneldll), which is
    # add_cargo_module's KIND and so its post-link.
    kernel_kind: str = ""
    # KernelDll: the driver's own body custom command (_custom_body), which the rewrite drops.
    custom_body: dict = field(default_factory=dict)

    @property
    def crate(self) -> str:
        """The cell's cargo package. A package name cannot start with a digit, so a
        target that does (3dtext) takes its body crate's stem (text3d-cargo)."""
        if self.target[:1].isdigit():
            return self.body_crate.removesuffix("-rust") + "-cargo"
        return f"{self.target}-cargo"

    # Per arch, the C/asm compiled into the module on that arch only (on top of c_shims).
    arch_shims: dict[str, list[str]] = field(default_factory=dict)
    # Per arch, when the arches link different subsystem versions (welcome: 4.00 on i386).
    subsystems: dict[str, str] = field(default_factory=dict)
    # add_cd_file argument lists (TARGET dropped) for the boot-CD-only placements.
    extra_cd: list[str] = field(default_factory=list)
    # Per arch, the module's part of cmake's windres command (Linker::rc).
    rc_flags: dict = field(default_factory=dict)


# ---------------------------------------------------------------- build graph


@functools.cache
def _read(path: Path) -> str:
    return path.read_text(errors="replace")


@functools.cache
def _ccdb(path: Path) -> list:
    return json.loads(path.read_text())


def link_edge(arch: str, target: str, repo: Path = REPO) -> Link:
    ninja = _read(repo / f"build-{arch}/build.ninja")
    m = re.search(
        # The config suffix anchors the name: `cmd_\S*` also matched cmd_help's edge.
        rf"^build (\S+): (C_\w+?)_LINKER__{re.escape(target)}_{CONFIGS}( .*)?\n((?:  .*\n)+)",
        ninja,
        re.M,
    )
    if not m:
        raise GenError(f"{target}: no link edge in build-{arch}/build.ninja (a static lib?)")
    output, kind, ins, var_block = m.group(1), m.group(2), m.group(3) or "", m.group(4)
    flags = re.search(r"^  LINK_FLAGS = (.*)$", var_block, re.M)
    flags_s = flags.group(1) if flags else ""
    libs_m = re.search(r"^  LINK_LIBRARIES = (.*)$", var_block, re.M)
    # `set_module_type(... ENTRYPOINT 0)` (gdi32_vista) links `-Wl,-e,__ImageBase`.
    entry = re.search(r"-entry,(\S+)", flags_s) or re.search(r"-Wl,-e,(\S+)", flags_s)
    # The last one wins in lld: welcome's i386 target_link_options lowers it to 4.00.
    sub = (re.findall(r"--subsystem,([\w:.]+)", flags_s) or [None])[-1]
    base = re.search(r"--image-base,(0x[0-9a-fA-F]+)", flags_s)
    stack = re.search(r"--stack,(\d+)", flags_s)
    if "--delayload" in flags_s:
        raise GenError(f"{target}: lld --delayload is not modelled (add_delay_importlibs is)")
    defs = re.findall(r"(\S+\.def)\b", m.group(0).split("\n", 1)[0])
    return Link(
        arch,
        output,
        kind,
        entry.group(1) if entry else "",
        sub,
        int(base.group(1), 16) if base else None,
        int(stack.group(1)) if stack else None,
        ins.split("|")[0].split() + [d for d in defs if d not in ins.split("|")[0].split()],
        libs_m.group(1).split() if libs_m else [],
    )


@functools.cache
def spec_index(repo: Path = REPO) -> dict[str, tuple[str, str] | None]:
    """import-lib basename -> (spec, dll file name); None when it cannot be told.

    The build's own `spec2def --implib` command decides (`build-<arch>/build.ninja`):
    uxtheme's CMakeLists has two `spec2def(uxtheme.dll ...)`, uxtheme_vista.spec under
    an option, and the build uses uxtheme.spec. A regular import lib hides picking the
    wrong one (only referenced symbols import); a delay lib carries every thunk, and
    comctl32 grew twelve delay imports. With no build command, a `spec2def(...
    ADD_IMPORTLIB)` only counts when it is the only one for that DLL.
    """
    cands: dict[str, list[tuple[str, str]]] = {}
    for cm in repo.rglob("CMakeLists.txt"):
        rel = cm.relative_to(repo).as_posix()
        if rel.startswith(("build-", "vendor/", "modules/rostests", ".")):
            continue
        text = cm.read_text(errors="replace")
        for m in re.finditer(r"spec2def\(\s*(\S+?)\s+(\S+?\.spec)\b([^)]*)\)", text):
            dll, spec = m.group(1), m.group(2)
            if "ADD_IMPORTLIB" not in m.group(3):
                continue
            spec_rel = (cm.parent / spec).relative_to(repo).as_posix()
            c = cands.setdefault(dll.rsplit(".", 1)[0].lower(), [])
            if (spec_rel, dll) not in c:
                c.append((spec_rel, dll))
    built: dict[str, set[tuple[str, str]]] = {}
    for a in ARCHES:
        for m in re.finditer(
            r"spec2def [^&|\n]*?-n=(\S+) [^&|\n]*?--implib -d=\S*/lib(\w+)_implib\.def "
            r"/tmp/vibeos/(\S+\.spec)",
            _read(repo / f"build-{a}/build.ninja"),
        ):
            # hal's is `hal/halx86/../hal.spec` on x86 and `hal/halarm64/../hal.spec` on arm64.
            spec = os.path.normpath(m.group(3))
            built.setdefault(m.group(2).lower(), set()).add((spec, m.group(1)))
    out: dict[str, tuple[str, str] | None] = {}
    for base in cands.keys() | built.keys():
        if base in built:
            out[base] = next(iter(built[base])) if len(built[base]) == 1 else None
        else:
            out[base] = cands[base][0] if len(cands[base]) == 1 else None
    return out


def cd_dirs(arch: str, output: str, repo: Path = REPO) -> set[str]:
    """The livecd directories the build puts `output` (build-dir relative) in."""
    lsts = [
        p
        for p in (repo / f"build-{arch}/boot").glob("livecd.*.lst")
        if p.name not in ("livecd.cmake.lst", "livecd.stripped.lst")
    ]
    if len(lsts) != 1:
        raise GenError(f"build-{arch}/boot: expected one livecd.<config>.lst, found {lsts}")
    want = f"/tmp/vibeos/build-{arch}/{output}"
    return {
        ln.split("=", 1)[0].rsplit("/", 1)[0]
        for ln in _read(lsts[0]).splitlines()
        if ln.partition("=")[2] == want
    }


def _one_list(arch: str, stem: str, repo: Path) -> Path | None:
    """build-<arch>/boot/<stem>.<config>.lst, the list the build wrote (None: no such CD)."""
    lsts = [
        p for p in (repo / f"build-{arch}/boot").glob(f"{stem}.*.lst") if p.suffixes[-2] != ".cmake"
    ]
    lsts = [p for p in lsts if p.name != f"{stem}.stripped.lst"]
    if len(lsts) > 1:
        raise GenError(f"build-{arch}/boot: several {stem}.<config>.lst: {lsts}")
    return lsts[0] if lsts else None


@functools.cache
def txtsetup_files(repo: Path = REPO) -> frozenset[str]:
    """The files txtsetup.sif names (lower case), read as add_cargo_module reads them."""
    sif = (repo / "boot/bootdata/txtsetup.sif").read_text(errors="replace")
    return frozenset(m.lower() for m in re.findall(r"(?m)^([A-Za-z0-9_.-]+)[ \t]*=", sif))


def cd_placement(arch: str, output: str, repo: Path = REPO) -> list[str]:
    """Every place the CDs put `output`: `<cd>:<path on the CD>` for the loose lists and
    `cab:<dir number>` for reactos.cab, which is what setup installs. A cell that ships
    a file to the live CD but drops it from the cabinet leaves it out of every install:
    add_cargo_module did that to every destination but system32, drivers and winsxs."""
    want = f"/tmp/vibeos/build-{arch}/{output}"
    out = []
    for cd in ("livecd", "bootcd", "bootcdregtest"):
        lst = _one_list(arch, cd, repo)
        if lst is None:
            continue
        out += [
            f"{cd}:{ln.split('=', 1)[0]}"
            for ln in _read(lst).splitlines()
            if ln.partition("=")[2] == want
        ]
    dyn = list((repo / f"build-{arch}/boot/bootdata/packages").glob("reactos.dff.*.dyn"))
    if len(dyn) != 1:
        raise GenError(f"build-{arch}: expected one reactos.dff.<config>.dyn, found {dyn}")
    out += [
        f"cab:{m.group(1)}"
        for m in re.finditer(rf'^"{re.escape(want)}" (\d+)$', _read(dyn[0]), re.M)
    ]
    return sorted(set(out))


def _rc_command(arch: str, rc_obj: str, repo: Path = REPO) -> tuple[str, list[str]]:
    db = _ccdb(repo / f"build-{arch}/compile_commands.json")
    for e in db:
        cmd = e.get("command", "")
        if rc_obj in cmd and e["file"].endswith(".rc"):
            src = e["file"].replace("/tmp/vibeos/", "")
            return src, re.findall(r"\s-D(\S+)", cmd)
    raise GenError(f"no windres command for {rc_obj} in build-{arch}/compile_commands.json")


# ---------------------------------------------------------------- resources

# cmake's windres rule (CMAKE_RC_COMPILE_OBJECT, sdk/cmake/gcc.cmake):
#   windres -O coff --codepage=65001 <INCLUDES> <FLAGS>
#           -DRC_INVOKED -D__WIN32__=1 -D__FLAT__=1 ${I18N_DEFS} <DEFINES> <SOURCE> <OBJECT>
# <DEFINES> comes out sorted. What every module shares is committed as
# cflags/rc.<arch>.flags (`--rc-template`); a cell's build.rs carries the rest
# (Linker::rc), and reactos-link rebuilds the command from the two.
CANON = "/tmp/vibeos"
RC_HEAD = ["-O", "coff", "--codepage=65001"]


def rc_template_path(arch: str, repo: Path = REPO) -> Path:
    return repo / f"sdk/lib/rust/reactos-link/cflags/rc.{arch}.flags"


@dataclass
class RcParts:
    windres: str
    includes: list[str]  # absolute, canonical
    flags: list[str]  # <FLAGS>: CMAKE_RC_FLAGS and whatever the target added
    defs: list[str]  # RC_INVOKED .. the I18N_DEFS
    defines: list[str]  # <DEFINES>, sorted
    src: str


def _rc_parts(cmd: str) -> RcParts:
    import shlex

    t = shlex.split(cmd)
    if t[1:4] != RC_HEAD:
        raise GenError(f"not cmake's windres rule: {' '.join(t[:5])}")
    i, inc = 4, []
    while t[i] == "-I":
        inc.append(t[i + 1])
        i += 2
    # <FLAGS> opens with CMAKE_RC_FLAGS' own `-I <crt>`, the last -I pair.
    if not inc or not inc[-1].endswith("/sdk/include/crt"):
        raise GenError(f"windres command without CMAKE_RC_FLAGS' -I crt: {t[-2]}")
    k = t.index("-DRC_INVOKED", i)
    flags = ["-I", inc.pop(), *t[i:k]]
    j = k + 3
    while t[j].startswith("-DLANGUAGE_"):
        j += 1
    defines = t[j:-2]
    if defines != sorted(defines):
        raise GenError(f"{t[-2]}: <DEFINES> not sorted, the rule this models")
    return RcParts(t[0], inc, flags, t[k:j], defines, t[-2])


def _rc_enc(path: str, arch: str) -> str:
    b = f"{CANON}/build-{arch}"
    if path == b or path.startswith(b + "/"):
        return "@build" + path[len(b) :]
    if path.startswith(CANON + "/"):
        return path[len(CANON) + 1 :]
    raise GenError(f"windres include outside the tree: {path}")


def _rc_dec(tok: str, arch: str) -> str:
    if tok == "@build" or tok.startswith("@build/"):
        return f"{CANON}/build-{arch}" + tok[len("@build") :]
    return f"{CANON}/{tok}"


@dataclass
class RcTemplate:
    windres: str
    std: list[str]  # encoded
    flags: list[str]
    defs: list[str]
    defines: list[str]

    def text(self, arch: str) -> str:
        out = [
            f"# cmake's windres rule, the part every module shares ({arch}). Generated by",
            "# python3 tools/cargo_cell_gen.py --rc-template from "
            "build-<arch>/compile_commands.json;",
            "# reactos-link's Linker::rc adds the module's own includes, flags and defines.",
        ]
        for name, toks in (
            ("windres", [self.windres]),
            ("std", self.std),
            ("flags", self.flags),
            ("defs", self.defs),
            ("defines", self.defines),
        ):
            out.append(f"[{name}]")
            out.extend(toks)
        return "\n".join(out) + "\n"


def _rc_commands(arch: str, repo: Path = REPO) -> list[dict]:
    db = _ccdb(repo / f"build-{arch}/compile_commands.json")
    return [e for e in db if e["file"].endswith(".rc") and "windres" in e.get("command", "")]


def derive_rc_template(arch: str, repo: Path = REPO) -> RcTemplate:
    """What every windres command in the tree shares: the binary, the longest include run
    from sdk/include they all carry, CMAKE_RC_FLAGS, the RC_INVOKED/I18N block, and the
    defines in all of them."""
    parts = [_rc_parts(e["command"]) for e in _rc_commands(arch, repo)]
    if not parts:
        raise GenError(f"build-{arch}: no windres commands")
    windres = {p.windres for p in parts}
    defs = {tuple(p.defs) for p in parts}
    if len(windres) != 1 or len(defs) != 1:
        raise GenError(f"build-{arch}: windres binary or RC_INVOKED block differs by target")
    head = [p.flags for p in parts]
    n = min(len(f) for f in head)
    while n and any(f[:n] != head[0][:n] for f in head):
        n -= 1
    # The standard run: the longest include run from sdk/include that nearly every command
    # carries (freeldr's fdebug.rc sets its own); a command without it lists every -I.
    first = parts[0].includes
    start = first.index(f"{CANON}/sdk/include")

    def has(run: list[str], inc: list[str]) -> bool:
        return any(inc[i : i + len(run)] == run for i in range(len(inc) - len(run) + 1))

    def share(run: list[str]) -> float:
        return sum(has(run, p.includes) for p in parts) / len(parts)

    end = start + 1
    while end < len(first) and share(first[start : end + 1]) >= 0.95:
        end += 1
    common = set(parts[0].defines).intersection(*(set(p.defines) for p in parts[1:]))
    return RcTemplate(
        windres=windres.pop().replace(CANON, "{REPO}"),
        std=[_rc_enc(x, arch) for x in first[start:end]],
        flags=[x.replace(CANON, "{REPO}") for x in head[0][:n]],
        defs=list(defs.pop()),
        defines=sorted(common),
    )


def read_rc_template(arch: str, repo: Path = REPO) -> RcTemplate:
    path = rc_template_path(arch, repo)
    if not path.exists():
        raise GenError(f"{path}: missing (python3 tools/cargo_cell_gen.py --rc-template)")
    sec: dict[str, list[str]] = {}
    cur = None
    for line in path.read_text().splitlines():
        if not line or line.startswith("#"):
            continue
        if line.startswith("[") and line.endswith("]"):
            cur = sec.setdefault(line[1:-1], [])
        elif cur is not None:
            cur.append(line)
    return RcTemplate(
        sec["windres"][0], sec["std"], sec["flags"], sec["defs"], sec.get("defines", [])
    )


@dataclass(frozen=True)
class RcFlags:
    """A module's part of its windres command, as Linker::rc takes it."""

    includes: tuple[str, ...]
    flags: tuple[str, ...]
    defines: tuple[str, ...]


def rc_decode(f: RcFlags, tmpl: RcTemplate, arch: str, src: str) -> list[str]:
    """The windres command reactos-link builds (Linker::rc), minus the output path."""
    sub = lambda x: x.replace("{REPO}", CANON)  # noqa: E731
    out = [sub(tmpl.windres), *RC_HEAD]
    for inc in f.includes:
        for d in tmpl.std if inc == "@std" else [inc]:
            out += ["-I", _rc_dec(d, arch)]
    out += [sub(x) for x in tmpl.flags] + [sub(x) for x in f.flags] + tmpl.defs
    out += sorted(tmpl.defines + [f"-D{d}" for d in f.defines])
    return [*out, src]


def rc_encode(cmd: str, tmpl: RcTemplate, arch: str) -> RcFlags:
    """The module's part of cmake's windres command; refuses unless rc_decode rebuilds the
    command token for token."""
    import shlex

    p = _rc_parts(cmd)
    std = [_rc_dec(x, arch) for x in tmpl.std]
    inc = list(p.includes)
    at = next((i for i in range(len(inc) - len(std) + 1) if inc[i : i + len(std)] == std), None)
    if at is None:
        includes = [_rc_enc(x, arch) for x in inc]
    else:
        includes = [_rc_enc(x, arch) for x in inc[:at]] + ["@std"]
        includes += [_rc_enc(x, arch) for x in inc[at + len(std) :]]
    tflags = [x.replace("{REPO}", CANON) for x in tmpl.flags]
    if p.flags[: len(tflags)] != tflags:
        raise GenError(f"{p.src}: <FLAGS> do not open with the template's")
    extra = p.flags[len(tflags) :]
    if any(CANON in x for x in extra):
        raise GenError(f"{p.src}: a path in the target's own RC flags: {extra}")
    left = list(p.defines)
    for d in tmpl.defines:
        if d not in left:
            raise GenError(f"{p.src}: lacks the global define {d}")
        left.remove(d)
    if any(not d.startswith("-D") for d in left):
        raise GenError(f"{p.src}: a non -D token among the defines")
    f = RcFlags(tuple(includes), tuple(extra), tuple(d[2:] for d in left))
    if rc_decode(f, tmpl, arch, p.src) != shlex.split(cmd)[:-1]:
        raise GenError(f"{p.src}: the command does not rebuild from the template")
    return f


def rc_flags_for(rc_rel: str, marker: str, repo: Path = REPO) -> dict[str, RcFlags]:
    """Per arch, the cell's part of cmake's command compiling `rc_rel` (in the target whose
    `CMakeFiles/<t>.dir` is `marker`)."""
    out = {}
    for a in ARCHES:
        tmpl = read_rc_template(a, repo)
        hits = [
            e
            for e in _rc_commands(a, repo)
            if e["file"] == f"{CANON}/{rc_rel}" and marker in e["command"]
        ]
        if len(hits) > 1:
            raise GenError(f"{rc_rel}: {len(hits)} windres commands in {marker} on {a}")
        if hits:
            out[a] = rc_encode(hits[0]["command"], tmpl, a)
    if not out:
        raise GenError(f"{rc_rel}: no windres command in {marker} on any arch")
    # An arch cmake does not build the module on never runs its build script there.
    have = next(iter(out.values()))
    return {a: out.get(a, have) for a in ARCHES}


# ---------------------------------------------------------------- cmake side


def _statements(text: str) -> list[tuple[int, int, str, str]]:
    """(start, end, command, args) for every top-level cmake command call."""
    out, i = [], 0
    for m in re.finditer(r"(?m)^[ \t]*([A-Za-z_][A-Za-z0-9_]*)[ \t]*\(", text):
        if m.start() < i:
            continue
        depth, j = 0, m.end() - 1
        while j < len(text):
            c = text[j]
            if c == "(":
                depth += 1
            elif c == ")":
                depth -= 1
                if depth == 0:
                    break
            elif c == "#" and depth == 1 and text[j - 1] in " \t\n":
                j = text.index("\n", j) - 1 if "\n" in text[j:] else len(text)
            j += 1
        end = j + 1
        if end < len(text) and text[end] == "\n":
            end += 1
        out.append((m.start(), end, m.group(1).lower(), text[m.end() : j]))
        i = end
    return out


def _args(s: str) -> list[str]:
    s = re.sub(r"#[^\n]*", "", s)
    return re.findall(r'"[^"]*"|\S+', s)


def cmake_facts(cmake: Path, target: str, libs: list[str] | None = None) -> dict:
    """What the CMakeLists says about `target`. `libs` (the link edge's libraries) names
    the body when its rust_staticlib_variant is not called `<target>_rust`: the variant
    whose `lib<name>.a` the module actually links (cmd_help_rust for help.exe)."""
    text = cmake.read_text()
    facts: dict = {
        "destinations": [],
        "name_on_cd": None,
        "body": None,
        "body_var": "",
        "transfer": [],
        "unknown": [],
        "unicode": False,
        "body_rustflags": {},
        "body_env_block": "",
        "extra_cd": [],
        "no_cab": False,
    }
    variants = {}
    linked = {lib.split("/")[-1] for lib in libs or []}
    defined = False
    for _s, _e, cmd, raw in _statements(text):
        a = _args(raw)
        if cmd == "rust_staticlib_variant" and len(a) >= 4:
            variants[a[0]] = a
        if cmd in ("add_executable", "add_library") and a and a[0] == target:
            defined = True
        if not a or (a[0] != target and not (cmd == "add_cd_file" and target in a)):
            # The rewrite drops the SOURCE lists, so another target built from them
            # (gdi32_vista_static, which gdi32 links) would be left with no sources.
            # Fine when the list holds only placeholders and headers (gdi32_vista_static:
            # one rust_cohab_stub.c): the rewrite spells the list out in that statement.
            if re.search(r"\$\{(SOURCE|PCH_SKIP_SOURCE)\}", raw) and not _inlinable_source(
                text, cmake.parent
            ):
                facts["unknown"].append(f"{cmd} {a[0] if a else ''} shares ${{SOURCE}}")
            continue
        if cmd == "set_module_type" and "UNICODE" in a:
            facts["unicode"] = True
        if cmd == "set_module_type" and len(a) > 1:
            facts["module_type"] = a[1]
        if cmd == "add_cd_file" and not {"all", "livecd"} & set(_cd_for(a)):
            # A boot-CD-only placement (usetup as the setup boot's smss.exe, welcome as
            # the CD root's setup.exe): add_cargo_cd_file, in the statement's own words.
            facts["extra_cd"].append(" ".join(_drop_target(a, target)))
        elif cmd == "add_cd_file":
            facts["no_cab"] = facts["no_cab"] or "NO_CAB" in a
            if "DESTINATION" in a:
                dest = a[a.index("DESTINATION") + 1]
                # Several are fine (comctl32 also ships in two winsxs assemblies);
                # model() checks them against the CD lists the build wrote, which is
                # what refuses winprint's one-per-arch prtprocs dirs.
                if dest not in facts["destinations"]:
                    facts["destinations"].append(dest)
            if "NAME_ON_CD" in a:
                facts["name_on_cd"] = a[a.index("NAME_ON_CD") + 1]
        elif cmd.lower() in TRANSFERRED:
            facts["transfer"].append(raw)
        elif cmd == "set_target_properties" and _names_only(a):
            pass  # the output name comes from the link edge
        elif cmd == "target_link_options" and _subsystem_only(a):
            pass  # the subsystem comes from each arch's link edge
        elif cmd == "target_sources" and _ssp_objects(a):
            # atapi's `if(STACK_PROTECTOR) target_sources(atapi ... gcc_ssp_nt)`: the
            # rewrite makes the option a configure error for this module.
            facts["ssp"] = True
        elif cmd not in REPLACED and cmd not in KEPT:
            facts["unknown"].append(cmd)
    if not defined:
        # The keyboard layouts are made in a foreach over `${_keyboard_layout}`: there is
        # no statement of the target's own to replace.
        facts["unknown"].append("target made in a loop or function")
    named = [n for n in variants if n in (f"{target}_rust",) or n.removesuffix("_rust") == target]
    named = named or [n for n in variants if f"lib{n}.a" in linked]
    if len(named) == 1:
        a = variants[named[0]]
        feats = a[2].strip('"').replace(";", " ").split()
        facts["body"] = (a[1], feats, a[3].upper() == "TRUE")
        facts["body_var"] = named[0]
        if len(a) > 4:
            # calc/taskmgr/wordpad pass `${X_RUST_ENV}` (i386 +sse,+sse2 for the body).
            # Only that exact shape is modelled: the cell builds the body with the same
            # RUSTFLAGS through rust_build_rustflags. Anything else is refused, since
            # pe_equiv cannot see a codegen change.
            env = _body_env(text, a[4:])
            if env is None:
                facts["unknown"].append(f"rust_staticlib_variant extras {a[4:]}")
            else:
                facts["body_rustflags"], facts["body_env_block"] = env
    elif len(named) > 1:
        facts["unknown"].append(f"several bodies {named}")
    elif (custom := _custom_body(text)) is not None:
        # The drivers build their body with a custom command of their own.
        facts["body"] = (custom["crate"], [], True)
        facts["body_rustflags"] = custom["rustflags"]
        facts["custom_body"] = custom
    return facts


def _arch_values(text: str, var: str) -> dict[str, str]:
    """{arch: value} for `set(<var> "<value>")` inside an `if(ARCH STREQUAL ...)` chain."""
    out: dict[str, str] = {}
    arch = None
    for _s, _e, cmd, raw in _statements(text):
        a = _args(raw)
        if cmd in ("if", "elseif"):
            m = re.fullmatch(r'ARCH STREQUAL "?(\w+)"?', raw.strip())
            arch = m.group(1) if m else "?"
        elif cmd in ("else", "endif"):
            arch = None if cmd == "endif" else "?"
        elif cmd == "set" and a[:1] == [var]:
            if arch in (None, "?") or len(a) != 2:
                return {}
            out[arch] = a[1].strip('"')
    return out


def _custom_body(text: str) -> dict | None:
    """A driver's own body build, in its one shape:

        add_custom_command(OUTPUT ${X_RUST_LIB}
            COMMAND ${CMAKE_COMMAND} -E env CARGO_TARGET_DIR=${X_RUST_TARGET_DIR}
                RUSTFLAGS=${X_RUST_RUSTFLAGS}
                cargo build --release --target ${X_RUST_TARGET} -p <crate> --manifest-path ...
            COMMAND ${CMAKE_COMMAND} -E copy ... ${X_RUST_LIB} ...)
        add_custom_target(rust_x DEPENDS ${X_RUST_LIB})

    with the target and RUSTFLAGS set per arch. The cell builds the same crate on the
    same builtin triple with the same RUSTFLAGS (Linker::rust_user_target). Anything
    else in the command (features, another env, a second crate) is refused."""
    cmds = [
        (s, e, raw)
        for s, e, cmd, raw in _statements(text)
        if cmd == "add_custom_command" and re.search(r"\bcargo build\b", raw)
    ]
    if len(cmds) != 1:
        return None
    raw = re.sub(r"#[^\n]*", "", cmds[0][2])
    m = re.fullmatch(
        r"\s*OUTPUT \$\{(\w+)\}\s+"
        r"COMMAND \$\{CMAKE_COMMAND\} -E env\s+CARGO_TARGET_DIR=\$\{(\w+)\}\s+"
        r"RUSTFLAGS=\$\{(\w+)\}\s+"
        r"cargo build --release --target \$\{(\w+)\}\s+-p ([\w-]+)\s+"
        r"--manifest-path \$\{(\w+)\}/Cargo\.toml\s+"
        r"COMMAND \$\{CMAKE_COMMAND\} -E copy\s+\$\{\2\}/\$\{\4\}/release/(lib\w+\.a)\s+\$\{\1\}\s+"
        r"DEPENDS \$\{(\w+)\}\s+\$\{\6\}/\5/Cargo\.toml\s+"
        r"WORKING_DIRECTORY \$\{\6\}\s+"
        r'COMMENT "[^"]*"\s+VERBATIM\s*',
        raw,
    )
    if not m:
        return None
    lib, tdir, rf, tgt, crate, ws, archive, srcs = m.groups()
    targets = _arch_values(text, tgt)
    flags = _arch_values(text, rf)
    if targets != TARGET or set(flags) != set(ARCHES):
        return None
    out_lib = re.search(
        rf'set\({lib} "\$\{{CMAKE_CURRENT_BINARY_DIR\}}/({re.escape(archive)})"\)', text
    )
    custom_target = re.search(rf"add_custom_target\((\w+) DEPENDS \$\{{{lib}\}}\)", text)
    if not out_lib or not custom_target:
        return None
    return {
        "crate": crate,
        "archive": archive,
        "rustflags": {a: f for a, f in flags.items() if f},
        "vars": {lib, tdir, rf, tgt, ws, srcs},
        "custom_target": custom_target.group(1),
    }


def _source_list(text: str) -> list[str]:
    """Every entry `list(APPEND SOURCE ...)` adds, in order."""
    out: list[str] = []
    for _s, _e, cmd, raw in _statements(text):
        a = _args(raw)
        if cmd == "list" and a[:2] == ["APPEND", "SOURCE"]:
            out += a[2:]
    return out


def _inlinable_source(text: str, cmake_dir: Path) -> list[str] | None:
    """The SOURCE list when it is only placeholder stubs and headers, else None."""
    src = _source_list(text)
    if "PCH_SKIP_SOURCE" in text or not src:
        return None
    for x in src:
        if x.endswith((".h", ".hpp")):
            continue
        if not (x.endswith(".c") and (cmake_dir / x).exists() and _placeholder(cmake_dir / x)):
            return None
    return src


def _cd_for(a: list[str]) -> list[str]:
    """add_cd_file's FOR list (it runs to the end of the call)."""
    return a[a.index("FOR") + 1 :] if "FOR" in a else []


def _drop_target(a: list[str], target: str) -> list[str]:
    i = a.index("TARGET")
    if a[i + 1] != target:
        raise GenError(f"{target}: add_cd_file names TARGET {a[i + 1]}")
    return a[:i] + a[i + 2 :]


def _body_env(text: str, extras: list[str]) -> tuple[dict[str, str], str] | None:
    """`extras` is one `${VAR}` that is empty but on one arch `ENV "RUSTFLAGS=..."`:
    ({arch: flags}, the block that sets it). None for any other shape."""
    m = re.fullmatch(r"\$\{(\w+)\}", extras[0]) if len(extras) == 1 else None
    if not m:
        return None
    var = re.escape(m.group(1))
    block = re.search(
        rf"set\({var}\)\n"
        rf'if\(ARCH STREQUAL "(\w+)"\)\n'
        rf'\s*set\({var} ENV "RUSTFLAGS=([^"]*)"\)\n'
        r"endif\(\)\n",
        text,
    )
    if not block or block.group(1) not in ARCHES:
        return None
    if len(re.findall(rf"\b{var}\b", text)) != 3:
        return None  # set twice above, read once by the variant: nothing else
    return {block.group(1): block.group(2)}, block.group(0)


def _ssp_objects(a: list[str]) -> bool:
    """target_sources(<t> PRIVATE $<TARGET_OBJECTS:gcc_ssp_...>), under STACK_PROTECTOR."""
    return (
        len(a) == 3
        and a[1] == "PRIVATE"
        and bool(re.fullmatch(r"\$<TARGET_OBJECTS:gcc_ssp_\w+>", a[2]))
    )


def _subsystem_only(a: list[str]) -> bool:
    """target_link_options(<t> PRIVATE "-Wl,--subsystem,...") (welcome on i386)."""
    opts = [x.strip('"') for x in a[1:] if x not in ("PRIVATE", "PUBLIC")]
    return bool(opts) and all(re.fullmatch(r"-Wl,--subsystem,[\w:.]+", o) for o in opts)


def _names_only(a: list[str]) -> bool:
    """set_target_properties(<t> PROPERTIES K V ...) touching only the output name."""
    if len(a) < 4 or a[1] != "PROPERTIES" or len(a) % 2:
        return False
    return all(k in NAME_PROPERTIES for k in a[2::2])


def _retarget(cmd: str, raw: str, new: str) -> str:
    """A transferred statement (`raw` is its argument text), re-aimed at `new`."""
    first = _args(raw)[0]
    return f"{cmd}({raw.strip().replace(first, new, 1)})"


def rewrite_cmake(cmake: Path, cell: Cell) -> str:
    """The module's CMakeLists with the target's link statements replaced by the cell."""
    text = cmake.read_text()
    if cell.body_env_block:
        # The body's RUSTFLAGS now live in build.rs (Cell.body_rustflags).
        text = text.replace(cell.body_env_block, "", 1)
    keep, pos = [], 0
    body_lib = None
    body_var = cell.body_var or f"{cell.target}_rust"
    transferred: list[tuple[str, str]] = []
    # `add_dependencies(m evtmsgstr)` orders a GENERATED input (an mc header the rc #includes)
    # before the compile that reads it. The cell drops the target's add_dependencies, and the
    # compile now lives on `<m>_res`/`<m>_shims`, so the ordering moves there with it — or a
    # fresh build races windres against the mc step. The `rust_*` body deps are the cell's own.
    build_deps: list[str] = []
    rc_gen_deps: list[str] = []
    shared = any(
        re.search(rf"\$\{{{body_var}_LIB\}}|rust_{body_var}\b", raw)
        for _s, _e, _cmd, raw in _statements(text)
        if _args(raw)[:1] != [cell.target] and _cmd != "rust_staticlib_variant"
    )
    # `# main.c  # replaced by Rust (...)` lines are how cohab_census and the coverage
    # tools know a TU was translated: whatever statement held them, they are kept.
    carried: list[str] = []
    # What the target linked: their INTERFACE usage requirements reached its compiles
    # (libmsvcrt's `_DLL __USE_CRTIMP`, without which a flex scanner's `stdin` is not
    # dllimport and `_iob` is left undefined), so the <m>_shims compiles must link them.
    usage: list[str] = []

    def carry(chunk: str) -> None:
        carried.extend(ln.strip() for ln in chunk.splitlines() if SRC_IN_COMMENT.match(ln))

    guards = _enclosing_ifs(text, cell.target)

    for s, e, cmd, raw in _statements(text):
        a = _args(raw)
        drop = False
        keep_comment = False
        if (
            cmd in REPLACED
            and a
            and (a[0] == cell.target or (cmd == "add_cd_file" and cell.target in a))
        ):
            drop = True
        if a and a[0] == cell.target and cmd == "add_importlibs":
            usage += [f"lib{x}" for x in a[1:]]
        if a and a[0] == cell.target and cmd == "target_link_libraries":
            usage += [x for x in a[1:] if not x.startswith("${") and x not in ("PRIVATE", "PUBLIC")]
        if cmd == "add_dependencies" and a and a[0] == cell.target:
            build_deps += [d for d in a[1:] if not d.startswith("rust_") and d not in build_deps]
        rc_deps = _rc_object_depends(cmd, raw) if cell.rc else None
        if rc_deps is not None:
            # The engine compiles the .rc; a generated input it embeds orders the cell.
            drop = True
            rc_gen_deps += [d for d in rc_deps if "CMAKE_CURRENT_BINARY_DIR" in d]
        if a and a[0] == cell.target and cmd.lower() in TRANSFERRED:
            drop = True  # re-emitted below, on the OBJECT libraries
            transferred.append((cmd, raw))
        if a and a[0] == cell.target and cmd == "set_target_properties" and _names_only(a):
            drop = True
        if a and a[0] == cell.target and cmd == "target_link_options" and _subsystem_only(a):
            drop = True  # build.rs sets each arch's subsystem
        if a[:1] == [cell.target] and cmd == "target_sources" and _ssp_objects(a):
            keep.append(text[pos:s])
            keep.append(
                text[s:e].replace(
                    text[s:e].strip(),
                    f'message(FATAL_ERROR "{cell.target} is linked by cargo, which does not link '
                    f'{a[2]} (STACK_PROTECTOR)")',
                    1,
                )
            )
            pos = e
            continue
        cb = cell.custom_body
        if cb and (
            (
                cmd in ("set", "rust_storage_target", "rust_crate_sources")
                and a[:1]
                and a[0] in cb["vars"]
            )
            or (cmd == "add_custom_command" and re.search(r"\bcargo build\b", raw))
            or (cmd == "add_custom_target" and a[:1] == [cb["custom_target"]])
        ):
            drop = True  # the body's custom command: the cell builds the crate now
            keep_comment = True  # a driver's header comment sits on its first set()
        if cmd == "rust_staticlib_variant" and a and a[0] == body_var:
            # Kept when another target still links it (a body two modules share).
            drop, body_lib = not shared, a[0]
        if (
            cmd == "list"
            and len(a) > 2
            and a[0] == "APPEND"
            and a[1] in ("SOURCE", "PCH_SKIP_SOURCE")
        ):
            drop = True  # only the replaced add_* consumed these
        if not drop and "${SOURCE}" in raw and _args(raw)[:1] != [cell.target]:
            # Another target built from the dropped SOURCE list (gdi32_vista_static):
            # cmake_facts accepted it only when the list is placeholders and headers.
            src = _inlinable_source(text, cmake.parent)
            if src is None:
                raise GenError(f"{cmake}: {cmd} shares ${{SOURCE}}")
            new_raw = raw.replace("${SOURCE}", " ".join(src))
            keep.append(text[pos:s])
            keep.append(text[s:e].replace(raw, new_raw, 1))
            pos = e
            continue
        gap = text[pos:s]
        if drop and not keep_comment:
            # A comment block attached to a dropped statement describes it: drop it too.
            lines = gap.split("\n")
            block_lines: list[str] = []
            while len(lines) > 1 and lines[-2].lstrip().startswith("#"):
                block_lines.insert(0, lines[-2])
                del lines[-2]
            # A `# x.c  # replaced by Rust` line is carried alone; prose that names
            # files (scsiport's "C bodies replaced by Rust (fdo.c ioctl.c ...") is
            # carried whole, or a sentence is cut off mid-way.
            if any(SRC_IN_COMMENT.match(ln) for ln in block_lines):
                carried.extend(ln.strip() for ln in block_lines)
            gap = "\n".join(lines)
        if drop and keep_comment and gap.rstrip(" \t").endswith("\n") and "#" in gap:
            # A blank line after the kept header, so the empty-if cleanup below does
            # not take it for the comment of the if-chain these statements leave.
            gap += "\n"
        keep.append(gap)
        if not drop:
            keep.append(text[s:e])
        else:
            carry(text[s:e])
        pos = e
    keep.append(text[pos:])
    out = "".join(keep).lstrip("\n")
    if body_lib and not shared and re.search(rf"\$\{{{body_lib}_LIB\}}|rust_{body_lib}\b", out):
        raise GenError(f"{cmake}: something else still uses {body_lib}")
    for v in cell.custom_body.get("vars", ()):
        if re.search(rf"\$\{{{v}\}}", out):
            raise GenError(f"{cmake}: something else still uses ${{{v}}}")
    # An if(ARCH ...) chain whose only statements were the target's (winprint's
    # add_cd_file per arch) is left with nothing in it.
    # The comment block right above such an if (welcome's "Reduce the required subsystem
    # ...") explained the statement it held, and goes with it.
    # Repeated, because emptying an inner chain empties the if around it (pciide's
    # per-arch body flags inside its NOT ENABLE_UNIATA).
    empty_if = re.compile(
        r"^(?:[ \t]*#[^\n]*\n)*[ \t]*if\([^\n]*\)\n(?:[ \t]*\n)*"
        r"(?:[ \t]*(?:elseif\([^\n]*\)|else\(\))\n(?:[ \t]*\n)*)*[ \t]*endif\(\)\n",
        re.M,
    )
    while True:
        out, n = empty_if.subn("", out)
        if not n:
            break
    # A GLOB that only fed a dropped add_rc_deps.
    for m in re.finditer(r"file\(GLOB (\w+)[^)]*\)\n", out):
        if "${" + m.group(1) + "}" not in out:
            out = out.replace(m.group(0), "", 1)
    out = re.sub(r"\n{3,}", "\n\n", out).rstrip() + "\n"
    block = [
        "",
        f"# {cell.output_name} is all Rust ({cell.body_crate}) and cargo links it:",
        f"# sdk/lib/rust/{cell.crate} (reactos-link, generated by tools/cargo_cell_gen.py).",
        "# The C stays on disk, uncompiled.",
    ]
    if carried:
        block.append("# The translated C, kept on disk:")
        block += list(dict.fromkeys(carried))
    extra = []
    shims = f"{cell.target}_shims"
    # Lines that configure the shims library: guarded with it when some arch has none.
    shim_lines: list[str] = []
    guarded = bool(cell.arch_shims)
    if cell.c_shims or cell.arch_shims:
        block.append(
            "# The C still compiled, in an OBJECT library whose compile the cargo link replays."
        )
        names = [
            f"${{CMAKE_CURRENT_BINARY_DIR}}/{Path(c).name}"
            if c in cell.generated
            else f"${{REACTOS_SOURCE_DIR}}/{c}"
            if c in cell.foreign
            else Path(c).name
            for c in cell.c_shims
        ]
        if guarded:
            # Some of it on some arches only, and an OBJECT library may not be empty.
            var = f"{cell.target.upper()}_SHIMS"
            block.append(f"set({var} {' '.join(names)})".replace(" )", ")"))
            for a in ARCHES:
                if cell.arch_shims[a]:
                    block.append(f'if(ARCH STREQUAL "{a}")')
                    names_a = " ".join(Path(c).name for c in cell.arch_shims[a])
                    block.append(f"    list(APPEND {var} {names_a})")
                    block.append("endif()")
            block.append(f"set({var}_TARGET)")
            shim_lines.append(f"add_library({shims} OBJECT ${{{var}}})")
            shim_lines.append(f"set({var}_TARGET {shims})")
        else:
            shim_lines.append(f"add_library({shims} OBJECT {' '.join(names)})")
        if usage:
            shim_lines.append(
                f"target_link_libraries({shims} PRIVATE {' '.join(dict.fromkeys(usage))})"
            )
        if cell.rc_defines:
            shim_lines.append(
                f"target_compile_definitions({shims} PRIVATE {' '.join(cell.rc_defines)})"
            )
        extra.append(shims)
    if cell.rc and "psdk" not in extra:
        # The engine compiles the .rc (Linker::rc); winnt.h is psdk's, generated.
        extra.append("psdk")
    extra += [d for d in rc_gen_deps if d not in extra]
    objlibs = [x for x in extra if x == shims]
    if transferred and objlibs:
        block.append("# How cmake compiled the target's sources, now on the OBJECT libraries.")
        for lib in objlibs:
            lines = [_retarget(c, r, lib) for c, r in transferred]
            (shim_lines if lib == shims else block).extend(lines)
    if build_deps and objlibs:
        block.append("# Generated inputs those compiles read, ordered before them.")
        for lib in objlibs:
            line = f"add_dependencies({lib} {' '.join(build_deps)})"
            (shim_lines if lib == shims else block).append(line)
    if guarded:
        block.append(f"if({cell.target.upper()}_SHIMS)")
        block += ["    " + ln for ln in shim_lines]
        block.append("endif()")
        extra[extra.index(shims)] = f"${{{cell.target.upper()}_SHIMS_TARGET}}"
    else:
        block += shim_lines
    # cmake builds what the cell still reuses; the engine builds `engine:` libraries.
    reused = {
        a: [x for x in libs if not x.startswith("engine:")] for a, libs in cell.static_libs.items()
    }
    every = sorted({lib for libs in reused.values() for lib in libs})
    by_arch = {a: set(libs) for a, libs in reused.items()}
    common = [lib for lib in every if all(lib in by_arch[a] for a in ARCHES)]
    if every:
        var = f"{cell.target.upper()}_REUSED"
        block.append(f"set({var}")
        block += [f"    ${{CMAKE_BINARY_DIR}}/{lib}" for lib in common]
        block[-1] += ")" if common else ""
        if not common:
            block[-1] = f"set({var})"
        for a in ARCHES:
            only = sorted(by_arch[a] - set(common))
            if only:
                block.append(f'if(ARCH STREQUAL "{a}")')
                block += [f"    list(APPEND {var} ${{CMAKE_BINARY_DIR}}/{lib})" for lib in only]
                block.append("endif()")
        extra.append(f"${{{var}}}")
    named = {lib for libs in cell.static_libs.values() for lib in libs}
    extra += sorted({d for lib in named for d in _engine_deps(lib)})
    kind = cell.kernel_kind or ("win32exe" if cell.module_kind == "Win32Exe" else "win32dll")
    call = [f"add_cargo_module({cell.target} {cell.crate} {cell.output_name}"]
    call.append(f"    KIND {kind}")
    call.append(f"    BODY_CRATE {cell.body_crate}")
    if cell.arch_destinations:
        var = f"{cell.target.upper()}_DESTINATION"
        for i, a in enumerate(ARCHES):
            block.append(f'{"if" if i == 0 else "elseif"}(ARCH STREQUAL "{a}")')
            block.append(f"    set({var} {' '.join(cell.arch_destinations[a])})")
        block.append("endif()")
        call.append(f"    DESTINATION ${{{var}}}")
    elif cell.destinations:
        call.append(f"    DESTINATION {' '.join(cell.destinations)}")
    else:
        call.append("    NO_CD")
    if cell.name_on_cd:
        call.append(f"    NAME_ON_CD {cell.name_on_cd}")
    if cell.delay_import_libs:
        call.append("    DELAY_IMPORTS")
    if cell.no_cab:
        call.append("    NO_CAB")
    extra = list(dict.fromkeys(extra))
    if extra:
        call.append(f"    EXTRA_DEPENDS {' '.join(extra)})")
    else:
        call[-1] += ")"
    if build_deps and not objlibs:
        call.append(f"add_dependencies({cell.target} {' '.join(build_deps)})")
    call += [f"add_cargo_cd_file({cell.target} {x})" for x in cell.extra_cd]
    lines = block + call
    for cond in reversed(guards):
        # The target was built only under this condition (pciide's NOT ENABLE_UNIATA), so
        # the cell is too; the emptied original if is gone with the cleanup above.
        lines = [lines[0], f"if({cond})"] + ["    " + ln if ln else ln for ln in lines[1:]]
        lines.append("endif()")
    return out + "\n".join(lines) + "\n"


def _enclosing_ifs(text: str, target: str) -> list[str]:
    """The conditions of the if() blocks the target's add_library/add_executable sits in,
    outermost first. Only a plain if() is modelled: a target in an elseif/else branch
    depends on the conditions before it, which one wrapping if cannot say."""
    stack: list[str | None] = []
    for _s, _e, cmd, raw in _statements(text):
        if cmd == "if":
            stack.append(" ".join(raw.split()))
        elif cmd in ("elseif", "else") and stack:
            stack[-1] = None
        elif cmd == "endif" and stack:
            stack.pop()
        elif cmd in ("add_library", "add_executable") and _args(raw)[:1] == [target]:
            if any(c is None for c in stack):
                raise GenError(f"{target}: its link is in an elseif/else branch")
            return [c for c in stack if c is not None]
    return []


# cmake static libraries reactos-link builds itself (`engine_lib`): a cell names them
# `engine:<name>` and no longer waits on cmake to build them.
ENGINE_LIBS = {
    "sdk/lib/crt/CMakeFiles/oldnames.dir/oldnames.a": "engine:oldnames",
    "sdk/lib/crt/libchkstk.a": "engine:chkstk",
    "sdk/lib/pseh/libpseh.a": "engine:pseh",
    "sdk/lib/libwine/libwine.a": "engine:wine",
    "sdk/lib/conutils/libconutils.a": "engine:conutils",
    "sdk/lib/conutils/libconutils_rust.a": "engine:conutils_rust",
    "sdk/lib/uuid/libuuid.a": "engine:uuid",
    "sdk/lib/delayimp/libdelayimp_rust.a": "engine:delayimp_rust",
    "sdk/lib/evtlib/libevtlib_rust.a": "engine:evtlib_rust",
    "sdk/lib/skiplist/libskiplist_rust.a": "engine:skiplist_rust",
    "dll/win32/hid/libhid_hidparser_rust.a": "engine:hid_hidparser_rust",
    "sdk/lib/evtlib/libevtlib.a": "engine:c:evtlib",
    "sdk/lib/skiplist/libskiplist16.a": "engine:c:skiplist16",
    "sdk/lib/udmihelp/libudmihelp.a": "engine:c:udmihelp",
    "sdk/lib/dmilib/libdmilib.a": "engine:c:dmilib",
    "sdk/lib/dxguid/libdxguid.a": "engine:c:dxguid",
    "sdk/lib/tzlib/libtzlib.a": "engine:c:tzlib",
    "sdk/lib/tzlib/libtzlib_rust.a": "engine:tzlib_rust",
    "sdk/lib/scrnsave/libscrnsave.a": "engine:c:scrnsave",
    "dll/appcompat/shims/shimlib/libshimlib.a": "engine:c:shimlib",
    "win32ss/user/winsrv/concfg/libconcfg.a": "engine:c:concfg",
    "win32ss/user/winsrv/concfg/libconcfg_rust.a": "engine:concfg_rust",
    "win32ss/libwin32ksys.a": "engine:c:win32ksys",
    "dll/opengl/mesa/libmesa.a": "engine:c:mesa",
    "sdk/lib/3rdparty/zlib/libzlib_solo.a": "engine:c:zlib_solo",
    "sdk/lib/inflib/libinflib_rust.a": "engine:inflib_rust",
    "sdk/lib/crt/libmemcmp.a": "engine:c:memcmp",
    "sdk/lib/crt/liblibcntpr.a": "engine:c:libcntpr",
    "sdk/lib/crt/libcntpr_rust.a": "engine:cntpr_rust",
    "sdk/lib/cryptlib/libcryptlib_rust.a": "engine:cryptlib_rust",
    "sdk/lib/drivers/sptilib/libsptilib_rust.a": "engine:sptilib_rust",
    "sdk/lib/ioevent/libioevent.a": "engine:c:ioevent",
    "sdk/lib/wdmguid/libwdmguid.a": "engine:c:wdmguid",
    "sdk/lib/drivers/ntoskrnl_vista/libntoskrnl_vista_rust.a": "engine:ntoskrnl_vista_rust",
    "sdk/lib/drivers/ntoskrnl_vista/libntoskrnl_vista_utf8.a": "engine:ntoskrnl_vista_utf8",
    "sdk/lib/3rdparty/freetype/libfreetype.a": "engine:c:freetype",
    "sdk/lib/vcruntime/libsetjmp.a": "engine:c:setjmp",
    "sdk/lib/drivers/aux_klib/libaux_klib.a": "engine:c:aux_klib",
}


@functools.lru_cache(maxsize=1)
def _library_index(repo: Path = REPO) -> dict[str, tuple[str, list[str], list[str]]]:
    """name -> (kind, args, add_dependencies) for every `add_library` and
    `rust_staticlib_variant` in the tree's CMakeLists. A static library's archive is
    `lib<name>.a` wherever cmake puts it (libsamplerate's binary dir is not its source dir),
    so the name is the key. A name defined twice maps to nothing: never guess."""
    found: dict[str, list[tuple[str, list[str]]]] = {}
    deps: dict[str, list[str]] = {}
    for cm in repo.glob("**/CMakeLists.txt"):
        if cm.parts[len(repo.parts)].startswith(("build-", ".")):
            continue
        text = cm.read_text(errors="replace")
        if "add_library" not in text and "rust_staticlib_variant" not in text:
            continue
        for _s, _e, cmd, raw in _statements(text):
            a = _args(raw)
            if not a:
                continue
            if cmd in ("add_library", "rust_staticlib_variant"):
                found.setdefault(a[0], []).append((cmd, a))
            elif cmd == "add_dependencies":
                deps.setdefault(a[0], []).extend(a[1:])
    return {
        name: (defs[0][0], defs[0][1], deps.get(name, []))
        for name, defs in found.items()
        if len(defs) == 1
    }


_NOT_STATIC = {"SHARED", "MODULE", "OBJECT", "INTERFACE", "IMPORTED", "ALIAS"}


def engine_lib(lib: str, repo: Path = REPO) -> tuple[str, list[str]] | None:
    """The `engine:` entry that builds cmake's `lib`, and the cmake targets its compile
    must order after; None when the engine cannot build it.

    The hand-kept ENGINE_LIBS table first; then any `rust_staticlib_variant` archive
    becomes `engine:crate:<crate>:<features>:<defaults>` (what that function's
    `cargo build` does, which `engine_crate` repeats; a variant passing ENV is not
    modelled), and any other cmake static library `engine:c:<target>` (its compile
    replayed from compile_commands.json), after the targets its `add_dependencies` names.
    """
    if lib in ENGINE_LIBS:
        name = ENGINE_LIBS[lib]
        return name, ENGINE_LIB_DEPS.get(name, "").split()
    m = re.fullmatch(r"(?:.*/)?lib([\w.+-]+)\.a", lib)
    entry = _library_index(repo).get(m.group(1)) if m else None
    if entry is None:
        return None
    kind, a, deps = entry
    if kind == "rust_staticlib_variant":
        if len(a) < 4 or "ENV" in a[4:]:
            return None
        feats = ",".join(f for f in re.split(r"[ ,;]+", a[2].strip('"')) if f)
        return f"engine:crate:{a[1]}:{feats}:{a[3].upper()}", []
    if any(x in _NOT_STATIC for x in a[1:2]):
        return None
    return f"engine:c:{a[0]}", [d for d in deps if not d.startswith("rust_")] or ["psdk"]


# The engine replays cmake's compile of these, which reads what cmake generates (psdk
# and xdk headers, uuid's widl output): the cell builds after the targets that make it.
ENGINE_LIB_DEPS = {
    "engine:wine": "psdk",
    "engine:conutils": "xdk",
    "engine:uuid": "uuid_sources",
    "engine:c:evtlib": "xdk",
    "engine:c:skiplist16": "psdk",
    "engine:c:udmihelp": "psdk",
    "engine:c:dmilib": "psdk",
    "engine:c:dxguid": "psdk",
    "engine:c:tzlib": "xdk",
    "engine:c:scrnsave": "psdk",
    "engine:c:shimlib": "xdk",
    "engine:c:concfg": "xdk",
    "engine:c:win32ksys": "asm",
    "engine:c:mesa": "psdk",
    "engine:c:memcmp": "psdk",
    "engine:c:libcntpr": "psdk asm",
    "engine:c:ioevent": "xdk",
    "engine:c:wdmguid": "psdk",
    "engine:c:freetype": "psdk",
    "engine:c:setjmp": "psdk asm",
    "engine:c:aux_klib": "xdk",
}


# Deps of the engine libraries `engine_lib` resolved while modelling (the generic ones
# have no ENGINE_LIB_DEPS row).
_ENGINE_DEPS_SEEN: dict[str, list[str]] = {}


def _engine_deps(name: str) -> list[str]:
    if name in ENGINE_LIB_DEPS:
        return ENGINE_LIB_DEPS[name].split()
    return _ENGINE_DEPS_SEEN.get(name, [])


# ---------------------------------------------------------------- the model


def _placeholder(path: Path) -> bool:
    """A `rust_cohab_stub.c` that only exists to give cmake a TU."""
    body = re.sub(r"/\*.*?\*/|//[^\n]*", "", path.read_text(errors="replace"), flags=re.S)
    body = re.sub(r"typedef\s+int\s+\w+\s*;", "", body)
    return not body.strip()


def _links_delayimp(static: list[str]) -> bool:
    """Whether a link's static libs (after the ENGINE_LIBS mapping) carry the delayimp runtime."""
    return any(
        re.search(r"/libdelayimp(_rust)?\.a$|^engine:delayimp(_rust)?$", lib) for lib in static
    )


def _placeholder_archive(arch: str, lib: str, repo: Path) -> bool:
    """A cmake static library whose every compiled source is a placeholder stub."""
    m = re.fullmatch(r"(?:.*/)?lib(\w+)\.a", lib)
    if not m:
        return False
    marker = f"CMakeFiles/{m.group(1)}.dir/"
    files = [
        e["file"]
        for e in _ccdb(repo / f"build-{arch}/compile_commands.json")
        if marker in e["command"]
    ]
    return bool(files) and all(
        f.endswith(".c") and _placeholder(Path(f.replace("/tmp/vibeos/", f"{repo}/", 1)))
        for f in files
    )


def model(target: str, repo: Path = REPO) -> Cell:
    links = {a: link_edge(a, target, repo) for a in ARCHES}
    first = links["amd64"]
    cmake_dir = repo / Path(first.output).parent
    cmake = cmake_dir / "CMakeLists.txt"
    if not cmake.exists():
        raise GenError(f"{target}: no {cmake.relative_to(repo)}")
    facts = cmake_facts(cmake, target, first.libs)
    if facts["unknown"]:
        raise GenError(f"{target}: unmodelled statements on the target: {facts['unknown']}")
    if not facts["body"]:
        raise GenError(f"{target}: no rust_staticlib_variant names its body crate")
    if facts.get("ssp") and any(
        re.search(
            r"^STACK_PROTECTOR:BOOL=(ON|TRUE|1)$", _read(repo / f"build-{a}/CMakeCache.txt"), re.M
        )
        for a in ARCHES
    ):
        raise GenError(f"{target}: STACK_PROTECTOR is on and links gcc_ssp objects")
    if first.kind == "C_EXECUTABLE":
        # A native app (autochk, usetup: set_module_type nativecui) links exactly like a
        # Win32 exe but for its subsystem: user-mode Rust, --gc-sections, auto image base.
        if not (first.subsystem or "").startswith(("windows", "console", "native")):
            raise GenError(f"{target}: {first.subsystem} exe is not modelled")
        module_kind = "Win32Exe"
    elif first.kind in ("C_MODULE_LIBRARY", "C_SHARED_LIBRARY"):
        # A kernel-mode driver or a win32k font driver links native:, with a kernel
        # target and no user CRT: that is KernelDll, which the generator does not model.
        if (first.subsystem or "").startswith("native"):
            # A kernel-mode driver: KernelDll, with set_module_type's post-link.
            if facts.get("module_type") not in KERNEL_KINDS:
                raise GenError(
                    f"{target}: kernel-mode module type {facts.get('module_type')} is not modelled"
                )
            module_kind = "KernelDll"
        else:
            module_kind = "Win32Dll"
    else:
        raise GenError(f"{target}: link kind {first.kind}")
    specs = spec_index(repo)
    custom = facts.get("custom_body")
    body_ar = custom["archive"] if custom else f"lib{facts['body_var']}.a"
    if custom and module_kind != "KernelDll":
        raise GenError(f"{target}: a custom-command body outside a kernel-mode driver")
    per_arch: dict[str, tuple] = {}
    late: dict[str, list[str]] = {a: [] for a in ARCHES}
    # cmake's order of the body, import libs and static libs (Linker::link_order): the
    # first library on the command line to define a symbol wins, and an import lib
    # defines what an archive may define too. A repeated library adds nothing.
    orders: dict[str, list[str]] = {a: [] for a in ARCHES}
    for a in ARCHES:
        imports, delayed, static, msvcrtex = [], [], [], None
        order: list[tuple[str, bool]] = []
        after_body = False
        seq = orders[a]
        for lib in links[a].libs:
            name = lib.split("/")[-1]
            # Only the module's OWN body is the cell's rust_staticlib; another crate's
            # `_rust.a` (subst's libconutils_rust.a) is a library like any other.
            if name == body_ar:
                after_body = True
                if "body" not in seq:
                    seq.append("body")
                continue
            if "libclang_rt" in lib:
                continue
            m = re.fullmatch(r"libmsvcrtex_(\w+)\.a", name)
            if m:
                if m.group(1) in MSVCRTEX:
                    msvcrtex = MSVCRTEX[m.group(1)]
                continue
            # add_delay_importlibs: `lib<x>_delayed.a`, which gen-delaylib.py made
            # from the same implib .def the engine generates.
            m = re.search(r"CMakeFiles/lib(\w+)_delayed\.dir/lib\1_delayed\.a$", lib)
            if m:
                base = m.group(1).lower()
                if not specs.get(base):
                    raise GenError(f"{target}: delay import {base}: no single spec2def --implib")
                if (specs[base][0], specs[base][1], base) not in delayed:
                    delayed.append((specs[base][0], specs[base][1], base))
                    order.append((base, True))
                if f"import:{base}" in seq:
                    raise GenError(f"{target}: {base} is both a delay and a regular import")
                seq.append(f"import:{base}")
                continue
            m = re.search(r"CMakeFiles/lib(\w+)\.dir/lib\1\.a$", lib)
            if m:
                base = m.group(1).lower()
                if not specs.get(base):
                    raise GenError(f"{target}: import lib {base}: no single spec2def --implib")
                if (specs[base][0], specs[base][1], base) not in imports:
                    imports.append((specs[base][0], specs[base][1], base))
                    order.append((base, False))
                    seq.append(f"import:{base}")
                continue
            if lib not in ENGINE_LIBS and _placeholder_archive(a, lib, repo):
                # cmake's user32_vista_static holds one rust_cohab_stub.c and nothing
                # else (the body it carries is linked as the body), inflib one empty
                # inflib_rust_stub.c beside libinflib_rust.a: they add no code.
                continue
            mapped = engine_lib(lib, repo)
            if mapped:
                lib = mapped[0]
                _ENGINE_DEPS_SEEN[lib] = mapped[1]
            if lib not in static:
                static.append(lib)
                seq.append(lib)
                if after_body:
                    late[a].append(lib)
        if not after_body:
            raise GenError(f"{target}: {a} link has no {body_ar}")
        # Matched AFTER the ENGINE_LIBS mapping: once the engine built delayimp_rust itself,
        # the cmake path never reached this list and every new delay-importing cell (cscript)
        # was refused for a runtime it was linking all along.
        if delayed and not _links_delayimp(static):
            raise GenError(f"{target}: delay imports without the delayimp runtime")
        per_arch[a] = (tuple(imports), tuple(static), msvcrtex, tuple(delayed), tuple(order))
    if len({(v[0], v[2], v[3], v[4]) for v in per_arch.values()}) > 1:
        raise GenError(f"{target}: imports or CRT differ by arch (not modelled)")

    imports, _, msvcrtex, delayed, order = per_arch["amd64"]
    static_by_arch = {a: list(v[1]) for a, v in per_arch.items()}
    shims, rc, rc_defines, spec, stubs = [], None, [], None, False
    rc_obj = ""
    generated: list[str] = []
    foreign: list[str] = []
    # The inputs are read from the amd64 link; an `if(ARCH ...)` that adds a file on some
    # arches (acgenral's msys2.c on amd64, opengl32's i386 asm trampolines) is a shim of
    # those arches only. Every other input must agree.
    by_arch = {a: [o.split("/")[-1] for o in links[a].inputs] for a in ARCHES}
    common = set.intersection(*(set(v) for v in by_arch.values()))
    arch_shims: dict[str, list[str]] = {a: [] for a in ARCHES}
    for a in ARCHES:
        for obj in links[a].inputs:
            nm = obj.split("/")[-1]
            if nm in common:
                continue
            src = cmake_dir / nm.removesuffix(".obj")
            if not re.search(r"\.(c|s|S)\.obj$", nm) or not src.exists():
                odd = {x: sorted(set(v) - common) for x, v in by_arch.items()}
                raise GenError(f"{target}: link inputs differ by arch {odd}")
            arch_shims[a].append(src.relative_to(repo).as_posix())
    for obj in first.inputs:
        nm = obj.split("/")[-1]
        if nm not in common:
            continue
        if nm.endswith(".rc.res"):
            rc, _defs = _rc_command("amd64", obj, repo)
            rc_obj = obj
        elif nm.endswith("_stubs.rs.obj") or nm.endswith("_stubs.c.obj"):
            stubs = True
        elif nm.endswith(".def"):
            m = re.search(r"spec2def\(\s*(\S+)\s+(\S+\.spec)", cmake.read_text())
            if not m:
                raise GenError(f"{target}: a .def without a spec2def()")
            spec = ((cmake_dir / m.group(2)).relative_to(repo).as_posix(), m.group(1))
        elif nm.endswith(".c.obj"):
            src = cmake_dir / nm.removesuffix(".obj")
            # A source from outside the module's directory: CMake spells each `..` of its
            # path as `__` under `<target>.dir/` (cscript compiles ../wscript's varargs shim).
            m = re.search(r"\.dir/((?:__/)+.*)\.obj$", obj)
            if m:
                src = (cmake_dir / m.group(1).replace("__/", "../")).resolve()
                foreign.append(src.relative_to(repo).as_posix())
            if not src.exists():
                # Generated C: widl's RPC stubs (srvsvc_s.c, from add_rpc_files) and
                # flex output live in the build dir. The shim is named build-dir
                # relative, which c_from_compile_db resolves there, and cmake still
                # generates it (as a source of the <m>_shims OBJECT library).
                rel = cmake_dir.relative_to(repo) / nm.removesuffix(".obj")
                if not all((repo / f"build-{a}" / rel).exists() for a in ARCHES):
                    raise GenError(f"{target}: compiled C {nm} found in no source or build dir")
                shims.append(rel.as_posix())
                generated.append(rel.as_posix())
                continue
            if nm == "rust_cohab_stub.c.obj" and _placeholder(src):
                continue
            shims.append(src.relative_to(repo).as_posix())
        else:
            raise GenError(f"{target}: unmodelled link input {nm}")
    # set_module_type(... UNICODE) is the one target-level define a user module's
    # compiles get; the OBJECT libraries that replace the target must carry it.
    rc_defines = ["UNICODE", "_UNICODE"] if facts["unicode"] else []
    native = (first.subsystem or "").startswith("native")
    if module_kind == "Win32Exe" and not msvcrtex and not native:
        # (A native app has no CRT: NtProcessStartup comes from libnt.a.)
        raise GenError(f"{target}: exe without a msvcrtex CRT startup")
    stacks = {links[a].stack for a in ARCHES}
    if len(stacks) > 1:
        raise GenError(f"{target}: stack reserve differs by arch {stacks}")
    # The CMakeLists' add_cd_file statements are read without their if(ARCH ...)
    # context: the CD list each arch's build wrote must be exactly what they say.
    var = r"\\\$\\\{\w+\\\}"  # a ${VAR} once re.escape()d
    pats = [re.compile(re.sub(var, "[^/]+", re.escape(d))) for d in facts["destinations"]]
    shipped: dict[str, list[str]] = {}
    for a in ARCHES:
        dirs = cd_dirs(a, links[a].output, repo)
        if not facts["destinations"]:
            # Boot CD only (usetup), or on no CD at all (storahci): nothing on the live
            # CD. --check compares every CD placement against the baseline.
            if dirs:
                raise GenError(f"{target}: on the live CD at {sorted(dirs)} with no FOR all/livecd")
            shipped[a] = []
            continue
        hits = [
            [d for d, p in zip(facts["destinations"], pats, strict=True) if p.fullmatch(x)]
            for x in dirs
        ]
        if not dirs or any(len(h) != 1 for h in hits):
            raise GenError(
                f"{target}: CD destinations differ by arch ({a} ships to {sorted(dirs)},"
                f" the CMakeLists names {facts['destinations']})"
            )
        shipped[a] = sorted({h[0] for h in hits}, key=facts["destinations"].index)
    # winprint ships to one prtprocs dir per arch (an if(ARCH) chain of add_cd_file):
    # each arch's own list, as long as together they are exactly what the CMakeLists names.
    per_arch = {}
    if any(shipped[a] != facts["destinations"] for a in ARCHES):
        if {d for a in ARCHES for d in shipped[a]} != set(facts["destinations"]):
            raise GenError(
                f"{target}: CD destinations {shipped} leave some of {facts['destinations']} unused"
            )
        per_arch = shipped
    body, feats, defaults = facts["body"]
    subs = {a: links[a].subsystem or "" for a in ARCHES}
    if len(set(subs.values())) > 1 and not all(subs.values()):
        raise GenError(f"{target}: subsystem set on some arches only {subs}")
    panics = {a: carries_panic_handler(repo / f"build-{a}" / links[a].output) for a in ARCHES}
    for a in ARCHES:
        bound = _exports_bound_to_imports(repo / f"build-{a}" / links[a].output)
        if bound and not bound <= REBOUND_EXPORTS.get(target, set()):
            # user32_vista: cmake's link resolved its exports to user32's import thunks
            # before it reached the body, so the Rust is never linked and the DLL only
            # forwards. A cell links the body first, which is a different module.
            raise GenError(f"{target}: {a} exports resolve to import thunks {sorted(bound)[:4]}")
    if module_kind == "KernelDll":
        # The cell's handler is panic_common's kernel one (KeBugCheckEx) where cmake's
        # link pulled the body's handler in, else panic-lean's fastfail trap, which
        # references nothing: the archive member holding the body's handler was never
        # pulled because nothing in the driver can panic.
        if len(set(panics.values())) > 1:
            raise GenError(f"{target}: the body's panic handler on some arches only {panics}")
        if any(links[a].image_base is None for a in ARCHES):
            raise GenError(f"{target}: a kernel-mode link without --image-base")
    return Cell(
        target=target,
        cmake_dir=cmake_dir,
        output_name=Path(first.output).name,
        module_kind=module_kind,
        entries={a: links[a].entry for a in ARCHES},
        subsystem=first.subsystem,
        image_base={a: links[a].image_base for a in ARCHES},
        stack=stacks.pop(),
        body_crate=body,
        body_features=feats,
        body_defaults=defaults,
        body_rustflags=facts["body_rustflags"],
        body_env_block=facts["body_env_block"],
        import_libs=list(imports),
        delay_import_libs=list(delayed),
        generated=generated,
        import_order=list(order) if delayed else [],
        msvcrtex=msvcrtex,
        static_libs=static_by_arch,
        spec=spec,
        spec_stubs=stubs,
        c_shims=shims,
        foreign=foreign,
        rc=rc,
        rc_defines=rc_defines,
        rc_flags=rc_flags_for(rc, rc_obj, repo) if rc else {},
        destinations=facts["destinations"],
        arch_destinations=per_arch,
        name_on_cd=facts["name_on_cd"],
        extra_cd=facts["extra_cd"],
        arch_shims=arch_shims if any(arch_shims.values()) else {},
        subsystems=subs if len(set(subs.values())) > 1 else {},
        panics=panics,
        body_var=facts["body_var"],
        late_libs=late,
        # REBOUND_EXPORTS: the body ahead of the import libs, so a name both exported
        # and imported binds to the body's definition, not the import thunk.
        link_order=(
            {a: ["body"] + [x for x in o if x != "body"] for a, o in orders.items()}
            if target in REBOUND_EXPORTS
            else orders
        ),
        no_cab=facts["no_cab"],
        kernel_kind=facts.get("module_type", "") if module_kind == "KernelDll" else "",
        custom_body=custom or {},
    )


# ---------------------------------------------------------------- emit


def _rs_list(items: tuple[str, ...] | list[str], indent: int) -> str:
    one = "&[" + ", ".join(f'"{x}"' for x in items) + "]"
    if indent + len(one) < 92:
        return one
    pad = " " * (indent + 4)
    return "&[\n" + "".join(f'{pad}"{x}",\n' for x in items) + " " * indent + "]"


def rc_binding(flags: dict) -> list[str]:
    """The `rc_includes`/`rc_flags`/`rc_defines` a cell passes to Linker::rc."""
    L = [
        "    // The module's part of cmake's windres command; reactos-link adds what every",
        "    // module shares from reactos-link/cflags/rc.<arch>.flags (Linker::rc).",
    ]
    for name in ("includes", "flags", "defines"):
        vals = {a: getattr(flags[a], name) for a in ARCHES}
        if len(set(vals.values())) == 1:
            L.append(f"    let rc_{name}: &[&str] = {_rs_list(vals['amd64'], 4)};")
            continue
        L.append(f"    let rc_{name}: &[&str] = match arch.as_str() {{")
        for a in ARCHES:
            L.append(f'        "{TRIPLE[a]}" => {_rs_list(vals[a], 8)},')
        L += ["        _ => unreachable!(),", "    };"]
    L.append("")
    return L


def rc_call(rc: str) -> str:
    return f'        .rc("{rc}", rc_includes, rc_flags, rc_defines)'


def build_rs(cell: Cell) -> str:
    L = [
        f"//! Cargo-driven `{cell.output_name}` link, generated by tools/cargo_cell_gen.py",
        "//! from the cmake link it replaces.",
        f"//! The body is {cell.body_crate}; reactos-link builds the PE.",
        "",
        "use reactos_link::{Linker, ModuleKind"
        + (", MsvcrtexKind" if cell.msvcrtex else "")
        + "};",
        "use std::env;",
        "",
        "fn main() {",
        '    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH");',
    ]
    dll = cell.module_kind == "Win32Dll"
    kernel = cell.module_kind == "KernelDll"
    L.append(
        "    let (target_json, entry"
        + (", image_base" if dll or kernel else "")
        + ") = match arch.as_str() {"
    )
    for a in ARCHES:
        base = cell.image_base[a]
        tail = ""
        if dll:
            # None: cmake passed no --image-base, so lld's auto base applies.
            tail = f", Some({base:#x}u64)" if base is not None else ", None"
        if kernel:
            tail = f", {base:#x}u64"
        # A kernel cell with no i686-kernel.json builds on the builtin triple on i386
        # too (add_cargo_module), like its body (rust_user_target below): one `core`.
        tgt = TARGET[a]
        L.append(f'        "{TRIPLE[a]}" => ("{tgt}", "{cell.entries[a]}"{tail}),')
    L.append(f'        o => panic!("{cell.crate}: unsupported arch {{o}}"),')
    L.append("    };")
    L.append("")
    if cell.subsystems:
        L.append("    // The arches link different subsystem versions, as cmake's links did.")
        L.append("    let subsystem = match arch.as_str() {")
        for a in ARCHES:
            L.append(f'        "{TRIPLE[a]}" => "{cell.subsystems[a]}",')
        L.append("        _ => unreachable!(),")
        L.append("    };")
        L.append("")
    late = {a: cell.late_libs.get(a, []) for a in ARCHES}
    early = {a: [x for x in cell.static_libs[a] if x not in late[a]] for a in ARCHES}
    tables = (("static_libs", early), ("late_libs", late))
    if cell.link_order:
        tables = (("link_order", cell.link_order),)
        early = late = {a: [] for a in ARCHES}
    for var, table in tables:
        if not any(table.values()):
            continue
        L.append(f"    let {var}: &[&str] = match arch.as_str() {{")
        for a in ARCHES:
            libs = ", ".join(f'"{x}"' for x in table[a])
            L.append(f'        "{TRIPLE[a]}" => &[{libs}],')
        L.append("        _ => unreachable!(),")
        L.append("    };")
        L.append("")
    if cell.rc:
        L += rc_binding(cell.rc_flags)
    if cell.body_rustflags:
        L.append(
            "    // cmake built the body with these RUSTFLAGS (its rust_staticlib_variant ENV)."
        )
        L.append("    let body_rustflags = match arch.as_str() {")
        for a in ARCHES:
            if a in cell.body_rustflags:
                L.append(f'        "{TRIPLE[a]}" => Some("{cell.body_rustflags[a]}"),')
        L.append("        _ => None,")
        L.append("    };")
        L.append("    let mut link = Linker::new(target_json);")
        L.append("    if let Some(flags) = body_rustflags {")
        L.append("        link = link.rust_build_rustflags(flags);")
        L.append("    }")
        L.append("    link")
    else:
        L.append("    Linker::new(target_json)")
    if cell.body_features or not cell.body_defaults:
        feats = ", ".join(f'"{f}"' for f in cell.body_features)
        L.append(
            f'        .rust_lib("{cell.body_crate}", &[{feats}], {str(cell.body_defaults).lower()})'
        )
    else:
        L.append(f'        .rust_staticlib("{cell.body_crate}")')
    if kernel:
        # cmake's custom command built it on the builtin triple, not the kernel json.
        L.append("        .rust_user_target()")
    if cell.arch_shims:
        # cmake compiles some of it on some arches only.
        L.append("        .c_from_compile_db(")
        L.append(f'            "{cell.target}_shims",')
        L.append("            match arch.as_str() {")
        for a in ARCHES:
            L.append(
                f'                "{TRIPLE[a]}" => &['
                + ", ".join(f'"{x}"' for x in cell.c_shims + cell.arch_shims[a])
                + "],"
            )
        L.append("                _ => unreachable!(),")
        L.append("            },")
        L.append("        )")
    elif cell.c_shims:
        # Replays cmake's own compile of each file (the <m>_shims OBJECT library; `<m>_c`
        # is taken: c_oracle.cmake builds the pre-flip C under that name), so the C keeps
        # the exact flags it had. Compiled with the engine's c_shims flags instead,
        # expand's varargs shim grew an arm64 __chkstk import.
        L.append(
            f'        .c_from_compile_db("{cell.target}_shims", &['
            + ", ".join(f'"{s}"' for s in cell.c_shims)
            + "])"
        )
    if cell.rc:
        L.append(rc_call(cell.rc))
    if cell.spec:
        L.append(f'        .spec2def("{cell.spec[0]}", "{cell.spec[1]}")')
    if cell.spec_stubs:
        L.append("        .spec_stubs()")
    regular = {lib[2]: lib for lib in cell.import_libs}
    delayed = {lib[2]: lib for lib in cell.delay_import_libs}
    order = cell.import_order or [(b, False) for b in regular] + [(b, True) for b in delayed]
    for base, delay in order:
        spec, dllname, _ = (delayed if delay else regular)[base]
        method = "delay_import_lib" if delay else "import_lib"
        L.append(f'        .{method}("{spec}", "{dllname}", "{base}")')
    if any(early.values()):
        L.append("        // cmake-built static libraries, reused until each has a cargo home.")
        L.append("        .static_libs(static_libs)")
    if any(late.values()):
        L.append("        // ...and those cmake linked after the body, in the same order.")
        L.append("        .static_libs_after_body(late_libs)")
    if cell.link_order:
        L.append(
            "        // The body, import libs and static libs in cmake's link order: the first"
        )
        L.append("        // to define a symbol wins. cmake-built ones are reused until each has a")
        L.append("        // cargo home; `engine:` ones reactos-link builds.")
        L.append("        .link_order(link_order)")
    if kernel:
        L.append("        // cmake's kernel-mode links keep every section.")
        L.append("        .keep_all_sections()")
    if cell.msvcrtex:
        L.append(f"        .msvcrtex_crt(MsvcrtexKind::{cell.msvcrtex})")
    if cell.stack:
        L.append(f"        .stack_reserve({cell.stack})")
    if kernel:
        L.append("        .link(ModuleKind::KernelDll {")
        L.append("            entry: entry.into(),")
        L.append("            image_base,")
        L.append("        });")
    elif dll:
        sub = f'Some("{cell.subsystem}".into())' if cell.subsystem else "None"
        sub = "Some(subsystem.into())" if cell.subsystems else sub
        L.append("        .link(ModuleKind::Win32Dll {")
        L.append("            entry: entry.into(),")
        L.append("            image_base,")
        L.append(f"            subsystem: {sub},")
        L.append("        });")
    else:
        L.append("        .link(ModuleKind::Win32Exe {")
        L.append("            entry: entry.into(),")
        sub = "subsystem" if cell.subsystems else f'"{cell.subsystem}"'
        L.append(f"            subsystem: {sub}.into(),")
        L.append("        });")
    L.append("}")
    return "\n".join(L) + "\n"


CFG_ARCH = {"i386": "x86", "amd64": "x86_64", "arm64": "aarch64"}


def _exports_bound_to_imports(pe_path: Path) -> set[str]:
    """Names the cmake-built module both exports and imports."""
    import pe_equiv

    pe = pe_equiv.load(pe_path)
    exported = {e.split(" ", 1)[1].split(" ", 1)[0] for e in pe.exports()}
    imported = {i.split("!", 1)[1] for i in pe.imports()}
    return exported & imported


def carries_panic_handler(pe_path: Path) -> bool:
    """Did this (cmake-built) module link a Rust panic handler? panic_common's user
    handler is the only thing in these modules that imports RtlWalkFrameChain."""
    import pe_equiv

    if not pe_path.exists():
        raise GenError(f"{pe_path}: build the module with cmake first (it decides the handler)")
    return any(i.endswith("!RtlWalkFrameChain") for i in pe_equiv.load(pe_path).imports())


def _cfg(arches: list[str]) -> str:
    preds = [f'target_arch = "{CFG_ARCH[a]}"' for a in arches]
    return preds[0] if len(preds) == 1 else f"any({', '.join(preds)})"


def cargo_toml(cell: Cell) -> str:
    bin_name = cell.crate.removesuffix("-cargo")  # add_cargo_module derives it so
    return f"""# Cargo-driven `{cell.output_name}` link, generated by tools/cargo_cell_gen.py.

[package]
name = "{cell.crate}"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "{bin_name}"
path = "src/main.rs"

{_panic_deps(cell)}[build-dependencies]
reactos-link = {{ path = "../reactos-link" }}
"""


def _panic_deps(cell: Cell) -> str:
    if cell.module_kind == "KernelDll" and all(cell.panics.values()):
        return (
            '[dependencies]\npanic-common = { path = "../panic-common", features = ["kernel"] }\n'
        )
    if cell.module_kind == "KernelDll":
        return '[dependencies]\npanic-lean = { path = "../panic-lean" }\n'
    return _user_panic_deps(cell)


def _user_panic_deps(cell: Cell) -> str:
    full = [a for a in ARCHES if cell.panics[a]]
    common = 'panic-common = { path = "../panic-common", features = ["user"] }\n'
    lean = 'panic-lean = { path = "../panic-lean" }\n'
    if len(full) == len(ARCHES):
        return "[dependencies]\n" + common
    out = "[dependencies]\n" + lean
    if full:
        # Arch-scoped, never unused: a linked panic_common loads core on its own.
        out += f"[target.'cfg({_cfg(full)})'.dependencies]\n" + common
    return out


def main_rs(cell: Cell) -> str:
    full = [a for a in ARCHES if cell.panics[a]]
    if cell.module_kind == "KernelDll" and not full:
        return MAIN_RS_HEAD + "    let _ = info;\n    panic_lean::panic_fastfail()\n}\n"
    head = MAIN_RS_HEAD
    common = '    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)\n'
    lean = f'    panic_lean::panic_lean(b"{cell.crate}\\0", info)\n'
    if len(full) == len(ARCHES):
        return head + common + "}\n"
    if not full:
        return head + lean + "}\n"
    return (
        head
        + f"    #[cfg({_cfg(full)})]\n"
        + common.replace("    panic_common", "    return panic_common")[:-1]
        + ";\n"
        + f"    #[cfg(not({_cfg(full)}))]\n"
        + lean.replace("    panic_lean", "    return panic_lean")[:-1]
        + ";\n}\n"
    )


MAIN_RS_HEAD = """\
//! Empty bin shell: the PE's entry (the CRT startup, DllMain, DriverEntry) is in the
//! linked body, and nothing here runs. Cargo needs a `main.rs` to drive the link.
//!
//! The handler is panic_common only where cmake's link carried the body's handler;
//! elsewhere panic-lean, which keeps all of `core` out of a module that cannot panic.

#![no_std]
#![no_main]
#![allow(internal_features)]

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
"""


def apply(cell: Cell, repo: Path = REPO) -> list[Path]:
    crate = repo / "sdk/lib/rust" / cell.crate
    if crate.exists():
        raise GenError(f"{crate.relative_to(repo)} already exists")
    if not cell.body_crate or not (repo / "sdk/lib/rust" / cell.body_crate).is_dir():
        raise GenError(f"{cell.target}: body crate {cell.body_crate} not in the workspace")
    # Everything is computed before anything is written: a refusal leaves no half-cell.
    new_cmake = rewrite_cmake(cell.cmake_dir / "CMakeLists.txt", cell)
    rs, toml = build_rs(cell), cargo_toml(cell)
    (crate / "src").mkdir(parents=True)
    (crate / "Cargo.toml").write_text(toml)
    (crate / "build.rs").write_text(rs)
    (crate / "src/main.rs").write_text(main_rs(cell))
    ws = repo / "sdk/lib/rust/Cargo.toml"
    text = ws.read_text()
    member = f'    "{cell.crate}",\n'
    if member not in text:
        text = re.sub(r"(members = \[\n)", rf"\g<1>{member}", text, count=1)
        ws.write_text(text)
    (cell.cmake_dir / "CMakeLists.txt").write_text(new_cmake)
    return [crate, ws, cell.cmake_dir / "CMakeLists.txt"]


def survey(repo: Path = REPO) -> list[tuple[str, str]]:
    """(target, verdict) for every link edge whose only compiled C is a placeholder stub."""
    ninja = _read(repo / "build-amd64/build.ninja")
    out = []
    for m in re.finditer(
        r"^build \S+: C_\w+?_LINKER__(\w+?)_(?:Debug|Release)\S* ([^|\n]*)", ninja, re.M
    ):
        target, ins = m.group(1), m.group(2).split()
        objs = [i.split("/")[-1] for i in ins]
        if "rust_cohab_stub.c.obj" not in objs:
            continue
        try:
            cell = model(target, repo)
            out.append((target, f"OK {cell.module_kind}"))
        except (GenError, OSError) as exc:
            out.append((target, f"REFUSED {exc}"))
    return sorted(out)


def cell_crate(target: str, repo: Path = REPO) -> str:
    """The cargo package of an applied cell: what its add_cargo_module names."""
    if not target[:1].isdigit():
        return f"{target}-cargo"
    for cm in repo.glob("**/CMakeLists.txt"):
        if cm.parts[len(repo.parts)].startswith(("build-", ".")):
            continue
        m = re.search(
            rf"add_cargo_module\({re.escape(target)} (\S+)", cm.read_text(errors="replace")
        )
        if m:
            return m.group(1)
    raise GenError(f"{target}: no add_cargo_module names it")


def _cell_uses_panic_common(target: str, arch: str, repo: Path) -> bool:
    main = repo / "sdk/lib/rust" / cell_crate(target, repo) / "src/main.rs"
    text = main.read_text() if main.exists() else ""
    if "panic_lean" not in text:
        return True
    m = re.search(r"#\[cfg\((.*)\)\]\n\s*return panic_common", text)
    return bool(m) and f'"{CFG_ARCH[arch]}"' in m.group(1)


def _baseline_pe(baseline: Path, arch: str, target: str) -> list[Path]:
    """The saved cmake-built PE(s) for target. The index --save-baseline writes knows
    the output name (cmd_help links help.exe); older baselines fall back to a glob."""
    index = baseline / "targets.json"
    if index.exists():
        out = json.loads(index.read_text()).get(target, {}).get(arch)
        if out:
            return [baseline / arch / out]
    return list((baseline / arch).rglob(f"{target}.*"))


def reshell(targets: list[str], baseline: Path, repo: Path = REPO) -> None:
    """Re-choose an existing cell's panic handler from the cmake-built PEs in `baseline`."""
    for t in targets:
        panics = {}
        for a in ARCHES:
            olds = _baseline_pe(baseline, a, t)
            if len(olds) != 1:
                raise GenError(f"{t} {a}: {len(olds)} baseline candidates")
            panics[a] = carries_panic_handler(olds[0])
        crate = repo / "sdk/lib/rust" / cell_crate(t, repo)
        toml = (crate / "Cargo.toml").read_text()
        head = toml[: toml.index("[dependencies]")]
        cell_like = Cell(**{**_STUB_CELL, "target": t, "panics": panics})
        (crate / "Cargo.toml").write_text(
            head + _panic_deps(cell_like) + toml[toml.index("[build-dependencies]") :]
        )
        (crate / "src/main.rs").write_text(main_rs(cell_like))
        kind = "full" if all(panics.values()) else "lean" if not any(panics.values()) else panics
        print(f"{t:14} {kind}")


# The fields reshell() does not read; main_rs/_panic_deps use only target and panics.
_STUB_CELL: dict = dict(
    cmake_dir=Path("."),
    output_name="",
    module_kind="",
    entries={},
    subsystem=None,
    image_base={},
    stack=None,
    body_crate="",
    body_features=[],
    body_defaults=True,
    import_libs=[],
    msvcrtex=None,
    static_libs={},
    spec=None,
    spec_stubs=False,
    c_shims=[],
    rc=None,
    rc_defines=[],
    destinations=[],
    name_on_cd=None,
)


def check(targets: list[str], baseline: Path, repo: Path = REPO) -> int:
    import pe_equiv

    bad = 0
    for t in targets:
        for a in ARCHES:
            olds = _baseline_pe(baseline, a, t)
            if len(olds) != 1:
                print(f"{t:14} {a:6} NO BASELINE ({len(olds)} candidates)")
                bad += 1
                continue
            new = repo / f"build-{a}" / olds[0].relative_to(baseline / a)
            # The baseline is cmake's PE: when cargo did not relink the module (a failed
            # build leaves cmake's file in place), comparing it with itself is a vacuous
            # pass. It happened: an all-arch CARGO_CHECK_FAIL checked 18/18 EQUIVALENT.
            if not new.exists() or new.read_bytes() == olds[0].read_bytes():
                print(f"{t:14} {a:6} NOT RELINKED (still the baseline PE: did the build fail?)")
                bad += 1
                continue
            try:
                diffs = pe_equiv.compare(pe_equiv.load(olds[0]), pe_equiv.load(new))
            except pe_equiv.PEError as exc:
                print(f"{t:14} {a:6} UNREADABLE {exc}")
                bad += 1
                continue
            # The cargo bin's own #[panic_handler] (panic_common, the same handler the body
            # crate defines) survives a link where cmake's garbage-collected the body's copy.
            # Its imports are the one accepted addition; any other change is a regression.
            extra = {d for d in diffs if d.removeprefix("imports: + ") in PANIC_COMMON_IMPORTS}
            probes = {
                d
                for d in diffs
                if a != "i386" and d.removeprefix("imports: + ") in STACK_PROBE_IMPORTS
            }
            rsbegin = {d for d in diffs if a == "i386" and d in I386_RSBEGIN_SECTIONS}
            if rsbegin and a == "i386":
                data = [x for x in pe_equiv.load(new).sections if x[0] == ".data"]
                rsbegin |= {
                    d
                    for d in diffs
                    if d.startswith("sections: + .data ")
                    and data
                    and data[0][2] == I386_RSBEGIN_DATA
                }
            # A panic-lean kernel cell's own handler is a function with an unwind entry:
            # a module cmake linked with none (watchdog) gains a `.pdata`.
            lean_unwind = {
                d
                for d in diffs
                if a != "i386"
                and d.startswith("sections: + .pdata ")
                and not _cell_uses_panic_common(t, a, repo)
            }
            dead = {d for d in diffs if d.removeprefix("imports: - ") in DEAD_IMPORTS.get(t, ())}
            rebound = {
                d
                for d in diffs
                if d.startswith("imports: - ") and d.split("!")[-1] in REBOUND_EXPORTS.get(t, set())
            }
            dead |= rebound
            # A panic-lean cell links only what its body, CRT and exports reach; cmake's
            # link also kept whatever shared an archive member with the body's panic
            # path. An import can only vanish when no linked code references it, so the
            # removals are that unreachable code. Reported, never silent.
            lean = (
                {d for d in diffs if d.startswith("imports: - ")}
                if not _cell_uses_panic_common(t, a, repo)
                else set()
            ) - dead
            diffs = [
                d for d in diffs if d not in extra | rsbegin | probes | dead | lean | lean_unwind
            ]
            # Size is not part of the contract, but a cell must not carry code cmake's
            # link did not: that is how all of core rode into 94 modules unnoticed.
            grew = pe_equiv.load(new).mapped_size() - pe_equiv.load(olds[0]).mapped_size()
            if grew > SIZE_SLACK:
                diffs.append(
                    f"size: mapped image grew {grew // 1024} KB (slack {SIZE_SLACK // 1024})"
                )
            # REBOUND_EXPORTS: cmake's module never linked the body, so the cell is a
            # different (fixed) module. What linking the body brings is accepted and
            # reported: imports it calls, sections it has, the size it adds. Losing an
            # export, a resource or a CD placement still fails.
            rebound_diffs = set()
            if t in REBOUND_EXPORTS:
                rebound_diffs = {
                    d
                    for d in diffs
                    if d.startswith(("imports: + ", "sections: + ", "size: mapped image grew"))
                }
                diffs = [d for d in diffs if d not in rebound_diffs]
            # Where the CDs put it. cmake's `FOR all` is the live CD plus reactos.cab; the
            # cell's add_cargo_module also lays the file loose on the boot CD, which is the
            # one accepted addition. Anything removed is a file some CD or install lost.
            cd_old = json.loads((baseline / "targets.json").read_text()).get(t, {}).get(f"cd:{a}")
            loose = set()
            note_txtsetup = False
            if cd_old is not None:
                cd_new = set(cd_placement(a, str(new.relative_to(repo / f"build-{a}"))))
                gone, added = set(cd_old) - cd_new, cd_new - set(cd_old)
                loose = {x for x in added if x.startswith(("bootcd:", "bootcdregtest:"))}
                # A file txtsetup.sif installs (a boot driver) stays out of the cabinet,
                # where cmake's add_cd_file put a second copy that could not overwrite it.
                if new.name.lower() in txtsetup_files(repo):
                    by_txtsetup = {x for x in gone if x.startswith("cab:")}
                    gone -= by_txtsetup
                    note_txtsetup = bool(by_txtsetup)
                diffs += [f"cd: - {x}" for x in sorted(gone)]
                diffs += [f"cd: + {x}" for x in sorted(added - loose)]
            note = " (+ panic_common imports)" if extra else ""
            note += (
                f" (rebound exports: linked the body; {len(rebound_diffs)} expected differences: "
                + "; ".join(sorted(rebound_diffs))
                + ")"
                if rebound_diffs
                else ""
            )
            note += " (+ loose on the boot CD)" if loose else ""
            note += " (- cab: txtsetup.sif installs it)" if note_txtsetup else ""
            note += " (+ rsbegin .ctors/.dtors)" if rsbegin else ""
            note += " (+ the lean handler's .pdata)" if lean_unwind else ""
            note += " (+ __chkstk: frame pointers)" if probes else ""
            note += " (- dead-code import, see DEAD_IMPORTS)" if dead else ""
            note += f" (- {len(lean)} imports only the panic path reached)" if lean else ""
            print(f"{t:14} {a:6} {'EQUIVALENT' + note if not diffs else 'DIFFERENT'}")
            for d in diffs[:12]:
                print(f"    {d}")
            bad += bool(diffs)
    print(f"\n{bad} problem(s)")
    return 1 if bad else 0


def remap(crates: list[str] | None = None, repo: Path = REPO) -> list[str]:
    """Move existing cells off cmake-built archives the engine can now build itself.

    A cell's `static_libs` may still name a raw `<dir>/lib<x>.a` that cmake builds (the
    generator only knew ENGINE_LIBS when it ran). For each one `engine_lib` resolves, this
    rewrites the build.rs entry to the engine name, drops the archive from the cell's
    `<T>_REUSED` list in its CMakeLists (and the variable, once empty) and adds the engine
    library's order-only targets to EXTRA_DEPENDS. Returns the crates it changed.
    """
    changed = []
    for build_rs_path in sorted((repo / "sdk/lib/rust").glob("*-cargo/build.rs")):
        crate = build_rs_path.parent.name
        if crates and crate not in crates:
            continue
        text = build_rs_path.read_text()
        raw = sorted(set(re.findall(r'"([^"]+\.a)"', re.sub(r"//[^\n]*", "", text))))
        mapping = {lib: engine_lib(lib, repo) for lib in raw}
        mapping = {lib: m for lib, m in mapping.items() if m}
        if not mapping:
            continue
        cm = next(
            (
                c
                for c in repo.glob("**/CMakeLists.txt")
                if not c.parts[len(repo.parts)].startswith(("build-", "."))
                and re.search(
                    rf"add_cargo_module\(\S+ {re.escape(crate)}\b", c.read_text(errors="replace")
                )
            ),
            None,
        )
        if cm is None:
            # The loader cells (freeldr, rosload, uefildr) are hand-made recipes, not
            # add_cargo_module cells; their archives are the loader rung's.
            print(f"skipped {crate}: not an add_cargo_module cell")
            continue
        for lib, (name, _deps) in mapping.items():
            text = text.replace(f'"{lib}"', f'"{name}"')
        cml = cm.read_text()
        for lib in mapping:
            cml = re.sub(
                rf"(?m)^[ \t]*(?:list\(APPEND \w+ )?\$\{{CMAKE_BINARY_DIR\}}/{re.escape(lib)}\)?\n",
                lambda m: (
                    ")\n"
                    if m.group(0).rstrip().endswith(")") and "list(APPEND" not in m.group(0)
                    else ""
                ),
                cml,
            )
        # An emptied `set(X_REUSED\n)` and the list(APPEND) guards left with nothing in them.
        for var in re.findall(r"set\((\w+_REUSED)\s*\)", cml):
            cml = re.sub(rf"set\({var}\s*\)\n", "", cml)
            cml = cml.replace(f" ${{{var}}}", "")
        cml = re.sub(r'(?m)^if\(ARCH STREQUAL "\w+"\)\nendif\(\)\n', "", cml)
        deps = sorted({d for _n, ds in mapping.values() for d in ds})
        m = re.search(r"(?m)^(\s*EXTRA_DEPENDS)([^)\n]*)\)", cml)
        if deps:
            if m:
                have = m.group(2).split()
                new = have + [d for d in deps if d not in have]
                cml = cml[: m.start()] + f"{m.group(1)} {' '.join(new)})" + cml[m.end() :]
            else:
                raise GenError(f"{crate}: no EXTRA_DEPENDS to add {deps} to")
        build_rs_path.write_text(text)
        cm.write_text(cml)
        changed.append(crate)
    return changed


# ---------------------------------------------------------------- C

# cmake's C rule (the Ninja generator's): <CC> --target=<t> <DEFINES> <INCLUDES> <FLAGS>
# -o <OBJECT> -c <SOURCE>, <DEFINES> sorted. Shared: cflags/c.<arch>.flags (`--c-template`);
# the library's part is Linker::c_compiled's.


def c_template_path(arch: str, repo: Path = REPO) -> Path:
    return repo / f"sdk/lib/rust/reactos-link/cflags/c.{arch}.flags"


@dataclass
class CParts:
    cc: str
    target: str
    defines: list[str]
    includes: list[str]
    flags: list[str]
    obj: str
    src: str


def _c_parts(cmd: str) -> CParts:
    import shlex

    t = shlex.split(cmd)
    if not t[1].startswith("--target="):
        raise GenError(f"not cmake's C rule: {' '.join(t[:3])}")
    i, defines, includes = 2, [], []
    while t[i].startswith("-D"):
        defines.append(t[i])
        i += 1
    while t[i].startswith("-I"):
        if t[i] == "-I":
            raise GenError(f"a separate -I in cmake's C rule: {t[-1]}")
        includes.append(t[i][2:])
        i += 1
    o = t.index("-o", i)
    if t[o + 2] != "-c" or len(t) != o + 4:
        raise GenError(f"C command does not end -o <obj> -c <src>: {t[-1]}")
    if defines != sorted(defines):
        raise GenError(f"{t[-1]}: <DEFINES> not sorted, the rule this models")
    return CParts(t[0], t[1], defines, includes, t[i:o], t[o + 1], t[o + 3])


def _flag_enc(tok: str, arch: str) -> str:
    return tok.replace(f"{CANON}/build-{arch}", "{BUILD}").replace(CANON, "{REPO}")


def _flag_dec(tok: str, arch: str) -> str:
    return tok.replace("{BUILD}", f"{CANON}/build-{arch}").replace("{REPO}", CANON)


@dataclass
class CTemplate:
    cc: str
    target: str
    std: list[str]  # encoded include dirs
    core: list[str]  # encoded flags
    defines: list[str]

    def text(self, arch: str) -> str:
        out = [
            f"# cmake's C rule, the part every C compile in the tree shares ({arch}). Generated",
            "# by python3 tools/cargo_cell_gen.py --c-template from "
            "build-<arch>/compile_commands.json;",
            "# reactos-link's Linker::c_compiled adds the library's own includes, flags and "
            "defines.",
        ]
        for name, toks in (
            ("cc", [self.cc]),
            ("target", [self.target]),
            ("std", self.std),
            ("core", self.core),
            ("defines", self.defines),
        ):
            out.append(f"[{name}]")
            out.extend(toks)
        return "\n".join(out) + "\n"


def _c_commands(arch: str, repo: Path = REPO) -> list[dict]:
    db = _ccdb(repo / f"build-{arch}/compile_commands.json")
    return [e for e in db if e["file"].endswith(".c") and " --target=" in e.get("command", "")]


def _majority_run(lists: list[list[str]], share: float = 0.95) -> list[str]:
    """The longest run of consecutive tokens that `share` of the lists contain."""

    def has(run: list[str], seq: list[str]) -> bool:
        return any(seq[i : i + len(run)] == run for i in range(len(seq) - len(run) + 1))

    first = max(lists, key=len)
    best: list[str] = []
    for start in range(len(first)):
        end = start + len(best) + 1
        if end > len(first) or sum(has(first[start:end], s) for s in lists) < share * len(lists):
            continue
        while end < len(first) and sum(
            has(first[start : end + 1], s) for s in lists
        ) >= share * len(lists):
            end += 1
        best = first[start:end]
    return best


def derive_c_template(arch: str, repo: Path = REPO) -> CTemplate:
    parts = []
    for e in _c_commands(arch, repo):
        try:
            parts.append(_c_parts(e["command"]))
        except GenError:
            continue  # (ole32's per-source defines): encoding refuses it, not the template
    if not parts:
        raise GenError(f"build-{arch}: no C commands")
    ccs = {p.cc for p in parts}
    targets = {p.target for p in parts}
    if len(ccs) != 1 or len(targets) != 1:
        raise GenError(f"build-{arch}: C compiler or --target differs by target")
    common = set(parts[0].defines).intersection(*(set(p.defines) for p in parts[1:]))
    return CTemplate(
        cc=ccs.pop().replace(CANON, "{REPO}"),
        target=targets.pop(),
        std=[_rc_enc(x, arch) for x in _majority_run([p.includes for p in parts])],
        core=[_flag_enc(x, arch) for x in _majority_run([p.flags for p in parts])],
        defines=sorted(common),
    )


def read_c_template(arch: str, repo: Path = REPO) -> CTemplate:
    path = c_template_path(arch, repo)
    if not path.exists():
        raise GenError(f"{path}: missing (python3 tools/cargo_cell_gen.py --c-template)")
    sec: dict[str, list[str]] = {}
    cur = None
    for line in path.read_text().splitlines():
        if not line or line.startswith("#"):
            continue
        if line.startswith("[") and line.endswith("]"):
            cur = sec.setdefault(line[1:-1], [])
        elif cur is not None:
            cur.append(line)
    return CTemplate(
        sec["cc"][0], sec["target"][0], sec["std"], sec["core"], sec.get("defines", [])
    )


@dataclass(frozen=True)
class CFlags:
    """A C library's part of cmake's C rule, as Linker::c_compiled takes it."""

    includes: tuple[str, ...]
    flags: tuple[str, ...]
    defines: tuple[str, ...]


def c_decode(f: CFlags, tmpl: CTemplate, arch: str) -> list[str]:
    """The command Linker::c_compiled builds, up to `-o`."""
    out = [_flag_dec(tmpl.cc, arch), tmpl.target]
    out += sorted(tmpl.defines + [f"-D{d}" for d in f.defines])
    for inc in f.includes:
        out += [f"-I{_rc_dec(d, arch)}" for d in (tmpl.std if inc == "@std" else [inc])]
    for fl in f.flags:
        out += [_flag_dec(x, arch) for x in tmpl.core] if fl == "@core" else [_flag_dec(fl, arch)]
    return out


def _with_marker(seq: list[str], run: list[str], marker: str) -> list[str] | None:
    for i in range(len(seq) - len(run) + 1):
        if seq[i : i + len(run)] == run:
            return [*seq[:i], marker, *seq[i + len(run) :]]
    return None


def c_encode(cmd: str, tmpl: CTemplate, arch: str) -> CFlags:
    """The library's part of cmake's C command; refuses unless c_decode rebuilds it token
    for token."""
    import shlex

    p = _c_parts(cmd)
    inc = [_rc_enc(x, arch) for x in p.includes]
    inc = _with_marker(inc, tmpl.std, "@std") or inc
    flags = [_flag_enc(x, arch) for x in p.flags]
    flags = _with_marker(flags, tmpl.core, "@core") or flags
    left = list(p.defines)
    for d in tmpl.defines:
        if d not in left:
            raise GenError(f"{p.src}: lacks the global define {d}")
        left.remove(d)
    f = CFlags(tuple(inc), tuple(flags), tuple(d[2:] for d in left))
    t = shlex.split(cmd)
    if c_decode(f, tmpl, arch) != t[: t.index("-o")]:
        raise GenError(f"{p.src}: the C command does not rebuild from the template")
    return f


def c_lib_for(target: str, repo: Path = REPO) -> tuple[dict, dict]:
    """Per arch, cmake target `target`'s sources (repo-relative, `@build/` when generated)
    in its compile order, and the one CFlags all of them share."""
    srcs, flags = {}, {}
    for a in ARCHES:
        tmpl = read_c_template(a, repo)
        es = [e for e in _c_commands(a, repo) if f"/CMakeFiles/{target}.dir/" in e["command"]]
        if not es:
            continue
        fs = {c_encode(e["command"], tmpl, a) for e in es}
        if len(fs) != 1:
            raise GenError(
                f"{target}: its sources compile with {len(fs)} different flag sets on {a}"
            )
        flags[a] = fs.pop()
        srcs[a] = [_rc_enc(e["file"], a) for e in es]
    if not flags:
        raise GenError(f"{target}: no C commands on any arch")
    have = next(iter(flags))
    return {a: srcs.get(a, srcs[have]) for a in ARCHES}, {
        a: flags.get(a, flags[have]) for a in ARCHES
    }


RES_CALL = re.compile(r'\.resource_from_rc\("([^"]+)",\s*"([^"]+)"\)')


def _cmakelists(repo: Path) -> list[Path]:
    out = subprocess.run(
        ["git", "-C", str(repo), "ls-files", "*CMakeLists.txt"],
        capture_output=True,
        text=True,
        check=True,
    ).stdout.split()
    return [repo / x for x in out]


def own_rc(crates: list[str] | None = None, repo: Path = REPO) -> list[str]:
    """Move existing cells off cmake's `<m>_res` OBJECT libraries: each
    `.resource_from_rc(rc, "<m>_res")` (which replays the windres command cmake records for
    that library) becomes `.rc(rc, ...)` with the module's part of that command, and a
    `<m>_res` no cell replays any more leaves its CMakeLists. Reads compile_commands.json,
    so it runs on a graph configured from the CMakeLists it rewrites."""
    rust = repo / "sdk/lib/rust"
    dirs = [rust / c for c in crates] if crates else sorted(rust.glob("*-cargo"))
    moved, targets = [], set()
    for d in dirs:
        rs = d / "build.rs"
        text = rs.read_text() if rs.exists() else ""
        calls = RES_CALL.findall(text)
        if not calls:
            continue
        if len(calls) > 1:
            raise GenError(f"{rs}: {len(calls)} resource_from_rc calls")
        rc, res_target = calls[0]
        try:
            flags = rc_flags_for(rc, f"/CMakeFiles/{res_target}.dir/", repo)
        except GenError:
            flags = _rc_flags_by_analogy(rc, repo)
        lines = text.split("\n")
        at = next(i for i, ln in enumerate(lines) if ln.strip().startswith("let arch = env::var("))
        lines[at + 1 : at + 1] = ["", *rc_binding(flags)[:-1]]
        if lines[at + 1 + len(rc_binding(flags))].strip():
            lines.insert(at + 1 + len(rc_binding(flags)), "")
        text = "\n".join(lines)
        text = RES_CALL.sub(f'.rc("{rc}", rc_includes, rc_flags, rc_defines)', text, count=1)
        rs.write_text(text)
        targets.add(res_target)
        moved.append(d.name)
    still = {t for f in rust.glob("*/build.rs") for _rc, t in RES_CALL.findall(f.read_text())}
    # Also the libraries of cells an earlier, interrupted run moved.
    for f in _cmakelists(repo):
        for m in re.finditer(r"EXTRA_DEPENDS ([^)\n]*)", f.read_text()):
            targets |= {x for x in m.group(1).split() if x.endswith("_res")}
    for t in sorted(targets - still):
        _drop_res_library(t, repo)
    return moved


# ---------------------------------------------------------------- asm, and shared C libraries

# cmake's ASM rule: <CC> -x assembler-with-cpp -o <OBJECT> <INCLUDES> <FLAGS> <DEFINES> -c <SOURCE>.
ASM_HEAD = ["-x", "assembler-with-cpp"]
ASM_TAIL = "-D__ASM__"


def asm_template_path(arch: str, repo: Path = REPO) -> Path:
    return repo / f"sdk/lib/rust/reactos-link/cflags/asm.{arch}.flags"


def _asm_parts(cmd: str) -> CParts:
    import shlex

    t = shlex.split(cmd)
    if t[1:3] != ASM_HEAD or t[3] != "-o":
        raise GenError(f"not cmake's ASM rule: {' '.join(t[:4])}")
    obj, i, includes = t[4], 5, []
    while t[i].startswith("-I"):
        includes.append(t[i][2:])
        i += 1
    if t[-2] != "-c":
        raise GenError(f"ASM command does not end -c <src>: {t[-1]}")
    # CMAKE_ASM_COMPILE_OBJECT ends `<DEFINES> -D__ASM__ -c <SOURCE>`.
    end = len(t) - 3 if t[-3] == ASM_TAIL else len(t) - 2
    j = end
    while t[j - 1].startswith("-D"):
        j -= 1
    run = t[j:end]
    # <DEFINES> is the sorted run; -D's a source adds after it (libcntpr's asm: CRTDLL,
    # __CRT__NO_INLINE) keep their place, after the `@defines` marker in the flags.
    k = next((n for n in range(len(run) - 1) if run[n] > run[n + 1]), len(run) - 1) + 1
    defines, post = run[:k], run[k:]
    flags = t[i:j] + (["@defines", *post] if post else [])
    return CParts(t[0], "", defines, includes, flags, obj, t[-1])


def _asm_commands(arch: str, repo: Path = REPO) -> list[dict]:
    db = _ccdb(repo / f"build-{arch}/compile_commands.json")
    return [
        e
        for e in db
        if re.search(r"\.(s|S)$", e["file"]) and " -x assembler-with-cpp " in e.get("command", "")
    ]


def derive_asm_template(arch: str, repo: Path = REPO) -> CTemplate:
    parts = []
    for e in _asm_commands(arch, repo):
        try:
            parts.append(_asm_parts(e["command"]))
        except GenError:
            continue
    if not parts:
        raise GenError(f"build-{arch}: no ASM commands")
    common = set(parts[0].defines).intersection(*(set(p.defines) for p in parts[1:]))
    return CTemplate(
        cc=parts[0].cc.replace(CANON, "{REPO}"),
        target="",
        std=[_rc_enc(x, arch) for x in _majority_run([p.includes for p in parts], 0.6)],
        core=[_flag_enc(x, arch) for x in _majority_run([p.flags for p in parts], 0.6)],
        defines=sorted(common),
    )


def read_asm_template(arch: str, repo: Path = REPO) -> CTemplate:
    path = asm_template_path(arch, repo)
    if not path.exists():
        raise GenError(f"{path}: missing (python3 tools/cargo_cell_gen.py --c-template)")
    sec: dict[str, list[str]] = {}
    cur = None
    for line in path.read_text().splitlines():
        if not line or line.startswith("#"):
            continue
        if line.startswith("[") and line.endswith("]"):
            cur = sec.setdefault(line[1:-1], [])
        elif cur is not None:
            cur.append(line)
    return CTemplate(
        sec["cc"][0], "", sec.get("std", []), sec.get("core", []), sec.get("defines", [])
    )


def asm_decode(f: CFlags, tmpl: CTemplate, arch: str, obj: str) -> list[str]:
    """The command Linker builds for an asm source, up to `-c`."""
    out = [_flag_dec(tmpl.cc, arch), *ASM_HEAD, "-o", obj]
    for inc in f.includes:
        out += [f"-I{_rc_dec(d, arch)}" for d in (tmpl.std if inc == "@std" else [inc])]
    defines = sorted(tmpl.defines + [f"-D{d}" for d in f.defines])
    flags = list(f.flags) if "@defines" in f.flags else [*f.flags, "@defines"]
    for fl in flags:
        if fl == "@core":
            out += [_flag_dec(x, arch) for x in tmpl.core]
        elif fl == "@defines":
            out += defines
        else:
            out.append(_flag_dec(fl, arch))
    return [*out, ASM_TAIL]


def asm_encode(cmd: str, tmpl: CTemplate, arch: str) -> CFlags:
    import shlex

    p = _asm_parts(cmd)
    inc = [_rc_enc(x, arch) for x in p.includes]
    inc = (_with_marker(inc, tmpl.std, "@std") or inc) if tmpl.std else inc
    flags = [_flag_enc(x, arch) for x in p.flags]
    flags = (_with_marker(flags, tmpl.core, "@core") or flags) if tmpl.core else flags
    left = list(p.defines)
    for d in tmpl.defines:
        if d not in left:
            raise GenError(f"{p.src}: lacks the global ASM define {d}")
        left.remove(d)
    f = CFlags(tuple(inc), tuple(flags), tuple(d[2:] for d in left))
    if asm_decode(f, tmpl, arch, p.obj) != shlex.split(cmd)[:-2]:
        raise GenError(f"{p.src}: the ASM command does not rebuild from the template")
    return f


def clib_recipe_path(arch: str, target: str, repo: Path = REPO) -> Path:
    return repo / f"sdk/lib/rust/reactos-link/clibs/{arch}/{target}.recipe"


def clib_recipe(arch: str, target: str, repo: Path = REPO) -> str:
    """A shared C library the engine builds (`engine:c:<t>`, wine, uuid, ...) as a recipe:
    one line per source in cmake's archive order, `<c|asm>\t<src>\t<includes>\t<flags>\t
    <defines>`, the part of that source's command the template does not hold. Generated
    sources are `@build/<rel>`, which a cmake-off build's generated SDK provides."""
    ctmpl, atmpl = read_c_template(arch, repo), read_asm_template(arch, repo)
    db = _ccdb(repo / f"build-{arch}/compile_commands.json")
    marker = f"/CMakeFiles/{target}.dir/"
    lines = [
        f"# {target} ({arch}), from cmake's compile of it: tools/cargo_cell_gen.py --clib-recipes"
    ]
    seen = set()
    for e in db:
        cmd = e.get("command", "")
        if marker not in cmd or e["file"] in seen:
            continue
        seen.add(e["file"])
        if e["file"].endswith(".c"):
            kind, f = "c", c_encode(cmd, ctmpl, arch)
        elif re.search(r"\.(s|S)$", e["file"]):
            kind, f = "asm", asm_encode(cmd, atmpl, arch)
        else:
            raise GenError(f"{target}: unmodelled source {e['file']}")
        toks = [*f.includes, *f.flags, *f.defines]
        if any(" " in x or "\t" in x for x in toks):
            raise GenError(f"{target}: a token with whitespace in {e['file']}")
        lines.append(
            "\t".join(
                [
                    kind,
                    _rc_enc(e["file"], arch),
                    " ".join(f.includes),
                    " ".join(f.flags),
                    " ".join(f.defines),
                ]
            )
        )
    if len(lines) == 1:
        raise GenError(f"{target}: no sources on {arch}")
    return "\n".join(lines) + "\n"


def engine_c_targets(repo: Path = REPO) -> list[str]:
    """Every cmake C library the engine compiles for a cell: `engine:c:<t>` and the named
    replays (wine, uuid, conutils)."""
    text = "".join(f.read_text() for f in (repo / "sdk/lib/rust").glob("*-cargo/build.rs"))
    names = set(re.findall(r'"engine:c:(\w+)"', text)) | {"wine", "uuid", "conutils"}
    return sorted(names)


SHIMS_CALL = re.compile(r'\.c_from_compile_db\(\s*"(\w+)"')
# c_from_compile_db libraries several cells share, which are cmake libraries of their own.
SHARED_C_LIBS = {"rtl", "blrtl", "libcntpr", "win32k_residue"}


def _call_span(text: str, start: int) -> int:
    """End of the call whose `(` follows `start`, past its `)`."""
    depth, i = 0, text.index("(", start)
    while True:
        c = text[i]
        depth += c == "("
        depth -= c == ")"
        i += 1
        if depth == 0:
            return i


def c_binding(srcs: dict, flags: dict) -> list[str]:
    """The `c_sources`/`c_includes`/`c_flags`/`c_defines` a cell passes to Linker::c_compiled."""
    L = [
        "    // The C left, and its part of cmake's C rule; reactos-link adds what every C",
        "    // compile shares from reactos-link/cflags/c.<arch>.flags (Linker::c_compiled).",
    ]
    tables = {"sources": {a: tuple(srcs[a]) for a in ARCHES}}
    for name in ("includes", "flags", "defines"):
        tables[name] = {a: getattr(flags[a], name) for a in ARCHES}
    for name, vals in tables.items():
        if len(set(vals.values())) == 1:
            L.append(f"    let c_{name}: &[&str] = {_rs_list(vals['amd64'], 4)};")
            continue
        L.append(f"    let c_{name}: &[&str] = match arch.as_str() {{")
        for a in ARCHES:
            L.append(f'        "{TRIPLE[a]}" => {_rs_list(vals[a], 8)},')
        L += ["        _ => unreachable!(),", "    };"]
    return L


def own_c(crates: list[str] | None = None, repo: Path = REPO) -> list[str]:
    """Move cells off cmake's `<m>_shims` OBJECT libraries: `.c_from_compile_db("<m>_shims",
    ...)` (a replay of cmake's commands) becomes `.c_compiled(...)` with the library's part
    of cmake's C rule, and the OBJECT library leaves the CMakeLists. An arch-guarded
    `<M>_SHIMS` list (acgenral, opengl32) is left as it is."""
    rust = repo / "sdk/lib/rust"
    dirs = [rust / c for c in crates] if crates else sorted(rust.glob("*-cargo"))
    moved = []
    for d in dirs:
        rs = d / "build.rs"
        text = rs.read_text() if rs.exists() else ""
        m = SHIMS_CALL.search(text)
        if not m or m.group(1) in SHARED_C_LIBS:
            continue
        name = m.group(1)
        plain = rf"add_library\(\s*{name}\s+OBJECT\s+(?!\$\{{\w+_SHIMS\}})"
        cmake = [f for f in _cmakelists(repo) if re.search(plain, f.read_text())]
        if len(cmake) != 1:
            print(f"left on replay: {d.name} ({name}: {len(cmake)} plain definitions)")
            continue
        srcs, flags = c_lib_for(name, repo)
        end = _call_span(text, m.start())
        text = (
            text[: m.start()]
            + f'.c_compiled("{name}", c_sources, c_includes, c_flags, c_defines)'
            + text[end:]
        )
        lines = text.split("\n")
        at = next(i for i, ln in enumerate(lines) if ln.strip().startswith("let arch = env::var("))
        lines[at + 1 : at + 1] = ["", *c_binding(srcs, flags)]
        rs.write_text("\n".join(lines))
        gen = sorted({x for a in ARCHES for x in srcs[a] if x.startswith("@build/")})
        _drop_shims_in(
            cmake[0], name, ["${REACTOS_BINARY_DIR}/" + x[len("@build/") :] for x in gen]
        )
        moved.append(d.name)
    return moved


def _drop_shims_in(cmake: Path, target: str, generated: list[str]) -> None:
    """Remove the OBJECT library and its statements; what ordered or fed its compiles
    (add_dependencies, the targets it linked for their usage requirements, the generated
    sources) orders the cell instead."""
    text = cmake.read_text()
    keep, pos, deps = [], 0, []
    for s, e, cmd, raw in _statements(text):
        a = _args(raw)
        if a[:1] == [target]:
            if cmd == "add_dependencies":
                deps += a[1:]
            if cmd == "target_link_libraries":
                deps += [
                    x
                    for x in a[1:]
                    if x not in ("PRIVATE", "PUBLIC", "INTERFACE") and not x.startswith("${")
                ]
            gap = text[pos:s]
            if cmd == "add_library":
                lines = gap.split("\n")
                while len(lines) > 1 and lines[-2].lstrip().startswith(
                    ("# The C still compiled", "# whose compile")
                ):
                    del lines[-2]
                gap = "\n".join(lines)
            keep.append(gap)
            pos = e
            continue
        keep.append(text[pos:e])
        pos = e
    keep.append(text[pos:])
    out = "".join(keep)
    for c in (
        "# How cmake compiled the target's sources, now on the OBJECT libraries.\n",
        "# Generated inputs those compiles read, ordered before them.\n",
    ):
        out = out.replace(c, "")

    def extra_deps(m: re.Match) -> str:
        names = [x for x in m.group(1).split() if x != target]
        for d in deps + generated:
            if d not in names:
                names.append(d)
        return "EXTRA_DEPENDS " + " ".join(names)

    out, n = re.subn(rf"EXTRA_DEPENDS ([^)\n]*\b{re.escape(target)}\b[^)\n]*)", extra_deps, out)
    if n == 0 or re.search(rf"\b{re.escape(target)}\b", out):
        raise GenError(f"{cmake}: {target} still named after the rewrite")
    cmake.write_text(re.sub(r"\n{3,}", "\n\n", out))


def _rc_flags_by_analogy(rc_rel: str, repo: Path) -> dict:
    """A module the configuration does not build (kdcom.dll's alternative providers: one of
    kdrosdbg, kdcom, kdgdb is added per build) has no windres command to read. Its
    `<m>_res` is declared exactly like its configured sibling's in the same parent
    directory, so it takes that command with the module directory swapped."""
    mine = Path(rc_rel).parent
    for e in _rc_commands("amd64", repo):
        src = Path(e["file"].replace(CANON + "/", ""))
        if src.parent.parent != mine.parent or not re.search(
            r"/CMakeFiles/\w+_res\.dir/", e["command"]
        ):
            continue
        sib = rc_flags_for(src.as_posix(), str(src.parent) + "/CMakeFiles/", repo)

        def swap(f: RcFlags, old: str = str(src.parent), new: str = str(mine)) -> RcFlags:
            def fix(x: str) -> str:
                return re.sub(rf"(^|/){re.escape(old)}$", rf"\g<1>{new}", x)

            return RcFlags(tuple(fix(x) for x in f.includes), f.flags, f.defines)

        return {a: swap(f) for a, f in sib.items()}
    raise GenError(f"{rc_rel}: no windres command, and no configured sibling to take one from")


def _rc_object_depends(cmd: str, raw: str) -> list[str] | None:
    """What `add_rc_deps(x.rc ...)` / `set_source_files_properties(x.rc PROPERTIES
    OBJECT_DEPENDS ...)` made the .rc compile wait for, or None for any other statement.
    With the compile in the engine they order nothing; a GENERATED input among them (the
    typelib wscript's rsrc.rc embeds) must order the cell instead."""
    a = _args(raw)
    if cmd == "add_rc_deps" and a and a[0].endswith(".rc"):
        return a[1:]
    if (
        cmd == "set_source_files_properties"
        and len(a) >= 4
        and all(x.endswith(".rc") for x in a[: a.index("PROPERTIES")])
        and a[a.index("PROPERTIES") + 1 :][:1] == ["OBJECT_DEPENDS"]
    ):
        return [x.strip('"') for x in a[a.index("PROPERTIES") + 2 :]]
    return None


def _drop_res_library(target: str, repo: Path) -> None:
    """Remove `add_library(<target> OBJECT ...)` and every statement on it from the
    CMakeLists that defines it; what it was ordered after (`add_dependencies(<target> X)`)
    moves to the add_cargo_module EXTRA_DEPENDS that named it."""
    hits = [
        f
        for f in _cmakelists(repo)
        if re.search(rf"add_library\(\s*{re.escape(target)}\s+OBJECT", f.read_text())
    ]
    if not hits:
        raise GenError(f"{target}: defined in no CMakeLists")
    for cmake in hits:
        _drop_res_in(cmake, target)


def _drop_res_in(cmake: Path, target: str) -> None:
    text = cmake.read_text()
    keep, pos, deps = [], 0, []
    for s, e, cmd, raw in _statements(text):
        a = _args(raw)
        rc_deps = _rc_object_depends(cmd, raw)
        if a[:1] == [target] or rc_deps is not None:
            if cmd == "add_dependencies":
                deps += a[1:]
            deps += [d for d in rc_deps or [] if "CMAKE_CURRENT_BINARY_DIR" in d]
            gap = text[pos:s]
            if cmd == "add_library":
                # Its explanatory comment goes with it.
                lines = gap.split("\n")
                while len(lines) > 1 and lines[-2].lstrip().startswith(
                    ("# The resources are", "# whose windres command")
                ):
                    del lines[-2]
                gap = "\n".join(lines)
            keep.append(gap)
            pos = e
            continue
        keep.append(text[pos:e])
        pos = e
    keep.append(text[pos:])
    out = "".join(keep)
    # The GLOB that only fed add_rc_deps.
    for m in re.finditer(r"file\(GLOB (\w+)[^)]*\)\n", out):
        if out.count("${" + m.group(1) + "}") == 0:
            out = out.replace(m.group(0), "", 1)
    for c in (
        "# How cmake compiled the target's sources, now on the OBJECT libraries.\n",
        "# Generated inputs those compiles read, ordered before them.\n",
    ):
        if c in out and not re.search(c.replace(".", r"\.") + r"\s*\S*_shims", out):
            nxt = out[out.index(c) + len(c) :]
            if not re.match(r"(?:\s*if\([^)]*\)\s*)?\S+\(\s*\S*_shims", nxt):
                out = out.replace(c, "", 1)

    def extra_deps(m: re.Match) -> str:
        names = [x for x in m.group(1).split() if x != target]
        for d in [*deps, "psdk"]:
            if d not in names:
                names.append(d)
        return "EXTRA_DEPENDS " + " ".join(names)

    out, n = re.subn(rf"EXTRA_DEPENDS ([^)\n]*\b{re.escape(target)}\b[^)\n]*)", extra_deps, out)
    if n == 0 and re.search(rf"\b{re.escape(target)}\b", out):
        raise GenError(f"{cmake}: {target} still named after the rewrite")
    out = re.sub(r"\n{3,}", "\n\n", out).lstrip("\n")
    cmake.write_text(out)


def save_baseline(targets: list[str], baseline: Path, repo: Path = REPO) -> None:
    """Copy each target's cmake-built PE, per arch, into `baseline`, with the index
    `--check` reads (output name and CD placement per arch)."""
    index_path = baseline / "targets.json"
    index = json.loads(index_path.read_text()) if index_path.exists() else {}
    for t in targets:
        for a in ARCHES:
            out = link_edge(a, t, repo).output
            src = repo / f"build-{a}" / out
            if not src.is_file():
                raise GenError(f"{t}: no cmake-built {out} in build-{a}; build every arch first")
            dst = baseline / a / out
            dst.parent.mkdir(parents=True, exist_ok=True)
            dst.write_bytes(src.read_bytes())
            index.setdefault(t, {})[a] = out
            index[t][f"cd:{a}"] = cd_placement(a, out, repo)
    index_path.parent.mkdir(parents=True, exist_ok=True)
    index_path.write_text(json.dumps(index, indent=1, sort_keys=True))


def _ninja(arch: str, targets: list[str], repo: Path) -> None:
    """Regenerate the graph from the rewritten CMakeLists, then build only the cells."""
    build = repo / f"build-{arch}"
    for step in (["ninja", "build.ninja"], ["ninja", *targets]):
        cp = subprocess.run(
            ["bash", str(repo / "tools/canon-run.sh"), *step],
            cwd=build,
            capture_output=True,
            text=True,
            check=False,
        )
        if cp.returncode:
            tail = "\n".join((cp.stdout + cp.stderr).splitlines()[-15:])
            raise GenError(f"build-{arch}: `{' '.join(step)}` failed:\n{tail}")


def convert(targets: list[str], baseline: Path | None = None, repo: Path = REPO) -> int:
    """Move freshly flipped modules from cmake's link to cargo cells in one command.

    The wave procedure, minus the three full builds it used to cost: the cmake-built PEs
    the lane's own build left in `build-<arch>` are the baseline, every target is modelled
    before anything is written (one refusal writes nothing), and after `--apply` only the
    new cells are rebuilt (`ninja <targets>` under the canonical mount) before `--check`
    compares them. Run it after the build that follows `rustconv port flip`; the next full
    build and the gate's smoke then cover the boot.
    """
    if not targets:
        raise GenError("--convert needs at least one target")
    cells = []
    for t in targets:
        cells.append(model(t, repo))
    baseline = baseline or (
        Path.home() / "vibeos-cargo-baselines" / time.strftime("convert-%Y%m%d-%H%M%S")
    )
    save_baseline(targets, baseline, repo)
    print(f"baseline: {baseline}")
    for cell in cells:
        for p in apply(cell, repo):
            print(f"wrote {p.relative_to(repo)}")
    for arch in ARCHES:
        _ninja(arch, targets, repo)
        print(f"built {len(targets)} cell(s) on {arch}")
    rc = check(targets, baseline, repo)
    refresh_manifests(repo)
    return rc


def refresh_manifests(repo: Path = REPO) -> None:
    """What a build with no cmake reads, regenerated from the graphs this wave just
    configured: the cell and media manifests (scan_cargo_manifests fails while they are
    stale) and the shared C libraries' recipes."""
    for tool in ("cell_manifest.py", "media_manifest.py"):
        subprocess.run([sys.executable, str(repo / "tools" / tool), "--write"], check=True)
    for a in ARCHES:
        for t in engine_c_targets(repo):
            try:
                clib_recipe_path(a, t, repo).write_text(clib_recipe(a, t, repo))
            except GenError:
                continue  # not built on this arch (freeldr's BIOS asm is amd64 only)


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("targets", nargs="*")
    ap.add_argument("--apply", action="store_true")
    ap.add_argument("--survey", action="store_true")
    ap.add_argument(
        "--save-baseline",
        metavar="DIR",
        type=Path,
        help="copy each target's cmake-built PE, per arch, into DIR",
    )
    ap.add_argument(
        "--check",
        metavar="DIR",
        type=Path,
        help="pe_equiv each target's current PE against DIR, on every arch",
    )
    ap.add_argument(
        "--convert",
        action="store_true",
        help="the whole wave for TARGETS in one command: baseline from the current build, "
        "apply, build just the new cells on every arch, check equivalence",
    )
    ap.add_argument(
        "--own-rc",
        action="store_true",
        help="move existing cells (all, or the named *-cargo crates) off cmake's <m>_res "
        "libraries: the engine compiles the .rc (Linker::rc)",
    )
    ap.add_argument(
        "--own-c",
        action="store_true",
        help="move existing cells (all, or the named *-cargo crates) off cmake's <m>_shims "
        "libraries: the engine compiles the C (Linker::c_compiled)",
    )
    ap.add_argument(
        "--clib-recipes",
        action="store_true",
        help="regenerate reactos-link/clibs/<arch>/<t>.recipe for every C library the engine "
        "compiles for a cell (engine:c:<t>, wine, uuid, conutils)",
    )
    ap.add_argument(
        "--c-template",
        action="store_true",
        help="regenerate reactos-link/cflags/c.<arch>.flags from each build's C commands",
    )
    ap.add_argument(
        "--rc-template",
        action="store_true",
        help="regenerate reactos-link/cflags/rc.<arch>.flags from each build's windres commands",
    )
    ap.add_argument(
        "--remap",
        action="store_true",
        help="move existing cells (all, or the named *-cargo crates) off cmake-built "
        "archives the engine can build itself",
    )
    ap.add_argument(
        "--baseline",
        metavar="DIR",
        type=Path,
        help="with --convert: where to keep the baselines "
        "(default ~/vibeos-cargo-baselines/convert-<time>)",
    )
    ap.add_argument(
        "--reshell",
        type=Path,
        metavar="DIR",
        help="re-choose existing cells' panic handler from the cmake-built PEs in DIR",
    )
    args = ap.parse_args(argv)
    if args.save_baseline:
        save_baseline(args.targets, args.save_baseline)
        print(f"saved {len(args.targets)} modules x {len(ARCHES)} arches")
        return 0
    if args.rc_template:
        for a in ARCHES:
            rc_template_path(a).write_text(derive_rc_template(a).text(a))
            print(f"wrote {rc_template_path(a).relative_to(REPO)}")
        return 0
    if args.c_template:
        for a in ARCHES:
            c_template_path(a).write_text(derive_c_template(a).text(a))
            asm_template_path(a).write_text(derive_asm_template(a).text(a))
            print(f"wrote {c_template_path(a).relative_to(REPO)} and asm.{a}.flags")
        return 0
    if args.clib_recipes:
        for a in ARCHES:
            for t in engine_c_targets():
                try:
                    text = clib_recipe(a, t)
                except GenError as e:
                    print(f"{a} {t}: {e}")
                    continue
                path = clib_recipe_path(a, t)
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(text)
        return 0
    if args.own_c:
        for crate in own_c(args.targets or None):
            print(f"owns its C: {crate}")
        return 0
    if args.own_rc:
        for crate in own_rc(args.targets or None):
            print(f"owns its rc: {crate}")
        return 0
    if args.remap:
        for crate in remap(args.targets or None):
            print(f"remapped {crate}")
        return 0
    if args.convert:
        return convert(args.targets, args.baseline)
    if args.check:
        return check(args.targets, args.check)
    if args.reshell:
        reshell(args.targets, args.reshell)
        return 0
    if args.survey:
        rows = survey()
        for t, v in rows:
            print(f"{t:24} {v}")
        ok = sum(1 for _, v in rows if v.startswith("OK"))
        print(f"\n{ok} of {len(rows)} placeholder-stub modules generate cleanly")
        return 0
    rc = 0
    for t in args.targets:
        try:
            cell = model(t)
            if args.apply:
                for p in apply(cell):
                    print(f"wrote {p.relative_to(REPO)}")
            else:
                print(f"== {cell.crate}/build.rs\n{build_rs(cell)}")
                print(
                    f"== {cell.cmake_dir.relative_to(REPO)}/CMakeLists.txt\n"
                    f"{rewrite_cmake(cell.cmake_dir / 'CMakeLists.txt', cell)}"
                )
        except GenError as exc:
            print(f"REFUSED {exc}")
            rc = 1
    return rc


if __name__ == "__main__":
    sys.exit(main())
