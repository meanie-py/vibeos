// This file mirrors ldrutils.c's manipulation of C-owned `static mut` loader
// globals; reads/writes through `static mut` are expected and intentional.
#![allow(static_mut_refs)]

use ntdll_sys::nt::*;
use core::mem::offset_of;

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


// ===========================================================================
// Prelude — ntdllp.h / ntstatus.h surface absent from the ntdll-sys prelude.
// (See memory feedback_rustconv_header_types_integration.)
// ===========================================================================

#[inline]
fn NT_SUCCESS(status: NTSTATUS) -> bool {
    status >= 0
}

#[inline]
unsafe fn RtlZeroMemory(dest: PVOID, length: usize) {
    core::ptr::write_bytes(dest as *mut u8, 0, length);
}

// ntstatus.h cast-macros not in the bindgen surface.
const STATUS_INVALID_PARAMETER: NTSTATUS = 0xC000_000Du32 as NTSTATUS;
const STATUS_NO_MEMORY: NTSTATUS = 0xC000_0017u32 as NTSTATUS;
const STATUS_CONFLICTING_ADDRESSES: NTSTATUS = 0xC000_0018u32 as NTSTATUS;
const STATUS_PROCEDURE_NOT_FOUND: NTSTATUS = 0xC000_007Au32 as NTSTATUS;
const STATUS_INVALID_IMAGE_FORMAT: NTSTATUS = 0xC000_007Bu32 as NTSTATUS;
const STATUS_NAME_TOO_LONG: NTSTATUS = 0xC000_0106u32 as NTSTATUS;
const STATUS_DLL_NOT_FOUND: NTSTATUS = 0xC000_0135u32 as NTSTATUS;
const STATUS_IMAGE_MACHINE_TYPE_MISMATCH: NTSTATUS = 0x4000_000Eu32 as NTSTATUS;
const STATUS_ILLEGAL_DLL_RELOCATION: NTSTATUS = 0xC000_0269u32 as NTSTATUS;
const STATUS_SXS_SECTION_NOT_FOUND: NTSTATUS = 0xC015_0001u32 as NTSTATUS;
const STATUS_SXS_KEY_NOT_FOUND: NTSTATUS = 0xC015_0008u32 as NTSTATUS;

// ldrtypes.h / pe constants.
const DPFLTR_LDR_ID: ULONG = 85;
const IMAGE_LOADER_FLAGS_COMPLUS: ULONG = 0x0000_0001;
const LDRP_ENTRY_PROCESSED: ULONG = 0x0000_4000;
const UNICODE_STRING_MAX_BYTES: USHORT = 65534;
const LDR_HASH_TABLE_ENTRIES: usize = 32;

// ldrutils.c LdrpUpdateLoadCount2/3 flags (ntdllp.h LDRP_UPDATE_*).
const LDRP_UPDATE_REFCOUNT: ULONG = 1;
const LDRP_UPDATE_DEREFCOUNT: ULONG = 2;
const LDRP_UPDATE_PIN: ULONG = 3;

// ntdllp.h: `#define LDR_GET_HASH_ENTRY(x) (RtlUpcaseUnicodeChar(x) & (N-1))`.
#[inline]
unsafe fn LDR_GET_HASH_ENTRY(ch: WCHAR) -> ULONG {
    (RtlUpcaseUnicodeChar(ch) as ULONG) & (LDR_HASH_TABLE_ENTRIES as ULONG - 1)
}

// FIELD_OFFSET(LDR_DATA_TABLE_ENTRY, <link>) — CONTAINING_RECORD back-offsets.
#[inline]
fn FIELD_OFFSET_LDR_InLoadOrderLinks() -> usize {
    offset_of!(_LDR_DATA_TABLE_ENTRY, InLoadOrderLinks)
}
#[inline]
fn FIELD_OFFSET_LDR_HashLinks() -> usize {
    // HashLinks lives in the first anonymous union (bindgen __bindgen_anon_1).
    offset_of!(_LDR_DATA_TABLE_ENTRY, __bindgen_anon_1)
}

// ALIGN_DOWN(size, type) = size & ~(sizeof(type)-1).
#[inline]
fn ALIGN_DOWN(size: usize, align: usize) -> usize {
    size & !(align - 1)
}

// C-owned loader globals (ldrinit.c) referenced by ldrutils. Types per ntdllp.h.
extern "C" {
    static mut ShowSnaps: BOOLEAN;
    static mut LdrpHeap: PVOID;
    static mut LdrpDefaultPath: _UNICODE_STRING;
    static mut LdrpLoaderLock: RTL_CRITICAL_SECTION;
    static mut LdrpKnownDllObjectDirectory: HANDLE;
    static mut LdrpKnownDllPath: _UNICODE_STRING;
    static mut LdrpInLdrInit: BOOLEAN;
    static mut LdrpLdrDatabaseIsSetup: BOOLEAN;
    static mut LdrpFatalHardErrorCount: ULONG;
    static mut LdrpNumberOfProcessors: ULONG;
    static mut LdrApiDefaultExtension: _UNICODE_STRING;
    static mut LdrpHashTable: [LIST_ENTRY; LDR_HASH_TABLE_ENTRIES];
}

// Still-C loader helpers (ldrinit.c / ldrapi.c) + SxS redirection. NTAPI.
extern "system" {
    fn LdrpEnsureLoaderLockIsHeld();
    fn LdrpRunInitializeRoutines(Context: PCONTEXT) -> NTSTATUS;
    fn LdrpValidateImageForMp(LdrEntry: PLDR_DATA_TABLE_ENTRY);
    fn LdrpSendDllNotifications(LdrEntry: PLDR_DATA_TABLE_ENTRY, NotificationReason: ULONG);
    fn LdrpApplyFileNameRedirection(
        OriginalName: *mut _UNICODE_STRING,
        Extension: *mut _UNICODE_STRING,
        StaticString: *mut _UNICODE_STRING,
        DynamicString: *mut _UNICODE_STRING,
        NewName: *mut *mut _UNICODE_STRING,
        Redirected: *mut BOOLEAN,
    ) -> NTSTATUS;
    // LdrpWalkImportDescriptor + LdrpSnapThunk live in ldrpe.rs (same crate).
    fn LdrpWalkImportDescriptor(DllPath: LPWSTR, LdrEntry: PLDR_DATA_TABLE_ENTRY) -> NTSTATUS;
    fn LdrpSnapThunk(
        ExportBase: PVOID,
        ImportBase: PVOID,
        OriginalThunk: PIMAGE_THUNK_DATA,
        Thunk: PIMAGE_THUNK_DATA,
        ExportDirectory: PIMAGE_EXPORT_DIRECTORY,
        ExportSize: ULONG,
        Static: BOOLEAN,
        DllName: LPSTR,
    ) -> NTSTATUS;
}

// C runtime — cdecl, NOT NTAPI.
extern "C" {
    fn wcslen(s: *const WCHAR) -> usize;
}

/// `RtlGetProcessHeap` — FORCEINLINE in rtlfuncs.h (no exported symbol).
#[inline(always)]
unsafe fn RtlGetProcessHeap() -> PVOID {
    (*NtCurrentPeb()).ProcessHeap
}

/// `RtlInitEmptyUnicodeString` — FORCEINLINE in rtlfuncs.h (no exported symbol).
#[inline(always)]
unsafe fn RtlInitEmptyUnicodeString(s: *mut UNICODE_STRING, buffer: PWSTR, buffer_size: USHORT) {
    (*s).Length = 0;
    (*s).MaximumLength = buffer_size;
    (*s).Buffer = buffer;
}

/// `InitializeObjectAttributes` macro (ntdef.h) rendered as an inline fn.
#[inline]
unsafe fn InitializeObjectAttributes_inline(
    p: *mut OBJECT_ATTRIBUTES,
    n: *mut _UNICODE_STRING,
    a: ULONG,
    r: HANDLE,
    s: PVOID,
) {
    (*p).Length = core::mem::size_of::<OBJECT_ATTRIBUTES>() as ULONG;
    (*p).RootDirectory = r;
    (*p).Attributes = a;
    (*p).ObjectName = n;
    (*p).SecurityDescriptor = s;
    (*p).SecurityQualityOfService = core::ptr::null_mut();
}

