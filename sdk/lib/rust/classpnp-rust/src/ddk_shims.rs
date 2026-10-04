//! ddk_shims.rs — implementations for the DDK macros / FORCEINLINEs / storport+scsi
//! inline helpers that the classpnp C calls as if they were exported functions. These
//! have no kernel export symbol (they're header inlines/macros), so the cohab link needs
//! the crate to provide them. Hot-path helpers (IRP stack, SRB accessors) are faithful;
//! tracing/ETW are no-ops (telemetry, not boot-critical). Per-symbol ABI matches the C
//! caller decoration (i386 NTAPI = stdcall = `extern "system"`; plain inlines = `extern "C"`).
#![allow(non_snake_case, unused_variables, dead_code)]
use ntkrnl_sys::ddk::*;
use crate::types::{PCDB, PSTORAGE_REQUEST_BLOCK, _STORAGE_REQUEST_BLOCK, SRB_TYPE_STORAGE_REQUEST_BLOCK};
use core::ffi::c_void;

// ============================ IRP stack inlines ============================
#[no_mangle] pub unsafe extern "C" fn IoGetCurrentIrpStackLocation(Irp: PIRP) -> PIO_STACK_LOCATION {
    (*Irp).Tail.Overlay.__bindgen_anon_2.__bindgen_anon_1.CurrentStackLocation
}
#[no_mangle] pub unsafe extern "C" fn IoGetNextIrpStackLocation(Irp: PIRP) -> PIO_STACK_LOCATION {
    (*Irp).Tail.Overlay.__bindgen_anon_2.__bindgen_anon_1.CurrentStackLocation.offset(-1)
}
#[no_mangle] pub unsafe extern "C" fn IoSkipCurrentIrpStackLocation(Irp: PIRP) {
    let p = &mut (*Irp).Tail.Overlay.__bindgen_anon_2.__bindgen_anon_1.CurrentStackLocation;
    *p = (*p).offset(1);
}
#[no_mangle] pub unsafe extern "C" fn IoSetNextIrpStackLocation(Irp: PIRP) {
    let p = &mut (*Irp).Tail.Overlay.__bindgen_anon_2.__bindgen_anon_1.CurrentStackLocation;
    *p = (*p).offset(-1);
    (*Irp).CurrentLocation = ((*Irp).CurrentLocation as i8 - 1) as i8;
}
#[no_mangle] pub unsafe extern "C" fn IoCopyCurrentIrpStackLocationToNext(Irp: PIRP) {
    let cur = IoGetCurrentIrpStackLocation(Irp);
    let next = IoGetNextIrpStackLocation(Irp);
    core::ptr::copy_nonoverlapping(cur as *const u8, next as *mut u8, 0x24);
    (*next).Control = 0;
}
#[no_mangle] pub unsafe extern "C" fn IoMarkIrpPending(Irp: PIRP) {
    (*IoGetCurrentIrpStackLocation(Irp)).Control |= 0x01; // SL_PENDING_RETURNED
}
#[no_mangle] pub unsafe extern "C" fn IoSetCompletionRoutine(
    Irp: PIRP, CompletionRoutine: PIO_COMPLETION_ROUTINE, Context: PVOID,
    Success: u8, Error: u8, Cancel: u8,
) {
    let stk = IoGetNextIrpStackLocation(Irp);
    (*stk).CompletionRoutine = CompletionRoutine;
    (*stk).Context = Context;
    let mut ctl: u8 = 0;
    if Success != 0 { ctl |= 0x40; }  // SL_INVOKE_ON_SUCCESS
    if Error   != 0 { ctl |= 0x80; }  // SL_INVOKE_ON_ERROR
    if Cancel  != 0 { ctl |= 0x20; }  // SL_INVOKE_ON_CANCEL
    (*stk).Control = ctl;
}
#[no_mangle] pub unsafe extern "C" fn IoIsErrorUserInduced(Status: NTSTATUS) -> u8 {
    matches!(Status as u32,
        0xC000009A|0xC0000010|0xC0000013|0xC0000022|0xC000000E|0xC0000056|0xC0000288|0xC000020C) as u8
}

