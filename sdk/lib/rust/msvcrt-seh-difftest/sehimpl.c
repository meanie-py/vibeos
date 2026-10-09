/* msvcrt-seh-difftest's substrate: a Windows-shaped 32-bit SEH environment on Linux, the
 * shared sinks the reference and the subject both call, and one duel driver per graded
 * export.
 *
 * THE TEB
 * -------
 * 32-bit SEH is not a table lookup; it is a linked list whose head lives at `fs:0`, and
 * `NtCurrentTeb()` reads the TEB's self-pointer from `fs:0x18`.  Both sides go straight to
 * those addresses — the subject through seh_i386.rs's `asm!`, the reference through
 * wine/exception.h's inlines — so the harness has to make them real.  On i386 Linux `%fs`
 * is a free selector (glibc's TLS is on `%gs`), so `set_thread_area` installs a descriptor
 * whose base is a TEB-shaped block of ours and `%fs` is loaded with it.  That is what Wine
 * does, and it is why this harness can exist at all.
 *
 * WHY THE DRIVERS LIVE HERE AND NOT IN THE RUST TEST
 * --------------------------------------------------
 * Two reasons, both structural.  First, one path under test ENDS IN A JUMP:
 * `call_finally_block` is DECLSPEC_NORETURN and leaves on a foreign stack, so the only way
 * to grade the `__finally` arm is to `setjmp` before the call and let the trampoline
 * `longjmp` back — and a `setjmp` whose frame has returned is undefined, so the setjmp and
 * the call must sit in one C function.  Second, calling both sides from ONE C file with ONE
 * set of declarations makes "the same call" true by construction rather than by two
 * hand-matched FFI signatures.
 *
 * THE LEDGER IS A SINK, AND THAT IS FINE
 * ---------------------------------------
 * There is one event log, not one per side, and the protocol is reset -> subject -> snapshot
 * -> reset -> reference -> snapshot -> compare.  This is the shape processimpl.c already
 * uses; it is not the §9.73 trap, because nothing is ever compared across a reset.
 */

#include <setjmp.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/syscall.h>
#include <unistd.h>
#include <asm/ldt.h>

/* seh_types.h is force-included by build.rs ahead of this file too, so the Win32 shapes here
 * are the SAME transcription the reference compiles against — one definition, checked by
 * seh_layout.h.  glibc's <setjmp.h> is included above it on purpose: this file needs the
 * real `jmp_buf`, and the reference never sees it. */

/* ── the fake TEB ─────────────────────────────────────────────────────── */

#define TEB_BLOCK 0x1000
/* PSEH's dummy arm: sdk/lib/pseh/dummy.c defines this for the DLL. except_i386.rs reads it
 * to guard cxx_frame_handler's `__EXCEPT_CTX(se_translation_filter, &ctx)`, exactly as
 * except.rs does on x64 (msvcrt-difftest/cppimpl.c carries the same definition). Zero is
 * DUMMY_PSEH: the filter compiles and never runs. */
int _SEH2_Volatile0 = 0;

static unsigned char g_teb_block[TEB_BLOCK] __attribute__((aligned(16)));
static int g_teb_ready;

/* Windows' end-of-chain sentinel. */
#define SEH_CHAIN_END ((unsigned)0xffffffffu)

void seh_teb_install(void)
{
    struct user_desc d;
    unsigned sel;

    if (g_teb_ready)
        return;

    memset(&d, 0, sizeof d);
    d.entry_number = -1; /* let the kernel pick a free GDT slot */
    d.base_addr = (unsigned long)g_teb_block;
    d.limit = 0xfffff;
    d.seg_32bit = 1;
    d.contents = 0;
    d.read_exec_only = 0;
    d.limit_in_pages = 1;
    d.seg_not_present = 0;
    d.useable = 1;

    if (syscall(SYS_set_thread_area, &d) != 0)
    {
        /* HARD FAIL, never a skip: a harness that quietly ran without a TEB would take the
         * `fs:0` path into a null selector and die somewhere unrelated, or — worse, on a
         * kernel that happened to map it — grade both sides against garbage. */
        perror("msvcrt-seh-difftest: set_thread_area");
        fprintf(stderr,
                "This harness needs a writable %%fs segment to stand in for the Windows TEB.\n"
                "Without it neither fs:0 (the SEH chain head) nor fs:0x18 (NtCurrentTeb)\n"
                "means anything, and every row would grade noise.\n");
        exit(70);
    }

    sel = ((unsigned)d.entry_number << 3) | 3;
    __asm__ __volatile__("movw %w0, %%fs" ::"r"(sel));

    /* NT_TIB.Self at +0x18 must point at the TEB itself — that IS NtCurrentTeb(). */
    *(unsigned *)(g_teb_block + 0x18) = (unsigned)(unsigned long)g_teb_block;
    *(unsigned *)(g_teb_block + 0x00) = SEH_CHAIN_END;
    g_teb_ready = 1;
}

static unsigned chain_head(void)
{
    unsigned v;
    __asm__ __volatile__("movl %%fs:0,%0" : "=r"(v));
    return v;
}

static void chain_set(unsigned v) { __asm__ __volatile__("movl %0,%%fs:0" ::"r"(v)); }

unsigned seh_chain_head(void) { return chain_head(); }

/* ── the event ledger ─────────────────────────────────────────────────── */

enum
{
    SEH_EV_FILTER = 1,
    SEH_EV_HANDLER,
    SEH_EV_FINALLY,
    SEH_EV_GLOBAL_UNWIND2,
    SEH_EV_RTLUNWIND,
    SEH_EV_NESTED,
    SEH_EV_CUSTOM
};

