use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    _INTERNAL_BCB, PINTERNAL_BCB, PROS_SHARED_CACHE_MAP, PROS_VACB,
    _NPAGED_LOOKASIDE_LIST,
    PFILE_OBJECT, PLARGE_INTEGER, PIO_STATUS_BLOCK, ERESOURCE_THREAD,
    CcFlushCache, CcRosEnsureVacbResident, CcRosGetVacb, CcRosMarkDirtyVacb, CcRosReleaseVacb,
    ExAcquireResourceExclusiveLite, ExAcquireSharedStarveExclusive, ExDeleteResourceLite,
    ExInitializeResourceLite, ExReleaseResourceForThreadLite, ExReleaseResourceLite,
    MmMakeSegmentDirty, PsGetCurrentThread,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "C" {
    static mut iBcbLookasideList: _NPAGED_LOOKASIDE_LIST;
    fn ExAllocateFromNPagedLookasideList(list: *mut _NPAGED_LOOKASIDE_LIST) -> PVOID;
    fn ExFreeToNPagedLookasideList(list: *mut _NPAGED_LOOKASIDE_LIST, entry: PVOID);
}

#[no_mangle]
pub static mut CcMapDataWait: ULONG = 0;
#[no_mangle]
pub static mut CcMapDataNoWait: ULONG = 0;
#[no_mangle]
pub static mut CcPinReadWait: ULONG = 0;
#[no_mangle]
pub static mut CcPinReadNoWait: ULONG = 0;
#[no_mangle]
pub static mut CcPinMappedDataCount: ULONG = 0;

const VACB_MAPPING_GRANULARITY: u64 = 262144;

const MAP_WAIT: ULONG = 1;
const MAP_NO_READ: ULONG = 16;
const PIN_WAIT: ULONG = 1;
const PIN_EXCLUSIVE: ULONG = 2;
const PIN_NO_READ: ULONG = 4;
const PIN_IF_BCB: ULONG = 8;

#[inline]
unsafe fn BooleanFlagOn(flags: ULONG, bit: ULONG) -> BOOLEAN {
    if (flags & bit) != 0 { 1u8 } else { 0u8 }
}

unsafe fn CcpFindBcb(
    shared_cache_map: PROS_SHARED_CACHE_MAP,
    file_offset: PLARGE_INTEGER,
    length: ULONG,
    pinned: BOOLEAN,
) -> PINTERNAL_BCB {
    let mut found = 0u8;
    let mut next_entry = (*shared_cache_map).BcbList.Flink;

    while next_entry != core::ptr::addr_of_mut!((*shared_cache_map).BcbList) as PLIST_ENTRY {
        let bcb = ((next_entry as usize) - core::mem::offset_of!(_INTERNAL_BCB, BcbEntry)) as PINTERNAL_BCB;

        if (*bcb).PFCB.MappedFileOffset.QuadPart <= (*file_offset).QuadPart
            && ((*bcb).PFCB.MappedFileOffset.QuadPart + (*bcb).PFCB.MappedLength as i64)
                >= ((*file_offset).QuadPart + length as i64)
        {
            if (pinned != 0 && (*bcb).PinCount > 0) || (pinned == 0 && (*bcb).PinCount == 0) {
                found = 1u8;
                break;
            }
        }

        next_entry = (*next_entry).Flink;
    }

    if found != 0 {
        ((next_entry as usize) - core::mem::offset_of!(_INTERNAL_BCB, BcbEntry)) as PINTERNAL_BCB
    } else {
        core::ptr::null_mut()
    }
}

