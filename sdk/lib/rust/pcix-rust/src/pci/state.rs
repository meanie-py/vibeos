#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::guids::*;
use ntkrnl_sys::base::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::pci::config::RtlAssert;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ddk::PCSTR;
use core::ffi::c_void;


// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut PciTransitionText: [PCHAR; 7] = [
    b"PciNotStarted\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"PciStarted\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"PciDeleted\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"PciStopped\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"PciSurpriseRemoved\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"PciSynchronizedOperation\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"PciMaxObjectState\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
];

pub static mut PnpStateCancelArray: [NTSTATUS; 6] = [
    STATUS_INVALID_DEVICE_REQUEST,
    STATUS_FAIL_CHECK,
    STATUS_INVALID_DEVICE_STATE,
    STATUS_INVALID_DEVICE_STATE,
    STATUS_FAIL_CHECK,
    STATUS_FAIL_CHECK,
];

pub static mut PnpStateTransitionArray: [NTSTATUS; 36] = [
    STATUS_SUCCESS,
    STATUS_SUCCESS,
    STATUS_FAIL_CHECK,
    STATUS_SUCCESS,
    STATUS_FAIL_CHECK,
    STATUS_FAIL_CHECK,
    STATUS_SUCCESS,
    STATUS_FAIL_CHECK,
    STATUS_FAIL_CHECK,
    STATUS_SUCCESS,
    STATUS_FAIL_CHECK,
    STATUS_FAIL_CHECK,
    STATUS_SUCCESS,
    STATUS_SUCCESS,
    STATUS_FAIL_CHECK,
    STATUS_FAIL_CHECK,
    STATUS_SUCCESS,
    STATUS_FAIL_CHECK,
    STATUS_INVALID_DEVICE_REQUEST,
    STATUS_SUCCESS,
    STATUS_FAIL_CHECK,
    STATUS_FAIL_CHECK,
    STATUS_FAIL_CHECK,
    STATUS_FAIL_CHECK,
    STATUS_SUCCESS,
    STATUS_SUCCESS,
    STATUS_FAIL_CHECK,
    STATUS_SUCCESS,
    STATUS_FAIL_CHECK,
    STATUS_FAIL_CHECK,
    STATUS_SUCCESS,
    STATUS_SUCCESS,
    STATUS_INVALID_DEVICE_STATE,
    STATUS_SUCCESS,
    STATUS_INVALID_DEVICE_STATE,
    STATUS_FAIL_CHECK,
];

pub const NDEBUG: bool = true;

extern "system" {
}
#[no_mangle]
pub unsafe extern "system" fn PciInitializeState(DeviceExtension: *mut _PCI_FDO_EXTENSION) {
    (*DeviceExtension).DeviceState = PciNotStarted as i32 as u8;
    (*DeviceExtension).TentativeNextState = PciNotStarted as i32 as u8;
}

