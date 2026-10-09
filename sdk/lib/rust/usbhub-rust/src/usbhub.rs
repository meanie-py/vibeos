use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{ExFreePool, _KWAIT_REASON_Executive, _IO_STATUS_BLOCK, IoAcquireCancelSpinLock, IoAllocateIrp, IoAllocateWorkItem, IoAttachDeviceToDeviceStack, IoBuildDeviceIoControlRequest, IoCancelIrp, IoCreateDevice, IoDeleteDevice, IoFreeIrp, IoFreeWorkItem, IoGetCurrentIrpStackLocation, IoGetNextIrpStackLocation, IoInitializeIrp, IoInvalidateDeviceRelations, IoOpenDeviceRegistryKey, IoQueueWorkItem, IoReleaseCancelSpinLock, IoSetCancelRoutine, IoSetCompletionRoutine, IoSkipCurrentIrpStackLocation, KEVENT, KeCancelTimer, KeDelayExecutionThread, KeInitializeDpc, KeInitializeEvent, KeInitializeSemaphore, KeInitializeSpinLock, KeInitializeTimer, KeQueryTimeIncrement, KeReleaseSemaphore, KeSetTimer, KeWaitForSingleObject, LARGE_INTEGER, LIST_ENTRY, LPWSTR, _EVENT_TYPE_NotificationEvent, PDEVICE_OBJECT, PDEVICE_POWER_STATE, PDRIVER_OBJECT, PIO_STACK_LOCATION, PIO_STATUS_BLOCK, PIO_WORKITEM, PIRP, POWER_STATE, PRKEVENT, PUNICODE_STRING, RTL_QUERY_REGISTRY_TABLE, RtlQueryRegistryValues, _KWAIT_REASON_Suspended};
// Enum-member aliases (bindgen prefixes them `_ENUM_Member`):
use ntkrnl_sys::ddk::_DEVICE_RELATION_TYPE_BusRelations as BusRelations;
use ntkrnl_sys::ddk::_WORK_QUEUE_TYPE_DelayedWorkQueue as DelayedWorkQueue;
use ntkrnl_sys::ddk::_DEVICE_POWER_STATE_PowerDeviceD0 as PowerDeviceD0;
use ntkrnl_sys::ddk::_DEVICE_POWER_STATE_PowerDeviceD2 as PowerDeviceD2;
use ntkrnl_sys::ddk::_DEVICE_POWER_STATE_PowerDeviceD3 as PowerDeviceD3;
use ntkrnl_sys::ddk::_SYSTEM_POWER_STATE_PowerSystemMaximum as PowerSystemMaximum;
use ntkrnl_sys::ddk::_SYSTEM_POWER_STATE_PowerSystemWorking as PowerSystemWorking;
// Type aliases (bindgen `_X` → public alias):
use ntkrnl_sys::ddk::WORK_QUEUE_TYPE;
use ntkrnl_sys::ddk::SYSTEM_POWER_STATE;
use crate::types::USB_DEVICE_TYPE;
use crate::types::_USB_DEVICE_TYPE_Usb11Device as Usb11Device;
use crate::types::_USB_DEVICE_TYPE_Usb20Device as Usb20Device;
// Driver-type aliases the translator named without the bindgen `_`/`P` decoration:
use crate::types::_USB_HUB_STATUS_AND_CHANGE as USB_HUB_STATUS_AND_CHANGE;
pub type PUSB_IDLE_CALLBACK_INFO = *mut crate::types::_USB_IDLE_CALLBACK_INFO;
use ntkrnl_sys::ddk::PRKSEMAPHORE;
// ExInterlockedRemoveHeadList is a C macro aliasing ExfInterlockedRemoveHeadList (exfuncs.h:82):
// ExInterlockedRemoveHeadList is a fastcall macro→ExfInterlockedRemoveHeadList on i386, but a
// plain export on amd64/arm64 — cfg-gate so all 3 arches resolve.
#[cfg(target_arch = "x86")]
use ntkrnl_sys::ddk::ExfInterlockedRemoveHeadList as ExInterlockedRemoveHeadList;
#[cfg(not(target_arch = "x86"))]
use ntkrnl_sys::ddk::ExInterlockedRemoveHeadList;
use core::ffi::c_void;

extern "C" {
    fn IoSizeOfIrp(StackSize: i8) -> u16;
    fn DbgPrint(Format: *const i8, ...) -> u32;
    fn DPRINT(fmt: *const i8, ...);
    fn RtlStringCbPrintfW(pszDest: *mut u16, cbDest: usize, pszFormat: *const u16, ...) -> NTSTATUS;
}
unsafe extern "system" {
    fn USBD_ParseConfigurationDescriptorEx( ConfigurationDescriptor: PUSB_CONFIGURATION_DESCRIPTOR, StartPosition: *mut core::ffi::c_void, InterfaceNumber: i32, AlternateSetting: i32, InterfaceClass: i32, InterfaceSubClass: i32, InterfaceProtocol: i32, ) -> PUSB_INTERFACE_DESCRIPTOR;
}
unsafe extern "system" {
    fn USBH_PdoInternalControl(PortExtension: PUSBHUB_PORT_PDO_EXTENSION, Irp: PIRP) -> NTSTATUS;
}
unsafe extern "system" {
    fn USBH_PdoPower(PortExtension: PUSBHUB_PORT_PDO_EXTENSION, Irp: PIRP, MinorFunction: UCHAR) -> NTSTATUS;
}
unsafe extern "system" {
    fn USBH_HubSetD0(HubExtension: PUSBHUB_FDO_EXTENSION) -> NTSTATUS;
}
unsafe extern "system" {
    fn USBHUB_DumpingDeviceDescriptor(DeviceDescriptor: *mut USB_DEVICE_DESCRIPTOR);
}
unsafe extern "system" {
    fn USBHUB_DumpingConfiguration(ConfigDescriptor: PUSB_CONFIGURATION_DESCRIPTOR);
}
unsafe extern "system" {
    fn USBH_HubStartESDRecovery(HubExtension: PUSBHUB_FDO_EXTENSION,) -> NTSTATUS;
}

// NTAPI (stdcall on i386) kernel exports — MUST be extern "system" so the `_Name@N` decoration
// matches the ntoskrnl import (Zw*/Rtl* are NTAPI; USBH_PdoPnP is now defined extern "system"
// in pnp.rs). Declaring these extern "C" emitted undecorated `_Name`, undefined at link.
extern "system" {
    fn ZwSetValueKey( KeyHandle: HANDLE, ValueName: PUNICODE_STRING, TitleIndex: ULONG, Type: ULONG, Data: PVOID, DataSize: ULONG, ) -> NTSTATUS;
    fn ZwClose(Handle: HANDLE) -> NTSTATUS;
    fn RtlInitUnicodeString(DestinationString: PUNICODE_STRING, SourceString: PCWSTR);
    fn RtlCompareMemory(Source1: *const core::ffi::c_void, Source2: *const core::ffi::c_void, Length: SIZE_T) -> SIZE_T;
    fn RtlIntegerToUnicodeString(Value: ULONG, Base: ULONG, String: PUNICODE_STRING) -> NTSTATUS;
    fn USBH_PdoPnP(PortExtension: PUSBHUB_PORT_PDO_EXTENSION, Irp: PIRP, MinorFunction: UCHAR, ShouldCompleteIrp: *mut BOOLEAN) -> NTSTATUS;
    fn InterlockedExchange(Target: *mut LONG, Value: LONG) -> LONG;
    fn PoRequestPowerIrp(
        DeviceObject: PDEVICE_OBJECT,
        MinorFunction: UCHAR,
        PowerState: POWER_STATE,
        CompletionFunction: ntkrnl_sys::ddk::PREQUEST_POWER_COMPLETE,
        Context: PVOID,
        Irp: *mut PIRP,
    ) -> NTSTATUS;
}

// ---- C macros (faithful inline translations) ----
#[inline]
fn USBD_SUCCESS(Status: USBD_STATUS) -> BOOL {
    (Status >= 0) as BOOL // usb.h:167
}
#[inline]
fn USBD_ERROR(Status: USBD_STATUS) -> BOOL {
    (Status < 0) as BOOL // usb.h:169
}
#[inline]
fn MAKELANGID(p: u16, s: u16) -> u16 {
    ((s << 10) | p) as u16 // ntbasedef.h:939
}

// Coerce an `extern "system"` fn item to a bindgen NTAPI callback slot
// (`Option<unsafe extern "stdcall" fn ...>`). On i386 "system"==stdcall (and on
// amd64/arm64 both are the C convention), so the fn-pointer bit pattern is ABI-identical;
// the slot type is inferred from the assignment/argument context.
macro_rules! cb {
    ($f:expr) => {
        core::mem::transmute($f as usize)
    };
}

// Bindgen-union accessor helpers for the nested USB port-status unions.
// _USB_PORT_STATUS_AND_CHANGE { union { ULONG AsUlong32; struct { USB_PORT_STATUS PortStatus;
//   USB_PORT_CHANGE PortChange; } } }, where USB_PORT_STATUS is itself a union over a bitfield.
#[inline]
unsafe fn psc_status20(
    psc: *const crate::types::_USB_PORT_STATUS_AND_CHANGE,
) -> *const crate::types::_USB_20_PORT_STATUS__bindgen_ty_1 {
    let inner = (*psc).__bindgen_anon_1.as_ref();
    inner.PortStatus.Usb20PortStatus.as_ref().__bindgen_anon_1.as_ref()
}
#[inline]
unsafe fn psc_status20_mut(
    psc: *mut crate::types::_USB_PORT_STATUS_AND_CHANGE,
) -> *mut crate::types::_USB_20_PORT_STATUS__bindgen_ty_1 {
    let inner = (*psc).__bindgen_anon_1.as_mut();
    inner.PortStatus.Usb20PortStatus.as_mut().__bindgen_anon_1.as_mut()
}
#[inline]
unsafe fn psc_set_asulong32(psc: *mut crate::types::_USB_PORT_STATUS_AND_CHANGE, v: ULONG) {
    *(*psc).AsUlong32.as_mut() = v;
}
#[inline]
unsafe fn psc_get_asulong32(psc: *const crate::types::_USB_PORT_STATUS_AND_CHANGE) -> ULONG {
    *(*psc).AsUlong32.as_ref()
}
// Bitfield struct of a bare USB_PORT_STATUS / USB_20_PORT_STATUS.
#[inline]
unsafe fn ps20_bits(
    ps: *const crate::types::_USB_20_PORT_STATUS,
) -> *const crate::types::_USB_20_PORT_STATUS__bindgen_ty_1 {
    (*ps).__bindgen_anon_1.as_ref()
}
// Bare USB_20_PORT_CHANGE bitfield struct.
#[inline]
unsafe fn pc20_bits(
    pc: *const crate::types::_USB_20_PORT_CHANGE,
) -> *const crate::types::_USB_20_PORT_CHANGE__bindgen_ty_1 {
    (*pc).__bindgen_anon_1.as_ref()
}
// PortChange.Usb20PortChange bitfield, from a _USB_PORT_STATUS_AND_CHANGE.
#[inline]
unsafe fn psc_change20(
    psc: *const crate::types::_USB_PORT_STATUS_AND_CHANGE,
) -> *const crate::types::_USB_20_PORT_CHANGE__bindgen_ty_1 {
    let inner = (*psc).__bindgen_anon_1.as_ref();
    inner.PortChange.Usb20PortChange.as_ref().__bindgen_anon_1.as_ref()
}
// _USB_HUB_STATUS_AND_CHANGE accessors.
#[inline]
unsafe fn hsc_get_asulong32(hsc: *const crate::types::_USB_HUB_STATUS_AND_CHANGE) -> ULONG {
    *(*hsc).AsUlong32.as_ref()
}
#[inline]
unsafe fn hsc_change_bits(
    hsc: *const crate::types::_USB_HUB_STATUS_AND_CHANGE,
) -> *const crate::types::_USB_HUB_CHANGE__bindgen_ty_1 {
    (*hsc).__bindgen_anon_1.as_ref().HubChange.__bindgen_anon_1.as_ref()
}
#[inline]
unsafe fn hsc_status_bits(
    hsc: *const crate::types::_USB_HUB_STATUS_AND_CHANGE,
) -> *const crate::types::_USB_HUB_STATUS__bindgen_ty_1 {
    (*hsc).__bindgen_anon_1.as_ref().HubStatus.__bindgen_anon_1.as_ref()
}

// _BM_REQUEST_TYPE is a bindgen union (raw byte `B` + bitfield view `_BM`). The bitfield
// view isn't generated as a named union member, so reinterpret the (1-byte) struct as the
// bitfield struct to reach the bindgen setters. Layout-identical (both 1 byte).
trait BmReqExt {
    fn set_B(&mut self, v: UCHAR);
    fn set_Recipient<V: Into<u64>>(&mut self, v: V);
    fn set_Type<V: Into<u64>>(&mut self, v: V);
    fn set_Dir<V: Into<u64>>(&mut self, v: V);
}
impl BmReqExt for crate::types::_BM_REQUEST_TYPE {
    #[inline]
    fn set_B(&mut self, v: UCHAR) {
        unsafe { *self.B.as_mut() = v; }
    }
    #[inline]
    fn set_Recipient<V: Into<u64>>(&mut self, v: V) {
        let bm = self as *mut _ as *mut crate::types::_BM_REQUEST_TYPE__BM;
        unsafe { crate::types::_BM_REQUEST_TYPE__BM::set_Recipient_raw(bm, v.into() as UCHAR) }
    }
    #[inline]
    fn set_Type<V: Into<u64>>(&mut self, v: V) {
        let bm = self as *mut _ as *mut crate::types::_BM_REQUEST_TYPE__BM;
        unsafe { crate::types::_BM_REQUEST_TYPE__BM::set_Type_raw(bm, v.into() as UCHAR) }
    }
    #[inline]
    fn set_Dir<V: Into<u64>>(&mut self, v: V) {
        let bm = self as *mut _ as *mut crate::types::_BM_REQUEST_TYPE__BM;
        unsafe { crate::types::_BM_REQUEST_TYPE__BM::set_Dir_raw(bm, v.into() as UCHAR) }
    }
}

// ---- file-scope preamble (translated) ----
pub const NDEBUG: () = ();

pub const NDEBUG_USBHUB_SCE: () = ();

pub const NDEBUG_USBHUB_PNP: () = ();

// #[no_mangle] so the cross-module `_GenericUSBDeviceString` reference (pnp.rs) resolves to
// this single definition (usbhub.c global `PWSTR GenericUSBDeviceString = NULL`).
#[no_mangle]
pub static mut GenericUSBDeviceString: PWSTR = core::ptr::null_mut();

const PLUGPLAY_REGKEY_DEVICE: ULONG = 1;

const IOCTL_STORAGE_GET_MEDIA_SERIAL_NUMBER: u32 = 0x002D0C10;
const IOCTL_KS_PROPERTY: u32 = 0x002F0003;

// ---- #define IOCTLs (CTL_CODE(FILE_DEVICE_USB=0x22, fn, METHOD_NEITHER=3, FILE_ANY_ACCESS=0)) ----
// usbioctl.h / usbiodef.h. Verified: USB_GET_PORT_STATUS=4, USB_ENABLE_PORT=5,
// USB_IDLE_NOTIFICATION=9, USB_GET_DEVICE_HANDLE=268.
const IOCTL_INTERNAL_USB_GET_PORT_STATUS: u32 = 0x00220013;
const IOCTL_INTERNAL_USB_ENABLE_PORT: u32 = 0x00220017;
const IOCTL_INTERNAL_USB_SUBMIT_IDLE_NOTIFICATION: u32 = 0x00220027;
const IOCTL_INTERNAL_USB_GET_DEVICE_HANDLE: u32 = 0x00220433;
const FILE_DEVICE_USB: u32 = 0x00000022; // usbiodef.h: == FILE_DEVICE_UNKNOWN (iotypes.h:84)

// ---- #define NTSTATUS (ntstatus.h) ----
const STATUS_INVALID_DEVICE_STATE: NTSTATUS = 0xC0000184u32 as i32; // ntstatus.h:715
const STATUS_POWER_STATE_INVALID: NTSTATUS = 0xC00002D3u32 as i32; // ntstatus.h:957
const STATUS_DEVICE_BUSY: NTSTATUS = 0x80000011u32 as i32; // ntstatus.h:277
const STATUS_DEVICE_REMOVED: NTSTATUS = 0xC00002B6u32 as i32; // ntstatus.h:934
const STATUS_NO_SUCH_DEVICE: NTSTATUS = 0xC000000Eu32 as i32; // ntstatus.h:344

// ---- #define misc ----
const USB_DEVICE_CLASS_MISCELLANEOUS: u8 = 0xEF; // usb200.h:310
const LANG_ENGLISH: u16 = 0x09; // ntbasedef.h:988
const SUBLANG_ENGLISH_US: u16 = 0x01; // ntbasedef.h:1174

// ---- _USB_CONNECTION_STATUS enum members (usbioctl.h:248-249) ----
use crate::types::_USB_CONNECTION_STATUS_NoDeviceConnected as NoDeviceConnected;
use crate::types::_USB_CONNECTION_STATUS_DeviceConnected as DeviceConnected;

#[no_mangle]
pub unsafe extern "system" fn USBH_Wait(Milliseconds: ULONG) -> NTSTATUS {
    let mut Interval: LARGE_INTEGER = core::mem::zeroed();

    // DPRINT("USBH_Wait: Milliseconds - %x\n", Milliseconds);
    *(core::ptr::addr_of_mut!(Interval.QuadPart)) =
        -10000i64 * Milliseconds as i64 - ((KeQueryTimeIncrement() as u64 - 1) as i64);
    return KeDelayExecutionThread(0i32 as i8, 0, core::ptr::addr_of_mut!(Interval));
}

#[no_mangle]
pub unsafe extern "system" fn USBH_GetConfigValue(
    ValueName: PWSTR,
    ValueType: ULONG,
    ValueData: PVOID,
    ValueLength: ULONG,
    Context: PVOID,
    EntryContext: PVOID,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    // DPRINT("USBHUB_GetConfigValue: ... \n");

    if ValueType == REG_BINARY {
        *(EntryContext as *mut UCHAR) = *(ValueData as *mut UCHAR);
    } else if ValueType == REG_DWORD {
        *(EntryContext as *mut ULONG) = *(ValueData as *mut ULONG);
    } else {
        Status = STATUS_INVALID_PARAMETER;
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_CompleteIrp(Irp: PIRP, CompleteStatus: NTSTATUS) {
    if CompleteStatus != STATUS_SUCCESS {
        // DPRINT1("USBH_CompleteIrp: Irp - %p, CompleteStatus - %X\n", Irp, CompleteStatus);
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = CompleteStatus;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
}

#[no_mangle]
pub unsafe extern "system" fn USBH_PassIrp(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    DPRINT_PNP!(
        b"USBH_PassIrp: DeviceObject - %p, Irp - %p\n\0".as_ptr() as *const i8,
        DeviceObject,
        Irp,
    );

    IoSkipCurrentIrpStackLocation(Irp);
    return IoCallDriver(DeviceObject, Irp);
}

#[no_mangle]
pub unsafe extern "system" fn USBH_SyncIrpComplete(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    let HubTimeoutContext: PUSBHUB_URB_TIMEOUT_CONTEXT;
    let mut OldIrql: KIRQL = 0;
    let TimerCancelled: BOOLEAN;

    // DPRINT("USBH_SyncIrpComplete: ... \n");

    HubTimeoutContext = Context as PUSBHUB_URB_TIMEOUT_CONTEXT;

    KeAcquireSpinLock(
        core::ptr::addr_of_mut!((*HubTimeoutContext).UrbTimeoutSpinLock) as PKSPIN_LOCK,
        core::ptr::addr_of_mut!(OldIrql),
    );
    (*HubTimeoutContext).IsNormalCompleted = 1;
    TimerCancelled = KeCancelTimer(core::ptr::addr_of_mut!((*HubTimeoutContext).UrbTimeoutTimer));
    KeReleaseSpinLock(
        core::ptr::addr_of_mut!((*HubTimeoutContext).UrbTimeoutSpinLock) as PKSPIN_LOCK,
        OldIrql,
    );

    if TimerCancelled != 0 {
        KeSetEvent(core::ptr::addr_of_mut!((*HubTimeoutContext).UrbTimeoutEvent) as PVOID, EVENT_INCREMENT as LONG, 0);
    }

    return STATUS_SUCCESS;
}

// Faithful translation of USBH_UrbTimeoutDPC (usbhub.c:182). The translator left the
// tail of the file commented-out; this DPC is referenced live by KeInitializeDpc above.
pub unsafe extern "system" fn USBH_UrbTimeoutDPC(
    _Dpc: PKDPC,
    DeferredContext: PVOID,
    _SystemArgument1: PVOID,
    _SystemArgument2: PVOID,
) {
    let HubTimeoutContext: PUSBHUB_URB_TIMEOUT_CONTEXT;
    let mut OldIrql: KIRQL = 0;
    let IsCompleted: BOOL;

    // DPRINT("USBH_TimeoutDPC ... \n");

    HubTimeoutContext = DeferredContext as PUSBHUB_URB_TIMEOUT_CONTEXT;

    KeAcquireSpinLock(
        core::ptr::addr_of_mut!((*HubTimeoutContext).UrbTimeoutSpinLock) as PKSPIN_LOCK,
        core::ptr::addr_of_mut!(OldIrql),
    );
    IsCompleted = (*HubTimeoutContext).IsNormalCompleted;
    KeReleaseSpinLock(
        core::ptr::addr_of_mut!((*HubTimeoutContext).UrbTimeoutSpinLock) as PKSPIN_LOCK,
        OldIrql,
    );

    if IsCompleted == 0 {
        IoCancelIrp((*HubTimeoutContext).Irp);
    }

    KeSetEvent(core::ptr::addr_of_mut!((*HubTimeoutContext).UrbTimeoutEvent) as PVOID, EVENT_INCREMENT as LONG, 0);
}

#[no_mangle]
pub unsafe extern "system" fn IsBitSet(BitMapAddress: PUCHAR, Bit: USHORT) -> BOOLEAN {
    let mut IsSet: BOOLEAN;

    IsSet = ((*BitMapAddress.offset((Bit / 8) as isize) as i32 & (1 << (Bit & 7))) != 0) as BOOLEAN;
    // DPRINT("IsBitSet: Bit - %lX, IsSet - %x\n", Bit, IsSet);
    return IsSet;
}

#[no_mangle]
pub unsafe extern "system" fn PdoExt(DeviceObject: PDEVICE_OBJECT) -> PUSBHUB_PORT_PDO_EXTENSION {
    let mut PdoExtension: PVOID;

    // DPRINT("PdoExt: DeviceObject - %p\n", DeviceObject);

    if !DeviceObject.is_null() {
        PdoExtension = (*DeviceObject).DeviceExtension;
    } else {
        PdoExtension = core::ptr::null_mut();
    }

    return PdoExtension as PUSBHUB_PORT_PDO_EXTENSION;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_WriteFailReasonID(
    DeviceObject: PDEVICE_OBJECT,
    FailReason: ULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut KeyHandle: HANDLE = core::ptr::null_mut();

    let ValueNameBuffer: [u16; 13] = [
        0x0046, 0x0061, 0x0069, 0x006C, 0x0052, 0x0065, 0x0061, 0x0073, 0x006F, 0x006E, 0x0049,
        0x0044, 0x0000,
    ]; // L"FailReasonID"
    let mut ValueName: _UNICODE_STRING = _UNICODE_STRING {
        Length: (12 * core::mem::size_of::<u16>()) as u16,
        MaximumLength: (13 * core::mem::size_of::<u16>()) as u16,
        Buffer: ValueNameBuffer.as_ptr() as PWSTR,
    };

    // DPRINT("USBH_WriteFailReason: ID - %x\n", FailReason);

    Status = IoOpenDeviceRegistryKey(
        DeviceObject,
        PLUGPLAY_REGKEY_DEVICE,
        STANDARD_RIGHTS_ALL,
        core::ptr::addr_of_mut!(KeyHandle),
    );

    if NT_SUCCESS(Status) {
        ZwSetValueKey(
            KeyHandle,
            core::ptr::addr_of_mut!(ValueName),
            0,
            REG_DWORD,
            core::ptr::addr_of!(FailReason) as PVOID,
            core::mem::size_of::<ULONG>() as ULONG,
        );

        ZwClose(KeyHandle);
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_SyncSubmitUrb(
    DeviceObject: PDEVICE_OBJECT,
    Urb: PURB,
) -> NTSTATUS {
    let mut Event: KEVENT = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let Irp: PIRP;
    let IoStack: PIO_STACK_LOCATION;
    let HubTimeoutContext: PUSBHUB_URB_TIMEOUT_CONTEXT;
    let mut IsWaitTimeout: BOOLEAN = 0;
    let mut DueTime: LARGE_INTEGER = core::mem::zeroed();
    let Status: NTSTATUS;

    // DPRINT("USBH_SyncSubmitUrb: ... \n");

    (*Urb).__bindgen_anon_1.UrbHeader.as_mut().UsbdDeviceHandle = core::ptr::null_mut();

    KeInitializeEvent(core::ptr::addr_of_mut!(Event), _EVENT_TYPE_NotificationEvent, 0);

    Irp = IoBuildDeviceIoControlRequest(
        IOCTL_INTERNAL_USB_SUBMIT_URB,
        DeviceObject,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        0,
        1,
        core::ptr::addr_of_mut!(Event),
        core::ptr::addr_of_mut!(IoStatusBlock),
    );

    if Irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    IoStack = IoGetNextIrpStackLocation(Irp);
    (*IoStack).Parameters.Others.Argument1 = Urb as *mut core::ffi::c_void;

    HubTimeoutContext = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<USBHUB_URB_TIMEOUT_CONTEXT>(),
        USB_HUB_TAG,
    ) as PUSBHUB_URB_TIMEOUT_CONTEXT;

    if !HubTimeoutContext.is_null() {
        RtlZeroMemory(
            HubTimeoutContext as *mut core::ffi::c_void,
            core::mem::size_of::<USBHUB_URB_TIMEOUT_CONTEXT>(),
        );

        (*HubTimeoutContext).Irp = Irp;
        (*HubTimeoutContext).IsNormalCompleted = 0;

        KeInitializeEvent(
            core::ptr::addr_of_mut!((*HubTimeoutContext).UrbTimeoutEvent),
            _EVENT_TYPE_NotificationEvent,
            0,
        );

        KeInitializeSpinLock(core::ptr::addr_of_mut!((*HubTimeoutContext).UrbTimeoutSpinLock));
        KeInitializeTimer(core::ptr::addr_of_mut!((*HubTimeoutContext).UrbTimeoutTimer));

        KeInitializeDpc(
            core::ptr::addr_of_mut!((*HubTimeoutContext).UrbTimeoutDPC),
            cb!(USBH_UrbTimeoutDPC),
            HubTimeoutContext as *mut core::ffi::c_void,
        );

        DueTime.QuadPart = -5000i64 * 10000i64; // Timeout 5 sec.

        KeSetTimer(
            core::ptr::addr_of_mut!((*HubTimeoutContext).UrbTimeoutTimer),
            DueTime,
            core::ptr::addr_of_mut!((*HubTimeoutContext).UrbTimeoutDPC),
        );

        IoSetCompletionRoutine(
            Irp,
            cb!(USBH_SyncIrpComplete),
            HubTimeoutContext as *mut core::ffi::c_void,
            1,
            1,
            1,
        );

        IsWaitTimeout = 1;
    }

    Status = IoCallDriver(DeviceObject, Irp);

    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(Event) as *mut core::ffi::c_void,
            _KWAIT_REASON_Suspended,
            0i32 as i8,
            0,
            core::ptr::null_mut(),
        );
    } else {
        IoStatusBlock.__bindgen_anon_1.Status = Status;
    }

    if IsWaitTimeout != 0 {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!((*HubTimeoutContext).UrbTimeoutEvent) as *mut core::ffi::c_void,
            _KWAIT_REASON_Suspended,
            0i32 as i8,
            0,
            core::ptr::null_mut(),
        );

        ExFreePoolWithTag(HubTimeoutContext as *mut core::ffi::c_void, USB_HUB_TAG);
    }

    IoStatusBlock.__bindgen_anon_1.Status
}

#[no_mangle]
pub unsafe extern "system" fn USBH_FdoSyncSubmitUrb(
    FdoDevice: PDEVICE_OBJECT,
    Urb: PURB,
) -> NTSTATUS {
    let HubExtension: PUSBHUB_FDO_EXTENSION;

    // DPRINT("USBH_FdoSyncSubmitUrb: FdoDevice - %p, Urb - %p\n", FdoDevice, Urb);

    HubExtension = (*FdoDevice).DeviceExtension as PUSBHUB_FDO_EXTENSION;
    return USBH_SyncSubmitUrb((*HubExtension).LowerDevice, Urb);
}

#[no_mangle]
pub unsafe extern "system" fn USBH_Transact(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    TransferBuffer: PVOID,
    BufferLen: ULONG,
    IsDeviceToHost: BOOLEAN,
    Function: USHORT,
    RequestType: BM_REQUEST_TYPE,
    Request: UCHAR,
    RequestValue: USHORT,
    RequestIndex: USHORT,
) -> NTSTATUS {
    let Urb: *mut _URB_CONTROL_VENDOR_OR_CLASS_REQUEST;
    let TransferFlags: ULONG;
    let mut Buffer: PVOID = core::ptr::null_mut();
    let Length: ULONG;
    let Status: NTSTATUS;

    // DPRINT("USBH_Transact: ... \n");

    if BufferLen != 0 {
        Length = ALIGN_DOWN_BY(
            (BufferLen as usize + core::mem::size_of::<ULONG>()) as usize,
            core::mem::size_of::<ULONG>(),
        ) as ULONG;

        Buffer = ExAllocatePoolWithTag(NonPagedPool, Length as usize, USB_HUB_TAG);

        if Buffer.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        RtlZeroMemory(Buffer, Length as usize);
    }

    Urb = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<_URB_CONTROL_VENDOR_OR_CLASS_REQUEST>(),
        USB_HUB_TAG,
    ) as *mut _URB_CONTROL_VENDOR_OR_CLASS_REQUEST;

    if Urb.is_null() {
        if !Buffer.is_null() {
            ExFreePoolWithTag(Buffer, USB_HUB_TAG);
        }

        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlZeroMemory(
        Urb as PVOID,
        core::mem::size_of::<_URB_CONTROL_VENDOR_OR_CLASS_REQUEST>(),
    );

    if IsDeviceToHost != 0 {
        if BufferLen != 0 {
            RtlZeroMemory(TransferBuffer, BufferLen as usize);
        }

        TransferFlags = USBD_TRANSFER_DIRECTION_IN | USBD_SHORT_TRANSFER_OK;
    } else {
        if BufferLen != 0 {
            RtlCopyMemory(Buffer, TransferBuffer, BufferLen as usize);
        }

        TransferFlags = USBD_TRANSFER_DIRECTION_OUT;
    }

    (*Urb).Hdr.Length = core::mem::size_of::<_URB_CONTROL_VENDOR_OR_CLASS_REQUEST>() as USHORT;
    (*Urb).Hdr.Function = Function as USHORT;
    (*Urb).Hdr.UsbdDeviceHandle = core::ptr::null_mut();

    (*Urb).TransferFlags = TransferFlags;
    (*Urb).TransferBuffer = if BufferLen != 0 { Buffer } else { core::ptr::null_mut() };
    (*Urb).TransferBufferLength = BufferLen;
    (*Urb).TransferBufferMDL = core::ptr::null_mut();
    (*Urb).UrbLink = core::ptr::null_mut();

    (*Urb).RequestTypeReservedBits = *RequestType.B.as_ref();
    (*Urb).Request = Request as UCHAR;
    (*Urb).Value = RequestValue;
    (*Urb).Index = RequestIndex;

    Status = USBH_FdoSyncSubmitUrb((*HubExtension).Common.SelfDevice, Urb as PURB);

    if IsDeviceToHost != 0 && BufferLen != 0 {
        RtlCopyMemory(TransferBuffer, Buffer, BufferLen as usize);
    }

    if !Buffer.is_null() {
        ExFreePoolWithTag(Buffer, USB_HUB_TAG);
    }

    ExFreePoolWithTag(Urb as PVOID, USB_HUB_TAG);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_SyncResetPort(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    Port: USHORT,
) -> NTSTATUS {
    let mut PortStatus: USB_PORT_STATUS_AND_CHANGE = core::mem::zeroed();
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Timeout: LARGE_INTEGER = core::mem::zeroed();
    let mut ResetRetry: ULONG = 0;
    let mut Status: NTSTATUS;

    // DPRINT("USBH_SyncResetPort: Port - %x\n", Port);

    InterlockedIncrement(core::ptr::addr_of_mut!((*HubExtension).PendingRequestCount));

    KeWaitForSingleObject(
        core::ptr::addr_of_mut!((*HubExtension).HubPortSemaphore) as PVOID,
        _KWAIT_REASON_Executive,
        0i32 as i8,
        FALSE as u8,
        core::ptr::null_mut(),
    );

    Status = USBH_SyncGetPortStatus(
        HubExtension,
        Port,
        core::ptr::addr_of_mut!(PortStatus),
        core::mem::size_of::<USB_PORT_STATUS_AND_CHANGE>() as ULONG,
    );

    'exit: loop {
        if NT_SUCCESS(Status)
            && ((*psc_status20(core::ptr::addr_of!(PortStatus))).CurrentConnectStatus() == 0)
        {
            Status = STATUS_UNSUCCESSFUL;
            break 'exit;
        }

        (*HubExtension).HubFlags |= USBHUB_FDO_FLAG_RESET_PORT_LOCK;

        loop {
            let mut RequestType: BM_REQUEST_TYPE = core::mem::zeroed();

            KeInitializeEvent(
                core::ptr::addr_of_mut!(Event),
                _EVENT_TYPE_NotificationEvent,
                FALSE as u8,
            );

            InterlockedExchangePointer(
                core::ptr::addr_of_mut!((*HubExtension).pResetPortEvent) as *mut PVOID,
                core::ptr::addr_of_mut!(Event) as PVOID,
            );

            RequestType.set_B(0);
            RequestType.set_Recipient(BMREQUEST_TO_DEVICE);
            RequestType.set_Type(BMREQUEST_CLASS);
            RequestType.set_Dir(BMREQUEST_HOST_TO_DEVICE);

            Status = USBH_Transact(
                HubExtension,
                core::ptr::null_mut(),
                0,
                BMREQUEST_HOST_TO_DEVICE as u8,
                URB_FUNCTION_CLASS_OTHER as USHORT,
                RequestType,
                USB_REQUEST_SET_FEATURE as UCHAR,
                USBHUB_FEATURE_PORT_RESET as USHORT,
                Port,
            );

            Timeout.QuadPart = -5000i64 * 10000;

            if !NT_SUCCESS(Status) {
                InterlockedExchangePointer(
                    core::ptr::addr_of_mut!((*HubExtension).pResetPortEvent) as *mut PVOID,
                    core::ptr::null_mut(),
                );

                USBH_Wait(10);
                (*HubExtension).HubFlags &= !USBHUB_FDO_FLAG_RESET_PORT_LOCK;

                break 'exit;
            }

            Status = KeWaitForSingleObject(
                core::ptr::addr_of_mut!(Event) as PVOID,
                _KWAIT_REASON_Suspended,
                0i32 as i8,
                FALSE as u8,
                core::ptr::addr_of_mut!(Timeout),
            );

            if Status != STATUS_TIMEOUT {
                break;
            }

            Status = USBH_SyncGetPortStatus(
                HubExtension,
                Port,
                core::ptr::addr_of_mut!(PortStatus),
                core::mem::size_of::<USB_PORT_STATUS_AND_CHANGE>() as ULONG,
            );

            if !NT_SUCCESS(Status)
                || ((*psc_status20(core::ptr::addr_of!(PortStatus))).CurrentConnectStatus() == 0)
                || ResetRetry >= USBHUB_RESET_PORT_MAX_RETRY
            {
                InterlockedExchangePointer(
                    core::ptr::addr_of_mut!((*HubExtension).pResetPortEvent) as *mut PVOID,
                    core::ptr::null_mut(),
                );

                USBH_Wait(10);
                (*HubExtension).HubFlags &= !USBHUB_FDO_FLAG_RESET_PORT_LOCK;

                Status = STATUS_DEVICE_DATA_ERROR;
                break 'exit;
            }

            ResetRetry += 1;
        }

        Status = USBH_SyncGetPortStatus(
            HubExtension,
            Port,
            core::ptr::addr_of_mut!(PortStatus),
            core::mem::size_of::<USB_PORT_STATUS_AND_CHANGE>() as ULONG,
        );

        if ((*psc_status20(core::ptr::addr_of!(PortStatus))).CurrentConnectStatus() == 0)
            && NT_SUCCESS(Status)
            && ((*HubExtension).HubFlags & USBHUB_FDO_FLAG_USB20_HUB) != 0
        {
            Status = STATUS_DEVICE_DATA_ERROR;
        }

        USBH_Wait(10);
        (*HubExtension).HubFlags &= !USBHUB_FDO_FLAG_RESET_PORT_LOCK;

        break 'exit;
    }

    KeReleaseSemaphore(
        core::ptr::addr_of_mut!((*HubExtension).HubPortSemaphore) as PRKSEMAPHORE,
        LOW_REALTIME_PRIORITY as i32,
        1,
        FALSE as u8,
    );

    if InterlockedDecrement(core::ptr::addr_of_mut!((*HubExtension).PendingRequestCount)) == 0 {
        KeSetEvent(core::ptr::addr_of_mut!((*HubExtension).PendingRequestEvent) as PVOID, EVENT_INCREMENT as i32, FALSE as u8);
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_GetDeviceType(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    DeviceHandle: PUSB_DEVICE_HANDLE,
    OutDeviceType: *mut USB_DEVICE_TYPE,
) -> NTSTATUS {
    let QueryDeviceInformation: PUSB_BUSIFFN_GET_DEVICE_INFORMATION;
    let mut DeviceInfo: PUSB_DEVICE_INFORMATION_0;
    let mut DeviceInformationBufferLength: SIZE_T;
    let mut DeviceType: USB_DEVICE_TYPE = Usb11Device;
    let mut dummy: ULONG = 0;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    QueryDeviceInformation = (*HubExtension).BusInterface.QueryDeviceInformation;

    if QueryDeviceInformation.is_none() {
        return STATUS_NOT_IMPLEMENTED;
    }

    DeviceInformationBufferLength = core::mem::size_of::<USB_DEVICE_INFORMATION_0>() as SIZE_T;

    loop {
        DeviceInfo = ExAllocatePoolWithTag(
            PagedPool,
            DeviceInformationBufferLength,
            USB_HUB_TAG,
        ) as PUSB_DEVICE_INFORMATION_0;

        if DeviceInfo.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break;
        }

        RtlZeroMemory(DeviceInfo as *mut core::ffi::c_void, DeviceInformationBufferLength);

        (*DeviceInfo).InformationLevel = 0;

        Status = (QueryDeviceInformation.unwrap())(
            (*HubExtension).BusInterface.BusContext,
            DeviceHandle,
            DeviceInfo as PVOID,
            DeviceInformationBufferLength as u32,
            core::ptr::addr_of_mut!(dummy),
        );

        if Status != STATUS_BUFFER_TOO_SMALL {
            if NT_SUCCESS(Status) {
                DeviceType = (*DeviceInfo).DeviceType;
            }

            ExFreePoolWithTag(DeviceInfo as *mut core::ffi::c_void, USB_HUB_TAG);
            break;
        }

        DeviceInformationBufferLength = (*DeviceInfo).ActualLength as SIZE_T;
        ExFreePoolWithTag(DeviceInfo as *mut core::ffi::c_void, USB_HUB_TAG);
    }

    if !OutDeviceType.is_null() {
        *OutDeviceType = DeviceType;
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBHUB_GetExtendedHubInfo(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    HubInfoBuffer: PUSB_EXTHUB_INFORMATION_0,
) -> NTSTATUS {
    let GetExtendedHubInformation: PUSB_BUSIFFN_GET_EXTENDED_HUB_INFO;
    let mut dummy: ULONG = 0;

    GetExtendedHubInformation = (*HubExtension).BusInterface.GetExtendedHubInformation;

    return (GetExtendedHubInformation.unwrap())(
        (*HubExtension).BusInterface.BusContext,
        (*HubExtension).LowerPDO,
        HubInfoBuffer as PVOID,
        core::mem::size_of::<USB_EXTHUB_INFORMATION_0>() as ULONG,
        core::ptr::addr_of_mut!(dummy),
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBH_GetRootHubExtension(
    HubExtension: PUSBHUB_FDO_EXTENSION,
) -> PUSBHUB_FDO_EXTENSION {
    let mut Device: PDEVICE_OBJECT;
    let mut RootHubExtension: PUSBHUB_FDO_EXTENSION;

    // DPRINT("USBH_GetRootHubExtension: HubExtension - %p\n", HubExtension);

    RootHubExtension = HubExtension;

    if (*HubExtension).LowerPDO != (*HubExtension).RootHubPdo {
        Device = (*HubExtension).RootHubPdo;

        loop {
            Device = (*Device).AttachedDevice;
            if !((*Device).DriverObject
                != (*(*HubExtension).Common.SelfDevice).DriverObject)
            {
                break;
            }
        }

        RootHubExtension = (*Device).DeviceExtension as PUSBHUB_FDO_EXTENSION;
    }

    // DPRINT("USBH_GetRootHubExtension: RootHubExtension - %p\n", RootHubExtension);

    return RootHubExtension;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_SyncGetRootHubPdo(
    DeviceObject: PDEVICE_OBJECT,
    OutPdo1: *mut PDEVICE_OBJECT,
    OutPdo2: *mut PDEVICE_OBJECT,
) -> NTSTATUS {
    const IOCTL_INTERNAL_USB_GET_ROOTHUB_PDO: u32 = 0x0022000F; // FIXED 2026-07-05: translation invented wrong value; verified vs usbioctl.h

    let mut Event: KEVENT = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let Irp: PIRP;
    let IoStack: PIO_STACK_LOCATION;
    let Status: NTSTATUS;

    // DPRINT("USBH_SyncGetRootHubPdo: ... \n");

    KeInitializeEvent(core::ptr::addr_of_mut!(Event), _EVENT_TYPE_NotificationEvent, 0);

    Irp = IoBuildDeviceIoControlRequest(
        IOCTL_INTERNAL_USB_GET_ROOTHUB_PDO,
        DeviceObject,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        0,
        1,
        core::ptr::addr_of_mut!(Event),
        core::ptr::addr_of_mut!(IoStatusBlock),
    );

    if Irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    IoStack = IoGetNextIrpStackLocation(Irp);
    (*IoStack).Parameters.Others.Argument1 = OutPdo1 as *mut core::ffi::c_void;
    (*IoStack).Parameters.Others.Argument2 = OutPdo2 as *mut core::ffi::c_void;

    Status = IoCallDriver(DeviceObject, Irp);

    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(Event) as *mut core::ffi::c_void,
            _KWAIT_REASON_Suspended,
            0i32 as i8,
            0,
            core::ptr::null_mut(),
        );
    } else {
        IoStatusBlock.__bindgen_anon_1.Status = Status;
    }

    return IoStatusBlock.__bindgen_anon_1.Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_SyncGetHubCount(
    DeviceObject: PDEVICE_OBJECT,
    OutHubCount: PULONG,
) -> NTSTATUS {
    const IOCTL_INTERNAL_USB_GET_HUB_COUNT: u32 = 0x0022001B; // FIXED 2026-07-05: translation invented wrong value; verified vs usbioctl.h
    const Suspended: i32 = 5;

    let mut Event: KEVENT = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let Irp: PIRP;
    let IoStack: PIO_STACK_LOCATION;
    let Status: NTSTATUS;

    // DPRINT("USBH_SyncGetHubCount: *OutHubCount - %x\n", *OutHubCount);

    KeInitializeEvent(core::ptr::addr_of_mut!(Event), _EVENT_TYPE_NotificationEvent, 0);

    Irp = IoBuildDeviceIoControlRequest(
        IOCTL_INTERNAL_USB_GET_HUB_COUNT,
        DeviceObject,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        0,
        1,
        core::ptr::addr_of_mut!(Event),
        core::ptr::addr_of_mut!(IoStatusBlock),
    );

    if Irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    IoStack = IoGetNextIrpStackLocation(Irp);
    (*IoStack).Parameters.Others.Argument1 = OutHubCount as *mut core::ffi::c_void;

    Status = IoCallDriver(DeviceObject, Irp);

    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(Event) as *mut core::ffi::c_void,
            _KWAIT_REASON_Suspended,
            0i32 as i8,
            0,
            core::ptr::null_mut(),
        );
    } else {
        IoStatusBlock.__bindgen_anon_1.Status = Status;
    }

    return IoStatusBlock.__bindgen_anon_1.Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_SyncGetDeviceHandle(
    DeviceObject: PDEVICE_OBJECT,
) -> PUSB_DEVICE_HANDLE {
    let Irp: PIRP;
    let mut Event: KEVENT = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut DeviceHandle: PUSB_DEVICE_HANDLE = core::ptr::null_mut();
    let IoStack: PIO_STACK_LOCATION;

    // DPRINT("USBH_SyncGetDeviceHandle: ... \n");

    KeInitializeEvent(core::ptr::addr_of_mut!(Event), _EVENT_TYPE_NotificationEvent, 0);

    Irp = IoBuildDeviceIoControlRequest(
        IOCTL_INTERNAL_USB_GET_DEVICE_HANDLE,
        DeviceObject,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        0,
        1,
        core::ptr::addr_of_mut!(Event),
        core::ptr::addr_of_mut!(IoStatusBlock),
    );

    if Irp.is_null() {
        // DPRINT1("USBH_SyncGetDeviceHandle: Irp - NULL!\n");
        return core::ptr::null_mut();
    }

    IoStack = IoGetNextIrpStackLocation(Irp);
    (*IoStack).Parameters.Others.Argument1 = core::ptr::addr_of_mut!(DeviceHandle) as *mut core::ffi::c_void;

    if IoCallDriver(DeviceObject, Irp) == STATUS_PENDING {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(Event) as *mut core::ffi::c_void,
            _KWAIT_REASON_Suspended,
            0i32 as i8,
            0,
            core::ptr::null_mut(),
        );
    }

    return DeviceHandle;
}

// Note: `IOCTL_INTERNAL_USB_GET_DEVICE_HANDLE`, `_KWAIT_REASON_Suspended`, and `PUSB_DEVICE_HANDLE` are referenced by name (caller collects definitions). If `_KWAIT_REASON_Suspended` is not available it should be defined as the appropriate `KWAIT_REASON` variant value.

#[no_mangle]
pub unsafe extern "system" fn USBH_GetDeviceDescriptor(
    DeviceObject: PDEVICE_OBJECT,
    HubDeviceDescriptor: PUSB_DEVICE_DESCRIPTOR,
) -> NTSTATUS {
    const USB_HUB_TAG: u32 = 0x42554855; // 'USHB'
    const URB_FUNCTION_GET_DESCRIPTOR_FROM_DEVICE: u16 = 0x000B;
    const USB_DEVICE_DESCRIPTOR_TYPE: u8 = 0x01;

    let Urb: *mut _URB_CONTROL_DESCRIPTOR_REQUEST;
    let Status: NTSTATUS;

    // DPRINT("USBH_GetDeviceDescriptor: ... \n");

    Urb = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<_URB_CONTROL_DESCRIPTOR_REQUEST>(),
        USB_HUB_TAG,
    ) as *mut _URB_CONTROL_DESCRIPTOR_REQUEST;

    if Urb.is_null() {
        // DPRINT1("USBH_SyncGetDeviceHandle: Urb - NULL!\n");
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlZeroMemory(
        Urb as *mut core::ffi::c_void,
        core::mem::size_of::<_URB_CONTROL_DESCRIPTOR_REQUEST>(),
    );

    (*Urb).Hdr.Function = URB_FUNCTION_GET_DESCRIPTOR_FROM_DEVICE as USHORT;
    (*Urb).Hdr.Length = core::mem::size_of::<_URB_CONTROL_DESCRIPTOR_REQUEST>() as u16;

    (*Urb).TransferBufferLength = core::mem::size_of::<USB_DEVICE_DESCRIPTOR>() as u32;
    (*Urb).TransferBuffer = HubDeviceDescriptor as *mut core::ffi::c_void;
    (*Urb).DescriptorType = USB_DEVICE_DESCRIPTOR_TYPE as UCHAR;

    Status = USBH_FdoSyncSubmitUrb(DeviceObject, Urb as PURB);

    ExFreePoolWithTag(Urb as *mut core::ffi::c_void, USB_HUB_TAG);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_SyncGetDeviceConfigurationDescriptor(
    DeviceObject: PDEVICE_OBJECT,
    ConfigDescriptor: PUSB_CONFIGURATION_DESCRIPTOR,
    NumberOfBytes: ULONG,
    OutLength: PULONG,
) -> NTSTATUS {
    let DeviceExtension: PCOMMON_DEVICE_EXTENSION;
    let Urb: *mut _URB_CONTROL_DESCRIPTOR_REQUEST;
    let Status: NTSTATUS;

    DeviceExtension = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;

    if !OutLength.is_null() {
        *OutLength = 0;
    }

    Urb = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<_URB_CONTROL_DESCRIPTOR_REQUEST>(),
        USB_HUB_TAG,
    ) as *mut _URB_CONTROL_DESCRIPTOR_REQUEST;

    if Urb.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlZeroMemory(
        Urb as *mut core::ffi::c_void,
        core::mem::size_of::<_URB_CONTROL_DESCRIPTOR_REQUEST>(),
    );

    (*Urb).Hdr.Function = URB_FUNCTION_GET_DESCRIPTOR_FROM_DEVICE as u16;
    (*Urb).Hdr.Length = core::mem::size_of::<_URB_CONTROL_DESCRIPTOR_REQUEST>() as u16;

    (*Urb).TransferBufferLength = NumberOfBytes;
    (*Urb).TransferBuffer = ConfigDescriptor as *mut core::ffi::c_void;
    (*Urb).DescriptorType = USB_CONFIGURATION_DESCRIPTOR_TYPE as u8;

    if (*DeviceExtension).ExtensionType == USBH_EXTENSION_TYPE_HUB
        || (*DeviceExtension).ExtensionType == USBH_EXTENSION_TYPE_PARENT
    {
        Status = USBH_FdoSyncSubmitUrb(DeviceObject, Urb as PURB);
    } else {
        Status = USBH_SyncSubmitUrb(DeviceObject, Urb as PURB);
    }

    if !OutLength.is_null() {
        *OutLength = (*Urb).TransferBufferLength;
    }

    if !Urb.is_null() {
        ExFreePoolWithTag(Urb as *mut core::ffi::c_void, USB_HUB_TAG);
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_GetConfigurationDescriptor(
    DeviceObject: PDEVICE_OBJECT,
    OutDescriptor: *mut PUSB_CONFIGURATION_DESCRIPTOR,
) -> NTSTATUS {
    const MAXUCHAR: SIZE_T = 0xff;
    const USB_HUB_TAG: ULONG = 0x42554855; // 'USHB' approximation

    let mut ConfigDescriptor: PUSB_CONFIGURATION_DESCRIPTOR;
    let mut ReturnedLen: ULONG = 0;
    let mut DescriptorLen: SIZE_T;
    let mut Status: NTSTATUS;

    DescriptorLen = MAXUCHAR;

    loop {
        ConfigDescriptor = ExAllocatePoolWithTag(
            NonPagedPool,
            DescriptorLen,
            USB_HUB_TAG,
        ) as PUSB_CONFIGURATION_DESCRIPTOR;

        if ConfigDescriptor.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break;
        }

        Status = USBH_SyncGetDeviceConfigurationDescriptor(
            DeviceObject,
            ConfigDescriptor,
            DescriptorLen as ULONG,
            core::ptr::addr_of_mut!(ReturnedLen),
        );

        if (ReturnedLen as usize) < core::mem::size_of::<USB_CONFIGURATION_DESCRIPTOR>() {
            Status = STATUS_DEVICE_DATA_ERROR;
        }

        if !NT_SUCCESS(Status) {
            break;
        }

        *OutDescriptor = ConfigDescriptor;

        if (*ConfigDescriptor).wTotalLength as SIZE_T <= DescriptorLen {
            break;
        }

        DescriptorLen = (*ConfigDescriptor).wTotalLength as SIZE_T;

        ExFreePool(ConfigDescriptor as *mut core::ffi::c_void);
        *OutDescriptor = core::ptr::null_mut();
    }

    if NT_SUCCESS(Status) {
        if (ReturnedLen as u32) < (*ConfigDescriptor).wTotalLength as u32 {
            Status = STATUS_DEVICE_DATA_ERROR;
        }
    } else {
        if !ConfigDescriptor.is_null() {
            ExFreePool(ConfigDescriptor as *mut core::ffi::c_void);
        }

        *OutDescriptor = core::ptr::null_mut();
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn USBH_SyncGetHubDescriptor(
    HubExtension: PUSBHUB_FDO_EXTENSION,
) -> NTSTATUS {
    let mut ExtendedHubInfo: PUSB_EXTHUB_INFORMATION_0;
    let mut NumberPorts: ULONG;
    let mut PortData: PUSBHUB_PORT_DATA;
    let mut RequestValue: USHORT;
    let mut NumberOfBytes: ULONG;
    let mut Status: NTSTATUS;
    let mut HubDescriptor: PUSB_HUB_DESCRIPTOR = core::ptr::null_mut();
    let mut ix: ULONG;
    let mut Retry: ULONG;

    // DPRINT("USBH_SyncGetHubDescriptor: ... \n");

    ExtendedHubInfo = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<USB_EXTHUB_INFORMATION_0>() as ULONG as usize,
        USB_HUB_TAG,
    ) as PUSB_EXTHUB_INFORMATION_0;

    'error_exit: loop {
        if ExtendedHubInfo.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'error_exit;
        }

        RtlZeroMemory(
            ExtendedHubInfo as PVOID,
            core::mem::size_of::<USB_EXTHUB_INFORMATION_0>(),
        );

        Status = USBHUB_GetExtendedHubInfo(HubExtension, ExtendedHubInfo);

        if !NT_SUCCESS(Status) {
            ExFreePoolWithTag(ExtendedHubInfo as PVOID, USB_HUB_TAG);
            ExtendedHubInfo = core::ptr::null_mut();
        }

        NumberOfBytes = core::mem::size_of::<USB_HUB_DESCRIPTOR>() as ULONG;

        HubDescriptor = ExAllocatePoolWithTag(NonPagedPool, NumberOfBytes as usize, USB_HUB_TAG)
            as PUSB_HUB_DESCRIPTOR;

        if HubDescriptor.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'error_exit;
        }

        RtlZeroMemory(HubDescriptor as PVOID, NumberOfBytes as usize);

        RequestValue = 0;
        Retry = 0;

        loop {
            while Retry <= 5 {
                let mut RequestType: BM_REQUEST_TYPE = core::mem::zeroed();

                RequestType.set_B(0);
                RequestType.set_Recipient(BMREQUEST_TO_DEVICE);
                RequestType.set_Type(BMREQUEST_STANDARD);
                RequestType.set_Dir(BMREQUEST_DEVICE_TO_HOST);

                Status = USBH_Transact(
                    HubExtension,
                    HubDescriptor as PVOID,
                    NumberOfBytes,
                    BMREQUEST_DEVICE_TO_HOST as BOOLEAN,
                    URB_FUNCTION_CLASS_DEVICE as USHORT,
                    RequestType,
                    USB_REQUEST_GET_DESCRIPTOR as UCHAR,
                    RequestValue,
                    0,
                );

                if NT_SUCCESS(Status) {
                    break;
                }

                RequestValue = 0x2900; // Hub DescriptorType - 0x29

                Retry += 1;
            }

            if (*HubDescriptor).bDescriptorLength as ULONG <= NumberOfBytes {
                break;
            }

            NumberOfBytes = (*HubDescriptor).bDescriptorLength as ULONG;
            ExFreePoolWithTag(HubDescriptor as PVOID, USB_HUB_TAG);

            if Retry >= 5 {
                Status = STATUS_DEVICE_DATA_ERROR;
                HubDescriptor = core::ptr::null_mut();
                break 'error_exit;
            }

            HubDescriptor =
                ExAllocatePoolWithTag(NonPagedPool, NumberOfBytes as usize, USB_HUB_TAG)
                    as PUSB_HUB_DESCRIPTOR;

            if HubDescriptor.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
                break 'error_exit;
            }

            RtlZeroMemory(HubDescriptor as PVOID, NumberOfBytes as usize);
        }

        NumberPorts = (*HubDescriptor).bNumberOfPorts as ULONG;

        if !(*HubExtension).PortData.is_null() {
            PortData = (*HubExtension).PortData;

            ix = 0;
            while ix < NumberPorts {
                psc_set_asulong32(core::ptr::addr_of_mut!((*PortData.offset(ix as isize)).PortStatus), 0);

                if !ExtendedHubInfo.is_null() {
                    (*PortData.offset(ix as isize)).PortAttributes =
                        (*ExtendedHubInfo).Port[ix as usize].PortAttributes;
                } else {
                    (*PortData.offset(ix as isize)).PortAttributes = 0;
                }

                (*PortData.offset(ix as isize)).ConnectionStatus = NoDeviceConnected;

                if !(*PortData.offset(ix as isize)).DeviceObject.is_null() {
                    (*PortData.offset(ix as isize)).ConnectionStatus = DeviceConnected;
                }

                ix += 1;
            }
        } else {
            PortData = core::ptr::null_mut();

            if (*HubDescriptor).bNumberOfPorts != 0 {
                PortData = ExAllocatePoolWithTag(
                    NonPagedPool,
                    NumberPorts as usize * core::mem::size_of::<USBHUB_PORT_DATA>(),
                    USB_HUB_TAG,
                ) as PUSBHUB_PORT_DATA;
            }

            if PortData.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
                break 'error_exit;
            }

            RtlZeroMemory(
                PortData as PVOID,
                NumberPorts as usize * core::mem::size_of::<USBHUB_PORT_DATA>(),
            );

            ix = 0;
            while ix < NumberPorts {
                (*PortData.offset(ix as isize)).ConnectionStatus = NoDeviceConnected;

                if !ExtendedHubInfo.is_null() {
                    (*PortData.offset(ix as isize)).PortAttributes =
                        (*ExtendedHubInfo).Port[ix as usize].PortAttributes;
                }

                ix += 1;
            }
        }

        if !NT_SUCCESS(Status) {
            break 'error_exit;
        }

        (*HubExtension).HubDescriptor = HubDescriptor;

        (*HubExtension).PortData = PortData;

        if !ExtendedHubInfo.is_null() {
            ExFreePoolWithTag(ExtendedHubInfo as PVOID, USB_HUB_TAG);
        }

        return Status;
    }

    // ErrorExit:

    if !HubDescriptor.is_null() {
        ExFreePoolWithTag(HubDescriptor as PVOID, USB_HUB_TAG);
    }

    if !ExtendedHubInfo.is_null() {
        ExFreePoolWithTag(ExtendedHubInfo as PVOID, USB_HUB_TAG);
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_SyncGetStringDescriptor(
    DeviceObject: PDEVICE_OBJECT,
    Index: UCHAR,
    LanguageId: USHORT,
    Descriptor: PUSB_STRING_DESCRIPTOR,
    NumberOfBytes: ULONG,
    OutLength: PULONG,
    IsValidateLength: BOOLEAN,
) -> NTSTATUS {
    let Urb: *mut _URB_CONTROL_DESCRIPTOR_REQUEST;
    let TransferedLength: ULONG;
    let mut Status: NTSTATUS;

    // DPRINT("USBH_SyncGetStringDescriptor: Index - %x, LanguageId - %x\n", Index, LanguageId);

    Urb = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<_URB_CONTROL_DESCRIPTOR_REQUEST>(),
        USB_HUB_TAG,
    ) as *mut _URB_CONTROL_DESCRIPTOR_REQUEST;

    if Urb.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    core::ptr::write_bytes(
        Urb as *mut u8,
        0,
        core::mem::size_of::<_URB_CONTROL_DESCRIPTOR_REQUEST>(),
    );

    (*Urb).Hdr.Function = URB_FUNCTION_GET_DESCRIPTOR_FROM_DEVICE as USHORT;
    (*Urb).Hdr.Length = core::mem::size_of::<_URB_CONTROL_DESCRIPTOR_REQUEST>() as USHORT;

    (*Urb).TransferBuffer = Descriptor as PVOID;
    (*Urb).TransferBufferLength = NumberOfBytes;

    (*Urb).Index = Index;
    (*Urb).DescriptorType = USB_STRING_DESCRIPTOR_TYPE as UCHAR;
    (*Urb).LanguageId = LanguageId;

    Status = USBH_SyncSubmitUrb(DeviceObject, Urb as PURB);

    if !NT_SUCCESS(Status) {
        ExFreePoolWithTag(Urb as PVOID, USB_HUB_TAG);
        return Status;
    }

    TransferedLength = (*Urb).TransferBufferLength;

    if TransferedLength > NumberOfBytes {
        Status = STATUS_DEVICE_DATA_ERROR;
    }

    if !NT_SUCCESS(Status) {
        ExFreePoolWithTag(Urb as PVOID, USB_HUB_TAG);
        return Status;
    }

    if !OutLength.is_null() {
        *OutLength = TransferedLength;
    }

    if IsValidateLength != 0 && TransferedLength != (*Descriptor).bLength as ULONG {
        Status = STATUS_DEVICE_DATA_ERROR;
    }

    ExFreePoolWithTag(Urb as PVOID, USB_HUB_TAG);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_SyncGetStatus(
    DeviceObject: PDEVICE_OBJECT,
    OutStatus: PUSHORT,
    Function: USHORT,
    RequestIndex: USHORT,
) -> NTSTATUS {
    let Urb: *mut _URB_CONTROL_GET_STATUS_REQUEST;
    let NtStatus: NTSTATUS;
    let mut UsbStatus: USHORT = 0;

    Urb = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<_URB_CONTROL_GET_STATUS_REQUEST>(),
        USB_HUB_TAG,
    ) as *mut _URB_CONTROL_GET_STATUS_REQUEST;

    if Urb.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlZeroMemory(
        Urb as *mut core::ffi::c_void,
        core::mem::size_of::<_URB_CONTROL_GET_STATUS_REQUEST>(),
    );

    (*Urb).Hdr.Length = core::mem::size_of::<_URB_CONTROL_GET_STATUS_REQUEST>() as USHORT;
    (*Urb).Hdr.Function = Function as USHORT;

    (*Urb).TransferBuffer = core::ptr::addr_of_mut!(UsbStatus) as PVOID;
    (*Urb).TransferBufferLength = core::mem::size_of::<USHORT>() as ULONG;
    (*Urb).Index = RequestIndex;

    NtStatus = USBH_FdoSyncSubmitUrb(DeviceObject, Urb as PURB);

    *OutStatus = UsbStatus;

    ExFreePoolWithTag(Urb as *mut core::ffi::c_void, USB_HUB_TAG);

    return NtStatus;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_SyncGetHubStatus(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    HubStatus: PUSB_HUB_STATUS_AND_CHANGE,
    Length: ULONG,
) -> NTSTATUS {
    let mut RequestType: BM_REQUEST_TYPE = core::mem::zeroed();

    // DPRINT("USBH_SyncGetHubStatus\n");

    RequestType.set_B(0);
    RequestType.set_Recipient(BMREQUEST_TO_DEVICE);
    RequestType.set_Type(BMREQUEST_CLASS);
    RequestType.set_Dir(BMREQUEST_DEVICE_TO_HOST);

    return USBH_Transact(
        HubExtension,
        HubStatus as PVOID,
        Length,
        BMREQUEST_DEVICE_TO_HOST as BOOLEAN,
        URB_FUNCTION_CLASS_DEVICE as USHORT,
        RequestType,
        USB_REQUEST_GET_STATUS as UCHAR,
        0,
        0,
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBH_SyncClearHubStatus(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    RequestValue: USHORT,
) -> NTSTATUS {
    let mut RequestType: BM_REQUEST_TYPE = core::mem::zeroed();

    // DPRINT("USBH_SyncClearHubStatus: RequestValue - %x\n", RequestValue);

    RequestType.set_B(0);
    RequestType.set_Recipient(BMREQUEST_TO_DEVICE);
    RequestType.set_Type(BMREQUEST_CLASS);
    RequestType.set_Dir(BMREQUEST_HOST_TO_DEVICE);

    return USBH_Transact(
        HubExtension,
        core::ptr::null_mut(),
        0,
        BMREQUEST_HOST_TO_DEVICE as BOOLEAN,
        URB_FUNCTION_CLASS_DEVICE as USHORT,
        RequestType,
        USB_REQUEST_CLEAR_FEATURE as UCHAR,
        RequestValue,
        0,
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBH_SyncGetPortStatus(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    Port: USHORT,
    PortStatus: PUSB_PORT_STATUS_AND_CHANGE,
    Length: ULONG,
) -> NTSTATUS {
    let mut RequestType: BM_REQUEST_TYPE = core::mem::zeroed();

    // DPRINT("USBH_SyncGetPortStatus: Port - %x\n", Port);

    RequestType.set_B(0);
    RequestType.set_Recipient(BMREQUEST_TO_OTHER as u8);
    RequestType.set_Type(BMREQUEST_CLASS as u8);
    RequestType.set_Dir(BMREQUEST_DEVICE_TO_HOST as u8);

    return USBH_Transact(
        HubExtension,
        PortStatus as PVOID,
        Length,
        BMREQUEST_DEVICE_TO_HOST as BOOLEAN,
        URB_FUNCTION_CLASS_OTHER as USHORT,
        RequestType,
        USB_REQUEST_GET_STATUS as UCHAR,
        0,
        Port,
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBH_SyncClearPortStatus(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    Port: USHORT,
    RequestValue: USHORT,
) -> NTSTATUS {
    let mut RequestType: BM_REQUEST_TYPE = core::mem::zeroed();

    RequestType.set_B(0);
    RequestType.set_Recipient(BMREQUEST_TO_OTHER as UCHAR);
    RequestType.set_Type(BMREQUEST_CLASS as UCHAR);
    RequestType.set_Dir(BMREQUEST_HOST_TO_DEVICE as UCHAR);

    return USBH_Transact(
        HubExtension,
        core::ptr::null_mut(),
        0,
        BMREQUEST_HOST_TO_DEVICE as BOOLEAN,
        URB_FUNCTION_CLASS_OTHER as USHORT,
        RequestType,
        USB_REQUEST_CLEAR_FEATURE as UCHAR,
        RequestValue,
        Port,
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBH_SyncPowerOnPort(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    Port: USHORT,
    IsWait: BOOLEAN,
) -> NTSTATUS {
    let PortData: PUSBHUB_PORT_DATA;
    let HubDescriptor: PUSB_HUB_DESCRIPTOR;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut RequestType: BM_REQUEST_TYPE = core::mem::zeroed();
    let PortStatus: PUSB_PORT_STATUS_AND_CHANGE;

    PortData = (*HubExtension).PortData.add((Port - 1) as usize);
    PortStatus = core::ptr::addr_of_mut!((*PortData).PortStatus);

    if (*psc_status20(PortStatus)).CurrentConnectStatus() == 1 {
        return Status;
    }

    RequestType.set_B(0);
    RequestType.set_Recipient(BMREQUEST_TO_DEVICE);
    RequestType.set_Type(BMREQUEST_CLASS);
    RequestType.set_Dir(BMREQUEST_HOST_TO_DEVICE);

    Status = USBH_Transact(
        HubExtension,
        core::ptr::null_mut(),
        0,
        BMREQUEST_HOST_TO_DEVICE as BOOLEAN,
        URB_FUNCTION_CLASS_OTHER as USHORT,
        RequestType,
        USB_REQUEST_SET_FEATURE as UCHAR,
        USBHUB_FEATURE_PORT_POWER as USHORT,
        Port,
    );

    if NT_SUCCESS(Status) {
        if IsWait != 0 {
            HubDescriptor = (*HubExtension).HubDescriptor;
            USBH_Wait(2 * (*HubDescriptor).bPowerOnToPowerGood as ULONG);
        }

        (*psc_status20_mut(PortStatus)).set_CurrentConnectStatus(1);
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_SyncPowerOnPorts(
    HubExtension: PUSBHUB_FDO_EXTENSION,
) -> NTSTATUS {
    let HubDescriptor: PUSB_HUB_DESCRIPTOR;
    let mut Status: NTSTATUS = STATUS_UNSUCCESSFUL;
    let mut Port: USHORT;
    let NumberOfPorts: UCHAR;

    HubDescriptor = (*HubExtension).HubDescriptor;
    NumberOfPorts = (*HubDescriptor).bNumberOfPorts;

    Port = 1;
    while Port as UCHAR <= NumberOfPorts {
        Status = USBH_SyncPowerOnPort(HubExtension, Port, 0);

        if !NT_SUCCESS(Status) {
            break;
        }

        Port += 1;
    }

    USBH_Wait(2u32 * (*HubDescriptor).bPowerOnToPowerGood as ULONG);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_SyncDisablePort(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    Port: USHORT,
) -> NTSTATUS {
    let PortData: PUSBHUB_PORT_DATA;
    let Status: NTSTATUS;
    let mut RequestType: BM_REQUEST_TYPE = core::mem::zeroed();

    PortData = (*HubExtension).PortData.add((Port - 1) as usize);

    RequestType.set_B(0);
    RequestType.set_Recipient(BMREQUEST_TO_DEVICE);
    RequestType.set_Type(BMREQUEST_CLASS);
    RequestType.set_Dir(BMREQUEST_HOST_TO_DEVICE);

    Status = USBH_Transact(
        HubExtension,
        core::ptr::null_mut(),
        0,
        BMREQUEST_HOST_TO_DEVICE as BOOLEAN,
        URB_FUNCTION_CLASS_OTHER as USHORT,
        RequestType,
        USB_REQUEST_CLEAR_FEATURE as UCHAR,
        USBHUB_FEATURE_PORT_ENABLE as USHORT,
        Port,
    );

    if NT_SUCCESS(Status) {
        (*psc_status20_mut(core::ptr::addr_of_mut!((*PortData).PortStatus))).set_PortEnabledDisabled(0);
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_HubIsBusPowered(
    DeviceObject: PDEVICE_OBJECT,
    HubConfigDescriptor: PUSB_CONFIGURATION_DESCRIPTOR,
) -> BOOLEAN {
    let Result: BOOLEAN;
    let mut UsbStatus: USHORT = 0;
    let Status: NTSTATUS;

    Status = USBH_SyncGetStatus(
        DeviceObject,
        core::ptr::addr_of_mut!(UsbStatus),
        URB_FUNCTION_GET_STATUS_FROM_DEVICE as USHORT,
        0,
    );

    if !NT_SUCCESS(Status) {
        Result = (((*HubConfigDescriptor).bmAttributes as u32 & USB_CONFIG_POWERED_MASK)
            == USB_CONFIG_BUS_POWERED) as BOOLEAN;
    } else {
        Result = ((UsbStatus as u32 & USB_GETSTATUS_SELF_POWERED) == 0) as BOOLEAN;
    }

    return Result;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_ChangeIndicationAckChangeComplete(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    let HubExtension: PUSBHUB_FDO_EXTENSION;
    let Event: PVOID;
    let Port: USHORT;

    HubExtension = Context as PUSBHUB_FDO_EXTENSION;

    DPRINT_SCE!(b"USBH_ChangeIndicationAckChangeComplete: ... \n\0".as_ptr() as *const i8);

    // ASSERT(HubExtension->Port > 0);
    Port = (*HubExtension).Port - 1;

    core::ptr::copy_nonoverlapping(core::ptr::addr_of!((*HubExtension).PortStatus), core::ptr::addr_of_mut!((*(*HubExtension).PortData.add(Port as usize)).PortStatus), 1);

    Event = InterlockedExchangePointer(
        core::ptr::addr_of_mut!((*HubExtension).pResetPortEvent) as *mut PVOID,
        core::ptr::null_mut(),
    );

    if !Event.is_null() {
        KeSetEvent(Event as PVOID, EVENT_INCREMENT as LONG, 0);
    }

    USBH_SubmitStatusChangeTransfer(HubExtension);

    if InterlockedDecrement(core::ptr::addr_of_mut!((*HubExtension).ResetRequestCount)) == 0 {
        KeSetEvent(core::ptr::addr_of_mut!((*HubExtension).ResetEvent) as PVOID, EVENT_INCREMENT as LONG, 0);
    }

    return STATUS_MORE_PROCESSING_REQUIRED;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_ChangeIndicationAckChange(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    Irp: PIRP,
    Urb: *mut _URB_CONTROL_VENDOR_OR_CLASS_REQUEST,
    Port: USHORT,
    RequestValue: USHORT,
) -> NTSTATUS {

    let IoStack: PIO_STACK_LOCATION;
    let mut RequestType: BM_REQUEST_TYPE = core::mem::zeroed();

    // DPRINT_SCE!("USBH_ChangeIndicationAckChange: ... \n");

    (*Urb).Hdr.Length = core::mem::size_of::<_URB_CONTROL_VENDOR_OR_CLASS_REQUEST>() as USHORT;
    (*Urb).Hdr.Function = URB_FUNCTION_CLASS_OTHER as USHORT;
    (*Urb).Hdr.UsbdDeviceHandle = core::ptr::null_mut();

    (*Urb).TransferFlags = USBD_SHORT_TRANSFER_OK;
    (*Urb).TransferBufferLength = 0;
    (*Urb).TransferBuffer = core::ptr::null_mut();
    (*Urb).TransferBufferMDL = core::ptr::null_mut();
    (*Urb).UrbLink = core::ptr::null_mut();

    RequestType.set_B(0);
    RequestType.set_Recipient(BMREQUEST_TO_OTHER);
    RequestType.set_Type(BMREQUEST_CLASS);
    RequestType.set_Dir(BMREQUEST_HOST_TO_DEVICE);

    (*Urb).RequestTypeReservedBits = *RequestType.B.as_ref();
    (*Urb).Request = USB_REQUEST_CLEAR_FEATURE as UCHAR;
    (*Urb).Index = Port;
    (*Urb).Value = RequestValue;

    IoInitializeIrp(
        Irp,
        IoSizeOfIrp((*(*HubExtension).LowerDevice).StackSize),
        (*(*HubExtension).LowerDevice).StackSize,
    );

    IoStack = IoGetNextIrpStackLocation(Irp);

    (*IoStack).MajorFunction = IRP_MJ_INTERNAL_DEVICE_CONTROL as u8;
    (*IoStack).Parameters.Others.Argument1 = Urb as *mut core::ffi::c_void;
    (*IoStack).Parameters.DeviceIoControl.IoControlCode = IOCTL_INTERNAL_USB_SUBMIT_URB;

    IoSetCompletionRoutine(
        Irp,
        cb!(USBH_ChangeIndicationAckChangeComplete),
        HubExtension as *mut core::ffi::c_void,
        1,
        1,
        1,
    );

    return IoCallDriver((*HubExtension).LowerDevice, Irp);
}

#[no_mangle]
pub unsafe extern "system" fn USBH_ChangeIndicationProcessChange(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    let HubExtension: PUSBHUB_FDO_EXTENSION;
    let WorkItem: PUSBHUB_IO_WORK_ITEM;
    let RequestValue: USHORT;

    HubExtension = Context as PUSBHUB_FDO_EXTENSION;

    DPRINT_SCE!(
        b"USBH_ChangeIndicationProcessChange: PortStatus - %lX\n\0".as_ptr() as *const i8,
        psc_get_asulong32(core::ptr::addr_of!((*HubExtension).PortStatus)),
    );

    if (NT_SUCCESS((*Irp).IoStatus.__bindgen_anon_1.Status)
        || USBD_SUCCESS((*HubExtension).SCEWorkerUrb.Hdr.Status) != 0)
        && ((*psc_change20(core::ptr::addr_of!((*HubExtension).PortStatus))).ResetChange()
            != 0
            || (*psc_change20(core::ptr::addr_of!((*HubExtension).PortStatus))).PortEnableDisableChange()
                != 0)
    {
        if InterlockedDecrement(core::ptr::addr_of_mut!((*HubExtension).PendingRequestCount)) == 0 {
            KeSetEvent(core::ptr::addr_of_mut!((*HubExtension).PendingRequestEvent) as PVOID, EVENT_INCREMENT as KPRIORITY, 0);
        }

        USBH_FreeWorkItem((*HubExtension).WorkItemToQueue);

        (*HubExtension).WorkItemToQueue = core::ptr::null_mut();

        if (*psc_change20(core::ptr::addr_of!((*HubExtension).PortStatus))).ResetChange()
            != 0
        {
            RequestValue = USBHUB_FEATURE_C_PORT_RESET as USHORT;
        } else {
            RequestValue = USBHUB_FEATURE_C_PORT_ENABLE as USHORT;
        }

        USBH_ChangeIndicationAckChange(
            HubExtension,
            (*HubExtension).ResetPortIrp,
            core::ptr::addr_of_mut!((*HubExtension).SCEWorkerUrb),
            (*HubExtension).Port,
            RequestValue,
        );
    } else {
        ASSERT!(!(*HubExtension).WorkItemToQueue.is_null());

        WorkItem = (*HubExtension).WorkItemToQueue;
        (*HubExtension).WorkItemToQueue = core::ptr::null_mut();

        USBH_QueueWorkItem(HubExtension, WorkItem);
    }

    return STATUS_MORE_PROCESSING_REQUIRED;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_ChangeIndicationQueryChange(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    Irp: PIRP,
    Urb: *mut _URB_CONTROL_VENDOR_OR_CLASS_REQUEST,
    Port: USHORT,
) -> NTSTATUS {

    let WorkItem: PUSBHUB_IO_WORK_ITEM;
    let Status: NTSTATUS;
    let IoStack: PIO_STACK_LOCATION;
    let mut RequestType: BM_REQUEST_TYPE = core::mem::zeroed();

    DPRINT_SCE!(
        b"USBH_ChangeIndicationQueryChange: Port - %x\n\0".as_ptr() as *const i8,
        Port as u32,
    );

    InterlockedIncrement(core::ptr::addr_of_mut!((*HubExtension).PendingRequestCount));

    if Port == 0 {
        WorkItem = (*HubExtension).WorkItemToQueue;
        (*HubExtension).WorkItemToQueue = core::ptr::null_mut();

        USBH_QueueWorkItem(HubExtension, WorkItem);

        return STATUS_SUCCESS;
    }

    (*Urb).Hdr.Length = core::mem::size_of::<_URB_CONTROL_VENDOR_OR_CLASS_REQUEST>() as USHORT;
    (*Urb).Hdr.UsbdDeviceHandle = core::ptr::null_mut();
    (*Urb).Hdr.Function = URB_FUNCTION_CLASS_OTHER as USHORT;

    (*Urb).TransferFlags = (USBD_SHORT_TRANSFER_OK | USBD_TRANSFER_DIRECTION_IN) as u32;
    (*Urb).TransferBuffer = core::ptr::addr_of_mut!((*HubExtension).PortStatus) as PVOID;
    (*Urb).TransferBufferLength = core::mem::size_of_val(&(*HubExtension).PortStatus) as u32;
    (*Urb).TransferBufferMDL = core::ptr::null_mut();
    (*Urb).UrbLink = core::ptr::null_mut();

    RequestType.set_B(0);
    RequestType.set_Recipient(BMREQUEST_TO_OTHER as u8);
    RequestType.set_Type(BMREQUEST_CLASS as u8);
    RequestType.set_Dir(BMREQUEST_DEVICE_TO_HOST as u8);

    (*Urb).RequestTypeReservedBits = *RequestType.B.as_ref();
    (*Urb).Request = USB_REQUEST_GET_STATUS as u8;
    (*Urb).Value = 0;
    (*Urb).Index = Port;

    (*HubExtension).Port = Port;

    IoInitializeIrp(
        Irp,
        IoSizeOfIrp((*(*HubExtension).LowerDevice).StackSize),
        (*(*HubExtension).LowerDevice).StackSize,
    );

    IoStack = IoGetNextIrpStackLocation(Irp);

    (*IoStack).MajorFunction = IRP_MJ_INTERNAL_DEVICE_CONTROL as u8;
    (*IoStack).Parameters.Others.Argument1 = Urb as PVOID;
    (*IoStack).Parameters.DeviceIoControl.IoControlCode = IOCTL_INTERNAL_USB_SUBMIT_URB;

    IoSetCompletionRoutine(
        Irp,
        cb!(USBH_ChangeIndicationProcessChange),
        HubExtension as PVOID,
        1,
        1,
        1,
    );

    Status = IoCallDriver((*HubExtension).LowerDevice, Irp);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_ProcessHubStateChange(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    HubStatus: PUSB_HUB_STATUS_AND_CHANGE,
) {
    DPRINT_SCE!(
        b"USBH_ProcessHubStateChange: HubStatus - %lx\n\0".as_ptr() as *const i8,
        hsc_get_asulong32(HubStatus),
    );

    // HubStatusChange = HubStatus->HubChange; (bindgen union view, accessed via helper)
    let HubStatusChange = hsc_change_bits(HubStatus);

    if (*HubStatusChange).LocalPowerChange() != 0 {
        DbgPrint(b"USBH_ProcessHubStateChange: LocalPowerChange\n\0".as_ptr() as *const i8);
        USBH_SyncClearHubStatus(HubExtension, USBHUB_FEATURE_C_HUB_LOCAL_POWER as USHORT);
    }

    if (*HubStatusChange).OverCurrentChange() != 0 {
        USBH_SyncClearHubStatus(HubExtension, USBHUB_FEATURE_C_HUB_OVER_CURRENT as USHORT);
        if (*hsc_status_bits(HubStatus)).OverCurrent() != 0 {
            DbgPrint(b"USBH_ProcessHubStateChange: OverCurrent UNIMPLEMENTED. FIXME\n\0".as_ptr() as *const i8);
            DbgBreakPoint();
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBH_ProcessPortStateChange(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    Port: USHORT,
    PortStatus: PUSB_PORT_STATUS_AND_CHANGE,
) {
    let PortData: PUSBHUB_PORT_DATA;
    let PortStatusChange: USB_20_PORT_CHANGE;
    let PortDevice: PDEVICE_OBJECT;
    let PortExtension: PUSBHUB_PORT_PDO_EXTENSION;
    let SerialNumber: PVOID;
    let DeviceHandle: PVOID;
    let RequestValue: USHORT;
    let mut Irql: KIRQL = 0;

    DPRINT_SCE!(b"USBH_ProcessPortStateChange ... \n\0".as_ptr() as *const i8);

    let PortData = (*HubExtension).PortData.offset((Port - 1) as isize);

    // PortStatusChange = PortStatus->PortChange.Usb20PortChange; (bindgen union → bitfield ptr)
    let PortStatusChange = psc_change20(PortStatus);

    if (*PortStatusChange).ConnectStatusChange() != 0 {
        core::ptr::copy_nonoverlapping(PortStatus, core::ptr::addr_of_mut!((*PortData).PortStatus), 1);

        USBH_SyncClearPortStatus(HubExtension, Port, USBHUB_FEATURE_C_PORT_CONNECTION as USHORT);

        let PortData = (*HubExtension).PortData.offset((Port - 1) as isize);

        let PortDevice = (*PortData).DeviceObject;

        if PortDevice.is_null() {
            IoInvalidateDeviceRelations((*HubExtension).LowerPDO, BusRelations);
            return;
        }

        let PortExtension = (*PortDevice).DeviceExtension as PUSBHUB_PORT_PDO_EXTENSION;

        if (*PortExtension).PortPdoFlags & USBHUB_PDO_FLAG_OVERCURRENT_PORT != 0 {
            return;
        }

        KeAcquireSpinLock(
            core::ptr::addr_of_mut!((*HubExtension).RelationsWorkerSpinLock) as PKSPIN_LOCK,
            core::ptr::addr_of_mut!(Irql),
        );

        if (*PortExtension).PortPdoFlags & USBHUB_PDO_FLAG_POWER_D3 != 0 {
            KeReleaseSpinLock(
                core::ptr::addr_of_mut!((*HubExtension).RelationsWorkerSpinLock) as PKSPIN_LOCK,
                Irql,
            );
            IoInvalidateDeviceRelations((*HubExtension).LowerPDO, BusRelations);
            return;
        }

        (*PortData).DeviceObject = core::ptr::null_mut();
        (*PortData).ConnectionStatus = NoDeviceConnected;

        (*HubExtension).HubFlags |= USBHUB_FDO_FLAG_STATE_CHANGING;

        InsertTailList(
            core::ptr::addr_of_mut!((*HubExtension).PdoList),
            core::ptr::addr_of_mut!((*PortExtension).PortLink),
        );

        KeReleaseSpinLock(
            core::ptr::addr_of_mut!((*HubExtension).RelationsWorkerSpinLock) as PKSPIN_LOCK,
            Irql,
        );

        let SerialNumber = InterlockedExchangePointer(
            core::ptr::addr_of_mut!((*PortExtension).SerialNumber) as *mut PVOID,
            core::ptr::null_mut(),
        );

        if !SerialNumber.is_null() {
            ExFreePoolWithTag(SerialNumber, USB_HUB_TAG);
        }

        let DeviceHandle = InterlockedExchangePointer(
            core::ptr::addr_of_mut!((*PortExtension).DeviceHandle),
            core::ptr::null_mut(),
        );

        if !DeviceHandle.is_null() {
            USBD_RemoveDeviceEx(HubExtension, DeviceHandle, 0);
            USBH_SyncDisablePort(HubExtension, Port);
        }

        IoInvalidateDeviceRelations((*HubExtension).LowerPDO, BusRelations);
        return;
    }

    if (*PortStatusChange).PortEnableDisableChange() != 0 {
        let RequestValue = USBHUB_FEATURE_C_PORT_ENABLE;
        core::ptr::copy_nonoverlapping(PortStatus, core::ptr::addr_of_mut!((*PortData).PortStatus), 1);
        USBH_SyncClearPortStatus(HubExtension, Port, RequestValue as USHORT);
    }

    if (*PortStatusChange).SuspendChange() != 0 {
        DbgPrint(b"USBH_ProcessPortStateChange: SuspendChange UNIMPLEMENTED. FIXME\n\0".as_ptr() as *const i8);
        DbgBreakPoint();
    }

    if (*PortStatusChange).OverCurrentIndicatorChange() != 0 {
        DbgPrint(b"USBH_ProcessPortStateChange: OverCurrentIndicatorChange UNIMPLEMENTED. FIXME\n\0".as_ptr() as *const i8);
        DbgBreakPoint();
    }

    if (*PortStatusChange).ResetChange() != 0 {
        let RequestValue = USBHUB_FEATURE_C_PORT_RESET;
        core::ptr::copy_nonoverlapping(PortStatus, core::ptr::addr_of_mut!((*PortData).PortStatus), 1);
        USBH_SyncClearPortStatus(HubExtension, Port, RequestValue as USHORT);
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBH_GetPortStatus(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    PortStatus: PULONG,
) -> NTSTATUS {
    let Irp: PIRP;
    let IoStack: PIO_STACK_LOCATION;
    let Status: NTSTATUS;
    let mut Event: KEVENT = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    // DPRINT("USBH_GetPortStatus ... \n");

    *PortStatus = 0;

    KeInitializeEvent(core::ptr::addr_of_mut!(Event), _EVENT_TYPE_NotificationEvent, 0);

    Irp = IoBuildDeviceIoControlRequest(
        IOCTL_INTERNAL_USB_GET_PORT_STATUS,
        (*HubExtension).LowerDevice,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        0,
        1,
        core::ptr::addr_of_mut!(Event),
        core::ptr::addr_of_mut!(IoStatusBlock),
    );

    if Irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    IoStack = IoGetNextIrpStackLocation(Irp);
    (*IoStack).Parameters.Others.Argument1 = PortStatus as *mut core::ffi::c_void;

    Status = IoCallDriver((*HubExtension).LowerDevice, Irp);

    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(Event) as *mut core::ffi::c_void,
            _KWAIT_REASON_Suspended,
            0i32 as i8,
            0,
            core::ptr::null_mut(),
        );
    } else {
        IoStatusBlock.__bindgen_anon_1.Status = Status;
    }

    IoStatusBlock.__bindgen_anon_1.Status
}

#[no_mangle]
pub unsafe extern "system" fn USBH_EnableParentPort(
    HubExtension: PUSBHUB_FDO_EXTENSION,
) -> NTSTATUS {
    let Irp: PIRP;
    let Status: NTSTATUS;
    let mut Event: KEVENT = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    // DPRINT("USBH_EnableParentPort ... \n");

    KeInitializeEvent(core::ptr::addr_of_mut!(Event), _EVENT_TYPE_NotificationEvent, 0);

    Irp = IoBuildDeviceIoControlRequest(
        IOCTL_INTERNAL_USB_ENABLE_PORT,
        (*HubExtension).LowerDevice,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        0,
        1,
        core::ptr::addr_of_mut!(Event),
        core::ptr::addr_of_mut!(IoStatusBlock),
    );

    if Irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = IoCallDriver((*HubExtension).LowerDevice, Irp);

    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(Event) as *mut core::ffi::c_void,
            _KWAIT_REASON_Suspended,
            0i32 as i8,
            0,
            core::ptr::null_mut(),
        );
    } else {
        IoStatusBlock.__bindgen_anon_1.Status = Status;
    }

    return IoStatusBlock.__bindgen_anon_1.Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_ResetInterruptPipe(
    HubExtension: PUSBHUB_FDO_EXTENSION,
) -> NTSTATUS {
    let Urb: *mut _URB_PIPE_REQUEST;
    let mut Status: NTSTATUS;

    Urb = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<_URB_PIPE_REQUEST>(),
        USB_HUB_TAG,
    ) as *mut _URB_PIPE_REQUEST;

    if !Urb.is_null() {
        RtlZeroMemory(Urb as *mut _, core::mem::size_of::<_URB_PIPE_REQUEST>());

        (*Urb).Hdr.Length = core::mem::size_of::<_URB_PIPE_REQUEST>() as USHORT;
        (*Urb).Hdr.Function = URB_FUNCTION_SYNC_RESET_PIPE_AND_CLEAR_STALL as USHORT;
        (*Urb).PipeHandle = (*HubExtension).PipeInfo.PipeHandle;

        Status = USBH_FdoSyncSubmitUrb((*HubExtension).Common.SelfDevice, Urb as PURB);

        ExFreePoolWithTag(Urb as *mut _, USB_HUB_TAG);
    } else {
        Status = STATUS_INSUFFICIENT_RESOURCES;
    }

    if NT_SUCCESS(Status) {
        (*HubExtension).RequestErrors = 0;
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_ResetHub(HubExtension: PUSBHUB_FDO_EXTENSION) -> NTSTATUS {
    const USBD_PORT_ENABLED: ULONG = 0x00000001;
    const USBD_PORT_CONNECTED: ULONG = 0x00000002;

    let mut Status: NTSTATUS;
    let mut PortStatusFlags: ULONG = 0;

    // DPRINT removed

    Status = USBH_GetPortStatus(HubExtension, core::ptr::addr_of_mut!(PortStatusFlags));

    if !NT_SUCCESS(Status) {
        return Status;
    }

    if (PortStatusFlags & USBD_PORT_ENABLED) == 0 {
        if (PortStatusFlags & USBD_PORT_CONNECTED) != 0 {
            USBH_EnableParentPort(HubExtension);
        }
    }

    Status = USBH_ResetInterruptPipe(HubExtension);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_ChangeIndicationWorker(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    Context: PVOID,
) {
    let mut LowerHubExtension: PUSBHUB_FDO_EXTENSION;
    let mut LowerPortExtension: PUSBHUB_PORT_PDO_EXTENSION;
    let mut WorkItem: PUSBHUB_STATUS_CHANGE_CONTEXT;
    let mut PortStatus: USB_PORT_STATUS_AND_CHANGE = core::mem::zeroed();
    let mut HubStatus: USB_HUB_STATUS_AND_CHANGE = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut Port: USHORT = 0;

    DPRINT_SCE!(b"USBH_ChangeIndicationWorker ... \n\0".as_ptr() as *const i8);

    WorkItem = Context as PUSBHUB_STATUS_CHANGE_CONTEXT;

    KeWaitForSingleObject(
        core::ptr::addr_of_mut!((*HubExtension).HubSemaphore) as PVOID,
        _KWAIT_REASON_Executive,
        0i32 as i8,
        0,
        core::ptr::null_mut(),
    );

    'Exit: loop {
        'Enum: loop {
            if (*HubExtension).HubFlags & USBHUB_FDO_FLAG_DEVICE_STOPPING != 0 {
                KeSetEvent(core::ptr::addr_of_mut!((*HubExtension).StatusChangeEvent) as PVOID, EVENT_INCREMENT as LONG, 0);

                break 'Exit;
            }

            if (*HubExtension).RequestErrors == 0 {
                break 'Enum;
            }

            DPRINT_SCE!(
                b"USBH_ChangeIndicationWorker: RequestErrors - %x\n\0".as_ptr() as *const i8,
                (*HubExtension).RequestErrors,
            );

            if (*HubExtension).LowerPDO == (*HubExtension).RootHubPdo {
                break 'Enum;
            }

            LowerPortExtension =
                (*(*HubExtension).LowerPDO).DeviceExtension as PUSBHUB_PORT_PDO_EXTENSION;

            if (*LowerPortExtension).PortPdoFlags & USBHUB_PDO_FLAG_POWER_D1_OR_D2 != 0 {
                break 'Enum;
            }

            LowerHubExtension = (*LowerPortExtension).HubExtension;

            if LowerHubExtension.is_null() {
                break 'Enum;
            }

            Status = USBH_SyncGetPortStatus(
                LowerHubExtension,
                (*LowerPortExtension).PortNumber,
                core::ptr::addr_of_mut!(PortStatus),
                core::mem::size_of::<USB_PORT_STATUS_AND_CHANGE>() as ULONG,
            );

            if !NT_SUCCESS(Status)
                || (*psc_status20(core::ptr::addr_of!(PortStatus))).CurrentConnectStatus()
                    == 0
            {
                (*HubExtension).HubFlags |= USBHUB_FDO_FLAG_DEVICE_REMOVED;

                KeSetEvent(core::ptr::addr_of_mut!((*HubExtension).StatusChangeEvent) as PVOID, EVENT_INCREMENT as LONG, 0);

                break 'Exit;
            }

            if (*HubExtension).HubFlags & USBHUB_FDO_FLAG_ESD_RECOVERING == 0 {
                (*HubExtension).HubFlags |= USBHUB_FDO_FLAG_ESD_RECOVERING;

                // DPRINT removed

                /* Initiate ESD recovery: power cycle the hub (D3 -> D0) */
                USBH_HubStartESDRecovery(HubExtension);

                break 'Exit;
            }

            break 'Enum;
        }

        // Enum:
        if (*WorkItem).IsRequestErrors != 0 {
            USBH_ResetHub(HubExtension);
        } else {
            Port = 0;
            while Port < (*(*HubExtension).HubDescriptor).bNumberOfPorts as USHORT {
                if IsBitSet(WorkItem.offset(1) as PUCHAR, Port) != 0 {
                    break;
                }
                Port += 1;
            }

            if Port != 0 {
                Status = USBH_SyncGetPortStatus(
                    HubExtension,
                    Port,
                    core::ptr::addr_of_mut!(PortStatus),
                    core::mem::size_of::<USB_PORT_STATUS_AND_CHANGE>() as ULONG,
                );
            } else {
                Status = USBH_SyncGetHubStatus(
                    HubExtension,
                    core::ptr::addr_of_mut!(HubStatus),
                    core::mem::size_of::<USB_HUB_STATUS_AND_CHANGE>() as ULONG,
                );
            }

            if NT_SUCCESS(Status) {
                if Port != 0 {
                    USBH_ProcessPortStateChange(
                        HubExtension,
                        Port,
                        core::ptr::addr_of_mut!(PortStatus),
                    );
                } else {
                    USBH_ProcessHubStateChange(HubExtension, core::ptr::addr_of_mut!(HubStatus));
                }
            } else {
                (*HubExtension).RequestErrors += 1;

                if (*HubExtension).RequestErrors > USBHUB_MAX_REQUEST_ERRORS {
                    (*HubExtension).HubFlags |= USBHUB_FDO_FLAG_DEVICE_FAILED;
                    break 'Exit;
                }
            }
        }

        USBH_SubmitStatusChangeTransfer(HubExtension);

        break 'Exit;
    }

    // Exit:
    KeReleaseSemaphore(
        core::ptr::addr_of_mut!((*HubExtension).HubSemaphore),
        LOW_REALTIME_PRIORITY as KPRIORITY,
        1,
        0,
    );

    if InterlockedDecrement(core::ptr::addr_of_mut!((*HubExtension).PendingRequestCount)) == 0 {
        KeSetEvent(core::ptr::addr_of_mut!((*HubExtension).PendingRequestEvent) as PVOID, EVENT_INCREMENT as LONG, 0);
    }

    if InterlockedDecrement(core::ptr::addr_of_mut!((*HubExtension).ResetRequestCount) as PLONG) == 0
    {
        KeSetEvent(core::ptr::addr_of_mut!((*HubExtension).ResetEvent) as PVOID, EVENT_INCREMENT as LONG, 0);

        if (*HubExtension).HubFlags & USBHUB_FDO_FLAG_DEFER_CHECK_IDLE != 0 {
            USBH_CheckHubIdle(HubExtension);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBH_ChangeIndication(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    let HubExtension: PUSBHUB_FDO_EXTENSION;
    let UrbStatus: USBD_STATUS;
    let mut IsErrors: BOOLEAN = 0;
    let mut HubWorkItem: PUSBHUB_IO_WORK_ITEM = core::ptr::null_mut();
    let mut HubWorkItemBuffer: PUSBHUB_STATUS_CHANGE_CONTEXT = core::ptr::null_mut();
    let NumPorts: USHORT;
    let mut Port: USHORT;
    let mut Status: NTSTATUS;
    let Bitmap: PVOID;
    let BufferLength: ULONG;

    HubExtension = Context as PUSBHUB_FDO_EXTENSION;
    UrbStatus = (*HubExtension).SCEWorkerUrb.Hdr.Status;

    DPRINT_SCE!(
        b"USBH_ChangeIndication: IrpStatus - %x, UrbStatus - %x, HubFlags - %lX\n\0".as_ptr() as *const i8,
        (*Irp).IoStatus.__bindgen_anon_1.Status,
        UrbStatus,
        (*HubExtension).HubFlags,
    );

    if ((*Irp).IoStatus.__bindgen_anon_1.Status & 0xC0000000u32 as i32) == 0xC0000000u32 as i32
        || USBD_ERROR(UrbStatus) != 0
        || ((*HubExtension).HubFlags & USBHUB_FDO_FLAG_DEVICE_FAILED) != 0
        || ((*HubExtension).HubFlags & USBHUB_FDO_FLAG_DEVICE_STOPPING) != 0
    {
        (*HubExtension).RequestErrors += 1;

        IsErrors = 1;

        KeSetEvent(core::ptr::addr_of_mut!((*HubExtension).StatusChangeEvent) as PVOID, EVENT_INCREMENT as LONG, 0);

        if (*HubExtension).HubFlags & USBHUB_FDO_FLAG_DEVICE_STOPPING != 0
            || (*HubExtension).HubFlags & USBHUB_FDO_FLAG_DEVICE_FAILED != 0
            || (*HubExtension).RequestErrors > USBHUB_MAX_REQUEST_ERRORS
            || (*Irp).IoStatus.__bindgen_anon_1.Status == STATUS_DELETE_PENDING
        {
            DPRINT_SCE!(
                b"USBH_ChangeIndication: HubExtension->RequestErrors - %x\n\0".as_ptr() as *const i8,
                (*HubExtension).RequestErrors,
            );

            return STATUS_MORE_PROCESSING_REQUIRED;
        }

        DPRINT_SCE!(
            b"USBH_ChangeIndication: HubExtension->RequestErrors - %x\n\0".as_ptr() as *const i8,
            (*HubExtension).RequestErrors,
        );
    } else {
        (*HubExtension).RequestErrors = 0;
    }

    BufferLength = core::mem::size_of::<USBHUB_STATUS_CHANGE_CONTEXT>() as ULONG
        + (*HubExtension).SCEBitmapLength;

    Status = USBH_AllocateWorkItem(
        HubExtension,
        core::ptr::addr_of_mut!(HubWorkItem),
        cb!(USBH_ChangeIndicationWorker),
        BufferLength as SIZE_T,
        core::ptr::addr_of_mut!(HubWorkItemBuffer) as *mut PVOID,
        DelayedWorkQueue,
    );

    if !NT_SUCCESS(Status) {
        return STATUS_MORE_PROCESSING_REQUIRED;
    }

    RtlZeroMemory(HubWorkItemBuffer as PVOID, BufferLength as SIZE_T);

    (*HubWorkItemBuffer).IsRequestErrors = 0;

    if IsErrors != 0 {
        (*HubWorkItemBuffer).IsRequestErrors = 1;
    }

    if InterlockedIncrement(core::ptr::addr_of_mut!((*HubExtension).ResetRequestCount)) == 1 {
        KeClearEvent(core::ptr::addr_of_mut!((*HubExtension).ResetEvent) as PVOID);
    }

    (*HubWorkItemBuffer).HubExtension = HubExtension;

    (*HubExtension).WorkItemToQueue = HubWorkItem;

    Bitmap = HubWorkItemBuffer.offset(1) as PVOID;

    RtlCopyMemory(
        Bitmap,
        (*HubExtension).SCEBitmap as PVOID,
        (*HubExtension).SCEBitmapLength as SIZE_T,
    );

    NumPorts = (*(*HubExtension).HubDescriptor).bNumberOfPorts as USHORT;

    Port = 0;
    while Port <= NumPorts {
        if IsBitSet(Bitmap as PUCHAR, Port) != 0 {
            break;
        }
        Port += 1;
    }

    if Port > NumPorts {
        Port = 0;
    }

    Status = USBH_ChangeIndicationQueryChange(
        HubExtension,
        (*HubExtension).ResetPortIrp,
        core::ptr::addr_of_mut!((*HubExtension).SCEWorkerUrb) as *mut _URB_CONTROL_VENDOR_OR_CLASS_REQUEST,
        Port,
    );

    if (Status & 0xC0000000u32 as i32) == 0xC0000000u32 as i32 {
        (*HubExtension).HubFlags |= USBHUB_FDO_FLAG_DEVICE_FAILED;
    }

    return STATUS_MORE_PROCESSING_REQUIRED;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_SubmitStatusChangeTransfer(
    HubExtension: PUSBHUB_FDO_EXTENSION,
) -> NTSTATUS {

    let Irp: PIRP;
    let Status: NTSTATUS;
    let Urb: *mut _URB_BULK_OR_INTERRUPT_TRANSFER;
    let IoStack: PIO_STACK_LOCATION;

    DPRINT_SCE!(
        b"USBH_SubmitStatusChangeTransfer: HubExtension - %p, SCEIrp - %p\n\0".as_ptr() as *const i8,
        HubExtension,
        (*HubExtension).SCEIrp,
    );

    if (*HubExtension).HubFlags & USBHUB_FDO_FLAG_NOT_D0_STATE != 0 {
        DPRINT_SCE!(
            b"USBH_SubmitStatusChangeTransfer: USBHUB_FDO_FLAG_NOT_D0_STATE\n\0".as_ptr()
                as *const i8,
        );
        DPRINT_SCE!(
            b"USBH_SubmitStatusChangeTransfer: HubFlags - %lX\n\0".as_ptr() as *const i8,
            (*HubExtension).HubFlags,
        );

        return STATUS_INVALID_DEVICE_STATE;
    }

    Irp = (*HubExtension).SCEIrp;

    if Irp.is_null() {
        return STATUS_INVALID_DEVICE_STATE;
    }

    Urb = core::ptr::addr_of_mut!((*HubExtension).SCEWorkerUrb)
        as *mut _URB_BULK_OR_INTERRUPT_TRANSFER;

    (*Urb).Hdr.Length = core::mem::size_of::<_URB_BULK_OR_INTERRUPT_TRANSFER>() as u16;
    (*Urb).Hdr.Function = URB_FUNCTION_BULK_OR_INTERRUPT_TRANSFER as u16;
    (*Urb).Hdr.UsbdDeviceHandle = core::ptr::null_mut();

    (*Urb).PipeHandle = (*HubExtension).PipeInfo.PipeHandle;
    (*Urb).TransferFlags = USBD_SHORT_TRANSFER_OK;
    (*Urb).TransferBuffer = (*HubExtension).SCEBitmap;
    (*Urb).TransferBufferLength = (*HubExtension).SCEBitmapLength;
    (*Urb).TransferBufferMDL = core::ptr::null_mut();
    (*Urb).UrbLink = core::ptr::null_mut();

    IoInitializeIrp(
        Irp,
        IoSizeOfIrp((*(*HubExtension).LowerDevice).StackSize as i8),
        (*(*HubExtension).LowerDevice).StackSize,
    );

    IoStack = IoGetNextIrpStackLocation(Irp);

    (*IoStack).MajorFunction = IRP_MJ_INTERNAL_DEVICE_CONTROL as u8;
    (*IoStack).Parameters.Others.Argument1 = core::ptr::addr_of_mut!((*HubExtension).SCEWorkerUrb) as *mut _;
    (*IoStack).Parameters.DeviceIoControl.IoControlCode = IOCTL_INTERNAL_USB_SUBMIT_URB;

    IoSetCompletionRoutine(
        Irp,
        cb!(USBH_ChangeIndication),
        HubExtension as *mut _,
        1,
        1,
        1,
    );

    KeClearEvent(core::ptr::addr_of_mut!((*HubExtension).StatusChangeEvent) as PVOID);

    Status = IoCallDriver((*HubExtension).LowerDevice, Irp);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBD_CreateDeviceEx(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    OutDeviceHandle: *mut PUSB_DEVICE_HANDLE,
    UsbPortStatus: USB_PORT_STATUS,
    Port: USHORT,
) -> NTSTATUS {
    let HubDeviceHandle: PUSB_DEVICE_HANDLE;
    let CreateUsbDevice: PUSB_BUSIFFN_CREATE_USB_DEVICE;

    CreateUsbDevice = (*HubExtension).BusInterface.CreateUsbDevice;

    if CreateUsbDevice.is_none() {
        return STATUS_NOT_IMPLEMENTED;
    }

    HubDeviceHandle = USBH_SyncGetDeviceHandle((*HubExtension).LowerDevice);

    return (CreateUsbDevice.unwrap())(
        (*HubExtension).BusInterface.BusContext,
        OutDeviceHandle,
        HubDeviceHandle,
        *UsbPortStatus.AsUshort16.as_ref(),
        Port,
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBD_RemoveDeviceEx(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    DeviceHandle: PUSB_DEVICE_HANDLE,
    Flags: ULONG,
) -> NTSTATUS {
    let RemoveUsbDevice: PUSB_BUSIFFN_REMOVE_USB_DEVICE;

    // DPRINT("USBD_RemoveDeviceEx: DeviceHandle - %p, Flags - %X\n", DeviceHandle, Flags);

    RemoveUsbDevice = (*HubExtension).BusInterface.RemoveUsbDevice;

    if RemoveUsbDevice.is_none() {
        return STATUS_NOT_IMPLEMENTED;
    }

    return (RemoveUsbDevice.unwrap())(
        (*HubExtension).BusInterface.BusContext,
        DeviceHandle,
        Flags,
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBD_InitializeDeviceEx(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    DeviceHandle: PUSB_DEVICE_HANDLE,
    DeviceDescriptorBuffer: PUCHAR,
    mut DeviceDescriptorBufferLength: ULONG,
    ConfigDescriptorBuffer: PUCHAR,
    mut ConfigDescriptorBufferLength: ULONG,
) -> NTSTATUS {
    let Status: NTSTATUS;
    let InitializeUsbDevice: PUSB_BUSIFFN_INITIALIZE_USB_DEVICE;
    let GetUsbDescriptors: PUSB_BUSIFFN_GET_USB_DESCRIPTORS;

    // DPRINT("USBD_InitializeDeviceEx: ... \n");

    InitializeUsbDevice = (*HubExtension).BusInterface.InitializeUsbDevice;
    GetUsbDescriptors = (*HubExtension).BusInterface.GetUsbDescriptors;

    if InitializeUsbDevice.is_none() || GetUsbDescriptors.is_none() {
        return STATUS_NOT_IMPLEMENTED;
    }

    Status = (InitializeUsbDevice.unwrap())(
        (*HubExtension).BusInterface.BusContext,
        DeviceHandle,
    );

    if !NT_SUCCESS(Status) {
        return Status;
    }

    return (GetUsbDescriptors.unwrap())(
        (*HubExtension).BusInterface.BusContext,
        DeviceHandle,
        DeviceDescriptorBuffer,
        core::ptr::addr_of_mut!(DeviceDescriptorBufferLength),
        ConfigDescriptorBuffer,
        core::ptr::addr_of_mut!(ConfigDescriptorBufferLength),
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBHUB_SetDeviceHandleData(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    UsbDevicePdo: PDEVICE_OBJECT,
    DeviceHandle: PVOID,
) {
    let SetDeviceHandleData: PUSB_BUSIFFN_SET_DEVHANDLE_DATA;

    // DPRINT("USBHUB_SetDeviceHandleData ... \n");

    SetDeviceHandleData = (*HubExtension).BusInterface.SetDeviceHandleData;

    if SetDeviceHandleData.is_none() {
        return;
    }

    (SetDeviceHandleData.unwrap())(
        (*HubExtension).BusInterface.BusContext,
        DeviceHandle,
        UsbDevicePdo,
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBHUB_FlushAllTransfers(HubExtension: PUSBHUB_FDO_EXTENSION) {
    let FlushTransfers: PUSB_BUSIFFN_FLUSH_TRANSFERS;

    // DPRINT("USBHUB_FlushAllTransfers ... \n");

    FlushTransfers = (*HubExtension).BusInterface.FlushTransfers;

    if FlushTransfers.is_some() {
        (FlushTransfers.unwrap())((*HubExtension).BusInterface.BusContext, core::ptr::null_mut());
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBD_GetDeviceInformationEx(
    PortExtension: PUSBHUB_PORT_PDO_EXTENSION,
    HubExtension: PUSBHUB_FDO_EXTENSION,
    Info: PUSB_NODE_CONNECTION_INFORMATION_EX,
    mut Length: ULONG,
    DeviceHandle: PUSB_DEVICE_HANDLE,
) -> NTSTATUS {
    const USB_HUB_TAG: u32 = 0x42554855; // 'USHB'

    let QueryDeviceInformation: PUSB_BUSIFFN_GET_DEVICE_INFORMATION;
    let mut DeviceInfo: PUSB_DEVICE_INFORMATION_0;
    let mut DeviceInfoLength: SIZE_T;
    let mut NodeInfo: PUSB_NODE_CONNECTION_INFORMATION_EX;
    let mut NodeInfoLength: SIZE_T;
    let mut PipeNumber: ULONG;
    let mut dummy: ULONG = 0;
    let mut Status: NTSTATUS;

    // DPRINT("USBD_GetDeviceInformationEx ... \n");

    QueryDeviceInformation = (*HubExtension).BusInterface.QueryDeviceInformation;

    if QueryDeviceInformation.is_none() {
        Status = STATUS_NOT_IMPLEMENTED;
        return Status;
    }

    let QueryDeviceInformation = QueryDeviceInformation.unwrap();

    DeviceInfoLength = core::mem::size_of::<USB_DEVICE_INFORMATION_0>() as SIZE_T;

    loop {
        DeviceInfo = ExAllocatePoolWithTag(PagedPool, DeviceInfoLength, USB_HUB_TAG)
            as PUSB_DEVICE_INFORMATION_0;

        if DeviceInfo.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        RtlZeroMemory(DeviceInfo as *mut _, DeviceInfoLength);

        (*DeviceInfo).InformationLevel = 0;

        Status = QueryDeviceInformation(
            (*HubExtension).BusInterface.BusContext,
            DeviceHandle,
            DeviceInfo as PVOID,
            DeviceInfoLength as u32,
            core::ptr::addr_of_mut!(dummy),
        );

        if Status != STATUS_BUFFER_TOO_SMALL {
            break;
        }

        DeviceInfoLength = (*DeviceInfo).ActualLength as SIZE_T;

        ExFreePoolWithTag(DeviceInfo as *mut _, USB_HUB_TAG);
    }

    NodeInfo = core::ptr::null_mut();
    NodeInfoLength = 0;

    if NT_SUCCESS(Status) {
        NodeInfoLength = (core::mem::size_of::<USB_NODE_CONNECTION_INFORMATION_EX>()
            - core::mem::size_of::<USB_PIPE_INFO>()) as SIZE_T
            + (*DeviceInfo).NumberOfOpenPipes as SIZE_T
                * core::mem::size_of::<USB_PIPE_INFO>() as SIZE_T;

        NodeInfo = ExAllocatePoolWithTag(PagedPool, NodeInfoLength, USB_HUB_TAG)
            as PUSB_NODE_CONNECTION_INFORMATION_EX;

        if NodeInfo.is_null() {
            ExFreePoolWithTag(DeviceInfo as *mut _, USB_HUB_TAG);
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        RtlZeroMemory(NodeInfo as *mut _, NodeInfoLength);

        (*NodeInfo).ConnectionIndex = (*Info).ConnectionIndex;

        RtlCopyMemory(
            core::ptr::addr_of_mut!((*NodeInfo).DeviceDescriptor) as *mut _,
            core::ptr::addr_of!((*DeviceInfo).DeviceDescriptor) as *const _,
            core::mem::size_of::<USB_DEVICE_DESCRIPTOR>() as SIZE_T,
        );

        (*NodeInfo).CurrentConfigurationValue = (*DeviceInfo).CurrentConfigurationValue;
        (*NodeInfo).Speed = (*DeviceInfo).DeviceSpeed as UCHAR;
        (*NodeInfo).DeviceIsHub =
            ((*PortExtension).PortPdoFlags & USBHUB_PDO_FLAG_HUB_DEVICE) as _;
        (*NodeInfo).DeviceAddress = (*DeviceInfo).DeviceAddress;
        (*NodeInfo).NumberOfOpenPipes = (*DeviceInfo).NumberOfOpenPipes;
        (*NodeInfo).ConnectionStatus = (*Info).ConnectionStatus;

        PipeNumber = 0;
        while PipeNumber < (*DeviceInfo).NumberOfOpenPipes {
            RtlCopyMemory(
                // `addr_of!(x.PipeList[n])` READS as address-taking but the index inside
                // the place expression still bounds-checks, and both PipeLists are
                // flexible arrays the caller over-allocates (`PipeList[ANYSIZE_ARRAY]`
                // in usbioctl.h, `PipeList[1]` in hubbusif.h) that bindgen models as
                // `[T; 1]`. The loop runs to NumberOfOpenPipes, so ANY device with more
                // than one open pipe panicked here. Walk each array by its own stride.
                (&raw mut (*NodeInfo).PipeList)
                    .cast::<USB_PIPE_INFO>()
                    .add(PipeNumber as usize) as *mut _,
                (&raw const (*DeviceInfo).PipeList)
                    .cast::<USB_PIPE_INFORMATION_0>()
                    .add(PipeNumber as usize) as *const _,
                core::mem::size_of::<USB_PIPE_INFO>() as SIZE_T,
            );
            PipeNumber += 1;
        }
    }

    ExFreePoolWithTag(DeviceInfo as *mut _, USB_HUB_TAG);

    if !NodeInfo.is_null() {
        if NodeInfoLength <= Length as SIZE_T {
            Length = NodeInfoLength as ULONG;
        } else {
            Status = STATUS_BUFFER_TOO_SMALL;
        }

        RtlCopyMemory(Info as *mut _, NodeInfo as *const _, Length as SIZE_T);

        ExFreePoolWithTag(NodeInfo as *mut _, USB_HUB_TAG);
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBD_RestoreDeviceEx(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    OldDeviceHandle: PUSB_DEVICE_HANDLE,
    NewDeviceHandle: PUSB_DEVICE_HANDLE,
) -> NTSTATUS {
    let RestoreUsbDevice: PUSB_BUSIFFN_RESTORE_DEVICE;
    let Status: NTSTATUS;

    RestoreUsbDevice = (*HubExtension).BusInterface.RestoreUsbDevice;

    if RestoreUsbDevice.is_some() {
        Status = (RestoreUsbDevice.unwrap())(
            (*HubExtension).BusInterface.BusContext,
            OldDeviceHandle,
            NewDeviceHandle,
        );
    } else {
        Status = STATUS_NOT_IMPLEMENTED;
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_AllocateWorkItem(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    OutHubIoWorkItem: *mut PUSBHUB_IO_WORK_ITEM,
    WorkerRoutine: PUSBHUB_WORKER_ROUTINE,
    BufferLength: SIZE_T,
    OutHubWorkItemBuffer: *mut PVOID,
    Type: WORK_QUEUE_TYPE,
) -> NTSTATUS {
    let HubIoWorkItem: PUSBHUB_IO_WORK_ITEM;
    let WorkItem: PIO_WORKITEM;
    let WorkItemBuffer: PVOID;

    // DPRINT("USBH_AllocateWorkItem: ... \n");

    if ((*HubExtension).HubFlags & USBHUB_FDO_FLAG_WITEM_INIT) == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    HubIoWorkItem = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<USBHUB_IO_WORK_ITEM>(),
        USB_HUB_TAG,
    ) as PUSBHUB_IO_WORK_ITEM;

    if HubIoWorkItem.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlZeroMemory(
        HubIoWorkItem as PVOID,
        core::mem::size_of::<USBHUB_IO_WORK_ITEM>(),
    );

    WorkItem = IoAllocateWorkItem((*HubExtension).Common.SelfDevice);

    (*HubIoWorkItem).HubWorkItem = WorkItem;

    if WorkItem.is_null() {
        ExFreePoolWithTag(HubIoWorkItem as PVOID, USB_HUB_TAG);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    if BufferLength != 0 && !OutHubWorkItemBuffer.is_null() {
        WorkItemBuffer = ExAllocatePoolWithTag(NonPagedPool, BufferLength, USB_HUB_TAG);

        (*HubIoWorkItem).HubWorkItemBuffer = WorkItemBuffer;

        if WorkItemBuffer.is_null() {
            IoFreeWorkItem((*HubIoWorkItem).HubWorkItem);
            ExFreePoolWithTag(HubIoWorkItem as PVOID, USB_HUB_TAG);

            return STATUS_INSUFFICIENT_RESOURCES;
        }

        RtlZeroMemory(WorkItemBuffer, BufferLength);
    } else {
        (*HubIoWorkItem).HubWorkItemBuffer = core::ptr::null_mut();
    }

    (*HubIoWorkItem).HubWorkItemType = Type;
    (*HubIoWorkItem).HubExtension = HubExtension;
    (*HubIoWorkItem).HubWorkerRoutine = WorkerRoutine;

    if !OutHubIoWorkItem.is_null() {
        *OutHubIoWorkItem = HubIoWorkItem;
    }

    if !OutHubWorkItemBuffer.is_null() {
        *OutHubWorkItemBuffer = (*HubIoWorkItem).HubWorkItemBuffer;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn USBH_Worker(
    DeviceObject: PDEVICE_OBJECT,
    Context: PVOID,
) {
    let HubIoWorkItem: PUSBHUB_IO_WORK_ITEM;
    let HubExtension: PUSBHUB_FDO_EXTENSION;
    let mut OldIrql: KIRQL = 0;
    let WorkItem: PIO_WORKITEM;

    // DPRINT("USBH_Worker: HubIoWorkItem - %p\n", Context);

    HubIoWorkItem = Context as PUSBHUB_IO_WORK_ITEM;

    InterlockedDecrement(core::ptr::addr_of_mut!((*HubIoWorkItem).HubWorkerQueued));

    HubExtension = (*HubIoWorkItem).HubExtension;
    WorkItem = (*HubIoWorkItem).HubWorkItem;

    ((*HubIoWorkItem).HubWorkerRoutine).unwrap()(
        (*HubIoWorkItem).HubExtension,
        (*HubIoWorkItem).HubWorkItemBuffer,
    );

    KeAcquireSpinLock(
        core::ptr::addr_of_mut!((*HubExtension).WorkItemSpinLock) as PKSPIN_LOCK,
        core::ptr::addr_of_mut!(OldIrql),
    );
    RemoveEntryList(core::ptr::addr_of_mut!((*HubIoWorkItem).HubWorkItemLink));
    KeReleaseSpinLock(
        core::ptr::addr_of_mut!((*HubExtension).WorkItemSpinLock) as PKSPIN_LOCK,
        OldIrql,
    );

    if !(*HubIoWorkItem).HubWorkItemBuffer.is_null() {
        ExFreePoolWithTag((*HubIoWorkItem).HubWorkItemBuffer, USB_HUB_TAG);
    }

    ExFreePoolWithTag(HubIoWorkItem as PVOID, USB_HUB_TAG);

    if InterlockedDecrement(core::ptr::addr_of_mut!((*HubExtension).PendingRequestCount)) == 0 {
        KeSetEvent(core::ptr::addr_of_mut!((*HubExtension).PendingRequestEvent) as PVOID, EVENT_INCREMENT as i32, 0);
    }

    IoFreeWorkItem(WorkItem);

    // DPRINT("USBH_Worker: HubIoWorkItem %p complete\n", Context);
}

#[no_mangle]
pub unsafe extern "system" fn USBH_QueueWorkItem(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    HubIoWorkItem: PUSBHUB_IO_WORK_ITEM,
) {
    InterlockedIncrement(core::ptr::addr_of_mut!((*HubExtension).PendingRequestCount));
    InterlockedIncrement(core::ptr::addr_of_mut!((*HubIoWorkItem).HubWorkerQueued));

    ExInterlockedInsertTailList(
        core::ptr::addr_of_mut!((*HubExtension).WorkItemList),
        core::ptr::addr_of_mut!((*HubIoWorkItem).HubWorkItemLink),
        core::ptr::addr_of_mut!((*HubExtension).WorkItemSpinLock) as ntkrnl_sys::ddk::PKSPIN_LOCK,
    );

    IoQueueWorkItem(
        (*HubIoWorkItem).HubWorkItem,
        cb!(USBH_Worker),
        (*HubIoWorkItem).HubWorkItemType,
        HubIoWorkItem as *mut core::ffi::c_void,
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBH_FreeWorkItem(HubIoWorkItem: PUSBHUB_IO_WORK_ITEM) {
    let WorkItem: PIO_WORKITEM;

    // DPRINT("USBH_FreeWorkItem: ... \n");

    WorkItem = (*HubIoWorkItem).HubWorkItem;

    if !(*HubIoWorkItem).HubWorkItemBuffer.is_null() {
        ExFreePoolWithTag((*HubIoWorkItem).HubWorkItemBuffer, USB_HUB_TAG);
    }

    ExFreePoolWithTag(HubIoWorkItem as *mut core::ffi::c_void, USB_HUB_TAG);

    IoFreeWorkItem(WorkItem);
}

#[no_mangle]
pub unsafe extern "system" fn USBHUB_RootHubCallBack(Context: PVOID) {
    let HubExtension: PUSBHUB_FDO_EXTENSION;

    HubExtension = Context as PUSBHUB_FDO_EXTENSION;

    if !(*HubExtension).SCEIrp.is_null() {
        (*HubExtension).HubFlags |= USBHUB_FDO_FLAG_DO_ENUMERATION
            | USBHUB_FDO_FLAG_NOT_ENUMERATED;

        USBH_SubmitStatusChangeTransfer(HubExtension);

        IoInvalidateDeviceRelations((*HubExtension).LowerPDO, BusRelations);
    } else {
        (*HubExtension).HubFlags |= USBHUB_FDO_FLAG_DO_ENUMERATION;
    }

    KeSetEvent(core::ptr::addr_of_mut!((*HubExtension).RootHubNotificationEvent) as PVOID, EVENT_INCREMENT as i32, 0);
}

#[no_mangle]
pub unsafe extern "system" fn USBD_RegisterRootHubCallBack(
    HubExtension: PUSBHUB_FDO_EXTENSION,
) -> NTSTATUS {
    let RootHubInitNotification: PUSB_BUSIFFN_ROOTHUB_INIT_NOTIFY;

    // DPRINT("USBD_RegisterRootHubCallBack: ... \n");

    RootHubInitNotification = (*HubExtension).BusInterface.RootHubInitNotification;

    if RootHubInitNotification.is_none() {
        return STATUS_NOT_IMPLEMENTED;
    }

    KeClearEvent(core::ptr::addr_of_mut!((*HubExtension).RootHubNotificationEvent) as PVOID);

    return (RootHubInitNotification.unwrap())(
        (*HubExtension).BusInterface.BusContext,
        HubExtension as *mut core::ffi::c_void,
        cb!(USBHUB_RootHubCallBack),
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBD_UnRegisterRootHubCallBack(HubExtension: PUSBHUB_FDO_EXTENSION) -> NTSTATUS {
    let RootHubInitNotification: PUSB_BUSIFFN_ROOTHUB_INIT_NOTIFY;
    let Status: NTSTATUS;

    RootHubInitNotification = (*HubExtension).BusInterface.RootHubInitNotification;

    if RootHubInitNotification.is_none() {
        return STATUS_NOT_IMPLEMENTED;
    }

    Status = (RootHubInitNotification.unwrap())(
        (*HubExtension).BusInterface.BusContext,
        core::ptr::null_mut(),
        None,
    );

    if !NT_SUCCESS(Status) {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!((*HubExtension).RootHubNotificationEvent) as *mut _,
            _KWAIT_REASON_Executive,
            0i32 as i8,
            0,
            core::ptr::null_mut(),
        );
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_HubSetDWakeCompletion(
    DeviceObject: PDEVICE_OBJECT,
    MinorFunction: UCHAR,
    PowerState: POWER_STATE,
    Context: PVOID,
    IoStatus: PIO_STATUS_BLOCK,
) {
    // DPRINT("USBH_HubSetDWakeCompletion: ... \n");
    KeSetEvent(Context as PVOID, IO_NO_INCREMENT as KPRIORITY, 0);
}

#[no_mangle]
pub unsafe extern "system" fn USBH_HubQueuePortIdleIrps(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    IdleList: PLIST_ENTRY,
) {
    let mut PortDevice: PDEVICE_OBJECT;
    let mut PortExtension: PUSBHUB_PORT_PDO_EXTENSION;
    let mut IdleIrp: PIRP;
    let mut HubIdleIrp: PIRP;
    let mut NumPorts: ULONG;
    let mut Port: ULONG;
    let mut Irql: KIRQL = 0;

    // DPRINT("USBH_HubQueuePortIdleIrps ... \n");

    InitializeListHead(IdleList);

    IoAcquireCancelSpinLock(core::ptr::addr_of_mut!(Irql));

    NumPorts = (*(*HubExtension).HubDescriptor).bNumberOfPorts as ULONG;

    Port = 0;
    while Port < NumPorts {
        PortDevice = (*(*HubExtension).PortData.add(Port as usize)).DeviceObject;

        if !PortDevice.is_null() {
            PortExtension = (*PortDevice).DeviceExtension as PUSBHUB_PORT_PDO_EXTENSION;

            IdleIrp = (*PortExtension).IdleNotificationIrp;
            (*PortExtension).IdleNotificationIrp = core::ptr::null_mut();

            if !IdleIrp.is_null() && !IoSetCancelRoutine(IdleIrp, None).is_none() {
                // DPRINT1("USBH_HubQueuePortIdleIrps: IdleIrp != NULL. FIXME\n");
                DbgBreakPoint();
            }
        }

        Port += 1;
    }

    if ((*HubExtension).HubFlags & USBHUB_FDO_FLAG_WAIT_IDLE_REQUEST) != 0 {
        HubIdleIrp = (*HubExtension).PendingIdleIrp;
        (*HubExtension).PendingIdleIrp = core::ptr::null_mut();
    } else {
        HubIdleIrp = core::ptr::null_mut();
    }

    IoReleaseCancelSpinLock(Irql);

    if !HubIdleIrp.is_null() {
        USBH_HubCancelIdleIrp(HubExtension, HubIdleIrp);
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBH_HubCompleteQueuedPortIdleIrps(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    IdleList: PLIST_ENTRY,
    NtStatus: NTSTATUS,
) {
    while !IsListEmpty(IdleList) {
        DbgBreakPoint();
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBH_FlushPortPwrList(HubExtension: PUSBHUB_FDO_EXTENSION) {
    let mut PortDevice: PDEVICE_OBJECT;
    let mut PortExtension: PUSBHUB_PORT_PDO_EXTENSION;
    let mut Entry: PLIST_ENTRY;
    let mut Port: ULONG;

    // DPRINT("USBH_FlushPortPwrList ... \n");

    InterlockedIncrement(core::ptr::addr_of_mut!((*HubExtension).PendingRequestCount) as *mut i32);

    KeWaitForSingleObject(
        core::ptr::addr_of_mut!((*HubExtension).ResetDeviceSemaphore) as *mut core::ffi::c_void,
        _KWAIT_REASON_Executive,
        0i32 as i8,
        0,
        core::ptr::null_mut(),
    );

    Port = 0;
    while Port < (*(*HubExtension).HubDescriptor).bNumberOfPorts as ULONG {
        PortDevice = (*HubExtension).PortData.offset(Port as isize).read().DeviceObject;

        if PortDevice.is_null() {
            Port += 1;
            continue;
        }

        PortExtension = (*PortDevice).DeviceExtension as PUSBHUB_PORT_PDO_EXTENSION;

        InterlockedExchange(core::ptr::addr_of_mut!((*PortExtension).StateBehindD2) as *mut i32, 0);

        loop {
            Entry = ExInterlockedRemoveHeadList(
                core::ptr::addr_of_mut!((*PortExtension).PortPowerList),
                core::ptr::addr_of_mut!((*PortExtension).PortPowerListSpinLock) as ntkrnl_sys::ddk::PKSPIN_LOCK,
            );

            if Entry.is_null() {
                break;
            }

            // DPRINT1("USBH_FlushPortPwrList: PortPowerList FIXME\n");
            DbgBreakPoint();
        }

        Port += 1;
    }

    KeReleaseSemaphore(
        core::ptr::addr_of_mut!((*HubExtension).ResetDeviceSemaphore),
        LOW_REALTIME_PRIORITY as KPRIORITY,
        1,
        0,
    );

    if InterlockedDecrement(core::ptr::addr_of_mut!((*HubExtension).PendingRequestCount) as *mut i32) == 0 {
        KeSetEvent(core::ptr::addr_of_mut!((*HubExtension).PendingRequestEvent) as PVOID, IO_NO_INCREMENT as i32, 0);
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBH_HubCompletePortIdleIrps(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    NtStatus: NTSTATUS,
) {
    let mut IdleList: LIST_ENTRY = core::mem::zeroed();

    if (*HubExtension).HubFlags & USBHUB_FDO_FLAG_DEVICE_STARTED != 0 {
        USBH_HubQueuePortIdleIrps(HubExtension, core::ptr::addr_of_mut!(IdleList));

        USBH_HubCompleteQueuedPortIdleIrps(
            HubExtension,
            core::ptr::addr_of_mut!(IdleList),
            NtStatus,
        );

        USBH_FlushPortPwrList(HubExtension);
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBH_HubCancelIdleIrp(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    IdleIrp: PIRP,
) {
    IoCancelIrp(IdleIrp);

    // IdleRequestLock arbitrates which of cancel / completion frees the idle
    // IRP: whoever swaps second frees it. The swap was a plain read-then-write
    // on both sides, so a racing cancel and completion could both see 0 (IRP
    // leaked) or both see 1 (double IoFreeIrp).
    if InterlockedExchange(core::ptr::addr_of_mut!((*HubExtension).IdleRequestLock), 1) != 0 {
        IoFreeIrp(IdleIrp);
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBH_CheckIdleAbort(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    IsWait: BOOLEAN,
    IsExtCheck: BOOLEAN,
) -> BOOLEAN {
    let mut PdoDevice: PDEVICE_OBJECT;
    let mut PortExtension: PUSBHUB_PORT_PDO_EXTENSION;
    let mut PortData: PUSBHUB_PORT_DATA;
    let mut Port: ULONG;
    let mut Result: BOOLEAN = 0;

    InterlockedIncrement(core::ptr::addr_of_mut!((*HubExtension).PendingRequestCount));

    if IsWait == 1 {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!((*HubExtension).ResetDeviceSemaphore) as *mut _,
            _KWAIT_REASON_Executive,
            0i32 as i8,
            0,
            core::ptr::null_mut(),
        );
    }

    PortData = (*HubExtension).PortData;

    'wait: loop {
        Port = 0;
        while Port < (*(*HubExtension).HubDescriptor).bNumberOfPorts as ULONG {
            PdoDevice = (*PortData.add(Port as usize)).DeviceObject;

            if !PdoDevice.is_null() {
                PortExtension = (*PdoDevice).DeviceExtension as PUSBHUB_PORT_PDO_EXTENSION;

                if (*PortExtension).PoRequestCounter != 0 {
                    Result = 1;
                    break 'wait;
                }
            }
            Port = Port + 1;
        }

        if IsExtCheck == 1 {
            PortData = (*HubExtension).PortData;

            Port = 0;
            while Port < (*(*HubExtension).HubDescriptor).bNumberOfPorts as ULONG {
                PdoDevice = (*PortData.add(Port as usize)).DeviceObject;

                if !PdoDevice.is_null() {
                    PortExtension = (*PdoDevice).DeviceExtension as PUSBHUB_PORT_PDO_EXTENSION;
                    InterlockedExchange(
                        core::ptr::addr_of_mut!((*PortExtension).StateBehindD2),
                        0,
                    );
                }
                Port = Port + 1;
            }
        }

        break 'wait;
    }

    if IsWait == 1 {
        KeReleaseSemaphore(
            core::ptr::addr_of_mut!((*HubExtension).ResetDeviceSemaphore) as *mut _,
            LOW_REALTIME_PRIORITY as KPRIORITY,
            1,
            0,
        );
    }

    if InterlockedDecrement(core::ptr::addr_of_mut!((*HubExtension).PendingRequestCount)) == 0 {
        KeSetEvent(core::ptr::addr_of_mut!((*HubExtension).PendingRequestEvent) as PVOID, EVENT_INCREMENT as i32, 0);
    }

    return Result;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_FdoWaitWakeIrpCompletion(
    DeviceObject: PDEVICE_OBJECT,
    MinorFunction: UCHAR,
    PowerState: POWER_STATE,
    Context: PVOID,
    IoStatus: PIO_STATUS_BLOCK,
) {
    // DPRINT("USBH_FdoWaitWakeIrpCompletion ... \n");
}

#[no_mangle]
pub unsafe extern "system" fn USBH_FdoSubmitWaitWakeIrp(
    HubExtension: PUSBHUB_FDO_EXTENSION,
) -> NTSTATUS {
    let mut PowerState: POWER_STATE = core::mem::zeroed();
    let Status: NTSTATUS;
    let mut Irp: PIRP = core::ptr::null_mut();
    let mut Irql: KIRQL = 0;

    PowerState.SystemState = (*HubExtension).SystemWake;
    (*HubExtension).HubFlags |= USBHUB_FDO_FLAG_PENDING_WAKE_IRP;

    InterlockedIncrement(core::ptr::addr_of_mut!((*HubExtension).PendingRequestCount));
    InterlockedExchange(core::ptr::addr_of_mut!((*HubExtension).FdoWaitWakeLock), 0);

    Status = PoRequestPowerIrp(
        (*HubExtension).LowerPDO,
        IRP_MN_WAIT_WAKE as UCHAR,
        PowerState,
        cb!(USBH_FdoWaitWakeIrpCompletion),
        HubExtension as PVOID,
        core::ptr::addr_of_mut!(Irp),
    );

    IoAcquireCancelSpinLock(core::ptr::addr_of_mut!(Irql));

    if Status == STATUS_PENDING {
        if (*HubExtension).HubFlags & USBHUB_FDO_FLAG_PENDING_WAKE_IRP != 0 {
            (*HubExtension).PendingWakeIrp = Irp;
        }
    } else {
        (*HubExtension).HubFlags &= !USBHUB_FDO_FLAG_PENDING_WAKE_IRP;

        if InterlockedDecrement(core::ptr::addr_of_mut!((*HubExtension).PendingRequestCount)) == 0 {
            KeSetEvent(core::ptr::addr_of_mut!((*HubExtension).PendingRequestEvent) as PVOID, EVENT_INCREMENT as i32, 0);
        }
    }

    IoReleaseCancelSpinLock(Irql);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_FdoIdleNotificationCallback(Context: PVOID) {
    let HubExtension: PUSBHUB_FDO_EXTENSION;
    let mut PortData: PUSBHUB_PORT_DATA;
    let mut PortDevice: PDEVICE_OBJECT;
    let mut PortExtension: PUSBHUB_PORT_PDO_EXTENSION;
    let mut Irp: PIRP = core::ptr::null_mut();
    let mut IdleIrp: PIRP;
    let mut PowerState: POWER_STATE = core::mem::zeroed();
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Port: ULONG;
    let mut IoStack: PIO_STACK_LOCATION;
    let mut CallbackInfo: PUSB_IDLE_CALLBACK_INFO;
    let mut IsReady: BOOLEAN;
    let mut OldIrql: KIRQL = 0;
    let mut Status: NTSTATUS;

    HubExtension = Context as PUSBHUB_FDO_EXTENSION;

    if (*HubExtension).HubFlags
        & (USBHUB_FDO_FLAG_ENUM_POST_RECOVER
            | USBHUB_FDO_FLAG_WAKEUP_START
            | USBHUB_FDO_FLAG_DEVICE_REMOVED
            | USBHUB_FDO_FLAG_STATE_CHANGING
            | USBHUB_FDO_FLAG_ESD_RECOVERING
            | USBHUB_FDO_FLAG_DEVICE_FAILED
            | USBHUB_FDO_FLAG_DEVICE_STOPPING)
        != 0
    {
        DbgBreakPoint();
        return;
    }

    (*HubExtension).HubFlags |= USBHUB_FDO_FLAG_GOING_IDLE;

    if ((*HubExtension).HubFlags & USBHUB_FDO_FLAG_PENDING_WAKE_IRP) == 0 {
        Status = USBH_FdoSubmitWaitWakeIrp(HubExtension);

        if Status != STATUS_PENDING {
            DbgBreakPoint();
            (*HubExtension).HubFlags &= !USBHUB_FDO_FLAG_GOING_IDLE;
            return;
        }
    }

    InterlockedIncrement(core::ptr::addr_of_mut!((*HubExtension).PendingRequestCount));

    KeWaitForSingleObject(
        core::ptr::addr_of_mut!((*HubExtension).ResetDeviceSemaphore) as PVOID,
        _KWAIT_REASON_Executive,
        0i32 as i8,
        FALSE as BOOLEAN,
        core::ptr::null_mut(),
    );

    PortData = (*HubExtension).PortData;
    IsReady = TRUE as BOOLEAN;

    'idlehub: loop {
        Port = 0;
        while Port < (*(*HubExtension).HubDescriptor).bNumberOfPorts as ULONG {
            PortDevice = (*PortData.add(Port as usize)).DeviceObject;

            if !PortDevice.is_null() {
                PortExtension = (*PortDevice).DeviceExtension as PUSBHUB_PORT_PDO_EXTENSION;

                IdleIrp = (*PortExtension).IdleNotificationIrp;

                if IdleIrp.is_null() {
                    IsReady = FALSE as BOOLEAN;
                    break 'idlehub;
                }

                IoStack = IoGetCurrentIrpStackLocation(IdleIrp);

                CallbackInfo = (*IoStack).Parameters.DeviceIoControl.Type3InputBuffer
                    as PUSB_IDLE_CALLBACK_INFO;

                if CallbackInfo.is_null() {
                    IsReady = FALSE as BOOLEAN;
                    break 'idlehub;
                }

                if (*CallbackInfo).IdleCallback.is_none() {
                    IsReady = FALSE as BOOLEAN;
                    break 'idlehub;
                }

                if (*PortExtension).PendingSystemPoRequest != 0 {
                    IsReady = FALSE as BOOLEAN;
                    break 'idlehub;
                }

                if {
                    let dst = core::ptr::addr_of_mut!((*PortExtension).StateBehindD2);
                    let old = *dst;
                    if old == 0 {
                        *dst = 1;
                    }
                    old
                } != 0
                {
                    IsReady = FALSE as BOOLEAN;
                    break 'idlehub;
                }

                ((*CallbackInfo).IdleCallback.unwrap())((*CallbackInfo).IdleContext);

                if (*PortExtension).CurrentPowerState.DeviceState == PowerDeviceD0 {
                    IsReady = FALSE as BOOLEAN;
                    break 'idlehub;
                }
            }

            Port += 1;
        }

        if ((*HubExtension).HubFlags & USBHUB_FDO_FLAG_DEVICE_STOPPING) == 0
            && (USBH_CheckIdleAbort(HubExtension, FALSE as BOOLEAN, FALSE as BOOLEAN)
                == TRUE as BOOLEAN)
        {
            IsReady = FALSE as BOOLEAN;
        }

        break 'idlehub;
    }

    KeReleaseSemaphore(
        core::ptr::addr_of_mut!((*HubExtension).ResetDeviceSemaphore) as PRKSEMAPHORE,
        LOW_REALTIME_PRIORITY as KPRIORITY,
        1,
        FALSE as BOOLEAN,
    );

    if InterlockedDecrement(core::ptr::addr_of_mut!((*HubExtension).PendingRequestCount)) == 0 {
        KeSetEvent(core::ptr::addr_of_mut!((*HubExtension).PendingRequestEvent) as PVOID, EVENT_INCREMENT as LONG, FALSE as BOOLEAN);
    }

    if IsReady == 0 || ((*HubExtension).HubFlags & USBHUB_FDO_FLAG_DEVICE_SUSPENDED) != 0 {
        (*HubExtension).HubFlags &=
            !(USBHUB_FDO_FLAG_DEVICE_SUSPENDED | USBHUB_FDO_FLAG_GOING_IDLE);

        /* Aborting Idle for Hub */
        IoAcquireCancelSpinLock(core::ptr::addr_of_mut!(OldIrql));

        if !(*HubExtension).PendingIdleIrp.is_null() {
            Irp = (*HubExtension).PendingIdleIrp;
            (*HubExtension).PendingIdleIrp = core::ptr::null_mut();
        }

        IoReleaseCancelSpinLock(OldIrql);

        if !Irp.is_null() {
            USBH_HubCancelIdleIrp(HubExtension, Irp);
        }

        DbgBreakPoint();
        USBH_HubCompletePortIdleIrps(HubExtension, STATUS_CANCELLED);
    } else {
        PowerState.DeviceState = (*HubExtension).DeviceWake;

        KeWaitForSingleObject(
            core::ptr::addr_of_mut!((*HubExtension).IdleSemaphore) as PVOID,
            _KWAIT_REASON_Executive,
            0i32 as i8,
            FALSE as BOOLEAN,
            core::ptr::null_mut(),
        );

        (*HubExtension).HubFlags &= !USBHUB_FDO_FLAG_GOING_IDLE;
        (*HubExtension).HubFlags |= USBHUB_FDO_FLAG_DO_SUSPENSE;

        KeInitializeEvent(
            core::ptr::addr_of_mut!(Event),
            _EVENT_TYPE_NotificationEvent,
            FALSE as BOOLEAN,
        );

        Status = PoRequestPowerIrp(
            (*HubExtension).LowerPDO,
            IRP_MN_SET_POWER as UCHAR,
            PowerState,
            cb!(USBH_HubSetDWakeCompletion),
            core::ptr::addr_of_mut!(Event) as PVOID,
            core::ptr::null_mut(),
        );

        if Status == STATUS_PENDING {
            KeWaitForSingleObject(
                core::ptr::addr_of_mut!(Event) as PVOID,
                _KWAIT_REASON_Executive,
                0i32 as i8,
                FALSE as BOOLEAN,
                core::ptr::null_mut(),
            );
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBH_CompletePortIdleIrpsWorker(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    Context: PVOID,
) {
    let IdlePortContext: PUSBHUB_IDLE_PORT_CONTEXT;
    let NtStatus: NTSTATUS;
    let Status: NTSTATUS;
    let IsFlush: BOOLEAN = 0;

    IdlePortContext = Context as PUSBHUB_IDLE_PORT_CONTEXT;
    NtStatus = (*IdlePortContext).Status;

    USBH_HubCompleteQueuedPortIdleIrps(
        HubExtension,
        core::ptr::addr_of_mut!((*IdlePortContext).PwrList),
        NtStatus,
    );

    Status = STATUS_NOT_IMPLEMENTED; // USBH_RegQueryFlushPortPowerIrpsFlag(&IsFlush);

    if NT_SUCCESS(Status) {
        if IsFlush != 0 {
            USBH_FlushPortPwrList(HubExtension);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBH_IdleCompletePowerHubWorker(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    Context: PVOID,
) {

    let HubWorkItemBuffer: PUSBHUB_IDLE_HUB_CONTEXT;

    // DPRINT("USBH_IdleCompletePowerHubWorker ... \n");

    if !HubExtension.is_null()
        && (*HubExtension).CurrentPowerState.DeviceState != PowerDeviceD0
        && ((*HubExtension).HubFlags & USBHUB_FDO_FLAG_DEVICE_STARTED) != 0
    {
        USBH_HubSetD0(HubExtension);
    }

    HubWorkItemBuffer = Context as PUSBHUB_IDLE_HUB_CONTEXT;

    USBH_HubCompletePortIdleIrps(HubExtension, (*HubWorkItemBuffer).Status);
}

#[no_mangle]
pub unsafe extern "system" fn USBH_FdoIdleNotificationRequestComplete(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    let HubExtension: PUSBHUB_FDO_EXTENSION;
    let NtStatus: NTSTATUS;
    let IdleIrp: PVOID;
    let mut Irql: KIRQL = 0;
    let Status: NTSTATUS;
    let mut HubIoWorkItem: PUSBHUB_IO_WORK_ITEM = core::ptr::null_mut();

    IoAcquireCancelSpinLock(core::ptr::addr_of_mut!(Irql));

    HubExtension = Context as PUSBHUB_FDO_EXTENSION;
    (*HubExtension).HubFlags &= !USBHUB_FDO_FLAG_WAIT_IDLE_REQUEST;

    IdleIrp = InterlockedExchangePointer(
        core::ptr::addr_of_mut!((*HubExtension).PendingIdleIrp) as *mut PVOID,
        core::ptr::null_mut(),
    );

    if InterlockedDecrement(core::ptr::addr_of_mut!((*HubExtension).PendingRequestCount)) == 0 {
        KeSetEvent(core::ptr::addr_of_mut!((*HubExtension).PendingRequestEvent) as PVOID, EVENT_INCREMENT as LONG, 0);
    }

    IoReleaseCancelSpinLock(Irql);

    NtStatus = (*Irp).IoStatus.__bindgen_anon_1.Status;

    if !NT_SUCCESS(NtStatus)
        && NtStatus != STATUS_POWER_STATE_INVALID
        && ((*HubExtension).HubFlags & USBHUB_FDO_FLAG_DEVICE_REMOVED) == 0
        && ((*HubExtension).HubFlags & USBHUB_FDO_FLAG_DEVICE_STOPPED) == 0
    {
        if (*HubExtension).CurrentPowerState.DeviceState == PowerDeviceD0 {
            let mut HubWorkItemBuffer: PUSBHUB_IDLE_PORT_CONTEXT = core::ptr::null_mut();

            Status = USBH_AllocateWorkItem(
                HubExtension,
                core::ptr::addr_of_mut!(HubIoWorkItem),
                cb!(USBH_CompletePortIdleIrpsWorker),
                core::mem::size_of::<USBHUB_IDLE_PORT_CONTEXT>() as SIZE_T,
                core::ptr::addr_of_mut!(HubWorkItemBuffer) as *mut PVOID,
                DelayedWorkQueue,
            );

            if NT_SUCCESS(Status) {
                (*HubWorkItemBuffer).Status = NtStatus;

                USBH_HubQueuePortIdleIrps(
                    HubExtension,
                    core::ptr::addr_of_mut!((*HubWorkItemBuffer).PwrList),
                );

                USBH_QueueWorkItem(HubExtension, HubIoWorkItem);
            }
        } else {
            let mut HubWorkItemBuffer: PUSBHUB_IDLE_HUB_CONTEXT = core::ptr::null_mut();

            Status = USBH_AllocateWorkItem(
                HubExtension,
                core::ptr::addr_of_mut!(HubIoWorkItem),
                cb!(USBH_IdleCompletePowerHubWorker),
                core::mem::size_of::<USBHUB_IDLE_HUB_CONTEXT>() as SIZE_T,
                core::ptr::addr_of_mut!(HubWorkItemBuffer) as *mut PVOID,
                DelayedWorkQueue,
            );

            if NT_SUCCESS(Status) {
                (*HubWorkItemBuffer).Status = NtStatus;
                USBH_QueueWorkItem(HubExtension, HubIoWorkItem);
            }
        }
    }

    // InterlockedExchange: the other half of USBH_HubCancelIdleIrp's handshake.
    if !IdleIrp.is_null()
        || InterlockedExchange(core::ptr::addr_of_mut!((*HubExtension).IdleRequestLock), 1) != 0
    {
        IoFreeIrp(Irp);
    }

    STATUS_MORE_PROCESSING_REQUIRED
}

#[no_mangle]
pub unsafe extern "system" fn USBH_FdoSubmitIdleRequestIrp(
    HubExtension: PUSBHUB_FDO_EXTENSION,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let HubFlags: ULONG;
    let LowerPDO: PDEVICE_OBJECT;
    let Irp: PIRP;
    let IoStack: PIO_STACK_LOCATION;
    let mut Irql: KIRQL = 0;

    // DPRINT("USBH_FdoSubmitIdleRequestIrp: HubExtension - %p, PendingIdleIrp - %p\n",
    //        HubExtension, HubExtension->PendingIdleIrp);

    if !(*HubExtension).PendingIdleIrp.is_null() {
        Status = STATUS_DEVICE_BUSY;
        KeSetEvent(core::ptr::addr_of_mut!((*HubExtension).IdleEvent) as PVOID, EVENT_INCREMENT as i32, 0);
        return Status;
    }

    HubFlags = (*HubExtension).HubFlags;

    if (HubFlags & USBHUB_FDO_FLAG_DEVICE_STOPPING) != 0
        || (HubFlags & USBHUB_FDO_FLAG_DEVICE_REMOVED) != 0
    {
        (*HubExtension).HubFlags = HubFlags & !USBHUB_FDO_FLAG_WAIT_IDLE_REQUEST;
        KeSetEvent(core::ptr::addr_of_mut!((*HubExtension).IdleEvent) as PVOID, EVENT_INCREMENT as i32, 0);
        return STATUS_DEVICE_REMOVED;
    }

    LowerPDO = (*HubExtension).LowerPDO;

    (*HubExtension).IdleCallbackInfo.IdleCallback =
        cb!(USBH_FdoIdleNotificationCallback);
    (*HubExtension).IdleCallbackInfo.IdleContext = HubExtension as *mut core::ffi::c_void;

    Irp = IoAllocateIrp((*LowerPDO).StackSize, 0);

    if Irp.is_null() {
        (*HubExtension).HubFlags &= !USBHUB_FDO_FLAG_WAIT_IDLE_REQUEST;
        Status = STATUS_INSUFFICIENT_RESOURCES;

        KeSetEvent(core::ptr::addr_of_mut!((*HubExtension).IdleEvent) as PVOID, EVENT_INCREMENT as i32, 0);
        return Status;
    }

    IoStack = IoGetNextIrpStackLocation(Irp);

    (*IoStack).MajorFunction = IRP_MJ_INTERNAL_DEVICE_CONTROL as u8;

    (*IoStack).Parameters.DeviceIoControl.InputBufferLength =
        core::mem::size_of::<USB_IDLE_CALLBACK_INFO>() as ULONG;
    (*IoStack).Parameters.DeviceIoControl.IoControlCode =
        IOCTL_INTERNAL_USB_SUBMIT_IDLE_NOTIFICATION;
    (*IoStack).Parameters.DeviceIoControl.Type3InputBuffer =
        core::ptr::addr_of_mut!((*HubExtension).IdleCallbackInfo) as *mut core::ffi::c_void;

    IoSetCompletionRoutine(
        Irp,
        cb!(USBH_FdoIdleNotificationRequestComplete),
        HubExtension as *mut core::ffi::c_void,
        1,
        1,
        1,
    );

    InterlockedIncrement(core::ptr::addr_of_mut!((*HubExtension).PendingRequestCount));
    InterlockedExchange(core::ptr::addr_of_mut!((*HubExtension).IdleRequestLock), 0);

    (*HubExtension).HubFlags &=
        !(USBHUB_FDO_FLAG_DEVICE_SUSPENDED | USBHUB_FDO_FLAG_GOING_IDLE);

    Status = IoCallDriver((*HubExtension).LowerPDO, Irp);

    IoAcquireCancelSpinLock(core::ptr::addr_of_mut!(Irql));

    if Status == STATUS_PENDING
        && ((*HubExtension).HubFlags & USBHUB_FDO_FLAG_WAIT_IDLE_REQUEST) != 0
    {
        (*HubExtension).PendingIdleIrp = Irp;
    }

    IoReleaseCancelSpinLock(Irql);

    KeSetEvent(core::ptr::addr_of_mut!((*HubExtension).IdleEvent) as PVOID, EVENT_INCREMENT as i32, 0);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_CheckHubIdle(HubExtension: PUSBHUB_FDO_EXTENSION) {
    let mut PdoDevice: PDEVICE_OBJECT;
    let mut PortExtension: PUSBHUB_PORT_PDO_EXTENSION;
    let mut PortData: PUSBHUB_PORT_DATA;
    let mut HubFlags: ULONG;
    let mut Port: ULONG;
    let mut Irql: KIRQL = 0;
    let mut IsHubIdle: BOOLEAN = 0;
    let mut IsAllPortsIdle: BOOLEAN;
    let mut IsHubCheck: BOOLEAN = 1;

    // DPRINT("USBH_CheckHubIdle: FIXME !!! HubExtension - %p\n", HubExtension);

    return; //HACK: delete it line after fixing Power Manager!!!

    #[allow(unreachable_code)]
    {
        KeAcquireSpinLock(
            core::ptr::addr_of_mut!((*HubExtension).CheckIdleSpinLock) as PKSPIN_LOCK,
            core::ptr::addr_of_mut!(Irql),
        );

        if (*HubExtension).HubFlags & USBHUB_FDO_FLAG_CHECK_IDLE_LOCK != 0 {
            KeReleaseSpinLock(core::ptr::addr_of_mut!((*HubExtension).CheckIdleSpinLock) as PKSPIN_LOCK, Irql);
            return;
        }

        (*HubExtension).HubFlags |= USBHUB_FDO_FLAG_CHECK_IDLE_LOCK;
        KeReleaseSpinLock(core::ptr::addr_of_mut!((*HubExtension).CheckIdleSpinLock) as PKSPIN_LOCK, Irql);

        if (*USBH_GetRootHubExtension(HubExtension)).SystemPowerState.SystemState
            != PowerSystemWorking
        {
            KeAcquireSpinLock(
                core::ptr::addr_of_mut!((*HubExtension).CheckIdleSpinLock) as PKSPIN_LOCK,
                core::ptr::addr_of_mut!(Irql),
            );
            (*HubExtension).HubFlags &= !USBHUB_FDO_FLAG_CHECK_IDLE_LOCK;
            KeReleaseSpinLock(core::ptr::addr_of_mut!((*HubExtension).CheckIdleSpinLock) as PKSPIN_LOCK, Irql);
            return;
        }

        HubFlags = (*HubExtension).HubFlags;
        // DPRINT("USBH_CheckHubIdle: HubFlags - %lX\n", HubFlags);

        'exit: loop {
            if (HubFlags & USBHUB_FDO_FLAG_DEVICE_STARTED) == 0
                || (HubFlags & USBHUB_FDO_FLAG_DO_ENUMERATION) == 0
            {
                break 'exit;
            }

            if HubFlags & USBHUB_FDO_FLAG_NOT_ENUMERATED != 0
                || HubFlags & USBHUB_FDO_FLAG_ENUM_POST_RECOVER != 0
                || HubFlags & USBHUB_FDO_FLAG_DEVICE_FAILED != 0
                || HubFlags & USBHUB_FDO_FLAG_DEVICE_STOPPING != 0
                || HubFlags & USBHUB_FDO_FLAG_DEVICE_REMOVED != 0
                || HubFlags & USBHUB_FDO_FLAG_STATE_CHANGING != 0
                || HubFlags & USBHUB_FDO_FLAG_WAKEUP_START != 0
                || HubFlags & USBHUB_FDO_FLAG_ESD_RECOVERING != 0
            {
                break 'exit;
            }

            if (*HubExtension).ResetRequestCount != 0 {
                (*HubExtension).HubFlags |= USBHUB_FDO_FLAG_DEFER_CHECK_IDLE;
                break 'exit;
            }

            (*HubExtension).HubFlags &= !USBHUB_FDO_FLAG_DEFER_CHECK_IDLE;

            InterlockedIncrement(core::ptr::addr_of_mut!((*HubExtension).PendingRequestCount));

            KeWaitForSingleObject(
                core::ptr::addr_of_mut!((*HubExtension).ResetDeviceSemaphore) as *mut _,
                _KWAIT_REASON_Executive,
                0i32 as i8,
                0,
                core::ptr::null_mut(),
            );

            IoAcquireCancelSpinLock(core::ptr::addr_of_mut!(Irql));

            IsAllPortsIdle = 1;

            PortData = (*HubExtension).PortData;

            Port = 0;
            while Port < (*(*HubExtension).HubDescriptor).bNumberOfPorts as ULONG {
                PdoDevice = (*PortData.add(Port as usize)).DeviceObject;

                if !PdoDevice.is_null() {
                    PortExtension = (*PdoDevice).DeviceExtension as PUSBHUB_PORT_PDO_EXTENSION;

                    if (*PortExtension).IdleNotificationIrp.is_null() {
                        // DPRINT("USBH_CheckHubIdle: PortExtension - %p\n", PortExtension);

                        IsAllPortsIdle = 0;
                        IsHubCheck = 0;

                        break;
                    }
                }

                Port += 1;
            }

            if IsHubCheck != 0
                && ((*HubExtension).HubFlags & USBHUB_FDO_FLAG_WAIT_IDLE_REQUEST) == 0
            {
                KeClearEvent(core::ptr::addr_of_mut!((*HubExtension).IdleEvent) as PVOID);
                (*HubExtension).HubFlags |= USBHUB_FDO_FLAG_WAIT_IDLE_REQUEST;
                IsHubIdle = 1;
            }

            IoReleaseCancelSpinLock(Irql);

            KeReleaseSemaphore(
                core::ptr::addr_of_mut!((*HubExtension).ResetDeviceSemaphore),
                LOW_REALTIME_PRIORITY as KPRIORITY,
                1,
                0,
            );

            if InterlockedDecrement(core::ptr::addr_of_mut!((*HubExtension).PendingRequestCount))
                == 0
            {
                KeSetEvent(core::ptr::addr_of_mut!((*HubExtension).PendingRequestEvent) as PVOID, EVENT_INCREMENT as LONG, 0);
            }

            // DPRINT("USBH_CheckHubIdle: IsAllPortsIdle - %x, IsHubIdle - %x\n", IsAllPortsIdle, IsHubIdle);

            if IsAllPortsIdle != 0 && IsHubIdle != 0 {
                USBH_FdoSubmitIdleRequestIrp(HubExtension);
            }

            break 'exit;
        }

        KeAcquireSpinLock(
            core::ptr::addr_of_mut!((*HubExtension).CheckIdleSpinLock) as PKSPIN_LOCK,
            core::ptr::addr_of_mut!(Irql),
        );
        (*HubExtension).HubFlags &= !USBHUB_FDO_FLAG_CHECK_IDLE_LOCK;
        KeReleaseSpinLock(core::ptr::addr_of_mut!((*HubExtension).CheckIdleSpinLock) as PKSPIN_LOCK, Irql);
    }
}
#[no_mangle]
pub unsafe extern "system" fn USBH_CheckIdleWorker(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    Context: PVOID,
) {
    // DPRINT("USBH_CheckIdleWorker: ... \n");
    USBH_CheckHubIdle(HubExtension);
}

#[no_mangle]
pub unsafe extern "system" fn USBH_CheckIdleDeferred(HubExtension: PUSBHUB_FDO_EXTENSION) {
    let mut HubIoWorkItem: PUSBHUB_IO_WORK_ITEM = core::ptr::null_mut();
    let Status: NTSTATUS;

    // DPRINT("USBH_CheckIdleDeferred: HubExtension - %p\n", HubExtension);

    Status = USBH_AllocateWorkItem(
        HubExtension,
        core::ptr::addr_of_mut!(HubIoWorkItem),
        cb!(USBH_CheckIdleWorker),
        0,
        core::ptr::null_mut(),
        DelayedWorkQueue,
    );

    // DPRINT("USBH_CheckIdleDeferred: HubIoWorkItem - %p\n", HubIoWorkItem);

    if NT_SUCCESS(Status) {
        USBH_QueueWorkItem(HubExtension, HubIoWorkItem);
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBH_PdoSetCapabilities(PortExtension: PUSBHUB_PORT_PDO_EXTENSION) {
    let HubExtension: PUSBHUB_FDO_EXTENSION;
    let mut State: ULONG;
    let mut SystemPowerState: SYSTEM_POWER_STATE;
    let mut pDeviceState: PDEVICE_POWER_STATE;

    HubExtension = (*PortExtension).HubExtension;

    (*PortExtension).Capabilities.Size = 64;
    (*PortExtension).Capabilities.Version = 1;

    (*PortExtension).Capabilities.set_Removable(1);
    (*PortExtension).Capabilities.Address = (*PortExtension).PortNumber as ULONG;

    if (*PortExtension).SerialNumber != core::ptr::null_mut() {
        (*PortExtension).Capabilities.set_UniqueID(1);
    } else {
        (*PortExtension).Capabilities.set_UniqueID(0);
    }

    (*PortExtension).Capabilities.set_RawDeviceOK(0);

    RtlCopyMemory(
        (*PortExtension).Capabilities.DeviceState.as_mut_ptr() as *mut core::ffi::c_void,
        (*HubExtension).DeviceState.as_ptr() as *const core::ffi::c_void,
        ((PowerSystemMaximum + 2) as usize) * core::mem::size_of::<POWER_STATE>(),
    );

    (*PortExtension).Capabilities.DeviceState[1] = PowerDeviceD0;

    if (*PortExtension).PortPdoFlags & USBHUB_PDO_FLAG_REMOTE_WAKEUP != 0 {
        (*PortExtension).Capabilities.DeviceWake = PowerDeviceD2;

        (*PortExtension).Capabilities.set_DeviceD1(1);
        (*PortExtension).Capabilities.set_DeviceD2(1);

        (*PortExtension).Capabilities.set_WakeFromD0(1);
        (*PortExtension).Capabilities.set_WakeFromD1(1);
        (*PortExtension).Capabilities.set_WakeFromD2(1);

        pDeviceState = core::ptr::addr_of_mut!((*PortExtension).Capabilities.DeviceState[2]);

        State = 2;
        while State <= 5 {
            SystemPowerState = State as SYSTEM_POWER_STATE;

            if (*PortExtension).Capabilities.SystemWake < SystemPowerState {
                *pDeviceState = PowerDeviceD3;
            } else {
                *pDeviceState = PowerDeviceD2;
            }

            pDeviceState = pDeviceState.add(1);
            State += 1;
        }
    } else {
        (*PortExtension).Capabilities.DeviceWake = PowerDeviceD0;
        (*PortExtension).Capabilities.DeviceState[2] = PowerDeviceD3;
        (*PortExtension).Capabilities.DeviceState[3] = PowerDeviceD3;
        (*PortExtension).Capabilities.DeviceState[4] = PowerDeviceD3;
        (*PortExtension).Capabilities.DeviceState[5] = PowerDeviceD3;
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBH_ProcessDeviceInformation(
    PortExtension: PUSBHUB_PORT_PDO_EXTENSION,
) -> NTSTATUS {

    let Pid: PUSB_INTERFACE_DESCRIPTOR;
    let mut ConfigDescriptor: PUSB_CONFIGURATION_DESCRIPTOR;
    let mut Status: NTSTATUS;

    // DPRINT("USBH_ProcessDeviceInformation ... \n");

    ConfigDescriptor = core::ptr::null_mut();

    RtlZeroMemory(
        core::ptr::addr_of_mut!((*PortExtension).InterfaceDescriptor) as *mut core::ffi::c_void,
        core::mem::size_of::<USB_INTERFACE_DESCRIPTOR>(),
    );

    (*PortExtension).PortPdoFlags &= !USBHUB_PDO_FLAG_HUB_DEVICE;

    Status = USBH_GetConfigurationDescriptor(
        (*PortExtension).Common.SelfDevice,
        core::ptr::addr_of_mut!(ConfigDescriptor),
    );

    if !NT_SUCCESS(Status) {
        if !ConfigDescriptor.is_null() {
            ExFreePool(ConfigDescriptor as *mut core::ffi::c_void);
        }

        return Status;
    }

    (*PortExtension).PortPdoFlags &= !USBHUB_PDO_FLAG_REMOTE_WAKEUP;

    if (*ConfigDescriptor).bmAttributes & 0x20 != 0 {
        /* device configuration supports remote wakeup */
        (*PortExtension).PortPdoFlags |= USBHUB_PDO_FLAG_REMOTE_WAKEUP;
    }

    USBHUB_DumpingDeviceDescriptor(core::ptr::addr_of_mut!((*PortExtension).DeviceDescriptor));
    USBHUB_DumpingConfiguration(ConfigDescriptor);

    // DPRINT_PNP!("USBH_ProcessDeviceInformation: Class - %x, SubClass - %x, Protocol - %x\n", ...)

    // DPRINT_PNP!("USBH_ProcessDeviceInformation: bNumConfigurations - %x, bNumInterfaces - %x\n", ...)

    /* Enumeration of USB Composite Devices (msdn): ... */

    if (((*PortExtension).DeviceDescriptor.bDeviceClass == (USB_DEVICE_CLASS_RESERVED as u8))
        || ((*PortExtension).DeviceDescriptor.bDeviceClass == (USB_DEVICE_CLASS_MISCELLANEOUS as u8)
            && (*PortExtension).DeviceDescriptor.bDeviceSubClass == 0x02
            && (*PortExtension).DeviceDescriptor.bDeviceProtocol == 0x01))
        && ((*ConfigDescriptor).bNumInterfaces > 1)
        && ((*PortExtension).DeviceDescriptor.bNumConfigurations < 2)
    {
        // DPRINT("USBH_ProcessDeviceInformation: Multi-Interface configuration\n");

        (*PortExtension).PortPdoFlags |= USBHUB_PDO_FLAG_MULTI_INTERFACE;

        if !ConfigDescriptor.is_null() {
            ExFreePool(ConfigDescriptor as *mut core::ffi::c_void);
        }

        return Status;
    }

    Pid = USBD_ParseConfigurationDescriptorEx(
        ConfigDescriptor,
        ConfigDescriptor as *mut core::ffi::c_void,
        -1,
        -1,
        -1,
        -1,
        -1,
    );
    if !Pid.is_null() {
        RtlCopyMemory(
            core::ptr::addr_of_mut!((*PortExtension).InterfaceDescriptor) as *mut core::ffi::c_void,
            Pid as *const core::ffi::c_void,
            core::mem::size_of::<USB_INTERFACE_DESCRIPTOR>(),
        );

        if (*Pid).bInterfaceClass == (USB_DEVICE_CLASS_HUB as u8) {
            (*PortExtension).PortPdoFlags |=
                USBHUB_PDO_FLAG_HUB_DEVICE | USBHUB_PDO_FLAG_REMOTE_WAKEUP;
        }
    } else {
        Status = STATUS_UNSUCCESSFUL;
    }

    if !ConfigDescriptor.is_null() {
        ExFreePool(ConfigDescriptor as *mut core::ffi::c_void);
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_CheckDeviceIDUnique(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    idVendor: USHORT,
    idProduct: USHORT,
    SerialNumber: PVOID,
    SN_DescriptorLength: USHORT,
) -> BOOLEAN {
    let mut PortDevice: PDEVICE_OBJECT;
    let mut PortExtension: PUSBHUB_PORT_PDO_EXTENSION;
    let mut Port: ULONG;
    let mut NumberBytes: SIZE_T;

    if (*(*HubExtension).HubDescriptor).bNumberOfPorts == 0 {
        return TRUE as BOOLEAN;
    }

    Port = 0;
    while (Port as USHORT) < (*(*HubExtension).HubDescriptor).bNumberOfPorts as USHORT {
        PortDevice = (*(*HubExtension).PortData.offset(Port as isize)).DeviceObject;

        if !PortDevice.is_null() {
            PortExtension = (*PortDevice).DeviceExtension as PUSBHUB_PORT_PDO_EXTENSION;

            if (*PortExtension).DeviceDescriptor.idVendor == idVendor
                && (*PortExtension).DeviceDescriptor.idProduct == idProduct
                && (*PortExtension).SN_DescriptorLength == SN_DescriptorLength
            {
                if !(*PortExtension).SerialNumber.is_null() {
                    NumberBytes = RtlCompareMemory(
                        (*PortExtension).SerialNumber as *const core::ffi::c_void,
                        SerialNumber as *const core::ffi::c_void,
                        SN_DescriptorLength as SIZE_T,
                    );

                    if NumberBytes == SN_DescriptorLength as SIZE_T {
                        return FALSE as BOOLEAN;
                    }
                }
            }
        }

        Port += 1;
    }

    return TRUE as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_ValidateSerialNumberString(SerialNumberString: PUSHORT) -> BOOLEAN {
    let mut ix: USHORT;
    let mut Symbol: USHORT;

    ix = 0;
    while *SerialNumberString.offset(ix as isize) != UNICODE_NULL {
        Symbol = *SerialNumberString.offset(ix as isize);

        if (Symbol as i32) < 0x20 || (Symbol as i32) > 0x7F || Symbol as i32 == 0x2C {
            return 0;
        }

        ix = ix.wrapping_add(1);
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_CheckDeviceLanguage(
    DeviceObject: PDEVICE_OBJECT,
    LanguageId: USHORT,
) -> NTSTATUS {
    let mut Descriptor: PUSB_STRING_DESCRIPTOR;
    let mut Status: NTSTATUS;
    let mut NumSymbols: ULONG;
    let mut ix: ULONG;
    let mut pSymbol: PWCHAR;
    let mut Length: ULONG = 0;

    // DPRINT("USBH_CheckDeviceLanguage: LanguageId - 0x%04X\n", LanguageId);

    Descriptor = ExAllocatePoolWithTag(
        NonPagedPool,
        MAXIMUM_USB_STRING_LENGTH as usize,
        USB_HUB_TAG,
    ) as PUSB_STRING_DESCRIPTOR;

    if Descriptor.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlZeroMemory(Descriptor as *mut _, MAXIMUM_USB_STRING_LENGTH as usize);

    Status = USBH_SyncGetStringDescriptor(
        DeviceObject,
        0,
        0,
        Descriptor,
        MAXIMUM_USB_STRING_LENGTH as ULONG,
        core::ptr::addr_of_mut!(Length),
        1,
    );

    loop {
        if !NT_SUCCESS(Status) || (Length as usize) < core::mem::size_of::<USB_COMMON_DESCRIPTOR>() {
            break;
        }

        NumSymbols = (Length - core::mem::offset_of!(USB_STRING_DESCRIPTOR, bString) as ULONG)
            / core::mem::size_of::<WCHAR>() as ULONG;

        pSymbol = core::ptr::addr_of_mut!((*Descriptor).bString).cast();

        ix = 1;
        while ix < NumSymbols {
            if *pSymbol == LanguageId as WCHAR {
                Status = STATUS_SUCCESS;
                break;
            }

            pSymbol = pSymbol.offset(1);
            ix += 1;
        }

        if ix < NumSymbols {
            break;
        }

        Status = STATUS_NOT_SUPPORTED;
        break;
    }

    ExFreePoolWithTag(Descriptor as *mut _, USB_HUB_TAG);
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_GetSerialNumberString(
    DeviceObject: PDEVICE_OBJECT,
    OutSerialNumber: *mut LPWSTR,
    OutDescriptorLength: PUSHORT,
    LanguageId: USHORT,
    Index: UCHAR,
) -> NTSTATUS {
    let Descriptor: PUSB_STRING_DESCRIPTOR;
    let mut Status: NTSTATUS;
    let mut SerialNumberBuffer: LPWSTR = core::ptr::null_mut();
    let StringLength: UCHAR;
    let Length: UCHAR;

    // DPRINT("USBH_GetSerialNumberString: ... \n");

    *OutSerialNumber = core::ptr::null_mut();
    *OutDescriptorLength = 0;

    Descriptor = ExAllocatePoolWithTag(
        NonPagedPool,
        MAXIMUM_USB_STRING_LENGTH as SIZE_T,
        USB_HUB_TAG,
    ) as PUSB_STRING_DESCRIPTOR;

    if Descriptor.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlZeroMemory(Descriptor as *mut core::ffi::c_void, MAXIMUM_USB_STRING_LENGTH as SIZE_T);

    Status = USBH_CheckDeviceLanguage(DeviceObject, LanguageId);

    loop {
        if !NT_SUCCESS(Status) {
            break;
        }

        Status = USBH_SyncGetStringDescriptor(
            DeviceObject,
            Index,
            LanguageId,
            Descriptor,
            MAXIMUM_USB_STRING_LENGTH as ULONG,
            core::ptr::null_mut(),
            1,
        );

        if !NT_SUCCESS(Status)
            || ((*Descriptor).bLength as usize) <= core::mem::size_of::<USB_COMMON_DESCRIPTOR>()
        {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break;
        }

        StringLength = ((*Descriptor).bLength as usize
            - core::mem::offset_of!(USB_STRING_DESCRIPTOR, bString)) as UCHAR;

        Length = (StringLength as usize + core::mem::size_of::<WCHAR>()) as UCHAR;

        SerialNumberBuffer =
            ExAllocatePoolWithTag(PagedPool, Length as SIZE_T, USB_HUB_TAG) as LPWSTR;

        if SerialNumberBuffer.is_null() {
            break;
        }

        RtlZeroMemory(SerialNumberBuffer as *mut core::ffi::c_void, Length as SIZE_T);
        RtlCopyMemory(
            SerialNumberBuffer as *mut core::ffi::c_void,
            core::ptr::addr_of!((*Descriptor).bString) as *const core::ffi::c_void,
            StringLength as SIZE_T,
        );

        *OutSerialNumber = SerialNumberBuffer;
        *OutDescriptorLength = Length as USHORT;

        break;
    }

    ExFreePoolWithTag(Descriptor as *mut core::ffi::c_void, USB_HUB_TAG);
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_CreateDevice(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    Port: USHORT,
    UsbPortStatus: USB_PORT_STATUS,
    IsWait: ULONG,
) -> NTSTATUS {

    let mut PdoNumber: ULONG = 0;
    let mut CharDeviceName: [WCHAR; 64] = [0; 64];
    let mut DeviceName: _UNICODE_STRING = core::mem::zeroed();
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let PortExtension: PUSBHUB_PORT_PDO_EXTENSION;
    let DeviceHandle: PUSB_DEVICE_HANDLE;
    let mut SerialNumberBuffer: LPWSTR;
    let IsHsDevice: BOOLEAN;
    let IsLsDevice: BOOLEAN;
    let mut IgnoringHwSerial: BOOLEAN = 0;
    let mut Status: NTSTATUS;
    let mut DestinationString: _UNICODE_STRING = core::mem::zeroed();

    ntkrnl_sys::driver_ddk::DbgPrint(
        b"[USBHUB] CreateDevice port=%u status=0x%x\n\0".as_ptr() as *const i8,
        Port as u32,
        *UsbPortStatus.AsUshort16.as_ref() as u32,
    );

    loop {
        // L"\\Device\\USBPDO-%d" as a NUL-terminated UTF-16 literal (no_std: fixed array, not Vec).
        const USBPDO_FMT: [u16; 18] = [
            0x005C, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, // \Device\
            0x0055, 0x0053, 0x0042, 0x0050, 0x0044, 0x004F, 0x002D, // USBPDO-
            0x0025, 0x0064, 0x0000, // %d\0
        ];
        RtlStringCbPrintfW(
            CharDeviceName.as_mut_ptr(),
            core::mem::size_of::<[WCHAR; 64]>(),
            USBPDO_FMT.as_ptr(),
            PdoNumber,
        );

        RtlInitUnicodeString(
            core::ptr::addr_of_mut!(DeviceName),
            CharDeviceName.as_ptr(),
        );

        Status = IoCreateDevice(
            (*(*HubExtension).Common.SelfDevice).DriverObject,
            core::mem::size_of::<USBHUB_PORT_PDO_EXTENSION>() as ULONG,
            core::ptr::addr_of_mut!(DeviceName),
            FILE_DEVICE_USB,
            0,
            0,
            core::ptr::addr_of_mut!(DeviceObject),
        );

        PdoNumber += 1;

        if Status != STATUS_OBJECT_NAME_COLLISION {
            break;
        }
    }

    if !NT_SUCCESS(Status) {
        (*(*HubExtension).PortData.add((Port - 1) as usize)).DeviceObject = DeviceObject;
        return Status;
    }

    (*DeviceObject).StackSize = (*(*HubExtension).RootHubPdo2).StackSize;

    PortExtension = (*DeviceObject).DeviceExtension as PUSBHUB_PORT_PDO_EXTENSION;

    RtlZeroMemory(
        PortExtension as PVOID,
        core::mem::size_of::<USBHUB_PORT_PDO_EXTENSION>(),
    );

    (*PortExtension).Common.ExtensionType = USBH_EXTENSION_TYPE_PORT;
    (*PortExtension).Common.SelfDevice = DeviceObject;

    (*PortExtension).HubExtension = HubExtension;
    (*PortExtension).RootHubExtension = HubExtension;

    (*PortExtension).PortNumber = Port;
    (*PortExtension).CurrentPowerState.DeviceState = PowerDeviceD0;
    (*PortExtension).IgnoringHwSerial = 0;

    KeInitializeSpinLock(core::ptr::addr_of_mut!((*PortExtension).PortTimeoutSpinLock));

    InitializeListHead(core::ptr::addr_of_mut!((*PortExtension).PortPowerList));
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*PortExtension).PortPowerListSpinLock));

    (*PortExtension).PoRequestCounter = 0;
    (*PortExtension).PendingSystemPoRequest = 0;
    (*PortExtension).PendingDevicePoRequest = 0;
    (*PortExtension).StateBehindD2 = 0;

    SerialNumberBuffer = core::ptr::null_mut();

    IsHsDevice = (*PortExtension).HubExtension as usize as u8; // placeholder
    let _ = IsHsDevice;
    let IsHsDevice: BOOLEAN = (*ps20_bits(UsbPortStatus.Usb20PortStatus.as_ref() as *const _)).HighSpeedDeviceAttached() as BOOLEAN;
    IsLsDevice = (*ps20_bits(UsbPortStatus.Usb20PortStatus.as_ref() as *const _)).LowSpeedDeviceAttached() as BOOLEAN;

    if IsLsDevice == 0 {
        if IsHsDevice != 0 {
            (*PortExtension).PortPdoFlags = USBHUB_PDO_FLAG_PORT_HIGH_SPEED;
        }
    } else {
        (*PortExtension).PortPdoFlags = USBHUB_PDO_FLAG_PORT_LOW_SPEED;
    }

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(DestinationString),
        core::ptr::addr_of!((*PortExtension).InstanceID) as PCWSTR,
    );
    DestinationString.MaximumLength = (4 * core::mem::size_of::<WCHAR>()) as USHORT;
    Status = RtlIntegerToUnicodeString(
        Port as ULONG,
        10,
        core::ptr::addr_of_mut!(DestinationString),
    );

    (*DeviceObject).Flags |= DO_POWER_PAGABLE;
    (*DeviceObject).Flags &= !DO_DEVICE_INITIALIZING;

    'error_exit: loop {
        if !NT_SUCCESS(Status) {
            break 'error_exit;
        }

        Status = USBD_CreateDeviceEx(
            HubExtension,
            core::ptr::addr_of_mut!((*PortExtension).DeviceHandle),
            UsbPortStatus,
            Port,
        );

        if !NT_SUCCESS(Status) {
            break 'error_exit;
        }

        Status = USBH_SyncResetPort(HubExtension, Port);

        if !NT_SUCCESS(Status) {
            break 'error_exit;
        }

        if IsWait != 0 {
            USBH_Wait(50);
        }

        Status = USBD_InitializeDeviceEx(
            HubExtension,
            (*PortExtension).DeviceHandle,
            core::ptr::addr_of_mut!((*PortExtension).DeviceDescriptor) as PUCHAR,
            core::mem::size_of::<USB_DEVICE_DESCRIPTOR>() as ULONG,
            core::ptr::addr_of_mut!((*PortExtension).ConfigDescriptor) as PUCHAR,
            core::mem::size_of::<USB_CONFIGURATION_DESCRIPTOR>() as ULONG,
        );

        if !NT_SUCCESS(Status) {
            (*PortExtension).DeviceHandle = core::ptr::null_mut();
            break 'error_exit;
        }

        if true {
            IgnoringHwSerial = 0;
        }

        if IgnoringHwSerial != 0 {
            (*PortExtension).IgnoringHwSerial = 1;
        }

        if (*PortExtension).DeviceDescriptor.iSerialNumber != 0
            && (*PortExtension).IgnoringHwSerial == 0
        {
            InterlockedExchangePointer(
                core::ptr::addr_of_mut!((*PortExtension).SerialNumber) as *mut PVOID,
                core::ptr::null_mut(),
            );

            USBH_GetSerialNumberString(
                (*PortExtension).Common.SelfDevice,
                core::ptr::addr_of_mut!(SerialNumberBuffer),
                core::ptr::addr_of_mut!((*PortExtension).SN_DescriptorLength),
                MAKELANGID(LANG_ENGLISH, SUBLANG_ENGLISH_US),
                (*PortExtension).DeviceDescriptor.iSerialNumber,
            );

            if !SerialNumberBuffer.is_null() {
                if USBH_ValidateSerialNumberString(SerialNumberBuffer as PUSHORT) == 0 {
                    ExFreePoolWithTag(SerialNumberBuffer as PVOID, USB_HUB_TAG);
                    SerialNumberBuffer = core::ptr::null_mut();
                }

                if !SerialNumberBuffer.is_null()
                    && USBH_CheckDeviceIDUnique(
                        HubExtension,
                        (*PortExtension).DeviceDescriptor.idVendor,
                        (*PortExtension).DeviceDescriptor.idProduct,
                        SerialNumberBuffer as PVOID,
                        (*PortExtension).SN_DescriptorLength,
                    ) == 0
                {
                    ExFreePoolWithTag(SerialNumberBuffer as PVOID, USB_HUB_TAG);
                    SerialNumberBuffer = core::ptr::null_mut();
                }
            }

            InterlockedExchangePointer(
                core::ptr::addr_of_mut!((*PortExtension).SerialNumber) as *mut PVOID,
                SerialNumberBuffer as PVOID,
            );
        }

        Status = USBH_ProcessDeviceInformation(PortExtension);

        USBH_PdoSetCapabilities(PortExtension);

        if NT_SUCCESS(Status) {
            // goto Exit
            (*(*HubExtension).PortData.add((Port - 1) as usize)).DeviceObject = DeviceObject;
            return Status;
        }

        break 'error_exit;
    }

    // ErrorExit:
    (*PortExtension).PortPdoFlags |= USBHUB_PDO_FLAG_INIT_PORT_FAILED;

    DeviceHandle = InterlockedExchangePointer(
        core::ptr::addr_of_mut!((*PortExtension).DeviceHandle) as *mut PVOID,
        core::ptr::null_mut(),
    ) as PUSB_DEVICE_HANDLE;

    if !DeviceHandle.is_null() {
        USBD_RemoveDeviceEx(HubExtension, DeviceHandle, 0);
    }

    SerialNumberBuffer = InterlockedExchangePointer(
        core::ptr::addr_of_mut!((*PortExtension).SerialNumber) as *mut PVOID,
        core::ptr::null_mut(),
    ) as LPWSTR;

    if !SerialNumberBuffer.is_null() {
        ExFreePoolWithTag(SerialNumberBuffer as PVOID, USB_HUB_TAG);
    }

    // Exit:
    (*(*HubExtension).PortData.add((Port - 1) as usize)).DeviceObject = DeviceObject;
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_ResetDevice(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    Port: USHORT,
    IsKeepDeviceData: BOOLEAN,
    IsWait: BOOLEAN,
) -> NTSTATUS {

    let mut Status: NTSTATUS;
    let PortData: PUSBHUB_PORT_DATA;
    let PortDevice: PDEVICE_OBJECT;
    let PortExtension: PUSBHUB_PORT_PDO_EXTENSION;
    let NewDeviceHandle: PVOID;
    let Handle: PVOID;
    let mut OldDeviceHandle: PVOID;
    let DeviceHandle: *mut PUSB_DEVICE_HANDLE;
    let mut PortStatus: USB_PORT_STATUS_AND_CHANGE = core::mem::zeroed();

    Status = USBH_SyncGetPortStatus(
        HubExtension,
        Port,
        core::ptr::addr_of_mut!(PortStatus),
        core::mem::size_of::<USB_PORT_STATUS_AND_CHANGE>() as ULONG,
    );

    if !NT_SUCCESS(Status)
        || (*psc_status20(core::ptr::addr_of!(PortStatus))).CurrentConnectStatus() == 0
    {
        return STATUS_UNSUCCESSFUL;
    }

    InterlockedIncrement(core::ptr::addr_of_mut!((*HubExtension).PendingRequestCount));

    KeWaitForSingleObject(
        core::ptr::addr_of_mut!((*HubExtension).ResetDeviceSemaphore) as *mut _,
        _KWAIT_REASON_Executive,
        0i32 as i8,
        FALSE as BOOLEAN,
        core::ptr::null_mut(),
    );

    PortData = core::ptr::addr_of_mut!(*(*HubExtension).PortData.offset((Port - 1) as isize));

    PortDevice = (*PortData).DeviceObject;

    if PortDevice.is_null() {
        Status = STATUS_INVALID_PARAMETER;

        KeReleaseSemaphore(
            core::ptr::addr_of_mut!((*HubExtension).ResetDeviceSemaphore) as *mut _,
            LOW_REALTIME_PRIORITY as KPRIORITY,
            1,
            FALSE as BOOLEAN,
        );

        if InterlockedDecrement(core::ptr::addr_of_mut!((*HubExtension).PendingRequestCount)) == 0 {
            KeSetEvent(core::ptr::addr_of_mut!((*HubExtension).PendingRequestEvent) as PVOID, EVENT_INCREMENT as LONG, FALSE as BOOLEAN);
        }

        return Status;
    }

    PortExtension = (*PortDevice).DeviceExtension as PUSBHUB_PORT_PDO_EXTENSION;
    DeviceHandle = core::ptr::addr_of_mut!((*PortExtension).DeviceHandle);

    OldDeviceHandle = InterlockedExchangePointer(
        core::ptr::addr_of_mut!((*PortExtension).DeviceHandle) as *mut PVOID,
        core::ptr::null_mut(),
    );

    if !OldDeviceHandle.is_null() {
        if ((*PortExtension).PortPdoFlags & USBHUB_PDO_FLAG_REMOVING_PORT_PDO) == 0 {
            Status = USBD_RemoveDeviceEx(HubExtension, OldDeviceHandle, IsKeepDeviceData as ULONG);

            (*PortExtension).PortPdoFlags |= USBHUB_PDO_FLAG_REMOVING_PORT_PDO;
        }
    } else {
        OldDeviceHandle = core::ptr::null_mut();
    }

    'exit: loop {
        'error_exit: loop {
            if !NT_SUCCESS(Status) {
                break 'error_exit;
            }

            Status = USBH_SyncResetPort(HubExtension, Port);

            if !NT_SUCCESS(Status) {
                break 'error_exit;
            }

            Status = USBH_SyncGetPortStatus(
                HubExtension,
                Port,
                core::ptr::addr_of_mut!(PortStatus),
                core::mem::size_of::<USB_PORT_STATUS_AND_CHANGE>() as ULONG,
            );

            if !NT_SUCCESS(Status) {
                break 'error_exit;
            }

            Status = USBD_CreateDeviceEx(
                HubExtension,
                DeviceHandle,
                core::ptr::read_unaligned(core::ptr::addr_of!(
                    PortStatus.__bindgen_anon_1.as_ref().PortStatus
                )),
                Port,
            );

            if !NT_SUCCESS(Status) {
                break 'error_exit;
            }

            Status = USBH_SyncResetPort(HubExtension, Port);

            if IsWait != 0 {
                USBH_Wait(50);
            }

            if !NT_SUCCESS(Status) {
                break 'error_exit;
            }

            Status = USBD_InitializeDeviceEx(
                HubExtension,
                *DeviceHandle,
                core::ptr::addr_of_mut!((*PortExtension).DeviceDescriptor.bLength),
                core::mem::size_of_val(&(*PortExtension).DeviceDescriptor) as ULONG,
                core::ptr::addr_of_mut!((*PortExtension).ConfigDescriptor.bLength),
                core::mem::size_of_val(&(*PortExtension).ConfigDescriptor) as ULONG,
            );

            if NT_SUCCESS(Status) {
                if IsKeepDeviceData != 0 {
                    Status = USBD_RestoreDeviceEx(HubExtension, OldDeviceHandle, *DeviceHandle);

                    if !NT_SUCCESS(Status) {
                        Handle = InterlockedExchangePointer(
                            DeviceHandle as *mut PVOID,
                            core::ptr::null_mut(),
                        );

                        USBD_RemoveDeviceEx(HubExtension, Handle, 0);
                        USBH_SyncDisablePort(HubExtension, Port);

                        Status = STATUS_NO_SUCH_DEVICE;
                    }
                } else {
                    (*PortExtension).PortPdoFlags &= !USBHUB_PDO_FLAG_REMOVING_PORT_PDO;
                }

                break 'exit;
            }

            *DeviceHandle = core::ptr::null_mut();

            break 'error_exit;
        }

        // ErrorExit:
        NewDeviceHandle =
            InterlockedExchangePointer(DeviceHandle as *mut PVOID, OldDeviceHandle);

        if !NewDeviceHandle.is_null() {
            Status = USBD_RemoveDeviceEx(HubExtension, NewDeviceHandle, 0);
        }

        break 'exit;
    }

    // Exit:
    KeReleaseSemaphore(
        core::ptr::addr_of_mut!((*HubExtension).ResetDeviceSemaphore) as *mut _,
        LOW_REALTIME_PRIORITY as KPRIORITY,
        1,
        FALSE as BOOLEAN,
    );

    if InterlockedDecrement(core::ptr::addr_of_mut!((*HubExtension).PendingRequestCount)) == 0 {
        KeSetEvent(core::ptr::addr_of_mut!((*HubExtension).PendingRequestEvent) as PVOID, EVENT_INCREMENT as LONG, FALSE as BOOLEAN);
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_PdoDispatch(
    PortExtension: PUSBHUB_PORT_PDO_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let IoStack: PIO_STACK_LOCATION;
    let MajorFunction: UCHAR;
    let mut ShouldCompleteIrp: BOOLEAN = 0;
    let ControlCode: ULONG;
    let Status: NTSTATUS;

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    MajorFunction = (*IoStack).MajorFunction;

    match MajorFunction {
        IRP_MJ_CREATE | IRP_MJ_CLOSE => {
            Status = STATUS_SUCCESS;
            USBH_CompleteIrp(Irp, Status);
        }

        IRP_MJ_DEVICE_CONTROL => {
            ControlCode = (*IoStack).Parameters.DeviceIoControl.IoControlCode;

            if ControlCode == IOCTL_STORAGE_GET_MEDIA_SERIAL_NUMBER {
                Status = STATUS_NOT_SUPPORTED;
                USBH_CompleteIrp(Irp, Status);
            } else if ControlCode == IOCTL_KS_PROPERTY {
                DbgBreakPoint();
                Status = STATUS_NOT_SUPPORTED;
                USBH_CompleteIrp(Irp, Status);
            } else {
                Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
                USBH_CompleteIrp(Irp, Status);
            }
        }

        IRP_MJ_INTERNAL_DEVICE_CONTROL => {
            Status = USBH_PdoInternalControl(PortExtension, Irp);
        }

        IRP_MJ_PNP => {
            Status = USBH_PdoPnP(
                PortExtension,
                Irp,
                (*IoStack).MinorFunction,
                core::ptr::addr_of_mut!(ShouldCompleteIrp),
            );

            if ShouldCompleteIrp != 0 {
                USBH_CompleteIrp(Irp, Status);
            }
        }

        IRP_MJ_POWER => {
            Status = USBH_PdoPower(PortExtension, Irp, (*IoStack).MinorFunction);
        }

        IRP_MJ_SYSTEM_CONTROL => {
            Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
            USBH_CompleteIrp(Irp, Status);
        }

        _ => {
            Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
            USBH_CompleteIrp(Irp, Status);
        }
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_FdoDispatch(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let IoStack: PIO_STACK_LOCATION;
    let MajorFunction: UCHAR;
    let Status: NTSTATUS;

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    MajorFunction = (*IoStack).MajorFunction;

    match MajorFunction {
        IRP_MJ_CREATE | IRP_MJ_CLOSE => {
            Status = STATUS_SUCCESS;
            USBH_CompleteIrp(Irp, Status);
        }

        IRP_MJ_DEVICE_CONTROL => {
            Status = USBH_DeviceControl(HubExtension, Irp);
        }

        IRP_MJ_PNP => {
            Status = USBH_FdoPnP(HubExtension, Irp, (*IoStack).MinorFunction);
        }

        IRP_MJ_POWER => {
            Status = USBH_FdoPower(HubExtension, Irp, (*IoStack).MinorFunction);
        }

        IRP_MJ_SYSTEM_CONTROL => {
            /* fall through */
            Status = USBH_PassIrp((*HubExtension).LowerDevice, Irp);
        }

        IRP_MJ_INTERNAL_DEVICE_CONTROL => {
            Status = USBH_PassIrp((*HubExtension).LowerDevice, Irp);
        }

        _ => {
            Status = USBH_PassIrp((*HubExtension).LowerDevice, Irp);
        }
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_AddDevice(
    DriverObject: PDRIVER_OBJECT,
    LowerPDO: PDEVICE_OBJECT,
) -> NTSTATUS {
    let mut DeviceObject: PDEVICE_OBJECT;
    let mut Status: NTSTATUS;
    let HubExtension: PUSBHUB_FDO_EXTENSION;
    let LowerDevice: PDEVICE_OBJECT;

    DeviceObject = core::ptr::null_mut();

    Status = IoCreateDevice(
        DriverObject,
        core::mem::size_of::<USBHUB_FDO_EXTENSION>() as u32,
        core::ptr::null_mut(),
        0x8600,
        FILE_AUTOGENERATED_DEVICE_NAME,
        0,
        core::ptr::addr_of_mut!(DeviceObject),
    );

    if !NT_SUCCESS(Status) {
        if !DeviceObject.is_null() {
            IoDeleteDevice(DeviceObject);
        }

        return Status;
    }

    HubExtension = (*DeviceObject).DeviceExtension as PUSBHUB_FDO_EXTENSION;
    RtlZeroMemory(
        HubExtension as *mut core::ffi::c_void,
        core::mem::size_of::<USBHUB_FDO_EXTENSION>(),
    );

    (*HubExtension).Common.ExtensionType = USBH_EXTENSION_TYPE_HUB;

    LowerDevice = IoAttachDeviceToDeviceStack(DeviceObject, LowerPDO);

    if LowerDevice.is_null() {
        if !DeviceObject.is_null() {
            IoDeleteDevice(DeviceObject);
        }

        return STATUS_UNSUCCESSFUL;
    }

    (*HubExtension).Common.SelfDevice = DeviceObject;

    (*HubExtension).LowerPDO = LowerPDO;
    (*HubExtension).LowerDevice = LowerDevice;

    KeInitializeSemaphore(core::ptr::addr_of_mut!((*HubExtension).IdleSemaphore), 1, 1);

    (*DeviceObject).Flags |= DO_POWER_PAGABLE;
    (*DeviceObject).Flags &= !DO_DEVICE_INITIALIZING;

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_DriverUnload(DriverObject: PDRIVER_OBJECT) {
    if !GenericUSBDeviceString.is_null() {
        ExFreePool(GenericUSBDeviceString as *mut core::ffi::c_void);
        GenericUSBDeviceString = core::ptr::null_mut();
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBH_HubDispatch(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let DeviceExtension: PCOMMON_DEVICE_EXTENSION;
    let ExtensionType: ULONG;
    let Status: NTSTATUS;

    DeviceExtension = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    ExtensionType = (*DeviceExtension).ExtensionType;

    if ExtensionType == USBH_EXTENSION_TYPE_HUB {
        // DPRINT("USBH_HubDispatch: DeviceObject - %p, Irp - %p\n", DeviceObject, Irp);

        Status = USBH_FdoDispatch(DeviceExtension as PUSBHUB_FDO_EXTENSION, Irp);
    } else if ExtensionType == USBH_EXTENSION_TYPE_PORT {
        let IoStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
        let MajorFunction: UCHAR = (*IoStack).MajorFunction;
        let mut IsDprint: BOOLEAN = 1;

        if MajorFunction == IRP_MJ_INTERNAL_DEVICE_CONTROL {
            let ControlCode: ULONG = (*IoStack).Parameters.DeviceIoControl.IoControlCode;

            if ControlCode == IOCTL_INTERNAL_USB_SUBMIT_URB {
                IsDprint = 0;
            }
        }

        if IsDprint != 0 {
            // DPRINT("USBH_HubDispatch: DeviceObject - %p, Irp - %p\n", DeviceObject, Irp);
        }

        Status = USBH_PdoDispatch(DeviceExtension as PUSBHUB_PORT_PDO_EXTENSION, Irp);
    } else {
        // DPRINT1("USBH_HubDispatch: Unknown ExtensionType - %x\n", ExtensionType);
        DbgBreakPoint();
        Status = STATUS_ASSERTION_FAILURE;
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_RegQueryGenericUSBDeviceString(
    USBDeviceString: PVOID,
) -> NTSTATUS {
    let mut QueryTable: [RTL_QUERY_REGISTRY_TABLE; 2] = core::mem::zeroed();

    RtlZeroMemory(
        QueryTable.as_mut_ptr() as *mut core::ffi::c_void,
        core::mem::size_of::<[RTL_QUERY_REGISTRY_TABLE; 2]>(),
    );

    QueryTable[0].QueryRoutine = cb!(USBH_GetConfigValue);
    QueryTable[0].Flags = RTL_QUERY_REGISTRY_REQUIRED;
    QueryTable[0].Name =
        "G\0e\0n\0e\0r\0i\0c\0U\0S\0B\0D\0e\0v\0i\0c\0e\0S\0t\0r\0i\0n\0g\0\0\0"
            .as_ptr() as PWSTR;
    QueryTable[0].EntryContext = USBDeviceString;
    QueryTable[0].DefaultType = REG_NONE;
    QueryTable[0].DefaultData = core::ptr::null_mut();
    QueryTable[0].DefaultLength = 0;

    return RtlQueryRegistryValues(
        RTL_REGISTRY_CONTROL,
        "u\0s\0b\0f\0l\0a\0g\0s\0\0\0".as_ptr() as PCWSTR,
        QueryTable.as_mut_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    // DPRINT("USBHUB: DriverEntry - %wZ\n", RegistryPath);

    (*(*DriverObject).DriverExtension).AddDevice = cb!(USBH_AddDevice);
    (*DriverObject).DriverUnload = cb!(USBH_DriverUnload);

    (*DriverObject).MajorFunction[IRP_MJ_CREATE as usize] = cb!(USBH_HubDispatch);
    (*DriverObject).MajorFunction[IRP_MJ_CLOSE as usize] = cb!(USBH_HubDispatch);

    (*DriverObject).MajorFunction[IRP_MJ_DEVICE_CONTROL as usize] = cb!(USBH_HubDispatch);
    (*DriverObject).MajorFunction[IRP_MJ_INTERNAL_DEVICE_CONTROL as usize] = cb!(USBH_HubDispatch);

    (*DriverObject).MajorFunction[IRP_MJ_PNP as usize] = cb!(USBH_HubDispatch);
    (*DriverObject).MajorFunction[IRP_MJ_POWER as usize] = cb!(USBH_HubDispatch);
    (*DriverObject).MajorFunction[IRP_MJ_SYSTEM_CONTROL as usize] = cb!(USBH_HubDispatch);

    USBH_RegQueryGenericUSBDeviceString(
        core::ptr::addr_of_mut!(GenericUSBDeviceString) as PVOID,
    );

    return STATUS_SUCCESS;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS USB Hub Driver
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     USBHub main driver functions
//  * COPYRIGHT:   Copyright 2017 Vadim Galyant <vgal@rambler.ru>
//  */
//
// #include "usbhub.h"
//
// #define NDEBUG
// #include <debug.h>
//
// #define NDEBUG_USBHUB_SCE
// #define NDEBUG_USBHUB_PNP
// #include "dbg_uhub.h"
//
// #include <ntddstor.h>
//
// PWSTR GenericUSBDeviceString = NULL;
//
// NTSTATUS
// NTAPI
// USBH_Wait(IN ULONG Milliseconds)
// {
//     LARGE_INTEGER Interval;
//
//     DPRINT("USBH_Wait: Milliseconds - %x\n", Milliseconds);
//     Interval.QuadPart = -10000LL * Milliseconds - ((ULONGLONG)KeQueryTimeIncrement() - 1);
//     return KeDelayExecutionThread(KernelMode, FALSE, &Interval);
// }
//
// NTSTATUS
// NTAPI
// USBH_GetConfigValue(IN PWSTR ValueName,
//                     IN ULONG ValueType,
//                     IN PVOID ValueData,
//                     IN ULONG ValueLength,
//                     IN PVOID Context,
//                     IN PVOID EntryContext)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     DPRINT("USBHUB_GetConfigValue: ... \n");
//
//     if (ValueType == REG_BINARY)
//     {
//         *(PUCHAR)EntryContext = *(PUCHAR)ValueData;
//     }
//     else if (ValueType == REG_DWORD)
//     {
//         *(PULONG)EntryContext = *(PULONG)ValueData;
//     }
//     else
//     {
//         Status = STATUS_INVALID_PARAMETER;
//     }
//
//     return Status;
// }
//
// VOID
// NTAPI
// USBH_CompleteIrp(IN PIRP Irp,
//                  IN NTSTATUS CompleteStatus)
// {
//     if (CompleteStatus != STATUS_SUCCESS)
//     {
//         DPRINT1("USBH_CompleteIrp: Irp - %p, CompleteStatus - %X\n",
//                 Irp,
//                 CompleteStatus);
//     }
//
//     Irp->IoStatus.Status = CompleteStatus;
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
// }
//
// NTSTATUS
// NTAPI
// USBH_PassIrp(IN PDEVICE_OBJECT DeviceObject,
//              IN PIRP Irp)
// {
//     DPRINT_PNP!("USBH_PassIrp: DeviceObject - %p, Irp - %p\n",
//                DeviceObject,
//                Irp);
//
//     IoSkipCurrentIrpStackLocation(Irp);
//     return IoCallDriver(DeviceObject, Irp);
// }
//
// NTSTATUS
// NTAPI
// USBH_SyncIrpComplete(IN PDEVICE_OBJECT DeviceObject,
//                      IN PIRP Irp,
//                      IN PVOID Context)
// {
//     PUSBHUB_URB_TIMEOUT_CONTEXT HubTimeoutContext;
//     KIRQL OldIrql;
//     BOOLEAN TimerCancelled;
//
//     DPRINT("USBH_SyncIrpComplete: ... \n");
//
//     HubTimeoutContext = Context;
//
//     KeAcquireSpinLock(&HubTimeoutContext->UrbTimeoutSpinLock, &OldIrql);
//     HubTimeoutContext->IsNormalCompleted = TRUE;
//     TimerCancelled = KeCancelTimer(&HubTimeoutContext->UrbTimeoutTimer);
//     KeReleaseSpinLock(&HubTimeoutContext->UrbTimeoutSpinLock, OldIrql);
//
//     if (TimerCancelled)
//     {
//         KeSetEvent(&HubTimeoutContext->UrbTimeoutEvent,
//                    EVENT_INCREMENT,
//                    FALSE);
//     }
//
//     return STATUS_SUCCESS;
// }
//
// BOOLEAN
// NTAPI
// IsBitSet(IN PUCHAR BitMapAddress,
//          IN USHORT Bit)
// {
//     BOOLEAN IsSet;
//
//     IsSet = (BitMapAddress[Bit / 8] & (1 << (Bit & 7))) != 0;
//     DPRINT("IsBitSet: Bit - %lX, IsSet - %x\n", Bit, IsSet);
//     return IsSet;
// }
//
// PUSBHUB_PORT_PDO_EXTENSION
// NTAPI
// PdoExt(IN PDEVICE_OBJECT DeviceObject)
// {
//     PVOID PdoExtension;
//
//     DPRINT("PdoExt: DeviceObject - %p\n", DeviceObject);
//
//     if (DeviceObject)
//     {
//         PdoExtension = DeviceObject->DeviceExtension;
//     }
//     else
//     {
//         PdoExtension = NULL;
//     }
//
//     return (PUSBHUB_PORT_PDO_EXTENSION)PdoExtension;
// }
//
// NTSTATUS
// NTAPI
// USBH_WriteFailReasonID(IN PDEVICE_OBJECT DeviceObject,
//                        IN ULONG FailReason)
// {
//     NTSTATUS Status;
//     HANDLE KeyHandle;
//     UNICODE_STRING ValueName = RTL_CONSTANT_STRING(L"FailReasonID");
//
//     DPRINT("USBH_WriteFailReason: ID - %x\n", FailReason);
//
//     Status = IoOpenDeviceRegistryKey(DeviceObject,
//                                      PLUGPLAY_REGKEY_DEVICE,
//                                      STANDARD_RIGHTS_ALL,
//                                      &KeyHandle);
//
//     if (NT_SUCCESS(Status))
//     {
//         ZwSetValueKey(KeyHandle,
//                       &ValueName,
//                       0,
//                       REG_DWORD,
//                       &FailReason,
//                       sizeof(FailReason));
//
//         ZwClose(KeyHandle);
//     }
//
//     return Status;
// }
//
// VOID
// NTAPI
// USBH_UrbTimeoutDPC(IN PKDPC Dpc,
//                    IN PVOID DeferredContext,
//                    IN PVOID SystemArgument1,
//                    IN PVOID SystemArgument2)
// {
//     PUSBHUB_URB_TIMEOUT_CONTEXT HubTimeoutContext;
//     KIRQL OldIrql;
//     BOOL IsCompleted;
//
//     DPRINT("USBH_TimeoutDPC ... \n");
//
//     HubTimeoutContext = DeferredContext;
//
//     KeAcquireSpinLock(&HubTimeoutContext->UrbTimeoutSpinLock, &OldIrql);
//     IsCompleted = HubTimeoutContext->IsNormalCompleted;
//     KeReleaseSpinLock(&HubTimeoutContext->UrbTimeoutSpinLock, OldIrql);
//
//     if (!IsCompleted)
//     {
//         IoCancelIrp(HubTimeoutContext->Irp);
//     }
//
//     KeSetEvent(&HubTimeoutContext->UrbTimeoutEvent,
//                EVENT_INCREMENT,
//                FALSE);
// }
//
// NTSTATUS
// NTAPI
// USBH_SetPdoRegistryParameter(IN PDEVICE_OBJECT DeviceObject,
//                              IN PCWSTR ValueName,
//                              IN PVOID Data,
//                              IN ULONG DataSize,
//                              IN ULONG Type,
//                              IN ULONG DevInstKeyType)
// {
//     NTSTATUS Status;
//     UNICODE_STRING ValueNameString;
//     HANDLE KeyHandle;
//
//     DPRINT("USBH_SetPdoRegistryParameter ... \n");
//
//     RtlInitUnicodeString(&ValueNameString, ValueName);
//
//     Status = IoOpenDeviceRegistryKey(DeviceObject,
//                                      DevInstKeyType,
//                                      STANDARD_RIGHTS_ALL,
//                                      &KeyHandle);
//
//     if (NT_SUCCESS(Status))
//     {
//          ZwSetValueKey(KeyHandle,
//                        &ValueNameString,
//                        0,
//                        Type,
//                        Data,
//                        DataSize);
//
//         ZwClose(KeyHandle);
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_SyncSubmitUrb(IN PDEVICE_OBJECT DeviceObject,
//                    IN PURB Urb)
// {
//     KEVENT Event;
//     IO_STATUS_BLOCK IoStatusBlock;
//     PIRP Irp;
//     PIO_STACK_LOCATION IoStack;
//     PUSBHUB_URB_TIMEOUT_CONTEXT HubTimeoutContext;
//     BOOLEAN IsWaitTimeout = FALSE;
//     LARGE_INTEGER DueTime;
//     NTSTATUS Status;
//
//     DPRINT("USBH_SyncSubmitUrb: ... \n");
//
//     Urb->UrbHeader.UsbdDeviceHandle = NULL;
//
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//     Irp = IoBuildDeviceIoControlRequest(IOCTL_INTERNAL_USB_SUBMIT_URB,
//                                         DeviceObject,
//                                         NULL,
//                                         0,
//                                         NULL,
//                                         0,
//                                         TRUE,
//                                         &Event,
//                                         &IoStatusBlock);
//
//     if (!Irp)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     IoStack = IoGetNextIrpStackLocation(Irp);
//     IoStack->Parameters.Others.Argument1 = Urb;
//
//     HubTimeoutContext = ExAllocatePoolWithTag(NonPagedPool,
//                                               sizeof(USBHUB_URB_TIMEOUT_CONTEXT),
//                                               USB_HUB_TAG);
//
//     if (HubTimeoutContext)
//     {
//         RtlZeroMemory(HubTimeoutContext, sizeof(USBHUB_URB_TIMEOUT_CONTEXT));
//
//         HubTimeoutContext->Irp = Irp;
//         HubTimeoutContext->IsNormalCompleted = FALSE;
//
//         KeInitializeEvent(&HubTimeoutContext->UrbTimeoutEvent,
//                           NotificationEvent,
//                           FALSE);
//
//         KeInitializeSpinLock(&HubTimeoutContext->UrbTimeoutSpinLock);
//         KeInitializeTimer(&HubTimeoutContext->UrbTimeoutTimer);
//
//         KeInitializeDpc(&HubTimeoutContext->UrbTimeoutDPC,
//                         USBH_UrbTimeoutDPC,
//                         HubTimeoutContext);
//
//         DueTime.QuadPart = -5000 * 10000; // Timeout 5 sec.
//
//         KeSetTimer(&HubTimeoutContext->UrbTimeoutTimer,
//                    DueTime,
//                    &HubTimeoutContext->UrbTimeoutDPC);
//
//         IoSetCompletionRoutine(Irp,
//                                USBH_SyncIrpComplete,
//                                HubTimeoutContext,
//                                TRUE,
//                                TRUE,
//                                TRUE);
//
//         IsWaitTimeout = TRUE;
//     }
//
//     Status = IoCallDriver(DeviceObject, Irp);
//
//     if (Status == STATUS_PENDING)
//     {
//         KeWaitForSingleObject(&Event,
//                               Suspended,
//                               KernelMode,
//                               FALSE,
//                               NULL);
//     }
//     else
//     {
//         IoStatusBlock.Status = Status;
//     }
//
//     if (IsWaitTimeout)
//     {
//         KeWaitForSingleObject(&HubTimeoutContext->UrbTimeoutEvent,
//                               Suspended,
//                               KernelMode,
//                               FALSE,
//                               NULL);
//
//         ExFreePoolWithTag(HubTimeoutContext, USB_HUB_TAG);
//     }
//
//     return IoStatusBlock.Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_FdoSyncSubmitUrb(IN PDEVICE_OBJECT FdoDevice,
//                       IN PURB Urb)
// {
//     PUSBHUB_FDO_EXTENSION HubExtension;
//
//     DPRINT("USBH_FdoSyncSubmitUrb: FdoDevice - %p, Urb - %p\n",
//            FdoDevice,
//            Urb);
//
//     HubExtension = FdoDevice->DeviceExtension;
//     return USBH_SyncSubmitUrb(HubExtension->LowerDevice, Urb);
// }
//
// NTSTATUS
// NTAPI
// USBH_Transact(IN PUSBHUB_FDO_EXTENSION HubExtension,
//               IN PVOID TransferBuffer,
//               IN ULONG BufferLen,
//               IN BOOLEAN IsDeviceToHost,
//               IN USHORT Function,
//               IN BM_REQUEST_TYPE RequestType,
//               IN UCHAR Request,
//               IN USHORT RequestValue,
//               IN USHORT RequestIndex)
// {
//     struct _URB_CONTROL_VENDOR_OR_CLASS_REQUEST * Urb;
//     ULONG TransferFlags;
//     PVOID Buffer = NULL;
//     ULONG Length;
//     NTSTATUS Status;
//
//     DPRINT("USBH_Transact: ... \n");
//
//     if (BufferLen)
//     {
//         Length = ALIGN_DOWN_BY(BufferLen + sizeof(ULONG), sizeof(ULONG));
//
//         Buffer = ExAllocatePoolWithTag(NonPagedPool, Length, USB_HUB_TAG);
//
//         if (!Buffer)
//         {
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         RtlZeroMemory(Buffer, Length);
//     }
//
//     Urb = ExAllocatePoolWithTag(NonPagedPool,
//                                 sizeof(struct _URB_CONTROL_VENDOR_OR_CLASS_REQUEST),
//                                 USB_HUB_TAG);
//
//     if (!Urb)
//     {
//         if (Buffer)
//         {
//             ExFreePoolWithTag(Buffer, USB_HUB_TAG);
//         }
//
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlZeroMemory(Urb, sizeof(struct _URB_CONTROL_VENDOR_OR_CLASS_REQUEST));
//
//     if (IsDeviceToHost)
//     {
//         if (BufferLen)
//         {
//             RtlZeroMemory(TransferBuffer, BufferLen);
//         }
//
//         TransferFlags = USBD_TRANSFER_DIRECTION_IN | USBD_SHORT_TRANSFER_OK;
//     }
//     else
//     {
//         if (BufferLen)
//         {
//             RtlCopyMemory(Buffer, TransferBuffer, BufferLen);
//         }
//
//         TransferFlags = USBD_TRANSFER_DIRECTION_OUT;
//     }
//
//     Urb->Hdr.Length = sizeof(struct _URB_CONTROL_VENDOR_OR_CLASS_REQUEST);
//     Urb->Hdr.Function = Function;
//     Urb->Hdr.UsbdDeviceHandle = NULL;
//
//     Urb->TransferFlags = TransferFlags;
//     Urb->TransferBuffer = BufferLen != 0 ? Buffer : NULL;
//     Urb->TransferBufferLength = BufferLen;
//     Urb->TransferBufferMDL = NULL;
//     Urb->UrbLink = NULL;
//
//     Urb->RequestTypeReservedBits = RequestType.B;
//     Urb->Request = Request;
//     Urb->Value = RequestValue;
//     Urb->Index = RequestIndex;
//
//     Status = USBH_FdoSyncSubmitUrb(HubExtension->Common.SelfDevice, (PURB)Urb);
//
//     if (IsDeviceToHost && BufferLen)
//     {
//         RtlCopyMemory(TransferBuffer, Buffer, BufferLen);
//     }
//
//     if (Buffer)
//     {
//         ExFreePoolWithTag(Buffer, USB_HUB_TAG);
//     }
//
//     ExFreePoolWithTag(Urb, USB_HUB_TAG);
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_SyncResetPort(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                    IN USHORT Port)
// {
//     USB_PORT_STATUS_AND_CHANGE PortStatus;
//     KEVENT Event;
//     LARGE_INTEGER Timeout;
//     ULONG ResetRetry = 0;
//     NTSTATUS Status;
//
//     DPRINT("USBH_SyncResetPort: Port - %x\n", Port);
//
//     InterlockedIncrement(&HubExtension->PendingRequestCount);
//
//     KeWaitForSingleObject(&HubExtension->HubPortSemaphore,
//                           Executive,
//                           KernelMode,
//                           FALSE,
//                           NULL);
//
//     Status = USBH_SyncGetPortStatus(HubExtension,
//                                     Port,
//                                     &PortStatus,
//                                     sizeof(USB_PORT_STATUS_AND_CHANGE));
//
//     if (NT_SUCCESS(Status) &&
//         (PortStatus.PortStatus.Usb20PortStatus.CurrentConnectStatus == 0))
//     {
//         Status = STATUS_UNSUCCESSFUL;
//         goto Exit;
//     }
//
//     HubExtension->HubFlags |= USBHUB_FDO_FLAG_RESET_PORT_LOCK;
//
//     while (TRUE)
//     {
//         BM_REQUEST_TYPE RequestType;
//
//         KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//         InterlockedExchangePointer((PVOID)&HubExtension->pResetPortEvent,
//                                    &Event);
//
//         RequestType.B = 0;
//         RequestType.Recipient = BMREQUEST_TO_DEVICE;
//         RequestType.Type = BMREQUEST_CLASS;
//         RequestType.Dir = BMREQUEST_HOST_TO_DEVICE;
//
//         Status = USBH_Transact(HubExtension,
//                                NULL,
//                                0,
//                                BMREQUEST_HOST_TO_DEVICE,
//                                URB_FUNCTION_CLASS_OTHER,
//                                RequestType,
//                                USB_REQUEST_SET_FEATURE,
//                                USBHUB_FEATURE_PORT_RESET,
//                                Port);
//
//         Timeout.QuadPart = -5000 * 10000;
//
//         if (!NT_SUCCESS(Status))
//         {
//             InterlockedExchangePointer((PVOID)&HubExtension->pResetPortEvent,
//                                        NULL);
//
//             USBH_Wait(10);
//             HubExtension->HubFlags &= ~USBHUB_FDO_FLAG_RESET_PORT_LOCK;
//
//             goto Exit;
//         }
//
//         Status = KeWaitForSingleObject(&Event,
//                                        Suspended,
//                                        KernelMode,
//                                        FALSE,
//                                        &Timeout);
//
//         if (Status != STATUS_TIMEOUT)
//         {
//             break;
//         }
//
//         Status = USBH_SyncGetPortStatus(HubExtension,
//                                         Port,
//                                         &PortStatus,
//                                         sizeof(USB_PORT_STATUS_AND_CHANGE));
//
//         if (!NT_SUCCESS(Status) ||
//             (PortStatus.PortStatus.Usb20PortStatus.CurrentConnectStatus == 0) ||
//             ResetRetry >= USBHUB_RESET_PORT_MAX_RETRY)
//         {
//             InterlockedExchangePointer((PVOID)&HubExtension->pResetPortEvent,
//                                        NULL);
//
//             USBH_Wait(10);
//             HubExtension->HubFlags &= ~USBHUB_FDO_FLAG_RESET_PORT_LOCK;
//
//             Status = STATUS_DEVICE_DATA_ERROR;
//             goto Exit;
//         }
//
//         ResetRetry++;
//     }
//
//     Status = USBH_SyncGetPortStatus(HubExtension,
//                                     Port,
//                                     &PortStatus,
//                                     sizeof(USB_PORT_STATUS_AND_CHANGE));
//
//     if ((PortStatus.PortStatus.Usb20PortStatus.CurrentConnectStatus == 0) &&
//         NT_SUCCESS(Status) &&
//         HubExtension->HubFlags & USBHUB_FDO_FLAG_USB20_HUB)
//     {
//         Status = STATUS_DEVICE_DATA_ERROR;
//     }
//
//     USBH_Wait(10);
//     HubExtension->HubFlags &= ~USBHUB_FDO_FLAG_RESET_PORT_LOCK;
//
// Exit:
//
//     KeReleaseSemaphore(&HubExtension->HubPortSemaphore,
//                        LOW_REALTIME_PRIORITY,
//                        1,
//                        FALSE);
//
//     if (!InterlockedDecrement(&HubExtension->PendingRequestCount))
//     {
//         KeSetEvent(&HubExtension->PendingRequestEvent,
//                    EVENT_INCREMENT,
//                    FALSE);
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_GetDeviceType(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                    IN PUSB_DEVICE_HANDLE DeviceHandle,
//                    OUT USB_DEVICE_TYPE * OutDeviceType)
// {
//     PUSB_BUSIFFN_GET_DEVICE_INFORMATION QueryDeviceInformation;
//     PUSB_DEVICE_INFORMATION_0 DeviceInfo;
//     SIZE_T DeviceInformationBufferLength;
//     USB_DEVICE_TYPE DeviceType = Usb11Device;
//     ULONG dummy;
//     NTSTATUS Status;
//
//     DPRINT("USBH_GetDeviceType: ... \n");
//
//     QueryDeviceInformation = HubExtension->BusInterface.QueryDeviceInformation;
//
//     if (!QueryDeviceInformation)
//     {
//         DPRINT1("USBH_GetDeviceType: no QueryDeviceInformation()\n");
//         return STATUS_NOT_IMPLEMENTED;
//     }
//
//     DeviceInformationBufferLength = sizeof(USB_DEVICE_INFORMATION_0);
//
//     while (TRUE)
//     {
//         DeviceInfo = ExAllocatePoolWithTag(PagedPool,
//                                            DeviceInformationBufferLength,
//                                            USB_HUB_TAG);
//
//         if (!DeviceInfo)
//         {
//             DPRINT1("USBH_GetDeviceType: ExAllocatePoolWithTag() failed\n");
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             break;
//         }
//
//         RtlZeroMemory(DeviceInfo, DeviceInformationBufferLength);
//
//         DeviceInfo->InformationLevel = 0;
//
//         Status = QueryDeviceInformation(HubExtension->BusInterface.BusContext,
//                                         DeviceHandle,
//                                         DeviceInfo as PVOID,
//                                         DeviceInformationBufferLength as u32,
//                                         &dummy);
//
//         if (Status != STATUS_BUFFER_TOO_SMALL)
//         {
//             if (NT_SUCCESS(Status))
//             {
//                 DeviceType = DeviceInfo->DeviceType;
//             }
//
//             ExFreePoolWithTag(DeviceInfo, USB_HUB_TAG);
//             break;
//         }
//
//         DeviceInformationBufferLength = DeviceInfo->ActualLength;
//         ExFreePoolWithTag(DeviceInfo, USB_HUB_TAG);
//     }
//
//     if (OutDeviceType)
//     {
//         *OutDeviceType = DeviceType;
//         DPRINT("USBH_GetDeviceType: DeviceType - %x\n", DeviceType);
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBHUB_GetExtendedHubInfo(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                           IN PUSB_EXTHUB_INFORMATION_0 HubInfoBuffer)
// {
//     PUSB_BUSIFFN_GET_EXTENDED_HUB_INFO GetExtendedHubInformation;
//     ULONG dummy = 0;
//
//     DPRINT("USBHUB_GetExtendedHubInfo: ... \n");
//
//     GetExtendedHubInformation = HubExtension->BusInterface.GetExtendedHubInformation;
//
//     return GetExtendedHubInformation(HubExtension->BusInterface.BusContext,
//                                      HubExtension->LowerPDO,
//                                      HubInfoBuffer,
//                                      sizeof(USB_EXTHUB_INFORMATION_0),
//                                      &dummy);
// }
//
// PUSBHUB_FDO_EXTENSION
// NTAPI
// USBH_GetRootHubExtension(IN PUSBHUB_FDO_EXTENSION HubExtension)
// {
//     PDEVICE_OBJECT Device;
//     PUSBHUB_FDO_EXTENSION RootHubExtension;
//
//     DPRINT("USBH_GetRootHubExtension: HubExtension - %p\n", HubExtension);
//
//     RootHubExtension = HubExtension;
//
//     if (HubExtension->LowerPDO != HubExtension->RootHubPdo)
//     {
//         Device = HubExtension->RootHubPdo;
//
//         do
//         {
//             Device = Device->AttachedDevice;
//         }
//         while (Device->DriverObject != HubExtension->Common.SelfDevice->DriverObject);
//
//         RootHubExtension = Device->DeviceExtension;
//     }
//
//     DPRINT("USBH_GetRootHubExtension: RootHubExtension - %p\n", RootHubExtension);
//
//     return RootHubExtension;
// }
//
// NTSTATUS
// NTAPI
// USBH_SyncGetRootHubPdo(IN PDEVICE_OBJECT DeviceObject,
//                        IN OUT PDEVICE_OBJECT * OutPdo1,
//                        IN OUT PDEVICE_OBJECT * OutPdo2)
// {
//     KEVENT Event;
//     IO_STATUS_BLOCK IoStatusBlock;
//     PIRP Irp;
//     PIO_STACK_LOCATION IoStack;
//     NTSTATUS Status;
//
//     DPRINT("USBH_SyncGetRootHubPdo: ... \n");
//
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//     Irp = IoBuildDeviceIoControlRequest(IOCTL_INTERNAL_USB_GET_ROOTHUB_PDO,
//                                         DeviceObject,
//                                         NULL,
//                                         0,
//                                         NULL,
//                                         0,
//                                         TRUE,
//                                         &Event,
//                                         &IoStatusBlock);
//
//     if (!Irp)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     IoStack = IoGetNextIrpStackLocation(Irp);
//     IoStack->Parameters.Others.Argument1 = OutPdo1;
//     IoStack->Parameters.Others.Argument2 = OutPdo2;
//
//     Status = IoCallDriver(DeviceObject, Irp);
//
//     if (Status == STATUS_PENDING)
//     {
//         KeWaitForSingleObject(&Event,
//                               Suspended,
//                               KernelMode,
//                               FALSE,
//                               NULL);
//     }
//     else
//     {
//         IoStatusBlock.Status = Status;
//     }
//
//     return IoStatusBlock.Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_SyncGetHubCount(IN PDEVICE_OBJECT DeviceObject,
//                      IN OUT PULONG OutHubCount)
// {
//     KEVENT Event;
//     IO_STATUS_BLOCK IoStatusBlock;
//     PIRP Irp;
//     PIO_STACK_LOCATION IoStack;
//     NTSTATUS Status;
//
//     DPRINT("USBH_SyncGetHubCount: *OutHubCount - %x\n", *OutHubCount);
//
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//     Irp = IoBuildDeviceIoControlRequest(IOCTL_INTERNAL_USB_GET_HUB_COUNT,
//                                         DeviceObject,
//                                         NULL,
//                                         0,
//                                         NULL,
//                                         0,
//                                         TRUE,
//                                         &Event,
//                                         &IoStatusBlock);
//
//     if (!Irp)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     IoStack = IoGetNextIrpStackLocation(Irp);
//     IoStack->Parameters.Others.Argument1 = OutHubCount;
//
//     Status = IoCallDriver(DeviceObject, Irp);
//
//     if (Status == STATUS_PENDING)
//     {
//         KeWaitForSingleObject(&Event,
//                               Suspended,
//                               KernelMode,
//                               FALSE,
//                               NULL);
//     }
//     else
//     {
//         IoStatusBlock.Status = Status;
//     }
//
//     return IoStatusBlock.Status;
// }
//
// PUSB_DEVICE_HANDLE
// NTAPI
// USBH_SyncGetDeviceHandle(IN PDEVICE_OBJECT DeviceObject)
// {
//     PIRP Irp;
//     KEVENT Event;
//     IO_STATUS_BLOCK IoStatusBlock;
//     PUSB_DEVICE_HANDLE DeviceHandle = NULL;
//     PIO_STACK_LOCATION IoStack;
//
//     DPRINT("USBH_SyncGetDeviceHandle: ... \n");
//
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//     Irp = IoBuildDeviceIoControlRequest(IOCTL_INTERNAL_USB_GET_DEVICE_HANDLE,
//                                         DeviceObject,
//                                         NULL,
//                                         0,
//                                         NULL,
//                                         0,
//                                         TRUE,
//                                         &Event,
//                                         &IoStatusBlock);
//
//     if (!Irp)
//     {
//         DPRINT1("USBH_SyncGetDeviceHandle: Irp - NULL!\n");
//         return NULL;
//     }
//
//     IoStack = IoGetNextIrpStackLocation(Irp);
//     IoStack->Parameters.Others.Argument1 = &DeviceHandle;
//
//     if (IoCallDriver(DeviceObject, Irp) == STATUS_PENDING)
//     {
//         KeWaitForSingleObject(&Event,
//                               Suspended,
//                               KernelMode,
//                               FALSE,
//                               NULL);
//     }
//
//     return DeviceHandle;
// }
//
// NTSTATUS
// NTAPI
// USBH_GetDeviceDescriptor(IN PDEVICE_OBJECT DeviceObject,
//                          IN PUSB_DEVICE_DESCRIPTOR HubDeviceDescriptor)
// {
//     struct _URB_CONTROL_DESCRIPTOR_REQUEST * Urb;
//     NTSTATUS Status;
//
//     DPRINT("USBH_GetDeviceDescriptor: ... \n");
//
//     Urb = ExAllocatePoolWithTag(NonPagedPool,
//                                 sizeof(struct _URB_CONTROL_DESCRIPTOR_REQUEST),
//                                 USB_HUB_TAG);
//
//     if (!Urb)
//     {
//         DPRINT1("USBH_SyncGetDeviceHandle: Urb - NULL!\n");
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlZeroMemory(Urb, sizeof(struct _URB_CONTROL_DESCRIPTOR_REQUEST));
//
//     Urb->Hdr.Function = URB_FUNCTION_GET_DESCRIPTOR_FROM_DEVICE;
//     Urb->Hdr.Length = sizeof(struct _URB_CONTROL_DESCRIPTOR_REQUEST);
//
//     Urb->TransferBufferLength = sizeof(USB_DEVICE_DESCRIPTOR);
//     Urb->TransferBuffer = HubDeviceDescriptor;
//     Urb->DescriptorType = USB_DEVICE_DESCRIPTOR_TYPE;
//
//     Status = USBH_FdoSyncSubmitUrb(DeviceObject, (PURB)Urb);
//
//     ExFreePoolWithTag(Urb, USB_HUB_TAG);
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_SyncGetDeviceConfigurationDescriptor(IN PDEVICE_OBJECT DeviceObject,
//                                           IN PUSB_CONFIGURATION_DESCRIPTOR ConfigDescriptor,
//                                           IN ULONG NumberOfBytes,
//                                           IN PULONG OutLength)
// {
//     PCOMMON_DEVICE_EXTENSION DeviceExtension;
//     struct _URB_CONTROL_DESCRIPTOR_REQUEST * Urb;
//     NTSTATUS Status;
//
//     DPRINT("USBH_SyncGetDeviceConfigurationDescriptor: ... \n");
//
//     DeviceExtension = DeviceObject->DeviceExtension;
//
//     if (OutLength)
//     {
//         *OutLength = 0;
//     }
//
//     Urb = ExAllocatePoolWithTag(NonPagedPool,
//                                 sizeof(struct _URB_CONTROL_DESCRIPTOR_REQUEST),
//                                 USB_HUB_TAG);
//
//     if (!Urb)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlZeroMemory(Urb, sizeof(struct _URB_CONTROL_DESCRIPTOR_REQUEST));
//
//     Urb->Hdr.Function = URB_FUNCTION_GET_DESCRIPTOR_FROM_DEVICE;
//     Urb->Hdr.Length = sizeof(struct _URB_CONTROL_DESCRIPTOR_REQUEST);
//
//     Urb->TransferBufferLength = NumberOfBytes;
//     Urb->TransferBuffer = ConfigDescriptor;
//     Urb->DescriptorType = USB_CONFIGURATION_DESCRIPTOR_TYPE;
//
//     if (DeviceExtension->ExtensionType == USBH_EXTENSION_TYPE_HUB ||
//         DeviceExtension->ExtensionType == USBH_EXTENSION_TYPE_PARENT)
//     {
//         Status = USBH_FdoSyncSubmitUrb(DeviceObject, (PURB)Urb);
//     }
//     else
//     {
//         Status = USBH_SyncSubmitUrb(DeviceObject, (PURB)Urb);
//     }
//
//     if (OutLength)
//     {
//         *OutLength = Urb->TransferBufferLength;
//     }
//
//     if (Urb)
//     {
//         ExFreePoolWithTag(Urb, USB_HUB_TAG);
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_GetConfigurationDescriptor(IN PDEVICE_OBJECT DeviceObject,
//                                 IN PUSB_CONFIGURATION_DESCRIPTOR * OutDescriptor)
// {
//     PUSB_CONFIGURATION_DESCRIPTOR ConfigDescriptor;
//     ULONG ReturnedLen;
//     SIZE_T DescriptorLen;
//     NTSTATUS Status;
//
//     DPRINT("USBH_GetConfigurationDescriptor: ... \n");
//
//     DescriptorLen = MAXUCHAR;
//
//     while (TRUE)
//     {
//         ConfigDescriptor = ExAllocatePoolWithTag(NonPagedPool,
//                                                  DescriptorLen,
//                                                  USB_HUB_TAG);
//
//         if (!ConfigDescriptor)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             break;
//         }
//
//         Status = USBH_SyncGetDeviceConfigurationDescriptor(DeviceObject,
//                                                            ConfigDescriptor,
//                                                            DescriptorLen,
//                                                            &ReturnedLen);
//
//         if (ReturnedLen < sizeof(USB_CONFIGURATION_DESCRIPTOR))
//         {
//             Status = STATUS_DEVICE_DATA_ERROR;
//         }
//
//         if (!NT_SUCCESS(Status))
//         {
//             break;
//         }
//
//         *OutDescriptor = ConfigDescriptor;
//
//         if (ConfigDescriptor->wTotalLength <= DescriptorLen)
//         {
//             break;
//         }
//
//         DescriptorLen = ConfigDescriptor->wTotalLength;
//
//         ExFreePool(ConfigDescriptor);
//         *OutDescriptor = NULL;
//     }
//
//     if (NT_SUCCESS(Status))
//     {
//         if (ReturnedLen < ConfigDescriptor->wTotalLength)
//         {
//             Status = STATUS_DEVICE_DATA_ERROR;
//         }
//     }
//     else
//     {
//         if (ConfigDescriptor)
//         {
//             ExFreePool(ConfigDescriptor);
//         }
//
//         *OutDescriptor = NULL;
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_SyncGetHubDescriptor(IN PUSBHUB_FDO_EXTENSION HubExtension)
// {
//     PUSB_EXTHUB_INFORMATION_0 ExtendedHubInfo;
//     ULONG NumberPorts;
//     PUSBHUB_PORT_DATA PortData;
//     USHORT RequestValue;
//     ULONG NumberOfBytes;
//     NTSTATUS Status;
//     PUSB_HUB_DESCRIPTOR HubDescriptor = NULL;
//     ULONG ix;
//     ULONG Retry;
//
//     DPRINT("USBH_SyncGetHubDescriptor: ... \n");
//
//     ExtendedHubInfo = ExAllocatePoolWithTag(NonPagedPool,
//                                             sizeof(USB_EXTHUB_INFORMATION_0),
//                                             USB_HUB_TAG);
//
//     if (!ExtendedHubInfo)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto ErrorExit;
//     }
//
//     RtlZeroMemory(ExtendedHubInfo, sizeof(USB_EXTHUB_INFORMATION_0));
//
//     Status = USBHUB_GetExtendedHubInfo(HubExtension, ExtendedHubInfo);
//
//     if (!NT_SUCCESS(Status))
//     {
//         ExFreePoolWithTag(ExtendedHubInfo, USB_HUB_TAG);
//         ExtendedHubInfo = NULL;
//     }
//
//     NumberOfBytes = sizeof(USB_HUB_DESCRIPTOR);
//
//     HubDescriptor = ExAllocatePoolWithTag(NonPagedPool,
//                                           NumberOfBytes,
//                                           USB_HUB_TAG);
//
//     if (!HubDescriptor)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto ErrorExit;
//     }
//
//     RtlZeroMemory(HubDescriptor, NumberOfBytes);
//
//     RequestValue = 0;
//     Retry = 0;
//
//     while (TRUE)
//     {
//         while (Retry <= 5)
//         {
//             BM_REQUEST_TYPE RequestType;
//
//             RequestType.B = 0;
//             RequestType.Recipient = BMREQUEST_TO_DEVICE;
//             RequestType.Type = BMREQUEST_STANDARD;
//             RequestType.Dir = BMREQUEST_DEVICE_TO_HOST;
//
//             Status = USBH_Transact(HubExtension,
//                                    HubDescriptor,
//                                    NumberOfBytes,
//                                    BMREQUEST_DEVICE_TO_HOST,
//                                    URB_FUNCTION_CLASS_DEVICE,
//                                    RequestType,
//                                    USB_REQUEST_GET_DESCRIPTOR,
//                                    RequestValue,
//                                    0);
//
//             if (NT_SUCCESS(Status))
//             {
//                 break;
//             }
//
//             RequestValue = 0x2900; // Hub DescriptorType - 0x29
//
//             Retry++;
//         }
//
//         if (HubDescriptor->bDescriptorLength <= NumberOfBytes)
//         {
//             break;
//         }
//
//         NumberOfBytes = HubDescriptor->bDescriptorLength;
//         ExFreePoolWithTag(HubDescriptor, USB_HUB_TAG);
//
//         if (Retry >= 5)
//         {
//             Status = STATUS_DEVICE_DATA_ERROR;
//             HubDescriptor = NULL;
//             goto ErrorExit;
//         }
//
//         HubDescriptor = ExAllocatePoolWithTag(NonPagedPool,
//                                               NumberOfBytes,
//                                               USB_HUB_TAG);
//
//         if (!HubDescriptor)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto ErrorExit;
//         }
//
//         RtlZeroMemory(HubDescriptor, NumberOfBytes);
//     }
//
//     NumberPorts = HubDescriptor->bNumberOfPorts;
//
//     if (HubExtension->PortData)
//     {
//         PortData = HubExtension->PortData;
//
//         for (ix = 0; ix < NumberPorts; ix++)
//         {
//             PortData[ix].PortStatus.AsUlong32 = 0;
//
//             if (ExtendedHubInfo)
//             {
//                 PortData[ix].PortAttributes = ExtendedHubInfo->Port[ix].PortAttributes;
//             }
//             else
//             {
//                 PortData[ix].PortAttributes = 0;
//             }
//
//             PortData[ix].ConnectionStatus = NoDeviceConnected;
//
//             if (PortData[ix].DeviceObject != NULL)
//             {
//                 PortData[ix].ConnectionStatus = DeviceConnected;
//             }
//         }
//     }
//     else
//     {
//         PortData = NULL;
//
//         if (HubDescriptor->bNumberOfPorts)
//         {
//             PortData = ExAllocatePoolWithTag(NonPagedPool,
//                                              NumberPorts * sizeof(USBHUB_PORT_DATA),
//                                              USB_HUB_TAG);
//         }
//
//         if (!PortData)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto ErrorExit;
//         }
//
//         RtlZeroMemory(PortData, NumberPorts * sizeof(USBHUB_PORT_DATA));
//
//         for (ix = 0; ix < NumberPorts; ix++)
//         {
//             PortData[ix].ConnectionStatus = NoDeviceConnected;
//
//             if (ExtendedHubInfo)
//             {
//                 PortData[ix].PortAttributes = ExtendedHubInfo->Port[ix].PortAttributes;
//             }
//         }
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         goto ErrorExit;
//     }
//
//     HubExtension->HubDescriptor = HubDescriptor;
//
//     HubExtension->PortData = PortData;
//
//     if (ExtendedHubInfo)
//     {
//         ExFreePoolWithTag(ExtendedHubInfo, USB_HUB_TAG);
//     }
//
//     return Status;
//
// ErrorExit:
//
//     if (HubDescriptor)
//     {
//         ExFreePoolWithTag(HubDescriptor, USB_HUB_TAG);
//     }
//
//     if (ExtendedHubInfo)
//     {
//         ExFreePoolWithTag(ExtendedHubInfo, USB_HUB_TAG);
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_SyncGetStringDescriptor(IN PDEVICE_OBJECT DeviceObject,
//                              IN UCHAR Index,
//                              IN USHORT LanguageId,
//                              IN PUSB_STRING_DESCRIPTOR Descriptor,
//                              IN ULONG NumberOfBytes,
//                              IN PULONG OutLength,
//                              IN BOOLEAN IsValidateLength)
// {
//     struct _URB_CONTROL_DESCRIPTOR_REQUEST * Urb;
//     ULONG TransferedLength;
//     NTSTATUS Status;
//
//     DPRINT("USBH_SyncGetStringDescriptor: Index - %x, LanguageId - %x\n",
//            Index,
//            LanguageId);
//
//     Urb = ExAllocatePoolWithTag(NonPagedPool,
//                                 sizeof(struct _URB_CONTROL_DESCRIPTOR_REQUEST),
//                                 USB_HUB_TAG);
//
//     if (!Urb)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlZeroMemory(Urb, sizeof(struct _URB_CONTROL_DESCRIPTOR_REQUEST));
//
//     Urb->Hdr.Function = URB_FUNCTION_GET_DESCRIPTOR_FROM_DEVICE;
//     Urb->Hdr.Length = sizeof(struct _URB_CONTROL_DESCRIPTOR_REQUEST);
//
//     Urb->TransferBuffer = Descriptor;
//     Urb->TransferBufferLength = NumberOfBytes;
//
//     Urb->Index = Index;
//     Urb->DescriptorType = USB_STRING_DESCRIPTOR_TYPE;
//     Urb->LanguageId = LanguageId;
//
//     Status = USBH_SyncSubmitUrb(DeviceObject, (PURB)Urb);
//
//     if (!NT_SUCCESS(Status))
//     {
//         ExFreePoolWithTag(Urb, USB_HUB_TAG);
//         return Status;
//     }
//
//     TransferedLength = Urb->TransferBufferLength;
//
//     if (TransferedLength > NumberOfBytes)
//     {
//         Status = STATUS_DEVICE_DATA_ERROR;
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         ExFreePoolWithTag(Urb, USB_HUB_TAG);
//         return Status;
//     }
//
//     if (OutLength)
//     {
//         *OutLength = TransferedLength;
//     }
//
//     if (IsValidateLength && TransferedLength != Descriptor->bLength)
//     {
//         Status = STATUS_DEVICE_DATA_ERROR;
//     }
//
//     ExFreePoolWithTag(Urb, USB_HUB_TAG);
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_SyncGetStatus(IN PDEVICE_OBJECT DeviceObject,
//                    IN PUSHORT OutStatus,
//                    IN USHORT Function,
//                    IN USHORT RequestIndex)
// {
//     struct _URB_CONTROL_GET_STATUS_REQUEST * Urb;
//     NTSTATUS NtStatus;
//     USHORT UsbStatus;
//
//     DPRINT("USBH_SyncGetStatus: ... \n");
//
//     Urb = ExAllocatePoolWithTag(NonPagedPool,
//                                 sizeof(struct _URB_CONTROL_GET_STATUS_REQUEST),
//                                 USB_HUB_TAG);
//
//     if (!Urb)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlZeroMemory(Urb, sizeof(struct _URB_CONTROL_GET_STATUS_REQUEST));
//
//     Urb->Hdr.Length = sizeof(struct _URB_CONTROL_GET_STATUS_REQUEST);
//     Urb->Hdr.Function = Function;
//
//     Urb->TransferBuffer = &UsbStatus;
//     Urb->TransferBufferLength = sizeof(UsbStatus);
//     Urb->Index = RequestIndex;
//
//     NtStatus = USBH_FdoSyncSubmitUrb(DeviceObject, (PURB)Urb);
//
//     *OutStatus = UsbStatus;
//
//     ExFreePoolWithTag(Urb, USB_HUB_TAG);
//
//     return NtStatus;
// }
//
// NTSTATUS
// NTAPI
// USBH_SyncGetHubStatus(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                       IN PUSB_HUB_STATUS_AND_CHANGE HubStatus,
//                       IN ULONG Length)
// {
//     BM_REQUEST_TYPE RequestType;
//
//     DPRINT("USBH_SyncGetHubStatus\n");
//
//     RequestType.B = 0;
//     RequestType.Recipient = BMREQUEST_TO_DEVICE;
//     RequestType.Type = BMREQUEST_CLASS;
//     RequestType.Dir = BMREQUEST_DEVICE_TO_HOST;
//
//     return USBH_Transact(HubExtension,
//                          HubStatus,
//                          Length,
//                          BMREQUEST_DEVICE_TO_HOST,
//                          URB_FUNCTION_CLASS_DEVICE,
//                          RequestType,
//                          USB_REQUEST_GET_STATUS,
//                          0,
//                          0);
// }
//
// NTSTATUS
// NTAPI
// USBH_SyncClearHubStatus(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                         IN USHORT RequestValue)
// {
//     BM_REQUEST_TYPE RequestType;
//
//     DPRINT("USBH_SyncClearHubStatus: RequestValue - %x\n", RequestValue);
//
//     RequestType.B = 0;
//     RequestType.Recipient = BMREQUEST_TO_DEVICE;
//     RequestType.Type = BMREQUEST_CLASS;
//     RequestType.Dir = BMREQUEST_HOST_TO_DEVICE;
//
//     return USBH_Transact(HubExtension,
//                          NULL,
//                          0,
//                          BMREQUEST_HOST_TO_DEVICE,
//                          URB_FUNCTION_CLASS_DEVICE,
//                          RequestType,
//                          USB_REQUEST_CLEAR_FEATURE,
//                          RequestValue,
//                          0);
// }
//
// NTSTATUS
// NTAPI
// USBH_SyncGetPortStatus(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                        IN USHORT Port,
//                        IN PUSB_PORT_STATUS_AND_CHANGE PortStatus,
//                        IN ULONG Length)
// {
//     BM_REQUEST_TYPE RequestType;
//
//     DPRINT("USBH_SyncGetPortStatus: Port - %x\n", Port);
//
//     RequestType.B = 0;
//     RequestType.Recipient = BMREQUEST_TO_OTHER;
//     RequestType.Type = BMREQUEST_CLASS;
//     RequestType.Dir = BMREQUEST_DEVICE_TO_HOST;
//
//     return USBH_Transact(HubExtension,
//                          PortStatus,
//                          Length,
//                          BMREQUEST_DEVICE_TO_HOST,
//                          URB_FUNCTION_CLASS_OTHER,
//                          RequestType,
//                          USB_REQUEST_GET_STATUS,
//                          0,
//                          Port);
// }
//
//
// NTSTATUS
// NTAPI
// USBH_SyncClearPortStatus(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                          IN USHORT Port,
//                          IN USHORT RequestValue)
// {
//     BM_REQUEST_TYPE RequestType;
//
//     DPRINT("USBH_SyncClearPortStatus: Port - %x, RequestValue - %x\n",
//            Port,
//            RequestValue);
//
//     RequestType.B = 0;
//     RequestType.Recipient = BMREQUEST_TO_OTHER;
//     RequestType.Type = BMREQUEST_CLASS;
//     RequestType.Dir = BMREQUEST_HOST_TO_DEVICE;
//
//     return USBH_Transact(HubExtension,
//                          NULL,
//                          0,
//                          BMREQUEST_HOST_TO_DEVICE,
//                          URB_FUNCTION_CLASS_OTHER,
//                          RequestType,
//                          USB_REQUEST_CLEAR_FEATURE,
//                          RequestValue,
//                          Port);
// }
//
// NTSTATUS
// NTAPI
// USBH_SyncPowerOnPort(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                      IN USHORT Port,
//                      IN BOOLEAN IsWait)
// {
//     PUSBHUB_PORT_DATA PortData;
//     PUSB_HUB_DESCRIPTOR HubDescriptor;
//     NTSTATUS Status = STATUS_SUCCESS;
//     BM_REQUEST_TYPE RequestType;
//     PUSB_PORT_STATUS_AND_CHANGE PortStatus;
//
//     DPRINT("USBH_SyncPowerOnPort: Port - %x, IsWait - %x\n", Port, IsWait);
//
//     ASSERT(Port > 0);
//     PortData = &HubExtension->PortData[Port - 1];
//     PortStatus = &PortData->PortStatus;
//
//     if (PortStatus->PortStatus.Usb20PortStatus.CurrentConnectStatus == 1)
//     {
//         return Status;
//     }
//
//     RequestType.B = 0;
//     RequestType.Recipient = BMREQUEST_TO_DEVICE;
//     RequestType.Type = BMREQUEST_CLASS;
//     RequestType.Dir = BMREQUEST_HOST_TO_DEVICE;
//
//     Status = USBH_Transact(HubExtension,
//                            NULL,
//                            0,
//                            BMREQUEST_HOST_TO_DEVICE,
//                            URB_FUNCTION_CLASS_OTHER,
//                            RequestType,
//                            USB_REQUEST_SET_FEATURE,
//                            USBHUB_FEATURE_PORT_POWER,
//                            Port);
//
//     if (NT_SUCCESS(Status))
//     {
//         if (IsWait)
//         {
//             HubDescriptor = HubExtension->HubDescriptor;
//             USBH_Wait(2 * HubDescriptor->bPowerOnToPowerGood);
//         }
//
//         PortStatus->PortStatus.Usb20PortStatus.CurrentConnectStatus = 1;
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_SyncPowerOnPorts(IN PUSBHUB_FDO_EXTENSION HubExtension)
// {
//     PUSB_HUB_DESCRIPTOR HubDescriptor;
//     NTSTATUS Status = STATUS_UNSUCCESSFUL;
//     USHORT Port;
//     UCHAR NumberOfPorts;
//
//     DPRINT("USBH_SyncPowerOnPorts: ... \n");
//
//     HubDescriptor = HubExtension->HubDescriptor;
//     NumberOfPorts = HubDescriptor->bNumberOfPorts;
//
//     for (Port = 1; Port <= NumberOfPorts; ++Port)
//     {
//         Status = USBH_SyncPowerOnPort(HubExtension, Port, 0);
//
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("USBH_SyncPowerOnPorts: USBH_SyncPowerOnPort() failed - %lX\n",
//                     Status);
//             break;
//         }
//     }
//
//     USBH_Wait(2 * HubDescriptor->bPowerOnToPowerGood);
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_SyncDisablePort(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                      IN USHORT Port)
// {
//     PUSBHUB_PORT_DATA PortData;
//     NTSTATUS Status;
//     BM_REQUEST_TYPE RequestType;
//
//     DPRINT("USBH_SyncDisablePort ... \n");
//
//     PortData = &HubExtension->PortData[Port - 1];
//
//     RequestType.B = 0;
//     RequestType.Recipient = BMREQUEST_TO_DEVICE;
//     RequestType.Type = BMREQUEST_CLASS;
//     RequestType.Dir = BMREQUEST_HOST_TO_DEVICE;
//
//     Status = USBH_Transact(HubExtension,
//                            NULL,
//                            0,
//                            BMREQUEST_HOST_TO_DEVICE,
//                            URB_FUNCTION_CLASS_OTHER,
//                            RequestType,
//                            USB_REQUEST_CLEAR_FEATURE,
//                            USBHUB_FEATURE_PORT_ENABLE,
//                            Port);
//
//     if (NT_SUCCESS(Status))
//     {
//         PortData->PortStatus.PortStatus.Usb20PortStatus.PortEnabledDisabled = 0;
//     }
//
//     return Status;
// }
//
// BOOLEAN
// NTAPI
// USBH_HubIsBusPowered(IN PDEVICE_OBJECT DeviceObject,
//                      IN PUSB_CONFIGURATION_DESCRIPTOR HubConfigDescriptor)
// {
//     BOOLEAN Result;
//     USHORT UsbStatus;
//     NTSTATUS Status;
//
//     DPRINT("USBH_HubIsBusPowered: ... \n");
//
//     Status = USBH_SyncGetStatus(DeviceObject,
//                                 &UsbStatus,
//                                 URB_FUNCTION_GET_STATUS_FROM_DEVICE,
//                                 0);
//
//     if (!NT_SUCCESS(Status))
//     {
//         Result = (HubConfigDescriptor->bmAttributes & USB_CONFIG_POWERED_MASK)
//                                                    == USB_CONFIG_BUS_POWERED;
//     }
//     else
//     {
//         Result = (UsbStatus & USB_GETSTATUS_SELF_POWERED) == 0;
//     }
//
//     return Result;
// }
//
// NTSTATUS
// NTAPI
// USBH_ChangeIndicationAckChangeComplete(IN PDEVICE_OBJECT DeviceObject,
//                                        IN PIRP Irp,
//                                        IN PVOID Context)
// {
//     PUSBHUB_FDO_EXTENSION HubExtension;
//     PVOID Event;
//     USHORT Port;
//
//     HubExtension = Context;
//
//     DPRINT_SCE!("USBH_ChangeIndicationAckChangeComplete: ... \n");
//
//     ASSERT(HubExtension->Port > 0);
//     Port = HubExtension->Port - 1;
//
//     HubExtension->PortData[Port].PortStatus = HubExtension->PortStatus;
//
//     Event = InterlockedExchangePointer((PVOID)&HubExtension->pResetPortEvent,
//                                        NULL);
//
//     if (Event)
//     {
//         KeSetEvent(Event, EVENT_INCREMENT, FALSE);
//     }
//
//     USBH_SubmitStatusChangeTransfer(HubExtension);
//
//     if (!InterlockedDecrement(&HubExtension->ResetRequestCount))
//     {
//         KeSetEvent(&HubExtension->ResetEvent,
//                    EVENT_INCREMENT,
//                    FALSE);
//     }
//
//     return STATUS_MORE_PROCESSING_REQUIRED;
// }
//
// NTSTATUS
// NTAPI
// USBH_ChangeIndicationAckChange(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                                IN PIRP Irp,
//                                IN struct _URB_CONTROL_VENDOR_OR_CLASS_REQUEST * Urb,
//                                IN USHORT Port,
//                                IN USHORT RequestValue)
// {
//     PIO_STACK_LOCATION IoStack;
//     BM_REQUEST_TYPE RequestType;
//
//     DPRINT_SCE!("USBH_ChangeIndicationAckChange: ... \n");
//
//     Urb->Hdr.Length = sizeof(struct _URB_CONTROL_VENDOR_OR_CLASS_REQUEST);
//     Urb->Hdr.Function = URB_FUNCTION_CLASS_OTHER;
//     Urb->Hdr.UsbdDeviceHandle = NULL;
//
//     Urb->TransferFlags = USBD_SHORT_TRANSFER_OK;
//     Urb->TransferBufferLength = 0;
//     Urb->TransferBuffer = NULL;
//     Urb->TransferBufferMDL = NULL;
//     Urb->UrbLink = NULL;
//
//     RequestType.B = 0;
//     RequestType.Recipient = BMREQUEST_TO_OTHER;
//     RequestType.Type = BMREQUEST_CLASS;
//     RequestType.Dir = BMREQUEST_HOST_TO_DEVICE;
//
//     Urb->RequestTypeReservedBits = RequestType.B;
//     Urb->Request = USB_REQUEST_CLEAR_FEATURE;
//     Urb->Index = Port;
//     Urb->Value = RequestValue;
//
//     IoInitializeIrp(Irp,
//                     IoSizeOfIrp(HubExtension->LowerDevice->StackSize),
//                     HubExtension->LowerDevice->StackSize);
//
//     IoStack = IoGetNextIrpStackLocation(Irp);
//
//     IoStack->MajorFunction = IRP_MJ_INTERNAL_DEVICE_CONTROL;
//     IoStack->Parameters.Others.Argument1 = Urb;
//     IoStack->Parameters.DeviceIoControl.IoControlCode = IOCTL_INTERNAL_USB_SUBMIT_URB;
//
//     IoSetCompletionRoutine(Irp,
//                            USBH_ChangeIndicationAckChangeComplete,
//                            HubExtension,
//                            TRUE,
//                            TRUE,
//                            TRUE);
//
//     return IoCallDriver(HubExtension->LowerDevice, Irp);
// }
//
// NTSTATUS
// NTAPI
// USBH_ChangeIndicationProcessChange(IN PDEVICE_OBJECT DeviceObject,
//                                    IN PIRP Irp,
//                                    IN PVOID Context)
// {
//     PUSBHUB_FDO_EXTENSION HubExtension;
//     PUSBHUB_IO_WORK_ITEM WorkItem;
//     USHORT RequestValue;
//
//     HubExtension = Context;
//
//     DPRINT_SCE!("USBH_ChangeIndicationProcessChange: PortStatus - %lX\n",
//                HubExtension->PortStatus.AsUlong32);
//
//     if ((NT_SUCCESS(Irp->IoStatus.Status) ||
//         USBD_SUCCESS(HubExtension->SCEWorkerUrb.Hdr.Status)) &&
//         (HubExtension->PortStatus.PortChange.Usb20PortChange.ResetChange ||
//          HubExtension->PortStatus.PortChange.Usb20PortChange.PortEnableDisableChange))
//     {
//         if (!InterlockedDecrement(&HubExtension->PendingRequestCount))
//         {
//             KeSetEvent(&HubExtension->PendingRequestEvent,
//                        EVENT_INCREMENT,
//                        FALSE);
//         }
//
//         USBH_FreeWorkItem(HubExtension->WorkItemToQueue);
//
//         HubExtension->WorkItemToQueue = NULL;
//
//         if (HubExtension->PortStatus.PortChange.Usb20PortChange.ResetChange)
//         {
//            RequestValue = USBHUB_FEATURE_C_PORT_RESET;
//         }
//         else
//         {
//             RequestValue = USBHUB_FEATURE_C_PORT_ENABLE;
//         }
//
//         USBH_ChangeIndicationAckChange(HubExtension,
//                                        HubExtension->ResetPortIrp,
//                                        &HubExtension->SCEWorkerUrb,
//                                        HubExtension->Port,
//                                        RequestValue);
//     }
//     else
//     {
//         ASSERT(HubExtension->WorkItemToQueue != NULL);
//
//         WorkItem = HubExtension->WorkItemToQueue;
//         HubExtension->WorkItemToQueue = NULL;
//
//         USBH_QueueWorkItem(HubExtension, WorkItem);
//     }
//
//     return STATUS_MORE_PROCESSING_REQUIRED;
// }
//
// NTSTATUS
// NTAPI
// USBH_ChangeIndicationQueryChange(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                                  IN PIRP Irp,
//                                  IN struct _URB_CONTROL_VENDOR_OR_CLASS_REQUEST * Urb,
//                                  IN USHORT Port)
// {
//     PUSBHUB_IO_WORK_ITEM WorkItem;
//     NTSTATUS Status;
//     PIO_STACK_LOCATION IoStack;
//     BM_REQUEST_TYPE RequestType;
//
//     DPRINT_SCE!("USBH_ChangeIndicationQueryChange: Port - %x\n", Port);
//
//     InterlockedIncrement(&HubExtension->PendingRequestCount);
//
//     if (!Port)
//     {
//         ASSERT(HubExtension->WorkItemToQueue != NULL);
//
//         WorkItem = HubExtension->WorkItemToQueue;
//         HubExtension->WorkItemToQueue = NULL;
//
//         USBH_QueueWorkItem(HubExtension, WorkItem);
//
//         return STATUS_SUCCESS;
//     }
//
//     Urb->Hdr.Length = sizeof(struct _URB_CONTROL_VENDOR_OR_CLASS_REQUEST);
//     Urb->Hdr.UsbdDeviceHandle = NULL;
//     Urb->Hdr.Function = URB_FUNCTION_CLASS_OTHER;
//
//     Urb->TransferFlags = USBD_SHORT_TRANSFER_OK | USBD_TRANSFER_DIRECTION_IN;
//     Urb->TransferBuffer = &HubExtension->PortStatus;
//     Urb->TransferBufferLength = sizeof(HubExtension->PortStatus);
//     Urb->TransferBufferMDL = NULL;
//     Urb->UrbLink = NULL;
//
//     RequestType.B = 0;
//     RequestType.Recipient = BMREQUEST_TO_OTHER;
//     RequestType.Type = BMREQUEST_CLASS;
//     RequestType.Dir = BMREQUEST_DEVICE_TO_HOST;
//
//     Urb->RequestTypeReservedBits = RequestType.B;
//     Urb->Request = USB_REQUEST_GET_STATUS;
//     Urb->Value = 0;
//     Urb->Index = Port;
//
//     HubExtension->Port = Port;
//
//     IoInitializeIrp(Irp,
//                     IoSizeOfIrp(HubExtension->LowerDevice->StackSize),
//                     HubExtension->LowerDevice->StackSize);
//
//     IoStack = IoGetNextIrpStackLocation(Irp);
//
//     IoStack->MajorFunction = IRP_MJ_INTERNAL_DEVICE_CONTROL;
//     IoStack->Parameters.Others.Argument1 = Urb;
//     IoStack->Parameters.DeviceIoControl.IoControlCode = IOCTL_INTERNAL_USB_SUBMIT_URB;
//
//     IoSetCompletionRoutine(Irp,
//                            USBH_ChangeIndicationProcessChange,
//                            HubExtension,
//                            TRUE,
//                            TRUE,
//                            TRUE);
//
//     Status = IoCallDriver(HubExtension->LowerDevice, Irp);
//
//     return Status;
// }
//
// VOID
// NTAPI
// USBH_ProcessHubStateChange(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                            IN PUSB_HUB_STATUS_AND_CHANGE HubStatus)
// {
//     USB_HUB_CHANGE HubStatusChange;
//
//     DPRINT_SCE!("USBH_ProcessHubStateChange: HubStatus - %lx\n", HubStatus->AsUlong32);
//
//     HubStatusChange = HubStatus->HubChange;
//
//     if (HubStatusChange.LocalPowerChange)
//     {
//         DPRINT1("USBH_ProcessHubStateChange: LocalPowerChange\n");
//         USBH_SyncClearHubStatus(HubExtension,
//                                 USBHUB_FEATURE_C_HUB_LOCAL_POWER);
//     }
//     /*
//      * Per USB 2.0 spec multiple hub change bits may be set simultaneously.
//      * Handle OverCurrentChange independently so we don’t miss clearing it
//      * when LocalPowerChange is also set.
//      */
//     if (HubStatusChange.OverCurrentChange)
//     {
//         USBH_SyncClearHubStatus(HubExtension,
//                                 USBHUB_FEATURE_C_HUB_OVER_CURRENT);
//         if (HubStatus->HubStatus.OverCurrent)
//         {
//             DPRINT1("USBH_ProcessHubStateChange: OverCurrent UNIMPLEMENTED. FIXME\n");
//             DbgBreakPoint();
//         }
//     }
// }
//
// VOID
// NTAPI
// USBH_ProcessPortStateChange(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                             IN USHORT Port,
//                             IN PUSB_PORT_STATUS_AND_CHANGE PortStatus)
// {
//     PUSBHUB_PORT_DATA PortData;
//     USB_20_PORT_CHANGE PortStatusChange;
//     PDEVICE_OBJECT PortDevice;
//     PUSBHUB_PORT_PDO_EXTENSION PortExtension;
//     PVOID SerialNumber;
//     PVOID DeviceHandle;
//     USHORT RequestValue;
//     KIRQL Irql;
//
//     DPRINT_SCE!("USBH_ProcessPortStateChange ... \n");
//
//     ASSERT(Port > 0);
//     PortData = &HubExtension->PortData[Port - 1];
//
//     PortStatusChange = PortStatus->PortChange.Usb20PortChange;
//
//     if (PortStatusChange.ConnectStatusChange)
//     {
//         PortData->PortStatus = *PortStatus;
//
//         USBH_SyncClearPortStatus(HubExtension,
//                                  Port,
//                                  USBHUB_FEATURE_C_PORT_CONNECTION);
//
//         PortData = &HubExtension->PortData[Port - 1];
//
//         PortDevice = PortData->DeviceObject;
//
//         if (!PortDevice)
//         {
//             IoInvalidateDeviceRelations(HubExtension->LowerPDO, BusRelations);
//             return;
//         }
//
//         PortExtension = PortDevice->DeviceExtension;
//
//         if (PortExtension->PortPdoFlags & USBHUB_PDO_FLAG_OVERCURRENT_PORT)
//         {
//             return;
//         }
//
//         KeAcquireSpinLock(&HubExtension->RelationsWorkerSpinLock, &Irql);
//
//         if (PortExtension->PortPdoFlags & USBHUB_PDO_FLAG_POWER_D3)
//         {
//             KeReleaseSpinLock(&HubExtension->RelationsWorkerSpinLock, Irql);
//             IoInvalidateDeviceRelations(HubExtension->LowerPDO, BusRelations);
//             return;
//         }
//
//         PortData->DeviceObject = NULL;
//         PortData->ConnectionStatus = NoDeviceConnected;
//
//         HubExtension->HubFlags |= USBHUB_FDO_FLAG_STATE_CHANGING;
//
//         InsertTailList(&HubExtension->PdoList, &PortExtension->PortLink);
//
//         KeReleaseSpinLock(&HubExtension->RelationsWorkerSpinLock, Irql);
//
//         SerialNumber = InterlockedExchangePointer((PVOID)&PortExtension->SerialNumber,
//                                                   NULL);
//
//         if (SerialNumber)
//         {
//             ExFreePoolWithTag(SerialNumber, USB_HUB_TAG);
//         }
//
//         DeviceHandle = InterlockedExchangePointer(&PortExtension->DeviceHandle,
//                                                   NULL);
//
//         if (DeviceHandle)
//         {
//             USBD_RemoveDeviceEx(HubExtension, DeviceHandle, 0);
//             USBH_SyncDisablePort(HubExtension, Port);
//         }
//
//         IoInvalidateDeviceRelations(HubExtension->LowerPDO, BusRelations);
//         return;
//     }
//
//     /*
//      * Multiple port change bits can be set at once (e.g. enable + reset).
//      * Handle each independently.
//      */
//     if (PortStatusChange.PortEnableDisableChange)
//     {
//         RequestValue = USBHUB_FEATURE_C_PORT_ENABLE;
//         PortData->PortStatus = *PortStatus;
//         USBH_SyncClearPortStatus(HubExtension, Port, RequestValue);
//     }
//
//     if (PortStatusChange.SuspendChange)
//     {
//         DPRINT1("USBH_ProcessPortStateChange: SuspendChange UNIMPLEMENTED. FIXME\n");
//         DbgBreakPoint();
//     }
//
//     if (PortStatusChange.OverCurrentIndicatorChange)
//     {
//         DPRINT1("USBH_ProcessPortStateChange: OverCurrentIndicatorChange UNIMPLEMENTED. FIXME\n");
//         DbgBreakPoint();
//     }
//
//     if (PortStatusChange.ResetChange)
//     {
//         RequestValue = USBHUB_FEATURE_C_PORT_RESET;
//         PortData->PortStatus = *PortStatus;
//         USBH_SyncClearPortStatus(HubExtension, Port, RequestValue);
//     }
// }
//
// NTSTATUS
// NTAPI
// USBH_GetPortStatus(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                    IN PULONG PortStatus)
// {
//     PIRP Irp;
//     PIO_STACK_LOCATION IoStack;
//     NTSTATUS Status;
//     KEVENT Event;
//     IO_STATUS_BLOCK IoStatusBlock;
//
//     DPRINT("USBH_GetPortStatus ... \n");
//
//     *PortStatus = 0;
//
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//     Irp = IoBuildDeviceIoControlRequest(IOCTL_INTERNAL_USB_GET_PORT_STATUS,
//                                         HubExtension->LowerDevice,
//                                         NULL,
//                                         0,
//                                         NULL,
//                                         0,
//                                         TRUE,
//                                         &Event,
//                                         &IoStatusBlock);
//
//     if (!Irp)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     IoStack = IoGetNextIrpStackLocation(Irp);
//     IoStack->Parameters.Others.Argument1 = PortStatus;
//
//     Status = IoCallDriver(HubExtension->LowerDevice, Irp);
//
//     if (Status == STATUS_PENDING)
//     {
//         KeWaitForSingleObject(&Event,
//                               Suspended,
//                               KernelMode,
//                               FALSE,
//                               NULL);
//     }
//     else
//     {
//         IoStatusBlock.Status = Status;
//     }
//
//     return IoStatusBlock.Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_EnableParentPort(IN PUSBHUB_FDO_EXTENSION HubExtension)
// {
//     PIRP Irp;
//     NTSTATUS Status;
//     KEVENT Event;
//     IO_STATUS_BLOCK IoStatusBlock;
//
//     DPRINT("USBH_EnableParentPort ... \n");
//
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//     Irp = IoBuildDeviceIoControlRequest(IOCTL_INTERNAL_USB_ENABLE_PORT,
//                                         HubExtension->LowerDevice,
//                                         NULL,
//                                         0,
//                                         NULL,
//                                         0,
//                                         TRUE,
//                                         &Event,
//                                         &IoStatusBlock);
//
//     if (!Irp)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     Status = IoCallDriver(HubExtension->LowerDevice, Irp);
//
//     if (Status == STATUS_PENDING)
//     {
//         KeWaitForSingleObject(&Event,
//                               Suspended,
//                               KernelMode,
//                               FALSE,
//                               NULL);
//     }
//     else
//     {
//         IoStatusBlock.Status = Status;
//     }
//
//     return IoStatusBlock.Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_ResetInterruptPipe(IN PUSBHUB_FDO_EXTENSION HubExtension)
// {
//     struct _URB_PIPE_REQUEST * Urb;
//     NTSTATUS Status;
//
//     DPRINT("USBH_ResetInterruptPipe ... \n");
//
//     Urb = ExAllocatePoolWithTag(NonPagedPool,
//                                 sizeof(struct _URB_PIPE_REQUEST),
//                                 USB_HUB_TAG);
//
//     if (Urb)
//     {
//         RtlZeroMemory(Urb, sizeof(struct _URB_PIPE_REQUEST));
//
//         Urb->Hdr.Length = sizeof(struct _URB_PIPE_REQUEST);
//         Urb->Hdr.Function = URB_FUNCTION_SYNC_RESET_PIPE_AND_CLEAR_STALL;
//         Urb->PipeHandle = HubExtension->PipeInfo.PipeHandle;
//
//         Status = USBH_FdoSyncSubmitUrb(HubExtension->Common.SelfDevice,
//                                        (PURB)Urb);
//
//         ExFreePoolWithTag(Urb, USB_HUB_TAG);
//     }
//     else
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     if (NT_SUCCESS(Status))
//     {
//         HubExtension->RequestErrors = 0;
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_ResetHub(IN PUSBHUB_FDO_EXTENSION HubExtension)
// {
//     NTSTATUS Status;
//     ULONG PortStatusFlags = 0;
//
//     DPRINT("USBH_ResetHub: ... \n");
//
//     Status = USBH_GetPortStatus(HubExtension, &PortStatusFlags);
//
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     if (!(PortStatusFlags & USBD_PORT_ENABLED))
//     {
//         if (PortStatusFlags & USBD_PORT_CONNECTED)
//         {
//             USBH_EnableParentPort(HubExtension);
//         }
//     }
//
//     Status = USBH_ResetInterruptPipe(HubExtension);
//
//     return Status;
// }
//
// VOID
// NTAPI
// USBH_ChangeIndicationWorker(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                             IN PVOID Context)
// {
//     PUSBHUB_FDO_EXTENSION LowerHubExtension;
//     PUSBHUB_PORT_PDO_EXTENSION LowerPortExtension;
//     PUSBHUB_STATUS_CHANGE_CONTEXT WorkItem;
//     USB_PORT_STATUS_AND_CHANGE PortStatus;
//     USB_HUB_STATUS_AND_CHANGE HubStatus;
//     NTSTATUS Status;
//     USHORT Port = 0;
//
//     DPRINT_SCE!("USBH_ChangeIndicationWorker ... \n");
//
//     WorkItem = Context;
//
//     KeWaitForSingleObject(&HubExtension->HubSemaphore,
//                           Executive,
//                           KernelMode,
//                           FALSE,
//                           NULL);
//
//     if (HubExtension->HubFlags & USBHUB_FDO_FLAG_DEVICE_STOPPING)
//     {
//         KeSetEvent(&HubExtension->StatusChangeEvent,
//                    EVENT_INCREMENT,
//                    FALSE);
//
//         goto Exit;
//     }
//
//     if (!HubExtension->RequestErrors)
//     {
//         goto Enum;
//     }
//
//     DPRINT_SCE!("USBH_ChangeIndicationWorker: RequestErrors - %x\n",
//                HubExtension->RequestErrors);
//
//     if (HubExtension->LowerPDO == HubExtension->RootHubPdo)
//     {
//         goto Enum;
//     }
//
//     LowerPortExtension = HubExtension->LowerPDO->DeviceExtension;
//
//     if (LowerPortExtension->PortPdoFlags & USBHUB_PDO_FLAG_POWER_D1_OR_D2)
//     {
//         goto Enum;
//     }
//
//     LowerHubExtension = LowerPortExtension->HubExtension;
//
//     if (!LowerHubExtension)
//     {
//         goto Enum;
//     }
//
//     Status = USBH_SyncGetPortStatus(LowerHubExtension,
//                                     LowerPortExtension->PortNumber,
//                                     &PortStatus,
//                                     sizeof(USB_PORT_STATUS_AND_CHANGE));
//
//     if (!NT_SUCCESS(Status) ||
//         !PortStatus.PortStatus.Usb20PortStatus.CurrentConnectStatus)
//     {
//         HubExtension->HubFlags |= USBHUB_FDO_FLAG_DEVICE_REMOVED;
//
//         KeSetEvent(&HubExtension->StatusChangeEvent,
//                    EVENT_INCREMENT,
//                    FALSE);
//
//         goto Exit;
//     }
//
//     if (!(HubExtension->HubFlags & USBHUB_FDO_FLAG_ESD_RECOVERING))
//     {
//         HubExtension->HubFlags |= USBHUB_FDO_FLAG_ESD_RECOVERING;
//
//         DPRINT("USBH_ChangeIndicationWorker: Starting ESD recovery\n");
//
//         /* Initiate ESD recovery: power cycle the hub (D3 -> D0) */
//         USBH_HubStartESDRecovery(HubExtension);
//
//         goto Exit;
//     }
//
// Enum:
//
//     if (WorkItem->IsRequestErrors)
//     {
//         USBH_ResetHub(HubExtension);
//     }
//     else
//     {
//         for (Port = 0;
//              Port < HubExtension->HubDescriptor->bNumberOfPorts;
//              Port++)
//         {
//             if (IsBitSet((PUCHAR)(WorkItem + 1), Port))
//             {
//                 break;
//             }
//         }
//
//         if (Port)
//         {
//             Status = USBH_SyncGetPortStatus(HubExtension,
//                                             Port,
//                                             &PortStatus,
//                                             sizeof(PortStatus));
//         }
//         else
//         {
//             Status = USBH_SyncGetHubStatus(HubExtension,
//                                            &HubStatus,
//                                            sizeof(HubStatus));
//         }
//
//         if (NT_SUCCESS(Status))
//         {
//             if (Port)
//             {
//                 USBH_ProcessPortStateChange(HubExtension,
//                                             Port,
//                                             &PortStatus);
//             }
//             else
//             {
//                 USBH_ProcessHubStateChange(HubExtension,
//                                            &HubStatus);
//             }
//         }
//         else
//         {
//             HubExtension->RequestErrors++;
//
//             if (HubExtension->RequestErrors > USBHUB_MAX_REQUEST_ERRORS)
//             {
//                 HubExtension->HubFlags |= USBHUB_FDO_FLAG_DEVICE_FAILED;
//                 goto Exit;
//             }
//         }
//     }
//
//     USBH_SubmitStatusChangeTransfer(HubExtension);
//
// Exit:
//
//     KeReleaseSemaphore(&HubExtension->HubSemaphore,
//                        LOW_REALTIME_PRIORITY,
//                        1,
//                        FALSE);
//
//     if (!InterlockedDecrement(&HubExtension->PendingRequestCount))
//     {
//         KeSetEvent(&HubExtension->PendingRequestEvent,
//                    EVENT_INCREMENT,
//                    FALSE);
//     }
//
//     if (!InterlockedDecrement((PLONG)&HubExtension->ResetRequestCount))
//     {
//         KeSetEvent(&HubExtension->ResetEvent,
//                    EVENT_INCREMENT,
//                    FALSE);
//
//         if (HubExtension->HubFlags & USBHUB_FDO_FLAG_DEFER_CHECK_IDLE)
//         {
//             USBH_CheckHubIdle(HubExtension);
//         }
//     }
// }
//
// NTSTATUS
// NTAPI
// USBH_ChangeIndication(IN PDEVICE_OBJECT DeviceObject,
//                       IN PIRP Irp,
//                       IN PVOID Context)
// {
//     PUSBHUB_FDO_EXTENSION HubExtension;
//     USBD_STATUS UrbStatus;
//     BOOLEAN IsErrors = FALSE;
//     PUSBHUB_IO_WORK_ITEM HubWorkItem;
//     PUSBHUB_STATUS_CHANGE_CONTEXT HubWorkItemBuffer;
//     USHORT NumPorts;
//     USHORT Port;
//     NTSTATUS Status;
//     PVOID Bitmap;
//     ULONG BufferLength;
//
//     HubExtension = Context;
//     UrbStatus = HubExtension->SCEWorkerUrb.Hdr.Status;
//
//     DPRINT_SCE!("USBH_ChangeIndication: IrpStatus - %x, UrbStatus - %x, HubFlags - %lX\n",
//                Irp->IoStatus.Status,
//                UrbStatus,
//                HubExtension->HubFlags);
//
//     if (NT_ERROR(Irp->IoStatus.Status) || USBD_ERROR(UrbStatus) ||
//        (HubExtension->HubFlags & USBHUB_FDO_FLAG_DEVICE_FAILED) ||
//        (HubExtension->HubFlags & USBHUB_FDO_FLAG_DEVICE_STOPPING))
//     {
//         HubExtension->RequestErrors++;
//
//         IsErrors = TRUE;
//
//         KeSetEvent(&HubExtension->StatusChangeEvent,
//                    EVENT_INCREMENT,
//                    FALSE);
//
//         if (HubExtension->HubFlags & USBHUB_FDO_FLAG_DEVICE_STOPPING ||
//             HubExtension->HubFlags & USBHUB_FDO_FLAG_DEVICE_FAILED ||
//             HubExtension->RequestErrors > USBHUB_MAX_REQUEST_ERRORS ||
//             Irp->IoStatus.Status == STATUS_DELETE_PENDING)
//         {
//             DPRINT_SCE!("USBH_ChangeIndication: HubExtension->RequestErrors - %x\n",
//                        HubExtension->RequestErrors);
//
//             return STATUS_MORE_PROCESSING_REQUIRED;
//         }
//
//         DPRINT_SCE!("USBH_ChangeIndication: HubExtension->RequestErrors - %x\n",
//                    HubExtension->RequestErrors);
//     }
//     else
//     {
//         HubExtension->RequestErrors = 0;
//     }
//
//     BufferLength = sizeof(USBHUB_STATUS_CHANGE_CONTEXT) +
//                    HubExtension->SCEBitmapLength;
//
//     Status = USBH_AllocateWorkItem(HubExtension,
//                                    &HubWorkItem,
//                                    USBH_ChangeIndicationWorker,
//                                    BufferLength,
//                                    (PVOID *)&HubWorkItemBuffer,
//                                    DelayedWorkQueue);
//
//     if (!NT_SUCCESS(Status))
//     {
//         return STATUS_MORE_PROCESSING_REQUIRED;
//     }
//
//     RtlZeroMemory(HubWorkItemBuffer, BufferLength);
//
//     HubWorkItemBuffer->IsRequestErrors = FALSE;
//
//     if (IsErrors)
//     {
//         HubWorkItemBuffer->IsRequestErrors = TRUE;
//     }
//
//     if (InterlockedIncrement(&HubExtension->ResetRequestCount) == 1)
//     {
//         KeClearEvent(&HubExtension->ResetEvent);
//     }
//
//     HubWorkItemBuffer->HubExtension = HubExtension;
//
//     HubExtension->WorkItemToQueue = HubWorkItem;
//
//     Bitmap = HubWorkItemBuffer + 1;
//
//     RtlCopyMemory(Bitmap,
//                   HubExtension->SCEBitmap,
//                   HubExtension->SCEBitmapLength);
//
//     NumPorts = HubExtension->HubDescriptor->bNumberOfPorts;
//
//     for (Port = 0; Port <= NumPorts; ++Port)
//     {
//         if (IsBitSet(Bitmap, Port))
//         {
//             break;
//         }
//     }
//
//     if (Port > NumPorts)
//     {
//         Port = 0;
//     }
//
//     Status = USBH_ChangeIndicationQueryChange(HubExtension,
//                                               HubExtension->ResetPortIrp,
//                                               &HubExtension->SCEWorkerUrb,
//                                               Port);
//
//     if (NT_ERROR(Status))
//     {
//         HubExtension->HubFlags |= USBHUB_FDO_FLAG_DEVICE_FAILED;
//     }
//
//     return STATUS_MORE_PROCESSING_REQUIRED;
// }
//
// NTSTATUS
// NTAPI
// USBH_SubmitStatusChangeTransfer(IN PUSBHUB_FDO_EXTENSION HubExtension)
// {
//     PIRP Irp;
//     NTSTATUS Status;
//     struct _URB_BULK_OR_INTERRUPT_TRANSFER * Urb;
//     PIO_STACK_LOCATION IoStack;
//
//     DPRINT_SCE!("USBH_SubmitStatusChangeTransfer: HubExtension - %p, SCEIrp - %p\n",
//                HubExtension,
//                HubExtension->SCEIrp);
//
//     if (HubExtension->HubFlags & USBHUB_FDO_FLAG_NOT_D0_STATE)
//     {
//         DPRINT_SCE!("USBH_SubmitStatusChangeTransfer: USBHUB_FDO_FLAG_NOT_D0_STATE\n");
//         DPRINT_SCE!("USBH_SubmitStatusChangeTransfer: HubFlags - %lX\n",
//                    HubExtension->HubFlags);
//
//         return STATUS_INVALID_DEVICE_STATE;
//     }
//
//     Irp = HubExtension->SCEIrp;
//
//     if (!Irp)
//     {
//         return STATUS_INVALID_DEVICE_STATE;
//     }
//
//     Urb = (struct _URB_BULK_OR_INTERRUPT_TRANSFER *)&HubExtension->SCEWorkerUrb;
//
//     Urb->Hdr.Length = sizeof(struct _URB_BULK_OR_INTERRUPT_TRANSFER);
//     Urb->Hdr.Function = URB_FUNCTION_BULK_OR_INTERRUPT_TRANSFER;
//     Urb->Hdr.UsbdDeviceHandle = NULL;
//
//     Urb->PipeHandle = HubExtension->PipeInfo.PipeHandle;
//     Urb->TransferFlags = USBD_SHORT_TRANSFER_OK;
//     Urb->TransferBuffer = HubExtension->SCEBitmap;
//     Urb->TransferBufferLength = HubExtension->SCEBitmapLength;
//     Urb->TransferBufferMDL = NULL;
//     Urb->UrbLink = NULL;
//
//     IoInitializeIrp(Irp,
//                     IoSizeOfIrp(HubExtension->LowerDevice->StackSize),
//                     HubExtension->LowerDevice->StackSize);
//
//     IoStack = IoGetNextIrpStackLocation(Irp);
//
//     IoStack->MajorFunction = IRP_MJ_INTERNAL_DEVICE_CONTROL;
//     IoStack->Parameters.Others.Argument1 = &HubExtension->SCEWorkerUrb;
//     IoStack->Parameters.DeviceIoControl.IoControlCode = IOCTL_INTERNAL_USB_SUBMIT_URB;
//
//     IoSetCompletionRoutine(Irp,
//                            USBH_ChangeIndication,
//                            HubExtension,
//                            TRUE,
//                            TRUE,
//                            TRUE);
//
//     KeClearEvent(&HubExtension->StatusChangeEvent);
//
//     Status = IoCallDriver(HubExtension->LowerDevice, Irp);
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBD_CreateDeviceEx(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                     IN PUSB_DEVICE_HANDLE * OutDeviceHandle,
//                     IN USB_PORT_STATUS UsbPortStatus,
//                     IN USHORT Port)
// {
//     PUSB_DEVICE_HANDLE HubDeviceHandle;
//     PUSB_BUSIFFN_CREATE_USB_DEVICE CreateUsbDevice;
//
//     DPRINT("USBD_CreateDeviceEx: Port - %x, UsbPortStatus - 0x%04X\n",
//            Port,
//            UsbPortStatus.AsUshort16);
//
//     CreateUsbDevice = HubExtension->BusInterface.CreateUsbDevice;
//
//     if (!CreateUsbDevice)
//     {
//         return STATUS_NOT_IMPLEMENTED;
//     }
//
//     HubDeviceHandle = USBH_SyncGetDeviceHandle(HubExtension->LowerDevice);
//
//     return CreateUsbDevice(HubExtension->BusInterface.BusContext,
//                            OutDeviceHandle,
//                            HubDeviceHandle,
//                            UsbPortStatus.AsUshort16,
//                            Port);
// }
//
// NTSTATUS
// NTAPI
// USBD_RemoveDeviceEx(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                     IN PUSB_DEVICE_HANDLE DeviceHandle,
//                     IN ULONG Flags)
// {
//     PUSB_BUSIFFN_REMOVE_USB_DEVICE RemoveUsbDevice;
//
//     DPRINT("USBD_RemoveDeviceEx: DeviceHandle - %p, Flags - %X\n",
//            DeviceHandle,
//            Flags);
//
//     RemoveUsbDevice = HubExtension->BusInterface.RemoveUsbDevice;
//
//     if (!RemoveUsbDevice)
//     {
//         return STATUS_NOT_IMPLEMENTED;
//     }
//
//     return RemoveUsbDevice(HubExtension->BusInterface.BusContext,
//                            DeviceHandle,
//                            Flags);
// }
//
// NTSTATUS
// NTAPI
// USBD_InitializeDeviceEx(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                         IN PUSB_DEVICE_HANDLE DeviceHandle,
//                         IN PUCHAR DeviceDescriptorBuffer,
//                         IN ULONG DeviceDescriptorBufferLength,
//                         IN PUCHAR ConfigDescriptorBuffer,
//                         IN ULONG ConfigDescriptorBufferLength)
// {
//     NTSTATUS Status;
//     PUSB_BUSIFFN_INITIALIZE_USB_DEVICE InitializeUsbDevice;
//     PUSB_BUSIFFN_GET_USB_DESCRIPTORS GetUsbDescriptors;
//
//     DPRINT("USBD_InitializeDeviceEx: ... \n");
//
//     InitializeUsbDevice = HubExtension->BusInterface.InitializeUsbDevice;
//     GetUsbDescriptors = HubExtension->BusInterface.GetUsbDescriptors;
//
//     if (!InitializeUsbDevice || !GetUsbDescriptors)
//     {
//         return STATUS_NOT_IMPLEMENTED;
//     }
//
//     Status = InitializeUsbDevice(HubExtension->BusInterface.BusContext,
//                                  DeviceHandle);
//
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     return GetUsbDescriptors(HubExtension->BusInterface.BusContext,
//                              DeviceHandle,
//                              DeviceDescriptorBuffer,
//                              &DeviceDescriptorBufferLength,
//                              ConfigDescriptorBuffer,
//                              &ConfigDescriptorBufferLength);
// }
//
// VOID
// NTAPI
// USBHUB_SetDeviceHandleData(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                            IN PDEVICE_OBJECT UsbDevicePdo,
//                            IN PVOID DeviceHandle)
// {
//     PUSB_BUSIFFN_SET_DEVHANDLE_DATA SetDeviceHandleData;
//
//     DPRINT("USBHUB_SetDeviceHandleData ... \n");
//
//     SetDeviceHandleData = HubExtension->BusInterface.SetDeviceHandleData;
//
//     if (!SetDeviceHandleData)
//     {
//         return;
//     }
//
//     SetDeviceHandleData(HubExtension->BusInterface.BusContext,
//                         DeviceHandle,
//                         UsbDevicePdo);
// }
//
// VOID
// NTAPI
// USBHUB_FlushAllTransfers(IN PUSBHUB_FDO_EXTENSION HubExtension)
// {
//     PUSB_BUSIFFN_FLUSH_TRANSFERS FlushTransfers;
//
//     DPRINT("USBHUB_FlushAllTransfers ... \n");
//
//     FlushTransfers = HubExtension->BusInterface.FlushTransfers;
//
//     if (FlushTransfers)
//     {
//         FlushTransfers(HubExtension->BusInterface.BusContext, NULL);
//     }
// }
//
// NTSTATUS
// NTAPI
// USBD_GetDeviceInformationEx(IN PUSBHUB_PORT_PDO_EXTENSION PortExtension,
//                             IN PUSBHUB_FDO_EXTENSION HubExtension,
//                             IN PUSB_NODE_CONNECTION_INFORMATION_EX Info,
//                             IN ULONG Length,
//                             IN PUSB_DEVICE_HANDLE DeviceHandle)
// {
//     PUSB_BUSIFFN_GET_DEVICE_INFORMATION QueryDeviceInformation;
//     PUSB_DEVICE_INFORMATION_0 DeviceInfo;
//     SIZE_T DeviceInfoLength;
//     PUSB_NODE_CONNECTION_INFORMATION_EX NodeInfo;
//     SIZE_T NodeInfoLength;
//     ULONG PipeNumber;
//     ULONG dummy;
//     NTSTATUS Status;
//
//     DPRINT("USBD_GetDeviceInformationEx ... \n");
//
//     QueryDeviceInformation = HubExtension->BusInterface.QueryDeviceInformation;
//
//     if (!QueryDeviceInformation)
//     {
//         Status = STATUS_NOT_IMPLEMENTED;
//         return Status;
//     }
//
//     DeviceInfoLength = sizeof(USB_DEVICE_INFORMATION_0);
//
//     while (TRUE)
//     {
//         DeviceInfo = ExAllocatePoolWithTag(PagedPool,
//                                            DeviceInfoLength,
//                                            USB_HUB_TAG);
//
//         if (!DeviceInfo)
//         {
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         RtlZeroMemory(DeviceInfo, DeviceInfoLength);
//
//         DeviceInfo->InformationLevel = 0;
//
//         Status = QueryDeviceInformation(HubExtension->BusInterface.BusContext,
//                                         DeviceHandle,
//                                         DeviceInfo,
//                                         DeviceInfoLength,
//                                         &dummy);
//
//         if (Status != STATUS_BUFFER_TOO_SMALL)
//         {
//             break;
//         }
//
//         DeviceInfoLength = DeviceInfo->ActualLength;
//
//         ExFreePoolWithTag(DeviceInfo, USB_HUB_TAG);
//     }
//
//     NodeInfo = NULL;
//     NodeInfoLength = 0;
//
//     if (NT_SUCCESS(Status))
//     {
//         NodeInfoLength = (sizeof(USB_NODE_CONNECTION_INFORMATION_EX) - sizeof(USB_PIPE_INFO)) +
//                          DeviceInfo->NumberOfOpenPipes * sizeof(USB_PIPE_INFO);
//
//         NodeInfo = ExAllocatePoolWithTag(PagedPool, NodeInfoLength, USB_HUB_TAG);
//
//         if (!NodeInfo)
//         {
//             ExFreePoolWithTag(DeviceInfo, USB_HUB_TAG);
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         RtlZeroMemory(NodeInfo, NodeInfoLength);
//
//         NodeInfo->ConnectionIndex = Info->ConnectionIndex;
//
//         RtlCopyMemory(&NodeInfo->DeviceDescriptor,
//                       &DeviceInfo->DeviceDescriptor,
//                       sizeof(USB_DEVICE_DESCRIPTOR));
//
//         NodeInfo->CurrentConfigurationValue = DeviceInfo->CurrentConfigurationValue;
//         NodeInfo->Speed = DeviceInfo->DeviceSpeed;
//         NodeInfo->DeviceIsHub = PortExtension->PortPdoFlags & USBHUB_PDO_FLAG_HUB_DEVICE;
//         NodeInfo->DeviceAddress = DeviceInfo->DeviceAddress;
//         NodeInfo->NumberOfOpenPipes = DeviceInfo->NumberOfOpenPipes;
//         NodeInfo->ConnectionStatus = Info->ConnectionStatus;
//
//         for (PipeNumber = 0;
//              PipeNumber < DeviceInfo->NumberOfOpenPipes;
//              PipeNumber++)
//         {
//             RtlCopyMemory(&NodeInfo->PipeList[PipeNumber],
//                           &DeviceInfo->PipeList[PipeNumber],
//                           sizeof(USB_PIPE_INFO));
//         }
//     }
//
//     ExFreePoolWithTag(DeviceInfo, USB_HUB_TAG);
//
//     if (NodeInfo)
//     {
//         if (NodeInfoLength <= Length)
//         {
//             Length = NodeInfoLength;
//         }
//         else
//         {
//             Status = STATUS_BUFFER_TOO_SMALL;
//         }
//
//         RtlCopyMemory(Info, NodeInfo, Length);
//
//         ExFreePoolWithTag(NodeInfo, USB_HUB_TAG);
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBD_RestoreDeviceEx(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                      IN OUT PUSB_DEVICE_HANDLE OldDeviceHandle,
//                      IN OUT PUSB_DEVICE_HANDLE NewDeviceHandle)
// {
//     PUSB_BUSIFFN_RESTORE_DEVICE RestoreUsbDevice;
//     NTSTATUS Status;
//
//     DPRINT("USBD_RestoreDeviceEx: HubExtension - %p, OldDeviceHandle - %p, NewDeviceHandle - %p\n",
//            HubExtension,
//            OldDeviceHandle,
//            NewDeviceHandle);
//
//     RestoreUsbDevice = HubExtension->BusInterface.RestoreUsbDevice;
//
//     if (RestoreUsbDevice)
//     {
//         Status = RestoreUsbDevice(HubExtension->BusInterface.BusContext,
//                                   OldDeviceHandle,
//                                   NewDeviceHandle);
//     }
//     else
//     {
//         Status = STATUS_NOT_IMPLEMENTED;
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_AllocateWorkItem(PUSBHUB_FDO_EXTENSION HubExtension,
//                       PUSBHUB_IO_WORK_ITEM * OutHubIoWorkItem,
//                       PUSBHUB_WORKER_ROUTINE WorkerRoutine,
//                       SIZE_T BufferLength,
//                       PVOID * OutHubWorkItemBuffer,
//                       WORK_QUEUE_TYPE Type)
// {
//     PUSBHUB_IO_WORK_ITEM HubIoWorkItem;
//     PIO_WORKITEM WorkItem;
//     PVOID WorkItemBuffer;
//
//     DPRINT("USBH_AllocateWorkItem: ... \n");
//
//     if (!(HubExtension->HubFlags & USBHUB_FDO_FLAG_WITEM_INIT))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     HubIoWorkItem = ExAllocatePoolWithTag(NonPagedPool,
//                                           sizeof(USBHUB_IO_WORK_ITEM),
//                                           USB_HUB_TAG);
//
//     if (!HubIoWorkItem)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlZeroMemory(HubIoWorkItem, sizeof(USBHUB_IO_WORK_ITEM));
//
//     WorkItem = IoAllocateWorkItem(HubExtension->Common.SelfDevice);
//
//     HubIoWorkItem->HubWorkItem = WorkItem;
//
//     if (!WorkItem)
//     {
//         ExFreePoolWithTag(HubIoWorkItem, USB_HUB_TAG);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     if (BufferLength && OutHubWorkItemBuffer)
//     {
//         WorkItemBuffer = ExAllocatePoolWithTag(NonPagedPool,
//                                                BufferLength,
//                                                USB_HUB_TAG);
//
//         HubIoWorkItem->HubWorkItemBuffer = WorkItemBuffer;
//
//         if (!WorkItemBuffer)
//         {
//             IoFreeWorkItem(HubIoWorkItem->HubWorkItem);
//             ExFreePoolWithTag(HubIoWorkItem, USB_HUB_TAG);
//
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         RtlZeroMemory(WorkItemBuffer, BufferLength);
//     }
//     else
//     {
//         HubIoWorkItem->HubWorkItemBuffer = NULL;
//     }
//
//     HubIoWorkItem->HubWorkItemType = Type;
//     HubIoWorkItem->HubExtension = HubExtension;
//     HubIoWorkItem->HubWorkerRoutine = WorkerRoutine;
//
//     if (OutHubIoWorkItem)
//     {
//         *OutHubIoWorkItem = HubIoWorkItem;
//     }
//
//     if (OutHubWorkItemBuffer)
//     {
//         *OutHubWorkItemBuffer = HubIoWorkItem->HubWorkItemBuffer;
//     }
//
//     return STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// USBH_Worker(IN PDEVICE_OBJECT DeviceObject,
//             IN PVOID Context)
// {
//     PUSBHUB_IO_WORK_ITEM HubIoWorkItem;
//     PUSBHUB_FDO_EXTENSION HubExtension;
//     KIRQL OldIrql;
//     PIO_WORKITEM WorkItem;
//
//     DPRINT("USBH_Worker: HubIoWorkItem - %p\n", Context);
//
//     HubIoWorkItem = Context;
//
//     InterlockedDecrement(&HubIoWorkItem->HubWorkerQueued);
//
//     HubExtension = HubIoWorkItem->HubExtension;
//     WorkItem = HubIoWorkItem->HubWorkItem;
//
//     HubIoWorkItem->HubWorkerRoutine(HubIoWorkItem->HubExtension,
//                                     HubIoWorkItem->HubWorkItemBuffer);
//
//     KeAcquireSpinLock(&HubExtension->WorkItemSpinLock, &OldIrql);
//     RemoveEntryList(&HubIoWorkItem->HubWorkItemLink);
//     KeReleaseSpinLock(&HubExtension->WorkItemSpinLock, OldIrql);
//
//     if (HubIoWorkItem->HubWorkItemBuffer)
//     {
//         ExFreePoolWithTag(HubIoWorkItem->HubWorkItemBuffer, USB_HUB_TAG);
//     }
//
//     ExFreePoolWithTag(HubIoWorkItem, USB_HUB_TAG);
//
//     if (!InterlockedDecrement(&HubExtension->PendingRequestCount))
//     {
//         KeSetEvent(&HubExtension->PendingRequestEvent,
//                    EVENT_INCREMENT,
//                    FALSE);
//     }
//
//     IoFreeWorkItem(WorkItem);
//
//     DPRINT("USBH_Worker: HubIoWorkItem %p complete\n", Context);
// }
//
// VOID
// NTAPI
// USBH_QueueWorkItem(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                    IN PUSBHUB_IO_WORK_ITEM HubIoWorkItem)
// {
//     DPRINT("USBH_QueueWorkItem: ... \n");
//
//     InterlockedIncrement(&HubExtension->PendingRequestCount);
//     InterlockedIncrement(&HubIoWorkItem->HubWorkerQueued);
//
//     ExInterlockedInsertTailList(&HubExtension->WorkItemList,
//                                 &HubIoWorkItem->HubWorkItemLink,
//                                 &HubExtension->WorkItemSpinLock);
//
//     IoQueueWorkItem(HubIoWorkItem->HubWorkItem,
//                     USBH_Worker,
//                     HubIoWorkItem->HubWorkItemType,
//                     HubIoWorkItem);
// }
//
// VOID
// NTAPI
// USBH_FreeWorkItem(IN PUSBHUB_IO_WORK_ITEM HubIoWorkItem)
// {
//     PIO_WORKITEM WorkItem;
//
//     DPRINT("USBH_FreeWorkItem: ... \n");
//
//     WorkItem = HubIoWorkItem->HubWorkItem;
//
//     if (HubIoWorkItem->HubWorkItemBuffer)
//     {
//         ExFreePoolWithTag(HubIoWorkItem->HubWorkItemBuffer, USB_HUB_TAG);
//     }
//
//     ExFreePoolWithTag(HubIoWorkItem, USB_HUB_TAG);
//
//     IoFreeWorkItem(WorkItem);
// }
//
// VOID
// NTAPI
// USBHUB_RootHubCallBack(IN PVOID Context)
// {
//     PUSBHUB_FDO_EXTENSION HubExtension;
//
//     DPRINT("USBHUB_RootHubCallBack: ... \n");
//
//     HubExtension = Context;
//
//     if (HubExtension->SCEIrp)
//     {
//         HubExtension->HubFlags |= (USBHUB_FDO_FLAG_DO_ENUMERATION |
//                                    USBHUB_FDO_FLAG_NOT_ENUMERATED);
//
//         USBH_SubmitStatusChangeTransfer(HubExtension);
//
//         IoInvalidateDeviceRelations(HubExtension->LowerPDO, BusRelations);
//     }
//     else
//     {
//         HubExtension->HubFlags |= USBHUB_FDO_FLAG_DO_ENUMERATION;
//     }
//
//     KeSetEvent(&HubExtension->RootHubNotificationEvent,
//                EVENT_INCREMENT,
//                FALSE);
// }
//
// NTSTATUS
// NTAPI
// USBD_RegisterRootHubCallBack(IN PUSBHUB_FDO_EXTENSION HubExtension)
// {
//     PUSB_BUSIFFN_ROOTHUB_INIT_NOTIFY RootHubInitNotification;
//
//     DPRINT("USBD_RegisterRootHubCallBack: ... \n");
//
//     RootHubInitNotification = HubExtension->BusInterface.RootHubInitNotification;
//
//     if (!RootHubInitNotification)
//     {
//         return STATUS_NOT_IMPLEMENTED;
//     }
//
//     KeClearEvent(&HubExtension->RootHubNotificationEvent);
//
//     return RootHubInitNotification(HubExtension->BusInterface.BusContext,
//                                    HubExtension,
//                                    USBHUB_RootHubCallBack);
// }
//
// NTSTATUS
// NTAPI
// USBD_UnRegisterRootHubCallBack(IN PUSBHUB_FDO_EXTENSION HubExtension)
// {
//     PUSB_BUSIFFN_ROOTHUB_INIT_NOTIFY RootHubInitNotification;
//     NTSTATUS Status;
//
//     DPRINT("USBD_UnRegisterRootHubCallBack ... \n");
//
//     RootHubInitNotification = HubExtension->BusInterface.RootHubInitNotification;
//
//     if (!RootHubInitNotification)
//     {
//         return STATUS_NOT_IMPLEMENTED;
//     }
//
//     Status = RootHubInitNotification(HubExtension->BusInterface.BusContext,
//                                      NULL,
//                                      NULL);
//
//     if (!NT_SUCCESS(Status))
//     {
//         KeWaitForSingleObject(&HubExtension->RootHubNotificationEvent,
//                               Executive,
//                               KernelMode,
//                               FALSE,
//                               NULL);
//     }
//
//     return Status;
// }
//
// VOID
// NTAPI
// USBH_HubSetDWakeCompletion(IN PDEVICE_OBJECT DeviceObject,
//                            IN UCHAR MinorFunction,
//                            IN POWER_STATE PowerState,
//                            IN PVOID Context,
//                            IN PIO_STATUS_BLOCK IoStatus)
// {
//     DPRINT("USBH_HubSetDWakeCompletion: ... \n");
//     KeSetEvent((PRKEVENT)Context, IO_NO_INCREMENT, FALSE);
// }
//
// VOID
// NTAPI
// USBH_HubQueuePortIdleIrps(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                           IN PLIST_ENTRY IdleList)
// {
//     PDEVICE_OBJECT PortDevice;
//     PUSBHUB_PORT_PDO_EXTENSION PortExtension;
//     PIRP IdleIrp;
//     PIRP HubIdleIrp;
//     ULONG NumPorts;
//     ULONG Port;
//     KIRQL Irql;
//
//     DPRINT("USBH_HubQueuePortIdleIrps ... \n");
//
//     InitializeListHead(IdleList);
//
//     IoAcquireCancelSpinLock(&Irql);
//
//     NumPorts = HubExtension->HubDescriptor->bNumberOfPorts;
//
//     for (Port = 0; Port < NumPorts; ++Port)
//     {
//         PortDevice = HubExtension->PortData[Port].DeviceObject;
//
//         if (PortDevice)
//         {
//             PortExtension = PortDevice->DeviceExtension;
//
//             IdleIrp = PortExtension->IdleNotificationIrp;
//             PortExtension->IdleNotificationIrp = NULL;
//
//             if (IdleIrp && IoSetCancelRoutine(IdleIrp, NULL))
//             {
//                 DPRINT1("USBH_HubQueuePortIdleIrps: IdleIrp != NULL. FIXME\n");
//                 DbgBreakPoint();
//             }
//         }
//     }
//
//     if (HubExtension->HubFlags & USBHUB_FDO_FLAG_WAIT_IDLE_REQUEST)
//     {
//         HubIdleIrp = HubExtension->PendingIdleIrp;
//         HubExtension->PendingIdleIrp = NULL;
//     }
//     else
//     {
//         HubIdleIrp = NULL;
//     }
//
//     IoReleaseCancelSpinLock(Irql);
//
//     if (HubIdleIrp)
//     {
//         USBH_HubCancelIdleIrp(HubExtension, HubIdleIrp);
//     }
// }
//
// VOID
// NTAPI
// USBH_HubCompleteQueuedPortIdleIrps(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                                    IN PLIST_ENTRY IdleList,
//                                    IN NTSTATUS NtStatus)
// {
//     DPRINT("USBH_HubCompleteQueuedPortIdleIrps ... \n");
//
//     while (!IsListEmpty(IdleList))
//     {
//         DPRINT1("USBH_HubCompleteQueuedPortIdleIrps: IdleList not Empty. FIXME\n");
//         DbgBreakPoint();
//     }
// }
//
// VOID
// NTAPI
// USBH_FlushPortPwrList(IN PUSBHUB_FDO_EXTENSION HubExtension)
// {
//     PDEVICE_OBJECT PortDevice;
//     PUSBHUB_PORT_PDO_EXTENSION PortExtension;
//     PLIST_ENTRY Entry;
//     ULONG Port;
//
//     DPRINT("USBH_FlushPortPwrList ... \n");
//
//     InterlockedIncrement((PLONG)&HubExtension->PendingRequestCount);
//
//     KeWaitForSingleObject(&HubExtension->ResetDeviceSemaphore,
//                           Executive,
//                           KernelMode,
//                           FALSE,
//                           NULL);
//
//     for (Port = 0; Port < HubExtension->HubDescriptor->bNumberOfPorts; ++Port)
//     {
//         PortDevice = HubExtension->PortData[Port].DeviceObject;
//
//         if (!PortDevice)
//         {
//             continue;
//         }
//
//         PortExtension = PortDevice->DeviceExtension;
//
//         InterlockedExchange((PLONG)&PortExtension->StateBehindD2, 0);
//
//         while (TRUE)
//         {
//             Entry = ExInterlockedRemoveHeadList(&PortExtension->PortPowerList,
//                                                 &PortExtension->PortPowerListSpinLock);
//
//             if (!Entry)
//             {
//                 break;
//             }
//
//             DPRINT1("USBH_FlushPortPwrList: PortPowerList FIXME\n");
//             DbgBreakPoint();
//         }
//     }
//
//     KeReleaseSemaphore(&HubExtension->ResetDeviceSemaphore,
//                        LOW_REALTIME_PRIORITY,
//                        1,
//                        FALSE);
//
//     if (!InterlockedDecrement((PLONG)&HubExtension->PendingRequestCount))
//     {
//         KeSetEvent(&HubExtension->PendingRequestEvent,
//                    EVENT_INCREMENT,
//                    FALSE);
//     }
// }
//
// VOID
// NTAPI
// USBH_HubCompletePortIdleIrps(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                              IN NTSTATUS NtStatus)
// {
//     LIST_ENTRY IdleList;
//
//     DPRINT("USBH_HubCompletePortIdleIrps ... \n");
//
//     if (HubExtension->HubFlags & USBHUB_FDO_FLAG_DEVICE_STARTED)
//     {
//         USBH_HubQueuePortIdleIrps(HubExtension, &IdleList);
//
//         USBH_HubCompleteQueuedPortIdleIrps(HubExtension,
//                                            &IdleList,
//                                            NtStatus);
//
//         USBH_FlushPortPwrList(HubExtension);
//     }
// }
//
// VOID
// NTAPI
// USBH_HubCancelIdleIrp(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                       IN PIRP IdleIrp)
// {
//     DPRINT("USBH_HubCancelIdleIrp ... \n");
//
//     IoCancelIrp(IdleIrp);
//
//     if (InterlockedExchange(&HubExtension->IdleRequestLock, 1))
//     {
//         IoFreeIrp(IdleIrp);
//     }
// }
//
// BOOLEAN
// NTAPI
// USBH_CheckIdleAbort(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                     IN BOOLEAN IsWait,
//                     IN BOOLEAN IsExtCheck)
// {
//     PDEVICE_OBJECT PdoDevice;
//     PUSBHUB_PORT_PDO_EXTENSION PortExtension;
//     PUSBHUB_PORT_DATA PortData;
//     ULONG Port;
//     BOOLEAN Result = FALSE;
//
//     DPRINT("USBH_CheckIdleAbort: ... \n");
//
//     InterlockedIncrement(&HubExtension->PendingRequestCount);
//
//     if (IsWait == TRUE)
//     {
//         KeWaitForSingleObject(&HubExtension->ResetDeviceSemaphore,
//                               Executive,
//                               KernelMode,
//                               FALSE,
//                               NULL);
//     }
//
//     PortData = HubExtension->PortData;
//
//     for (Port = 0; Port < HubExtension->HubDescriptor->bNumberOfPorts; Port++)
//     {
//         PdoDevice = PortData[Port].DeviceObject;
//
//         if (PdoDevice)
//         {
//             PortExtension = PdoDevice->DeviceExtension;
//
//             if (PortExtension->PoRequestCounter)
//             {
//                 Result = TRUE;
//                 goto Wait;
//             }
//         }
//     }
//
//     if (IsExtCheck == TRUE)
//     {
//         PortData = HubExtension->PortData;
//
//         for (Port = 0;
//              Port < HubExtension->HubDescriptor->bNumberOfPorts;
//              Port++)
//         {
//             PdoDevice = PortData[Port].DeviceObject;
//
//             if (PdoDevice)
//             {
//                 PortExtension = PdoDevice->DeviceExtension;
//                 InterlockedExchange(&PortExtension->StateBehindD2, 0);
//             }
//         }
//     }
//
// Wait:
//
//     if (IsWait == TRUE)
//     {
//         KeReleaseSemaphore(&HubExtension->ResetDeviceSemaphore,
//                            LOW_REALTIME_PRIORITY,
//                            1,
//                            FALSE);
//     }
//
//     if (!InterlockedDecrement(&HubExtension->PendingRequestCount))
//     {
//         KeSetEvent(&HubExtension->PendingRequestEvent,
//                    EVENT_INCREMENT,
//                    FALSE);
//     }
//
//     return Result;
// }
//
// VOID
// NTAPI
// USBH_FdoWaitWakeIrpCompletion(IN PDEVICE_OBJECT DeviceObject,
//                               IN UCHAR MinorFunction,
//                               IN POWER_STATE PowerState,
//                               IN PVOID Context,
//                               IN PIO_STATUS_BLOCK IoStatus)
// {
//     DPRINT("USBH_FdoWaitWakeIrpCompletion ... \n");
// }
//
// NTSTATUS
// NTAPI
// USBH_FdoSubmitWaitWakeIrp(IN PUSBHUB_FDO_EXTENSION HubExtension)
// {
//     POWER_STATE PowerState;
//     NTSTATUS Status;
//     PIRP Irp = NULL;
//     KIRQL Irql;
//
//     DPRINT("USBH_FdoSubmitWaitWakeIrp: ... \n");
//
//     PowerState.SystemState = HubExtension->SystemWake;
//     HubExtension->HubFlags |= USBHUB_FDO_FLAG_PENDING_WAKE_IRP;
//
//     InterlockedIncrement(&HubExtension->PendingRequestCount);
//     InterlockedExchange(&HubExtension->FdoWaitWakeLock, 0);
//
//     Status = PoRequestPowerIrp(HubExtension->LowerPDO,
//                                IRP_MN_WAIT_WAKE,
//                                PowerState,
//                                USBH_FdoWaitWakeIrpCompletion,
//                                HubExtension,
//                                &Irp);
//
//     IoAcquireCancelSpinLock(&Irql);
//
//     if (Status == STATUS_PENDING)
//     {
//         if (HubExtension->HubFlags & USBHUB_FDO_FLAG_PENDING_WAKE_IRP)
//         {
//             HubExtension->PendingWakeIrp = Irp;
//             DPRINT("USBH_FdoSubmitWaitWakeIrp: PendingWakeIrp - %p\n",
//                    HubExtension->PendingWakeIrp);
//         }
//     }
//     else
//     {
//         HubExtension->HubFlags &= ~USBHUB_FDO_FLAG_PENDING_WAKE_IRP;
//
//         if (!InterlockedDecrement(&HubExtension->PendingRequestCount))
//         {
//             KeSetEvent(&HubExtension->PendingRequestEvent,
//                        EVENT_INCREMENT,
//                        FALSE);
//         }
//     }
//
//     IoReleaseCancelSpinLock(Irql);
//
//     return Status;
// }
//
// VOID
// NTAPI
// USBH_FdoIdleNotificationCallback(IN PVOID Context)
// {
//     PUSBHUB_FDO_EXTENSION HubExtension;
//     PUSBHUB_PORT_DATA PortData;
//     PDEVICE_OBJECT PortDevice;
//     PUSBHUB_PORT_PDO_EXTENSION PortExtension;
//     PIRP Irp = NULL;
//     PIRP IdleIrp;
//     POWER_STATE PowerState;
//     KEVENT Event;
//     ULONG Port;
//     PIO_STACK_LOCATION IoStack;
//     PUSB_IDLE_CALLBACK_INFO CallbackInfo;
//     BOOLEAN IsReady;
//     KIRQL OldIrql;
//     NTSTATUS Status;
//
//     HubExtension = Context;
//
//     DPRINT("USBH_FdoIdleNotificationCallback: HubExtension - %p, HubFlags - %lX\n",
//            HubExtension,
//            HubExtension->HubFlags);
//
//     if (HubExtension->HubFlags & (USBHUB_FDO_FLAG_ENUM_POST_RECOVER |
//                                   USBHUB_FDO_FLAG_WAKEUP_START |
//                                   USBHUB_FDO_FLAG_DEVICE_REMOVED |
//                                   USBHUB_FDO_FLAG_STATE_CHANGING |
//                                   USBHUB_FDO_FLAG_ESD_RECOVERING |
//                                   USBHUB_FDO_FLAG_DEVICE_FAILED |
//                                   USBHUB_FDO_FLAG_DEVICE_STOPPING))
//     {
//         DbgBreakPoint();
//         return;
//     }
//
//     HubExtension->HubFlags |= USBHUB_FDO_FLAG_GOING_IDLE;
//
//     if (!(HubExtension->HubFlags & USBHUB_FDO_FLAG_PENDING_WAKE_IRP))
//     {
//         Status = USBH_FdoSubmitWaitWakeIrp(HubExtension);
//
//         if (Status != STATUS_PENDING)
//         {
//             DPRINT("Status != STATUS_PENDING. DbgBreakPoint()\n");
//             DbgBreakPoint();
//             HubExtension->HubFlags &= ~USBHUB_FDO_FLAG_GOING_IDLE;
//             return;
//         }
//     }
//
//     InterlockedIncrement(&HubExtension->PendingRequestCount);
//
//     KeWaitForSingleObject(&HubExtension->ResetDeviceSemaphore,
//                           Executive,
//                           KernelMode,
//                           FALSE,
//                           NULL);
//
//     PortData = HubExtension->PortData;
//     IsReady = TRUE;
//
//     for (Port = 0;
//          Port < HubExtension->HubDescriptor->bNumberOfPorts;
//          Port++)
//     {
//         PortDevice = PortData[Port].DeviceObject;
//
//         if (PortDevice)
//         {
//             PortExtension = PortDevice->DeviceExtension;
//
//             IdleIrp = PortExtension->IdleNotificationIrp;
//
//             if (!IdleIrp)
//             {
//                 IsReady = FALSE;
//                 goto IdleHub;
//             }
//
//             IoStack = IoGetCurrentIrpStackLocation(IdleIrp);
//
//             CallbackInfo = IoStack->Parameters.DeviceIoControl.Type3InputBuffer;
//
//             if (!CallbackInfo)
//             {
//                 IsReady = FALSE;
//                 goto IdleHub;
//             }
//
//             if (!CallbackInfo->IdleCallback)
//             {
//                 IsReady = FALSE;
//                 goto IdleHub;
//             }
//
//             if (PortExtension->PendingSystemPoRequest)
//             {
//                 IsReady = FALSE;
//                 goto IdleHub;
//             }
//
//             if (InterlockedCompareExchange(&PortExtension->StateBehindD2,
//                                            1,
//                                            0))
//             {
//                 IsReady = FALSE;
//                 goto IdleHub;
//             }
//
//             DPRINT("USBH_FdoIdleNotificationCallback: IdleContext - %p\n",
//                    CallbackInfo->IdleContext);
//
//             CallbackInfo->IdleCallback(CallbackInfo->IdleContext);
//
//             if (PortExtension->CurrentPowerState.DeviceState == PowerDeviceD0)
//             {
//                 IsReady = FALSE;
//                 goto IdleHub;
//             }
//         }
//     }
//
//     if (!(HubExtension->HubFlags & USBHUB_FDO_FLAG_DEVICE_STOPPING) &&
//         (USBH_CheckIdleAbort(HubExtension, FALSE, FALSE) == TRUE))
//     {
//         IsReady = FALSE;
//     }
//
// IdleHub:
//
//     KeReleaseSemaphore(&HubExtension->ResetDeviceSemaphore,
//                        LOW_REALTIME_PRIORITY,
//                        1,
//                        FALSE);
//
//     if (!InterlockedDecrement(&HubExtension->PendingRequestCount))
//     {
//         KeSetEvent(&HubExtension->PendingRequestEvent,
//                    EVENT_INCREMENT,
//                    FALSE);
//     }
//
//     if (!IsReady ||
//         (HubExtension->HubFlags & USBHUB_FDO_FLAG_DEVICE_SUSPENDED))
//     {
//         DPRINT1("USBH_FdoIdleNotificationCallback: HubFlags - %lX\n",
//                 HubExtension->HubFlags);
//
//         HubExtension->HubFlags &= ~(USBHUB_FDO_FLAG_DEVICE_SUSPENDED |
//                                     USBHUB_FDO_FLAG_GOING_IDLE);
//
//         /* Aborting Idle for Hub */
//         IoAcquireCancelSpinLock(&OldIrql);
//
//         if (HubExtension->PendingIdleIrp)
//         {
//             Irp = HubExtension->PendingIdleIrp;
//             HubExtension->PendingIdleIrp = NULL;
//         }
//
//         IoReleaseCancelSpinLock(OldIrql);
//
//         if (Irp)
//         {
//             USBH_HubCancelIdleIrp(HubExtension, Irp);
//         }
//
//         DbgBreakPoint();
//         USBH_HubCompletePortIdleIrps(HubExtension, STATUS_CANCELLED);
//     }
//     else
//     {
//         PowerState.DeviceState = HubExtension->DeviceWake;
//
//         KeWaitForSingleObject(&HubExtension->IdleSemaphore,
//                               Executive,
//                               KernelMode,
//                               FALSE,
//                               NULL);
//
//         HubExtension->HubFlags &= ~USBHUB_FDO_FLAG_GOING_IDLE;
//         HubExtension->HubFlags |= USBHUB_FDO_FLAG_DO_SUSPENSE;
//
//         KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//         DPRINT("USBH_FdoIdleNotificationCallback: LowerPdo - %p\n",
//                 HubExtension->LowerPDO);
//
//         DPRINT("USBH_FdoIdleNotificationCallback: PowerState.DeviceState - %x\n",
//                PowerState.DeviceState);
//
//         Status = PoRequestPowerIrp(HubExtension->LowerPDO,
//                                    IRP_MN_SET_POWER,
//                                    PowerState,
//                                    USBH_HubSetDWakeCompletion,
//                                    &Event,
//                                    NULL);
//
//         if (Status == STATUS_PENDING)
//         {
//             KeWaitForSingleObject(&Event,
//                                   Executive,
//                                   KernelMode,
//                                   FALSE,
//                                   NULL);
//         }
//     }
// }
//
// VOID
// NTAPI
// USBH_CompletePortIdleIrpsWorker(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                                 IN PVOID Context)
// {
//     PUSBHUB_IDLE_PORT_CONTEXT IdlePortContext;
//     NTSTATUS NtStatus;
//     NTSTATUS Status;
//     BOOLEAN IsFlush = FALSE;
//
//     DPRINT("USBH_CompletePortIdleIrpsWorker ... \n");
//
//     IdlePortContext = Context;
//     NtStatus = IdlePortContext->Status;
//
//     USBH_HubCompleteQueuedPortIdleIrps(HubExtension,
//                                        &IdlePortContext->PwrList,
//                                        NtStatus);
//
//     DPRINT1("USBH_CompletePortIdleIrpsWorker: USBH_RegQueryFlushPortPowerIrpsFlag() UNIMPLEMENTED. FIXME\n");
//     Status = STATUS_NOT_IMPLEMENTED;// USBH_RegQueryFlushPortPowerIrpsFlag(&IsFlush);
//
//     if (NT_SUCCESS(Status))
//     {
//         if (IsFlush)
//         {
//             USBH_FlushPortPwrList(HubExtension);
//         }
//     }
// }
//
// VOID
// NTAPI
// USBH_IdleCompletePowerHubWorker(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                                 IN PVOID Context)
// {
//     PUSBHUB_IDLE_HUB_CONTEXT HubWorkItemBuffer;
//
//     DPRINT("USBH_IdleCompletePowerHubWorker ... \n");
//
//     if (HubExtension &&
//         HubExtension->CurrentPowerState.DeviceState != PowerDeviceD0 &&
//         HubExtension->HubFlags & USBHUB_FDO_FLAG_DEVICE_STARTED)
//     {
//         USBH_HubSetD0(HubExtension);
//     }
//
//     HubWorkItemBuffer = Context;
//
//     USBH_HubCompletePortIdleIrps(HubExtension, HubWorkItemBuffer->Status);
//
// }
//
// NTSTATUS
// NTAPI
// USBH_FdoIdleNotificationRequestComplete(IN PDEVICE_OBJECT DeviceObject,
//                                         IN PIRP Irp,
//                                         IN PVOID Context)
// {
//     PUSBHUB_FDO_EXTENSION HubExtension;
//     NTSTATUS NtStatus;
//     PVOID IdleIrp;
//     KIRQL Irql;
//     NTSTATUS Status;
//     PUSBHUB_IO_WORK_ITEM HubIoWorkItem;
//
//     IoAcquireCancelSpinLock(&Irql);
//
//     HubExtension = Context;
//     HubExtension->HubFlags &= ~USBHUB_FDO_FLAG_WAIT_IDLE_REQUEST;
//
//     IdleIrp = InterlockedExchangePointer((PVOID)&HubExtension->PendingIdleIrp,
//                                          NULL);
//
//     DPRINT("USBH_FdoIdleNotificationRequestComplete: IdleIrp - %p\n", IdleIrp);
//
//     if (!InterlockedDecrement(&HubExtension->PendingRequestCount))
//     {
//         KeSetEvent(&HubExtension->PendingRequestEvent, EVENT_INCREMENT, FALSE);
//     }
//
//     IoReleaseCancelSpinLock(Irql);
//
//     NtStatus = Irp->IoStatus.Status;
//
//     DPRINT("USBH_FdoIdleNotificationRequestComplete: NtStatus - %lX\n",
//            NtStatus);
//
//     if (!NT_SUCCESS(NtStatus) &&
//         NtStatus != STATUS_POWER_STATE_INVALID &&
//         !(HubExtension->HubFlags & USBHUB_FDO_FLAG_DEVICE_REMOVED) &&
//         !(HubExtension->HubFlags & USBHUB_FDO_FLAG_DEVICE_STOPPED))
//     {
//         DPRINT("USBH_FdoIdleNotificationRequestComplete: DeviceState - %x\n",
//                HubExtension->CurrentPowerState.DeviceState);
//
//         if (HubExtension->CurrentPowerState.DeviceState == PowerDeviceD0)
//         {
//             PUSBHUB_IDLE_PORT_CONTEXT HubWorkItemBuffer;
//
//             Status = USBH_AllocateWorkItem(HubExtension,
//                                            &HubIoWorkItem,
//                                            USBH_CompletePortIdleIrpsWorker,
//                                            sizeof(USBHUB_IDLE_PORT_CONTEXT),
//                                            (PVOID *)&HubWorkItemBuffer,
//                                            DelayedWorkQueue);
//
//             if (NT_SUCCESS(Status))
//             {
//                 HubWorkItemBuffer->Status = NtStatus;
//
//                 USBH_HubQueuePortIdleIrps(HubExtension,
//                                           &HubWorkItemBuffer->PwrList);
//
//                 USBH_QueueWorkItem(HubExtension, HubIoWorkItem);
//             }
//         }
//         else
//         {
//             PUSBHUB_IDLE_HUB_CONTEXT HubWorkItemBuffer;
//
//             Status = USBH_AllocateWorkItem(HubExtension,
//                                            &HubIoWorkItem,
//                                            USBH_IdleCompletePowerHubWorker,
//                                            sizeof(USBHUB_IDLE_HUB_CONTEXT),
//                                            (PVOID *)&HubWorkItemBuffer,
//                                            DelayedWorkQueue);
//
//             if (NT_SUCCESS(Status))
//             {
//                 HubWorkItemBuffer->Status = NtStatus;
//                 USBH_QueueWorkItem(HubExtension, HubIoWorkItem);
//             }
//         }
//     }
//
//     if (IdleIrp ||
//         InterlockedExchange((PLONG)&HubExtension->IdleRequestLock, 1))
//     {
//         DPRINT("USBH_FdoIdleNotificationRequestComplete: Irp - %p\n", Irp);
//         IoFreeIrp(Irp);
//     }
//
//     return STATUS_MORE_PROCESSING_REQUIRED;
// }
//
// NTSTATUS
// NTAPI
// USBH_FdoSubmitIdleRequestIrp(IN PUSBHUB_FDO_EXTENSION HubExtension)
// {
//     NTSTATUS Status;
//     ULONG HubFlags;
//     PDEVICE_OBJECT LowerPDO;
//     PIRP Irp;
//     PIO_STACK_LOCATION IoStack;
//     KIRQL Irql;
//
//     DPRINT("USBH_FdoSubmitIdleRequestIrp: HubExtension - %p, PendingIdleIrp - %p\n",
//            HubExtension,
//            HubExtension->PendingIdleIrp);
//
//     if (HubExtension->PendingIdleIrp)
//     {
//         Status = STATUS_DEVICE_BUSY;
//         KeSetEvent(&HubExtension->IdleEvent, EVENT_INCREMENT, FALSE);
//         return Status;
//     }
//
//     HubFlags = HubExtension->HubFlags;
//
//     if (HubFlags & USBHUB_FDO_FLAG_DEVICE_STOPPING ||
//         HubFlags & USBHUB_FDO_FLAG_DEVICE_REMOVED)
//     {
//         HubExtension->HubFlags = HubFlags & ~USBHUB_FDO_FLAG_WAIT_IDLE_REQUEST;
//         KeSetEvent(&HubExtension->IdleEvent, EVENT_INCREMENT, FALSE);
//         return STATUS_DEVICE_REMOVED;
//     }
//
//     LowerPDO = HubExtension->LowerPDO;
//
//     HubExtension->IdleCallbackInfo.IdleCallback = USBH_FdoIdleNotificationCallback;
//     HubExtension->IdleCallbackInfo.IdleContext = HubExtension;
//
//     Irp = IoAllocateIrp(LowerPDO->StackSize, FALSE);
//
//     if (!Irp)
//     {
//         HubExtension->HubFlags &= ~USBHUB_FDO_FLAG_WAIT_IDLE_REQUEST;
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//
//         KeSetEvent(&HubExtension->IdleEvent, EVENT_INCREMENT, FALSE);
//         return Status;
//     }
//
//     IoStack = IoGetNextIrpStackLocation(Irp);
//
//     IoStack->MajorFunction = IRP_MJ_INTERNAL_DEVICE_CONTROL;
//
//     IoStack->Parameters.DeviceIoControl.InputBufferLength = sizeof(USB_IDLE_CALLBACK_INFO);
//     IoStack->Parameters.DeviceIoControl.IoControlCode = IOCTL_INTERNAL_USB_SUBMIT_IDLE_NOTIFICATION;
//     IoStack->Parameters.DeviceIoControl.Type3InputBuffer = &HubExtension->IdleCallbackInfo;
//
//     IoSetCompletionRoutine(Irp,
//                            USBH_FdoIdleNotificationRequestComplete,
//                            HubExtension,
//                            TRUE,
//                            TRUE,
//                            TRUE);
//
//     InterlockedIncrement(&HubExtension->PendingRequestCount);
//     InterlockedExchange(&HubExtension->IdleRequestLock, 0);
//
//     HubExtension->HubFlags &= ~(USBHUB_FDO_FLAG_DEVICE_SUSPENDED |
//                                 USBHUB_FDO_FLAG_GOING_IDLE);
//
//     Status = IoCallDriver(HubExtension->LowerPDO, Irp);
//
//     IoAcquireCancelSpinLock(&Irql);
//
//     if (Status == STATUS_PENDING &&
//         HubExtension->HubFlags & USBHUB_FDO_FLAG_WAIT_IDLE_REQUEST)
//     {
//         HubExtension->PendingIdleIrp = Irp;
//     }
//
//     IoReleaseCancelSpinLock(Irql);
//
//     KeSetEvent(&HubExtension->IdleEvent, EVENT_INCREMENT, FALSE);
//
//     return Status;
// }
//
// VOID
// NTAPI
// USBH_CheckHubIdle(IN PUSBHUB_FDO_EXTENSION HubExtension)
// {
//     PDEVICE_OBJECT PdoDevice;
//     PUSBHUB_PORT_PDO_EXTENSION PortExtension;
//     PUSBHUB_PORT_DATA PortData;
//     ULONG HubFlags;
//     ULONG Port;
//     KIRQL Irql;
//     BOOLEAN IsHubIdle = FALSE;
//     BOOLEAN IsAllPortsIdle;
//     BOOLEAN IsHubCheck = TRUE;
//
//     DPRINT("USBH_CheckHubIdle: FIXME !!! HubExtension - %p\n", HubExtension);
//
// return; //HACK: delete it line after fixing Power Manager!!!
//
//     KeAcquireSpinLock(&HubExtension->CheckIdleSpinLock, &Irql);
//
//     if (HubExtension->HubFlags & USBHUB_FDO_FLAG_CHECK_IDLE_LOCK)
//     {
//         KeReleaseSpinLock(&HubExtension->CheckIdleSpinLock, Irql);
//         return;
//     }
//
//     HubExtension->HubFlags |= USBHUB_FDO_FLAG_CHECK_IDLE_LOCK;
//     KeReleaseSpinLock(&HubExtension->CheckIdleSpinLock, Irql);
//
//     if (USBH_GetRootHubExtension(HubExtension)->SystemPowerState.SystemState != PowerSystemWorking)
//     {
//         KeAcquireSpinLock(&HubExtension->CheckIdleSpinLock, &Irql);
//         HubExtension->HubFlags &= ~USBHUB_FDO_FLAG_CHECK_IDLE_LOCK;
//         KeReleaseSpinLock(&HubExtension->CheckIdleSpinLock, Irql);
//         return;
//     }
//
//     HubFlags = HubExtension->HubFlags;
//     DPRINT("USBH_CheckHubIdle: HubFlags - %lX\n", HubFlags);
//
//     if (!(HubFlags & USBHUB_FDO_FLAG_DEVICE_STARTED) ||
//         !(HubFlags & USBHUB_FDO_FLAG_DO_ENUMERATION))
//     {
//         goto Exit;
//     }
//
//     if (HubFlags & USBHUB_FDO_FLAG_NOT_ENUMERATED ||
//         HubFlags & USBHUB_FDO_FLAG_ENUM_POST_RECOVER ||
//         HubFlags & USBHUB_FDO_FLAG_DEVICE_FAILED ||
//         HubFlags & USBHUB_FDO_FLAG_DEVICE_STOPPING ||
//         HubFlags & USBHUB_FDO_FLAG_DEVICE_REMOVED ||
//         HubFlags & USBHUB_FDO_FLAG_STATE_CHANGING ||
//         HubFlags & USBHUB_FDO_FLAG_WAKEUP_START ||
//         HubFlags & USBHUB_FDO_FLAG_ESD_RECOVERING)
//     {
//         goto Exit;
//     }
//
//     if (HubExtension->ResetRequestCount)
//     {
//         HubExtension->HubFlags |= USBHUB_FDO_FLAG_DEFER_CHECK_IDLE;
//         goto Exit;
//     }
//
//     HubExtension->HubFlags &= ~USBHUB_FDO_FLAG_DEFER_CHECK_IDLE;
//
//     InterlockedIncrement(&HubExtension->PendingRequestCount);
//
//     KeWaitForSingleObject(&HubExtension->ResetDeviceSemaphore,
//                           Executive,
//                           KernelMode,
//                           FALSE,
//                           NULL);
//
//     IoAcquireCancelSpinLock(&Irql);
//
//     IsAllPortsIdle = TRUE;
//
//     PortData = HubExtension->PortData;
//
//     for (Port = 0;
//          Port < HubExtension->HubDescriptor->bNumberOfPorts;
//          Port++)
//     {
//         PdoDevice = PortData[Port].DeviceObject;
//
//         if (PdoDevice)
//         {
//             PortExtension = PdoDevice->DeviceExtension;
//
//             if (!PortExtension->IdleNotificationIrp)
//             {
//                 DPRINT("USBH_CheckHubIdle: PortExtension - %p\n",
//                        PortExtension);
//
//                 IsAllPortsIdle = FALSE;
//                 IsHubCheck = FALSE;
//
//                 break;
//             }
//         }
//     }
//
//     if (IsHubCheck &&
//         !(HubExtension->HubFlags & USBHUB_FDO_FLAG_WAIT_IDLE_REQUEST))
//     {
//         KeClearEvent(&HubExtension->IdleEvent);
//         HubExtension->HubFlags |= USBHUB_FDO_FLAG_WAIT_IDLE_REQUEST;
//         IsHubIdle = TRUE;
//     }
//
//     IoReleaseCancelSpinLock(Irql);
//
//     KeReleaseSemaphore(&HubExtension->ResetDeviceSemaphore,
//                        LOW_REALTIME_PRIORITY,
//                        1,
//                        FALSE);
//
//     if (!InterlockedDecrement(&HubExtension->PendingRequestCount))
//     {
//         KeSetEvent(&HubExtension->PendingRequestEvent,
//                    EVENT_INCREMENT,
//                    FALSE);
//     }
//
//     DPRINT("USBH_CheckHubIdle: IsAllPortsIdle - %x, IsHubIdle - %x\n",
//            IsAllPortsIdle,
//            IsHubIdle);
//
//     if (IsAllPortsIdle && IsHubIdle)
//     {
//         USBH_FdoSubmitIdleRequestIrp(HubExtension);
//     }
//
// Exit:
//     KeAcquireSpinLock(&HubExtension->CheckIdleSpinLock, &Irql);
//     HubExtension->HubFlags &= ~USBHUB_FDO_FLAG_CHECK_IDLE_LOCK;
//     KeReleaseSpinLock(&HubExtension->CheckIdleSpinLock, Irql);
// }
//
// VOID
// NTAPI
// USBH_CheckIdleWorker(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                      IN PVOID Context)
// {
//     DPRINT("USBH_CheckIdleWorker: ... \n");
//     USBH_CheckHubIdle(HubExtension);
// }
//
// VOID
// NTAPI
// USBH_CheckIdleDeferred(IN PUSBHUB_FDO_EXTENSION HubExtension)
// {
//     PUSBHUB_IO_WORK_ITEM HubIoWorkItem;
//     NTSTATUS Status;
//
//     DPRINT("USBH_CheckIdleDeferred: HubExtension - %p\n", HubExtension);
//
//     Status = USBH_AllocateWorkItem(HubExtension,
//                                    &HubIoWorkItem,
//                                    USBH_CheckIdleWorker,
//                                    0,
//                                    NULL,
//                                    DelayedWorkQueue);
//
//     DPRINT("USBH_CheckIdleDeferred: HubIoWorkItem - %p\n", HubIoWorkItem);
//
//     if (NT_SUCCESS(Status))
//     {
//         USBH_QueueWorkItem(HubExtension, HubIoWorkItem);
//     }
// }
//
// VOID
// NTAPI
// USBH_PdoSetCapabilities(IN PUSBHUB_PORT_PDO_EXTENSION PortExtension)
// {
//     PUSBHUB_FDO_EXTENSION HubExtension;
//     ULONG State;
//     SYSTEM_POWER_STATE SystemPowerState;
//     PDEVICE_POWER_STATE pDeviceState;
//
//     DPRINT("USBH_PdoSetCapabilities ... \n");
//
//     HubExtension = PortExtension->HubExtension;
//
//     PortExtension->Capabilities.Size = 64;
//     PortExtension->Capabilities.Version = 1;
//
//     PortExtension->Capabilities.Removable = 1;
//     PortExtension->Capabilities.Address = PortExtension->PortNumber;
//
//     if (PortExtension->SerialNumber)
//     {
//         PortExtension->Capabilities.UniqueID = 1;
//     }
//     else
//     {
//         PortExtension->Capabilities.UniqueID = 0;
//     }
//
//     PortExtension->Capabilities.RawDeviceOK = 0;
//
//     RtlCopyMemory(PortExtension->Capabilities.DeviceState,
//                   HubExtension->DeviceState,
//                   (PowerSystemMaximum + 2) * sizeof(POWER_STATE));
//
//     PortExtension->Capabilities.DeviceState[1] = PowerDeviceD0;
//
//     if (PortExtension->PortPdoFlags & USBHUB_PDO_FLAG_REMOTE_WAKEUP)
//     {
//         PortExtension->Capabilities.DeviceWake = PowerDeviceD2;
//
//         PortExtension->Capabilities.DeviceD1 = 1;
//         PortExtension->Capabilities.DeviceD2 = 1;
//
//         PortExtension->Capabilities.WakeFromD0 = 1;
//         PortExtension->Capabilities.WakeFromD1 = 1;
//         PortExtension->Capabilities.WakeFromD2 = 1;
//
//         pDeviceState = &PortExtension->Capabilities.DeviceState[2];
//
//         for (State = 2; State <= 5; State++)
//         {
//             SystemPowerState = State;
//
//             if (PortExtension->Capabilities.SystemWake < SystemPowerState)
//             {
//                 *pDeviceState = PowerDeviceD3;
//             }
//             else
//             {
//                 *pDeviceState = PowerDeviceD2;
//             }
//
//             ++pDeviceState;
//         }
//     }
//     else
//     {
//         PortExtension->Capabilities.DeviceWake = PowerDeviceD0;
//         PortExtension->Capabilities.DeviceState[2] = PowerDeviceD3;
//         PortExtension->Capabilities.DeviceState[3] = PowerDeviceD3;
//         PortExtension->Capabilities.DeviceState[4] = PowerDeviceD3;
//         PortExtension->Capabilities.DeviceState[5] = PowerDeviceD3;
//     }
// }
//
// NTSTATUS
// NTAPI
// USBH_ProcessDeviceInformation(IN PUSBHUB_PORT_PDO_EXTENSION PortExtension)
// {
//     PUSB_INTERFACE_DESCRIPTOR Pid;
//     PUSB_CONFIGURATION_DESCRIPTOR ConfigDescriptor;
//     NTSTATUS Status;
//
//     DPRINT("USBH_ProcessDeviceInformation ... \n");
//
//     ConfigDescriptor = NULL;
//
//     RtlZeroMemory(&PortExtension->InterfaceDescriptor,
//                   sizeof(PortExtension->InterfaceDescriptor));
//
//     PortExtension->PortPdoFlags &= ~USBHUB_PDO_FLAG_HUB_DEVICE;
//
//     Status = USBH_GetConfigurationDescriptor(PortExtension->Common.SelfDevice,
//                                              &ConfigDescriptor);
//
//     if (!NT_SUCCESS(Status))
//     {
//         if (ConfigDescriptor)
//         {
//             ExFreePool(ConfigDescriptor);
//         }
//
//         return Status;
//     }
//
//     PortExtension->PortPdoFlags &= ~USBHUB_PDO_FLAG_REMOTE_WAKEUP;
//
//     if (ConfigDescriptor->bmAttributes & 0x20)
//     {
//         /* device configuration supports remote wakeup */
//         PortExtension->PortPdoFlags |= USBHUB_PDO_FLAG_REMOTE_WAKEUP;
//     }
//
//     USBHUB_DumpingDeviceDescriptor(&PortExtension->DeviceDescriptor);
//     USBHUB_DumpingConfiguration(ConfigDescriptor);
//
//     DPRINT_PNP!("USBH_ProcessDeviceInformation: Class - %x, SubClass - %x, Protocol - %x\n",
//                PortExtension->DeviceDescriptor.bDeviceClass,
//                PortExtension->DeviceDescriptor.bDeviceSubClass,
//                PortExtension->DeviceDescriptor.bDeviceProtocol);
//
//     DPRINT_PNP!("USBH_ProcessDeviceInformation: bNumConfigurations - %x, bNumInterfaces - %x\n",
//                PortExtension->DeviceDescriptor.bNumConfigurations,
//                ConfigDescriptor->bNumInterfaces);
//
//
//     /* Enumeration of USB Composite Devices (msdn):
//        1) The device class field of the device descriptor (bDeviceClass) must contain a value of zero,
//        or the class (bDeviceClass), subclass (bDeviceSubClass), and protocol (bDeviceProtocol)
//        fields of the device descriptor must have the values 0xEF, 0x02 and 0x01 respectively,
//        as explained in USB Interface Association Descriptor.
//        2) The device must have multiple interfaces
//        3) The device must have a single configuration.
//     */
//
//     if (((PortExtension->DeviceDescriptor.bDeviceClass == (USB_DEVICE_CLASS_RESERVED as u8)) ||
//         (PortExtension->DeviceDescriptor.bDeviceClass == (USB_DEVICE_CLASS_MISCELLANEOUS as u8) &&
//          PortExtension->DeviceDescriptor.bDeviceSubClass == 0x02 &&
//          PortExtension->DeviceDescriptor.bDeviceProtocol == 0x01)) &&
//          (ConfigDescriptor->bNumInterfaces > 1) &&
//          (PortExtension->DeviceDescriptor.bNumConfigurations < 2))
//     {
//         DPRINT("USBH_ProcessDeviceInformation: Multi-Interface configuration\n");
//
//         PortExtension->PortPdoFlags |= USBHUB_PDO_FLAG_MULTI_INTERFACE;
//
//         if (ConfigDescriptor)
//         {
//             ExFreePool(ConfigDescriptor);
//         }
//
//         return Status;
//     }
//
//     Pid = USBD_ParseConfigurationDescriptorEx(ConfigDescriptor,
//                                               ConfigDescriptor,
//                                               -1,
//                                               -1,
//                                               -1,
//                                               -1,
//                                               -1);
//     if (Pid)
//     {
//         RtlCopyMemory(&PortExtension->InterfaceDescriptor,
//                       Pid,
//                       sizeof(PortExtension->InterfaceDescriptor));
//
//         if (Pid->bInterfaceClass == USB_DEVICE_CLASS_HUB)
//         {
//             PortExtension->PortPdoFlags |= (USBHUB_PDO_FLAG_HUB_DEVICE |
//                                             USBHUB_PDO_FLAG_REMOTE_WAKEUP);
//         }
//     }
//     else
//     {
//         Status = STATUS_UNSUCCESSFUL;
//     }
//
//     if (ConfigDescriptor)
//     {
//         ExFreePool(ConfigDescriptor);
//     }
//
//     return Status;
// }
//
// BOOLEAN
// NTAPI
// USBH_CheckDeviceIDUnique(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                          IN USHORT idVendor,
//                          IN USHORT idProduct,
//                          IN PVOID SerialNumber,
//                          IN USHORT SN_DescriptorLength)
// {
//     PDEVICE_OBJECT PortDevice;
//     PUSBHUB_PORT_PDO_EXTENSION PortExtension;
//     ULONG Port;
//     SIZE_T NumberBytes;
//
//     DPRINT("USBH_CheckDeviceIDUnique: idVendor - 0x%04X, idProduct - 0x%04X\n",
//            idVendor,
//            idProduct);
//
//     if (!HubExtension->HubDescriptor->bNumberOfPorts)
//     {
//         return TRUE;
//     }
//
//     for (Port = 0; Port < HubExtension->HubDescriptor->bNumberOfPorts; Port++)
//     {
//         PortDevice = HubExtension->PortData[Port].DeviceObject;
//
//         if (PortDevice)
//         {
//             PortExtension = PortDevice->DeviceExtension;
//
//             if (PortExtension->DeviceDescriptor.idVendor == idVendor &&
//                 PortExtension->DeviceDescriptor.idProduct == idProduct &&
//                 PortExtension->SN_DescriptorLength == SN_DescriptorLength)
//             {
//                 if (PortExtension->SerialNumber)
//                 {
//                     NumberBytes = RtlCompareMemory(PortExtension->SerialNumber,
//                                                    SerialNumber,
//                                                    SN_DescriptorLength);
//
//                     if (NumberBytes == SN_DescriptorLength)
//                     {
//                         return FALSE;
//                     }
//                 }
//             }
//         }
//     }
//
//     return TRUE;
// }
//
// BOOLEAN
// NTAPI
// USBH_ValidateSerialNumberString(IN PUSHORT SerialNumberString)
// {
//     USHORT ix;
//     USHORT Symbol;
//
//     DPRINT("USBH_ValidateSerialNumberString: ... \n");
//
//     for (ix = 0; SerialNumberString[ix] != UNICODE_NULL; ix++)
//     {
//         Symbol = SerialNumberString[ix];
//
//         if (Symbol < 0x20 || Symbol > 0x7F || Symbol == 0x2C) // ','
//         {
//             return FALSE;
//         }
//     }
//
//     return TRUE;
// }
//
//
// NTSTATUS
// NTAPI
// USBH_CheckDeviceLanguage(IN PDEVICE_OBJECT DeviceObject,
//                          IN USHORT LanguageId)
// {
//     PUSB_STRING_DESCRIPTOR Descriptor;
//     NTSTATUS Status;
//     ULONG NumSymbols;
//     ULONG ix;
//     PWCHAR pSymbol;
//     ULONG Length;
//
//     DPRINT("USBH_CheckDeviceLanguage: LanguageId - 0x%04X\n", LanguageId);
//
//     Descriptor = ExAllocatePoolWithTag(NonPagedPool,
//                                        MAXIMUM_USB_STRING_LENGTH,
//                                        USB_HUB_TAG);
//
//     if (!Descriptor)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlZeroMemory(Descriptor, MAXIMUM_USB_STRING_LENGTH);
//
//     Status = USBH_SyncGetStringDescriptor(DeviceObject,
//                                           0,
//                                           0,
//                                           Descriptor,
//                                           MAXIMUM_USB_STRING_LENGTH,
//                                           &Length,
//                                           TRUE);
//
//     if (!NT_SUCCESS(Status) ||
//         Length < sizeof(USB_COMMON_DESCRIPTOR))
//     {
//         goto Exit;
//     }
//
//     NumSymbols = (Length -
//                   FIELD_OFFSET(USB_STRING_DESCRIPTOR, bString)) / sizeof(WCHAR);
//
//     pSymbol = Descriptor->bString;
//
//     for (ix = 1; ix < NumSymbols; ix++)
//     {
//         if (*pSymbol == (WCHAR)LanguageId)
//         {
//             Status = STATUS_SUCCESS;
//             goto Exit;
//         }
//
//         pSymbol++;
//     }
//
//     Status = STATUS_NOT_SUPPORTED;
//
// Exit:
//     ExFreePoolWithTag(Descriptor, USB_HUB_TAG);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_GetSerialNumberString(IN PDEVICE_OBJECT DeviceObject,
//                            IN LPWSTR * OutSerialNumber,
//                            IN PUSHORT OutDescriptorLength,
//                            IN USHORT LanguageId,
//                            IN UCHAR Index)
// {
//     PUSB_STRING_DESCRIPTOR Descriptor;
//     NTSTATUS Status;
//     LPWSTR SerialNumberBuffer = NULL;
//     UCHAR StringLength;
//     UCHAR Length;
//
//     DPRINT("USBH_GetSerialNumberString: ... \n");
//
//     *OutSerialNumber = NULL;
//     *OutDescriptorLength = 0;
//
//     Descriptor = ExAllocatePoolWithTag(NonPagedPool,
//                                        MAXIMUM_USB_STRING_LENGTH,
//                                        USB_HUB_TAG);
//
//     if (!Descriptor)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlZeroMemory(Descriptor, MAXIMUM_USB_STRING_LENGTH);
//
//     Status = USBH_CheckDeviceLanguage(DeviceObject, LanguageId);
//
//     if (!NT_SUCCESS(Status))
//     {
//         goto Exit;
//     }
//
//     Status = USBH_SyncGetStringDescriptor(DeviceObject,
//                                           Index,
//                                           LanguageId,
//                                           Descriptor,
//                                           MAXIMUM_USB_STRING_LENGTH,
//                                           NULL,
//                                           TRUE);
//
//     if (!NT_SUCCESS(Status) ||
//         Descriptor->bLength <= sizeof(USB_COMMON_DESCRIPTOR))
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto Exit;
//     }
//
//     StringLength = Descriptor->bLength -
//                    FIELD_OFFSET(USB_STRING_DESCRIPTOR, bString);
//
//     Length = StringLength + sizeof(UNICODE_NULL);
//
//     SerialNumberBuffer = ExAllocatePoolWithTag(PagedPool, Length, USB_HUB_TAG);
//
//     if (!SerialNumberBuffer)
//     {
//         goto Exit;
//     }
//
//     RtlZeroMemory(SerialNumberBuffer, Length);
//     RtlCopyMemory(SerialNumberBuffer, Descriptor->bString, StringLength);
//
//     *OutSerialNumber = SerialNumberBuffer;
//     *OutDescriptorLength = Length;
//
// Exit:
//     ExFreePoolWithTag(Descriptor, USB_HUB_TAG);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_CreateDevice(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                   IN USHORT Port,
//                   IN USB_PORT_STATUS UsbPortStatus,
//                   IN ULONG IsWait)
// {
//     ULONG PdoNumber = 0;
//     WCHAR CharDeviceName[64];
//     UNICODE_STRING DeviceName;
//     PDEVICE_OBJECT DeviceObject = NULL;
//     PUSBHUB_PORT_PDO_EXTENSION PortExtension;
//     PUSB_DEVICE_HANDLE DeviceHandle;
//     LPWSTR SerialNumberBuffer;
//     BOOLEAN IsHsDevice;
//     BOOLEAN IsLsDevice;
//     BOOLEAN IgnoringHwSerial = FALSE;
//     NTSTATUS Status;
//     UNICODE_STRING DestinationString;
//
//     DPRINT("USBH_CreateDevice: Port - %x, UsbPortStatus - %lX\n",
//            Port,
//            UsbPortStatus.AsUshort16);
//
//     do
//     {
//         RtlStringCbPrintfW(CharDeviceName,
//                            sizeof(CharDeviceName),
//                            L"\\Device\\USBPDO-%d",
//                            PdoNumber);
//
//         RtlInitUnicodeString(&DeviceName, CharDeviceName);
//
//         Status = IoCreateDevice(HubExtension->Common.SelfDevice->DriverObject,
//                                 sizeof(USBHUB_PORT_PDO_EXTENSION),
//                                 &DeviceName,
//                                 FILE_DEVICE_USB,
//                                 0,
//                                 FALSE,
//                                 &DeviceObject);
//
//         ++PdoNumber;
//     }
//     while (Status == STATUS_OBJECT_NAME_COLLISION);
//
//     if (!NT_SUCCESS(Status))
//     {
//         ASSERT(Port > 0);
//         HubExtension->PortData[Port-1].DeviceObject = DeviceObject;
//         return Status;
//     }
//
//     DeviceObject->StackSize = HubExtension->RootHubPdo2->StackSize;
//
//     PortExtension = DeviceObject->DeviceExtension;
//
//     DPRINT("USBH_CreateDevice: PortDevice - %p, <%wZ>\n", DeviceObject, &DeviceName);
//     DPRINT("USBH_CreateDevice: PortExtension - %p\n", PortExtension);
//
//     RtlZeroMemory(PortExtension, sizeof(USBHUB_PORT_PDO_EXTENSION));
//
//     PortExtension->Common.ExtensionType = USBH_EXTENSION_TYPE_PORT;
//     PortExtension->Common.SelfDevice = DeviceObject;
//
//     PortExtension->HubExtension = HubExtension;
//     PortExtension->RootHubExtension = HubExtension;
//
//     PortExtension->PortNumber = Port;
//     PortExtension->CurrentPowerState.DeviceState = PowerDeviceD0;
//     PortExtension->IgnoringHwSerial = FALSE;
//
//     KeInitializeSpinLock(&PortExtension->PortTimeoutSpinLock);
//
//     InitializeListHead(&PortExtension->PortPowerList);
//     KeInitializeSpinLock(&PortExtension->PortPowerListSpinLock);
//
//     PortExtension->PoRequestCounter = 0;
//     PortExtension->PendingSystemPoRequest = 0;
//     PortExtension->PendingDevicePoRequest = 0;
//     PortExtension->StateBehindD2 = 0;
//
//     SerialNumberBuffer = NULL;
//
//     IsHsDevice = UsbPortStatus.Usb20PortStatus.HighSpeedDeviceAttached;
//     IsLsDevice = UsbPortStatus.Usb20PortStatus.LowSpeedDeviceAttached;
//
//     if (IsLsDevice == 0)
//     {
//         if (IsHsDevice)
//         {
//             PortExtension->PortPdoFlags = USBHUB_PDO_FLAG_PORT_HIGH_SPEED;
//         }
//     }
//     else
//     {
//         PortExtension->PortPdoFlags = USBHUB_PDO_FLAG_PORT_LOW_SPEED;
//     }
//
//     /* Initialize PortExtension->InstanceID */
//     RtlInitUnicodeString(&DestinationString, (PCWSTR)&PortExtension->InstanceID);
//     DestinationString.MaximumLength = 4 * sizeof(WCHAR);
//     Status = RtlIntegerToUnicodeString(Port, 10, &DestinationString);
//
//     DeviceObject->Flags |= DO_POWER_PAGABLE;
//     DeviceObject->Flags &= ~DO_DEVICE_INITIALIZING;
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("USBH_CreateDevice: IoCreateDevice() failed - %lX\n", Status);
//         goto ErrorExit;
//     }
//
//     Status = USBD_CreateDeviceEx(HubExtension,
//                                  &PortExtension->DeviceHandle,
//                                  UsbPortStatus,
//                                  Port);
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("USBH_CreateDevice: USBD_CreateDeviceEx() failed - %lX\n", Status);
//         goto ErrorExit;
//     }
//
//     Status = USBH_SyncResetPort(HubExtension, Port);
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("USBH_CreateDevice: USBH_SyncResetPort() failed - %lX\n", Status);
//         goto ErrorExit;
//     }
//
//     if (IsWait)
//     {
//         USBH_Wait(50);
//     }
//
//     Status = USBD_InitializeDeviceEx(HubExtension,
//                                      PortExtension->DeviceHandle,
//                                      (PUCHAR)&PortExtension->DeviceDescriptor,
//                                      sizeof(USB_DEVICE_DESCRIPTOR),
//                                      (PUCHAR)&PortExtension->ConfigDescriptor,
//                                      sizeof(USB_CONFIGURATION_DESCRIPTOR));
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("USBH_CreateDevice: USBD_InitializeDeviceEx() failed - %lX\n", Status);
//         PortExtension->DeviceHandle = NULL;
//         goto ErrorExit;
//     }
//
//     DPRINT1("USBH_RegQueryDeviceIgnoreHWSerNumFlag UNIMPLEMENTED. FIXME\n");
//     //Status = USBH_RegQueryDeviceIgnoreHWSerNumFlag(PortExtension->DeviceDescriptor.idVendor,
//     //                                               PortExtension->DeviceDescriptor.idProduct,
//     //                                               &IgnoringHwSerial);
//
//     if (TRUE)//Status == STATUS_OBJECT_NAME_NOT_FOUND)
//     {
//         IgnoringHwSerial = FALSE;
//     }
//
//     if (IgnoringHwSerial)
//     {
//         PortExtension->IgnoringHwSerial = TRUE;
//     }
//
//     if (PortExtension->DeviceDescriptor.iSerialNumber &&
//        !PortExtension->IgnoringHwSerial)
//     {
//         InterlockedExchangePointer((PVOID)&PortExtension->SerialNumber, NULL);
//
//         USBH_GetSerialNumberString(PortExtension->Common.SelfDevice,
//                                    &SerialNumberBuffer,
//                                    &PortExtension->SN_DescriptorLength,
//                                    MAKELANGID(LANG_ENGLISH, SUBLANG_ENGLISH_US),
//                                    PortExtension->DeviceDescriptor.iSerialNumber);
//
//         if (SerialNumberBuffer)
//         {
//             if (!USBH_ValidateSerialNumberString((PUSHORT)SerialNumberBuffer))
//             {
//                 ExFreePoolWithTag(SerialNumberBuffer, USB_HUB_TAG);
//                 SerialNumberBuffer = NULL;
//             }
//
//             if (SerialNumberBuffer &&
//                 !USBH_CheckDeviceIDUnique(HubExtension,
//                                           PortExtension->DeviceDescriptor.idVendor,
//                                           PortExtension->DeviceDescriptor.idProduct,
//                                           SerialNumberBuffer,
//                                           PortExtension->SN_DescriptorLength))
//             {
//                 ExFreePoolWithTag(SerialNumberBuffer, USB_HUB_TAG);
//                 SerialNumberBuffer = NULL;
//             }
//         }
//
//         InterlockedExchangePointer((PVOID)&PortExtension->SerialNumber,
//                                    SerialNumberBuffer);
//     }
//
//     Status = USBH_ProcessDeviceInformation(PortExtension);
//
//     USBH_PdoSetCapabilities(PortExtension);
//
//     if (NT_SUCCESS(Status))
//     {
//         goto Exit;
//     }
//
// ErrorExit:
//
//     PortExtension->PortPdoFlags |= USBHUB_PDO_FLAG_INIT_PORT_FAILED;
//
//     DeviceHandle = InterlockedExchangePointer(&PortExtension->DeviceHandle,
//                                               NULL);
//
//     if (DeviceHandle)
//     {
//         USBD_RemoveDeviceEx(HubExtension, DeviceHandle, 0);
//     }
//
//     SerialNumberBuffer = InterlockedExchangePointer((PVOID)&PortExtension->SerialNumber,
//                                                      NULL);
//
//     if (SerialNumberBuffer)
//     {
//         ExFreePoolWithTag(SerialNumberBuffer, USB_HUB_TAG);
//     }
//
// Exit:
//
//     ASSERT(Port > 0);
//     HubExtension->PortData[Port-1].DeviceObject = DeviceObject;
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_ResetDevice(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                  IN USHORT Port,
//                  IN BOOLEAN IsKeepDeviceData,
//                  IN BOOLEAN IsWait)
// {
//     NTSTATUS Status;
//     PUSBHUB_PORT_DATA PortData;
//     PDEVICE_OBJECT PortDevice;
//     PUSBHUB_PORT_PDO_EXTENSION PortExtension;
//     PVOID NewDeviceHandle;
//     PVOID Handle;
//     PVOID OldDeviceHandle;
//     PUSB_DEVICE_HANDLE * DeviceHandle;
//     USB_PORT_STATUS_AND_CHANGE PortStatus;
//
//     DPRINT("USBH_ResetDevice: HubExtension - %p, Port - %x, IsKeepDeviceData - %x, IsWait - %x\n",
//            HubExtension,
//            Port,
//            IsKeepDeviceData,
//            IsWait);
//
//     Status = USBH_SyncGetPortStatus(HubExtension,
//                                     Port,
//                                     &PortStatus,
//                                     sizeof(USB_PORT_STATUS_AND_CHANGE));
//
//     if (!NT_SUCCESS(Status) ||
//         !(PortStatus.PortStatus.Usb20PortStatus.CurrentConnectStatus))
//     {
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     InterlockedIncrement(&HubExtension->PendingRequestCount);
//
//     KeWaitForSingleObject(&HubExtension->ResetDeviceSemaphore,
//                           Executive,
//                           KernelMode,
//                           FALSE,
//                           NULL);
//
//     ASSERT(Port > 0);
//     PortData = &HubExtension->PortData[Port-1];
//
//     PortDevice = PortData->DeviceObject;
//
//     if (!PortDevice)
//     {
//         Status = STATUS_INVALID_PARAMETER;
//
//         KeReleaseSemaphore(&HubExtension->ResetDeviceSemaphore,
//                            LOW_REALTIME_PRIORITY,
//                            1,
//                            FALSE);
//
//         if (!InterlockedDecrement(&HubExtension->PendingRequestCount))
//         {
//             KeSetEvent(&HubExtension->PendingRequestEvent,
//                        EVENT_INCREMENT,
//                        FALSE);
//         }
//
//         return Status;
//     }
//
//     PortExtension = PortDevice->DeviceExtension;
//     DeviceHandle = &PortExtension->DeviceHandle;
//
//     OldDeviceHandle = InterlockedExchangePointer(&PortExtension->DeviceHandle,
//                                                  NULL);
//
//     if (OldDeviceHandle)
//     {
//         if (!(PortExtension->PortPdoFlags & USBHUB_PDO_FLAG_REMOVING_PORT_PDO))
//         {
//             Status = USBD_RemoveDeviceEx(HubExtension,
//                                          OldDeviceHandle,
//                                          IsKeepDeviceData);
//
//             PortExtension->PortPdoFlags |= USBHUB_PDO_FLAG_REMOVING_PORT_PDO;
//         }
//     }
//     else
//     {
//         OldDeviceHandle = NULL;
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         goto ErrorExit;
//     }
//
//     Status = USBH_SyncResetPort(HubExtension, Port);
//
//     if (!NT_SUCCESS(Status))
//     {
//         goto ErrorExit;
//     }
//
//     Status = USBH_SyncGetPortStatus(HubExtension,
//                                     Port,
//                                     &PortStatus,
//                                     sizeof(USB_PORT_STATUS_AND_CHANGE));
//
//     if (!NT_SUCCESS(Status))
//     {
//         goto ErrorExit;
//     }
//
//     Status = USBD_CreateDeviceEx(HubExtension,
//                                  DeviceHandle,
//                                  PortStatus.PortStatus,
//                                  Port);
//
//     if (!NT_SUCCESS(Status))
//     {
//         goto ErrorExit;
//     }
//
//     Status = USBH_SyncResetPort(HubExtension, Port);
//
//     if (IsWait)
//     {
//         USBH_Wait(50);
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         goto ErrorExit;
//     }
//
//     Status = USBD_InitializeDeviceEx(HubExtension,
//                                      *DeviceHandle,
//                                      &PortExtension->DeviceDescriptor.bLength,
//                                      sizeof(PortExtension->DeviceDescriptor),
//                                      &PortExtension->ConfigDescriptor.bLength,
//                                      sizeof(PortExtension->ConfigDescriptor));
//
//     if (NT_SUCCESS(Status))
//     {
//         if (IsKeepDeviceData)
//         {
//             Status = USBD_RestoreDeviceEx(HubExtension,
//                                           OldDeviceHandle,
//                                           *DeviceHandle);
//
//             if (!NT_SUCCESS(Status))
//             {
//                 Handle = InterlockedExchangePointer(DeviceHandle, NULL);
//
//                 USBD_RemoveDeviceEx(HubExtension, Handle, 0);
//                 USBH_SyncDisablePort(HubExtension, Port);
//
//                 Status = STATUS_NO_SUCH_DEVICE;
//             }
//         }
//         else
//         {
//             PortExtension->PortPdoFlags &= ~USBHUB_PDO_FLAG_REMOVING_PORT_PDO;
//         }
//
//         goto Exit;
//     }
//
//     *DeviceHandle = NULL;
//
// ErrorExit:
//
//     NewDeviceHandle = InterlockedExchangePointer(DeviceHandle,
//                                                  OldDeviceHandle);
//
//     if (NewDeviceHandle)
//     {
//         Status = USBD_RemoveDeviceEx(HubExtension, NewDeviceHandle, 0);
//     }
//
// Exit:
//
//     KeReleaseSemaphore(&HubExtension->ResetDeviceSemaphore,
//                        LOW_REALTIME_PRIORITY,
//                        1,
//                        FALSE);
//
//     if (!InterlockedDecrement(&HubExtension->PendingRequestCount))
//     {
//         KeSetEvent(&HubExtension->PendingRequestEvent,
//                    EVENT_INCREMENT,
//                    FALSE);
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_PdoDispatch(IN PUSBHUB_PORT_PDO_EXTENSION PortExtension,
//                  IN PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStack;
//     UCHAR MajorFunction;
//     BOOLEAN ShouldCompleteIrp;
//     ULONG ControlCode;
//     NTSTATUS Status;
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     MajorFunction = IoStack->MajorFunction;
//
//     switch (MajorFunction)
//     {
//         case IRP_MJ_CREATE:
//         case IRP_MJ_CLOSE:
//             DPRINT("USBH_PdoDispatch: IRP_MJ_CREATE / IRP_MJ_CLOSE (%d)\n",
//                    MajorFunction);
//             Status = STATUS_SUCCESS;
//             USBH_CompleteIrp(Irp, Status);
//             break;
//
//         case IRP_MJ_DEVICE_CONTROL:
//             ControlCode = IoStack->Parameters.DeviceIoControl.IoControlCode;
//             DPRINT("USBH_PdoDispatch: IRP_MJ_DEVICE_CONTROL ControlCode - %x\n",
//                    ControlCode);
//
//             if (ControlCode == IOCTL_STORAGE_GET_MEDIA_SERIAL_NUMBER)
//             {
//                 Status = STATUS_NOT_SUPPORTED;
//                 USBH_CompleteIrp(Irp, Status);
//                 break;
//             }
//
//             if (ControlCode == IOCTL_KS_PROPERTY)
//             {
//                 DPRINT1("USBH_PdoDispatch: IOCTL_KS_PROPERTY FIXME\n");
//                 DbgBreakPoint();
//                 Status = STATUS_NOT_SUPPORTED;
//                 USBH_CompleteIrp(Irp, Status);
//                 break;
//             }
//
//             Status = Irp->IoStatus.Status;
//             USBH_CompleteIrp(Irp, Status);
//             break;
//
//         case IRP_MJ_INTERNAL_DEVICE_CONTROL:
//             Status = USBH_PdoInternalControl(PortExtension, Irp);
//             break;
//
//         case IRP_MJ_PNP:
//             Status = USBH_PdoPnP(PortExtension,
//                                  Irp,
//                                  IoStack->MinorFunction,
//                                  &ShouldCompleteIrp);
//
//             if (ShouldCompleteIrp)
//             {
//                 USBH_CompleteIrp(Irp, Status);
//             }
//
//             break;
//
//         case IRP_MJ_POWER:
//             Status = USBH_PdoPower(PortExtension, Irp, IoStack->MinorFunction);
//             break;
//
//         case IRP_MJ_SYSTEM_CONTROL:
//             DPRINT1("USBH_PdoDispatch: USBH_SystemControl() UNIMPLEMENTED. FIXME\n");
//             //USBH_PortSystemControl(PortExtension, Irp);
//             Status = Irp->IoStatus.Status;
//             USBH_CompleteIrp(Irp, Status);
//             break;
//
//         default:
//             DPRINT("USBH_PdoDispatch: Unhandled MajorFunction - %d\n", MajorFunction);
//             Status = Irp->IoStatus.Status;
//             USBH_CompleteIrp(Irp, Status);
//             break;
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_FdoDispatch(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                  IN PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStack;
//     UCHAR MajorFunction;
//     NTSTATUS Status;
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     DPRINT("USBH_FdoDispatch: HubExtension - %p, Irp - %p, MajorFunction - %X\n",
//            HubExtension,
//            Irp,
//            IoStack->MajorFunction);
//
//     MajorFunction = IoStack->MajorFunction;
//
//     switch (MajorFunction)
//     {
//         case IRP_MJ_CREATE:
//         case IRP_MJ_CLOSE:
//             Status = STATUS_SUCCESS;
//             USBH_CompleteIrp(Irp, Status);
//             break;
//
//         case IRP_MJ_DEVICE_CONTROL:
//             Status = USBH_DeviceControl(HubExtension, Irp);
//             break;
//
//         case IRP_MJ_PNP:
//             Status = USBH_FdoPnP(HubExtension, Irp, IoStack->MinorFunction);
//             break;
//
//         case IRP_MJ_POWER:
//             Status = USBH_FdoPower(HubExtension, Irp, IoStack->MinorFunction);
//             break;
//
//         case IRP_MJ_SYSTEM_CONTROL:
//             DPRINT1("USBH_FdoDispatch: USBH_SystemControl() UNIMPLEMENTED. FIXME\n");
//             /* fall through */
//
//         case IRP_MJ_INTERNAL_DEVICE_CONTROL:
//         default:
//             Status = USBH_PassIrp(HubExtension->LowerDevice, Irp);
//             break;
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_AddDevice(IN PDRIVER_OBJECT DriverObject,
//                IN PDEVICE_OBJECT LowerPDO)
// {
//     PDEVICE_OBJECT DeviceObject;
//     NTSTATUS Status;
//     PUSBHUB_FDO_EXTENSION HubExtension;
//     PDEVICE_OBJECT LowerDevice;
//
//     DPRINT("USBH_AddDevice: DriverObject - %p, LowerPDO - %p\n",
//            DriverObject,
//            LowerPDO);
//
//     DeviceObject = NULL;
//
//     Status = IoCreateDevice(DriverObject,
//                             sizeof(USBHUB_FDO_EXTENSION),
//                             NULL,
//                             0x8600,
//                             FILE_AUTOGENERATED_DEVICE_NAME,
//                             FALSE,
//                             &DeviceObject);
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("USBH_AddDevice: IoCreateDevice() fail\n");
//
//         if (DeviceObject)
//         {
//             IoDeleteDevice(DeviceObject);
//         }
//
//         return Status;
//     }
//
//     DPRINT("USBH_AddDevice: DeviceObject - %p\n", DeviceObject);
//
//     HubExtension = DeviceObject->DeviceExtension;
//     RtlZeroMemory(HubExtension, sizeof(USBHUB_FDO_EXTENSION));
//
//     HubExtension->Common.ExtensionType = USBH_EXTENSION_TYPE_HUB;
//
//     LowerDevice = IoAttachDeviceToDeviceStack(DeviceObject, LowerPDO);
//
//     if (!LowerDevice)
//     {
//         DPRINT1("USBH_AddDevice: IoAttachDeviceToDeviceStack() fail\n");
//
//         if (DeviceObject)
//         {
//             IoDeleteDevice(DeviceObject);
//         }
//
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     DPRINT("USBH_AddDevice: LowerDevice  - %p\n", LowerDevice);
//
//     HubExtension->Common.SelfDevice = DeviceObject;
//
//     HubExtension->LowerPDO = LowerPDO;
//     HubExtension->LowerDevice = LowerDevice;
//
//     KeInitializeSemaphore(&HubExtension->IdleSemaphore, 1, 1);
//
//     DeviceObject->Flags |= DO_POWER_PAGABLE;
//     DeviceObject->Flags &= ~DO_DEVICE_INITIALIZING;
//
//     DPRINT("USBH_AddDevice: call IoWMIRegistrationControl() UNIMPLEMENTED. FIXME\n");
//
//     return Status;
// }
//
// VOID
// NTAPI
// USBH_DriverUnload(IN PDRIVER_OBJECT DriverObject)
// {
//     DPRINT("USBH_DriverUnload: UNIMPLEMENTED\n");
//
//     if (GenericUSBDeviceString)
//     {
//         ExFreePool(GenericUSBDeviceString);
//         GenericUSBDeviceString = NULL;
//     }
// }
//
// NTSTATUS
// NTAPI
// USBH_HubDispatch(IN PDEVICE_OBJECT DeviceObject,
//                  IN PIRP Irp)
// {
//     PCOMMON_DEVICE_EXTENSION DeviceExtension;
//     ULONG ExtensionType;
//     NTSTATUS Status;
//
//
//     DeviceExtension = DeviceObject->DeviceExtension;
//     ExtensionType = DeviceExtension->ExtensionType;
//
//     if (ExtensionType == USBH_EXTENSION_TYPE_HUB)
//     {
//         DPRINT("USBH_HubDispatch: DeviceObject - %p, Irp - %p\n",
//                DeviceObject,
//                Irp);
//
//         Status = USBH_FdoDispatch((PUSBHUB_FDO_EXTENSION)DeviceExtension, Irp);
//     }
//     else if (ExtensionType == USBH_EXTENSION_TYPE_PORT)
//     {
//         PIO_STACK_LOCATION IoStack = IoGetCurrentIrpStackLocation(Irp);
//         UCHAR MajorFunction = IoStack->MajorFunction;
//         BOOLEAN IsDprint = TRUE;
//
//         if (MajorFunction == IRP_MJ_INTERNAL_DEVICE_CONTROL)
//         {
//             ULONG ControlCode = IoStack->Parameters.DeviceIoControl.IoControlCode;
//
//             if (ControlCode == IOCTL_INTERNAL_USB_SUBMIT_URB)
//             {
//                 IsDprint = FALSE;
//             }
//         }
//
//         if (IsDprint)
//         {
//             DPRINT("USBH_HubDispatch: DeviceObject - %p, Irp - %p\n",
//                    DeviceObject,
//                    Irp);
//         }
//
//         Status = USBH_PdoDispatch((PUSBHUB_PORT_PDO_EXTENSION)DeviceExtension, Irp);
//     }
//     else
//     {
//         DPRINT1("USBH_HubDispatch: Unknown ExtensionType - %x\n", ExtensionType);
//         DbgBreakPoint();
//         Status = STATUS_ASSERTION_FAILURE;
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_RegQueryGenericUSBDeviceString(PVOID USBDeviceString)
// {
//     RTL_QUERY_REGISTRY_TABLE QueryTable[2];
//
//     DPRINT("USBH_RegQueryGenericUSBDeviceString ... \n");
//
//     RtlZeroMemory(QueryTable, sizeof(QueryTable));
//
//     QueryTable[0].QueryRoutine = USBH_GetConfigValue;
//     QueryTable[0].Flags = RTL_QUERY_REGISTRY_REQUIRED;
//     QueryTable[0].Name = L"GenericUSBDeviceString";
//     QueryTable[0].EntryContext = USBDeviceString;
//     QueryTable[0].DefaultType = REG_NONE;
//     QueryTable[0].DefaultData = 0;
//     QueryTable[0].DefaultLength = 0;
//
//     return RtlQueryRegistryValues(RTL_REGISTRY_CONTROL,
//                                   L"usbflags",
//                                   QueryTable,
//                                   NULL,
//                                   NULL);
// }
//
// NTSTATUS
// NTAPI
// DriverEntry(IN PDRIVER_OBJECT DriverObject,
//             IN PUNICODE_STRING RegistryPath)
// {
//     DPRINT("USBHUB: DriverEntry - %wZ\n", RegistryPath);
//
//     DriverObject->DriverExtension->AddDevice = USBH_AddDevice;
//     DriverObject->DriverUnload = USBH_DriverUnload;
//
//     DriverObject->MajorFunction[IRP_MJ_CREATE] = USBH_HubDispatch;
//     DriverObject->MajorFunction[IRP_MJ_CLOSE] = USBH_HubDispatch;
//
//     DriverObject->MajorFunction[IRP_MJ_DEVICE_CONTROL] = USBH_HubDispatch;
//     DriverObject->MajorFunction[IRP_MJ_INTERNAL_DEVICE_CONTROL] = USBH_HubDispatch;
//
//     DriverObject->MajorFunction[IRP_MJ_PNP] = USBH_HubDispatch;
//     DriverObject->MajorFunction[IRP_MJ_POWER] = USBH_HubDispatch;
//     DriverObject->MajorFunction[IRP_MJ_SYSTEM_CONTROL] = USBH_HubDispatch;
//
//     USBH_RegQueryGenericUSBDeviceString(&GenericUSBDeviceString);
//
//     return STATUS_SUCCESS;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
