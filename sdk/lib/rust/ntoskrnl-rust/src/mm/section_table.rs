//! Translated from `ntoskrnl/cache/section/sptab.c`.
//!
//! Section-segment page tables, backed by `RtlGenericTable` 256-page chunks.
//! Each entry encodes `bit 0`=swap-entry, `bit 1`=dirty, `bits 3..11`=mapcount,
//! upper bits=PFN.  The page-table operations also maintain the segment rmap
//! (see `mm/rmap.rs`) so a paged-out PFN can be located back to its segment.
//!
//! Calls to `_MmSet/GetPageEntrySectionSegment` must be serialised by holding
//! the segment lock — assertion in the original C is preserved.

use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{
    PMM_SECTION_SEGMENT, PLARGE_INTEGER, LARGE_INTEGER, _LARGE_INTEGER,
    PRTL_GENERIC_TABLE, RTL_GENERIC_TABLE, _RTL_GENERIC_TABLE,
    RTL_GENERIC_COMPARE_RESULTS,
    _RTL_GENERIC_COMPARE_RESULTS_GenericLessThan,
    _RTL_GENERIC_COMPARE_RESULTS_GenericGreaterThan,
    _RTL_GENERIC_COMPARE_RESULTS_GenericEqual,
    RtlInitializeGenericTable, RtlLookupElementGenericTable,
    RtlInsertElementGenericTable, RtlGetElementGenericTable,
    RtlDeleteElementGenericTable,
    ExAllocatePoolWithTag, ExFreePoolWithTag,
    CLONG,
    MmDereferenceSegment, MiAcquirePfnLock, MiReleasePfnLock,
};

extern "system" {
    // Defined in `mm/rmap.rs` (Rust-side, not in bindgen).
    fn MmDeleteSectionAssociation(Page: PFN_NUMBER);
    fn MmInsertRmap(Page: PFN_NUMBER, Process: PVOID, Address: PVOID);
    fn MmGetSegmentRmap(Page: PFN_NUMBER, RawOffset: *mut ULONG) -> PVOID;
}

// MM_PAGED_POOL_TAG is `'tPmM'` in the C source. C multi-char literals are
// big-endian when read in source order, so the in-memory byte sequence is
// "MmPt". `u32::from_le_bytes(*b"MmPt")` yields the same numeric value as the
// C literal.
const MM_SECTION_PAGE_TABLE_TAG: u32 = u32::from_le_bytes(*b"MmPt");

const ENTRIES_PER_ELEMENT: usize = 256;

// `RMAP_SEGMENT_MASK = ~((ULONG_PTR)0xff)` — see ntoskrnl/include/internal/mm.h:938.
// Note: ntkrnl-sys/src/mm.rs has this defined as `!1usize` which is wrong; we
// use the literal value here to match the C header until the sys-crate is fixed.
const RMAP_SEGMENT_MASK: ULONG_PTR = !0xFFusize;

// NonPagedPool — POOL_TYPE variant 0. Bindgen renders POOL_TYPE as i32.
const NON_PAGED_POOL: ntkrnl_sys::ddk::POOL_TYPE = 0;

#[repr(C)]
struct _CACHE_SECTION_PAGE_TABLE {
    pub FileOffset: ntkrnl_sys::ddk::LARGE_INTEGER,
    pub Segment: PMM_SECTION_SEGMENT,
    pub Refcount: ULONG,
    pub PageEntries: [ULONG_PTR; ENTRIES_PER_ELEMENT],
}
type PCACHE_SECTION_PAGE_TABLE = *mut _CACHE_SECTION_PAGE_TABLE;

// SSE (Section Segment Entry) bit-encoding helpers. See ntoskrnl/include/internal/mm.h.
#[inline] fn IS_SWAP_FROM_SSE(e: ULONG_PTR) -> bool { (e & 0x1) != 0 }
#[inline] fn IS_DIRTY_SSE(e: ULONG_PTR) -> bool    { (e & 0x2) != 0 }
#[inline] fn PFN_FROM_SSE(e: ULONG_PTR) -> PFN_NUMBER {
    (e >> PAGE_SHIFT) as PFN_NUMBER
}