// ============================ SRB accessors (storport) ============================
#[inline] unsafe fn srb_is_ex(srb: PSTORAGE_REQUEST_BLOCK) -> bool {
    (*srb).Function as u32 == 0x28 /* SRB_FUNCTION_STORAGE_REQUEST_BLOCK */
}
#[no_mangle] pub unsafe extern "system" fn SrbGetCdb(Srb: PVOID) -> PCDB {
    let srb = Srb as PSTORAGE_REQUEST_BLOCK;
    if srb_is_ex(srb) {
        crate::srblib::SrbGetScsiData(srb, core::ptr::null_mut(), core::ptr::null_mut(),
            core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut()) as PCDB
    } else {
        core::ptr::addr_of_mut!((*(Srb as *mut crate::types::_SCSI_REQUEST_BLOCK)).Cdb) as PCDB
    }
}
#[no_mangle] pub unsafe extern "C" fn SrbGetScsiStatus(Srb: PVOID) -> u8 {
    let srb = Srb as PSTORAGE_REQUEST_BLOCK;
    if srb_is_ex(srb) { 0 } else { (*(Srb as *mut crate::types::_SCSI_REQUEST_BLOCK)).ScsiStatus }
}
#[no_mangle] pub unsafe extern "C" fn SrbGetCdbLength(Srb: PVOID) -> u8 {
    let srb = Srb as PSTORAGE_REQUEST_BLOCK;
    if srb_is_ex(srb) { 16 } else { (*(Srb as *mut crate::types::_SCSI_REQUEST_BLOCK)).CdbLength }
}
#[no_mangle] pub unsafe extern "C" fn SrbGetRequestAttribute(Srb: PVOID) -> u8 {
    let srb = Srb as PSTORAGE_REQUEST_BLOCK; if srb_is_ex(srb) { ((*srb).RequestAttribute) as u8 } else { (*(Srb as *mut crate::types::_SCSI_REQUEST_BLOCK)).QueueAction }
}
#[no_mangle] pub unsafe extern "system" fn SrbSetDataBuffer(Srb: PVOID, Buf: PVOID) {
    let srb = Srb as PSTORAGE_REQUEST_BLOCK;
    if srb_is_ex(srb) { (*srb).DataBuffer = Buf; } else { (*(Srb as *mut crate::types::_SCSI_REQUEST_BLOCK)).DataBuffer = Buf; }
}
#[no_mangle] pub unsafe extern "system" fn SrbSetDataTransferLength(Srb: PVOID, Len: u32) {
    let srb = Srb as PSTORAGE_REQUEST_BLOCK;
    if srb_is_ex(srb) { (*srb).DataTransferLength = Len; } else { (*(Srb as *mut crate::types::_SCSI_REQUEST_BLOCK)).DataTransferLength = Len; }
}
#[no_mangle] pub unsafe extern "C" fn SrbSetSrbFlags(Srb: PVOID, Flags: u32) {
    let srb = Srb as PSTORAGE_REQUEST_BLOCK;
    if srb_is_ex(srb) { (*srb).SrbFlags |= Flags; } else { (*(Srb as *mut crate::types::_SCSI_REQUEST_BLOCK)).SrbFlags |= Flags; }
}
#[no_mangle] pub unsafe extern "C" fn SrbClearSrbFlags(Srb: PVOID, Flags: u32) {
    let srb = Srb as PSTORAGE_REQUEST_BLOCK;
    if srb_is_ex(srb) { (*srb).SrbFlags &= !Flags; } else { (*(Srb as *mut crate::types::_SCSI_REQUEST_BLOCK)).SrbFlags &= !Flags; }
}
#[no_mangle] pub unsafe extern "system" fn SrbAssignSrbFlags(Srb: PVOID, Flags: u32) {
    let srb = Srb as PSTORAGE_REQUEST_BLOCK;
    if srb_is_ex(srb) { (*srb).SrbFlags = Flags; } else { (*(Srb as *mut crate::types::_SCSI_REQUEST_BLOCK)).SrbFlags = Flags; }
}
#[no_mangle] pub unsafe extern "system" fn SrbSetCdbLength(Srb: PVOID, Len: u8) {
    let srb = Srb as PSTORAGE_REQUEST_BLOCK;
    if !srb_is_ex(srb) { (*(Srb as *mut crate::types::_SCSI_REQUEST_BLOCK)).CdbLength = Len; }
}
#[no_mangle] pub unsafe extern "system" fn SrbSetRequestAttribute(Srb: PVOID, Attr: u8) {
    let srb = Srb as PSTORAGE_REQUEST_BLOCK;
    if srb_is_ex(srb) { (*srb).RequestAttribute = (Attr) as u16; } else { (*(Srb as *mut crate::types::_SCSI_REQUEST_BLOCK)).QueueAction = Attr; }
}
#[no_mangle] pub unsafe extern "C" fn SrbSetRequestTag(Srb: PVOID, Tag: u32) {
    let srb = Srb as PSTORAGE_REQUEST_BLOCK;
    if srb_is_ex(srb) { (*srb).RequestTag = Tag; } else { (*(Srb as *mut crate::types::_SCSI_REQUEST_BLOCK)).QueueTag = Tag as u8; }
}
#[no_mangle] pub unsafe extern "system" fn SrbSetTimeOutValue(Srb: PVOID, T: u32) {
    let srb = Srb as PSTORAGE_REQUEST_BLOCK;
    if srb_is_ex(srb) { (*srb).TimeOutValue = T; } else { (*(Srb as *mut crate::types::_SCSI_REQUEST_BLOCK)).TimeOutValue = T; }
}
#[no_mangle] pub unsafe extern "system" fn SrbSetOriginalRequest(Srb: PVOID, Irp: PVOID) {
    let srb = Srb as PSTORAGE_REQUEST_BLOCK;
    if srb_is_ex(srb) { (*srb).OriginalRequest = Irp; } else { (*(Srb as *mut crate::types::_SCSI_REQUEST_BLOCK)).OriginalRequest = Irp; }
}
#[no_mangle] pub unsafe extern "system" fn SrbSetSenseInfoBuffer(Srb: PVOID, Buf: PVOID) {
    let srb = Srb as PSTORAGE_REQUEST_BLOCK;
    if !srb_is_ex(srb) { (*(Srb as *mut crate::types::_SCSI_REQUEST_BLOCK)).SenseInfoBuffer = Buf; }
}
#[no_mangle] pub unsafe extern "system" fn SrbSetSenseInfoBufferLength(Srb: PVOID, Len: u8) {
    let srb = Srb as PSTORAGE_REQUEST_BLOCK;
    if !srb_is_ex(srb) { (*(Srb as *mut crate::types::_SCSI_REQUEST_BLOCK)).SenseInfoBufferLength = Len; }
}
#[no_mangle] pub unsafe extern "C" fn SrbSetNextSrb(Srb: PVOID, Next: PVOID) {
    let srb = Srb as *mut crate::types::_SCSI_REQUEST_BLOCK;
    if (*(Srb as PSTORAGE_REQUEST_BLOCK)).Function as u32 != 0x28 { (*srb).NextSrb = Next as *mut crate::types::_SCSI_REQUEST_BLOCK; }
}

