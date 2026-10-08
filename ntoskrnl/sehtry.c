/*
 * sehtry.c — real __try/__except and __try/__finally for RUST kernel code.
 *
 * Under USE_DUMMY_PSEH every _SEH2_EXCEPT arm is dead, and the Rust translations
 * inherited that: a raise-by-design (cdfs/fastfat's CdRaiseStatus/FatRaiseStatus,
 * STATUS_CANT_WAIT, STATUS_VERIFY_REQUIRED, Cc/Mm in-page errors) finds no handler
 * and bugchecks 0x1E. docs/engineering/dummy-pseh-verdict.md §C concluded "Rust
 * cannot consume native SEH" — but it can, one frame down: these trampolines call a
 * Rust body through a function pointer INSIDE the protected region, which is the
 * call-shaped case native SEH covers. The Rust frames between the raise and the
 * trampoline carry .pdata (requires-uwtable, field manual §9.42), so the unwinder
 * walks straight through them. The Rust side is ntkrnl_sys::seh.
 *
 *   BOOLEAN RtlSehTryExcept(Body, BodyCtx, Filter, FilterCtx, &Code)
 *       Runs Body(BodyCtx). If an exception reaches here and Filter returns
 *       EXCEPTION_EXECUTE_HANDLER (1; NULL Filter means always), unwinds to here,
 *       stores the code, returns TRUE. CONTINUE_SEARCH (0) and
 *       CONTINUE_EXECUTION (-1) have their usual meanings.
 *   VOID RtlSehTryFinally(Body, BodyCtx, Finally, FinallyCtx)
 *       Runs Body(BodyCtx), then Finally(FinallyCtx, Abnormal). Finally also runs
 *       while an exception unwinds THROUGH this frame, with the frames above it
 *       still intact, which is what a C __finally is for: releasing the locks
 *       the body took.
 *
 * amd64/arm64: native clang SEH (-fms-extensions via c_shim_flags), the recipe
 * seh_poc.c proves at every boot. i386: clang's 32-bit SEH lowering is MSVC-env
 * only, so the same contract is built by hand on the fs:[0] registration chain
 * that the i386 RtlDispatchException/RtlUnwind (rtl-except/src/i386) walk.
 *
 * Headerless on purpose, like seh_poc.c: pseh2.h's DUMMY macro layer must not be
 * able to turn __try back into a no-op.
 *
 * ntdll compiles this same file with -DSEHTRY_USER_MODE and exports both names, for
 * the user-mode Rust that dropped the C's RpcTryExcept (ntdll_sys::seh). User mode
 * skips the interrupt-mask restore: `msr daifclr` traps at EL0 and `sti` faults at
 * ring 3, and neither is needed there because user code always runs unmasked.
 */

typedef void (__cdecl *SEH_BODY)(void *Context);
typedef long (__cdecl *SEH_FILTER)(void *Context, void *ExceptionPointers);
typedef void (__cdecl *SEH_FINALLY)(void *Context, unsigned char Abnormal);

#if defined(__x86_64__) || defined(__aarch64__)

__attribute__((noinline))
unsigned char __cdecl
RtlSehTryExcept(SEH_BODY Body, void *BodyContext, SEH_FILTER Filter, void *FilterContext,
                long *ExceptionCode)
{
#if defined(__aarch64__) && !defined(SEHTRY_USER_MODE)
    /* arm64 dispatches a kernel fault with IRQs masked (the sync-exception path keeps
     * DAIF as the exception entry left it), and the except block resumes on a CONTEXT
     * captured during that dispatch, so it would carry on masked: a synchronously
     * completed IRP's completion APC then cannot run before IoCallDriver returns (the
     * amd64 fastfat stack-event corruption). Record I at try entry and put it back,
     * as the i386 path below does with IF. amd64 fixes this at the trap instead. */
    volatile unsigned long Daif;
    __asm__ __volatile__("mrs %0, daif" : "=r"(Daif));
#endif
    __try {
        Body(BodyContext);
    } __except (Filter ? Filter(FilterContext, _exception_info()) : 1) {
#if defined(__aarch64__) && !defined(SEHTRY_USER_MODE)
        if (!(Daif & 0x80 /* PSTATE.I */))
            __asm__ __volatile__("msr daifclr, #2" ::: "memory");
#endif
        if (ExceptionCode)
            *ExceptionCode = _exception_code();
        return 1;
    }
    return 0;
}