// ---- file-scope globals OWNED by ldrutils.c — #[no_mangle] so the still-C
// loader TUs (ldrapi.c/ldrinit.c) resolve against these Rust definitions. ----
#[no_mangle]
pub static mut LdrpLoadedDllHandleCache: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
#[no_mangle]
pub static mut LdrpGetModuleHandleCache: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
#[no_mangle]
pub static mut g_ShimsEnabled: BOOLEAN = 0;
#[no_mangle]
pub static mut g_pShimEngineModule: PVOID = core::ptr::null_mut();
#[no_mangle]
pub static mut g_pfnSE_DllLoaded: PVOID = core::ptr::null_mut();
#[no_mangle]
pub static mut g_pfnSE_DllUnloaded: PVOID = core::ptr::null_mut();
#[no_mangle]
pub static mut g_pfnSE_InstallBeforeInit: PVOID = core::ptr::null_mut();
#[no_mangle]
pub static mut g_pfnSE_InstallAfterInit: PVOID = core::ptr::null_mut();
#[no_mangle]
pub static mut g_pfnSE_ProcessDying: PVOID = core::ptr::null_mut();
#[no_mangle]
pub unsafe extern "system" fn LdrpAllocateUnicodeString(StringOut: PUNICODE_STRING, Length: u32) -> i32 {
    const UNICODE_STRING_MAX_BYTES: u32 = 0xFFFE;
    const STATUS_INVALID_PARAMETER: i32 = 0xC000000Du32 as i32;

    (*StringOut).Length = 0;

    if (Length & 1) != 0 {
        (*StringOut).Buffer = core::ptr::null_mut();
        (*StringOut).MaximumLength = 0;
        return STATUS_INVALID_PARAMETER;
    }

    (*StringOut).Buffer = RtlAllocateHeap(LdrpHeap, 0, (Length as usize + core::mem::size_of::<u16>()) as SIZE_T) as *mut u16;
    if (*StringOut).Buffer.is_null() {
        (*StringOut).MaximumLength = 0;
        return STATUS_NO_MEMORY;
    }

    *(*StringOut).Buffer.add((Length / core::mem::size_of::<u16>() as u32) as usize) = 0;

    if Length != UNICODE_STRING_MAX_BYTES {
        (*StringOut).MaximumLength = (Length as u16) + core::mem::size_of::<u16>() as u16;
    } else {
        (*StringOut).MaximumLength = UNICODE_STRING_MAX_BYTES as u16;
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn LdrpFreeUnicodeString(StringIn: PUNICODE_STRING) {
    if !(*StringIn).Buffer.is_null() {
        RtlFreeHeap(LdrpHeap, 0, (*StringIn).Buffer as *mut _);
    }

    RtlInitEmptyUnicodeString(StringIn, core::ptr::null_mut(), 0);
}

#[no_mangle]
pub unsafe extern "system" fn LdrpCallInitRoutine(
    EntryPoint: PDLL_INIT_ROUTINE,
    BaseAddress: PVOID,
    Reason: ULONG,
    Context: PVOID,
) -> BOOLEAN {
    (EntryPoint.unwrap())(BaseAddress, Reason, Context as *mut _CONTEXT)
}

#[no_mangle]
pub unsafe extern "system" fn LdrpUpdateLoadCount3(
    LdrEntry: PLDR_DATA_TABLE_ENTRY,
    Flags: ULONG,
    UpdateString: PUNICODE_STRING,
) {
    let mut NewImportForwarder: PIMAGE_BOUND_FORWARDER_REF;
    let FirstEntry: PIMAGE_BOUND_IMPORT_DESCRIPTOR;
    let mut BoundEntry: PIMAGE_BOUND_IMPORT_DESCRIPTOR;
    let mut ImportEntry: PIMAGE_IMPORT_DESCRIPTOR;
    let mut FirstThunk: PIMAGE_THUNK_DATA;
    let mut Entry: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
    let ImportNameUnic: PUNICODE_STRING;
    let mut RedirectedImportName: PUNICODE_STRING;
    let mut ImportNameAnsi: ANSI_STRING = core::mem::zeroed();
    let mut ImportName: LPSTR;
    let mut ImportSize: ULONG = 0;
    let mut Status: NTSTATUS;
    let mut i: ULONG;
    let mut RedirectedDll: BOOLEAN;
    let mut ActCtx: RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED =
        core::mem::zeroed();

    ActCtx.Size = core::mem::size_of::<RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED>() as _;
    ActCtx.Format = RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_FORMAT_WHISTLER;
    RtlZeroMemory(
        core::ptr::addr_of_mut!(ActCtx.Frame) as PVOID,
        core::mem::size_of::<RTL_ACTIVATION_CONTEXT_STACK_FRAME>(),
    );

    RtlActivateActivationContextUnsafeFast(
        &mut ActCtx,
        (*LdrEntry).EntryPointActivationContext,
    );

    'done: {
        if Flags == LDRP_UPDATE_REFCOUNT || Flags == LDRP_UPDATE_PIN {
            if (*LdrEntry).Flags & LDRP_LOAD_IN_PROGRESS != 0 {
                break 'done;
            }
            (*LdrEntry).Flags |= LDRP_LOAD_IN_PROGRESS;
        } else if Flags == LDRP_UPDATE_DEREFCOUNT {
            if (*LdrEntry).Flags & LDRP_UNLOAD_IN_PROGRESS != 0 {
                break 'done;
            }
            (*LdrEntry).Flags |= LDRP_UNLOAD_IN_PROGRESS;
        }

        ImportNameUnic = core::ptr::addr_of_mut!((*NtCurrentTeb()).StaticUnicodeString);

        FirstEntry = RtlImageDirectoryEntryToData(
            (*LdrEntry).DllBase,
            1u8,
            IMAGE_DIRECTORY_ENTRY_BOUND_IMPORT as USHORT,
            &mut ImportSize,
        ) as PIMAGE_BOUND_IMPORT_DESCRIPTOR;

        if !FirstEntry.is_null() {
            if Flags == LDRP_UPDATE_REFCOUNT {
                (*LdrEntry).Flags |= LDRP_LOAD_IN_PROGRESS;
            } else if Flags == LDRP_UPDATE_DEREFCOUNT {
                (*LdrEntry).Flags |= LDRP_UNLOAD_IN_PROGRESS;
            }

            BoundEntry = FirstEntry;
            while (*BoundEntry).OffsetModuleName != 0 {
                ImportName = (FirstEntry as *mut u8)
                    .add((*BoundEntry).OffsetModuleName as usize)
                    as LPSTR;

                RtlInitAnsiString(&mut ImportNameAnsi, ImportName as PCSZ);
                Status = RtlAnsiStringToUnicodeString(ImportNameUnic, &mut ImportNameAnsi, 0u8);

                if NT_SUCCESS(Status) {
                    RedirectedDll = 0u8;
                    RedirectedImportName = ImportNameUnic;

                    Status = LdrpApplyFileNameRedirection(
                        ImportNameUnic,
                        core::ptr::addr_of_mut!(LdrApiDefaultExtension),
                        UpdateString,
                        core::ptr::null_mut(),
                        &mut RedirectedImportName,
                        &mut RedirectedDll,
                    );

                    if NT_SUCCESS(Status) {
                        if LdrpCheckForLoadedDll(
                            core::ptr::null_mut(),
                            RedirectedImportName,
                            1u8,
                            RedirectedDll,
                            &mut Entry,
                        ) != 0
                        {
                            if (*Entry).LoadCount != 0xFFFF {
                                match Flags {
                                    LDRP_UPDATE_REFCOUNT => {
                                        (*Entry).LoadCount += 1;
                                    }
                                    LDRP_UPDATE_DEREFCOUNT => {
                                        (*Entry).LoadCount -= 1;
                                    }
                                    LDRP_UPDATE_PIN => {
                                        (*Entry).LoadCount = 0xFFFF;
                                    }
                                    _ => {}
                                }
                            }

                            LdrpUpdateLoadCount3(Entry, Flags, UpdateString);
                        }
                    }
                }

                NewImportForwarder = BoundEntry.add(1) as PIMAGE_BOUND_FORWARDER_REF;
                i = 0;
                while i < (*BoundEntry).NumberOfModuleForwarderRefs as ULONG {
                    ImportName = (FirstEntry as *mut u8)
                        .add((*NewImportForwarder).OffsetModuleName as usize)
                        as LPSTR;

                    RtlInitAnsiString(&mut ImportNameAnsi, ImportName as PCSZ);
                    Status =
                        RtlAnsiStringToUnicodeString(ImportNameUnic, &mut ImportNameAnsi, 0u8);
                    if NT_SUCCESS(Status) {
                        RedirectedDll = 0u8;
                        RedirectedImportName = ImportNameUnic;

                        Status = LdrpApplyFileNameRedirection(
                            ImportNameUnic,
                            core::ptr::addr_of_mut!(LdrApiDefaultExtension),
                            UpdateString,
                            core::ptr::null_mut(),
                            &mut RedirectedImportName,
                            &mut RedirectedDll,
                        );

                        if NT_SUCCESS(Status) {
                            if LdrpCheckForLoadedDll(
                                core::ptr::null_mut(),
                                RedirectedImportName,
                                1u8,
                                RedirectedDll,
                                &mut Entry,
                            ) != 0
                            {
                                if (*Entry).LoadCount != 0xFFFF {
                                    match Flags {
                                        LDRP_UPDATE_REFCOUNT => {
                                            (*Entry).LoadCount += 1;
                                        }
                                        LDRP_UPDATE_DEREFCOUNT => {
                                            (*Entry).LoadCount -= 1;
                                        }
                                        LDRP_UPDATE_PIN => {
                                            (*Entry).LoadCount = 0xFFFF;
                                        }
                                        _ => {}
                                    }
                                }

                                LdrpUpdateLoadCount3(Entry, Flags, UpdateString);
                            }
                        }
                    }

                    NewImportForwarder = NewImportForwarder.add(1);
                    i += 1;
                }

                BoundEntry = NewImportForwarder as PIMAGE_BOUND_IMPORT_DESCRIPTOR;
            }

            break 'done;
        }

        ImportEntry = RtlImageDirectoryEntryToData(
            (*LdrEntry).DllBase,
            1u8,
            IMAGE_DIRECTORY_ENTRY_IMPORT as USHORT,
            &mut ImportSize,
        ) as PIMAGE_IMPORT_DESCRIPTOR;

        if !ImportEntry.is_null() {
            while (*ImportEntry).Name != 0 && (*ImportEntry).FirstThunk != 0 {
                FirstThunk = ((*LdrEntry).DllBase as ULONG_PTR
                    + (*ImportEntry).FirstThunk as ULONG_PTR)
                    as PIMAGE_THUNK_DATA;

                if (*FirstThunk).u1.Function == 0 {
                    ImportEntry = ImportEntry.add(1);
                    continue;
                }

                ImportName = ((*LdrEntry).DllBase as ULONG_PTR
                    + (*ImportEntry).Name as ULONG_PTR) as LPSTR;

                RtlInitAnsiString(&mut ImportNameAnsi, ImportName as PCSZ);
                Status = RtlAnsiStringToUnicodeString(ImportNameUnic, &mut ImportNameAnsi, 0u8);
                if NT_SUCCESS(Status) {
                    RedirectedDll = 0u8;
                    RedirectedImportName = ImportNameUnic;

                    Status = LdrpApplyFileNameRedirection(
                        ImportNameUnic,
                        core::ptr::addr_of_mut!(LdrApiDefaultExtension),
                        UpdateString,
                        core::ptr::null_mut(),
                        &mut RedirectedImportName,
                        &mut RedirectedDll,
                    );

                    if NT_SUCCESS(Status) {
                        if LdrpCheckForLoadedDll(
                            core::ptr::null_mut(),
                            RedirectedImportName,
                            1u8,
                            RedirectedDll,
                            &mut Entry,
                        ) != 0
                        {
                            if (*Entry).LoadCount != 0xFFFF {
                                match Flags {
                                    LDRP_UPDATE_REFCOUNT => {
                                        (*Entry).LoadCount += 1;
                                    }
                                    LDRP_UPDATE_DEREFCOUNT => {
                                        (*Entry).LoadCount -= 1;
                                    }
                                    LDRP_UPDATE_PIN => {
                                        (*Entry).LoadCount = 0xFFFF;
                                    }
                                    _ => {}
                                }
                            }

                            LdrpUpdateLoadCount3(Entry, Flags, UpdateString);
                        }
                    }
                }

                ImportEntry = ImportEntry.add(1);
            }
        }
    }

    RtlDeactivateActivationContextUnsafeFast(&mut ActCtx);
}

#[no_mangle]
pub unsafe extern "system" fn LdrpUpdateLoadCount2(
    LdrEntry: PLDR_DATA_TABLE_ENTRY,
    Flags: ULONG,
) {
    let mut Buffer: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut UpdateString: _UNICODE_STRING = core::mem::zeroed();

    RtlInitEmptyUnicodeString(
        &mut UpdateString,
        Buffer.as_mut_ptr(),
        core::mem::size_of_val(&Buffer) as u16,
    );
    LdrpUpdateLoadCount3(LdrEntry, Flags, &mut UpdateString);
}

#[no_mangle]
pub unsafe extern "system" fn LdrpCallTlsInitializers(
    LdrEntry: PLDR_DATA_TABLE_ENTRY,
    Reason: ULONG,
) {
    let mut Size: ULONG = 0;

    let TlsDirectory: PIMAGE_TLS_DIRECTORY = RtlImageDirectoryEntryToData(
        (*LdrEntry).DllBase,
        1u8,
        IMAGE_DIRECTORY_ENTRY_TLS as USHORT,
        &mut Size,
    ) as PIMAGE_TLS_DIRECTORY;

    if !TlsDirectory.is_null() {
        let mut Array: *mut PIMAGE_TLS_CALLBACK =
            (*TlsDirectory).AddressOfCallBacks as *mut PIMAGE_TLS_CALLBACK;
        if !Array.is_null() {
            while !(*Array).is_none() {
                let Callback: PIMAGE_TLS_CALLBACK = *Array;
                Array = Array.add(1);

                LdrpCallInitRoutine(
                    core::mem::transmute::<PIMAGE_TLS_CALLBACK, PDLL_INIT_ROUTINE>(Callback),
                    (*LdrEntry).DllBase,
                    Reason,
                    core::ptr::null_mut(),
                );
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn LdrpCodeAuthzCheckDllAllowed(
    _FullName: PUNICODE_STRING,
    _DllHandle: HANDLE,
) -> NTSTATUS {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn LdrpCreateDllSection(
    FullName: PUNICODE_STRING,
    DllHandle: HANDLE,
    DllCharacteristics: PULONG,
    SectionHandle: PHANDLE,
) -> NTSTATUS {
    let mut FileHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut HardErrorParameters: [ULONG_PTR; 1] = [0];
    let mut Response: ULONG = 0;
    let mut SectionImageInfo: SECTION_IMAGE_INFORMATION = core::mem::zeroed();

    if DllHandle.is_null() {
        /* InitializeObjectAttributes expanded */
        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
        ObjectAttributes.RootDirectory = core::ptr::null_mut();
        ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
        ObjectAttributes.ObjectName = FullName;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        Status = NtOpenFile(
            &mut FileHandle,
            SYNCHRONIZE | FILE_EXECUTE | FILE_READ_DATA,
            &mut ObjectAttributes,
            &mut IoStatusBlock,
            FILE_SHARE_READ | FILE_SHARE_DELETE,
            FILE_NON_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT,
        );

        if !NT_SUCCESS(Status) {
            Status = NtOpenFile(
                &mut FileHandle,
                SYNCHRONIZE | FILE_EXECUTE,
                &mut ObjectAttributes,
                &mut IoStatusBlock,
                FILE_SHARE_READ | FILE_SHARE_DELETE,
                FILE_NON_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT,
            );

            if !NT_SUCCESS(Status) {
                if Status == STATUS_OBJECT_NAME_NOT_FOUND {
                    Status = STATUS_DLL_NOT_FOUND;
                }

                *SectionHandle = core::ptr::null_mut();
                return Status;
            }
        }
    } else {
        FileHandle = DllHandle;
    }

    Status = NtCreateSection(
        SectionHandle,
        SECTION_MAP_READ | SECTION_MAP_EXECUTE | SECTION_MAP_WRITE | SECTION_QUERY,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        PAGE_EXECUTE,
        SEC_IMAGE,
        FileHandle,
    );

    'exit: {
        if !NT_SUCCESS(Status) {
            *SectionHandle = core::ptr::null_mut();

            HardErrorParameters[0] = FullName as ULONG_PTR;

            ZwRaiseHardError(
                STATUS_INVALID_IMAGE_FORMAT,
                1,
                1,
                HardErrorParameters.as_mut_ptr(),
                OptionOk,
                &mut Response,
            );

            if LdrpInLdrInit != 0 {
                LdrpFatalHardErrorCount += 1;
            }

            break 'exit;
        }

        if DllCharacteristics.is_null() || (*DllCharacteristics & IMAGE_FILE_SYSTEM) == 0 {
            Status = ZwQuerySection(
                *SectionHandle,
                SectionImageInformation,
                &mut SectionImageInfo as *mut _ as *mut _,
                core::mem::size_of::<SECTION_IMAGE_INFORMATION>() as SIZE_T,
                core::ptr::null_mut(),
            );
            if NT_SUCCESS(Status) {
                if (SectionImageInfo.LoaderFlags & IMAGE_LOADER_FLAGS_COMPLUS) == 0 {
                    Status = LdrpCodeAuthzCheckDllAllowed(FullName, DllHandle);
                    if !NT_SUCCESS(Status) && Status != STATUS_NOT_FOUND {
                        NtClose(*SectionHandle);
                        *SectionHandle = core::ptr::null_mut();
                    }
                }
            } else {
                NtClose(*SectionHandle);
                *SectionHandle = core::ptr::null_mut();
            }
        }
    }

    NtClose(FileHandle);

    Status
}
#[no_mangle]
pub unsafe extern "system" fn LdrpResolveDllName(
    DllPath: PWSTR,
    DllName: PWSTR,
    FullDllName: PUNICODE_STRING,
    BaseDllName: PUNICODE_STRING,
) -> u8 {
    let mut p2: *mut u16 = core::ptr::null_mut();
    let BufSize: u32 = 500;

    (*FullDllName).Buffer = RtlAllocateHeap(LdrpHeap, 0, (BufSize as usize + 2) as _) as PWSTR;
    if (*FullDllName).Buffer.is_null() {
        return 0u8;
    }

    let Length = RtlDosSearchPath_U(
        if !DllPath.is_null() { DllPath } else { LdrpDefaultPath.Buffer },
        DllName,
        core::ptr::null_mut(),
        BufSize,
        (*FullDllName).Buffer,
        core::ptr::addr_of_mut!((*BaseDllName).Buffer),
    );

    if Length == 0 || Length > BufSize {
        LdrpFreeUnicodeString(FullDllName);
        return 0u8;
    }

    (*FullDllName).Length = Length as u16;
    (*FullDllName).MaximumLength = (*FullDllName).Length + 2;

    let NameBuffer = RtlAllocateHeap(LdrpHeap, 0, (*FullDllName).MaximumLength as _) as PWSTR;
    if NameBuffer.is_null() {
        RtlFreeHeap(LdrpHeap, 0, (*FullDllName).Buffer as *mut _);
        return 0u8;
    }

    core::ptr::copy_nonoverlapping(
        (*FullDllName).Buffer as *const u8,
        NameBuffer as *mut u8,
        (*FullDllName).MaximumLength as usize,
    );
    RtlFreeHeap(LdrpHeap, 0, (*FullDllName).Buffer as *mut _);
    (*FullDllName).Buffer = NameBuffer;

    let mut p1: *mut u16 = (*FullDllName).Buffer;
    while *p1 != 0 {
        let ch = *p1;
        p1 = p1.add(1);
        if ch == b'\\' as u16 {
            p2 = p1;
        }
    }

    if !p2.is_null() {
        p1 = p2;
    } else {
        p1 = DllName;
    }

    p2 = p1;

    while *p1 != 0 {
        p1 = p1.add(1);
    }

    (*BaseDllName).Length = ((p1 as usize) - (p2 as usize)) as u16;
    (*BaseDllName).MaximumLength = (*BaseDllName).Length + 2;
    (*BaseDllName).Buffer = RtlAllocateHeap(LdrpHeap, 0, (*BaseDllName).MaximumLength as _) as PWSTR;

    if (*BaseDllName).Buffer.is_null() {
        RtlFreeHeap(LdrpHeap, 0, NameBuffer as *mut _);
        return 0u8;
    }

    core::ptr::copy(
        p2 as *const u8,
        (*BaseDllName).Buffer as *mut u8,
        (*BaseDllName).Length as usize,
    );

    *(*BaseDllName).Buffer.add(((*BaseDllName).Length / 2) as usize) = 0;

    1u8
}
#[no_mangle]
pub unsafe extern "system" fn LdrpFetchAddressOfEntryPoint(ImageBase: PVOID) -> PVOID {
    let mut EntryPoint: ULONG_PTR = 0;

    let NtHeaders: PIMAGE_NT_HEADERS = RtlImageNtHeader(ImageBase);
    if !NtHeaders.is_null() {
        EntryPoint = (*NtHeaders).OptionalHeader.AddressOfEntryPoint as ULONG_PTR;
        if EntryPoint != 0 {
            EntryPoint += ImageBase as ULONG_PTR;
        }
    }

    EntryPoint as PVOID
}
#[no_mangle]
pub unsafe extern "system" fn LdrpCheckForKnownDll(
    DllName: PWSTR,
    FullDllName: PUNICODE_STRING,
    BaseDllName: PUNICODE_STRING,
    SectionHandle: *mut HANDLE,
) -> NTSTATUS {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Section: HANDLE = core::ptr::null_mut();
    let mut DllNameUnic: _UNICODE_STRING = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut p1: PCHAR;
    let mut p2: PWCHAR;

    if !SectionHandle.is_null() {
        *SectionHandle = core::ptr::null_mut();
    }

    if !FullDllName.is_null() {
        (*FullDllName).Length = 0;
        (*FullDllName).MaximumLength = 0;
        (*FullDllName).Buffer = core::ptr::null_mut();
    }

    if !BaseDllName.is_null() {
        (*BaseDllName).Length = 0;
        (*BaseDllName).MaximumLength = 0;
        (*BaseDllName).Buffer = core::ptr::null_mut();
    }

    if SectionHandle.is_null() || FullDllName.is_null() || BaseDllName.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    LdrpEnsureLoaderLockIsHeld();

    RtlInitUnicodeString(&mut DllNameUnic, DllName);

    Status = RtlFindActivationContextSectionString(
        0,
        core::ptr::null_mut(),
        ACTIVATION_CONTEXT_SECTION_DLL_REDIRECTION,
        &mut DllNameUnic,
        core::ptr::null_mut(),
    );

    'failure: {
        if Status == STATUS_SXS_SECTION_NOT_FOUND || Status == STATUS_SXS_KEY_NOT_FOUND {
            (*BaseDllName).Length = DllNameUnic.Length;
            (*BaseDllName).MaximumLength = DllNameUnic.MaximumLength;
            (*BaseDllName).Buffer = RtlAllocateHeap(
                LdrpHeap,
                0,
                DllNameUnic.MaximumLength as SIZE_T,
            ) as PWSTR;
            if (*BaseDllName).Buffer.is_null() {
                Status = STATUS_NO_MEMORY;
                break 'failure;
            }

            core::ptr::copy_nonoverlapping(
                DllNameUnic.Buffer as *const u8,
                (*BaseDllName).Buffer as *mut u8,
                DllNameUnic.MaximumLength as usize,
            );

            (*FullDllName).Length = LdrpKnownDllPath.Length
                + (*BaseDllName).Length
                + core::mem::size_of::<u16>() as u16;
            (*FullDllName).MaximumLength =
                (*FullDllName).Length + core::mem::size_of::<u16>() as u16;
            (*FullDllName).Buffer = RtlAllocateHeap(
                LdrpHeap,
                0,
                (*FullDllName).MaximumLength as SIZE_T,
            ) as PWSTR;
            if (*FullDllName).Buffer.is_null() {
                Status = STATUS_NO_MEMORY;
                break 'failure;
            }

            core::ptr::copy_nonoverlapping(
                LdrpKnownDllPath.Buffer as *const u8,
                (*FullDllName).Buffer as *mut u8,
                LdrpKnownDllPath.Length as usize,
            );

            p1 = ((*FullDllName).Buffer as PCHAR).add(LdrpKnownDllPath.Length as usize);
            p2 = p1 as PWCHAR;
            *p2 = '\\' as u16;
            p2 = p2.add(1);
            p1 = p2 as PCHAR;

            DllNameUnic.Buffer = p1 as PWSTR;
            DllNameUnic.Length = (*BaseDllName).Length;
            DllNameUnic.MaximumLength =
                DllNameUnic.Length + core::mem::size_of::<u16>() as u16;

            core::ptr::copy_nonoverlapping(
                (*BaseDllName).Buffer as *const u8,
                p1 as *mut u8,
                (*BaseDllName).MaximumLength as usize,
            );

            ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
            ObjectAttributes.RootDirectory = LdrpKnownDllObjectDirectory;
            ObjectAttributes.ObjectName = &mut DllNameUnic;
            ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
            ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
            ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

            Status = NtOpenSection(
                &mut Section,
                SECTION_MAP_READ | SECTION_MAP_EXECUTE | SECTION_MAP_WRITE,
                &mut ObjectAttributes,
            );
            if !NT_SUCCESS(Status) {
                if Status == STATUS_OBJECT_NAME_NOT_FOUND {
                    Status = STATUS_SUCCESS;
                }
                break 'failure;
            }

            *SectionHandle = Section;
            return STATUS_SUCCESS;
        }
    }

    if !Section.is_null() {
        NtClose(Section);
    }

    if !(*BaseDllName).Buffer.is_null() {
        RtlFreeHeap(LdrpHeap, 0, (*BaseDllName).Buffer as *mut _);
    }
    if !(*FullDllName).Buffer.is_null() {
        RtlFreeHeap(LdrpHeap, 0, (*FullDllName).Buffer as *mut _);
    }

    Status
}
#[no_mangle]
pub unsafe extern "system" fn LdrpSetProtection(ViewBase: PVOID, Restore: BOOLEAN) -> NTSTATUS {
    let NtHeaders: PIMAGE_NT_HEADERS = RtlImageNtHeader(ViewBase);
    if NtHeaders.is_null() {
        return STATUS_INVALID_IMAGE_FORMAT;
    }

    let mut Section: PIMAGE_SECTION_HEADER = (NtHeaders as usize
        + core::mem::offset_of!(IMAGE_NT_HEADERS, OptionalHeader)
        + (*NtHeaders).FileHeader.SizeOfOptionalHeader as usize)
        as PIMAGE_SECTION_HEADER;

    let mut NewProtection: ULONG;
    let mut OldProtection: ULONG = 0;
    let mut Status: NTSTATUS;

    for _i in 0..(*NtHeaders).FileHeader.NumberOfSections as ULONG {
        if (*Section).SizeOfRawData != 0
            && ((*Section).Characteristics & IMAGE_SCN_MEM_WRITE) == 0
        {
            if Restore != 0 {
                if ((*Section).Characteristics & IMAGE_SCN_MEM_EXECUTE) != 0 {
                    NewProtection = PAGE_EXECUTE;
                } else {
                    NewProtection = PAGE_READONLY;
                }

                if ((*Section).Characteristics & IMAGE_SCN_MEM_NOT_CACHED) != 0 {
                    NewProtection |= PAGE_NOCACHE;
                }
            } else {
                NewProtection = PAGE_READWRITE;
            }

            let mut SectionBase: PVOID =
                (ViewBase as ULONG_PTR + (*Section).VirtualAddress as ULONG_PTR) as PVOID;
            let mut SectionSize: SIZE_T = (*Section).SizeOfRawData as SIZE_T;
            if SectionSize != 0 {
                Status = ZwProtectVirtualMemory(
                    -1isize as HANDLE,
                    &mut SectionBase,
                    &mut SectionSize,
                    NewProtection,
                    &mut OldProtection,
                );
                if !NT_SUCCESS(Status) {
                    return Status;
                }
            }
        }

        Section = Section.add(1);
    }

    if Restore != 0 {
        ZwFlushInstructionCache(-1isize as HANDLE, core::ptr::null_mut(), 0);
    }
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn LdrpMapDll(
    SearchPath: PWSTR,
    DllPath2: PWSTR,
    DllName: PWSTR,
    DllCharacteristics: PULONG,
    Static: BOOLEAN,
    Redirect: BOOLEAN,
    DataTableEntry: *mut PLDR_DATA_TABLE_ENTRY,
) -> NTSTATUS {
    let Teb: PTEB = NtCurrentTeb();
    let Peb: PPEB = (*Teb).ProcessEnvironmentBlock as PPEB;
    let mut p1: PWCHAR = DllName;
    let mut TempChar: WCHAR;
    let mut KnownDll: BOOLEAN = 0u8;
    let mut FullDllName: _UNICODE_STRING = core::mem::zeroed();
    let mut BaseDllName: _UNICODE_STRING = core::mem::zeroed();
    let mut SectionHandle: HANDLE = core::ptr::null_mut();
    let DllHandle: HANDLE = core::ptr::null_mut();
    let mut NtPathDllName: _UNICODE_STRING = core::mem::zeroed();
    let mut HardErrorParameters: [ULONG_PTR; 2] = [0; 2];
    let mut HardErrorDllName: _UNICODE_STRING = core::mem::zeroed();
    let mut HardErrorDllPath: _UNICODE_STRING = core::mem::zeroed();
    let mut Response: ULONG = 0;
    let mut ViewSize: SIZE_T = 0;
    let mut ViewBase: PVOID = core::ptr::null_mut();
    let mut ArbitraryUserPointer: PVOID;
    let NtHeaders: PIMAGE_NT_HEADERS;
    let mut HardErrorStatus: NTSTATUS;
    let mut Status: NTSTATUS;
    let mut OverlapDllFound: BOOLEAN = 0u8;
    let ImageBase: ULONG_PTR;
    let ImageEnd: ULONG_PTR;
    let ListHead: PLIST_ENTRY;
    let mut NextEntry: PLIST_ENTRY;
    let mut CandidateEntry: PLDR_DATA_TABLE_ENTRY;
    let mut LdrEntry: PLDR_DATA_TABLE_ENTRY;
    let mut CandidateBase: ULONG_PTR;
    let mut CandidateEnd: ULONG_PTR;
    let mut OverlapDll: _UNICODE_STRING = core::mem::zeroed();
    let mut RelocatableDll: BOOLEAN = 1u8;
    let mut IllegalDll: _UNICODE_STRING = core::mem::zeroed();
    let RelocData: PVOID;
    let mut RelocDataSize: ULONG = 0;

    if ShowSnaps != 0 {
        // DPRINT removed
    }

    'skip_check: {
        if !LdrpKnownDllObjectDirectory.is_null() && Redirect == 0u8 {
            while *p1 != 0 {
                TempChar = *p1;
                p1 = p1.add(1);
                if TempChar == b'\\' as WCHAR || TempChar == b'/' as WCHAR {
                    break 'skip_check;
                }
            }

            Status = LdrpCheckForKnownDll(
                DllName,
                core::ptr::addr_of_mut!(FullDllName),
                core::ptr::addr_of_mut!(BaseDllName),
                core::ptr::addr_of_mut!(SectionHandle),
            );

            if !NT_SUCCESS(Status) && Status != STATUS_DLL_NOT_FOUND {
                DbgPrintEx(
                    DPFLTR_LDR_ID as ULONG,
                    DPFLTR_ERROR_LEVEL as ULONG,
                    b"LDR: %s - call to LdrpCheckForKnownDll(\"%ws\", ...) failed with status %x\n\0"
                        .as_ptr() as PCSTR,
                    b"LdrpMapDll\0".as_ptr() as PCSTR,
                    DllName,
                    Status,
                );

                return Status;
            }
        }
    }

    if SectionHandle.is_null() {
        if LdrpResolveDllName(
            SearchPath,
            DllName,
            core::ptr::addr_of_mut!(FullDllName),
            core::ptr::addr_of_mut!(BaseDllName),
        ) != 0
        {
            if ShowSnaps != 0 {
                // DPRINT removed
            }

            if RtlDosPathNameToNtPathName_U(
                FullDllName.Buffer,
                core::ptr::addr_of_mut!(NtPathDllName),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            ) == 0u8
            {
                return STATUS_OBJECT_PATH_SYNTAX_BAD;
            }

            Status = LdrpCreateDllSection(
                core::ptr::addr_of_mut!(NtPathDllName),
                DllHandle,
                DllCharacteristics,
                core::ptr::addr_of_mut!(SectionHandle),
            );

            RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathDllName.Buffer as *mut _);

            if !NT_SUCCESS(Status) {
                LdrpFreeUnicodeString(core::ptr::addr_of_mut!(FullDllName));
                LdrpFreeUnicodeString(core::ptr::addr_of_mut!(BaseDllName));
                return Status;
            }
        } else {
            // ARM64: don't raise a hard error for missing DLLs — delay imports
            // become regular imports on arm64 (llvm-dlltool limitation), so many
            // DLLs appear as hard imports but are never actually called. Let the
            // caller skip them. (Restores the #ifdef _M_ARM64 branch in
            // ldrutils.c:1127 that the translation flattened to the desktop path.)
            if Static != 0 && !cfg!(target_arch = "aarch64") {
                RtlInitUnicodeString(core::ptr::addr_of_mut!(HardErrorDllName), DllName);
                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(HardErrorDllPath),
                    if !DllPath2.is_null() {
                        DllPath2
                    } else {
                        LdrpDefaultPath.Buffer
                    },
                );

                HardErrorParameters[0] = core::ptr::addr_of!(HardErrorDllName) as ULONG_PTR;
                HardErrorParameters[1] = core::ptr::addr_of!(HardErrorDllPath) as ULONG_PTR;

                NtRaiseHardError(
                    STATUS_DLL_NOT_FOUND,
                    2,
                    0x00000003,
                    HardErrorParameters.as_mut_ptr(),
                    OptionOk,
                    core::ptr::addr_of_mut!(Response),
                );

                if LdrpInLdrInit != 0u8 {
                    LdrpFatalHardErrorCount += 1;
                }
            }

            return STATUS_DLL_NOT_FOUND;
        }
    } else {
        KnownDll = 1u8;
    }

    ArbitraryUserPointer = (*Teb).NtTib.ArbitraryUserPointer;
    (*Teb).NtTib.ArbitraryUserPointer = FullDllName.Buffer as PVOID;

    ViewBase = core::ptr::null_mut();
    ViewSize = 0;
    Status = NtMapViewOfSection(
        SectionHandle,
        -1isize as HANDLE,
        core::ptr::addr_of_mut!(ViewBase),
        0,
        0,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(ViewSize),
        ViewShare,
        0,
        PAGE_READWRITE,
    );

    (*Teb).NtTib.ArbitraryUserPointer = ArbitraryUserPointer;

    if !NT_SUCCESS(Status) {
        NtClose(SectionHandle);
        return Status;
    }

    NtHeaders = RtlImageNtHeader(ViewBase);
    if NtHeaders.is_null() {
        NtUnmapViewOfSection(-1isize as HANDLE, ViewBase);
        NtClose(SectionHandle);
        return STATUS_INVALID_IMAGE_FORMAT;
    }

    LdrEntry = LdrpAllocateDataTableEntry(ViewBase);
    if LdrEntry.is_null() {
        NtUnmapViewOfSection(-1isize as HANDLE, ViewBase);
        NtClose(SectionHandle);
        return STATUS_NO_MEMORY;
    }

    (*LdrEntry).Flags = if Static != 0 { LDRP_STATIC_LINK } else { 0 };
    if Redirect != 0 {
        (*LdrEntry).Flags |= LDRP_REDIRECTED;
    }
    (*LdrEntry).LoadCount = 0;
    (*LdrEntry).FullDllName = FullDllName;
    (*LdrEntry).BaseDllName = BaseDllName;
    (*LdrEntry).EntryPoint = LdrpFetchAddressOfEntryPoint((*LdrEntry).DllBase);

    if ShowSnaps != 0 {
        // DPRINT removed
    }

    LdrpInsertMemoryTableEntry(LdrEntry);

    LdrpSendDllNotifications(LdrEntry, LDR_DLL_NOTIFICATION_REASON_LOADED);

    if Status == STATUS_IMAGE_MACHINE_TYPE_MISMATCH {
        let ImageNtHeader: PIMAGE_NT_HEADERS =
            RtlImageNtHeader((*Peb).ImageBaseAddress);

        HardErrorStatus = STATUS_SUCCESS;
        Response = ResponseCancel as ULONG;

        if (*ImageNtHeader).OptionalHeader.MajorSubsystemVersion <= 3 {
            (*LdrEntry).EntryPoint = core::ptr::null_mut();
            HardErrorParameters[0] = core::ptr::addr_of!(FullDllName) as ULONG_PTR;

            HardErrorStatus = ZwRaiseHardError(
                STATUS_IMAGE_MACHINE_TYPE_MISMATCH,
                1,
                1,
                HardErrorParameters.as_mut_ptr(),
                OptionOkCancel,
                core::ptr::addr_of_mut!(Response),
            );
        }

        if NT_SUCCESS(HardErrorStatus) && Response == ResponseCancel as ULONG {
            RemoveEntryList(core::ptr::addr_of_mut!((*LdrEntry).InLoadOrderLinks));
            RemoveEntryList(core::ptr::addr_of_mut!((*LdrEntry).InMemoryOrderLinks));
            RemoveEntryList(core::ptr::addr_of_mut!((*LdrEntry).__bindgen_anon_1.HashLinks));

            RtlFreeHeap(LdrpHeap, 0, LdrEntry as *mut _);

            NtUnmapViewOfSection(-1isize as HANDLE, ViewBase);
            NtClose(SectionHandle);

            if (*ImageNtHeader).OptionalHeader.MajorSubsystemVersion <= 3 {
                if LdrpInLdrInit != 0u8 {
                    LdrpFatalHardErrorCount += 1;
                }
            }

            return STATUS_INVALID_IMAGE_FORMAT;
        }
    } else {
        if ((*NtHeaders).FileHeader.Characteristics as ULONG & IMAGE_FILE_DLL) != 0 {
            (*LdrEntry).Flags |= LDRP_IMAGE_DLL;
        }

        if ((*LdrEntry).Flags & LDRP_IMAGE_DLL) == 0 {
            (*LdrEntry).EntryPoint = core::ptr::null_mut();
        }
    }

    *DataTableEntry = LdrEntry;

    if Status == STATUS_IMAGE_NOT_AT_BASE {
        (*LdrEntry).Flags |= LDRP_IMAGE_NOT_AT_BASE;

        ImageBase = (*NtHeaders).OptionalHeader.ImageBase as ULONG_PTR;
        ImageEnd = ImageBase + ViewSize;

        ListHead = core::ptr::addr_of_mut!((*(*Peb).Ldr).InLoadOrderModuleList);
        NextEntry = (*ListHead).Flink;
        while NextEntry != ListHead {
            CandidateEntry = (NextEntry as usize
                - core::mem::offset_of!(LDR_DATA_TABLE_ENTRY, InLoadOrderLinks))
                as PLDR_DATA_TABLE_ENTRY;
            NextEntry = (*NextEntry).Flink;

            CandidateBase = (*CandidateEntry).DllBase as ULONG_PTR;
            CandidateEnd = CandidateBase + (*CandidateEntry).SizeOfImage as ULONG_PTR;

            if (*CandidateEntry).InMemoryOrderLinks.Flink.is_null() {
                continue;
            }

            if (ImageBase >= CandidateBase && ImageBase <= CandidateEnd)
                || (ImageEnd >= CandidateBase && ImageEnd <= CandidateEnd)
                || (CandidateBase >= ImageBase && CandidateBase <= ImageEnd)
            {
                OverlapDllFound = 1u8;
                OverlapDll = (*CandidateEntry).FullDllName;
                break;
            }
        }

        if OverlapDllFound == 0u8 {
            RtlInitUnicodeString(
                core::ptr::addr_of_mut!(OverlapDll),
                w!("Dynamically Allocated Memory").as_ptr() as PWSTR,
            );
        }

        if ((*LdrEntry).Flags & LDRP_IMAGE_DLL) != 0 {
            'no_reloc_needed: {
                'do_reloc: {
                    if ((*NtHeaders).FileHeader.Characteristics as ULONG
                        & IMAGE_FILE_RELOCS_STRIPPED)
                        == 0
                    {
                        RelocData = RtlImageDirectoryEntryToData(
                            ViewBase,
                            1u8,
                            IMAGE_DIRECTORY_ENTRY_BASERELOC as USHORT,
                            core::ptr::addr_of_mut!(RelocDataSize),
                        );

                        if RelocData.is_null() && RelocDataSize == 0 {
                            break 'no_reloc_needed;
                        }
                    }

                    RtlInitUnicodeString(
                        core::ptr::addr_of_mut!(IllegalDll),
                        w!("user32.dll").as_ptr() as PWSTR,
                    );
                    if RtlEqualUnicodeString(
                        core::ptr::addr_of!(BaseDllName),
                        core::ptr::addr_of!(IllegalDll),
                        1u8,
                    ) != 0u8
                    {
                        RelocatableDll = 0u8;
                    } else {
                        RtlInitUnicodeString(
                            core::ptr::addr_of_mut!(IllegalDll),
                            w!("kernel32.dll").as_ptr() as PWSTR,
                        );
                        if RtlEqualUnicodeString(
                            core::ptr::addr_of!(BaseDllName),
                            core::ptr::addr_of!(IllegalDll),
                            1u8,
                        ) != 0u8
                        {
                            RelocatableDll = 0u8;
                        }
                    }

                    if KnownDll != 0u8 && RelocatableDll == 0u8 {
                        HardErrorParameters[0] = core::ptr::addr_of!(IllegalDll) as ULONG_PTR;
                        HardErrorParameters[1] = core::ptr::addr_of!(OverlapDll) as ULONG_PTR;

                        ZwRaiseHardError(
                            STATUS_ILLEGAL_DLL_RELOCATION,
                            2,
                            3,
                            HardErrorParameters.as_mut_ptr(),
                            OptionOk,
                            core::ptr::addr_of_mut!(Response),
                        );

                        if LdrpInLdrInit != 0u8 {
                            LdrpFatalHardErrorCount += 1;
                        }

                        Status = STATUS_CONFLICTING_ADDRESSES;
                        break 'do_reloc;
                    }

                    Status = LdrpSetProtection(ViewBase, 0u8);

                    if NT_SUCCESS(Status) {
                        // bindgen types this NTSTATUS-returning fn as ULONG; cast at the boundary.
                        Status = LdrRelocateImageWithBias(
                            ViewBase,
                            0i64,
                            core::ptr::null_mut(),
                            STATUS_SUCCESS as ULONG,
                            STATUS_CONFLICTING_ADDRESSES as ULONG,
                            STATUS_INVALID_IMAGE_FORMAT as ULONG,
                        ) as NTSTATUS;

                        if NT_SUCCESS(Status) {
                            ArbitraryUserPointer = (*Teb).NtTib.ArbitraryUserPointer;
                            (*Teb).NtTib.ArbitraryUserPointer = FullDllName.Buffer as PVOID;

                            (*Teb).NtTib.ArbitraryUserPointer = ArbitraryUserPointer;

                            Status = LdrpSetProtection(ViewBase, 1u8);
                        }
                    }
                }

                if !NT_SUCCESS(Status) {
                    RemoveEntryList(core::ptr::addr_of_mut!((*LdrEntry).InLoadOrderLinks));
                    RemoveEntryList(core::ptr::addr_of_mut!((*LdrEntry).InMemoryOrderLinks));
                    RemoveEntryList(core::ptr::addr_of_mut!((*LdrEntry).__bindgen_anon_1.HashLinks));

                    NtUnmapViewOfSection(-1isize as HANDLE, ViewBase);
                    LdrEntry = core::ptr::null_mut();
                }

                if ShowSnaps != 0 {
                    // DPRINT removed
                }
            }
        } else {
            Status = STATUS_SUCCESS;

            ArbitraryUserPointer = (*Teb).NtTib.ArbitraryUserPointer;
            (*Teb).NtTib.ArbitraryUserPointer = FullDllName.Buffer as PVOID;

            (*Teb).NtTib.ArbitraryUserPointer = ArbitraryUserPointer;

            if ShowSnaps != 0 {
                // DPRINT removed
            }
        }
    }

    if LdrpNumberOfProcessors > 1
        && !LdrEntry.is_null()
        && ((*LdrEntry).Flags & LDRP_IMAGE_DLL) != 0
    {
        LdrpValidateImageForMp(LdrEntry);
    }

    NtClose(SectionHandle);
    Status
}
#[no_mangle]
pub unsafe extern "system" fn LdrpAllocateDataTableEntry(BaseAddress: PVOID) -> PLDR_DATA_TABLE_ENTRY {
    let mut LdrEntry: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
    let NtHeader: PIMAGE_NT_HEADERS = RtlImageNtHeader(BaseAddress);

    if !NtHeader.is_null() {
        LdrEntry = RtlAllocateHeap(
            LdrpHeap,
            HEAP_ZERO_MEMORY,
            core::mem::size_of::<LDR_DATA_TABLE_ENTRY>() as SIZE_T,
        ) as PLDR_DATA_TABLE_ENTRY;

        if !LdrEntry.is_null() {
            (*LdrEntry).DllBase = BaseAddress;
            (*LdrEntry).SizeOfImage = (*NtHeader).OptionalHeader.SizeOfImage;
            (*LdrEntry).__bindgen_anon_2.TimeDateStamp = (*NtHeader).FileHeader.TimeDateStamp;
            (*LdrEntry).PatchInformation = core::ptr::null_mut();
        }
    }

    LdrEntry
}

