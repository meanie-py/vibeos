// PROJECT:     ReactOS Kernel (Rust port)
// PURPOSE:     amd64 multiprocessor startup — KeStartAllProcessors
//
// This is a PORT of the working i386 APINFO approach
// (ntoskrnl/ke/i386/mproc.c, Justin Miller / Victor Perevertkin, 2023) to
// amd64. Upstream ReactOS never implemented amd64 KeStartAllProcessors (it was
// an UNIMPLEMENTED stub); this file supplies the real body.
//
// See DESIGN.md (this directory) for the port rationale, the amd64-vs-i386
// deltas, the HAL contract assumptions, and the integrator checklist.
//
// Gating: the real body is `#[cfg(feature = "smp")]`. Under
// `#[cfg(not(feature = "smp"))]` KeStartAllProcessors is a no-op (the BSP is the
// only CPU) — it must NOT panic in the UP build.

// ---------------------------------------------------------------------------
// UP build: no-op. (Do not panic — the UP kernel calls this unconditionally.)
// ---------------------------------------------------------------------------
#[cfg(not(feature = "smp"))]
#[no_mangle]
pub unsafe extern "system" fn KeStartAllProcessors() {
    // Uniprocessor build — nothing to start. BSP is already running.
}

// ---------------------------------------------------------------------------
// SMP build: the real implementation.
// ---------------------------------------------------------------------------
#[cfg(feature = "smp")]
pub use smp_impl::KeStartAllProcessors;

#[cfg(feature = "smp")]
mod smp_impl {
    use crate::ke::ke_defs::*;
    use ntkrnl_sys::globals::*;
    use ntkrnl_sys::base::*;
    use ntkrnl_sys::constants::*;
    use ntkrnl_sys::macros::*;
    use ntkrnl_sys::ke::*;
    use ntkrnl_sys::mm::*;
    use ntkrnl_sys::rtl::*;
    use ntkrnl_sys::ex::*;

    use ntkrnl_sys::ddk::{
        _KIDTENTRY64,
        _KGDTENTRY64,
        _KIPCR,
        _ETHREAD,
        _KTSS64,
        _KPROCESSOR_STATE,
        PKPROCESSOR_STATE,
        PKTHREAD,
        ExAllocatePoolZero,
        HalStartNextProcessor,
        MmCreateKernelStack,
        MmDeleteKernelStack,
    };

    // Rust-defined kernel globals — imported (not extern'd) so the LTO=false
    // firewall keeps a single definition, matching kiinit.rs / i386 mproc.rs.
    use crate::ke::krnlinit::KeLoaderBlock;
    use crate::ke::processor::{KeMaximumProcessors, KeNumprocSpecified, KeBootprocSpecified};

    // Size of the shared double-fault / NMI IST stack per AP.
    // nolint: shadowed-c-define (i386's 0x3000, above amd64 ketypes.h's 0x2000; larger is safe)
    const DOUBLE_FAULT_STACK_SIZE: usize = 0x3000;

    // amd64 GDT selectors (sdk/include/ndk/amd64/ketypes.h — KGDT64_*).
    const KGDT64_R0_CODE: u16 = 0x0010;
    const KGDT64_R0_DATA: u16 = 0x0018;
    const KGDT64_R3_DATA: u16 = 0x0028;
    const KGDT64_SYS_TSS: u16 = 0x0040;

    const INITIAL_MXCSR: u32 = 0x1F80;
    const EFLAGS_INTERRUPT_MASK: u64 = 0x200;

    // ------------------------------------------------------------------
    // Per-AP structure block. Mirrors the i386 APINFO, adjusted for amd64:
    //   * KIDTENTRY64/KGDTENTRY64 are 16 bytes each (vs 8 on i386).
    //   * A single KTSS64 (no separate DF/NMI task gates — amd64 uses IST
    //     stacks inside the one TSS, exactly like the P0 boot path).
    //   * One shared double-fault/NMI IST stack region.
    // GDT/IDT are page-aligned like i386 (DECLSPEC_ALIGN(PAGE_SIZE)); on amd64
    // that alignment is defensive, not architecturally required — see DESIGN.md.
    // ------------------------------------------------------------------
    #[repr(C, align(4096))]
    struct APINFO {
        Idt: [_KIDTENTRY64; 256],
        // 256 * 16 == PAGE_SIZE, so this pad is zero-length; it also acts as a
        // compile-time tripwire if KIDTENTRY64 ever changes size.
        _pad_after_idt: [u8; PAGE_SIZE - (core::mem::size_of::<_KIDTENTRY64>() * 256)],
        Gdt: [_KGDTENTRY64; 128],
        _pad_after_gdt: [u8; PAGE_SIZE - (core::mem::size_of::<_KGDTENTRY64>() * 128)],
        NMIStackData: core::mem::MaybeUninit<[u8; DOUBLE_FAULT_STACK_SIZE]>,
        Pcr: _KIPCR,
        Thread: _ETHREAD,
        Tss: _KTSS64,
    }

