#!/usr/bin/env python3
"""Move a fully-Rust module's link from cmake to cargo: generate its `<m>-cargo` cell.

A module whose C is all flipped still has cmake link it: `add_executable`/`add_library`
around a placeholder `rust_cohab_stub.c`, `set_module_type`, `add_importlibs`. This tool
reads what that link ACTUALLY does, from each `build-<arch>/build.ninja` (entry point,
subsystem, image base, the libraries, the `.def`, the resources), and writes the
reactos-link cell that does the same thing:

  sdk/lib/rust/<m>-cargo/{Cargo.toml, build.rs, src/main.rs, i686-kernel.json}
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
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO / "tools"))
from cohab_census import SRC_IN_COMMENT  # noqa: E402

RUST = REPO / "sdk/lib/rust"
ARCHES = ("i386", "amd64", "arm64")
TRIPLE = {"i386": "x86", "amd64": "x86_64", "arm64": "aarch64"}
TARGET = {
    "i386": "i686-kernel.json",
    "amd64": "x86_64-pc-windows-gnu",
    "arm64": "aarch64-pc-windows-gnullvm",
}
# The user-mode (hard-float) variant every user cell carries; see cargo_orchestrate.
USER_JSON_FROM = RUST / "userinit-cargo/i686-kernel.json"
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
# The engine builds a body with `-C force-frame-pointers=yes` (stack walks need them) where
# cmake's rust_staticlib_variant did not; a frame pointer can push a big frame over the
# probe threshold. ntdll exports __chkstk on amd64/arm64 and kernel32 forwards it there.
STACK_PROBE_IMPORTS = {"kernel32.dll!__chkstk", "ntdll.dll!__chkstk"}
# What compiler-builtins' `mem` feature defines on i386 (see check()).
I386_BUILTIN_MEM = {"memcmp", "memcpy", "memmove", "memset", "strlen", "bcmp"}
# How much more the loader may map for a cargo-linked module than for cmake's: the cell's
# own handler, startup and section padding. All of core is ~200 KB on amd64.
SIZE_SLACK = 16 * 1024
# i386 only, and only where the cell keeps panic_common: the cell builds its own core
# (nightly build-std on the kernel json) beside the body's i686-pc-windows-gnu one, so
# the handler links a second copy of core::fmt, measured at 70-78 KB. The fix is a
# user-target i386 cell (owed: docs/engineering/cargo-drive-plan.md).
I386_SECOND_CORE = 80 * 1024
# Statements on the target that the cargo cell replaces outright.
REPLACED = (
    "add_executable",
    "add_library",
    "set_module_type",
    "add_importlibs",
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
# edge (chcp.com, help.exe for cmd_help).
NAME_PROPERTIES = {"SUFFIX", "OUTPUT_NAME", "PREFIX"}
# Statements naming the target that neither link nor compile it.
KEPT = ("project",)

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
    destination: str | None
    name_on_cd: str | None
    # Per arch: did cmake's link carry the body's panic handler? Only then does the cell
    # take panic_common (and with it all of core); otherwise panic-lean.
    panics: dict[str, bool] = field(default_factory=lambda: dict.fromkeys(ARCHES, True))
    # The rust_staticlib_variant that builds the body; "" means `<target>_rust`.
    body_var: str = ""
    # Per arch, the static libs cmake linked AFTER the body (they lose a duplicate
    # definition to it); the rest of `static_libs` precede it.
    late_libs: dict[str, list[str]] = field(default_factory=dict)
    notes: list[str] = field(default_factory=list)


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
    entry = re.search(r"-entry,(\S+)", flags_s)
    sub = re.search(r"--subsystem,([\w:.]+)", flags_s)
    base = re.search(r"--image-base,(0x[0-9a-fA-F]+)", flags_s)
    stack = re.search(r"--stack,(\d+)", flags_s)
    if "--delayload" in flags_s or "delayimp" in (libs_m.group(1) if libs_m else ""):
        raise GenError(f"{target}: delay imports are not modelled yet")
    defs = re.findall(r"(\S+\.def)\b", m.group(0).split("\n", 1)[0])
    return Link(
        arch,
        output,
        kind,
        entry.group(1) if entry else "",
        sub.group(1) if sub else None,
        int(base.group(1), 16) if base else None,
        int(stack.group(1)) if stack else None,
        ins.split("|")[0].split() + [d for d in defs if d not in ins.split("|")[0].split()],
        libs_m.group(1).split() if libs_m else [],
    )


@functools.cache
def spec_index(repo: Path = REPO) -> dict[str, tuple[str, str]]:
    """import-lib basename -> (spec, dll file name), from every `spec2def(... ADD_IMPORTLIB)`."""
    out: dict[str, tuple[str, str]] = {}
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
            out.setdefault(dll.rsplit(".", 1)[0].lower(), (spec_rel, dll))
    return out


def _rc_command(arch: str, rc_obj: str, repo: Path = REPO) -> tuple[str, list[str]]:
    db = _ccdb(repo / f"build-{arch}/compile_commands.json")
    for e in db:
        cmd = e.get("command", "")
        if rc_obj in cmd and e["file"].endswith(".rc"):
            src = e["file"].replace("/tmp/vibeos/", "")
            return src, re.findall(r"\s-D(\S+)", cmd)
    raise GenError(f"no windres command for {rc_obj} in build-{arch}/compile_commands.json")


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
        "destination": None,
        "name_on_cd": None,
        "body": None,
        "body_var": "",
        "transfer": [],
        "unknown": [],
        "unicode": False,
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
            if re.search(r"\$\{(SOURCE|PCH_SKIP_SOURCE)\}", raw):
                facts["unknown"].append(f"{cmd} {a[0] if a else ''} shares ${{SOURCE}}")
            continue
        if cmd == "set_module_type" and "UNICODE" in a:
            facts["unicode"] = True
        if cmd == "add_cd_file":
            if "DESTINATION" in a:
                dest = a[a.index("DESTINATION") + 1]
                if facts["destination"] not in (None, dest):
                    # winprint: one add_cd_file per arch, each to its own prtprocs dir.
                    facts["unknown"].append(
                        f"add_cd_file to both {facts['destination']} and {dest}"
                    )
                facts["destination"] = dest
            if "NAME_ON_CD" in a:
                facts["name_on_cd"] = a[a.index("NAME_ON_CD") + 1]
        elif cmd.lower() in TRANSFERRED:
            facts["transfer"].append(raw)
        elif cmd == "set_target_properties" and _names_only(a):
            pass  # the output name comes from the link edge
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
            # calc/taskmgr/wordpad pass `${X_RUST_ENV}` (i386 +sse,+sse2 for the body):
            # the cell would build the body without it, and pe_equiv cannot see a
            # codegen change.
            facts["unknown"].append(f"rust_staticlib_variant extras {a[4:]}")
    elif len(named) > 1:
        facts["unknown"].append(f"several bodies {named}")
    return facts


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
    keep, pos = [], 0
    body_lib = None
    body_var = cell.body_var or f"{cell.target}_rust"
    transferred: list[tuple[str, str]] = []
    # `add_dependencies(m evtmsgstr)` orders a GENERATED input (an mc header the rc #includes)
    # before the compile that reads it. The cell drops the target's add_dependencies, and the
    # compile now lives on `<m>_res`/`<m>_shims`, so the ordering moves there with it — or a
    # fresh build races windres against the mc step. The `rust_*` body deps are the cell's own.
    build_deps: list[str] = []
    shared = any(
        re.search(rf"\$\{{{body_var}_LIB\}}|rust_{body_var}\b", raw)
        for _s, _e, _cmd, raw in _statements(text)
        if _args(raw)[:1] != [cell.target] and _cmd != "rust_staticlib_variant"
    )
    # `# main.c  # replaced by Rust (...)` lines are how cohab_census and the coverage
    # tools know a TU was translated: whatever statement held them, they are kept.
    carried: list[str] = []

    def carry(chunk: str) -> None:
        carried.extend(ln.strip() for ln in chunk.splitlines() if SRC_IN_COMMENT.match(ln))

    for s, e, cmd, raw in _statements(text):
        a = _args(raw)
        drop = False
        if (
            cmd in REPLACED
            and a
            and (a[0] == cell.target or (cmd == "add_cd_file" and cell.target in a))
        ):
            drop = True
        if cmd == "add_dependencies" and a and a[0] == cell.target:
            build_deps += [d for d in a[1:] if not d.startswith("rust_") and d not in build_deps]
        if a and a[0] == cell.target and cmd.lower() in TRANSFERRED:
            drop = True  # re-emitted below, on the OBJECT libraries
            transferred.append((cmd, raw))
        if a and a[0] == cell.target and cmd == "set_target_properties" and _names_only(a):
            drop = True
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
        gap = text[pos:s]
        if drop:
            # A comment block attached to a dropped statement describes it: drop it too.
            lines = gap.split("\n")
            while len(lines) > 1 and lines[-2].lstrip().startswith("#"):
                carry(lines[-2])
                del lines[-2]
            gap = "\n".join(lines)
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
    out = re.sub(r"\n{3,}", "\n\n", out).rstrip() + "\n"
    block = [
        "",
        f"# {cell.output_name} is all Rust ({cell.body_crate}) and cargo links it:",
        f"# sdk/lib/rust/{cell.target}-cargo (reactos-link, generated by tools/cargo_cell_gen.py).",
        "# The C stays on disk, uncompiled.",
    ]
    if carried:
        block.append("# The translated C, kept on disk:")
        block += list(dict.fromkeys(carried))
    extra = []
    if cell.c_shims:
        block.append(
            "# The C still compiled, in an OBJECT library whose compile the cargo link replays."
        )
        shims = f"{cell.target}_shims"
        block.append(f"add_library({shims} OBJECT {' '.join(Path(c).name for c in cell.c_shims)})")
        if cell.rc_defines:
            block.append(f"target_compile_definitions({shims} PRIVATE {' '.join(cell.rc_defines)})")
        extra.append(f"{cell.target}_shims")
    if cell.rc:
        rc_name = Path(cell.rc).name
        block.append(
            "# The resources are the one thing cmake still compiles, in this OBJECT library"
        )
        block.append("# whose windres command the cargo link replays.")
        block.append(f"add_library({cell.target}_res OBJECT {rc_name})")
        if cell.rc_defines:
            block.append(
                f"target_compile_definitions({cell.target}_res PRIVATE {' '.join(cell.rc_defines)})"
            )
        block.append(
            f"target_include_directories({cell.target}_res PRIVATE ${{CMAKE_CURRENT_SOURCE_DIR}})"
        )
        extra.append(f"{cell.target}_res")
    objlibs = [x for x in extra if x in (f"{cell.target}_shims", f"{cell.target}_res")]
    if transferred and objlibs:
        block.append("# How cmake compiled the target's sources, now on the OBJECT libraries.")
        block += [_retarget(c, r, lib) for lib in objlibs for c, r in transferred]
    if build_deps and objlibs:
        block.append("# Generated inputs those compiles read, ordered before them.")
        block += [f"add_dependencies({lib} {' '.join(build_deps)})" for lib in objlibs]
    every = sorted({lib for libs in cell.static_libs.values() for lib in libs})
    by_arch = {a: set(libs) for a, libs in cell.static_libs.items()}
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
    kind = "win32exe" if cell.module_kind == "Win32Exe" else "win32dll"
    call = [f"add_cargo_module({cell.target} {cell.target}-cargo {cell.output_name}"]
    call.append(f"    KIND {kind}")
    call.append(f"    BODY_CRATE {cell.body_crate}")
    if cell.destination:
        call.append(f"    DESTINATION {cell.destination}")
    else:
        call.append("    NO_CD")
    if cell.name_on_cd:
        call.append(f"    NAME_ON_CD {cell.name_on_cd}")
    if extra:
        call.append(f"    EXTRA_DEPENDS {' '.join(extra)})")
    else:
        call[-1] += ")"
    if build_deps and not objlibs:
        call.append(f"add_dependencies({cell.target} {' '.join(build_deps)})")
    return out + "\n".join(block + call) + "\n"


# ---------------------------------------------------------------- the model


def _placeholder(path: Path) -> bool:
    """A `rust_cohab_stub.c` that only exists to give cmake a TU."""
    body = re.sub(r"/\*.*?\*/|//[^\n]*", "", path.read_text(errors="replace"), flags=re.S)
    body = re.sub(r"typedef\s+int\s+\w+\s*;", "", body)
    return not body.strip()


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
    if first.kind == "C_EXECUTABLE":
        if not (first.subsystem or "").startswith(("windows", "console")):
            raise GenError(f"{target}: {first.subsystem} exe is not modelled (native?)")
        module_kind = "Win32Exe"
    elif first.kind in ("C_MODULE_LIBRARY", "C_SHARED_LIBRARY"):
        # A kernel-mode driver or a win32k font driver links native:, with a kernel
        # target and no user CRT: that is KernelDll, which the generator does not model.
        if (first.subsystem or "").startswith("native"):
            raise GenError(f"{target}: kernel-mode {first.subsystem} module is not modelled")
        module_kind = "Win32Dll"
    else:
        raise GenError(f"{target}: link kind {first.kind}")
    specs = spec_index(repo)
    body_ar = f"lib{facts['body_var']}.a"
    per_arch: dict[str, tuple] = {}
    late: dict[str, list[str]] = {a: [] for a in ARCHES}
    for a in ARCHES:
        imports, static, msvcrtex = [], [], None
        after_body = False
        for lib in links[a].libs:
            name = lib.split("/")[-1]
            # Only the module's OWN body is the cell's rust_staticlib; another crate's
            # `_rust.a` (subst's libconutils_rust.a) is a library like any other.
            if name == body_ar:
                after_body = True
                continue
            if "libclang_rt" in lib:
                continue
            m = re.fullmatch(r"libmsvcrtex_(\w+)\.a", name)
            if m:
                if m.group(1) in MSVCRTEX:
                    msvcrtex = MSVCRTEX[m.group(1)]
                continue
            m = re.search(r"CMakeFiles/lib(\w+)\.dir/lib\1\.a$", lib)
            if m:
                base = m.group(1).lower()
                if base not in specs:
                    raise GenError(f"{target}: import lib {base} has no spec2def ADD_IMPORTLIB")
                if (specs[base][0], specs[base][1], base) not in imports:
                    imports.append((specs[base][0], specs[base][1], base))
                continue
            if lib not in static:
                static.append(lib)
                if after_body:
                    late[a].append(lib)
        per_arch[a] = (tuple(imports), tuple(static), msvcrtex)
    if len({(v[0], v[2]) for v in per_arch.values()}) > 1:
        raise GenError(f"{target}: imports or CRT differ by arch (not modelled)")
    imports, _, msvcrtex = per_arch["amd64"]
    static_by_arch = {a: list(v[1]) for a, v in per_arch.items()}
    shims, rc, rc_defines, spec, stubs = [], None, [], None, False
    for obj in first.inputs:
        nm = obj.split("/")[-1]
        if nm.endswith(".rc.res"):
            rc, _defs = _rc_command("amd64", obj, repo)
        elif nm.endswith("_stubs.rs.obj") or nm.endswith("_stubs.c.obj"):
            stubs = True
        elif nm.endswith(".def"):
            m = re.search(r"spec2def\(\s*(\S+)\s+(\S+\.spec)", cmake.read_text())
            if not m:
                raise GenError(f"{target}: a .def without a spec2def()")
            spec = ((cmake_dir / m.group(2)).relative_to(repo).as_posix(), m.group(1))
        elif nm.endswith(".c.obj"):
            src = cmake_dir / nm.removesuffix(".obj")
            if not src.exists():
                raise GenError(f"{target}: compiled C {nm} not found beside the CMakeLists")
            if nm == "rust_cohab_stub.c.obj" and _placeholder(src):
                continue
            shims.append(src.relative_to(repo).as_posix())
        else:
            raise GenError(f"{target}: unmodelled link input {nm}")
    # The inputs are read from the amd64 link; an `if(ARCH ...)` that adds a file on one
    # arch (acgenral's msys2.c, amd64 only) would compile it everywhere.
    # The same goes for opengl32's i386-only asm trampolines: every link input must agree.
    by_arch = {a: sorted(o.split("/")[-1] for o in links[a].inputs) for a in ARCHES}
    if len({tuple(v) for v in by_arch.values()}) > 1:
        odd = {a: sorted(set(v) ^ set(by_arch["amd64"])) for a, v in by_arch.items()}
        raise GenError(f"{target}: link inputs differ by arch {odd}")
    # set_module_type(... UNICODE) is the one target-level define a user module's
    # compiles get; the OBJECT libraries that replace the target must carry it.
    rc_defines = ["UNICODE", "_UNICODE"] if facts["unicode"] else []
    if module_kind == "Win32Exe" and not msvcrtex:
        raise GenError(f"{target}: exe without a msvcrtex CRT startup")
    stacks = {links[a].stack for a in ARCHES}
    if len(stacks) > 1:
        raise GenError(f"{target}: stack reserve differs by arch {stacks}")
    body, feats, defaults = facts["body"]
    panics = {a: carries_panic_handler(repo / f"build-{a}" / links[a].output) for a in ARCHES}
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
        import_libs=list(imports),
        msvcrtex=msvcrtex,
        static_libs=static_by_arch,
        spec=spec,
        spec_stubs=stubs,
        c_shims=shims,
        rc=rc,
        rc_defines=rc_defines,
        destination=facts["destination"],
        name_on_cd=facts["name_on_cd"],
        panics=panics,
        body_var=facts["body_var"],
        late_libs=late,
    )


# ---------------------------------------------------------------- emit


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
    L.append(
        "    let (target_json, entry"
        + (", image_base" if dll else "")
        + ") = match arch.as_str() {"
    )
    for a in ARCHES:
        base = cell.image_base[a]
        tail = ""
        if dll:
            # None: cmake passed no --image-base, so lld's auto base applies.
            tail = f", Some({base:#x}u64)" if base is not None else ", None"
        L.append(f'        "{TRIPLE[a]}" => ("{TARGET[a]}", "{cell.entries[a]}"{tail}),')
    L.append(f'        o => panic!("{cell.target}-cargo: unsupported arch {{o}}"),')
    L.append("    };")
    L.append("")
    late = {a: cell.late_libs.get(a, []) for a in ARCHES}
    early = {a: [x for x in cell.static_libs[a] if x not in late[a]] for a in ARCHES}
    for var, table in (("static_libs", early), ("late_libs", late)):
        if not any(table.values()):
            continue
        L.append(f"    let {var}: &[&str] = match arch.as_str() {{")
        for a in ARCHES:
            libs = ", ".join(f'"{x}"' for x in table[a])
            L.append(f'        "{TRIPLE[a]}" => &[{libs}],')
        L.append("        _ => unreachable!(),")
        L.append("    };")
        L.append("")
    L.append("    Linker::new(target_json)")
    if cell.body_features or not cell.body_defaults:
        feats = ", ".join(f'"{f}"' for f in cell.body_features)
        L.append(
            f'        .rust_lib("{cell.body_crate}", &[{feats}], {str(cell.body_defaults).lower()})'
        )
    else:
        L.append(f'        .rust_staticlib("{cell.body_crate}")')
    if cell.c_shims:
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
        L.append(f'        .resource_from_rc("{cell.rc}", "{cell.target}_res")')
    if cell.spec:
        L.append(f'        .spec2def("{cell.spec[0]}", "{cell.spec[1]}")')
    if cell.spec_stubs:
        L.append("        .spec_stubs()")
    for spec, dllname, base in cell.import_libs:
        L.append(f'        .import_lib("{spec}", "{dllname}", "{base}")')
    if any(early.values()):
        L.append("        // cmake-built static libraries, reused until each has a cargo home.")
        L.append("        .static_libs(static_libs)")
    if any(late.values()):
        L.append("        // ...and those cmake linked after the body, in the same order.")
        L.append("        .static_libs_after_body(late_libs)")
    if cell.msvcrtex:
        L.append(f"        .msvcrtex_crt(MsvcrtexKind::{cell.msvcrtex})")
    if cell.stack:
        L.append(f"        .stack_reserve({cell.stack})")
    if dll:
        sub = f'Some("{cell.subsystem}".into())' if cell.subsystem else "None"
        L.append("        .link(ModuleKind::Win32Dll {")
        L.append("            entry: entry.into(),")
        L.append("            image_base,")
        L.append(f"            subsystem: {sub},")
        L.append("        });")
    else:
        L.append("        .link(ModuleKind::Win32Exe {")
        L.append("            entry: entry.into(),")
        L.append(f'            subsystem: "{cell.subsystem}".into(),')
        L.append("        });")
    L.append("}")
    return "\n".join(L) + "\n"


CFG_ARCH = {"i386": "x86", "amd64": "x86_64", "arm64": "aarch64"}


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
    bin_name = cell.target
    return f"""# Cargo-driven `{cell.output_name}` link, generated by tools/cargo_cell_gen.py.

