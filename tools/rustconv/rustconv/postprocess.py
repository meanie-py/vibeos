"""Post-processor for LLM-generated Rust translations.

Applies SAFE mechanical transforms to fix predictable differences between
what the LLM generates and what the ntkrnl-sys bindgen output expects.

NOTE: Most union field access fixes (.__bindgen_anon_N) are context-dependent
and handled by the system prompt.  However, a few well-known unions are
universal enough to fix mechanically:
  - LARGE_INTEGER/ULARGE_INTEGER: .LowPart/.HighPart → .__bindgen_anon_1.LowPart/HighPart
  - IO_STATUS_BLOCK: .Status → .__bindgen_anon_1.Status
"""

from __future__ import annotations

import re
import sys
from pathlib import Path
from typing import TYPE_CHECKING

from .abi_enforce import enforce_abi_from_c
from .abi_map import build_win32k_abi_map

if TYPE_CHECKING:
    from .scanner import CGlobal
    from .spec_parser import SpecEntry


# ============================================================================
# Type name fixes: C names → bindgen underscore-prefixed names
# ============================================================================

# Types that need underscore prefix (bindgen generates _STRUCT_NAME)
UNDERSCORE_PREFIX_TYPES = {
    "HANDLE_TABLE_ENTRY": "_HANDLE_TABLE_ENTRY",
    "OBJECT_HEADER": "_OBJECT_HEADER",
    "OBJECT_TYPE": "_OBJECT_TYPE",
    "OBJECT_HANDLE_INFORMATION": "_OBJECT_HANDLE_INFORMATION",
    "OBJECT_HANDLE_COUNT_DATABASE": "_OBJECT_HANDLE_COUNT_DATABASE",
    "OBJECT_HANDLE_COUNT_ENTRY": "_OBJECT_HANDLE_COUNT_ENTRY",
    "OBJECT_ATTRIBUTES": "_OBJECT_ATTRIBUTES",
    "ACCESS_STATE": "_ACCESS_STATE",
    "AUX_ACCESS_DATA": "_AUX_ACCESS_DATA",
    "KAPC_STATE": "_KAPC_STATE",
    "GENERIC_MAPPING": "_GENERIC_MAPPING",
    "SECURITY_DESCRIPTOR": "_SECURITY_DESCRIPTOR",
    "UNICODE_STRING": "_UNICODE_STRING",
    "OBP_LOOKUP_CONTEXT": "_OBP_LOOKUP_CONTEXT",
    "OBP_FIND_HANDLE_DATA": "_OBP_FIND_HANDLE_DATA",
    "OBP_CLOSE_HANDLE_CONTEXT": "_OBP_CLOSE_HANDLE_CONTEXT",
    "OBP_SET_HANDLE_ATTRIBUTES_CONTEXT": "_OBP_SET_HANDLE_ATTRIBUTES_CONTEXT",
    "OB_TEMP_BUFFER": "_OB_TEMP_BUFFER",
    "OBJECT_HANDLE_ATTRIBUTE_INFORMATION": "_OBJECT_HANDLE_ATTRIBUTE_INFORMATION",
    "OBJECT_CREATE_INFORMATION": "_OBJECT_CREATE_INFORMATION",
    "EPROCESS": "_EPROCESS",
    "ETHREAD": "_ETHREAD",
    "CLIENT_ID": "_CLIENT_ID",
    "IO_STATUS_BLOCK": "_IO_STATUS_BLOCK",
}

# Pointer type aliases that DON'T need prefix
POINTER_TYPE_KEEP = {
    "POBJECT_HEADER",
    "POBJECT_TYPE",
    "POBJECT_CREATE_INFORMATION",
    "POBJECT_HEADER_HANDLE_INFO",
    "POBJECT_HEADER_QUOTA_INFO",
    "POBJECT_HEADER_NAME_INFO",
    "POBJECT_HEADER_CREATOR_INFO",
    "POBJECT_HANDLE_COUNT_DATABASE",
    "POBJECT_HANDLE_COUNT_ENTRY",
    "PHANDLE_TABLE",
    "PHANDLE_TABLE_ENTRY",
    "PACCESS_STATE",
    "PAUX_ACCESS_DATA",
    "PGENERIC_MAPPING",
    "PISECURITY_DESCRIPTOR",
    "PSECURITY_DESCRIPTOR",
    "PUNICODE_STRING",
    "PCHAR",
    "PEPROCESS",
    "PETHREAD",
    "PVOID",
    "PCVOID",
}


def fix_type_names(code: str) -> str:
    """Replace bare struct type names with underscore-prefixed versions.

    Only replaces when used as a type (after ':', 'as', in generics, etc.),
    not when used as a pointer type alias (POBJECT_HEADER stays).
    """
    for c_name, bindgen_name in UNDERSCORE_PREFIX_TYPES.items():
        code = re.sub(
            rf"(?<![A-Za-z_])(?<!P){re.escape(c_name)}(?![A-Za-z0-9_])",
            bindgen_name,
            code,
        )
    return code


# ============================================================================
# Import fixes
# ============================================================================


def fix_imports(code: str, target_name: str = "ntoskrnl") -> str:
    """Fix common import issues.

    For HAL target, the base/constants/globals modules are ntoskrnl-rust-internal
    and don't exist for hal-rust crate — skip those additions.  Also strip them
    if the LLM emitted them.
    """
    # Replace ddk::* with a comment reminder (it conflicts with base::*)
    if "use ntkrnl_sys::ddk::*;" in code:
        code = code.replace(
            "use ntkrnl_sys::ddk::*;",
            "// TODO: Replace ddk::* with selective imports (conflicts with base::*)\n"
            "// use ntkrnl_sys::ddk::{...};",
        )

    if target_name == "hal":
        # Strip ntoskrnl-rust internal module imports — HAL doesn't have them.
        for forbidden in (
            "use ntkrnl_sys::globals::*;",
            "use ntkrnl_sys::constants::*;",
            "use ntkrnl_sys::base::*;",
            "use ntkrnl_sys::macros::*;",
        ):
            code = re.sub(rf"^{re.escape(forbidden)}\s*\n", "", code, flags=re.MULTILINE)
    else:
        # Remove NonPagedPool/PagedPool from ddk imports (ntoskrnl: they're in mm::*)
        code = code.replace("ntkrnl_sys::ddk::NonPagedPool, ", "")
        code = code.replace("ntkrnl_sys::ddk::PagedPool,\n", "")
        code = code.replace(", ntkrnl_sys::ddk::PagedPool", "")
        code = code.replace("ntkrnl_sys::ddk::NonPagedPool,\n", "")

    # Ensure core::ffi::c_void is imported
    if "c_void" in code and "core::ffi::c_void" not in code:
        code = re.sub(
            r"(use core::ptr::\{[^}]+\};)",
            r"\1\nuse core::ffi::c_void;",
            code,
        )

    if target_name != "hal":
        # Ensure base::* is imported (provides NTSTATUS, BOOLEAN, ULONG etc.)
        if "use ntkrnl_sys::base::*;" not in code and "ntkrnl_sys::ddk::" in code:
            # Insert at top of file (before first use statement)
            first_use = re.search(r"^use ", code, re.MULTILINE)
            if first_use:
                code = (
                    code[: first_use.start()]
                    + "use ntkrnl_sys::base::*;\n"
                    + code[first_use.start() :]
                )

        # Ensure constants::* is imported (provides STATUS_*, IO_*, TAG_*, etc.)
        if "use ntkrnl_sys::constants::*;" not in code and "ntkrnl_sys::ddk::" in code:
            first_use = re.search(r"^use ", code, re.MULTILINE)
            if first_use:
                code = (
                    code[: first_use.start()]
                    + "use ntkrnl_sys::constants::*;\n"
                    + code[first_use.start() :]
                )

        # Ensure globals::* is imported (provides extern global declarations)
        if "use ntkrnl_sys::globals::*;" not in code and "ntkrnl_sys::ddk::" in code:
            first_use = re.search(r"^use ", code, re.MULTILINE)
            if first_use:
                code = (
                    code[: first_use.start()]
                    + "use ntkrnl_sys::globals::*;\n"
                    + code[first_use.start() :]
                )

    return code


# ============================================================================
# Bogus ddk import filtering
# ============================================================================

