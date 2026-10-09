use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
// ke globals referenced across the crate boundary as link symbols
// (all #[no_mangle] in ntoskrnl-rust ke/*). crate-split Phase 1b.
extern "C" {
    static mut KeLoaderBlock: ntkrnl_sys::ddk::PLOADER_PARAMETER_BLOCK;
    static mut KeNumberProcessors: core::ffi::c_char;
}
use crate::kdtrap::{KdpStub, KdpTrap};

use ntkrnl_sys::ddk::{
    _DBGKD_DEBUG_DATA_HEADER64,
    _KDDEBUGGER_DATA64,
    _KDPC,
    _KD_CONTEXT,
    _KTIMER,
    _KUSER_SHARED_DATA,
    _WORK_QUEUE_ITEM,
    BREAKPOINT_ENTRY,
    DBGKD_DEBUG_DATA_HEADER64,
    DBGKD_GET_VERSION64,
    KDDEBUGGER_DATA64,
    KD_CONTEXT,
    PDBGKD_DEBUG_DATA_HEADER64,
    PKDEBUG_ROUTINE,
    PLDR_DATA_TABLE_ENTRY,
    PLOADER_PARAMETER_BLOCK,
    PMEMORY_ALLOCATION_DESCRIPTOR,
    PSTR,
    ULONG64,
    KeInitializeDpc,
    KeInitializeTimer,
    KeQueryPerformanceCounter,
    MiIsMemoryTypeInvisible,
    PsGetCurrentProcessId,
    RtlInitString,
    STRING,
    KPCR, PKPCR,
};

#[cfg(not(target_arch = "x86"))]
use ntkrnl_sys::ddk::KeGetPcr;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn KeGetPcr() -> PKPCR {
    let pcr: PKPCR;
    core::arch::asm!("mov {}, dword ptr fs:[0x1C]", out(reg) pcr, options(nostack, preserves_flags));
    pcr
}

extern "C" {
    static mut KdDebuggerEnabled: BOOLEAN;
    static mut KiDebugRoutine: PKDEBUG_ROUTINE;
    static mut KdBreakAfterSymbolLoad: BOOLEAN;
    static mut KdpDebuggerDataListHead: LIST_ENTRY;
    static mut KdpDataSpinLock: ULONG_PTR; // KSPIN_LOCK
    static mut KdDebuggerDataBlock: KDDEBUGGER_DATA64;
    static mut KdVersionBlock: DBGKD_GET_VERSION64;
    static mut PsLoadedModuleList: LIST_ENTRY;
    static mut NtBuildNumber: ULONG;
    static mut PsNtosImageBase: ULONG_PTR;
    static mut KdPitchDebugger: BOOLEAN;
    static mut KdPerformanceCounterRate: LARGE_INTEGER;
    static mut KdpContext: KD_CONTEXT;
    static mut KdpOweBreakpoint: BOOLEAN;
    static mut KdpBreakpointTable: [BREAKPOINT_ENTRY; KD_BREAKPOINT_MAX];
    static mut KdpTimeSlipDpc: _KDPC;
    static mut KdpTimeSlipTimer: _KTIMER;
    static mut KdpTimeSlipWorkItem: _WORK_QUEUE_ITEM;
    static mut KdpDebuggerStructuresInitialized: BOOLEAN;
    static mut KdTimerStart: LARGE_INTEGER;
    static mut SharedUserData: *mut _KUSER_SHARED_DATA;
    static mut KdBlockEnable: BOOLEAN;
    static mut KdAutoEnableOnEvent: BOOLEAN;
    static mut KdIgnoreUmExceptions: BOOLEAN;
    static mut KdDebuggerNotPresent: BOOLEAN;
    static mut KeUserCallbackDispatcher: ULONG_PTR;
    fn _strupr(s: PSTR) -> PSTR;
    fn strstr(s1: PSTR, s2: PSTR) -> PSTR;
    fn strncmp(s1: PSTR, s2: PSTR, n: usize) -> i32;
    fn DbgPrint(Format: *const u8, ...) -> u32;
}

extern "system" {
    fn KdDebuggerInitialize0(LoaderBlock: PLOADER_PARAMETER_BLOCK) -> NTSTATUS;
    /// kd-rust kdio.rs (crash-value): init + arm the embedded kdcom-serial
    /// wire transport for real WinDbg KD on COM2.
    fn KdpWireTransportArm(LoaderBlock: PVOID) -> NTSTATUS;
    /// ntoskrnl-rust tripwire.rs: exact-token lookup in the fw_cfg tripwire
    /// blob (safe pre-Mm; returns 0 off-QEMU).
    fn VibeTripwireHasToken(needle: *const u8, len: usize) -> u8;
    fn KdDisableDebuggerWithLock(BlockEnable: BOOLEAN) -> NTSTATUS;
    fn KdPollBreakIn() -> BOOLEAN;
    fn DbgLoadImageSymbols(ImageName: *mut STRING, Base: PVOID, ProcessId: ULONG_PTR);
}