[package]
name = "{cell.target}-cargo"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "{bin_name}"
path = "src/main.rs"

{_panic_deps(cell)}[build-dependencies]
reactos-link = {{ path = "../reactos-link" }}
"""


def _panic_deps(cell: Cell) -> str:
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
    head = MAIN_RS_HEAD
    common = '    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)\n'
    lean = f'    panic_lean::panic_lean(b"{cell.target}-cargo\\0", info)\n'
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
//! Empty bin shell: the PE's entry is the CRT startup or DllMain in the linked body,
//! and nothing here runs. Cargo needs a `main.rs` to drive the link.
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
    crate = repo / "sdk/lib/rust" / f"{cell.target}-cargo"
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
    (crate / "i686-kernel.json").write_bytes(USER_JSON_FROM.read_bytes())
    ws = repo / "sdk/lib/rust/Cargo.toml"
    text = ws.read_text()
    member = f'    "{cell.target}-cargo",\n'
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


def _cell_uses_panic_common(target: str, arch: str, repo: Path) -> bool:
    main = repo / "sdk/lib/rust" / f"{target}-cargo/src/main.rs"
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
        crate = repo / "sdk/lib/rust" / f"{t}-cargo"
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
    destination=None,
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
            # i386 cells build core with `-Z build-std-features=compiler-builtins-mem`, so the
            # mem/str intrinsics resolve statically instead of importing the CRT's (userinit
            # and lsass have linked that way since June). Accepted as a removal, i386 only.
            builtins = {
                d
                for d in diffs
                if a == "i386"
                and d.startswith("imports: - ")
                and d.split("!", 1)[-1] in I386_BUILTIN_MEM
            }
            dead = {d for d in diffs if d.removeprefix("imports: - ") in DEAD_IMPORTS.get(t, ())}
            # A panic-lean cell links only what its body, CRT and exports reach; cmake's
            # link also kept whatever shared an archive member with the body's panic
            # path. An import can only vanish when no linked code references it, so the
            # removals are that unreachable code. Reported, never silent.
            lean = (
                (
                    {d for d in diffs if d.startswith("imports: - ")}
                    if not _cell_uses_panic_common(t, a, repo)
                    else set()
                )
                - builtins
                - dead
            )
            diffs = [d for d in diffs if d not in extra | builtins | probes | dead | lean]
            # Size is not part of the contract, but a cell must not carry code cmake's
            # link did not: that is how all of core rode into 94 modules unnoticed.
            grew = pe_equiv.load(new).mapped_size() - pe_equiv.load(olds[0]).mapped_size()
            second_core = a == "i386" and _cell_uses_panic_common(t, a, repo)
            slack = SIZE_SLACK + (I386_SECOND_CORE if second_core else 0)
            if grew > slack:
                diffs.append(f"size: mapped image grew {grew // 1024} KB (slack {slack // 1024})")
            note = " (+ panic_common imports)" if extra else ""
            note += " (mem intrinsics from compiler-builtins)" if builtins else ""
            note += " (+ __chkstk: frame pointers)" if probes else ""
            note += " (- dead-code import, see DEAD_IMPORTS)" if dead else ""
            note += f" (- {len(lean)} imports only the panic path reached)" if lean else ""
            if second_core and grew > SIZE_SLACK:
                note += f" (+{grew // 1024} KB: i386 second core)"
            print(f"{t:14} {a:6} {'EQUIVALENT' + note if not diffs else 'DIFFERENT'}")
            for d in diffs[:12]:
                print(f"    {d}")
            bad += bool(diffs)
    print(f"\n{bad} problem(s)")
    return 1 if bad else 0


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
        "--reshell",
        type=Path,
        metavar="DIR",
        help="re-choose existing cells' panic handler from the cmake-built PEs in DIR",
    )
    args = ap.parse_args(argv)
    if args.save_baseline:
        index_path = args.save_baseline / "targets.json"
        index = json.loads(index_path.read_text()) if index_path.exists() else {}
        for t in args.targets:
            for a in ARCHES:
                out = link_edge(a, t).output
                dst = args.save_baseline / a / out
                dst.parent.mkdir(parents=True, exist_ok=True)
                dst.write_bytes((REPO / f"build-{a}" / out).read_bytes())
                index.setdefault(t, {})[a] = out
        index_path.parent.mkdir(parents=True, exist_ok=True)
        index_path.write_text(json.dumps(index, indent=1, sort_keys=True))
        print(f"saved {len(args.targets)} modules x {len(ARCHES)} arches")
        return 0
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
                print(f"== {t}-cargo/build.rs\n{build_rs(cell)}")
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
