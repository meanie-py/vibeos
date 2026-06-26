//! usbport glue — cohab shims not in the bindgen types.rs. DbgPrint/RtlAssert and the
//! kernel/HAL imports resolve from the ntoskrnl/hal import libs (modules declare them in
//! their own extern blocks). Mirrors the usbehci/ohci/uhci miniport glue.
#![allow(unused_imports)]

pub use crate::types::*;

// ---- kernel type aliases the modules name bare but don't import from ddk ----
pub use ntkrnl_sys::ddk::PCSTR;
use ntkrnl_sys::base::PVOID;
use ntkrnl_sys::base::ULONG;
use ntkrnl_sys::base::{BOOLEAN, NTSTATUS, KIRQL};
pub use ntkrnl_sys::ddk::DEVICE_REGISTRY_PROPERTY_DevicePropertyDriverKeyName as DevicePropertyDriverKeyName;
pub use ntkrnl_sys::ddk::{
    ACCESS_MASK, CCHAR, GUID, IO_ALLOCATION_ACTION, KEY_VALUE_INFORMATION_CLASS, KPRIORITY,
    KPROCESSOR_MODE, PRKTHREAD, _KSEMAPHORE, _LIST_ENTRY, _UNICODE_STRING, wchar_t,
};
pub use crate::types::_USB_PORT_STATUS_AND_CHANGE as USB_PORT_STATUS_AND_CHANGE;

// ---- misc consts ----
pub const USB_PORT_TAG: u32 = 0x70627375; // 'pbsu' (usbport.h:44)
pub const EVENT_INCREMENT: i8 = 1; // ntddk.h

// ---- USB_DEVICE_SPEED / USB_DEVICE_TYPE enum members (crate bindgen names them `_ENUM_*`) ----
pub use crate::types::_USB_DEVICE_SPEED_UsbLowSpeed as UsbLowSpeed;
pub use crate::types::_USB_DEVICE_SPEED_UsbFullSpeed as UsbFullSpeed;
pub use crate::types::_USB_DEVICE_SPEED_UsbHighSpeed as UsbHighSpeed;
pub use crate::types::_USB_DEVICE_TYPE_Usb11Device as Usb11Device;
pub use crate::types::_USB_DEVICE_TYPE_Usb20Device as Usb20Device;

// ---- kernel enum members imported by bare name (bindgen names them `_ENUM_Member`) ----
pub use ntkrnl_sys::ddk::_DEVICE_POWER_STATE_PowerDeviceD0 as PowerDeviceD0;
pub use ntkrnl_sys::ddk::_DEVICE_POWER_STATE_PowerDeviceD1 as PowerDeviceD1;
pub use ntkrnl_sys::ddk::_DEVICE_POWER_STATE_PowerDeviceD2 as PowerDeviceD2;
pub use ntkrnl_sys::ddk::_DEVICE_POWER_STATE_PowerDeviceD3 as PowerDeviceD3;
pub use ntkrnl_sys::ddk::_SYSTEM_POWER_STATE_PowerSystemUnspecified as PowerSystemUnspecified;
pub use ntkrnl_sys::ddk::_SYSTEM_POWER_STATE_PowerSystemWorking as PowerSystemWorking;
pub use ntkrnl_sys::ddk::_SYSTEM_POWER_STATE_PowerSystemSleeping1 as PowerSystemSleeping1;
pub use ntkrnl_sys::ddk::_SYSTEM_POWER_STATE_PowerSystemSleeping2 as PowerSystemSleeping2;
pub use ntkrnl_sys::ddk::_SYSTEM_POWER_STATE_PowerSystemSleeping3 as PowerSystemSleeping3;
pub use ntkrnl_sys::ddk::_SYSTEM_POWER_STATE_PowerSystemHibernate as PowerSystemHibernate;
pub use ntkrnl_sys::ddk::_POWER_STATE_TYPE_DevicePowerState as DevicePowerState;
pub use ntkrnl_sys::ddk::_WORK_QUEUE_TYPE_CriticalWorkQueue as CriticalWorkQueue;
pub use ntkrnl_sys::ddk::_KEY_VALUE_INFORMATION_CLASS_KeyValueFullInformation as KeyValueFullInformation;
pub use ntkrnl_sys::ddk::_MM_PAGE_PRIORITY_NormalPagePriority as NormalPagePriority;
pub use ntkrnl_sys::ddk::_IO_ALLOCATION_ACTION_DeallocateObjectKeepRegisters as DeallocateObjectKeepRegisters;

