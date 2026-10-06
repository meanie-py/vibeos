// pci/ecam.rs — drivers/bus/pcix/pci/ecam.c (upstream e51b1c1dbfc, #9542): basic ECAM
// (PCIe memory-mapped configuration space) handling. Hand-ported, $0.
#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ddk::{
    MmMapIoSpace, MmUnmapIoSpace, PCI_SLOT_NUMBER, PHYSICAL_ADDRESS, PIO_RESOURCE_DESCRIPTOR,
    PIO_RESOURCE_REQUIREMENTS_LIST, PKEY_VALUE_PARTIAL_INFORMATION, PCSTR, RtlInitUnicodeString,
    UNICODE_STRING, ZwClose, ZwQueryValueKey,
};
use ntkrnl_sys::ddk::_KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation as KeyValuePartialInformation;
use ntkrnl_sys::driver_ddk::DbgPrint;
use core::ffi::c_void;

// ---- MMIO register access (xdk/iofuncs.h) ----
// i386 and arm64 import READ/WRITE_REGISTER_* from ntoskrnl (ntoskrnl.spec
// `-arch=i386,arm,arm64`); amd64 has only the FORCEINLINE volatile access, where a write
// is followed by an InterlockedOr on a local as a full barrier.
#[cfg(not(target_arch = "x86_64"))]
extern "system" {
    fn READ_REGISTER_UCHAR(Register: *mut UCHAR) -> UCHAR;
    fn READ_REGISTER_USHORT(Register: *mut USHORT) -> USHORT;
    fn READ_REGISTER_ULONG(Register: *mut ULONG) -> ULONG;
    fn WRITE_REGISTER_UCHAR(Register: *mut UCHAR, Value: UCHAR);
    fn WRITE_REGISTER_USHORT(Register: *mut USHORT, Value: USHORT);
    fn WRITE_REGISTER_ULONG(Register: *mut ULONG, Value: ULONG);
}
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn READ_REGISTER_UCHAR(Register: *mut UCHAR) -> UCHAR {
    core::ptr::read_volatile(Register)
}
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn READ_REGISTER_USHORT(Register: *mut USHORT) -> USHORT {
    core::ptr::read_volatile(Register)
}
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn READ_REGISTER_ULONG(Register: *mut ULONG) -> ULONG {
    core::ptr::read_volatile(Register)
}
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn WRITE_REGISTER_UCHAR(Register: *mut UCHAR, Value: UCHAR) {
    core::ptr::write_volatile(Register, Value);
    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
}
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn WRITE_REGISTER_USHORT(Register: *mut USHORT, Value: USHORT) {
    core::ptr::write_volatile(Register, Value);
    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
}
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn WRITE_REGISTER_ULONG(Register: *mut ULONG, Value: ULONG) {
    core::ptr::write_volatile(Register, Value);
    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
}

/* GLOBALS ********************************************************************/

const PCI_ECAM_BUS_COUNT: usize = (PCI_MAX_BRIDGE_NUMBER + 1) as usize;
const PCI_ECAM_BUS_SIZE: ULONG = PCI_MAX_DEVICES * PCI_MAX_FUNCTION * PCI_EXTENDED_CONFIG_LENGTH;

#[no_mangle]
pub static mut PciEcamVerified: BOOLEAN = 0;

static mut PciEcamWindowQueried: BOOLEAN = 0;
static mut PciEcamSkipK8Northbridge: BOOLEAN = 0;
static mut PciEcamWindowStart: ULONGLONG = 0;
static mut PciEcamWindowBuses: ULONG = 0;
static mut PciEcamBusView: [PUCHAR; PCI_ECAM_BUS_COUNT] = [core::ptr::null_mut(); PCI_ECAM_BUS_COUNT];

/* FUNCTIONS ******************************************************************/

