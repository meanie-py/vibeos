use ntkrnl_sys::base::*;
use crate::frldr::*;
use crate::meminit::*;
use crate::mm::*;
use core::ffi::c_void;

// The C's `__rdtsc()` intrinsic, for the debug-only allocation timing below. Nothing
// declared it, so a debug `cargo check` of this crate never compiled.
#[cfg(all(debug_assertions, target_arch = "x86_64"))]
#[inline(always)]
unsafe fn __rdtsc() -> u64 {
    core::arch::x86_64::_rdtsc()
}
#[cfg(all(debug_assertions, target_arch = "x86"))]
#[inline(always)]
unsafe fn __rdtsc() -> u64 {
    core::arch::x86::_rdtsc()
}
extern "C" {
    fn TRACE(fmt: *const i8, ...);
    fn ERR(fmt: *const i8, ...);
    fn WARN(fmt: *const i8, ...);
}

// heap.c file-local types (rustconv dropped these typedefs — restored
// byte-exact from lib/mm/heap.c). Not in any public header.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct BLOCK_DATA {
    // C: ULONG_PTR Flink:32; ULONG_PTR Blink:32; — 8 bytes both arches.
    pub Flink: u32,
    pub Blink: u32,
}
pub type PBLOCK_DATA = *mut BLOCK_DATA;

#[repr(C)]
pub struct HEAP_BLOCK {
    pub Size: USHORT,
    pub PreviousSize: USHORT,
    pub Tag: ULONG,
    pub Data: [BLOCK_DATA; 0], // flexible array — use ptr arithmetic
}
pub type PHEAP_BLOCK = *mut HEAP_BLOCK;

#[repr(C)]
pub struct HEAP {
    pub MaximumSize: SIZE_T,
    pub CurrentAllocBytes: SIZE_T,
    pub MaxAllocBytes: SIZE_T,
    pub NumAllocs: ULONG,
    pub NumFrees: ULONG,
    pub LargestAllocation: SIZE_T,
    pub AllocationTime: ULONGLONG,
    pub FreeTime: ULONGLONG,
    pub TerminatingBlock: ULONG_PTR,
    pub Blocks: HEAP_BLOCK,
}
pub type PHEAP = *mut HEAP;

// heap.c REDZONE_* macros + file-static heap handles.
#[allow(non_snake_case)]
#[inline(always)]
pub unsafe fn REDZONE_SIZE(b: PHEAP_BLOCK) -> *mut u64 {
    (*b).Data.as_ptr() as *mut u64
}
#[allow(non_snake_case)] #[inline(always)]
pub unsafe fn REDZONE_LOW(b: PHEAP_BLOCK) -> *mut u64 { (REDZONE_SIZE(b)).add(1) }
#[allow(non_snake_case)] #[inline(always)]
pub unsafe fn REDZONE_HI(b: PHEAP_BLOCK) -> *mut u64 {
    ((*b).Data.as_ptr() as *mut u8).add(16 + *REDZONE_SIZE(b) as usize) as *mut u64 // nolint: magic-offset (freeldr heap redzone layout, crate-local struct)
}
#[no_mangle]
pub static mut FrLdrDefaultHeap: PVOID = core::ptr::null_mut();
#[no_mangle]
pub static mut FrLdrTempHeap: PVOID = core::ptr::null_mut();

#[no_mangle]
pub unsafe extern "C" fn FrLdrHeapCreate(
    MaximumSize: usize,
    MemoryType: TYPE_OF_MEMORY,
) -> PVOID {
    let Heap: PHEAP;
    let mut Block: PHEAP_BLOCK;
    let mut Remaining: usize;
    let mut PreviousSize: u16;
    let mut MaximumSize = MaximumSize;

    TRACE(b"HeapCreate(MemoryType=%ld)\n\0".as_ptr() as PCHAR, MemoryType);

    MaximumSize = ALIGN_UP_BY(MaximumSize, MM_PAGE_SIZE);
    Heap = MmAllocateMemoryWithType(MaximumSize, MemoryType) as PHEAP;
    if Heap.is_null() {
        ERR(b"HEAP: Failed to allocate heap of size 0x%lx, Type %lu\n\0".as_ptr() as PCHAR,
            MaximumSize, MemoryType);
        return core::ptr::null_mut();
    }

    (*Heap).MaximumSize = MaximumSize;
    (*Heap).CurrentAllocBytes = 0;
    (*Heap).MaxAllocBytes = 0;
    (*Heap).NumAllocs = 0;
    (*Heap).NumFrees = 0;
    (*Heap).LargestAllocation = 0;

    Remaining = (MaximumSize - core::mem::size_of::<HEAP>()) / core::mem::size_of::<HEAP_BLOCK>();
    TRACE(b"Remaining = %ld\n\0".as_ptr() as PCHAR, Remaining);

    Remaining -= 2;

    Block = core::ptr::addr_of_mut!((*Heap).Blocks);
    PreviousSize = 0;

    while Remaining > 1 {
        (*Block).Size = min(65535, Remaining - 1) as u16;
        (*Block).PreviousSize = PreviousSize;
        (*Block).Tag = 0;
        (*(*Block).Data.as_mut_ptr()).Flink = (Block.offset_from(core::ptr::addr_of_mut!((*Heap).Blocks)) as isize + (*Block).Size as isize + 1) as u32;
        (*(*Block).Data.as_mut_ptr()).Blink = (Block.offset_from(core::ptr::addr_of_mut!((*Heap).Blocks)) as isize - 1 - PreviousSize as isize) as u32;

        Remaining -= (*Block).Size as usize + 1 ;

        PreviousSize = (*Block).Size;
        Block = Block.offset((*Block).Size as isize + 1);

        TRACE(b"Remaining = %ld\n\0".as_ptr() as PCHAR, Remaining);
    }

    (*Heap).TerminatingBlock = Block.offset_from(core::ptr::addr_of_mut!((*Heap).Blocks)) as usize;
    (*Block).Size = 0;
    (*Block).PreviousSize = PreviousSize;
    (*Block).Tag = 0x23646E45;
    (*(*Block).Data.as_mut_ptr()).Flink = 0;
    (*(*Block).Data.as_mut_ptr()).Blink = (Block.offset_from(core::ptr::addr_of_mut!((*Heap).Blocks)) as isize - 1 - PreviousSize as isize) as u32;
    (*(*Heap).Blocks.Data.as_mut_ptr()).Blink = (*Heap).TerminatingBlock as u32;

    Heap as PVOID
}

