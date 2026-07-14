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
NTSTATUS WINAPI NtReadFile(HANDLE File, HANDLE Event, PVOID ApcRoutine,
                           PVOID ApcContext, PVOID Iosb, PVOID Buffer,
                           ULONG Length, PLARGE_INTEGER ByteOffset, PULONG Key);
NTSTATUS WINAPI NtWriteFile(HANDLE File, HANDLE Event, PVOID ApcRoutine,
                            PVOID ApcContext, PVOID Iosb, PVOID Buffer,
                            ULONG Length, PLARGE_INTEGER ByteOffset, PULONG Key);
NTSTATUS WINAPI NtQueryInformationFile(HANDLE File, PVOID Iosb, PVOID Info,
                                       ULONG Length, ULONG Class);
NTSTATUS WINAPI NtSetInformationFile(HANDLE File, PVOID Iosb, PVOID Info,
                                     ULONG Length, ULONG Class);
NTSTATUS WINAPI NtQueryVolumeInformationFile(HANDLE File, PVOID Iosb, PVOID Info,
                                             ULONG Length, ULONG Class);
NTSTATUS WINAPI NtFlushBuffersFile(HANDLE File, PVOID Iosb);
NTSTATUS WINAPI NtQueryDirectoryFile(HANDLE File, HANDLE Event, PVOID ApcRoutine,
                                     PVOID ApcContext, PVOID Iosb, PVOID Info,
                                     ULONG Length, ULONG Class, BOOLEAN SingleEntry,
                                     PVOID FileName, BOOLEAN RestartScan);
NTSTATUS WINAPI NtWaitForSingleObject(HANDLE Object, BOOLEAN Alertable,
                                      PLARGE_INTEGER Timeout);

/* M12: the rest of the declared-manual queue. Every one of these was a loud
 * WOW64_MANUAL_UNIMPL stub; a stock app trips them the moment it does anything
 * beyond the narrow paths our hand-written guests exercised. (NtCreateSemaphore
 * is the one that actually blocked M11: wininet's DllMain calls it via
 * CreateSemaphoreW, rpcrt4 statically imports wininet, and user32 imports
 * rpcrt4 — so it killed EVERY GUI app with STATUS_DLL_INIT_FAILED.) */
NTSTATUS WINAPI NtCreateSemaphore(PHANDLE Handle, ACCESS_MASK Access, PVOID Oa,
                                  LONG Initial, LONG Maximum);
NTSTATUS WINAPI NtOpenSemaphore(PHANDLE Handle, ACCESS_MASK Access, PVOID Oa);
NTSTATUS WINAPI NtCreateMutant(PHANDLE Handle, ACCESS_MASK Access, PVOID Oa,
                               BOOLEAN InitialOwner);
NTSTATUS WINAPI NtOpenMutant(PHANDLE Handle, ACCESS_MASK Access, PVOID Oa);
NTSTATUS WINAPI NtOpenKeyedEvent(PHANDLE Handle, ACCESS_MASK Access, PVOID Oa);
NTSTATUS WINAPI NtCreateKey(PHANDLE Handle, ACCESS_MASK Access, PVOID Oa,
                            ULONG TitleIndex, PVOID Class, ULONG Options,
                            PULONG Disposition);
NTSTATUS WINAPI NtCreateSymbolicLinkObject(PHANDLE Handle, ACCESS_MASK Access,
                                           PVOID Oa, PVOID Target);
NTSTATUS WINAPI NtOpenProcess(PHANDLE Handle, ACCESS_MASK Access, PVOID Oa,
                              PVOID ClientId);
NTSTATUS WINAPI NtOpenThread(PHANDLE Handle, ACCESS_MASK Access, PVOID Oa,
                             PVOID ClientId);
NTSTATUS WINAPI NtCancelIoFile(HANDLE File, PVOID Iosb);
NTSTATUS WINAPI NtLockFile(HANDLE File, HANDLE Event, PVOID ApcRoutine,
                           PVOID ApcContext, PVOID Iosb, PLARGE_INTEGER ByteOffset,
                           PLARGE_INTEGER Length, ULONG Key, BOOLEAN FailImmediately,
                           BOOLEAN ExclusiveLock);
NTSTATUS WINAPI NtUnlockFile(HANDLE File, PVOID Iosb, PLARGE_INTEGER ByteOffset,
                             PLARGE_INTEGER Length, ULONG Key);
NTSTATUS WINAPI NtSetVolumeInformationFile(HANDLE File, PVOID Iosb, PVOID Info,
                                           ULONG Length, ULONG Class);
NTSTATUS WINAPI NtQueryEaFile(HANDLE File, PVOID Iosb, PVOID Buffer, ULONG Length,
                              BOOLEAN SingleEntry, PVOID EaList, ULONG EaListLength,
                              PULONG EaIndex, BOOLEAN RestartScan);
NTSTATUS WINAPI NtSetEaFile(HANDLE File, PVOID Iosb, PVOID Buffer, ULONG Length);
NTSTATUS WINAPI NtDeviceIoControlFile(HANDLE File, HANDLE Event, PVOID ApcRoutine,
                                      PVOID ApcContext, PVOID Iosb, ULONG Code,
                                      PVOID InBuf, ULONG InLen, PVOID OutBuf,
                                      ULONG OutLen);
NTSTATUS WINAPI NtFsControlFile(HANDLE File, HANDLE Event, PVOID ApcRoutine,
                                PVOID ApcContext, PVOID Iosb, ULONG Code,
                                PVOID InBuf, ULONG InLen, PVOID OutBuf, ULONG OutLen);
NTSTATUS WINAPI NtCreateNamedPipeFile(PHANDLE Handle, ACCESS_MASK Access, PVOID Oa,
                                      PVOID Iosb, ULONG ShareAccess,
                                      ULONG CreateDisposition, ULONG CreateOptions,
                                      ULONG NamedPipeType, ULONG ReadMode,
                                      ULONG CompletionMode, ULONG MaximumInstances,
                                      ULONG InboundQuota, ULONG OutboundQuota,
                                      PLARGE_INTEGER DefaultTimeout);
NTSTATUS WINAPI NtCreateMailslotFile(PHANDLE Handle, ACCESS_MASK Access, PVOID Oa,
                                     PVOID Iosb, ULONG CreateOptions, ULONG Quota,
                                     ULONG MaximumMessageSize,
                                     PLARGE_INTEGER ReadTimeout);

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

/* ---- the IOSB-returning file family ------------------------------------- *
 * These all take an IO_STATUS_BLOCK and were previously AUTO-thunked, i.e. the
 * guest's 8-byte IOSB was handed straight to a host kernel that writes 16 —
 * an 8-byte overflow past the guest struct plus an Information read from the
 * wrong offset. Every one of them now marshals through a host IOSB.
 *
 * Async is the hazard. Our host IOSB lives on THIS stack frame, so returning
 * STATUS_PENDING would leave the kernel holding a pointer into a frame that is
 * about to die — the dead-frame-IOSB corruption class (field manual §9.36). We
 * never do that:
 *
 *   - An explicit async request (guest passed an Event or an APC routine) is
 *     REFUSED loudly before the call is issued. No IRP, no corruption.
 *   - If the host pends anyway (the handle was opened async and the guest is
 *     polling its IOSB), we block on the file handle until completion rather
 *     than return with a live pointer into a dead frame.
 *
 * The real fix for async is kernel-side: I/O completion should write an
 * IO_STATUS_BLOCK32 when the requesting process is WoW64 (what NT's
 * IoIs32bitProcess() gates). Owed; see wow64-plan.md.
 */
#define WOW64_STATUS_PENDING         ((NTSTATUS)0x00000103)
#define WOW64_STATUS_NOT_IMPLEMENTED ((NTSTATUS)0xC0000002)

/* Guest asked for async (Event or ApcRoutine)? Refuse loudly — see above. */
static int Wow64pRefuseAsync(ULONG Event, ULONG ApcRoutine, const char *Who)
{
    if (!Event && !ApcRoutine)
        return 0;
    DbgPrint("[WOW64]   %s: async request (Event=%08lx Apc=%08lx) is not supported "
             "under WoW64 yet (needs kernel-side IO_STATUS_BLOCK32) — refusing\n",
             Who, Event, ApcRoutine);
    return 1;
}

