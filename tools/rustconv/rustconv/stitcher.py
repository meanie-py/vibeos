"""Assemble individually-translated functions into a complete .rs file."""

from __future__ import annotations

import re
from typing import TYPE_CHECKING

from .callgraph import FunctionDep
from .config import TARGETS, Target
from .resolver import ResolveResult

if TYPE_CHECKING:
    from .scanner import ScanResult


# For HAL target, collapse these hand-written-module symbols into `ddk` —
# they are all re-exported from ddk and HAL files (dma.rs, usage.rs, kdpci.rs)
# consistently import via ddk + hal_ddk only.
_HAL_KERNEL_API_MODULES = {
    "ke",
    "ex",
    "mm",
    "rtl",
    "ob",
    "ps",
    "interlocked",
    "base",
    "macros",
    "constants",
}

# Modules whose `use ntkrnl_sys::X::*` imports must be DROPPED for HAL targets,
# even if the LLM emits them.  These are ntoskrnl-rust internal modules.
_HAL_FORBIDDEN_USE_PATHS = (
    "use ntkrnl_sys::globals::",
    "use ntkrnl_sys::constants::",
    "use ntkrnl_sys::base::",
    "use ntkrnl_sys::macros::",
    "use ntkrnl_sys::ke::",
    "use ntkrnl_sys::ex::",
    "use ntkrnl_sys::mm::",
    "use ntkrnl_sys::rtl::",
    "use ntkrnl_sys::ob::",
    "use ntkrnl_sys::ps::",
    "use ntkrnl_sys::interlocked::",
)

# Bindgen surfaces these C macros / FORCEINLINEs as extern decls, so the
# resolver "finds" them in the catalog — but the linker has no symbol and the
# HAL system prompt teaches the LLM to inline these locally.  Strip from
# generated imports.
_HAL_FORBIDDEN_IMPORT_NAMES = {
    "InitializeListHead",
    "InsertTailList",
    "InsertHeadList",
    "RemoveEntryList",
    "RemoveHeadList",
    "RemoveTailList",
    "IsListEmpty",
    "KeAcquireSpinLock",
    "KeReleaseSpinLock",
    "KeInitializeSpinLock",
    "RtlCopyMemory",
    "RtlMoveMemory",
    "RtlZeroMemory",
    "RtlFillMemory",
    "RtlCompareMemory",
    "FIELD_OFFSET",
    "PAGE_ROUND_UP",
    "PAGE_ROUND_DOWN",
    "BYTES_TO_PAGES",
    "PAGE_ALIGN",
    "BYTE_OFFSET",
    "UlongToPtr",
    "PtrToUlong",
    "NT_SUCCESS",
    "NT_ERROR",
    "NT_WARNING",
    "NT_INFORMATION",
    "WRITE_PORT_UCHAR",
    "READ_PORT_UCHAR",
    "WRITE_PORT_USHORT",
    "READ_PORT_USHORT",
    "WRITE_PORT_ULONG",
    "READ_PORT_ULONG",
    "ASSERT",
    "ASSERTMSG",
    "DPRINT",
    "DPRINT1",
    "PAGED_CODE",
    "CONTAINING_RECORD",
    "RTL_CONSTANT_STRING",
    "RtlInitEmptyUnicodeString",
    "RtlInitUnicodeString",
    "MmGetMdlPfnArray",
    "MmGetMdlVirtualAddress",
    "MmGetMdlByteCount",
    "MmGetMdlByteOffset",
    "MmGetSystemAddressForMdlSafe",
    "InitializeObjectAttributes",
}


def _strip_forbidden_hal_uses(uses: list[str]) -> list[str]:
    """Drop ntkrnl_sys::{globals,constants,base,macros,ke,ex,...}::... imports."""
    return [u for u in uses if not any(u.startswith(p) for p in _HAL_FORBIDDEN_USE_PATHS)]


def _strip_forbidden_hal_imports(use_stmt: str) -> str | None:
    """Filter out forbidden symbol names from an `ntkrnl_sys::{ddk,hal_ddk}::{...}` import.

    Returns the cleaned statement, or None if the statement becomes empty.
    """
    m = re.match(r"^(use ntkrnl_sys::(ddk|hal_ddk)::)\{([^}]*)\};?$", use_stmt.strip())
    if not m:
        return use_stmt
    prefix, _module, body = m.group(1), m.group(2), m.group(3)
    items = [s.strip() for s in body.split(",") if s.strip()]
    # Drop literal-like junk (anything not starting with letter/underscore)
    items = [s for s in items if re.match(r"^[A-Za-z_]\w*$", s)]
    items = [s for s in items if s not in _HAL_FORBIDDEN_IMPORT_NAMES]
    if not items:
        return None
    return f"{prefix}{{{', '.join(items)}}};"