#[inline]
unsafe fn InterlockedIncrement64(addend: *mut i64) -> i64 {
    let atom = &*(addend as *const core::sync::atomic::AtomicI64);
    atom.fetch_add(1, core::sync::atomic::Ordering::SeqCst) + 1
}

#[inline]
fn round_down(value: i64, align: i64) -> i64 {
    value & !(align - 1)
}

/* ============================== Generic-table glue =========================
 * The three callbacks register with `RtlInitializeGenericTable`. Their
 * bindgen signatures use `extern "C"` (DDK callback ABI); the original C
 * declares them `NTAPI`. `abi_cast!` bridges the type-system difference; on
 * each target the two ABIs are identical at the machine-code level.
 */

extern "system" fn MiSectionPageTableAllocate(
    _Table: PRTL_GENERIC_TABLE,
    Bytes: CLONG,
) -> PVOID {
    unsafe { ExAllocatePoolWithTag(NON_PAGED_POOL, Bytes as _, MM_SECTION_PAGE_TABLE_TAG) }
}

extern "system" fn MiSectionPageTableFree(
    _Table: PRTL_GENERIC_TABLE,
    Data: PVOID,
) {
    unsafe { ExFreePoolWithTag(Data, MM_SECTION_PAGE_TABLE_TAG) }
}

extern "system" fn MiSectionPageTableCompare(
    _Table: PRTL_GENERIC_TABLE,
    PtrA: PVOID,
    PtrB: PVOID,
) -> RTL_GENERIC_COMPARE_RESULTS {
    unsafe {
        let a = (*(PtrA as *const _LARGE_INTEGER)).QuadPart;
        let b = (*(PtrB as *const _LARGE_INTEGER)).QuadPart;
        if a < b {
            _RTL_GENERIC_COMPARE_RESULTS_GenericLessThan
        } else if a == b {
            _RTL_GENERIC_COMPARE_RESULTS_GenericEqual
        } else {
            _RTL_GENERIC_COMPARE_RESULTS_GenericGreaterThan
        }
    }
}

unsafe fn MiSectionPageTableGet(
    Table: PRTL_GENERIC_TABLE,
    FileOffset: PLARGE_INTEGER,
) -> PCACHE_SECTION_PAGE_TABLE {
    let mut search: LARGE_INTEGER = core::mem::zeroed();
    search.QuadPart = round_down(
        (*FileOffset).QuadPart,
        (ENTRIES_PER_ELEMENT * PAGE_SIZE) as i64,
    );
    RtlLookupElementGenericTable(
        Table,
        core::ptr::addr_of_mut!(search) as PVOID,
    ) as PCACHE_SECTION_PAGE_TABLE
}

// The insert path needs a zeroed `_CACHE_SECTION_PAGE_TABLE` (~2KB, dominated by
// `PageEntries: [ULONG_PTR; 256]`) as the scratch buffer that
// `RtlInsertElementGenericTable` copies into freshly-allocated table memory. Kept
// in its own `#[inline(never)]` function so the 2KB stack frame is reserved ONLY on
// the rare allocate path — never on the common GET-hit path, and never folded into
// the caller (`_MmSetPageEntrySectionSegment`) where it would inflate every call's
// frame. Stack frugality matters: the page-table path runs deep inside the Cc dirty
// flush chain on a kernel stack.
#[inline(never)]
unsafe fn MiSectionPageTableInsert(
    Table: PRTL_GENERIC_TABLE,
    FileOffset: PLARGE_INTEGER,
) -> PCACHE_SECTION_PAGE_TABLE {
    let mut zero: _CACHE_SECTION_PAGE_TABLE = core::mem::zeroed();
    zero.FileOffset.QuadPart = round_down(
        (*FileOffset).QuadPart,
        (ENTRIES_PER_ELEMENT * PAGE_SIZE) as i64,
    );
    zero.Refcount = 1;

    RtlInsertElementGenericTable(
        Table,
        core::ptr::addr_of_mut!(zero) as PVOID,
        core::mem::size_of::<_CACHE_SECTION_PAGE_TABLE>() as ULONG,
        core::ptr::null_mut(),
    ) as PCACHE_SECTION_PAGE_TABLE
}

