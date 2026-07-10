/*
 * PROJECT:     vibeos wow64.dll
 * PURPOSE:     Win10+-model WOW64 host layer — Phase A/B skeleton
 *              (docs/engineering/wow64-plan.md). This is OUR layer of the
 *              three-layer BTCpu architecture: the CPU layer below it is
 *              libwow64fex.dll (vendored FEX, contract-only), the kernel/
 *              loader layer above it is Phase C.
 *
 *              Phase A scope: exist, export the two symbols wow64fex imports,
 *              and make Wow64SystemServiceEx loudly report any syscall it is
 *              asked to thunk. The real 32→64 thunk table is Phase B.
 */

#define WIN32_NO_STATUS
#include <windef.h>
#include <winbase.h>
#include <stdint.h>

ULONG
WINAPI
DbgPrint(_In_z_ _Printf_format_string_ PCSTR Format, ...);

LONG
WINAPI
NtTerminateProcess(_In_ HANDLE ProcessHandle, _In_ LONG ExitStatus);

LONG
WINAPI
NtQueryInformationProcess(_In_ HANDLE ProcessHandle,
                          _In_ ULONG ProcessInformationClass,
                          _Out_ PVOID ProcessInformation,
                          _In_ ULONG ProcessInformationLength,
                          _Out_opt_ PULONG ReturnLength);

/* ntdll-only DLL: no kernel32 import (a WoW64 process must not pull 64-bit
 * kernel32 through us — Win10 keeps it out of wow64 processes too). */
#define WOW64_CURRENT_PROCESS ((HANDLE)(LONG_PTR)-1)

/* ---- ntdll surface we use (this DLL links ntdll only) ------------------- */

typedef struct _WOW64_UNICODE_STRING {
    USHORT Length;
    USHORT MaximumLength;
    PWSTR Buffer;
} WOW64_UNICODE_STRING;

typedef struct _WOW64_ANSI_STRING {
    USHORT Length;
    USHORT MaximumLength;
    PSTR Buffer;
} WOW64_ANSI_STRING;

typedef struct _WOW64_OBJECT_ATTRIBUTES {
    ULONG Length;
    HANDLE RootDirectory;
    WOW64_UNICODE_STRING *ObjectName;
    ULONG Attributes;
    PVOID SecurityDescriptor;
    PVOID SecurityQualityOfService;
} WOW64_OBJECT_ATTRIBUTES;

typedef struct _WOW64_IO_STATUS_BLOCK {
    union { LONG Status; PVOID Pointer; };
    ULONG_PTR Information;
} WOW64_IO_STATUS_BLOCK;

LONG WINAPI LdrLoadDll(PWSTR SearchPath, PULONG Flags,
                       WOW64_UNICODE_STRING *Name, PVOID *BaseAddress);
LONG WINAPI LdrGetDllHandle(PWSTR Path, PULONG Unused,
                            WOW64_UNICODE_STRING *Name, PVOID *BaseAddress);
LONG WINAPI LdrGetProcedureAddress(PVOID BaseAddress, WOW64_ANSI_STRING *Name,
                                   ULONG Ordinal, PVOID *Address);
VOID WINAPI RtlInitUnicodeString(WOW64_UNICODE_STRING *Dest, PCWSTR Src);
VOID WINAPI RtlInitAnsiString(WOW64_ANSI_STRING *Dest, PCSTR Src);
LONG WINAPI RtlSetEnvironmentVariable(PWSTR *Environment,
                                      WOW64_UNICODE_STRING *Name,
                                      WOW64_UNICODE_STRING *Value);
PVOID WINAPI RtlAddVectoredExceptionHandler(ULONG First,
                                            PVECTORED_EXCEPTION_HANDLER Handler);
LONG WINAPI NtAllocateVirtualMemory(HANDLE Process, PVOID *BaseAddress,
                                    ULONG_PTR ZeroBits, SIZE_T *RegionSize,
                                    ULONG AllocationType, ULONG Protect);