    // KDESCRIPTOR shape for SGDT/SIDT — the 10-byte {limit:u16, base:u64} pair
    // is written at offset 6 (Limit) / 8 (Base) so Base stays 8-aligned.
    // Identical layout to kiinit.rs's local KDESCRIPTOR.
    #[repr(C)]
    struct KDESCRIPTOR {
        Pad: [u16; 3],
        Limit: u16,
        Base: *mut u8,
    }

    extern "system" {
        // Provided by kiinit.rs (#[no_mangle]). Sets up the PCR (GdtBase/IdtBase/
        // TssBase, PRCB version/number/set, CurrentThread, Irql) AND the TSS:
        // zeroes the TSS, sets Rsp0 = KernelStack, Ist[1..3], and writes the
        // 16-byte AMD64_TSS system descriptor into GdtBase[KGDT64_SYS_TSS].
        // Pointer args are declared as *mut u8 (ABI-identical to the concrete
        // pointer types in kiinit.rs).
        fn KiInitializeProcessorBootStructures(
            ProcessorNumber: u32,
            Pcr: *mut _KIPCR,
            GdtBase: *mut u8,
            IdtBase: *mut u8,
            TssBase: *mut u8,
            IdleThread: PKTHREAD,
            KernelStack: PVOID,
            DpcStack: PVOID,
            DoubleFaultStack: PVOID,
            NmiStack: PVOID,
        );

        // The AP's entry point. #[no_mangle] in kiinit.rs; we only need its
        // address to seed ProcessorState.ContextFrame.Rip.
        fn KiSystemStartup(LoaderBlock: PVOID) -> !;
    }

    #[inline(always)]
    unsafe fn readcr0() -> u64 {
        let v: u64;
        core::arch::asm!("mov {}, cr0", out(reg) v, options(nomem, nostack));
        v
    }
    #[inline(always)]
    unsafe fn readcr3() -> u64 {
        let v: u64;
        core::arch::asm!("mov {}, cr3", out(reg) v, options(nomem, nostack));
        v
    }
    #[inline(always)]
    unsafe fn readcr4() -> u64 {
        let v: u64;
        core::arch::asm!("mov {}, cr4", out(reg) v, options(nomem, nostack));
        v
    }
    #[inline(always)]
    unsafe fn readeflags() -> u64 {
        let v: u64;
        core::arch::asm!("pushfq; pop {}", out(reg) v, options(nomem));
        v
    }

