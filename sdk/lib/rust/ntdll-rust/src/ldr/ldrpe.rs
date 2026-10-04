// This file mirrors ldrpe.c's manipulation of file-scope mutable globals and
// of C-side `static mut` loader globals (LdrApiDefaultExtension etc.); reads
// and writes through `static mut` are expected and intentional.
#![allow(static_mut_refs)]

use ntdll_sys::nt::*;
use core::mem::offset_of;

// ---------------------------------------------------------------------------
// ntimage.h PE-format macros rendered as inline fns (rustconv mis-emitted
// these as extern "C" fns — they have no symbol and must be expanded inline).
// ---------------------------------------------------------------------------
#[cfg(target_arch = "x86")]
const IMAGE_ORDINAL_FLAG: ULONG_PTR = 0x8000_0000;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
const IMAGE_ORDINAL_FLAG: ULONG_PTR = 0x8000_0000_0000_0000;

#[inline]
unsafe fn IMAGE_SNAP_BY_ORDINAL(ordinal: ULONG_PTR) -> BOOLEAN {
    ((ordinal & IMAGE_ORDINAL_FLAG) != 0) as BOOLEAN
}
#[inline]
unsafe fn IMAGE_ORDINAL(ordinal: ULONG_PTR) -> ULONG_PTR {
    ordinal & 0xffff
}
#[inline]
unsafe fn IMAGE_FIRST_SECTION(nt: PIMAGE_NT_HEADERS) -> PIMAGE_SECTION_HEADER {
    // (ULONG_PTR)nt + FIELD_OFFSET(IMAGE_NT_HEADERS, OptionalHeader)
    //            + FileHeader.SizeOfOptionalHeader
    ((nt as ULONG_PTR)
        + offset_of!(IMAGE_NT_HEADERS, OptionalHeader) as ULONG_PTR
        + (*nt).FileHeader.SizeOfOptionalHeader as ULONG_PTR) as PIMAGE_SECTION_HEADER
}

extern "system" {
    fn LdrpApplyFileNameRedirection( OriginalName: *mut _UNICODE_STRING, Extension: *mut _UNICODE_STRING, StaticString: *mut _UNICODE_STRING, DynamicString: *mut _UNICODE_STRING, NewName: *mut *mut _UNICODE_STRING, Redirected: *mut BOOLEAN, ) -> NTSTATUS;
    fn LdrpCheckForLoadedDllHandle(Base: PVOID, Entry: *mut PLDR_DATA_TABLE_ENTRY) -> BOOLEAN;
    fn LdrpCheckForLoadedDll( DllPath: PWSTR, DllName: *mut _UNICODE_STRING, Flag: BOOLEAN, RedirectedDll: BOOLEAN, LdrEntry: *mut PLDR_DATA_TABLE_ENTRY, ) -> BOOLEAN;
    fn LdrpMapDll( DllPath: PWSTR, DllName2: PWSTR, DllName: PWSTR, DllCharacteristics: PULONG, Static: BOOLEAN, RedirectedDll: BOOLEAN, DataTableEntry: *mut PLDR_DATA_TABLE_ENTRY, ) -> NTSTATUS;
    fn LdrpGetProcedureAddress( BaseAddress: PVOID, Name: PANSI_STRING, Ordinal: ULONG, ProcedureAddress: *mut PVOID, Execute: BOOLEAN, ) -> NTSTATUS;
    fn LdrpLoadDll( Redirected: BOOLEAN, DllPath: PWSTR, DllCharacteristics: PULONG, DllName: *mut _UNICODE_STRING, BaseAddress: *mut PVOID, CallInit: BOOLEAN, ) -> NTSTATUS;
    fn NtFlushInstructionCache( ProcessHandle: HANDLE, BaseAddress: PVOID, NumberOfBytesToFlush: SIZE_T, ) -> NTSTATUS;
    // Application Verifier notifications — defined in Rust (ldr/verifier.rs).
    fn AVrfPageHeapDllNotification(LdrEntry: PLDR_DATA_TABLE_ENTRY);
    fn AVrfDllLoadNotification(LdrEntry: PLDR_DATA_TABLE_ENTRY);
    // C-side loader globals (ldrinit.c / ldrutils.c).
    static mut LdrApiDefaultExtension: _UNICODE_STRING;
    static mut LdrpInLdrInit: BOOLEAN;
    static mut LdrpFatalHardErrorCount: ULONG;
}

// C runtime string helpers are cdecl, NOT NTAPI — must be extern "C" so i386
// gets the `_name` (cdecl) decoration, not stdcall `_name@N`.
extern "C" {
    fn strchr(s: LPSTR, c: i32) -> LPSTR;
    fn strcmp(s1: *const i8, s2: *const i8) -> i32;
    fn _stricmp(s1: LPSTR, s2: LPSTR) -> i32;
}

// ---------------------------------------------------------------------------
// NTSTATUS helper + ntstatus.h / dpfilter.h constants absent from the
// ntdll-sys prelude (verifier.rs defines its STATUS_* the same way).
// ---------------------------------------------------------------------------
#[inline]
fn NT_SUCCESS(status: NTSTATUS) -> bool {
    status >= 0
}

#[inline]
unsafe fn RtlZeroMemory(dest: PVOID, length: ULONG) {
    core::ptr::write_bytes(dest as *mut u8, 0, length as usize);
}

const STATUS_INVALID_IMAGE_FORMAT: NTSTATUS = 0xC000_007Bu32 as NTSTATUS;
const STATUS_RESOURCE_DATA_NOT_FOUND: NTSTATUS = 0xC000_0089u32 as NTSTATUS;
const STATUS_RESOURCE_TYPE_NOT_FOUND: NTSTATUS = 0xC000_008Au32 as NTSTATUS;
const STATUS_RESOURCE_NAME_NOT_FOUND: NTSTATUS = 0xC000_008Bu32 as NTSTATUS;
const STATUS_NAME_TOO_LONG: NTSTATUS = 0xC000_0106u32 as NTSTATUS;
const STATUS_ORDINAL_NOT_FOUND: NTSTATUS = 0xC000_0138u32 as NTSTATUS;
const STATUS_ENTRYPOINT_NOT_FOUND: NTSTATUS = 0xC000_0139u32 as NTSTATUS;
const STATUS_RESOURCE_LANG_NOT_FOUND: NTSTATUS = 0xC000_0204u32 as NTSTATUS;
const DPFLTR_SXS_ID: ULONG = 51;
const DPFLTR_LDR_ID: ULONG = 85;

/// `RtlInitEmptyUnicodeString` — FORCEINLINE in rtlfuncs.h (no exported symbol).
#[inline(always)]
unsafe fn RtlInitEmptyUnicodeString(s: *mut UNICODE_STRING, buffer: PWSTR, buffer_size: USHORT) {
    (*s).Length = 0;
    (*s).MaximumLength = buffer_size;
    (*s).Buffer = buffer;
}

// ---- file-scope preamble (translated) ----
// #[no_mangle] so the C TUs that still reference these ldrpe.c globals
// (ldrapi.c's LdrSetDllManifestProber, etc.) resolve against the Rust defn.
#[no_mangle]
pub static mut LdrpManifestProberRoutine: PLDR_MANIFEST_PROBER_ROUTINE = None;
#[no_mangle]
pub static mut LdrpNormalSnap: ULONG = 0;