#define SEH_MAX_EVENTS 64
#define SEH_MAX_SCOPES 16

typedef struct
{
    int kind;
    int id;         /* which callback slot fired, or a disposition for NESTED */
    unsigned ebp;   /* the %ebp the asm thunk installed before calling us */
    int trylevel;   /* the driver's frame trylevel at that moment */
    unsigned chain; /* fs:0 at that moment — proves __wine_push_frame ran */
    unsigned arg;
} seh_event;

static seh_event g_ev[SEH_MAX_EVENTS];
static int g_ev_n;
static int g_ev_overflow;

/* Which frame's `trylevel` an event should snapshot.  Set by each driver, so the ledger
 * follows the structure actually being walked instead of guessing. */
static int *g_trylevel_ptr;

static void ev_push(int kind, int id, unsigned ebp, unsigned arg)
{
    if (g_ev_n >= SEH_MAX_EVENTS)
    {
        g_ev_overflow = 1;
        return;
    }
    g_ev[g_ev_n].kind = kind;
    g_ev[g_ev_n].id = id;
    g_ev[g_ev_n].ebp = ebp;
    g_ev[g_ev_n].trylevel = g_trylevel_ptr ? *g_trylevel_ptr : -999;
    g_ev[g_ev_n].chain = chain_head();
    g_ev[g_ev_n].arg = arg;
    g_ev_n++;
}

int seh_events(void) { return g_ev_n; }
int seh_event_overflow(void) { return g_ev_overflow; }
int seh_event_kind(int i) { return (i >= 0 && i < g_ev_n) ? g_ev[i].kind : 0; }
int seh_event_id(int i) { return (i >= 0 && i < g_ev_n) ? g_ev[i].id : -1; }
unsigned seh_event_ebp(int i) { return (i >= 0 && i < g_ev_n) ? g_ev[i].ebp : 0; }
int seh_event_trylevel(int i) { return (i >= 0 && i < g_ev_n) ? g_ev[i].trylevel : -999; }
unsigned seh_event_chain(int i) { return (i >= 0 && i < g_ev_n) ? g_ev[i].chain : 0; }
unsigned seh_event_arg(int i) { return (i >= 0 && i < g_ev_n) ? g_ev[i].arg : 0; }

/* ── the structures a row builds ──────────────────────────────────────── */

typedef struct
{
    int previousTryLevel;
    int (*lpfnFilter)(EXCEPTION_POINTERS *);
    void *(*lpfnHandler)(void);
} SCOPETABLE;

typedef struct
{
    int gs_cookie_offset;
    unsigned long gs_cookie_xor;
    int eh_cookie_offset;
    unsigned long eh_cookie_xor;
    SCOPETABLE entries[SEH_MAX_SCOPES];
} SCOPETABLE_V4;

typedef struct
{
    EXCEPTION_REGISTRATION_RECORD *prev;
    void (*handler)(EXCEPTION_RECORD *, EXCEPTION_REGISTRATION_RECORD *, CONTEXT *,
                    EXCEPTION_RECORD *);
    SCOPETABLE *scopetable;
    int trylevel;
    int _ebp;
    EXCEPTION_POINTERS *xpointers;
} MSVCRT_EXCEPTION_FRAME;

typedef struct
{
    EXCEPTION_REGISTRATION_RECORD frame;
    int trylevel;
    unsigned long ebp;
} cxx_exception_frame;

/* Both frames carry a leading scratch word, because the C really does write through
 * `*((DWORD *)frame - 1)` — that slot is the caller's `xpointers`.  A frame allocated as a
 * bare object would have the handler scribbling on whatever precedes it. */
static struct
{
    unsigned scratch;
    MSVCRT_EXCEPTION_FRAME frame;
} g_blk;

static struct
{
    unsigned scratch;
    cxx_exception_frame frame;
} g_cxx;

static SCOPETABLE_V4 g_v4;
static SCOPETABLE g_v3[SEH_MAX_SCOPES];
static int g_scopes;
static unsigned long g_cookie;

static EXCEPTION_RECORD g_rec;
static CONTEXT g_ctx;
static _JUMP_BUFFER g_jmp;
static EXCEPTION_REGISTRATION_RECORD *g_dispatcher;

static int g_filter_result[SEH_MAX_SCOPES];
static int g_probe_nested_flags = -1; /* -1 = do not probe */

static unwind_info g_unwind[SEH_MAX_SCOPES];
static tryblock_info g_tryblock[SEH_MAX_SCOPES];
static cxx_function_descr g_descr;

static thread_data_t g_td;

thread_data_t *msvcrt_get_thread_data(void) { return &g_td; }

/* ── the callbacks the tables point at ────────────────────────────────── */

/* The %ebp the asm thunk installed.  `call_handler` CALLs and `call_finally_block` JMPs, but
 * either way the value is live in %ebp when our callback's own prologue pushes it — this
 * file is compiled -fno-omit-frame-pointer so that slot exists and this reads it. */
#define INCOMING_EBP (*(unsigned *)__builtin_frame_address(0))

/* When a row asks for it, a callback also reaches the CURRENT head of the SEH chain and
 * calls its Handler.  That head is the registration record `msvcrt_local_unwind2/4` pushed,
 * and its Handler is that side's own `MSVCRT_nested_handler` — which is `static`, so this is
 * the only way to grade it. */
static EXCEPTION_RECORD g_probe_rec;
static CONTEXT g_probe_ctx;