// ============================ SCSI sense inlines (minimal) ============================
#[no_mangle] pub unsafe extern "C" fn IsDescriptorSenseDataFormat(buf: PVOID) -> u8 {
    if buf.is_null() { return 0; } (((*(buf as *const u8)) & 0x7f) == 0x72 || ((*(buf as *const u8)) & 0x7f) == 0x73) as u8
}
#[no_mangle] pub unsafe extern "C" fn IsSenseDataFormatValueValid(buf: PVOID) -> u8 {
    if buf.is_null() { return 0; } let c = (*(buf as *const u8)) & 0x7f; (c==0x70||c==0x71||c==0x72||c==0x73) as u8
}
#[no_mangle] pub unsafe extern "C" fn ScsiGetSenseErrorCode(buf: PVOID) -> u8 { if buf.is_null() {0} else {(*(buf as *const u8)) & 0x7f} }
// ---- descriptor-format sense accessors -------------------------------------------------
// These three took FEWER parameters than every declaration and call site in the crate, so
// each argument bound to the wrong slot. `ScsiGetNextSenseDescriptorByType` was the live
// hazard: its 4th parameter was `found: *mut u8` while callers pass `typeList.len() as
// ULONG` (= 2) there, making `*found = 0` a write to ADDRESS 0x2 — a bugcheck at DISPATCH
// on any device that answers with descriptor-format sense. It has not fired only because
// `IsDescriptorSenseDataFormat()` gates the whole block and our virtual devices return
// fixed-format sense. Found by tools/scan_extern_arity.py.
//
// Signatures now match the declarations exactly. The behaviour is deliberately the
// conservative one — "no descriptor found", FALSE — rather than the old truthy return:
// callers that got TRUE went on to read out-parameters this stub never wrote.
#[no_mangle] pub unsafe extern "C" fn ScsiGetTotalSenseByteCountIndicated(
    buf: PVOID,
    _buf_len: UCHAR,
    count: *mut UCHAR,
) -> BOOLEAN {
    if buf.is_null() || count.is_null() {
        return 0;
    }
    *count = (*((buf as *const u8).add(7))).wrapping_add(8);
    1
}
#[no_mangle] pub unsafe extern "C" fn ScsiGetSenseDescriptor(
    buf: PVOID,
    _buf_len: UCHAR,
    desc_buf: *mut PVOID,
    desc_len: *mut UCHAR,
) -> BOOLEAN {
    if buf.is_null() || desc_buf.is_null() || desc_len.is_null() {
        return 0;
    }
    // Descriptor list starts at byte 8 of a descriptor-format sense buffer; byte 7 is the
    // additional-sense-length that bounds it.
    *desc_buf = (buf as *mut u8).add(8) as PVOID;
    *desc_len = *((buf as *const u8).add(7));
    1
}
#[no_mangle] pub unsafe extern "C" fn ScsiGetSenseDescriptorLength(d: PVOID) -> u32 { if d.is_null() {0} else {(*((d as *const u8).add(1)) as u32).wrapping_add(2)} }
#[no_mangle] pub unsafe extern "C" fn ScsiGetNextSenseDescriptorByType(
    _start: PVOID,
    _start_len: UCHAR,
    _types: *const UCHAR,
    _type_count: ULONG,
    out_type: *mut UCHAR,
    out_buf: *mut PVOID,
    out_len: *mut UCHAR,
) -> BOOLEAN {
    // Not implemented: report "no descriptor of that type", and leave the out-parameters
    // in a defined state so the caller's loop terminates instead of walking a stale buffer.
    if !out_type.is_null() {
        *out_type = 0;
    }
    if !out_buf.is_null() {
        *out_buf = core::ptr::null_mut();
    }
    if !out_len.is_null() {
        *out_len = 0;
    }
    0
}
#[no_mangle] pub unsafe extern "C" fn ScsiValidateBlockCommandSenseDescriptor(d: PVOID, len: UCHAR) -> u8 { 0 }
#[no_mangle] pub unsafe extern "C" fn ScsiValidateInformationSenseDescriptor(d: PVOID, len: UCHAR) -> u8 { 0 }
#[no_mangle] pub unsafe extern "C" fn ScsiConvertToFixedSenseFormat(buf: PVOID, len: UCHAR, out: PVOID, outlen: UCHAR) -> u8 { 0 }
#[no_mangle] pub unsafe extern "system" fn ScsiGetSenseKeyAndCodes(buf: PVOID, len: UCHAR, opts: ULONG, key: *mut u8, asc: *mut u8, ascq: *mut u8) -> u8 {
    if buf.is_null() || len == 0 { return 0; }
    let b = buf as *const u8; let fmt = (*b) & 0x7f;
    if fmt == 0x70 || fmt == 0x71 {
        if !key.is_null() { *key = (*b.add(2)) & 0x0f; }
        if !asc.is_null() { *asc = *b.add(12); }
        if !ascq.is_null() { *ascq = *b.add(13); }
    } else {
        if !key.is_null() { *key = (*b.add(1)) & 0x0f; }
        if !asc.is_null() { *asc = *b.add(2); }
        if !ascq.is_null() { *ascq = *b.add(3); }
    }
    1
}

// ============================ macros-as-functions ============================
#[no_mangle] pub unsafe extern "C" fn FREE_POOL(Ptr: PVOID) { if !Ptr.is_null() { ExFreePool(Ptr); } }
#[no_mangle] pub unsafe extern "C" fn TEST_FLAG(Flags: u32, Bit: u32) -> u8 { ((Flags & Bit) != 0) as u8 }
#[no_mangle] pub unsafe extern "system" fn NT_ASSERT(_cond: u8) {}
#[no_mangle] pub unsafe extern "system" fn PAGED_CODE() {}
#[no_mangle] pub unsafe extern "system" fn ASSERT_PDO(_pdo: PVOID) {}
#[no_mangle] pub unsafe extern "C" fn SRB_STATUS(st: u8) -> u8 { st & 0x3f }
#[no_mangle] pub unsafe extern "C" fn Int64ShrlMod32(v: u64, sh: u32) -> u64 { v >> sh }
#[no_mangle] pub unsafe extern "C" fn LongToPtr(v: i32) -> PVOID { v as isize as usize as PVOID }
#[no_mangle] pub unsafe extern "C" fn ULongToPtr(v: u32) -> PVOID { v as usize as PVOID }
#[no_mangle] pub unsafe extern "C" fn RtlEqualMemory(a: PVOID, b: PVOID, n: usize) -> u8 {
    (core::slice::from_raw_parts(a as *const u8, n) == core::slice::from_raw_parts(b as *const u8, n)) as u8
}
#[no_mangle] pub unsafe extern "system" fn IsEqualGUID(a: *const GUID, b: *const GUID) -> u8 {
    (core::slice::from_raw_parts(a as *const u8, 16) == core::slice::from_raw_parts(b as *const u8, 16)) as u8
}
// _Analysis_assume_ (→ i386 `__Analysis_assume_`) lives in rust_cohab_stub.c — the i386
// no_mangle underscore decoration for a leading-underscore name is unreliable; C is exact.
#[no_mangle] pub unsafe extern "C" fn PORT_ALLOCATED_SENSE(fdo: PVOID, srb: PVOID) -> u8 { 0 }
#[no_mangle] pub unsafe extern "C" fn PORT_ALLOCATED_SENSE_EX(fdo: PVOID, srb: PVOID) -> u8 { 0 }
#[no_mangle] pub unsafe extern "C" fn FREE_PORT_ALLOCATED_SENSE_BUFFER(fdo: PVOID, srb: PVOID) {}
#[no_mangle] pub unsafe extern "C" fn FREE_PORT_ALLOCATED_SENSE_BUFFER_EX(fdo: PVOID, srb: PVOID) {}
#[no_mangle] pub unsafe extern "system" fn GET_FDO_EXTENSON_SENSE_DATA_LENGTH(fdo: PVOID) -> u8 { 18 }

