//! inlines.rs — FORCEINLINE / header-only / intrinsic helpers the LLM rendered as calls.
//! The READ/WRITE_REGISTER_* DDK memory-mapped-register intrinsics (volatile loads/
//! stores), the VerifyIrp*BufferSize scsiport.h FORCEINLINEs, KeRaiseIrql (wraps the
//! exported KfRaiseIrql), MmGetMdlVirtualAddress, and SENSE_BUFFER_SIZE. None are
//! ntoskrnl exports themselves (they're header macros/inlines).
#![allow(non_snake_case, non_upper_case_globals, dead_code, non_camel_case_types)]
use ntkrnl_sys::ddk::{ULONG, KIRQL, PIRP, PVOID};

// FORCEINLINE IRP-stack walker (Tail.Overlay arithmetic) — re-export the tier1 inline.
pub use ntkrnl_sys::inline::tier1::IoGetCurrentIrpStackLocation;

// ── DDK memory-mapped register I/O (wdm.h READ/WRITE_REGISTER_*) — volatile load/store.
// scsiport's ScsiPortReadRegister*/WriteRegister* export wrappers bottom out here.
#[inline(always)] pub unsafe fn READ_REGISTER_UCHAR(r: *mut u8) -> u8 { core::ptr::read_volatile(r) }
#[inline(always)] pub unsafe fn READ_REGISTER_USHORT(r: *mut u16) -> u16 { core::ptr::read_volatile(r) }
#[inline(always)] pub unsafe fn READ_REGISTER_ULONG(r: *mut u32) -> u32 { core::ptr::read_volatile(r) }
#[inline(always)] pub unsafe fn WRITE_REGISTER_UCHAR(r: *mut u8, v: u8) { core::ptr::write_volatile(r, v) }
#[inline(always)] pub unsafe fn WRITE_REGISTER_USHORT(r: *mut u16, v: u16) { core::ptr::write_volatile(r, v) }
#[inline(always)] pub unsafe fn WRITE_REGISTER_ULONG(r: *mut u32, v: u32) { core::ptr::write_volatile(r, v) }

#[inline(always)] pub unsafe fn READ_REGISTER_BUFFER_UCHAR(r: *mut u8, b: *mut u8, n: ULONG) {
    for i in 0..n as isize { *b.offset(i) = core::ptr::read_volatile(r.offset(i)); }
}
#[inline(always)] pub unsafe fn READ_REGISTER_BUFFER_USHORT(r: *mut u16, b: *mut u16, n: ULONG) {
    for i in 0..n as isize { *b.offset(i) = core::ptr::read_volatile(r.offset(i)); }
}
#[inline(always)] pub unsafe fn READ_REGISTER_BUFFER_ULONG(r: *mut u32, b: *mut u32, n: ULONG) {
    for i in 0..n as isize { *b.offset(i) = core::ptr::read_volatile(r.offset(i)); }
}
#[inline(always)] pub unsafe fn WRITE_REGISTER_BUFFER_UCHAR(r: *mut u8, b: *mut u8, n: ULONG) {
    for i in 0..n as isize { core::ptr::write_volatile(r.offset(i), *b.offset(i)); }
}
#[inline(always)] pub unsafe fn WRITE_REGISTER_BUFFER_USHORT(r: *mut u16, b: *mut u16, n: ULONG) {
    for i in 0..n as isize { core::ptr::write_volatile(r.offset(i), *b.offset(i)); }
}
#[inline(always)] pub unsafe fn WRITE_REGISTER_BUFFER_ULONG(r: *mut u32, b: *mut u32, n: ULONG) {
    for i in 0..n as isize { core::ptr::write_volatile(r.offset(i), *b.offset(i)); }
}

// KeRaiseIrql(NewIrql, OldIrql*) macro → *OldIrql = KfRaiseIrql(NewIrql). KfRaiseIrql is
// the real ntoskrnl export (fastcall on i386, cdecl on amd64/arm64).
#[cfg(target_arch = "x86")]
extern "fastcall" { fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL; }
#[cfg(not(target_arch = "x86"))]
extern "C" { fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL; }
#[inline(always)]
pub unsafe fn KeRaiseIrql(NewIrql: KIRQL, OldIrql: *mut KIRQL) {
    *OldIrql = KfRaiseIrql(NewIrql);
}