extern "system" {
    fn KdpTimeSlipDpcRoutine(Dpc: *mut _KDPC, Context: PVOID, Arg1: PVOID, Arg2: PVOID);
}
extern "system" {
    fn KdpTimeSlipWork(Context: PVOID);
}

const KDBG_TAG: ULONG = 0x4742444B;
const KD_BREAKPOINT_MAX: usize = 32;

// DBGKD_MAJOR_TYPES starts at DBGKD_MAJOR_NT = 0; 6 is DBGKD_MAJOR_TNT, so the version block
// told the debugger it was talking to a different OS.
const DBGKD_MAJOR_NT: u16 = ntkrnl_sys::ddk::_DBGKD_MAJOR_TYPES_DBGKD_MAJOR_NT as u16;
const DBGKD_VERS_FLAG_MP: u16 = 0x0001;

const DbgKdMinimumStateChange: ULONG = 0x00003030;
const DbgKdMaximumStateChange: ULONG = 0x00003033;
const DbgKdMinimumManipulate: ULONG = 0x00003130;
const DbgKdMaximumManipulate: ULONG = 0x00003161;

const LoaderBad: u32 = 2;

unsafe fn KdpGetMemorySizeInMBs(loader_block: PLOADER_PARAMETER_BLOCK) -> SIZE_T {
    let mut number_of_physical_pages: SIZE_T = 0;

    if loader_block.is_null() {
        number_of_physical_pages = MmNumberOfPhysicalPages;
    } else {
        let mut list_entry = (*loader_block).MemoryDescriptorListHead.Flink;
        while list_entry != core::ptr::addr_of_mut!((*loader_block).MemoryDescriptorListHead) as PLIST_ENTRY {
            let descriptor = (list_entry as *mut u8)
                .sub(core::mem::offset_of!(ntkrnl_sys::ddk::_MEMORY_ALLOCATION_DESCRIPTOR, ListEntry))
                as PMEMORY_ALLOCATION_DESCRIPTOR;

            if MiIsMemoryTypeInvisible((*descriptor).MemoryType) == 0 {
                if (*descriptor).MemoryType != LoaderBad {
                    number_of_physical_pages += (*descriptor).PageCount as SIZE_T;
                }
            }

            list_entry = (*list_entry).Flink;
        }
    }

    let total_bytes = number_of_physical_pages * PAGE_SIZE;
    let aligned_bytes = (total_bytes + 1024 * 1024 - 1) / (1024 * 1024) * (1024 * 1024);
    aligned_bytes / (1024 * 1024)
}

unsafe fn KdpPrintBanner() {
    let mem_size_mbs = KdpGetMemorySizeInMBs(KeLoaderBlock);

    // crate::kdp_dprintf!(b"-----------------------------------------------------\n\0");
    // crate::kdp_dprintf!(b"ReactOS (Build)\n\0");
    // crate::kdp_dprintf!(b"%u System Processor [%u MB Memory]\n\0", KeNumberProcessors, mem_size_mbs);

    if !KeLoaderBlock.is_null() {
        // crate::kdp_dprintf!(b"Command Line: %s\n\0", (*KeLoaderBlock).LoadOptions);
        // KdpDprintf(
            // b"ARC Paths: %s %s %s %s\n\0".as_ptr() as PSTR,
            // (*KeLoaderBlock).ArcBootDeviceName,
            // (*KeLoaderBlock).NtHalPathName,
            // (*KeLoaderBlock).ArcHalDeviceName,
            // (*KeLoaderBlock).NtBootPathName
        // );
    }
}

#[no_mangle]
pub unsafe extern "system" fn KdUpdateDataBlock() {
    KdDebuggerDataBlock.KeUserCallbackDispatcher = KeUserCallbackDispatcher as ULONG64;
}

