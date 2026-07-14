// PROJECT:     ReactOS Kernel (Rust port) — arm64 multiprocessor startup
// PURPOSE:     KeStartAllProcessors (smp-bringup-quest.md M5a).
//
// The kernel half of arm64 AP bring-up. Mirrors the amd64 mproc.rs contract
// but is far simpler: arm64 has no GDT/IDT/TSS, so the per-AP block collapses
// to a _KIPCR (with embedded PRCB) + a zeroed idle _ETHREAD + a kernel stack.
// The HAL half (PSCI CPU_ON + trampoline) is hal-rust/src/arm64/mpboot.rs.
//
// Flow (per AP, serialized by the LoaderBlock->Prcb handshake):
//   1. allocate APINFO (PCR+idle thread) + kernel stack
//   2. KiInitializePcr(&Pcr, cpu, &idle_thread)      (kiinit.rs, reused from P0)
//   3. publish KernelStack/Prcb/Thread into the shared KeLoaderBlock
//   4. HalStartNextProcessor -> PSCI CPU_ON; the AP reuses KiSystemStartup
//      (already `if Cpu == 0`-gated for global init) and zeroes LoaderBlock->Prcb
//      once it has latched its PRCB (the alive handshake)
//   5. bounded-wait on LoaderBlock->Prcb == 0; loud + stop if it never signals
//   6. HalStartNextProcessor returning FALSE = PSCI INVALID_PARAMETERS = "no
//      more CPUs" — end enumeration (no MADT parser needed)
//
// The idle _ETHREAD is handed to KiSystemStartup ZEROED — exactly as amd64
// mproc does, and amd64 -smp 2 proves that suffices (KiSystemStartupBootStack
// does the minimal per-CPU thread setup; KiInitializeKernel is Cpu==0-only).

// ---------------------------------------------------------------------------
// UP build: no-op (BSP is the only CPU; must not panic — called unconditionally).
// ---------------------------------------------------------------------------
#[cfg(not(feature = "smp"))]
#[no_mangle]
pub unsafe extern "system" fn KeStartAllProcessors() {}

#[cfg(feature = "smp")]
pub use smp_impl::KeStartAllProcessors;

#[cfg(feature = "smp")]
mod smp_impl {
    use ntkrnl_sys::base::*;
    use ntkrnl_sys::constants::*;
    use ntkrnl_sys::mm::NonPagedPool;
    use ntkrnl_sys::ddk::{
        _ETHREAD, _KIPCR, PKTHREAD,
        ExAllocatePoolZero, ExFreePoolWithTag,
        HalStartNextProcessor, MmCreateKernelStack, MmDeleteKernelStack,
    };
    use crate::ke::ke_defs::TAG_KERNEL;
    use crate::ke::krnlinit::KeLoaderBlock;
    use super::super::kiinit::{KiInitializePcr, KiSystemStartup};

    // Per-AP block. No GDT/IDT/TSS on arm64 — just the PCR (with its PRCB) and
    // a zeroed idle thread. Page-aligned so the PCR keeps its natural alignment.
    #[repr(C, align(4096))]
    struct APINFO {
        Pcr: _KIPCR,
        Thread: _ETHREAD,
    }

