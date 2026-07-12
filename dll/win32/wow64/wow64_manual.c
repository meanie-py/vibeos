/*
 * PROJECT:     vibeos wow64.dll
 * PURPOSE:     Hand-written deep marshallers (WoW64 Phase D). wow64gen emits a
 *              loud null-thunk stub for every syscall whose 32/64 struct
 *              layouts differ (tools/wow64gen/manual.txt); this file provides
 *              the real 32->64 conversions, dispatched by name from
 *              Wow64DispatchSystemService when the generated Thunk is null.
 *
 *              Each marshaller reads the guest's 32-bit argument dwords, builds
 *              the 64-bit struct(s) on the host stack, calls the real 64-bit
 *              Nt*, then writes any output struct back down to 32-bit width.
 */

#define WIN32_NO_STATUS
#include <windef.h>
#include <winbase.h>

typedef LONG NTSTATUS;

ULONG WINAPI DbgPrint(_In_z_ _Printf_format_string_ PCSTR Format, ...);

/* Sentinel: this name has no hand marshaller yet -> caller fails loud. */
#define WOW64_MANUAL_UNIMPL ((NTSTATUS)0x7FFFFFFF)

int Wow64pStrEq(const char *a, const char *b);

/* ---- host (64-bit) Nt* prototypes we marshal into ----------------------- */
NTSTATUS WINAPI NtQueryVirtualMemory(HANDLE ProcessHandle, PVOID BaseAddress,
                                     ULONG MemoryInformationClass,
                                     PVOID MemoryInformation,
                                     SIZE_T MemoryInformationLength,
                                     PSIZE_T ReturnLength);
NTSTATUS WINAPI NtQueryInformationProcess(HANDLE ProcessHandle, ULONG InfoClass,
                                          PVOID Info, ULONG Len, PULONG RetLen);
NTSTATUS WINAPI NtQuerySystemInformation(ULONG InfoClass, PVOID Info, ULONG Len,
                                         PULONG RetLen);
NTSTATUS WINAPI NtOpenKey(PHANDLE KeyHandle, ACCESS_MASK DesiredAccess,
                          PVOID ObjectAttributes);
NTSTATUS WINAPI NtOpenDirectoryObject(PHANDLE Handle, ACCESS_MASK Access,
                                      PVOID ObjectAttributes);
NTSTATUS WINAPI NtOpenSymbolicLinkObject(PHANDLE Handle, ACCESS_MASK Access,
                                         PVOID ObjectAttributes);
NTSTATUS WINAPI NtOpenSection(PHANDLE Handle, ACCESS_MASK Access,
                              PVOID ObjectAttributes);
NTSTATUS WINAPI NtQuerySymbolicLinkObject(HANDLE LinkHandle, PVOID LinkTarget,
                                          PULONG ReturnedLength);
NTSTATUS WINAPI NtAllocateVirtualMemory(HANDLE Process, PVOID *BaseAddress,
                                        ULONG_PTR ZeroBits, PSIZE_T RegionSize,
                                        ULONG AllocationType, ULONG Protect);
NTSTATUS WINAPI NtFreeVirtualMemory(HANDLE Process, PVOID *BaseAddress,
                                    PSIZE_T RegionSize, ULONG FreeType);
NTSTATUS WINAPI NtProtectVirtualMemory(HANDLE Process, PVOID *BaseAddress,
                                       PSIZE_T RegionSize, ULONG NewProtect,
                                       PULONG OldProtect);
NTSTATUS WINAPI NtMapViewOfSection(HANDLE Section, HANDLE Process,
                                   PVOID *BaseAddress, ULONG_PTR ZeroBits,
                                   SIZE_T CommitSize, PLARGE_INTEGER SectionOffset,
                                   PSIZE_T ViewSize, ULONG InheritDisposition,
                                   ULONG AllocationType, ULONG Win32Protect);
NTSTATUS WINAPI NtUnmapViewOfSection(HANDLE Process, PVOID BaseAddress);
NTSTATUS WINAPI NtOpenFile(PHANDLE Handle, ACCESS_MASK Access, PVOID Oa,
                           PVOID Iosb, ULONG ShareAccess, ULONG OpenOptions);
NTSTATUS WINAPI NtCreateFile(PHANDLE Handle, ACCESS_MASK Access, PVOID Oa,
                             PVOID Iosb, PLARGE_INTEGER AllocationSize,
                             ULONG FileAttributes, ULONG ShareAccess,
                             ULONG CreateDisposition, ULONG CreateOptions,
                             PVOID EaBuffer, ULONG EaLength);
NTSTATUS WINAPI NtCreateSection(PHANDLE Handle, ACCESS_MASK Access, PVOID Oa,
                                PLARGE_INTEGER MaximumSize, ULONG PageProtection,
                                ULONG AllocationAttributes, HANDLE FileHandle);
NTSTATUS WINAPI NtQueryAttributesFile(PVOID Oa, PVOID FileInformation);
NTSTATUS WINAPI NtQuerySection(HANDLE Section, ULONG InfoClass, PVOID Info,
                               SIZE_T Length, PSIZE_T ReturnLength);
NTSTATUS WINAPI NtCreateKeyedEvent(PHANDLE Handle, ACCESS_MASK Access, PVOID Oa,
                                   ULONG Flags);
NTSTATUS WINAPI NtCreateDirectoryObject(PHANDLE Handle, ACCESS_MASK Access,
                                        PVOID Oa);
NTSTATUS WINAPI NtCreateEvent(PHANDLE Handle, ACCESS_MASK Access, PVOID Oa,
                              ULONG EventType, BOOLEAN InitialState);
NTSTATUS WINAPI NtOpenEvent(PHANDLE Handle, ACCESS_MASK Access, PVOID Oa);
NTSTATUS WINAPI NtSecureConnectPort(PHANDLE PortHandle, PVOID PortName,
                                    PVOID SecurityQos, PVOID ClientView,
                                    PVOID ServerSid, PVOID ServerView,
                                    PULONG MaxMessageLength,
                                    PVOID ConnectionInformation,
                                    PULONG ConnectionInformationLength);
NTSTATUS WINAPI NtConnectPort(PHANDLE PortHandle, PVOID PortName,
                              PVOID SecurityQos, PVOID ClientView,
                              PVOID ServerView, PULONG MaxMessageLength,
                              PVOID ConnectionInformation,
                              PULONG ConnectionInformationLength);
NTSTATUS WINAPI NtQueryFullAttributesFile(PVOID Oa, PVOID FileInformation);

/* ---- shared 32->64 struct marshalling ----------------------------------- *
 * The guest passes 32-bit structs whose pointer/ULONG_PTR members are half the
 * host width. These helpers rebuild the 64-bit form on the host stack. Used by
 * every OBJECT_ATTRIBUTES-taking syscall (the bulk of the manual list). */

typedef struct _WOW64_UNICODE_STRING32_M {
    USHORT Length;
    USHORT MaximumLength;
    ULONG Buffer;
} WOW64_UNICODE_STRING32_M;

typedef struct _WOW64_UNICODE_STRING64_M {
    USHORT Length;
    USHORT MaximumLength;
    ULONG __pad;
    ULONGLONG Buffer;
} WOW64_UNICODE_STRING64_M;

typedef struct _WOW64_OBJECT_ATTRIBUTES32_M {
    ULONG Length;
    ULONG RootDirectory;
    ULONG ObjectName;          /* -> UNICODE_STRING32 */
    ULONG Attributes;
    ULONG SecurityDescriptor;
    ULONG SecurityQualityOfService;
} WOW64_OBJECT_ATTRIBUTES32_M;

typedef struct _WOW64_OBJECT_ATTRIBUTES64_M {
    ULONG Length;
    ULONG __pad0;
    ULONGLONG RootDirectory;
    ULONGLONG ObjectName;      /* -> UNICODE_STRING64 */
    ULONG Attributes;
    ULONG __pad1;
    ULONGLONG SecurityDescriptor;
    ULONGLONG SecurityQualityOfService;
} WOW64_OBJECT_ATTRIBUTES64_M;

/* Build an OBJECT_ATTRIBUTES64 (+ its nested UNICODE_STRING64) from a guest
 * OBJECT_ATTRIBUTES32. Uname64 is caller-provided scratch for the name. */
static void Wow64pMarshalOa(WOW64_OBJECT_ATTRIBUTES64_M *Oa64,
                            WOW64_UNICODE_STRING64_M *Uname64,
                            ULONG Oa32Ptr)
{
    WOW64_OBJECT_ATTRIBUTES32_M *Oa32 = (WOW64_OBJECT_ATTRIBUTES32_M *)(ULONG_PTR)Oa32Ptr;
    Oa64->Length = sizeof(*Oa64);
    Oa64->__pad0 = 0;
    Oa64->RootDirectory = (LONG_PTR)(LONG)Oa32->RootDirectory; /* handle: sign-ext */
    Oa64->Attributes = Oa32->Attributes;
    Oa64->__pad1 = 0;
    Oa64->SecurityDescriptor = (ULONG_PTR)Oa32->SecurityDescriptor;
    Oa64->SecurityQualityOfService = (ULONG_PTR)Oa32->SecurityQualityOfService;
    Oa64->ObjectName = 0;
    if (Oa32->ObjectName)
    {
        WOW64_UNICODE_STRING32_M *N32 = (WOW64_UNICODE_STRING32_M *)(ULONG_PTR)Oa32->ObjectName;
        Uname64->Length = N32->Length;
        Uname64->MaximumLength = N32->MaximumLength;
        Uname64->__pad = 0;
        Uname64->Buffer = (ULONG_PTR)N32->Buffer;
        Oa64->ObjectName = (ULONGLONG)(ULONG_PTR)Uname64;
    }
}