#[no_mangle]
pub unsafe extern "system" fn KdRegisterDebuggerDataBlock(
    tag: ULONG,
    data_header: PDBGKD_DEBUG_DATA_HEADER64,
    size: ULONG,
) -> BOOLEAN {
    let mut old_irql: KIRQL = 0;

    // [guard] KeAcquireSpinLock(core::ptr::addr_of_mut!(KdpDataSpinLock), core::ptr::addr_of_mut!(old_irql));
    let spin_guard = unsafe { ntkrnl_sys::guards::SpinLockGuard::acquire(core::ptr::addr_of_mut!(KdpDataSpinLock)) };

    let mut next_entry = KdpDebuggerDataListHead.Flink;
    while next_entry != core::ptr::addr_of_mut!(KdpDebuggerDataListHead) as PLIST_ENTRY {
        let current_header = (next_entry as *mut u8)
            .sub(core::mem::offset_of!(_DBGKD_DEBUG_DATA_HEADER64, List))
            as PDBGKD_DEBUG_DATA_HEADER64;

        next_entry = (*next_entry).Flink;

        if current_header == data_header || (*current_header).OwnerTag == tag {
            // [guard-drop] KeReleaseSpinLock(core::ptr::addr_of_mut!(KdpDataSpinLock), old_irql);
            return 0u8;
        }
    }

    (*data_header).OwnerTag = tag;
    (*data_header).Size = size;

    InsertTailList(
        core::ptr::addr_of_mut!(KdpDebuggerDataListHead) as PLIST_ENTRY,
        core::ptr::addr_of_mut!((*data_header).List) as PLIST_ENTRY
    );
    // [guard-drop] KeReleaseSpinLock(core::ptr::addr_of_mut!(KdpDataSpinLock), old_irql);
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn KdInitSystem(
    boot_phase: ULONG,
    loader_block: PLOADER_PARAMETER_BLOCK,
) -> BOOLEAN {
    let loader_block = opaque_ptr(loader_block);
    let mut enable_kd: BOOLEAN;
    let mut disable_kd_after_init = 0u8;
    let mut block_enable = 0u8;

    if boot_phase != 0 {
        KeQueryPerformanceCounter(core::ptr::addr_of_mut!(KdPerformanceCounterRate) as *mut ntkrnl_sys::ddk::_LARGE_INTEGER);
        return 1u8;
    }

    if KdDebuggerEnabled != 0 {
        return 1u8;
    }

    KiDebugRoutine = Some(abi_cast!(KdpStub));
    KdBreakAfterSymbolLoad = 0u8;

    if (*core::ptr::addr_of!(KdpDebuggerDataListHead)).Flink.is_null() {
        InitializeListHead(core::ptr::addr_of_mut!(KdpDebuggerDataListHead));

        super::kddata::KdpInitDebuggerDataBlock();
        KdRegisterDebuggerDataBlock(
            KDBG_TAG,
            core::ptr::addr_of_mut!(KdDebuggerDataBlock) as PDBGKD_DEBUG_DATA_HEADER64,
            core::mem::size_of_val(&*(&raw const KdDebuggerDataBlock)) as ULONG
        );

        KdVersionBlock.MajorVersion = ((DBGKD_MAJOR_NT as u32) << 8 | (NtBuildNumber >> 28)) as USHORT;
        KdVersionBlock.MinorVersion = (NtBuildNumber & 0xFFFF) as USHORT;

        #[cfg(feature = "smp")]
        {
            KdVersionBlock.Flags |= DBGKD_VERS_FLAG_MP;
        }

        KdVersionBlock.PsLoadedModuleList = core::ptr::addr_of_mut!(PsLoadedModuleList) as ULONG64;
        KdVersionBlock.DebuggerDataList = core::ptr::addr_of_mut!(KdpDebuggerDataListHead) as ULONG64;

        KdVersionBlock.MaxStateChange = (DbgKdMaximumStateChange - DbgKdMinimumStateChange) as UCHAR;
        KdVersionBlock.MaxManipulate = (DbgKdMaximumManipulate - DbgKdMinimumManipulate) as UCHAR;
        KdVersionBlock.Unused[0] = 0;

        (*KeGetPcr()).KdVersionBlock = core::ptr::addr_of_mut!(KdVersionBlock) as PVOID;
    }

    if !loader_block.is_null() {
        let ldr_entry = ((*loader_block).LoadOrderListHead.Flink as *mut u8)
            .sub(core::mem::offset_of!(ntkrnl_sys::ddk::_LDR_DATA_TABLE_ENTRY, InLoadOrderLinks))
            as PLDR_DATA_TABLE_ENTRY;

        PsNtosImageBase = (*ldr_entry).DllBase as ULONG_PTR;
        KdVersionBlock.KernBase = (*ldr_entry).DllBase as ULONG64;

        let command_line = (*loader_block).LoadOptions;
        if !command_line.is_null() {
            _strupr(command_line);
            enable_kd = 0u8;

            if !strstr(command_line, b"CRASHDEBUG\0".as_ptr() as PSTR).is_null() {
                KdPitchDebugger = 0u8;
            } else if !strstr(command_line, b"NODEBUG\0".as_ptr() as PSTR).is_null() {
                KdPitchDebugger = 1u8;
            } else {
                let debug_line = strstr(command_line, b"DEBUG\0".as_ptr() as PSTR);
                if !debug_line.is_null() {
                    enable_kd = 1u8;

                    if *debug_line.add(5) == b'=' as i8 {
                        let mut debug_option_start = debug_line.add(6);
                        let mut debug_option_end = debug_option_start;

                        loop {
                            while *debug_option_end != 0 {
                                if *debug_option_end == b',' as i8
                                    || *debug_option_end == b' ' as i8
                                    || *debug_option_end == b'\t' as i8
                                {
                                    break;
                                } else {
                                    debug_option_end = debug_option_end.add(1);
                                }
                            }

                            let debug_option_length = debug_option_end.offset_from(debug_option_start) as usize;

                            if debug_option_length == 0 {
                                break;
                            }

                            if debug_option_length == 10
                                && strncmp(debug_option_start, b"AUTOENABLE\0".as_ptr() as PSTR, 10) == 0
                            {
                                disable_kd_after_init = 1u8;
                                block_enable = 0u8;
                                KdAutoEnableOnEvent = 1u8;
                            } else if debug_option_length == 7
                                && strncmp(debug_option_start, b"DISABLE\0".as_ptr() as PSTR, 7) == 0
                            {
                                disable_kd_after_init = 1u8;
                                block_enable = 1u8;
                                KdAutoEnableOnEvent = 0u8;
                            } else if debug_option_length == 6
                                && strncmp(debug_option_start, b"NOUMEX\0".as_ptr() as PSTR, 6) == 0
                            {
                                KdIgnoreUmExceptions = 1u8;
                            }

                            if *debug_option_end != b',' as i8 {
                                break;
                            }

                            debug_option_end = debug_option_end.add(1);
                            debug_option_start = debug_option_end;
                        }
                    }
                }
            }
        } else {
            KdPitchDebugger = 1u8;
            enable_kd = 0u8;
        }
    } else {
        KdVersionBlock.KernBase = PsNtosImageBase as ULONG64;
        enable_kd = 1u8;
    }

    KdDebuggerDataBlock.KernBase = KdVersionBlock.KernBase;

    // Historically a BLANKET enable_kd=0 on x86/amd64: the INT3→KdpTrap path
    // died in a recursive exception flood — which was the §9.35 class (zero
    // .pdata for Rust frames, fixed by requires-uwtable=true), so real KD is
    // viable again. It stays OPT-IN rather than default-on for a different
    // reason: COM1 is the smoke/console log channel, and a live KD claims its
    // port for binary packet framing. Opt in per-boot with
    // /DEBUG /DEBUGPORT=COM2 (WinDbg on a dedicated second UART); every other
    // /DEBUG spelling keeps today's KdpStub text-to-COM1 behavior.
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if !loader_block.is_null() && !(*loader_block).LoadOptions.is_null() {
        // DEBUGPORT=COM:0x<port> moves KdpStub's text console off COM1.
        let at = strstr((*loader_block).LoadOptions, b"DEBUGPORT=COM:\0".as_ptr() as PSTR);
        if !at.is_null() {
            let port = console_port(at.add(b"DEBUGPORT=COM:".len()));
            if port != 0 {
                crate::kdtrap::RtlpDebugConsolePort = port;
            }
        }
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        let mut kd_on_com2 = false;
        if !loader_block.is_null() {
            let command_line = (*loader_block).LoadOptions;
            if !command_line.is_null()
                && !strstr(command_line, b"DEBUGPORT=COM2\0".as_ptr() as PSTR).is_null()
            {
                kd_on_com2 = true;
            }
        }
        // Per-boot opt-in with no media rebuild: `--tripwire "KD=COM2"`
        // (fw_cfg — read directly here since KdInitSystem runs long before
        // the Mm-phase tripwire parse).
        if !kd_on_com2 && VibeTripwireHasToken(b"KD=COM2".as_ptr(), 7) != 0 {
            kd_on_com2 = true;
        }
        if !kd_on_com2 {
            enable_kd = 0u8;
        } else {
            DbgPrint(
                b"KD: DEBUGPORT=COM2 -- real KD transport opt-in (KiDebugRoutine=KdpTrap)\n\0"
                    .as_ptr(),
            );
            // Arm the embedded kdcom-serial wire transport (kd-rust kdio.rs
            // delegation). If the COM2 port init fails, fall back to today's
            // KdpStub text path rather than wedging boot on a dead wire.
            if KdpWireTransportArm(loader_block as PVOID) < 0 {
                enable_kd = 0u8;
            }
        }
    }

    if enable_kd != 0 && KdDebuggerInitialize0(loader_block) >= 0 {
        KiDebugRoutine = Some(abi_cast!(KdpTrap));

        if KdpDebuggerStructuresInitialized == 0 {
            KdpContext.KdpDefaultRetries = 20;

            KdpOweBreakpoint = 0u8;
            // Zero the entire breakpoint table (32 entries * entry size)
            core::ptr::write_bytes(
                core::ptr::addr_of_mut!(KdpBreakpointTable) as *mut u8,
                0,
                core::mem::size_of_val(&*(&raw const KdpBreakpointTable)),
            );

            KeInitializeDpc(
                core::ptr::addr_of_mut!(KdpTimeSlipDpc) as *mut ntkrnl_sys::ddk::_KDPC,
                Some(abi_cast!(KdpTimeSlipDpcRoutine)),
                core::ptr::null_mut()
            );
            KeInitializeTimer(core::ptr::addr_of_mut!(KdpTimeSlipTimer) as *mut ntkrnl_sys::ddk::_KTIMER);
            // ExInitializeWorkItem(&KdpTimeSlipWorkItem, KdpTimeSlipWork, NULL)
            KdpTimeSlipWorkItem.WorkerRoutine = Some(abi_cast!(KdpTimeSlipWork));
            KdpTimeSlipWorkItem.Parameter = core::ptr::null_mut();

            KdpDebuggerStructuresInitialized = 1u8;
        }

        KdTimerStart.QuadPart = 0;

        KdPitchDebugger = 0u8;
        KdDebuggerEnabled = 1u8;

        (*SharedUserData).KdDebuggerEnabled = 1u8;

        KdpPrintBanner();

        if disable_kd_after_init != 0 {
            let _ = KdDisableDebuggerWithLock(0u8);
            KdBlockEnable = block_enable;
            return 1u8;
        }

        if !loader_block.is_null() {
            let mut next_entry = (*loader_block).LoadOrderListHead.Flink;
            let mut i = 0usize;

            while next_entry != core::ptr::addr_of_mut!((*loader_block).LoadOrderListHead) as PLIST_ENTRY && i < 2 {
                let ldr_entry = (next_entry as *mut u8)
                    .sub(core::mem::offset_of!(ntkrnl_sys::ddk::_LDR_DATA_TABLE_ENTRY, InLoadOrderLinks))
                    as PLDR_DATA_TABLE_ENTRY;

                let name_ptr = (*ldr_entry).FullDllName.Buffer;
                let length = ((*ldr_entry).FullDllName.Length / 2) as usize;
                let mut name_buffer: [i8; 256] = [0; 256];
                let mut j = 0usize;

                while j < length {
                    name_buffer[j] = *name_ptr.add(j) as i8;
                    j += 1;
                }
                name_buffer[j] = 0;

                #[cfg(target_arch = "aarch64")]
                {
                    let mut image_name: STRING = core::mem::zeroed();
                    RtlInitString(core::ptr::addr_of_mut!(image_name), name_buffer.as_ptr() as PSTR);
                    DbgLoadImageSymbols(
                        core::ptr::addr_of_mut!(image_name),
                        (*ldr_entry).DllBase,
                        PsGetCurrentProcessId() as ULONG_PTR
                    );
                }

                next_entry = (*next_entry).Flink;
                i += 1;
            }

            KdBreakAfterSymbolLoad = KdPollBreakIn();
        }
    } else {
        KdDebuggerNotPresent = 1u8;
    }

    1u8
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/kd64/kdinit.c
//  * PURPOSE:         KD64 Initialization Code
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Stefan Ginsberg (stefan.ginsberg@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #include <reactos/buildno.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /*
//  * Override DbgPrint(), used by the debugger banner DPRINTs below,
//  * because KdInitSystem() can be called under the debugger lock by
//  * KdEnableDebugger(WithLock)().
//  */
// #define DbgPrint(fmt, ...) (KdpDprintf(fmt, ##__VA_ARGS__), 0)
// #define DbgPrintEx(cmpid, lvl, fmt, ...) (KdpDprintf(fmt, ##__VA_ARGS__), 0)
//
// /* UTILITY FUNCTIONS *********************************************************/
//
// #include <mm/ARM3/miarm.h> // For MiIsMemoryTypeInvisible()
//
// /**
//  * @brief
//  * Retrieves the total size of the memory before Mm is initialized,
//  * by counting the number of physical pages. Useful for debug logging.
//  *
//  * Adapted from mm/ARM3/mminit.c!MiScanMemoryDescriptors().
//  **/
// static
// SIZE_T
// KdpGetMemorySizeInMBs(
//     _In_opt_ PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     PLIST_ENTRY ListEntry;
//     PMEMORY_ALLOCATION_DESCRIPTOR Descriptor;
//     SIZE_T NumberOfPhysicalPages = 0;
//
//     /*
//      * If no loader block is present (e.g. the debugger is initialized only
//      * much later after boot), just use the already-initialized Mm-computed
//      * number of physical pages. Otherwise do the evaluation ourselves.
//      */
//     if (!LoaderBlock)
//     {
//         NumberOfPhysicalPages = MmNumberOfPhysicalPages;
//         goto ReturnSize;
//     }
//
//     /* Loop the memory descriptors */
//     for (ListEntry = LoaderBlock->MemoryDescriptorListHead.Flink;
//          ListEntry != &LoaderBlock->MemoryDescriptorListHead;
//          ListEntry = ListEntry->Flink)
//     {
//         /* Get the descriptor */
//         Descriptor = CONTAINING_RECORD(ListEntry,
//                                        MEMORY_ALLOCATION_DESCRIPTOR,
//                                        ListEntry);
//
//         /* If this is invisible memory, skip this descriptor */
//         if (MiIsMemoryTypeInvisible(Descriptor->MemoryType))
//             continue;
//
//         /* Check if this isn't bad memory */
//         if (Descriptor->MemoryType != LoaderBad)
//         {
//             /* Count it in the physical pages */
//             NumberOfPhysicalPages += Descriptor->PageCount;
//         }
//     }
//
// ReturnSize:
//     /* Round size up. Assumed to better match actual physical RAM size */
//     return ALIGN_UP_BY(NumberOfPhysicalPages * PAGE_SIZE, 1024 * 1024) / (1024 * 1024);
// }
//
// /**
//  * @brief
//  * Displays the kernel debugger initialization banner.
//  **/
// static
// VOID
// KdpPrintBanner(VOID)
// {
//     SIZE_T MemSizeMBs = KdpGetMemorySizeInMBs(KeLoaderBlock);
//
//     DPRINT1("-----------------------------------------------------\n");
//     DPRINT1("ReactOS " KERNEL_VERSION_STR " (Build " KERNEL_VERSION_BUILD_STR ") (Commit " KERNEL_VERSION_COMMIT_HASH ")\n");
//     DPRINT1("%u System Processor [%u MB Memory]\n", KeNumberProcessors, MemSizeMBs);
//
//     if (KeLoaderBlock)
//     {
//         DPRINT1("Command Line: %s\n", KeLoaderBlock->LoadOptions);
//         DPRINT1("ARC Paths: %s %s %s %s\n",
//                 KeLoaderBlock->ArcBootDeviceName, KeLoaderBlock->NtHalPathName,
//                 KeLoaderBlock->ArcHalDeviceName, KeLoaderBlock->NtBootPathName);
//     }
// }
//
// /* FUNCTIONS *****************************************************************/
//
// VOID
// NTAPI
// KdUpdateDataBlock(VOID)
// {
//     /* Update the KeUserCallbackDispatcher pointer */
//     KdDebuggerDataBlock.KeUserCallbackDispatcher =
//         (ULONG_PTR)KeUserCallbackDispatcher;
// }
//
// BOOLEAN
// NTAPI
// KdRegisterDebuggerDataBlock(IN ULONG Tag,
//                             IN PDBGKD_DEBUG_DATA_HEADER64 DataHeader,
//                             IN ULONG Size)
// {
//     KIRQL OldIrql;
//     PLIST_ENTRY NextEntry;
//     PDBGKD_DEBUG_DATA_HEADER64 CurrentHeader;
//
//     /* Acquire the Data Lock */
//     KeAcquireSpinLock(&KdpDataSpinLock, &OldIrql);
//
//     /* Loop the debugger data list */
//     NextEntry = KdpDebuggerDataListHead.Flink;
//     while (NextEntry != &KdpDebuggerDataListHead)
//     {
//         /* Get the header for this entry */
//         CurrentHeader = CONTAINING_RECORD(NextEntry,
//                                           DBGKD_DEBUG_DATA_HEADER64,
//                                           List);
//
//         /*  Move to the next one */
//         NextEntry = NextEntry->Flink;
//
//         /* Check if we already have this data block */
//         if ((CurrentHeader == DataHeader) || (CurrentHeader->OwnerTag == Tag))
//         {
//             /* Release the lock and fail */
//             KeReleaseSpinLock(&KdpDataSpinLock, OldIrql);
//             return FALSE;
//         }
//     }
//
//     /* Setup the header */
//     DataHeader->OwnerTag = Tag;
//     DataHeader->Size = Size;
//
//     /* Insert it into the list and release the lock */
//     InsertTailList(&KdpDebuggerDataListHead, (PLIST_ENTRY)&DataHeader->List);
//     KeReleaseSpinLock(&KdpDataSpinLock, OldIrql);
//     return TRUE;
// }
//
// BOOLEAN
// NTAPI
// KdInitSystem(
//     _In_ ULONG BootPhase,
//     _In_opt_ PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     BOOLEAN EnableKd, DisableKdAfterInit = FALSE, BlockEnable = FALSE;
//     PLDR_DATA_TABLE_ENTRY LdrEntry;
//     ULONG i;
//
//     /* Check if this is Phase 1 */
//     if (BootPhase)
//     {
//         /* Just query the performance counter */
//         KeQueryPerformanceCounter(&KdPerformanceCounterRate);
//         return TRUE;
//     }
//
//     /* Check if we already initialized once */
//     if (KdDebuggerEnabled)
//         return TRUE;
//
//     /* Set the Debug Routine as the Stub for now */
//     KiDebugRoutine = KdpStub;
//
//     /* Disable break after symbol load for now */
//     KdBreakAfterSymbolLoad = FALSE;
//
//     /* Check if the Debugger Data Block was already initialized */
//     if (!KdpDebuggerDataListHead.Flink)
//     {
//         /* It wasn't...Initialize the KD Data Listhead */
//         InitializeListHead(&KdpDebuggerDataListHead);
//
//         /* Register the Debugger Data Block */
//         KdRegisterDebuggerDataBlock(KDBG_TAG,
//                                     &KdDebuggerDataBlock.Header,
//                                     sizeof(KdDebuggerDataBlock));
//
//         /* Fill out the KD Version Block */
//         KdVersionBlock.MajorVersion = (USHORT)((DBGKD_MAJOR_NT << 8) | (NtBuildNumber >> 28));
//         KdVersionBlock.MinorVersion = (USHORT)(NtBuildNumber & 0xFFFF);
//
// #ifdef CONFIG_SMP
//         /* This is an MP Build */
//         KdVersionBlock.Flags |= DBGKD_VERS_FLAG_MP;
// #endif
//
//         /* Save Pointers to Loaded Module List and Debugger Data */
//         KdVersionBlock.PsLoadedModuleList = (ULONG64)(LONG_PTR)&PsLoadedModuleList;
//         KdVersionBlock.DebuggerDataList = (ULONG64)(LONG_PTR)&KdpDebuggerDataListHead;
//
//         /* Set protocol limits */
//         KdVersionBlock.MaxStateChange = DbgKdMaximumStateChange -
//                                         DbgKdMinimumStateChange;
//         KdVersionBlock.MaxManipulate = DbgKdMaximumManipulate -
//                                        DbgKdMinimumManipulate;
//         KdVersionBlock.Unused[0] = 0;
//
//         /* Link us in the KPCR */
//         KeGetPcr()->KdVersionBlock = &KdVersionBlock;
//     }
//
//     /* Check if we have a loader block */
//     if (LoaderBlock)
//     {
//         PSTR CommandLine, DebugLine;
//
//         /* Get the image entry */
//         LdrEntry = CONTAINING_RECORD(LoaderBlock->LoadOrderListHead.Flink,
//                                      LDR_DATA_TABLE_ENTRY,
//                                      InLoadOrderLinks);
//
//         /* Save the Kernel Base */
//         PsNtosImageBase = (ULONG_PTR)LdrEntry->DllBase;
//         KdVersionBlock.KernBase = (ULONG64)(LONG_PTR)LdrEntry->DllBase;
//
//         /* Check if we have a command line */
//         CommandLine = LoaderBlock->LoadOptions;
//         if (CommandLine)
//         {
//             /* Upcase it */
//             _strupr(CommandLine);
//
//             /* Assume we'll disable KD */
//             EnableKd = FALSE;
//
//             /* Check for CRASHDEBUG, NODEBUG and just DEBUG */
//             if (strstr(CommandLine, "CRASHDEBUG"))
//             {
//                 /* Don't enable KD now, but allow it to be enabled later */
//                 KdPitchDebugger = FALSE;
//             }
//             else if (strstr(CommandLine, "NODEBUG"))
//             {
//                 /* Don't enable KD and don't let it be enabled later */
//                 KdPitchDebugger = TRUE;
//             }
//             else if ((DebugLine = strstr(CommandLine, "DEBUG")))
//             {
//                 /* Enable KD */
//                 EnableKd = TRUE;
//
//                 /* Check if there are any options */
//                 if (DebugLine[5] == '=')
//                 {
//                     /* Save pointers */
//                     PSTR DebugOptionStart, DebugOptionEnd;
//                     DebugOptionStart = DebugOptionEnd = &DebugLine[6];
//
//                     /* Scan the string for debug options */
//                     for (;;)
//                     {
//                         SIZE_T DebugOptionLength;
//
//                         /* Loop until we reach the end of the string */
//                         while (*DebugOptionEnd != ANSI_NULL)
//                         {
//                             /* Check if this is a comma, a space or a tab */
//                             if ((*DebugOptionEnd == ',') ||
//                                 (*DebugOptionEnd == ' ') ||
//                                 (*DebugOptionEnd == '\t'))
//                             {
//                                 /*
//                                  * We reached the end of the option or
//                                  * the end of the string, break out.
//                                  */
//                                 break;
//                             }
//                             else
//                             {
//                                 /* Move on to the next character */
//                                 DebugOptionEnd++;
//                             }
//                         }
//
//                         /* Calculate the length of the current option */
//                         DebugOptionLength = (DebugOptionEnd - DebugOptionStart);
//
//                         /*
//                          * Break out if we reached the last option
//                          * or if there were no options at all.
//                          */
//                         if (!DebugOptionLength)
//                             break;
//
//                         /* Now check which option this is */
//                         if ((DebugOptionLength == 10) &&
//                             !(strncmp(DebugOptionStart, "AUTOENABLE", 10)))
//                         {
//                             /* Disable the debugger, but
//                              * allow to re-enable it later */
//                             DisableKdAfterInit = TRUE;
//                             BlockEnable = FALSE;
//                             KdAutoEnableOnEvent = TRUE;
//                         }
//                         else if ((DebugOptionLength == 7) &&
//                                  !(strncmp(DebugOptionStart, "DISABLE", 7)))
//                         {
//                             /* Disable the debugger */
//                             DisableKdAfterInit = TRUE;
//                             BlockEnable = TRUE;
//                             KdAutoEnableOnEvent = FALSE;
//                         }
//                         else if ((DebugOptionLength == 6) &&
//                                  !(strncmp(DebugOptionStart, "NOUMEX", 6)))
//                         {
//                             /* Ignore user mode exceptions */
//                             KdIgnoreUmExceptions = TRUE;
//                         }
//
//                         /*
//                          * If there are more options then the next character
//                          * should be a comma. Break out if it isn't.
//                          */
//                         if (*DebugOptionEnd != ',')
//                             break;
//
//                         /* Move on to the next option */
//                         DebugOptionEnd++;
//                         DebugOptionStart = DebugOptionEnd;
//                     }
//                 }
//             }
//         }
//         else
//         {
//             /* No command line options? Disable debugger by default */
//             KdPitchDebugger = TRUE;
//             EnableKd = FALSE;
//         }
//     }
//     else
//     {
//         /* Called from a bugcheck or a re-enable. Save the Kernel Base. */
//         KdVersionBlock.KernBase = (ULONG64)(LONG_PTR)PsNtosImageBase;
//
//         /* Unconditionally enable KD */
//         EnableKd = TRUE;
//     }
//
//     /* Set the Kernel Base in the Data Block */
//     KdDebuggerDataBlock.KernBase = (ULONG_PTR)KdVersionBlock.KernBase;
//
//     /* Initialize the debugger if requested */
//     if (EnableKd && (NT_SUCCESS(KdDebuggerInitialize0(LoaderBlock))))
//     {
//         /* Now set our real KD routine */
//         KiDebugRoutine = KdpTrap;
//
//         /* Check if we've already initialized our structures */
//         if (!KdpDebuggerStructuresInitialized)
//         {
//             /* Set Retries */
//             KdpContext.KdpDefaultRetries = 20;
//
//             /* Initialize breakpoints owed flag and table */
//             KdpOweBreakpoint = FALSE;
//             for (i = 0; i < KD_BREAKPOINT_MAX; i++)
//             {
//                 KdpBreakpointTable[i].Flags   = 0;
//                 KdpBreakpointTable[i].DirectoryTableBase = 0;
//                 KdpBreakpointTable[i].Address = NULL;
//             }
//
//             /* Initialize the Time Slip DPC */
//             KeInitializeDpc(&KdpTimeSlipDpc, KdpTimeSlipDpcRoutine, NULL);
//             KeInitializeTimer(&KdpTimeSlipTimer);
//             ExInitializeWorkItem(&KdpTimeSlipWorkItem, KdpTimeSlipWork, NULL);
//
//             /* First-time initialization done! */
//             KdpDebuggerStructuresInitialized = TRUE;
//         }
//
//         /* Initialize the timer */
//         KdTimerStart.QuadPart = 0;
//
//         /* Officially enable KD */
//         KdPitchDebugger = FALSE;
//         KdDebuggerEnabled = TRUE;
//
//         /* Let user-mode know that it's enabled as well */
//         SharedUserData->KdDebuggerEnabled = TRUE;
//
//         /* Display separator + ReactOS version at the start of the debug log */
//         KdpPrintBanner();
//
//         /* Check if the debugger should be disabled initially */
//         if (DisableKdAfterInit)
//         {
//             /* Disable it */
//             KdDisableDebuggerWithLock(FALSE);
//
//             /*
//              * Save the enable block state and return initialized
//              * (the debugger is active but disabled).
//              */
//             KdBlockEnable = BlockEnable;
//             return TRUE;
//         }
//
//         /* Check if we have a loader block */
//         if (LoaderBlock)
//         {
//             PLIST_ENTRY NextEntry;
//             ULONG j, Length;
//             PWCHAR Name;
//             STRING ImageName;
//             CHAR NameBuffer[256];
//
//             /* Loop over the first two boot images: HAL and kernel */
//             for (NextEntry = LoaderBlock->LoadOrderListHead.Flink, i = 0;
//                  NextEntry != &LoaderBlock->LoadOrderListHead && (i < 2);
//                  NextEntry = NextEntry->Flink, ++i)
//             {
//                 /* Get the image entry */
//                 LdrEntry = CONTAINING_RECORD(NextEntry,
//                                              LDR_DATA_TABLE_ENTRY,
//                                              InLoadOrderLinks);
//
//                 /* Generate the image name */
//                 Name = LdrEntry->FullDllName.Buffer;
//                 Length = LdrEntry->FullDllName.Length / sizeof(WCHAR);
//                 j = 0;
//                 do
//                 {
//                     /* Do cheap Unicode to ANSI conversion */
//                     NameBuffer[j++] = (CHAR)*Name++;
//                 } while (j < Length);
//
//                 /* Null-terminate */
//                 NameBuffer[j] = ANSI_NULL;
//
//                 /* Load the symbols */
//                 RtlInitString(&ImageName, NameBuffer);
//                 DbgLoadImageSymbols(&ImageName,
//                                     LdrEntry->DllBase,
//                                     (ULONG_PTR)PsGetCurrentProcessId());
//             }
//
//             /* Check for incoming break-in and break on symbol load
//              * if requested, see ex/init.c!ExpLoadBootSymbols() */
//             KdBreakAfterSymbolLoad = KdPollBreakIn();
//         }
//     }
//     else
//     {
//         /* Disable debugger */
//         KdDebuggerNotPresent = TRUE;
//     }
//
//     /* Return initialized */
//     return TRUE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

/// `0x5000` / `0X5000` as a 16550 I/O port base; 0 if absent or out of range.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
unsafe fn console_port(mut p: PSTR) -> u16 {
    if *p == b'0' as i8 && (*p.add(1) == b'x' as i8 || *p.add(1) == b'X' as i8) {
        p = p.add(2);
    }
    let mut v: u32 = 0;
    loop {
        let c = *p as u8;
        let d = match c {
            b'0'..=b'9' => c - b'0',
            b'a'..=b'f' => c - b'a' + 10,
            b'A'..=b'F' => c - b'A' + 10,
            _ => break,
        };
        v = v.saturating_mul(16).saturating_add(d as u32);
        p = p.add(1);
    }
    // All eight registers must be addressable.
    if v == 0 || v > 0xfff8 { 0 } else { v as u16 }
}
