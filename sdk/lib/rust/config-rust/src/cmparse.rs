use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::opaque_ptr;
#[cfg(not(target_arch = "x86"))]
use ntkrnl_sys::inline::ke::KeQuerySystemTime;
#[cfg(target_arch = "x86")]
use ntkrnl_sys::ddk::KeQuerySystemTime;
use ntkrnl_sys::ddk::{
    CM_HASH_CACHE_STACK, CM_KEY_BODY, CM_KEY_CONTROL_BLOCK, CM_KEY_NODE,
    CmpAcquireKcbLockExclusiveByIndex, CmpAddSubKey, CmpAssignSecurityDescriptor,
    CmpBuildAndLockKcbArray, CmpCleanUpKcbCacheWithLock, CmpCleanUpKcbValueCache,
    CmpCleanUpSubKeyInfo, CmpCompareCompressedName, CmpCopyName, CmpCreateKeyControlBlock,
    CmpDereferenceKeyControlBlock, CmpDereferenceKeyControlBlockWithLock,
    CmpFindSubKeyByName, CmpFindValueByName, CmpFreeKeyByCell, CmpGetValueData,
    CmpLockHiveFlusherShared, CmpLockRegistry, CmpNameSize,
    CmpReferenceKeyControlBlock, CmpRemoveFromDelayedClose, CmpRemoveKeyControlBlock,
    CmpReportNotify, CmpUnLockKcbArray, CmpUnlockHiveFlusher, ExFreePool,
    HCELL_INDEX, KPROCESSOR_MODE, LARGE_INTEGER,
    _UNICODE_STRING, _CM_KEY_CONTROL_BLOCK, POBJECT_TYPE,
    ObCheckObjectAccess, ObCreateObject, ObDereferenceObjectDeferDelete,
    PACCESS_STATE, PCELL_DATA, PCMHIVE, PCM_HASH_CACHE_STACK, PCM_KEY_BODY,
    PCM_KEY_CONTROL_BLOCK, PCM_KEY_HASH, PCM_KEY_NODE, PCM_KEY_VALUE,
    PCM_NAME_CONTROL_BLOCK, PCM_PARSE_CONTEXT, PHCELL_INDEX, PHHIVE,
    PSECURITY_DESCRIPTOR, PUNICODE_STRING, PsGetCurrentProcessId,
    RtlAppendUnicodeStringToString, RtlCompareUnicodeString,
    SeAssignSecurity, SeDeassignSecurity,
};

// Constants — local to this file, not in ntkrnl-sys (see ntoskrnl/include/internal/cm.h).
const HCELL_NIL: HCELL_INDEX = 0xFFFFFFFFu32;
const TAG_CM: u32 = u32::from_le_bytes(*b"CM  "); // C: '  MC'  (multi-char literal is big-endian)
const UNICODE_NULL: u16 = 0;
const STATUS_RETRY: i32 = 0xC000022Du32 as i32;
const STATUS_OBJECT_NAME_NOT_FOUND_MOD: i32 = 0xC0000034u32 as i32;
const CMP_SUBKEY_LEVELS_DEPTH_LIMIT: usize = 32;
const CM_KCB_SYM_LINK_FOUND: u32 = 0x00000008;
// Module scope, not function scope: CmpDoOpen tests this too, and a copy
// buried in one function is how the other call site ended up open-coded.
const REG_OPTION_CREATE_LINK: u32 = 0x00000002;

extern "C" {
    pub static CmpKeyObjectType: POBJECT_TYPE;
    pub static mut CmSymbolicLinkValueName: _UNICODE_STRING;
    fn CmpIsKcbLockedExclusive(Kcb: PCM_KEY_CONTROL_BLOCK) -> u8;
    fn GET_HASH_INDEX(ConvKey: u32) -> u32;
    fn CmpAcquireKcbLockSharedByIndex(index: ULONG);
    static mut CmpCacheTable: *mut core::ffi::c_void;
    static mut CmpHashTableSize: u32;
}

extern "system" {
    fn EnlistKeyBodyWithKCB(KeyBody: PCM_KEY_BODY, Flags: u32);
}

// Macros from cm_x.h — implement inline so the LLM-translated call sites work.
// LLM elided the `->Entry` dereference: original C is `GET_HASH_ENTRY(t,k)->Entry`
// which returns the *first* PCM_KEY_HASH in the linked list at hash bucket.
#[inline(always)]
unsafe fn GET_HASH_ENTRY(table: *mut core::ffi::c_void, conv_key: u32) -> PCM_KEY_HASH {
    let entries = table as *mut ntkrnl_sys::ddk::CM_KEY_HASH_TABLE_ENTRY;
    let idx = GET_HASH_INDEX(conv_key) as usize;
    (*entries.add(idx)).Entry
}

#[inline(always)]
unsafe fn COMPUTE_HASH_CHAR(key: ULONG, ch: WCHAR) -> ULONG {
    /* Mirror the C macro at cm_x.h:31-32:
     *   #define COMPUTE_HASH_CHAR(ConvKey, Char)
     *       (37 * ConvKey + RtlUpcaseUnicodeChar(Char))
     * The LLM-generated translation omitted the RtlUpcaseUnicodeChar call,
     * which produced ConvKeys that disagreed with cmkcbncb.rs's correct
     * implementation. The mismatch left lookup-cache hash indexes unlocked
     * when CmpCreateKeyControlBlock was called with CMP_LOCK_HASHES_FOR_KCB,
     * leading to the cm_x.h:120 / ex.h:1305 ASSERT cascade in SMSS init. */
    key.wrapping_mul(37).wrapping_add(ntkrnl_sys::ddk::RtlUpcaseUnicodeChar(ch) as ULONG)
}

#[inline(always)]
unsafe fn NT_ASSERT(_expr: u8) {}
extern "system" {
    fn HvAllocateCell( Hive: PHHIVE, Size: u32, StorageType: u32, Flink: HCELL_INDEX, ) -> HCELL_INDEX;
    fn HvFreeCell(Hive: PHHIVE, Cell: HCELL_INDEX);
    fn HvMarkCellDirty(Hive: PHHIVE, Cell: HCELL_INDEX, Shared: u8) -> u8;
    static mut CmiVolatileHive: PCMHIVE;
    fn CmpUnlockRegistry();
}
extern "C" {
    fn HvGetCell(Hive: PHHIVE, Cell: HCELL_INDEX) -> *mut core::ffi::c_void;
    fn HvReleaseCell(Hive: PHHIVE, Cell: HCELL_INDEX);
    fn CMP_ASSERT_KCB_LOCK(Kcb: PCM_KEY_CONTROL_BLOCK);
    fn HvGetCellType(Cell: HCELL_INDEX) -> u32;
}

extern "C" {
    static mut CmpNoVolatileCreates: u8;
}

#[no_mangle]
pub unsafe extern "system" fn CmpGetNextName(
    RemainingName: PUNICODE_STRING,
    NextName: PUNICODE_STRING,
    LastName: PBOOLEAN,
) -> u8 {
    let RemainingName = opaque_ptr(RemainingName);
    let NextName = opaque_ptr(NextName);
    let LastName = opaque_ptr(LastName);
    let mut NameValid: u8 = 1u8;

    if (*RemainingName).Buffer.is_null()
        || (*RemainingName).Length == 0
        || *(*RemainingName).Buffer == 0
    {
        *LastName = 1u8;
        (*NextName).Buffer = core::ptr::null_mut();
        (*NextName).Length = 0;
        return 1u8;
    }

    while (*RemainingName).Length > 0
        && *(*RemainingName).Buffer == 0x5Cu16
    {
        (*RemainingName).Buffer = (*RemainingName).Buffer.offset(1);
        (*RemainingName).Length -= core::mem::size_of::<u16>() as u16;
        (*RemainingName).MaximumLength -= core::mem::size_of::<u16>() as u16;
    }

    (*NextName).Buffer = (*RemainingName).Buffer;
    while (*RemainingName).Length > 0
        && *(*RemainingName).Buffer != 0x5Cu16
    {
        (*RemainingName).Buffer = (*RemainingName).Buffer.offset(1);
        (*RemainingName).Length -= core::mem::size_of::<u16>() as u16;
        (*RemainingName).MaximumLength -= core::mem::size_of::<u16>() as u16;
    }

    (*NextName).Length = ((*RemainingName).Buffer as usize - (*NextName).Buffer as usize) as u16;
    if (*NextName).Length > 512 {
        NameValid = 0u8;
    }
    (*NextName).MaximumLength = (*NextName).Length;

    *LastName = if (*RemainingName).Length == 0 { 1u8 } else { 0u8 };
    NameValid
}

#[no_mangle]
pub unsafe extern "system" fn CmpGetSymbolicLink(
    Hive: PHHIVE,
    ObjectName: PUNICODE_STRING,
    SymbolicKcb: PCM_KEY_CONTROL_BLOCK,
    RemainingName: PUNICODE_STRING,
) -> u8 {
    let Hive = opaque_ptr(Hive);
    let ObjectName = opaque_ptr(ObjectName);
    let SymbolicKcb = opaque_ptr(SymbolicKcb);
    let RemainingName = opaque_ptr(RemainingName);

    let mut LinkCell: HCELL_INDEX = HCELL_NIL;
    let mut LinkValue: PCM_KEY_VALUE = core::ptr::null_mut();
    let mut LinkName: PWSTR = core::ptr::null_mut();
    let mut LinkNameAllocated: u8 = 0u8;
    let mut NewBuffer: PWSTR;
    let mut Length: u32 = 0;
    let mut ValueLength: u32 = 0;
    let mut Result: u8 = 0u8;
    let mut CellToRelease: HCELL_INDEX = HCELL_NIL;
    let Node: PCM_KEY_NODE;
    let mut NewObjectName: _UNICODE_STRING = core::mem::zeroed();

    'Exit: {
        if (*SymbolicKcb).__bindgen_anon_1.Delete() != 0 {
            return 0u8;
        }

        Node = HvGetCell(
            (*SymbolicKcb).__bindgen_anon_2.__bindgen_anon_1.KeyHive,
            (*SymbolicKcb).__bindgen_anon_2.__bindgen_anon_1.KeyCell,
        ) as PCM_KEY_NODE;
        if Node.is_null() {
            break 'Exit;
        }

        LinkCell = CmpFindValueByName(Hive, Node, core::ptr::addr_of_mut!(CmSymbolicLinkValueName));
        HvReleaseCell(
            (*SymbolicKcb).__bindgen_anon_2.__bindgen_anon_1.KeyHive,
            (*SymbolicKcb).__bindgen_anon_2.__bindgen_anon_1.KeyCell,
        );
        if LinkCell == HCELL_NIL {
            break 'Exit;
        }

        LinkValue = HvGetCell(Hive, LinkCell) as PCM_KEY_VALUE;
        if LinkValue.is_null() {
            break 'Exit;
        }

        if (*LinkValue).Type != REG_LINK {
            break 'Exit;
        }

        if CmpGetValueData(
            Hive,
            LinkValue,
            core::ptr::addr_of_mut!(ValueLength),
            core::ptr::addr_of_mut!(LinkName) as *mut *mut core::ffi::c_void,
            core::ptr::addr_of_mut!(LinkNameAllocated),
            core::ptr::addr_of_mut!(CellToRelease),
        ) == 0
        {
            break 'Exit;
        }

        Length = ValueLength + core::mem::size_of::<u16>() as u32;

        if *LinkName != OBJ_NAME_PATH_SEPARATOR {
            break 'Exit;
        }

        if !RemainingName.is_null() {
            Length += (*RemainingName).Length as u32 + core::mem::size_of::<u16>() as u32;
        }

        if Length > 0xFFFF {
            break 'Exit;
        }

        if Length > (*ObjectName).MaximumLength as u32 {
            NewBuffer = ExAllocatePoolWithTag(PagedPool, Length as usize, TAG_CM) as PWSTR;
            if NewBuffer.is_null() {
                break 'Exit;
            }

            NewObjectName.Buffer = NewBuffer;
            NewObjectName.MaximumLength = Length as u16;
            NewObjectName.Length = ValueLength as u16;
            core::ptr::copy_nonoverlapping(LinkName as *const u8, NewBuffer as *mut u8, ValueLength as usize);

            if !RemainingName.is_null() {
                *NewBuffer.offset((ValueLength / core::mem::size_of::<u16>() as u32) as isize) = OBJ_NAME_PATH_SEPARATOR;
                NewObjectName.Length += core::mem::size_of::<u16>() as u16;
                RtlAppendUnicodeStringToString(core::ptr::addr_of_mut!(NewObjectName), RemainingName);
            }

            ExFreePool((*ObjectName).Buffer as *mut core::ffi::c_void);
            *ObjectName = NewObjectName;
        } else {
            (*ObjectName).Length = ValueLength as u16;
            if !RemainingName.is_null() {
                core::ptr::copy(
                    (*RemainingName).Buffer as *const u8,
                    (*ObjectName).Buffer.offset((ValueLength / core::mem::size_of::<u16>() as u32) as isize + 1) as *mut u8,
                    (*RemainingName).Length as usize,
                );

                *(*ObjectName).Buffer.offset((ValueLength / core::mem::size_of::<u16>() as u32) as isize) = OBJ_NAME_PATH_SEPARATOR;
                (*ObjectName).Length += (*RemainingName).Length + core::mem::size_of::<u16>() as u16;
            }

            core::ptr::copy_nonoverlapping(LinkName as *const u8, (*ObjectName).Buffer as *mut u8, ValueLength as usize);
        }

        *(*ObjectName).Buffer.offset(((*ObjectName).Length / core::mem::size_of::<u16>() as u16) as isize) = UNICODE_NULL;
        Result = 1u8;
    }

    if LinkNameAllocated != 0 {
        ExFreePool(LinkName as *mut core::ffi::c_void);
    }

    if !LinkValue.is_null() {
        HvReleaseCell(Hive, LinkCell);
    }

    if CellToRelease != HCELL_NIL {
        HvReleaseCell(Hive, CellToRelease);
    }
    Result
}

unsafe fn CmpDoCreateChild(
    Hive: PHHIVE,
    ParentCell: HCELL_INDEX,
    ParentDescriptor: PSECURITY_DESCRIPTOR,
    AccessState: PACCESS_STATE,
    Name: PUNICODE_STRING,
    AccessMode: KPROCESSOR_MODE,
    ParseContext: PCM_PARSE_CONTEXT,
    ParentKcb: PCM_KEY_CONTROL_BLOCK,
    Flags: u32,
    KeyCell: PHCELL_INDEX,
    Object: *mut *mut core::ffi::c_void,
) -> i32 {
    let Hive = opaque_ptr(Hive);
    let ParentDescriptor = opaque_ptr(ParentDescriptor);
    let AccessState = opaque_ptr(AccessState);
    let Name = opaque_ptr(Name);
    let ParseContext = opaque_ptr(ParseContext);
    let ParentKcb = opaque_ptr(ParentKcb);
    let KeyCell = opaque_ptr(KeyCell);
    let Object = opaque_ptr(Object);

    const HCELL_NIL: HCELL_INDEX = 0xFFFFFFFF;
    const Stable: u32 = 0;
    const Volatile: u32 = 1;
    const REG_OPTION_VOLATILE: u32 = 0x00000001;
    const CM_KEY_BODY_TYPE: u32 = 0x6B793032;
    const CM_KEY_NODE_SIGNATURE: u16 = 0x6B6E;
    const KEY_COMP_NAME: u16 = 0x0020;
    const CMP_LOCK_HASHES_FOR_KCB: u32 = 0x00000002;
    const CMP_ENLIST_KCB_LOCKED_EXCLUSIVE: u32 = 0x00000002;
    const REG_NOTIFY_CHANGE_NAME: u32 = 0x00000001;

    let mut Status: i32 = STATUS_SUCCESS;
    let mut KeyBody: PCM_KEY_BODY;
    let mut ClassCell: HCELL_INDEX = HCELL_NIL;
    let KeyNode: PCM_KEY_NODE;
    let CellData: PCELL_DATA;
    let mut StorageType: u32;
    let Kcb: PCM_KEY_CONTROL_BLOCK;
    let mut NewDescriptor: PSECURITY_DESCRIPTOR = core::ptr::null_mut();

    'Quickie: loop {
        StorageType = Stable;
        if (*ParseContext).CreateOptions & REG_OPTION_VOLATILE != 0 {
            StorageType = Volatile;
        }

        *KeyCell = HvAllocateCell(
            Hive,
            core::mem::offset_of!(CM_KEY_NODE, Name) as u32 + CmpNameSize(Hive, Name) as u32,
            StorageType,
            HCELL_NIL,
        );
        if *KeyCell == HCELL_NIL {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'Quickie;
        }

        KeyNode = HvGetCell(Hive, *KeyCell) as PCM_KEY_NODE;
        if KeyNode.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'Quickie;
        }

        HvReleaseCell(Hive, *KeyCell);

        if (*ParseContext).Class.Length > 0 {
            ClassCell = HvAllocateCell(
                Hive,
                (*ParseContext).Class.Length as u32,
                StorageType,
                HCELL_NIL,
            );
            if ClassCell == HCELL_NIL {
                Status = STATUS_INSUFFICIENT_RESOURCES;
                break 'Quickie;
            }
        }

        Status = ObCreateObject(
            AccessMode,
            CmpKeyObjectType,
            core::ptr::null_mut(),
            AccessMode,
            core::ptr::null_mut(),
            core::mem::size_of::<CM_KEY_BODY>() as u32,
            0,
            0,
            Object,
        );
        if !NT_SUCCESS(Status) {
            break 'Quickie;
        }

        KeyBody = (*Object) as PCM_KEY_BODY;
        (*KeyBody).Type = CM_KEY_BODY_TYPE;
        (*KeyBody).KeyControlBlock = core::ptr::null_mut();
        (*KeyBody).KcbLocked = 0u8;

        if (*ParseContext).Class.Length > 0 {
            CellData = HvGetCell(Hive, ClassCell) as PCELL_DATA;
            if CellData.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
                ObDereferenceObject(*Object);
                break 'Quickie;
            }

            HvReleaseCell(Hive, ClassCell);

            core::ptr::copy_nonoverlapping(
                (*ParseContext).Class.Buffer,
                (*CellData).u.KeyString.as_mut_ptr(),
                (*ParseContext).Class.Length as usize / 2,
            );
        }

        (*KeyNode).Signature = CM_KEY_NODE_SIGNATURE;
        (*KeyNode).Flags = Flags as u16;
        {
            let mut now: LARGE_INTEGER = core::mem::zeroed();
            KeQuerySystemTime(core::ptr::addr_of_mut!(now));
            let time_ptr = core::ptr::addr_of_mut!((*KeyNode).LastWriteTime);
            *(time_ptr as *mut LARGE_INTEGER) = now;
        }
        (*KeyNode).Spare = 0;
        (*KeyNode).Parent = ParentCell;
        (*KeyNode).SubKeyCounts[Stable as usize] = 0;
        (*KeyNode).SubKeyCounts[Volatile as usize] = 0;
        (*KeyNode).__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[Stable as usize] = HCELL_NIL;
        (*KeyNode).__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[Volatile as usize] = HCELL_NIL;
        (*KeyNode).__bindgen_anon_1.__bindgen_anon_1.ValueList.Count = 0;
        (*KeyNode).__bindgen_anon_1.__bindgen_anon_1.ValueList.List = HCELL_NIL;
        (*KeyNode).Security = HCELL_NIL;
        (*KeyNode).Class = ClassCell;
        (*KeyNode).ClassLength = (*ParseContext).Class.Length;
        (*KeyNode).MaxValueDataLen = 0;
        (*KeyNode).MaxNameLen = 0;
        (*KeyNode).MaxValueNameLen = 0;
        (*KeyNode).MaxClassLen = 0;
        (*KeyNode).NameLength = CmpCopyName(
            Hive,
            core::ptr::addr_of_mut!((*KeyNode).Name) as *mut u16,
            Name,
        );
        if (*KeyNode).NameLength < (*Name).Length {
            (*KeyNode).Flags |= KEY_COMP_NAME;
        }

        Kcb = CmpCreateKeyControlBlock(
            Hive,
            *KeyCell,
            KeyNode,
            ParentKcb,
            CMP_LOCK_HASHES_FOR_KCB,
            Name,
        );
        if Kcb.is_null() {
            ObDereferenceObjectDeferDelete(*Object);
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'Quickie;
        }

        (*KeyBody).NotifyBlock = core::ptr::null_mut();
        core::ptr::write_volatile(
            core::ptr::addr_of_mut!((*KeyBody).ProcessID),
            opaque_ptr(PsGetCurrentProcessId()),
        );
        (*KeyBody).KeyControlBlock = Kcb;

        EnlistKeyBodyWithKCB(KeyBody, CMP_ENLIST_KCB_LOCKED_EXCLUSIVE);

        Status = SeAssignSecurity(
            ParentDescriptor,
            (*AccessState).SecurityDescriptor,
            core::ptr::addr_of_mut!(NewDescriptor),
            1u8,
            core::ptr::addr_of_mut!((*AccessState).SubjectSecurityContext),
            core::ptr::addr_of_mut!((*CmpKeyObjectType).TypeInfo.GenericMapping),
            0,
        );
        if NT_SUCCESS(Status) {
            Status = CmpAssignSecurityDescriptor(Kcb, NewDescriptor);
        }

        SeDeassignSecurity(core::ptr::addr_of_mut!(NewDescriptor));

        if NT_SUCCESS(Status) {
            CmpReportNotify(
                Kcb,
                Hive,
                (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyCell,
                REG_NOTIFY_CHANGE_NAME,
            );
        }

        return Status;
    }

    if !NT_SUCCESS(Status) {
        if (*ParseContext).Class.Length > 0 {
            HvFreeCell(Hive, ClassCell);
        }
        HvFreeCell(Hive, *KeyCell);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn CmpDoCreate(
    Hive: PHHIVE,
    Cell: HCELL_INDEX,
    AccessState: PACCESS_STATE,
    Name: PUNICODE_STRING,
    AccessMode: KPROCESSOR_MODE,
    ParseContext: PCM_PARSE_CONTEXT,
    ParentKcb: PCM_KEY_CONTROL_BLOCK,
    Object: *mut *mut core::ffi::c_void,
) -> i32 {
    let Hive = opaque_ptr(Hive);
    let AccessState = opaque_ptr(AccessState);
    let Name = opaque_ptr(Name);
    let ParseContext = opaque_ptr(ParseContext);
    let ParentKcb = opaque_ptr(ParentKcb);
    let Object = opaque_ptr(Object);

    const Volatile: u32 = 1;
    const REG_OPTION_VOLATILE: u32 = 0x00000001;
    const CM_KCB_READ_ONLY_KEY: u32 = 0x0080;
    const KEY_SYM_LINK: u16 = 0x0010;
    const HCELL_NIL: HCELL_INDEX = 0xFFFFFFFF;
    const STATUS_ACCESS_DENIED: i32 = 0xC0000022u32 as i32;
    const STATUS_OBJECT_NAME_NOT_FOUND: i32 = 0xC0000034u32 as i32;
    const STATUS_REPARSE: i32 = 0x00000104u32 as i32;
    const STATUS_CHILD_MUST_BE_VOLATILE: i32 = 0xC0000181u32 as i32;

    let mut Status: i32;
    let CellData: PCELL_DATA;
    let mut KeyCell: HCELL_INDEX = 0;
    let ParentType: u32;
    let KeyBody: PCM_KEY_BODY;
    let SecurityDescriptor: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut TimeStamp: LARGE_INTEGER = core::mem::zeroed();
    let mut KeyNode: PCM_KEY_NODE;

    CMP_ASSERT_KCB_LOCK(ParentKcb);
    CmpLockHiveFlusherShared(Hive as PCMHIVE);

    'Exit: {
        if (*ParentKcb).__bindgen_anon_1.ExtFlags() & CM_KCB_READ_ONLY_KEY != 0 {
            Status = STATUS_ACCESS_DENIED;
            break 'Exit;
        }

        if (*ParentKcb).__bindgen_anon_1.Delete() != 0 {
            Status = STATUS_OBJECT_NAME_NOT_FOUND;
            break 'Exit;
        }

        KeyNode = HvGetCell(Hive, Cell) as PCM_KEY_NODE;
        if KeyNode.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'Exit;
        }

        if CmpFindSubKeyByName(Hive, KeyNode, Name) != HCELL_NIL {
            Status = STATUS_REPARSE;
            break 'Exit;
        }

        ParentType = HvGetCellType(Cell);
        if (ParentType == Volatile) && ((*ParseContext).CreateOptions & REG_OPTION_VOLATILE == 0) {
            Status = STATUS_CHILD_MUST_BE_VOLATILE;
            break 'Exit;
        }

        if (*ParentKcb).__bindgen_anon_5.Flags() as u32 & KEY_SYM_LINK as u32 != 0 {
            Status = STATUS_ACCESS_DENIED;
            break 'Exit;
        }

        HvMarkCellDirty(Hive, Cell, 0u8);

        Status = CmpDoCreateChild(
            Hive,
            Cell,
            SecurityDescriptor,
            AccessState,
            Name,
            AccessMode,
            ParseContext,
            ParentKcb,
            0,
            core::ptr::addr_of_mut!(KeyCell),
            Object,
        );

        if NT_SUCCESS(Status) {
            KeyBody = *Object as PCM_KEY_BODY;

            if CmpAddSubKey(Hive, Cell, KeyCell) == 0 {
                CmpFreeKeyByCell(Hive, KeyCell, 0u8);

                (*(*KeyBody).KeyControlBlock).__bindgen_anon_1.set_Delete(1);
                CmpRemoveKeyControlBlock((*KeyBody).KeyControlBlock);

                ObDereferenceObjectDeferDelete(*Object);
                Status = STATUS_INSUFFICIENT_RESOURCES;
                break 'Exit;
            }

            KeyNode = HvGetCell(Hive, Cell) as PCM_KEY_NODE;
            if KeyNode.is_null() {
                CmpFreeKeyByCell(Hive, KeyCell, 1u8);

                (*(*KeyBody).KeyControlBlock).__bindgen_anon_1.set_Delete(1);
                CmpRemoveKeyControlBlock((*KeyBody).KeyControlBlock);

                ObDereferenceObjectDeferDelete(*Object);
                Status = STATUS_INSUFFICIENT_RESOURCES;
                break 'Exit;
            }

            CmpCleanUpSubKeyInfo((*(*KeyBody).KeyControlBlock).ParentKcb);

            KeQuerySystemTime(core::ptr::addr_of_mut!(TimeStamp));

            (*KeyNode).LastWriteTime = TimeStamp;
            (*(*(*KeyBody).KeyControlBlock).ParentKcb).KcbLastWriteTime = TimeStamp;

            if ((*KeyNode).MaxNameLen as u32) < (*Name).Length as u32 {
                (*KeyNode).MaxNameLen = (*Name).Length as u32;
                (*(*(*KeyBody).KeyControlBlock).ParentKcb).KcbMaxNameLen = (*Name).Length;
            }

            if ((*KeyNode).MaxClassLen as u32) < (*ParseContext).Class.Length as u32 {
                (*KeyNode).MaxClassLen = (*ParseContext).Class.Length as u32;
            }

            if (*ParseContext).CreateOptions & REG_OPTION_CREATE_LINK != 0 {
                CellData = HvGetCell(Hive, KeyCell) as PCELL_DATA;
                if CellData.is_null() {
                    CmpFreeKeyByCell(Hive, KeyCell, 1u8);

                    (*(*KeyBody).KeyControlBlock).__bindgen_anon_1.set_Delete(1);
                    CmpRemoveKeyControlBlock((*KeyBody).KeyControlBlock);

                    ObDereferenceObjectDeferDelete(*Object);
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    break 'Exit;
                }

                (*CellData).u.KeyNode.Flags |= KEY_SYM_LINK;
                (*(*KeyBody).KeyControlBlock)
                    .__bindgen_anon_5
                    .set_Flags((*CellData).u.KeyNode.Flags as u32);
                HvReleaseCell(Hive, KeyCell);
            }
        }
    }

    CmpUnlockHiveFlusher(Hive as PCMHIVE);
    Status
}