LONG WINAPI NtOpenFile(HANDLE *FileHandle, ACCESS_MASK DesiredAccess,
                       WOW64_OBJECT_ATTRIBUTES *ObjectAttributes,
                       WOW64_IO_STATUS_BLOCK *IoStatusBlock,
                       ULONG ShareAccess, ULONG OpenOptions);
LONG WINAPI NtCreateSection(HANDLE *SectionHandle, ACCESS_MASK DesiredAccess,
                            WOW64_OBJECT_ATTRIBUTES *ObjectAttributes,
                            PLARGE_INTEGER MaximumSize, ULONG PageProtection,
                            ULONG AllocationAttributes, HANDLE FileHandle);
LONG WINAPI NtMapViewOfSection(HANDLE SectionHandle, HANDLE ProcessHandle,
                               PVOID *BaseAddress, ULONG_PTR ZeroBits,
                               SIZE_T CommitSize, PLARGE_INTEGER SectionOffset,
                               SIZE_T *ViewSize, ULONG InheritDisposition,
                               ULONG AllocationType, ULONG Win32Protect);
LONG WINAPI NtClose(HANDLE Handle);
LONG WINAPI NtQueryInformationProcess(HANDLE, ULONG, PVOID, ULONG, PULONG);

/* ---- WoW64/FEX contract constants (same as wow64probe / wow64shim.rs) --- */

#define WOW64_TEB_TLSSLOTS_OFFSET  0x1480  /* TEB64->TlsSlots[]              */
#define WOW64_TLS_WOW64INFO        10
#define WOW64_TLS_CPURESERVED      1
#define WOW64_INITBLOCK_NTDLL_OFF  72      /* SYSTEM_DLL_INIT_BLOCK.ntdll_handle */
#define WOW64_ZEROBITS_BELOW_2G    ((ULONG_PTR)0x7FFFFFFF) /* mask form */
#define WOW64_TEB_WOWTEBOFFSET     0x180c  /* raw Win10 slot (see MmCreateTeb) */
#define WOW64_MACHINE_I386         0x014C
#define WOW64_MACHINE_ARM64        0xAA64

/* FEX winternl.h WOW64INFO — TlsSlots[10] points here; BTCpuProcessInit
 * writes CpuFlags. */
typedef struct _WOW64INFO_FEX {
    ULONG NativeSystemPageSize;
    ULONG CpuFlags;
    ULONG Wow64ExecuteFlags;
    ULONG Unknown;
    ULONGLONG SectionHandle;
    ULONGLONG CrossProcessWorkList;
    USHORT NativeMachineType;
    USHORT EmulatedMachineType;
} WOW64INFO_FEX;

static PVOID *Wow64pTlsSlots(VOID)
{
    return (PVOID *)((PBYTE)NtCurrentTeb() + WOW64_TEB_TLSSLOTS_OFFSET);
}

/* The CPU layer's exception entry: services LookupCache overcommit faults /
 * SMC / suspends, NtContinue-ing itself when handled. Every exception routes
 * here first (real wow64 wires this through Wow64PrepareForException). */
static LONG (WINAPI *Wow64pBTCpuResetToConsistentState)(EXCEPTION_POINTERS *);

static LONG WINAPI Wow64pVeh(EXCEPTION_POINTERS *Info)
{
    if (Wow64pBTCpuResetToConsistentState)
        Wow64pBTCpuResetToConsistentState(Info); /* NtContinues if handled */

    DbgPrint("[FEX-WOW64C] VEH: unhandled code=%08lx addr=%p pc=%p\n",
             Info->ExceptionRecord->ExceptionCode,
             Info->ExceptionRecord->ExceptionAddress,
             (PVOID)Info->ContextRecord->Pc);
    return EXCEPTION_CONTINUE_SEARCH;
}