// ---- USBD_STATUS codes (usb.h). USBD_STATUS = LONG. Values verified vs the C header. ----
use ntkrnl_sys::ddk::LONG;
pub const USBD_STATUS_SUCCESS: LONG = 0x00000000u32 as LONG;
pub const USBD_STATUS_HALTED: LONG = 0xC0000000u32 as LONG;
pub const USBD_STATUS_STALL_PID: LONG = 0xC0000004u32 as LONG;
pub const USBD_STATUS_DEVICE_GONE: LONG = 0xC0000005u32 as LONG;
pub const USBD_STATUS_NOT_SUPPORTED: LONG = 0xC0000E00u32 as LONG;
pub const USBD_STATUS_INVALID_URB_FUNCTION: LONG = 0x80000200u32 as LONG;
pub const USBD_STATUS_INVALID_PARAMETER: LONG = 0x80000300u32 as LONG;
pub const USBD_STATUS_INVALID_PIPE_HANDLE: LONG = 0x80000600u32 as LONG;
pub const USBD_STATUS_INVALID_PIPE_FLAGS: LONG = 0x80000700u32 as LONG;
pub const USBD_STATUS_INSUFFICIENT_RESOURCES: LONG = 0x80000C00u32 as LONG;
pub const USBD_STATUS_INTERFACE_NOT_FOUND: LONG = 0x80000A00u32 as LONG;
pub const USBD_STATUS_DEVICE_NOT_RESPONDING: LONG = 0xC0000005u32 as LONG;

// ---- DPRINT family — NDEBUG-disabled DbgPrint wrappers (usbdebug.h). Variadic decls put the
// name in scope for modules that don't declare it themselves; the no-op symbols satisfy the
// link (cdecl, caller cleans varargs). ----
extern "C" {
    pub fn DPRINT(fmt: *const ::core::ffi::c_char, ...);
    pub fn DPRINT_CORE(fmt: *const ::core::ffi::c_char, ...);
    pub fn DPRINT_URB(fmt: *const ::core::ffi::c_char, ...);
    pub fn DPRINT_QUEUE(fmt: *const ::core::ffi::c_char, ...);
    pub fn DPRINT_USB(fmt: *const ::core::ffi::c_char, ...);
    pub fn DPRINT_TIMER(fmt: *const ::core::ffi::c_char, ...);
    pub fn DPRINT_INT(fmt: *const ::core::ffi::c_char, ...);
    pub fn DPRINT_MINIPORT(fmt: *const ::core::ffi::c_char, ...);
}
mod dprint_syms {
    macro_rules! noop { ($($n:ident),*) => { $(
        #[no_mangle] unsafe extern "C" fn $n(_fmt: *const ::core::ffi::c_char) {}
    )* } }
    noop!(DPRINT, DPRINT_CORE, DPRINT_URB, DPRINT_QUEUE, DPRINT_USB, DPRINT_TIMER, DPRINT_INT, DPRINT_MINIPORT);
}

// ASSERT — NT_ASSERT, no-op in retail (NDEBUG).
#[inline] pub fn ASSERT(_cond: i32) {}

// ---- KeQuerySystemTime / SharedSystemTime — read KI_USER_SHARED_DATA.SystemTime ----
#[inline]
pub unsafe fn KeQuerySystemTime(current_time: *mut ntkrnl_sys::ddk::_LARGE_INTEGER) {
    let shared = ntkrnl_sys::mm::KI_USER_SHARED_DATA as *mut ntkrnl_sys::ddk::_KUSER_SHARED_DATA;
    loop {
        let high1 = core::ptr::addr_of!((*shared).SystemTime.High1Time).read_volatile();
        let low = core::ptr::addr_of!((*shared).SystemTime.LowPart).read_volatile();
        let high2 = core::ptr::addr_of!((*shared).SystemTime.High2Time).read_volatile();
        if high1 == high2 {
            (*current_time).QuadPart = ((high1 as i64) << 32) | (low as u32 as i64);
            break;
        }
    }
}
#[inline]
pub unsafe fn SharedSystemTimePtr() -> *mut ntkrnl_sys::ddk::ULONG64 {
    let shared = ntkrnl_sys::mm::KI_USER_SHARED_DATA as *mut ntkrnl_sys::ddk::_KUSER_SHARED_DATA;
    core::ptr::addr_of_mut!((*shared).SystemTime) as *mut ntkrnl_sys::ddk::ULONG64
}

