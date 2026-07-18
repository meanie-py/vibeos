//! usbhub glue — c2rust phantom-extern bridges + cohab shims not in the bindgen types.rs.
//! Filled during integration/compile-fix (usbstor/usbccgp pattern). DbgPrint/RtlAssert resolve
//! from the ntoskrnl import lib (modules declare them in their own extern blocks).
#![allow(unused_imports)]

// C-variadic entry points live in drivers/usb/usbhub/rust_varargs_shim.c (stable Rust
// cannot DEFINE one — c_variadic, rust-lang#44930). `VaArgs` reads their va_list.
use varargs_rust::VaArgs;

pub use crate::types::*;

// Basic kernel types the modules name bare but only import partially from ddk.
// PCHAR/PUCHAR/PCWSTR EXCLUDED (arrive via base/driver_ddk globs → would be ambiguous).
pub use ntkrnl_sys::ddk::{
    CCHAR, KPRIORITY, KPROCESSOR_MODE, PCSTR, PCWCH, PDRIVER_DISPATCH, PWCH,
    _UNICODE_STRING, LANGID, GUID,
};
pub type size_t = usize;
pub type wchar_t = u16;
pub type NTSTRSAFE_PWSTR = ntkrnl_sys::ddk::PWSTR;
pub type NTSTRSAFE_PCWSTR = ntkrnl_sys::ddk::PCWSTR;

// High-frequency consts the translator inlined as bare idents across many functions.
// Verified against the C headers (values, not guessed):
pub const USB_HUB_TAG: u32 = 0x42554855; // 'BUHU' (usbhub.h:26)
pub const EVENT_INCREMENT: i8 = 1; // ntddk.h:597
pub const STATUS_MORE_PROCESSING_REQUIRED: ntkrnl_sys::ddk::NTSTATUS = 0xC0000016u32 as i32; // ntstatus.h
pub const IOCTL_INTERNAL_USB_SUBMIT_URB: u32 = 0x00220003; // CTL_CODE(FILE_DEVICE_USB,0,METHOD_NEITHER,0)

// ---- inline-only DDK helpers + NDEBUG debug-macro no-ops (no exported symbol; undefined at
// cohab link). Decoration matches the undefined symbols (cdecl `_Name` except InterlockedExchange
// stdcall `_Name@8`). IRP-stack helpers wrap ntkrnl_sys::inline (same as usbstor/usbccgp). ----
use ntkrnl_sys::ddk::{
    PIRP, PIO_STACK_LOCATION, PDRIVER_CANCEL, PIO_COMPLETION_ROUTINE, PVOID, LONG, NTSTATUS,
    IRP, IO_STACK_LOCATION,
};