unsafe fn CcpDereferenceBcb(shared_cache_map: PROS_SHARED_CACHE_MAP, bcb: PINTERNAL_BCB) {
    let mut old_irql: KIRQL = 0;

    KeAcquireSpinLock(
        core::ptr::addr_of_mut!((*shared_cache_map).BcbSpinLock) as *mut _,
        &mut old_irql,
    );
    (*bcb).RefCount -= 1;
    let ref_count = (*bcb).RefCount;
    if ref_count == 0 {
        RemoveEntryList(core::ptr::addr_of_mut!((*bcb).BcbEntry) as PLIST_ENTRY);
        KeReleaseSpinLock(
            core::ptr::addr_of_mut!((*shared_cache_map).BcbSpinLock) as *mut _,
            old_irql,
        );

        CcRosReleaseVacb(shared_cache_map, (*bcb).Vacb, 0u8, 0u8);

        ExDeleteResourceLite(core::ptr::addr_of_mut!((*bcb).Lock));
        ExFreeToNPagedLookasideList(core::ptr::addr_of_mut!(iBcbLookasideList), bcb as PVOID);
    } else {
        KeReleaseSpinLock(
            core::ptr::addr_of_mut!((*shared_cache_map).BcbSpinLock) as *mut _,
            old_irql,
        );
    }
}

unsafe fn CcpGetAppropriateBcb(
    shared_cache_map: PROS_SHARED_CACHE_MAP,
    vacb: PROS_VACB,
    file_offset: PLARGE_INTEGER,
    length: ULONG,
    pin_flags: ULONG,
    to_pin: BOOLEAN,
) -> PVOID {
    let mut old_irql: KIRQL = 0;
    let mut result: BOOLEAN;

    let i_bcb = ExAllocateFromNPagedLookasideList(core::ptr::addr_of_mut!(iBcbLookasideList)) as PINTERNAL_BCB;
    if i_bcb.is_null() {
        CcRosReleaseVacb(shared_cache_map, vacb, 0u8, 0u8);
        return core::ptr::null_mut();
    }

    core::ptr::write_bytes(i_bcb as *mut u8, 0, core::mem::size_of::<_INTERNAL_BCB>());
    (*i_bcb).PFCB.NodeTypeCode = 0x2FD;
    (*i_bcb).PFCB.NodeByteSize = 0;
    (*i_bcb).PFCB.MappedLength = length;
    (*i_bcb).PFCB.MappedFileOffset = *file_offset;
    (*i_bcb).Vacb = vacb;
    (*i_bcb).PinCount = 0;
    (*i_bcb).RefCount = 1;
    ExInitializeResourceLite(core::ptr::addr_of_mut!((*i_bcb).Lock));

    KeAcquireSpinLock(
        core::ptr::addr_of_mut!((*shared_cache_map).BcbSpinLock) as *mut _,
        &mut old_irql,
    );

    let mut dup_bcb = CcpFindBcb(shared_cache_map, file_offset, length, to_pin);
    if !dup_bcb.is_null() {
        (*dup_bcb).RefCount += 1;
        result = 1u8;
        KeReleaseSpinLock(
            core::ptr::addr_of_mut!((*shared_cache_map).BcbSpinLock) as *mut _,
            old_irql,
        );

        if to_pin != 0 {
            // Pin the DUP's lock — NOT the about-to-be-discarded temp iBcb's.
            // (Upstream ReactOS pin.c acquires iBcb->Lock here and then
            // ExDeleteResourceLite's it WHILE HELD, and CcUnpinData later
            // releases DupBcb->Lock which was never acquired — an over-release
            // that corrupts the ERESOURCE owner table. That upstream bug is what
            // produced our "ExFreePool of an ETHREAD-interior pointer" crashes
            // once overlapping FAT-page pins started exercising the dup path.)
            if BooleanFlagOn(pin_flags, PIN_EXCLUSIVE) != 0 {
                result = ExAcquireResourceExclusiveLite(
                    core::ptr::addr_of_mut!((*dup_bcb).Lock),
                    BooleanFlagOn(pin_flags, PIN_WAIT),
                );
            } else {
                result = ExAcquireSharedStarveExclusive(
                    core::ptr::addr_of_mut!((*dup_bcb).Lock),
                    BooleanFlagOn(pin_flags, PIN_WAIT),
                );
            }

            if result != 0 {
                (*dup_bcb).PinCount += 1;
            } else {
                CcpDereferenceBcb(shared_cache_map, dup_bcb);
                dup_bcb = core::ptr::null_mut();
            }
        }

        if !dup_bcb.is_null() {
            CcRosReleaseVacb(shared_cache_map, vacb, 0u8, 0u8);
            ExDeleteResourceLite(core::ptr::addr_of_mut!((*i_bcb).Lock));
            ExFreeToNPagedLookasideList(core::ptr::addr_of_mut!(iBcbLookasideList), i_bcb as PVOID);
        }

        return dup_bcb as PVOID;
    } else {
        if to_pin != 0 {
            (*i_bcb).PinCount += 1;

            if BooleanFlagOn(pin_flags, PIN_EXCLUSIVE) != 0 {
                result = ExAcquireResourceExclusiveLite(
                    core::ptr::addr_of_mut!((*i_bcb).Lock),
                    BooleanFlagOn(pin_flags, PIN_WAIT),
                );
            } else {
                result = ExAcquireSharedStarveExclusive(
                    core::ptr::addr_of_mut!((*i_bcb).Lock),
                    BooleanFlagOn(pin_flags, PIN_WAIT),
                );
            }
        }

        InsertTailList(
            core::ptr::addr_of_mut!((*shared_cache_map).BcbList) as PLIST_ENTRY,
            core::ptr::addr_of_mut!((*i_bcb).BcbEntry) as PLIST_ENTRY,
        );
        KeReleaseSpinLock(
            core::ptr::addr_of_mut!((*shared_cache_map).BcbSpinLock) as *mut _,
            old_irql,
        );
    }

    i_bcb as PVOID
}

