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
 * Header-free since the cargo-driven build PoC: no `<ntoskrnl.h>`, no
 * `<debug.h>`, no `<mm/ARM3/miarm.h>`. The MM_SECTION_SEGMENT and
 * LARGE_INTEGER types appear only as opaque pointers — we never
 * dereference them — so a tiny local typedef suffices.
 */

#include <stddef.h>
#include <stdint.h>

/* --- Local NT type aliases (replaces <ntoskrnl.h>) ------------------ */
typedef uintptr_t         ULONG_PTR;

/* Opaque pointers — we never dereference these. */
struct _MM_SECTION_SEGMENT;
typedef struct _MM_SECTION_SEGMENT *PMM_SECTION_SEGMENT;

struct _LARGE_INTEGER;
typedef struct _LARGE_INTEGER *PLARGE_INTEGER;

#define IN
#if defined(_M_IX86)
# define NTAPI __attribute__((stdcall))
#else
# define NTAPI
#endif

/* --- Externs from ntoskrnl-rust mm crate ---------------------------- */
extern void NTAPI _MmLockSectionSegment(PMM_SECTION_SEGMENT Segment,
                                        const char *file, int line);
extern void NTAPI _MmUnlockSectionSegment(PMM_SECTION_SEGMENT Segment,
                                          const char *file, int line);
extern ULONG_PTR NTAPI _MmGetPageEntrySectionSegment(
    PMM_SECTION_SEGMENT Segment, PLARGE_INTEGER Offset,
    const char *file, int line);

void NTAPI MmLockSectionSegment(IN PMM_SECTION_SEGMENT Segment)
{ _MmLockSectionSegment(Segment, __FILE__, __LINE__); }

void NTAPI MmUnlockSectionSegment(IN PMM_SECTION_SEGMENT Segment)
{ _MmUnlockSectionSegment(Segment, __FILE__, __LINE__); }

ULONG_PTR NTAPI MmGetPageEntrySectionSegment(
    IN PMM_SECTION_SEGMENT Segment, IN PLARGE_INTEGER Offset)
{ return _MmGetPageEntrySectionSegment(Segment, Offset, __FILE__, __LINE__); }