unsafe fn CmpDoOpen(
    Hive: PHHIVE,
    Cell: HCELL_INDEX,
    Node: PCM_KEY_NODE,
    AccessState: PACCESS_STATE,
    AccessMode: KPROCESSOR_MODE,
    Attributes: u32,
    Context: PCM_PARSE_CONTEXT,
    ControlFlags: u32,
    CachedKcb: *mut PCM_KEY_CONTROL_BLOCK,
    KcbsLocked: *mut u32,
    KeyName: PUNICODE_STRING,
    Object: *mut *mut core::ffi::c_void,
) -> i32 {
    let Hive = opaque_ptr(Hive);
    let Node = opaque_ptr(Node);
    let AccessState = opaque_ptr(AccessState);
    let Context = opaque_ptr(Context);
    let CachedKcb = opaque_ptr(CachedKcb);
    let KcbsLocked = opaque_ptr(KcbsLocked);
    let KeyName = opaque_ptr(KeyName);
    let Object = opaque_ptr(Object);
    let mut Status: i32;
    let mut LockKcb: u8 = 0u8;
    let mut IsLockShared: u8 = 0u8;
    let mut KeyBody: PCM_KEY_BODY = core::ptr::null_mut();
    let mut Kcb: PCM_KEY_CONTROL_BLOCK = core::ptr::null_mut();

    // C: HiveFlags & HIVE_IS_UNLOADING. 0x4 is HIVE_HAS_BEEN_REPLACED — it happened
    // to match CmLoadKey's equally-wrong 0x4, so this guard worked on the LOAD path
    // and silently did nothing on the unload path it exists for.
    if (((*Hive).HiveFlags & crate::hiveflags::HIVE_IS_UNLOADING) != 0) &&
        ((*(Hive as PCMHIVE)).CreatorOwner != KeGetCurrentThread() as _)
    {
        return 0xC0000034u32 as i32;
    }

    if !Context.is_null() {
        if (*Context).CreateLink != 0 {
            return 0xC0000022u32 as i32;
        }

        // C: `Context->CreateOptions & REG_OPTION_CREATE_LINK`, which is 0x2.
        // The literal was 0x20, which is not any REG_OPTION_* bit at all, so
        // this guard could never fire: creating a symlink over a key that
        // already exists returned success and opened the existing key instead
        // of STATUS_OBJECT_NAME_COLLISION.
        if ((*Context).CreateOptions & REG_OPTION_CREATE_LINK) != 0 {
            return 0xC0000035u32 as i32;
        }

        (*Context).Disposition = 0x00000001;
    }

    if (ControlFlags & 0x00000002) != 0 {
        LockKcb = 1u8;
    }

    if (ControlFlags & 0x00000004) != 0 {
        IsLockShared = 1u8;

        if (((**CachedKcb).__bindgen_anon_5.Flags() & 0x00000010) != 0) && ((Attributes & 0x00000100) == 0) {
            // C: `ExtFlags & CM_KCB_SYM_LINK_FOUND`, which is 0x08
            // (ntoskrnl/include/internal/cm.h). The literal here was 0x40 —
            // CM_KCB_INVALID_CACHED_INFO. Two failure modes, both silent: a
            // symlink whose target IS resolved took the reparse path instead of
            // using RealKcb, and a KCB carrying INVALID_CACHED_INFO read
            // `ValueCache.RealKcb` out of a union that at that point holds value
            // cache data, not a KCB pointer.
            if ((**CachedKcb).__bindgen_anon_1.ExtFlags() & CM_KCB_SYM_LINK_FOUND) != 0 {
                Kcb = (**CachedKcb).ValueCache.__bindgen_anon_1.RealKcb;

                // Three bugs lived here, all on the path that resolves a registry
                // symlink -- most prominently CurrentControlSet, i.e. essentially every
                // boot-time driver and service lookup. Upstream-Commit: a45832a8ae4
                //
                //   1. `Delete` on the REAL kcb was read while only the SYMLINK's hash
                //      bucket was locked, so the flag could change under us (TOCTOU).
                //   2. the relock below took GET_HASH_INDEX of the SYMLINK's ConvKey,
                //      not the real KCB's, so the KCB actually being used was never the
                //      one locked.
                //   3. all locks were dropped before the real KCB was ever referenced,
                //      so nothing pinned it in existence across the gap.
                //
                // Reference first, then move the lock to the REAL KCB, and only then
                // look at Delete.
                if CmpReferenceKeyControlBlock(Kcb) == 0 {
                    crate::dbg_print!(b"Failed to reference the real KCB, attempt a reparse\n\0");
                    return 0x00000104u32 as i32; // STATUS_REPARSE
                }

                CmpUnLockKcbArray(KcbsLocked);
                *KcbsLocked.offset(0) = 1;
                *KcbsLocked.offset(1) = GET_HASH_INDEX((*Kcb).__bindgen_anon_2.__bindgen_anon_1.ConvKey);
                CmpAcquireKcbLockExclusiveByIndex(*KcbsLocked.offset(1));
                // Back to exclusive, or EnlistKeyBodyWithKCB unlocks and relocks later.
                IsLockShared = 0u8;

                if (*Kcb).__bindgen_anon_1.Delete() != 0 {
                    crate::dbg_print!(b"The real KCB is deleted, attempt a reparse\n\0");
                    CmpUnLockKcbArray(KcbsLocked);
                    CmpDereferenceKeyControlBlock(Kcb);
                    CmpAcquireKcbLockExclusiveByIndex(GET_HASH_INDEX((**CachedKcb).__bindgen_anon_2.__bindgen_anon_1.ConvKey));
                    CmpCleanUpKcbValueCache(*CachedKcb);
                    *KcbsLocked.offset(0) = 1;
                    *KcbsLocked.offset(1) = GET_HASH_INDEX((**CachedKcb).__bindgen_anon_2.__bindgen_anon_1.ConvKey);
                    return 0x00000104u32 as i32; // STATUS_REPARSE
                }
            } else {
                return 0x00000104u32 as i32; // STATUS_REPARSE
            }
        } else {
            Kcb = *CachedKcb;
        }

        if CmpReferenceKeyControlBlock(Kcb) == 0 {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
    } else {
        if (((*Node).Flags & 0x0010) != 0) && ((Attributes & 0x00000100) == 0) {
            Kcb = CmpCreateKeyControlBlock(
                Hive,
                Cell,
                Node,
                *CachedKcb,
                if LockKcb != 0 { 0x00000002 } else { 0 },
                KeyName,
            );
            if Kcb.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            *CachedKcb = Kcb;
            return 0x00000104u32 as i32; // STATUS_REPARSE
        }

        Kcb = CmpCreateKeyControlBlock(
            Hive,
            Cell,
            Node,
            *CachedKcb,
            if LockKcb != 0 { 0x00000002 } else { 0 },
            KeyName,
        );
        if Kcb.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        *CachedKcb = Kcb;
    }

    Status = ObCreateObject(
        AccessMode,
        CmpKeyObjectType,
        core::ptr::null_mut(),
        AccessMode,
        core::ptr::null_mut(),
        core::mem::size_of::<CM_KEY_BODY>() as u32,
        0,
        0,
        Object,
    );
    if NT_SUCCESS(Status) {
        KeyBody = *Object as PCM_KEY_BODY;
        (*KeyBody).KeyControlBlock = Kcb;
        (*KeyBody).Type = 0x6b793032;
        core::ptr::write_volatile(
            core::ptr::addr_of_mut!((*KeyBody).ProcessID),
            opaque_ptr(PsGetCurrentProcessId()),
        );
        (*KeyBody).NotifyBlock = core::ptr::null_mut();

        EnlistKeyBodyWithKCB(
            KeyBody,
            if IsLockShared != 0 { 0x00000001 } else { 0x00000002 },
        );

        (*KeyBody).KcbLocked = 1u8;

        if ObCheckObjectAccess(
            *Object,
            AccessState,
            0u8,
            AccessMode,
            core::ptr::addr_of_mut!(Status),
        ) == 0
        {
            ObDereferenceObject(*Object);
        }

        (*KeyBody).KcbLocked = 0u8;
    } else {
        CmpDereferenceKeyControlBlockWithLock(Kcb, 0u8);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn CmpCreateLinkNode(
    Hive: PHHIVE,
    Cell: HCELL_INDEX,
    AccessState: PACCESS_STATE,
    Name: _UNICODE_STRING,
    AccessMode: KPROCESSOR_MODE,
    CreateOptions: u32,
    Context: PCM_PARSE_CONTEXT,
    ParentKcb: PCM_KEY_CONTROL_BLOCK,
    KcbsLocked: *mut u32,
    Object: *mut *mut core::ffi::c_void,
) -> i32 {
    let Hive = opaque_ptr(Hive);
    let AccessState = opaque_ptr(AccessState);
    let Context = opaque_ptr(Context);
    let ParentKcb = opaque_ptr(ParentKcb);
    let KcbsLocked = opaque_ptr(KcbsLocked);
    let Object = opaque_ptr(Object);

    const CM_KCB_READ_ONLY_KEY: u32 = 0x00000080;
    const KEY_HIVE_ENTRY: u32 = 0x0004;
    const KEY_HIVE_EXIT: u32 = 0x0002;
    const KEY_NO_DELETE: u32 = 0x0008;
    const KEY_COMP_NAME: u16 = 0x0020;
    const CM_LINK_NODE_SIGNATURE: u16 = 0x6b6c;
    const CMP_CREATE_KCB_KCB_LOCKED: u32 = 0x00000002;
    const HCELL_NIL: HCELL_INDEX = 0xFFFFFFFF;
    const Stable: u32 = 0;
    const Volatile: u32 = 1;
    const STATUS_ACCESS_DENIED: i32 = 0xC0000022u32 as i32;
    const STATUS_OBJECT_NAME_NOT_FOUND: i32 = 0xC0000034u32 as i32;

    let mut Status: i32;
    let mut KeyCell: HCELL_INDEX;
    let mut LinkCell: HCELL_INDEX;
    let mut ChildCell: HCELL_INDEX = HCELL_NIL;
    let mut KeyBody: PCM_KEY_BODY;
    let mut TimeStamp: LARGE_INTEGER = core::mem::zeroed();
    let mut KeyNode: PCM_KEY_NODE;
    let mut Kcb: PCM_KEY_CONTROL_BLOCK = ParentKcb;

    if Hive != core::ptr::addr_of_mut!((*CmiVolatileHive).Hive) {
        crate::dbg_print!(b"Invalid link node attempt\n\0");
        return STATUS_ACCESS_DENIED;
    }

    CMP_ASSERT_KCB_LOCK(ParentKcb);
    CmpLockHiveFlusherShared(Hive as PCMHIVE);
    CmpLockHiveFlusherShared((*Context).ChildHive.KeyHive as PCMHIVE);

    if ((*ParentKcb).__bindgen_anon_1.ExtFlags() & CM_KCB_READ_ONLY_KEY) != 0 {
        Status = STATUS_ACCESS_DENIED;
        CmpUnlockHiveFlusher((*Context).ChildHive.KeyHive as PCMHIVE);
        CmpUnlockHiveFlusher(Hive as PCMHIVE);
        return Status;
    }

    if (*ParentKcb).__bindgen_anon_1.Delete() != 0 {
        Status = STATUS_OBJECT_NAME_NOT_FOUND;
        CmpUnlockHiveFlusher((*Context).ChildHive.KeyHive as PCMHIVE);
        CmpUnlockHiveFlusher(Hive as PCMHIVE);
        return Status;
    }

    let name_size = CmpNameSize(Hive, core::ptr::addr_of!(Name));
    let offset = core::mem::size_of::<CM_KEY_NODE>() - core::mem::size_of::<[u16; 1]>();
    LinkCell = HvAllocateCell(
        Hive,
        (offset + name_size as usize) as u32,
        Stable,
        HCELL_NIL,
    );
    if LinkCell == HCELL_NIL {
        Status = STATUS_INSUFFICIENT_RESOURCES;
        CmpUnlockHiveFlusher((*Context).ChildHive.KeyHive as PCMHIVE);
        CmpUnlockHiveFlusher(Hive as PCMHIVE);
        return Status;
    }

    KeyCell = (*Context).ChildHive.KeyCell;
    if KeyCell != HCELL_NIL {
        ChildCell = KeyCell;

        KeyNode = HvGetCell((*Context).ChildHive.KeyHive, ChildCell) as PCM_KEY_NODE;
        if KeyNode.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            CmpUnlockHiveFlusher((*Context).ChildHive.KeyHive as PCMHIVE);
            CmpUnlockHiveFlusher(Hive as PCMHIVE);
            return Status;
        }

        (*KeyNode).Parent = LinkCell;
        (*KeyNode).Flags |= (KEY_HIVE_ENTRY | KEY_NO_DELETE) as u16;
        HvReleaseCell((*Context).ChildHive.KeyHive, ChildCell);

        KeyNode = HvGetCell((*Context).ChildHive.KeyHive, KeyCell) as PCM_KEY_NODE;
        if KeyNode.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            CmpUnlockHiveFlusher((*Context).ChildHive.KeyHive as PCMHIVE);
            CmpUnlockHiveFlusher(Hive as PCMHIVE);
            return Status;
        }

        Status = CmpDoOpen(
            (*Context).ChildHive.KeyHive,
            KeyCell,
            KeyNode,
            AccessState,
            AccessMode,
            CreateOptions,
            core::ptr::null_mut(),
            CMP_CREATE_KCB_KCB_LOCKED,
            core::ptr::addr_of_mut!(Kcb),
            KcbsLocked,
            core::ptr::addr_of!(Name) as *mut _UNICODE_STRING,
            Object,
        );
        HvReleaseCell((*Context).ChildHive.KeyHive, KeyCell);
    } else {
        Status = CmpDoCreateChild(
            (*Context).ChildHive.KeyHive,
            Cell,
            core::ptr::null_mut(),
            AccessState,
            core::ptr::addr_of!(Name) as *mut _UNICODE_STRING,
            AccessMode,
            Context,
            ParentKcb,
            KEY_HIVE_ENTRY | KEY_NO_DELETE,
            core::ptr::addr_of_mut!(ChildCell),
            Object,
        );
        if NT_SUCCESS(Status) {
            (*(*(*Context).ChildHive.KeyHive).BaseBlock).RootCell = ChildCell;
        }
    }

    if NT_SUCCESS(Status) {
        HvMarkCellDirty((*Context).ChildHive.KeyHive, ChildCell, 0u8);

        KeyNode = HvGetCell((*Context).ChildHive.KeyHive, ChildCell) as PCM_KEY_NODE;
        if KeyNode.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            CmpUnlockHiveFlusher((*Context).ChildHive.KeyHive as PCMHIVE);
            CmpUnlockHiveFlusher(Hive as PCMHIVE);
            return Status;
        }

        HvReleaseCell((*Context).ChildHive.KeyHive, ChildCell);

        (*KeyNode).Parent = LinkCell;
        (*KeyNode).Flags |= (KEY_HIVE_ENTRY | KEY_NO_DELETE) as u16;

        KeyNode = HvGetCell(Hive, LinkCell) as PCM_KEY_NODE;
        if KeyNode.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            CmpUnlockHiveFlusher((*Context).ChildHive.KeyHive as PCMHIVE);
            CmpUnlockHiveFlusher(Hive as PCMHIVE);
            return Status;
        }

        (*KeyNode).Signature = CM_LINK_NODE_SIGNATURE;
        (*KeyNode).Flags = (KEY_HIVE_EXIT | KEY_NO_DELETE) as u16;
        (*KeyNode).Parent = Cell;
        (*KeyNode).NameLength = CmpCopyName(
            Hive,
            core::ptr::addr_of_mut!((*KeyNode).Name) as *mut u16,
            core::ptr::addr_of!(Name),
        );
        if ((*KeyNode).NameLength as u32) < Name.Length as u32 {
            (*KeyNode).Flags |= KEY_COMP_NAME;
        }
        KeQuerySystemTime(core::ptr::addr_of_mut!(TimeStamp));
        (*KeyNode).LastWriteTime = TimeStamp;

        (*KeyNode).SubKeyCounts[Stable as usize] = 0;
        (*KeyNode).SubKeyCounts[Volatile as usize] = 0;
        (*KeyNode).__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[Stable as usize] = HCELL_NIL;
        (*KeyNode).__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[Volatile as usize] = HCELL_NIL;
        (*KeyNode).__bindgen_anon_1.__bindgen_anon_1.ValueList.Count = 0;
        (*KeyNode).__bindgen_anon_1.__bindgen_anon_1.ValueList.List = HCELL_NIL;
        (*KeyNode).ClassLength = 0;

        (*KeyNode).__bindgen_anon_1.ChildHiveReference.KeyHive = (*Context).ChildHive.KeyHive;
        (*KeyNode).__bindgen_anon_1.ChildHiveReference.KeyCell = ChildCell;
        HvReleaseCell(Hive, LinkCell);

        KeyNode = HvGetCell(Hive, Cell) as PCM_KEY_NODE;
        if KeyNode.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            CmpUnlockHiveFlusher((*Context).ChildHive.KeyHive as PCMHIVE);
            CmpUnlockHiveFlusher(Hive as PCMHIVE);
            return Status;
        }

        if CmpAddSubKey(Hive, Cell, LinkCell) == 0 {
        }

        KeyBody = (*Object) as PCM_KEY_BODY;

        CmpCleanUpSubKeyInfo((*(*KeyBody).KeyControlBlock).ParentKcb);

        KeQuerySystemTime(core::ptr::addr_of_mut!(TimeStamp));
        (*KeyNode).LastWriteTime = TimeStamp;
        (*(*(*KeyBody).KeyControlBlock).ParentKcb).KcbLastWriteTime = TimeStamp;

        if ((*KeyNode).MaxNameLen as u32) < Name.Length as u32 {
            (*KeyNode).MaxNameLen = Name.Length as u32;
            (*(*(*KeyBody).KeyControlBlock).ParentKcb).KcbMaxNameLen = Name.Length;
        }

        if (*KeyNode).MaxClassLen < (*Context).Class.Length as u32 {
            (*KeyNode).MaxClassLen = (*Context).Class.Length as u32;
        }

        /* Assign the descriptor the caller supplied (CmpInitializeHiveList passes
         * CmpHiveRootSecurityDescriptor). Only CmpDoCreateChild used to do this, so every
         * key reached by LINKING a hive -- SAM, SECURITY, SOFTWARE, SYSTEM, .DEFAULT --
         * got no descriptor at all and fell back to granting World everything.
         *
         * ONLY when the hive carries none of its own. This is the kernel's DEFAULT, and a
         * key that already has an `sk` cell outranks it: those bytes are the ACL the hive
         * actually shipped, and mkhive writes one on every hive root. Assigning
         * unconditionally silently overwrote them -- the sk-cell read in cmse could never
         * fire, because the KCB cache was already populated by the time anyone queried, so
         * SOFTWARE/SYSTEM/.DEFAULT reported the generic default rather than their real
         * ACLs and the numbers did not move at all.
         *
         * Ask the READER, do not re-derive the test here. `KeyNode` above is the LINK cell
         * in the MASTER hive; the KCB for the linked key names the CHILD hive's root cell,
         * which is where the `sk` lives. Testing KeyNode->Security looked right, compiled,
         * booted, and changed nothing -- two hive-cell walks that are almost the same is
         * exactly the shape that hides a bug. One walk, one owner. */
        if crate::cmse::CmpCacheSecurityFromHive((*KeyBody).KeyControlBlock).is_null() {
            let mut NewDescriptor: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
            let SecStatus = SeAssignSecurity(
                core::ptr::null_mut(),
                (*AccessState).SecurityDescriptor,
                core::ptr::addr_of_mut!(NewDescriptor),
                1u8,
                core::ptr::addr_of_mut!((*AccessState).SubjectSecurityContext),
                core::ptr::addr_of_mut!((*CmpKeyObjectType).TypeInfo.GenericMapping),
                0,
            );
            if NT_SUCCESS(SecStatus) {
                CmpAssignSecurityDescriptor((*KeyBody).KeyControlBlock, NewDescriptor);
                SeDeassignSecurity(core::ptr::addr_of_mut!(NewDescriptor));
            }
        }

        HvReleaseCell(Hive, Cell);
    } else {
        HvReleaseCell(Hive, LinkCell);
    }

    CmpUnlockHiveFlusher((*Context).ChildHive.KeyHive as PCMHIVE);
    CmpUnlockHiveFlusher(Hive as PCMHIVE);
    Status
}

