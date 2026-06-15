"""Match C identifiers against catalog to determine required imports."""

from dataclasses import dataclass, field

from .catalog import Catalog, Symbol
from .scanner import ScanResult
from .spec_parser import SpecEntry

# Well-known C identifiers that don't need imports (language/compiler builtins)
BUILTIN_NAMES = {
    "NULL",
    "TRUE",
    "FALSE",
    "VOID",
    "void",
    "NTSTATUS",
    "BOOLEAN",
    "BOOL",
    "ULONG",
    "LONG",
    "USHORT",
    "SHORT",
    "UCHAR",
    "CHAR",
    "WCHAR",
    "PVOID",
    "HANDLE",
    "SIZE_T",
    "PULONG",
    "PLONG",
    "PUSHORT",
    "PUCHAR",
    "PCHAR",
    "PWCHAR",
    "ULONG_PTR",
    "LONG_PTR",
    "DWORD",
    "BYTE",
    "WORD",
    "LONGLONG",
    "ULONGLONG",
    "LONG64",
    "ULONG64",
    "INT",
    "UINT",
    "CCHAR",
    "CSHORT",
    "PAGED_CODE",
    "DPRINT",
    "DPRINT1",
    "ASSERT",
    "ASSERTMSG",
    "NDEBUG",
    "UNIMPLEMENTED",
    # C keywords that appear as identifiers in tree-sitter
    "if",
    "else",
    "while",
    "for",
    "do",
    "return",
    "break",
    "continue",
    "switch",
    "case",
    "default",
    "goto",
    "sizeof",
    "typedef",
    "struct",
    "union",
    "enum",
    "static",
    "extern",
    "const",
    "volatile",
    "IN",
    "OUT",
    "OPTIONAL",
    "CONST",
    # Calling-convention / linkage / annotation macros — these are never
    # importable symbols on ANY target (they expand to attributes or
    # nothing). Globally correct to treat as builtins.
    "NTAPI",
    "WINAPI",
    "CALLBACK",
    "APIENTRY",
    "APIPRIVATE",
    "WINAPIV",
    "NTSYSAPI",
    "NTSYSCALLAPI",
    "NTKERNELAPI",
    "DECLSPEC_NORETURN",
    "FORCEINLINE",
    "NTHALAPI",
    "RESTRICTED_POINTER",
    "POINTER_ALIGNMENT",
    "UNREFERENCED_PARAMETER",
    "UNREFERENCED_LOCAL_VARIABLE",
    "DBG_UNREFERENCED_PARAMETER",
    "DBG_UNREFERENCED_LOCAL_VARIABLE",
    "C_ASSERT",
    "NOTHING",
    "ARGUMENT_PRESENT",
    "NT_ASSERT",
    "EXCEPTION_CONTINUE_SEARCH",
    "EXCEPTION_CONTINUE_EXECUTION",
    # Universal NT layout/const macros — never importable symbols.
    "FIELD_OFFSET",
    "RTL_NUMBER_OF",
    "RTL_NUMBER_OF_V2",
    "RTL_CONSTANT_STRING",
    "RTL_CONSTANT_OBJECT_ATTRIBUTES",
    "UNICODE_NULL",
    "ANSI_NULL",
    "RTL_FIELD_SIZE",
    "ALIGN_UP",
    "ALIGN_DOWN",
    "ALIGN_UP_BY",
    "ALIGN_DOWN_BY",
}