#[no_mangle]
pub unsafe extern "system" fn PciBeginStateTransition(
    DeviceExtension: PPCI_FDO_EXTENSION,
    NewState: PCI_STATE,
) -> NTSTATUS {
    let mut CurrentState: PCI_STATE;
    let mut Status: NTSTATUS;

    // DPRINT1 macro expansion
    if DbgPrint(
        b"(%s:%d) PCI Request to begin transition of Extension %p to %s ->\0".as_ptr() as PCSTR,
        b"drivers/bus/pcix/pci/state.c\0".as_ptr()
            as *const ::core::ffi::c_char,
        104 as ::core::ffi::c_int,
        DeviceExtension,
        PciTransitionText[NewState as usize],
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"drivers/bus/pcix/pci/state.c\0".as_ptr()
                as *const ::core::ffi::c_char,
            104 as ::core::ffi::c_int,
        );
    }

    // ASSERT(DeviceExtension->TentativeNextState == DeviceExtension->DeviceState)
    if !((*DeviceExtension).TentativeNextState as i32 == (*DeviceExtension).DeviceState as i32) {
        RtlAssert(
            b"DeviceExtension->TentativeNextState == DeviceExtension->DeviceState\0".as_ptr()
                as *const ::core::ffi::c_char as PVOID,
            b"drivers/bus/pcix/pci/state.c\0".as_ptr()
                as *const ::core::ffi::c_char as PVOID,
            107 as ULONG,
            (core::ptr::null_mut::<CHAR>()).cast(),
        );
    }

    CurrentState = (*DeviceExtension).DeviceState as PCI_STATE;

    // ASSERT(CurrentState < PciMaxObjectState)
    if !((CurrentState as u32) < (PciMaxObjectState as u32)) {
        RtlAssert(
            b"CurrentState < PciMaxObjectState\0".as_ptr() as *const ::core::ffi::c_char as PVOID,
            b"drivers/bus/pcix/pci/state.c\0".as_ptr()
                as *const ::core::ffi::c_char as PVOID,
            111 as ULONG,
            (core::ptr::null_mut::<CHAR>()).cast(),
        );
    }

    // ASSERT(NewState < PciMaxObjectState)
    if !((NewState as u32) < (PciMaxObjectState as u32)) {
        RtlAssert(
            b"NewState < PciMaxObjectState\0".as_ptr() as *const ::core::ffi::c_char as PVOID,
            b"drivers/bus/pcix/pci/state.c\0".as_ptr()
                as *const ::core::ffi::c_char as PVOID,
            112 as ULONG,
            (core::ptr::null_mut::<CHAR>()).cast(),
        );
    }

    // Lookup if this state transition is valid
    Status = PnpStateTransitionArray
        [(CurrentState as u32).wrapping_add((6u32).wrapping_mul(NewState as u32)) as usize];

    if Status == STATUS_FAIL_CHECK {
        // Invalid transition (logical fault)
        if DbgPrint(
            b"(%s:%d) ERROR\nPCI: Error trying to enter state \"%s\" from state \"%s\"\n\0"
                .as_ptr() as PCSTR,
            b"drivers/bus/pcix/pci/state.c\0".as_ptr()
                as *const ::core::ffi::c_char,
            122 as ::core::ffi::c_int,
            PciTransitionText[NewState as usize],
            PciTransitionText[CurrentState as usize],
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                b"drivers/bus/pcix/pci/state.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                122 as ::core::ffi::c_int,
            );
        }
        DbgBreakPoint();
    } else if Status == STATUS_INVALID_DEVICE_REQUEST {
        // Invalid transition (illegal request)
        if DbgPrint(
            b"(%s:%d) ERROR\nPCI: Illegal request to try to enter state \"%s\" from state \"%s\", rejecting\0"
                .as_ptr() as PCSTR,
            b"drivers/bus/pcix/pci/state.c\0".as_ptr()
                as *const ::core::ffi::c_char,
            131 as ::core::ffi::c_int,
            PciTransitionText[NewState as usize],
            PciTransitionText[CurrentState as usize],
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                b"drivers/bus/pcix/pci/state.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                131 as ::core::ffi::c_int,
            );
        }
    }

    // ASSERT((NewState != DeviceExtension->DeviceState) || (!NT_SUCCESS(Status)))
    if !((NewState as u32 != (*DeviceExtension).DeviceState as u32)
        || !NT_SUCCESS(Status))
    {
        RtlAssert(
            b"(NewState != DeviceExtension->DeviceState) || (!NT_SUCCESS(Status))\0".as_ptr()
                as *const ::core::ffi::c_char as PVOID,
            b"drivers/bus/pcix/pci/state.c\0".as_ptr()
                as *const ::core::ffi::c_char as PVOID,
            135 as ULONG,
            (core::ptr::null_mut::<CHAR>()).cast(),
        );
    }

    // Enter the new state if successful
    if NT_SUCCESS(Status) {
        (*DeviceExtension).TentativeNextState = NewState as BOOLEAN;
    }

    DbgPrint(b"%x\n\0".as_ptr() as PCSTR, Status);

    Status
}
#[no_mangle]
pub unsafe extern "system" fn PciCancelStateTransition(
    DeviceExtension: PPCI_FDO_EXTENSION,
    StateNotEntered: PCI_STATE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    if DbgPrint(
        b"(%s:%d) PCI Request to cancel transition of Extension %p to %s ->\0" as *const u8
            as *const i8,
        b"drivers/bus/pcix/pci/state.c\0" as *const u8
            as *const i8,
        151i32,
        DeviceExtension,
        PciTransitionText[StateNotEntered as usize],
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
            b"drivers/bus/pcix/pci/state.c\0" as *const u8
                as *const i8,
            151i32,
        );
    }
    if (*DeviceExtension).TentativeNextState as i32 == (*DeviceExtension).DeviceState as i32 {
        if !((StateNotEntered as u32) < PciMaxObjectState as u32) {
            RtlAssert(
                b"StateNotEntered < PciMaxObjectState\0" as *const u8 as *const i8 as *mut core::ffi::c_void,
                b"drivers/bus/pcix/pci/state.c\0"
                    as *const u8 as *const i8 as *mut core::ffi::c_void,
                157u32,
                (core::ptr::null_mut::<i8>()).cast(),
            );
        }
        if !(PnpStateCancelArray[StateNotEntered as usize] != 0xc0000229u32 as NTSTATUS) {
            RtlAssert(
                b"PnpStateCancelArray[StateNotEntered] != STATUS_FAIL_CHECK\0" as *const u8
                    as *const i8 as *mut core::ffi::c_void,
                b"drivers/bus/pcix/pci/state.c\0"
                    as *const u8 as *const i8 as *mut core::ffi::c_void,
                158u32,
                (core::ptr::null_mut::<i8>()).cast(),
            );
        }
        Status = STATUS_INVALID_DEVICE_STATE;
        DbgPrint(b"%x\n\0" as *const u8 as *const i8, Status);
    } else {
        if !((*DeviceExtension).TentativeNextState as u32 == StateNotEntered as u32) {
            RtlAssert(
                b"DeviceExtension->TentativeNextState == StateNotEntered\0" as *const u8
                    as *const i8 as *mut core::ffi::c_void,
                b"drivers/bus/pcix/pci/state.c\0"
                    as *const u8 as *const i8 as *mut core::ffi::c_void,
                167u32,
                (core::ptr::null_mut::<i8>()).cast(),
            );
        }
        (*DeviceExtension).TentativeNextState = (*DeviceExtension).DeviceState;
        Status = STATUS_SUCCESS;
        DbgPrint(b"%x\n\0" as *const u8 as *const i8, Status);
    }
    Status
}
#[no_mangle]
pub unsafe extern "system" fn PciCommitStateTransition(
    DeviceExtension: PPCI_FDO_EXTENSION,
    NewState: PCI_STATE,
) {
    if DbgPrint(
        b"(%s:%d) PCI Commit transition of Extension %p to %s\n\0" as *const u8 as PCSTR,
        b"drivers/bus/pcix/pci/state.c\0" as *const u8
            as *const core::ffi::c_char,
        185 as i32,
        DeviceExtension as *mut core::ffi::c_void,
        PciTransitionText[NewState as usize],
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"drivers/bus/pcix/pci/state.c\0" as *const u8
                as *const core::ffi::c_char,
            185 as i32,
        );
    }
    if !(NewState as u32 != PciSynchronizedOperation as i32 as u32) {
        RtlAssert(
            b"NewState != PciSynchronizedOperation\0" as *const u8 as *const core::ffi::c_char
                as PVOID,
            b"drivers/bus/pcix/pci/state.c\0" as *const u8
                as *const core::ffi::c_char as PVOID,
            188 as ULONG,
            (core::ptr::null_mut::<core::ffi::c_char>()).cast(),
        );
    }
    if !((*DeviceExtension).TentativeNextState as u32 == NewState as u32) {
        RtlAssert(
            b"DeviceExtension->TentativeNextState == NewState\0" as *const u8
                as *const core::ffi::c_char as PVOID,
            b"drivers/bus/pcix/pci/state.c\0" as *const u8
                as *const core::ffi::c_char as PVOID,
            189 as ULONG,
            (core::ptr::null_mut::<core::ffi::c_char>()).cast(),
        );
    }
    (*DeviceExtension).DeviceState = NewState as u8;
}