#[no_mangle]
pub unsafe extern "C" fn FrLdrHeapDestroy(HeapHandle: PVOID) {
    let Heap = HeapHandle as PHEAP;

    MmMarkPagesInLookupTable(
        PageLookupTableAddress,
        Heap as ULONG_PTR / MM_PAGE_SIZE,
        ((*Heap).MaximumSize / MM_PAGE_SIZE) as PFN_NUMBER,
        LoaderFirmwareTemporary,
    );

    #[cfg(debug_assertions)]
    {
        core::ptr::write_bytes(
            Heap as *mut u8,
            0xCC,
            (*Heap).MaximumSize,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn FrLdrHeapVerify(
    HeapHandle: PVOID)
{
    let Heap = HeapHandle as PHEAP;
    let mut Block: PHEAP_BLOCK;

    Block = core::ptr::addr_of_mut!((*Heap).Blocks);
    loop {
        if (*Block).Size == 0 {
            break;
        }

        if (*Block).Tag != 0 {
            ntkrnl_sys::ASSERT!(*REDZONE_SIZE(Block) <= (*Block).Size as u64 * core::mem::size_of::<HEAP_BLOCK>() as u64);
            ntkrnl_sys::ASSERT!(*REDZONE_LOW(Block) == REDZONE_MARK);
            ntkrnl_sys::ASSERT!(*REDZONE_HI(Block) == REDZONE_MARK);
        }

        Block = Block.offset(1 + (*Block).Size as isize);
    }
}

#[no_mangle]
pub unsafe extern "C" fn FrLdrHeapRelease(HeapHandle: PVOID) {
    let Heap = HeapHandle as PHEAP;
    let mut Block: PHEAP_BLOCK;
    let mut StartAddress: PUCHAR;
    let mut EndAddress: PUCHAR;
    let mut FreePages: PFN_COUNT;
    let mut AllFreePages: PFN_COUNT = 0;

    TRACE(b"HeapRelease(%p)\n\0".as_ptr() as PCHAR, HeapHandle);

    Block = core::ptr::addr_of_mut!((*Heap).Blocks);
    
    loop {
        if (*Block).Size == 0 {
            break;
        }

        if (*Block).Tag != 0 {
            Block = (Block as usize + (1 + (*Block).Size as usize) * core::mem::size_of::<HEAP_BLOCK>()) as PHEAP_BLOCK;
            continue;
        }

        StartAddress = ALIGN_UP_POINTER_BY((*Block).Data.as_mut_ptr() as *mut u8, PAGE_SIZE);

        while (*Block).Tag == 0 {
            Block = (Block as usize + ((*Block).Size as usize + 1) * core::mem::size_of::<HEAP_BLOCK>()) as PHEAP_BLOCK;
        }

        if (*Block).Size == 0 {
            EndAddress = ALIGN_UP_POINTER_BY((*Block).Data.as_mut_ptr() as *mut u8, PAGE_SIZE);
        } else {
            EndAddress = ALIGN_DOWN_POINTER_BY((*Block).Data.as_mut_ptr() as *mut u8, PAGE_SIZE);
        }

        if EndAddress > StartAddress {
            FreePages = ((EndAddress as usize - StartAddress as usize) / MM_PAGE_SIZE) as PFN_COUNT;
            AllFreePages += FreePages;

            MmMarkPagesInLookupTable(
                PageLookupTableAddress,
                (StartAddress as ULONG_PTR) / MM_PAGE_SIZE as ULONG_PTR,
                FreePages as PFN_NUMBER,
                LoaderFree,
            );
        }

        if (*Block).Size == 0 {
            break;
        }

        Block = (Block as usize + (1 + (*Block).Size as usize) * core::mem::size_of::<HEAP_BLOCK>()) as PHEAP_BLOCK;
    }

    TRACE(
        b"HeapRelease() done, freed %lu of %lu pages\n\0".as_ptr() as PCHAR,
        AllFreePages,
        (*Heap).MaximumSize / MM_PAGE_SIZE,
    );
}

#[no_mangle]
pub unsafe extern "C" fn FrLdrHeapCleanupAll() {
    // `as PHEAP`: FrLdr*Heap are PVOID handles, and the C original's `#if DBG`
    // block had a PHEAP local. Both of these blocks are `cfg(debug_assertions)`,
    // which the release profile never set — so from the day this file was
    // translated until 2026-07-28 the code below was never once type-checked,
    // and the missing cast (14 x E0609 "no field on type c_void") sat here
    // invisible. Turning debug-assertions on is what compiled it for the first
    // time. A cfg'd-out branch is not "code that works"; it is code with no
    // evidence either way.
    #[cfg(debug_assertions)]
    {
        let Heap = FrLdrDefaultHeap as PHEAP;
        TRACE(
            b"Heap statistics for default heap:\nCurrentAlloc=0x%lx, MaxAlloc=0x%lx, LargestAllocation=0x%lx\nNumAllocs=%ld, NumFrees=%ld\n\0".as_ptr() as PCHAR,
            (*Heap).CurrentAllocBytes,
            (*Heap).MaxAllocBytes,
            (*Heap).LargestAllocation,
            (*Heap).NumAllocs,
            (*Heap).NumFrees,
        );
        TRACE(
            b"AllocTime = %I64d, FreeTime = %I64d, sum = %I64d\n\0".as_ptr() as PCHAR,
            (*Heap).AllocationTime,
            (*Heap).FreeTime,
            (*Heap).AllocationTime + (*Heap).FreeTime,
        );
    }

    FrLdrHeapRelease(FrLdrDefaultHeap);

    #[cfg(debug_assertions)]
    {
        let Heap = FrLdrTempHeap as PHEAP;
        TRACE(
            b"Heap statistics for temp heap:\nCurrentAlloc=0x%lx, MaxAlloc=0x%lx, LargestAllocation=0x%lx\nNumAllocs=%ld, NumFrees=%ld\n\0".as_ptr() as PCHAR,
            (*Heap).CurrentAllocBytes,
            (*Heap).MaxAllocBytes,
            (*Heap).LargestAllocation,
            (*Heap).NumAllocs,
            (*Heap).NumFrees,
        );
    }

    FrLdrHeapDestroy(FrLdrTempHeap);
}

unsafe fn FrLdrHeapRemoveFreeList(
    Heap: PHEAP,
    Block: PHEAP_BLOCK)
{
    let Previous: PHEAP_BLOCK;
    let Next: PHEAP_BLOCK;

    Next = core::ptr::addr_of_mut!((*Heap).Blocks).offset((*(*Block).Data.as_mut_ptr()).Flink as isize);
    Previous = core::ptr::addr_of_mut!((*Heap).Blocks).offset((*(*Block).Data.as_mut_ptr()).Blink as isize);
    
    (*(*Next).Data.as_mut_ptr()).Blink = Previous.offset_from(core::ptr::addr_of_mut!((*Heap).Blocks)) as u32;
    (*(*Previous).Data.as_mut_ptr()).Flink = Next.offset_from(core::ptr::addr_of_mut!((*Heap).Blocks)) as u32;
}

unsafe fn FrLdrHeapInsertFreeList(
    Heap: *mut HEAP,
    FreeBlock: *mut HEAP_BLOCK)
{
    let ListHead: *mut HEAP_BLOCK;
    let mut NextBlock: *mut HEAP_BLOCK;

    ListHead = core::ptr::addr_of_mut!((*Heap).Blocks).offset((*Heap).TerminatingBlock as isize);

    NextBlock = core::ptr::addr_of_mut!((*Heap).Blocks).offset((*(*ListHead).Data.as_mut_ptr()).Flink as isize);
    while NextBlock < FreeBlock {
        NextBlock = core::ptr::addr_of_mut!((*Heap).Blocks).offset((*(*NextBlock).Data.as_mut_ptr()).Flink as isize);
    }

    (*(*FreeBlock).Data.as_mut_ptr()).Flink = NextBlock.offset_from(core::ptr::addr_of_mut!((*Heap).Blocks)) as u32;
    (*(*FreeBlock).Data.as_mut_ptr()).Blink = (*(*NextBlock).Data.as_mut_ptr()).Blink;
    (*(*NextBlock).Data.as_mut_ptr()).Blink = FreeBlock.offset_from(core::ptr::addr_of_mut!((*Heap).Blocks)) as u32;
    NextBlock = core::ptr::addr_of_mut!((*Heap).Blocks).offset((*(*FreeBlock).Data.as_mut_ptr()).Blink as isize);
    (*(*NextBlock).Data.as_mut_ptr()).Flink = FreeBlock.offset_from(core::ptr::addr_of_mut!((*Heap).Blocks)) as u32;
}

#[no_mangle]
pub unsafe extern "C" fn FrLdrHeapAllocateEx(
    HeapHandle: PVOID,
    ByteSize: SIZE_T,
    Tag: ULONG) -> PVOID
{
    let Heap = HeapHandle as PHEAP;
    let mut Block: PHEAP_BLOCK;
    let mut NextBlock: PHEAP_BLOCK;
    let mut BlockSize: USHORT;
    let Remaining: USHORT;
    let ByteSize = ByteSize;
    let mut Tag = Tag;

    #[cfg(all(debug_assertions, not(any(target_arch = "arm", target_arch = "aarch64"))))]
    let Time = __rdtsc();

    #[cfg(feature = "FREELDR_HEAP_VERIFIER")]
    {
        FrLdrHeapVerify(HeapHandle);
        ByteSize += REDZONE_ALLOCATION;
    }

    if (ByteSize + core::mem::size_of::<HEAP_BLOCK>()) > MAXUSHORT as usize * core::mem::size_of::<HEAP_BLOCK>() {
        crate::frldr::dbgcon_str(b"HUGE!");
        ERR(b"HEAP: Allocation of 0x%lx bytes too large\n\0".as_ptr() as PCHAR, ByteSize);
        return core::ptr::null_mut();
    }
    if Heap.is_null() {
        crate::frldr::dbgcon_str(b"HEAPnull!");
        return core::ptr::null_mut();
    }

    if Tag == 0 {
        Tag = 0x654e6f4e; // 'enoN'
    }

    BlockSize = ((ByteSize + core::mem::size_of::<HEAP_BLOCK>() - 1) / core::mem::size_of::<HEAP_BLOCK>()) as USHORT;

    Block = core::ptr::addr_of_mut!((*Heap).Blocks).offset((*Heap).TerminatingBlock as isize);
    Block = core::ptr::addr_of_mut!((*Heap).Blocks).offset((*(*Block).Data.as_mut_ptr()).Flink as isize);
    while (*Block).Size != 0 {
        ntkrnl_sys::ASSERT!((*Block).Tag == 0);

        if (*Block).Size < BlockSize {
            Block = core::ptr::addr_of_mut!((*Heap).Blocks).offset((*(*Block).Data.as_mut_ptr()).Flink as isize);
            continue;
        }

        (*Block).Tag = Tag;

        FrLdrHeapRemoveFreeList(Heap, Block);

        Remaining = (*Block).Size - BlockSize;

        if Remaining > 1 {
            (*Block).Size = BlockSize;

            NextBlock = Block.offset(1 + BlockSize as isize);

            (*NextBlock).Tag = 0;
            (*NextBlock).Size = Remaining - 1;
            (*NextBlock).PreviousSize = BlockSize;
            BlockSize = (*NextBlock).Size;
            FrLdrHeapInsertFreeList(Heap, NextBlock);

            NextBlock = NextBlock.offset(1 + BlockSize as isize);
        } else {
            BlockSize = (*Block).Size;

            NextBlock = Block.offset(1 + BlockSize as isize);
        }

        (*NextBlock).PreviousSize = BlockSize;

        (*Heap).NumAllocs += 1;
        (*Heap).CurrentAllocBytes += (*Block).Size as SIZE_T * core::mem::size_of::<HEAP_BLOCK>();
        (*Heap).MaxAllocBytes = max((*Heap).MaxAllocBytes, (*Heap).CurrentAllocBytes);
        (*Heap).LargestAllocation = max((*Heap).LargestAllocation, (*Block).Size as SIZE_T * core::mem::size_of::<HEAP_BLOCK>());

        #[cfg(all(debug_assertions, not(any(target_arch = "arm", target_arch = "aarch64"))))]
        {
            (*Heap).AllocationTime += __rdtsc() - Time;
        }

        TRACE(b"HeapAllocate(%p, %ld, %.4s) -> return %p\n\0".as_ptr() as PCHAR,
              HeapHandle, ByteSize, core::ptr::addr_of!(Tag) as PVOID, (*Block).Data.as_mut_ptr() as PVOID);

        RtlZeroMemory((*Block).Data.as_mut_ptr() as PVOID, (*Block).Size as SIZE_T * core::mem::size_of::<HEAP_BLOCK>());

        #[cfg(feature = "FREELDR_HEAP_VERIFIER")]
        {
            *REDZONE_SIZE(Block) = ByteSize - REDZONE_ALLOCATION;
            *REDZONE_LOW(Block) = REDZONE_MARK;
            *REDZONE_HI(Block) = REDZONE_MARK;

            return ((*Block).Data.as_mut_ptr() as PUCHAR).offset(REDZONE_LOW_OFFSET as isize) as PVOID;
        }

        #[cfg(not(feature = "FREELDR_HEAP_VERIFIER"))]
        return (*Block).Data.as_mut_ptr() as PVOID;
    }

    crate::frldr::dbgcon_str(b"HEAPfull[bsz=");
    crate::frldr::dbgcon_hex32(BlockSize as u32);
    crate::frldr::dbgcon_str(b",tb=");
    crate::frldr::dbgcon_hex32((*Heap).TerminatingBlock as u32);
    // Walk free list.
    crate::frldr::dbgcon_str(b",fl=");
    let term_idx = (*Heap).TerminatingBlock as u32;
    let term = core::ptr::addr_of_mut!((*Heap).Blocks).offset(term_idx as isize);
    let mut walk_idx: u32 = (*(*term).Data.as_mut_ptr()).Flink;
    let mut depth = 0u32;
    while depth < 8 {
        let blk = core::ptr::addr_of_mut!((*Heap).Blocks).offset(walk_idx as isize);
        crate::frldr::dbgcon_hex32(walk_idx);
        crate::frldr::dbgcon(b':');
        crate::frldr::dbgcon_hex32((*blk).Size as u32);
        crate::frldr::dbgcon(b'/');
        crate::frldr::dbgcon_hex32((*blk).Tag);
        crate::frldr::dbgcon(b' ');
        if (*blk).Size == 0 { break; }
        walk_idx = (*(*blk).Data.as_mut_ptr()).Flink;
        depth += 1;
    }
    crate::frldr::dbgcon_str(b"]");
    WARN(b"HEAP: nothing suitable found for 0x%lx bytes\n\0".as_ptr() as PCHAR, ByteSize);
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn FrLdrHeapFreeEx(
    HeapHandle: PVOID,
    Pointer: PVOID,
    Tag: ULONG)
{
    let Heap = HeapHandle as PHEAP;
    let mut Block: PHEAP_BLOCK;
    let PrevBlock: PHEAP_BLOCK;
    let mut NextBlock: PHEAP_BLOCK;
    #[cfg(all(debug_assertions, not(any(target_arch = "arm", target_arch = "aarch64"))))]
    let Time = __rdtsc();

    TRACE(b"HeapFree(%p, %p)\n\0".as_ptr() as PCHAR, HeapHandle, Pointer);
    ntkrnl_sys::ASSERT!(Tag != 0x646E4523);

    #[cfg(feature = "FREELDR_HEAP_VERIFIER")]
    {
        FrLdrHeapVerify(HeapHandle);
    }

    if (Pointer < Heap.offset(1) as PVOID) ||
        (Pointer > (Heap as PUCHAR).offset((*Heap).MaximumSize as isize) as PVOID)
    {
        ERR(b"HEAP: trying to free %p outside of heap %p\n\0".as_ptr() as PCHAR, Pointer, Heap);
        ntkrnl_sys::ASSERT!(0u8 == 1u8);
    }

    Block = (Pointer as PHEAP_BLOCK).offset(-1);
    #[cfg(feature = "FREELDR_HEAP_VERIFIER")]
    {
        Block = (Block as PUCHAR).offset(-(REDZONE_LOW_OFFSET as isize)) as PHEAP_BLOCK;

        ntkrnl_sys::ASSERT!(*REDZONE_SIZE(Block) <= (*Block).Size as u64 * core::mem::size_of::<HEAP_BLOCK>() as u64);
        ntkrnl_sys::ASSERT!(*REDZONE_LOW(Block) == REDZONE_MARK);
        ntkrnl_sys::ASSERT!(*REDZONE_HI(Block) == REDZONE_MARK);
    }

    if (Tag != 0 && (*Block).Tag != Tag) || ((*Block).Tag == 0)
    {
        ERR(b"HEAP: Bad tag! Pointer=%p: block tag '%.4s', requested '%.4s', size=0x%lx\n\0".as_ptr() as PCHAR,
            Pointer, core::ptr::addr_of!((*Block).Tag), core::ptr::addr_of!(Tag), (*Block).Size as u32);
        ntkrnl_sys::ASSERT!(0u8 == 1u8);
    }

    (*Block).Tag = 0;

    #[cfg(debug_assertions)]
    {
        core::ptr::write_bytes((*Block).Data.as_mut_ptr() as *mut u8, 0xCC, ((*Block).Size as usize) * core::mem::size_of::<HEAP_BLOCK>());
    }

    (*Heap).NumFrees = (*Heap).NumFrees.wrapping_add(1);
    (*Heap).CurrentAllocBytes = (*Heap).CurrentAllocBytes.wrapping_sub((*Block).Size as usize * core::mem::size_of::<HEAP_BLOCK>());

    PrevBlock = Block.offset(-((*Block).PreviousSize as isize) - 1);
    NextBlock = Block.offset((*Block).Size as isize + 1);

    // Use u32 for the overflow guard — Rust u16+u16+1 wraps silently in
    // release mode. The C version is safe because integer promotion does
    // the add in `int`.
    if ((*NextBlock).Tag == 0) &&
        (((*Block).Size as u32 + (*NextBlock).Size as u32 + 1) <= 0xFFFF)
    {
        (*Block).Size = (*Block).Size.wrapping_add((*NextBlock).Size).wrapping_add(1);
        FrLdrHeapRemoveFreeList(Heap, NextBlock);

        NextBlock = Block.offset((*Block).Size as isize + 1);
    }

    if ((*Block).PreviousSize != 0) && ((*PrevBlock).Tag == 0) &&
        (((*PrevBlock).Size as u32 + (*Block).Size as u32 + 1) <= 0xFFFF)
    {
        (*PrevBlock).Size = (*PrevBlock).Size.wrapping_add((*Block).Size).wrapping_add(1);
        Block = PrevBlock;
    }
    else
    {
        FrLdrHeapInsertFreeList(Heap, Block);
    }

    (*NextBlock).PreviousSize = (*Block).Size;
    #[cfg(all(debug_assertions, not(any(target_arch = "arm", target_arch = "aarch64"))))]
    {
        (*Heap).FreeTime = (*Heap).FreeTime.wrapping_add(__rdtsc().wrapping_sub(Time));
    }
}

// read_volatile on the static-mut load. Without it, LLVM (cgu=256 + lto=false)
// sees the file-local initializer `static mut FrLdrDefaultHeap: PVOID = null()`
// and caches NULL as the value. MmInitializeHeap's runtime assignment lives in
// (potentially) another CGU and isn't visible. The cached NULL feeds into
// FrLdrHeapAllocateEx → InsertFreeList where `(*Heap).Blocks` becomes `NULL + 0x2c`
// and the freelist walk reads garbage from low memory (BIOS IVT) and loops
// forever. Same class as the GetHarddiskConfigurationData fix in pchw.rs.
// See feedback_dup_abi_name_collision.md (cross-CGU static-mut variant).
#[no_mangle]
pub unsafe extern "C" fn FrLdrHeapAlloc(MemorySize: SIZE_T, Tag: ULONG) -> PVOID {
    let heap = core::ptr::read_volatile(core::ptr::addr_of!(FrLdrDefaultHeap));
    FrLdrHeapAllocateEx(heap, MemorySize, Tag)
}

#[no_mangle]
pub unsafe extern "C" fn FrLdrHeapFree(MemoryPointer: PVOID, Tag: ULONG)
{
    let heap = core::ptr::read_volatile(core::ptr::addr_of!(FrLdrDefaultHeap));
    FrLdrHeapFreeEx(heap, MemoryPointer, Tag);
}

#[no_mangle]
pub unsafe extern "C" fn FrLdrTempAlloc(
    Size: SIZE_T,
    Tag: ULONG) -> PVOID
{
    let heap = core::ptr::read_volatile(core::ptr::addr_of!(FrLdrTempHeap));
    FrLdrHeapAllocateEx(heap, Size, Tag)
}

#[no_mangle]
pub unsafe extern "C" fn FrLdrTempFree(
    Allocation: PVOID,
    Tag: ULONG)
{
    let heap = core::ptr::read_volatile(core::ptr::addr_of!(FrLdrTempHeap));
    FrLdrHeapFreeEx(heap, Allocation, Tag);
}

/// Diagnostic: emit a single byte to QEMU debugcon (port 0xE9).
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[no_mangle]
pub unsafe extern "C" fn FrLdrDbgcon(b: u8) {
    core::arch::asm!("out dx, al", in("dx") 0xe9u16, in("al") b,
        options(nostack, preserves_flags));
}

/// Diagnostic: emit 8 hex digits + space to debugcon.
#[no_mangle]
pub unsafe extern "C" fn FrLdrDbgconHex32(v: u32) {
    crate::frldr::dbgcon_hex32(v);
}

/// Diagnostic: dump default-heap free-list state to debugcon.
/// Tag byte identifies the call site. All reads via `read_volatile` to
/// defeat any LLVM aliasing assumption about the heap memory being immutable.
/// Output: `<H label idx:size/tag idx:size/tag ...>`
#[no_mangle]
pub unsafe extern "C" fn FrLdrHeapDump(label: u8) {
    let h = core::ptr::read_volatile(core::ptr::addr_of!(FrLdrDefaultHeap)) as PHEAP;
    if h.is_null() { return; }
    crate::frldr::dbgcon_str(b"<H");
    crate::frldr::dbgcon(label);
    crate::frldr::dbgcon(b' ');
    let term_idx = core::ptr::read_volatile(core::ptr::addr_of!((*h).TerminatingBlock)) as u32;
    let blocks_base = core::ptr::addr_of_mut!((*h).Blocks);
    let term = blocks_base.offset(term_idx as isize);
    let mut walk: u32 = core::ptr::read_volatile(
        core::ptr::addr_of!((*(*term).Data.as_ptr()).Flink));
    let mut depth = 0u32;
    while depth < 12 {
        let max_idx = (core::ptr::read_volatile(core::ptr::addr_of!((*h).MaximumSize))
            / core::mem::size_of::<HEAP_BLOCK>()) as u32;
        if walk > max_idx {
            crate::frldr::dbgcon_str(b"OOB:");
            crate::frldr::dbgcon_hex32(walk);
            break;
        }
        let blk = blocks_base.offset(walk as isize);
        let sz = core::ptr::read_volatile(core::ptr::addr_of!((*blk).Size));
        let tg = core::ptr::read_volatile(core::ptr::addr_of!((*blk).Tag));
        crate::frldr::dbgcon_hex32(walk);
        crate::frldr::dbgcon(b':');
        crate::frldr::dbgcon_hex32(sz as u32);
        crate::frldr::dbgcon(b'/');
        crate::frldr::dbgcon_hex32(tg);
        crate::frldr::dbgcon(b' ');
        if sz == 0 { break; }
        walk = core::ptr::read_volatile(
            core::ptr::addr_of!((*(*blk).Data.as_ptr()).Flink));
        depth += 1;
    }
    crate::frldr::dbgcon_str(b">");
}

#[no_mangle]
pub unsafe extern "C" fn MmInitializeHeap(_PageLookupTable: PVOID) {
    TRACE(b"MmInitializeHeap()\n\0".as_ptr() as PCHAR);

    FrLdrDefaultHeap = FrLdrHeapCreate(DEFAULT_HEAP_SIZE, LoaderOsloaderHeap);
    ASSERT(FrLdrDefaultHeap as usize != 0);

    FrLdrTempHeap = FrLdrHeapCreate(TEMP_HEAP_SIZE, LoaderFirmwareTemporary);
    ASSERT(FrLdrTempHeap as usize != 0);

    // Dump default heap free list right after init.
    crate::frldr::dbgcon_str(b"\nHeapInit[def=");
    crate::frldr::dbgcon_hex32(FrLdrDefaultHeap as usize as u32);
    crate::frldr::dbgcon_str(b",tmp=");
    crate::frldr::dbgcon_hex32(FrLdrTempHeap as usize as u32);
    let dh = FrLdrDefaultHeap as PHEAP;
    crate::frldr::dbgcon_str(b",tb=");
    crate::frldr::dbgcon_hex32((*dh).TerminatingBlock as u32);
    let term = core::ptr::addr_of_mut!((*dh).Blocks).offset((*dh).TerminatingBlock as isize);
    let mut walk: u32 = (*(*term).Data.as_mut_ptr()).Flink;
    let mut depth = 0u32;
    crate::frldr::dbgcon_str(b",fl=");
    while depth < 8 {
        let blk = core::ptr::addr_of_mut!((*dh).Blocks).offset(walk as isize);
        crate::frldr::dbgcon_hex32(walk);
        crate::frldr::dbgcon(b':');
        crate::frldr::dbgcon_hex32((*blk).Size as u32);
        crate::frldr::dbgcon(b' ');
        if (*blk).Size == 0 { break; }
        walk = (*(*blk).Data.as_mut_ptr()).Flink;
        depth += 1;
    }
    crate::frldr::dbgcon_str(b"]\n");

    TRACE(
        b"MmInitializeHeap() done, default heap %p, temp heap %p\n\0".as_ptr() as PCHAR,
        FrLdrDefaultHeap,
        FrLdrTempHeap,
    );
}

#[no_mangle]
pub unsafe extern "system" fn ExAllocatePoolWithTag(
    _PoolType: POOL_TYPE,
    NumberOfBytes: SIZE_T,
    Tag: ULONG) -> PVOID
{
    FrLdrHeapAllocateEx(FrLdrDefaultHeap, NumberOfBytes, Tag)
}

#[no_mangle]
pub unsafe extern "system" fn ExAllocatePool(
    _PoolType: POOL_TYPE,
    NumberOfBytes: SIZE_T) -> PVOID
{
    return FrLdrHeapAllocateEx(FrLdrDefaultHeap, NumberOfBytes, 0);
}

#[no_mangle]
pub unsafe extern "system" fn ExFreePool(
    P: PVOID)
{
    FrLdrHeapFreeEx(FrLdrDefaultHeap, P, 0);
}

#[no_mangle]
pub unsafe extern "system" fn ExFreePoolWithTag(
    P: PVOID,
    Tag: ULONG)
{
    FrLdrHeapFreeEx(FrLdrDefaultHeap, P, Tag);
}

// heap.c:623 and :639 both pass ' ltR' -- ONE tag for the pair. Translation
// emitted two different literals: the free got 0x206C7452 (correct) and the
// alloc got 0x20746C52, 't' and 'l' transposed. Every RtlFreeHeap in freeldr
// therefore hit the tag-mismatch branch. Named once so the pair cannot drift
// again: a constant declared once cannot disagree with itself.
const TAG_RTL_HEAP: ULONG = u32::from_be_bytes(*b" ltR");

#[no_mangle]
pub unsafe extern "system" fn RtlAllocateHeap(
    _HeapHandle: PVOID,
    Flags: ULONG,
    Size: SIZE_T) -> PVOID
{
    let ptr = FrLdrHeapAllocateEx(FrLdrDefaultHeap, Size, TAG_RTL_HEAP);
    if !ptr.is_null() && (Flags & HEAP_ZERO_MEMORY) != 0
    {
        RtlZeroMemory(ptr, Size);
    }

    ptr
}

#[no_mangle]
pub unsafe extern "system" fn RtlFreeHeap(
    _HeapHandle: PVOID,
    _Flags: ULONG,
    HeapBase: PVOID) -> BOOLEAN
{
    FrLdrHeapFreeEx(FrLdrDefaultHeap, HeapBase, TAG_RTL_HEAP);
    1u8
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  FreeLoader
//  *  Copyright (C) 2011 Timo Kreuzer (timo.kreuzer@reactos.org)
//  *
//  *  This program is free software; you can redistribute it and/or modify
//  *  it under the terms of the GNU General Public License as published by
//  *  the Free Software Foundation; either version 2 of the License, or
//  *  (at your option) any later version.
//  *
//  *  This program is distributed in the hope that it will be useful,
//  *  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  *  GNU General Public License for more details.
//  *
//  *  You should have received a copy of the GNU General Public License along
//  *  with this program; if not, write to the Free Software Foundation, Inc.,
//  *  51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
//  */
//
// #include <freeldr.h>
//
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(HEAP);
//
// #define FREELDR_HEAP_VERIFIER
//
// #define REDZONE_MARK 0xCCCCCCCCCCCCCCCCULL
// #define REDZONE_ALLOCATION 24
// #define REDZONE_LOW_OFFSET 16
// #define REDZONE_SIZE(Block) ((ULONG64*)Block->Data)
// #define REDZONE_LOW(Block) ((ULONG64*)Block->Data + 1)
// #define REDZONE_HI(Block) ((ULONG64*)((PUCHAR)Block->Data + 16 + *REDZONE_SIZE(Block)))
//
// static PVOID FrLdrDefaultHeap;
// static PVOID FrLdrTempHeap;
//
// typedef struct _BLOCK_DATA
// {
//     ULONG_PTR Flink:32;
//     ULONG_PTR Blink:32;
// } BLOCK_DATA, *PBLOCK_DATA;
//
// typedef struct _HEAP_BLOCK
// {
//     USHORT Size;
//     USHORT PreviousSize;
//     ULONG Tag;
//     BLOCK_DATA Data[];
// } HEAP_BLOCK, *PHEAP_BLOCK;
//
// typedef struct _HEAP
// {
//     SIZE_T MaximumSize;
//     SIZE_T CurrentAllocBytes;
//     SIZE_T MaxAllocBytes;
//     ULONG NumAllocs;
//     ULONG NumFrees;
//     SIZE_T LargestAllocation;
//     ULONGLONG AllocationTime;
//     ULONGLONG FreeTime;
//     ULONG_PTR TerminatingBlock;
//     HEAP_BLOCK Blocks;
// } HEAP, *PHEAP;
//
// PVOID
// FrLdrHeapCreate(
//     SIZE_T MaximumSize,
//     TYPE_OF_MEMORY MemoryType)
// {
//     PHEAP Heap;
//     PHEAP_BLOCK Block;
//     SIZE_T Remaining;
//     USHORT PreviousSize;
//
//     TRACE("HeapCreate(MemoryType=%ld)\n", MemoryType);
//
//     /* Allocate some memory for the heap */
//     MaximumSize = ALIGN_UP_BY(MaximumSize, MM_PAGE_SIZE);
//     Heap = MmAllocateMemoryWithType(MaximumSize, MemoryType);
//     if (!Heap)
//     {
//         ERR("HEAP: Failed to allocate heap of size 0x%lx, Type %lu\n",
//             MaximumSize, MemoryType);
//         return NULL;
//     }
//
//     /* Initialize the heap header */
//     Heap->MaximumSize = MaximumSize;
//     Heap->CurrentAllocBytes = 0;
//     Heap->MaxAllocBytes = 0;
//     Heap->NumAllocs = 0;
//     Heap->NumFrees = 0;
//     Heap->LargestAllocation = 0;
//
//     /* Calculate what's left to process */
//     Remaining = (MaximumSize - sizeof(HEAP)) / sizeof(HEAP_BLOCK);
//     TRACE("Remaining = %ld\n", Remaining);
//
//     /* Substract 2 for the terminating entry (header + free entry) */
//     Remaining -= 2;
//
//     Block = &Heap->Blocks;
//     PreviousSize = 0;
//
//     /* Create free blocks */
//     while (Remaining > 1)
//     {
//         /* Initialize this free block */
//         Block->Size = (USHORT)min(MAXUSHORT, Remaining - 1);
//         Block->PreviousSize = PreviousSize;
//         Block->Tag = 0;
//         Block->Data[0].Flink = (Block - &Heap->Blocks) + Block->Size + 1;
//         Block->Data[0].Blink = (Block - &Heap->Blocks) - 1 - PreviousSize;
//
//         /* Substract current block size from remainder */
//         Remaining -= (Block->Size + 1);
//
//         /* Go to next block */
//         PreviousSize = Block->Size;
//         Block = Block + Block->Size + 1;
//
//         TRACE("Remaining = %ld\n", Remaining);
//     }
//
//     /* Now finish with a terminating block */
//     Heap->TerminatingBlock = Block - &Heap->Blocks;
//     Block->Size = 0;
//     Block->PreviousSize = PreviousSize;
//     Block->Tag = 'dnE#';
//     Block->Data[0].Flink = 0;
//     Block->Data[0].Blink = (Block - &Heap->Blocks) - 1 - PreviousSize;
//     Heap->Blocks.Data[0].Blink = Heap->TerminatingBlock;
//
//     return Heap;
// }
//
// VOID
// FrLdrHeapDestroy(
//     PVOID HeapHandle)
// {
//     PHEAP Heap = HeapHandle;
//
//     /* Mark all pages as firmware temporary, so they are free for the kernel */
//     MmMarkPagesInLookupTable(PageLookupTableAddress,
//                              (ULONG_PTR)Heap / MM_PAGE_SIZE,
//                              (PFN_COUNT)(Heap->MaximumSize / MM_PAGE_SIZE),
//                              LoaderFirmwareTemporary);
//
// #if DBG
//     /* Make sure everything is dead */
//     RtlFillMemory(Heap, Heap->MaximumSize, 0xCCCCCCCC);
// #endif
// }
//
// #ifdef FREELDR_HEAP_VERIFIER
// VOID
// FrLdrHeapVerify(
//     PVOID HeapHandle)
// {
//     PHEAP Heap = HeapHandle;
//     PHEAP_BLOCK Block;
//
//     /* Loop all heap chunks */
//     for (Block = &Heap->Blocks;
//          Block->Size != 0;
//          Block = Block + 1 + Block->Size)
//     {
//         /* Continue, if its not free */
//         if (Block->Tag != 0)
//         {
//             /* Verify size and redzones */
//             ASSERT(*REDZONE_SIZE(Block) <= Block->Size * sizeof(HEAP_BLOCK));
//             ASSERT(*REDZONE_LOW(Block) == REDZONE_MARK);
//             ASSERT(*REDZONE_HI(Block) == REDZONE_MARK);
//             continue;
//         }
//     }
// }
// #endif /* FREELDR_HEAP_VERIFIER */
//
// VOID
// FrLdrHeapRelease(
//     PVOID HeapHandle)
// {
//     PHEAP Heap = HeapHandle;
//     PHEAP_BLOCK Block;
//     PUCHAR StartAddress, EndAddress;
//     PFN_COUNT FreePages, AllFreePages = 0;
//
//     TRACE("HeapRelease(%p)\n", HeapHandle);
//
//     /* Loop all heap chunks */
//     for (Block = &Heap->Blocks;
//          Block->Size != 0;
//          Block = Block + 1 + Block->Size)
//     {
//         /* Continue, if its not free */
//         if (Block->Tag != 0)
//         {
// #ifdef FREELDR_HEAP_VERIFIER
//             /* Verify size and redzones */
//             ASSERT(*REDZONE_SIZE(Block) <= Block->Size * sizeof(HEAP_BLOCK));
//             ASSERT(*REDZONE_LOW(Block) == REDZONE_MARK);
//             ASSERT(*REDZONE_HI(Block) == REDZONE_MARK);
// #endif
//             continue;
//         }
//
//         /* Calculate page aligned start address of the free region */
//         StartAddress = ALIGN_UP_POINTER_BY(Block->Data, PAGE_SIZE);
//
//         /* Walk over adjacent free blocks */
//         while (Block->Tag == 0) Block = Block + Block->Size + 1;
//
//         /* Check if this was the last block */
//         if (Block->Size == 0)
//         {
//             /* Align the end address up to cover the end of the heap */
//             EndAddress = ALIGN_UP_POINTER_BY(Block->Data, PAGE_SIZE);
//         }
//         else
//         {
//             /* Align the end address down to not cover any allocations */
//             EndAddress = ALIGN_DOWN_POINTER_BY(Block->Data, PAGE_SIZE);
//         }
//
//         /* Check if we have free pages */
//         if (EndAddress > StartAddress)
//         {
//             /* Calculate the size of the free region in pages */
//             FreePages = (PFN_COUNT)((EndAddress - StartAddress) / MM_PAGE_SIZE);
//             AllFreePages += FreePages;
//
//             /* Now mark the pages free */
//             MmMarkPagesInLookupTable(PageLookupTableAddress,
//                                      (ULONG_PTR)StartAddress / MM_PAGE_SIZE,
//                                      FreePages,
//                                      LoaderFree);
//         }
//
//         /* bail out, if it was the last block */
//         if (Block->Size == 0) break;
//     }
//
//     TRACE("HeapRelease() done, freed %lu of %lu pages\n", AllFreePages, Heap->MaximumSize / MM_PAGE_SIZE);
// }
//
// VOID
// FrLdrHeapCleanupAll(VOID)
// {
// #if DBG
//     PHEAP Heap;
//
//     Heap = FrLdrDefaultHeap;
//     TRACE("Heap statistics for default heap:\n"
//           "CurrentAlloc=0x%lx, MaxAlloc=0x%lx, LargestAllocation=0x%lx\n"
//           "NumAllocs=%ld, NumFrees=%ld\n",
//           Heap->CurrentAllocBytes, Heap->MaxAllocBytes, Heap->LargestAllocation,
//           Heap->NumAllocs, Heap->NumFrees);
//     TRACE("AllocTime = %I64d, FreeTime = %I64d, sum = %I64d\n",
//         Heap->AllocationTime, Heap->FreeTime, Heap->AllocationTime + Heap->FreeTime);
// #endif
//
//     /* Release free pages from the default heap */
//     FrLdrHeapRelease(FrLdrDefaultHeap);
//
// #if DBG
//     Heap = FrLdrTempHeap;
//     TRACE("Heap statistics for temp heap:\n"
//           "CurrentAlloc=0x%lx, MaxAlloc=0x%lx, LargestAllocation=0x%lx\n"
//           "NumAllocs=%ld, NumFrees=%ld\n",
//           Heap->CurrentAllocBytes, Heap->MaxAllocBytes, Heap->LargestAllocation,
//           Heap->NumAllocs, Heap->NumFrees);
// #endif
//
//     /* Destroy the temp heap */
//     FrLdrHeapDestroy(FrLdrTempHeap);
// }
//
// static VOID
// FrLdrHeapRemoveFreeList(
//     PHEAP Heap,
//     PHEAP_BLOCK Block)
// {
//     PHEAP_BLOCK Previous, Next;
//
//     Next = &Heap->Blocks + Block->Data[0].Flink;
//     Previous = &Heap->Blocks + Block->Data[0].Blink;
//     ASSERT((Next->Tag == 0) || (Next->Tag == 'dnE#'));
//     ASSERT(Next->Data[0].Blink == Block - &Heap->Blocks);
//     ASSERT((Previous->Tag == 0) || (Previous->Tag == 'dnE#'));
//     ASSERT(Previous->Data[0].Flink == Block - &Heap->Blocks);
//
//     Next->Data[0].Blink = Previous - &Heap->Blocks;
//     Previous->Data[0].Flink = Next - &Heap->Blocks;
// }
//
// static VOID
// FrLdrHeapInsertFreeList(
//     PHEAP Heap,
//     PHEAP_BLOCK FreeBlock)
// {
//     PHEAP_BLOCK ListHead, NextBlock;
//     ASSERT(FreeBlock->Tag == 0);
//
//     /* Terminating block serves as free list head */
//     ListHead = &Heap->Blocks + Heap->TerminatingBlock;
//
//     for (NextBlock = &Heap->Blocks + ListHead->Data[0].Flink;
//          NextBlock < FreeBlock;
//          NextBlock = &Heap->Blocks + NextBlock->Data[0].Flink);
//
//     FreeBlock->Data[0].Flink = NextBlock - &Heap->Blocks;
//     FreeBlock->Data[0].Blink = NextBlock->Data[0].Blink;
//     NextBlock->Data[0].Blink = FreeBlock - &Heap->Blocks;
//     NextBlock = &Heap->Blocks + FreeBlock->Data[0].Blink;
//     NextBlock->Data[0].Flink = FreeBlock - &Heap->Blocks;
// }
//
// PVOID
// FrLdrHeapAllocateEx(
//     PVOID HeapHandle,
//     SIZE_T ByteSize,
//     ULONG Tag)
// {
//     PHEAP Heap = HeapHandle;
//     PHEAP_BLOCK Block, NextBlock;
//     USHORT BlockSize, Remaining;
// #if DBG && !defined(_M_ARM) && !defined(_M_ARM64)
//     ULONGLONG Time = __rdtsc();
// #endif
//
// #ifdef FREELDR_HEAP_VERIFIER
//     /* Verify the heap */
//     FrLdrHeapVerify(HeapHandle);
//
//     /* Add space for a size field and 2 redzones */
//     ByteSize += REDZONE_ALLOCATION;
// #endif
//
//     /* Check if the allocation is too large */
//     if ((ByteSize +  sizeof(HEAP_BLOCK)) > MAXUSHORT * sizeof(HEAP_BLOCK))
//     {
//         ERR("HEAP: Allocation of 0x%lx bytes too large\n", ByteSize);
//         return NULL;
//     }
//
//     /* We need a proper tag */
//     if (Tag == 0) Tag = 'enoN';
//
//     /* Calculate alloc size */
//     BlockSize = (USHORT)((ByteSize + sizeof(HEAP_BLOCK) - 1) / sizeof(HEAP_BLOCK));
//
//     /* Walk the free block list */
//     Block = &Heap->Blocks + Heap->TerminatingBlock;
//     for (Block = &Heap->Blocks + Block->Data[0].Flink;
//          Block->Size != 0;
//          Block = &Heap->Blocks + Block->Data[0].Flink)
//     {
//         ASSERT(Block->Tag == 0);
//
//         /* Continue, if its too small */
//         if (Block->Size < BlockSize) continue;
//
//         /* This block is just fine, use it */
//         Block->Tag = Tag;
//
//         /* Remove this entry from the free list */
//         FrLdrHeapRemoveFreeList(Heap, Block);
//
//         /* Calculate the remaining size */
//         Remaining = Block->Size - BlockSize;
//
//         /* Check if the remaining space is large enough for a new block */
//         if (Remaining > 1)
//         {
//             /* Make the allocated block as large as necessary */
//             Block->Size = BlockSize;
//
//             /* Get pointer to the new block */
//             NextBlock = Block + 1 + BlockSize;
//
//             /* Make it a free block */
//             NextBlock->Tag = 0;
//             NextBlock->Size = Remaining - 1;
//             NextBlock->PreviousSize = BlockSize;
//             BlockSize = NextBlock->Size;
//             FrLdrHeapInsertFreeList(Heap, NextBlock);
//
//             /* Advance to the next block */
//             NextBlock = NextBlock + 1 + BlockSize;
//         }
//         else
//         {
//             /* Not enough left, use the full block */
//             BlockSize = Block->Size;
//
//             /* Get the next block */
//             NextBlock = Block + 1 + BlockSize;
//         }
//
//         /* Update the next blocks back link */
//         NextBlock->PreviousSize = BlockSize;
//
//         /* Update heap usage */
//         Heap->NumAllocs++;
//         Heap->CurrentAllocBytes += Block->Size * sizeof(HEAP_BLOCK);
//         Heap->MaxAllocBytes = max(Heap->MaxAllocBytes, Heap->CurrentAllocBytes);
//         Heap->LargestAllocation = max(Heap->LargestAllocation,
//                                       Block->Size * sizeof(HEAP_BLOCK));
// #if DBG && !defined(_M_ARM) && !defined(_M_ARM64)
//         Heap->AllocationTime += (__rdtsc() - Time);
// #endif
//         TRACE("HeapAllocate(%p, %ld, %.4s) -> return %p\n",
//               HeapHandle, ByteSize, &Tag, Block->Data);
//
//         /* HACK: zero out the allocation */
//         RtlZeroMemory(Block->Data, Block->Size * sizeof(HEAP_BLOCK));
//
// #ifdef FREELDR_HEAP_VERIFIER
//         /* Write size and redzones */
//         *REDZONE_SIZE(Block) = ByteSize - REDZONE_ALLOCATION;
//         *REDZONE_LOW(Block) = REDZONE_MARK;
//         *REDZONE_HI(Block) = REDZONE_MARK;
//
//         /* Allocation starts after size field and redzone */
//         return (PUCHAR)Block->Data + REDZONE_LOW_OFFSET;
// #endif
//         /* Return pointer to the data */
//         return Block->Data;
//     }
//
//     /* We found nothing */
//     WARN("HEAP: nothing suitable found for 0x%lx bytes\n", ByteSize);
//     return NULL;
// }
//
// VOID
// FrLdrHeapFreeEx(
//     PVOID HeapHandle,
//     PVOID Pointer,
//     ULONG Tag)
// {
//     PHEAP Heap = HeapHandle;
//     PHEAP_BLOCK Block, PrevBlock, NextBlock;
// #if DBG && !defined(_M_ARM) && !defined(_M_ARM64)
//     ULONGLONG Time = __rdtsc();
// #endif
//
//     TRACE("HeapFree(%p, %p)\n", HeapHandle, Pointer);
//     ASSERT(Tag != 'dnE#');
//
// #ifdef FREELDR_HEAP_VERIFIER
//     /* Verify the heap */
//     FrLdrHeapVerify(HeapHandle);
// #endif
//
//     /* Check if the block is really inside this heap */
//     if ((Pointer < (PVOID)(Heap + 1)) ||
//         (Pointer > (PVOID)((PUCHAR)Heap + Heap->MaximumSize)))
//     {
//         ERR("HEAP: trying to free %p outside of heap %p\n", Pointer, Heap);
//         ASSERT(FALSE);
//     }
//
//     Block = ((PHEAP_BLOCK)Pointer) - 1;
// #ifdef FREELDR_HEAP_VERIFIER
//     Block = (PHEAP_BLOCK)((PUCHAR)Block - REDZONE_LOW_OFFSET);
//
//     /* Verify size and redzones */
//     ASSERT(*REDZONE_SIZE(Block) <= Block->Size * sizeof(HEAP_BLOCK));
//     ASSERT(*REDZONE_LOW(Block) == REDZONE_MARK);
//     ASSERT(*REDZONE_HI(Block) == REDZONE_MARK);
// #endif
//
//     /* Check if the tag matches */
//     if ((Tag && (Block->Tag != Tag)) || (Block->Tag == 0))
//     {
//         ERR("HEAP: Bad tag! Pointer=%p: block tag '%.4s', requested '%.4s', size=0x%lx\n",
//             Pointer, &Block->Tag, &Tag, Block->Size);
//         ASSERT(FALSE);
//     }
//
//     /* Mark as free */
//     Block->Tag = 0;
//
// #if DBG
//     /* Erase contents */
//     RtlFillMemory(Block->Data, Block->Size * sizeof(HEAP_BLOCK), 0xCCCCCCCC);
// #endif
//
//     /* Update heap usage */
//     Heap->NumFrees++;
//     Heap->CurrentAllocBytes -= Block->Size * sizeof(HEAP_BLOCK);
//
//     /* Get pointers to the next and previous block */
//     PrevBlock = Block - Block->PreviousSize - 1;
//     NextBlock = Block + Block->Size + 1;
//
//     /* Check if next block is free */
//     if ((NextBlock->Tag == 0) &&
//         ((Block->Size + NextBlock->Size + 1) <= MAXUSHORT))
//     {
//         /* Merge next block into current */
//         Block->Size += NextBlock->Size + 1;
//         FrLdrHeapRemoveFreeList(Heap, NextBlock);
//
//         NextBlock = Block + Block->Size + 1;
//     }
//
//     /* Check if there is a block before and it's free */
//     if ((Block->PreviousSize != 0) && (PrevBlock->Tag == 0) &&
//         ((PrevBlock->Size + Block->Size + 1) <= MAXUSHORT))
//     {
//         /* Merge current block into previous */
//         PrevBlock->Size += Block->Size + 1;
//         Block = PrevBlock;
//     }
//     else
//     {
//         /* Insert the entry into the free list */
//         FrLdrHeapInsertFreeList(Heap, Block);
//     }
//
//     /* Update the next block's back link */
//     NextBlock->PreviousSize = Block->Size;
// #if DBG && !defined(_M_ARM) && !defined(_M_ARM64)
//     Heap->FreeTime += (__rdtsc() - Time);
// #endif
// }
//
// PVOID
// FrLdrHeapAlloc(SIZE_T MemorySize, ULONG Tag)
// {
//     return FrLdrHeapAllocateEx(FrLdrDefaultHeap, MemorySize, Tag);
// }
//
// VOID
// FrLdrHeapFree(PVOID MemoryPointer, ULONG Tag)
// {
//     FrLdrHeapFreeEx(FrLdrDefaultHeap, MemoryPointer, Tag);
// }
//
// PVOID
// FrLdrTempAlloc(
//     _In_ SIZE_T Size,
//     _In_ ULONG Tag)
// {
//     return FrLdrHeapAllocateEx(FrLdrTempHeap, Size, Tag);
// }
//
// VOID
// FrLdrTempFree(
//     PVOID Allocation, ULONG Tag)
// {
//     FrLdrHeapFreeEx(FrLdrTempHeap, Allocation, Tag);
// }
//
// /* Wrapper functions *********************************************************/
//
// VOID
// MmInitializeHeap(PVOID PageLookupTable)
// {
//     TRACE("MmInitializeHeap()\n");
//
//     /* Create the default heap */
//     FrLdrDefaultHeap = FrLdrHeapCreate(DEFAULT_HEAP_SIZE, LoaderOsloaderHeap);
//     ASSERT(FrLdrDefaultHeap);
//
//     /* Create a temporary heap */
//     FrLdrTempHeap = FrLdrHeapCreate(TEMP_HEAP_SIZE, LoaderFirmwareTemporary);
//     ASSERT(FrLdrTempHeap);
//
//     TRACE("MmInitializeHeap() done, default heap %p, temp heap %p\n",
//           FrLdrDefaultHeap, FrLdrTempHeap);
// }
//
// PVOID
// NTAPI
// ExAllocatePoolWithTag(
//     IN POOL_TYPE PoolType,
//     IN SIZE_T NumberOfBytes,
//     IN ULONG Tag)
// {
//     return FrLdrHeapAllocateEx(FrLdrDefaultHeap, NumberOfBytes, Tag);
// }
//
// PVOID
// NTAPI
// ExAllocatePool(
//     IN POOL_TYPE PoolType,
//     IN SIZE_T NumberOfBytes)
// {
//     return FrLdrHeapAllocateEx(FrLdrDefaultHeap, NumberOfBytes, 0);
// }
//
// VOID
// NTAPI
// ExFreePool(
//     IN PVOID P)
// {
//     FrLdrHeapFreeEx(FrLdrDefaultHeap, P, 0);
// }
//
// VOID
// NTAPI
// ExFreePoolWithTag(
//   IN PVOID P,
//   IN ULONG Tag)
// {
//     FrLdrHeapFreeEx(FrLdrDefaultHeap, P, Tag);
// }
//
// PVOID
// NTAPI
// RtlAllocateHeap(
//     IN PVOID HeapHandle,
//     IN ULONG Flags,
//     IN SIZE_T Size)
// {
//     PVOID ptr;
//
//     ptr = FrLdrHeapAllocateEx(FrLdrDefaultHeap, Size, ' ltR');
//     if (ptr && (Flags & HEAP_ZERO_MEMORY))
//     {
//         RtlZeroMemory(ptr, Size);
//     }
//
//     return ptr;
// }
//
// BOOLEAN
// NTAPI
// RtlFreeHeap(
//     IN PVOID HeapHandle,
//     IN ULONG Flags,
//     IN PVOID HeapBase)
// {
//     FrLdrHeapFreeEx(FrLdrDefaultHeap, HeapBase, ' ltR');
//     return TRUE;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