// ============================ tracing / WPP / ETW (no-ops) ============================
// TracePrint / StorDebugPrint MOVED to the drvtrace-shim crate: the variadic
// call sites + non-variadic defs are UB to LLVM inside one module (instcombine
// folds the calls to ud2 — the 2026-07-18 classpnp bomb set); a separate crate
// under lto=false is a separate LLVM module, which makes the pattern fold-proof.
#[no_mangle] pub unsafe extern "C" fn WPP_INIT_TRACING(_drv: PVOID, _reg: PVOID) {}
#[no_mangle] pub unsafe extern "C" fn WPP_CLEANUP(_drv: PVOID) {}
const EMPTY: *const i8 = b"\0".as_ptr() as *const i8;
#[no_mangle] pub unsafe extern "C" fn DbgGetScsiOpStr(_s: PVOID) -> *const i8 { EMPTY }
// C debug.h: #define DBGGETIOCTLSTR(_ioctl) DbgGetIoctlStr(_ioctl) — string-table
// lookup for trace text only; empty string is faithful to the other Dbg* stubs here.
#[no_mangle] pub unsafe extern "C" fn DBGGETIOCTLSTR(_code: u32) -> *const i8 { EMPTY }
// Both are declared (debug.rs) and called with ONE argument — the SRB pointer — not two
// bytes. Harmless while the body ignores its parameters, but the arity was still a
// caller/callee disagreement; scan_extern_arity.py flagged it.
#[no_mangle] pub unsafe extern "C" fn DbgGetSenseCodeStr(_srb: PVOID) -> *const i8 { EMPTY }
#[no_mangle] pub unsafe extern "C" fn DbgGetAdditionalSenseCodeStr(_srb: PVOID) -> *const i8 { EMPTY }
#[no_mangle] pub unsafe extern "C" fn DbgGetSrbStatusStr(_s: PVOID) -> *const i8 { EMPTY }
#[no_mangle] pub unsafe extern "C" fn DBGGETSCSIOPSTR(_s: PVOID) -> *const i8 { EMPTY }
#[no_mangle] pub unsafe extern "C" fn DBGGETSENSECODESTR(_s: PVOID) -> *const i8 { EMPTY }
#[no_mangle] pub unsafe extern "C" fn DBGGETADSENSECODESTR(_s: PVOID) -> *const i8 { EMPTY }
#[no_mangle] pub unsafe extern "C" fn DBGGETADSENSEQUALIFIERSTR(_s: PVOID) -> *const i8 { EMPTY }
#[no_mangle] pub unsafe extern "C" fn DBGGETSRBSTATUSSTR(_s: PVOID) -> *const i8 { EMPTY }

// ============================ ETW (no-op) ============================
#[no_mangle] pub unsafe extern "system" fn EtwRegister(a: PVOID, b: PVOID, c: PVOID, h: *mut u64) -> NTSTATUS { if !h.is_null() {*h=0;} 0 }
#[no_mangle] pub unsafe extern "system" fn EtwUnregister(h: u64) -> NTSTATUS { 0 }
#[no_mangle] pub unsafe extern "system" fn EtwWrite(h: u64, d: PVOID, a: PVOID, n: u32, dd: PVOID) -> NTSTATUS { 0 }
// Declared and called with 8 arguments (autorun.rs:61, :3303). The 0-parameter form was
// benign — cdecl, and the body is empty — but the shape still disagreed with every caller.
// Behaviour is unchanged: the ETW write this feeds is stubbed too.
#[no_mangle] pub unsafe extern "C" fn EventDescCreate(
    _EventDescriptor: PVOID,
    _Id: u16,
    _Version: u8,
    _Channel: u8,
    _Level: u8,
    _Task: u16,
    _Opcode: u8,
    _Keyword: u64,
) {
}

// ============================ Ex timer / lookaside (stubs) ============================
#[no_mangle] pub unsafe extern "C" fn ExAllocateFromNPagedLookasideList(l: PVOID) -> PVOID { core::ptr::null_mut() }
#[no_mangle] pub unsafe extern "C" fn ExSetTimer(t: PVOID, due: i64, period: i64, p: PVOID) -> u8 { 0 }
#[no_mangle] pub unsafe extern "C" fn ExDeleteTimer(t: PVOID, cancel: u8, wait: u8, p: PVOID) -> u8 { 0 }
#[no_mangle] pub unsafe extern "C" fn ExInitializeSetTimerParameters(p: PVOID) {}
#[no_mangle] pub unsafe extern "C" fn ExInitializeDeleteTimerParameters(p: PVOID) {}

// ============================ Po power-setting (stubs) ============================
#[no_mangle] pub unsafe extern "system" fn PoRegisterPowerSettingCallback(devobj: PVOID, guid: PVOID, cb: PVOID, ctx: PVOID, h: *mut PVOID) -> NTSTATUS { if !h.is_null() {*h=core::ptr::null_mut();} 0 }
#[no_mangle] pub unsafe extern "system" fn PoUnregisterPowerSettingCallback(h: PVOID) -> NTSTATUS { 0 }

