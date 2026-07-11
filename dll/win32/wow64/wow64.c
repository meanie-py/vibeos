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
LONG WINAPI NtProtectVirtualMemory(HANDLE ProcessHandle, PVOID *BaseAddress,
                                   SIZE_T *RegionSize, ULONG NewProtect,
                                   PULONG OldProtect);

/* wow64_thunks.c — the wow64gen-generated 32->64 syscall thunk table. */
LONG Wow64DispatchSystemService(ULONG ServiceNumber, PULONG Arguments);

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

/* Resolve an export from the mapped 32-bit ntdll by hand (PE32 export
 * directory walk — arch-independent format; we avoid LdrGetProcedureAddress
 * because the i386 image is a raw section mapping, not a loader entry). */
static PVOID Wow64pFindExport32(PVOID Base, const char *Name)
{
    PBYTE B = (PBYTE)Base;
    PIMAGE_NT_HEADERS32 Nt = (PIMAGE_NT_HEADERS32)(B + ((PIMAGE_DOS_HEADER)B)->e_lfanew);
    ULONG ExpRva = Nt->OptionalHeader.DataDirectory[0].VirtualAddress;
    PIMAGE_EXPORT_DIRECTORY Exp;
    PULONG NameRvas, FuncRvas;
    PUSHORT Ordinals;
    ULONG i;

    if (!ExpRva)
        return NULL;
    Exp = (PIMAGE_EXPORT_DIRECTORY)(B + ExpRva);
    NameRvas = (PULONG)(B + Exp->AddressOfNames);
    Ordinals = (PUSHORT)(B + Exp->AddressOfNameOrdinals);
    FuncRvas = (PULONG)(B + Exp->AddressOfFunctions);
    for (i = 0; i < Exp->NumberOfNames; i++)
    {
        const char *N = (const char *)(B + NameRvas[i]);
        const char *M = Name;
        while (*N && *N == *M) { N++; M++; }
        if (*N == *M)
            return B + FuncRvas[Ordinals[i]];
    }
    return NULL;
}

/* ---- 32-bit process-parameters down-conversion (M3b) -------------------- *
 * The kernel builds PEB32 by mirroring scalars only; ProcessParameters stays
 * NULL. The 32-bit loader (LdrpInitializeProcess) derefs it immediately, so
 * we down-convert the process's real 64-bit RTL_USER_PROCESS_PARAMETERS
 * (PEB64->ProcessParameters, already built by the native loader) into a
 * 32-bit one below 2GB — width conversion of the UNICODE_STRINGs, not content
 * synthesis. Win10's wow64.dll does the same PEB32 fixup at process init. */

typedef struct _WOW64_UNICODE_STRING32 {
    USHORT Length;
    USHORT MaximumLength;
    ULONG Buffer; /* 32-bit guest pointer */
} WOW64_UNICODE_STRING32;

/* 32-bit RTL_USER_PROCESS_PARAMETERS (offsets are the canonical ntdll ones;
 * only the fields the loader reads are converted, the rest stay zero). */
typedef struct _WOW64_RTL_USER_PROCESS_PARAMETERS32 {
    ULONG MaximumLength;                  /* +0x00 */
    ULONG Length;                         /* +0x04 */
    ULONG Flags;                          /* +0x08 */
    ULONG DebugFlags;                     /* +0x0c */
    ULONG ConsoleHandle;                  /* +0x10 */
    ULONG ConsoleFlags;                   /* +0x14 */
    ULONG StandardInput;                  /* +0x18 */
    ULONG StandardOutput;                 /* +0x1c */
    ULONG StandardError;                  /* +0x20 */
    WOW64_UNICODE_STRING32 CurrentDirectoryPath; /* +0x24 CURDIR.DosPath */
    ULONG CurrentDirectoryHandle;         /* +0x2c CURDIR.Handle */
    WOW64_UNICODE_STRING32 DllPath;       /* +0x30 */
    WOW64_UNICODE_STRING32 ImagePathName; /* +0x38 */
    WOW64_UNICODE_STRING32 CommandLine;   /* +0x40 */
    ULONG Environment;                    /* +0x48 */
    ULONG StartingX, StartingY, CountX, CountY, CountCharsX, CountCharsY;
    ULONG FillAttribute, WindowFlags, ShowWindowFlags;
    WOW64_UNICODE_STRING32 WindowTitle;   /* +0x70 */
    WOW64_UNICODE_STRING32 DesktopInfo;   /* +0x78 */
    WOW64_UNICODE_STRING32 ShellInfo;     /* +0x80 */
    WOW64_UNICODE_STRING32 RuntimeData;   /* +0x88 */
} WOW64_RTL_USER_PROCESS_PARAMETERS32;