/* ---- guest (32-bit) struct shapes --------------------------------------- */
typedef struct _WOW64_MEMORY_BASIC_INFORMATION32 {
    ULONG BaseAddress;
    ULONG AllocationBase;
    ULONG AllocationProtect;
    ULONG RegionSize;
    ULONG State;
    ULONG Protect;
    ULONG Type;
} WOW64_MEMORY_BASIC_INFORMATION32;

typedef struct _WOW64_MEMORY_BASIC_INFORMATION64 {
    ULONGLONG BaseAddress;
    ULONGLONG AllocationBase;
    ULONG AllocationProtect;
    ULONG __align0;
    ULONGLONG RegionSize;
    ULONG State;
    ULONG Protect;
    ULONG Type;
    ULONG __align1;
} WOW64_MEMORY_BASIC_INFORMATION64;

#define WOW64_MEMORY_BASIC_INFORMATION_CLASS 0 /* MemoryBasicInformation */

/* NtQueryVirtualMemory: MemoryBasicInformation (class 0) is the loader's
 * bread-and-butter query and the one whose struct differs by width. Other
 * classes (MappedFilename, WorkingSet, ...) return variable/string data — pass
 * those to the host unchanged for now and widen the length so they don't hit a
 * spurious INFO_LENGTH_MISMATCH; revisit per-class when a caller needs one. */
static NTSTATUS Wow64Manual_NtQueryVirtualMemory(ULONG *a)
{
    HANDLE Process = (HANDLE)(LONG_PTR)(LONG)a[0];
    PVOID Base = (PVOID)(ULONG_PTR)a[1];
    ULONG Class = a[2];
    PVOID Buf32 = (PVOID)(ULONG_PTR)a[3];
    ULONG Len32 = a[4];
    ULONG *RetLen32 = (ULONG *)(ULONG_PTR)a[5];

    if (Class == WOW64_MEMORY_BASIC_INFORMATION_CLASS)
    {
        WOW64_MEMORY_BASIC_INFORMATION64 Mbi64 = {0};
        WOW64_MEMORY_BASIC_INFORMATION32 *Mbi32 = Buf32;
        SIZE_T Ret = 0;
        NTSTATUS St;

        if (Len32 < sizeof(*Mbi32))
        {
            if (RetLen32) *RetLen32 = sizeof(*Mbi32);
            return (NTSTATUS)0xC0000004L; /* STATUS_INFO_LENGTH_MISMATCH */
        }
        St = NtQueryVirtualMemory(Process, Base, WOW64_MEMORY_BASIC_INFORMATION_CLASS,
                                  &Mbi64, sizeof(Mbi64), &Ret);
        if (St >= 0 && Mbi32)
        {
            Mbi32->BaseAddress = (ULONG)Mbi64.BaseAddress;
            Mbi32->AllocationBase = (ULONG)Mbi64.AllocationBase;
            Mbi32->AllocationProtect = Mbi64.AllocationProtect;
            Mbi32->RegionSize = (ULONG)Mbi64.RegionSize;
            Mbi32->State = Mbi64.State;
            Mbi32->Protect = Mbi64.Protect;
            Mbi32->Type = Mbi64.Type;
            if (RetLen32) *RetLen32 = sizeof(*Mbi32);
        }
        return St;
    }

    /* Non-MBI class: value/string data, layout-compatible enough to pass
     * through (guest pointer zero-extends; length as-is). */
    {
        SIZE_T Ret = 0;
        NTSTATUS St = NtQueryVirtualMemory(Process, Base, Class, Buf32, Len32, &Ret);
        if (RetLen32) *RetLen32 = (ULONG)Ret;
        return St;
    }
}

/* ---- NtQueryInformationProcess ------------------------------------------ */
typedef struct _WOW64_PROCESS_BASIC_INFORMATION32 {
    NTSTATUS ExitStatus;
    ULONG PebBaseAddress;   /* MUST be PEB32, not PEB64 */
    ULONG AffinityMask;
    LONG BasePriority;
    ULONG UniqueProcessId;
    ULONG InheritedFromUniqueProcessId;
} WOW64_PROCESS_BASIC_INFORMATION32;

typedef struct _WOW64_PROCESS_BASIC_INFORMATION64 {
    NTSTATUS ExitStatus;
    ULONG __pad;
    ULONGLONG PebBaseAddress;
    ULONGLONG AffinityMask;
    LONG BasePriority;
    ULONG __pad2;
    ULONGLONG UniqueProcessId;
    ULONGLONG InheritedFromUniqueProcessId;
} WOW64_PROCESS_BASIC_INFORMATION64;

#define WOW64_PROCESS_CURRENT ((HANDLE)(LONG_PTR)-1)
#define ProcessBasicInformation  0
#define ProcessWow64Information   26

static NTSTATUS Wow64Manual_NtQueryInformationProcess(ULONG *a)
{
    HANDLE Process = (HANDLE)(LONG_PTR)(LONG)a[0];
    ULONG Class = a[1];
    PVOID Buf32 = (PVOID)(ULONG_PTR)a[2];
    ULONG Len32 = a[3];
    ULONG *RetLen32 = (ULONG *)(ULONG_PTR)a[4];

    if (Class == ProcessBasicInformation)
    {
        WOW64_PROCESS_BASIC_INFORMATION64 Pbi64 = {0};
        WOW64_PROCESS_BASIC_INFORMATION32 *Pbi32 = Buf32;
        ULONG_PTR Peb32 = 0;
        ULONG Ret64 = 0;
        NTSTATUS St;
        if (Len32 < sizeof(*Pbi32))
        {
            if (RetLen32) *RetLen32 = sizeof(*Pbi32);
            return (NTSTATUS)0xC0000004L;
        }
        St = NtQueryInformationProcess(Process, ProcessBasicInformation,
                                       &Pbi64, sizeof(Pbi64), &Ret64);
        if (St < 0) return St;
        /* PebBaseAddress must be the 32-bit PEB — query it explicitly. */
        NtQueryInformationProcess(Process, ProcessWow64Information,
                                  &Peb32, sizeof(Peb32), NULL);
        Pbi32->ExitStatus = Pbi64.ExitStatus;
        Pbi32->PebBaseAddress = (ULONG)Peb32;
        Pbi32->AffinityMask = (ULONG)Pbi64.AffinityMask;
        Pbi32->BasePriority = Pbi64.BasePriority;
        Pbi32->UniqueProcessId = (ULONG)Pbi64.UniqueProcessId;
        Pbi32->InheritedFromUniqueProcessId = (ULONG)Pbi64.InheritedFromUniqueProcessId;
        if (RetLen32) *RetLen32 = sizeof(*Pbi32);
        return St;
    }
    if (Class == ProcessWow64Information)
    {
        /* PEB32 pointer, 32-bit wide to the guest. */
        ULONG_PTR Peb32 = 0;
        NTSTATUS St = NtQueryInformationProcess(Process, ProcessWow64Information,
                                                &Peb32, sizeof(Peb32), NULL);
        if (St >= 0 && Buf32 && Len32 >= sizeof(ULONG))
        {
            *(ULONG *)Buf32 = (ULONG)Peb32;
            if (RetLen32) *RetLen32 = sizeof(ULONG);
        }
        return St;
    }
    /* Scalar / layout-identical classes (ProcessDefaultHardErrorMode,
     * ProcessCookie, ...): pass the guest buffer straight through. Log the
     * class so a pointer-bearing one we haven't special-cased is visible. */
    {
        ULONG Ret = 0;
        NTSTATUS St = NtQueryInformationProcess(Process, Class, Buf32, Len32, &Ret);
        if (RetLen32) *RetLen32 = Ret;
        DbgPrint("[WOW64]   NtQueryInformationProcess class=%lu passthrough => %08lx (len=%lu)\n",
                 Class, St, Len32);
        return St;
    }
}

/* ---- NtQuerySystemInformation ------------------------------------------- */
typedef struct _WOW64_SYSTEM_BASIC_INFORMATION32 {
    ULONG Reserved;
    ULONG TimerResolution;
    ULONG PageSize;
    ULONG NumberOfPhysicalPages;
    ULONG LowestPhysicalPageNumber;
    ULONG HighestPhysicalPageNumber;
    ULONG AllocationGranularity;
    ULONG MinimumUserModeAddress;
    ULONG MaximumUserModeAddress;
    ULONG ActiveProcessorsAffinityMask;
    UCHAR NumberOfProcessors;
} WOW64_SYSTEM_BASIC_INFORMATION32;

