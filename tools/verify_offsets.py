#!/usr/bin/env python3
"""Verify Rust struct FIELD OFFSETS (and fixed-array lengths) against C ground truth.

Companion to `sdk/lib/rust/ntkrnl-sys/verify_sizes.py`. Where verify_sizes only
checks total `sizeof`, this tool checks per-field `__builtin_offsetof` — catching
the bug class where a struct has the correct total size but individual fields land
at the wrong offset (e.g. a reordered/dropped field, or a bindgen anon-union mirror
that drifted from the C layout). That field-offset class is behind the open amd64
taskbar BUGCHECK 0x50 / win32k bad-ptr family.

Mechanism (identical recipe to verify_sizes / verify_inline_offsets):
  1. Cover the SAME struct set verify_sizes covers: bindgen `ddk` structs +
     the hand-written FFI list (POOL_HEADER / KDPC / LIST_ENTRY / ...).
  2. For each struct, read its field names from the Rust source (bindgen_<arch>.rs
     for ddk structs, src/<module>.rs for hand-written). Recurse through bindgen
     `__bindgen_anon_N` members so nested-union fields get a real offset_of path.
  3. Emit C `volatile unsigned int off_N = __builtin_offsetof(CType, field);` for
     every field whose name exists in the C struct, plus `sizeof(field)` for fixed
     arrays. Cross-compile C -> LLVM IR (clang, per-arch target+defs) and parse the
     `@off_N = ... i32 VALUE` constants.
  4. Generate per-arch `src/offset_check_<arch>.rs` of
     `const _: () = assert!(core::mem::offset_of!(<path>::NAME, field) == <C off>);`
     wired into lib.rs behind the same cfg as size_check_<arch>.rs. The AUTHORITATIVE
     check is rustc — we never re-implement Rust layout in Python (i386 u64-alignment
     ABI subtleties would cause false positives). A real layout bug = a failed
     compile of the generated assert.

Usage:
    python3 tools/verify_offsets.py            # Generate src/offset_check_<arch>.rs
    python3 tools/verify_offsets.py --check     # Regenerate to temp, diff vs committed
"""

import re
import subprocess
import sys
import tempfile
from pathlib import Path

# tools/verify_offsets.py  ->  repo root is parent of tools/
SCRIPT_DIR = Path(__file__).resolve().parent
REACTOS = SCRIPT_DIR.parent
NTKRNL_SYS = REACTOS / "sdk/lib/rust/ntkrnl-sys"
SRC_DIR = NTKRNL_SYS / "src"  # Rust sources we read/write live in THIS tree.


def _find_build_repo() -> Path:
    """Repo that actually holds the cross-compiled `build-<arch>` dirs.

    The generated headers (ntdef.h etc.) live only under build-<arch>/sdk/include,
    so when running inside a worktree that lacks its own build dirs we fall back to
    a sibling checkout that has them. Includes/build dirs/toolchain are resolved
    against this repo; the Rust src we read/write stays in REACTOS (the worktree).
    """

    def has_builds(p: Path) -> bool:
        return any((p / f"build-{a}").is_dir() for a in ("amd64", "i386", "arm64"))

    if has_builds(REACTOS):
        return REACTOS
    # Deliberately NO sibling fallback. These headers are generated from source at
    # a specific commit, so borrowing another worktree's build tree checks THIS
    # tree's Rust offsets against ANOTHER lane's C layouts — which can green-light
    # a real offset drift or invent a phantom one. Fail loud instead.
    sys.exit(
        f"FATAL: no build-<arch>/ in {REACTOS}. verify_offsets needs the generated\n"
        f"headers from THIS worktree's build tree; a sibling worktree's tree is NOT\n"
        f"a substitute (another lane's source at another commit).\n"
        f"  fix: ./vibeos-cli build --arch <amd64|i386|arm64>"
    )


BUILD_REPO = _find_build_repo()


# --- toolchain detection (in-repo .build-tools first, then build-repo, sibling)
def _find_toolchain() -> Path:
    candidates = [
        REACTOS / ".build-tools/llvm-mingw",
        BUILD_REPO / ".build-tools/llvm-mingw",
        REACTOS.parent / "llvm-mingw",
        REACTOS.parent / "llvm-mingw-20250114-ucrt-ubuntu-20.04-x86_64",
        REACTOS.parent / "llvm-mingw-20250114-ucrt-macos-universal",
    ]
    for c in candidates:
        if (c / "bin" / "clang").exists():
            return c
    raise RuntimeError(
        "Cannot find llvm-mingw toolchain (looked in " + ", ".join(str(c) for c in candidates) + ")"
    )