static PVOID Wow64pAllocBelow2G(SIZE_T Size, ULONG Protect, const char *What)
{
    PVOID Base = NULL;
    LONG Status = NtAllocateVirtualMemory(WOW64_CURRENT_PROCESS, &Base,
                                          WOW64_ZEROBITS_BELOW_2G, &Size,
                                          MEM_RESERVE | MEM_COMMIT, Protect);
    if (Status < 0 || (ULONG_PTR)Base >= 0x80000000u)
    {
        DbgPrint("[FEX-WOW64C] %s: below-2GB alloc FAILED (status=%08lx base=%p)\n",
                 What, Status, Base);
        return NULL;
    }
    return Base;
}

/* Map the staged 32-bit ntdll (SEC_IMAGE) for LdrSystemDllInitBlock.
 * Path: \??\<SystemRoot>\syswow64\ntdll.dll from KUSER_SHARED_DATA. */
static PVOID Wow64pMapNtdll32(VOID)
{
    static WCHAR PathBuf[280];
    const WCHAR *SystemRoot = (const WCHAR *)((PBYTE)0x7FFE0000 + 0x30);
    WCHAR *P = PathBuf;
    const WCHAR *S;
    WOW64_UNICODE_STRING Path;
    WOW64_OBJECT_ATTRIBUTES Oa;
    WOW64_IO_STATUS_BLOCK Iosb;
    HANDLE File = NULL, Section = NULL;
    PVOID Base = NULL;
    SIZE_T ViewSize = 0;
    LONG Status;

    for (S = L"\\??\\"; *S;) *P++ = *S++;
    for (S = SystemRoot; *S && (P - PathBuf) < 260;) *P++ = *S++;
    for (S = L"\\syswow64\\ntdll.dll"; *S;) *P++ = *S++;
    *P = 0;
    RtlInitUnicodeString(&Path, PathBuf);

    Oa.Length = sizeof(Oa);
    Oa.RootDirectory = NULL;
    Oa.ObjectName = &Path;
    Oa.Attributes = 0x40; /* OBJ_CASE_INSENSITIVE */
    Oa.SecurityDescriptor = NULL;
    Oa.SecurityQualityOfService = NULL;

    Status = NtOpenFile(&File, GENERIC_READ | GENERIC_EXECUTE | SYNCHRONIZE,
                        &Oa, &Iosb, FILE_SHARE_READ,
                        0x60 /* FILE_NON_DIRECTORY_FILE|FILE_SYNCHRONOUS_IO_NONALERT */);
    if (Status < 0)
    {
        DbgPrint("[FEX-WOW64C] NtOpenFile(%S) FAILED %08lx\n", PathBuf, Status);
        return NULL;
    }

    Status = NtCreateSection(&Section,
                             SECTION_MAP_READ | SECTION_MAP_EXECUTE | SECTION_QUERY,
                             NULL, NULL, PAGE_EXECUTE, SEC_IMAGE, File);
    NtClose(File);
    if (Status < 0)
    {
        DbgPrint("[FEX-WOW64C] NtCreateSection(ntdll32) FAILED %08lx\n", Status);
        return NULL;
    }

    Status = NtMapViewOfSection(Section, WOW64_CURRENT_PROCESS, &Base, 0, 0,
                                NULL, &ViewSize, 1 /* ViewShare */, 0,
                                PAGE_EXECUTE);
    NtClose(Section);
    if (Status < 0)
    {
        DbgPrint("[FEX-WOW64C] NtMapViewOfSection(ntdll32) FAILED %08lx\n", Status);
        return NULL;
    }
    DbgPrint("[FEX-WOW64C] ntdll32 mapped at %p size=%zx\n", Base, ViewSize);
    return Base;
}

/*
 * Phase-B probe services (wow64probe.exe / docs/engineering/wow64-plan.md).
 * Magic service numbers far above any real syscall id: ECHO proves 32-bit
 * guest-stack arguments marshal through the CPU layer and a 64-bit result
 * flows back into guest EAX; VERDICT receives that result back FROM the
 * guest, proving the full round-trip, then ends the guest process (the only
 * way out of BTCpuSimulate). Real Nt* thunks are Phase C/D.
 */
#define WOW64B_SVC_ECHO      0xF00Du
#define WOW64B_SVC_VERDICT   0xD00Eu
#define WOW64B_ECHO_MAGIC    0x5CA11EDu

