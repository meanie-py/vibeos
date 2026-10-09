//! TSC calibration + RDTSC-based stall / perf counter — translated from
//! `hal/halx86/apic/tsc.c`.  Compiled on the APIC flavor (`hal_apic`): amd64
//! always, i386 with the `apic` feature.
//!
//! Calibrates the TSC frequency by running the CMOS RTC at 1024 Hz, sampling
//! NUM_SAMPLES rdtsc readings via TscCalibrationISR (apic/tsccal.S), then
//! linear-regressing to get cycles-per-second.  Drives KeQueryPerformanceCounter
//! and KeStallExecutionProcessor.  The only per-arch delta is the IDT-gate write
//! (8-byte i386 gate vs 16-byte amd64 gate) and the eflags push width.

#![allow(non_snake_case, non_upper_case_globals, dead_code, unused_unsafe)]

use ntkrnl_sys::ddk::KeBugCheck;
use ntkrnl_sys::hal_ddk::{
    CLOCK_LEVEL, HalDisableSystemInterrupt, HalEnableSystemInterrupt, HalpReadCmos, HalpWriteCmos,
    HalPrivateDispatchTable, KIRQL, LARGE_INTEGER, PLARGE_INTEGER, PVOID, RTC_REGISTER_A,
    RTC_REGISTER_B, RTC_REGISTER_C, RTC_REG_B_PI, UCHAR, ULONG, ULONG_PTR, _KINTERRUPT_MODE_Latched,
    _KIPCR,
};

use crate::x86::pcr::{KeGetCurrentPrcb, KeGetPcr};

// ---------------------------------------------------------------------------
// Constants — matched against the C source / shared headers.
// ---------------------------------------------------------------------------

// Bug check 0x5C — HAL_INITIALIZATION_FAILED.  See
// sdk/include/reactos/mc/bugcodes.mc; same value used in fstub/halstub.rs.
const HAL_INITIALIZATION_FAILED: ULONG = 0x5C;

// Feature bit for RDTSC support — sdk/include/ndk/amd64/ketypes.h:32.
const KF_RDTSC: ULONG = 0x00000002;

// MSR_RDTSC — direct TSC write MSR (hal/halx86/apic/tsc.h:6).
const MSR_RDTSC: u32 = 0x10;

// APIC clock vector on amd64 (apic_halinit.rs / apicp.h).
const APIC_CLOCK_VECTOR: ULONG = 0xD1;

// NUM_SAMPLES — TSC calibration sample count (tsc.h:5).
const NUM_SAMPLES: usize = 4;

// RTC mode 6 = 1024 Hz.  SAMPLE_FREQUENCY = (32768 << 1) >> RTC_MODE = 1024.
const RTC_MODE: UCHAR = 6;
const SAMPLE_FREQUENCY: u32 = (32768u32 << 1) >> RTC_MODE;

// INITIAL_STALL_COUNT — sdk/include/ndk/amd64/ketypes.h:347.  Seeds
// HalpCpuClockFrequency before calibration completes.
const INITIAL_STALL_COUNT: i64 = 100;

// ---------------------------------------------------------------------------
// Locally inlined intrinsics — clang `__readeflags`/`_disable`/etc. have no
// linkable symbol; replicate inline.
// ---------------------------------------------------------------------------

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn read_eflags() -> ULONG_PTR {
    let f: ULONG_PTR;
    core::arch::asm!(
        "pushfq",
        "pop {0}",
        out(reg) f,
        options(nomem, preserves_flags),
    );
    f
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn write_eflags(f: ULONG_PTR) {
    core::arch::asm!(
        "push {0}",
        "popfq",
        in(reg) f,
        options(nomem),
    );
}

#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn read_eflags() -> ULONG_PTR {
    let f: ULONG_PTR;
    core::arch::asm!("pushfd", "pop {0:e}", out(reg) f, options(nomem, preserves_flags));
    f
}

#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn write_eflags(f: ULONG_PTR) {
    core::arch::asm!("push {0:e}", "popfd", in(reg) f, options(nomem));
}

#[inline(always)]
unsafe fn cli() {
    core::arch::asm!("cli", options(nomem, nostack));
}

#[inline(always)]
unsafe fn sti() {
    core::arch::asm!("sti", options(nomem, nostack));
}

