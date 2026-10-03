use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{_UNICODE_STRING, IMAGE_LOAD_CONFIG_DIRECTORY, IMAGE_THUNK_DATA, LARGE_INTEGER, LDR_DATA_TABLE_ENTRY, LIST_ENTRY, PCCH, PCH, PCSTR, PCUNICODE_STRING, PIMAGE_EXPORT_DIRECTORY, PIMAGE_IMPORT_BY_NAME, PIMAGE_IMPORT_DESCRIPTOR, PIMAGE_LOAD_CONFIG_DIRECTORY, PIMAGE_NT_HEADERS, PIMAGE_SECTION_HEADER, PIMAGE_THUNK_DATA, PLDR_DATA_TABLE_ENTRY, PSTR, UINT32};
use core::ffi::c_void;
use core::cmp::min;
use crate::ffi::*;

// C: DBG_DEFAULT_CHANNEL(PELOADER) — the channel ERR/WARN/TRACE report on.
const DbgDefaultChannel: u32 = crate::frldr::DPRINT_PELOADER;
#[allow(unused_imports)] use crate::{TRACE, WARN, ERR, FIXME};


unsafe fn PeLdrpFetchAddressOfSecurityCookie(BaseAddress: PVOID, SizeOfImage: ULONG) -> PVOID {
    let ConfigDir: PIMAGE_LOAD_CONFIG_DIRECTORY;
    let mut DirSize: ULONG = 0;
    let mut Cookie: PULONG_PTR = core::ptr::null_mut();

    ConfigDir = RtlImageDirectoryEntryToData(
        BaseAddress,
        1u8,
        IMAGE_DIRECTORY_ENTRY_LOAD_CONFIG,
        &mut DirSize
    ) as PIMAGE_LOAD_CONFIG_DIRECTORY;

    if ConfigDir.is_null() ||
        (DirSize as usize) < core::mem::offset_of!(IMAGE_LOAD_CONFIG_DIRECTORY, SecurityCookie) + core::mem::size_of::<ULONGLONG>()
    {
        return core::ptr::null_mut();
    }

    Cookie = VaToPa((*ConfigDir).SecurityCookie as PULONG_PTR);

    if (Cookie as PCHAR) <= (BaseAddress as PCHAR) ||
        (Cookie as PCHAR) >= (BaseAddress as PCHAR).add(SizeOfImage as usize - core::mem::size_of::<ULONG_PTR>())
    {
        Cookie = core::ptr::null_mut();
    }

    Cookie as PVOID
}

unsafe fn PeLdrpCompareDllName(
    mut DllName: PCSTR,
    UnicodeName: PCUNICODE_STRING,
) -> u8 {
    let mut Buffer: PWSTR;
    let mut i: usize;
    let Length: usize;

    Length = strlen(DllName);

    #[cfg(debug_assertions)]
    {
        let mut UnicodeNamePA: _UNICODE_STRING = core::mem::zeroed();
        UnicodeNamePA.Length = (*UnicodeName).Length;
        UnicodeNamePA.MaximumLength = (*UnicodeName).MaximumLength;
        UnicodeNamePA.Buffer = VaToPa((*UnicodeName).Buffer);
        TRACE!(
            "PeLdrpCompareDllName: %s and %wZ, Length = %d, UN->Length %d\n",
            DllName,
            &UnicodeNamePA as *const _UNICODE_STRING,
            Length,
            (*UnicodeName).Length as core::ffi::c_uint,
        );
    }

    if (Length * core::mem::size_of::<u16>()) > (*UnicodeName).Length as usize {
        return 0u8;
    }

    Buffer = VaToPa((*UnicodeName).Buffer);

    i = 0;
    while i < Length {
        if toupper(*DllName as i32) != toupper(*Buffer as i32) {
            return 0u8;
        }

        DllName = DllName.add(1);
        Buffer = Buffer.add(1);
        i += 1;
    }

    if ((*UnicodeName).Length as usize == Length * core::mem::size_of::<u16>()) || (*Buffer == '.' as u16) {
        return 1u8;
    }

    0u8
}

unsafe fn PeLdrpBindImportName(
    ModuleListHead: PLIST_ENTRY,
    DllBase: *mut core::ffi::c_void,
    ImageBase: *mut core::ffi::c_void,
    ThunkName: PIMAGE_THUNK_DATA,
    ThunkData: PIMAGE_THUNK_DATA,
    ExportDirectory: PIMAGE_EXPORT_DIRECTORY,
    ExportSize: u32,
    ProcessForwards: u8,
    DirectoryPath: PCSTR,
    Parent: PLIST_ENTRY,
) -> u8 {
    let Ordinal: u32;
    let NameTable: *mut u32;
    let FunctionTable: *mut u32;
    let OrdinalTable: *mut u16;
    let mut High: i32;
    let mut Low: i32;
    let mut Middle: i32 = 0;
    let mut Result: i32;
    let Hint: u32;
    let ImportData: PIMAGE_IMPORT_BY_NAME;
    let mut ExportName: *mut i8;
    let _ForwarderName: *mut i8;
    let mut Success: u8;

    if DllBase.is_null() {
        WARN!("DllBase == NULL\n");
        return 0u8;
    }

    let ThunkName = VaToPa(ThunkName as *const core::ffi::c_void) as PIMAGE_THUNK_DATA;
    let ThunkData = VaToPa(ThunkData as *const core::ffi::c_void) as PIMAGE_THUNK_DATA;

    if IMAGE_SNAP_BY_ORDINAL((*ThunkName).u1.Ordinal as usize) != 0 && ProcessForwards == 0 {
        Ordinal = (IMAGE_ORDINAL((*ThunkName).u1.Ordinal as usize) - (*ExportDirectory).Base as usize) as u32;
    } else {
        let ImportDataVa: usize;

        if ProcessForwards == 0 {
            ImportDataVa = RVA(ImageBase, (*ThunkName).u1.AddressOfData as u32) as usize;
        } else {
            ImportDataVa = (*ThunkName).u1.AddressOfData as usize;
        }

        ImportData = VaToPa(ImportDataVa as *const core::ffi::c_void) as PIMAGE_IMPORT_BY_NAME;

        NameTable = VaToPa(RVA(DllBase, (*ExportDirectory).AddressOfNames)) as *mut u32;
        OrdinalTable = VaToPa(RVA(DllBase, (*ExportDirectory).AddressOfNameOrdinals)) as *mut u16;

        Hint = (*ImportData).Hint as u32;

        ExportName = VaToPa(RVA(DllBase, *NameTable.offset(Hint as isize))) as *mut i8;

        if (Hint < (*ExportDirectory).NumberOfNames) &&
            (strcmp(ExportName, (*ImportData).Name.as_ptr() as *const i8) == 0)
        {
            Ordinal = *OrdinalTable.offset(Hint as isize) as u32;
        } else {
            Low = 0;
            High = ((*ExportDirectory).NumberOfNames - 1) as i32;

            while High >= Low {
                Middle = (Low + High) / 2;

                ExportName = VaToPa(RVA(DllBase, *NameTable.offset(Middle as isize))) as *mut i8;

                Result = strcmp(ExportName, (*ImportData).Name.as_ptr() as *const i8);

                if Result > 0 {
                    High = Middle - 1;
                } else if Result < 0 {
                    Low = Middle + 1;
                } else {
                    break;
                }
            }

            if High < Low {
                ERR!("Did not find export '%s'!\n", (*ImportData).Name.as_ptr());
                return 0u8;
            }

            Ordinal = *OrdinalTable.offset(Middle as isize) as u32;
        }
    }

    if Ordinal >= (*ExportDirectory).NumberOfFunctions {
        ERR!("Ordinal number is invalid!\n");
        return 0u8;
    }

    FunctionTable = VaToPa(RVA(DllBase, (*ExportDirectory).AddressOfFunctions)) as *mut u32;

    (*ThunkData).u1.Function = RVA(DllBase, *FunctionTable.offset(Ordinal as isize)) as usize as _;

    let ForwarderName = VaToPa((*ThunkData).u1.Function as *const core::ffi::c_void) as *mut i8;
    if ((ForwarderName as usize) > (ExportDirectory as usize)) &&
        ((ForwarderName as usize) < ((ExportDirectory as usize) + ExportSize as usize))
    {
        let mut DataTableEntry: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
        let RefExportDirectory: PIMAGE_EXPORT_DIRECTORY;
        let mut RefExportSize: u32 = 0;
        let mut ForwardDllName: [i8; 256] = [0; 256];

        TRACE!("PeLdrpBindImportName(): ForwarderName %s\n", ForwarderName);

        core::ptr::copy_nonoverlapping(ForwarderName, ForwardDllName.as_mut_ptr() as *mut CHAR, 256);

        *strrchr(ForwardDllName.as_mut_ptr() as *mut CHAR, '.' as i32) = 0;

        if ForwardDllName[0] == '_' as i8 {
            let mut i: u32 = 0;
            while ForwardDllName[i as usize] != 0 {
                ForwardDllName[i as usize] = ForwardDllName[(i + 1) as usize];
                i += 1;
            }
        }

        if PeLdrCheckForLoadedDll(ModuleListHead, ForwardDllName.as_ptr() as PCSTR, core::ptr::addr_of_mut!(DataTableEntry)) == 0 {
            if strchr(ForwardDllName.as_ptr(), '.' as i32).is_null() {
                RtlStringCbCatA(ForwardDllName.as_mut_ptr() as *mut CHAR, 256, b".dll\0".as_ptr() as *const i8);
            }

            Success = PeLdrpLoadAndScanReferencedDll(
                ModuleListHead,
                DirectoryPath,
                ForwardDllName.as_mut_ptr() as *mut CHAR,
                Parent,
                core::ptr::addr_of_mut!(DataTableEntry),
            );
            if Success == 0 {
                ERR!("PeLdrpLoadAndScanReferencedDll() failed to load forwarder dll.\n");
                return Success;
            }
        }

        RefExportDirectory = RtlImageDirectoryEntryToData(
            VaToPa((*DataTableEntry).DllBase as *const core::ffi::c_void),
            1u8,
            0u16,
            core::ptr::addr_of_mut!(RefExportSize),
        ) as PIMAGE_EXPORT_DIRECTORY;

        if !RefExportDirectory.is_null() {
            let mut Buffer: [u8; 128] = [0; 128];
            let mut RefThunkData: IMAGE_THUNK_DATA = core::mem::zeroed();
            let ImportByName: PIMAGE_IMPORT_BY_NAME;
            let ImportName: *mut i8;

            ImportName = strrchr(ForwarderName, '.' as i32).offset(1);

            ImportByName = Buffer.as_mut_ptr() as PIMAGE_IMPORT_BY_NAME;

            core::ptr::copy_nonoverlapping(
                ImportName as *const u8,
                (*ImportByName).Name.as_mut_ptr(),
                strlen(ImportName) + 1,
            );

            (*ImportByName).Hint = 0;

            RefThunkData.u1.AddressOfData = ImportByName as usize as _;

            Success = PeLdrpBindImportName(
                ModuleListHead,
                (*DataTableEntry).DllBase as *mut core::ffi::c_void,
                ImageBase,
                core::ptr::addr_of_mut!(RefThunkData),
                core::ptr::addr_of_mut!(RefThunkData),
                RefExportDirectory,
                RefExportSize,
                1u8,
                DirectoryPath,
                Parent,
            );

            (*ThunkData).u1 = RefThunkData.u1;

            return Success;
        } else {
            return 0u8;
        }
    }

    1u8
}

