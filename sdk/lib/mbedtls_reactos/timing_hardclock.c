/*
 * ReactOS-side implementation of mbedtls_timing_hardclock().
 *
 * This file exists so dll/3rdparty/mbedtls stays a PRISTINE upstream drop. Upstream's
 * timing.c offers a QueryPerformanceCounter-based hardclock only under `_MSC_VER`; on
 * this tree (clang/llvm-mingw) that branch is skipped and control falls through to
 * upstream's generic `#if !defined(HAVE_HARDCLOCK)` fallback. The fork used to solve
 * that by editing timing.c to add a `_WIN32` branch — a one-file patch that made every
 * re-vendor a merge.
 *
 * Instead the build now passes -DHAVE_HARDCLOCK, which suppresses EVERY definition in
 * upstream's timing.c (all of them, including the final fallback, are guarded on
 * `!defined(HAVE_HARDCLOCK)`), and the symbol is supplied here. Same implementation
 * upstream uses for MSVC, same API, and timing.c is byte-identical to 2.16.11.
 *
 * NOTE this feeds entropy_poll.c and havege.c as one timer input among several. It
 * does not change entropy policy — it provides the counter the MSVC branch would have
 * provided, on a compiler upstream's guard does not recognise.
 */

#include <windows.h>

unsigned long mbedtls_timing_hardclock(void)
{
    LARGE_INTEGER offset;

    QueryPerformanceCounter(&offset);

    return (unsigned long)(offset.QuadPart);
}
