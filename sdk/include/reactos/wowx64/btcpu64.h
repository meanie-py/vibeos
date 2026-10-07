/*
 * btcpu64.h — the vibeos BTCpu64 contract: x86-64 guests on an arm64 host.
 *
 * Fork B of docs/engineering/wow64-x64-guest-design.md. Two DLLs meet here:
 *
 *   wowx64.dll       OS side (sdk/lib/rust/wowx64-rust). ntdll diverts an x64
 *                    process to WowX64LdrpInitialize; it loads the CPU layer,
 *                    owns the guest's stacks, and services guest syscalls.
 *   libwowx64fex.dll CPU side (tools/fex/wowx64fex). Links the vendored FEXCore
 *                    in 64-bit mode; exports the BTCpu64* entry points below.
 *
 * It is deliberately BTCpu-SHAPED (the same verbs as the 32-bit WoW64 CPU
 * layer, so the OS side is a familiar shape) but it is OUR contract, not
 * Microsoft's: there is no CHPE / ARM64EC hybrid-PE ABI anywhere in it. A guest
 * is a plain x86-64 process running the amd64 system DLLs fully emulated.
 *
 * What makes 64-on-64 cheap, and is load-bearing for the syscall path: amd64 and
 * arm64 are both LLP64 AND share one system-service table (ntoskrnl/sysfuncs.lst
 * is arch-independent), so a guest `syscall` is forwarded by REGISTER SHUFFLE
 * alone — the guest's service number IS the native one, and every argument is
 * already the right width. No per-service thunk exists or is needed.
 *
 * The CPU side calls BACK into the OS side through WOWX64_HOST_SERVICES, passed
 * to BTCpu64ProcessInit, rather than importing wowx64.dll. That keeps the
 * plugin's import table at ntdll alone and its build free of our import libs.
 *
 * The Rust side mirrors these types by hand (wowx64-rust/src/contract.rs) with
 * size asserts; change both together and bump WOWX64_CONTRACT_VERSION.
 */
#pragma once

#include <stdint.h>

#define WOWX64_CONTRACT_VERSION 3

/* Register arguments of a guest system service: r10 (the stub's copy of rcx),
 * rdx, r8, r9. The rest are on the guest stack at rsp+0x28 and are read by the
 * OS side, which knows how many each service takes. */
#define WOWX64_REGISTER_ARGS 4

/* x86-64 CONTEXT, laid out exactly as on amd64 Windows (0x4D0 bytes). The
 * plugin is compiled for arm64, where CONTEXT is the ARM64 one, so the guest's
 * shape needs a name of its own. */
#define WOWX64_CONTEXT_AMD64          0x00100000u
#define WOWX64_CONTEXT_CONTROL        (WOWX64_CONTEXT_AMD64 | 0x01u)
#define WOWX64_CONTEXT_INTEGER        (WOWX64_CONTEXT_AMD64 | 0x02u)
#define WOWX64_CONTEXT_SEGMENTS       (WOWX64_CONTEXT_AMD64 | 0x04u)
#define WOWX64_CONTEXT_FLOATING_POINT (WOWX64_CONTEXT_AMD64 | 0x08u)
#define WOWX64_CONTEXT_FULL \
    (WOWX64_CONTEXT_CONTROL | WOWX64_CONTEXT_INTEGER | WOWX64_CONTEXT_FLOATING_POINT)
#define WOWX64_CONTEXT_ALL (WOWX64_CONTEXT_FULL | WOWX64_CONTEXT_SEGMENTS)

/* Flat 64-bit user selectors, as on amd64 Windows. */
#define WOWX64_SEG_CS 0x33
#define WOWX64_SEG_DS 0x2B

typedef struct _WOWX64_M128A {
    uint64_t Low;
    int64_t High;
} WOWX64_M128A;

typedef struct _WOWX64_XSAVE_FORMAT {
    uint16_t ControlWord;
    uint16_t StatusWord;
    uint8_t TagWord;
    uint8_t Reserved1;
    uint16_t ErrorOpcode;
    uint32_t ErrorOffset;
    uint16_t ErrorSelector;
    uint16_t Reserved2;
    uint32_t DataOffset;
    uint16_t DataSelector;
    uint16_t Reserved3;
    uint32_t MxCsr;
    uint32_t MxCsr_Mask;
    WOWX64_M128A FloatRegisters[8];
    WOWX64_M128A XmmRegisters[16];
    uint8_t Reserved4[96];
} WOWX64_XSAVE_FORMAT;

typedef struct __attribute__((aligned(16))) _WOWX64_CONTEXT {
    uint64_t P1Home, P2Home, P3Home, P4Home, P5Home, P6Home;
    uint32_t ContextFlags;
    uint32_t MxCsr;
    uint16_t SegCs, SegDs, SegEs, SegFs, SegGs, SegSs;
    uint32_t EFlags;
    uint64_t Dr0, Dr1, Dr2, Dr3, Dr6, Dr7;
    uint64_t Rax, Rcx, Rdx, Rbx, Rsp, Rbp, Rsi, Rdi;
    uint64_t R8, R9, R10, R11, R12, R13, R14, R15;
    uint64_t Rip;
    WOWX64_XSAVE_FORMAT FltSave;
    WOWX64_M128A VectorRegister[26];
    uint64_t VectorControl;
    uint64_t DebugControl;
    uint64_t LastBranchToRip;
    uint64_t LastBranchFromRip;
    uint64_t LastExceptionToRip;
    uint64_t LastExceptionFromRip;
} WOWX64_CONTEXT;

