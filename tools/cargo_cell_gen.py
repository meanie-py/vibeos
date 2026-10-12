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
}
# The engine builds a body with `-C force-frame-pointers=yes` (stack walks need them) where
# cmake's rust_staticlib_variant did not; a frame pointer can push a big frame over the
# probe threshold. ntdll exports __chkstk on amd64/arm64 and kernel32 forwards it there.
STACK_PROBE_IMPORTS = {"kernel32.dll!__chkstk", "ntdll.dll!__chkstk"}
# What compiler-builtins' `mem` feature defines on i386 (see check()).
I386_BUILTIN_MEM = {"memcmp", "memcpy", "memmove", "memset", "strlen", "bcmp"}
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
        rf"^build (\S+): (C_\w+?)_LINKER__{re.escape(target)}_\S*( .*)?\n((?:  .*\n)+)",
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


def cmake_facts(cmake: Path, target: str) -> dict:
    text = cmake.read_text()
    facts: dict = {
        "destination": None,
        "name_on_cd": None,
        "body": None,
        "unknown": [],
        "unicode": False,
    }
    for _s, _e, cmd, raw in _statements(text):
        a = _args(raw)
        if cmd == "rust_staticlib_variant" and len(a) >= 4:
            name = a[0]
            if f"{target}_rust" == name or name.removesuffix("_rust") == target:
                feats = a[2].strip('"').replace(";", " ").split()
                facts["body"] = (a[1], feats, a[3].upper() == "TRUE")
        if not a or (a[0] != target and not (cmd == "add_cd_file" and target in a)):
            continue
        if cmd == "set_module_type" and "UNICODE" in a:
            facts["unicode"] = True
        if cmd == "add_cd_file":
            if "DESTINATION" in a:
                facts["destination"] = a[a.index("DESTINATION") + 1]
            if "NAME_ON_CD" in a:
                facts["name_on_cd"] = a[a.index("NAME_ON_CD") + 1]
        elif cmd not in REPLACED:
            facts["unknown"].append(cmd)
    return facts


def rewrite_cmake(cmake: Path, cell: Cell) -> str:
    """The module's CMakeLists with the target's link statements replaced by the cell."""
    text = cmake.read_text()
    keep, pos = [], 0
    body_lib = None
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
        if cmd == "rust_staticlib_variant" and a and a[0].removesuffix("_rust") == cell.target:
            drop, body_lib = True, a[0]
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
    if body_lib and re.search(rf"\$\{{{body_lib}_LIB\}}|rust_{body_lib}\b", out):
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
        block.append(
            f"add_library({cell.target}_c OBJECT {' '.join(Path(c).name for c in cell.c_shims)})"
        )
        if cell.rc_defines:
            block.append(
                f"target_compile_definitions({cell.target}_c PRIVATE {' '.join(cell.rc_defines)})"
            )
        extra.append(f"{cell.target}_c")
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
    facts = cmake_facts(cmake, target)
    if facts["unknown"]:
        raise GenError(f"{target}: unmodelled statements on the target: {facts['unknown']}")
    if not facts["body"]:
        raise GenError(f"{target}: no rust_staticlib_variant names its body crate")
    if first.kind == "C_EXECUTABLE":
        if not (first.subsystem or "").startswith(("windows", "console")):
            raise GenError(f"{target}: {first.subsystem} exe is not modelled (native?)")
        module_kind = "Win32Exe"
    elif first.kind in ("C_MODULE_LIBRARY", "C_SHARED_LIBRARY"):
        module_kind = "Win32Dll"
    else:
        raise GenError(f"{target}: link kind {first.kind}")
    specs = spec_index(repo)
    body_ar = f"lib{target}_rust.a"
    per_arch: dict[str, tuple] = {}
    for a in ARCHES:
        imports, static, msvcrtex = [], [], None
        for lib in links[a].libs:
            name = lib.split("/")[-1]
            # Only the module's OWN body is the cell's rust_staticlib; another crate's
            # `_rust.a` (subst's libconutils_rust.a) is a library like any other.
            if name == body_ar or "libclang_rt" in lib:
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
    # set_module_type(... UNICODE) is the one target-level define a user module's
    # compiles get; the OBJECT libraries that replace the target must carry it.
    rc_defines = ["UNICODE", "_UNICODE"] if facts["unicode"] else []
    if module_kind == "Win32Exe" and not msvcrtex:
        raise GenError(f"{target}: exe without a msvcrtex CRT startup")
    stacks = {links[a].stack for a in ARCHES}
    if len(stacks) > 1:
        raise GenError(f"{target}: stack reserve differs by arch {stacks}")
    body, feats, defaults = facts["body"]
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
    if any(cell.static_libs.values()):
        L.append("    let static_libs: &[&str] = match arch.as_str() {")
        for a in ARCHES:
            libs = ", ".join(f'"{x}"' for x in cell.static_libs[a])
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
        # Replays cmake's own compile of each file (the <m>_c OBJECT library), so the C
        # keeps the exact flags it had: compiled with the engine's c_shims flags instead,
        # expand's varargs shim grew an arm64 __chkstk import.
        L.append(
            f'        .c_from_compile_db("{cell.target}_c", &['
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
    if any(cell.static_libs.values()):
        L.append("        // cmake-built static libraries, reused until each has a cargo home.")
        L.append("        .static_libs(static_libs)")
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

[dependencies]
panic-common = {{ path = "../panic-common", features = ["user"] }}
[build-dependencies]
reactos-link = {{ path = "../reactos-link" }}
"""


MAIN_RS = """//! Empty bin shell: the PE's entry is the CRT startup or DllMain in the linked body,
//! and nothing here runs. Cargo needs a `main.rs` to drive the link.

#![no_std]
#![no_main]
#![allow(internal_features)]

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
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
    (crate / "src/main.rs").write_text(MAIN_RS)
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


def check(targets: list[str], baseline: Path, repo: Path = REPO) -> int:
    import pe_equiv

    bad = 0
    for t in targets:
        for a in ARCHES:
            olds = list((baseline / a).rglob(f"{t}.*"))
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
            diffs = [d for d in diffs if d not in extra | builtins | probes]
            note = " (+ panic_common imports)" if extra else ""
            note += " (mem intrinsics from compiler-builtins)" if builtins else ""
            note += " (+ __chkstk: frame pointers)" if probes else ""
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
    args = ap.parse_args(argv)
    if args.save_baseline:
        for t in args.targets:
            for a in ARCHES:
                out = link_edge(a, t).output
                dst = args.save_baseline / a / out
                dst.parent.mkdir(parents=True, exist_ok=True)
                dst.write_bytes((REPO / f"build-{a}" / out).read_bytes())
        print(f"saved {len(args.targets)} modules x {len(ARCHES)} arches")
        return 0
    if args.check:
        return check(args.targets, args.check)
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
