//! Real-mode BIOS call support via the fast486 emulator.
//!
//! Translated from `hal/halx86/generic/x86bios.c`.  Compiled for both
//! i386 and amd64 (the C file is in generic.cmake unconditionally).
//! `HalpBiosDisplayReset` is amd64-only — on i386 the V86-mode path in
//! `generic/bios.c` provides it instead.
//!
//! ## FAST486_STATE mirror
//!
//! `FAST486_STATE` is mirrored field-for-field with `#[repr(C)]` so the
//! Rust layout is byte-identical to the C struct (`sdk/include/reactos/
//! libs/fast486/fast486.h`).  We only *read/write* the prefix
//! (GeneralRegs / SegmentRegs / InstPtr / Flags); the tail (GDT/IDT/LDT/
//! TSS/FPU/prefetch) is mirrored purely so `Fast486Initialize` and the
//! emulator step functions — which touch those fields at the C offsets —
//! stay in-bounds and at the right offsets.  Compile-time size asserts
//! guard the per-arch layout.
//!
//! ## Calling conventions
//!
//! fast486 callbacks are `FASTCALL` = `__fastcall` on i386, plain cdecl
//! (Win64) on amd64 (cf. `sdk/include/xdk/ntdef.template.h`).  The
//! `Fast486*` library entry points are `NTAPI` = stdcall on i386 / Win64
//! on amd64 → `extern "system"` uniformly.

#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]

use core::ffi::c_void;

use ntkrnl_sys::hal_ddk::{
    BOOLEAN, NTSTATUS, PFN_NUMBER, PLOADER_PARAMETER_BLOCK, PUCHAR, PVOID, ULONG, ULONG64,
    USHORT, _MDL, _MEMORY_ALLOCATION_DESCRIPTOR, _LOADER_PARAMETER_BLOCK,
    _LIST_ENTRY, HalpAllocPhysicalMemory, PX86_BIOS_REGISTERS,
};

const PAGE_SIZE: ULONG = 0x1000;
const MDL_PAGES_LOCKED: i16 = 2;
const HighPagePriority: ULONG = 32;
const LoaderFirmwarePermanent: u32 = 6;
const LoaderSpecialMemory: u32 = 22;

const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_INSUFFICIENT_RESOURCES: NTSTATUS = 0xC000009Au32 as NTSTATUS;
const STATUS_INVALID_PARAMETER: NTSTATUS = 0xC000000Du32 as NTSTATUS;

// fast486 register-index enums.
const FAST486_REG_EAX: usize = 0;
const FAST486_REG_ECX: usize = 1;
const FAST486_REG_EDX: usize = 2;
const FAST486_REG_EBX: usize = 3;
const FAST486_REG_ESI: usize = 6;
const FAST486_REG_EDI: usize = 7;
const FAST486_REG_ES: u32 = 0;
const FAST486_REG_CS: usize = 1;
const FAST486_REG_DS: u32 = 3;

// ---------------------------------------------------------------------------
// FAST486_STATE layout mirror (see module docs).
// ---------------------------------------------------------------------------

