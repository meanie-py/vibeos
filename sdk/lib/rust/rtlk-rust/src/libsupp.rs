use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ps::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ex::*;

use ntkrnl_sys::ddk::{
    _EPROCESS, _HANDLE_TABLE_ENTRY, _HEAP_LOCK, _KPRCB, _KTRAP_FRAME,
    _RTL_ATOM_TABLE, _RTL_ATOM_TABLE_ENTRY,
    DWORD64, ExAcquireFastMutex, ExAcquireResourceExclusiveLite, ExAcquireResourceSharedLite,
    ExCreateHandle, ExCreateHandleTable, ExDeleteResourceLite, ExDestroyHandle,
    ExDestroyHandleTable, ExInitializeFastMutex, ExInitializePagedLookasideList,
    ExInitializeResourceLite, ExMapHandleToPointer, ExReleaseFastMutex, ExReleaseResourceLite,
    ExSweepHandleTable, ExUnlockHandleTableEntry, HANDLE, IMAGE_RESOURCE_DIRECTORY,
    KPROCESSOR_MODE, KeEnterCriticalRegion,
    KeIsAttachedProcess, KeLeaveCriticalRegion, KiPcToFileHeader, LDR_RESOURCE_INFO,
    LPCWSTR, PAGED_LOOKASIDE_LIST, PCONTEXT, PDWORD64, PETHREAD, PEXCEPTION_RECORD,
    PEXCEPTION_REGISTRATION_RECORD, PHANDLE_TABLE_ENTRY, PHEAP_LOCK, PIMAGE_NT_HEADERS,
    PKTHREAD, PKTRAP_FRAME, PLDR_DATA_TABLE_ENTRY, PPEB, PRTL_ATOM_TABLE,
    PRTL_ATOM_TABLE_ENTRY, PRTL_HEAP_PARAMETERS, PTEB,
};

#[cfg(target_arch = "x86_64")]
use ntkrnl_sys::ddk::KeGetPcr;
#[cfg(not(target_arch = "x86"))]
use ntkrnl_sys::ddk::PRUNTIME_FUNCTION;
#[cfg(not(target_arch = "x86"))]
use ntkrnl_sys::ddk::PUNWIND_HISTORY_TABLE;
#[cfg(target_arch = "x86")]
use ntkrnl_sys::ddk::PKPCR;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

/// i386 KeGetPcr — reads KPCR pointer from fs:0x1C (SelfPcr field). A private
/// helper, not an export: ntkrnl-sys's `global_asm!` owns the `_KeGetPcr` symbol on
/// i386, and a `#[no_mangle]` here made two providers of it in the kernel link.
#[cfg(target_arch = "x86")]
#[inline(always)]
pub unsafe extern "C" fn KeGetPcr() -> PKPCR {
    let pcr: PKPCR;
    core::arch::asm!("mov {}, fs:[0x1C]", out(reg) pcr, options(nostack, readonly));
    pcr
}

/// aarch64 KeGetPcr — reads KPCR pointer from TPIDR_EL1, same as the C
/// FORCEINLINE (xdk/arm64/ke.h) and ntkrnl_sys::inline::ke::KeGetPcr.
/// NEVER read x18 for this: ctxswitch.S loads x18 = NewThread->Teb, so in
/// switch/early-thread context x18 is a TEB (or NULL), not the PCR —
/// cswitchdep first light caught exactly this (design doc, arm64 findings).
#[no_mangle]
#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub unsafe fn KeGetPcr() -> *mut ntkrnl_sys::ddk::_KPCR {
    let pcr: *mut ntkrnl_sys::ddk::_KPCR;
    core::arch::asm!("mrs {}, tpidr_el1", out(reg) pcr, options(nostack, readonly));
    pcr
}

// Constants — these are C #define macros, not extern symbols.
const FLG_ENABLE_EXCEPTION_LOGGING: ULONG = 0x00800000;
const TAG_ASTR: ULONG = 0x52545341;
const TAG_OSTR: ULONG = 0x5254534F;
const TAG_USTR: ULONG = 0x52545355;
const TAG_RTHL: ULONG = u32::from_le_bytes(*b"LHtR");
const TAG_ATMT: ULONG = u32::from_le_bytes(*b"TotA");
const POOL_COLD_ALLOCATION: ULONG = 0x100;
#[cfg(target_arch = "x86")]
const KERNEL_STACK_SIZE: ULONG = 0x3000;
#[cfg(target_arch = "x86_64")]
const KERNEL_STACK_SIZE: ULONG = 0xC000;
#[cfg(target_arch = "aarch64")]
const KERNEL_STACK_SIZE: ULONG = 0xC000; // 48KB — matches xdk/arm64/ke.h (bumped from i386's 12KB)

extern "system" {
    static mut NtGlobalFlag: ULONG;
    static mut MmHeapSegmentCommit: SIZE_T;
    static mut MmHeapSegmentReserve: SIZE_T;
    static mut MmHeapDeCommitFreeBlockThreshold: SIZE_T;
    static mut MmHeapDeCommitTotalFreeThreshold: SIZE_T;
    static mut MmHighestUserAddress: PVOID;
    static mut PsLoadedModuleSpinLock: KSPIN_LOCK;

    fn RtlImageDirectoryEntryToData(
        BaseAddress: PVOID,
        MappedAsImage: BOOLEAN,
        DirectoryEntry: USHORT,
        Size: *mut ULONG,
    ) -> PVOID;

    fn KeGetCurrentThread() -> PKTHREAD;
    fn ExFreePool(P: PVOID);
}
extern "system" {
    fn PsGetCurrentThread() -> PETHREAD;
}
extern "C" {
    fn KeGetTrapFrame(Thread: PKTHREAD) -> PKTRAP_FRAME;
}

// find_entry_by_name / find_entry_by_id / find_first_entry are plain C
// functions from sdk/lib/rtl/res.c — no NTAPI, so use extern "C".
extern "C" {
    fn find_entry_by_name(
        dir: *mut IMAGE_RESOURCE_DIRECTORY,
        name: LPCWSTR,
        root: PVOID,
        want_dir: i32,
    ) -> *mut IMAGE_RESOURCE_DIRECTORY;

    fn find_entry_by_id(
        dir: *mut IMAGE_RESOURCE_DIRECTORY,
        id: USHORT,
        root: PVOID,
        want_dir: i32,
    ) -> *mut IMAGE_RESOURCE_DIRECTORY;

    fn find_first_entry(
        dir: *mut IMAGE_RESOURCE_DIRECTORY,
        root: PVOID,
        want_dir: i32,
    ) -> *mut IMAGE_RESOURCE_DIRECTORY;
}

const STATUS_RESOURCE_DATA_NOT_FOUND: NTSTATUS = 0xC0000089u32 as i32;
const STATUS_RESOURCE_TYPE_NOT_FOUND: NTSTATUS = 0xC000008Au32 as i32;
const STATUS_RESOURCE_NAME_NOT_FOUND: NTSTATUS = 0xC000008Bu32 as i32;

/// Type alias for HEAP_LOCK — bindgen uses _HEAP_LOCK
type HEAP_LOCK = _HEAP_LOCK;

const IMAGE_DIRECTORY_ENTRY_RESOURCE: USHORT = 2;
const EXCEPTION_EXECUTE_HANDLER: i32 = 1;

#[repr(C)]
struct RTL_RANGE {
    _opaque: [u8; 0],
}

#[repr(C)]
struct RTL_RANGE_ENTRY {
    Entry: LIST_ENTRY,
    Range: RTL_RANGE,
}

type PRTL_RANGE_ENTRY = *mut RTL_RANGE_ENTRY;

#[no_mangle]
pub static mut RtlpRangeListEntryLookasideList: PAGED_LOOKASIDE_LIST = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut RtlpAllocDeallocQueryBufferSize: SIZE_T = 128;

