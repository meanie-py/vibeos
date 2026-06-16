"""Automated C vs Rust object file comparison tool.

Compiles original C files (commented out of CMakeLists), extracts symbols from
both C and Rust object files, and flags differences — especially calling
convention mismatches on i386 where stdcall/fastcall/cdecl coexist.

Pipeline (7 stages):
  1. Flag extraction from build.ninja
  2. C compilation
  3. Rust symbol extraction (llvm-nm on libkernel_rust.a)
  4. C symbol extraction (llvm-nm on compiled .obj)
  5. i386 calling convention analysis (llvm-objdump -d → ret N)
  6. Comparison engine
  7. Report generation
"""

import json
import re
import subprocess
import sys
import tempfile
from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import dataclass
from pathlib import Path

from .config import (
    BUILD_DIRS,
    CLANG_TARGETS,
    LLVM_MINGW,
    NTOS_CMAKE,
    NTOSKRNL_DIR,
    OUTPUT_CRATE,
    REACTOS_ROOT,
    RUST_TARGETS,
)
from .spec_parser import parse_spec

# ── Data structures ──────────────────────────────────────────────────────────


@dataclass
class Symbol:
    name: str  # Undecorated (e.g., "NtAddAtom")
    decorated_name: str  # As-is from nm (e.g., "_NtAddAtom@12")
    symbol_type: str  # "function" | "data"
    size: int  # Bytes (0 if unknown)
    section: str  # ".text", "INIT", "PAGE", etc.
    calling_convention: str  # "stdcall" | "fastcall" | "cdecl" | "unknown"
    param_bytes: int  # From @N decoration (i386) or ret N instruction (-1 = unknown)
    ret_instruction: str  # "ret" | "ret N" (from disassembly)
    # Deep prologue info (filled by analyze_prologues)
    stack_frame_size: int = -1  # sub esp/rsp, N → N bytes (-1 = unknown)
    callee_saved_regs: int = -1  # Number of push instructions in prologue (-1 = unknown)
    has_frame_pointer: bool = False  # push ebp; mov ebp, esp


@dataclass
class Difference:
    kind: str  # "calling_conv_mismatch", "missing_in_rust", etc.
    severity: str  # "critical" | "warning" | "info"
    symbol_name: str
    c_value: str
    rust_value: str
    detail: str
    file: str  # "ex/atom"


@dataclass
class FilePair:
    """A C file and its Rust counterpart."""

    subsystem: str  # "ex", "ke/i386", "mm/ARM3"
    stem: str  # "atom", "pushlock"
    c_path: Path  # ntoskrnl/ex/atom.c
    rs_path: Path  # ntoskrnl-rust/src/ex/atom.rs
    label: str  # "ex/atom" for display


@dataclass
class CompileFlags:
    """Extracted compilation flags from build.ninja."""

    defines: list[str]
    flags: list[str]
    includes: list[str]
    clang: str  # Full path to clang binary


@dataclass
class ObjdiffReport:
    """Full report from objdiff run."""

    arch: str
    file_pairs: list[FilePair]
    differences: list[Difference]
    compile_failures: list[tuple[str, str]]  # (label, error)
    c_symbols_count: int
    rust_symbols_count: int
    baseline: str | None = None  # git commit hash if using baseline


# ── Stage 1: Flag extraction from build.ninja ────────────────────────────────

_flag_cache: dict[str, CompileFlags] = {}


def extract_compile_flags(arch: str) -> CompileFlags:
    """Parse build.ninja to extract DEFINES, FLAGS, INCLUDES for ntkrnlmp C files."""
    if arch in _flag_cache:
        return _flag_cache[arch]

    build_dir = BUILD_DIRS[arch]
    ninja_file = build_dir / "build.ninja"

    if not ninja_file.exists():
        raise FileNotFoundError(f"build.ninja not found at {ninja_file}")

    triplet = CLANG_TARGETS[arch]
    clang = str(LLVM_MINGW / "bin" / f"{triplet}-clang")

    # Find a C_COMPILER__ntkrnlmp build statement and extract its variables.
    # We look for any actively-compiled ntoskrnl C file (e.g. kd64/kdapi.c).
    defines = []
    flags_list = []
    includes = []

    # Read build.ninja and find a ntkrnlmp build block for an actual .c.obj
    # (Skip .pch build statements — they have different flags)
    found = False
    with open(ninja_file) as f:
        for line in f:
            if "C_COMPILER__ntkrnlmp" in line and line.startswith("build "):
                # Only match actual .c.obj builds, not .pch generation
                if ".c.obj" not in line:
                    # Skip variable lines for this build statement
                    for var_line in f:
                        if not var_line.startswith("  "):
                            break
                    continue
                # Found a real C compilation; read its variable assignments
                for var_line in f:
                    stripped = var_line.strip()
                    if not stripped or not var_line.startswith("  "):
                        break
                    if stripped.startswith("DEFINES = "):
                        defines = _split_flags(stripped[len("DEFINES = ") :])
                    elif stripped.startswith("FLAGS = "):
                        flags_list = _split_flags(stripped[len("FLAGS = ") :])
                    elif stripped.startswith("INCLUDES = "):
                        includes = _split_flags(stripped[len("INCLUDES = ") :])
                found = True
                break

    if not found:
        raise RuntimeError(f"No C_COMPILER__ntkrnlmp build statements found in {ninja_file}")

    # Post-process flags:
    # 1. Strip PCH flags (we'll include the master header directly)
    # 2. Strip debug flags (we only need symbols for nm)
    flags_list = _strip_pch_flags(flags_list)
    flags_list = [f for f in flags_list if f not in ("-gdwarf-2", "-gstrict-dwarf")]

    # Add the master ntoskrnl header include
    master_header = str(NTOSKRNL_DIR / "include" / "ntoskrnl.h")
    flags_list.extend(["-include", master_header])

    result = CompileFlags(
        defines=defines,
        flags=flags_list,
        includes=includes,
        clang=clang,
    )
    _flag_cache[arch] = result
    return result


def _split_flags(s: str) -> list[str]:
    """Split a space-separated flag string, respecting -Xclang pairs."""
    tokens = s.split()
    return tokens


