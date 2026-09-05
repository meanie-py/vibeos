//! pci.c — ReactOS PCI bus driver entry + cross-file ID/description string
//! builders, translated to Rust (pci-rust cohab crate).
//!
//! Faithful transliteration of `drivers/bus/pci/pci.c`. One C file → one module.
//! No SEH in this driver. `DPRINT` is an NDEBUG no-op; `DPRINT1` is live (see
//! `crate::dprint`).
//!
//! Cross-file symbol contract (pci.h): DriverEntry + the Pci*String builders +
//! PciDuplicateUnicodeString are `#[no_mangle] extern "system"` (NTAPI). The
//! dispatch routers (PciDispatch*, Pci{Pnp,Power}Control, PciAddDevice,
//! PciUnload) are likewise `extern "system"` so their fn-pointers carry the
//! right per-arch ABI when transmuted into the DRIVER_OBJECT dispatch slots.
//!
//! Fdo{Pnp,Power}Control / Pdo{Pnp,Power}Control live in fdo.rs / pdo.rs and are
//! declared in an `extern "system"` block below (matching the pci.h prototypes).

use core::mem::transmute;
use core::ptr;

use ntkrnl_sys::ddk::*;
// NT_SUCCESS + most STATUS_* live in ntkrnl_sys::base (re-exported at crate root),
// NOT in the ddk bindgen glob. STATUS_PENDING / STATUS_INVALID_PARAMETER live in
// ntkrnl_sys::constants. Bring them all in explicitly.
use ntkrnl_sys::{
    NT_SUCCESS, STATUS_INSUFFICIENT_RESOURCES, STATUS_NOT_IMPLEMENTED, STATUS_NO_MEMORY,
    STATUS_SUCCESS,
};
use ntkrnl_sys::constants::{STATUS_INVALID_PARAMETER, STATUS_PENDING};
// These DDK "functions" are actually inline/macro wrappers not present in the
// ddk bindgen glob:
//   IoCompleteRequest — driver_ddk (wraps the IofCompleteRequest fastcall thunk)
//   IoGetCurrentIrpStackLocation — inline::tier1 (Irp->Tail.Overlay walk)
//   KeInitializeSpinLock — inline::ke (zeroes the lock)
use ntkrnl_sys::driver_ddk::IoCompleteRequest;
use ntkrnl_sys::inline::ke::KeInitializeSpinLock;
use ntkrnl_sys::inline::tier1::IoGetCurrentIrpStackLocation;

use crate::shared::{
    dsStopped, PPCI_DEVICE, PCOMMON_DEVICE_EXTENSION, PFDO_DEVICE_EXTENSION,
    FDO_DEVICE_EXTENSION, PCI_DRIVER_EXTENSION, TAG_PCI,
};

// ---------------------------------------------------------------------------
// Constants not emitted by the kernel bindgen (#define-only in the C headers).
// Values lifted verbatim from sdk/include/xdk/iotypes.h, rtltypes.h, rtlfuncs.h,
// ntbasedef.h and wine/winioctl.h.
// ---------------------------------------------------------------------------

// IRP major function codes (xdk/iofuncs.h) — index into MajorFunction[28].
const IRP_MJ_DEVICE_CONTROL: usize = 0x0e;
const IRP_MJ_POWER: usize = 0x16;
const IRP_MJ_PNP: usize = 0x1b;

// IoCreateDevice device type / characteristics (xdk/iotypes.h, wine/winioctl.h).
const FILE_DEVICE_BUS_EXTENDER: ULONG = 0x0000_002a;
const FILE_DEVICE_SECURE_OPEN: ULONG = 0x0000_0100;

// DEVICE_OBJECT.Flags (xdk/iotypes.h).
const DO_DEVICE_INITIALIZING: ULONG = 0x0000_0080;

// IoCompleteRequest priority boost (xdk/iotypes.h). CCHAR (= c_char) per the
// IoCompleteRequest wrapper signature.
const IO_NO_INCREMENT: CCHAR = 0;

// RtlQueryRegistryValues (xdk/rtltypes.h).
const RTL_REGISTRY_SERVICES: ULONG = 1;
const RTL_QUERY_REGISTRY_REQUIRED: ULONG = 0x0000_0004;
const RTL_QUERY_REGISTRY_DIRECT: ULONG = 0x0000_0020;

// RtlDuplicateUnicodeString flags (xdk/rtlfuncs.h).
const RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE: ULONG = 1;
const RTL_DUPLICATE_UNICODE_STRING_ALLOCATE_NULL_STRING: ULONG = 2;