typedef struct _WOW64_SYSTEM_BASIC_INFORMATION64 {
    ULONG Reserved;
    ULONG TimerResolution;
    ULONG PageSize;
    ULONG NumberOfPhysicalPages;
    ULONG LowestPhysicalPageNumber;
    ULONG HighestPhysicalPageNumber;
    ULONG AllocationGranularity;
    ULONG __pad;
    ULONGLONG MinimumUserModeAddress;
    ULONGLONG MaximumUserModeAddress;
    ULONGLONG ActiveProcessorsAffinityMask;
    UCHAR NumberOfProcessors;
} WOW64_SYSTEM_BASIC_INFORMATION64;

#define SystemBasicInformation 0
#define SystemEmulationBasicInformation 62

static NTSTATUS Wow64Manual_NtQuerySystemInformation(ULONG *a)
{
    ULONG Class = a[0];
    PVOID Buf32 = (PVOID)(ULONG_PTR)a[1];
    ULONG Len32 = a[2];
    ULONG *RetLen32 = (ULONG *)(ULONG_PTR)a[3];

    if (Class == SystemBasicInformation || Class == SystemEmulationBasicInformation)
    {
        WOW64_SYSTEM_BASIC_INFORMATION64 Sbi64 = {0};
        WOW64_SYSTEM_BASIC_INFORMATION32 *Sbi32 = Buf32;
        ULONG Ret = 0;
        NTSTATUS St;
        if (Len32 < sizeof(*Sbi32))
        {
            if (RetLen32) *RetLen32 = sizeof(*Sbi32);
            return (NTSTATUS)0xC0000004L;
        }
        St = NtQuerySystemInformation(Class, &Sbi64, sizeof(Sbi64), &Ret);
        if (St < 0) return St;
        Sbi32->Reserved = Sbi64.Reserved;
        Sbi32->TimerResolution = Sbi64.TimerResolution;
        Sbi32->PageSize = Sbi64.PageSize;
        Sbi32->NumberOfPhysicalPages = Sbi64.NumberOfPhysicalPages;
        Sbi32->LowestPhysicalPageNumber = Sbi64.LowestPhysicalPageNumber;
        Sbi32->HighestPhysicalPageNumber = Sbi64.HighestPhysicalPageNumber;
        Sbi32->AllocationGranularity = Sbi64.AllocationGranularity;
        Sbi32->MinimumUserModeAddress = (ULONG)Sbi64.MinimumUserModeAddress;
        /* Clamp the max user address to the 32-bit guest's 2GB ceiling. */
        Sbi32->MaximumUserModeAddress =
            Sbi64.MaximumUserModeAddress > 0x7FFEFFFFull ? 0x7FFEFFFFu
                                                         : (ULONG)Sbi64.MaximumUserModeAddress;
        Sbi32->ActiveProcessorsAffinityMask = (ULONG)Sbi64.ActiveProcessorsAffinityMask;
        Sbi32->NumberOfProcessors = Sbi64.NumberOfProcessors;
        if (RetLen32) *RetLen32 = sizeof(*Sbi32);
        return St;
    }
    {
        ULONG Ret = 0;
        NTSTATUS St = NtQuerySystemInformation(Class, Buf32, Len32, &Ret);
        if (RetLen32) *RetLen32 = Ret;
        DbgPrint("[WOW64]   NtQuerySystemInformation class=%lu passthrough => %08lx (len=%lu)\n",
                 Class, St, Len32);
        return St;
    }
}

/* ---- generic (PHANDLE, ACCESS_MASK, POBJECT_ATTRIBUTES) opens ------------ *
 * NtOpenKey/OpenDirectoryObject/OpenSymbolicLinkObject/OpenSection/OpenEvent/
 * ... all share this shape: marshal the OA (and its nested UNICODE_STRING),
 * call the host, write the 64-bit handle back into the guest's 32-bit cell. */
typedef NTSTATUS (WINAPI *WOW64_OPEN_FN)(PHANDLE, ACCESS_MASK, PVOID);

static NTSTATUS Wow64pOpenByOa(WOW64_OPEN_FN Fn, ULONG *a)
{
    ULONG *Handle32 = (ULONG *)(ULONG_PTR)a[0];
    ACCESS_MASK Access = a[1];
    WOW64_OBJECT_ATTRIBUTES64_M Oa64;
    WOW64_UNICODE_STRING64_M Uname64;
    HANDLE Handle = NULL;
    NTSTATUS St;

    Wow64pMarshalOa(&Oa64, &Uname64, a[2]);
    St = Fn(&Handle, Access, &Oa64);
    if (St >= 0 && Handle32)
        *Handle32 = (ULONG)(ULONG_PTR)Handle;
    return St;
}

/* ---- file syscalls (OBJECT_ATTRIBUTES + IO_STATUS_BLOCK) ----------------- *
 * IO_STATUS_BLOCK32 is 8 bytes ({ULONG Status; ULONG Information}); the host
 * writes 16 ({union; ULONG_PTR}). Use a host-side IOSB and copy the two fields
 * back down. FILE_BASIC/NETWORK_OPEN_INFORMATION are width-identical (all
 * LARGE_INTEGER/ULONG) so those pass through. */
typedef struct _WOW64_IO_STATUS_BLOCK32_M {
    ULONG Status;
    ULONG Information;
} WOW64_IO_STATUS_BLOCK32_M;

typedef struct _WOW64_IO_STATUS_BLOCK64_M {
    ULONGLONG Status;       /* union {NTSTATUS; PVOID} — low 4 bytes hold status */
    ULONGLONG Information;
} WOW64_IO_STATUS_BLOCK64_M;

/* WOW64 filesystem redirection: rewrite "\system32\" -> "\syswow64\" (same
 * length) so guest file opens land on the 32-bit DLL set. Real wow64.dll does
 * exactly this in its file-syscall thunks. The rewrite goes into caller
 * scratch — guest memory is never modified. */
#define WOW64_REDIR_MAX 260

static void Wow64pRedirectPath(WOW64_UNICODE_STRING64_M *Name64, WCHAR *Scratch)
{
    static const WCHAR Sys32[] = {'\\','s','y','s','t','e','m','3','2','\\',0};
    static const WCHAR Wow64[] = {'\\','s','y','s','w','o','w','6','4','\\',0};
    const WCHAR *Src = (const WCHAR *)(ULONG_PTR)Name64->Buffer;
    ULONG Chars = Name64->Length / sizeof(WCHAR);
    ULONG i, j;
    LONG Hit = -1;

    if (!Src || Chars == 0 || Chars > WOW64_REDIR_MAX)
        return;

    for (i = 0; i + 10 <= Chars; i++)
    {
        for (j = 0; j < 10; j++)
        {
            WCHAR c = Src[i + j];
            if (c >= 'A' && c <= 'Z') c += 32;
            if (c != Sys32[j]) break;
        }
        if (j == 10) { Hit = (LONG)i; break; }
    }
    if (Hit < 0)
        return;

    for (i = 0; i < Chars; i++)
        Scratch[i] = Src[i];
    for (j = 0; j < 10; j++)
        Scratch[Hit + j] = Wow64[j];
    Name64->Buffer = (ULONG_PTR)Scratch;
}

static void Wow64pCopyIosbDown(WOW64_IO_STATUS_BLOCK64_M *I64, ULONG Iosb32Ptr)
{
    WOW64_IO_STATUS_BLOCK32_M *I32 = (WOW64_IO_STATUS_BLOCK32_M *)(ULONG_PTR)Iosb32Ptr;
    if (!I32) return;
    I32->Status = (ULONG)I64->Status;
    I32->Information = (ULONG)I64->Information;
}

static NTSTATUS Wow64Manual_NtOpenFile(ULONG *a)
{
    ULONG *Handle32 = (ULONG *)(ULONG_PTR)a[0];
    ACCESS_MASK Access = a[1];
    WOW64_OBJECT_ATTRIBUTES64_M Oa64;
    WOW64_UNICODE_STRING64_M Uname64;
    WCHAR RedirScratch[WOW64_REDIR_MAX];
    WOW64_IO_STATUS_BLOCK64_M Iosb64 = {0};
    HANDLE Handle = NULL;
    NTSTATUS St;

    Wow64pMarshalOa(&Oa64, &Uname64, a[2]);
    Wow64pRedirectPath(&Uname64, RedirScratch);
    St = NtOpenFile(&Handle, Access, &Oa64, &Iosb64, a[4], a[5]);
    if (St < 0 && Oa64.ObjectName)
        DbgPrint("[WOW64]   NtOpenFile '%.*S' => %08lx\n",
                 (int)(Uname64.Length / sizeof(WCHAR)),
                 (PCWSTR)(ULONG_PTR)Uname64.Buffer, St);
    Wow64pCopyIosbDown(&Iosb64, a[3]);
    if (St >= 0 && Handle32)
        *Handle32 = (ULONG)(ULONG_PTR)Handle;
    return St;
}

