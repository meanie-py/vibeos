//! HAL initialization — HalInitSystem (entrypoint) + HalInitialize-
//! Processor (per-CPU init) + a small command-line parser
//! (HalpGetParameters).  Sets up the dispatch tables that the kernel
//! reads from HalDispatchTable / HalPrivateDispatchTable.
//!
//! Translated from `hal/halx86/generic/halinit.c`.  Two HAL-private
//! globals stay defined here:
//! - HalpOnlyBootProcessor — set if "ONECPU" is on the kernel command line
//! - HalpPciLockSettings — set if "PCILOCK" is on the kernel command line

// KeInitializeSpinLock is FORCEINLINE in C (just `*p = 0`) — inline here
// directly.  KeGetPcr / KeGetCurrentPrcb live in `pcr` (local replicas
// of the ntkrnl-sys inlines; cross-crate inline-asm doesn't reliably
// fire at our opt-level — see feedback_cross_crate_inline_asm.md).
// KeBugCheckEx resolves fine via the import library.
use ntkrnl_sys::ddk::KeBugCheckEx;
use ntkrnl_sys::hal_ddk::{
    HalDispatchTable, HalPrivateDispatchTable, HalStopProfileInterrupt,
    HalpActiveProcessors, HalpAddressUsageList, HalpBusType,
    HalpCalibrateStallExecution, HalpDefaultInterruptAffinity, HalpDefaultIoSpace,
    HalpGetDmaAdapter, HalpInitPhase0, HalpInitPhase1, HalpInitProcessor,
    HalpInitializeClock, HalpInitializeCmos, HalpInitializePICs,
    HalpRegisterKdSupportFunctions, HalpSetupAcpiPhase0, HalpSystemHardwareLock,
    HaliHaltSystem, HaliInitPnpDriver, BOOLEAN, PLOADER_PARAMETER_BLOCK, ULONG,
    _KPROFILE_SOURCE_ProfileTime as ProfileTime,
};
use crate::x86::buildtype::HalpBuildType;
use crate::x86::pcr::{KeGetCurrentPrcb, KeGetPcr};
use crate::x86::sysinfo::{HaliQuerySystemInformation, HaliSetSystemInformation};

// Bugcheck code for ACPI init failure (sdk/include/reactos/mc/bugcodes.h).
const ACPI_BIOS_ERROR: ULONG = 0xa5;
// Bugcheck code for kernel/HAL header revision mismatch.
const MISMATCHED_HAL: ULONG = 0x79;
// Per-arch ndk constants (sdk/include/ndk/{i386,amd64}/ketypes.h).
const PRCB_MAJOR_VERSION: u16 = 1;
const INITIAL_STALL_COUNT: ULONG = 100;

// HAL-private command-line parameter state.  Externally referenced
// (HalpOnlyBootProcessor by various SMP paths; HalpPciLockSettings by
// PCI bus enumeration).
#[unsafe(no_mangle)]
pub static mut HalpOnlyBootProcessor: BOOLEAN = 0; /* FALSE */
#[unsafe(no_mangle)]
pub static mut HalpPciLockSettings: BOOLEAN = 0; /* FALSE */

/// Tiny strstr — sufficient for parsing the kernel command line for
/// the three options HalpGetParameters cares about (ONECPU, PCILOCK,
/// BREAK).  Avoids pulling in libcntpr's strstr from a Rust crate
/// and the variadic-FFI issues that come with it.
fn cmdline_contains(haystack: *const i8, needle: &[u8]) -> bool {
    if haystack.is_null() {
        return false;
    }
    unsafe {
        let mut p = haystack as *const u8;
        loop {
            // Compare needle against haystack starting at p.
            let mut q = p;
            let mut n = needle;
            let mut matched = true;
            while !n.is_empty() {
                let hc = *q;
                if hc == 0 {
                    return false; // hit end of haystack mid-match
                }
                if hc != n[0] {
                    matched = false;
                    break;
                }
                q = q.add(1);
                n = &n[1..];
            }
            if matched {
                return true;
            }
            if *p == 0 {
                return false;
            }
            p = p.add(1);
        }
    }
}