# Names that should NEVER appear as ddk imports — they are C struct field names,
# parameter names, or FORCEINLINE functions that the LLM hallucinates as types.
# We detect these by checking: (a) against a known-bad list, and (b) names that
# start with lowercase or are very short single-word identifiers.
KNOWN_BOGUS_DDK_IMPORTS = {
    # Struct field names commonly confused with types
    "FileOffset",
    "Owner",
    "Locked",
    "Flags",
    "NextEntry",
    "Pinned",
    "Allocated",
    "Offset",
    "Reserved",
    "Index",
    "WatchTree",
    "Version",
    "Output",
    "ReadAhead",
    "RetryForceCheckPerFile",
    # FORCEINLINE functions not in bindgen
    "KeGetCurrentThread",
    "KeGetCurrentIrql",
    # Parameter names
    "Context1",
    "Context2",
    "LogHandle",
    "DirtyPageRoutine",
    "FileObject",
    "OldestLsn",
    "Bcb",
    "iBcb",
    # C macros / constants that don't exist as ddk symbols
    "BooleanFlagOn",
    "min",
    "max",
}


def fix_debug_macros(code: str) -> str:
    """Replace C debug macros with Rust equivalents.

    DPRINT/DPRINT1 → no-op (debug printing, not needed in Rust translation).
    UNIMPLEMENTED → unimplemented!() (already a Rust macro, just needs ! suffix).
    DbgPrint → no-op (kernel debug print).
    offset_of → core::mem::offset_of (requires Rust 1.77+).
    """
    # Replace entire lines containing DPRINT1!( or DPRINT!( with a no-op comment.
    # The LLM generates complex macro calls with nested parens, format strings etc.
    # that are hard to regex-match. Simplest safe fix: replace the entire statement.
    lines = code.split("\n")
    result_lines = []
    skip_until_semicolon = False
    for line in lines:
        stripped = line.strip()
        if skip_until_semicolon:
            if stripped.endswith(";") or stripped.endswith(");"):
                skip_until_semicolon = False
            continue

        if (
            re.match(r"DPRINT1?\!?\(", stripped)
            or stripped.startswith("DPRINT1!(")
            or stripped.startswith("DPRINT!(")
        ):
            # Single-line case
            if stripped.endswith(";") or stripped.endswith(");"):
                indent = line[: len(line) - len(line.lstrip())]
                result_lines.append(f"{indent}// DPRINT removed")
            else:
                # Multi-line: skip until semicolon
                indent = line[: len(line) - len(line.lstrip())]
                result_lines.append(f"{indent}// DPRINT removed")
                skip_until_semicolon = True
        elif "/* DPRINT1 */" in stripped or "/* DPRINT */" in stripped:
            # Previously partially mangled by old regex — replace entire line
            # but only if it contains leftover garbage
            if stripped == "/* DPRINT1 */" or stripped == "/* DPRINT */":
                result_lines.append(line)
            else:
                indent = line[: len(line) - len(line.lstrip())]
                result_lines.append(f"{indent}// DPRINT removed")
        else:
            result_lines.append(line)

    code = "\n".join(result_lines)

    # UNIMPLEMENTED! → unimplemented!()
    code = code.replace("UNIMPLEMENTED!();", "unimplemented!();")
    code = code.replace("UNIMPLEMENTED!()", "unimplemented!()")
    code = code.replace("UNIMPLEMENTED!", "unimplemented!()")

    # offset_of! → core::mem::offset_of! (stabilized in Rust 1.77)
    code = code.replace("offset_of!(", "core::mem::offset_of!(")
    # Avoid double-qualifying
    code = code.replace("core::mem::core::mem::offset_of!", "core::mem::offset_of!")

    return code


def fix_bogus_ddk_imports(code: str) -> str:
    """Remove bogus ddk imports — struct field names, parameter names, etc.

    The LLM sometimes generates `use ntkrnl_sys::ddk::FileOffset;` when
    FileOffset is actually a struct field name or parameter name, not a type.
    """

    def clean_ddk_import(match: re.Match) -> str:
        prefix = match.group(1)
        body = match.group(2)
        suffix = match.group(3)

        items = [item.strip() for item in body.split(",")]
        items = [item for item in items if item]

        filtered = [item for item in items if item not in KNOWN_BOGUS_DDK_IMPORTS]

        if len(filtered) == len(items):
            return match.group(0)

        if not filtered:
            return ""

        new_body = ", ".join(filtered)
        if "\n" in body:
            new_body = "\n    " + ",\n    ".join(filtered) + ",\n"

        return f"{prefix}{new_body}{suffix}"

    code = re.sub(
        r"(use ntkrnl_sys::ddk::\{)(.*?)(\};)",
        clean_ddk_import,
        code,
        flags=re.DOTALL,
    )

    # Also handle single-item imports: use ntkrnl_sys::ddk::FileOffset;
    for bogus in KNOWN_BOGUS_DDK_IMPORTS:
        code = re.sub(
            rf"^use ntkrnl_sys::ddk::{re.escape(bogus)};\n?",
            "",
            code,
            flags=re.MULTILINE,
        )

    # Remove any resulting blank lines from removed imports
    code = re.sub(r"\n{3,}", "\n\n", code)

    return code


# ============================================================================
# Enum variant name fixes
# ============================================================================

# Enum name → list of variant names (without prefix)
# Bindgen generates _ENUM_NAME_VariantName for each variant.
#
# NOTE: POOL_TYPE is NOT here — PagedPool/NonPagedPool are available as
# short-name constants from ntkrnl_sys::mm::* and should stay as-is.
ENUM_VARIANTS: dict[str, list[str]] = {
    "OB_OPEN_REASON": [
        "ObCreateHandle",
        "ObOpenHandle",
        "ObDuplicateHandle",
        "ObInheritHandle",
    ],
    "OBJECT_INFORMATION_CLASS": [
        "ObjectBasicInformation",
        "ObjectNameInformation",
        "ObjectTypeInformation",
        "ObjectTypesInformation",
        "ObjectHandleFlagInformation",
        "ObjectSessionInformation",
        "MaxObjectInfoClass",
    ],
    "SECURITY_OPERATION_CODE": [
        "SetSecurityDescriptor",
        "QuerySecurityDescriptor",
        "DeleteSecurityDescriptor",
        "AssignSecurityDescriptor",
    ],
    "EVENT_TYPE": [
        "NotificationEvent",
        "SynchronizationEvent",
    ],
    "KWAIT_REASON": [
        "Executive",
        "FreePage",
        "PageIn",
        "PoolAllocation",
        "DelayExecution",
        "Suspended",
        "UserRequest",
        "WrExecutive",
        "WrFreePage",
        "WrPageIn",
        "WrPoolAllocation",
        "WrDelayExecution",
        "WrSuspended",
        "WrUserRequest",
        "WrEventPair",
        "WrQueue",
        "WrLpcReceive",
        "WrLpcReply",
        "WrVirtualMemory",
        "WrPageOut",
        "WrRendezvous",
        "WrKeyedEvent",
        "WrTerminated",
        "WrProcessInSwap",
        "WrCpuRateControl",
        "WrCalloutStack",
        "WrKernel",
        "WrResource",
        "WrPushLock",
        "WrMutex",
        "WrQuantumEnd",
        "WrDispatchInt",
        "WrPreempted",
        "WrYieldExecution",
        "WrFastMutex",
        "WrGuardedMutex",
        "WrRundown",
        "MaximumWaitReason",
    ],
    "PP_NPAGED_LOOKASIDE_NUMBER": [
        "LookasideSmallIrpList",
        "LookasideLargeIrpList",
        "LookasideMdlList",
        "LookasideCreateInfoList",
        "LookasideNameBufferList",
        "LookasideTwilightList",
        "LookasideCompletionList",
        "LookasideMaximumList",
    ],
}


# Enum constants that should be replaced with their integer literals.
# These are enum variants from C that bindgen doesn't always expose cleanly,
# or that the LLM uses by name but aren't available as standalone constants.
ENUM_LITERAL_REPLACEMENTS: dict[str, str] = {
    "KernelMode": "0i32",
    "UserMode": "1i32",
    "LockQueueMasterLock": "17i32",
    "LockQueueWorkQueueLock": "6i32",
}


def fix_enum_variants(code: str) -> str:
    """Fix enum variant names to match bindgen output.

    Bindgen generates _ENUM_NAME_VariantName, not just VariantName.

    Skips:
    - Already-prefixed names
    - const alias declarations (e.g., `const ObCreateHandle: ... = _OB_OPEN_REASON_...`)
    - Names inside comments (lines starting with //)
    """
    for enum_name, variants in ENUM_VARIANTS.items():
        prefixed = f"_{enum_name}_"
        for variant in variants:
            # Replace standalone variant name, but NOT:
            # - already prefixed with _ENUM_NAME_
            # - preceded by `const ` (const alias declaration)
            # - inside // comments
            def replace_variant(
                m: re.Match, *, code: str = code, prefixed: str = prefixed, variant: str = variant
            ) -> str:
                # Check if this line is a comment
                line_start = code.rfind("\n", 0, m.start()) + 1
                line_text = code[line_start : m.start()]
                if "//" in line_text:
                    return m.group(0)
                # Check if preceded by `const ` on the same line
                if re.search(r"\bconst\s+$", line_text):
                    return m.group(0)
                return f"{prefixed}{variant}"

            code = re.sub(
                rf"(?<!{re.escape(prefixed)})(?<![A-Za-z_]){re.escape(variant)}(?![A-Za-z0-9_])",
                replace_variant,
                code,
            )
            # Fix double-application
            code = code.replace(f"{prefixed}{prefixed}{variant}", f"{prefixed}{variant}")

    return code