static NTSTATUS Wow64Manual_NtCreateFile(ULONG *a)
{
    ULONG *Handle32 = (ULONG *)(ULONG_PTR)a[0];
    ACCESS_MASK Access = a[1];
    WOW64_OBJECT_ATTRIBUTES64_M Oa64;
    WOW64_UNICODE_STRING64_M Uname64;
    WCHAR RedirScratch[WOW64_REDIR_MAX];
    WOW64_IO_STATUS_BLOCK64_M Iosb64 = {0};
    PLARGE_INTEGER AllocSize = (PLARGE_INTEGER)(ULONG_PTR)a[4]; /* layout-identical */
    HANDLE Handle = NULL;
    NTSTATUS St;

    Wow64pMarshalOa(&Oa64, &Uname64, a[2]);
    Wow64pRedirectPath(&Uname64, RedirScratch);
    St = NtCreateFile(&Handle, Access, &Oa64, &Iosb64, AllocSize,
                      a[5] /*FileAttributes*/, a[6] /*ShareAccess*/,
                      a[7] /*CreateDisposition*/, a[8] /*CreateOptions*/,
                      (PVOID)(ULONG_PTR)a[9] /*EaBuffer*/, a[10] /*EaLength*/);
    Wow64pCopyIosbDown(&Iosb64, a[3]);
    if (St >= 0 && Handle32)
        *Handle32 = (ULONG)(ULONG_PTR)Handle;
    return St;
}

static NTSTATUS Wow64Manual_NtCreateSection(ULONG *a)
{
    ULONG *Handle32 = (ULONG *)(ULONG_PTR)a[0];
    ACCESS_MASK Access = a[1];
    WOW64_OBJECT_ATTRIBUTES64_M Oa64;
    WOW64_UNICODE_STRING64_M Uname64;
    PVOID Oa64Ptr = NULL;
    PLARGE_INTEGER MaxSize = (PLARGE_INTEGER)(ULONG_PTR)a[3]; /* layout-identical */
    HANDLE Handle = NULL;
    NTSTATUS St;

    if (a[2]) /* OBJECT_ATTRIBUTES is optional for anonymous sections */
    {
        Wow64pMarshalOa(&Oa64, &Uname64, a[2]);
        Oa64Ptr = &Oa64;
    }
    St = NtCreateSection(&Handle, Access, Oa64Ptr, MaxSize,
                         a[4] /*SectionPageProtection*/, a[5] /*AllocationAttributes*/,
                         (HANDLE)(LONG_PTR)(LONG)a[6] /*FileHandle*/);
    if (St >= 0 && Handle32)
        *Handle32 = (ULONG)(ULONG_PTR)Handle;
    return St;
}

static NTSTATUS Wow64Manual_NtQueryAttributesFile(ULONG *a)
{
    WOW64_OBJECT_ATTRIBUTES64_M Oa64;
    WOW64_UNICODE_STRING64_M Uname64;
    WCHAR RedirScratch[WOW64_REDIR_MAX];

    Wow64pMarshalOa(&Oa64, &Uname64, a[0]);
    Wow64pRedirectPath(&Uname64, RedirScratch);
    /* FILE_BASIC_INFORMATION: 4x LARGE_INTEGER + ULONG — width-identical */
    return NtQueryAttributesFile(&Oa64, (PVOID)(ULONG_PTR)a[1]);
}

static NTSTATUS Wow64Manual_NtQueryFullAttributesFile(ULONG *a)
{
    WOW64_OBJECT_ATTRIBUTES64_M Oa64;
    WOW64_UNICODE_STRING64_M Uname64;
    WCHAR RedirScratch[WOW64_REDIR_MAX];

    Wow64pMarshalOa(&Oa64, &Uname64, a[0]);
    Wow64pRedirectPath(&Uname64, RedirScratch);
    /* FILE_NETWORK_OPEN_INFORMATION: LARGE_INTEGERs + ULONGs — width-identical */
    return NtQueryFullAttributesFile(&Oa64, (PVOID)(ULONG_PTR)a[1]);
}

/* ---- virtual-memory syscalls with in/out 32-bit cells ------------------- *
 * BaseAddress and RegionSize are POINTERS to guest 32-bit cells; the host Nt*
 * reads/writes 64-bit cells. The auto thunk would hand the host a 4-byte guest
 * cell to treat as 8 bytes — clobbering adjacent memory and reading garbage
 * (the INVALID_PARAMETER_2 we saw). These marshal the cells by value. */
#define WOW64_ZEROBITS_BELOW_2G ((ULONG_PTR)0x7FFFFFFF)

static NTSTATUS Wow64Manual_NtAllocateVirtualMemory(ULONG *a)
{
    HANDLE Process = (HANDLE)(LONG_PTR)(LONG)a[0];
    ULONG *Base32 = (ULONG *)(ULONG_PTR)a[1];
    ULONG_PTR ZeroBits = a[2];
    ULONG *Size32 = (ULONG *)(ULONG_PTR)a[3];
    ULONG Type = a[4], Protect = a[5];
    PVOID Base = Base32 ? (PVOID)(ULONG_PTR)*Base32 : NULL;
    SIZE_T Size = Size32 ? *Size32 : 0;
    NTSTATUS St;

    /* Keep guest allocations addressable from 32-bit: when the guest lets the
     * system choose (Base==0), force the result below 2GB (kernel ZeroBits>32
     * mask semantics). */
    if (!Base && ZeroBits == 0)
        ZeroBits = WOW64_ZEROBITS_BELOW_2G;

    St = NtAllocateVirtualMemory(Process, &Base, ZeroBits, &Size, Type, Protect);
    if (St >= 0)
    {
        if (Base32) *Base32 = (ULONG)(ULONG_PTR)Base;
        if (Size32) *Size32 = (ULONG)Size;
    }
    return St;
}

static NTSTATUS Wow64Manual_NtFreeVirtualMemory(ULONG *a)
{
    HANDLE Process = (HANDLE)(LONG_PTR)(LONG)a[0];
    ULONG *Base32 = (ULONG *)(ULONG_PTR)a[1];
    ULONG *Size32 = (ULONG *)(ULONG_PTR)a[2];
    ULONG FreeType = a[3];
    PVOID Base = Base32 ? (PVOID)(ULONG_PTR)*Base32 : NULL;
    SIZE_T Size = Size32 ? *Size32 : 0;
    NTSTATUS St = NtFreeVirtualMemory(Process, &Base, &Size, FreeType);
    if (St >= 0)
    {
        if (Base32) *Base32 = (ULONG)(ULONG_PTR)Base;
        if (Size32) *Size32 = (ULONG)Size;
    }
    return St;
}

static NTSTATUS Wow64Manual_NtProtectVirtualMemory(ULONG *a)
{
    HANDLE Process = (HANDLE)(LONG_PTR)(LONG)a[0];
    ULONG *Base32 = (ULONG *)(ULONG_PTR)a[1];
    ULONG *Size32 = (ULONG *)(ULONG_PTR)a[2];
    ULONG NewProtect = a[3];
    ULONG *Old32 = (ULONG *)(ULONG_PTR)a[4];
    PVOID Base = Base32 ? (PVOID)(ULONG_PTR)*Base32 : NULL;
    SIZE_T Size = Size32 ? *Size32 : 0;
    ULONG Old = 0;
    NTSTATUS St = NtProtectVirtualMemory(Process, &Base, &Size, NewProtect, &Old);
    if (St >= 0)
    {
        if (Base32) *Base32 = (ULONG)(ULONG_PTR)Base;
        if (Size32) *Size32 = (ULONG)Size;
        if (Old32) *Old32 = Old;
    }
    return St;
}

/* ---- NtMapViewOfSection -------------------------------------------------- *
 * BaseAddress and ViewSize are guest 32-bit in/out cells; SectionOffset is a
 * guest pointer to an 8-byte LARGE_INTEGER (same layout both widths — pass the
 * zero-extended pointer straight through). Same below-2GB ZeroBits forcing as
 * NtAllocateVirtualMemory so system-chosen views stay guest-addressable. */
static NTSTATUS Wow64Manual_NtMapViewOfSection(ULONG *a)
{
    HANDLE Section = (HANDLE)(LONG_PTR)(LONG)a[0];
    HANDLE Process = (HANDLE)(LONG_PTR)(LONG)a[1];
    ULONG *Base32 = (ULONG *)(ULONG_PTR)a[2];
    ULONG_PTR ZeroBits = a[3];
    SIZE_T CommitSize = a[4];
    PLARGE_INTEGER SectionOffset = (PLARGE_INTEGER)(ULONG_PTR)a[5];
    ULONG *Size32 = (ULONG *)(ULONG_PTR)a[6];
    ULONG Inherit = a[7], Type = a[8], Protect = a[9];
    PVOID Base = Base32 ? (PVOID)(ULONG_PTR)*Base32 : NULL;
    SIZE_T Size = Size32 ? *Size32 : 0;
    NTSTATUS St;

    if (!Base && ZeroBits == 0)
        ZeroBits = WOW64_ZEROBITS_BELOW_2G;

    St = NtMapViewOfSection(Section, Process, &Base, ZeroBits, CommitSize,
                            SectionOffset, &Size, Inherit, Type, Protect);
    if (St >= 0)
    {
        if (Base32) *Base32 = (ULONG)(ULONG_PTR)Base;
        if (Size32) *Size32 = (ULONG)Size;
        DbgPrint("[WOW64]   NtMapViewOfSection => %08lx base=%p size=%p\n",
                 St, Base, (PVOID)Size);
        /* Tell the CPU layer about the new mapping (real wow64 notifies on
         * every map) — without this FEX won't translate code from the range. */
        {
            extern LONG (WINAPI *Wow64pBTCpuNotifyMap)(PVOID, PVOID, PVOID,
                                                       SIZE_T, ULONG, ULONG);
            extern VOID Wow64pPatchTransitionSlot(PVOID MBase, SIZE_T MSize);
            if (Wow64pBTCpuNotifyMap)
                Wow64pBTCpuNotifyMap(NULL, Base, NULL, Size, Type, Protect);
            /* Guest DLLs carrying their own win32ksys syscall stubs
             * (gdi32/user32) export a per-module Wow64Transition slot that
             * must point at the BOP, same as ntdll32's. */
            Wow64pPatchTransitionSlot(Base, Size);
        }
    }
    else
    {
        DbgPrint("[WOW64] NtMapViewOfSection FAIL %08lx: sec=%p proc=%p base=%p "
                 "zero=%p commit=%p offp=%p off=%I64x size=%p inherit=%lu type=%08lx prot=%08lx\n",
                 St, Section, Process, Base, (PVOID)ZeroBits, (PVOID)CommitSize,
                 SectionOffset, SectionOffset ? SectionOffset->QuadPart : 0,
                 (PVOID)Size, Inherit, Type, Protect);
    }
    return St;
}

