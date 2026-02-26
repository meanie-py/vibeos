use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;

use ntkrnl_sys::ddk::{
    _EPROCESS, _ETHREAD, PMMSUPPORT,
    PsGetCurrentThread,
    MiLockWorkingSet, MiUnlockWorkingSet,
};

#[no_mangle]
pub static mut MmMinimumWorkingSetSize: SIZE_T = 0;
#[no_mangle]
pub static mut MmMaximumWorkingSetSize: SIZE_T = 0;
#[no_mangle]
pub static mut MmPagesAboveWsMinimum: SIZE_T = 0;

#[no_mangle]
pub unsafe extern "system" fn MmMapUserAddressesToPage(
    _base_address: PVOID,
    _number_of_bytes: SIZE_T,
    _page_address: PVOID,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn MmAdjustWorkingSetSize(
    working_set_minimum_in_bytes: SIZE_T,
    working_set_maximum_in_bytes: SIZE_T,
    _system_cache: ULONG,
    increase_okay: BOOLEAN,
) -> NTSTATUS {
    let mut minimum_working_set_size: SIZE_T;
    let mut maximum_working_set_size: SIZE_T;
    let mut delta: isize;
    let ws: PMMSUPPORT;
    let mut status: NTSTATUS;

    if (working_set_minimum_in_bytes == usize::MAX) &&
       (working_set_maximum_in_bytes == usize::MAX)
    {
        return STATUS_NOT_IMPLEMENTED;
    }

    status = STATUS_SUCCESS;

    let process = PsGetCurrentProcess();
    ws = core::ptr::addr_of_mut!((*(process as *mut _EPROCESS)).Vm);
    MiLockWorkingSet(PsGetCurrentThread() as *mut _ETHREAD, ws);

    minimum_working_set_size = if working_set_minimum_in_bytes != 0 {
        working_set_minimum_in_bytes / PAGE_SIZE
    } else {
        (*ws).MinimumWorkingSetSize as SIZE_T
    };
    maximum_working_set_size = if working_set_maximum_in_bytes != 0 {
        working_set_maximum_in_bytes / PAGE_SIZE
    } else {
        (*ws).MaximumWorkingSetSize as SIZE_T
    };

    if maximum_working_set_size > MmMaximumWorkingSetSize {
        maximum_working_set_size = MmMaximumWorkingSetSize;
        status = STATUS_WORKING_SET_LIMIT_RANGE;
    }

    if minimum_working_set_size < MmMinimumWorkingSetSize {
        minimum_working_set_size = MmMinimumWorkingSetSize;
        status = STATUS_WORKING_SET_LIMIT_RANGE;
    }

    if minimum_working_set_size > maximum_working_set_size {
        status = STATUS_BAD_WORKING_SET_LIMIT;
        MiUnlockWorkingSet(PsGetCurrentThread() as *mut _ETHREAD, ws);
        return status;
    }

    delta = minimum_working_set_size as isize - (*ws).MinimumWorkingSetSize as isize;
    if delta > 0 {
        if increase_okay == 0 {
            status = STATUS_PRIVILEGE_NOT_HELD;
            MiUnlockWorkingSet(PsGetCurrentThread() as *mut _ETHREAD, ws);
            return status;
        }

        if ((delta as SIZE_T) / 1024) > (MmAvailablePages - 128) {
            status = STATUS_INSUFFICIENT_RESOURCES;
            MiUnlockWorkingSet(PsGetCurrentThread() as *mut _ETHREAD, ws);
            return status;
        }

        if (delta as SIZE_T) > ((MmResidentAvailablePages - MmSystemLockPagesCount as isize - 256) as SIZE_T) {
            status = STATUS_INSUFFICIENT_RESOURCES;
            MiUnlockWorkingSet(PsGetCurrentThread() as *mut _ETHREAD, ws);
            return status;
        }
    }

    if delta != 0 {
        InterlockedExchangeAddSizeT(
            core::ptr::addr_of_mut!(MmResidentAvailablePages) as *mut isize,
            -delta,
        );
    }

    let ws_size = (*ws).WorkingSetSize as isize;
    delta += core::cmp::max(ws_size - minimum_working_set_size as isize, 0);

    delta -= core::cmp::max(ws_size - (*ws).MinimumWorkingSetSize as isize, 0);

    if delta != 0 {
        InterlockedExchangeAddSizeT(
            core::ptr::addr_of_mut!(MmPagesAboveWsMinimum) as *mut isize,
            delta,
        );
    }

    (*ws).MinimumWorkingSetSize = minimum_working_set_size as ULONG;
    (*ws).MaximumWorkingSetSize = maximum_working_set_size as ULONG;

    MiUnlockWorkingSet(PsGetCurrentThread() as *mut _ETHREAD, ws);
    status
}

#[no_mangle]
pub unsafe extern "system" fn MmSetAddressRangeModified(
    _address: PVOID,
    _length: SIZE_T,
) -> BOOLEAN {
    0u8
}

#[no_mangle]
pub unsafe extern "system" fn MmIsAddressValid(virtual_address: PVOID) -> BOOLEAN {
    #[cfg(target_pointer_width = "64")]
    {
        if !(*MiAddressToPxe(virtual_address)).valid() {
            return 0u8;
        }
        if !(*MiAddressToPpe(virtual_address)).valid() {
            return 0u8;
        }
    }

    if !(*MiAddressToPde(virtual_address)).valid() {
        return 0u8;
    }

    if !(*MiAddressToPte(virtual_address)).valid() {
        return 0u8;
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn MmIsNonPagedSystemAddressValid(virtual_address: PVOID) -> BOOLEAN {
    MmIsAddressValid(virtual_address)
}

#[no_mangle]
pub unsafe extern "system" fn MmSetBankedSection(
    _process_handle: HANDLE,
    _virtual_address: PVOID,
    _bank_length: ULONG,
    _read_write_bank: BOOLEAN,
    _bank_routine: PVOID,
    _context: PVOID,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn MmIsRecursiveIoFault() -> BOOLEAN {
    let thread = PsGetCurrentThread() as *mut _ETHREAD;

    ((*thread).DisablePageFaultClustering | (*thread).ForwardClusterOnly) as BOOLEAN
}

#[no_mangle]
pub unsafe extern "system" fn MmIsThisAnNtAsSystem() -> BOOLEAN {
    (MmProductType & 0xFF) as BOOLEAN
}

#[no_mangle]
pub unsafe extern "system" fn MmQuerySystemSize() -> MM_SYSTEMSIZE {
    MmSystemSize
}

#[no_mangle]
pub unsafe extern "system" fn MmCreateMirror() -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}
