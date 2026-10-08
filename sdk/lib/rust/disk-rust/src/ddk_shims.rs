//! ddk_shims.rs — cohab substrate: kernel inlines that are real exports on some
//! arches but inline (no symbol) on others. Mirrors classpnp-rust/src/ddk_shims.rs.
#![allow(non_snake_case)]

// KeGetCurrentIrql is a real export on i386 but an inline on amd64 (cr8 holds the IRQL).
#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "system" fn KeGetCurrentIrql() -> u8 {
    let irql: u64;
    core::arch::asm!("mov {}, cr8", out(reg) irql, options(nomem, nostack, preserves_flags));
    irql as u8
}

// arm64: KeGetCurrentIrql resolves via the ntkrnl_sys inline (references KeGetCurrentPrcb,
// not exported on arm64) — forward to the register-reading inline (distinct IR name).
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn KeGetCurrentPrcb() -> ntkrnl_sys::ddk::PKPRCB {
    ntkrnl_sys::inline::ke::KeGetCurrentPrcb()
}

// IoGetCurrentIrpStackLocation is a FORCEINLINE in the DDK (no exported symbol) — provide
// it backed by the ntkrnl_sys inline (distinct IR name, so no self-collision).
#[no_mangle]
pub unsafe extern "C" fn IoGetCurrentIrpStackLocation(
    irp: ntkrnl_sys::ddk::PIRP,
) -> ntkrnl_sys::ddk::PIO_STACK_LOCATION {
    ntkrnl_sys::inline::tier1::IoGetCurrentIrpStackLocation(irp)
}

// StorDebugPrint is a storage-tracing macro (storswtr.h) with no real symbol — no-op stub.
// Callers invoke it variadic; a non-variadic cdecl stub is ABI-compatible (caller cleans up).
// StorDebugPrint MOVED to drvtrace-shim.

// __int2c — DbgRaiseAssertionFailure intrinsic. The x86 definition lives in geometry.rs
// (cfg-gated x86-only, empty stub); arm64 needs its own symbol: brk #0xF001 is the
// Windows-on-ARM64 assertion break.
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn __int2c() {
    core::arch::asm!("brk #0xF001");
}

// ====================================================================================
// DDK FORCEINLINEs / macros the translation calls as extern functions (mirrors
// classpnp-rust/src/ddk_shims.rs — same substrate, disk.sys's own copies since
// classpnp.sys only exports its Class* .spec surface).
// ====================================================================================
use ntkrnl_sys::ddk::{PIRP, PIO_STACK_LOCATION, PIO_COMPLETION_ROUTINE, PVOID, LIST_ENTRY,
    PDRIVER_OBJECT, PUNICODE_STRING};
use core::ffi::c_void;