#[no_mangle]
pub unsafe extern "system" fn LdrpInsertMemoryTableEntry(LdrEntry: PLDR_DATA_TABLE_ENTRY) {
    let PebData: PPEB_LDR_DATA = (*NtCurrentPeb()).Ldr;
    let i: u32;

    i = LDR_GET_HASH_ENTRY(*(*LdrEntry).BaseDllName.Buffer);
    InsertTailList(
        core::ptr::addr_of_mut!(LdrpHashTable[i as usize]),
        core::ptr::addr_of_mut!((*LdrEntry).__bindgen_anon_1.HashLinks),
    ); // (already correct accessor)

    InsertTailList(
        core::ptr::addr_of_mut!((*PebData).InLoadOrderModuleList),
        core::ptr::addr_of_mut!((*LdrEntry).InLoadOrderLinks),
    );
    InsertTailList(
        core::ptr::addr_of_mut!((*PebData).InMemoryOrderModuleList),
        core::ptr::addr_of_mut!((*LdrEntry).InMemoryOrderLinks),
    );
}
#[no_mangle]
pub unsafe extern "system" fn LdrpFinalizeAndDeallocateDataTableEntry(Entry: PLDR_DATA_TABLE_ENTRY) {
    if !(*Entry).EntryPointActivationContext.is_null()
        && (*Entry).EntryPointActivationContext != -1isize as *mut _
    {
        RtlReleaseActivationContext((*Entry).EntryPointActivationContext);
        (*Entry).EntryPointActivationContext = -1isize as *mut _;
    }

    if !(*Entry).FullDllName.Buffer.is_null() {
        LdrpFreeUnicodeString(core::ptr::addr_of_mut!((*Entry).FullDllName));
    }

    RtlFreeHeap(LdrpHeap, 0, Entry as *mut _);
}

#[no_mangle]
pub unsafe extern "system" fn LdrpCheckForLoadedDllHandle(
    Base: PVOID,
    LdrEntry: *mut PLDR_DATA_TABLE_ENTRY,
) -> BOOLEAN {
    let mut Current: PLDR_DATA_TABLE_ENTRY;
    let ListHead: PLIST_ENTRY;
    let mut Next: PLIST_ENTRY;

    if !LdrpLoadedDllHandleCache.is_null()
        && (*LdrpLoadedDllHandleCache).DllBase == Base
    {
        *LdrEntry = LdrpLoadedDllHandleCache;
        return 1u8;
    }

    ListHead = core::ptr::addr_of_mut!(
        (*(*NtCurrentPeb()).Ldr).InLoadOrderModuleList
    );
    Next = (*ListHead).Flink;
    while Next != ListHead {
        Current = (Next as *mut u8).offset(
            -(core::mem::offset_of!(_LDR_DATA_TABLE_ENTRY, InLoadOrderLinks) as isize)
        ) as PLDR_DATA_TABLE_ENTRY;

        if !(*Current).InMemoryOrderLinks.Flink.is_null() && Base == (*Current).DllBase {
            LdrpLoadedDllHandleCache = Current;
            *LdrEntry = Current;
            return 1u8;
        }

        Next = (*Next).Flink;
    }

    0u8
}
#[no_mangle]
pub unsafe extern "system" fn LdrpResolveFullName(
    OriginalName: PUNICODE_STRING,
    PathName: PUNICODE_STRING,
    FullPathName: PUNICODE_STRING,
    ExpandedName: *mut PUNICODE_STRING,
) -> i32 {
    const STATUS_NAME_TOO_LONG: i32 = 0xC0000106u32 as i32;
    const UNICODE_STRING_MAX_BYTES: u32 = 0xFFFE;
    const DPFLTR_LDR_ID: u32 = 85;

    let mut Status: i32 = STATUS_SUCCESS;
    let mut Length: u32;

    if ShowSnaps != 0 {
        DbgPrintEx(
            DPFLTR_LDR_ID,
            DPFLTR_ERROR_LEVEL,
            b"LDR: %s - Expanding full name of %wZ\n\0".as_ptr() as *const i8,
            b"LdrpResolveFullName\0".as_ptr() as *const i8,
            OriginalName,
        );
    }

    'quickie: {
        Length = 0;

        if Length == 0 || Length > UNICODE_STRING_MAX_BYTES {
            Status = STATUS_NAME_TOO_LONG;
            break 'quickie;
        }

        if Length <= (*PathName).Length as u32 {
            *ExpandedName = PathName;
            (*PathName).Length = Length as u16;
            break 'quickie;
        }

        Status = LdrpAllocateUnicodeString(FullPathName, Length - core::mem::size_of::<u16>() as u32);
        if !NT_SUCCESS(Status) {
            break 'quickie;
        }

        Length = 0;

        if Length == 0 || Length > (*FullPathName).Length as u32 {
            LdrpFreeUnicodeString(FullPathName);
            Status = STATUS_NAME_TOO_LONG;
        } else {
            *ExpandedName = FullPathName;
            (*FullPathName).Length = Length as u16;
        }
    }

    if ShowSnaps != 0 {
        if NT_SUCCESS(Status) {
            DbgPrintEx(
                DPFLTR_LDR_ID,
                DPFLTR_ERROR_LEVEL,
                b"LDR: %s - Expanded to %wZ\n\0".as_ptr() as *const i8,
                b"LdrpResolveFullName\0".as_ptr() as *const i8,
                *ExpandedName,
            );
        } else {
            DbgPrintEx(
                DPFLTR_LDR_ID,
                DPFLTR_ERROR_LEVEL,
                b"LDR: %s - Failed to expand %wZ; 0x%08x\n\0".as_ptr() as *const i8,
                b"LdrpResolveFullName\0".as_ptr() as *const i8,
                OriginalName,
                Status,
            );
        }
    }

    if !NT_SUCCESS(Status) {
        *ExpandedName = core::ptr::null_mut();
    }

    Status
}
#[no_mangle]
pub unsafe extern "system" fn LdrpSearchPath(
    SearchPath: *mut *mut u16,
    DllName: *mut u16,
    PathName: PUNICODE_STRING,
    FullPathName: PUNICODE_STRING,
    ExpandedName: *mut PUNICODE_STRING,
) -> i32 {
    const STATUS_SUCCESS: i32 = 0;
    const STATUS_NAME_TOO_LONG: i32 = 0xC0000106u32 as i32;
    const STATUS_NOT_FOUND: i32 = 0xC0000225u32 as i32;
    const DPFLTR_LDR_ID: u32 = 85;

    let mut TryAgain: u8 = 0;
    let mut ActualSearchPath: *mut u16 = *SearchPath;
    let mut TestName: _UNICODE_STRING = core::mem::zeroed();
    let mut Status: i32;
    let mut Buffer: *mut u16;
    let mut BufEnd: *mut u16 = core::ptr::null_mut();
    let mut Length: u32 = 0;
    let mut p: u16;

    'quickie: {
        if ActualSearchPath.is_null() {
            *SearchPath = LdrpDefaultPath.Buffer;
        }

        if ShowSnaps != 0 {
            DbgPrintEx(
                DPFLTR_LDR_ID,
                DPFLTR_ERROR_LEVEL,
                b"LDR: %s - Looking for %ws in %ws\n\0".as_ptr() as *const i8,
                b"LdrpSearchPath\0".as_ptr() as *const i8,
                DllName,
                *SearchPath,
            );
        }

        if RtlDetermineDosPathNameType_U(DllName) != RtlPathTypeRelative {
            Status = RtlInitUnicodeStringEx(&mut TestName, DllName);
            if !NT_SUCCESS(Status) {
                break 'quickie;
            }

            Status = LdrpResolveFullName(&mut TestName, PathName, FullPathName, ExpandedName);
            break 'quickie;
        }

        Length += wcslen(DllName) as u32 + 1;
        if Length > UNICODE_STRING_MAX_CHARS {
            Status = STATUS_NAME_TOO_LONG;
            break 'quickie;
        }

        Buffer = RtlAllocateHeap(
            RtlGetProcessHeap(),
            0,
            ((Length as usize) * core::mem::size_of::<u16>()) as SIZE_T,
        ) as *mut u16;
        if Buffer.is_null() {
            Status = STATUS_NO_MEMORY;
            break 'quickie;
        }

        Status = STATUS_NOT_FOUND;
        BufEnd = Buffer;

        loop {
            p = *ActualSearchPath;
            if p == 0 || p == b';' as u16 {
                if ShowSnaps != 0 {
                    DbgPrintEx(
                        DPFLTR_LDR_ID,
                        DPFLTR_ERROR_LEVEL,
                        b"LDR: %s - Looking for %ws\n\0".as_ptr() as *const i8,
                        b"LdrpSearchPath\0".as_ptr() as *const i8,
                        Buffer,
                    );
                }

                let diff = BufEnd.offset_from(Buffer) as usize;
                TestName.Length = ALIGN_DOWN(diff, core::mem::size_of::<u16>()) as u16;

                {
                    TestName.MaximumLength = (ALIGN_DOWN(diff, core::mem::size_of::<u16>())
                        as u16)
                        + core::mem::size_of::<u16>() as u16;
                    TestName.Buffer = RtlReAllocateHeap(
                        RtlGetProcessHeap(),
                        0,
                        Buffer as *mut core::ffi::c_void,
                        TestName.MaximumLength as SIZE_T,
                    ) as *mut u16;
                    if TestName.Buffer.is_null() {
                        TestName.Buffer = Buffer;
                    } else {
                        Buffer = TestName.Buffer;
                    }

                    *SearchPath = ActualSearchPath;
                    ActualSearchPath = ActualSearchPath.add(1);
                    Status = LdrpResolveFullName(
                        &mut TestName,
                        PathName,
                        FullPathName,
                        ExpandedName,
                    );
                    break;
                }
            } else {
                *BufEnd = p;
                BufEnd = BufEnd.add(1);
            }

            if *ActualSearchPath == 0 {
                TryAgain = 1;
            }

            ActualSearchPath = ActualSearchPath.add(1);

            if TryAgain != 0 {
                break;
            }
        }

        if !Buffer.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer as *mut core::ffi::c_void);
        }
    }

    if !NT_SUCCESS(Status) {
        *ExpandedName = core::ptr::null_mut();
    }

    if ShowSnaps != 0 {
        if NT_SUCCESS(Status) {
            DbgPrintEx(
                DPFLTR_LDR_ID,
                DPFLTR_ERROR_LEVEL,
                b"LDR: %s - Returning %wZ\n\0".as_ptr() as *const i8,
                b"LdrpSearchPath\0".as_ptr() as *const i8,
                *ExpandedName,
            );
        } else {
            DbgPrintEx(
                DPFLTR_LDR_ID,
                DPFLTR_ERROR_LEVEL,
                b"LDR: %s -  Unable to locate %ws in %ws: 0x%08x\n\0".as_ptr() as *const i8,
                b"LdrpSearchPath\0".as_ptr() as *const i8,
                DllName,
                ActualSearchPath,
                Status,
            );
        }
    }

    Status
}

unsafe fn unicode_has_path_separator(Name: PUNICODE_STRING) -> bool {
    let n = (*Name).Length as usize / core::mem::size_of::<WCHAR>();
    (0..n).any(|i| {
        let c = *(*Name).Buffer.add(i);
        c == b'\\' as u16 || c == b'/' as u16
    })
}

#[no_mangle]
pub unsafe extern "system" fn LdrpCheckForLoadedDll(
    DllPath: PWSTR,
    DllName: PUNICODE_STRING,
    mut Flag: BOOLEAN,
    RedirectedDll: BOOLEAN,
    LdrEntry: *mut PLDR_DATA_TABLE_ENTRY,
) -> BOOLEAN {
    let HashIndex: ULONG;
    let mut ListHead: PLIST_ENTRY;
    let mut ListEntry: PLIST_ENTRY;
    let mut CurEntry: PLDR_DATA_TABLE_ENTRY;
    let mut FullPath: BOOLEAN = 0u8;
    let mut wc: PWCHAR;
    let mut NameBuf: [WCHAR; 266] = [0; 266];
    let mut FullDllName: _UNICODE_STRING = core::mem::zeroed();
    let mut NtPathName: _UNICODE_STRING = core::mem::zeroed();
    let mut Length: ULONG;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut FileHandle: HANDLE = core::ptr::null_mut();
    let mut SectionHandle: HANDLE = core::ptr::null_mut();
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut ViewBase: PVOID = core::ptr::null_mut();
    let mut ViewSize: SIZE_T = 0;
    let mut NtHeader: PIMAGE_NT_HEADERS = core::ptr::null_mut();
    let mut NtHeader2: PIMAGE_NT_HEADERS;

    if (*DllName).Buffer.is_null() || *(*DllName).Buffer == 0 {
        return 0u8;
    }

    // An api-set redirect here yields the BARE host name ("kernel32.dll", found by the search
    // path; see LdrpApplyFileNameRedirection), not the absolute path stock ReactOS builds. Taken
    // as a full path it matched no FullDllName, so every api-set load of an already-loaded host
    // mapped a second copy: ucrtbase's LCMapStringEx thunk loaded kernel32 twice, the copy's
    // DllMain re-pointed LdrpManifestProberRoutine into it, and the next DLL load called
    // through that pointer after the copy was gone (setup.exe, amd64). Look it up by base name.
    let RedirectedDll: BOOLEAN = if RedirectedDll != 0 && !unicode_has_path_separator(DllName) {
        0u8
    } else {
        RedirectedDll
    };

    'lookinhash: loop {
        if Flag != 0 && RedirectedDll == 0 {
            HashIndex = LDR_GET_HASH_ENTRY(*(*DllName).Buffer);

            ListHead = core::ptr::addr_of_mut!(LdrpHashTable[HashIndex as usize]);
            ListEntry = (*ListHead).Flink;
            while ListEntry != ListHead {
                CurEntry = (ListEntry as *mut u8)
                    .offset(-(FIELD_OFFSET_LDR_HashLinks() as isize))
                    as PLDR_DATA_TABLE_ENTRY;

                if RtlEqualUnicodeString(DllName, core::ptr::addr_of_mut!((*CurEntry).BaseDllName), 1u8) != 0 {
                    *LdrEntry = CurEntry;
                    return 1u8;
                }

                ListEntry = (*ListEntry).Flink;
            }

            return 0u8;
        }

        if RedirectedDll != 0 {
            FullPath = 1u8;
            FullDllName = *DllName;
        } else {
            wc = (*DllName).Buffer;
            while *wc != 0 {
                if *wc == b'\\' as u16 || *wc == b'/' as u16 {
                    FullPath = 1u8;

                    FullDllName.Buffer = NameBuf.as_mut_ptr();

                    Length = RtlDosSearchPath_U(
                        if !DllPath.is_null() { DllPath } else { LdrpDefaultPath.Buffer },
                        (*DllName).Buffer,
                        core::ptr::null_mut(),
                        (core::mem::size_of_val(&NameBuf) - core::mem::size_of::<u16>()) as ULONG,
                        FullDllName.Buffer,
                        core::ptr::null_mut(),
                    );

                    if Length == 0 || Length > (core::mem::size_of_val(&NameBuf) - core::mem::size_of::<u16>()) as ULONG {
                        if ShowSnaps != 0 {
                            DbgPrint(b"LDR: LdrpCheckForLoadedDll - Unable To Locate %wZ: 0x%08x\n\0".as_ptr() as PCHAR,
                                DllName, Length);
                        }
                    }

                    FullDllName.Length = Length as USHORT;
                    FullDllName.MaximumLength = FullDllName.Length + core::mem::size_of::<u16>() as USHORT;
                    break;
                }

                wc = wc.add(1);
            }
        }

        if FullPath == 0 {
            Flag = 1u8;
            continue 'lookinhash;
        }

        break;
    }

    ListHead = core::ptr::addr_of_mut!((*(*NtCurrentPeb()).Ldr).InLoadOrderModuleList);
    ListEntry = (*ListHead).Flink;
    while ListEntry != ListHead {
        CurEntry = (ListEntry as *mut u8)
            .offset(-(FIELD_OFFSET_LDR_InLoadOrderLinks() as isize))
            as PLDR_DATA_TABLE_ENTRY;
        ListEntry = (*ListEntry).Flink;

        if (*CurEntry).InMemoryOrderLinks.Flink.is_null() {
            continue;
        }

        if RtlEqualUnicodeString(
            core::ptr::addr_of_mut!(FullDllName),
            core::ptr::addr_of_mut!((*CurEntry).FullDllName),
            1u8,
        ) != 0
        {
            *LdrEntry = CurEntry;
            return 1u8;
        }
    }

    if RtlDosPathNameToNtPathName_U(
        FullDllName.Buffer,
        core::ptr::addr_of_mut!(NtPathName),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) == 0
    {
        return 0u8;
    }

    InitializeObjectAttributes_inline(
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::addr_of_mut!(NtPathName),
        OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    Status = NtOpenFile(
        core::ptr::addr_of_mut!(FileHandle),
        SYNCHRONIZE | FILE_EXECUTE,
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::addr_of_mut!(Iosb),
        FILE_SHARE_READ | FILE_SHARE_DELETE,
        FILE_NON_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT,
    );

    RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathName.Buffer as PVOID);

    if !NT_SUCCESS(Status) {
        return 0u8;
    }

    Status = NtCreateSection(
        core::ptr::addr_of_mut!(SectionHandle),
        SECTION_MAP_READ | SECTION_MAP_EXECUTE | SECTION_MAP_WRITE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        PAGE_EXECUTE,
        SEC_COMMIT,
        FileHandle,
    );

    NtClose(FileHandle);

    if !NT_SUCCESS(Status) {
        return 0u8;
    }

    Status = ZwMapViewOfSection(
        SectionHandle,
        NtCurrentProcess(),
        core::ptr::addr_of_mut!(ViewBase),
        0,
        0,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(ViewSize),
        ViewShare,
        0,
        PAGE_EXECUTE,
    );

    NtClose(SectionHandle);

    if !NT_SUCCESS(Status) {
        return 0u8;
    }

    Status = RtlImageNtHeaderEx(0, ViewBase, ViewSize as u64, core::ptr::addr_of_mut!(NtHeader));
    if !NT_SUCCESS(Status) || NtHeader.is_null() {
        NtUnmapViewOfSection(NtCurrentProcess(), ViewBase);
        return 0u8;
    }

    ListHead = core::ptr::addr_of_mut!((*(*NtCurrentPeb()).Ldr).InLoadOrderModuleList);
    ListEntry = (*ListHead).Flink;
    while ListEntry != ListHead {
        CurEntry = (ListEntry as *mut u8)
            .offset(-(FIELD_OFFSET_LDR_InLoadOrderLinks() as isize))
            as PLDR_DATA_TABLE_ENTRY;
        ListEntry = (*ListEntry).Flink;

        if (*CurEntry).InMemoryOrderLinks.Flink.is_null() {
            continue;
        }

        // _SEH2_TRY - DUMMY_PSEH: execute body directly
        if (*CurEntry).__bindgen_anon_2.TimeDateStamp == (*NtHeader).FileHeader.TimeDateStamp
            && (*CurEntry).SizeOfImage == (*NtHeader).OptionalHeader.SizeOfImage
        {
            NtHeader2 = RtlImageNtHeader((*CurEntry).DllBase);
            if RtlCompareMemory(
                NtHeader2 as *const _,
                NtHeader as *const _,
                core::mem::size_of::<IMAGE_NT_HEADERS>() as SIZE_T,
            ) != 0
            {
                Status = ZwAreMappedFilesTheSame((*CurEntry).DllBase, ViewBase);
                if NT_SUCCESS(Status) {
                    *LdrEntry = CurEntry;
                    NtUnmapViewOfSection(NtCurrentProcess(), ViewBase);
                    return 1u8;
                }
            }
        }
    }

    NtUnmapViewOfSection(NtCurrentProcess(), ViewBase);
    0u8
}