unsafe fn PeLdrpLoadAndScanReferencedDll(
    ModuleListHead: PLIST_ENTRY,
    DirectoryPath: PCCH,
    ImportName: PCH,
    Parent: PLIST_ENTRY,
    DataTableEntry: *mut PLDR_DATA_TABLE_ENTRY,
) -> BOOLEAN {
    let mut FullDllName: [u8; 256] = [0; 256];
    let mut Success: BOOLEAN;
    let mut BasePA: PVOID = core::ptr::null_mut();

    RtlStringCbCopyA(FullDllName.as_mut_ptr() as *mut CHAR, core::mem::size_of_val(&FullDllName), DirectoryPath);
    RtlStringCbCatA(FullDllName.as_mut_ptr() as *mut CHAR, core::mem::size_of_val(&FullDllName), ImportName);

    TRACE!("Loading referenced DLL: %s\n", FullDllName.as_ptr());

    if let Some(cb) = PeLdrImportDllLoadCallback {
        cb(FullDllName.as_ptr() as *const CHAR);
    }

    Success = PeLdrLoadImage(FullDllName.as_ptr() as PCSTR, LoaderBootDriver, core::ptr::addr_of_mut!(BasePA));
    if Success == 0 {
        ERR!("PeLdrLoadImage('%s') failed\n", FullDllName.as_ptr());
        return Success;
    }

    Success = PeLdrAllocateDataTableEntry(
        if !Parent.is_null() { (*Parent).Blink } else { ModuleListHead },
        ImportName,
        FullDllName.as_ptr() as PCCH,
        PaToVa(BasePA),
        DataTableEntry,
    );
    if Success == 0 {
        ERR!("PeLdrAllocateDataTableEntry('%s') failed\n", FullDllName.as_ptr());
        MmFreeMemory(BasePA);
        return Success;
    }

    PeLdrInitSecurityCookie(*DataTableEntry);

    (**DataTableEntry).Flags |= LDRP_DRIVER_DEPENDENT_DLL;

    TRACE!(
        "PeLdrScanImportDescriptorTable() calling ourselves for '%.*S'\n",
        (**DataTableEntry).BaseDllName.Length as i32 / core::mem::size_of::<WCHAR>() as i32,
        VaToPa((**DataTableEntry).BaseDllName.Buffer as PVOID),
    );
    Success = PeLdrScanImportDescriptorTable(ModuleListHead, DirectoryPath, *DataTableEntry);
    if Success == 0 {
        ERR!("PeLdrScanImportDescriptorTable() failed\n");
        PeLdrFreeDataTableEntry(*DataTableEntry);
        MmFreeMemory(BasePA);
        return Success;
    }

    1u8
}

// _TYPE_OF_MEMORY_LoaderBootDriver = 11 (NOT 2). 2 is LoaderFree! Treating
// boot-driver allocations as "free" made MmFindAvailablePages reuse the same
// pages for subsequent drivers — usbport.sys's export table got clobbered
// by usbhub.sys at the same pa=0xE7E000, so usbohci.sys's import scan saw
// a NULL export directory for usbport and BD-failed (then UiMessageBox
// "Cannot load boot driver" hung the smoke harness).
const LoaderBootDriver: TYPE_OF_MEMORY = 11;
const LDRP_DRIVER_DEPENDENT_DLL: u32 = 0x04000000; // ndk/ldrtypes.h:60 (was 0x40, an undefined bit the kernel never tests)
type TYPE_OF_MEMORY = u32;
type BOOLEAN = u8;

unsafe fn PeLdrpScanImportAddressTable(
    ModuleListHead: PLIST_ENTRY,
    DllBase: *mut core::ffi::c_void,
    ImageBase: *mut core::ffi::c_void,
    ThunkName: PIMAGE_THUNK_DATA,
    ThunkData: PIMAGE_THUNK_DATA,
    DirectoryPath: PCSTR,
    Parent: PLIST_ENTRY,
) -> u8 {

    let mut ExportDirectory: PIMAGE_EXPORT_DIRECTORY = core::ptr::null_mut();
    let mut Success: u8;
    let mut ExportSize: u32 = 0;
    let mut ThunkNamePtr = ThunkName;
    let mut ThunkDataPtr = ThunkData;

    TRACE!(
        "PeLdrpScanImportAddressTable(): DllBase 0x%p, ImageBase 0x%p, ThunkName 0x%p, ThunkData 0x%p\n",
        DllBase,
        ImageBase,
        ThunkName,
        ThunkData,
    );

    if DllBase.is_null() {
        ERR!("DllBase == NULL\n");
        return 0u8;
    } else {
        // 0 = IMAGE_DIRECTORY_ENTRY_EXPORT (NOT 1 = IMPORT).
        // rustconv translated the C `IMAGE_DIRECTORY_ENTRY_EXPORT` macro
        // as `1u16`, the wrong constant, which made every import lookup
        // hit a NULL export table and silently fail.
        ExportDirectory = RtlImageDirectoryEntryToData(
            VaToPa(DllBase),
            1u8,
            0u16,
            &mut ExportSize,
        ) as PIMAGE_EXPORT_DIRECTORY;
    }

    TRACE!(
        "PeLdrpScanImportAddressTable(): ExportDirectory 0x%p\n",
        ExportDirectory,
    );

    if ExportDirectory.is_null() {
        ERR!(
            "No ExportDir, DllBase = %p (%p)\n",
            DllBase,
            VaToPa(DllBase),
        );
        return 0u8;
    }

    while (*(VaToPa(ThunkNamePtr as *mut core::ffi::c_void) as PIMAGE_THUNK_DATA))
        .u1
        .AddressOfData
        != 0
    {
        Success = PeLdrpBindImportName(
            ModuleListHead,
            DllBase,
            ImageBase,
            ThunkNamePtr,
            ThunkDataPtr,
            ExportDirectory,
            ExportSize,
            0u8,
            DirectoryPath,
            Parent,
        );

        if Success == 0 {
            return Success;
        }

        ThunkNamePtr = ThunkNamePtr.offset(1);
        ThunkDataPtr = ThunkDataPtr.offset(1);
    }

    return 1u8;
}

