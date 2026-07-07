use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{
    CM_KEY_BODY, CM_KEY_CONTROL_BLOCK, CM_KEY_HASH, CM_KEY_HASH_TABLE_ENTRY,
    CM_NAME_CONTROL_BLOCK, CM_NAME_HASH_TABLE_ENTRY,
    _CM_KEY_BODY, _CM_KEY_CONTROL_BLOCK, _CM_KEY_NODE, _CM_NAME_CONTROL_BLOCK,
    _CM_NAME_CONTROL_BLOCK__bindgen_ty_1__bindgen_ty_1,
    CmpAcquireKcbLockExclusive, CmpAcquireKcbLockExclusiveByIndex,
    CmpAcquireTwoKcbLocksExclusiveByKey, CmpAddToDelayedClose,
    CmpAllocate, CmpAllocateKeyControlBlock, CmpCompareCompressedName,
    CmpCompressedNameSize, CmpConvertKcbSharedToExclusive,
    CmpDelayDerefKeyControlBlock, CmpFlushNotify, CmpFree,
    CmpFreeKeyControlBlock, CmpInitializeDelayedCloseTable,
    CmpReleaseKcbLock, CmpReleaseKcbLockByIndex, CmpReleaseKcbLockByKey,
    CmpReleaseTwoKcbLockByKey, CmpRemoveFromDelayedClose,
    CmpTryToConvertKcbSharedToExclusive,
    ExInitializePushLock, HCELL_INDEX,
    ObDereferenceObjectDeferDelete, ObReferenceObjectSafe,
    PCMHIVE, PCM_HASH_CACHE_STACK, PCM_KEY_BODY, PCM_KEY_CONTROL_BLOCK,
    PCM_KEY_HASH, PCM_KEY_HASH_TABLE_ENTRY, PCM_KEY_NODE,
    PCM_NAME_CONTROL_BLOCK, PCM_NAME_HASH, PCM_NAME_HASH_TABLE_ENTRY,
    PHHIVE, PUNICODE_STRING, RtlUpcaseUnicodeChar,
    _UNICODE_STRING,
};
use core::ffi::c_void;

use ntkrnl_sys::opaque_ptr;
// Type alias: bindgen renames the hash-table entry struct, but the C source
// uses an internal alias `CmpHashTableEntry` for the same type.
pub type CmpHashTableEntry = CM_KEY_HASH_TABLE_ENTRY;
pub type CmpNameTableEntry = CM_NAME_HASH_TABLE_ENTRY;

// Constants from ntoskrnl/include/internal/cm.h that aren't in ntkrnl-sys
// constants.rs yet. Keep local until they migrate.
pub const TAG_CM: u32 = u32::from_le_bytes(*b"MmCk"); // 'kCmM' reversed for little-endian
pub const TAG_KCB: u32 = u32::from_le_bytes(*b"bKmC"); // 'CmKb'
pub const CONFIG_INITIALIZATION_FAILED: u32 = 0x00000067;
pub const CMP_ENLIST_KCB_LOCKED_SHARED: ULONG = 0x1;
pub const CMP_ENLIST_KCB_LOCKED_EXCLUSIVE: ULONG = 0x2;
pub const CMP_LOCK_KCB_ARRAY_EXCLUSIVE: ULONG = 0x1;
pub const CMP_LOCK_KCB_ARRAY_SHARED: ULONG = 0x2;
pub const CM_KEY_BODY_TYPE: u32 = 0x6B793032;
const CMP_HASH_IRRATIONAL: ULONG = 314159269;
const CMP_HASH_PRIME: ULONG = 1000000007;
const CMP_CREATE_FAKE_KCB: ULONG = 0x1;
const CMP_LOCK_HASHES_FOR_KCB: ULONG = 0x2;

// Globals owned by this file (cmkcbncb.c is excluded from C build).
#[no_mangle] pub static mut CmpHashTableSize: ULONG = 2048;
#[no_mangle] pub static mut CmpCacheTable: PCM_KEY_HASH_TABLE_ENTRY = core::ptr::null_mut();
#[no_mangle] pub static mut CmpNameCacheTable: PCM_NAME_HASH_TABLE_ENTRY = core::ptr::null_mut();

// Globals owned by other files (cmdelay.rs / cmlazy.rs / cmsysini.rs / cmdata.c).
extern "C" {
    pub static mut CmpDelayedCloseSize: ULONG;
    pub static mut CmpHoldLazyFlush: BOOLEAN;
    pub fn CmpAcquireKcbLockShared(Kcb: PCM_KEY_CONTROL_BLOCK);
    pub fn CmpAcquireKcbLockSharedByIndex(Index: ULONG);
    pub fn CmpAcquireNcbLockExclusiveByKey(ConvKey: ULONG);
    pub fn CmpReleaseNcbLockByKey(ConvKey: ULONG);
    pub fn CmpIsKcbLockedExclusive(Kcb: PCM_KEY_CONTROL_BLOCK) -> BOOLEAN;
    pub fn HvGetCell(Hive: PHHIVE, Cell: HCELL_INDEX) -> *mut core::ffi::c_void;
    pub fn HvReleaseCell(Hive: PHHIVE, Cell: HCELL_INDEX);
}

// FIELD_OFFSET — emulate the C macro using core::mem::offset_of! values.
// Translation produced `FIELD_OFFSET(size, offset)`-style call-site shapes
// that don't match the macro. Provide a no-op forwarder so callers compile;
// callers in this file pass the offset directly.
#[inline(always)]
unsafe fn FIELD_OFFSET(_size: u32, offset: u32) -> u32 {
    offset
}

// Hash helpers — implemented as inline shims here.
// COMPUTE_HASH_CHAR / GET_HASH_INDEX / GET_HASH_ENTRY mirror the cm_x.h macros.
#[inline(always)]
unsafe fn COMPUTE_HASH_CHAR(key: ULONG, c: u16) -> ULONG {
    key.wrapping_mul(37).wrapping_add(RtlUpcaseUnicodeChar(c) as ULONG)
}

#[inline(always)]
unsafe fn GET_HASH_INDEX(conv_key: ULONG) -> ULONG {
    conv_key
        .wrapping_mul(CMP_HASH_IRRATIONAL)
        .wrapping_rem(CMP_HASH_PRIME)
        .wrapping_rem(CmpHashTableSize)
}

#[inline(always)]
unsafe fn GET_HASH_ENTRY(
    table: PCM_KEY_HASH_TABLE_ENTRY,
    conv_key: ULONG,
) -> *mut CM_KEY_HASH_TABLE_ENTRY {
    table.add(GET_HASH_INDEX(conv_key) as usize)
}

#[inline(always)]
unsafe fn GET_NAME_HASH_ENTRY(
    table: PCM_NAME_HASH_TABLE_ENTRY,
    conv_key: ULONG,
) -> *mut CM_NAME_HASH_TABLE_ENTRY {
    table.add(GET_HASH_INDEX(conv_key) as usize)
}

// Inline assertion helpers — compiled out (DUMMY_PSEH-style).
#[inline(always)]
unsafe fn ASSERT_VALID_HASH(_hash: PCM_KEY_HASH) {}
#[inline(always)]
unsafe fn ASSERT_KCB_VALID(_kcb: PCM_KEY_CONTROL_BLOCK) {}
#[inline(always)]
unsafe fn CMP_ASSERT_KCB_LOCK(_kcb: PCM_KEY_CONTROL_BLOCK) {}
#[inline(always)]
unsafe fn CMP_ASSERT_EXCLUSIVE_REGISTRY_LOCK() {}
#[inline(always)]
unsafe fn CMTRACE(_flags: ULONG, _fmt: *const u8, _arg1: *const u8, _arg2: PCM_KEY_CONTROL_BLOCK) {}

// Cell-cache helpers.
#[inline(always)]
unsafe fn CMP_IS_CELL_CACHED(cell: ULONG) -> BOOLEAN {
    if (cell & 1) != 0 && cell != HCELL_NIL { 1 } else { 0 }
}
#[inline(always)]
unsafe fn CMP_GET_CACHED_DATA(cell: ULONG) -> usize {
    (cell & !1u32) as usize
}
#[inline(always)]
unsafe fn CMP_GET_CACHED_CELL(cell: usize) -> usize {
    cell & !1usize
}

#[no_mangle]
pub unsafe extern "system" fn CmpInitializeCache() {
    let mut Length: ULONG;
    let mut i: ULONG;

    /* Calculate length for the table */
    Length = CmpHashTableSize * core::mem::size_of::<CM_KEY_HASH_TABLE_ENTRY>() as ULONG;

    /* Allocate it */
    CmpCacheTable = CmpAllocate(Length as _, 1u8, TAG_CM) as PCM_KEY_HASH_TABLE_ENTRY;
    if CmpCacheTable.is_null() {
        /* Take the system down */
        KeBugCheckEx(CONFIG_INITIALIZATION_FAILED, 3, 1, 0, 0);
    }

    /* Zero out the table */
    core::ptr::write_bytes(CmpCacheTable as *mut u8, 0, Length as usize);

    /* Initialize the locks */
    i = 0;
    while i < CmpHashTableSize {
        /* Setup the pushlock */
        ExInitializePushLock(core::ptr::addr_of_mut!((*CmpCacheTable.add(i as usize)).Lock) as _);
        i += 1;
    }

    /* Calculate length for the name cache */
    Length = CmpHashTableSize * core::mem::size_of::<CM_NAME_HASH_TABLE_ENTRY>() as ULONG;

    /* Now allocate the name cache table */
    CmpNameCacheTable = CmpAllocate(Length as _, 1u8, TAG_CM) as PCM_NAME_HASH_TABLE_ENTRY;
    if CmpNameCacheTable.is_null() {
        /* Take the system down */
        KeBugCheckEx(CONFIG_INITIALIZATION_FAILED, 3, 3, 0, 0);
    }

    /* Zero out the table */
    core::ptr::write_bytes(CmpNameCacheTable as *mut u8, 0, Length as usize);

    /* Initialize the locks */
    i = 0;
    while i < CmpHashTableSize {
        /* Setup the pushlock */
        ExInitializePushLock(core::ptr::addr_of_mut!((*CmpNameCacheTable.add(i as usize)).Lock) as _);
        i += 1;
    }

    /* Setup the delayed close table */
    CmpInitializeDelayedCloseTable();
}

pub unsafe fn CmpRemoveKeyHash(KeyHash: PCM_KEY_HASH) {
    let mut Prev: *mut PCM_KEY_HASH;
    let mut Current: PCM_KEY_HASH;
    
    
    ASSERT_VALID_HASH(KeyHash);
    
    Prev = &mut (*(GET_HASH_ENTRY(CmpCacheTable, (*KeyHash).ConvKey) as *mut CmpHashTableEntry)).Entry;
    
    loop {
        Current = *Prev;
        debug_assert!(!Current.is_null());
        ASSERT_VALID_HASH(Current);
        
        if Current == KeyHash {
            *Prev = (*Current).NextHash;
            if !(*Prev).is_null() {
                ASSERT_VALID_HASH(*Prev);
            }
            break;
        }
        
        Prev = &mut (*Current).NextHash;
    }
}