#[no_mangle]
pub unsafe extern "system" fn LdrpLoadDll(
    Redirected: BOOLEAN,
    DllPath: PWSTR,
    DllCharacteristics: PULONG,
    DllName: PUNICODE_STRING,
    BaseAddress: *mut PVOID,
    CallInit: BOOLEAN,
) -> NTSTATUS {
    let Peb: PPEB = NtCurrentPeb();
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut p: *const WCHAR;
    let mut GotExtension: BOOLEAN;
    let mut c: WCHAR;
    let mut NameBuffer: [WCHAR; (MAX_PATH + 6) as usize] = [0; (MAX_PATH + 6) as usize];
    let mut RawDllName: _UNICODE_STRING = core::mem::zeroed();
    let mut LdrEntry: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
    let InInit: BOOLEAN = LdrpInLdrInit;

    /* Save the Raw DLL Name */
    if (*DllName).Length as usize >= core::mem::size_of_val(&NameBuffer) {
        return STATUS_NAME_TOO_LONG;
    }
    RtlInitEmptyUnicodeString(
        &mut RawDllName,
        NameBuffer.as_mut_ptr(),
        core::mem::size_of_val(&NameBuffer) as u16,
    );
    RtlCopyUnicodeString(&mut RawDllName, DllName);

    /* Find the extension, if present */
    p = (*DllName).Buffer.offset((*DllName).Length as isize / core::mem::size_of::<WCHAR>() as isize - 1);
    GotExtension = 0u8;
    while p >= (*DllName).Buffer {
        c = *p;
        p = p.offset(-1);
        if c == '.' as WCHAR {
            GotExtension = 1u8;
            break;
        } else if c == '\\' as WCHAR {
            break;
        }
    }

    /* If no extension was found, add the default extension */
    if GotExtension == 0 {
        if ((*DllName).Length as usize + LdrApiDefaultExtension.Length as usize + core::mem::size_of::<WCHAR>())
            >= core::mem::size_of_val(&NameBuffer)
        {
            DbgPrintEx(
                DPFLTR_LDR_ID,
                DPFLTR_ERROR_LEVEL,
                b"LDR: %s - Dll name missing extension; with extension added the name is too long\n   DllName: (@ %p) \"%wZ\"\n   DllName->Length: %u\n\0".as_ptr() as PCHAR,
                b"LdrpLoadDll\0".as_ptr() as PCHAR,
                DllName,
                DllName,
                (*DllName).Length as ULONG,
            );
            return STATUS_NAME_TOO_LONG;
        }

        RtlAppendUnicodeStringToString(&mut RawDllName, &LdrApiDefaultExtension);
    }

    /* Check for init flag and acquire lock */
    if InInit == 0 {
        RtlEnterCriticalSection(&mut LdrpLoaderLock);
    }

    'try_block: {
        /* Show debug message */
        if ShowSnaps != 0 {
            DbgPrint(
                b"LDR: LdrLoadDll, loading %wZ from %ws\n\0".as_ptr() as PCHAR,
                &RawDllName,
                if !DllPath.is_null() { DllPath } else { w!("").as_ptr() as PWSTR },
            );
        }

        /* Check if the DLL is already loaded */
        if LdrpCheckForLoadedDll(DllPath, &mut RawDllName, 0u8, Redirected, &mut LdrEntry) == 0 {
            /* Map it */
            Status = LdrpMapDll(
                DllPath,
                DllPath,
                NameBuffer.as_mut_ptr(),
                DllCharacteristics,
                0u8,
                Redirected,
                &mut LdrEntry,
            );
            if !NT_SUCCESS(Status) {
                break 'try_block;
            }

            /* Check if IMAGE_FILE_EXECUTABLE_IMAGE was provided */
            if !DllCharacteristics.is_null() && (*DllCharacteristics & IMAGE_FILE_EXECUTABLE_IMAGE) != 0 {
                (*LdrEntry).EntryPoint = core::ptr::null_mut();
                (*LdrEntry).Flags &= !LDRP_IMAGE_DLL;
            }

            /* Make sure it's a DLL */
            if ((*LdrEntry).Flags & LDRP_IMAGE_DLL) != 0 {
                /* Check if this is a .NET Image */
                if ((*LdrEntry).Flags & LDRP_COR_IMAGE) == 0 {
                    Status = LdrpWalkImportDescriptor(DllPath, LdrEntry);
                }

                /* Update load count, unless it's locked */
                if (*LdrEntry).LoadCount != 0xFFFF {
                    (*LdrEntry).LoadCount += 1;
                }
                LdrpUpdateLoadCount2(LdrEntry, LDRP_UPDATE_REFCOUNT);

                /* Check if we failed */
                if !NT_SUCCESS(Status) {
                    (*LdrEntry).EntryPoint = core::ptr::null_mut();
                    InsertTailList(
                        core::ptr::addr_of_mut!((*(*Peb).Ldr).InInitializationOrderModuleList),
                        core::ptr::addr_of_mut!((*LdrEntry).InInitializationOrderLinks),
                    );

                    LdrpClearLoadInProgress();

                    if ShowSnaps != 0 {
                        DbgPrint(
                            b"LDR: Unloading %wZ due to error %x walking import descriptors\n\0".as_ptr() as PCHAR,
                            DllName,
                            Status,
                        );
                    }
                    LdrUnloadDll((*LdrEntry).DllBase);

                    break 'try_block;
                }
            } else if (*LdrEntry).LoadCount != 0xFFFF {
                (*LdrEntry).LoadCount += 1;
            }

            /* Insert it into the list */
            InsertTailList(
                core::ptr::addr_of_mut!((*(*Peb).Ldr).InInitializationOrderModuleList),
                core::ptr::addr_of_mut!((*LdrEntry).InInitializationOrderLinks),
            );

            /* If we have to run the entrypoint, make sure the DB is ready */
            if CallInit != 0 && LdrpLdrDatabaseIsSetup != 0 {
                /* Notify Shim Engine */
                if g_ShimsEnabled != 0 {
                    let SE_DllLoaded: Option<unsafe extern "system" fn(PLDR_DATA_TABLE_ENTRY)> =
                        core::mem::transmute(RtlDecodeSystemPointer(g_pfnSE_DllLoaded));
                    (SE_DllLoaded.unwrap())(LdrEntry);
                }

                /* Run the init routine */
                Status = LdrpRunInitializeRoutines(core::ptr::null_mut());
                if !NT_SUCCESS(Status) {
                    if ShowSnaps != 0 {
                        DbgPrint(
                            b"LDR: Unloading %wZ because either its init routine or one of its static imports failed; status = 0x%08lx\n\0".as_ptr() as PCHAR,
                            DllName,
                            Status,
                        );
                    }
                    LdrUnloadDll((*LdrEntry).DllBase);
                }
            } else {
                Status = STATUS_SUCCESS;
            }
        } else {
            /* We were already loaded. Are we a DLL? */
            if ((*LdrEntry).Flags & LDRP_IMAGE_DLL) != 0 && (*LdrEntry).LoadCount != 0xFFFF {
                (*LdrEntry).LoadCount += 1;
                LdrpUpdateLoadCount2(LdrEntry, LDRP_UPDATE_REFCOUNT);
                LdrpClearLoadInProgress();
            } else {
                if (*LdrEntry).LoadCount != 0xFFFF {
                    (*LdrEntry).LoadCount += 1;
                }
            }
        }
    }

    /* FINALLY: Release the lock */
    if InInit == 0 {
        RtlLeaveCriticalSection(&mut LdrpLoaderLock);
    }

    /* Check for success */
    if NT_SUCCESS(Status) {
        *BaseAddress = (*LdrEntry).DllBase;
    } else {
        *BaseAddress = core::ptr::null_mut();
    }

    Status
}
#[no_mangle]
pub unsafe extern "system" fn LdrpClearLoadInProgress() -> ULONG {
    let mut ModulesCount: ULONG = 0;

    let ListHead: PLIST_ENTRY = core::ptr::addr_of_mut!((*(*NtCurrentPeb()).Ldr).InInitializationOrderModuleList);
    let mut Entry: PLIST_ENTRY = (*ListHead).Flink;
    while Entry != ListHead {
        let LdrEntry: PLDR_DATA_TABLE_ENTRY = (Entry as *mut u8)
            .offset(-(core::mem::offset_of!(_LDR_DATA_TABLE_ENTRY, InInitializationOrderLinks) as isize))
            as PLDR_DATA_TABLE_ENTRY;

        (*LdrEntry).Flags &= !LDRP_LOAD_IN_PROGRESS;

        if !(*LdrEntry).EntryPoint.is_null() && ((*LdrEntry).Flags & LDRP_ENTRY_PROCESSED) == 0 {
            ModulesCount += 1;
        }

        Entry = (*Entry).Flink;
    }

    ModulesCount
}
#[no_mangle]
pub unsafe extern "C" fn LdrpGetShimEngineFunction(FunctionName: PCSZ) -> PVOID {
    let mut Function: ANSI_STRING = core::mem::zeroed();
    let mut Address: PVOID = core::ptr::null_mut();
    RtlInitAnsiString(&mut Function, FunctionName);
    let Status = LdrpGetProcedureAddress(g_pShimEngineModule, &mut Function, 0, &mut Address, 0u8);
    if NT_SUCCESS(Status) { Address } else { core::ptr::null_mut() }
}

#[no_mangle]
pub unsafe extern "system" fn LdrpGetShimEngineInterface() {
    let SE_DllLoaded: PVOID = LdrpGetShimEngineFunction(b"SE_DllLoaded\0".as_ptr() as PCSZ);
    let SE_DllUnloaded: PVOID = LdrpGetShimEngineFunction(b"SE_DllUnloaded\0".as_ptr() as PCSZ);
    let SE_InstallBeforeInit: PVOID = LdrpGetShimEngineFunction(b"SE_InstallBeforeInit\0".as_ptr() as PCSZ);
    let SE_InstallAfterInit: PVOID = LdrpGetShimEngineFunction(b"SE_InstallAfterInit\0".as_ptr() as PCSZ);
    let SE_ProcessDying: PVOID = LdrpGetShimEngineFunction(b"SE_ProcessDying\0".as_ptr() as PCSZ);

    if !SE_DllLoaded.is_null() && !SE_DllUnloaded.is_null() && !SE_InstallBeforeInit.is_null() && !SE_InstallAfterInit.is_null() && !SE_ProcessDying.is_null() {
        g_pfnSE_DllLoaded = RtlEncodeSystemPointer(SE_DllLoaded);
        g_pfnSE_DllUnloaded = RtlEncodeSystemPointer(SE_DllUnloaded);
        g_pfnSE_InstallBeforeInit = RtlEncodeSystemPointer(SE_InstallBeforeInit);
        g_pfnSE_InstallAfterInit = RtlEncodeSystemPointer(SE_InstallAfterInit);
        g_pfnSE_ProcessDying = RtlEncodeSystemPointer(SE_ProcessDying);
        g_ShimsEnabled = 1u8;
    } else {
        LdrpUnloadShimEngine();
    }
}

#[no_mangle]
pub unsafe extern "system" fn LdrpRunShimEngineInitRoutine(Reason: ULONG) {
    let ListHead: PLIST_ENTRY = core::ptr::addr_of_mut!((*(*NtCurrentPeb()).Ldr).InLoadOrderModuleList);
    let mut Next: PLIST_ENTRY = (*ListHead).Flink;
    while Next != ListHead {
        let LdrEntry: PLDR_DATA_TABLE_ENTRY = (Next as *mut u8)
            .offset(-(core::mem::offset_of!(_LDR_DATA_TABLE_ENTRY, InLoadOrderLinks) as isize))
            as PLDR_DATA_TABLE_ENTRY;

        if g_pShimEngineModule == (*LdrEntry).DllBase {
            if !(*LdrEntry).EntryPoint.is_null() {
                LdrpCallInitRoutine(
                    core::mem::transmute((*LdrEntry).EntryPoint),
                    (*LdrEntry).DllBase,
                    Reason,
                    core::ptr::null_mut(),
                );
            }
            return;
        }

        Next = (*Next).Flink;
    }
}

