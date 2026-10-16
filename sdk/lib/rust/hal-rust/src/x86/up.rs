//! Uniprocessor (UP) variants of the SMP entry points. On a UP build
//! these are stubs: HalRequestIpi/HalSendNMI/HalSendSoftwareInterrupt
//! never have a target to send to, HalStartNextProcessor always
//! returns FALSE (no second processor), HalpBroadcastClockIpi /
//! HalpSetupProcessorsTable do nothing.
//!
//! Translated from `hal/halx86/generic/up.c`.
//!
//! SMP: on an amd64 `smp` HAL build these stubs are cfg'd OUT and the real
//! APIC implementations in smp_processor.rs take the symbols (mirrors the C
//! model where up.c and apic/apicsmp.c are never linked into the same HAL).

#![cfg_attr(
    all(feature = "smp", any(target_arch = "x86_64", all(target_arch = "x86", feature = "apic")), not(feature = "minihal")),
    allow(unused_imports)
)]

use ntkrnl_sys::hal_ddk::{
    BOOLEAN, KAFFINITY, PKPROCESSOR_STATE, PLOADER_PARAMETER_BLOCK, UCHAR,
};
#[cfg(target_arch = "x86_64")]
use ntkrnl_sys::hal_ddk::KIRQL;

/// hal.spec: `HalRequestIpi`. Should never be called on a UP system —
/// no target to send to. Trip the debugger if it happens.
#[cfg(any(not(feature = "smp"), all(target_arch = "x86", not(feature = "apic")), feature = "minihal"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalRequestIpi(_TargetProcessors: KAFFINITY) {
    // __debugbreak() — `int 3`
    unsafe { core::arch::asm!("int 3", options(nostack)); }
}

/// hal.spec: `HalStartNextProcessor`. Always returns FALSE on UP —
/// there is no next processor to start.
#[cfg(any(not(feature = "smp"), all(target_arch = "x86", not(feature = "apic")), feature = "minihal"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalStartNextProcessor(
    _LoaderBlock: PLOADER_PARAMETER_BLOCK,
    _ProcessorState: PKPROCESSOR_STATE,
) -> BOOLEAN {
    0 /* FALSE */
}

/// HAL-internal: `HalpSetupProcessorsTable`. UP no-op.
#[cfg(any(not(feature = "smp"), all(target_arch = "x86", not(feature = "apic")), feature = "minihal"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn HalpSetupProcessorsTable(_NTProcessorNumber: u32) {}

/// HAL-internal: `HalpBroadcastClockIpi`. UP no-op (no clock IPIs to
/// broadcast on a single-processor system). FASTCALL on i386 — caller
/// (apic/rtctimer.c:190) and definition both use FASTCALL per smp.h.
#[cfg(any(not(feature = "smp"), all(target_arch = "x86", not(feature = "apic")), feature = "minihal"))]
#[unsafe(no_mangle)]
pub unsafe extern "fastcall" fn HalpBroadcastClockIpi(_Vector: UCHAR) {}

// ============================================================================
// amd64-only entry points
// ============================================================================

/// hal.spec: `HalSendNMI`. UP no-op — no remote processor to NMI.
/// amd64-only per the original C `#ifdef _M_AMD64`.
#[cfg(target_arch = "x86_64")]
#[cfg(any(not(feature = "smp"), all(target_arch = "x86", not(feature = "apic")), feature = "minihal"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalSendNMI(_TargetSet: KAFFINITY) {}

/// hal.spec: `HalSendSoftwareInterrupt`. UP no-op.
/// amd64-only per the original C `#ifdef _M_AMD64`.
#[cfg(target_arch = "x86_64")]
#[cfg(any(not(feature = "smp"), all(target_arch = "x86", not(feature = "apic")), feature = "minihal"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalSendSoftwareInterrupt(
    _TargetSet: KAFFINITY,
    _Irql: KIRQL,
) {}