unsafe fn CmpHandleExitNode(
    Hive: *mut PHHIVE,
    Cell: *mut HCELL_INDEX,
    KeyNode: *mut PCM_KEY_NODE,
    ReleaseHive: *mut PHHIVE,
    ReleaseCell: *mut HCELL_INDEX,
) {

    if *ReleaseCell != HCELL_NIL {
        HvReleaseCell(*ReleaseHive, *ReleaseCell);
    }

    *Hive = (**KeyNode).__bindgen_anon_1.ChildHiveReference.KeyHive;
    *Cell = (**KeyNode).__bindgen_anon_1.ChildHiveReference.KeyCell;

    *KeyNode = HvGetCell(*Hive, *Cell) as PCM_KEY_NODE;
    if !(*KeyNode).is_null() {
        *ReleaseCell = *Cell;
        *ReleaseHive = *Hive;
    } else {
        *ReleaseCell = HCELL_NIL;
        *ReleaseHive = core::ptr::null_mut();
    }
}

unsafe fn CmpComputeHashValue(
    RemainingName: PUNICODE_STRING,
    ConvKey: ULONG,
    HashCacheStack: PCM_HASH_CACHE_STACK,
    TotalSubKeys: *mut ULONG,
) -> ULONG {

    let mut CopyConvKey: ULONG;
    let mut SubkeysInTotal: ULONG;
    let mut RemainingSubkeysInTotal: ULONG;
    let mut RemainingNameBuffer: PWCHAR;
    let mut RemainingNameLength: USHORT;
    let mut KeyNameLength: USHORT;

    RemainingNameBuffer = (*RemainingName).Buffer;
    RemainingNameLength = (*RemainingName).Length;
    if RemainingNameLength == 0 {
        *TotalSubKeys = 0;
        return 0;
    }

    while RemainingNameLength >= core::mem::size_of::<WCHAR>() as USHORT
        && *RemainingNameBuffer == OBJ_NAME_PATH_SEPARATOR as WCHAR
    {
        RemainingNameBuffer = RemainingNameBuffer.add(1);
        RemainingNameLength -= core::mem::size_of::<WCHAR>() as USHORT;
    }

    SubkeysInTotal = 0;
    RemainingSubkeysInTotal = 0;
    KeyNameLength = 0;
    CopyConvKey = ConvKey;
    (*HashCacheStack.add(RemainingSubkeysInTotal as usize)).NameOfKey.Buffer = RemainingNameBuffer;
    while RemainingNameLength > 0 {
        if *RemainingNameBuffer != OBJ_NAME_PATH_SEPARATOR as WCHAR {
            CopyConvKey = COMPUTE_HASH_CHAR(CopyConvKey, *RemainingNameBuffer);

            RemainingNameBuffer = RemainingNameBuffer.add(1);
            KeyNameLength += core::mem::size_of::<WCHAR>() as USHORT;
            RemainingNameLength -= core::mem::size_of::<WCHAR>() as USHORT;

            if RemainingNameLength == 0 {
                if RemainingSubkeysInTotal < CMP_SUBKEY_LEVELS_DEPTH_LIMIT as ULONG {
                    (*HashCacheStack.add(RemainingSubkeysInTotal as usize)).NameOfKey.Length = KeyNameLength;
                    (*HashCacheStack.add(RemainingSubkeysInTotal as usize)).NameOfKey.MaximumLength = KeyNameLength;
                    (*HashCacheStack.add(RemainingSubkeysInTotal as usize)).ConvKey = CopyConvKey;
                    RemainingSubkeysInTotal += 1;
                }

                SubkeysInTotal += 1;
            }
        } else {
            while RemainingNameLength >= core::mem::size_of::<WCHAR>() as USHORT
                && *RemainingNameBuffer == OBJ_NAME_PATH_SEPARATOR as WCHAR
            {
                RemainingNameBuffer = RemainingNameBuffer.add(1);
                RemainingNameLength -= core::mem::size_of::<WCHAR>() as USHORT;
            }

            SubkeysInTotal += 1;

            if RemainingSubkeysInTotal < CMP_SUBKEY_LEVELS_DEPTH_LIMIT as ULONG {
                (*HashCacheStack.add(RemainingSubkeysInTotal as usize)).NameOfKey.Length = KeyNameLength;
                (*HashCacheStack.add(RemainingSubkeysInTotal as usize)).NameOfKey.MaximumLength = KeyNameLength;
                (*HashCacheStack.add(RemainingSubkeysInTotal as usize)).ConvKey = CopyConvKey;

                RemainingSubkeysInTotal += 1;
                KeyNameLength = 0;

                if RemainingSubkeysInTotal < CMP_SUBKEY_LEVELS_DEPTH_LIMIT as ULONG {
                    (*HashCacheStack.add(RemainingSubkeysInTotal as usize)).NameOfKey.Buffer = RemainingNameBuffer;
                }
            }
        }
    }

    *TotalSubKeys = SubkeysInTotal;
    return RemainingSubkeysInTotal;
}

unsafe fn CmpCompareSubkeys(
    HashCacheStack: PCM_HASH_CACHE_STACK,
    CurrentKcb: PCM_KEY_CONTROL_BLOCK,
    RemainingSubkeys: u32,
    ParentKcb: *mut PCM_KEY_CONTROL_BLOCK,
) -> u8 {
    let mut HashStackIndex: i32;
    let mut Result: i32;
    let mut NameBlock: PCM_NAME_CONTROL_BLOCK;
    let mut CurrentNameBlock: _UNICODE_STRING = core::mem::zeroed();

    HashStackIndex = RemainingSubkeys as i32;
    while HashStackIndex >= 0 {
        if (*CurrentKcb).__bindgen_anon_2.__bindgen_anon_1.ConvKey
            != (*HashCacheStack.offset(HashStackIndex as isize)).ConvKey
        {
            *ParentKcb = CurrentKcb;
            return 0u8;
        }

        NameBlock = (*CurrentKcb).NameBlock;
        if (*NameBlock).Compressed != 0 {
            Result = CmpCompareCompressedName(
                core::ptr::addr_of_mut!((*HashCacheStack.offset(HashStackIndex as isize)).NameOfKey),
                (*NameBlock).__bindgen_anon_1.__bindgen_anon_1.Name.as_mut_ptr(),
                (*NameBlock).__bindgen_anon_1.__bindgen_anon_1.NameLength as u32,
            );
        } else {
            CurrentNameBlock.Buffer = (*NameBlock).__bindgen_anon_1.__bindgen_anon_1.Name.as_mut_ptr();
            CurrentNameBlock.Length = (*NameBlock).__bindgen_anon_1.__bindgen_anon_1.NameLength;
            CurrentNameBlock.MaximumLength = (*NameBlock).__bindgen_anon_1.__bindgen_anon_1.NameLength;

            Result = RtlCompareUnicodeString(
                core::ptr::addr_of_mut!((*HashCacheStack.offset(HashStackIndex as isize)).NameOfKey),
                core::ptr::addr_of_mut!(CurrentNameBlock),
                1u8,
            );
        }

        if Result != 0 {
            *ParentKcb = CurrentKcb;
            return 0u8;
        }

        HashStackIndex -= 1;
    }

    *ParentKcb = (*CurrentKcb).ParentKcb;
    return 1u8;
}

unsafe fn CmpRemoveSubkeysInRemainingName(
    HashCacheStack: PCM_HASH_CACHE_STACK,
    RemainingSubkeys: ULONG,
    RemainingName: PUNICODE_STRING)
{
    let mut HashStackIndex: ULONG = 0;

    while (*RemainingName).Length >= core::mem::size_of::<WCHAR>() as u16 &&
           *(*RemainingName).Buffer.offset(0) == OBJ_NAME_PATH_SEPARATOR as u16
    {
        (*RemainingName).Buffer = (*RemainingName).Buffer.offset(1);
        (*RemainingName).Length -= core::mem::size_of::<WCHAR>() as u16;
    }

    while HashStackIndex <= RemainingSubkeys
    {
        (*RemainingName).Buffer = (*RemainingName).Buffer.offset(
            (*HashCacheStack.offset(HashStackIndex as isize)).NameOfKey.Length as isize / core::mem::size_of::<WCHAR>() as isize
        );
        (*RemainingName).Length -= (*HashCacheStack.offset(HashStackIndex as isize)).NameOfKey.Length;

        while (*RemainingName).Length >= core::mem::size_of::<WCHAR>() as u16 &&
               *(*RemainingName).Buffer.offset(0) == OBJ_NAME_PATH_SEPARATOR as u16
        {
            (*RemainingName).Buffer = (*RemainingName).Buffer.offset(1);
            (*RemainingName).Length -= core::mem::size_of::<WCHAR>() as u16;
        }

        HashStackIndex += 1;
    }
}

