/* The rename wall for reference TU #13 — `dll/win32/msvcrt/except_i386.c`.
 *
 * Force-included ahead of the genuine C.  Stage order matters and each stage says why.
 */

#ifndef MSVCRT_SEH_PRELUDE_H
#define MSVCRT_SEH_PRELUDE_H

/* ── stage 0: neutralise the two headers that live BESIDE the reference ───
 * A *quoted* include always searches the including file's own directory first, so
 * `#include "msvcrt.h"` and `#include "cppexcept.h"` inside except_i386.c reach the REAL
 * headers no matter what `-I` says — and those drag in the whole PSDK chain.  Claiming
 * their include guards ahead of time makes both expand to nothing without editing a line of
 * the reference.  (The sibling msvcrt-difftest's refstub/msvcrt.h documents the same trick;
 * build.rs asserts these two spellings are still the guards the headers use, because a
 * renamed guard would silently un-shadow them.)
 *
 * The rest — windef.h, winbase.h, winternl.h, excpt.h, fpieee.h, setjmp.h and wine's two —
 * do not live in that directory, so empty sockets on the include path are enough. */
#define __WINE_MSVCRT_H
#define __MSVCRT_CPPEXCEPT_H

/* ── stage 1: the types, before anything can reference them ────────────── */
#include "seh_types.h"

/* ── stage 2: the layout guardian ─────────────────────────────────────────
 * Generated from the REAL i386 headers by tools/gen_msvcrt_seh_layout.py.  It is included
 * HERE, immediately after the transcription and before the reference sees a single line, so
 * a drifted field is a compile error rather than a divergence forty rows later. */
#include "seh_layout.h"

/* ── stage 3: debug channels ─────────────────────────────────────────────
 * TRACE/WARN are off upstream, and ERR/FIXME go to a debugger that is not attached to a
 * Linux test binary.  Neutering them here matches what the Rust does with the same macros —
 * and, unlike the reference's real ones, needs no wine_dbg_sprintf/ros_dbg_log substrate.
 * `(void)0` rather than empty so `if (x) TRACE(...); else ...` still parses. */
#define WINE_DEFAULT_DEBUG_CHANNEL(x)
#define WINE_DECLARE_DEBUG_CHANNEL(x)
#define TRACE(...) ((void)0)
#define WARN(...) ((void)0)
#define ERR(...) ((void)0)
#define FIXME(...) ((void)0)
#define TRACE_ON(ch) 0
#define WARN_ON(ch) 0
#define ERR_ON(ch) 0
#define TRACE_EXCEPTION_TYPE(...) ((void)0)
#define debugstr_a(s) (s)
#define debugstr_w(s) (s)

/* ── stage 4: wine/exception.h ───────────────────────────────────────────
 * The two frame-list inlines are `static inline` in the real header, so they define no
 * symbol on either side and are substrate, not graded text.  They go straight to fs:0
 * exactly as the header does — sehimpl.c's `seh_teb_install()` is what makes that address
 * a real Windows-shaped TEB on this host.
 *
 * `__TRY`/`__EXCEPT_CTX`/`__ENDTRY` run the guarded body and never the handler.  That is
 * NOT a simplification chosen for the harness: except_i386.rs translates the same block as
 * a bare call to `data->se_translator`, with no guard, so this is what makes the two sides
 * the same program.  Every row leaves `se_translator` NULL, so the block is not entered at
 * all and `se_translation_filter` stays ungraded — recorded in the crate docstring. */
static inline EXCEPTION_REGISTRATION_RECORD *__wine_push_frame(EXCEPTION_REGISTRATION_RECORD *frame)
{
    EXCEPTION_REGISTRATION_RECORD *prev;
    __asm__ __volatile__("movl %%fs:0,%0" : "=r"(prev));
    frame->Prev = prev;
    __asm__ __volatile__("movl %0,%%fs:0" ::"r"(frame));
    return prev;
}

static inline EXCEPTION_REGISTRATION_RECORD *__wine_pop_frame(EXCEPTION_REGISTRATION_RECORD *frame)
{
    __asm__ __volatile__("movl %0,%%fs:0" ::"r"(frame->Prev));
    return frame->Prev;
}

static inline TEB *NtCurrentTeb(void)
{
    TEB *teb;
    __asm__ __volatile__("movl %%fs:0x18,%0" : "=r"(teb));
    return teb;
}

