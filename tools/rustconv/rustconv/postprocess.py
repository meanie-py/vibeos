"""Post-processor for LLM-generated Rust translations.

Applies SAFE mechanical transforms to fix predictable differences between
what the LLM generates and what the ntkrnl-sys bindgen output expects.

IMPORTANT: Does NOT attempt union field access fixes (.__bindgen_anon_N).
Those are context-dependent (different structs have same-named fields in
different positions) and must be handled by the system prompt teaching the
LLM to generate correct accessors directly.
"""

from __future__ import annotations

import re
from pathlib import Path
from typing import TYPE_CHECKING

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
    "POBJECT_HEADER", "POBJECT_TYPE", "POBJECT_CREATE_INFORMATION",
    "POBJECT_HEADER_HANDLE_INFO", "POBJECT_HEADER_QUOTA_INFO",
    "POBJECT_HEADER_NAME_INFO", "POBJECT_HEADER_CREATOR_INFO",
    "POBJECT_HANDLE_COUNT_DATABASE", "POBJECT_HANDLE_COUNT_ENTRY",
    "PHANDLE_TABLE", "PHANDLE_TABLE_ENTRY",
    "PACCESS_STATE", "PAUX_ACCESS_DATA", "PGENERIC_MAPPING",
    "PISECURITY_DESCRIPTOR", "PSECURITY_DESCRIPTOR",
    "PUNICODE_STRING", "PCHAR",
    "PEPROCESS", "PETHREAD", "PVOID", "PCVOID",
}


def fix_type_names(code: str) -> str:
    """Replace bare struct type names with underscore-prefixed versions.

    Only replaces when used as a type (after ':', 'as', in generics, etc.),
    not when used as a pointer type alias (POBJECT_HEADER stays).
    """
    for c_name, bindgen_name in UNDERSCORE_PREFIX_TYPES.items():
        code = re.sub(
            rf'(?<![A-Za-z_])(?<!P){re.escape(c_name)}(?![A-Za-z0-9_])',
            bindgen_name,
            code,
        )
    return code


# ============================================================================
# Import fixes
# ============================================================================