#[no_mangle]
pub unsafe extern "system" fn LdrpSnapIAT(
    ExportLdrEntry: PLDR_DATA_TABLE_ENTRY,
    ImportLdrEntry: PLDR_DATA_TABLE_ENTRY,
    IatEntry: PIMAGE_IMPORT_DESCRIPTOR,
    EntriesValid: BOOLEAN,
) -> NTSTATUS {

    let mut Iat: PVOID;
    let mut Status: NTSTATUS;
    let mut OriginalThunk: PIMAGE_THUNK_DATA;
    let mut FirstThunk: PIMAGE_THUNK_DATA;
    let mut NtHeader: PIMAGE_NT_HEADERS;
    let mut SectionHeader: PIMAGE_SECTION_HEADER;
    let ExportDirectory: PIMAGE_EXPORT_DIRECTORY;
    let ImportName: LPSTR;
    let mut ForwarderChain: ULONG;
    let mut i: ULONG;
    let Rva: ULONG;
    let mut OldProtect: ULONG = 0;
    let mut IatSize: ULONG = 0;
    let mut ExportSize: ULONG = 0;
    let mut ImportSize: SIZE_T;

    /* Get export directory */
    ExportDirectory = RtlImageDirectoryEntryToData(
        (*ExportLdrEntry).DllBase,
        1u8,
        IMAGE_DIRECTORY_ENTRY_EXPORT as USHORT,
        &mut ExportSize,
    ) as PIMAGE_EXPORT_DIRECTORY;

    if ExportDirectory.is_null() {
        DbgPrint(
            b"LDR: %wZ doesn't contain an EXPORT table\n\0".as_ptr() as PCHAR,
            core::ptr::addr_of_mut!((*ExportLdrEntry).BaseDllName),
        );
        return STATUS_INVALID_IMAGE_FORMAT;
    }

    /* Get the IAT */
    Iat = RtlImageDirectoryEntryToData(
        (*ImportLdrEntry).DllBase,
        1u8,
        IMAGE_DIRECTORY_ENTRY_IAT as USHORT,
        &mut IatSize,
    );
    ImportSize = IatSize as SIZE_T;

    if Iat.is_null() {
        NtHeader = RtlImageNtHeader((*ImportLdrEntry).DllBase);
        if NtHeader.is_null() {
            return STATUS_INVALID_IMAGE_FORMAT;
        }
        SectionHeader = IMAGE_FIRST_SECTION(NtHeader);

        Rva = (*NtHeader).OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_IMPORT as usize].VirtualAddress;

        if Rva != 0 {
            i = 0;
            while i < (*NtHeader).FileHeader.NumberOfSections as ULONG {
                if Rva >= (*SectionHeader).VirtualAddress
                    && Rva < ((*SectionHeader).VirtualAddress + (*SectionHeader).SizeOfRawData)
                {
                    Iat = ((*ImportLdrEntry).DllBase as ULONG_PTR + (*SectionHeader).VirtualAddress as ULONG_PTR) as PVOID;

                    IatSize = (*SectionHeader).Misc.VirtualSize;

                    if IatSize == 0 {
                        IatSize = (*SectionHeader).SizeOfRawData;
                    }

                    break;
                }

                SectionHeader = SectionHeader.add(1);
                i += 1;
            }
        }

        if Iat.is_null() {
            DbgPrint(
                b"LDR: Unable to unprotect IAT for %wZ (Image Base %p)\n\0".as_ptr() as PCHAR,
                core::ptr::addr_of_mut!((*ImportLdrEntry).BaseDllName),
                (*ImportLdrEntry).DllBase,
            );
            return STATUS_INVALID_IMAGE_FORMAT;
        }

        ImportSize = IatSize as SIZE_T;
    }

    /* Unprotect the IAT */
    Status = NtProtectVirtualMemory(
        NtCurrentProcess(),
        &mut Iat,
        &mut ImportSize,
        PAGE_READWRITE,
        &mut OldProtect,
    );
    if !NT_SUCCESS(Status) {
        DbgPrint(
            b"LDR: Unable to unprotect IAT for %wZ (Status %x)\n\0".as_ptr() as PCHAR,
            core::ptr::addr_of_mut!((*ImportLdrEntry).BaseDllName),
            Status,
        );
        return Status;
    }

    if EntriesValid != 0 {
        ImportName = ((*ImportLdrEntry).DllBase as ULONG_PTR + (*IatEntry).Name as ULONG_PTR) as LPSTR;

        ForwarderChain = (*IatEntry).ForwarderChain;

        while ForwarderChain != ULONG::MAX {
            OriginalThunk = ((*ImportLdrEntry).DllBase as ULONG_PTR
                + (*IatEntry).__bindgen_anon_1.OriginalFirstThunk as ULONG_PTR
                + (ForwarderChain as ULONG_PTR) * (core::mem::size_of::<IMAGE_THUNK_DATA>() as ULONG_PTR))
                as PIMAGE_THUNK_DATA;

            FirstThunk = ((*ImportLdrEntry).DllBase as ULONG_PTR
                + (*IatEntry).FirstThunk as ULONG_PTR
                + (ForwarderChain as ULONG_PTR) * (core::mem::size_of::<IMAGE_THUNK_DATA>() as ULONG_PTR))
                as PIMAGE_THUNK_DATA;

            ForwarderChain = (*FirstThunk).u1.Ordinal as ULONG;

            Status = LdrpSnapThunk(
                (*ExportLdrEntry).DllBase,
                (*ImportLdrEntry).DllBase,
                OriginalThunk,
                FirstThunk,
                ExportDirectory,
                ExportSize,
                1u8,
                ImportName,
            );
            FirstThunk = FirstThunk.add(1);

            if !NT_SUCCESS(Status) {
                // ARM64: tolerate missing exports, keep snapping (matches the
                // #ifdef _M_ARM64 branch in ldrpe.c).
                if cfg!(target_arch = "aarch64") {
                    Status = STATUS_SUCCESS;
                } else {
                    break;
                }
            }
        }
    } else if (*IatEntry).FirstThunk != 0 {
        FirstThunk = ((*ImportLdrEntry).DllBase as ULONG_PTR + (*IatEntry).FirstThunk as ULONG_PTR) as PIMAGE_THUNK_DATA;

        NtHeader = RtlImageNtHeader((*ImportLdrEntry).DllBase);

        if (*IatEntry).__bindgen_anon_1.Characteristics < (*NtHeader).OptionalHeader.SizeOfHeaders
            || (*IatEntry).__bindgen_anon_1.Characteristics >= (*NtHeader).OptionalHeader.SizeOfImage
        {
            OriginalThunk = FirstThunk;
        } else {
            OriginalThunk = ((*ImportLdrEntry).DllBase as ULONG_PTR
                + (*IatEntry).__bindgen_anon_1.OriginalFirstThunk as ULONG_PTR)
                as PIMAGE_THUNK_DATA;
        }

        ImportName = ((*ImportLdrEntry).DllBase as ULONG_PTR + (*IatEntry).Name as ULONG_PTR) as LPSTR;

        while (*OriginalThunk).u1.AddressOfData != 0 {
            Status = LdrpSnapThunk(
                (*ExportLdrEntry).DllBase,
                (*ImportLdrEntry).DllBase,
                OriginalThunk,
                FirstThunk,
                ExportDirectory,
                ExportSize,
                1u8,
                ImportName,
            );
            OriginalThunk = OriginalThunk.add(1);
            FirstThunk = FirstThunk.add(1);

            if !NT_SUCCESS(Status) {
                // ARM64: tolerate missing exports, continue snapping (matches
                // the #ifdef _M_ARM64 branch in ldrpe.c).
                if cfg!(target_arch = "aarch64") {
                    Status = STATUS_SUCCESS;
                    continue;
                } else {
                    break;
                }
            }
        }
    }

    /* Protect the IAT again */
    NtProtectVirtualMemory(
        NtCurrentProcess(),
        &mut Iat,
        &mut ImportSize,
        OldProtect,
        &mut OldProtect,
    );

    NtFlushInstructionCache(NtCurrentProcess(), Iat, IatSize as SIZE_T);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LdrpHandleOneNewFormatImportDescriptor(
    DllPath: LPWSTR,
    LdrEntry: PLDR_DATA_TABLE_ENTRY,
    BoundEntryPtr: *mut PIMAGE_BOUND_IMPORT_DESCRIPTOR,
    mut FirstEntry: PIMAGE_BOUND_IMPORT_DESCRIPTOR,
) -> NTSTATUS {

    let mut ImportName: LPSTR = core::ptr::null_mut();
    let BoundImportName: LPSTR;
    let mut ForwarderName: LPSTR;
    let mut Status: NTSTATUS;
    let mut AlreadyLoaded: BOOLEAN = 0u8;
    let mut Stale: BOOLEAN;
    let mut ImportEntry: PIMAGE_IMPORT_DESCRIPTOR;
    let mut DllLdrEntry: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
    let mut ForwarderLdrEntry: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
    let mut ForwarderEntry: PIMAGE_BOUND_FORWARDER_REF;
    let BoundEntry: PIMAGE_BOUND_IMPORT_DESCRIPTOR;
    let Peb: PPEB = (*NtCurrentTeb()).ProcessEnvironmentBlock;
    let mut i: ULONG;
    let mut IatSize: ULONG = 0;

    BoundEntry = *BoundEntryPtr;

    BoundImportName = (FirstEntry as *mut u8).add((*BoundEntry).OffsetModuleName as usize) as LPSTR;

    'quickie: {
        Status = LdrpLoadImportModule(
            DllPath,
            BoundImportName,
            &mut DllLdrEntry,
            &mut AlreadyLoaded,
        );
        if !NT_SUCCESS(Status) {
            // ARM64 early boot: tolerate missing bound import (matches the
            // #ifdef _M_ARM64 branch in ldrpe.c — set SUCCESS, goto Quickie).
            if cfg!(target_arch = "aarch64") {
                Status = STATUS_SUCCESS;
            }
            break 'quickie;
        }

        if AlreadyLoaded == 0 {
            InsertTailList(
                core::ptr::addr_of_mut!((*(*Peb).Ldr).InInitializationOrderModuleList),
                core::ptr::addr_of_mut!((*DllLdrEntry).InInitializationOrderLinks),
            );
        }

        if (*BoundEntry).TimeDateStamp != (*DllLdrEntry).__bindgen_anon_2.TimeDateStamp
            || ((*DllLdrEntry).Flags & LDRP_IMAGE_NOT_AT_BASE) != 0
        {
            Stale = 1u8;
        } else {
            Stale = 0u8;
        }

        ForwarderEntry = BoundEntry.add(1) as PIMAGE_BOUND_FORWARDER_REF;

        i = 0;
        while i < (*BoundEntry).NumberOfModuleForwarderRefs as ULONG {
            ForwarderName = (FirstEntry as *mut u8)
                .add((*ForwarderEntry).OffsetModuleName as usize)
                as LPSTR;

            Status = LdrpLoadImportModule(
                DllPath,
                ForwarderName,
                &mut ForwarderLdrEntry,
                &mut AlreadyLoaded,
            );
            if NT_SUCCESS(Status) {
                if AlreadyLoaded == 0 {
                    InsertTailList(
                        core::ptr::addr_of_mut!((*(*Peb).Ldr).InInitializationOrderModuleList),
                        core::ptr::addr_of_mut!((*ForwarderLdrEntry).InInitializationOrderLinks),
                    );
                }
            }

            if !NT_SUCCESS(Status)
                || (*ForwarderEntry).TimeDateStamp != (*ForwarderLdrEntry).__bindgen_anon_2.TimeDateStamp
                || ((*ForwarderLdrEntry).Flags & LDRP_IMAGE_NOT_AT_BASE) != 0
            {
                Stale = 1u8;
            } else {
                Stale = 0u8;
            }

            ForwarderEntry = ForwarderEntry.add(1);
            i += 1;
        }

        FirstEntry = ForwarderEntry as PIMAGE_BOUND_IMPORT_DESCRIPTOR;

        if Stale != 0 {
            LdrpNormalSnap += 1;
            ImportEntry = RtlImageDirectoryEntryToData(
                (*LdrEntry).DllBase,
                1u8,
                IMAGE_DIRECTORY_ENTRY_IMPORT as u16,
                &mut IatSize,
            ) as PIMAGE_IMPORT_DESCRIPTOR;

            while (*ImportEntry).Name != 0 {
                ImportName = ((*LdrEntry).DllBase as usize + (*ImportEntry).Name as usize) as LPSTR;

                if _stricmp(ImportName, BoundImportName) == 0 {
                    break;
                }

                ImportEntry = ImportEntry.add(1);
            }

            if (*ImportEntry).Name == 0 {
                Status = STATUS_OBJECT_NAME_INVALID;
                break 'quickie;
            }

            Status = LdrpSnapIAT(DllLdrEntry, LdrEntry, ImportEntry, 0u8);

            if !NT_SUCCESS(Status) {
                break 'quickie;
            }
        }

        Status = STATUS_SUCCESS;
    }

    *BoundEntryPtr = FirstEntry;
    Status
}