// ============================ Simple SLIST ============================
#[no_mangle] pub unsafe extern "C" fn SimpleInitSlistHdr(h: *mut PVOID) { *h = core::ptr::null_mut(); }
#[no_mangle] pub unsafe extern "C" fn SimpleIsSlistEmpty(h: *mut PVOID) -> u8 { (*h).is_null() as u8 }
#[no_mangle] pub unsafe extern "C" fn SimplePushSlist(h: *mut PVOID, e: *mut PVOID) { *e = *h; *h = e as PVOID; }
#[no_mangle] pub unsafe extern "C" fn SimplePopSlist(h: *mut PVOID) -> PVOID {
    let e = *h; if !e.is_null() { *h = *(e as *mut PVOID); } e
}

// ============================ Rtl helpers ============================
#[no_mangle] pub unsafe extern "system" fn RtlInterlockedPushEntrySList(head: PVOID, entry: PVOID) -> PVOID { core::ptr::null_mut() }
#[no_mangle] pub unsafe extern "system" fn RtlInterlockedPopEntrySList(head: PVOID) -> PVOID { core::ptr::null_mut() }
#[no_mangle] pub unsafe extern "C" fn RtlStringCbPrintfW(dst: *mut u16, cb: usize, fmt: *const u16) -> NTSTATUS { if cb>=2 { *dst = 0; } 0 }
#[no_mangle] pub unsafe extern "C" fn RtlStringCchPrintfA(dst: *mut i8, cch: usize, fmt: *const i8) -> NTSTATUS { if cch>=1 { *dst = 0; } 0 }
#[no_mangle] pub unsafe extern "C" fn RtlULongAdd(a: u32, b: u32, out: *mut u32) -> NTSTATUS {
    match a.checked_add(b) { Some(v) => { *out = v; 0 } None => { *out = 0; 0xC0000095u32 as i32 } }
}
#[no_mangle] pub unsafe extern "C" fn RtlULongLongSub(a: u64, b: u64, out: *mut u64) -> NTSTATUS {
    match a.checked_sub(b) { Some(v) => { *out = v; 0 } None => { *out = 0; 0xC0000095u32 as i32 } }
}

// ============================ resurrected-PnP-arm deps ============================
// These three became live when the missing IRP_MN_* constants were added to
// ntkrnl_sys::driver_ddk (the PnP dispatch match arms referencing them had been
// silent catch-all bindings — dead code — so their deps never surfaced at link).

/// IoAdjustPagingPathCount — wdm.h macro (InterlockedIncrement/Decrement on Count).
#[no_mangle] pub unsafe extern "C" fn IoAdjustPagingPathCount(Count: *mut i32, Increment: u8) {
    let a = &*(Count as *const core::sync::atomic::AtomicI32);
    if Increment != 0 {
        a.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
    } else {
        a.fetch_sub(1, core::sync::atomic::Ordering::SeqCst);
    }
}

/// NT_ASSERTMSG — assertion with message; fre-build semantics (report, don't halt).
#[no_mangle] pub unsafe extern "C" fn NT_ASSERTMSG(Msg: *const i8, Expr: u8) {
    if Expr == 0 {
        ntkrnl_sys::driver_ddk::DbgPrint(
            b"[CLASSPNP] NT_ASSERTMSG failed: %s\n\0".as_ptr() as *const i8,
            Msg,
        );
    }
}

/// ClassQueryPnpCapabilities — faithful translation of classpnp class.c: route the
/// QUERY_CAPABILITIES request to the class driver's Fdo/Pdo query routine, if any.
#[no_mangle] pub unsafe extern "C" fn ClassQueryPnpCapabilities(
    DeviceObject: PDEVICE_OBJECT,
    Capabilities: PDEVICE_CAPABILITIES,
) -> i32 {
    use crate::types::{ClassGetDriverExtension, PCOMMON_DEVICE_EXTENSION};
    const STATUS_NOT_IMPLEMENTED: i32 = 0xC0000002u32 as i32;

    let driverExtension = ClassGetDriverExtension((*DeviceObject).DriverObject);
    let commonExtension = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;

    let queryRoutine = if (*commonExtension).__bindgen_anon_1.IsFdo() != 0 {
        (*driverExtension).InitData.FdoData.ClassQueryPnpCapabilities
    } else {
        (*driverExtension).InitData.PdoData.ClassQueryPnpCapabilities
    };

    match queryRoutine {
        Some(f) => f(DeviceObject, Capabilities),
        None => STATUS_NOT_IMPLEMENTED,
    }
}

// ============================ Mm / Ke ============================
#[no_mangle] pub unsafe extern "C" fn MmGetMdlVirtualAddress(Mdl: PMDL) -> PVOID {
    ((*Mdl).StartVa as *mut u8).add((*Mdl).ByteOffset as usize) as PVOID
}
#[no_mangle] pub unsafe extern "C" fn MmPrepareMdlForReuse(Mdl: PMDL) {}
// KeGetCurrentNodeNumber / KeQueryHighestNodeNumber / KeSetCoalescableTimer and
// IoQueueWorkItemEx come from ntoskrnl-vista-rust, which classpnp-cargo links on every
// arch (as C classpnp links libntoskrnl_vista.a). Copies here were duplicate definitions
// that --allow-multiple-definition resolved silently; this crate's KeSetCoalescableTimer
// was a `0` stub that never armed the timer. scan_duplicate_symbols gates it now.

// ============================ classp.h FORCEINLINE helpers ============================
// These are header inlines the C calls as functions; cohab needs the crate to provide
// them. Referenced cdecl (`extern "C"`) by the translated callers.
use crate::types::{PTRANSFER_PACKET, PCLASS_PRIVATE_FDO_DATA};