TOOLCHAIN = _find_toolchain()


def _clang_include() -> str:
    """clang builtin include dir (lib/clang/<ver>/include) — version-globbed."""
    for p in sorted((TOOLCHAIN / "lib" / "clang").glob("*/include"), reverse=True):
        return str(p)
    raise RuntimeError(f"No clang builtin include dir under {TOOLCHAIN}/lib/clang/*/include")


ARCHS = [
    {
        "name": "arm64",
        "clang_target": "aarch64-pc-windows-gnu",
        "defs": ["-D_WIN64", "-D_M_ARM64", "-D__aarch64__", "-D__arm64__", "-D_ARM64_"],
        "rs_file": "bindgen_arm64.rs",
        "rust_target_arch": "aarch64",
        "out_file": "offset_check_arm64.rs",
    },
    {
        "name": "amd64",
        "clang_target": "x86_64-pc-windows-gnu",
        "defs": ["-D_WIN64", "-D_M_AMD64", "-D__x86_64__", "-D_AMD64_", "-mxsave"],
        "rs_file": "bindgen_amd64.rs",
        "rust_target_arch": "x86_64",
        "out_file": "offset_check_amd64.rs",
    },
    {
        "name": "i386",
        "clang_target": "i686-pc-windows-gnu",
        "defs": ["-D_M_IX86", "-D__i386__", "-D_X86_"],
        "rs_file": "bindgen_i386.rs",
        "rust_target_arch": "x86",
        "out_file": "offset_check_i386.rs",
    },
]

# Hand-written FFI structs (rust_struct_name, c_typedef, rust_module) — mirrors
# verify_sizes.extract_hand_written_types(). Field offsets verified against the
# C typedef; Rust path is read from src/<module>.rs.
HAND_WRITTEN = [
    ("POOL_HEADER", "POOL_HEADER", "mm"),
    ("POOL_DESCRIPTOR", "POOL_DESCRIPTOR", "mm"),
    ("POOL_TRACKER_TABLE", "POOL_TRACKER_TABLE", "mm"),
    ("POOL_TRACKER_BIG_PAGES", "POOL_TRACKER_BIG_PAGES", "mm"),
    ("MMFREE_POOL_ENTRY", "MMFREE_POOL_ENTRY", "mm"),
    ("MM_PAGED_POOL_INFO", "MM_PAGED_POOL_INFO", "mm"),
    ("RTL_BITMAP", "RTL_BITMAP", "mm"),
    ("MMPTE", "MMPTE", "mm"),
    ("MMPFN", "MMPFN", "mm"),
    ("KGUARDED_MUTEX", "KGUARDED_MUTEX", "ke"),
    ("KDPC", "KDPC", "ke"),
    ("KLOCK_QUEUE_HANDLE", "KLOCK_QUEUE_HANDLE", "ke"),
    ("PP_LOOKASIDE_LIST", "PP_LOOKASIDE_LIST", "ke"),
    ("GENERAL_LOOKASIDE", "GENERAL_LOOKASIDE", "ke"),
    ("LIST_ENTRY", "LIST_ENTRY", "base"),
    ("SINGLE_LIST_ENTRY", "SINGLE_LIST_ENTRY", "base"),
    ("SLIST_HEADER", "SLIST_HEADER", "base"),
    ("UNICODE_STRING", "UNICODE_STRING", "base"),
    ("SYSTEM_POOLTAG", "SYSTEM_POOLTAG", "ex"),
]

# Same skips verify_sizes uses for bindgen extraction.
SKIP_TYPES = {
    "_GENERAL_LOOKASIDE_POOL",  # complex union, bindgen disagrees with C
    "_LONGDOUBLE",  # x87 long double, platform padding
}


# =============================================================================
# Rust struct parsing
# =============================================================================

FIELD_RE = re.compile(r"^\s*pub\s+(\w+)\s*:\s*(.+?)\s*,?\s*$")


