#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::guids::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{INT_ROUTE_INTERFACE_STANDARD, PDEVICE_OBJECT, PINTERFACE};
use ntkrnl_sys::ddk::RtlAssert;
use ntkrnl_sys::driver_ddk::DbgPrint;
use ntkrnl_sys::ke::DbgBreakPoint;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut PciLegacyDeviceHead: PPCI_LEGACY_DEVICE =
    ::core::ptr::null::<_PCI_LEGACY_DEVICE>() as *mut _PCI_LEGACY_DEVICE;

#[no_mangle]
pub static mut PciRoutingInterface: PCI_INTERFACE = unsafe {
    _PCI_INTERFACE {
        InterfaceType: &raw const GUID_INT_ROUTE_INTERFACE_STANDARD,
        MinSize: ::core::mem::size_of::<INT_ROUTE_INTERFACE_STANDARD>() as USHORT,
        MinVersion: PCI_INT_ROUTE_INTRF_STANDARD_VER as USHORT,
        MaxVersion: PCI_INT_ROUTE_INTRF_STANDARD_VER as USHORT,
        Flags: PCI_INTERFACE_FDO as USHORT,
        ReferenceCount: 0 as LONG,
        Signature: PciInterface_IntRouteHandler,
        Constructor: Some(core::mem::transmute(routeintrf_Constructor as *const ())),
        Initializer: Some(core::mem::transmute(routeintrf_Initializer as *const ())),
    }
};
#[no_mangle]
pub unsafe extern "system" fn routeintrf_Initializer(Instance: PVOID) -> NTSTATUS {
    _ = Instance; // UNREFERENCED_PARAMETER
    if 0 as ::core::ffi::c_int == 0 {
        RtlAssert(
            b"FALSE\0".as_ptr() as *mut ::core::ffi::c_void,
            b"drivers/bus/pcix/intrface/routintf.c\0"
                .as_ptr() as *mut ::core::ffi::c_void,
            41u32,
            b"PCI routeintrf_Initializer, unexpected call.\n\0".as_ptr() as *mut i8,
        );
    }
    STATUS_UNSUCCESSFUL
}
#[no_mangle]
pub unsafe extern "system" fn routeintrf_Constructor(
    DeviceExtension: *mut core::ffi::c_void,
    Instance: *mut core::ffi::c_void,
    InterfaceData: *mut core::ffi::c_void,
    Version: u16,
    Size: u16,
    Interface: *mut core::ffi::c_void,
) -> i32 {
    if Version as i32 != PCI_INT_ROUTE_INTRF_STANDARD_VER as i32 {
        return STATUS_NOINTERFACE;
    }
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"routeintrf_Constructor\0".as_ptr() as *const i8,
        b"drivers/bus/pcix/intrface/routintf.c\0".as_ptr() as *const i8,
        64i32,
    );
    DbgPrint(b"\0".as_ptr() as *const i8);
    DbgBreakPoint();
    return STATUS_NOT_IMPLEMENTED;
}
#[no_mangle]
pub unsafe extern "system" fn PciCacheLegacyDeviceRouting(
    mut DeviceObject: PDEVICE_OBJECT,
    mut BusNumber: ULONG,
    mut SlotNumber: ULONG,
    mut InterruptLine: UCHAR,
    mut InterruptPin: UCHAR,
    mut BaseClass: UCHAR,
    mut SubClass: UCHAR,
    mut PhysicalDeviceObject: PDEVICE_OBJECT,
    mut PdoExtension: PPCI_PDO_EXTENSION,
    mut pFoundDeviceObject: *mut PDEVICE_OBJECT,
) -> NTSTATUS {
    let mut Link: *mut PPCI_LEGACY_DEVICE;
    let mut LegacyDevice: PPCI_LEGACY_DEVICE;
    let mut FoundDeviceObject: PDEVICE_OBJECT;

    LegacyDevice = PciLegacyDeviceHead;
    Link = &raw mut PciLegacyDeviceHead;
    while !LegacyDevice.is_null() {
        if BusNumber == (*LegacyDevice).BusNumber && SlotNumber == (*LegacyDevice).SlotNumber {
            break;
        }
        if (*LegacyDevice).DeviceObject == DeviceObject {
            *Link = (*LegacyDevice).Next;
            ExFreePoolWithTag(LegacyDevice as PVOID, 0u32);
            LegacyDevice = *Link;
        } else {
            Link = &raw mut (*LegacyDevice).Next;
            LegacyDevice = (*LegacyDevice).Next;
        }
    }

    if LegacyDevice.is_null() {
        LegacyDevice = ExAllocatePoolWithTag(
            PagedPool,
            ::core::mem::size_of::<PCI_LEGACY_DEVICE>() as SIZE_T,
            0x50636952u32,
        ) as PPCI_LEGACY_DEVICE;
        if LegacyDevice.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        core::ptr::write_bytes(
            LegacyDevice as *mut u8,
            0u8,
            ::core::mem::size_of::<PCI_LEGACY_DEVICE>(),
        );
        (*LegacyDevice).BusNumber = BusNumber;
        (*LegacyDevice).SlotNumber = SlotNumber;
        (*LegacyDevice).InterruptLine = InterruptLine;
        (*LegacyDevice).InterruptPin = InterruptPin;
        (*LegacyDevice).BaseClass = BaseClass;
        (*LegacyDevice).SubClass = SubClass;
        (*LegacyDevice).PhysicalDeviceObject = PhysicalDeviceObject;
        (*LegacyDevice).DeviceObject = DeviceObject;
        (*LegacyDevice).PdoExtension = PdoExtension;
        (*LegacyDevice).Next = PciLegacyDeviceHead;
        PciLegacyDeviceHead = LegacyDevice;
    }

    FoundDeviceObject = (*LegacyDevice).DeviceObject;
    if FoundDeviceObject == DeviceObject {
        if !pFoundDeviceObject.is_null() {
            *pFoundDeviceObject = DeviceObject;
        }
        return STATUS_SUCCESS;
    }

    (*LegacyDevice).DeviceObject = DeviceObject;
    if !pFoundDeviceObject.is_null() {
        *pFoundDeviceObject = FoundDeviceObject;
    }
    STATUS_SUCCESS
}
