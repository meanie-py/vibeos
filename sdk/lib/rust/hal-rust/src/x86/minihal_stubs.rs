//! Mini-HAL stubs — replaces `hal/halx86/minihal/halinit.c`.
//!
//! When freeldr builds the mini HAL (`cargo build … --features minihal`),
//! these `#[no_mangle]` stubs take the place of the full-HAL real impls
//! living in `pic_halinit.rs`, `pic_pic.rs`, `halpcat.rs`, `dma.rs`, and
//! `bios.c`.  Those real impls are gated `#[cfg(not(feature = "minihal"))]`
//! so they vanish from `libhal_rust_mini.a`, letting these stubs win
//! without `--allow-multiple-definition`.
//!
//! Mini HAL is i386-only (freeldr) — gate the whole module accordingly.

#![cfg(all(target_arch = "x86", feature = "minihal"))]
#![allow(non_snake_case, non_upper_case_globals)]

use ntkrnl_sys::hal_ddk::{
    BOOLEAN, NTSTATUS, PDEVICE_DESCRIPTION, PDMA_ADAPTER, PLOADER_PARAMETER_BLOCK, PVOID, ULONG,
};

const STATUS_SUCCESS: NTSTATUS = 0;

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpInitProcessor(
    _ProcessorNumber: ULONG,
    _LoaderBlock: PLOADER_PARAMETER_BLOCK,
) {
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn HalpInitPhase0(_LoaderBlock: PLOADER_PARAMETER_BLOCK) {}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn HalpInitPhase1() {}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpSetupAcpiPhase0(
    _LoaderBlock: PLOADER_PARAMETER_BLOCK,
) -> NTSTATUS {
    STATUS_SUCCESS
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpInitializePICs(_EnableInterrupts: BOOLEAN) {}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpGetDmaAdapter(
    _Context: PVOID,
    _DeviceDescription: PDEVICE_DESCRIPTION,
    _NumberOfMapRegisters: *mut ULONG,
) -> PDMA_ADAPTER {
    core::ptr::null_mut()
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpBiosDisplayReset() -> BOOLEAN {
    0
}