unsafe fn CcpPinData(
    shared_cache_map: PROS_SHARED_CACHE_MAP,
    file_offset: PLARGE_INTEGER,
    length: ULONG,
    flags: ULONG,
    bcb: *mut PVOID,
    buffer: *mut PVOID,
) -> BOOLEAN {
    let mut old_irql: KIRQL = 0;
    let mut length = length;
    let vacb_offset = ((*file_offset).QuadPart % VACB_MAPPING_GRANULARITY as i64) as ULONG;

    if (vacb_offset + length) as u64 > VACB_MAPPING_GRANULARITY {
        length = (VACB_MAPPING_GRANULARITY as ULONG) - vacb_offset;
    }

    KeAcquireSpinLock(
        core::ptr::addr_of_mut!((*shared_cache_map).BcbSpinLock) as *mut _,
        &mut old_irql,
    );
    let mut new_bcb = CcpFindBcb(shared_cache_map, file_offset, length, 1u8);

    if !new_bcb.is_null() {
        (*new_bcb).RefCount += 1;
        KeReleaseSpinLock(
            core::ptr::addr_of_mut!((*shared_cache_map).BcbSpinLock) as *mut _,
            old_irql,
        );

        let result = if BooleanFlagOn(flags, PIN_EXCLUSIVE) != 0 {
            ExAcquireResourceExclusiveLite(
                core::ptr::addr_of_mut!((*new_bcb).Lock),
                BooleanFlagOn(flags, PIN_WAIT),
            )
        } else {
            ExAcquireSharedStarveExclusive(
                core::ptr::addr_of_mut!((*new_bcb).Lock),
                BooleanFlagOn(flags, PIN_WAIT),
            )
        };

        if result == 0 {
            CcpDereferenceBcb(shared_cache_map, new_bcb);
            return 0u8;
        }

        (*new_bcb).PinCount += 1;
    } else {
        KeReleaseSpinLock(
            core::ptr::addr_of_mut!((*shared_cache_map).BcbSpinLock) as *mut _,
            old_irql,
        );

        if BooleanFlagOn(flags, PIN_IF_BCB) != 0 {
            return 0u8;
        }

        let r_offset = ((*file_offset).QuadPart / VACB_MAPPING_GRANULARITY as i64) * VACB_MAPPING_GRANULARITY as i64;
        let mut vacb: PROS_VACB = core::ptr::null_mut();
        let status = CcRosGetVacb(shared_cache_map, r_offset, &mut vacb);
        if status < 0 {
            // DUMMY_PSEH: ExRaiseStatus would be caught by caller's SEH — return FALSE instead
            return 0u8;
        }

        new_bcb = CcpGetAppropriateBcb(shared_cache_map, vacb, file_offset, length, flags, 1u8) as PINTERNAL_BCB;
        if new_bcb.is_null() {
            CcRosReleaseVacb(shared_cache_map, vacb, 0u8, 0u8);
            return 0u8;
        }
    }

    let result = CcRosEnsureVacbResident(
        (*new_bcb).Vacb,
        BooleanFlagOn(flags, PIN_WAIT),
        BooleanFlagOn(flags, PIN_NO_READ),
        vacb_offset,
        length,
    );

    if result == 0 {
        CcUnpinData(core::ptr::addr_of_mut!((*new_bcb).PFCB) as PVOID);
        *bcb = core::ptr::null_mut();
        *buffer = core::ptr::null_mut();
    } else {
        *bcb = core::ptr::addr_of_mut!((*new_bcb).PFCB) as PVOID;
        *buffer = ((*(*new_bcb).Vacb).BaseAddress as usize + vacb_offset as usize) as PVOID;
    }

    result
}

