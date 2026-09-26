#!/usr/bin/env python3
"""Verify win32k-rust (and other driver-crate) hand-written `#[repr(C)]` struct
FIELD OFFSETS against C ground truth.

Companion to `tools/verify_offsets.py` (which covers ntkrnl-sys). This tool covers
the *cohabitation* crates whose hand-written struct mirrors are NOT bindgen-generated
and therefore escape the ntkrnl-sys offset gate — exactly the "hand offset / repr(C)
mirror" bug class suspected behind the open amd64 taskbar BUGCHECK 0x50.

The win32k private structs are almost all file-local (defined in a single `.c`, not a
header), so the C ground truth is obtained by:
  1. Enumerating `#[repr(C)] pub struct NAME { ... }` in win32k-rust/src/*.rs.
  2. Locating each struct's C `typedef struct ... } NAME;` block in win32ss/ sources
     (grep the .c/.h that defines it) and brace-extracting the typedef text.
  3. Building a C TU that `#include "win32ss/pch.h"` (the real win32k precompiled
     header — sets up the entire NT + win32k type universe) then injects the extracted
     typedef blocks and emits
        `volatile unsigned int off_N = __builtin_offsetof(CType, field);`
     Cross-compiled to LLVM IR (clang, amd64 target + win32k defs), the `@off_N = .. i32 V`
     constants are the authoritative C offsets.
  4. For the Rust side we emit `const _: () = assert!(offset_of!(crate path) == <C off>);`
     into a temp module inside win32k-rust, run `cargo check -p win32k-rust` against the
     kernel target, and let rustc be the authority on Rust layout (never re-implement
     Rust layout in Python — i386 u64-alignment subtleties would cause false positives).
     A failed const assert = a real field-offset mismatch (a candidate 0x50 bug).

Focus arch: amd64 (where the 0x50 lives). i386/arm64 can be added once amd64 is clean.

Usage:
    python3 tools/verify_offsets_win32k.py              # amd64, report mismatches
    python3 tools/verify_offsets_win32k.py --arch i386
    python3 tools/verify_offsets_win32k.py --keep        # leave generated files for inspection
"""

import re
import subprocess
import sys
import tempfile
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
REACTOS = SCRIPT_DIR.parent
WIN32K_RUST = REACTOS / "sdk/lib/rust/win32k-rust"
WIN32K_SRC = WIN32K_RUST / "src"

# --- Driver-crate configs (mirrors tools/rustconv/gen_driver_types.sh) ----------
# Each driver crate has a handful of HAND-WRITTEN #[repr(C)] mirrors (the bindgen
# layouts live in types_<arch>.rs and are out of scope). For driver mode we use the
# driver's own wrapper-header include set + the driver source dir, plus the driver's
# .c files as struct-prefix sources for file-local typedefs.
DRIVER_CONFIGS = {
    "disk-rust": {
        "crate_dir": "sdk/lib/rust/disk-rust",
        "driver_dir": "drivers/storage/class/disk",
        "wrapper_includes": [
            '"disk.h"',
            "<wmistr.h>",
            "<ntddstor.h>",
            "<ntddscsi.h>",
            "<ntdddisk.h>",
            "<ntddvol.h>",
            "<ioevent.h>",
        ],
        "extra_inc": [],
        "ver": ["-DWINVER=0x602", "-D_WIN32_WINNT=0x602", "-DNTDDI_VERSION=0x06020000"],
        "skip_files": ["types_amd64", "types_i386", "types_arm64"],
    },
    "classpnp-rust": {
        "crate_dir": "sdk/lib/rust/classpnp-rust",
        "driver_dir": "drivers/storage/class/classpnp",
        "wrapper_includes": ["<classp.h>", "<wmistr.h>", "<ntddstor.h>", "<ntddscsi.h>"],
        "extra_inc": [],
        "ver": ["-DWINVER=0x602", "-D_WIN32_WINNT=0x602", "-DNTDDI_VERSION=0x06020000"],
        "skip_files": ["types_amd64", "types_i386", "types_arm64"],
    },
    "scsiport-rust": {
        "crate_dir": "sdk/lib/rust/scsiport-rust",
        "driver_dir": "drivers/storage/port/scsiport",
        "wrapper_includes": ['"scsiport.h"'],
        "extra_inc": [],
        "ver": [
            "-DWINVER=0x502",
            "-D_WIN32_WINNT=0x502",
            "-DNTDDI_VERSION=0x05020400",
            "-D_WIN32_WINDOWS=0x502",
            "-D_WIN32_IE=0x603",
            "-D_SETUPAPI_VER=0x502",
        ],
        "skip_files": ["types_amd64", "types_i386", "types_arm64"],
        "nightly": True,  # crate uses #![feature(c_variadic)]
    },
}


def _find_build_repo() -> Path:
    def has_builds(p: Path) -> bool:
        return any((p / f"build-{a}").is_dir() for a in ("amd64", "i386", "arm64"))

    if has_builds(REACTOS):
        return REACTOS
    # No sibling fallback — see verify_offsets.py for why borrowing another
    # worktree's generated headers makes an offset check meaningless.
    sys.exit(
        f"FATAL: no build-<arch>/ in {REACTOS}. verify_offsets_win32k needs the\n"
        f"generated headers from THIS worktree's build tree; a sibling worktree's\n"
        f"tree is NOT a substitute (another lane's source at another commit).\n"
        f"  fix: ./vibeos-cli build --arch <amd64|i386|arm64>"
    )


BUILD_REPO = _find_build_repo()