/* ---- NtConnectPort / NtSecureConnectPort --------------------------------- *
 * The CSR connect (kernel32 -> csrss). PHANDLE + PORT_VIEW/REMOTE_PORT_VIEW
 * widen; SECURITY_QUALITY_OF_SERVICE is pointer-free (12 bytes both widths)
 * and passes through. The port views are in/out: the kernel returns the
 * mapped view bases/sizes, copied back down to the guest cells. */
typedef struct _WOW64_PORT_VIEW32_M {
    ULONG Length;
    ULONG SectionHandle;
    ULONG SectionOffset;
    ULONG ViewSize;
    ULONG ViewBase;
    ULONG ViewRemoteBase;
} WOW64_PORT_VIEW32_M;

typedef struct _WOW64_PORT_VIEW64_M {
    ULONG Length;
    ULONG __pad0;
    ULONGLONG SectionHandle;
    ULONG SectionOffset;
    ULONG __pad1;
    ULONGLONG ViewSize;
    ULONGLONG ViewBase;
    ULONGLONG ViewRemoteBase;
} WOW64_PORT_VIEW64_M;

typedef struct _WOW64_REMOTE_PORT_VIEW32_M {
    ULONG Length;
    ULONG ViewSize;
    ULONG ViewBase;
} WOW64_REMOTE_PORT_VIEW32_M;

typedef struct _WOW64_REMOTE_PORT_VIEW64_M {
    ULONG Length;
    ULONG __pad0;
    ULONGLONG ViewSize;
    ULONGLONG ViewBase;
} WOW64_REMOTE_PORT_VIEW64_M;

/* CSR_API_CONNECTINFO widths (csrmsg.h): the 32-bit client sends 0x24 bytes,
 * 64-bit csrss reads/writes 0x38. Field-for-field thunk; the three returned
 * pointers (SharedSectionBase/StaticServerData/Heap) must land below 4GB for
 * the guest to use them — loud print if csrss's view base is guest-invisible. */
typedef struct _WOW64_CSR_CONNECTINFO32_M {
    ULONG ObjectDirectory;
    ULONG SharedSectionBase;
    ULONG SharedStaticServerData;
    ULONG SharedSectionHeap;
    ULONG DebugFlags;
    ULONG SizeOfPebData;
    ULONG SizeOfTebData;
    ULONG NumberOfServerDllNames;
    ULONG ServerProcessId;
} WOW64_CSR_CONNECTINFO32_M; /* 0x24 */

typedef struct _WOW64_CSR_CONNECTINFO64_M {
    ULONGLONG ObjectDirectory;
    ULONGLONG SharedSectionBase;
    ULONGLONG SharedStaticServerData;
    ULONGLONG SharedSectionHeap;
    ULONG DebugFlags;
    ULONG SizeOfPebData;
    ULONG SizeOfTebData;
    ULONG NumberOfServerDllNames;
    ULONGLONG ServerProcessId;
} WOW64_CSR_CONNECTINFO64_M; /* 0x38 */

static NTSTATUS Wow64pConnectPort(ULONG *a, int Secure)
{
    ULONG *Handle32 = (ULONG *)(ULONG_PTR)a[0];
    WOW64_UNICODE_STRING32_M *Name32 = (WOW64_UNICODE_STRING32_M *)(ULONG_PTR)a[1];
    WOW64_UNICODE_STRING64_M Name64;
    PVOID Qos = (PVOID)(ULONG_PTR)a[2]; /* pointer-free struct: passthrough */
    WOW64_PORT_VIEW32_M *Cv32 = (WOW64_PORT_VIEW32_M *)(ULONG_PTR)a[3];
    PVOID ServerSid = Secure ? (PVOID)(ULONG_PTR)a[4] : NULL;
    WOW64_REMOTE_PORT_VIEW32_M *Sv32 =
        (WOW64_REMOTE_PORT_VIEW32_M *)(ULONG_PTR)a[Secure ? 5 : 4];
    ULONG *MaxLen32 = (ULONG *)(ULONG_PTR)a[Secure ? 6 : 5];
    PVOID ConnInfo = (PVOID)(ULONG_PTR)a[Secure ? 7 : 6];
    ULONG *ConnLen32 = (ULONG *)(ULONG_PTR)a[Secure ? 8 : 7];
    WOW64_PORT_VIEW64_M Cv64, *Cv64Ptr = NULL;
    WOW64_REMOTE_PORT_VIEW64_M Sv64, *Sv64Ptr = NULL;
    WOW64_CSR_CONNECTINFO64_M Csr64;
    WOW64_CSR_CONNECTINFO32_M *Csr32 = NULL;
    ULONG MaxLen = 0, ConnLen = 0;
    HANDLE Handle = NULL;
    NTSTATUS St;

    if (!Name32)
        return (NTSTATUS)0xC000000DL;
    Name64.Length = Name32->Length;
    Name64.MaximumLength = Name32->MaximumLength;
    Name64.__pad = 0;
    Name64.Buffer = (ULONG_PTR)Name32->Buffer;

    /* CSR connect-info thunk: a 0x24-byte blob aimed at an *ApiPort is the
     * 32-bit CSR_API_CONNECTINFO — widen it so 64-bit csrss reads/writes its
     * own layout, then narrow the reply below. */
    if (ConnInfo && ConnLen32 && *ConnLen32 == sizeof(WOW64_CSR_CONNECTINFO32_M) &&
        Name64.Length >= 7 * sizeof(WCHAR))
    {
        PCWSTR N = (PCWSTR)(ULONG_PTR)Name64.Buffer;
        ULONG NChars = Name64.Length / sizeof(WCHAR);
        if (N[NChars - 7] == L'A' && N[NChars - 6] == L'p' && N[NChars - 5] == L'i' &&
            N[NChars - 4] == L'P' && N[NChars - 3] == L'o' && N[NChars - 2] == L'r' &&
            N[NChars - 1] == L't')
        {
            Csr32 = (WOW64_CSR_CONNECTINFO32_M *)ConnInfo;
            Csr64.ObjectDirectory = Csr32->ObjectDirectory;
            Csr64.SharedSectionBase = Csr32->SharedSectionBase;
            Csr64.SharedStaticServerData = Csr32->SharedStaticServerData;
            Csr64.SharedSectionHeap = Csr32->SharedSectionHeap;
            Csr64.DebugFlags = Csr32->DebugFlags;
            Csr64.SizeOfPebData = Csr32->SizeOfPebData;
            Csr64.SizeOfTebData = Csr32->SizeOfTebData;
            Csr64.NumberOfServerDllNames = Csr32->NumberOfServerDllNames;
            Csr64.ServerProcessId = Csr32->ServerProcessId;
            ConnInfo = &Csr64;
            ConnLen = sizeof(Csr64);
        }
    }

    if (Cv32)
    {
        Cv64.Length = sizeof(Cv64);
        Cv64.__pad0 = Cv64.__pad1 = 0;
        Cv64.SectionHandle = (LONG_PTR)(LONG)Cv32->SectionHandle;
        Cv64.SectionOffset = Cv32->SectionOffset;
        Cv64.ViewSize = Cv32->ViewSize;
        Cv64.ViewBase = Cv32->ViewBase;
        Cv64.ViewRemoteBase = Cv32->ViewRemoteBase;
        Cv64Ptr = &Cv64;
    }
    if (Sv32)
    {
        Sv64.Length = sizeof(Sv64);
        Sv64.__pad0 = 0;
        Sv64.ViewSize = Sv32->ViewSize;
        Sv64.ViewBase = Sv32->ViewBase;
        Sv64Ptr = &Sv64;
    }
    if (ConnLen32 && !Csr32)
        ConnLen = *ConnLen32;

    if (Secure)
        St = NtSecureConnectPort(&Handle, &Name64, Qos, Cv64Ptr, ServerSid,
                                 Sv64Ptr, MaxLen32 ? &MaxLen : NULL, ConnInfo,
                                 ConnLen32 ? &ConnLen : NULL);
    else
        St = NtConnectPort(&Handle, &Name64, Qos, Cv64Ptr, Sv64Ptr,
                           MaxLen32 ? &MaxLen : NULL, ConnInfo,
                           ConnLen32 ? &ConnLen : NULL);

    DbgPrint("[WOW64]   Nt%sConnectPort '%.*S' => %08lx (handle=%p maxlen=%lu)\n",
             Secure ? "Secure" : "", (int)(Name64.Length / sizeof(WCHAR)),
             (PCWSTR)(ULONG_PTR)Name64.Buffer, St, Handle, MaxLen);

    if (St >= 0)
    {
        if (Handle32) *Handle32 = (ULONG)(ULONG_PTR)Handle;
        if (Cv32)
        {
            Cv32->ViewSize = (ULONG)Cv64.ViewSize;
            Cv32->ViewBase = (ULONG)Cv64.ViewBase;
            Cv32->ViewRemoteBase = (ULONG)Cv64.ViewRemoteBase;
        }
        if (Sv32)
        {
            Sv32->ViewSize = (ULONG)Sv64.ViewSize;
            Sv32->ViewBase = (ULONG)Sv64.ViewBase;
        }
        if (MaxLen32) *MaxLen32 = MaxLen;
        if (Csr32)
        {
            /* Narrow the CSR reply. The shared-section pointers must be
             * guest-addressable; a >4GB base means csrss's view landed
             * outside the guest's world — report it loudly, the connect
             * "succeeded" but kernel32 would read garbage. */
            if ((Csr64.SharedSectionBase | Csr64.SharedStaticServerData |
                 Csr64.SharedSectionHeap) >> 32)
                DbgPrint("[WOW64]   CSR connect: shared section ABOVE 4GB "
                         "(base=%I64x static=%I64x heap=%I64x) — guest-invisible!\n",
                         Csr64.SharedSectionBase, Csr64.SharedStaticServerData,
                         Csr64.SharedSectionHeap);
            Csr32->ObjectDirectory = (ULONG)Csr64.ObjectDirectory;
            Csr32->SharedSectionBase = (ULONG)Csr64.SharedSectionBase;
            Csr32->SharedStaticServerData = (ULONG)Csr64.SharedStaticServerData;
            Csr32->SharedSectionHeap = (ULONG)Csr64.SharedSectionHeap;
            Csr32->DebugFlags = Csr64.DebugFlags;
            Csr32->SizeOfPebData = Csr64.SizeOfPebData;
            Csr32->SizeOfTebData = Csr64.SizeOfTebData;
            Csr32->NumberOfServerDllNames = Csr64.NumberOfServerDllNames;
            Csr32->ServerProcessId = (ULONG)Csr64.ServerProcessId;
            if (ConnLen32) *ConnLen32 = sizeof(*Csr32);
        }
        else if (ConnLen32) *ConnLen32 = ConnLen;
    }
    return St;
}