    // CODE_SEG("INIT")
    #[no_mangle]
    pub unsafe extern "system" fn KeStartAllProcessors() {
        let mut kernel_stack: PVOID = core::ptr::null_mut();
        let mut dpc_stack: PVOID = core::ptr::null_mut();
        let mut ap_info: *mut APINFO = core::ptr::null_mut();

        let mut maximum_processors: u32 = KeMaximumProcessors;
        if KeNumprocSpecified != 0 {
            maximum_processors = core::cmp::min(maximum_processors, KeNumprocSpecified);
        }
        if KeBootprocSpecified != 0 {
            maximum_processors = core::cmp::min(maximum_processors, KeBootprocSpecified);
        }

        crate::dbg_print!(
            b"[SMP] KeStartAllProcessors: attempting up to %lu processors\n\0",
            maximum_processors
        );

        // Start at 1 — the boot CPU (P0) is already running.
        let mut processor_count: u32 = 1;
        while processor_count < maximum_processors {
            kernel_stack = core::ptr::null_mut();
            dpc_stack = core::ptr::null_mut();

            // Allocate the per-AP structure block (page-aligned via the pool's
            // large-allocation path — see DESIGN.md "pool alignment").
            ap_info = ExAllocatePoolZero(
                NonPagedPool,
                core::mem::size_of::<APINFO>() as _,
                TAG_KERNEL,
            ) as *mut APINFO;
            if ap_info.is_null() {
                crate::dbg_print!(b"[SMP] APINFO alloc failed; stopping\n\0");
                break;
            }

            kernel_stack = MmCreateKernelStack(0u8, 0);
            if kernel_stack.is_null() {
                crate::dbg_print!(b"[SMP] kernel-stack alloc failed; stopping\n\0");
                break;
            }

            dpc_stack = MmCreateKernelStack(0u8, 0);
            if dpc_stack.is_null() {
                crate::dbg_print!(b"[SMP] DPC-stack alloc failed; stopping\n\0");
                break;
            }

            crate::dbg_print!(
                b"[SMP] starting processor %lu (APINFO=%p kstack=%p dpc=%p)\n\0",
                processor_count,
                ap_info as PVOID,
                kernel_stack,
                dpc_stack
            );

            // Top of the shared double-fault / NMI IST stack (stacks grow down).
            let df_nmi_stack_top = (core::ptr::addr_of_mut!((*ap_info).NMIStackData) as *mut u8)
                .add(DOUBLE_FAULT_STACK_SIZE) as PVOID;

            // Copy the BSP's live GDT and IDT into this AP's block *before*
            // KiInitializeProcessorBootStructures — that call writes the AP's
            // TSS descriptor into the copied GDT, so the copy must happen first
            // or it would clobber the TSS entry.
            let mut bsp_gdt = KDESCRIPTOR { Pad: [0; 3], Limit: 0, Base: core::ptr::null_mut() };
            let mut bsp_idt = KDESCRIPTOR { Pad: [0; 3], Limit: 0, Base: core::ptr::null_mut() };
            core::arch::asm!("sgdt [{}]", in(reg) core::ptr::addr_of_mut!(bsp_gdt.Limit), options(nostack));
            core::arch::asm!("sidt [{}]", in(reg) core::ptr::addr_of_mut!(bsp_idt.Limit), options(nostack));

            RtlCopyMemory(
                core::ptr::addr_of_mut!((*ap_info).Gdt) as PVOID,
                bsp_gdt.Base as PVOID,
                (bsp_gdt.Limit as usize) + 1,
            );
            RtlCopyMemory(
                core::ptr::addr_of_mut!((*ap_info).Idt) as PVOID,
                bsp_idt.Base as PVOID,
                (bsp_idt.Limit as usize) + 1,
            );

            // Initialize the AP's PCR + PRCB and its TSS (Rsp0/IST + the TSS GDT
            // descriptor). On amd64 the PCR is located via GS base (set later by
            // the AP in KiInitializeCpu), NOT via a GDT selector — so, unlike
            // i386, there is NO KGDT_R0_PCR descriptor base to patch here.
            let pcr = core::ptr::addr_of_mut!((*ap_info).Pcr);
            KiInitializeProcessorBootStructures(
                processor_count,
                pcr,
                core::ptr::addr_of_mut!((*ap_info).Gdt) as *mut u8,
                core::ptr::addr_of_mut!((*ap_info).Idt) as *mut u8,
                core::ptr::addr_of_mut!((*ap_info).Tss) as *mut u8,
                core::ptr::addr_of_mut!((*ap_info).Thread.Tcb) as PKTHREAD,
                kernel_stack,
                dpc_stack,
                df_nmi_stack_top, // DoubleFaultStack (Ist[1], Ist[2])
                df_nmi_stack_top, // NmiStack        (Ist[3]) — shared, as on P0
            );

            // Fill the KPROCESSOR_STATE that HalStartNextProcessor loads into the
            // AP before it jumps to KiSystemStartup.
            let processor_state: PKPROCESSOR_STATE =
                core::ptr::addr_of_mut!((*pcr).Prcb.ProcessorState);
            RtlZeroMemory(
                processor_state as PVOID,
                core::mem::size_of::<_KPROCESSOR_STATE>(),
            );

            // Special registers: control regs + descriptor tables + TR.
            (*processor_state).SpecialRegisters.Cr0 = readcr0();
            (*processor_state).SpecialRegisters.Cr3 = readcr3();
            (*processor_state).SpecialRegisters.Cr4 = readcr4();

            (*processor_state).SpecialRegisters.Gdtr.Base =
                core::ptr::addr_of_mut!((*ap_info).Gdt) as PVOID;
            (*processor_state).SpecialRegisters.Gdtr.Limit =
                (core::mem::size_of_val(&(*ap_info).Gdt) - 1) as u16;
            (*processor_state).SpecialRegisters.Idtr.Base =
                core::ptr::addr_of_mut!((*ap_info).Idt) as PVOID;
            (*processor_state).SpecialRegisters.Idtr.Limit =
                (core::mem::size_of_val(&(*ap_info).Idt) - 1) as u16;

            (*processor_state).SpecialRegisters.Tr = KGDT64_SYS_TSS;
            (*processor_state).SpecialRegisters.Ldtr = 0;
            (*processor_state).SpecialRegisters.MxCsr = INITIAL_MXCSR;

            // Context frame: 64-bit long-mode entry state.
            (*processor_state).ContextFrame.MxCsr = INITIAL_MXCSR;
            (*processor_state).ContextFrame.SegCs = KGDT64_R0_CODE;
            (*processor_state).ContextFrame.SegSs = KGDT64_R0_DATA;
            (*processor_state).ContextFrame.SegDs = KGDT64_R3_DATA;
            (*processor_state).ContextFrame.SegEs = KGDT64_R3_DATA;
            (*processor_state).ContextFrame.SegFs = KGDT64_R3_DATA;
            (*processor_state).ContextFrame.SegGs = KGDT64_R0_DATA;
            (*processor_state).ContextFrame.EFlags =
                (readeflags() & !EFLAGS_INTERRUPT_MASK) as u32;

            // RSP = top of the AP boot stack. RIP = KiSystemStartup.
            (*processor_state).ContextFrame.Rsp = kernel_stack as u64;
            (*processor_state).ContextFrame.Rip = KiSystemStartup as usize as u64;

            // Windows x64 ABI: first argument in RCX. KiSystemStartup(LoaderBlock)
            // reads LoaderBlock->Prcb/Thread/Process/KernelStack. See DESIGN.md
            // "HAL contract" — this is the primary assumption about how
            // HalStartNextProcessor delivers the loader block to the AP.
            (*processor_state).ContextFrame.Rcx = KeLoaderBlock as u64;

            // Publish this AP's boot parameters into the shared loader block.
            // KiSystemStartup does Pcr = CONTAINING_RECORD(LoaderBlock->Prcb,
            // KIPCR, Prcb) — so Prcb must be the ADDRESS of the PRCB field.
            // Process is left as the BSP set it (KiInitialProcess) and persists.
            // (LOADER_PARAMETER_BLOCK fields are bindgen ULONG_PTR = u64.)
            (*KeLoaderBlock).KernelStack = kernel_stack as u64;
            (*KeLoaderBlock).Prcb = core::ptr::addr_of_mut!((*pcr).Prcb) as u64;
            (*KeLoaderBlock).Thread =
                core::ptr::addr_of_mut!((*ap_info).Thread.Tcb) as u64;

            crate::dbg_print!(
                b"[SMP] handing processor %lu to HAL (Rip=%p Rsp=%p Prcb=%p)\n\0",
                processor_count,
                KiSystemStartup as usize as PVOID,
                kernel_stack,
                core::ptr::addr_of_mut!((*pcr).Prcb) as PVOID
            );

            if HalStartNextProcessor(KeLoaderBlock as *mut _, processor_state) == 0 {
                crate::dbg_print!(
                    b"[SMP] HalStartNextProcessor returned FALSE for %lu; stopping\n\0",
                    processor_count
                );
                break;
            }

            // Wait for the AP to consume the loader block. The AP path in
            // KiSystemStartup (Cpu != 0) zeroes LoaderBlock->Prcb once it has
            // latched its PRCB. BOUNDED: a wedged AP (trampoline fault, SIPI
            // lost) must degrade to a loud serial report + continue as fewer
            // CPUs, not hang the BSP forever. ~10^9 pause iterations is
            // multiple seconds on any host — generous for QEMU's instant APs.
            let mut spins: u64 = 0;
            const AP_START_SPIN_LIMIT: u64 = 1_000_000_000;
            while core::ptr::read_volatile(core::ptr::addr_of!((*KeLoaderBlock).Prcb)) != 0 {
                core::arch::asm!("pause", options(nomem, nostack));
                spins += 1;
                if spins >= AP_START_SPIN_LIMIT {
                    break;
                }
            }
            if spins >= AP_START_SPIN_LIMIT {
                crate::dbg_print!(
                    b"[SMP] AP %lu NEVER SIGNALED (trampoline fault / SIPI lost?) - continuing without it\n\0",
                    processor_count
                );
                break;
            }

            crate::dbg_print!(b"[SMP] processor %lu online\n\0", processor_count);

            processor_count += 1;
        }

        // The final iteration's AP either wasn't attempted or didn't start —
        // discount it and free that iteration's still-owned resources.
        processor_count = processor_count.saturating_sub(1);

        if !ap_info.is_null() {
            ExFreePoolWithTag(ap_info as PVOID, TAG_KERNEL);
        }
        if !kernel_stack.is_null() {
            MmDeleteKernelStack(kernel_stack, 0u8);
        }
        if !dpc_stack.is_null() {
            MmDeleteKernelStack(dpc_stack, 0u8);
        }

        crate::dbg_print!(
            b"[SMP] KeStartAllProcessors: successful AP startup count is %lu\n\0",
            processor_count
        );
    }
}


// ============================================================================
// ORIGINAL C SOURCE (amd64 upstream — the stub this file replaces)
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Kernel
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Architecture specific source file to hold multiprocessor functions
//  * COPYRIGHT:   Copyright 2023 Justin Miller <justin.miller@reactos.org>
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS *****************************************************************/
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// KeStartAllProcessors(VOID)
// {
//     UNIMPLEMENTED;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
//
// The body above is a port of ntoskrnl/ke/i386/mproc.c (the same 2023
// Miller/Perevertkin APINFO design) to amd64. See DESIGN.md for the mapping.