// ---------- IRP stack inlines ----------
#[no_mangle]
pub unsafe extern "C" fn IoGetNextIrpStackLocation(Irp: PIRP) -> PIO_STACK_LOCATION {
    (*Irp).Tail.Overlay.__bindgen_anon_2.__bindgen_anon_1.CurrentStackLocation.offset(-1)
}
#[no_mangle]
pub unsafe extern "C" fn IoSetNextIrpStackLocation(Irp: PIRP) {
    let p = &mut (*Irp).Tail.Overlay.__bindgen_anon_2.__bindgen_anon_1.CurrentStackLocation;
    *p = (*p).offset(-1);
    (*Irp).CurrentLocation = ((*Irp).CurrentLocation as i8 - 1) as i8;
}
#[no_mangle]
pub unsafe extern "C" fn IoMarkIrpPending(Irp: PIRP) {
    let stk = (*Irp).Tail.Overlay.__bindgen_anon_2.__bindgen_anon_1.CurrentStackLocation;
    (*stk).Control |= 0x01; // SL_PENDING_RETURNED
}
#[no_mangle]
pub unsafe extern "C" fn IoSetCompletionRoutine(
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

// ---------- LIST_ENTRY inlines ----------
#[no_mangle]
pub unsafe extern "C" fn IsListEmpty(ListHead: *mut LIST_ENTRY) -> u8 {
    ((*ListHead).Flink == ListHead) as u8
}
#[no_mangle]
pub unsafe extern "C" fn InsertTailList(ListHead: *mut LIST_ENTRY, Entry: *mut LIST_ENTRY) {
    let blink = (*ListHead).Blink;
    (*Entry).Flink = ListHead;
    (*Entry).Blink = blink;
    (*blink).Flink = Entry;
    (*ListHead).Blink = Entry;
}
#[no_mangle]
pub unsafe extern "C" fn RemoveHeadList(ListHead: *mut LIST_ENTRY) -> *mut LIST_ENTRY {
    let entry = (*ListHead).Flink;
    let flink = (*entry).Flink;
    (*ListHead).Flink = flink;
    (*flink).Blink = ListHead;
    entry
}

// ---------- Ke inlines ----------
// CC = ntkrnl-sys bindgen_i386 decl class (callers import THAT decl; a mismatched
// shim def in the same crate folds every such call to ud2 — the 2026-07-18 i386
// driver-fleet bomb class).
#[no_mangle]
pub unsafe extern "C" fn KeInitializeSpinLock(SpinLock: *mut usize) { *SpinLock = 0; }
// KeFlushIoBuffers is a MACRO, per arch: empty on x86/amd64 (cache-coherent DMA,
// xdk/x86/ke.h:188) and `_DataSynchronizationBarrier()` on arm64 (xdk/arm64/ke.h:106).
// That barrier IS the whole arm64 implementation -- there is no cache maintenance to
// add -- so this does exactly what the header does. QEMU virt's DMA is coherent, which
// is why its absence never showed. The `-> i32` is the callers' declaration (diskwmi.rs).
#[no_mangle]
pub unsafe extern "C" fn KeFlushIoBuffers(_mdl: *mut c_void, _read: i32, _dma: i32) -> i32 {
    // Not `nomem`: the barrier must also order the compiler's own memory accesses.
    #[cfg(target_arch = "aarch64")]
    core::arch::asm!("dsb sy", options(nostack, preserves_flags));
    0
}

// ---------- compiler intrinsics ----------
use core::sync::atomic::{AtomicI32, Ordering};
#[no_mangle]
pub unsafe extern "system" fn _InterlockedExchange(Target: *mut i32, Value: i32) -> i32 {
    (&*(Target as *const AtomicI32)).swap(Value, Ordering::SeqCst)
}

// ---------- macros-as-functions ----------
// scsi.h REVERSE_BYTES(Dst, Src): reverse 4 bytes (Dst[0]=Src[3]…Dst[3]=Src[0]).
// A C macro the translator externalized; disk had no def → i386 undefined symbol.
#[no_mangle] pub unsafe extern "C" fn REVERSE_BYTES(dst: *mut u8, src: *const u8) {
    *dst = *src.add(3); *dst.add(1) = *src.add(2);
    *dst.add(2) = *src.add(1); *dst.add(3) = *src;
}
#[no_mangle] pub unsafe extern "C" fn CHECK_IRQL() {}
#[no_mangle] pub unsafe extern "C" fn NT_ASSERT(_expression: i32) {}
#[no_mangle] pub unsafe extern "C" fn FREE_POOL(Ptr: *mut c_void) {
    if !Ptr.is_null() { ntkrnl_sys::ddk::ExFreePool(Ptr); }
}
#[no_mangle] pub unsafe extern "C" fn TEST_FLAG(value: u8, flag: u8) -> bool { (value & flag) != 0 }
#[no_mangle] pub unsafe extern "C" fn SET_FLAG(var: *mut u32, flag: u32) {
    core::ptr::write_volatile(var, core::ptr::read_volatile(var) | flag);
}
#[no_mangle] pub unsafe extern "C" fn CLEAR_FLAG(var: *mut u32, flag: u32) {
    core::ptr::write_volatile(var, core::ptr::read_volatile(var) & !flag);
}
#[no_mangle] pub unsafe extern "C" fn max(a: usize, b: usize) -> usize { if a > b { a } else { b } }
// disk.h macro: valid SMART self-test subcommands (ntdddisk.h values 0/1/2)
#[no_mangle] pub unsafe extern "C" fn DiskIsValidSmartSelfTest(Subcommand: u8) -> u8 {
    matches!(Subcommand, 0 | 1 | 2) as u8 // OFFLINE_ROUTINE | SHORT_SELFTEST | EXTENDED_SELFTEST
}