static NTSTATUS Wow64Manual_NtSecureConnectPort(ULONG *a)
{
    return Wow64pConnectPort(a, 1);
}

static NTSTATUS Wow64Manual_NtConnectPort(ULONG *a)
{
    return Wow64pConnectPort(a, 0);
}

/* ---- NtCreateKeyedEvent --------------------------------------------------- *
 * PHANDLE out-cell (4 bytes guest vs 8 host — the kernel's 8-byte-aligned
 * probe rejects the guest cell) + optional OBJECT_ATTRIBUTES. ntdll_vista's
 * DllMain creates the process keyed event through this. */
/* NtCreateEvent: PHANDLE out-cell + optional OA + two trailing scalars. */
static NTSTATUS Wow64Manual_NtCreateEvent(ULONG *a)
{
    ULONG *Handle32 = (ULONG *)(ULONG_PTR)a[0];
    ACCESS_MASK Access = a[1];
    WOW64_OBJECT_ATTRIBUTES64_M Oa64;
    WOW64_UNICODE_STRING64_M Uname64;
    PVOID Oa64Ptr = NULL;
    HANDLE Handle = NULL;
    NTSTATUS St;

    if (a[2])
    {
        Wow64pMarshalOa(&Oa64, &Uname64, a[2]);
        Oa64Ptr = &Oa64;
    }
    St = NtCreateEvent(&Handle, Access, Oa64Ptr, a[3], (BOOLEAN)a[4]);
    if (St >= 0 && Handle32)
        *Handle32 = (ULONG)(ULONG_PTR)Handle;
    return St;
}

static NTSTATUS Wow64Manual_NtCreateKeyedEvent(ULONG *a)
{
    ULONG *Handle32 = (ULONG *)(ULONG_PTR)a[0];
    ACCESS_MASK Access = a[1];
    WOW64_OBJECT_ATTRIBUTES64_M Oa64;
    WOW64_UNICODE_STRING64_M Uname64;
    PVOID Oa64Ptr = NULL;
    HANDLE Handle = NULL;
    NTSTATUS St;

    if (a[2])
    {
        Wow64pMarshalOa(&Oa64, &Uname64, a[2]);
        Oa64Ptr = &Oa64;
    }
    St = NtCreateKeyedEvent(&Handle, Access, Oa64Ptr, a[3]);
    if (St >= 0 && Handle32)
        *Handle32 = (ULONG)(ULONG_PTR)Handle;
    return St;
}

/* ---- NtQuerySection ------------------------------------------------------ *
 * SectionBasicInformation: 16 bytes guest vs 24 host (PVOID + alignment).
 * SectionImageInformation: 48 bytes guest vs 64 host (4 pointer/SIZE_T fields
 * widen). The loader queries image info for every mapped DLL. */
typedef struct _WOW64_SECTION_BASIC_INFORMATION64 {
    ULONGLONG BaseAddress;
    ULONG Attributes;
    ULONG __pad;
    ULONGLONG MaximumSize;
} WOW64_SECTION_BASIC_INFORMATION64;

typedef struct _WOW64_SECTION_IMAGE_INFORMATION64 {
    ULONGLONG TransferAddress;
    ULONG ZeroBits;
    ULONG __pad0;
    ULONGLONG MaximumStackSize;
    ULONGLONG CommittedStackSize;
    ULONG SubSystemType;
    USHORT SubSystemMinorVersion;
    USHORT SubSystemMajorVersion;
    ULONG GpValue;
    USHORT ImageCharacteristics;
    USHORT DllCharacteristics;
    USHORT Machine;
    BOOLEAN ImageContainsCode;
    UCHAR ImageFlags;
    ULONG LoaderFlags;
    ULONG ImageFileSize;
    ULONG CheckSum;
} WOW64_SECTION_IMAGE_INFORMATION64;

typedef struct _WOW64_SECTION_IMAGE_INFORMATION32 {
    ULONG TransferAddress;
    ULONG ZeroBits;
    ULONG MaximumStackSize;
    ULONG CommittedStackSize;
    ULONG SubSystemType;
    USHORT SubSystemMinorVersion;
    USHORT SubSystemMajorVersion;
    ULONG GpValue;
    USHORT ImageCharacteristics;
    USHORT DllCharacteristics;
    USHORT Machine;
    BOOLEAN ImageContainsCode;
    UCHAR ImageFlags;
    ULONG LoaderFlags;
    ULONG ImageFileSize;
    ULONG CheckSum;
} WOW64_SECTION_IMAGE_INFORMATION32;

static NTSTATUS Wow64Manual_NtQuerySection(ULONG *a)
{
    HANDLE Section = (HANDLE)(LONG_PTR)(LONG)a[0];
    ULONG Class = a[1];
    PVOID Buf32 = (PVOID)(ULONG_PTR)a[2];
    ULONG Len32 = a[3];
    ULONG *RetLen32 = (ULONG *)(ULONG_PTR)a[4];
    SIZE_T Ret = 0;
    NTSTATUS St;

    if (Class == 1) /* SectionImageInformation */
    {
        WOW64_SECTION_IMAGE_INFORMATION64 Sii64;
        WOW64_SECTION_IMAGE_INFORMATION32 *Sii32 = Buf32;
        if (Len32 < sizeof(*Sii32))
            return (NTSTATUS)0xC0000004L; /* STATUS_INFO_LENGTH_MISMATCH */
        St = NtQuerySection(Section, Class, &Sii64, sizeof(Sii64), &Ret);
        if (St >= 0)
        {
            Sii32->TransferAddress = (ULONG)Sii64.TransferAddress;
            Sii32->ZeroBits = Sii64.ZeroBits;
            Sii32->MaximumStackSize = (ULONG)Sii64.MaximumStackSize;
            Sii32->CommittedStackSize = (ULONG)Sii64.CommittedStackSize;
            Sii32->SubSystemType = Sii64.SubSystemType;
            Sii32->SubSystemMinorVersion = Sii64.SubSystemMinorVersion;
            Sii32->SubSystemMajorVersion = Sii64.SubSystemMajorVersion;
            Sii32->GpValue = Sii64.GpValue;
            Sii32->ImageCharacteristics = Sii64.ImageCharacteristics;
            Sii32->DllCharacteristics = Sii64.DllCharacteristics;
            Sii32->Machine = Sii64.Machine;
            Sii32->ImageContainsCode = Sii64.ImageContainsCode;
            Sii32->ImageFlags = Sii64.ImageFlags;
            Sii32->LoaderFlags = Sii64.LoaderFlags;
            Sii32->ImageFileSize = Sii64.ImageFileSize;
            Sii32->CheckSum = Sii64.CheckSum;
            if (RetLen32) *RetLen32 = sizeof(*Sii32);
        }
        return St;
    }
    if (Class == 0) /* SectionBasicInformation */
    {
        WOW64_SECTION_BASIC_INFORMATION64 Sbi64;
        struct { ULONG Base; ULONG Attr; LARGE_INTEGER Size; } *Sbi32 = Buf32;
        if (Len32 < sizeof(*Sbi32))
            return (NTSTATUS)0xC0000004L;
        St = NtQuerySection(Section, Class, &Sbi64, sizeof(Sbi64), &Ret);
        if (St >= 0)
        {
            Sbi32->Base = (ULONG)Sbi64.BaseAddress;
            Sbi32->Attr = Sbi64.Attributes;
            Sbi32->Size.QuadPart = (LONGLONG)Sbi64.MaximumSize;
            if (RetLen32) *RetLen32 = sizeof(*Sbi32);
        }
        return St;
    }
    DbgPrint("[WOW64]   NtQuerySection class=%lu unhandled\n", Class);
    return (NTSTATUS)0xC0000003L; /* STATUS_INVALID_INFO_CLASS */
}