// MmGetMdlVirtualAddress — wdm.h FORCEINLINE: MDL->StartVa + MDL->ByteOffset.
#[inline]
pub unsafe fn MmGetMdlVirtualAddress(Mdl: *mut ntkrnl_sys::ddk::_MDL) -> *mut core::ffi::c_void {
    ((*Mdl).StartVa as *mut u8).add((*Mdl).ByteOffset as usize) as *mut core::ffi::c_void
}

// ---- IRQL — driver-side KeRaiseIrql/KeLowerIrql (same pattern as scsiport).
// i386: fastcall KfRaiseIrql/KfLowerIrql HAL exports. amd64: IRQL IS cr8 — the DDK
// accessors are FORCEINLINE, NOT driver-visible exports (the kernel-internal tier1
// KeRaiseIrql is unavailable to a .sys). arm64: real KfRaiseIrql/KeLowerIrql exports.
#[cfg(target_arch = "x86")]
pub use ntkrnl_sys::inline::tier1::{KeRaiseIrql, KeLowerIrql};
#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn KeRaiseIrql(NewIrql: KIRQL, OldIrql: *mut KIRQL) {
    let old: u64;
    core::arch::asm!("mov {}, cr8", out(reg) old, options(nomem, nostack));
    *OldIrql = old as KIRQL;
    core::arch::asm!("mov cr8, {}", in(reg) NewIrql as u64, options(nomem, nostack));
}
#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn KeLowerIrql(NewIrql: KIRQL) {
    core::arch::asm!("mov cr8, {}", in(reg) NewIrql as u64, options(nomem, nostack));
}
#[cfg(target_arch = "aarch64")]
extern "C" {
    fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL;
}
#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub unsafe fn KeRaiseIrql(NewIrql: KIRQL, OldIrql: *mut KIRQL) {
    *OldIrql = KfRaiseIrql(NewIrql);
}
#[cfg(target_arch = "aarch64")]
mod lower_irql {
    extern "C" {
        pub fn KeLowerIrql(NewIrql: super::KIRQL);
    }
}
#[cfg(target_arch = "aarch64")]
pub use lower_irql::KeLowerIrql;

// ---- BM_REQUEST_TYPE — bindgen collapsed the bitfield struct to just `B: UCHAR`; supply the
// USB-spec bmRequestType bitfield accessors (Recipient[0:4], Type[5:6], Dir[7]) on the byte. ----
impl crate::types::_BM_REQUEST_TYPE {
    #[inline] pub fn Recipient(&self) -> ntkrnl_sys::ddk::UCHAR { unsafe { self.B & 0x1F } }
    #[inline] pub fn set_Recipient(&mut self, v: ntkrnl_sys::ddk::UCHAR) { unsafe { self.B = (self.B & !0x1F) | (v & 0x1F); } }
    #[inline] pub fn Type(&self) -> ntkrnl_sys::ddk::UCHAR { unsafe { (self.B >> 5) & 0x3 } }
    #[inline] pub fn set_Type(&mut self, v: ntkrnl_sys::ddk::UCHAR) { unsafe { self.B = (self.B & !0x60) | ((v & 0x3) << 5); } }
    #[inline] pub fn Dir(&self) -> ntkrnl_sys::ddk::UCHAR { unsafe { (self.B >> 7) & 0x1 } }
    #[inline] pub fn set_Dir(&mut self, v: ntkrnl_sys::ddk::UCHAR) { unsafe { self.B = (self.B & !0x80) | ((v & 0x1) << 7); } }
    #[inline] pub fn Reserved(&self) -> ntkrnl_sys::ddk::UCHAR { 0 }
    #[inline] pub fn set_Reserved(&mut self, _v: ntkrnl_sys::ddk::UCHAR) {}
}

