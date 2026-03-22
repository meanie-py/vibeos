use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    DRIVER_VERIFIER_THUNK_PAIRS, PDRIVER_VERIFIER_THUNK_PAIRS,
    PLDR_DATA_TABLE_ENTRY, _LDR_DATA_TABLE_ENTRY,
    PDRIVER_OBJECT, MM_DRIVER_VERIFIER_DATA, PRKMUTANT,
    KeEnterCriticalRegion, KeLeaveCriticalRegion,
    KeReleaseMutant, KeWaitForSingleObject,
    _LIST_ENTRY,
};

#[no_mangle]
pub static mut MmVerifierData: MM_DRIVER_VERIFIER_DATA = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut MiVerifierDriverAddedThunkListHead: _LIST_ENTRY = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut MiActiveVerifierThunks: ULONG = 0;

#[no_mangle]
pub static mut MmTriageActionTaken: PVOID = core::ptr::null_mut();

#[no_mangle]
pub static mut KernelVerifier: PVOID = core::ptr::null_mut();

#[no_mangle]
pub static mut MmVerifyDriverBuffer: [WCHAR; 512] = [0; 512];

#[no_mangle]
pub static mut MmVerifyDriverBufferLength: ULONG = core::mem::size_of::<[WCHAR; 512]>() as ULONG;

#[no_mangle]
pub static mut MmVerifyDriverBufferType: ULONG = 0; // REG_NONE

#[no_mangle]
pub static mut MmVerifyDriverLevel: ULONG = 0xFFFFFFFF; // -1

const MUTANT_INCREMENT: i32 = 1;
const WrVirtualMemory: u32 = 18; // _KWAIT_REASON_WrVirtualMemory
const LDRP_IMAGE_VERIFYING: ULONG = 0x00004000;

#[repr(C)]
struct DRIVER_SPECIFIED_VERIFIER_THUNKS {
    ListEntry: _LIST_ENTRY,
    DataTableEntry: PLDR_DATA_TABLE_ENTRY,
    NumberOfThunks: ULONG,
}

#[no_mangle]
pub unsafe extern "system" fn MmUnlockPageableImageSection(_ImageSectionHandle: PVOID) {
    static mut WARN: ULONG = 0;
    if WARN == 0 {
        WARN = 1;
    }
}

#[no_mangle]
pub unsafe extern "system" fn MmLockPageableSectionByHandle(_ImageSectionHandle: PVOID) {
}

#[no_mangle]
pub unsafe extern "system" fn MmLockPageableDataSection(AddressWithinSection: PVOID) -> PVOID {
    static mut WARN: ULONG = 0;
    if WARN == 0 {
        WARN = 1;
    }
    AddressWithinSection
}

#[no_mangle]
pub unsafe extern "system" fn MmTrimAllSystemPageableMemory(_PurgeTransitionList: ULONG) -> ULONG {
    0
}

#[no_mangle]
pub unsafe extern "system" fn MmAddVerifierThunks(
    ThunkBuffer: PVOID,
    ThunkBufferSize: ULONG,
) -> NTSTATUS {
    let mut Status = STATUS_SUCCESS;

    if MiVerifierDriverAddedThunkListHead.Flink.is_null() {
        return STATUS_NOT_SUPPORTED;
    }

    let ThunkCount = ThunkBufferSize / core::mem::size_of::<DRIVER_VERIFIER_THUNK_PAIRS>() as ULONG;
    if ThunkCount == 0 {
        return STATUS_INVALID_PARAMETER_1;
    }

    let DriverThunks = ExAllocatePoolWithTag(
        PagedPool,
        (core::mem::size_of::<DRIVER_SPECIFIED_VERIFIER_THUNKS>() +
         ThunkCount as usize * core::mem::size_of::<DRIVER_VERIFIER_THUNK_PAIRS>()) as SIZE_T,
        u32::from_le_bytes(*b"tVmM"),
    ) as *mut DRIVER_SPECIFIED_VERIFIER_THUNKS;

    if DriverThunks.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let ThunkTable = (DriverThunks as *mut u8).add(core::mem::size_of::<DRIVER_SPECIFIED_VERIFIER_THUNKS>())
        as PDRIVER_VERIFIER_THUNK_PAIRS;

    core::ptr::copy_nonoverlapping(
        ThunkBuffer as *const u8,
        ThunkTable as *mut u8,
        ThunkCount as usize * core::mem::size_of::<DRIVER_VERIFIER_THUNK_PAIRS>(),
    );

    KeEnterCriticalRegion();
    KeWaitForSingleObject(
        core::ptr::addr_of_mut!(MmSystemLoadLock) as PVOID,
        WrVirtualMemory,
        0i8, // KernelMode
        0u8, // FALSE
        core::ptr::null_mut(),
    );

    let LdrEntry = MiLookupDataTableEntry(
        (*ThunkTable).PristineRoutine.map(|f| f as *mut core::ffi::c_void).unwrap_or(core::ptr::null_mut())
    ) as PLDR_DATA_TABLE_ENTRY;

    if LdrEntry.is_null() {
        Status = STATUS_INVALID_PARAMETER_2;
    } else {
        let ModuleBase = (*LdrEntry).DllBase;
        let ModuleEnd = ((*LdrEntry).DllBase as usize + (*LdrEntry).SizeOfImage as usize) as PVOID;

        if (ModuleBase as usize) < (0x80000000usize.wrapping_add(MmBootImageSize)) {
            Status = STATUS_INVALID_PARAMETER_2;
        } else {
            let mut valid = true;
            for i in 0..ThunkCount {
                let thunk = ThunkTable.add(i as usize);
                let pristine_addr = (*thunk).PristineRoutine.map(|f| f as usize).unwrap_or(0);
                if pristine_addr < (ModuleBase as usize) ||
                   pristine_addr >= (ModuleEnd as usize) {
                    valid = false;
                    break;
                }
            }

            if !valid {
                Status = STATUS_INVALID_PARAMETER_2;
            } else {
                (*DriverThunks).DataTableEntry = LdrEntry;
                (*DriverThunks).NumberOfThunks = ThunkCount;
                MiActiveVerifierThunks += 1;
                InsertTailList(
                    core::ptr::addr_of_mut!(MiVerifierDriverAddedThunkListHead) as PLIST_ENTRY,
                    core::ptr::addr_of_mut!((*DriverThunks).ListEntry) as PLIST_ENTRY,
                );
            }
        }
    }

    KeReleaseMutant(
        core::ptr::addr_of_mut!(MmSystemLoadLock) as PRKMUTANT,
        MUTANT_INCREMENT,
        0u8, // FALSE
        0u8, // FALSE
    );
    KeLeaveCriticalRegion();

    if Status != STATUS_SUCCESS {
        ExFreePoolWithTag(DriverThunks as PVOID, u32::from_le_bytes(*b"tVmM"));
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn MmIsDriverVerifying(DriverObject: PDRIVER_OBJECT) -> ULONG {
    let LdrEntry = (*DriverObject).DriverSection as PLDR_DATA_TABLE_ENTRY;
    if LdrEntry.is_null() {
        return 0;
    }

    if ((*LdrEntry).Flags & LDRP_IMAGE_VERIFYING) != 0 {
        1
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "system" fn MmIsVerifierEnabled(VerifierFlags: *mut ULONG) -> NTSTATUS {
    if !MiVerifierDriverAddedThunkListHead.Flink.is_null() {
        core::ptr::write_volatile(VerifierFlags, MmVerifierData.Level);
        STATUS_SUCCESS
    } else {
        core::ptr::write_volatile(VerifierFlags, 0);
        STATUS_NOT_SUPPORTED
    }
}
