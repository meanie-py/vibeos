#[allow(unused_imports)] use crate::{TRACE, WARN};
#[allow(dead_code, non_upper_case_globals)] const DbgDefaultChannel: u32 = crate::frldr::DPRINT_WINDOWS; // wlregistry.c: DBG_DEFAULT_CHANNEL(WINDOWS)
use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{ARC_STATUS, BOOT_DRIVER_NODE, CmpAllocate, CmpFindDrivers, CmpFree, CmpFreeDriverList, CmpResolveDriverDependencies, CmpSortDriverList, FILEINFORMATION, PBOOT_DRIVER_LIST_ENTRY, PBOOT_DRIVER_NODE, PCSTR, PCUNICODE_STRING, PHBASE_BLOCK, PLOADER_PARAMETER_BLOCK, PUNICODE_STRING, RtlAppendUnicodeStringToString, RtlAppendUnicodeToString, RtlCompareUnicodeString, RtlCreateUnicodeString, RtlFreeUnicodeString, RtlInitEmptyUnicodeString, RtlInitUnicodeString, _UNICODE_STRING};

// registry.h: typedef HANDLE HKEY, *PHKEY;
pub type HKEY = ntkrnl_sys::ddk::HANDLE;
pub type PHKEY = *mut HKEY;
// winerror.h
pub const ERROR_SUCCESS: i32 = 0;

// wlregistry.c defines these (lines 20-21); wlmemory.c externs them.
#[no_mangle]
pub static mut TotalNLSSize: u32 = 0;
#[no_mangle]
pub static mut NlsDataBase: *mut core::ffi::c_void = core::ptr::null_mut();

// wlregistry.c file-local enum (GoodHive = 1).
#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum BAD_HIVE_REASON {
    GoodHive = 1,
    CorruptHive,
    NoHive,
    NoHiveAlloc,
}
pub type PBAD_HIVE_REASON = *mut BAD_HIVE_REASON;

// registry.c macro: HKEY_TO_HCI(hKey) == (HCELL_INDEX)(ULONG_PTR)hKey
#[inline]
unsafe fn HKEY_TO_HCI(hKey: HKEY) -> ntkrnl_sys::ddk::HCELL_INDEX {
    hKey as ULONG_PTR as ntkrnl_sys::ddk::HCELL_INDEX
}
// registry.h macro: RegCloseKey(hKey) == ERROR_SUCCESS
#[inline]
#[no_mangle]
pub unsafe extern "C" fn RegCloseKey(_hKey: HKEY) -> i32 {
    ERROR_SUCCESS
}
use core::ffi::c_void;
pub use ntkrnl_sys::ddk::_CM_SERVICE_LOAD_TYPE_BootLoad as BootLoad;
pub use ntkrnl_sys::ddk::_ARC_CODES_ESUCCESS as ESUCCESS;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderNlsData as LoaderNlsData;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderRegistryData as LoaderRegistryData;
pub use ntkrnl_sys::ddk::_OPENMODE_OpenReadOnly as OpenReadOnly;

extern "C" {
    // registry.c defines SystemHive (PHHIVE); winldr.c defines
    // WinLdrSystemBlock (PLOADER_SYSTEM_BLOCK). This TU references both.
    static mut SystemHive: ntkrnl_sys::ddk::PHHIVE;
    static mut WinLdrSystemBlock: PLOADER_SYSTEM_BLOCK;
    fn ArcClose(FileId: u32) -> ARC_STATUS;
    fn ArcGetFileInformation(FileId: u32, FileInfo: *mut FILEINFORMATION) -> ARC_STATUS;
    fn ArcOpen(Path: PCSTR, OpenMode: u32, FileId: *mut u32) -> ARC_STATUS;
    fn ArcRead(FileId: u32, Buffer: *mut core::ffi::c_void, Length: u32, BytesRead: *mut u32) -> ARC_STATUS;
    fn FsGetServiceName(FileId: u32) -> PCSTR;
    fn MM_SIZE_TO_PAGES(Size: u32) -> u32;
    fn MmAllocateMemoryWithType(Size: usize, Type: u32) -> *mut core::ffi::c_void;
    fn NtLdrOutputLoadMsg(Path: PCSTR, Name: PCSTR);
    fn PaToVa(Pa: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
    fn RtlStringCbCatA(Dest: *mut u8, DestSize: usize, Src: PCSTR) -> NTSTATUS;
    static mut BootFileSystem: PCSTR;
    fn RtlStringCbPrintfA(pszDest: *mut i8, cbDest: usize, pszFormat: PCSTR, ...) -> i32;
    fn UiMessageBox(fmt: PCSTR, ...);
    fn VaToPa(Va: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
    fn wcslen(s: PCWSTR) -> usize;
    fn RegImportBinaryHive( ChunkBase: *mut core::ffi::c_void, ChunkSize: u32, Name: PCSTR, Recover: BOOLEAN, ) -> BOOLEAN;
    fn RegInitCurrentControlSet(IsSetup: BOOLEAN) -> BOOLEAN;
    static mut CurrentControlSetKey: HKEY;
    // registry.c FLDRAPI (cdecl) functions
    fn RegOpenKey(ParentKey: HKEY, KeyName: PCWSTR, Key: *mut HKEY) -> i32;
    fn RegQueryValue( Key: HKEY, ValueName: PCWSTR, Type: *mut u32, Data: *mut u8, DataSize: *mut u32, ) -> i32;
}

// ntoskrnl/config/cmboot.c CmpIsDriverInList is NTAPI (stdcall on i386);
// libuefifreeldr_common.a exports `_CmpIsDriverInList@12`. Declaring it
// cdecl (`extern "C"`) made the i386 link fail with undefined
// `_CmpIsDriverInList` (no `@12`). Must be `extern "system"`.
extern "system" {
    fn CmpIsDriverInList(
        DriverListHead: PLIST_ENTRY,
        DriverName: *mut _UNICODE_STRING,
        DriverNode: *mut PBOOT_DRIVER_NODE,
    ) -> u8;
}

const MAX_PATH: usize = 260;
const MM_PAGE_SIZE: u32 = 4096;
const MM_PAGE_SHIFT: u32 = 12;

unsafe fn WinLdrLoadSystemHive(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
    DirectoryPath: PCSTR,
    HiveName: PCSTR,
    Reason: *mut BAD_HIVE_REASON,
) -> BOOLEAN {

    let mut FileId: u32 = 0;
    let mut FullHiveName: [u8; 260] = [0; 260];
    let mut Status: ARC_STATUS;
    let mut FileInfo: FILEINFORMATION = core::mem::zeroed();
    let HiveFileSize: u32;
    let HiveDataPhysical: *mut core::ffi::c_void;
    let HiveDataVirtual: *mut core::ffi::c_void;
    let mut BytesRead: u32 = 0;

    *Reason = BAD_HIVE_REASON::GoodHive;

    RtlStringCbCopyA(FullHiveName.as_mut_ptr() as *mut i8 , core::mem::size_of_val(&FullHiveName), DirectoryPath);
    RtlStringCbCatA(FullHiveName.as_mut_ptr(), core::mem::size_of_val(&FullHiveName), HiveName);

    NtLdrOutputLoadMsg(FullHiveName.as_ptr() as PCSTR, core::ptr::null());
    Status = ArcOpen(FullHiveName.as_ptr() as PCSTR, OpenReadOnly, &mut FileId);
    if Status != ESUCCESS {
        WARN!("Error while opening '%s', Status: %u\n", (FullHiveName.as_ptr()) as usize, (Status) as usize);
        *Reason = BAD_HIVE_REASON::NoHive;
        return 0u8;
    }

    Status = ArcGetFileInformation(FileId, &mut FileInfo);
    if Status != ESUCCESS {
        WARN!("Hive file has 0 size!\n");
        *Reason = BAD_HIVE_REASON::CorruptHive;
        ArcClose(FileId);
        return 0u8;
    }
    HiveFileSize = (*core::ptr::addr_of!(FileInfo.EndingAddress)).__bindgen_anon_1.LowPart;

    HiveDataPhysical = MmAllocateMemoryWithType(
        (MM_SIZE_TO_PAGES(HiveFileSize + 4096 - 1) << 12) as usize,
        LoaderRegistryData,
    );

    if HiveDataPhysical.is_null() {
        WARN!("Could not alloc memory for hive!\n");
        *Reason = BAD_HIVE_REASON::NoHiveAlloc;
        ArcClose(FileId);
        return 0u8;
    }

    HiveDataVirtual = PaToVa(HiveDataPhysical);

    (*LoaderBlock).RegistryLength = HiveFileSize;
    (*LoaderBlock).RegistryBase = HiveDataVirtual;

    Status = ArcRead(FileId, HiveDataPhysical, HiveFileSize, &mut BytesRead);
    if Status != ESUCCESS {
        WARN!("Error while reading '%s', Status: %u\n", (FullHiveName.as_ptr()) as usize, (Status) as usize);
        *Reason = BAD_HIVE_REASON::CorruptHive;
        ArcClose(FileId);
        return 0u8;
    }

    BootFileSystem = FsGetServiceName(FileId);

    ArcClose(FileId);
    1u8
}

#[no_mangle]
pub unsafe extern "C" fn WinLdrInitSystemHive(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
    SystemRoot: PCSTR,
    Setup: BOOLEAN,
) -> BOOLEAN {

    let mut SearchPath: [u8; 1024] = [0; 1024];
    let ChunkBase: *mut core::ffi::c_void;
    let mut HiveName: PCSTR;
    let mut Success: BOOLEAN;
    let mut Reason: BAD_HIVE_REASON = BAD_HIVE_REASON::GoodHive;

    if Setup != 0 {
        RtlStringCbCopyA(SearchPath.as_mut_ptr() as *mut i8 , 1024, SystemRoot);
        HiveName = b"SETUPREG.HIV\0".as_ptr() as PCSTR;
    } else {
        RtlStringCbCopyA(SearchPath.as_mut_ptr() as *mut i8 , 1024, SystemRoot);
        RtlStringCbCatA(SearchPath.as_mut_ptr(), 1024, b"system32\\config\\\0".as_ptr() as *const i8 );
        HiveName = b"SYSTEM\0".as_ptr() as PCSTR;
    }

    TRACE!("WinLdrInitSystemHive: loading hive %s%s\n", (SearchPath.as_ptr()) as usize, (HiveName) as usize);
    Success = WinLdrLoadSystemHive(LoaderBlock, SearchPath.as_ptr() as *const i8 , HiveName, &mut Reason);
    if Success == 0 {
        if Reason == BAD_HIVE_REASON::CorruptHive || Reason == BAD_HIVE_REASON::NoHive {
            HiveName = b"SYSTEM.ALT\0".as_ptr() as PCSTR;
            Success = WinLdrLoadSystemHive(LoaderBlock, SearchPath.as_ptr() as *const i8 , HiveName, &mut Reason);
            if Success == 0 {
                UiMessageBox(b"Could not load %s hive!\0".as_ptr() as *const i8 , HiveName);
                return 0u8;
            }

            Success = RegImportBinaryHive(VaToPa((*LoaderBlock).RegistryBase), (*LoaderBlock).RegistryLength, SearchPath.as_ptr() as *const i8 , 1u8);
            if Success == 0 {
                UiMessageBox(b"Importing binary hive failed!\0".as_ptr() as *const i8 );
                return 0u8;
            }

            WARN!("SYSTEM hive does not exist or is corrupt and SYSTEM.ALT has been loaded!\n");
            ChunkBase = VaToPa((*LoaderBlock).RegistryBase);
            (*(ChunkBase as PHBASE_BLOCK)).BootRecover = 0x2u32;
        } else {
            UiMessageBox(b"Could not load %s hive!\0".as_ptr() as *const i8 , HiveName);
            return 0u8;
        }
    } else {
        Success = RegImportBinaryHive(VaToPa((*LoaderBlock).RegistryBase), (*LoaderBlock).RegistryLength, SearchPath.as_ptr() as *const i8 , 0u8);
        if Success == 0 {
            HiveName = b"SYSTEM.ALT\0".as_ptr() as PCSTR;
            Success = WinLdrLoadSystemHive(LoaderBlock, SearchPath.as_ptr() as *const i8 , HiveName, &mut Reason);
            if Success == 0 {
                UiMessageBox(b"Could not load %s hive!\0".as_ptr() as *const i8 , HiveName);
                return 0u8;
            }

            Success = RegImportBinaryHive(VaToPa((*LoaderBlock).RegistryBase), (*LoaderBlock).RegistryLength, SearchPath.as_ptr() as *const i8 , 1u8);
            if Success == 0 {
                UiMessageBox(b"Importing binary hive failed!\0".as_ptr() as *const i8 );
                return 0u8;
            }

            WARN!("SYSTEM hive does not exist or is corrupt and SYSTEM.ALT has been loaded!\n");
            ChunkBase = VaToPa((*LoaderBlock).RegistryBase);
            (*(ChunkBase as PHBASE_BLOCK)).BootRecover = 0x2u32;
        }
    }

    if RegInitCurrentControlSet(0u8) == 0 {
        UiMessageBox(b"Initializing CurrentControlSet link failed!\0".as_ptr() as *const i8 );
        return 0u8;
    }

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn WinLdrScanSystemHive(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
    SystemRoot: PCSTR,
) -> u8 {

    let mut Success: u8;
    // ntdef.h DECLARE_UNICODE_STRING_SIZE(v,260): WCHAR v_buffer[260];
    // UNICODE_STRING v = { 0, 260*sizeof(WCHAR), v_buffer };
    let mut AnsiFileName_buffer: [u16; 260] = [0; 260];
    let mut OemFileName_buffer: [u16; 260] = [0; 260];
    let mut LangFileName_buffer: [u16; 260] = [0; 260];
    let mut OemHalFileName_buffer: [u16; 260] = [0; 260];
    let mut AnsiFileName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: (260 * core::mem::size_of::<u16>()) as USHORT,
        Buffer: AnsiFileName_buffer.as_mut_ptr(),
    };
    let mut OemFileName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: (260 * core::mem::size_of::<u16>()) as USHORT,
        Buffer: OemFileName_buffer.as_mut_ptr(),
    };
    let mut LangFileName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: (260 * core::mem::size_of::<u16>()) as USHORT,
        Buffer: LangFileName_buffer.as_mut_ptr(),
    };
    let mut OemHalFileName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: (260 * core::mem::size_of::<u16>()) as USHORT,
        Buffer: OemHalFileName_buffer.as_mut_ptr(),
    };
    let mut SearchPath: [u8; 1024] = [0; 1024];

    Success = WinLdrScanRegistry(core::ptr::addr_of_mut!((*LoaderBlock).BootDriverListHead));
    if Success == 0 {
        UiMessageBox(b"Failed to load boot drivers!\0".as_ptr() as PCSTR);
        return 0u8;
    }

    Success = WinLdrGetNLSNames(
        CurrentControlSetKey,
        core::ptr::addr_of_mut!(AnsiFileName),
        core::ptr::addr_of_mut!(OemFileName),
        core::ptr::addr_of_mut!(LangFileName),
        core::ptr::addr_of_mut!(OemHalFileName),
    );
    if Success == 0 {
        UiMessageBox(b"Getting NLS names from registry failed!\0".as_ptr() as PCSTR);
        return 0u8;
    }

    TRACE!("NLS data: '%wZ' '%wZ' '%wZ' '%wZ'\n", (core::ptr::addr_of!(AnsiFileName)) as usize, (core::ptr::addr_of!(OemFileName)) as usize, (core::ptr::addr_of!(LangFileName)) as usize, (core::ptr::addr_of!(OemHalFileName)) as usize);

    RtlStringCbCopyA(
        SearchPath.as_mut_ptr() as *mut i8 ,
        core::mem::size_of_val(&SearchPath),
        SystemRoot,
    );
    RtlStringCbCatA(
        SearchPath.as_mut_ptr(),
        core::mem::size_of_val(&SearchPath),
        b"system32\\\0".as_ptr() as PCSTR,
    );
    Success = WinLdrLoadNLSData(
        LoaderBlock,
        SearchPath.as_ptr() as PCSTR,
        core::ptr::addr_of_mut!(AnsiFileName),
        core::ptr::addr_of_mut!(OemFileName),
        core::ptr::addr_of_mut!(LangFileName),
        core::ptr::addr_of_mut!(OemHalFileName),
    );
    TRACE!("NLS data loading %s\n", (if Success != 0 {
            b"successful\0".as_ptr() as PCSTR
        } else {
            b"failed\0".as_ptr() as PCSTR
        }) as usize);

    1u8
}