#[no_mangle]
pub unsafe extern "system" fn RtlPcToFileHeader(
    PcValue: PVOID,
    BaseOfImage: *mut PVOID,
) -> PVOID {
    let PcValue = opaque_ptr(PcValue);
    let BaseOfImage = opaque_ptr(BaseOfImage);
    let mut LdrEntry: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
    let mut InSystem: BOOLEAN = 0;
    let mut OldIrql: KIRQL = 0;

    if (PcValue as ULONG_PTR) > (MmHighestUserAddress as ULONG_PTR) {
        // [guard] KeAcquireSpinLock(core::ptr::addr_of_mut!(PsLoadedModuleSpinLock), core::ptr::addr_of_mut!(OldIrql));
        let spin_guard = unsafe { ntkrnl_sys::guards::SpinLockGuard::acquire(core::ptr::addr_of_mut!(PsLoadedModuleSpinLock)) };

        *BaseOfImage = KiPcToFileHeader(
            PcValue,
            core::ptr::addr_of_mut!(LdrEntry),
            0u8,
            core::ptr::addr_of_mut!(InSystem),
        );

        drop(spin_guard);
    } else {
        *BaseOfImage = core::ptr::null_mut();
    }

    *BaseOfImage
}

#[no_mangle]
pub unsafe extern "system" fn RtlInitializeRangeListPackage() {
    ExInitializePagedLookasideList(
        core::ptr::addr_of_mut!(RtlpRangeListEntryLookasideList),
        None,
        None,
        POOL_COLD_ALLOCATION,
        core::mem::size_of::<RTL_RANGE_ENTRY>() as _,
        u32::from_le_bytes(*b"elRR"),
        16,
    );
}

#[no_mangle]
pub unsafe extern "system" fn RtlpCheckForActiveDebugger() -> BOOLEAN {
    0u8
}

#[no_mangle]
pub unsafe extern "system" fn RtlpSetInDbgPrint() -> BOOLEAN {
    0u8
}

#[no_mangle]
pub unsafe extern "system" fn RtlpClearInDbgPrint() {
}

#[no_mangle]
pub unsafe extern "system" fn RtlpGetMode() -> KPROCESSOR_MODE {
    0i8
}

#[no_mangle]
pub unsafe extern "system" fn RtlpAllocateMemory(Bytes: ULONG, Tag: ULONG) -> PVOID {
    ExAllocatePoolWithTag(PagedPool, Bytes as SIZE_T, Tag)
}