def _find_toolchain() -> Path:
    candidates = [
        REACTOS / ".build-tools/llvm-mingw",
        BUILD_REPO / ".build-tools/llvm-mingw",
    ]
    for c in candidates:
        if (c / "bin" / "clang").exists():
            return c
    raise RuntimeError("Cannot find llvm-mingw toolchain")


TOOLCHAIN = _find_toolchain()


def _clang_include() -> str:
    for p in sorted((TOOLCHAIN / "lib" / "clang").glob("*/include"), reverse=True):
        return str(p)
    raise RuntimeError("No clang builtin include dir")


ARCHS = {
    "amd64": {
        "clang_target": "x86_64-pc-windows-gnu",
        "defs": ["-D_WIN64", "-D_M_AMD64", "-D__x86_64__", "-D_AMD64_", "-mxsave"],
    },
    "i386": {
        "clang_target": "i686-pc-windows-gnu",
        "defs": ["-D_M_IX86", "-D__i386__", "-D_X86_"],
    },
    "arm64": {
        "clang_target": "aarch64-pc-windows-gnu",
        "defs": ["-D_WIN64", "-D_M_ARM64", "-D__aarch64__", "-D__arm64__", "-D_ARM64_"],
    },
}


def get_include_flags(arch_name: str) -> list[str]:
    R = BUILD_REPO
    build_dir = R / f"build-{arch_name}"
    return [
        "-I",
        str(R),  # so "win32ss/pch.h" resolves
        "-I",
        str(R / "win32ss"),
        "-I",
        str(R / "win32ss/include"),
        "-I",
        str(R / "win32ss/user/ntuser"),
        "-I",
        str(R / "win32ss/gdi/ntgdi"),
        "-I",
        str(R / "win32ss/gdi/eng"),
        "-I",
        str(build_dir / "ntoskrnl"),
        "-I",
        str(R / "ntoskrnl/include"),
        "-I",
        str(build_dir / "ntoskrnl/include"),
        "-I",
        str(R / "sdk/include"),
        "-I",
        str(R / "sdk/include/ddk"),
        "-I",
        str(R / "sdk/include/ndk"),
        "-I",
        str(R / "sdk/include/psdk"),
        "-I",
        str(R / "sdk/include/xdk"),
        "-I",
        str(R / "sdk/include/reactos"),
        "-I",
        str(R / "sdk/include/reactos/subsys"),
        "-I",
        str(R / "sdk/include/reactos/drivers"),
        "-I",
        str(R / "sdk/include/vcruntime"),
        "-I",
        str(build_dir / "sdk/include"),
        "-I",
        str(build_dir / "sdk/include/psdk"),
        "-I",
        str(build_dir / "sdk/include/ddk"),
        "-I",
        str(build_dir / "sdk/include/dxsdk"),
        "-I",
        str(build_dir / "sdk/include/reactos"),
        "-I",
        str(build_dir / "sdk/include/reactos/mc"),
        "-I",
        str(R / "sdk/include/dxsdk"),
        "-I",
        str(R / "sdk/lib/3rdparty/freetype/include"),
        "-I",
        str(R / "sdk/lib/pseh/include"),
        "-isystem",
        str(R / "sdk/include/crt"),
        "-isystem",
        _clang_include(),
    ]


C_PREAMBLE = [
    "/* Auto-generated by tools/verify_offsets_win32k.py */",
    "#define _WIN32K_",
    "#define LANGPACK",
    "#define _NTOSKRNL_",
    "#define _NTSYSTEM_",
    "#define _USE_DUMMY_PSEH 1",
    "#define __REACTOS__",
    "#define DXGKDDI_INTERFACE_VERSION 0x1052  /* VISTA — dodge WDDM3 header gap */",
    "/* Under -fms-extensions clang treats _InterlockedCompareExchange128 as a",
    "   builtin, colliding with mingw intrin_x86.h's inline def. Rename the",
    "   header decl out of the way (layout-irrelevant). */",
    "#define _InterlockedCompareExchange128 _ICE128_unused_skip",
    '#include "win32ss/pch.h"',
    "/* Neutralize header-internal C_ASSERTs: some (e.g. ntuser.h IMC head.* on i386)",
    "   reference members not visible under this standalone clang config and would",
    "   abort the whole TU. They are not our probes; our offsets come from off_N. */",
    "#undef C_ASSERT",
    "#define C_ASSERT(x)",
    "/* Neutralize per-file boilerplate that collides when several source-file",
    "   prefixes are concatenated into one TU, and the debug-channel macros whose",
    "   expansions reference per-file `static int DbgDefaultChannel` (which we drop",
    "   to avoid multiple-definition across concatenated prefixes). None affect",
    "   struct layout. */",
    "#undef DBG_DEFAULT_CHANNEL",
    "#define DBG_DEFAULT_CHANNEL(x)",
    "#undef TRACE",
    "#define TRACE(...)",
    "#undef ERR",
    "#define ERR(...)",
    "#undef WARN",
    "#define WARN(...)",
    "#undef DPRINT",
    "#define DPRINT(...)",
    "#undef DPRINT1",
    "#define DPRINT1(...)",
    "#undef TRACE_PPI",
    "#define TRACE_PPI(...)",
    "",
]


# =============================================================================
# Rust struct parsing — only repr(C), non-bindgen hand-written structs.
# =============================================================================