def _strip_pch_flags(flags: list[str]) -> list[str]:
    """Remove PCH-related flags from the compiler flags.

    Ninja format uses: -Xclang -include-pch -Xclang <path.pch> -Xclang -include -Xclang <path.h>
    That's 4-token groups: (-Xclang <clang-flag> -Xclang <arg>)
    """
    result = []
    i = 0
    while i < len(flags):
        f = flags[i]

        # -Xclang -include-pch -Xclang <path.pch> (4 tokens)
        if (
            f == "-Xclang"
            and i + 3 < len(flags)
            and flags[i + 1] == "-include-pch"
            and flags[i + 2] == "-Xclang"
        ):
            i += 4  # skip all 4
            continue

        # -Xclang -include -Xclang <cmake_pch.h> (4 tokens)
        if (
            f == "-Xclang"
            and i + 3 < len(flags)
            and flags[i + 1] == "-include"
            and flags[i + 2] == "-Xclang"
            and "cmake_pch" in flags[i + 3]
        ):
            i += 4
            continue

        # -Winvalid-pch
        if f == "-Winvalid-pch":
            i += 1
            continue

        result.append(f)
        i += 1
    return result


# ── Baseline extraction ──────────────────────────────────────────────────────


def extract_baseline_c_files(
    file_pairs: list[FilePair], commit: str, output_dir: Path
) -> dict[str, Path]:
    """Extract C files at a given git commit to a temp directory.

    Returns {label: extracted_c_path}.  Files that don't exist at that commit
    are silently skipped.
    """
    extracted: dict[str, Path] = {}

    for fp in file_pairs:
        # Build the git-relative path (e.g., ntoskrnl/ex/atom.c)
        try:
            rel = fp.c_path.relative_to(REACTOS_ROOT)
        except ValueError:
            continue

        git_path = str(rel)
        dest_dir = output_dir / fp.c_path.parent.name
        dest_dir.mkdir(parents=True, exist_ok=True)
        dest_file = dest_dir / fp.c_path.name

        try:
            result = subprocess.run(
                ["git", "-C", str(REACTOS_ROOT), "show", f"{commit}:{git_path}"],
                capture_output=True,
                timeout=10,
            )
            if result.returncode == 0:
                dest_file.write_bytes(result.stdout)
                extracted[fp.label] = dest_file
        except Exception:
            continue

    return extracted


# ── Stage 2: C compilation ───────────────────────────────────────────────────


def compile_c_file(
    c_path: Path,
    arch: str,
    output_dir: Path,
    compile_flags: CompileFlags,
    label_override: str | None = None,
) -> tuple[str, Path | None, str]:
    """Compile a single C file. Returns (label, obj_path_or_None, error_msg)."""
    label = label_override or _c_path_to_label(c_path)
    obj_name = f"{c_path.parent.name}_{c_path.stem}.obj"
    obj_path = output_dir / obj_name

    triplet = CLANG_TARGETS[arch]

    # Adjust flags for C++ files
    flags = compile_flags.flags
    if c_path.suffix in (".cpp", ".cxx", ".cc"):
        flags = [f for f in flags if f != "-std=gnu99"]

    cmd = [
        compile_flags.clang,
        f"--target={triplet}",
        *compile_flags.defines,
        *compile_flags.includes,
        *flags,
        "-c",
        str(c_path),
        "-o",
        str(obj_path),
    ]

    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=60,
        )
        if result.returncode != 0:
            return label, None, result.stderr[:500]
        return label, obj_path, ""
    except subprocess.TimeoutExpired:
        return label, None, "compilation timed out"
    except Exception as e:
        return label, obj_path, str(e)


def compile_c_files(
    file_pairs: list[FilePair],
    arch: str,
    output_dir: Path,
    compile_flags: CompileFlags,
    max_workers: int = 8,
    path_overrides: dict[str, Path] | None = None,
) -> tuple[dict[str, Path], list[tuple[str, str]]]:
    """Compile all C files in parallel. Returns ({label: obj_path}, failures).

    If path_overrides is set, use those paths instead of the file pair's c_path.
    Only files present in path_overrides are compiled when overrides are provided.
    """
    results: dict[str, Path] = {}
    failures: list[tuple[str, str]] = []

    with ThreadPoolExecutor(max_workers=max_workers) as executor:
        futures = {}
        for fp in file_pairs:
            if path_overrides is not None:
                if fp.label not in path_overrides:
                    continue
                c_path = path_overrides[fp.label]
            else:
                c_path = fp.c_path
            futures[
                executor.submit(
                    compile_c_file,
                    c_path,
                    arch,
                    output_dir,
                    compile_flags,
                    label_override=fp.label,
                )
            ] = fp
        for future in as_completed(futures):
            label, obj_path, error = future.result()
            if obj_path and not error:
                results[label] = obj_path
            elif error:
                failures.append((label, error))

    return results, failures


# ── Stage 3: Rust symbol extraction ──────────────────────────────────────────


def _find_rust_lib(arch: str) -> Path | None:
    """Find the libkernel_rust.a path for an architecture."""
    build_dir = BUILD_DIRS[arch]
    lib_path = build_dir / "ntoskrnl" / "libkernel_rust.a"
    if lib_path.exists():
        return lib_path
    rust_target = RUST_TARGETS[arch]
    lib_path = (
        build_dir / "ntoskrnl" / "rust-kernel-target" / rust_target / "release" / "libkernel_rust.a"
    )
    if lib_path.exists():
        return lib_path
    return None


def extract_rust_symbols(arch: str) -> dict[str, Symbol]:
    """Extract symbols from libkernel_rust.a using llvm-nm."""
    lib_path = _find_rust_lib(arch)

    if not lib_path:
        raise FileNotFoundError(f"libkernel_rust.a not found for {arch}")

    nm = str(LLVM_MINGW / "bin" / "llvm-nm")
    cmd = [nm, "--defined-only", "--print-size", str(lib_path)]

    result = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
    if result.returncode != 0:
        raise RuntimeError(f"llvm-nm failed: {result.stderr[:500]}")

    return _parse_nm_output(result.stdout, arch)