/* Phase C guest exit: the exit stub (emitted below the image entry hand-off)
 * lands here when the guest's entry point returns. Arguments[0] = guest EAX.
 * hello32.exe returns the 0x51DE sentinel; that maps to harness PASS. */
#define WOW64C_SVC_EXIT      0xE0D1u
#define WOW64C_HELLO32_SENTINEL 0x51DEu

/*
 * Wow64SystemServiceEx — the syscall thunk dispatcher. The CPU layer traps a
 * guest `syscall`/BOP, stores the guest context, and calls here with the
 * 32-bit service number and a pointer to the 32-bit argument array (guest
 * stack). Unknown services report and fail loud (STATUS_INVALID_SYSTEM_
 * SERVICE) so a premature guest syscall is visible on serial instead of
 * silently returning success.
 */
ULONG
WINAPI
Wow64SystemServiceEx(ULONG ServiceNumber, PULONG Arguments)
{
    switch (ServiceNumber)
    {
        case WOW64B_SVC_ECHO:
            DbgPrint("[FEX-WOW64B] ECHO service: args %p = %08lx %08lx %08lx\n",
                     Arguments, Arguments[0], Arguments[1], Arguments[2]);
            return WOW64B_ECHO_MAGIC;

        case WOW64B_SVC_VERDICT:
            if (Arguments[0] == WOW64B_ECHO_MAGIC)
            {
                DbgPrint("[FEX-WOW64B] guest syscall round-trip PASS (echo magic returned by guest)\n");
                NtTerminateProcess(WOW64_CURRENT_PROCESS, 0x600D);
            }
            DbgPrint("[FEX-WOW64B] guest syscall round-trip FAIL (guest passed %08lx, want %08x)\n",
                     Arguments[0], WOW64B_ECHO_MAGIC);
            NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
            return 0; /* unreachable */

        case WOW64C_SVC_EXIT:
            DbgPrint("[FEX-WOW64C] guest entry returned %08lx (want %08x)\n",
                     Arguments[0], WOW64C_HELLO32_SENTINEL);
            if (Arguments[0] == WOW64C_HELLO32_SENTINEL)
            {
                DbgPrint("[FEX-WOW64C] M2 PASS: real i386 image code executed under FEX in a WoW64 process\n");
                NtTerminateProcess(WOW64_CURRENT_PROCESS, 0x600D);
            }
            NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
            return 0; /* unreachable */

        default:
            DbgPrint("[WOW64] Wow64SystemServiceEx: service 0x%lx args %p — no thunk table yet (Phase B)\n",
                     ServiceNumber, Arguments);
            return (ULONG)0xC000001CL; /* STATUS_INVALID_SYSTEM_SERVICE */
    }
}

/*
 * Wow64LdrpInitialize — the thread handoff from the 64-bit ntdll loader.
 * For a WoW64 process, LdrpInitializeProcess (ntdll-rust ldrinit.rs) skips the
 * i386 image entirely, loads this DLL, and calls here with the initial thread
 * CONTEXT. This function owns the thread from here on and never returns: it
 * will bootstrap the 32-bit side (map syswow64 ntdll, init the CPU dll, build
 * the guest context, BTCpuSimulate).
 *
 * Phase C milestone 1 scope: prove the kernel+loader chain end-to-end —
 * EPROCESS marked WoW64 (PEB32 via ProcessWow64Information), TEB32 reachable
 * through the raw WowTebOffset slot (TEB+0x180c, kernel MmCreateTeb wrote it),
 * then exit 0x600D as the harness verdict. The CPU-layer bring-up (proven
 * standalone by wow64probe in Phase B) moves in here as milestone 2.
 */