// UNICODE_NULL (xdk/ntbasedef.h).
const UNICODE_NULL: u16 = 0;

// PCI base-class codes (xdk/iotypes.h).
const PCI_CLASS_PRE_20: UCHAR = 0x00;
const PCI_CLASS_MASS_STORAGE_CTLR: UCHAR = 0x01;
const PCI_CLASS_NETWORK_CTLR: UCHAR = 0x02;
const PCI_CLASS_DISPLAY_CTLR: UCHAR = 0x03;
const PCI_CLASS_MULTIMEDIA_DEV: UCHAR = 0x04;
const PCI_CLASS_MEMORY_CTLR: UCHAR = 0x05;
const PCI_CLASS_BRIDGE_DEV: UCHAR = 0x06;
const PCI_CLASS_SIMPLE_COMMS_CTLR: UCHAR = 0x07;
const PCI_CLASS_BASE_SYSTEM_DEV: UCHAR = 0x08;
const PCI_CLASS_INPUT_DEV: UCHAR = 0x09;
const PCI_CLASS_DOCKING_STATION: UCHAR = 0x0a;
const PCI_CLASS_PROCESSOR: UCHAR = 0x0b;
const PCI_CLASS_SERIAL_BUS_CTLR: UCHAR = 0x0c;

// PCI sub-class codes (xdk/iotypes.h).
const PCI_SUBCLASS_PRE_20_NON_VGA: UCHAR = 0x00;
const PCI_SUBCLASS_PRE_20_VGA: UCHAR = 0x01;

const PCI_SUBCLASS_MSC_SCSI_BUS_CTLR: UCHAR = 0x00;
const PCI_SUBCLASS_MSC_IDE_CTLR: UCHAR = 0x01;
const PCI_SUBCLASS_MSC_FLOPPY_CTLR: UCHAR = 0x02;
const PCI_SUBCLASS_MSC_IPI_CTLR: UCHAR = 0x03;
const PCI_SUBCLASS_MSC_RAID_CTLR: UCHAR = 0x04;

const PCI_SUBCLASS_NET_ETHERNET_CTLR: UCHAR = 0x00;
const PCI_SUBCLASS_NET_TOKEN_RING_CTLR: UCHAR = 0x01;
const PCI_SUBCLASS_NET_FDDI_CTLR: UCHAR = 0x02;
const PCI_SUBCLASS_NET_ATM_CTLR: UCHAR = 0x03;

const PCI_SUBCLASS_VID_VGA_CTLR: UCHAR = 0x00;
const PCI_SUBCLASS_VID_XGA_CTLR: UCHAR = 0x01;
const PCI_SUBCLASS_VID_3D_CTLR: UCHAR = 0x02;

const PCI_SUBCLASS_MM_VIDEO_DEV: UCHAR = 0x00;
const PCI_SUBCLASS_MM_AUDIO_DEV: UCHAR = 0x01;
const PCI_SUBCLASS_MM_TELEPHONY_DEV: UCHAR = 0x02;

const PCI_SUBCLASS_MEM_RAM: UCHAR = 0x00;
const PCI_SUBCLASS_MEM_FLASH: UCHAR = 0x01;

const PCI_SUBCLASS_BR_HOST: UCHAR = 0x00;
const PCI_SUBCLASS_BR_ISA: UCHAR = 0x01;
const PCI_SUBCLASS_BR_EISA: UCHAR = 0x02;
const PCI_SUBCLASS_BR_MCA: UCHAR = 0x03;
const PCI_SUBCLASS_BR_PCI_TO_PCI: UCHAR = 0x04;
const PCI_SUBCLASS_BR_PCMCIA: UCHAR = 0x05;
const PCI_SUBCLASS_BR_NUBUS: UCHAR = 0x06;
const PCI_SUBCLASS_BR_CARDBUS: UCHAR = 0x07;

const PCI_SUBCLASS_SB_IEEE1394: UCHAR = 0x00;
const PCI_SUBCLASS_SB_ACCESS: UCHAR = 0x01;
const PCI_SUBCLASS_SB_SSA: UCHAR = 0x02;
const PCI_SUBCLASS_SB_USB: UCHAR = 0x03;
const PCI_SUBCLASS_SB_FIBRE_CHANNEL: UCHAR = 0x04;
const PCI_SUBCLASS_SB_SMBUS: UCHAR = 0x05;

