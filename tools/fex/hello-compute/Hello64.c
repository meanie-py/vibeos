/*
 * hello64.exe — the first x86-64 guest of Fork B (BTCpu64: x86-64 guests on
 * arm64; docs/engineering/wow64-x64-guest-design.md). Milestone M1: a plain
 * amd64 PE enters FEX in 64-bit mode and makes a system call.
 *
 * No CRT and NO IMPORTS, on purpose: M1 is the CPU + syscall path alone, before
 * the guest has an amd64 ntdll of its own. So the syscalls are issued raw, by
 * number, exactly as an amd64 ntdll stub would (`mov r10, rcx; syscall`). The
 * numbers come from ntoskrnl/sysfuncs.lst at build time (-DSVC_*) — that list
 * is arch-independent, which is the property the whole register-shuffle syscall
 * design rests on, so baking them in here is checking it rather than assuming it.
 *
 * Every rung checks a VALUE, never mere survival, and a failure exits 0xBAD0+N
 * so the serial line names the rung. The same binary also runs NATIVELY on
 * amd64, where it must pass identically — a free oracle that the guest itself
 * is right, independent of the emulator.
 *
 *   1  long mode: a REX.W movabs through r15 round-trips a 64-bit constant (in
 *      32-bit mode the REX bytes decode as inc/dec and this cannot come out)
 *   2  GS is the TEB: gs:[0x30] (NtTib.Self) is non-null and reads back itself
 *   3  the PEB is shared: gs:[0x60]->ImageBaseAddress is this very image
 *   4  syscall round-trip: NtClose(NULL) == STATUS_INVALID_HANDLE
 *   then NtTerminateProcess(NtCurrentProcess(), 0x600D).
 */

#ifndef SVC_NTCLOSE
#error "build with -DSVC_NTCLOSE=<index in ntoskrnl/sysfuncs.lst> (tools/fex/build-hello-compute.sh)"
#endif
#ifndef SVC_NTTERMINATEPROCESS
#error "build with -DSVC_NTTERMINATEPROCESS=<index in ntoskrnl/sysfuncs.lst>"
#endif

typedef unsigned long long u64;

#define STATUS_INVALID_HANDLE 0xC0000008ull
#define PASS 0x600D
#define FAIL(n) (0xBAD0 + (n))

extern char __ImageBase[]; /* the linker's image-base symbol: where WE are mapped */

static inline u64 Syscall4(u64 Service, u64 A1, u64 A2, u64 A3, u64 A4)
{
    register u64 R10 __asm__("r10") = A1;
    register u64 R8 __asm__("r8") = A3;
    register u64 R9 __asm__("r9") = A4;
    u64 Ret;
    __asm__ volatile("syscall"
                     : "=a"(Ret), "+r"(R10), "+r"(R8), "+r"(R9)
                     : "a"(Service), "d"(A2)
                     : "rcx", "r11", "memory");
    return Ret;
}

static __attribute__((noreturn)) void Exit(u64 Code)
{
    Syscall4(SVC_NTTERMINATEPROCESS, (u64)-1 /* NtCurrentProcess() */, Code, 0, 0);
    for (;;)
        __asm__ volatile("hlt"); /* unreachable: terminate does not return */
}

void start(void)
{
    u64 V, Teb, Self, Peb, ImageBase;

    /* 1: long mode */
    __asm__ volatile("movabs $0x1122334455667788, %%r15\n\t"
                     "mov %%r15, %0"
                     : "=r"(V) : : "r15");
    if (V != 0x1122334455667788ull)
        Exit(FAIL(1));

    /* 2: GS = TEB */
    __asm__ volatile("mov %%gs:0x30, %0" : "=r"(Teb));
    if (!Teb)
        Exit(FAIL(2));
    Self = *(volatile u64 *)(Teb + 0x30);
    if (Self != Teb)
        Exit(FAIL(2));

    /* 3: the PEB the native side sees is ours too */
    __asm__ volatile("mov %%gs:0x60, %0" : "=r"(Peb));
    ImageBase = Peb ? *(volatile u64 *)(Peb + 0x10) : 0;
    if (ImageBase != (u64)__ImageBase)
        Exit(FAIL(3));

    /* 4: a real system call, round-tripped. Compared as the 32-bit NTSTATUS it
     * is (amd64 code reads it from eax): the arm64 service returns it in w0,
     * and AAPCS64 leaves x0's upper half unspecified for a 32-bit result. */
    if ((unsigned)Syscall4(SVC_NTCLOSE, 0, 0, 0, 0) != (unsigned)STATUS_INVALID_HANDLE)
        Exit(FAIL(4));

    Exit(PASS);
}
