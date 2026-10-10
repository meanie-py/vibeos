#[allow(unused_imports)] use crate::{ERR, TRACE, WARN};
#[allow(dead_code, non_upper_case_globals)] const DbgDefaultChannel: u32 = crate::frldr::DPRINT_REGISTRY; // registry.c: DBG_DEFAULT_CHANNEL(REGISTRY)
use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{ARC_STATUS, CMHIVE, CM_CHECK_REGISTRY_STATUS, CmCheckRegistry, CmpCompressedNameSize, CmpCopyCompressedName, CmpFindControlSet, CmpFindSubKeyByName, CmpFindSubKeyByNumber, CmpFindValueByName, CmpValueToData, FILEINFORMATION, HCELL_INDEX, LARGE_INTEGER, PCELL_DATA, PCMHIVE, PCM_KEY_NODE, PCM_KEY_VALUE, PCSTR, PHBASE_BLOCK, PHHIVE, PUNICODE_STRING, RtlEqualUnicodeString, RtlInitUnicodeString, _HHIVE, _UNICODE_STRING};
// freeldr_ddk_imports dropped (not in ddk): Type
use core::ffi::c_void;

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

pub use ntkrnl_sys::ddk::_ARC_CODES_ESUCCESS as ESUCCESS;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderRegistryData as LoaderRegistryData;
pub use ntkrnl_sys::ddk::_OPENMODE_OpenReadOnly as OpenReadOnly;
pub use ntkrnl_sys::ddk::_SEEKMODE_SeekAbsolute as SeekAbsolute;

// registry.h: typedef HANDLE HKEY, *PHKEY;
pub type HKEY = ntkrnl_sys::ddk::HANDLE;
pub type PHKEY = *mut HKEY;
pub use ntkrnl_sys::ddk::{PCUNICODE_STRING, PWCH};

// ntbasedef.h
pub const MAXULONG: u32 = 0xffffffff;
pub const UNICODE_NULL: u16 = 0;
// cmdata.h
pub const KEY_COMP_NAME: u16 = 0x0020;
pub const VALUE_COMP_NAME: u16 = 0x0001;
// cmlib.h: CM_CHECK_REGISTRY_GOOD == 0;
//          CM_CHECK_REGISTRY_SUCCESS(s) == ((ULONG)s == GOOD)
pub const CM_CHECK_REGISTRY_GOOD: u32 = 0;
#[inline]
unsafe fn CM_CHECK_REGISTRY_SUCCESS(StatusCode: CM_CHECK_REGISTRY_STATUS) -> bool {
    (StatusCode as ULONG) == CM_CHECK_REGISTRY_GOOD
}

// cmlib hivedata.h
pub const HCELL_NIL: HCELL_INDEX = MAXULONG;
pub const HINIT_FLAT: u32 = 4;
pub const HFILE_TYPE_PRIMARY: u32 = 0;
pub const HFILE_TYPE_ALTERNATE: u32 = 3;
pub const HBOOT_NO_BOOT_RECOVER: u32 = 0;
pub const HBOOT_BOOT_RECOVERED_BY_HIVE_LOG: u32 = 1;
// cmlib.h
pub const CM_CHECK_REGISTRY_BOOTLOADER_PURGE_VOLATILES: u32 = 0x4;
pub const CM_CHECK_REGISTRY_VALIDATE_HIVE: u32 = 0x8;
// winerror.h
pub const ERROR_SUCCESS: i32 = 0;
pub const ERROR_FILE_NOT_FOUND: i32 = 2;
pub const ERROR_PATH_NOT_FOUND: i32 = 3;
pub const ERROR_NO_MORE_ITEMS: i32 = 259;

// registry.c file-scope statics. SystemHive/CurrentControlSetKey have
// external linkage (registry.h externs them; wlregistry.rs refs them) →
// this TU is the #[no_mangle] definer. CmSystemHive/SystemRootCell are
// `static` (internal linkage) → private.
#[no_mangle]
pub static mut SystemHive: PHHIVE = core::ptr::null_mut();
#[no_mangle]
pub static mut CurrentControlSetKey: HKEY = core::ptr::null_mut();
static mut CmSystemHive: PCMHIVE = core::ptr::null_mut();
static mut SystemRootCell: HCELL_INDEX = 0;

// registry.c macros — restored as inline helpers (NOT extern fns).
#[inline]
unsafe fn HCI_TO_HKEY(CellIndex: HCELL_INDEX) -> HKEY {
    CellIndex as ULONG_PTR as HKEY
}
#[inline]
unsafe fn HKEY_TO_HCI(hKey: HKEY) -> HCELL_INDEX {
    hKey as ULONG_PTR as HCELL_INDEX
}
#[inline]
unsafe fn GET_HHIVE(CmHive: PCMHIVE) -> *mut _HHIVE {
    core::ptr::addr_of_mut!((*CmHive).Hive)
}
#[inline]
unsafe fn GET_HHIVE_FROM_HKEY(_hKey: HKEY) -> PHHIVE {
    GET_HHIVE(CmSystemHive)
}
#[inline]
unsafe fn GET_CM_KEY_NODE(hHive: PHHIVE, hKey: HKEY) -> PCM_KEY_NODE {
    HvGetCell(hHive, HKEY_TO_HCI(hKey)) as PCM_KEY_NODE
}
#[inline]
unsafe fn GET_HBASE_BLOCK(ChunkBase: *mut core::ffi::c_void) -> PHBASE_BLOCK {
    ChunkBase as PHBASE_BLOCK
}
// cmlib.h: HvGetCell / HvReleaseCell macros (call through HHIVE routines).
#[inline]
unsafe fn HvGetCell(Hive: PHHIVE, Cell: HCELL_INDEX) -> PCELL_DATA {
    ((*Hive).GetCellRoutine.unwrap())(Hive, Cell) as PCELL_DATA
}
#[inline]
unsafe fn HvReleaseCell(Hive: PHHIVE, Cell: HCELL_INDEX) {
    if let Some(rel) = (*Hive).ReleaseCellRoutine {
        rel(Hive, Cell);
    }
}