# ============================================================================
# Portable type width fixes (ULONG_PTR is u32 on i386, u64 on 64-bit)
# ============================================================================


def fix_portable_types(code: str) -> str:
    """Fix hardcoded u64 casts on OBJ_* constants.

    ObAttributes in _HANDLE_TABLE_ENTRY is ULONG_PTR — u32 on i386, u64 on 64-bit.
    The LLM often generates `as u64` which breaks on i386.
    Replace with `as _` (type-inferred) which adapts to the receiving type.
    Note: can't use `as ULONG_PTR` because base::ULONG_PTR is usize, not
    the same as bindgen's ULONG_PTR (c_ulonglong on 64-bit, c_ulong on i386).
    """
    # On lines containing ObAttributes, replace OBJ_* as u64 with OBJ_* as _
    lines = code.split("\n")
    new_lines = []
    for line in lines:
        if "ObAttributes" in line:
            line = re.sub(r"(OBJ_\w+(?:\))?)\s+as\s+u64", r"\1 as _", line)
        new_lines.append(line)
    return "\n".join(new_lines)


# ============================================================================
# Bindgen anonymous union field access fixes
# ============================================================================


def fix_bindgen_anon_field_access(code: str) -> str:
    """Fix field access on well-known unions that need .__bindgen_anon_1.

    LARGE_INTEGER / ULARGE_INTEGER:
      .LowPart  → .__bindgen_anon_1.LowPart
      .HighPart → .__bindgen_anon_1.HighPart
      (.QuadPart is a direct field on the union — NOT changed)

    IO_STATUS_BLOCK:
      .Status → .__bindgen_anon_1.Status
      (.Information is a direct field — NOT changed)

    The regex matches field access preceded by ')' or a word char (i.e. a
    real expression, not the start of a line or a comment).  Comment lines
    (starting with //) are skipped entirely.  Already-correct patterns
    (.__bindgen_anon_1.Field) are left alone (idempotent).
    """
    lines = code.split("\n")
    result = []
    for line in lines:
        stripped = line.lstrip()
        # Skip comment lines
        if stripped.startswith("//"):
            result.append(line)
            continue

        # LARGE_INTEGER / ULARGE_INTEGER: .LowPart and .HighPart
        # Match: (word-char or ')') + .LowPart/.HighPart, NOT already preceded
        # by __bindgen_anon_1.  Use negative lookbehind on __bindgen_anon_1.
        line = re.sub(
            r"(?<!__bindgen_anon_1)(?<=[)\w])\.(LowPart|HighPart)(?![A-Za-z0-9_])",
            r".__bindgen_anon_1.\1",
            line,
        )

        # IO_STATUS_BLOCK: .Status
        # Match: .Status that is NOT .StatusBlock, .StatusPointer, etc.
        # and NOT already inside __bindgen_anon_1.Status
        line = re.sub(
            r"(?<!__bindgen_anon_1)(?<=[)\w])\.Status(?![A-Za-z0-9_])",
            ".__bindgen_anon_1.Status",
            line,
        )

        result.append(line)
    return "\n".join(result)


# ============================================================================
# Enum literal replacements
# ============================================================================


def fix_enum_literal_replacements(code: str) -> str:
    """Replace bare enum variant names with their integer literals.

    Some C enum values (KernelMode, UserMode, LockQueue*) are used directly
    in code but may not be available as Rust constants.  Replace them with
    the known integer values.

    Only replaces standalone identifiers (not inside comments or strings),
    not when they appear as part of a larger identifier.
    """
    for name, literal in ENUM_LITERAL_REPLACEMENTS.items():
        # Replace standalone uses, skip comments and already-qualified names
        def replace_literal(m: re.Match, *, code: str = code, literal: str = literal) -> str:
            # Check if this is inside a comment
            line_start = code.rfind("\n", 0, m.start()) + 1
            line_prefix = code[line_start : m.start()]
            if "//" in line_prefix:
                return m.group(0)
            # Don't replace in string literals (rough check)
            if '"' in line_prefix and line_prefix.count('"') % 2 == 1:
                return m.group(0)
            return literal

        code = re.sub(
            rf"(?<![A-Za-z_]){re.escape(name)}(?![A-Za-z0-9_])",
            replace_literal,
            code,
        )

    return code


# ============================================================================
# Export #[no_mangle] fixes
# ============================================================================


def fix_exports(code: str, exports: dict[str, SpecEntry]) -> str:
    """Add #[no_mangle] to spec-exported function definitions that lack it.

    The LLM often generates `pub unsafe extern "system" fn Name(...)` but
    forgets `#[no_mangle]`. Without it, the linker can't find the symbol.
    """
    for name in exports:
        # Match: pub unsafe extern "..." fn Name( without a preceding #[no_mangle]
        # We look for the pattern and check the line above isn't #[no_mangle]
        pattern = re.compile(
            rf'^([ \t]*)(pub\s+unsafe\s+extern\s+"[^"]+"\s+fn\s+{re.escape(name)}\s*\()',
            re.MULTILINE,
        )
        for m in pattern.finditer(code):
            start = m.start()
            # Check if #[no_mangle] is already on the preceding line(s)
            preceding = code[max(0, start - 80) : start]
            if "#[no_mangle]" in preceding.split("\n")[-2:]:
                continue
            # Also check if there's a #[cfg(...)] + #[no_mangle] block
            lines_before = preceding.rstrip().rsplit("\n", 3)
            if any("#[no_mangle]" in lb for lb in lines_before):
                continue
            # Insert #[no_mangle] before the function
            indent = m.group(1)
            code = code[:start] + f"{indent}#[no_mangle]\n" + code[start:]
            break  # re-find after mutation (only one match per name expected)

    return code


def fix_global_exports(code: str, globals: list[CGlobal]) -> str:
    """Add #[no_mangle] to globally-visible static mut declarations.

    C globals defined in the translated file need #[no_mangle] so other
    C/Rust compilation units can reference them by name.
    """
    for g in globals:
        name = g.name
        pattern = re.compile(
            rf"^([ \t]*)(pub\s+static\s+mut\s+{re.escape(name)}\s*:)",
            re.MULTILINE,
        )
        for m in pattern.finditer(code):
            start = m.start()
            preceding = code[max(0, start - 80) : start]
            if any("#[no_mangle]" in lb for lb in preceding.split("\n")[-3:]):
                continue
            indent = m.group(1)
            code = code[:start] + f"{indent}#[no_mangle]\n" + code[start:]
            break
    return code


def fix_fastcall_exports(code: str, exports: dict[str, SpecEntry]) -> str:
    """Duplicate fastcall exports into dual cfg blocks for i386 vs others.

    Functions with `@ fastcall` in ntoskrnl.spec need different calling
    conventions on i386 (extern "fastcall") vs amd64/arm64 (extern "C").
    The LLM generates a single extern "system" version.
    """
    fastcall_names = [name for name, entry in exports.items() if entry.convention == "fastcall"]
    if not fastcall_names:
        return code

    for name in fastcall_names:
        # Find the function definition — may have #[no_mangle] above it
        # Match pattern: optional #[no_mangle]\n pub unsafe extern "..." fn Name(...) { body }
        pattern = re.compile(
            rf"^([ \t]*#\[no_mangle\]\n)?"
            rf'([ \t]*)(pub\s+unsafe\s+extern\s+"[^"]+"\s+fn\s+{re.escape(name)}\s*\([^)]*\))',
            re.MULTILINE,
        )
        m = pattern.search(code)
        if not m:
            continue

        # Find the complete function body by matching braces
        fn_start = m.start()
        # Find the opening brace
        brace_pos = code.index("{", m.end())
        depth = 1
        pos = brace_pos + 1
        while depth > 0 and pos < len(code):
            if code[pos] == "{":
                depth += 1
            elif code[pos] == "}":
                depth -= 1
            pos += 1
        fn_end = pos

        indent = m.group(2)
        # Extract just the params and return type
        # Re-parse the signature to get params
        sig_match = re.search(
            rf"fn\s+{re.escape(name)}\s*(\([^)]*\)(?:\s*->\s*\S+)?)",
            code[m.start() : brace_pos],
        )
        if not sig_match:
            continue
        params_and_ret = sig_match.group(1)
        body = code[brace_pos:fn_end]

        # Build the dual-cfg version
        x86_block = (
            f'{indent}#[cfg(target_arch = "x86")]\n'
            f"{indent}#[no_mangle]\n"
            f'{indent}pub unsafe extern "fastcall" fn {name}{params_and_ret} {body}'
        )
        other_block = (
            f'\n\n{indent}#[cfg(not(target_arch = "x86"))]\n'
            f"{indent}#[no_mangle]\n"
            f'{indent}pub unsafe extern "C" fn {name}{params_and_ret} {body}'
        )

        code = code[:fn_start] + x86_block + other_block + code[fn_end:]

    return code