#[no_mangle]
pub unsafe extern "system" fn LdrpLoadShimEngine(
    ImageName: PWSTR,
    ProcessImage: PUNICODE_STRING,
    pShimData: PVOID,
) {
    let mut ShimLibraryName: _UNICODE_STRING = core::mem::zeroed();
    let mut ShimLibrary: PVOID = core::ptr::null_mut();
    let Status: NTSTATUS;

    RtlInitUnicodeString(&mut ShimLibraryName, ImageName);

    Status = LdrpLoadDll(
        0u8,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut ShimLibraryName,
        &mut ShimLibrary,
        0u8,
    );

    if NT_SUCCESS(Status) {
        g_pShimEngineModule = ShimLibrary;
        LdrpRunShimEngineInitRoutine(DLL_PROCESS_ATTACH);
        LdrpGetShimEngineInterface();
        if g_ShimsEnabled != 0 {
            let SE_InstallBeforeInit: unsafe extern "system" fn(PUNICODE_STRING, PVOID) =
                core::mem::transmute(RtlDecodeSystemPointer(g_pfnSE_InstallBeforeInit));
            SE_InstallBeforeInit(ProcessImage, pShimData);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn LdrpUnloadShimEngine() {
    g_ShimsEnabled = 0u8;
    LdrpRunShimEngineInitRoutine(DLL_PROCESS_DETACH);
    LdrUnloadDll(g_pShimEngineModule);
    g_pShimEngineModule = core::ptr::null_mut();
}

// NOTE: hand-translated from ldrutils.c LdrpGetProcedureAddress (C lines
// 2237-2418). The rustconv scanner dropped this function entirely — its
// `_When_(...)` / `_In_opt_` SAL annotations on the parameter list confused
// tree-sitter so it never appeared in the topo list. DUMMY_PSEH: the two
// _SEH2_TRY blocks run unprotected (handlers are dead code) → plain blocks.
#[no_mangle]
pub unsafe extern "system" fn LdrpGetProcedureAddress(
    BaseAddress: PVOID,
    Name: PANSI_STRING,
    Ordinal: ULONG,
    ProcedureAddress: *mut PVOID,
    ExecuteInit: BOOLEAN,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut ImportBuffer: [UCHAR; 64] = [0; 64];
    let mut LdrEntry: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
    let mut Thunk: IMAGE_THUNK_DATA = core::mem::zeroed();
    let ImageBase: PVOID;
    let mut ImportName: PIMAGE_IMPORT_BY_NAME = core::ptr::null_mut();
    let ExportDir: PIMAGE_EXPORT_DIRECTORY;
    let mut ExportDirSize: ULONG = 0;
    let Length: ULONG;
    let Entry: PLIST_ENTRY;

    if ShowSnaps != 0 {
        DbgPrint(b"LDR: LdrGetProcedureAddress by \0".as_ptr() as PCHAR);
    }

    /* Check if we got a name */
    if !Name.is_null() {
        if ShowSnaps != 0 {
            DbgPrint(b"NAME - %s\n\0".as_ptr() as PCHAR, (*Name).Buffer);
        }

        /* Make sure it's not too long */
        Length = (*Name).Length as ULONG
            + core::mem::size_of::<CHAR>() as ULONG
            + core::mem::offset_of!(_IMAGE_IMPORT_BY_NAME, Name) as ULONG;
        if Length > UNICODE_STRING_MAX_BYTES as ULONG {
            return STATUS_NAME_TOO_LONG;
        }

        /* Check if our buffer is large enough */
        if Length as usize > core::mem::size_of_val(&ImportBuffer) {
            /* Allocate from heap, plus 2 bytes for the Hint */
            ImportName = RtlAllocateHeap(RtlGetProcessHeap(), 0, Length as SIZE_T)
                as PIMAGE_IMPORT_BY_NAME;
            if ImportName.is_null() {
                return STATUS_INVALID_PARAMETER;
            }
        } else {
            /* Use our internal buffer */
            ImportName = ImportBuffer.as_mut_ptr() as PIMAGE_IMPORT_BY_NAME;
        }

        /* Clear the hint */
        (*ImportName).Hint = 0;

        /* Copy the name and null-terminate it */
        core::ptr::copy_nonoverlapping(
            (*Name).Buffer as *const u8,
            (*ImportName).Name.as_mut_ptr() as *mut u8,
            (*Name).Length as usize,
        );
        *(*ImportName).Name.as_mut_ptr().add((*Name).Length as usize) = 0;

        /* Clear the high bit */
        ImageBase = ImportName as PVOID;
        Thunk.u1.AddressOfData = 0;
    } else {
        /* Do it by ordinal */
        ImageBase = core::ptr::null_mut();

        if ShowSnaps != 0 {
            DbgPrint(b"ORDINAL - %lx\n\0".as_ptr() as PCHAR, Ordinal);
        }

        /* Make sure an ordinal was given */
        if Ordinal == 0 {
            return STATUS_INVALID_PARAMETER;
        }

        /* Set the original flag in the thunk */
        // IMAGE_THUNK_DATA.u1.Ordinal is u64 on 64-bit, u32 on i386.
        #[cfg(target_pointer_width = "64")]
        {
            Thunk.u1.Ordinal = (Ordinal as u64) | (IMAGE_ORDINAL_FLAG as u64);
        }
        #[cfg(target_pointer_width = "32")]
        {
            Thunk.u1.Ordinal = Ordinal | (IMAGE_ORDINAL_FLAG as u32);
        }
    }

    /* Acquire lock unless we are initting */
    if LdrpInLdrInit == 0 {
        RtlEnterCriticalSection(core::ptr::addr_of_mut!(LdrpLoaderLock));
    }

    'quickie: {
        /* Try to find the loaded DLL */
        if LdrpCheckForLoadedDllHandle(BaseAddress, &mut LdrEntry) == 0 {
            Status = STATUS_DLL_NOT_FOUND;
            break 'quickie;
        }

        /* Get the pointer to the export directory */
        ExportDir = RtlImageDirectoryEntryToData(
            (*LdrEntry).DllBase,
            1u8,
            IMAGE_DIRECTORY_ENTRY_EXPORT as USHORT,
            &mut ExportDirSize,
        ) as PIMAGE_EXPORT_DIRECTORY;

        if ExportDir.is_null() {
            Status = STATUS_PROCEDURE_NOT_FOUND;
            break 'quickie;
        }

        /* Now get the thunk */
        Status = LdrpSnapThunk(
            (*LdrEntry).DllBase,
            ImageBase,
            &mut Thunk,
            &mut Thunk,
            ExportDir,
            ExportDirSize,
            0u8,
            core::ptr::null_mut(),
        );

        /* Finally, see if we're supposed to run the init routines */
        if NT_SUCCESS(Status) && ExecuteInit != 0 {
            /* It's possible a forwarded entry had us load the DLL. Use the
             * last loaded DLL for this. */
            Entry = (*(*NtCurrentPeb()).Ldr).InInitializationOrderModuleList.Blink;
            LdrEntry = (Entry as *mut u8)
                .offset(-(core::mem::offset_of!(_LDR_DATA_TABLE_ENTRY, InInitializationOrderLinks) as isize))
                as PLDR_DATA_TABLE_ENTRY;

            /* Make sure we didn't process it yet */
            if (*LdrEntry).Flags & LDRP_ENTRY_PROCESSED == 0 {
                /* Call the init routine */
                Status = LdrpRunInitializeRoutines(core::ptr::null_mut());
            }
        }

        /* Make sure we're OK till here */
        if NT_SUCCESS(Status) {
            /* Return the address */
            *ProcedureAddress = Thunk.u1.Function as PVOID;
        }
    }

    /* Cleanup: if we allocated from heap, free it */
    if !ImportName.is_null() && ImportName != ImportBuffer.as_mut_ptr() as PIMAGE_IMPORT_BY_NAME {
        RtlFreeHeap(RtlGetProcessHeap(), 0, ImportName as PVOID);
    }

    /* Release the CS if we entered it */
    if LdrpInLdrInit == 0 {
        RtlLeaveCriticalSection(core::ptr::addr_of_mut!(LdrpLoaderLock));
    }

    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS NT User-Mode Library
//  * FILE:            dll/ntdll/ldr/ldrutils.c
//  * PURPOSE:         Internal Loader Utility Functions
//  * PROGRAMMERS:     Alex Ionescu (alex@relsoft.net)
//  *                  Aleksey Bragin (aleksey@reactos.org)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntdll.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// PLDR_DATA_TABLE_ENTRY LdrpLoadedDllHandleCache, LdrpGetModuleHandleCache;
//
// BOOLEAN g_ShimsEnabled;
// PVOID g_pShimEngineModule;
// PVOID g_pfnSE_DllLoaded;
// PVOID g_pfnSE_DllUnloaded;
// PVOID g_pfnSE_InstallBeforeInit;
// PVOID g_pfnSE_InstallAfterInit;
// PVOID g_pfnSE_ProcessDying;
//
// /* FUNCTIONS *****************************************************************/
//
// NTSTATUS
// NTAPI
// LdrpAllocateUnicodeString(IN OUT PUNICODE_STRING StringOut,
//                           IN ULONG Length)
// {
//     /* Sanity checks */
//     ASSERT(StringOut);
//     ASSERT(Length <= UNICODE_STRING_MAX_BYTES);
//
//     /* Assume failure */
//     StringOut->Length = 0;
//
//     /* Make sure it's not mis-aligned */
//     if (Length & 1)
//     {
//         /* Fail */
//         StringOut->Buffer = NULL;
//         StringOut->MaximumLength = 0;
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Allocate the string*/
//     StringOut->Buffer = RtlAllocateHeap(LdrpHeap,
//                                         0,
//                                         Length + sizeof(WCHAR));
//     if (!StringOut->Buffer)
//     {
//         /* Fail */
//         StringOut->MaximumLength = 0;
//         return STATUS_NO_MEMORY;
//     }
//
//     /* Null-terminate it */
//     StringOut->Buffer[Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//     /* Check if this is a maximum-sized string */
//     if (Length != UNICODE_STRING_MAX_BYTES)
//     {
//         /* It's not, so set the maximum length to be one char more */
//         StringOut->MaximumLength = (USHORT)Length + sizeof(UNICODE_NULL);
//     }
//     else
//     {
//         /* The length is already the maximum possible */
//         StringOut->MaximumLength = UNICODE_STRING_MAX_BYTES;
//     }
//
//     /* Return success */
//     return STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// LdrpFreeUnicodeString(IN PUNICODE_STRING StringIn)
// {
//     ASSERT(StringIn != NULL);
//
//     /* If Buffer is not NULL - free it */
//     if (StringIn->Buffer)
//     {
//         RtlFreeHeap(LdrpHeap, 0, StringIn->Buffer);
//     }
//
//     /* Zero it out */
//     RtlInitEmptyUnicodeString(StringIn, NULL, 0);
// }
//
// BOOLEAN
// NTAPI
// LdrpCallInitRoutine(IN PDLL_INIT_ROUTINE EntryPoint,
//                     IN PVOID BaseAddress,
//                     IN ULONG Reason,
//                     IN PVOID Context)
// {
//     /* Call the entry */
//     return EntryPoint(BaseAddress, Reason, Context);
// }
//
// /* NOTE: This function is broken */
// VOID
// NTAPI
// LdrpUpdateLoadCount3(IN PLDR_DATA_TABLE_ENTRY LdrEntry,
//                      IN ULONG Flags,
//                      OUT PUNICODE_STRING UpdateString)
// {
//     PIMAGE_BOUND_FORWARDER_REF NewImportForwarder;
//     PIMAGE_BOUND_IMPORT_DESCRIPTOR FirstEntry;
//     PIMAGE_BOUND_IMPORT_DESCRIPTOR BoundEntry;
//     PIMAGE_IMPORT_DESCRIPTOR ImportEntry;
//     PIMAGE_THUNK_DATA FirstThunk;
//     PLDR_DATA_TABLE_ENTRY Entry;
//     PUNICODE_STRING ImportNameUnic, RedirectedImportName;
//     ANSI_STRING ImportNameAnsi;
//     LPSTR ImportName;
//     ULONG ImportSize;
//     NTSTATUS Status;
//     ULONG i;
//     BOOLEAN RedirectedDll;
//     RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED ActCtx;
//
//     /* Set up the Act Ctx */
//     ActCtx.Size = sizeof(ActCtx);
//     ActCtx.Format = RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_FORMAT_WHISTLER;
//     RtlZeroMemory(&ActCtx.Frame, sizeof(RTL_ACTIVATION_CONTEXT_STACK_FRAME));
//
//     /* Activate the ActCtx */
//     RtlActivateActivationContextUnsafeFast(&ActCtx,
//                                            LdrEntry->EntryPointActivationContext);
//
//     /* Check the action we need to perform */
//     if ((Flags == LDRP_UPDATE_REFCOUNT) || (Flags == LDRP_UPDATE_PIN))
//     {
//         /* Make sure entry is not being loaded already */
//         if (LdrEntry->Flags & LDRP_LOAD_IN_PROGRESS)
//             goto done;
//
//         LdrEntry->Flags |= LDRP_LOAD_IN_PROGRESS;
//     }
//     else if (Flags == LDRP_UPDATE_DEREFCOUNT)
//     {
//         /* Make sure the entry is not being unloaded already */
//         if (LdrEntry->Flags & LDRP_UNLOAD_IN_PROGRESS)
//             goto done;
//
//         LdrEntry->Flags |= LDRP_UNLOAD_IN_PROGRESS;
//     }
//
//     /* Go through all bound DLLs and dereference them */
//     ImportNameUnic = &NtCurrentTeb()->StaticUnicodeString;
//
//     /* Try to get the new import entry */
//     FirstEntry = RtlImageDirectoryEntryToData(LdrEntry->DllBase,
//                                               TRUE,
//                                               IMAGE_DIRECTORY_ENTRY_BOUND_IMPORT,
//                                               &ImportSize);
//
//     if (FirstEntry)
//     {
//         /* Set entry flags if refing/derefing */
//         if (Flags == LDRP_UPDATE_REFCOUNT)
//             LdrEntry->Flags |= LDRP_LOAD_IN_PROGRESS;
//         else if (Flags == LDRP_UPDATE_DEREFCOUNT)
//             LdrEntry->Flags |= LDRP_UNLOAD_IN_PROGRESS;
//
//         BoundEntry = FirstEntry;
//         while (BoundEntry->OffsetModuleName)
//         {
//             /* Get pointer to the current import name */
//             ImportName = (LPSTR)FirstEntry + BoundEntry->OffsetModuleName;
//
//             RtlInitAnsiString(&ImportNameAnsi, ImportName);
//             Status = RtlAnsiStringToUnicodeString(ImportNameUnic, &ImportNameAnsi, FALSE);
//
//             if (NT_SUCCESS(Status))
//             {
//                 RedirectedDll = FALSE;
//                 RedirectedImportName = ImportNameUnic;
//
//                 /* Check if the SxS Assemblies specify another file */
//                 Status = LdrpApplyFileNameRedirection(
//                     ImportNameUnic, &LdrApiDefaultExtension, UpdateString, NULL, &RedirectedImportName,
//                     &RedirectedDll);
//
//                 /* Check success */
//                 if (NT_SUCCESS(Status) && RedirectedDll)
//                 {
//                     if (ShowSnaps)
//                     {
//                         DPRINT1("LDR: %Z got redirected to %wZ\n", &ImportNameAnsi, RedirectedImportName);
//                     }
//                 }
//
//                 if (NT_SUCCESS(Status))
//                 {
//                     if (LdrpCheckForLoadedDll(NULL,
//                                               RedirectedImportName,
//                                               TRUE,
//                                               RedirectedDll,
//                                               &Entry))
//                     {
//                         if (Entry->LoadCount != 0xFFFF)
//                         {
//                             /* Perform the required action */
//                             switch (Flags)
//                             {
//                             case LDRP_UPDATE_REFCOUNT:
//                                 Entry->LoadCount++;
//                                 break;
//                             case LDRP_UPDATE_DEREFCOUNT:
//                                 Entry->LoadCount--;
//                                 break;
//                             case LDRP_UPDATE_PIN:
//                                 Entry->LoadCount = 0xFFFF;
//                                 break;
//                             }
//
//                             /* Show snaps */
//                             if (ShowSnaps)
//                             {
//                                 DPRINT1("LDR: Flags %lu  %wZ (%lx)\n", Flags, RedirectedImportName, Entry->LoadCount);
//                             }
//                         }
//
//                         /* Recurse into this entry */
//                         LdrpUpdateLoadCount3(Entry, Flags, UpdateString);
//                     }
//                     else if (RedirectedDll)
//                     {
//                         DPRINT1("LDR: LdrpCheckForLoadedDll failed for redirected dll %wZ\n", RedirectedImportName);
//                     }
//                 }
//                 else
//                 {
//                     /* Unrecoverable SxS failure */
//                     DPRINT1("LDR: LdrpApplyFileNameRedirection failed with status %x for dll %wZ\n", Status, ImportNameUnic);
//                 }
//
//             }
//
//             /* Go through forwarders */
//             NewImportForwarder = (PIMAGE_BOUND_FORWARDER_REF)(BoundEntry + 1);
//             for (i = 0; i < BoundEntry->NumberOfModuleForwarderRefs; i++)
//             {
//                 ImportName = (LPSTR)FirstEntry + NewImportForwarder->OffsetModuleName;
//
//                 RtlInitAnsiString(&ImportNameAnsi, ImportName);
//                 Status = RtlAnsiStringToUnicodeString(ImportNameUnic, &ImportNameAnsi, FALSE);
//                 if (NT_SUCCESS(Status))
//                 {
//                     RedirectedDll = FALSE;
//                     RedirectedImportName = ImportNameUnic;
//
//                     /* Check if the SxS Assemblies specify another file */
//                     Status = LdrpApplyFileNameRedirection(
//                         ImportNameUnic, &LdrApiDefaultExtension, UpdateString, NULL, &RedirectedImportName,
//                         &RedirectedDll);
//
//                     /* Check success */
//                     if (NT_SUCCESS(Status) && RedirectedDll)
//                     {
//                         if (ShowSnaps)
//                         {
//                             DPRINT1("LDR: %Z got redirected to %wZ\n", &ImportNameAnsi, RedirectedImportName);
//                         }
//                     }
//
//                     if (NT_SUCCESS(Status))
//                     {
//                         if (LdrpCheckForLoadedDll(NULL,
//                                                   RedirectedImportName,
//                                                   TRUE,
//                                                   RedirectedDll,
//                                                   &Entry))
//                         {
//                             if (Entry->LoadCount != 0xFFFF)
//                             {
//                                 /* Perform the required action */
//                                 switch (Flags)
//                                 {
//                                 case LDRP_UPDATE_REFCOUNT:
//                                     Entry->LoadCount++;
//                                     break;
//                                 case LDRP_UPDATE_DEREFCOUNT:
//                                     Entry->LoadCount--;
//                                     break;
//                                 case LDRP_UPDATE_PIN:
//                                     Entry->LoadCount = 0xFFFF;
//                                     break;
//                                 }
//
//                                 /* Show snaps */
//                                 if (ShowSnaps)
//                                 {
//                                     DPRINT1("LDR: Flags %lu  %wZ (%lx)\n", Flags, RedirectedImportName, Entry->LoadCount);
//                                 }
//                             }
//
//                             /* Recurse into this entry */
//                             LdrpUpdateLoadCount3(Entry, Flags, UpdateString);
//                         }
//                         else if (RedirectedDll)
//                         {
//                             DPRINT1("LDR: LdrpCheckForLoadedDll failed with status %x for redirected dll %wZ\n", Status, RedirectedImportName);
//                         }
//                     }
//                     else
//                     {
//                         /* Unrecoverable SxS failure */
//                         DPRINT1("LDR: LdrpApplyFileNameRedirection failed  with status %x for dll %wZ\n", Status, ImportNameUnic);
//                     }
//
//                 }
//
//                 NewImportForwarder++;
//             }
//
//             BoundEntry = (PIMAGE_BOUND_IMPORT_DESCRIPTOR)NewImportForwarder;
//         }
//
//         /* We're done */
//         goto done;
//     }
//
//     /* Check oldstyle import descriptor */
//     ImportEntry = (PIMAGE_IMPORT_DESCRIPTOR)RtlImageDirectoryEntryToData(LdrEntry->DllBase,
//                                                                          TRUE,
//                                                                          IMAGE_DIRECTORY_ENTRY_IMPORT,
//                                                                          &ImportSize);
//     if (ImportEntry)
//     {
//         /* There is old one, so go through its entries */
//         while (ImportEntry->Name && ImportEntry->FirstThunk)
//         {
//             FirstThunk = (PIMAGE_THUNK_DATA)((ULONG_PTR)LdrEntry->DllBase + ImportEntry->FirstThunk);
//
//             /* Skip this entry if needed */
//             if (!FirstThunk->u1.Function)
//             {
//                 ImportEntry++;
//                 continue;
//             }
//
//             ImportName = (PSZ)((ULONG_PTR)LdrEntry->DllBase + ImportEntry->Name);
//
//             RtlInitAnsiString(&ImportNameAnsi, ImportName);
//             Status = RtlAnsiStringToUnicodeString(ImportNameUnic, &ImportNameAnsi, FALSE);
//             if (NT_SUCCESS(Status))
//             {
//                 RedirectedDll = FALSE;
//                 RedirectedImportName = ImportNameUnic;
//
//                 /* Check if the SxS Assemblies specify another file */
//                 Status = LdrpApplyFileNameRedirection(
//                     ImportNameUnic, &LdrApiDefaultExtension, UpdateString, NULL, &RedirectedImportName, &RedirectedDll);
//
//                 /* Check success */
//                 if (NT_SUCCESS(Status) && RedirectedDll)
//                 {
//                     if (ShowSnaps)
//                     {
//                         DPRINT1("LDR: %Z got redirected to %wZ\n", &ImportNameAnsi, RedirectedImportName);
//                     }
//                 }
//
//                 if (NT_SUCCESS(Status))
//                 {
//                     if (LdrpCheckForLoadedDll(NULL,
//                                               RedirectedImportName,
//                                               TRUE,
//                                               RedirectedDll,
//                                               &Entry))
//                     {
//                         if (Entry->LoadCount != 0xFFFF)
//                         {
//                             /* Perform the required action */
//                             switch (Flags)
//                             {
//                             case LDRP_UPDATE_REFCOUNT:
//                                 Entry->LoadCount++;
//                                 break;
//                             case LDRP_UPDATE_DEREFCOUNT:
//                                 Entry->LoadCount--;
//                                 break;
//                             case LDRP_UPDATE_PIN:
//                                 Entry->LoadCount = 0xFFFF;
//                                 break;
//                             }
//
//                             /* Show snaps */
//                             if (ShowSnaps)
//                             {
//                                 DPRINT1("LDR: Flags %lu  %wZ (%lx)\n", Flags, RedirectedImportName, Entry->LoadCount);
//                             }
//                         }
//
//                         /* Recurse */
//                         LdrpUpdateLoadCount3(Entry, Flags, UpdateString);
//                     }
//                     else if (RedirectedDll)
//                     {
//                         DPRINT1("LDR: LdrpCheckForLoadedDll failed for redirected dll %wZ\n", RedirectedImportName);
//                     }
//
//                 }
//                 else
//                 {
//                     /* Unrecoverable SxS failure */
//                     DPRINT1("LDR: LdrpApplyFileNameRedirection failed for dll %wZ\n", ImportNameUnic);
//                 }
//             }
//
//             /* Go to the next entry */
//             ImportEntry++;
//         }
//     }
//
// done:
//     /* Release the context */
//     RtlDeactivateActivationContextUnsafeFast(&ActCtx);
// }
//
// VOID
// NTAPI
// LdrpUpdateLoadCount2(IN PLDR_DATA_TABLE_ENTRY LdrEntry,
//                      IN ULONG Flags)
// {
//     WCHAR Buffer[MAX_PATH];
//     UNICODE_STRING UpdateString;
//
//     /* Setup the string and call the extended API */
//     RtlInitEmptyUnicodeString(&UpdateString, Buffer, sizeof(Buffer));
//     LdrpUpdateLoadCount3(LdrEntry, Flags, &UpdateString);
// }
//
// VOID
// NTAPI
// LdrpCallTlsInitializers(IN PLDR_DATA_TABLE_ENTRY LdrEntry,
//                         IN ULONG Reason)
// {
//     PIMAGE_TLS_DIRECTORY TlsDirectory;
//     PIMAGE_TLS_CALLBACK *Array, Callback;
//     ULONG Size;
//
//     /* Get the TLS Directory */
//     TlsDirectory = RtlImageDirectoryEntryToData(LdrEntry->DllBase,
//                                                 TRUE,
//                                                 IMAGE_DIRECTORY_ENTRY_TLS,
//                                                 &Size);
//
//     /* Protect against invalid pointers */
//     _SEH2_TRY
//     {
//         /* Make sure it's valid */
//         if (TlsDirectory)
//         {
//             /* Get the array */
//             Array = (PIMAGE_TLS_CALLBACK *)TlsDirectory->AddressOfCallBacks;
//             if (Array)
//             {
//                 /* Display debug */
//                 if (ShowSnaps)
//                 {
//                     DPRINT1("LDR: Tls Callbacks Found. Imagebase %p Tls %p CallBacks %p\n",
//                             LdrEntry->DllBase, TlsDirectory, Array);
//                 }
//
//                 /* Loop the array */
//                 while (*Array)
//                 {
//                     /* Get the TLS Entrypoint */
//                     Callback = *Array++;
//
//                     /* Display debug */
//                     if (ShowSnaps)
//                     {
//                         DPRINT1("LDR: Calling Tls Callback Imagebase %p Function %p\n",
//                                 LdrEntry->DllBase, Callback);
//                     }
//
//                     /* Call it */
//                     LdrpCallInitRoutine((PDLL_INIT_ROUTINE)Callback,
//                                         LdrEntry->DllBase,
//                                         Reason,
//                                         NULL);
//                 }
//             }
//         }
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         DPRINT1("LDR: Exception 0x%x during Tls Callback(%u) for %wZ\n",
//                 _SEH2_GetExceptionCode(), Reason, &LdrEntry->BaseDllName);
//     }
//     _SEH2_END;
// }
//
// NTSTATUS
// NTAPI
// LdrpCodeAuthzCheckDllAllowed(IN PUNICODE_STRING FullName,
//                              IN HANDLE DllHandle)
// {
//     /* Not implemented */
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// LdrpCreateDllSection(IN PUNICODE_STRING FullName,
//                      IN HANDLE DllHandle,
//                      IN PULONG DllCharacteristics OPTIONAL,
//                      OUT PHANDLE SectionHandle)
// {
//     HANDLE FileHandle;
//     NTSTATUS Status;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     IO_STATUS_BLOCK IoStatusBlock;
//     ULONG_PTR HardErrorParameters[1];
//     ULONG Response;
//     SECTION_IMAGE_INFORMATION SectionImageInfo;
//
//     /* Check if we don't already have a handle */
//     if (!DllHandle)
//     {
//         /* Create the object attributes */
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    FullName,
//                                    OBJ_CASE_INSENSITIVE,
//                                    NULL,
//                                    NULL);
//
//         /* Open the DLL */
//         Status = NtOpenFile(&FileHandle,
//                             SYNCHRONIZE | FILE_EXECUTE | FILE_READ_DATA,
//                             &ObjectAttributes,
//                             &IoStatusBlock,
//                             FILE_SHARE_READ | FILE_SHARE_DELETE,
//                             FILE_NON_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT);
//
//         /* Check if we failed */
//         if (!NT_SUCCESS(Status))
//         {
//             /* Attempt to open for execute only */
//             Status = NtOpenFile(&FileHandle,
//                                 SYNCHRONIZE | FILE_EXECUTE,
//                                 &ObjectAttributes,
//                                 &IoStatusBlock,
//                                 FILE_SHARE_READ | FILE_SHARE_DELETE,
//                                 FILE_NON_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT);
//
//             /* Check if this failed too */
//             if (!NT_SUCCESS(Status))
//             {
//                 /* Show debug message */
//                 if (ShowSnaps)
//                 {
//                     DPRINT1("LDR: LdrpCreateDllSection - NtOpenFile failed; status = %x\n",
//                             Status);
//                 }
//
//                 /* Make sure to return an expected status code */
//                 if (Status == STATUS_OBJECT_NAME_NOT_FOUND)
//                 {
//                     /* Callers expect this instead */
//                     Status = STATUS_DLL_NOT_FOUND;
//                 }
//
//                 /* Return an empty section handle */
//                 *SectionHandle = NULL;
//                 return Status;
//             }
//         }
//     }
//     else
//     {
//         /* Use the handle we already have */
//         FileHandle = DllHandle;
//     }
//
//     /* Create a section for the DLL */
//     Status = NtCreateSection(SectionHandle,
//                              SECTION_MAP_READ | SECTION_MAP_EXECUTE |
//                              SECTION_MAP_WRITE | SECTION_QUERY,
//                              NULL,
//                              NULL,
//                              PAGE_EXECUTE,
//                              SEC_IMAGE,
//                              FileHandle);
//
//     /* If mapping failed, raise a hard error */
//     if (!NT_SUCCESS(Status))
//     {
//         /* Forget the handle */
//         *SectionHandle = NULL;
//
//         /* Give the DLL name */
//         HardErrorParameters[0] = (ULONG_PTR)FullName;
//
//         /* Raise the error */
//         ZwRaiseHardError(STATUS_INVALID_IMAGE_FORMAT,
//                          1,
//                          1,
//                          HardErrorParameters,
//                          OptionOk,
//                          &Response);
//
//         /* Increment the error count */
//         if (LdrpInLdrInit) LdrpFatalHardErrorCount++;
//
//         goto Exit;
//     }
//
//     /* Check for Safer restrictions */
//     if (!DllCharacteristics ||
//         !(*DllCharacteristics & IMAGE_FILE_SYSTEM))
//     {
//         /* Make sure it's executable */
//         Status = ZwQuerySection(*SectionHandle,
//                                 SectionImageInformation,
//                                 &SectionImageInfo,
//                                 sizeof(SECTION_IMAGE_INFORMATION),
//                                 NULL);
//         if (NT_SUCCESS(Status))
//         {
//             /* Bypass the check for .NET images */
//             if (!(SectionImageInfo.LoaderFlags & IMAGE_LOADER_FLAGS_COMPLUS))
//             {
//                 /* Check with Safer */
//                 Status = LdrpCodeAuthzCheckDllAllowed(FullName, DllHandle);
//                 if (!NT_SUCCESS(Status) && (Status != STATUS_NOT_FOUND))
//                 {
//                     /* Show debug message */
//                     if (ShowSnaps)
//                     {
//                         DPRINT1("LDR: Loading of (%wZ) blocked by Winsafer\n",
//                                 &FullName);
//                     }
//
//                     /* Failure case, close section handle */
//                     NtClose(*SectionHandle);
//                     *SectionHandle = NULL;
//                 }
//             }
//         }
//         else
//         {
//             /* Failure case, close section handle */
//             NtClose(*SectionHandle);
//             *SectionHandle = NULL;
//         }
//     }
//
// Exit:
//     /* Close the file handle, we don't need it */
//     NtClose(FileHandle);
//
//     /* Return status */
//     return Status;
// }
//
// /* NOTE: This function is totally b0rked and doesn't handle the parameters/functionality it should */
// BOOLEAN
// NTAPI
// LdrpResolveDllName(PWSTR DllPath,
//                    PWSTR DllName,
//                    PUNICODE_STRING FullDllName,
//                    PUNICODE_STRING BaseDllName)
// {
//     PWCHAR NameBuffer, p1, p2 = 0;
//     ULONG Length;
//     ULONG BufSize = 500;
//
//     /* Allocate space for full DLL name */
//     FullDllName->Buffer = RtlAllocateHeap(LdrpHeap, 0, BufSize + sizeof(UNICODE_NULL));
//     if (!FullDllName->Buffer) return FALSE;
//
//     Length = RtlDosSearchPath_U(DllPath ? DllPath : LdrpDefaultPath.Buffer,
//                                 DllName,
//                                 NULL,
//                                 BufSize,
//                                 FullDllName->Buffer,
//                                 &BaseDllName->Buffer);
//
//     if (!Length || Length > BufSize)
//     {
//         if (ShowSnaps)
//         {
//             DPRINT1("LDR: LdrResolveDllName - Unable to find ");
//             DPRINT1("%ws from %ws\n", DllName, DllPath ? DllPath : LdrpDefaultPath.Buffer);
//         }
//
//         LdrpFreeUnicodeString(FullDllName);
//         return FALSE;
//     }
//
//     /* Construct full DLL name */
//     FullDllName->Length = Length;
//     FullDllName->MaximumLength = FullDllName->Length + sizeof(UNICODE_NULL);
//
//     /* Allocate a new buffer */
//     NameBuffer = RtlAllocateHeap(LdrpHeap, 0, FullDllName->MaximumLength);
//     if (!NameBuffer)
//     {
//         RtlFreeHeap(LdrpHeap, 0, FullDllName->Buffer);
//         return FALSE;
//     }
//
//     /* Copy over the contents from the previous one and free it */
//     RtlCopyMemory(NameBuffer, FullDllName->Buffer, FullDllName->MaximumLength);
//     RtlFreeHeap(LdrpHeap, 0, FullDllName->Buffer);
//     FullDllName->Buffer = NameBuffer;
//
//     /* Find last backslash */
//     p1 = FullDllName->Buffer;
//     while (*p1)
//     {
//         if (*p1++ == L'\\')
//         {
//             p2 = p1;
//         }
//     }
//
//     /* If found, set p1 to it, otherwise p1 points to the beginning of DllName */
//     if (p2)
//         p1 = p2;
//     else
//         p1 = DllName;
//
//     p2 = p1;
//
//     /* Calculate remaining length */
//     while (*p1) ++p1;
//
//     /* Construct base DLL name */
//     BaseDllName->Length = (ULONG_PTR)p1 - (ULONG_PTR)p2;
//     BaseDllName->MaximumLength = BaseDllName->Length + sizeof(UNICODE_NULL);
//     BaseDllName->Buffer = RtlAllocateHeap(LdrpHeap, 0, BaseDllName->MaximumLength);
//
//     if (!BaseDllName->Buffer)
//     {
//         RtlFreeHeap(LdrpHeap, 0, NameBuffer);
//         return FALSE;
//     }
//
//     /* Copy base dll name to the new buffer */
//     RtlMoveMemory(BaseDllName->Buffer,
//                   p2,
//                   BaseDllName->Length);
//
//     /* Null-terminate the string */
//     BaseDllName->Buffer[BaseDllName->Length / sizeof(WCHAR)] = 0;
//
//     return TRUE;
// }
//
// PVOID
// NTAPI
// LdrpFetchAddressOfEntryPoint(IN PVOID ImageBase)
// {
//     PIMAGE_NT_HEADERS NtHeaders;
//     ULONG_PTR EntryPoint = 0;
//
//     /* Get entry point offset from NT headers */
//     NtHeaders = RtlImageNtHeader(ImageBase);
//     if (NtHeaders)
//     {
//         /* Add image base */
//         EntryPoint = NtHeaders->OptionalHeader.AddressOfEntryPoint;
//         if (EntryPoint) EntryPoint += (ULONG_PTR)ImageBase;
//     }
//
//     /* Return calculated pointer (or zero in case of failure) */
//     return (PVOID)EntryPoint;
// }
//
// /* NOTE: This function is partially missing SxS */
// NTSTATUS
// NTAPI
// LdrpCheckForKnownDll(PWSTR DllName,
//                      PUNICODE_STRING FullDllName,
//                      PUNICODE_STRING BaseDllName,
//                      HANDLE *SectionHandle)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE Section = NULL;
//     UNICODE_STRING DllNameUnic;
//     NTSTATUS Status;
//     PCHAR p1;
//     PWCHAR p2;
//
//     /* Zero initialize provided parameters */
//     if (SectionHandle) *SectionHandle = 0;
//
//     if (FullDllName)
//     {
//         FullDllName->Length = 0;
//         FullDllName->MaximumLength = 0;
//         FullDllName->Buffer = NULL;
//     }
//
//     if (BaseDllName)
//     {
//         BaseDllName->Length = 0;
//         BaseDllName->MaximumLength = 0;
//         BaseDllName->Buffer = NULL;
//     }
//
//     /* If any of these three params are missing then fail */
//     if (!SectionHandle || !FullDllName || !BaseDllName)
//         return STATUS_INVALID_PARAMETER;
//
//     /* Check the Loader Lock */
//     LdrpEnsureLoaderLockIsHeld();
//
//     /* Upgrade DllName to a unicode string */
//     RtlInitUnicodeString(&DllNameUnic, DllName);
//
//     /* FIXME: Missing RtlComputePrivatizedDllName_U related functionality */
//
//     /* Get the activation context */
//     Status = RtlFindActivationContextSectionString(0,
//                                                    NULL,
//                                                    ACTIVATION_CONTEXT_SECTION_DLL_REDIRECTION,
//                                                    &DllNameUnic,
//                                                    NULL);
//
//     /* Check if it's a SxS or not */
//     if (Status == STATUS_SXS_SECTION_NOT_FOUND ||
//         Status == STATUS_SXS_KEY_NOT_FOUND)
//     {
//         /* NOTE: Here it's beneficial to allocate one big unicode string
//                  using LdrpAllocateUnicodeString instead of fragmenting the heap
//                  with two allocations as it's done now. */
//
//         /* Set up BaseDllName */
//         BaseDllName->Length = DllNameUnic.Length;
//         BaseDllName->MaximumLength = DllNameUnic.MaximumLength;
//         BaseDllName->Buffer = RtlAllocateHeap(LdrpHeap,
//                                               0,
//                                               DllNameUnic.MaximumLength);
//         if (!BaseDllName->Buffer)
//         {
//             Status = STATUS_NO_MEMORY;
//             goto Failure;
//         }
//
//         /* Copy the contents there */
//         RtlMoveMemory(BaseDllName->Buffer, DllNameUnic.Buffer, DllNameUnic.MaximumLength);
//
//         /* Set up FullDllName */
//         FullDllName->Length = LdrpKnownDllPath.Length + BaseDllName->Length + sizeof(WCHAR);
//         FullDllName->MaximumLength = FullDllName->Length + sizeof(UNICODE_NULL);
//         FullDllName->Buffer = RtlAllocateHeap(LdrpHeap, 0, FullDllName->MaximumLength);
//         if (!FullDllName->Buffer)
//         {
//             Status = STATUS_NO_MEMORY;
//             goto Failure;
//         }
//
//         RtlMoveMemory(FullDllName->Buffer, LdrpKnownDllPath.Buffer, LdrpKnownDllPath.Length);
//
//         /* Put a slash there */
//         p1 = (PCHAR)FullDllName->Buffer + LdrpKnownDllPath.Length;
//         p2 = (PWCHAR)p1;
//         *p2++ = (WCHAR)'\\';
//         p1 = (PCHAR)p2;
//
//         /* Set up DllNameUnic for a relative path */
//         DllNameUnic.Buffer = (PWSTR)p1;
//         DllNameUnic.Length = BaseDllName->Length;
//         DllNameUnic.MaximumLength = DllNameUnic.Length + sizeof(UNICODE_NULL);
//
//         /* Copy the contents */
//         RtlMoveMemory(p1, BaseDllName->Buffer, BaseDllName->MaximumLength);
//
//         /* There are all names, init attributes and open the section */
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &DllNameUnic,
//                                    OBJ_CASE_INSENSITIVE,
//                                    LdrpKnownDllObjectDirectory,
//                                    NULL);
//
//         Status = NtOpenSection(&Section,
//                                SECTION_MAP_READ | SECTION_MAP_EXECUTE | SECTION_MAP_WRITE,
//                                &ObjectAttributes);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Clear status in case it was just not found */
//             if (Status == STATUS_OBJECT_NAME_NOT_FOUND) Status = STATUS_SUCCESS;
//             goto Failure;
//         }
//
//         /* Pass section handle to the caller and return success */
//         *SectionHandle = Section;
//         return STATUS_SUCCESS;
//     }
//
// Failure:
//     /* Close section object if it was opened */
//     if (Section) NtClose(Section);
//
//     /* Free string resources */
//     if (BaseDllName->Buffer) RtlFreeHeap(LdrpHeap, 0, BaseDllName->Buffer);
//     if (FullDllName->Buffer) RtlFreeHeap(LdrpHeap, 0, FullDllName->Buffer);
//
//     /* Return status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// LdrpSetProtection(PVOID ViewBase,
//                   BOOLEAN Restore)
// {
//     PIMAGE_NT_HEADERS NtHeaders;
//     PIMAGE_SECTION_HEADER Section;
//     NTSTATUS Status;
//     PVOID SectionBase;
//     SIZE_T SectionSize;
//     ULONG NewProtection, OldProtection, i;
//
//     /* Get the NT headers */
//     NtHeaders = RtlImageNtHeader(ViewBase);
//     if (!NtHeaders) return STATUS_INVALID_IMAGE_FORMAT;
//
//     /* Compute address of the first section header */
//     Section = IMAGE_FIRST_SECTION(NtHeaders);
//
//     /* Go through all sections */
//     for (i = 0; i < NtHeaders->FileHeader.NumberOfSections; i++)
//     {
//         /* Check for read-only non-zero section */
//         if ((Section->SizeOfRawData) &&
//             !(Section->Characteristics & IMAGE_SCN_MEM_WRITE))
//         {
//             /* Check if we are setting or restoring protection */
//             if (Restore)
//             {
//                 /* Set it to either EXECUTE or READONLY */
//                 if (Section->Characteristics & IMAGE_SCN_MEM_EXECUTE)
//                 {
//                     NewProtection = PAGE_EXECUTE;
//                 }
//                 else
//                 {
//                     NewProtection = PAGE_READONLY;
//                 }
//
//                 /* Add PAGE_NOCACHE if needed */
//                 if (Section->Characteristics & IMAGE_SCN_MEM_NOT_CACHED)
//                 {
//                     NewProtection |= PAGE_NOCACHE;
//                 }
//             }
//             else
//             {
//                 /* Enable write access */
//                 NewProtection = PAGE_READWRITE;
//             }
//
//             /* Get the section VA */
//             SectionBase = (PVOID)((ULONG_PTR)ViewBase + Section->VirtualAddress);
//             SectionSize = Section->SizeOfRawData;
//             if (SectionSize)
//             {
//                 /* Set protection */
//                 Status = ZwProtectVirtualMemory(NtCurrentProcess(),
//                                                 &SectionBase,
//                                                 &SectionSize,
//                                                 NewProtection,
//                                                 &OldProtection);
//                 if (!NT_SUCCESS(Status)) return Status;
//             }
//         }
//
//         /* Move to the next section */
//         Section++;
//     }
//
//     /* Flush instruction cache if necessary */
//     if (Restore) ZwFlushInstructionCache(NtCurrentProcess(), NULL, 0);
//     return STATUS_SUCCESS;
// }
//
// /* NOTE: Not yet reviewed */
// NTSTATUS
// NTAPI
// LdrpMapDll(IN PWSTR SearchPath OPTIONAL,
//            IN PWSTR DllPath2,
//            IN PWSTR DllName OPTIONAL,
//            IN PULONG DllCharacteristics,
//            IN BOOLEAN Static,
//            IN BOOLEAN Redirect,
//            OUT PLDR_DATA_TABLE_ENTRY *DataTableEntry)
// {
//     PTEB Teb = NtCurrentTeb();
//     PPEB Peb = NtCurrentPeb();
//     PWCHAR p1 = DllName;
//     WCHAR TempChar;
//     BOOLEAN KnownDll = FALSE;
//     UNICODE_STRING FullDllName, BaseDllName;
//     HANDLE SectionHandle = NULL, DllHandle = 0;
//     UNICODE_STRING NtPathDllName;
//     ULONG_PTR HardErrorParameters[2];
//     UNICODE_STRING HardErrorDllName, HardErrorDllPath;
//     ULONG Response;
//     SIZE_T ViewSize = 0;
//     PVOID ViewBase = NULL;
//     PVOID ArbitraryUserPointer;
//     PIMAGE_NT_HEADERS NtHeaders;
//     NTSTATUS HardErrorStatus, Status;
//     BOOLEAN OverlapDllFound = FALSE;
//     ULONG_PTR ImageBase, ImageEnd;
//     PLIST_ENTRY ListHead, NextEntry;
//     PLDR_DATA_TABLE_ENTRY CandidateEntry, LdrEntry;
//     ULONG_PTR CandidateBase, CandidateEnd;
//     UNICODE_STRING OverlapDll;
//     BOOLEAN RelocatableDll = TRUE;
//     UNICODE_STRING IllegalDll;
//     PVOID RelocData;
//     ULONG RelocDataSize = 0;
//
//     // FIXME: AppCompat stuff is missing
//
//     if (ShowSnaps)
//     {
//         DPRINT1("LDR: LdrpMapDll: Image Name %ws, Search Path %ws\n",
//                 DllName,
//                 SearchPath ? SearchPath : L"");
//     }
//
//     /* Check if we have a known dll directory */
//     if (LdrpKnownDllObjectDirectory && Redirect == FALSE)
//     {
//         /* Check if the path is full */
//         while (*p1)
//         {
//             TempChar = *p1++;
//             if (TempChar == '\\' || TempChar == '/' )
//             {
//                 /* Complete path, don't do Known Dll lookup */
//                 goto SkipCheck;
//             }
//         }
//
//         /* Try to find a Known DLL */
//         Status = LdrpCheckForKnownDll(DllName,
//                                       &FullDllName,
//                                       &BaseDllName,
//                                       &SectionHandle);
//
//         if (!NT_SUCCESS(Status) && (Status != STATUS_DLL_NOT_FOUND))
//         {
//             /* Failure */
//             DbgPrintEx(DPFLTR_LDR_ID,
//                        DPFLTR_ERROR_LEVEL,
//                        "LDR: %s - call to LdrpCheckForKnownDll(\"%ws\", ...) failed with status %x\n",
//                         __FUNCTION__,
//                         DllName,
//                         Status);
//
//             return Status;
//         }
//     }
//
// SkipCheck:
//
//     /* Check if the Known DLL Check returned something */
//     if (!SectionHandle)
//     {
//         /* It didn't, so try to resolve the name now */
//         if (LdrpResolveDllName(SearchPath,
//                                DllName,
//                                &FullDllName,
//                                &BaseDllName))
//         {
//             /* Got a name, display a message */
//             if (ShowSnaps)
//             {
//                 DPRINT1("LDR: Loading (%s) %wZ\n",
//                         Static ? "STATIC" : "DYNAMIC",
//                         &FullDllName);
//             }
//
//             /* Convert to NT Name */
//             if (!RtlDosPathNameToNtPathName_U(FullDllName.Buffer,
//                                               &NtPathDllName,
//                                               NULL,
//                                               NULL))
//             {
//                 /* Path was invalid */
//                 return STATUS_OBJECT_PATH_SYNTAX_BAD;
//             }
//
//             /* Create a section for this dLL */
//             Status = LdrpCreateDllSection(&NtPathDllName,
//                                           DllHandle,
//                                           DllCharacteristics,
//                                           &SectionHandle);
//
//             /* Free the NT Name */
//             RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathDllName.Buffer);
//
//             /* If we failed */
//             if (!NT_SUCCESS(Status))
//             {
//                 /* Free the name strings and return */
//                 LdrpFreeUnicodeString(&FullDllName);
//                 LdrpFreeUnicodeString(&BaseDllName);
//                 return Status;
//             }
//         }
//         else
//         {
//             /* We couldn't resolve the name, is this a static load? */
//             if (Static)
//             {
// #ifdef _M_ARM64
//                 /* ARM64: Don't raise hard error for missing DLLs.
//                  * Delay imports become regular imports on ARM64 (llvm-dlltool
//                  * limitation), so many DLLs appear as hard imports but are
//                  * never actually called. Let the caller skip them. */
//                 DPRINT1("LDR: ARM64 DLL not found: %S (static load, skipping hard error)\n", DllName);
// #else
//                 /*
//                  * This is BAD! Static loads are CRITICAL. Bugcheck!
//                  * Initialize the strings for the error
//                  */
//                 RtlInitUnicodeString(&HardErrorDllName, DllName);
//                 RtlInitUnicodeString(&HardErrorDllPath,
//                                      DllPath2 ? DllPath2 : LdrpDefaultPath.Buffer);
//
//                 /* Set them as error parameters */
//                 HardErrorParameters[0] = (ULONG_PTR)&HardErrorDllName;
//                 HardErrorParameters[1] = (ULONG_PTR)&HardErrorDllPath;
//
//                 /* Raise the hard error */
//                 NtRaiseHardError(STATUS_DLL_NOT_FOUND,
//                                  2,
//                                  0x00000003,
//                                  HardErrorParameters,
//                                  OptionOk,
//                                  &Response);
//
//                 /* We're back, where we initializing? */
//                 if (LdrpInLdrInit) LdrpFatalHardErrorCount++;
// #endif
//             }
//
//             /* Return failure */
//             return STATUS_DLL_NOT_FOUND;
//         }
//     }
//     else
//     {
//         /* We have a section handle, so this is a known dll */
//         KnownDll = TRUE;
//     }
//
//     /* Stuff the image name in the TIB, for the debugger */
//     ArbitraryUserPointer = Teb->NtTib.ArbitraryUserPointer;
//     Teb->NtTib.ArbitraryUserPointer = FullDllName.Buffer;
//
//     /* Map the DLL */
//     ViewBase = NULL;
//     ViewSize = 0;
//     Status = NtMapViewOfSection(SectionHandle,
//                                 NtCurrentProcess(),
//                                 &ViewBase,
//                                 0,
//                                 0,
//                                 NULL,
//                                 &ViewSize,
//                                 ViewShare,
//                                 0,
//                                 PAGE_READWRITE);
//
//     /* Restore */
//     Teb->NtTib.ArbitraryUserPointer = ArbitraryUserPointer;
//
//     /* Fail if we couldn't map it */
//     if (!NT_SUCCESS(Status))
//     {
//         /* Close and return */
//         NtClose(SectionHandle);
//         return Status;
//     }
//
//     /* Get the NT Header */
//     if (!(NtHeaders = RtlImageNtHeader(ViewBase)))
//     {
//         /* Invalid image, unmap, close handle and fail */
//         NtUnmapViewOfSection(NtCurrentProcess(), ViewBase);
//         NtClose(SectionHandle);
//         return STATUS_INVALID_IMAGE_FORMAT;
//     }
//
//     // FIXME: .NET support is missing
//
//     /* Allocate an entry */
//     if (!(LdrEntry = LdrpAllocateDataTableEntry(ViewBase)))
//     {
//         /* Invalid image, unmap, close handle and fail */
//         NtUnmapViewOfSection(NtCurrentProcess(), ViewBase);
//         NtClose(SectionHandle);
//         return STATUS_NO_MEMORY;
//     }
//
//     /* Setup the entry */
//     LdrEntry->Flags = Static ? LDRP_STATIC_LINK : 0;
//     if (Redirect) LdrEntry->Flags |= LDRP_REDIRECTED;
//     LdrEntry->LoadCount = 0;
//     LdrEntry->FullDllName = FullDllName;
//     LdrEntry->BaseDllName = BaseDllName;
//     LdrEntry->EntryPoint = LdrpFetchAddressOfEntryPoint(LdrEntry->DllBase);
//
//     /* Show debug message */
//     if (ShowSnaps)
//     {
//         DPRINT1("LDR: LdrpMapDll: Full Name %wZ, Base Name %wZ\n",
//                 &FullDllName,
//                 &BaseDllName);
//     }
//
//     /* Insert this entry */
//     LdrpInsertMemoryTableEntry(LdrEntry);
//
// #if (_WIN32_WINNT >= _WIN32_WINNT_VISTA) || (DLL_EXPORT_VERSION >= _WIN32_WINNT_VISTA)
//     LdrpSendDllNotifications(LdrEntry, LDR_DLL_NOTIFICATION_REASON_LOADED);
// #if (_WIN32_WINNT >= _WIN32_WINNT_WIN8)
//     LdrEntry->Flags |= LDRP_LOAD_NOTIFICATIONS_SENT; /* LdrEntry->LoadNotificationsSent = TRUE; */
// #endif
// #endif
//
//     /* Check for invalid CPU Image */
//     if (Status == STATUS_IMAGE_MACHINE_TYPE_MISMATCH)
//     {
//         /* Load our header */
//         PIMAGE_NT_HEADERS ImageNtHeader = RtlImageNtHeader(Peb->ImageBaseAddress);
//
//         /* Assume defaults if we don't have to run the Hard Error path */
//         HardErrorStatus = STATUS_SUCCESS;
//         Response = ResponseCancel;
//
//         /* Are we an NT 3.0 image? [Do these still exist? LOL -- IAI] */
//         if (ImageNtHeader->OptionalHeader.MajorSubsystemVersion <= 3)
//         {
//             /* Reset the entrypoint, save our Dll Name */
//             LdrEntry->EntryPoint = 0;
//             HardErrorParameters[0] = (ULONG_PTR)&FullDllName;
//
//             /* Raise the error */
//             HardErrorStatus = ZwRaiseHardError(STATUS_IMAGE_MACHINE_TYPE_MISMATCH,
//                                                1,
//                                                1,
//                                                HardErrorParameters,
//                                                OptionOkCancel,
//                                                &Response);
//         }
//
//         /* Check if the user pressed cancel */
//         if (NT_SUCCESS(HardErrorStatus) && Response == ResponseCancel)
//         {
//             /* Remove the DLL from the lists */
//             RemoveEntryList(&LdrEntry->InLoadOrderLinks);
//             RemoveEntryList(&LdrEntry->InMemoryOrderLinks);
//             RemoveEntryList(&LdrEntry->HashLinks);
//
//             /* Remove the LDR Entry */
//             RtlFreeHeap(LdrpHeap, 0, LdrEntry );
//
//             /* Unmap and close section */
//             NtUnmapViewOfSection(NtCurrentProcess(), ViewBase);
//             NtClose(SectionHandle);
//
//             /* Did we do a hard error? */
//             if (ImageNtHeader->OptionalHeader.MajorSubsystemVersion <= 3)
//             {
//                 /* Yup, so increase fatal error count if we are initializing */
//                 if (LdrpInLdrInit) LdrpFatalHardErrorCount++;
//             }
//
//             /* Return failure */
//             return STATUS_INVALID_IMAGE_FORMAT;
//         }
//     }
//     else
//     {
//         /* The image was valid. Is it a DLL? */
//         if (NtHeaders->FileHeader.Characteristics & IMAGE_FILE_DLL)
//         {
//             /* Set the DLL Flag */
//             LdrEntry->Flags |= LDRP_IMAGE_DLL;
//         }
//
//         /* If we're not a DLL, clear the entrypoint */
//         if (!(LdrEntry->Flags & LDRP_IMAGE_DLL))
//         {
//             LdrEntry->EntryPoint = 0;
//         }
//     }
//
//     /* Return it for the caller */
//     *DataTableEntry = LdrEntry;
//
//     /* Check if we loaded somewhere else */
//     if (Status == STATUS_IMAGE_NOT_AT_BASE)
//     {
//         /* Write the flag */
//         LdrEntry->Flags |= LDRP_IMAGE_NOT_AT_BASE;
//
//         /* Find our region */
//         ImageBase = (ULONG_PTR)NtHeaders->OptionalHeader.ImageBase;
//         ImageEnd = ImageBase + ViewSize;
//
//         DPRINT("LDR: LdrpMapDll Relocating Image Name %ws (%p-%p -> %p)\n", DllName, (PVOID)ImageBase, (PVOID)ImageEnd, ViewBase);
//
//         /* Scan all the modules */
//         ListHead = &Peb->Ldr->InLoadOrderModuleList;
//         NextEntry = ListHead->Flink;
//         while (NextEntry != ListHead)
//         {
//             /* Get the entry */
//             CandidateEntry = CONTAINING_RECORD(NextEntry,
//                                                LDR_DATA_TABLE_ENTRY,
//                                                InLoadOrderLinks);
//             NextEntry = NextEntry->Flink;
//
//             /* Get the entry's bounds */
//             CandidateBase = (ULONG_PTR)CandidateEntry->DllBase;
//             CandidateEnd = CandidateBase + CandidateEntry->SizeOfImage;
//
//             /* Make sure this entry isn't unloading */
//             if (!CandidateEntry->InMemoryOrderLinks.Flink) continue;
//
//             /* Check if our regions are colliding */
//             if ((ImageBase >= CandidateBase && ImageBase <= CandidateEnd) ||
//                 (ImageEnd >= CandidateBase && ImageEnd <= CandidateEnd) ||
//                 (CandidateBase >= ImageBase && CandidateBase <= ImageEnd))
//             {
//                 /* Found who is overlapping */
//                 OverlapDllFound = TRUE;
//                 OverlapDll = CandidateEntry->FullDllName;
//                 break;
//             }
//         }
//
//         /* Check if we found the DLL overlapping with us */
//         if (!OverlapDllFound)
//         {
//             /* It's not another DLL, it's memory already here */
//             RtlInitUnicodeString(&OverlapDll, L"Dynamically Allocated Memory");
//         }
//
//         DPRINT("Overlapping DLL: %wZ\n", &OverlapDll);
//
//         /* Are we dealing with a DLL? */
//         if (LdrEntry->Flags & LDRP_IMAGE_DLL)
//         {
//             /* Check if relocs were stripped */
//             if (!(NtHeaders->FileHeader.Characteristics & IMAGE_FILE_RELOCS_STRIPPED))
//             {
//                 /* Get the relocation data */
//                 RelocData = RtlImageDirectoryEntryToData(ViewBase,
//                                                          TRUE,
//                                                          IMAGE_DIRECTORY_ENTRY_BASERELOC,
//                                                          &RelocDataSize);
//
//                 /* Does the DLL not have any? */
//                 if (!RelocData && !RelocDataSize)
//                 {
//                     /* We'll allow this and simply continue */
//                     goto NoRelocNeeded;
//                 }
//             }
//
//             /* See if this is an Illegal DLL - IE: user32 and kernel32 */
//             RtlInitUnicodeString(&IllegalDll,L"user32.dll");
//             if (RtlEqualUnicodeString(&BaseDllName, &IllegalDll, TRUE))
//             {
//                 /* Can't relocate user32 */
//                 RelocatableDll = FALSE;
//             }
//             else
//             {
//                 RtlInitUnicodeString(&IllegalDll, L"kernel32.dll");
//                 if (RtlEqualUnicodeString(&BaseDllName, &IllegalDll, TRUE))
//                 {
//                     /* Can't relocate kernel32 */
//                     RelocatableDll = FALSE;
//                 }
//             }
//
//             /* Known DLLs are not allowed to be relocated */
//             if (KnownDll && !RelocatableDll)
//             {
//                 /* Setup for hard error */
//                 HardErrorParameters[0] = (ULONG_PTR)&IllegalDll;
//                 HardErrorParameters[1] = (ULONG_PTR)&OverlapDll;
//
//                 DPRINT1("Illegal DLL relocation! %wZ overlaps %wZ\n", &OverlapDll, &IllegalDll);
//
//                 /* Raise the error */
//                 ZwRaiseHardError(STATUS_ILLEGAL_DLL_RELOCATION,
//                                  2,
//                                  3,
//                                  HardErrorParameters,
//                                  OptionOk,
//                                  &Response);
//
//                 /* If initializing, increase the error count */
//                 if (LdrpInLdrInit) LdrpFatalHardErrorCount++;
//
//                 /* Don't do relocation */
//                 Status = STATUS_CONFLICTING_ADDRESSES;
//                 goto FailRelocate;
//             }
//
//             /* Change the protection to prepare for relocation */
//             Status = LdrpSetProtection(ViewBase, FALSE);
//
//             /* Make sure we changed the protection */
//             if (NT_SUCCESS(Status))
//             {
//                 /* Do the relocation */
//                 Status = LdrRelocateImageWithBias(ViewBase, 0LL, NULL, STATUS_SUCCESS,
//                     STATUS_CONFLICTING_ADDRESSES, STATUS_INVALID_IMAGE_FORMAT);
//
//                 if (NT_SUCCESS(Status))
//                 {
//                     /* Stuff the image name in the TIB, for the debugger */
//                     ArbitraryUserPointer = Teb->NtTib.ArbitraryUserPointer;
//                     Teb->NtTib.ArbitraryUserPointer = FullDllName.Buffer;
// #if 0
//                     /* Map the DLL */
//                     Status = NtMapViewOfSection(SectionHandle,
//                                                 NtCurrentProcess(),
//                                                 &ViewBase,
//                                                 0,
//                                                 0,
//                                                 NULL,
//                                                 &ViewSize,
//                                                 ViewShare,
//                                                 0,
//                                                 PAGE_READWRITE);
// #endif
//                     /* Restore */
//                     Teb->NtTib.ArbitraryUserPointer = ArbitraryUserPointer;
//
//                     /* Return the protection */
//                     Status = LdrpSetProtection(ViewBase, TRUE);
//                 }
//             }
// FailRelocate:
//             /* Handle any kind of failure */
//             if (!NT_SUCCESS(Status))
//             {
//                 /* Remove it from the lists */
//                 RemoveEntryList(&LdrEntry->InLoadOrderLinks);
//                 RemoveEntryList(&LdrEntry->InMemoryOrderLinks);
//                 RemoveEntryList(&LdrEntry->HashLinks);
//
//                 /* Unmap it, clear the entry */
//                 NtUnmapViewOfSection(NtCurrentProcess(), ViewBase);
//                 LdrEntry = NULL;
//             }
//
//             /* Show debug message */
//             if (ShowSnaps)
//             {
//                 DPRINT1("LDR: Fixups %successfully re-applied @ %p\n",
//                         NT_SUCCESS(Status) ? "s" : "uns", ViewBase);
//             }
//         }
//         else
//         {
// NoRelocNeeded:
//             /* Not a DLL, or no relocation needed */
//             Status = STATUS_SUCCESS;
//
//             /* Stuff the image name in the TIB, for the debugger */
//             ArbitraryUserPointer = Teb->NtTib.ArbitraryUserPointer;
//             Teb->NtTib.ArbitraryUserPointer = FullDllName.Buffer;
// #if 0
//             /* Map the DLL */
//             Status = NtMapViewOfSection(SectionHandle,
//                                         NtCurrentProcess(),
//                                         &ViewBase,
//                                         0,
//                                         0,
//                                         NULL,
//                                         &ViewSize,
//                                         ViewShare,
//                                         0,
//                                         PAGE_READWRITE);
// #endif
//             /* Restore */
//             Teb->NtTib.ArbitraryUserPointer = ArbitraryUserPointer;
//
//             /* Show debug message */
//             if (ShowSnaps)
//             {
//                 DPRINT1("LDR: Fixups won't be re-applied to non-Dll @ %p\n", ViewBase);
//             }
//         }
//     }
//
//     // FIXME: LdrpCheckCorImage() is missing
//
//     /* Check if this is an SMP Machine and a DLL */
//     if ((LdrpNumberOfProcessors > 1) &&
//         (LdrEntry && (LdrEntry->Flags & LDRP_IMAGE_DLL)))
//     {
//         /* Validate the image for MP */
//         LdrpValidateImageForMp(LdrEntry);
//     }
//
//     // FIXME: LdrpCorUnloadImage() is missing
//
//     /* Close section and return status */
//     NtClose(SectionHandle);
//     return Status;
// }
//
// PLDR_DATA_TABLE_ENTRY
// NTAPI
// LdrpAllocateDataTableEntry(IN PVOID BaseAddress)
// {
//     PLDR_DATA_TABLE_ENTRY LdrEntry = NULL;
//     PIMAGE_NT_HEADERS NtHeader;
//
//     /* Make sure the header is valid */
//     NtHeader = RtlImageNtHeader(BaseAddress);
//     DPRINT("LdrpAllocateDataTableEntry(%p), NtHeader %p\n", BaseAddress, NtHeader);
//
//     if (NtHeader)
//     {
//         /* Allocate an entry */
//         LdrEntry = RtlAllocateHeap(LdrpHeap,
//                                    HEAP_ZERO_MEMORY,
//                                    sizeof(LDR_DATA_TABLE_ENTRY));
//
//         /* Make sure we got one */
//         if (LdrEntry)
//         {
//             /* Set it up */
//             LdrEntry->DllBase = BaseAddress;
//             LdrEntry->SizeOfImage = NtHeader->OptionalHeader.SizeOfImage;
//             LdrEntry->TimeDateStamp = NtHeader->FileHeader.TimeDateStamp;
//             LdrEntry->PatchInformation = NULL;
//         }
//     }
//
//     /* Return the entry */
//     return LdrEntry;
// }
//
// VOID
// NTAPI
// LdrpInsertMemoryTableEntry(IN PLDR_DATA_TABLE_ENTRY LdrEntry)
// {
//     PPEB_LDR_DATA PebData = NtCurrentPeb()->Ldr;
//     ULONG i;
//
//     /* Insert into hash table */
//     i = LDR_GET_HASH_ENTRY(LdrEntry->BaseDllName.Buffer[0]);
//     InsertTailList(&LdrpHashTable[i], &LdrEntry->HashLinks);
//
//     /* Insert into other lists */
//     InsertTailList(&PebData->InLoadOrderModuleList, &LdrEntry->InLoadOrderLinks);
//     InsertTailList(&PebData->InMemoryOrderModuleList, &LdrEntry->InMemoryOrderLinks);
// }
//
// VOID
// NTAPI
// LdrpFinalizeAndDeallocateDataTableEntry(IN PLDR_DATA_TABLE_ENTRY Entry)
// {
//     /* Sanity check */
//     ASSERT(Entry != NULL);
//
//     /* Release the activation context if it exists and wasn't already released */
//     if ((Entry->EntryPointActivationContext) &&
//         (Entry->EntryPointActivationContext != INVALID_HANDLE_VALUE))
//     {
//         /* Mark it as invalid */
//         RtlReleaseActivationContext(Entry->EntryPointActivationContext);
//         Entry->EntryPointActivationContext = INVALID_HANDLE_VALUE;
//     }
//
//     /* Release the full dll name string */
//     if (Entry->FullDllName.Buffer) LdrpFreeUnicodeString(&Entry->FullDllName);
//
//     /* Finally free the entry's memory */
//     RtlFreeHeap(LdrpHeap, 0, Entry);
// }
//
// BOOLEAN
// NTAPI
// LdrpCheckForLoadedDllHandle(IN PVOID Base,
//                             OUT PLDR_DATA_TABLE_ENTRY *LdrEntry)
// {
//     PLDR_DATA_TABLE_ENTRY Current;
//     PLIST_ENTRY ListHead, Next;
//
//     /* Check the cache first */
//     if ((LdrpLoadedDllHandleCache) &&
//         (LdrpLoadedDllHandleCache->DllBase == Base))
//     {
//         /* We got lucky, return the cached entry */
//         *LdrEntry = LdrpLoadedDllHandleCache;
//         return TRUE;
//     }
//
//     /* Time for a lookup */
//     ListHead = &NtCurrentPeb()->Ldr->InLoadOrderModuleList;
//     Next = ListHead->Flink;
//     while (Next != ListHead)
//     {
//         /* Get the current entry */
//         Current = CONTAINING_RECORD(Next,
//                                     LDR_DATA_TABLE_ENTRY,
//                                     InLoadOrderLinks);
//
//         /* Make sure it's not unloading and check for a match */
//         if ((Current->InMemoryOrderLinks.Flink) && (Base == Current->DllBase))
//         {
//             /* Save in cache */
//             LdrpLoadedDllHandleCache = Current;
//
//             /* Return it */
//             *LdrEntry = Current;
//             return TRUE;
//         }
//
//         /* Move to the next one */
//         Next = Next->Flink;
//     }
//
//     /* Nothing found */
//     return FALSE;
// }
//
// NTSTATUS
// NTAPI
// LdrpResolveFullName(IN PUNICODE_STRING OriginalName,
//                     IN PUNICODE_STRING PathName,
//                     IN PUNICODE_STRING FullPathName,
//                     IN PUNICODE_STRING *ExpandedName)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
// //    RTL_PATH_TYPE PathType;
// //    BOOLEAN InvalidName;
//     ULONG Length;
//
//     /* Display debug output if snaps are on */
//     if (ShowSnaps)
//     {
//         DbgPrintEx(DPFLTR_LDR_ID,
//                    DPFLTR_ERROR_LEVEL,
//                    "LDR: %s - Expanding full name of %wZ\n",
//                    __FUNCTION__,
//                    OriginalName);
//     }
//
//     /* FIXME: Lock the PEB */
//     //RtlEnterCriticalSection(&FastPebLock);
// #if 0
//     /* Get the path name */
//     Length = RtlGetFullPathName_Ustr(OriginalName,
//                                      PathName->Length,
//                                      PathName->Buffer,
//                                      NULL,
//                                      &InvalidName,
//                                      &PathType);
// #else
//     Length = 0;
// #endif
//     if (!(Length) || (Length > UNICODE_STRING_MAX_BYTES))
//     {
//         /* Fail */
//         Status = STATUS_NAME_TOO_LONG;
//         goto Quickie;
//     }
//
//     /* Check if the length hasn't changed */
//     if (Length <= PathName->Length)
//     {
//         /* Return the same thing */
//         *ExpandedName = PathName;
//         PathName->Length = (USHORT)Length;
//         goto Quickie;
//     }
//
//     /* Sanity check */
//     ASSERT(Length >= sizeof(WCHAR));
//
//     /* Allocate a string */
//     Status = LdrpAllocateUnicodeString(FullPathName, Length - sizeof(WCHAR));
//     if (!NT_SUCCESS(Status)) goto Quickie;
//
//     /* Now get the full path again */
// #if 0
//     Length = RtlGetFullPathName_Ustr(OriginalName,
//                                      FullPathName->Length,
//                                      FullPathName->Buffer,
//                                      NULL,
//                                      &InvalidName,
//                                      &PathType);
// #else
//     Length = 0;
// #endif
//     if (!(Length) || (Length > FullPathName->Length))
//     {
//         /* Fail */
//         LdrpFreeUnicodeString(FullPathName);
//         Status = STATUS_NAME_TOO_LONG;
//     }
//     else
//     {
//         /* Return the expanded name */
//         *ExpandedName = FullPathName;
//         FullPathName->Length = (USHORT)Length;
//     }
//
// Quickie:
//     /* FIXME: Unlock the PEB */
//     //RtlLeaveCriticalSection(&FastPebLock);
//
//     /* Display debug output if snaps are on */
//     if (ShowSnaps)
//     {
//         /* Check which output to use -- failure or success */
//         if (NT_SUCCESS(Status))
//         {
//             DbgPrintEx(DPFLTR_LDR_ID,
//                        DPFLTR_ERROR_LEVEL,
//                        "LDR: %s - Expanded to %wZ\n",
//                        __FUNCTION__,
//                        *ExpandedName);
//         }
//         else
//         {
//             DbgPrintEx(DPFLTR_LDR_ID,
//                        DPFLTR_ERROR_LEVEL,
//                        "LDR: %s - Failed to expand %wZ; 0x%08x\n",
//                        __FUNCTION__,
//                        OriginalName,
//                        Status);
//         }
//     }
//
//     /* If we failed, return NULL */
//     if (!NT_SUCCESS(Status)) *ExpandedName = NULL;
//
//     /* Return status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// LdrpSearchPath(IN PWCHAR *SearchPath,
//                IN PWCHAR DllName,
//                IN PUNICODE_STRING PathName,
//                IN PUNICODE_STRING FullPathName,
//                IN PUNICODE_STRING *ExpandedName)
// {
//     BOOLEAN TryAgain = FALSE;
//     PWCHAR ActualSearchPath = *SearchPath;
//     UNICODE_STRING TestName;
//     NTSTATUS Status;
//     PWCHAR Buffer, BufEnd = NULL;
//     ULONG Length = 0;
//     WCHAR p;
//     //PWCHAR pp;
//
//     /* Check if we don't have a search path */
//     if (!ActualSearchPath) *SearchPath = LdrpDefaultPath.Buffer;
//
//     /* Display debug output if snaps are on */
//     if (ShowSnaps)
//     {
//         DbgPrintEx(DPFLTR_LDR_ID,
//                    DPFLTR_ERROR_LEVEL,
//                    "LDR: %s - Looking for %ws in %ws\n",
//                    __FUNCTION__,
//                    DllName,
//                    *SearchPath);
//     }
//
//     /* Check if we're dealing with a relative path */
//     if (RtlDetermineDosPathNameType_U(DllName) != RtlPathTypeRelative)
//     {
//         /* Good, we're not. Create the name string */
//         Status = RtlInitUnicodeStringEx(&TestName, DllName);
//         if (!NT_SUCCESS(Status)) goto Quickie;
//
//         /* Make sure it exists */
//         #if 0
//         if (!RtlDoesFileExists_UstrEx(&TestName, TRUE))
//         {
//             /* It doesn't, fail */
//             Status = STATUS_DLL_NOT_FOUND;
//             goto Quickie;
//         }
//         #endif
//
//         /* Resolve the full name */
//         Status = LdrpResolveFullName(&TestName,
//                                      PathName,
//                                      FullPathName,
//                                      ExpandedName);
//         goto Quickie;
//     }
//
//     /* FIXME: Handle relative case semicolon-lookup here */
//
//     /* Calculate length */
//     Length += (ULONG)wcslen(DllName) + 1;
//     if (Length > UNICODE_STRING_MAX_CHARS)
//     {
//         /* Too long, fail */
//         Status = STATUS_NAME_TOO_LONG;
//         goto Quickie;
//     }
//
//     /* Allocate buffer */
//     Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, Length * sizeof(WCHAR));
//     if (!Buffer)
//     {
//         /* Fail */
//         Status = STATUS_NO_MEMORY;
//         goto Quickie;
//     }
//
//     /* FIXME: Setup TestName here */
//     Status = STATUS_NOT_FOUND;
//     BufEnd = Buffer;
//
//     /* Start loop */
//     do
//     {
//         /* Get character */
//         p = *ActualSearchPath;
//         if (!(p) || (p == ';'))
//         {
//             /* FIXME: We don't have a character, or is a semicolon.*/
//
//             /* Display debug output if snaps are on */
//             if (ShowSnaps)
//             {
//                 DbgPrintEx(DPFLTR_LDR_ID,
//                            DPFLTR_ERROR_LEVEL,
//                            "LDR: %s - Looking for %ws\n",
//                            __FUNCTION__,
//                            Buffer);
//             }
//
//             /* Sanity check */
//             TestName.Length = (USHORT)ALIGN_DOWN((BufEnd - Buffer), WCHAR);
// #if 0
//             ASSERT(TestName.Length < TestName.MaximumLength);
// #endif
//
//             /* Check if the file exists */
//             #if 0
//             if (RtlDoesFileExists_UstrEx(&TestName, FALSE))
//             #endif
//             {
//                 /* It does. Reallocate the buffer */
//                 TestName.MaximumLength = (USHORT)ALIGN_DOWN((BufEnd - Buffer), WCHAR) + sizeof(WCHAR);
//                 TestName.Buffer = RtlReAllocateHeap(RtlGetProcessHeap(),
//                                                     0,
//                                                     Buffer,
//                                                     TestName.MaximumLength);
//                 if (!TestName.Buffer)
//                 {
//                     /* Keep the old one */
//                     TestName.Buffer = Buffer;
//                 }
//                 else
//                 {
//                     /* Update buffer */
//                     Buffer = TestName.Buffer;
//                 }
//
//                 /* Make sure we have a buffer at least */
//                 ASSERT(TestName.Buffer);
//
//                 /* Resolve the name */
//                 *SearchPath = ActualSearchPath++;
//                 Status = LdrpResolveFullName(&TestName,
//                                              PathName,
//                                              FullPathName,
//                                              ExpandedName);
//                 break;
//             }
//
//             /* Update buffer end */
//             BufEnd = Buffer;
//
//             /* Update string position */
//             //pp = ActualSearchPath++;
//         }
//         else
//         {
//             /* Otherwise, write the character */
//             *BufEnd = p;
//             BufEnd++;
//         }
//
//         /* Check if the string is empty, meaning we're done */
//         if (!(*ActualSearchPath)) TryAgain = TRUE;
//
//         /* Advance in the string */
//         ActualSearchPath++;
//     } while (!TryAgain);
//
//     /* Check if we had a buffer and free it */
//     if (Buffer) RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer);
//
// Quickie:
//     /* Check if we got here through failure */
//     if (!NT_SUCCESS(Status)) *ExpandedName = NULL;
//
//     /* Display debug output if snaps are on */
//     if (ShowSnaps)
//     {
//         /* Check which output to use -- failure or success */
//         if (NT_SUCCESS(Status))
//         {
//             DbgPrintEx(DPFLTR_LDR_ID,
//                        DPFLTR_ERROR_LEVEL,
//                        "LDR: %s - Returning %wZ\n",
//                        __FUNCTION__,
//                        *ExpandedName);
//         }
//         else
//         {
//             DbgPrintEx(DPFLTR_LDR_ID,
//                        DPFLTR_ERROR_LEVEL,
//                        "LDR: %s -  Unable to locate %ws in %ws: 0x%08x\n",
//                        __FUNCTION__,
//                        DllName,
//                        ActualSearchPath,
//                        Status);
//         }
//     }
//
//     /* Return status */
//     return Status;
// }
//
//
// /* NOTE: This function is b0rked and in the process of being slowly unf*cked */
// BOOLEAN
// NTAPI
// LdrpCheckForLoadedDll(IN PWSTR DllPath,
//                       IN PUNICODE_STRING DllName,
//                       IN BOOLEAN Flag,
//                       IN BOOLEAN RedirectedDll,
//                       OUT PLDR_DATA_TABLE_ENTRY *LdrEntry)
// {
//     ULONG HashIndex;
//     PLIST_ENTRY ListHead, ListEntry;
//     PLDR_DATA_TABLE_ENTRY CurEntry;
//     BOOLEAN FullPath = FALSE;
//     PWCHAR wc;
//     WCHAR NameBuf[266];
//     UNICODE_STRING FullDllName, NtPathName;
//     ULONG Length;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     NTSTATUS Status;
//     HANDLE FileHandle, SectionHandle;
//     IO_STATUS_BLOCK Iosb;
//     PVOID ViewBase = NULL;
//     SIZE_T ViewSize = 0;
//     PIMAGE_NT_HEADERS NtHeader, NtHeader2;
//     DPRINT("LdrpCheckForLoadedDll('%S' '%wZ' %u %u %p)\n", DllPath ? ((ULONG_PTR)DllPath == 1 ? L"" : DllPath) : L"", DllName, Flag, RedirectedDll, LdrEntry);
//
//     /* Check if a dll name was provided */
//     if (!(DllName->Buffer) || !(DllName->Buffer[0])) return FALSE;
//
//     /* FIXME: Warning, "Flag" is used as magic instead of "Static" */
//     /* FIXME: Warning, code does not support redirection at all */
//
//     /* Look in the hash table if flag was set */
// lookinhash:
//     if (Flag  /* the second check is a hack */ && !RedirectedDll)
//     {
//         /* FIXME: if we get redirected dll it means that we also get a full path so we need to find its filename for the hash lookup */
//
//         /* Get hash index */
//         HashIndex = LDR_GET_HASH_ENTRY(DllName->Buffer[0]);
//
//         /* Traverse that list */
//         ListHead = &LdrpHashTable[HashIndex];
//         ListEntry = ListHead->Flink;
//         while (ListEntry != ListHead)
//         {
//             /* Get the current entry */
//             CurEntry = CONTAINING_RECORD(ListEntry, LDR_DATA_TABLE_ENTRY, HashLinks);
//
//             /* Check base name of that module */
//             if (RtlEqualUnicodeString(DllName, &CurEntry->BaseDllName, TRUE))
//             {
//                 /* It matches, return it */
//                 *LdrEntry = CurEntry;
//                 return TRUE;
//             }
//
//             /* Advance to the next entry */
//             ListEntry = ListEntry->Flink;
//         }
//
//         /* Module was not found, return failure */
//         return FALSE;
//     }
//
//     /* Check if this is a redirected DLL */
//     if (RedirectedDll)
//     {
//         /* Redirected dlls already have a full path */
//         FullPath = TRUE;
//         FullDllName = *DllName;
//     }
//     else
//     {
//         /* Check if there is a full path in this DLL */
//         wc = DllName->Buffer;
//         while (*wc)
//         {
//             /* Check for a slash in the current position*/
//             if ((*wc == L'\\') || (*wc == L'/'))
//             {
//                 /* Found the slash, so dll name contains path */
//                 FullPath = TRUE;
//
//                 /* Setup full dll name string */
//                 FullDllName.Buffer = NameBuf;
//
//                 /* FIXME: This is from the Windows 2000 loader, not XP/2003, we should call LdrpSearchPath */
//                 Length = RtlDosSearchPath_U(DllPath ? DllPath : LdrpDefaultPath.Buffer,
//                                             DllName->Buffer,
//                                             NULL,
//                                             sizeof(NameBuf) - sizeof(UNICODE_NULL),
//                                             FullDllName.Buffer,
//                                             NULL);
//
//                 /* Check if that was successful */
//                 if (!(Length) || (Length > (sizeof(NameBuf) - sizeof(UNICODE_NULL))))
//                 {
//                     if (ShowSnaps)
//                     {
//                         DPRINT1("LDR: LdrpCheckForLoadedDll - Unable To Locate %wZ: 0x%08x\n",
//                             &DllName, Length);
//                     }
//                 }
//
//                 /* Full dll name is found */
//                 FullDllName.Length = Length;
//                 FullDllName.MaximumLength = FullDllName.Length + sizeof(UNICODE_NULL);
//                 break;
//             }
//
//             wc++;
//         }
//     }
//
//     /* Go check the hash table */
//     if (!FullPath)
//     {
//         Flag = TRUE;
//         goto lookinhash;
//     }
//
//     /* FIXME: Warning, activation context missing */
//     DPRINT("Warning, activation context missing\n");
//
//     /* NOTE: From here on down, everything looks good */
//
//     /* Loop the module list */
//     ListHead = &NtCurrentPeb()->Ldr->InLoadOrderModuleList;
//     ListEntry = ListHead->Flink;
//     while (ListEntry != ListHead)
//     {
//         /* Get the current entry and advance to the next one */
//         CurEntry = CONTAINING_RECORD(ListEntry,
//                                      LDR_DATA_TABLE_ENTRY,
//                                      InLoadOrderLinks);
//         ListEntry = ListEntry->Flink;
//
//         /* Check if it's being unloaded */
//         if (!CurEntry->InMemoryOrderLinks.Flink) continue;
//
//         /* Check if name matches */
//         if (RtlEqualUnicodeString(&FullDllName,
//                                   &CurEntry->FullDllName,
//                                   TRUE))
//         {
//             /* Found it */
//             *LdrEntry = CurEntry;
//             return TRUE;
//         }
//     }
//
//     /* Convert given path to NT path */
//     if (!RtlDosPathNameToNtPathName_U(FullDllName.Buffer,
//                                       &NtPathName,
//                                       NULL,
//                                       NULL))
//     {
//         /* Fail if conversion failed */
//         return FALSE;
//     }
//
//     /* Initialize object attributes and open it */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &NtPathName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//     Status = NtOpenFile(&FileHandle,
//                         SYNCHRONIZE | FILE_EXECUTE,
//                         &ObjectAttributes,
//                         &Iosb,
//                         FILE_SHARE_READ | FILE_SHARE_DELETE,
//                         FILE_NON_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT);
//
//     /* Free NT path name */
//     RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathName.Buffer);
//
//     /* If opening the file failed - return failure */
//     if (!NT_SUCCESS(Status)) return FALSE;
//
//     /* Create a section for this file */
//     Status = NtCreateSection(&SectionHandle,
//                              SECTION_MAP_READ |
//                              SECTION_MAP_EXECUTE |
//                              SECTION_MAP_WRITE,
//                              NULL,
//                              NULL,
//                              PAGE_EXECUTE,
//                              SEC_COMMIT,
//                              FileHandle);
//
//     /* Close file handle */
//     NtClose(FileHandle);
//
//     /* If creating section failed - return failure */
//     if (!NT_SUCCESS(Status)) return FALSE;
//
//     /* Map view of this section */
//     Status = ZwMapViewOfSection(SectionHandle,
//                                 NtCurrentProcess(),
//                                 &ViewBase,
//                                 0,
//                                 0,
//                                 NULL,
//                                 &ViewSize,
//                                 ViewShare,
//                                 0,
//                                 PAGE_EXECUTE);
//
//     /* Close section handle */
//     NtClose(SectionHandle);
//
//     /* If section mapping failed - return failure */
//     if (!NT_SUCCESS(Status)) return FALSE;
//
//     /* Get pointer to the NT header of this section */
//     Status = RtlImageNtHeaderEx(0, ViewBase, ViewSize, &NtHeader);
//     if (!(NT_SUCCESS(Status)) || !(NtHeader))
//     {
//         /* Unmap the section and fail */
//         NtUnmapViewOfSection(NtCurrentProcess(), ViewBase);
//         return FALSE;
//     }
//
//     /* Go through the list of modules again */
//     ListHead = &NtCurrentPeb()->Ldr->InLoadOrderModuleList;
//     ListEntry = ListHead->Flink;
//     while (ListEntry != ListHead)
//     {
//         /* Get the current entry and advance to the next one */
//         CurEntry = CONTAINING_RECORD(ListEntry,
//                                      LDR_DATA_TABLE_ENTRY,
//                                      InLoadOrderLinks);
//         ListEntry = ListEntry->Flink;
//
//         /* Check if it's in the process of being unloaded */
//         if (!CurEntry->InMemoryOrderLinks.Flink) continue;
//
//         /* The header is untrusted, use SEH */
//         _SEH2_TRY
//         {
//             /* Check if timedate stamp and sizes match */
//             if ((CurEntry->TimeDateStamp == NtHeader->FileHeader.TimeDateStamp) &&
//                 (CurEntry->SizeOfImage == NtHeader->OptionalHeader.SizeOfImage))
//             {
//                 /* Time, date and size match. Let's compare their headers */
//                 NtHeader2 = RtlImageNtHeader(CurEntry->DllBase);
//                 if (RtlCompareMemory(NtHeader2, NtHeader, sizeof(IMAGE_NT_HEADERS)))
//                 {
//                     /* Headers match too! Finally ask the kernel to compare mapped files */
//                     Status = ZwAreMappedFilesTheSame(CurEntry->DllBase, ViewBase);
//                     if (NT_SUCCESS(Status))
//                     {
//                         /* This is our entry!, unmap and return success */
//                         *LdrEntry = CurEntry;
//                         NtUnmapViewOfSection(NtCurrentProcess(), ViewBase);
//                         _SEH2_YIELD(return TRUE;)
//                     }
//                 }
//             }
//         }
//         _SEH2_EXCEPT (EXCEPTION_EXECUTE_HANDLER)
//         {
//             _SEH2_YIELD(break;)
//         }
//         _SEH2_END;
//     }
//
//     /* Unmap the section and fail */
//     NtUnmapViewOfSection(NtCurrentProcess(), ViewBase);
//     return FALSE;
// }
//
// NTSTATUS
// NTAPI
// LdrpGetProcedureAddress(
//     _In_ PVOID BaseAddress,
//     _In_opt_ _When_(Ordinal == 0, _Notnull_) PANSI_STRING Name,
//     _In_opt_ _When_(Name == NULL, _In_range_(>, 0)) ULONG Ordinal,
//     _Out_ PVOID *ProcedureAddress,
//     _In_ BOOLEAN ExecuteInit)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     UCHAR ImportBuffer[64]; // 128 since NT6.2
//     PLDR_DATA_TABLE_ENTRY LdrEntry;
//     IMAGE_THUNK_DATA Thunk;
//     PVOID ImageBase;
//     PIMAGE_IMPORT_BY_NAME ImportName = NULL;
//     PIMAGE_EXPORT_DIRECTORY ExportDir;
//     ULONG ExportDirSize, Length;
//     PLIST_ENTRY Entry;
//
//     /* Show debug message */
//     if (ShowSnaps) DPRINT1("LDR: LdrGetProcedureAddress by ");
//
//     /* Check if we got a name */
//     if (Name)
//     {
//         /* Show debug message */
//         if (ShowSnaps) DbgPrint("NAME - %s\n", Name->Buffer);
//
//         /* Make sure it's not too long */
//         Length = Name->Length +
//                  sizeof(CHAR) +
//                  FIELD_OFFSET(IMAGE_IMPORT_BY_NAME, Name);
//         if (Length > UNICODE_STRING_MAX_BYTES)
//         {
//             /* Won't have enough space to add the hint */
//             return STATUS_NAME_TOO_LONG;
//         }
//
//         /* Check if our buffer is large enough */
//         if (Length > sizeof(ImportBuffer))
//         {
//             /* Allocate from heap, plus 2 bytes for the Hint */
//             ImportName = RtlAllocateHeap(RtlGetProcessHeap(),
//                                          0,
//                                          Length);
//             if (!ImportName)
//             {
//                 /* Return STATUS_INSUFFICIENT_RESOURCES since NT6.2 */
//                 return STATUS_INVALID_PARAMETER;
//             }
//         }
//         else
//         {
//             /* Use our internal buffer */
//             ImportName = (PIMAGE_IMPORT_BY_NAME)ImportBuffer;
//         }
//
//         /* Clear the hint */
//         ImportName->Hint = 0;
//
//         /* Copy the name and null-terminate it */
//         RtlCopyMemory(ImportName->Name, Name->Buffer, Name->Length);
//         ImportName->Name[Name->Length] = ANSI_NULL;
//
//         /* Clear the high bit */
//         ImageBase = ImportName;
//         Thunk.u1.AddressOfData = 0;
//     }
//     else
//     {
//         /* Do it by ordinal */
//         ImageBase = NULL;
//
//         /* Show debug message */
//         if (ShowSnaps) DbgPrint("ORDINAL - %lx\n", Ordinal);
//
//         /* Make sure an ordinal was given */
//         if (!Ordinal)
//         {
//             /* No ordinal */
//             DPRINT1("No ordinal and no name\n");
//             return STATUS_INVALID_PARAMETER;
//         }
//
//         /* Set the original flag in the thunk */
//         Thunk.u1.Ordinal = Ordinal | IMAGE_ORDINAL_FLAG;
//     }
//
//     /* Acquire lock unless we are initting */
//     if (!LdrpInLdrInit) RtlEnterCriticalSection(&LdrpLoaderLock);
//
//     _SEH2_TRY
//     {
//         /* Try to find the loaded DLL */
//         if (!LdrpCheckForLoadedDllHandle(BaseAddress, &LdrEntry))
//         {
//             /* Invalid base */
//             DPRINT1("Invalid base address %p\n", BaseAddress);
//             Status = STATUS_DLL_NOT_FOUND;
//             _SEH2_YIELD(goto Quickie;)
//         }
//
//         /* Get the pointer to the export directory */
//         ExportDir = RtlImageDirectoryEntryToData(LdrEntry->DllBase,
//                                                  TRUE,
//                                                  IMAGE_DIRECTORY_ENTRY_EXPORT,
//                                                  &ExportDirSize);
//
//         if (!ExportDir)
//         {
//             DPRINT1("Image %wZ has no exports, but were trying to get procedure %Z. BaseAddress asked 0x%p, got entry BA 0x%p\n",
//                     &LdrEntry->BaseDllName, Name, BaseAddress, LdrEntry->DllBase);
//             Status = STATUS_PROCEDURE_NOT_FOUND;
//             _SEH2_YIELD(goto Quickie;)
//         }
//
//         /* Now get the thunk */
//         Status = LdrpSnapThunk(LdrEntry->DllBase,
//                                ImageBase,
//                                &Thunk,
//                                &Thunk,
//                                ExportDir,
//                                ExportDirSize,
//                                FALSE,
//                                NULL);
//
//         /* Finally, see if we're supposed to run the init routines */
//         if ((NT_SUCCESS(Status)) && (ExecuteInit))
//         {
//             /*
//             * It's possible a forwarded entry had us load the DLL. In that case,
//             * then we will call its DllMain. Use the last loaded DLL for this.
//             */
//             Entry = NtCurrentPeb()->Ldr->InInitializationOrderModuleList.Blink;
//             LdrEntry = CONTAINING_RECORD(Entry,
//                                          LDR_DATA_TABLE_ENTRY,
//                                          InInitializationOrderLinks);
//
//             /* Make sure we didn't process it yet*/
//             if (!(LdrEntry->Flags & LDRP_ENTRY_PROCESSED))
//             {
//                 /* Call the init routine */
//                 _SEH2_TRY
//                 {
//                     Status = LdrpRunInitializeRoutines(NULL);
//                 }
//                 _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                 {
//                     /* Get the exception code */
//                     Status = _SEH2_GetExceptionCode();
//                 }
//                 _SEH2_END;
//             }
//         }
//
//         /* Make sure we're OK till here */
//         if (NT_SUCCESS(Status))
//         {
//             /* Return the address */
//             *ProcedureAddress = (PVOID)Thunk.u1.Function;
//         }
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         /* Just ignore exceptions */
//     }
//     _SEH2_END;
//
// Quickie:
//     /* Cleanup */
//     if (ImportName && (ImportName != (PIMAGE_IMPORT_BY_NAME)ImportBuffer))
//     {
//         /* We allocated from heap, free it */
//         RtlFreeHeap(RtlGetProcessHeap(), 0, ImportName);
//     }
//
//     /* Release the CS if we entered it */
//     if (!LdrpInLdrInit) RtlLeaveCriticalSection(&LdrpLoaderLock);
//
//     /* We're done */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// LdrpLoadDll(IN BOOLEAN Redirected,
//             IN PWSTR DllPath OPTIONAL,
//             IN PULONG DllCharacteristics OPTIONAL,
//             IN PUNICODE_STRING DllName,
//             OUT PVOID *BaseAddress,
//             IN BOOLEAN CallInit)
// {
//     PPEB Peb = NtCurrentPeb();
//     NTSTATUS Status = STATUS_SUCCESS;
//     const WCHAR *p;
//     BOOLEAN GotExtension;
//     WCHAR c;
//     WCHAR NameBuffer[MAX_PATH + 6];
//     UNICODE_STRING RawDllName;
//     PLDR_DATA_TABLE_ENTRY LdrEntry;
//     BOOLEAN InInit = LdrpInLdrInit;
//
//     /* Save the Raw DLL Name */
//     if (DllName->Length >= sizeof(NameBuffer)) return STATUS_NAME_TOO_LONG;
//     RtlInitEmptyUnicodeString(&RawDllName, NameBuffer, sizeof(NameBuffer));
//     RtlCopyUnicodeString(&RawDllName, DllName);
//
//     /* Find the extension, if present */
//     /* NOTE: Access violation is expected here in some cases (Buffer[-1]) */
//     p = DllName->Buffer + DllName->Length / sizeof(WCHAR) - 1;
//     GotExtension = FALSE;
//     while (p >= DllName->Buffer)
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
//         if ((DllName->Length + LdrApiDefaultExtension.Length + sizeof(UNICODE_NULL)) >=
//             sizeof(NameBuffer))
//         {
//             /* No space to add the extension */
//             DbgPrintEx(DPFLTR_LDR_ID,
//                        DPFLTR_ERROR_LEVEL,
//                        "LDR: %s - Dll name missing extension; with extension "
//                        "added the name is too long\n"
//                        "   DllName: (@ %p) \"%wZ\"\n"
//                        "   DllName->Length: %u\n",
//                        __FUNCTION__,
//                        DllName,
//                        DllName,
//                        DllName->Length);
//             return STATUS_NAME_TOO_LONG;
//         }
//
//         /* Add it. Needs to be null terminated, thus the length check above */
//         (VOID)RtlAppendUnicodeStringToString(&RawDllName,
//                                              &LdrApiDefaultExtension);
//     }
//
//     /* Check for init flag and acquire lock */
//     if (!InInit) RtlEnterCriticalSection(&LdrpLoaderLock);
//
//     _SEH2_TRY
//     {
//         /* Show debug message */
//         if (ShowSnaps)
//         {
//             DPRINT1("LDR: LdrLoadDll, loading %wZ from %ws\n",
//                      &RawDllName,
//                      DllPath ? DllPath : L"");
//         }
//
//         /* Check if the DLL is already loaded */
//         if (!LdrpCheckForLoadedDll(DllPath,
//                                    &RawDllName,
//                                    FALSE,
//                                    Redirected,
//                                    &LdrEntry))
//         {
//             /* Map it */
//             Status = LdrpMapDll(DllPath,
//                                 DllPath,
//                                 NameBuffer,
//                                 DllCharacteristics,
//                                 FALSE,
//                                 Redirected,
//                                 &LdrEntry);
//             if (!NT_SUCCESS(Status))
//                 _SEH2_LEAVE;
//
//             /* FIXME: Need to mark the DLL range for the stack DB */
//             //RtlpStkMarkDllRange(LdrEntry);
//
//             /* Check if IMAGE_FILE_EXECUTABLE_IMAGE was provided */
//             if ((DllCharacteristics) &&
//                 (*DllCharacteristics & IMAGE_FILE_EXECUTABLE_IMAGE))
//             {
//                 /* This is not a DLL, so remove such data */
//                 LdrEntry->EntryPoint = NULL;
//                 LdrEntry->Flags &= ~LDRP_IMAGE_DLL;
//             }
//
//             /* Make sure it's a DLL */
//             if (LdrEntry->Flags & LDRP_IMAGE_DLL)
//             {
//                 /* Check if this is a .NET Image */
//                 if (!(LdrEntry->Flags & LDRP_COR_IMAGE))
//                 {
//                     /* Walk the Import Descriptor */
//                     Status = LdrpWalkImportDescriptor(DllPath, LdrEntry);
//                 }
//
//                 /* Update load count, unless it's locked */
//                 if (LdrEntry->LoadCount != 0xFFFF) LdrEntry->LoadCount++;
//                 LdrpUpdateLoadCount2(LdrEntry, LDRP_UPDATE_REFCOUNT);
//
//                 /* Check if we failed */
//                 if (!NT_SUCCESS(Status))
//                 {
//                     /* Clear entrypoint, and insert into list */
//                     LdrEntry->EntryPoint = NULL;
//                     InsertTailList(&Peb->Ldr->InInitializationOrderModuleList,
//                                    &LdrEntry->InInitializationOrderLinks);
//
//                     /* Cancel the load */
//                     LdrpClearLoadInProgress();
//
//                     /* Unload the DLL */
//                     if (ShowSnaps)
//                     {
//                         DbgPrint("LDR: Unloading %wZ due to error %x walking "
//                                  "import descriptors\n",
//                                  DllName,
//                                  Status);
//                     }
//                     LdrUnloadDll(LdrEntry->DllBase);
//
//                     /* Return the error */
//                     _SEH2_LEAVE;
//                 }
//             }
//             else if (LdrEntry->LoadCount != 0xFFFF)
//             {
//                 /* Increase load count */
//                 LdrEntry->LoadCount++;
//             }
//
//             /* Insert it into the list */
//             InsertTailList(&Peb->Ldr->InInitializationOrderModuleList,
//                            &LdrEntry->InInitializationOrderLinks);
//
//             /* If we have to run the entrypoint, make sure the DB is ready */
//             if (CallInit && LdrpLdrDatabaseIsSetup)
//             {
//                 /* Notify Shim Engine */
//                 if (g_ShimsEnabled)
//                 {
//                     VOID (NTAPI* SE_DllLoaded)(PLDR_DATA_TABLE_ENTRY) = RtlDecodeSystemPointer(g_pfnSE_DllLoaded);
//                     SE_DllLoaded(LdrEntry);
//                 }
//
//                 /* Run the init routine */
//                 Status = LdrpRunInitializeRoutines(NULL);
//                 if (!NT_SUCCESS(Status))
//                 {
//                     /* Failed, unload the DLL */
//                     if (ShowSnaps)
//                     {
//                         DbgPrint("LDR: Unloading %wZ because either its init "
//                                  "routine or one of its static imports failed; "
//                                  "status = 0x%08lx\n",
//                                  DllName,
//                                  Status);
//                     }
//                     LdrUnloadDll(LdrEntry->DllBase);
//                 }
//             }
//             else
//             {
//                 /* The DB isn't ready, which means we were loaded because of a forwarder */
//                 Status = STATUS_SUCCESS;
//             }
//         }
//         else
//         {
//             /* We were already loaded. Are we a DLL? */
//             if ((LdrEntry->Flags & LDRP_IMAGE_DLL) && (LdrEntry->LoadCount != 0xFFFF))
//             {
//                 /* Increase load count */
//                 LdrEntry->LoadCount++;
//                 LdrpUpdateLoadCount2(LdrEntry, LDRP_UPDATE_REFCOUNT);
//
//                 /* Clear the load in progress */
//                 LdrpClearLoadInProgress();
//             }
//             else
//             {
//                 /* Not a DLL, just increase the load count */
//                 if (LdrEntry->LoadCount != 0xFFFF) LdrEntry->LoadCount++;
//             }
//         }
//
//     }
//     _SEH2_FINALLY
//     {
//         /* Release the lock */
//         if (!InInit) RtlLeaveCriticalSection(&LdrpLoaderLock);
//     }
//     _SEH2_END;
//
//     /* Check for success */
//     if (NT_SUCCESS(Status))
//     {
//         /* Return the base address */
//         *BaseAddress = LdrEntry->DllBase;
//     }
//     else
//     {
//         /* Nothing found */
//         *BaseAddress = NULL;
//     }
//
//     /* Return status */
//     return Status;
// }
//
// ULONG
// NTAPI
// LdrpClearLoadInProgress(VOID)
// {
//     PLIST_ENTRY ListHead, Entry;
//     PLDR_DATA_TABLE_ENTRY LdrEntry;
//     ULONG ModulesCount = 0;
//
//     /* Traverse the init list */
//     ListHead = &NtCurrentPeb()->Ldr->InInitializationOrderModuleList;
//     Entry = ListHead->Flink;
//     while (Entry != ListHead)
//     {
//         /* Get the loader entry */
//         LdrEntry = CONTAINING_RECORD(Entry,
//                                      LDR_DATA_TABLE_ENTRY,
//                                      InInitializationOrderLinks);
//
//         /* Clear load in progress flag */
//         LdrEntry->Flags &= ~LDRP_LOAD_IN_PROGRESS;
//
//         /* Check for modules with entry point count but not processed yet */
//         if ((LdrEntry->EntryPoint) &&
//             !(LdrEntry->Flags & LDRP_ENTRY_PROCESSED))
//         {
//             /* Increase counter */
//             ModulesCount++;
//         }
//
//         /* Advance to the next entry */
//         Entry = Entry->Flink;
//     }
//
//     /* Return final count */
//     return ModulesCount;
// }
//
// PVOID LdrpGetShimEngineFunction(PCSZ FunctionName)
// {
//     ANSI_STRING Function;
//     NTSTATUS Status;
//     PVOID Address;
//     RtlInitAnsiString(&Function, FunctionName);
//     /* Skip Dll init */
//     Status = LdrpGetProcedureAddress(g_pShimEngineModule, &Function, 0, &Address, FALSE);
//     return NT_SUCCESS(Status) ? Address : NULL;
// }
//
// VOID
// NTAPI
// LdrpGetShimEngineInterface()
// {
//     PVOID SE_DllLoaded = LdrpGetShimEngineFunction("SE_DllLoaded");
//     PVOID SE_DllUnloaded = LdrpGetShimEngineFunction("SE_DllUnloaded");
//     PVOID SE_InstallBeforeInit = LdrpGetShimEngineFunction("SE_InstallBeforeInit");
//     PVOID SE_InstallAfterInit = LdrpGetShimEngineFunction("SE_InstallAfterInit");
//     PVOID SE_ProcessDying = LdrpGetShimEngineFunction("SE_ProcessDying");
//
//     if (SE_DllLoaded && SE_DllUnloaded && SE_InstallBeforeInit && SE_InstallAfterInit && SE_ProcessDying)
//     {
//         g_pfnSE_DllLoaded = RtlEncodeSystemPointer(SE_DllLoaded);
//         g_pfnSE_DllUnloaded = RtlEncodeSystemPointer(SE_DllUnloaded);
//         g_pfnSE_InstallBeforeInit = RtlEncodeSystemPointer(SE_InstallBeforeInit);
//         g_pfnSE_InstallAfterInit = RtlEncodeSystemPointer(SE_InstallAfterInit);
//         g_pfnSE_ProcessDying = RtlEncodeSystemPointer(SE_ProcessDying);
//         g_ShimsEnabled = TRUE;
//     }
//     else
//     {
//         LdrpUnloadShimEngine();
//     }
// }
//
// VOID
// NTAPI
// LdrpRunShimEngineInitRoutine(IN ULONG Reason)
// {
//     PLIST_ENTRY ListHead, Next;
//     PLDR_DATA_TABLE_ENTRY LdrEntry;
//
//     ListHead = &NtCurrentPeb()->Ldr->InLoadOrderModuleList;
//     Next = ListHead->Flink;
//     while (Next != ListHead)
//     {
//         LdrEntry = CONTAINING_RECORD(Next, LDR_DATA_TABLE_ENTRY, InLoadOrderLinks);
//
//         if (g_pShimEngineModule == LdrEntry->DllBase)
//         {
//             if (LdrEntry->EntryPoint)
//             {
//                 _SEH2_TRY
//                 {
//                     LdrpCallInitRoutine(LdrEntry->EntryPoint, LdrEntry->DllBase, Reason, NULL);
//                 }
//                 _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                 {
//                     DPRINT1("WARNING: Exception 0x%x during LdrpRunShimEngineInitRoutine(%u)\n",
//                             _SEH2_GetExceptionCode(), Reason);
//                 }
//                 _SEH2_END;
//             }
//             return;
//         }
//
//         Next = Next->Flink;
//     }
// }
//
// VOID
// NTAPI
// LdrpLoadShimEngine(IN PWSTR ImageName, IN PUNICODE_STRING ProcessImage, IN PVOID pShimData)
// {
//     UNICODE_STRING ShimLibraryName;
//     PVOID ShimLibrary;
//     NTSTATUS Status;
//     RtlInitUnicodeString(&ShimLibraryName, ImageName);
//     /* We should NOT pass CallInit = TRUE!
//        If we do this, other init routines will be called before we get a chance to shim stuff.. */
//     Status = LdrpLoadDll(FALSE, NULL, NULL, &ShimLibraryName, &ShimLibrary, FALSE);
//     if (NT_SUCCESS(Status))
//     {
//         g_pShimEngineModule = ShimLibrary;
//         LdrpRunShimEngineInitRoutine(DLL_PROCESS_ATTACH);
//         LdrpGetShimEngineInterface();
//         if (g_ShimsEnabled)
//         {
//             VOID(NTAPI *SE_InstallBeforeInit)(PUNICODE_STRING, PVOID);
//             SE_InstallBeforeInit = RtlDecodeSystemPointer(g_pfnSE_InstallBeforeInit);
//             SE_InstallBeforeInit(ProcessImage, pShimData);
//         }
//     }
// }
//
// VOID
// NTAPI
// LdrpUnloadShimEngine()
// {
//     /* Make sure we do not call into the shim engine anymore */
//     g_ShimsEnabled = FALSE;
//     LdrpRunShimEngineInitRoutine(DLL_PROCESS_DETACH);
//     LdrUnloadDll(g_pShimEngineModule);
//     g_pShimEngineModule = NULL;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
