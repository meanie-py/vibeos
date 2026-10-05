#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use ntkrnl_sys::ke::KeGetCurrentIrql;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::guids::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ddk::{KeDelayExecutionThread, KeQueryTimeIncrement, KeStallExecutionProcessor, LARGE_INTEGER, PCI_PMCSR, PCI_PM_CAPABILITY, PIO_STACK_LOCATION, PIRP};
use ntkrnl_sys::ddk::{
    _DEVICE_POWER_STATE_PowerDeviceD0 as PowerDeviceD0,
    _DEVICE_POWER_STATE_PowerDeviceD3 as PowerDeviceD3,
    _DEVICE_POWER_STATE_PowerDeviceUnspecified as PowerDeviceUnspecified,
};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{DEVICE_POWER_STATE, PCSTR, PSTR, RtlAssert, _PCI_COMMON_HEADER};
use ntkrnl_sys::driver_ddk::{DbgPrint};
use ntkrnl_sys::ddk::{PDEVICE_OBJECT, PULONG, VF_FAILURE_CLASS};

// PCI_VERIFIER_DETECTED_VIOLATION bugcheck code (mc/bugcodes.mc = 0xf6)
pub const PCI_VERIFIER_DETECTED_VIOLATION: ULONG = 0xf6;

extern "C" {
    // Driver Verifier interface (ndk/vffuncs.h; not in ntkrnl_sys bindgen).
    // variadic → must be cdecl (NTAPI variadics are cdecl on all arches)
    fn VfFailDeviceNode(
        PhysicalDeviceObject: PDEVICE_OBJECT,
        BugCheckMajorCode: ULONG,
        BugCheckMinorCode: ULONG,
        FailureClass: VF_FAILURE_CLASS,
        AssertionControl: PULONG,
        DebuggerMessageText: PSTR,
        ParameterFormatString: PSTR,
        ...
    );
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut PciPowerDelayTable: [ULONG; 16] = [
    0 as ::core::ffi::c_int as ULONG,
    0 as ::core::ffi::c_int as ULONG,
    200 as ::core::ffi::c_int as ULONG,
    10000 as ::core::ffi::c_int as ULONG,
    0 as ::core::ffi::c_int as ULONG,
    0 as ::core::ffi::c_int as ULONG,
    200 as ::core::ffi::c_int as ULONG,
    10000 as ::core::ffi::c_int as ULONG,
    200 as ::core::ffi::c_int as ULONG,
    200 as ::core::ffi::c_int as ULONG,
    0 as ::core::ffi::c_int as ULONG,
    10000 as ::core::ffi::c_int as ULONG,
    10000 as ::core::ffi::c_int as ULONG,
    10000 as ::core::ffi::c_int as ULONG,
    10000 as ::core::ffi::c_int as ULONG,
    0 as ::core::ffi::c_int as ULONG,
];
#[no_mangle]
pub unsafe extern "system" fn PciStallForPowerChange(
    PdoExtension: PPCI_PDO_EXTENSION,
    PowerState: DEVICE_POWER_STATE,
    CapOffset: ULONG_PTR,
) -> NTSTATUS {
    let mut PciState: ULONG;
    let mut TimeoutEntry: ULONG;
    let mut PmcsrOffset: ULONG;
    let mut TryCount: ULONG;
    let mut VerifierData: PPCI_VERIFIER_DATA;
    let mut Interval: LARGE_INTEGER = core::mem::zeroed();
    let mut Pmcsr: PCI_PMCSR;
    let Irql: KIRQL;

    // ASSERT((*PdoExtension).PowerState.CurrentDeviceState >= PowerDeviceD0 && ...)
    if !((*PdoExtension).PowerState.CurrentDeviceState as u32 >= PowerDeviceD0 as u32
        && (*PdoExtension).PowerState.CurrentDeviceState as u32 <= PowerDeviceD3 as u32)
    {
        RtlAssert(
            b"(PdoExtension->PowerState.CurrentDeviceState >= PowerDeviceD0) && (PdoExtension->PowerState.CurrentDeviceState <= PowerDeviceD3)\0"
                .as_ptr() as PVOID,
            b"drivers/bus/pcix/power.c\0".as_ptr()
                as PVOID,
            57 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }
    if !(PowerState as u32 >= PowerDeviceD0 as u32
        && PowerState as u32 <= PowerDeviceD3 as u32)
    {
        RtlAssert(
            b"(PowerState >= PowerDeviceD0) && (PowerState <= PowerDeviceD3)\0".as_ptr() as PVOID,
            b"drivers/bus/pcix/power.c\0".as_ptr()
                as PVOID,
            58 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }
    if (*PdoExtension).HackFlags & 0x20000000 != 0 {
        RtlAssert(
            b"!(PdoExtension->HackFlags & PCI_HACK_NO_PM_CAPS)\0".as_ptr() as PVOID,
            b"drivers/bus/pcix/power.c\0".as_ptr()
                as PVOID,
            59 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }

    Irql = KeGetCurrentIrql();
    TimeoutEntry = PciPowerDelayTable[
        (PowerState as u32)
            .wrapping_mul((*PdoExtension).PowerState.CurrentDeviceState as u32)
            as usize
    ];
    PciState = (PowerState as u32).wrapping_sub(1) as ULONG;
    // PmcsrOffset = CapOffset + FIELD_OFFSET(PCI_PM_CAPABILITY, PMCSR) = CapOffset + 4
    PmcsrOffset = (CapOffset as ULONG).wrapping_add(4);
    TryCount = 100u32;

    loop {
        TryCount = TryCount.wrapping_sub(1);
        if TryCount == 0 {
            break;
        }

        if TimeoutEntry > 0 {
            if Irql as i32 >= DISPATCH_LEVEL as i32 {
                KeStallExecutionProcessor(TimeoutEntry);
            } else {
                Interval.QuadPart =
                    (-(10i32) as ULONG).wrapping_mul(TimeoutEntry) as LONGLONG;
                Interval.QuadPart -=
                    (KeQueryTimeIncrement() as ULONG).wrapping_sub(1) as LONGLONG;
                KeDelayExecutionThread(
                    0i8,
                    0u8, // FALSE
                    &raw mut Interval,
                );
            }
        }

        Pmcsr = core::mem::zeroed::<PCI_PMCSR>();
        PciReadDeviceConfig(
            PdoExtension,
            &raw mut Pmcsr as PVOID,
            PmcsrOffset,
            core::mem::size_of::<PCI_PMCSR>() as ULONG,
        );
        if Pmcsr.PowerState() as ULONG == PciState {
            return STATUS_SUCCESS;
        }

        TimeoutEntry = 1000;
    }

    VerifierData = PciVerifierRetrieveFailureData(2);
    if VerifierData.is_null() {
        RtlAssert(
            b"VerifierData\0".as_ptr() as PVOID,
            b"drivers/bus/pcix/power.c\0".as_ptr()
                as PVOID,
            105 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }

    VfFailDeviceNode(
        (*PdoExtension).PhysicalDeviceObject,
        PCI_VERIFIER_DETECTED_VIOLATION,
        2,
        (*VerifierData).FailureClass,
        &raw mut (*VerifierData).AssertionControl,
        (*VerifierData).DebuggerMessageText as PSTR,
        b"%DevObj%Ulong\0".as_ptr() as PSTR,
        (*PdoExtension).PhysicalDeviceObject,
        PciState,
    );

    STATUS_DEVICE_PROTOCOL_ERROR
}
#[no_mangle]
pub unsafe extern "system" fn PciSetPowerManagedDevicePowerState(
    DeviceExtension: PPCI_PDO_EXTENSION,
    DeviceState: DEVICE_POWER_STATE,
    IrpSet: BOOLEAN,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;

    if PciCanDisableDecodes(DeviceExtension, core::ptr::null_mut::<_PCI_COMMON_HEADER>(), 0, 1u8) == 0
        && (DeviceState as u32) != PowerDeviceD0 as u32
    {
        if DbgPrint(
            b"(%s:%d) Cannot disable decodes on this device, ignoring PM request...\n\0".as_ptr() as PCSTR,
            b"drivers/bus/pcix/power.c\0".as_ptr() as *const i8,
            137i32,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                b"drivers/bus/pcix/power.c\0".as_ptr() as *const i8,
                137i32,
            );
        }
        return status;
    }

    if (*DeviceExtension).HackFlags & (PCI_HACK_NO_PM_CAPS as u64) == 0 {
        let mut pm_caps: PCI_PM_CAPABILITY = core::mem::zeroed();
        let caps_offset: u32 = PciReadDeviceCapability(
            DeviceExtension,
            (*DeviceExtension).CapabilitiesPtr as u8,
            PCI_CAPABILITY_ID_POWER_MANAGEMENT as u32,
            (&raw mut pm_caps.Header as PVOID).cast(),
            core::mem::size_of::<PCI_PM_CAPABILITY>() as u32,
        ) as u32;

        if caps_offset == 0 {
            RtlAssert(
                b"CapsOffset\0".as_ptr() as PVOID,
                b"drivers/bus/pcix/power.c\0".as_ptr() as PVOID,
                150u32,
                core::ptr::null_mut::<i8>(),
            );
        }
        if !(DeviceState as u32 != PowerDeviceUnspecified as u32) {
            RtlAssert(
                b"DeviceState != PowerDeviceUnspecified\0".as_ptr() as PVOID,
                b"drivers/bus/pcix/power.c\0".as_ptr() as PVOID,
                151u32,
                core::ptr::null_mut::<i8>(),
            );
        }

        if DeviceState as u32 == PowerDeviceD0 as u32 {
            pm_caps.PMCSR.ControlStatus.set_PowerState(0u16);
            if pm_caps.PMC.Capabilities.Support.PMED3Cold() != 0 {
                pm_caps.PMCSR.ControlStatus.set_PMEStatus(1u16);
            }
        } else {
            pm_caps.PMCSR.ControlStatus.set_PowerState((DeviceState as u32).wrapping_sub(1) as u16);
        }

        PciWriteDeviceConfig(
            DeviceExtension,
            &raw mut pm_caps.PMCSR as PVOID,
            caps_offset.wrapping_add(4u32),
            core::mem::size_of::<PCI_PMCSR>() as u32,
        );

        status = PciStallForPowerChange(DeviceExtension, DeviceState, caps_offset as ULONG_PTR);
        if !NT_SUCCESS(status) {
            return status;
        }
    } else {
        if DbgPrint(
            b"(%s:%d) No PM on this device, ignoring request\n\0".as_ptr() as PCSTR,
            b"drivers/bus/pcix/power.c\0".as_ptr() as *const i8,
            185i32,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                b"drivers/bus/pcix/power.c\0".as_ptr() as *const i8,
                185i32,
            );
        }
    }

    if IrpSet != 0 {
        if (DeviceState as u32) < (*DeviceExtension).PowerState.CurrentDeviceState as u32 {
            DbgPrint(
                b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
                b"PciSetPowerManagedDevicePowerState\0".as_ptr() as *const i8,
                b"drivers/bus/pcix/power.c\0".as_ptr() as *const i8,
                195i32,
            );
            DbgPrint(b"\0".as_ptr() as PCSTR);
            DbgBreakPoint();
            status = STATUS_NOT_IMPLEMENTED;
        }
    }

    status
}
#[no_mangle]
pub unsafe extern "system" fn PciFdoWaitWake(
    _Irp: PIRP,
    _IoStackLocation: PIO_STACK_LOCATION,
    _DeviceExtension: PPCI_FDO_EXTENSION,
) -> NTSTATUS {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const u8 as *const i8,
        b"PciFdoWaitWake\0".as_ptr() as *const u8 as *const i8,
        b"drivers/bus/pcix/power.c\0".as_ptr() as *const u8 as *const i8,
        214i32,
    );
    // power.c follows UNIMPLEMENTED with `while (TRUE);` and then the
    // `return STATUS_NOT_SUPPORTED;` it can never reach. Return it: an FDO power IRP
    // should fail, not spin the CPU that delivered it.
    STATUS_NOT_SUPPORTED
}
#[no_mangle]
pub unsafe extern "system" fn PciFdoSetPowerState(
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_FDO_EXTENSION,
) -> NTSTATUS {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as PCSTR,
        b"PciFdoSetPowerState\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/bus/pcix/power.c\0" as *const u8
            as *const ::core::ffi::c_char,
        229 as ::core::ffi::c_int,
    );
    // power.c follows UNIMPLEMENTED with `while (TRUE);` and then the
    // `return STATUS_NOT_SUPPORTED;` it can never reach. Return it: an FDO power IRP
    // should fail, not spin the CPU that delivered it.
    STATUS_NOT_SUPPORTED
}
#[no_mangle]
pub unsafe extern "system" fn PciFdoIrpQueryPower(
    Irp: PIRP,
    IoStackLocation: PIO_STACK_LOCATION,
    DeviceExtension: PPCI_FDO_EXTENSION,
) -> NTSTATUS {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"PciFdoIrpQueryPower\0".as_ptr() as *const i8,
        b"power.c\0".as_ptr() as *const i8,
        244i32,
    );
    // power.c follows UNIMPLEMENTED with `while (TRUE);` and then the
    // `return STATUS_NOT_SUPPORTED;` it can never reach. Return it: an FDO power IRP
    // should fail, not spin the CPU that delivered it.
    STATUS_NOT_SUPPORTED
}