# Known gap functions — C macros/FORCEINLINE that won't appear in ntkrnl-sys
# These need special handling in the prompt
GAP_FUNCTIONS = {
    # Memory operations (expand to core::ptr::*)
    "RtlCopyMemory",
    "RtlMoveMemory",
    "RtlZeroMemory",
    "RtlFillMemory",
    "RtlCompareMemory",
    # Object attributes macro
    "InitializeObjectAttributes",
    # Probe macros
    "ProbeForWrite",
    "ProbeForRead",
    # String init macros
    "RtlInitUnicodeString",
    "RtlInitAnsiString",
    # Status check macros
    "NT_SUCCESS",
    "NT_ERROR",
    "NT_WARNING",
    "NT_INFORMATION",
    # Exception handling macros
    "_SEH2_TRY",
    "_SEH2_EXCEPT",
    "_SEH2_END",
    "_SEH2_YIELD",
    "_SEH2_GetExceptionCode",
    "_SEH2_GetExceptionInformation",
    "EXCEPTION_EXECUTE_HANDLER",
    # Debug macros
    "PAGED_CODE",
    "DPRINT",
    "DPRINT1",
    "ASSERT",
    # List macros
    "InitializeListHead",
    "InsertTailList",
    "InsertHeadList",
    "RemoveEntryList",
    "RemoveHeadList",
    "RemoveTailList",
    "IsListEmpty",
    # Object header macro
    "OBJECT_TO_OBJECT_HEADER",
    "CONTAINING_RECORD",
    # Time
    "KeQuerySystemTime",
    "KeQueryPerformanceCounter",
    # Interlocked (may or may not be in catalog)
    "InterlockedIncrement",
    "InterlockedDecrement",
    "InterlockedExchange",
    "InterlockedCompareExchange",
    "InterlockedExchangeAdd",
    "InterlockedExchangeAdd64",
    "InterlockedCompareExchangePointer",
    # Mutex/sync
    "ExInitializeFastMutex",
    "ExAcquireFastMutex",
    "ExReleaseFastMutex",
    # Previous mode
    "ExGetPreviousMode",
    "KeGetPreviousMode",
    # Security
    "SeCaptureSubjectContext",
    "SeReleaseSubjectContext",
    "SeQuerySubjectContextToken",
    "SeQueryAuthenticationIdToken",
    # Security descriptor helpers (FORCEINLINE in se.h)
    "SepGetOwnerFromDescriptor",
    "SepGetGroupFromDescriptor",
    "SepGetDaclFromDescriptor",
    "SepGetSaclFromDescriptor",
    # Token lock macros (se.h)
    "SepAcquireTokenLockExclusive",
    "SepAcquireTokenLockShared",
    "SepReleaseTokenLock",
    # Probe helpers (FORCEINLINE in probe.h)
    "DefaultQueryInfoBufferCheck",
    "DefaultSetInfoBufferCheck",
    # Luid
    "RtlEqualLuid",
    "SYSTEM_LUID",
    "ANONYMOUS_LOGON_LUID",
    # Status raising
    "RtlRaiseStatus",
    # Resource lock (FORCEINLINE/macro in ex.h)
    "ExAcquireResourceExclusiveLite",
    "ExAcquireResourceSharedLite",
    "ExReleaseResourceLite",
    "ExInitializeResourceLite",
    "ExDeleteResourceLite",
    # Push lock (FORCEINLINE in ex.h)
    "ExAcquirePushLockExclusive",
    "ExAcquirePushLockShared",
    "ExReleasePushLock",
    "ExReleasePushLockExclusive",
    "ExInitializePushLock",
    # Misc inline/macro
    "PsGetCurrentThread",
    "PsGetCurrentProcess",
    "KeGetCurrentPrcb",
    "KeGetCurrentIrql",
    "RtlInitEmptyUnicodeString",
    "ObDereferenceObject",
    "ObReferenceObject",
    # User-mode native: FORCEINLINE current-object accessors (winnt.h /
    # ntddk.h) — never emitted as a bindgen extern, must be a gap.
    "NtCurrentPeb",
    "NtCurrentTeb",
    "NtCurrentProcess",
    "NtCurrentThread",
    "NtCurrentSession",
    "ZwCurrentProcess",
    "ZwCurrentThread",
    "RtlProcessHeap",
    "NtGetCurrentProcessId",
}