/* PEB64 field offsets (ntdll native layout). */
#define PEB64_PROCESS_PARAMETERS_OFF  0x20
#define PEB32_PROCESS_PARAMETERS_OFF  0x10
/* 64-bit RTL_USER_PROCESS_PARAMETERS UNICODE_STRING member offsets. */
#define UPP64_IMAGEPATHNAME_OFF  0x60
#define UPP64_COMMANDLINE_OFF    0x70
#define UPP64_WINDOWTITLE_OFF    0xB0
#define UPP64_DESKTOPINFO_OFF    0xC0

/* Copy one UNICODE_STRING64 (at Src) into Dst32, appending its buffer bytes to
 * *Arena (a below-2GB bump pointer). NULL/empty stays zero. */
static VOID Wow64pCopyUstr32(WOW64_UNICODE_STRING32 *Dst32, PBYTE Src, PBYTE *Arena)
{
    USHORT Length = *(USHORT *)(Src + 0);
    PWSTR Buffer = *(PWSTR *)(Src + 8); /* +8: after Length/Max/pad in UNICODE_STRING64 */
    Dst32->Length = Length;
    if (!Length || !Buffer)
    {
        Dst32->MaximumLength = Length ? (USHORT)(Length + sizeof(WCHAR)) : 0;
        Dst32->Buffer = 0;
        return;
    }
    Dst32->MaximumLength = (USHORT)(Length + sizeof(WCHAR));
    Dst32->Buffer = (ULONG)(ULONG_PTR)*Arena;
    for (USHORT i = 0; i < Length; i++)
        (*Arena)[i] = ((PBYTE)Buffer)[i];
    (*Arena)[Length] = 0;
    (*Arena)[Length + 1] = 0;
    *Arena += (Length + sizeof(WCHAR) + 3) & ~3u;
}

static VOID Wow64pBuildProcessParams32(ULONG_PTR Peb32)
{
    PBYTE Peb64 = (PBYTE)NtCurrentTeb() + 0; /* set below */
    PBYTE Upp64;
    WOW64_RTL_USER_PROCESS_PARAMETERS32 *Upp32;
    PBYTE Arena, Block;
    SIZE_T Size = 0x2000; /* params + string arena */

    Peb64 = *(PBYTE *)((PBYTE)NtCurrentTeb() + 0x60); /* TEB64->ProcessEnvironmentBlock */
    Upp64 = *(PBYTE *)(Peb64 + PEB64_PROCESS_PARAMETERS_OFF);
    if (!Upp64)
    {
        DbgPrint("[FEX-WOW64C] WARN: PEB64->ProcessParameters NULL — 32-bit params left empty\n");
        return;
    }

    Block = (PBYTE)Wow64pAllocBelow2G(Size, PAGE_READWRITE, "proc params32");
    if (!Block)
        NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
    Upp32 = (WOW64_RTL_USER_PROCESS_PARAMETERS32 *)Block;
    Arena = Block + sizeof(*Upp32);
    Arena = (PBYTE)(((ULONG_PTR)Arena + 3) & ~(ULONG_PTR)3);

    Upp32->MaximumLength = (ULONG)Size;
    Upp32->Length = (ULONG)Size;
    Upp32->Flags = 0x6001; /* RTL_USER_PROC_PARAMS_NORMALIZED | ProfileUser */
    Wow64pCopyUstr32(&Upp32->ImagePathName, Upp64 + UPP64_IMAGEPATHNAME_OFF, &Arena);
    Wow64pCopyUstr32(&Upp32->CommandLine, Upp64 + UPP64_COMMANDLINE_OFF, &Arena);
    Wow64pCopyUstr32(&Upp32->WindowTitle, Upp64 + UPP64_WINDOWTITLE_OFF, &Arena);
    Wow64pCopyUstr32(&Upp32->DesktopInfo, Upp64 + UPP64_DESKTOPINFO_OFF, &Arena);

    *(ULONG *)((PBYTE)Peb32 + PEB32_PROCESS_PARAMETERS_OFF) = (ULONG)(ULONG_PTR)Upp32;
    DbgPrint("[FEX-WOW64C] PEB32->ProcessParameters built at %p (image='%.*S')\n",
             Upp32, Upp32->ImagePathName.Length / 2,
             (PWSTR)(ULONG_PTR)Upp32->ImagePathName.Buffer);
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
 * lands here when the guest's entry point returns. M3a shape: the stub first
 * calls the REAL ntdll32 NtClose(NULL) stub (mov eax,N; call
 * [Wow64Transition]) so Arguments[0] = that call's NTSTATUS (want
 * STATUS_INVALID_HANDLE from the generated thunk table) and Arguments[1] =
 * the guest entry's EAX (hello32.exe returns the 0x51DE sentinel). */
