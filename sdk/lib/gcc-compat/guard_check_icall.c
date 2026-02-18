/*
 * Stubs for Control Flow Guard (CFG) function pointers.
 * The llvm-mingw libc++/libc++abi is compiled with CFG enabled and references
 * these symbols. Since ReactOS doesn't use CFG, provide no-op stubs.
 */

/* __guard_check_icall_fptr: validates an indirect call target (no-op without CFG) */
static void __guard_check_icall_nop(void)
{
    /* No-op: CFG is not used */
}

void (*__guard_check_icall_fptr)(void) = __guard_check_icall_nop;

/*
 * __guard_dispatch_icall_fptr: dispatches an indirect call through CFG.
 * Without CFG, this should just call the target.
 * On amd64, the target address is in RAX.
 * On i386, the target address is in ECX.
 */
#if defined(_M_AMD64) || defined(__x86_64__)

__attribute__((naked))
void __guard_dispatch_icall_nop(void)
{
    __asm__("jmp *%rax");
}

#elif defined(_M_IX86) || defined(__i386__)

__attribute__((naked))
void __guard_dispatch_icall_nop(void)
{
    __asm__("jmp *%ecx");
}

#elif defined(_M_ARM64) || defined(__aarch64__)

__attribute__((naked))
void __guard_dispatch_icall_nop(void)
{
    __asm__("br x15");
}

#else

void __guard_dispatch_icall_nop(void)
{
    /* No-op fallback */
}

#endif

void (*__guard_dispatch_icall_fptr)(void) = __guard_dispatch_icall_nop;