def fix_win32k_crate_path(code: str) -> str:
    """Rewrite `ntkrnl_sys::win32k` → `win32k_sys::win32k`.

    The win32k-internal FFI surface lives in the `win32k_sys` crate;
    `ntkrnl_sys` has no `win32k` module (its bindgen covers ntoskrnl.h
    only). The prompt now renders the correct crate path, but the LLM
    may still echo `ntkrnl_sys::win32k` from habit. This rewrite is a
    context-free, always-correct safety net: `ntkrnl_sys::win32k` does
    not exist anywhere, so substituting it can never be wrong.
    """
    return re.sub(r"\bntkrnl_sys::win32k\b", "win32k_sys::win32k", code)


# ntkrnl_sys submodules whose glob `use` clashes with `win32k_sys::win32k`.
# win32k-sys' bindgen comes from pch.h → ntifs.h/ndk, so it is a SUPERSET:
# it already defines ULONG/PVOID/EPROCESS/NTSTATUS/… plus every
# win32k-internal type. Globbing both makes every primitive `ambiguous`
# (E0659) and silently resolves shared structs (e.g. _BASEOBJECT) to the
# kernel variant that lacks win32k fields. Drop the clashing ntkrnl_sys
# globs; keep `win32k_sys::win32k::*` as the single primary namespace and
# any *named* `use ntkrnl_sys::<m>::{A, B}` (kernel APIs not in win32k
# headers) untouched.
_NTKRNL_CLASH_MODS = (
    "base",
    "globals",
    "constants",
    "ddk",
    "ke",
    "mm",
    "ex",
    "ob",
    "ps",
    "se",
    "io",
    "rtl",
    "interlocked",
    "kd",
    "po",
    "cc",
    "fsrtl",
)
_RE_NTKRNL_GLOB = re.compile(
    r"^[ \t]*use[ \t]+ntkrnl_sys::(?:" + "|".join(_NTKRNL_CLASH_MODS) + r")::\*[ \t]*;[ \t]*\n",
    re.MULTILINE,
)


def normalize_win32k_imports(code: str) -> str:
    """Drop ntkrnl_sys glob imports that clash with win32k_sys::win32k.

    Deterministic and win32k-only: for a win32k TU, `win32k_sys::win32k`
    is the authoritative superset namespace. Removing the duplicate
    ntkrnl_sys globs eliminates the whole E0659-ambiguity error class and
    fixes wrong-crate struct resolution — the project's worst per-file
    win32k tax. Named ntkrnl_sys imports (`use ntkrnl_sys::ddk::{...}`)
    are preserved (the `::*` anchor in the regex won't match them).
    """
    code = _RE_NTKRNL_GLOB.sub("", code)
    if "use win32k_sys::win32k::*;" not in code:
        code = "use win32k_sys::win32k::*;\n" + code
    return code


def enforce_win32k_abi(code: str) -> str:
    """Overwrite the LLM's `extern "..."` on win32k exports with ground truth.

    Calling convention is deterministic (win32k-sys bindgen_i386 +
    win32k.spec) and the project's #1 i386 crash cause — so it is taken
    out of the LLM's hands entirely. For every `#[no_mangle]` exported
    fn whose authoritative i386 convention is stdcall/fastcall, rewrite
    it into the canonical dual-cfg block (x86 = the true convention,
    non-x86 = cdecl). Whatever ABI the LLM emitted is discarded.

    Generalization of `fix_fastcall_exports`: not name-list-from-spec but
    map-driven (stdcall *and* fastcall), keyed off the authoritative
    win32k ABI map. cdecl names are left as-is (the LLM default is
    already cdecl-compatible).

    Authoritative, not advisory: the LLM frequently emits its own
    (mangled) `#[cfg(target_arch = "x86")]` scaffolding — e.g. a dangling
    cfg with no item, or a single body under the wrong cfg. This pass
    therefore (a) finds *every* definition of the target fn (balanced
    paren/brace parsing, so fn-pointer params don't break it), (b)
    absorbs any preceding `#[cfg(...)]` / `#[no_mangle]` / blank
    scaffolding lines, (c) collapses them all to one canonical dual-cfg
    block. Idempotent only on its *own exact* prior output (both the x86
    `extern "<conv>"` and non-x86 `extern "C"` blocks already present).
    """
    abi = build_win32k_abi_map()

    # Candidate exported fn names defined in this file (any extern).
    name_re = re.compile(
        r'#\[no_mangle\]\s*\n\s*pub\s+unsafe\s+extern\s+"[^"]+"\s+fn\s+(\w+)\s*\(',
    )
    names = []
    seen = set()
    for nm in name_re.finditer(code):
        n = nm.group(1)
        if n in seen:
            continue
        seen.add(n)
        conv = abi.get(n)
        if conv in ("stdcall", "fastcall"):
            names.append((n, conv))

    def _match(s: str, open_idx: int, opc: str, clc: str) -> int:
        """Index just past the balancer matching s[open_idx]==opc."""
        depth = 0
        i = open_idx
        while i < len(s):
            c = s[i]
            if c == opc:
                depth += 1
            elif c == clc:
                depth -= 1
                if depth == 0:
                    return i + 1
            i += 1
        return -1

    attr_line = re.compile(r"^[ \t]*(#\[[^\n]*\])?[ \t]*$")

    for name, conv in names:
        # Idempotency: already in *our* canonical dual-cfg form? skip.
        canon_x86 = re.search(
            rf'#\[cfg\(target_arch = "x86"\)\]\s*\n\s*#\[no_mangle\]\s*\n\s*'
            rf'pub unsafe extern "{re.escape(conv)}" fn {re.escape(name)}\b',
            code,
        )
        canon_other = re.search(
            rf'#\[cfg\(not\(target_arch = "x86"\)\)\]\s*\n\s*#\[no_mangle\]\s*\n\s*'
            rf'pub unsafe extern "C" fn {re.escape(name)}\b',
            code,
        )
        if canon_x86 and canon_other:
            continue

        head_re = re.compile(
            rf'pub\s+unsafe\s+extern\s+"[^"]+"\s+fn\s+{re.escape(name)}\s*\(',
        )
        spans = []  # (span_start, body_end)
        canonical = None  # (params, ret_suffix, body, indent)
        for hm in head_re.finditer(code):
            paren_open = code.index("(", hm.end() - 1)
            paren_end = _match(code, paren_open, "(", ")")
            if paren_end < 0:
                continue
            try:
                brace_open = code.index("{", paren_end)
            except ValueError:
                continue
            body_end = _match(code, brace_open, "{", "}")
            if body_end < 0:
                continue
            params = code[paren_open:paren_end]
            ret = code[paren_end:brace_open].strip()
            ret_suffix = f" {ret}" if ret else ""
            body = code[brace_open:body_end]

            line_start = code.rfind("\n", 0, hm.start()) + 1
            indent = code[line_start : hm.start()]
            # Absorb contiguous preceding attribute / blank lines (the
            # LLM's mangled cfg scaffolding).
            span_start = line_start
            while span_start > 0:
                prev_nl = span_start - 1  # the '\n'
                prev_start = code.rfind("\n", 0, prev_nl) + 1
                prev_line = code[prev_start:prev_nl]
                if attr_line.match(prev_line):
                    span_start = prev_start
                else:
                    break

            spans.append((span_start, body_end))
            if canonical is None:
                canonical = (params, ret_suffix, body, indent)

        if not spans or canonical is None:
            continue

        params, ret_suffix, body, indent = canonical
        block = (
            f'{indent}#[cfg(target_arch = "x86")]\n'
            f"{indent}#[no_mangle]\n"
            f'{indent}pub unsafe extern "{conv}" fn {name}{params}{ret_suffix} {body}\n\n'
            f'{indent}#[cfg(not(target_arch = "x86"))]\n'
            f"{indent}#[no_mangle]\n"
            f'{indent}pub unsafe extern "C" fn {name}{params}{ret_suffix} {body}'
        )
        spans.sort()
        first_start = spans[0][0]
        for s, e in sorted(spans, reverse=True):
            code = code[:s] + code[e:]
        code = code[:first_start] + block + code[first_start:]

    return code