def extract_c_symbols(obj_path: Path, arch: str) -> dict[str, Symbol]:
    """Extract symbols from a single C .obj file using llvm-nm."""
    nm = str(LLVM_MINGW / "bin" / "llvm-nm")
    cmd = [nm, "--defined-only", "--print-size", str(obj_path)]

    result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
    if result.returncode != 0:
        return {}

    return _parse_nm_output(result.stdout, arch)


def _parse_nm_output(output: str, arch: str) -> dict[str, Symbol]:
    """Parse llvm-nm --defined-only --print-size output.

    Format: <addr> <size> <type> <name>
    or:     <addr> <type> <name>  (no size)
    """
    symbols: dict[str, Symbol] = {}

    for line in output.splitlines():
        line = line.strip()
        if not line:
            continue

        parts = line.split()
        if len(parts) < 3:
            continue

        # Determine format: with or without size field
        # Type is a single character (T, D, B, R, etc.)
        if len(parts) >= 4 and len(parts[2]) == 1 and parts[2].isalpha():
            # <addr> <size> <type> <name>
            size = int(parts[1], 16)
            sym_type_char = parts[2]
            decorated_name = parts[3]
        elif len(parts) >= 3 and len(parts[1]) == 1 and parts[1].isalpha():
            # <addr> <type> <name>
            size = 0
            sym_type_char = parts[1]
            decorated_name = parts[2]
        else:
            continue

        # Filter to global symbols only (uppercase type chars)
        if not sym_type_char.isupper():
            continue

        # Classify: T/t = text (function), D/d/B/b = data, R/r = rodata
        if sym_type_char in ("T",):
            symbol_type = "function"
        elif sym_type_char in ("D", "B", "R"):
            symbol_type = "data"
        else:
            continue  # skip other types (A, C, etc.)

        # Undecorate for i386
        if arch == "i386":
            undecorated, calling_conv, param_bytes = undecorate_i386(decorated_name)
        else:
            undecorated = decorated_name
            calling_conv = "unknown"
            param_bytes = -1

        # Skip compiler-internal symbols
        if undecorated.startswith("__") or undecorated.startswith("_$"):
            continue
        if undecorated in ("DllMainCRTStartup", "_DllMainCRTStartup"):
            continue
        # Skip .refptr.* stubs (compiler-generated reference pointers on i386/amd64)
        if decorated_name.startswith(".refptr.") or undecorated.startswith(".refptr."):
            continue
        # Skip SEH-related symbols
        if "_SEH2_" in undecorated or "_SEH2_" in decorated_name:
            continue

        sym = Symbol(
            name=undecorated,
            decorated_name=decorated_name,
            symbol_type=symbol_type,
            size=size,
            section="",  # filled by objdump if needed
            calling_convention=calling_conv,
            param_bytes=param_bytes,
            ret_instruction="",
        )
        symbols[undecorated] = sym

    return symbols


# ── Stage 4: i386 calling convention analysis ────────────────────────────────


def analyze_calling_conventions(obj_path: Path, symbols: dict[str, Symbol], arch: str) -> None:
    """Run llvm-objdump -d to find ret/ret N instructions for each function.
    Mutates the symbols dict in place.
    Only meaningful on i386.
    """
    if arch != "i386":
        return

    objdump = str(LLVM_MINGW / "bin" / "llvm-objdump")
    cmd = [objdump, "-d", "--no-show-raw-insn", str(obj_path)]

    result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
    if result.returncode != 0:
        return

    # Parse disassembly to find ret instructions per function
    current_func = None
    last_ret = None

    for line in result.stdout.splitlines():
        # Function label: "00000000 <_NtAddAtom@12>:"
        m = re.match(r"^[0-9a-f]+\s+<(.+?)>:", line)
        if m:
            # Save the last ret for the previous function
            if current_func and last_ret is not None:
                _apply_ret_info(symbols, current_func, last_ret, arch)
            current_func = m.group(1)
            last_ret = None
            continue

        # Instruction lines
        stripped = line.strip()
        if not stripped:
            continue

        # Match ret instructions
        # "retl" (cdecl) or "retl $0xN" (stdcall/fastcall)
        if "\tret" in stripped:
            # Extract the full instruction
            # Format: "addr: retl" or "addr: retl $0x10"
            parts = stripped.split("\t")
            if len(parts) >= 2:
                insn = parts[-1].strip()
                if insn.startswith("ret"):
                    last_ret = insn

    # Don't forget the last function
    if current_func and last_ret is not None:
        _apply_ret_info(symbols, current_func, last_ret, arch)


def _apply_ret_info(symbols: dict[str, Symbol], func_label: str, ret_insn: str, arch: str) -> None:
    """Apply ret instruction info to the matching symbol."""
    # Undecorate the label from objdump
    if arch == "i386":
        undecorated, _, _ = undecorate_i386(func_label)
    else:
        undecorated = func_label

    if undecorated not in symbols:
        return

    sym = symbols[undecorated]
    sym.ret_instruction = ret_insn

    # Parse ret N to get param bytes from actual instruction
    m = re.match(r"retl?\s+\$(?:0x)?([0-9a-fA-F]+)", ret_insn)
    if m:
        ret_bytes = int(m.group(1), 16)
        # ret $N means callee pops N bytes → stdcall or fastcall
        if sym.calling_convention == "unknown":
            sym.calling_convention = "stdcall"  # could be fastcall too
        sym.param_bytes = ret_bytes
    else:
        # Plain ret → cdecl (caller cleans stack)
        if ret_insn.startswith("ret"):
            if sym.calling_convention == "unknown":
                sym.calling_convention = "cdecl"
            if sym.param_bytes == -1:
                sym.param_bytes = 0  # cdecl: callee pops 0


# ── Stage 4b: Deep prologue analysis ─────────────────────────────────────────


