//! usbstor glue — c2rust phantom-extern bridges + cohab shims not present in the
//! bindgen types.rs. Filled during integration/compile-fix (ramdisk/ksecdd pattern).
//! DbgPrint/RtlAssert resolve from the ntoskrnl import lib (modules declare them in their
//! own extern blocks). Below: re-export basic kernel types the translated modules name bare
//! but only import explicitly/partially from ddk, plus aliases for types absent from ddk.
#![allow(unused_imports)]

// C-variadic entry points live in drivers/usb/usbstor/rust_varargs_shim.c (stable Rust
// cannot DEFINE one — c_variadic, rust-lang#44930). `VaArgs` reads their va_list.
use varargs_rust::VaArgs;

pub use crate::types::*;

// Basic kernel scalar/pointer types — present in ntkrnl_sys::ddk but the translated modules
// reference them bare (PCSTR alone appears 47×) without globbing ddk. Re-export so the
// `use crate::glue::*` each module already has brings them into scope. NOTE: PCHAR/PUCHAR/
// PCWSTR are deliberately EXCLUDED — they already arrive via the modules' ntkrnl_sys::base/
// driver_ddk globs, so re-exporting them here would make the bare name ambiguous (E0659).
pub use ntkrnl_sys::ddk::{
    CCHAR, KPRIORITY, KPROCESSOR_MODE, PCSTR, PCWCH, PDRIVER_DISPATCH, PWCH, PWSTR,
    _UNICODE_STRING,
};

// size_t — C size type; bindgen emits it per-TU but it's absent from the curated ddk surface.
pub type size_t = usize;

// NTSTRSAFE_PWSTR / NTSTRSAFE_PCWSTR — ntstrsafe.h string aliases (FORCEINLINE header, not in
// the ddk bindgen). Faithful to <ntstrsafe.h>: wide mutable/const string pointers.
pub type NTSTRSAFE_PWSTR = ntkrnl_sys::ddk::PWSTR;
pub type NTSTRSAFE_PCWSTR = ntkrnl_sys::ddk::PCWSTR;

// SrbGetCdb — srbhelper.h FORCEINLINE helper (no exported symbol; C inlines it per-TU, so the
// Rust translation's extern call is undefined at cohab link). Faithful to <srbhelper.h>:
// for a classic SCSI_REQUEST_BLOCK it returns `(PCDB)&Srb->Cdb[0]`. usbstor declares the
// parameter as PSCSI_REQUEST_BLOCK and only ever builds classic Bulk-Only SRBs
// (SRB_FUNCTION_EXECUTE_SCSI / _CLAIM_DEVICE / …), never SRB_FUNCTION_STORAGE_REQUEST_BLOCK
// (0x28) — so the extended-SRB branch of srbhelper.h's SrbGetCdb is unreachable here. cdecl
// (`_SrbGetCdb`) to match usbstor's own `extern "C"` declaration. (Mirrors classpnp-rust's
// ddk_shims::SrbGetCdb classic path.)
#[no_mangle]
pub unsafe extern "C" fn SrbGetCdb(Srb: PSCSI_REQUEST_BLOCK) -> PCDB {
    core::ptr::addr_of_mut!((*Srb).Cdb) as PCDB
}