_CHAR_LITERAL = re.compile(r"'(?:\\.|[^\\'])'")


def _ends_in_block_comment(line: str, in_comment: bool) -> bool:
    """Whether a `/* */` comment is still open at the end of LINE (strings and `//` aware)."""
    i = 0
    while i < len(line):
        if in_comment:
            end = line.find("*/", i)
            if end < 0:
                return True
            in_comment, i = False, end + 2
        elif line.startswith("//", i):
            return False
        elif line.startswith("/*", i):
            in_comment, i = True, i + 2
        elif (char := _CHAR_LITERAL.match(line, i)) is not None:
            i = char.end()
        elif line[i] == '"':
            i += 1
            while i < len(line) and line[i] != '"':
                i += 2 if line[i] == "\\" else 1
            i += 1
        else:
            i += 1
    return in_comment


def _extract_use_statements(code: str) -> tuple[list[str], str]:
    """Extract `use` statements (single- and multi-line) from a code fragment.

    Handles both:
        use ntkrnl_sys::ddk::Foo;
        use ntkrnl_sys::ddk::{Foo, Bar, Baz};
        use ntkrnl_sys::ddk::{
            Foo, Bar, Baz,
            Qux,
        };

    Returns:
        Tuple of (list of use statement strings (each terminated with ';'),
                  code with uses and any orphan continuation lines removed).
    """
    uses: list[str] = []
    remaining_lines: list[str] = []
    lines = code.splitlines()
    i = 0
    in_comment = False
    while i < len(lines):
        line = lines[i]
        stripped = line.strip()
        # A C comment carried into the body wraps like prose: toolbar.c's
        # `/* if hot, attempt to draw with hot image list, if fails,\n use default image list */`
        # put a line starting `use ` inside a block comment, and hoisting it left the comment
        # open -- swallowing the other 168 functions of the file.
        starts_in_comment = in_comment
        in_comment = _ends_in_block_comment(line, in_comment)
        if not starts_in_comment and stripped.startswith("use "):
            # Accumulate until we see a `;` terminator with balanced braces.
            buf = [line]
            joined = stripped
            brace = stripped.count("{") - stripped.count("}")
            i += 1
            while not (joined.endswith(";") and brace == 0) and i < len(lines):
                nxt = lines[i]
                buf.append(nxt)
                joined = (joined + " " + nxt.strip()).strip()
                brace += nxt.count("{") - nxt.count("}")
                i += 1
            # Collapse to single-line representation for downstream dedup.
            uses.append(re.sub(r"\s+", " ", joined))
        else:
            remaining_lines.append(line)
            i += 1

    # Strip leading blank lines from remaining
    while remaining_lines and not remaining_lines[0].strip():
        remaining_lines.pop(0)

    return uses, "\n".join(remaining_lines)


def _extract_extern_blocks(code: str) -> tuple[list[str], str]:
    """Extract standalone extern block declarations from code.

    Finds `extern "X" { ... }` blocks and extracts them.
    Returns (list of extern block strings, code with blocks removed).
    """
    extern_blocks = []

    # Find extern blocks line by line
    lines = code.splitlines()
    result_lines = []
    i = 0
    while i < len(lines):
        line = lines[i]
        stripped = line.strip()

        # Detect start of extern block
        # `unsafe extern "X" {` is the Rust 2024 spelling the model increasingly writes;
        # missing it left the block (or, after a later strip, a bare `unsafe` line) behind.
        if re.match(r'(?:unsafe\s+)?extern\s+"[^"]*"\s*\{', stripped):
            block_lines = [line]
            brace_count = stripped.count("{") - stripped.count("}")
            i += 1
            while i < len(lines) and brace_count > 0:
                block_lines.append(lines[i])
                brace_count += lines[i].count("{") - lines[i].count("}")
                i += 1
            extern_blocks.append("\n".join(block_lines))
        else:
            result_lines.append(line)
            i += 1

    return extern_blocks, "\n".join(result_lines)