#[no_mangle]
pub unsafe extern "C" fn IoGetCurrentIrpStackLocation(Irp: PIRP) -> PIO_STACK_LOCATION {
    ntkrnl_sys::inline::tier1::IoGetCurrentIrpStackLocation(Irp)
}
#[no_mangle]
pub unsafe extern "C" fn IoGetNextIrpStackLocation(Irp: PIRP) -> PIO_STACK_LOCATION {
    ntkrnl_sys::inline::tier1::IoGetNextIrpStackLocation(Irp)
}
#[no_mangle]
pub unsafe extern "C" fn IoSkipCurrentIrpStackLocation(Irp: PIRP) {
    ntkrnl_sys::inline::tier1::IoSkipCurrentIrpStackLocation(Irp)
}
#[no_mangle]
pub unsafe extern "C" fn IoMarkIrpPending(Irp: PIRP) {
    ntkrnl_sys::inline::tier1::IoMarkIrpPending(Irp)
}
#[no_mangle]
pub unsafe extern "C" fn IoSetCancelRoutine(Irp: PIRP, CancelRoutine: PDRIVER_CANCEL) -> PDRIVER_CANCEL {
    ntkrnl_sys::inline::tier1::IoSetCancelRoutine(Irp, CancelRoutine)
}
#[no_mangle]
pub unsafe extern "C" fn IoSetCompletionRoutine(
    Irp: PIRP, CompletionRoutine: PIO_COMPLETION_ROUTINE, Context: PVOID,
    InvokeOnSuccess: u8, InvokeOnError: u8, InvokeOnCancel: u8,
) {
    let stk = IoGetNextIrpStackLocation(Irp);
    (*stk).CompletionRoutine = CompletionRoutine;
    (*stk).Context = Context;
    let mut ctl: u8 = 0;
    if InvokeOnSuccess != 0 { ctl |= 0x40; }
    if InvokeOnError != 0 { ctl |= 0x80; }
    if InvokeOnCancel != 0 { ctl |= 0x20; }
    (*stk).Control = ctl;
}
// IoCopyCurrentIrpStackLocationToNext — copy current→next stack up to (excl) CompletionRoutine,
// clear next Control (wdm.h FORCEINLINE).
#[no_mangle]
pub unsafe extern "C" fn IoCopyCurrentIrpStackLocationToNext(Irp: PIRP) {
    let cur = IoGetCurrentIrpStackLocation(Irp);
    let next = IoGetNextIrpStackLocation(Irp);
    let n = core::mem::offset_of!(IO_STACK_LOCATION, CompletionRoutine);
    core::ptr::copy_nonoverlapping(cur as *const u8, next as *mut u8, n);
    (*next).Control = 0;
}
// IoSizeOfIrp — wdm.h FORCEINLINE: sizeof(IRP) + StackSize * sizeof(IO_STACK_LOCATION).
#[no_mangle]
pub unsafe extern "C" fn IoSizeOfIrp(StackSize: i8) -> u16 {
    (core::mem::size_of::<IRP>() + (StackSize as usize) * core::mem::size_of::<IO_STACK_LOCATION>()) as u16
}
// CC = ntkrnl-sys bindgen_i386 decl class (callers import THAT decl; a mismatched
// shim def in the same crate folds every such call to ud2 — the 2026-07-18 i386
// driver-fleet bomb class).
#[no_mangle]
pub unsafe extern "C" fn KeInitializeSpinLock(SpinLock: *mut usize) {
    *SpinLock = 0;
}
// InterlockedExchange — atomic swap (intrinsic on i386, no ntoskrnl export). stdcall `@8`.
#[no_mangle]
pub unsafe extern "system" fn InterlockedExchange(Target: *mut LONG, Value: LONG) -> LONG {
    (*(Target as *const core::sync::atomic::AtomicI32)).swap(Value, core::sync::atomic::Ordering::SeqCst)
}
// RtlStringCbPrintfW — ntstrsafe.h FORCEINLINE wide formatted print; variadic formatting is not
// reconstructible from the fixed extern decl. cdecl (caller cleans the stack, so this fixed-arity
// def safely satisfies the variadic call site). Null-terminates the dest. NOTE: known fidelity
// gap — owed a faithful ntstrsafe port (mirrors cdrom-rust's RtlStringCchPrintfW shim).
// DPRINT_PNP / DPRINT_SCE — NDEBUG-disabled DbgPrint wrappers (dbg_uhub.h); no symbol in retail.
// cdecl no-ops (fixed arity is fine: cdecl caller cleans the varargs).
#[no_mangle]
pub unsafe extern "C" fn DPRINT_PNP(_fmt: *const i8) {}
#[no_mangle]
pub unsafe extern "C" fn DPRINT_SCE(_fmt: *const i8) {}
// InterlockedIncrement/Decrement — atomic +1/-1 returning the NEW value (intrinsics, no export).
// stdcall `@4`.
#[no_mangle]
unsafe extern "system" fn InterlockedIncrement(Addend: *mut core::ffi::c_long) -> core::ffi::c_long {
    (*(Addend as *const core::sync::atomic::AtomicI32)).fetch_add(1, core::sync::atomic::Ordering::SeqCst) + 1
}
#[no_mangle]
unsafe extern "system" fn InterlockedDecrement(Addend: *mut core::ffi::c_long) -> core::ffi::c_long {
    (*(Addend as *const core::sync::atomic::AtomicI32)).fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1
}
// URB_FROM_IRP — usbdlib.h macro: (PURB)IoGetCurrentIrpStackLocation(Irp)->Parameters.Others.Argument1.
// stdcall `@4`.
#[no_mangle]
pub unsafe extern "system" fn URB_FROM_IRP(Irp: PIRP) -> PURB {
    let stk = IoGetCurrentIrpStackLocation(Irp);
    (*stk).Parameters.Others.Argument1 as PURB
}
// RtlStringCbPrintfExW — ntstrsafe.h FORCEINLINE; variadic formatting not reconstructible from
// the fixed extern decl. cdecl no-op (null-terminates dest). NOTE: fidelity gap, like
// RtlStringCbPrintfW above — owed a faithful ntstrsafe port.
// USB_BUS_INTERFACE_{HUB,USBDI}_GUID — usbbusif.h/hubbusif.h DEFINE_GUID data symbols. bindgen
// emitted extern decls into types.rs (both headers allowlisted) → these provide the BACKING
// definitions only. PRIVATE (no `pub`) so they don't enter the glob and clash (E0659).
#[no_mangle]
static USB_BUS_INTERFACE_HUB_GUID: GUID = GUID {
    Data1: 0xb2bb8c0a, Data2: 0x5ab4, Data3: 0x11d3,
    Data4: [0xa8, 0xcd, 0x00, 0xc0, 0x4f, 0x68, 0x74, 0x7a],
};
#[no_mangle]
static USB_BUS_INTERFACE_USBDI_GUID: GUID = GUID {
    Data1: 0xb1a96a13, Data2: 0x3de0, Data3: 0x4574,
    Data4: [0x9b, 0x01, 0xc0, 0x8f, 0xea, 0xb3, 0x18, 0xd6],
};

// Real ntstrsafe wide formatter — ported from usbport-rust/src/glue.rs 2026-07-06.
// The previous no-op stubs here returned success with an EMPTY string, so the
// PDO DeviceID/HardwareIDs/InstanceID all formatted to "" -> PiInitializeDevNode
// failed CreateDeviceInstancePath with STATUS_PLUGPLAY_NO_DEVICE and the usb-kbd/
// usb-mouse devnodes never came up. Same disease, same cure as usbport.
// TODO: factor into a shared strsafe rlib instead of per-crate copies.
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

    unsafe fn push_num(&mut self, val: u64, base: u64, upper: bool, zero_pad: bool, width: usize, negative: bool) {
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
                out.push_num(v.unsigned_abs() as u64, 10, false, zero_pad, width, v < 0);
            }
            b'u' => {
                let v = args.arg::<u32>();
                out.push_num(v as u64, 10, false, zero_pad, width, false);
            }
            b'x' | b'X' => {
                let v = args.arg::<u32>();
                out.push_num(v as u64, 16, spec as u8 == b'X', zero_pad, width, false);
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
                    b"[USBHUB] strsafe_vformat_w: unsupported format specifier '%%%c' - FIX THE FORMATTER\n\0"
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
/// `drivers/usb/usbhub/rust_varargs_shim.c`. The C side exists only to do the
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
