//! usbport glue — cohab shims not in the bindgen types.rs. DbgPrint/RtlAssert and the
//! kernel/HAL imports resolve from the ntoskrnl/hal import libs (modules declare them in
//! their own extern blocks). Mirrors the usbehci/ohci/uhci miniport glue.
#![allow(unused_imports)]

// C-variadic entry points live in drivers/usb/usbport/rust_varargs_shim.c (stable Rust
// cannot DEFINE one — c_variadic, rust-lang#44930). `VaArgs` reads their va_list.
use varargs_rust::VaArgs;

pub use crate::types::*;

// PnP minor function codes (wdm.h). usbport's PDO/dispatch match arms use these;
// they live here (not driver_ddk) because every driver crate keeps its own copy —
// a shared driver_ddk::* glob collides with each crate's local set (E0659).
pub const IRP_MN_QUERY_INTERFACE: u8 = 0x08;
pub const IRP_MN_QUERY_CAPABILITIES: u8 = 0x09;
pub const IRP_MN_QUERY_RESOURCES: u8 = 0x0A;
pub const IRP_MN_QUERY_RESOURCE_REQUIREMENTS: u8 = 0x0B;
pub const IRP_MN_QUERY_DEVICE_TEXT: u8 = 0x0C;
pub const IRP_MN_FILTER_RESOURCE_REQUIREMENTS: u8 = 0x0D;
pub const IRP_MN_READ_CONFIG: u8 = 0x0F;
pub const IRP_MN_WRITE_CONFIG: u8 = 0x10;
pub const IRP_MN_EJECT: u8 = 0x11;
pub const IRP_MN_SET_LOCK: u8 = 0x12;
pub const IRP_MN_QUERY_ID: u8 = 0x13;
pub const IRP_MN_QUERY_PNP_DEVICE_STATE: u8 = 0x14;
pub const IRP_MN_QUERY_BUS_INFORMATION: u8 = 0x15;
pub const IRP_MN_DEVICE_USAGE_NOTIFICATION: u8 = 0x16;
pub const IRP_MN_QUERY_LEGACY_BUS_INFORMATION: u8 = 0x18;

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
pub const USBD_STATUS_DEVICE_GONE: LONG = 0xC0007000u32 as i32;
pub const USBD_STATUS_NOT_SUPPORTED: LONG = 0xC0000E00u32 as LONG;
pub const USBD_STATUS_INVALID_URB_FUNCTION: LONG = 0x80000200u32 as LONG;
pub const USBD_STATUS_INVALID_PARAMETER: LONG = 0x80000300u32 as LONG;
pub const USBD_STATUS_INVALID_PIPE_HANDLE: LONG = 0x80000600u32 as LONG;
pub const USBD_STATUS_INVALID_PIPE_FLAGS: LONG = 0xC0005000u32 as i32;
pub const USBD_STATUS_INSUFFICIENT_RESOURCES: LONG = 0xC0001000u32 as i32;
pub const USBD_STATUS_INTERFACE_NOT_FOUND: LONG = 0xC0004000u32 as i32;
pub const USBD_STATUS_DEVICE_NOT_RESPONDING: LONG = 0xC0000005u32 as LONG;

// ---- DPRINT family — NDEBUG-disabled DbgPrint wrappers (usbdebug.h). Variadic decls put the
// name in scope for modules that don't declare it themselves; the no-op symbols satisfy the
// link (cdecl, caller cleans varargs). ----
extern "C" {
    pub fn DPRINT(fmt: *const ::core::ffi::c_char, ...);
    pub fn DPRINT_USB(fmt: *const ::core::ffi::c_char, ...);
}
mod dprint_syms {
    macro_rules! noop { ($($n:ident),*) => { $(
        #[no_mangle] unsafe extern "C" fn $n(_fmt: *const ::core::ffi::c_char) {}
    )* } }
    // DPRINT_CORE/URB/QUEUE/TIMER/INT/MINIPORT relocated to drvtrace-shim
    // (§9.99: variadic call sites + fixed-arity in-crate def = instcombine ud2
    // fold). DPRINT/DPRINT_USB stay local: exact-arity callers only.
    noop!(DPRINT, DPRINT_USB);
}