static void probe_nested(void)
{
    EXCEPTION_REGISTRATION_RECORD *head;
    EXCEPTION_REGISTRATION_RECORD *dispatch = NULL;
    int disp;

    if (g_probe_nested_flags < 0)
        return;
    head = (EXCEPTION_REGISTRATION_RECORD *)(unsigned long)chain_head();
    if (!head || (unsigned long)head == SEH_CHAIN_END || !head->Handler)
        return;

    memset(&g_probe_rec, 0, sizeof g_probe_rec);
    g_probe_rec.ExceptionCode = 0xc0000005;
    g_probe_rec.ExceptionFlags = (DWORD)g_probe_nested_flags;
    disp = (int)head->Handler(&g_probe_rec, head, &g_probe_ctx, &dispatch);
    ev_push(SEH_EV_NESTED, disp, 0, dispatch == head ? 1 : 0);
}

static jmp_buf g_recover;
static volatile int g_left_by_jump;

#define DEF_FILTER(n)                                                                        \
    static int seh_filter##n(EXCEPTION_POINTERS *p)                                          \
    {                                                                                        \
        unsigned ebp = INCOMING_EBP;                                                         \
        ev_push(SEH_EV_FILTER, n, ebp, (unsigned)(unsigned long)p);                          \
        probe_nested();                                                                      \
        return g_filter_result[n];                                                           \
    }

#define DEF_HANDLER(n)                                                                       \
    static void *seh_handler##n(void)                                                        \
    {                                                                                        \
        unsigned ebp = INCOMING_EBP;                                                         \
        ev_push(SEH_EV_HANDLER, n, ebp, 0);                                                  \
        probe_nested();                                                                      \
        return (void *)(unsigned long)(0x51000000u + n);                                     \
    }

/* A `__finally` arm: `call_finally_block` JUMPS here and never expects a return, so this is
 * where the row comes back from. */
#define DEF_FINALLY(n)                                                                       \
    static void *seh_finally##n(void)                                                        \
    {                                                                                        \
        unsigned ebp = INCOMING_EBP;                                                          \
        ev_push(SEH_EV_FINALLY, n, ebp, 0);                                                  \
        g_left_by_jump = 1;                                                                  \
        longjmp(g_recover, 1);                                                               \
        return 0;                                                                            \
    }

DEF_FILTER(0)
DEF_FILTER(1)
DEF_FILTER(2)
DEF_FILTER(3)
DEF_FILTER(4)
DEF_FILTER(5)
DEF_FILTER(6)
DEF_FILTER(7)

DEF_HANDLER(0)
DEF_HANDLER(1)
DEF_HANDLER(2)
DEF_HANDLER(3)
DEF_HANDLER(4)
DEF_HANDLER(5)
DEF_HANDLER(6)
DEF_HANDLER(7)

DEF_FINALLY(0)
DEF_FINALLY(1)
DEF_FINALLY(2)
DEF_FINALLY(3)

static int (*const FILTERS[8])(EXCEPTION_POINTERS *) = {
    seh_filter0, seh_filter1, seh_filter2, seh_filter3,
    seh_filter4, seh_filter5, seh_filter6, seh_filter7,
};
static void *(*const HANDLERS[8])(void) = {
    seh_handler0, seh_handler1, seh_handler2, seh_handler3,
    seh_handler4, seh_handler5, seh_handler6, seh_handler7,
};
static void *(*const FINALLYS[4])(void) = {
    seh_finally0,
    seh_finally1,
    seh_finally2,
    seh_finally3,
};

/* ── the shared Win32/NT sinks ────────────────────────────────────────── */

/* In msvcrt.dll these are pseh's `i386/seh.s` and msvcrt's own `i386/__wine_RtlUnwind.s`;
 * neither can run on a Linux host, so the harness records the call.  A row grades the
 * RECORD — the arguments and the order — which is what a translation can get wrong. */
void _global_unwind2(EXCEPTION_REGISTRATION_RECORD *frame)
{
    ev_push(SEH_EV_GLOBAL_UNWIND2, 0, 0, (unsigned)(unsigned long)frame);
}

void SEH_WINAPI __wine_RtlUnwind(EXCEPTION_REGISTRATION_RECORD *frame, void *target,
                                 EXCEPTION_RECORD *rec, void *retval)
{
    (void)target;
    (void)retval;
    ev_push(SEH_EV_RTLUNWIND, 0, (unsigned)(unsigned long)rec, (unsigned)(unsigned long)frame);
}

static char g_current_row[160];

/* ── symbols the LINKER wants, that no row reaches ────────────────────────
 *
 * `msvcrt-crt-rust` is one rlib, and its codegen units do not follow module boundaries: the
 * member that carries except_i386.rs also carries printf and locale code, so pulling in the
 * seven exports pulls in references to twenty Win32/CRT entry points nothing here calls.
 *
 * They are LOUD, not empty.  An empty stub for a locale or invalid-parameter hook is how a
 * harness silently grades a path it never modelled — the row still runs, both sides take the
 * same wrong branch, and it reports MATCH.  Aborting names the symbol and stops.
 *
 * The three heap entry points are the exception and are real: cpp.rs's exception-object
 * registry may allocate, and a working malloc there costs nothing and models the DLL
 * faithfully.
 */
#define UNMODELLED(name)                                                                     \
    do                                                                                       \
    {                                                                                        \
        fprintf(stderr,                                                                      \
                "\nmsvcrt-seh-difftest: a row reached `%s`, which this harness does not\n"   \
                "model. That is not a divergence — it means the scenario walked into code\n" \
                "outside the SEH substrate, so whatever it would have compared is not\n"     \
                "meaningful. Model it or narrow the row.\nrow: %s\n",                        \
                name, g_current_row);                                                        \
        fflush(stderr);                                                                      \
        _Exit(71);                                                                           \
    } while (0)