/* ---- NtQuerySymbolicLinkObject ------------------------------------------ *
 * In/out UNICODE_STRING: the guest struct is 8 bytes (32-bit Buffer), the host
 * writes 16. Build a UNICODE_STRING64 pointing at the SAME guest buffer (the
 * host writes the link target straight into guest memory <4GB), then copy the
 * updated Length back down. */
static NTSTATUS Wow64Manual_NtQuerySymbolicLinkObject(ULONG *a)
{
    HANDLE Link = (HANDLE)(LONG_PTR)(LONG)a[0];
    WOW64_UNICODE_STRING32_M *Tgt32 = (WOW64_UNICODE_STRING32_M *)(ULONG_PTR)a[1];
    ULONG *RetLen32 = (ULONG *)(ULONG_PTR)a[2];
    WOW64_UNICODE_STRING64_M Tgt64;
    ULONG RetLen = 0;
    NTSTATUS St;

    if (!Tgt32)
        return (NTSTATUS)0xC000000DL; /* STATUS_INVALID_PARAMETER */
    Tgt64.Length = Tgt32->Length;
    Tgt64.MaximumLength = Tgt32->MaximumLength;
    Tgt64.__pad = 0;
    Tgt64.Buffer = (ULONG_PTR)Tgt32->Buffer; /* guest buffer, host writes here */
    St = NtQuerySymbolicLinkObject(Link, &Tgt64, &RetLen);
    Tgt32->Length = Tgt64.Length;
    if (RetLen32) *RetLen32 = RetLen;
    return St;
}

/* NtContinue: load the guest's 32-bit CONTEXT into the CPU layer instead of
 * letting the host kernel reinterpret it as an arm64 CONTEXT. FEX marks the
 * CPU area dirty and reloads guest state when BTCpuSimulate reenters, so
 * returning "success" here resumes the guest at the new Eip. */
static NTSTATUS Wow64Manual_NtContinue(ULONG *a)
{
    extern LONG (WINAPI *Wow64pBTCpuSetContext)(HANDLE, HANDLE, PVOID, PVOID);
    PVOID Ctx32 = (PVOID)(ULONG_PTR)a[0];

    if (!Ctx32 || !Wow64pBTCpuSetContext)
        return (NTSTATUS)0xC000000DL;
    return Wow64pBTCpuSetContext((HANDLE)(LONG_PTR)-2 /* thread */,
                                 (HANDLE)(LONG_PTR)-1 /* process */,
                                 NULL, Ctx32);
}

/* ---- NtGdiOpenDCW (M4/wow64win) ----------------------------------------- *
 * The first win32k marshaller: gdi32's CreateDC path. The two UNICODE_STRING
 * args (device name, log address) widen 8->16 bytes; DEVMODEW is a flat
 * pointer-free public struct (identical 32/64 bytes) so its guest pointer
 * passes through; hspool is a 32-bit HANDLE; pUMdhpdev is an output PVOID*
 * (kernel writes an 8-byte pointer, narrow to the 4-byte guest cell — for a
 * display DC it's typically untouched). The HDC return is a 32-bit GDI handle
 * even on Win64, so it fits straight into guest EAX.
 *   NtGdiOpenDCW(pustrDevice, pdm, pustrLogAddr, iType, bDisplay, hspool, pUMdhpdev) */
static NTSTATUS Wow64Manual_NtGdiOpenDCW(ULONG *a)
{
    extern HANDLE NtGdiOpenDCW(PVOID, PVOID, PVOID, ULONG, BOOL, HANDLE, PVOID);
    WOW64_UNICODE_STRING32_M *Dev32 = (WOW64_UNICODE_STRING32_M *)(ULONG_PTR)a[0];
    PVOID Pdm = (PVOID)(ULONG_PTR)a[1]; /* DEVMODEW: flat, passthrough */
    WOW64_UNICODE_STRING32_M *Log32 = (WOW64_UNICODE_STRING32_M *)(ULONG_PTR)a[2];
    ULONG IType = a[3];
    BOOL BDisplay = (BOOL)a[4];
    HANDLE HSpool = (HANDLE)(ULONG_PTR)a[5];
    ULONG *UMdh32 = (ULONG *)(ULONG_PTR)a[6];
    WOW64_UNICODE_STRING64_M Dev64, Log64;
    PVOID Dev64Ptr = NULL, Log64Ptr = NULL, UMdh64 = NULL;
    HANDLE Hdc;

    if (Dev32)
    {
        Dev64.Length = Dev32->Length;
        Dev64.MaximumLength = Dev32->MaximumLength;
        Dev64.__pad = 0;
        Dev64.Buffer = (ULONG_PTR)Dev32->Buffer;
        Dev64Ptr = &Dev64;
    }
    if (Log32)
    {
        Log64.Length = Log32->Length;
        Log64.MaximumLength = Log32->MaximumLength;
        Log64.__pad = 0;
        Log64.Buffer = (ULONG_PTR)Log32->Buffer;
        Log64Ptr = &Log64;
    }

    Hdc = NtGdiOpenDCW(Dev64Ptr, Pdm, Log64Ptr, IType, BDisplay, HSpool,
                       UMdh32 ? &UMdh64 : NULL);
    if (UMdh32)
        *UMdh32 = (ULONG)(ULONG_PTR)UMdh64;
    DbgPrint("[WOW64]   NtGdiOpenDCW(dev='%.*S' type=%lu disp=%d) => hdc=%p\n",
             Dev64Ptr ? (int)(Dev64.Length / sizeof(WCHAR)) : 0,
             Dev64Ptr ? (PCWSTR)(ULONG_PTR)Dev64.Buffer : L"", IType, BDisplay, Hdc);
    return (NTSTATUS)(ULONG)(ULONG_PTR)Hdc;
}

/* ---- NtQueryValueKey ------------------------------------------------------ *
 * ValueName is a PUNICODE_STRING (widens); the KEY_VALUE_*_INFORMATION output
 * blobs are pointer-free and layout-identical 32/64, so the guest buffer
 * passes straight through. First hit: user32 init reading registry config
 * right after NtOpenKey (a raw auto-thunk fed the kernel a 32-bit
 * UNICODE_STRING, whose garbage Buffer probe-raised C0000005 -> 0x1E under
 * DUMMY_PSEH). */
static NTSTATUS Wow64Manual_NtQueryValueKey(ULONG *a)
{
    extern NTSTATUS NTAPI NtQueryValueKey(HANDLE, PVOID, ULONG, PVOID, ULONG, PULONG);
    HANDLE Key = (HANDLE)(LONG_PTR)(LONG)a[0];
    WOW64_UNICODE_STRING32_M *Name32 = (WOW64_UNICODE_STRING32_M *)(ULONG_PTR)a[1];
    ULONG Class = a[2];
    PVOID Buf = (PVOID)(ULONG_PTR)a[3];
    ULONG Len = a[4];
    ULONG *RetLen = (ULONG *)(ULONG_PTR)a[5];
    WOW64_UNICODE_STRING64_M Name64;
    PVOID Name64Ptr = NULL;

    if (Name32)
    {
        Name64.Length = Name32->Length;
        Name64.MaximumLength = Name32->MaximumLength;
        Name64.__pad = 0;
        Name64.Buffer = (ULONG_PTR)Name32->Buffer;
        Name64Ptr = &Name64;
    }
    return NtQueryValueKey(Key, Name64Ptr, Class, Buf, Len, RetLen);
}