// ---------- arm64: geometry.rs is cfg'd x86-only (matches geometry.c's #if _X86_||_AMD64_).
// disk.h provides macro fallbacks on other arches — mirror them as functions:
//   DiskReadDriveCapacity(Fdo)            → ClassReadDriveCapacity(Fdo)
//   DiskGetDetectInfo(FdoExt, DetectInfo) → STATUS_UNSUCCESSFUL
//   DiskDriverReinitialization            → never registered/called off-x86 (no-op)
#[cfg(target_arch = "aarch64")]
extern "system" {
    fn ClassReadDriveCapacity(Fdo: ntkrnl_sys::ddk::PDEVICE_OBJECT) -> i32;
}
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn DiskReadDriveCapacity(Fdo: ntkrnl_sys::ddk::PDEVICE_OBJECT) -> i32 {
    ClassReadDriveCapacity(Fdo)
}
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn DiskGetDetectInfo(_fdo_ext: *mut c_void, _detect_info: *mut c_void) -> i32 {
    0xC0000001u32 as i32 // STATUS_UNSUCCESSFUL
}
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn DiskDriverReinitialization(_drv: *mut c_void, _nothing: PVOID, _count: u32) {}

// ---------- tracing / WPP (callers are variadic cdecl — caller cleans up) ----------
// Error/warning levels forward the raw format string to debug serial so disk.sys
// failure paths aren't silent; %-specifiers print literally (identifies the site).
// TracePrint MOVED to drvtrace-shim (fold-proof separate LLVM module — see that crate).
#[no_mangle] pub unsafe extern "C" fn WPP_INIT_TRACING(_drv: PDRIVER_OBJECT, _reg: PUNICODE_STRING) {}

// ---------- SRB_EX accessors (srbhelper.h inlines; logic mirrors classpnp srblib) ----------
use crate::types::{PSTORAGE_REQUEST_BLOCK, STORAGE_REQUEST_BLOCK, PSRBEX_DATA,
    PSRBEX_DATA_SCSI_CDB16, PSRBEX_DATA_SCSI_CDB32, PSRBEX_DATA_SCSI_CDB_VAR,
    SRBEX_DATA_SCSI_CDB16, SRBEX_DATA_SCSI_CDB32, SRBEX_DATA_SCSI_CDB_VAR, CDB,
    SRB_FUNCTION_STORAGE_REQUEST_BLOCK, SRB_FUNCTION_EXECUTE_SCSI};

#[no_mangle]
pub unsafe extern "C" fn GetSrbScsiData(
    SrbEx: PSTORAGE_REQUEST_BLOCK,
    CdbLength8: *mut u8,
    CdbLength32: *mut u32,
    ScsiStatus: *mut u8,
    SenseInfoBuffer: *mut PVOID,
    SenseInfoBufferLength: *mut u8,
) -> *mut CDB {
    let mut cdb: *mut CDB = core::ptr::null_mut();
    if !((*SrbEx).Function as u32 == SRB_FUNCTION_STORAGE_REQUEST_BLOCK
        && (*SrbEx).SrbFunction == SRB_FUNCTION_EXECUTE_SCSI)
    {
        return cdb;
    }
    let offsets = (*SrbEx).SrbExDataOffset.as_ptr();
    let mut i: u32 = 0;
    while i < (*SrbEx).NumSrbExData {
        let off = *offsets.offset(i as isize);
        i = i.wrapping_add(1);
        if (off as usize) < core::mem::size_of::<STORAGE_REQUEST_BLOCK>()
            || off >= (*SrbEx).SrbLength
        {
            continue;
        }
        let d = (SrbEx as *mut u8).offset(off as isize) as PSRBEX_DATA;
        match (*d).Type as u32 {
            64 => { // SrbExDataTypeScsiCdb16
                if off as usize + core::mem::size_of::<SRBEX_DATA_SCSI_CDB16>() <= (*SrbEx).SrbLength as usize {
                    let d = d as PSRBEX_DATA_SCSI_CDB16;
                    if !CdbLength8.is_null() { *CdbLength8 = (*d).CdbLength; }
                    if (*d).CdbLength > 0 { cdb = (&raw mut (*d).Cdb) as *mut CDB; }
                    if !ScsiStatus.is_null() { *ScsiStatus = (*d).ScsiStatus; }
                    if !SenseInfoBuffer.is_null() { *SenseInfoBuffer = (*d).SenseInfoBuffer; }
                    if !SenseInfoBufferLength.is_null() { *SenseInfoBufferLength = (*d).SenseInfoBufferLength; }
                }
            }
            65 => { // SrbExDataTypeScsiCdb32
                if off as usize + core::mem::size_of::<SRBEX_DATA_SCSI_CDB32>() <= (*SrbEx).SrbLength as usize {
                    let d = d as PSRBEX_DATA_SCSI_CDB32;
                    if !CdbLength8.is_null() { *CdbLength8 = (*d).CdbLength; }
                    if (*d).CdbLength > 0 { cdb = (&raw mut (*d).Cdb) as *mut CDB; }
                    if !ScsiStatus.is_null() { *ScsiStatus = (*d).ScsiStatus; }
                    if !SenseInfoBuffer.is_null() { *SenseInfoBuffer = (*d).SenseInfoBuffer; }
                    if !SenseInfoBufferLength.is_null() { *SenseInfoBufferLength = (*d).SenseInfoBufferLength; }
                }
            }
            66 => { // SrbExDataTypeScsiCdbVar
                if off as usize + core::mem::size_of::<SRBEX_DATA_SCSI_CDB_VAR>() <= (*SrbEx).SrbLength as usize {
                    let d = d as PSRBEX_DATA_SCSI_CDB_VAR;
                    if !CdbLength32.is_null() { *CdbLength32 = (*d).CdbLength; }
                    if (*d).CdbLength > 0 { cdb = (&raw mut (*d).Cdb) as *mut CDB; }
                    if !ScsiStatus.is_null() { *ScsiStatus = (*d).ScsiStatus; }
                    if !SenseInfoBuffer.is_null() { *SenseInfoBuffer = (*d).SenseInfoBuffer; }
                    if !SenseInfoBufferLength.is_null() { *SenseInfoBufferLength = (*d).SenseInfoBufferLength; }
                }
            }
            _ => {}
        }
    }
    cdb
}

