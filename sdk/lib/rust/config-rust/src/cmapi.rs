use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{CMHIVE, CM_CHECK_REGISTRY_STATUS, CM_KEY_CONTROL_BLOCK, CM_KEY_VALUE, CM_PARSE_CONTEXT, CmCheckRegistry, CmpAcquireKcbLockExclusive, CmpAcquireTwoKcbLocksExclusiveByKey, CmpAddSubKey, CmpAddToHiveFileList, CmpAddValueToList, CmpCleanUpKcbCacheWithLock, CmpCleanUpKcbValueCache, CmpCleanUpSubKeyInfo, CmpCloseHiveFiles, CmpCmdHiveOpen, CmpCompareNewValueDataAgainstKCBCache, CmpCompressedNameSize, CmpConvertKcbSharedToExclusive, CmpCopyCell, CmpCopyCompressedName, CmpCopyKeyValueList, CmpCopyName, CmpDestroyHive, CmpDestroyHiveViewList, CmpDestroySecurityCache, CmpFindNameInList, CmpFindSubKeyByNumber, CmpFindValueByNameFromCache, CmpFlushNotifiesOnKeyBodyList, CmpFree, CmpFreeKeyByCell, CmpFreeValue, CmpFreeValueData, CmpGetValueKeyFromCache, CmpGetValueListFromCache, CmpInitializeHive, CmpLinkHiveToMaster, CmpLockHiveFlusherExclusive, CmpLockHiveFlusherShared, CmpLockRegistry, CmpLockRegistryExclusive, CmpMarkValueDataDirty, CmpNameSize, CmpQueryKeyValueData, CmpReleaseKcbLock, CmpReleaseTwoKcbLockByKey, CmpRemoveFromDelayedClose, CmpRemoveFromHiveFileList, CmpRemoveKeyControlBlock, CmpRemoveValueFromList, CmpReportNotify, CmpSetGlobalQuotaAllowed, CmpSetValueDataNew, CmpTryToConvertKcbSharedToExclusive, CmpUnlockHiveFlusher, CmpUnlockRegistry, ExAcquirePushLockExclusive, ExAcquirePushLockShared, ExDeleteResourceLite, ExReleasePushLock, ExReleasePushLockExclusive, HCELL_INDEX, HSTORAGE_TYPE, HV_TRACK_CELL_REF, KEY_BASIC_INFORMATION, KEY_FULL_INFORMATION, KEY_NAME_INFORMATION, KEY_NODE_INFORMATION, ObOpenObjectByName, ObReferenceObjectByHandle, PCELL_DATA, PCHILD_LIST, PCMHIVE, PCM_CACHED_VALUE, PCM_KEY_BODY, PCM_KEY_CONTROL_BLOCK, PCM_KEY_HASH, PCM_KEY_NODE, PCM_KEY_VALUE, PHCELL_INDEX, PHHIVE, PKEY_CACHED_INFORMATION, PKEY_INFORMATION, PKEY_NAME_INFORMATION, PKEY_USER_FLAGS_INFORMATION, POBJECT_ATTRIBUTES, PUNICODE_STRING, PsGetCurrentThread, RtlCompareUnicodeString, RtlEqualUnicodeString, SECURITY_CLIENT_CONTEXT, SECURITY_QUALITY_OF_SERVICE, SeCreateClientSecurity, ZwClose};
use ntkrnl_sys::ddk::{KEY_VALUE_INFORMATION_CLASS, KEY_CACHED_INFORMATION, _CELL_DATA, _CM_CACHED_VALUE, _CM_KEY_CONTROL_BLOCK, _KEY_USER_FLAGS_INFORMATION, _LARGE_INTEGER, _UNICODE_STRING as DDK_UNICODE_STRING};
use core::ffi::c_void;

use ntkrnl_sys::opaque_ptr;
// ----- Local stubs for missing symbols (FIXME: stubs added during translation cleanup) -----
// These should be moved to ntkrnl-sys when proper bindings are added.

#[allow(non_camel_case_types)]
pub type _UNICODE_STRING = ntkrnl_sys::base::UNICODE_STRING;
#[allow(non_camel_case_types)]
pub type KUSER_SHARED_DATA = u8;  // opaque - only used through pointer
#[allow(non_camel_case_types)]
pub type VALUE_SEARCH_RETURN_TYPE = u32;

// Registry constants
pub const KEY_HIVE_ENTRY: u32 = 0x00000004;
pub const KEY_NO_DELETE: u32 = 0x00000008;
pub const KEY_SYM_LINK: u32 = 0x00000010;
pub const KEY_COMP_NAME: u16 = 0x0020;
pub const REG_NOTIFY_CHANGE_NAME: u32 = 0x00000001;
pub const REG_NOTIFY_CHANGE_LAST_SET: u32 = 0x00000004;

// CM_KCB extflags
pub const CM_KCB_NO_SUBKEY: u32 = 0x0001;
pub const CM_KCB_SUBKEY_ONE: u32 = 0x0002;
pub const CM_KCB_SUBKEY_HINT: u32 = 0x0004;
pub const CM_KCB_SYM_LINK_FOUND: u32 = 0x0008;
pub const CM_KCB_INVALID_CACHED_INFO: u32 = 0x0040;
pub const CM_KCB_READ_ONLY_KEY: u32 = 0x0080;

// CmCheckRegistry flags
pub const CM_CHECK_REGISTRY_DONT_PURGE_VOLATILES: u32 = 0x00000000;
pub const CM_CHECK_REGISTRY_VALIDATE_HIVE: u32 = 0x00000008;

// Hive init types
pub const HINIT_CREATE: u32 = 0;
pub const HFILE_TYPE_PRIMARY: u32 = 0;

// HiveFlags bits come from ONE place; see crate::hiveflags.
pub use crate::hiveflags::{HIVE_IS_UNLOADING, HIVE_NOLAZYFLUSH, HIVE_VOLATILE};

// Storage types
pub const Stable: u32 = 0;
pub const Volatile: u32 = 1;

// Search results
pub const SearchSuccess: u32 = 0;
pub const SearchNeedExclusiveLock: u32 = 1;
pub const SearchFail: u32 = 2;

// SECURITY_IMPERSONATION_LEVEL value (from base SecurityImpersonation = 2)
#[allow(non_upper_case_globals)]
pub const _SECURITY_IMPERSONATION_LEVEL_SecurityImpersonation: u32 = 2;

// Symbolic link value name placeholder (reactos/ntoskrnl/config/cmdata.c)
extern "C" {
    pub static CmSymbolicLinkValueName: ntkrnl_sys::base::UNICODE_STRING;
}

// CM globals (defined in cmdata.c / cmsysini.c)
extern "C" {
    pub static mut CmpForceForceFlush: u8;
    pub static mut CmpHiveListHeadLock: u64;
    pub static mut CmpHiveListHead: ntkrnl_sys::base::LIST_ENTRY;
    pub static mut CmpKeyObjectType: ntkrnl_sys::base::PVOID;
    pub static mut CmpProfileLoaded: u8;
    pub static mut CmpWasSetupBoot: u8;
    pub static mut CmpHashTableSize: u32;
    pub static mut CmpCacheTable: ntkrnl_sys::base::PVOID;
}

// SeDeleteClientSecurity is the macro from sefuncs.h, exposed as Rust inline.
use ntkrnl_sys::inline::tier1::SeDeleteClientSecurity;
use ntkrnl_sys::ddk::RtlAssert;

// UNIMPLEMENTED_DBGBREAK: debug-only no-op stub
#[allow(non_snake_case)]
#[inline]
unsafe fn UNIMPLEMENTED_DBGBREAK(_msg: *const i8) {}

extern "C" {
    fn CmpAcquireKcbLockShared(Kcb: PCM_KEY_CONTROL_BLOCK);
    static mut CmpNoWrite: u8;
    static mut CmiVolatileHive: PCMHIVE;
    static mut CmpSpecialBootCondition: u8;
    fn CMP_ASSERT_EXCLUSIVE_REGISTRY_LOCK();
    static mut CmpLoadHiveLock: u64;
}
// Hv* are CMAPI = NTAPI = stdcall; must be extern "system".
// Exception: the dual-decoration cm shims (HvGetCell, HvReleaseCell,
// HvGetCellType, ASSERT_VALUE_BIG, CMP_ASSERT_FLUSH_LOCK,
// CmpIsKeyValueSmall, CMP_ASSERT_KCB_LOCK, CmpIsKcbLockedExclusive)
// are exported as cdecl by cm_rust_shims.c — declared in the extern "C"
// block below.
extern "system" {
    fn HvMarkCellDirty(Hive: PHHIVE, Cell: HCELL_INDEX, Shared: BOOLEAN) -> BOOLEAN;
    fn HvIsCellDirty(Hive: PHHIVE, Cell: HCELL_INDEX) -> BOOLEAN;
    fn HvHiveWillShrink(Hive: PHHIVE) -> u8;
    fn HvSyncHive(Hive: PHHIVE) -> u8;
    fn HvTrackCellRef(CellReferences: *mut HV_TRACK_CELL_REF, Hive: PHHIVE, Cell: u32) -> u8;
    fn HvReleaseFreeCellRefArray(CellReferences: *mut HV_TRACK_CELL_REF);
    fn HvFree(Hive: PHHIVE);
}
extern "system" {
    fn HvAllocateCell(Hive: PHHIVE, Size: u32, StorageType: u32, Flink: HCELL_INDEX) -> HCELL_INDEX;
    fn HvFreeCell(Hive: PHHIVE, Cell: HCELL_INDEX);
    fn HvGetCellSize(Hive: PHHIVE, CellData: PCELL_DATA) -> i32;
    fn HvReallocateCell(Hive: PHHIVE, Cell: HCELL_INDEX, Size: u32) -> HCELL_INDEX;
    static mut SharedUserData: *mut KUSER_SHARED_DATA;
    fn KeQuerySystemTime(CurrentTime: *mut LARGE_INTEGER);
    fn HvWriteHive(Hive: PHHIVE);
}
extern "C" {
    fn HvGetCell(Hive: PHHIVE, Cell: HCELL_INDEX) -> PVOID;
    fn HvReleaseCell(Hive: PHHIVE, Cell: HCELL_INDEX);
    fn HvGetCellType(Cell: HCELL_INDEX) -> u32;
    fn ASSERT_VALUE_BIG(Hive: PHHIVE, Length: u32);
    fn CMP_ASSERT_FLUSH_LOCK(Hive: PHHIVE);
    fn CmpIsKeyValueSmall(Length: *mut u32, DataLength: u32) -> u8;
    fn CMP_ASSERT_KCB_LOCK(Kcb: PCM_KEY_CONTROL_BLOCK);
    fn CmpIsKcbLockedExclusive(Kcb: PCM_KEY_CONTROL_BLOCK) -> u8;
}

#[no_mangle]
pub unsafe extern "system" fn CmpIsHiveAlreadyLoaded(
    KeyHandle: HANDLE,
    SourceFile: POBJECT_ATTRIBUTES,
    CmHive: *mut PCMHIVE,
) -> BOOLEAN {
    let KeyHandle = opaque_ptr(KeyHandle);
    let SourceFile = opaque_ptr(SourceFile);
    let CmHive = opaque_ptr(CmHive);
    let mut Status: NTSTATUS;
    let mut KeyBody: PCM_KEY_BODY = core::ptr::null_mut();
    let mut Hive: PCMHIVE;
    let mut Loaded: BOOLEAN = 0u8;

    CMP_ASSERT_EXCLUSIVE_REGISTRY_LOCK();

    Status = ObReferenceObjectByHandle(
        KeyHandle,
        0,
        CmpKeyObjectType as *mut _,
        0i32 as i8,
        &mut KeyBody as *mut _ as *mut PVOID,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return Loaded;
    }

    if (*(*KeyBody).KeyControlBlock).__bindgen_anon_1.Delete() != 0 {
        // Upstream ReactOS leaks the KeyBody reference on this deleted-KCB
        // path (cmapi.c: bare `return Loaded;` while every other exit derefs)
        // — found by scan_ob_refcount.py; release it like the other exits do.
        ObDereferenceObject(KeyBody as PVOID);
        return Loaded;
    }

    Hive = (((*(*KeyBody).KeyControlBlock).__bindgen_anon_2.__bindgen_anon_1.KeyHive as *mut u8)
        .offset(-(core::mem::offset_of!(CMHIVE, Hive) as isize))) as PCMHIVE;

    if ((*(*KeyBody).KeyControlBlock).__bindgen_anon_5.Flags() & KEY_HIVE_ENTRY) == 0
        || (*Hive).FileUserName.Buffer.is_null()
    {
        ObDereferenceObject(KeyBody as PVOID);
        return Loaded;
    }

    if RtlCompareUnicodeString(
        &(*Hive).FileUserName,
        (*SourceFile).ObjectName,
        1u8,
    ) == 0
    {
        Loaded = 1u8;
        *CmHive = Hive;

        if (*Hive).Frozen != 0 {
            UNIMPLEMENTED_DBGBREAK(b"ERROR: Hive is frozen\n\0".as_ptr() as PCHAR);
        }
    }

    ObDereferenceObject(KeyBody as PVOID);
    return Loaded;
}

#[no_mangle]
pub unsafe extern "system" fn CmpDoFlushAll(ForceFlush: u8) -> u8 {
    let mut NextEntry: PLIST_ENTRY;
    let mut Hive: PCMHIVE;
    let mut Status: i32;
    let mut Result: u8 = 1u8;

    if CmpNoWrite != 0 {
        return 1u8;
    }

    CmpForceForceFlush = 0u8;
    ExAcquirePushLockShared(core::ptr::addr_of_mut!(CmpHiveListHeadLock) as *mut _);

    NextEntry = CmpHiveListHead.Flink;
    while NextEntry != core::ptr::addr_of_mut!(CmpHiveListHead) {
        Hive = (NextEntry as *mut u8).offset(-(core::mem::offset_of!(CMHIVE, HiveList) as isize)) as PCMHIVE;
        // C: `!(Hive->Hive.HiveFlags & HIVE_NOLAZYFLUSH)`, which is 0x2. The
        // literal here was 0x1 — HIVE_VOLATILE — so the lazy flusher skipped
        // every VOLATILE hive (which has nothing to flush anyway) and happily
        // flushed the hives that asked it not to.
        if ((*Hive).Hive.HiveFlags & crate::hiveflags::HIVE_NOLAZYFLUSH) == 0 {
            CmpLockHiveFlusherExclusive(Hive);

            if ForceFlush != 0 && (*Hive).UseCount != 0 {
                CMP_ASSERT_EXCLUSIVE_REGISTRY_LOCK();
                UNIMPLEMENTED_DBGBREAK(b"FIXME: Hive is damaged and needs fixup\n\0".as_ptr() as *const i8);
            }

            if ForceFlush != 0 || HvHiveWillShrink(core::ptr::addr_of_mut!((*Hive).Hive)) == 0 {
                Status = HvSyncHive(core::ptr::addr_of_mut!((*Hive).Hive)) as i32;

                if !NT_SUCCESS(Status) {
                    Result = 0u8;
                }
            } else {
                Result = 0u8;
                CmpForceForceFlush = 1u8;
            }

            CmpUnlockHiveFlusher(Hive);
        }

        NextEntry = (*NextEntry).Flink;
    }

    ExReleasePushLock(core::ptr::addr_of_mut!(CmpHiveListHeadLock) as *mut _);
    return Result;
}