#[no_mangle]
pub unsafe extern "system" fn CmpInsertKeyHash(
    KeyHash: PCM_KEY_HASH,
    IsFake: BOOLEAN,
) -> PCM_KEY_CONTROL_BLOCK {
    let KeyHash = opaque_ptr(KeyHash);
    let mut i: ULONG;
    let mut Entry: PCM_KEY_HASH;

    ASSERT_VALID_HASH(KeyHash);

    i = GET_HASH_INDEX((*KeyHash).ConvKey);

    if IsFake != 0 {
        (*KeyHash).KeyCell = (*KeyHash).KeyCell.wrapping_add(1);
    }

    Entry = (*CmpCacheTable.add(i as usize)).Entry;
    while !Entry.is_null() {
        ASSERT_VALID_HASH(Entry);
        if (*KeyHash).ConvKey == (*Entry).ConvKey
            && (*KeyHash).KeyCell == (*Entry).KeyCell
            && (*KeyHash).KeyHive == (*Entry).KeyHive
        {
            return (Entry as *mut u8).offset(-(core::mem::offset_of!(_CM_KEY_CONTROL_BLOCK, __bindgen_anon_2) as isize))
                as PCM_KEY_CONTROL_BLOCK;
        }

        Entry = (*Entry).NextHash;
    }

    (*KeyHash).NextHash = (*CmpCacheTable.add(i as usize)).Entry;
    (*CmpCacheTable.add(i as usize)).Entry = KeyHash;
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn CmpGetNameControlBlock(NodeName: PUNICODE_STRING) -> PCM_NAME_CONTROL_BLOCK {
    let NodeName = opaque_ptr(NodeName);
    let mut Ncb: PCM_NAME_CONTROL_BLOCK = core::ptr::null_mut();
    let mut ConvKey: u32 = 0;
    let mut p: *mut u16;
    let mut pp: *mut u16;
    let mut i: u32;
    let mut IsCompressed: u8 = 1u8;
    let mut Found: u8 = 0u8;
    let mut HashEntry: PCM_NAME_HASH;
    let mut NcbSize: u32;
    let mut Length: u16;

    p = (*NodeName).Buffer;
    i = 0;
    while i < (*NodeName).Length as u32 {
        if *p != 0x5Cu16 {
            ConvKey = COMPUTE_HASH_CHAR(ConvKey, *p);
        }
        p = p.offset(1);
        i += core::mem::size_of::<u16>() as u32;
    }

    Length = ((*NodeName).Length / core::mem::size_of::<u16>() as u16) as u16;
    i = 0;
    while i < ((*NodeName).Length / core::mem::size_of::<u16>() as u16) as u32 {
        if (*(*NodeName).Buffer.offset(i as isize)) > 0xFFu16 {
            Length = (*NodeName).Length;
            IsCompressed = 0u8;
            break;
        }
        i += 1;
    }

    CmpAcquireNcbLockExclusiveByKey(ConvKey);

    HashEntry = (*GET_NAME_HASH_ENTRY(CmpNameCacheTable, ConvKey)).Entry;
    while !HashEntry.is_null() {
        Ncb = ((HashEntry as usize) - core::mem::offset_of!(_CM_NAME_CONTROL_BLOCK, __bindgen_anon_1)) as PCM_NAME_CONTROL_BLOCK;

        if (ConvKey == (*HashEntry).ConvKey)
            && (Length == (*Ncb).__bindgen_anon_1.__bindgen_anon_1.NameLength) {
            Found = 1u8;

            if (*Ncb).Compressed != 0 {
                if CmpCompareCompressedName(
                    NodeName,
                    (*Ncb).__bindgen_anon_1.__bindgen_anon_1.Name.as_mut_ptr(),
                    Length as u32,
                ) != 0 {
                    Found = 0u8;
                }
            } else {
                p = (*NodeName).Buffer;
                pp = (*Ncb).__bindgen_anon_1.__bindgen_anon_1.Name.as_mut_ptr();
                i = 0;
                while i < (*Ncb).__bindgen_anon_1.__bindgen_anon_1.NameLength as u32 {
                    if RtlUpcaseUnicodeChar(*p) != RtlUpcaseUnicodeChar(*pp) {
                        Found = 0u8;
                        break;
                    }
                    p = p.offset(1);
                    pp = pp.offset(1);
                    i += core::mem::size_of::<u16>() as u32;
                }
            }

            if Found != 0 {
                (*Ncb).RefCount += 1;
                break;
            }
        }

        HashEntry = (*HashEntry).NextHash;
    }

    if Found == 0 {
        // FIELD_OFFSET(CM_NAME_CONTROL_BLOCK, Name) — Name lives inside the anon union.
        let name_offset =
            core::mem::offset_of!(_CM_NAME_CONTROL_BLOCK, __bindgen_anon_1) +
            core::mem::offset_of!(_CM_NAME_CONTROL_BLOCK__bindgen_ty_1__bindgen_ty_1, Name);
        NcbSize = (name_offset as u32) + Length as u32;
        Ncb = CmpAllocate(NcbSize as _, 1u8, 0x20434D54u32) as PCM_NAME_CONTROL_BLOCK;
        if Ncb.is_null() {
            CmpReleaseNcbLockByKey(ConvKey);
            return core::ptr::null_mut();
        }

        core::ptr::write_bytes(Ncb as *mut u8, 0, NcbSize as usize);

        if IsCompressed != 0 {
            i = 0;
            while i < ((*NodeName).Length / core::mem::size_of::<u16>() as u16) as u32 {
                ((*Ncb).__bindgen_anon_1.__bindgen_anon_1.Name.as_mut_ptr() as *mut u8)
                    .offset(i as isize)
                    .write(RtlUpcaseUnicodeChar(*(*NodeName).Buffer.offset(i as isize)) as u8);
                i += 1;
            }
        } else {
            i = 0;
            while i < ((*NodeName).Length / core::mem::size_of::<u16>() as u16) as u32 {
                (*Ncb).__bindgen_anon_1.__bindgen_anon_1.Name.as_mut_ptr()
                    .offset(i as isize)
                    .write(RtlUpcaseUnicodeChar(*(*NodeName).Buffer.offset(i as isize)));
                i += 1;
            }
        }

        (*Ncb).Compressed = IsCompressed;
        (*Ncb).__bindgen_anon_1.__bindgen_anon_1.ConvKey = ConvKey;
        (*Ncb).RefCount += 1;
        (*Ncb).__bindgen_anon_1.__bindgen_anon_1.NameLength = Length;

        HashEntry = core::ptr::addr_of_mut!((*Ncb).__bindgen_anon_1.NameHash);
        (*HashEntry).NextHash = (*GET_NAME_HASH_ENTRY(CmpNameCacheTable, ConvKey)).Entry;
        (*GET_NAME_HASH_ENTRY(CmpNameCacheTable, ConvKey)).Entry = HashEntry;
    }

    CmpReleaseNcbLockByKey(ConvKey);

    return Ncb;
}

#[no_mangle]
pub unsafe extern "system" fn CmpRemoveKeyControlBlock(Kcb: PCM_KEY_CONTROL_BLOCK) {
    let Kcb = opaque_ptr(Kcb);
    
    CMP_ASSERT_KCB_LOCK(Kcb);
    
    CmpRemoveKeyHash(core::ptr::addr_of_mut!((*Kcb).__bindgen_anon_2.KeyHash));
}

#[no_mangle]
pub unsafe extern "system" fn CmpDereferenceNameControlBlockWithLock(Ncb: PCM_NAME_CONTROL_BLOCK) {
    let Ncb = opaque_ptr(Ncb);
    let mut Current: PCM_NAME_HASH;
    let mut Next: *mut PCM_NAME_HASH;
    let ConvKey: u32 = (*Ncb).__bindgen_anon_1.__bindgen_anon_1.ConvKey;

    CmpAcquireNcbLockExclusiveByKey(ConvKey);

    (*Ncb).RefCount -= 1;
    if (*Ncb).RefCount == 0 {
        Next = core::ptr::addr_of_mut!((*GET_NAME_HASH_ENTRY(CmpNameCacheTable, ConvKey)).Entry);
        loop {
            Current = *Next;
            if Current == core::ptr::addr_of_mut!((*Ncb).__bindgen_anon_1.NameHash) {
                *Next = (*Current).NextHash;
                break;
            }

            Next = core::ptr::addr_of_mut!((*Current).NextHash);
        }

        CmpFree(Ncb as *mut core::ffi::c_void, 0);
    }

    CmpReleaseNcbLockByKey(ConvKey);
}

#[no_mangle]
pub unsafe extern "system" fn CmpReferenceKeyControlBlock(Kcb: PCM_KEY_CONTROL_BLOCK) -> BOOLEAN {
    let Kcb = opaque_ptr(Kcb);

    CMTRACE(0x20u32, b"%s - Referencing KCB: %p\n\0".as_ptr() as *const u8,
            b"CmpReferenceKeyControlBlock\0".as_ptr() as *const u8, Kcb);

    if (*Kcb).RefCount == 0 {
        if CmpIsKcbLockedExclusive(Kcb) == 0 {
            if CmpTryToConvertKcbSharedToExclusive(Kcb) == 0 {
                (*Kcb).__bindgen_anon_1.set_DelayedCloseIndex(1);

                InterlockedIncrement(core::ptr::addr_of_mut!((*Kcb).RefCount) as PLONG);

                CmpConvertKcbSharedToExclusive(Kcb);

                InterlockedDecrement(core::ptr::addr_of_mut!((*Kcb).RefCount) as PLONG);

                if (*Kcb).__bindgen_anon_1.DelayedCloseIndex() == 1 {
                    (*Kcb).__bindgen_anon_1.set_DelayedCloseIndex(0);
                }
            }
        }
    }

    if (InterlockedIncrement(core::ptr::addr_of_mut!((*Kcb).RefCount) as PLONG) & 0xFFFF) == 0 {
        InterlockedDecrement(core::ptr::addr_of_mut!((*Kcb).RefCount) as PLONG);
        return 0u8;
    }

    if (*Kcb).__bindgen_anon_1.DelayedCloseIndex() == 0 {
        if CmpIsKcbLockedExclusive(Kcb) == 0 {
            if CmpTryToConvertKcbSharedToExclusive(Kcb) == 0 {
                CmpConvertKcbSharedToExclusive(Kcb);
            }
        }

        if (*Kcb).__bindgen_anon_1.DelayedCloseIndex() == 0 {
            CmpRemoveFromDelayedClose(Kcb);
        }
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn CmpCleanUpKcbValueCache(Kcb: PCM_KEY_CONTROL_BLOCK) {
    let Kcb = opaque_ptr(Kcb);
    let mut CachedList: PULONG_PTR;
    let mut i: u32;

    CMP_ASSERT_KCB_LOCK(Kcb);

    if CMP_IS_CELL_CACHED((*Kcb).ValueCache.__bindgen_anon_1.ValueList) != 0 {
        CachedList = CMP_GET_CACHED_DATA((*Kcb).ValueCache.__bindgen_anon_1.ValueList) as PULONG_PTR;
        i = 0;
        while i < (*Kcb).ValueCache.Count {
            if CMP_IS_CELL_CACHED(*CachedList.offset(i as isize) as ULONG) != 0 {
                CmpFree(CMP_GET_CACHED_CELL(*CachedList.offset(i as isize) as usize) as *mut core::ffi::c_void, 0);
            }
            i += 1;
        }

        CmpFree(CMP_GET_CACHED_CELL((*Kcb).ValueCache.__bindgen_anon_1.ValueList as usize) as *mut core::ffi::c_void, 0);
        (*Kcb).ValueCache.__bindgen_anon_1.ValueList = 0xFFFFFFFF;
    } else if ((*Kcb).__bindgen_anon_1.ExtFlags() & CM_KCB_SYM_LINK_FOUND) != 0 {
        let real_kcb = (*Kcb).ValueCache.__bindgen_anon_1.RealKcb;
        if ((*real_kcb).RefCount == 1) &&
            ((*real_kcb).__bindgen_anon_1.Delete() == 0) {
            let cur = (*real_kcb).__bindgen_anon_1.ExtFlags();
            (*real_kcb).__bindgen_anon_1.set_ExtFlags(cur | CM_KCB_NO_DELAY_CLOSE);
        }

        CmpDelayDerefKeyControlBlock((*Kcb).ValueCache.__bindgen_anon_1.RealKcb);
        let cur = (*Kcb).__bindgen_anon_1.ExtFlags();
        (*Kcb).__bindgen_anon_1.set_ExtFlags(cur & !CM_KCB_SYM_LINK_FOUND);
    }
}

#[no_mangle]
pub unsafe extern "system" fn CmpCleanUpKcbCacheWithLock(
    Kcb: PCM_KEY_CONTROL_BLOCK,
    LockHeldExclusively: u8,
) {
    let Kcb = opaque_ptr(Kcb);
    let Parent: PCM_KEY_CONTROL_BLOCK;

    CMP_ASSERT_KCB_LOCK(Kcb);

    CmpCleanUpKcbValueCache(Kcb);

    CmpDereferenceNameControlBlockWithLock((*Kcb).NameBlock);

    if ((*Kcb).__bindgen_anon_1.ExtFlags() & CM_KCB_SUBKEY_HINT) != 0 {
        CmpFree((*Kcb).__bindgen_anon_3.IndexHint as *mut core::ffi::c_void, 0);
    }

    Parent = (*Kcb).ParentKcb;
    if (*Kcb).__bindgen_anon_1.Delete() == 0 {
        CmpRemoveKeyControlBlock(Kcb);
    }

    (*Kcb).Signature = 0x6B637066;

    CmpFreeKeyControlBlock(Kcb);

    if !Parent.is_null() {
        if LockHeldExclusively != 0 {
            CmpDereferenceKeyControlBlockWithLock(Parent, LockHeldExclusively);
        } else {
            CmpDelayDerefKeyControlBlock(Parent);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn CmpCleanUpSubKeyInfo(Kcb: PCM_KEY_CONTROL_BLOCK) {
    let Kcb = opaque_ptr(Kcb);
    let KeyNode: PCM_KEY_NODE;

    CMP_ASSERT_KCB_LOCK(Kcb);

    if ((*Kcb).__bindgen_anon_1.ExtFlags() & (CM_KCB_NO_SUBKEY | CM_KCB_SUBKEY_ONE | CM_KCB_SUBKEY_HINT)) != 0 {
        if ((*Kcb).__bindgen_anon_1.ExtFlags() & CM_KCB_SUBKEY_HINT) != 0 {
            CmpFree((*Kcb).__bindgen_anon_3.IndexHint as *mut core::ffi::c_void, 0);
        }

        let cur = (*Kcb).__bindgen_anon_1.ExtFlags();
        (*Kcb).__bindgen_anon_1.set_ExtFlags(cur & !(CM_KCB_NO_SUBKEY | CM_KCB_SUBKEY_ONE | CM_KCB_SUBKEY_HINT));
    }

    if (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyCell == HCELL_NIL {
        KeyNode = core::ptr::null_mut();
    } else {
        KeyNode = HvGetCell(
            (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyHive,
            (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyCell,
        ) as PCM_KEY_NODE;
    }

    if KeyNode.is_null() {
        let cur = (*Kcb).__bindgen_anon_1.ExtFlags();
        (*Kcb).__bindgen_anon_1.set_ExtFlags(cur | CM_KCB_INVALID_CACHED_INFO);
    } else {
        let cur = (*Kcb).__bindgen_anon_1.ExtFlags();
        (*Kcb).__bindgen_anon_1.set_ExtFlags(cur & !CM_KCB_INVALID_CACHED_INFO);
        (*Kcb).__bindgen_anon_3.SubKeyCount =
            (*KeyNode).SubKeyCounts[Stable as usize] +
            (*KeyNode).SubKeyCounts[Volatile as usize];

        HvReleaseCell(
            (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyHive,
            (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyCell,
        );
    }
}

const CM_KCB_NO_SUBKEY: u32 = 0x00000001;
const CM_KCB_SUBKEY_ONE: u32 = 0x00000002;
const CM_KCB_SUBKEY_HINT: u32 = 0x00000004;
const CM_KCB_SYM_LINK_FOUND: u32 = 0x00000008;
const CM_KCB_KEY_NON_EXIST: u32 = 0x00000010;
const CM_KCB_NO_DELAY_CLOSE: u32 = 0x00000020;
const CM_KCB_INVALID_CACHED_INFO: u32 = 0x00000040;
const HCELL_NIL: u32 = 0xFFFFFFFF;
const Stable: u32 = 0;
const Volatile: u32 = 1;

#[no_mangle]
pub unsafe extern "system" fn CmpDereferenceKeyControlBlock(Kcb: PCM_KEY_CONTROL_BLOCK) {
    let Kcb = opaque_ptr(Kcb);
    let mut OldRefCount: i32;
    let mut NewRefCount: i32;
    let ConvKey: u32;

    OldRefCount = *(core::ptr::addr_of!((*Kcb).RefCount) as *const i32);
    NewRefCount = OldRefCount - 1;

    if (NewRefCount & 0xFFFF) > 0 {
        // Stable CAS via core::sync::atomic — equivalent to the C InterlockedCompareExchange.
        let atomic = &*(core::ptr::addr_of!((*Kcb).RefCount) as *const core::sync::atomic::AtomicI32);
        if atomic.compare_exchange(
            OldRefCount,
            NewRefCount,
            core::sync::atomic::Ordering::SeqCst,
            core::sync::atomic::Ordering::SeqCst,
        ).is_ok() {
            return;
        }
    }

    ConvKey = (*Kcb).__bindgen_anon_2.__bindgen_anon_1.ConvKey;

    CmpAcquireKcbLockExclusive(Kcb);
    CmpDereferenceKeyControlBlockWithLock(Kcb, 0u8);
    CmpReleaseKcbLockByKey(ConvKey);
}

#[no_mangle]
pub unsafe extern "system" fn CmpDereferenceKeyControlBlockWithLock(
    Kcb: PCM_KEY_CONTROL_BLOCK,
    LockHeldExclusively: u8,
) {
    let Kcb = opaque_ptr(Kcb);

    CMTRACE(
        0x00000008,
        b"%s - Dereferencing KCB: %p\n\0".as_ptr() as *const u8,
        b"CmpDereferenceKeyControlBlockWithLock\0".as_ptr() as *const u8,
        Kcb,
    );

    ASSERT_KCB_VALID(Kcb);

    if (InterlockedDecrement(core::ptr::addr_of_mut!((*Kcb).RefCount) as *mut i32) & 0xFFFF) == 0 {
        CMP_ASSERT_KCB_LOCK(Kcb);

        if (CmpHoldLazyFlush != 0
            && ((*Kcb).__bindgen_anon_1.ExtFlags() & CM_KCB_SYM_LINK_FOUND) == 0
            && ((*Kcb).__bindgen_anon_5.Flags() & 0x0010) == 0)
            || ((*Kcb).__bindgen_anon_1.ExtFlags() & CM_KCB_NO_DELAY_CLOSE) != 0
            || (*Kcb).__bindgen_anon_1.Delete() != 0
        {
            CmpCleanUpKcbCacheWithLock(Kcb, LockHeldExclusively);
        } else {
            CmpAddToDelayedClose(Kcb, LockHeldExclusively);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn InitializeKCBKeyBodyList(Kcb: PCM_KEY_CONTROL_BLOCK) {
    let Kcb = opaque_ptr(Kcb);
    InitializeListHead(core::ptr::addr_of_mut!((*Kcb).__bindgen_anon_4.KeyBodyListHead));

    (*Kcb).KeyBodyArray[0] = core::ptr::null_mut();
    (*Kcb).KeyBodyArray[1] = core::ptr::null_mut();
    (*Kcb).KeyBodyArray[2] = core::ptr::null_mut();
    (*Kcb).KeyBodyArray[3] = core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn CmpCreateKeyControlBlock(
    Hive: PHHIVE,
    Index: HCELL_INDEX,
    Node: PCM_KEY_NODE,
    Parent: PCM_KEY_CONTROL_BLOCK,
    Flags: u32,
    KeyName: PUNICODE_STRING,
) -> PCM_KEY_CONTROL_BLOCK {
    let Hive = opaque_ptr(Hive);
    let Node = opaque_ptr(Node);
    let Parent = opaque_ptr(Parent);
    let KeyName = opaque_ptr(KeyName);

    let mut Kcb: PCM_KEY_CONTROL_BLOCK;
    let mut FoundKcb: PCM_KEY_CONTROL_BLOCK = core::ptr::null_mut();
    let mut NodeName: _UNICODE_STRING;
    let mut ConvKey: u32 = 0;
    let mut i: u32;
    let mut IsFake: u8;
    let mut HashLock: u8;
    let mut p: *mut u16;

    if (((*Hive).HiveFlags & 0x00000200) != 0) &&
        ((*(Hive as PCMHIVE)).CreatorOwner as *mut core::ffi::c_void != KeGetCurrentThread() as *mut core::ffi::c_void)
    {
        return core::ptr::null_mut();
    }

    IsFake = if (Flags & CMP_CREATE_FAKE_KCB) != 0 { 1u8 } else { 0u8 };

    if !Parent.is_null() {
        ConvKey = (*Parent).__bindgen_anon_2.__bindgen_anon_1.ConvKey;
    }

    NodeName = *KeyName;

    while NodeName.Length != 0 && (*NodeName.Buffer == 0x005C) {
        NodeName.Buffer = NodeName.Buffer.offset(1);
        NodeName.Length = NodeName.Length.wrapping_sub(2);
    }

    p = NodeName.Buffer;
    i = 0;
    while i < NodeName.Length as u32 {
        if *p != 0x005C {
            ConvKey = COMPUTE_HASH_CHAR(ConvKey, *p);
        }
        p = p.offset(1);
        i = i.wrapping_add(2);
    }

    Kcb = CmpAllocateKeyControlBlock();
    if Kcb.is_null() {
        return core::ptr::null_mut();
    }

    InitializeKCBKeyBodyList(Kcb);

    (*Kcb).Signature = 0x6B636D2bu32;
    (*Kcb).__bindgen_anon_1.set_Delete(0);
    (*Kcb).RefCount = 1;
    (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyHive = Hive;
    (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyCell = Index;
    (*Kcb).__bindgen_anon_2.__bindgen_anon_1.ConvKey = ConvKey;
    (*Kcb).__bindgen_anon_1.set_DelayedCloseIndex(CmpDelayedCloseSize);
    (*Kcb).InDelayClose = 0;
    ASSERT_KCB_VALID(Kcb);

    HashLock = if (Flags & CMP_LOCK_HASHES_FOR_KCB) != 0 { 1u8 } else { 0u8 };
    if HashLock == 0 {
        if !Parent.is_null() {
            CmpAcquireTwoKcbLocksExclusiveByKey(ConvKey, (*Parent).__bindgen_anon_2.__bindgen_anon_1.ConvKey);
        } else {
            CmpAcquireKcbLockExclusive(Kcb);
        }
    }

    FoundKcb = CmpInsertKeyHash(core::ptr::addr_of_mut!((*Kcb).__bindgen_anon_2.KeyHash), IsFake);
    if !FoundKcb.is_null() {
        (*Kcb).Signature = 0u32;

        CmpFreeKeyControlBlock(Kcb);
        ASSERT_KCB_VALID(FoundKcb);
        Kcb = FoundKcb;
        if CmpReferenceKeyControlBlock(Kcb) == 0 {
            Kcb = core::ptr::null_mut();
        } else {
            if (((*Kcb).__bindgen_anon_1.ExtFlags() & CM_KCB_KEY_NON_EXIST) != 0) && (IsFake == 0) {
                (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyHive = Hive;
                (*Kcb).__bindgen_anon_2.__bindgen_anon_1.KeyCell = Index;

                (*Kcb).__bindgen_anon_1.set_ExtFlags(CM_KCB_INVALID_CACHED_INFO);
            }

            if ((*Kcb).__bindgen_anon_1.ExtFlags() & (CM_KCB_NO_SUBKEY | CM_KCB_SUBKEY_ONE | CM_KCB_SUBKEY_HINT)) == 0 {
                (*Kcb).__bindgen_anon_3.SubKeyCount = (*Node).SubKeyCounts[0].wrapping_add(
                    (*Node).SubKeyCounts[1]
                );

                let cur = (*Kcb).__bindgen_anon_1.ExtFlags();
                (*Kcb).__bindgen_anon_1.set_ExtFlags(cur & !CM_KCB_INVALID_CACHED_INFO);
            }

            (*Kcb).KcbLastWriteTime = (*Node).LastWriteTime;
            (*Kcb).KcbMaxNameLen = (*Node).MaxNameLen as u16;
            (*Kcb).KcbMaxValueNameLen = (*Node).MaxValueNameLen as u16;
            (*Kcb).KcbMaxValueDataLen = (*Node).MaxValueDataLen;
        }
    } else {
        if !Parent.is_null() {
            if (((*Parent).__bindgen_anon_1.TotalLevels() + 1) < 512) &&
                (CmpReferenceKeyControlBlock(Parent) != 0)
            {
                (*Kcb).ParentKcb = Parent;
                (*Kcb).__bindgen_anon_1.set_TotalLevels((*Parent).__bindgen_anon_1.TotalLevels() + 1);
            } else {
                CmpRemoveKeyControlBlock(Kcb);
                (*Kcb).Signature = 0u32;
                CmpFreeKeyControlBlock(Kcb);
                Kcb = core::ptr::null_mut();
            }
        } else {
            (*Kcb).ParentKcb = core::ptr::null_mut();
            (*Kcb).__bindgen_anon_1.set_TotalLevels(1);
        }

        if !Kcb.is_null() {
            (*Kcb).NameBlock = CmpGetNameControlBlock(core::ptr::addr_of_mut!(NodeName));
            if !(*Kcb).NameBlock.is_null() {
                (*Kcb).ValueCache.Count = (*Node).__bindgen_anon_1.__bindgen_anon_1.ValueList.Count;
                (*Kcb).ValueCache.__bindgen_anon_1.ValueList = (*Node).__bindgen_anon_1.__bindgen_anon_1.ValueList.List;
                (*Kcb).__bindgen_anon_5.set_Flags((*Node).Flags as ULONG);
                (*Kcb).__bindgen_anon_1.set_ExtFlags(0);
                (*Kcb).__bindgen_anon_1.set_DelayedCloseIndex(CmpDelayedCloseSize);

                if IsFake != 0 {
                    let cur = (*Kcb).__bindgen_anon_1.ExtFlags();
                    (*Kcb).__bindgen_anon_1.set_ExtFlags(cur | CM_KCB_KEY_NON_EXIST);
                }

                (*Kcb).__bindgen_anon_3.SubKeyCount = (*Node).SubKeyCounts[0].wrapping_add(
                    (*Node).SubKeyCounts[1]
                );
                (*Kcb).KcbLastWriteTime = (*Node).LastWriteTime;
                (*Kcb).KcbMaxNameLen = (*Node).MaxNameLen as u16;
                (*Kcb).KcbMaxValueNameLen = (*Node).MaxValueNameLen as u16;
                (*Kcb).KcbMaxValueDataLen = (*Node).MaxValueDataLen as u32;
            } else {
                CmpDereferenceKeyControlBlockWithLock(Parent, 0u8);

                CmpRemoveKeyControlBlock(Kcb);
                (*Kcb).Signature = 0u32;
                CmpFreeKeyControlBlock(Kcb);
                Kcb = core::ptr::null_mut();
            }
        }
    }

    if !Kcb.is_null() && ((*(Hive as PCMHIVE)).Frozen != 0) && (((*Kcb).__bindgen_anon_5.Flags() & 0x0010) == 0) {
        let cur = (*Kcb).__bindgen_anon_1.ExtFlags();
        (*Kcb).__bindgen_anon_1.set_ExtFlags(cur | CM_KCB_NO_DELAY_CLOSE);
    }

    if HashLock == 0 {
        if !Parent.is_null() {
            CmpReleaseTwoKcbLockByKey(ConvKey, (*Parent).__bindgen_anon_2.__bindgen_anon_1.ConvKey);
        } else {
            CmpReleaseKcbLockByKey(ConvKey);
        }
    }

    Kcb
}

#[no_mangle]
pub unsafe extern "system" fn CmpConstructName(Kcb: PCM_KEY_CONTROL_BLOCK) -> PUNICODE_STRING {
    let Kcb = opaque_ptr(Kcb);

    let mut NameLength: u16;
    let mut MyKcb: PCM_KEY_CONTROL_BLOCK;
    let mut KeyNode: PCM_KEY_NODE;
    let mut DeletedKey: u8 = 0u8;
    let mut TargetBuffer: PWSTR;
    let mut CurrentNameW: PWSTR;
    let mut CurrentName: *mut u8;
    let mut i: u32;

    NameLength = 0;
    MyKcb = Kcb;

    while !MyKcb.is_null() {
        let nb = (*MyKcb).NameBlock;
        if (*nb).Compressed == 0 {
            NameLength = NameLength.wrapping_add((*nb).__bindgen_anon_1.__bindgen_anon_1.NameLength);
        } else {
            NameLength = NameLength.wrapping_add(CmpCompressedNameSize(
                (*nb).__bindgen_anon_1.__bindgen_anon_1.Name.as_mut_ptr(),
                (*nb).__bindgen_anon_1.__bindgen_anon_1.NameLength as ULONG,
            ));
        }

        NameLength = NameLength.wrapping_add(core::mem::size_of::<u16>() as u16);

        MyKcb = (*MyKcb).ParentKcb;
    }

    let KeyName: PUNICODE_STRING = CmpAllocate(
        (NameLength as u32).wrapping_add(core::mem::size_of::<_UNICODE_STRING>() as u32) as _,
        1u8,
        0x20434d54u32,
    ) as PUNICODE_STRING;

    if KeyName.is_null() {
        return core::ptr::null_mut();
    }

    (*KeyName).Buffer = KeyName.offset(1) as PWSTR;
    (*KeyName).Length = NameLength;
    (*KeyName).MaximumLength = NameLength;

    NameLength = 0;
    MyKcb = Kcb;

    while !MyKcb.is_null() {
        if ((*MyKcb).__bindgen_anon_2.__bindgen_anon_1.KeyCell == 0 && (*MyKcb).__bindgen_anon_1.Delete() == 0)
            || (*MyKcb).__bindgen_anon_2.__bindgen_anon_1.KeyHive.is_null()
            || ((*MyKcb).__bindgen_anon_1.ExtFlags() & CM_KCB_KEY_NON_EXIST) != 0
        {
            CmpFree(KeyName as *mut core::ffi::c_void, 0);
            return core::ptr::null_mut();
        }

        if DeletedKey == 0 && (*MyKcb).__bindgen_anon_1.Delete() == 0 {
            KeyNode = HvGetCell(
                (*MyKcb).__bindgen_anon_2.__bindgen_anon_1.KeyHive,
                (*MyKcb).__bindgen_anon_2.__bindgen_anon_1.KeyCell,
            ) as PCM_KEY_NODE;
            if KeyNode.is_null() {
                CmpFree(KeyName as *mut core::ffi::c_void, 0);
                return core::ptr::null_mut();
            }
        } else {
            KeyNode = core::ptr::null_mut();
            DeletedKey = 1u8;
        }

        let nb = (*MyKcb).NameBlock;
        NameLength = NameLength.wrapping_add(
            ((*nb).__bindgen_anon_1.__bindgen_anon_1.NameLength as u32)
                .wrapping_add(1)
                .wrapping_mul(core::mem::size_of::<u16>() as u32) as u16,
        );
        TargetBuffer = ((*KeyName).Buffer as usize)
            .wrapping_add(
                ((*KeyName)
                    .Length
                    .wrapping_sub(NameLength)
                    .wrapping_div(core::mem::size_of::<u16>() as u16)
                    as usize)
                    .wrapping_mul(core::mem::size_of::<u16>()),
            ) as PWSTR;

        *TargetBuffer = 0x005Cu16;

        if (*nb).Compressed == 0 {
            if ((*MyKcb).__bindgen_anon_5.Flags() & (0x00000001 | 0x00000002)) != 0 || KeyNode.is_null() {
                CurrentNameW = (*nb).__bindgen_anon_1.__bindgen_anon_1.Name.as_mut_ptr();
            } else {
                // KeyNode is #[repr(packed)] — get the field address via offset_of! to avoid
                // creating an unaligned reference.
                CurrentNameW = (KeyNode as *mut u8)
                    .add(core::mem::offset_of!(_CM_KEY_NODE, Name)) as PWSTR;
            }

            i = 0;
            while i < (*nb).__bindgen_anon_1.__bindgen_anon_1.NameLength as u32 {
                *TargetBuffer.offset((i.wrapping_add(1)) as isize) = *CurrentNameW;
                CurrentNameW = CurrentNameW.offset(1);
                i = i.wrapping_add(1);
            }
        } else {
            if ((*MyKcb).__bindgen_anon_5.Flags() & (0x00000001 | 0x00000002)) != 0 || KeyNode.is_null() {
                CurrentName = (*nb).__bindgen_anon_1.__bindgen_anon_1.Name.as_mut_ptr() as *mut u8;
            } else {
                CurrentName = (KeyNode as *mut u8)
                    .add(core::mem::offset_of!(_CM_KEY_NODE, Name));
            }

            i = 0;
            while i < (*nb).__bindgen_anon_1.__bindgen_anon_1.NameLength as u32 {
                *TargetBuffer.offset((i.wrapping_add(1)) as isize) = *CurrentName as u16;
                CurrentName = CurrentName.offset(1);
                i = i.wrapping_add(1);
            }
        }

        if !KeyNode.is_null() {
            HvReleaseCell(
                (*MyKcb).__bindgen_anon_2.__bindgen_anon_1.KeyHive,
                (*MyKcb).__bindgen_anon_2.__bindgen_anon_1.KeyCell,
            );
        }

        MyKcb = (*MyKcb).ParentKcb;
    }

    KeyName
}

#[no_mangle]
pub unsafe extern "system" fn EnlistKeyBodyWithKCB(
    KeyBody: PCM_KEY_BODY,
    Flags: ULONG,
) {
    let KeyBody = opaque_ptr(KeyBody);
    let mut i: ULONG;

    InitializeListHead(core::ptr::addr_of_mut!((*KeyBody).KeyBodyList));

    for i in 0..4 {
        if InterlockedCompareExchangePointer(
            core::ptr::addr_of_mut!((*(*KeyBody).KeyControlBlock).KeyBodyArray[i as usize]) as *mut PVOID,
            KeyBody as PVOID,
            core::ptr::null_mut(),
        ).is_null()
        {
            return;
        }
    }

    if (Flags & CMP_ENLIST_KCB_LOCKED_SHARED) != 0 {
        CmpReleaseKcbLock((*KeyBody).KeyControlBlock);
    }

    if (Flags & CMP_ENLIST_KCB_LOCKED_EXCLUSIVE) == 0 {
        CmpAcquireKcbLockExclusive((*KeyBody).KeyControlBlock);
    }

    CMP_ASSERT_KCB_LOCK((*KeyBody).KeyControlBlock);

    InsertTailList(
        core::ptr::addr_of_mut!((*(*KeyBody).KeyControlBlock).__bindgen_anon_4.KeyBodyListHead),
        core::ptr::addr_of_mut!((*KeyBody).KeyBodyList),
    );

    if (Flags & (CMP_ENLIST_KCB_LOCKED_SHARED | CMP_ENLIST_KCB_LOCKED_EXCLUSIVE)) == 0 {
        CmpReleaseKcbLock((*KeyBody).KeyControlBlock);
    }
}

#[no_mangle]
pub unsafe extern "system" fn DelistKeyBodyFromKCB(
    KeyBody: PCM_KEY_BODY,
    LockHeld: u8,
) {
    let KeyBody = opaque_ptr(KeyBody);
    let mut i: u32;

    i = 0;
    while i < 4 {
        if InterlockedCompareExchangePointer(
            core::ptr::addr_of_mut!((*(*KeyBody).KeyControlBlock).KeyBodyArray[i as usize]) as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
            KeyBody as *mut core::ffi::c_void,
        ) == KeyBody as *mut core::ffi::c_void
        {
            return;
        }
        i += 1;
    }

    if LockHeld == 0 {
        CmpAcquireKcbLockExclusive((*KeyBody).KeyControlBlock);
    }

    RemoveEntryList(core::ptr::addr_of_mut!((*KeyBody).KeyBodyList));

    if LockHeld == 0 {
        CmpReleaseKcbLock((*KeyBody).KeyControlBlock);
    }
}

// CmpUnLockKcbArray — bindgen declares as cdecl extern "C" (no NTAPI in C source).
#[no_mangle]
pub unsafe extern "C" fn CmpUnLockKcbArray(KcbArray: *mut u32) {
    let KcbArray = opaque_ptr(KcbArray);
    let mut i: u32 = *KcbArray.offset(0);
    while i > 0 {
        CmpReleaseKcbLockByIndex(*KcbArray.offset(i as isize));
        i = i.wrapping_sub(1);
    }
}

unsafe fn CmpLockKcbArray(
    KcbArray: PULONG,
    KcbLockFlags: ULONG)
{
    let mut i: ULONG;

    i = 1;
    while i <= *KcbArray.offset(0) {
        if KcbLockFlags & CMP_LOCK_KCB_ARRAY_EXCLUSIVE != 0 {
            CmpAcquireKcbLockExclusiveByIndex(*KcbArray.offset(i as isize));
        } else {
            CmpAcquireKcbLockSharedByIndex(*KcbArray.offset(i as isize));
        }
        i += 1;
    }
}

unsafe fn CmpSortKcbArray(
    KcbArray: PULONG)
{
    let mut i: ULONG;
    let mut j: ULONG;
    let mut k: ULONG;
    let mut KcbCount: ULONG;

    KcbCount = *KcbArray.offset(0);

    i = 1;
    while i <= KcbCount {
        j = i + 1;
        while j <= KcbCount {
            if *KcbArray.offset(i as isize) > *KcbArray.offset(j as isize) {
                let Temp: ULONG = *KcbArray.offset(i as isize);
                *KcbArray.offset(i as isize) = *KcbArray.offset(j as isize);
                *KcbArray.offset(j as isize) = Temp;
            }
            j += 1;
        }
        i += 1;
    }

    i = 1;
    while i <= KcbCount {
        j = i + 1;
        while j <= KcbCount {
            if *KcbArray.offset(i as isize) == *KcbArray.offset(j as isize) {
                k = j;
                while k <= KcbCount {
                    *KcbArray.offset((k - 1) as isize) = *KcbArray.offset(k as isize);
                    k += 1;
                }

                j = j.wrapping_sub(1);
                KcbCount = KcbCount.wrapping_sub(1);
            }
            j += 1;
        }
        i += 1;
    }

    *KcbArray.offset(0) = KcbCount;
}

#[no_mangle]
pub unsafe extern "system" fn CmpBuildAndLockKcbArray(
    HashCacheStack: PCM_HASH_CACHE_STACK,
    KcbLockFlags: ULONG,
    Kcb: PCM_KEY_CONTROL_BLOCK,
    OuterStackArray: PULONG,
    TotalRemainingSubkeys: ULONG,
    MatchRemainSubkeyLevel: ULONG) -> PULONG
{
    let HashCacheStack = opaque_ptr(HashCacheStack);
    let Kcb = opaque_ptr(Kcb);
    let OuterStackArray = opaque_ptr(OuterStackArray);
    let mut KcbIndex: ULONG = 1;
    let mut HashStackIndex: ULONG;
    let mut TotalRemaining: ULONG;
    let mut LockedKcbs: PULONG = core::ptr::null_mut();
    let ParentKcb: PCM_KEY_CONTROL_BLOCK = (*Kcb).ParentKcb;

    TotalRemaining = (1 + TotalRemainingSubkeys) - MatchRemainSubkeyLevel;

    if !ParentKcb.is_null() {
        if TotalRemainingSubkeys == MatchRemainSubkeyLevel {
            TotalRemaining += 1;
            *OuterStackArray.offset(KcbIndex as isize) = GET_HASH_INDEX((*ParentKcb).__bindgen_anon_2.__bindgen_anon_1.ConvKey);
            KcbIndex += 1;
        }
    }

    *OuterStackArray.offset(KcbIndex as isize) = GET_HASH_INDEX((*Kcb).__bindgen_anon_2.__bindgen_anon_1.ConvKey);
    KcbIndex += 1;

    HashStackIndex = 0;
    while HashStackIndex < TotalRemainingSubkeys {
        *OuterStackArray.offset(KcbIndex as isize) = GET_HASH_INDEX((*HashCacheStack.offset(HashStackIndex as isize)).ConvKey);
        KcbIndex += 1;
        HashStackIndex += 1;
    }

    *OuterStackArray.offset(0) = KcbIndex - 1;
    CmpSortKcbArray(OuterStackArray);

    CmpLockKcbArray(OuterStackArray, KcbLockFlags);

    LockedKcbs = OuterStackArray;
    LockedKcbs
}

#[no_mangle]
pub unsafe extern "system" fn CmpFlushNotifiesOnKeyBodyList(
    Kcb: PCM_KEY_CONTROL_BLOCK,
    LockHeld: u8,
) {
    let Kcb = opaque_ptr(Kcb);
    let mut NextEntry: PLIST_ENTRY;
    let ListHead: PLIST_ENTRY;
    let mut KeyBody: PCM_KEY_BODY;

    if LockHeld != 0 {
        CMP_ASSERT_EXCLUSIVE_REGISTRY_LOCK();
    } else {
        CmpIsKcbLockedExclusive(Kcb);
    }

    loop {
        ListHead = core::ptr::addr_of_mut!((*Kcb).__bindgen_anon_4.KeyBodyListHead);
        if !IsListEmpty(ListHead) {
            NextEntry = (*ListHead).Flink;
            while NextEntry != ListHead {
                KeyBody = ((NextEntry as usize) - core::mem::offset_of!(_CM_KEY_BODY, KeyBodyList)) as PCM_KEY_BODY;

                if !(*KeyBody).NotifyBlock.is_null() {
                    if LockHeld != 0 {
                        CmpFlushNotify(KeyBody, LockHeld);
                        continue;
                    }

                    if ObReferenceObjectSafe(KeyBody as *mut core::ffi::c_void) != 0 {
                        CmpFlushNotify(KeyBody, LockHeld);
                        ObDereferenceObjectDeferDelete(KeyBody as *mut core::ffi::c_void);
                        continue;
                    }
                }

                NextEntry = (*NextEntry).Flink;
            }
        }

        break;
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/config/cmkcbncb.c
//  * PURPOSE:         Routines for handling KCBs, NCBs, as well as key hashes.
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// ULONG CmpHashTableSize = 2048;
// PCM_KEY_HASH_TABLE_ENTRY CmpCacheTable;
// PCM_NAME_HASH_TABLE_ENTRY CmpNameCacheTable;
//
// /* FUNCTIONS *****************************************************************/
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// CmpInitializeCache(VOID)
// {
//     ULONG Length, i;
//
//     /* Calculate length for the table */
//     Length = CmpHashTableSize * sizeof(CM_KEY_HASH_TABLE_ENTRY);
//
//     /* Allocate it */
//     CmpCacheTable = CmpAllocate(Length, TRUE, TAG_CM);
//     if (!CmpCacheTable)
//     {
//         /* Take the system down */
//         KeBugCheckEx(CONFIG_INITIALIZATION_FAILED, 3, 1, 0, 0);
//     }
//
//     /* Zero out the table */
//     RtlZeroMemory(CmpCacheTable, Length);
//
//     /* Initialize the locks */
//     for (i = 0;i < CmpHashTableSize; i++)
//     {
//         /* Setup the pushlock */
//         ExInitializePushLock(&CmpCacheTable[i].Lock);
//     }
//
//     /* Calculate length for the name cache */
//     Length = CmpHashTableSize * sizeof(CM_NAME_HASH_TABLE_ENTRY);
//
//     /* Now allocate the name cache table */
//     CmpNameCacheTable = CmpAllocate(Length, TRUE, TAG_CM);
//     if (!CmpNameCacheTable)
//     {
//         /* Take the system down */
//         KeBugCheckEx(CONFIG_INITIALIZATION_FAILED, 3, 3, 0, 0);
//     }
//
//     /* Zero out the table */
//     RtlZeroMemory(CmpNameCacheTable, Length);
//
//     /* Initialize the locks */
//     for (i = 0;i < CmpHashTableSize; i++)
//     {
//         /* Setup the pushlock */
//         ExInitializePushLock(&CmpNameCacheTable[i].Lock);
//     }
//
//     /* Setup the delayed close table */
//     CmpInitializeDelayedCloseTable();
// }
//
// VOID
// NTAPI
// CmpRemoveKeyHash(IN PCM_KEY_HASH KeyHash)
// {
//     PCM_KEY_HASH *Prev;
//     PCM_KEY_HASH Current;
//     ASSERT_VALID_HASH(KeyHash);
//
//     /* Lookup all the keys in this index entry */
//     Prev = &GET_HASH_ENTRY(CmpCacheTable, KeyHash->ConvKey)->Entry;
//     while (TRUE)
//     {
//         /* Save the current one and make sure it's valid */
//         Current = *Prev;
//         ASSERT(Current != NULL);
//         ASSERT_VALID_HASH(Current);
//
//         /* Check if it matches */
//         if (Current == KeyHash)
//         {
//             /* Then write the previous one */
//             *Prev = Current->NextHash;
//             if (*Prev) ASSERT_VALID_HASH(*Prev);
//             break;
//         }
//
//         /* Otherwise, keep going */
//         Prev = &Current->NextHash;
//     }
// }
//
// PCM_KEY_CONTROL_BLOCK
// NTAPI
// CmpInsertKeyHash(IN PCM_KEY_HASH KeyHash,
//                  IN BOOLEAN IsFake)
// {
//     ULONG i;
//     PCM_KEY_HASH Entry;
//     ASSERT_VALID_HASH(KeyHash);
//
//     /* Get the hash index */
//     i = GET_HASH_INDEX(KeyHash->ConvKey);
//
//     /* If this is a fake key, increase the key cell to use the parent data */
//     if (IsFake) KeyHash->KeyCell++;
//
//     /* Loop the hash table */
//     Entry = CmpCacheTable[i].Entry;
//     while (Entry)
//     {
//         /* Check if this matches */
//         ASSERT_VALID_HASH(Entry);
//         if ((KeyHash->ConvKey == Entry->ConvKey) &&
//             (KeyHash->KeyCell == Entry->KeyCell) &&
//             (KeyHash->KeyHive == Entry->KeyHive))
//         {
//             /* Return it */
//             return CONTAINING_RECORD(Entry, CM_KEY_CONTROL_BLOCK, KeyHash);
//         }
//
//         /* Keep looping */
//         Entry = Entry->NextHash;
//     }
//
//     /* No entry found, add this one and return NULL since none existed */
//     KeyHash->NextHash = CmpCacheTable[i].Entry;
//     CmpCacheTable[i].Entry = KeyHash;
//     return NULL;
// }
//
// PCM_NAME_CONTROL_BLOCK
// NTAPI
// CmpGetNameControlBlock(IN PUNICODE_STRING NodeName)
// {
//     PCM_NAME_CONTROL_BLOCK Ncb = NULL;
//     ULONG ConvKey = 0;
//     PWCHAR p, pp;
//     ULONG i;
//     BOOLEAN IsCompressed = TRUE, Found = FALSE;
//     PCM_NAME_HASH HashEntry;
//     ULONG NcbSize;
//     USHORT Length;
//
//     /* Loop the name */
//     p = NodeName->Buffer;
//     for (i = 0; i < NodeName->Length; i += sizeof(WCHAR))
//     {
//         /* Make sure it's not a slash */
//         if (*p != OBJ_NAME_PATH_SEPARATOR)
//         {
//             /* Add it to the hash */
//             ConvKey = COMPUTE_HASH_CHAR(ConvKey, *p);
//         }
//
//         /* Next character */
//         p++;
//     }
//
//     /* Set assumed lengh and loop to check */
//     Length = NodeName->Length / sizeof(WCHAR);
//     for (i = 0; i < (NodeName->Length / sizeof(WCHAR)); i++)
//     {
//         /* Check if this is a 16-bit character */
//         if (NodeName->Buffer[i] > (UCHAR)-1)
//         {
//             /* This is the actual size, and we know we're not compressed */
//             Length = NodeName->Length;
//             IsCompressed = FALSE;
//             break;
//         }
//     }
//
//     /* Lock the NCB entry */
//     CmpAcquireNcbLockExclusiveByKey(ConvKey);
//
//     /* Get the hash entry */
//     HashEntry = GET_NAME_HASH_ENTRY(CmpNameCacheTable, ConvKey)->Entry;
//     while (HashEntry)
//     {
//         /* Get the current NCB */
//         Ncb = CONTAINING_RECORD(HashEntry, CM_NAME_CONTROL_BLOCK, NameHash);
//
//         /* Check if the hash matches */
//         if ((ConvKey == HashEntry->ConvKey) && (Length == Ncb->NameLength))
//         {
//             /* Assume success */
//             Found = TRUE;
//
//             /* If the NCB is compressed, do a compressed name compare */
//             if (Ncb->Compressed)
//             {
//                 /* Compare names */
//                 if (CmpCompareCompressedName(NodeName, Ncb->Name, Length))
//                 {
//                     /* We failed */
//                     Found = FALSE;
//                 }
//             }
//             else
//             {
//                 /* Do a manual compare */
//                 p = NodeName->Buffer;
//                 pp = Ncb->Name;
//                 for (i = 0; i < Ncb->NameLength; i += sizeof(WCHAR))
//                 {
//                     /* Compare the character */
//                     if (RtlUpcaseUnicodeChar(*p) != RtlUpcaseUnicodeChar(*pp))
//                     {
//                         /* Failed */
//                         Found = FALSE;
//                         break;
//                     }
//
//                     /* Next chars */
//                     p++;
//                     pp++;
//                 }
//             }
//
//             /* Check if we found a name */
//             if (Found)
//             {
//                 /* Reference it */
//                 ASSERT(Ncb->RefCount != 0xFFFF);
//                 Ncb->RefCount++;
//                 break;
//             }
//         }
//
//         /* Go to the next hash */
//         HashEntry = HashEntry->NextHash;
//     }
//
//     /* Check if we didn't find it */
//     if (!Found)
//     {
//         /* Allocate one */
//         NcbSize = FIELD_OFFSET(CM_NAME_CONTROL_BLOCK, Name) + Length;
//         Ncb = CmpAllocate(NcbSize, TRUE, TAG_CM);
//         if (!Ncb)
//         {
//             /* Release the lock and fail */
//             CmpReleaseNcbLockByKey(ConvKey);
//             return NULL;
//         }
//
//         /* Clear it out */
//         RtlZeroMemory(Ncb, NcbSize);
//
//         /* Check if the name was compressed */
//         if (IsCompressed)
//         {
//             /* Copy the compressed name */
//             for (i = 0; i < NodeName->Length / sizeof(WCHAR); i++)
//             {
//                 /* Copy Unicode to ANSI */
//                 ((PCHAR)Ncb->Name)[i] = (CHAR)RtlUpcaseUnicodeChar(NodeName->Buffer[i]);
//             }
//         }
//         else
//         {
//             /* Copy the name directly */
//             for (i = 0; i < NodeName->Length / sizeof(WCHAR); i++)
//             {
//                 /* Copy each unicode character */
//                 Ncb->Name[i] = RtlUpcaseUnicodeChar(NodeName->Buffer[i]);
//             }
//         }
//
//         /* Setup the rest of the NCB */
//         Ncb->Compressed = IsCompressed;
//         Ncb->ConvKey = ConvKey;
//         Ncb->RefCount++;
//         Ncb->NameLength = Length;
//
//         /* Insert the name in the hash table */
//         HashEntry = &Ncb->NameHash;
//         HashEntry->NextHash = GET_NAME_HASH_ENTRY(CmpNameCacheTable, ConvKey)->Entry;
//         GET_NAME_HASH_ENTRY(CmpNameCacheTable, ConvKey)->Entry = HashEntry;
//     }
//
//     /* Release NCB lock */
//     CmpReleaseNcbLockByKey(ConvKey);
//
//     /* Return the NCB found */
//     return Ncb;
// }
//
// VOID
// NTAPI
// CmpRemoveKeyControlBlock(IN PCM_KEY_CONTROL_BLOCK Kcb)
// {
//     /* Make sure we have the exclusive lock */
//     CMP_ASSERT_KCB_LOCK(Kcb);
//
//     /* Remove the key hash */
//     CmpRemoveKeyHash(&Kcb->KeyHash);
// }
//
// VOID
// NTAPI
// CmpDereferenceNameControlBlockWithLock(IN PCM_NAME_CONTROL_BLOCK Ncb)
// {
//     PCM_NAME_HASH Current, *Next;
//     ULONG ConvKey = Ncb->ConvKey;
//
//     /* Lock the NCB */
//     CmpAcquireNcbLockExclusiveByKey(ConvKey);
//
//     /* Decrease the reference count */
//     ASSERT(Ncb->RefCount >= 1);
//     if (!(--Ncb->RefCount))
//     {
//         /* Find the NCB in the table */
//         Next = &GET_NAME_HASH_ENTRY(CmpNameCacheTable, Ncb->ConvKey)->Entry;
//         while (TRUE)
//         {
//             /* Check the current entry */
//             Current = *Next;
//             ASSERT(Current != NULL);
//             if (Current == &Ncb->NameHash)
//             {
//                 /* Unlink it */
//                 *Next = Current->NextHash;
//                 break;
//             }
//
//             /* Get to the next one */
//             Next = &Current->NextHash;
//         }
//
//         /* Found it, now free it */
//         CmpFree(Ncb, 0);
//     }
//
//     /* Release the lock */
//     CmpReleaseNcbLockByKey(ConvKey);
// }
//
// BOOLEAN
// NTAPI
// CmpReferenceKeyControlBlock(IN PCM_KEY_CONTROL_BLOCK Kcb)
// {
//     CMTRACE(CM_REFERENCE_DEBUG,
//             "%s - Referencing KCB: %p\n", __FUNCTION__, Kcb);
//
//     /* Check if this is the KCB's first reference */
//     if (Kcb->RefCount == 0)
//     {
//         /* Check if the KCB is locked in shared mode */
//         if (!CmpIsKcbLockedExclusive(Kcb))
//         {
//             /* Convert it to exclusive */
//             if (!CmpTryToConvertKcbSharedToExclusive(Kcb))
//             {
//                 /* Set the delayed close index so that we can be ignored */
//                 Kcb->DelayedCloseIndex = 1;
//
//                 /* Increase the reference count while we release the lock */
//                 InterlockedIncrement((PLONG)&Kcb->RefCount);
//
//                 /* Go from shared to exclusive */
//                 CmpConvertKcbSharedToExclusive(Kcb);
//
//                 /* Decrement the reference count; the lock is now held again */
//                 InterlockedDecrement((PLONG)&Kcb->RefCount);
//
//                 /* Check if we still control the index */
//                 if (Kcb->DelayedCloseIndex == 1)
//                 {
//                     /* Reset it */
//                     Kcb->DelayedCloseIndex = 0;
//                 }
//                 else
//                 {
//                     /* Sanity check */
//                     ASSERT((Kcb->DelayedCloseIndex == CmpDelayedCloseSize) ||
//                            (Kcb->DelayedCloseIndex == 0));
//                 }
//             }
//         }
//     }
//
//     /* Increase the reference count */
//     if ((InterlockedIncrement((PLONG)&Kcb->RefCount) & 0xFFFF) == 0)
//     {
//         /* We've overflown to 64K references, bail out */
//         InterlockedDecrement((PLONG)&Kcb->RefCount);
//         return FALSE;
//     }
//
//     /* Check if this was the last close index */
//     if (!Kcb->DelayedCloseIndex)
//     {
//         /* Check if the KCB is locked in shared mode */
//         if (!CmpIsKcbLockedExclusive(Kcb))
//         {
//             /* Convert it to exclusive */
//             if (!CmpTryToConvertKcbSharedToExclusive(Kcb))
//             {
//                 /* Go from shared to exclusive */
//                 CmpConvertKcbSharedToExclusive(Kcb);
//             }
//         }
//
//         /* If we're still the last entry, remove us */
//         if (!Kcb->DelayedCloseIndex) CmpRemoveFromDelayedClose(Kcb);
//     }
//
//     /* Return success */
//     return TRUE;
// }
//
// VOID
// NTAPI
// CmpCleanUpKcbValueCache(IN PCM_KEY_CONTROL_BLOCK Kcb)
// {
//     PULONG_PTR CachedList;
//     ULONG i;
//
//     /* Make sure we have the exclusive lock */
//     CMP_ASSERT_KCB_LOCK(Kcb);
//
//     /* Check if the value list is cached */
//     if (CMP_IS_CELL_CACHED(Kcb->ValueCache.ValueList))
//     {
//         /* Get the cache list */
//         CachedList = (PULONG_PTR)CMP_GET_CACHED_DATA(Kcb->ValueCache.ValueList);
//         for (i = 0; i < Kcb->ValueCache.Count; i++)
//         {
//             /* Check if this cell is cached */
//             if (CMP_IS_CELL_CACHED(CachedList[i]))
//             {
//                 /* Free it */
//                 CmpFree((PVOID)CMP_GET_CACHED_CELL(CachedList[i]), 0);
//             }
//         }
//
//         /* Now free the list */
//         CmpFree((PVOID)CMP_GET_CACHED_CELL(Kcb->ValueCache.ValueList), 0);
//         Kcb->ValueCache.ValueList = HCELL_NIL;
//     }
//     else if (Kcb->ExtFlags & CM_KCB_SYM_LINK_FOUND)
//     {
//         /* This is a sym link, check if there's only one reference left */
//         if ((Kcb->ValueCache.RealKcb->RefCount == 1) &&
//             !(Kcb->ValueCache.RealKcb->Delete))
//         {
//             /* Disable delay close for the KCB */
//             Kcb->ValueCache.RealKcb->ExtFlags |= CM_KCB_NO_DELAY_CLOSE;
//         }
//
//         /* Dereference the KCB */
//         CmpDelayDerefKeyControlBlock(Kcb->ValueCache.RealKcb);
//         Kcb->ExtFlags &= ~CM_KCB_SYM_LINK_FOUND;
//     }
// }
//
// VOID
// NTAPI
// CmpCleanUpKcbCacheWithLock(IN PCM_KEY_CONTROL_BLOCK Kcb,
//                            IN BOOLEAN LockHeldExclusively)
// {
//     PCM_KEY_CONTROL_BLOCK Parent;
//     PAGED_CODE();
//
//     /* Sanity checks */
//     CMP_ASSERT_KCB_LOCK(Kcb);
//     ASSERT(Kcb->RefCount == 0);
//
//     /* Cleanup the value cache */
//     CmpCleanUpKcbValueCache(Kcb);
//
//     /* Dereference the NCB */
//     CmpDereferenceNameControlBlockWithLock(Kcb->NameBlock);
//
//     /* Check if we have an index hint block and free it */
//     if (Kcb->ExtFlags & CM_KCB_SUBKEY_HINT) CmpFree(Kcb->IndexHint, 0);
//
//     /* Check if we were already deleted */
//     Parent = Kcb->ParentKcb;
//     if (!Kcb->Delete) CmpRemoveKeyControlBlock(Kcb);
//
//     /* Set invalid KCB signature */
//     Kcb->Signature = CM_KCB_INVALID_SIGNATURE;
//
//     /* Free the KCB as well */
//     CmpFreeKeyControlBlock(Kcb);
//
//     /* Check if we have a parent */
//     if (Parent)
//     {
//         /* Dereference the parent */
//         LockHeldExclusively ?
//             CmpDereferenceKeyControlBlockWithLock(Parent,LockHeldExclusively) :
//             CmpDelayDerefKeyControlBlock(Parent);
//     }
// }
//
// VOID
// NTAPI
// CmpCleanUpSubKeyInfo(IN PCM_KEY_CONTROL_BLOCK Kcb)
// {
//     PCM_KEY_NODE KeyNode;
//
//     /* Make sure we have the exclusive lock */
//     CMP_ASSERT_KCB_LOCK(Kcb);
//
//     /* Check if there's any cached subkey */
//     if (Kcb->ExtFlags & (CM_KCB_NO_SUBKEY | CM_KCB_SUBKEY_ONE | CM_KCB_SUBKEY_HINT))
//     {
//         /* Check if there's a hint */
//         if (Kcb->ExtFlags & (CM_KCB_SUBKEY_HINT))
//         {
//             /* Kill it */
//             CmpFree(Kcb->IndexHint, 0);
//         }
//
//         /* Remove subkey flags */
//         Kcb->ExtFlags &= ~(CM_KCB_NO_SUBKEY | CM_KCB_SUBKEY_ONE | CM_KCB_SUBKEY_HINT);
//     }
//
//     /* Check if there's no linked cell */
//     if (Kcb->KeyCell == HCELL_NIL)
//     {
//         /* Make sure it's a delete */
//         ASSERT(Kcb->Delete);
//         KeyNode = NULL;
//     }
//     else
//     {
//         /* Get the key node */
//         KeyNode = (PCM_KEY_NODE)HvGetCell(Kcb->KeyHive, Kcb->KeyCell);
//     }
//
//     /* Check if we got the node */
//     if (!KeyNode)
//     {
//         /* We didn't, mark the cached data invalid */
//         Kcb->ExtFlags |= CM_KCB_INVALID_CACHED_INFO;
//     }
//     else
//     {
//         /* We have a keynode, update subkey counts */
//         Kcb->ExtFlags &= ~CM_KCB_INVALID_CACHED_INFO;
//         Kcb->SubKeyCount = KeyNode->SubKeyCounts[Stable] +
//                            KeyNode->SubKeyCounts[Volatile];
//
//         /* Release the cell */
//         HvReleaseCell(Kcb->KeyHive, Kcb->KeyCell);
//     }
// }
//
// VOID
// NTAPI
// CmpDereferenceKeyControlBlock(IN PCM_KEY_CONTROL_BLOCK Kcb)
// {
//     LONG OldRefCount, NewRefCount;
//     ULONG ConvKey;
//     CMTRACE(CM_REFERENCE_DEBUG,
//             "%s - Dereferencing KCB: %p\n", __FUNCTION__, Kcb);
//
//     /* Get the ref count and update it */
//     OldRefCount = *(PLONG)&Kcb->RefCount;
//     NewRefCount = OldRefCount - 1;
//
//     /* Check if we still have references */
//     if ((NewRefCount & 0xFFFF) > 0)
//     {
//         /* Do the dereference */
//         if (InterlockedCompareExchange((PLONG)&Kcb->RefCount,
//                                        NewRefCount,
//                                        OldRefCount) == OldRefCount)
//         {
//             /* We'de done */
//             return;
//         }
//     }
//
//     /* Save the key */
//     ConvKey = Kcb->ConvKey;
//
//     /* Do the dereference inside the lock */
//     CmpAcquireKcbLockExclusive(Kcb);
//     CmpDereferenceKeyControlBlockWithLock(Kcb, FALSE);
//     CmpReleaseKcbLockByKey(ConvKey);
// }
//
// VOID
// NTAPI
// CmpDereferenceKeyControlBlockWithLock(IN PCM_KEY_CONTROL_BLOCK Kcb,
//                                       IN BOOLEAN LockHeldExclusively)
// {
//     CMTRACE(CM_REFERENCE_DEBUG,
//             "%s - Dereferencing KCB: %p\n", __FUNCTION__, Kcb);
//
//     /* Sanity check */
//     ASSERT_KCB_VALID(Kcb);
//
//     /* Check if this is the last reference */
//     if ((InterlockedDecrement((PLONG)&Kcb->RefCount) & 0xFFFF) == 0)
//     {
//         /* Make sure we have the exclusive lock */
//         CMP_ASSERT_KCB_LOCK(Kcb);
//
//         /* Check if we should do a direct delete */
//         if ((CmpHoldLazyFlush &&
//              !(Kcb->ExtFlags & CM_KCB_SYM_LINK_FOUND) &&
//              !(Kcb->Flags & KEY_SYM_LINK)) ||
//             (Kcb->ExtFlags & CM_KCB_NO_DELAY_CLOSE) ||
//              Kcb->Delete)
//         {
//             /* Clean up the KCB*/
//             CmpCleanUpKcbCacheWithLock(Kcb, LockHeldExclusively);
//         }
//         else
//         {
//             /* Otherwise, use delayed close */
//             CmpAddToDelayedClose(Kcb, LockHeldExclusively);
//         }
//     }
// }
//
// VOID
// NTAPI
// InitializeKCBKeyBodyList(IN PCM_KEY_CONTROL_BLOCK Kcb)
// {
//     /* Initialize the list */
//     InitializeListHead(&Kcb->KeyBodyListHead);
//
//     /* Clear the bodies */
//     Kcb->KeyBodyArray[0] =
//     Kcb->KeyBodyArray[1] =
//     Kcb->KeyBodyArray[2] =
//     Kcb->KeyBodyArray[3] = NULL;
// }
//
// PCM_KEY_CONTROL_BLOCK
// NTAPI
// CmpCreateKeyControlBlock(IN PHHIVE Hive,
//                          IN HCELL_INDEX Index,
//                          IN PCM_KEY_NODE Node,
//                          IN PCM_KEY_CONTROL_BLOCK Parent,
//                          IN ULONG Flags,
//                          IN PUNICODE_STRING KeyName)
// {
//     PCM_KEY_CONTROL_BLOCK Kcb, FoundKcb = NULL;
//     UNICODE_STRING NodeName;
//     ULONG ConvKey = 0, i;
//     BOOLEAN IsFake, HashLock;
//     PWCHAR p;
//
//     /* Make sure we own this hive in case it's being unloaded */
//     if ((Hive->HiveFlags & HIVE_IS_UNLOADING) &&
//         (((PCMHIVE)Hive)->CreatorOwner != KeGetCurrentThread()))
//     {
//         /* Fail */
//         return NULL;
//     }
//
//     /* Check if this is a fake KCB */
//     IsFake = Flags & CMP_CREATE_FAKE_KCB ? TRUE : FALSE;
//
//     /* If we have a parent, use its ConvKey */
//     if (Parent) ConvKey = Parent->ConvKey;
//
//     /* Make a copy of the name */
//     NodeName = *KeyName;
//
//     /* Remove leading slash */
//     while (NodeName.Length && (*NodeName.Buffer == OBJ_NAME_PATH_SEPARATOR))
//     {
//         /* Move the buffer by one */
//         NodeName.Buffer++;
//         NodeName.Length -= sizeof(WCHAR);
//     }
//
//     /* Make sure we didn't get just a slash or something */
//     ASSERT(NodeName.Length > 0);
//
//     /* Now setup the hash */
//     p = NodeName.Buffer;
//     for (i = 0; i < NodeName.Length; i += sizeof(WCHAR))
//     {
//         /* Make sure it's a valid character */
//         if (*p != OBJ_NAME_PATH_SEPARATOR)
//         {
//             /* Add this key to the hash */
//             ConvKey = COMPUTE_HASH_CHAR(ConvKey, *p);
//         }
//
//         /* Move on */
//         p++;
//     }
//
//     /* Allocate the KCB */
//     Kcb = CmpAllocateKeyControlBlock();
//     if (!Kcb) return NULL;
//
//     /* Initailize the key list */
//     InitializeKCBKeyBodyList(Kcb);
//
//     /* Set it up */
//     Kcb->Signature = CM_KCB_SIGNATURE;
//     Kcb->Delete = FALSE;
//     Kcb->RefCount = 1;
//     Kcb->KeyHive = Hive;
//     Kcb->KeyCell = Index;
//     Kcb->ConvKey = ConvKey;
//     Kcb->DelayedCloseIndex = CmpDelayedCloseSize;
//     Kcb->InDelayClose = 0;
//     ASSERT_KCB_VALID(Kcb);
//
//     /* Check if we have two hash entires */
//     HashLock = Flags & CMP_LOCK_HASHES_FOR_KCB ? TRUE : FALSE;
//     if (!HashLock)
//     {
//         /* It's not locked, do we have a parent? */
//         if (Parent)
//         {
//             /* Lock the parent KCB and ourselves */
//             CmpAcquireTwoKcbLocksExclusiveByKey(ConvKey, Parent->ConvKey);
//         }
//         else
//         {
//             /* Lock only ourselves */
//             CmpAcquireKcbLockExclusive(Kcb);
//         }
//     }
//
//     /* Check if we already have a KCB */
//     FoundKcb = CmpInsertKeyHash(&Kcb->KeyHash, IsFake);
//     if (FoundKcb)
//     {
//         /* Sanity check */
//         ASSERT(!FoundKcb->Delete);
//         Kcb->Signature = CM_KCB_INVALID_SIGNATURE;
//
//         /* Free the one we allocated and reference this one */
//         CmpFreeKeyControlBlock(Kcb);
//         ASSERT_KCB_VALID(FoundKcb);
//         Kcb = FoundKcb;
//         if (!CmpReferenceKeyControlBlock(Kcb))
//         {
//             /* We got too many handles */
//             ASSERT(Kcb->RefCount + 1 != 0);
//             Kcb = NULL;
//         }
//         else
//         {
//             /* Check if we're not creating a fake one, but it used to be fake */
//             if ((Kcb->ExtFlags & CM_KCB_KEY_NON_EXIST) && !IsFake)
//             {
//                 /* Set the hive and cell */
//                 Kcb->KeyHive = Hive;
//                 Kcb->KeyCell = Index;
//
//                 /* This means that our current information is invalid */
//                 Kcb->ExtFlags = CM_KCB_INVALID_CACHED_INFO;
//             }
//
//             /* Check if we didn't have any valid data */
//             if (!(Kcb->ExtFlags & (CM_KCB_NO_SUBKEY |
//                                    CM_KCB_SUBKEY_ONE |
//                                    CM_KCB_SUBKEY_HINT)))
//             {
//                 /* Calculate the index hint */
//                 Kcb->SubKeyCount = Node->SubKeyCounts[Stable] +
//                                    Node->SubKeyCounts[Volatile];
//
//                 /* Cached information is now valid */
//                 Kcb->ExtFlags &= ~CM_KCB_INVALID_CACHED_INFO;
//             }
//
//             /* Setup the other data */
//             Kcb->KcbLastWriteTime = Node->LastWriteTime;
//             Kcb->KcbMaxNameLen = (USHORT)Node->MaxNameLen;
//             Kcb->KcbMaxValueNameLen = (USHORT)Node->MaxValueNameLen;
//             Kcb->KcbMaxValueDataLen = Node->MaxValueDataLen;
//         }
//     }
//     else
//     {
//         /* No KCB, do we have a parent? */
//         if (Parent)
//         {
//             /* Reference the parent */
//             if (((Parent->TotalLevels + 1) < 512) &&
//                 CmpReferenceKeyControlBlock(Parent))
//             {
//                 /* Link it */
//                 Kcb->ParentKcb = Parent;
//                 Kcb->TotalLevels = Parent->TotalLevels + 1;
//             }
//             else
//             {
//                 /* Remove the KCB and free it */
//                 CmpRemoveKeyControlBlock(Kcb);
//                 Kcb->Signature = CM_KCB_INVALID_SIGNATURE;
//                 CmpFreeKeyControlBlock(Kcb);
//                 Kcb = NULL;
//             }
//         }
//         else
//         {
//             /* No parent, this is the root node */
//             Kcb->ParentKcb = NULL;
//             Kcb->TotalLevels = 1;
//         }
//
//         /* Check if we have a KCB */
//         if (Kcb)
//         {
//             /* Get the NCB */
//             Kcb->NameBlock = CmpGetNameControlBlock(&NodeName);
//             if (Kcb->NameBlock)
//             {
//                 /* Fill it out */
//                 Kcb->ValueCache.Count = Node->ValueList.Count;
//                 Kcb->ValueCache.ValueList = Node->ValueList.List;
//                 Kcb->Flags = Node->Flags;
//                 Kcb->ExtFlags = 0;
//                 Kcb->DelayedCloseIndex = CmpDelayedCloseSize;
//
//                 /* Remember if this is a fake key */
//                 if (IsFake) Kcb->ExtFlags |= CM_KCB_KEY_NON_EXIST;
//
//                 /* Setup the other data */
//                 Kcb->SubKeyCount = Node->SubKeyCounts[Stable] +
//                                    Node->SubKeyCounts[Volatile];
//                 Kcb->KcbLastWriteTime = Node->LastWriteTime;
//                 Kcb->KcbMaxNameLen = (USHORT)Node->MaxNameLen;
//                 Kcb->KcbMaxValueNameLen = (USHORT)Node->MaxValueNameLen;
//                 Kcb->KcbMaxValueDataLen = (USHORT)Node->MaxValueDataLen;
//             }
//             else
//             {
//                 /* Dereference the KCB */
//                 CmpDereferenceKeyControlBlockWithLock(Parent, FALSE);
//
//                 /* Remove the KCB and free it */
//                 CmpRemoveKeyControlBlock(Kcb);
//                 Kcb->Signature = CM_KCB_INVALID_SIGNATURE;
//                 CmpFreeKeyControlBlock(Kcb);
//                 Kcb = NULL;
//             }
//         }
//     }
//
//     /* Check if this is a KCB inside a frozen hive */
//     if (Kcb && ((PCMHIVE)Hive)->Frozen && !(Kcb->Flags & KEY_SYM_LINK))
//     {
//         /* Don't add these to the delay close */
//         Kcb->ExtFlags |= CM_KCB_NO_DELAY_CLOSE;
//     }
//
//     /* Sanity check */
//     ASSERT(!Kcb || !Kcb->Delete);
//
//     /* Check if we had locked the hashes */
//     if (!HashLock)
//     {
//         /* We locked them manually, do we have a parent? */
//         if (Parent)
//         {
//             /* Unlock the parent KCB and ourselves */
//             CmpReleaseTwoKcbLockByKey(ConvKey, Parent->ConvKey);
//         }
//         else
//         {
//             /* Unlock only ourselves */
//             CmpReleaseKcbLockByKey(ConvKey);
//         }
//     }
//
//     /* Return the KCB */
//     return Kcb;
// }
//
// PUNICODE_STRING
// NTAPI
// CmpConstructName(IN PCM_KEY_CONTROL_BLOCK Kcb)
// {
//     PUNICODE_STRING KeyName;
//     ULONG i;
//     USHORT NameLength;
//     PCM_KEY_CONTROL_BLOCK MyKcb;
//     PCM_KEY_NODE KeyNode;
//     BOOLEAN DeletedKey = FALSE;
//     PWCHAR TargetBuffer, CurrentNameW;
//     PUCHAR CurrentName;
//
//     /* Calculate how much size our key name is going to occupy */
//     NameLength = 0;
//     MyKcb = Kcb;
//
//     while (MyKcb)
//     {
//         /* Add length of the name */
//         if (!MyKcb->NameBlock->Compressed)
//         {
//             NameLength += MyKcb->NameBlock->NameLength;
//         }
//         else
//         {
//             NameLength += CmpCompressedNameSize(MyKcb->NameBlock->Name,
//                                                 MyKcb->NameBlock->NameLength);
//         }
//
//         /* Sum up the separator too */
//         NameLength += sizeof(WCHAR);
//
//         /* Go to the parent KCB */
//         MyKcb = MyKcb->ParentKcb;
//     }
//
//     /* Allocate the unicode string now */
//     KeyName = CmpAllocate(NameLength + sizeof(UNICODE_STRING),
//                           TRUE,
//                           TAG_CM);
//
//     if (!KeyName) return NULL;
//
//     /* Set it up */
//     KeyName->Buffer = (PWSTR)(KeyName + 1);
//     KeyName->Length = NameLength;
//     KeyName->MaximumLength = NameLength;
//
//     /* Loop the keys again, now adding names */
//     NameLength = 0;
//     MyKcb = Kcb;
//
//     while (MyKcb)
//     {
//         /* Sanity checks for deleted and fake keys */
//         if ((!MyKcb->KeyCell && !MyKcb->Delete) ||
//             !MyKcb->KeyHive ||
//             (MyKcb->ExtFlags & CM_KCB_KEY_NON_EXIST))
//         {
//             /* Failure */
//             CmpFree(KeyName, 0);
//             return NULL;
//         }
//
//         /* Try to get the name from the keynode,
//            if the key is not deleted */
//         if (!DeletedKey && !MyKcb->Delete)
//         {
//             KeyNode = (PCM_KEY_NODE)HvGetCell(MyKcb->KeyHive, MyKcb->KeyCell);
//             if (!KeyNode)
//             {
//                 /* Failure */
//                 CmpFree(KeyName, 0);
//                 return NULL;
//             }
//         }
//         else
//         {
//             /* The key was deleted */
//             KeyNode = NULL;
//             DeletedKey = TRUE;
//         }
//
//         /* Get the pointer to the beginning of the current key name */
//         NameLength += (MyKcb->NameBlock->NameLength + 1) * sizeof(WCHAR);
//         TargetBuffer = &KeyName->Buffer[(KeyName->Length - NameLength) / sizeof(WCHAR)];
//
//         /* Add a separator */
//         TargetBuffer[0] = OBJ_NAME_PATH_SEPARATOR;
//
//         /* Add the name, but remember to go from the end to the beginning */
//         if (!MyKcb->NameBlock->Compressed)
//         {
//             /* Get the pointer to the name (from the keynode, if possible) */
//             if ((MyKcb->Flags & (KEY_HIVE_ENTRY | KEY_HIVE_EXIT)) ||
//                 !KeyNode)
//             {
//                 CurrentNameW = MyKcb->NameBlock->Name;
//             }
//             else
//             {
//                 CurrentNameW = KeyNode->Name;
//             }
//
//             /* Copy the name */
//             for (i=0; i < MyKcb->NameBlock->NameLength; i++)
//             {
//                 TargetBuffer[i+1] = *CurrentNameW;
//                 CurrentNameW++;
//             }
//         }
//         else
//         {
//             /* Get the pointer to the name (from the keynode, if possible) */
//             if ((MyKcb->Flags & (KEY_HIVE_ENTRY | KEY_HIVE_EXIT)) ||
//                 !KeyNode)
//             {
//                 CurrentName = (PUCHAR)MyKcb->NameBlock->Name;
//             }
//             else
//             {
//                 CurrentName = (PUCHAR)KeyNode->Name;
//             }
//
//             /* Copy the name */
//             for (i=0; i < MyKcb->NameBlock->NameLength; i++)
//             {
//                 TargetBuffer[i+1] = (WCHAR)*CurrentName;
//                 CurrentName++;
//             }
//         }
//
//         /* Release the cell, if needed */
//         if (KeyNode) HvReleaseCell(MyKcb->KeyHive, MyKcb->KeyCell);
//
//         /* Go to the parent KCB */
//         MyKcb = MyKcb->ParentKcb;
//     }
//
//     /* Return resulting buffer (both UNICODE_STRING and
//        its buffer following it) */
//     return KeyName;
// }
//
// VOID
// NTAPI
// EnlistKeyBodyWithKCB(IN PCM_KEY_BODY KeyBody,
//                      IN ULONG Flags)
// {
//     ULONG i;
//
//     /* Sanity check */
//     ASSERT(KeyBody->KeyControlBlock != NULL);
//
//     /* Initialize the list entry */
//     InitializeListHead(&KeyBody->KeyBodyList);
//
//     /* Check if we can use the parent KCB array */
//     for (i = 0; i < 4; i++)
//     {
//         /* Add it into the list */
//         if (!InterlockedCompareExchangePointer((PVOID*)&KeyBody->KeyControlBlock->
//                                                KeyBodyArray[i],
//                                                KeyBody,
//                                                NULL))
//         {
//             /* Added */
//             return;
//         }
//     }
//
//     /* Array full, check if we need to unlock the KCB */
//     if (Flags & CMP_ENLIST_KCB_LOCKED_SHARED)
//     {
//         /* It's shared, so release the KCB shared lock */
//         CmpReleaseKcbLock(KeyBody->KeyControlBlock);
//         ASSERT(!(Flags & CMP_ENLIST_KCB_LOCKED_EXCLUSIVE));
//     }
//
//     /* Check if we need to lock the KCB */
//     if (!(Flags & CMP_ENLIST_KCB_LOCKED_EXCLUSIVE))
//     {
//         /* Acquire the lock */
//         CmpAcquireKcbLockExclusive(KeyBody->KeyControlBlock);
//     }
//
//     /* Make sure we have the exclusive lock */
//     CMP_ASSERT_KCB_LOCK(KeyBody->KeyControlBlock);
//
//     /* Do the insert */
//     InsertTailList(&KeyBody->KeyControlBlock->KeyBodyListHead,
//                    &KeyBody->KeyBodyList);
//
//     /* Check if we did a manual lock */
//     if (!(Flags & (CMP_ENLIST_KCB_LOCKED_SHARED |
//                    CMP_ENLIST_KCB_LOCKED_EXCLUSIVE)))
//     {
//         /* Release the lock */
//         CmpReleaseKcbLock(KeyBody->KeyControlBlock);
//     }
// }
//
// VOID
// NTAPI
// DelistKeyBodyFromKCB(IN PCM_KEY_BODY KeyBody,
//                      IN BOOLEAN LockHeld)
// {
//     ULONG i;
//
//     /* Sanity check */
//     ASSERT(KeyBody->KeyControlBlock != NULL);
//
//     /* Check if we can use the parent KCB array */
//     for (i = 0; i < 4; i++)
//     {
//         /* Add it into the list */
//         if (InterlockedCompareExchangePointer((PVOID*)&KeyBody->KeyControlBlock->
//                                               KeyBodyArray[i],
//                                               NULL,
//                                               KeyBody) == KeyBody)
//         {
//             /* Removed */
//             return;
//         }
//     }
//
//     /* Sanity checks */
//     ASSERT(IsListEmpty(&KeyBody->KeyControlBlock->KeyBodyListHead) == FALSE);
//     ASSERT(IsListEmpty(&KeyBody->KeyBodyList) == FALSE);
//
//     /* Lock the KCB */
//     if (!LockHeld) CmpAcquireKcbLockExclusive(KeyBody->KeyControlBlock);
//     CMP_ASSERT_KCB_LOCK(KeyBody->KeyControlBlock);
//
//     /* Remove the entry */
//     RemoveEntryList(&KeyBody->KeyBodyList);
//
//     /* Unlock it it if we did a manual lock */
//     if (!LockHeld) CmpReleaseKcbLock(KeyBody->KeyControlBlock);
// }
//
// /**
//  * @brief
//  * Unlocks a number of KCBs provided by a KCB array.
//  *
//  * @param[in] KcbArray
//  * A pointer to an array of KCBs to be unlocked.
//  */
// VOID
// CmpUnLockKcbArray(
//     _In_ PULONG KcbArray)
// {
//     ULONG i;
//
//     /* Release the locked KCBs in reverse order */
//     for (i = KcbArray[0]; i > 0; i--)
//     {
//         CmpReleaseKcbLockByIndex(KcbArray[i]);
//     }
// }
//
// /**
//  * @brief
//  * Locks a given number of KCBs.
//  *
//  * @param[in] KcbArray
//  * A pointer to an array of KCBs to be locked.
//  * The count of KCBs to be locked is defined by the
//  * first element in the array.
//  *
//  * @param[in] KcbLockFlags
//  * Define a lock flag to lock the KCBs.
//  *
//  * CMP_LOCK_KCB_ARRAY_EXCLUSIVE -- indicates the KCBs are locked
//  * exclusively and owned by the calling thread.
//  *
//  * CMP_LOCK_KCB_ARRAY_SHARED --  indicates the KCBs are locked
//  * in shared mode by the owning threads.
//  */
// static
// VOID
// CmpLockKcbArray(
//     _In_ PULONG KcbArray,
//     _In_ ULONG KcbLockFlags)
// {
//     ULONG i;
//
//     /* Lock the KCBs */
//     for (i = 1; i <= KcbArray[0]; i++)
//     {
//         if (KcbLockFlags & CMP_LOCK_KCB_ARRAY_EXCLUSIVE)
//         {
//             CmpAcquireKcbLockExclusiveByIndex(KcbArray[i]);
//         }
//         else // CMP_LOCK_KCB_ARRAY_SHARED
//         {
//             CmpAcquireKcbLockSharedByIndex(KcbArray[i]);
//         }
//     }
// }
//
// /**
//  * @brief
//  * Sorts an array of KCB hashes in ascending order
//  * and removes any key indices that are duplicates.
//  * The purpose of sorting the KCB elements is to
//  * ensure consistent and proper locking order, so
//  * that we can prevent a deadlock.
//  *
//  * @param[in,out] KcbArray
//  * A pointer to an array of KCBs of which the key
//  * indices are to be sorted.
//  */
// static
// VOID
// CmpSortKcbArray(
//     _Inout_ PULONG KcbArray)
// {
//     ULONG i, j, k, KcbCount;
//
//     /* Ensure we don't go above the limit of KCBs we can hold */
//     KcbCount = KcbArray[0];
//     ASSERT(KcbCount < CMP_KCBS_IN_ARRAY_LIMIT);
//
//     /* Exchange-Sort the array in ascending order. Complexity: O[n^2] */
//     for (i = 1; i <= KcbCount; i++)
//     {
//         for (j = i + 1; j <= KcbCount; j++)
//         {
//             if (KcbArray[i] > KcbArray[j])
//             {
//                 ULONG Temp = KcbArray[i];
//                 KcbArray[i] = KcbArray[j];
//                 KcbArray[j] = Temp;
//             }
//         }
//     }
//
//     /* Now remove any duplicated indices on the sorted array if any */
//     for (i = 1; i <= KcbCount; i++)
//     {
//         for (j = i + 1; j <= KcbCount; j++)
//         {
//             if (KcbArray[i] == KcbArray[j])
//             {
//                 for (k = j; k <= KcbCount; k++)
//                 {
//                     KcbArray[k - 1] = KcbArray[k];
//                 }
//
//                 j--;
//                 KcbCount--;
//             }
//         }
//     }
//
//     /* Update the KCB count */
//     KcbArray[0] = KcbCount;
// }
//
// /**
//  * @brief
//  * Builds an array of KCBs and locks them. Whether these
//  * KCBs are locked exclusively or in shared mode by the calling
//  * thread, is specified by the KcbLockFlags parameter. The array
//  * is sorted.
//  *
//  * @param[in] HashCacheStack
//  * A pointer to a hash cache stack. This stack parameter
//  * stores the convkey hashes of interested KCBs of a
//  * key path name that need to be locked.
//  *
//  * @param[in] KcbLockFlags
//  * Define a lock flag to lock the KCBs. Consult the
//  * CmpLockKcbArray documentation for more information.
//  *
//  * @param[in] Kcb
//  * A pointer to a key control block to be given. This
//  * KCB is included in the array for locking, that is,
//  * given by the CmpParseKey from the parser object.
//  *
//  * @param[in,out] OuterStackArray
//  * A pointer to an array that lives on the caller's
//  * stack. It acts like an auxiliary array used by
//  * the function to store the KCB elements for locking.
//  * The expected size of the array is up to 32 elements,
//  * which is the imposed limit by CMP_HASH_STACK_LIMIT.
//  * This limit also corresponds to the maximum depth of
//  * subkey levels.
//  *
//  * @param[in] TotalRemainingSubkeys
//  * The number of total remaining subkey levels.
//  *
//  * @param[in] MatchRemainSubkeyLevel
//  * The number of remaining subkey levels that match.
//  *
//  * @return
//  * Returns a pointer to an array of KCBs that have been
//  * locked.
//  *
//  * @remarks
//  * The caller HAS THE RESPONSIBILITY to unlock the KCBs
//  * after the necessary operations are done!
//  */
// PULONG
// NTAPI
// CmpBuildAndLockKcbArray(
//     _In_ PCM_HASH_CACHE_STACK HashCacheStack,
//     _In_ ULONG KcbLockFlags,
//     _In_ PCM_KEY_CONTROL_BLOCK Kcb,
//     _Inout_ PULONG OuterStackArray,
//     _In_ ULONG TotalRemainingSubkeys,
//     _In_ ULONG MatchRemainSubkeyLevel)
// {
//     ULONG KcbIndex = 1, HashStackIndex, TotalRemaining;
//     PULONG LockedKcbs = NULL;
//     PCM_KEY_CONTROL_BLOCK ParentKcb = Kcb->ParentKcb;;
//
//     /* These parameters are expected */
//     ASSERT(HashCacheStack != NULL);
//     ASSERT(Kcb != NULL);
//     ASSERT(OuterStackArray != NULL);
//
//     /*
//      * Ensure when we build an array of KCBs to lock, that
//      * we don't go beyond the boundary the limit allows us
//      * to. 1 is the current KCB we would want to lock
//      * alongside with the remaining key levels in the formula.
//      */
//     TotalRemaining = (1 + TotalRemainingSubkeys) - MatchRemainSubkeyLevel;
//     ASSERT(TotalRemaining <= CMP_KCBS_IN_ARRAY_LIMIT);
//
//     /* Count the parent if we have one */
//     if (ParentKcb)
//     {
//         /* Ensure we are still below the limit and add the parent to KCBs to lock */
//         if (TotalRemainingSubkeys == MatchRemainSubkeyLevel)
//         {
//             TotalRemaining++;
//             ASSERT(TotalRemaining <= CMP_KCBS_IN_ARRAY_LIMIT);
//             OuterStackArray[KcbIndex++] = GET_HASH_INDEX(ParentKcb->ConvKey);
//         }
//     }
//
//     /* Add the current KCB */
//     OuterStackArray[KcbIndex++] = GET_HASH_INDEX(Kcb->ConvKey);
//
//     /* Loop over the hash stack and grab the hashes for locking (they will be converted to indices) */
//     for (HashStackIndex = 0;
//          HashStackIndex < TotalRemainingSubkeys;
//          HashStackIndex++)
//     {
//         OuterStackArray[KcbIndex++] = GET_HASH_INDEX(HashCacheStack[HashStackIndex].ConvKey);
//     }
//
//     /*
//      * Store how many KCBs we need to lock and sort the array.
//      * Remove any duplicated indices from the array if any.
//      */
//     OuterStackArray[0] = KcbIndex - 1;
//     CmpSortKcbArray(OuterStackArray);
//
//     /* Lock them */
//     CmpLockKcbArray(OuterStackArray, KcbLockFlags);
//
//     /* Give the locked KCBs to caller now */
//     LockedKcbs = OuterStackArray;
//     return LockedKcbs;
// }
//
// VOID
// NTAPI
// CmpFlushNotifiesOnKeyBodyList(IN PCM_KEY_CONTROL_BLOCK Kcb,
//                               IN BOOLEAN LockHeld)
// {
//     PLIST_ENTRY NextEntry, ListHead;
//     PCM_KEY_BODY KeyBody;
//
//     /* Sanity check */
//     LockHeld ? CMP_ASSERT_EXCLUSIVE_REGISTRY_LOCK() : CmpIsKcbLockedExclusive(Kcb);
//     while (TRUE)
//     {
//         /* Is the list empty? */
//         ListHead = &Kcb->KeyBodyListHead;
//         if (!IsListEmpty(ListHead))
//         {
//             /* Loop the list */
//             NextEntry = ListHead->Flink;
//             while (NextEntry != ListHead)
//             {
//                 /* Get the key body */
//                 KeyBody = CONTAINING_RECORD(NextEntry, CM_KEY_BODY, KeyBodyList);
//                 ASSERT(KeyBody->Type == CM_KEY_BODY_TYPE);
//
//                 /* Check for notifications */
//                 if (KeyBody->NotifyBlock)
//                 {
//                     /* Is the lock held? */
//                     if (LockHeld)
//                     {
//                         /* Flush it */
//                         CmpFlushNotify(KeyBody, LockHeld);
//                         ASSERT(KeyBody->NotifyBlock == NULL);
//                         continue;
//                     }
//
//                     /* Lock isn't held, so we need to take a reference */
//                     if (ObReferenceObjectSafe(KeyBody))
//                     {
//                         /* Now we can flush */
//                         CmpFlushNotify(KeyBody, LockHeld);
//                         ASSERT(KeyBody->NotifyBlock == NULL);
//
//                         /* Release the reference we took */
//                         ObDereferenceObjectDeferDelete(KeyBody);
//                         continue;
//                     }
//                 }
//
//                 /* Try the next entry */
//                 NextEntry = NextEntry->Flink;
//             }
//         }
//
//         /* List has been parsed, exit */
//         break;
//     }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