#[inline(always)]
unsafe fn rdtsc() -> u64 {
    let lo: u32;
    let hi: u32;
    core::arch::asm!(
        "rdtsc",
        out("eax") lo,
        out("edx") hi,
        options(nomem, nostack, preserves_flags),
    );
    ((hi as u64) << 32) | (lo as u64)
}

#[inline(always)]
unsafe fn port_in(port: u16) -> u8 {
    let v: u8;
    core::arch::asm!("in al, dx", in("dx") port, out("al") v, options(nomem, nostack, preserves_flags));
    v
}

#[inline(always)]
unsafe fn port_out(port: u16, v: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") v, options(nomem, nostack, preserves_flags));
}

/// TSC rate measured against a POLLED PIT channel-2 one-shot (~50 ms), as Linux's
/// `pit_calibrate_tsc` does. The RTC-interrupt calibration below samples four ticks of
/// 1/1024 s; under QEMU TCG a late tick inflates one sample by up to 1.5x, and three
/// boots of one image calibrated 4.23, 4.71 and 5.63 GHz against a ~4.07 GHz TSC. QPC
/// is the raw TSC at this rate, so clock() ran 4-28% slow against system time (Wine's
/// ucrtbase misc.c:1355). Reading the 8254 involves no interrupt, so nothing can arrive
/// late, and 50 ms makes a few cycles of read latency at either end irrelevant.
/// `None` if the counter never reaches terminal count.
unsafe fn calibrate_tsc_pit() -> Option<u64> {
    const PIT_HZ: u64 = 1_193_182;
    const TICKS: u16 = 59_659; // 50 ms
    // Port 0x61: bit 0 gates channel 2, bit 1 drives the speaker (keep it off), bit 5
    // reads channel 2's OUT.
    let p61 = port_in(0x61);
    port_out(0x61, (p61 & !0x02) | 0x01);
    port_out(0x43, 0xB0); // channel 2, lobyte/hibyte, mode 0 (interrupt on terminal count)
    port_out(0x42, (TICKS & 0xff) as u8);
    port_out(0x42, (TICKS >> 8) as u8);
    let start = rdtsc();
    let mut end = start;
    let mut done = false;
    // Bounded: 2^24 polls is seconds even under TCG, never a hang.
    for _ in 0..(1u32 << 24) {
        if port_in(0x61) & 0x20 != 0 {
            end = rdtsc();
            done = true;
            break;
        }
    }
    port_out(0x61, p61);
    if !done || end <= start {
        return None;
    }
    Some((end - start) * PIT_HZ / TICKS as u64)
}

#[inline(always)]
unsafe fn write_msr(msr: u32, value: u64) {
    let lo = value as u32;
    let hi = (value >> 32) as u32;
    core::arch::asm!(
        "wrmsr",
        in("ecx") msr,
        in("eax") lo,
        in("edx") hi,
        options(nomem, nostack, preserves_flags),
    );
}

#[inline(always)]
unsafe fn read_write_barrier() {
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
}

// ---------------------------------------------------------------------------
// Externs — apic/tsccal.S provides TscCalibrationISR.
// ---------------------------------------------------------------------------

unsafe extern "C" {
    fn TscCalibrationISR();
}

// ---------------------------------------------------------------------------
// IDT helpers — KeRegisterInterruptHandler / KeQueryInterruptHandler are
// FORCEINLINE in `ntoskrnl/include/internal/amd64/ke.h` (no linkable symbol).
// Inline the bodies locally; same shape as `usage.rs::register_interrupt_handler`.
// ---------------------------------------------------------------------------