void *SEH_WINAPI GetProcessHeap(void) { return (void *)(unsigned long)0x48454150u; /* 'HEAP' */ }
void *SEH_WINAPI HeapAlloc(void *heap, unsigned flags, size_t n)
{
    void *p;
    (void)heap;
    p = malloc(n ? n : 1);
    if (p && (flags & 8)) /* HEAP_ZERO_MEMORY */
        memset(p, 0, n);
    return p;
}
int SEH_WINAPI HeapFree(void *heap, unsigned flags, void *p)
{
    (void)heap;
    (void)flags;
    free(p);
    return 1;
}

/* The aborting stubs below carry no convention on purpose: they call `_Exit` and never
 * return, so nobody pops anything. Only a stub that RETURNS has to match its caller. */
int CompareStringA(void) { UNMODELLED("CompareStringA"); }
int CompareStringW(void) { UNMODELLED("CompareStringW"); }
int GetStringTypeExA(void) { UNMODELLED("GetStringTypeExA"); }
int GetStringTypeW(void) { UNMODELLED("GetStringTypeW"); }
int GetTimeZoneInformation(void) { UNMODELLED("GetTimeZoneInformation"); }
int LCMapStringA(void) { UNMODELLED("LCMapStringA"); }
int LCMapStringW(void) { UNMODELLED("LCMapStringW"); }
int MultiByteToWideChar(void) { UNMODELLED("MultiByteToWideChar"); }
int WideCharToMultiByte(void) { UNMODELLED("WideCharToMultiByte"); }
int __crtLCMapStringA(void) { UNMODELLED("__crtLCMapStringA"); }
int lstrcpynA(void) { UNMODELLED("lstrcpynA"); }
void _invalid_parameter(void) { UNMODELLED("_invalid_parameter"); }
int _snprintf(void) { UNMODELLED("_snprintf"); }
void *get_locinfo(void) { UNMODELLED("get_locinfo"); }

int CreateFileA(void) { UNMODELLED("CreateFileA"); }
int DebugBreak(void) { UNMODELLED("DebugBreak"); }
int ExitProcess(void) { UNMODELLED("ExitProcess"); }
int GetActiveWindow(void) { UNMODELLED("GetActiveWindow"); }
int GetLastError(void) { UNMODELLED("GetLastError"); }
int GetModuleHandleW(void) { UNMODELLED("GetModuleHandleW"); }
int GetProcAddress(void) { UNMODELLED("GetProcAddress"); }
int MessageBoxIndirectW(void) { UNMODELLED("MessageBoxIndirectW"); }
int NtQuerySystemTime(void) { UNMODELLED("NtQuerySystemTime"); }
int WriteConsoleA(void) { UNMODELLED("WriteConsoleA"); }
int _snwprintf(void) { UNMODELLED("_snwprintf"); }
/* cppexcept.h's thiscall wrappers.  A row that reaches one is destroying a real C++
 * exception object, which is the catch-dispatch half this harness deliberately stops short
 * of — so reaching it means the scenario went further than intended. */
int call_copy_ctor(void) { UNMODELLED("call_copy_ctor"); }
int call_dtor(void) { UNMODELLED("call_dtor"); }

/* The CRT's locks. `__DestructExceptionObject` and the exception-object registry take them,
 * so these ARE on graded paths — and a single-threaded harness needs nothing more than a
 * pair that returns. They are no-ops rather than aborts for that reason, and the reason is
 * worth stating: an UNMODELLED abort here would have been wrong, because the row really
 * does reach them and really is comparable across the two sides that both go through the
 * same shared registry. */
void _lock(int which) { (void)which; }
void _unlock(int which) { (void)which; }
void SEH_WINAPI EnterCriticalSection(void *cs) { (void)cs; }
void SEH_WINAPI LeaveCriticalSection(void *cs) { (void)cs; }

/* MESSAGE OUTPUT IS NOT GRADED, and this is where that is decided.
 *
 * The reference's `ERR`/`FIXME` are neutered by seh_prelude.h; the subject's expand to an
 * `OutputDebugStringA` of a bare literal (except_i386.rs drops the format arguments on
 * purpose, and says so).  The two could not be compared even in principle, so this is a
 * silent sink rather than a ledger event — recording it would make every invalid-magic row
 * report a divergence that is really a difference in how the two TRACE macros were defined.
 * The `cxx_frame_handler` rows below therefore grade its RETURN and its effects, not its
 * diagnostics. */
void SEH_WINAPI OutputDebugStringA(const char *s) { (void)s; }

/* DATA symbols from the same pull-in.  Zeroed rather than absent: a data reference has no
 * call site to abort in, so the honest thing is to make sure any read is of a defined object
 * rather than of whatever followed it in the image. */
WORD *MSVCRT__pwctype;
BOOL initial_locale;
void *MSVCRT__wpgmptr;
int MSVCRT_app_type;

/* Rust's startup calls `signal(SIGPIPE, SIG_IGN)` and treats SIG_ERR as fatal.  Linked
 * against a CRT under test, that call reaches msvcrt's `signal`, which has no SIGPIPE — so
 * the harness died before main with a message about SIGPIPE.  build.rs passes
 * `--wrap=signal` so std's reference lands here instead; the subject keeps its own
 * definition untouched.  Implemented on `sigaction` rather than by calling through to
 * `signal`, because naming `signal` at all would resolve straight back to the subject. */