def _dedup_uses(use_statements: list[str]) -> list[str]:
    """Deduplicate and sort use statements."""
    # Drop brace-list items that aren't valid Rust import entries. The LLM
    # occasionally hallucinates a literal (e.g. `0i32`) into a
    # `use ...::{A, B, 0i32, C};` list; keeping it yields
    # `error: expected identifier, found 0i32`. A real entry is an
    # identifier (optionally `as alias`), `self`, or `*`.
    _valid_item = re.compile(r"^(?:[A-Za-z_]\w*(?:\s+as\s+[A-Za-z_]\w*)?|self|\*)$")

    def _sanitize_use(stmt: str) -> str:
        m = re.search(r"\{([^{}]*)\}", stmt)
        if not m:
            return stmt
        items = [x.strip() for x in m.group(1).split(",") if x.strip()]
        kept = [x for x in items if _valid_item.match(x)]
        return stmt[: m.start(1)] + ", ".join(kept) + stmt[m.end(1) :]

    # Normalize: remove trailing semicolons for comparison, re-add
    seen = set()
    deduped = []
    for stmt in use_statements:
        stmt = _sanitize_use(stmt)
        normalized = stmt.strip().rstrip(";") + ";"
        if normalized not in seen:
            seen.add(normalized)
            deduped.append(normalized)

    # Sort: hand-written modules first, then ddk, then core
    def sort_key(s):
        if "ntkrnl_sys::ddk" in s:
            return (1, s)
        elif "ntkrnl_sys::" in s:
            return (0, s)
        elif "core::" in s:
            return (2, s)
        return (3, s)

    return sorted(deduped, key=sort_key)


def _extract_decl_name(decl: str) -> str | None:
    """Extract the symbol name from an extern declaration for deduplication.

    E.g., 'fn CCTRACE(level: u32, format: *const i8, ...);' → 'CCTRACE'
          'static mut CcDirtyPages: u32;' → 'CcDirtyPages'
          'static CcZeroPage: usize;' → 'CcZeroPage'
    """
    # Match fn declarations
    m = re.match(r"fn\s+(\w+)\s*\(", decl.strip())
    if m:
        return m.group(1)
    # Match static declarations
    m = re.match(r"static\s+(?:mut\s+)?(\w+)\s*:", decl.strip())
    if m:
        return m.group(1)
    return None


def _merge_extern_blocks(blocks: list[str], defined_fns: set[str] | None = None) -> list[str]:
    """Merge extern blocks with the same linkage into one block each.

    Deduplicates by symbol name — if multiple functions declared the same extern
    with different parameter names, keep only the first. Additionally:
      * DROP a forward-decl whose symbol is a function the file itself defines
        (``defined_fns``). Per-function caches forward-declare file-local callees;
        emitting those alongside the real definition yields duplicate-definition
        errors and, when the decl is malformed (DeepSeek sometimes writes a
        ``#[no_mangle] pub unsafe extern "system" fn NAME(...);`` definition-shaped
        line), a hard parse error.
      * DROP anything that isn't a clean foreign decl (`fn NAME(...);` /
        `static NAME: T;`, no body braces) — robust against brace-counting slips.
    """
    defined_fns = defined_fns or set()
    by_linkage: dict[str, list[str]] = {}
    seen_names: dict[str, set[str]] = {}  # linkage → set of declared symbol names

    for block in blocks:
        # Extract linkage type
        m = re.match(r'extern\s+"([^"]+)"\s*\{', block.strip())
        if not m:
            continue
        linkage = m.group(1)

        # Extract inner declarations
        inner = re.search(r"\{(.*)\}", block, re.DOTALL)
        if not inner:
            continue

        decls_text = inner.group(1).strip()
        if decls_text:
            if linkage not in by_linkage:
                by_linkage[linkage] = []
                seen_names[linkage] = set()

            # Join multi-line declarations into single lines first
            # A declaration ends with ';' — accumulate lines until we hit one
            joined_decls = []
            current = ""
            for line in decls_text.splitlines():
                stripped = line.strip()
                if not stripped:
                    continue
                current = (current + " " + stripped).strip() if current else stripped
                if current.endswith(";"):
                    joined_decls.append(current)
                    current = ""
            if current:  # Leftover without semicolon
                joined_decls.append(current)

            for decl in joined_decls:
                # Only clean foreign decls survive: must terminate with ';' and
                # carry no body braces (a stray '}' or a leaked definition is junk).
                if "{" in decl or "}" in decl or not decl.rstrip().endswith(";"):
                    continue
                name = _extract_decl_name(decl)
                if name:
                    if name in defined_fns:
                        continue  # the file defines this — don't forward-declare it
                    if name in seen_names[linkage]:
                        continue  # Skip duplicate
                    seen_names[linkage].add(name)
                else:
                    continue  # un-named ⇒ not a recognizable decl; drop
                by_linkage[linkage].append(decl)

    result = []
    for linkage in sorted(by_linkage.keys()):
        decls = by_linkage[linkage]
        if decls:
            result.append(f'extern "{linkage}" {{')
            for decl in decls:
                result.append(f"    {decl}")
            result.append("}")

    return result


