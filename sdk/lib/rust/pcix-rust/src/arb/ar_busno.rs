#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::guids::*;
use crate::arb::arb_comn::PciArbiterConstructor;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{ARBITER_INTERFACE, PARBITER_INTERFACE, PINTERFACE, INT32, CM_RESOURCE_TYPE,
    PIO_RESOURCE_DESCRIPTOR, PCM_PARTIAL_RESOURCE_DESCRIPTOR, PULONGLONG};
use ntkrnl_sys::driver_ddk::DbgPrint;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[no_mangle]
pub static mut ArbiterInterfaceBusNumber: PCI_INTERFACE = unsafe {
    _PCI_INTERFACE {
        InterfaceType: &raw const GUID_ARBITER_INTERFACE_STANDARD,
        MinSize: ::core::mem::size_of::<ARBITER_INTERFACE>() as USHORT,
        MinVersion: 0 as USHORT,
        MaxVersion: 0 as USHORT,
        Flags: PCI_INTERFACE_FDO as USHORT,
        ReferenceCount: 0 as LONG,
        Signature: PciArb_BusNumber,
        Constructor: Some(core::mem::transmute(arbusno_Constructor as *const ())),
        Initializer: Some(core::mem::transmute(arbusno_Initializer as *const ())),
    }
};

#[no_mangle]
pub unsafe extern "system" fn arbusno_UnpackRequirement(
    Descriptor: PIO_RESOURCE_DESCRIPTOR,
    Minimum: PULONGLONG,
    Maximum: PULONGLONG,
    Length: PULONGLONG,
    Alignment: PULONGLONG,
) -> NTSTATUS {
    // This arbiter is only ever handed bus number descriptors
    if (*Descriptor).Type as u32 != CmResourceTypeBusNumber as u32 {
        return STATUS_INVALID_PARAMETER;
    }

    // Bus numbers are counted, not addressed, so they are always unit-aligned
    *Minimum = (*Descriptor).u.BusNumber.MinBusNumber as ULONGLONG;
    *Maximum = (*Descriptor).u.BusNumber.MaxBusNumber as ULONGLONG;
    *Length = (*Descriptor).u.BusNumber.Length as ULONGLONG;
    *Alignment = 1;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn arbusno_PackResource(
    Descriptor: PIO_RESOURCE_DESCRIPTOR,
    Start: ULONGLONG,
    Resource: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
) -> NTSTATUS {
    // Turn the run of bus numbers the engine settled on into a resource
    if (*Descriptor).Type as u32 != CmResourceTypeBusNumber as u32 {
        return STATUS_INVALID_PARAMETER;
    }

    (*Resource).Type = CmResourceTypeBusNumber as UCHAR;
    (*Resource).Flags = (*Descriptor).Flags;
    (*Resource).ShareDisposition = (*Descriptor).ShareDisposition;
    (*Resource).u.BusNumber.Start = Start as ULONG;
    (*Resource).u.BusNumber.Length = (*Descriptor).u.BusNumber.Length;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn arbusno_UnpackResource(
    Resource: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
    Start: PULONGLONG,
    Length: PULONGLONG,
) -> NTSTATUS {
    // Read a previously packed run of bus numbers back out
    if (*Resource).Type as u32 != CmResourceTypeBusNumber as u32 {
        return STATUS_INVALID_PARAMETER;
    }

    *Start = (*Resource).u.BusNumber.Start as ULONGLONG;
    *Length = (*Resource).u.BusNumber.Length as ULONGLONG;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn arbusno_ScoreRequirement(Descriptor: PIO_RESOURCE_DESCRIPTOR) -> INT32 {
    let mut Minimum: ULONGLONG = 0;
    let mut Maximum: ULONGLONG = 0;
    let mut Length: ULONGLONG = 0;
    let mut Alignment: ULONGLONG = 0;

    // A requirement this arbiter cannot read is treated as unsatisfiable
    if !NT_SUCCESS(arbusno_UnpackRequirement(
        Descriptor,
        &mut Minimum,
        &mut Maximum,
        &mut Length,
        &mut Alignment,
    )) || Maximum < Minimum
    {
        return -1;
    }

    // The score is how many places the run could go, so that the engine places
    // the most constrained requirements first.
    // MAXLONG is ntdef.h's 0x7fffffff, LONG's maximum
    if Maximum - Minimum >= LONG::MAX as ULONGLONG {
        return LONG::MAX as INT32;
    }
    (Maximum - Minimum + 1) as INT32
}

#[no_mangle]
pub unsafe extern "system" fn arbusno_Initializer(Instance: PVOID) -> NTSTATUS {
    let Arbiter = Instance as PPCI_ARBITER_INSTANCE;
    PAGED_CODE!();

    core::ptr::write_bytes(
        core::ptr::addr_of_mut!((*Arbiter).CommonInstance) as *mut u8,
        0,
        core::mem::size_of_val(&(*Arbiter).CommonInstance),
    );

    let FdoExtension = (*Arbiter).BusFdoExtension;

    (*Arbiter).CommonInstance.UnpackRequirement = Some(arbusno_UnpackRequirement);
    (*Arbiter).CommonInstance.PackResource = Some(arbusno_PackResource);
    (*Arbiter).CommonInstance.UnpackResource = Some(arbusno_UnpackResource);
    (*Arbiter).CommonInstance.ScoreRequirement = Some(arbusno_ScoreRequirement);

    let Status = ArbiterLibInitializeInstance(
        core::ptr::addr_of_mut!((*Arbiter).CommonInstance),
        (*FdoExtension).FunctionalDeviceObject,
        CmResourceTypeBusNumber as CM_RESOURCE_TYPE,
        core::ptr::addr_of!((*Arbiter).InstanceName) as PCWSTR,
        w!("Pci").as_ptr(),
        None,
    );
    if !NT_SUCCESS(Status) {
        win32macros_rust::DPRINT1!("arbusno_Initializer: init arbiter return %X\n", Status);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn arbusno_Constructor(
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

    // This one only arbitrates bus numbers
    if InterfaceData as ULONG_PTR != CmResourceTypeBusNumber as ULONG_PTR {
        return STATUS_INVALID_PARAMETER_5;
    }

    // Hand out the instance that was built for this bus
    PciArbiterConstructor(FdoExtension, PciArb_BusNumber, Interface as PARBITER_INTERFACE)
}