def fix_self_imports(code: str) -> str:
    """Remove self-imports: names imported from ddk:: that are also defined locally.

    The LLM often imports functions from ddk:: that it also defines with
    #[no_mangle] in the same file, or declares as extern, causing E0255/E0428
    (duplicate definitions).
    """
    # Collect locally-defined function names (with or without #[no_mangle])
    local_names = set()
    for m in re.finditer(
        r'#\[no_mangle\]\s*\n\s*pub\s+(?:unsafe\s+)?(?:extern\s+"[^"]+"\s+)?fn\s+(\w+)', code
    ):
        local_names.add(m.group(1))

    # Also catch pub unsafe extern fn without #[no_mangle] (LLM sometimes omits it)
    for m in re.finditer(
        r'^pub\s+(?:unsafe\s+)?(?:extern\s+"[^"]+"\s+)?fn\s+(\w+)', code, re.MULTILINE
    ):
        local_names.add(m.group(1))

    # Also collect #[no_mangle] pub static mut names
    for m in re.finditer(r"#\[no_mangle\]\s*\n\s*pub\s+static\s+mut\s+(\w+)", code):
        local_names.add(m.group(1))

    # Also collect names declared in extern blocks (extern "C" { fn X(); })
    # These also conflict with ddk imports
    for m in re.finditer(r'extern\s+"[^"]+"\s*\{(.*?)\}', code, re.DOTALL):
        block_body = m.group(1)
        for fn_match in re.finditer(r"fn\s+(\w+)\s*\(", block_body):
            local_names.add(fn_match.group(1))
        for static_match in re.finditer(r"static\s+(?:mut\s+)?(\w+)\s*:", block_body):
            local_names.add(static_match.group(1))

    if not local_names:
        return code

    # Find and clean up ddk import blocks
    def clean_import_block(match: re.Match) -> str:
        full = match.group(0)
        prefix = match.group(1)  # "use ntkrnl_sys::ddk::{"
        body = match.group(2)  # the imports between { }
        suffix = match.group(3)  # "};"

        # Parse individual imports
        items = [item.strip() for item in body.split(",")]
        items = [item for item in items if item]  # remove empty
        # Filter out locally-defined names
        filtered = [item for item in items if item not in local_names]

        # If nothing was removed, return original unchanged
        if len(filtered) == len(items):
            return full

        if not filtered:
            return "// (all imports from ddk moved to local definitions)"

        new_body = ", ".join(filtered)
        # Re-wrap if multi-line original
        if "\n" in body:
            new_body = "\n    " + ",\n    ".join(filtered) + ",\n"

        return f"{prefix}{new_body}{suffix}"

    # Match use ntkrnl_sys::ddk::{...};  (potentially multi-line)
    code = re.sub(
        r"(use ntkrnl_sys::ddk::\{)(.*?)(\};)",
        clean_import_block,
        code,
        flags=re.DOTALL,
    )

    return code


# ============================================================================
# Callback calling convention fixes
# ============================================================================


def _split_extern_decls(inner_lines: list[str]) -> list[tuple[str, str]]:
    """Split the inner lines of an `extern "..." { ... }` block into a list of
    (kind, text) entries, where kind is 'comment' or 'decl'.

    Standalone `//` line comments (i.e. comments that sit between declarations,
    not interleaved inside a multi-line declaration) are preserved as their own
    'comment' entries so callers can pass them through dedup unchanged.
    Multi-line declarations split across newlines for formatting are joined
    back into a single `decl` entry, terminated by `;`.

    Note: comments interleaved inside a multi-line declaration (between the
    opening `(` and the trailing `;`) are still folded into the joined text —
    that case is rare in extern blocks and not the target of this helper.
    """
    out: list[tuple[str, str]] = []
    current = ""
    for line in inner_lines:
        s = line.strip()
        if not s:
            continue
        # A `//` line that appears between declarations (i.e. not while we are
        # mid-way through accumulating a declaration) is a standalone comment.
        if s.startswith("//") and not current:
            out.append(("comment", s))
            continue
        current = (current + " " + s).strip() if current else s
        if current.endswith(";"):
            out.append(("decl", current))
            current = ""
    if current:
        out.append(("decl", current))
    return out


def fix_extern_vs_local_static_dups(code: str) -> str:
    """Remove extern `static` decls for globals also DEFINED locally.

    Since cmd_translate added the preamble step, file-scope globals get a
    real local definition (`pub static mut gpentHmgr: PENTRY = ...;`),
    but the per-function chunks still extern-declare the same globals
    (`extern "C" { static mut gpentHmgr: PENTRY; }`) → E0428 on every
    multi-function file with shared TU globals. Drop the extern decl in
    favour of the definition (the static analog of
    fix_extern_vs_local_dups).
    """
    # Locally-defined static names (have an `=` initializer, top-level).
    local_static_names = set()
    for m in re.finditer(
        r"^(?:[ \t]*#\[no_mangle\][ \t]*\n)?[ \t]*pub\s+static\s+(?:mut\s+)?"
        r"(\w+)\s*:[^;=]+=",
        code,
        re.MULTILINE,
    ):
        local_static_names.add(m.group(1))

    if not local_static_names:
        return code

    def clean_block(match: re.Match) -> str:
        block = match.group(0)
        header_m = re.match(r'(extern\s+"[^"]+"\s*\{)', block)
        inner = re.search(r"\{(.*)\}", block, re.DOTALL)
        if not header_m or not inner:
            return block
        entries = _split_extern_decls(inner.group(1).splitlines())
        filtered: list[tuple[str, str]] = []
        for kind, text in entries:
            if kind == "decl":
                sm = re.match(r"static\s+(?:mut\s+)?(\w+)\s*:", text)
                if sm and sm.group(1) in local_static_names:
                    continue  # defined locally — drop the extern decl
            filtered.append((kind, text))
        if not any(k == "decl" for k, _ in filtered):
            return ""
        if len(filtered) == len(entries):
            return block
        return header_m.group(1) + "\n" + "".join(f"    {t}\n" for _, t in filtered) + "}"

    return re.sub(r'extern\s+"[^"]+"\s*\{[^}]*\}', clean_block, code, flags=re.DOTALL)


def fix_extern_vs_local_dups(code: str) -> str:
    """Remove extern block declarations for functions also defined locally.

    When a function is both declared in an extern block AND defined with a body
    in the same file, the extern declaration must be removed (E0428).
    """
    # Collect locally-defined function names (with body)
    # Must be at top-level (column 0 or after #[no_mangle]) — not inside extern blocks
    local_fn_names = set()

    # First, identify ranges of extern blocks so we can skip them
    extern_ranges = []
    for m in re.finditer(r'extern\s+"[^"]+"\s*\{', code):
        start = m.start()
        brace_depth = 1
        pos = m.end()
        while pos < len(code) and brace_depth > 0:
            if code[pos] == "{":
                brace_depth += 1
            elif code[pos] == "}":
                brace_depth -= 1
            pos += 1
        extern_ranges.append((start, pos))

    def in_extern_block(offset: int) -> bool:
        return any(s <= offset < e for s, e in extern_ranges)

    for m in re.finditer(
        r'(?:pub\s+)?(?:unsafe\s+)?(?:extern\s+"[^"]+"\s+)?fn\s+(\w+)\s*\(',
        code,
    ):
        if in_extern_block(m.start()):
            continue  # Skip declarations inside extern blocks

        # Verify it has a body (opening '{' after params)
        name = m.group(1)
        after = code[m.end() :]
        paren_depth = 1
        pos = 0
        while pos < len(after) and paren_depth > 0:
            if after[pos] == "(":
                paren_depth += 1
            elif after[pos] == ")":
                paren_depth -= 1
            pos += 1
        rest = after[pos : pos + 80].strip()
        if rest.startswith("{") or rest.startswith("->"):
            local_fn_names.add(name)

    if not local_fn_names:
        return code

    # Remove matching declarations from extern blocks
    def clean_extern_block(match: re.Match) -> str:
        block = match.group(0)
        header_match = re.match(r'(extern\s+"[^"]+"\s*\{)', block)
        if not header_match:
            return block

        header = header_match.group(1)
        inner = re.search(r"\{(.*)\}", block, re.DOTALL)
        if not inner:
            return block

        decls_text = inner.group(1)
        entries = _split_extern_decls(decls_text.splitlines())

        # Filter out declarations matching local function names; keep comments.
        filtered: list[tuple[str, str]] = []
        for kind, text in entries:
            if kind == "decl":
                fn_m = re.match(r"fn\s+(\w+)\s*\(", text)
                if fn_m and fn_m.group(1) in local_fn_names:
                    continue  # Remove this extern decl — defined locally
            filtered.append((kind, text))

        # Drop the block entirely if nothing but comments would remain.
        if not any(k == "decl" for k, _ in filtered):
            return ""
        if len(filtered) == len(entries):
            return block  # Nothing changed

        result = header + "\n"
        for _, text in filtered:
            result += f"    {text}\n"
        result += "}"
        return result

    code = re.sub(
        r'extern\s+"[^"]+"\s*\{[^}]*\}',
        clean_extern_block,
        code,
        flags=re.DOTALL,
    )
    # Clean up blank lines from removed blocks
    code = re.sub(r"\n{3,}", "\n\n", code)
    return code


