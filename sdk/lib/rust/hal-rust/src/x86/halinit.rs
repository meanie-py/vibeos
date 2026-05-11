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
// directly instead of importing.  KeGetCurrentPrcb is a per-arch
// inline reading fs:[0x1C] / gs:[0x18]; bindgen mistypes it on i386
// (NTAPI cc dropped) and the PRCB sanity asserts it's used for are
// nice-to-have — skip them.  KeBugCheckEx resolves fine via the
// existing import library.
use ntkrnl_sys::ddk::KeBugCheckEx;
use ntkrnl_sys::hal_ddk::{
    HalDispatchTable, HalInitializeBios, HalPrivateDispatchTable, HalStopProfileInterrupt,
    HalpActiveProcessors, HalpAddressUsageList, HalpBiosDisplayReset, HalpBusType,
    HalpCalibrateStallExecution, HalpDefaultInterruptAffinity, HalpDefaultIoSpace,
    HalpGetDmaAdapter, HalpInitPhase0, HalpInitPhase1, HalpInitProcessor,
    HalpInitializeClock, HalpInitializeCmos, HalpInitializePICs,
    HalpRegisterKdSupportFunctions, HalpSetupAcpiPhase0, HalpSystemHardwareLock,
    HaliHaltSystem, HaliInitPnpDriver, BOOLEAN, PLOADER_PARAMETER_BLOCK, ULONG,
    _KPROFILE_SOURCE_ProfileTime as ProfileTime,
};
use crate::x86::sysinfo::{HaliQuerySystemInformation, HaliSetSystemInformation};

// Bugcheck code for ACPI init failure (sdk/include/reactos/mc/bugcodes.h).
const ACPI_BIOS_ERROR: ULONG = 0xa5;

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

        // C version: `KeGetPcr()->StallScaleFactor = INITIAL_STALL_COUNT;`
        // KeGetPcr is a per-arch inline reading fs:[0x1C] / gs:[0x18].
        // Skipped — bindgen mistypes it (NTAPI dropped) and on the i386
        // and amd64 paths KiInitializeKernel already sets StallScaleFactor
        // to INITIAL_STALL_COUNT before HalInitializeProcessor runs.
        // TODO: lift KeGetPcr into ntkrnl-sys::inline and restore.

        // Track this processor in both HAL affinity masks.  The C uses
        // InterlockedBitTestAndSetAffinity macros; the underlying op is
        // just `lock bts` against a KAFFINITY.  Use the Rust atomic ops.
        let mask = 1u64 << (ProcessorNumber as u64);
        let active = &raw mut HalpActiveProcessors as *mut u64;
        let default_aff = &raw mut HalpDefaultInterruptAffinity as *mut u64;
        let active_atomic = &*(active as *const core::sync::atomic::AtomicU64);
        let default_atomic = &*(default_aff as *const core::sync::atomic::AtomicU64);
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
        if BootPhase == 0 {
            // Stash the bus type (PC-AT, EISA, MCA, PC-98, ...).  The
            // I386 union member is reused on amd64 too — both PE arches
            // share that field at the same offset.
            let machine_type = (*LoaderBlock).u.I386.MachineType & 0xFF;
            core::ptr::write_volatile(&raw mut HalpBusType, machine_type);

            HalpGetParameters(LoaderBlock);

            // PRCB version/build-type sanity asserts skipped — KeGetCurrentPrcb
            // is bindgen-mistyped on i386 and the asserts have never fired
            // since they're checking that ntoskrnl built against the same
            // header revision as the HAL.  TODO: lift KeGetCurrentPrcb into
            // ntkrnl-sys::inline (per-arch fs:/gs: deref) and restore.

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
            HalPrivateDispatchTable.HalResetDisplay = Some(core::mem::transmute(HalpBiosDisplayReset as usize));
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
            HalInitializeBios(0, LoaderBlock);
        } else if BootPhase == 1 {
            ntkrnl_sys::hal_ddk::HalpInitBusHandlers();
            HalpInitPhase1();
            HalInitializeBios(1, LoaderBlock);
        }
    }
    1 /* TRUE */
}