def _generate_imports_from_resolve(
    resolve: ResolveResult, target: Target | None = None
) -> list[str]:
    """Generate use statements from a file-level ResolveResult.

    For HAL: route kernel-API hand-written modules (ke/ex/mm/rtl/...) through
    `ddk` and HAL-internal types through `hal_ddk`. No `use ntkrnl_sys::ke::*`
    style globs.
    """
    if target is None:
        target = TARGETS["ntoskrnl"]
    if target.name == "shell32":
        # The C++ COM path uses the crate's hand-maintained Win32 substrate and
        # deterministic COM declarations. Never route it through ntkrnl_sys.
        # win32 is the shared substrate the free-function TUs are translated
        # against; without it every assembled module opens with a few hundred
        # E0425/E0412 that are pure import, not missing facts. A local definition
        # shadows a glob import, so this cannot disturb a module that has its own.
        return [
            "use crate::shellcom::*;",
            "use crate::types::*;",
            "use crate::win32::*;",
            "use core::ffi::c_void;",
        ]
    is_hal = target.name == "hal"

    # Build a merged view: module → set of names
    merged: dict[str, set[str]] = {}
    for module, group in resolve.imports.items():
        bucket = module
        if is_hal and module in _HAL_KERNEL_API_MODULES:
            bucket = "ddk"
        merged.setdefault(bucket, set()).update(s.name for s in group.symbols)

    uses: list[str] = []
    for module in sorted(merged):
        names = sorted(merged[module])
        if not names:
            continue
        if target.crate_bindgen_files and module == target.crate_module:
            uses.append(f"use crate::{module}::*;")
        elif is_hal and module in ("ddk", "hal_ddk"):
            # Selective imports — never glob — for HAL.
            uses.append(f"use ntkrnl_sys::{module}::{{{', '.join(names)}}};")
        elif module == "ddk":
            # ntoskrnl: selective imports from ddk
            uses.append(f"use ntkrnl_sys::ddk::{{{', '.join(names)}}};")
        else:
            # ntoskrnl hand-written modules: glob imports
            uses.append(f"use ntkrnl_sys::{module}::*;")

    # Always include core::ffi::c_void — used by almost every file
    uses.append("use core::ffi::c_void;")

    return uses


def _build_globals_section(scan: ScanResult, all_resolves: dict[str, ResolveResult]) -> str:
    """Build the globals section from scan result.

    This is a placeholder — the LLM may have emitted global declarations in
    individual function outputs. We collect them here for deduplication.
    """
    # Globals are typically emitted by the first function or as standalone.
    # The stitcher will preserve any static mut declarations found in function outputs.
    return ""


_FN_START = re.compile(
    r"#\[no_mangle\]|#\[cfg\("
    r"|(?:pub(?:\([^)]*\))?\s+)?(?:const\s+)?(?:unsafe\s+)?(?:extern\s+\"[^\"]+\"\s+)?fn\s"
)