// ---- inline-only DDK helpers (no exported symbol; C inlines them per-TU). The translation
// emits extern calls, undefined at cohab link → provide faithful out-of-line shims. cdecl
// vs stdcall decoration matches how usbstor's modules declared each (see the undefined-symbol
// decorations: most cdecl `_Name`, ExAllocatePoolZero/BooleanFlagOn stdcall `_Name@N`). The
// IRP-stack/cancel/pending helpers wrap the canonical ntkrnl_sys::inline bodies (same as the
// sibling disk-rust/cdrom-rust ddk_shims). ----
use ntkrnl_sys::ddk::{
    PIRP, PIO_STACK_LOCATION, PDRIVER_CANCEL, PIO_COMPLETION_ROUTINE, POOL_TYPE,
    BOOLEAN, PVOID, ULONG, USHORT, UCHAR, PMDL, SIZE_T, CHAR, GUID, NTSTATUS,
};
// PURB / _URB_CONTROL_GET_STATUS_REQUEST come from crate::types (URB surface is crate-local).

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
// IoSetCompletionRoutine — sets the NEXT stack location's completion routine + invoke flags
// (wdm.h FORCEINLINE). SL_INVOKE_ON_{SUCCESS,ERROR,CANCEL} = 0x40/0x80/0x20.
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
// KeInitializeSpinLock — zeroes the lock word (ke.h FORCEINLINE).
#[no_mangle]
pub unsafe extern "C" fn KeInitializeSpinLock(SpinLock: *mut usize) {
    *SpinLock = 0;
}
// ExAllocatePoolZero — allocate-and-zero (newer wdm.h inline; not exported by this ntoskrnl).
// stdcall (`_ExAllocatePoolZero@12`) to match the translation's declaration.
#[no_mangle]
pub unsafe extern "system" fn ExAllocatePoolZero(
    PoolType: POOL_TYPE, NumberOfBytes: ntkrnl_sys::ddk::SIZE_T, Tag: ULONG,
) -> PVOID {
    ntkrnl_sys::inline::mm::ExAllocatePoolZero(PoolType, NumberOfBytes as _, Tag)
}
// BooleanFlagOn — `((Flags & SingleFlag) != 0)` (ntdef.h macro). stdcall (`_BooleanFlagOn@8`).
#[no_mangle]
pub unsafe extern "system" fn BooleanFlagOn(Flags: ULONG, SingleFlag: ULONG) -> BOOLEAN {
    if (Flags & SingleFlag) != 0 { 1 } else { 0 }
}
// RtlAssert is ntoskrnl's export: it prints the failed assertion and returns. An
// empty local definition here won the link and made every failed ASSERT in this
// driver vanish (scan_export_shadows).
extern "system" {
    pub fn RtlAssert(FailedAssertion: PVOID, FileName: PVOID, LineNumber: ULONG, Message: *mut ntkrnl_sys::ddk::CHAR);
}
// UsbBuildGetStatusRequest — usbdlib.h FORCEINLINE macro building a GET_STATUS URB. Faithful
// to the macro: set UrbHeader.Function/Length + the UrbControlGetStatusRequest fields.
#[no_mangle]
pub unsafe extern "C" fn UsbBuildGetStatusRequest(
    Urb: PURB, Op: USHORT, Index: USHORT, TransferBuffer: PVOID, TransferBufferMDL: PVOID, Link: PVOID,
) {
    let hdr = (*Urb).__bindgen_anon_1.UrbHeader.as_mut();
    (*hdr).Function = Op;
    (*hdr).Length = core::mem::size_of::<_URB_CONTROL_GET_STATUS_REQUEST>() as USHORT;
    let r = (*Urb).__bindgen_anon_1.UrbControlGetStatusRequest.as_mut();
    (*r).TransferBufferLength = core::mem::size_of::<USHORT>() as ULONG;
    (*r).TransferBufferMDL = TransferBufferMDL as ntkrnl_sys::ddk::PMDL;
    (*r).TransferBuffer = TransferBuffer;
    (*r).Index = Index;
    (*r).UrbLink = Link as PURB;
}
// USB_BUS_INTERFACE_USBDI_GUID — usbbusif.h DEFINE_GUID data symbol (referenced by the bus
// interface query). {b1a96a13-3de0-4574-9b01-c08feab318d6}.
#[no_mangle]
pub static USB_BUS_INTERFACE_USBDI_GUID: ntkrnl_sys::ddk::GUID = ntkrnl_sys::ddk::GUID {
    Data1: 0xb1a96a13,
    Data2: 0x3de0,
    Data3: 0x4574,
    Data4: [0x9b, 0x01, 0xc0, 0x8f, 0xea, 0xb3, 0x18, 0xd6],
};

// ---- amd64/arm64-only inline shims: i386 ntoskrnl/usbd EXPORT these (so the i386 cohab
// link resolves them from the import lib), but on amd64/arm64 they are header inlines with
// no exported symbol → provide them here. cfg-gated to NOT(x86) so the i386 link keeps using
// the real exports (avoids a duplicate-definition there). ----

// ExAllocatePoolUninitialized — the non-zeroing pool alloc (wdm.h inline → ExAllocatePoolWithTag).
// No arch exports this (wdm.h inline; nm over build-i386 confirms) — provide it everywhere.
// "system": ntkrnl-sys i386 declares it stdcall (@12 refs); identical to "C" on amd64/arm64.
#[no_mangle]
pub unsafe extern "system" fn ExAllocatePoolUninitialized(
    PoolType: POOL_TYPE, NumberOfBytes: SIZE_T, Tag: ULONG,
) -> PVOID {
    ntkrnl_sys::ddk::ExAllocatePoolWithTag(PoolType, NumberOfBytes, Tag)
}

