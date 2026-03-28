# Milestone 6: Ramdisk Boot Debugging Journal

## Session 1: Initial Ramdisk Setup

- Created ramdisk.img with ntoskrnl.exe, hal.dll, kdcom.dll, bootvid.dll, ntdll.dll, NLS files, ramdisk.sys, fastfat.sys, SYSTEM hive
- Updated freeldr.ini: SystemPath=ramdisk(0)\reactos, /RDPATH=\ramdisk.img
- Fixed MmMapIoSpace ARM64 assert (iosup.c: added _M_ARM64 to skip HighPart==0 assert)
- Updated IO init ARM64 guards (iomgr.c: IopReassignSystemRoot, PsLocateSystemDll)
- Updated Phase 1 completion path (init.c: conditional ARM64 early-return)
- Boot got to FastFAT filesystem mount, then hung

## Session 2: CcMapData / Splay Tree Hang

- **Symptom**: Kernel hung during CcMapData call inside FatReadVolumeFile (FastFAT initialization)
- **Diagnosis**: Added UART traces through CcMapData -> CcRosGetVacb -> section page table lookup
- Hang was inside `RtlInsertElementGenericTable` in `sdk/lib/rtl/generictable.c`
- Splay tree operations deadlocked due to QEMU TCG (JIT) optimization issue
- **Root cause**: QEMU TCG's translation block caching can reorder or cache stale values for non-volatile memory operations. Standard ARM64 barriers (`dsb sy; isb`) were NOT sufficient
- **Fix**: Added `GT_TRACE` UART MMIO writes (device memory at PL011 0x09000000) at 8 points in generictable.c. Device memory accesses force TCG to flush/synchronize its JIT state. This is NOT a timing fix -- it is a memory ordering fix specific to QEMU's translation cache
- **Key insight**: On real ARM64 hardware, `dsb sy` would suffice. QEMU TCG needs stronger synchronization via device-memory MMIO writes
- GT_TRACE must remain always-on for ARM64 QEMU builds

## Session 3: PDE=0 Silent Bugcheck (Page Fault Hang)

- **Symptom**: After splay tree fix, system progressed to page fault at VACB address 0xFFFF800005600000. Trap handler's `[PAGE] CreateVMap` fired but `MmAccessFault resolved` never appeared
- **Diagnosis path**:
  1. Traced MmAccessFault dispatch: system space VACB -> VAD lookup returns NULL -> legacy MmNotPresentFault -> MmNotPresentFaultSectionView -> MmCreateVirtualMapping -> MmCreateVirtualMappingUnsafeEx
  2. Inside MmCreateVirtualMappingUnsafeEx: `MiIsPdeForAddressValid(Address)` returns FALSE -> KeBugCheck(MEMORY_MANAGEMENT)
  3. **Why it looked like a hang**: KeBugCheck calls DbgPrint -> then BRK #0xF000 for debugger break -> ARM64 BRK handler silently skips (Pc += 4) -> system halts. No UART output = appears frozen
  4. Added per-step UART traces showing: PXE=valid, PPE=valid, PDE=0x0000000000000000
- **Root cause**: System PTE space initialized by MiBuildSystemPteSpace covers only MI_NUMBER_SYSTEM_PTES (22000) x 4KB = ~86MB from MI_DEFAULT_SYSTEM_RANGE_START (0xFFFF800000000000) to ~0xFFFF8000055E1000. VACB address 0xFFFF800005600000 is 0x1F000 bytes PAST the end. Legacy MM's MmCreateMemoryArea allocator assigns VAs beyond this range where no PDEs exist
- **Fix**: Added `MiEnsureKernelPdeValid()` in `ntoskrnl/mm/arm64/page.c` -- creates PPE/PDE on demand using MiRemoveAnyPage + MI_WRITE_VALID_PTE + TLB barriers. Called from MmCreateVirtualMappingUnsafeEx when MiIsPdeForAddressValid returns FALSE
- **Verified**: Boot output shows `[PAGE] Creating PDE for 0xFFFF800005600000` followed by successful page fault resolution

## Key Debugging Patterns Learned

1. **Silent bugchecks on ARM64**: KeBugCheck -> DbgPrint -> BRK -> skip -> halt. Looks like a hang. Always add UART traces before suspected bugcheck paths
2. **QEMU TCG quirks**: Device memory MMIO writes (not standard barriers) are needed to synchronize QEMU's TCG JIT cache. This affects any code with complex memory access patterns (splay trees, linked lists)
3. **UART trace strategy**: Use direct PL011 MMIO writes (0x09000000) for critical traces -- these bypass DbgPrint/BRK entirely and are always visible
4. **PDE gaps in system space**: ARM64 kernel VA space is 256TB but PDEs only cover a small portion. Any code that allocates kernel VAs dynamically must ensure PDE hierarchy exists
5. **PFN assertion noise**: ~1500 non-fatal PFN assertion failures per boot dominate UART output and slow boot significantly

## Current State (Post-Session 3)

- Boot progresses through Phase 1 init, PDE creation works, splay trees work
- FastFAT filesystem mount in progress (GT and PAGE traces at end of 5-minute run)
- Blocked by: UART output speed -- 1514 PFN assertions + traces make boot take >5 minutes
- Next step: Silence PFN assertions, add targeted boot milestone UART traces, verify ramdisk.sys/fastfat.sys DriverEntry execution
