/*
 * The environment the C++ EH reference TUs stand on — cpp.c (#14) and except.c (#15).
 *
 * Deliberately compiled WITHOUT the rename prelude, like sehimpl.c and for the same reason:
 * this file has to name both sides of a pair in one translation unit, so the `#define X ref_X`
 * wall must not be in scope here.  It gets the types from refstub/msvcrt_cpp_types.h directly.
 *
 * Everything here is BELOW the code under test.  Nothing in this file is a translation of
 * anything, and nothing in it is graded — it is the platform the two TUs are graded ON.  The
 * design rule throughout: where a Windows primitive cannot exist on a Linux host, model it as
 * an OBSERVABLE SINK that records what it was asked to do and then refuses to pretend, rather
 * than as a plausible-looking fake.  A fake unwinder would make rows pass that grade nothing.
 */

#include <msvcrt.h>
#include <msvcrt_crt_consts.h>
#include <msvcrt_cpp_types.h>

#include <setjmp.h>
#include <signal.h>
#include <stdio.h>

/* PSEH's dummy arm declares this extern and both TUs reference it; sdk/lib/pseh/dummy.c is
 * where the DLL gets it.  Zero is what makes `__EXCEPT_PAGE_FAULT` compile and never run. */
int _SEH2_Volatile0 = 0;
int _SEH2_VolatileExceptionCode = 0xC0000005;

/* ── RaiseException: the observable sink ──────────────────────────────────
 *
 * `_CxxThrowException` marshals an exception object and its type descriptor into an
 * EXCEPTION_RECORD and calls this.  On Windows it never returns; here there is no exception
 * to raise, so the model records the ARGUMENTS — which is precisely where translation bugs
 * live, and precisely what no other instrument in this harness can see — and then longjmps
 * back to the row.
 *
 * Recording rather than faking is the whole point.  Pretending to unwind would let a row
 * "pass" having graded a control flow that does not exist; a recorded throw lets a row assert
 * the exact code, flags, parameter count and every ExceptionInformation slot, on both sides,
 * and diverge if one of them marshals differently.
 */
struct ref_raise_record {
    int       fired;
    DWORD     code;
    DWORD     flags;
    DWORD     nargs;
    ULONG_PTR args[EXCEPTION_MAXIMUM_PARAMETERS];
};

static struct ref_raise_record ref_raise_log;
static jmp_buf                 ref_raise_landing;
static int                     ref_raise_armed;

void WINAPI RaiseException(DWORD code, DWORD flags, DWORD nargs, const ULONG_PTR *args)
{
    DWORD i;

    ref_raise_log.fired = 1;
    ref_raise_log.code  = code;
    ref_raise_log.flags = flags;
    ref_raise_log.nargs = nargs;
    for (i = 0; i < nargs && i < EXCEPTION_MAXIMUM_PARAMETERS; i++)
        ref_raise_log.args[i] = args ? args[i] : 0;

    /* A throw with nobody watching is a harness bug, not a test result: the row forgot to arm
     * the landing pad, and without this the process would fall off the end of a `noreturn`
     * function and do something undefined.  Fail loud. */
    if (!ref_raise_armed) {
        fprintf(stderr,
                "cppimpl: RaiseException(0x%08lX) with no landing pad armed — a row reached a\n"
                "         throw path it did not set up. Use ref_raise_arm() around it.\n",
                (unsigned long)code);
        fflush(stderr);
        _Exit(93);
    }
    ref_raise_armed = 0;
    longjmp(ref_raise_landing, 1);
}

/* The row-facing API.  `ref_raise_arm` returns 0 the first time and 1 when a throw has landed
 * back on it, exactly like `setjmp` — because it IS one, and it has to be a macro-free
 * function pair rather than a wrapper, since a `setjmp` whose frame has returned is undefined.
 * Hence the buffer is armed here and jumped to above. */
int  ref_raise_arm(void)    { ref_raise_armed = 1; return 0; }
void ref_raise_reset(void)  { ref_raise_log.fired = 0; ref_raise_armed = 0; }
int  ref_raise_fired(void)  { return ref_raise_log.fired; }
DWORD ref_raise_code(void)  { return ref_raise_log.code; }
DWORD ref_raise_flags(void) { return ref_raise_log.flags; }
DWORD ref_raise_nargs(void) { return ref_raise_log.nargs; }
ULONG_PTR ref_raise_arg(unsigned i)
{
    return i < EXCEPTION_MAXIMUM_PARAMETERS ? ref_raise_log.args[i] : 0;
}
jmp_buf *ref_raise_buf(void) { return &ref_raise_landing; }

