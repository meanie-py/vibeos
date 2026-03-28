use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ddk::{
    PPHYSICAL_ADDRESS,
    PLARGE_INTEGER,
    PPHYSICAL_MEMORY_RANGE,
    PHYSICAL_MEMORY_RANGE,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

#[no_mangle]
pub unsafe extern "system" fn MmAddPhysicalMemory(
    _start_address: PPHYSICAL_ADDRESS,
    _number_of_bytes: PLARGE_INTEGER,
) -> NTSTATUS {
    let _number_of_bytes = opaque_ptr(_number_of_bytes);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn MmMarkPhysicalMemoryAsBad(
    _start_address: PPHYSICAL_ADDRESS,
    _number_of_bytes: PLARGE_INTEGER,
) -> NTSTATUS {
    let _number_of_bytes = opaque_ptr(_number_of_bytes);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn MmMarkPhysicalMemoryAsGood(
    _start_address: PPHYSICAL_ADDRESS,
    _number_of_bytes: PLARGE_INTEGER,
) -> NTSTATUS {
    let _number_of_bytes = opaque_ptr(_number_of_bytes);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn MmRemovePhysicalMemory(
    _start_address: PPHYSICAL_ADDRESS,
    _number_of_bytes: PLARGE_INTEGER,
) -> NTSTATUS {
    let _number_of_bytes = opaque_ptr(_number_of_bytes);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn MmGetPhysicalMemoryRanges() -> PPHYSICAL_MEMORY_RANGE {
    let size: ULONG;
    let mut entry: PPHYSICAL_MEMORY_RANGE;
    let buffer: PPHYSICAL_MEMORY_RANGE;
    let old_irql: KIRQL;

    size = (core::mem::size_of::<PHYSICAL_MEMORY_RANGE>() * ((*MmPhysicalMemoryBlock).NumberOfRuns as usize + 1)) as ULONG;

    buffer = ExAllocatePoolWithTag(NonPagedPool, size as usize, 0x68506d4d) as PPHYSICAL_MEMORY_RANGE;
    if buffer.is_null() {
        return core::ptr::null_mut();
    }
    entry = buffer;

    old_irql = MiAcquirePfnLock();

    for i in 0..(*MmPhysicalMemoryBlock).NumberOfRuns {
        (*entry).BaseAddress.QuadPart = (((*(*MmPhysicalMemoryBlock).Run.as_ptr().add(i as usize)).BasePage as u64) << PAGE_SHIFT) as i64;
        (*entry).NumberOfBytes.QuadPart = (((*(*MmPhysicalMemoryBlock).Run.as_ptr().add(i as usize)).PageCount as u64) << PAGE_SHIFT) as i64;
        entry = entry.add(1);
    }

    (*entry).BaseAddress.QuadPart = 0;
    (*entry).NumberOfBytes.QuadPart = 0;

    MiReleasePfnLock(old_irql);
    buffer
}