// ---------------------------------------------------------------------------
// Substrate workaround: the kernel bindgen lost the PCI_COMMON_HEADER fields of
// PCI_COMMON_CONFIG (it emits only `DeviceSpecific: [u8; 192]` because the
// header is spliced in via an anonymous, GNU-extension struct that bindgen drops).
// In C, `PciConfig.VendorID` etc. resolve through that anonymous header, which
// physically occupies the first 64 bytes of PCI_COMMON_CONFIG. We recover the
// fields by reinterpreting a `&PciConfig` as a `*const PCI_COMMON_HEADER`
// (bindgen DID emit the full header type with the type0/type1/type2 union).
// ---------------------------------------------------------------------------
#[inline]
unsafe fn pci_hdr(device: PPCI_DEVICE) -> *const PCI_COMMON_HEADER {
    ptr::addr_of!((*device).PciConfig) as *const PCI_COMMON_HEADER
}

// ---------------------------------------------------------------------------
// Inline DDK helpers not present in ddk/inline (trivial list primitive).
// ---------------------------------------------------------------------------
#[inline]
unsafe fn InitializeListHead(list_head: *mut LIST_ENTRY) {
    (*list_head).Flink = list_head;
    (*list_head).Blink = list_head;
}

// swprintf — real CRT export, C variadic.
extern "C" {
    fn swprintf(buffer: *mut u16, format: *const u16, ...) -> i32;
}