/* ── RtlPcToFileHeader: the image base the RVA model hangs off ────────────
 *
 * On x86-64 cxx.h sets RTTI_USE_RVA, so every RTTI pointer in the reference's tables is a
 * 32-bit displacement from "the module base".  There is no PE image here, so the model is the
 * one property the arithmetic actually needs: a base far enough below every table that each
 * displacement is a small positive number, and the SAME base every time (`init_*_rtti` writes
 * the displacements once, and a base that moved between the write and the read would decode
 * garbage).
 *
 * `&_SEH2_Volatile0` is used as the anchor rather than a literal: it is a .data object in this
 * very binary, so the whole reference's tables sit within a few hundred KB of it and every
 * `(char *)&x - base` fits in the `unsigned int` the RVA fields are.  Asserted, not assumed —
 * see ref_rtti_base_sane below.
 */
static char *ref_image_base;

void *WINAPI RtlPcToFileHeader(void *pc, void **base)
{
    (void)pc;
    if (!ref_image_base) {
        /* 1 MiB below a .data anchor: below every table, and close enough that no
         * displacement can overflow 32 bits. */
        ref_image_base = (char *)&_SEH2_Volatile0 - (1u << 20);
    }
    if (base)
        *base = ref_image_base;
    return ref_image_base;
}

/* The RVA model's load-bearing assumption, checked rather than believed.  A row calls this
 * with the address of any reference-side RTTI table; a displacement that does not fit is the
 * difference between "the demangled name came out wrong" and "the harness has been decoding
 * one table as another all along". */
int ref_rtti_base_sane(const void *table)
{
    void *base = NULL;
    long long d;
    RtlPcToFileHeader((void *)table, &base);
    d = (const char *)table - (const char *)base;
    return d > 0 && d < 0xFFFFFFFFLL;
}

/* ── the unwind entry points: recorded, never faked ───────────────────────
 *
 * These sit under `__CxxFrameHandler`, which is the part of except.c that genuinely cannot be
 * modelled on this host — it needs a real PE unwind table and a real stack to unwind.  So they
 * record and then fail loud if a row actually reaches them, rather than returning a plausible
 * value that would let a row report a verdict about a dispatch that never happened.
 *
 * That the dispatcher is out of reach is a RECORDED limit of reference TU #15.  The rows it
 * does carry — the frame-info list, the exception-object registration and refcounting, the
 * signal layer — never come here.
 */
static int ref_unwind_reached;

int ref_unwind_was_reached(void) { return ref_unwind_reached; }

static void ref_out_of_reach(const char *what)
{
    ref_unwind_reached = 1;
    fprintf(stderr,
            "cppimpl: %s is not modelled on this host — it needs a real PE unwind table and a\n"
            "         real stack to unwind. A row reached the table-based dispatcher, which\n"
            "         reference TU #15 does not grade. Returning a plausible value here would\n"
            "         make that row report a verdict about a dispatch that never happened.\n",
            what);
    fflush(stderr);
    _Exit(94);
}

PRUNTIME_FUNCTION WINAPI RtlLookupFunctionEntry(ULONG_PTR pc, ULONG_PTR *base,
                                                UNWIND_HISTORY_TABLE *table)
{
    (void)pc;
    (void)table;
    if (base)
        *base = (ULONG_PTR)ref_image_base;
    /* NULL is the HONEST answer: no PC in this binary has a PE unwind entry.  except.c reads
     * it as "no function entry here" and takes its own not-found path, which is a real arm of
     * the code and is graded — unlike the found path below. */
    return NULL;
}

void WINAPI RtlUnwindEx(void *end_frame, void *target_ip, EXCEPTION_RECORD *rec, void *retval,
                        CONTEXT *context, UNWIND_HISTORY_TABLE *table)
{
    (void)end_frame; (void)target_ip; (void)rec; (void)retval; (void)context; (void)table;
    ref_out_of_reach("RtlUnwindEx");
}

/* except_x86_64.c's three asm thunks, which the SUBJECT supplies from its own Rust copy.  The
 * reference gets these instead of binding to the subject's — a reference that calls the code
 * under test is not a reference (§9.66).  They are equally out of reach for the same reason
 * RtlUnwindEx is: each one enters a handler on somebody else's frame. */
void *ref_call_catch_handler(EXCEPTION_RECORD *rec)
{
    (void)rec;
    ref_out_of_reach("call_catch_handler");
    return NULL;
}

void *ref_call_unwind_handler(void *func, uintptr_t frame, DISPATCHER_CONTEXT *dispatch)
{
    (void)func; (void)frame; (void)dispatch;
    ref_out_of_reach("call_unwind_handler");
    return NULL;
}

ULONG_PTR ref_get_exception_pc(DISPATCHER_CONTEXT *dispatch)
{
    return dispatch ? (ULONG_PTR)dispatch->ControlPc : 0;
}

/* The SUBJECT's half of the same hole, and the only symbol msvcrt-crt-rust deliberately leaves
 * undefined off Windows.  `except_x86_64.rs`'s `call_exc_handler` is a PE-only `global_asm!` —
 * COFF `.def`/`.endef` plus four `.seh_*` unwind records, and a body that is Win64 ABI from
 * end to end.  Re-spelling it for ELF would mean inventing a calling sequence, so the shipping
 * module emits nothing there and this is what the link resolves to instead.  Same rule as the
 * reference side above: record and refuse, never fake.  Note it is NOT `ref_`-prefixed — the
 * subject reaches it under its own name. */
