/*
 * hello32.exe — the i386 guest for WoW64 Phase C (docs/engineering/wow64-plan.md).
 * Built with the llvm-mingw i686 target, no CRT, no imports: a minimal valid
 * i386 PE the WoW64 bootstrap chain can launch on an arm64 host.
 *
 * Milestone 1: this entry never actually executes — wow64!Wow64LdrpInitialize
 * proves the kernel+loader chain and terminates the process before any guest
 * code runs. Milestone 2 will run this under the FEX CPU layer, at which point
 * the body becomes a real ntdll32 syscall exercise.
 */

/* Entry must not depend on any import: spin-free exit via a plain return.
 * If this ever runs natively on arm64 something is catastrophically wrong —
 * but it can't: the bytes below are i386 machine code. */
int start(void) /* cdecl: i386 symbol is _start, matching -Wl,-e,_start */
{
    return 0x51DE; /* sentinel exit code if the guest ever runs to completion */
}