__attribute__((noinline))
void __cdecl
RtlSehTryFinally(SEH_BODY Body, void *BodyContext, SEH_FINALLY Finally, void *FinallyContext)
{
    __try {
        Body(BodyContext);
    } __finally {
        Finally(FinallyContext, (unsigned char)_abnormal_termination());
    }
}

#elif defined(__i386__)

typedef struct _SEH_RECORD {
    unsigned long ExceptionCode;
    unsigned long ExceptionFlags;
} SEH_RECORD;

typedef struct _SEH_POINTERS {
    void *ExceptionRecord;
    void *ContextRecord;
} SEH_POINTERS;

/* EXCEPTION_REGISTRATION_RECORD first, then MSVC's EH3 ScopeTable/TryLevel, then our
 * state. The EH3 pair is for code that reads the CURRENT frame as an MSVC one: the
 * CRT's _setjmp/_setjmp3 store fs:[0]->TryLevel ([reg+12]), and kernel32's top-level
 * frame (BaseProcessStartup/BaseThreadStartup) is what a Win32 main() runs under.
 * TryLevel 0 is the truth -- the body runs inside one __try -- and a NULL ScopeTable
 * with equal levels makes _local_unwind2 a no-op. Without it [reg+12] was JmpBuf[1],
 * a code address. */
typedef struct _SEH_FRAME {
    struct _SEH_FRAME *Next;
    void *Handler;
    void *ScopeTable;
    unsigned long TryLevel;
    void *JmpBuf[5]; /* __builtin_setjmp's buffer: fp, ip, sp + 2 spare */
    SEH_FILTER Filter;
    void *FilterContext;
    SEH_FINALLY Finally;
    void *FinallyContext;
    long Code;
} SEH_FRAME;

#define SEH_UNWIND_FLAGS 0x6 /* EXCEPTION_UNWINDING | EXCEPTION_EXIT_UNWIND */
#define DISPOSITION_CONTINUE_EXECUTION 0
#define DISPOSITION_CONTINUE_SEARCH 1

static inline SEH_FRAME *SehGetList(void)
{
    SEH_FRAME *Frame;
    __asm__ __volatile__("movl %%fs:0, %0" : "=r"(Frame));
    return Frame;
}

static inline void SehSetList(SEH_FRAME *Frame)
{
    __asm__ __volatile__("movl %0, %%fs:0" : : "r"(Frame) : "memory");
}

/* RtlUnwind to Frame. It returns through ZwContinue on a CONTEXT it captured inside
 * itself, so ebx/esi/edi come back holding RtlUnwind's values, not ours; that is
 * why MSVC's __global_unwind2 saves them around the call, and so does this. */
static void SehGlobalUnwind(SEH_FRAME *Frame, SEH_RECORD *Record)
{
    __asm__ __volatile__(
        "pushl %%ebp\n\t"
        "pushl %%ebx\n\t"
        "pushl %%esi\n\t"
        "pushl %%edi\n\t"
        "pushl $0\n\t"
        "pushl %1\n\t"
        "pushl $0\n\t"
        "pushl %0\n\t"
        "call _RtlUnwind@16\n\t"
        "popl %%edi\n\t"
        "popl %%esi\n\t"
        "popl %%ebx\n\t"
        "popl %%ebp\n\t"
        :
        : "r"(Frame), "r"(Record)
        : "eax", "ecx", "edx", "memory", "cc");
}