#[no_mangle]
pub unsafe extern "system" fn CcMapData(
    file_object: PFILE_OBJECT,
    file_offset: PLARGE_INTEGER,
    length: ULONG,
    flags: ULONG,
    p_bcb: *mut PVOID,
    p_buffer: *mut PVOID,
) -> BOOLEAN {
    let file_object = opaque_ptr(file_object);
    let file_offset = opaque_ptr(file_offset);
    let p_bcb = opaque_ptr(p_bcb);
    let p_buffer = opaque_ptr(p_buffer);
    let shared_cache_map = (*(*file_object).SectionObjectPointer).SharedCacheMap as PROS_SHARED_CACHE_MAP;

    if (flags & MAP_WAIT) != 0 {
        CcMapDataWait += 1;
    } else {
        CcMapDataNoWait += 1;
    }

    let vacb_offset = ((*file_offset).QuadPart % VACB_MAPPING_GRANULARITY as i64) as ULONG;
    let mut length = length;
    if (vacb_offset + length) as u64 > VACB_MAPPING_GRANULARITY {
        length = (VACB_MAPPING_GRANULARITY as ULONG) - vacb_offset;
    }

    let mut old_irql: KIRQL = 0;
    KeAcquireSpinLock(
        core::ptr::addr_of_mut!((*shared_cache_map).BcbSpinLock) as *mut _,
        &mut old_irql,
    );
    let mut i_bcb = CcpFindBcb(shared_cache_map, file_offset, length, 0u8);

    if i_bcb.is_null() {
        KeReleaseSpinLock(
            core::ptr::addr_of_mut!((*shared_cache_map).BcbSpinLock) as *mut _,
            old_irql,
        );
        #[cfg(target_arch = "aarch64")]
        {
            core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
        }

        let mut vacb: PROS_VACB = core::ptr::null_mut();
        let status = CcRosGetVacb(shared_cache_map, (*file_offset).QuadPart, &mut vacb);
        if status < 0 {
            // DUMMY_PSEH: ExRaiseStatus would be caught by caller's SEH — return FALSE instead
            core::ptr::write_volatile(p_bcb, core::ptr::null_mut());
            *p_buffer = core::ptr::null_mut();
            return 0u8;
        }

        i_bcb = CcpGetAppropriateBcb(shared_cache_map, vacb, file_offset, length, 0, 0u8) as PINTERNAL_BCB;
        if i_bcb.is_null() {
            CcRosReleaseVacb(shared_cache_map, vacb, 0u8, 0u8);
            core::ptr::write_volatile(p_bcb, core::ptr::null_mut());
            *p_buffer = core::ptr::null_mut();
            return 0u8;
        }
    } else {
        (*i_bcb).RefCount += 1;
        KeReleaseSpinLock(
            core::ptr::addr_of_mut!((*shared_cache_map).BcbSpinLock) as *mut _,
            old_irql,
        );
    }

    let result = CcRosEnsureVacbResident(
        (*i_bcb).Vacb,
        BooleanFlagOn(flags, MAP_WAIT),
        BooleanFlagOn(flags, MAP_NO_READ),
        vacb_offset,
        length,
    );

    if result == 0 {
        CcpDereferenceBcb(shared_cache_map, i_bcb);
        core::ptr::write_volatile(p_bcb, core::ptr::null_mut());
        *p_buffer = core::ptr::null_mut();
    } else {
        core::ptr::write_volatile(p_bcb, core::ptr::addr_of_mut!((*i_bcb).PFCB) as PVOID);
        let base_addr = (*(*i_bcb).Vacb).BaseAddress;
        *p_buffer = (base_addr as usize + vacb_offset as usize) as PVOID;
    }

    result
}