#[no_mangle] pub unsafe extern "C" fn ClasspTransferPacketGetCdb(Pkt: PTRANSFER_PACKET) -> PCDB {
    SrbGetCdb((*Pkt).Srb as PVOID)
}
#[no_mangle] pub unsafe extern "C" fn ClasspTransferPacketGetSenseInfoBuffer(Pkt: PTRANSFER_PACKET) -> PVOID {
    crate::srblib::SrbGetSenseInfoBuffer((*Pkt).Srb as PVOID)
}
#[no_mangle] pub unsafe extern "C" fn ClasspTransferPacketGetSenseInfoBufferLength(Pkt: PTRANSFER_PACKET) -> u8 {
    crate::srblib::SrbGetSenseInfoBufferLength((*Pkt).Srb as PVOID)
}
// currentTime = KeQueryPerformanceCounter(NULL)  (ReactOS path)
#[no_mangle] pub unsafe extern "C" fn ClasspGetCurrentTime() -> LARGE_INTEGER {
    KeQueryPerformanceCounter(core::ptr::null_mut())
}
// TimeDiff /= (10 * 1000)  — 100ns units → ms
#[no_mangle] pub unsafe extern "C" fn ClasspTimeDiffToMs(TimeDiff: u64) -> u64 { TimeDiff / 10_000 }
// Irp->Tail.Overlay.DriverContext[1] = Idle  (ULONG_PTR slot)
#[no_mangle] pub unsafe extern "C" fn ClasspMarkIrpAsIdle(Irp: PIRP, Idle: u8) {
    (*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[1] = Idle as usize as PVOID;
}
#[no_mangle] pub unsafe extern "C" fn ClasspIsIdleRequest(Irp: PIRP) -> u8 {
    ((*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[1] as usize & 0xff) as u8
}
#[no_mangle] pub unsafe extern "C" fn ClasspIsIdleRequestSupported(FdoData: PCLASS_PRIVATE_FDO_DATA, _Irp: PIRP) -> u8 {
    (*FdoData).IdlePrioritySupported
}
// if ex: ((PSTORAGE_REQUEST_BLOCK)Srb)->MiniportContext = Irp; else ((PSCSI_REQUEST_BLOCK)Srb)->SrbExtension = Irp
#[no_mangle] pub unsafe extern "C" fn ClasspSrbSetOriginalIrp(Srb: PVOID, Irp: PVOID) {
    let srb = Srb as PSTORAGE_REQUEST_BLOCK;
    if srb_is_ex(srb) { (*srb).MiniportContext = Irp; }
    else { (*(Srb as *mut crate::types::_SCSI_REQUEST_BLOCK)).SrbExtension = Irp; }
}
// (((_k) & 0xFFFFFE00) == READ_COPY_NUMBER_KEY(0x52434e00))
#[no_mangle] pub unsafe extern "C" fn IsKeyReadCopyNumber(Key: u32) -> u8 {
    ((Key & 0xFFFFFE00) == 0x52434e00) as u8
}
// ClassAcquireRemoveLock(devobj, tag) => ClassAcquireRemoveLockEx(devobj, tag, __FILE__, __LINE__)
#[no_mangle] pub unsafe extern "C" fn ClassAcquireRemoveLock(DeviceObject: PDEVICE_OBJECT, Tag: PVOID) -> u32 {
    crate::residual_part_b::ClassAcquireRemoveLockEx(DeviceObject, Tag, b"classpnp\0".as_ptr() as *const i8, 0)
}

// ============================ Interlocked intrinsics (i386 has no kernel export) ============================
use core::sync::atomic::{AtomicI32, Ordering};
// Interlocked* are FASTCALL on i386 (per bindgen/ntoskrnl.spec — decls in class.rs
// match); plain C on other arches. One body each, cfg'd per convention.
#[cfg(target_arch = "x86")]
#[no_mangle] pub unsafe extern "fastcall" fn InterlockedCompareExchange(Dest: *mut i32, Exch: i32, Comp: i32) -> i32 {
    let a = &*(Dest as *const AtomicI32);
    match a.compare_exchange(Comp, Exch, Ordering::SeqCst, Ordering::SeqCst) { Ok(v) => v, Err(v) => v }
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle] pub unsafe extern "C" fn InterlockedCompareExchange(Dest: *mut i32, Exch: i32, Comp: i32) -> i32 {
    let a = &*(Dest as *const AtomicI32);
    match a.compare_exchange(Comp, Exch, Ordering::SeqCst, Ordering::SeqCst) { Ok(v) => v, Err(v) => v }
}
#[cfg(target_arch = "x86")]
#[no_mangle] pub unsafe extern "fastcall" fn InterlockedDecrement(Addend: *mut i32) -> i32 {
    (&*(Addend as *const AtomicI32)).fetch_sub(1, Ordering::SeqCst) - 1
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle] pub unsafe extern "C" fn InterlockedDecrement(Addend: *mut i32) -> i32 {
    (&*(Addend as *const AtomicI32)).fetch_sub(1, Ordering::SeqCst) - 1
}
#[cfg(target_arch = "x86")]
#[no_mangle] pub unsafe extern "fastcall" fn InterlockedExchange(Target: *mut i32, Value: i32) -> i32 {
    (&*(Target as *const AtomicI32)).swap(Value, Ordering::SeqCst)
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle] pub unsafe extern "C" fn InterlockedExchange(Target: *mut i32, Value: i32) -> i32 {
    (&*(Target as *const AtomicI32)).swap(Value, Ordering::SeqCst)
}
// KeInitializeSpinLock(lock) => *lock = 0
// CC = ntkrnl-sys bindgen_i386 decl class ("C") — callers import that decl; a
// mismatched shim def in the same crate folds every call to ud2 (this one took
// out ClasspInitializeIdleTimer & friends, 2026-07-18 i386 bomb set).
#[no_mangle] pub unsafe extern "C" fn KeInitializeSpinLock(SpinLock: *mut usize) { *SpinLock = 0; }

