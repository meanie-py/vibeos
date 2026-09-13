//! Persist a prepared self-relative descriptor independently of KCB lifetime.
//! The caller owns the key's exclusive KCB lock and has allocated its replacement
//! cache before calling: after this succeeds, publishing that cache cannot fail.
use ntkrnl_sys::base::{NTSTATUS, STATUS_INSUFFICIENT_RESOURCES, STATUS_SUCCESS};
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{
    CmpLockHiveFlusherShared, CmpUnlockHiveFlusher, ExAcquirePushLockExclusive,
    ExReleasePushLockExclusive, KeEnterCriticalRegion, KeLeaveCriticalRegion, CM_KEY_SECURITY,
    PCMHIVE, PCM_KEY_CONTROL_BLOCK, PCM_KEY_NODE, PCM_KEY_SECURITY, PHHIVE,
};

const NIL: u32 = u32::MAX;
const NK: u16 = 0x6b6e;
const SK: u16 = 0x6b73;
// sdk/include/psdk/ntstatus.h; absent from the current ntkrnl-sys constants island.
const STATUS_REGISTRY_CORRUPT: NTSTATUS = 0xc000014cu32 as i32;

extern "C" {
    fn HvGetCell(hive: PHHIVE, cell: u32) -> *mut core::ffi::c_void;
    fn HvReleaseCell(hive: PHHIVE, cell: u32);
}
extern "system" {
    fn HvAllocateCell(hive: PHHIVE, size: u32, storage: u32, vicinity: u32) -> u32;
    fn HvFreeCell(hive: PHHIVE, cell: u32);
    fn HvMarkCellDirty(hive: PHHIVE, cell: u32, shared: u8) -> u8;
}

struct Cell {
    hive: PHHIVE,
    index: u32,
    ptr: *mut core::ffi::c_void,
}
impl Cell {
    unsafe fn get(hive: PHHIVE, index: u32) -> Result<Self, NTSTATUS> {
        if index == NIL {
            return Err(STATUS_REGISTRY_CORRUPT);
        }
        let ptr = HvGetCell(hive, index);
        if ptr.is_null() {
            return Err(STATUS_INSUFFICIENT_RESOURCES);
        }
        Ok(Self { hive, index, ptr })
    }
    unsafe fn dirty(&self) -> Result<(), NTSTATUS> {
        if HvMarkCellDirty(self.hive, self.index, 0) == 0 {
            Err(STATUS_INSUFFICIENT_RESOURCES)
        } else {
            Ok(())
        }
    }
    unsafe fn security(&self) -> Result<PCM_KEY_SECURITY, NTSTATUS> {
        let sk = self.ptr as PCM_KEY_SECURITY;
        if (*sk).Signature != SK || (*sk).ReferenceCount == 0 {
            Err(STATUS_REGISTRY_CORRUPT)
        } else {
            Ok(sk)
        }
    }
}
impl Drop for Cell {
    fn drop(&mut self) {
        unsafe {
            HvReleaseCell(self.hive, self.index);
        }
    }
}

struct Locks(PCMHIVE);
impl Drop for Locks {
    fn drop(&mut self) {
        unsafe {
            ExReleasePushLockExclusive(core::ptr::addr_of_mut!((*self.0).SecurityLock));
            KeLeaveCriticalRegion();
            CmpUnlockHiveFlusher(self.0);
        }
    }
}

pub(crate) unsafe fn persist(
    kcb: PCM_KEY_CONTROL_BLOCK,
    descriptor: *const u8,
    length: u32,
) -> NTSTATUS {
    match persist_inner(kcb, descriptor, length) {
        Ok(()) => STATUS_SUCCESS,
        Err(status) => status,
    }
}