#[inline(always)]
unsafe fn vector_to_idt_entry(vector: ULONG) -> usize {
    let dispatch = &raw const HalPrivateDispatchTable;
    let vec_to_idt = (*dispatch).HalVectorToIDTEntry.unwrap_unchecked();
    vec_to_idt(vector) as usize
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn register_interrupt_handler(vector: ULONG, handler: PVOID) {
    let pcr = crate::x86::pcr::KeGetPcr() as *mut _KIPCR;
    let entry = vector_to_idt_entry(vector);
    let idt = (*pcr).IdtBase.add(entry);
    let inner = &mut (*idt).__bindgen_anon_1;
    let address = handler as usize;
    inner.OffsetLow = (address & 0xffff) as u16;
    inner.OffsetMiddle = ((address >> 16) & 0xffff) as u16;
    inner.OffsetHigh = (address >> 32) as u32;
    const KGDT64_R0_CODE: u16 = 0x0010;
    inner.Selector = KGDT64_R0_CODE;
    inner.set_IstIndex(0);
    inner.set_Type(0x0e);
    inner.set_Dpl(0);
    inner.set_Present(1);
    inner.set_Reserved0(0);
    inner.Reserved1 = 0;
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn query_interrupt_handler(vector: ULONG) -> PVOID {
    let pcr = crate::x86::pcr::KeGetPcr() as *mut _KIPCR;
    let entry = vector_to_idt_entry(vector);
    let idt = (*pcr).IdtBase.add(entry);
    let inner = &(*idt).__bindgen_anon_1;
    let address: u64 = ((inner.OffsetHigh as u64) << 32)
        | ((inner.OffsetMiddle as u64) << 16)
        | (inner.OffsetLow as u64);
    address as PVOID
}

// i386: 8-byte KIDTENTRY — patch only the split 32-bit offset (kernel pre-set
// the selector + gate type).  Same shape as apic.rs's register_interrupt_handler.
#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn register_interrupt_handler(vector: ULONG, handler: PVOID) {
    let pcr = crate::x86::pcr::KeGetPcr() as *mut _KIPCR;
    let entry = vector_to_idt_entry(vector) as usize;
    let e = &mut *(*pcr).IDT.add(entry);
    let address = handler as usize;
    e.Offset = (address & 0xffff) as u16;
    e.ExtendedOffset = ((address >> 16) & 0xffff) as u16;
}

#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn query_interrupt_handler(vector: ULONG) -> PVOID {
    let pcr = crate::x86::pcr::KeGetPcr() as *mut _KIPCR;
    let entry = vector_to_idt_entry(vector) as usize;
    let e = &*(*pcr).IDT.add(entry);
    let address = ((e.ExtendedOffset as u32) << 16) | (e.Offset as u32);
    address as usize as PVOID
}

// ---------------------------------------------------------------------------
// Globals — TscCalibrationArray + TscCalibrationPhase are written by the
// asm ISR; HalpCpuClockFrequency is read by apictimer.rs.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub static mut HalpCpuClockFrequency: LARGE_INTEGER = LARGE_INTEGER {
    QuadPart: INITIAL_STALL_COUNT * 1_000_000,
};

#[unsafe(no_mangle)]
pub static mut TscCalibrationPhase: UCHAR = 0;

#[unsafe(no_mangle)]
pub static mut TscCalibrationArray: [u64; NUM_SAMPLES] = [0; NUM_SAMPLES];

// ---------------------------------------------------------------------------
// DoLinearRegression — same simplified linear regression as the C source.
// ---------------------------------------------------------------------------

unsafe fn do_linear_regression(x_max: ULONG, array_y: *const u64) -> u64 {
    let sum_xx: ULONG = (x_max * (x_max + 1) * (2 * x_max + 1)) / 6;
    let mut sum_xy: u64 = 0;
    for x in 1..=x_max {
        sum_xy += (x as u64) * (*array_y.add(x as usize) - *array_y);
    }
    sum_xy *= SAMPLE_FREQUENCY as u64;
    (sum_xy + (sum_xx as u64 / 2)) / sum_xx as u64
}

