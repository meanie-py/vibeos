/*
 * Stub for __guard_check_icall_fptr - Control Flow Guard check function pointer.
 * The llvm-mingw libc++abi.a is compiled with CFG enabled and references this symbol.
 * Since ReactOS doesn't use CFG, provide a no-op stub.
 */

static void __guard_check_icall_nop(void)
{
    /* No-op: CFG is not used */
}

void (*__guard_check_icall_fptr)(void) = __guard_check_icall_nop;