unsafe fn WinLdrGetNLSNames(
    ControlSet: HKEY,
    AnsiFileName: PUNICODE_STRING,
    OemFileName: PUNICODE_STRING,
    LangFileName: PUNICODE_STRING,
    OemHalFileName: PUNICODE_STRING,
) -> BOOLEAN {

    let mut rc: i32;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut BufferSize: u32;
    let mut szIdBuffer: [u16; 80] = [0; 80];

    rc = RegOpenKey(ControlSet, crate::wide!("Control\\NLS\\CodePage\0"), &mut hKey);
    if rc != ERROR_SUCCESS {
        return 0u8;
    }

    BufferSize = core::mem::size_of_val(&szIdBuffer) as u32;
    rc = RegQueryValue(hKey, crate::wide!("ACP\0"), core::ptr::null_mut(), szIdBuffer.as_mut_ptr() as *mut u8, &mut BufferSize);
    if rc != ERROR_SUCCESS {
        RegCloseKey(hKey);
        return 0u8;
    }

    BufferSize = (*AnsiFileName).MaximumLength as u32;
    rc = RegQueryValue(hKey, szIdBuffer.as_ptr(), core::ptr::null_mut(),
                       (*AnsiFileName).Buffer as *mut u8, &mut BufferSize);
    if rc != ERROR_SUCCESS {
        (*AnsiFileName).Length = 0;
        RtlAppendUnicodeToString(AnsiFileName, crate::wide!("c_1252.nls\0"));
    } else {
        (*AnsiFileName).Length = BufferSize as u16 - core::mem::size_of::<u16>() as u16;
    }

    BufferSize = core::mem::size_of_val(&szIdBuffer) as u32;
    rc = RegQueryValue(hKey, crate::wide!("OEMCP\0"), core::ptr::null_mut(), szIdBuffer.as_mut_ptr() as *mut u8, &mut BufferSize);
    if rc != ERROR_SUCCESS {
        RegCloseKey(hKey);
        return 0u8;
    }

    BufferSize = (*OemFileName).MaximumLength as u32;
    rc = RegQueryValue(hKey, szIdBuffer.as_ptr(), core::ptr::null_mut(),
                       (*OemFileName).Buffer as *mut u8, &mut BufferSize);
    if rc != ERROR_SUCCESS {
        (*OemFileName).Length = 0;
        RtlAppendUnicodeToString(OemFileName, crate::wide!("c_437.nls\0"));
    } else {
        (*OemFileName).Length = BufferSize as u16 - core::mem::size_of::<u16>() as u16;
    }

    BufferSize = (*OemHalFileName).MaximumLength as u32;
    rc = RegQueryValue(hKey, crate::wide!("OEMHAL\0"), core::ptr::null_mut(),
                       (*OemHalFileName).Buffer as *mut u8, &mut BufferSize);
    if rc != ERROR_SUCCESS {
        RtlInitEmptyUnicodeString(OemHalFileName, core::ptr::null_mut(), 0);
    } else {
        (*OemHalFileName).Length = BufferSize as u16 - core::mem::size_of::<u16>() as u16;
    }

    RegCloseKey(hKey);

    rc = RegOpenKey(ControlSet, crate::wide!("Control\\NLS\\Language\0"), &mut hKey);
    if rc != ERROR_SUCCESS {
        return 0u8;
    }

    BufferSize = core::mem::size_of_val(&szIdBuffer) as u32;
    rc = RegQueryValue(hKey, crate::wide!("Default\0"), core::ptr::null_mut(), szIdBuffer.as_mut_ptr() as *mut u8, &mut BufferSize);
    if rc != ERROR_SUCCESS {
        RegCloseKey(hKey);
        return 0u8;
    }

    BufferSize = (*LangFileName).MaximumLength as u32;
    rc = RegQueryValue(hKey, szIdBuffer.as_ptr(), core::ptr::null_mut(),
                       (*LangFileName).Buffer as *mut u8, &mut BufferSize);
    if rc != ERROR_SUCCESS {
        (*LangFileName).Length = 0;
        RtlAppendUnicodeToString(LangFileName, crate::wide!("l_intl.nls\0"));
    } else {
        (*LangFileName).Length = BufferSize as u16 - core::mem::size_of::<u16>() as u16;
    }

    RegCloseKey(hKey);
    if rc == ERROR_SUCCESS { 1u8 } else { 0u8 }
}

