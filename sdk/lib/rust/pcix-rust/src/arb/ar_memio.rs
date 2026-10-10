#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::guids::*;
use crate::arb::arb_comn::PciArbiterConstructor;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{ARBITER_INTERFACE, PARBITER_INTERFACE, PARBITER_INSTANCE, PINTERFACE, INT32,
    CM_RESOURCE_TYPE, PIO_RESOURCE_DESCRIPTOR, PCM_PARTIAL_RESOURCE_DESCRIPTOR, PULONGLONG,
    RtlIoDecodeMemIoResource, RtlCmDecodeMemIoResource};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[no_mangle]
pub static mut ArbiterInterfaceMemory: PCI_INTERFACE = unsafe {
    _PCI_INTERFACE {
        InterfaceType: &raw const GUID_ARBITER_INTERFACE_STANDARD,
        MinSize: ::core::mem::size_of::<ARBITER_INTERFACE>() as USHORT,
        MinVersion: 0 as USHORT,
        MaxVersion: 0 as USHORT,
        Flags: PCI_INTERFACE_FDO as USHORT,
        ReferenceCount: 0 as LONG,
        Signature: PciArb_Memory,
        Constructor: Some(core::mem::transmute(armem_Constructor as *const ())),
        Initializer: Some(core::mem::transmute(armem_Initializer as *const ())),
    }
};

#[no_mangle]
pub static mut ArbiterInterfaceIo: PCI_INTERFACE = unsafe {
    _PCI_INTERFACE {
        InterfaceType: &raw const GUID_ARBITER_INTERFACE_STANDARD,
        MinSize: ::core::mem::size_of::<ARBITER_INTERFACE>() as USHORT,
        MinVersion: 0 as USHORT,
        MaxVersion: 0 as USHORT,
        Flags: PCI_INTERFACE_FDO as USHORT,
        ReferenceCount: 0 as LONG,
        Signature: PciArb_Io,
        Constructor: Some(core::mem::transmute(ario_Constructor as *const ())),
        Initializer: Some(core::mem::transmute(ario_Initializer as *const ())),
    }
};