VOID
WINAPI
Wow64LdrpInitialize(PVOID Context)
{
    ULONG_PTR Peb32 = 0;
    PBYTE Teb = (PBYTE)NtCurrentTeb();
    LONG WowTebOffset = *(LONG *)(Teb + WOW64_TEB_WOWTEBOFFSET);
    PBYTE Teb32 = WowTebOffset ? Teb + WowTebOffset : NULL;
    ULONG ImageBase32;
    PBYTE ImageBase;
    PIMAGE_NT_HEADERS32 Nt32;
    ULONG EntryRva;
    PVOID Ntdll32, InitBlock, Fex, Bop;
    PBYTE ExitStub, GuestStack;
    ULONG GuestEsp;
    WOW64_UNICODE_STRING DllName;
    WOW64_ANSI_STRING ProcName;
    VOID (WINAPI *BTCpuProcessInit)(VOID);
    VOID (WINAPI *BTCpuThreadInit)(VOID);
    PVOID (WINAPI *BTCpuGetBopCode)(VOID);
    VOID (WINAPI *BTCpuSimulate)(VOID);
    static WOW64INFO_FEX Wow64Info;
    static UCHAR CpuAreaBuf[16 + sizeof(WOW64_CONTEXT) + 16];
    PBYTE CpuArea;
    WOW64_CONTEXT *Wow64Ctx;
    PBYTE XSave;
    LONG Status;

    DbgPrint("[FEX-WOW64C] Wow64LdrpInitialize: 64-bit loader handed the thread over (Context=%p)\n",
             Context);

    /* Register the crash oracle FIRST: libwow64fex.dll (and its kernel32
     * dependency) load below, before any BTCpu call — a fault there must be
     * dumped, not swallowed into a silent process exit. */
    RtlAddVectoredExceptionHandler(1, Wow64pVeh);

    /* -- M1 chain verification (kernel bootstrap sanity) ------------------ */
    Status = NtQueryInformationProcess(WOW64_CURRENT_PROCESS,
                                       26 /* ProcessWow64Information */,
                                       &Peb32, sizeof(Peb32), NULL);
    if (Status < 0 || !Peb32 || !Teb32 ||
        *(ULONG *)(Teb32 + 0x30 /* TEB32.ProcessEnvironmentBlock */) != (ULONG)Peb32)
    {
        DbgPrint("[FEX-WOW64C] FAIL: PEB32/TEB32 chain inconsistent (PEB32=%p TEB32=%p st=%08lx)\n",
                 (PVOID)Peb32, Teb32, Status);
        NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
    }

    ImageBase32 = *(ULONG *)((PBYTE)Peb32 + 0x08 /* PEB32.ImageBaseAddress */);
    ImageBase = (PBYTE)(ULONG_PTR)ImageBase32;
    Nt32 = (PIMAGE_NT_HEADERS32)(ImageBase + ((PIMAGE_DOS_HEADER)ImageBase)->e_lfanew);
    EntryRva = Nt32->OptionalHeader.AddressOfEntryPoint;
    DbgPrint("[FEX-WOW64C] PEB32=%p TEB32=%p image=%08lx entry=+%lx\n",
             (PVOID)Peb32, Teb32, ImageBase32, EntryRva);

    /* -- 32-bit ntdll + LdrSystemDllInitBlock ----------------------------- */
    Ntdll32 = Wow64pMapNtdll32();
    if (!Ntdll32)
        NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);

    {
        WOW64_UNICODE_STRING NtdllName;
        PVOID NtdllBase = NULL;
        RtlInitUnicodeString(&NtdllName, L"ntdll.dll");
        LdrGetDllHandle(NULL, NULL, &NtdllName, &NtdllBase);
        RtlInitAnsiString(&ProcName, "LdrSystemDllInitBlock");
        InitBlock = NULL;
        if (!NtdllBase ||
            LdrGetProcedureAddress(NtdllBase, &ProcName, 0, &InitBlock) < 0 ||
            !InitBlock)
        {
            DbgPrint("[FEX-WOW64C] FAIL: LdrSystemDllInitBlock unresolved\n");
            NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
        }
        *(ULONGLONG *)((PBYTE)InitBlock + WOW64_INITBLOCK_NTDLL_OFF) =
            (ULONGLONG)(ULONG_PTR)Ntdll32;
    }

    /* -- WOW64INFO + environment landmine (Logging::Init formats
     *    getenv("LOCALAPPDATA") unchecked; default it before the CPU dll's
     *    mini-CRT snapshots the PEB environment) --------------------------- */
    Wow64Info.NativeSystemPageSize = 0x1000;
    Wow64Info.NativeMachineType = WOW64_MACHINE_ARM64;
    Wow64Info.EmulatedMachineType = WOW64_MACHINE_I386;
    Wow64pTlsSlots()[WOW64_TLS_WOW64INFO] = &Wow64Info;

    {
        WOW64_UNICODE_STRING EnvName, EnvValue;
        RtlInitUnicodeString(&EnvName, L"LOCALAPPDATA");
        RtlInitUnicodeString(&EnvValue, L"C:\\ReactOS\\Temp");
        RtlSetEnvironmentVariable(NULL, &EnvName, &EnvValue);
    }

    /* -- Load the CPU layer ------------------------------------------------ */
    DbgPrint("[FEX-WOW64C] InitBlock.ntdll_handle set; loading libwow64fex.dll...\n");
    RtlInitUnicodeString(&DllName, L"libwow64fex.dll");
    Fex = NULL;
    Status = LdrLoadDll(NULL, NULL, &DllName, &Fex);
    if (Status < 0 || !Fex)
    {
        DbgPrint("[FEX-WOW64C] FAIL: libwow64fex.dll load %08lx\n", Status);
        NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
    }