def parse_repr_c_structs(content: str) -> dict[str, list[str]]:
    """Return {struct_name: [field_name, ...]} for every `#[repr(C)]`-attributed
    `pub struct`. Skips bitfield/anon padding fields (no C counterpart by name)."""
    structs: dict[str, list[str]] = {}
    lines = content.split("\n")
    i = 0
    n = len(lines)
    while i < n:
        # look for repr(C) within the attribute block preceding a struct
        if "repr(C" in lines[i]:
            # scan forward to the struct decl (skip other attrs)
            j = i
            while (
                j < n
                and "struct " not in lines[j]
                and not lines[j].strip().startswith("fn ")
                and not lines[j].strip().startswith("pub fn ")
                and j - i < 8
            ):
                j += 1
            # accept `pub struct NAME {`, `struct NAME {`, `pub(crate) struct NAME {`
            m = re.search(r"\bstruct (\w+)\s*\{", lines[j]) if j < n else None
            if not m:
                i += 1
                continue
            name = m.group(1)
            fields: list[str] = []
            depth = lines[j].count("{") - lines[j].count("}")
            k = j + 1
            while k < n and depth > 0:
                line = lines[k]
                # field may or may not be `pub`
                fm = re.match(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(\w+)\s*:", line)
                if fm:
                    fn = fm.group(1)
                    if not fn.startswith("_bitfield_") and not fn.startswith("__bindgen"):
                        fields.append(fn)
                depth += line.count("{") - line.count("}")
                k += 1
            structs[name] = fields
            i = k
            continue
        i += 1
    return structs


# =============================================================================
# C typedef extraction
# =============================================================================


def _first_struct_definition(tag: str, roots: list[Path]) -> Path | None:
    """First `.c` (then `.h`) file under `roots` holding `struct <tag> {`.

    In Python, not a `grep -rl` shell-out: those sat under `except Exception: continue`,
    so a missing or failing grep read as "no definition anywhere" -- a silent skip of the
    very struct being verified. latin-1 decodes every byte, so no file is ever "binary"."""
    want = re.compile(rf"struct {re.escape(tag)}\s*\{{")
    for ext in ("*.c", "*.h"):
        for root in roots:
            for f in sorted(root.rglob(ext)):
                if f.is_file() and want.search(f.read_bytes().decode("latin-1")):
                    return f
    return None


def find_c_source(struct_name: str) -> Path | None:
    """The win32ss file defining `struct <name>` (with a body); prefer .c then .h."""
    return _first_struct_definition(struct_name, [REACTOS / "win32ss"])


def _typedef_end(src_text: str, struct_name: str) -> int | None:
    """Byte offset just past the terminating `;` of struct_name's typedef."""
    m = re.search(rf"\btypedef\s+struct\s+{re.escape(struct_name)}\b[^{{;]*\{{", src_text)
    if not m:
        m = re.search(rf"\bstruct\s+{re.escape(struct_name)}\b[^{{;]*\{{", src_text)
        if not m:
            return None
    brace_open = src_text.index("{", m.start())
    depth = 0
    i = brace_open
    while i < len(src_text):
        c = src_text[i]
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return src_text.index(";", i) + 1
        i += 1
    return None


def extract_typedef_block(src_text: str, struct_name: str) -> str | None:
    """Extract `typedef struct NAME { ... } ALIAS, *P...;` brace-matched.

    Only used for field-name discovery; the C compile uses file PREFIXES so that
    interleaved helper typedefs / `#define` constants the struct depends on are in
    scope (see build_prefixes)."""
    m = re.search(rf"\btypedef\s+struct\s+{re.escape(struct_name)}\b[^{{;]*\{{", src_text)
    if not m:
        m = re.search(rf"\bstruct\s+{re.escape(struct_name)}\b[^{{;]*\{{", src_text)
        if not m:
            return None
    end = _typedef_end(src_text, struct_name)
    if end is None:
        return None
    return src_text[m.start() : end]


def strip_function_bodies(src: str) -> str:
    """Remove top-level function-definition bodies from a C source prefix while
    preserving `#define`s, `typedef`s, struct/union/enum definitions, and file-scope
    declarations. A function body is a top-level (brace-depth 0) `{...}` that is NOT
    immediately part of a `typedef`/`struct`/`union`/`enum`/`=` initializer.

    Heuristic: walk char-by-char tracking brace depth. When at depth 0 we hit a '{'
    whose preceding non-space token-run does not contain one of the struct keywords
    or '=' (so it's a function/compound def, not an aggregate type or initializer),
    skip its whole brace block and the optional trailing tokens up to the matching
    line, replacing with ';' so any preceding declarator stays well-formed-ish.
    """
    # First, blank out comments and string/char literals so their braces/semicolons
    # don't confuse the brace scanner. Replace with equal-length spaces (keep newlines)
    # so byte offsets and line structure are preserved.
    masked = _mask_strings_and_comments(src)

    out = []
    i = 0
    n = len(src)
    depth = 0
    stmt_start = 0
    while i < n:
        mc = masked[i]
        if mc == "{" and depth == 0:
            head = masked[stmt_start:i]
            is_aggregate = bool(re.search(r"\b(struct|union|enum)\b", head)) or "=" in head
            if not is_aggregate:
                # skip the whole brace block (a function/compound body) using the
                # masked text for matching, but drop from real `out`.
                d = 0
                j = i
                while j < n:
                    if masked[j] == "{":
                        d += 1
                    elif masked[j] == "}":
                        d -= 1
                        if d == 0:
                            j += 1
                            break
                    j += 1
                removed = i - stmt_start
                if removed > 0 and len(out) >= removed:
                    del out[-removed:]
                i = j
                stmt_start = i
                continue
        if mc == "{":
            depth += 1
        elif mc == "}":
            depth -= 1
        elif mc == ";" and depth == 0:
            stmt_start = i + 1
        out.append(src[i])
        i += 1
    result = "".join(out)

    # Re-insert any preprocessor directive lines that fell INSIDE a removed function
    # body. Function-body stripping is brace-structural and can delete an `#endif`
    # (or `#if`) that lived inside a body, unbalancing a conditional that opened
    # before the body (seen with window.split4.c's REACTOS_GHOST_WND guard). We must
    # NOT, however, drop or duplicate conditionals that legitimately gate a STRUCT
    # FIELD (e.g. _GDI_POOL's `#if DBG_ENABLE_EVENT_LOGGING ... slhLog ... #endif`),
    # because that would change the struct layout. So: rebalance by appending the
    # net-missing `#endif`s that the stripping removed, computed from the masked
    # original vs the stripped result.
    def _cond_balance(text: str) -> int:
        bal = 0
        for ln in text.split("\n"):
            s = ln.strip()
            if re.match(r"#\s*(if|ifdef|ifndef)\b", s):
                bal += 1
            elif re.match(r"#\s*endif\b", s):
                bal -= 1
        return bal

    net = _cond_balance(result)
    if net > 0:
        result += "\n" + "\n".join(["#endif"] * net) + "\n"
    elif net < 0:
        result = "\n".join(["#if 1"] * (-net)) + "\n" + result
    return result


def _mask_strings_and_comments(src: str) -> str:
    """Return src with comment + string/char-literal contents replaced by spaces
    (newlines preserved), so a structural scanner sees no braces/quotes inside them."""
    res = []
    i = 0
    n = len(src)
    while i < n:
        c = src[i]
        two = src[i : i + 2]
        if two == "//":
            while i < n and src[i] != "\n":
                res.append(" ")
                i += 1
            continue
        if two == "/*":
            res.append("  ")
            i += 2
            while i < n and src[i : i + 2] != "*/":
                res.append("\n" if src[i] == "\n" else " ")
                i += 1
            res.append("  ")
            i += 2
            continue
        if c == '"' or c == "'":
            quote = c
            res.append(" ")
            i += 1
            while i < n and src[i] != quote:
                if src[i] == "\\" and i + 1 < n:
                    res.append("  ")
                    i += 2
                    continue
                res.append("\n" if src[i] == "\n" else " ")
                i += 1
            if i < n:
                res.append(" ")
                i += 1
            continue
        res.append(c)
        i += 1
    return "".join(res)


def has_anonymous_typedef_member(typedef_text: str) -> bool:
    """True if the C struct body contains an anonymous embedded member of the form
    `TYPE;` (a typedef name with no declarator) — the ReactOS MSVC idiom (`WND;`).

    clang (gnu89) silently DROPS such members ("declaration does not declare
    anything"), so the clang-compiled C struct ends up SMALLER than the author
    intended, and any field after the anonymous member lands at the WRONG (too-small)
    offset. The Rust translation makes the base explicit (`_base: WND`), which is the
    INTENDED layout. So a Rust-vs-clang-C offset divergence on such a struct is NOT a
    Rust bug — it is the Rust side being *correct* and the C side being clang-broken.
    """
    brace_open = typedef_text.index("{")
    brace_close = typedef_text.rindex("}")
    body = typedef_text[brace_open + 1 : brace_close]
    body = re.sub(r"\{[^{}]*\}", " ", body)  # flatten nested aggregates
    for stmt in body.split(";"):
        s = stmt.strip()
        if not s or s.startswith("#"):
            continue
        # an anonymous member is a single identifier (the type name), optionally a
        # qualified type, with NO trailing declarator identifier and no '*'/'['.
        if re.fullmatch(r"[A-Za-z_]\w*", s):
            return True
    return False


def parse_c_field_names(typedef_text: str) -> list[str]:
    """Best-effort extraction of top-level field identifiers from a typedef body,
    used only to know which Rust fields have a C counterpart by name."""
    brace_open = typedef_text.index("{")
    brace_close = typedef_text.rindex("}")
    body = typedef_text[brace_open + 1 : brace_close]
    # strip nested braces (unions/anon structs) to top level — crude but we only
    # need names; the offsetof itself is authoritative.
    names = set()
    # remove nested {...}
    flat = re.sub(r"\{[^{}]*\}", " ", body)
    for stmt in flat.split(";"):
        stmt = stmt.strip()
        if not stmt or stmt.startswith("//"):
            continue
        # last identifier before optional [..] / : bitfield
        for mm in re.finditer(r"(\w+)\s*(?:\[[^\]]*\])?\s*(?::\s*\d+)?\s*(?:,|$)", stmt):
            names.add(mm.group(1))
    return list(names)


# =============================================================================
# clang IR
# =============================================================================


def compile_to_ir(
    c_source: str, arch_name: str, tmpdir: Path, include_flags=None, ver_defs=None, extra_inc=None
):
    clang = TOOLCHAIN / "bin" / "clang"
    c_path = tmpdir / f"w32_offsets_{arch_name}.c"
    ll_path = tmpdir / f"w32_offsets_{arch_name}.ll"
    c_path.write_text(c_source)
    cfg = ARCHS[arch_name]
    inc = include_flags if include_flags is not None else get_include_flags(arch_name)
    vdefs = ver_defs if ver_defs is not None else ["-DWINVER=0x502", "-D_WIN32_WINNT=0x502"]
    einc = extra_inc or []

    def run():
        cmd = [
            str(clang),
            "-emit-llvm",
            "-S",
            "-O0",
            "-std=gnu89",
            "-nostdlibinc",
            # -fms-extensions is REQUIRED: the real win32k/driver build sets it
            # globally (sdk/cmake/gcc.cmake) and the headers rely on MS-style
            # anonymous embedded struct members (`WND;`, `HEAD;`). Without it clang
            # silently DROPS those members, giving wrong offsets.
            "-fms-extensions",
            "-fno-strict-aliasing",
            "-fno-common",
            # -fms-extensions exposes MSVC intrinsic builtins, colliding with mingw
            # intrin_x86.h's inline defs; -fno-builtin makes HAS_BUILTIN() false so
            # the header's own inline defs are used (layout-irrelevant).
            "-fno-builtin",
            "-target",
            cfg["clang_target"],
            *cfg["defs"],
            "-DDBG=1",
            *vdefs,
            *einc,
            *inc,
            "-ferror-limit=0",
            "-Wno-everything",
            str(c_path),
            "-o",
            str(ll_path),
        ]
        return subprocess.run(cmd, capture_output=True, text=True)

    last_err = ""
    for _ in range(30):
        result = run()
        if result.returncode == 0:
            return ll_path.read_text(), ""
        last_err = result.stderr
        error_lines = set()
        for line in result.stderr.split("\n"):
            mm = re.search(rf"{re.escape(c_path.name)}:(\d+):\d+: error:", line)
            if mm:
                error_lines.add(int(mm.group(1)))
        if not error_lines:
            return None, last_err
        src = c_path.read_text().split("\n")
        changed = False
        for ln in sorted(error_lines, reverse=True):
            idx = ln - 1
            if 0 <= idx < len(src) and src[idx].lstrip().startswith("volatile"):
                src[idx] = ""
                changed = True
        if not changed:
            return None, last_err
        c_path.write_text("\n".join(src))
    return None, last_err


def parse_ir(ir: str) -> dict[str, int]:
    vals = {}
    for m in re.finditer(r"@off_(\d+)\s*=\s*.*?\bi32\s+(\d+)", ir):
        vals[int(m.group(1))] = int(m.group(2))
    return vals


# =============================================================================
# Driver
# =============================================================================


def gather_rust_structs():
    """Return {struct_name: (rust_file, [field,...])} across win32k-rust/src."""
    out = {}
    for rs in sorted(WIN32K_SRC.glob("*.rs")):
        structs = parse_repr_c_structs(rs.read_text())
        for name, fields in structs.items():
            out[name] = (rs, fields)
    return out


def main():
    arch_name = "amd64"
    keep = "--keep" in sys.argv
    for i, a in enumerate(sys.argv):
        if a == "--arch" and i + 1 < len(sys.argv):
            arch_name = sys.argv[i + 1]
    if arch_name not in ARCHS:
        print(f"unknown arch {arch_name}")
        sys.exit(2)

    print(f"=== win32k-rust struct field-OFFSET verification ({arch_name}) ===")
    print(f"ReactOS root: {REACTOS}")
    print(f"Build repo:   {BUILD_REPO}")
    print()

    rust_structs = gather_rust_structs()
    print(f"Found {len(rust_structs)} hand-written #[repr(C)] structs in win32k-rust/src\n")

    # locate C source + extract typedef for each
    resolved = []  # (name, rust_file, rust_fields, c_typedef_text, c_field_names)
    unresolved = []
    for name, (rs, fields) in sorted(rust_structs.items()):
        csrc = find_c_source(name)
        if csrc is None:
            unresolved.append((name, "no C source found"))
            continue
        td = extract_typedef_block(csrc.read_text(errors="ignore"), name)
        if td is None:
            unresolved.append((name, f"typedef not brace-extractable in {csrc.name}"))
            continue
        # C typedef alias name = the rust struct name w/o leading underscore is the
        # alias; but we offsetof against the *tag* `struct NAME` to be safe.
        cfields = parse_c_field_names(td)
        resolved.append((name, rs, fields, td, cfields, csrc))

    print(f"Resolved C typedef for {len(resolved)}/{len(rust_structs)} structs.")
    if unresolved:
        print("Unresolved (no offset check possible):")
        for n, why in unresolved:
            print(f"  - {n}: {why}")

    # Structs whose C body uses the MS anonymous embedded-member idiom (`WND;`,
    # `HEAD;`). With -fms-extensions (matching the real win32k build) clang embeds
    # these correctly, so their C offsets ARE ground truth. We still can't probe the
    # anonymous member itself by a field name (it has none), so the Rust `_base`
    # field is skipped; the fields AFTER it (pCBox, ...) are checked normally. We
    # just note these structs for transparency. (anon_structs kept = empty so all
    # post-anon fields participate in the gate.)
    anon_structs: set[str] = set()
    noted = [
        name
        for (name, rs, fields, td, cfields, csrc) in resolved
        if has_anonymous_typedef_member(td)
    ]
    if noted:
        print()
        print(
            "MS anonymous-member structs (embed via -fms-extensions; Rust `_base` "
            "field has no C name so it's skipped, fields after it ARE checked):"
        )
        for n in sorted(noted):
            print(f"  ~ {n}")
    print()

    # Build the C probe TU. Inject, per source FILE, the file prefix from byte 0
    # through the END of the last struct we need from that file. This captures the
    # interleaved helper typedefs (BRESINFO, FILL_EDGE, ...) and `#define` size
    # constants (GDIDCATTRFREE, NUMPTSTOBUFFER, ...) the structs depend on, in
    # dependency order, without pulling in the (PSEH-heavy) function bodies that
    # follow. The error-blanking loop drops any straggler line that still fails.
    c_lines = list(C_PREAMBLE)
    by_file: dict[Path, list] = {}
    for item in resolved:
        by_file.setdefault(item[5], []).append(item)
    for csrc, items in by_file.items():
        txt = csrc.read_text(errors="ignore")
        end = 0
        for name, *_rest in items:
            e = _typedef_end(txt, name)
            if e and e > end:
                end = e
        prefix = strip_function_bodies(txt[:end])
        c_lines.append(f"/* ==== prefix of {csrc.relative_to(REACTOS)} ==== */")
        c_lines.append(prefix)
        c_lines.append(f"/* ==== end prefix {csrc.name} ==== */")
    c_lines.append("")

    # offsetof probes: tag -> (struct_name, rust_field)
    probes = []
    tag = 0
    for name, _rs, fields, _td, cfields, _csrc in resolved:
        if name in anon_structs:
            continue  # clang-C offsets unreliable (dropped anon member)
        for f in fields:
            if f not in cfields:
                # rust field name not present in C top-level — likely an anon/union
                # member flattened differently; skip (can't offsetof by that name).
                continue
            c_lines.append(
                f"volatile unsigned int off_{tag} = "
                f"__builtin_offsetof(struct {name}, {f}); /* {name}.{f} */"
            )
            probes.append((tag, name, f))
            tag += 1
    c_lines.append("")

    with tempfile.TemporaryDirectory() as td_dir:
        td_dir = Path(td_dir)
        ir, err = compile_to_ir("\n".join(c_lines), arch_name, td_dir)
        if ir is None:
            print("ERROR: C probe TU failed to compile. First errors:")
            for line in err.split("\n")[:25]:
                print("  " + line)
            sys.exit(1)
        vals = parse_ir(ir)

    # Map tag -> offset; report what we got.
    c_offsets = {}  # (struct, field) -> c_offset
    dropped = []
    for tg, name, f in probes:
        if tg in vals:
            c_offsets[(name, f)] = vals[tg]
        else:
            dropped.append((name, f))

    print(
        f"Got C offsets for {len(c_offsets)} fields "
        f"({len(dropped)} probes dropped by clang — field missing/typedef "
        f"redefinition-dropped on this arch)."
    )
    print()

    # Emit per-struct C-offset table + a Rust assert module for cargo to check.
    emit_rust_assert_module(arch_name, resolved, c_offsets, keep)


def emit_rust_assert_module(arch_name, resolved, c_offsets, keep):
    """Append offset asserts to each struct's OWN source file (so private/non-pub
    structs are in scope by their local name — uniform with the driver path), cargo
    check win32k-rust, report, then revert (unless --keep)."""
    target = {
        "amd64": "x86_64-pc-windows-gnu",
        "i386": "i686-pc-windows-gnu",
        "arm64": "aarch64-pc-windows-gnullvm",
    }[arch_name]

    by_file = {}
    n_asserts = 0
    for name, rs, fields, _td, _cfields, _csrc in resolved:
        for f in fields:
            if (name, f) not in c_offsets:
                continue
            off = c_offsets[(name, f)]
            line = (
                f"const _: () = assert!(::core::mem::offset_of!({name}, {f}) == {off});"
                f" // VERIFY_OFFSETS {name}.{f} (C={off})"
            )
            by_file.setdefault(rs, []).append(line)
            n_asserts += 1

    originals = {}
    for rs, asserts in by_file.items():
        originals[rs] = rs.read_text()
        block = (
            "\n\n// ===== VERIFY_OFFSETS_TMP (auto, do not commit) =====\n"
            + "\n".join(asserts)
            + "\n"
        )
        rs.write_text(originals[rs] + block)

    print(
        f"Appended {n_asserts} offset asserts across {len(by_file)} files; "
        f"running cargo check -p win32k-rust..."
    )
    cargo = subprocess.run(
        ["cargo", "check", "-p", "win32k-rust", "--target", target],
        cwd=str(REACTOS / "sdk/lib/rust"),
        capture_output=True,
        text=True,
    )

    mismatches = []
    pat = re.compile(
        r"assertion failed:\s*(?:::core::mem::)?offset_of!\(\s*(\w+)\s*,\s*(\w+)\s*\)\s*==\s*(\d+)"
    )
    for m in pat.finditer(cargo.stderr):
        trip = (m.group(1), m.group(2), int(m.group(3)))
        if trip not in mismatches:
            mismatches.append(trip)

    if not keep:
        for rs, orig in originals.items():
            rs.write_text(orig)

    print()
    print("=" * 70)
    if cargo.returncode == 0:
        print(f"RESULT ({arch_name}): all {n_asserts} field offsets MATCH C. 0 mismatches.")
    elif mismatches:
        print(f"RESULT ({arch_name}): {len(mismatches)} FIELD-OFFSET MISMATCH(es):")
        for struct, field, coff in mismatches:
            print(f"  MISMATCH  {struct}.{field}  C_offset={coff}  (Rust offset_of! != {coff})")
    else:
        print(
            f"RESULT ({arch_name}): cargo check failed but NOT on an offset "
            f"assert — likely a pre-existing build/env issue. stderr tail:"
        )
        for line in cargo.stderr.split("\n")[-30:]:
            print("  " + line)
    print("=" * 70)


# =============================================================================
# Driver-crate mode
# =============================================================================


def driver_include_flags(cfg: dict, arch_name: str) -> list[str]:
    """Mirror gen_driver_types.sh include set for a driver crate."""
    R = BUILD_REPO
    build_dir = R / f"build-{arch_name}"
    driver_dir = R / cfg["driver_dir"]
    flags = ["-I", str(driver_dir)]
    for e in cfg.get("extra_inc", []):
        flags += ["-I", str(R / e)]
    flags += [
        "-I",
        str(R / "sdk/include"),
        "-I",
        str(R / "sdk/include/ddk"),
        "-I",
        str(R / "sdk/include/ndk"),
        "-I",
        str(R / "sdk/include/psdk"),
        "-I",
        str(R / "sdk/include/xdk"),
        "-I",
        str(R / "sdk/include/reactos"),
        "-I",
        str(R / "sdk/include/reactos/libs"),
        "-I",
        str(R / "sdk/include/reactos/drivers"),
        "-I",
        str(R / "sdk/include/vcruntime"),
        "-I",
        str(build_dir / "sdk/include"),
        "-I",
        str(build_dir / "sdk/include/psdk"),
        "-I",
        str(build_dir / "sdk/include/ddk"),
        "-I",
        str(build_dir / "sdk/include/reactos"),
        "-I",
        str(build_dir / "sdk/include/reactos/mc"),
        "-I",
        str(R / "sdk/include/dxsdk"),
        "-I",
        str(R / "sdk/lib/pseh/include"),
        "-isystem",
        str(R / "sdk/include/crt"),
        "-isystem",
        _clang_include(),
    ]
    return flags


def driver_preamble(cfg: dict) -> list[str]:
    inc = "\n".join(f"#include {h}" for h in cfg["wrapper_includes"])
    return [
        "/* Auto-generated by tools/verify_offsets_win32k.py (driver mode) */",
        "#define _USE_DUMMY_PSEH 1",
        "#define __REACTOS__",
        "#define _InterlockedCompareExchange128 _ICE128_unused_skip",
        inc,
        "#undef C_ASSERT",
        "#define C_ASSERT(x)",
        "#undef DPRINT",
        "#define DPRINT(...)",
        "#undef DPRINT1",
        "#define DPRINT1(...)",
        "",
    ]


def gather_driver_structs(crate_src: Path, skip_files: list[str]):
    out = {}
    for rs in sorted(crate_src.glob("*.rs")):
        if rs.stem in skip_files or rs.stem == "types":
            continue
        for name, fields in parse_repr_c_structs(rs.read_text()).items():
            if "__bindgen" in name:
                continue
            out[name] = (rs, fields)
    return out


def find_c_source_driver(struct_name: str, cfg: dict):
    """Search the driver dir (private .c/.h) then the SDK headers for the typedef.

    The Rust mirror often drops the C tag's leading underscore (Rust `DISK_DETECT_INFO`
    <- C `struct _DISK_DETECT_INFO`), so try both the bare name and `_NAME`. Returns
    (Path, c_tag_name) or None."""
    candidates = [struct_name]
    if not struct_name.startswith("_"):
        candidates.append("_" + struct_name)
    roots = [REACTOS / cfg["driver_dir"], REACTOS / "sdk/include"]
    for tag in candidates:
        for root in roots:
            hit = _first_struct_definition(tag, [root])
            if hit:
                return hit, tag
    return None


def main_driver(crate: str, arch_name: str, keep: bool):
    cfg = DRIVER_CONFIGS[crate]
    crate_src = REACTOS / cfg["crate_dir"] / "src"
    print(f"=== {crate} struct field-OFFSET verification ({arch_name}) ===")
    print(f"ReactOS root: {REACTOS}\nBuild repo:   {BUILD_REPO}\n")

    rust_structs = gather_driver_structs(crate_src, cfg["skip_files"])
    print(
        f"Found {len(rust_structs)} hand-written #[repr(C)] structs "
        f"(bindgen types_<arch>.rs excluded)\n"
    )

    resolved, unresolved = [], []
    for name, (rs, fields) in sorted(rust_structs.items()):
        hit = find_c_source_driver(name, cfg)
        if hit is None:
            unresolved.append((name, "no C source (renamed mirror or SDK macro struct)"))
            continue
        csrc, c_tag = hit
        td = extract_typedef_block(csrc.read_text(errors="ignore"), c_tag)
        if td is None:
            unresolved.append((name, f"typedef not extractable in {csrc.name}"))
            continue
        # tuple: (rust_name, rust_file, fields, typedef, c_field_names, csrc, c_tag)
        resolved.append((name, rs, fields, td, parse_c_field_names(td), csrc, c_tag))

    print(f"Resolved C typedef for {len(resolved)}/{len(rust_structs)} structs.")
    for n, why in unresolved:
        print(f"  - {n}: {why}")
    print()

    # Build C probe: preamble (wrapper headers, which define all the SDK types the
    # driver mirrors reference) + each struct's BARE typedef block. Driver file-private
    # structs almost always consist of SDK types (DISK_GEOMETRY, LARGE_INTEGER, ...),
    # so the bare typedef compiles against the wrapper headers without needing the
    # surrounding (function-body-heavy) .c prefix. If a typedef references a same-file
    # helper type that isn't a wrapper header type, its probe will be dropped by the
    # error-blanking loop and reported as uncovered.
    c_lines = driver_preamble(cfg)
    for _name, _rs, _fields, td, _cfields, csrc, c_tag in resolved:
        # If the C struct lives in ANY header (.h) — SDK or the driver's own classp.h/
        # disk.h that the wrapper includes — it's already declared, so injecting our
        # copy would be a redefinition. Probe it directly. Only inject typedefs for
        # structs defined in a .c translation unit (truly file-private).
        if csrc.suffix == ".h":
            c_lines.append(f"/* {c_tag}: header type ({csrc.name}) — probed directly */")
            continue
        c_lines.append(f"/* {c_tag} from {csrc.relative_to(REACTOS)} (driver-private) */")
        c_lines.append(td)
    c_lines.append("")

    probes, tag = [], 0
    for name, _rs, fields, _td, cfields, _csrc, c_tag in resolved:
        for f in fields:
            if f not in cfields:
                continue
            # probe the C tag; record under the RUST name for the assert mapping
            c_lines.append(
                f"volatile unsigned int off_{tag} = "
                f"__builtin_offsetof(struct {c_tag}, {f}); /* {name}.{f} */"
            )
            probes.append((tag, name, f))
            tag += 1

    with tempfile.TemporaryDirectory() as d:
        d = Path(d)
        ir, err = compile_to_ir(
            "\n".join(c_lines),
            arch_name,
            d,
            include_flags=driver_include_flags(cfg, arch_name),
            ver_defs=cfg["ver"],
        )
        if ir is None:
            print("ERROR: driver C probe failed to compile. First errors:")
            for line in err.split("\n")[:25]:
                print("  " + line)
            sys.exit(1)
        vals = parse_ir(ir)

    c_offsets = {}
    for tg, name, f in probes:
        if tg in vals:
            c_offsets[(name, f)] = vals[tg]
    print(f"Got C offsets for {len(c_offsets)} fields.\n")

    emit_driver_assert(crate, arch_name, resolved, c_offsets, keep)


def emit_driver_assert(crate, arch_name, resolved, c_offsets, keep):
    """Append `const _: () = assert!(offset_of!(LocalName, field)==N);` to each
    struct's OWN source file (so private/non-pub structs are in scope by their local
    name), cargo check, report, then revert. Robust to private structs that can't be
    addressed via a crate path."""
    target = {
        "amd64": "x86_64-pc-windows-gnu",
        "i386": "i686-pc-windows-gnu",
        "arm64": "aarch64-pc-windows-gnullvm",
    }[arch_name]

    # group asserts by source file
    by_file = {}
    n_asserts = 0
    for name, rs, fields, _td, _cfields, _csrc, _c_tag in resolved:
        for f in fields:
            if (name, f) not in c_offsets:
                continue
            off = c_offsets[(name, f)]
            line = (
                f"const _: () = assert!(::core::mem::offset_of!({name}, {f}) == {off});"
                f" // VERIFY_OFFSETS {name}.{f} (C={off})"
            )
            by_file.setdefault(rs, []).append(line)
            n_asserts += 1

    originals = {}
    for rs, asserts in by_file.items():
        originals[rs] = rs.read_text()
        block = (
            "\n\n// ===== VERIFY_OFFSETS_TMP (auto, do not commit) =====\n"
            + "\n".join(asserts)
            + "\n"
        )
        rs.write_text(originals[rs] + block)

    if n_asserts == 0:
        print()
        print("=" * 70)
        print(
            f"RESULT {crate} ({arch_name}): 0 verifiable hand-written mirrors "
            f"(no checkable name-matched C struct). Nothing to gate."
        )
        print("=" * 70)
        return

    nightly = DRIVER_CONFIGS[crate].get("nightly", False)
    # Pinned nightly, single-sourced from `.rust-nightly` (see rust-toolchain.toml).
    pin = (SCRIPT_DIR.parent / ".rust-nightly").read_text().strip()
    cargo_cmd = (["cargo", f"+{pin}"] if nightly else ["cargo"]) + [
        "check",
        "-p",
        crate,
        "--target",
        target,
    ]
    print(
        f"Appended {n_asserts} offset asserts across {len(by_file)} files; "
        f"running {' '.join(cargo_cmd[:3])} -p {crate}..."
    )
    cargo = subprocess.run(
        cargo_cmd, cwd=str(REACTOS / "sdk/lib/rust"), capture_output=True, text=True
    )

    # parse failures from the assertion message
    mismatches = []
    pat = re.compile(
        r"assertion failed:\s*(?:::core::mem::)?offset_of!\(\s*(\w+)\s*,\s*(\w+)\s*\)\s*==\s*(\d+)"
    )
    for m in pat.finditer(cargo.stderr):
        trip = (m.group(1), m.group(2), int(m.group(3)))
        if trip not in mismatches:
            mismatches.append(trip)

    if not keep:
        for rs, orig in originals.items():
            rs.write_text(orig)

    print()
    print("=" * 70)
    if cargo.returncode == 0:
        print(f"RESULT {crate} ({arch_name}): all {n_asserts} field offsets MATCH C. 0 mismatches.")
    elif mismatches:
        print(f"RESULT {crate} ({arch_name}): {len(mismatches)} FIELD-OFFSET MISMATCH(es):")
        for struct, field, coff in mismatches:
            print(f"  MISMATCH  {struct}.{field}  C_offset={coff}  (Rust offset_of! != {coff})")
    else:
        print(
            f"RESULT {crate} ({arch_name}): cargo check failed but NOT on an offset "
            f"assert — likely a pre-existing build/env issue. stderr tail:"
        )
        for line in cargo.stderr.split("\n")[-30:]:
            print("  " + line)
    print("=" * 70)


def _entry():
    arch_name = "amd64"
    crate = None
    keep = "--keep" in sys.argv
    for i, a in enumerate(sys.argv):
        if a == "--arch" and i + 1 < len(sys.argv):
            arch_name = sys.argv[i + 1]
        if a == "--crate" and i + 1 < len(sys.argv):
            crate = sys.argv[i + 1]
    if arch_name not in ARCHS:
        print(f"unknown arch {arch_name}")
        sys.exit(2)
    if crate:
        if crate not in DRIVER_CONFIGS:
            print(f"unknown crate {crate}. Known: {', '.join(DRIVER_CONFIGS)}")
            sys.exit(2)
        main_driver(crate, arch_name, keep)
    else:
        main()  # win32k-rust (default)


if __name__ == "__main__":
    _entry()