/* Takes the first window, which is segment 0 since the HAL writes them in segment order */
unsafe fn PciEcamParseWindow(Value: PKEY_VALUE_PARTIAL_INFORMATION) -> BOOLEAN {
    let Requirements: PIO_RESOURCE_REQUIREMENTS_LIST;
    let Window: PIO_RESOURCE_DESCRIPTOR;
    let Start: ULONGLONG;
    let End: ULONGLONG;
    let Buses: ULONGLONG;
    PAGED_CODE();

    if (*Value).Type != REG_RESOURCE_REQUIREMENTS_LIST
        || ((*Value).DataLength as usize) < core::mem::size_of::<ntkrnl_sys::ddk::IO_RESOURCE_REQUIREMENTS_LIST>()
    {
        return 0;
    }

    Requirements = core::ptr::addr_of_mut!((*Value).Data) as PIO_RESOURCE_REQUIREMENTS_LIST;
    if (*Requirements).AlternativeLists == 0 || (*Requirements).List[0].Count == 0 {
        return 0;
    }

    Window = core::ptr::addr_of_mut!((*Requirements).List[0].Descriptors) as PIO_RESOURCE_DESCRIPTOR;
    if (*Window).Type as u32 != CmResourceTypeMemory
        && (*Window).Type as u32 != CmResourceTypeMemoryLarge
    {
        return 0;
    }

    Start = (*Window).u.Memory.MinimumAddress.QuadPart as ULONGLONG;
    End = (*Window).u.Memory.MaximumAddress.QuadPart as ULONGLONG;
    if End < Start {
        return 0;
    }

    Buses = (End - Start).wrapping_add(1) / PCI_ECAM_BUS_SIZE as ULONGLONG;
    if Buses == 0 || Buses > PCI_ECAM_BUS_COUNT as ULONGLONG {
        return 0;
    }

    PciEcamWindowStart = Start;
    PciEcamWindowBuses = Buses as ULONG;
    1
}

