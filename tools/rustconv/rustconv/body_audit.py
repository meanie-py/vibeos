"""body_audit.py — did a translated body keep what its C body DOES?

Call sealing (`--seal-calls`) makes a dropped call fail loudly at translation time. Crates
translated BEFORE sealing never had that check, and they lose statements silently. The case
that made this a tool: fastfat's `FatCreateNewDirent` lost the tail of its C body, including
`RtlSetBits` on the free-dirent bitmap. It compiled, mounted and passed smoke on three arches
for months while every new directory entry stayed FREE in memory. A later create that reused
deleted slots could be handed a live entry and overwrite another file's dirent (2026-09-26).

For every C function the module defines that has a Rust definition of the same name in the
module's crates, compare the C body's callees with the Rust body. A callee counts as KEPT when
its name appears in the Rust body, in a crate-local helper that body reaches (wrappers such as
cdrom's `DeviceEvtFileClose` -> `CdromDeviceEvtFileClose`, depth 3), or through a known
lowering (`RtlZeroMemory` -> `write_bytes`, `KeAcquireSpinLock` ->
`KeAcquireSpinLockRaiseToDpc`, ...). Callees in code the build compiles but never runs (a
disabled `#if` arm, a DUMMY_PSEH except handler) and in `#if DBG` code are OPTIONAL and never
reported. `_SEH2_FINALLY` blocks DO run under DUMMY_PSEH, so their calls stay required.

This is a lead generator for a human reading the C, not a gate: a macro whose expansion the
Rust open-codes under another name still reads as a drop. Findings are ranked so the ones that
lose STATE come first:

  stub-silent  the Rust body is a small fraction of the C and says nothing about it
  high         a dropped callee whose name mutates state (Set/Release/Unpin/Complete/...)
  medium       any other dropped function call
  low          a dropped memory op / ALL-CAPS macro with no recognised lowering
  stub-loud    a small body that bugchecks or returns NOT_IMPLEMENTED (known, not silent)
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path

import tree_sitter_rust
from tools.source_walk import source_files
from tree_sitter import Language, Parser

from .c_defs import c_functions, compiled_tus
from .dead_regions import dummy_pseh_except_spans, never_compiled_lines
from .export_audit import _repo_root_of, module_rust_crates
from .lex_mask import directive_spans, mask_code
from .profile_derive import ProfileDeriveError

_CALL_RE = re.compile(r"\b([A-Za-z_]\w*)\s*\(")
_IDENT_RE = re.compile(r"\b[A-Za-z_]\w*\b")
_C_KEYWORDS = frozenset(
    {
        *("if", "else", "for", "while", "switch", "return", "sizeof", "do", "case"),
        *("default", "goto", "typedef", "struct", "union", "enum", "defined"),
        *("__typeof__", "__alignof__", "_Alignof"),
    }
)
# Never a finding: tracing, assertions, SAL, and the SEH scaffolding itself. None of these
# changes what the function does in a release build.
_IGNORED = re.compile(
    r"^(?:DebugTrace|DebugDump|DebugUnwind|DbgPrint\w*|KdPrint\w*|TracePrint\w*|TraceLog\w*|"
    r"DPRINT\w*|TRACE\w*|WARN|ERR|FIXME|ASSERT\w*|NT_ASSERT\w*|NT_VERIFY\w*|PAGED_CODE|"
    r"UNREFERENCED_PARAMETER|UNREFERENCED_LOCAL_VARIABLE|DBG_UNREFERENCED\w*|_Analysis_\w+|"
    r"_SEH2_\w+|__try|__except|__finally|__leave|try_return|try_leave|TimerStart|TimerStop|"
    r"C_ASSERT|FIELD_OFFSET|RTL_FIELD_SIZE|ARRAYSIZE|RTL_NUMBER_OF\w*|_countof|warning|"
    r"__analysis_assume|__assume|UNIMPLEMENTED|DBGCHK\w*|WPP_\w+|DoTraceMessage|_Pragma|"
    r"except|GetExceptionCode|GetExceptionInformation|_exception_code|_exception_info|"
    r"ULongToPtr|PtrToUlong|PtrToUshort|UlongToPtr|LongToPtr|PtrToLong|HandleToUlong|"
    r"UlongToHandle|IntToPtr|PtrToInt|debugstr_\w+|wine_dbgstr_\w+|wine_dbg_sprintf|"
    r"pdump|pcheck|FIXME_\w*|TRACE_\w*|WARN_\w*|ERR_\w*|MAKEINTRESOURCE[AW]?|SUCCEEDED|FAILED|"
    r"HRESULT_FROM_WIN32|MAKELONG|MAKEWORD|LOWORD|HIWORD|LOBYTE|HIBYTE|MAKEWPARAM|MAKELPARAM)$"
)
# Names whose Rust spelling is legitimately different. Any listed pattern satisfies the callee.
_LOWERINGS: dict[str, tuple[str, ...]] = {
    "ZeroMemory": (r"write_bytes", r"zeroed", r"memset", r"RtlZeroMemory", r"fill\(\s*0"),
    "CopyMemory": (r"copy_nonoverlapping", r"ptr::copy", r"\bcopy\(", r"memcpy", r"RtlCopyMemory"),
    "MoveMemory": (r"ptr::copy", r"\bcopy\(", r"memmove", r"RtlMoveMemory"),
    "FillMemory": (r"write_bytes", r"memset", r"\.fill\("),
    "strcpyW": (r"wcscpy", r"lstrcpyW", r"copy", r"strcpyW"),
    "wcscpy": (r"wcscpy", r"lstrcpyW", r"copy"),
    "strcpy": (r"strcpy", r"lstrcpyA", r"copy"),
    "RtlZeroMemory": (r"write_bytes", r"zeroed", r"memset", r"fill\(\s*0"),
    "RtlSecureZeroMemory": (r"write_bytes", r"write_volatile", r"zeroed"),
    "RtlFillMemory": (r"write_bytes", r"memset", r"\.fill\("),
    "RtlFillMemoryUlong": (r"write_bytes", r"\.fill\(", r"write_unaligned", r"\bwrite\("),
    "memset": (r"write_bytes", r"zeroed", r"\.fill\("),
    "RtlCopyMemory": (r"copy_nonoverlapping", r"ptr::copy", r"\bcopy\(", r"copy_from", r"memcpy"),
    "RtlMoveMemory": (
        r"ptr::copy",
        r"\bcopy\(",
        r"copy_nonoverlapping",
        r"memmove",
        r"copy_within",
    ),
    "memcpy": (r"copy_nonoverlapping", r"ptr::copy", r"\bcopy\(", r"copy_from"),
    "memmove": (r"ptr::copy", r"\bcopy\(", r"copy_within"),
    "RtlEqualMemory": (r"==", r"memcmp", r"RtlCompareMemory"),
    "RtlCompareMemory": (r"memcmp", r"==", r"iter\(\)"),
    "IsEqualGUID": (r"==", r"memcmp", r"RtlCompareMemory", r"IsEqualGUID"),
    "InlineIsEqualGUID": (r"==", r"IsEqualGUID"),
    "KeAcquireSpinLock": (r"KeAcquireSpinLock\w*", r"KfAcquireSpinLock"),
    "KeReleaseSpinLock": (r"KeReleaseSpinLock\w*", r"KfReleaseSpinLock"),
    "KeRaiseIrql": (r"KfRaiseIrql", r"KeRaiseIrql\w*"),
    "KeLowerIrql": (r"KfLowerIrql", r"KeLowerIrql"),
    "ExInitializeWorkItem": (r"WorkerRoutine",),
    "InitializeListHead": (r"InitializeListHead", r"Flink"),
    "InsertTailList": (r"InsertTailList", r"Blink"),
    "InsertHeadList": (r"InsertHeadList", r"Flink"),
    "RemoveEntryList": (r"RemoveEntryList", r"Flink"),
    "RemoveHeadList": (r"RemoveHeadList", r"Flink"),
    "IsListEmpty": (r"IsListEmpty", r"Flink\s*==", r"==\s*\(?\*?\w*\)?\.?Flink"),
    "NT_SUCCESS": (r">=\s*0", r"NT_SUCCESS", r"< 0", r"<\s*0"),
    "min": (r"\.min\(", r"\bmin\b", r"cmp::min"),
    "max": (r"\.max\(", r"\bmax\b", r"cmp::max"),
    "CONTAINING_RECORD": (r"offset_of", r"CONTAINING_RECORD", r"\.sub\(", r"byte_sub"),
    "ObDereferenceObject": (r"ObDereferenceObject", r"ObfDereferenceObject"),
    "ObReferenceObject": (r"ObReferenceObject", r"ObfReferenceObject"),
    "IoCallDriver": (r"IoCallDriver", r"IofCallDriver"),
    "IoCompleteRequest": (r"IoCompleteRequest", r"IofCompleteRequest"),
    "IoGetCurrentIrpStackLocation": (r"CurrentStackLocation", r"IoGetCurrentIrpStackLocation"),
    "IoGetNextIrpStackLocation": (r"CurrentStackLocation", r"IoGetNextIrpStackLocation"),
    "IoSkipCurrentIrpStackLocation": (r"CurrentLocation", r"IoSkipCurrentIrpStackLocation"),
    "IoCopyCurrentIrpStackLocationToNext": (
        r"IoCopyCurrentIrpStackLocationToNext",
        r"copy_nonoverlapping",
        r"CompletionRoutine",
    ),
    "InterlockedIncrement": (r"Interlocked\w*", r"fetch_add", r"atomic"),
    "InterlockedDecrement": (r"Interlocked\w*", r"fetch_sub", r"atomic"),
    "InterlockedExchange": (r"Interlocked\w*", r"swap", r"atomic"),
    "InterlockedCompareExchange": (r"Interlocked\w*", r"compare_exchange", r"atomic"),
    "InterlockedExchangePointer": (r"Interlocked\w*", r"swap", r"atomic"),
    "InterlockedCompareExchangePointer": (r"Interlocked\w*", r"compare_exchange", r"atomic"),
    "InterlockedOr": (r"Interlocked\w*", r"fetch_or", r"atomic"),
    "InterlockedAnd": (r"Interlocked\w*", r"fetch_and", r"atomic"),
}
_MEMORY_OPS = frozenset(k for k in _LOWERINGS if "Memory" in k or k.startswith("mem"))
# A dropped callee whose NAME says it changes state is the class that corrupts things.
_STATE_VERB = re.compile(
    r"(Set|Clear|Delete|Free|Release|Acquire|Unpin|Pin|Repin|Complete|Insert|Remove|"
    r"Initialize|Uninitialize|Close|Dereference|Reference|Truncate|Flush|Purge|Unlock|Lock|"
    r"Raise|Add|Write|Mark|Queue|Signal|Cancel|Destroy|Deallocate|Allocate|Create|Open|"
    r"Unmap|Map|Commit|Update|Enter|Leave|Wait|Post|Send|Start|Stop|Enable|Disable|Put|"
    r"Push|Pop|Register|Unregister|Attach|Detach|Invalidate|Notify|Reset|Store|Save|"
    r"Restore|Move|Rename|Grow|Shrink|Extend|Zero)"
)
_LOUD_STUB = re.compile(
    r"(?i:bug_?check)|STATUS_NOT_IMPLEMENTED|STATUS_NOT_SUPPORTED|STATUS_INVALID_DEVICE_REQUEST|"
    r"unimplemented!|todo!|panic!|UNIMPLEMENTED|unreachable_unchecked"
)
# A body this far below its C is a stub unless it delegates, which the helper walk measures.
_STUB_RATIO = 0.15
_STUB_MIN_C_LINES = 30
_HELPER_DEPTH = 3

SEVERITY_ORDER = ("stub-silent", "high", "medium", "low", "stub-loud")


@dataclass
class Finding:
    function: str
    c_file: str
    rust_file: str
    severity: str
    c_lines: int
    rust_lines: int
    dropped: list[str] = field(default_factory=list)


def _line_of(text: str, pos: int) -> int:
    return text.count("\n", 0, pos) + 1


def _matching(text: str, open_idx: int, opener: str, closer: str) -> int | None:
    depth = 0
    for pos in range(open_idx, len(text)):
        ch = text[pos]
        if ch == opener:
            depth += 1
        elif ch == closer:
            depth -= 1
            if depth == 0:
                return pos
    return None


def _dbg_lines(text: str) -> set[int]:
    """Lines inside `#if DBG` / `#ifdef DBG` / `#if <module>DBG` arms: optional in release."""
    out: set[int] = set()
    stack: list[bool] = []
    for n, line in enumerate(text.splitlines(), 1):
        s = line.strip()
        m = re.match(r"#\s*(if|ifdef|ifndef|elif|else|endif)\b(.*)", s)
        if m:
            d, rest = m.group(1), m.group(2)
            if d in ("if", "ifdef"):
                stack.append(bool(re.search(r"\b\w*DBG\w*\b|\bDEBUG\b", rest)) and "!" not in rest)
            elif d == "ifndef":
                stack.append(False)
            elif d in ("else", "elif") and stack:
                stack[-1] = False
            elif d == "endif" and stack:
                stack.pop()
            continue
        if any(stack):
            out.add(n)
    return out


@dataclass
class CBody:
    name: str
    file: Path
    lines: int
    required: set[str]


def c_bodies(path: Path) -> dict[str, CBody]:
    """Each function this .c DEFINES, with the callees its LIVE body must perform."""
    text = path.read_text(encoding="utf-8", errors="replace")
    masked = mask_code(text)
    dead = never_compiled_lines(text)
    masked_lines = masked.splitlines()
    optional_lines = dead | _dbg_lines(text)
    except_spans = dummy_pseh_except_spans(text, masked)
    directive = directive_spans(text)
    out: dict[str, CBody] = {}
    for name in c_functions(text):
        for m in re.finditer(rf"\b{re.escape(name)}\s*\(", masked):
            close = _matching(masked, m.end() - 1, "(", ")")
            if close is None:
                continue
            brace = close + 1
            while brace < len(masked) and masked[brace] in " \t\r\n":
                brace += 1
            if brace >= len(masked) or masked[brace] != "{":
                continue
            end = _matching(masked, brace, "{", "}")
            if end is None:
                continue
            required: set[str] = set()
            for call in _CALL_RE.finditer(masked, brace, end):
                callee = call.group(1)
                if callee in _C_KEYWORDS or callee == name or _IGNORED.match(callee):
                    continue
                pos = call.start(1)
                if directive[pos]:  # `#pragma prefast(...)`, a nested `#define X(`
                    continue
                if _line_of(masked, pos) in optional_lines:
                    continue
                if any(a <= pos < b for a, b in except_spans):
                    continue
                required.add(callee)
            first, last = _line_of(masked, brace), _line_of(masked, end)
            # Only lines the build compiles: a dead `#else` arm is not body the Rust owes.
            live_lines = sum(
                1 for n in range(first, last + 1) if n not in dead and masked_lines[n - 1].strip()
            )
            out[name] = CBody(name, path, live_lines, required)
            break
    return out


# `#define KeWaitForMutexObject KeWaitForSingleObject`: an object-like ALIAS of a function.
_ALIAS_RE = re.compile(r"^[ \t]*#[ \t]*define[ \t]+([A-Za-z_]\w*)[ \t]+([A-Za-z_]\w*)[ \t]*$")
_DEFINE_RE = re.compile(r"^[ \t]*#[ \t]*define[ \t]+([A-Za-z_]\w*)\(([^)]*)\)(.*)$")
# Where a module's function-like macros come from: its own headers, and the DDK/NDK/XDK the
# drivers include. Read, not preprocessed: a macro is only ever EXPANDED here to ask which
# real calls it makes, so a slightly wrong arm costs a false lead, never a hidden drop.
_MACRO_INCLUDE_DIRS = (
    "sdk/include/xdk",
    "sdk/include/ddk",
    "sdk/include/ndk",
    "sdk/include/reactos",
)


def macro_table(module_dir: Path, repo: Path) -> dict[str, str]:
    """name -> body of every function-like `#define` visible to the module (first wins)."""
    table: dict[str, str] = {}
    headers = sorted(source_files(module_dir, "*.h"))
    for d in _MACRO_INCLUDE_DIRS:
        if (repo / d).is_dir():
            headers += sorted(source_files(repo / d, "*.h"))
    for h in headers:
        lines = h.read_text(encoding="utf-8", errors="replace").splitlines()
        i = 0
        while i < len(lines):
            m = _DEFINE_RE.match(lines[i])
            if not m:
                alias = _ALIAS_RE.match(lines[i])
                if alias:
                    # Modelled as a macro whose body calls the target.
                    table.setdefault(alias.group(1), f"{alias.group(2)}()")
                i += 1
                continue
            body = m.group(3)
            while body.rstrip().endswith("\\") and i + 1 < len(lines):
                i += 1
                body = body.rstrip()[:-1] + " " + lines[i]
            table.setdefault(m.group(1), mask_code(body))
            i += 1
    return table


def _macro_calls(name: str, macros: dict[str, str], depth: int = 3) -> set[str] | None:
    """The real function calls a macro expands to (None: not a macro)."""
    if name not in macros:
        return None
    out: set[str] = set()
    for callee in set(_CALL_RE.findall(macros[name])):
        if callee in _C_KEYWORDS or callee == name or _IGNORED.match(callee):
            continue
        inner = _macro_calls(callee, macros, depth - 1) if depth else None
        if inner is None:
            out.add(callee)
        else:
            out |= inner
    return out


def rust_definitions_all(text: str) -> list[tuple[str, str]]:
    """Every `fn` with a body, INCLUDING cfg-split twins of one name.

    `function_coverage.rust_definition_bodies` keys a dict, so a second definition of a name
    in the same file replaces the first. hal's `HalpQueryPciRegistryInfo` has a full body under
    `cfg(not(feature = "minihal"))` and a `return NULL` face under `cfg(feature = "minihal")`,
    mirroring the C's `#ifndef _MINIHAL_`; the audit saw only the three-line face and called a
    complete translation a stub. The union of the variants is what the C's arms are judged
    against.
    """
    source = text.encode()
    tree = Parser(Language(tree_sitter_rust.language())).parse(source)
    out: list[tuple[str, str]] = []
    stack = [tree.root_node]
    while stack:
        node = stack.pop()
        if node.type == "function_item":
            name = node.child_by_field_name("name")
            if name is not None and node.child_by_field_name("body") is not None:
                out.append(
                    (
                        source[name.start_byte : name.end_byte].decode(),
                        source[node.start_byte : node.end_byte].decode(),
                    )
                )
        if node.type in ("macro_definition", "macro_invocation"):
            continue
        stack.extend(node.named_children)
    return out


@dataclass
class RustIndex:
    bodies: dict[str, str]  # masked (comment/literal-free) body text per fn name
    files: dict[str, Path]
    macros: dict[str, str]


def rust_index(srcs: list[Path]) -> RustIndex:
    bodies: dict[str, str] = {}
    files: dict[str, Path] = {}
    macros: dict[str, str] = {}
    for f in srcs:
        text = f.read_text(encoding="utf-8", errors="replace")
        # Generated bindgen islands hold declarations, never translated bodies.
        if "bindgen" in f.name or f.name.startswith("types_"):
            continue
        for name, body in rust_definitions_all(text):
            # Masked, because translated Rust carries the original C in `//` comments: a
            # dropped call quoted in a comment must not count as kept.
            bodies[name] = bodies.get(name, "") + "\n" + mask_code(body)
            files.setdefault(name, f)
        for m in re.finditer(r"macro_rules!\s*(\w+)\s*\{", text):
            end = _matching(text, m.end() - 1, "{", "}")
            if end is not None:
                macros[m.group(1)] = mask_code(text[m.end() : end])
    return RustIndex(bodies, files, macros)


def _effective(name: str, idx: RustIndex, c_names: set[str]) -> str:
    """The Rust body plus the crate-local helpers and macros it reaches (not other ports)."""
    seen = {name}
    parts = [idx.bodies.get(name, "")]
    frontier = [parts[0]]
    for _ in range(_HELPER_DEPTH):
        nxt = []
        for text in frontier:
            for ident in set(_IDENT_RE.findall(text)):
                if ident in seen:
                    continue
                if ident in idx.macros and re.search(rf"\b{ident}!", text):
                    seen.add(ident)
                    parts.append(idx.macros[ident])
                    nxt.append(idx.macros[ident])
                elif ident in idx.bodies and ident not in c_names:
                    seen.add(ident)
                    parts.append(idx.bodies[ident])
                    nxt.append(idx.bodies[ident])
        frontier = nxt
    return "\n".join(parts)


def _snake(name: str) -> str:
    return re.sub(r"(?<=[a-z0-9])(?=[A-Z])", "_", name).lower()


_COM_MACRO = re.compile(r"^I[A-Z]\w*?_(\w+)$")


def _kept(callee: str, rust: str, idents: set[str]) -> bool:
    if callee in idents or _snake(callee) in idents:
        return True
    # A/W generic names resolve to one of their variants.
    if callee + "W" in idents or callee + "A" in idents:
        return True
    # Compiler intrinsics (`__readmsr`, `_ReadWriteBarrier`, `__inbyte`) are inline asm or
    # fences in Rust, under whatever name the port chose.
    if (callee.startswith("__") or callee.endswith("Barrier")) and re.search(
        r"asm!|compiler_fence|\bfence\(|_mm_|intrinsics", rust
    ):
        return True
    # A COM vtable macro `IShellFolder_Release(p)` is `((*(*p).lpVtbl).Release)(p)` in Rust.
    com = _COM_MACRO.match(callee)
    if com and com.group(1) in idents:
        return True
    return any(re.search(p, rust) for p in _LOWERINGS.get(callee, ()))


def _severity(callee: str) -> str:
    callee = callee.split("(", 1)[0] if "(->" not in callee else callee.split("(->", 1)[1]
    if callee in _MEMORY_OPS or (callee.isupper() and len(callee) > 2):
        return "low"
    return "high" if _STATE_VERB.search(callee) else "medium"


def audit(
    c_files: list[Path],
    rust_srcs: list[Path],
    repo: Path,
    *,
    glue_c: list[Path] | None = None,
    macros: dict[str, str] | None = None,
) -> list[Finding]:
    """`glue_c`: C the build still compiles (cohab shims). A Rust body that delegates to one
    of its functions has not dropped that work, so those bodies join the helper pool."""
    macros = macros or {}
    c_all: dict[str, CBody] = {}
    for f in c_files:
        c_all.update(c_bodies(f))
    idx = rust_index(rust_srcs)
    for f in glue_c or []:
        text = f.read_text(encoding="utf-8", errors="replace")
        masked = mask_code(text)
        for name in c_functions(text):
            if name in idx.bodies:
                continue
            m = re.search(rf"\b{re.escape(name)}\s*\([^;{{]*?\)\s*\{{", masked)
            if not m:
                continue
            end = _matching(masked, m.end() - 1, "{", "}")
            if end is not None:
                idx.bodies[name] = masked[m.end() - 1 : end + 1]
                idx.files.setdefault(name, f)
    c_names = set(c_all)
    findings: list[Finding] = []
    for name in sorted(c_all):
        if name not in idx.bodies:
            continue
        cb = c_all[name]
        rust = _effective(name, idx, c_names)
        rust_lines = max(1, rust.strip().count("\n") + 1) if rust.strip() else 0
        idents = set(_IDENT_RE.findall(rust))
        dropped = []
        for c in sorted(cb.required):
            if _kept(c, rust, idents):
                continue
            expansion = _macro_calls(c, macros)
            if expansion is not None:
                # A macro is kept when every real call it expands to is; a pure one
                # (SetFlag, FlagOn, ARGUMENT_PRESENT) makes none and cannot be dropped.
                lost = sorted(x for x in expansion if not _kept(x, rust, idents))
                if not lost:
                    continue
                c = f"{c}(->{','.join(lost)})"
            dropped.append(c)

        def rel(p: Path) -> str:
            try:
                return str(p.resolve().relative_to(repo))
            except ValueError:
                return str(p)

        common = (name, rel(cb.file), rel(idx.files[name]))
        if cb.lines >= _STUB_MIN_C_LINES and rust_lines < cb.lines * _STUB_RATIO:
            sev = "stub-loud" if _LOUD_STUB.search(rust) else "stub-silent"
            findings.append(Finding(*common, sev, cb.lines, rust_lines, dropped))
            continue
        if not dropped:
            continue
        sev = min((_severity(c) for c in dropped), key=SEVERITY_ORDER.index)
        findings.append(Finding(*common, sev, cb.lines, rust_lines, dropped))
    findings.sort(key=lambda f: (SEVERITY_ORDER.index(f.severity), f.c_file, f.function))
    return findings


def module_inputs(
    module_dir: Path, arch: str = "i386"
) -> tuple[list[Path], list[Path], list[Path]]:
    """The module's REPLACED C (on disk, out of the build) and its crates' Rust sources.

    Only flipped-out TUs are audited: a .c the build still compiles is what ships, and its
    same-named Rust (if any) is parked, so a difference there is not a lost statement.
    Hand-written cohab glue (`rust_cohab_*.c`) is never a translation source.
    """
    repo = _repo_root_of(module_dir)
    cml = (module_dir / "CMakeLists.txt").read_text(encoding="utf-8", errors="replace")
    if "add_cargo_" in cml and not re.search(r"^[ \t]*add_(?:library|executable)\(", cml, re.M):
        # A pure cargo link (lsass, smss, ...): the module compiles no C of its own.
        live: set[Path] = set()
    else:
        live = {p.resolve() for p in compiled_tus(module_dir, repo, arch=arch)}
    c_files = [
        p
        for p in sorted(source_files(module_dir, "*.c"))
        if p.resolve() not in live and not p.name.startswith("rust_")
    ]
    glue = sorted(p for p in live if p.is_relative_to(module_dir.resolve()))
    workspace = repo / "sdk" / "lib" / "rust"
    srcs: list[Path] = []
    for crate in module_rust_crates(module_dir, workspace):
        if (crate / "src").is_dir():
            srcs.extend(sorted(source_files(crate / "src", "*.rs")))
    return c_files, srcs, glue


_AUDIT_ROOTS = ("base", "dll", "drivers", "hal", "ntoskrnl", "subsystems", "win32ss", "boot")


def all_modules(repo: Path) -> list[Path]:
    """Every dir whose CMakeLists links a Rust crate (cheap text test, resolved later)."""
    out = []
    for root in _AUDIT_ROOTS:
        for cml in sorted(source_files(repo / root, "CMakeLists.txt")):
            text = cml.read_text(encoding="utf-8", errors="replace")
            if "rust_crate_sources(" in text or "add_cargo_" in text:
                out.append(cml.parent)
    return out


def render(module: str, findings: list[Finding], limit: int) -> str:
    counts = {s: sum(1 for f in findings if f.severity == s) for s in SEVERITY_ORDER}
    head = ", ".join(f"{n} {s}" for s, n in counts.items() if n)
    lines = [f"body-audit {module}: {head or 'clean'}"]
    for f in findings[:limit] if limit else findings:
        dropped = ", ".join(f.dropped[:8]) + (" …" if len(f.dropped) > 8 else "")
        lines.append(
            f"  [{f.severity}] {f.function}  C {f.c_lines}L / Rust {f.rust_lines}L"
            f"  ({f.c_file} -> {f.rust_file})"
        )
        if dropped:
            lines.append(f"      dropped: {dropped}")
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        prog="rustconv body-audit", description=(__doc__ or "").split("\n")[0]
    )
    ap.add_argument("modules", nargs="*", type=Path, help="module source dirs (with CMakeLists)")
    ap.add_argument(
        "--all",
        action="store_true",
        help="every module dir that links a Rust crate and has flipped C",
    )
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--limit", type=int, default=0, help="show at most N findings per module")
    ap.add_argument(
        "--min-severity", choices=SEVERITY_ORDER, default="stub-loud", help="hide lower ranks"
    )
    args = ap.parse_args(argv)
    cutoff = SEVERITY_ORDER.index(args.min_severity)
    modules = [m.resolve() for m in args.modules]
    if args.all:
        modules += all_modules(_repo_root_of(Path.cwd()))
    if not modules:
        ap.error("name a module dir or pass --all")
    report = {}
    skipped: list[str] = []
    for module in modules:
        try:
            c_files, srcs, glue = module_inputs(module)
        except ProfileDeriveError as exc:
            if not args.all:
                raise
            skipped.append(f"{module}: {exc}")
            continue
        if not srcs:
            if args.all:
                continue
            print(f"body-audit {module}: no Rust crate linked into this module", file=sys.stderr)
            return 2
        if not c_files:
            continue
        repo = _repo_root_of(module)
        macros = macro_table(module, repo)
        found = [
            f
            for f in audit(c_files, srcs, repo, glue_c=glue, macros=macros)
            if SEVERITY_ORDER.index(f.severity) <= cutoff
        ]
        key = str(module.relative_to(repo))
        report[key] = found
        if not args.json:
            print(render(key, found, args.limit))
    for line in skipped:
        print(f"body-audit SKIPPED (could not resolve the build): {line}", file=sys.stderr)
    if args.json:
        print(json.dumps({k: [asdict(f) for f in v] for k, v in report.items()}, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