#[no_mangle]
pub unsafe extern "C" fn WinLdrLoadNLSData(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
    DirectoryPath: PCSTR,
    AnsiFileName: PCUNICODE_STRING,
    OemFileName: PCUNICODE_STRING,
    LangFileName: PCUNICODE_STRING,
    _OemHalFileName: PCUNICODE_STRING,
) -> u8 {
    let mut Status: ARC_STATUS;
    let mut FileInfo: FILEINFORMATION = core::mem::zeroed();
    let mut AnsiFileId: u32 = u32::MAX;
    let mut OemFileId: u32 = u32::MAX;
    let mut LangFileId: u32 = u32::MAX;
    let AnsiFileSize: u32;
    let OemFileSize: u32;
    let LangFileSize: u32;
    let TotalSize: u32;
    let mut BytesRead: u32 = 0;
    let NlsVirtual: *mut core::ffi::c_void;
    let mut AnsiEqualsOem: u8 = 0u8;
    let mut FileName: [i8; MAX_PATH] = [0; MAX_PATH];

    if RtlCompareUnicodeString(AnsiFileName, OemFileName, 1) == 0 {
        AnsiEqualsOem = 1u8;
    }

    RtlStringCbPrintfA(
        FileName.as_mut_ptr(),
        core::mem::size_of_val(&FileName),
        b"%s%wZ\0".as_ptr() as PCSTR,
        DirectoryPath,
        AnsiFileName,
    );
    Status = ArcOpen(FileName.as_ptr(), OpenReadOnly, &mut AnsiFileId);
    if Status != ESUCCESS {
        WARN!("Error while opening '%s', Status: %u\n", (FileName.as_ptr()) as usize, (Status) as usize);
        'Quit: {
            if LangFileId != u32::MAX {
                ArcClose(LangFileId);
            }
            if OemFileId != u32::MAX {
                ArcClose(OemFileId);
            }
            if AnsiFileId != u32::MAX {
                ArcClose(AnsiFileId);
            }

            if Status != ESUCCESS {
                UiMessageBox(
                    b"Error reading NLS file %s\0".as_ptr() as PCSTR,
                    FileName.as_ptr(),
                );
            }

            return if Status == ESUCCESS { 1u8 } else { 0u8 };
        }
    }

    Status = ArcGetFileInformation(AnsiFileId, &mut FileInfo);
    if Status != ESUCCESS {
        'Quit: {
            if LangFileId != u32::MAX {
                ArcClose(LangFileId);
            }
            if OemFileId != u32::MAX {
                ArcClose(OemFileId);
            }
            if AnsiFileId != u32::MAX {
                ArcClose(AnsiFileId);
            }

            if Status != ESUCCESS {
                UiMessageBox(
                    b"Error reading NLS file %s\0".as_ptr() as PCSTR,
                    FileName.as_ptr(),
                );
            }

            return if Status == ESUCCESS { 1u8 } else { 0u8 };
        }
    }
    AnsiFileSize = FileInfo.EndingAddress.__bindgen_anon_1.LowPart;
    TRACE!("AnsiFileSize: %d\n", (AnsiFileSize) as usize);

    if AnsiEqualsOem != 0 {
        OemFileSize = 0;
    } else {
        RtlStringCbPrintfA(
            FileName.as_mut_ptr(),
            core::mem::size_of_val(&FileName),
            b"%s%wZ\0".as_ptr() as PCSTR,
            DirectoryPath,
            OemFileName,
        );
        Status = ArcOpen(FileName.as_ptr(), OpenReadOnly, &mut OemFileId);
        if Status != ESUCCESS {
            WARN!("Error while opening '%s', Status: %u\n", (FileName.as_ptr()) as usize, (Status) as usize);
            'Quit: {
                if LangFileId != u32::MAX {
                    ArcClose(LangFileId);
                }
                if OemFileId != u32::MAX {
                    ArcClose(OemFileId);
                }
                if AnsiFileId != u32::MAX {
                    ArcClose(AnsiFileId);
                }

                if Status != ESUCCESS {
                    UiMessageBox(
                        b"Error reading NLS file %s\0".as_ptr() as PCSTR,
                        FileName.as_ptr(),
                    );
                }

                return if Status == ESUCCESS { 1u8 } else { 0u8 };
            }
        }

        Status = ArcGetFileInformation(OemFileId, &mut FileInfo);
        if Status != ESUCCESS {
            'Quit: {
                if LangFileId != u32::MAX {
                    ArcClose(LangFileId);
                }
                if OemFileId != u32::MAX {
                    ArcClose(OemFileId);
                }
                if AnsiFileId != u32::MAX {
                    ArcClose(AnsiFileId);
                }

                if Status != ESUCCESS {
                    UiMessageBox(
                        b"Error reading NLS file %s\0".as_ptr() as PCSTR,
                        FileName.as_ptr(),
                    );
                }

                return if Status == ESUCCESS { 1u8 } else { 0u8 };
            }
        }
        OemFileSize = FileInfo.EndingAddress.__bindgen_anon_1.LowPart;
    }
    TRACE!("OemFileSize: %d\n", (OemFileSize) as usize);

    RtlStringCbPrintfA(
        FileName.as_mut_ptr(),
        core::mem::size_of_val(&FileName),
        b"%s%wZ\0".as_ptr() as PCSTR,
        DirectoryPath,
        LangFileName,
    );
    Status = ArcOpen(FileName.as_ptr(), OpenReadOnly, &mut LangFileId);
    if Status != ESUCCESS {
        WARN!("Error while opening '%s', Status: %u\n", (FileName.as_ptr()) as usize, (Status) as usize);
        'Quit: {
            if LangFileId != u32::MAX {
                ArcClose(LangFileId);
            }
            if OemFileId != u32::MAX {
                ArcClose(OemFileId);
            }
            if AnsiFileId != u32::MAX {
                ArcClose(AnsiFileId);
            }

            if Status != ESUCCESS {
                UiMessageBox(
                    b"Error reading NLS file %s\0".as_ptr() as PCSTR,
                    FileName.as_ptr(),
                );
            }

            return if Status == ESUCCESS { 1u8 } else { 0u8 };
        }
    }

    Status = ArcGetFileInformation(LangFileId, &mut FileInfo);
    if Status != ESUCCESS {
        'Quit: {
            if LangFileId != u32::MAX {
                ArcClose(LangFileId);
            }
            if OemFileId != u32::MAX {
                ArcClose(OemFileId);
            }
            if AnsiFileId != u32::MAX {
                ArcClose(AnsiFileId);
            }

            if Status != ESUCCESS {
                UiMessageBox(
                    b"Error reading NLS file %s\0".as_ptr() as PCSTR,
                    FileName.as_ptr(),
                );
            }

            return if Status == ESUCCESS { 1u8 } else { 0u8 };
        }
    }
    LangFileSize = FileInfo.EndingAddress.__bindgen_anon_1.LowPart;
    TRACE!("LangFileSize: %d\n", (LangFileSize) as usize);

    TotalSize = MM_SIZE_TO_PAGES(AnsiFileSize)
        + MM_SIZE_TO_PAGES(OemFileSize)
        + MM_SIZE_TO_PAGES(LangFileSize);

    TotalNLSSize = TotalSize;

    NlsDataBase = MmAllocateMemoryWithType((TotalSize * MM_PAGE_SIZE) as usize, LoaderNlsData);
    if NlsDataBase.is_null() {
        'Quit: {
            if LangFileId != u32::MAX {
                ArcClose(LangFileId);
            }
            if OemFileId != u32::MAX {
                ArcClose(OemFileId);
            }
            if AnsiFileId != u32::MAX {
                ArcClose(AnsiFileId);
            }

            if Status != ESUCCESS {
                UiMessageBox(
                    b"Error reading NLS file %s\0".as_ptr() as PCSTR,
                    FileName.as_ptr(),
                );
            }

            return if Status == ESUCCESS { 1u8 } else { 0u8 };
        }
    }

    NlsVirtual = PaToVa(NlsDataBase);
    (*(*LoaderBlock).NlsData).AnsiCodePageData = NlsVirtual;

    (*(*LoaderBlock).NlsData).OemCodePageData = (NlsVirtual as usize
        + ((MM_SIZE_TO_PAGES(AnsiFileSize) << MM_PAGE_SHIFT) as usize))
        as *mut core::ffi::c_void;

    (*(*LoaderBlock).NlsData).UnicodeCodePageData = (NlsVirtual as usize
        + ((MM_SIZE_TO_PAGES(AnsiFileSize) << MM_PAGE_SHIFT) as usize)
        + ((MM_SIZE_TO_PAGES(OemFileSize) << MM_PAGE_SHIFT) as usize))
        as *mut core::ffi::c_void;

    if AnsiEqualsOem != 0 {
        (*(*LoaderBlock).NlsData).OemCodePageData = (*(*LoaderBlock).NlsData).AnsiCodePageData;
    }

    RtlStringCbPrintfA(
        FileName.as_mut_ptr(),
        core::mem::size_of_val(&FileName),
        b"%s%wZ\0".as_ptr() as PCSTR,
        DirectoryPath,
        AnsiFileName,
    );
    NtLdrOutputLoadMsg(FileName.as_ptr(), core::ptr::null());
    Status = ArcRead(
        AnsiFileId,
        VaToPa((*(*LoaderBlock).NlsData).AnsiCodePageData),
        AnsiFileSize,
        &mut BytesRead,
    );
    if Status != ESUCCESS {
        WARN!("Error while reading '%s', Status: %u\n", (FileName.as_ptr()) as usize, (Status) as usize);
        'Quit: {
            if LangFileId != u32::MAX {
                ArcClose(LangFileId);
            }
            if OemFileId != u32::MAX {
                ArcClose(OemFileId);
            }
            if AnsiFileId != u32::MAX {
                ArcClose(AnsiFileId);
            }

            if Status != ESUCCESS {
                UiMessageBox(
                    b"Error reading NLS file %s\0".as_ptr() as PCSTR,
                    FileName.as_ptr(),
                );
            }

            return if Status == ESUCCESS { 1u8 } else { 0u8 };
        }
    }

    if AnsiEqualsOem == 0 {
        RtlStringCbPrintfA(
            FileName.as_mut_ptr(),
            core::mem::size_of_val(&FileName),
            b"%s%wZ\0".as_ptr() as PCSTR,
            DirectoryPath,
            OemFileName,
        );
        NtLdrOutputLoadMsg(FileName.as_ptr(), core::ptr::null());
        Status = ArcRead(
            OemFileId,
            VaToPa((*(*LoaderBlock).NlsData).OemCodePageData),
            OemFileSize,
            &mut BytesRead,
        );
        if Status != ESUCCESS {
            WARN!("Error while reading '%s', Status: %u\n", (FileName.as_ptr()) as usize, (Status) as usize);
            'Quit: {
                if LangFileId != u32::MAX {
                    ArcClose(LangFileId);
                }
                if OemFileId != u32::MAX {
                    ArcClose(OemFileId);
                }
                if AnsiFileId != u32::MAX {
                    ArcClose(AnsiFileId);
                }

                if Status != ESUCCESS {
                    UiMessageBox(
                        b"Error reading NLS file %s\0".as_ptr() as PCSTR,
                        FileName.as_ptr(),
                    );
                }

                return if Status == ESUCCESS { 1u8 } else { 0u8 };
            }
        }
    }

    RtlStringCbPrintfA(
        FileName.as_mut_ptr(),
        core::mem::size_of_val(&FileName),
        b"%s%wZ\0".as_ptr() as PCSTR,
        DirectoryPath,
        LangFileName,
    );
    NtLdrOutputLoadMsg(FileName.as_ptr(), core::ptr::null());
    Status = ArcRead(
        LangFileId,
        VaToPa((*(*LoaderBlock).NlsData).UnicodeCodePageData),
        LangFileSize,
        &mut BytesRead,
    );
    if Status != ESUCCESS {
        WARN!("Error while reading '%s', Status: %u\n", (FileName.as_ptr()) as usize, (Status) as usize);
        'Quit: {
            if LangFileId != u32::MAX {
                ArcClose(LangFileId);
            }
            if OemFileId != u32::MAX {
                ArcClose(OemFileId);
            }
            if AnsiFileId != u32::MAX {
                ArcClose(AnsiFileId);
            }

            if Status != ESUCCESS {
                UiMessageBox(
                    b"Error reading NLS file %s\0".as_ptr() as PCSTR,
                    FileName.as_ptr(),
                );
            }

            return if Status == ESUCCESS { 1u8 } else { 0u8 };
        }
    }

    (*LoaderBlock).OemFontFile = VaToPa((*(*LoaderBlock).NlsData).UnicodeCodePageData);

    (*LoaderBlock).NlsData = PaToVa((*LoaderBlock).NlsData as *mut core::ffi::c_void) as *mut _;

    if LangFileId != u32::MAX {
        ArcClose(LangFileId);
    }
    if OemFileId != u32::MAX {
        ArcClose(OemFileId);
    }
    if AnsiFileId != u32::MAX {
        ArcClose(AnsiFileId);
    }

    if Status != ESUCCESS {
        UiMessageBox(
            b"Error reading NLS file %s\0".as_ptr() as PCSTR,
            FileName.as_ptr(),
        );
    }

    if Status == ESUCCESS { 1u8 } else { 0u8 }
}