#[no_mangle]
pub unsafe extern "system" fn CcPinMappedData(
    file_object: PFILE_OBJECT,
    file_offset: PLARGE_INTEGER,
    length: ULONG,
    flags: ULONG,
    bcb: *mut PVOID,
) -> BOOLEAN {
    let file_object = opaque_ptr(file_object);
    let file_offset = opaque_ptr(file_offset);
    let bcb = opaque_ptr(bcb);
    let shared_cache_map = (*(*file_object).SectionObjectPointer).SharedCacheMap as PROS_SHARED_CACHE_MAP;

    if (*shared_cache_map).PinAccess == 0 {
        return 0u8;
    }

    let i_bcb = if !(*bcb).is_null() {
        (((*bcb) as usize) - core::mem::offset_of!(_INTERNAL_BCB, PFCB)) as PINTERNAL_BCB
    } else {
        core::ptr::null_mut()
    };

    CcPinMappedDataCount += 1;

    let mut buffer: PVOID = core::ptr::null_mut();
    let result = CcpPinData(shared_cache_map, file_offset, length, flags, bcb, &mut buffer);
    if result != 0 && !i_bcb.is_null() {
        CcUnpinData(core::ptr::addr_of_mut!((*i_bcb).PFCB) as PVOID);
    }

    result
}

#[no_mangle]
pub unsafe extern "system" fn CcPinRead(
    file_object: PFILE_OBJECT,
    file_offset: PLARGE_INTEGER,
    length: ULONG,
    flags: ULONG,
    bcb: *mut PVOID,
    buffer: *mut PVOID,
) -> BOOLEAN {
    let file_object = opaque_ptr(file_object);
    let file_offset = opaque_ptr(file_offset);
    let bcb = opaque_ptr(bcb);
    let buffer = opaque_ptr(buffer);
    let shared_cache_map = (*(*file_object).SectionObjectPointer).SharedCacheMap as PROS_SHARED_CACHE_MAP;

    if (*shared_cache_map).PinAccess == 0 {
        return 0u8;
    }

    if (flags & PIN_WAIT) != 0 {
        CcPinReadWait += 1;
    } else {
        CcPinReadNoWait += 1;
    }

    CcpPinData(shared_cache_map, file_offset, length, flags, bcb, buffer)
}

#[no_mangle]
pub unsafe extern "system" fn CcPreparePinWrite(
    file_object: PFILE_OBJECT,
    file_offset: PLARGE_INTEGER,
    length: ULONG,
    zero: BOOLEAN,
    flags: ULONG,
    bcb: *mut PVOID,
    buffer: *mut PVOID,
) -> BOOLEAN {
    let file_object = opaque_ptr(file_object);
    let file_offset = opaque_ptr(file_offset);
    let bcb = opaque_ptr(bcb);
    let buffer = opaque_ptr(buffer);
    CcPinRead(file_object, file_offset, length, flags, bcb, buffer)
}

#[no_mangle]
pub unsafe extern "system" fn CcSetDirtyPinnedData(bcb: PVOID, lsn: PLARGE_INTEGER) {
    let bcb = opaque_ptr(bcb);
    let lsn = opaque_ptr(lsn);
    let i_bcb = ((bcb as usize) - core::mem::offset_of!(_INTERNAL_BCB, PFCB)) as PINTERNAL_BCB;
    let vacb = (*i_bcb).Vacb;

    MmMakeSegmentDirty(
        (*(*(*vacb).SharedCacheMap).FileObject).SectionObjectPointer,
        (*i_bcb).PFCB.MappedFileOffset.QuadPart,
        (*i_bcb).PFCB.MappedLength,
    );

    if (*vacb).Dirty == 0 {
        CcRosMarkDirtyVacb(vacb);
    }
}