#define WOW64C_SVC_EXIT      0xE0D1u
#define WOW64C_HELLO32_SENTINEL 0x51DEu
#define WOW64C_STATUS_INVALID_HANDLE 0xC0000008u

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
            DbgPrint("[FEX-WOW64C] guest entry returned %08lx (want %08x), NtClose(NULL) via ntdll32 stub => %08lx (want %08x)\n",
                     Arguments[1], WOW64C_HELLO32_SENTINEL,
                     Arguments[0], WOW64C_STATUS_INVALID_HANDLE);
            if (Arguments[1] == WOW64C_HELLO32_SENTINEL &&
                Arguments[0] == WOW64C_STATUS_INVALID_HANDLE)
            {
                DbgPrint("[FEX-WOW64C] M2 PASS: real i386 image code executed under FEX in a WoW64 process\n");
                DbgPrint("[FEX-WOW64C] M3a PASS: real ntdll32 syscall stub round-tripped Wow64Transition -> BOP -> thunk table\n");
                DbgPrint("[FEX-WOW64C] M3b PASS: guest ntdll32 LdrInitializeThunk ran the 32-bit loader, then handed off to the image entry\n");
                NtTerminateProcess(WOW64_CURRENT_PROCESS, 0x600D);
            }
            NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
            return 0; /* unreachable */

        default:
            /* Phase D: generated thunk table (wow64_thunks.c), indexed by the
             * real service number the guest's ntdll32 stub put in EAX. */
            return (ULONG)Wow64DispatchSystemService(ServiceNumber, Arguments);
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
    PBYTE ExitStub, GuestStack, ApcStack;
    PVOID KiUserApcDispatcher32, LdrInitThunk32;
    ULONG GuestEsp, ApcEsp;
    WOW64_UNICODE_STRING DllName;
    WOW64_ANSI_STRING ProcName;
    VOID (WINAPI *BTCpuProcessInit)(VOID);
    VOID (WINAPI *BTCpuThreadInit)(VOID);
    PVOID (WINAPI *BTCpuGetBopCode)(VOID);
    VOID (WINAPI *BTCpuSimulate)(VOID);
    LONG (WINAPI *BTCpuNotifyMapViewOfSection)(PVOID, PVOID, PVOID, SIZE_T, ULONG, ULONG);
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
    WOW64_RESOLVE(BTCpuNotifyMapViewOfSection);
    {
        PVOID Addr = NULL;
        RtlInitAnsiString(&ProcName, "BTCpuResetToConsistentState");
        LdrGetProcedureAddress(Fex, &ProcName, 0, &Addr);
        Wow64pBTCpuResetToConsistentState = (LONG (WINAPI *)(EXCEPTION_POINTERS *))Addr;
    }