unsafe fn persist_inner(
    kcb: PCM_KEY_CONTROL_BLOCK,
    descriptor: *const u8,
    length: u32,
) -> Result<(), NTSTATUS> {
    if kcb.is_null() || descriptor.is_null() {
        return Err(STATUS_INVALID_PARAMETER);
    }
    let hive = (*kcb).__bindgen_anon_2.__bindgen_anon_1.KeyHive;
    let index = (*kcb).__bindgen_anon_2.__bindgen_anon_1.KeyCell;
    if hive.is_null() || index == NIL {
        return Err(STATUS_INVALID_PARAMETER);
    }
    if (*hive).ReadOnly != 0 {
        return Err(STATUS_MEDIA_WRITE_PROTECTED);
    }
    let size = (core::mem::offset_of!(CM_KEY_SECURITY, Descriptor) as u32)
        .checked_add(length)
        .filter(|size| *size <= u32::MAX - 16)
        .ok_or(STATUS_INVALID_SECURITY_DESCR)?;

    // Parse/create already holds the flusher resource shared; its recursive shared
    // acquisition is intentional. SecurityLock serializes shared sk refcounts.
    let cmhive = hive as PCMHIVE;
    CmpLockHiveFlusherShared(cmhive);
    KeEnterCriticalRegion();
    ExAcquirePushLockExclusive(core::ptr::addr_of_mut!((*cmhive).SecurityLock));
    let _locks = Locks(cmhive);

    let node_cell = Cell::get(hive, index)?;
    let node = node_cell.ptr as PCM_KEY_NODE;
    if (*node).Signature != NK {
        return Err(STATUS_REGISTRY_CORRUPT);
    }
    let previous = (*node).Security;
    let old_cell = if previous == NIL {
        None
    } else {
        Some(Cell::get(hive, previous)?)
    };
    let old = match old_cell.as_ref() {
        Some(cell) => cell.security()?,
        None => core::ptr::null_mut(),
    };
    // Validate and retain every list neighbor BEFORE changing any descriptor or
    // reference count. No fallible operation may remain after publication.
    let remove_old = !old.is_null() && (*old).ReferenceCount == 1;
    let forward = if remove_old {
        Some(Cell::get(hive, (*old).Flink)?)
    } else {
        None
    };
    let backward = if remove_old {
        Some(Cell::get(hive, (*old).Blink)?)
    } else {
        None
    };
    if let (Some(f), Some(b)) = (&forward, &backward) {
        let fs = f.security()?;
        let bs = b.security()?;
        if (*fs).Blink != previous || (*bs).Flink != previous {
            return Err(STATUS_REGISTRY_CORRUPT);
        }
    }
    node_cell.dirty()?;
    if let Some(cell) = &old_cell {
        cell.dirty()?;
    }
    if let Some(cell) = &forward {
        cell.dirty()?;
    }
    if let Some(cell) = &backward {
        cell.dirty()?;
    }

    // Cell-index bit 31 is the cmlib storage class, including volatile hive roots.
    let new_index = HvAllocateCell(hive, size, index >> 31, NIL);
    if new_index == NIL {
        return Err(STATUS_INSUFFICIENT_RESOURCES);
    }
    let new_cell = match Cell::get(hive, new_index) {
        Ok(cell) => cell,
        Err(status) => {
            HvFreeCell(hive, new_index);
            return Err(status);
        }
    };
    let new = new_cell.ptr as PCM_KEY_SECURITY;
    (*new).Signature = SK;
    (*new).Reserved = 0;
    (*new).Flink = new_index;
    (*new).Blink = new_index;
    (*new).ReferenceCount = 1;
    (*new).DescriptorLength = length;
    core::ptr::copy_nonoverlapping(
        descriptor,
        core::ptr::addr_of_mut!((*new).Descriptor).cast::<u8>(),
        length as usize,
    );

    if let (Some(forward), Some(backward)) = (&forward, &backward) {
        let f = forward.ptr as PCM_KEY_SECURITY;
        let b = backward.ptr as PCM_KEY_SECURITY;
        (*f).Blink = (*old).Blink;
        (*b).Flink = (*old).Flink;
    } else if !old.is_null() {
        (*old).ReferenceCount -= 1;
    }
    (*node).Security = new_index;
    drop(new_cell);
    drop(forward);
    drop(backward);
    drop(old_cell);
    if remove_old {
        HvFreeCell(hive, previous);
    }
    Ok(())
}