#[no_mangle]
pub unsafe extern "system" fn LdrpHandleNewFormatImportDescriptors(
    DllPath: LPWSTR,
    LdrEntry: PLDR_DATA_TABLE_ENTRY,
    BoundEntry: PIMAGE_BOUND_IMPORT_DESCRIPTOR,
) -> NTSTATUS {
    let FirstEntry: PIMAGE_BOUND_IMPORT_DESCRIPTOR = BoundEntry;
    let mut BoundEntry = BoundEntry;
    let mut Status: NTSTATUS;

    while (*BoundEntry).OffsetModuleName != 0 {
        Status = LdrpHandleOneNewFormatImportDescriptor(
            DllPath,
            LdrEntry,
            &mut BoundEntry,
            FirstEntry,
        );
        if !NT_SUCCESS(Status) {
            return Status;
        }
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn LdrpHandleOneOldFormatImportDescriptor(
    DllPath: LPWSTR,
    LdrEntry: PLDR_DATA_TABLE_ENTRY,
    ImportEntry: *mut PIMAGE_IMPORT_DESCRIPTOR,
) -> NTSTATUS {

    let mut AlreadyLoaded: BOOLEAN = 0u8;
    let mut DllLdrEntry: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    let Peb: PPEB = NtCurrentPeb();

    let ImportName: LPSTR =
        ((*LdrEntry).DllBase as ULONG_PTR + (**ImportEntry).Name as ULONG_PTR) as LPSTR;

    let FirstThunk: PIMAGE_THUNK_DATA =
        ((*LdrEntry).DllBase as ULONG_PTR + (**ImportEntry).FirstThunk as ULONG_PTR)
            as PIMAGE_THUNK_DATA;

    'skip: {
        if (*FirstThunk).u1.Function == 0 {
            break 'skip;
        }

        Status = LdrpLoadImportModule(
            DllPath,
            ImportName,
            &mut DllLdrEntry,
            &mut AlreadyLoaded,
        );
        if !NT_SUCCESS(Status) {
            DbgPrint(
                b"LDR: Skipping missing import %s for %wZ (status %x)\n\0".as_ptr() as PCHAR,
                ImportName,
                core::ptr::addr_of_mut!((*LdrEntry).BaseDllName),
                Status,
            );
            break 'skip;
        }

        LdrpNormalSnap += 1;
        if AlreadyLoaded == 0 {
            InsertTailList(
                core::ptr::addr_of_mut!((*(*Peb).Ldr).InInitializationOrderModuleList),
                core::ptr::addr_of_mut!((*DllLdrEntry).InInitializationOrderLinks),
            );
        }

        Status = LdrpSnapIAT(DllLdrEntry, LdrEntry, *ImportEntry, 0u8);
        if !NT_SUCCESS(Status) {
            return Status;
        }
    }

    *ImportEntry = (*ImportEntry).offset(1);
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn LdrpHandleOldFormatImportDescriptors(
    DllPath: LPWSTR,
    LdrEntry: PLDR_DATA_TABLE_ENTRY,
    mut ImportEntry: PIMAGE_IMPORT_DESCRIPTOR,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    while (*ImportEntry).Name != 0 && (*ImportEntry).FirstThunk != 0 {
        Status = LdrpHandleOneOldFormatImportDescriptor(
            DllPath,
            LdrEntry,
            &mut ImportEntry,
        );
        if !NT_SUCCESS(Status) {
            return Status;
        }
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn LdrpNameToOrdinal(
    ImportName: LPSTR,
    NumberOfNames: ULONG,
    ExportBase: PVOID,
    NameTable: PULONG,
    OrdinalTable: PUSHORT,
) -> USHORT {
    let mut Start: LONG;
    let mut End: LONG;
    let mut Next: LONG;
    let mut CmpResult: LONG;

    Start = 0;
    Next = 0;
    End = NumberOfNames as LONG - 1;
    while End >= Start {
        Next = (Start + End) >> 1;

        CmpResult = strcmp(
            ImportName as *const _,
            (ExportBase as ULONG_PTR + *NameTable.offset(Next as isize) as ULONG_PTR) as *const _,
        );

        if CmpResult == 0 {
            break;
        }

        if CmpResult < 0 {
            End = Next - 1;
        } else if CmpResult > 0 {
            Start = Next + 1;
        }
    }

    if End < Start {
        return -1i32 as USHORT;
    }

    *OrdinalTable.offset(Next as isize)
}

#[no_mangle]
pub unsafe extern "system" fn LdrpWalkImportDescriptor(
    DllPath: LPWSTR,
    LdrEntry: PLDR_DATA_TABLE_ENTRY,
) -> NTSTATUS {
    let mut ActCtx: RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED =
        core::mem::zeroed();
    let Peb: PPEB = (*NtCurrentTeb()).ProcessEnvironmentBlock;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let Status2: NTSTATUS;
    let mut BoundEntry: PIMAGE_BOUND_IMPORT_DESCRIPTOR = core::ptr::null_mut();
    let ImportEntry: PIMAGE_IMPORT_DESCRIPTOR;
    let mut BoundSize: ULONG = 0;
    let mut IatSize: ULONG = 0;

    RtlZeroMemory(
        core::ptr::addr_of_mut!(ActCtx) as *mut _,
        core::mem::size_of::<RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED>() as ULONG,
    );
    ActCtx.Size = core::mem::size_of::<RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED>() as _;
    ActCtx.Format = RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_FORMAT_WHISTLER;

    if let Some(prober) = LdrpManifestProberRoutine {
        Status2 = prober(
            (*LdrEntry).DllBase,
            (*LdrEntry).FullDllName.Buffer,
            core::ptr::addr_of_mut!((*LdrEntry).EntryPointActivationContext),
        );

        if !NT_SUCCESS(Status2)
            && Status2 != STATUS_NO_SUCH_FILE
            && Status2 != STATUS_RESOURCE_DATA_NOT_FOUND
            && Status2 != STATUS_RESOURCE_TYPE_NOT_FOUND
            && Status2 != STATUS_RESOURCE_NAME_NOT_FOUND
            && Status2 != STATUS_RESOURCE_LANG_NOT_FOUND
        {
            DbgPrintEx(
                DPFLTR_SXS_ID,
                DPFLTR_WARNING_LEVEL,
                b"LDR: LdrpWalkImportDescriptor() failed to probe %wZ for its manifest, ntstatus = 0x%08lx\n\0".as_ptr() as *const i8,
                core::ptr::addr_of!((*LdrEntry).FullDllName),
                Status2,
            );
        }
    }

    if !NT_SUCCESS(Status) {
        return Status;
    }

    if (*LdrEntry).EntryPointActivationContext.is_null() {
        Status = RtlGetActiveActivationContext(core::ptr::addr_of_mut!(
            (*LdrEntry).EntryPointActivationContext
        ));

        if !NT_SUCCESS(Status) {
            DbgPrintEx(
                DPFLTR_SXS_ID,
                DPFLTR_WARNING_LEVEL,
                b"LDR: RtlGetActiveActivationContext() failed; ntstatus = 0x%08lx\n\0".as_ptr() as *const i8,
                Status,
            );
            return Status;
        }
    }

    RtlActivateActivationContextUnsafeFast(
        &mut ActCtx,
        (*LdrEntry).EntryPointActivationContext,
    );

    if ((*LdrEntry).Flags & LDRP_REDIRECTED) == 0 {
        BoundEntry = RtlImageDirectoryEntryToData(
            (*LdrEntry).DllBase,
            1u8,
            IMAGE_DIRECTORY_ENTRY_BOUND_IMPORT as u16,
            &mut BoundSize,
        ) as PIMAGE_BOUND_IMPORT_DESCRIPTOR;
    }

    ImportEntry = RtlImageDirectoryEntryToData(
        (*LdrEntry).DllBase,
        1u8,
        IMAGE_DIRECTORY_ENTRY_IMPORT as u16,
        &mut IatSize,
    ) as PIMAGE_IMPORT_DESCRIPTOR;

    if !BoundEntry.is_null() || !ImportEntry.is_null() {
        if !BoundEntry.is_null() {
            Status = LdrpHandleNewFormatImportDescriptors(DllPath, LdrEntry, BoundEntry);
        } else {
            Status = LdrpHandleOldFormatImportDescriptors(DllPath, LdrEntry, ImportEntry);
        }

        if NT_SUCCESS(Status) {
            if ((*Peb).NtGlobalFlag & FLG_HEAP_ENABLE_TAG_BY_DLL) != 0 {
                // FIXME: Per-DLL Heap Tagging not supported
            }

            if ((*Peb).NtGlobalFlag & FLG_HEAP_PAGE_ALLOCS) != 0 {
                AVrfPageHeapDllNotification(LdrEntry);
            }

            if ((*Peb).NtGlobalFlag & FLG_APPLICATION_VERIFIER) != 0 {
                AVrfDllLoadNotification(LdrEntry);
            }

            Status = STATUS_SUCCESS;
        }
    }

    RtlDeactivateActivationContextUnsafeFast(&mut ActCtx);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LdrpSnapThunk(
    ExportBase: PVOID,
    ImportBase: PVOID,
    OriginalThunk: PIMAGE_THUNK_DATA,
    Thunk: PIMAGE_THUNK_DATA,
    ExportDirectory: PIMAGE_EXPORT_DIRECTORY,
    ExportSize: ULONG,
    Static: BOOLEAN,
    DllName: LPSTR,
) -> NTSTATUS {

    let IsOrdinal: BOOLEAN;
    let mut Ordinal: USHORT = 0;
    let mut OriginalOrdinal: ULONG = 0;
    let AddressOfData: PIMAGE_IMPORT_BY_NAME;
    let NameTable: PULONG;
    let OrdinalTable: PUSHORT;
    let mut ImportName: LPSTR = core::ptr::null_mut();
    let mut DotPosition: LPSTR;
    let mut Hint: USHORT = 0;
    let mut Status: NTSTATUS;
    let mut HardErrorParameters: [ULONG_PTR; 3] = [0; 3];
    let mut HardErrorDllName: _UNICODE_STRING = core::mem::zeroed();
    let mut HardErrorEntryPointName: _UNICODE_STRING = core::mem::zeroed();
    let mut TempString: ANSI_STRING = core::mem::zeroed();
    let mut Mask: ULONG;
    let mut Response: ULONG = 0;
    let mut AddressOfFunctions: PULONG;
    let mut TempUString: _UNICODE_STRING = core::mem::zeroed();
    let mut ForwarderName: ANSI_STRING = core::mem::zeroed();
    let mut ForwardName: PANSI_STRING;
    let mut ForwarderHandle: PVOID = core::ptr::null_mut();
    let mut ForwardOrdinal: ULONG = 0;

    IsOrdinal = IMAGE_SNAP_BY_ORDINAL((*OriginalThunk).u1.Ordinal as ULONG_PTR);
    if IsOrdinal != 0 {
        OriginalOrdinal = IMAGE_ORDINAL((*OriginalThunk).u1.Ordinal as ULONG_PTR) as ULONG;
        Ordinal = (OriginalOrdinal - (*ExportDirectory).Base) as USHORT;
    } else {
        AddressOfData = ((ImportBase as ULONG_PTR)
            + ((*OriginalThunk).u1.AddressOfData as ULONG_PTR & 0xffffffff))
            as PIMAGE_IMPORT_BY_NAME;

        ImportName = core::ptr::addr_of_mut!((*AddressOfData).Name) as LPSTR;

        NameTable = ((ExportBase as ULONG_PTR)
            + ((*ExportDirectory).AddressOfNames as ULONG_PTR))
            as PULONG;
        OrdinalTable = ((ExportBase as ULONG_PTR)
            + ((*ExportDirectory).AddressOfNameOrdinals as ULONG_PTR))
            as PUSHORT;

        Hint = (*AddressOfData).Hint;

        if (Hint as ULONG) < (*ExportDirectory).NumberOfNames
            && strcmp(
                ImportName,
                ((ExportBase as ULONG_PTR) + *NameTable.offset(Hint as isize) as ULONG_PTR) as LPSTR,
            ) == 0
        {
            Ordinal = *OrdinalTable.offset(Hint as isize);
        } else {
            Ordinal = LdrpNameToOrdinal(
                ImportName,
                (*ExportDirectory).NumberOfNames,
                ExportBase,
                NameTable,
                OrdinalTable,
            );
        }
    }

    'failure: loop {
        if (Ordinal as ULONG) >= (*ExportDirectory).NumberOfFunctions {
            // FailurePath
            // A forwarder DLL loaded on the way here must be unloaded, or it
            // lingers in the loader waiting to run its init routine (upstream
            // 4a96f375a0f "[NTDLL] Unload DLLs on loader failure"). Not on arm64,
            // where the C's own _M_ARM64 arm tolerates the failed snap below and the
            // process carries on resolving through the forwarder: unloading it there
            // unmapped a DLL csrss was still snapping and faulted in LdrpSnapThunk
            // reading its import table (smoke:arm64 hang, 2026-10-04).
            #[cfg(not(target_arch = "aarch64"))]
            if !ForwarderHandle.is_null() {
                LdrUnloadDll(ForwarderHandle);
            }

            // ARM64: DUMMY_PSEH can't catch RtlRaiseStatus. Skip the hard error
            // and fall through to the non-fatal path (Thunk = 0xffbadd11); the
            // caller tolerates it. Matches the #ifdef _M_ARM64 branch in ldrpe.c.
            if Static != 0 && !cfg!(target_arch = "aarch64") {
                let mut SnapTarget: _UNICODE_STRING = core::mem::zeroed();
                let mut LdrEntry: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();

                RtlInitAnsiString(
                    &mut TempString,
                    if !DllName.is_null() {
                        DllName
                    } else {
                        b"Unknown\0".as_ptr() as LPSTR
                    },
                );

                if LdrpCheckForLoadedDllHandle(ImportBase, &mut LdrEntry) != 0 {
                    SnapTarget = (*LdrEntry).BaseDllName;
                } else {
                    static UNKNOWN_W: [WCHAR; 8] =
                        [b'U' as WCHAR, b'n' as WCHAR, b'k' as WCHAR, b'n' as WCHAR,
                         b'o' as WCHAR, b'w' as WCHAR, b'n' as WCHAR, 0];
                    RtlInitUnicodeString(&mut SnapTarget, UNKNOWN_W.as_ptr());
                }

                RtlAnsiStringToUnicodeString(&mut HardErrorDllName, &mut TempString, 1u8);

                HardErrorParameters[1] = &mut HardErrorDllName as *mut _ as ULONG_PTR;
                Mask = 2;

                if IsOrdinal != 0 {
                    HardErrorParameters[0] = OriginalOrdinal as ULONG_PTR;
                } else {
                    RtlInitAnsiString(&mut TempString, ImportName);
                    RtlAnsiStringToUnicodeString(
                        &mut HardErrorEntryPointName,
                        &mut TempString,
                        1u8,
                    );

                    HardErrorParameters[0] = &mut HardErrorEntryPointName as *mut _ as ULONG_PTR;
                    Mask = 3;
                }

                // Name the failing snap on serial BEFORE the hard error: the
                // -smp 4 deterministic c0000139 during csrss LoadServerDll
                // (winsrv) dies without this ever reaching the log.
                if IsOrdinal != 0 {
                    DbgPrint(
                        b"LDR: SNAP FAIL ordinal %u from %wZ\n\0".as_ptr() as PCHAR,
                        OriginalOrdinal as u32,
                        core::ptr::addr_of_mut!(SnapTarget),
                    );
                } else {
                    DbgPrint(
                        b"LDR: SNAP FAIL import %s from %wZ (importer=%s thunk_rva=%p hint=%u nfuncs=%u ord=%u)\n\0".as_ptr() as PCHAR,
                        ImportName,
                        core::ptr::addr_of_mut!(SnapTarget),
                        if !DllName.is_null() { DllName } else { b"?\0".as_ptr() as LPSTR },
                        (*OriginalThunk).u1.AddressOfData as usize as *const u8,
                        Hint as u32,
                        (*ExportDirectory).NumberOfFunctions,
                        Ordinal as u32,
                    );
                }

                NtRaiseHardError(
                    if IsOrdinal != 0 {
                        STATUS_ORDINAL_NOT_FOUND
                    } else {
                        STATUS_ENTRYPOINT_NOT_FOUND
                    },
                    2,
                    Mask,
                    HardErrorParameters.as_mut_ptr(),
                    OptionOk,
                    &mut Response,
                );

                if LdrpInLdrInit != 0 {
                    LdrpFatalHardErrorCount += 1;
                }

                RtlFreeUnicodeString(&mut HardErrorDllName);
                if IsOrdinal == 0 {
                    RtlFreeUnicodeString(&mut HardErrorEntryPointName);
                    RtlRaiseStatus(STATUS_ENTRYPOINT_NOT_FOUND);
                }

                RtlRaiseStatus(STATUS_ORDINAL_NOT_FOUND);
            }

            (*Thunk).u1.Function = 0xffbadd11 as ULONG_PTR;

            Status = if IsOrdinal != 0 {
                STATUS_ORDINAL_NOT_FOUND
            } else {
                STATUS_ENTRYPOINT_NOT_FOUND
            };
            break 'failure;
        }

        AddressOfFunctions = ((ExportBase as ULONG_PTR)
            + ((*ExportDirectory).AddressOfFunctions as ULONG_PTR))
            as PULONG;

        (*Thunk).u1.Function =
            (ExportBase as ULONG_PTR) + *AddressOfFunctions.offset(Ordinal as isize) as ULONG_PTR;

        if (*Thunk).u1.Function > (ExportDirectory as ULONG_PTR)
            && (*Thunk).u1.Function < ((ExportDirectory as ULONG_PTR) + ExportSize as ULONG_PTR)
        {
            ImportName = (*Thunk).u1.Function as LPSTR;

            DotPosition = strchr(ImportName, '.' as i32);
            if DotPosition.is_null() {
                Ordinal = (*ExportDirectory).NumberOfFunctions as USHORT;
                continue 'failure;
            }

            ForwarderName.Buffer = ImportName;
            ForwarderName.Length =
                (DotPosition as ULONG_PTR - ImportName as ULONG_PTR) as USHORT;
            ForwarderName.MaximumLength = ForwarderName.Length;

            if ForwarderName.Length > 1 && *ForwarderName.Buffer == b'_' as i8 {
                ForwarderName.Buffer = ForwarderName.Buffer.offset(1);
                ForwarderName.Length -= 1;
                ForwarderName.MaximumLength -= 1;
            }

            Status = RtlAnsiStringToUnicodeString(&mut TempUString, &mut ForwarderName, 1u8);

            if NT_SUCCESS(Status) {
                let mut StringBuffer: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
                let mut StaticString: _UNICODE_STRING = core::mem::zeroed();
                let mut RedirectedImportName: *mut _UNICODE_STRING = core::ptr::null_mut();
                let mut Redirected: BOOLEAN = 0;

                RtlInitEmptyUnicodeString(
                    &mut StaticString,
                    StringBuffer.as_mut_ptr(),
                    core::mem::size_of_val(&StringBuffer) as USHORT,
                );

                Status = LdrpApplyFileNameRedirection(
                    &mut TempUString,
                    &mut LdrApiDefaultExtension,
                    &mut StaticString,
                    core::ptr::null_mut(),
                    &mut RedirectedImportName,
                    &mut Redirected,
                );

                if !(NT_SUCCESS(Status) && Redirected != 0) {
                    RedirectedImportName = &mut TempUString;
                }

                Status = LdrpLoadDll(
                    Redirected,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    RedirectedImportName,
                    &mut ForwarderHandle,
                    0u8,
                );

                RtlFreeUnicodeString(&mut TempUString);
            }

            if !NT_SUCCESS(Status) {
                Ordinal = (*ExportDirectory).NumberOfFunctions as USHORT;
                continue 'failure;
            }

            RtlInitAnsiString(
                &mut ForwarderName,
                ImportName.offset(ForwarderName.Length as isize + 1),
            );

            if ForwarderName.Length > 1 && *ForwarderName.Buffer == b'#' as i8 {
                ForwardName = core::ptr::null_mut();

                Status = RtlCharToInteger(
                    ForwarderName.Buffer.offset(1),
                    0,
                    &mut ForwardOrdinal,
                );

                if !NT_SUCCESS(Status) {
                    Ordinal = (*ExportDirectory).NumberOfFunctions as USHORT;
                    continue 'failure;
                }
            } else {
                ForwardName = &mut ForwarderName;
                ForwardOrdinal = 0;
            }

            Status = LdrpGetProcedureAddress(
                ForwarderHandle,
                ForwardName,
                ForwardOrdinal,
                core::ptr::addr_of_mut!((*Thunk).u1.Function) as *mut PVOID,
                0u8,
            );
            if !NT_SUCCESS(Status) {
                Ordinal = (*ExportDirectory).NumberOfFunctions as USHORT;
                continue 'failure;
            }
        } else {
            if *AddressOfFunctions.offset(Ordinal as isize) == 0 {
                Ordinal = (*ExportDirectory).NumberOfFunctions as USHORT;
                continue 'failure;
            }
        }

        Status = STATUS_SUCCESS;
        break 'failure;
    }

    Status
}

// NOTE: hand-translated — the rustconv stitcher dropped this function's body
// (it was generated, per the run log, but lost in stitching and left as a
// mangled extern stub). Mirrors ldrpe.c LdrpLoadImportModule verbatim.
#[no_mangle]
pub unsafe extern "system" fn LdrpLoadImportModule(
    DllPath: PWSTR,
    ImportName: LPSTR,
    DataTableEntry: *mut PLDR_DATA_TABLE_ENTRY,
    Existing: *mut BOOLEAN,
) -> NTSTATUS {
    let mut AnsiString: ANSI_STRING = core::mem::zeroed();
    let mut ImpDescName: *mut _UNICODE_STRING;
    let mut p: *const WCHAR;
    let mut GotExtension: BOOLEAN;
    let mut c: WCHAR;
    let mut Status: NTSTATUS;
    let Peb: PPEB = RtlGetCurrentPeb();
    let Teb: PTEB = NtCurrentTeb();
    let mut RedirectedImpDescName: _UNICODE_STRING = core::mem::zeroed();
    let mut RedirectedDll: BOOLEAN = 0u8;

    RtlInitEmptyUnicodeString(&mut RedirectedImpDescName, core::ptr::null_mut(), 0);

    /* Convert import descriptor name to unicode string */
    ImpDescName = core::ptr::addr_of_mut!((*Teb).StaticUnicodeString);
    RtlInitAnsiString(&mut AnsiString, ImportName);
    Status = RtlAnsiStringToUnicodeString(ImpDescName, &mut AnsiString, 0u8);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* Find the extension, if present */
    p = (*ImpDescName).Buffer.offset(
        (*ImpDescName).Length as isize / core::mem::size_of::<WCHAR>() as isize - 1,
    );
    GotExtension = 0u8;
    while p >= (*ImpDescName).Buffer {
        c = *p;
        p = p.offset(-1);
        if c == b'.' as WCHAR {
            GotExtension = 1u8;
            break;
        } else if c == b'\\' as WCHAR {
            break;
        }
    }

    /* If no extension was found, add the default extension */
    if GotExtension == 0 {
        if ((*ImpDescName).Length as ULONG
            + LdrApiDefaultExtension.Length as ULONG
            + core::mem::size_of::<WCHAR>() as ULONG)
            >= core::mem::size_of_val(&(*Teb).StaticUnicodeBuffer) as ULONG
        {
            DbgPrintEx(
                DPFLTR_LDR_ID,
                DPFLTR_ERROR_LEVEL,
                b"LDR: %s - Dll name missing extension; with extension added the name is too long\n   ImpDescName: (@ %p) \"%wZ\"\n   ImpDescName->Length: %u\n\0".as_ptr() as *const i8,
                b"LdrpLoadImportModule\0".as_ptr() as *const i8,
                ImpDescName,
                ImpDescName,
                (*ImpDescName).Length as ULONG,
            );
            return STATUS_NAME_TOO_LONG;
        }

        /* Add it. Needs to be null terminated, thus the length check above */
        RtlAppendUnicodeStringToString(ImpDescName, core::ptr::addr_of_mut!(LdrApiDefaultExtension));
    }

    'done: {
        /* Check if the SxS Assemblies specify another file */
        Status = LdrpApplyFileNameRedirection(
            ImpDescName,
            core::ptr::addr_of_mut!(LdrApiDefaultExtension),
            core::ptr::null_mut(),
            &mut RedirectedImpDescName,
            &mut ImpDescName,
            &mut RedirectedDll,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        /* Check if it's loaded */
        if LdrpCheckForLoadedDll(DllPath, ImpDescName, 1u8, RedirectedDll, DataTableEntry) != 0 {
            *Existing = 1u8;
            Status = STATUS_SUCCESS;
            break 'done;
        }

        /* We're loading it for the first time */
        *Existing = 0u8;

        /* Map it */
        Status = LdrpMapDll(
            DllPath,
            core::ptr::null_mut(),
            (*ImpDescName).Buffer,
            core::ptr::null_mut(),
            1u8,
            RedirectedDll,
            DataTableEntry,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        /* Walk its import descriptor table */
        Status = LdrpWalkImportDescriptor(DllPath, *DataTableEntry);
        if !NT_SUCCESS(Status) {
            /* Add it to the in-init-order list in case of failure */
            InsertTailList(
                core::ptr::addr_of_mut!((*(*Peb).Ldr).InInitializationOrderModuleList),
                core::ptr::addr_of_mut!((**DataTableEntry).InInitializationOrderLinks),
            );
        }
    }

    RtlFreeUnicodeString(&mut RedirectedImpDescName);
    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS kernel
//  * FILE:            lib/ntdll/ldr/ldrpe.c
//  * PURPOSE:         Loader Functions dealing low-level PE Format structures
//  * PROGRAMMERS:     Alex Ionescu (alex@relsoft.net)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntdll.h>
//
// #define NDEBUG
// #include <debug.h>
//
// #ifdef _M_ARM64
// /* ARM64: User-mode cannot access kernel UART MMIO (PA 0x09000000).
//  * Keep as no-op — use NtClose marker syscalls for tracing instead. */
// #define LdrpUartPuts(s) ((void)0)
// #endif
//
// /* GLOBALS *******************************************************************/
//
// PLDR_MANIFEST_PROBER_ROUTINE LdrpManifestProberRoutine;
// ULONG LdrpNormalSnap;
//
// /* FUNCTIONS *****************************************************************/
//
//
// NTSTATUS
// NTAPI
// LdrpSnapIAT(IN PLDR_DATA_TABLE_ENTRY ExportLdrEntry,
//             IN PLDR_DATA_TABLE_ENTRY ImportLdrEntry,
//             IN PIMAGE_IMPORT_DESCRIPTOR IatEntry,
//             IN BOOLEAN EntriesValid)
// {
//     PVOID Iat;
//     NTSTATUS Status;
//     PIMAGE_THUNK_DATA OriginalThunk, FirstThunk;
//     PIMAGE_NT_HEADERS NtHeader;
//     PIMAGE_SECTION_HEADER SectionHeader;
//     PIMAGE_EXPORT_DIRECTORY ExportDirectory;
//     LPSTR ImportName;
//     ULONG ForwarderChain, i, Rva, OldProtect, IatSize, ExportSize;
//     SIZE_T ImportSize;
//     DPRINT("LdrpSnapIAT(%wZ %wZ %p %u)\n", &ExportLdrEntry->BaseDllName, &ImportLdrEntry->BaseDllName, IatEntry, EntriesValid);
//
//     /* Get export directory */
//     ExportDirectory = RtlImageDirectoryEntryToData(ExportLdrEntry->DllBase,
//                                                    TRUE,
//                                                    IMAGE_DIRECTORY_ENTRY_EXPORT,
//                                                    &ExportSize);
//
//     /* Make sure it has one */
//     if (!ExportDirectory)
//     {
//         /* Fail */
//         DbgPrint("LDR: %wZ doesn't contain an EXPORT table\n",
//                  &ExportLdrEntry->BaseDllName);
//         return STATUS_INVALID_IMAGE_FORMAT;
//     }
//
//     /* Get the IAT */
//     Iat = RtlImageDirectoryEntryToData(ImportLdrEntry->DllBase,
//                                        TRUE,
//                                        IMAGE_DIRECTORY_ENTRY_IAT,
//                                        &IatSize);
//     ImportSize = IatSize;
//
//     /* Check if we don't have one */
//     if (!Iat)
//     {
//         /* Get the NT Header and the first section */
//         NtHeader = RtlImageNtHeader(ImportLdrEntry->DllBase);
//         if (!NtHeader) return STATUS_INVALID_IMAGE_FORMAT;
//         SectionHeader = IMAGE_FIRST_SECTION(NtHeader);
//
//         /* Get the RVA of the import directory */
//         Rva = NtHeader->OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_IMPORT].VirtualAddress;
//
//         /* Make sure we got one */
//         if (Rva)
//         {
//             /* Loop all the sections */
//             for (i = 0; i < NtHeader->FileHeader.NumberOfSections; i++)
//             {
//                 /* Check if we are inside this section */
//                 if ((Rva >= SectionHeader->VirtualAddress) &&
//                     (Rva < (SectionHeader->VirtualAddress +
//                      SectionHeader->SizeOfRawData)))
//                 {
//                     /* We are, so set the IAT here */
//                     Iat = (PVOID)((ULONG_PTR)(ImportLdrEntry->DllBase) +
//                                       SectionHeader->VirtualAddress);
//
//                     /* Set the size */
//                     IatSize = SectionHeader->Misc.VirtualSize;
//
//                     /* Deal with Watcom and other retarded compilers */
//                     if (!IatSize) IatSize = SectionHeader->SizeOfRawData;
//
//                     /* Found it, get out */
//                     break;
//                 }
//
//                 /* No match, move to the next section */
//                 SectionHeader++;
//             }
//         }
//
//         /* If we still don't have an IAT, that's bad */
//         if (!Iat)
//         {
//             /* Fail */
//             DbgPrint("LDR: Unable to unprotect IAT for %wZ (Image Base %p)\n",
//                      &ImportLdrEntry->BaseDllName,
//                      ImportLdrEntry->DllBase);
//             return STATUS_INVALID_IMAGE_FORMAT;
//         }
//
//         /* Set the right size */
//         ImportSize = IatSize;
//     }
//
//     /* Unprotect the IAT */
//     Status = NtProtectVirtualMemory(NtCurrentProcess(),
//                                     &Iat,
//                                     &ImportSize,
//                                     PAGE_READWRITE,
//                                     &OldProtect);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         DbgPrint("LDR: Unable to unprotect IAT for %wZ (Status %x)\n",
//                  &ImportLdrEntry->BaseDllName,
//                  Status);
//         return Status;
//     }
//
//     /* Check if the Thunks are already valid */
//     if (EntriesValid)
//     {
//         /* We'll only do forwarders. Get the import name */
//         ImportName = (LPSTR)((ULONG_PTR)ImportLdrEntry->DllBase + IatEntry->Name);
//
//         /* Get the list of forwarders */
//         ForwarderChain = IatEntry->ForwarderChain;
//
//         /* Loop them */
//         while (ForwarderChain != -1)
//         {
//             /* Get the cached thunk VA*/
//             OriginalThunk = (PIMAGE_THUNK_DATA)
//                             ((ULONG_PTR)ImportLdrEntry->DllBase +
//                              IatEntry->OriginalFirstThunk +
//                              (ForwarderChain * sizeof(IMAGE_THUNK_DATA)));
//
//             /* Get the first thunk */
//             FirstThunk = (PIMAGE_THUNK_DATA)
//                          ((ULONG_PTR)ImportLdrEntry->DllBase +
//                           IatEntry->FirstThunk +
//                           (ForwarderChain * sizeof(IMAGE_THUNK_DATA)));
//
//             /* Get the Forwarder from the thunk */
//             ForwarderChain = (ULONG)FirstThunk->u1.Ordinal;
//
//             /* Snap the thunk */
//             _SEH2_TRY
//             {
//                 Status = LdrpSnapThunk(ExportLdrEntry->DllBase,
//                                        ImportLdrEntry->DllBase,
//                                        OriginalThunk,
//                                        FirstThunk,
//                                        ExportDirectory,
//                                        ExportSize,
//                                        TRUE,
//                                        ImportName);
//
//                 /* Move to the next thunk */
//                 FirstThunk++;
//             } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Fail with the SEH error */
//                 Status = _SEH2_GetExceptionCode();
//             } _SEH2_END;
//
//             /* If we messed up, exit */
//             if (!NT_SUCCESS(Status))
//             {
// #ifdef _M_ARM64
//                 /* ARM64: tolerate missing exports, continue snapping */
//                 Status = STATUS_SUCCESS;
// #else
//                 break;
// #endif
//             }
//         }
//     }
//     else if (IatEntry->FirstThunk)
//     {
//         /* Full snapping. Get the First thunk */
//         FirstThunk = (PIMAGE_THUNK_DATA)
//                       ((ULONG_PTR)ImportLdrEntry->DllBase +
//                        IatEntry->FirstThunk);
//
//         /* Get the NT Header */
//         NtHeader = RtlImageNtHeader(ImportLdrEntry->DllBase);
//
//         /* Get the Original thunk VA, watch out for weird images */
//         if ((IatEntry->Characteristics < NtHeader->OptionalHeader.SizeOfHeaders) ||
//             (IatEntry->Characteristics >= NtHeader->OptionalHeader.SizeOfImage))
//         {
//             /* Refuse it, this is a strange linked file */
//             OriginalThunk = FirstThunk;
//         }
//         else
//         {
//             /* Get the address from the field and convert to VA */
//             OriginalThunk = (PIMAGE_THUNK_DATA)
//                             ((ULONG_PTR)ImportLdrEntry->DllBase +
//                              IatEntry->OriginalFirstThunk);
//         }
//
//         /* Get the Import name VA */
//         ImportName = (LPSTR)((ULONG_PTR)ImportLdrEntry->DllBase +
//                              IatEntry->Name);
//
//         /* Loop while it's valid */
//         while (OriginalThunk->u1.AddressOfData)
//         {
//             /* Snap the Thunk */
//             _SEH2_TRY
//             {
//                 Status = LdrpSnapThunk(ExportLdrEntry->DllBase,
//                                        ImportLdrEntry->DllBase,
//                                        OriginalThunk,
//                                        FirstThunk,
//                                        ExportDirectory,
//                                        ExportSize,
//                                        TRUE,
//                                        ImportName);
//
//                 /* Next thunks */
//                 OriginalThunk++;
//                 FirstThunk++;
//             } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Fail with the SEH error */
//                 Status = _SEH2_GetExceptionCode();
//             } _SEH2_END;
//
//             /* If we failed the snap, break out */
//             if (!NT_SUCCESS(Status))
//             {
// #ifdef _M_ARM64
//                 /* ARM64: tolerate missing exports, continue snapping */
//                 Status = STATUS_SUCCESS;
//                 continue;
// #else
//                 break;
// #endif
//             }
//         }
//     }
//
//     /* Protect the IAT again */
//     NtProtectVirtualMemory(NtCurrentProcess(),
//                            &Iat,
//                            &ImportSize,
//                            OldProtect,
//                            &OldProtect);
//
//     /* Also flush out the cache */
//     NtFlushInstructionCache(NtCurrentProcess(), Iat, IatSize);
//
//     /* Return to Caller */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// LdrpHandleOneNewFormatImportDescriptor(IN LPWSTR DllPath OPTIONAL,
//                                        IN PLDR_DATA_TABLE_ENTRY LdrEntry,
//                                        IN PIMAGE_BOUND_IMPORT_DESCRIPTOR *BoundEntryPtr,
//                                        IN PIMAGE_BOUND_IMPORT_DESCRIPTOR FirstEntry)
// {
//     LPSTR ImportName = NULL, BoundImportName, ForwarderName;
//     NTSTATUS Status;
//     BOOLEAN AlreadyLoaded = FALSE, Stale;
//     PIMAGE_IMPORT_DESCRIPTOR ImportEntry;
//     PLDR_DATA_TABLE_ENTRY DllLdrEntry, ForwarderLdrEntry;
//     PIMAGE_BOUND_FORWARDER_REF ForwarderEntry;
//     PIMAGE_BOUND_IMPORT_DESCRIPTOR BoundEntry;
//     PPEB Peb = NtCurrentPeb();
//     ULONG i, IatSize;
//
//     /* Get the pointer to the bound entry */
//     BoundEntry = *BoundEntryPtr;
//
//     /* Get the name's VA */
//     BoundImportName = (LPSTR)FirstEntry + BoundEntry->OffsetModuleName;
//
//     /* Show debug message */
//     if (ShowSnaps)
//     {
//         DPRINT1("LDR: %wZ bound to %s\n", &LdrEntry->BaseDllName, BoundImportName);
//     }
//
//     /* Load the module for this entry */
//     Status = LdrpLoadImportModule(DllPath,
//                                   BoundImportName,
//                                   &DllLdrEntry,
//                                   &AlreadyLoaded);
//     if (!NT_SUCCESS(Status))
//     {
// #ifdef _M_ARM64
//         /* ARM64 early boot: tolerate missing DLLs (see OldFormat handler) */
//         DPRINT1("LDR: ARM64 skipping missing bound import %s for %wZ (status %x)\n",
//                 BoundImportName, &LdrEntry->BaseDllName, Status);
//         Status = STATUS_SUCCESS;
//         goto Quickie;
// #else
//         /* Show debug message */
//         if (ShowSnaps)
//         {
//             DPRINT1("LDR: %wZ failed to load import module %s; status = %x\n",
//                     &LdrEntry->BaseDllName,
//                     BoundImportName,
//                     Status);
//         }
//         goto Quickie;
// #endif
//     }
//
//     /* Check if it wasn't already loaded */
//     if (!AlreadyLoaded)
//     {
//         /* Add it to our list */
//         InsertTailList(&Peb->Ldr->InInitializationOrderModuleList,
//                        &DllLdrEntry->InInitializationOrderLinks);
//     }
//
//     /* Check if the Bound Entry is now invalid */
//     if ((BoundEntry->TimeDateStamp != DllLdrEntry->TimeDateStamp) ||
//         (DllLdrEntry->Flags & LDRP_IMAGE_NOT_AT_BASE))
//     {
//         /* Show debug message */
//         if (ShowSnaps)
//         {
//             DPRINT1("LDR: %wZ has stale binding to %s\n",
//                     &LdrEntry->BaseDllName,
//                     BoundImportName);
//         }
//
//         /* Remember it's become stale */
//         Stale = TRUE;
//     }
//     else
//     {
//         /* Show debug message */
//         if (ShowSnaps)
//         {
//             DPRINT1("LDR: %wZ has correct binding to %s\n",
//                     &LdrEntry->BaseDllName,
//                     BoundImportName);
//         }
//
//         /* Remember it's valid */
//         Stale = FALSE;
//     }
//
//     /* Get the forwarders */
//     ForwarderEntry = (PIMAGE_BOUND_FORWARDER_REF)(BoundEntry + 1);
//
//     /* Loop them */
//     for (i = 0; i < BoundEntry->NumberOfModuleForwarderRefs; i++)
//     {
//         /* Get the name */
//         ForwarderName = (LPSTR)FirstEntry + ForwarderEntry->OffsetModuleName;
//
//         /* Show debug message */
//         if (ShowSnaps)
//         {
//             DPRINT1("LDR: %wZ bound to %s via forwarder(s) from %wZ\n",
//                     &LdrEntry->BaseDllName,
//                     ForwarderName,
//                     &DllLdrEntry->BaseDllName);
//         }
//
//         /* Load the module */
//         Status = LdrpLoadImportModule(DllPath,
//                                       ForwarderName,
//                                       &ForwarderLdrEntry,
//                                       &AlreadyLoaded);
//         if (NT_SUCCESS(Status))
//         {
//             /* Loaded it, was it already loaded? */
//             if (!AlreadyLoaded)
//             {
//                 /* Add it to our list */
//                 InsertTailList(&Peb->Ldr->InInitializationOrderModuleList,
//                                &ForwarderLdrEntry->InInitializationOrderLinks);
//             }
//         }
//
//         /* Check if the Bound Entry is now invalid */
//         if (!(NT_SUCCESS(Status)) ||
//             (ForwarderEntry->TimeDateStamp != ForwarderLdrEntry->TimeDateStamp) ||
//             (ForwarderLdrEntry->Flags & LDRP_IMAGE_NOT_AT_BASE))
//         {
//             /* Show debug message */
//             if (ShowSnaps)
//             {
//                 DPRINT1("LDR: %wZ has stale binding to %s\n",
//                         &LdrEntry->BaseDllName,
//                         ForwarderName);
//             }
//
//             /* Remember it's become stale */
//             Stale = TRUE;
//         }
//         else
//         {
//             /* Show debug message */
//             if (ShowSnaps)
//             {
//                 DPRINT1("LDR: %wZ has correct binding to %s\n",
//                         &LdrEntry->BaseDllName,
//                         ForwarderName);
//             }
//
//             /* Remember it's valid */
//             Stale = FALSE;
//         }
//
//         /* Move to the next one */
//         ForwarderEntry++;
//     }
//
//     /* Set the next bound entry to the forwarder */
//     FirstEntry = (PIMAGE_BOUND_IMPORT_DESCRIPTOR)ForwarderEntry;
//
//     /* Check if the binding was stale */
//     if (Stale)
//     {
//         /* It was, so find the IAT entry for it */
//         ++LdrpNormalSnap;
//         ImportEntry = RtlImageDirectoryEntryToData(LdrEntry->DllBase,
//                                                    TRUE,
//                                                    IMAGE_DIRECTORY_ENTRY_IMPORT,
//                                                    &IatSize);
//
//         /* Make sure it has a name */
//         while (ImportEntry->Name)
//         {
//             /* Get the name */
//             ImportName = (LPSTR)((ULONG_PTR)LdrEntry->DllBase + ImportEntry->Name);
//
//             /* Compare it */
//             if (!_stricmp(ImportName, BoundImportName)) break;
//
//             /* Move to next entry */
//             ImportEntry++;
//         }
//
//         /* If we didn't find a name, fail */
//         if (!ImportEntry->Name)
//         {
//             /* Show debug message */
//             if (ShowSnaps)
//             {
//                 DPRINT1("LDR: LdrpWalkImportTable - failing with"
//                         "STATUS_OBJECT_NAME_INVALID due to no import descriptor name\n");
//             }
//
//             /* Return error */
//             Status = STATUS_OBJECT_NAME_INVALID;
//             goto Quickie;
//         }
//
//         /* Show debug message */
//         if (ShowSnaps)
//         {
//             DPRINT1("LDR: Stale Bind %s from %wZ\n",
//                     ImportName,
//                     &LdrEntry->BaseDllName);
//         }
//
//         /* Snap the IAT Entry*/
//         Status = LdrpSnapIAT(DllLdrEntry,
//                              LdrEntry,
//                              ImportEntry,
//                              FALSE);
//
//         /* Make sure we didn't fail */
//         if (!NT_SUCCESS(Status))
//         {
//             /* Show debug message */
//             if (ShowSnaps)
//             {
//                 DPRINT1("LDR: %wZ failed to load import module %s; status = %x\n",
//                         &LdrEntry->BaseDllName,
//                         BoundImportName,
//                         Status);
//             }
//
//             /* Return */
//             goto Quickie;
//         }
//     }
//
//     /* All done */
//     Status = STATUS_SUCCESS;
//
// Quickie:
//     /* Write where we are now and return */
//     *BoundEntryPtr = FirstEntry;
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// LdrpHandleNewFormatImportDescriptors(IN LPWSTR DllPath OPTIONAL,
//                                     IN PLDR_DATA_TABLE_ENTRY LdrEntry,
//                                     IN PIMAGE_BOUND_IMPORT_DESCRIPTOR BoundEntry)
// {
//     PIMAGE_BOUND_IMPORT_DESCRIPTOR FirstEntry = BoundEntry;
//     NTSTATUS Status;
//
//     /* Make sure we have a name */
//     while (BoundEntry->OffsetModuleName)
//     {
//         /* Parse this descriptor */
//         Status = LdrpHandleOneNewFormatImportDescriptor(DllPath,
//                                                         LdrEntry,
//                                                         &BoundEntry,
//                                                         FirstEntry);
//         if (!NT_SUCCESS(Status)) return Status;
//     }
//
//     /* Done */
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// LdrpHandleOneOldFormatImportDescriptor(IN LPWSTR DllPath OPTIONAL,
//                                        IN PLDR_DATA_TABLE_ENTRY LdrEntry,
//                                        IN PIMAGE_IMPORT_DESCRIPTOR *ImportEntry)
// {
//     LPSTR ImportName;
//     NTSTATUS Status;
//     BOOLEAN AlreadyLoaded = FALSE;
//     PLDR_DATA_TABLE_ENTRY DllLdrEntry;
//     PIMAGE_THUNK_DATA FirstThunk;
//     PPEB Peb = NtCurrentPeb();
//
//     /* Get the import name's VA */
//     ImportName = (LPSTR)((ULONG_PTR)LdrEntry->DllBase + (*ImportEntry)->Name);
//
//     /* Get the first thunk */
//     FirstThunk = (PIMAGE_THUNK_DATA)((ULONG_PTR)LdrEntry->DllBase +
//                                      (*ImportEntry)->FirstThunk);
//
//     /* Make sure it's valid */
//     if (!FirstThunk->u1.Function) goto SkipEntry;
//
//     /* Show debug message */
//     if (ShowSnaps)
//     {
//         DPRINT1("LDR: %s used by %wZ\n",
//                 ImportName,
//                 &LdrEntry->BaseDllName);
//     }
//
//     /* Load the module associated to it */
//     Status = LdrpLoadImportModule(DllPath,
//                                   ImportName,
//                                   &DllLdrEntry,
//                                   &AlreadyLoaded);
//     if (!NT_SUCCESS(Status))
//     {
// #if defined(_M_ARM64) || defined(_USE_DUMMY_PSEH) || defined(_USE_PSEH_CLANG)
//         /* Early boot: tolerate missing DLLs. Many DLLs have large
//          * transitive dependency trees (rpcrt4 -> wininet -> shell32 etc.)
//          * that aren't needed at boot time. Skip the missing import and
//          * leave its IAT thunks unresolved — DllMain code paths don't
//          * call functions from these missing dependencies. */
//         DbgPrint("LDR: Skipping missing import %s for %wZ (status %x)\n",
//                 ImportName, &LdrEntry->BaseDllName, Status);
//         goto SkipEntry;
// #else
//         /* Fail */
//         if (ShowSnaps)
//         {
//             DbgPrint("LDR: LdrpWalkImportTable - LdrpLoadImportModule failed "
//                      "on import %s with status %x\n",
//                      ImportName,
//                      Status);
//         }
//
//         /* Return */
//         return Status;
// #endif
//     }
//
//     /* Show debug message */
//     if (ShowSnaps)
//     {
//         DPRINT1("LDR: Snapping imports for %wZ from %s\n",
//                 &LdrEntry->BaseDllName,
//                 ImportName);
//     }
//
//     /* Check if it wasn't already loaded */
//     ++LdrpNormalSnap;
//     if (!AlreadyLoaded)
//     {
//         /* Add the DLL to our list */
//         InsertTailList(&Peb->Ldr->InInitializationOrderModuleList,
//                        &DllLdrEntry->InInitializationOrderLinks);
//     }
//
//     /* Now snap the IAT Entry */
//     Status = LdrpSnapIAT(DllLdrEntry, LdrEntry, *ImportEntry, FALSE);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         if (ShowSnaps)
//         {
//             DbgPrint("LDR: LdrpWalkImportTable - LdrpSnapIAT #2 failed with "
//                      "status %x\n",
//                      Status);
//         }
//
//         /* Return */
//         return Status;
//     }
//
// SkipEntry:
//     /* Move on */
//     (*ImportEntry)++;
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// LdrpHandleOldFormatImportDescriptors(IN LPWSTR DllPath OPTIONAL,
//                                      IN PLDR_DATA_TABLE_ENTRY LdrEntry,
//                                      IN PIMAGE_IMPORT_DESCRIPTOR ImportEntry)
// {
//     NTSTATUS Status;
//
//     /* Check for Name and Thunk */
//     while ((ImportEntry->Name) && (ImportEntry->FirstThunk))
//     {
//         /* Parse this descriptor */
//         Status = LdrpHandleOneOldFormatImportDescriptor(DllPath,
//                                                         LdrEntry,
//                                                         &ImportEntry);
//         if (!NT_SUCCESS(Status)) return Status;
//     }
//
//     /* Done */
//     return STATUS_SUCCESS;
// }
//
// USHORT
// NTAPI
// LdrpNameToOrdinal(IN LPSTR ImportName,
//                   IN ULONG NumberOfNames,
//                   IN PVOID ExportBase,
//                   IN PULONG NameTable,
//                   IN PUSHORT OrdinalTable)
// {
//     LONG Start, End, Next, CmpResult;
//
//     /* Use classical binary search to find the ordinal */
//     Start = Next = 0;
//     End = NumberOfNames - 1;
//     while (End >= Start)
//     {
//         /* Next will be exactly between Start and End */
//         Next = (Start + End) >> 1;
//
//         /* Compare this name with the one we need to find */
//         CmpResult = strcmp(ImportName, (PCHAR)((ULONG_PTR)ExportBase + NameTable[Next]));
//
//         /* We found our entry if result is 0 */
//         if (!CmpResult) break;
//
//         /* We didn't find, update our range then */
//         if (CmpResult < 0)
//         {
//             End = Next - 1;
//         }
//         else if (CmpResult > 0)
//         {
//             Start = Next + 1;
//         }
//     }
//
//     /* If end is before start, then the search failed */
//     if (End < Start) return -1;
//
//     /* Return found name */
//     return OrdinalTable[Next];
// }
//
// NTSTATUS
// NTAPI
// LdrpWalkImportDescriptor(IN LPWSTR DllPath OPTIONAL,
//                          IN PLDR_DATA_TABLE_ENTRY LdrEntry)
// {
//     RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED ActCtx;
//     PPEB Peb = NtCurrentPeb();
//     NTSTATUS Status = STATUS_SUCCESS, Status2;
//     PIMAGE_BOUND_IMPORT_DESCRIPTOR BoundEntry = NULL;
//     PIMAGE_IMPORT_DESCRIPTOR ImportEntry;
//     ULONG BoundSize, IatSize;
//
//     DPRINT("LdrpWalkImportDescriptor - BEGIN (%wZ %p '%S')\n", &LdrEntry->BaseDllName, LdrEntry, DllPath);
//
//     /* Set up the Act Ctx */
//     RtlZeroMemory(&ActCtx, sizeof(ActCtx));
//     ActCtx.Size = sizeof(ActCtx);
//     ActCtx.Format = RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_FORMAT_WHISTLER;
//
//     /* Check if we have a manifest prober routine */
//     if (LdrpManifestProberRoutine)
//     {
//         /* Probe the DLL for its manifest. Some details are omitted */
//         Status2 = LdrpManifestProberRoutine(LdrEntry->DllBase, LdrEntry->FullDllName.Buffer, &LdrEntry->EntryPointActivationContext);
//
//         if (!NT_SUCCESS(Status2) &&
//             Status2 != STATUS_NO_SUCH_FILE &&
//             Status2 != STATUS_RESOURCE_DATA_NOT_FOUND &&
//             Status2 != STATUS_RESOURCE_TYPE_NOT_FOUND &&
//             Status2 != STATUS_RESOURCE_NAME_NOT_FOUND &&
//             Status2 != STATUS_RESOURCE_LANG_NOT_FOUND)
//         {
//             /* Some serious issue */
//             //Status = Status2; // FIXME: Ignore that error for now
//             DbgPrintEx(DPFLTR_SXS_ID,
//                 DPFLTR_WARNING_LEVEL,
//                 "LDR: LdrpWalkImportDescriptor() failed to probe %wZ for its "
//                 "manifest, ntstatus = 0x%08lx\n",
//                 &LdrEntry->FullDllName, Status2);
//         }
//     }
//
//     /* Check if we failed above */
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Get the Active ActCtx */
//     if (!LdrEntry->EntryPointActivationContext)
//     {
//         Status = RtlGetActiveActivationContext(&LdrEntry->EntryPointActivationContext);
//
//         if (!NT_SUCCESS(Status))
//         {
//             /* Exit */
//             DbgPrintEx(DPFLTR_SXS_ID,
//                 DPFLTR_WARNING_LEVEL,
//                 "LDR: RtlGetActiveActivationContext() failed; ntstatus = "
//                 "0x%08lx\n",
//                 Status);
//             return Status;
//         }
//     }
//
//     /* Activate the ActCtx */
//     RtlActivateActivationContextUnsafeFast(&ActCtx,
//                                            LdrEntry->EntryPointActivationContext);
//
//     /* Check if we were redirected */
//     if (!(LdrEntry->Flags & LDRP_REDIRECTED))
//     {
//         /* Get the Bound IAT */
//         BoundEntry = RtlImageDirectoryEntryToData(LdrEntry->DllBase,
//                                                   TRUE,
//                                                   IMAGE_DIRECTORY_ENTRY_BOUND_IMPORT,
//                                                   &BoundSize);
//     }
//
//     /* Get the regular IAT, for fallback */
//     ImportEntry = RtlImageDirectoryEntryToData(LdrEntry->DllBase,
//                                                TRUE,
//                                                IMAGE_DIRECTORY_ENTRY_IMPORT,
//                                                &IatSize);
//
//     /* Check if we got at least one */
//     if ((BoundEntry) || (ImportEntry))
//     {
//         /* Do we have a Bound IAT */
//         if (BoundEntry)
//         {
//             /* Handle the descriptor */
//             Status = LdrpHandleNewFormatImportDescriptors(DllPath,
//                                                           LdrEntry,
//                                                           BoundEntry);
//         }
//         else
//         {
//             /* Handle the descriptor */
//             Status = LdrpHandleOldFormatImportDescriptors(DllPath,
//                                                           LdrEntry,
//                                                           ImportEntry);
//         }
//
//         /* Check the status of the handlers */
//         if (NT_SUCCESS(Status))
//         {
//             /* Check for Per-DLL Heap Tagging */
//             if (Peb->NtGlobalFlag & FLG_HEAP_ENABLE_TAG_BY_DLL)
//             {
//                 /* FIXME */
//                 DPRINT1("We don't support Per-DLL Heap Tagging yet!\n");
//             }
//
//             /* Check if Page Heap was enabled */
//             if (Peb->NtGlobalFlag & FLG_HEAP_PAGE_ALLOCS)
//             {
//                 /* Initialize target DLL */
//                 AVrfPageHeapDllNotification(LdrEntry);
//             }
//
//             /* Check if Application Verifier was enabled */
//             if (Peb->NtGlobalFlag & FLG_APPLICATION_VERIFIER)
//             {
//                 AVrfDllLoadNotification(LdrEntry);
//             }
//
//             /* Just to be safe */
//             Status = STATUS_SUCCESS;
//         }
//     }
//
//     /* Release the activation context */
//     RtlDeactivateActivationContextUnsafeFast(&ActCtx);
//
//     DPRINT("LdrpWalkImportDescriptor - END (%wZ %p)\n", &LdrEntry->BaseDllName, LdrEntry);
//
//     /* Return status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// LdrpLoadImportModule(IN PWSTR DllPath OPTIONAL,
//                      IN LPSTR ImportName,
//                      OUT PLDR_DATA_TABLE_ENTRY *DataTableEntry,
//                      OUT PBOOLEAN Existing)
// {
//     ANSI_STRING AnsiString;
//     PUNICODE_STRING ImpDescName;
//     const WCHAR *p;
//     BOOLEAN GotExtension;
//     WCHAR c;
//     NTSTATUS Status;
//     PPEB Peb = RtlGetCurrentPeb();
//     PTEB Teb = NtCurrentTeb();
//     UNICODE_STRING RedirectedImpDescName;
//     BOOLEAN RedirectedDll;
//
//     DPRINT("LdrpLoadImportModule('%S' '%s' %p %p)\n", DllPath, ImportName, DataTableEntry, Existing);
//
//     RedirectedDll = FALSE;
//     RtlInitEmptyUnicodeString(&RedirectedImpDescName, NULL, 0);
//
//     /* Convert import descriptor name to unicode string */
//     ImpDescName = &Teb->StaticUnicodeString;
//     RtlInitAnsiString(&AnsiString, ImportName);
//     Status = RtlAnsiStringToUnicodeString(ImpDescName, &AnsiString, FALSE);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Find the extension, if present */
//     p = ImpDescName->Buffer + ImpDescName->Length / sizeof(WCHAR) - 1;
//     GotExtension = FALSE;
//     while (p >= ImpDescName->Buffer)
//     {
//         c = *p--;
//         if (c == L'.')
//         {
//             GotExtension = TRUE;
//             break;
//         }
//         else if (c == L'\\')
//         {
//             break;
//         }
//     }
//
//     /* If no extension was found, add the default extension */
//     if (!GotExtension)
//     {
//         /* Check that we have space to add one */
//         if ((ImpDescName->Length + LdrApiDefaultExtension.Length + sizeof(UNICODE_NULL)) >=
//             sizeof(Teb->StaticUnicodeBuffer))
//         {
//             /* No space to add the extension */
//             DbgPrintEx(DPFLTR_LDR_ID,
//                        DPFLTR_ERROR_LEVEL,
//                        "LDR: %s - Dll name missing extension; with extension "
//                        "added the name is too long\n"
//                        "   ImpDescName: (@ %p) \"%wZ\"\n"
//                        "   ImpDescName->Length: %u\n",
//                        __FUNCTION__,
//                        ImpDescName,
//                        ImpDescName,
//                        ImpDescName->Length);
//             return STATUS_NAME_TOO_LONG;
//         }
//
//         /* Add it. Needs to be null terminated, thus the length check above */
//         (VOID)RtlAppendUnicodeStringToString(ImpDescName,
//                                              &LdrApiDefaultExtension);
//     }
//
//     /* Check if the SxS Assemblies specify another file */
//     Status = LdrpApplyFileNameRedirection(
//         ImpDescName, &LdrApiDefaultExtension, NULL, &RedirectedImpDescName, &ImpDescName, &RedirectedDll);
//
//     if (!NT_SUCCESS(Status))
//     {
//         /* Unrecoverable SxS failure */
//         DPRINT1("LDR: LdrpApplyFileNameRedirection failed  with status %x for dll %wZ\n", Status, ImpDescName);
//         goto done;
//     }
//
//     /* Check if it's loaded */
//     if (LdrpCheckForLoadedDll(DllPath,
//                               ImpDescName,
//                               TRUE,
//                               RedirectedDll,
//                               DataTableEntry))
//     {
//         /* It's already existing in the list */
//         *Existing = TRUE;
//         Status = STATUS_SUCCESS;
//         goto done;
//     }
//
//     /* We're loading it for the first time */
//     *Existing = FALSE;
//
//     /* Map it */
//     Status = LdrpMapDll(DllPath,
//                         NULL,
//                         ImpDescName->Buffer,
//                         NULL,
//                         TRUE,
//                         RedirectedDll,
//                         DataTableEntry);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("LDR: LdrpMapDll failed  with status %x for dll %wZ\n", Status, ImpDescName);
//         goto done;
//     }
//
//     /* Walk its import descriptor table */
//     Status = LdrpWalkImportDescriptor(DllPath,
//                                       *DataTableEntry);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Add it to the in-init-order list in case of failure */
//         InsertTailList(&Peb->Ldr->InInitializationOrderModuleList,
//                        &(*DataTableEntry)->InInitializationOrderLinks);
//     }
//
// done:
//     RtlFreeUnicodeString(&RedirectedImpDescName);
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// LdrpSnapThunk(IN PVOID ExportBase,
//               IN PVOID ImportBase,
//               IN PIMAGE_THUNK_DATA OriginalThunk,
//               IN OUT PIMAGE_THUNK_DATA Thunk,
//               IN PIMAGE_EXPORT_DIRECTORY ExportDirectory,
//               IN ULONG ExportSize,
//               IN BOOLEAN Static,
//               IN LPSTR DllName)
// {
//     BOOLEAN IsOrdinal;
//     USHORT Ordinal;
//     ULONG OriginalOrdinal = 0;
//     PIMAGE_IMPORT_BY_NAME AddressOfData;
//     PULONG NameTable;
//     PUSHORT OrdinalTable;
//     LPSTR ImportName = NULL, DotPosition;
//     USHORT Hint;
//     NTSTATUS Status;
//     ULONG_PTR HardErrorParameters[3];
//     UNICODE_STRING HardErrorDllName, HardErrorEntryPointName;
//     ANSI_STRING TempString;
//     ULONG Mask;
//     ULONG Response;
//     PULONG AddressOfFunctions;
//     UNICODE_STRING TempUString;
//     ANSI_STRING ForwarderName;
//     PANSI_STRING ForwardName;
//     PVOID ForwarderHandle;
//     ULONG ForwardOrdinal;
//
//     /* Check if the snap is by ordinal */
//     if ((IsOrdinal = IMAGE_SNAP_BY_ORDINAL(OriginalThunk->u1.Ordinal)))
//     {
//         /* Get the ordinal number, and its normalized version */
//         OriginalOrdinal = IMAGE_ORDINAL(OriginalThunk->u1.Ordinal);
//         Ordinal = (USHORT)(OriginalOrdinal - ExportDirectory->Base);
//     }
//     else
//     {
//         /* First get the data VA */
//         AddressOfData = (PIMAGE_IMPORT_BY_NAME)
//                         ((ULONG_PTR)ImportBase +
//                         ((ULONG_PTR)OriginalThunk->u1.AddressOfData & 0xffffffff));
//
//         /* Get the name */
//         ImportName = (LPSTR)AddressOfData->Name;
//
//         /* Now get the VA of the Name and Ordinal Tables */
//         NameTable = (PULONG)((ULONG_PTR)ExportBase +
//                              (ULONG_PTR)ExportDirectory->AddressOfNames);
//         OrdinalTable = (PUSHORT)((ULONG_PTR)ExportBase +
//                                  (ULONG_PTR)ExportDirectory->AddressOfNameOrdinals);
//
//         /* Get the hint */
//         Hint = AddressOfData->Hint;
//
//         /* Try to get a match by using the hint */
//         if (((ULONG)Hint < ExportDirectory->NumberOfNames) &&
//              (!strcmp(ImportName, ((LPSTR)((ULONG_PTR)ExportBase + NameTable[Hint])))))
//         {
//             /* We got a match, get the Ordinal from the hint */
//             Ordinal = OrdinalTable[Hint];
//         }
//         else
//         {
//             /* Well bummer, hint didn't work, do it the long way */
//             Ordinal = LdrpNameToOrdinal(ImportName,
//                                         ExportDirectory->NumberOfNames,
//                                         ExportBase,
//                                         NameTable,
//                                         OrdinalTable);
//         }
//     }
//
//     /* Check if the ordinal is invalid */
//     if ((ULONG)Ordinal >= ExportDirectory->NumberOfFunctions)
//     {
// FailurePath:
//         /* Is this a static snap? */
//         if (Static)
//         {
// #ifdef _M_ARM64
//             /* ARM64: DUMMY_PSEH can't catch RtlRaiseStatus. Skip the hard
//              * error and fall through to the non-fatal path which sets the
//              * thunk to 0xffbadd11. The caller will tolerate the error. */
//             if (IsOrdinal)
//                 DPRINT1("ARM64: Missing export ordinal 0x%x from %s\n",
//                         OriginalOrdinal, DllName ? DllName : "Unknown");
//             else
//                 DPRINT1("ARM64: Missing export %s from %s\n",
//                         ImportName, DllName ? DllName : "Unknown");
// #else
//             UNICODE_STRING SnapTarget;
//             PLDR_DATA_TABLE_ENTRY LdrEntry;
//
//             /* What was the module we were searching in */
//             RtlInitAnsiString(&TempString, DllName ? DllName : "Unknown");
//
//             /* What was the module we were searching for */
//             if (LdrpCheckForLoadedDllHandle(ImportBase, &LdrEntry))
//                 SnapTarget = LdrEntry->BaseDllName;
//             else
//                 RtlInitUnicodeString(&SnapTarget, L"Unknown");
//
//             /* Inform the debug log */
//             if (IsOrdinal)
//                 DPRINT1("Failed to snap ordinal %Z!0x%x for %wZ\n", &TempString, OriginalOrdinal, &SnapTarget);
//             else
//                 DPRINT1("Failed to snap %Z!%s for %wZ\n", &TempString, ImportName, &SnapTarget);
//
//             /* These are critical errors. Setup a string for the DLL name */
//             RtlAnsiStringToUnicodeString(&HardErrorDllName, &TempString, TRUE);
//
//             /* Set it as the parameter */
//             HardErrorParameters[1] = (ULONG_PTR)&HardErrorDllName;
//             Mask = 2;
//
//             /* Check if we have an ordinal */
//             if (IsOrdinal)
//             {
//                 /* Then set the ordinal as the 1st parameter */
//                 HardErrorParameters[0] = OriginalOrdinal;
//             }
//             else
//             {
//                 /* We don't, use the entrypoint. Set up a string for it */
//                 RtlInitAnsiString(&TempString, ImportName);
//                 RtlAnsiStringToUnicodeString(&HardErrorEntryPointName,
//                                              &TempString,
//                                              TRUE);
//
//                 /* Set it as the parameter */
//                 HardErrorParameters[0] = (ULONG_PTR)&HardErrorEntryPointName;
//                 Mask = 3;
//             }
//
//             /* Raise the error */
//             NtRaiseHardError(IsOrdinal ? STATUS_ORDINAL_NOT_FOUND :
//                                          STATUS_ENTRYPOINT_NOT_FOUND,
//                              2,
//                              Mask,
//                              HardErrorParameters,
//                              OptionOk,
//                              &Response);
//
//             /* Increase the error count */
//             if (LdrpInLdrInit) LdrpFatalHardErrorCount++;
//
//             /* Free our string */
//             RtlFreeUnicodeString(&HardErrorDllName);
//             if (!IsOrdinal)
//             {
//                 /* Free our second string. Return entrypoint error */
//                 RtlFreeUnicodeString(&HardErrorEntryPointName);
//                 RtlRaiseStatus(STATUS_ENTRYPOINT_NOT_FOUND);
//             }
//
//             /* Return ordinal error */
//             RtlRaiseStatus(STATUS_ORDINAL_NOT_FOUND);
// #endif
//         }
//         else
//         {
//             /* Inform the debug log */
//             if (IsOrdinal)
//                 DPRINT("Non-fatal: Failed to snap ordinal 0x%x\n", OriginalOrdinal);
//             else
//                 DPRINT("Non-fatal: Failed to snap %s\n", ImportName);
//         }
//
//         /* Set this as a bad DLL */
//         Thunk->u1.Function = (ULONG_PTR)0xffbadd11;
//
//         /* Return the right error code */
//         Status = IsOrdinal ? STATUS_ORDINAL_NOT_FOUND :
//                              STATUS_ENTRYPOINT_NOT_FOUND;
//     }
//     else
//     {
//         /* The ordinal seems correct, get the AddressOfFunctions VA */
//         AddressOfFunctions = (PULONG)
//                              ((ULONG_PTR)ExportBase +
//                               (ULONG_PTR)ExportDirectory->AddressOfFunctions);
//
//         /* Write the function pointer*/
//         Thunk->u1.Function = (ULONG_PTR)ExportBase + AddressOfFunctions[Ordinal];
//
//         /* Make sure it's within the exports */
//         if ((Thunk->u1.Function > (ULONG_PTR)ExportDirectory) &&
//             (Thunk->u1.Function < ((ULONG_PTR)ExportDirectory + ExportSize)))
//         {
//             /* Get the Import and Forwarder Names */
//             ImportName = (LPSTR)Thunk->u1.Function;
//
//             DotPosition = strchr(ImportName, '.');
//             ASSERT(DotPosition != NULL);
//             if (!DotPosition)
//                 goto FailurePath;
//
//             ForwarderName.Buffer = ImportName;
//             ForwarderName.Length = (USHORT)(DotPosition - ImportName);
//             ForwarderName.MaximumLength = ForwarderName.Length;
//
//             /* LLD/MinGW workaround: LLD prepends '_' to forward module names
//                on i386 (e.g. "_ntdll.RtlFoo" instead of "ntdll.RtlFoo").
//                DLL module names never start with '_', so strip it. */
//             if (ForwarderName.Length > 1 && ForwarderName.Buffer[0] == '_')
//             {
//                 ForwarderName.Buffer++;
//                 ForwarderName.Length--;
//                 ForwarderName.MaximumLength--;
//             }
//
//             Status = RtlAnsiStringToUnicodeString(&TempUString,
//                                                   &ForwarderName,
//                                                   TRUE);
//
//             /* Make sure the conversion was OK */
//             if (NT_SUCCESS(Status))
//             {
//                 WCHAR StringBuffer[MAX_PATH];
//                 UNICODE_STRING StaticString, *RedirectedImportName;
//                 BOOLEAN Redirected = FALSE;
//
//                 RtlInitEmptyUnicodeString(&StaticString, StringBuffer, sizeof(StringBuffer));
//
//                 /* Check if the SxS Assemblies specify another file */
//                 Status = LdrpApplyFileNameRedirection(
//                     &TempUString, &LdrApiDefaultExtension, &StaticString, NULL, &RedirectedImportName, &Redirected);
//
//                 if (NT_SUCCESS(Status) && Redirected)
//                 {
//                     if (ShowSnaps)
//                         DPRINT1("LDR: %Z got redirected to %wZ\n", &ForwarderName, RedirectedImportName);
//                 }
//                 else
//                 {
//                     RedirectedImportName = &TempUString;
//                 }
//
//                 /* Load the forwarder */
//                 Status = LdrpLoadDll(Redirected,
//                                      NULL,
//                                      NULL,
//                                      RedirectedImportName,
//                                      &ForwarderHandle,
//                                      FALSE);
//
//                 RtlFreeUnicodeString(&TempUString);
//             }
//
//             /* If the load or conversion failed, use the failure path */
//             if (!NT_SUCCESS(Status)) goto FailurePath;
//
//             /* Now set up a name for the actual forwarder dll */
//             RtlInitAnsiString(&ForwarderName,
//                               ImportName + ForwarderName.Length + sizeof(CHAR));
//
//             /* Check if it's an ordinal forward */
//             if ((ForwarderName.Length > 1) && (*ForwarderName.Buffer == '#'))
//             {
//                 /* We don't have an actual function name */
//                 ForwardName = NULL;
//
//                 /* Convert the string into an ordinal */
//                 Status = RtlCharToInteger(ForwarderName.Buffer + sizeof(CHAR),
//                                           0,
//                                           &ForwardOrdinal);
//
//                 /* If this fails, then error out */
//                 if (!NT_SUCCESS(Status)) goto FailurePath;
//             }
//             else
//             {
//                 /* Import by name */
//                 ForwardName = &ForwarderName;
//                 ForwardOrdinal = 0;
//             }
//
//             /* Get the pointer */
//             Status = LdrpGetProcedureAddress(ForwarderHandle,
//                                              ForwardName,
//                                              ForwardOrdinal,
//                                              (PVOID*)&Thunk->u1.Function,
//                                              FALSE);
//             /* If this fails, then error out */
//             if (!NT_SUCCESS(Status)) goto FailurePath;
//         }
//         else
//         {
//             /* It's not within the exports, let's hope it's valid */
//             if (!AddressOfFunctions[Ordinal]) goto FailurePath;
//         }
//
//         /* If we got here, then it's success */
//         Status = STATUS_SUCCESS;
//     }
//
//     /* Return status */
//     return Status;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