/* Copy the host IOSB down to the guest, blocking first if the host pended. */
static NTSTATUS Wow64pFinishIosb(NTSTATUS St, HANDLE File,
                                 WOW64_IO_STATUS_BLOCK64_M *I64, ULONG Iosb32Ptr,
                                 const char *Who)
{
    if (St == WOW64_STATUS_PENDING)
    {
        NTSTATUS Wait;
        DbgPrint("[WOW64]   %s pended on an async handle — blocking on it "
                 "(our host IOSB is stack-resident; we must not outlive it)\n", Who);
        Wait = NtWaitForSingleObject(File, FALSE, NULL);
        if (Wait < 0)
        {
            DbgPrint("[WOW64]   %s: wait failed %08lx — guest IOSB left untouched\n",
                     Who, Wait);
            return Wait;
        }
        St = (NTSTATUS)(ULONG)I64->Status;
    }
    /* A failing Nt* may never touch the IOSB, and ours is zero-initialised —
     * copying that down would tell the guest "success, 0 bytes". Carry the real
     * status instead. (A no-op when the kernel did write the block.) */
    if (St < 0)
        I64->Status = (ULONG)St;
    Wow64pCopyIosbDown(I64, Iosb32Ptr);
    return St;
}

/* NtReadFile / NtWriteFile share their shape exactly. */
typedef NTSTATUS (WINAPI *WOW64_RW_FN)(HANDLE, HANDLE, PVOID, PVOID, PVOID,
                                       PVOID, ULONG, PLARGE_INTEGER, PULONG);

static NTSTATUS Wow64pReadWriteFile(WOW64_RW_FN Fn, ULONG *a, const char *Who)
{
    HANDLE File = (HANDLE)(LONG_PTR)(LONG)a[0];
    WOW64_IO_STATUS_BLOCK64_M Iosb64 = {0};
    NTSTATUS St;

    if (Wow64pRefuseAsync(a[1], a[2], Who))
        return WOW64_STATUS_NOT_IMPLEMENTED;

    /* ByteOffset is a LARGE_INTEGER* and Key a ULONG* — both layout-identical. */
    St = Fn(File, NULL, NULL, NULL, &Iosb64,
            (PVOID)(ULONG_PTR)a[5], a[6],
            (PLARGE_INTEGER)(ULONG_PTR)a[7], (PULONG)(ULONG_PTR)a[8]);
    return Wow64pFinishIosb(St, File, &Iosb64, a[4], Who);
}

static NTSTATUS Wow64Manual_NtReadFile(ULONG *a)
{
    return Wow64pReadWriteFile(NtReadFile, a, "NtReadFile");
}

static NTSTATUS Wow64Manual_NtWriteFile(ULONG *a)
{
    return Wow64pReadWriteFile(NtWriteFile, a, "NtWriteFile");
}

/* Every FILE_*_INFORMATION / FS_*_INFORMATION struct is built from
 * LARGE_INTEGER/ULONG/WCHAR only — no pointers, no ULONG_PTR — so the info
 * buffers themselves are width-identical and pass through. The exceptions all
 * carry a HANDLE and are set-only; refuse those loudly rather than marshal a
 * class no caller needs yet. */
#define WOW64_FileRenameInformation       10
#define WOW64_FileLinkInformation         11
#define WOW64_FileCompletionInformation   30
#define WOW64_FileMoveClusterInformation  31

static NTSTATUS Wow64Manual_NtQueryInformationFile(ULONG *a)
{
    HANDLE File = (HANDLE)(LONG_PTR)(LONG)a[0];
    WOW64_IO_STATUS_BLOCK64_M Iosb64 = {0};
    NTSTATUS St = NtQueryInformationFile(File, &Iosb64, (PVOID)(ULONG_PTR)a[2],
                                         a[3], a[4]);
    return Wow64pFinishIosb(St, File, &Iosb64, a[1], "NtQueryInformationFile");
}

static NTSTATUS Wow64Manual_NtSetInformationFile(ULONG *a)
{
    HANDLE File = (HANDLE)(LONG_PTR)(LONG)a[0];
    ULONG Class = a[4];
    WOW64_IO_STATUS_BLOCK64_M Iosb64 = {0};
    NTSTATUS St;

    if (Class == WOW64_FileRenameInformation || Class == WOW64_FileLinkInformation ||
        Class == WOW64_FileCompletionInformation ||
        Class == WOW64_FileMoveClusterInformation)
    {
        /* These embed a HANDLE (and FileCompletionInformation a PVOID key), so
         * the 32-bit struct is narrower. No caller needs them yet — fail loud
         * rather than pass a mis-parsed struct to the kernel. */
        DbgPrint("[WOW64]   NtSetInformationFile class=%lu embeds a HANDLE — "
                 "no 32->64 marshaller yet, refusing\n", Class);
        return WOW64_STATUS_NOT_IMPLEMENTED;
    }

    St = NtSetInformationFile(File, &Iosb64, (PVOID)(ULONG_PTR)a[2], a[3], Class);
    return Wow64pFinishIosb(St, File, &Iosb64, a[1], "NtSetInformationFile");
}

static NTSTATUS Wow64Manual_NtQueryVolumeInformationFile(ULONG *a)
{
    HANDLE File = (HANDLE)(LONG_PTR)(LONG)a[0];
    WOW64_IO_STATUS_BLOCK64_M Iosb64 = {0};
    NTSTATUS St = NtQueryVolumeInformationFile(File, &Iosb64, (PVOID)(ULONG_PTR)a[2],
                                               a[3], a[4]);
    return Wow64pFinishIosb(St, File, &Iosb64, a[1], "NtQueryVolumeInformationFile");
}

static NTSTATUS Wow64Manual_NtFlushBuffersFile(ULONG *a)
{
    HANDLE File = (HANDLE)(LONG_PTR)(LONG)a[0];
    WOW64_IO_STATUS_BLOCK64_M Iosb64 = {0};
    NTSTATUS St = NtFlushBuffersFile(File, &Iosb64);
    return Wow64pFinishIosb(St, File, &Iosb64, a[1], "NtFlushBuffersFile");
}

/* The FILE_*_DIRECTORY_INFORMATION output structs are width-identical; only the
 * IOSB and the optional PUNICODE_STRING search mask need marshalling. */
