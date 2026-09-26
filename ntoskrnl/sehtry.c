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
    __try {
        Body(BodyContext);
    } __except (Filter ? Filter(FilterContext, _exception_info()) : 1) {
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

/* EXCEPTION_REGISTRATION_RECORD first, then our state. */
typedef struct _SEH_FRAME {
    struct _SEH_FRAME *Next;
    void *Handler;
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

    Frame.Handler = (void *)SehExceptHandler;
    Frame.Filter = Filter;
    Frame.FilterContext = FilterContext;
    Frame.Code = 0;
    Frame.Next = SehGetList();
    if (__builtin_setjmp((void **)Frame.JmpBuf)) {
        /* The handler already unlinked Frame. */
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