// ASSERT — NT_ASSERT, no-op in retail (NDEBUG).

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
    extern "system" {
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
// CC = ntkrnl-sys bindgen_i386 decl class (callers import THAT decl; a mismatched
// shim def in the same crate folds every such call to ud2 — the 2026-07-18 i386
// driver-fleet bomb class).
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
// RtlStringCbPrintfW / RtlStringCbPrintfExW / RtlStringCbCopyW: ntstrsafe FORCEINLINEs
// (no exported kernel symbol), so the crate must carry real bodies. usbport IS boot-loaded
// (usbehci/ohci/uhci import usbport.sys and are Boot Bus Extender services), and these
// format the FDO/PDO device names and the root-hub device/hardware IDs — an empty string
// here breaks PnP enumeration of the entire USB bus. Mini formatter below supports the
// specifiers usbport actually uses (%d, %u, %[0][width]x/X, %s, %c, %%) and FAILS LOUD
// (DbgPrint + STATUS_INVALID_PARAMETER) on anything else.
const STATUS_BUFFER_OVERFLOW_W: NTSTATUS = 0x80000005u32 as NTSTATUS;
const STATUS_INVALID_PARAMETER_W: NTSTATUS = 0xC000000Du32 as NTSTATUS;

struct WOut {
    dest: *mut u16,
    cap_chars: usize, // capacity in WCHARs, including the NUL slot
    len: usize,
    overflow: bool,
}

impl WOut {
    #[inline]
    unsafe fn push(&mut self, c: u16) {
        if self.len + 1 < self.cap_chars {
            *self.dest.add(self.len) = c;
            self.len += 1;
        } else {
            self.overflow = true;
        }
    }

    unsafe fn push_num(&mut self, val: u32, base: u32, upper: bool, zero_pad: bool, width: usize, negative: bool) {
        let digits = if upper {
            b"0123456789ABCDEF"
        } else {
            b"0123456789abcdef"
        };
        let mut tmp = [0u16; 24];
        let mut n = val;
        let mut i = 0usize;
        loop {
            tmp[i] = digits[(n % base) as usize] as u16;
            n /= base;
            i += 1;
            if n == 0 {
                break;
            }
        }
        let body = i + if negative { 1 } else { 0 };
        if zero_pad && negative {
            self.push(b'-' as u16);
        }
        let mut pad = width.saturating_sub(body);
        while pad > 0 {
            self.push(if zero_pad { b'0' as u16 } else { b' ' as u16 });
            pad -= 1;
        }
        if !zero_pad && negative {
            self.push(b'-' as u16);
        }
        while i > 0 {
            i -= 1;
            self.push(tmp[i]);
        }
    }
}

/// Shared ntstrsafe-style formatter core. Returns (chars_written, status).
unsafe fn strsafe_vformat_w(
    pszDest: *mut u16,
    cbDest: usize,
    pszFormat: *const u16,
    args: &mut VaArgs,
) -> (usize, NTSTATUS) {
    if pszDest.is_null() || pszFormat.is_null() || cbDest < core::mem::size_of::<u16>() {
        return (0, STATUS_INVALID_PARAMETER_W);
    }
    let mut out = WOut {
        dest: pszDest,
        cap_chars: cbDest / core::mem::size_of::<u16>(),
        len: 0,
        overflow: false,
    };
    let mut p = pszFormat;
    while *p != 0 {
        let c = *p;
        p = p.add(1);
        if c != b'%' as u16 {
            out.push(c);
            continue;
        }
        // parse [0][width] then specifier
        let mut zero_pad = false;
        let mut width = 0usize;
        if *p == b'0' as u16 {
            zero_pad = true;
            p = p.add(1);
        }
        while *p >= b'0' as u16 && *p <= b'9' as u16 {
            width = width * 10 + (*p - b'0' as u16) as usize;
            p = p.add(1);
        }
        // skip C length prefix 'l' (e.g. %lx) — args are still 32-bit-promoted here
        if *p == b'l' as u16 {
            p = p.add(1);
        }
        let spec = *p;
        p = p.add(1);
        match spec as u8 {
            b'd' | b'i' => {
                let v = args.arg::<i32>();
                out.push_num(v.unsigned_abs() as u32, 10, false, zero_pad, width, v < 0);
            }
            b'u' => {
                let v = args.arg::<u32>();
                out.push_num(v as u32, 10, false, zero_pad, width, false);
            }
            b'x' | b'X' => {
                let v = args.arg::<u32>();
                out.push_num(v as u32, 16, spec as u8 == b'X', zero_pad, width, false);
            }
            b's' => {
                let mut s = args.arg::<*const u16>();
                if !s.is_null() {
                    while *s != 0 {
                        out.push(*s);
                        s = s.add(1);
                    }
                }
            }
            b'c' => {
                let v = args.arg::<u32>();
                out.push(v as u16);
            }
            b'%' => out.push(b'%' as u16),
            other => {
                ntkrnl_sys::driver_ddk::DbgPrint(
                    b"[USBPORT] strsafe_vformat_w: unsupported format specifier '%%%c' - FIX THE FORMATTER\n\0"
                        .as_ptr() as *const i8,
                    other as u32,
                );
                *pszDest.add(out.len.min(out.cap_chars - 1)) = 0;
                return (out.len, STATUS_INVALID_PARAMETER_W);
            }
        }
    }
    *out.dest.add(out.len.min(out.cap_chars - 1)) = 0;
    if out.overflow {
        (out.len, STATUS_BUFFER_OVERFLOW_W)
    } else {
        (out.len, 0)
    }
}

/// Fixed-arity twin of the `RtlStringCbPrintfW` C-variadic entry point in
/// `drivers/usb/usbport/rust_varargs_shim.c`. The C side exists only to do the
/// `va_start` that stable Rust cannot (`c_variadic`, rust-lang#44930); `ap` is a
/// pointer to its `va_list`. All the logic lives here — see the module header of
/// `varargs-rust` for the contract.
#[no_mangle]
pub unsafe extern "C" fn RtlStringCbPrintfW_v(
    pszDest: *mut u16,
    cbDest: usize,
    pszFormat: *const u16,
    ap: *mut core::ffi::c_void,
) -> NTSTATUS {
    let mut args = VaArgs::from_ptr(ap);
    let (_len, status) = strsafe_vformat_w(pszDest, cbDest, pszFormat, &mut args);
    status
}

#[no_mangle]
pub unsafe extern "C" fn RtlStringCbCopyW(
    pszDest: *mut u16,
    cbDest: usize,
    pszSrc: *const u16,
) -> NTSTATUS {
    if pszDest.is_null() || pszSrc.is_null() || cbDest < core::mem::size_of::<u16>() {
        return STATUS_INVALID_PARAMETER_W;
    }
    let cap_chars = cbDest / core::mem::size_of::<u16>();
    let mut i = 0usize;
    while *pszSrc.add(i) != 0 {
        if i + 1 >= cap_chars {
            *pszDest.add(i) = 0;
            return STATUS_BUFFER_OVERFLOW_W;
        }
        *pszDest.add(i) = *pszSrc.add(i);
        i += 1;
    }
    *pszDest.add(i) = 0;
    0
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
// IRP_MN_QUERY_INTERFACE / IRP_MN_QUERY_CAPABILITIES now come from
// ntkrnl_sys::driver_ddk with the full PnP minor set (bind-all match-arm fix).

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
// RtlStringCbPrintfExW: ntstrsafe FORCEINLINE, no export. Real body — usbport builds the
// root-hub HardwareIDs multi-sz with it and dereferences the returned DestEnd, so the old
// no-op stub (which never set the out-params) was a latent NULL-offset crash.
/// Fixed-arity twin of the `RtlStringCbPrintfExW` C-variadic entry point — see
/// `RtlStringCbPrintfW_v` above.
#[no_mangle]
pub unsafe extern "C" fn RtlStringCbPrintfExW_v(
    pszDest: *mut u16,
    cbDest: usize,
    ppszDestEnd: *mut *mut u16,
    pcbRemaining: *mut usize,
    _dwFlags: u32,
    pszFormat: *const u16,
    ap: *mut core::ffi::c_void,
) -> i32 {
    let mut args = VaArgs::from_ptr(ap);
    let (len, status) = strsafe_vformat_w(pszDest, cbDest, pszFormat, &mut args);
    if status >= 0 {
        if !ppszDestEnd.is_null() {
            *ppszDestEnd = pszDest.add(len);
        }
        if !pcbRemaining.is_null() {
            *pcbRemaining = cbDest - len * core::mem::size_of::<u16>();
        }
    }
    status
}