void *call_exc_handler(void *handler, ULONG_PTR frame, unsigned flags)
{
    (void)handler; (void)frame; (void)flags;
    ref_out_of_reach("call_exc_handler (the subject's PE-only asm thunk)");
    return NULL;
}

/* `_fpieee_flt` hands a decoded FP fault to the user's handler.  Reaching it needs a real
 * CONTEXT with real x87/SSE state, which this host's CONTEXT is deliberately an opaque block
 * of — so this is the same recorded reach. */
int ref_handle_fpieee_flt(__msvcrt_ulong code, EXCEPTION_POINTERS *ep,
                      int (__cdecl *handler)(_FPIEEE_RECORD *))
{
    (void)code; (void)ep; (void)handler;
    ref_out_of_reach("handle_fpieee_flt");
    return 0;
}

/* ── the small Win32 surface ──────────────────────────────────────────────
 *
 * `SetConsoleCtrlHandler` is RECORDED rather than stubbed: `msvcrt_init_signals` installs
 * msvcrt's console handler through it and `msvcrt_free_signals` removes it, so whether the
 * install happened, with which handler, and whether the remove matched, is a value contract
 * two graded rows compare.  A stub returning TRUE would make both sides look identical while
 * grading nothing.
 */
static void *ref_console_handler;
static int   ref_console_installs;
static int   ref_console_removes;

BOOL WINAPI SetConsoleCtrlHandler(BOOL(WINAPI *handler)(DWORD), BOOL add)
{
    if (add) {
        ref_console_handler = (void *)handler;
        ref_console_installs++;
    } else {
        if ((void *)handler == ref_console_handler)
            ref_console_handler = NULL;
        ref_console_removes++;
    }
    return TRUE;
}

void *ref_console_handler_get(void) { return ref_console_handler; }
int   ref_console_installs_get(void) { return ref_console_installs; }
int   ref_console_removes_get(void) { return ref_console_removes; }
void  ref_console_reset(void)
{
    ref_console_handler = NULL;
    ref_console_installs = ref_console_removes = 0;
}

/* wine/exception.h's frame helpers name this; nothing either TU calls does.  It exists so the
 * header compiles, and returns NULL so a caller that appeared would fault immediately rather
 * than walk an invented TEB. */
void *WINAPI NtCurrentTeb(void) { return NULL; }

/* `operator_delete` is heap.c's, which this crate does not compile as a reference TU.  Routed
 * to the reference's own allocator rather than to the subject's #[no_mangle] `free`, so
 * cpp.c's vector-dtor path frees what cpp.c's ctor allocated.  `ref_free` is string.c's wall
 * name for free — declared here rather than reached through the prelude, which this TU
 * deliberately does not include. */
void __cdecl ref_free(void *ptr);
void __cdecl operator_delete(void *ptr) { ref_free(ptr); }

/* except.c's Ctrl-C path calls `_exit`; the prelude redirects it here.  Same rule as
 * process.c's `ref_process_exit_overlay`: a reference that terminated the process would take
 * the whole suite down mid-row instead of failing one, and silent-green is the one outcome a
 * differential harness must never have. */
static int ref_exit_seen;
static int ref_exit_code;

void ref_cpp_exit_overlay(int code)
{
    ref_exit_seen = 1;
    ref_exit_code = code;
}

int ref_cpp_exit_was_called(void) { return ref_exit_seen; }
int ref_cpp_exit_code(void) { return ref_exit_code; }
void ref_cpp_exit_reset(void) { ref_exit_seen = 0; ref_exit_code = 0; }

/* ── Rust's own startup, out from under the subject's `signal` ────────────
 *
 * except.rs exports `#[no_mangle] signal`, and a definition in the binary beats glibc's for
 * everyone — including Rust's startup, which calls `signal(SIGPIPE, SIG_IGN)` and treats
 * SIG_ERR as fatal.  msvcrt's `signal` has no SIGPIPE, so without this the harness dies before
 * `main` with a message about SIGPIPE and nothing near the cause.
 *
 * build.rs passes `-Wl,--wrap=signal`, which redirects the REFERENCES rather than replacing
 * the definition: the subject keeps its own `signal` exactly as msvcrt.dll has it — which is
 * what tests/cpp.rs grades — and std lands here.  Implemented on `sigaction` because naming
 * `signal` at all would resolve straight back to the subject.  This is verbatim the fix
 * msvcrt-seh-difftest's sehimpl.c already carries, for the same reason.
 */
typedef void (*ref_sighandler)(int);

ref_sighandler __wrap_signal(int sig, ref_sighandler handler)
{
    struct sigaction sa, old;
    memset(&sa, 0, sizeof sa);
    sa.sa_handler = handler;
    sa.sa_flags = SA_RESTART;
    sigemptyset(&sa.sa_mask);
    if (sigaction(sig, &sa, &old) != 0)
        return SIG_ERR;
    return old.sa_handler;
}