static int __cdecl
SehExceptHandler(SEH_RECORD *Record, SEH_FRAME *Frame, void *Context, void *Dispatcher)
{
    SEH_POINTERS Pointers;
    long Verdict;

    (void)Dispatcher;
    if (Record->ExceptionFlags & SEH_UNWIND_FLAGS)
        return DISPOSITION_CONTINUE_SEARCH;

    Pointers.ExceptionRecord = Record;
    Pointers.ContextRecord = Context;
    Verdict = Frame->Filter ? Frame->Filter(Frame->FilterContext, &Pointers) : 1;
    if (Verdict < 0)
        return DISPOSITION_CONTINUE_EXECUTION;
    if (Verdict == 0)
        return DISPOSITION_CONTINUE_SEARCH;

    /* Run every newer frame's unwind handler (their finallys) while the stack is
     * still intact; RtlUnwind stops at Frame and leaves fs:[0] pointing at it. */
    Frame->Code = (long)Record->ExceptionCode;
    SehGlobalUnwind(Frame, Record);
    SehSetList(Frame->Next);
    __builtin_longjmp(Frame->JmpBuf, 1);
}

static int __cdecl
SehFinallyHandler(SEH_RECORD *Record, SEH_FRAME *Frame, void *Context, void *Dispatcher)
{
    (void)Context;
    (void)Dispatcher;
    if (Record->ExceptionFlags & SEH_UNWIND_FLAGS)
        Frame->Finally(Frame->FinallyContext, 1);
    return DISPOSITION_CONTINUE_SEARCH;
}

__attribute__((noinline))
unsigned char __cdecl
RtlSehTryExcept(SEH_BODY Body, void *BodyContext, SEH_FILTER Filter, void *FilterContext,
                long *ExceptionCode)
{
    volatile SEH_FRAME Frame;
    volatile unsigned long EFlags;

    /* A kernel-mode int3 is dispatched with interrupts still disabled (KiDebugHandler
     * only re-enables them for user traps), and __builtin_longjmp restores no flags, so
     * the catch below would resume with IF=0 and the next page fault bugchecks
     * IRQL_NOT_LESS_OR_EQUAL with IRQL -1. amd64 gets EFlags back from the unwound
     * context; here the try records IF and the catch puts it back. */
    __asm__ __volatile__("pushfl\n\tpopl %0" : "=r"(EFlags));
    Frame.Handler = (void *)SehExceptHandler;
    Frame.ScopeTable = 0;
    Frame.TryLevel = 0;
    Frame.Filter = Filter;
    Frame.FilterContext = FilterContext;
    Frame.Code = 0;
    Frame.Next = SehGetList();
    if (__builtin_setjmp((void **)Frame.JmpBuf)) {
        /* The handler already unlinked Frame. */
#if !defined(SEHTRY_USER_MODE)
        if (EFlags & 0x200 /* EFLAGS_INTERRUPT_MASK */)
            __asm__ __volatile__("sti" ::: "memory");
#endif
        if (ExceptionCode)
            *ExceptionCode = Frame.Code;
        return 1;
    }
    SehSetList((SEH_FRAME *)&Frame);
    Body(BodyContext);
    SehSetList(Frame.Next);
    return 0;
}

__attribute__((noinline))
void __cdecl
RtlSehTryFinally(SEH_BODY Body, void *BodyContext, SEH_FINALLY Finally, void *FinallyContext)
{
    volatile SEH_FRAME Frame;

    Frame.Handler = (void *)SehFinallyHandler;
    Frame.ScopeTable = 0;
    Frame.TryLevel = 0;
    Frame.Finally = Finally;
    Frame.FinallyContext = FinallyContext;
    Frame.Next = SehGetList();
    SehSetList((SEH_FRAME *)&Frame);
    Body(BodyContext);
    SehSetList(Frame.Next);
    Finally(FinallyContext, 0);
}

#else
#error "sehtry.c: unsupported architecture"
#endif