#define WOW64_RESOLVE(fn)                                                     \
    do {                                                                      \
        PVOID Addr = NULL;                                                    \
        RtlInitAnsiString(&ProcName, #fn);                                    \
        if (LdrGetProcedureAddress(Fex, &ProcName, 0, &Addr) < 0 || !Addr)    \
        {                                                                     \
            DbgPrint("[FEX-WOW64C] FAIL: " #fn " unresolved\n");              \
            NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);                 \
        }                                                                     \
        *(PVOID *)&fn = Addr;                                                 \
    } while (0)

    WOW64_RESOLVE(BTCpuProcessInit);
    WOW64_RESOLVE(BTCpuThreadInit);
    WOW64_RESOLVE(BTCpuGetBopCode);
    WOW64_RESOLVE(BTCpuSimulate);
    {
        PVOID Addr = NULL;
        RtlInitAnsiString(&ProcName, "BTCpuResetToConsistentState");
        LdrGetProcedureAddress(Fex, &ProcName, 0, &Addr);
        Wow64pBTCpuResetToConsistentState = (LONG (WINAPI *)(EXCEPTION_POINTERS *))Addr;
    }
#undef WOW64_RESOLVE

    DbgPrint("[FEX-WOW64C] libwow64fex.dll loaded @ %p; calling BTCpuProcessInit...\n", Fex);
    BTCpuProcessInit();
    Bop = BTCpuGetBopCode();
    DbgPrint("[FEX-WOW64C] BTCpuProcessInit OK — BOP=%p CpuFlags=%lx\n",
             Bop, Wow64Info.CpuFlags);
    if ((ULONG_PTR)Bop >= 0x80000000u)
    {
        DbgPrint("[FEX-WOW64C] FAIL: BOP above 2GB\n");
        NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
    }

    /* -- Guest exit stub + stack ------------------------------------------ */
    /* When the image entry (cdecl, no args) returns, EIP lands on this stub
     * with EAX = the return value. Stub shape mirrors a 32-bit ntdll syscall
     * stub (HandleSyscallImpl pops the ret-to-stub slot, args at ESP+8):
     *   exit:  50               push eax          ; arg1 = guest exit code
     *          B8 <EXIT>        mov eax, SVC_EXIT
     *          E8 <rel bopcall> call bopcall      ; (never returns)
     *          F4               hlt
     *   bopcall: BA <BOP>       mov edx, BOP
     *          FF D2            call edx
     *          C3               ret
     */
    ExitStub = Wow64pAllocBelow2G(0x1000, PAGE_EXECUTE_READWRITE, "exit stub");
    GuestStack = Wow64pAllocBelow2G(0x10000, PAGE_READWRITE, "guest stack");
    if (!ExitStub || !GuestStack)
        NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
    {
        PBYTE P = ExitStub;
        PBYTE BopCall;
        ULONG Rel;
        *P++ = 0x50;                                             /* push eax */
        *P++ = 0xB8; *(ULONG *)P = WOW64C_SVC_EXIT; P += 4;      /* mov eax  */
        *P++ = 0xE8; BopCall = P; P += 4;                        /* call rel */
        *P++ = 0xF4;                                             /* hlt      */
        Rel = (ULONG)(P - (BopCall + 4));
        *(ULONG *)BopCall = Rel;
        *P++ = 0xBA; *(ULONG *)P = (ULONG)(ULONG_PTR)Bop; P += 4;/* mov edx  */
        *P++ = 0xFF; *P++ = 0xD2;                                /* call edx */
        *P++ = 0xC3;                                             /* ret      */
    }

    GuestEsp = (ULONG)(ULONG_PTR)GuestStack + 0x10000 - 64;
    GuestEsp -= 4;
    *(ULONG *)(ULONG_PTR)GuestEsp = (ULONG)(ULONG_PTR)ExitStub; /* ret addr */

    /* -- CPU area (WOW64_CPURESERVED + 16-aligned WOW64_CONTEXT) ----------- */
    CpuArea = (PBYTE)(((ULONG_PTR)CpuAreaBuf + 15) & ~(ULONG_PTR)15);
    *(USHORT *)(CpuArea + 2) = WOW64_MACHINE_I386; /* WOW64_CPURESERVED.Machine */
    Wow64Ctx = (WOW64_CONTEXT *)(CpuArea + 16);
    Wow64Ctx->ContextFlags = WOW64_CONTEXT_FULL | WOW64_CONTEXT_EXTENDED_REGISTERS;
    Wow64Ctx->Eip = ImageBase32 + EntryRva;
    Wow64Ctx->Esp = GuestEsp;
    Wow64Ctx->EFlags = 0x202;
    Wow64Ctx->SegCs = 0x23;
    Wow64Ctx->SegSs = 0x2B;
    Wow64Ctx->SegDs = 0x2B;
    Wow64Ctx->SegEs = 0x2B;
    Wow64Ctx->SegFs = 0x53;
    Wow64Ctx->SegGs = 0x2B;
    XSave = (PBYTE)Wow64Ctx->ExtendedRegisters;
    *(USHORT *)(XSave + 0x00) = 0x027F;     /* FCW   */
    *(ULONG *)(XSave + 0x18) = 0x00001F80;  /* MXCSR */
    Wow64pTlsSlots()[WOW64_TLS_CPURESERVED] = CpuArea;

    BTCpuThreadInit();

    DbgPrint("[FEX-WOW64C] entering BTCpuSimulate (eip=%08lx esp=%08lx) — verdict via EXIT service\n",
             Wow64Ctx->Eip, Wow64Ctx->Esp);
    BTCpuSimulate();

    DbgPrint("[FEX-WOW64C] FAIL: BTCpuSimulate returned\n");
    NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
}

/*
 * Wow64ProcessPendingCrossProcessItems — drains the cross-process work list
 * (writes into this process's guest memory by other processes, e.g.
 * CreateRemoteThread / WriteProcessMemory notifications). No cross-process
 * machinery exists yet; nothing to drain.
 */
VOID
WINAPI
Wow64ProcessPendingCrossProcessItems(VOID)
{
}

BOOL
WINAPI
DllMain(HINSTANCE hDll, DWORD dwReason, LPVOID lpReserved)
{
    UNREFERENCED_PARAMETER(lpReserved);

    UNREFERENCED_PARAMETER(hDll);

    if (dwReason == DLL_PROCESS_ATTACH)
    {
        /* No DisableThreadLibraryCalls: that's kernel32, and this DLL is
         * deliberately ntdll-only. Thread attach/detach no-op through DllMain. */
        DbgPrint("[WOW64] wow64.dll loaded\n");
    }
    return TRUE;
}