    #[no_mangle]
    pub unsafe extern "system" fn KeStartAllProcessors() {
        crate::dbg_print!(b"[SMP] arm64 KeStartAllProcessors: probing for APs via PSCI\n\0");

        let mut ap_info: *mut APINFO = core::ptr::null_mut();
        let mut kernel_stack: PVOID = core::ptr::null_mut();
        let mut processor_count: u32 = 1; // P0 already running

        loop {
            ap_info = ExAllocatePoolZero(
                NonPagedPool,
                core::mem::size_of::<APINFO>() as _,
                TAG_KERNEL,
            ) as *mut APINFO;
            if ap_info.is_null() {
                crate::dbg_print!(b"[SMP] arm64: APINFO alloc failed; stopping\n\0");
                break;
            }

            kernel_stack = MmCreateKernelStack(0u8, 0);
            if kernel_stack.is_null() {
                crate::dbg_print!(b"[SMP] arm64: kernel-stack alloc failed; stopping\n\0");
                break;
            }

            let pcr = core::ptr::addr_of_mut!((*ap_info).Pcr);
            let idle_thread =
                core::ptr::addr_of_mut!((*ap_info).Thread.Tcb) as PKTHREAD;

            // Init the AP's PCR/PRCB (Number, SetMember, CurrentThread, IRQL,
            // MP BuildType). Same helper the BSP's P0 path uses.
            KiInitializePcr(pcr, processor_count, idle_thread);

            // Publish this AP's boot params into the shared loader block. The AP
            // reads Prcb (CONTAINING_RECORD -> PCR), Thread, KernelStack; Process
            // stays as the BSP set it (KiInitialProcess) and persists.
            (*KeLoaderBlock).KernelStack = kernel_stack as u64;
            (*KeLoaderBlock).Prcb = core::ptr::addr_of_mut!((*pcr).Prcb) as u64;
            (*KeLoaderBlock).Thread = idle_thread as u64;

            // Seed the AP's entry (KiSystemStartup) + SP into ProcessorState so
            // the HAL delivers them without referencing the kernel symbol itself
            // (keeps the standalone HAL link clean — see mpboot.rs). The kernel
            // may reference KiSystemStartup: it's in this same binary.
            let processor_state = core::ptr::addr_of_mut!((*pcr).Prcb.ProcessorState);
            (*processor_state).ContextFrame.Pc = KiSystemStartup as usize as u64;
            (*processor_state).ContextFrame.Sp = kernel_stack as u64;

            crate::dbg_print!(
                b"[SMP] arm64: handing CPU %lu to HAL (Prcb=%p kstack=%p)\n\0",
                processor_count,
                core::ptr::addr_of_mut!((*pcr).Prcb) as PVOID,
                kernel_stack
            );

            if HalStartNextProcessor(KeLoaderBlock as *mut _, processor_state) == 0 {
                // PSCI INVALID_PARAMETERS / error: no more CPUs. Free this
                // iteration's unused resources and stop.
                ExFreePoolWithTag(ap_info as PVOID, TAG_KERNEL);
                MmDeleteKernelStack(kernel_stack, 0u8);
                ap_info = core::ptr::null_mut();
                kernel_stack = core::ptr::null_mut();
                break;
            }

            // Bounded wait for the AP to latch its PRCB and zero LoaderBlock->Prcb.
            // A wedged AP (trampoline fault) degrades to a loud report + continue
            // with fewer CPUs, never a BSP hang.
            let mut spins: u64 = 0;
            const AP_START_SPIN_LIMIT: u64 = 2_000_000_000;
            while core::ptr::read_volatile(core::ptr::addr_of!((*KeLoaderBlock).Prcb)) != 0 {
                core::arch::asm!("yield", options(nomem, nostack, preserves_flags));
                spins += 1;
                if spins >= AP_START_SPIN_LIMIT {
                    break;
                }
            }
            if spins >= AP_START_SPIN_LIMIT {
                crate::dbg_print!(
                    b"[SMP] arm64: CPU %lu NEVER SIGNALED (trampoline fault?) - continuing without it\n\0",
                    processor_count
                );
                // The AP owns ap_info/kernel_stack if it ever wakes — leak them
                // rather than risk a use-after-free racing a late AP.
                ap_info = core::ptr::null_mut();
                kernel_stack = core::ptr::null_mut();
                break;
            }

            crate::dbg_print!(b"[SMP] arm64: CPU %lu online\n\0", processor_count);
            processor_count += 1;
        }

        crate::dbg_print!(
            b"[SMP] arm64 KeStartAllProcessors: %lu processor(s) online\n\0",
            processor_count
        );
    }
}