#[no_mangle]
pub unsafe extern "system" fn ario_UnpackRequirement(
    Descriptor: PIO_RESOURCE_DESCRIPTOR,
    Minimum: PULONGLONG,
    Maximum: PULONGLONG,
    Length: PULONGLONG,
    Alignment: PULONGLONG,
) -> NTSTATUS {
    // This arbiter is only ever handed I/O port descriptors
    if (*Descriptor).Type as u32 != CmResourceTypePort as u32 {
        return STATUS_INVALID_PARAMETER;
    }

    // An alignment of zero would place the range nowhere, so make it one byte
    *Minimum = (*Descriptor).u.Port.MinimumAddress.QuadPart as ULONGLONG;
    *Maximum = (*Descriptor).u.Port.MaximumAddress.QuadPart as ULONGLONG;
    *Length = (*Descriptor).u.Port.Length as ULONGLONG;
    *Alignment = if (*Descriptor).u.Port.Alignment != 0 {
        (*Descriptor).u.Port.Alignment as ULONGLONG
    } else {
        1
    };
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ario_PackResource(
    Descriptor: PIO_RESOURCE_DESCRIPTOR,
    Start: ULONGLONG,
    Resource: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
) -> NTSTATUS {
    // Turn the placement the engine settled on back into an I/O port resource
    if (*Descriptor).Type as u32 != CmResourceTypePort as u32 {
        return STATUS_INVALID_PARAMETER;
    }

    (*Resource).Type = CmResourceTypePort as UCHAR;
    (*Resource).Flags = (*Descriptor).Flags;
    (*Resource).ShareDisposition = (*Descriptor).ShareDisposition;
    (*Resource).u.Port.Start.QuadPart = Start as LONGLONG;
    (*Resource).u.Port.Length = (*Descriptor).u.Port.Length;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ario_UnpackResource(
    Resource: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
    Start: PULONGLONG,
    Length: PULONGLONG,
) -> NTSTATUS {
    // Read a previously packed I/O port resource back out
    if (*Resource).Type as u32 != CmResourceTypePort as u32 {
        return STATUS_INVALID_PARAMETER;
    }

    *Start = (*Resource).u.Port.Start.QuadPart as ULONGLONG;
    *Length = (*Resource).u.Port.Length as ULONGLONG;
    STATUS_SUCCESS
}

/// The three ScoreRequirement callbacks are the same C body over a different unpacker.
unsafe fn score_requirement(
    Unpack: unsafe extern "system" fn(
        PIO_RESOURCE_DESCRIPTOR,
        PULONGLONG,
        PULONGLONG,
        PULONGLONG,
        PULONGLONG,
    ) -> NTSTATUS,
    Descriptor: PIO_RESOURCE_DESCRIPTOR,
) -> INT32 {
    let mut Minimum: ULONGLONG = 0;
    let mut Maximum: ULONGLONG = 0;
    let mut Length: ULONGLONG = 0;
    let mut Alignment: ULONGLONG = 0;

    // A requirement this arbiter cannot read is treated as unsatisfiable
    if !NT_SUCCESS(Unpack(Descriptor, &mut Minimum, &mut Maximum, &mut Length, &mut Alignment))
        || Maximum < Minimum
    {
        return -1;
    }

    // MAXLONG is ntdef.h's 0x7fffffff, LONG's maximum
    if Maximum - Minimum >= LONG::MAX as ULONGLONG {
        return LONG::MAX as INT32;
    }
    (Maximum - Minimum + 1) as INT32
}

#[no_mangle]
pub unsafe extern "system" fn ario_ScoreRequirement(Descriptor: PIO_RESOURCE_DESCRIPTOR) -> INT32 {
    score_requirement(ario_UnpackRequirement, Descriptor)
}

#[no_mangle]
pub unsafe extern "system" fn ario_Initializer(Instance: PVOID) -> NTSTATUS {
    let Arbiter = Instance as PPCI_ARBITER_INSTANCE;
    PAGED_CODE!();

    // Start from a clean engine instance
    core::ptr::write_bytes(
        core::ptr::addr_of_mut!((*Arbiter).CommonInstance) as *mut u8,
        0,
        core::mem::size_of_val(&(*Arbiter).CommonInstance),
    );
    let FdoExtension = (*Arbiter).BusFdoExtension;

    // Teach the engine how to read and write I/O port descriptors
    (*Arbiter).CommonInstance.UnpackRequirement = Some(ario_UnpackRequirement);
    (*Arbiter).CommonInstance.PackResource = Some(ario_PackResource);
    (*Arbiter).CommonInstance.UnpackResource = Some(ario_UnpackResource);
    (*Arbiter).CommonInstance.ScoreRequirement = Some(ario_ScoreRequirement);

    // The rest of the engine, and the PCI assignment ordering, is generic
    let Status = ArbiterLibInitializeInstance(
        core::ptr::addr_of_mut!((*Arbiter).CommonInstance),
        (*FdoExtension).FunctionalDeviceObject,
        CmResourceTypePort as CM_RESOURCE_TYPE,
        core::ptr::addr_of!((*Arbiter).InstanceName) as PCWSTR,
        w!("Pci").as_ptr(),
        None,
    );
    if !NT_SUCCESS(Status) {
        win32macros_rust::DPRINT1!("ario_Initializer: init arbiter returned %X\n", Status);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn ario_Constructor(
    DeviceExtension: PVOID,
    PciInterface: PVOID,
    InterfaceData: PVOID,
    Version: USHORT,
    Size: USHORT,
    Interface: PINTERFACE,
) -> NTSTATUS {
    let FdoExtension = DeviceExtension as PPCI_FDO_EXTENSION;
    PAGED_CODE!();

    UNREFERENCED_PARAMETER!(PciInterface);
    UNREFERENCED_PARAMETER!(Version);
    UNREFERENCED_PARAMETER!(Size);

    // This one only arbitrates I/O ports
    if InterfaceData as ULONG_PTR != CmResourceTypePort as ULONG_PTR {
        return STATUS_INVALID_PARAMETER_5;
    }

    // Hand out the instance that was built for this bus
    PciArbiterConstructor(FdoExtension, PciArb_Io, Interface as PARBITER_INTERFACE)
}

#[no_mangle]
pub unsafe extern "system" fn ario_ApplyBrokenVideoHack(FdoExtension: PPCI_FDO_EXTENSION) {
    // Only valid for root FDOs who are being applied the hack for the first time
    ntkrnl_sys::ASSERT!((*FdoExtension).BrokenVideoHackApplied == 0);
    ntkrnl_sys::ASSERT!((*FdoExtension).BusRootFdoExtension == FdoExtension);

    // Find the I/O arbiter
    let PciArbiter = PciFindNextSecondaryExtension(
        (*FdoExtension).SecondaryExtension.Next,
        PciArb_Io,
    ) as PPCI_ARBITER_INSTANCE;
    ntkrnl_sys::ASSERT!(!PciArbiter.is_null());
    if PciArbiter.is_null() {
        return;
    }

    // Get the Arb instance
    let CommonInstance: PARBITER_INSTANCE = core::ptr::addr_of_mut!((*PciArbiter).CommonInstance);

    // Free the two lists, enabling full VGA access
    ArbiterLibFreeOrderingList(core::ptr::addr_of_mut!((*CommonInstance).OrderingList));
    ArbiterLibFreeOrderingList(core::ptr::addr_of_mut!((*CommonInstance).ReservedList));

    // Build the ordering for broken video PCI access
    let Status = ArbiterLibDefaultAssignmentOrdering(
        CommonInstance,
        w!("Pci").as_ptr(),
        w!("BrokenVideo").as_ptr(),
        None,
    );
    ntkrnl_sys::ASSERT!(NT_SUCCESS(Status));
    if !NT_SUCCESS(Status) {
        win32macros_rust::DPRINT1!("ario_ApplyBrokenVideoHack: ordering failed %X\n", Status);
    }

    // Now the hack has been applied
    (*FdoExtension).BrokenVideoHackApplied = TRUE as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn armem_UnpackRequirement(
    Descriptor: PIO_RESOURCE_DESCRIPTOR,
    Minimum: PULONGLONG,
    Maximum: PULONGLONG,
    Length: PULONGLONG,
    Alignment: PULONGLONG,
) -> NTSTATUS {
    // This arbiter is only ever handed device memory descriptors, large ones included
    if (*Descriptor).Type as u32 != CmResourceTypeMemory as u32
        && (*Descriptor).Type as u32 != CmResourceTypeMemoryLarge as u32
    {
        return STATUS_INVALID_PARAMETER;
    }

    // An alignment of zero would place the range nowhere, so make it one byte
    *Length = RtlIoDecodeMemIoResource(Descriptor, Alignment, Minimum, Maximum);
    if *Alignment == 0 {
        *Alignment = 1;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn armem_PackResource(
    Descriptor: PIO_RESOURCE_DESCRIPTOR,
    Start: ULONGLONG,
    Resource: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
) -> NTSTATUS {
    // Turn the placement the engine settled on back into a memory resource
    if (*Descriptor).Type as u32 != CmResourceTypeMemory as u32
        && (*Descriptor).Type as u32 != CmResourceTypeMemoryLarge as u32
    {
        return STATUS_INVALID_PARAMETER;
    }

    // The flags say how a large length is scaled, so both come across unchanged
    (*Resource).Type = (*Descriptor).Type;
    (*Resource).Flags = (*Descriptor).Flags;
    (*Resource).ShareDisposition = (*Descriptor).ShareDisposition;
    (*Resource).u.Memory.Start.QuadPart = Start as LONGLONG;
    (*Resource).u.Memory.Length = (*Descriptor).u.Memory.Length;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn armem_UnpackResource(
    Resource: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
    Start: PULONGLONG,
    Length: PULONGLONG,
) -> NTSTATUS {
    // Read a previously packed memory resource back out
    if (*Resource).Type as u32 != CmResourceTypeMemory as u32
        && (*Resource).Type as u32 != CmResourceTypeMemoryLarge as u32
    {
        return STATUS_INVALID_PARAMETER;
    }

    *Length = RtlCmDecodeMemIoResource(Resource, Start);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn armem_ScoreRequirement(Descriptor: PIO_RESOURCE_DESCRIPTOR) -> INT32 {
    score_requirement(armem_UnpackRequirement, Descriptor)
}

#[no_mangle]
pub unsafe extern "system" fn armem_Initializer(Instance: PVOID) -> NTSTATUS {
    let Arbiter = Instance as PPCI_ARBITER_INSTANCE;
    PAGED_CODE!();

    // Start from a clean engine instance
    core::ptr::write_bytes(
        core::ptr::addr_of_mut!((*Arbiter).CommonInstance) as *mut u8,
        0,
        core::mem::size_of_val(&(*Arbiter).CommonInstance),
    );
    let FdoExtension = (*Arbiter).BusFdoExtension;

    // Teach the engine how to read and write memory descriptors
    (*Arbiter).CommonInstance.UnpackRequirement = Some(armem_UnpackRequirement);
    (*Arbiter).CommonInstance.PackResource = Some(armem_PackResource);
    (*Arbiter).CommonInstance.UnpackResource = Some(armem_UnpackResource);
    (*Arbiter).CommonInstance.ScoreRequirement = Some(armem_ScoreRequirement);

    // The rest of the engine, and the PCI assignment ordering, is generic
    let Status = ArbiterLibInitializeInstance(
        core::ptr::addr_of_mut!((*Arbiter).CommonInstance),
        (*FdoExtension).FunctionalDeviceObject,
        CmResourceTypeMemory as CM_RESOURCE_TYPE,
        core::ptr::addr_of!((*Arbiter).InstanceName) as PCWSTR,
        w!("Pci").as_ptr(),
        None,
    );
    if !NT_SUCCESS(Status) {
        win32macros_rust::DPRINT1!("armem_Initializer: init arbiter returned %X\n", Status);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn armem_Constructor(
    DeviceExtension: PVOID,
    PciInterface: PVOID,
    InterfaceData: PVOID,
    Version: USHORT,
    Size: USHORT,
    Interface: PINTERFACE,
) -> NTSTATUS {
    let FdoExtension = DeviceExtension as PPCI_FDO_EXTENSION;
    PAGED_CODE!();

    UNREFERENCED_PARAMETER!(PciInterface);
    UNREFERENCED_PARAMETER!(Version);
    UNREFERENCED_PARAMETER!(Size);

    // This one only arbitrates device memory
    if InterfaceData as ULONG_PTR != CmResourceTypeMemory as ULONG_PTR {
        return STATUS_INVALID_PARAMETER_5;
    }

    // Hand out the instance that was built for this bus
    PciArbiterConstructor(FdoExtension, PciArb_Memory, Interface as PARBITER_INTERFACE)
}