def fix_duplicate_fn_defs(code: str) -> str:
    """Remove duplicate function definitions with the same name.

    The stitcher may emit the same function twice (e.g., from two LLM attempts).
    Keep the last definition (which typically has more context from the LLM).

    Only operates on top-level functions that start at column 0 with
    #[no_mangle] + pub unsafe extern pattern. Avoids matching fn inside
    extern blocks or nested scopes.
    """
    # Match only top-level function definitions: #[no_mangle]\npub unsafe extern "..." fn Name(
    fn_pattern = re.compile(
        r"^#\[no_mangle\]\s*\n"
        r'^(?:pub\s+)?(?:unsafe\s+)?(?:extern\s+"[^"]+"\s+)?fn\s+(\w+)\s*\(',
        re.MULTILINE,
    )

    # First pass: find all function names and their start positions.
    # Key by (name, cfg-guard): two definitions under *complementary*
    # cfg guards — `#[cfg(target_arch = "x86")]` vs
    # `#[cfg(not(target_arch = "x86"))]` — are NOT duplicates; they are
    # the mutually-exclusive per-arch builds that `enforce_win32k_abi`
    # and `fix_fastcall_exports` deliberately emit. Deduping them
    # collapses the dual-cfg ABI block and silently reintroduces the
    # #1 i386 crash cause. Only same-(name, guard) pairs are real dups.
    def _cfg_guard_above(pos: int) -> str:
        prev_end = code.rfind("\n", 0, pos)
        if prev_end <= 0:
            return ""
        prev_start = code.rfind("\n", 0, prev_end) + 1
        prev_line = code[prev_start:prev_end].strip()
        return prev_line if prev_line.startswith("#[cfg(") else ""

    fn_positions: dict[tuple[str, str], list[int]] = {}
    for m in fn_pattern.finditer(code):
        key = (m.group(1), _cfg_guard_above(m.start()))
        fn_positions.setdefault(key, []).append(m.start())

    # Find functions defined more than once under the *same* guard.
    dups = {key: positions for key, positions in fn_positions.items() if len(positions) > 1}
    if not dups:
        return code

    # Second pass: remove all but the last occurrence of each dup.
    to_remove = []
    for positions in dups.values():
        for pos in positions[:-1]:
            fn_start = pos
            # Absorb an immediately-preceding `#[cfg(...)]` line so a
            # real-dup removal doesn't leave a dangling cfg attribute.
            prev_end = code.rfind("\n", 0, pos)
            if prev_end > 0:
                prev_start = code.rfind("\n", 0, prev_end) + 1
                if code[prev_start:prev_end].strip().startswith("#[cfg("):
                    fn_start = prev_start

            # Find the opening brace after the fn signature
            try:
                brace_start = code.index("{", pos)
            except ValueError:
                continue
            depth = 1
            end = brace_start + 1
            while depth > 0 and end < len(code):
                if code[end] == "{":
                    depth += 1
                elif code[end] == "}":
                    depth -= 1
                end += 1
            to_remove.append((fn_start, end))

    # Sort by position (reverse) and remove
    to_remove.sort(reverse=True)
    for start, end in to_remove:
        code = code[:start] + code[end:]

    code = re.sub(r"\n{3,}", "\n\n", code)
    return code


def fix_callback_conventions(code: str) -> str:
    """Fix callback function calling conventions for cross-platform compatibility."""
    return code


def fix_extern_const(code: str) -> str:
    """Remove `const` declarations from extern blocks (not valid in Rust).

    The LLM sometimes puts `const NAME: type;` inside extern blocks.
    These should be standalone `const` declarations outside the block.
    """
    lines = code.split("\n")
    result = []
    in_extern = 0

    for line in lines:
        stripped = line.strip()
        if re.match(r'extern\s+"[^"]*"\s*\{', stripped):
            in_extern += stripped.count("{") - stripped.count("}")
            result.append(line)
        elif in_extern > 0:
            in_extern += stripped.count("{") - stripped.count("}")
            # Check if this is a const declaration
            if stripped.startswith("const ") and stripped.endswith(";"):
                # Extract it — will be placed outside the extern block
                # But we can't give it a value here, so just remove it
                # (these are typically C #defines that need manual handling)
                continue  # Skip it entirely
            else:
                result.append(line)
        else:
            result.append(line)

    return "\n".join(result)


def fix_variadic_extern_system(code: str) -> str:
    """Change variadic extern "system" declarations to extern "C".

    C-variadic functions with "system" calling convention are unstable in Rust.
    Variadic functions must use extern "C".
    """
    # Find extern "system" blocks containing variadic declarations (...)
    lines = code.split("\n")
    result = []
    i = 0
    while i < len(lines):
        line = lines[i]
        stripped = line.strip()

        # Detect extern "system" block start
        if re.match(r'extern\s+"system"\s*\{', stripped):
            # Collect the entire block
            block_lines = [line]
            brace_count = stripped.count("{") - stripped.count("}")
            i += 1
            while i < len(lines) and brace_count > 0:
                block_lines.append(lines[i])
                brace_count += lines[i].count("{") - lines[i].count("}")
                i += 1

            block_text = "\n".join(block_lines)
            # If block contains variadic fn (...), change to extern "C"
            if "..." in block_text:
                block_text = re.sub(
                    r'extern\s+"system"\s*\{',
                    'extern "C" {',
                    block_text,
                    count=1,
                )
            result.append(block_text)
        else:
            result.append(line)
            i += 1

    return "\n".join(result)


_GDIOBJ_MACRO_FAMILIES = (
    "DRIVEROBJ",
    "BRUSHOBJ",
    "DC",
    "PATH",
    "REGION",
    "SURFACE",
    "PALETTE",
    "XLATEOBJ",
    "LFONT",
    "TEXTOBJ",
    "COLORSPACEOBJ",
    "ICMLCS",
)

_GDIOBJ_MACRO_VERBS = (
    # exact suffix patterns the headers use
    "AllocObject",
    "AllocObjectWithHandle",
    "AllocPath",
    "AllocPathWithHandle",
    "AllocCS",
    "AllocCSWithHandle",
    "AllocFont",
    "AllocFontWithHandle",
    "AllocBrush",
    "AllocBrushWithHandle",
    "AllocPalette",
    "AllocPaletteWithHandle",
    "AllocSurface",
    "AllocSurfaceWithHandle",
    "AllocRgn",
    "AllocRgnWithHandle",
    "AllocText",
    "AllocTextWithHandle",
    "LockObject",
    "LockBrush",
    "LockDc",
    "LockPath",
    "LockRgn",
    "LockSurface",
    "LockPalette",
    "LockFont",
    "LockText",
    "LockCS",
    "UnlockObject",
    "UnlockBrush",
    "UnlockDc",
    "UnlockPath",
    "UnlockRgn",
    "UnlockSurface",
    "UnlockPalette",
    "UnlockFont",
    "UnlockText",
    "UnlockCS",
    "TryLockObject",
    "ShareLockObject",
    "ShareLockSurface",
    "ShareLockPalette",
    "ShareLockFont",
    "ShareUnlockObject",
    "ShareUnlockSurface",
    "ShareUnlockPalette",
    "ShareUnlockFont",
    "vUnlockObject",
    "vReleaseObj",
)