#[no_mangle]
pub unsafe extern "system" fn CcUnpinData(bcb: PVOID) {
    let bcb = opaque_ptr(bcb);
    CcUnpinDataForThread(bcb, PsGetCurrentThread() as ERESOURCE_THREAD);
}

#[no_mangle]
pub unsafe extern "system" fn CcUnpinDataForThread(bcb: PVOID, resource_thread_id: ERESOURCE_THREAD) {
    let bcb = opaque_ptr(bcb);
    let i_bcb = ((bcb as usize) - core::mem::offset_of!(_INTERNAL_BCB, PFCB)) as PINTERNAL_BCB;

    if (*i_bcb).PinCount != 0 {
        ExReleaseResourceForThreadLite(
            core::ptr::addr_of_mut!((*i_bcb).Lock),
            resource_thread_id,
        );
        (*i_bcb).PinCount -= 1;
    }

    CcpDereferenceBcb((*(*i_bcb).Vacb).SharedCacheMap, i_bcb);
}

#[no_mangle]
pub unsafe extern "system" fn CcRepinBcb(bcb: PVOID) {
    let bcb = opaque_ptr(bcb);
    let i_bcb = ((bcb as usize) - core::mem::offset_of!(_INTERNAL_BCB, PFCB)) as PINTERNAL_BCB;
    (*i_bcb).RefCount += 1;
}

#[no_mangle]
pub unsafe extern "system" fn CcUnpinRepinnedBcb(
    bcb: PVOID,
    write_through: BOOLEAN,
    io_status: PIO_STATUS_BLOCK,
) {
    let bcb = opaque_ptr(bcb);
    let io_status = opaque_ptr(io_status);
    let i_bcb = ((bcb as usize) - core::mem::offset_of!(_INTERNAL_BCB, PFCB)) as PINTERNAL_BCB;
    let mut old_irql: KIRQL = 0;
    let shared_cache_map = (*(*i_bcb).Vacb).SharedCacheMap;

    (*io_status).__bindgen_anon_1.Status = STATUS_SUCCESS;

    if write_through != 0 {
        CcFlushCache(
            (*(*shared_cache_map).FileObject).SectionObjectPointer,
            core::ptr::addr_of_mut!((*i_bcb).PFCB.MappedFileOffset),
            (*i_bcb).PFCB.MappedLength,
            io_status,
        );
    } else {
        (*io_status).__bindgen_anon_1.Status = STATUS_SUCCESS;
        (*io_status).Information = 0;
    }

    KeAcquireSpinLock(
        core::ptr::addr_of_mut!((*shared_cache_map).BcbSpinLock) as *mut _,
        &mut old_irql,
    );
    (*i_bcb).RefCount -= 1;
    if (*i_bcb).RefCount == 0 {
        RemoveEntryList(core::ptr::addr_of_mut!((*i_bcb).BcbEntry) as PLIST_ENTRY);
        KeReleaseSpinLock(
            core::ptr::addr_of_mut!((*shared_cache_map).BcbSpinLock) as *mut _,
            old_irql,
        );

        if (*i_bcb).PinCount != 0 {
            ExReleaseResourceLite(core::ptr::addr_of_mut!((*i_bcb).Lock));
            (*i_bcb).PinCount -= 1;
        }

        CcRosReleaseVacb(
            (*(*i_bcb).Vacb).SharedCacheMap,
            (*i_bcb).Vacb,
            0u8,
            0u8,
        );

        ExDeleteResourceLite(core::ptr::addr_of_mut!((*i_bcb).Lock));
        ExFreeToNPagedLookasideList(core::ptr::addr_of_mut!(iBcbLookasideList), i_bcb as PVOID);
    } else {
        KeReleaseSpinLock(
            core::ptr::addr_of_mut!((*shared_cache_map).BcbSpinLock) as *mut _,
            old_irql,
        );
    }
}