def parse_rust_structs(content: str) -> dict[str, list[tuple[str, str, list[str]]]]:
    """Parse every `pub struct NAME { ... }`.

    Returns {struct_name: [(field_name, field_type, cfg_list), ...]}.
    cfg_list holds any `#[cfg(...)]` predicates attached to the field (so we can
    gate the generated assert identically — e.g. POOL_HEADER's pointer-width
    variants).
    """
    structs: dict[str, list[tuple[str, str, list[str]]]] = {}
    lines = content.split("\n")
    i = 0
    n = len(lines)
    while i < n:
        m = re.match(r"^pub struct (\w+)\s*\{", lines[i])
        if not m:
            i += 1
            continue
        name = m.group(1)
        fields: list[tuple[str, str, list[str]]] = []
        pending_cfgs: list[str] = []
        i += 1
        depth = 1
        while i < n and depth > 0:
            line = lines[i]
            depth += line.count("{") - line.count("}")
            if depth <= 0:
                break
            stripped = line.strip()
            cm = re.match(r"#\[cfg\((.+)\)\]", stripped)
            if cm:
                pending_cfgs.append(cm.group(1))
                i += 1
                continue
            if stripped.startswith("#["):
                # other attribute (doc, etc.) — ignore, keep cfgs
                i += 1
                continue
            fm = FIELD_RE.match(line)
            if fm:
                fields.append((fm.group(1), fm.group(2), list(pending_cfgs)))
                pending_cfgs = []
            elif stripped and not stripped.startswith("//") and not stripped.startswith("///"):
                # non-field content (rare) — reset pending cfgs
                pending_cfgs = []
            i += 1
        structs[name] = fields
    return structs


ARRAY_RE = re.compile(r"^\[\s*(.+?)\s*;\s*(\w+)\s*(?:usize)?\s*\]$")


def is_array_type(rust_type: str) -> bool:
    return rust_type.strip().startswith("[")


def is_bitfield_field(field_name: str) -> bool:
    return field_name.startswith("_bitfield_")


def is_anon_field(field_name: str) -> bool:
    return field_name.startswith("__bindgen_anon_") or field_name.startswith("__bindgen_padding")


def collect_fields(
    struct_name: str,
    structs: dict[str, list[tuple[str, str, list[str]]]],
    prefix: str = "",
    cfg_acc: tuple[str, ...] = (),
    seen: frozenset = frozenset(),
) -> list[tuple[str, str, tuple[str, ...], bool]]:
    """Recursively flatten a Rust struct into (c_field, rust_path, cfgs, is_array).

    - Named non-anon fields become (name, prefix+name).
    - __bindgen_anon_N fields recurse into their struct type, prefixing the path.
    - Bitfield units / padding are skipped (no offsetof / not a real C field name).
    The c_field returned is the leaf field name (what __builtin_offsetof needs);
    the rust_path is the full dotted access path for offset_of!.
    """
    if struct_name in seen:
        return []
    seen = seen | {struct_name}
    out: list[tuple[str, str, tuple[str, ...], bool]] = []
    for fname, ftype, cfgs in structs.get(struct_name, []):
        merged_cfgs = cfg_acc + tuple(cfgs)
        if is_bitfield_field(fname):
            continue
        if is_anon_field(fname):
            # recurse into the nested anon struct type (strip generics/pointers)
            base = (
                re.sub(r"<.*", "", ftype)
                .strip()
                .lstrip("*")
                .replace("mut ", "")
                .replace("const ", "")
                .strip()
            )
            base = base.split()[0] if base else base
            if base in structs:
                out.extend(collect_fields(base, structs, prefix + fname + ".", merged_cfgs, seen))
            continue
        path = prefix + fname
        out.append((fname, path, merged_cfgs, is_array_type(ftype)))
    return out


# =============================================================================
# C generation + clang
# =============================================================================


def get_include_flags(arch_name: str) -> list[str]:
    # Resolve includes against BUILD_REPO — it holds both the source headers and
    # the generated build-<arch>/sdk/include headers (ntdef.h etc.).
    R = BUILD_REPO
    build_dir = R / f"build-{arch_name}"
    return [
        "-I",
        str(build_dir / "ntoskrnl"),
        "-I",
        str(R / "ntoskrnl"),
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
        str(R / "sdk/include/winrt"),
        "-I",
        str(R / "sdk/lib/pseh/include"),
        "-I",
        str(R),
        "-I",
        str(R / "sdk/lib/drivers/arbiter"),
        "-I",
        str(R / "sdk/lib/cmlib"),
        "-I",
        str(R / "ntoskrnl/include"),
        "-I",
        str(build_dir / "ntoskrnl/include"),
        "-I",
        str(build_dir / "ntoskrnl/include/internal"),
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
        "-isystem",
        str(R / "sdk/include/crt"),
        "-isystem",
        _clang_include(),
    ]


C_PREAMBLE = [
    "/* Auto-generated by tools/verify_offsets.py */",
    "#define _NTOSKRNL_",
    "#define _NTSYSTEM_",
    "#define _USE_DUMMY_PSEH 1",
    "#define __REACTOS__",
    "#define _INTRIN_H_",
    "#define _INTRIN_X86_H_",
    "#ifndef __int64",
    "#define __int64 long long",
    "#endif",
    "#undef FORCEINLINE",
    "#define FORCEINLINE",
    "#include <ntoskrnl.h>",
    '#include "mm/ARM3/miarm.h"',
    "",
]