#define __TRY {
#define __EXCEPT_CTX(func, ctx) } if (0) {
#define __ENDTRY }

/* ── stage 5: the substrate the reference calls but does not define ───────
 * Everything below is SHARED ENVIRONMENT, resolved to ONE implementation both sides call.
 *
 *   * `find_catch_handler`, `dump_function_descr`, `_CreateFrameInfo`,
 *     `__CxxUnregisterExceptionObject`, `__DestructExceptionObject`, `terminate` come from
 *     the SUBJECT crate (cpp.rs / except.rs).  Sharing them is deliberate: they are the
 *     C++ object registry, graded — if at all — as cpp.c, and a second copy here would be
 *     an oracle grading a transcription of itself.
 *   * `_global_unwind2` and `__wine_RtlUnwind` come from sehimpl.c.  In the DLL the first
 *     is pseh's asm and the second msvcrt's own `i386/__wine_RtlUnwind.s`; neither can run
 *     on a Linux host, so the harness records the call instead.  A row grades the RECORD,
 *     which is the argument list and the order — the two things a translation can get
 *     wrong. */
extern void *find_catch_handler(void *object, ULONG_PTR frame, ULONG_PTR exc_base,
                                const tryblock_info *tryblock, cxx_exception_type *exc_type,
                                ULONG_PTR image_base);
extern void dump_function_descr(const cxx_function_descr *descr, ULONG_PTR base);
extern frame_info *_CreateFrameInfo(frame_info *fi, void *obj);
extern void __CxxUnregisterExceptionObject(cxx_frame_info *, BOOL);
extern void __DestructExceptionObject(EXCEPTION_RECORD *);
extern void terminate(void);
extern thread_data_t *msvcrt_get_thread_data(void);
extern void _global_unwind2(EXCEPTION_REGISTRATION_RECORD *frame);
/* stdcall, because msvcrt-sys declares it `extern "stdcall"` and the SUBJECT calls it that
 * way; a cdecl declaration here would have the two sides disagreeing about who pops. */
extern void SEH_WINAPI __wine_RtlUnwind(EXCEPTION_REGISTRATION_RECORD *frame, void *target,
                                        EXCEPTION_RECORD *rec, void *retval);

/* msvcrt's CMakeLists compiles this TU with `-DRtlUnwind=__wine_RtlUnwind`; the same
 * substitution here is what keeps the call sites reading as the C reads. */
#define RtlUnwind __wine_RtlUnwind

/* ── stage 6: the asm thunks are NOT redefined by the reference ───────────
 * `__ASM_GLOBAL_FUNC` stringifies its first argument, so a `#define call_filter
 * ref_call_filter` cannot reach the emitted label — the reference would define the SAME
 * six symbols as the subject and the link would fail.  Neutering the macro is the honest
 * resolution rather than a workaround: those six bodies are irreducible machine code, they
 * are already proven byte-for-byte against the C objects by objdump, and seh_i386.rs emits
 * them from ONE instruction text under two symbol decorations.  A row that "graded" them
 * would be comparing a transcription with itself. */
#define __ASM_GLOBAL_FUNC(name, code)
#define __ASM_NAME(name) name
#define __ASM_CFI(str)
#define __ASM_STDCALL(name, args) name

/* No declarations are needed here: except_i386.c declares all four itself, immediately
 * above each `__ASM_GLOBAL_FUNC` it no longer expands, so the reference's call sites stay
 * checked against the C's own prototypes rather than against a copy of them. */

/* ── stage 7: the rename wall ────────────────────────────────────────────
 * Only the externally-visible definitions need renaming; except_i386.c's ten `static`
 * bodies cannot collide with the subject's and are reached THROUGH these. */
#define cxx_frame_handler ref_cxx_frame_handler
#define __CxxLongjmpUnwind ref___CxxLongjmpUnwind
#define _local_unwind4 ref__local_unwind4
#define _except_handler4_common ref__except_handler4_common
#define _seh_longjmp_unwind ref__seh_longjmp_unwind
#define _seh_longjmp_unwind4 ref__seh_longjmp_unwind4
#define handle_fpieee_flt ref_handle_fpieee_flt

#endif /* MSVCRT_SEH_PRELUDE_H */
