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
#[no_mangle] pub unsafe extern "C" fn ScsiGetTotalSenseByteCountIndicated(buf: PVOID) -> u8 {
    if buf.is_null() {0} else { (*( (buf as *const u8).add(7) )).wrapping_add(8) }
}
#[no_mangle] pub unsafe extern "C" fn ScsiGetSenseDescriptor(buf: PVOID) -> PVOID { if buf.is_null() {core::ptr::null_mut()} else {(buf as *mut u8).add(8) as PVOID} }
#[no_mangle] pub unsafe extern "C" fn ScsiGetSenseDescriptorLength(d: PVOID) -> u32 { if d.is_null() {0} else {(*((d as *const u8).add(1)) as u32).wrapping_add(2)} }
#[no_mangle] pub unsafe extern "C" fn ScsiGetNextSenseDescriptorByType(buf: PVOID, len: u32, ty: u8, found: *mut u8) -> PVOID { if !found.is_null() {*found=0;} core::ptr::null_mut() }
#[no_mangle] pub unsafe extern "C" fn ScsiValidateBlockCommandSenseDescriptor(d: PVOID, len: u32) -> u8 { 0 }
#[no_mangle] pub unsafe extern "C" fn ScsiValidateInformationSenseDescriptor(d: PVOID, len: u32) -> u8 { 0 }
#[no_mangle] pub unsafe extern "C" fn ScsiConvertToFixedSenseFormat(buf: PVOID, len: u32, out: PVOID, outlen: u32) -> u8 { 0 }
#[no_mangle] pub unsafe extern "system" fn ScsiGetSenseKeyAndCodes(buf: PVOID, len: u32, opts: u32, key: *mut u8, asc: *mut u8, ascq: *mut u8) -> u8 {
    if buf.is_null() { return 0; }
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
#[no_mangle] pub unsafe extern "C" fn Analysis_assume_(_e: u8) {}
#[no_mangle] pub unsafe extern "C" fn SRB_STATUS(st: u8) -> u8 { st & 0x3f }
#[no_mangle] pub unsafe extern "C" fn Int64ShrlMod32(v: u64, sh: u32) -> u64 { v >> sh }
#[no_mangle] pub unsafe extern "C" fn LongToPtr(v: i32) -> PVOID { v as isize as usize as PVOID }
#[no_mangle] pub unsafe extern "C" fn ULongToPtr(v: u32) -> PVOID { v as usize as PVOID }
#[no_mangle] pub unsafe extern "C" fn RtlEqualMemory(a: PVOID, b: PVOID, n: usize) -> u8 {
    (core::slice::from_raw_parts(a as *const u8, n) == core::slice::from_raw_parts(b as *const u8, n)) as u8
}
#[no_mangle] pub unsafe extern "C" fn IsEqualGUID(a: *const GUID, b: *const GUID) -> u8 {
    (core::slice::from_raw_parts(a as *const u8, 16) == core::slice::from_raw_parts(b as *const u8, 16)) as u8
}
#[no_mangle] pub unsafe extern "C" fn _Analysis_assume_(_e: u8) {}
#[no_mangle] pub unsafe extern "C" fn PORT_ALLOCATED_SENSE(fdo: PVOID, srb: PVOID) -> u8 { 0 }
#[no_mangle] pub unsafe extern "C" fn PORT_ALLOCATED_SENSE_EX(fdo: PVOID, srb: PVOID) -> u8 { 0 }
#[no_mangle] pub unsafe extern "C" fn FREE_PORT_ALLOCATED_SENSE_BUFFER(fdo: PVOID, srb: PVOID) {}
#[no_mangle] pub unsafe extern "C" fn FREE_PORT_ALLOCATED_SENSE_BUFFER_EX(fdo: PVOID, srb: PVOID) {}
#[no_mangle] pub unsafe extern "system" fn GET_FDO_EXTENSON_SENSE_DATA_LENGTH(fdo: PVOID) -> u8 { 18 }