#[no_mangle]
pub unsafe extern "C" fn PeLdrInitializeModuleList() -> BOOLEAN {
    crate::frldr::dbgcon(b'P'); crate::frldr::dbgcon(b'I');
    let mut FreeldrDTE: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();

    InitializeListHead(core::ptr::addr_of_mut!(FrLdrModuleList));
    crate::frldr::dbgcon(b'L');

    if PeLdrAllocateDataTableEntry(
        core::ptr::addr_of_mut!(FrLdrModuleList),
        b"freeldr.sys\0".as_ptr() as PCCH,
        b"freeldr.sys\0".as_ptr() as PCCH,
        core::ptr::addr_of!(__ImageBase) as PVOID,
        core::ptr::addr_of_mut!(FreeldrDTE),
    ) == 0u8
    {
        ERR!("Failed to allocate DTE for freeldr\n");
        return 0u8;
    }
    crate::frldr::dbgcon(b'l');

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn PeLdrInitSecurityCookie(LdrEntry: PLDR_DATA_TABLE_ENTRY) -> PVOID {
    let Cookie: PULONG_PTR;
    let mut NewCookie: ULONG_PTR;

    Cookie = PeLdrpFetchAddressOfSecurityCookie(
        VaToPa((*LdrEntry).DllBase as PULONG_PTR) as PVOID,
        (*LdrEntry).SizeOfImage
    ) as PULONG_PTR;

    if Cookie.is_null() {
        return core::ptr::null_mut();
    }

    if (*Cookie == DEFAULT_SECURITY_COOKIE) || (*Cookie == 0) {
        NewCookie = Cookie as ULONG_PTR ^ ArcGetRelativeTime() as ULONG_PTR;
        #[cfg(target_pointer_width = "64")]
        {
            if NewCookie > COOKIE_MAX {
                NewCookie >>= 16;
            }
        }
        if (NewCookie == 0) || (NewCookie == *Cookie) {
            NewCookie = DEFAULT_SECURITY_COOKIE + 1;
        }

        *Cookie = NewCookie;
    }

    Cookie as PVOID
}

#[no_mangle]
pub unsafe extern "C" fn PeLdrCheckForLoadedDll(
    ModuleListHead: PLIST_ENTRY,
    DllName: PCSTR,
    LoadedEntry: *mut PLDR_DATA_TABLE_ENTRY,
) -> u8 {
    let mut ModuleEntry: PLIST_ENTRY;
    let _DataTableEntry: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();

    TRACE!("PeLdrCheckForLoadedDll: DllName %s\n", DllName);

    ModuleEntry = (*ModuleListHead).Flink;
    while ModuleEntry != ModuleListHead {
        let DataTableEntry = (ModuleEntry as *const u8).offset(
            -(core::mem::offset_of!(LDR_DATA_TABLE_ENTRY, InLoadOrderLinks) as isize)
        ) as PLDR_DATA_TABLE_ENTRY;

        TRACE!(
            "PeLdrCheckForLoadedDll: DTE %p, EP %p, Base %p, Name '%.*S'\n",
            DataTableEntry,
            (*DataTableEntry).EntryPoint,
            (*DataTableEntry).DllBase,
            (*DataTableEntry).BaseDllName.Length as i32 / core::mem::size_of::<u16>() as i32,
            VaToPa((*DataTableEntry).BaseDllName.Buffer as *const core::ffi::c_void),
        );

        if PeLdrpCompareDllName(DllName, core::ptr::addr_of!((*DataTableEntry).BaseDllName)) != 0 {
            *LoadedEntry = DataTableEntry;
            (*DataTableEntry).LoadCount += 1;
            TRACE!("PeLdrCheckForLoadedDll: LoadedEntry 0x%p\n", DataTableEntry);
            return 1u8;
        }

        ModuleEntry = (*ModuleEntry).Flink;
    }

    return 0u8;
}

#[no_mangle]
pub unsafe extern "C" fn PeLdrScanImportDescriptorTable(
    ModuleListHead: PLIST_ENTRY,
    DirectoryPath: PCCH,
    ScanDTE: PLDR_DATA_TABLE_ENTRY,
) -> BOOLEAN {

    let DataTableEntry: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
    let mut ImportTable: PIMAGE_IMPORT_DESCRIPTOR;
    let mut ImportTableSize: u32 = 0;
    let mut ImportName: PCH;
    let mut Success: BOOLEAN;

    ImportTable = RtlImageDirectoryEntryToData(
        VaToPa((*ScanDTE).DllBase),
        1u8,
        1u16,
        &mut ImportTableSize as *mut u32,
    ) as PIMAGE_IMPORT_DESCRIPTOR;

    #[cfg(debug_assertions)]
    {
        let mut BaseName: _UNICODE_STRING = core::mem::zeroed();
        BaseName.Buffer = VaToPa((*ScanDTE).BaseDllName.Buffer as *mut core::ffi::c_void) as *mut u16;
        BaseName.MaximumLength = (*ScanDTE).BaseDllName.MaximumLength;
        BaseName.Length = (*ScanDTE).BaseDllName.Length;
        TRACE!(
            "PeLdrScanImportDescriptorTable(): %wZ ImportTable = 0x%p\n",
            &BaseName as *const _UNICODE_STRING,
            ImportTable,
        );
    }

    if ImportTable.is_null() {
        return 1u8;
    }

    loop {
        if (*ImportTable).Name == 0 || (*ImportTable).__bindgen_anon_1.OriginalFirstThunk == 0 {
            break;
        }

        let ThunkName: PIMAGE_THUNK_DATA = RVA((*ScanDTE).DllBase, (*ImportTable).__bindgen_anon_1.OriginalFirstThunk) as PIMAGE_THUNK_DATA;
        let ThunkData: PIMAGE_THUNK_DATA = RVA((*ScanDTE).DllBase, (*ImportTable).FirstThunk) as PIMAGE_THUNK_DATA;

        ImportName = VaToPa(RVA((*ScanDTE).DllBase, (*ImportTable).Name)) as PCH;
        TRACE!("PeLdrScanImportDescriptorTable(): Looking at %s\n", ImportName);

        if PeLdrpCompareDllName(ImportName, &(*ScanDTE).BaseDllName as *const _UNICODE_STRING) != 0 {
            ImportTable = ImportTable.offset(1);
            continue;
        }

        if PeLdrCheckForLoadedDll(ModuleListHead, ImportName, &DataTableEntry as *const PLDR_DATA_TABLE_ENTRY as *mut PLDR_DATA_TABLE_ENTRY) == 0 {
            Success = PeLdrpLoadAndScanReferencedDll(
                ModuleListHead,
                DirectoryPath,
                ImportName,
                &(*ScanDTE).InLoadOrderLinks as *const LIST_ENTRY as PLIST_ENTRY,
                &DataTableEntry as *const PLDR_DATA_TABLE_ENTRY as *mut PLDR_DATA_TABLE_ENTRY,
            );
            if Success == 0 {
                ERR!("PeLdrpLoadAndScanReferencedDll() failed\n");
                return Success;
            }
        }

        Success = PeLdrpScanImportAddressTable(
            ModuleListHead,
            (*DataTableEntry).DllBase,
            (*ScanDTE).DllBase,
            ThunkName,
            ThunkData,
            DirectoryPath,
            &(*ScanDTE).InLoadOrderLinks as *const LIST_ENTRY as PLIST_ENTRY,
        );

        if Success == 0 {
            ERR!(
                "PeLdrpScanImportAddressTable() failed: ImportName = '%s', DirectoryPath = '%s'\n",
                ImportName,
                DirectoryPath,
            );
            return Success;
        }

        ImportTable = ImportTable.offset(1);
    }

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn PeLdrAllocateDataTableEntry(
    ModuleListHead: PLIST_ENTRY,
    BaseDllName: PCCH,
    FullDllName: PCCH,
    BaseVA: PVOID,
    NewEntry: *mut PLDR_DATA_TABLE_ENTRY,
) -> BOOLEAN {
    crate::frldr::dbgcon(b'a'); crate::frldr::dbgcon(b'1');
    let BasePA = VaToPa(BaseVA);
    let BaseDllNameBuffer: PWSTR;
    let mut Buffer: PWSTR;
    let mut DataTableEntry: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
    let NtHeaders: PIMAGE_NT_HEADERS;
    let mut Length: USHORT;

    TRACE!(
        "PeLdrAllocateDataTableEntry('%s', '%s', %p)\n",
        BaseDllName,
        FullDllName,
        BasePA,
    );

    crate::frldr::dbgcon(b'a'); crate::frldr::dbgcon(b'2');
    DataTableEntry = FrLdrHeapAlloc(
        core::mem::size_of::<LDR_DATA_TABLE_ENTRY>(),
        TAG_WLDR_DTE,
    ) as PLDR_DATA_TABLE_ENTRY;
    crate::frldr::dbgcon(b'a'); crate::frldr::dbgcon(b'3');
    if DataTableEntry.is_null() {
        crate::frldr::dbgcon(b'N');
        return 0u8;
    }
    // Heap dump after successful alloc. Gated off for `rosload` — when this
    // crate builds for rosload.exe the heap module is excluded (FrLdrHeap*
    // comes from freeldr.sys via importlib) so the HEAP/PHEAP types and the
    // FrLdrDefaultHeap static aren't visible here.
    #[cfg(not(feature = "rosload"))]
    {
        let dh = crate::heap::FrLdrDefaultHeap as crate::heap::PHEAP;
        if !dh.is_null() {
            crate::frldr::dbgcon_str(b"{na=");
            crate::frldr::dbgcon_hex32((*dh).NumAllocs);
            let term = core::ptr::addr_of_mut!((*dh).Blocks).offset((*dh).TerminatingBlock as isize);
            let mut walk: u32 = (*(*term).Data.as_mut_ptr()).Flink;
            let mut depth = 0u32;
            crate::frldr::dbgcon_str(b",fl=");
            while depth < 5 {
                let blk = core::ptr::addr_of_mut!((*dh).Blocks).offset(walk as isize);
                crate::frldr::dbgcon_hex32(walk);
                crate::frldr::dbgcon(b':');
                crate::frldr::dbgcon_hex32((*blk).Size as u32);
                crate::frldr::dbgcon(b' ');
                if (*blk).Size == 0 { break; }
                walk = (*(*blk).Data.as_mut_ptr()).Flink;
                depth += 1;
            }
            crate::frldr::dbgcon_str(b"}");
        }
    }
    crate::frldr::dbgcon(b'a'); crate::frldr::dbgcon(b'4');

    NtHeaders = RtlImageNtHeader(BasePA);
    crate::frldr::dbgcon(b'a'); crate::frldr::dbgcon(b'5');

    RtlZeroMemory(
        DataTableEntry as PVOID,
        core::mem::size_of::<LDR_DATA_TABLE_ENTRY>(),
    );
    crate::frldr::dbgcon(b'a'); crate::frldr::dbgcon(b'6');
    (*DataTableEntry).DllBase = BaseVA;
    (*DataTableEntry).SizeOfImage = (*NtHeaders).OptionalHeader.SizeOfImage;
    (*DataTableEntry).EntryPoint = RVA(BaseVA, (*NtHeaders).OptionalHeader.AddressOfEntryPoint);
    (*DataTableEntry).__bindgen_anon_1.__bindgen_anon_1.SectionPointer = core::ptr::null_mut();
    (*DataTableEntry).__bindgen_anon_1.__bindgen_anon_1.CheckSum = (*NtHeaders).OptionalHeader.CheckSum;
    crate::frldr::dbgcon(b'a'); crate::frldr::dbgcon(b'7');

    Length = (strlen(BaseDllName) * core::mem::size_of::<WCHAR>()) as USHORT;
    crate::frldr::dbgcon(b'a'); crate::frldr::dbgcon(b'8');
    Buffer = FrLdrHeapAlloc(Length as usize, TAG_WLDR_NAME) as PWSTR;
    crate::frldr::dbgcon(b'a'); crate::frldr::dbgcon(b'9');
    if Buffer.is_null() {
        FrLdrHeapFree(DataTableEntry as PVOID, TAG_WLDR_DTE);
        return 0u8;
    }

    BaseDllNameBuffer = Buffer;

    (*DataTableEntry).BaseDllName.Length = Length;
    (*DataTableEntry).BaseDllName.MaximumLength = Length;
    (*DataTableEntry).BaseDllName.Buffer = PaToVa(Buffer as PVOID) as PWSTR;

    RtlZeroMemory(Buffer as PVOID, Length as usize);
    Length /= core::mem::size_of::<WCHAR>() as USHORT;
    let mut base_ptr = BaseDllName;
    let mut buf_ptr = Buffer;
    while Length > 0 {
        *buf_ptr = *base_ptr as WCHAR;
        buf_ptr = buf_ptr.add(1);
        base_ptr = base_ptr.add(1);
        Length -= 1;
    }

    Length = (strlen(FullDllName) * core::mem::size_of::<WCHAR>()) as USHORT;
    Buffer = FrLdrHeapAlloc(Length as usize, TAG_WLDR_NAME) as PWSTR;
    if Buffer.is_null() {
        FrLdrHeapFree(BaseDllNameBuffer as PVOID, TAG_WLDR_NAME);
        FrLdrHeapFree(DataTableEntry as PVOID, TAG_WLDR_DTE);
        return 0u8;
    }

    (*DataTableEntry).FullDllName.Length = Length;
    (*DataTableEntry).FullDllName.MaximumLength = Length;
    (*DataTableEntry).FullDllName.Buffer = PaToVa(Buffer as PVOID) as PWSTR;

    RtlZeroMemory(Buffer as PVOID, Length as usize);
    Length /= core::mem::size_of::<WCHAR>() as USHORT;
    let mut full_ptr = FullDllName;
    let mut buf_ptr = Buffer;
    while Length > 0 {
        *buf_ptr = *full_ptr as WCHAR;
        buf_ptr = buf_ptr.add(1);
        full_ptr = full_ptr.add(1);
        Length -= 1;
    }

    (*DataTableEntry).Flags = LDRP_ENTRY_PROCESSED;
    (*DataTableEntry).LoadCount = 1;

    if ((*NtHeaders).OptionalHeader.DllCharacteristics & IMAGE_DLLCHARACTERISTICS_FORCE_INTEGRITY) != 0 {
        (*DataTableEntry).Flags |= LDRP_IMAGE_INTEGRITY_FORCED;
    }

    InsertTailList(ModuleListHead, core::ptr::addr_of_mut!((*DataTableEntry).InLoadOrderLinks));
    TRACE!(
        "Inserting DTE %p, name='%.*S' DllBase=%p\n",
        DataTableEntry,
        ((*DataTableEntry).BaseDllName.Length / core::mem::size_of::<WCHAR>() as USHORT)
            as core::ffi::c_uint,
        VaToPa((*DataTableEntry).BaseDllName.Buffer as PVOID),
        (*DataTableEntry).DllBase,
    );

    *NewEntry = DataTableEntry;

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn PeLdrFreeDataTableEntry(Entry: PLDR_DATA_TABLE_ENTRY) {
    RemoveEntryList(core::ptr::addr_of_mut!((*Entry).InLoadOrderLinks));
    FrLdrHeapFree(VaToPa((*Entry).FullDllName.Buffer as *mut core::ffi::c_void), TAG_WLDR_NAME);
    FrLdrHeapFree(VaToPa((*Entry).BaseDllName.Buffer as *mut core::ffi::c_void), TAG_WLDR_NAME);
    FrLdrHeapFree(Entry as *mut core::ffi::c_void, TAG_WLDR_DTE);
}

const TAG_WLDR_NAME: u32 = 0x4E524C57;
const TAG_WLDR_DTE: u32 = 0x45544457;

#[no_mangle]
pub unsafe extern "C" fn PeLdrLoadImageEx(
    FilePath: PCSTR,
    MemoryType: TYPE_OF_MEMORY,
    ImageBasePA: *mut PVOID,
    KernelMapping: BOOLEAN,
) -> BOOLEAN {
    let mut FileId: ULONG = 0;
    let mut PhysicalBase: PVOID;
    let mut VirtualBase: PVOID = core::ptr::null_mut();
    let mut HeadersBuffer: [u8; SECTOR_SIZE as usize * 2] = [0; SECTOR_SIZE as usize * 2];
    let mut NtHeaders: PIMAGE_NT_HEADERS;
    let mut SectionHeader: PIMAGE_SECTION_HEADER;
    let mut VirtualSize: ULONG;
    let mut SizeOfRawData: ULONG;
    let NumberOfSections: ULONG;
    let mut Status: ARC_STATUS;
    let mut Position: LARGE_INTEGER = core::mem::zeroed();
    let mut i: ULONG;
    let mut BytesRead: ULONG = 0;

    TRACE!("PeLdrLoadImage('%s', %ld)\n", FilePath, MemoryType);

    Status = ArcOpen(FilePath as PSTR, OpenReadOnly, &mut FileId);
    if Status != ESUCCESS {
        WARN!("ArcOpen('%s') failed. Status: %u\n", FilePath, Status);
        return 0u8;
    }

    Status = ArcRead(FileId, HeadersBuffer.as_mut_ptr() as *mut core::ffi::c_void, (SECTOR_SIZE * 2) as ULONG, &mut BytesRead);
    if Status != ESUCCESS {
        ERR!("ArcRead('%s') failed. Status: %u\n", FilePath, Status);
        ArcClose(FileId);
        return 0u8;
    }

    NtHeaders = RtlImageNtHeader(HeadersBuffer.as_mut_ptr() as *mut core::ffi::c_void);
    if NtHeaders.is_null() {
        ERR!("No NT header found in \"%s\"\n", FilePath);
        ArcClose(FileId);
        return 0u8;
    }

    if ((*NtHeaders).FileHeader.Characteristics & IMAGE_FILE_EXECUTABLE_IMAGE) == 0 {
        ERR!("Not an executable image \"%s\"\n", FilePath);
        ArcClose(FileId);
        return 0u8;
    }

    NumberOfSections = (*NtHeaders).FileHeader.NumberOfSections as ULONG;
    SectionHeader = IMAGE_FIRST_SECTION(NtHeaders);

    crate::frldr::dbgcon_str(b"\nLI[ib=");
    crate::frldr::dbgcon_hex32((*NtHeaders).OptionalHeader.ImageBase as u32);
    crate::frldr::dbgcon_str(b",sz=");
    crate::frldr::dbgcon_hex32((*NtHeaders).OptionalHeader.SizeOfImage);
    PhysicalBase = MmAllocateMemoryAtAddress(
        (*NtHeaders).OptionalHeader.SizeOfImage as usize,
        ((*NtHeaders).OptionalHeader.ImageBase as usize & crate::ffi::KSEG0_BASE.wrapping_sub(1)) as PVOID,
        MemoryType,
    );
    crate::frldr::dbgcon_str(b",pa1=");
    crate::frldr::dbgcon_hex32(PhysicalBase as usize as u32);

    if PhysicalBase.is_null() {
        PhysicalBase = MmAllocateMemoryWithType((*NtHeaders).OptionalHeader.SizeOfImage as usize, MemoryType);
        crate::frldr::dbgcon_str(b",pa2=");
        crate::frldr::dbgcon_hex32(PhysicalBase as usize as u32);

        if PhysicalBase.is_null() {
            crate::frldr::dbgcon_str(b",FAIL]");
            ERR!("Failed to alloc %lu bytes for image %s\n", (*NtHeaders).OptionalHeader.SizeOfImage, FilePath);
            ArcClose(FileId);
            return 0u8;
        }
    }
    crate::frldr::dbgcon_str(b"]\n");

    VirtualBase = if KernelMapping != 0 {
        PaToVa(PhysicalBase)
    } else {
        PhysicalBase
    };

    TRACE!("Base PA: 0x%p, VA: 0x%p\n", PhysicalBase, VirtualBase);

    RtlCopyMemory(
        PhysicalBase,
        HeadersBuffer.as_ptr() as *const core::ffi::c_void,
        min((*NtHeaders).OptionalHeader.SizeOfHeaders, core::mem::size_of_val(&HeadersBuffer) as ULONG) as usize,
    );

    if (*NtHeaders).OptionalHeader.SizeOfHeaders > core::mem::size_of_val(&HeadersBuffer) as ULONG {
        Status = ArcRead(
            FileId,
            (PhysicalBase as PUCHAR).add(core::mem::size_of_val(&HeadersBuffer)) as *mut core::ffi::c_void,
            (*NtHeaders).OptionalHeader.SizeOfHeaders - core::mem::size_of_val(&HeadersBuffer) as ULONG,
            &mut BytesRead,
        );
        if Status != ESUCCESS {
            ERR!("ArcRead('%s') failed. Status: %u\n", FilePath, Status);
            ArcClose(FileId);
            MmFreeMemory(PhysicalBase);
            return 0u8;
        }
    }

    NtHeaders = RtlImageNtHeader(PhysicalBase);
    SectionHeader = IMAGE_FIRST_SECTION(NtHeaders);

    i = 0;
    while i < NumberOfSections {
        VirtualSize = (*SectionHeader).Misc.VirtualSize;
        SizeOfRawData = (*SectionHeader).SizeOfRawData;

        if VirtualSize == 0 {
            VirtualSize = SizeOfRawData;
        }

        if (*SectionHeader).PointerToRawData == 0 {
            SizeOfRawData = 0;
        } else {
            if SizeOfRawData > VirtualSize {
                SizeOfRawData = VirtualSize;
            }
        }

        if SizeOfRawData != 0 {
            Position.QuadPart = (*SectionHeader).PointerToRawData as i64;
            Status = ArcSeek(FileId, &mut Position as *mut _ as *mut i64, SeekAbsolute);

            TRACE!("SH->VA: 0x%X\n", (*SectionHeader).VirtualAddress);

            Status = ArcRead(
                FileId,
                (PhysicalBase as PUCHAR).add((*SectionHeader).VirtualAddress as usize) as *mut core::ffi::c_void,
                SizeOfRawData,
                &mut BytesRead,
            );
            if Status != ESUCCESS {
                ERR!("PeLdrLoadImage(): Error reading section from file!\n");
                break;
            }
        }

        if SizeOfRawData < VirtualSize {
            TRACE!("PeLdrLoadImage(): SORD %d < VS %d\n", SizeOfRawData, VirtualSize);
            RtlZeroMemory(
                ((*SectionHeader).VirtualAddress as ULONG_PTR + PhysicalBase as ULONG_PTR + SizeOfRawData as ULONG_PTR) as PVOID,
                (VirtualSize - SizeOfRawData) as usize,
            );
        }

        SectionHeader = SectionHeader.add(1);
        i += 1;
    }

    ArcClose(FileId);

    if Status != ESUCCESS {
        MmFreeMemory(PhysicalBase);
        return 0u8;
    }

    if (*NtHeaders).OptionalHeader.ImageBase as ULONG_PTR != VirtualBase as ULONG_PTR {
        WARN!("Relocating %p -> %p\n", (*NtHeaders).OptionalHeader.ImageBase as PVOID, VirtualBase);
        Status = LdrRelocateImageWithBias(
            PhysicalBase,
            (VirtualBase as ULONG_PTR).wrapping_sub(PhysicalBase as ULONG_PTR) as i64,
            b"FreeLdr\0".as_ptr() as PSTR,
            ESUCCESS,
            ESUCCESS,
            ENOEXEC,
        );
        if Status != ESUCCESS {
            MmFreeMemory(PhysicalBase);
            return 0u8;
        }
    }

    *ImageBasePA = PhysicalBase;

    TRACE!("PeLdrLoadImage() done, PA = %p\n", *ImageBasePA);
    1u8
}

#[no_mangle]
pub unsafe extern "C" fn PeLdrLoadImage(
    FilePath: PCSTR,
    MemoryType: TYPE_OF_MEMORY,
    ImageBasePA: *mut PVOID,
) -> BOOLEAN {
    PeLdrLoadImageEx(FilePath, MemoryType, ImageBasePA, 1u8)
}

#[no_mangle]
pub unsafe extern "C" fn PeLdrLoadBootImage(
    FilePath: PCSTR,
    BaseDllName: PCSTR,
    ImageBase: *mut PVOID,
    DataTableEntry: *mut PLDR_DATA_TABLE_ENTRY,
) -> BOOLEAN {
    crate::frldr::dbgcon(b'L'); crate::frldr::dbgcon(b'B'); crate::frldr::dbgcon(b'1');

    let mut Success: BOOLEAN;

    Success = PeLdrLoadImageEx(
        FilePath,
        LoaderLoadedProgram,
        ImageBase,
        0u8,
    );
    crate::frldr::dbgcon(b'L'); crate::frldr::dbgcon(b'B'); crate::frldr::dbgcon(b'2');
    if Success == 0 {
        crate::frldr::dbgcon(b'L'); crate::frldr::dbgcon(b'B'); crate::frldr::dbgcon(b'F');
        WARN!("Failed to load boot image '%s'\n", FilePath);
        return 0u8;
    }

    Success = PeLdrAllocateDataTableEntry(
        core::ptr::addr_of_mut!(FrLdrModuleList),
        BaseDllName,
        FilePath,
        *ImageBase,
        DataTableEntry,
    );
    crate::frldr::dbgcon(b'L'); crate::frldr::dbgcon(b'B'); crate::frldr::dbgcon(b'3');
    if Success == 0 {
        ERR!("Failed to allocate DTE for '%s'\n", FilePath);
        MmFreeMemory(*ImageBase);
        return 0u8;
    }

    Success = PeLdrScanImportDescriptorTable(
        core::ptr::addr_of_mut!(FrLdrModuleList),
        b"\0".as_ptr() as PCCH,
        *DataTableEntry,
    );
    crate::frldr::dbgcon(b'L'); crate::frldr::dbgcon(b'B'); crate::frldr::dbgcon(b'4');
    if Success == 0 {
        ERR!("Failed to resolve imports for '%s'\n", FilePath);
        PeLdrFreeDataTableEntry(*DataTableEntry);
        MmFreeMemory(*ImageBase);
        return 0u8;
    }

    1u8
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     FreeLoader
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     Provides routines for loading PE files.
//  *              (Deprecated remark) To be merged with arch/i386/loader.c in future.
//  *
//  * COPYRIGHT:   Copyright 1998-2003 Brian Palmer <brianp@sginet.com>
//  *              Copyright 2006-2019 Aleksey Bragin <aleksey@reactos.org>
//  *
//  * NOTES:       The source code in this file is based on the work of respective
//  *              authors of PE loading code in ReactOS and Brian Palmer and
//  *              Alex Ionescu's arch/i386/loader.c, and my research project
//  *              (creating a native EFI loader for Windows).
//  *
//  *              This article was very handy during development:
//  *              https://web.archive.org/web/20131202091645/http://msdn.microsoft.com/en-us/magazine/cc301808.aspx
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <freeldr.h>
//
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(PELOADER);
//
// /* GLOBALS *******************************************************************/
//
// LIST_ENTRY FrLdrModuleList;
//
// PELDR_IMPORTDLL_LOAD_CALLBACK PeLdrImportDllLoadCallback = NULL;
//
// #ifdef _WIN64
// #define COOKIE_MAX 0x0000FFFFFFFFFFFFll
// #define DEFAULT_SECURITY_COOKIE 0x00002B992DDFA232ll
// #else
// #define DEFAULT_SECURITY_COOKIE 0xBB40E64E
// #endif
//
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// static PVOID
// PeLdrpFetchAddressOfSecurityCookie(PVOID BaseAddress, ULONG SizeOfImage)
// {
//     PIMAGE_LOAD_CONFIG_DIRECTORY ConfigDir;
//     ULONG DirSize;
//     PULONG_PTR Cookie = NULL;
//
//     /* Get the pointer to the config directory */
//     ConfigDir = RtlImageDirectoryEntryToData(BaseAddress,
//                                              TRUE,
//                                              IMAGE_DIRECTORY_ENTRY_LOAD_CONFIG,
//                                              &DirSize);
//
//     /* Check for sanity */
//     if (!ConfigDir ||
//         DirSize < RTL_SIZEOF_THROUGH_FIELD(IMAGE_LOAD_CONFIG_DIRECTORY, SecurityCookie))
//     {
//         /* Invalid directory*/
//         return NULL;
//     }
//
//     /* Now get the cookie */
//     Cookie = VaToPa((PULONG_PTR)ConfigDir->SecurityCookie);
//
//     /* Check this cookie */
//     if ((PCHAR)Cookie <= (PCHAR)BaseAddress ||
//         (PCHAR)Cookie >= (PCHAR)BaseAddress + SizeOfImage - sizeof(*Cookie))
//     {
//         Cookie = NULL;
//     }
//
//     /* Return validated security cookie */
//     return Cookie;
// }
//
// /* DllName: physical, UnicodeString->Buffer: virtual */
// static BOOLEAN
// PeLdrpCompareDllName(
//     _In_ PCSTR DllName,
//     _In_ PCUNICODE_STRING UnicodeName)
// {
//     PWSTR Buffer;
//     SIZE_T i, Length;
//
//     /* First obvious check: for length of two names */
//     Length = strlen(DllName);
//
// #if DBG
//     {
//         UNICODE_STRING UnicodeNamePA;
//         UnicodeNamePA.Length = UnicodeName->Length;
//         UnicodeNamePA.MaximumLength = UnicodeName->MaximumLength;
//         UnicodeNamePA.Buffer = VaToPa(UnicodeName->Buffer);
//         TRACE("PeLdrpCompareDllName: %s and %wZ, Length = %d, UN->Length %d\n",
//               DllName, &UnicodeNamePA, Length, UnicodeName->Length);
//     }
// #endif
//
//     if ((Length * sizeof(WCHAR)) > UnicodeName->Length)
//         return FALSE;
//
//     /* Store pointer to unicode string's buffer */
//     Buffer = VaToPa(UnicodeName->Buffer);
//
//     /* Loop character by character */
//     for (i = 0; i < Length; i++)
//     {
//         /* Compare two characters, uppercasing them */
//         if (toupper(*DllName) != toupper((CHAR)*Buffer))
//             return FALSE;
//
//         /* Move to the next character */
//         DllName++;
//         Buffer++;
//     }
//
//     /* Check, if strings either fully match, or match till the "." (w/o extension) */
//     if ((UnicodeName->Length == Length * sizeof(WCHAR)) || (*Buffer == L'.'))
//     {
//         /* Yes they do */
//         return TRUE;
//     }
//
//     /* Strings don't match */
//     return FALSE;
// }
//
// static BOOLEAN
// PeLdrpLoadAndScanReferencedDll(
//     IN OUT PLIST_ENTRY ModuleListHead,
//     IN PCCH DirectoryPath,
//     IN PCH ImportName,
//     IN PLIST_ENTRY Parent OPTIONAL,
//     OUT PLDR_DATA_TABLE_ENTRY *DataTableEntry);
//
// static BOOLEAN
// PeLdrpBindImportName(
//     _Inout_ PLIST_ENTRY ModuleListHead,
//     _In_ PVOID DllBase,
//     _In_ PVOID ImageBase,
//     _In_ PIMAGE_THUNK_DATA ThunkName,
//     _Inout_ PIMAGE_THUNK_DATA ThunkData,
//     _In_ PIMAGE_EXPORT_DIRECTORY ExportDirectory,
//     _In_ ULONG ExportSize,
//     _In_ BOOLEAN ProcessForwards,
//     _In_ PCSTR DirectoryPath,
//     _In_ PLIST_ENTRY Parent)
// {
//     ULONG Ordinal;
//     PULONG NameTable, FunctionTable;
//     PUSHORT OrdinalTable;
//     LONG High, Low, Middle, Result;
//     ULONG Hint;
//     PIMAGE_IMPORT_BY_NAME ImportData;
//     PCHAR ExportName, ForwarderName;
//     BOOLEAN Success;
//
//     //TRACE("PeLdrpBindImportName(): "
//     //      "DllBase 0x%p, ImageBase 0x%p, ThunkName 0x%p, ThunkData 0x%p, ExportDirectory 0x%p, ExportSize %d, ProcessForwards 0x%X\n",
//     //      DllBase, ImageBase, ThunkName, ThunkData, ExportDirectory, ExportSize, ProcessForwards);
//
//     /* Check passed DllBase */
//     if (!DllBase)
//     {
//         WARN("DllBase == NULL\n");
//         return FALSE;
//     }
//
//     /* Convert all non-critical pointers to PA from VA */
//     ThunkName = VaToPa(ThunkName);
//     ThunkData = VaToPa(ThunkData);
//
//     /* Is the reference by ordinal? */
//     if (IMAGE_SNAP_BY_ORDINAL(ThunkName->u1.Ordinal) && !ProcessForwards)
//     {
//         /* Yes, calculate the ordinal */
//         Ordinal = (ULONG)(IMAGE_ORDINAL(ThunkName->u1.Ordinal) - (UINT32)ExportDirectory->Base);
//         //TRACE("PeLdrpBindImportName(): Ordinal %d\n", Ordinal);
//     }
//     else
//     {
//         ULONG_PTR ImportDataVa;
//
//         /* It's reference by name, we have to look it up in the export directory */
//         if (!ProcessForwards)
//         {
//             /* AddressOfData in thunk entry is an RVA — convert to VA via a local
//              * so we don't mutate OriginalFirstThunk.  When lld emits duplicate
//              * PE import descriptors sharing the same OFT/IAT (a known artifact
//              * of Rust raw-dylib with multiple CGUs in hal-rust), a second pass
//              * over the OFT must still see the original RVAs; mutating in place
//              * would leave a kernel-mode VA whose high bit makes the next pass
//              * misinterpret the slot as IMAGE_SNAP_BY_ORDINAL with a bogus
//              * ordinal, triggering "Ordinal number is invalid!". */
//             ImportDataVa = (ULONG_PTR)RVA(ImageBase, ThunkName->u1.AddressOfData);
//         }
//         else
//         {
//             /* Forwarder case: ThunkName already holds a VA. */
//             ImportDataVa = ThunkName->u1.AddressOfData;
//         }
//
//         /* Get the import name, convert it to a physical pointer */
//         ImportData = VaToPa((PVOID)ImportDataVa);
//
//         /* Get pointers to Name and Ordinal tables (RVA -> VA) */
//         NameTable = VaToPa(RVA(DllBase, ExportDirectory->AddressOfNames));
//         OrdinalTable = VaToPa(RVA(DllBase, ExportDirectory->AddressOfNameOrdinals));
//
//         //TRACE("NameTable 0x%X, OrdinalTable 0x%X, ED->AddressOfNames 0x%X, ED->AOFO 0x%X\n",
//         //      NameTable, OrdinalTable, ExportDirectory->AddressOfNames, ExportDirectory->AddressOfNameOrdinals);
//
//         /* Get the hint */
//         Hint = ImportData->Hint;
//         //TRACE("HintIndex %d\n", Hint);
//
//         /* Get the export name from the hint */
//         ExportName = VaToPa(RVA(DllBase, NameTable[Hint]));
//
//         /* If Hint is less than total number of entries in the export directory,
//            and import name == export name, then we can just get it from the OrdinalTable */
//         if ((Hint < ExportDirectory->NumberOfNames) &&
//             (strcmp(ExportName, (PCHAR)ImportData->Name) == 0))
//         {
//             Ordinal = OrdinalTable[Hint];
//             //TRACE("PeLdrpBindImportName(): Ordinal %d\n", Ordinal);
//         }
//         else
//         {
//             /* It's not the easy way, we have to lookup import name in the name table.
//                Let's use a binary search for this task. */
//
//             //TRACE("PeLdrpBindImportName() looking up the import name using binary search...\n");
//
//             /* Low boundary is set to 0, and high boundary to the maximum index */
//             Low = 0;
//             High = ExportDirectory->NumberOfNames - 1;
//
//             /* Perform a binary-search loop */
//             while (High >= Low)
//             {
//                 /* Divide by 2 by shifting to the right once */
//                 Middle = (Low + High) / 2;
//
//                 /* Get the name from the name table */
//                 ExportName = VaToPa(RVA(DllBase, NameTable[Middle]));
//
//                 /* Compare the names */
//                 Result = strcmp(ExportName, (PCHAR)ImportData->Name);
//
//                 // TRACE("Binary search: comparing Import '%s', Export '%s'\n",
//                       // (PCHAR)ImportData->Name, ExportName);
//
//                 /* Depending on result of strcmp, perform different actions */
//                 if (Result > 0)
//                 {
//                     /* Adjust top boundary */
//                     High = Middle - 1;
//                 }
//                 else if (Result < 0)
//                 {
//                     /* Adjust bottom boundary */
//                     Low = Middle + 1;
//                 }
//                 else
//                 {
//                     /* Yay, found it! */
//                     break;
//                 }
//             }
//
//             /* If high boundary is less than low boundary, then no result found */
//             if (High < Low)
//             {
//                 ERR("Did not find export '%s'!\n", (PCHAR)ImportData->Name);
//                 return FALSE;
//             }
//
//             /* Everything alright, get the ordinal */
//             Ordinal = OrdinalTable[Middle];
//             //TRACE("PeLdrpBindImportName() found Ordinal %d\n", Ordinal);
//         }
//     }
//
//     /* Check ordinal number for validity! */
//     if (Ordinal >= ExportDirectory->NumberOfFunctions)
//     {
//         ERR("Ordinal number is invalid!\n");
//         return FALSE;
//     }
//
//     /* Get a pointer to the function table */
//     FunctionTable = (PULONG)VaToPa(RVA(DllBase, ExportDirectory->AddressOfFunctions));
//
//     /* Save a pointer to the function */
//     ThunkData->u1.Function = (ULONG_PTR)RVA(DllBase, FunctionTable[Ordinal]);
//
//     /* Is it a forwarder? (function pointer is within the export directory) */
//     ForwarderName = (PCHAR)VaToPa((PVOID)ThunkData->u1.Function);
//     if (((ULONG_PTR)ForwarderName > (ULONG_PTR)ExportDirectory) &&
//         ((ULONG_PTR)ForwarderName < ((ULONG_PTR)ExportDirectory + ExportSize)))
//     {
//         PLDR_DATA_TABLE_ENTRY DataTableEntry;
//         PIMAGE_EXPORT_DIRECTORY RefExportDirectory;
//         ULONG RefExportSize;
//         CHAR ForwardDllName[256];
//
//         TRACE("PeLdrpBindImportName(): ForwarderName %s\n", ForwarderName);
//
//         /* Save the name of the forward dll */
//         RtlCopyMemory(ForwardDllName, ForwarderName, sizeof(ForwardDllName));
//
//         /* Strip out the symbol name */
//         *strrchr(ForwardDllName, '.') = ANSI_NULL;
//
//         /* Strip leading underscore from DLL name. LLD adds i386 symbol
//          * decoration to PE forwarder targets, turning "NTOSKRNL.func"
//          * into "_NTOSKRNL.func" in the export table. */
//         if (ForwardDllName[0] == '_')
//         {
//             ULONG i;
//             for (i = 0; ForwardDllName[i]; i++)
//                 ForwardDllName[i] = ForwardDllName[i + 1];
//         }
//
//         /* Check if the target image is already loaded */
//         if (!PeLdrCheckForLoadedDll(ModuleListHead, ForwardDllName, &DataTableEntry))
//         {
//             /* Check if the forward dll name has an extension */
//             if (strchr(ForwardDllName, '.') == NULL)
//             {
//                 /* Name does not have an extension, append '.dll' */
//                 RtlStringCbCatA(ForwardDllName, sizeof(ForwardDllName), ".dll");
//             }
//
//             /* Now let's try to load it! */
//             Success = PeLdrpLoadAndScanReferencedDll(ModuleListHead,
//                                                      DirectoryPath,
//                                                      ForwardDllName,
//                                                      Parent,
//                                                      &DataTableEntry);
//             if (!Success)
//             {
//                 ERR("PeLdrpLoadAndScanReferencedDll() failed to load forwarder dll.\n");
//                 return Success;
//             }
//         }
//
//         /* Get pointer to the export directory of loaded DLL */
//         RefExportDirectory = (PIMAGE_EXPORT_DIRECTORY)
//             RtlImageDirectoryEntryToData(VaToPa(DataTableEntry->DllBase),
//             TRUE,
//             IMAGE_DIRECTORY_ENTRY_EXPORT,
//             &RefExportSize);
//
//         /* Fail if it's NULL */
//         if (RefExportDirectory)
//         {
//             UCHAR Buffer[128];
//             IMAGE_THUNK_DATA RefThunkData;
//             PIMAGE_IMPORT_BY_NAME ImportByName;
//             PCHAR ImportName;
//
//             /* Get pointer to the import name */
//             ImportName = strrchr(ForwarderName, '.') + 1;
//
//             /* Create a IMAGE_IMPORT_BY_NAME structure, pointing to the local Buffer */
//             ImportByName = (PIMAGE_IMPORT_BY_NAME)Buffer;
//
//             /* Fill the name with the import name */
//             RtlCopyMemory(ImportByName->Name, ImportName, strlen(ImportName)+1);
//
//             /* Set Hint to 0 */
//             ImportByName->Hint = 0;
//
//             /* And finally point ThunkData's AddressOfData to that structure */
//             RefThunkData.u1.AddressOfData = (ULONG_PTR)ImportByName;
//
//             /* And recursively call ourselves */
//             Success = PeLdrpBindImportName(ModuleListHead,
//                                            DataTableEntry->DllBase,
//                                            ImageBase,
//                                            &RefThunkData,
//                                            &RefThunkData,
//                                            RefExportDirectory,
//                                            RefExportSize,
//                                            TRUE,
//                                            DirectoryPath,
//                                            Parent);
//
//             /* Fill out the ThunkData with data from RefThunkData */
//             ThunkData->u1 = RefThunkData.u1;
//
//             /* Return what we got from the recursive call */
//             return Success;
//         }
//         else
//         {
//             /* Fail if ExportDirectory is NULL */
//             return FALSE;
//         }
//     }
//
//     /* Success! */
//     return TRUE;
// }
//
// static BOOLEAN
// PeLdrpLoadAndScanReferencedDll(
//     IN OUT PLIST_ENTRY ModuleListHead,
//     IN PCCH DirectoryPath,
//     IN PCH ImportName,
//     IN PLIST_ENTRY Parent OPTIONAL,
//     OUT PLDR_DATA_TABLE_ENTRY *DataTableEntry)
// {
//     CHAR FullDllName[256];
//     BOOLEAN Success;
//     PVOID BasePA = NULL;
//
//     /* Prepare the full path to the file to be loaded */
//     RtlStringCbCopyA(FullDllName, sizeof(FullDllName), DirectoryPath);
//     RtlStringCbCatA(FullDllName, sizeof(FullDllName), ImportName);
//
//     TRACE("Loading referenced DLL: %s\n", FullDllName);
//
//     if (PeLdrImportDllLoadCallback)
//         PeLdrImportDllLoadCallback(FullDllName);
//
//     /* Load the image */
//     Success = PeLdrLoadImage(FullDllName, LoaderBootDriver, &BasePA);
//     if (!Success)
//     {
//         ERR("PeLdrLoadImage('%s') failed\n", FullDllName);
//         return Success;
//     }
//
//     /* Allocate DTE for newly loaded DLL */
//     Success = PeLdrAllocateDataTableEntry(Parent ? Parent->Blink : ModuleListHead,
//                                           ImportName,
//                                           FullDllName,
//                                           PaToVa(BasePA),
//                                           DataTableEntry);
//     if (!Success)
//     {
//         /* Cleanup and bail out */
//         ERR("PeLdrAllocateDataTableEntry('%s') failed\n", FullDllName);
//         MmFreeMemory(BasePA);
//         return Success;
//     }
//
//     /* Init security cookie */
//     PeLdrInitSecurityCookie(*DataTableEntry);
//
//     (*DataTableEntry)->Flags |= LDRP_DRIVER_DEPENDENT_DLL;
//
//     /* Scan its dependencies too */
//     TRACE("PeLdrScanImportDescriptorTable() calling ourselves for '%.*S'\n",
//           (*DataTableEntry)->BaseDllName.Length / sizeof(WCHAR),
//           VaToPa((*DataTableEntry)->BaseDllName.Buffer));
//     Success = PeLdrScanImportDescriptorTable(ModuleListHead, DirectoryPath, *DataTableEntry);
//     if (!Success)
//     {
//         /* Cleanup and bail out */
//         ERR("PeLdrScanImportDescriptorTable() failed\n");
//         PeLdrFreeDataTableEntry(*DataTableEntry);
//         MmFreeMemory(BasePA);
//         return Success;
//     }
//
//     return TRUE;
// }
//
// static BOOLEAN
// PeLdrpScanImportAddressTable(
//     _Inout_ PLIST_ENTRY ModuleListHead,
//     _In_ PVOID DllBase,
//     _In_ PVOID ImageBase,
//     _In_ PIMAGE_THUNK_DATA ThunkName,
//     _Inout_ PIMAGE_THUNK_DATA ThunkData,
//     _In_ PCSTR DirectoryPath,
//     _In_ PLIST_ENTRY Parent)
// {
//     PIMAGE_EXPORT_DIRECTORY ExportDirectory = NULL;
//     BOOLEAN Success;
//     ULONG ExportSize;
//
//     TRACE("PeLdrpScanImportAddressTable(): "
//           "DllBase 0x%p, ImageBase 0x%p, ThunkName 0x%p, ThunkData 0x%p\n",
//           DllBase, ImageBase, ThunkName, ThunkData);
//
//     /* Obtain the export table from the DLL's base */
//     if (!DllBase)
//     {
//         ERR("DllBase == NULL\n");
//         return FALSE;
//     }
//     else
//     {
//         ExportDirectory =
//             (PIMAGE_EXPORT_DIRECTORY)RtlImageDirectoryEntryToData(VaToPa(DllBase),
//                 TRUE,
//                 IMAGE_DIRECTORY_ENTRY_EXPORT,
//                 &ExportSize);
//     }
//     TRACE("PeLdrpScanImportAddressTable(): ExportDirectory 0x%p\n", ExportDirectory);
//
//     /* Fail if no export directory */
//     if (!ExportDirectory)
//     {
//         ERR("No ExportDir, DllBase = %p (%p)\n", DllBase, VaToPa(DllBase));
//         return FALSE;
//     }
//
//     /* Go through each thunk in the table and bind it */
//     while (((PIMAGE_THUNK_DATA)VaToPa(ThunkName))->u1.AddressOfData != 0)
//     {
//         /* Bind it */
//         Success = PeLdrpBindImportName(ModuleListHead,
//                                        DllBase,
//                                        ImageBase,
//                                        ThunkName,
//                                        ThunkData,
//                                        ExportDirectory,
//                                        ExportSize,
//                                        FALSE,
//                                        DirectoryPath,
//                                        Parent);
//         /* Fail if binding was unsuccessful */
//         if (!Success)
//             return Success;
//
//         /* Move to the next thunk */
//         ThunkName++;
//         ThunkData++;
//     }
//
//     /* Return success */
//     return TRUE;
// }
//
//
// /* FUNCTIONS *****************************************************************/
//
// BOOLEAN
// PeLdrInitializeModuleList(VOID)
// {
//     PLDR_DATA_TABLE_ENTRY FreeldrDTE;
//
//     InitializeListHead(&FrLdrModuleList);
//
//     /* Allocate a data table entry for freeldr.sys */
//     if (!PeLdrAllocateDataTableEntry(&FrLdrModuleList,
//                                      "freeldr.sys",
//                                      "freeldr.sys",
//                                      &__ImageBase,
//                                      &FreeldrDTE))
//     {
//         /* Cleanup and bail out */
//         ERR("Failed to allocate DTE for freeldr\n");
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// PVOID
// PeLdrInitSecurityCookie(PLDR_DATA_TABLE_ENTRY LdrEntry)
// {
//     PULONG_PTR Cookie;
//     ULONG_PTR NewCookie;
//
//     /* Fetch address of the cookie */
//     Cookie = PeLdrpFetchAddressOfSecurityCookie(VaToPa(LdrEntry->DllBase), LdrEntry->SizeOfImage);
//
//     if (!Cookie)
//         return NULL;
//
//     /* Check if it's a default one */
//     if ((*Cookie == DEFAULT_SECURITY_COOKIE) ||
//         (*Cookie == 0))
//     {
//         /* Generate new cookie using cookie address and time as seed */
//         NewCookie = (ULONG_PTR)Cookie ^ (ULONG_PTR)ArcGetRelativeTime();
// #ifdef _WIN64
//         /* Some images expect first 16 bits to be kept clean (like in default cookie) */
//         if (NewCookie > COOKIE_MAX)
//         {
//             NewCookie >>= 16;
//         }
// #endif
//         /* If the result is 0 or the same as we got, just add one to the default value */
//         if ((NewCookie == 0) || (NewCookie == *Cookie))
//         {
//             NewCookie = DEFAULT_SECURITY_COOKIE + 1;
//         }
//
//         /* Set the new cookie value */
//         *Cookie = NewCookie;
//     }
//
//     return Cookie;
// }
//
// /* Returns TRUE if the DLL has already been loaded in the module list */
// BOOLEAN
// PeLdrCheckForLoadedDll(
//     _Inout_ PLIST_ENTRY ModuleListHead,
//     _In_ PCSTR DllName,
//     _Out_ PLDR_DATA_TABLE_ENTRY* LoadedEntry)
// {
//     PLIST_ENTRY ModuleEntry;
//     PLDR_DATA_TABLE_ENTRY DataTableEntry;
//
//     TRACE("PeLdrCheckForLoadedDll: DllName %s\n", DllName);
//
//     /* Go through each entry in the LoadOrderList and
//      * compare the module's name with the given name */
//     for (ModuleEntry = ModuleListHead->Flink;
//          ModuleEntry != ModuleListHead;
//          ModuleEntry = ModuleEntry->Flink)
//     {
//         /* Get a pointer to the current DTE */
//         DataTableEntry = CONTAINING_RECORD(ModuleEntry,
//                                            LDR_DATA_TABLE_ENTRY,
//                                            InLoadOrderLinks);
//
//         TRACE("PeLdrCheckForLoadedDll: DTE %p, EP %p, Base %p, Name '%.*S'\n",
//               DataTableEntry, DataTableEntry->EntryPoint, DataTableEntry->DllBase,
//               DataTableEntry->BaseDllName.Length / sizeof(WCHAR),
//               VaToPa(DataTableEntry->BaseDllName.Buffer));
//
//         /* Compare names */
//         if (PeLdrpCompareDllName(DllName, &DataTableEntry->BaseDllName))
//         {
//             /* Found it, return a pointer to the loaded module's
//              * DTE to the caller and increase its load count */
//             *LoadedEntry = DataTableEntry;
//             DataTableEntry->LoadCount++;
//             TRACE("PeLdrCheckForLoadedDll: LoadedEntry 0x%p\n", DataTableEntry);
//             return TRUE;
//         }
//     }
//
//     /* Nothing found */
//     return FALSE;
// }
//
// BOOLEAN
// PeLdrScanImportDescriptorTable(
//     IN OUT PLIST_ENTRY ModuleListHead,
//     IN PCCH DirectoryPath,
//     IN PLDR_DATA_TABLE_ENTRY ScanDTE)
// {
//     PLDR_DATA_TABLE_ENTRY DataTableEntry;
//     PIMAGE_IMPORT_DESCRIPTOR ImportTable;
//     ULONG ImportTableSize;
//     PCH ImportName;
//     BOOLEAN Success;
//
//     /* Get a pointer to the import table of this image */
//     ImportTable = (PIMAGE_IMPORT_DESCRIPTOR)RtlImageDirectoryEntryToData(VaToPa(ScanDTE->DllBase),
//         TRUE, IMAGE_DIRECTORY_ENTRY_IMPORT, &ImportTableSize);
//
// #if DBG
//     {
//         UNICODE_STRING BaseName;
//         BaseName.Buffer = VaToPa(ScanDTE->BaseDllName.Buffer);
//         BaseName.MaximumLength = ScanDTE->BaseDllName.MaximumLength;
//         BaseName.Length = ScanDTE->BaseDllName.Length;
//         TRACE("PeLdrScanImportDescriptorTable(): %wZ ImportTable = 0x%p\n",
//               &BaseName, ImportTable);
//     }
// #endif
//
//     /* If the image doesn't have any import directory, just return success */
//     if (!ImportTable)
//         return TRUE;
//
//     /* Loop through all the entries */
//     for (;(ImportTable->Name != 0) && (ImportTable->OriginalFirstThunk != 0);ImportTable++)
//     {
//         PIMAGE_THUNK_DATA ThunkName = RVA(ScanDTE->DllBase, ImportTable->OriginalFirstThunk);
//         PIMAGE_THUNK_DATA ThunkData = RVA(ScanDTE->DllBase, ImportTable->FirstThunk);
//
//         /* Get pointer to the name */
//         ImportName = (PCH)VaToPa(RVA(ScanDTE->DllBase, ImportTable->Name));
//         TRACE("PeLdrScanImportDescriptorTable(): Looking at %s\n", ImportName);
//
//         /* In case we get a reference to ourselves - just skip it */
//         if (PeLdrpCompareDllName(ImportName, &ScanDTE->BaseDllName))
//             continue;
//
//         /* Load the DLL if it is not already loaded */
//         if (!PeLdrCheckForLoadedDll(ModuleListHead, ImportName, &DataTableEntry))
//         {
//             Success = PeLdrpLoadAndScanReferencedDll(ModuleListHead,
//                                                      DirectoryPath,
//                                                      ImportName,
//                                                      &ScanDTE->InLoadOrderLinks,
//                                                      &DataTableEntry);
//             if (!Success)
//             {
//                 ERR("PeLdrpLoadAndScanReferencedDll() failed\n");
//                 return Success;
//             }
//         }
//
//         /* Scan its import address table */
//         Success = PeLdrpScanImportAddressTable(ModuleListHead,
//                                                DataTableEntry->DllBase,
//                                                ScanDTE->DllBase,
//                                                ThunkName,
//                                                ThunkData,
//                                                DirectoryPath,
//                                                &ScanDTE->InLoadOrderLinks);
//
//         if (!Success)
//         {
//             ERR("PeLdrpScanImportAddressTable() failed: ImportName = '%s', DirectoryPath = '%s'\n",
//                 ImportName, DirectoryPath);
//             return Success;
//         }
//     }
//
//     return TRUE;
// }
//
// BOOLEAN
// PeLdrAllocateDataTableEntry(
//     IN OUT PLIST_ENTRY ModuleListHead,
//     IN PCCH BaseDllName,
//     IN PCCH FullDllName,
//     IN PVOID BaseVA,
//     OUT PLDR_DATA_TABLE_ENTRY *NewEntry)
// {
//     PVOID BasePA = VaToPa(BaseVA);
//     PWSTR BaseDllNameBuffer, Buffer;
//     PLDR_DATA_TABLE_ENTRY DataTableEntry;
//     PIMAGE_NT_HEADERS NtHeaders;
//     USHORT Length;
//
//     TRACE("PeLdrAllocateDataTableEntry('%s', '%s', %p)\n",
//           BaseDllName, FullDllName, BasePA);
//
//     /* Allocate memory for a data table entry, zero-initialize it */
//     DataTableEntry = (PLDR_DATA_TABLE_ENTRY)FrLdrHeapAlloc(sizeof(LDR_DATA_TABLE_ENTRY),
//                                                            TAG_WLDR_DTE);
//     if (DataTableEntry == NULL)
//         return FALSE;
//
//     /* Get NT headers from the image */
//     NtHeaders = RtlImageNtHeader(BasePA);
//
//     /* Initialize corresponding fields of DTE based on NT headers value */
//     RtlZeroMemory(DataTableEntry, sizeof(LDR_DATA_TABLE_ENTRY));
//     DataTableEntry->DllBase = BaseVA;
//     DataTableEntry->SizeOfImage = NtHeaders->OptionalHeader.SizeOfImage;
//     DataTableEntry->EntryPoint = RVA(BaseVA, NtHeaders->OptionalHeader.AddressOfEntryPoint);
//     DataTableEntry->SectionPointer = 0;
//     DataTableEntry->CheckSum = NtHeaders->OptionalHeader.CheckSum;
//
//     /* Initialize BaseDllName field (UNICODE_STRING) from the Ansi BaseDllName
//        by simple conversion - copying each character */
//     Length = (USHORT)(strlen(BaseDllName) * sizeof(WCHAR));
//     Buffer = (PWSTR)FrLdrHeapAlloc(Length, TAG_WLDR_NAME);
//     if (Buffer == NULL)
//     {
//         FrLdrHeapFree(DataTableEntry, TAG_WLDR_DTE);
//         return FALSE;
//     }
//
//     /* Save Buffer, in case of later failure */
//     BaseDllNameBuffer = Buffer;
//
//     DataTableEntry->BaseDllName.Length = Length;
//     DataTableEntry->BaseDllName.MaximumLength = Length;
//     DataTableEntry->BaseDllName.Buffer = PaToVa(Buffer);
//
//     RtlZeroMemory(Buffer, Length);
//     Length /= sizeof(WCHAR);
//     while (Length--)
//     {
//         *Buffer++ = *BaseDllName++;
//     }
//
//     /* Initialize FullDllName field (UNICODE_STRING) from the Ansi FullDllName
//        using the same method */
//     Length = (USHORT)(strlen(FullDllName) * sizeof(WCHAR));
//     Buffer = (PWSTR)FrLdrHeapAlloc(Length, TAG_WLDR_NAME);
//     if (Buffer == NULL)
//     {
//         FrLdrHeapFree(BaseDllNameBuffer, TAG_WLDR_NAME);
//         FrLdrHeapFree(DataTableEntry, TAG_WLDR_DTE);
//         return FALSE;
//     }
//
//     DataTableEntry->FullDllName.Length = Length;
//     DataTableEntry->FullDllName.MaximumLength = Length;
//     DataTableEntry->FullDllName.Buffer = PaToVa(Buffer);
//
//     RtlZeroMemory(Buffer, Length);
//     Length /= sizeof(WCHAR);
//     while (Length--)
//     {
//         *Buffer++ = *FullDllName++;
//     }
//
//     /* Initialize what's left - LoadCount which is 1, and set Flags so that
//        we know this entry is processed */
//     DataTableEntry->Flags = LDRP_ENTRY_PROCESSED;
//     DataTableEntry->LoadCount = 1;
//
//     /* Honour the FORCE_INTEGRITY flag */
//     if (NtHeaders->OptionalHeader.DllCharacteristics & IMAGE_DLLCHARACTERISTICS_FORCE_INTEGRITY)
//     {
//         /*
//          * On Vista and above, the LDRP_IMAGE_INTEGRITY_FORCED flag must be set
//          * if IMAGE_DLLCHARACTERISTICS_FORCE_INTEGRITY is set in the image header.
//          * This is done after the image has been loaded and the digital signature
//          * check has passed successfully. (We do not do it yet!)
//          *
//          * Several OS functionality depend on the presence of this flag.
//          * For example, when using Object-Manager callbacks the latter will call
//          * MmVerifyCallbackFunction() to verify whether the flag is present.
//          * If not callbacks will not work.
//          * (See Windows Internals Part 1, 6th edition, p. 176.)
//          */
//         DataTableEntry->Flags |= LDRP_IMAGE_INTEGRITY_FORCED;
//     }
//
//     /* Insert this DTE to a list in the LPB */
//     InsertTailList(ModuleListHead, &DataTableEntry->InLoadOrderLinks);
//     TRACE("Inserting DTE %p, name='%.*S' DllBase=%p\n", DataTableEntry,
//           DataTableEntry->BaseDllName.Length / sizeof(WCHAR),
//           VaToPa(DataTableEntry->BaseDllName.Buffer),
//           DataTableEntry->DllBase);
//
//     /* Save pointer to a newly allocated and initialized entry */
//     *NewEntry = DataTableEntry;
//
//     /* Return success */
//     return TRUE;
// }
//
// VOID
// PeLdrFreeDataTableEntry(
//     // _In_ PLIST_ENTRY ModuleListHead,
//     _In_ PLDR_DATA_TABLE_ENTRY Entry)
// {
//     // ASSERT(ModuleListHead);
//     ASSERT(Entry);
//
//     RemoveEntryList(&Entry->InLoadOrderLinks);
//     FrLdrHeapFree(VaToPa(Entry->FullDllName.Buffer), TAG_WLDR_NAME);
//     FrLdrHeapFree(VaToPa(Entry->BaseDllName.Buffer), TAG_WLDR_NAME);
//     FrLdrHeapFree(Entry, TAG_WLDR_DTE);
// }
//
// /**
//  * @brief
//  * Loads the specified image from the file.
//  *
//  * PeLdrLoadImage doesn't perform any additional operations on the file path,
//  * it just directly calls the file I/O routines. It then relocates the image
//  * so that it's ready to be used when paging is enabled.
//  *
//  * @note
//  * Addressing mode: physical.
//  **/
// BOOLEAN
// PeLdrLoadImageEx(
//     _In_ PCSTR FilePath,
//     _In_ TYPE_OF_MEMORY MemoryType,
//     _Out_ PVOID* ImageBasePA,
//     _In_ BOOLEAN KernelMapping)
// {
//     ULONG FileId;
//     PVOID PhysicalBase;
//     PVOID VirtualBase = NULL;
//     UCHAR HeadersBuffer[SECTOR_SIZE * 2];
//     PIMAGE_NT_HEADERS NtHeaders;
//     PIMAGE_SECTION_HEADER SectionHeader;
//     ULONG VirtualSize, SizeOfRawData, NumberOfSections;
//     ARC_STATUS Status;
//     LARGE_INTEGER Position;
//     ULONG i, BytesRead;
//
//     TRACE("PeLdrLoadImage('%s', %ld)\n", FilePath, MemoryType);
//
//     /* Open the image file */
//     Status = ArcOpen((PSTR)FilePath, OpenReadOnly, &FileId);
//     if (Status != ESUCCESS)
//     {
//         WARN("ArcOpen('%s') failed. Status: %u\n", FilePath, Status);
//         return FALSE;
//     }
//
//     /* Load the first 2 sectors of the image so we can read the PE header */
//     Status = ArcRead(FileId, HeadersBuffer, SECTOR_SIZE * 2, &BytesRead);
//     if (Status != ESUCCESS)
//     {
//         ERR("ArcRead('%s') failed. Status: %u\n", FilePath, Status);
//         ArcClose(FileId);
//         return FALSE;
//     }
//
//     /* Now read the MZ header to get the offset to the PE Header */
//     NtHeaders = RtlImageNtHeader(HeadersBuffer);
//     if (!NtHeaders)
//     {
//         ERR("No NT header found in \"%s\"\n", FilePath);
//         ArcClose(FileId);
//         return FALSE;
//     }
//
//     /* Ensure this is executable image */
//     if (((NtHeaders->FileHeader.Characteristics & IMAGE_FILE_EXECUTABLE_IMAGE) == 0))
//     {
//         ERR("Not an executable image \"%s\"\n", FilePath);
//         ArcClose(FileId);
//         return FALSE;
//     }
//
//     /* Store number of sections to read and a pointer to the first section */
//     NumberOfSections = NtHeaders->FileHeader.NumberOfSections;
//     SectionHeader = IMAGE_FIRST_SECTION(NtHeaders);
//
//     /* Try to allocate this memory; if it fails, allocate somewhere else */
//     PhysicalBase = MmAllocateMemoryAtAddress(NtHeaders->OptionalHeader.SizeOfImage,
//                        (PVOID)((ULONG)NtHeaders->OptionalHeader.ImageBase & (KSEG0_BASE - 1)),
//                        MemoryType);
//
//     if (PhysicalBase == NULL)
//     {
//         /* Don't fail, allocate again at any other "low" place */
//         PhysicalBase = MmAllocateMemoryWithType(NtHeaders->OptionalHeader.SizeOfImage, MemoryType);
//
//         if (PhysicalBase == NULL)
//         {
//             ERR("Failed to alloc %lu bytes for image %s\n", NtHeaders->OptionalHeader.SizeOfImage, FilePath);
//             ArcClose(FileId);
//             return FALSE;
//         }
//     }
//
//     /* This is the real image base, in form of a virtual address */
//     VirtualBase = KernelMapping ? PaToVa(PhysicalBase) : PhysicalBase;
//
//     TRACE("Base PA: 0x%p, VA: 0x%p\n", PhysicalBase, VirtualBase);
//
//     /* Copy headers from already read data */
//     RtlCopyMemory(PhysicalBase, HeadersBuffer, min(NtHeaders->OptionalHeader.SizeOfHeaders, sizeof(HeadersBuffer)));
//     /* If headers are quite big, request next bytes from file */
//     if (NtHeaders->OptionalHeader.SizeOfHeaders > sizeof(HeadersBuffer))
//     {
//         Status = ArcRead(FileId, (PUCHAR)PhysicalBase + sizeof(HeadersBuffer), NtHeaders->OptionalHeader.SizeOfHeaders - sizeof(HeadersBuffer), &BytesRead);
//         if (Status != ESUCCESS)
//         {
//             ERR("ArcRead('%s') failed. Status: %u\n", FilePath, Status);
//             // UiMessageBox("Error reading headers.");
//             ArcClose(FileId);
//             goto Failure;
//         }
//     }
//
//     /*
//      * On Vista and above, a digital signature check is performed when the image
//      * has the IMAGE_DLLCHARACTERISTICS_FORCE_INTEGRITY flag set in its header.
//      * (We of course do not perform this check yet!)
//      */
//
//     /* Reload the NT Header */
//     NtHeaders = RtlImageNtHeader(PhysicalBase);
//
//     /* Load the first section */
//     SectionHeader = IMAGE_FIRST_SECTION(NtHeaders);
//
//     /* Walk through each section and read it (check/fix any possible
//        bad situations, if they arise) */
//     for (i = 0; i < NumberOfSections; i++)
//     {
//         VirtualSize = SectionHeader->Misc.VirtualSize;
//         SizeOfRawData = SectionHeader->SizeOfRawData;
//
//         /* Handle a case when VirtualSize equals 0 */
//         if (VirtualSize == 0)
//             VirtualSize = SizeOfRawData;
//
//         /* If PointerToRawData is 0, then force its size to be also 0 */
//         if (SectionHeader->PointerToRawData == 0)
//         {
//             SizeOfRawData = 0;
//         }
//         else
//         {
//             /* Cut the loaded size to the VirtualSize extents */
//             if (SizeOfRawData > VirtualSize)
//                 SizeOfRawData = VirtualSize;
//         }
//
//         /* Actually read the section (if its size is not 0) */
//         if (SizeOfRawData != 0)
//         {
//             /* Seek to the correct position */
//             Position.QuadPart = SectionHeader->PointerToRawData;
//             Status = ArcSeek(FileId, &Position, SeekAbsolute);
//
//             TRACE("SH->VA: 0x%X\n", SectionHeader->VirtualAddress);
//
//             /* Read this section from the file, size = SizeOfRawData */
//             Status = ArcRead(FileId, (PUCHAR)PhysicalBase + SectionHeader->VirtualAddress, SizeOfRawData, &BytesRead);
//             if (Status != ESUCCESS)
//             {
//                 ERR("PeLdrLoadImage(): Error reading section from file!\n");
//                 break;
//             }
//         }
//
//         /* Size of data is less than the virtual size: fill up the remainder with zeroes */
//         if (SizeOfRawData < VirtualSize)
//         {
//             TRACE("PeLdrLoadImage(): SORD %d < VS %d\n", SizeOfRawData, VirtualSize);
//             RtlZeroMemory((PVOID)(SectionHeader->VirtualAddress + (ULONG_PTR)PhysicalBase + SizeOfRawData), VirtualSize - SizeOfRawData);
//         }
//
//         SectionHeader++;
//     }
//
//     /* We are done with the file, close it */
//     ArcClose(FileId);
//
//     /* If loading failed, return right now */
//     if (Status != ESUCCESS)
//         goto Failure;
//
//     /* Relocate the image, if it needs it */
//     if (NtHeaders->OptionalHeader.ImageBase != (ULONG_PTR)VirtualBase)
//     {
//         WARN("Relocating %p -> %p\n", NtHeaders->OptionalHeader.ImageBase, VirtualBase);
//         Status = LdrRelocateImageWithBias(PhysicalBase,
//                                           (ULONG_PTR)VirtualBase - (ULONG_PTR)PhysicalBase,
//                                           "FreeLdr",
//                                           ESUCCESS,
//                                           ESUCCESS, /* In case of conflict still return success */
//                                           ENOEXEC);
//         if (Status != ESUCCESS)
//             goto Failure;
//     }
//
//     /* Fill output parameters */
//     *ImageBasePA = PhysicalBase;
//
//     TRACE("PeLdrLoadImage() done, PA = %p\n", *ImageBasePA);
//     return TRUE;
//
// Failure:
//     /* Cleanup and bail out */
//     MmFreeMemory(PhysicalBase);
//     return FALSE;
// }
//
// BOOLEAN
// PeLdrLoadImage(
//     _In_ PCSTR FilePath,
//     _In_ TYPE_OF_MEMORY MemoryType,
//     _Out_ PVOID* ImageBasePA)
// {
//     return PeLdrLoadImageEx(FilePath, MemoryType, ImageBasePA, TRUE);
// }
//
// BOOLEAN
// PeLdrLoadBootImage(
//     _In_ PCSTR FilePath,
//     _In_ PCSTR BaseDllName,
//     _Out_ PVOID* ImageBase,
//     _Out_ PLDR_DATA_TABLE_ENTRY* DataTableEntry)
// {
//     BOOLEAN Success;
//
//     /* Load the image as a bootloader image */
//     Success = PeLdrLoadImageEx(FilePath,
//                                LoaderLoadedProgram,
//                                ImageBase,
//                                FALSE);
//     if (!Success)
//     {
//         WARN("Failed to load boot image '%s'\n", FilePath);
//         return FALSE;
//     }
//
//     /* Allocate a DTE */
//     Success = PeLdrAllocateDataTableEntry(&FrLdrModuleList,
//                                           BaseDllName,
//                                           FilePath,
//                                           *ImageBase,
//                                           DataTableEntry);
//     if (!Success)
//     {
//         /* Cleanup and bail out */
//         ERR("Failed to allocate DTE for '%s'\n", FilePath);
//         MmFreeMemory(*ImageBase);
//         return FALSE;
//     }
//
//     /* Resolve imports */
//     Success = PeLdrScanImportDescriptorTable(&FrLdrModuleList, "", *DataTableEntry);
//     if (!Success)
//     {
//         /* Cleanup and bail out */
//         ERR("Failed to resolve imports for '%s'\n", FilePath);
//         PeLdrFreeDataTableEntry(*DataTableEntry);
//         MmFreeMemory(*ImageBase);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