/// Pull the three command-line flags HAL cares about out of the
/// loader block.  ONECPU sets HalpOnlyBootProcessor (UP override),
/// PCILOCK sets HalpPciLockSettings, BREAK triggers an immediate
/// debugger break.
unsafe fn HalpGetParameters(LoaderBlock: PLOADER_PARAMETER_BLOCK) {
    if LoaderBlock.is_null() {
        return;
    }
    let load_options = unsafe { (*LoaderBlock).LoadOptions };
    if load_options.is_null() {
        return;
    }
    if cmdline_contains(load_options, b"ONECPU") {
        unsafe { core::ptr::write_volatile(&raw mut HalpOnlyBootProcessor, 1); }
    }
    if cmdline_contains(load_options, b"PCILOCK") {
        unsafe { core::ptr::write_volatile(&raw mut HalpPciLockSettings, 1); }
    }
    if cmdline_contains(load_options, b"BREAK") {
        unsafe { core::arch::asm!("int 3", options(nostack)); }
    }
}

/// hal.spec: `HalInitializeProcessor`. Per-CPU initialization.  Sets
/// up the per-processor PCR fields HAL cares about (stall scale
/// factor), registers this CPU in the HAL affinity masks, and on
/// the boot processor (#0) wires KD support callbacks.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalInitializeProcessor(
    ProcessorNumber: ULONG,
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
) {
    unsafe {
        HalpInitProcessor(ProcessorNumber, LoaderBlock);

        // Set default stall count.  StallScaleFactor lives at the same
        // offset (0x51c on i386, 0x368 on amd64) in both KPCR layouts;
        // the bindgen field is named identically.
        (*KeGetPcr()).StallScaleFactor = INITIAL_STALL_COUNT;

        // Track this processor in both HAL affinity masks.  The C uses
        // InterlockedBitTestAndSetAffinity macros; the underlying op is
        // just `lock bts` against a KAFFINITY.  Use the Rust atomic ops, at
        // KAFFINITY's width (ULONG_PTR): a 64-bit view of the 4-byte i386
        // globals was a misaligned 8-byte RMW spanning the next variable.
        let mask: usize = 1 << ProcessorNumber;
        let active = &raw mut HalpActiveProcessors as *mut usize;
        let default_aff = &raw mut HalpDefaultInterruptAffinity as *mut usize;
        let active_atomic = core::sync::atomic::AtomicUsize::from_ptr(active);
        let default_atomic = core::sync::atomic::AtomicUsize::from_ptr(default_aff);
        active_atomic.fetch_or(mask, core::sync::atomic::Ordering::SeqCst);
        default_atomic.fetch_or(mask, core::sync::atomic::Ordering::SeqCst);

        if ProcessorNumber == 0 {
            HalpRegisterKdSupportFunctions();
        }
    }
}