def drop_gdiobj_macro_externs(code: str) -> str:
    """Mode-4 safety net: drop `extern "C" { fn <T>_<Verb>(...); }` decls
    where <T>_<Verb> is a known GDI-object macro family pattern.

    These names look like C functions but are macros (gdi/eng/<T>.h,
    gdi/ntgdi/<T>.h) — declaring them extern produces undefined symbols
    at link. Dropping the extern downgrades the failure to a clean
    compile error at the call site, much easier to fix. The Rust
    translation should inline-expand to the underlying GDIOBJ_*
    primitives (see win32k_translation_prompt.md §6).
    """
    pat = re.compile(
        r'^extern\s+"(?:C|system|stdcall|fastcall)"\s*\{\s*\n'
        r"((?:[ \t]+(?:#\[[^\]]+\]\s*\n[ \t]+)?fn\s+[A-Za-z_]\w*\b"
        r"(?:\s*\([^)]*\))?\s*(?:->\s*[^;{]+)?\s*;\s*\n)+)"
        r"\}\s*\n?",
        re.MULTILINE,
    )

    def rewrite_block(m: re.Match) -> str:
        body = m.group(1)
        kept_fns: list[str] = []
        dropped: list[str] = []
        for line in body.splitlines(keepends=False):
            fn_m = re.match(r"^\s*fn\s+([A-Za-z_]\w*)\b", line)
            if not fn_m:
                kept_fns.append(line)
                continue
            name = fn_m.group(1)
            prefix, _, rest = name.partition("_")
            if prefix in _GDIOBJ_MACRO_FAMILIES and rest in _GDIOBJ_MACRO_VERBS:
                dropped.append(name)
                continue
            kept_fns.append(line)
        if not dropped:
            return m.group(0)
        out = ""
        if any(ln.strip() for ln in kept_fns):
            out += 'extern "C" {\n' + "\n".join(kept_fns) + "\n}\n"
        out += (
            f"// drop_gdiobj_macro_externs: dropped {len(dropped)} bogus "
            f"extern decl(s) for GDI-object macro(s): "
            f"{', '.join(dropped)} — these are macros in "
            f"gdi/eng/<type>.h or gdi/ntgdi/<type>.h, not real fns. "
            f"Inline-expand via GDIOBJ_* primitives per "
            f"win32k_translation_prompt.md §6.\n"
        )
        return out

    return pat.sub(rewrite_block, code)


_PSEH_MACRO_EXTERN_NAMES_RE = re.compile(r"^(?:_SEH2_\w+|RtlGetProcessHeap)$")


def drop_pseh_macro_externs(code: str) -> str:
    """User-mode safety net: drop `extern "…" { fn _SEH2_…(); … }` decls.

    Under `USE_DUMMY_PSEH` the PSEH handlers are dead code and the system
    prompt tells the LLM to translate the protected body directly and drop
    the `_SEH2_EXCEPT` handler — but rustconv still occasionally emits an
    `extern "system" { fn _SEH2_TRY(); fn _SEH2_GetExceptionCode(); … }`
    block. Those are PSEH *macros*, not symbols: declaring them extern is a
    guaranteed link failure if any survives a reference, and dead noise
    otherwise. `strip_user_inline_redeclarations` only nukes a block when
    EVERY fn is a known PEB/TEB inline, so a mixed block (SEH2 + a real
    import like WlStrDup) is left intact and the SEH2 lines leak through.

    This strips just the `_SEH2_*` (and the `RtlGetProcessHeap` macro)
    decls line-by-line, keeps every real fn under the original ABI, and
    removes the block entirely if nothing real remains. Mirrors
    drop_gdiobj_macro_externs (win32k) for the user-mode substrate.
    """
    pat = re.compile(
        r'extern\s+"([^"]+)"\s*\{\s*\n'
        r"((?:[ \t]*(?:#\[[^\]]+\]\s*\n[ \t]*)?(?:pub\s+)?fn\s+[A-Za-z_]\w*\b"
        r"(?:\s*\([^)]*\))?\s*(?:->\s*[^;{]+)?\s*;\s*\n)+)"
        r"[ \t]*\}\s*\n?",
        re.MULTILINE,
    )

    def rewrite_block(m: re.Match) -> str:
        abi = m.group(1)
        body = m.group(2)
        kept: list[str] = []
        dropped: list[str] = []
        for line in body.splitlines(keepends=False):
            fn_m = re.match(r"^\s*(?:pub\s+)?fn\s+([A-Za-z_]\w*)\b", line)
            if fn_m and _PSEH_MACRO_EXTERN_NAMES_RE.match(fn_m.group(1)):
                dropped.append(fn_m.group(1))
                continue
            kept.append(line)
        if not dropped:
            return m.group(0)
        out = ""
        if any(ln.strip() for ln in kept):
            out += f'extern "{abi}" {{\n' + "\n".join(kept) + "\n}\n"
        out += (
            f"// drop_pseh_macro_externs: dropped {len(dropped)} bogus "
            f"extern decl(s): {', '.join(dropped)} — PSEH _SEH2_* "
            f"(and RtlGetProcessHeap) are macros, not symbols. Under "
            f"USE_DUMMY_PSEH the SEH path is dead; translate the body "
            f"directly. RtlGetProcessHeap → GetProcessHeap().\n"
        )
        return out

    return pat.sub(rewrite_block, code)


def fix_markdown_fence_leak(code: str) -> str:
    """Strip stray markdown ``` fences the LLM occasionally leaks into
    the preamble translation output (empty preambles → bare ```rust\\n```).

    Bare fence lines are never valid Rust — drop them. Only strips
    fence-marker lines (open + close); preserves any code between them.
    """
    out = []
    for line in code.splitlines(keepends=False):
        stripped = line.strip()
        # match `\`\`\`rust`, `\`\`\``, `\`\`\`c`, etc. — language tag is `\w*`
        if re.fullmatch(r"```\w*", stripped):
            continue
        out.append(line)
    # preserve trailing newline shape
    suffix = "\n" if code.endswith("\n") else ""
    return "\n".join(out) + suffix


def fix_duplicate_extern_decls(code: str) -> str:
    """Deduplicate extern block declarations within a single file.

    When the stitcher merges per-function outputs, different functions may
    declare the same extern with different parameter names or linkage.
    This causes E0428 (duplicate definitions) errors.
    """
    lines = code.split("\n")
    result = []
    i = 0
    # Track all extern declarations across all blocks
    seen_extern_fns: dict[str, int] = {}  # name → first-seen line
    seen_extern_statics: dict[str, int] = {}

    while i < len(lines):
        line = lines[i]
        stripped = line.strip()

        # Detect extern block start
        if re.match(r'extern\s+"[^"]*"\s*\{', stripped):
            block_lines = [line]
            brace_count = stripped.count("{") - stripped.count("}")
            i += 1
            while i < len(lines) and brace_count > 0:
                block_lines.append(lines[i])
                brace_count += lines[i].count("{") - lines[i].count("}")
                i += 1

            # Parse declarations within the block, deduplicate
            header = block_lines[0]
            inner_lines = block_lines[1:-1] if len(block_lines) > 2 else []

            entries = _split_extern_decls(inner_lines)

            # Filter duplicates by symbol name; pass comments through unchanged.
            deduped: list[tuple[str, str]] = []
            for kind, text in entries:
                if kind == "decl":
                    fn_m = re.match(r"fn\s+(\w+)\s*\(", text)
                    static_m = re.match(r"static\s+(?:mut\s+)?(\w+)\s*:", text)
                    if fn_m:
                        name = fn_m.group(1)
                        if name in seen_extern_fns:
                            continue  # skip duplicate
                        seen_extern_fns[name] = len(result)
                    elif static_m:
                        name = static_m.group(1)
                        if name in seen_extern_statics:
                            continue
                        seen_extern_statics[name] = len(result)
                deduped.append((kind, text))

            # Emit only if there is at least one real declaration left;
            # an extern block of just comments would be useless and odd.
            if any(k == "decl" for k, _ in deduped):
                result.append(header)
                for _, text in deduped:
                    result.append(f"    {text}")
                result.append("}")
            # else: skip the block entirely
        else:
            result.append(line)
            i += 1

    return "\n".join(result)


# ============================================================================
# Main entry point
# ============================================================================

# User-mode FFI substrate crates (each re-exports its bindgen under an
# umbrella module). The LLM sometimes imports the wrong sibling; postprocess
# retargets it onto the configured one (see collapse_sys_crate).
_KNOWN_USER_SYS_CRATES = {
    "ntdll_sys",
    "win32k_sys",
    "winsrv_sys",
    "winlogon_sys",
    "userinit_sys",
    "csrsrv_sys",
    "smss_sys",
    "basesrv_sys",
}