typedef void (*seh_sighandler)(int);

seh_sighandler __wrap_signal(int sig, seh_sighandler handler)
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

/* ── the two sides ────────────────────────────────────────────────────── */

extern int _except_handler4_common(unsigned long *cookie, void (*check_cookie)(void),
                                   EXCEPTION_RECORD *rec, MSVCRT_EXCEPTION_FRAME *frame,
                                   CONTEXT *context, EXCEPTION_REGISTRATION_RECORD **dispatcher);
extern int ref__except_handler4_common(unsigned long *cookie, void (*check_cookie)(void),
                                       EXCEPTION_RECORD *rec, MSVCRT_EXCEPTION_FRAME *frame,
                                       CONTEXT *context,
                                       EXCEPTION_REGISTRATION_RECORD **dispatcher);

extern void _local_unwind4(unsigned long *cookie, MSVCRT_EXCEPTION_FRAME *frame, int trylevel);
extern void ref__local_unwind4(unsigned long *cookie, MSVCRT_EXCEPTION_FRAME *frame,
                               int trylevel);

extern void _seh_longjmp_unwind(_JUMP_BUFFER *jmp);
extern void ref__seh_longjmp_unwind(_JUMP_BUFFER *jmp);

extern void _seh_longjmp_unwind4(_JUMP_BUFFER *jmp);
extern void ref__seh_longjmp_unwind4(_JUMP_BUFFER *jmp);

extern void __CxxLongjmpUnwind(const _JUMP_BUFFER *buf);
extern void ref___CxxLongjmpUnwind(const _JUMP_BUFFER *buf);

extern DWORD cxx_frame_handler(EXCEPTION_RECORD *rec, cxx_exception_frame *frame, CONTEXT *context,
                               EXCEPTION_REGISTRATION_RECORD **dispatch,
                               const cxx_function_descr *descr, void *nested_frame);
extern DWORD ref_cxx_frame_handler(EXCEPTION_RECORD *rec, cxx_exception_frame *frame,
                                   CONTEXT *context, EXCEPTION_REGISTRATION_RECORD **dispatch,
                                   const cxx_function_descr *descr, void *nested_frame);

extern int handle_fpieee_flt(unsigned long code, EXCEPTION_POINTERS *ep,
                             int (*handler)(_FPIEEE_RECORD *));
extern int ref_handle_fpieee_flt(unsigned long code, EXCEPTION_POINTERS *ep,
                                 int (*handler)(_FPIEEE_RECORD *));

/* ── the scenario API ─────────────────────────────────────────────────── */

void seh_reset(void)
{
    seh_teb_install();
    memset(g_ev, 0, sizeof g_ev);
    g_ev_n = 0;
    g_ev_overflow = 0;
    g_trylevel_ptr = &g_blk.frame.trylevel;

    memset(&g_blk, 0, sizeof g_blk);
    memset(&g_cxx, 0, sizeof g_cxx);
    memset(&g_v4, 0, sizeof g_v4);
    memset(g_v3, 0, sizeof g_v3);
    memset(&g_rec, 0, sizeof g_rec);
    memset(&g_ctx, 0, sizeof g_ctx);
    memset(&g_jmp, 0, sizeof g_jmp);
    memset(g_unwind, 0, sizeof g_unwind);
    memset(g_tryblock, 0, sizeof g_tryblock);
    memset(&g_descr, 0, sizeof g_descr);
    memset(&g_td, 0, sizeof g_td);
    memset(&g_probe_rec, 0, sizeof g_probe_rec);
    memset(&g_probe_ctx, 0, sizeof g_probe_ctx);
    g_scopes = 0;
    g_cookie = 0;
    g_dispatcher = NULL;
    g_left_by_jump = 0;
    g_probe_nested_flags = -1;
    for (int i = 0; i < SEH_MAX_SCOPES; i++)
        g_filter_result[i] = EXCEPTION_CONTINUE_SEARCH;
    chain_set(SEH_CHAIN_END);
}

void seh_set_cookie(unsigned long c) { g_cookie = c; }
void seh_set_trylevel(int t) { g_blk.frame.trylevel = t; }
void seh_set_cxx_trylevel(int t) { g_cxx.frame.trylevel = t; }
void seh_set_exception(unsigned code, unsigned flags)
{
    g_rec.ExceptionCode = code;
    g_rec.ExceptionFlags = flags;
    g_rec.NumberParameters = 3;
}
void seh_set_exception_info(int i, unsigned long v)
{
    if (i >= 0 && i < EXCEPTION_MAXIMUM_PARAMETERS)
        g_rec.ExceptionInformation[i] = v;
}
void seh_set_filter_result(int id, int r)
{
    if (id >= 0 && id < SEH_MAX_SCOPES)
        g_filter_result[id] = r;
}
void seh_probe_nested(int flags) { g_probe_nested_flags = flags; }

/* `cxx_frame_handler`'s rethrow arm copies `*msvcrt_get_thread_data()->exc_record` over the
 * record it was handed.  With a NULL `exc_record` that arm is a null dereference on BOTH
 * sides, so a row that wants to reach it has to furnish one — and the copied-in record is
 * given a distinguishable code so the row can tell the copy happened. */
static EXCEPTION_RECORD g_alt_rec;
static CONTEXT g_alt_ctx;