#[repr(C)]
#[derive(Copy, Clone)]
struct FAST486_REG {
    Long: ULONG,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct FAST486_SEG_REG {
    Selector: USHORT,
    _pad: USHORT,
    _descriptor_cache: ULONG,
    Limit: ULONG,
    Base: ULONG,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct FAST486_FLAGS_REG {
    Long: ULONG,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct FAST486_TABLE_REG {
    Size: USHORT,
    _pad: USHORT,
    Address: ULONG,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct FAST486_LDT_REG {
    Selector: USHORT,
    _pad: USHORT,
    Base: ULONG,
    Limit: ULONG,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct FAST486_TASK_REG {
    Selector: USHORT,
    _pad: USHORT,
    Base: ULONG,
    Limit: ULONG,
    Modern: BOOLEAN,
    _pad2: [u8; 3],
}

#[repr(C)]
#[derive(Copy, Clone)]
struct FAST486_FPU_DATA_REG {
    Mantissa: u64,
    Exponent: USHORT,
    Sign: u8,
    _pad: [u8; 5],
}

#[repr(C)]
#[derive(Copy, Clone)]
struct FAST486_FPU_STATUS_REG {
    raw: ULONG,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct FAST486_FPU_CONTROL_REG {
    raw: ULONG,
}

// Callback fn-pointer slots: pointer-sized placeholders.  We never call
// through these from Rust — Fast486Initialize stores the real pointers
// it's handed and the emulator invokes them itself.
type Fast486Callback = Option<unsafe extern "C" fn()>;

#[repr(C)]
struct FAST486_STATE {
    MemReadCallback: Fast486Callback,
    MemWriteCallback: Fast486Callback,
    IoReadCallback: Fast486Callback,
    IoWriteCallback: Fast486Callback,
    BopCallback: Fast486Callback,
    IntAckCallback: Fast486Callback,
    FpuCallback: Fast486Callback,
    GeneralRegs: [FAST486_REG; 8],
    SegmentRegs: [FAST486_SEG_REG; 6],
    InstPtr: FAST486_REG,
    SavedInstPtr: FAST486_REG,
    SavedStackPtr: FAST486_REG,
    Flags: FAST486_FLAGS_REG,
    Gdtr: FAST486_TABLE_REG,
    Idtr: FAST486_TABLE_REG,
    Ldtr: FAST486_LDT_REG,
    TaskReg: FAST486_TASK_REG,
    Cpl: u8,
    ControlRegisters: [ULONG; 3],
    DebugRegisters: [ULONG; 6],
    ExceptionCount: ULONG,
    PrefixFlags: ULONG,
    SegmentOverride: ULONG,
    Halted: BOOLEAN,
    IntSignaled: BOOLEAN,
    DoNotInterrupt: BOOLEAN,
    Tlb: *mut ULONG,
    TlbEmpty: BOOLEAN,
    // FAST486_NO_PREFETCH undefined → prefetch fields present.
    PrefetchValid: BOOLEAN,
    PrefetchAddress: ULONG,
    PrefetchCache: [u8; 32],
    // FAST486_NO_FPU undefined → FPU fields present.
    FpuRegisters: [FAST486_FPU_DATA_REG; 8],
    FpuStatus: FAST486_FPU_STATUS_REG,
    FpuControl: FAST486_FPU_CONTROL_REG,
    FpuTag: USHORT,
    FpuLastInstPtr: FAST486_REG,
    FpuLastCodeSel: USHORT,
    FpuLastOpPtr: FAST486_REG,
    FpuLastDataSel: USHORT,
}

// Layout tripwires — the accessed-prefix offsets must match the C header
// or BIOS register marshalling silently corrupts.
const _: () = {
    assert!(core::mem::offset_of!(FAST486_STATE, GeneralRegs) == 7 * core::mem::size_of::<usize>());
    assert!(core::mem::size_of::<FAST486_REG>() == 4);
    assert!(core::mem::size_of::<FAST486_SEG_REG>() == 16);
    assert!(core::mem::size_of::<FAST486_FLAGS_REG>() == 4);
    assert!(core::mem::size_of::<FAST486_FPU_DATA_REG>() == 16);
};

// FAST486_FLAGS_REG bit positions (cf. fast486.h _FAST486_FLAGS_REG).
const FLAG_ALWAYS_SET: ULONG = 1 << 1;
const FLAG_IF: ULONG = 1 << 9;

// ---------------------------------------------------------------------------
// fast486 library entry points — NTAPI (extern "system" on both arches).
// ---------------------------------------------------------------------------

#[cfg(target_arch = "x86")]
type Fast486MemProc =
    Option<unsafe extern "fastcall" fn(*mut FAST486_STATE, ULONG, PVOID, ULONG)>;
#[cfg(target_arch = "x86_64")]
type Fast486MemProc = Option<unsafe extern "C" fn(*mut FAST486_STATE, ULONG, PVOID, ULONG)>;

#[cfg(target_arch = "x86")]
type Fast486IoProc =
    Option<unsafe extern "fastcall" fn(*mut FAST486_STATE, USHORT, PVOID, ULONG, u8)>;
#[cfg(target_arch = "x86_64")]
type Fast486IoProc = Option<unsafe extern "C" fn(*mut FAST486_STATE, USHORT, PVOID, ULONG, u8)>;

#[cfg(target_arch = "x86")]
type Fast486BopProc = Option<unsafe extern "fastcall" fn(*mut FAST486_STATE, u8)>;
#[cfg(target_arch = "x86_64")]
type Fast486BopProc = Option<unsafe extern "C" fn(*mut FAST486_STATE, u8)>;

#[cfg(target_arch = "x86")]
type Fast486IntAckProc = Option<unsafe extern "fastcall" fn(*mut FAST486_STATE) -> u8>;
#[cfg(target_arch = "x86_64")]
type Fast486IntAckProc = Option<unsafe extern "C" fn(*mut FAST486_STATE) -> u8>;

type Fast486FpuProc = Option<unsafe extern "C" fn()>;

unsafe extern "system" {
    fn Fast486Initialize(
        State: *mut FAST486_STATE,
        MemReadCallback: Fast486MemProc,
        MemWriteCallback: Fast486MemProc,
        IoReadCallback: Fast486IoProc,
        IoWriteCallback: Fast486IoProc,
        BopCallback: Fast486BopProc,
        IntAckCallback: Fast486IntAckProc,
        FpuCallback: Fast486FpuProc,
        Tlb: *mut ULONG,
    );
    fn Fast486StepInto(State: *mut FAST486_STATE);
    fn Fast486ExecuteAt(State: *mut FAST486_STATE, Segment: USHORT, Offset: ULONG);
    fn Fast486SetStack(State: *mut FAST486_STATE, Segment: USHORT, Offset: ULONG);
    fn Fast486SetSegment(State: *mut FAST486_STATE, Segment: u32, Selector: USHORT);
}

// ntoskrnl / HAL imports.
unsafe extern "system" {
    fn IoAllocateMdl(
        VirtualAddress: PVOID,
        Length: ULONG,
        SecondaryBuffer: BOOLEAN,
        ChargeQuota: BOOLEAN,
        Irp: PVOID,
    ) -> *mut _MDL;
    fn MmMapLockedPagesSpecifyCache(
        Mdl: *mut _MDL,
        AccessMode: u8,
        CacheType: u32,
        RequestedAddress: PVOID,
        BugCheckOnFailure: ULONG,
        Priority: ULONG,
    ) -> PVOID;
}

unsafe extern "C" {
    fn DbgPrint(Format: *const u8, ...) -> u32;
}

// Port-buffer I/O — translate the READ/WRITE_PORT_BUFFER_* macros to
// `rep ins/outs` inline asm.  rep uses (R|E)DI/(R|E)SI/(R|E)CX implicitly;
// the assembler picks the width from the named register operands, which
// are pointer-width per arch (rdi/rsi/rcx on amd64, edi/esi/ecx on i386).
#[cfg(target_arch = "x86_64")]
macro_rules! port_in {
    ($name:ident, $insn:literal) => {
        #[inline(always)]
        unsafe fn $name(port: u16, buf: *mut u8, count: ULONG) {
            core::arch::asm!(
                $insn,
                in("dx") port,
                inout("rdi") buf => _,
                inout("rcx") count as usize => _,
                options(nostack, preserves_flags),
            );
        }
    };
}
#[cfg(target_arch = "x86_64")]
macro_rules! port_out {
    ($name:ident, $insn:literal) => {
        #[inline(always)]
        unsafe fn $name(port: u16, buf: *const u8, count: ULONG) {
            core::arch::asm!(
                $insn,
                in("dx") port,
                inout("rsi") buf => _,
                inout("rcx") count as usize => _,
                options(nostack, preserves_flags),
            );
        }
    };
}
#[cfg(target_arch = "x86")]
macro_rules! port_in {
    ($name:ident, $insn:literal) => {
        #[inline(always)]
        unsafe fn $name(port: u16, buf: *mut u8, count: ULONG) {
            core::arch::asm!(
                $insn,
                in("dx") port,
                inout("edi") buf => _,
                inout("ecx") count as usize => _,
                options(nostack, preserves_flags),
            );
        }
    };
}
// i386: LLVM reserves ESI, so we can't bind it directly.  Stash the
// source pointer in a scratch reg, xchg it into ESI around the string
// op, xchg back (same idiom as the CMPXCHG8B EBX workaround).
#[cfg(target_arch = "x86")]
macro_rules! port_out {
    ($name:ident, $insn:literal) => {
        #[inline(always)]
        unsafe fn $name(port: u16, buf: *const u8, count: ULONG) {
            core::arch::asm!(
                "xchg esi, {ptr}",
                $insn,
                "xchg esi, {ptr}",
                ptr = inout(reg) buf => _,
                in("dx") port,
                inout("ecx") count as usize => _,
                options(nostack, preserves_flags),
            );
        }
    };
}
port_in!(read_port_buffer_uchar, "rep insb");
port_in!(read_port_buffer_ushort, "rep insw");
port_in!(read_port_buffer_ulong, "rep insd");
port_out!(write_port_buffer_uchar, "rep outsb");
port_out!(write_port_buffer_ushort, "rep outsw");
port_out!(write_port_buffer_ulong, "rep outsd");

// ---------------------------------------------------------------------------
// Globals — match the C symbol names so consumers (display.rs / kdpci /
// bootvid) keep resolving.
// ---------------------------------------------------------------------------

#[no_mangle]
pub static mut x86BiosFallbackPfn: PFN_NUMBER = 0;

#[no_mangle]
pub static mut x86BiosIsInitialized: BOOLEAN = 0;

#[no_mangle]
pub static mut x86BiosBufferIsAllocated: i32 = 0;

#[no_mangle]
pub static mut x86BiosMemoryMapping: PUCHAR = core::ptr::null_mut();

#[no_mangle]
pub static mut x86BiosBufferPhysical: ULONG64 = 0;

#[inline(always)]
unsafe fn rtl_copy_memory(dst: *mut u8, src: *const u8, len: usize) {
    core::ptr::copy_nonoverlapping(src, dst, len);
}

// ---------------------------------------------------------------------------
// HalInitializeBios — phase 0 grabs two low-1MB physical pages; phase 1
// builds a 1MB MDL mapping firmware/fallback pages and maps it to system
// space so the emulator's flat address space === real-mode 0..1MB.
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "system" fn HalInitializeBios(
    Phase: ULONG,
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
) {
    if Phase == 0 {
        let phys = HalpAllocPhysicalMemory(LoaderBlock, 0x100000, 1, 0);
        debug_assert!(phys != 0);
        x86BiosFallbackPfn = (phys / PAGE_SIZE as ULONG64) as PFN_NUMBER;
        debug_assert!(x86BiosFallbackPfn != 0);

        x86BiosBufferPhysical = HalpAllocPhysicalMemory(LoaderBlock, 0x100000, 1, 0);
        debug_assert!(x86BiosBufferPhysical != 0);
        return;
    }

    // Phase 1.
    let mdl = IoAllocateMdl(core::ptr::null_mut(), 0x100000, 0, 0, core::ptr::null_mut());
    debug_assert!(!mdl.is_null());

    // MmGetMdlPfnArray(Mdl) == (PPFN_NUMBER)(Mdl + 1)
    let pfn_array = mdl.add(1) as *mut PFN_NUMBER;

    // Fill the array with the fallback page.
    for pfn in 0..0x100usize {
        *pfn_array.add(pfn) = x86BiosFallbackPfn;
    }

    // Walk the loader memory descriptors; map firmware pages 1:1.
    let head = core::ptr::addr_of_mut!((*LoaderBlock).MemoryDescriptorListHead);
    let mut list_entry = (*head).Flink;
    while list_entry != head as *mut _LIST_ENTRY {
        // CONTAINING_RECORD(ListEntry, MEMORY_ALLOCATION_DESCRIPTOR, ListEntry)
        // — ListEntry is the first field, so the cast is a no-op offset.
        let descriptor = list_entry as *mut _MEMORY_ALLOCATION_DESCRIPTOR;

        if (*descriptor).BasePage < 0x100 {
            let mt = (*descriptor).MemoryType;
            if mt == LoaderFirmwarePermanent || mt == LoaderSpecialMemory {
                let base = (*descriptor).BasePage;
                let last = core::cmp::min(base + (*descriptor).PageCount, 0x100);
                let mut pfn = base;
                while pfn < last {
                    *pfn_array.add(pfn as usize) = pfn;
                    pfn += 1;
                }
            }
        }

        list_entry = (*list_entry).Flink;
    }

    // Map the scratch buffer page proper, too.
    let pfn = (x86BiosBufferPhysical / PAGE_SIZE as ULONG64) as PFN_NUMBER;
    *pfn_array.add(pfn as usize) = pfn;

    (*mdl).MdlFlags = MDL_PAGES_LOCKED;

    // MmGetSystemAddressForMdlSafe(Mdl, HighPagePriority): the MDL isn't
    // yet system-mapped, so this resolves to MmMapLockedPagesSpecifyCache
    // (KernelMode=0, MmCached=1, no requested addr, no bugcheck).
    x86BiosMemoryMapping = MmMapLockedPagesSpecifyCache(
        mdl,
        0,
        1,
        core::ptr::null_mut(),
        0,
        HighPagePriority,
    ) as PUCHAR;
    debug_assert!(!x86BiosMemoryMapping.is_null());

    x86BiosIsInitialized = 1;
}

// ---------------------------------------------------------------------------
// Scratch-buffer allocator (single page, single client).
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "system" fn x86BiosAllocateBuffer(
    Size: *mut ULONG,
    Segment: *mut USHORT,
    Offset: *mut USHORT,
) -> NTSTATUS {
    if x86BiosIsInitialized == 0 || *Size > PAGE_SIZE {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // InterlockedBitTestAndSet(&x86BiosBufferIsAllocated, 0)
    let prev = core::sync::atomic::AtomicI32::from_ptr(
        core::ptr::addr_of_mut!(x86BiosBufferIsAllocated),
    )
    .fetch_or(1, core::sync::atomic::Ordering::SeqCst);
    if prev & 1 != 0 {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    *Size = PAGE_SIZE;
    *Segment = (x86BiosBufferPhysical / 16) as USHORT;
    *Offset = 0;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn x86BiosFreeBuffer(
    Segment: USHORT,
    Offset: USHORT,
) -> NTSTATUS {
    if x86BiosIsInitialized == 0 || Segment != 0x2000 || Offset != 0 {
        return STATUS_INVALID_PARAMETER;
    }

    // InterlockedBitTestAndReset(&x86BiosBufferIsAllocated, 0)
    let prev = core::sync::atomic::AtomicI32::from_ptr(
        core::ptr::addr_of_mut!(x86BiosBufferIsAllocated),
    )
    .fetch_and(!1, core::sync::atomic::Ordering::SeqCst);
    if prev & 1 == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn x86BiosReadMemory(
    Segment: USHORT,
    Offset: USHORT,
    Buffer: PVOID,
    Size: ULONG,
) -> NTSTATUS {
    let address = ((Segment as usize) << 4) + Offset as usize;
    if x86BiosIsInitialized == 0 || (address + Size as usize) > 0x100000 {
        return STATUS_INVALID_PARAMETER;
    }
    rtl_copy_memory(
        Buffer as *mut u8,
        x86BiosMemoryMapping.add(address) as *const u8,
        Size as usize,
    );
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn x86BiosWriteMemory(
    Segment: USHORT,
    Offset: USHORT,
    Buffer: PVOID,
    Size: ULONG,
) -> NTSTATUS {
    let address = ((Segment as usize) << 4) + Offset as usize;
    if x86BiosIsInitialized == 0 || (address + Size as usize) > 0x100000 {
        return STATUS_INVALID_PARAMETER;
    }
    rtl_copy_memory(
        x86BiosMemoryMapping.add(address),
        Buffer as *const u8,
        Size as usize,
    );
    STATUS_SUCCESS
}

// ---------------------------------------------------------------------------
// fast486 callbacks — FASTCALL on i386, cdecl/Win64 on amd64.  Shared
// bodies live in *_impl helpers so the cfg'd ABI wrappers stay thin.
// ---------------------------------------------------------------------------

#[inline]
unsafe fn x86_mem_read_impl(_state: *mut FAST486_STATE, address: ULONG, buffer: PVOID, size: ULONG) {
    if (address as u64 + size as u64) < 0x100000 {
        rtl_copy_memory(
            buffer as *mut u8,
            x86BiosMemoryMapping.add(address as usize) as *const u8,
            size as usize,
        );
    } else {
        core::ptr::write_bytes(buffer as *mut u8, 0xCC, size as usize);
        DbgPrint(b"x86MemRead: invalid read at 0x%lx (size 0x%lx)\n\0".as_ptr(), address, size);
    }
}

#[inline]
unsafe fn x86_mem_write_impl(_state: *mut FAST486_STATE, address: ULONG, buffer: PVOID, size: ULONG) {
    if (address as u64 + size as u64) < 0x100000 {
        rtl_copy_memory(
            x86BiosMemoryMapping.add(address as usize),
            buffer as *const u8,
            size as usize,
        );
    } else {
        DbgPrint(b"x86MemWrite: invalid write at 0x%lx (size 0x%lx)\n\0".as_ptr(), address, size);
    }
}

fn validate_port(port: USHORT, size: u8, is_write: bool) -> bool {
    match port {
        // VGA — https://wiki.osdev.org/VGA_Hardware#Port_0x3C0
        0x3C0 => size == 1 && is_write,
        0x3C1 => size == 1 && !is_write,
        0x3C2 => size == 1 && is_write,
        0x3C4 => is_write,
        0x3C5 => size <= 2,
        0x3C7 => size == 1 && is_write,
        0x3CC => size == 1 && !is_write,
        0x3CE => is_write,
        0x3CF => size <= 2,
        0x3D4 => is_write,
        0x3D5 => size <= 2,
        0x3C6 => size == 1,
        0x3C8 => size == 1 && is_write,
        0x3C9 => size == 1,
        0x3DA => size == 1 && !is_write,
        // OVMF debug port (VBox / QEMU).
        0x402 => size == 1 && is_write,
        // BOCHS VBE.
        0x1CE => size == 1 && is_write,
        0x1CF => size == 1,
        0x3B6 => size <= 2,
        _ => {
            unsafe {
                DbgPrint(
                    b"Unknown port 0x%x, size %d, write %d\n\0".as_ptr(),
                    port as u32,
                    size as u32,
                    is_write as u32,
                );
            }
            true
        }
    }
}

#[inline]
unsafe fn x86_io_read_impl(
    _state: *mut FAST486_STATE,
    port: USHORT,
    buffer: PVOID,
    data_count: ULONG,
    data_size: u8,
) {
    if !validate_port(port, data_size, false) {
        DbgPrint(
            b"Invalid IO port read access (port: 0x%x, count: 0x%x)\n\0".as_ptr(),
            port as u32,
            data_size as u32,
        );
    }
    match data_size {
        1 => read_port_buffer_uchar(port, buffer as *mut u8, data_count),
        2 => read_port_buffer_ushort(port, buffer as *mut u8, data_count),
        4 => read_port_buffer_ulong(port, buffer as *mut u8, data_count),
        _ => {}
    }
}

#[inline]
unsafe fn x86_io_write_impl(
    _state: *mut FAST486_STATE,
    port: USHORT,
    buffer: PVOID,
    data_count: ULONG,
    data_size: u8,
) {
    if !validate_port(port, data_size, true) {
        DbgPrint(
            b"Invalid IO port write access (port: 0x%x, count: 0x%x)\n\0".as_ptr(),
            port as u32,
            data_size as u32,
        );
    }
    match data_size {
        1 => write_port_buffer_uchar(port, buffer as *const u8, data_count),
        2 => write_port_buffer_ushort(port, buffer as *const u8, data_count),
        4 => write_port_buffer_ulong(port, buffer as *const u8, data_count),
        _ => {}
    }
}

#[inline]
unsafe fn x86_bop_impl(_state: *mut FAST486_STATE, _bop: u8) {
    debug_assert!(false, "x86BOP invoked");
}

#[inline]
unsafe fn x86_int_ack_impl(_state: *mut FAST486_STATE) -> u8 {
    debug_assert!(false, "x86IntAck invoked");
    0
}

macro_rules! fast486_cb {
    ($name:ident, $abi:literal, ($($a:ident: $t:ty),*) $(-> $r:ty)?, $impl:ident) => {
        pub unsafe extern $abi fn $name($($a: $t),*) $(-> $r)? {
            super::$impl($($a),*)
        }
    };
}

#[cfg(target_arch = "x86")]
mod cb {
    use super::*;
    fast486_cb!(x86MemRead, "fastcall", (s: *mut FAST486_STATE, a: ULONG, b: PVOID, n: ULONG), x86_mem_read_impl);
    fast486_cb!(x86MemWrite, "fastcall", (s: *mut FAST486_STATE, a: ULONG, b: PVOID, n: ULONG), x86_mem_write_impl);
    fast486_cb!(x86IoRead, "fastcall", (s: *mut FAST486_STATE, p: USHORT, b: PVOID, c: ULONG, z: u8), x86_io_read_impl);
    fast486_cb!(x86IoWrite, "fastcall", (s: *mut FAST486_STATE, p: USHORT, b: PVOID, c: ULONG, z: u8), x86_io_write_impl);
    fast486_cb!(x86BOP, "fastcall", (s: *mut FAST486_STATE, c: u8), x86_bop_impl);
    fast486_cb!(x86IntAck, "fastcall", (s: *mut FAST486_STATE) -> u8, x86_int_ack_impl);
}

#[cfg(target_arch = "x86_64")]
mod cb {
    use super::*;
    fast486_cb!(x86MemRead, "C", (s: *mut FAST486_STATE, a: ULONG, b: PVOID, n: ULONG), x86_mem_read_impl);
    fast486_cb!(x86MemWrite, "C", (s: *mut FAST486_STATE, a: ULONG, b: PVOID, n: ULONG), x86_mem_write_impl);
    fast486_cb!(x86IoRead, "C", (s: *mut FAST486_STATE, p: USHORT, b: PVOID, c: ULONG, z: u8), x86_io_read_impl);
    fast486_cb!(x86IoWrite, "C", (s: *mut FAST486_STATE, p: USHORT, b: PVOID, c: ULONG, z: u8), x86_io_write_impl);
    fast486_cb!(x86BOP, "C", (s: *mut FAST486_STATE, c: u8), x86_bop_impl);
    fast486_cb!(x86IntAck, "C", (s: *mut FAST486_STATE) -> u8, x86_int_ack_impl);
}

// ---------------------------------------------------------------------------
// x86BiosCall — set up the emulator, plant an `INT n; NOP` stub at the
// top of the scratch stack, run until we return to it.
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "system" fn x86BiosCall(
    InterruptNumber: ULONG,
    Registers: PX86_BIOS_REGISTERS,
) -> BOOLEAN {
    const STACK_BASE: ULONG = 0x2000;
    let mut ctx: FAST486_STATE = core::mem::zeroed();

    Fast486Initialize(
        &mut ctx,
        Some(cb::x86MemRead),
        Some(cb::x86MemWrite),
        Some(cb::x86IoRead),
        Some(cb::x86IoWrite),
        Some(cb::x86BOP),
        Some(cb::x86IntAck),
        None,
        core::ptr::null_mut(),
    );

    ctx.GeneralRegs[FAST486_REG_EAX].Long = (*Registers).Eax;
    ctx.GeneralRegs[FAST486_REG_EBX].Long = (*Registers).Ebx;
    ctx.GeneralRegs[FAST486_REG_ECX].Long = (*Registers).Ecx;
    ctx.GeneralRegs[FAST486_REG_EDX].Long = (*Registers).Edx;
    ctx.GeneralRegs[FAST486_REG_ESI].Long = (*Registers).Esi;
    ctx.GeneralRegs[FAST486_REG_EDI].Long = (*Registers).Edi;

    Fast486SetSegment(&mut ctx, FAST486_REG_DS, (*Registers).SegDs);
    Fast486SetSegment(&mut ctx, FAST486_REG_ES, (*Registers).SegEs);

    ctx.Flags.Long = FLAG_ALWAYS_SET | FLAG_IF;

    // Plant `INT n; NOP` at StackBase-4.
    let flat_ip = STACK_BASE - 4;
    let ip = x86BiosMemoryMapping.add(flat_ip as usize);
    *ip.add(0) = 0xCD; // INT
    *ip.add(1) = InterruptNumber as u8;
    *ip.add(2) = 0x90; // NOP — we stop here.

    Fast486SetStack(&mut ctx, 0, STACK_BASE - 8);
    Fast486ExecuteAt(&mut ctx, 0x00, flat_ip);

    loop {
        let flat = ((ctx.SegmentRegs[FAST486_REG_CS].Selector as ULONG) << 4)
            + ctx.InstPtr.Long;

        if flat >= 0x100000 {
            DbgPrint(
                b"x86BiosCall: invalid IP (0x%lx) during BIOS execution\n\0".as_ptr(),
                flat,
            );
            return 0;
        }

        // Returned from our INT stub?
        if flat == STACK_BASE - 2 {
            break;
        }

        Fast486StepInto(&mut ctx);
    }

    (*Registers).Eax = ctx.GeneralRegs[FAST486_REG_EAX].Long;
    (*Registers).Ebx = ctx.GeneralRegs[FAST486_REG_EBX].Long;
    (*Registers).Ecx = ctx.GeneralRegs[FAST486_REG_ECX].Long;
    (*Registers).Edx = ctx.GeneralRegs[FAST486_REG_EDX].Long;
    (*Registers).Esi = ctx.GeneralRegs[FAST486_REG_ESI].Long;
    (*Registers).Edi = ctx.GeneralRegs[FAST486_REG_EDI].Long;
    (*Registers).SegDs = ctx.SegmentRegs[FAST486_REG_DS as usize].Selector;
    (*Registers).SegEs = ctx.SegmentRegs[FAST486_REG_ES as usize].Selector;

    1
}

// ---------------------------------------------------------------------------
// HalpBiosDisplayReset — amd64 only (i386 uses the V86 path in bios.c).
// INT 0x10, AX=0x0012 → 640x480x16 VGA.
// ---------------------------------------------------------------------------

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn read_eflags() -> u64 {
    let f: u64;
    core::arch::asm!("pushfq", "pop {0}", out(reg) f, options(nomem, preserves_flags));
    f
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn write_eflags(f: u64) {
    core::arch::asm!("push {0}", "popfq", in(reg) f, options(nomem));
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "system" fn HalpBiosDisplayReset() -> BOOLEAN {
    let old_eflags = read_eflags();
    core::arch::asm!("cli", options(nomem, nostack));

    let mut regs: ntkrnl_sys::hal_ddk::X86_BIOS_REGISTERS = core::mem::zeroed();
    regs.Eax = 0x0012;
    let success = x86BiosCall(0x10, &mut regs);

    write_eflags(old_eflags);
    success
}