#[no_mangle]
pub unsafe extern "C" fn SetSrbScsiData(
    SrbEx: PSTORAGE_REQUEST_BLOCK,
    CdbLength8: u8,
    CdbLength32: u32,
    ScsiStatus: u8,
    SenseInfoBuffer: PVOID,
    SenseInfoBufferLength: u8,
) {
    if !((*SrbEx).Function as u32 == SRB_FUNCTION_STORAGE_REQUEST_BLOCK
        && (*SrbEx).SrbFunction == SRB_FUNCTION_EXECUTE_SCSI)
    {
        return;
    }
    let offsets = (*SrbEx).SrbExDataOffset.as_ptr();
    let mut i: u32 = 0;
    while i < (*SrbEx).NumSrbExData {
        let off = *offsets.offset(i as isize);
        i = i.wrapping_add(1);
        if (off as usize) < core::mem::size_of::<STORAGE_REQUEST_BLOCK>()
            || off >= (*SrbEx).SrbLength
        {
            continue;
        }
        let d = (SrbEx as *mut u8).offset(off as isize) as PSRBEX_DATA;
        match (*d).Type as u32 {
            64 => {
                let d = d as PSRBEX_DATA_SCSI_CDB16;
                (*d).CdbLength = CdbLength8;
                (*d).ScsiStatus = ScsiStatus;
                (*d).SenseInfoBuffer = SenseInfoBuffer;
                (*d).SenseInfoBufferLength = SenseInfoBufferLength;
            }
            65 => {
                let d = d as PSRBEX_DATA_SCSI_CDB32;
                (*d).CdbLength = CdbLength8;
                (*d).ScsiStatus = ScsiStatus;
                (*d).SenseInfoBuffer = SenseInfoBuffer;
                (*d).SenseInfoBufferLength = SenseInfoBufferLength;
            }
            66 => {
                let d = d as PSRBEX_DATA_SCSI_CDB_VAR;
                (*d).CdbLength = CdbLength32;
                (*d).ScsiStatus = ScsiStatus;
                (*d).SenseInfoBuffer = SenseInfoBuffer;
                (*d).SenseInfoBufferLength = SenseInfoBufferLength;
            }
            _ => {}
        }
    }
}

// ---------- SCSI sense inline (scsi.h, Win8+) ----------
#[no_mangle]
pub unsafe extern "C" fn ScsiGetSenseKeyAndCodes(
    sense_buffer: *const c_void, _sense_buffer_length: u8, _options: u8,
    sense_key: *mut u8, asc: *mut u8, ascq: *mut u8,
) -> u8 {
    if sense_buffer.is_null() { return 0; }
    let b = sense_buffer as *const u8;
    let fmt = (*b) & 0x7f;
    if fmt == 0x70 || fmt == 0x71 { // fixed format
        if !sense_key.is_null() { *sense_key = (*b.add(2)) & 0x0f; }
        if !asc.is_null() { *asc = *b.add(12); }
        if !ascq.is_null() { *ascq = *b.add(13); }
    } else { // descriptor format
        if !sense_key.is_null() { *sense_key = (*b.add(1)) & 0x0f; }
        if !asc.is_null() { *asc = *b.add(2); }
        if !ascq.is_null() { *ascq = *b.add(3); }
    }
    1
}