def analyze_prologues(obj_path: Path, symbols: dict[str, Symbol], arch: str) -> None:
    """Parse function prologues to extract stack frame sizes and callee-saved registers.

    Mutates the symbols dict in place. Works on all architectures.
    """
    objdump = str(LLVM_MINGW / "bin" / "llvm-objdump")
    cmd = [objdump, "-d", "--no-show-raw-insn", str(obj_path)]

    result = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
    if result.returncode != 0:
        return

    # Architecture-specific patterns
    if arch == "i386":
        # sub esp, N
        re_sub_sp = re.compile(r"subl?\s+\$(?:0x)?([0-9a-fA-F]+),\s+%esp")
        # push <reg>
        re_push = re.compile(r"pushl?\s+%(\w+)")
        # mov ebp, esp (frame pointer setup)
        re_frame_ptr = re.compile(r"movl?\s+%esp,\s+%ebp")
        callee_saved = {"ebx", "esi", "edi", "ebp"}
    elif arch == "amd64":
        # sub rsp, N
        re_sub_sp = re.compile(r"subq?\s+\$(?:0x)?([0-9a-fA-F]+),\s+%rsp")
        re_push = re.compile(r"pushq?\s+%(\w+)")
        re_frame_ptr = re.compile(r"movq?\s+%rsp,\s+%rbp")
        callee_saved = {"rbx", "rbp", "r12", "r13", "r14", "r15", "rsi", "rdi"}
    elif arch == "arm64":
        # sub sp, sp, #N or stp with pre-index [sp, #-N]!
        re_sub_sp = re.compile(r"sub\s+sp,\s+sp,\s+#(\d+)")
        re_push = re.compile(r"stp\s+(\w+),\s+(\w+),\s+\[sp")  # stp pair = 2 regs
        re_frame_ptr = re.compile(r"mov\s+x29,\s+sp")
        callee_saved = set()  # ARM64 uses stp pairs
    else:
        return

    current_func = None
    in_prologue = True  # True for first N instructions of function
    prologue_insn_count = 0
    push_count = 0
    frame_size = -1
    has_fp = False

    def _save_prologue():
        nonlocal current_func, push_count, frame_size, has_fp
        if not current_func:
            return
        if arch == "i386":
            undecorated, _, _ = undecorate_i386(current_func)
        else:
            undecorated = current_func
        if undecorated in symbols:
            sym = symbols[undecorated]
            sym.stack_frame_size = frame_size
            sym.callee_saved_regs = push_count
            sym.has_frame_pointer = has_fp

    for line in result.stdout.splitlines():
        # Function label
        m = re.match(r"^[0-9a-f]+\s+<(.+?)>:", line)
        if m:
            _save_prologue()
            current_func = m.group(1)
            in_prologue = True
            prologue_insn_count = 0
            push_count = 0
            frame_size = -1
            has_fp = False
            continue

        if not in_prologue or not current_func:
            continue

        stripped = line.strip()
        if not stripped:
            continue

        prologue_insn_count += 1
        # Only look at first 15 instructions for prologue
        if prologue_insn_count > 15:
            in_prologue = False
            continue

        # Extract instruction (after addr: prefix)
        # Format: "addr:\tmnemonic\toperands" or "addr: \tmnemonic\toperands"
        parts = stripped.split("\t")
        if len(parts) < 2:
            continue
        # Join all parts after the address into a single instruction string
        insn = " ".join(p.strip() for p in parts[1:] if p.strip())

        # Check for sub sp, N
        m = re_sub_sp.search(insn)
        if m:
            frame_size = (
                int(m.group(1), 16) if "0x" in insn or not insn[-1].isdigit() else int(m.group(1))
            )
            # sub esp/rsp often ends the prologue
            if arch != "arm64":
                in_prologue = False
            continue

        # Check for push
        m = re_push.search(insn)
        if m:
            if arch == "arm64":
                push_count += 2  # stp saves 2 registers
            else:
                reg = m.group(1)
                if reg in callee_saved:
                    push_count += 1
            continue

        # Check for frame pointer setup
        if re_frame_ptr.search(insn):
            has_fp = True
            continue

        # Non-prologue instruction: call, mov to non-stack, etc.
        # Stop looking at prologue
        if arch == "i386" or arch == "amd64":
            # Allow some non-push/sub instructions in prologue (like mov, lea for args)
            if insn.startswith(("mov", "lea", "xor", "and", "nop")):
                continue
            in_prologue = False

    _save_prologue()


# ── Stage 5: Comparison engine ───────────────────────────────────────────────


def compare_symbols(
    c_syms: dict[str, Symbol],
    rust_syms: dict[str, Symbol],
    label: str,
    arch: str,
    spec: dict | None = None,
    deep: bool = False,
) -> list[Difference]:
    """Compare C and Rust symbols for a single file pair."""
    diffs: list[Difference] = []

    # Only compare functions that are non-static (exported from the .obj)
    for name, c_sym in c_syms.items():
        if c_sym.symbol_type == "function":
            if name not in rust_syms:
                diffs.append(
                    Difference(
                        kind="missing_in_rust",
                        severity="warning",
                        symbol_name=name,
                        c_value=c_sym.decorated_name,
                        rust_value="",
                        detail="Function present in C .obj but not in Rust .a",
                        file=label,
                    )
                )
                continue

            r_sym = rust_syms[name]
            if r_sym.symbol_type != "function":
                diffs.append(
                    Difference(
                        kind="type_mismatch",
                        severity="critical",
                        symbol_name=name,
                        c_value=f"function ({c_sym.decorated_name})",
                        rust_value=f"data ({r_sym.decorated_name})",
                        detail="C has function, Rust has data symbol",
                        file=label,
                    )
                )
                continue

            # i386: calling convention check
            if arch == "i386":
                diffs.extend(_check_i386_convention(c_sym, r_sym, label, spec))

            # Size outlier check (only if both have sizes)
            if c_sym.size > 0 and r_sym.size > 0:
                ratio = r_sym.size / c_sym.size if c_sym.size > 0 else 0
                if ratio < 0.25 or ratio > 5.0:
                    diffs.append(
                        Difference(
                            kind="size_outlier",
                            severity="info",
                            symbol_name=name,
                            c_value=f"{c_sym.size} bytes",
                            rust_value=f"{r_sym.size} bytes",
                            detail=f"Size ratio: {ratio:.1f}x",
                            file=label,
                        )
                    )

            # Deep prologue comparison
            if deep:
                diffs.extend(_check_prologue(c_sym, r_sym, label))

        elif c_sym.symbol_type == "data":
            if name not in rust_syms:
                diffs.append(
                    Difference(
                        kind="missing_global_in_rust",
                        severity="warning",
                        symbol_name=name,
                        c_value=c_sym.decorated_name,
                        rust_value="",
                        detail="Global variable present in C but not in Rust",
                        file=label,
                    )
                )
                continue

            r_sym = rust_syms[name]
            if r_sym.symbol_type != "data":
                continue  # Functions re-exported as data? Skip.

            # Global size check
            if c_sym.size > 0 and r_sym.size > 0 and c_sym.size != r_sym.size:
                diffs.append(
                    Difference(
                        kind="global_size_mismatch",
                        severity="warning",
                        symbol_name=name,
                        c_value=f"{c_sym.size} bytes",
                        rust_value=f"{r_sym.size} bytes",
                        detail="Global variable size differs",
                        file=label,
                    )
                )

    # Check for symbols in Rust but not C (info only)
    for name, r_sym in rust_syms.items():
        if name not in c_syms and r_sym.symbol_type == "function":
            # Only report if it looks like it belongs to this file
            # (We can't do perfect attribution from the .a file)
            pass  # Skip: too noisy without file attribution

    return diffs