void seh_set_prev_exception(unsigned code, unsigned flags)
{
    memset(&g_alt_rec, 0, sizeof g_alt_rec);
    g_alt_rec.ExceptionCode = code;
    g_alt_rec.ExceptionFlags = flags;
    g_alt_rec.NumberParameters = 3;
    g_alt_rec.ExceptionInformation[0] = 0x11111111;
    g_alt_rec.ExceptionInformation[1] = 0x22222222;
    g_alt_rec.ExceptionInformation[2] = 0x33333333;
    g_td.exc_record = &g_alt_rec;
    g_td.ctx_record = &g_alt_ctx;
}

/* The C++ exception type the record's third parameter points at.  `custom_handler` is the
 * one field except_i386.c reads, and it dispatches straight through it — so a recording
 * function there grades a whole arm of `cxx_frame_handler` that would otherwise need a real
 * thrown object. */
static cxx_exception_type g_exc_type;

static DWORD custom_handler(EXCEPTION_RECORD *rec, void *frame, CONTEXT *ctx, void *dispatch,
                            const cxx_function_descr *descr, int nested_trylevel,
                            EXCEPTION_REGISTRATION_RECORD *nested_frame, DWORD unknown)
{
    (void)rec;
    (void)frame;
    (void)ctx;
    (void)dispatch;
    (void)descr;
    (void)unknown;
    ev_push(SEH_EV_CUSTOM, nested_trylevel, (unsigned)(unsigned long)nested_frame, 0);
    return 0x0c0c0c0cu;
}

/* mode 0: no type at all.  1: a type whose custom_handler is NULL.  2: one that dispatches. */
void seh_set_exc_type(int mode)
{
    memset(&g_exc_type, 0, sizeof g_exc_type);
    if (mode == 0)
    {
        g_rec.ExceptionInformation[2] = 0;
        return;
    }
    g_exc_type.custom_handler = (mode == 2) ? (void *)custom_handler : NULL;
    g_rec.ExceptionInformation[2] = (unsigned long)&g_exc_type;
}

/* `cxx_frame_handler` may OVERWRITE the record it was handed (the rethrow arm), so the row
 * reads it back rather than assuming it is unchanged. */
unsigned seh_rec_code(void) { return (unsigned)g_rec.ExceptionCode; }
unsigned seh_rec_flags(void) { return (unsigned)g_rec.ExceptionFlags; }
unsigned seh_rec_info(int i)
{
    return (i >= 0 && i < EXCEPTION_MAXIMUM_PARAMETERS) ? (unsigned)g_rec.ExceptionInformation[i]
                                                        : 0;
}

/* kind: 0 = no handler, 1 = HANDLERS[id] (returns), 2 = FINALLYS[id] (longjmps back). */
void seh_add_scope(int previous_try_level, int filter_id, int handler_kind, int handler_id)
{
    SCOPETABLE e;
    if (g_scopes >= SEH_MAX_SCOPES)
        return;
    e.previousTryLevel = previous_try_level;
    e.lpfnFilter = (filter_id >= 0 && filter_id < 8) ? FILTERS[filter_id] : NULL;
    if (handler_kind == 1 && handler_id >= 0 && handler_id < 8)
        e.lpfnHandler = HANDLERS[handler_id];
    else if (handler_kind == 2 && handler_id >= 0 && handler_id < 4)
        e.lpfnHandler = FINALLYS[handler_id];
    else
        e.lpfnHandler = NULL;
    g_v4.entries[g_scopes] = e;
    g_v3[g_scopes] = e;
    g_scopes++;
}

void seh_add_unwind(int prev, int handler_kind, int handler_id)
{
    int n = (int)g_descr.unwind_count;
    if (n >= SEH_MAX_SCOPES)
        return;
    g_unwind[n].prev = prev;
    if (handler_kind == 1 && handler_id >= 0 && handler_id < 8)
        g_unwind[n].handler = HANDLERS[handler_id];
    else
        g_unwind[n].handler = NULL;
    g_descr.unwind_count = (UINT)(n + 1);
}

void seh_add_tryblock(int start_level, int end_level, int catch_level)
{
    int n = (int)g_descr.tryblock_count;
    if (n >= SEH_MAX_SCOPES)
        return;
    g_tryblock[n].start_level = start_level;
    g_tryblock[n].end_level = end_level;
    g_tryblock[n].catch_level = catch_level;
    g_tryblock[n].catchblock_count = 0;
    g_tryblock[n].catchblock = NULL;
    g_descr.tryblock_count = (UINT)(n + 1);
}

void seh_set_descr(unsigned magic, unsigned bbt_flags, unsigned flags)
{
    g_descr.magic = magic;
    g_descr.bbt_flags = bbt_flags;
    g_descr.flags = flags;
    g_descr.unwind_table = g_unwind;
    g_descr.tryblock = g_tryblock;
    g_descr.ipmap_count = 0;
    g_descr.ipmap = NULL;
    g_descr.expect_list = NULL;
}

/* ── snapshot accessors ───────────────────────────────────────────────── */

int seh_frame_trylevel(void) { return g_blk.frame.trylevel; }
int seh_cxx_trylevel(void) { return g_cxx.frame.trylevel; }
unsigned seh_frame_scratch(void) { return g_blk.scratch; }
unsigned seh_cxx_scratch(void) { return g_cxx.scratch; }
unsigned seh_dispatcher(void) { return (unsigned)(unsigned long)g_dispatcher; }
int seh_left_by_jump(void) { return g_left_by_jump; }
int seh_processing_throw(void) { return g_td.processing_throw; }

/* The frame and scope table live at fixed addresses shared by both sides, so a pointer the
 * reference stored is directly comparable with the one the subject stored. */