def collapse_sys_crate(code: str, sys_crate: str, sys_umbrella: str) -> str:
    """Deterministically retarget a user-mode translation's FFI imports.

    rustconv's prompt/catalog are kernel-oriented, so the LLM emits
    `ntkrnl_sys::{ddk,base,constants,globals,macros}::Sym`. A user-mode
    target's substrate (ntdll-sys) re-exports its whole bindgen under
    one umbrella module (`ntdll_sys::nt`). This collapses every
    `ntkrnl_sys::<mod>::` to `<sys_crate>::<sys_umbrella>::` and
    replaces the now-redundant explicit `use` lines with a single glob
    — LLM-independent, mirrors enforce_win32k_abi philosophy.
    """
    pfx = f"{sys_crate}::{sys_umbrella}::"
    # 1. Any ntkrnl_sys::<module>:: (incl. nested) → <crate>::<umbrella>::
    code = re.sub(r"\bntkrnl_sys::\w+::", pfx, code)
    # 1b. Sibling user-mode substrate crates the LLM may have guessed wrong
    #     (the user-mode prompt names ntdll-sys, so it often imports
    #     `ntdll_sys::nt::*` even for a winlogon/winsrv/etc. target). Retarget
    #     any known *-sys crate that isn't THIS target's onto the right one,
    #     collapsing its module to the umbrella exactly like ntkrnl_sys above.
    for other in _KNOWN_USER_SYS_CRATES:
        if other == sys_crate:
            continue
        code = re.sub(rf"\b{re.escape(other)}::\w+::", pfx, code)
    # 2. Drop every `use <crate>::<umbrella>::(* | {…} | Name);` line
    #    (single- or multi-line braced list) — replaced by one glob.
    use_pat = re.compile(
        rf"^[ \t]*use\s+{re.escape(sys_crate)}::{re.escape(sys_umbrella)}::"
        r"(?:\*|\{[^}]*\}|\w+)\s*;[ \t]*\n",
        re.MULTILINE | re.DOTALL,
    )
    code = use_pat.sub("", code)
    # 3. Prepend a single glob use after the leading //! doc / comments.
    lines = code.split("\n")
    insert_at = 0
    for i, ln in enumerate(lines):
        s = ln.strip()
        if s.startswith("//!") or s.startswith("//") or s == "":
            insert_at = i + 1
        else:
            break
    glob = f"use {sys_crate}::{sys_umbrella}::*;"
    if glob not in code:
        lines.insert(insert_at, glob)
    return "\n".join(lines)


_USER_INLINE_NAMES = {
    "NtCurrentPeb",
    "NtCurrentTeb",
    "NtCurrentProcess",
    "NtCurrentThread",
    "ZwCurrentProcess",
    "ZwCurrentThread",
}


def strip_user_inline_redeclarations(code: str) -> str:
    """Deterministic net for user-mode targets: drop LLM re-declarations
    of substrate-provided FORCEINLINE accessors + bogus local PEB/TEB.

    Even with the user-mode prompt addendum the LLM sometimes still
    emits `extern "system" { fn NtCurrentPeb …; }` (a guaranteed link
    failure — those are not ntdll exports) and a hand-rolled
    `struct PEB { _padding: [u8; 0x…], SessionId … }` (arch-wrong
    guessed offset). The real ones come from `ntdll_sys::nt::*`. This
    is target-scoped (only called when sys_umbrella is set).
    """

    # 1. Remove extern blocks whose every `fn NAME` is a known inline.
    def drop_extern(m: re.Match) -> str:
        body = m.group(2)
        fns = re.findall(r"\bfn\s+(\w+)", body)
        if fns and all(f in _USER_INLINE_NAMES for f in fns):
            return ""
        return m.group(0)

    code = re.sub(
        r'(extern\s+"[^"]+"\s*\{)(.*?)(\})',
        drop_extern,
        code,
        flags=re.DOTALL,
    )
    # 2. Remove a bogus local `struct PEB|TEB { … _padding … }` (with
    #    or without a #[repr(C)] / #[derive(...)] attr line above it).
    code = re.sub(
        r"(?:#\[[^\]]*\]\s*)*"
        r"(?:pub\s+)?struct\s+(?:PEB|TEB)\s*\{[^}]*?_padding[^}]*?\}\s*",
        "",
        code,
        flags=re.DOTALL,
    )
    # 3. Retarget casts that referenced those local structs onto the
    #    genuine bindgen layouts (in scope via the nt glob).
    code = re.sub(r"\bas\s+(\*\s*(?:const|mut)\s*)PEB\b", r"as \1_PEB", code)
    code = re.sub(r"\bas\s+(\*\s*(?:const|mut)\s*)TEB\b", r"as \1_TEB", code)
    return code


def postprocess(
    code: str,
    exports: dict[str, SpecEntry] | None = None,
    globals: list[CGlobal] | None = None,
    target_name: str = "ntoskrnl",
    sys_crate: str = "ntkrnl_sys",
    sys_umbrella: str | None = None,
    c_source_path: Path | None = None,
) -> str:
    """Apply all post-processing transforms to LLM-generated Rust code.

    Args:
        code: Raw LLM-generated Rust code
        exports: Spec-exported functions (from resolver), enables #[no_mangle]
                 and fastcall fixes
        globals: C globals from scanner, enables global #[no_mangle] fixes

    Only does SAFE transforms that don't depend on struct type context:
    - Type name underscore-prefixing (always correct)
    - Import fixes (ddk::* → selective, missing c_void)
    - Enum variant name prefixing (always correct)
    - Portable type width (as u64 → as ULONG_PTR for OBJ_* constants)
    - Bindgen anon field access (LARGE_INTEGER .LowPart/.HighPart,
      IO_STATUS_BLOCK .Status — universal, struct-type independent)
    - Enum literal replacements (KernelMode→0i32, UserMode→1i32, etc.)
    - #[no_mangle] for spec exports and globals (when metadata provided)
    - Fastcall dual-cfg for i386 (when metadata provided)
    - Self-import removal (always safe)

    NOTE: General union field access fixes (__bindgen_anon_N) are still
    context-dependent and handled by the system prompt.  Only the
    universally-safe cases above are done here.
    """
    code = fix_imports(code, target_name=target_name)
    code = fix_bogus_ddk_imports(code)
    code = fix_debug_macros(code)
    code = fix_type_names(code)
    code = fix_enum_variants(code)
    code = fix_portable_types(code)
    code = fix_bindgen_anon_field_access(code)
    code = fix_enum_literal_replacements(code)
    if exports:
        code = fix_exports(code, exports)
        code = fix_fastcall_exports(code, exports)
    # win32k: calling convention is deterministic ground truth — overwrite
    # the LLM's extern "..." with the win32k-sys/spec map (the project's
    # #1 i386 crash cause, taken out of the LLM's hands entirely).
    if target_name == "win32k":
        code = fix_win32k_crate_path(code)
        code = normalize_win32k_imports(code)
        code = enforce_win32k_abi(code)
        code = drop_gdiobj_macro_externs(code)
    if globals:
        code = fix_global_exports(code, globals)
    # Target-agnostic ABI / #[no_mangle] enforcement from the C source.
    # Calling convention is the #1 i386 crash cause and dropped-#[no_mangle]
    # is a top rustconv defect; the C file is ground truth for both. Derives
    # name -> (convention, is_static) and rewrites each fn the C defines to
    # its canonical extern/visibility. See abi_enforce.py. (win32k keeps its
    # own bindgen/spec-fused enforcer above; this covers every other target.)
    if c_source_path is not None and target_name != "win32k":
        try:
            code = enforce_abi_from_c(code, c_source_path)
        except Exception as e:
            print(f"  [abi_enforce] skipped ({e})", file=sys.stderr)
    code = fix_self_imports(code)
    code = fix_extern_vs_local_dups(code)
    code = fix_extern_vs_local_static_dups(code)
    code = fix_duplicate_fn_defs(code)
    code = fix_extern_const(code)
    code = fix_variadic_extern_system(code)
    code = fix_duplicate_extern_decls(code)
    code = fix_markdown_fence_leak(code)
    # User-mode target: collapse the kernel-crate imports the kernel-
    # oriented prompt produced onto the user-mode substrate umbrella.
    # Last (after all import fixes have settled).
    if sys_umbrella:
        code = collapse_sys_crate(code, sys_crate, sys_umbrella)
        code = strip_user_inline_redeclarations(code)
        code = drop_pseh_macro_externs(code)
    return code


def postprocess_file(path: Path) -> str:
    """Read a file, post-process it, return the result.

    Auto-detects the target (ntoskrnl/hal/win32k/freeldr) from the path
    so target-gated passes (e.g. `enforce_win32k_abi`,
    `drop_gdiobj_macro_externs`) fire on win32k-rust files.
    """
    code = path.read_text()
    target_name = "ntoskrnl"
    p = path.as_posix()
    if "/win32k-rust/" in p:
        target_name = "win32k"
    elif "/hal-rust/" in p:
        target_name = "hal"
    elif "/freeldr-rust/" in p:
        target_name = "freeldr"
    return postprocess(code, target_name=target_name)