// The C wraps the CmpCopyName below in _SEH2_TRY/_SEH2_EXCEPT, whose handler reports
// DPRINT1("Invalid user data!"), frees the cell and returns the exception code — the C
// comment on it says "this can crash since the name is coming from user-mode."
//
// That handler is NOT reachable here, and adding a print for it would be a lie about
// where this function's input comes from. ValueName is kernel-pool memory by the time
// it arrives: the one live caller is CmSetValueKey <- NtSetValueKey, which captures the
// string with ProbeAndCaptureUnicodeString (ntkrnl-sys/src/inline/common.rs) — for a
// user-mode caller that probes the header AND the buffer and copies both into pool, so
// no user page is touched from here. A KernelMode caller hands us its own kernel
// buffer, and a driver passing a bad one bugchecks with or without this handler,
// because DUMMY_PSEH means there is nothing to catch it (dummy-pseh-verdict.md).
//
// So the honest translation of the SEH region is its guarded statement alone. The place
// that DID need the check is NtSetValueKey's Data path, where the probe really had been
// dropped — see the comment there. nolint: dropped-print
unsafe fn CmpSetValueKeyNew(
    Hive: PHHIVE,
    Parent: PCM_KEY_NODE,
    ValueName: PUNICODE_STRING,
    Index: u32,
    Type: u32,
    Data: *mut core::ffi::c_void,
    DataSize: u32,
    StorageType: u32,
    SmallData: u32,
) -> i32 {
    let mut CellData: PCELL_DATA;
    let mut ValueCell: HCELL_INDEX;
    let mut Status: i32;

    if (*Parent).__bindgen_anon_1.__bindgen_anon_1.ValueList.Count != 0 {
        if (*Parent).__bindgen_anon_1.__bindgen_anon_1.ValueList.List == 0xFFFFFFFF {
            RtlAssert(
                b"Parent->ValueList.List != HCELL_NIL\0".as_ptr() as *mut core::ffi::c_void,
                b"cmapi.rs\0".as_ptr() as *mut core::ffi::c_void,
                line!(),
                core::ptr::null_mut::<i8>(),
            );
        }
        if HvMarkCellDirty(Hive, (*Parent).__bindgen_anon_1.__bindgen_anon_1.ValueList.List, 0u8) == 0 {
            return STATUS_NO_LOG_SPACE;
        }
    }

    ValueCell = HvAllocateCell(
        Hive,
        core::mem::offset_of!(CM_KEY_VALUE, Name) as u32 + CmpNameSize(Hive, ValueName) as u32,
        StorageType,
        0xFFFFFFFF,
    );
    if ValueCell == 0xFFFFFFFF {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    CellData = HvGetCell(Hive, ValueCell) as PCELL_DATA;
    if CellData.is_null() {
        RtlAssert(
            b"CellData\0".as_ptr() as *mut core::ffi::c_void,
            b"cmapi.rs\0".as_ptr() as *mut core::ffi::c_void,
            line!(),
            core::ptr::null_mut::<i8>(),
        );
    }

    HvReleaseCell(Hive, ValueCell);
    if HvIsCellDirty(Hive, ValueCell) == 0 {
        RtlAssert(
            b"HvIsCellDirty(Hive, ValueCell)\0".as_ptr() as *mut core::ffi::c_void,
            b"cmapi.rs\0".as_ptr() as *mut core::ffi::c_void,
            line!(),
            core::ptr::null_mut::<i8>(),
        );
    }

    (*CellData).u.KeyValue.Signature = 0x6B76u16;

    {
        let kv = core::ptr::addr_of_mut!((*CellData).u.KeyValue);
        let name_ptr = core::ptr::addr_of_mut!((*kv).Name) as *mut u16;
        let copied = CmpCopyName(Hive, name_ptr, ValueName);
        core::ptr::write_unaligned(core::ptr::addr_of_mut!((*kv).NameLength), copied);
    }

    if (*CellData).u.KeyValue.NameLength < (*ValueName).Length {
        (*CellData).u.KeyValue.Flags = 0x0001u16;
    } else {
        (*CellData).u.KeyValue.Flags = 0;
    }

    if DataSize > CM_KEY_VALUE_SMALL {
        Status = CmpSetValueDataNew(
            Hive,
            Data,
            DataSize,
            StorageType,
            ValueCell,
            core::ptr::addr_of_mut!((*CellData).u.KeyValue.Data),
        );
        if !NT_SUCCESS(Status) {
            HvFreeCell(Hive, ValueCell);
            return Status;
        }

        (*CellData).u.KeyValue.DataLength = DataSize;
        if HvIsCellDirty(Hive, (*CellData).u.KeyValue.Data) == 0 {
            RtlAssert(
                b"HvIsCellDirty(Hive, CellData->u.KeyValue.Data)\0".as_ptr() as *mut core::ffi::c_void,
                b"cmapi.rs\0".as_ptr() as *mut core::ffi::c_void,
                line!(),
                core::ptr::null_mut::<i8>(),
            );
        }
    } else {
        (*CellData).u.KeyValue.DataLength = DataSize + 0x80000000u32;
        (*CellData).u.KeyValue.Data = SmallData;
    }

    (*CellData).u.KeyValue.Type = Type;

    Status = CmpAddValueToList(
        Hive,
        ValueCell,
        Index,
        StorageType,
        core::ptr::addr_of_mut!((*Parent).__bindgen_anon_1.__bindgen_anon_1.ValueList),
    );

    if !NT_SUCCESS(Status) {
        CmpFreeValue(Hive, ValueCell);
        Status = STATUS_INSUFFICIENT_RESOURCES;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn CmpSetValueKeyExisting(
    Hive: PHHIVE,
    OldChild: HCELL_INDEX,
    Value: PCM_KEY_VALUE,
    Type: u32,
    Data: *mut core::ffi::c_void,
    DataSize: u32,
    StorageType: u32,
    TempData: u32,
) -> i32 {
    let Hive = opaque_ptr(Hive);
    let Value = opaque_ptr(Value);
    let Data = opaque_ptr(Data);
    let mut DataCell: HCELL_INDEX;
    let mut NewCell: HCELL_INDEX;
    let mut CellData: PCELL_DATA;
    let mut Length: u32 = 0;
    let WasSmall: u8;
    let IsSmall: u8;

    CMP_ASSERT_FLUSH_LOCK(Hive);

    if HvMarkCellDirty(Hive, OldChild, 0u8) == 0 {
        return STATUS_NO_LOG_SPACE;
    }

    WasSmall = CmpIsKeyValueSmall(&mut Length as *mut u32, (*Value).DataLength);

    IsSmall = if DataSize <= CM_KEY_VALUE_SMALL { 1u8 } else { 0u8 };

    ASSERT_VALUE_BIG(Hive, Length);
    ASSERT_VALUE_BIG(Hive, DataSize);

    if CmpMarkValueDataDirty(Hive, Value) == 0 {
        return STATUS_NO_LOG_SPACE;
    }

    if IsSmall != 0 {
        if WasSmall == 0 && Length > 0 {
            CmpFreeValueData(Hive, (*Value).Data, Length);
        }

        (*Value).DataLength = DataSize + CM_KEY_VALUE_SPECIAL_SIZE;
        (*Value).Data = TempData;
        (*Value).Type = Type;
        return STATUS_SUCCESS;
    }

    if WasSmall == 0 && Length > 0 {
        DataCell = (*Value).Data;
        CellData = HvGetCell(Hive, DataCell) as PCELL_DATA;
        if CellData.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        HvReleaseCell(Hive, DataCell);

        if DataSize <= HvGetCellSize(Hive, CellData) as u32 {
            NewCell = DataCell;
        } else {
            NewCell = HvReallocateCell(Hive, DataCell, DataSize);
            if NewCell == HCELL_NIL {
                return STATUS_INSUFFICIENT_RESOURCES;
            }
        }
    } else {
        NewCell = HvAllocateCell(Hive, DataSize, StorageType, HCELL_NIL);
        if NewCell == HCELL_NIL {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
    }

    CellData = HvGetCell(Hive, NewCell) as PCELL_DATA;

    HvReleaseCell(Hive, NewCell);

    core::ptr::copy_nonoverlapping(Data as *const u8, CellData as *mut u8, DataSize as usize);
    (*Value).Data = NewCell;
    (*Value).DataLength = DataSize;
    (*Value).Type = Type;

    return STATUS_SUCCESS;
}

const STATUS_NO_LOG_SPACE: i32 = 0xC000017D_u32 as i32;
const CM_KEY_VALUE_SMALL: u32 = 4;
const CM_KEY_VALUE_SPECIAL_SIZE: u32 = 0x80000000;
const HCELL_NIL: HCELL_INDEX = 0xFFFFFFFF;

#[inline(always)]
unsafe fn CMP_IS_CELL_CACHED(value: HCELL_INDEX) -> BOOLEAN {
    if (value & 1) != 0 && value != HCELL_NIL { 1 } else { 0 }
}

unsafe fn CmpQueryKeyData(
    Hive: PHHIVE,
    Node: PCM_KEY_NODE,
    KeyInformationClass: u32,
    KeyInformation: *mut core::ffi::c_void,
    Length: u32,
    ResultLength: *mut u32,
) -> i32 {
    let mut Status: i32;
    let mut Size: u32;
    let mut SizeLeft: u32;
    let mut MinimumSize: u32;
    let mut Offset: u32;
    let Info: PKEY_INFORMATION = KeyInformation as PKEY_INFORMATION;
    let NameLength: u16;
    let mut ClassData: *mut core::ffi::c_void;

    if (*Node).Flags & 0x0020 != 0 {
        NameLength = CmpCompressedNameSize(core::ptr::addr_of!((*Node).Name) as *mut u16, (*Node).NameLength as u32) as u16;
    } else {
        NameLength = (*Node).NameLength;
    }

    match KeyInformationClass {
        0 => {
            Size = core::mem::offset_of!(KEY_BASIC_INFORMATION, Name) as u32 + NameLength as u32;
            MinimumSize = core::mem::offset_of!(KEY_BASIC_INFORMATION, Name) as u32;
            *ResultLength = Size;
            Status = STATUS_SUCCESS;

            if Length < MinimumSize {
                Status = STATUS_BUFFER_TOO_SMALL;
            } else {
                (*Info).__bindgen_anon_1.KeyBasicInformation.LastWriteTime = (*Node).LastWriteTime;
                (*Info).__bindgen_anon_1.KeyBasicInformation.TitleIndex = 0;
                (*Info).__bindgen_anon_1.KeyBasicInformation.NameLength = NameLength as u32;

                SizeLeft = Length - MinimumSize;
                Size = NameLength as u32;

                if SizeLeft < Size {
                    Size = SizeLeft;
                    Status = 0x80000005u32 as i32;
                }

                if (*Node).Flags & 0x0020 != 0 {
                    CmpCopyCompressedName(
                        (*Info).__bindgen_anon_1.KeyBasicInformation.Name.as_mut_ptr(),
                        SizeLeft,
                        core::ptr::addr_of!((*Node).Name) as *mut u16,
                        (*Node).NameLength as u32,
                    );
                } else {
                    RtlCopyMemory(
                        (*Info).__bindgen_anon_1.KeyBasicInformation.Name.as_mut_ptr() as *mut core::ffi::c_void,
                        core::ptr::addr_of!((*Node).Name) as *const core::ffi::c_void,
                        Size as usize,
                    );
                }
            }
        }
        1 => {
            Size = core::mem::offset_of!(KEY_NODE_INFORMATION, Name) as u32
                + NameLength as u32
                + (*Node).ClassLength as u32;
            MinimumSize = core::mem::offset_of!(KEY_NODE_INFORMATION, Name) as u32;
            *ResultLength = Size;
            Status = STATUS_SUCCESS;

            if Length < MinimumSize {
                Status = STATUS_BUFFER_TOO_SMALL;
            } else {
                (*Info).__bindgen_anon_1.KeyNodeInformation.LastWriteTime = (*Node).LastWriteTime;
                (*Info).__bindgen_anon_1.KeyNodeInformation.TitleIndex = 0;
                (*Info).__bindgen_anon_1.KeyNodeInformation.ClassLength = (*Node).ClassLength as u32;
                (*Info).__bindgen_anon_1.KeyNodeInformation.NameLength = NameLength as u32;

                SizeLeft = Length - MinimumSize;
                Size = NameLength as u32;

                if SizeLeft < Size {
                    Size = SizeLeft;
                    Status = 0x80000005u32 as i32;
                }

                if (*Node).Flags & 0x0020 != 0 {
                    CmpCopyCompressedName(
                        (*Info).__bindgen_anon_1.KeyNodeInformation.Name.as_mut_ptr(),
                        SizeLeft,
                        core::ptr::addr_of!((*Node).Name) as *mut u16,
                        (*Node).NameLength as u32,
                    );
                } else {
                    RtlCopyMemory(
                        (*Info).__bindgen_anon_1.KeyNodeInformation.Name.as_mut_ptr() as *mut core::ffi::c_void,
                        core::ptr::addr_of!((*Node).Name) as *const core::ffi::c_void,
                        Size as usize,
                    );
                }

                if (*Node).ClassLength > 0 {
                    Offset = core::mem::offset_of!(KEY_NODE_INFORMATION, Name) as u32 + NameLength as u32;
                    Offset = ALIGN_UP_BY(Offset as usize, core::mem::size_of::<u32>()) as u32;
                    (*Info).__bindgen_anon_1.KeyNodeInformation.ClassOffset = Offset;

                    ClassData = HvGetCell(Hive, (*Node).Class);
                    if ClassData.is_null() {
                        Status = STATUS_INSUFFICIENT_RESOURCES;
                    } else {
                        if Length > Offset {
                            let class_len_u32: u32 = (*Node).ClassLength as u32;
                            let copy_size = if class_len_u32 < Length - Offset {
                                class_len_u32
                            } else {
                                Length - Offset
                            };
                            RtlCopyMemory(
                                (Info as *mut u8).add(Offset as usize) as *mut core::ffi::c_void,
                                ClassData,
                                copy_size as usize,
                            );
                        }

                        if Length < Offset + (*Node).ClassLength as u32 {
                            Status = 0x80000005u32 as i32;
                        }

                        HvReleaseCell(Hive, (*Node).Class);
                    }
                } else {
                    (*Info).__bindgen_anon_1.KeyNodeInformation.ClassOffset = 0xFFFFFFFF;
                }
            }
        }
        2 => {
            Size = core::mem::offset_of!(KEY_FULL_INFORMATION, Class) as u32 + (*Node).ClassLength as u32;
            MinimumSize = core::mem::offset_of!(KEY_FULL_INFORMATION, Class) as u32;
            *ResultLength = Size;
            Status = STATUS_SUCCESS;

            if Length < MinimumSize {
                Status = STATUS_BUFFER_TOO_SMALL;
            } else {
                (*Info).__bindgen_anon_1.KeyFullInformation.LastWriteTime = (*Node).LastWriteTime;
                (*Info).__bindgen_anon_1.KeyFullInformation.TitleIndex = 0;
                (*Info).__bindgen_anon_1.KeyFullInformation.ClassLength = (*Node).ClassLength as u32;
                (*Info).__bindgen_anon_1.KeyFullInformation.SubKeys = (*Node).SubKeyCounts[0] + (*Node).SubKeyCounts[1];
                (*Info).__bindgen_anon_1.KeyFullInformation.Values = (*Node).__bindgen_anon_1.__bindgen_anon_1.ValueList.Count;
                (*Info).__bindgen_anon_1.KeyFullInformation.MaxNameLen = (*Node).MaxNameLen;
                (*Info).__bindgen_anon_1.KeyFullInformation.MaxClassLen = (*Node).MaxClassLen;
                (*Info).__bindgen_anon_1.KeyFullInformation.MaxValueNameLen = (*Node).MaxValueNameLen;
                (*Info).__bindgen_anon_1.KeyFullInformation.MaxValueDataLen = (*Node).MaxValueDataLen;

                if (*Node).ClassLength > 0 {
                    Offset = core::mem::offset_of!(KEY_FULL_INFORMATION, Class) as u32;
                    (*Info).__bindgen_anon_1.KeyFullInformation.ClassOffset = Offset;

                    ClassData = HvGetCell(Hive, (*Node).Class);
                    if ClassData.is_null() {
                        Status = STATUS_INSUFFICIENT_RESOURCES;
                    } else {
                        let class_len_u32: u32 = (*Node).ClassLength as u32;
                        let copy_size = if class_len_u32 < Length - Offset {
                            class_len_u32
                        } else {
                            Length - Offset
                        };
                        RtlCopyMemory(
                            (*Info).__bindgen_anon_1.KeyFullInformation.Class.as_mut_ptr() as *mut core::ffi::c_void,
                            ClassData,
                            copy_size as usize,
                        );

                        if Length < Offset + (*Node).ClassLength as u32 {
                            Status = 0x80000005u32 as i32;
                        }

                        HvReleaseCell(Hive, (*Node).Class);
                    }
                } else {
                    (*Info).__bindgen_anon_1.KeyFullInformation.ClassOffset = 0xFFFFFFFF;
                }
            }
        }
        _ => {
            Status = 0xC000000Du32 as i32;
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn CmSetValueKey(
    Kcb: PCM_KEY_CONTROL_BLOCK,
    ValueName: PUNICODE_STRING,
    Type: u32,
    Data: *mut core::ffi::c_void,
    DataLength: u32,
) -> i32 {
    let Kcb = opaque_ptr(Kcb);
    let ValueName = opaque_ptr(ValueName);
    let Data = opaque_ptr(Data);
    let mut Hive: PHHIVE = core::ptr::null_mut();
    let mut Parent: PCM_KEY_NODE;
    let mut Value: PCM_KEY_VALUE = core::ptr::null_mut();
    let mut CurrentChild: HCELL_INDEX = HCELL_NIL;
    let mut Cell: HCELL_INDEX;
    let mut Status: i32;
    let mut Found: u8;
    let mut Result: u8;
    let mut Count: u32;
    let mut ChildIndex: u32 = 0;
    let mut SmallData: u32;
    let mut Storage: u32;
    let mut SearchResult: u32;
    let mut FirstTry: u8 = 1;
    let mut FlusherLocked: u8 = 0;
    let mut ParentCell: HCELL_INDEX = 0xffffffff;
    let mut ChildCell: HCELL_INDEX = 0xffffffff;

    CmpLockRegistry();
    CmpAcquireKcbLockShared(Kcb);

    // Initialize Status / Cell / Parent / Found to satisfy the Rust borrow checker
    // (C code relied on flow-sensitive analysis but the Quickie loop control flow
    // now needs explicit init).
    Status = STATUS_SUCCESS;
    Cell = 0;
    Parent = core::ptr::null_mut();
    Found = 0;

    'Quickie: loop {
        'DoAgain: loop {
            if (*Kcb).__bindgen_anon_1.Delete() != 0 {
                Status = 0xC000017Cu32 as i32;
                break 'Quickie;
            }

            // C: `Kcb->Flags & KEY_SYM_LINK`. KEY_SYM_LINK is 0x0010
            // (sdk/lib/cmlib/cmdata.h); the literal here was 0x0004, which is
            // KEY_HIVE_ENTRY — so "don't let anyone mess with symlinks" guarded
            // HIVE ROOTS (\Registry\Machine\System, ...\Software, ...\Sam,
            // ...\Security, \Registry\User\.Default) and left symlink keys
            // unguarded. Setting any value on a hive root returned ACCESS_DENIED.
            if ((*Kcb).__bindgen_anon_5.Flags() & KEY_SYM_LINK) != 0 &&
                ((Type != 6) ||
                 ValueName.is_null() ||
                 RtlEqualUnicodeString(
                     core::ptr::addr_of!(CmSymbolicLinkValueName) as PUNICODE_STRING,
                     ValueName,
                     1
                 ) == 0)
            {
                Status = 0xC0000022u32 as i32;
                break 'Quickie;
            }

            if FirstTry != 0 {
                SearchResult = CmpCompareNewValueDataAgainstKCBCache(
                    Kcb,
                    ValueName,
                    Type,
                    Data,
                    DataLength
                );
                if SearchResult == SearchNeedExclusiveLock {
                    CmpConvertKcbSharedToExclusive(Kcb);
                    continue 'DoAgain;
                } else if SearchResult == SearchSuccess {
                    Status = STATUS_SUCCESS;
                    break 'Quickie;
                }

                if CmpIsKcbLockedExclusive(Kcb) == 0 &&
                    CmpTryToConvertKcbSharedToExclusive(Kcb) == 0
                {
                    CmpConvertKcbSharedToExclusive(Kcb);
                }

                FirstTry = 0;

                CmpLockHiveFlusherShared((*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyHive as PCMHIVE);
                FlusherLocked = 1;
                continue 'DoAgain;
            } else {
                Hive = (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyHive;
                Cell = (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyCell;

                Parent = HvGetCell(Hive, Cell) as PCM_KEY_NODE;
                ParentCell = Cell;

                Count = (*Parent).__bindgen_anon_1.__bindgen_anon_1.ValueList.Count;
                Found = 0;
                if Count > 0 {
                    Result = CmpFindNameInList(
                        Hive,
                        core::ptr::addr_of_mut!((*Parent).__bindgen_anon_1.__bindgen_anon_1.ValueList),
                        ValueName,
                        core::ptr::addr_of_mut!(ChildIndex),
                        core::ptr::addr_of_mut!(CurrentChild)
                    );
                    if Result == 0 {
                        Status = STATUS_INSUFFICIENT_RESOURCES;
                        break 'Quickie;
                    }

                    if CurrentChild != 0xffffffff {
                        if ChildCell != 0xffffffff {
                            HvReleaseCell(Hive, ChildCell);
                            ChildCell = 0xffffffff;
                        }

                        Value = HvGetCell(Hive, CurrentChild) as PCM_KEY_VALUE;
                        if Value.is_null() {
                            Status = STATUS_INSUFFICIENT_RESOURCES;
                            break 'Quickie;
                        }

                        ChildCell = CurrentChild;
                        Found = 1;
                    }
                } else {
                    ChildIndex = 0;
                }
            }

            break;
        }

        CMP_ASSERT_KCB_LOCK(Kcb);

        if HvMarkCellDirty(Hive, Cell, 0) == 0 {
            Status = STATUS_NO_LOG_SPACE;
            break 'Quickie;
        }

        Storage = HvGetCellType(Cell);

        SmallData = 0;
        if DataLength <= 4 && DataLength > 0 {
            core::ptr::copy_nonoverlapping(Data as *const u8, core::ptr::addr_of_mut!(SmallData) as *mut u8, DataLength as usize);
        }

        if Found == 0 {
            Status = CmpSetValueKeyNew(
                Hive,
                Parent,
                ValueName,
                ChildIndex,
                Type,
                Data,
                DataLength,
                Storage,
                SmallData
            );
        } else {
            Status = CmpSetValueKeyExisting(
                Hive,
                CurrentChild,
                Value,
                Type,
                Data,
                DataLength,
                Storage,
                SmallData
            );
        }

        if NT_SUCCESS(Status) {
            if ((*Parent).MaxValueNameLen as u32) < ((*ValueName).Length as u32) {
                (*Parent).MaxValueNameLen = (*ValueName).Length as u32;
                (*Kcb).KcbMaxValueNameLen = (*ValueName).Length;
            }

            if (*Parent).MaxValueDataLen < DataLength {
                (*Parent).MaxValueDataLen = DataLength;
                (*Kcb).KcbMaxValueDataLen = (*Parent).MaxValueDataLen;
            }

            KeQuerySystemTime(core::ptr::addr_of_mut!((*Parent).LastWriteTime) as *mut _);
            (*Kcb).KcbLastWriteTime = (*Parent).LastWriteTime;

            if Found != 0 && CMP_IS_CELL_CACHED((*Kcb).ValueCache.__bindgen_anon_1.ValueList) != 0 {
            } else {
                CmpCleanUpKcbValueCache(Kcb);

                (*Kcb).ValueCache.Count = (*Parent).__bindgen_anon_1.__bindgen_anon_1.ValueList.Count;
                (*Kcb).ValueCache.__bindgen_anon_1.ValueList = (*Parent).__bindgen_anon_1.__bindgen_anon_1.ValueList.List;
            }

            CmpReportNotify(
                Kcb,
                Hive,
                (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyCell,
                0x00000004
            );
        }

        break 'Quickie;
    }

    if ParentCell != 0xffffffff && !Hive.is_null() {
        HvReleaseCell(Hive, ParentCell);
    }
    if ChildCell != 0xffffffff && !Hive.is_null() {
        HvReleaseCell(Hive, ChildCell);
    }

    if FlusherLocked != 0 {
        CmpUnlockHiveFlusher(Hive as PCMHIVE);
    }
    CmpReleaseKcbLock(Kcb);
    CmpUnlockRegistry();
    Status
}

#[no_mangle]
pub unsafe extern "system" fn CmDeleteValueKey(
    Kcb: PCM_KEY_CONTROL_BLOCK,
    ValueName: _UNICODE_STRING,
) -> NTSTATUS {
    let Kcb = opaque_ptr(Kcb);

    let mut Status: NTSTATUS = STATUS_OBJECT_NAME_NOT_FOUND;
    let mut Hive: PHHIVE;
    let mut Parent: PCM_KEY_NODE;
    let mut ChildCell: HCELL_INDEX = HCELL_NIL;
    let Cell: HCELL_INDEX;
    let mut ChildList: PCHILD_LIST;
    let mut Value: PCM_KEY_VALUE = core::ptr::null_mut();
    let mut ChildIndex: ULONG = 0;
    let mut Result: BOOLEAN;

    CmpLockRegistry();
    CmpAcquireKcbLockExclusive(Kcb);

    if (*Kcb).__bindgen_anon_1.Delete() != 0 {
        CmpReleaseKcbLock(Kcb);
        CmpUnlockRegistry();
        return STATUS_KEY_DELETED;
    }

    Hive = (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyHive;
    Cell = (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyCell;

    CmpLockHiveFlusherShared(Hive as PCMHIVE);

    Parent = HvGetCell(Hive, Cell) as PCM_KEY_NODE;

    ChildList = core::ptr::addr_of_mut!((*Parent).__bindgen_anon_1.__bindgen_anon_1.ValueList);
    let mut LocalValueName = ValueName;
    if (*ChildList).Count != 0 {
        Result = CmpFindNameInList(
            Hive,
            ChildList,
            core::ptr::addr_of_mut!(LocalValueName) as PUNICODE_STRING,
            core::ptr::addr_of_mut!(ChildIndex),
            core::ptr::addr_of_mut!(ChildCell),
        );
        if Result == 0 {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            // goto Quickie
        } else if ChildCell == HCELL_NIL {
            // goto Quickie
        } else {
            if HvMarkCellDirty(Hive, Cell, 0u8) != 0
                && HvMarkCellDirty(Hive, (*Parent).__bindgen_anon_1.__bindgen_anon_1.ValueList.List, 0u8) != 0
                && HvMarkCellDirty(Hive, ChildCell, 0u8) != 0
            {
                Value = HvGetCell(Hive, ChildCell) as PCM_KEY_VALUE;

                if CmpMarkValueDataDirty(Hive, Value) != 0 {
                    Status = CmpRemoveValueFromList(Hive, ChildIndex, ChildList);
                    if !NT_SUCCESS(Status) {
                        Status = STATUS_INSUFFICIENT_RESOURCES;
                    } else {
                        if CmpFreeValue(Hive, ChildCell) == 0 {
                            Status = STATUS_INSUFFICIENT_RESOURCES;
                        } else {
                            let mut SystemTime: LARGE_INTEGER = core::mem::zeroed();
                            core::ptr::copy_nonoverlapping(
                                KeGetSystemTime(core::ptr::addr_of_mut!(SystemTime)) as *const _,
                                core::ptr::addr_of_mut!((*Parent).LastWriteTime),
                                1,
                            );
                            (*Kcb).KcbLastWriteTime = (*Parent).LastWriteTime;

                            if (*Parent).__bindgen_anon_1.__bindgen_anon_1.ValueList.Count == 0 {
                                (*Parent).MaxValueNameLen = 0;
                                (*Parent).MaxValueDataLen = 0;
                                (*Kcb).KcbMaxValueNameLen = 0;
                                (*Kcb).KcbMaxValueDataLen = 0;
                            }

                            CmpCleanUpKcbValueCache(Kcb);

                            (*Kcb).ValueCache.Count = (*ChildList).Count;
                            (*Kcb).ValueCache.__bindgen_anon_1.ValueList = (*ChildList).List;

                            CmpReportNotify(Kcb, Hive, Cell, REG_NOTIFY_CHANGE_LAST_SET);

                            Status = STATUS_SUCCESS;
                        }
                    }
                } else {
                    Status = STATUS_NO_LOG_SPACE;
                }
            } else {
                Status = STATUS_NO_LOG_SPACE;
            }
        }
    }

    if !Parent.is_null() {
        HvReleaseCell(Hive, Cell);
    }

    if !Value.is_null() {
        HvReleaseCell(Hive, ChildCell);
    }

    CmpUnlockHiveFlusher(Hive as PCMHIVE);
    CmpReleaseKcbLock(Kcb);
    CmpUnlockRegistry();
    Status
}

#[inline]
unsafe fn KeGetSystemTime(CurrentTime: *mut LARGE_INTEGER) -> *mut LARGE_INTEGER {
    // SharedUserData->SystemTime is a tick-counter union read; on this build the
    // KUSER_SHARED_DATA layout is a stub. Use KeQuerySystemTime as a portable
    // fallback and fill the LARGE_INTEGER in QuadPart-form.
    let _ = SharedUserData; // suppress unused-warning if stub is unused
    KeQuerySystemTime(CurrentTime as *mut _);
    CurrentTime
}

#[no_mangle]
pub unsafe extern "system" fn CmQueryValueKey(
    Kcb: PCM_KEY_CONTROL_BLOCK,
    ValueName: _UNICODE_STRING,
    KeyValueInformationClass: KEY_VALUE_INFORMATION_CLASS,
    KeyValueInformation: PVOID,
    Length: ULONG,
    ResultLength: PULONG,
) -> NTSTATUS {
    let Kcb = opaque_ptr(Kcb);
    let KeyValueInformation = opaque_ptr(KeyValueInformation);
    let ResultLength = opaque_ptr(ResultLength);
    let mut Status: NTSTATUS = STATUS_OBJECT_NAME_NOT_FOUND;
    let mut ValueData: PCM_KEY_VALUE = core::ptr::null_mut();
    let mut Index: ULONG = 0;
    let mut ValueCached: BOOLEAN = 0u8;
    let mut CachedValue: *mut PCM_CACHED_VALUE = core::ptr::null_mut();
    let mut CellToRelease: HCELL_INDEX = HCELL_NIL;
    let mut Result: VALUE_SEARCH_RETURN_TYPE;
    let mut Hive: PHHIVE = core::ptr::null_mut();

    CmpLockRegistry();

    CmpAcquireKcbLockShared(Kcb);

    let LocalValueName = ValueName;
    'DoAgain: loop {
        if (*Kcb).__bindgen_anon_1.Delete() != 0 {
            CmpReleaseKcbLock(Kcb);
            CmpUnlockRegistry();
            return STATUS_KEY_DELETED;
        }

        if ((*Kcb).__bindgen_anon_1.ExtFlags() & CM_KCB_SYM_LINK_FOUND) != 0 {
            // ASSERT(FALSE);
        }

        Hive = (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyHive;

        Result = CmpFindValueByNameFromCache(
            Kcb,
            core::ptr::addr_of!(LocalValueName) as *const _ as PUNICODE_STRING,
            core::ptr::addr_of_mut!(CachedValue),
            core::ptr::addr_of_mut!(Index),
            core::ptr::addr_of_mut!(ValueData),
            core::ptr::addr_of_mut!(ValueCached),
            core::ptr::addr_of_mut!(CellToRelease),
        );

        if Result == SearchNeedExclusiveLock {
            CmpConvertKcbSharedToExclusive(Kcb);
            continue 'DoAgain;
        }

        if Result == SearchSuccess {
            Result = CmpQueryKeyValueData(
                Kcb,
                CachedValue,
                ValueData,
                ValueCached,
                KeyValueInformationClass,
                KeyValueInformation,
                Length,
                ResultLength,
                core::ptr::addr_of_mut!(Status),
            );
            
            if Result == SearchNeedExclusiveLock {
                if CellToRelease != HCELL_NIL {
                    HvReleaseCell(Hive, CellToRelease);
                    CellToRelease = HCELL_NIL;
                }

                CmpConvertKcbSharedToExclusive(Kcb);
                continue 'DoAgain;
            }
        } else {
            Status = STATUS_OBJECT_NAME_NOT_FOUND;
        }

        break;
    }

    if CellToRelease != HCELL_NIL {
        HvReleaseCell(Hive, CellToRelease);
    }

    CmpReleaseKcbLock(Kcb);
    CmpUnlockRegistry();

    Status
}

#[no_mangle]
pub unsafe extern "system" fn CmEnumerateValueKey(
    Kcb: PCM_KEY_CONTROL_BLOCK,
    Index: u32,
    KeyValueInformationClass: i32,
    KeyValueInformation: *mut core::ffi::c_void,
    Length: u32,
    ResultLength: *mut u32,
) -> i32 {
    let Kcb = opaque_ptr(Kcb);
    let KeyValueInformation = opaque_ptr(KeyValueInformation);
    let ResultLength = opaque_ptr(ResultLength);
    let mut Status: i32 = STATUS_SUCCESS;
    let mut Hive: PHHIVE = core::ptr::null_mut();
    let mut Parent: PCM_KEY_NODE = core::ptr::null_mut();
    let mut CellToRelease: HCELL_INDEX = 0xFFFFFFFF;
    let mut CellToRelease2: HCELL_INDEX = 0xFFFFFFFF;
    let mut Result: u32;
    let mut IndexIsCached: u8 = 0;
    let mut ValueIsCached: u8 = 0u8;
    let mut CellData: PCELL_DATA = core::ptr::null_mut();
    let mut CachedValue: *mut PCM_CACHED_VALUE = core::ptr::null_mut();
    let mut ValueData: PCM_KEY_VALUE = core::ptr::null_mut();

    CmpLockRegistry();
    CmpAcquireKcbLockShared(Kcb);

    'DoAgain: loop {
        if (*Kcb).__bindgen_anon_1.Delete() != 0 {
            CmpReleaseKcbLock(Kcb);
            CmpUnlockRegistry();
            return STATUS_KEY_DELETED;
        }

        Hive = (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyHive;
        Parent = HvGetCell(Hive, (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyCell) as PCM_KEY_NODE;

        if (*Kcb).ValueCache.Count != (*Parent).__bindgen_anon_1.__bindgen_anon_1.ValueList.Count {
            crate::dbg_print!(b"HACK: Overriding value cache count\n\0");
            (*Kcb).ValueCache.Count = (*Parent).__bindgen_anon_1.__bindgen_anon_1.ValueList.Count;
        }

        if Index >= (*Kcb).ValueCache.Count {
            HvReleaseCell(Hive, (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyCell);
            Status = 0x8000001Au32 as i32;
            break 'DoAgain;
        }

        if ((*Kcb).__bindgen_anon_1.ExtFlags() & 0x00000040) != 0 {
        }

        Result = CmpGetValueListFromCache(
            Kcb,
            core::ptr::addr_of_mut!(CellData) as *mut PCELL_DATA,
            core::ptr::addr_of_mut!(IndexIsCached),
            core::ptr::addr_of_mut!(CellToRelease),
        );

        if Result == SearchNeedExclusiveLock {
            HvReleaseCell(Hive, (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyCell);
            CmpConvertKcbSharedToExclusive(Kcb);
            continue 'DoAgain;
        } else if Result != SearchSuccess {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'DoAgain;
        }

        Result = CmpGetValueKeyFromCache(
            Kcb,
            CellData,
            Index,
            core::ptr::addr_of_mut!(CachedValue) as *mut *mut PCM_CACHED_VALUE,
            core::ptr::addr_of_mut!(ValueData),
            IndexIsCached,
            core::ptr::addr_of_mut!(ValueIsCached),
            core::ptr::addr_of_mut!(CellToRelease2),
        );

        if Result == SearchNeedExclusiveLock {
            if CellToRelease != 0xFFFFFFFF {
                HvReleaseCell(Hive, CellToRelease);
                CellToRelease = 0xFFFFFFFF;
            }
            HvReleaseCell(Hive, (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyCell);
            CmpConvertKcbSharedToExclusive(Kcb);
            continue 'DoAgain;
        } else if Result != SearchSuccess {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'DoAgain;
        }

        Result = CmpQueryKeyValueData(
            Kcb,
            CachedValue,
            ValueData,
            ValueIsCached,
            KeyValueInformationClass as u32,
            KeyValueInformation,
            Length,
            ResultLength,
            core::ptr::addr_of_mut!(Status),
        );

        if Result == SearchNeedExclusiveLock {
            if CellToRelease2 != 0xFFFFFFFF {
                HvReleaseCell(Hive, CellToRelease2);
            }
            HvReleaseCell(Hive, (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyCell);
            if CellToRelease != 0xFFFFFFFF {
                HvReleaseCell(Hive, CellToRelease);
            }
            CmpConvertKcbSharedToExclusive(Kcb);
            continue 'DoAgain;
        }

        break 'DoAgain;
    }

    if CellToRelease2 != 0xFFFFFFFF {
        HvReleaseCell(Hive, CellToRelease2);
    }
    HvReleaseCell(Hive, (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyCell);
    if CellToRelease != 0xFFFFFFFF {
        HvReleaseCell(Hive, CellToRelease);
    }

    CmpReleaseKcbLock(Kcb);
    CmpUnlockRegistry();
    Status
}

unsafe fn CmpQueryKeyDataFromCache(
    Kcb: PCM_KEY_CONTROL_BLOCK,
    KeyCachedInfo: PKEY_CACHED_INFORMATION,
    Length: u32,
    ResultLength: *mut u32
) -> i32 {
    let KeyHive: PHHIVE;
    let KeyCell: HCELL_INDEX;
    let NameLength: u16;

    KeyHive = (*Kcb).__bindgen_anon_2.KeyHash.KeyHive;
    KeyCell = (*Kcb).__bindgen_anon_2.KeyHash.KeyCell;

    if (*Kcb).NameBlock == core::ptr::null_mut() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    if (*(*Kcb).NameBlock).Compressed != 0 {
        NameLength = CmpCompressedNameSize(
            (*(*Kcb).NameBlock).__bindgen_anon_1.NameHash.Name.as_ptr() as *mut u16,
            (*(*Kcb).NameBlock).__bindgen_anon_1.NameHash.NameLength as u32,
        ) as u16;
    } else {
        NameLength = (*(*Kcb).NameBlock).__bindgen_anon_1.NameHash.NameLength;
    }

    *ResultLength = core::mem::size_of::<KEY_CACHED_INFORMATION>() as u32;
    if Length < *ResultLength {
        return STATUS_BUFFER_TOO_SMALL;
    }

    (*KeyCachedInfo).LastWriteTime = (*Kcb).KcbLastWriteTime;
    (*KeyCachedInfo).TitleIndex = 0;
    (*KeyCachedInfo).NameLength = NameLength as u32;
    (*KeyCachedInfo).Values = (*Kcb).ValueCache.Count;
    (*KeyCachedInfo).MaxNameLen = (*Kcb).KcbMaxNameLen as u32;
    (*KeyCachedInfo).MaxValueNameLen = (*Kcb).KcbMaxValueNameLen as u32;
    (*KeyCachedInfo).MaxValueDataLen = (*Kcb).KcbMaxValueDataLen;

    if ((*Kcb).__bindgen_anon_1.ExtFlags() & CM_KCB_INVALID_CACHED_INFO) != 0 {
        crate::dbg_print!(b"Kcb cache incoherency detected, kcb = %p\n\0", Kcb);
        let Node: PCM_KEY_NODE = HvGetCell(KeyHive, KeyCell) as PCM_KEY_NODE;
        if Node == core::ptr::null_mut() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        (*KeyCachedInfo).SubKeys = (*Node).SubKeyCounts[0] + (*Node).SubKeyCounts[1];

        HvReleaseCell(KeyHive, KeyCell);
    } else if ((*Kcb).__bindgen_anon_1.ExtFlags() & CM_KCB_NO_SUBKEY) != 0 {
        (*KeyCachedInfo).SubKeys = 0;
    } else if ((*Kcb).__bindgen_anon_1.ExtFlags() & CM_KCB_SUBKEY_ONE) != 0 {
        (*KeyCachedInfo).SubKeys = 1;
    } else if ((*Kcb).__bindgen_anon_1.ExtFlags() & CM_KCB_SUBKEY_HINT) != 0 {
        (*KeyCachedInfo).SubKeys = (*(*Kcb).__bindgen_anon_3.IndexHint).Count;
    } else {
        (*KeyCachedInfo).SubKeys = (*Kcb).__bindgen_anon_3.SubKeyCount;
    }

    STATUS_SUCCESS
}

unsafe fn CmpQueryFlagsInformation(
    Kcb: PCM_KEY_CONTROL_BLOCK,
    KeyFlagsInfo: PKEY_USER_FLAGS_INFORMATION,
    Length: u32,
    ResultLength: *mut u32,
) -> i32 {
    *ResultLength = core::mem::size_of::<_KEY_USER_FLAGS_INFORMATION>() as u32;
    if Length < *ResultLength {
        return STATUS_BUFFER_TOO_SMALL;
    }

    (*KeyFlagsInfo).UserFlags = (*Kcb).__bindgen_anon_5.KcbUserFlags();

    STATUS_SUCCESS
}

unsafe fn CmpQueryNameInformation(
    Kcb: PCM_KEY_CONTROL_BLOCK,
    KeyNameInfo: PKEY_NAME_INFORMATION,
    Length: ULONG,
    ResultLength: PULONG,
) -> NTSTATUS {
    let mut NeededLength: ULONG;
    let mut CurrentKcb: PCM_KEY_CONTROL_BLOCK;

    NeededLength = 0;
    CurrentKcb = Kcb;

    while !CurrentKcb.is_null() {
        if (*(*CurrentKcb).NameBlock).Compressed != 0 {
            NeededLength += CmpCompressedNameSize(
                (*(*CurrentKcb).NameBlock).__bindgen_anon_1.__bindgen_anon_1.Name.as_ptr() as *mut u16,
                (*(*CurrentKcb).NameBlock).__bindgen_anon_1.__bindgen_anon_1.NameLength as u32,
            ) as u32;
        } else {
            NeededLength += (*(*CurrentKcb).NameBlock).__bindgen_anon_1.__bindgen_anon_1.NameLength as u32;
        }

        NeededLength += core::mem::size_of::<u16>() as ULONG;

        CurrentKcb = (*CurrentKcb).ParentKcb;
    }

    *ResultLength = core::mem::offset_of!(KEY_NAME_INFORMATION, Name) as ULONG + NeededLength;
    if Length < (core::mem::offset_of!(KEY_NAME_INFORMATION, Name) as ULONG + core::mem::size_of::<ULONG>() as ULONG) {
        return STATUS_BUFFER_TOO_SMALL;
    }
    if Length < *ResultLength {
        (*KeyNameInfo).NameLength = NeededLength;
        return STATUS_BUFFER_OVERFLOW;
    }

    /* Do the real copy: walk parents from leaf up, filling the buffer from
     * tail toward head. C source: ntoskrnl/config/cmapi.c:1560-1601. */
    (*KeyNameInfo).NameLength = 0;
    CurrentKcb = Kcb;
    let NameBuf = (*KeyNameInfo).Name.as_mut_ptr();
    const WCHAR_SIZE: u32 = core::mem::size_of::<u16>() as u32;
    const PATH_SEP: u16 = b'\\' as u16;

    while !CurrentKcb.is_null() {
        let NameLength: u32;
        if (*(*CurrentKcb).NameBlock).Compressed != 0 {
            NameLength = CmpCompressedNameSize(
                (*(*CurrentKcb).NameBlock).__bindgen_anon_1.__bindgen_anon_1.Name.as_ptr() as *mut u16,
                (*(*CurrentKcb).NameBlock).__bindgen_anon_1.__bindgen_anon_1.NameLength as u32,
            ) as u32;
            CmpCopyCompressedName(
                NameBuf.add(((NeededLength - NameLength) / WCHAR_SIZE) as usize),
                NameLength,
                (*(*CurrentKcb).NameBlock).__bindgen_anon_1.__bindgen_anon_1.Name.as_ptr() as *mut u16,
                (*(*CurrentKcb).NameBlock).__bindgen_anon_1.__bindgen_anon_1.NameLength as u32,
            );
        } else {
            NameLength = (*(*CurrentKcb).NameBlock).__bindgen_anon_1.__bindgen_anon_1.NameLength as u32;
            core::ptr::copy_nonoverlapping(
                (*(*CurrentKcb).NameBlock).__bindgen_anon_1.__bindgen_anon_1.Name.as_ptr() as *const u8,
                NameBuf.add(((NeededLength - NameLength) / WCHAR_SIZE) as usize) as *mut u8,
                NameLength as usize,
            );
        }

        NeededLength -= NameLength;
        NeededLength -= WCHAR_SIZE;
        *NameBuf.add((NeededLength / WCHAR_SIZE) as usize) = PATH_SEP;
        (*KeyNameInfo).NameLength += NameLength + WCHAR_SIZE;

        CurrentKcb = (*CurrentKcb).ParentKcb;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn CmQueryKey(
    Kcb: PCM_KEY_CONTROL_BLOCK,
    KeyInformationClass: u32,
    KeyInformation: *mut core::ffi::c_void,
    Length: u32,
    ResultLength: *mut u32,
) -> i32 {
    let Kcb = opaque_ptr(Kcb);
    let KeyInformation = opaque_ptr(KeyInformation);
    let ResultLength = opaque_ptr(ResultLength);
    let mut Status: i32;
    let Hive: PHHIVE;
    let Parent: PCM_KEY_NODE;
    let mut CellReferences: HV_TRACK_CELL_REF = core::mem::zeroed();

    CmpLockRegistry();

    CmpAcquireKcbLockShared(Kcb);

    if (*Kcb).__bindgen_anon_1.Delete() != 0 {
        Status = STATUS_KEY_DELETED;
        HvReleaseFreeCellRefArray(core::ptr::addr_of_mut!(CellReferences));
        CmpReleaseKcbLock(Kcb);
        CmpUnlockRegistry();
        return Status;
    }

    // KEY_INFORMATION_CLASS values (from wdm.h)
    const KEY_BASIC_INFORMATION_CLASS: u32 = 0;
    const KEY_NODE_INFORMATION_CLASS: u32 = 1;
    const KEY_FULL_INFORMATION_CLASS: u32 = 2;
    const KEY_NAME_INFORMATION_CLASS: u32 = 3;
    const KEY_CACHED_INFORMATION_CLASS: u32 = 4;
    const KEY_FLAGS_INFORMATION_CLASS: u32 = 5;

    Status = if KeyInformationClass == KEY_FULL_INFORMATION_CLASS
        || KeyInformationClass == KEY_BASIC_INFORMATION_CLASS
        || KeyInformationClass == KEY_NODE_INFORMATION_CLASS
    {
        Hive = (*Kcb).__bindgen_anon_2.KeyHash.KeyHive;
        Parent = HvGetCell(Hive, (*Kcb).__bindgen_anon_2.KeyHash.KeyCell) as PCM_KEY_NODE;

        if HvTrackCellRef(core::ptr::addr_of_mut!(CellReferences), Hive, (*Kcb).__bindgen_anon_2.KeyHash.KeyCell) == 0 {
            STATUS_INSUFFICIENT_RESOURCES
        } else {
            CmpQueryKeyData(
                Hive,
                Parent,
                KeyInformationClass,
                KeyInformation,
                Length,
                ResultLength,
            )
        }
    } else if KeyInformationClass == KEY_CACHED_INFORMATION_CLASS {
        CmpQueryKeyDataFromCache(
            Kcb,
            KeyInformation as PKEY_CACHED_INFORMATION,
            Length,
            ResultLength,
        )
    } else if KeyInformationClass == KEY_FLAGS_INFORMATION_CLASS {
        CmpQueryFlagsInformation(
            Kcb,
            KeyInformation as PKEY_USER_FLAGS_INFORMATION,
            Length,
            ResultLength,
        )
    } else if KeyInformationClass == KEY_NAME_INFORMATION_CLASS {
        CmpQueryNameInformation(
            Kcb,
            KeyInformation as PKEY_NAME_INFORMATION,
            Length,
            ResultLength,
        )
    } else {
        crate::dbg_print!(b"Unsupported class: %d!\n\0", KeyInformationClass);
        STATUS_INVALID_INFO_CLASS
    };

    HvReleaseFreeCellRefArray(core::ptr::addr_of_mut!(CellReferences));
    CmpReleaseKcbLock(Kcb);
    CmpUnlockRegistry();
    Status
}

#[no_mangle]
pub unsafe extern "system" fn CmEnumerateKey(
    Kcb: PCM_KEY_CONTROL_BLOCK,
    Index: u32,
    KeyInformationClass: u32,
    KeyInformation: *mut core::ffi::c_void,
    Length: u32,
    ResultLength: *mut u32,
) -> i32 {
    let Kcb = opaque_ptr(Kcb);
    let KeyInformation = opaque_ptr(KeyInformation);
    let ResultLength = opaque_ptr(ResultLength);
    let mut Status: i32 = STATUS_SUCCESS;
    let mut Hive: PHHIVE;
    let mut Parent: PCM_KEY_NODE;
    let mut Child: PCM_KEY_NODE;
    let mut ChildCell: HCELL_INDEX;
    let mut CellReferences: HV_TRACK_CELL_REF = core::mem::zeroed();

    CmpLockRegistry();

    CmpAcquireKcbLockShared(Kcb);

    'Quickie: loop {
        if (*Kcb).__bindgen_anon_1.Delete() != 0 {
            Status = STATUS_KEY_DELETED;
            break 'Quickie;
        }

        Hive = (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyHive;
        Parent = HvGetCell(Hive, (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyCell) as PCM_KEY_NODE;

        ChildCell = CmpFindSubKeyByNumber(Hive, Parent, Index);

        HvReleaseCell(Hive, (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyCell);

        if ChildCell == HCELL_NIL {
            Status = STATUS_NO_MORE_ENTRIES;
            break 'Quickie;
        }

        Child = HvGetCell(Hive, ChildCell) as PCM_KEY_NODE;

        if HvTrackCellRef(core::ptr::addr_of_mut!(CellReferences), Hive, ChildCell) == 0 {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'Quickie;
        }

        Status = CmpQueryKeyData(
            Hive,
            Child,
            KeyInformationClass,
            KeyInformation,
            Length,
            ResultLength,
        );

        break 'Quickie;
    }

    // Quickie: release tracked cell refs and locks (matches C cmapi.c:1811-1817).
    HvReleaseFreeCellRefArray(core::ptr::addr_of_mut!(CellReferences));
    CmpReleaseKcbLock(Kcb);
    CmpUnlockRegistry();
    Status
}

#[no_mangle]
pub unsafe extern "system" fn CmDeleteKey(KeyBody: PCM_KEY_BODY) -> NTSTATUS {
    let KeyBody = opaque_ptr(KeyBody);
    let mut Status: NTSTATUS;
    let Hive: PHHIVE;
    let mut Node: PCM_KEY_NODE;
    let mut Parent: PCM_KEY_NODE;
    let Cell: HCELL_INDEX;
    let ParentCell: HCELL_INDEX;
    let Kcb: PCM_KEY_CONTROL_BLOCK;

    CmpLockRegistry();

    Kcb = (*KeyBody).KeyControlBlock;

    if (*Kcb).ParentKcb.is_null() {
        CmpUnlockRegistry();
        return STATUS_CANNOT_DELETE;
    }

    CmpAcquireTwoKcbLocksExclusiveByKey((*Kcb).__bindgen_anon_2.__bindgen_anon_1.ConvKey, (*(*Kcb).ParentKcb).__bindgen_anon_2.__bindgen_anon_1.ConvKey);

    if (*Kcb).__bindgen_anon_1.Delete() != 0 {
        Status = STATUS_SUCCESS;
        CmpReleaseTwoKcbLockByKey((*Kcb).__bindgen_anon_2.__bindgen_anon_1.ConvKey, (*(*Kcb).ParentKcb).__bindgen_anon_2.__bindgen_anon_1.ConvKey);
        CmpUnlockRegistry();
        return Status;
    }

    Hive = (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyHive;
    Cell = (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyCell;

    CmpLockHiveFlusherShared(Hive as PCMHIVE);

    Node = HvGetCell(Hive, Cell) as PCM_KEY_NODE;

    if ((*Node).SubKeyCounts[0] + (*Node).SubKeyCounts[1]) == 0 &&
        ((*Node).Flags & KEY_NO_DELETE as u16) == 0 {
        CmpReportNotify(Kcb, Hive, Cell, REG_NOTIFY_CHANGE_NAME);

        ParentCell = (*Node).Parent;
        // Deletion shares sk reference counts with security replacement.
        ntkrnl_sys::ddk::KeEnterCriticalRegion();
        ExAcquirePushLockExclusive(core::ptr::addr_of_mut!((*(Hive as PCMHIVE)).SecurityLock));
        Status = CmpFreeKeyByCell(Hive, Cell, 1u8);
        ExReleasePushLockExclusive(core::ptr::addr_of_mut!((*(Hive as PCMHIVE)).SecurityLock));
        ntkrnl_sys::ddk::KeLeaveCriticalRegion();
        if NT_SUCCESS(Status) {
            CmpFlushNotifiesOnKeyBodyList(Kcb, 0u8);

            CmpCleanUpSubKeyInfo((*Kcb).ParentKcb);

            Parent = HvGetCell(Hive, ParentCell) as PCM_KEY_NODE;
            if !Parent.is_null() {
                (*(*Kcb).ParentKcb).KcbMaxNameLen = (*Parent).MaxNameLen as USHORT;

                KeQuerySystemTime(core::ptr::addr_of_mut!((*Parent).LastWriteTime) as *mut _);
                (*(*Kcb).ParentKcb).KcbLastWriteTime = (*Parent).LastWriteTime;

                HvReleaseCell(Hive, ParentCell);
            }

            (*Kcb).__bindgen_anon_1.set_Delete(1);
            CmpRemoveKeyControlBlock(Kcb);

            (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyCell = HCELL_NIL;
        }
    } else {
        Status = STATUS_CANNOT_DELETE;
    }

    HvReleaseCell(Hive, Cell);

    CmpUnlockHiveFlusher(Hive as PCMHIVE);

    CmpReleaseTwoKcbLockByKey((*Kcb).__bindgen_anon_2.__bindgen_anon_1.ConvKey, (*(*Kcb).ParentKcb).__bindgen_anon_2.__bindgen_anon_1.ConvKey);

    CmpUnlockRegistry();
    Status
}

#[no_mangle]
pub unsafe extern "system" fn CmFlushKey(
    Kcb: PCM_KEY_CONTROL_BLOCK,
    ExclusiveLock: u8,
) -> i32 {
    let Kcb = opaque_ptr(Kcb);
    let mut CmHive: PCMHIVE;
    let mut Status: i32 = STATUS_SUCCESS;
    let mut Hive: PHHIVE;

    if CmpNoWrite != 0 {
        return STATUS_SUCCESS;
    }

    Hive = (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyHive;
    CmHive = Hive as PCMHIVE;

    if CmHive == CmiVolatileHive {
        CmpDoFlushAll(0u8);
    } else {
        #[cfg(debug_assertions)]
        {
            let _CheckStatus = CmCheckRegistry(CmHive, 0x4 | 0x100);
        }

        CmpLockHiveFlusherExclusive(CmHive);

        KeAcquireGuardedMutex((*CmHive).ViewLock as *mut _);
        (*CmHive).ViewLockOwner = KeGetCurrentThread() as *mut _;

        if HvHiveWillShrink(Hive) != 0 {
        } else {
            (*CmHive).ViewLockOwner = core::ptr::null_mut();
            KeReleaseGuardedMutex((*CmHive).ViewLock as *mut _);
        }

        if HvSyncHive(Hive) == 0 {
            Status = 0xC000014Du32 as i32; // STATUS_REGISTRY_IO_FAILED
        }

        CmpUnlockHiveFlusher(CmHive);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn CmLoadKey(
    TargetKey: POBJECT_ATTRIBUTES,
    SourceFile: POBJECT_ATTRIBUTES,
    Flags: ULONG,
    KeyBody: PCM_KEY_BODY,
) -> NTSTATUS {
    let TargetKey = opaque_ptr(TargetKey);
    let SourceFile = opaque_ptr(SourceFile);
    let KeyBody = opaque_ptr(KeyBody);
    let mut ServiceQos: SECURITY_QUALITY_OF_SERVICE = core::mem::zeroed();
    let mut ClientSecurityContext: SECURITY_CLIENT_CONTEXT = core::mem::zeroed();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut Allocate: BOOLEAN = 1u8;
    let mut CmHive: PCMHIVE = core::ptr::null_mut();
    let mut LoadedHive: PCMHIVE = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut ParseContext: CM_PARSE_CONTEXT = core::mem::zeroed();

    if !KeyBody.is_null() {
        // DPRINT removed
    }

    ServiceQos.Length = core::mem::size_of::<SECURITY_QUALITY_OF_SERVICE>() as ULONG;
    ServiceQos.ImpersonationLevel = _SECURITY_IMPERSONATION_LEVEL_SecurityImpersonation;
    ServiceQos.ContextTrackingMode = 1u8;
    ServiceQos.EffectiveOnly = 1u8;
    Status = SeCreateClientSecurity(
        PsGetCurrentThread(),
        &mut ServiceQos,
        0u8,
        &mut ClientSecurityContext,
    );
    if !NT_SUCCESS(Status) {
        crate::dbg_print!(b"Security context failed\n\0");
        return Status;
    }

    core::ptr::write_bytes(&mut ParseContext as *mut CM_PARSE_CONTEXT, 0, 1);
    ParseContext.CreateOperation = 0u8;
    Status = ObOpenObjectByName(
        TargetKey,
        CmpKeyObjectType as *mut _,
        0i8,
        core::ptr::null_mut(),
        1u32,
        &mut ParseContext as *mut CM_PARSE_CONTEXT as *mut core::ffi::c_void,
        &mut KeyHandle,
    );
    if !NT_SUCCESS(Status) {
        KeyHandle = core::ptr::null_mut();
    }

    Status = CmpCmdHiveOpen(
        SourceFile,
        &mut ClientSecurityContext,
        &mut Allocate,
        &mut CmHive,
        0x00000001u32,
    );

    SeDeleteClientSecurity(&mut ClientSecurityContext);

    if !NT_SUCCESS(Status) {
        if !KeyHandle.is_null() {
            CmpLockRegistryExclusive();

            if CmpIsHiveAlreadyLoaded(KeyHandle, SourceFile, &mut LoadedHive) != 0 {
                ASSERT!(LoadedHive.is_null() == false);
                Status = STATUS_SUCCESS;
            }

            CmpUnlockRegistry();
        }

        if !KeyHandle.is_null() {
            ZwClose(KeyHandle);
        }
        return Status;
    }

    CmpLockRegistry();

    ExAcquirePushLockExclusive(core::ptr::addr_of_mut!(CmpLoadHiveLock) as *mut _);

    // C: HiveFlags |= HIVE_IS_UNLOADING (0x20) — the grind wrote 0x4, which is
    // HIVE_HAS_BEEN_REPLACED, and CmUnloadKey then wrote a third value again.
    (*CmHive).Hive.HiveFlags |= HIVE_IS_UNLOADING;
    (*CmHive).CreatorOwner = KeGetCurrentThread() as *mut _;

    // REG_NO_LAZY_FLUSH (0x4) is the caller's flag; the hive bit it sets is
    // HIVE_NOLAZYFLUSH (0x2), not 0x8 (HIVE_HAS_BEEN_FREED).
    if (Flags & 0x00000004u32) != 0 {
        (*CmHive).Hive.HiveFlags |= HIVE_NOLAZYFLUSH;
    }

    Status = CmpLinkHiveToMaster(
        (*TargetKey).ObjectName,
        (*TargetKey).RootDirectory,
        CmHive,
        Allocate,
        (*TargetKey).SecurityDescriptor,
    );
    if NT_SUCCESS(Status) {
        CmpAddToHiveFileList(CmHive);

        if Allocate != 0 {
            CmpLockHiveFlusherExclusive(CmHive);
            HvSyncHive(&mut (*CmHive).Hive);
            CmpUnlockHiveFlusher(CmHive);
        }

        (*CmHive).Hive.HiveFlags &= !HIVE_IS_UNLOADING;
        (*CmHive).CreatorOwner = core::ptr::null_mut();
    } else {
        crate::dbg_print!(b"CmpLinkHiveToMaster failed, Status %lx\n\0", Status);

        (*CmHive).HiveIsLoading = 1u8;

        CmpCloseHiveFiles(CmHive);

        CmpDestroyHive(CmHive);
    }

    ExReleasePushLock(core::ptr::addr_of_mut!(CmpLoadHiveLock) as *mut _);

    if CmpProfileLoaded == 0 && CmpWasSetupBoot == 0 {
        CmpProfileLoaded = 1u8;
        CmpSetGlobalQuotaAllowed();
    }

    CmpUnlockRegistry();

    if !KeyHandle.is_null() {
        ZwClose(KeyHandle);
    }
    return Status;
}

unsafe fn CmpUnlinkHiveFromMaster(
    CmHive: PCMHIVE,
    Cell: HCELL_INDEX,
) -> u8 {
    let mut CellData: PCELL_DATA;
    let mut LinkCell: HCELL_INDEX;
    let mut Status: i32;

    CellData = HvGetCell(core::ptr::addr_of_mut!((*CmHive).Hive), Cell) as PCELL_DATA;
    if CellData.is_null() {
        return 0u8;
    }

    LinkCell = (*CellData).u.KeyNode.Parent;
    HvReleaseCell(core::ptr::addr_of_mut!((*CmHive).Hive), Cell);

    CmpLockHiveFlusherExclusive(CmiVolatileHive);
    Status = CmpFreeKeyByCell(
        CmiVolatileHive as PHHIVE,
        LinkCell,
        1u8,
    );
    CmpUnlockHiveFlusher(CmiVolatileHive);
    if !NT_SUCCESS(Status) {
        crate::dbg_print!(b"CmpFreeKeyByCell() failed (Status 0x%08lx)\n\0", Status);
        return 0u8;
    }

    ExAcquirePushLockExclusive(core::ptr::addr_of_mut!(CmpHiveListHeadLock) as *mut _);
    RemoveEntryList(core::ptr::addr_of_mut!((*CmHive).HiveList));
    ExReleasePushLock(core::ptr::addr_of_mut!(CmpHiveListHeadLock) as *mut _);

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn CmUnloadKey(
    Kcb: PCM_KEY_CONTROL_BLOCK,
    Flags: u32,
) -> i32 {
    let Kcb = opaque_ptr(Kcb);
    let mut Hive: PHHIVE;
    let mut CmHive: PCMHIVE;
    let mut Cell: HCELL_INDEX;

    const STATUS_INVALID_PARAMETER: i32 = -1073741811i32;
    const STATUS_CANNOT_DELETE: i32 = -1073741535i32;
    const REG_FORCE_UNLOAD: u32 = 1;
    // C: cmlib.h `#define TAG_CMHIVE 'vHMC'` (big-endian multi-char literal). cminit.rs ALLOCATES the ViewLock/FlusherLock
    // this function frees, so a divergent value here is a real tag mismatch.
    const TAG_CMHIVE: u32 = u32::from_be_bytes(*b"vHMC");
    const TAG_CM: u32 = 0x20204D43;

    CMP_ASSERT_EXCLUSIVE_REGISTRY_LOCK();

    Hive = (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyHive;
    Cell = (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyCell;
    CmHive = Hive as PCMHIVE;

    if Cell != (*(*Hive).BaseBlock).RootCell {
        crate::dbg_print!(b"Key is not a hive root key!\n\0");
        return STATUS_INVALID_PARAMETER;
    }

    if CmHive == CmiVolatileHive {
        crate::dbg_print!(b"Do not try to unload the master hive!\n\0");
        return STATUS_INVALID_PARAMETER;
    }

    (*Hive).HiveFlags |= HIVE_IS_UNLOADING;

    if (*Kcb).RefCount > 1 {
        if Flags != REG_FORCE_UNLOAD {
            if CmpEnumerateOpenSubKeys(Kcb, 1, 0) != 0 {
                (*Hive).HiveFlags &= !HIVE_IS_UNLOADING;
                return STATUS_CANNOT_DELETE;
            }
        } else {
            if CmpEnumerateOpenSubKeys(Kcb, 1, 1) != 0 {
                (*Hive).HiveFlags &= !HIVE_IS_UNLOADING;
                return STATUS_CANNOT_DELETE;
            }
        }
    }

    (*CmHive).HiveIsLoading = 1;

    CmFlushKey(Kcb, 1);

    if CmpUnlinkHiveFromMaster(CmHive, Cell) == 0 {
        (*Hive).HiveFlags &= !HIVE_IS_UNLOADING;
        (*CmHive).HiveIsLoading = 0;
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    if Flags == REG_FORCE_UNLOAD {
        CmpFlushNotifiesOnKeyBodyList(Kcb, 1);
    }

    CmpCleanUpSubKeyInfo((*Kcb).ParentKcb);

    (*Kcb).__bindgen_anon_1.set_Delete(1);
    CmpRemoveKeyControlBlock(Kcb);

    ExReleasePushLockExclusive(core::ptr::addr_of_mut!(CmpLoadHiveLock) as *mut _);

    CmpUnlockRegistry();

    // Remove the hive from the hive file list BEFORE closing the file handles.
    // Upstream 396019984c9 swapped these two: CmpRemoveFromHiveFileList() now reads the
    // path captured at add time rather than re-deriving it, and the ordering matches the
    // symmetry of the linking process (Win2003 RTM order).
    CmpRemoveFromHiveFileList(CmHive);

    CmpCloseHiveFiles(CmHive);

    CmpDestroySecurityCache(CmHive);

    CmpDestroyHiveViewList(CmHive);

    ExDeleteResourceLite((*CmHive).FlusherLock);
    ExFreePoolWithTag((*CmHive).FlusherLock as *mut core::ffi::c_void, TAG_CMHIVE);

    ExFreePoolWithTag((*CmHive).ViewLock as *mut core::ffi::c_void, TAG_CMHIVE);

    HvFree(Hive);

    CmpFree(CmHive as *mut core::ffi::c_void, TAG_CM);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn CmpEnumerateOpenSubKeys(
    RootKcb: PCM_KEY_CONTROL_BLOCK,
    RemoveEmptyCacheEntries: BOOLEAN,
    DereferenceOpenedEntries: BOOLEAN,
) -> ULONG {
    let RootKcb = opaque_ptr(RootKcb);
    let mut Entry: PCM_KEY_HASH;
    let mut CachedKcb: PCM_KEY_CONTROL_BLOCK;
    let mut ParentKcb: PCM_KEY_CONTROL_BLOCK;
    let mut ParentKeyCount: ULONG;
    let mut i: ULONG;
    let mut j: ULONG;
    let mut SubKeys: ULONG = 0;

    CMP_ASSERT_EXCLUSIVE_REGISTRY_LOCK();

    if (*RootKcb).RefCount == 1 {
        return 0;
    }

    let CacheTable = CmpCacheTable as *mut ntkrnl_sys::ddk::_CM_KEY_HASH_TABLE_ENTRY;

    i = 0;
    while i < CmpHashTableSize {
        Entry = (*CacheTable.offset(i as isize)).Entry;

        while !Entry.is_null() {
            CachedKcb = (Entry as *mut u8).offset(-(core::mem::offset_of!(_CM_KEY_CONTROL_BLOCK, __bindgen_anon_2) as isize)) as PCM_KEY_CONTROL_BLOCK;

            if (*CachedKcb).__bindgen_anon_1.TotalLevels() > (*RootKcb).__bindgen_anon_1.TotalLevels() {
                ParentKeyCount = (*CachedKcb).__bindgen_anon_1.TotalLevels() - (*RootKcb).__bindgen_anon_1.TotalLevels();

                ParentKcb = CachedKcb;
                j = 0;
                while j < ParentKeyCount {
                    ParentKcb = (*ParentKcb).ParentKcb;
                    j += 1;
                }

                if ParentKcb == RootKcb {
                    if (*CachedKcb).RefCount > 0 {
                        if DereferenceOpenedEntries != 0 &&
                            ((*CachedKcb).__bindgen_anon_1.ExtFlags() & CM_KCB_READ_ONLY_KEY) == 0
                        {
                            CmpFlushNotifiesOnKeyBodyList(CachedKcb, 1u8);

                            CmpCleanUpSubKeyInfo((*CachedKcb).ParentKcb);

                            Entry = (*CachedKcb).__bindgen_anon_2.__bindgen_anon_1.NextHash;

                            (*CachedKcb).__bindgen_anon_1.set_Delete(1);
                            CmpRemoveKeyControlBlock(CachedKcb);

                            (*CachedKcb).__bindgen_anon_2.__bindgen_anon_1.KeyCell = HCELL_NIL;

                            continue;
                        }

                        SubKeys += 1;
                    } else if (*CachedKcb).RefCount == 0 && RemoveEmptyCacheEntries != 0 {
                        CmpRemoveFromDelayedClose(CachedKcb);

                        CmpCleanUpKcbCacheWithLock(CachedKcb, 1u8);

                        Entry = (*CacheTable.offset(i as isize)).Entry;
                        continue;
                    }
                }
            }

            Entry = (*Entry).NextHash;
        }

        i += 1;
    }

    if SubKeys > 0 {
        crate::dbg_print!(b"Open sub keys: %u\n\0", SubKeys);
    }

    SubKeys
}

unsafe fn CmpDeepCopyKeyInternal(
    SourceHive: PHHIVE,
    SrcKeyCell: HCELL_INDEX,
    DestinationHive: PHHIVE,
    Parent: HCELL_INDEX,
    StorageType: HSTORAGE_TYPE,
    DestKeyCell: PHCELL_INDEX,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut SrcNode: PCM_KEY_NODE;
    let mut DestNode: PCM_KEY_NODE = core::ptr::null_mut();
    let mut NewKeyCell: HCELL_INDEX = HCELL_NIL;
    let mut NewClassCell: HCELL_INDEX = HCELL_NIL;
    let mut NewSecCell: HCELL_INDEX = HCELL_NIL;
    let mut SubKey: HCELL_INDEX;
    let mut NewSubKey: HCELL_INDEX = HCELL_NIL;
    let mut Index: ULONG;
    let SubKeyCount: ULONG;

    SrcNode = HvGetCell(SourceHive, SrcKeyCell) as PCM_KEY_NODE;

    NewKeyCell = CmpCopyCell(SourceHive, SrcKeyCell, DestinationHive, StorageType);
    if NewKeyCell == HCELL_NIL {
        Status = STATUS_INSUFFICIENT_RESOURCES;
        'Cleanup: loop {
            if !DestNode.is_null() {
                HvReleaseCell(DestinationHive, NewKeyCell);
            }
            if !SrcNode.is_null() {
                HvReleaseCell(SourceHive, SrcKeyCell);
            }

            if !NT_SUCCESS(Status) {
                if NewSecCell != HCELL_NIL {
                    HvFreeCell(DestinationHive, NewSecCell);
                }

                if NewClassCell != HCELL_NIL {
                    HvFreeCell(DestinationHive, NewClassCell);
                }

                if NewKeyCell != HCELL_NIL {
                    HvFreeCell(DestinationHive, NewKeyCell);
                }

                NewKeyCell = HCELL_NIL;
            }

            if !DestKeyCell.is_null() {
                *DestKeyCell = NewKeyCell;
            }
            return Status;
        }
    }

    DestNode = HvGetCell(DestinationHive, NewKeyCell) as PCM_KEY_NODE;

    (*DestNode).Parent = Parent;
    (*DestNode).Flags = (*SrcNode).Flags & KEY_COMP_NAME;
    if Parent == HCELL_NIL {
        (*DestNode).Flags |= (KEY_HIVE_ENTRY | KEY_NO_DELETE) as u16;
    }

    if (*SrcNode).ClassLength > 0 {
        NewClassCell = CmpCopyCell(SourceHive, (*SrcNode).Class, DestinationHive, StorageType);
        if NewClassCell == HCELL_NIL {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            'Cleanup: loop {
                if !DestNode.is_null() {
                    HvReleaseCell(DestinationHive, NewKeyCell);
                }
                if !SrcNode.is_null() {
                    HvReleaseCell(SourceHive, SrcKeyCell);
                }

                if !NT_SUCCESS(Status) {
                    if NewSecCell != HCELL_NIL {
                        HvFreeCell(DestinationHive, NewSecCell);
                    }

                    if NewClassCell != HCELL_NIL {
                        HvFreeCell(DestinationHive, NewClassCell);
                    }

                    if NewKeyCell != HCELL_NIL {
                        HvFreeCell(DestinationHive, NewKeyCell);
                    }

                    NewKeyCell = HCELL_NIL;
                }

                if !DestKeyCell.is_null() {
                    *DestKeyCell = NewKeyCell;
                }
                return Status;
            }
        }

        (*DestNode).Class = NewClassCell;
        (*DestNode).ClassLength = (*SrcNode).ClassLength;
    } else {
        (*DestNode).Class = HCELL_NIL;
        (*DestNode).ClassLength = 0;
    }

    if (*SrcNode).Security != HCELL_NIL {
        NewSecCell = CmpCopyCell(SourceHive, (*SrcNode).Security, DestinationHive, StorageType);
        let copied_security = if NewSecCell == HCELL_NIL {
            core::ptr::null_mut()
        } else {
            HvGetCell(DestinationHive, NewSecCell) as ntkrnl_sys::ddk::PCM_KEY_SECURITY
        };
        if copied_security.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            'Cleanup: loop {
                if !DestNode.is_null() {
                    HvReleaseCell(DestinationHive, NewKeyCell);
                }
                if !SrcNode.is_null() {
                    HvReleaseCell(SourceHive, SrcKeyCell);
                }

                if !NT_SUCCESS(Status) {
                    if NewSecCell != HCELL_NIL {
                        HvFreeCell(DestinationHive, NewSecCell);
                    }

                    if NewClassCell != HCELL_NIL {
                        HvFreeCell(DestinationHive, NewClassCell);
                    }

                    if NewKeyCell != HCELL_NIL {
                        HvFreeCell(DestinationHive, NewKeyCell);
                    }

                    NewKeyCell = HCELL_NIL;
                }

                if !DestKeyCell.is_null() {
                    *DestKeyCell = NewKeyCell;
                }
                return Status;
            }
        }
        // Links and references from the source hive do not belong to this copy.
        (*copied_security).Flink = NewSecCell;
        (*copied_security).Blink = NewSecCell;
        (*copied_security).ReferenceCount = 1;
        HvReleaseCell(DestinationHive, NewSecCell);
    }
    (*DestNode).Security = NewSecCell;

    Status = CmpCopyKeyValueList(
        SourceHive,
        core::ptr::addr_of_mut!((*SrcNode).__bindgen_anon_1.__bindgen_anon_1.ValueList),
        DestinationHive,
        core::ptr::addr_of_mut!((*DestNode).__bindgen_anon_1.__bindgen_anon_1.ValueList),
        StorageType,
    );
    if !NT_SUCCESS(Status) {
        'Cleanup: loop {
            if !DestNode.is_null() {
                HvReleaseCell(DestinationHive, NewKeyCell);
            }
            if !SrcNode.is_null() {
                HvReleaseCell(SourceHive, SrcKeyCell);
            }

            if !NT_SUCCESS(Status) {
                if NewSecCell != HCELL_NIL {
                    HvFreeCell(DestinationHive, NewSecCell);
                }

                if NewClassCell != HCELL_NIL {
                    HvFreeCell(DestinationHive, NewClassCell);
                }

                if NewKeyCell != HCELL_NIL {
                    HvFreeCell(DestinationHive, NewKeyCell);
                }

                NewKeyCell = HCELL_NIL;
            }

            if !DestKeyCell.is_null() {
                *DestKeyCell = NewKeyCell;
            }
            return Status;
        }
    }

    (*DestNode).SubKeyCounts[Stable as usize] = 0;
    (*DestNode).SubKeyCounts[Volatile as usize] = 0;
    (*DestNode).__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[Stable as usize] = HCELL_NIL;
    (*DestNode).__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[Volatile as usize] = HCELL_NIL;

    SubKeyCount = (*SrcNode).SubKeyCounts[Stable as usize] + (*SrcNode).SubKeyCounts[Volatile as usize];

    Index = 0;
    while Index < SubKeyCount {
        SubKey = CmpFindSubKeyByNumber(SourceHive, SrcNode, Index);

        Status = CmpDeepCopyKeyInternal(
            SourceHive,
            SubKey,
            DestinationHive,
            NewKeyCell,
            StorageType,
            core::ptr::addr_of_mut!(NewSubKey),
        );
        if !NT_SUCCESS(Status) {
            'Cleanup: loop {
                if !DestNode.is_null() {
                    HvReleaseCell(DestinationHive, NewKeyCell);
                }
                if !SrcNode.is_null() {
                    HvReleaseCell(SourceHive, SrcKeyCell);
                }

                if !NT_SUCCESS(Status) {
                    if NewSecCell != HCELL_NIL {
                        HvFreeCell(DestinationHive, NewSecCell);
                    }

                    if NewClassCell != HCELL_NIL {
                        HvFreeCell(DestinationHive, NewClassCell);
                    }

                    if NewKeyCell != HCELL_NIL {
                        HvFreeCell(DestinationHive, NewKeyCell);
                    }

                    NewKeyCell = HCELL_NIL;
                }

                if !DestKeyCell.is_null() {
                    *DestKeyCell = NewKeyCell;
                }
                return Status;
            }
        }

        if CmpAddSubKey(DestinationHive, NewKeyCell, NewSubKey) == 0 {
            HvFreeCell(DestinationHive, NewSubKey);

            Status = STATUS_INSUFFICIENT_RESOURCES;
            'Cleanup: loop {
                if !DestNode.is_null() {
                    HvReleaseCell(DestinationHive, NewKeyCell);
                }
                if !SrcNode.is_null() {
                    HvReleaseCell(SourceHive, SrcKeyCell);
                }

                if !NT_SUCCESS(Status) {
                    if NewSecCell != HCELL_NIL {
                        HvFreeCell(DestinationHive, NewSecCell);
                    }

                    if NewClassCell != HCELL_NIL {
                        HvFreeCell(DestinationHive, NewClassCell);
                    }

                    if NewKeyCell != HCELL_NIL {
                        HvFreeCell(DestinationHive, NewKeyCell);
                    }

                    NewKeyCell = HCELL_NIL;
                }

                if !DestKeyCell.is_null() {
                    *DestKeyCell = NewKeyCell;
                }
                return Status;
            }
        }

        Index += 1;
    }

    Status = STATUS_SUCCESS;

    if !DestNode.is_null() {
        HvReleaseCell(DestinationHive, NewKeyCell);
    }
    if !SrcNode.is_null() {
        HvReleaseCell(SourceHive, SrcKeyCell);
    }

    if !NT_SUCCESS(Status) {
        if NewSecCell != HCELL_NIL {
            HvFreeCell(DestinationHive, NewSecCell);
        }

        if NewClassCell != HCELL_NIL {
            HvFreeCell(DestinationHive, NewClassCell);
        }

        if NewKeyCell != HCELL_NIL {
            HvFreeCell(DestinationHive, NewKeyCell);
        }

        NewKeyCell = HCELL_NIL;
    }

    if !DestKeyCell.is_null() {
        *DestKeyCell = NewKeyCell;
    }
    Status
}

#[no_mangle]
pub unsafe extern "system" fn CmpDeepCopyKey(
    SourceHive: PHHIVE,
    SrcKeyCell: HCELL_INDEX,
    DestinationHive: PHHIVE,
    StorageType: HSTORAGE_TYPE,
    DestKeyCell: PHCELL_INDEX,
) -> NTSTATUS {
    let SourceHive = opaque_ptr(SourceHive);
    let DestinationHive = opaque_ptr(DestinationHive);
    let DestKeyCell = opaque_ptr(DestKeyCell);
    CmpDeepCopyKeyInternal(
        SourceHive,
        SrcKeyCell,
        DestinationHive,
        HCELL_NIL,
        StorageType,
        DestKeyCell,
    )
}

#[no_mangle]
pub unsafe extern "system" fn CmSaveKey(
    Kcb: PCM_KEY_CONTROL_BLOCK,
    FileHandle: HANDLE,
    Flags: ULONG,
) -> NTSTATUS {
    let Kcb = opaque_ptr(Kcb);
    let FileHandle = opaque_ptr(FileHandle);

    #[cfg(debug_assertions)]
    let mut CheckStatus: CM_CHECK_REGISTRY_STATUS = 0;
    #[cfg(debug_assertions)]
    let mut HiveToValidate: PCMHIVE = core::ptr::null_mut();

    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut KeyHive: PCMHIVE = core::ptr::null_mut();

    CmpLockRegistry();
    CmpAcquireKcbLockShared(Kcb);

    #[cfg(debug_assertions)]
    {
        HiveToValidate = (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyHive as PCMHIVE;
    }

    'Cleanup: loop {
        if (*Kcb).__bindgen_anon_1.Delete() != 0 {
            Status = STATUS_KEY_DELETED;
            break 'Cleanup;
        }

        if (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyHive == core::ptr::addr_of_mut!((*CmiVolatileHive).Hive) {
            Status = STATUS_ACCESS_DENIED;
            break 'Cleanup;
        }

        #[cfg(debug_assertions)]
        {
            CheckStatus = CmCheckRegistry(
                HiveToValidate,
                CM_CHECK_REGISTRY_DONT_PURGE_VOLATILES | CM_CHECK_REGISTRY_VALIDATE_HIVE,
            );
        }

        Status = CmpInitializeHive(
            core::ptr::addr_of_mut!(KeyHive),
            HINIT_CREATE,
            HIVE_VOLATILE,
            HFILE_TYPE_PRIMARY,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            CM_CHECK_REGISTRY_DONT_PURGE_VOLATILES,
        );
        if !NT_SUCCESS(Status) {
            break 'Cleanup;
        }

        Status = CmpDeepCopyKey(
            (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyHive,
            (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyCell,
            core::ptr::addr_of_mut!((*KeyHive).Hive),
            Stable,
            core::ptr::addr_of_mut!((*(*KeyHive).Hive.BaseBlock).RootCell),
        );
        if !NT_SUCCESS(Status) {
            break 'Cleanup;
        }

        (*KeyHive).FileHandles[HFILE_TYPE_PRIMARY as usize] = FileHandle;

        HvWriteHive(core::ptr::addr_of_mut!((*KeyHive).Hive));

        break 'Cleanup;
    }

    if !KeyHive.is_null() {
        CmpDestroyHive(KeyHive);
    }

    #[cfg(debug_assertions)]
    {
        if NT_SUCCESS(Status) {
            CheckStatus = CmCheckRegistry(
                HiveToValidate,
                CM_CHECK_REGISTRY_DONT_PURGE_VOLATILES | CM_CHECK_REGISTRY_VALIDATE_HIVE,
            );
        }
    }

    CmpReleaseKcbLock(Kcb);
    CmpUnlockRegistry();

    Status
}

#[no_mangle]
pub unsafe extern "system" fn CmSaveMergedKeys(
    HighKcb: PCM_KEY_CONTROL_BLOCK,
    LowKcb: PCM_KEY_CONTROL_BLOCK,
    FileHandle: HANDLE,
) -> NTSTATUS {
    let HighKcb = opaque_ptr(HighKcb);
    let LowKcb = opaque_ptr(LowKcb);
    let FileHandle = opaque_ptr(FileHandle);

    let mut KeyHive: PCMHIVE = core::ptr::null_mut();
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    CmpLockRegistry();
    CmpAcquireKcbLockShared(HighKcb);
    CmpAcquireKcbLockShared(LowKcb);

    if (*LowKcb).__bindgen_anon_1.Delete() != 0 || (*HighKcb).__bindgen_anon_1.Delete() != 0 {
        Status = STATUS_KEY_DELETED;
        CmpReleaseKcbLock(LowKcb);
        CmpReleaseKcbLock(HighKcb);
        CmpUnlockRegistry();
        return Status;
    }

    Status = CmpInitializeHive(
        core::ptr::addr_of_mut!(KeyHive),
        HINIT_CREATE,
        HIVE_VOLATILE,
        HFILE_TYPE_PRIMARY,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        CM_CHECK_REGISTRY_DONT_PURGE_VOLATILES,
    );
    if !NT_SUCCESS(Status) {
        CmpReleaseKcbLock(LowKcb);
        CmpReleaseKcbLock(HighKcb);
        CmpUnlockRegistry();
        return Status;
    }

    Status = CmpDeepCopyKey(
        (*LowKcb).__bindgen_anon_2.__bindgen_anon_1.KeyHive,
        (*LowKcb).__bindgen_anon_2.__bindgen_anon_1.KeyCell,
        core::ptr::addr_of_mut!((*KeyHive).Hive),
        Stable,
        core::ptr::addr_of_mut!((*(*KeyHive).Hive.BaseBlock).RootCell),
    );
    if !NT_SUCCESS(Status) {
        if !KeyHive.is_null() {
            CmpDestroyHive(KeyHive);
        }
        CmpReleaseKcbLock(LowKcb);
        CmpReleaseKcbLock(HighKcb);
        CmpUnlockRegistry();
        return Status;
    }

    Status = CmpDeepCopyKey(
        (*HighKcb).__bindgen_anon_2.__bindgen_anon_1.KeyHive,
        (*HighKcb).__bindgen_anon_2.__bindgen_anon_1.KeyCell,
        core::ptr::addr_of_mut!((*KeyHive).Hive),
        Stable,
        core::ptr::addr_of_mut!((*(*KeyHive).Hive.BaseBlock).RootCell),
    );
    if !NT_SUCCESS(Status) {
        if !KeyHive.is_null() {
            CmpDestroyHive(KeyHive);
        }
        CmpReleaseKcbLock(LowKcb);
        CmpReleaseKcbLock(HighKcb);
        CmpUnlockRegistry();
        return Status;
    }

    (*KeyHive).FileHandles[HFILE_TYPE_PRIMARY as usize] = FileHandle;

    HvWriteHive(core::ptr::addr_of_mut!((*KeyHive).Hive));

    if !KeyHive.is_null() {
        CmpDestroyHive(KeyHive);
    }

    CmpReleaseKcbLock(LowKcb);
    CmpReleaseKcbLock(HighKcb);
    CmpUnlockRegistry();

    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/config/cmapi.c
//  * PURPOSE:         Configuration Manager - Internal Registry APIs
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Eric Kohl
//  *                  Aleksandar Andrejevic <theflash AT sdf DOT lonestar DOT org>
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include "ntoskrnl.h"
// #define NDEBUG
// #include "debug.h"
//
// /* FUNCTIONS *****************************************************************/
//
// BOOLEAN
// NTAPI
// CmpIsHiveAlreadyLoaded(IN HANDLE KeyHandle,
//                        IN POBJECT_ATTRIBUTES SourceFile,
//                        OUT PCMHIVE *CmHive)
// {
//     NTSTATUS Status;
//     PCM_KEY_BODY KeyBody;
//     PCMHIVE Hive;
//     BOOLEAN Loaded = FALSE;
//     PAGED_CODE();
//
//     /* Sanity check */
//     CMP_ASSERT_EXCLUSIVE_REGISTRY_LOCK();
//
//     /* Reference the handle */
//     Status = ObReferenceObjectByHandle(KeyHandle,
//                                        0,
//                                        CmpKeyObjectType,
//                                        KernelMode,
//                                        (PVOID)&KeyBody,
//                                        NULL);
//     if (!NT_SUCCESS(Status)) return Loaded;
//
//     /* Don't touch deleted KCBs */
//     if (KeyBody->KeyControlBlock->Delete) return Loaded;
//
//     Hive = CONTAINING_RECORD(KeyBody->KeyControlBlock->KeyHive, CMHIVE, Hive);
//
//     /* Must be the root key */
//     if (!(KeyBody->KeyControlBlock->Flags & KEY_HIVE_ENTRY) ||
//         !(Hive->FileUserName.Buffer))
//     {
//         /* It isn't */
//         ObDereferenceObject(KeyBody);
//         return Loaded;
//     }
//
//     /* Now compare the name of the file */
//     if (!RtlCompareUnicodeString(&Hive->FileUserName,
//                                  SourceFile->ObjectName,
//                                  TRUE))
//     {
//         /* Same file found */
//         Loaded = TRUE;
//         *CmHive = Hive;
//
//         /* If the hive is frozen, not sure what to do */
//         if (Hive->Frozen)
//         {
//             /* FIXME: TODO */
//             UNIMPLEMENTED_DBGBREAK("ERROR: Hive is frozen\n");
//         }
//      }
//
//      /* Dereference and return result */
//      ObDereferenceObject(KeyBody);
//      return Loaded;
//  }
//
// BOOLEAN
// NTAPI
// CmpDoFlushAll(IN BOOLEAN ForceFlush)
// {
//     PLIST_ENTRY NextEntry;
//     PCMHIVE Hive;
//     NTSTATUS Status;
//     BOOLEAN Result = TRUE;
//
//     /* Make sure that the registry isn't read-only now */
//     if (CmpNoWrite) return TRUE;
//
//     /* Otherwise, acquire the hive list lock and disable force flush */
//     CmpForceForceFlush = FALSE;
//     ExAcquirePushLockShared(&CmpHiveListHeadLock);
//
//     /* Loop the hive list */
//     NextEntry = CmpHiveListHead.Flink;
//     while (NextEntry != &CmpHiveListHead)
//     {
//         /* Get the hive */
//         Hive = CONTAINING_RECORD(NextEntry, CMHIVE, HiveList);
//         if (!(Hive->Hive.HiveFlags & HIVE_NOLAZYFLUSH))
//         {
//             /* Acquire the flusher lock */
//             CmpLockHiveFlusherExclusive(Hive);
//
//             /* Check for illegal state */
//             if (ForceFlush && Hive->UseCount)
//             {
//                 /* Registry needs to be locked down */
//                 CMP_ASSERT_EXCLUSIVE_REGISTRY_LOCK();
//                 UNIMPLEMENTED_DBGBREAK("FIXME: Hive is damaged and needs fixup\n");
//             }
//
//             /* Only sync if we are forced to or if it won't cause a hive shrink */
//             if (ForceFlush || !HvHiveWillShrink(&Hive->Hive))
//             {
//                 /* Do the sync */
//                 Status = HvSyncHive(&Hive->Hive);
//
//                 /* If something failed - set the flag and continue looping */
//                 if (!NT_SUCCESS(Status))
//                     Result = FALSE;
//             }
//             else
//             {
//                 /* We won't flush if the hive might shrink */
//                 Result = FALSE;
//                 CmpForceForceFlush = TRUE;
//             }
//
//             /* Release the flusher lock */
//             CmpUnlockHiveFlusher(Hive);
//         }
//
//         /* Try the next entry */
//         NextEntry = NextEntry->Flink;
//     }
//
//     /* Release lock and return */
//     ExReleasePushLock(&CmpHiveListHeadLock);
//     return Result;
// }
//
// NTSTATUS
// NTAPI
// CmpSetValueKeyNew(IN PHHIVE Hive,
//                   IN PCM_KEY_NODE Parent,
//                   IN PUNICODE_STRING ValueName,
//                   IN ULONG Index,
//                   IN ULONG Type,
//                   IN PVOID Data,
//                   IN ULONG DataSize,
//                   IN ULONG StorageType,
//                   IN ULONG SmallData)
// {
//     PCELL_DATA CellData;
//     HCELL_INDEX ValueCell;
//     NTSTATUS Status;
//
//     /* Check if we already have a value list */
//     if (Parent->ValueList.Count)
//     {
//         /* Then make sure it's valid and dirty it */
//         ASSERT(Parent->ValueList.List != HCELL_NIL);
//         if (!HvMarkCellDirty(Hive, Parent->ValueList.List, FALSE))
//         {
//             /* Fail if we're out of space for log changes */
//             return STATUS_NO_LOG_SPACE;
//         }
//     }
//
//     /* Allocate a value cell */
//     ValueCell = HvAllocateCell(Hive,
//                                FIELD_OFFSET(CM_KEY_VALUE, Name) +
//                                CmpNameSize(Hive, ValueName),
//                                StorageType,
//                                HCELL_NIL);
//     if (ValueCell == HCELL_NIL) return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Get the actual data for it */
//     CellData = HvGetCell(Hive, ValueCell);
//     ASSERT(CellData);
//
//     /* Now we can release it, make sure it's also dirty */
//     HvReleaseCell(Hive, ValueCell);
//     ASSERT(HvIsCellDirty(Hive, ValueCell));
//
//     /* Set it up and copy the name */
//     CellData->u.KeyValue.Signature = CM_KEY_VALUE_SIGNATURE;
//     _SEH2_TRY
//     {
//         /* This can crash since the name is coming from user-mode */
//         CellData->u.KeyValue.NameLength = CmpCopyName(Hive,
//                                                       CellData->u.KeyValue.Name,
//                                                       ValueName);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         /* Fail */
//         DPRINT1("Invalid user data!\n");
//         HvFreeCell(Hive, ValueCell);
//         _SEH2_YIELD(return _SEH2_GetExceptionCode());
//     }
//     _SEH2_END;
//
//     /* Check for compressed name */
//     if (CellData->u.KeyValue.NameLength < ValueName->Length)
//     {
//         /* This is a compressed name */
//         CellData->u.KeyValue.Flags = VALUE_COMP_NAME;
//     }
//     else
//     {
//         /* No flags to set */
//         CellData->u.KeyValue.Flags = 0;
//     }
//
//     /* Check if this is a normal key */
//     if (DataSize > CM_KEY_VALUE_SMALL)
//     {
//         /* Build a data cell for it */
//         Status = CmpSetValueDataNew(Hive,
//                                     Data,
//                                     DataSize,
//                                     StorageType,
//                                     ValueCell,
//                                     &CellData->u.KeyValue.Data);
//         if (!NT_SUCCESS(Status))
//         {
//             /* We failed, free the cell */
//             HvFreeCell(Hive, ValueCell);
//             return Status;
//         }
//
//         /* Otherwise, set the data length, and make sure the data is dirty */
//         CellData->u.KeyValue.DataLength = DataSize;
//         ASSERT(HvIsCellDirty(Hive, CellData->u.KeyValue.Data));
//     }
//     else
//     {
//         /* This is a small key, set the data directly inside */
//         CellData->u.KeyValue.DataLength = DataSize + CM_KEY_VALUE_SPECIAL_SIZE;
//         CellData->u.KeyValue.Data = SmallData;
//     }
//
//     /* Set the type now */
//     CellData->u.KeyValue.Type = Type;
//
//     /* Add this value cell to the child list */
//     Status = CmpAddValueToList(Hive,
//                                ValueCell,
//                                Index,
//                                StorageType,
//                                &Parent->ValueList);
//
//     /* If we failed, free the entire cell, including the data */
//     if (!NT_SUCCESS(Status))
//     {
//         /* Overwrite the status with a known one */
//         CmpFreeValue(Hive, ValueCell);
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Return Status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// CmpSetValueKeyExisting(IN PHHIVE Hive,
//                        IN HCELL_INDEX OldChild,
//                        IN PCM_KEY_VALUE Value,
//                        IN ULONG Type,
//                        IN PVOID Data,
//                        IN ULONG DataSize,
//                        IN ULONG StorageType,
//                        IN ULONG TempData)
// {
//     HCELL_INDEX DataCell, NewCell;
//     PCELL_DATA CellData;
//     ULONG Length;
//     BOOLEAN WasSmall, IsSmall;
//
//     /* Registry writes must be blocked */
//     CMP_ASSERT_FLUSH_LOCK(Hive);
//
//     /* Mark the old child cell dirty */
//     if (!HvMarkCellDirty(Hive, OldChild, FALSE)) return STATUS_NO_LOG_SPACE;
//
//     /* See if this is a small or normal key */
//     WasSmall = CmpIsKeyValueSmall(&Length, Value->DataLength);
//
//     /* See if our new data can fit in a small key */
//     IsSmall = (DataSize <= CM_KEY_VALUE_SMALL) ? TRUE: FALSE;
//
//     /* Big keys are unsupported */
//     ASSERT_VALUE_BIG(Hive, Length);
//     ASSERT_VALUE_BIG(Hive, DataSize);
//
//     /* Mark the old value dirty */
//     if (!CmpMarkValueDataDirty(Hive, Value)) return STATUS_NO_LOG_SPACE;
//
//     /* Check if we have a small key */
//     if (IsSmall)
//     {
//         /* Check if we had a normal key with some data in it */
//         if (!(WasSmall) && (Length > 0))
//         {
//             /* Free the previous data */
//             CmpFreeValueData(Hive, Value->Data, Length);
//         }
//
//         /* Write our data directly */
//         Value->DataLength = DataSize + CM_KEY_VALUE_SPECIAL_SIZE;
//         Value->Data = TempData;
//         Value->Type = Type;
//         return STATUS_SUCCESS;
//     }
//
//     /* We have a normal key. Was the old cell also normal and had data? */
//     if (!(WasSmall) && (Length > 0))
//     {
//         /* Get the current data cell and actual data inside it */
//         DataCell = Value->Data;
//         ASSERT(DataCell != HCELL_NIL);
//         CellData = HvGetCell(Hive, DataCell);
//         if (!CellData) return STATUS_INSUFFICIENT_RESOURCES;
//
//         /* Immediately release the cell */
//         HvReleaseCell(Hive, DataCell);
//
//         /* Make sure that the data cell actually has a size */
//         ASSERT(HvGetCellSize(Hive, CellData) > 0);
//
//         /* Check if the previous data cell could fit our new data */
//         if (DataSize <= (ULONG)(HvGetCellSize(Hive, CellData)))
//         {
//             /* Re-use it then */
//             NewCell = DataCell;
//         }
//         else
//         {
//             /* Otherwise, re-allocate the current data cell */
//             NewCell = HvReallocateCell(Hive, DataCell, DataSize);
//             if (NewCell == HCELL_NIL) return STATUS_INSUFFICIENT_RESOURCES;
//         }
//     }
//     else
//     {
//         /* This was a small key, or a key with no data, allocate a cell */
//         NewCell = HvAllocateCell(Hive, DataSize, StorageType, HCELL_NIL);
//         if (NewCell == HCELL_NIL) return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Now get the actual data for our data cell */
//     CellData = HvGetCell(Hive, NewCell);
//     ASSERT(CellData);
//
//     /* Release it immediately */
//     HvReleaseCell(Hive, NewCell);
//
//     /* Copy our data into the data cell's buffer, and set up the value */
//     RtlCopyMemory(CellData, Data, DataSize);
//     Value->Data = NewCell;
//     Value->DataLength = DataSize;
//     Value->Type = Type;
//
//     /* Return success */
//     ASSERT(HvIsCellDirty(Hive, NewCell));
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// CmpQueryKeyData(IN PHHIVE Hive,
//                 IN PCM_KEY_NODE Node,
//                 IN KEY_INFORMATION_CLASS KeyInformationClass,
//                 IN OUT PVOID KeyInformation,
//                 IN ULONG Length,
//                 IN OUT PULONG ResultLength)
// {
//     NTSTATUS Status;
//     ULONG Size, SizeLeft, MinimumSize, Offset;
//     PKEY_INFORMATION Info = (PKEY_INFORMATION)KeyInformation;
//     USHORT NameLength;
//     PVOID ClassData;
//
//     /* Check if the value is compressed */
//     if (Node->Flags & KEY_COMP_NAME)
//     {
//         /* Get the compressed name size */
//         NameLength = CmpCompressedNameSize(Node->Name, Node->NameLength);
//     }
//     else
//     {
//         /* Get the real size */
//         NameLength = Node->NameLength;
//     }
//
//     /* Check what kind of information is being requested */
//     switch (KeyInformationClass)
//     {
//         /* Basic information */
//         case KeyBasicInformation:
//
//             /* This is the size we need */
//             Size = FIELD_OFFSET(KEY_BASIC_INFORMATION, Name) + NameLength;
//
//             /* And this is the minimum we can work with */
//             MinimumSize = FIELD_OFFSET(KEY_BASIC_INFORMATION, Name);
//
//             /* Let the caller know and assume success */
//             *ResultLength = Size;
//             Status = STATUS_SUCCESS;
//
//             /* Check if the bufer we got is too small */
//             if (Length < MinimumSize)
//             {
//                 /* Let the caller know and fail */
//                 Status = STATUS_BUFFER_TOO_SMALL;
//                 break;
//             }
//
//             /* Copy the basic information */
//             Info->KeyBasicInformation.LastWriteTime = Node->LastWriteTime;
//             Info->KeyBasicInformation.TitleIndex = 0;
//             Info->KeyBasicInformation.NameLength = NameLength;
//
//             /* Only the name is left */
//             SizeLeft = Length - MinimumSize;
//             Size = NameLength;
//
//             /* Check if we don't have enough space for the name */
//             if (SizeLeft < Size)
//             {
//                 /* Truncate the name we'll return, and tell the caller */
//                 Size = SizeLeft;
//                 Status = STATUS_BUFFER_OVERFLOW;
//             }
//
//             /* Check if this is a compressed key */
//             if (Node->Flags & KEY_COMP_NAME)
//             {
//                 /* Copy the compressed name */
//                 CmpCopyCompressedName(Info->KeyBasicInformation.Name,
//                                       SizeLeft,
//                                       Node->Name,
//                                       Node->NameLength);
//             }
//             else
//             {
//                 /* Otherwise, copy the raw name */
//                 RtlCopyMemory(Info->KeyBasicInformation.Name,
//                               Node->Name,
//                               Size);
//             }
//             break;
//
//         /* Node information */
//         case KeyNodeInformation:
//
//             /* Calculate the size we need */
//             Size = FIELD_OFFSET(KEY_NODE_INFORMATION, Name) +
//                    NameLength +
//                    Node->ClassLength;
//
//             /* And the minimum size we can support */
//             MinimumSize = FIELD_OFFSET(KEY_NODE_INFORMATION, Name);
//
//             /* Return the size to the caller and assume succes */
//             *ResultLength = Size;
//             Status = STATUS_SUCCESS;
//
//             /* Check if the caller's buffer is too small */
//             if (Length < MinimumSize)
//             {
//                 /* Let them know, and fail */
//                 Status = STATUS_BUFFER_TOO_SMALL;
//                 break;
//             }
//
//             /* Copy the basic information */
//             Info->KeyNodeInformation.LastWriteTime = Node->LastWriteTime;
//             Info->KeyNodeInformation.TitleIndex = 0;
//             Info->KeyNodeInformation.ClassLength = Node->ClassLength;
//             Info->KeyNodeInformation.NameLength = NameLength;
//
//             /* Now the name is left */
//             SizeLeft = Length - MinimumSize;
//             Size = NameLength;
//
//             /* Check if the name can fit entirely */
//             if (SizeLeft < Size)
//             {
//                 /* It can't, we'll have to truncate. Tell the caller */
//                 Size = SizeLeft;
//                 Status = STATUS_BUFFER_OVERFLOW;
//             }
//
//             /* Check if the key node name is compressed */
//             if (Node->Flags & KEY_COMP_NAME)
//             {
//                 /* Copy the compressed name */
//                 CmpCopyCompressedName(Info->KeyNodeInformation.Name,
//                                       SizeLeft,
//                                       Node->Name,
//                                       Node->NameLength);
//             }
//             else
//             {
//                 /* It isn't, so copy the raw name */
//                 RtlCopyMemory(Info->KeyNodeInformation.Name,
//                               Node->Name,
//                               Size);
//             }
//
//             /* Check if the node has a class */
//             if (Node->ClassLength > 0)
//             {
//                 /* Set the class offset */
//                 Offset = FIELD_OFFSET(KEY_NODE_INFORMATION, Name) + NameLength;
//                 Offset = ALIGN_UP_BY(Offset, sizeof(ULONG));
//                 Info->KeyNodeInformation.ClassOffset = Offset;
//
//                 /* Get the class data */
//                 ClassData = HvGetCell(Hive, Node->Class);
//                 if (ClassData == NULL)
//                 {
//                     Status = STATUS_INSUFFICIENT_RESOURCES;
//                     break;
//                 }
//
//                 /* Check if we can copy anything */
//                 if (Length > Offset)
//                 {
//                     /* Copy the class data */
//                     RtlCopyMemory((PUCHAR)Info + Offset,
//                                   ClassData,
//                                   min(Node->ClassLength, Length - Offset));
//                 }
//
//                 /* Check if the buffer was large enough */
//                 if (Length < Offset + Node->ClassLength)
//                 {
//                     Status = STATUS_BUFFER_OVERFLOW;
//                 }
//
//                 /* Release the class cell */
//                 HvReleaseCell(Hive, Node->Class);
//             }
//             else
//             {
//                 /* It doesn't, so set offset to -1, not 0! */
//                 Info->KeyNodeInformation.ClassOffset = 0xFFFFFFFF;
//             }
//             break;
//
//         /* Full information requsted */
//         case KeyFullInformation:
//
//             /* This is the size we need */
//             Size = FIELD_OFFSET(KEY_FULL_INFORMATION, Class) +
//                    Node->ClassLength;
//
//             /* This is what we can work with */
//             MinimumSize = FIELD_OFFSET(KEY_FULL_INFORMATION, Class);
//
//             /* Return it to caller and assume success */
//             *ResultLength = Size;
//             Status = STATUS_SUCCESS;
//
//             /* Check if the caller's buffer is to small */
//             if (Length < MinimumSize)
//             {
//                 /* Let them know and fail */
//                 Status = STATUS_BUFFER_TOO_SMALL;
//                 break;
//             }
//
//             /* Now copy all the basic information */
//             Info->KeyFullInformation.LastWriteTime = Node->LastWriteTime;
//             Info->KeyFullInformation.TitleIndex = 0;
//             Info->KeyFullInformation.ClassLength = Node->ClassLength;
//             Info->KeyFullInformation.SubKeys = Node->SubKeyCounts[Stable] +
//                                                Node->SubKeyCounts[Volatile];
//             Info->KeyFullInformation.Values = Node->ValueList.Count;
//             Info->KeyFullInformation.MaxNameLen = Node->MaxNameLen;
//             Info->KeyFullInformation.MaxClassLen = Node->MaxClassLen;
//             Info->KeyFullInformation.MaxValueNameLen = Node->MaxValueNameLen;
//             Info->KeyFullInformation.MaxValueDataLen = Node->MaxValueDataLen;
//
//             /* Check if we have a class */
//             if (Node->ClassLength > 0)
//             {
//                 /* Set the class offset */
//                 Offset = FIELD_OFFSET(KEY_FULL_INFORMATION, Class);
//                 Info->KeyFullInformation.ClassOffset = Offset;
//
//                 /* Get the class data */
//                 ClassData = HvGetCell(Hive, Node->Class);
//                 if (ClassData == NULL)
//                 {
//                     Status = STATUS_INSUFFICIENT_RESOURCES;
//                     break;
//                 }
//
//                 /* Copy the class data */
//                 ASSERT(Length >= Offset);
//                 RtlCopyMemory(Info->KeyFullInformation.Class,
//                               ClassData,
//                               min(Node->ClassLength, Length - Offset));
//
//                 /* Check if the buffer was large enough */
//                 if (Length < Offset + Node->ClassLength)
//                 {
//                     Status = STATUS_BUFFER_OVERFLOW;
//                 }
//
//                 /* Release the class cell */
//                 HvReleaseCell(Hive, Node->Class);
//             }
//             else
//             {
//                 /* We don't have a class, so set offset to -1, not 0! */
//                 Info->KeyFullInformation.ClassOffset = 0xFFFFFFFF;
//             }
//             break;
//
//         /* Any other class that got sent here is invalid! */
//         default:
//
//             /* Set failure code */
//             Status = STATUS_INVALID_PARAMETER;
//             break;
//     }
//
//     /* Return status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// CmSetValueKey(IN PCM_KEY_CONTROL_BLOCK Kcb,
//               IN PUNICODE_STRING ValueName,
//               IN ULONG Type,
//               IN PVOID Data,
//               IN ULONG DataLength)
// {
//     PHHIVE Hive = NULL;
//     PCM_KEY_NODE Parent;
//     PCM_KEY_VALUE Value = NULL;
//     HCELL_INDEX CurrentChild, Cell;
//     NTSTATUS Status;
//     BOOLEAN Found, Result;
//     ULONG Count, ChildIndex, SmallData, Storage;
//     VALUE_SEARCH_RETURN_TYPE SearchResult;
//     BOOLEAN FirstTry = TRUE, FlusherLocked = FALSE;
//     HCELL_INDEX ParentCell = HCELL_NIL, ChildCell = HCELL_NIL;
//
//     /* Acquire hive and KCB lock */
//     CmpLockRegistry();
//     CmpAcquireKcbLockShared(Kcb);
//
//     /* Sanity check */
//     ASSERT(sizeof(ULONG) == CM_KEY_VALUE_SMALL);
//
//     /* Don't touch deleted KCBs */
// DoAgain:
//     if (Kcb->Delete)
//     {
//         /* Fail */
//         Status = STATUS_KEY_DELETED;
//         goto Quickie;
//     }
//
//     /* Don't let anyone mess with symlinks */
//     if ((Kcb->Flags & KEY_SYM_LINK) &&
//         ((Type != REG_LINK) ||
//          !(ValueName) ||
//          !(RtlEqualUnicodeString(&CmSymbolicLinkValueName, ValueName, TRUE))))
//     {
//         /* Invalid modification of a symlink key */
//         Status = STATUS_ACCESS_DENIED;
//         goto Quickie;
//     }
//
//     /* Check if this is the first attempt */
//     if (FirstTry)
//     {
//         /* Search for the value in the cache */
//         SearchResult = CmpCompareNewValueDataAgainstKCBCache(Kcb,
//                                                              ValueName,
//                                                              Type,
//                                                              Data,
//                                                              DataLength);
//         if (SearchResult == SearchNeedExclusiveLock)
//         {
//             /* Try again with the exclusive lock */
//             CmpConvertKcbSharedToExclusive(Kcb);
//             goto DoAgain;
//         }
//         else if (SearchResult == SearchSuccess)
//         {
//             /* We don't actually need to do anything! */
//             Status = STATUS_SUCCESS;
//             goto Quickie;
//         }
//
//         /* We need the exclusive KCB lock now */
//         if (!CmpIsKcbLockedExclusive(Kcb) &&
//             !CmpTryToConvertKcbSharedToExclusive(Kcb))
//         {
//             /* Acquire exclusive lock */
//             CmpConvertKcbSharedToExclusive(Kcb);
//         }
//
//         /* Cache lookup failed, so don't try it next time */
//         FirstTry = FALSE;
//
//         /* Now grab the flush lock since the key will be modified */
//         ASSERT(FlusherLocked == FALSE);
//         CmpLockHiveFlusherShared((PCMHIVE)Kcb->KeyHive);
//         FlusherLocked = TRUE;
//         goto DoAgain;
//     }
//     else
//     {
//         /* Get pointer to key cell */
//         Hive = Kcb->KeyHive;
//         Cell = Kcb->KeyCell;
//
//         /* Get the parent */
//         Parent = (PCM_KEY_NODE)HvGetCell(Hive, Cell);
//         ASSERT(Parent);
//         ParentCell = Cell;
//
//         /* Prepare to scan the key node */
//         Count = Parent->ValueList.Count;
//         Found = FALSE;
//         if (Count > 0)
//         {
//             /* Try to find the existing name */
//             Result = CmpFindNameInList(Hive,
//                                        &Parent->ValueList,
//                                        ValueName,
//                                        &ChildIndex,
//                                        &CurrentChild);
//             if (!Result)
//             {
//                 /* Fail */
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 goto Quickie;
//             }
//
//             /* Check if we found something */
//             if (CurrentChild != HCELL_NIL)
//             {
//                 /* Release existing child */
//                 if (ChildCell != HCELL_NIL)
//                 {
//                     HvReleaseCell(Hive, ChildCell);
//                     ChildCell = HCELL_NIL;
//                 }
//
//                 /* Get its value */
//                 Value = (PCM_KEY_VALUE)HvGetCell(Hive, CurrentChild);
//                 if (!Value)
//                 {
//                     /* Fail */
//                     Status = STATUS_INSUFFICIENT_RESOURCES;
//                     goto Quickie;
//                 }
//
//                 /* Remember that we found it */
//                 ChildCell = CurrentChild;
//                 Found = TRUE;
//             }
//         }
//         else
//         {
//             /* No child list, we'll need to add it */
//             ChildIndex = 0;
//         }
//     }
//
//     /* Should only get here on the second pass */
//     ASSERT(FirstTry == FALSE);
//
//     /* The KCB must be locked exclusive at this point */
//     CMP_ASSERT_KCB_LOCK(Kcb);
//
//     /* Mark the cell dirty */
//     if (!HvMarkCellDirty(Hive, Cell, FALSE))
//     {
//         /* Not enough log space, fail */
//         Status = STATUS_NO_LOG_SPACE;
//         goto Quickie;
//     }
//
//     /* Get the storage type */
//     Storage = HvGetCellType(Cell);
//
//     /* Check if this is small data */
//     SmallData = 0;
//     if ((DataLength <= CM_KEY_VALUE_SMALL) && (DataLength > 0))
//     {
//         /* Need SEH because user data may be invalid */
//         _SEH2_TRY
//         {
//             /* Copy it */
//             RtlCopyMemory(&SmallData, Data, DataLength);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return failure code */
//             Status = _SEH2_GetExceptionCode();
//             _SEH2_YIELD(goto Quickie);
//         }
//         _SEH2_END;
//     }
//
//     /* Check if we didn't find a matching key */
//     if (!Found)
//     {
//         /* Call the internal routine */
//         Status = CmpSetValueKeyNew(Hive,
//                                    Parent,
//                                    ValueName,
//                                    ChildIndex,
//                                    Type,
//                                    Data,
//                                    DataLength,
//                                    Storage,
//                                    SmallData);
//     }
//     else
//     {
//         /* Call the internal routine */
//         Status = CmpSetValueKeyExisting(Hive,
//                                         CurrentChild,
//                                         Value,
//                                         Type,
//                                         Data,
//                                         DataLength,
//                                         Storage,
//                                         SmallData);
//     }
//
//     /* Check for success */
//     if (NT_SUCCESS(Status))
//     {
//         /* Check if the maximum value name length changed */
//         ASSERT(Parent->MaxValueNameLen == Kcb->KcbMaxValueNameLen);
//         if (Parent->MaxValueNameLen < ValueName->Length)
//         {
//             /* Set the new values */
//             Parent->MaxValueNameLen = ValueName->Length;
//             Kcb->KcbMaxValueNameLen = ValueName->Length;
//         }
//
//         /* Check if the maximum data length changed */
//         ASSERT(Parent->MaxValueDataLen == Kcb->KcbMaxValueDataLen);
//         if (Parent->MaxValueDataLen < DataLength)
//         {
//             /* Update it */
//             Parent->MaxValueDataLen = DataLength;
//             Kcb->KcbMaxValueDataLen = Parent->MaxValueDataLen;
//         }
//
//         /* Save the write time */
//         KeQuerySystemTime(&Parent->LastWriteTime);
//         Kcb->KcbLastWriteTime = Parent->LastWriteTime;
//
//         /* Check if the cell is cached */
//         if (Found && CMP_IS_CELL_CACHED(Kcb->ValueCache.ValueList))
//         {
//             /* Shouldn't happen */
//             ASSERT(FALSE);
//         }
//         else
//         {
//             /* Cleanup the value cache */
//             CmpCleanUpKcbValueCache(Kcb);
//
//             /* Sanity checks */
//             ASSERT(!CMP_IS_CELL_CACHED(Kcb->ValueCache.ValueList));
//             ASSERT(!(Kcb->ExtFlags & CM_KCB_SYM_LINK_FOUND));
//
//             /* Set the value cache */
//             Kcb->ValueCache.Count = Parent->ValueList.Count;
//             Kcb->ValueCache.ValueList = Parent->ValueList.List;
//         }
//
//         /* Notify registered callbacks */
//         CmpReportNotify(Kcb,
//                         Hive,
//                         Kcb->KeyCell,
//                         REG_NOTIFY_CHANGE_LAST_SET);
//     }
//
//     /* Release the cells */
// Quickie:
//     if ((ParentCell != HCELL_NIL) && Hive) HvReleaseCell(Hive, ParentCell);
//     if ((ChildCell != HCELL_NIL) && Hive) HvReleaseCell(Hive, ChildCell);
//
//     /* Release the locks */
//     if (FlusherLocked) CmpUnlockHiveFlusher((PCMHIVE)Hive);
//     CmpReleaseKcbLock(Kcb);
//     CmpUnlockRegistry();
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// CmDeleteValueKey(IN PCM_KEY_CONTROL_BLOCK Kcb,
//                  IN UNICODE_STRING ValueName)
// {
//     NTSTATUS Status = STATUS_OBJECT_NAME_NOT_FOUND;
//     PHHIVE Hive;
//     PCM_KEY_NODE Parent;
//     HCELL_INDEX ChildCell, Cell;
//     PCHILD_LIST ChildList;
//     PCM_KEY_VALUE Value = NULL;
//     ULONG ChildIndex;
//     BOOLEAN Result;
//
//     /* Acquire hive lock */
//     CmpLockRegistry();
//
//     /* Lock KCB exclusively */
//     CmpAcquireKcbLockExclusive(Kcb);
//
//     /* Don't touch deleted keys */
//     if (Kcb->Delete)
//     {
//         /* Undo everything */
//         CmpReleaseKcbLock(Kcb);
//         CmpUnlockRegistry();
//         return STATUS_KEY_DELETED;
//     }
//
//     /* Get the hive and the cell index */
//     Hive = Kcb->KeyHive;
//     Cell = Kcb->KeyCell;
//
//     /* Lock flushes */
//     CmpLockHiveFlusherShared((PCMHIVE)Hive);
//
//     /* Get the parent key node */
//     Parent = (PCM_KEY_NODE)HvGetCell(Hive, Cell);
//     ASSERT(Parent);
//
//     /* Get the value list and check if it has any entries */
//     ChildList = &Parent->ValueList;
//     if (ChildList->Count)
//     {
//         /* Try to find this value */
//         Result = CmpFindNameInList(Hive,
//                                    ChildList,
//                                    &ValueName,
//                                    &ChildIndex,
//                                    &ChildCell);
//         if (!Result)
//         {
//             /* Fail */
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto Quickie;
//         }
//
//         /* Value not found, return error */
//         if (ChildCell == HCELL_NIL) goto Quickie;
//
//         /* We found the value, mark all relevant cells dirty */
//         if (!(HvMarkCellDirty(Hive, Cell, FALSE) &&
//               HvMarkCellDirty(Hive, Parent->ValueList.List, FALSE) &&
//               HvMarkCellDirty(Hive, ChildCell, FALSE)))
//         {
//             /* Not enough log space, fail */
//             Status = STATUS_NO_LOG_SPACE;
//             goto Quickie;
//         }
//
//         /* Get the key value */
//         Value = (PCM_KEY_VALUE)HvGetCell(Hive, ChildCell);
//         ASSERT(Value);
//
//         /* Mark it and all related data as dirty */
//         if (!CmpMarkValueDataDirty(Hive, Value))
//         {
//             /* Not enough log space, fail */
//             Status = STATUS_NO_LOG_SPACE;
//             goto Quickie;
//         }
//
//         /* Sanity checks */
//         ASSERT(HvIsCellDirty(Hive, Parent->ValueList.List));
//         ASSERT(HvIsCellDirty(Hive, ChildCell));
//
//         /* Remove the value from the child list */
//         Status = CmpRemoveValueFromList(Hive, ChildIndex, ChildList);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Set known error */
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto Quickie;
//         }
//
//         /* Remove the value and its data itself */
//         if (!CmpFreeValue(Hive, ChildCell))
//         {
//             /* Failed to free the value, fail */
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto Quickie;
//         }
//
//         /* Set the last write time */
//         KeQuerySystemTime(&Parent->LastWriteTime);
//         Kcb->KcbLastWriteTime = Parent->LastWriteTime;
//
//         /* Sanity check */
//         ASSERT(Parent->MaxValueNameLen == Kcb->KcbMaxValueNameLen);
//         ASSERT(Parent->MaxValueDataLen == Kcb->KcbMaxValueDataLen);
//         ASSERT(HvIsCellDirty(Hive, Cell));
//
//         /* Check if the value list is empty now */
//         if (!Parent->ValueList.Count)
//         {
//             /* Then clear key node data */
//             Parent->MaxValueNameLen = 0;
//             Parent->MaxValueDataLen = 0;
//             Kcb->KcbMaxValueNameLen = 0;
//             Kcb->KcbMaxValueDataLen = 0;
//         }
//
//         /* Cleanup the value cache */
//         CmpCleanUpKcbValueCache(Kcb);
//
//         /* Sanity checks */
//         ASSERT(!CMP_IS_CELL_CACHED(Kcb->ValueCache.ValueList));
//         ASSERT(!(Kcb->ExtFlags & CM_KCB_SYM_LINK_FOUND));
//
//         /* Set the value cache */
//         Kcb->ValueCache.Count = ChildList->Count;
//         Kcb->ValueCache.ValueList = ChildList->List;
//
//         /* Notify registered callbacks */
//         CmpReportNotify(Kcb, Hive, Cell, REG_NOTIFY_CHANGE_LAST_SET);
//
//         /* Change default Status to success */
//         Status = STATUS_SUCCESS;
//     }
//
// Quickie:
//     /* Release the parent cell, if any */
//     if (Parent) HvReleaseCell(Hive, Cell);
//
//     /* Check if we had a value */
//     if (Value)
//     {
//         /* Release the child cell */
//         ASSERT(ChildCell != HCELL_NIL);
//         HvReleaseCell(Hive, ChildCell);
//     }
//
//     /* Release locks */
//     CmpUnlockHiveFlusher((PCMHIVE)Hive);
//     CmpReleaseKcbLock(Kcb);
//     CmpUnlockRegistry();
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// CmQueryValueKey(IN PCM_KEY_CONTROL_BLOCK Kcb,
//                 IN UNICODE_STRING ValueName,
//                 IN KEY_VALUE_INFORMATION_CLASS KeyValueInformationClass,
//                 IN PVOID KeyValueInformation,
//                 IN ULONG Length,
//                 IN PULONG ResultLength)
// {
//     NTSTATUS Status;
//     PCM_KEY_VALUE ValueData;
//     ULONG Index;
//     BOOLEAN ValueCached = FALSE;
//     PCM_CACHED_VALUE *CachedValue;
//     HCELL_INDEX CellToRelease;
//     VALUE_SEARCH_RETURN_TYPE Result;
//     PHHIVE Hive;
//     PAGED_CODE();
//
//     /* Acquire hive lock */
//     CmpLockRegistry();
//
//     /* Lock the KCB shared */
//     CmpAcquireKcbLockShared(Kcb);
//
//     /* Don't touch deleted keys */
// DoAgain:
//     if (Kcb->Delete)
//     {
//         /* Undo everything */
//         CmpReleaseKcbLock(Kcb);
//         CmpUnlockRegistry();
//         return STATUS_KEY_DELETED;
//     }
//
//     /* We don't deal with this yet */
//     if (Kcb->ExtFlags & CM_KCB_SYM_LINK_FOUND)
//     {
//         /* Shouldn't happen */
//         ASSERT(FALSE);
//     }
//
//     /* Get the hive */
//     Hive = Kcb->KeyHive;
//
//     /* Find the key value */
//     Result = CmpFindValueByNameFromCache(Kcb,
//                                          &ValueName,
//                                          &CachedValue,
//                                          &Index,
//                                          &ValueData,
//                                          &ValueCached,
//                                          &CellToRelease);
//     if (Result == SearchNeedExclusiveLock)
//     {
//         /* Check if we need an exclusive lock */
//         ASSERT(CellToRelease == HCELL_NIL);
//         ASSERT(ValueData == NULL);
//
//         /* Try with exclusive KCB lock */
//         CmpConvertKcbSharedToExclusive(Kcb);
//         goto DoAgain;
//     }
//
//     if (Result == SearchSuccess)
//     {
//         /* Sanity check */
//         ASSERT(ValueData != NULL);
//
//         /* User data, protect against exceptions */
//         _SEH2_TRY
//         {
//             /* Query the information requested */
//             Result = CmpQueryKeyValueData(Kcb,
//                                           CachedValue,
//                                           ValueData,
//                                           ValueCached,
//                                           KeyValueInformationClass,
//                                           KeyValueInformation,
//                                           Length,
//                                           ResultLength,
//                                           &Status);
//             if (Result == SearchNeedExclusiveLock)
//             {
//                 /* Release the value cell */
//                 if (CellToRelease != HCELL_NIL)
//                 {
//                     HvReleaseCell(Hive, CellToRelease);
//                     CellToRelease = HCELL_NIL;
//                 }
//
//                 /* Try with exclusive KCB lock */
//                 CmpConvertKcbSharedToExclusive(Kcb);
//                 _SEH2_YIELD(goto DoAgain);
//             }
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//     }
//     else
//     {
//         /* Failed to find the value */
//         Status = STATUS_OBJECT_NAME_NOT_FOUND;
//     }
//
//     /* If we have a cell to release, do so */
//     if (CellToRelease != HCELL_NIL) HvReleaseCell(Hive, CellToRelease);
//
//     /* Release locks */
//     CmpReleaseKcbLock(Kcb);
//     CmpUnlockRegistry();
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// CmEnumerateValueKey(IN PCM_KEY_CONTROL_BLOCK Kcb,
//                     IN ULONG Index,
//                     IN KEY_VALUE_INFORMATION_CLASS KeyValueInformationClass,
//                     IN PVOID KeyValueInformation,
//                     IN ULONG Length,
//                     IN PULONG ResultLength)
// {
//     NTSTATUS Status;
//     PHHIVE Hive;
//     PCM_KEY_NODE Parent;
//     HCELL_INDEX CellToRelease = HCELL_NIL, CellToRelease2 = HCELL_NIL;
//     VALUE_SEARCH_RETURN_TYPE Result;
//     BOOLEAN IndexIsCached, ValueIsCached = FALSE;
//     PCELL_DATA CellData;
//     PCM_CACHED_VALUE *CachedValue;
//     PCM_KEY_VALUE ValueData = NULL;
//     PAGED_CODE();
//
//     /* Acquire hive lock */
//     CmpLockRegistry();
//
//     /* Lock the KCB shared */
//     CmpAcquireKcbLockShared(Kcb);
//
//     /* Don't touch deleted keys */
// DoAgain:
//     if (Kcb->Delete)
//     {
//         /* Undo everything */
//         CmpReleaseKcbLock(Kcb);
//         CmpUnlockRegistry();
//         return STATUS_KEY_DELETED;
//     }
//
//     /* Get the hive and parent */
//     Hive = Kcb->KeyHive;
//     Parent = (PCM_KEY_NODE)HvGetCell(Hive, Kcb->KeyCell);
//     ASSERT(Parent);
//
//     /* FIXME: Lack of cache? */
//     if (Kcb->ValueCache.Count != Parent->ValueList.Count)
//     {
//         DPRINT1("HACK: Overriding value cache count\n");
//         Kcb->ValueCache.Count = Parent->ValueList.Count;
//     }
//
//     /* Make sure the index is valid */
//     if (Index >= Kcb->ValueCache.Count)
//     {
//         /* Release the cell and fail */
//         HvReleaseCell(Hive, Kcb->KeyCell);
//         Status = STATUS_NO_MORE_ENTRIES;
//         goto Quickie;
//     }
//
//     /* We don't deal with this yet */
//     if (Kcb->ExtFlags & CM_KCB_SYM_LINK_FOUND)
//     {
//         /* Shouldn't happen */
//         ASSERT(FALSE);
//     }
//
//     /* Find the value list */
//     Result = CmpGetValueListFromCache(Kcb,
//                                       &CellData,
//                                       &IndexIsCached,
//                                       &CellToRelease);
//     if (Result == SearchNeedExclusiveLock)
//     {
//         /* Check if we need an exclusive lock */
//         ASSERT(CellToRelease == HCELL_NIL);
//         HvReleaseCell(Hive, Kcb->KeyCell);
//
//         /* Try with exclusive KCB lock */
//         CmpConvertKcbSharedToExclusive(Kcb);
//         goto DoAgain;
//     }
//     else if (Result != SearchSuccess)
//     {
//         /* Sanity check */
//         ASSERT(CellData == NULL);
//
//         /* Release the cell and fail */
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto Quickie;
//     }
//
//     /* Now get the key value */
//     Result = CmpGetValueKeyFromCache(Kcb,
//                                      CellData,
//                                      Index,
//                                      &CachedValue,
//                                      &ValueData,
//                                      IndexIsCached,
//                                      &ValueIsCached,
//                                      &CellToRelease2);
//     if (Result == SearchNeedExclusiveLock)
//     {
//         /* Cleanup state */
//         ASSERT(CellToRelease2 == HCELL_NIL);
//         if (CellToRelease)
//         {
//             HvReleaseCell(Hive, CellToRelease);
//             CellToRelease = HCELL_NIL;
//         }
//         HvReleaseCell(Hive, Kcb->KeyCell);
//
//         /* Try with exclusive KCB lock */
//         CmpConvertKcbSharedToExclusive(Kcb);
//         goto DoAgain;
//     }
//     else if (Result != SearchSuccess)
//     {
//         /* Sanity check */
//         ASSERT(ValueData == NULL);
//
//         /* Release the cells and fail */
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto Quickie;
//     }
//
//     /* User data, need SEH */
//     _SEH2_TRY
//     {
//         /* Query the information requested */
//         Result = CmpQueryKeyValueData(Kcb,
//                                       CachedValue,
//                                       ValueData,
//                                       ValueIsCached,
//                                       KeyValueInformationClass,
//                                       KeyValueInformation,
//                                       Length,
//                                       ResultLength,
//                                       &Status);
//         if (Result == SearchNeedExclusiveLock)
//         {
//             /* Cleanup state */
//             if (CellToRelease2) HvReleaseCell(Hive, CellToRelease2);
//             HvReleaseCell(Hive, Kcb->KeyCell);
//             if (CellToRelease) HvReleaseCell(Hive, CellToRelease);
//
//             /* Try with exclusive KCB lock */
//             CmpConvertKcbSharedToExclusive(Kcb);
//             _SEH2_YIELD(goto DoAgain);
//         }
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         /* Get exception code */
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
// Quickie:
//     /* If we have a cell to release, do so */
//     if (CellToRelease != HCELL_NIL) HvReleaseCell(Hive, CellToRelease);
//
//     /* Release the parent cell */
//     HvReleaseCell(Hive, Kcb->KeyCell);
//
//     /* If we have a cell to release, do so */
//     if (CellToRelease2 != HCELL_NIL) HvReleaseCell(Hive, CellToRelease2);
//
//     /* Release locks */
//     CmpReleaseKcbLock(Kcb);
//     CmpUnlockRegistry();
//     return Status;
// }
//
// static
// NTSTATUS
// CmpQueryKeyDataFromCache(
//     _In_ PCM_KEY_CONTROL_BLOCK Kcb,
//     _Out_ PKEY_CACHED_INFORMATION KeyCachedInfo,
//     _In_ ULONG Length,
//     _Out_ PULONG ResultLength)
// {
//     PCM_KEY_NODE Node;
//     PHHIVE KeyHive;
//     HCELL_INDEX KeyCell;
//     USHORT NameLength;
//     PAGED_CODE();
//
//     /* Get the hive and cell index */
//     KeyHive = Kcb->KeyHash.KeyHive;
//     KeyCell = Kcb->KeyHash.KeyCell;
//
// #if DBG
//     /* Get the cell node */
//     Node = (PCM_KEY_NODE)HvGetCell(KeyHive, KeyCell);
//     if (Node != NULL)
//     {
//         ULONG SubKeyCount;
//         ASSERT(Node->ValueList.Count == Kcb->ValueCache.Count);
//
//         if (!(Kcb->ExtFlags & CM_KCB_INVALID_CACHED_INFO))
//         {
//             SubKeyCount = Node->SubKeyCounts[0] + Node->SubKeyCounts[1];
//             if (Kcb->ExtFlags & CM_KCB_NO_SUBKEY)
//             {
//                 ASSERT(SubKeyCount == 0);
//             }
//             else if (Kcb->ExtFlags & CM_KCB_SUBKEY_ONE)
//             {
//                 ASSERT(SubKeyCount == 1);
//             }
//             else if (Kcb->ExtFlags & CM_KCB_SUBKEY_HINT)
//             {
//                 ASSERT(SubKeyCount == Kcb->IndexHint->Count);
//             }
//             else
//             {
//                 ASSERT(SubKeyCount == Kcb->SubKeyCount);
//             }
//         }
//
//         ASSERT(Node->LastWriteTime.QuadPart == Kcb->KcbLastWriteTime.QuadPart);
//         ASSERT(Node->MaxNameLen == Kcb->KcbMaxNameLen);
//         ASSERT(Node->MaxValueNameLen == Kcb->KcbMaxValueNameLen);
//         ASSERT(Node->MaxValueDataLen == Kcb->KcbMaxValueDataLen);
//
//         /* Release the cell */
//         HvReleaseCell(KeyHive, KeyCell);
//     }
// #endif // DBG
//
//     /* Make sure we have a name block */
//     if (Kcb->NameBlock == NULL)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Check for compressed name */
//     if (Kcb->NameBlock->Compressed)
//     {
//         /* Calculate the name size */
//         NameLength = CmpCompressedNameSize(Kcb->NameBlock->NameHash.Name,
//                                            Kcb->NameBlock->NameHash.NameLength);
//     }
//     else
//     {
//         /* Use the stored name size */
//         NameLength = Kcb->NameBlock->NameHash.NameLength;
//     }
//
//     /* Validate buffer length (we do not copy the name!) */
//     *ResultLength = sizeof(*KeyCachedInfo);
//     if (Length < *ResultLength)
//     {
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
//     /* Fill the structure */
//     KeyCachedInfo->LastWriteTime = Kcb->KcbLastWriteTime;
//     KeyCachedInfo->TitleIndex = 0;
//     KeyCachedInfo->NameLength = NameLength;
//     KeyCachedInfo->Values = Kcb->ValueCache.Count;
//     KeyCachedInfo->MaxNameLen = Kcb->KcbMaxNameLen;
//     KeyCachedInfo->MaxValueNameLen = Kcb->KcbMaxValueNameLen;
//     KeyCachedInfo->MaxValueDataLen = Kcb->KcbMaxValueDataLen;
//
//     /* Check the ExtFlags for what we have */
//     if (Kcb->ExtFlags & CM_KCB_INVALID_CACHED_INFO)
//     {
//         /* Cache is not valid, do a full lookup */
//         DPRINT1("Kcb cache incoherency detected, kcb = %p\n", Kcb);
//
//         /* Get the cell node */
//         Node = (PCM_KEY_NODE)HvGetCell(KeyHive, KeyCell);
//         if (Node == NULL)
//         {
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         /* Calculate number of subkeys */
//         KeyCachedInfo->SubKeys = Node->SubKeyCounts[0] + Node->SubKeyCounts[1];
//
//         /* Release the cell */
//         HvReleaseCell(KeyHive, KeyCell);
//     }
//     else if (Kcb->ExtFlags & CM_KCB_NO_SUBKEY)
//     {
//         /* There are no subkeys */
//         KeyCachedInfo->SubKeys = 0;
//     }
//     else if (Kcb->ExtFlags & CM_KCB_SUBKEY_ONE)
//     {
//         /* There is exactly one subley */
//         KeyCachedInfo->SubKeys = 1;
//     }
//     else if (Kcb->ExtFlags & CM_KCB_SUBKEY_HINT)
//     {
//         /* Get the number of subkeys from the subkey hint */
//         KeyCachedInfo->SubKeys = Kcb->IndexHint->Count;
//     }
//     else
//     {
//         /* No subkey hint, use the key count field */
//         KeyCachedInfo->SubKeys = Kcb->SubKeyCount;
//     }
//
//     return STATUS_SUCCESS;
// }
//
// static
// NTSTATUS
// CmpQueryFlagsInformation(
//     _In_ PCM_KEY_CONTROL_BLOCK Kcb,
//     _Out_ PKEY_USER_FLAGS_INFORMATION KeyFlagsInfo,
//     _In_ ULONG Length,
//     _In_ PULONG ResultLength)
// {
//     /* Validate the buffer size */
//     *ResultLength = sizeof(*KeyFlagsInfo);
//     if (Length < *ResultLength)
//     {
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
//     /* Copy the user flags */
//     KeyFlagsInfo->UserFlags = Kcb->KcbUserFlags;
//
//     return STATUS_SUCCESS;
// }
//
// static
// NTSTATUS
// CmpQueryNameInformation(
//     _In_ PCM_KEY_CONTROL_BLOCK Kcb,
//     _Out_opt_ PKEY_NAME_INFORMATION KeyNameInfo,
//     _In_ ULONG Length,
//     _Out_ PULONG ResultLength)
// {
//     ULONG NeededLength;
//     PCM_KEY_CONTROL_BLOCK CurrentKcb;
//
//     NeededLength = 0;
//     CurrentKcb = Kcb;
//
//     /* Count the needed buffer size */
//     while (CurrentKcb)
//     {
//         if (CurrentKcb->NameBlock->Compressed)
//             NeededLength += CmpCompressedNameSize(CurrentKcb->NameBlock->Name, CurrentKcb->NameBlock->NameLength);
//         else
//             NeededLength += CurrentKcb->NameBlock->NameLength;
//
//         NeededLength += sizeof(OBJ_NAME_PATH_SEPARATOR);
//
//         CurrentKcb = CurrentKcb->ParentKcb;
//     }
//
//     _SEH2_TRY
//     {
//         *ResultLength = FIELD_OFFSET(KEY_NAME_INFORMATION, Name) + NeededLength;
//         if (Length < RTL_SIZEOF_THROUGH_FIELD(KEY_NAME_INFORMATION, NameLength))
//             _SEH2_YIELD(return STATUS_BUFFER_TOO_SMALL);
//         if (Length < *ResultLength)
//         {
//             KeyNameInfo->NameLength = NeededLength;
//             _SEH2_YIELD(return STATUS_BUFFER_OVERFLOW);
//         }
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         _SEH2_YIELD(return _SEH2_GetExceptionCode());
//     }
//     _SEH2_END;
//
//     /* Do the real copy */
//     KeyNameInfo->NameLength = 0;
//     CurrentKcb = Kcb;
//
//     _SEH2_TRY
//     {
//         while (CurrentKcb)
//         {
//             ULONG NameLength;
//
//             if (CurrentKcb->NameBlock->Compressed)
//             {
//                 NameLength = CmpCompressedNameSize(CurrentKcb->NameBlock->Name, CurrentKcb->NameBlock->NameLength);
//                 /* Copy the compressed name */
//                 CmpCopyCompressedName(&KeyNameInfo->Name[(NeededLength - NameLength)/sizeof(WCHAR)],
//                                       NameLength,
//                                       CurrentKcb->NameBlock->Name,
//                                       CurrentKcb->NameBlock->NameLength);
//             }
//             else
//             {
//                 NameLength = CurrentKcb->NameBlock->NameLength;
//                 /* Otherwise, copy the raw name */
//                 RtlCopyMemory(&KeyNameInfo->Name[(NeededLength - NameLength)/sizeof(WCHAR)],
//                               CurrentKcb->NameBlock->Name,
//                               NameLength);
//             }
//
//             NeededLength -= NameLength;
//             NeededLength -= sizeof(OBJ_NAME_PATH_SEPARATOR);
//             /* Add path separator */
//             KeyNameInfo->Name[NeededLength/sizeof(WCHAR)] = OBJ_NAME_PATH_SEPARATOR;
//             KeyNameInfo->NameLength += NameLength + sizeof(OBJ_NAME_PATH_SEPARATOR);
//
//             CurrentKcb = CurrentKcb->ParentKcb;
//         }
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         _SEH2_YIELD(return _SEH2_GetExceptionCode());
//     }
//     _SEH2_END;
//
//     /* Make sure we copied everything */
//     ASSERT(NeededLength == 0);
//     ASSERT(KeyNameInfo->Name[0] == OBJ_NAME_PATH_SEPARATOR);
//
//     /* We're done */
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// NTAPI
// CmQueryKey(_In_ PCM_KEY_CONTROL_BLOCK Kcb,
//            _In_ KEY_INFORMATION_CLASS KeyInformationClass,
//            _Out_opt_ PVOID KeyInformation,
//            _In_ ULONG Length,
//            _Out_ PULONG ResultLength)
// {
//     NTSTATUS Status;
//     PHHIVE Hive;
//     PCM_KEY_NODE Parent;
//     HV_TRACK_CELL_REF CellReferences = {0};
//
//     /* Acquire hive lock */
//     CmpLockRegistry();
//
//     /* Lock KCB shared */
//     CmpAcquireKcbLockShared(Kcb);
//
//     /* Don't touch deleted keys */
//     if (Kcb->Delete)
//     {
//         /* Fail */
//         Status = STATUS_KEY_DELETED;
//         goto Quickie;
//     }
//
//     /* Data can be user-mode, use SEH */
//     _SEH2_TRY
//     {
//         /* Check what class we got */
//         switch (KeyInformationClass)
//         {
//             /* Typical information */
//             case KeyFullInformation:
//             case KeyBasicInformation:
//             case KeyNodeInformation:
//             {
//                 /* Get the hive and parent */
//                 Hive = Kcb->KeyHive;
//                 Parent = (PCM_KEY_NODE)HvGetCell(Hive, Kcb->KeyCell);
//                 ASSERT(Parent);
//
//                 /* Track cell references */
//                 if (!HvTrackCellRef(&CellReferences, Hive, Kcb->KeyCell))
//                 {
//                     /* Not enough memory to track references */
//                     Status = STATUS_INSUFFICIENT_RESOURCES;
//                 }
//                 else
//                 {
//                     /* Call the internal API */
//                     Status = CmpQueryKeyData(Hive,
//                                              Parent,
//                                              KeyInformationClass,
//                                              KeyInformation,
//                                              Length,
//                                              ResultLength);
//                 }
//                 break;
//             }
//
//             case KeyCachedInformation:
//             {
//                 /* Call the internal API */
//                 Status = CmpQueryKeyDataFromCache(Kcb,
//                                                   KeyInformation,
//                                                   Length,
//                                                   ResultLength);
//                 break;
//             }
//
//             case KeyFlagsInformation:
//             {
//                 /* Call the internal API */
//                 Status = CmpQueryFlagsInformation(Kcb,
//                                                   KeyInformation,
//                                                   Length,
//                                                   ResultLength);
//                 break;
//             }
//
//             case KeyNameInformation:
//             {
//                 /* Call the internal API */
//                 Status = CmpQueryNameInformation(Kcb,
//                                                  KeyInformation,
//                                                  Length,
//                                                  ResultLength);
//                 break;
//             }
//
//             /* Illegal classes */
//             default:
//             {
//                 /* Print message and fail */
//                 DPRINT1("Unsupported class: %d!\n", KeyInformationClass);
//                 Status = STATUS_INVALID_INFO_CLASS;
//                 break;
//             }
//         }
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         /* Fail with exception code */
//         Status = _SEH2_GetExceptionCode();
//         _SEH2_YIELD(goto Quickie);
//     }
//     _SEH2_END;
//
// Quickie:
//     /* Release references */
//     HvReleaseFreeCellRefArray(&CellReferences);
//
//     /* Release locks */
//     CmpReleaseKcbLock(Kcb);
//     CmpUnlockRegistry();
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// CmEnumerateKey(IN PCM_KEY_CONTROL_BLOCK Kcb,
//                IN ULONG Index,
//                IN KEY_INFORMATION_CLASS KeyInformationClass,
//                IN PVOID KeyInformation,
//                IN ULONG Length,
//                IN PULONG ResultLength)
// {
//     NTSTATUS Status;
//     PHHIVE Hive;
//     PCM_KEY_NODE Parent, Child;
//     HCELL_INDEX ChildCell;
//     HV_TRACK_CELL_REF CellReferences = {0};
//
//     /* Acquire hive lock */
//     CmpLockRegistry();
//
//     /* Lock the KCB shared */
//     CmpAcquireKcbLockShared(Kcb);
//
//     /* Don't touch deleted keys */
//     if (Kcb->Delete)
//     {
//         /* Undo everything */
//         Status = STATUS_KEY_DELETED;
//         goto Quickie;
//     }
//
//     /* Get the hive and parent */
//     Hive = Kcb->KeyHive;
//     Parent = (PCM_KEY_NODE)HvGetCell(Hive, Kcb->KeyCell);
//     ASSERT(Parent);
//
//     /* Get the child cell */
//     ChildCell = CmpFindSubKeyByNumber(Hive, Parent, Index);
//
//     /* Release the parent cell */
//     HvReleaseCell(Hive, Kcb->KeyCell);
//
//     /* Check if we found the child */
//     if (ChildCell == HCELL_NIL)
//     {
//         /* We didn't, fail */
//         Status = STATUS_NO_MORE_ENTRIES;
//         goto Quickie;
//     }
//
//     /* Now get the actual child node */
//     Child = (PCM_KEY_NODE)HvGetCell(Hive, ChildCell);
//     ASSERT(Child);
//
//     /* Track references */
//     if (!HvTrackCellRef(&CellReferences, Hive, ChildCell))
//     {
//         /* Can't allocate memory for tracking */
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto Quickie;
//     }
//
//     /* Data can be user-mode, use SEH */
//     _SEH2_TRY
//     {
//         /* Query the data requested */
//         Status = CmpQueryKeyData(Hive,
//                                  Child,
//                                  KeyInformationClass,
//                                  KeyInformation,
//                                  Length,
//                                  ResultLength);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         /* Fail with exception code */
//         Status = _SEH2_GetExceptionCode();
//         _SEH2_YIELD(goto Quickie);
//     }
//     _SEH2_END;
//
// Quickie:
//     /* Release references */
//     HvReleaseFreeCellRefArray(&CellReferences);
//
//     /* Release locks */
//     CmpReleaseKcbLock(Kcb);
//     CmpUnlockRegistry();
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// CmDeleteKey(IN PCM_KEY_BODY KeyBody)
// {
//     NTSTATUS Status;
//     PHHIVE Hive;
//     PCM_KEY_NODE Node, Parent;
//     HCELL_INDEX Cell, ParentCell;
//     PCM_KEY_CONTROL_BLOCK Kcb;
//
//     /* Acquire hive lock */
//     CmpLockRegistry();
//
//     /* Get the kcb */
//     Kcb = KeyBody->KeyControlBlock;
//
//     /* Don't allow deleting the root */
//     if (!Kcb->ParentKcb)
//     {
//         /* Fail */
//         CmpUnlockRegistry();
//         return STATUS_CANNOT_DELETE;
//     }
//
//     /* Lock parent and child */
//     CmpAcquireTwoKcbLocksExclusiveByKey(Kcb->ConvKey, Kcb->ParentKcb->ConvKey);
//
//     /* Check if we're already being deleted */
//     if (Kcb->Delete)
//     {
//         /* Don't do it twice */
//         Status = STATUS_SUCCESS;
//         goto Quickie;
//     }
//
//     /* Get the hive and node */
//     Hive = Kcb->KeyHive;
//     Cell = Kcb->KeyCell;
//
//     /* Lock flushes */
//     CmpLockHiveFlusherShared((PCMHIVE)Hive);
//
//     /* Get the key node */
//     Node = (PCM_KEY_NODE)HvGetCell(Hive, Cell);
//     ASSERT(Node);
//
//     /* Sanity check */
//     ASSERT(Node->Flags == Kcb->Flags);
//
//     /* Check if we don't have any children */
//     if (!(Node->SubKeyCounts[Stable] + Node->SubKeyCounts[Volatile]) &&
//         !(Node->Flags & KEY_NO_DELETE))
//     {
//         /* Send notification to registered callbacks */
//         CmpReportNotify(Kcb, Hive, Cell, REG_NOTIFY_CHANGE_NAME);
//
//         /* Get the parent and free the cell */
//         ParentCell = Node->Parent;
//         Status = CmpFreeKeyByCell(Hive, Cell, TRUE);
//         if (NT_SUCCESS(Status))
//         {
//             /* Flush any notifications */
//             CmpFlushNotifiesOnKeyBodyList(Kcb, FALSE);
//
//             /* Clean up information we have on the subkey */
//             CmpCleanUpSubKeyInfo(Kcb->ParentKcb);
//
//             /* Get the parent node */
//             Parent = (PCM_KEY_NODE)HvGetCell(Hive, ParentCell);
//             if (Parent)
//             {
//                 /* Update the maximum name length */
//                 Kcb->ParentKcb->KcbMaxNameLen = (USHORT)Parent->MaxNameLen;
//
//                 /* Make sure we're dirty */
//                 ASSERT(HvIsCellDirty(Hive, ParentCell));
//
//                 /* Update the write time */
//                 KeQuerySystemTime(&Parent->LastWriteTime);
//                 Kcb->ParentKcb->KcbLastWriteTime = Parent->LastWriteTime;
//
//                 /* Release the cell */
//                 HvReleaseCell(Hive, ParentCell);
//             }
//
//             /* Set the KCB in delete mode and remove it */
//             Kcb->Delete = TRUE;
//             CmpRemoveKeyControlBlock(Kcb);
//
//             /* Clear the cell */
//             Kcb->KeyCell = HCELL_NIL;
//         }
//     }
//     else
//     {
//         /* Fail */
//         Status = STATUS_CANNOT_DELETE;
//     }
//
//     /* Release the cell */
//     HvReleaseCell(Hive, Cell);
//
//     /* Release flush lock */
//     CmpUnlockHiveFlusher((PCMHIVE)Hive);
//
//     /* Release the KCB locks */
// Quickie:
//     CmpReleaseTwoKcbLockByKey(Kcb->ConvKey, Kcb->ParentKcb->ConvKey);
//
//     /* Release hive lock */
//     CmpUnlockRegistry();
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// CmFlushKey(IN PCM_KEY_CONTROL_BLOCK Kcb,
//            IN BOOLEAN ExclusiveLock)
// {
//     PCMHIVE CmHive;
// #if DBG
//     CM_CHECK_REGISTRY_STATUS CheckStatus;
// #endif
//     NTSTATUS Status = STATUS_SUCCESS;
//     PHHIVE Hive;
//
//     /* Ignore flushes until we're ready */
//     if (CmpNoWrite) return STATUS_SUCCESS;
//
//     /* Get the hives */
//     Hive = Kcb->KeyHive;
//     CmHive = (PCMHIVE)Hive;
//
//     /* Check if this is the master hive */
//     if (CmHive == CmiVolatileHive)
//     {
//         /* Flush all the hives instead */
//         CmpDoFlushAll(FALSE);
//     }
//     else
//     {
// #if DBG
//         /* Make sure the registry hive we're going to flush is OK */
//         CheckStatus = CmCheckRegistry(CmHive, CM_CHECK_REGISTRY_DONT_PURGE_VOLATILES | CM_CHECK_REGISTRY_VALIDATE_HIVE);
//         ASSERT(CM_CHECK_REGISTRY_SUCCESS(CheckStatus));
// #endif
//
//         /* Don't touch the hive */
//         CmpLockHiveFlusherExclusive(CmHive);
//
//         ASSERT(CmHive->ViewLock);
//         KeAcquireGuardedMutex(CmHive->ViewLock);
//         CmHive->ViewLockOwner = KeGetCurrentThread();
//
//         /* Will the hive shrink? */
//         if (HvHiveWillShrink(Hive))
//         {
//             /* I don't believe the current Hv does shrinking */
//             ASSERT(FALSE);
//             // CMP_ASSERT_EXCLUSIVE_REGISTRY_LOCK_OR_LOADING(CmHive);
//         }
//         else
//         {
//             /* Now we can release views */
//             ASSERT(CmHive->ViewLock);
//             // CMP_ASSERT_VIEW_LOCK_OWNED(CmHive);
//             ASSERT((CmpSpecialBootCondition == TRUE) ||
//                    (CmHive->HiveIsLoading == TRUE) ||
//                    (CmHive->ViewLockOwner == KeGetCurrentThread()) ||
//                    (CmpTestRegistryLockExclusive() == TRUE));
//             CmHive->ViewLockOwner = NULL;
//             KeReleaseGuardedMutex(CmHive->ViewLock);
//         }
//
//         /* Flush only this hive */
//         if (!HvSyncHive(Hive))
//         {
//             /* Fail */
//             Status = STATUS_REGISTRY_IO_FAILED;
//         }
//
//         /* Release the flush lock */
//         CmpUnlockHiveFlusher(CmHive);
//     }
//
//     /* Return the status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// CmLoadKey(IN POBJECT_ATTRIBUTES TargetKey,
//           IN POBJECT_ATTRIBUTES SourceFile,
//           IN ULONG Flags,
//           IN PCM_KEY_BODY KeyBody)
// {
//     SECURITY_QUALITY_OF_SERVICE ServiceQos;
//     SECURITY_CLIENT_CONTEXT ClientSecurityContext;
//     HANDLE KeyHandle;
//     BOOLEAN Allocate = TRUE;
//     PCMHIVE CmHive, LoadedHive;
//     NTSTATUS Status;
//     CM_PARSE_CONTEXT ParseContext;
//
//     /* Check if we have a trust key */
//     if (KeyBody)
//     {
//         /* Fail */
//         DPRINT("Trusted classes not yet supported\n");
//     }
//
//     /* Build a service QoS for a security context */
//     ServiceQos.Length = sizeof(SECURITY_QUALITY_OF_SERVICE);
//     ServiceQos.ImpersonationLevel = SecurityImpersonation;
//     ServiceQos.ContextTrackingMode = SECURITY_DYNAMIC_TRACKING;
//     ServiceQos.EffectiveOnly = TRUE;
//     Status = SeCreateClientSecurity(PsGetCurrentThread(),
//                                     &ServiceQos,
//                                     FALSE,
//                                     &ClientSecurityContext);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         DPRINT1("Security context failed\n");
//         return Status;
//     }
//
//     /* Open the target key */
//     RtlZeroMemory(&ParseContext, sizeof(ParseContext));
//     ParseContext.CreateOperation = FALSE;
//     Status = ObOpenObjectByName(TargetKey,
//                                 CmpKeyObjectType,
//                                 KernelMode,
//                                 NULL,
//                                 KEY_READ,
//                                 &ParseContext,
//                                 &KeyHandle);
//     if (!NT_SUCCESS(Status)) KeyHandle = NULL;
//
//     /* Open the hive */
//     Status = CmpCmdHiveOpen(SourceFile,
//                             &ClientSecurityContext,
//                             &Allocate,
//                             &CmHive,
//                             CM_CHECK_REGISTRY_PURGE_VOLATILES);
//
//     /* Get rid of the security context */
//     SeDeleteClientSecurity(&ClientSecurityContext);
//
//     /* See if we failed */
//     if (!NT_SUCCESS(Status))
//     {
//         /* See if the target already existed */
//         if (KeyHandle)
//         {
//             /* Lock the registry */
//             CmpLockRegistryExclusive();
//
//             /* Check if we are already loaded */
//             if (CmpIsHiveAlreadyLoaded(KeyHandle, SourceFile, &LoadedHive))
//             {
//                 /* That's okay then */
//                 ASSERT(LoadedHive);
//                 Status = STATUS_SUCCESS;
//             }
//
//             /* Release the registry */
//             CmpUnlockRegistry();
//         }
//
//         /* Close the key handle if we had one */
//         if (KeyHandle) ZwClose(KeyHandle);
//         return Status;
//     }
//
//     /* Lock the registry shared */
//     CmpLockRegistry();
//
//     /* Lock loading */
//     ExAcquirePushLockExclusive(&CmpLoadHiveLock);
//
//     /* Lock the hive to this thread */
//     CmHive->Hive.HiveFlags |= HIVE_IS_UNLOADING;
//     CmHive->CreatorOwner = KeGetCurrentThread();
//
//     /* Set flag */
//     if (Flags & REG_NO_LAZY_FLUSH) CmHive->Hive.HiveFlags |= HIVE_NOLAZYFLUSH;
//
//     /* Link the hive */
//     Status = CmpLinkHiveToMaster(TargetKey->ObjectName,
//                                  TargetKey->RootDirectory,
//                                  CmHive,
//                                  Allocate,
//                                  TargetKey->SecurityDescriptor);
//     if (NT_SUCCESS(Status))
//     {
//         /* Add to HiveList key */
//         CmpAddToHiveFileList(CmHive);
//
//         /* Sync the hive if necessary */
//         if (Allocate)
//         {
//             /* Sync it under the flusher lock */
//             CmpLockHiveFlusherExclusive(CmHive);
//             HvSyncHive(&CmHive->Hive);
//             CmpUnlockHiveFlusher(CmHive);
//         }
//
//         /* Release the hive */
//         CmHive->Hive.HiveFlags &= ~HIVE_IS_UNLOADING;
//         CmHive->CreatorOwner = NULL;
//     }
//     else
//     {
//         DPRINT1("CmpLinkHiveToMaster failed, Status %lx\n", Status);
//
//         /* We're touching this hive, set the loading flag */
//         CmHive->HiveIsLoading = TRUE;
//
//         /* Close associated file handles */
//         CmpCloseHiveFiles(CmHive);
//
//         /* Cleanup its resources */
//         CmpDestroyHive(CmHive);
//     }
//
//     /* Allow loads */
//     ExReleasePushLock(&CmpLoadHiveLock);
//
//     /* Is this first profile load? */
//     if (!CmpProfileLoaded && !CmpWasSetupBoot)
//     {
//         /* User is now logged on, set quotas */
//         CmpProfileLoaded = TRUE;
//         CmpSetGlobalQuotaAllowed();
//     }
//
//     /* Unlock the registry */
//     CmpUnlockRegistry();
//
//     /* Close handle and return */
//     if (KeyHandle) ZwClose(KeyHandle);
//     return Status;
// }
//
// static
// BOOLEAN
// NTAPI
// CmpUnlinkHiveFromMaster(IN PCMHIVE CmHive,
//                         IN HCELL_INDEX Cell)
// {
//     PCELL_DATA CellData;
//     HCELL_INDEX LinkCell;
//     NTSTATUS Status;
//
//     DPRINT("CmpUnlinkHiveFromMaster()\n");
//
//     /* Get the cell data */
//     CellData = HvGetCell(&CmHive->Hive, Cell);
//     if (CellData == NULL)
//         return FALSE;
//
//     /* Get the link cell and release the current cell */
//     LinkCell = CellData->u.KeyNode.Parent;
//     HvReleaseCell(&CmHive->Hive, Cell);
//
//     /* Remove the link cell from the master hive */
//     CmpLockHiveFlusherExclusive(CmiVolatileHive);
//     Status = CmpFreeKeyByCell((PHHIVE)CmiVolatileHive,
//                               LinkCell,
//                               TRUE);
//     CmpUnlockHiveFlusher(CmiVolatileHive);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("CmpFreeKeyByCell() failed (Status 0x%08lx)\n", Status);
//         return FALSE;
//     }
//
//     /* Remove the hive from the list */
//     ExAcquirePushLockExclusive(&CmpHiveListHeadLock);
//     RemoveEntryList(&CmHive->HiveList);
//     ExReleasePushLock(&CmpHiveListHeadLock);
//
//     return TRUE;
// }
//
// NTSTATUS
// NTAPI
// CmUnloadKey(
//     _In_ PCM_KEY_CONTROL_BLOCK Kcb,
//     _In_ ULONG Flags)
// {
//     PHHIVE Hive;
//     PCMHIVE CmHive;
//     HCELL_INDEX Cell;
//
//     DPRINT("CmUnloadKey(%p, %lx)\n", Kcb, Flags);
//
//     /* Ensure the registry is locked exclusively for the calling thread */
//     CMP_ASSERT_EXCLUSIVE_REGISTRY_LOCK();
//
//     /* Get the hive */
//     Hive = Kcb->KeyHive;
//     Cell = Kcb->KeyCell;
//     CmHive = (PCMHIVE)Hive;
//
//     /* Fail if the key is not a hive root key */
//     if (Cell != Hive->BaseBlock->RootCell)
//     {
//         DPRINT1("Key is not a hive root key!\n");
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Fail if we try to unload the master hive */
//     if (CmHive == CmiVolatileHive)
//     {
//         DPRINT1("Do not try to unload the master hive!\n");
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Mark this hive as being unloaded */
//     Hive->HiveFlags |= HIVE_IS_UNLOADING;
//
//     /* Search for any opened keys in this hive, and take an appropriate action */
//     if (Kcb->RefCount > 1)
//     {
//         if (Flags != REG_FORCE_UNLOAD)
//         {
//             if (CmpEnumerateOpenSubKeys(Kcb, TRUE, FALSE) != 0)
//             {
//                 /* There are open subkeys but we don't force hive unloading, fail */
//                 Hive->HiveFlags &= ~HIVE_IS_UNLOADING;
//                 return STATUS_CANNOT_DELETE;
//             }
//         }
//         else
//         {
//             if (CmpEnumerateOpenSubKeys(Kcb, TRUE, TRUE) != 0)
//             {
//                 /* There are open subkeys that we cannot force to unload, fail */
//                 Hive->HiveFlags &= ~HIVE_IS_UNLOADING;
//                 return STATUS_CANNOT_DELETE;
//             }
//         }
//     }
//
//     /* Set the loading flag */
//     CmHive->HiveIsLoading = TRUE;
//
//     /* Flush the hive */
//     CmFlushKey(Kcb, TRUE);
//
//     /* Unlink the hive from the master hive */
//     if (!CmpUnlinkHiveFromMaster(CmHive, Cell))
//     {
//         DPRINT("CmpUnlinkHiveFromMaster() failed!\n");
//
//         /* Remove the unloading flag */
//         Hive->HiveFlags &= ~HIVE_IS_UNLOADING;
//
//         /* Reset the loading flag */
//         CmHive->HiveIsLoading = FALSE;
//
//         /* Return failure */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Flush any notifications if we force hive unloading */
//     if (Flags == REG_FORCE_UNLOAD)
//         CmpFlushNotifiesOnKeyBodyList(Kcb, TRUE); // Lock is already held
//
//     /* Clean up information we have on the subkey */
//     CmpCleanUpSubKeyInfo(Kcb->ParentKcb);
//
//     /* Set the KCB in delete mode and remove it */
//     Kcb->Delete = TRUE;
//     CmpRemoveKeyControlBlock(Kcb);
//
//     /* Release the hive loading lock */
//     ExReleasePushLockExclusive(&CmpLoadHiveLock);
//
//     /* Release hive lock */
//     CmpUnlockRegistry();
//
//     /* Close file handles */
//     CmpCloseHiveFiles(CmHive);
//
//     /* Remove the hive from the hive file list */
//     CmpRemoveFromHiveFileList(CmHive);
//
// /**
//  ** NOTE:
//  ** The following code is mostly equivalent to what we "call" CmpDestroyHive()
//  **/
//     /* Destroy the security descriptor cache */
//     CmpDestroySecurityCache(CmHive);
//
//     /* Destroy the view list */
//     CmpDestroyHiveViewList(CmHive);
//
//     /* Delete the flusher lock */
//     ExDeleteResourceLite(CmHive->FlusherLock);
//     ExFreePoolWithTag(CmHive->FlusherLock, TAG_CMHIVE);
//
//     /* Delete the view lock */
//     ExFreePoolWithTag(CmHive->ViewLock, TAG_CMHIVE);
//
//     /* Free the hive storage */
//     HvFree(Hive);
//
//     /* Free the hive */
//     CmpFree(CmHive, TAG_CM);
//
//     return STATUS_SUCCESS;
// }
//
// ULONG
// NTAPI
// CmpEnumerateOpenSubKeys(
//     _In_ PCM_KEY_CONTROL_BLOCK RootKcb,
//     _In_ BOOLEAN RemoveEmptyCacheEntries,
//     _In_ BOOLEAN DereferenceOpenedEntries)
// {
//     PCM_KEY_HASH Entry;
//     PCM_KEY_CONTROL_BLOCK CachedKcb;
//     PCM_KEY_CONTROL_BLOCK ParentKcb;
//     ULONG ParentKeyCount;
//     ULONG i, j;
//     ULONG SubKeys = 0;
//
//     DPRINT("CmpEnumerateOpenSubKeys() called\n");
//
//     /* Ensure the registry is locked exclusively for the calling thread */
//     CMP_ASSERT_EXCLUSIVE_REGISTRY_LOCK();
//
//     /* The root key is the only referenced key. There are no referenced sub keys. */
//     if (RootKcb->RefCount == 1)
//     {
//         DPRINT("Open sub keys: 0\n");
//         return 0;
//     }
//
//     /* Enumerate all hash lists */
//     for (i = 0; i < CmpHashTableSize; i++)
//     {
//         /* Get the first cache entry */
//         Entry = CmpCacheTable[i].Entry;
//
//         /* Enumerate all cache entries */
//         while (Entry)
//         {
//             /* Get the KCB of the current cache entry */
//             CachedKcb = CONTAINING_RECORD(Entry, CM_KEY_CONTROL_BLOCK, KeyHash);
//
//             /* Check keys only that are subkeys to our root key */
//             if (CachedKcb->TotalLevels > RootKcb->TotalLevels)
//             {
//                 /* Calculate the number of parent keys to the root key */
//                 ParentKeyCount = CachedKcb->TotalLevels - RootKcb->TotalLevels;
//
//                 /* Find a parent key that could be the root key */
//                 ParentKcb = CachedKcb;
//                 for (j = 0; j < ParentKeyCount; j++)
//                 {
//                     ParentKcb = ParentKcb->ParentKcb;
//                 }
//
//                 /* Check whether the parent is the root key */
//                 if (ParentKcb == RootKcb)
//                 {
//                     DPRINT("Found a sub key, RefCount = %u\n", CachedKcb->RefCount);
//
//                     if (CachedKcb->RefCount > 0)
//                     {
//                         DPRINT("Found a sub key pointing to '%.*s', RefCount = %u\n",
//                                CachedKcb->NameBlock->NameLength, CachedKcb->NameBlock->Name,
//                                CachedKcb->RefCount);
//
//                         /* If we dereference opened KCBs, don't touch read-only keys */
//                         if (DereferenceOpenedEntries &&
//                             !(CachedKcb->ExtFlags & CM_KCB_READ_ONLY_KEY))
//                         {
//                             /* Flush any notifications */
//                             CmpFlushNotifiesOnKeyBodyList(CachedKcb, TRUE); // Lock is already held
//
//                             /* Clean up information we have on the subkey */
//                             CmpCleanUpSubKeyInfo(CachedKcb->ParentKcb);
//
//                             /* Get and cache the next cache entry */
//                             // Entry = Entry->NextHash;
//                             Entry = CachedKcb->NextHash;
//
//                             /* Set the KCB in delete mode and remove it */
//                             CachedKcb->Delete = TRUE;
//                             CmpRemoveKeyControlBlock(CachedKcb);
//
//                             /* Clear the cell */
//                             CachedKcb->KeyCell = HCELL_NIL;
//
//                             /* Restart with the next cache entry */
//                             continue;
//                         }
//                         /* Else, the key cannot be dereferenced, and we count it as in use */
//
//                         /* Count the current hash entry if it is in use */
//                         SubKeys++;
//                     }
//                     else if ((CachedKcb->RefCount == 0) && RemoveEmptyCacheEntries)
//                     {
//                         /* Remove the current key from the delayed close list */
//                         CmpRemoveFromDelayedClose(CachedKcb);
//
//                         /* Remove the current cache entry */
//                         CmpCleanUpKcbCacheWithLock(CachedKcb, TRUE);
//
//                         /* Restart, because the hash list has changed */
//                         Entry = CmpCacheTable[i].Entry;
//                         continue;
//                     }
//                 }
//             }
//
//             /* Get the next cache entry */
//             Entry = Entry->NextHash;
//         }
//     }
//
//     if (SubKeys > 0)
//         DPRINT1("Open sub keys: %u\n", SubKeys);
//
//     return SubKeys;
// }
//
// static
// NTSTATUS
// CmpDeepCopyKeyInternal(IN PHHIVE SourceHive,
//                        IN HCELL_INDEX SrcKeyCell,
//                        IN PHHIVE DestinationHive,
//                        IN HCELL_INDEX Parent,
//                        IN HSTORAGE_TYPE StorageType,
//                        OUT PHCELL_INDEX DestKeyCell OPTIONAL)
// {
//     NTSTATUS Status;
//     PCM_KEY_NODE SrcNode;
//     PCM_KEY_NODE DestNode = NULL;
//     HCELL_INDEX NewKeyCell = HCELL_NIL;
//     HCELL_INDEX NewClassCell = HCELL_NIL, NewSecCell = HCELL_NIL;
//     HCELL_INDEX SubKey, NewSubKey;
//     ULONG Index, SubKeyCount;
//
//     PAGED_CODE();
//
//     DPRINT("CmpDeepCopyKeyInternal(0x%08X, 0x%08X, 0x%08X, 0x%08X, 0x%08X, 0x%08X)\n",
//            SourceHive,
//            SrcKeyCell,
//            DestinationHive,
//            Parent,
//            StorageType,
//            DestKeyCell);
//
//     /* Get the source cell node */
//     SrcNode = (PCM_KEY_NODE)HvGetCell(SourceHive, SrcKeyCell);
//     ASSERT(SrcNode);
//
//     /* Sanity check */
//     ASSERT(SrcNode->Signature == CM_KEY_NODE_SIGNATURE);
//
//     /* Create a simple copy of the source key */
//     NewKeyCell = CmpCopyCell(SourceHive,
//                              SrcKeyCell,
//                              DestinationHive,
//                              StorageType);
//     if (NewKeyCell == HCELL_NIL)
//     {
//         /* Not enough storage space */
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto Cleanup;
//     }
//
//     /* Get the destination cell node */
//     DestNode = (PCM_KEY_NODE)HvGetCell(DestinationHive, NewKeyCell);
//     ASSERT(DestNode);
//
//     /* Set the parent and copy the flags */
//     DestNode->Parent = Parent;
//     DestNode->Flags  = (SrcNode->Flags & KEY_COMP_NAME); // Keep only the single permanent flag
//     if (Parent == HCELL_NIL)
//     {
//         /* This is the new root node */
//         DestNode->Flags |= KEY_HIVE_ENTRY | KEY_NO_DELETE;
//     }
//
//     /* Copy the class cell */
//     if (SrcNode->ClassLength > 0)
//     {
//         NewClassCell = CmpCopyCell(SourceHive,
//                                    SrcNode->Class,
//                                    DestinationHive,
//                                    StorageType);
//         if (NewClassCell == HCELL_NIL)
//         {
//             /* Not enough storage space */
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto Cleanup;
//         }
//
//         DestNode->Class = NewClassCell;
//         DestNode->ClassLength = SrcNode->ClassLength;
//     }
//     else
//     {
//         DestNode->Class = HCELL_NIL;
//         DestNode->ClassLength = 0;
//     }
//
//     /* Copy the security cell (FIXME: HACKish poor-man version) */
//     if (SrcNode->Security != HCELL_NIL)
//     {
//         NewSecCell = CmpCopyCell(SourceHive,
//                                  SrcNode->Security,
//                                  DestinationHive,
//                                  StorageType);
//         if (NewSecCell == HCELL_NIL)
//         {
//             /* Not enough storage space */
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto Cleanup;
//         }
//     }
//     DestNode->Security = NewSecCell;
//
//     /* Copy the value list */
//     Status = CmpCopyKeyValueList(SourceHive,
//                                  &SrcNode->ValueList,
//                                  DestinationHive,
//                                  &DestNode->ValueList,
//                                  StorageType);
//     if (!NT_SUCCESS(Status))
//         goto Cleanup;
//
//     /* Clear the invalid subkey index */
//     DestNode->SubKeyCounts[Stable] = DestNode->SubKeyCounts[Volatile] = 0;
//     DestNode->SubKeyLists[Stable] = DestNode->SubKeyLists[Volatile] = HCELL_NIL;
//
//     /* Calculate the total number of subkeys */
//     SubKeyCount = SrcNode->SubKeyCounts[Stable] + SrcNode->SubKeyCounts[Volatile];
//
//     /* Loop through all the subkeys */
//     for (Index = 0; Index < SubKeyCount; Index++)
//     {
//         /* Get the subkey */
//         SubKey = CmpFindSubKeyByNumber(SourceHive, SrcNode, Index);
//         ASSERT(SubKey != HCELL_NIL);
//
//         /* Call the function recursively for the subkey */
//         //
//         // FIXME: Danger!! Kernel stack exhaustion!!
//         //
//         Status = CmpDeepCopyKeyInternal(SourceHive,
//                                         SubKey,
//                                         DestinationHive,
//                                         NewKeyCell,
//                                         StorageType,
//                                         &NewSubKey);
//         if (!NT_SUCCESS(Status))
//             goto Cleanup;
//
//         /* Add the copy of the subkey to the new key */
//         if (!CmpAddSubKey(DestinationHive,
//                           NewKeyCell,
//                           NewSubKey))
//         {
//             /* Cleanup allocated cell */
//             HvFreeCell(DestinationHive, NewSubKey);
//
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto Cleanup;
//         }
//     }
//
//     /* Set success */
//     Status = STATUS_SUCCESS;
//
// Cleanup:
//
//     /* Release the cells */
//     if (DestNode) HvReleaseCell(DestinationHive, NewKeyCell);
//     if (SrcNode) HvReleaseCell(SourceHive, SrcKeyCell);
//
//     /* Cleanup allocated cells in case of failure */
//     if (!NT_SUCCESS(Status))
//     {
//         if (NewSecCell != HCELL_NIL)
//             HvFreeCell(DestinationHive, NewSecCell);
//
//         if (NewClassCell != HCELL_NIL)
//             HvFreeCell(DestinationHive, NewClassCell);
//
//         if (NewKeyCell != HCELL_NIL)
//             HvFreeCell(DestinationHive, NewKeyCell);
//
//         NewKeyCell = HCELL_NIL;
//     }
//
//     /* Set the cell index if requested and return status */
//     if (DestKeyCell) *DestKeyCell = NewKeyCell;
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// CmpDeepCopyKey(IN PHHIVE SourceHive,
//                IN HCELL_INDEX SrcKeyCell,
//                IN PHHIVE DestinationHive,
//                IN HSTORAGE_TYPE StorageType,
//                OUT PHCELL_INDEX DestKeyCell OPTIONAL)
// {
//     /* Call the internal function */
//     return CmpDeepCopyKeyInternal(SourceHive,
//                                   SrcKeyCell,
//                                   DestinationHive,
//                                   HCELL_NIL,
//                                   StorageType,
//                                   DestKeyCell);
// }
//
// NTSTATUS
// NTAPI
// CmSaveKey(IN PCM_KEY_CONTROL_BLOCK Kcb,
//           IN HANDLE FileHandle,
//           IN ULONG Flags)
// {
// #if DBG
//     CM_CHECK_REGISTRY_STATUS CheckStatus;
//     PCMHIVE HiveToValidate = NULL;
// #endif
//     NTSTATUS Status = STATUS_SUCCESS;
//     PCMHIVE KeyHive = NULL;
//     PAGED_CODE();
//
//     DPRINT("CmSaveKey(0x%08X, 0x%08X, %lu)\n", Kcb, FileHandle, Flags);
//
//     /* Lock the registry and KCB */
//     CmpLockRegistry();
//     CmpAcquireKcbLockShared(Kcb);
//
// #if DBG
//     /* Get the hive for validation */
//     HiveToValidate = (PCMHIVE)Kcb->KeyHive;
// #endif
//
//     if (Kcb->Delete)
//     {
//         /* The source key has been deleted, do nothing */
//         Status = STATUS_KEY_DELETED;
//         goto Cleanup;
//     }
//
//     if (Kcb->KeyHive == &CmiVolatileHive->Hive)
//     {
//         /* Keys that are directly in the master hive can't be saved */
//         Status = STATUS_ACCESS_DENIED;
//         goto Cleanup;
//     }
//
// #if DBG
//     /* Make sure this control block has a sane hive */
//     CheckStatus = CmCheckRegistry(HiveToValidate, CM_CHECK_REGISTRY_DONT_PURGE_VOLATILES | CM_CHECK_REGISTRY_VALIDATE_HIVE);
//     ASSERT(CM_CHECK_REGISTRY_SUCCESS(CheckStatus));
// #endif
//
//     /* Create a new hive that will hold the key */
//     Status = CmpInitializeHive(&KeyHive,
//                                HINIT_CREATE,
//                                HIVE_VOLATILE,
//                                HFILE_TYPE_PRIMARY,
//                                NULL,
//                                NULL,
//                                NULL,
//                                NULL,
//                                NULL,
//                                NULL,
//                                CM_CHECK_REGISTRY_DONT_PURGE_VOLATILES);
//     if (!NT_SUCCESS(Status)) goto Cleanup;
//
//     /* Copy the key recursively into the new hive */
//     Status = CmpDeepCopyKey(Kcb->KeyHive,
//                             Kcb->KeyCell,
//                             &KeyHive->Hive,
//                             Stable,
//                             &KeyHive->Hive.BaseBlock->RootCell);
//     if (!NT_SUCCESS(Status)) goto Cleanup;
//
//     /* Set the primary handle of the hive */
//     KeyHive->FileHandles[HFILE_TYPE_PRIMARY] = FileHandle;
//
//     /* Dump the hive into the file */
//     HvWriteHive(&KeyHive->Hive);
//
// Cleanup:
//
//     /* Free the hive */
//     if (KeyHive) CmpDestroyHive(KeyHive);
//
// #if DBG
//     if (NT_SUCCESS(Status))
//     {
//         /* Before we say goodbye, make sure the hive is still OK */
//         CheckStatus = CmCheckRegistry(HiveToValidate, CM_CHECK_REGISTRY_DONT_PURGE_VOLATILES | CM_CHECK_REGISTRY_VALIDATE_HIVE);
//         ASSERT(CM_CHECK_REGISTRY_SUCCESS(CheckStatus));
//     }
// #endif
//
//     /* Release the locks */
//     CmpReleaseKcbLock(Kcb);
//     CmpUnlockRegistry();
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// CmSaveMergedKeys(IN PCM_KEY_CONTROL_BLOCK HighKcb,
//                  IN PCM_KEY_CONTROL_BLOCK LowKcb,
//                  IN HANDLE FileHandle)
// {
// #if DBG
//     CM_CHECK_REGISTRY_STATUS CheckStatus;
//     PCMHIVE LowHiveToValidate = NULL;
//     PCMHIVE HighHiveToValidate = NULL;
// #endif
//     PCMHIVE KeyHive = NULL;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     PAGED_CODE();
//
//     DPRINT("CmSaveKey(%p, %p, %p)\n", HighKcb, LowKcb, FileHandle);
//
//     /* Lock the registry and the KCBs */
//     CmpLockRegistry();
//     CmpAcquireKcbLockShared(HighKcb);
//     CmpAcquireKcbLockShared(LowKcb);
//
// #if DBG
//     /* Get the high and low hives for validation */
//     HighHiveToValidate = (PCMHIVE)HighKcb->KeyHive;
//     LowHiveToValidate = (PCMHIVE)LowKcb->KeyHive;
// #endif
//
//     if (LowKcb->Delete || HighKcb->Delete)
//     {
//         /* The source key has been deleted, do nothing */
//         Status = STATUS_KEY_DELETED;
//         goto done;
//     }
//
// #if DBG
//     /* Make sure that both the high and low precedence hives are OK */
//     CheckStatus = CmCheckRegistry(HighHiveToValidate, CM_CHECK_REGISTRY_DONT_PURGE_VOLATILES | CM_CHECK_REGISTRY_VALIDATE_HIVE);
//     ASSERT(CM_CHECK_REGISTRY_SUCCESS(CheckStatus));
//     CheckStatus = CmCheckRegistry(LowHiveToValidate, CM_CHECK_REGISTRY_DONT_PURGE_VOLATILES | CM_CHECK_REGISTRY_VALIDATE_HIVE);
//     ASSERT(CM_CHECK_REGISTRY_SUCCESS(CheckStatus));
// #endif
//
//     /* Create a new hive that will hold the key */
//     Status = CmpInitializeHive(&KeyHive,
//                                HINIT_CREATE,
//                                HIVE_VOLATILE,
//                                HFILE_TYPE_PRIMARY,
//                                NULL,
//                                NULL,
//                                NULL,
//                                NULL,
//                                NULL,
//                                NULL,
//                                CM_CHECK_REGISTRY_DONT_PURGE_VOLATILES);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Copy the low precedence key recursively into the new hive */
//     Status = CmpDeepCopyKey(LowKcb->KeyHive,
//                             LowKcb->KeyCell,
//                             &KeyHive->Hive,
//                             Stable,
//                             &KeyHive->Hive.BaseBlock->RootCell);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Copy the high precedence key recursively into the new hive */
//     Status = CmpDeepCopyKey(HighKcb->KeyHive,
//                             HighKcb->KeyCell,
//                             &KeyHive->Hive,
//                             Stable,
//                             &KeyHive->Hive.BaseBlock->RootCell);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set the primary handle of the hive */
//     KeyHive->FileHandles[HFILE_TYPE_PRIMARY] = FileHandle;
//
//     /* Dump the hive into the file */
//     HvWriteHive(&KeyHive->Hive);
//
// done:
//     /* Free the hive */
//     if (KeyHive)
//         CmpDestroyHive(KeyHive);
//
// #if DBG
//     if (NT_SUCCESS(Status))
//     {
//         /* Check those hives again before we say goodbye */
//         CheckStatus = CmCheckRegistry(HighHiveToValidate, CM_CHECK_REGISTRY_DONT_PURGE_VOLATILES | CM_CHECK_REGISTRY_VALIDATE_HIVE);
//         ASSERT(CM_CHECK_REGISTRY_SUCCESS(CheckStatus));
//         CheckStatus = CmCheckRegistry(LowHiveToValidate, CM_CHECK_REGISTRY_DONT_PURGE_VOLATILES | CM_CHECK_REGISTRY_VALIDATE_HIVE);
//         ASSERT(CM_CHECK_REGISTRY_SUCCESS(CheckStatus));
//     }
// #endif
//
//     /* Release the locks */
//     CmpReleaseKcbLock(LowKcb);
//     CmpReleaseKcbLock(HighKcb);
//     CmpUnlockRegistry();
//
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