unsigned seh_frame_addr(void) { return (unsigned)(unsigned long)&g_blk.frame; }
unsigned seh_cxx_frame_addr(void) { return (unsigned)(unsigned long)&g_cxx.frame; }
unsigned seh_ebp_addr(void) { return (unsigned)(unsigned long)&g_blk.frame._ebp; }

/* ── the watchdog ─────────────────────────────────────────────────────────
 *
 * Every graded body is a WALK over a caller-supplied chain, and a chain that never reaches
 * its sentinel is an infinite loop with no call in it — no callback fires, no memory is
 * touched, nothing to observe.  Both sides would spin identically and the run would look
 * merely slow, which is the worst thing a gate can do: `vibeos-cli difftest` has a timeout,
 * but it would report the whole harness as hung rather than the row.
 *
 * So each driver arms an alarm and disarms it on return.  A row that trips it names itself.
 */
static volatile int g_alarm_armed;

void seh_row_label(const char *s)
{
    unsigned i = 0;
    while (s && s[i] && i < sizeof g_current_row - 1)
    {
        g_current_row[i] = s[i];
        i++;
    }
    g_current_row[i] = 0;
}

static void watchdog(int sig)
{
    /* write(2) rather than fprintf: this runs in a signal handler, and on the fault path the
     * process is already in an unknown state. */
    static const char hang[] =
        "\nmsvcrt-seh-difftest: a scenario did not terminate. The row below built a scope or\n"
        "unwind chain that never reaches its sentinel, so the walk spins with nothing to\n"
        "observe. Fix the row's chain; do not raise the timeout.\nrow: ";
    static const char fault[] =
        "\nmsvcrt-seh-difftest: a scenario FAULTED. The row below walked off a structure the\n"
        "harness built — a trylevel outside its table, or a pointer the scenario never set.\n"
        "Naming the row here is the whole point: a bare SIGSEGV with no output is the one\n"
        "failure mode that tells you nothing.\nrow: ";
    const char *head = (sig == SIGALRM) ? hang : fault;
    unsigned n = 0;
    while (head[n])
        n++;
    ssize_t r = write(2, head, n);
    r = write(2, g_current_row, strlen(g_current_row));
    r = write(2, "\n", 1);
    (void)r;
    _Exit(sig == SIGALRM ? 72 : 73);
}

static void arm(void)
{
    if (!g_alarm_armed)
    {
        signal(SIGALRM, watchdog);
        signal(SIGSEGV, watchdog);
        signal(SIGBUS, watchdog);
        g_alarm_armed = 1;
    }
    alarm(5);
}

static void disarm(void) { alarm(0); }

/* ── the drivers ──────────────────────────────────────────────────────── */

#define SEH_NORETURN_SENTINEL (-424242)

static void arm_v4_frame(void)
{
    /* The v4 scope table pointer in the frame is XOR-masked with the cookie; the C recovers
     * it through `get_scopetable_v4`, so this is what makes that helper's arithmetic the
     * thing under test rather than an identity. */
    g_blk.frame.scopetable = (SCOPETABLE *)((unsigned long)&g_v4 ^ g_cookie);
    g_blk.frame.prev = NULL;
    g_blk.frame.handler = NULL;
    g_blk.frame.xpointers = NULL;
}

static void arm_v3_frame(void)
{
    g_blk.frame.scopetable = g_v3;
    g_blk.frame.prev = NULL;
    g_blk.frame.handler = NULL;
    g_blk.frame.xpointers = NULL;
}

int seh_run_handler4_common(int side)
{
    arm();
    int r;
    arm_v4_frame();
    g_trylevel_ptr = &g_blk.frame.trylevel;
    g_dispatcher = NULL;
    if (setjmp(g_recover) != 0)
    {
        disarm();
        return SEH_NORETURN_SENTINEL;
    }
    r = side ? ref__except_handler4_common(&g_cookie, NULL, &g_rec, &g_blk.frame, &g_ctx,
                                           &g_dispatcher)
             : _except_handler4_common(&g_cookie, NULL, &g_rec, &g_blk.frame, &g_ctx,
                                       &g_dispatcher);
    disarm();
    return r;
}

int seh_run_local_unwind4(int side, int trylevel)
{
    arm();
    arm_v4_frame();
    g_trylevel_ptr = &g_blk.frame.trylevel;
    if (setjmp(g_recover) != 0)
    {
        disarm();
        return SEH_NORETURN_SENTINEL;
    }
    if (side)
        ref__local_unwind4(&g_cookie, &g_blk.frame, trylevel);
    else
        _local_unwind4(&g_cookie, &g_blk.frame, trylevel);
    disarm();
    return 0;
}

int seh_run_seh_longjmp_unwind(int side, int trylevel)
{
    arm();
    arm_v3_frame();
    g_trylevel_ptr = &g_blk.frame.trylevel;
    g_jmp.Registration = (unsigned long)&g_blk.frame;
    g_jmp.TryLevel = (unsigned long)trylevel;
    g_jmp.Ebp = (unsigned long)&g_blk.frame._ebp;
    if (setjmp(g_recover) != 0)
    {
        disarm();
        return SEH_NORETURN_SENTINEL;
    }
    if (side)
        ref__seh_longjmp_unwind(&g_jmp);
    else
        _seh_longjmp_unwind(&g_jmp);
    disarm();
    return 0;
}

