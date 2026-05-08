/*
 * mm_lock_outlines.c — minimal trampoline for `MmLockSectionSegment`,
 * `MmUnlockSectionSegment`, `MmGetPageEntrySectionSegment`.
 *
 * Each just forwards to its `_Mm*` underlying primitive with `__FILE__` /
 * `__LINE__` as extra arguments — those C macros expand at the call site
 * into the recording string used by the lock-tracking diagnostics, and
 * Rust can't replicate that without `core::file!()` / `core::line!()` at
 * macro_rules level (and even then the resulting strings would be Rust
 * source paths, not the C-side paths the diagnostics want to see).
 *
 * Headers: needs `<ntoskrnl.h>` for `PMM_SECTION_SEGMENT` /
 * `PLARGE_INTEGER` and the underscored primitives' declarations.
 * Pulling miarm.h directly via the ARM3 path because the prototypes
 * for `_MmLockSectionSegment` etc. are nested in the kernel-private
 * MM internals.
 */

#include <ntoskrnl.h>
#define NDEBUG
#include <debug.h>
#include <mm/ARM3/miarm.h>

#undef MmLockSectionSegment
#undef MmUnlockSectionSegment
#undef MmGetPageEntrySectionSegment

void NTAPI MmLockSectionSegment(IN PMM_SECTION_SEGMENT Segment)
{ _MmLockSectionSegment(Segment, __FILE__, __LINE__); }

void NTAPI MmUnlockSectionSegment(IN PMM_SECTION_SEGMENT Segment)
{ _MmUnlockSectionSegment(Segment, __FILE__, __LINE__); }

ULONG_PTR NTAPI MmGetPageEntrySectionSegment(
    IN PMM_SECTION_SEGMENT Segment, IN PLARGE_INTEGER Offset)
{ return _MmGetPageEntrySectionSegment(Segment, Offset, __FILE__, __LINE__); }