def _check_i386_convention(
    c_sym: Symbol, r_sym: Symbol, label: str, spec: dict | None = None
) -> list[Difference]:
    """Check i386 calling convention and param byte count between C and Rust."""
    diffs: list[Difference] = []

    # Decoration check: different decoration = different ABI
    c_dec = c_sym.decorated_name
    r_dec = r_sym.decorated_name

    # Compare conventions inferred from decoration
    if (
        c_sym.calling_convention != "unknown"
        and r_sym.calling_convention != "unknown"
        and c_sym.calling_convention != r_sym.calling_convention
    ):
        diffs.append(
            Difference(
                kind="calling_conv_mismatch",
                severity="critical",
                symbol_name=c_sym.name,
                c_value=f"{c_sym.calling_convention} ({c_dec})",
                rust_value=f"{r_sym.calling_convention} ({r_dec})",
                detail="Calling convention mismatch — will cause stack corruption",
                file=label,
            )
        )
        return diffs  # Don't double-report param bytes

    # Decoration mismatch (even if convention is the same)
    # On i386, _Name@N decorations must match exactly
    # But some names may differ just by leading underscore (cdecl)
    if c_dec != r_dec and _normalize_decoration(c_dec) != _normalize_decoration(r_dec):
        diffs.append(
            Difference(
                kind="decoration_mismatch",
                severity="critical",
                symbol_name=c_sym.name,
                c_value=c_dec,
                rust_value=r_dec,
                detail="Symbol decoration differs — linker will not match",
                file=label,
            )
        )

    # Param bytes check (from decoration @N and/or ret N)
    c_bytes = c_sym.param_bytes
    r_bytes = r_sym.param_bytes

    # Prefer ret instruction param bytes over decoration for accuracy
    if c_sym.ret_instruction and r_sym.ret_instruction:
        c_ret_bytes = _parse_ret_bytes(c_sym.ret_instruction)
        r_ret_bytes = _parse_ret_bytes(r_sym.ret_instruction)
        if c_ret_bytes is not None and r_ret_bytes is not None and c_ret_bytes != r_ret_bytes:
            diffs.append(
                Difference(
                    kind="param_bytes_mismatch",
                    severity="critical",
                    symbol_name=c_sym.name,
                    c_value=f"ret ${c_ret_bytes:#x} ({c_ret_bytes} bytes)",
                    rust_value=f"ret ${r_ret_bytes:#x} ({r_ret_bytes} bytes)",
                    detail="Stack cleanup size mismatch — will corrupt stack",
                    file=label,
                )
            )
    elif c_bytes >= 0 and r_bytes >= 0 and c_bytes != r_bytes:
        diffs.append(
            Difference(
                kind="param_bytes_mismatch",
                severity="critical",
                symbol_name=c_sym.name,
                c_value=f"@{c_bytes}",
                rust_value=f"@{r_bytes}",
                detail="Parameter byte count from decoration differs",
                file=label,
            )
        )

    # Cross-reference with spec if available
    if spec and c_sym.name in spec:
        entry = spec[c_sym.name]
        expected_conv = entry.convention
        # Check Rust against spec
        if expected_conv in ("stdcall", "fastcall") and r_sym.calling_convention == "cdecl":
            diffs.append(
                Difference(
                    kind="spec_conv_mismatch",
                    severity="critical",
                    symbol_name=c_sym.name,
                    c_value=f"spec: {expected_conv}",
                    rust_value=f"actual: {r_sym.calling_convention}",
                    detail=(
                        f"ntoskrnl.spec says {expected_conv}, Rust uses {r_sym.calling_convention}"
                    ),
                    file=label,
                )
            )

    return diffs


def _parse_ret_bytes(ret_insn: str) -> int | None:
    """Parse 'ret $0xN' or 'retl $0xN' to get N. Returns None for plain ret."""
    m = re.match(r"retl?\s+\$(?:0x)?([0-9a-fA-F]+)", ret_insn)
    if m:
        return int(m.group(1), 16)
    if ret_insn.strip().rstrip("l") == "ret":
        return 0
    return None


def _normalize_decoration(name: str) -> str:
    """Normalize decoration for comparison (strip leading _ for cdecl)."""
    return name