#[no_mangle]
pub unsafe extern "system" fn RtlpFreeMemory(Mem: PVOID, Tag: ULONG) {
    let Mem = opaque_ptr(Mem);
    if Tag == TAG_ASTR || Tag == TAG_OSTR || Tag == TAG_USTR {
        ExFreePool(Mem);
    } else {
        ExFreePoolWithTag(Mem, Tag);
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlAcquirePebLock() {
}

#[no_mangle]
pub unsafe extern "system" fn RtlReleasePebLock() {
}

#[no_mangle]
pub unsafe extern "system" fn LdrShutdownThread() -> NTSTATUS {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlGetCurrentPeb() -> PPEB {
    let current_thread = KeGetCurrentThread();
    let process = (*current_thread).__bindgen_anon_1.ApcState.Process as *mut _EPROCESS;
    (*process).Peb
}

#[no_mangle]
pub unsafe extern "system" fn RtlDeleteHeapLock(Lock: PHEAP_LOCK) -> NTSTATUS {
    ExDeleteResourceLite(core::ptr::addr_of_mut!((*Lock).__bindgen_anon_1.Resource));
    ExFreePoolWithTag(Lock as PVOID, TAG_RTHL);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlEnterHeapLock(Lock: PHEAP_LOCK, Exclusive: BOOLEAN) -> NTSTATUS {
    KeEnterCriticalRegion();

    if Exclusive != 0 {
        ExAcquireResourceExclusiveLite(core::ptr::addr_of_mut!((*Lock).__bindgen_anon_1.Resource), 1u8);
    } else {
        ExAcquireResourceSharedLite(core::ptr::addr_of_mut!((*Lock).__bindgen_anon_1.Resource), 1u8);
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlTryEnterHeapLock(Lock: PHEAP_LOCK, Exclusive: BOOLEAN) -> BOOLEAN {
    let Success: BOOLEAN;
    KeEnterCriticalRegion();

    if Exclusive != 0 {
        Success = ExAcquireResourceExclusiveLite(core::ptr::addr_of_mut!((*Lock).__bindgen_anon_1.Resource), 0u8);
    } else {
        Success = ExAcquireResourceSharedLite(core::ptr::addr_of_mut!((*Lock).__bindgen_anon_1.Resource), 0u8);
    }

    if Success == 0 {
        KeLeaveCriticalRegion();
    }

    Success
}

#[no_mangle]
pub unsafe extern "system" fn RtlInitializeHeapLock(Lock: *mut PHEAP_LOCK) -> NTSTATUS {
    let Lock = opaque_ptr(Lock);
    let HeapLock = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<HEAP_LOCK>(),
        TAG_RTHL,
    ) as PHEAP_LOCK;
    
    if HeapLock.is_null() {
        return STATUS_NO_MEMORY;
    }

    ExInitializeResourceLite(core::ptr::addr_of_mut!((*HeapLock).__bindgen_anon_1.Resource));
    *Lock = HeapLock;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlLeaveHeapLock(Lock: PHEAP_LOCK) -> NTSTATUS {
    ExReleaseResourceLite(core::ptr::addr_of_mut!((*Lock).__bindgen_anon_1.Resource));
    KeLeaveCriticalRegion();
    STATUS_SUCCESS
}

#[repr(C)]
struct _HEAP {
    _opaque: [u8; 0],
}

#[no_mangle]
pub unsafe extern "system" fn RtlpAddHeapToProcessList(_Heap: *mut _HEAP) {
    let _Heap = opaque_ptr(_Heap);
}

#[no_mangle]
pub unsafe extern "system" fn RtlpRemoveHeapFromProcessList(_Heap: *mut _HEAP) {
    let _Heap = opaque_ptr(_Heap);
}

#[no_mangle]
pub unsafe extern "C" fn RtlInitializeHeapManager() {
}

// #if DBG — called from sdk/lib/rtl via PAGED_CODE_RTL() macro
// C signature: VOID FASTCALL CHECK_PAGED_CODE_RTL(char *file, int line)
#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn CHECK_PAGED_CODE_RTL(_file: *const u8, _line: i32) {
    // Debug check: KeGetCurrentIrql() > APC_LEVEL → DbgPrint + ASSERT
    // For now, no-op in Rust (matches DUMMY_PSEH builds where assertions are non-fatal)
}
#[no_mangle]
#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn CHECK_PAGED_CODE_RTL(_file: *const u8, _line: i32) {
    // Debug check: no-op in Rust
}

#[no_mangle]
pub unsafe extern "system" fn RtlpSetHeapParameters(Parameters: PRTL_HEAP_PARAMETERS) {
    if (*Parameters).SegmentCommit == 0 {
        (*Parameters).SegmentCommit = MmHeapSegmentCommit as _;
    }
    if (*Parameters).SegmentReserve == 0 {
        (*Parameters).SegmentReserve = MmHeapSegmentReserve as _;
    }
    if (*Parameters).DeCommitFreeBlockThreshold == 0 {
        (*Parameters).DeCommitFreeBlockThreshold = MmHeapDeCommitFreeBlockThreshold as _;
    }
    if (*Parameters).DeCommitTotalFreeThreshold == 0 {
        (*Parameters).DeCommitTotalFreeThreshold = MmHeapDeCommitTotalFreeThreshold as _;
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlpCheckLogException(
    _ExceptionRecord: PEXCEPTION_RECORD,
    _ContextRecord: PCONTEXT,
    _ContextData: PVOID,
    _Size: ULONG,
) {
    let _ContextData = opaque_ptr(_ContextData);
    if (NtGlobalFlag & FLG_ENABLE_EXCEPTION_LOGGING) != 0 {
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlpHandleDpcStackException(
    _RegistrationFrame: PEXCEPTION_REGISTRATION_RECORD,
    RegistrationFrameEnd: ULONG_PTR,
    StackLow: *mut ULONG_PTR,
    StackHigh: *mut ULONG_PTR,
) -> BOOLEAN {
    let StackLow = opaque_ptr(StackLow);
    let StackHigh = opaque_ptr(StackHigh);
    let Prcb: *mut _KPRCB;
    let DpcStack: ULONG_PTR;

    if KeGetCurrentIrql() >= DISPATCH_LEVEL {
        Prcb = KeGetCurrentPrcb() as *mut _KPRCB;
        DpcStack = (*Prcb).DpcStack as ULONG_PTR;

        if ((*Prcb).DpcRoutineActive != 0)
            && (RegistrationFrameEnd <= DpcStack)
            && ((_RegistrationFrame as ULONG_PTR) >= DpcStack - KERNEL_STACK_SIZE as ULONG_PTR)
        {
            *StackHigh = DpcStack;
            *StackLow = DpcStack - KERNEL_STACK_SIZE as ULONG_PTR;
            return 1u8;
        }
    }

    0u8
}

#[no_mangle]
#[cfg(not(any(target_arch = "arm", target_arch = "x86_64")))]
pub unsafe extern "system" fn RtlpCaptureStackLimits(
    Ebp: ULONG_PTR,
    StackBegin: *mut ULONG_PTR,
    StackEnd: *mut ULONG_PTR,
) -> BOOLEAN {
    let StackBegin = opaque_ptr(StackBegin);
    let StackEnd = opaque_ptr(StackEnd);
    let Thread = KeGetCurrentThread();

    if KeGetCurrentIrql() > DISPATCH_LEVEL {
        return 0u8;
    }

    *StackBegin = (*Thread).StackLimit as ULONG_PTR;
    *StackEnd = (*Thread).StackBase as ULONG_PTR;

    if (*StackBegin <= Ebp) && (Ebp <= *StackEnd) {
        *StackBegin = Ebp;
    } else {
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        {
            *StackEnd = (*(*KeGetPcr()).__bindgen_anon_1.__bindgen_anon_1.CurrentPrcb).DpcStack as ULONG_PTR;
        }
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            *StackEnd = (*(*KeGetPcr()).Prcb).DpcStack as ULONG_PTR;
        }
        *StackBegin = *StackEnd - KERNEL_STACK_SIZE as ULONG_PTR;

        if (*StackEnd != 0) && (*StackBegin < Ebp) && (Ebp <= *StackEnd) {
            *StackBegin = Ebp;
        } else {
            *StackBegin = Ebp;
            *StackEnd = PAGE_ALIGN(*StackBegin) as ULONG_PTR;
        }
    }

    1u8
}

#[cfg(not(any(target_arch = "arm", target_arch = "x86_64")))]
#[no_mangle]
pub unsafe extern "system" fn RtlWalkFrameChain(
    Callers: *mut PVOID,
    Count: ULONG,
    Flags: ULONG,
) -> ULONG {
    let Callers = opaque_ptr(Callers);
    // RtlWalkFrameChain(NULL, ...) writes no frames and must return 0. The C reference
    // wrapped the whole walk in _SEH2_TRY/_SEH2_EXCEPT{ i = 0 } so the fault from the
    // NULL output store (`*Callers.add(i) = ...` below) was caught and it returned 0.
    // That __except is DEAD under USE_DUMMY_PSEH / native SEH, so the NULL store AVs ->
    // BUGCHECK 0x1E at RtlWalkFrameChain+0x2b0 (the RtlStackKM/RtlStack quarantine).
    // Replicate the dropped handler explicitly. The walk itself is already bounds-checked
    // against StackBegin/StackEnd, so the NULL store was the only unguarded write.
    if Callers.is_null() {
        return 0;
    }
    let mut Stack: ULONG_PTR;
    let mut NewStack: ULONG_PTR;
    let mut StackBegin: ULONG_PTR;
    let mut StackEnd: ULONG_PTR = 0;
    let mut Eip: ULONG;
    let Result: BOOLEAN;
    let mut StopSearch = false;
    let mut i: ULONG = 0;
    let Thread = PsGetCurrentThread();
    let mut Teb: PTEB;
    let mut TrapFrame: PKTRAP_FRAME;

    #[cfg(target_arch = "x86")]
    {
        core::arch::asm!("mov {}, ebp", out(reg) Stack);
    }
    #[cfg(target_arch = "mips")]
    {
        core::arch::asm!("move {}, $sp", out(reg) Stack);
    }
    #[cfg(target_arch = "powerpc")]
    {
        core::arch::asm!("mr {}, 1", out(reg) Stack);
    }
    #[cfg(target_arch = "arm")]
    {
        core::arch::asm!("mov {}, sp", out(reg) Stack);
    }
    #[cfg(target_arch = "aarch64")]
    {
        core::arch::asm!("mov {}, sp", out(reg) Stack);
    }

    StackBegin = Stack;

    if Flags == 0 {
        Result = RtlpCaptureStackLimits(
            Stack,
            core::ptr::addr_of_mut!(StackBegin),
            core::ptr::addr_of_mut!(StackEnd),
        );
        if Result == 0 {
            return 0;
        }
    }

    if Flags == 1 {
        TrapFrame = KeGetTrapFrame(core::ptr::addr_of_mut!((*Thread).Tcb));
        Teb = (*Thread).Tcb.Teb;

        if Teb.is_null()
            || (KeIsAttachedProcess() != 0)
            || (KeGetCurrentIrql() >= DISPATCH_LEVEL)
        {
            return 0;
        }

        StackBegin = (*Teb).NtTib.StackLimit as ULONG_PTR;
        StackEnd = (*Teb).NtTib.StackBase as ULONG_PTR;
        
        #[cfg(target_arch = "x86")]
        {
            Stack = (*TrapFrame).Ebp as ULONG_PTR;
        }
        #[cfg(target_arch = "x86_64")]
        {
            Stack = (*TrapFrame).Rbp;
        }
        #[cfg(target_arch = "aarch64")]
        {
            Stack = (*TrapFrame).Fp as ULONG_PTR;
        }
        #[cfg(target_arch = "arm")]
        {
            Stack = (*TrapFrame).R11 as ULONG_PTR;
        }
        #[cfg(target_arch = "powerpc")]
        {
            Stack = (*TrapFrame).Gpr1 as ULONG_PTR;
        }

        if StackEnd <= StackBegin {
            return 0;
        }
        // C: the walk is one _SEH2_TRY whose handler sets `i = 0`; a bad stack
        // returns no frames. Same for the frame reads below.
        if ntkrnl_sys::probe::probe_user_buffer(StackBegin as *const core::ffi::c_void, StackEnd - StackBegin, 1, 1)
            .is_err()
        {
            return 0;
        }
    }

    i = 0;
    while i < Count {
        if (Stack >= StackEnd)
            || (if i == 0 { Stack < StackBegin } else { Stack <= StackBegin })
            || ((StackEnd - Stack) < (2 * core::mem::size_of::<ULONG_PTR>()))
        {
            break;
        }

        NewStack = match ntkrnl_sys::probe::probe_read_generic(Stack as *const ULONG_PTR, 1, 0) {
            Ok(value) => value,
            Err(_) => return 0,
        };
        Eip = match ntkrnl_sys::probe::probe_read_generic(
            (Stack + core::mem::size_of::<ULONG_PTR>()) as *const ULONG,
            1,
            0,
        ) {
            Ok(value) => value,
            Err(_) => return 0,
        };

        if !((Stack < NewStack) && (NewStack < StackEnd)) {
            StopSearch = true;
        }

        if (StackBegin < Eip as ULONG_PTR) && ((Eip as ULONG_PTR) < StackEnd) {
            break;
        }

        if (Flags == 0) && ((Eip & 0x80000000) == 0) {
            break;
        }

        *Callers.add(i as usize) = Eip as PVOID;

        if StopSearch {
            i += 1;
            break;
        }

        Stack = NewStack;
        i += 1;
    }

    i
}

#[no_mangle]
pub unsafe extern "system" fn RtlpGetStackLimits(LowLimit: *mut ULONG_PTR, HighLimit: *mut ULONG_PTR) {
    let LowLimit = opaque_ptr(LowLimit);
    let HighLimit = opaque_ptr(HighLimit);
    let CurrentThread = KeGetCurrentThread();
    *LowLimit = (*CurrentThread).StackLimit as ULONG_PTR;
    #[cfg(target_arch = "x86")]
    {
        // The i386 NPX save area sits at the top of the kernel stack, so the
        // usable stack ends BELOW it. Size comes from the bindgen type, never a
        // hand-written number: the local `[u8; 512]` stand-in this replaced was
        // 16 bytes short (real FX_SAVE_AREA is 528 — FXSAVE_FORMAT is 520 with
        // its Align16Byte tail, plus NpxSavedCpu + Cr0NpxState). That made
        // StackHigh 16 bytes too high, so `RtlDispatchException`'s
        // "is this registration frame really on the stack?" bounds check — the
        // guard that stops a corrupt/hostile fs:[0] chain from being followed —
        // accepted a 16-byte window past the true top of stack.
        *HighLimit = ((*CurrentThread).InitialStack as ULONG_PTR)
            - core::mem::size_of::<ntkrnl_sys::ddk::FX_SAVE_AREA>();
    }
    #[cfg(not(target_arch = "x86"))]
    {
        *HighLimit = (*CurrentThread).InitialStack as ULONG_PTR;
    }
}

#[no_mangle]
pub unsafe extern "C" fn RtlpInitAtomTableLock(AtomTable: PRTL_ATOM_TABLE) -> NTSTATUS {
    ExInitializeFastMutex(core::ptr::addr_of_mut!((*AtomTable).__bindgen_anon_1.FastMutex));
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn RtlpDestroyAtomTableLock(_AtomTable: PRTL_ATOM_TABLE) {
}

#[no_mangle]
pub unsafe extern "C" fn RtlpLockAtomTable(AtomTable: PRTL_ATOM_TABLE) -> BOOLEAN {
    ExAcquireFastMutex(core::ptr::addr_of_mut!((*AtomTable).__bindgen_anon_1.FastMutex));
    1u8
}

#[no_mangle]
pub unsafe extern "C" fn RtlpUnlockAtomTable(AtomTable: PRTL_ATOM_TABLE) {
    ExReleaseFastMutex(core::ptr::addr_of_mut!((*AtomTable).__bindgen_anon_1.FastMutex));
}

#[no_mangle]
pub unsafe extern "C" fn RtlpCreateAtomHandleTable(AtomTable: PRTL_ATOM_TABLE) -> BOOLEAN {
    (*AtomTable).__bindgen_anon_2.ExHandleTable = ExCreateHandleTable(core::ptr::null_mut());
    if (*AtomTable).__bindgen_anon_2.ExHandleTable.is_null() { 0u8 } else { 1u8 }
}

#[no_mangle]
pub unsafe extern "system" fn RtlpCloseHandleCallback(
    HandleTableEntry: PHANDLE_TABLE_ENTRY,
    Handle: HANDLE,
    HandleTable: PVOID,
) -> BOOLEAN {
    let HandleTableEntry = opaque_ptr(HandleTableEntry);
    let Handle = opaque_ptr(Handle);
    let HandleTable = opaque_ptr(HandleTable);
    ExDestroyHandle(HandleTable as *mut _, Handle, HandleTableEntry)
}

#[no_mangle]
pub unsafe extern "C" fn RtlpDestroyAtomHandleTable(AtomTable: PRTL_ATOM_TABLE) {
    if !(*AtomTable).__bindgen_anon_2.ExHandleTable.is_null() {
        ExSweepHandleTable(
            (*AtomTable).__bindgen_anon_2.ExHandleTable,
            core::mem::transmute(RtlpCloseHandleCallback as usize),
            (*AtomTable).__bindgen_anon_2.ExHandleTable as PVOID,
        );
        ExDestroyHandleTable((*AtomTable).__bindgen_anon_2.ExHandleTable, core::ptr::null_mut());
        (*AtomTable).__bindgen_anon_2.ExHandleTable = core::ptr::null_mut();
    }
}

#[no_mangle]
pub unsafe extern "C" fn RtlpAllocAtomTable(Size: ULONG) -> PRTL_ATOM_TABLE {
    let Table = ExAllocatePoolWithTag(NonPagedPool, Size as SIZE_T, TAG_ATMT) as PRTL_ATOM_TABLE;
    if !Table.is_null() {
        RtlZeroMemory(Table as PVOID, Size as SIZE_T);
    }
    Table
}

#[no_mangle]
pub unsafe extern "C" fn RtlpFreeAtomTable(AtomTable: PRTL_ATOM_TABLE) {
    ExFreePoolWithTag(AtomTable as PVOID, TAG_ATMT);
}

#[no_mangle]
pub unsafe extern "C" fn RtlpAllocAtomTableEntry(Size: ULONG) -> PRTL_ATOM_TABLE_ENTRY {
    let Entry = ExAllocatePoolWithTag(NonPagedPool, Size as SIZE_T, TAG_ATMT) as PRTL_ATOM_TABLE_ENTRY;
    if !Entry.is_null() {
        RtlZeroMemory(Entry as PVOID, Size as SIZE_T);
    }
    Entry
}

#[no_mangle]
pub unsafe extern "C" fn RtlpFreeAtomTableEntry(Entry: PRTL_ATOM_TABLE_ENTRY) {
    ExFreePoolWithTag(Entry as PVOID, TAG_ATMT);
}

#[no_mangle]
pub unsafe extern "C" fn RtlpFreeAtomHandle(AtomTable: PRTL_ATOM_TABLE, Entry: PRTL_ATOM_TABLE_ENTRY) {
    ExDestroyHandle(
        (*AtomTable).__bindgen_anon_2.ExHandleTable,
        (((*Entry).HandleIndex as ULONG_PTR) << 2) as HANDLE,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "C" fn RtlpCreateAtomHandle(AtomTable: PRTL_ATOM_TABLE, Entry: PRTL_ATOM_TABLE_ENTRY) -> BOOLEAN {
    let mut ExEntry: _HANDLE_TABLE_ENTRY = core::mem::zeroed();
    let Handle: HANDLE;
    let HandleIndex: USHORT;

    ExEntry.__bindgen_anon_1.Object = Entry as PVOID;
    ExEntry.__bindgen_anon_2.GrantedAccess = 0x1;

    Handle = ExCreateHandle((*AtomTable).__bindgen_anon_2.ExHandleTable, core::ptr::addr_of_mut!(ExEntry));
    if Handle.is_null() {
        return 0u8;
    }

    HandleIndex = ((Handle as ULONG_PTR) >> 2) as USHORT;

    if HandleIndex >= 0xC000 {
        ExDestroyHandle((*AtomTable).__bindgen_anon_2.ExHandleTable, Handle, core::ptr::null_mut());
        return 0u8;
    }

    (*Entry).HandleIndex = HandleIndex;
    (*Entry).Atom = 0xC000 + HandleIndex;

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn RtlpGetAtomEntry(AtomTable: PRTL_ATOM_TABLE, Index: ULONG) -> PRTL_ATOM_TABLE_ENTRY {
    let ExEntry: PHANDLE_TABLE_ENTRY;
    let mut Entry: PRTL_ATOM_TABLE_ENTRY = core::ptr::null_mut();

    ExEntry = ExMapHandleToPointer(
        (*AtomTable).__bindgen_anon_2.ExHandleTable,
        ((Index as ULONG_PTR) << 2) as HANDLE,
    );
    
    if !ExEntry.is_null() {
        Entry = (*ExEntry).__bindgen_anon_1.Object as PRTL_ATOM_TABLE_ENTRY;
        ExUnlockHandleTableEntry((*AtomTable).__bindgen_anon_2.ExHandleTable, ExEntry);
    }

    Entry
}

extern "system" {
    fn RtlpImageNtHeaderEx(
        Flags: ULONG,
        Base: PVOID,
        Size: DWORD64,
        OutHeaders: *mut PIMAGE_NT_HEADERS,
    ) -> NTSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn RtlImageNtHeaderEx(
    Flags: ULONG,
    Base: PVOID,
    Size: DWORD64,
    OutHeaders: *mut PIMAGE_NT_HEADERS,
) -> NTSTATUS {
    let Base = opaque_ptr(Base);
    let OutHeaders = opaque_ptr(OutHeaders);
    let mut Status: NTSTATUS;

    if !OutHeaders.is_null() {
        *OutHeaders = core::ptr::null_mut();
    }

    Status = RtlpImageNtHeaderEx(Flags, Base, Size, OutHeaders);

    Status
}

#[no_mangle]
pub unsafe extern "C" fn find_entry(
    BaseAddress: PVOID,
    info: *mut LDR_RESOURCE_INFO,
    level: ULONG,
    ret: *mut PVOID,
    want_dir: i32,
) -> NTSTATUS {
    let BaseAddress = opaque_ptr(BaseAddress);
    let info = opaque_ptr(info);
    let ret = opaque_ptr(ret);
    let mut size: ULONG = 0;
    let root: PVOID;
    let mut resdirptr: *mut IMAGE_RESOURCE_DIRECTORY;
    let mut current_level = level;

    root = RtlImageDirectoryEntryToData(
        BaseAddress,
        1u8,
        IMAGE_DIRECTORY_ENTRY_RESOURCE,
        core::ptr::addr_of_mut!(size),
    );
    
    if root.is_null() {
        return STATUS_RESOURCE_DATA_NOT_FOUND;
    }
    if size < core::mem::size_of::<IMAGE_RESOURCE_DIRECTORY>() as ULONG {
        return STATUS_RESOURCE_DATA_NOT_FOUND;
    }
    resdirptr = root as *mut IMAGE_RESOURCE_DIRECTORY;

    if current_level == 0 {
        *ret = resdirptr as PVOID;
        return STATUS_SUCCESS;
    }
    current_level -= 1;

    *ret = find_entry_by_name(
        resdirptr,
        (*info).Type as LPCWSTR,
        root,
        if want_dir != 0 || current_level > 0 { 1 } else { 0 },
    ) as PVOID;
    
    if (*ret).is_null() {
        return STATUS_RESOURCE_TYPE_NOT_FOUND;
    }
    if current_level == 0 {
        return STATUS_SUCCESS;
    }
    current_level -= 1;

    resdirptr = *ret as *mut IMAGE_RESOURCE_DIRECTORY;
    *ret = find_entry_by_name(
        resdirptr,
        (*info).Name as LPCWSTR,
        root,
        if want_dir != 0 || current_level > 0 { 1 } else { 0 },
    ) as PVOID;
    
    if (*ret).is_null() {
        return STATUS_RESOURCE_NAME_NOT_FOUND;
    }
    if current_level == 0 {
        return STATUS_SUCCESS;
    }
    current_level -= 1;
    
    if current_level > 0 {
        return STATUS_INVALID_PARAMETER;
    }

    resdirptr = *ret as *mut IMAGE_RESOURCE_DIRECTORY;
    *ret = find_first_entry(resdirptr, root, want_dir) as PVOID;
    
    if !(*ret).is_null() {
        return STATUS_SUCCESS;
    }

    STATUS_RESOURCE_DATA_NOT_FOUND
}

#[no_mangle]
pub unsafe extern "system" fn RtlpSafeCopyMemory(
    Destination: PVOID,
    Source: PVOID,
    Length: SIZE_T,
) -> NTSTATUS {
    let Destination = opaque_ptr(Destination);
    let Source = opaque_ptr(Source);
    RtlCopyMemory(Destination, Source, Length);
    STATUS_SUCCESS
}

/* RtlCallVectoredExceptionHandlers, RtlCallVectoredContinueHandlers, and
 * RtlpLookupDynamicFunctionEntry are provided by sdk/lib/rtl/ (vectoreh.c,
 * amd64/dynfntbl.c). Do NOT define them here — they belong to the SDK RTL,
 * not the kernel's ntoskrnl/rtl/ support files. */

// ============================================================================
// Wave THR/PRC/ENV kernel-side libsupp helpers — stub forwarders.
//
// These mirror the user-mode helpers in dll/ntdll/rtl/libsupp.c that exist
// to let the Rust shims (env.rs / thread.rs / process.rs in
// sdk/lib/rust/rtl-strings) read fields out of PEB / TEB / IMAGE_NT_HEADERS
// without mirroring those layouts in Rust. In kernel mode the wrapping Rust
// fns (RtlAcquirePrivilege, RtlExitUserThread, RtlCreateUserProcess, etc.)
// are user-mode primitives — they get linked in but are never reached on a
// kernel-mode codepath. The stubs exist only so the linker can resolve the
// extern fn references from librtl_rust.a.
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn RtlpGetPebEnvironment() -> PVOID {
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn RtlpSetPebEnvironment(_new_env: PVOID) {}

#[no_mangle]
pub unsafe extern "system" fn RtlpThrGetPebImageBaseAddress() -> PVOID {
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn RtlpThrGetPebMinimumStackCommit() -> SIZE_T {
    0
}

#[no_mangle]
pub unsafe extern "system" fn RtlpThrGetPebNtGlobalFlag() -> ULONG {
    0
}

#[no_mangle]
pub unsafe extern "system" fn RtlpThrSetFreeStackOnTermination() {}

#[no_mangle]
pub unsafe extern "system" fn RtlpThrTebDeallocStackAddr(_teb_base: PVOID) -> PVOID {
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn RtlpThrImageStackReserve(_headers: PVOID) -> SIZE_T {
    0
}

#[no_mangle]
pub unsafe extern "system" fn RtlpThrImageStackCommit(_headers: PVOID) -> SIZE_T {
    0
}

#[no_mangle]
pub unsafe extern "system" fn RtlpThrCurrentTeb() -> PVOID {
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn RtlpPrcGetPebOSMajorVersion() -> ULONG {
    0
}

#[no_mangle]
pub unsafe extern "system" fn RtlpPrcGetPebOSMinorVersion() -> ULONG {
    0
}

#[no_mangle]
pub unsafe extern "system" fn RtlpPrcPebProcessParametersAddr(peb_base: PVOID) -> PVOID {
    // `&((PPEB)PebBase)->ProcessParameters`. This is pure pointer arithmetic
    // on the *target* process's PEB base (a user VA passed in by the caller),
    // NOT a dereference of the current kernel process — so it is valid in
    // kernel mode. Returning NULL here (the old stub) made
    // RtlpInitEnvironment's third ZwWriteVirtualMemory target address 0,
    // faulting in MmCopyVirtualMemory (uncatchable under DUMMY_PSEH) →
    // BUGCHECK 0x1E during the kernel-side creation of the first user process.
    if peb_base.is_null() {
        return core::ptr::null_mut();
    }
    core::ptr::addr_of_mut!((*(peb_base as PPEB)).ProcessParameters) as PVOID
}

/// Kernel-mode `RtlGetProcessHeap()` — no per-process heap exists in
/// kernel mode. Returns NULL. The C macro `(NtCurrentPeb()->ProcessHeap)`
/// is the user-mode definition; this is the kernel stub equivalent.
#[no_mangle]
pub unsafe extern "system" fn RtlGetProcessHeap() -> PVOID {
    core::ptr::null_mut()
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS kernel
//  * FILE:            ntoskrnl/rtl/libsupp.c
//  * PURPOSE:         RTL Support Routines
//  * PROGRAMMERS:     Alex Ionescu (alex@relsoft.net)
//  *                  Gunnar Dalsnes
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// extern ULONG NtGlobalFlag;
//
// typedef struct _RTL_RANGE_ENTRY
// {
//     LIST_ENTRY Entry;
//     RTL_RANGE Range;
// } RTL_RANGE_ENTRY, *PRTL_RANGE_ENTRY;
//
// PAGED_LOOKASIDE_LIST RtlpRangeListEntryLookasideList;
// SIZE_T RtlpAllocDeallocQueryBufferSize = 128;
//
// /* FUNCTIONS *****************************************************************/
//
// PVOID
// NTAPI
// RtlPcToFileHeader(
//     IN  PVOID PcValue,
//     OUT PVOID *BaseOfImage)
// {
//     PLDR_DATA_TABLE_ENTRY LdrEntry;
//     BOOLEAN InSystem;
//     KIRQL OldIrql;
//
//     /* Get the base for this file */
//     if ((ULONG_PTR)PcValue > (ULONG_PTR)MmHighestUserAddress)
//     {
//         /* Acquire the loaded module spinlock */
//         KeAcquireSpinLock(&PsLoadedModuleSpinLock, &OldIrql);
//
//         /* We are in kernel */
//         *BaseOfImage = KiPcToFileHeader(PcValue, &LdrEntry, FALSE, &InSystem);
//
//         /* Release lock */
//         KeReleaseSpinLock(&PsLoadedModuleSpinLock, OldIrql);
//     }
//     else
//     {
//         /* User mode is not handled here! */
//         *BaseOfImage = NULL;
//     }
//
//     return *BaseOfImage;
// }
//
// VOID
// NTAPI
// RtlInitializeRangeListPackage(VOID)
// {
//     /* Setup the lookaside list for allocations (not used yet) */
//     ExInitializePagedLookasideList(&RtlpRangeListEntryLookasideList,
//                                    NULL,
//                                    NULL,
//                                    POOL_COLD_ALLOCATION,
//                                    sizeof(RTL_RANGE_ENTRY),
//                                    'elRR',
//                                    16);
// }
//
// BOOLEAN
// NTAPI
// RtlpCheckForActiveDebugger(VOID)
// {
//     /* This check is meaningless in kernel-mode */
//     return FALSE;
// }
//
// BOOLEAN
// NTAPI
// RtlpSetInDbgPrint(VOID)
// {
//     /* Nothing to set in kernel mode */
//     return FALSE;
// }
//
// VOID
// NTAPI
// RtlpClearInDbgPrint(VOID)
// {
//     /* Nothing to clear in kernel mode */
// }
//
// KPROCESSOR_MODE
// NTAPI
// RtlpGetMode(VOID)
// {
//    return KernelMode;
// }
//
// PVOID
// NTAPI
// RtlpAllocateMemory(ULONG Bytes,
//                    ULONG Tag)
// {
//     return ExAllocatePoolWithTag(PagedPool,
//                                  (SIZE_T)Bytes,
//                                  Tag);
// }
//
// VOID
// NTAPI
// RtlpFreeMemory(PVOID Mem,
//                ULONG Tag)
// {
//     if (Tag == TAG_ASTR || Tag == TAG_OSTR || Tag == TAG_USTR)
//         ExFreePool(Mem);
//     else
//         ExFreePoolWithTag(Mem, Tag);
// }
//
// /*
//  * @implemented
//  */
// VOID NTAPI
// RtlAcquirePebLock(VOID)
// {
//
// }
//
// /*
//  * @implemented
//  */
// VOID NTAPI
// RtlReleasePebLock(VOID)
// {
//
// }
//
// NTSTATUS
// NTAPI
// LdrShutdownThread(VOID)
// {
//     return STATUS_SUCCESS;
// }
//
//
// PPEB
// NTAPI
// RtlGetCurrentPeb(VOID)
// {
//    return ((PEPROCESS)(KeGetCurrentThread()->ApcState.Process))->Peb;
// }
//
// NTSTATUS
// NTAPI
// RtlDeleteHeapLock(IN OUT PHEAP_LOCK Lock)
// {
//     ExDeleteResourceLite(&Lock->Resource);
//     ExFreePoolWithTag(Lock, TAG_RTHL);
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// RtlEnterHeapLock(IN OUT PHEAP_LOCK Lock, IN BOOLEAN Exclusive)
// {
//     KeEnterCriticalRegion();
//
//     if (Exclusive)
//         ExAcquireResourceExclusiveLite(&Lock->Resource, TRUE);
//     else
//         ExAcquireResourceSharedLite(&Lock->Resource, TRUE);
//
//     return STATUS_SUCCESS;
// }
//
// BOOLEAN
// NTAPI
// RtlTryEnterHeapLock(IN OUT PHEAP_LOCK Lock, IN BOOLEAN Exclusive)
// {
//     BOOLEAN Success;
//     KeEnterCriticalRegion();
//
//     if (Exclusive)
//         Success = ExAcquireResourceExclusiveLite(&Lock->Resource, FALSE);
//     else
//         Success = ExAcquireResourceSharedLite(&Lock->Resource, FALSE);
//
//     if (!Success)
//         KeLeaveCriticalRegion();
//
//     return Success;
// }
//
// NTSTATUS
// NTAPI
// RtlInitializeHeapLock(IN OUT PHEAP_LOCK *Lock)
// {
//     PHEAP_LOCK HeapLock = ExAllocatePoolWithTag(NonPagedPool,
//                                                 sizeof(HEAP_LOCK),
//                                                 TAG_RTHL);
//     if (HeapLock == NULL)
//         return STATUS_NO_MEMORY;
//
//     ExInitializeResourceLite(&HeapLock->Resource);
//     *Lock = HeapLock;
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// RtlLeaveHeapLock(IN OUT PHEAP_LOCK Lock)
// {
//     ExReleaseResourceLite(&Lock->Resource);
//     KeLeaveCriticalRegion();
//
//     return STATUS_SUCCESS;
// }
//
// struct _HEAP;
//
// VOID
// NTAPI
// RtlpAddHeapToProcessList(struct _HEAP *Heap)
// {
//     UNREFERENCED_PARAMETER(Heap);
// }
//
// VOID
// NTAPI
// RtlpRemoveHeapFromProcessList(struct _HEAP *Heap)
// {
//     UNREFERENCED_PARAMETER(Heap);
// }
//
// VOID
// RtlInitializeHeapManager(VOID)
// {
// }
//
// #if DBG
// VOID FASTCALL
// CHECK_PAGED_CODE_RTL(char *file, int line)
// {
//   if(KeGetCurrentIrql() > APC_LEVEL)
//   {
//     DbgPrint("%s:%i: Pagable code called at IRQL > APC_LEVEL (%u)\n", file, line, KeGetCurrentIrql());
//     ASSERT(FALSE);
//   }
// }
// #endif
//
// VOID
// NTAPI
// RtlpSetHeapParameters(IN PRTL_HEAP_PARAMETERS Parameters)
// {
//     /* Apply defaults for non-set parameters */
//     if (!Parameters->SegmentCommit) Parameters->SegmentCommit = MmHeapSegmentCommit;
//     if (!Parameters->SegmentReserve) Parameters->SegmentReserve = MmHeapSegmentReserve;
//     if (!Parameters->DeCommitFreeBlockThreshold) Parameters->DeCommitFreeBlockThreshold = MmHeapDeCommitFreeBlockThreshold;
//     if (!Parameters->DeCommitTotalFreeThreshold) Parameters->DeCommitTotalFreeThreshold = MmHeapDeCommitTotalFreeThreshold;
// }
//
// VOID
// NTAPI
// RtlpCheckLogException(IN PEXCEPTION_RECORD ExceptionRecord,
//                       IN PCONTEXT ContextRecord,
//                       IN PVOID ContextData,
//                       IN ULONG Size)
// {
//     /* Check the global flag */
//     if (NtGlobalFlag & FLG_ENABLE_EXCEPTION_LOGGING)
//     {
//         /* FIXME: Log this exception */
//     }
// }
//
// BOOLEAN
// NTAPI
// RtlpHandleDpcStackException(IN PEXCEPTION_REGISTRATION_RECORD RegistrationFrame,
//                             IN ULONG_PTR RegistrationFrameEnd,
//                             IN OUT PULONG_PTR StackLow,
//                             IN OUT PULONG_PTR StackHigh)
// {
//     PKPRCB Prcb;
//     ULONG_PTR DpcStack;
//
//     /* Check if we are at DISPATCH or higher */
//     if (KeGetCurrentIrql() >= DISPATCH_LEVEL)
//     {
//         /* Get the PRCB and DPC Stack */
//         Prcb = KeGetCurrentPrcb();
//         DpcStack = (ULONG_PTR)Prcb->DpcStack;
//
//         /* Check if we are in a DPC and the stack matches */
//         if ((Prcb->DpcRoutineActive) &&
//             (RegistrationFrameEnd <= DpcStack) &&
//             ((ULONG_PTR)RegistrationFrame >= DpcStack - KERNEL_STACK_SIZE))
//         {
//             /* Update the limits to the DPC Stack's */
//             *StackHigh = DpcStack;
//             *StackLow = DpcStack - KERNEL_STACK_SIZE;
//             return TRUE;
//         }
//     }
//
//     /* Not in DPC stack */
//     return FALSE;
// }
//
// #if !defined(_ARM_) && !defined(_AMD64_)
//
// BOOLEAN
// NTAPI
// RtlpCaptureStackLimits(IN ULONG_PTR Ebp,
//                        IN ULONG_PTR *StackBegin,
//                        IN ULONG_PTR *StackEnd)
// {
//     PKTHREAD Thread = KeGetCurrentThread();
//
//     /* Don't even try at ISR level or later */
//     if (KeGetCurrentIrql() > DISPATCH_LEVEL) return FALSE;
//
//     /* Start with defaults */
//     *StackBegin = Thread->StackLimit;
//     *StackEnd = (ULONG_PTR)Thread->StackBase;
//
//     /* Check if EBP is inside the stack */
//     if ((*StackBegin <= Ebp) && (Ebp <= *StackEnd))
//     {
//         /* Then make the stack start at EBP */
//         *StackBegin = Ebp;
//     }
//     else
//     {
//         /* Now we're going to assume we're on the DPC stack */
// #if defined(_M_AMD64) || defined(_M_ARM64)
//         *StackEnd = (ULONG_PTR)(KeGetPcr()->CurrentPrcb->DpcStack);
// #else
//         *StackEnd = (ULONG_PTR)(KeGetPcr()->Prcb->DpcStack);
// #endif
//         *StackBegin = *StackEnd - KERNEL_STACK_SIZE;
//
//         /* Check if we seem to be on the DPC stack */
//         if ((*StackEnd) && (*StackBegin < Ebp) && (Ebp <= *StackEnd))
//         {
//             /* We're on the DPC stack */
//             *StackBegin = Ebp;
//         }
//         else
//         {
//             /* We're somewhere else entirely... use EBP for safety */
//             *StackBegin = Ebp;
//             *StackEnd = (ULONG_PTR)PAGE_ALIGN(*StackBegin);
//         }
//     }
//
//     /* Return success */
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// ULONG
// NTAPI
// RtlWalkFrameChain(OUT PVOID *Callers,
//                   IN ULONG Count,
//                   IN ULONG Flags)
// {
//     ULONG_PTR Stack, NewStack, StackBegin, StackEnd = 0;
//     ULONG Eip;
//     BOOLEAN Result, StopSearch = FALSE;
//     ULONG i = 0;
//     PETHREAD Thread = PsGetCurrentThread();
//     PTEB Teb;
//     PKTRAP_FRAME TrapFrame;
//
//     /* Get current EBP */
// #if defined(_M_IX86)
// #if defined __GNUC__
//     __asm__("mov %%ebp, %0" : "=r" (Stack) : );
// #elif defined(_MSC_VER)
//     __asm mov Stack, ebp
// #endif
// #elif defined(_M_MIPS)
//         __asm__("move $sp, %0" : "=r" (Stack) : );
// #elif defined(_M_PPC)
//     __asm__("mr %0,1" : "=r" (Stack) : );
// #elif defined(_M_ARM)
//     __asm__("mov sp, %0" : "=r"(Stack) : );
// #elif defined(_M_ARM64)
//     __asm__("mov %0, sp" : "=r"(Stack));
// #else
// #error Unknown architecture
// #endif
//
//     /* Set it as the stack begin limit as well */
//     StackBegin = (ULONG_PTR)Stack;
//
//     /* Check if we're called for non-logging mode */
//     if (!Flags)
//     {
//         /* Get the actual safe limits */
//         Result = RtlpCaptureStackLimits((ULONG_PTR)Stack,
//                                         &StackBegin,
//                                         &StackEnd);
//         if (!Result) return 0;
//     }
//
//     /* Use a SEH block for maximum protection */
//     _SEH2_TRY
//     {
//         /* Check if we want the user-mode stack frame */
//         if (Flags == 1)
//         {
//             /* Get the trap frame and TEB */
//             TrapFrame = KeGetTrapFrame(&Thread->Tcb);
//             Teb = Thread->Tcb.Teb;
//
//             /* Make sure we can trust the TEB and trap frame */
//             if (!(Teb) ||
//                 (KeIsAttachedProcess()) ||
//                 (KeGetCurrentIrql() >= DISPATCH_LEVEL))
//             {
//                 /* Invalid or unsafe attempt to get the stack */
//                 _SEH2_YIELD(return 0;)
//             }
//
//             /* Get the stack limits */
//             StackBegin = (ULONG_PTR)Teb->NtTib.StackLimit;
//             StackEnd = (ULONG_PTR)Teb->NtTib.StackBase;
// #ifdef _M_IX86
//             Stack = TrapFrame->Ebp;
// #elif defined(_M_AMD64)
//             Stack = TrapFrame->Rbp;
// #elif defined(_M_ARM64)
//             Stack = TrapFrame->Fp;
// #elif defined(_M_ARM)
//             Stack = TrapFrame->R11;
// #elif defined(_M_PPC)
//             Stack = TrapFrame->Gpr1;
// #else
// #error Unknown architecture
// #endif
//
//             /* Validate them */
//             if (StackEnd <= StackBegin) _SEH2_YIELD(return 0);
//             ProbeForRead((PVOID)StackBegin,
//                          StackEnd - StackBegin,
//                          sizeof(CHAR));
//         }
//
//         /* Loop the frames */
//         for (i = 0; i < Count; i++)
//         {
//             /*
//              * Leave if we're past the stack,
//              * if we're before the stack,
//              * or if we've reached ourselves.
//              */
//             if ((Stack >= StackEnd) ||
//                 (!i ? (Stack < StackBegin) : (Stack <= StackBegin)) ||
//                 ((StackEnd - Stack) < (2 * sizeof(ULONG_PTR))))
//             {
//                 /* We're done or hit a bad address */
//                 break;
//             }
//
//             /* Get new stack and EIP */
//             NewStack = *(PULONG_PTR)Stack;
//             Eip = *(PULONG_PTR)(Stack + sizeof(ULONG_PTR));
//
//             /* Check if the new pointer is above the oldone and past the end */
//             if (!((Stack < NewStack) && (NewStack < StackEnd)))
//             {
//                 /* Stop searching after this entry */
//                 StopSearch = TRUE;
//             }
//
//             /* Also make sure that the EIP isn't a stack address */
//             if ((StackBegin < Eip) && (Eip < StackEnd)) break;
//
//             /* Check if we reached a user-mode address */
//             if (!(Flags) && !(Eip & 0x80000000)) break; // FIXME: 3GB breakage
//
//             /* Save this frame */
//             Callers[i] = (PVOID)Eip;
//
//             /* Check if we should continue */
//             if (StopSearch)
//             {
//                 /* Return the next index */
//                 i++;
//                 break;
//             }
//
//             /* Move to the next stack */
//             Stack = NewStack;
//         }
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         /* No index */
//         i = 0;
//     }
//     _SEH2_END;
//
//     /* Return frames parsed */
//     return i;
// }
//
// #endif
//
// VOID
// NTAPI
// RtlpGetStackLimits(
//     OUT PULONG_PTR LowLimit,
//     OUT PULONG_PTR HighLimit)
// {
//     PKTHREAD CurrentThread = KeGetCurrentThread();
//     *LowLimit = (ULONG_PTR)CurrentThread->StackLimit;
// #ifdef _M_IX86
//     *HighLimit = (ULONG_PTR)CurrentThread->InitialStack -
//         sizeof(FX_SAVE_AREA);
// #else
//     *HighLimit = (ULONG_PTR)CurrentThread->InitialStack;
// #endif
// }
//
// /* RTL Atom Tables ************************************************************/
//
// NTSTATUS
// RtlpInitAtomTableLock(PRTL_ATOM_TABLE AtomTable)
// {
//    ExInitializeFastMutex(&AtomTable->FastMutex);
//
//    return STATUS_SUCCESS;
// }
//
//
// VOID
// RtlpDestroyAtomTableLock(PRTL_ATOM_TABLE AtomTable)
// {
// }
//
//
// BOOLEAN
// RtlpLockAtomTable(PRTL_ATOM_TABLE AtomTable)
// {
//    ExAcquireFastMutex(&AtomTable->FastMutex);
//    return TRUE;
// }
//
// VOID
// RtlpUnlockAtomTable(PRTL_ATOM_TABLE AtomTable)
// {
//    ExReleaseFastMutex(&AtomTable->FastMutex);
// }
//
// BOOLEAN
// RtlpCreateAtomHandleTable(PRTL_ATOM_TABLE AtomTable)
// {
//    AtomTable->ExHandleTable = ExCreateHandleTable(NULL);
//    return (AtomTable->ExHandleTable != NULL);
// }
//
// BOOLEAN
// NTAPI
// RtlpCloseHandleCallback(
//     IN PHANDLE_TABLE_ENTRY HandleTableEntry,
//     IN HANDLE Handle,
//     IN PVOID HandleTable)
// {
//     /* Destroy and unlock the handle entry */
//     return ExDestroyHandle(HandleTable, Handle, HandleTableEntry);
// }
//
// VOID
// RtlpDestroyAtomHandleTable(PRTL_ATOM_TABLE AtomTable)
// {
//    if (AtomTable->ExHandleTable)
//    {
//       ExSweepHandleTable(AtomTable->ExHandleTable,
//                          RtlpCloseHandleCallback,
//                          AtomTable->ExHandleTable);
//       ExDestroyHandleTable(AtomTable->ExHandleTable, NULL);
//       AtomTable->ExHandleTable = NULL;
//    }
// }
//
// PRTL_ATOM_TABLE
// RtlpAllocAtomTable(ULONG Size)
// {
//    PRTL_ATOM_TABLE Table = ExAllocatePoolWithTag(NonPagedPool,
//                                                  Size,
//                                                  TAG_ATMT);
//    if (Table != NULL)
//    {
//       RtlZeroMemory(Table,
//                     Size);
//    }
//
//    return Table;
// }
//
// VOID
// RtlpFreeAtomTable(PRTL_ATOM_TABLE AtomTable)
// {
//    ExFreePoolWithTag(AtomTable, TAG_ATMT);
// }
//
// PRTL_ATOM_TABLE_ENTRY
// RtlpAllocAtomTableEntry(ULONG Size)
// {
//     PRTL_ATOM_TABLE_ENTRY Entry;
//
//     Entry = ExAllocatePoolWithTag(NonPagedPool, Size, TAG_ATMT);
//     if (Entry != NULL)
//     {
//         RtlZeroMemory(Entry, Size);
//     }
//
//     return Entry;
// }
//
// VOID
// RtlpFreeAtomTableEntry(PRTL_ATOM_TABLE_ENTRY Entry)
// {
//     ExFreePoolWithTag(Entry, TAG_ATMT);
// }
//
// VOID
// RtlpFreeAtomHandle(PRTL_ATOM_TABLE AtomTable, PRTL_ATOM_TABLE_ENTRY Entry)
// {
//    ExDestroyHandle(AtomTable->ExHandleTable,
//                    (HANDLE)((ULONG_PTR)Entry->HandleIndex << 2),
//                    NULL);
// }
//
// BOOLEAN
// RtlpCreateAtomHandle(PRTL_ATOM_TABLE AtomTable, PRTL_ATOM_TABLE_ENTRY Entry)
// {
//    HANDLE_TABLE_ENTRY ExEntry;
//    HANDLE Handle;
//    USHORT HandleIndex;
//
//    /* Initialize ex handle table entry */
//    ExEntry.Object = Entry;
//    ExEntry.GrantedAccess = 0x1; /* FIXME - valid handle */
//
//    /* Create ex handle */
//    Handle = ExCreateHandle(AtomTable->ExHandleTable,
//                            &ExEntry);
//    if (!Handle) return FALSE;
//
//    /* Calculate HandleIndex (by getting rid of the first two bits) */
//    HandleIndex = (USHORT)((ULONG_PTR)Handle >> 2);
//
//    /* Index must be less than 0xC000 */
//    if (HandleIndex >= 0xC000)
//    {
//        /* Destroy ex handle */
//        ExDestroyHandle(AtomTable->ExHandleTable,
//                        Handle,
//                        NULL);
//
//        /* Return failure */
//        return FALSE;
//    }
//
//    /* Initialize atom table entry */
//    Entry->HandleIndex = HandleIndex;
//    Entry->Atom = 0xC000 + HandleIndex;
//
//    /* Return success */
//    return TRUE;
// }
//
// PRTL_ATOM_TABLE_ENTRY
// RtlpGetAtomEntry(PRTL_ATOM_TABLE AtomTable, ULONG Index)
// {
//    PHANDLE_TABLE_ENTRY ExEntry;
//    PRTL_ATOM_TABLE_ENTRY Entry = NULL;
//
//    /* NOTE: There's no need to explicitly enter a critical region because it's
//             guaranteed that we're in a critical region right now (as we hold
//             the atom table lock) */
//
//    ExEntry = ExMapHandleToPointer(AtomTable->ExHandleTable,
//                                   (HANDLE)((ULONG_PTR)Index << 2));
//    if (ExEntry != NULL)
//    {
//       Entry = ExEntry->Object;
//
//       ExUnlockHandleTableEntry(AtomTable->ExHandleTable,
//                                ExEntry);
//    }
//
//    return Entry;
// }
//
// /* Ldr SEH-Protected access to IMAGE_NT_HEADERS */
//
// /* Rtl SEH-Free version of this */
// NTSTATUS
// NTAPI
// RtlpImageNtHeaderEx(
//     _In_ ULONG Flags,
//     _In_ PVOID Base,
//     _In_ ULONG64 Size,
//     _Out_ PIMAGE_NT_HEADERS *OutHeaders);
//
// /*
//  * @implemented
//  * @note: This is here, so that we do not drag SEH into rosload, freeldr and bootmgfw
//  */
// NTSTATUS
// NTAPI
// RtlImageNtHeaderEx(
//     _In_ ULONG Flags,
//     _In_ PVOID Base,
//     _In_ ULONG64 Size,
//     _Out_ PIMAGE_NT_HEADERS *OutHeaders)
// {
//     NTSTATUS Status;
//
//     /* Assume failure. This is also done in RtlpImageNtHeaderEx, but this is guarded by SEH. */
//     if (OutHeaders != NULL)
//         *OutHeaders = NULL;
//
//     _SEH2_TRY
//     {
//         Status = RtlpImageNtHeaderEx(Flags, Base, Size, OutHeaders);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         /* Fail with the SEH error */
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     return Status;
// }
//
// /*
//  * Ldr Resource support code
//  */
//
// IMAGE_RESOURCE_DIRECTORY *find_entry_by_name( IMAGE_RESOURCE_DIRECTORY *dir,
//                                               LPCWSTR name, void *root,
//                                               int want_dir );
// IMAGE_RESOURCE_DIRECTORY *find_entry_by_id( IMAGE_RESOURCE_DIRECTORY *dir,
//                                             USHORT id, void *root, int want_dir );
// IMAGE_RESOURCE_DIRECTORY *find_first_entry( IMAGE_RESOURCE_DIRECTORY *dir,
//                                             void *root, int want_dir );
//
// /**********************************************************************
//  *  find_entry
//  *
//  * Find a resource entry
//  */
// NTSTATUS find_entry( PVOID BaseAddress, LDR_RESOURCE_INFO *info,
//                      ULONG level, void **ret, int want_dir )
// {
//     ULONG size;
//     void *root;
//     IMAGE_RESOURCE_DIRECTORY *resdirptr;
//
//     root = RtlImageDirectoryEntryToData( BaseAddress, TRUE, IMAGE_DIRECTORY_ENTRY_RESOURCE, &size );
//     if (!root) return STATUS_RESOURCE_DATA_NOT_FOUND;
//     if (size < sizeof(*resdirptr)) return STATUS_RESOURCE_DATA_NOT_FOUND;
//     resdirptr = root;
//
//     if (!level--) goto done;
//     if (!(*ret = find_entry_by_name( resdirptr, (LPCWSTR)info->Type, root, want_dir || level )))
//         return STATUS_RESOURCE_TYPE_NOT_FOUND;
//     if (!level--) return STATUS_SUCCESS;
//
//     resdirptr = *ret;
//     if (!(*ret = find_entry_by_name( resdirptr, (LPCWSTR)info->Name, root, want_dir || level )))
//         return STATUS_RESOURCE_NAME_NOT_FOUND;
//     if (!level--) return STATUS_SUCCESS;
//     if (level) return STATUS_INVALID_PARAMETER;  /* level > 3 */
//
//     resdirptr = *ret;
//
//     if ((*ret = find_first_entry( resdirptr, root, want_dir ))) return STATUS_SUCCESS;
//
//     return STATUS_RESOURCE_DATA_NOT_FOUND;
//
// done:
//     *ret = resdirptr;
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// RtlpSafeCopyMemory(
//    _Out_writes_bytes_all_(Length) VOID UNALIGNED *Destination,
//    _In_reads_bytes_(Length) CONST VOID UNALIGNED *Source,
//    _In_ SIZE_T Length)
// {
//     _SEH2_TRY
//     {
//         RtlCopyMemory(Destination, Source, Length);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         _SEH2_YIELD(return _SEH2_GetExceptionCode());
//     }
//     _SEH2_END;
//
//     return STATUS_SUCCESS;
// }
//
// BOOLEAN
// NTAPI
// RtlCallVectoredExceptionHandlers(_In_ PEXCEPTION_RECORD ExceptionRecord,
//                                  _In_ PCONTEXT Context)
// {
//     /* In the kernel we don't have vectored exception handlers */
//     return FALSE;
// }
//
// VOID
// NTAPI
// RtlCallVectoredContinueHandlers(_In_ PEXCEPTION_RECORD ExceptionRecord,
//                                 _In_ PCONTEXT Context)
// {
//     /* No vectored continue handlers either in kernel mode */
//     return;
// }
//
// #ifdef _M_AMD64
//
// PRUNTIME_FUNCTION
// NTAPI
// RtlpLookupDynamicFunctionEntry(
//     _In_ DWORD64 ControlPc,
//     _Out_ PDWORD64 ImageBase,
//     _In_ PUNWIND_HISTORY_TABLE HistoryTable)
// {
//     /* No support for dynamic function tables in the kernel */
//     return NULL;
// }
//
// #endif
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