// ============================ tracing / WPP / ETW (no-ops) ============================
#[no_mangle] pub unsafe extern "C" fn TracePrint(_lvl: u32, _flags: u32, _fmt: *const i8) {}
#[no_mangle] pub unsafe extern "C" fn StorDebugPrint(_lvl: u32, _fmt: *const i8) {}
#[no_mangle] pub unsafe extern "C" fn WPP_INIT_TRACING(_drv: PVOID, _reg: PVOID) {}
#[no_mangle] pub unsafe extern "C" fn WPP_CLEANUP(_drv: PVOID) {}
const EMPTY: *const i8 = b"\0".as_ptr() as *const i8;
#[no_mangle] pub unsafe extern "C" fn DbgGetScsiOpStr(_s: PVOID) -> *const i8 { EMPTY }
#[no_mangle] pub unsafe extern "C" fn DbgGetSenseCodeStr(_a: u8, _b: u8) -> *const i8 { EMPTY }
#[no_mangle] pub unsafe extern "C" fn DbgGetAdditionalSenseCodeStr(_a: u8, _b: u8) -> *const i8 { EMPTY }
#[no_mangle] pub unsafe extern "C" fn DbgGetSrbStatusStr(_s: u8) -> *const i8 { EMPTY }
#[no_mangle] pub unsafe extern "C" fn DBGGETSCSIOPSTR(_s: PVOID) -> *const i8 { EMPTY }
#[no_mangle] pub unsafe extern "C" fn DBGGETSENSECODESTR(_s: PVOID) -> *const i8 { EMPTY }
#[no_mangle] pub unsafe extern "C" fn DBGGETADSENSECODESTR(_s: PVOID) -> *const i8 { EMPTY }
#[no_mangle] pub unsafe extern "C" fn DBGGETADSENSEQUALIFIERSTR(_s: PVOID) -> *const i8 { EMPTY }
#[no_mangle] pub unsafe extern "C" fn DBGGETSRBSTATUSSTR(_s: PVOID) -> *const i8 { EMPTY }

// ============================ ETW (no-op) ============================
#[no_mangle] pub unsafe extern "C" fn EtwRegister(a: PVOID, b: PVOID, c: PVOID, h: *mut u64) -> NTSTATUS { if !h.is_null() {*h=0;} 0 }
#[no_mangle] pub unsafe extern "C" fn EtwUnregister(h: u64) -> NTSTATUS { 0 }
#[no_mangle] pub unsafe extern "C" fn EtwWrite(h: u64, d: PVOID, a: PVOID, n: u32, dd: PVOID) -> NTSTATUS { 0 }
#[no_mangle] pub unsafe extern "C" fn EventDescCreate() {}

// ============================ Ex timer / lookaside (stubs) ============================
#[no_mangle] pub unsafe extern "C" fn ExAllocateFromNPagedLookasideList(l: PVOID) -> PVOID { core::ptr::null_mut() }
#[no_mangle] pub unsafe extern "C" fn ExSetTimer(t: PVOID, due: i64, period: i64, p: PVOID) -> u8 { 0 }
#[no_mangle] pub unsafe extern "C" fn ExDeleteTimer(t: PVOID, cancel: u8, wait: u8, p: PVOID) -> u8 { 0 }
#[no_mangle] pub unsafe extern "C" fn ExInitializeSetTimerParameters(p: PVOID) {}
#[no_mangle] pub unsafe extern "C" fn ExInitializeDeleteTimerParameters(p: PVOID) {}

// ============================ Po power-setting (stubs) ============================
#[no_mangle] pub unsafe extern "C" fn PoRegisterPowerSettingCallback(devobj: PVOID, guid: PVOID, cb: PVOID, ctx: PVOID, h: *mut PVOID) -> NTSTATUS { if !h.is_null() {*h=core::ptr::null_mut();} 0 }
#[no_mangle] pub unsafe extern "C" fn PoUnregisterPowerSettingCallback(h: PVOID) -> NTSTATUS { 0 }

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

// ============================ Mm / Ke ============================
#[no_mangle] pub unsafe extern "C" fn MmGetMdlVirtualAddress(Mdl: PMDL) -> PVOID {
    ((*Mdl).StartVa as *mut u8).add((*Mdl).ByteOffset as usize) as PVOID
}
#[no_mangle] pub unsafe extern "C" fn MmPrepareMdlForReuse(Mdl: PMDL) {}
#[no_mangle] pub unsafe extern "C" fn KeGetCurrentNodeNumber() -> u16 { 0 }
#[no_mangle] pub unsafe extern "C" fn KeQueryHighestNodeNumber() -> u32 { 0 }
#[no_mangle] pub unsafe extern "C" fn KeSetCoalescableTimer(t: PVOID, due: LARGE_INTEGER, period: u32, tol: u32, dpc: PVOID) -> u8 { 0 }

// ============================ data symbols ============================
#[no_mangle] pub static mut SharedSystemTime: LARGE_INTEGER = unsafe { core::mem::zeroed() };