int seh_run_seh_longjmp_unwind4(int side, int trylevel)
{
    arm();
    arm_v4_frame();
    g_trylevel_ptr = &g_blk.frame.trylevel;
    g_jmp.Registration = (unsigned long)&g_blk.frame;
    g_jmp.TryLevel = (unsigned long)trylevel;
    g_jmp.Ebp = (unsigned long)&g_blk.frame._ebp;
    g_jmp.Cookie = g_cookie;
    if (setjmp(g_recover) != 0)
    {
        disarm();
        return SEH_NORETURN_SENTINEL;
    }
    if (side)
        ref__seh_longjmp_unwind4(&g_jmp);
    else
        _seh_longjmp_unwind4(&g_jmp);
    disarm();
    return 0;
}

int seh_run_cxx_longjmp_unwind(int side, int trylevel)
{
    arm();
    g_trylevel_ptr = &g_cxx.frame.trylevel;
    g_jmp.Registration = (unsigned long)&g_cxx.frame;
    g_jmp.TryLevel = (unsigned long)trylevel;
    g_jmp.UnwindData[0] = (unsigned long)&g_descr;
    if (setjmp(g_recover) != 0)
    {
        disarm();
        return SEH_NORETURN_SENTINEL;
    }
    if (side)
        ref___CxxLongjmpUnwind(&g_jmp);
    else
        __CxxLongjmpUnwind(&g_jmp);
    disarm();
    return 0;
}

int seh_run_cxx_frame_handler(int side)
{
    arm();
    DWORD r;
    g_trylevel_ptr = &g_cxx.frame.trylevel;
    g_dispatcher = NULL;
    if (setjmp(g_recover) != 0)
    {
        disarm();
        return SEH_NORETURN_SENTINEL;
    }
    r = side ? ref_cxx_frame_handler(&g_rec, &g_cxx.frame, &g_ctx, &g_dispatcher, &g_descr, NULL)
             : cxx_frame_handler(&g_rec, &g_cxx.frame, &g_ctx, &g_dispatcher, &g_descr, NULL);
    disarm();
    return (int)r;
}

/* ── handle_fpieee_flt ────────────────────────────────────────────────── */

/* `handle_fpieee_flt` DEREFERENCES ContextFlags-adjacent fields as addresses:
 * `*(WORD *)ctx->ErrorOffset` is the faulting opcode and `*(double *)ctx->DataOffset` the
 * second operand.  So those two fields have to point at real memory, and the row sets what
 * is IN it rather than the field itself. */
static unsigned char g_opcode[8];
static double g_operand2;
static _FPIEEE_RECORD g_seen;
static int g_seen_calls;
static int g_fp_handler_result;

static int fpieee_handler(_FPIEEE_RECORD *rec)
{
    g_seen_calls++;
    memcpy(&g_seen, rec, sizeof g_seen);
    /* The C copies `rec->Result.Value.Fp80Value` back into the register area when the
     * handler asks to continue, so write something recognisable there first. */
    memset(&rec->Result.Value.Fp80Value, 0x5a, sizeof(_FP80));
    return g_fp_handler_result;
}

void seh_fp_setup(unsigned control, unsigned status, unsigned opcode, double operand2,
                  int handler_result)
{
    memset(&g_ctx, 0, sizeof g_ctx);
    memset(g_opcode, 0, sizeof g_opcode);
    memset(&g_seen, 0, sizeof g_seen);
    g_seen_calls = 0;
    g_opcode[0] = (unsigned char)(opcode & 0xff);
    g_opcode[1] = (unsigned char)((opcode >> 8) & 0xff);
    g_operand2 = operand2;
    g_fp_handler_result = handler_result;
    g_ctx.FloatSave.ControlWord = control;
    g_ctx.FloatSave.StatusWord = status;
    g_ctx.FloatSave.ErrorOffset = (DWORD)(unsigned long)g_opcode;
    g_ctx.FloatSave.DataOffset = (DWORD)(unsigned long)&g_operand2;
    for (int i = 0; i < SIZE_OF_80387_REGISTERS; i++)
        g_ctx.FloatSave.RegisterArea[i] = (BYTE)(0xa0 + i);
}

int seh_run_fpieee(int side, unsigned long code)
{
    arm();
    EXCEPTION_POINTERS ep;
    ep.ExceptionRecord = &g_rec;
    ep.ContextRecord = &g_ctx;
    g_seen_calls = 0;
    memset(&g_seen, 0, sizeof g_seen);
    int r = side ? ref_handle_fpieee_flt(code, &ep, fpieee_handler)
                 : handle_fpieee_flt(code, &ep, fpieee_handler);
    disarm();
    return r;
}

int seh_fp_calls(void) { return g_seen_calls; }
unsigned char seh_fp_seen_byte(int i)
{
    return (i >= 0 && i < (int)sizeof g_seen) ? ((unsigned char *)&g_seen)[i] : 0;
}
int seh_fp_seen_size(void) { return (int)sizeof g_seen; }
unsigned char seh_fp_regarea_byte(int i)
{
    return (i >= 0 && i < SIZE_OF_80387_REGISTERS) ? g_ctx.FloatSave.RegisterArea[i] : 0;
}

/*
 * DbgPrint — what crt-shared-rust's ERR/FIXME macros call.
 *
 * except.c's SEH diagnostics ("invalid trylevel %d", "frame not found, native crashes
 * in this case") were empty macros until 2026-09-05 and print for real now, so the host
 * link needs the symbol. stderr stands in for the serial port; deliberately not silent,
 * since an SEH ERR firing mid-difftest is precisely a finding.
 */
#include <stdarg.h>

int DbgPrint(const char *format, ...)
{
    va_list ap;
    int n;

    va_start(ap, format);
    n = vfprintf(stderr, format, ap);
    va_end(ap);
    return n;
}