// IoCopyCurrentIrpStackLocationToNext — copy the current stack location to the next, up to
// (excluding) CompletionRoutine, then clear the next Control (wdm.h FORCEINLINE). Mirrors
// cdrom-rust's cross-arch offset_of body (NOT a hardcoded offset). No arch exports this
// (wdm.h FORCEINLINE; nm over build-i386 confirms) — provide it everywhere, cdecl per the
// ntkrnl-sys i386 decl.
#[no_mangle]
pub unsafe extern "C" fn IoCopyCurrentIrpStackLocationToNext(Irp: PIRP) {
    let cur = IoGetCurrentIrpStackLocation(Irp);
    let next = IoGetNextIrpStackLocation(Irp);
    let n = core::mem::offset_of!(ntkrnl_sys::ddk::IO_STACK_LOCATION, CompletionRoutine);
    core::ptr::copy_nonoverlapping(cur as *const u8, next as *mut u8, n);
    (*next).Control = 0;
}

// Real ntstrsafe wide formatter — ported from usbport-rust/src/glue.rs 2026-07-06
// (was a null-terminate-only stub; same empty-string disease that broke usbhub's
// PDO IDs). Cch variant wraps the byte-count core.
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
                    b"[USBSTOR] strsafe_vformat_w: unsupported format specifier '%%%c' - FIX THE FORMATTER\n\0"
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
/// `drivers/usb/usbstor/rust_varargs_shim.c`. The C side exists only to do the
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


/// Fixed-arity twin of the `RtlStringCchPrintfW` C-variadic entry point — see
/// `RtlStringCbPrintfW_v` above. All arches: no import lib actually exports the entry
/// point (nm over build-i386 confirms — the old "i386 resolves it from the import lib"
/// note was wrong), so the C trampoline + this twin exist everywhere.
#[no_mangle]
pub unsafe extern "C" fn RtlStringCchPrintfW_v(
    Dst: *mut u16,
    CchDst: usize,
    Format: *const u16,
    ap: *mut core::ffi::c_void,
) -> NTSTATUS {
    let mut args = VaArgs::from_ptr(ap);
    let (_len, status) = strsafe_vformat_w(Dst, CchDst * core::mem::size_of::<u16>(), Format, &mut args);
    status
}

// KeGetCurrentIrql — amd64 reads CR8 (TPR). i386 exports it; arm64 didn't reference it here.
// CC = ntkrnl-sys bindgen_i386 decl class (callers import THAT decl; a mismatched
// shim def in the same crate folds every such call to ud2 — the 2026-07-18 i386
// driver-fleet bomb class).
#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "system" fn KeGetCurrentIrql() -> u8 {
    let irql: u64;
    core::arch::asm!("mov {}, cr8", out(reg) irql, options(nomem, nostack, preserves_flags));
    irql as u8
}

// UsbBuildGetDescriptorRequest — usbdlib.h FORCEINLINE macro building a GET_DESCRIPTOR URB.
// URB_FUNCTION_GET_DESCRIPTOR_FROM_DEVICE = 0x000B (usb.h). No arch exports it — provide
// everywhere.
#[no_mangle]
pub unsafe extern "C" fn UsbBuildGetDescriptorRequest(
    Urb: PURB, Length: USHORT, DescriptorType: UCHAR, Index: UCHAR, LanguageId: USHORT,
    TransferBuffer: PVOID, TransferBufferMDL: PVOID, TransferBufferLength: ULONG, UrbLink: PURB,
) {
    let hdr = (*Urb).__bindgen_anon_1.UrbHeader.as_mut();
    (*hdr).Function = 0x000B; // URB_FUNCTION_GET_DESCRIPTOR_FROM_DEVICE
    (*hdr).Length = Length;
    let r = (*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.as_mut();
    (*r).TransferBufferLength = TransferBufferLength;
    (*r).TransferBufferMDL = TransferBufferMDL as PMDL;
    (*r).TransferBuffer = TransferBuffer;
    (*r).DescriptorType = DescriptorType;
    (*r).Index = Index;
    (*r).LanguageId = LanguageId;
    (*r).UrbLink = UrbLink;
}