extern "C" {
    fn FrLdrHeapAlloc(Size: usize, Tag: u32) -> *mut core::ffi::c_void;
    fn FrLdrHeapFree(ptr: PVOID, flags: ULONG);
    fn FrLdrTempAlloc(Size: usize, Tag: u32) -> *mut core::ffi::c_void;
    fn FrLdrTempFree(ptr: *mut core::ffi::c_void, Tag: u32);
    fn ArcClose(FileId: u32) -> ARC_STATUS;
    fn ArcGetFileInformation(FileId: u32, FileInformation: *mut FILEINFORMATION) -> ARC_STATUS;
    fn ArcOpen(Path: PCSTR, OpenMode: u32, FileId: *mut u32) -> ARC_STATUS;
    fn ArcRead(FileId: u32, Buffer: *mut core::ffi::c_void, Length: u32, Count: *mut u32) -> ARC_STATUS;
    fn ArcSeek(FileId: u32, Position: *mut LARGE_INTEGER, SeekMode: u32) -> ARC_STATUS;
    fn MmAllocateMemoryWithType(MemorySize: usize, MemoryType: u32) -> *mut core::ffi::c_void;
    fn PaToVa(Pa: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
    fn RtlStringCbCatA(pszDest: *mut u8, cbDest: usize, pszSrc: PCSTR) -> i32;
    fn VaToPa(va: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
}
// cmlib.h: CMAPI == NTAPI (stdcall on i386) → extern "system".
// Hvp* live in cmlib (libblcmlib.a exports `_HvpVerifyHiveHeader@8` /
// `_HvpHiveHeaderChecksum@4`) — must be declared stdcall, not cdecl,
// or i386 link fails with undefined `_HvpVerifyHiveHeader`.
extern "system" {
    fn HvpVerifyHiveHeader(base_block: PHBASE_BLOCK, file_type: u32) -> u8;
    fn HvpHiveHeaderChecksum(block: PHBASE_BLOCK) -> u32;
    fn HvInitialize(
        RegistryHive: PHHIVE,
        OperationType: ULONG,
        HiveFlags: ULONG,
        FileType: ULONG,
        HiveData: PVOID,
        Allocate: ntkrnl_sys::ddk::PALLOCATE_ROUTINE,
        Free: ntkrnl_sys::ddk::PFREE_ROUTINE,
        FileSetSize: ntkrnl_sys::ddk::PFILE_SET_SIZE_ROUTINE,
        FileWrite: ntkrnl_sys::ddk::PFILE_WRITE_ROUTINE,
        FileRead: ntkrnl_sys::ddk::PFILE_READ_ROUTINE,
        FileFlush: ntkrnl_sys::ddk::PFILE_FLUSH_ROUTINE,
        Cluster: ULONG,
        FileName: PCUNICODE_STRING,
    ) -> NTSTATUS;
}

#[inline]
#[no_mangle]
pub unsafe extern "C" fn MM_SIZE_TO_PAGES(a: u32) -> u32 {
    (a >> MM_PAGE_SHIFT) + (if a & MM_PAGE_MASK != 0 { 1 } else { 0 })
}

// ntdef.h RTL_CONSTANT_STRING(s): build a UNICODE_STRING over a NUL-
// terminated wide literal. C uses sizeof(literal); here the literal is a
// `*const u16` so we measure with wcslen.
#[inline]
unsafe fn RTL_CONSTANT_STRING(s: *const u16) -> _UNICODE_STRING {
    let chars = wcslen(s);
    let bytes = (chars * core::mem::size_of::<u16>()) as USHORT;
    _UNICODE_STRING {
        Length: bytes,
        MaximumLength: bytes + core::mem::size_of::<u16>() as USHORT,
        Buffer: s as PWCH,
    }
}

// NTAPI — signature + ABI MUST be token-identical to ddk::PALLOCATE_ROUTINE
// / PFREE_ROUTINE so `Some(CmpAllocate)` coerces to the HvInitialize fn-ptr
// parameter. Those ddk callback types are `extern "stdcall"` on i386 and
// `extern "C"` on amd64/arm64. Rust does NOT unify `extern "system" fn`
// with `extern "C" fn` even where ABI-identical, so the declared ABI must
// match the ddk type *literally* per-arch (was hardcoded "stdcall", wrong
// on amd64 where the ddk callback is cdecl).
#[no_mangle]
pub unsafe extern "system" fn CmpAllocate(
    Size: ntkrnl_sys::ddk::SIZE_T,
    Paged: ntkrnl_sys::ddk::BOOLEAN,
    Tag: ntkrnl_sys::ddk::ULONG,
) -> ntkrnl_sys::ddk::PVOID {
    let _ = Paged;
    FrLdrHeapAlloc(Size as usize, Tag)
}

#[no_mangle]
pub unsafe extern "system" fn CmpFree(
    Ptr: ntkrnl_sys::ddk::PVOID,
    Quota: ntkrnl_sys::ddk::ULONG,
) {
    let _ = Quota;
    FrLdrHeapFree(Ptr, 0);
}

unsafe fn RegInitializeHive(
    CmHive: PCMHIVE,
    ChunkBase: PVOID,
    LoadAlternate: BOOLEAN,
) -> BOOLEAN {
    let Status: NTSTATUS;
    let CmStatusCode: CM_CHECK_REGISTRY_STATUS;

    Status = HvInitialize(
        GET_HHIVE(CmHive),
        HINIT_FLAT,
        0,
        if LoadAlternate != 0 { HFILE_TYPE_ALTERNATE } else { HFILE_TYPE_PRIMARY },
        ChunkBase,
        Some(CmpAllocate),
        Some(CmpFree),
        None,
        None,
        None,
        None,
        1,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        ERR!("Failed to initialize the flat hive (Status 0x%lx)\n", (Status) as usize);
        return 0u8;
    }

    CmStatusCode = CmCheckRegistry(CmHive, CM_CHECK_REGISTRY_BOOTLOADER_PURGE_VOLATILES | CM_CHECK_REGISTRY_VALIDATE_HIVE);
    if !CM_CHECK_REGISTRY_SUCCESS(CmStatusCode) {
        ERR!("CmCheckRegistry detected problems with the loaded flat hive (check code %lu)\n", (CmStatusCode) as usize);
        return 0u8;
    }

    return 1u8;
}

unsafe fn RegLoadHiveLog(
    DirectoryPath: PCSTR,
    LogFileOffset: u32,
    LogName: PCSTR,
    LogData: *mut *mut core::ffi::c_void,
) -> u8 {
    let mut Status: ARC_STATUS;
    let mut LogId: u32 = 0;
    let mut LogPath: [u8; 260] = [0; 260];
    let LogFileSize: u32;
    let mut FileInfo: FILEINFORMATION = core::mem::zeroed();
    let mut Position: LARGE_INTEGER = core::mem::zeroed();
    let mut BytesRead: u32 = 0;
    let LogDataVirtual: *mut core::ffi::c_void;
    let LogDataPhysical: *mut core::ffi::c_void;

    RtlStringCbCopyA(LogPath.as_mut_ptr() as *mut i8 , core::mem::size_of_val(&LogPath), DirectoryPath);
    RtlStringCbCatA(LogPath.as_mut_ptr(), core::mem::size_of_val(&LogPath), LogName);

    Status = ArcOpen(LogPath.as_ptr() as PCSTR, OpenReadOnly, &mut LogId);
    if Status != ESUCCESS {
        ERR!("Failed to open %s (ARC code %lu)\n", (LogName) as usize, (Status) as usize);
        return 0u8;
    }

    Status = ArcGetFileInformation(LogId, &mut FileInfo);
    if Status != ESUCCESS {
        ERR!("Failed to get file information from %s (ARC code %lu)\n", (LogName) as usize, (Status) as usize);
        ArcClose(LogId);
        return 0u8;
    }

    LogFileSize = FileInfo.EndingAddress.u.LowPart;
    if LogFileSize == 0 {
        ERR!("LogFileSize is 0, %s is corrupt\n", (LogName) as usize);
        ArcClose(LogId);
        return 0u8;
    }

    LogDataPhysical = MmAllocateMemoryWithType(
        (MM_SIZE_TO_PAGES(LogFileSize + 4096u32 - 1) << 12) as usize,
        LoaderRegistryData,
    );
    if LogDataPhysical.is_null() {
        ERR!("Failed to allocate memory for log data\n");
        ArcClose(LogId);
        return 0u8;
    }

    LogDataVirtual = PaToVa(LogDataPhysical);

    Position.QuadPart = LogFileOffset as i64;
    Status = ArcSeek(LogId, &mut Position, SeekAbsolute);
    if Status != ESUCCESS {
        ERR!("Failed to seek at %s (ARC code %lu)\n", (LogName) as usize, (Status) as usize);
        ArcClose(LogId);
        return 0u8;
    }

    Status = ArcRead(LogId, LogDataPhysical, LogFileSize, &mut BytesRead);
    if Status != ESUCCESS {
        ERR!("Failed to read %s (ARC code %lu)\n", (LogName) as usize, (Status) as usize);
        ArcClose(LogId);
        return 0u8;
    }

    *LogData = LogDataVirtual;
    ArcClose(LogId);
    return 1u8;
}

unsafe fn RegRecoverHeaderHive(
    ChunkBase: *mut core::ffi::c_void,
    DirectoryPath: PCSTR,
    LogName: PCSTR,
) -> u8 {
    let Success: u8;
    let mut FullLogFileName: [u8; 260] = [0; 260];
    let mut LogData: *mut core::ffi::c_void = core::ptr::null_mut();
    let HiveBaseBlock: PHBASE_BLOCK;
    let LogBaseBlock: PHBASE_BLOCK;

    const HFILE_TYPE_LOG: u32 = 2;
    const HFILE_TYPE_PRIMARY: u32 = 0;
    const HSECTOR_SIZE: u32 = 512;

    RtlStringCbCopyA(FullLogFileName.as_mut_ptr() as *mut i8 , core::mem::size_of_val(&FullLogFileName), LogName);
    RtlStringCbCatA(FullLogFileName.as_mut_ptr(), core::mem::size_of_val(&FullLogFileName), b".LOG\0".as_ptr() as *const i8 );
    Success = RegLoadHiveLog(DirectoryPath, 0, FullLogFileName.as_ptr() as *const i8 , &mut LogData);
    if Success == 0 {
        ERR!("Failed to read the hive log\n");
        return 0u8;
    }

    LogData = VaToPa(LogData);
    LogBaseBlock = GET_HBASE_BLOCK(LogData);
    if HvpVerifyHiveHeader(LogBaseBlock, HFILE_TYPE_LOG) == 0 {
        ERR!("The hive log has corrupt base block\n");
        return 0u8;
    }

    HiveBaseBlock = GET_HBASE_BLOCK(ChunkBase);
    WARN!("Recovering the hive base block...\n");
    RtlCopyMemory(
        HiveBaseBlock as *mut core::ffi::c_void,
        LogBaseBlock as *const core::ffi::c_void,
        ((*LogBaseBlock).Cluster * HSECTOR_SIZE) as usize,
    );
    (*HiveBaseBlock).Type = HFILE_TYPE_PRIMARY;
    return 1u8;
}

unsafe fn RegRecoverDataHive(
    ChunkBase: *mut core::ffi::c_void,
    DirectoryPath: PCSTR,
    LogName: PCSTR,
) -> u8 {
    let Success: u8;
    let StorageLength: u32;
    let mut BlockIndex: u32;
    let mut LogIndex: u32;
    let _BlockPtr: *mut u8;
    let _BlockDest: *mut u8;
    let mut FullLogFileName: [u8; 260] = [0; 260];
    let mut LogData: *mut core::ffi::c_void = core::ptr::null_mut();
    let LogDataPhysical: *mut u8;
    let HiveBaseBlock: PHBASE_BLOCK;

    RtlStringCbCopyA(FullLogFileName.as_mut_ptr() as *mut i8 , 260, LogName);
    RtlStringCbCatA(FullLogFileName.as_mut_ptr(), 260, b".LOG\0".as_ptr() as *const i8 );
    Success = RegLoadHiveLog(DirectoryPath, 4096, FullLogFileName.as_ptr() as *const i8 , &mut LogData);
    if Success == 0 {
        ERR!("Failed to read the hive log\n");
        return 0u8;
    }

    LogDataPhysical = VaToPa(LogData) as *mut u8;
    if *(LogDataPhysical as *const u32) != 0x64697274 {
        ERR!("The hive log dirty signature could not be found\n");
        return 0u8;
    }

    LogIndex = 0;
    BlockIndex = 0;
    HiveBaseBlock = GET_HBASE_BLOCK(ChunkBase);
    StorageLength = (*HiveBaseBlock).Length / 4096;
    while BlockIndex < StorageLength {
        if *LogDataPhysical.offset((BlockIndex + 4) as isize) != 1 {
            BlockIndex += 1;
            continue;
        }

        let BlockPtr = (LogDataPhysical as usize + 2 * 512 + (LogIndex * 4096) as usize) as *mut u8;
        let BlockDest = (ChunkBase as usize + ((BlockIndex + 1) * 4096) as usize) as *mut u8;
        RtlCopyMemory(BlockDest as *mut core::ffi::c_void, BlockPtr as *const core::ffi::c_void, 4096);

        LogIndex += 1;
        BlockIndex += 1;
    }

    (*HiveBaseBlock).Sequence2 = (*HiveBaseBlock).Sequence1;
    (*HiveBaseBlock).CheckSum = HvpHiveHeaderChecksum(HiveBaseBlock);
    1u8
}

#[no_mangle]
pub unsafe extern "C" fn RegImportBinaryHive(
    ChunkBase: PVOID,
    ChunkSize: ULONG,
    SearchPath: PCSTR,
    LoadAlternate: BOOLEAN,
) -> BOOLEAN {
    let mut Success: BOOLEAN;
    let KeyNode: PCM_KEY_NODE;

    TRACE!("RegImportBinaryHive(%p, 0x%lx)\n", (ChunkBase) as usize, (ChunkSize) as usize);

    (*(ChunkBase as PHBASE_BLOCK)).BootRecover = HBOOT_NO_BOOT_RECOVER;

    CmSystemHive = FrLdrTempAlloc(core::mem::size_of::<CMHIVE>(), 0x65766948u32) as PCMHIVE;
    Success = RegInitializeHive(CmSystemHive, ChunkBase, LoadAlternate);
    if Success == 0 {
        FrLdrTempFree(CmSystemHive as PVOID, 0x65766948u32);
        CmSystemHive = core::ptr::null_mut();

        if RegRecoverHeaderHive(ChunkBase, SearchPath, b"SYSTEM\0".as_ptr() as PCSTR) == 0 {
            ERR!("Failed to recover the hive header block\n");
            return 0u8;
        }

        if RegRecoverDataHive(ChunkBase, SearchPath, b"SYSTEM\0".as_ptr() as PCSTR) == 0 {
            ERR!("Failed to recover the hive data\n");
            return 0u8;
        }

        CmSystemHive = FrLdrTempAlloc(core::mem::size_of::<CMHIVE>(), 0x65766948u32) as PCMHIVE;
        Success = RegInitializeHive(CmSystemHive, ChunkBase, LoadAlternate);
        if Success == 0 {
            ERR!("Corrupted hive (despite recovery) %p\n", (ChunkBase) as usize);
            FrLdrTempFree(CmSystemHive as PVOID, 0x65766948u32);
            return 0u8;
        }

        (*(ChunkBase as PHBASE_BLOCK)).BootRecover = HBOOT_BOOT_RECOVERED_BY_HIVE_LOG;
    }

    SystemHive = GET_HHIVE(CmSystemHive);
    SystemRootCell = (*(*SystemHive).BaseBlock).RootCell;

    KeyNode = HvGetCell(SystemHive, SystemRootCell) as PCM_KEY_NODE;
    HvReleaseCell(SystemHive, SystemRootCell);

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn RegInitCurrentControlSet(LastKnownGood: u8) -> u8 {
    let mut ControlSetName: _UNICODE_STRING = core::mem::zeroed();
    let ControlCell: HCELL_INDEX;
    let KeyNode: PCM_KEY_NODE;
    let mut AutoSelect: u8 = 0;

    TRACE!("RegInitCurrentControlSet\n");

    RtlInitUnicodeString(
        &mut ControlSetName,
        if LastKnownGood != 0 {
            w!("LastKnownGood").as_ptr() as *const u16
        } else {
            w!("Default").as_ptr() as *const u16
        },
    );

    ControlCell = CmpFindControlSet(
        SystemHive,
        SystemRootCell,
        &mut ControlSetName,
        &mut AutoSelect,
    );
    if ControlCell == HCELL_NIL {
        ERR!("CmpFindControlSet('%wZ') failed\n", (core::ptr::addr_of!(ControlSetName)) as usize);
        return 0u8;
    }

    CurrentControlSetKey = HCI_TO_HKEY(ControlCell);

    KeyNode = HvGetCell(SystemHive, ControlCell) as PCM_KEY_NODE;
    HvReleaseCell(SystemHive, ControlCell);

    1u8
}

unsafe fn GetNextPathElement(
    NextElement: PUNICODE_STRING,
    RemainingPath: PUNICODE_STRING
) -> u8 {
    if (*RemainingPath).Length < core::mem::size_of::<u16>() as u16 {
        return 0u8;
    }

    (*NextElement).Buffer = (*RemainingPath).Buffer;

    while ((*RemainingPath).Length >= core::mem::size_of::<u16>() as u16) &&
          (*(*RemainingPath).Buffer.offset(0) != '\\' as u16) {
        (*RemainingPath).Buffer = (*RemainingPath).Buffer.offset(1);
        (*RemainingPath).Length -= core::mem::size_of::<u16>() as u16;
    }

    (*NextElement).Length = ((*RemainingPath).Buffer.offset_from((*NextElement).Buffer) as u16) * core::mem::size_of::<u16>() as u16;
    (*NextElement).MaximumLength = (*NextElement).Length;

    if (*RemainingPath).Length >= core::mem::size_of::<u16>() as u16 {
        (*RemainingPath).Buffer = (*RemainingPath).Buffer.offset(1);
        (*RemainingPath).Length -= core::mem::size_of::<u16>() as u16;
    }

    return 1u8;
}

unsafe fn RegEnumKey(
    Key: HKEY,
    Index: ULONG,
    Name: PWCHAR,
    NameSize: PULONG,
    SubKey: PHKEY,
) -> LONG {
    let Hive = GET_HHIVE_FROM_HKEY(Key);
    let KeyNode: PCM_KEY_NODE;
    let SubKeyNode: PCM_KEY_NODE;
    let CellIndex: HCELL_INDEX;
    let NameLength: USHORT;

    TRACE!("RegEnumKey(%p, %lu, %p, %p->%u)\n", (Key) as usize, (Index) as usize, (Name) as usize, (NameSize) as usize, (if !NameSize.is_null() { *NameSize } else { 0 }) as usize);

    KeyNode = GET_CM_KEY_NODE(Hive, Key);

    CellIndex = CmpFindSubKeyByNumber(Hive, KeyNode, Index);
    if CellIndex == HCELL_NIL {
        let kn_namelen = core::ptr::addr_of!((*KeyNode).NameLength).read_unaligned();
        let kn_name = core::ptr::addr_of!((*KeyNode).Name) as *const u16;
        TRACE!("RegEnumKey index out of bounds (%d) in key (%.*s)\n", (Index) as usize, (kn_namelen) as usize, (kn_name) as usize);
        HvReleaseCell(Hive, HKEY_TO_HCI(Key));
        return ERROR_NO_MORE_ITEMS;
    }
    HvReleaseCell(Hive, HKEY_TO_HCI(Key));

    SubKeyNode = HvGetCell(Hive, CellIndex) as PCM_KEY_NODE;

    if (*SubKeyNode).Flags & KEY_COMP_NAME != 0 {
        let sk_namelen = core::ptr::addr_of!((*SubKeyNode).NameLength).read_unaligned();
        let sk_name = core::ptr::addr_of!((*SubKeyNode).Name) as *const u16;
        NameLength = CmpCompressedNameSize(sk_name as _, sk_namelen as u32);

        CmpCopyCompressedName(
            Name,
            *NameSize,
            sk_name as _,
            sk_namelen as u32,
        );
    } else {
        let skn_namelen = core::ptr::addr_of!((*SubKeyNode).NameLength).read_unaligned();
        let skn_name = core::ptr::addr_of!((*SubKeyNode).Name) as *const u8;
        NameLength = skn_namelen;

        core::ptr::copy_nonoverlapping(
            skn_name,
            Name as *mut u8,
            min(*NameSize, skn_namelen as u32 ) as usize,
        );
    }

    if *NameSize >= NameLength as ULONG + core::mem::size_of::<WCHAR>() as ULONG {
        *Name.offset((NameLength as usize / core::mem::size_of::<WCHAR>()) as isize) = UNICODE_NULL;
    }

    *NameSize = NameLength as ULONG + core::mem::size_of::<WCHAR>() as ULONG;

    HvReleaseCell(Hive, CellIndex);

    if !SubKey.is_null() {
        *SubKey = HCI_TO_HKEY(CellIndex);
    }

    TRACE!("RegEnumKey done -> %u, '%.*S'\n", (*NameSize) as usize, (*NameSize) as usize, (Name) as usize);
    return ERROR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "C" fn RegOpenKey(
    ParentKey: HKEY,
    KeyName: PCWSTR,
    Key: PHKEY
) -> i32 {
    let mut RemainingPath: _UNICODE_STRING = core::mem::zeroed();
    let mut SubKeyName: _UNICODE_STRING = core::mem::zeroed();
    let CurrentControlSet: _UNICODE_STRING = RTL_CONSTANT_STRING(w!("CurrentControlSet").as_ptr() as *const u16);
    let Hive: PHHIVE = if ParentKey != core::ptr::null_mut() {
        GET_HHIVE_FROM_HKEY(ParentKey)
    } else {
        GET_HHIVE(CmSystemHive)
    };
    let mut KeyNode: PCM_KEY_NODE;
    let mut CellIndex: HCELL_INDEX;

    TRACE!("RegOpenKey(%p, '%S', %p)\n", (ParentKey) as usize, (KeyName) as usize, (Key) as usize);

    RtlInitUnicodeString(core::ptr::addr_of_mut!(RemainingPath), KeyName);

    if ParentKey == core::ptr::null_mut() {
        let mut SubKeyName1: _UNICODE_STRING = core::mem::zeroed();
        let mut SubKeyName2: _UNICODE_STRING = core::mem::zeroed();
        let mut SubKeyName3: _UNICODE_STRING = core::mem::zeroed();
        let RegistryPath: _UNICODE_STRING = RTL_CONSTANT_STRING(w!("Registry").as_ptr() as *const u16);
        let MachinePath: _UNICODE_STRING = RTL_CONSTANT_STRING(w!("MACHINE").as_ptr() as *const u16);
        let SystemPath: _UNICODE_STRING = RTL_CONSTANT_STRING(w!("SYSTEM").as_ptr() as *const u16);

        TRACE!("RegOpenKey: absolute path\n");

        if (RemainingPath.Length < core::mem::size_of::<u16>() as u16) ||
            *RemainingPath.Buffer.offset(0) != '\\' as u16
        {
            ERR!("RegOpenKey: invalid path '%S' (%wZ)\n", (KeyName) as usize, (core::ptr::addr_of!(RemainingPath)) as usize);
            return ERROR_PATH_NOT_FOUND;
        }

        RemainingPath.Buffer = RemainingPath.Buffer.offset(1);
        RemainingPath.Length -= core::mem::size_of::<u16>() as u16;

        GetNextPathElement(core::ptr::addr_of_mut!(SubKeyName1), core::ptr::addr_of_mut!(RemainingPath));
        GetNextPathElement(core::ptr::addr_of_mut!(SubKeyName2), core::ptr::addr_of_mut!(RemainingPath));
        GetNextPathElement(core::ptr::addr_of_mut!(SubKeyName3), core::ptr::addr_of_mut!(RemainingPath));
        TRACE!("RegOpenKey: %wZ / %wZ / %wZ\n", (core::ptr::addr_of!(SubKeyName1)) as usize, (core::ptr::addr_of!(SubKeyName2)) as usize, (core::ptr::addr_of!(SubKeyName3)) as usize);

        if RtlEqualUnicodeString(core::ptr::addr_of!(SubKeyName1), core::ptr::addr_of!(RegistryPath), 1u8) == 0 ||
            RtlEqualUnicodeString(core::ptr::addr_of!(SubKeyName2), core::ptr::addr_of!(MachinePath), 1u8) == 0 ||
            RtlEqualUnicodeString(core::ptr::addr_of!(SubKeyName3), core::ptr::addr_of!(SystemPath), 1u8) == 0
        {
            ERR!("RegOpenKey: invalid path '%S' (%wZ)\n", (KeyName) as usize, (core::ptr::addr_of!(RemainingPath)) as usize);
            return ERROR_PATH_NOT_FOUND;
        }

        CellIndex = SystemRootCell;
    } else {
        CellIndex = HKEY_TO_HCI(ParentKey);
    }

    if CellIndex == SystemRootCell {
        let mut TempPath: _UNICODE_STRING = RemainingPath;

        GetNextPathElement(core::ptr::addr_of_mut!(SubKeyName), core::ptr::addr_of_mut!(TempPath));

        if RtlEqualUnicodeString(core::ptr::addr_of!(SubKeyName), core::ptr::addr_of!(CurrentControlSet), 1u8) != 0 {
            CellIndex = HKEY_TO_HCI(CurrentControlSetKey);
            RemainingPath = TempPath;
        }
    }

    KeyNode = HvGetCell(Hive, CellIndex) as PCM_KEY_NODE;

    TRACE!("RegOpenKey: RemainingPath '%wZ'\n", (core::ptr::addr_of!(RemainingPath)) as usize);

    while GetNextPathElement(core::ptr::addr_of_mut!(SubKeyName), core::ptr::addr_of_mut!(RemainingPath)) != 0 {
        let NextCellIndex: HCELL_INDEX;

        TRACE!("RegOpenKey: next element '%wZ'\n", (core::ptr::addr_of!(SubKeyName)) as usize);

        NextCellIndex = CmpFindSubKeyByName(Hive, KeyNode, core::ptr::addr_of_mut!(SubKeyName));
        HvReleaseCell(Hive, CellIndex);
        CellIndex = NextCellIndex;
        if CellIndex == HCELL_NIL {
            WARN!("Did not find sub key '%wZ' (full: %S)\n", (core::ptr::addr_of!(SubKeyName)) as usize, (KeyName) as usize);
            return ERROR_PATH_NOT_FOUND;
        }

        KeyNode = HvGetCell(Hive, CellIndex) as PCM_KEY_NODE;
    }

    HvReleaseCell(Hive, CellIndex);
    *Key = HCI_TO_HKEY(CellIndex);

    return ERROR_SUCCESS;
}

unsafe fn RepGetValueData(
    Hive: PHHIVE,
    ValueCell: PCM_KEY_VALUE,
    Type: *mut u32,
    Data: *mut u8,
    DataSize: *mut u32)
{
    let mut DataLength: u32 = 0;
    let DataCell: PCELL_DATA;

    if !Type.is_null() {
        *Type = (*ValueCell).Type;
    }

    if !DataSize.is_null() {
        DataCell = CmpValueToData(Hive, ValueCell, &mut DataLength);

        if !Data.is_null() && (*DataSize != 0) {
            let copy_size = if *DataSize < DataLength { *DataSize } else { DataLength };
            core::ptr::copy_nonoverlapping(
                DataCell as *const u8,
                Data,
                copy_size as usize
            );
        }

        *DataSize = DataLength;
    }
}

#[no_mangle]
pub unsafe extern "C" fn RegQueryValue(
    Key: HKEY,
    ValueName: PCWSTR,
    Type: *mut u32,
    Data: *mut u8,
    DataSize: *mut u32)
-> i32
{

    let Hive: PHHIVE = GET_HHIVE_FROM_HKEY(Key);
    let KeyNode: PCM_KEY_NODE;
    let ValueCell: PCM_KEY_VALUE;
    let CellIndex: HCELL_INDEX;
    let mut ValueNameString: _UNICODE_STRING = core::mem::zeroed();

    TRACE!("RegQueryValue(%p, '%S', %p, %p, %p)\n", (Key) as usize, (ValueName) as usize, (Type) as usize, (Data) as usize, (DataSize) as usize);

    KeyNode = GET_CM_KEY_NODE(Hive, Key);

    RtlInitUnicodeString(core::ptr::addr_of_mut!(ValueNameString), ValueName);
    CellIndex = CmpFindValueByName(Hive, KeyNode, core::ptr::addr_of_mut!(ValueNameString));
    if CellIndex == HCELL_NIL {
        let kn_namelen = core::ptr::addr_of!((*KeyNode).NameLength).read_unaligned();
        let kn_name = core::ptr::addr_of!((*KeyNode).Name) as *const u16;
        TRACE!("RegQueryValue value not found in key (%.*s)\n", (kn_namelen) as usize, (kn_name) as usize);
        HvReleaseCell(Hive, HKEY_TO_HCI(Key));
        return ERROR_FILE_NOT_FOUND;
    }
    HvReleaseCell(Hive, HKEY_TO_HCI(Key));

    ValueCell = HvGetCell(Hive, CellIndex) as PCM_KEY_VALUE;

    RepGetValueData(Hive, ValueCell, Type, Data, DataSize);

    HvReleaseCell(Hive, CellIndex);

    return ERROR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "C" fn RegEnumValue(
    Key: HKEY,
    Index: u32,
    ValueName: *mut u16,
    NameSize: *mut u32,
    Type: *mut u32,
    Data: *mut u8,
    DataSize: *mut u32,
) -> i32 {

    let Hive: PHHIVE = GET_HHIVE_FROM_HKEY(Key);
    let KeyNode: PCM_KEY_NODE;
    let ValueListCell: PCELL_DATA;
    let ValueCell: PCM_KEY_VALUE;
    let NameLength: u16;

    TRACE!("RegEnumValue(%p, %lu, %S, %p, %p, %p, %p (%lu))\n", (Key) as usize, (Index) as usize, (ValueName) as usize, (NameSize) as usize, (Type) as usize, (Data) as usize, (DataSize) as usize, (if !DataSize.is_null() { *DataSize } else { 0 }) as usize);

    KeyNode = GET_CM_KEY_NODE(Hive, Key);

    let VlCount = core::ptr::addr_of!((*KeyNode).__bindgen_anon_1.__bindgen_anon_1.ValueList.Count).read_unaligned();
    let VlList = core::ptr::addr_of!((*KeyNode).__bindgen_anon_1.__bindgen_anon_1.ValueList.List).read_unaligned();
    if (VlCount == 0)
        || (VlList == HCELL_NIL)
        || (Index >= VlCount)
    {
        ERR!("RegEnumValue: index invalid\n");
        HvReleaseCell(Hive, HKEY_TO_HCI(Key));
        return ERROR_NO_MORE_ITEMS;
    }

    ValueListCell = HvGetCell(Hive, VlList) as PCELL_DATA;

    // KeyList is a C flexible array (`HCELL_INDEX KeyList[ANYSIZE_ARRAY]`, cmlib/cmdata.h:205)
    // holding the key's ValueList.Count cells. bindgen's [_; 1] panicked -- in the boot
    // loader -- enumerating any key's second value. Constant index 0 is checked at
    // compile time.
    let value_index = *(&raw const (*ValueListCell).u.KeyList[0]).add(Index as usize);
    ValueCell = HvGetCell(Hive, value_index) as PCM_KEY_VALUE;

    if ((*ValueCell).Flags & VALUE_COMP_NAME) != 0 {
        let vc_namelen = core::ptr::addr_of!((*ValueCell).NameLength).read_unaligned();
        let vc_name = core::ptr::addr_of!((*ValueCell).Name) as *const u16;
        NameLength = CmpCompressedNameSize(
            vc_name as _,
            vc_namelen as u32,
        );

        CmpCopyCompressedName(
            ValueName,
            *NameSize,
            vc_name as _,
            vc_namelen as u32,
        );
    } else {
        let vc_namelen2 = core::ptr::addr_of!((*ValueCell).NameLength).read_unaligned();
        let vc_name2 = core::ptr::addr_of!((*ValueCell).Name) as *const core::ffi::c_void;
        NameLength = vc_namelen2;

        RtlCopyMemory(
            ValueName as *mut core::ffi::c_void,
            vc_name2,
            min(*NameSize, vc_namelen2 as u32 ) as usize,
        );
    }

    if *NameSize >= NameLength as u32 + core::mem::size_of::<u16>() as u32 {
        *ValueName.add((NameLength as usize) / core::mem::size_of::<u16>()) = 0u16;
    }

    *NameSize = NameLength as u32 + core::mem::size_of::<u16>() as u32;

    RepGetValueData(Hive, ValueCell, Type, Data, DataSize);

    HvReleaseCell(Hive, value_index);
    HvReleaseCell(Hive, VlList);
    HvReleaseCell(Hive, HKEY_TO_HCI(Key));

    TRACE!("RegEnumValue done -> %u, '%.*S'\n", (*NameSize) as usize, (*NameSize) as usize, (ValueName) as usize);
    return ERROR_SUCCESS;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  FreeLoader
//  *
//  *  Copyright (C) 2014  Timo Kreuzer <timo.kreuzer@reactos.org>
//  *                2022  George Bișoc <george.bisoc@reactos.org>
//  *
//  *  This program is free software; you can redistribute it and/or modify
//  *  it under the terms of the GNU General Public License as published by
//  *  the Free Software Foundation; either version 2 of the License, or
//  *  (at your option) any later version.
//  *
//  *  This program is distributed in the hope that it will be useful,
//  *  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  *  GNU General Public License for more details.
//  *
//  *  You should have received a copy of the GNU General Public License along
//  *  with this program; if not, write to the Free Software Foundation, Inc.,
//  *  51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
//  */
//
// #include <freeldr.h>
// #include <cmlib.h>
// #include "registry.h"
// #include <internal/cmboot.h>
//
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(REGISTRY);
//
// static PCMHIVE CmSystemHive;
// static HCELL_INDEX SystemRootCell;
//
// PHHIVE SystemHive = NULL;
// HKEY CurrentControlSetKey = NULL;
//
// #define HCI_TO_HKEY(CellIndex)          ((HKEY)(ULONG_PTR)(CellIndex))
// #ifndef HKEY_TO_HCI // See also registry.h
// #define HKEY_TO_HCI(hKey)               ((HCELL_INDEX)(ULONG_PTR)(hKey))
// #endif
//
// #define GET_HHIVE(CmHive)               (&((CmHive)->Hive))
// #define GET_HHIVE_FROM_HKEY(hKey)       GET_HHIVE(CmSystemHive)
// #define GET_CM_KEY_NODE(hHive, hKey)    ((PCM_KEY_NODE)HvGetCell(hHive, HKEY_TO_HCI(hKey)))
//
// #define GET_HBASE_BLOCK(ChunkBase)      ((PHBASE_BLOCK)ChunkBase)
//
// PVOID
// NTAPI
// CmpAllocate(
//     IN SIZE_T Size,
//     IN BOOLEAN Paged,
//     IN ULONG Tag)
// {
//     UNREFERENCED_PARAMETER(Paged);
//     return FrLdrHeapAlloc(Size, Tag);
// }
//
// VOID
// NTAPI
// CmpFree(
//     IN PVOID Ptr,
//     IN ULONG Quota)
// {
//     UNREFERENCED_PARAMETER(Quota);
//     FrLdrHeapFree(Ptr, 0);
// }
//
// /**
//  * @brief
//  * Initializes a flat hive descriptor for the
//  * hive and validates the registry hive.
//  * Volatile data is purged during this procedure
//  * for initialization.
//  *
//  * @param[in] CmHive
//  * A pointer to a CM (in-memory) hive descriptor
//  * containing the hive descriptor to be initialized.
//  *
//  * @param[in] ChunkBase
//  * An arbitrary pointer that points to the registry
//  * chunk base. This pointer serves as the base block
//  * containing the hive file header data.
//  *
//  * @param[in] LoadAlternate
//  * If set to TRUE, the function will initialize the
//  * hive as an alternate hive, otherwise FALSE to initialize
//  * it as primary.
//  *
//  * @return
//  * Returns TRUE if the hive has been initialized
//  * and registry data inside the hive is valid, FALSE
//  * otherwise.
//  */
// static
// BOOLEAN
// RegInitializeHive(
//     _In_ PCMHIVE CmHive,
//     _In_ PVOID ChunkBase,
//     _In_ BOOLEAN LoadAlternate)
// {
//     NTSTATUS Status;
//     CM_CHECK_REGISTRY_STATUS CmStatusCode;
//
//     /* Initialize the hive */
//     Status = HvInitialize(GET_HHIVE(CmHive),
//                           HINIT_FLAT, // HINIT_MEMORY_INPLACE
//                           0,
//                           LoadAlternate ? HFILE_TYPE_ALTERNATE : HFILE_TYPE_PRIMARY,
//                           ChunkBase,
//                           CmpAllocate,
//                           CmpFree,
//                           NULL,
//                           NULL,
//                           NULL,
//                           NULL,
//                           1,
//                           NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("Failed to initialize the flat hive (Status 0x%lx)\n", (Status) as usize);
//         return FALSE;
//     }
//
//     /* Now check the hive and purge volatile data */
//     CmStatusCode = CmCheckRegistry(CmHive, CM_CHECK_REGISTRY_BOOTLOADER_PURGE_VOLATILES | CM_CHECK_REGISTRY_VALIDATE_HIVE);
//     if (!CM_CHECK_REGISTRY_SUCCESS(CmStatusCode))
//     {
//         ERR("CmCheckRegistry detected problems with the loaded flat hive (check code %lu)\n", (CmStatusCode) as usize);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /**
//  * @brief
//  * Loads and reads a hive log at specified
//  * file offset.
//  *
//  * @param[in] DirectoryPath
//  * A pointer to a string that denotes the directory
//  * path of the hives and logs location.
//  *
//  * @param[in] LogFileOffset
//  * The file offset of which this function uses to
//  * seek at specific position during read.
//  *
//  * @param[in] LogName
//  * A pointer to a string that denotes the name of
//  * the desired hive log (e.g. "SYSTEM").
//  *
//  * @param[out] LogData
//  * A pointer to the returned hive log data that was
//  * read. The following data varies depending on the
//  * specified offset set up by the caller, that is used
//  * to where to start reading from the hive log.
//  *
//  * @return
//  * Returns TRUE if the hive log was loaded and read
//  * successfully, FALSE otherwise.
//  *
//  * @remarks
//  * The returned log data pointer to the caller is a
//  * virtual address. You must use VaToPa that converts
//  * the address to a physical one in order to actually
//  * use it!
//  */
// static
// BOOLEAN
// RegLoadHiveLog(
//     _In_ PCSTR DirectoryPath,
//     _In_ ULONG LogFileOffset,
//     _In_ PCSTR LogName,
//     _Out_ PVOID *LogData)
// {
//     ARC_STATUS Status;
//     ULONG LogId;
//     CHAR LogPath[MAX_PATH];
//     ULONG LogFileSize;
//     FILEINFORMATION FileInfo;
//     LARGE_INTEGER Position;
//     ULONG BytesRead;
//     PVOID LogDataVirtual;
//     PVOID LogDataPhysical;
//
//     /* Build the full path to the hive log */
//     RtlStringCbCopyA(LogPath, sizeof(LogPath), DirectoryPath);
//     RtlStringCbCatA(LogPath, sizeof(LogPath), LogName);
//
//     /* Open the file */
//     Status = ArcOpen(LogPath, OpenReadOnly, &LogId);
//     if (Status != ESUCCESS)
//     {
//         ERR("Failed to open %s (ARC code %lu)\n", (LogName) as usize, (Status) as usize);
//         return FALSE;
//     }
//
//     /* Get the file length */
//     Status = ArcGetFileInformation(LogId, &FileInfo);
//     if (Status != ESUCCESS)
//     {
//         ERR("Failed to get file information from %s (ARC code %lu)\n", (LogName) as usize, (Status) as usize);
//         ArcClose(LogId);
//         return FALSE;
//     }
//
//     /* Capture the size of the hive log file */
//     LogFileSize = FileInfo.EndingAddress.LowPart;
//     if (LogFileSize == 0)
//     {
//         ERR("LogFileSize is 0, %s is corrupt\n", (LogName) as usize);
//         ArcClose(LogId);
//         return FALSE;
//     }
//
//     /* Allocate memory blocks for our log data */
//     LogDataPhysical = MmAllocateMemoryWithType(
//         MM_SIZE_TO_PAGES(LogFileSize + MM_PAGE_SIZE - 1) << MM_PAGE_SHIFT,
//         LoaderRegistryData);
//     if (LogDataPhysical == NULL)
//     {
//         ERR("Failed to allocate memory for log data\n");
//         ArcClose(LogId);
//         return FALSE;
//     }
//
//     /* Convert the address to virtual so that it can be useable */
//     LogDataVirtual = PaToVa(LogDataPhysical);
//
//     /* Seek within the log file at desired position */
//     Position.QuadPart = LogFileOffset;
//     Status = ArcSeek(LogId, &Position, SeekAbsolute);
//     if (Status != ESUCCESS)
//     {
//         ERR("Failed to seek at %s (ARC code %lu)\n", (LogName) as usize, (Status) as usize);
//         ArcClose(LogId);
//         return FALSE;
//     }
//
//     /* And read the actual data from the log */
//     Status = ArcRead(LogId, LogDataPhysical, LogFileSize, &BytesRead);
//     if (Status != ESUCCESS)
//     {
//         ERR("Failed to read %s (ARC code %lu)\n", (LogName) as usize, (Status) as usize);
//         ArcClose(LogId);
//         return FALSE;
//     }
//
//     *LogData = LogDataVirtual;
//     ArcClose(LogId);
//     return TRUE;
// }
//
// /**
//  * @brief
//  * Recovers the header base block of a flat
//  * registry hive.
//  *
//  * @param[in] ChunkBase
//  * A pointer to the registry hive chunk base of
//  * which the damaged header block is to be recovered.
//  *
//  * @param[in] DirectoryPath
//  * A pointer to a string that denotes the directory
//  * path of the hives and logs location.
//  *
//  * @param[in] LogName
//  * A pointer to a string that denotes the name of
//  * the desired hive log (e.g. "SYSTEM").
//  *
//  * @return
//  * Returns TRUE if the header base block was successfully
//  * recovered, FALSE otherwise.
//  */
// static
// BOOLEAN
// RegRecoverHeaderHive(
//     _Inout_ PVOID ChunkBase,
//     _In_ PCSTR DirectoryPath,
//     _In_ PCSTR LogName)
// {
//     BOOLEAN Success;
//     CHAR FullLogFileName[MAX_PATH];
//     PVOID LogData;
//     PHBASE_BLOCK HiveBaseBlock;
//     PHBASE_BLOCK LogBaseBlock;
//
//     /* Build the complete path of the hive log */
//     RtlStringCbCopyA(FullLogFileName, sizeof(FullLogFileName), LogName);
//     RtlStringCbCatA(FullLogFileName, sizeof(FullLogFileName), ".LOG");
//     Success = RegLoadHiveLog(DirectoryPath, 0, FullLogFileName, &LogData);
//     if (!Success)
//     {
//         ERR("Failed to read the hive log\n");
//         return FALSE;
//     }
//
//     /* Make sure the header from the hive log is actually sane  */
//     LogData = VaToPa(LogData);
//     LogBaseBlock = GET_HBASE_BLOCK(LogData);
//     if (!HvpVerifyHiveHeader(LogBaseBlock, HFILE_TYPE_LOG))
//     {
//         ERR("The hive log has corrupt base block\n");
//         return FALSE;
//     }
//
//     /* Copy the healthy header base block into the primary hive */
//     HiveBaseBlock = GET_HBASE_BLOCK(ChunkBase);
//     WARN("Recovering the hive base block...\n");
//     RtlCopyMemory(HiveBaseBlock,
//                   LogBaseBlock,
//                   LogBaseBlock->Cluster * HSECTOR_SIZE);
//     HiveBaseBlock->Type = HFILE_TYPE_PRIMARY;
//     return TRUE;
// }
//
// /**
//  * @brief
//  * Recovers the corrupt data of a primary flat
//  * registry hive.
//  *
//  * @param[in] ChunkBase
//  * A pointer to the registry hive chunk base of
//  * which the damaged hive data is to be replaced
//  * with healthy data from the corresponding hive log.
//  *
//  * @param[in] DirectoryPath
//  * A pointer to a string that denotes the directory
//  * path of the hives and logs location.
//  *
//  * @param[in] LogName
//  * A pointer to a string that denotes the name of
//  * the desired hive log (e.g. "SYSTEM").
//  *
//  * @return
//  * Returns TRUE if the hive data was successfully
//  * recovered, FALSE otherwise.
//  *
//  * @remarks
//  * Data recovery of the target hive does not always
//  * guarantee the primary hive is fully recovered.
//  * It could happen a block from a hive log is not
//  * marked dirty (pending to be written to disk) that
//  * has healthy data therefore the following bad block
//  * would still remain in corrupt state in the main primary
//  * hive. In such scenarios an alternate hive must be replayed.
//  */
// static
// BOOLEAN
// RegRecoverDataHive(
//     _Inout_ PVOID ChunkBase,
//     _In_ PCSTR DirectoryPath,
//     _In_ PCSTR LogName)
// {
//     BOOLEAN Success;
//     ULONG StorageLength;
//     ULONG BlockIndex, LogIndex;
//     PUCHAR BlockPtr, BlockDest;
//     CHAR FullLogFileName[MAX_PATH];
//     PVOID LogData;
//     PUCHAR LogDataPhysical;
//     PHBASE_BLOCK HiveBaseBlock;
//
//     /* Build the complete path of the hive log */
//     RtlStringCbCopyA(FullLogFileName, sizeof(FullLogFileName), LogName);
//     RtlStringCbCatA(FullLogFileName, sizeof(FullLogFileName), ".LOG");
//     Success = RegLoadHiveLog(DirectoryPath, HV_LOG_HEADER_SIZE, FullLogFileName, &LogData);
//     if (!Success)
//     {
//         ERR("Failed to read the hive log\n");
//         return FALSE;
//     }
//
//     /* Make sure the dirty vector signature is there otherwise the hive log is corrupt */
//     LogDataPhysical = (PUCHAR)VaToPa(LogData);
//     if (*((PULONG)LogDataPhysical) != HV_LOG_DIRTY_SIGNATURE)
//     {
//         ERR("The hive log dirty signature could not be found\n");
//         return FALSE;
//     }
//
//     /* Copy the dirty data into the primary hive */
//     LogIndex = 0;
//     BlockIndex = 0;
//     HiveBaseBlock = GET_HBASE_BLOCK(ChunkBase);
//     StorageLength = HiveBaseBlock->Length / HBLOCK_SIZE;
//     for (; BlockIndex < StorageLength; ++BlockIndex)
//     {
//         /* Skip this block if it's not dirty and go to the next one */
//         if (LogDataPhysical[BlockIndex + sizeof(HV_LOG_DIRTY_SIGNATURE)] != HV_LOG_DIRTY_BLOCK)
//         {
//             continue;
//         }
//
//         /* Read the dirty block and copy it at right offsets */
//         BlockPtr = (PUCHAR)((ULONG_PTR)LogDataPhysical + 2 * HSECTOR_SIZE + LogIndex * HBLOCK_SIZE);
//         BlockDest = (PUCHAR)((ULONG_PTR)ChunkBase + (BlockIndex + 1) * HBLOCK_SIZE);
//         RtlCopyMemory(BlockDest, BlockPtr, HBLOCK_SIZE);
//
//         /* Increment the index in log as we continue further */
//         LogIndex++;
//     }
//
//     /* Fix the secondary sequence of the primary hive and compute a new checksum */
//     HiveBaseBlock->Sequence2 = HiveBaseBlock->Sequence1;
//     HiveBaseBlock->CheckSum = HvpHiveHeaderChecksum(HiveBaseBlock);
//     return TRUE;
// }
//
// /**
//  * @brief
//  * Imports the SYSTEM binary hive from
//  * the registry base chunk that's been
//  * provided by the loader block.
//  *
//  * @param[in] ChunkBase
//  * A pointer to the registry base chunk
//  * that serves for SYSTEM hive initialization.
//  *
//  * @param[in] ChunkSize
//  * The size of the registry base chunk. This
//  * parameter refers to the actual size of
//  * the SYSTEM hive. This parameter is currently
//  * unused.
//  *
//  * @param[in] LoadAlternate
//  * If set to TRUE, the function will initialize the
//  * hive as an alternate hive, otherwise FALSE to initialize
//  * it as primary.
//  *
//  * @return
//  * Returns TRUE if hive importing and initialization
//  * have succeeded, FALSE otherwise.
//  */
// BOOLEAN
// RegImportBinaryHive(
//     _In_ PVOID ChunkBase,
//     _In_ ULONG ChunkSize,
//     _In_ PCSTR SearchPath,
//     _In_ BOOLEAN LoadAlternate)
// {
//     BOOLEAN Success;
//     PCM_KEY_NODE KeyNode;
//
//     TRACE("RegImportBinaryHive(%p, 0x%lx)\n", (ChunkBase) as usize, (ChunkSize) as usize);
//
//     /* Assume that we don't need boot recover, unless we have to */
//     ((PHBASE_BLOCK)ChunkBase)->BootRecover = HBOOT_NO_BOOT_RECOVER;
//
//     /* Allocate and initialize the hive */
//     CmSystemHive = FrLdrTempAlloc(sizeof(CMHIVE), 'eviH');
//     Success = RegInitializeHive(CmSystemHive, ChunkBase, LoadAlternate);
//     if (!Success)
//     {
//         /* Free the buffer and retry again */
//         FrLdrTempFree(CmSystemHive, 'eviH');
//         CmSystemHive = NULL;
//
//         if (!RegRecoverHeaderHive(ChunkBase, SearchPath, "SYSTEM"))
//         {
//             ERR("Failed to recover the hive header block\n");
//             return FALSE;
//         }
//
//         if (!RegRecoverDataHive(ChunkBase, SearchPath, "SYSTEM"))
//         {
//             ERR("Failed to recover the hive data\n");
//             return FALSE;
//         }
//
//         /* Now retry initializing the hive again */
//         CmSystemHive = FrLdrTempAlloc(sizeof(CMHIVE), 'eviH');
//         Success = RegInitializeHive(CmSystemHive, ChunkBase, LoadAlternate);
//         if (!Success)
//         {
//             ERR("Corrupted hive (despite recovery) %p\n", (ChunkBase) as usize);
//             FrLdrTempFree(CmSystemHive, 'eviH');
//             return FALSE;
//         }
//
//         /*
//          * Acknowledge the kernel we recovered the SYSTEM hive
//          * on our side by applying log data.
//          */
//         ((PHBASE_BLOCK)ChunkBase)->BootRecover = HBOOT_BOOT_RECOVERED_BY_HIVE_LOG;
//     }
//
//     /* Save the root key node */
//     SystemHive = GET_HHIVE(CmSystemHive);
//     SystemRootCell = SystemHive->BaseBlock->RootCell;
//     ASSERT(SystemRootCell != HCELL_NIL);
//
//     /* Verify it is accessible */
//     KeyNode = (PCM_KEY_NODE)HvGetCell(SystemHive, SystemRootCell);
//     ASSERT(KeyNode);
//     ASSERT(KeyNode->Signature == CM_KEY_NODE_SIGNATURE);
//     HvReleaseCell(SystemHive, SystemRootCell);
//
//     return TRUE;
// }
//
// BOOLEAN
// RegInitCurrentControlSet(
//     _In_ BOOLEAN LastKnownGood)
// {
//     UNICODE_STRING ControlSetName;
//     HCELL_INDEX ControlCell;
//     PCM_KEY_NODE KeyNode;
//     BOOLEAN AutoSelect;
//
//     TRACE("RegInitCurrentControlSet\n");
//
//     /* Choose which control set to open and set it as the new "Current" */
//     RtlInitUnicodeString(&ControlSetName,
//                          LastKnownGood ? L"LastKnownGood"
//                                        : L"Default");
//
//     ControlCell = CmpFindControlSet(SystemHive,
//                                     SystemRootCell,
//                                     &ControlSetName,
//                                     &AutoSelect);
//     if (ControlCell == HCELL_NIL)
//     {
//         ERR("CmpFindControlSet('%wZ') failed\n", (&ControlSetName) as usize);
//         return FALSE;
//     }
//
//     CurrentControlSetKey = HCI_TO_HKEY(ControlCell);
//
//     /* Verify it is accessible */
//     KeyNode = (PCM_KEY_NODE)HvGetCell(SystemHive, ControlCell);
//     ASSERT(KeyNode);
//     ASSERT(KeyNode->Signature == CM_KEY_NODE_SIGNATURE);
//     HvReleaseCell(SystemHive, ControlCell);
//
//     return TRUE;
// }
//
// static
// BOOLEAN
// GetNextPathElement(
//     _Out_ PUNICODE_STRING NextElement,
//     _Inout_ PUNICODE_STRING RemainingPath)
// {
//     /* Check if there are any characters left */
//     if (RemainingPath->Length < sizeof(WCHAR))
//     {
//         /* Nothing left, bail out early */
//         return FALSE;
//     }
//
//     /* The next path elements starts with the remaining path */
//     NextElement->Buffer = RemainingPath->Buffer;
//
//     /* Loop until the path element ends */
//     while ((RemainingPath->Length >= sizeof(WCHAR)) &&
//            (RemainingPath->Buffer[0] != '\\'))
//     {
//         /* Skip this character */
//         RemainingPath->Buffer++;
//         RemainingPath->Length -= sizeof(WCHAR);
//     }
//
//     NextElement->Length = (USHORT)(RemainingPath->Buffer - NextElement->Buffer) * sizeof(WCHAR);
//     NextElement->MaximumLength = NextElement->Length;
//
//     /* Check if the path element ended with a path separator */
//     if (RemainingPath->Length >= sizeof(WCHAR))
//     {
//         /* Skip the path separator */
//         ASSERT(RemainingPath->Buffer[0] == '\\');
//         RemainingPath->Buffer++;
//         RemainingPath->Length -= sizeof(WCHAR);
//     }
//
//     /* Return whether we got any characters */
//     return TRUE;
// }
//
// #if 0
// LONG
// RegEnumKey(
//     _In_ HKEY Key,
//     _In_ ULONG Index,
//     _Out_ PWCHAR Name,
//     _Inout_ PULONG NameSize,
//     _Out_opt_ PHKEY SubKey)
// {
//     PHHIVE Hive = GET_HHIVE_FROM_HKEY(Key);
//     PCM_KEY_NODE KeyNode, SubKeyNode;
//     HCELL_INDEX CellIndex;
//     USHORT NameLength;
//
//     TRACE("RegEnumKey(%p, %lu, %p, %p->%u)\n", (//           Key) as usize, (Index) as usize, (Name) as usize, (NameSize) as usize, (NameSize ? *NameSize : 0) as usize);
//
//     /* Get the key node */
//     KeyNode = GET_CM_KEY_NODE(Hive, Key);
//     ASSERT(KeyNode);
//     ASSERT(KeyNode->Signature == CM_KEY_NODE_SIGNATURE);
//
//     CellIndex = CmpFindSubKeyByNumber(Hive, KeyNode, Index);
//     if (CellIndex == HCELL_NIL)
//     {
//         TRACE("RegEnumKey index out of bounds (%d) in key (%.*s)\n", (//               Index) as usize, (KeyNode->NameLength) as usize, (KeyNode->Name) as usize);
//         HvReleaseCell(Hive, HKEY_TO_HCI(Key));
//         return ERROR_NO_MORE_ITEMS;
//     }
//     HvReleaseCell(Hive, HKEY_TO_HCI(Key));
//
//     /* Get the value cell */
//     SubKeyNode = (PCM_KEY_NODE)HvGetCell(Hive, CellIndex);
//     ASSERT(SubKeyNode != NULL);
//     ASSERT(SubKeyNode->Signature == CM_KEY_NODE_SIGNATURE);
//
//     if (SubKeyNode->Flags & KEY_COMP_NAME)
//     {
//         NameLength = CmpCompressedNameSize(SubKeyNode->Name, SubKeyNode->NameLength);
//
//         /* Compressed name */
//         CmpCopyCompressedName(Name,
//                               *NameSize,
//                               SubKeyNode->Name,
//                               SubKeyNode->NameLength);
//     }
//     else
//     {
//         NameLength = SubKeyNode->NameLength;
//
//         /* Normal name */
//         RtlCopyMemory(Name, SubKeyNode->Name,
//                       min(*NameSize, SubKeyNode->NameLength));
//     }
//
//     if (*NameSize >= NameLength + sizeof(WCHAR))
//     {
//         Name[NameLength / sizeof(WCHAR)] = UNICODE_NULL;
//     }
//
//     *NameSize = NameLength + sizeof(WCHAR);
//
//     HvReleaseCell(Hive, CellIndex);
//
//     if (SubKey != NULL)
//         *SubKey = HCI_TO_HKEY(CellIndex);
//
//     TRACE("RegEnumKey done -> %u, '%.*S'\n", (*NameSize) as usize, (*NameSize) as usize, (Name) as usize);
//     return ERROR_SUCCESS;
// }
// #endif
//
// LONG
// RegOpenKey(
//     _In_ HKEY ParentKey,
//     _In_z_ PCWSTR KeyName,
//     _Out_ PHKEY Key)
// {
//     UNICODE_STRING RemainingPath, SubKeyName;
//     UNICODE_STRING CurrentControlSet = RTL_CONSTANT_STRING(L"CurrentControlSet");
//     PHHIVE Hive = (ParentKey ? GET_HHIVE_FROM_HKEY(ParentKey) : GET_HHIVE(CmSystemHive));
//     PCM_KEY_NODE KeyNode;
//     HCELL_INDEX CellIndex;
//
//     TRACE("RegOpenKey(%p, '%S', %p)\n", (ParentKey) as usize, (KeyName) as usize, (Key) as usize);
//
//     /* Initialize the remaining path name */
//     RtlInitUnicodeString(&RemainingPath, KeyName);
//
//     /* Check if we have a parent key */
//     if (ParentKey == NULL)
//     {
//         UNICODE_STRING SubKeyName1, SubKeyName2, SubKeyName3;
//         UNICODE_STRING RegistryPath = RTL_CONSTANT_STRING(L"Registry");
//         UNICODE_STRING MachinePath = RTL_CONSTANT_STRING(L"MACHINE");
//         UNICODE_STRING SystemPath = RTL_CONSTANT_STRING(L"SYSTEM");
//
//         TRACE("RegOpenKey: absolute path\n");
//
//         if ((RemainingPath.Length < sizeof(WCHAR)) ||
//             RemainingPath.Buffer[0] != '\\')
//         {
//             /* The key path is not absolute */
//             ERR("RegOpenKey: invalid path '%S' (%wZ)\n", (KeyName) as usize, (&RemainingPath) as usize);
//             return ERROR_PATH_NOT_FOUND;
//         }
//
//         /* Skip initial path separator */
//         RemainingPath.Buffer++;
//         RemainingPath.Length -= sizeof(WCHAR);
//
//         /* Get the first 3 path elements */
//         GetNextPathElement(&SubKeyName1, &RemainingPath);
//         GetNextPathElement(&SubKeyName2, &RemainingPath);
//         GetNextPathElement(&SubKeyName3, &RemainingPath);
//         TRACE("RegOpenKey: %wZ / %wZ / %wZ\n", (&SubKeyName1) as usize, (&SubKeyName2) as usize, (&SubKeyName3) as usize);
//
//         /* Check if we have the correct path */
//         if (!RtlEqualUnicodeString(&SubKeyName1, &RegistryPath, TRUE) ||
//             !RtlEqualUnicodeString(&SubKeyName2, &MachinePath, TRUE) ||
//             !RtlEqualUnicodeString(&SubKeyName3, &SystemPath, TRUE))
//         {
//             /* The key path is not inside HKLM\Machine\System */
//             ERR("RegOpenKey: invalid path '%S' (%wZ)\n", (KeyName) as usize, (&RemainingPath) as usize);
//             return ERROR_PATH_NOT_FOUND;
//         }
//
//         /* Use the root key */
//         CellIndex = SystemRootCell;
//     }
//     else
//     {
//         /* Use the parent key */
//         CellIndex = HKEY_TO_HCI(ParentKey);
//     }
//
//     /* Check if this is the root key */
//     if (CellIndex == SystemRootCell)
//     {
//         UNICODE_STRING TempPath = RemainingPath;
//
//         /* Get the first path element */
//         GetNextPathElement(&SubKeyName, &TempPath);
//
//         /* Check if this is CurrentControlSet */
//         if (RtlEqualUnicodeString(&SubKeyName, &CurrentControlSet, TRUE))
//         {
//             /* Use the CurrentControlSetKey and update the remaining path */
//             CellIndex = HKEY_TO_HCI(CurrentControlSetKey);
//             RemainingPath = TempPath;
//         }
//     }
//
//     /* Get the key node */
//     KeyNode = (PCM_KEY_NODE)HvGetCell(Hive, CellIndex);
//     ASSERT(KeyNode);
//     ASSERT(KeyNode->Signature == CM_KEY_NODE_SIGNATURE);
//
//     TRACE("RegOpenKey: RemainingPath '%wZ'\n", (&RemainingPath) as usize);
//
//     /* Loop while there are path elements */
//     while (GetNextPathElement(&SubKeyName, &RemainingPath))
//     {
//         HCELL_INDEX NextCellIndex;
//
//         TRACE("RegOpenKey: next element '%wZ'\n", (&SubKeyName) as usize);
//
//         /* Get the next sub key */
//         NextCellIndex = CmpFindSubKeyByName(Hive, KeyNode, &SubKeyName);
//         HvReleaseCell(Hive, CellIndex);
//         CellIndex = NextCellIndex;
//         if (CellIndex == HCELL_NIL)
//         {
//             WARN("Did not find sub key '%wZ' (full: %S)\n", (&SubKeyName) as usize, (KeyName) as usize);
//             return ERROR_PATH_NOT_FOUND;
//         }
//
//         /* Get the found key */
//         KeyNode = (PCM_KEY_NODE)HvGetCell(Hive, CellIndex);
//         ASSERT(KeyNode);
//         ASSERT(KeyNode->Signature == CM_KEY_NODE_SIGNATURE);
//     }
//
//     HvReleaseCell(Hive, CellIndex);
//     *Key = HCI_TO_HKEY(CellIndex);
//
//     return ERROR_SUCCESS;
// }
//
// static
// VOID
// RepGetValueData(
//     _In_ PHHIVE Hive,
//     _In_ PCM_KEY_VALUE ValueCell,
//     _Out_opt_ PULONG Type,
//     _Out_opt_ PUCHAR Data,
//     _Inout_opt_ PULONG DataSize)
// {
//     ULONG DataLength;
//     PVOID DataCell;
//
//     /* Does the caller want the type? */
//     if (Type != NULL)
//         *Type = ValueCell->Type;
//
//     /* Does the caller provide DataSize? */
//     if (DataSize != NULL)
//     {
//         // NOTE: CmpValueToData doesn't support big data (the function will
//         // bugcheck if so), FreeLdr is not supposed to read such data.
//         // If big data is needed, use instead CmpGetValueData.
//         // CmpGetValueData(Hive, ValueCell, DataSize, &DataCell, ...);
//         DataCell = CmpValueToData(Hive, ValueCell, &DataLength);
//
//         /* Does the caller want the data? */
//         if ((Data != NULL) && (*DataSize != 0))
//         {
//             RtlCopyMemory(Data,
//                           DataCell,
//                           min(*DataSize, DataLength));
//         }
//
//         /* Return the actual data length */
//         *DataSize = DataLength;
//     }
// }
//
// LONG
// RegQueryValue(
//     _In_ HKEY Key,
//     _In_z_ PCWSTR ValueName,
//     _Out_opt_ PULONG Type,
//     _Out_opt_ PUCHAR Data,
//     _Inout_opt_ PULONG DataSize)
// {
//     PHHIVE Hive = GET_HHIVE_FROM_HKEY(Key);
//     PCM_KEY_NODE KeyNode;
//     PCM_KEY_VALUE ValueCell;
//     HCELL_INDEX CellIndex;
//     UNICODE_STRING ValueNameString;
//
//     TRACE("RegQueryValue(%p, '%S', %p, %p, %p)\n", (//           Key) as usize, (ValueName) as usize, (Type) as usize, (Data) as usize, (DataSize) as usize);
//
//     /* Get the key node */
//     KeyNode = GET_CM_KEY_NODE(Hive, Key);
//     ASSERT(KeyNode);
//     ASSERT(KeyNode->Signature == CM_KEY_NODE_SIGNATURE);
//
//     /* Initialize value name string */
//     RtlInitUnicodeString(&ValueNameString, ValueName);
//     CellIndex = CmpFindValueByName(Hive, KeyNode, &ValueNameString);
//     if (CellIndex == HCELL_NIL)
//     {
//         TRACE("RegQueryValue value not found in key (%.*s)\n", (//               KeyNode->NameLength) as usize, (KeyNode->Name) as usize);
//         HvReleaseCell(Hive, HKEY_TO_HCI(Key));
//         return ERROR_FILE_NOT_FOUND;
//     }
//     HvReleaseCell(Hive, HKEY_TO_HCI(Key));
//
//     /* Get the value cell */
//     ValueCell = (PCM_KEY_VALUE)HvGetCell(Hive, CellIndex);
//     ASSERT(ValueCell != NULL);
//
//     RepGetValueData(Hive, ValueCell, Type, Data, DataSize);
//
//     HvReleaseCell(Hive, CellIndex);
//
//     return ERROR_SUCCESS;
// }
//
// /*
//  * NOTE: This function is currently unused in FreeLdr; however it is kept here
//  * as an implementation reference of RegEnumValue using CMLIB that may be used
//  * elsewhere in ReactOS.
//  */
// #if 0
// LONG
// RegEnumValue(
//     _In_ HKEY Key,
//     _In_ ULONG Index,
//     _Out_ PWCHAR ValueName,
//     _Inout_ PULONG NameSize,
//     _Out_opt_ PULONG Type,
//     _Out_opt_ PUCHAR Data,
//     _Inout_opt_ PULONG DataSize)
// {
//     PHHIVE Hive = GET_HHIVE_FROM_HKEY(Key);
//     PCM_KEY_NODE KeyNode;
//     PCELL_DATA ValueListCell;
//     PCM_KEY_VALUE ValueCell;
//     USHORT NameLength;
//
//     TRACE("RegEnumValue(%p, %lu, %S, %p, %p, %p, %p (%lu))\n", (//           Key) as usize, (Index) as usize, (ValueName) as usize, (NameSize) as usize, (Type) as usize, (Data) as usize, (DataSize) as usize, (*DataSize) as usize);
//
//     /* Get the key node */
//     KeyNode = GET_CM_KEY_NODE(Hive, Key);
//     ASSERT(KeyNode);
//     ASSERT(KeyNode->Signature == CM_KEY_NODE_SIGNATURE);
//
//     /* Check if the index is valid */
//     if ((KeyNode->ValueList.Count == 0) ||
//         (KeyNode->ValueList.List == HCELL_NIL) ||
//         (Index >= KeyNode->ValueList.Count))
//     {
//         ERR("RegEnumValue: index invalid\n");
//         HvReleaseCell(Hive, HKEY_TO_HCI(Key));
//         return ERROR_NO_MORE_ITEMS;
//     }
//
//     ValueListCell = (PCELL_DATA)HvGetCell(Hive, KeyNode->ValueList.List);
//     ASSERT(ValueListCell != NULL);
//
//     /* Get the value cell */
//     ValueCell = (PCM_KEY_VALUE)HvGetCell(Hive, ValueListCell->KeyList[Index]);
//     ASSERT(ValueCell != NULL);
//     ASSERT(ValueCell->Signature == CM_KEY_VALUE_SIGNATURE);
//
//     if (ValueCell->Flags & VALUE_COMP_NAME)
//     {
//         NameLength = CmpCompressedNameSize(ValueCell->Name, ValueCell->NameLength);
//
//         /* Compressed name */
//         CmpCopyCompressedName(ValueName,
//                               *NameSize,
//                               ValueCell->Name,
//                               ValueCell->NameLength);
//     }
//     else
//     {
//         NameLength = ValueCell->NameLength;
//
//         /* Normal name */
//         RtlCopyMemory(ValueName, ValueCell->Name,
//                       min(*NameSize, ValueCell->NameLength));
//     }
//
//     if (*NameSize >= NameLength + sizeof(WCHAR))
//     {
//         ValueName[NameLength / sizeof(WCHAR)] = UNICODE_NULL;
//     }
//
//     *NameSize = NameLength + sizeof(WCHAR);
//
//     RepGetValueData(Hive, ValueCell, Type, Data, DataSize);
//
//     HvReleaseCell(Hive, ValueListCell->KeyList[Index]);
//     HvReleaseCell(Hive, KeyNode->ValueList.List);
//     HvReleaseCell(Hive, HKEY_TO_HCI(Key));
//
//     TRACE("RegEnumValue done -> %u, '%.*S'\n", (*NameSize) as usize, (*NameSize) as usize, (ValueName) as usize);
//     return ERROR_SUCCESS;
// }
// #endif
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