#undef WOW64_RESOLVE

    DbgPrint("[FEX-WOW64C] libwow64fex.dll loaded @ %p; calling BTCpuProcessInit...\n", Fex);
    /* FEX discovers the process address-space layout during ProcessInit.
     * Allocate every raw guest page first; pages created afterward are not
     * visible until the full memory-notification contract is wired. */
    ExitStub = Wow64pAllocBelow2G(0x1000, PAGE_EXECUTE_READWRITE, "exit stub");
    GuestStack = Wow64pAllocBelow2G(0x10000, PAGE_READWRITE, "guest run stack");
    ApcStack = Wow64pAllocBelow2G(0x40000, PAGE_READWRITE, "guest loader stack");
    if (!ExitStub || !GuestStack || !ApcStack)
        NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);

    /* Build PEB32->ProcessParameters (down-converted from PEB64) so the 32-bit
     * loader has valid ImagePathName/CommandLine to run LdrpInitializeProcess. */
    Wow64pBuildProcessParams32(Peb32);

    BTCpuProcessInit();
    Bop = BTCpuGetBopCode();
    DbgPrint("[FEX-WOW64C] BTCpuProcessInit OK — BOP=%p CpuFlags=%lx\n",
             Bop, Wow64Info.CpuFlags);
    if ((ULONG_PTR)Bop >= 0x80000000u)
    {
        DbgPrint("[FEX-WOW64C] FAIL: BOP above 2GB\n");
        NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
    }

    /* -- Publish the BOP into the guest ntdll32's Wow64Transition slot ------
     * (Win10-parity: every i386 user syscall stub is `mov eax, N; call
     * [Wow64Transition]`; statically the slot holds the native fallback, we
     * repoint it at the CPU layer's bridge in OUR process's COW data page). */
    {
        PULONG Slot = (PULONG)Wow64pFindExport32(Ntdll32, "Wow64Transition");
        PVOID Page;
        SIZE_T Size = sizeof(ULONG);
        ULONG OldProt = 0;
        if (!Slot)
        {
            DbgPrint("[FEX-WOW64C] FAIL: ntdll32 has no Wow64Transition export (stale syswow64 stage?)\n");
            NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
        }
        Page = Slot;
        NtProtectVirtualMemory(WOW64_CURRENT_PROCESS, &Page, &Size,
                               PAGE_READWRITE, &OldProt); /* COW data — best effort */
        *Slot = (ULONG)(ULONG_PTR)Bop;
        DbgPrint("[FEX-WOW64C] Wow64Transition slot %p patched -> BOP %p\n", Slot, Bop);
    }

    /* -- Register the guest images with FEX -------------------------------
     * The kernel mapped hello32 (image base) and we mapped ntdll32 BEFORE
     * BTCpuProcessInit ran its address-space discovery — so FEX's image
     * tracker doesn't know they're images and the guest ntdll's
     * RtlImageNtHeader read of the PE header at the image base returns NULL
     * (loader then near-null-writes). Notify FEX now (real WoW64 calls this
     * from the map-view hook). */
    BTCpuNotifyMapViewOfSection(NULL, (PVOID)(ULONG_PTR)ImageBase32, NULL, 0, 0, PAGE_EXECUTE_READ);
    BTCpuNotifyMapViewOfSection(NULL, Ntdll32, NULL, 0, 0, PAGE_EXECUTE_READ);
    DbgPrint("[FEX-WOW64C] BTCpuNotifyMapViewOfSection: image=%08lx ntdll32=%p registered\n",
             ImageBase32, Ntdll32);

    /* -- Resolve the guest ntdll32 loader entrypoints (M3b) ---------------- */
    KiUserApcDispatcher32 = Wow64pFindExport32(Ntdll32, "KiUserApcDispatcher");
    LdrInitThunk32 = Wow64pFindExport32(Ntdll32, "LdrInitializeThunk");
    if (!KiUserApcDispatcher32 || !LdrInitThunk32)
    {
        DbgPrint("[FEX-WOW64C] FAIL: ntdll32 KiUserApcDispatcher=%p LdrInitializeThunk=%p\n",
                 KiUserApcDispatcher32, LdrInitThunk32);
        NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
    }

    /* -- Guest exit stub + stack ------------------------------------------ */
    /* When the image entry (cdecl, no args) returns, EIP lands on this stub
     * with EAX = the return value. M3a shape: exercise a REAL ntdll32 syscall
     * stub first (NtClose(NULL) -> STATUS_INVALID_HANDLE through
     * Wow64Transition -> BOP -> generated thunk table), then report both
     * results through the EXIT service (HandleSyscallImpl pops the
     * ret-to-bopcall slot, args at ESP+8):
     *   exit:  50               push eax          ; save entry sentinel
     *          6A 00            push 0            ; NtClose arg: handle=NULL
     *          BA <NtClose32>   mov edx, ntdll32!NtClose
     *          FF D2            call edx          ; stub does `ret 4`
     *          50               push eax          ; NtClose status
     *          B8 <EXIT>        mov eax, SVC_EXIT ; args: [status, sentinel]
     *          E8 <rel bopcall> call bopcall      ; (never returns)
     *          F4               hlt
     *   bopcall: BA <BOP>       mov edx, BOP
     *          FF D2            call edx
     *          C3               ret
     */
    {
        PVOID NtClose32 = Wow64pFindExport32(Ntdll32, "NtClose");
        PBYTE P = ExitStub;
        PBYTE BopCall;
        ULONG Rel;
        if (!NtClose32)
        {
            DbgPrint("[FEX-WOW64C] FAIL: ntdll32 has no NtClose export\n");
            NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
        }
        *P++ = 0x50;                                             /* push eax */
        *P++ = 0x6A; *P++ = 0x00;                                /* push 0   */
        *P++ = 0xBA; *(ULONG *)P = (ULONG)(ULONG_PTR)NtClose32; P += 4;
        *P++ = 0xFF; *P++ = 0xD2;                                /* call edx */
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

    /* -- Inner "run" CONTEXT32: what the guest executes AFTER the loader
     *    finishes (LdrInitializeThunk ends by NtContinue-ing to this). It is
     *    the i386 image entry on the run stack, returning to the exit stub. -- */
    {
        static WOW64_CONTEXT RunCtx;
        RunCtx.ContextFlags = WOW64_CONTEXT_FULL | WOW64_CONTEXT_EXTENDED_REGISTERS;
        RunCtx.Eip = ImageBase32 + EntryRva;
        RunCtx.Esp = GuestEsp;
        RunCtx.EFlags = 0x202;
        RunCtx.SegCs = 0x23; RunCtx.SegSs = 0x2B; RunCtx.SegDs = 0x2B;
        RunCtx.SegEs = 0x2B; RunCtx.SegFs = 0x53; RunCtx.SegGs = 0x2B;
        *(USHORT *)((PBYTE)RunCtx.ExtendedRegisters + 0x00) = 0x027F; /* FCW   */
        *(ULONG *)((PBYTE)RunCtx.ExtendedRegisters + 0x18) = 0x00001F80; /* MXCSR */

        /* -- Loader APC frame on the loader stack (KiUserApcDispatcher ABI):
         *    [E+0]=LdrInitializeThunk [E+4]=&CONTEXT [E+8,+12]=SysArgs
         *    [E+16..]=CONTEXT [E+16+CAS..]=SEH scratch. Entry Eip=dispatcher,
         *    Esp=E; the loader runs on the stack below E, then NtContinues to
         *    RunCtx (the image entry). ---------------------------------------- */
        {
            ULONG CAS = (ULONG)((sizeof(WOW64_CONTEXT) + 3) & ~3u);
            PBYTE FrameTop = (PBYTE)ApcStack + 0x40000 - 0x100;
            ULONG E = (ULONG)(ULONG_PTR)(FrameTop - (16 + CAS + 8));
            E &= ~15u;
            *(ULONG *)(ULONG_PTR)(E + 0)  = (ULONG)(ULONG_PTR)LdrInitThunk32;
            *(ULONG *)(ULONG_PTR)(E + 4)  = E + 16; /* NormalContext = &CONTEXT */
            *(ULONG *)(ULONG_PTR)(E + 8)  = 0;
            *(ULONG *)(ULONG_PTR)(E + 12) = 0;
            for (ULONG i = 0; i < sizeof(WOW64_CONTEXT); i++)
                *(PBYTE)(ULONG_PTR)(E + 16 + i) = ((PBYTE)&RunCtx)[i];
            ApcEsp = E;
        }
    }

    /* -- CPU area (WOW64_CPURESERVED + 16-aligned WOW64_CONTEXT): the CONTEXT
     *    the JIT enters — the loader dispatcher, NOT the image entry. -------- */
    CpuArea = (PBYTE)(((ULONG_PTR)CpuAreaBuf + 15) & ~(ULONG_PTR)15);
    *(USHORT *)(CpuArea + 2) = WOW64_MACHINE_I386; /* WOW64_CPURESERVED.Machine */
    Wow64Ctx = (WOW64_CONTEXT *)(CpuArea + 16);
    Wow64Ctx->ContextFlags = WOW64_CONTEXT_FULL | WOW64_CONTEXT_EXTENDED_REGISTERS;
    Wow64Ctx->Eip = (ULONG)(ULONG_PTR)KiUserApcDispatcher32;
    Wow64Ctx->Esp = ApcEsp;
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

    DbgPrint("[FEX-WOW64C] entering BTCpuSimulate via loader (apcDisp=%08lx esp=%08lx LdrInit=%p runEip=%08lx exit=%p)\n",
             Wow64Ctx->Eip, Wow64Ctx->Esp, LdrInitThunk32, ImageBase32 + EntryRva, ExitStub);
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