unsafe fn MiSectionPageTableGetOrAllocate(
    Table: PRTL_GENERIC_TABLE,
    FileOffset: PLARGE_INTEGER,
) -> PCACHE_SECTION_PAGE_TABLE {
    let existing = MiSectionPageTableGet(Table, FileOffset);
    if !existing.is_null() {
        return existing;
    }
    MiSectionPageTableInsert(Table, FileOffset)
}

#[no_mangle]
pub unsafe extern "system" fn MiInitializeSectionPageTable(Segment: PMM_SECTION_SEGMENT) {
    let table_ptr = core::ptr::addr_of_mut!((*Segment).PageTable) as PRTL_GENERIC_TABLE;
    RtlInitializeGenericTable(
        table_ptr,
        Some(abi_cast!(MiSectionPageTableCompare)),
        Some(abi_cast!(MiSectionPageTableAllocate)),
        Some(abi_cast!(MiSectionPageTableFree)),
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn _MmSetPageEntrySectionSegment(
    Segment: PMM_SECTION_SEGMENT,
    Offset: PLARGE_INTEGER,
    Entry: ULONG_PTR,
    _file: *const i8,
    _line: i32,
) -> NTSTATUS {
    ntkrnl_sys::ASSERT!((*Segment).Locked != 0);
    ntkrnl_sys::ASSERT!(!IS_SWAP_FROM_SSE(Entry) || !IS_DIRTY_SSE(Entry));

    let table_ptr = core::ptr::addr_of_mut!((*Segment).PageTable) as PRTL_GENERIC_TABLE;
    let PageTable = MiSectionPageTableGetOrAllocate(table_ptr, Offset);
    if PageTable.is_null() {
        return STATUS_NO_MEMORY;
    }

    ntkrnl_sys::ASSERT!(!MiSectionPageTableGet(table_ptr, Offset).is_null());

    (*PageTable).Segment = Segment;
    let page_index = (((*Offset).QuadPart - (*PageTable).FileOffset.QuadPart) as ULONG_PTR)
        / PAGE_SIZE;
    // Never rewrite the slot of a page that is still on the standby list: take it back
    // first, so a repurpose cannot race this write (mm/standby.rs).
    crate::mm::standby::MiRosStandbyClaim(core::ptr::addr_of_mut!((*PageTable).PageEntries[page_index]));
    let OldEntry: ULONG_PTR = (*PageTable).PageEntries[page_index];

    // Maintain segment refcount: gain a reference on a 0→nonzero transition,
    // drop one on nonzero→0.
    if Entry != 0 && OldEntry == 0 {
        InterlockedIncrement64((*Segment).ReferenceCount);
    }
    if OldEntry != 0 && Entry == 0 {
        MmDereferenceSegment(Segment);
    }

    if Entry != 0 && !IS_SWAP_FROM_SSE(Entry) {
        // Switching to / replacing a valid entry.
        if OldEntry != 0 && !IS_SWAP_FROM_SSE(OldEntry) {
            // Old was also valid: swap rmaps only if the PFN actually changed.
            if PFN_FROM_SSE(Entry) != PFN_FROM_SSE(OldEntry) {
                MmDeleteSectionAssociation(PFN_FROM_SSE(OldEntry));
                // Must update the PTE before re-inserting the rmap to avoid a
                // race with the page-out path.
                (*PageTable).PageEntries[page_index] = Entry;
                MmSetSectionAssociation(PFN_FROM_SSE(Entry), Segment, Offset);
            } else {
                (*PageTable).PageEntries[page_index] = Entry;
            }
        } else {
            // Old was invalid (zero or swap): take a fresh rmap reference.
            (*PageTable).PageEntries[page_index] = Entry;
            MmSetSectionAssociation(PFN_FROM_SSE(Entry), Segment, Offset);

            let last_page_byte = ((*Segment).LastPage as i64) << PAGE_SHIFT as i64;
            if (*Offset).QuadPart >= last_page_byte {
                (*Segment).LastPage = (((*Offset).QuadPart >> PAGE_SHIFT as i64) + 1) as u64;
            }
        }
    } else if OldEntry != 0 && !IS_SWAP_FROM_SSE(OldEntry) {
        // Switching from valid to invalid: drop the rmap.
        MmDeleteSectionAssociation(PFN_FROM_SSE(OldEntry));
        (*PageTable).PageEntries[page_index] = Entry;

        let last_byte_of_last_page = (((*Segment).LastPage as i64) - 1) << PAGE_SHIFT as i64;
        if (*Offset).QuadPart == last_byte_of_last_page {
            // Walk LastPage down past trailing invalid entries.
            while (*Segment).LastPage > 0 {
                (*Segment).LastPage -= 1;
                if (*Segment).LastPage == 0 { break; }
                let mut check_offset: LARGE_INTEGER = core::mem::zeroed();
                check_offset.QuadPart =
                    (((*Segment).LastPage as i64) - 1) << PAGE_SHIFT as i64;
                // Raw read: this only asks whether a page is there, and must not pull a
                // trailing page off the standby list to find out.
                let slot = MiSectionPageEntrySlot(Segment, core::ptr::addr_of_mut!(check_offset));
                let entry = if slot.is_null() { 0 } else { core::ptr::read_volatile(slot) };
                if entry != 0 && !IS_SWAP_FROM_SSE(entry) {
                    break;
                }
            }
        }
    } else {
        (*PageTable).PageEntries[page_index] = Entry;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn _MmGetPageEntrySectionSegment(
    Segment: PMM_SECTION_SEGMENT,
    Offset: PLARGE_INTEGER,
    _file: *const i8,
    _line: i32,
) -> ULONG_PTR {
    ntkrnl_sys::ASSERT!((*Segment).Locked != 0);

    let mut file_offset: LARGE_INTEGER = core::mem::zeroed();
    file_offset.QuadPart = round_down(
        (*Offset).QuadPart,
        (ENTRIES_PER_ELEMENT * PAGE_SIZE) as i64,
    );

    let table_ptr = core::ptr::addr_of_mut!((*Segment).PageTable) as PRTL_GENERIC_TABLE;
    let PageTable = MiSectionPageTableGet(table_ptr, core::ptr::addr_of_mut!(file_offset));
    if PageTable.is_null() {
        return 0;
    }
    let page_index = (((*Offset).QuadPart - (*PageTable).FileOffset.QuadPart) as ULONG_PTR)
        / PAGE_SIZE;
    // A page on the ROS standby list is taken back before anyone sees its entry, and a
    // slot whose page was repurposed reads as not resident (mm/standby.rs).
    let entry = crate::mm::standby::MiRosStandbyClaim(
        core::ptr::addr_of_mut!((*PageTable).PageEntries[page_index]),
    );
    if entry == crate::mm::standby::MM_REPURPOSED_SSE { 0 } else { entry }
}

/// The address of `Offset`'s entry, or null if its table was never allocated. The
/// tables are nonpaged and live until `MmFreePageTablesSectionSegment`, which is what
/// lets the standby list point at a slot from the PFN database.
pub(crate) unsafe fn MiSectionPageEntrySlot(
    Segment: PMM_SECTION_SEGMENT,
    Offset: PLARGE_INTEGER,
) -> *mut ULONG_PTR {
    let table_ptr = core::ptr::addr_of_mut!((*Segment).PageTable) as PRTL_GENERIC_TABLE;
    let PageTable = MiSectionPageTableGet(table_ptr, Offset);
    if PageTable.is_null() {
        return core::ptr::null_mut();
    }
    let page_index = (((*Offset).QuadPart - (*PageTable).FileOffset.QuadPart) as ULONG_PTR)
        / PAGE_SIZE;
    core::ptr::addr_of_mut!((*PageTable).PageEntries[page_index])
}

/// What a segment association rmap entry names: its table (the entry's `Process`) and
/// the index in it (the low byte of its `Address`), as the segment, file offset and slot.
pub(crate) unsafe fn MiSectionPageTableResolve(
    Table: PVOID,
    Index: usize,
    Offset: *mut i64,
) -> (PMM_SECTION_SEGMENT, *mut ULONG_PTR) {
    let table = Table as PCACHE_SECTION_PAGE_TABLE;
    *Offset = (*table).FileOffset.QuadPart + (Index as i64) * (PAGE_SIZE as i64);
    ((*table).Segment, core::ptr::addr_of_mut!((*table).PageEntries[Index]))
}

/// `FREE_SECTION_PAGE_FUN` — called for each non-zero, non-swap entry as the
/// section's page table is torn down. Must match the bindgen-generated
/// callback ABI (`extern "C"`) but is declared `NTAPI` in the original headers
/// — same machine code on all three targets.
pub type FreeSectionPageFun = unsafe extern "C" fn(PMM_SECTION_SEGMENT, *mut LARGE_INTEGER);

#[no_mangle]
pub unsafe extern "system" fn MmFreePageTablesSectionSegment(
    Segment: PMM_SECTION_SEGMENT,
    _KWAIT_REASON_FreePage: Option<FreeSectionPageFun>,
) {
    let table_ptr = core::ptr::addr_of_mut!((*Segment).PageTable) as PRTL_GENERIC_TABLE;
    loop {
        let element_raw = RtlGetElementGenericTable(table_ptr, 0);
        if element_raw.is_null() { break; }
        let element = element_raw as PCACHE_SECTION_PAGE_TABLE;

        if let Some(free_page) = _KWAIT_REASON_FreePage {
            for i in 0..ENTRIES_PER_ELEMENT {
                let entry = (*element).PageEntries[i];
                if entry != 0 && !IS_SWAP_FROM_SSE(entry) {
                    let mut offset: LARGE_INTEGER = core::mem::zeroed();
                    offset.QuadPart =
                        (*element).FileOffset.QuadPart + (i as i64) * (PAGE_SIZE as i64);
                    free_page(Segment, core::ptr::addr_of_mut!(offset));
                }
            }
        }

        RtlDeleteElementGenericTable(table_ptr, element_raw);
    }
}

/// Look up which segment a PFN belongs to via the segment rmap. Returns NULL
/// if the page isn't owned by any segment. On success, fills in `Offset` with
/// the file offset within the segment and bumps the segment's refcount.
#[no_mangle]
pub unsafe extern "system" fn MmGetSectionAssociation(
    Page: PFN_NUMBER,
    Offset: PLARGE_INTEGER,
) -> PMM_SECTION_SEGMENT {
    let mut segment: PMM_SECTION_SEGMENT = core::ptr::null_mut();
    let mut raw_offset: ULONG = 0;

    let old_irql = MiAcquirePfnLock();

    let page_table_raw = MmGetSegmentRmap(Page, core::ptr::addr_of_mut!(raw_offset));
    if !page_table_raw.is_null() {
        let page_table = page_table_raw as PCACHE_SECTION_PAGE_TABLE;
        segment = (*page_table).Segment;
        (*Offset).QuadPart = (*page_table).FileOffset.QuadPart
            + ((raw_offset as u64) << PAGE_SHIFT) as i64;
        ntkrnl_sys::ASSERT!(PFN_FROM_SSE((*page_table).PageEntries[raw_offset as usize]) == Page);
        InterlockedIncrement64((*segment).ReferenceCount);
    }

    MiReleasePfnLock(old_irql);
    segment
}

#[no_mangle]
pub unsafe extern "system" fn MmSetSectionAssociation(
    Page: PFN_NUMBER,
    Segment: PMM_SECTION_SEGMENT,
    Offset: PLARGE_INTEGER,
) -> NTSTATUS {
    let table_ptr = core::ptr::addr_of_mut!((*Segment).PageTable) as PRTL_GENERIC_TABLE;
    let page_table = MiSectionPageTableGet(table_ptr, Offset);
    ntkrnl_sys::ASSERT!(!page_table.is_null());

    let actual_offset =
        (((*Offset).QuadPart - (*page_table).FileOffset.QuadPart) as ULONG_PTR) as ULONG;
    let address = (RMAP_SEGMENT_MASK | ((actual_offset >> PAGE_SHIFT) as ULONG_PTR)) as PVOID;
    MmInsertRmap(Page, page_table as PVOID, address);

    STATUS_SUCCESS
}