/// hal.spec: `HalInitSystem`. Top-level HAL boot entry point, called
/// twice by the kernel (BootPhase 0 then 1).
///
/// Phase 0: PRCB version check, ACPI phase 0, PIC init, CMOS lock,
/// dispatch-table wiring, stall calibration, clock init.
///
/// Phase 1: bus handlers, x86bios phase 1.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalInitSystem(
    BootPhase: ULONG,
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
) -> BOOLEAN {
    unsafe {
        // NOTE (i386 SMP M5 Phase C): the AP is prevented from re-running this
        // GLOBAL HAL init at the CALLER — ex-rust ExpInitializeExecutive skips
        // HalInitSystem when its explicit `Cpu` arg != 0. We do NOT gate here on
        // (*KeGetCurrentPrcb()).Number because the HAL's `hal_ddk` bindgen lays
        // out _KPRCB.Number at a different offset than the kernel's `ddk` view
        // (proven by GDB: KiInitializePcr writes Number=1 at the ddk offset, but a
        // hal_ddk read returns 0 on the AP) — an unreliable cross-bindgen field
        // read. The Cpu-parameter gate at the caller is layout-independent.
        if BootPhase == 0 {
            let prcb = KeGetCurrentPrcb();

            // Stash the bus type (PC-AT, EISA, MCA, PC-98, ...).  The
            // I386 union member is reused on amd64 too — both PE arches
            // share that field at the same offset.
            let machine_type = (*LoaderBlock).u.I386.MachineType & 0xFF;
            core::ptr::write_volatile(&raw mut HalpBusType, machine_type);

            HalpGetParameters(LoaderBlock);

            // PRCB version mismatch: ntoskrnl was built against a different
            // header revision than this HAL.
            let major = (*prcb).MajorVersion;
            if major != PRCB_MAJOR_VERSION {
                KeBugCheckEx(MISMATCHED_HAL, 1, major as _, PRCB_MAJOR_VERSION as _, 0);
            }
            // Checked/free HAL requires checked/free kernel.  PRCB.BuildType
            // is USHORT on i386 and UCHAR on amd64; widen to USHORT for the
            // comparison against HalpBuildType (USHORT on both).
            let build: u16 = (*prcb).BuildType as u16;
            let hal_build = core::ptr::read_volatile(&raw const HalpBuildType);
            if build != hal_build {
                KeBugCheckEx(MISMATCHED_HAL, 2, build as _, hal_build as _, 0);
            }

            // Phase 0 ACPI.
            let status = HalpSetupAcpiPhase0(LoaderBlock);
            if status < 0 {
                KeBugCheckEx(ACPI_BIOS_ERROR, status as _, 0, 0, 0);
            }

            HalpInitializePICs(1 /* TRUE */);
            // KeInitializeSpinLock is FORCEINLINE *p = 0 — inline directly.
            core::ptr::write_volatile(&raw mut HalpSystemHardwareLock as *mut usize, 0);
            HalpInitializeCmos();

            // Wire dispatch tables.  HalDispatchTable + HalPrivate-
            // DispatchTable live in ntoskrnl.exe and are imported via
            // raw-dylib (per postprocess_bindgen --hal).
            // Bindgen typed each field as `extern "stdcall"` on i386 (from
            // the `NTAPI` annotation on the typedef), but our Rust HAL
            // functions are declared `extern "system"`.  Both are the same
            // ABI (stdcall on i386, sysv on amd64), but Rust's type system
            // doesn't unify them — transmute through `as _` casts.  Same
            // for the C-side ntoskrnl exports we're stuffing into the
            // private dispatch table.
            HalDispatchTable.HalQuerySystemInformation = Some(core::mem::transmute(HaliQuerySystemInformation as usize));
            HalDispatchTable.HalSetSystemInformation = Some(core::mem::transmute(HaliSetSystemInformation as usize));
            HalDispatchTable.HalInitPnpDriver = Some(core::mem::transmute(HaliInitPnpDriver as usize));
            HalDispatchTable.HalGetDmaAdapter = Some(core::mem::transmute(HalpGetDmaAdapter as usize));
            HalDispatchTable.HalGetInterruptTranslator = None; // FIXME: TODO
            // INT10/VGA-BIOS display reset removed (no fast486 / x86bios). The
            // kernel's default HalResetDisplay is NULL (see halstub) and callers
            // null-check it; on a GOP framebuffer there is no text mode to reset.
            HalPrivateDispatchTable.HalResetDisplay = None;
            HalPrivateDispatchTable.HalHaltSystem = Some(core::mem::transmute(HaliHaltSystem as usize));

            // Stitch the default I/O space into the address-usage list.
            (*(&raw mut HalpDefaultIoSpace)).Next = HalpAddressUsageList;
            core::ptr::write_volatile(&raw mut HalpAddressUsageList, &raw mut HalpDefaultIoSpace);

            HalpCalibrateStallExecution();
            HalpInitializeClock();

            // Could be rebooting with a pending profile interrupt;
            // clear it before interrupts come on.
            HalStopProfileInterrupt(ProfileTime);

            HalpInitPhase0(LoaderBlock);
        } else if BootPhase == 1 {
            ntkrnl_sys::hal_ddk::HalpInitBusHandlers();
            HalpInitPhase1();
        }
    }
    1 /* TRUE */
}