// ---- USBPORT_MPStatusToRHStatus (roothub.c:18) — dropped by the pure-LLM pass; faithful re-add. ----
pub const RH_STATUS_SUCCESS: ntkrnl_sys::ddk::ULONG = 0;
#[no_mangle]
pub unsafe extern "system" fn USBPORT_MPStatusToRHStatus(MPStatus: MPSTATUS) -> RHSTATUS {
    let mut rh: RHSTATUS = RH_STATUS_SUCCESS;
    if MPStatus != 0 {
        rh = (MPStatus != MP_STATUS_FAILURE) as RHSTATUS;
        rh += 1;
    }
    rh
}

// ---- misc consts / inline helpers ----
pub const USBD_STATUS_BUFFER_TOO_SMALL: LONG = 0xC0003000u32 as LONG;
pub const BUGCODE_USB_DRIVER: u32 = 0xFE; // bugcheck.h
#[inline] pub fn NT_ERROR(status: ntkrnl_sys::ddk::NTSTATUS) -> bool { ((status as u32) >> 30) == 3 }
#[inline] pub unsafe fn __int2c() {} // __debugbreak-ish; no-op in retail
#[inline] pub unsafe fn wcslen(mut s: *const u16) -> usize { let mut n = 0; while *s != 0 { s = s.add(1); n += 1; } n }
#[inline] pub unsafe fn InterlockedCompareExchange(dest: *mut LONG, exch: LONG, comp: LONG) -> LONG {
    let a = &*(dest as *const core::sync::atomic::AtomicI32);
    match a.compare_exchange(comp, exch, core::sync::atomic::Ordering::SeqCst, core::sync::atomic::Ordering::SeqCst) {
        Ok(v) | Err(v) => v,
    }
}

// ---- usbport: types/consts bindgen inlined or omitted (faithful re-decls) ----
pub const USBD_STATUS_BAD_START_FRAME: USBD_STATUS = 0xC0000A00u32 as USBD_STATUS;

// ASYNC_TIMER_CALLBACK: usbport.h `typedef VOID ASYNC_TIMER_CALLBACK(PVOID, PVOID)` —
// bindgen inlined the field type, so the named alias is absent. Matches the
// _USBPORT_ASYNC_CALLBACK_DATA.CallbackFunction slot Option<fn(PVOID,PVOID)>.
#[cfg(target_arch = "x86")]
pub type ASYNC_TIMER_CALLBACK = ::core::option::Option<unsafe extern "system" fn(arg1: PVOID, arg2: PVOID)>;

// USB_IDLE_CALLBACK_INFO: usbiodef.h — usbiodef.h not in the bindgen allowlist, re-declared.
pub type USB_IDLE_CALLBACK = ::core::option::Option<unsafe extern "system" fn(Context: PVOID)>;
#[repr(C)]
pub struct USB_IDLE_CALLBACK_INFO {
    pub IdleCallback: USB_IDLE_CALLBACK,
    pub IdleContext: PVOID,
}
pub type PUSB_IDLE_CALLBACK_INFO = *mut USB_IDLE_CALLBACK_INFO;

// USB_HUB_STATUS_AND_CHANGE: un-prefixed alias for the bindgen `_`-prefixed union.
pub use crate::types::_USB_HUB_STATUS_AND_CHANGE as USB_HUB_STATUS_AND_CHANGE;

// ---- USB user-mode IOCTL types/consts (usbuser.h + usbioctl.h; not in bindgen allowlist) ----
pub type USB_USER_ERROR_CODE = i32;
pub const UsbUserSuccess: USB_USER_ERROR_CODE = 0;
pub const UsbUserNotSupported: USB_USER_ERROR_CODE = 1;
pub const UsbUserInvalidRequestCode: USB_USER_ERROR_CODE = 2;
pub const UsbUserFeatureDisabled: USB_USER_ERROR_CODE = 3;
pub const UsbUserInvalidHeaderParameter: USB_USER_ERROR_CODE = 4;
pub const UsbUserInvalidParameter: USB_USER_ERROR_CODE = 5;
pub const UsbUserMiniportError: USB_USER_ERROR_CODE = 6;
pub const UsbUserBufferTooSmall: USB_USER_ERROR_CODE = 7;
pub const UsbUserErrorNotMapped: USB_USER_ERROR_CODE = 8;
pub const UsbUserDeviceNotStarted: USB_USER_ERROR_CODE = 9;
pub const UsbUserNoDeviceConnected: USB_USER_ERROR_CODE = 10;