// ---------------------------------------------------------------------------
// HalpInitializeTsc — runs RTC at 1024 Hz, samples NUM_SAMPLES via the asm
// ISR, then linear-regresses to get cycles per second.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpInitializeTsc() {
    // Check the CPU supports RDTSC.
    let prcb = KeGetCurrentPrcb();
    if ((*prcb).FeatureBits & KF_RDTSC) == 0 {
        KeBugCheck(HAL_INITIALIZATION_FAILED);
    }

    // Save flags and disable interrupts.
    let flags = read_eflags();
    cli();

    // Enable the periodic interrupt in CMOS.
    let register_b = HalpReadCmos(RTC_REGISTER_B as UCHAR);
    HalpWriteCmos(RTC_REGISTER_B as UCHAR, register_b | RTC_REG_B_PI as UCHAR);

    // Modify register A to RTC_MODE to get SAMPLE_FREQUENCY.
    let mut register_a = HalpReadCmos(RTC_REGISTER_A as UCHAR);
    register_a = (register_a & 0xF0) | RTC_MODE;
    HalpWriteCmos(RTC_REGISTER_A as UCHAR, register_a);

    // Save old IDT entry.
    let previous_handler = query_interrupt_handler(APIC_CLOCK_VECTOR);

    // Set the calibration ISR.
    register_interrupt_handler(APIC_CLOCK_VECTOR, TscCalibrationISR as PVOID);

    // Reset TSC value to 0.
    write_msr(MSR_RDTSC, 0);

    // Enable the timer interrupt.
    HalEnableSystemInterrupt(APIC_CLOCK_VECTOR, CLOCK_LEVEL as KIRQL, _KINTERRUPT_MODE_Latched);

    // Read register C, so the next interrupt can happen.
    HalpReadCmos(RTC_REGISTER_C as UCHAR);

    // Wait for completion.
    sti();
    while TscCalibrationPhase < NUM_SAMPLES as UCHAR {
        read_write_barrier();
    }
    cli();

    // Disable the periodic interrupt in CMOS.
    HalpWriteCmos(RTC_REGISTER_B as UCHAR, register_b & !(RTC_REG_B_PI as UCHAR));

    // Disable the timer interrupt.
    HalDisableSystemInterrupt(APIC_CLOCK_VECTOR, CLOCK_LEVEL as KIRQL);

    // Restore the previous handler.
    register_interrupt_handler(APIC_CLOCK_VECTOR, previous_handler);

    // Calculate average via simplified linear regression.
    HalpCpuClockFrequency.QuadPart =
        do_linear_regression(
            (NUM_SAMPLES - 1) as ULONG,
            (&raw const TscCalibrationArray) as *const u64,
        ) as i64;

    // Prefer the polled PIT measurement (see calibrate_tsc_pit); the RTC regression is
    // the fallback, and both are named so a bad clock rate is visible in the boot log --
    // QPC on the APIC HAL is the raw TSC at this rate.
    {
        unsafe extern "C" {
            fn DbgPrint(Format: *const u8, ...) -> i32;
        }
        let rtc_khz = (HalpCpuClockFrequency.QuadPart / 1000) as u32;
        match calibrate_tsc_pit() {
            Some(hz) => {
                HalpCpuClockFrequency.QuadPart = hz as i64;
                DbgPrint(
                    b"[HAL] TSC %u kHz (PIT, 50 ms); RTC 4-tick estimate was %u kHz\n\0".as_ptr(),
                    (hz / 1000) as u32,
                    rtc_khz,
                );
            }
            None => {
                DbgPrint(
                    b"[HAL] TSC %u kHz from the RTC estimate: PIT channel 2 never reached \
                      terminal count, so QPC may drift\n\0"
                        .as_ptr(),
                    rtc_khz,
                );
            }
        }
    }

    // Restore flags.
    write_eflags(flags);
}

// ---------------------------------------------------------------------------
// HalpCalibrateStallExecution — run TSC calibration, set PCR.StallScaleFactor.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpCalibrateStallExecution() {
    HalpInitializeTsc();
    (*KeGetPcr()).StallScaleFactor = (HalpCpuClockFrequency.QuadPart / 1_000_000) as ULONG;
}

// ---------------------------------------------------------------------------
// KeQueryPerformanceCounter — exported, called from drivers.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn KeQueryPerformanceCounter(
    PerformanceFrequency: PLARGE_INTEGER,
) -> LARGE_INTEGER {
    ntkrnl_sys::ASSERT!(HalpCpuClockFrequency.QuadPart != 0);

    if !PerformanceFrequency.is_null() {
        core::ptr::write_volatile(PerformanceFrequency, HalpCpuClockFrequency);
    }

    LARGE_INTEGER {
        QuadPart: rdtsc() as i64,
    }
}

// ---------------------------------------------------------------------------
// KeStallExecutionProcessor — exported, busy-spin on TSC for `MicroSeconds`.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn KeStallExecutionProcessor(MicroSeconds: ULONG) {
    let start = rdtsc();
    let end = start + (*KeGetPcr()).StallScaleFactor as u64 * MicroSeconds as u64;
    while rdtsc() < end {}
}

// ---------------------------------------------------------------------------
// HalCalibratePerformanceCounter — UNIMPLEMENTED stub.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalCalibratePerformanceCounter(
    _Count: *mut i32, // volatile PLONG
    _NewCount: u64,
) {
    ntkrnl_sys::ASSERT!(false, "HalCalibratePerformanceCounter UNIMPLEMENTED");
}
