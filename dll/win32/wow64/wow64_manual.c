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
#undef WOW64_IS
    return WOW64_MANUAL_UNIMPL;
}

int Wow64pStrEq(const char *a, const char *b)
{
    while (*a && *a == *b) { a++; b++; }
    return *a == *b;
}
