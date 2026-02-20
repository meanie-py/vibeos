"""Post-processor for LLM-generated Rust translations.

Applies SAFE mechanical transforms to fix predictable differences between
what the LLM generates and what the ntkrnl-sys bindgen output expects.

IMPORTANT: Does NOT attempt union field access fixes (.__bindgen_anon_N).
Those are context-dependent (different structs have same-named fields in
different positions) and must be handled by the system prompt teaching the
LLM to generate correct accessors directly.
"""

import re
from pathlib import Path


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

def fix_enum_variants(code: str) -> str:
    """Fix enum variant names to match bindgen output.

    Bindgen generates _ENUM_NAME_VariantName, not just VariantName.
    """
    # OB_OPEN_REASON variants
    for variant in ("ObCreateHandle", "ObOpenHandle", "ObDuplicateHandle", "ObInheritHandle"):
        # Replace standalone variant name (not after _OB_OPEN_REASON_)
        code = re.sub(
            rf'(?<!_OB_OPEN_REASON_)(?<![A-Za-z_]){variant}(?![A-Za-z0-9_])',
            f'_OB_OPEN_REASON_{variant}',
            code,
        )
        # Fix double-application
        code = code.replace(f'_OB_OPEN_REASON__OB_OPEN_REASON_{variant}',
                            f'_OB_OPEN_REASON_{variant}')

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

def postprocess(code: str) -> str:
    """Apply all post-processing transforms to LLM-generated Rust code.

    Only does SAFE transforms that don't depend on struct type context:
    - Type name underscore-prefixing (always correct)
    - Import fixes (ddk::* → selective, missing c_void)
    - Enum variant name prefixing (always correct)
    - Portable type width (as u64 → as ULONG_PTR for OBJ_* constants)

    Does NOT do union field access fixes (__bindgen_anon_N) — those
    require knowing which struct type is being accessed and are handled
    by the system prompt instead.
    """
    code = fix_imports(code)
    code = fix_type_names(code)
    code = fix_enum_variants(code)
    code = fix_portable_types(code)
    return code


def postprocess_file(path: Path) -> str:
    """Read a file, post-process it, return the result."""
    code = path.read_text()
    return postprocess(code)