def compile_to_ir(c_source: str, arch: dict, tmpdir: Path) -> str | None:
    clang = TOOLCHAIN / "bin" / "clang"
    c_path = tmpdir / f"offsets_{arch['name']}.c"
    ll_path = tmpdir / f"offsets_{arch['name']}.ll"
    c_path.write_text(c_source)

    def run() -> subprocess.CompletedProcess:
        cmd = [
            str(clang),
            "-emit-llvm",
            "-S",
            "-O0",
            "-std=gnu89",
            "-nostdlibinc",
            "-target",
            arch["clang_target"],
            *arch["defs"],
            "-DDBG=1",
            "-DWINVER=0x502",
            "-D_WIN32_WINNT=0x502",
            *get_include_flags(arch["name"]),
            "-ferror-limit=0",
            "-Wno-everything",
            str(c_path),
            "-o",
            str(ll_path),
        ]
        return subprocess.run(cmd, capture_output=True, text=True)

    # Iteratively blank lines that error (field/type missing on this arch).
    for _attempt in range(20):
        result = run()
        if result.returncode == 0:
            return ll_path.read_text()
        error_lines: set[int] = set()
        for line in result.stderr.split("\n"):
            m = re.search(rf"{re.escape(c_path.name)}:(\d+):\d+: error:", line)
            if m:
                error_lines.add(int(m.group(1)))
        if not error_lines:
            print(f"  ERROR: unrecoverable clang failure for {arch['name']}")
            for line in result.stderr.split("\n")[:6]:
                print(f"    {line}")
            return None
        src = c_path.read_text().split("\n")
        for ln in sorted(error_lines, reverse=True):
            idx = ln - 1
            if 0 <= idx < len(src) and src[idx].lstrip().startswith("volatile"):
                src[idx] = ""
        c_path.write_text("\n".join(src))
    print(f"  ERROR: failed after max clang retries for {arch['name']}")
    return None


def parse_ir(ir: str) -> dict[str, int]:
    """Return {tag: value} for every `@<tag> = ... i32 VALUE`."""
    vals: dict[str, int] = {}
    for m in re.finditer(r"@(off|asz)_(\d+)\s*=\s*.*?\bi32\s+(\d+)", ir):
        vals[f"{m.group(1)}_{m.group(2)}"] = int(m.group(3))
    return vals


# =============================================================================
# Driver
# =============================================================================


def bindgen_struct_set(structs: dict) -> dict[str, str]:
    """Mirror verify_sizes.extract_types selection: pub struct names -> shortest
    non-pointer typedef alias (used as the C type in offsetof)."""
    content_names = set(structs.keys())
    selected = set()
    for name in content_names:
        if name.startswith("__Bindgen") or name.startswith("__Incomplete"):
            continue
        if "__bindgen_ty_" in name:
            continue
        if name in SKIP_TYPES:
            continue
        selected.add(name)
    return selected


def alias_map(content: str, selected: set[str]) -> dict[str, str]:
    struct_to_alias: dict[str, str] = {}
    for m in re.finditer(r"^pub type (\w+)\s*=\s*(\w+)\s*;", content, re.MULTILINE):
        alias, target = m.group(1), m.group(2)
        if target not in selected:
            continue
        if f"pub type {alias} = *" in content:
            continue
        if target not in struct_to_alias or len(alias) < len(struct_to_alias[target]):
            struct_to_alias[target] = alias
    return struct_to_alias