def fix_imports(code: str) -> str:
    """Fix common import issues."""
    # Replace ddk::* with a comment reminder (it conflicts with base::*)
    if "use ntkrnl_sys::ddk::*;" in code:
        code = code.replace(
            "use ntkrnl_sys::ddk::*;",
            "// TODO: Replace ddk::* with selective imports (conflicts with base::*)\n"
            "// use ntkrnl_sys::ddk::{...};"
        )

    # Remove NonPagedPool/PagedPool from ddk imports (they're in mm::*)
    code = code.replace("ntkrnl_sys::ddk::NonPagedPool, ", "")
    code = code.replace("ntkrnl_sys::ddk::PagedPool,\n", "")
    code = code.replace(", ntkrnl_sys::ddk::PagedPool", "")
    code = code.replace("ntkrnl_sys::ddk::NonPagedPool,\n", "")

    # Ensure core::ffi::c_void is imported
    if "c_void" in code and "core::ffi::c_void" not in code:
        code = re.sub(
            r'(use core::ptr::\{[^}]+\};)',
            r'\1\nuse core::ffi::c_void;',
            code,
        )

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
        "ObCreateHandle", "ObOpenHandle", "ObDuplicateHandle", "ObInheritHandle",
    ],
    "OBJECT_INFORMATION_CLASS": [
        "ObjectBasicInformation", "ObjectNameInformation", "ObjectTypeInformation",
        "ObjectTypesInformation", "ObjectHandleFlagInformation",
        "ObjectSessionInformation", "MaxObjectInfoClass",
    ],
    "SECURITY_OPERATION_CODE": [
        "SetSecurityDescriptor", "QuerySecurityDescriptor",
        "DeleteSecurityDescriptor", "AssignSecurityDescriptor",
    ],
    "EVENT_TYPE": [
        "NotificationEvent", "SynchronizationEvent",
    ],
    "KWAIT_REASON": [
        "Executive", "FreePage", "PageIn", "PoolAllocation", "DelayExecution",
        "Suspended", "UserRequest", "WrExecutive", "WrFreePage", "WrPageIn",
        "WrPoolAllocation", "WrDelayExecution", "WrSuspended", "WrUserRequest",
        "WrEventPair", "WrQueue", "WrLpcReceive", "WrLpcReply",
        "WrVirtualMemory", "WrPageOut", "WrRendezvous", "WrKeyedEvent",
        "WrTerminated", "WrProcessInSwap", "WrCpuRateControl",
        "WrCalloutStack", "WrKernel", "WrResource", "WrPushLock",
        "WrMutex", "WrQuantumEnd", "WrDispatchInt", "WrPreempted",
        "WrYieldExecution", "WrFastMutex", "WrGuardedMutex", "WrRundown",
        "MaximumWaitReason",
    ],
    "PP_NPAGED_LOOKASIDE_NUMBER": [
        "LookasideSmallIrpList", "LookasideLargeIrpList", "LookasideMdlList",
        "LookasideCreateInfoList", "LookasideNameBufferList",
        "LookasideTwilightList", "LookasideCompletionList", "LookasideMaximumList",
    ],
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
            def replace_variant(m: re.Match) -> str:
                # Check if this line is a comment
                line_start = code.rfind('\n', 0, m.start()) + 1
                line_text = code[line_start:m.start()]
                if '//' in line_text:
                    return m.group(0)
                # Check if preceded by `const ` on the same line
                if re.search(r'\bconst\s+$', line_text):
                    return m.group(0)
                return f'{prefixed}{variant}'

            code = re.sub(
                rf'(?<!{re.escape(prefixed)})(?<![A-Za-z_]){re.escape(variant)}(?![A-Za-z0-9_])',
                replace_variant,
                code,
            )
            # Fix double-application
            code = code.replace(f'{prefixed}{prefixed}{variant}',
                                f'{prefixed}{variant}')

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
    lines = code.split('\n')
    new_lines = []
    for line in lines:
        if 'ObAttributes' in line:
            line = re.sub(r'(OBJ_\w+(?:\))?)\s+as\s+u64', r'\1 as _', line)
        new_lines.append(line)
    return '\n'.join(new_lines)


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
            preceding = code[max(0, start - 80):start]
            if '#[no_mangle]' in preceding.split('\n')[-2:]:
                continue
            # Also check if there's a #[cfg(...)] + #[no_mangle] block
            lines_before = preceding.rstrip().rsplit('\n', 3)
            if any('#[no_mangle]' in lb for lb in lines_before):
                continue
            # Insert #[no_mangle] before the function
            indent = m.group(1)
            code = code[:start] + f'{indent}#[no_mangle]\n' + code[start:]
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
            rf'^([ \t]*)(pub\s+static\s+mut\s+{re.escape(name)}\s*:)',
            re.MULTILINE,
        )
        for m in pattern.finditer(code):
            start = m.start()
            preceding = code[max(0, start - 80):start]
            if any('#[no_mangle]' in lb for lb in preceding.split('\n')[-3:]):
                continue
            indent = m.group(1)
            code = code[:start] + f'{indent}#[no_mangle]\n' + code[start:]
            break
    return code


def fix_fastcall_exports(code: str, exports: dict[str, SpecEntry]) -> str:
    """Duplicate fastcall exports into dual cfg blocks for i386 vs others.

    Functions with `@ fastcall` in ntoskrnl.spec need different calling
    conventions on i386 (extern "fastcall") vs amd64/arm64 (extern "C").
    The LLM generates a single extern "system" version.
    """
    fastcall_names = [name for name, entry in exports.items()
                      if entry.convention == "fastcall"]
    if not fastcall_names:
        return code

    for name in fastcall_names:
        # Find the function definition — may have #[no_mangle] above it
        # Match pattern: optional #[no_mangle]\n pub unsafe extern "..." fn Name(...) { body }
        pattern = re.compile(
            rf'^([ \t]*#\[no_mangle\]\n)?'
            rf'([ \t]*)(pub\s+unsafe\s+extern\s+"[^"]+"\s+fn\s+{re.escape(name)}\s*\([^)]*\))',
            re.MULTILINE,
        )
        m = pattern.search(code)
        if not m:
            continue

        # Find the complete function body by matching braces
        fn_start = m.start()
        # Find the opening brace
        brace_pos = code.index('{', m.end())
        depth = 1
        pos = brace_pos + 1
        while depth > 0 and pos < len(code):
            if code[pos] == '{':
                depth += 1
            elif code[pos] == '}':
                depth -= 1
            pos += 1
        fn_end = pos

        indent = m.group(2)
        # Extract just the params and return type
        # Re-parse the signature to get params
        sig_match = re.search(
            rf'fn\s+{re.escape(name)}\s*(\([^)]*\)(?:\s*->\s*\S+)?)',
            code[m.start():brace_pos],
        )
        if not sig_match:
            continue
        params_and_ret = sig_match.group(1)
        body = code[brace_pos:fn_end]

        # Build the dual-cfg version
        x86_block = (
            f'{indent}#[cfg(target_arch = "x86")]\n'
            f'{indent}#[no_mangle]\n'
            f'{indent}pub unsafe extern "fastcall" fn {name}{params_and_ret} {body}'
        )
        other_block = (
            f'\n\n{indent}#[cfg(not(target_arch = "x86"))]\n'
            f'{indent}#[no_mangle]\n'
            f'{indent}pub unsafe extern "C" fn {name}{params_and_ret} {body}'
        )

        code = code[:fn_start] + x86_block + other_block + code[fn_end:]

    return code