@dataclass
class ImportGroup:
    """A group of symbols from the same source module."""

    module: str  # "base", "ke", "mm", "ddk", etc.
    symbols: list[Symbol] = field(default_factory=list)


@dataclass
class ResolveResult:
    """Dependency resolution result for a C file."""

    imports: dict[str, ImportGroup] = field(default_factory=dict)  # module → ImportGroup
    gap_functions: set[str] = field(default_factory=set)  # used but not in catalog
    unresolved: set[str] = field(default_factory=set)  # not in catalog, not a gap, not builtin
    exports: dict[str, SpecEntry] = field(
        default_factory=dict
    )  # functions exported via ntoskrnl.spec


def resolve(scan: ScanResult, catalog: Catalog, spec: dict[str, SpecEntry]) -> ResolveResult:
    """Resolve a scanned file's identifiers against the catalog."""
    result = ResolveResult()

    # Find which functions are exported
    for func in scan.functions:
        if func.name in spec:
            result.exports[func.name] = spec[func.name]

    # Identifiers that are defined *in this file* (function names and their
    # parameters) are never FFI imports — for a translation crate they
    # become Rust items in the output crate itself, not extern decls.
    # Strictly correct for every target (this is what catches the
    # intra-crate csrsrv-internal pattern: CsrLoadServerDll &c. are
    # defined in csrsrv, not imported from ntdll).
    same_file_names: set[str] = {f.name for f in scan.functions}
    for f in scan.functions:
        for _ptype, pname in f.params:
            if pname:
                same_file_names.add(pname)

    # Resolve identifiers
    for ident in scan.identifiers:
        if ident in BUILTIN_NAMES:
            continue

        # Check catalog
        sym = catalog.lookup(ident)
        if sym:
            if sym.module not in result.imports:
                result.imports[sym.module] = ImportGroup(module=sym.module)
            # Avoid duplicates
            if not any(s.name == sym.name for s in result.imports[sym.module].symbols):
                result.imports[sym.module].symbols.append(sym)
            continue

        # Check gap functions
        if ident in GAP_FUNCTIONS:
            result.gap_functions.add(ident)
            continue

        # Check defines from the file itself
        if ident in scan.defines:
            continue

        # Defined in this file (function name or parameter) — not an import
        if ident in same_file_names:
            continue

        # Local variable / parameter declarator in this file — not an
        # import (placed after catalog lookup so a real imported symbol
        # is never suppressed; a true local shadows it in C anyway).
        if ident in scan.local_names:
            continue

        # Remaining — unresolved
        # Filter out very short names (likely local variables)
        if len(ident) > 2 and ident[0].isupper():
            result.unresolved.add(ident)

    return result


def print_resolve_result(result: ResolveResult) -> None:
    """Print resolution results."""
    print("Imports:")
    for module, group in sorted(result.imports.items()):
        names = sorted(s.name for s in group.symbols)
        print(f"  {module} ({len(names)}):")
        for name in names[:20]:
            print(f"    {name}")
        if len(names) > 20:
            print(f"    ... and {len(names) - 20} more")

    if result.exports:
        print(f"\nExports ({len(result.exports)}):")
        for name, entry in sorted(result.exports.items()):
            fwd = f" → {entry.forward_target}" if entry.forward_target else ""
            arch = f" [{','.join(entry.arch_filter)}]" if entry.arch_filter else ""
            print(f"  {entry.convention} {name}({' '.join(entry.params)}){fwd}{arch}")

    if result.gap_functions:
        print(f"\nGap functions ({len(result.gap_functions)}):")
        for name in sorted(result.gap_functions):
            print(f"  {name}")

    if result.unresolved:
        print(f"\nUnresolved ({len(result.unresolved)}):")
        for name in sorted(result.unresolved)[:30]:
            print(f"  {name}")
        if len(result.unresolved) > 30:
            print(f"  ... and {len(result.unresolved) - 30} more")