def build_for_arch(arch: dict, tmpdir: Path) -> tuple[str, int, int] | None:
    """Returns (rust_source, num_structs, num_field_asserts) or None on failure."""
    rs_path = SRC_DIR / arch["rs_file"]
    if not rs_path.exists():
        print(f"  SKIP — {rs_path.name} not found")
        return None
    bindgen_content = rs_path.read_text()
    bindgen_structs = parse_rust_structs(bindgen_content)
    selected = bindgen_struct_set(bindgen_structs)
    aliases = alias_map(bindgen_content, selected)

    # Hand-written module sources, parsed once.
    module_structs: dict[str, dict] = {}
    for mod in {m for _, _, m in HAND_WRITTEN}:
        mpath = SRC_DIR / f"{mod}.rs"
        module_structs[mod] = parse_rust_structs(mpath.read_text()) if mpath.exists() else {}

    # Assemble work items: (c_type, rust_path_struct, rust_module, fields[])
    # fields[] = list of (c_field, rust_field_path, cfgs, is_array)
    work: list[dict] = []

    # 1. bindgen ddk structs
    for sname in sorted(selected):
        fields = collect_fields(sname, bindgen_structs)
        if not fields:
            continue
        c_type = aliases.get(sname, sname)
        work.append(
            {
                "c_type": c_type,
                "rust_type": f"super::ddk::{sname}",
                "fields": fields,
            }
        )

    # 2. hand-written structs
    for rust_name, c_type, mod in HAND_WRITTEN:
        fields = collect_fields(rust_name, module_structs.get(mod, {}))
        if not fields:
            continue
        work.append(
            {
                "c_type": c_type,
                "rust_type": f"super::{mod}::{rust_name}",
                "fields": fields,
            }
        )

    # Build C source. Each (struct,field) gets an off_N tag; arrays also get asz_N.
    c_lines = list(C_PREAMBLE)
    tag = 0
    # entries: (tag, work_index, field_index, kind)
    entries: list[tuple[int, int, int, str]] = []
    for wi, item in enumerate(work):
        ct = item["c_type"]
        for fi, (c_field, _path, _cfgs, is_arr) in enumerate(item["fields"]):
            c_lines.append(
                f"volatile unsigned int off_{tag} = "
                f"__builtin_offsetof({ct}, {c_field}); /* {ct}.{c_field} */"
            )
            entries.append((tag, wi, fi, "off"))
            tag += 1
            if is_arr:
                c_lines.append(
                    f"volatile unsigned int asz_{tag} = "
                    f"(unsigned int)sizeof((({ct}*)0)->{c_field}); /* {ct}.{c_field} arrsz */"
                )
                entries.append((tag, wi, fi, "asz"))
                tag += 1
    c_lines.append("")

    ir = compile_to_ir("\n".join(c_lines), arch, tmpdir)
    if ir is None:
        return None
    vals = parse_ir(ir)

    # Generate Rust asserts.
    out = [
        f"//! Compile-time struct FIELD-OFFSET assertions for {arch['name']}.",
        "//!",
        "//! Auto-generated by `tools/verify_offsets.py` — do not edit manually.",
        "//! Complements size_check_<arch>.rs (sizeof) with per-field offsets,",
        "//! catching right-size/wrong-offset layout drift.",
        "//!",
        "//! Regenerate: `python3 tools/verify_offsets.py`",
        "",
        "#![allow(dead_code)]",
        "",
        "use core::mem::offset_of;",
        "",
    ]
    struct_count = 0
    field_count = 0
    seen_structs: set[int] = set()
    for tg, wi, fi, kind in entries:
        if kind != "off":
            continue
        key = f"off_{tg}"
        if key not in vals:
            continue  # field was removed (didn't exist on this arch)
        item = work[wi]
        c_field, rust_path, cfgs, _is_arr = item["fields"][fi]
        offset = vals[key]
        for c in cfgs:
            out.append(f"#[cfg({c})]")
        out.append(
            f"const _: () = assert!(offset_of!({item['rust_type']}, {rust_path}) == {offset}); "
            f"// {item['c_type']}.{c_field}"
        )
        field_count += 1
        if wi not in seen_structs:
            seen_structs.add(wi)
            struct_count += 1
    out.append("")
    return "\n".join(out), struct_count, field_count


def main():
    check_mode = "--check" in sys.argv
    print("=== ntkrnl-sys struct field-OFFSET verification ===")
    print(f"ReactOS root: {REACTOS}")
    print(f"Build repo:   {BUILD_REPO}")
    print(f"Toolchain:    {TOOLCHAIN}")
    print()

    all_ok = True
    with tempfile.TemporaryDirectory() as td:
        td = Path(td)
        for arch in ARCHS:
            print(f"--- {arch['name']} ---")
            res = build_for_arch(arch, td)
            if res is None:
                all_ok = False
                continue
            rust_src, nstructs, nfields = res
            print(f"  {nstructs} structs, {nfields} field-offset asserts")
            out_path = SRC_DIR / arch["out_file"]
            if check_mode:
                if out_path.exists() and out_path.read_text() == rust_src:
                    print(f"  OK — {out_path.name} up to date")
                else:
                    print(f"  MISMATCH — run verify_offsets.py to regenerate {out_path.name}")
                    all_ok = False
            else:
                out_path.write_text(rust_src)
                print(f"  Wrote {out_path.name}")

    print()
    if check_mode and not all_ok:
        print("FAILED — offset assertions need regeneration")
        sys.exit(1)
    if not check_mode:
        print(
            "Done. Ensure offset_check_<arch> modules are declared in lib.rs, "
            "then cargo check -p ntkrnl-sys."
        )


if __name__ == "__main__":
    main()