def fix_self_imports(code: str) -> str:
    """Remove self-imports: names imported from ddk:: that are also defined locally.

    The LLM often imports functions from ddk:: that it also defines with
    #[no_mangle] in the same file, causing E0255 (duplicate definitions).
    """
    # Collect locally-defined #[no_mangle] function names
    local_fns = set()
    for m in re.finditer(r'#\[no_mangle\]\s*\n\s*pub\s+(?:unsafe\s+)?(?:extern\s+"[^"]+"\s+)?fn\s+(\w+)',
                         code):
        local_fns.add(m.group(1))

    # Also collect #[no_mangle] pub static mut names
    for m in re.finditer(r'#\[no_mangle\]\s*\n\s*pub\s+static\s+mut\s+(\w+)',
                         code):
        local_fns.add(m.group(1))

    if not local_fns:
        return code

    # Find and clean up ddk import blocks
    def clean_import_block(match: re.Match) -> str:
        full = match.group(0)
        prefix = match.group(1)  # "use ntkrnl_sys::ddk::{"
        body = match.group(2)    # the imports between { }
        suffix = match.group(3)  # "};"

        # Parse individual imports
        items = [item.strip() for item in body.split(',')]
        items = [item for item in items if item]  # remove empty
        # Filter out locally-defined names
        filtered = [item for item in items if item not in local_fns]

        # If nothing was removed, return original unchanged
        if len(filtered) == len(items):
            return full

        if not filtered:
            return f"// (all imports from ddk moved to local definitions)"

        new_body = ', '.join(filtered)
        # Re-wrap if multi-line original
        if '\n' in body:
            new_body = '\n    ' + ',\n    '.join(filtered) + ',\n'

        return f'{prefix}{new_body}{suffix}'

    # Match use ntkrnl_sys::ddk::{...};  (potentially multi-line)
    code = re.sub(
        r'(use ntkrnl_sys::ddk::\{)(.*?)(\};)',
        clean_import_block,
        code,
        flags=re.DOTALL,
    )

    return code


# ============================================================================
# Callback calling convention fixes
# ============================================================================

def fix_callback_conventions(code: str) -> str:
    """Fix callback function calling conventions for cross-platform compatibility.

    Bindgen generates extern "C" on 64-bit and extern "stdcall" on i386 for
    callback function pointer types. Using extern "system" should work but
    Rust's type system considers extern "C" and extern "system" as different
    types even though they have the same ABI on 64-bit.

    For callbacks passed to ExEnumHandleTable, ExChangeHandle, ExSweepHandleTable,
    ExDupHandleTable — these need cfg-gated calling conventions.
    """
    # This is left to the system prompt for now — it's context-dependent
    # which functions are callbacks. The prompt instructs using extern "C"
    # on 64-bit and adding cfg for i386.
    return code


# ============================================================================
# Main entry point
# ============================================================================

def postprocess(
    code: str,
    exports: dict[str, SpecEntry] | None = None,
    globals: list[CGlobal] | None = None,
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
    - #[no_mangle] for spec exports and globals (when metadata provided)
    - Fastcall dual-cfg for i386 (when metadata provided)
    - Self-import removal (always safe)

    Does NOT do union field access fixes (__bindgen_anon_N) — those
    require knowing which struct type is being accessed and are handled
    by the system prompt instead.
    """
    code = fix_imports(code)
    code = fix_type_names(code)
    code = fix_enum_variants(code)
    code = fix_portable_types(code)
    if exports:
        code = fix_exports(code, exports)
        code = fix_fastcall_exports(code, exports)
    if globals:
        code = fix_global_exports(code, globals)
    code = fix_self_imports(code)
    return code


def postprocess_file(path: Path) -> str:
    """Read a file, post-process it, return the result."""
    code = path.read_text()
    return postprocess(code)