/* The HAL publishes the MCFG windows here so the arbiters keep them reserved */
unsafe fn PciEcamQueryWindow() -> NTSTATUS {
    let Value: PKEY_VALUE_PARTIAL_INFORMATION;
    let mut ValueName: UNICODE_STRING = core::mem::zeroed();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;
    let mut Size: ULONG = 0;
    PAGED_CODE();

    if PciOpenKey(
        win32macros_rust::wide_bytes!(b"\\\0R\0e\0g\0i\0s\0t\0r\0y\0\\\0M\0a\0c\0h\0i\0n\0e\0\\\0S\0y\0s\0t\0e\0m\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0C\0o\0n\0t\0r\0o\0l\0\\\0A\0r\0b\0i\0t\0e\0r\0s\0\\\0R\0e\0s\0e\0r\0v\0e\0d\0R\0e\0s\0o\0u\0r\0c\0e\0s\0\0\0") as PWCHAR,
        core::ptr::null_mut(),
        KEY_QUERY_VALUE as _,
        &mut KeyHandle as *mut HANDLE as _,
        &mut Status,
    ) == 0
    {
        return Status;
    }

    RtlInitUnicodeString(
        &mut ValueName,
        win32macros_rust::wide_bytes!(b"M\0m\0C\0o\0n\0f\0i\0g\0R\0a\0n\0g\0e\0\0\0"),
    );
    Status = ZwQueryValueKey(
        KeyHandle,
        &mut ValueName,
        KeyValuePartialInformation,
        core::ptr::null_mut(),
        0,
        &mut Size,
    );
    if Status != STATUS_BUFFER_TOO_SMALL && Status != STATUS_BUFFER_OVERFLOW {
        ZwClose(KeyHandle);
        return if Status >= 0 { STATUS_UNSUCCESSFUL } else { Status };
    }

    Value = ExAllocatePoolWithTag(PagedPool, Size as SIZE_T, PCI_POOL_TAG) as PKEY_VALUE_PARTIAL_INFORMATION;
    if Value.is_null() {
        ZwClose(KeyHandle);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = ZwQueryValueKey(
        KeyHandle,
        &mut ValueName,
        KeyValuePartialInformation,
        Value as PVOID,
        Size,
        &mut Size,
    );
    ZwClose(KeyHandle);

    if Status >= 0 && PciEcamParseWindow(Value) == 0 {
        Status = STATUS_UNSUCCESSFUL;
    }

    ExFreePoolWithTag(Value as PVOID, 0);
    Status
}

/*
 * K8 northbridge functions on bus 0 are not decoded by the chipset ECAM window.
 * This really should be a hardware quirk so I'm marking this as TODO:
 * However I want to be able to boot until we got a solid system for this!
 */
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
unsafe fn PciEcamIsAmdK8() -> BOOLEAN {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::__cpuid;
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::__cpuid;

    /* "AuthenticAMD" */
    let CpuInfo = __cpuid(0);
    if CpuInfo.ebx != 0x68747541 || CpuInfo.edx != 0x69746E65 || CpuInfo.ecx != 0x444D4163 {
        return 0;
    }

    /* Family 0Fh with no extended family */
    let CpuInfo = __cpuid(1);
    ((CpuInfo.eax & 0x0FF00F00) == 0x00000F00) as BOOLEAN
}

/* Mappings are kept for the life of the driver since config access can happen at any IRQL */
unsafe fn PciEcamMapBus(Bus: ULONG) -> PUCHAR {
    let mut Address: PHYSICAL_ADDRESS = core::mem::zeroed();
    PAGED_CODE();

    if Bus >= PciEcamWindowBuses {
        return core::ptr::null_mut();
    }
    if !PciEcamBusView[Bus as usize].is_null() {
        return PciEcamBusView[Bus as usize];
    }

    Address.QuadPart =
        PciEcamWindowStart.wrapping_add((Bus as ULONGLONG) * PCI_ECAM_BUS_SIZE as ULONGLONG) as LONGLONG;
    PciEcamBusView[Bus as usize] = MmMapIoSpace(Address, PCI_ECAM_BUS_SIZE as _, MmNonCached) as PUCHAR;
    if PciEcamBusView[Bus as usize].is_null() {
        DbgPrint(
            b"(%s:%d) PCI: Unable to map ECAM for bus 0x%lx\n\0".as_ptr() as PCSTR,
            b"drivers/bus/pcix/pci/ecam.c\0".as_ptr() as PCSTR,
            170,
            Bus,
        );
    }

    PciEcamBusView[Bus as usize]
}

unsafe fn PciEcamDiscardWindow() {
    let mut Bus: ULONG = 0;
    PAGED_CODE();

    while Bus < PciEcamWindowBuses {
        if !PciEcamBusView[Bus as usize].is_null() {
            MmUnmapIoSpace(PciEcamBusView[Bus as usize] as PVOID, PCI_ECAM_BUS_SIZE as _);
            PciEcamBusView[Bus as usize] = core::ptr::null_mut();
        }
        Bus += 1;
    }

    PciEcamWindowBuses = 0;
}

unsafe fn PciEcamFunctionBase(Bus: ULONG, Slot: PCI_SLOT_NUMBER) -> PUCHAR {
    let Index: ULONG;

    if Bus >= PciEcamWindowBuses || PciEcamBusView[Bus as usize].is_null() {
        return core::ptr::null_mut();
    }

    if PciEcamSkipK8Northbridge != 0 && Bus == 0 && Slot.u.bits.DeviceNumber() >= 0x18 {
        return core::ptr::null_mut();
    }

    Index = (Slot.u.bits.DeviceNumber() as ULONG * PCI_MAX_FUNCTION) + Slot.u.bits.FunctionNumber() as ULONG;
    PciEcamBusView[Bus as usize].add((Index * PCI_EXTENDED_CONFIG_LENGTH) as usize)
}

/* Picks the same access widths as the HAL does for the legacy mechanism */
unsafe fn PciEcamTransfer(Function: PUCHAR, mut Buffer: PUCHAR, mut Offset: ULONG, mut Length: ULONG, Read: BOOLEAN) {
    let mut Register: PUCHAR;
    let mut Width: ULONG;

    while Length != 0 {
        Register = Function.add(Offset as usize);

        if (Offset & 1) != 0 || (Length & 3) == 1 {
            Width = core::mem::size_of::<UCHAR>() as ULONG;
            if Read != 0 {
                *Buffer = READ_REGISTER_UCHAR(Register);
            } else {
                WRITE_REGISTER_UCHAR(Register, *Buffer);
            }
        } else if (Offset & 3) == 0 && (Length & 3) == 0 {
            Width = core::mem::size_of::<ULONG>() as ULONG;
            if Read != 0 {
                (Buffer as PULONG).write_unaligned(READ_REGISTER_ULONG(Register as PULONG));
            } else {
                WRITE_REGISTER_ULONG(Register as PULONG, (Buffer as PULONG).read_unaligned());
            }
        } else {
            Width = core::mem::size_of::<USHORT>() as ULONG;
            if Read != 0 {
                (Buffer as PUSHORT).write_unaligned(READ_REGISTER_USHORT(Register as PUSHORT));
            } else {
                WRITE_REGISTER_USHORT(Register as PUSHORT, (Buffer as PUSHORT).read_unaligned());
            }
        }

        Buffer = Buffer.add(Width as usize);
        Offset += Width;
        Length -= Width;
    }
}

/* The window is assumed to start at bus 0, so compare IDs through both mechanisms to prove it */
unsafe fn PciEcamVerifyBus(FdoExtension: PPCI_FDO_EXTENSION) -> NTSTATUS {
    let mut Slot: PCI_SLOT_NUMBER = core::mem::zeroed();
    let mut Function: PUCHAR;
    let mut Device: ULONG;
    let mut LegacyIds: ULONG;
    let mut EcamIds: ULONG;
    let mut Matched: ULONG;
    PAGED_CODE();

    Slot.u.AsULONG = 0;
    Matched = 0;

    Device = 0;
    while Device < PCI_MAX_DEVICES {
        Slot.u.bits.set_DeviceNumber(Device);

        Function = PciEcamFunctionBase((*FdoExtension).BaseBus as ULONG, Slot);
        if !Function.is_null() {
            LegacyIds = MAXULONG;
            PciReadSlotConfig(
                FdoExtension,
                Slot,
                &mut LegacyIds as *mut ULONG as PVOID,
                0,
                core::mem::size_of::<ULONG>() as ULONG,
            );
            if LegacyIds as USHORT as u32 != PCI_INVALID_VENDORID && LegacyIds as USHORT != 0 {
                EcamIds = READ_REGISTER_ULONG(Function as PULONG);
                if EcamIds != LegacyIds {
                    DbgPrint(
                        b"(%s:%d) PCI: ECAM mismatch at %02lx:%02lx.0, read %08lx expected %08lx\n\0".as_ptr() as PCSTR,
                        b"drivers/bus/pcix/pci/ecam.c\0".as_ptr() as PCSTR,
                        298,
                        (*FdoExtension).BaseBus as ULONG,
                        Device,
                        EcamIds,
                        LegacyIds,
                    );
                    return STATUS_DEVICE_CONFIGURATION_ERROR;
                }

                Matched += 1;
            }
        }
        Device += 1;
    }

    if Matched != 0 { STATUS_SUCCESS } else { STATUS_NO_SUCH_DEVICE }
}

/// Maps the bus an FDO decodes into the ECAM window and verifies the window the first
/// time a bus with devices on it is seen.
#[no_mangle]
pub unsafe extern "system" fn PciInitializeEcam(FdoExtension: PPCI_FDO_EXTENSION) {
    let mut Status: NTSTATUS;
    PAGED_CODE();

    if PciEcamWindowQueried == 0 {
        PciEcamWindowQueried = 1;

        Status = PciEcamQueryWindow();
        if Status < 0 {
            DbgPrint(
                b"(%s:%d) PCI: No ECAM window available (0x%lx)\n\0".as_ptr() as PCSTR,
                b"drivers/bus/pcix/pci/ecam.c\0".as_ptr() as PCSTR,
                333,
                Status,
            );
            return;
        }

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            PciEcamSkipK8Northbridge = PciEcamIsAmdK8();
        }
        DbgPrint(
            b"(%s:%d) PCI: ECAM window at 0x%I64x for 0x%lx buses\n\0".as_ptr() as PCSTR,
            b"drivers/bus/pcix/pci/ecam.c\0".as_ptr() as PCSTR,
            340,
            PciEcamWindowStart,
            PciEcamWindowBuses,
        );
    }

    if PciEcamMapBus((*FdoExtension).BaseBus as ULONG).is_null() || PciEcamVerified != 0 {
        return;
    }

    /* An empty bus proves nothing, so leave it to the next one */
    Status = PciEcamVerifyBus(FdoExtension);
    if Status == STATUS_DEVICE_CONFIGURATION_ERROR {
        PciEcamDiscardWindow();
    } else if Status >= 0 {
        PciEcamVerified = 1;
        DbgPrint(
            b"(%s:%d) PCI: ECAM verified on bus 0x%lx\n\0".as_ptr() as PCSTR,
            b"drivers/bus/pcix/pci/ecam.c\0".as_ptr() as PCSTR,
            358,
            (*FdoExtension).BaseBus as ULONG,
        );
    }
}

/// Performs a configuration space access through the ECAM window. Returns FALSE if the
/// window cannot serve the access, in which case the caller falls back to the legacy
/// mechanism.
#[no_mangle]
pub unsafe extern "system" fn PciEcamReadWriteConfig(
    Bus: ULONG,
    Slot: PCI_SLOT_NUMBER,
    Buffer: PVOID,
    Offset: ULONG,
    Length: ULONG,
    Read: BOOLEAN,
) -> BOOLEAN {
    let Function: PUCHAR;

    if PciEcamVerified == 0 {
        return 0;
    }

    if Offset >= PCI_EXTENDED_CONFIG_LENGTH || Length > (PCI_EXTENDED_CONFIG_LENGTH - Offset) {
        return 0;
    }

    Function = PciEcamFunctionBase(Bus, Slot);
    if Function.is_null() {
        return 0;
    }

    PciEcamTransfer(Function, Buffer as PUCHAR, Offset, Length, Read);
    1
}