def stitch_file(
    functions: dict[str, str],
    deps: list[FunctionDep],
    scan: ScanResult,
    file_resolve: ResolveResult,
    target: Target | None = None,
    preamble: str | None = None,
) -> str:
    """Assemble individually-translated functions into a complete .rs file.

    Args:
        functions: dict of function_name → Rust code from LLM
        deps: list of FunctionDep in topological order
        scan: ScanResult for the original C file
        file_resolve: File-level ResolveResult for overall imports
        preamble: Rust translation of the file-scope preamble (structs,
            typedefs, #define→const, static tables) from a synthetic
            preamble unit — emitted before the function bodies so the
            stitched file actually compiles. Any stray `use`/`extern`
            it contains is lifted into the normal dedup machinery.

    Returns:
        Complete .rs file contents as a string.
    """
    all_uses: list[str] = []
    all_externs: list[str] = []
    function_bodies: list[tuple[int, str, str]] = []  # (line_start, name, code)
    standalone_items: list[str] = []  # const, static, type alias outside functions
    preamble_body = ""

    if preamble and preamble.strip():
        p_uses, p_no_uses = _extract_use_statements(preamble)
        all_uses.extend(p_uses)
        p_externs, p_no_externs = _extract_extern_blocks(p_no_uses)
        all_externs.extend(p_externs)
        preamble_body = p_no_externs.strip()

    def _emit(name: str, line_start: int) -> None:
        code = functions[name]

        # Extract use statements
        uses, code_no_uses = _extract_use_statements(code)
        all_uses.extend(uses)

        # Extract extern blocks
        externs, code_no_externs = _extract_extern_blocks(code_no_uses)
        all_externs.extend(externs)

        # Extract standalone items (static mut, const, type alias) that appear
        # before the function definition
        fn_lines = code_no_externs.splitlines()
        fn_start_idx = None
        for i, line in enumerate(fn_lines):
            # Detect function definition start -- at COLUMN 0 only. This used to match the
            # STRIPPED line and missed `unsafe extern "C" fn` (no `pub`), so in setuplib's
            # UpdateDiskLayout the first hit was an INDENTED `#[cfg(feature = ..)]` statement
            # deep in the body: the whole function above it was hoisted as a "standalone
            # item" and its last statement and `}` were emitted 2,700 lines later, nesting
            # 20 functions inside it. Compiles; the functions vanish from the module.
            if _FN_START.match(line) or line.startswith("// CODE_SEG"):
                fn_start_idx = i
                break

        if fn_start_idx is not None and fn_start_idx > 0:
            fn_preamble = "\n".join(fn_lines[:fn_start_idx]).strip()
            if fn_preamble:
                standalone_items.append(fn_preamble)
            code_no_externs = "\n".join(fn_lines[fn_start_idx:])

        function_bodies.append((line_start, name, code_no_externs.strip()))

    emitted: set[str] = set()
    for dep in deps:
        if dep.name not in functions:
            continue
        _emit(dep.name, dep.line_start)
        emitted.add(dep.name)

    # Safety net: NEVER drop a translated function. Any name present in
    # `functions` (paid LLM output) but absent from the topological `deps`
    # list — e.g. vtable-only methods or indirectly-called helpers the
    # call-graph resolver failed to enumerate — would otherwise be silently
    # discarded (the loop above only emits names that appear in `deps`).
    # On large files this dropped ~16% of functions (mlang: 27/169). Emit any
    # leftover translated bodies after the source-ordered ones (a large
    # line_start sorts them to the end; order is compile-irrelevant).
    for name in functions:
        if name in emitted:
            continue
        _emit(name, 10**9)
        emitted.add(name)

    # Sort function bodies by original C source line order
    function_bodies.sort(key=lambda x: x[0])

    # Assemble the file
    parts = []

    # 1. Use statements — generate from file_resolve + collect any LLM-emitted ones
    resolve_uses = _generate_imports_from_resolve(file_resolve, target=target)
    all_uses_combined = resolve_uses + all_uses

    # HAL: drop forbidden module paths + scrub forbidden symbol names from ddk/hal_ddk imports
    if target is not None and target.name == "hal":
        all_uses_combined = _strip_forbidden_hal_uses(all_uses_combined)
        scrubbed: list[str] = []
        for u in all_uses_combined:
            cleaned = _strip_forbidden_hal_imports(u)
            if cleaned is not None:
                scrubbed.append(cleaned)
        all_uses_combined = scrubbed

    deduped_uses = _dedup_uses(all_uses_combined)
    if deduped_uses:
        for stmt in deduped_uses:
            parts.append(stmt)
        parts.append("")

    # 2. Merged extern blocks — drop forward-decls of functions the file defines
    defined_fn_names = {name for (_, name, _) in function_bodies}
    merged_externs = _merge_extern_blocks(all_externs, defined_fns=defined_fn_names)
    if merged_externs:
        for line in merged_externs:
            parts.append(line)
        parts.append("")

    # 3. File-scope preamble (translated structs / typedefs / #define→const
    #    / static tables). Emitted before standalone items + functions so
    #    the stitched file actually compiles. (Marker-independent: derived
    #    from scan.preamble_text; see scanner.scan_file / build_preamble_prompt.)
    if preamble_body:
        parts.append("// ---- file-scope preamble (translated) ----")
        parts.append(preamble_body)
        parts.append("")

    # 4. Standalone items (static mut, const, etc.) — deduped
    seen_items = set()
    for item in standalone_items:
        normalized = item.strip()
        if normalized and normalized not in seen_items:
            seen_items.add(normalized)
            parts.append(normalized)
            parts.append("")

    # 5. Functions in source order
    for _line_start, _name, code in function_bodies:
        parts.append(code)
        parts.append("")

    return "\n".join(parts)