def _check_prologue(c_sym: Symbol, r_sym: Symbol, label: str) -> list[Difference]:
    """Compare prologue characteristics between C and Rust functions.

    Reports significant differences in stack frame size and callee-saved
    register count. These are warnings, not criticals — optimizer choices
    legitimately differ.
    """
    diffs: list[Difference] = []

    # Stack frame size comparison
    if c_sym.stack_frame_size >= 0 and r_sym.stack_frame_size >= 0:
        c_frame = c_sym.stack_frame_size
        r_frame = r_sym.stack_frame_size
        if c_frame != r_frame:
            diff_bytes = abs(r_frame - c_frame)
            # Only report if the difference is significant (>= 64 bytes or 4x ratio)
            if diff_bytes >= 64 or (c_frame > 0 and r_frame / c_frame > 4.0):
                diffs.append(
                    Difference(
                        kind="frame_size_diff",
                        severity="warning",
                        symbol_name=c_sym.name,
                        c_value=f"{c_frame} bytes",
                        rust_value=f"{r_frame} bytes",
                        detail=f"Stack frame size differs by {diff_bytes} bytes",
                        file=label,
                    )
                )

    # Callee-saved register count comparison
    if c_sym.callee_saved_regs >= 0 and r_sym.callee_saved_regs >= 0:
        c_regs = c_sym.callee_saved_regs
        r_regs = r_sym.callee_saved_regs
        if abs(c_regs - r_regs) >= 3:
            diffs.append(
                Difference(
                    kind="callee_saved_diff",
                    severity="info",
                    symbol_name=c_sym.name,
                    c_value=f"{c_regs} regs",
                    rust_value=f"{r_regs} regs",
                    detail=f"Callee-saved register count differs (C: {c_regs}, Rust: {r_regs})",
                    file=label,
                )
            )

    # Frame pointer usage
    if c_sym.has_frame_pointer != r_sym.has_frame_pointer:
        diffs.append(
            Difference(
                kind="frame_pointer_diff",
                severity="info",
                symbol_name=c_sym.name,
                c_value="yes" if c_sym.has_frame_pointer else "no",
                rust_value="yes" if r_sym.has_frame_pointer else "no",
                detail="Frame pointer usage differs",
                file=label,
            )
        )

    return diffs


# ── i386 symbol undecoration ─────────────────────────────────────────────────


def undecorate_i386(name: str) -> tuple[str, str, int]:
    """Returns (undecorated, convention, param_bytes).

    i386 PE-COFF decoration:
      @Name@N  → fastcall, N param bytes
      _Name@N  → stdcall, N param bytes
      _Name    → cdecl
    """
    if name.startswith("@") and "@" in name[1:]:
        # @Name@N → fastcall
        parts = name[1:].rsplit("@", 1)
        if len(parts) == 2 and parts[1].isdigit():
            return parts[0], "fastcall", int(parts[1])
    if name.startswith("_") and "@" in name[1:]:
        # _Name@N → stdcall
        parts = name[1:].rsplit("@", 1)
        if len(parts) == 2 and parts[1].isdigit():
            return parts[0], "stdcall", int(parts[1])
    if name.startswith("_"):
        # _Name → cdecl
        return name[1:], "cdecl", -1
    return name, "unknown", -1


# ── File pair discovery ──────────────────────────────────────────────────────

# Regex patterns for commented-out C files in ntos.cmake
# Pattern 1: #${REACTOS_SOURCE_DIR}/ntoskrnl/ex/atom.c
RE_COMMENTED_1 = re.compile(r"#\s*\$\{REACTOS_SOURCE_DIR\}/ntoskrnl/(.+?)\.c")
# Pattern 2: # ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/ARM3/contmem.c (Rust)
RE_COMMENTED_2 = re.compile(r"#\s*\$\{REACTOS_SOURCE_DIR\}/ntoskrnl/(.+?)\.c\b")

# Arch-specific directory mapping
ARCH_DIRS = {
    "i386": {"ke/i386", "mm/i386", "mm/ARM3/i386", "vdm"},
    "amd64": {"ke/amd64", "mm/amd64", "mm/ARM3/amd64"},
    "arm64": {"ke/arm64", "mm/arm64", "mm/ARM3/arm64"},
}

# Map C directory paths to Rust module paths
C_TO_RUST_DIR = {
    "mm/ARM3": "ARM3",
    "mm/ARM3/i386": "ARM3",  # i386/init.c → ARM3/init.rs
    "mm/ARM3/amd64": "ARM3",  # amd64/init.c → ARM3/init.rs
    "mm/ARM3/arm64": "ARM3",  # arm64/init.c → ARM3/init.rs
    "mm/ARM3/arm": "ARM3",
}


def discover_file_pairs(arch: str, target: str | None = None) -> list[FilePair]:
    """Scan ntos.cmake for commented-out C files and find their Rust counterparts.

    Args:
        arch: Target architecture (i386, amd64, arm64)
        target: Optional filter — subsystem name (e.g., "ex") or file (e.g., "ex/atom")
    """
    cmake_text = NTOS_CMAKE.read_text()
    pairs: list[FilePair] = []
    seen = set()

    for line in cmake_text.splitlines():
        m = RE_COMMENTED_1.match(line.strip())
        if not m:
            m = RE_COMMENTED_2.match(line.strip())
        if not m:
            continue

        rel_path = m.group(1)  # e.g., "ex/atom", "mm/ARM3/contmem", "ke/i386/cpu"
        parts = rel_path.rsplit("/", 1)
        if len(parts) != 2:
            continue

        dir_part = parts[0]  # "ex", "mm/ARM3", "ke/i386"
        stem = parts[1]  # "atom", "contmem", "cpu"

        # Skip duplicates
        if rel_path in seen:
            continue
        seen.add(rel_path)

        # Filter by architecture: arch-specific dirs only for matching arch
        is_arch_specific = False
        for a, dirs in ARCH_DIRS.items():
            if dir_part in dirs:
                is_arch_specific = True
                if a != arch:
                    break  # Skip: wrong architecture
        else:
            pass  # Not arch-specific, include for all

        if is_arch_specific:
            skip = True
            if dir_part in ARCH_DIRS.get(arch, set()):
                skip = False
            if skip:
                continue

        # Build C path
        c_path = NTOSKRNL_DIR / f"{rel_path}.c"
        if not c_path.exists():
            # Some files might be .cpp
            c_path = NTOSKRNL_DIR / f"{rel_path}.cpp"
            if not c_path.exists():
                continue

        # Build Rust path
        # C: ntoskrnl/{subsys}/{file}.c → Rust: ntoskrnl-rust/src/{subsys}/{file}.rs
        rust_dir = C_TO_RUST_DIR.get(dir_part, dir_part)
        rs_path = OUTPUT_CRATE / "src" / rust_dir / f"{stem}.rs"
        if not rs_path.exists():
            # Try without arch subdirectory mapping
            rs_path = OUTPUT_CRATE / "src" / dir_part / f"{stem}.rs"
            if not rs_path.exists():
                continue

        # Compute subsystem and label for display
        subsystem = dir_part
        label = f"{dir_part}/{stem}"

        # Apply target filter
        if target:
            target_normalized = target.rstrip("/")
            if "/" in target_normalized:
                # Exact file match
                if label != target_normalized:
                    continue
            else:
                # Subsystem match (also match nested like ke/i386 under ke)
                if not dir_part.startswith(target_normalized):
                    continue

        pairs.append(
            FilePair(
                subsystem=subsystem,
                stem=stem,
                c_path=c_path,
                rs_path=rs_path,
                label=label,
            )
        )

    return sorted(pairs, key=lambda fp: fp.label)