// ============================ real-export cdecl wrappers ============================
// The translated callers declare these `extern "C"` (cdecl `_X`), but the kernel exports
// them stdcall (`_X@N`). Provide cdecl shims that forward to the real stdcall export
// (resolved via the ntoskrnl/hal importlibs on the cohab link line).
//
// A `#[no_mangle]` definition of a name the importlib ALSO offers is not a forwarder: the
// local strong definition wins, so `extern "system" { fn X }` written inside `fn X` binds
// to X itself. LLVM tail-folds that to `jmp .`, and these three shipped as literal 2-byte
// infinite loops — `4fb90 <KeRaiseIrql>: eb fe  jmp 4fb90` in classpnp.sys on amd64. Any
// classpnp path that touched IRQL spun the CPU forever: no bugcheck, no serial output.
// So define a symbol ONLY on the arch that has none, and stay silent where the export exists.
//   i386  — hal.spec exports the stdcall pair, but the callers want cdecl and the primitive
//           is FASTCALL KfRaiseIrql; the tier1 inline does that decoration.
//   amd64 — NEITHER is linkable. sdk/include/xdk/amd64/ke.h:102 makes KeRaiseIrql a macro over
//           KfRaiseIrql, so nothing exports it at all; KeLowerIrql is exported but
//           `-private` (ntoskrnl.spec:635 -> KxLowerIrql), and spec2def.c:750 keeps private
//           exports OUT of the import lib — so a driver cannot bind it either. Both get a
//           real CR8 body here. (Reading the .spec is not enough: check for `-private`.)
//   arm64 — hal.spec:84,86 export both stdcall, non-private. Define neither.
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle] pub unsafe extern "system" fn KeRaiseIrql(NewIrql: u8, OldIrql: *mut u8) {
    #[cfg(target_arch = "x86")]
    { ntkrnl_sys::inline::tier1::KeRaiseIrql(NewIrql, OldIrql); }
    #[cfg(target_arch = "x86_64")]
    { *OldIrql = ntkrnl_sys::inline::ke::KfRaiseIrql_amd64(NewIrql); }
}
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle] pub unsafe extern "system" fn KeLowerIrql(NewIrql: u8) {
    #[cfg(target_arch = "x86")]
    { ntkrnl_sys::inline::tier1::KeLowerIrql(NewIrql); }
    #[cfg(target_arch = "x86_64")]
    { ntkrnl_sys::inline::ke::KeLowerIrql_amd64(NewIrql); }
}
// IoWMIDeviceObjectToProviderId: ntoskrnl.spec:518 exports it on amd64/arm64 only, so those
// arches must IMPORT it (classwmi.rs:940 already declares it) rather than shadow it with a
// self-call. i386 has no export and this is a telemetry path, not boot-critical — stub to 0.
#[cfg(target_arch = "x86")]
#[no_mangle] pub unsafe extern "C" fn IoWMIDeviceObjectToProviderId(Dev: PDEVICE_OBJECT) -> u32 {
    let _ = Dev;
    0
}
// SET_FLAG(Flags, Flag) macro: *Flags |= Flag. Genuinely undefined (a C macro) — the i386 fold
// hid the caller; amd64's live ref surfaced it. write_volatile to keep it captures(none)-safe.
#[no_mangle] pub unsafe extern "C" fn SET_FLAG(Flags: *mut u32, Flag: u32) {
    core::ptr::write_volatile(Flags, core::ptr::read_volatile(Flags) | Flag);
}

// KeGetCurrentIrql / KeQueryTickCount are real exports on i386 (and KeQueryTickCount on arm) but
// are inlines on amd64 — provide them there. cr8 holds the current IRQL on amd64.
#[cfg(target_arch = "x86_64")]
#[no_mangle] pub unsafe extern "system" fn KeGetCurrentIrql() -> u8 {
    let irql: u64;
    core::arch::asm!("mov {}, cr8", out(reg) irql, options(nomem, nostack, preserves_flags));
    irql as u8
}
// arm64: KeGetCurrentIrql resolves via the ntkrnl_sys inline, which references KeGetCurrentPrcb.
// That symbol isn't exported on arm64 — forward to the register-reading inline (distinct IR name).
#[cfg(target_arch = "aarch64")]
#[no_mangle] pub unsafe extern "C" fn KeGetCurrentPrcb() -> ntkrnl_sys::ddk::PKPRCB {
    ntkrnl_sys::inline::ke::KeGetCurrentPrcb()
}
// KeQueryTickCount: exported on i386/ARM32 but an inline on amd64/arm64. KI_USER_SHARED_DATA is at
// the SAME VA (0xFFFFF78000000000) on both; TickCount (KSYSTEM_TIME) at +0x320. Lockless read.
#[cfg(not(target_arch = "x86"))]
#[no_mangle] pub unsafe extern "system" fn KeQueryTickCount(Count: *mut LARGE_INTEGER) {
    // (ULONG LowPart @0x320, LONG High1Time @0x324, LONG High2Time @0x328). Lockless read.
    const LOW: usize = 0xFFFFF78000000000 + 0x320;
    loop {
        let high1 = core::ptr::read_volatile((LOW + 4) as *const i32);
        let low = core::ptr::read_volatile(LOW as *const u32);
        let high2 = core::ptr::read_volatile((LOW + 8) as *const i32);
        if high1 == high2 {
            (*Count).QuadPart = ((high1 as i64) << 32) | (low as i64 & 0xFFFF_FFFF);
            return;
        }
    }
}

// ============================ data symbols ============================
#[no_mangle] pub static mut SharedSystemTime: LARGE_INTEGER = unsafe { core::mem::zeroed() };

// SharedUserData — C macro for the fixed-VA KUSER_SHARED_DATA, not a real export. ServiceTransferRequest
// reads it for timestamps. On i386 the C-macro symbol is provided elsewhere (and the ref was fold-hidden);
// on amd64 provide the fixed kernel VA (0xFFFFF78000000000). arm64 handled at its own link.
#[cfg(not(target_arch = "x86"))]
#[no_mangle] pub static mut SharedUserData: *mut ntkrnl_sys::ddk::_KUSER_SHARED_DATA =
    0xFFFFF78000000000usize as *mut ntkrnl_sys::ddk::_KUSER_SHARED_DATA;

