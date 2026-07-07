//! debugblock — the vibeos debugger data block (sync-instrumentation §5).
//!
//! What NT's `KDDEBUGGER_DATA64` does for WinDbg, this does for our GDB
//! tooling: a versioned, exported, self-describing block of **compile-time
//! struct offsets** (`offset_of!` on the bindgen types — per-arch correct by
//! construction) plus runtime-stamped addresses of the interesting globals.
//!
//! Why: the Rust kernel has no usable DWARF, so every `qemu/gdb-helpers/*.py`
//! script has hand-pinned KTHREAD/EPROCESS offsets that rot on any layout
//! change (field-manual §9 pins them per incident). Scripts should read this
//! block instead — one symbol lookup (`VIBEOS_DEBUG_BLOCK`), then every
//! offset is data. Decoder: `qemu/gdb-helpers/debug_block.py`.
//!
//! Layout contract: append-only. Bump `VERSION` when appending; NEVER reorder
//! or remove fields — the decoder indexes by offset within this struct.

#![allow(non_snake_case)]

use core::mem::{offset_of, size_of};
// PsActiveProcessHead: ps-rust global read across the crate boundary as a link symbol
// (#[no_mangle] in ps-rust). crate-split Phase 1b.
extern "C" {
    static mut PsActiveProcessHead: ntkrnl_sys::ddk::_LIST_ENTRY;
}
use core::sync::atomic::{AtomicUsize, Ordering::Relaxed};

use ntkrnl_sys::ddk::{_EPROCESS, _ETHREAD, _KPRCB, _KTHREAD};

pub const VIBEOS_DEBUG_BLOCK_MAGIC: u64 = 0x3142_4456_5342_4956; // "VIBSVDB1" LE
pub const VERSION: u32 = 1;

const ARCH_I386: u32 = 1;
const ARCH_AMD64: u32 = 2;
const ARCH_ARM64: u32 = 3;

#[cfg(target_arch = "x86")]
const ARCH: u32 = ARCH_I386;
#[cfg(target_arch = "x86_64")]
const ARCH: u32 = ARCH_AMD64;
#[cfg(target_arch = "aarch64")]
const ARCH: u32 = ARCH_ARM64;

// KTHREAD.State/WaitIrql live in a bindgen anon-union nest everywhere but
// i386 (mirrors ke_defs::kt_State). Compute the offsets through the same path.
#[cfg(target_arch = "x86")]
const KTHREAD_STATE: u32 = offset_of!(_KTHREAD, State) as u32;
#[cfg(not(target_arch = "x86"))]
const KTHREAD_STATE: u32 = offset_of!(_KTHREAD, __bindgen_anon_5.__bindgen_anon_6.State) as u32;

#[cfg(target_arch = "x86")]
const KTHREAD_WAIT_IRQL: u32 = offset_of!(_KTHREAD, WaitIrql) as u32;
#[cfg(not(target_arch = "x86"))]
const KTHREAD_WAIT_IRQL: u32 =
    offset_of!(_KTHREAD, __bindgen_anon_5.__bindgen_anon_6.WaitIrql) as u32;

// KTHREAD.WaitBlock[4] union path differs per arch (mirrors kt_WaitBlock).
#[cfg(target_arch = "x86")]
const KTHREAD_WAIT_BLOCK: u32 = offset_of!(_KTHREAD, __bindgen_anon_7.WaitBlock) as u32;
#[cfg(not(target_arch = "x86"))]
const KTHREAD_WAIT_BLOCK: u32 = offset_of!(_KTHREAD, __bindgen_anon_5.WaitBlock) as u32;

/// The block. `#[repr(C)]`, append-only (see module docs).
#[repr(C)]
pub struct VibeosDebugBlock {
    // ── header ──────────────────────────────────────────────────────
    pub magic: u64,
    pub version: u32,
    pub size: u32,
    pub arch: u32,
    pub pointer_width: u32,

    // ── KTHREAD offsets ─────────────────────────────────────────────
    pub kthread_size: u32,
    pub kthread_state: u32,
    pub kthread_wait_irql: u32,
    pub kthread_wait_reason: u32,
    pub kthread_kernel_stack: u32,
    pub kthread_stack_base: u32,
    /// union { WaitBlockList, GateObject }
    pub kthread_gate_object: u32,
    pub kthread_wait_block: u32,
    pub kthread_thread_list_entry: u32, // KPROCESS-list linkage
    pub kthread_queue: u32,

    // ── ETHREAD offsets ─────────────────────────────────────────────
    pub ethread_size: u32,
    pub ethread_cid: u32,
    pub ethread_thread_list_entry: u32, // EPROCESS-list linkage