def _c_path_to_label(c_path: Path) -> str:
    """Convert a C file path to a display label like 'ex/atom'."""
    try:
        rel = c_path.relative_to(NTOSKRNL_DIR)
        return str(rel.with_suffix(""))
    except ValueError:
        return c_path.stem


# ── Rust symbol attribution ──────────────────────────────────────────────────


def attribute_rust_symbols(
    rust_all_syms: dict[str, Symbol], file_pairs: list[FilePair]
) -> dict[str, dict[str, Symbol]]:
    """Attribute Rust symbols to source files by scanning .rs for #[no_mangle].

    Returns {label: {sym_name: Symbol}}.
    """
    # First, scan all .rs files to find #[no_mangle] pub fn/static declarations
    file_exports: dict[str, set[str]] = {}  # label → {function_name, ...}

    re_no_mangle_fn = re.compile(
        r'#\[no_mangle\].*?pub\s+(?:unsafe\s+)?(?:extern\s+"[^"]*"\s+)?fn\s+(\w+)',
        re.DOTALL,
    )
    re_no_mangle_static = re.compile(
        r"#\[no_mangle\]\s*pub\s+static\s+(?:mut\s+)?(\w+)",
    )
    re_export_name = re.compile(
        r'#\[export_name\s*=\s*"([^"]+)"\]',
    )

    for fp in file_pairs:
        if not fp.rs_path.exists():
            continue
        rs_text = fp.rs_path.read_text()
        names = set()

        for m in re_no_mangle_fn.finditer(rs_text):
            names.add(m.group(1))
        for m in re_no_mangle_static.finditer(rs_text):
            names.add(m.group(1))
        for m in re_export_name.finditer(rs_text):
            # export_name gives the decorated name; undecorate for matching
            export = m.group(1)
            # Try to undecorate i386 names
            if (export.startswith("_") and "@" in export) or (
                export.startswith("@") and "@" in export[1:]
            ):
                parts = export[1:].rsplit("@", 1)
                if parts[1].isdigit():
                    names.add(parts[0])
            elif export.startswith("_"):
                names.add(export[1:])
            else:
                names.add(export)

        file_exports[fp.label] = names

    # Now build per-file symbol dicts from the global Rust symbols
    result: dict[str, dict[str, Symbol]] = {}
    for fp in file_pairs:
        names = file_exports.get(fp.label, set())
        file_syms = {}
        for name in names:
            if name in rust_all_syms:
                file_syms[name] = rust_all_syms[name]
        result[fp.label] = file_syms

    return result


# ── Report generation ────────────────────────────────────────────────────────


def print_report(report: ObjdiffReport, verbose: bool = False, critical_only: bool = False) -> None:
    """Print human-readable report to stderr."""
    out = sys.stderr

    if report.baseline:
        print(
            f"\nobjdiff: Comparing Rust (current) vs C "
            f"(baseline: {report.baseline[:10]}) [{report.arch}]",
            file=out,
        )
    else:
        print(f"\nobjdiff: Comparing C vs Rust object files ({report.arch})", file=out)
    print(f"  File pairs: {len(report.file_pairs)}", file=out)
    if report.compile_failures:
        print(f"  Compile failures: {len(report.compile_failures)}", file=out)
    print(f"  C symbols extracted: {report.c_symbols_count}", file=out)
    print(f"  Rust symbols extracted: {report.rust_symbols_count}", file=out)
    print(file=out)

    # Group by severity
    critical = [d for d in report.differences if d.severity == "critical"]
    warnings = [d for d in report.differences if d.severity == "warning"]
    infos = [d for d in report.differences if d.severity == "info"]

    if critical:
        print(f"CRITICAL ({len(critical)}):", file=out)
        for d in critical:
            print(f"  {d.file}: {d.kind.upper()}", file=out)
            print(f"    {d.symbol_name}: C={d.c_value} Rust={d.rust_value}", file=out)
            if verbose:
                print(f"    {d.detail}", file=out)
        print(file=out)

    if not critical_only and warnings:
        print(f"WARNING ({len(warnings)}):", file=out)
        for d in warnings:
            print(f"  {d.file}: {d.kind.upper()}", file=out)
            print(f"    {d.symbol_name}: C={d.c_value} Rust={d.rust_value}", file=out)
            if verbose:
                print(f"    {d.detail}", file=out)
        print(file=out)

    if not critical_only and infos and verbose:
        print(f"INFO ({len(infos)}):", file=out)
        for d in infos:
            print(f"  {d.file}: {d.kind.upper()}", file=out)
            print(f"    {d.symbol_name}: C={d.c_value} Rust={d.rust_value}", file=out)
        print(file=out)

    # Compile failures
    if report.compile_failures:
        print(f"COMPILE FAILURES ({len(report.compile_failures)}):", file=out)
        for label, error in report.compile_failures:
            print(f"  {label}: {error[:200]}", file=out)
        print(file=out)

    # Summary
    print(
        f"Summary: {len(report.file_pairs)} files, "
        f"{len(critical)} critical, {len(warnings)} warnings, {len(infos)} info",
        file=out,
    )