// KeLowerIrql — fastcall KfLowerIrql on i386; a plain export elsewhere (the amd64 ddk
// bindgen has it, but the i386/arm64 ones don't — provide it here for all arches).
#[cfg(target_arch = "x86")]
extern "fastcall" { fn KfLowerIrql(NewIrql: KIRQL); }
#[cfg(target_arch = "x86")]
#[inline(always)]
pub unsafe fn KeLowerIrql(NewIrql: KIRQL) { KfLowerIrql(NewIrql) }
#[cfg(not(target_arch = "x86"))]
mod lower_irql { unsafe extern "C" { pub fn KeLowerIrql(NewIrql: super::KIRQL); } }
#[cfg(not(target_arch = "x86"))]
pub use lower_irql::KeLowerIrql;

// wdm.h: MmGetMdlVirtualAddress(Mdl) = (PVOID)((PCHAR)Mdl->StartVa + Mdl->ByteOffset).
#[inline(always)]
pub unsafe fn MmGetMdlVirtualAddress(Mdl: *mut ntkrnl_sys::ddk::_MDL) -> PVOID {
    ((*Mdl).StartVa as *mut u8).add((*Mdl).ByteOffset as usize) as PVOID
}

// scsiport.h FORCEINLINE VerifyIrpOutBufferSize / VerifyIrpInBufferSize.
#[inline(always)]
pub unsafe fn VerifyIrpOutBufferSize(Irp: PIRP, Size: usize) -> u8 {
    let io_stack = IoGetCurrentIrpStackLocation(Irp);
    if ((*io_stack).Parameters.DeviceIoControl.OutputBufferLength as usize) < Size {
        (*Irp).IoStatus.Information = Size as _;
        return 0;
    }
    1
}
#[inline(always)]
pub unsafe fn VerifyIrpInBufferSize(Irp: PIRP, Size: usize) -> u8 {
    let io_stack = IoGetCurrentIrpStackLocation(Irp);
    if ((*io_stack).Parameters.DeviceIoControl.InputBufferLength as usize) < Size {
        (*Irp).IoStatus.Information = Size as _;
        return 0;
    }
    1
}

// SENSE_BUFFER_SIZE = sizeof(SENSE_DATA) — fixed 18 bytes by the SCSI spec.
pub const SENSE_BUFFER_SIZE: usize = 18;

// mmfuncs.h: MmGetSystemAddressForMdl(Mdl) — MappedSystemVa if already mapped or the
// source is nonpaged pool, else MmMapLockedPages(Mdl, KernelMode).
const MDL_MAPPED_TO_SYSTEM_VA: u16 = 0x0001;
const MDL_SOURCE_IS_NONPAGED_POOL: u16 = 0x0004;
#[inline(always)]
pub unsafe fn MmGetSystemAddressForMdl(Mdl: *mut ntkrnl_sys::ddk::_MDL) -> PVOID {
    if (*Mdl).MdlFlags as u16 & (MDL_MAPPED_TO_SYSTEM_VA | MDL_SOURCE_IS_NONPAGED_POOL) != 0 {
        (*Mdl).MappedSystemVa
    } else {
        ntkrnl_sys::ddk::MmMapLockedPages(Mdl, 0 /* KernelMode */)
    }
}

// xdk/iofuncs.h FORCEINLINE IoAllocateAdapterChannel — dispatch through the DMA_ADAPTER
// operations table (not a kernel export on this SDK surface).
#[inline(always)]
pub unsafe fn IoAllocateAdapterChannel(
    DmaAdapter: ntkrnl_sys::ddk::PDMA_ADAPTER,
    DeviceObject: ntkrnl_sys::ddk::PDEVICE_OBJECT,
    NumberOfMapRegisters: ULONG,
    ExecutionRoutine: ntkrnl_sys::ddk::PDRIVER_CONTROL,
    Context: PVOID,
) -> ntkrnl_sys::ddk::NTSTATUS {
    let alloc = (*(*DmaAdapter).DmaOperations).AllocateAdapterChannel.unwrap();
    alloc(DmaAdapter, DeviceObject, NumberOfMapRegisters, ExecutionRoutine, Context)
}