unsafe fn WinLdrScanRegistry(
    BootDriverListHead: PLIST_ENTRY
) -> u8 {
    let mut Success: u8;

    // C puns BootFileSystem (PCSTR from FsGetServiceName) through the
    // PCWSTR parameter — preserve that behaviour with a cast.
    Success = CmpFindDrivers(SystemHive,
                             HKEY_TO_HCI(CurrentControlSetKey),
                             BootLoad,
                             BootFileSystem as *const u16,
                             BootDriverListHead);
    if Success == 0 {
        if Success == 0 {
            CmpFreeDriverList(SystemHive, BootDriverListHead);
        }
        return Success;
    }

    Success = CmpSortDriverList(SystemHive,
                                HKEY_TO_HCI(CurrentControlSetKey),
                                BootDriverListHead);
    if Success == 0 {
        if Success == 0 {
            CmpFreeDriverList(SystemHive, BootDriverListHead);
        }
        return Success;
    }

    Success = CmpResolveDriverDependencies(BootDriverListHead);
    if Success == 0 {
        if Success == 0 {
            CmpFreeDriverList(SystemHive, BootDriverListHead);
        }
        return Success;
    }

    if Success == 0 {
        CmpFreeDriverList(SystemHive, BootDriverListHead);
    }

    Success
}