    // ── EPROCESS offsets ────────────────────────────────────────────
    pub eprocess_size: u32,
    pub eprocess_unique_process_id: u32,
    pub eprocess_active_process_links: u32,
    pub eprocess_image_file_name: u32,
    pub eprocess_pcb_thread_list_head: u32, // KPROCESS list of KTHREADs
    pub eprocess_thread_list_head: u32,     // EPROCESS list of ETHREADs

    // ── KPRCB offsets ───────────────────────────────────────────────
    pub kprcb_current_thread: u32,

    pub _pad: u32,

    // ── runtime-stamped global addresses (0 until Phase-1) ──────────
    pub ps_active_process_head: AtomicUsize,
    pub ps_initial_system_process: AtomicUsize,
    pub synctrace_table: AtomicUsize,
    pub census_scan_count: AtomicUsize,
    pub flightrec_ring: AtomicUsize,
}

// SAFETY: header/offsets are immutable; addresses are atomics.
unsafe impl Sync for VibeosDebugBlock {}

#[no_mangle]
pub static VIBEOS_DEBUG_BLOCK: VibeosDebugBlock = VibeosDebugBlock {
    magic: VIBEOS_DEBUG_BLOCK_MAGIC,
    version: VERSION,
    size: size_of::<VibeosDebugBlock>() as u32,
    arch: ARCH,
    pointer_width: size_of::<usize>() as u32 * 8,

    kthread_size: size_of::<_KTHREAD>() as u32,
    kthread_state: KTHREAD_STATE,
    kthread_wait_irql: KTHREAD_WAIT_IRQL,
    kthread_wait_reason: offset_of!(_KTHREAD, WaitReason) as u32,
    kthread_kernel_stack: offset_of!(_KTHREAD, KernelStack) as u32,
    kthread_stack_base: offset_of!(_KTHREAD, StackBase) as u32,
    kthread_gate_object: offset_of!(_KTHREAD, __bindgen_anon_2) as u32,
    kthread_wait_block: KTHREAD_WAIT_BLOCK,
    kthread_thread_list_entry: offset_of!(_KTHREAD, ThreadListEntry) as u32,
    kthread_queue: offset_of!(_KTHREAD, Queue) as u32,

    ethread_size: size_of::<_ETHREAD>() as u32,
    ethread_cid: offset_of!(_ETHREAD, Cid) as u32,
    ethread_thread_list_entry: offset_of!(_ETHREAD, ThreadListEntry) as u32,

    eprocess_size: size_of::<_EPROCESS>() as u32,
    eprocess_unique_process_id: offset_of!(_EPROCESS, UniqueProcessId) as u32,
    eprocess_active_process_links: offset_of!(_EPROCESS, ActiveProcessLinks) as u32,
    eprocess_image_file_name: offset_of!(_EPROCESS, ImageFileName) as u32,
    eprocess_pcb_thread_list_head: offset_of!(_EPROCESS, Pcb.ThreadListHead) as u32,
    eprocess_thread_list_head: offset_of!(_EPROCESS, ThreadListHead) as u32,

    kprcb_current_thread: offset_of!(_KPRCB, CurrentThread) as u32,

    _pad: 0,

    ps_active_process_head: AtomicUsize::new(0),
    ps_initial_system_process: AtomicUsize::new(0),
    synctrace_table: AtomicUsize::new(0),
    census_scan_count: AtomicUsize::new(0),
    flightrec_ring: AtomicUsize::new(0),
};

extern "C" {
    static mut PsInitialSystemProcess: *mut core::ffi::c_void;
}

/// Stamp the runtime addresses. Called from `KiStartSyncCensus` (Phase-1).
pub unsafe fn debug_block_stamp() {
    VIBEOS_DEBUG_BLOCK.ps_active_process_head.store(
        core::ptr::addr_of!(PsActiveProcessHead) as usize,
        Relaxed,
    );
    VIBEOS_DEBUG_BLOCK.ps_initial_system_process.store(
        PsInitialSystemProcess as usize,
        Relaxed,
    );
    VIBEOS_DEBUG_BLOCK.synctrace_table.store(
        core::ptr::addr_of!(crate::ke::synctrace::VIBEOS_SYNCTRACE) as usize,
        Relaxed,
    );
    VIBEOS_DEBUG_BLOCK.census_scan_count.store(
        core::ptr::addr_of!(crate::ke::census::VIBEOS_CENSUS_SCANS) as usize,
        Relaxed,
    );
    VIBEOS_DEBUG_BLOCK.flightrec_ring.store(
        core::ptr::addr_of!(crate::ke::flightrec::VIBEOS_FLIGHTREC) as usize,
        Relaxed,
    );
}