_Static_assert(sizeof(WOWX64_XSAVE_FORMAT) == 0x200, "XSAVE_FORMAT layout");
_Static_assert(sizeof(WOWX64_CONTEXT) == 0x4D0, "x64 CONTEXT layout");
_Static_assert(__builtin_offsetof(WOWX64_CONTEXT, Rax) == 0x78, "x64 CONTEXT layout");
_Static_assert(__builtin_offsetof(WOWX64_CONTEXT, Rip) == 0xF8, "x64 CONTEXT layout");
_Static_assert(__builtin_offsetof(WOWX64_CONTEXT, FltSave) == 0x100, "x64 CONTEXT layout");

/* OS-side services handed to the CPU layer. */
typedef struct _WOWX64_HOST_SERVICES {
    uint32_t Size;    /* sizeof(WOWX64_HOST_SERVICES) */
    uint32_t Version; /* WOWX64_CONTRACT_VERSION */
    /* Run guest system service `ServiceNumber`. RegisterArgs holds the
     * WOWX64_REGISTER_ARGS register arguments in guest order; GuestRsp is the
     * guest rsp AT the syscall (the stub's return address on top), from which
     * the OS side reads stack arguments 5..N at rsp+0x28 -- exactly N, per the
     * service's own argument count, as the amd64 kernel does. (Version 1 had
     * the CPU layer copy a fixed 11 stack words, which read past the top of a
     * guest stack whenever a syscall was made within 0x80 bytes of it -- a
     * fresh thread's first NtContinue.) The return value goes to guest RAX. */
    uint64_t (*SystemService)(uint32_t ServiceNumber, const uint64_t *RegisterArgs, uint64_t GuestRsp);
} WOWX64_HOST_SERVICES;

/* ---- CPU-layer exports (libwowx64fex.dll) --------------------------------
 *
 * Thread handles: M1 supports only the calling thread (NtCurrentThread() or a
 * handle to it); other threads return STATUS_NOT_IMPLEMENTED, loudly.
 *
 *   int32_t BTCpu64ProcessInit(const WOWX64_HOST_SERVICES *Host);
 *   int32_t BTCpu64ThreadInit(void);
 *   void    BTCpu64ThreadTerm(void *Thread, int32_t ExitStatus);   -- (v3)
 *       Frees a thread's CPU-layer state. Called for the calling thread just
 *       before its own NtTerminateThread (nothing of the JIT runs after), or
 *       for another thread of the process once it has DIED. Never for a live
 *       other thread: freeing state a running JIT is using is not cleanup.
 *   int32_t BTCpu64GetContext(void *Thread, void *Process, void *Reserved, WOWX64_CONTEXT *Context);
 *   int32_t BTCpu64SetContext(void *Thread, void *Process, void *Reserved, const WOWX64_CONTEXT *Context);
 *   void    BTCpu64Simulate(void);                 -- runs the guest; never returns
 *   int32_t BTCpu64ResetToConsistentState(EXCEPTION_POINTERS *Ptrs);
 *       First refusal on EVERY native exception (the OS side calls it from a
 *       VEH). FEX-internal faults (overcommit, SMC, unaligned atomics) are
 *       resolved and NtContinue'd and never return. A genuine GUEST fault comes
 *       back with the record restated in guest terms (ExceptionAddress = guest
 *       RIP), the guest state saved for BTCpu64GetContext, and the host context
 *       replaced by the one captured at BTCpu64Simulate entry. A host fault
 *       comes back untouched.
 *   int32_t BTCpu64NotifyMapViewOfSection(void *Unk1, void *Address, void *Unk2,
 *                                         size_t Size, uint32_t AllocType, uint32_t Prot);
 *   void    BTCpu64NotifyUnmapViewOfSection(void *Address, int After, int32_t Status);
 *   void    BTCpu64NotifyMemoryAlloc(void *Address, size_t Size, uint32_t Type, uint32_t Prot, int After, int32_t Status);
 *   void    BTCpu64NotifyMemoryProtect(void *Address, size_t Size, uint32_t NewProt, int After, int32_t Status);
 *   void    BTCpu64NotifyMemoryFree(void *Address, size_t Size, uint32_t FreeType, int After, int32_t Status);
 *   void    BTCpu64FlushInstructionCache2(const void *Address, size_t Size);
 *   void    BTCpu64NotifyMemoryDirty(void *Address, size_t Size);
 *   void    BTCpu64NotifyReadFile(HANDLE File, void *Address, size_t Size, int After, int32_t Status);
 */