/// utils.c ClasspDeviceCopyOffloadProperty — StorageDeviceCopyOffloadProperty
/// query handler. This was a hand-written link-gap stub that answered every
/// standard query STATUS_NOT_SUPPORTED, claiming that was what the C gives a
/// device without ODX. It is not: the C answers a header-sized buffer with
/// Version/Size and success, a short buffer with STATUS_BUFFER_TOO_SMALL, a
/// device without the ROD-limits VPD page with STATUS_DEVICE_FEATURE_NOT_SUPPORTED,
/// and a device with it with the cached descriptor. Now the C body.
#[no_mangle]
pub unsafe extern "C" fn ClasspDeviceCopyOffloadProperty(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    _Srb: *mut crate::types::_SCSI_REQUEST_BLOCK,
) -> NTSTATUS {
    unsafe extern "system" {
        fn ClassReleaseRemoveLock(DeviceObject: PDEVICE_OBJECT, Tag: PIRP);
        fn ClassCompleteRequest(DeviceObject: PDEVICE_OBJECT, Irp: PIRP, Boost: i8);
    }
    const STATUS_SUCCESS: NTSTATUS = 0;
    const STATUS_NOT_SUPPORTED: NTSTATUS = 0xC00000BBu32 as NTSTATUS;
    const STATUS_BUFFER_TOO_SMALL: NTSTATUS = 0xC0000023u32 as NTSTATUS;
    const STATUS_INVALID_LEVEL: NTSTATUS = 0xC0000148u32 as NTSTATUS;
    const STATUS_DEVICE_FEATURE_NOT_SUPPORTED: NTSTATUS = 0xC0000463u32 as NTSTATUS;
    const DISPATCH_LEVEL: u8 = 2;

    let fdoExtension = (*DeviceObject).DeviceExtension as crate::types::PFUNCTIONAL_DEVICE_EXTENSION;
    let query = (*Irp).AssociatedIrp.SystemBuffer as *mut _STORAGE_PROPERTY_QUERY;
    let copyOffloadDescr = (*Irp).AssociatedIrp.SystemBuffer as PDEVICE_COPY_OFFLOAD_DESCRIPTOR;
    let irpStack = IoGetCurrentIrpStackLocation(Irp);
    let mut information: usize = 0;

    let status: NTSTATUS = 'exit: {
        // Check proper query type. PropertyExistsQuery always succeeds, for
        // consistency with how the other properties are handled.
        if (*query).QueryType == _STORAGE_QUERY_TYPE_PropertyExistsQuery {
            break 'exit STATUS_SUCCESS;
        } else if (*query).QueryType != _STORAGE_QUERY_TYPE_PropertyStandardQuery {
            break 'exit STATUS_NOT_SUPPORTED;
        }

        // Request validation (InputBufferLength and IsFdo were validated by
        // the caller).
        if KeGetCurrentIrql() >= DISPATCH_LEVEL {
            break 'exit STATUS_INVALID_LEVEL;
        }

        let length = (*irpStack).Parameters.DeviceIoControl.OutputBufferLength;

        if (length as usize) < core::mem::size_of::<DEVICE_COPY_OFFLOAD_DESCRIPTOR>() {
            if (length as usize) >= core::mem::size_of::<STORAGE_DESCRIPTOR_HEADER>() {
                information = core::mem::size_of::<STORAGE_DESCRIPTOR_HEADER>();
                (*copyOffloadDescr).Version = core::mem::size_of::<DEVICE_COPY_OFFLOAD_DESCRIPTOR>() as ULONG;
                (*copyOffloadDescr).Size = core::mem::size_of::<DEVICE_COPY_OFFLOAD_DESCRIPTOR>() as ULONG;
                break 'exit STATUS_SUCCESS;
            }
            break 'exit STATUS_BUFFER_TOO_SMALL;
        }

        let fsi = (*fdoExtension).FunctionSupportInfo;
        if (*fsi).ValidInquiryPages.BlockDeviceRODLimits() == 0 {
            break 'exit STATUS_DEVICE_FEATURE_NOT_SUPPORTED;
        }

        let rod = &(*fsi).BlockDeviceRODLimitsData;
        if rod.CommandStatus < 0 {
            break 'exit rod.CommandStatus;
        }

        // Fill in the output buffer from the Block Limits / Block Device ROD
        // Limits data cached when the device was first initialized.
        core::ptr::write_bytes(copyOffloadDescr as *mut u8, 0, length as usize);
        (*copyOffloadDescr).Version = 1;
        (*copyOffloadDescr).Size = core::mem::size_of::<DEVICE_COPY_OFFLOAD_DESCRIPTOR>() as ULONG;
        (*copyOffloadDescr).MaximumTokenLifetime = rod.MaximumInactivityTimer;
        (*copyOffloadDescr).DefaultTokenLifetime = rod.DefaultInactivityTimer;
        (*copyOffloadDescr).MaximumTransferSize = rod.MaximumTokenTransferSize;
        (*copyOffloadDescr).OptimalTransferCount = rod.OptimalTransferCount;
        (*copyOffloadDescr).MaximumDataDescriptors = rod.MaximumRangeDescriptors as ULONG;

        let b0 = &(*fsi).BlockLimitsData;
        if b0.CommandStatus >= 0 {
            (*copyOffloadDescr).MaximumTransferLengthPerDescriptor = b0.MaximumTransferLength;
            (*copyOffloadDescr).OptimalTransferLengthPerDescriptor = b0.OptimalTransferLength;
            (*copyOffloadDescr).OptimalTransferLengthGranularity = b0.OptimalTransferLengthGranularity;
        }

        information = core::mem::size_of::<DEVICE_COPY_OFFLOAD_DESCRIPTOR>();
        STATUS_SUCCESS
    };

    // Set the size and status in IRP
    (*Irp).IoStatus.Information = information as _;
    (*Irp).IoStatus.__bindgen_anon_1.Status = status;
    ClassReleaseRemoveLock(DeviceObject, Irp);
    ClassCompleteRequest(DeviceObject, Irp, 0 /* IO_NO_INCREMENT */);
    status
}