#[repr(C)]
pub struct USBUSER_REQUEST_HEADER {
    pub UsbUserRequest: ULONG,
    pub UsbUserStatusCode: USB_USER_ERROR_CODE,
    pub RequestBufferLength: ULONG,
    pub ActualBufferLength: ULONG,
}
pub type PUSBUSER_REQUEST_HEADER = *mut USBUSER_REQUEST_HEADER;

#[repr(C)]
pub struct USB_UNICODE_NAME {
    pub Length: ULONG,
    pub String: [u16; 1],
}
pub type PUSB_UNICODE_NAME = *mut USB_UNICODE_NAME;

#[repr(C)]
pub struct USBUSER_CONTROLLER_UNICODE_NAME {
    pub Header: USBUSER_REQUEST_HEADER,
    pub UnicodeName: USB_UNICODE_NAME,
}
pub type PUSBUSER_CONTROLLER_UNICODE_NAME = *mut USBUSER_CONTROLLER_UNICODE_NAME;

#[repr(C)]
pub struct USB_HCD_DRIVERKEY_NAME {
    pub ActualLength: ULONG,
    pub DriverKeyName: [u16; 1],
}
pub type PUSB_HCD_DRIVERKEY_NAME = *mut USB_HCD_DRIVERKEY_NAME;

pub const USBUSER_GET_CONTROLLER_DRIVER_KEY: ULONG = 0x00000002;
pub const USBUSER_GET_ROOTHUB_SYMBOLIC_NAME: ULONG = 0x00000007;
pub const IOCTL_GET_HCD_DRIVERKEY_NAME: ULONG = 0x00220424;

pub const USBD_STATUS_CANCELED: USBD_STATUS = 0xC0010000u32 as USBD_STATUS;
pub const USBD_STATUS_NO_BANDWIDTH: USBD_STATUS = 0x80000700u32 as USBD_STATUS;
pub const USBD_STATUS_TIMEOUT: USBD_STATUS = 0xC0006000u32 as USBD_STATUS;
pub const USBD_STATUS_PENDING: USBD_STATUS = 0x40000000u32 as USBD_STATUS;

// usb100.h direction macros (object-like, not bindgen-emitted)
pub const USB_ENDPOINT_DIRECTION_MASK: u8 = 0x80;
#[inline] pub unsafe fn USB_ENDPOINT_DIRECTION_OUT(x: u8) -> u32 { ((x & USB_ENDPOINT_DIRECTION_MASK) == 0) as u32 }
#[inline] pub unsafe fn USB_ENDPOINT_DIRECTION_IN(x: u8) -> u32 { ((x & USB_ENDPOINT_DIRECTION_MASK) != 0) as u32 }

// ---- C-macro inlines + intrinsic (kept as extern "C" decls in the modules; the
// linker resolves them to these #[no_mangle] cdecl symbols). The modules' local
// extern decls shadow the glob-imported names, so no Rust name clash. ----
#[no_mangle]
pub unsafe extern "C" fn IoGetCurrentIrpStackLocation(
    Irp: ntkrnl_sys::ddk::PIRP,
) -> ntkrnl_sys::ddk::PIO_STACK_LOCATION {
    ntkrnl_sys::inline::tier1::IoGetCurrentIrpStackLocation(Irp)
}
#[no_mangle]
pub unsafe extern "C" fn IoSkipCurrentIrpStackLocation(Irp: ntkrnl_sys::ddk::PIRP) {
    ntkrnl_sys::inline::tier1::IoSkipCurrentIrpStackLocation(Irp)
}
#[no_mangle]
pub unsafe extern "C" fn URB_FROM_IRP(Irp: ntkrnl_sys::ddk::PIRP) -> PURB {
    (*ntkrnl_sys::inline::tier1::IoGetCurrentIrpStackLocation(Irp))
        .Parameters
        .Others
        .Argument1 as PURB
}
#[no_mangle]
pub unsafe extern "C" fn USBD_TRANSFER_DIRECTION_FLAG(flags: u32) -> u32 {
    flags & 0x00000001 // USBD_TRANSFER_DIRECTION
}
#[no_mangle]
pub unsafe extern "C" fn _InterlockedCompareExchange(dest: *mut i32, exch: i32, comp: i32) -> i32 {
    let a = &*(dest as *const core::sync::atomic::AtomicI32);
    match a.compare_exchange(
        comp,
        exch,
        core::sync::atomic::Ordering::SeqCst,
        core::sync::atomic::Ordering::SeqCst,
    ) {
        Ok(v) | Err(v) => v,
    }
}