/* ---- NtUserProcessConnect (win32k client connect, task #29) -------------- *
 * user32's per-process win32k handshake. The USERCONNECT blob is
 * layout-divergent (pointers widen): 32-bit = 0x124 bytes, 64-bit = 0x240.
 * Build the native struct, call through win32u, then narrow the SHAREDINFO
 * back. The kernel maps the global user heap below 4GB for WoW64 processes
 * (win32k W32Wow64ZeroBits), so psi/aheList/pDispInfo user VAs truncate
 * losslessly; ulSharedDelta is a (kernel - user) difference whose 32-bit
 * truncation still converts kernel pointers correctly in the guest's modular
 * 32-bit arithmetic ((k - d) mod 2^32 = user VA when user VA < 4GB).
 * FNID_NUM = 31 (ntuser.h; C_ASSERT keeps the 32-bit size honest at 0x124). */
#define WOW64_FNID_NUM 31
typedef struct { ULONG maxMsgs; ULONG abMsgs; } WOW64_WNDMSG32;
typedef struct { ULONG maxMsgs; ULONG __pad; ULONGLONG abMsgs; } WOW64_WNDMSG64;
typedef struct {
    ULONG psi, aheList, pDispInfo, ulSharedDelta;
    WOW64_WNDMSG32 awmControl[WOW64_FNID_NUM];
    WOW64_WNDMSG32 DefWindowMsgs, DefWindowSpecMsgs;
} WOW64_SHAREDINFO32;
typedef struct {
    ULONGLONG psi, aheList, pDispInfo, ulSharedDelta;
    WOW64_WNDMSG64 awmControl[WOW64_FNID_NUM];
    WOW64_WNDMSG64 DefWindowMsgs, DefWindowSpecMsgs;
} WOW64_SHAREDINFO64;
typedef struct {
    ULONG ulVersion, ulCurrentVersion, dwDispatchCount;
    WOW64_SHAREDINFO32 siClient;
} WOW64_USERCONNECT32;
typedef struct {
    ULONG ulVersion, ulCurrentVersion, dwDispatchCount, __pad;
    WOW64_SHAREDINFO64 siClient;
} WOW64_USERCONNECT64;

static NTSTATUS Wow64Manual_NtUserProcessConnect(ULONG *a)
{
    extern NTSTATUS NtUserProcessConnect(HANDLE, PVOID, ULONG);
    HANDLE Process = (HANDLE)(LONG_PTR)(LONG)a[0];
    WOW64_USERCONNECT32 *Uc32 = (WOW64_USERCONNECT32 *)(ULONG_PTR)a[1];
    ULONG Size32 = a[2];
    WOW64_USERCONNECT64 Uc64;
    NTSTATUS Status;
    ULONG i;

    if (!Uc32 || Size32 != sizeof(WOW64_USERCONNECT32))
        return (NTSTATUS)0xC000000DL; /* STATUS_INVALID_PARAMETER */

    RtlZeroMemory(&Uc64, sizeof(Uc64));
    Uc64.ulVersion = Uc32->ulVersion;
    Uc64.ulCurrentVersion = Uc32->ulCurrentVersion;
    Uc64.dwDispatchCount = Uc32->dwDispatchCount;

    Status = NtUserProcessConnect(Process, &Uc64, sizeof(Uc64));
    if (Status >= 0)
    {
        Uc32->ulVersion = Uc64.ulVersion;
        Uc32->ulCurrentVersion = Uc64.ulCurrentVersion;
        Uc32->dwDispatchCount = Uc64.dwDispatchCount;
        if (Uc64.siClient.psi > 0xFFFFFFFFULL ||
            Uc64.siClient.aheList > 0xFFFFFFFFULL)
            DbgPrint("[WOW64] NtUserProcessConnect: SHARED PTRS ABOVE 4GB "
                     "(psi=%I64x ahe=%I64x) — kernel low-map missing?!\n",
                     Uc64.siClient.psi, Uc64.siClient.aheList);
        Uc32->siClient.psi = (ULONG)Uc64.siClient.psi;
        Uc32->siClient.aheList = (ULONG)Uc64.siClient.aheList;
        Uc32->siClient.pDispInfo = (ULONG)Uc64.siClient.pDispInfo;
        Uc32->siClient.ulSharedDelta = (ULONG)Uc64.siClient.ulSharedDelta;
        for (i = 0; i < WOW64_FNID_NUM; i++)
        {
            Uc32->siClient.awmControl[i].maxMsgs = Uc64.siClient.awmControl[i].maxMsgs;
            Uc32->siClient.awmControl[i].abMsgs = (ULONG)Uc64.siClient.awmControl[i].abMsgs;
        }
        Uc32->siClient.DefWindowMsgs.maxMsgs = Uc64.siClient.DefWindowMsgs.maxMsgs;
        Uc32->siClient.DefWindowMsgs.abMsgs = (ULONG)Uc64.siClient.DefWindowMsgs.abMsgs;
        Uc32->siClient.DefWindowSpecMsgs.maxMsgs = Uc64.siClient.DefWindowSpecMsgs.maxMsgs;
        Uc32->siClient.DefWindowSpecMsgs.abMsgs = (ULONG)Uc64.siClient.DefWindowSpecMsgs.abMsgs;
        DbgPrint("[WOW64]   NtUserProcessConnect: psi=%08lx ahe=%08lx delta=%08lx\n",
                 Uc32->siClient.psi, Uc32->siClient.aheList,
                 Uc32->siClient.ulSharedDelta);
    }
    return Status;
}

/* Name-dispatched entry point. Returns WOW64_MANUAL_UNIMPL if unhandled. */
NTSTATUS Wow64ManualDispatch(const char *Name, ULONG *Arguments)
{
    /* tiny hand strcmp to avoid a CRT dependency */
#define WOW64_IS(n) Wow64pStrEq(Name, n)
    if (WOW64_IS("NtQueryVirtualMemory"))
        return Wow64Manual_NtQueryVirtualMemory(Arguments);
    if (WOW64_IS("NtQueryInformationProcess"))
        return Wow64Manual_NtQueryInformationProcess(Arguments);
    if (WOW64_IS("NtQuerySystemInformation"))
        return Wow64Manual_NtQuerySystemInformation(Arguments);
    if (WOW64_IS("NtOpenKey"))
        return Wow64pOpenByOa(NtOpenKey, Arguments);
    if (WOW64_IS("NtOpenDirectoryObject"))
        return Wow64pOpenByOa(NtOpenDirectoryObject, Arguments);
    if (WOW64_IS("NtOpenSymbolicLinkObject"))
        return Wow64pOpenByOa(NtOpenSymbolicLinkObject, Arguments);
    if (WOW64_IS("NtOpenSection"))
        return Wow64pOpenByOa(NtOpenSection, Arguments);
    if (WOW64_IS("NtQuerySymbolicLinkObject"))
        return Wow64Manual_NtQuerySymbolicLinkObject(Arguments);
    if (WOW64_IS("NtAllocateVirtualMemory"))
        return Wow64Manual_NtAllocateVirtualMemory(Arguments);
    if (WOW64_IS("NtFreeVirtualMemory"))
        return Wow64Manual_NtFreeVirtualMemory(Arguments);
    if (WOW64_IS("NtProtectVirtualMemory"))
        return Wow64Manual_NtProtectVirtualMemory(Arguments);
    if (WOW64_IS("NtMapViewOfSection"))
        return Wow64Manual_NtMapViewOfSection(Arguments);
    if (WOW64_IS("NtOpenFile"))
        return Wow64Manual_NtOpenFile(Arguments);
    if (WOW64_IS("NtCreateFile"))
        return Wow64Manual_NtCreateFile(Arguments);
    if (WOW64_IS("NtCreateSection"))
        return Wow64Manual_NtCreateSection(Arguments);
    if (WOW64_IS("NtQueryAttributesFile"))
        return Wow64Manual_NtQueryAttributesFile(Arguments);
    if (WOW64_IS("NtQueryFullAttributesFile"))
        return Wow64Manual_NtQueryFullAttributesFile(Arguments);
    if (WOW64_IS("NtQuerySection"))
        return Wow64Manual_NtQuerySection(Arguments);
    if (WOW64_IS("NtCreateKeyedEvent"))
        return Wow64Manual_NtCreateKeyedEvent(Arguments);
    if (WOW64_IS("NtSecureConnectPort"))
        return Wow64Manual_NtSecureConnectPort(Arguments);
    if (WOW64_IS("NtConnectPort"))
        return Wow64Manual_NtConnectPort(Arguments);
    if (WOW64_IS("NtContinue"))
        return Wow64Manual_NtContinue(Arguments);
    if (WOW64_IS("NtCreateDirectoryObject"))
        return Wow64pOpenByOa(NtCreateDirectoryObject, Arguments);
    if (WOW64_IS("NtCreateEvent"))
        return Wow64Manual_NtCreateEvent(Arguments);
    if (WOW64_IS("NtOpenEvent"))
        return Wow64pOpenByOa(NtOpenEvent, Arguments);
    if (WOW64_IS("NtGdiOpenDCW"))
        return Wow64Manual_NtGdiOpenDCW(Arguments);
    if (WOW64_IS("NtUserProcessConnect"))
        return Wow64Manual_NtUserProcessConnect(Arguments);
    if (WOW64_IS("NtQueryValueKey"))
        return Wow64Manual_NtQueryValueKey(Arguments);
#undef WOW64_IS
    return WOW64_MANUAL_UNIMPL;
}

int Wow64pStrEq(const char *a, const char *b)
{
    while (*a && *a == *b) { a++; b++; }
    return *a == *b;
}