def report_to_json(report: ObjdiffReport) -> str:
    """Convert report to JSON string."""
    data = {
        "arch": report.arch,
        "baseline": report.baseline,
        "file_count": len(report.file_pairs),
        "c_symbols": report.c_symbols_count,
        "rust_symbols": report.rust_symbols_count,
        "compile_failures": [
            {"file": label, "error": error} for label, error in report.compile_failures
        ],
        "differences": [
            {
                "kind": d.kind,
                "severity": d.severity,
                "symbol": d.symbol_name,
                "c_value": d.c_value,
                "rust_value": d.rust_value,
                "detail": d.detail,
                "file": d.file,
            }
            for d in report.differences
        ],
        "summary": {
            "critical": sum(1 for d in report.differences if d.severity == "critical"),
            "warning": sum(1 for d in report.differences if d.severity == "warning"),
            "info": sum(1 for d in report.differences if d.severity == "info"),
        },
    }
    return json.dumps(data, indent=2)


# ── Main pipeline ────────────────────────────────────────────────────────────


def run_objdiff(
    arch: str = "i386",
    target: str | None = None,
    verbose: bool = False,
    critical_only: bool = False,
    json_output: bool = False,
    baseline: str | None = None,
    deep: bool = False,
) -> ObjdiffReport:
    """Run the full objdiff pipeline.

    Args:
        baseline: Git commit hash. When set, extract C files at that commit
                  and compare them against current Rust symbols.
        deep: When True, analyze function prologues for stack frame and
              callee-saved register differences.
    """
    out = sys.stderr

    # Step 0: Discover file pairs
    print(f"Discovering C→Rust file pairs for {arch}...", file=out, flush=True)
    file_pairs = discover_file_pairs(arch, target)
    if not file_pairs:
        print("No file pairs found.", file=out)
        return ObjdiffReport(
            arch=arch,
            file_pairs=[],
            differences=[],
            compile_failures=[],
            c_symbols_count=0,
            rust_symbols_count=0,
        )
    print(f"  Found {len(file_pairs)} file pairs", file=out, flush=True)

    # Step 1: Extract compile flags
    print("Extracting compile flags from build.ninja...", file=out, flush=True)
    compile_flags = extract_compile_flags(arch)

    # Step 2: Compile C files (optionally from baseline commit)
    with tempfile.TemporaryDirectory(prefix="objdiff_") as tmpdir:
        tmp_path = Path(tmpdir)

        path_overrides = None
        if baseline:
            print(
                f"Extracting C files at baseline {baseline[:10]}...", file=out, end=" ", flush=True
            )
            baseline_dir = tmp_path / "baseline"
            baseline_dir.mkdir()
            path_overrides = extract_baseline_c_files(file_pairs, baseline, baseline_dir)
            print(f"{len(path_overrides)} files extracted", file=out, flush=True)
            if not path_overrides:
                print(f"Error: no C files found at commit {baseline}", file=out)
                return ObjdiffReport(
                    arch=arch,
                    file_pairs=file_pairs,
                    differences=[],
                    compile_failures=[],
                    c_symbols_count=0,
                    rust_symbols_count=0,
                )

        compile_count = len(path_overrides) if path_overrides else len(file_pairs)
        label = f"baseline {baseline[:10]}" if baseline else "current"
        print(f"Compiling {compile_count} C files ({label})...", file=out, end=" ", flush=True)
        c_obj_map, compile_failures = compile_c_files(
            file_pairs,
            arch,
            tmp_path,
            compile_flags,
            path_overrides=path_overrides,
        )
        print(f"done ({len(c_obj_map)} ok, {len(compile_failures)} failed)", file=out, flush=True)

        # Step 3: Extract Rust symbols
        print("Extracting Rust symbols from libkernel_rust.a...", file=out, end=" ", flush=True)
        rust_all_syms = extract_rust_symbols(arch)
        print(f"{len(rust_all_syms)} symbols", file=out, flush=True)

        # Attribute Rust symbols to source files
        rust_per_file = attribute_rust_symbols(rust_all_syms, file_pairs)

        # Deep analysis: analyze Rust prologues from the .a file
        if deep:
            print("Analyzing Rust prologues...", file=out, end=" ", flush=True)
            rust_lib = _find_rust_lib(arch)
            if rust_lib:
                analyze_prologues(rust_lib, rust_all_syms, arch)
                analyzed = sum(1 for s in rust_all_syms.values() if s.stack_frame_size >= 0)
                print(f"{analyzed} prologues parsed", file=out, flush=True)
            else:
                print("skipped (lib not found)", file=out, flush=True)

        # Step 4: Extract C symbols + Step 5: Calling convention analysis
        print(f"Analyzing {len(c_obj_map)} C object files...", file=out, flush=True)
        all_diffs: list[Difference] = []
        total_c_syms = 0

        # Load spec for cross-reference
        spec = None
        try:
            spec = parse_spec()
        except Exception:
            pass

        for fp in file_pairs:
            if fp.label not in c_obj_map:
                continue

            obj_path = c_obj_map[fp.label]

            # Extract C symbols
            c_syms = extract_c_symbols(obj_path, arch)
            total_c_syms += len(c_syms)

            # i386: analyze calling conventions via disassembly
            if arch == "i386":
                analyze_calling_conventions(obj_path, c_syms, arch)

            # Deep: analyze C prologues
            if deep:
                analyze_prologues(obj_path, c_syms, arch)

            # Get attributed Rust symbols for this file
            rust_file_syms = rust_per_file.get(fp.label, {})

            # Compare
            diffs = compare_symbols(c_syms, rust_file_syms, fp.label, arch, spec, deep=deep)
            all_diffs.extend(diffs)

    report = ObjdiffReport(
        arch=arch,
        file_pairs=file_pairs,
        differences=all_diffs,
        compile_failures=compile_failures,
        c_symbols_count=total_c_syms,
        rust_symbols_count=len(rust_all_syms),
        baseline=baseline,
    )

    # Output
    if json_output:
        print(report_to_json(report))
    else:
        print_report(report, verbose=verbose, critical_only=critical_only)

    return report