// ---------------------------------------------------------------------------
// Cross-module callees (fdo.rs / pdo.rs) — pci.h prototypes, NTAPI.
// ---------------------------------------------------------------------------
extern "system" {
    fn FdoPnpControl(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
    fn FdoPowerControl(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
    fn PdoPnpControl(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
    fn PdoPowerControl(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
}

/*** PRIVATE *****************************************************************/

#[no_mangle]
unsafe extern "system" fn PciDispatchDeviceControl(
    _DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let Status: NTSTATUS;

    (*Irp).IoStatus.Information = 0;

    let IrpSp = IoGetCurrentIrpStackLocation(Irp);
    match (*IrpSp).Parameters.DeviceIoControl.IoControlCode {
        _ => {
            // Unknown IOCTL
            Status = STATUS_NOT_IMPLEMENTED;
        }
    }

    if Status != STATUS_PENDING {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
    }

    Status
}

/// Handle Plug and Play IRPs — route to FDO or PDO handler by Common.IsFDO.
#[no_mangle]
unsafe extern "system" fn PciPnpControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let DeviceExtension = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;

    let Status = if (*DeviceExtension).IsFDO != 0 {
        FdoPnpControl(DeviceObject, Irp)
    } else {
        PdoPnpControl(DeviceObject, Irp)
    };

    Status
}

/// Handle power-management IRPs — route to FDO or PDO handler by Common.IsFDO.
#[no_mangle]
unsafe extern "system" fn PciPowerControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let DeviceExtension = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;

    let Status = if (*DeviceExtension).IsFDO != 0 {
        FdoPowerControl(DeviceObject, Irp)
    } else {
        PdoPowerControl(DeviceObject, Irp)
    };

    Status
}

#[no_mangle]
unsafe extern "system" fn PciAddDevice(
    DriverObject: PDRIVER_OBJECT,
    PhysicalDeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS {
    if PhysicalDeviceObject.is_null() {
        return STATUS_SUCCESS;
    }

    let mut Fdo: PDEVICE_OBJECT = ptr::null_mut();
    let Status = IoCreateDevice(
        DriverObject,
        core::mem::size_of::<FDO_DEVICE_EXTENSION>() as ULONG,
        ptr::null_mut(),
        FILE_DEVICE_BUS_EXTENDER,
        FILE_DEVICE_SECURE_OPEN,
        1, // TRUE
        &mut Fdo,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    let DeviceExtension = (*Fdo).DeviceExtension as PFDO_DEVICE_EXTENSION;

    ptr::write_bytes(
        DeviceExtension as *mut u8,
        0,
        core::mem::size_of::<FDO_DEVICE_EXTENSION>(),
    );

    (*DeviceExtension).Common.IsFDO = 1; // TRUE

    (*DeviceExtension).Ldo = IoAttachDeviceToDeviceStack(Fdo, PhysicalDeviceObject);

    (*DeviceExtension).State = dsStopped;

    (*Fdo).Flags &= !DO_DEVICE_INITIALIZING;

    //(*Fdo).Flags |= DO_POWER_PAGABLE;

    STATUS_SUCCESS
}

#[no_mangle]
unsafe extern "system" fn PciUnload(_DriverObject: PDRIVER_OBJECT) {
    /* The driver object extension is destroyed by the I/O manager */
}

/// CODE_SEG("INIT") — locate KD (kernel-debugger) PCI devices via the registry.
unsafe fn PciLocateKdDevices() {
    let mut KeyNameBuffer: [u16; 16] = [0; 16];
    let mut BusNumber: ULONG = 0;
    let mut SlotNumber: ULONG = 0;
    let mut QueryTable: [RTL_QUERY_REGISTRY_TABLE; 3] = core::mem::zeroed();

    QueryTable[0].Flags = RTL_QUERY_REGISTRY_DIRECT | RTL_QUERY_REGISTRY_REQUIRED;
    QueryTable[0].Name = ntkrnl_sys::w!("Bus").as_ptr();
    QueryTable[0].EntryContext = ptr::addr_of_mut!(BusNumber) as PVOID;
    QueryTable[1].Flags = RTL_QUERY_REGISTRY_DIRECT | RTL_QUERY_REGISTRY_REQUIRED;
    QueryTable[1].Name = ntkrnl_sys::w!("Slot").as_ptr();
    QueryTable[1].EntryContext = ptr::addr_of_mut!(SlotNumber) as PVOID;

    // RTL_NUMBER_OF(PciDebuggingDevice) == 2 (the array is [_; 2]); literal avoids
    // forming a reference to the `static mut` just to read its length.
    const COUNT: usize = 2;
    let mut i: usize = 0;
    while i < COUNT {
        // RtlStringCbPrintfW(KeyNameBuffer, sizeof, L"PCI\\Debug\\%d", i)
        swprintf(
            KeyNameBuffer.as_mut_ptr(),
            ntkrnl_sys::w!("PCI\\Debug\\%d").as_ptr(),
            i as i32,
        );

        let Status = RtlQueryRegistryValues(
            RTL_REGISTRY_SERVICES,
            KeyNameBuffer.as_ptr(),
            QueryTable.as_mut_ptr(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            return;
        }

        crate::shared::HasDebuggingDevice = 1; // TRUE

        // PciSlot.u.AsULONG = SlotNumber; read DeviceNumber/FunctionNumber bitfields.
        let mut PciSlot: PCI_SLOT_NUMBER = core::mem::zeroed();
        PciSlot.u.AsULONG = SlotNumber;
        let device_number = PciSlot.u.bits.DeviceNumber();
        let function_number = PciSlot.u.bits.FunctionNumber();

        crate::shared::PciDebuggingDevice[i].set_DeviceNumber(device_number);
        crate::shared::PciDebuggingDevice[i].set_FunctionNumber(function_number);
        crate::shared::PciDebuggingDevice[i].set_BusNumber(BusNumber);
        crate::shared::PciDebuggingDevice[i].set_InUse(1); // TRUE

        DPRINT1!(
            "PCI debugging device %02x:%02x.%x\n",
            BusNumber,
            device_number,
            function_number
        );

        i += 1;
    }
}

/// CODE_SEG("INIT") — driver entry point. NTAPI.
#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    _RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    (*DriverObject).MajorFunction[IRP_MJ_DEVICE_CONTROL] =
        Some(transmute::<*const (), _>(PciDispatchDeviceControl as *const ()));
    (*DriverObject).MajorFunction[IRP_MJ_PNP] =
        Some(transmute::<*const (), _>(PciPnpControl as *const ()));
    (*DriverObject).MajorFunction[IRP_MJ_POWER] =
        Some(transmute::<*const (), _>(PciPowerControl as *const ()));
    (*(*DriverObject).DriverExtension).AddDevice =
        Some(transmute::<*const (), _>(PciAddDevice as *const ()));
    (*DriverObject).DriverUnload =
        Some(transmute::<*const (), _>(PciUnload as *const ()));

    let mut driver_ext: PVOID = ptr::null_mut();
    let Status = IoAllocateDriverObjectExtension(
        DriverObject,
        DriverObject as PVOID,
        core::mem::size_of::<PCI_DRIVER_EXTENSION>() as ULONG,
        &mut driver_ext,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }
    crate::shared::DriverExtension = driver_ext as *mut PCI_DRIVER_EXTENSION;

    ptr::write_bytes(
        crate::shared::DriverExtension as *mut u8,
        0,
        core::mem::size_of::<PCI_DRIVER_EXTENSION>(),
    );

    InitializeListHead(ptr::addr_of_mut!((*crate::shared::DriverExtension).BusListHead));
    // KeInitializeSpinLock (inline::ke) takes *mut base::ULONG_PTR (= *mut usize);
    // KSPIN_LOCK is the nominally-distinct ddk ULONG_PTR (same width) → cast.
    KeInitializeSpinLock(
        ptr::addr_of_mut!((*crate::shared::DriverExtension).BusListLock)
            as *mut ntkrnl_sys::base::ULONG_PTR,
    );

    PciLocateKdDevices();

    STATUS_SUCCESS
}

/*** cross-file string builders (pci.h exports, NTAPI) **********************/

#[no_mangle]
pub unsafe extern "system" fn PciCreateDeviceIDString(
    DeviceID: PUNICODE_STRING,
    Device: PPCI_DEVICE,
) -> NTSTATUS {
    let mut Buffer: [u16; 256] = [0; 256];
    let hdr = pci_hdr(Device);

    swprintf(
        Buffer.as_mut_ptr(),
        ntkrnl_sys::w!("PCI\\VEN_%04X&DEV_%04X&SUBSYS_%08X&REV_%02X").as_ptr(),
        (*hdr).VendorID as u32,
        (*hdr).DeviceID as u32,
        (((*hdr).u.type0.SubSystemID as u32) << 16) + (*hdr).u.type0.SubVendorID as u32,
        (*hdr).RevisionID as u32,
    );

    if RtlCreateUnicodeString(DeviceID, Buffer.as_ptr()) != 0 {
        STATUS_SUCCESS
    } else {
        STATUS_INSUFFICIENT_RESOURCES
    }
}

#[no_mangle]
pub unsafe extern "system" fn PciCreateInstanceIDString(
    InstanceID: PUNICODE_STRING,
    Device: PPCI_DEVICE,
) -> NTSTATUS {
    let mut Buffer: [u16; 3] = [0; 3];

    swprintf(
        Buffer.as_mut_ptr(),
        ntkrnl_sys::w!("%02X").as_ptr(),
        ((*Device).SlotNumber.u.AsULONG & 0xff) as u32,
    );

    if RtlCreateUnicodeString(InstanceID, Buffer.as_ptr()) != 0 {
        STATUS_SUCCESS
    } else {
        STATUS_INSUFFICIENT_RESOURCES
    }
}

#[no_mangle]
pub unsafe extern "system" fn PciCreateHardwareIDsString(
    HardwareIDs: PUNICODE_STRING,
    Device: PPCI_DEVICE,
) -> NTSTATUS {
    let mut Buffer: [u16; 256] = [0; 256];
    let hdr = pci_hdr(Device);
    let mut Index: i32 = 0;

    Index += swprintf(
        Buffer.as_mut_ptr().offset(Index as isize),
        ntkrnl_sys::w!("PCI\\VEN_%04X&DEV_%04X&SUBSYS_%08X&REV_%02X").as_ptr(),
        (*hdr).VendorID as u32,
        (*hdr).DeviceID as u32,
        (((*hdr).u.type0.SubSystemID as u32) << 16) + (*hdr).u.type0.SubVendorID as u32,
        (*hdr).RevisionID as u32,
    );
    Index += 1;

    Index += swprintf(
        Buffer.as_mut_ptr().offset(Index as isize),
        ntkrnl_sys::w!("PCI\\VEN_%04X&DEV_%04X&SUBSYS_%08X").as_ptr(),
        (*hdr).VendorID as u32,
        (*hdr).DeviceID as u32,
        (((*hdr).u.type0.SubSystemID as u32) << 16) + (*hdr).u.type0.SubVendorID as u32,
    );
    Index += 1;

    Index += swprintf(
        Buffer.as_mut_ptr().offset(Index as isize),
        ntkrnl_sys::w!("PCI\\VEN_%04X&DEV_%04X&CC_%02X%02X%02X").as_ptr(),
        (*hdr).VendorID as u32,
        (*hdr).DeviceID as u32,
        (*hdr).BaseClass as u32,
        (*hdr).SubClass as u32,
        (*hdr).ProgIf as u32,
    );
    Index += 1;

    Index += swprintf(
        Buffer.as_mut_ptr().offset(Index as isize),
        ntkrnl_sys::w!("PCI\\VEN_%04X&DEV_%04X&CC_%02X%02X").as_ptr(),
        (*hdr).VendorID as u32,
        (*hdr).DeviceID as u32,
        (*hdr).BaseClass as u32,
        (*hdr).SubClass as u32,
    );
    Index += 1;

    Buffer[Index as usize] = UNICODE_NULL;

    let mut BufferU: UNICODE_STRING = core::mem::zeroed();
    BufferU.Length = (Index as u16) * core::mem::size_of::<u16>() as u16;
    BufferU.MaximumLength = BufferU.Length;
    BufferU.Buffer = Buffer.as_mut_ptr();

    PciDuplicateUnicodeString(0, &BufferU, HardwareIDs)
}

#[no_mangle]
pub unsafe extern "system" fn PciCreateCompatibleIDsString(
    CompatibleIDs: PUNICODE_STRING,
    Device: PPCI_DEVICE,
) -> NTSTATUS {
    let mut Buffer: [u16; 256] = [0; 256];
    let hdr = pci_hdr(Device);
    let mut Index: i32 = 0;

    Index += swprintf(
        Buffer.as_mut_ptr().offset(Index as isize),
        ntkrnl_sys::w!("PCI\\VEN_%04X&DEV_%04X&REV_%02X").as_ptr(),
        (*hdr).VendorID as u32,
        (*hdr).DeviceID as u32,
        (*hdr).RevisionID as u32,
    );
    Index += 1;

    Index += swprintf(
        Buffer.as_mut_ptr().offset(Index as isize),
        ntkrnl_sys::w!("PCI\\VEN_%04X&DEV_%04X").as_ptr(),
        (*hdr).VendorID as u32,
        (*hdr).DeviceID as u32,
    );
    Index += 1;

    Index += swprintf(
        Buffer.as_mut_ptr().offset(Index as isize),
        ntkrnl_sys::w!("PCI\\VEN_%04X&CC_%02X%02X%02X").as_ptr(),
        (*hdr).VendorID as u32,
        (*hdr).BaseClass as u32,
        (*hdr).SubClass as u32,
        (*hdr).ProgIf as u32,
    );
    Index += 1;

    Index += swprintf(
        Buffer.as_mut_ptr().offset(Index as isize),
        ntkrnl_sys::w!("PCI\\VEN_%04X&CC_%02X%02X").as_ptr(),
        (*hdr).VendorID as u32,
        (*hdr).BaseClass as u32,
        (*hdr).SubClass as u32,
    );
    Index += 1;

    Index += swprintf(
        Buffer.as_mut_ptr().offset(Index as isize),
        ntkrnl_sys::w!("PCI\\VEN_%04X").as_ptr(),
        (*hdr).VendorID as u32,
    );
    Index += 1;

    Index += swprintf(
        Buffer.as_mut_ptr().offset(Index as isize),
        ntkrnl_sys::w!("PCI\\CC_%02X%02X%02X").as_ptr(),
        (*hdr).BaseClass as u32,
        (*hdr).SubClass as u32,
        (*hdr).ProgIf as u32,
    );
    Index += 1;

    Index += swprintf(
        Buffer.as_mut_ptr().offset(Index as isize),
        ntkrnl_sys::w!("PCI\\CC_%02X%02X").as_ptr(),
        (*hdr).BaseClass as u32,
        (*hdr).SubClass as u32,
    );
    Index += 1;

    Buffer[Index as usize] = UNICODE_NULL;

    let mut BufferU: UNICODE_STRING = core::mem::zeroed();
    BufferU.Length = (Index as u16) * core::mem::size_of::<u16>() as u16;
    BufferU.MaximumLength = BufferU.Length;
    BufferU.Buffer = Buffer.as_mut_ptr();

    PciDuplicateUnicodeString(0, &BufferU, CompatibleIDs)
}

#[no_mangle]
pub unsafe extern "system" fn PciCreateDeviceDescriptionString(
    DeviceDescription: PUNICODE_STRING,
    Device: PPCI_DEVICE,
) -> NTSTATUS {
    let hdr = pci_hdr(Device);
    let BaseClass = (*hdr).BaseClass;
    let SubClass = (*hdr).SubClass;

    // Each arm yields the wide L"..." literal as a NUL-terminated *const u16.
    let Description: *const u16 = match BaseClass {
        PCI_CLASS_PRE_20 => match SubClass {
            PCI_SUBCLASS_PRE_20_VGA => ntkrnl_sys::w!("VGA device").as_ptr(),
            // default + PCI_SUBCLASS_PRE_20_NON_VGA fall through to the same arm
            PCI_SUBCLASS_PRE_20_NON_VGA => ntkrnl_sys::w!("PCI device").as_ptr(),
            _ => ntkrnl_sys::w!("PCI device").as_ptr(),
        },

        PCI_CLASS_MASS_STORAGE_CTLR => match SubClass {
            PCI_SUBCLASS_MSC_SCSI_BUS_CTLR => ntkrnl_sys::w!("SCSI controller").as_ptr(),
            PCI_SUBCLASS_MSC_IDE_CTLR => ntkrnl_sys::w!("IDE controller").as_ptr(),
            PCI_SUBCLASS_MSC_FLOPPY_CTLR => ntkrnl_sys::w!("Floppy disk controller").as_ptr(),
            PCI_SUBCLASS_MSC_IPI_CTLR => ntkrnl_sys::w!("IPI controller").as_ptr(),
            PCI_SUBCLASS_MSC_RAID_CTLR => ntkrnl_sys::w!("RAID controller").as_ptr(),
            _ => ntkrnl_sys::w!("Mass storage controller").as_ptr(),
        },

        PCI_CLASS_NETWORK_CTLR => match SubClass {
            PCI_SUBCLASS_NET_ETHERNET_CTLR => ntkrnl_sys::w!("Ethernet controller").as_ptr(),
            PCI_SUBCLASS_NET_TOKEN_RING_CTLR => ntkrnl_sys::w!("Token-Ring controller").as_ptr(),
            PCI_SUBCLASS_NET_FDDI_CTLR => ntkrnl_sys::w!("FDDI controller").as_ptr(),
            PCI_SUBCLASS_NET_ATM_CTLR => ntkrnl_sys::w!("ATM controller").as_ptr(),
            _ => ntkrnl_sys::w!("Network controller").as_ptr(),
        },

        PCI_CLASS_DISPLAY_CTLR => match SubClass {
            PCI_SUBCLASS_VID_VGA_CTLR => ntkrnl_sys::w!("VGA display controller").as_ptr(),
            PCI_SUBCLASS_VID_XGA_CTLR => ntkrnl_sys::w!("XGA display controller").as_ptr(),
            PCI_SUBCLASS_VID_3D_CTLR => ntkrnl_sys::w!("Multimedia display controller").as_ptr(),
            _ => ntkrnl_sys::w!("Other display controller").as_ptr(),
        },

        PCI_CLASS_MULTIMEDIA_DEV => match SubClass {
            PCI_SUBCLASS_MM_VIDEO_DEV => ntkrnl_sys::w!("Multimedia video device").as_ptr(),
            PCI_SUBCLASS_MM_AUDIO_DEV => ntkrnl_sys::w!("Multimedia audio device").as_ptr(),
            PCI_SUBCLASS_MM_TELEPHONY_DEV => ntkrnl_sys::w!("Multimedia telephony device").as_ptr(),
            _ => ntkrnl_sys::w!("Other multimedia device").as_ptr(),
        },

        PCI_CLASS_MEMORY_CTLR => match SubClass {
            PCI_SUBCLASS_MEM_RAM => ntkrnl_sys::w!("PCI Memory").as_ptr(),
            PCI_SUBCLASS_MEM_FLASH => ntkrnl_sys::w!("PCI Flash Memory").as_ptr(),
            _ => ntkrnl_sys::w!("Other memory controller").as_ptr(),
        },

        PCI_CLASS_BRIDGE_DEV => match SubClass {
            PCI_SUBCLASS_BR_HOST => ntkrnl_sys::w!("PCI-Host bridge").as_ptr(),
            PCI_SUBCLASS_BR_ISA => ntkrnl_sys::w!("PCI-ISA bridge").as_ptr(),
            PCI_SUBCLASS_BR_EISA => ntkrnl_sys::w!("PCI-EISA bridge").as_ptr(),
            PCI_SUBCLASS_BR_MCA => ntkrnl_sys::w!("PCI-Micro Channel bridge").as_ptr(),
            PCI_SUBCLASS_BR_PCI_TO_PCI => ntkrnl_sys::w!("PCI-PCI bridge").as_ptr(),
            PCI_SUBCLASS_BR_PCMCIA => ntkrnl_sys::w!("PCI-PCMCIA bridge").as_ptr(),
            PCI_SUBCLASS_BR_NUBUS => ntkrnl_sys::w!("PCI-NUBUS bridge").as_ptr(),
            PCI_SUBCLASS_BR_CARDBUS => ntkrnl_sys::w!("PCI-CARDBUS bridge").as_ptr(),
            _ => ntkrnl_sys::w!("Other bridge device").as_ptr(),
        },

        PCI_CLASS_SIMPLE_COMMS_CTLR => match SubClass {
            _ => ntkrnl_sys::w!("Communication device").as_ptr(),
        },

        PCI_CLASS_BASE_SYSTEM_DEV => match SubClass {
            _ => ntkrnl_sys::w!("System device").as_ptr(),
        },

        PCI_CLASS_INPUT_DEV => match SubClass {
            _ => ntkrnl_sys::w!("Input device").as_ptr(),
        },

        PCI_CLASS_DOCKING_STATION => match SubClass {
            _ => ntkrnl_sys::w!("Docking station").as_ptr(),
        },

        PCI_CLASS_PROCESSOR => match SubClass {
            _ => ntkrnl_sys::w!("Processor").as_ptr(),
        },

        PCI_CLASS_SERIAL_BUS_CTLR => match SubClass {
            PCI_SUBCLASS_SB_IEEE1394 => ntkrnl_sys::w!("FireWire controller").as_ptr(),
            PCI_SUBCLASS_SB_ACCESS => ntkrnl_sys::w!("ACCESS bus controller").as_ptr(),
            PCI_SUBCLASS_SB_SSA => ntkrnl_sys::w!("SSA controller").as_ptr(),
            PCI_SUBCLASS_SB_USB => ntkrnl_sys::w!("USB controller").as_ptr(),
            PCI_SUBCLASS_SB_FIBRE_CHANNEL => ntkrnl_sys::w!("Fibre Channel controller").as_ptr(),
            PCI_SUBCLASS_SB_SMBUS => ntkrnl_sys::w!("SMBus controller").as_ptr(),
            _ => ntkrnl_sys::w!("Other serial bus controller").as_ptr(),
        },

        _ => ntkrnl_sys::w!("Other PCI Device").as_ptr(),
    };

    if RtlCreateUnicodeString(DeviceDescription, Description) != 0 {
        STATUS_SUCCESS
    } else {
        STATUS_INSUFFICIENT_RESOURCES
    }
}

#[no_mangle]
pub unsafe extern "system" fn PciCreateDeviceLocationString(
    DeviceLocation: PUNICODE_STRING,
    Device: PPCI_DEVICE,
) -> NTSTATUS {
    let mut Buffer: [u16; 256] = [0; 256];

    swprintf(
        Buffer.as_mut_ptr(),
        ntkrnl_sys::w!("PCI-Bus %lu, Device %u, Function %u").as_ptr(),
        (*Device).BusNumber,
        (*Device).SlotNumber.u.bits.DeviceNumber(),
        (*Device).SlotNumber.u.bits.FunctionNumber(),
    );

    if RtlCreateUnicodeString(DeviceLocation, Buffer.as_ptr()) != 0 {
        STATUS_SUCCESS
    } else {
        STATUS_INSUFFICIENT_RESOURCES
    }
}

#[no_mangle]
pub unsafe extern "system" fn PciDuplicateUnicodeString(
    Flags: ULONG,
    SourceString: PCUNICODE_STRING,
    DestinationString: PUNICODE_STRING,
) -> NTSTATUS {
    if SourceString.is_null()
        || DestinationString.is_null()
        || (*SourceString).Length > (*SourceString).MaximumLength
        || ((*SourceString).Length == 0
            && (*SourceString).MaximumLength > 0
            && (*SourceString).Buffer.is_null())
        || Flags == RTL_DUPLICATE_UNICODE_STRING_ALLOCATE_NULL_STRING
        || Flags >= 4
    {
        return STATUS_INVALID_PARAMETER;
    }

    if (*SourceString).Length == 0
        && Flags
            != (RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE
                | RTL_DUPLICATE_UNICODE_STRING_ALLOCATE_NULL_STRING)
    {
        (*DestinationString).Length = 0;
        (*DestinationString).MaximumLength = 0;
        (*DestinationString).Buffer = ptr::null_mut();
    } else {
        let mut DestMaxLength: u16 = (*SourceString).Length;

        if Flags & RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE != 0 {
            DestMaxLength += core::mem::size_of::<u16>() as u16; // sizeof(UNICODE_NULL)
        }

        (*DestinationString).Buffer = ExAllocatePoolWithTag(
            _POOL_TYPE_PagedPool,
            DestMaxLength as SIZE_T,
            TAG_PCI,
        ) as PWCH;
        if (*DestinationString).Buffer.is_null() {
            return STATUS_NO_MEMORY;
        }

        ptr::copy_nonoverlapping(
            (*SourceString).Buffer as *const u8,
            (*DestinationString).Buffer as *mut u8,
            (*SourceString).Length as usize,
        );
        (*DestinationString).Length = (*SourceString).Length;
        (*DestinationString).MaximumLength = DestMaxLength;

        if Flags & RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE != 0 {
            *(*DestinationString)
                .Buffer
                .add((*DestinationString).Length as usize / core::mem::size_of::<u16>()) = 0;
        }
    }

    STATUS_SUCCESS
}

/* EOF */