// ---- ddk macro-inlines referenced by the modules (forward to ntkrnl_sys inlines) ----
#[no_mangle]
pub unsafe extern "C" fn IoCopyCurrentIrpStackLocationToNext(Irp: ntkrnl_sys::ddk::PIRP) {
    ntkrnl_sys::inline::tier1::IoCopyCurrentIrpStackLocationToNext(Irp)
}
#[no_mangle]
pub unsafe extern "C" fn IoMarkIrpPending(Irp: ntkrnl_sys::ddk::PIRP) {
    ntkrnl_sys::inline::tier1::IoMarkIrpPending(Irp)
}
#[no_mangle]
pub unsafe extern "C" fn IoSetCancelRoutine(
    Irp: ntkrnl_sys::ddk::PIRP,
    CancelRoutine: ntkrnl_sys::ddk::PDRIVER_CANCEL,
) -> ntkrnl_sys::ddk::PDRIVER_CANCEL {
    ntkrnl_sys::inline::tier1::IoSetCancelRoutine(Irp, CancelRoutine)
}
#[no_mangle]
pub unsafe extern "C" fn KeInitializeSpinLock(SpinLock: *mut usize) {
    *SpinLock = 0;
}
// USBD_SUCCESS(Status) = ((USBD_STATUS)(Status) >= 0); USBD_ERROR = (< 0). Sign-tested as LONG.
#[no_mangle]
pub unsafe extern "system" fn USBD_SUCCESS(Status: USBD_STATUS) -> BOOLEAN {
    ((Status as i32) >= 0) as BOOLEAN
}
#[no_mangle]
pub unsafe extern "system" fn USBD_ERROR(Status: USBD_STATUS) -> BOOLEAN {
    ((Status as i32) < 0) as BOOLEAN
}
// RtlStringCbPrintfW: ntstrsafe FORCEINLINE (no exported symbol). usbport uses it only to
// format a registry path in a non-boot path; link-clean stub (usbport is NOT boot-loaded).
#[no_mangle]
pub unsafe extern "C" fn RtlStringCbPrintfW(
    _pszDest: *mut u16,
    _cbDest: usize,
    _pszFormat: *const u16,
    mut _args: ...
) -> NTSTATUS {
    0 // STATUS_SUCCESS
}

// IsEqualGUIDAligned (wdm FORCEINLINE) — compares the two 8-byte halves. iface.rs declares
// it extern (NTAPI), so resolve it to this #[no_mangle] symbol.
#[no_mangle]
pub unsafe extern "system" fn IsEqualGUIDAligned(
    guid1: *const GUID,
    guid2: *const GUID,
) -> BOOLEAN {
    let a = guid1 as *const u64;
    let b = guid2 as *const u64;
    ((*a == *b) && (*a.add(1) == *b.add(1))) as BOOLEAN
}

// KeGetCurrentPrcb — FORCEINLINE (ketypes.h). On arm64 it is not a driver export, so the
// USBPORT_WorkerThread reference needs a concrete symbol; forward to the ntkrnl-sys inline.
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn KeGetCurrentPrcb() -> ntkrnl_sys::ddk::PKPRCB {
    ntkrnl_sys::inline::ke::KeGetCurrentPrcb()
}

// ---- pnp.rs: PCI/IRP constants + GUID data symbols (the latter instantiated by guid.c
// via initguid.h; referenced here as extern statics, resolved at link within usbport.sys) ----
pub const PCI_USE_CLASS_SUBCLASS: u32 = 0x00000008;
pub const PCI_USE_PROGIF: u32 = 0x00000010;
pub const PCI_USE_LOCAL_BUS: u32 = 0x00000020;
pub const PCI_USE_LOCAL_DEVICE: u32 = 0x00000040;
pub const PCI_CLASS_SERIAL_BUS_CTLR: u32 = 0x0c;
pub const PCI_SUBCLASS_SB_USB: u32 = 0x03;
pub const IRP_MN_QUERY_INTERFACE: u8 = 8;
pub const IRP_MN_QUERY_CAPABILITIES: u8 = 9;