static NTSTATUS Wow64Manual_NtQueryDirectoryFile(ULONG *a)
{
    HANDLE File = (HANDLE)(LONG_PTR)(LONG)a[0];
    WOW64_IO_STATUS_BLOCK64_M Iosb64 = {0};
    WOW64_UNICODE_STRING64_M Mask64;
    PVOID Mask64Ptr = NULL;
    NTSTATUS St;

    if (Wow64pRefuseAsync(a[1], a[2], "NtQueryDirectoryFile"))
        return WOW64_STATUS_NOT_IMPLEMENTED;

    if (a[9])
    {
        WOW64_UNICODE_STRING32_M *M32 = (WOW64_UNICODE_STRING32_M *)(ULONG_PTR)a[9];
        Mask64.Length = M32->Length;
        Mask64.MaximumLength = M32->MaximumLength;
        Mask64.__pad = 0;
        Mask64.Buffer = (ULONG_PTR)M32->Buffer;
        Mask64Ptr = &Mask64;
    }

    St = NtQueryDirectoryFile(File, NULL, NULL, NULL, &Iosb64,
                              (PVOID)(ULONG_PTR)a[5], a[6], a[7],
                              (BOOLEAN)a[8], Mask64Ptr, (BOOLEAN)a[10]);
    return Wow64pFinishIosb(St, File, &Iosb64, a[4], "NtQueryDirectoryFile");
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

/* ---- M8: user32 window creation ------------------------------------------ *
 * The window path's syscalls carry pointer-bearing structs (WNDCLASSEX,
 * LARGE_STRING, MSG, PAINTSTRUCT, CLSMENUNAME) whose 32/64 layouts differ;
 * each gets a real 32->64 (and back) conversion here. Kernel->user callbacks
 * are the other half — wow64_callback.c; NtCallbackReturn below is its
 * guest-side completion. */

typedef struct _WOW64_MSG32 {
    ULONG hwnd; ULONG message; ULONG wParam; ULONG lParam;
    ULONG time; LONG ptx; LONG pty;
} WOW64_MSG32; /* 28 */

typedef struct _WOW64_MSG64 {
    ULONGLONG hwnd; ULONG message; ULONG _pad0;
    ULONGLONG wParam; ULONGLONG lParam;
    ULONG time; LONG ptx; LONG pty; ULONG _pad1;
} WOW64_MSG64; /* 48 */

static void Wow64pMsg32To64(const WOW64_MSG32 *M32, WOW64_MSG64 *M64)
{
    M64->hwnd = M32->hwnd; M64->message = M32->message; M64->_pad0 = 0;
    M64->wParam = M32->wParam; M64->lParam = M32->lParam;
    M64->time = M32->time; M64->ptx = M32->ptx; M64->pty = M32->pty;
    M64->_pad1 = 0;
}

static void Wow64pMsg64To32(const WOW64_MSG64 *M64, WOW64_MSG32 *M32)
{
    M32->hwnd = (ULONG)M64->hwnd; M32->message = M64->message;
    M32->wParam = (ULONG)M64->wParam; M32->lParam = (ULONG)M64->lParam;
    M32->time = M64->time; M32->ptx = M64->ptx; M32->pty = M64->pty;
}

typedef struct _WOW64_LARGE_STRING32 {
    ULONG Length; ULONG MaximumLength; ULONG Buffer;
} WOW64_LARGE_STRING32; /* bAnsi lives in MaximumLength bit 31 */

typedef struct _WOW64_LARGE_STRING64 {
    ULONG Length; ULONG MaximumLength; ULONGLONG Buffer;
} WOW64_LARGE_STRING64;

/* NtUserDefSetText: the OTHER LARGE_STRING taker.
 *
 * LARGE_STRING is { ULONG Length; ULONG MaximumLength:31; ULONG bAnsi:1;
 * PVOID Buffer; } — 12 bytes on i386, 16 on the host. Buffer sits at +8 in
 * both, but the host reads EIGHT bytes there, so the high half is whatever
 * happens to follow the guest's 12-byte struct on its stack: a wild pointer.
 *
 * win32k dereferences it while holding the USER lock, so this didn't fault
 * cleanly — it WEDGED THE WHOLE DESKTOP (every thread that touches win32k then
 * blocks behind the lock; even winlogon's WaitForSingleObject never returned).
 *
 * NtUserCreateWindowEx (M8) already marshals its three LARGE_STRINGs. This one
 * is the only other syscall in the family and it was never listed — the guest
 * only reaches it through DefWindowProcW(WM_NCCREATE) setting the window title
 * from CREATESTRUCT.lpszName, which every stock window does and none of our
 * hand-written guests did. */
static NTSTATUS Wow64Manual_NtUserDefSetText(ULONG *a)
{
    extern ULONG NtUserDefSetText(PVOID Wnd, PVOID WindowText);
    WOW64_LARGE_STRING64 Ls64;
    PVOID Ls64Ptr = NULL;

    if (a[1])
    {
        const WOW64_LARGE_STRING32 *Ls32 =
            (const WOW64_LARGE_STRING32 *)(ULONG_PTR)a[1];
        Ls64.Length = Ls32->Length;
        Ls64.MaximumLength = Ls32->MaximumLength; /* carries bAnsi in bit 31 */
        Ls64.Buffer = (ULONGLONG)Ls32->Buffer;
        Ls64Ptr = &Ls64;
    }
    /* A NULL WindowText is legal — it clears the title. */
    return (NTSTATUS)(ULONG)NtUserDefSetText((PVOID)(ULONG_PTR)a[0], Ls64Ptr);
}

static NTSTATUS Wow64Manual_NtUserRegisterClassExWOW(ULONG *a)
{
    extern ULONG NtUserRegisterClassExWOW(PVOID, PVOID, PVOID, PVOID, ULONG, ULONG, PVOID);
    typedef struct { ULONG cbSize, style, lpfnWndProc; LONG cbClsExtra, cbWndExtra;
                     ULONG hInstance, hIcon, hCursor, hbrBackground,
                           lpszMenuName, lpszClassName, hIconSm; } WC32;
    typedef struct { ULONG cbSize, style; ULONGLONG lpfnWndProc;
                     LONG cbClsExtra, cbWndExtra;
                     ULONGLONG hInstance, hIcon, hCursor, hbrBackground,
                               lpszMenuName, lpszClassName, hIconSm; } WC64;
    typedef struct { ULONG pszClientAnsiMenuName, pwszClientUnicodeMenuName,
                     pusMenuName; } CMN32;
    typedef struct { ULONGLONG pszClientAnsiMenuName, pwszClientUnicodeMenuName,
                     pusMenuName; } CMN64;

    const WC32 *W32 = (const WC32 *)(ULONG_PTR)a[0];
    WOW64_UNICODE_STRING32_M *CN32 = (WOW64_UNICODE_STRING32_M *)(ULONG_PTR)a[1];
    WOW64_UNICODE_STRING32_M *CV32 = (WOW64_UNICODE_STRING32_M *)(ULONG_PTR)a[2];
    const CMN32 *M32 = (const CMN32 *)(ULONG_PTR)a[3];
    WC64 W64;
    WOW64_UNICODE_STRING64_M CN64, CV64, MenuUs64;
    CMN64 M64;
    PVOID CN64p = NULL, CV64p = NULL, M64p = NULL;

    if (!W32)
        return 0; /* ATOM 0 = fail */
    W64.cbSize = sizeof(W64);      /* kernel probes cbSize == sizeof(WNDCLASSEXW) */
    W64.style = W32->style;
    W64.lpfnWndProc = W32->lpfnWndProc;
    W64.cbClsExtra = W32->cbClsExtra;
    W64.cbWndExtra = W32->cbWndExtra;
    W64.hInstance = W32->hInstance;
    W64.hIcon = W32->hIcon;
    W64.hCursor = W32->hCursor;
    W64.hbrBackground = W32->hbrBackground;
    W64.lpszMenuName = W32->lpszMenuName;
    W64.lpszClassName = W32->lpszClassName;
    W64.hIconSm = W32->hIconSm;

    if (CN32) { CN64.Length = CN32->Length; CN64.MaximumLength = CN32->MaximumLength;
                CN64.__pad = 0; CN64.Buffer = CN32->Buffer; CN64p = &CN64; }
    if (CV32) { CV64.Length = CV32->Length; CV64.MaximumLength = CV32->MaximumLength;
                CV64.__pad = 0; CV64.Buffer = CV32->Buffer; CV64p = &CV64; }
    if (M32)
    {
        M64.pszClientAnsiMenuName = M32->pszClientAnsiMenuName;
        M64.pwszClientUnicodeMenuName = M32->pwszClientUnicodeMenuName;
        M64.pusMenuName = 0;
        if (M32->pusMenuName)
        {
            WOW64_UNICODE_STRING32_M *MU32 =
                (WOW64_UNICODE_STRING32_M *)(ULONG_PTR)M32->pusMenuName;
            MenuUs64.Length = MU32->Length;
            MenuUs64.MaximumLength = MU32->MaximumLength;
            MenuUs64.__pad = 0;
            MenuUs64.Buffer = MU32->Buffer;
            M64.pusMenuName = (ULONGLONG)(ULONG_PTR)&MenuUs64;
        }
        M64p = &M64;
    }

    return (NTSTATUS)NtUserRegisterClassExWOW(&W64, CN64p, CV64p, M64p, a[4], a[5],
                                              (PVOID)(ULONG_PTR)a[6]);
}

static NTSTATUS Wow64Manual_NtUserCreateWindowEx(ULONG *a)
{
    extern ULONGLONG NtUserCreateWindowEx(ULONG, PVOID, PVOID, PVOID, ULONG,
                                          LONG, LONG, LONG, LONG, ULONGLONG,
                                          ULONGLONG, ULONGLONG, ULONGLONG,
                                          ULONG, PVOID);
    WOW64_LARGE_STRING64 Cls64, Ver64, Wnd64;
    PVOID Cls = NULL, Ver = NULL, Wnd = NULL;

    /* Class/version/name arrive as PLARGE_STRING32 — except a raw atom can
     * ride in the class slot (value < 0x10000): pass those through. */
#define WOW64_LS32TO64(idx, s64, out)                                          \
    do {                                                                       \
        ULONG _v = a[idx];                                                     \
        if (_v && _v >= 0x10000)                                               \
        {                                                                      \
            const WOW64_LARGE_STRING32 *_s32 =                                 \
                (const WOW64_LARGE_STRING32 *)(ULONG_PTR)_v;                   \
            (s64).Length = _s32->Length;                                       \
            (s64).MaximumLength = _s32->MaximumLength;                         \
            (s64).Buffer = _s32->Buffer;                                       \
            (out) = &(s64);                                                    \
        }                                                                      \
        else if (_v)                                                           \
            (out) = (PVOID)(ULONG_PTR)_v;                                      \
    } while (0)

    WOW64_LS32TO64(1, Cls64, Cls);
    WOW64_LS32TO64(2, Ver64, Ver);
    WOW64_LS32TO64(3, Wnd64, Wnd);
#undef WOW64_LS32TO64

    return (NTSTATUS)(ULONG)NtUserCreateWindowEx(
        a[0], Cls, Ver, Wnd, a[4],
        (LONG)a[5], (LONG)a[6], (LONG)a[7], (LONG)a[8],
        a[9], a[10], a[11], a[12], a[13], (PVOID)(ULONG_PTR)a[14]);
}

static NTSTATUS Wow64Manual_NtUserGetMessage(ULONG *a)
{
    extern LONG NtUserGetMessage(PVOID, ULONGLONG, ULONG, ULONG);
    WOW64_MSG32 *M32 = (WOW64_MSG32 *)(ULONG_PTR)a[0];
    WOW64_MSG64 M64;
    LONG Ret;

    if (!M32)
        return 0;
    Ret = NtUserGetMessage(&M64, a[1], a[2], a[3]);
    Wow64pMsg64To32(&M64, M32);
    return Ret;
}

static NTSTATUS Wow64Manual_NtUserPeekMessage(ULONG *a)
{
    extern LONG NtUserPeekMessage(PVOID, ULONGLONG, ULONG, ULONG, ULONG);
    WOW64_MSG32 *M32 = (WOW64_MSG32 *)(ULONG_PTR)a[0];
    WOW64_MSG64 M64;
    LONG Ret;

    if (!M32)
        return 0;
    Ret = NtUserPeekMessage(&M64, a[1], a[2], a[3], a[4]);
    if (Ret)
        Wow64pMsg64To32(&M64, M32);
    return Ret;
}

static NTSTATUS Wow64Manual_NtUserDispatchMessage(ULONG *a)
{
    extern ULONGLONG NtUserDispatchMessage(PVOID);
    WOW64_MSG32 *M32 = (WOW64_MSG32 *)(ULONG_PTR)a[0];
    WOW64_MSG64 M64;

    if (!M32)
        return 0;
    Wow64pMsg32To64(M32, &M64);
    return (NTSTATUS)(ULONG)NtUserDispatchMessage(&M64);
}

static NTSTATUS Wow64Manual_NtUserTranslateMessage(ULONG *a)
{
    extern LONG NtUserTranslateMessage(PVOID, ULONG);
    WOW64_MSG32 *M32 = (WOW64_MSG32 *)(ULONG_PTR)a[0];
    WOW64_MSG64 M64;

    if (!M32)
        return 0;
    Wow64pMsg32To64(M32, &M64);
    return NtUserTranslateMessage(&M64, a[1]);
}

typedef struct _WOW64_PAINTSTRUCT32 {
    ULONG hdc; ULONG fErase; LONG rc[4]; ULONG fRestore; ULONG fIncUpdate;
    UCHAR rgb[32];
} WOW64_PAINTSTRUCT32;

typedef struct _WOW64_PAINTSTRUCT64 {
    ULONGLONG hdc; ULONG fErase; LONG rc[4]; ULONG fRestore; ULONG fIncUpdate;
    UCHAR rgb[32]; ULONG _pad;
} WOW64_PAINTSTRUCT64;

static NTSTATUS Wow64Manual_NtUserBeginPaint(ULONG *a)
{
    extern ULONGLONG NtUserBeginPaint(ULONGLONG, PVOID);
    WOW64_PAINTSTRUCT32 *P32 = (WOW64_PAINTSTRUCT32 *)(ULONG_PTR)a[1];
    WOW64_PAINTSTRUCT64 P64;
    ULONGLONG Hdc;
    int i;

    if (!P32)
        return 0;
    Hdc = NtUserBeginPaint(a[0], &P64);
    P32->hdc = (ULONG)P64.hdc;
    P32->fErase = P64.fErase;
    for (i = 0; i < 4; i++) P32->rc[i] = P64.rc[i];
    P32->fRestore = P64.fRestore;
    P32->fIncUpdate = P64.fIncUpdate;
    for (i = 0; i < 32; i++) P32->rgb[i] = P64.rgb[i];
    return (NTSTATUS)(ULONG)Hdc;
}

static NTSTATUS Wow64Manual_NtUserEndPaint(ULONG *a)
{
    extern LONG NtUserEndPaint(ULONGLONG, PVOID);
    WOW64_PAINTSTRUCT32 *P32 = (WOW64_PAINTSTRUCT32 *)(ULONG_PTR)a[1];
    WOW64_PAINTSTRUCT64 P64;
    int i;

    if (!P32)
        return 0;
    P64.hdc = P32->hdc;
    P64.fErase = P32->fErase;
    for (i = 0; i < 4; i++) P64.rc[i] = P32->rc[i];
    P64.fRestore = P32->fRestore;
    P64.fIncUpdate = P32->fIncUpdate;
    for (i = 0; i < 32; i++) P64.rgb[i] = P32->rgb[i];
    P64._pad = 0;
    return NtUserEndPaint(a[0], &P64);
}

/* ---- M9: cursor/icon load path (LoadCursor(NULL, IDC_ARROW) during the
 * first window's class setup) -------------------------------------------- *
 * win32.exe registers with hCursor=NULL, so win32k loads the default arrow:
 * user32 decodes the cursor bitmap from its own resources (the StretchDIBits
 * seen on serial) then hands it to win32k via NtUserSetCursorIconData, with
 * the module + resource names as UNICODE_STRINGs. Both cursor syscalls probe
 * those strings with 8-byte alignment (TYPE_ALIGNMENT(UNICODE_STRING) on
 * Win64); a raw passthrough thunk fed the kernel a guest 32-bit UNICODE_STRING
 * (8 bytes / 4-aligned) -> ProbeForRead(.,16,8) raised DATATYPE_MISALIGNMENT
 * -> BUGCHECK 0x1E. Marshal the strings 32->64; CURSORDATA also diverges. */

/* Marshal a guest PUNICODE_STRING32 -> caller's UNICODE_STRING64. Returns the
 * 64-bit pointer, or NULL if the guest pointer was NULL. The Buffer is
 * zero-extended: a real guest string pointer widens cleanly, and a
 * MAKEINTRESOURCE atom (value < 0x10000 packed in Buffer) survives the kernel's
 * IS_ATOM low-bits check unchanged. */
static WOW64_UNICODE_STRING64_M *Wow64pUS32to64(ULONG Guest32,
                                                WOW64_UNICODE_STRING64_M *Out)
{
    WOW64_UNICODE_STRING32_M *S32 = (WOW64_UNICODE_STRING32_M *)(ULONG_PTR)Guest32;
    if (!S32)
        return NULL;
    Out->Length = S32->Length;
    Out->MaximumLength = S32->MaximumLength;
    Out->__pad = 0;
    Out->Buffer = (ULONGLONG)(ULONG_PTR)S32->Buffer;
    return Out;
}

/* NtUserFindExistingCursorIcon(pustrModule, pustrRsrc, param):
 * two UNICODE_STRINGs widen; param (FINDEXISTINGCURICONPARAM = 3 LONGs) is flat
 * and identical 32/64, so its guest pointer passes straight through. */
static NTSTATUS Wow64Manual_NtUserFindExistingCursorIcon(ULONG *a)
{
    extern HANDLE NtUserFindExistingCursorIcon(PVOID, PVOID, PVOID);
    WOW64_UNICODE_STRING64_M Mod64, Rsrc64;
    HANDLE Ret;
    Ret = NtUserFindExistingCursorIcon(Wow64pUS32to64(a[0], &Mod64),
                                       Wow64pUS32to64(a[1], &Rsrc64),
                                       (PVOID)(ULONG_PTR)a[2]);
    return (NTSTATUS)(ULONG)(ULONG_PTR)Ret;
}

/* CURSORDATA 32/64 mirrors (win32ss/include/ntuser.h tagCURSORDATA). The
 * pointer/HBITMAP fields widen 4->8; the scalar fields are identical. */
typedef struct _WOW64_CURSORDATA32 {
    ULONG lpName; ULONG lpModName;
    USHORT rt; USHORT dummy; ULONG CURSORF_flags;
    SHORT xHotspot; SHORT yHotspot;
    ULONG hbmMask; ULONG hbmColor; ULONG hbmAlpha;
    LONG rcBounds[4];
    ULONG hbmUserAlpha; ULONG bpp; ULONG cx; ULONG cy;
    ULONG cpcur; ULONG cicur;
    ULONG aspcur; ULONG aicur; ULONG ajifRate; ULONG iicur;
} WOW64_CURSORDATA32; /* 0x58 */

typedef struct _WOW64_CURSORDATA64 {
    ULONGLONG lpName; ULONGLONG lpModName;
    USHORT rt; USHORT dummy; ULONG CURSORF_flags;
    SHORT xHotspot; SHORT yHotspot; /* +4 pad to 8-align hbmMask */
    ULONGLONG hbmMask; ULONGLONG hbmColor; ULONGLONG hbmAlpha;
    LONG rcBounds[4];
    ULONGLONG hbmUserAlpha; ULONG bpp; ULONG cx; ULONG cy;
    ULONG cpcur; ULONG cicur; /* +4 pad to 8-align aspcur */
    ULONGLONG aspcur; ULONGLONG aicur; ULONGLONG ajifRate;
    UINT iicur;
} WOW64_CURSORDATA64; /* 0x88 */

#define WOW64_CURSORF_ACON 0x0008

/* NtUserSetCursorIconData(hcursor, pustrModule, pustrRsrc, pCursorData):
 * two UNICODE_STRINGs + a CURSORDATA all cross 32->64. For a static cursor
 * (the default arrow — CURSORF_ACON clear) the aspcur/aicur/ajifRate frame
 * pointers are unused; the animated case would need those guest arrays
 * re-marshalled per element (owed — loud note). */
static NTSTATUS Wow64Manual_NtUserSetCursorIconData(ULONG *a)
{
    extern LONG NtUserSetCursorIconData(HANDLE, PVOID, PVOID, PVOID);
    WOW64_UNICODE_STRING64_M Mod64, Rsrc64;
    WOW64_CURSORDATA32 *C32 = (WOW64_CURSORDATA32 *)(ULONG_PTR)a[3];
    WOW64_CURSORDATA64 C64;
    int i;

    if (C32)
    {
        C64.lpName = (ULONGLONG)(ULONG_PTR)C32->lpName;
        C64.lpModName = (ULONGLONG)(ULONG_PTR)C32->lpModName;
        C64.rt = C32->rt; C64.dummy = C32->dummy;
        C64.CURSORF_flags = C32->CURSORF_flags;
        C64.xHotspot = C32->xHotspot; C64.yHotspot = C32->yHotspot;
        C64.hbmMask = (ULONGLONG)(ULONG_PTR)C32->hbmMask;
        C64.hbmColor = (ULONGLONG)(ULONG_PTR)C32->hbmColor;
        C64.hbmAlpha = (ULONGLONG)(ULONG_PTR)C32->hbmAlpha;
        for (i = 0; i < 4; i++) C64.rcBounds[i] = C32->rcBounds[i];
        C64.hbmUserAlpha = (ULONGLONG)(ULONG_PTR)C32->hbmUserAlpha;
        C64.bpp = C32->bpp; C64.cx = C32->cx; C64.cy = C32->cy;
        C64.cpcur = C32->cpcur; C64.cicur = C32->cicur;
        C64.aspcur = (ULONGLONG)(ULONG_PTR)C32->aspcur;
        C64.aicur = (ULONGLONG)(ULONG_PTR)C32->aicur;
        C64.ajifRate = (ULONGLONG)(ULONG_PTR)C32->ajifRate;
        C64.iicur = C32->iicur;
        if (C32->CURSORF_flags & WOW64_CURSORF_ACON)
            DbgPrint("[WOW64] NtUserSetCursorIconData: ANIMATED cursor — "
                     "frame arrays not re-marshalled (owed)\n");
    }

    return NtUserSetCursorIconData((HANDLE)(LONG_PTR)(LONG)a[0],
                                   Wow64pUS32to64(a[1], &Mod64),
                                   Wow64pUS32to64(a[2], &Rsrc64),
                                   C32 ? &C64 : NULL);
}

/* Guest completion of a kernel->user callback (wow64_callback.c). */
int Wow64pCompleteUserCallback(ULONG Result32, ULONG Len32, NTSTATUS Status);

static NTSTATUS Wow64Manual_NtCallbackReturn(ULONG *a)
{
    if (!Wow64pCompleteUserCallback(a[0], a[1], (NTSTATUS)a[2]))
    {
        DbgPrint("[WOW64] NtCallbackReturn with NO callback in flight\n");
        return (NTSTATUS)0xC0000258L; /* STATUS_NO_CALLBACK_ACTIVE */
    }
    return 0; /* unreachable: CompleteUserCallback longjmps */
}

/* ======================================================================== *
 * M12: the remaining declared-manual queue.
 *
 * Shapes, and why each needs a marshaller at all:
 *   - object creators/openers: the PHANDLE out-cell is 4 bytes in the guest vs
 *     8 on the host (the kernel's aligned probe rejects the guest cell), and
 *     OBJECT_ATTRIBUTES is pointer-bearing.
 *   - CLIENT_ID: two ULONG_PTRs — 8 bytes guest, 16 host.
 *   - the IOSB takers: IO_STATUS_BLOCK ends in ULONG_PTR Information (the M10
 *     family). Same refuse-async + block-on-pend policy as M10: our host IOSB
 *     is a stack local of the thunk, so we must not return STATUS_PENDING and
 *     leave the kernel writing into a dead frame (the §9.36 class).
 * ======================================================================== */

typedef struct _WOW64_CLIENT_ID32_M {
    ULONG UniqueProcess;
    ULONG UniqueThread;
} WOW64_CLIENT_ID32_M;

typedef struct _WOW64_CLIENT_ID64_M {
    ULONGLONG UniqueProcess;
    ULONGLONG UniqueThread;
} WOW64_CLIENT_ID64_M;

/* ---- object creators / openers ------------------------------------------ */

static NTSTATUS Wow64Manual_NtCreateSemaphore(ULONG *a)
{
    ULONG *Handle32 = (ULONG *)(ULONG_PTR)a[0];
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
    St = NtCreateSemaphore(&Handle, a[1], Oa64Ptr, (LONG)a[3], (LONG)a[4]);
    if (St >= 0 && Handle32)
        *Handle32 = (ULONG)(ULONG_PTR)Handle;
    return St;
}

static NTSTATUS Wow64Manual_NtCreateMutant(ULONG *a)
{
    ULONG *Handle32 = (ULONG *)(ULONG_PTR)a[0];
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
    St = NtCreateMutant(&Handle, a[1], Oa64Ptr, (BOOLEAN)a[3]);
    if (St >= 0 && Handle32)
        *Handle32 = (ULONG)(ULONG_PTR)Handle;
    return St;
}

/* NtCreateKey: OA + optional Class (PUNICODE_STRING) + PULONG Disposition out. */
static NTSTATUS Wow64Manual_NtCreateKey(ULONG *a)
{
    ULONG *Handle32 = (ULONG *)(ULONG_PTR)a[0];
    ULONG *Disp32 = (ULONG *)(ULONG_PTR)a[6];
    WOW64_OBJECT_ATTRIBUTES64_M Oa64;
    WOW64_UNICODE_STRING64_M Uname64;
    WOW64_UNICODE_STRING64_M Class64;
    PVOID Class64Ptr = NULL;
    HANDLE Handle = NULL;
    ULONG Disposition = 0;
    NTSTATUS St;

    Wow64pMarshalOa(&Oa64, &Uname64, a[2]);
    if (a[4])
    {
        const WOW64_UNICODE_STRING32_M *Class32 =
            (const WOW64_UNICODE_STRING32_M *)(ULONG_PTR)a[4];
        Class64.Length = Class32->Length;
        Class64.MaximumLength = Class32->MaximumLength;
        Class64.__pad = 0;
        Class64.Buffer = (ULONGLONG)Class32->Buffer;
        Class64Ptr = &Class64;
    }
    St = NtCreateKey(&Handle, a[1], &Oa64, a[3], Class64Ptr, a[5], &Disposition);
    if (St >= 0 && Handle32)
        *Handle32 = (ULONG)(ULONG_PTR)Handle;
    if (Disp32)
        *Disp32 = Disposition;
    return St;
}

static NTSTATUS Wow64Manual_NtCreateSymbolicLinkObject(ULONG *a)
{
    ULONG *Handle32 = (ULONG *)(ULONG_PTR)a[0];
    WOW64_OBJECT_ATTRIBUTES64_M Oa64;
    WOW64_UNICODE_STRING64_M Uname64;
    WOW64_UNICODE_STRING64_M Target64;
    const WOW64_UNICODE_STRING32_M *Target32;
    HANDLE Handle = NULL;
    NTSTATUS St;

    if (!a[3])
        return (NTSTATUS)0xC000000DL; /* STATUS_INVALID_PARAMETER: target is required */

    Wow64pMarshalOa(&Oa64, &Uname64, a[2]);
    Target32 = (const WOW64_UNICODE_STRING32_M *)(ULONG_PTR)a[3];
    Target64.Length = Target32->Length;
    Target64.MaximumLength = Target32->MaximumLength;
    Target64.__pad = 0;
    Target64.Buffer = (ULONGLONG)Target32->Buffer;

    St = NtCreateSymbolicLinkObject(&Handle, a[1], &Oa64, &Target64);
    if (St >= 0 && Handle32)
        *Handle32 = (ULONG)(ULONG_PTR)Handle;
    return St;
}

/* NtOpenProcess/NtOpenThread: OA + optional CLIENT_ID (two ULONG_PTRs). */
typedef NTSTATUS (WINAPI *WOW64_OPEN_BY_CID_FN)(PHANDLE, ACCESS_MASK, PVOID, PVOID);
static NTSTATUS Wow64pOpenByCid(WOW64_OPEN_BY_CID_FN Fn, ULONG *a)
{
    ULONG *Handle32 = (ULONG *)(ULONG_PTR)a[0];
    WOW64_OBJECT_ATTRIBUTES64_M Oa64;
    WOW64_UNICODE_STRING64_M Uname64;
    WOW64_CLIENT_ID64_M Cid64;
    PVOID Cid64Ptr = NULL;
    HANDLE Handle = NULL;
    NTSTATUS St;

    Wow64pMarshalOa(&Oa64, &Uname64, a[2]);
    if (a[3])
    {
        const WOW64_CLIENT_ID32_M *Cid32 =
            (const WOW64_CLIENT_ID32_M *)(ULONG_PTR)a[3];
        Cid64.UniqueProcess = (ULONGLONG)Cid32->UniqueProcess;
        Cid64.UniqueThread = (ULONGLONG)Cid32->UniqueThread;
        Cid64Ptr = &Cid64;
    }
    St = Fn(&Handle, a[1], &Oa64, Cid64Ptr);
    if (St >= 0 && Handle32)
        *Handle32 = (ULONG)(ULONG_PTR)Handle;
    return St;
}

/* ---- the remaining IO_STATUS_BLOCK takers (the M10 family, continued) ----- */

static NTSTATUS Wow64Manual_NtCancelIoFile(ULONG *a)
{
    HANDLE File = (HANDLE)(LONG_PTR)(LONG)a[0];
    WOW64_IO_STATUS_BLOCK64_M Iosb64 = {0};
    NTSTATUS St = NtCancelIoFile(File, &Iosb64);
    if (St < 0)
        Iosb64.Status = (ULONG)St;
    Wow64pCopyIosbDown(&Iosb64, a[1]);
    return St;
}

static NTSTATUS Wow64Manual_NtLockFile(ULONG *a)
{
    HANDLE File = (HANDLE)(LONG_PTR)(LONG)a[0];
    WOW64_IO_STATUS_BLOCK64_M Iosb64 = {0};
    NTSTATUS St;

    if (Wow64pRefuseAsync(a[1], a[2], "NtLockFile"))
        return WOW64_STATUS_NOT_IMPLEMENTED;
    /* ByteOffset/Length are LARGE_INTEGER* — layout-identical. Key is by value. */
    St = NtLockFile(File, NULL, NULL, NULL, &Iosb64,
                    (PLARGE_INTEGER)(ULONG_PTR)a[5], (PLARGE_INTEGER)(ULONG_PTR)a[6],
                    a[7], (BOOLEAN)a[8], (BOOLEAN)a[9]);
    return Wow64pFinishIosb(St, File, &Iosb64, a[4], "NtLockFile");
}

static NTSTATUS Wow64Manual_NtUnlockFile(ULONG *a)
{
    HANDLE File = (HANDLE)(LONG_PTR)(LONG)a[0];
    WOW64_IO_STATUS_BLOCK64_M Iosb64 = {0};
    NTSTATUS St = NtUnlockFile(File, &Iosb64, (PLARGE_INTEGER)(ULONG_PTR)a[2],
                               (PLARGE_INTEGER)(ULONG_PTR)a[3], a[4]);
    if (St < 0)
        Iosb64.Status = (ULONG)St;
    Wow64pCopyIosbDown(&Iosb64, a[1]);
    return St;
}

/* FS_*_INFORMATION set classes are LARGE_INTEGER/ULONG/WCHAR only — the buffer
 * passes through unchanged; only the IOSB needs marshalling. */
static NTSTATUS Wow64Manual_NtSetVolumeInformationFile(ULONG *a)
{
    HANDLE File = (HANDLE)(LONG_PTR)(LONG)a[0];
    WOW64_IO_STATUS_BLOCK64_M Iosb64 = {0};
    NTSTATUS St = NtSetVolumeInformationFile(File, &Iosb64,
                                             (PVOID)(ULONG_PTR)a[2], a[3], a[4]);
    if (St < 0)
        Iosb64.Status = (ULONG)St;
    Wow64pCopyIosbDown(&Iosb64, a[1]);
    return St;
}

/* FILE_FULL_EA_INFORMATION / FILE_GET_EA_INFORMATION are ULONG/UCHAR/CHAR only
 * — width-identical, so the buffers pass straight through. */
static NTSTATUS Wow64Manual_NtQueryEaFile(ULONG *a)
{
    HANDLE File = (HANDLE)(LONG_PTR)(LONG)a[0];
    WOW64_IO_STATUS_BLOCK64_M Iosb64 = {0};
    NTSTATUS St = NtQueryEaFile(File, &Iosb64, (PVOID)(ULONG_PTR)a[2], a[3],
                                (BOOLEAN)a[4], (PVOID)(ULONG_PTR)a[5], a[6],
                                (PULONG)(ULONG_PTR)a[7], (BOOLEAN)a[8]);
    if (St < 0)
        Iosb64.Status = (ULONG)St;
    Wow64pCopyIosbDown(&Iosb64, a[1]);
    return St;
}

static NTSTATUS Wow64Manual_NtSetEaFile(ULONG *a)
{
    HANDLE File = (HANDLE)(LONG_PTR)(LONG)a[0];
    WOW64_IO_STATUS_BLOCK64_M Iosb64 = {0};
    NTSTATUS St = NtSetEaFile(File, &Iosb64, (PVOID)(ULONG_PTR)a[2], a[3]);
    if (St < 0)
        Iosb64.Status = (ULONG)St;
    Wow64pCopyIosbDown(&Iosb64, a[1]);
    return St;
}

/* NtDeviceIoControlFile / NtFsControlFile.
 *
 * The in/out buffers are IOCTL-specific and OPAQUE to us: most are flat
 * (width-identical) and pass through, but a minority embed pointers/ULONG_PTRs
 * and would need per-code marshalling. We can't know which from here, so we
 * marshal the IOSB (which we CAN see) and pass the buffers through. If a future
 * guest sends a pointer-bearing IOCTL, the failure will be a bad pointer inside
 * the driver — grep this comment when that happens. */
typedef NTSTATUS (WINAPI *WOW64_IOCTL_FN)(HANDLE, HANDLE, PVOID, PVOID, PVOID,
                                          ULONG, PVOID, ULONG, PVOID, ULONG);
static NTSTATUS Wow64pIoControl(WOW64_IOCTL_FN Fn, ULONG *a, const char *Who)
{
    HANDLE File = (HANDLE)(LONG_PTR)(LONG)a[0];
    WOW64_IO_STATUS_BLOCK64_M Iosb64 = {0};
    NTSTATUS St;

    if (Wow64pRefuseAsync(a[1], a[2], Who))
        return WOW64_STATUS_NOT_IMPLEMENTED;
    St = Fn(File, NULL, NULL, NULL, &Iosb64, a[5],
            (PVOID)(ULONG_PTR)a[6], a[7], (PVOID)(ULONG_PTR)a[8], a[9]);
    return Wow64pFinishIosb(St, File, &Iosb64, a[4], Who);
}

static NTSTATUS Wow64Manual_NtCreateNamedPipeFile(ULONG *a)
{
    ULONG *Handle32 = (ULONG *)(ULONG_PTR)a[0];
    WOW64_OBJECT_ATTRIBUTES64_M Oa64;
    WOW64_UNICODE_STRING64_M Uname64;
    WOW64_IO_STATUS_BLOCK64_M Iosb64 = {0};
    HANDLE Handle = NULL;
    NTSTATUS St;

    Wow64pMarshalOa(&Oa64, &Uname64, a[2]);
    St = NtCreateNamedPipeFile(&Handle, a[1], &Oa64, &Iosb64, a[4], a[5], a[6],
                               a[7], a[8], a[9], a[10], a[11], a[12],
                               (PLARGE_INTEGER)(ULONG_PTR)a[13]);
    if (St >= 0 && Handle32)
        *Handle32 = (ULONG)(ULONG_PTR)Handle;
    if (St < 0)
        Iosb64.Status = (ULONG)St;
    Wow64pCopyIosbDown(&Iosb64, a[3]);
    return St;
}

static NTSTATUS Wow64Manual_NtCreateMailslotFile(ULONG *a)
{
    ULONG *Handle32 = (ULONG *)(ULONG_PTR)a[0];
    WOW64_OBJECT_ATTRIBUTES64_M Oa64;
    WOW64_UNICODE_STRING64_M Uname64;
    WOW64_IO_STATUS_BLOCK64_M Iosb64 = {0};
    HANDLE Handle = NULL;
    NTSTATUS St;

    Wow64pMarshalOa(&Oa64, &Uname64, a[2]);
    St = NtCreateMailslotFile(&Handle, a[1], &Oa64, &Iosb64, a[4], a[5], a[6],
                              (PLARGE_INTEGER)(ULONG_PTR)a[7]);
    if (St >= 0 && Handle32)
        *Handle32 = (ULONG)(ULONG_PTR)Handle;
    if (St < 0)
        Iosb64.Status = (ULONG)St;
    Wow64pCopyIosbDown(&Iosb64, a[3]);
    return St;
}

/* ---- explicit loud refusals --------------------------------------------- *
 * These are reachable but cannot be done correctly yet. Refusing loudly beats
 * auto-thunking them straight through (which silently corrupts) and beats a
 * generic "NEEDS MANUAL" line (which doesn't say WHY). Each names its blocker. */
static NTSTATUS Wow64pRefuse(const char *Who, const char *Why)
{
    DbgPrint("[WOW64]   %s is not supported under WoW64 yet: %s — refusing\n", Who, Why);
    return WOW64_STATUS_NOT_IMPLEMENTED;
}

static NTSTATUS Wow64Manual_NtNotifyChangeDirectoryFile(ULONG *a)
{
    (void)a;
    return Wow64pRefuse("NtNotifyChangeDirectoryFile",
                        "it is inherently async (the IOSB outlives the call), which "
                        "needs the kernel-side IO_STATUS_BLOCK32 fix");
}

static NTSTATUS Wow64Manual_NtNotifyChangeKey(ULONG *a)
{
    (void)a;
    return Wow64pRefuse("NtNotifyChangeKey", "inherently async — same as "
                        "NtNotifyChangeDirectoryFile");
}

static NTSTATUS Wow64Manual_NtNotifyChangeMultipleKeys(ULONG *a)
{
    (void)a;
    return Wow64pRefuse("NtNotifyChangeMultipleKeys", "inherently async — same as "
                        "NtNotifyChangeDirectoryFile");
}

static NTSTATUS Wow64Manual_NtSetSystemInformation(ULONG *a)
{
    (void)a;
    return Wow64pRefuse("NtSetSystemInformation",
                        "the info buffer's layout is class-dependent and several "
                        "classes are pointer-bearing; passing one through unchecked "
                        "would let a guest corrupt kernel state");
}

/* M16 (task #43): NtUserMessageCall(hWnd, Msg, wParam, lParam, ResultInfo, FNID,
 * Ansi). For FNID_DEFWINDOWPROC + WM_WINDOWPOSCHANG*, lParam is a WINDOWPOS the
 * 64-bit win32k reads/writes as 40 bytes but the guest passes as 28 — the flat
 * thunk hands win32k a misaligned struct (fields shift past the two leading
 * handles), so DefWindowProc's size adjustment corrupts the window (collapses to
 * 0x0, WM_PAINT never fires). Marshal the payload 32<->64 for this (FNID,Msg);
 * every other message flat-passes exactly as the generated thunk did. See
 * docs/engineering/active-lanes/wow64-plan.md (M16) + user32 defwnd.c:783. */
#define WOW64_MC_FNID_DEFWINDOWPROC   0x029E
#define WOW64_MC_WM_WINDOWPOSCHANGING 0x0046
#define WOW64_MC_WM_WINDOWPOSCHANGED  0x0047
typedef struct { ULONG hwnd, hwndInsertAfter; LONG x, y, cx, cy; ULONG flags; }
    WOW64_MC_WP32; /* 28 */
typedef struct { ULONGLONG hwnd, hwndInsertAfter; LONG x, y, cx, cy; ULONG flags, __pad; }
    WOW64_MC_WP64; /* 40 */

extern ULONG Wow64TraceSyscalls;

static NTSTATUS Wow64Manual_NtUserMessageCall(ULONG *a)
{
    extern NTSTATUS NtUserMessageCall(PVOID, ULONG_PTR, PVOID, PVOID, PVOID,
                                      ULONG_PTR, ULONG_PTR);
    ULONG Msg = a[1];
    ULONG FNID = a[5];

    if (FNID == WOW64_MC_FNID_DEFWINDOWPROC &&
        (Msg == WOW64_MC_WM_WINDOWPOSCHANGING || Msg == WOW64_MC_WM_WINDOWPOSCHANGED) &&
        a[3])
    {
        WOW64_MC_WP32 *wp32 = (WOW64_MC_WP32 *)(ULONG_PTR)a[3];
        WOW64_MC_WP64 wp64;
        NTSTATUS St;
        LONG in_cx = wp32->cx, in_cy = wp32->cy;
        ULONG in_fl = wp32->flags;

        /* 28 -> 40, per field. Handles sign-extend (real handles are small +ve
         * and stay correct; HWND_TOPMOST/-NOTOPMOST sentinels widen correctly). */
        wp64.hwnd = (ULONGLONG)(LONG_PTR)(LONG)wp32->hwnd;
        wp64.hwndInsertAfter = (ULONGLONG)(LONG_PTR)(LONG)wp32->hwndInsertAfter;
        wp64.x = wp32->x; wp64.y = wp32->y;
        wp64.cx = wp32->cx; wp64.cy = wp32->cy;
        wp64.flags = wp32->flags; wp64.__pad = 0;

        St = NtUserMessageCall((PVOID)(LONG_PTR)(LONG)a[0], (ULONG_PTR)(LONG_PTR)(LONG)a[1],
                               (PVOID)(LONG_PTR)(LONG)a[2], &wp64,
                               (PVOID)(LONG_PTR)(LONG)a[4], (ULONG_PTR)(LONG_PTR)(LONG)a[5],
                               (ULONG_PTR)(LONG_PTR)(LONG)a[6]);

        /* 40 -> 28 back. Preserve the guest's original 64-bit-narrowed handles
         * (win32k does not repoint them for this message). */
        wp32->x = wp64.x; wp32->y = wp64.y;
        wp32->cx = wp64.cx; wp32->cy = wp64.cy;
        wp32->flags = wp64.flags;
        wp32->hwndInsertAfter = (ULONG)wp64.hwndInsertAfter;

        if (Wow64TraceSyscalls)
            DbgPrint("[WOW64] NtUserMessageCall DWP WINDOWPOS marshalled "
                     "(in cx=%ld cy=%ld fl=0x%lx -> out cx=%ld cy=%ld fl=0x%lx)\n",
                     (long)in_cx, (long)in_cy, (unsigned long)in_fl,
                     (long)wp32->cx, (long)wp32->cy, (unsigned long)wp32->flags);
        return St;
    }

    /* Flat pass-through — byte-identical to the generated thunk. */
    return NtUserMessageCall((PVOID)(LONG_PTR)(LONG)a[0], (ULONG_PTR)(LONG_PTR)(LONG)a[1],
                             (PVOID)(LONG_PTR)(LONG)a[2], (PVOID)(LONG_PTR)(LONG)a[3],
                             (PVOID)(LONG_PTR)(LONG)a[4], (ULONG_PTR)(LONG_PTR)(LONG)a[5],
                             (ULONG_PTR)(LONG_PTR)(LONG)a[6]);
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
    if (WOW64_IS("NtReadFile"))
        return Wow64Manual_NtReadFile(Arguments);
    if (WOW64_IS("NtWriteFile"))
        return Wow64Manual_NtWriteFile(Arguments);
    if (WOW64_IS("NtQueryInformationFile"))
        return Wow64Manual_NtQueryInformationFile(Arguments);
    if (WOW64_IS("NtSetInformationFile"))
        return Wow64Manual_NtSetInformationFile(Arguments);
    if (WOW64_IS("NtQueryVolumeInformationFile"))
        return Wow64Manual_NtQueryVolumeInformationFile(Arguments);
    if (WOW64_IS("NtFlushBuffersFile"))
        return Wow64Manual_NtFlushBuffersFile(Arguments);
    if (WOW64_IS("NtQueryDirectoryFile"))
        return Wow64Manual_NtQueryDirectoryFile(Arguments);
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
    if (WOW64_IS("NtUserMessageCall"))
        return Wow64Manual_NtUserMessageCall(Arguments);
    if (WOW64_IS("NtQueryValueKey"))
        return Wow64Manual_NtQueryValueKey(Arguments);
    if (WOW64_IS("NtUserRegisterClassExWOW"))
        return Wow64Manual_NtUserRegisterClassExWOW(Arguments);
    if (WOW64_IS("NtUserCreateWindowEx"))
        return Wow64Manual_NtUserCreateWindowEx(Arguments);
    if (WOW64_IS("NtUserGetMessage"))
        return Wow64Manual_NtUserGetMessage(Arguments);
    if (WOW64_IS("NtUserPeekMessage"))
        return Wow64Manual_NtUserPeekMessage(Arguments);
    if (WOW64_IS("NtUserDispatchMessage"))
        return Wow64Manual_NtUserDispatchMessage(Arguments);
    if (WOW64_IS("NtUserTranslateMessage"))
        return Wow64Manual_NtUserTranslateMessage(Arguments);
    if (WOW64_IS("NtUserBeginPaint"))
        return Wow64Manual_NtUserBeginPaint(Arguments);
    if (WOW64_IS("NtUserEndPaint"))
        return Wow64Manual_NtUserEndPaint(Arguments);
    if (WOW64_IS("NtUserFindExistingCursorIcon"))
        return Wow64Manual_NtUserFindExistingCursorIcon(Arguments);
    if (WOW64_IS("NtUserSetCursorIconData"))
        return Wow64Manual_NtUserSetCursorIconData(Arguments);
    if (WOW64_IS("NtUserDefSetText"))
        return Wow64Manual_NtUserDefSetText(Arguments);
    if (WOW64_IS("NtCallbackReturn"))
        return Wow64Manual_NtCallbackReturn(Arguments);

    /* M12: object creators/openers. */
    if (WOW64_IS("NtCreateSemaphore"))
        return Wow64Manual_NtCreateSemaphore(Arguments);
    if (WOW64_IS("NtOpenSemaphore"))
        return Wow64pOpenByOa(NtOpenSemaphore, Arguments);
    if (WOW64_IS("NtCreateMutant"))
        return Wow64Manual_NtCreateMutant(Arguments);
    if (WOW64_IS("NtOpenMutant"))
        return Wow64pOpenByOa(NtOpenMutant, Arguments);
    if (WOW64_IS("NtOpenKeyedEvent"))
        return Wow64pOpenByOa(NtOpenKeyedEvent, Arguments);
    if (WOW64_IS("NtCreateKey"))
        return Wow64Manual_NtCreateKey(Arguments);
    if (WOW64_IS("NtCreateSymbolicLinkObject"))
        return Wow64Manual_NtCreateSymbolicLinkObject(Arguments);
    if (WOW64_IS("NtOpenProcess"))
        return Wow64pOpenByCid(NtOpenProcess, Arguments);
    if (WOW64_IS("NtOpenThread"))
        return Wow64pOpenByCid(NtOpenThread, Arguments);

    /* M12: the rest of the IO_STATUS_BLOCK family. */
    if (WOW64_IS("NtCancelIoFile"))
        return Wow64Manual_NtCancelIoFile(Arguments);
    if (WOW64_IS("NtLockFile"))
        return Wow64Manual_NtLockFile(Arguments);
    if (WOW64_IS("NtUnlockFile"))
        return Wow64Manual_NtUnlockFile(Arguments);
    if (WOW64_IS("NtSetVolumeInformationFile"))
        return Wow64Manual_NtSetVolumeInformationFile(Arguments);
    if (WOW64_IS("NtQueryEaFile"))
        return Wow64Manual_NtQueryEaFile(Arguments);
    if (WOW64_IS("NtSetEaFile"))
        return Wow64Manual_NtSetEaFile(Arguments);
    if (WOW64_IS("NtDeviceIoControlFile"))
        return Wow64pIoControl(NtDeviceIoControlFile, Arguments, "NtDeviceIoControlFile");
    if (WOW64_IS("NtFsControlFile"))
        return Wow64pIoControl(NtFsControlFile, Arguments, "NtFsControlFile");
    if (WOW64_IS("NtCreateNamedPipeFile"))
        return Wow64Manual_NtCreateNamedPipeFile(Arguments);
    if (WOW64_IS("NtCreateMailslotFile"))
        return Wow64Manual_NtCreateMailslotFile(Arguments);

    /* M12: loud refusals — reachable, but not correctly doable yet. */
    if (WOW64_IS("NtNotifyChangeDirectoryFile"))
        return Wow64Manual_NtNotifyChangeDirectoryFile(Arguments);
    if (WOW64_IS("NtNotifyChangeKey"))
        return Wow64Manual_NtNotifyChangeKey(Arguments);
    if (WOW64_IS("NtNotifyChangeMultipleKeys"))
        return Wow64Manual_NtNotifyChangeMultipleKeys(Arguments);
    if (WOW64_IS("NtSetSystemInformation"))
        return Wow64Manual_NtSetSystemInformation(Arguments);
#undef WOW64_IS
    /* M17: fall through to the marshalgen-generated thunks (flat thunks
     * recovered from header prototypes + layout-diff deep marshallers).
     * Hand-written marshallers above always take precedence. */
    {
        extern NTSTATUS Wow64AutoMarshalDispatch(const char *, ULONG *);
        return Wow64AutoMarshalDispatch(Name, Arguments);
    }
}

int Wow64pStrEq(const char *a, const char *b)
{
    while (*a && *a == *b) { a++; b++; }
    return *a == *b;
}
