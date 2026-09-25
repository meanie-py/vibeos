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

import os
import re
from functools import lru_cache
from pathlib import Path
from typing import TYPE_CHECKING

from .abi_enforce import enforce_abi_from_c
from .abi_map import build_win32k_abi_map
from .foreign_abi import split_variadic_system_blocks

if TYPE_CHECKING:
    from collections.abc import Callable

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

    DPRINT/DPRINT1 → preserved macro invocations. The crate must provide their
      source-appropriate implementation; deleting a diagnostic also drops argument
      side effects and line-based deletion can swallow following definitions.
    Wine ERR/WARN/TRACE/FIXME/MESSAGE → KEPT, rewritten to the Rust macro form
      (`ERR(` → `ERR!(`). The crate provides `macro_rules!` ERR/WARN/TRACE/FIXME
      that expand to a channel-flag-gated `ros_dbg_log` call (libwine logger),
      so the traces survive translation — invaluable on boot/logon paths — and
      are controlled at one knob (the `wine_debug` cargo feature → channel flags).
      This is the deliberate counterpart to c2rust's *anchored* form, which already
      emits the gated `if (flags & class) { ros_dbg_log(...) }` expansion; both are
      now driven by the same default flags. Predicate forms (TRACE_ON/ERR_ON) and
      channel-qualified TRACE_/ERR_ forms are NOT matched (require `(` after name).
    UNIMPLEMENTED remains a source diagnostic macro; never turn it into a panic.
    offset_of → core::mem::offset_of (requires Rust 1.77+).
    """
    # Preserve calls and every source line. Substrate closure owns debug macro
    # implementations, including whether DPRINT is disabled by this TU's NDEBUG.
    from .debug_macros import preserve_debug_macros

    code = preserve_debug_macros(code)

    # Tame c2rust's wine-channel preamble: it hardcodes `flags: !(0) as c_uchar`
    # (0xFF = every class on → serial spam). Route it through the crate's
    # `WINE_DBG_DEFAULT_FLAGS` (0 normally, 0xFF under the `wine_debug` feature) so
    # BOTH the anchored `if (flags & class)` form and the rewritten ERR!/TRACE!
    # macros default to silent. Best-effort: only fires when the crate defines the
    # const (wine-style DLLs); harmless elsewhere.
    if "WINE_DBG_DEFAULT_FLAGS" in code or "ros_dbg_log" in code or "!(" in code:
        code = re.sub(
            r"flags:\s*!\(\s*0[^,]*?\)\s*as\s*(?:::core::ffi::)?c_uchar",
            "flags: WINE_DBG_DEFAULT_FLAGS",
            code,
        )

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
# memoffset:: paths (there is no memoffset crate in this workspace)
# ============================================================================

# The model reliably reaches for the `memoffset` crate for `offset_of!` (it was
# THE way to do this before Rust 1.77 stabilised `core::mem::offset_of!`), and
# it emits two shapes, both seen for real in the ole32 run:
#   memoffset::offset_of!(T, f)             <- the crate's own macro
#   memoffset::core::mem::offset_of!(T, f)  <- a hallucinated hybrid of the two
# Neither compiles: no crate in sdk/lib/rust/ depends on `memoffset`.  The
# workspace answer is always core's macro.
#
# NOTE: the `memoffset::` PATH prefix is what we key on, never the bare word —
# `memoffset` is also a legitimate *field* name in the wild (oleaut32's typelib
# MSFT_TypeInfoBase.memoffset), and `(*base).memoffset` must survive untouched.
_RE_MEMOFFSET_PATH = re.compile(r"\bmemoffset::(?:core::mem::)?offset_of\s*!")
_RE_MEMOFFSET_USE = re.compile(r"^[ \t]*use\s+memoffset::.*?;[ \t]*\r?\n", re.MULTILINE)


def fix_memoffset_paths(code: str) -> str:
    """Rewrite `memoffset::[core::mem::]offset_of!` → `core::mem::offset_of!`.

    Also rewrites `use memoffset::offset_of;` → `use core::mem::offset_of;`
    (keeping any bare `offset_of!(...)` call sites the model paired with it
    resolvable) and drops any other `use memoffset::...;` line, since the crate
    does not exist in this workspace at all.
    """
    code = _RE_MEMOFFSET_PATH.sub("core::mem::offset_of!", code)

    def fix_use(m: re.Match) -> str:
        line = m.group(0)
        if re.search(r"\buse\s+memoffset::offset_of\s*;", line):
            return line.replace("memoffset::offset_of", "core::mem::offset_of")
        # Anything else from a crate we don't have: drop the import outright.
        return ""

    return _RE_MEMOFFSET_USE.sub(fix_use, code)


# ============================================================================
# Old-bindgen union ACCESSOR-CALL form  →  plain field access
# ============================================================================

# Bindgen used to generate *methods* for anonymous-union members
# (`unsafe { x.u1() }`), and the model was trained on that era.  The bindgen we
# ship generates PLAIN FIELDS (verified: `pub QuadPart: LONGLONG` in
# sdk/lib/rust/com-sys/src/bindgen_*.rs — zero `pub fn QuadPart(` anywhere in
# the tree), so every `x.QuadPart()` is a compile error.
#
# Scoped by NAME (a curated accessor list), not by shape, because `.foo()` is
# obviously also the shape of every real method call in the file.  The names
# below are union/struct member names from the Windows headers; none of them is
# a method we define anywhere.
_ACCESSOR_FIELDS = frozenset(
    {
        "QuadPart",
        "HighPart",
        "LowPart",
        "u",
        "u1",
        "u2",
        "u3",
        "s",
        "s1",
        "s2",
        "s3",
        "n",
        "n1",
        "n2",
    }
)

# The DUMMY*/Anonymous* family is different: those names come from the SDK's
# NONAMELESSUNION macros and our bindgen NEVER emits them under any spelling
# (verified: zero hits for DUMMYUNIONNAME/DUMMYSTRUCTNAME/`pub Anonymous` across
# every sdk/lib/rust/*/src/bindgen_*.rs).  What it emits is `__bindgen_anon_N`.
# So for these, dropping the `()` is not enough — the field itself is renamed.
_RE_ANON_FAMILY = re.compile(
    r"(?<=[)\]\w])\.(?:DUMMYUNIONNAME|DUMMYSTRUCTNAME|Anonymous)(\d*)\s*(\(\s*\))?(?![A-Za-z0-9_])"
)

_RE_ACCESSOR_CALL = re.compile(
    r"(?<=[)\]\w])\.(" + "|".join(sorted(_ACCESSOR_FIELDS, key=len, reverse=True)) + r")\s*\(\s*\)"
)


def fix_bindgen_accessor_calls(code: str) -> str:
    """Old-bindgen union accessors: `x.QuadPart()` → `x.QuadPart`.

    Two rewrites:
      1. Known union/struct member names in method-call form lose the `()`
         (`x.u1()` → `x.u1`, `x.QuadPart()` → `x.QuadPart`).
      2. The `DUMMYUNIONNAME` / `DUMMYSTRUCTNAME` / `Anonymous` family — which
         our bindgen renames wholesale — becomes `__bindgen_anon_N`
         (`x.DUMMYUNIONNAME()` and bare `x.DUMMYUNIONNAME` both →
         `x.__bindgen_anon_1`; a trailing digit is carried over, so
         `DUMMYUNIONNAME2` → `__bindgen_anon_2`).

    Comment lines are skipped.  Runs BEFORE fix_bindgen_anon_field_access so
    that a `.LowPart()` first becomes `.LowPart` and then gets its
    `.__bindgen_anon_1.` qualifier from that pass.
    """
    lines = code.split("\n")
    out = []
    for line in lines:
        if line.lstrip().startswith("//"):
            out.append(line)
            continue
        line = _RE_ACCESSOR_CALL.sub(r".\1", line)
        line = _RE_ANON_FAMILY.sub(lambda m: f".__bindgen_anon_{m.group(1) or '1'}", line)
        out.append(line)
    return "\n".join(out)


# ============================================================================
# Rust-keyword C field names (`ref`, `type`, ...) — bindgen appends `_`
# ============================================================================

# A C struct field named `ref` (wine's refcounts: StorageBaseImpl.ref) is a hard
# Rust PARSE error, so bindgen escapes it by appending an underscore — verified
# against the generated bindings we actually ship (`pub type_: DWORD` in
# com-sys, `pub in_: [c_uchar; 64]` in advapi32-sys).  Trailing underscore, NOT
# a raw identifier, and NOT a numeric suffix.
#
# rustconv was not self-consistent about this: a single ole32 file declared the
# field `ref_` but then reached for it as `.ref`, `.ref_` AND `.ref_0` at
# different use sites.  Canonicalise every use site onto bindgen's spelling.
#
# Curated keyword list: Rust strict + reserved keywords that are plausible C
# field names.  Deliberately EXCLUDED: `await`/`self`/`super`/`crate` (postfix
# `.await` and `.self` shapes must not be touched) and `union` (a *weak*
# keyword — a field named `union` is a valid Rust ident, so bindgen keeps it).
_KEYWORD_FIELDS = (
    "abstract",
    "become",
    "box",
    "break",
    "const",
    "continue",
    "dyn",
    "enum",
    "final",
    "fn",
    "impl",
    "in",
    "let",
    "loop",
    "macro",
    "match",
    "mod",
    "move",
    "mut",
    "override",
    "priv",
    "ref",
    "return",
    "static",
    "struct",
    "trait",
    "type",
    "typeof",
    "unsafe",
    "use",
    "virtual",
    "where",
    "yield",
)

# `.ref` / `.r#ref` / `.ref_0`  →  `.ref_`
#   - `(?<=[)\]\w])` : a real receiver, so this is a field access
#   - `(?:_\d+)?\b`  : also catches the model's spurious `_0` disambiguator.
#     `.ref_` itself never matches (no word boundary between `f` and `_`), so the
#     pass is idempotent, and `.reference` never matches either.
_RE_KEYWORD_FIELD = re.compile(
    r"(?<=[)\]\w])\.(?:r#)?(" + "|".join(_KEYWORD_FIELDS) + r")(?:_\d+)?\b(?!_)"
)


def fix_keyword_field_names(code: str) -> str:
    """Canonicalise Rust-keyword C field names onto bindgen's `keyword_` spelling.

    `(*this).ref` / `(*this).r#ref` / `(*this).ref_0` → `(*this).ref_`.
    Generalised over the keyword list above so the next DLL's `type` / `box` /
    `match` field is handled without a code change.

    Comment lines are skipped.
    """
    lines = code.split("\n")
    out = []
    for line in lines:
        if line.lstrip().startswith("//"):
            out.append(line)
            continue
        out.append(_RE_KEYWORD_FIELD.sub(lambda m: f".{m.group(1)}_", line))
    return "\n".join(out)


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


# The conventions that exist ONLY on i386 and are plain cdecl everywhere else, mapped to
# the Rust `extern` string each needs on x86. Both are spelled per-arch in the .spec
# itself -- msvcrt.spec carries `thiscall -arch=i386` beside `cdecl -arch=win64` and
# `cdecl -arch=arm` for the very same export -- so this dual is transcribing the spec, not
# inventing a rule.
_X86_ONLY_CONVENTIONS = {"fastcall": "fastcall", "thiscall": "thiscall"}

# Poison value for `_import_abi_map`: deliberately not a Rust extern string. See the note
# at its `conv` table for why a thiscall name is dropped rather than mapped.
_THISCALL_X86_ONLY = "<thiscall: x86-only, no single ABI string>"


def fix_fastcall_exports(code: str, exports: dict[str, SpecEntry]) -> str:
    """Duplicate fastcall/thiscall exports into dual cfg blocks for i386 vs others.

    Functions with `@ fastcall` in ntoskrnl.spec need different calling
    conventions on i386 (extern "fastcall") vs amd64/arm64 (extern "C").
    The LLM generates a single extern "system" version.

    `thiscall` rides the same path, and NOT the `_SPEC_EXPORT_ABI` collapse below, because
    the two are not the same ABI: stdcall passes `this` on the stack, thiscall passes it in
    ECX. Mapping msvcrt's 40-odd C++ methods to `extern "system"` would compile, link and
    export correctly, and every one of them would read `this` from the wrong place on i386
    only. `extern "thiscall"` is stable (rustc 1.92) but x86-ONLY -- naming it on amd64 is
    a hard E0570 -- which is the other half of why this dual is required rather than nice.

    Keyed on `impl_symbol`, not on the export name, and that is not a detail: msvcrt's C++
    methods are exported under their MANGLED names (`??0bad_cast@@QAE@ABQBD@Z`) while the
    Rust fn is called `bad_cast_ctor`, so a dict-key match finds nothing and this whole pass
    silently does not run. It is also why the convention cannot be read off "the" entry --
    one impl has THREE spec lines, `thiscall -arch=i386` beside two `cdecl` ones, and only
    the i386 line carries the convention that needs a dual. Take the x86 line where one
    exists; a name whose only lines are cdecl is left alone.
    """
    x86_abi_of: dict[str, str] = {}
    for entry in exports.values():
        abi = _X86_ONLY_CONVENTIONS.get(entry.convention)
        if abi is None:
            continue
        arches = getattr(entry, "arch_filter", None) or []
        if arches and "i386" not in arches:
            continue
        x86_abi_of[getattr(entry, "impl_symbol", None) or entry.name] = abi
    if not x86_abi_of:
        return code

    for name, x86_abi in x86_abi_of.items():
        # Find the function definition — may have #[no_mangle] above it.
        # Match only as far as the NAME: the parameter list is then walked with a paren
        # counter rather than matched. `\([^)]*\)` cannot see a parameter that itself
        # contains parens, and every nullable C callback translates to exactly that
        # (`Option<unsafe extern "C" fn()>`), so the regex form silently skipped them.
        pattern = re.compile(
            rf"^([ \t]*#\[no_mangle\]\n)?"
            rf'([ \t]*)pub\s+unsafe\s+extern\s+"[^"]+"\s+fn\s+{re.escape(name)}\s*(?=\()',
            re.MULTILINE,
        )
        m = pattern.search(code)
        if not m:
            continue

        fn_start = m.start()
        params_start = code.index("(", m.end() - 1)

        # Find the complete function body by matching braces.
        brace_pos = code.index("{", params_start)
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
        # Everything between the '(' and the body IS the params-and-return-type, taken as
        # a SLICE rather than re-matched. The old `->\s*\S+` stopped at the first space, so
        # `-> *mut exception` was rewritten as `-> *mut` and the file no longer parsed.
        # It never fired before because ntoskrnl's fastcall exports return single tokens
        # (VOID/BOOLEAN/PVOID); msvcrt's C++ ctors are the first pointer returns to reach
        # this pass.
        params_and_ret = code[params_start:brace_pos].rstrip()
        body = code[brace_pos:fn_end]

        # Build the dual-cfg version
        x86_block = (
            f'{indent}#[cfg(target_arch = "x86")]\n'
            f"{indent}#[no_mangle]\n"
            f'{indent}pub unsafe extern "{x86_abi}" fn {name}{params_and_ret} {body}'
        )
        other_block = (
            f'\n\n{indent}#[cfg(not(target_arch = "x86"))]\n'
            f"{indent}#[no_mangle]\n"
            f'{indent}pub unsafe extern "C" fn {name}{params_and_ret} {body}'
        )

        code = code[:fn_start] + x86_block + other_block + code[fn_end:]

    return code


# spec convention -> the single Rust `extern` spelling for a uniform-per-arch export.
# fastcall AND thiscall are intentionally ABSENT: both need the per-arch dual cfg that
# `fix_fastcall_exports` emits, so this pass must not collapse either to one ABI. `extern`
# (a pure .spec forwarder) and `stub` have no Rust body to touch.
# `thiscall` used to sit here mapped to "system", which is stdcall — a DIFFERENT ABI, not a
# spelling of the same one: stdcall takes `this` as the first stack argument, thiscall takes
# it in ECX. Nothing downstream could catch it, because the export name, the arity and the
# stack cleanup all still agree; only the register does not, and only on i386.
_SPEC_EXPORT_ABI = {
    "stdcall": "system",
    "cdecl": "C",
    "varargs": "C",
}


def enforce_export_abi_from_spec(code: str, exports: dict[str, SpecEntry]) -> str:
    """Force every spec-EXPORTED fn's `extern "..."` to the convention its own DLL's .spec
    declares — the authoritative source, immune to per-DLL macro aliasing.

    WHY (the #1 i386 crash cause, closed for exports). `enforce_abi_from_c` derives the
    convention by reading the C *source* for a known macro (WINAPI/NTAPI/WSPAPI/…). A DLL
    that hides the convention behind a LOCAL alias defeats that entirely: `wshtcpip.c`
    defines every export as `EXPORT\\nWSHOpenSocket(...)` with `#define EXPORT WINAPI`
    (wshtcpip.h:25). `EXPORT` is not a known keyword, so `_derive_convention` FALLS BACK to
    cdecl and emits `extern "C"` for a `__stdcall` export — which on i386 makes the .def
    demand `_Name@N`, so the DLL will not link (or the caller walks the stack). And you
    cannot just teach the C reader `EXPORT`: a DDK header spells the same token
    `#define EXPORT __cdecl`. The .spec (`@ stdcall WSHOpenSocket`) is unambiguous ground
    truth, so it — not the C macro — decides an export's convention.

    This is `enforce_win32k_abi`'s philosophy (spec is ground truth, taken out of the LLM's
    hands) generalized to any DLL with a .spec. It runs AFTER `enforce_abi_from_c` so the
    spec has the final word on exported names; internal/static fns (never in `exports`) keep
    the C-source-derived convention. Idempotent: only rewrites when the current ABI differs.
    """
    for name, entry in exports.items():
        target = _SPEC_EXPORT_ABI.get(getattr(entry, "convention", ""))
        if target is None:  # fastcall (dual-cfg elsewhere) / extern / stub — leave alone
            continue
        # Rewrite the definition-site ABI: `pub [unsafe] extern "<X>" fn <name>(` only.
        # Import decls (`fn <name>(` inside an `extern "X" { … }` block) lack the
        # `pub … extern "…" fn` prefix, so they are never matched.
        pat = re.compile(
            rf'(pub\s+(?:unsafe\s+)?extern\s+")([^"]+)("\s+fn\s+{re.escape(name)}\s*\()'
        )

        def _sub(m: re.Match[str], _t: str = target) -> str:
            return m.group(0) if m.group(2) == _t else m.group(1) + _t + m.group(3)

        code = pat.sub(_sub, code)
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
    """Only variadic declarations move to C; fixed declarations keep their system ABI."""
    return split_variadic_system_blocks(code)


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


_EXTERN_OPEN_RE = re.compile(r'^[ \t]*extern\s+"([^"]+)"\s*\{[ \t]*$')
_EXTERN_FN_NAME_RE = re.compile(r"^[ \t]*(?:pub\s+)?fn\s+([A-Za-z_]\w*)\b")
_EXTERN_ATTR_RE = re.compile(r"^[ \t]*#\[[^\]]*\][ \t]*$")


#: Directories with no hand-written `.spec` in them, pruned BEFORE descending.
#: `target` is the one that matters: it is cargo's output tree, it is enormous,
#: and it churns constantly.
_SPEC_WALK_SKIP = {"target", ".git", "__pycache__", ".responses", "node_modules"}


def _walk_specs(root: Path):
    """Every hand-written `.spec` under `root`, pruning generated trees.

    `Path.rglob` was the obvious spelling and it is unsafe here. It descends
    into everything and only lets the caller filter what it has already
    yielded, so a `cargo build` running concurrently -- which the merge queue
    guarantees, because it gates three arches while other lanes build -- can
    delete a temp directory out from under the generator. pathlib then raises
    FileNotFoundError from inside `scandir` and takes the whole walk down
    mid-iteration. That bounced a lane out of the queue with a TOOLING_FAIL
    that had nothing to do with its change; the vanished path was
    `sdk/lib/rust/target/...-rustlib/i686-pc-windows-gnuutcZEh`.

    `os.walk` fixes both halves: pruning `dirnames` in place means the churning
    directories are never entered at all, and its default `onerror=None`
    swallows a directory that disappears anyway. Same prune-then-walk shape
    `tools/vendored.py` uses, and the same defect `rglob_stable` in
    tests/qemu/cli/provenance.py exists to work around on the other side of the
    tree -- there the answer was to retry the walk, here it is to not walk the
    volatile part in the first place.
    """
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [
            d for d in dirnames if d not in _SPEC_WALK_SKIP and not d.startswith("build-")
        ]
        for fn in filenames:
            if fn.endswith(".spec"):
                yield Path(dirpath) / fn


@lru_cache(maxsize=1)
def _import_abi_map() -> dict[str, str]:
    """symbol -> required extern ABI ("C" | "system"), from EVERY .spec in the tree.

    THE INVARIANT: an imported symbol's calling convention is decided by the DLL that
    EXPORTS it, and every one of those DLLs states it in a .spec. kernel32.spec says
    `@ stdcall HeapAlloc(long long long)`; msvcrt.spec says `@ cdecl strcpy(ptr str)`.
    So we never have to guess — and we must not, because guessing wrong is invisible.

    WHY THIS IS NEEDED. An LLM translating a user-mode DLL sees a C file that calls
    `strcpy(dst, src)` and `HeapAlloc(heap, 0, n)` and `SetLastError(e)` side by side, with
    nothing in the source distinguishing their conventions (the C compiler knew from the
    headers). It groups them into whichever `extern` block it happens to pick. ws2_32's raw
    translation declared **HeapAlloc as `extern "C"` in some modules and `extern "system"` in
    others** — HeapAlloc is stdcall, so the cdecl sites walk the i386 stack on every
    allocation. It compiles. It links. It boots.

    Nothing was checking: verify_ffi validates a module's decls against ITS OWN .spec, and
    HeapAlloc is not in ws2_32.spec because it is an IMPORT. The one checker that could have
    caught it had no reason to look at that name at all.

    Conflicts (a name exported with different conventions by different DLLs — 20 of 18,028,
    all kernel Ke*/Rtl* arch-variants) are DROPPED from the map rather than guessed. Fail
    safe: an unknown name is left exactly as the model wrote it.

    `varargs` is cdecl (that is how a variadic C fn is called).

    `thiscall` is DROPPED rather than mapped, and the reason is the same one that took it
    out of `_SPEC_EXPORT_ABI`: it is not a spelling of stdcall, it is a different ABI
    (`this` in ECX, not on the stack) that exists only on i386, so no single `extern`
    string is right on all three arches. This pass emits one string, so for these names it
    must emit none and leave the model's text alone -- the documented fail-safe above.
    An imported thiscall method needs the per-arch dual `fix_fastcall_exports` writes for
    definitions; nothing emits that for imports yet (task #47).

    `fastcall` is left mapped to "system" DELIBERATELY and is the same hazard: it is
    correct on amd64/arm64 (where the spec says cdecl and the two coincide for the
    kernel's Ke*/Rtl* fastcall pairs) and wrong on i386. It stays until someone has an
    ntoskrnl case in hand to test the change against -- changing it blind would move a
    known-shaped bug into a lane that cannot see it.
    """
    from .config import REACTOS_ROOT

    conv = {
        "stdcall": "system",
        "fastcall": "system",
        "cdecl": "C",
        "varargs": "C",
        # Not a Rust extern string and not meant to be one: it is a poison value. Any name
        # a .spec exports as thiscall ANYWHERE picks it up, and the filter below then drops
        # that name from the map entirely -- the same treatment a genuine cross-DLL conflict
        # gets. Dropping the name from the regex instead would be subtly worse: msvcrt
        # exports these under three per-arch spec lines, so the cdecl-arch=win64 line would
        # survive alone and force `extern "C"` onto an i386 import that must be thiscall.
        "thiscall": _THISCALL_X86_ONLY,
    }
    pat = re.compile(
        r"^@\s+(stdcall|cdecl|varargs|thiscall|fastcall)\s+(?:-\S+\s+)*([A-Za-z_]\w*)\s*\("
    )
    seen: dict[str, set[str]] = {}
    for spec in _walk_specs(REACTOS_ROOT):
        try:
            text = spec.read_text(errors="replace")
        except OSError:
            # Vanished between the walk and the read -- same race as the walk
            # itself, one step later. Skipping is right: a .spec that a
            # concurrent build just deleted was never ours to read.
            continue
        for line in text.splitlines():
            m = pat.match(line.strip())
            if m:
                seen.setdefault(m.group(2), set()).add(conv[m.group(1)])
    # Unambiguous names only, and never a name any .spec calls thiscall.
    return {
        name: next(iter(cs))
        for name, cs in seen.items()
        if len(cs) == 1 and _THISCALL_X86_ONLY not in cs
    }


def enforce_import_abi(code: str, exports: dict | None = None) -> str:
    """Force every imported fn decl into the extern block its exporting DLL's .spec requires.

    See _import_abi_map for the invariant and for why nothing else catches this. Names the
    module itself EXPORTS are skipped (a DLL may legitimately export its own fn that shares a
    name with someone else's import; forcing that one would invert the bug).
    """
    abi_of = _import_abi_map()
    if not abi_of:
        return code
    own = set(exports or {})

    lines = code.splitlines(keepends=True)
    out: list[str] = []
    # required-abi -> [decl lines]
    relocated: dict[str, list[str]] = {}
    moved: list[str] = []
    i, n = 0, len(lines)
    while i < n:
        m = _EXTERN_OPEN_RE.match(lines[i].rstrip("\n"))
        if not m or m.group(1) not in ("system", "stdcall", "C", "cdecl"):
            out.append(lines[i])
            i += 1
            continue
        # Normalize the block's ABI to the two Rust spellings we emit.
        block_abi = "system" if m.group(1) in ("system", "stdcall") else "C"
        depth, j = 1, i + 1
        while j < n:
            depth += lines[j].count("{") - lines[j].count("}")
            if depth == 0:
                break
            j += 1
        if j >= n:  # unterminated — leave exactly as-is
            out.append(lines[i])
            i += 1
            continue

        kept: list[str] = []
        pending: list[str] = []
        hit = False
        for ln in lines[i + 1 : j]:
            if _EXTERN_ATTR_RE.match(ln.rstrip("\n")):
                pending.append(ln)
                continue
            fn_m = _EXTERN_FN_NAME_RE.match(ln)
            name = fn_m.group(1) if fn_m else None
            want = abi_of.get(name) if name and name not in own else None
            if want and want != block_abi:
                relocated.setdefault(want, []).extend(pending)
                relocated.setdefault(want, []).append(ln)
                moved.append(f"{name}->{want}")
                pending.clear()
                hit = True
                continue
            kept.extend(pending)
            pending.clear()
            kept.append(ln)
        kept.extend(pending)

        if not hit:
            out.extend(lines[i : j + 1])  # byte-identical passthrough
            i = j + 1
            continue
        if any(k.strip() for k in kept):
            out.append(f'extern "{m.group(1)}" {{\n')
            out.extend(k if k.endswith("\n") else k + "\n" for k in kept)
            out.append("}\n")
        i = j + 1

    if not moved:
        return code
    out.append(
        f"// enforce_import_abi: moved {len(moved)} import decl(s) into the extern block their\n"
        f"// exporting DLL's .spec requires: {', '.join(sorted(set(moved)))}.\n"
        "// The CRT is cdecl, Win32 is stdcall, and the C source does not say which is which —\n"
        "// a wrong guess is a silent i386 stack walk on every call. The .spec is the truth.\n"
    )
    for abi, decls in sorted(relocated.items()):
        out.append(f'extern "{abi}" {{\n')
        out.extend(decls)
        out.append("}\n")
    return "".join(out)


def _filter_extern_block_decls(
    code: str,
    should_drop: Callable[[str], bool],
    note: str,
    abis: tuple[str, ...] | None = None,
) -> str:
    """Line-scan every `extern "X" { … }` block, dropping the decls `should_drop`
    selects. Linear — it cannot backtrack.

    This replaces a regex that was a catastrophic-backtracking bomb and hung
    `rustconv translate` outright on ole32's storage32.c and moniker.c. The old
    pattern matched the block body with an ambiguous `(…fn…;\\n)+` whose `[^;{]+`
    return-type class also matched newlines, so a single iteration could swallow
    several decl lines. That alone is survivable — the killer was that the
    stitcher's merged extern block legitimately contains `static` declarations
    among the fns, so the `+` ran out at the first `static` and could never reach
    the closing `}`. The engine then re-partitioned the preceding 46 fn decls
    every possible way: a clean 2^n (26 decls = 5s, 30 = 37s, 46 = never).

    Because the hang looked like the *stitcher* spinning, the run got killed and
    the pieces hand-assembled — silently bypassing all of postprocess(), ABI
    enforcement included. Nearly every defect in that translation traces here.

    The line scan is also strictly more capable: it can process the mixed
    fn/static blocks the old pattern could never match at all.
    """
    lines = code.splitlines(keepends=True)
    out: list[str] = []
    i, n = 0, len(lines)
    while i < n:
        m = _EXTERN_OPEN_RE.match(lines[i].rstrip("\n"))
        if not m or (abis is not None and m.group(1) not in abis):
            out.append(lines[i])
            i += 1
            continue

        abi = m.group(1)
        depth, j = 1, i + 1
        while j < n:
            depth += lines[j].count("{") - lines[j].count("}")
            if depth == 0:
                break
            j += 1
        if j >= n:  # unterminated block — leave it exactly as it was
            out.append(lines[i])
            i += 1
            continue

        kept: list[str] = []
        dropped: list[str] = []
        pending_attrs: list[str] = []
        for ln in lines[i + 1 : j]:
            if _EXTERN_ATTR_RE.match(ln.rstrip("\n")):
                pending_attrs.append(ln)
                continue
            fn_m = _EXTERN_FN_NAME_RE.match(ln)
            if fn_m and should_drop(fn_m.group(1)):
                dropped.append(fn_m.group(1))
                pending_attrs.clear()  # the attribute belonged to the dropped fn
                continue
            kept.extend(pending_attrs)
            pending_attrs.clear()
            kept.append(ln)  # non-fn lines (`static IID_Foo: IID;`) pass through
        kept.extend(pending_attrs)

        if not dropped:
            out.extend(lines[i : j + 1])  # byte-identical passthrough
            i = j + 1
            continue

        if any(k.strip() for k in kept):
            out.append(f'extern "{abi}" {{\n')
            out.extend(k if k.endswith("\n") else k + "\n" for k in kept)
            out.append("}\n")
        out.append(note.format(n=len(dropped), names=", ".join(dropped)))
        i = j + 1

    return "".join(out)


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

    def _is_gdiobj_macro(name: str) -> bool:
        prefix, _, rest = name.partition("_")
        return prefix in _GDIOBJ_MACRO_FAMILIES and rest in _GDIOBJ_MACRO_VERBS

    return _filter_extern_block_decls(
        code,
        _is_gdiobj_macro,
        "// drop_gdiobj_macro_externs: dropped {n} bogus extern decl(s) for "
        "GDI-object macro(s): {names} — these are macros in gdi/eng/<type>.h or "
        "gdi/ntgdi/<type>.h, not real fns. Inline-expand via GDIOBJ_* primitives "
        "per win32k_translation_prompt.md §6.\n",
        abis=("C", "system", "stdcall", "fastcall"),
    )


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
    return _filter_extern_block_decls(
        code,
        lambda name: bool(_PSEH_MACRO_EXTERN_NAMES_RE.match(name)),
        "// drop_pseh_macro_externs: dropped {n} bogus extern decl(s): {names} — "
        "PSEH _SEH2_* (and RtlGetProcessHeap) are macros, not symbols. Under "
        "USE_DUMMY_PSEH the SEH path is dead; translate the body directly. "
        "RtlGetProcessHeap → GetProcessHeap().\n",
    )


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


def collapse_sys_crate(code: str, sys_crate: str, sys_umbrella: str, crate_module: str = "") -> str:
    """Deterministically retarget a user-mode translation's FFI imports.

    rustconv's prompt/catalog are kernel-oriented, so the LLM emits
    `ntkrnl_sys::{ddk,base,constants,globals,macros}::Sym`. A user-mode
    target's substrate (ntdll-sys) re-exports its whole bindgen under
    one umbrella module (`ntdll_sys::nt`). This collapses every
    `ntkrnl_sys::<mod>::` to `<sys_crate>::<sys_umbrella>::` and
    replaces the now-redundant explicit `use` lines with a single glob
    — LLM-independent, mirrors enforce_win32k_abi philosophy.

    `crate_module` (e.g. "types") EXEMPTS the target's crate-local bindgen island
    from that collapse and sends it to `crate::<crate_module>::` instead. It must run
    FIRST, because the blanket `ntkrnl_sys::\\w+::` rule below would otherwise swallow
    `ntkrnl_sys::types::` into the sys umbrella — and those symbols are NOT there.
    ws2_32's `PWSSOCKET`/`TCATALOG`/`SOCKET_ERROR` live in ws2_32-rust's own generated
    `types` module, so collapsing them onto `ntdll_sys::nt` yields an unresolved-import
    wall at best, and at worst binds to a same-named-but-different NT symbol.

    `sys_umbrella` may be `""` for a substrate that re-exports its bindgen FLAT at the
    crate root (`msvcrt-sys`: `pub use bindgen_<arch>::*;`) rather than under a module
    (`ntdll-sys`: `pub mod nt`). Then the target path is just `<sys_crate>::`. Note ""
    is meaningfully different from None, which means "not a user-mode target" and skips
    this pass entirely — see `postprocess`.
    """
    pfx = f"{sys_crate}::{sys_umbrella}::" if sys_umbrella else f"{sys_crate}::"
    # 0. Crate-local bindgen island (gen_driver_types.sh) → `crate::<module>::`.
    #    MUST precede the blanket collapse in step 1.
    if crate_module:
        code = re.sub(
            rf"\b(?:ntkrnl_sys|{re.escape(sys_crate)})::{re.escape(crate_module)}::",
            f"crate::{crate_module}::",
            code,
        )
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
    _seg = (
        f"{re.escape(sys_crate)}::{re.escape(sys_umbrella)}"
        if sys_umbrella
        else re.escape(sys_crate)
    )
    use_pat = re.compile(
        rf"^[ \t]*use\s+{_seg}::"
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
    glob = f"use {sys_crate}::{sys_umbrella}::*;" if sys_umbrella else f"use {sys_crate}::*;"
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


_UTF16_LITERAL_RE = re.compile(
    r'"((?:[^"\\]|\\.)*?)"'  # a Rust string literal (group 1 = inner text)
    r"\s*\.encode_utf16\(\)"
    r"(?:\s*\.chain\(\s*core::iter::once\(0\)\s*\))?"  # optional NUL chain
    r"\s*\.collect::<\s*Vec\s*<\s*u16\s*>\s*>\(\)"
    r"(\s*\.as_ptr\(\))?"  # group 2 = trailing .as_ptr() (the dangling case)
)


def fix_utf16_vec_literals(code: str) -> str:
    """User-mode GUI targets: rewrite the LLM's
    `"Text\\0".encode_utf16().chain(once(0)).collect::<Vec<u16>>().as_ptr()`
    into `w!("Text").as_ptr()`.

    Two defects in one: (a) `Vec` is `alloc`/`std` — these crates are
    `no_std` staticlibs, so it doesn't even compile; (b) worse, when it DID
    compile the temporary `Vec` is dropped at the end of the statement, so
    the `.as_ptr()` is an immediate use-after-free. `winsrv_sys::w!` builds a
    `&'static [u16; N+1]` at compile time (NUL-terminated), so `.as_ptr()`
    is sound. Only applied on user-mode targets (where `w!` is in scope) —
    the kernel prompt never emits `encode_utf16` (kernel strings are bytes).
    """

    def repl(m: re.Match) -> str:
        inner = m.group(1)
        # w! adds the NUL, so strip a trailing literal \0 the model included.
        if inner.endswith("\\0"):
            inner = inner[:-2]
        tail = m.group(2) or ""
        return f'w!("{inner}"){tail}'

    return _UTF16_LITERAL_RE.sub(repl, code)


# The WIDE pointer spellings. A `b"..."` behind any of these is the defect below; behind a
# NARROW one (PCSTR/LPCSTR/*const c_char) it is correct and must not be touched. WCHAR is in
# the list and TCHAR deliberately is NOT: WCHAR is u16 on every target, whereas TCHAR is
# c_char in a module built without -DUNICODE, and the post-processor cannot see which.
_WIDE_PTR_TYPES = (
    "PCWSTR",
    "PWSTR",
    "LPCWSTR",
    "LPWSTR",
    "PCTSTR",
    "PTSTR",
    "LPCTSTR",
    "LPTSTR",
    "LPCOLESTR",
    "LPOLESTR",
    "BSTR",
    "PCUWSTR",
    "PUWSTR",
)
_WIDE_BYTE_STRING_RE = re.compile(
    r'b"((?:[^"\\]|\\.)*)"\s*\.as_ptr\(\)\s*as\s+'
    r"(\*\s*(?:const|mut)\s*(?:u16|WCHAR)\b|(?:" + "|".join(_WIDE_PTR_TYPES) + r")\b)"
)
_BYTE_ESCAPES = {"0": 0, "n": 0x0A, "r": 0x0D, "t": 0x09, "\\": 0x5C, '"': 0x22, "'": 0x27}


def _decode_rust_byte_string(body: str) -> bytes | None:
    """The bytes a Rust `b"..."` literal denotes, or None if we can't be sure."""
    out = bytearray()
    i = 0
    while i < len(body):
        c = body[i]
        if c != "\\":
            if ord(c) > 0x7F:
                return None  # non-ASCII source char in a byte string: not ours to judge
            out.append(ord(c))
            i += 1
            continue
        if i + 1 >= len(body):
            return None
        esc = body[i + 1]
        if esc == "x":
            if i + 3 >= len(body):
                return None
            try:
                out.append(int(body[i + 2 : i + 4], 16))
            except ValueError:
                return None
            i += 4
            continue
        if esc not in _BYTE_ESCAPES:
            return None
        out.append(_BYTE_ESCAPES[esc])
        i += 2
    return bytes(out)


def fix_wide_byte_string_literals(code: str) -> str:
    """Rewrite `b"A\\0S\\0U\\0S\\0\\0\\0".as_ptr() as PCWSTR` into `w!("ASUS") as PCWSTR`.

    The model hand-encodes a C `L"..."` as UTF-16LE bytes. The CONTENT is usually right, so
    nothing fails to compile and nothing looks wrong in review -- but `b"..."` has ALIGNMENT
    1, and a `*const u16` landing on an odd address is a BUGCHECK 0x1E
    STATUS_DATATYPE_MISALIGNMENT on amd64 and arm64 (mswsock shipped three of these; sysdm
    had 110). `w!` builds a real `[u16; N+1]` static, so it is aligned and NUL-terminated.

    Conservative by construction: anything that does not decode to clean NUL-terminated
    ASCII UTF-16 is left exactly as it was, because a wrong rewrite here is silent data
    corruption rather than a compile error.
    """

    def repl(m: re.Match) -> str:
        raw = _decode_rust_byte_string(m.group(1))
        if raw is None or len(raw) < 2 or raw[-1] != 0:
            return m.group(0)
        units = _as_utf16le(raw) or _as_narrow_ascii(raw)
        if units is None:
            return m.group(0)
        text = "".join(chr(u) for u in units).replace("\\", "\\\\").replace('"', '\\"')
        # keep the cast: `w!` yields *const u16, and the slot may want *mut / a typedef
        return f'w!("{text}") as {m.group(2)}'

    return _WIDE_BYTE_STRING_RE.sub(repl, code)


def _as_utf16le(raw: bytes) -> list[int] | None:
    """`b"A\\0S\\0\\0\\0"` -- a hand-encoded `L"AS"`. Right content, alignment 1."""
    if len(raw) % 2:
        return None
    units = [int.from_bytes(raw[i : i + 2], "little") for i in range(0, len(raw), 2)]
    if units[-1] != 0:
        return None  # no terminator -- not the pattern we understand
    units = units[:-1]
    if any(u == 0 or u >= 0x80 for u in units):
        return None  # interior NUL, or non-ASCII that `w!` would reject
    return units


def _as_narrow_ascii(raw: bytes) -> list[int] | None:
    """`b"LogEvent\\0"` behind a `*const u16` -- a NARROW string read as wide.

    Strictly worse than the misencoded case and a different bug: the content is not merely
    misaligned, it is garbage. `"Sy"` reinterpreted as UTF-16 is one code unit U+7953, so
    `RegOpenKeyExW` looks up a key nobody has ever created and the call just fails. sysdm
    shipped 30-odd of these, mostly registry paths and printf formats.
    """
    body = raw[:-1]
    if not body or any(b == 0 or b >= 0x80 for b in body):
        return None
    return list(body)


def strip_embedded_abspaths(code: str) -> str:
    """Determinism guard (deterministic-builds-plan.md rung 1).

    c2rust expands the C ``__FILE__`` macro to the ABSOLUTE path it was invoked
    with, so a translated .rs ends up carrying the build host's worktree root as
    string-literal DATA, e.g.::

        b"/home/me/.../<worktree>/dll/win32/kernel32/winnls/string/nls.c\\0"

    That data is invisible to ``--remap-path-prefix`` (which only rewrites
    rustc's own file refs), so it breaks cross-worktree byte-identity AND leaks
    the host's filesystem layout into every shipped translated binary. (A 1307-
    path sweep cleaned the historical ones; this keeps fresh translations clean.)

    Strip the repo-root prefix so ``__FILE__`` stays repo-relative + deterministic.
    Uses the config-detected ``REACTOS_ROOT`` (the worktree c2rust ran in), so it
    is portable across machines — not a hardcoded path.
    """
    from .config import REACTOS_ROOT

    root = str(REACTOS_ROOT).rstrip("/") + "/"
    return code.replace(root, "")


def postprocess(
    code: str,
    exports: dict[str, SpecEntry] | None = None,
    globals: list[CGlobal] | None = None,
    target_name: str = "ntoskrnl",
    sys_crate: str = "ntkrnl_sys",
    sys_umbrella: str | None = None,
    c_source_path: Path | None = None,
    com_method_macro: str | None = None,
    concrete_stdcall_slots: bool = True,
    crate_module: str = "",
) -> str:
    """Apply all post-processing transforms to LLM-generated Rust code.

    Args:
        code: Raw LLM-generated Rust code
        exports: Spec-exported functions (from resolver), enables #[no_mangle]
                 and fastcall fixes
        globals: C globals from scanner, enables global #[no_mangle] fixes
        com_method_macro: name of the crate's stdcall-wrapper macro (e.g.
                 "stdmethod"). A C `static … WINAPI` COM method is stdcall on
                 i386, and rustc will NOT unify `extern "system"` with the
                 `extern "stdcall"` fn type bindgen gives the vtbl slot — so the
                 body needs a per-arch cfg pair. Naming the macro here renders
                 that as one `stdmethod! { … }` instead of duplicating the body
                 under two cfgs (149 methods in oleaut32's remaining files).
                 None → the inline cfg-dual, for targets with no such macro
                 (kernel, HAL, drivers).

    Only does SAFE transforms that don't depend on struct type context:
    - Type name underscore-prefixing (always correct)
    - Import fixes (ddk::* → selective, missing c_void)
    - Enum variant name prefixing (always correct)
    - Portable type width (as u64 → as ULONG_PTR for OBJ_* constants)
    - Bindgen anon field access (LARGE_INTEGER .LowPart/.HighPart,
      IO_STATUS_BLOCK .Status — universal, struct-type independent)
    - Enum literal replacements (KernelMode→0i32, UserMode→1i32, etc.)
    - memoffset:: paths → core::mem::offset_of! (no memoffset crate here)
    - Old-bindgen union accessor CALLS (.QuadPart(), .u1(), .DUMMYUNIONNAME())
      → the plain fields / __bindgen_anon_N our bindgen actually emits
    - Rust-keyword C field names (.ref/.ref_0/.r#ref → .ref_, same for .type etc.)
    - #[no_mangle] for spec exports and globals (when metadata provided)
    - Fastcall dual-cfg for i386 (when metadata provided)
    - Self-import removal (always safe)

    NOTE: General union field access fixes (__bindgen_anon_N) are still
    context-dependent and handled by the system prompt.  Only the
    universally-safe cases above are done here.
    """
    code = strip_embedded_abspaths(code)
    code = fix_imports(code, target_name=target_name)
    code = fix_bogus_ddk_imports(code)
    code = fix_debug_macros(code)
    code = fix_type_names(code)
    code = fix_enum_variants(code)
    code = fix_portable_types(code)
    # Mechanical LLM output defects (all three hit for real in the ole32 run).
    # Must run BEFORE fix_bindgen_anon_field_access: the accessor-call pass turns
    # `.LowPart()` into `.LowPart`, which that pass then qualifies.
    code = fix_memoffset_paths(code)
    code = fix_bindgen_accessor_calls(code)
    code = fix_keyword_field_names(code)
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
        # Deliberately NO try/except. This is the one pass that guarantees a
        # translated `static … WINAPI` COM method keeps its __stdcall; if it
        # cannot run, the output is not safe to ship. Swallowing the failure and
        # printing to stderr is how ~130 ole32 vtbl methods shipped with the Rust
        # ABI — clean links on amd64/arm64, a corrupted stack on i386 at the first
        # call. Fail loud (AGENTS.md) and let the caller decide.
        code = enforce_abi_from_c(
            code,
            c_source_path,
            com_method_macro=com_method_macro,
            concrete_stdcall_slots=concrete_stdcall_slots,
        )
    # The .spec is ground truth for EXPORTS, so it overrides the C-source-derived
    # convention for any spec-exported name — a DLL-local `#define EXPORT WINAPI` defeats
    # the C-macro read (falls back to cdecl → `extern "C"` on a stdcall export → i386 link
    # break), but `@ stdcall` in the .spec does not. Runs LAST among the ABI passes so the
    # spec wins; win32k keeps its own bindgen/spec-fused enforcer above.
    if exports and target_name != "win32k":
        code = enforce_export_abi_from_spec(code, exports)
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
    #
    # `is not None` (not truthiness): "" is a REAL user-mode value meaning the
    # substrate re-exports its bindgen flat at the crate root (msvcrt-sys), and
    # such a target still needs every pass in this block — import-ABI enforcement,
    # inline-redeclaration stripping, the rest. Only None (a kernel target) skips.
    if sys_umbrella is not None:
        code = collapse_sys_crate(code, sys_crate, sys_umbrella, crate_module)
        # User-mode only: force every IMPORT into the extern block its exporting DLL's .spec
        # requires (CRT=cdecl, Win32=stdcall). The C source doesn't say which is which, so the
        # model guesses — and ws2_32's raw output declared HeapAlloc BOTH ways across modules.
        code = enforce_import_abi(code, exports)
        code = strip_user_inline_redeclarations(code)
        code = drop_pseh_macro_externs(code)
        code = fix_utf16_vec_literals(code)
        code = fix_wide_byte_string_literals(code)
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