extern "C" {
    pub static GUID_BUS_INTERFACE_STANDARD: GUID;
    pub static GUID_PCI_DEVICE_PRESENT_INTERFACE: GUID;
    pub static GUID_DEVINTERFACE_USB_HOST_CONTROLLER: GUID;
    pub static GUID_DEVINTERFACE_USB_HUB: GUID;
    pub static GUID_BUS_TYPE_USB: GUID;
}

// pnp.rs enum-member aliases (bindgen names them `_ENUM_Member`)
pub use ntkrnl_sys::ddk::_DEVICE_RELATION_TYPE as DEVICE_RELATION_TYPE;
pub use ntkrnl_sys::ddk::_BUS_QUERY_ID_TYPE_BusQueryDeviceID as BusQueryDeviceID;
pub use ntkrnl_sys::ddk::_BUS_QUERY_ID_TYPE_BusQueryHardwareIDs as BusQueryHardwareIDs;
pub use ntkrnl_sys::ddk::_BUS_QUERY_ID_TYPE_BusQueryCompatibleIDs as BusQueryCompatibleIDs;
pub use ntkrnl_sys::ddk::_BUS_QUERY_ID_TYPE_BusQueryInstanceID as BusQueryInstanceID;
pub use ntkrnl_sys::ddk::_BUS_QUERY_ID_TYPE_BusQueryDeviceSerialNumber as BusQueryDeviceSerialNumber;

// pnp.rs: more enum-member aliases
pub use ntkrnl_sys::ddk::_DEVICE_RELATION_TYPE_BusRelations as BusRelations;
pub use ntkrnl_sys::ddk::_DEVICE_RELATION_TYPE_RemovalRelations as RemovalRelations;
pub use ntkrnl_sys::ddk::_DEVICE_RELATION_TYPE_TargetDeviceRelation as TargetDeviceRelation;
pub use ntkrnl_sys::ddk::_CM_SHARE_DISPOSITION_CmResourceShareShared as CmResourceShareShared;
pub use ntkrnl_sys::ddk::_DMA_SPEED_Compatible as Compatible;
pub use ntkrnl_sys::ddk::DEVICE_REGISTRY_PROPERTY_DevicePropertyBusNumber as DevicePropertyBusNumber;
pub use ntkrnl_sys::ddk::_INTERFACE_TYPE_PCIBus as PCIBus;
pub use ntkrnl_sys::ddk::_INTERFACE_TYPE_PNPBus as PNPBus;
pub use ntkrnl_sys::ddk::_DMA_WIDTH_Width32Bits as Width32Bits;

// ---- pnp.rs ddk macro-inlines (forward to ntkrnl_sys inlines) ----
#[no_mangle]
pub unsafe extern "C" fn IoGetNextIrpStackLocation(
    Irp: ntkrnl_sys::ddk::PIRP,
) -> ntkrnl_sys::ddk::PIO_STACK_LOCATION {
    ntkrnl_sys::inline::tier1::IoGetNextIrpStackLocation(Irp)
}
#[no_mangle]
pub unsafe extern "C" fn IoSetCompletionRoutine(
    Irp: ntkrnl_sys::ddk::PIRP,
    CompletionRoutine: ntkrnl_sys::ddk::PIO_COMPLETION_ROUTINE,
    Context: PVOID,
    InvokeOnSuccess: BOOLEAN,
    InvokeOnError: BOOLEAN,
    InvokeOnCancel: BOOLEAN,
) {
    ntkrnl_sys::inline::tier1::IoSetCompletionRoutine(
        Irp, CompletionRoutine, Context, InvokeOnSuccess, InvokeOnError, InvokeOnCancel,
    )
}
// RtlStringCbPrintfExW: ntstrsafe FORCEINLINE, no export. Link-clean stub (non-boot path).
#[no_mangle]
pub unsafe extern "C" fn RtlStringCbPrintfExW(
    _pszDest: *mut u16,
    _cbDest: usize,
    _ppszDestEnd: *mut *mut u16,
    _pcbRemaining: *mut usize,
    _dwFlags: u32,
    _pszFormat: *const u16,
    mut _args: ...
) -> i32 {
    0 // STATUS_SUCCESS
}