unsafe fn CmpLookInCache(
    HashCacheStack: PCM_HASH_CACHE_STACK,
    LockKcbsExclusive: u8,
    TotalRemainingSubkeys: u32,
    RemainingName: PUNICODE_STRING,
    OuterStackArray: *mut u32,
    Kcb: *mut PCM_KEY_CONTROL_BLOCK,
    Hive: *mut PHHIVE,
    Cell: PHCELL_INDEX,
    MatchRemainSubkeyLevel: *mut u32,
) -> i32 {
    let mut RemainingSubkeys: i32;
    let mut TotalLevels: u32;
    let mut SubkeysMatch: u8;
    let mut CurrentKcb: PCM_KEY_CONTROL_BLOCK = core::ptr::null_mut();
    let mut ParentKcb: PCM_KEY_CONTROL_BLOCK = core::ptr::null_mut();
    let mut HashEntry: PCM_KEY_HASH = core::ptr::null_mut();
    let mut KeyFoundInCache: u8 = 0u8;
    let mut LockedKcbs: *mut u32 = core::ptr::null_mut();

    if CmpReferenceKeyControlBlock(*Kcb) == 0 {
        crate::dbg_print!(
            b"Could not reference the KCB, too many references (KCB 0x%p)\n\0",
            Kcb
        );
        return STATUS_UNSUCCESSFUL;
    }

    LockedKcbs = CmpBuildAndLockKcbArray(
        HashCacheStack,
        if LockKcbsExclusive != 0 { 1 } else { 2 },
        *Kcb,
        OuterStackArray,
        TotalRemainingSubkeys,
        0,
    );

    RemainingSubkeys = TotalRemainingSubkeys as i32 - 1;
    TotalLevels = TotalRemainingSubkeys + (**Kcb).__bindgen_anon_1.TotalLevels() + 1;

    while RemainingSubkeys >= 0 {
        HashEntry = GET_HASH_ENTRY(CmpCacheTable, (*HashCacheStack.offset(RemainingSubkeys as isize)).ConvKey);

        TotalLevels -= 1;

        while !HashEntry.is_null() {
            // CONTAINING_RECORD(HashEntry, CM_KEY_CONTROL_BLOCK, KeyHash) — KeyHash overlays
            // the __bindgen_anon_2 union, so the offset is the union's offset.
            CurrentKcb = (HashEntry as *mut u8)
                .offset(-(core::mem::offset_of!(_CM_KEY_CONTROL_BLOCK, __bindgen_anon_2) as isize))
                as PCM_KEY_CONTROL_BLOCK;

            if TotalLevels == (*CurrentKcb).__bindgen_anon_1.TotalLevels() {
                SubkeysMatch = CmpCompareSubkeys(
                    HashCacheStack,
                    CurrentKcb,
                    RemainingSubkeys as u32,
                    core::ptr::addr_of_mut!(ParentKcb),
                );

                if SubkeysMatch != 0 {
                    if *Kcb == ParentKcb {
                        if (*CurrentKcb).__bindgen_anon_1.Delete() != 0
                            || (*(*CurrentKcb).ParentKcb).__bindgen_anon_1.Delete() != 0
                        {
                            if LockKcbsExclusive == 0 {
                                CmpUnLockKcbArray(LockedKcbs);
                                CmpDereferenceKeyControlBlock(*Kcb);
                                crate::dbg_print!(
                                    b"The current KCB or its parent is deleted, retrying looking in the cache\n\0"
                                );
                                return STATUS_RETRY;
                            }

                            if (*CurrentKcb).__bindgen_anon_1.Delete() != 0 {
                                CmpRemoveKeyControlBlock(CurrentKcb);
                                CmpUnLockKcbArray(LockedKcbs);
                                CmpDereferenceKeyControlBlock(*Kcb);
                                crate::dbg_print!(
                                    b"The current KCB is deleted (KCB 0x%p)\n\0",
                                    CurrentKcb
                                );
                                return STATUS_OBJECT_NAME_NOT_FOUND;
                            }

                            if (*CurrentKcb).RefCount != 0 {
                                (*CurrentKcb).__bindgen_anon_1.set_Delete(1);
                                CmpRemoveKeyControlBlock(CurrentKcb);
                            } else {
                                CmpRemoveFromDelayedClose(CurrentKcb);
                                CmpCleanUpKcbCacheWithLock(CurrentKcb, 0u8);
                            }

                            break;
                        }

                        KeyFoundInCache = 1u8;

                        CmpRemoveSubkeysInRemainingName(HashCacheStack, RemainingSubkeys as u32, RemainingName);
                        break;
                    }
                }
            }

            HashEntry = (*HashEntry).NextHash;
        }

        if KeyFoundInCache != 0 {
            break;
        }

        RemainingSubkeys -= 1;
    }

    *MatchRemainSubkeyLevel = (RemainingSubkeys + 1) as u32;

    if KeyFoundInCache != 0 {
        /* Reference the new KCB BEFORE dropping the locks, and before switching
         * *Kcb to it. The old order dereferenced the caller's KCB and pointed *Kcb
         * at CurrentKcb first, so a failed reference handed the caller a KCB that
         * was never referenced while the one it did hold had already been released.
         * CmpLookInCache is on the registry-lookup fast path -- every boot-time key
         * resolution runs it. Upstream-Commit: e40c4d89cfb */
        if CmpReferenceKeyControlBlock(CurrentKcb) == 0 {
            crate::dbg_print!(
                b"Could not reference the KCB, too many references (KCB 0x%p)\n\0",
                CurrentKcb
            );
            /* Unlock and release the reference taken at entry, so the caller sees a
             * clean state rather than a half-switched one. */
            CmpUnLockKcbArray(LockedKcbs);
            CmpDereferenceKeyControlBlock(*Kcb);
            return STATUS_UNSUCCESSFUL;
        }

        /* Drop the locks before dereferencing: CmpDereferenceKeyControlBlock must
         * never run with a KCB lock held. */
        CmpUnLockKcbArray(LockedKcbs);
        CmpDereferenceKeyControlBlock(*Kcb);
        *Kcb = CurrentKcb;

        *Hive = (*CurrentKcb).__bindgen_anon_2.__bindgen_anon_1.KeyHive;
        *Cell = (*CurrentKcb).__bindgen_anon_2.__bindgen_anon_1.KeyCell;
        return STATUS_SUCCESS;
    }

    CmpUnLockKcbArray(LockedKcbs);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn CmpBuildHashStackAndLookupCache(
    ParseObject: PCM_KEY_BODY,
    Kcb: *mut PCM_KEY_CONTROL_BLOCK,
    Current: PUNICODE_STRING,
    Hive: *mut PHHIVE,
    Cell: PHCELL_INDEX,
    TotalRemainingSubkeys: *mut ULONG,
    MatchRemainSubkeyLevel: *mut ULONG,
    TotalSubkeys: *mut ULONG,
    OuterStackArray: *mut ULONG,
    LockedKcbs: *mut *mut ULONG,
) -> NTSTATUS {
    let ParseObject = opaque_ptr(ParseObject);
    let Kcb = opaque_ptr(Kcb);
    let Current = opaque_ptr(Current);
    let Hive = opaque_ptr(Hive);
    let Cell = opaque_ptr(Cell);
    let TotalRemainingSubkeys = opaque_ptr(TotalRemainingSubkeys);
    let MatchRemainSubkeyLevel = opaque_ptr(MatchRemainSubkeyLevel);
    let TotalSubkeys = opaque_ptr(TotalSubkeys);
    let OuterStackArray = opaque_ptr(OuterStackArray);
    let LockedKcbs = opaque_ptr(LockedKcbs);

    let mut Status: NTSTATUS;
    let mut ConvKey: ULONG;
    let mut SubkeysInTotal: ULONG = 0;
    let mut RemainingSubkeysInTotal: ULONG;
    let mut MatchRemainingSubkeys: ULONG = 0;
    let mut HashCacheStack: [CM_HASH_CACHE_STACK; CMP_SUBKEY_LEVELS_DEPTH_LIMIT] = core::mem::zeroed();

    CmpLockRegistry();

    ConvKey = (**Kcb).__bindgen_anon_2.__bindgen_anon_1.ConvKey;
    RemainingSubkeysInTotal = CmpComputeHashValue(
        Current,
        ConvKey,
        HashCacheStack.as_mut_ptr(),
        core::ptr::addr_of_mut!(SubkeysInTotal),
    );

    if SubkeysInTotal > CMP_SUBKEY_LEVELS_DEPTH_LIMIT as ULONG {
        crate::dbg_print!(
            b"The key path has too many subkeys - %lu\n\0",
            SubkeysInTotal
        );
        *LockedKcbs = core::ptr::null_mut();
        return 0xC0000106u32 as i32;
    }

    *Hive = (**Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyHive;
    *Cell = (**Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyCell;

    if RemainingSubkeysInTotal == 0 && SubkeysInTotal == 0 {
        if CmpReferenceKeyControlBlock(*Kcb) == 0 {
            crate::dbg_print!(
                b"Could not reference the KCB, too many references (KCB 0x%p)\n\0",
                Kcb
            );
            return 0xC0000001u32 as i32;
        }

        CmpAcquireKcbLockSharedByIndex(GET_HASH_INDEX(ConvKey));

        *OuterStackArray.offset(0) = 1;
        *OuterStackArray.offset(1) = GET_HASH_INDEX(ConvKey);
        *LockedKcbs = OuterStackArray;

        *TotalRemainingSubkeys = RemainingSubkeysInTotal;
        *MatchRemainSubkeyLevel = 0;
        *TotalSubkeys = SubkeysInTotal;
        return STATUS_SUCCESS;
    }

    Status = CmpLookInCache(
        HashCacheStack.as_mut_ptr(),
        0u8,
        RemainingSubkeysInTotal,
        Current,
        OuterStackArray,
        Kcb,
        Hive,
        Cell,
        core::ptr::addr_of_mut!(MatchRemainingSubkeys),
    );
    if !NT_SUCCESS(Status) {
        if Status != 0xC000022Du32 as i32 {
            crate::dbg_print!(b"CmpLookInCache() failed (Status 0x%lx)\n\0", Status);
            *LockedKcbs = core::ptr::null_mut();
            return Status;
        }

        Status = CmpLookInCache(
            HashCacheStack.as_mut_ptr(),
            1u8,
            RemainingSubkeysInTotal,
            Current,
            OuterStackArray,
            Kcb,
            Hive,
            Cell,
            core::ptr::addr_of_mut!(MatchRemainingSubkeys),
        );
        if !NT_SUCCESS(Status) {
            crate::dbg_print!(
                b"CmpLookInCache() failed after retry (Status 0x%lx)\n\0",
                Status
            );
            *LockedKcbs = core::ptr::null_mut();
            return Status;
        }
    }

    if RemainingSubkeysInTotal == MatchRemainingSubkeys {
        CmpAcquireKcbLockSharedByIndex(GET_HASH_INDEX((**Kcb).__bindgen_anon_2.__bindgen_anon_1.ConvKey));
        *OuterStackArray.offset(0) = 1;
        *OuterStackArray.offset(1) = GET_HASH_INDEX((**Kcb).__bindgen_anon_2.__bindgen_anon_1.ConvKey);
        *LockedKcbs = OuterStackArray;
    } else {
        *LockedKcbs = CmpBuildAndLockKcbArray(
            HashCacheStack.as_mut_ptr(),
            1,
            *Kcb,
            OuterStackArray,
            RemainingSubkeysInTotal,
            MatchRemainingSubkeys,
        );
        NT_ASSERT(if (*LockedKcbs).is_null() { 0u8 } else { 1u8 });
    }

    *TotalRemainingSubkeys = RemainingSubkeysInTotal;
    *MatchRemainSubkeyLevel = MatchRemainingSubkeys;
    *TotalSubkeys = SubkeysInTotal;
    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/config/cmparse.c
//  * PURPOSE:         Configuration Manager - Object Manager Parse Interface
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include "ntoskrnl.h"
// #define NDEBUG
// #include "debug.h"
//
// /* GLOBALS *******************************************************************/
//
// /* FUNCTIONS *****************************************************************/
//
// BOOLEAN
// NTAPI
// CmpGetNextName(IN OUT PUNICODE_STRING RemainingName,
//                OUT PUNICODE_STRING NextName,
//                OUT PBOOLEAN LastName)
// {
//     BOOLEAN NameValid = TRUE;
//
//     ASSERT(RemainingName->Length % sizeof(WCHAR) == 0);
//
//     /* Check if there's nothing left in the name */
//     if (!(RemainingName->Buffer) ||
//         (!RemainingName->Length) ||
//         !(*RemainingName->Buffer))
//     {
//         /* Clear the next name and set this as last */
//         *LastName = TRUE;
//         NextName->Buffer = NULL;
//         NextName->Length = 0;
//         return TRUE;
//     }
//
//     /* Check if we have a path separator */
//     while (RemainingName->Length &&
//            (*RemainingName->Buffer == OBJ_NAME_PATH_SEPARATOR))
//     {
//         /* Skip it */
//         RemainingName->Buffer++;
//         RemainingName->Length -= sizeof(WCHAR);
//         RemainingName->MaximumLength -= sizeof(WCHAR);
//     }
//
//     /* Start loop at where the current buffer is */
//     NextName->Buffer = RemainingName->Buffer;
//     while (RemainingName->Length &&
//            (*RemainingName->Buffer != OBJ_NAME_PATH_SEPARATOR))
//     {
//         /* Move to the next character */
//         RemainingName->Buffer++;
//         RemainingName->Length -= sizeof(WCHAR);
//         RemainingName->MaximumLength -= sizeof(WCHAR);
//     }
//
//     /* See how many chars we parsed and validate the length */
//     NextName->Length = (USHORT)((ULONG_PTR)RemainingName->Buffer -
//                                 (ULONG_PTR)NextName->Buffer);
//     if (NextName->Length > 512) NameValid = FALSE;
//     NextName->MaximumLength = NextName->Length;
//
//     /* If there's nothing left, we're last */
//     *LastName = !RemainingName->Length;
//     return NameValid;
// }
//
// BOOLEAN
// NTAPI
// CmpGetSymbolicLink(IN PHHIVE Hive,
//                    IN OUT PUNICODE_STRING ObjectName,
//                    IN OUT PCM_KEY_CONTROL_BLOCK SymbolicKcb,
//                    IN PUNICODE_STRING RemainingName OPTIONAL)
// {
//     HCELL_INDEX LinkCell = HCELL_NIL;
//     PCM_KEY_VALUE LinkValue = NULL;
//     PWSTR LinkName = NULL;
//     BOOLEAN LinkNameAllocated = FALSE;
//     PWSTR NewBuffer;
//     ULONG Length = 0;
//     ULONG ValueLength = 0;
//     BOOLEAN Result = FALSE;
//     HCELL_INDEX CellToRelease = HCELL_NIL;
//     PCM_KEY_NODE Node;
//     UNICODE_STRING NewObjectName;
//
//     /* Make sure we're not being deleted */
//     if (SymbolicKcb->Delete) return FALSE;
//
//     /* Get the key node */
//     Node = (PCM_KEY_NODE)HvGetCell(SymbolicKcb->KeyHive, SymbolicKcb->KeyCell);
//     if (!Node) goto Exit;
//
//     /* Find the symbolic link key */
//     LinkCell = CmpFindValueByName(Hive, Node, &CmSymbolicLinkValueName);
//     HvReleaseCell(SymbolicKcb->KeyHive, SymbolicKcb->KeyCell);
//     if (LinkCell == HCELL_NIL) goto Exit;
//
//     /* Get the value cell */
//     LinkValue = (PCM_KEY_VALUE)HvGetCell(Hive, LinkCell);
//     if (!LinkValue) goto Exit;
//
//     /* Make sure it's a registry link */
//     if (LinkValue->Type != REG_LINK) goto Exit;
//
//     /* Now read the value data */
//     if (!CmpGetValueData(Hive,
//                          LinkValue,
//                          &ValueLength,
//                          (PVOID*)&LinkName,
//                          &LinkNameAllocated,
//                          &CellToRelease))
//     {
//         /* Fail */
//         goto Exit;
//     }
//
//     /* Get the length */
//     Length = ValueLength + sizeof(WCHAR);
//
//     /* Make sure we start with a slash */
//     if (*LinkName != OBJ_NAME_PATH_SEPARATOR) goto Exit;
//
//     /* Add the remaining name if needed */
//     if (RemainingName) Length += RemainingName->Length + sizeof(WCHAR);
//
//     /* Check for overflow */
//     if (Length > 0xFFFF) goto Exit;
//
//     /* Check if we need a new buffer */
//     if (Length > ObjectName->MaximumLength)
//     {
//         /* We do -- allocate one */
//         NewBuffer = ExAllocatePoolWithTag(PagedPool, Length, TAG_CM);
//         if (!NewBuffer) goto Exit;
//
//         /* Setup the new string and copy the symbolic target */
//         NewObjectName.Buffer = NewBuffer;
//         NewObjectName.MaximumLength = (USHORT)Length;
//         NewObjectName.Length = (USHORT)ValueLength;
//         RtlCopyMemory(NewBuffer, LinkName, ValueLength);
//
//         /* Check if we need to add anything else */
//         if (RemainingName)
//         {
//             /* Add the remaining name */
//             NewBuffer[ValueLength / sizeof(WCHAR)] = OBJ_NAME_PATH_SEPARATOR;
//             NewObjectName.Length += sizeof(WCHAR);
//             RtlAppendUnicodeStringToString(&NewObjectName, RemainingName);
//         }
//
//         /* Free the old buffer */
//         ExFreePool(ObjectName->Buffer);
//         *ObjectName = NewObjectName;
//     }
//     else
//     {
//         /* The old name is large enough -- update the length */
//         ObjectName->Length = (USHORT)ValueLength;
//         if (RemainingName)
//         {
//             /* Copy the remaining name inside */
//             RtlMoveMemory(&ObjectName->Buffer[(ValueLength / sizeof(WCHAR)) + 1],
//                           RemainingName->Buffer,
//                           RemainingName->Length);
//
//             /* Add the slash and update the length */
//             ObjectName->Buffer[ValueLength / sizeof(WCHAR)] = OBJ_NAME_PATH_SEPARATOR;
//             ObjectName->Length += RemainingName->Length + sizeof(WCHAR);
//         }
//
//         /* Copy the symbolic link target name */
//         RtlCopyMemory(ObjectName->Buffer, LinkName, ValueLength);
//     }
//
//     /* Null-terminate the whole thing */
//     ObjectName->Buffer[ObjectName->Length / sizeof(WCHAR)] = UNICODE_NULL;
//     Result = TRUE;
//
// Exit:
//     /* Free the link name */
//     if (LinkNameAllocated) ExFreePool(LinkName);
//
//     /* Check if we had a value cell */
//     if (LinkValue)
//     {
//         /* Release it */
//         ASSERT(LinkCell != HCELL_NIL);
//         HvReleaseCell(Hive, LinkCell);
//     }
//
//     /* Check if we had an active cell and release it, then return the result */
//     if (CellToRelease != HCELL_NIL) HvReleaseCell(Hive, CellToRelease);
//     return Result;
// }
//
// NTSTATUS
// NTAPI
// CmpDoCreateChild(IN PHHIVE Hive,
//                  IN HCELL_INDEX ParentCell,
//                  IN PSECURITY_DESCRIPTOR ParentDescriptor OPTIONAL,
//                  IN PACCESS_STATE AccessState,
//                  IN PUNICODE_STRING Name,
//                  IN KPROCESSOR_MODE AccessMode,
//                  IN PCM_PARSE_CONTEXT ParseContext,
//                  IN PCM_KEY_CONTROL_BLOCK ParentKcb,
//                  IN ULONG Flags,
//                  OUT PHCELL_INDEX KeyCell,
//                  OUT PVOID *Object)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     PCM_KEY_BODY KeyBody;
//     HCELL_INDEX ClassCell = HCELL_NIL;
//     PCM_KEY_NODE KeyNode;
//     PCELL_DATA CellData;
//     ULONG StorageType;
//     PCM_KEY_CONTROL_BLOCK Kcb;
//     PSECURITY_DESCRIPTOR NewDescriptor;
//
//     /* Get the storage type */
//     StorageType = Stable;
//     if (ParseContext->CreateOptions & REG_OPTION_VOLATILE) StorageType = Volatile;
//
//     /* Allocate the child */
//     *KeyCell = HvAllocateCell(Hive,
//                               FIELD_OFFSET(CM_KEY_NODE, Name) +
//                               CmpNameSize(Hive, Name),
//                               StorageType,
//                               HCELL_NIL);
//     if (*KeyCell == HCELL_NIL)
//     {
//         /* Fail */
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto Quickie;
//     }
//
//     /* Get the key node */
//     KeyNode = (PCM_KEY_NODE)HvGetCell(Hive, *KeyCell);
//     if (!KeyNode)
//     {
//         /* Fail, this should never happen */
//         ASSERT(FALSE);
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto Quickie;
//     }
//
//     /* Release the cell */
//     HvReleaseCell(Hive, *KeyCell);
//
//     /* Check if we have a class name */
//     if (ParseContext->Class.Length > 0)
//     {
//         /* Allocate a class cell */
//         ClassCell = HvAllocateCell(Hive,
//                                    ParseContext->Class.Length,
//                                    StorageType,
//                                    HCELL_NIL);
//         if (ClassCell == HCELL_NIL)
//         {
//             /* Fail */
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto Quickie;
//         }
//     }
//
//     /* Allocate the Cm Object */
//     Status = ObCreateObject(AccessMode,
//                             CmpKeyObjectType,
//                             NULL,
//                             AccessMode,
//                             NULL,
//                             sizeof(CM_KEY_BODY),
//                             0,
//                             0,
//                             Object);
//     if (!NT_SUCCESS(Status)) goto Quickie;
//
//     /* Setup the key body */
//     KeyBody = (PCM_KEY_BODY)(*Object);
//     KeyBody->Type = CM_KEY_BODY_TYPE;
//     KeyBody->KeyControlBlock = NULL;
//     KeyBody->KcbLocked = FALSE;
//
//     /* Check if we had a class */
//     if (ParseContext->Class.Length > 0)
//     {
//         /* Get the class cell */
//         CellData = HvGetCell(Hive, ClassCell);
//         if (!CellData)
//         {
//             /* Fail, this should never happen */
//             ASSERT(FALSE);
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             ObDereferenceObject(*Object);
//             goto Quickie;
//         }
//
//         /* Release the cell */
//         HvReleaseCell(Hive, ClassCell);
//
//         /* Copy the class data */
//         RtlCopyMemory(&CellData->u.KeyString[0],
//                       ParseContext->Class.Buffer,
//                       ParseContext->Class.Length);
//     }
//
//     /* Fill out the key node */
//     KeyNode->Signature = CM_KEY_NODE_SIGNATURE;
//     KeyNode->Flags = Flags;
//     KeQuerySystemTime(&KeyNode->LastWriteTime);
//     KeyNode->Spare = 0;
//     KeyNode->Parent = ParentCell;
//     KeyNode->SubKeyCounts[Stable] = 0;
//     KeyNode->SubKeyCounts[Volatile] = 0;
//     KeyNode->SubKeyLists[Stable] = HCELL_NIL;
//     KeyNode->SubKeyLists[Volatile] = HCELL_NIL;
//     KeyNode->ValueList.Count = 0;
//     KeyNode->ValueList.List = HCELL_NIL;
//     KeyNode->Security = HCELL_NIL;
//     KeyNode->Class = ClassCell;
//     KeyNode->ClassLength = ParseContext->Class.Length;
//     KeyNode->MaxValueDataLen = 0;
//     KeyNode->MaxNameLen = 0;
//     KeyNode->MaxValueNameLen = 0;
//     KeyNode->MaxClassLen = 0;
//     KeyNode->NameLength = CmpCopyName(Hive, KeyNode->Name, Name);
//     if (KeyNode->NameLength < Name->Length) KeyNode->Flags |= KEY_COMP_NAME;
//
//     /* Create the KCB */
//     Kcb = CmpCreateKeyControlBlock(Hive,
//                                    *KeyCell,
//                                    KeyNode,
//                                    ParentKcb,
//                                    CMP_LOCK_HASHES_FOR_KCB,
//                                    Name);
//     if (!Kcb)
//     {
//         /* Fail */
//         ObDereferenceObjectDeferDelete(*Object);
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto Quickie;
//     }
//
//     /* Sanity check */
//     ASSERT(Kcb->RefCount == 1);
//
//     /* Now fill out the Cm object */
//     KeyBody->NotifyBlock = NULL;
//     KeyBody->ProcessID = PsGetCurrentProcessId();
//     KeyBody->KeyControlBlock = Kcb;
//
//     /* Link it with the KCB */
//     EnlistKeyBodyWithKCB(KeyBody, CMP_ENLIST_KCB_LOCKED_EXCLUSIVE);
//
//     /* Assign security */
//     Status = SeAssignSecurity(ParentDescriptor,
//                               AccessState->SecurityDescriptor,
//                               &NewDescriptor,
//                               TRUE,
//                               &AccessState->SubjectSecurityContext,
//                               &CmpKeyObjectType->TypeInfo.GenericMapping,
//                               CmpKeyObjectType->TypeInfo.PoolType);
//     if (NT_SUCCESS(Status))
//     {
//         /*
//          * FIXME: We must acquire a security lock when assigning
//          * a security descriptor to this hive but since the
//          * CmpAssignSecurityDescriptor function does nothing
//          * (we lack the necessary security management implementations
//          * anyway), do not do anything for now.
//          */
//         Status = CmpAssignSecurityDescriptor(Kcb, NewDescriptor);
//     }
//
//     /* Now that the security descriptor is copied in the hive, we can free the original */
//     SeDeassignSecurity(&NewDescriptor);
//
//     if (NT_SUCCESS(Status))
//     {
//         /* Send notification to registered callbacks */
//         CmpReportNotify(Kcb, Hive, Kcb->KeyCell, REG_NOTIFY_CHANGE_NAME);
//     }
//
// Quickie:
//     /* Check if we got here because of failure */
//     if (!NT_SUCCESS(Status))
//     {
//         /* Free any cells we might've allocated */
//         if (ParseContext->Class.Length > 0) HvFreeCell(Hive, ClassCell);
//         HvFreeCell(Hive, *KeyCell);
//     }
//
//     /* Return status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// CmpDoCreate(IN PHHIVE Hive,
//             IN HCELL_INDEX Cell,
//             IN PACCESS_STATE AccessState,
//             IN PUNICODE_STRING Name,
//             IN KPROCESSOR_MODE AccessMode,
//             IN PCM_PARSE_CONTEXT ParseContext,
//             IN PCM_KEY_CONTROL_BLOCK ParentKcb,
//             OUT PVOID *Object)
// {
//     NTSTATUS Status;
//     PCELL_DATA CellData;
//     HCELL_INDEX KeyCell;
//     ULONG ParentType;
//     PCM_KEY_BODY KeyBody;
//     PSECURITY_DESCRIPTOR SecurityDescriptor = NULL;
//     LARGE_INTEGER TimeStamp;
//     PCM_KEY_NODE KeyNode;
//
//     /* Make sure the KCB is locked and lock the flusher */
//     CMP_ASSERT_KCB_LOCK(ParentKcb);
//     CmpLockHiveFlusherShared((PCMHIVE)Hive);
//
//     /* Bail out on read-only KCBs */
//     if (ParentKcb->ExtFlags & CM_KCB_READ_ONLY_KEY)
//     {
//         Status = STATUS_ACCESS_DENIED;
//         goto Exit;
//     }
//
//     /* Check if the parent is being deleted */
//     if (ParentKcb->Delete)
//     {
//         /* It has, quit */
//         ASSERT(FALSE);
//         Status = STATUS_OBJECT_NAME_NOT_FOUND;
//         goto Exit;
//     }
//
//     /* Get the parent node */
//     KeyNode = (PCM_KEY_NODE)HvGetCell(Hive, Cell);
//     if (!KeyNode)
//     {
//         /* Fail */
//         ASSERT(FALSE);
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto Exit;
//     }
//
//     /* Make sure nobody added us yet */
//     if (CmpFindSubKeyByName(Hive, KeyNode, Name) != HCELL_NIL)
//     {
//         /* Fail */
//         ASSERT(FALSE);
//         Status = STATUS_REPARSE;
//         goto Exit;
//     }
//
//     /* Sanity check */
//     ASSERT(Cell == ParentKcb->KeyCell);
//
//     /* Get the parent type */
//     ParentType = HvGetCellType(Cell);
//     if ((ParentType == Volatile) &&
//         !(ParseContext->CreateOptions & REG_OPTION_VOLATILE))
//     {
//         /* Children of volatile parents must also be volatile */
//         //ASSERT(FALSE);
//         Status = STATUS_CHILD_MUST_BE_VOLATILE;
//         goto Exit;
//     }
//
//     /* Don't allow children under symlinks */
//     if (ParentKcb->Flags & KEY_SYM_LINK)
//     {
//         /* Fail */
//         ASSERT(FALSE);
//         Status = STATUS_ACCESS_DENIED;
//         goto Exit;
//     }
//
//     /* Make the cell dirty for now */
//     HvMarkCellDirty(Hive, Cell, FALSE);
//
//     /* Do the actual create operation */
//     Status = CmpDoCreateChild(Hive,
//                               Cell,
//                               SecurityDescriptor,
//                               AccessState,
//                               Name,
//                               AccessMode,
//                               ParseContext,
//                               ParentKcb,
//                               0,
//                               &KeyCell,
//                               Object);
//     if (NT_SUCCESS(Status))
//     {
//         /* Get the key body */
//         KeyBody = (PCM_KEY_BODY)(*Object);
//
//         /* Now add the subkey */
//         if (!CmpAddSubKey(Hive, Cell, KeyCell))
//         {
//             /* Free the created child */
//             CmpFreeKeyByCell(Hive, KeyCell, FALSE);
//
//             /* Purge out this KCB */
//             KeyBody->KeyControlBlock->Delete = TRUE;
//             CmpRemoveKeyControlBlock(KeyBody->KeyControlBlock);
//
//             /* And cleanup the key body object */
//             ObDereferenceObjectDeferDelete(*Object);
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto Exit;
//         }
//
//         /* Get the key node */
//         KeyNode = (PCM_KEY_NODE)HvGetCell(Hive, Cell);
//         if (!KeyNode)
//         {
//             /* Fail, this shouldn't happen */
//             CmpFreeKeyByCell(Hive, KeyCell, TRUE); // Subkey linked above
//
//             /* Purge out this KCB */
//             KeyBody->KeyControlBlock->Delete = TRUE;
//             CmpRemoveKeyControlBlock(KeyBody->KeyControlBlock);
//
//             /* And cleanup the key body object */
//             ObDereferenceObjectDeferDelete(*Object);
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto Exit;
//         }
//
//         /* Clean up information on this subkey */
//         CmpCleanUpSubKeyInfo(KeyBody->KeyControlBlock->ParentKcb);
//
//         /* Sanity checks */
//         ASSERT(KeyBody->KeyControlBlock->ParentKcb->KeyCell == Cell);
//         ASSERT(KeyBody->KeyControlBlock->ParentKcb->KeyHive == Hive);
//         ASSERT(KeyBody->KeyControlBlock->ParentKcb == ParentKcb);
//         ASSERT(KeyBody->KeyControlBlock->ParentKcb->KcbMaxNameLen == KeyNode->MaxNameLen);
//
//         /* Update the timestamp */
//         KeQuerySystemTime(&TimeStamp);
//         KeyNode->LastWriteTime = TimeStamp;
//         KeyBody->KeyControlBlock->ParentKcb->KcbLastWriteTime = TimeStamp;
//
//         /* Check if we need to update name maximum */
//         if (KeyNode->MaxNameLen < Name->Length)
//         {
//             /* Do it */
//             KeyNode->MaxNameLen = Name->Length;
//             KeyBody->KeyControlBlock->ParentKcb->KcbMaxNameLen = Name->Length;
//         }
//
//         /* Check if we need to update class length maximum */
//         if (KeyNode->MaxClassLen < ParseContext->Class.Length)
//         {
//             /* Update it */
//             KeyNode->MaxClassLen = ParseContext->Class.Length;
//         }
//
//         /* Check if we're creating a symbolic link */
//         if (ParseContext->CreateOptions & REG_OPTION_CREATE_LINK)
//         {
//             /* Get the cell data */
//             CellData = HvGetCell(Hive, KeyCell);
//             if (!CellData)
//             {
//                 /* This shouldn't happen */
//                 CmpFreeKeyByCell(Hive, KeyCell, TRUE); // Subkey linked above
//
//                 /* Purge out this KCB */
//                 KeyBody->KeyControlBlock->Delete = TRUE;
//                 CmpRemoveKeyControlBlock(KeyBody->KeyControlBlock);
//
//                 /* And cleanup the key body object */
//                 ObDereferenceObjectDeferDelete(*Object);
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 goto Exit;
//             }
//
//             /* Update the flags */
//             CellData->u.KeyNode.Flags |= KEY_SYM_LINK;
//             KeyBody->KeyControlBlock->Flags = CellData->u.KeyNode.Flags;
//             HvReleaseCell(Hive, KeyCell);
//         }
//     }
//
// Exit:
//     /* Release the flusher lock and return status */
//     CmpUnlockHiveFlusher((PCMHIVE)Hive);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// CmpDoOpen(IN PHHIVE Hive,
//           IN HCELL_INDEX Cell,
//           IN PCM_KEY_NODE Node,
//           IN PACCESS_STATE AccessState,
//           IN KPROCESSOR_MODE AccessMode,
//           IN ULONG Attributes,
//           IN PCM_PARSE_CONTEXT Context OPTIONAL,
//           IN ULONG ControlFlags,
//           IN OUT PCM_KEY_CONTROL_BLOCK *CachedKcb,
//           IN PULONG KcbsLocked,
//           IN PUNICODE_STRING KeyName,
//           OUT PVOID *Object)
// {
//     NTSTATUS Status;
//     BOOLEAN LockKcb = FALSE;
//     BOOLEAN IsLockShared = FALSE;
//     PCM_KEY_BODY KeyBody = NULL;
//     PCM_KEY_CONTROL_BLOCK Kcb = NULL;
//
//     /* Make sure the hive isn't locked */
//     if ((Hive->HiveFlags & HIVE_IS_UNLOADING) &&
//         (((PCMHIVE)Hive)->CreatorOwner != KeGetCurrentThread()))
//     {
//         /* It is, don't touch it */
//         return STATUS_OBJECT_NAME_NOT_FOUND;
//     }
//
//     /* Check if we have a context */
//     if (Context)
//     {
//         /* Check if this is a link create (which shouldn't be an open) */
//         if (Context->CreateLink)
//         {
//             return STATUS_ACCESS_DENIED;
//         }
//
//         /* Check if this is symlink create attempt */
//         if (Context->CreateOptions & REG_OPTION_CREATE_LINK)
//         {
//             /* Key already exists */
//             return STATUS_OBJECT_NAME_COLLISION;
//         }
//
//         /* Set the disposition */
//         Context->Disposition = REG_OPENED_EXISTING_KEY;
//     }
//
//     /* Lock the KCB on creation if asked */
//     if (ControlFlags & CMP_CREATE_KCB_KCB_LOCKED)
//     {
//         LockKcb = TRUE;
//     }
//
//     /* Check if caller doesn't want to create a KCB */
//     if (ControlFlags & CMP_OPEN_KCB_NO_CREATE)
//     {
//         /*
//          * The caller doesn't want to create a KCB. This means the KCB
//          * is already in cache and other threads may take use of it
//          * so it has to be locked in share mode.
//          */
//         IsLockShared = TRUE;
//
//         /* Check if this is a symlink */
//         if (((*CachedKcb)->Flags & KEY_SYM_LINK) && !(Attributes & OBJ_OPENLINK))
//         {
//             /* Is this symlink found? */
//             if ((*CachedKcb)->ExtFlags & CM_KCB_SYM_LINK_FOUND)
//             {
//                 /* Get the real KCB, is this deleted? */
//                 Kcb = (*CachedKcb)->ValueCache.RealKcb;
//                 if (Kcb->Delete)
//                 {
//                     /*
//                      * The real KCB is gone, do a reparse. We used to lock the KCB in
//                      * shared mode as others may have taken use of it but since we
//                      * must do a reparse of the key the only thing that matter is us.
//                      * Lock the KCB exclusively so nobody is going to mess with the KCB.
//                      */
//                     DPRINT1("The real KCB is deleted, attempt a reparse\n");
//                     CmpUnLockKcbArray(KcbsLocked);
//                     CmpAcquireKcbLockExclusiveByIndex(GET_HASH_INDEX((*CachedKcb)->ConvKey));
//                     CmpCleanUpKcbValueCache(*CachedKcb);
//                     KcbsLocked[0] = 1;
//                     KcbsLocked[1] = GET_HASH_INDEX((*CachedKcb)->ConvKey);
//                     return STATUS_REPARSE;
//                 }
//
//                 /*
//                  * The symlink has been found. As in the similar case above,
//                  * the KCB of the symlink exclusively, we don't want anybody
//                  * to mess it up.
//                  */
//                 CmpUnLockKcbArray(KcbsLocked);
//                 CmpAcquireKcbLockExclusiveByIndex(GET_HASH_INDEX((*CachedKcb)->ConvKey));
//                 KcbsLocked[0] = 1;
//                 KcbsLocked[1] = GET_HASH_INDEX((*CachedKcb)->ConvKey);
//             }
//             else
//             {
//                 /* We must do a reparse */
//                 DPRINT("The symlink is not found, attempt a reparse\n");
//                 return STATUS_REPARSE;
//             }
//         }
//         else
//         {
//             /* This is not a symlink, just give the cached KCB already */
//             Kcb = *CachedKcb;
//         }
//
//         /* The caller wants to open a cached KCB */
//         if (!CmpReferenceKeyControlBlock(Kcb))
//         {
//             /* Return failure code */
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//     }
//     else
//     {
//         /*
//          * The caller wants to create a new KCB. Unlike the code path above, here
//          * we must check if the lock is exclusively held because in the scenario
//          * where the caller doesn't want to create a KCB is because it is already
//          * in the cache and it must have a shared lock instead.
//          */
//         ASSERT(CmpIsKcbLockedExclusive(*CachedKcb));
//
//         /* Check if this is a symlink */
//         if ((Node->Flags & KEY_SYM_LINK) && !(Attributes & OBJ_OPENLINK))
//         {
//             /* Create the KCB for the symlink */
//             Kcb = CmpCreateKeyControlBlock(Hive,
//                                            Cell,
//                                            Node,
//                                            *CachedKcb,
//                                            LockKcb ? CMP_LOCK_HASHES_FOR_KCB : 0,
//                                            KeyName);
//             if (!Kcb)
//             {
//                 /* Return failure */
//                 return STATUS_INSUFFICIENT_RESOURCES;
//             }
//
//             /* Make sure it's also locked, and set the pointer */
//             ASSERT(CmpIsKcbLockedExclusive(Kcb));
//             *CachedKcb = Kcb;
//
//             /* Return reparse required */
//             return STATUS_REPARSE;
//         }
//
//         /* Create the KCB */
//         Kcb = CmpCreateKeyControlBlock(Hive,
//                                        Cell,
//                                        Node,
//                                        *CachedKcb,
//                                        LockKcb ? CMP_LOCK_HASHES_FOR_KCB : 0,
//                                        KeyName);
//         if (!Kcb)
//         {
//             /* Return failure */
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         /* Make sure it's also locked, and set the pointer */
//         ASSERT(CmpIsKcbLockedExclusive(Kcb));
//         *CachedKcb = Kcb;
//     }
//
//     /* Allocate the key object */
//     Status = ObCreateObject(AccessMode,
//                             CmpKeyObjectType,
//                             NULL,
//                             AccessMode,
//                             NULL,
//                             sizeof(CM_KEY_BODY),
//                             0,
//                             0,
//                             Object);
//     if (NT_SUCCESS(Status))
//     {
//         /* Get the key body and fill it out */
//         KeyBody = (PCM_KEY_BODY)(*Object);
//         KeyBody->KeyControlBlock = Kcb;
//         KeyBody->Type = CM_KEY_BODY_TYPE;
//         KeyBody->ProcessID = PsGetCurrentProcessId();
//         KeyBody->NotifyBlock = NULL;
//
//         /* Link to the KCB */
//         EnlistKeyBodyWithKCB(KeyBody, IsLockShared ? CMP_ENLIST_KCB_LOCKED_SHARED : CMP_ENLIST_KCB_LOCKED_EXCLUSIVE);
//
//         /*
//          * We are already holding a lock against the KCB that is assigned
//          * to this key body. This is to prevent a potential deadlock on
//          * CmpSecurityMethod as ObCheckObjectAccess will invoke the Object
//          * Manager to call that method, of which CmpSecurityMethod would
//          * attempt to acquire a lock again.
//          */
//         KeyBody->KcbLocked = TRUE;
//
//         if (!ObCheckObjectAccess(*Object,
//                                  AccessState,
//                                  FALSE,
//                                  AccessMode,
//                                  &Status))
//         {
//             /* Access check failed */
//             ObDereferenceObject(*Object);
//         }
//
//         /*
//          * We are done, the lock we are holding will be released
//          * once the registry parsing is done.
//          */
//         KeyBody->KcbLocked = FALSE;
//     }
//     else
//     {
//         /* Failed, dereference the KCB */
//         CmpDereferenceKeyControlBlockWithLock(Kcb, FALSE);
//     }
//
//     /* Return status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// CmpCreateLinkNode(IN PHHIVE Hive,
//                   IN HCELL_INDEX Cell,
//                   IN PACCESS_STATE AccessState,
//                   IN UNICODE_STRING Name,
//                   IN KPROCESSOR_MODE AccessMode,
//                   IN ULONG CreateOptions,
//                   IN PCM_PARSE_CONTEXT Context,
//                   IN PCM_KEY_CONTROL_BLOCK ParentKcb,
//                   IN PULONG KcbsLocked,
//                   OUT PVOID *Object)
// {
//     NTSTATUS Status;
//     HCELL_INDEX KeyCell, LinkCell, ChildCell;
//     PCM_KEY_BODY KeyBody;
//     LARGE_INTEGER TimeStamp;
//     PCM_KEY_NODE KeyNode;
//     PCM_KEY_CONTROL_BLOCK Kcb = ParentKcb;
//
//     /* Link nodes only allowed on the master */
//     if (Hive != &CmiVolatileHive->Hive)
//     {
//         /* Fail */
//         DPRINT1("Invalid link node attempt\n");
//         return STATUS_ACCESS_DENIED;
//     }
//
//     /* Make sure the KCB is locked and lock the flusher */
//     CMP_ASSERT_KCB_LOCK(ParentKcb);
//     CmpLockHiveFlusherShared((PCMHIVE)Hive);
//     CmpLockHiveFlusherShared((PCMHIVE)Context->ChildHive.KeyHive);
//
//     /* Bail out on read-only KCBs */
//     if (ParentKcb->ExtFlags & CM_KCB_READ_ONLY_KEY)
//     {
//         Status = STATUS_ACCESS_DENIED;
//         goto Exit;
//     }
//
//     /* Check if the parent is being deleted */
//     if (ParentKcb->Delete)
//     {
//         /* It is, quit */
//         ASSERT(FALSE);
//         Status = STATUS_OBJECT_NAME_NOT_FOUND;
//         goto Exit;
//     }
//
//     /* Allocate a link node */
//     LinkCell = HvAllocateCell(Hive,
//                               FIELD_OFFSET(CM_KEY_NODE, Name) +
//                               CmpNameSize(Hive, &Name),
//                               Stable,
//                               HCELL_NIL);
//     if (LinkCell == HCELL_NIL)
//     {
//         /* Fail */
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto Exit;
//     }
//
//     /* Get the key cell */
//     KeyCell = Context->ChildHive.KeyCell;
//     if (KeyCell != HCELL_NIL)
//     {
//         /* Hive exists! */
//         ChildCell = KeyCell;
//
//         /* Get the node data */
//         KeyNode = (PCM_KEY_NODE)HvGetCell(Context->ChildHive.KeyHive, ChildCell);
//         if (!KeyNode)
//         {
//             /* Fail */
//             ASSERT(FALSE);
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto Exit;
//         }
//
//         /* Fill out the data */
//         KeyNode->Parent = LinkCell;
//         KeyNode->Flags |= KEY_HIVE_ENTRY | KEY_NO_DELETE;
//         HvReleaseCell(Context->ChildHive.KeyHive, ChildCell);
//
//         /* Now open the key cell */
//         KeyNode = (PCM_KEY_NODE)HvGetCell(Context->ChildHive.KeyHive, KeyCell);
//         if (!KeyNode)
//         {
//             /* Fail */
//             ASSERT(FALSE);
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto Exit;
//         }
//
//         /* Open the parent */
//         Status = CmpDoOpen(Context->ChildHive.KeyHive,
//                            KeyCell,
//                            KeyNode,
//                            AccessState,
//                            AccessMode,
//                            CreateOptions,
//                            NULL,
//                            CMP_CREATE_KCB_KCB_LOCKED,
//                            &Kcb,
//                            KcbsLocked,
//                            &Name,
//                            Object);
//         HvReleaseCell(Context->ChildHive.KeyHive, KeyCell);
//     }
//     else
//     {
//         /* Do the actual create operation */
//         Status = CmpDoCreateChild(Context->ChildHive.KeyHive,
//                                   Cell,
//                                   NULL,
//                                   AccessState,
//                                   &Name,
//                                   AccessMode,
//                                   Context,
//                                   ParentKcb,
//                                   KEY_HIVE_ENTRY | KEY_NO_DELETE,
//                                   &ChildCell,
//                                   Object);
//         if (NT_SUCCESS(Status))
//         {
//             /* Setup root pointer */
//             Context->ChildHive.KeyHive->BaseBlock->RootCell = ChildCell;
//         }
//     }
//
//     /* Check if open or create suceeded */
//     if (NT_SUCCESS(Status))
//     {
//         /* Mark the cell dirty */
//         HvMarkCellDirty(Context->ChildHive.KeyHive, ChildCell, FALSE);
//
//         /* Get the key node */
//         KeyNode = (PCM_KEY_NODE)HvGetCell(Context->ChildHive.KeyHive, ChildCell);
//         if (!KeyNode)
//         {
//             /* Fail */
//             ASSERT(FALSE);
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto Exit;
//         }
//
//         /* Release it */
//         HvReleaseCell(Context->ChildHive.KeyHive, ChildCell);
//
//         /* Set the parent and flags */
//         KeyNode->Parent = LinkCell;
//         KeyNode->Flags |= KEY_HIVE_ENTRY | KEY_NO_DELETE;
//
//         /* Get the link node */
//         KeyNode = (PCM_KEY_NODE)HvGetCell(Hive, LinkCell);
//         if (!KeyNode)
//         {
//             /* Fail */
//             ASSERT(FALSE);
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto Exit;
//         }
//
//         /* Set it up */
//         KeyNode->Signature = CM_LINK_NODE_SIGNATURE;
//         KeyNode->Flags = KEY_HIVE_EXIT | KEY_NO_DELETE;
//         KeyNode->Parent = Cell;
//         KeyNode->NameLength = CmpCopyName(Hive, KeyNode->Name, &Name);
//         if (KeyNode->NameLength < Name.Length) KeyNode->Flags |= KEY_COMP_NAME;
//         KeQuerySystemTime(&TimeStamp);
//         KeyNode->LastWriteTime = TimeStamp;
//
//         /* Clear out the rest */
//         KeyNode->SubKeyCounts[Stable] = 0;
//         KeyNode->SubKeyCounts[Volatile] = 0;
//         KeyNode->SubKeyLists[Stable] = HCELL_NIL;
//         KeyNode->SubKeyLists[Volatile] = HCELL_NIL;
//         KeyNode->ValueList.Count = 0;
//         KeyNode->ValueList.List = HCELL_NIL;
//         KeyNode->ClassLength = 0;
//
//         /* Reference the root node */
//         KeyNode->ChildHiveReference.KeyHive = Context->ChildHive.KeyHive;
//         KeyNode->ChildHiveReference.KeyCell = ChildCell;
//         HvReleaseCell(Hive, LinkCell);
//
//         /* Get the parent node */
//         KeyNode = (PCM_KEY_NODE)HvGetCell(Hive, Cell);
//         if (!KeyNode)
//         {
//             /* Fail */
//             ASSERT(FALSE);
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto Exit;
//         }
//
//         /* Now add the subkey */
//         if (!CmpAddSubKey(Hive, Cell, LinkCell))
//         {
//             /* Failure! We don't handle this yet! */
//             ASSERT(FALSE);
//         }
//
//         /* Get the key body */
//         KeyBody = (PCM_KEY_BODY)*Object;
//
//         /* Clean up information on this subkey */
//         CmpCleanUpSubKeyInfo(KeyBody->KeyControlBlock->ParentKcb);
//
//         /* Sanity checks */
//         ASSERT(KeyBody->KeyControlBlock->ParentKcb->KeyCell == Cell);
//         ASSERT(KeyBody->KeyControlBlock->ParentKcb->KeyHive == Hive);
//         ASSERT(KeyBody->KeyControlBlock->ParentKcb->KcbMaxNameLen == KeyNode->MaxNameLen);
//
//         /* Update the timestamp */
//         KeQuerySystemTime(&TimeStamp);
//         KeyNode->LastWriteTime = TimeStamp;
//         KeyBody->KeyControlBlock->ParentKcb->KcbLastWriteTime = TimeStamp;
//
//         /* Check if we need to update name maximum */
//         if (KeyNode->MaxNameLen < Name.Length)
//         {
//             /* Do it */
//             KeyNode->MaxNameLen = Name.Length;
//             KeyBody->KeyControlBlock->ParentKcb->KcbMaxNameLen = Name.Length;
//         }
//
//         /* Check if we need to update class length maximum */
//         if (KeyNode->MaxClassLen < Context->Class.Length)
//         {
//             /* Update it */
//             KeyNode->MaxClassLen = Context->Class.Length;
//         }
//
//         /* Release the cell */
//         HvReleaseCell(Hive, Cell);
//     }
//     else
//     {
//         /* Release the link cell */
//         HvReleaseCell(Hive, LinkCell);
//     }
//
// Exit:
//     /* Release the flusher locks and return status */
//     CmpUnlockHiveFlusher((PCMHIVE)Context->ChildHive.KeyHive);
//     CmpUnlockHiveFlusher((PCMHIVE)Hive);
//     return Status;
// }
//
// VOID
// NTAPI
// CmpHandleExitNode(IN OUT PHHIVE *Hive,
//                   IN OUT HCELL_INDEX *Cell,
//                   IN OUT PCM_KEY_NODE *KeyNode,
//                   IN OUT PHHIVE *ReleaseHive,
//                   IN OUT HCELL_INDEX *ReleaseCell)
// {
//     /* Check if we have anything to release */
//     if (*ReleaseCell != HCELL_NIL)
//     {
//         /* Release it */
//         ASSERT(*ReleaseHive != NULL);
//         HvReleaseCell(*ReleaseHive, *ReleaseCell);
//     }
//
//     /* Get the link references */
//     *Hive = (*KeyNode)->ChildHiveReference.KeyHive;
//     *Cell = (*KeyNode)->ChildHiveReference.KeyCell;
//
//     /* Get the new node */
//     *KeyNode = (PCM_KEY_NODE)HvGetCell(*Hive, *Cell);
//     if (*KeyNode)
//     {
//         /* Set the new release values */
//         *ReleaseCell = *Cell;
//         *ReleaseHive = *Hive;
//     }
//     else
//     {
//         /* Nothing to release */
//         *ReleaseCell = HCELL_NIL;
//         *ReleaseHive = NULL;
//     }
// }
//
// /**
//  * @brief
//  * Computes the hashes of each subkey in key path name
//  * and stores them in a hash stack for cache lookup.
//  *
//  * @param[in] RemainingName
//  * A Unicode string structure consisting of the remaining
//  * registry key path name.
//  *
//  * @param[in] ConvKey
//  * The hash convkey of the current KCB to be supplied.
//  *
//  * @param[in,out] HashCacheStack
//  * An array stack. This function uses this array to store
//  * all the computed hashes of a key pathname.
//  *
//  * @param[out] TotalSubKeys
//  * The number of total subkeys that have been found, returned
//  * by this function to the caller. If no subkey levels are found
//  * the function returns 0.
//  *
//  * @return
//  * Returns the number of remaining subkey levels to caller.
//  * If no subkey levels are found then this function returns 0.
//  */
// static
// ULONG
// CmpComputeHashValue(
//     _In_ PUNICODE_STRING RemainingName,
//     _In_ ULONG ConvKey,
//     _Inout_ PCM_HASH_CACHE_STACK HashCacheStack,
//     _Out_ PULONG TotalSubKeys)
// {
//     ULONG CopyConvKey;
//     ULONG SubkeysInTotal;
//     ULONG RemainingSubkeysInTotal;
//     PWCHAR RemainingNameBuffer;
//     USHORT RemainingNameLength;
//     USHORT KeyNameLength;
//
//     /* Don't compute the hashes on a NULL remaining name */
//     RemainingNameBuffer = RemainingName->Buffer;
//     RemainingNameLength = RemainingName->Length;
//     if (RemainingNameLength == 0)
//     {
//         *TotalSubKeys = 0;
//         return 0;
//     }
//
//     /* Skip any leading separator */
//     while (RemainingNameLength >= sizeof(WCHAR) &&
//            *RemainingNameBuffer == OBJ_NAME_PATH_SEPARATOR)
//     {
//         RemainingNameBuffer++;
//         RemainingNameLength -= sizeof(WCHAR);
//     }
//
//     /* Now set up the hash stack entries and compute the hashes */
//     SubkeysInTotal = 0;
//     RemainingSubkeysInTotal = 0;
//     KeyNameLength = 0;
//     CopyConvKey = ConvKey;
//     HashCacheStack[RemainingSubkeysInTotal].NameOfKey.Buffer = RemainingNameBuffer;
//     while (RemainingNameLength > 0)
//     {
//         /* Is this character a separator? */
//         if (*RemainingNameBuffer != OBJ_NAME_PATH_SEPARATOR)
//         {
//             /* It's not, add it to the hash */
//             CopyConvKey = COMPUTE_HASH_CHAR(CopyConvKey, *RemainingNameBuffer);
//
//             /* Go to the next character (add up the length of the character as well) */
//             RemainingNameBuffer++;
//             KeyNameLength += sizeof(WCHAR);
//             RemainingNameLength -= sizeof(WCHAR);
//
//             /*
//              * We are at the end of the key name path. Take into account
//              * the last character and if we still have space in the hash
//              * stack, add it up in the remaining list.
//              */
//             if (RemainingNameLength == 0)
//             {
//                 if (RemainingSubkeysInTotal < CMP_SUBKEY_LEVELS_DEPTH_LIMIT)
//                 {
//                     HashCacheStack[RemainingSubkeysInTotal].NameOfKey.Length = KeyNameLength;
//                     HashCacheStack[RemainingSubkeysInTotal].NameOfKey.MaximumLength = KeyNameLength;
//                     HashCacheStack[RemainingSubkeysInTotal].ConvKey = CopyConvKey;
//                     RemainingSubkeysInTotal++;
//                 }
//
//                 SubkeysInTotal++;
//             }
//         }
//         else
//         {
//             /* Skip any leading separator */
//             while (RemainingNameLength >= sizeof(WCHAR) &&
//                    *RemainingNameBuffer == OBJ_NAME_PATH_SEPARATOR)
//             {
//                 RemainingNameBuffer++;
//                 RemainingNameLength -= sizeof(WCHAR);
//             }
//
//             /*
//              * It would be possible that a malformed key pathname may be passed
//              * to the registry parser such as a path with only separators like
//              * "\\\\" for example. This would trick the function into believing
//              * the key path has subkeys albeit that is not the case.
//              */
//             ASSERT(RemainingNameLength != 0);
//
//             /* Take into account this subkey */
//             SubkeysInTotal++;
//
//             /* And add it up to the hash stack */
//             if (RemainingSubkeysInTotal < CMP_SUBKEY_LEVELS_DEPTH_LIMIT)
//             {
//                 HashCacheStack[RemainingSubkeysInTotal].NameOfKey.Length = KeyNameLength;
//                 HashCacheStack[RemainingSubkeysInTotal].NameOfKey.MaximumLength = KeyNameLength;
//                 HashCacheStack[RemainingSubkeysInTotal].ConvKey = CopyConvKey;
//
//                 RemainingSubkeysInTotal++;
//                 KeyNameLength = 0;
//
//                 /*
//                  * Precaution check -- we have added up a remaining
//                  * subkey above but we must ensure we still have space
//                  * to hold up the new subkey for which we will compute
//                  * the hashes, so that we don't blow up the hash stack.
//                  */
//                 if (RemainingSubkeysInTotal < CMP_SUBKEY_LEVELS_DEPTH_LIMIT)
//                 {
//                     HashCacheStack[RemainingSubkeysInTotal].NameOfKey.Buffer = RemainingNameBuffer;
//                 }
//             }
//         }
//     }
//
//     *TotalSubKeys = SubkeysInTotal;
//     return RemainingSubkeysInTotal;
// }
//
// /**
//  * @brief
//  * Compares each subkey's hash and name with those
//  * captured in the hash cache stack.
//  *
//  * @param[in] HashCacheStack
//  * A pointer to a hash cache stack array filled with
//  * subkey hashes and names for comparison.
//  *
//  * @param[in] CurrentKcb
//  * A pointer to the currently given KCB.
//  *
//  * @param[in] RemainingSubkeys
//  * The remaining subkey levels to be supplied.
//  *
//  * @param[out] ParentKcb
//  * A pointer to the parent KCB returned to the caller.
//  * This parameter points to the parent of the current
//  * KCB if all the subkeys match, otherwise it points
//  * to the actual current KCB.
//  *
//  * @return
//  * Returns TRUE if all the subkey levels match, otherwise
//  * FALSE is returned.
//  */
// static
// BOOLEAN
// CmpCompareSubkeys(
//     _In_ PCM_HASH_CACHE_STACK HashCacheStack,
//     _In_ PCM_KEY_CONTROL_BLOCK CurrentKcb,
//     _In_ ULONG RemainingSubkeys,
//     _Out_ PCM_KEY_CONTROL_BLOCK *ParentKcb)
// {
//     LONG HashStackIndex;
//     LONG Result;
//     PCM_NAME_CONTROL_BLOCK NameBlock;
//     UNICODE_STRING CurrentNameBlock;
//
//     ASSERT(CurrentKcb != NULL);
//
//     /* Loop each hash and check that they match */
//     HashStackIndex = RemainingSubkeys;
//     while (HashStackIndex >= 0)
//     {
//         /* Does the subkey hash match? */
//         if (CurrentKcb->ConvKey != HashCacheStack[HashStackIndex].ConvKey)
//         {
//             *ParentKcb = CurrentKcb;
//             return FALSE;
//         }
//
//         /* Compare the subkey string, is the name compressed? */
//         NameBlock = CurrentKcb->NameBlock;
//         if (NameBlock->Compressed)
//         {
//             Result = CmpCompareCompressedName(&HashCacheStack[HashStackIndex].NameOfKey,
//                                               NameBlock->Name,
//                                               NameBlock->NameLength);
//         }
//         else
//         {
//             CurrentNameBlock.Buffer = NameBlock->Name;
//             CurrentNameBlock.Length = NameBlock->NameLength;
//             CurrentNameBlock.MaximumLength = NameBlock->NameLength;
//
//             Result = RtlCompareUnicodeString(&HashCacheStack[HashStackIndex].NameOfKey,
//                                              &CurrentNameBlock,
//                                              TRUE);
//         }
//
//         /* Do the subkey names match? */
//         if (Result)
//         {
//             *ParentKcb = CurrentKcb;
//             return FALSE;
//         }
//
//         /* Go to the next subkey hash */
//         HashStackIndex--;
//     }
//
//     /* All the subkeys match */
//     *ParentKcb = CurrentKcb->ParentKcb;
//     return TRUE;
// }
//
// /**
//  * @brief
//  * Removes the subkeys on a remaining key pathname.
//  *
//  * @param[in] HashCacheStack
//  * A pointer to a hash cache stack array filled with
//  * subkey hashes and names.
//  *
//  * @param[in] RemainingSubkeys
//  * The remaining subkey levels to be supplied.
//  *
//  * @param[in,out] RemainingName
//  * A Unicode string structure consisting of the remaining
//  * registry key path name, where the subkeys of such path
//  * are to be removed.
//  */
// static
// VOID
// CmpRemoveSubkeysInRemainingName(
//     _In_ PCM_HASH_CACHE_STACK HashCacheStack,
//     _In_ ULONG RemainingSubkeys,
//     _Inout_ PUNICODE_STRING RemainingName)
// {
//     ULONG HashStackIndex = 0;
//
//     /* Skip any leading separator on matching name */
//     while (RemainingName->Length >= sizeof(WCHAR) &&
//            RemainingName->Buffer[0] == OBJ_NAME_PATH_SEPARATOR)
//     {
//         RemainingName->Buffer++;
//         RemainingName->Length -= sizeof(WCHAR);
//     }
//
//     /* Skip the subkeys as well */
//     while (HashStackIndex <= RemainingSubkeys)
//     {
//         RemainingName->Buffer += HashCacheStack[HashStackIndex].NameOfKey.Length / sizeof(WCHAR);
//         RemainingName->Length -= HashCacheStack[HashStackIndex].NameOfKey.Length;
//
//         /* Skip any leading separator */
//         while (RemainingName->Length >= sizeof(WCHAR) &&
//                RemainingName->Buffer[0] == OBJ_NAME_PATH_SEPARATOR)
//         {
//             RemainingName->Buffer++;
//             RemainingName->Length -= sizeof(WCHAR);
//         }
//
//         /* Go to the next hash */
//         HashStackIndex++;
//     }
// }
//
// /**
//  * @brief
//  * Looks up in the pool cache for key pathname that matches
//  * with one in the said cache and returns a KCB pointing
//  * to that name. This function performs locking of KCBs
//  * during cache lookup.
//  *
//  * @param[in] HashCacheStack
//  * A pointer to a hash cache stack array filled with
//  * subkey hashes and names.
//  *
//  * @param[in] LockKcbsExclusive
//  * If set to TRUE, the KCBs are locked exclusively by the
//  * calling thread, otherwise they are locked in shared mode.
//  * See Remarks for further information.
//  *
//  * @param[in] TotalRemainingSubkeys
//  * The total remaining subkey levels to be supplied.
//  *
//  * @param[in,out] RemainingName
//  * A Unicode string structure consisting of the remaining
//  * registry key path name. The remaining name is updated
//  * by the function if a key pathname is found in cache.
//  *
//  * @param[in,out] OuterStackArray
//  * A pointer to an array that lives on the caller's stack.
//  * The expected size of the array is up to 32 elements,
//  * which is the imposed limit by CMP_HASH_STACK_LIMIT.
//  * This limit also corresponds to the maximum depth of
//  * subkey levels.
//  *
//  * @param[in,out] Kcb
//  * A pointer to a KCB, this KCB is changed if the key pathname
//  * is found in cache.
//  *
//  * @param[out] Hive
//  * A pointer to a hive, this hive is changed if the key pathname
//  * is found in cache.
//  *
//  * @param[out] Cell
//  * A pointer to a cell, this cell is changed if the key pathname
//  * is found in cache.
//  *
//  * @param[out] MatchRemainSubkeyLevel
//  * A pointer to match subkey levels returned by the function.
//  * If no match levels are found, this is 0.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if cache lookup has completed successfully.
//  * STATUS_OBJECT_NAME_NOT_FOUND is returned if the current KCB of
//  * the key pathname has been deleted. STATUS_RETRY is returned if
//  * at least the current KCB or its parent have been deleted
//  * and a cache lookup must be retried again. STATUS_UNSUCCESSFUL is
//  * returned if a KCB is referenced too many times.
//  *
//  * @remarks
//  * The function attempts to do a cache lookup with a shared lock
//  * on KCBs so that other threads can simultaneously get access
//  * to these KCBs. When the captured KCB is being deleted on us
//  * we have to retry a lookup with exclusive look so that no other
//  * threads will mess with the KCBs and perform appropriate actions
//  * if a KCB is deleted.
//  */
// static
// NTSTATUS
// CmpLookInCache(
//     _In_ PCM_HASH_CACHE_STACK HashCacheStack,
//     _In_ BOOLEAN LockKcbsExclusive,
//     _In_ ULONG TotalRemainingSubkeys,
//     _Inout_ PUNICODE_STRING RemainingName,
//     _Inout_ PULONG OuterStackArray,
//     _Inout_ PCM_KEY_CONTROL_BLOCK *Kcb,
//     _Out_ PHHIVE *Hive,
//     _Out_ PHCELL_INDEX Cell,
//     _Out_ PULONG MatchRemainSubkeyLevel)
// {
//     LONG RemainingSubkeys;
//     ULONG TotalLevels;
//     BOOLEAN SubkeysMatch;
//     PCM_KEY_CONTROL_BLOCK CurrentKcb, ParentKcb;
//     PCM_KEY_HASH HashEntry = NULL;
//     BOOLEAN KeyFoundInCache = FALSE;
//     PULONG LockedKcbs = NULL;
//
//     /* Reference the KCB */
//     if (!CmpReferenceKeyControlBlock(*Kcb))
//     {
//         /* This key is opened too many times, bail out */
//         DPRINT1("Could not reference the KCB, too many references (KCB 0x%p)\n", Kcb);
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     /* Prepare to lock the KCBs */
//     LockedKcbs = CmpBuildAndLockKcbArray(HashCacheStack,
//                                          LockKcbsExclusive ? CMP_LOCK_KCB_ARRAY_EXCLUSIVE : CMP_LOCK_KCB_ARRAY_SHARED,
//                                          *Kcb,
//                                          OuterStackArray,
//                                          TotalRemainingSubkeys,
//                                          0);
//     NT_ASSERT(LockedKcbs);
//
//     /* Lookup in the cache */
//     RemainingSubkeys = TotalRemainingSubkeys - 1;
//     TotalLevels = TotalRemainingSubkeys + (*Kcb)->TotalLevels + 1;
//     while (RemainingSubkeys >= 0)
//     {
//         /* Get the hash entry from the cache */
//         HashEntry = GET_HASH_ENTRY(CmpCacheTable, HashCacheStack[RemainingSubkeys].ConvKey)->Entry;
//
//         /* Take one level down as we are processing this hash entry */
//         TotalLevels--;
//
//         while (HashEntry != NULL)
//         {
//             /* Validate this hash and obtain the current KCB */
//             ASSERT_VALID_HASH(HashEntry);
//             CurrentKcb = CONTAINING_RECORD(HashEntry, CM_KEY_CONTROL_BLOCK, KeyHash);
//
//             /* Does this KCB have matching levels? */
//             if (TotalLevels == CurrentKcb->TotalLevels)
//             {
//                 /*
//                  * We have matching subkey levels but don't directly assume we have
//                  * a matching key path in cache. Start comparing each subkey.
//                  */
//                 SubkeysMatch = CmpCompareSubkeys(HashCacheStack,
//                                                  CurrentKcb,
//                                                  RemainingSubkeys,
//                                                  &ParentKcb);
//                 if (SubkeysMatch)
//                 {
//                     /* All subkeys match, now check if the base KCB matches with parent */
//                     if (*Kcb == ParentKcb)
//                     {
//                         /* Is the KCB marked as deleted? */
//                         if (CurrentKcb->Delete ||
//                             CurrentKcb->ParentKcb->Delete)
//                         {
//                             /*
//                              * Either the current or its parent KCB is marked
//                              * but we had a shared lock so probably a naughty
//                              * thread was deleting it. Retry doing a cache
//                              * lookup again with exclusive lock.
//                              */
//                             if (!LockKcbsExclusive)
//                             {
//                                 CmpUnLockKcbArray(LockedKcbs);
//                                 CmpDereferenceKeyControlBlock(*Kcb);
//                                 DPRINT1("The current KCB or its parent is deleted, retrying looking in the cache\n");
//                                 return STATUS_RETRY;
//                             }
//
//                             /* We're under an exclusive lock, is the KCB deleted yet? */
//                             if (CurrentKcb->Delete)
//                             {
//                                 /* The KCB is gone, the key should no longer belong in the cache */
//                                 CmpRemoveKeyControlBlock(CurrentKcb);
//                                 CmpUnLockKcbArray(LockedKcbs);
//                                 CmpDereferenceKeyControlBlock(*Kcb);
//                                 DPRINT1("The current KCB is deleted (KCB 0x%p)\n", CurrentKcb);
//                                 return STATUS_OBJECT_NAME_NOT_FOUND;
//                             }
//
//                             /*
//                              * The parent is deleted so it must be that somebody created
//                              * a fake key. Assert ourselves that is the case.
//                              */
//                             ASSERT(CurrentKcb->ExtFlags & CM_KCB_KEY_NON_EXIST);
//
//                             /* Remove this KCB out of cache if someone still uses it */
//                             if (CurrentKcb->RefCount != 0)
//                             {
//                                 CurrentKcb->Delete = TRUE;
//                                 CmpRemoveKeyControlBlock(CurrentKcb);
//                             }
//                             else
//                             {
//                                 /* Otherwise expunge it */
//                                 CmpRemoveFromDelayedClose(CurrentKcb);
//                                 CmpCleanUpKcbCacheWithLock(CurrentKcb, FALSE);
//                             }
//
//                             /* Stop looking for next hashes as the KCB is kaput */
//                             break;
//                         }
//
//                         /* We finally found the key in cache, acknowledge it */
//                         KeyFoundInCache = TRUE;
//
//                         /* Remove the subkeys in the remaining name and stop looking in the cache */
//                         CmpRemoveSubkeysInRemainingName(HashCacheStack, RemainingSubkeys, RemainingName);
//                         break;
//                     }
//                 }
//             }
//
//             /* Go to the next hash */
//             HashEntry = HashEntry->NextHash;
//         }
//
//         /* Stop looking in cache if we found the matching key */
//         if (KeyFoundInCache)
//         {
//             DPRINT("Key found in cache, stop looking\n");
//             break;
//         }
//
//         /* Keep looking in the cache until we run out of remaining subkeys */
//         RemainingSubkeys--;
//     }
//
//     /* Return the matching subkey levels */
//     *MatchRemainSubkeyLevel = RemainingSubkeys + 1;
//
//     /* We have to update the KCB if the key was found in cache */
//     if (KeyFoundInCache)
//     {
//         /*
//          * Before we change the KCB we must dereference the prior
//          * KCB that we no longer need it.
//          */
//         CmpDereferenceKeyControlBlock(*Kcb);
//         *Kcb = CurrentKcb;
//
//         /* Reference the new KCB now */
//         if (!CmpReferenceKeyControlBlock(*Kcb))
//         {
//             /* This key is opened too many times, bail out */
//             DPRINT1("Could not reference the KCB, too many references (KCB 0x%p)\n", Kcb);
//             return STATUS_UNSUCCESSFUL;
//         }
//
//         /* Update hive and cell data from current KCB */
//         *Hive = CurrentKcb->KeyHive;
//         *Cell = CurrentKcb->KeyCell;
//     }
//
//     /* Unlock the KCBs */
//     CmpUnLockKcbArray(LockedKcbs);
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Builds a hash stack cache and looks up in the
//  * pool cache for a matching key pathname.
//  *
//  * @param[in] ParseObject
//  * A pointer to a parse object, acting as a key
//  * body. This parameter is unused.
//  *
//  * @param[in,out] Kcb
//  * A pointer to a KCB. This KCB is used by the
//  * registry parser after hash stack and cache
//  * lookup are done. This KCB might change if the
//  * key is found to be cached in the cache pool.
//  *
//  * @param[in] Current
//  * The current remaining key pathname.
//  *
//  * @param[out] Hive
//  * A pointer to a registry hive, returned by the caller.
//  *
//  * @param[out] Cell
//  * A pointer to a hive cell, returned by the caller.
//  *
//  * @param[out] TotalRemainingSubkeys
//  * A pointer to a number of total remaining subkey levels,
//  * returned by the caller. This can be 0 if no subkey levels
//  * have been found.
//  *
//  * @param[out] MatchRemainSubkeyLevel
//  * A pointer to a number of remaining subkey levels that match,
//  * returned by the caller. This can be 0 if no matching levels
//  * are found.
//  *
//  * @param[out] TotalSubkeys
//  * A pointer to a number of total subkeys. This can be 0 if no
//  * subkey levels are found. By definition, both MatchRemainSubkeyLevel
//  * and TotalRemainingSubkeys are 0 as well.
//  *
//  * @param[in,out] OuterStackArray
//  * A pointer to an array that lives on the caller's stack.
//  * The expected size of the array is up to 32 elements,
//  * which is the imposed limit by CMP_HASH_STACK_LIMIT.
//  * This limit also corresponds to the maximum depth of
//  * subkey levels.
//  *
//  * @param[out] LockedKcbs
//  * A pointer to an array of locked KCBs, returned by the caller.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if all the operations have succeeded without
//  * problems. STATUS_NAME_TOO_LONG is returned if the key pathname has
//  * too many subkey levels (more than 32 levels deep). A failure NTSTATUS
//  * code is returned otherwise. Refer to CmpLookInCache documentation
//  * for more information about other returned status codes.
//  * STATUS_UNSUCCESSFUL is returned if a KCB is referenced too many times.
//  */
// NTSTATUS
// NTAPI
// CmpBuildHashStackAndLookupCache(
//     _In_ PCM_KEY_BODY ParseObject,
//     _Inout_ PCM_KEY_CONTROL_BLOCK *Kcb,
//     _In_ PUNICODE_STRING Current,
//     _Out_ PHHIVE *Hive,
//     _Out_ PHCELL_INDEX Cell,
//     _Out_ PULONG TotalRemainingSubkeys,
//     _Out_ PULONG MatchRemainSubkeyLevel,
//     _Out_ PULONG TotalSubkeys,
//     _Inout_ PULONG OuterStackArray,
//     _Out_ PULONG *LockedKcbs)
// {
//     NTSTATUS Status;
//     ULONG ConvKey;
//     ULONG SubkeysInTotal, RemainingSubkeysInTotal, MatchRemainingSubkeys;
//     CM_HASH_CACHE_STACK HashCacheStack[CMP_SUBKEY_LEVELS_DEPTH_LIMIT];
//
//     /* Make sure it's not a dead KCB */
//     ASSERT((*Kcb)->RefCount > 0);
//
//     /* Lock the registry */
//     CmpLockRegistry();
//
//     /* Calculate hash values for every subkey this key path has */
//     ConvKey = (*Kcb)->ConvKey;
//     RemainingSubkeysInTotal = CmpComputeHashValue(Current,
//                                                   ConvKey,
//                                                   HashCacheStack,
//                                                   &SubkeysInTotal);
//
//     /* This key path has too many subkeys */
//     if (SubkeysInTotal > CMP_SUBKEY_LEVELS_DEPTH_LIMIT)
//     {
//         DPRINT1("The key path has too many subkeys - %lu\n", SubkeysInTotal);
//         *LockedKcbs = NULL;
//         return STATUS_NAME_TOO_LONG;
//     }
//
//     /* Return hive and cell data */
//     *Hive = (*Kcb)->KeyHive;
//     *Cell = (*Kcb)->KeyCell;
//
//     /* Do we have any subkeys? */
//     if (!RemainingSubkeysInTotal && !SubkeysInTotal)
//     {
//         /*
//          * We don't have any subkeys nor remaining levels, the
//          * KCB points to the actual key. Lock it.
//          */
//         if (!CmpReferenceKeyControlBlock(*Kcb))
//         {
//             /* This key is opened too many times, bail out */
//             DPRINT1("Could not reference the KCB, too many references (KCB 0x%p)\n", Kcb);
//             return STATUS_UNSUCCESSFUL;
//         }
//
//         CmpAcquireKcbLockSharedByIndex(GET_HASH_INDEX(ConvKey));
//
//         /* Add this KCB in the array of locked KCBs */
//         OuterStackArray[0] = 1;
//         OuterStackArray[1] = GET_HASH_INDEX(ConvKey);
//         *LockedKcbs = OuterStackArray;
//
//         /* And return all the subkey level counters */
//         *TotalRemainingSubkeys = RemainingSubkeysInTotal;
//         *MatchRemainSubkeyLevel = 0;
//         *TotalSubkeys = SubkeysInTotal;
//         return STATUS_SUCCESS;
//     }
//
//     /* Lookup in the cache */
//     Status = CmpLookInCache(HashCacheStack,
//                             FALSE,
//                             RemainingSubkeysInTotal,
//                             Current,
//                             OuterStackArray,
//                             Kcb,
//                             Hive,
//                             Cell,
//                             &MatchRemainingSubkeys);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Bail out if cache lookup failed for other reasons */
//         if (Status != STATUS_RETRY)
//         {
//             DPRINT1("CmpLookInCache() failed (Status 0x%lx)\n", Status);
//             *LockedKcbs = NULL;
//             return Status;
//         }
//
//         /* Retry looking in the cache but with KCBs locked exclusively */
//         Status = CmpLookInCache(HashCacheStack,
//                                 TRUE,
//                                 RemainingSubkeysInTotal,
//                                 Current,
//                                 OuterStackArray,
//                                 Kcb,
//                                 Hive,
//                                 Cell,
//                                 &MatchRemainingSubkeys);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("CmpLookInCache() failed after retry (Status 0x%lx)\n", Status);
//             *LockedKcbs = NULL;
//             return Status;
//         }
//     }
//
//     /*
//      * Check if we have a full match of remaining levels.
//      *
//      * FIXME: It is possible we can catch a fake key from the cache
//      * when we did the lookup, in such case we should not do any
//      * locking as such KCB does not point to any real information.
//      * Currently ReactOS doesn't create fake KCBs so we are good
//      * for now.
//      */
//     if (RemainingSubkeysInTotal == MatchRemainingSubkeys)
//     {
//         /*
//          * Just simply lock this KCB as it points to the full
//          * subkey levels in cache.
//          */
//         CmpAcquireKcbLockSharedByIndex(GET_HASH_INDEX((*Kcb)->ConvKey));
//         OuterStackArray[0] = 1;
//         OuterStackArray[1] = GET_HASH_INDEX((*Kcb)->ConvKey);
//         *LockedKcbs = OuterStackArray;
//     }
//     else
//     {
//         /*
//          * We only have a partial match so other subkey levels
//          * have each KCB. Simply just lock them.
//          */
//         *LockedKcbs = CmpBuildAndLockKcbArray(HashCacheStack,
//                                               CMP_LOCK_KCB_ARRAY_EXCLUSIVE,
//                                               *Kcb,
//                                               OuterStackArray,
//                                               RemainingSubkeysInTotal,
//                                               MatchRemainingSubkeys);
//         NT_ASSERT(*LockedKcbs);
//     }
//
//     /* Return all the subkey level counters */
//     *TotalRemainingSubkeys = RemainingSubkeysInTotal;
//     *MatchRemainSubkeyLevel = MatchRemainingSubkeys;
//     *TotalSubkeys = SubkeysInTotal;
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// CmpParseKey — full translation from cmparse.c lines 1847-2311.
// Used as ObjectTypeInitializer.ParseProcedure for the registry key object type.
//
// Local flag values mirror ntoskrnl/include/internal/cm.h.
#[no_mangle]
pub unsafe extern "system" fn CmpParseKey(
    ParseObject: PVOID,
    ObjectType: PVOID,
    AccessState: PACCESS_STATE,
    AccessMode: KPROCESSOR_MODE,
    Attributes: u32,
    CompleteName: PUNICODE_STRING,
    RemainingName: PUNICODE_STRING,
    Context: PVOID,
    SecurityQos: PVOID,
    Object: *mut PVOID,
) -> NTSTATUS {
    // WoW64 registry redirection. Every key open in the system reaches this
    // parse procedure, so the whole feature lives in this wrapper: pick the
    // view, splice WOW6432NODE into the name if it applies, and fall back to
    // the 64-bit key when the 32-bit one does not exist. See cmwow64.rs.
    //
    // The flag consume ALSO runs for native processes, and must: it is what
    // clears KEY_WOW64_* before ObCheckObjectAccess sees them (E21/#79). It is
    // done here rather than in NtOpenKey/NtCreateKey so the bits survive long
    // enough to choose a view.
    let View32 = crate::cmwow64::CmpWow64ConsumeViewFlags(AccessState);

    let mut Redirected = if View32 {
        crate::cmwow64::CmpWow64BuildRedirectedName(ParseObject, RemainingName)
    } else {
        None
    };

    let Some(ref mut Redir) = Redirected else {
        return CmpParseKeyInner(
            ParseObject, ObjectType, AccessState, AccessMode, Attributes,
            CompleteName, RemainingName, Context, SecurityQos, Object,
        );
    };

    // A create must land in the 32-bit view or it isn't isolation; only reads
    // fall back. CM_PARSE_CONTEXT.CreateOperation is the same discriminator
    // CmpParseKeyInner uses to decide it is a create at all.
    let IsCreate = !Context.is_null() && (*(Context as PCM_PARSE_CONTEXT)).CreateOperation != 0;

    let mut Status = CmpParseKeyInner(
        ParseObject, ObjectType, AccessState, AccessMode, Attributes,
        CompleteName, core::ptr::addr_of_mut!(Redir.Name), Context, SecurityQos, Object,
    );

    if Status == STATUS_OBJECT_NAME_NOT_FOUND_MOD && !IsCreate {
        Status = CmpParseKeyInner(
            ParseObject, ObjectType, AccessState, AccessMode, Attributes,
            CompleteName, RemainingName, Context, SecurityQos, Object,
        );
    }

    Redir.free();
    Status
}

unsafe fn CmpParseKeyInner(
    ParseObject: PVOID,
    ObjectType: PVOID,
    AccessState: PACCESS_STATE,
    AccessMode: KPROCESSOR_MODE,
    Attributes: u32,
    CompleteName: PUNICODE_STRING,
    RemainingName: PUNICODE_STRING,
    Context: PVOID,
    _SecurityQos: PVOID,
    Object: *mut PVOID,
) -> NTSTATUS {
    let ParseObject = opaque_ptr(ParseObject);
    let ObjectType = opaque_ptr(ObjectType);
    let AccessState = opaque_ptr(AccessState);
    let CompleteName = opaque_ptr(CompleteName);
    let RemainingName = opaque_ptr(RemainingName);
    let Context = opaque_ptr(Context);
    let _SecurityQos = opaque_ptr(_SecurityQos);
    let Object = opaque_ptr(Object);
    const CMP_OPEN_KCB_NO_CREATE_LOCAL: u32 = 0x00000004;
    const CMP_CREATE_KCB_KCB_LOCKED_LOCAL: u32 = 0x00000002;
    const CMP_LOCK_HASHES_FOR_KCB_LOCAL: u32 = 0x00000002;
    const KEY_SYM_LINK_LOCAL: u32 = 0x0010;
    const KEY_HIVE_EXIT_LOCAL: u16 = 0x0002;
    const MAXUSHORT_LOCAL: u32 = 0xFFFF;

    let mut Status: NTSTATUS;
    let mut Kcb: PCM_KEY_CONTROL_BLOCK;
    let mut ParentKcb: PCM_KEY_CONTROL_BLOCK;
    let mut Hive: PHHIVE = core::ptr::null_mut();
    let mut Node: PCM_KEY_NODE = core::ptr::null_mut();
    let mut Cell: HCELL_INDEX = HCELL_NIL;
    let mut NextCell: HCELL_INDEX;
    let mut HiveToRelease: PHHIVE = core::ptr::null_mut();
    let mut CellToRelease: HCELL_INDEX = HCELL_NIL;
    let mut Current: _UNICODE_STRING;
    let mut NextName: _UNICODE_STRING = core::mem::zeroed();
    let mut ParseContext: PCM_PARSE_CONTEXT = Context as PCM_PARSE_CONTEXT;
    let mut TotalRemainingSubkeys: ULONG = 0;
    let mut MatchRemainSubkeyLevel: ULONG = 0;
    let mut TotalSubkeys: ULONG = 0;
    // CMP_KCBS_IN_ARRAY_LIMIT = CMP_SUBKEY_LEVELS_DEPTH_LIMIT (32) + 2 = 34.
    let mut LockedKcbArray: [ULONG; 34] = [0; 34];
    let mut LockedKcbs: *mut ULONG = core::ptr::null_mut();
    let mut IsKeyCached: u8 = 0u8;
    let mut Result: u8;
    let mut Last: u8 = 0u8;

    // Loop trailing path separators off RemainingName.
    while (*RemainingName).Length != 0
        && *(*RemainingName).Buffer.add(((*RemainingName).Length as usize / core::mem::size_of::<u16>()) - 1)
            == OBJ_NAME_PATH_SEPARATOR
    {
        (*RemainingName).Length -= core::mem::size_of::<u16>() as u16;
    }

    // Fail if this isn't a key object.
    if ObjectType != CmpKeyObjectType as PVOID {
        return STATUS_OBJECT_TYPE_MISMATCH;
    }

    // Copy the remaining name.
    Current = *RemainingName;

    // Check if this is a create.
    if ParseContext.is_null() || (*ParseContext).CreateOperation == 0 {
        ParseContext = core::ptr::null_mut();
    }

    // Grab the KCB.
    Kcb = (*(ParseObject as PCM_KEY_BODY)).KeyControlBlock;

    // Sanity check.
    // ASSERT(Kcb != NULL);

    // Fail if the key was marked as deleted.
    if (*Kcb).__bindgen_anon_1.Delete() != 0 {
        return STATUS_KEY_DELETED;
    }

    // Lookup in the cache.
    Status = CmpBuildHashStackAndLookupCache(
        ParseObject as PCM_KEY_BODY,
        core::ptr::addr_of_mut!(Kcb),
        core::ptr::addr_of_mut!(Current),
        core::ptr::addr_of_mut!(Hive),
        core::ptr::addr_of_mut!(Cell),
        core::ptr::addr_of_mut!(TotalRemainingSubkeys),
        core::ptr::addr_of_mut!(MatchRemainSubkeyLevel),
        core::ptr::addr_of_mut!(TotalSubkeys),
        LockedKcbArray.as_mut_ptr(),
        core::ptr::addr_of_mut!(LockedKcbs),
    );
    // CMP_ASSERT_REGISTRY_LOCK();
    if !NT_SUCCESS(Status) {
        crate::dbg_print!(
            b"Failed to look in cache, stop parsing (Status 0x%lx)\n\0",
            Status
        );
        ParentKcb = core::ptr::null_mut();
        // Goto Quickie.
        if !LockedKcbs.is_null() {
            CmpUnLockKcbArray(LockedKcbs);
        }
        if !ParentKcb.is_null() {
            CmpDereferenceKeyControlBlock(ParentKcb);
        }
        CmpUnlockRegistry();
        return Status;
    }

    // This is now the parent.
    ParentKcb = Kcb;

    // Don't do anything if we're being deleted.
    if (*Kcb).__bindgen_anon_1.Delete() != 0 {
        Status = STATUS_OBJECT_NAME_NOT_FOUND;
        // Goto Quickie.
        if !LockedKcbs.is_null() {
            CmpUnLockKcbArray(LockedKcbs);
        }
        if !ParentKcb.is_null() {
            CmpDereferenceKeyControlBlock(ParentKcb);
        }
        CmpUnlockRegistry();
        return Status;
    }

    // Fast path: nothing remaining → cached open of the existing KCB.
    let mut UseCachedOpenJump: bool = false;
    if TotalRemainingSubkeys == 0 {
        IsKeyCached = 1u8;
        UseCachedOpenJump = true;
    } else if MatchRemainSubkeyLevel != 0 {
        if MatchRemainSubkeyLevel == TotalSubkeys {
            IsKeyCached = 1u8;
            UseCachedOpenJump = true;
        }
        // else: partial match, Hive/Cell already from cache.
    }

    // KeyCachedOpenNow handler — performed inline before the loop if the
    // fast path was selected. The `NextName` value is empty on this path
    // (no further parsing), matching the C use of the still-zeroed local.
    if UseCachedOpenJump {
        Status = CmpDoOpen(
            Hive,
            Cell,
            Node,
            AccessState,
            AccessMode,
            Attributes,
            ParseContext,
            if IsKeyCached != 0 {
                CMP_OPEN_KCB_NO_CREATE_LOCAL
            } else {
                CMP_CREATE_KCB_KCB_LOCKED_LOCAL
            },
            core::ptr::addr_of_mut!(Kcb),
            LockedKcbs,
            core::ptr::addr_of_mut!(NextName),
            Object,
        );
        if Status == STATUS_REPARSE {
            // CmpGetSymbolicLink doesn't want a lock.
            CmpUnLockKcbArray(LockedKcbs);
            LockedKcbs = core::ptr::null_mut();

            if CmpGetSymbolicLink(Hive, CompleteName, Kcb, core::ptr::null_mut()) == 0 {
                Status = STATUS_OBJECT_NAME_NOT_FOUND;
            }
        }
        // Quickie.
        if !LockedKcbs.is_null() {
            CmpUnLockKcbArray(LockedKcbs);
        }
        if !ParentKcb.is_null() {
            CmpDereferenceKeyControlBlock(ParentKcb);
        }
        CmpUnlockRegistry();
        return Status;
    }

    // Check if this is a symlink.
    if ((*Kcb).__bindgen_anon_5.Flags() & KEY_SYM_LINK_LOCAL) != 0 {
        // Get the next name.
        Result = CmpGetNextName(
            core::ptr::addr_of_mut!(Current),
            core::ptr::addr_of_mut!(NextName),
            core::ptr::addr_of_mut!(Last),
        );
        let _ = Result;
        Current.Buffer = NextName.Buffer;

        // Validate the current name string length.
        if Current.Length as u32 + NextName.Length as u32 > MAXUSHORT_LOCAL {
            Status = 0xC0000106u32 as i32; // STATUS_NAME_TOO_LONG
            // Quickie.
            if !LockedKcbs.is_null() {
                CmpUnLockKcbArray(LockedKcbs);
            }
            if !ParentKcb.is_null() {
                CmpDereferenceKeyControlBlock(ParentKcb);
            }
            CmpUnlockRegistry();
            return Status;
        }
        Current.Length += NextName.Length;

        // Validate the current name string maximum length.
        if Current.MaximumLength as u32 + NextName.MaximumLength as u32 > MAXUSHORT_LOCAL {
            Status = 0xC0000106u32 as i32; // STATUS_NAME_TOO_LONG
            // Quickie.
            if !LockedKcbs.is_null() {
                CmpUnLockKcbArray(LockedKcbs);
            }
            if !ParentKcb.is_null() {
                CmpDereferenceKeyControlBlock(ParentKcb);
            }
            CmpUnlockRegistry();
            return Status;
        }
        Current.MaximumLength += NextName.MaximumLength;

        // CmpGetSymbolicLink doesn't want a lock.
        CmpUnLockKcbArray(LockedKcbs);
        LockedKcbs = core::ptr::null_mut();

        if CmpGetSymbolicLink(Hive, CompleteName, Kcb, core::ptr::addr_of_mut!(Current)) != 0 {
            Status = STATUS_REPARSE;
        } else {
            Status = STATUS_OBJECT_NAME_NOT_FOUND;
        }

        // Quickie.
        if !LockedKcbs.is_null() {
            CmpUnLockKcbArray(LockedKcbs);
        }
        if !ParentKcb.is_null() {
            CmpDereferenceKeyControlBlock(ParentKcb);
        }
        CmpUnlockRegistry();
        return Status;
    }

    // Get the key node.
    Node = HvGetCell(Hive, Cell) as PCM_KEY_NODE;
    if Node.is_null() {
        Status = STATUS_INSUFFICIENT_RESOURCES;
        // Quickie.
        if !LockedKcbs.is_null() {
            CmpUnLockKcbArray(LockedKcbs);
        }
        if !ParentKcb.is_null() {
            CmpDereferenceKeyControlBlock(ParentKcb);
        }
        CmpUnlockRegistry();
        return Status;
    }

    // Start parsing.
    Status = STATUS_NOT_IMPLEMENTED;
    'parse_loop: loop {
        // Get the next component.
        Result = CmpGetNextName(
            core::ptr::addr_of_mut!(Current),
            core::ptr::addr_of_mut!(NextName),
            core::ptr::addr_of_mut!(Last),
        );

        if Result != 0 && NextName.Length != 0 {
            // See if this is a sym link.
            if ((*Kcb).__bindgen_anon_5.Flags() & KEY_SYM_LINK_LOCAL) == 0 {
                // Find the subkey.
                NextCell = CmpFindSubKeyByName(Hive, Node, core::ptr::addr_of_mut!(NextName));
                if NextCell != HCELL_NIL {
                    // Get the new node.
                    Cell = NextCell;
                    Node = HvGetCell(Hive, Cell) as PCM_KEY_NODE;
                    // ASSERT(Node);

                    // Check if this was the last key.
                    if Last != 0 {
                        // Is this an exit node?
                        if ((*Node).Flags & KEY_HIVE_EXIT_LOCAL) != 0 {
                            CmpHandleExitNode(
                                core::ptr::addr_of_mut!(Hive),
                                core::ptr::addr_of_mut!(Cell),
                                core::ptr::addr_of_mut!(Node),
                                core::ptr::addr_of_mut!(HiveToRelease),
                                core::ptr::addr_of_mut!(CellToRelease),
                            );
                            if Node.is_null() {
                                Status = STATUS_INSUFFICIENT_RESOURCES;
                                break 'parse_loop;
                            }
                        }

                        // KeyCachedOpenNow.
                        Status = CmpDoOpen(
                            Hive,
                            Cell,
                            Node,
                            AccessState,
                            AccessMode,
                            Attributes,
                            ParseContext,
                            if IsKeyCached != 0 {
                                CMP_OPEN_KCB_NO_CREATE_LOCAL
                            } else {
                                CMP_CREATE_KCB_KCB_LOCKED_LOCAL
                            },
                            core::ptr::addr_of_mut!(Kcb),
                            LockedKcbs,
                            core::ptr::addr_of_mut!(NextName),
                            Object,
                        );
                        if Status == STATUS_REPARSE {
                            // CmpGetSymbolicLink doesn't want a lock.
                            CmpUnLockKcbArray(LockedKcbs);
                            LockedKcbs = core::ptr::null_mut();

                            if CmpGetSymbolicLink(Hive, CompleteName, Kcb, core::ptr::null_mut()) == 0 {
                                Status = STATUS_OBJECT_NAME_NOT_FOUND;
                            }
                        }

                        break 'parse_loop;
                    }

                    // Is this an exit node?
                    if ((*Node).Flags & KEY_HIVE_EXIT_LOCAL) != 0 {
                        CmpHandleExitNode(
                            core::ptr::addr_of_mut!(Hive),
                            core::ptr::addr_of_mut!(Cell),
                            core::ptr::addr_of_mut!(Node),
                            core::ptr::addr_of_mut!(HiveToRelease),
                            core::ptr::addr_of_mut!(CellToRelease),
                        );
                        if Node.is_null() {
                            Status = STATUS_INSUFFICIENT_RESOURCES;
                            break 'parse_loop;
                        }
                    }

                    // Create a KCB for this key.
                    Kcb = CmpCreateKeyControlBlock(
                        Hive,
                        Cell,
                        Node,
                        ParentKcb,
                        CMP_LOCK_HASHES_FOR_KCB_LOCAL,
                        core::ptr::addr_of_mut!(NextName),
                    );
                    if Kcb.is_null() {
                        Status = STATUS_INSUFFICIENT_RESOURCES;
                        break 'parse_loop;
                    }

                    // Dereference the parent and set the new one.
                    CmpDereferenceKeyControlBlockWithLock(ParentKcb, 0u8);
                    ParentKcb = Kcb;
                } else {
                    // Check if this was the last key for a create.
                    if Last != 0 && !ParseContext.is_null() {
                        // Check if we're doing a link node.
                        if (*ParseContext).CreateLink != 0 {
                            // The only thing we should see.
                            Status = CmpCreateLinkNode(
                                Hive,
                                Cell,
                                AccessState,
                                NextName,
                                AccessMode,
                                Attributes,
                                ParseContext,
                                ParentKcb,
                                LockedKcbs,
                                Object,
                            );
                        } else if Hive == core::ptr::addr_of_mut!((*CmiVolatileHive).Hive)
                            && CmpNoVolatileCreates != 0
                        {
                            // Creating keys in the master hive is not allowed.
                            Status = STATUS_INVALID_PARAMETER;
                        } else {
                            // Do the create.
                            Status = CmpDoCreate(
                                Hive,
                                Cell,
                                AccessState,
                                core::ptr::addr_of_mut!(NextName),
                                AccessMode,
                                ParseContext,
                                ParentKcb,
                                Object,
                            );
                        }

                        // Check for reparse (in this case, someone beat us).
                        if Status == STATUS_REPARSE {
                            break 'parse_loop;
                        }

                        // Update disposition.
                        (*ParseContext).Disposition = 0x00000001; // REG_CREATED_NEW_KEY
                        break 'parse_loop;
                    } else {
                        // Key not found.
                        Status = STATUS_OBJECT_NAME_NOT_FOUND;
                        break 'parse_loop;
                    }
                }
            } else {
                // Save the next name.
                Current.Buffer = NextName.Buffer;

                // Validate the current name string length.
                if Current.Length as u32 + NextName.Length as u32 > MAXUSHORT_LOCAL {
                    Status = 0xC0000106u32 as i32; // STATUS_NAME_TOO_LONG
                    break 'parse_loop;
                }
                Current.Length += NextName.Length;

                // Validate the current name string maximum length.
                if Current.MaximumLength as u32 + NextName.MaximumLength as u32 > MAXUSHORT_LOCAL {
                    Status = 0xC0000106u32 as i32; // STATUS_NAME_TOO_LONG
                    break 'parse_loop;
                }
                Current.MaximumLength += NextName.MaximumLength;

                // CmpGetSymbolicLink doesn't want a lock.
                CmpUnLockKcbArray(LockedKcbs);
                LockedKcbs = core::ptr::null_mut();

                if CmpGetSymbolicLink(Hive, CompleteName, Kcb, core::ptr::addr_of_mut!(Current)) != 0 {
                    Status = STATUS_REPARSE;
                } else {
                    Status = STATUS_OBJECT_NAME_NOT_FOUND;
                }

                break 'parse_loop;
            }
        } else if Result != 0 && Last != 0 {
            // Opening the root. Is this an exit node?
            if ((*Node).Flags & KEY_HIVE_EXIT_LOCAL) != 0 {
                CmpHandleExitNode(
                    core::ptr::addr_of_mut!(Hive),
                    core::ptr::addr_of_mut!(Cell),
                    core::ptr::addr_of_mut!(Node),
                    core::ptr::addr_of_mut!(HiveToRelease),
                    core::ptr::addr_of_mut!(CellToRelease),
                );
                if Node.is_null() {
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    break 'parse_loop;
                }
            }

            // Do the open.
            Status = CmpDoOpen(
                Hive,
                Cell,
                Node,
                AccessState,
                AccessMode,
                Attributes,
                ParseContext,
                CMP_OPEN_KCB_NO_CREATE_LOCAL,
                core::ptr::addr_of_mut!(Kcb),
                LockedKcbs,
                core::ptr::addr_of_mut!(NextName),
                Object,
            );
            // STATUS_REPARSE: original C had an empty body — preserve.
            break 'parse_loop;
        } else {
            // Bogus.
            Status = STATUS_INVALID_PARAMETER;
            break 'parse_loop;
        }
    }

    // Quickie:
    // Unlock all the KCBs.
    if !LockedKcbs.is_null() {
        CmpUnLockKcbArray(LockedKcbs);
    }

    // Dereference the parent if it exists.
    if !ParentKcb.is_null() {
        CmpDereferenceKeyControlBlock(ParentKcb);
    }

    // Unlock the registry.
    CmpUnlockRegistry();
    Status
}

//
// NTSTATUS
// NTAPI
// CmpParseKey(IN PVOID ParseObject,
//             IN PVOID ObjectType,
//             IN OUT PACCESS_STATE AccessState,
//             IN KPROCESSOR_MODE AccessMode,
//             IN ULONG Attributes,
//             IN OUT PUNICODE_STRING CompleteName,
//             IN OUT PUNICODE_STRING RemainingName,
//             IN OUT PVOID Context OPTIONAL,
//             IN PSECURITY_QUALITY_OF_SERVICE SecurityQos OPTIONAL,
//             OUT PVOID *Object)
// {
//     NTSTATUS Status;
//     PCM_KEY_CONTROL_BLOCK Kcb, ParentKcb;
//     PHHIVE Hive = NULL;
//     PCM_KEY_NODE Node = NULL;
//     HCELL_INDEX Cell = HCELL_NIL, NextCell;
//     PHHIVE HiveToRelease = NULL;
//     HCELL_INDEX CellToRelease = HCELL_NIL;
//     UNICODE_STRING Current, NextName;
//     PCM_PARSE_CONTEXT ParseContext = Context;
//     ULONG TotalRemainingSubkeys = 0, MatchRemainSubkeyLevel = 0, TotalSubkeys = 0;
//     ULONG LockedKcbArray[CMP_KCBS_IN_ARRAY_LIMIT];
//     PULONG LockedKcbs;
//     BOOLEAN IsKeyCached = FALSE;
//     BOOLEAN Result, Last;
//     PAGED_CODE();
//
//     /* Loop path separators at the end */
//     while (RemainingName->Length &&
//            (RemainingName->Buffer[(RemainingName->Length / sizeof(WCHAR)) - 1] ==
//             OBJ_NAME_PATH_SEPARATOR))
//     {
//         /* Remove path separator */
//         RemainingName->Length -= sizeof(WCHAR);
//     }
//
//     /* Fail if this isn't a key object */
//     if (ObjectType != CmpKeyObjectType) return STATUS_OBJECT_TYPE_MISMATCH;
//
//     /* Copy the remaining name */
//     Current = *RemainingName;
//
//     /* Check if this is a create */
//     if (!ParseContext || !ParseContext->CreateOperation)
//     {
//         /* It isn't, so no context */
//         ParseContext = NULL;
//     }
//
//     /* Grab the KCB */
//     Kcb = ((PCM_KEY_BODY)ParseObject)->KeyControlBlock;
//
//     /* Sanity check */
//     ASSERT(Kcb != NULL);
//
//     /* Fail if the key was marked as deleted */
//     if (Kcb->Delete)
//         return STATUS_KEY_DELETED;
//
//     /* Lookup in the cache */
//     Status = CmpBuildHashStackAndLookupCache(ParseObject,
//                                              &Kcb,
//                                              &Current,
//                                              &Hive,
//                                              &Cell,
//                                              &TotalRemainingSubkeys,
//                                              &MatchRemainSubkeyLevel,
//                                              &TotalSubkeys,
//                                              LockedKcbArray,
//                                              &LockedKcbs);
//     CMP_ASSERT_REGISTRY_LOCK();
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to look in cache, stop parsing (Status 0x%lx)\n", Status);
//         ParentKcb = NULL;
//         goto Quickie;
//     }
//
//     /* This is now the parent */
//     ParentKcb = Kcb;
//
//     /* Sanity check */
//     ASSERT(ParentKcb != NULL);
//
//     /* Don't do anything if we're being deleted */
//     if (Kcb->Delete)
//     {
//         Status = STATUS_OBJECT_NAME_NOT_FOUND;
//         goto Quickie;
//     }
//
//     /* Check if everything was found cached */
//     if (!TotalRemainingSubkeys)
//     {
//         /*
//          * We don't have any remaining subkey levels so we're good
//          * that we have an already perfect candidate for a KCB, just
//          * do the open directly.
//          */
//         DPRINT("No remaining subkeys, the KCB points to the actual key\n");
//         IsKeyCached = TRUE;
//         goto KeyCachedOpenNow;
//     }
//
//     /* Check if we have a matching level */
//     if (MatchRemainSubkeyLevel)
//     {
//         /*
//          * We have a matching level, check if that matches
//          * with the total levels of subkeys. Do the open directly
//          * if that is the case, because the whole subkeys levels
//          * is cached.
//          */
//         if (MatchRemainSubkeyLevel == TotalSubkeys)
//         {
//             DPRINT("We have a full matching level, open the key now\n");
//             IsKeyCached = TRUE;
//             goto KeyCachedOpenNow;
//         }
//
//         /*
//          * We only have a partial match, make sure we did not
//          * get mismatched hive data.
//          */
//         ASSERT(Hive == Kcb->KeyHive);
//         ASSERT(Cell == Kcb->KeyCell);
//     }
//
//     /*
//      * FIXME: Currently the registry parser doesn't check for fake
//      * KCBs. CmpCreateKeyControlBlock does have the necessary implementation
//      * to create such fake keys but we don't create these fake keys anywhere.
//      * When we will do, we must improve the registry parser routine to handle
//      * fake keys a bit differently here.
//      */
//
//     /* Check if this is a symlink */
//     if (Kcb->Flags & KEY_SYM_LINK)
//     {
//         /* Get the next name */
//         Result = CmpGetNextName(&Current, &NextName, &Last);
//         Current.Buffer = NextName.Buffer;
//
//         /* Validate the current name string length */
//         if (Current.Length + NextName.Length > MAXUSHORT)
//         {
//             /* too long */
//             Status = STATUS_NAME_TOO_LONG;
//             goto Quickie;
//         }
//         Current.Length += NextName.Length;
//
//         /* Validate the current name string maximum length */
//         if (Current.MaximumLength + NextName.MaximumLength > MAXUSHORT)
//         {
//             /* too long */
//             Status = STATUS_NAME_TOO_LONG;
//             goto Quickie;
//         }
//         Current.MaximumLength += NextName.MaximumLength;
//
//         /* CmpGetSymbolicLink doesn't want a lock */
//         CmpUnLockKcbArray(LockedKcbs);
//         LockedKcbs = NULL;
//
//         /* Parse the symlink */
//         if (CmpGetSymbolicLink(Hive,
//                                CompleteName,
//                                Kcb,
//                                &Current))
//         {
//             /* Symlink parse succeeded */
//             Status = STATUS_REPARSE;
//         }
//         else
//         {
//             /* Couldn't find symlink */
//             Status = STATUS_OBJECT_NAME_NOT_FOUND;
//         }
//
//         /* We're done */
//         goto Quickie;
//     }
//
//     /* Get the key node */
//     Node = (PCM_KEY_NODE)HvGetCell(Hive, Cell);
//     if (!Node)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto Quickie;
//     }
//
//     /* Start parsing */
//     Status = STATUS_NOT_IMPLEMENTED;
//     while (TRUE)
//     {
//         /* Get the next component */
//         Result = CmpGetNextName(&Current, &NextName, &Last);
//         if (Result && NextName.Length)
//         {
//             /* See if this is a sym link */
//             if (!(Kcb->Flags & KEY_SYM_LINK))
//             {
//                 /* Find the subkey */
//                 NextCell = CmpFindSubKeyByName(Hive, Node, &NextName);
//                 if (NextCell != HCELL_NIL)
//                 {
//                     /* Get the new node */
//                     Cell = NextCell;
//                     Node = (PCM_KEY_NODE)HvGetCell(Hive, Cell);
//                     ASSERT(Node);
//
//                     /* Check if this was the last key */
//                     if (Last)
//                     {
//                         /* Is this an exit node */
//                         if (Node->Flags & KEY_HIVE_EXIT)
//                         {
//                             /* Handle it */
//                             CmpHandleExitNode(&Hive,
//                                               &Cell,
//                                               &Node,
//                                               &HiveToRelease,
//                                               &CellToRelease);
//                             if (!Node)
//                             {
//                                 /* Fail */
//                                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                                 break;
//                             }
//                         }
//
// KeyCachedOpenNow:
//                         /* Do the open */
//                         Status = CmpDoOpen(Hive,
//                                            Cell,
//                                            Node,
//                                            AccessState,
//                                            AccessMode,
//                                            Attributes,
//                                            ParseContext,
//                                            IsKeyCached ? CMP_OPEN_KCB_NO_CREATE : CMP_CREATE_KCB_KCB_LOCKED,
//                                            &Kcb,
//                                            LockedKcbs,
//                                            &NextName,
//                                            Object);
//                         if (Status == STATUS_REPARSE)
//                         {
//                             /* CmpGetSymbolicLink doesn't want a lock */
//                             CmpUnLockKcbArray(LockedKcbs);
//                             LockedKcbs = NULL;
//
//                             /* Parse the symlink */
//                             if (!CmpGetSymbolicLink(Hive,
//                                                     CompleteName,
//                                                     Kcb,
//                                                     NULL))
//                             {
//                                 /* Symlink parse failed */
//                                 Status = STATUS_OBJECT_NAME_NOT_FOUND;
//                             }
//                         }
//
//                         /* We are done */
//                         break;
//                     }
//
//                     /* Is this an exit node */
//                     if (Node->Flags & KEY_HIVE_EXIT)
//                     {
//                         /* Handle it */
//                         CmpHandleExitNode(&Hive,
//                                           &Cell,
//                                           &Node,
//                                           &HiveToRelease,
//                                           &CellToRelease);
//                         if (!Node)
//                         {
//                             /* Fail */
//                             Status = STATUS_INSUFFICIENT_RESOURCES;
//                             break;
//                         }
//                     }
//
//                     /* Create a KCB for this key */
//                     Kcb = CmpCreateKeyControlBlock(Hive,
//                                                    Cell,
//                                                    Node,
//                                                    ParentKcb,
//                                                    CMP_LOCK_HASHES_FOR_KCB,
//                                                    &NextName);
//                     if (!Kcb)
//                     {
//                         /* Fail */
//                         Status = STATUS_INSUFFICIENT_RESOURCES;
//                         break;
//                     }
//
//                     /* Dereference the parent and set the new one */
//                     CmpDereferenceKeyControlBlockWithLock(ParentKcb, FALSE);
//                     ParentKcb = Kcb;
//                 }
//                 else
//                 {
//                     /* Check if this was the last key for a create */
//                     if (Last && ParseContext)
//                     {
//                         /* Check if we're doing a link node */
//                         if (ParseContext->CreateLink)
//                         {
//                             /* The only thing we should see */
//                             Status = CmpCreateLinkNode(Hive,
//                                                        Cell,
//                                                        AccessState,
//                                                        NextName,
//                                                        AccessMode,
//                                                        Attributes,
//                                                        ParseContext,
//                                                        ParentKcb,
//                                                        LockedKcbs,
//                                                        Object);
//                         }
//                         else if (Hive == &CmiVolatileHive->Hive && CmpNoVolatileCreates)
//                         {
//                             /* Creating keys in the master hive is not allowed */
//                             Status = STATUS_INVALID_PARAMETER;
//                         }
//                         else
//                         {
//                             /* Do the create */
//                             Status = CmpDoCreate(Hive,
//                                                  Cell,
//                                                  AccessState,
//                                                  &NextName,
//                                                  AccessMode,
//                                                  ParseContext,
//                                                  ParentKcb,
//                                                  Object);
//                         }
//
//                         /* Check for reparse (in this case, someone beat us) */
//                         if (Status == STATUS_REPARSE) break;
//
//                         /* Update disposition */
//                         ParseContext->Disposition = REG_CREATED_NEW_KEY;
//                         break;
//                     }
//                     else
//                     {
//                         /* Key not found */
//                         Status = STATUS_OBJECT_NAME_NOT_FOUND;
//                         break;
//                     }
//                 }
//             }
//             else
//             {
//                 /* Save the next name */
//                 Current.Buffer = NextName.Buffer;
//
//                 /* Validate the current name string length */
//                 if (Current.Length + NextName.Length > MAXUSHORT)
//                 {
//                     /* too long */
//                     Status = STATUS_NAME_TOO_LONG;
//                     break;
//                 }
//                 Current.Length += NextName.Length;
//
//                 /* Validate the current name string maximum length */
//                 if (Current.MaximumLength + NextName.MaximumLength > MAXUSHORT)
//                 {
//                     /* too long */
//                     Status = STATUS_NAME_TOO_LONG;
//                     break;
//                 }
//                 Current.MaximumLength += NextName.MaximumLength;
//
//                 /* CmpGetSymbolicLink doesn't want a lock */
//                 CmpUnLockKcbArray(LockedKcbs);
//                 LockedKcbs = NULL;
//
//                 /* Parse the symlink */
//                 if (CmpGetSymbolicLink(Hive,
//                                        CompleteName,
//                                        Kcb,
//                                        &Current))
//                 {
//                     /* Symlink parse succeeded */
//                     Status = STATUS_REPARSE;
//                 }
//                 else
//                 {
//                     /* Couldn't find symlink */
//                     Status = STATUS_OBJECT_NAME_NOT_FOUND;
//                 }
//
//                 /* We're done */
//                 break;
//             }
//         }
//         else if (Result && Last)
//         {
//             /* Opening the root. Is this an exit node? */
//             if (Node->Flags & KEY_HIVE_EXIT)
//             {
//                 /* Handle it */
//                 CmpHandleExitNode(&Hive,
//                                   &Cell,
//                                   &Node,
//                                   &HiveToRelease,
//                                   &CellToRelease);
//                 if (!Node)
//                 {
//                     /* Fail */
//                     Status = STATUS_INSUFFICIENT_RESOURCES;
//                     break;
//                 }
//             }
//
//             /* Do the open */
//             Status = CmpDoOpen(Hive,
//                                Cell,
//                                Node,
//                                AccessState,
//                                AccessMode,
//                                Attributes,
//                                ParseContext,
//                                CMP_OPEN_KCB_NO_CREATE,
//                                &Kcb,
//                                LockedKcbs,
//                                &NextName,
//                                Object);
//             if (Status == STATUS_REPARSE)
//             {
//                 /* Nothing to do */
//             }
//
//             /* We're done */
//             break;
//         }
//         else
//         {
//             /* Bogus */
//             Status = STATUS_INVALID_PARAMETER;
//             break;
//         }
//     }
//
// Quickie:
//     /* Unlock all the KCBs */
//     if (LockedKcbs != NULL)
//     {
//         CmpUnLockKcbArray(LockedKcbs);
//     }
//
//     /* Dereference the parent if it exists */
//     if (ParentKcb)
//         CmpDereferenceKeyControlBlock(ParentKcb);
//
//     /* Unlock the registry */
//     CmpUnlockRegistry();
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