#[no_mangle]
pub unsafe extern "C" fn WinLdrAddDriverToList(
    DriverListHead: PLIST_ENTRY,
    InsertAtHead: u8,
    DriverName: PCWSTR,
    ImagePath: PCWSTR,
    GroupName: PCWSTR,
    ErrorControl: u32,
    Tag: u32,
) -> u8 {
    let mut DriverNode: PBOOT_DRIVER_NODE = core::ptr::null_mut();
    let DriverEntry: PBOOT_DRIVER_LIST_ENTRY;
    let AlreadyInserted: u8;
    let mut PathLength: u16;
    let mut DriverNameU: _UNICODE_STRING = core::mem::zeroed();
    let mut RegistryPath: _UNICODE_STRING = core::mem::zeroed();
    let mut FilePath: _UNICODE_STRING = core::mem::zeroed();
    let mut RegistryString: _UNICODE_STRING = core::mem::zeroed();
    let mut GroupString: _UNICODE_STRING = core::mem::zeroed();

    RtlInitUnicodeString(&mut DriverNameU, DriverName);
    AlreadyInserted = CmpIsDriverInList(
        DriverListHead,
        &mut DriverNameU,
        &mut DriverNode,
    );
    if AlreadyInserted != 0 {
        DriverEntry = &mut (*DriverNode).ListEntry;
    } else {
        DriverNode = CmpAllocate(
            core::mem::size_of::<BOOT_DRIVER_NODE>() as _,
            0u8,
            0x20434D54u32,
        ) as PBOOT_DRIVER_NODE;
        if DriverNode.is_null() {
            return 0u8;
        }

        core::ptr::write_bytes(
            DriverNode as *mut u8,
            0,
            core::mem::size_of::<BOOT_DRIVER_NODE>(),
        );
        DriverEntry = &mut (*DriverNode).ListEntry;

        RtlInitEmptyUnicodeString(
            &mut (*DriverNode).Name,
            CmpAllocate(DriverNameU.Length as _, 0u8, 0x20434D54u32) as *mut u16,
            DriverNameU.Length,
        );
        if (*DriverNode).Name.Buffer.is_null() {
            return 'failure: {
                if !GroupString.Buffer.is_null() {
                    RtlFreeUnicodeString(&mut GroupString);
                }
                if !RegistryString.Buffer.is_null() {
                    CmpFree(RegistryString.Buffer as *mut core::ffi::c_void, RegistryString.MaximumLength as u32);
                }
                if !FilePath.Buffer.is_null() {
                    CmpFree(FilePath.Buffer as *mut core::ffi::c_void, FilePath.MaximumLength as u32);
                }

                if AlreadyInserted == 0 {
                    if !(*DriverEntry).RegistryPath.Buffer.is_null() {
                        CmpFree(
                            (*DriverEntry).RegistryPath.Buffer as *mut core::ffi::c_void,
                            (*DriverEntry).RegistryPath.MaximumLength as u32,
                        );
                    }
                    if !(*DriverEntry).FilePath.Buffer.is_null() {
                        CmpFree(
                            (*DriverEntry).FilePath.Buffer as *mut core::ffi::c_void,
                            (*DriverEntry).FilePath.MaximumLength as u32,
                        );
                    }
                    if !(*DriverNode).Name.Buffer.is_null() {
                        CmpFree(
                            (*DriverNode).Name.Buffer as *mut core::ffi::c_void,
                            (*DriverNode).Name.MaximumLength as u32,
                        );
                    }
                    CmpFree(DriverNode as *mut core::ffi::c_void, core::mem::size_of::<BOOT_DRIVER_NODE>() as u32);
                }

                0u8
            };
        }

        if !NT_SUCCESS(RtlAppendUnicodeStringToString(&mut (*DriverNode).Name, &mut DriverNameU)) {
            return 'failure: {
                if !GroupString.Buffer.is_null() {
                    RtlFreeUnicodeString(&mut GroupString);
                }
                if !RegistryString.Buffer.is_null() {
                    CmpFree(RegistryString.Buffer as *mut core::ffi::c_void, RegistryString.MaximumLength as u32);
                }
                if !FilePath.Buffer.is_null() {
                    CmpFree(FilePath.Buffer as *mut core::ffi::c_void, FilePath.MaximumLength as u32);
                }

                if AlreadyInserted == 0 {
                    if !(*DriverEntry).RegistryPath.Buffer.is_null() {
                        CmpFree(
                            (*DriverEntry).RegistryPath.Buffer as *mut core::ffi::c_void,
                            (*DriverEntry).RegistryPath.MaximumLength as u32,
                        );
                    }
                    if !(*DriverEntry).FilePath.Buffer.is_null() {
                        CmpFree(
                            (*DriverEntry).FilePath.Buffer as *mut core::ffi::c_void,
                            (*DriverEntry).FilePath.MaximumLength as u32,
                        );
                    }
                    if !(*DriverNode).Name.Buffer.is_null() {
                        CmpFree(
                            (*DriverNode).Name.Buffer as *mut core::ffi::c_void,
                            (*DriverNode).Name.MaximumLength as u32,
                        );
                    }
                    CmpFree(DriverNode as *mut core::ffi::c_void, core::mem::size_of::<BOOT_DRIVER_NODE>() as u32);
                }

                0u8
            };
        }
    }

    if !ImagePath.is_null() && *ImagePath != 0 {
        PathLength = (wcslen(ImagePath) * core::mem::size_of::<u16>()) as u16;
        RtlInitEmptyUnicodeString(
            &mut FilePath,
            CmpAllocate(PathLength as _, 0u8, 0x524E4C57u32) as *mut u16,
            PathLength,
        );
        if FilePath.Buffer.is_null() {
            return 'failure: {
                if !GroupString.Buffer.is_null() {
                    RtlFreeUnicodeString(&mut GroupString);
                }
                if !RegistryString.Buffer.is_null() {
                    CmpFree(RegistryString.Buffer as *mut core::ffi::c_void, RegistryString.MaximumLength as u32);
                }
                if !FilePath.Buffer.is_null() {
                    CmpFree(FilePath.Buffer as *mut core::ffi::c_void, FilePath.MaximumLength as u32);
                }

                if AlreadyInserted == 0 {
                    if !(*DriverEntry).RegistryPath.Buffer.is_null() {
                        CmpFree(
                            (*DriverEntry).RegistryPath.Buffer as *mut core::ffi::c_void,
                            (*DriverEntry).RegistryPath.MaximumLength as u32,
                        );
                    }
                    if !(*DriverEntry).FilePath.Buffer.is_null() {
                        CmpFree(
                            (*DriverEntry).FilePath.Buffer as *mut core::ffi::c_void,
                            (*DriverEntry).FilePath.MaximumLength as u32,
                        );
                    }
                    if !(*DriverNode).Name.Buffer.is_null() {
                        CmpFree(
                            (*DriverNode).Name.Buffer as *mut core::ffi::c_void,
                            (*DriverNode).Name.MaximumLength as u32,
                        );
                    }
                    CmpFree(DriverNode as *mut core::ffi::c_void, core::mem::size_of::<BOOT_DRIVER_NODE>() as u32);
                }

                0u8
            };
        }

        if !NT_SUCCESS(RtlAppendUnicodeToString(&mut FilePath, ImagePath)) {
            return 'failure: {
                if !GroupString.Buffer.is_null() {
                    RtlFreeUnicodeString(&mut GroupString);
                }
                if !RegistryString.Buffer.is_null() {
                    CmpFree(RegistryString.Buffer as *mut core::ffi::c_void, RegistryString.MaximumLength as u32);
                }
                if !FilePath.Buffer.is_null() {
                    CmpFree(FilePath.Buffer as *mut core::ffi::c_void, FilePath.MaximumLength as u32);
                }

                if AlreadyInserted == 0 {
                    if !(*DriverEntry).RegistryPath.Buffer.is_null() {
                        CmpFree(
                            (*DriverEntry).RegistryPath.Buffer as *mut core::ffi::c_void,
                            (*DriverEntry).RegistryPath.MaximumLength as u32,
                        );
                    }
                    if !(*DriverEntry).FilePath.Buffer.is_null() {
                        CmpFree(
                            (*DriverEntry).FilePath.Buffer as *mut core::ffi::c_void,
                            (*DriverEntry).FilePath.MaximumLength as u32,
                        );
                    }
                    if !(*DriverNode).Name.Buffer.is_null() {
                        CmpFree(
                            (*DriverNode).Name.Buffer as *mut core::ffi::c_void,
                            (*DriverNode).Name.MaximumLength as u32,
                        );
                    }
                    CmpFree(DriverNode as *mut core::ffi::c_void, core::mem::size_of::<BOOT_DRIVER_NODE>() as u32);
                }

                0u8
            };
        }
    } else {
        PathLength = (*DriverNode).Name.Length + (core::mem::size_of_val(&[
            's' as u16, 'y' as u16, 's' as u16, 't' as u16, 'e' as u16, 'm' as u16, '3' as u16, '2' as u16,
            '\\' as u16, 'd' as u16, 'r' as u16, 'i' as u16, 'v' as u16, 'e' as u16, 'r' as u16, 's' as u16,
            '\\' as u16, '.' as u16, 's' as u16, 'y' as u16, 's' as u16, 0u16
        ]) as u16);
        RtlInitEmptyUnicodeString(
            &mut FilePath,
            CmpAllocate(PathLength as _, 0u8, 0x524E4C57u32) as *mut u16,
            PathLength,
        );
        if FilePath.Buffer.is_null() {
            return 'failure: {
                if !GroupString.Buffer.is_null() {
                    RtlFreeUnicodeString(&mut GroupString);
                }
                if !RegistryString.Buffer.is_null() {
                    CmpFree(RegistryString.Buffer as *mut core::ffi::c_void, RegistryString.MaximumLength as u32);
                }
                if !FilePath.Buffer.is_null() {
                    CmpFree(FilePath.Buffer as *mut core::ffi::c_void, FilePath.MaximumLength as u32);
                }

                if AlreadyInserted == 0 {
                    if !(*DriverEntry).RegistryPath.Buffer.is_null() {
                        CmpFree(
                            (*DriverEntry).RegistryPath.Buffer as *mut core::ffi::c_void,
                            (*DriverEntry).RegistryPath.MaximumLength as u32,
                        );
                    }
                    if !(*DriverEntry).FilePath.Buffer.is_null() {
                        CmpFree(
                            (*DriverEntry).FilePath.Buffer as *mut core::ffi::c_void,
                            (*DriverEntry).FilePath.MaximumLength as u32,
                        );
                    }
                    if !(*DriverNode).Name.Buffer.is_null() {
                        CmpFree(
                            (*DriverNode).Name.Buffer as *mut core::ffi::c_void,
                            (*DriverNode).Name.MaximumLength as u32,
                        );
                    }
                    CmpFree(DriverNode as *mut core::ffi::c_void, core::mem::size_of::<BOOT_DRIVER_NODE>() as u32);
                }

                0u8
            };
        }

        if !NT_SUCCESS(RtlAppendUnicodeToString(&mut FilePath, ['s' as u16, 'y' as u16, 's' as u16, 't' as u16, 'e' as u16, 'm' as u16, '3' as u16, '2' as u16, '\\' as u16, 'd' as u16, 'r' as u16, 'i' as u16, 'v' as u16, 'e' as u16, 'r' as u16, 's' as u16, '\\' as u16, 0u16].as_ptr())) ||
           !NT_SUCCESS(RtlAppendUnicodeStringToString(&mut FilePath, &mut (*DriverNode).Name)) ||
           !NT_SUCCESS(RtlAppendUnicodeToString(&mut FilePath, ['.' as u16, 's' as u16, 'y' as u16, 's' as u16, 0u16].as_ptr()))
        {
            return 'failure: {
                if !GroupString.Buffer.is_null() {
                    RtlFreeUnicodeString(&mut GroupString);
                }
                if !RegistryString.Buffer.is_null() {
                    CmpFree(RegistryString.Buffer as *mut core::ffi::c_void, RegistryString.MaximumLength as u32);
                }
                if !FilePath.Buffer.is_null() {
                    CmpFree(FilePath.Buffer as *mut core::ffi::c_void, FilePath.MaximumLength as u32);
                }

                if AlreadyInserted == 0 {
                    if !(*DriverEntry).RegistryPath.Buffer.is_null() {
                        CmpFree(
                            (*DriverEntry).RegistryPath.Buffer as *mut core::ffi::c_void,
                            (*DriverEntry).RegistryPath.MaximumLength as u32,
                        );
                    }
                    if !(*DriverEntry).FilePath.Buffer.is_null() {
                        CmpFree(
                            (*DriverEntry).FilePath.Buffer as *mut core::ffi::c_void,
                            (*DriverEntry).FilePath.MaximumLength as u32,
                        );
                    }
                    if !(*DriverNode).Name.Buffer.is_null() {
                        CmpFree(
                            (*DriverNode).Name.Buffer as *mut core::ffi::c_void,
                            (*DriverNode).Name.MaximumLength as u32,
                        );
                    }
                    CmpFree(DriverNode as *mut core::ffi::c_void, core::mem::size_of::<BOOT_DRIVER_NODE>() as u32);
                }

                0u8
            };
        }
    }

    RtlInitUnicodeString(
        &mut RegistryPath,
        ['\\' as u16, 'R' as u16, 'e' as u16, 'g' as u16, 'i' as u16, 's' as u16, 't' as u16, 'r' as u16, 'y' as u16, '\\' as u16,
         'M' as u16, 'a' as u16, 'c' as u16, 'h' as u16, 'i' as u16, 'n' as u16, 'e' as u16, '\\' as u16,
         'S' as u16, 'y' as u16, 's' as u16, 't' as u16, 'e' as u16, 'm' as u16, '\\' as u16,
         'C' as u16, 'u' as u16, 'r' as u16, 'r' as u16, 'e' as u16, 'n' as u16, 't' as u16, 'C' as u16, 'o' as u16, 'n' as u16, 't' as u16, 'r' as u16, 'o' as u16, 'l' as u16, 'S' as u16, 'e' as u16, 't' as u16, '\\' as u16,
         'S' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, 's' as u16, '\\' as u16, 0u16].as_ptr(),
    );
    PathLength = RegistryPath.Length + (*DriverNode).Name.Length;
    RtlInitEmptyUnicodeString(
        &mut RegistryString,
        CmpAllocate(PathLength as _, 0u8, 0x524E4C57u32) as *mut u16,
        PathLength,
    );
    if RegistryString.Buffer.is_null() {
        return 'failure: {
            if !GroupString.Buffer.is_null() {
                RtlFreeUnicodeString(&mut GroupString);
            }
            if !RegistryString.Buffer.is_null() {
                CmpFree(RegistryString.Buffer as *mut core::ffi::c_void, RegistryString.MaximumLength as u32);
            }
            if !FilePath.Buffer.is_null() {
                CmpFree(FilePath.Buffer as *mut core::ffi::c_void, FilePath.MaximumLength as u32);
            }

            if AlreadyInserted == 0 {
                if !(*DriverEntry).RegistryPath.Buffer.is_null() {
                    CmpFree(
                        (*DriverEntry).RegistryPath.Buffer as *mut core::ffi::c_void,
                        (*DriverEntry).RegistryPath.MaximumLength as u32,
                    );
                }
                if !(*DriverEntry).FilePath.Buffer.is_null() {
                    CmpFree(
                        (*DriverEntry).FilePath.Buffer as *mut core::ffi::c_void,
                        (*DriverEntry).FilePath.MaximumLength as u32,
                    );
                }
                if !(*DriverNode).Name.Buffer.is_null() {
                    CmpFree(
                        (*DriverNode).Name.Buffer as *mut core::ffi::c_void,
                        (*DriverNode).Name.MaximumLength as u32,
                    );
                }
                CmpFree(DriverNode as *mut core::ffi::c_void, core::mem::size_of::<BOOT_DRIVER_NODE>() as u32);
            }

            0u8
        };
    }

    if !NT_SUCCESS(RtlAppendUnicodeStringToString(&mut RegistryString, &mut RegistryPath)) ||
       !NT_SUCCESS(RtlAppendUnicodeStringToString(&mut RegistryString, &mut (*DriverNode).Name))
    {
        return 'failure: {
            if !GroupString.Buffer.is_null() {
                RtlFreeUnicodeString(&mut GroupString);
            }
            if !RegistryString.Buffer.is_null() {
                CmpFree(RegistryString.Buffer as *mut core::ffi::c_void, RegistryString.MaximumLength as u32);
            }
            if !FilePath.Buffer.is_null() {
                CmpFree(FilePath.Buffer as *mut core::ffi::c_void, FilePath.MaximumLength as u32);
            }

            if AlreadyInserted == 0 {
                if !(*DriverEntry).RegistryPath.Buffer.is_null() {
                    CmpFree(
                        (*DriverEntry).RegistryPath.Buffer as *mut core::ffi::c_void,
                        (*DriverEntry).RegistryPath.MaximumLength as u32,
                    );
                }
                if !(*DriverEntry).FilePath.Buffer.is_null() {
                    CmpFree(
                        (*DriverEntry).FilePath.Buffer as *mut core::ffi::c_void,
                        (*DriverEntry).FilePath.MaximumLength as u32,
                    );
                }
                if !(*DriverNode).Name.Buffer.is_null() {
                    CmpFree(
                        (*DriverNode).Name.Buffer as *mut core::ffi::c_void,
                        (*DriverNode).Name.MaximumLength as u32,
                    );
                }
                CmpFree(DriverNode as *mut core::ffi::c_void, core::mem::size_of::<BOOT_DRIVER_NODE>() as u32);
            }

            0u8
        };
    }

    if !GroupName.is_null() && *GroupName != 0 {
        RtlCreateUnicodeString(&mut GroupString, GroupName);
        if GroupString.Buffer.is_null() {
            return 'failure: {
                if !GroupString.Buffer.is_null() {
                    RtlFreeUnicodeString(&mut GroupString);
                }
                if !RegistryString.Buffer.is_null() {
                    CmpFree(RegistryString.Buffer as *mut core::ffi::c_void, RegistryString.MaximumLength as u32);
                }
                if !FilePath.Buffer.is_null() {
                    CmpFree(FilePath.Buffer as *mut core::ffi::c_void, FilePath.MaximumLength as u32);
                }

                if AlreadyInserted == 0 {
                    if !(*DriverEntry).RegistryPath.Buffer.is_null() {
                        CmpFree(
                            (*DriverEntry).RegistryPath.Buffer as *mut core::ffi::c_void,
                            (*DriverEntry).RegistryPath.MaximumLength as u32,
                        );
                    }
                    if !(*DriverEntry).FilePath.Buffer.is_null() {
                        CmpFree(
                            (*DriverEntry).FilePath.Buffer as *mut core::ffi::c_void,
                            (*DriverEntry).FilePath.MaximumLength as u32,
                        );
                    }
                    if !(*DriverNode).Name.Buffer.is_null() {
                        CmpFree(
                            (*DriverNode).Name.Buffer as *mut core::ffi::c_void,
                            (*DriverNode).Name.MaximumLength as u32,
                        );
                    }
                    CmpFree(DriverNode as *mut core::ffi::c_void, core::mem::size_of::<BOOT_DRIVER_NODE>() as u32);
                }

                0u8
            };
        }
    } else {
        RtlInitEmptyUnicodeString(&mut GroupString, core::ptr::null_mut(), 0);
    }

    if !(*DriverEntry).FilePath.Buffer.is_null() {
        CmpFree(
            (*DriverEntry).FilePath.Buffer as *mut core::ffi::c_void,
            (*DriverEntry).FilePath.MaximumLength as u32,
        );
    }
    (*DriverEntry).FilePath = FilePath;
    FilePath.Buffer = core::ptr::null_mut();

    if !(*DriverEntry).RegistryPath.Buffer.is_null() {
        CmpFree(
            (*DriverEntry).RegistryPath.Buffer as *mut core::ffi::c_void,
            (*DriverEntry).RegistryPath.MaximumLength as u32,
        );
    }
    (*DriverEntry).RegistryPath = RegistryString;
    RegistryString.Buffer = core::ptr::null_mut();

    if !(*DriverNode).Group.Buffer.is_null() {
        let LoaderBlock: PLOADER_PARAMETER_BLOCK =
            if !WinLdrSystemBlock.is_null() {
                core::ptr::addr_of_mut!((*WinLdrSystemBlock).LoaderBlock)
            } else {
                core::ptr::null_mut()
            };

        if LoaderBlock.is_null() || (*LoaderBlock).RegistryBase.is_null() || (*LoaderBlock).RegistryLength == 0 ||
            (((*DriverNode).Group.Buffer as usize) <
                (VaToPa((*LoaderBlock).RegistryBase) as usize)) ||
            (((*DriverNode).Group.Buffer as usize) >=
                (VaToPa((*LoaderBlock).RegistryBase) as usize) + (*LoaderBlock).RegistryLength as usize)
        {
            RtlFreeUnicodeString(&mut (*DriverNode).Group);
        }
    }
    (*DriverNode).Group = GroupString;
    GroupString.Buffer = core::ptr::null_mut();

    (*DriverNode).ErrorControl = ErrorControl;
    (*DriverNode).Tag = Tag;

    if AlreadyInserted == 0 {
        if InsertAtHead != 0 {
            InsertHeadList(DriverListHead, &mut (*DriverEntry).Link);
        } else {
            InsertTailList(DriverListHead, &mut (*DriverEntry).Link);
        }
    }

    1u8
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         EFI Windows Loader
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            boot/freeldr/freeldr/windows/wlregistry.c
//  * PURPOSE:         Registry support functions
//  * PROGRAMMERS:     Aleksey Bragin (aleksey@reactos.org)
//  */
//
// /* INCLUDES ***************************************************************/
//
// #include <freeldr.h>
// #include "winldr.h"
// #include "registry.h"
// #include <internal/cmboot.h>
//
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(WINDOWS);
//
// // Used to re-mark NLS pages after heap cleanup in WinLdrSetupMemoryLayout()
// ULONG TotalNLSSize = 0;
// PVOID NlsDataBase = NULL;
//
// static BOOLEAN
// WinLdrGetNLSNames(
//     _In_ HKEY ControlSet,
//     _Inout_ PUNICODE_STRING AnsiFileName,
//     _Inout_ PUNICODE_STRING OemFileName,
//     _Inout_ PUNICODE_STRING LangFileName, // CaseTable
//     _Inout_ PUNICODE_STRING OemHalFileName);
//
// static BOOLEAN
// WinLdrScanRegistry(
//     IN OUT PLIST_ENTRY BootDriverListHead);
//
// typedef enum _BAD_HIVE_REASON
// {
//     GoodHive = 1,
//     CorruptHive,
//     NoHive,
//     NoHiveAlloc
// } BAD_HIVE_REASON, *PBAD_HIVE_REASON;
//
// /* FUNCTIONS **************************************************************/
//
// static BOOLEAN
// WinLdrLoadSystemHive(
//     _Inout_ PLOADER_PARAMETER_BLOCK LoaderBlock,
//     _In_ PCSTR DirectoryPath,
//     _In_ PCSTR HiveName,
//     _Out_ PBAD_HIVE_REASON Reason)
// {
//     ULONG FileId;
//     CHAR FullHiveName[MAX_PATH];
//     ARC_STATUS Status;
//     FILEINFORMATION FileInfo;
//     ULONG HiveFileSize;
//     PVOID HiveDataPhysical;
//     PVOID HiveDataVirtual;
//     ULONG BytesRead;
//
//     /* Do not setup any bad reason for now */
//     *Reason = GoodHive;
//
//     /* Concatenate path and filename to get the full name */
//     RtlStringCbCopyA(FullHiveName, sizeof(FullHiveName), DirectoryPath);
//     RtlStringCbCatA(FullHiveName, sizeof(FullHiveName), HiveName);
//
//     NtLdrOutputLoadMsg(FullHiveName, NULL);
//     Status = ArcOpen(FullHiveName, OpenReadOnly, &FileId);
//     if (Status != ESUCCESS)
//     {
//         WARN("Error while opening '%s', Status: %u\n", (FullHiveName) as usize, (Status) as usize);
//         *Reason = NoHive;
//         return FALSE;
//     }
//
//     /* Get the file length */
//     Status = ArcGetFileInformation(FileId, &FileInfo);
//     if (Status != ESUCCESS)
//     {
//         WARN("Hive file has 0 size!\n");
//         *Reason = CorruptHive;
//         ArcClose(FileId);
//         return FALSE;
//     }
//     HiveFileSize = FileInfo.EndingAddress.LowPart;
//
//     /* Round up the size to page boundary and alloc memory */
//     HiveDataPhysical = MmAllocateMemoryWithType(
//         MM_SIZE_TO_PAGES(HiveFileSize + MM_PAGE_SIZE - 1) << MM_PAGE_SHIFT,
//         LoaderRegistryData);
//
//     if (HiveDataPhysical == NULL)
//     {
//         WARN("Could not alloc memory for hive!\n");
//         *Reason = NoHiveAlloc;
//         ArcClose(FileId);
//         return FALSE;
//     }
//
//     /* Convert address to virtual */
//     HiveDataVirtual = PaToVa(HiveDataPhysical);
//
//     /* Fill LoaderBlock's entries */
//     LoaderBlock->RegistryLength = HiveFileSize;
//     LoaderBlock->RegistryBase = HiveDataVirtual;
//
//     /* Finally read from file to the memory */
//     Status = ArcRead(FileId, HiveDataPhysical, HiveFileSize, &BytesRead);
//     if (Status != ESUCCESS)
//     {
//         WARN("Error while reading '%s', Status: %u\n", (FullHiveName) as usize, (Status) as usize);
//         *Reason = CorruptHive;
//         ArcClose(FileId);
//         return FALSE;
//     }
//
//     // FIXME: HACK: Get the boot filesystem driver name now...
//     BootFileSystem = FsGetServiceName(FileId);
//
//     ArcClose(FileId);
//     return TRUE;
// }
//
// BOOLEAN
// WinLdrInitSystemHive(
//     IN OUT PLOADER_PARAMETER_BLOCK LoaderBlock,
//     IN PCSTR SystemRoot,
//     IN BOOLEAN Setup)
// {
//     CHAR SearchPath[1024];
//     PVOID ChunkBase;
//     PCSTR HiveName;
//     BOOLEAN Success;
//     BAD_HIVE_REASON Reason;
//
//     /* Load the corresponding text-mode setup system hive or the standard hive */
//     if (Setup)
//     {
//         RtlStringCbCopyA(SearchPath, sizeof(SearchPath), SystemRoot);
//         HiveName = "SETUPREG.HIV";
//     }
//     else
//     {
//         RtlStringCbCopyA(SearchPath, sizeof(SearchPath), SystemRoot);
//         RtlStringCbCatA(SearchPath, sizeof(SearchPath), "system32\\config\\");
//         HiveName = "SYSTEM";
//     }
//
//     TRACE("WinLdrInitSystemHive: loading hive %s%s\n", (SearchPath) as usize, (HiveName) as usize);
//     Success = WinLdrLoadSystemHive(LoaderBlock, SearchPath, HiveName, &Reason);
//     if (!Success)
//     {
//         /* Check whether the SYSTEM hive does not exist or is too corrupt to be read */
//         if (Reason == CorruptHive || Reason == NoHive)
//         {
//             /* Try loading the alternate hive, the main hive should be recovered later */
//             goto LoadAlternateHive;
//         }
//
//         /* We are failing for other reason, bail out */
//         UiMessageBox("Could not load %s hive!", HiveName);
//         return FALSE;
//     }
//
//     /* Import what was loaded */
//     Success = RegImportBinaryHive(VaToPa(LoaderBlock->RegistryBase), LoaderBlock->RegistryLength, SearchPath, FALSE);
//     if (!Success)
//     {
//         /*
//          * Importing of the SYSTEM hive failed. The scenarios that would
//          * have made this possible are the following:
//          *
//          * 1. The primary hive is corrupt beyond repair (such as when
//          *    core FS structures are total toast);
//          *
//          * 2. Repairing the hive could with a LOG could not recover it
//          *    to the fullest. This is the case when the hive and LOG have
//          *    diverged too much, or are mismatched, or the containing healthy
//          *    data in the LOG was not marked as dirty that could be copied
//          *    into the primary hive;
//          *
//          * 3. LOG is bad (e.g. corrupt dirty vector);
//          *
//          * 4. LOG does not physically exist on the backing storage.
//          *
//          * 5. SYSTEM hive does not physically exist or it is a 0 bytes file
//          *    (the latter case still counts as corruption).
//          *
//          * With the hope to boot the system, load the mirror counterpart
//          * of the main hive, the alternate. The kernel should be able to recover
//          * the main hive later on as soon as it starts writing to it.
//          */
// LoadAlternateHive:
//         HiveName = "SYSTEM.ALT";
//         Success = WinLdrLoadSystemHive(LoaderBlock, SearchPath, HiveName, &Reason);
//         if (!Success)
//         {
//             UiMessageBox("Could not load %s hive!", HiveName);
//             return FALSE;
//         }
//
//         /* Retry importing it again */
//         Success = RegImportBinaryHive(VaToPa(LoaderBlock->RegistryBase), LoaderBlock->RegistryLength, SearchPath, TRUE);
//         if (!Success)
//         {
//             UiMessageBox("Importing binary hive failed!");
//             return FALSE;
//         }
//
//         /*
//          * Acknowledge the kernel we recovered the SYSTEM hive
//          * on our side by loading the alternate variant of the hive.
//          */
//         WARN("SYSTEM hive does not exist or is corrupt and SYSTEM.ALT has been loaded!\n");
//         ChunkBase = VaToPa(LoaderBlock->RegistryBase);
//         ((PHBASE_BLOCK)ChunkBase)->BootRecover = HBOOT_BOOT_RECOVERED_BY_ALTERNATE_HIVE;
//     }
//
//     // FIXME: Load SYSTEM.SAV if GUI setup installation is still in progress
//
//     /* Initialize the 'CurrentControlSet' link */
//     if (!RegInitCurrentControlSet(FALSE))
//     {
//         UiMessageBox("Initializing CurrentControlSet link failed!");
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// BOOLEAN WinLdrScanSystemHive(IN OUT PLOADER_PARAMETER_BLOCK LoaderBlock,
//                              IN PCSTR SystemRoot)
// {
//     BOOLEAN Success;
//     DECLARE_UNICODE_STRING_SIZE(AnsiFileName, MAX_PATH);
//     DECLARE_UNICODE_STRING_SIZE(OemFileName, MAX_PATH);
//     DECLARE_UNICODE_STRING_SIZE(LangFileName, MAX_PATH); // CaseTable
//     DECLARE_UNICODE_STRING_SIZE(OemHalFileName, MAX_PATH);
//     CHAR SearchPath[1024];
//
//     /* Scan registry and prepare boot drivers list */
//     Success = WinLdrScanRegistry(&LoaderBlock->BootDriverListHead);
//     if (!Success)
//     {
//         UiMessageBox("Failed to load boot drivers!");
//         return FALSE;
//     }
//
//     /* Get names of NLS files */
//     Success = WinLdrGetNLSNames(CurrentControlSetKey,
//                                 &AnsiFileName,
//                                 &OemFileName,
//                                 &LangFileName,
//                                 &OemHalFileName);
//     if (!Success)
//     {
//         UiMessageBox("Getting NLS names from registry failed!");
//         return FALSE;
//     }
//
//     TRACE("NLS data: '%wZ' '%wZ' '%wZ' '%wZ'\n", (//           &AnsiFileName) as usize, (&OemFileName) as usize, (&LangFileName) as usize, (&OemHalFileName) as usize);
//
//     /* Load NLS data */
//     RtlStringCbCopyA(SearchPath, sizeof(SearchPath), SystemRoot);
//     RtlStringCbCatA(SearchPath, sizeof(SearchPath), "system32\\");
//     Success = WinLdrLoadNLSData(LoaderBlock,
//                                 SearchPath,
//                                 &AnsiFileName,
//                                 &OemFileName,
//                                 &LangFileName,
//                                 &OemHalFileName);
//     TRACE("NLS data loading %s\n", (Success ? "successful" : "failed") as usize);
//
//     return TRUE;
// }
//
//
// /* PRIVATE FUNCTIONS ******************************************************/
//
// // Queries registry for those three file names
// static BOOLEAN
// WinLdrGetNLSNames(
//     _In_ HKEY ControlSet,
//     _Inout_ PUNICODE_STRING AnsiFileName,
//     _Inout_ PUNICODE_STRING OemFileName,
//     _Inout_ PUNICODE_STRING LangFileName, // CaseTable
//     _Inout_ PUNICODE_STRING OemHalFileName)
// {
//     LONG rc;
//     HKEY hKey;
//     ULONG BufferSize;
//     WCHAR szIdBuffer[80];
//
//     /* Open the CodePage key */
//     rc = RegOpenKey(ControlSet, L"Control\\NLS\\CodePage", &hKey);
//     if (rc != ERROR_SUCCESS)
//     {
//         //TRACE("Couldn't open CodePage registry key\n");
//         return FALSE;
//     }
//
//     /* Get ANSI codepage file */
//     BufferSize = sizeof(szIdBuffer);
//     rc = RegQueryValue(hKey, L"ACP", NULL, (PUCHAR)szIdBuffer, &BufferSize);
//     if (rc != ERROR_SUCCESS)
//     {
//         //TRACE("Couldn't get ACP NLS setting\n");
//         goto Quit;
//     }
//
//     BufferSize = AnsiFileName->MaximumLength;
//     rc = RegQueryValue(hKey, szIdBuffer, NULL,
//                        (PUCHAR)AnsiFileName->Buffer, &BufferSize);
//     if (rc != ERROR_SUCCESS)
//     {
//         //TRACE("ACP NLS Setting exists, but isn't readable\n");
//         //goto Quit;
//         AnsiFileName->Length = 0;
//         RtlAppendUnicodeToString(AnsiFileName, L"c_1252.nls"); // HACK: ReactOS bug CORE-6105
//     }
//     else
//     {
//         AnsiFileName->Length = (USHORT)BufferSize - sizeof(UNICODE_NULL);
//     }
//
//     /* Get OEM codepage file */
//     BufferSize = sizeof(szIdBuffer);
//     rc = RegQueryValue(hKey, L"OEMCP", NULL, (PUCHAR)szIdBuffer, &BufferSize);
//     if (rc != ERROR_SUCCESS)
//     {
//         //TRACE("Couldn't get OEMCP NLS setting\n");
//         goto Quit;
//     }
//
//     BufferSize = OemFileName->MaximumLength;
//     rc = RegQueryValue(hKey, szIdBuffer, NULL,
//                        (PUCHAR)OemFileName->Buffer, &BufferSize);
//     if (rc != ERROR_SUCCESS)
//     {
//         //TRACE("OEMCP NLS setting exists, but isn't readable\n");
//         //goto Quit;
//         OemFileName->Length = 0;
//         RtlAppendUnicodeToString(OemFileName, L"c_437.nls"); // HACK: ReactOS bug CORE-6105
//     }
//     else
//     {
//         OemFileName->Length = (USHORT)BufferSize - sizeof(UNICODE_NULL);
//     }
//
//     /* Get OEM HAL font file */
//     BufferSize = OemHalFileName->MaximumLength;
//     rc = RegQueryValue(hKey, L"OEMHAL", NULL,
//                        (PUCHAR)OemHalFileName->Buffer, &BufferSize);
//     if (rc != ERROR_SUCCESS)
//     {
//         //TRACE("Couldn't get OEMHAL NLS setting\n");
//         //goto Quit;
//         RtlInitEmptyUnicodeString(OemHalFileName, NULL, 0);
//     }
//     else
//     {
//         OemHalFileName->Length = (USHORT)BufferSize - sizeof(UNICODE_NULL);
//     }
//
//     RegCloseKey(hKey);
//
//     /* Open the Language key */
//     rc = RegOpenKey(ControlSet, L"Control\\NLS\\Language", &hKey);
//     if (rc != ERROR_SUCCESS)
//     {
//         //TRACE("Couldn't open Language registry key\n");
//         return FALSE;
//     }
//
//     /* Get the Unicode case table file */
//     BufferSize = sizeof(szIdBuffer);
//     rc = RegQueryValue(hKey, L"Default", NULL, (PUCHAR)szIdBuffer, &BufferSize);
//     if (rc != ERROR_SUCCESS)
//     {
//         //TRACE("Couldn't get Language Default setting\n");
//         goto Quit;
//     }
//
//     BufferSize = LangFileName->MaximumLength;
//     rc = RegQueryValue(hKey, szIdBuffer, NULL,
//                        (PUCHAR)LangFileName->Buffer, &BufferSize);
//     if (rc != ERROR_SUCCESS)
//     {
//         //TRACE("Language Default setting exists, but isn't readable\n");
//         //goto Quit;
//         LangFileName->Length = 0;
//         RtlAppendUnicodeToString(LangFileName, L"l_intl.nls");
//     }
//     else
//     {
//         LangFileName->Length = (USHORT)BufferSize - sizeof(UNICODE_NULL);
//     }
//
// Quit:
//     RegCloseKey(hKey);
//     return (rc == ERROR_SUCCESS);
// }
//
// BOOLEAN
// WinLdrLoadNLSData(
//     _Inout_ PLOADER_PARAMETER_BLOCK LoaderBlock,
//     _In_ PCSTR DirectoryPath,
//     _In_ PCUNICODE_STRING AnsiFileName,
//     _In_ PCUNICODE_STRING OemFileName,
//     _In_ PCUNICODE_STRING LangFileName, // CaseTable
//     _In_ PCUNICODE_STRING OemHalFileName)
// {
//     ARC_STATUS Status;
//     FILEINFORMATION FileInfo;
//     ULONG AnsiFileId = -1, OemFileId = -1, LangFileId = -1;
//     ULONG AnsiFileSize, OemFileSize, LangFileSize;
//     ULONG TotalSize;
//     ULONG BytesRead;
//     PVOID NlsVirtual;
//     BOOLEAN AnsiEqualsOem = FALSE;
//     CHAR FileName[MAX_PATH];
//
//     /* There may be a case, where OEM and ANSI pages coincide */
//     if (RtlCompareUnicodeString(AnsiFileName, OemFileName, TRUE) == 0)
//         AnsiEqualsOem = TRUE;
//
//     /* Open file with ANSI and store its size */
//     RtlStringCbPrintfA(FileName, sizeof(FileName), "%s%wZ",
//                        DirectoryPath, AnsiFileName);
//     Status = ArcOpen(FileName, OpenReadOnly, &AnsiFileId);
//     if (Status != ESUCCESS)
//     {
//         WARN("Error while opening '%s', Status: %u\n", (FileName) as usize, (Status) as usize);
//         goto Quit;
//     }
//
//     Status = ArcGetFileInformation(AnsiFileId, &FileInfo);
//     if (Status != ESUCCESS)
//         goto Quit;
//     AnsiFileSize = FileInfo.EndingAddress.LowPart;
//     TRACE("AnsiFileSize: %d\n", (AnsiFileSize) as usize);
//
//     /* Open OEM file and store its length */
//     if (AnsiEqualsOem)
//     {
//         OemFileSize = 0;
//     }
//     else
//     {
//         RtlStringCbPrintfA(FileName, sizeof(FileName), "%s%wZ",
//                            DirectoryPath, OemFileName);
//         Status = ArcOpen(FileName, OpenReadOnly, &OemFileId);
//         if (Status != ESUCCESS)
//         {
//             WARN("Error while opening '%s', Status: %u\n", (FileName) as usize, (Status) as usize);
//             goto Quit;
//         }
//
//         Status = ArcGetFileInformation(OemFileId, &FileInfo);
//         if (Status != ESUCCESS)
//             goto Quit;
//         OemFileSize = FileInfo.EndingAddress.LowPart;
//     }
//     TRACE("OemFileSize: %d\n", (OemFileSize) as usize);
//
//     /* Finally open the language codepage file and store its length */
//     RtlStringCbPrintfA(FileName, sizeof(FileName), "%s%wZ",
//                        DirectoryPath, LangFileName);
//     Status = ArcOpen(FileName, OpenReadOnly, &LangFileId);
//     if (Status != ESUCCESS)
//     {
//         WARN("Error while opening '%s', Status: %u\n", (FileName) as usize, (Status) as usize);
//         goto Quit;
//     }
//
//     Status = ArcGetFileInformation(LangFileId, &FileInfo);
//     if (Status != ESUCCESS)
//         goto Quit;
//     LangFileSize = FileInfo.EndingAddress.LowPart;
//     TRACE("LangFileSize: %d\n", (LangFileSize) as usize);
//
//     //
//     // TODO: The OEMHAL file.
//     //
//
//     /* Sum up all three length, having in mind that every one of them
//        must start at a page boundary => thus round up each file to a page */
//     TotalSize = MM_SIZE_TO_PAGES(AnsiFileSize) +
//                 MM_SIZE_TO_PAGES(OemFileSize)  +
//                 MM_SIZE_TO_PAGES(LangFileSize);
//
//     /* Store it for later marking the pages as NlsData type */
//     TotalNLSSize = TotalSize;
//
//     NlsDataBase = MmAllocateMemoryWithType(TotalSize*MM_PAGE_SIZE, LoaderNlsData);
//     if (NlsDataBase == NULL)
//         goto Quit;
//
//     NlsVirtual = PaToVa(NlsDataBase);
//     LoaderBlock->NlsData->AnsiCodePageData = NlsVirtual;
//
//     LoaderBlock->NlsData->OemCodePageData =
//         (PVOID)((ULONG_PTR)NlsVirtual +
//         (MM_SIZE_TO_PAGES(AnsiFileSize) << MM_PAGE_SHIFT));
//
//     LoaderBlock->NlsData->UnicodeCodePageData =
//         (PVOID)((ULONG_PTR)NlsVirtual +
//         (MM_SIZE_TO_PAGES(AnsiFileSize) << MM_PAGE_SHIFT) +
//         (MM_SIZE_TO_PAGES(OemFileSize)  << MM_PAGE_SHIFT));
//
//     /* ANSI and OEM data are the same - just set pointers to the same area */
//     if (AnsiEqualsOem)
//         LoaderBlock->NlsData->OemCodePageData = LoaderBlock->NlsData->AnsiCodePageData;
//
//     /* Now actually read the data into memory, starting with the ANSI file */
//     RtlStringCbPrintfA(FileName, sizeof(FileName), "%s%wZ",
//                        DirectoryPath, AnsiFileName);
//     NtLdrOutputLoadMsg(FileName, NULL);
//     Status = ArcRead(AnsiFileId,
//                      VaToPa(LoaderBlock->NlsData->AnsiCodePageData),
//                      AnsiFileSize, &BytesRead);
//     if (Status != ESUCCESS)
//     {
//         WARN("Error while reading '%s', Status: %u\n", (FileName) as usize, (Status) as usize);
//         goto Quit;
//     }
//
//     /* OEM now, if it isn't the same as the ANSI one */
//     if (!AnsiEqualsOem)
//     {
//         RtlStringCbPrintfA(FileName, sizeof(FileName), "%s%wZ",
//                            DirectoryPath, OemFileName);
//         NtLdrOutputLoadMsg(FileName, NULL);
//         Status = ArcRead(OemFileId,
//                          VaToPa(LoaderBlock->NlsData->OemCodePageData),
//                          OemFileSize, &BytesRead);
//         if (Status != ESUCCESS)
//         {
//             WARN("Error while reading '%s', Status: %u\n", (FileName) as usize, (Status) as usize);
//             goto Quit;
//         }
//     }
//
//     /* Finally the language file */
//     RtlStringCbPrintfA(FileName, sizeof(FileName), "%s%wZ",
//                        DirectoryPath, LangFileName);
//     NtLdrOutputLoadMsg(FileName, NULL);
//     Status = ArcRead(LangFileId,
//                      VaToPa(LoaderBlock->NlsData->UnicodeCodePageData),
//                      LangFileSize, &BytesRead);
//     if (Status != ESUCCESS)
//     {
//         WARN("Error while reading '%s', Status: %u\n", (FileName) as usize, (Status) as usize);
//         goto Quit;
//     }
//
//     //
//     // THIS IS a HACK and should be replaced by actually loading the OEMHAL file!
//     //
//     LoaderBlock->OemFontFile = VaToPa(LoaderBlock->NlsData->UnicodeCodePageData);
//
//     /* Convert NlsTables address to VA */
//     LoaderBlock->NlsData = PaToVa(LoaderBlock->NlsData);
//
// Quit:
//     if (LangFileId != -1)
//         ArcClose(LangFileId);
//     if (OemFileId != -1)
//         ArcClose(OemFileId);
//     if (AnsiFileId != -1)
//         ArcClose(AnsiFileId);
//
//     if (Status != ESUCCESS)
//         UiMessageBox("Error reading NLS file %s", FileName);
//
//     return (Status == ESUCCESS);
// }
//
// static BOOLEAN
// WinLdrScanRegistry(
//     IN OUT PLIST_ENTRY BootDriverListHead)
// {
//     BOOLEAN Success;
//
//     /* Find all boot drivers */
//     Success = CmpFindDrivers(SystemHive,
//                              HKEY_TO_HCI(CurrentControlSetKey),
//                              BootLoad,
//                              BootFileSystem,
//                              BootDriverListHead);
//     if (!Success)
//         goto Quit;
//
//     /* Sort by group/tag */
//     Success = CmpSortDriverList(SystemHive,
//                                 HKEY_TO_HCI(CurrentControlSetKey),
//                                 BootDriverListHead);
//     if (!Success)
//         goto Quit;
//
//     /* Remove circular dependencies (cycles) and sort */
//     Success = CmpResolveDriverDependencies(BootDriverListHead);
//     if (!Success)
//         goto Quit;
//
// Quit:
//     /* In case of failure, free the boot driver list */
//     if (!Success)
//         CmpFreeDriverList(SystemHive, BootDriverListHead);
//
//     return Success;
// }
//
// /**
//  * @brief
//  * Inserts the specified driver entry into the driver list, or updates
//  * an existing entry with new ImagePath, ErrorControl, Group and Tag values.
//  *
//  * @param[in,out]   DriverListHead
//  * The driver list where to insert the driver entry.
//  *
//  * @param[in]   InsertAtHead
//  * Whether to insert the driver at the head (TRUE) or at the tail (FALSE)
//  * of the driver list.
//  *
//  * @param[in]   DriverName
//  * The driver's name.
//  *
//  * @param[in]   ImagePath
//  * Optional path the the driver's image. If none is specified,
//  * a default path is constructed out of the driver's name.
//  *
//  * @param[in]   GroupName
//  * Optional driver group name.
//  *
//  * @param[in]   ErrorControl
//  * @param[in]   Tag
//  * The ErrorControl and group Tag values for the driver.
//  *
//  * @return
//  * TRUE if the driver has been inserted into the list or updated, FALSE if not.
//  **/
// BOOLEAN
// WinLdrAddDriverToList(
//     _Inout_ PLIST_ENTRY DriverListHead,
//     _In_ BOOLEAN InsertAtHead,
//     _In_ PCWSTR DriverName,
//     _In_opt_ PCWSTR ImagePath,
//     _In_opt_ PCWSTR GroupName,
//     _In_ ULONG ErrorControl,
//     _In_ ULONG Tag)
// {
//     PBOOT_DRIVER_NODE DriverNode;
//     PBOOT_DRIVER_LIST_ENTRY DriverEntry;
//     BOOLEAN AlreadyInserted;
//     USHORT PathLength;
//     UNICODE_STRING DriverNameU;
//     UNICODE_STRING RegistryPath;
//     UNICODE_STRING FilePath = {0};
//     UNICODE_STRING RegistryString = {0};
//     UNICODE_STRING GroupString = {0};
//
//     /* Check whether the driver is already in the list */
//     RtlInitUnicodeString(&DriverNameU, DriverName);
//     AlreadyInserted = CmpIsDriverInList(DriverListHead,
//                                         &DriverNameU,
//                                         &DriverNode);
//     if (AlreadyInserted)
//     {
//         /* If so, we have obtained its node */
//         ASSERT(DriverNode);
//         DriverEntry = &DriverNode->ListEntry;
//     }
//     else
//     {
//         /* Allocate a driver node and initialize it */
//         DriverNode = CmpAllocate(sizeof(BOOT_DRIVER_NODE), FALSE, TAG_CM);
//         if (!DriverNode)
//             return FALSE;
//
//         RtlZeroMemory(DriverNode, sizeof(BOOT_DRIVER_NODE));
//         DriverEntry = &DriverNode->ListEntry;
//
//         /* Driver Name */
//         RtlInitEmptyUnicodeString(&DriverNode->Name,
//                                   CmpAllocate(DriverNameU.Length, FALSE, TAG_CM),
//                                   DriverNameU.Length);
//         if (!DriverNode->Name.Buffer)
//             goto Failure;
//
//         if (!NT_SUCCESS(RtlAppendUnicodeStringToString(&DriverNode->Name, &DriverNameU)))
//             goto Failure;
//     }
//
//     /* Check whether we have a valid ImagePath. If not, we need
//      * to build it like "System32\\Drivers\\blah.sys" */
//     if (ImagePath && *ImagePath)
//     {
//         /* Just copy ImagePath to the corresponding field in the structure */
//         PathLength = (USHORT)(wcslen(ImagePath)) * sizeof(WCHAR);
//         RtlInitEmptyUnicodeString(&FilePath,
//                                   CmpAllocate(PathLength, FALSE, TAG_WLDR_NAME),
//                                   PathLength);
//         if (!FilePath.Buffer)
//             goto Failure;
//
//         if (!NT_SUCCESS(RtlAppendUnicodeToString(&FilePath, ImagePath)))
//             goto Failure;
//     }
//     else
//     {
//         /* We have to construct ImagePath ourselves */
//         PathLength = DriverNode->Name.Length + sizeof(L"system32\\drivers\\.sys");
//         RtlInitEmptyUnicodeString(&FilePath,
//                                   CmpAllocate(PathLength, FALSE, TAG_WLDR_NAME),
//                                   PathLength);
//         if (!FilePath.Buffer)
//             goto Failure;
//
//         if (!NT_SUCCESS(RtlAppendUnicodeToString(&FilePath, L"system32\\drivers\\"))  ||
//             !NT_SUCCESS(RtlAppendUnicodeStringToString(&FilePath, &DriverNode->Name)) ||
//             !NT_SUCCESS(RtlAppendUnicodeToString(&FilePath, L".sys")))
//         {
//             goto Failure;
//         }
//     }
//
//     /* Registry path */
//     RtlInitUnicodeString(&RegistryPath,
//                          L"\\Registry\\Machine\\System\\CurrentControlSet\\Services\\");
//     PathLength = RegistryPath.Length + DriverNode->Name.Length;
//     RtlInitEmptyUnicodeString(&RegistryString,
//                               CmpAllocate(PathLength, FALSE, TAG_WLDR_NAME),
//                               PathLength);
//     if (!RegistryString.Buffer)
//         goto Failure;
//
//     if (!NT_SUCCESS(RtlAppendUnicodeStringToString(&RegistryString, &RegistryPath)) ||
//         !NT_SUCCESS(RtlAppendUnicodeStringToString(&RegistryString, &DriverNode->Name)))
//     {
//         goto Failure;
//     }
//
//     /* Group */
//     if (GroupName && *GroupName)
//     {
//         /*
//          * NOTE: Here we can use our own allocator as we alone maintain the
//          * group string. This is different from the other allocated strings,
//          * where we instead need to use the same (hive) allocator as the
//          * one used by CmpAddDriverToList(), for interoperability purposes.
//          */
//         RtlCreateUnicodeString(&GroupString, GroupName);
//         if (!GroupString.Buffer)
//             goto Failure;
//     }
//     else
//     {
//         RtlInitEmptyUnicodeString(&GroupString, NULL, 0);
//     }
//
//     /* Set or replace the driver node's file path */
//     if (DriverEntry->FilePath.Buffer)
//     {
//         CmpFree(DriverEntry->FilePath.Buffer,
//                 DriverEntry->FilePath.MaximumLength);
//     }
//     DriverEntry->FilePath = FilePath;
//     FilePath.Buffer = NULL;
//
//     /* Set or replace the driver node's registry path */
//     if (DriverEntry->RegistryPath.Buffer)
//     {
//         CmpFree(DriverEntry->RegistryPath.Buffer,
//                 DriverEntry->RegistryPath.MaximumLength);
//     }
//     DriverEntry->RegistryPath = RegistryString;
//     RegistryString.Buffer = NULL;
//
//     /* Set or replace the driver node's group */
//     if (DriverNode->Group.Buffer)
//     {
//         /*
//          * If the buffer is inside the registry hive's memory, this means that
//          * it has been set by CmpAddDriverToList() to point to some data within
//          * the hive; thus we should not free the buffer but just replace it.
//          * Otherwise, this is a buffer previously allocated by ourselves, that
//          * we can free.
//          *
//          * NOTE: This function does not have an explicit LoaderBlock input
//          * parameter pointer, since it does not need it, except for this
//          * very place. So instead, use the global WinLdrSystemBlock pointer.
//          */
//         PLOADER_PARAMETER_BLOCK LoaderBlock =
//             (WinLdrSystemBlock ? &WinLdrSystemBlock->LoaderBlock : NULL);
//
//         if (!LoaderBlock || !LoaderBlock->RegistryBase || !LoaderBlock->RegistryLength ||
//             ((ULONG_PTR)DriverNode->Group.Buffer <
//                 (ULONG_PTR)VaToPa(LoaderBlock->RegistryBase)) ||
//             ((ULONG_PTR)DriverNode->Group.Buffer >=
//                 (ULONG_PTR)VaToPa(LoaderBlock->RegistryBase) + LoaderBlock->RegistryLength))
//         {
//             RtlFreeUnicodeString(&DriverNode->Group);
//         }
//     }
//     DriverNode->Group = GroupString;
//     GroupString.Buffer = NULL;
//
//     /* ErrorControl and Tag */
//     DriverNode->ErrorControl = ErrorControl;
//     DriverNode->Tag = Tag;
//
//     /* Insert the entry into the list if it does not exist there already */
//     if (!AlreadyInserted)
//     {
//         if (InsertAtHead)
//             InsertHeadList(DriverListHead, &DriverEntry->Link);
//         else
//             InsertTailList(DriverListHead, &DriverEntry->Link);
//     }
//
//     return TRUE;
//
// Failure:
//     if (GroupString.Buffer)
//         RtlFreeUnicodeString(&GroupString);
//     if (RegistryString.Buffer)
//         CmpFree(RegistryString.Buffer, RegistryString.MaximumLength);
//     if (FilePath.Buffer)
//         CmpFree(FilePath.Buffer, FilePath.MaximumLength);
//
//     /* If it does not exist in the list already, free the allocated
//      * driver node, otherwise keep the original one in place. */
//     if (!AlreadyInserted)
//     {
//         if (DriverEntry->RegistryPath.Buffer)
//         {
//             CmpFree(DriverEntry->RegistryPath.Buffer,
//                     DriverEntry->RegistryPath.MaximumLength);
//         }
//         if (DriverEntry->FilePath.Buffer)
//         {
//             CmpFree(DriverEntry->FilePath.Buffer,
//                     DriverEntry->FilePath.MaximumLength);
//         }
//         if (DriverNode->Name.Buffer)
//         {
//             CmpFree(DriverNode->Name.Buffer,
//                     DriverNode->Name.MaximumLength);
//         }
//         CmpFree(DriverNode, sizeof(BOOT_DRIVER_NODE));
//     }
//
//     return FALSE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
