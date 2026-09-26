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

#include "wow64_btcpu.h"

ULONG WINAPI DbgPrint(_In_z_ _Printf_format_string_ PCSTR Format, ...);

#include "wow64_status.h"
#include "wow64_pfnmap.h"

int Wow64pStrEq(const char *a, const char *b);

/* ---- host (64-bit) Nt* prototypes we marshal into ----------------------- */
NTSTATUS WINAPI NtQueryVirtualMemory(HANDLE ProcessHandle, PVOID BaseAddress,
                                     ULONG MemoryInformationClass,
                                     PVOID MemoryInformation,
                                     SIZE_T MemoryInformationLength,
                                     PSIZE_T ReturnLength);
NTSTATUS WINAPI NtQueryInformationProcess(HANDLE ProcessHandle, ULONG InfoClass,
                                          PVOID Info, ULONG Len, PULONG RetLen);
NTSTATUS WINAPI NtQueryInformationThread(HANDLE ThreadHandle, ULONG InfoClass,
                                         PVOID Info, ULONG Len, PULONG RetLen);
NTSTATUS WINAPI NtOpenProcessTokenEx(HANDLE ProcessHandle, ACCESS_MASK Access,
                                     ULONG HandleAttributes, PHANDLE TokenHandle);
NTSTATUS WINAPI NtOpenThreadTokenEx(HANDLE ThreadHandle, ACCESS_MASK Access,
                                    BOOLEAN OpenAsSelf, ULONG HandleAttributes,
                                    PHANDLE TokenHandle);
NTSTATUS WINAPI NtQuerySystemInformation(ULONG InfoClass, PVOID Info, ULONG Len,
                                         PULONG RetLen);
/* #64 (initial-thread half): RtlCreateUserThread takes a FOREIGN ProcessHandle
 * and does the stack allocation + host CONTEXT build in that process, which is
 * the whole reason the cross-process initial thread needs no new machinery. */
LONG WINAPI RtlCreateUserThread(HANDLE ProcessHandle, PVOID SecurityDescriptor,
                                BOOLEAN CreateSuspended, ULONG StackZeroBits,
                                SIZE_T StackReserve, SIZE_T StackCommit,
                                PVOID StartAddress, PVOID Parameter,
                                PHANDLE ThreadHandle, PVOID ClientId);
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

/* The out-cell family (see "a 4-byte guest cell, an 8-byte kernel write"
 * below). OBJECT_ATTRIBUTES stays PVOID here for the same reason as the
 * declarations above: this TU deliberately does not pull in the NDK. */
NTSTATUS WINAPI NtOpenProcessToken(HANDLE Process, ACCESS_MASK Access,
                                   PHANDLE TokenHandle);
NTSTATUS WINAPI NtOpenThreadToken(HANDLE Thread, ACCESS_MASK Access,
                                  BOOLEAN OpenAsSelf, PHANDLE TokenHandle);
NTSTATUS WINAPI NtDuplicateObject(HANDLE SourceProcess, HANDLE SourceHandle,
                                  HANDLE TargetProcess, PHANDLE TargetHandle,
                                  ACCESS_MASK Access, ULONG Attributes,
                                  ULONG Options);
NTSTATUS WINAPI NtDuplicateToken(HANDLE ExistingToken, ACCESS_MASK Access,
                                 PVOID ObjectAttributes, BOOLEAN EffectiveOnly,
                                 ULONG TokenType, PHANDLE NewTokenHandle);
NTSTATUS WINAPI NtCreateEventPair(PHANDLE Handle, ACCESS_MASK Access, PVOID Oa);
NTSTATUS WINAPI NtOpenEventPair(PHANDLE Handle, ACCESS_MASK Access, PVOID Oa);
NTSTATUS WINAPI NtCreateJobObject(PHANDLE Handle, ACCESS_MASK Access, PVOID Oa);
NTSTATUS WINAPI NtOpenJobObject(PHANDLE Handle, ACCESS_MASK Access, PVOID Oa);
NTSTATUS WINAPI NtOpenTimer(PHANDLE Handle, ACCESS_MASK Access, PVOID Oa);
NTSTATUS WINAPI NtCreateTimer(PHANDLE Handle, ACCESS_MASK Access, PVOID Oa,
                              ULONG TimerType);
NTSTATUS WINAPI NtCreateDebugObject(PHANDLE Handle, ACCESS_MASK Access,
                                    PVOID Oa, ULONG Flags);
NTSTATUS WINAPI NtCreatePort(PHANDLE Handle, PVOID Oa, ULONG MaxConnectInfo,
                             ULONG MaxMessage, ULONG MaxPoolUsage);
NTSTATUS WINAPI NtCreateWaitablePort(PHANDLE Handle, PVOID Oa,
                                     ULONG MaxConnectInfo, ULONG MaxMessage,
                                     ULONG MaxPoolUsage);
NTSTATUS WINAPI NtCreateProcess(PHANDLE Handle, ACCESS_MASK Access, PVOID Oa,
                                HANDLE Parent, BOOLEAN InheritHandles,
                                HANDLE Section, HANDLE DebugPort,
                                HANDLE ExceptionPort);
NTSTATUS WINAPI NtCreateProcessEx(PHANDLE Handle, ACCESS_MASK Access, PVOID Oa,
                                  HANDLE Parent, ULONG Flags, HANDLE Section,
                                  HANDLE DebugPort, HANDLE ExceptionPort,
                                  BOOLEAN InJob);
NTSTATUS WINAPI NtCreateProfile(PHANDLE Handle, HANDLE Process, PVOID ImageBase,
                                SIZE_T ImageSize, ULONG Granularity,
                                PVOID Buffer, ULONG ProfilingSize,
                                ULONG Source, ULONG_PTR Affinity);
NTSTATUS WINAPI NtLockVirtualMemory(HANDLE Process, PVOID *BaseAddress,
                                    PSIZE_T NumberOfBytes, ULONG MapType);
NTSTATUS WINAPI NtUnlockVirtualMemory(HANDLE Process, PVOID *BaseAddress,
                                      PSIZE_T NumberOfBytes, ULONG MapType);
NTSTATUS WINAPI NtFlushVirtualMemory(HANDLE Process, PVOID *BaseAddress,
                                     PSIZE_T RegionSize, PVOID IoStatus);

/* The bare-PUNICODE_STRING family (see "a flat thunk reads the string pointer
 * from past the end of the guest's struct" below). PUNICODE_STRING and
 * POBJECT_ATTRIBUTES are PVOID here for the same reason as above. */
NTSTATUS WINAPI NtSetValueKey(HANDLE Key, PVOID ValueName, ULONG TitleIndex,
                              ULONG Type, PVOID Data, ULONG DataSize);
NTSTATUS WINAPI NtDeleteValueKey(HANDLE Key, PVOID ValueName);
NTSTATUS WINAPI NtRenameKey(HANDLE Key, PVOID NewName);
NTSTATUS WINAPI NtDeleteFile(PVOID ObjectAttributes);
NTSTATUS WINAPI NtDisplayString(PVOID String);
NTSTATUS WINAPI NtLoadDriver(PVOID DriverServiceName);
NTSTATUS WINAPI NtUnloadDriver(PVOID DriverServiceName);
NTSTATUS WINAPI NtQuerySystemEnvironmentValue(PVOID Name, PVOID Value,
                                              ULONG Length, PULONG RetLength);
NTSTATUS WINAPI NtSetSystemEnvironmentValue(PVOID Name, PVOID Value);
NTSTATUS WINAPI NtQuerySystemEnvironmentValueEx(PVOID Name, PVOID Guid,
                                                PVOID Value, PULONG RetLength,
                                                PULONG Attributes);
NTSTATUS WINAPI NtSetSystemEnvironmentValueEx(PVOID Name, PVOID Guid,
                                              PVOID Value, ULONG Length,
                                              ULONG Attributes);
NTSTATUS WINAPI NtCreatePagingFile(PVOID FileName, PVOID MinSize,
                                   PVOID MaxSize, ULONG Reserved);
NTSTATUS WINAPI NtLoadKey(PVOID KeyOa, PVOID FileOa);
NTSTATUS WINAPI NtLoadKey2(PVOID KeyOa, PVOID FileOa, ULONG Flags);
NTSTATUS WINAPI NtLoadKeyEx(PVOID TargetKey, PVOID SourceFile, ULONG Flags,
                            HANDLE TrustClassKey);
NTSTATUS WINAPI NtReplaceKey(PVOID NewFileOa, HANDLE Key, PVOID OldFileOa);
NTSTATUS WINAPI NtUnloadKey(PVOID KeyOa);
NTSTATUS WINAPI NtUnloadKey2(PVOID TargetKey, ULONG Flags);
NTSTATUS WINAPI NtUnloadKeyEx(PVOID TargetKey, HANDLE Event);
NTSTATUS WINAPI NtQueryOpenSubKeys(PVOID TargetKey, PULONG HandleCount);
NTSTATUS WINAPI NtQueryOpenSubKeysEx(PVOID TargetKey, ULONG BufferLength,
                                     PVOID Buffer, PULONG RequiredSize);
NTSTATUS WINAPI NtAccessCheckAndAuditAlarm(PVOID SubsystemName, PVOID HandleId,
                                           PVOID ObjectTypeName,
                                           PVOID ObjectName, PVOID Sd,
                                           ACCESS_MASK Desired, PVOID Mapping,
                                           BOOLEAN ObjectCreation,
                                           PVOID GrantedAccess,
                                           PVOID AccessStatus,
                                           PVOID GenerateOnClose);
NTSTATUS WINAPI NtCloseObjectAuditAlarm(PVOID SubsystemName, PVOID HandleId,
                                        BOOLEAN GenerateOnClose);
NTSTATUS WINAPI NtDeleteObjectAuditAlarm(PVOID SubsystemName, PVOID HandleId,
                                         BOOLEAN GenerateOnClose);
NTSTATUS WINAPI NtOpenObjectAuditAlarm(PVOID SubsystemName, PVOID HandleId,
                                       PVOID ObjectTypeName, PVOID ObjectName,
                                       PVOID Sd, HANDLE ClientToken,
                                       ACCESS_MASK Desired, ACCESS_MASK Granted,
                                       PVOID Privileges, BOOLEAN ObjectCreation,
                                       BOOLEAN AccessGranted,
                                       PVOID GenerateOnClose);
NTSTATUS WINAPI NtPrivilegeObjectAuditAlarm(PVOID SubsystemName, PVOID HandleId,
                                            HANDLE ClientToken,
                                            ACCESS_MASK Desired,
                                            PVOID Privileges,
                                            BOOLEAN AccessGranted);
NTSTATUS WINAPI NtPrivilegedServiceAuditAlarm(PVOID SubsystemName,
                                              PVOID ServiceName,
                                              HANDLE ClientToken,
                                              PVOID Privileges,
                                              BOOLEAN AccessGranted);
NTSTATUS WINAPI NtProtectVirtualMemory(HANDLE Process, PVOID *BaseAddress,
                                       PSIZE_T RegionSize, ULONG NewProtect,
                                       PULONG OldProtect);
NTSTATUS WINAPI NtMapViewOfSection(HANDLE Section, HANDLE Process,
                                   PVOID *BaseAddress, ULONG_PTR ZeroBits,
                                   SIZE_T CommitSize, PLARGE_INTEGER SectionOffset,
                                   PSIZE_T ViewSize, ULONG InheritDisposition,
                                   ULONG AllocationType, ULONG Win32Protect);
NTSTATUS WINAPI NtUnmapViewOfSection(HANDLE Process, PVOID BaseAddress);
NTSTATUS WINAPI NtReadVirtualMemory(HANDLE Process, PVOID BaseAddress,
                                    PVOID Buffer, SIZE_T Size, PSIZE_T Read);
NTSTATUS WINAPI NtWriteVirtualMemory(HANDLE Process, PVOID BaseAddress,
                                     PVOID Buffer, SIZE_T Size, PSIZE_T Written);
NTSTATUS WINAPI NtFlushInstructionCache(HANDLE Process, PVOID BaseAddress,
                                        SIZE_T Length);
NTSTATUS WINAPI NtSetInformationProcess(HANDLE Process, ULONG InfoClass,
                                        PVOID Info, ULONG Len);
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
NTSTATUS WINAPI NtCreateIoCompletion(PHANDLE Handle, ACCESS_MASK Access, PVOID Oa,
                                     ULONG Count);
NTSTATUS WINAPI NtOpenIoCompletion(PHANDLE Handle, ACCESS_MASK Access, PVOID Oa);
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
NTSTATUS WINAPI NtQueryInformationThread(HANDLE Thread, ULONG Class, PVOID Info,
                                         ULONG Length, PULONG ReturnLength);
NTSTATUS WINAPI NtRemoveIoCompletion(HANDLE Port, PVOID *Key, PVOID *Context,
                                     PVOID Iosb, PLARGE_INTEGER Timeout);
INT WINAPI NtUserEnumDisplayMonitors(HANDLE hDC, PVOID pRect,
                                     PVOID hMonitorList, PVOID monitorRectList,
                                     ULONG listSize);
NTSTATUS WINAPI NtQueryDirectoryFile(HANDLE File, HANDLE Event, PVOID ApcRoutine,
                                     PVOID ApcContext, PVOID Iosb, PVOID Info,
                                     ULONG Length, ULONG Class, BOOLEAN SingleEntry,
                                     PVOID FileName, BOOLEAN RestartScan);
NTSTATUS WINAPI NtWaitForSingleObject(HANDLE Object, BOOLEAN Alertable,
                                      PLARGE_INTEGER Timeout);
NTSTATUS WINAPI NtSetEvent(HANDLE Event, PLONG PreviousState);
NTSTATUS WINAPI NtWaitForMultipleObjects(ULONG Count, HANDLE Objects[],
                                         ULONG WaitType, BOOLEAN Alertable,
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
    if (!Oa32)
    {
        /* A guest may legally pass NULL where the parameter is _In_opt_, and an
         * ill-behaved one may pass it where it is not. Hand the kernel a valid
         * empty OA rather than dereferencing zero inside the thunk: a bad guest
         * argument must come back as a status code, not as a fault in wow64.dll.
         * Callers whose parameter is genuinely optional use the Opt form below,
         * which passes a real NULL through. */
        Oa64->RootDirectory = 0;
        Oa64->Attributes = 0;
        Oa64->__pad1 = 0;
        Oa64->SecurityDescriptor = 0;
        Oa64->SecurityQualityOfService = 0;
        Oa64->ObjectName = 0;
        return;
    }
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

#define WOW64_ProcessExecuteFlags 0x22

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

/* ---- the out-cell class: a 4-byte guest cell, an 8-byte kernel write ----- *
 *
 * These syscalls hand the kernel a guest pointer to somewhere it stores a
 * POINTER-SIZED result — a HANDLE, a base address, a region size. A flat thunk
 * passes the guest's cell straight through (a 32-bit guest pointer is a valid
 * 64-bit pointer, so nothing complains), and the kernel's 8-byte store then
 * runs FOUR BYTES PAST the variable the guest declared. On a `HANDLE h;` local
 * that is the next slot on the guest stack; in a struct it is the next field.
 *
 * It is invisible for the same reason the IO_STATUS_BLOCK overflow was: the
 * call SUCCEEDS, and the value the guest asked for arrives correctly in the
 * low half. Only the collateral write is wrong, so the damage surfaces later
 * and somewhere else. `marshalgen audit` is what found these — the whole point
 * of that gate is that a flat thunk over a divergent payload cannot be seen by
 * reading the code.
 *
 * Note NtOpenProcessTokenEx and NtOpenThreadTokenEx were already marshalled
 * while the plain variants below were not: this class is easy to close for the
 * call you happen to be looking at and easy to miss everywhere else, which is
 * exactly why the fix is driven off the audit list rather than off memory.
 *
 * Each marshaller is the same three lines — take the result in a host cell,
 * narrow it into the guest's. Only the argument slot varies, so they stay
 * trivial and separate instead of being folded into one clever dispatcher. */

static void Wow64pPutHandle(ULONG Cell32, HANDLE Value)
{
    ULONG *Cell = (ULONG *)(ULONG_PTR)Cell32;
    if (Cell)
        *Cell = (ULONG)(ULONG_PTR)Value;
}

/* OBJECT_ATTRIBUTES is _In_opt_ on several of the creates below (NtCreateTimer,
 * NtDuplicateToken, NtCreateProcessEx...). Marshal it only when the guest
 * actually passed one and give the host a real NULL otherwise — a pointer to a
 * zeroed OA is not the same thing to the object manager as no OA at all. */
static PVOID Wow64pMarshalOaOpt(WOW64_OBJECT_ATTRIBUTES64_M *Oa64,
                                WOW64_UNICODE_STRING64_M *Uname64,
                                ULONG Oa32Ptr)
{
    if (!Oa32Ptr)
        return NULL;
    Wow64pMarshalOa(Oa64, Uname64, Oa32Ptr);
    return Oa64;
}

/* --- out-cell only, no OBJECT_ATTRIBUTES --------------------------------- */

static NTSTATUS Wow64Manual_NtOpenProcessToken(ULONG *a)
{
    HANDLE Token = NULL;
    NTSTATUS St = NtOpenProcessToken((HANDLE)(LONG_PTR)(LONG)a[0], a[1], &Token);
    if (St >= 0)
        Wow64pPutHandle(a[2], Token);
    return St;
}

static NTSTATUS Wow64Manual_NtOpenThreadToken(ULONG *a)
{
    HANDLE Token = NULL;
    NTSTATUS St = NtOpenThreadToken((HANDLE)(LONG_PTR)(LONG)a[0], a[1],
                                    (BOOLEAN)a[2], &Token);
    if (St >= 0)
        Wow64pPutHandle(a[3], Token);
    return St;
}

static NTSTATUS Wow64Manual_NtDuplicateObject(ULONG *a)
{
    HANDLE Target = NULL;
    /* TargetHandle is _Out_opt_ — a DUPLICATE_CLOSE_SOURCE-only call passes
     * NULL, and that NULL has to reach the kernel as NULL. */
    NTSTATUS St = NtDuplicateObject((HANDLE)(LONG_PTR)(LONG)a[0],
                                    (HANDLE)(LONG_PTR)(LONG)a[1],
                                    (HANDLE)(LONG_PTR)(LONG)a[2],
                                    a[3] ? &Target : NULL,
                                    a[4], a[5], a[6]);
    if (St >= 0)
        Wow64pPutHandle(a[3], Target);
    return St;
}

/* --- out-cell + OBJECT_ATTRIBUTES ---------------------------------------- */

/* (PHANDLE, ACCESS_MASK, POBJECT_ATTRIBUTES, ULONG) — NtCreateTimer's
 * TIMER_TYPE and NtCreateDebugObject's Flags are both plain 32-bit enums. */
typedef NTSTATUS (WINAPI *WOW64_OPEN_FN_U32)(PHANDLE, ACCESS_MASK, PVOID, ULONG);

static NTSTATUS Wow64pOpenByOaU32(WOW64_OPEN_FN_U32 Fn, ULONG *a)
{
    WOW64_OBJECT_ATTRIBUTES64_M Oa64;
    WOW64_UNICODE_STRING64_M Uname64;
    HANDLE Handle = NULL;
    NTSTATUS St = Fn(&Handle, a[1],
                     Wow64pMarshalOaOpt(&Oa64, &Uname64, a[2]), a[3]);
    if (St >= 0)
        Wow64pPutHandle(a[0], Handle);
    return St;
}

/* (PHANDLE, POBJECT_ATTRIBUTES, ULONG, ULONG, ULONG) — the LPC port creates.
 * Note the OA is at slot 1 here, not 2: there is no ACCESS_MASK. */
typedef NTSTATUS (WINAPI *WOW64_PORT_FN)(PHANDLE, PVOID, ULONG, ULONG, ULONG);

static NTSTATUS Wow64pCreatePort(WOW64_PORT_FN Fn, ULONG *a)
{
    WOW64_OBJECT_ATTRIBUTES64_M Oa64;
    WOW64_UNICODE_STRING64_M Uname64;
    HANDLE Handle = NULL;
    NTSTATUS St = Fn(&Handle, Wow64pMarshalOaOpt(&Oa64, &Uname64, a[1]),
                     a[2], a[3], a[4]);
    if (St >= 0)
        Wow64pPutHandle(a[0], Handle);
    return St;
}

static NTSTATUS Wow64Manual_NtDuplicateToken(ULONG *a)
{
    WOW64_OBJECT_ATTRIBUTES64_M Oa64;
    WOW64_UNICODE_STRING64_M Uname64;
    HANDLE NewToken = NULL;
    NTSTATUS St = NtDuplicateToken((HANDLE)(LONG_PTR)(LONG)a[0], a[1],
                                   Wow64pMarshalOaOpt(&Oa64, &Uname64, a[2]),
                                   (BOOLEAN)a[3], a[4], &NewToken);
    if (St >= 0)
        Wow64pPutHandle(a[5], NewToken);
    return St;
}

static NTSTATUS Wow64Manual_NtCreateProcess(ULONG *a)
{
    WOW64_OBJECT_ATTRIBUTES64_M Oa64;
    WOW64_UNICODE_STRING64_M Uname64;
    HANDLE Process = NULL;
    NTSTATUS St = NtCreateProcess(&Process, a[1],
                                  Wow64pMarshalOaOpt(&Oa64, &Uname64, a[2]),
                                  (HANDLE)(LONG_PTR)(LONG)a[3], (BOOLEAN)a[4],
                                  (HANDLE)(LONG_PTR)(LONG)a[5],
                                  (HANDLE)(LONG_PTR)(LONG)a[6],
                                  (HANDLE)(LONG_PTR)(LONG)a[7]);
    if (St >= 0)
        Wow64pPutHandle(a[0], Process);
    return St;
}

/* TEB32 is TEB64 + WowTebOffset — MmCreateTeb allocates both as one region and
 * records the delta in that slot. Spelled locally, as the other TEB offsets in
 * this tree are. */
#define WOW64_TEB_WOWTEBOFFSET   0x180c
#define WOW64_TEB32_ARBUSERPTR   0x14   /* NT_TIB32.ArbitraryUserPointer */

/*
 * ---- the CreateProcess remote-PEB channel --------------------------------
 *
 * The SAME SHAPE as the win32k last-error channel in wow64.c, and the same
 * fix. ps/process.c:866 does
 *
 *     CurrentThread->Tcb.Teb->NtTib.ArbitraryUserPointer = Process->Peb;
 *
 * stashing the NEW process's PEB in the CALLER's TEB, and kernel32's
 * BasePushProcessParameters reads it straight back out to learn where in the
 * child to write the ProcessParameters pointer. Two things go wrong for a
 * 32-bit caller, and they compound: `KTHREAD.Teb` is the SIXTY-FOUR-bit TEB,
 * which the guest never reads, and the value stored is the child's 64-bit PEB
 * when a 32-bit parent needs its PEB32.
 *
 * So the guest read a TEB32 slot nothing had written. Measured before this
 * existed: the third NtWriteVirtualMemory of CreateProcess — the one the
 * source calls "Write the PEB Pointer" — landed on that stale value and came
 * back STATUS_PARTIAL_COPY, so CreateProcessW failed with ERROR_PARTIAL_COPY
 * and no 32-bit guest could start a process at all.
 *
 * ProcessWow64Information is the child's PEB32 by construction: procsup.rs
 * sets `EPROCESS.Wow64Process = Peb32` and that query is what hands it back.
 */
static void Wow64pPublishRemotePeb32(HANDLE Process)
{
    PBYTE Teb = (PBYTE)NtCurrentTeb();
    LONG  WowTebOffset = *(LONG *)(Teb + WOW64_TEB_WOWTEBOFFSET);
    PVOID Peb32 = NULL;

    if (!WowTebOffset)
        return;  /* no 32-bit side — nothing to publish to */
    if (NtQueryInformationProcess(Process, ProcessWow64Information,
                                  &Peb32, sizeof(Peb32), NULL) < 0)
        return;  /* not a WoW64 child; leave the slot alone */

    *(ULONG *)(Teb + WowTebOffset + WOW64_TEB32_ARBUSERPTR) =
        (ULONG)(ULONG_PTR)Peb32;
}

static NTSTATUS Wow64Manual_NtCreateProcessEx(ULONG *a)
{
    WOW64_OBJECT_ATTRIBUTES64_M Oa64;
    WOW64_UNICODE_STRING64_M Uname64;
    HANDLE Process = NULL;
    NTSTATUS St = NtCreateProcessEx(&Process, a[1],
                                    Wow64pMarshalOaOpt(&Oa64, &Uname64, a[2]),
                                    (HANDLE)(LONG_PTR)(LONG)a[3], a[4],
                                    (HANDLE)(LONG_PTR)(LONG)a[5],
                                    (HANDLE)(LONG_PTR)(LONG)a[6],
                                    (HANDLE)(LONG_PTR)(LONG)a[7],
                                    (BOOLEAN)a[8]);
    if (St >= 0)
    {
        Wow64pPutHandle(a[0], Process);
        Wow64pPublishRemotePeb32(Process);
    }
    return St;
}

static NTSTATUS Wow64Manual_NtCreateProfile(ULONG *a)
{
    HANDLE Profile = NULL;
    NTSTATUS St = NtCreateProfile(&Profile, (HANDLE)(LONG_PTR)(LONG)a[1],
                                  (PVOID)(ULONG_PTR)a[2], (SIZE_T)a[3], a[4],
                                  (PVOID)(ULONG_PTR)a[5], a[6], a[7],
                                  (ULONG_PTR)a[8]);
    if (St >= 0)
        Wow64pPutHandle(a[0], Profile);
    return St;
}

/* --- in/out BaseAddress + RegionSize (the Allocate/Free shape) ------------ */

typedef NTSTATUS (WINAPI *WOW64_VMLOCK_FN)(HANDLE, PVOID *, PSIZE_T, ULONG);

static NTSTATUS Wow64pLockUnlockVm(WOW64_VMLOCK_FN Fn, ULONG *a)
{
    ULONG *Base32 = (ULONG *)(ULONG_PTR)a[1];
    ULONG *Size32 = (ULONG *)(ULONG_PTR)a[2];
    PVOID Base = Base32 ? (PVOID)(ULONG_PTR)*Base32 : NULL;
    SIZE_T Size = Size32 ? *Size32 : 0;
    NTSTATUS St = Fn((HANDLE)(LONG_PTR)(LONG)a[0], &Base, &Size, a[3]);

    /* Both cells are IN/OUT: the kernel rounds the request out to page
     * boundaries and hands the rounded pair back. Mirror on success only,
     * matching the Allocate/Free marshallers above. */
    if (St >= 0)
    {
        if (Base32) *Base32 = (ULONG)(ULONG_PTR)Base;
        if (Size32) *Size32 = (ULONG)Size;
    }
    return St;
}

/* NtFlushVirtualMemory also takes an IO_STATUS_BLOCK, so it lives just after
 * that helper below rather than here with the rest of its family. */

/* ---- bare PUNICODE_STRING arguments ------------------------------------- *
 *
 * UNICODE_STRING is 8 bytes in the guest — {USHORT Length; USHORT MaximumLength;
 * ULONG Buffer} — and 16 on the host, because the pointer is 8-aligned:
 * {USHORT Length; USHORT MaximumLength; ULONG pad; ULONGLONG Buffer}. So Buffer
 * sits at offset 4 in the guest and offset 8 on the host.
 *
 * A flat thunk therefore makes the kernel read the string POINTER from offset 8
 * of an 8-byte object — i.e. from PAST THE END of the guest's struct. This is
 * not a truncation, it is a wholly unrelated address: whatever the guest
 * happened to put next to the UNICODE_STRING becomes the buffer pointer, while
 * Length and MaximumLength (same offsets on both sides) come through intact and
 * make it look plausible.
 *
 * The one that matters most is NtSetValueKey — every `RegSetValueEx` from a
 * 32-bit guest. NtQueryValueKey was already marshalled, so registry READS
 * worked and WRITES did not, which is the same "fix the variant you happen to
 * be looking at" pattern that left plain NtOpenProcessToken behind its Ex
 * sibling. Sourced from `marshalgen audit`, like the out-cell class above. */

static PVOID Wow64pMarshalUstr(WOW64_UNICODE_STRING64_M *U64, ULONG U32Ptr)
{
    WOW64_UNICODE_STRING32_M *U32 = (WOW64_UNICODE_STRING32_M *)(ULONG_PTR)U32Ptr;
    if (!U32)
        return NULL;   /* several of these are _In_opt_ */
    U64->Length = U32->Length;
    U64->MaximumLength = U32->MaximumLength;
    U64->__pad = 0;
    U64->Buffer = (ULONG_PTR)U32->Buffer;
    return U64;
}

/* --- registry value names (the hot one) ---------------------------------- */

static NTSTATUS Wow64Manual_NtSetValueKey(ULONG *a)
{
    WOW64_UNICODE_STRING64_M Name;
    return NtSetValueKey((HANDLE)(LONG_PTR)(LONG)a[0],
                         Wow64pMarshalUstr(&Name, a[1]), a[2], a[3],
                         (PVOID)(ULONG_PTR)a[4], a[5]);
}

static NTSTATUS Wow64Manual_NtDeleteValueKey(ULONG *a)
{
    WOW64_UNICODE_STRING64_M Name;
    return NtDeleteValueKey((HANDLE)(LONG_PTR)(LONG)a[0],
                            Wow64pMarshalUstr(&Name, a[1]));
}

static NTSTATUS Wow64Manual_NtRenameKey(ULONG *a)
{
    WOW64_UNICODE_STRING64_M Name;
    return NtRenameKey((HANDLE)(LONG_PTR)(LONG)a[0],
                       Wow64pMarshalUstr(&Name, a[1]));
}

/* --- single PUNICODE_STRING, nothing else -------------------------------- */

typedef NTSTATUS (WINAPI *WOW64_USTR1_FN)(PVOID);

static NTSTATUS Wow64pCallUstr1(WOW64_USTR1_FN Fn, ULONG *a)
{
    WOW64_UNICODE_STRING64_M S;
    return Fn(Wow64pMarshalUstr(&S, a[0]));
}

/* --- single POBJECT_ATTRIBUTES, nothing else ----------------------------- */

typedef NTSTATUS (WINAPI *WOW64_OA1_FN)(PVOID);

static NTSTATUS Wow64pCallOa1(WOW64_OA1_FN Fn, ULONG *a)
{
    WOW64_OBJECT_ATTRIBUTES64_M Oa;
    WOW64_UNICODE_STRING64_M Name;
    return Fn(Wow64pMarshalOaOpt(&Oa, &Name, a[0]));
}

/* --- environment values -------------------------------------------------- */

static NTSTATUS Wow64Manual_NtQuerySystemEnvironmentValue(ULONG *a)
{
    WOW64_UNICODE_STRING64_M Name;
    return NtQuerySystemEnvironmentValue(Wow64pMarshalUstr(&Name, a[0]),
                                         (PVOID)(ULONG_PTR)a[1], a[2],
                                         (PULONG)(ULONG_PTR)a[3]);
}

static NTSTATUS Wow64Manual_NtSetSystemEnvironmentValue(ULONG *a)
{
    WOW64_UNICODE_STRING64_M Name, Value;
    return NtSetSystemEnvironmentValue(Wow64pMarshalUstr(&Name, a[0]),
                                       Wow64pMarshalUstr(&Value, a[1]));
}

static NTSTATUS Wow64Manual_NtQuerySystemEnvironmentValueEx(ULONG *a)
{
    WOW64_UNICODE_STRING64_M Name;
    return NtQuerySystemEnvironmentValueEx(Wow64pMarshalUstr(&Name, a[0]),
                                           (PVOID)(ULONG_PTR)a[1],
                                           (PVOID)(ULONG_PTR)a[2],
                                           (PULONG)(ULONG_PTR)a[3],
                                           (PULONG)(ULONG_PTR)a[4]);
}

static NTSTATUS Wow64Manual_NtSetSystemEnvironmentValueEx(ULONG *a)
{
    WOW64_UNICODE_STRING64_M Name;
    return NtSetSystemEnvironmentValueEx(Wow64pMarshalUstr(&Name, a[0]),
                                         (PVOID)(ULONG_PTR)a[1],
                                         (PVOID)(ULONG_PTR)a[2], a[3], a[4]);
}

static NTSTATUS Wow64Manual_NtCreatePagingFile(ULONG *a)
{
    WOW64_UNICODE_STRING64_M Name;
    /* MinimumSize/MaximumSize are PLARGE_INTEGER — 8 bytes on both sides. */
    return NtCreatePagingFile(Wow64pMarshalUstr(&Name, a[0]),
                              (PVOID)(ULONG_PTR)a[1], (PVOID)(ULONG_PTR)a[2],
                              a[3]);
}

/* --- hive load/unload/replace (OBJECT_ATTRIBUTES pairs) ------------------ */

static NTSTATUS Wow64Manual_NtLoadKey(ULONG *a)
{
    WOW64_OBJECT_ATTRIBUTES64_M KeyOa, FileOa;
    WOW64_UNICODE_STRING64_M KeyName, FileName;
    return NtLoadKey(Wow64pMarshalOaOpt(&KeyOa, &KeyName, a[0]),
                     Wow64pMarshalOaOpt(&FileOa, &FileName, a[1]));
}

static NTSTATUS Wow64Manual_NtLoadKey2(ULONG *a)
{
    WOW64_OBJECT_ATTRIBUTES64_M KeyOa, FileOa;
    WOW64_UNICODE_STRING64_M KeyName, FileName;
    return NtLoadKey2(Wow64pMarshalOaOpt(&KeyOa, &KeyName, a[0]),
                      Wow64pMarshalOaOpt(&FileOa, &FileName, a[1]), a[2]);
}

static NTSTATUS Wow64Manual_NtLoadKeyEx(ULONG *a)
{
    WOW64_OBJECT_ATTRIBUTES64_M KeyOa, FileOa;
    WOW64_UNICODE_STRING64_M KeyName, FileName;
    return NtLoadKeyEx(Wow64pMarshalOaOpt(&KeyOa, &KeyName, a[0]),
                       Wow64pMarshalOaOpt(&FileOa, &FileName, a[1]), a[2],
                       (HANDLE)(LONG_PTR)(LONG)a[3]);
}

static NTSTATUS Wow64Manual_NtReplaceKey(ULONG *a)
{
    WOW64_OBJECT_ATTRIBUTES64_M NewOa, OldOa;
    WOW64_UNICODE_STRING64_M NewName, OldName;
    return NtReplaceKey(Wow64pMarshalOaOpt(&NewOa, &NewName, a[0]),
                        (HANDLE)(LONG_PTR)(LONG)a[1],
                        Wow64pMarshalOaOpt(&OldOa, &OldName, a[2]));
}

static NTSTATUS Wow64Manual_NtUnloadKey2(ULONG *a)
{
    WOW64_OBJECT_ATTRIBUTES64_M Oa;
    WOW64_UNICODE_STRING64_M Name;
    return NtUnloadKey2(Wow64pMarshalOaOpt(&Oa, &Name, a[0]), a[1]);
}

static NTSTATUS Wow64Manual_NtUnloadKeyEx(ULONG *a)
{
    WOW64_OBJECT_ATTRIBUTES64_M Oa;
    WOW64_UNICODE_STRING64_M Name;
    return NtUnloadKeyEx(Wow64pMarshalOaOpt(&Oa, &Name, a[0]),
                         (HANDLE)(LONG_PTR)(LONG)a[1]);
}

static NTSTATUS Wow64Manual_NtQueryOpenSubKeys(ULONG *a)
{
    WOW64_OBJECT_ATTRIBUTES64_M Oa;
    WOW64_UNICODE_STRING64_M Name;
    return NtQueryOpenSubKeys(Wow64pMarshalOaOpt(&Oa, &Name, a[0]),
                              (PULONG)(ULONG_PTR)a[1]);
}

static NTSTATUS Wow64Manual_NtQueryOpenSubKeysEx(ULONG *a)
{
    WOW64_OBJECT_ATTRIBUTES64_M Oa;
    WOW64_UNICODE_STRING64_M Name;
    return NtQueryOpenSubKeysEx(Wow64pMarshalOaOpt(&Oa, &Name, a[0]), a[1],
                                (PVOID)(ULONG_PTR)a[2],
                                (PULONG)(ULONG_PTR)a[3]);
}

/* --- the audit-alarm family ---------------------------------------------- *
 * Rarely called (advapi32's ObjectPrivilege APIs), but they take two or three
 * UNICODE_STRINGs each and cost one line apiece here. PRIVILEGE_SET and
 * GENERIC_MAPPING are pointer-free (LUIDs and ACCESS_MASKs), so they pass
 * through — which is what the audit says, and why they are not in the list. */

static NTSTATUS Wow64Manual_NtAccessCheckAndAuditAlarm(ULONG *a)
{
    WOW64_UNICODE_STRING64_M Sub, TypeName, ObjName;
    return NtAccessCheckAndAuditAlarm(Wow64pMarshalUstr(&Sub, a[0]),
                                      (PVOID)(ULONG_PTR)a[1],
                                      Wow64pMarshalUstr(&TypeName, a[2]),
                                      Wow64pMarshalUstr(&ObjName, a[3]),
                                      (PVOID)(ULONG_PTR)a[4], a[5],
                                      (PVOID)(ULONG_PTR)a[6], (BOOLEAN)a[7],
                                      (PVOID)(ULONG_PTR)a[8],
                                      (PVOID)(ULONG_PTR)a[9],
                                      (PVOID)(ULONG_PTR)a[10]);
}

static NTSTATUS Wow64Manual_NtOpenObjectAuditAlarm(ULONG *a)
{
    WOW64_UNICODE_STRING64_M Sub, TypeName, ObjName;
    return NtOpenObjectAuditAlarm(Wow64pMarshalUstr(&Sub, a[0]),
                                  (PVOID)(ULONG_PTR)a[1],
                                  Wow64pMarshalUstr(&TypeName, a[2]),
                                  Wow64pMarshalUstr(&ObjName, a[3]),
                                  (PVOID)(ULONG_PTR)a[4],
                                  (HANDLE)(LONG_PTR)(LONG)a[5], a[6], a[7],
                                  (PVOID)(ULONG_PTR)a[8], (BOOLEAN)a[9],
                                  (BOOLEAN)a[10], (PVOID)(ULONG_PTR)a[11]);
}

typedef NTSTATUS (WINAPI *WOW64_AUDIT_CLOSE_FN)(PVOID, PVOID, BOOLEAN);

static NTSTATUS Wow64pAuditClose(WOW64_AUDIT_CLOSE_FN Fn, ULONG *a)
{
    WOW64_UNICODE_STRING64_M Sub;
    return Fn(Wow64pMarshalUstr(&Sub, a[0]), (PVOID)(ULONG_PTR)a[1],
              (BOOLEAN)a[2]);
}

static NTSTATUS Wow64Manual_NtPrivilegeObjectAuditAlarm(ULONG *a)
{
    WOW64_UNICODE_STRING64_M Sub;
    return NtPrivilegeObjectAuditAlarm(Wow64pMarshalUstr(&Sub, a[0]),
                                       (PVOID)(ULONG_PTR)a[1],
                                       (HANDLE)(LONG_PTR)(LONG)a[2], a[3],
                                       (PVOID)(ULONG_PTR)a[4], (BOOLEAN)a[5]);
}

static NTSTATUS Wow64Manual_NtPrivilegedServiceAuditAlarm(ULONG *a)
{
    WOW64_UNICODE_STRING64_M Sub, Svc;
    return NtPrivilegedServiceAuditAlarm(Wow64pMarshalUstr(&Sub, a[0]),
                                         Wow64pMarshalUstr(&Svc, a[1]),
                                         (HANDLE)(LONG_PTR)(LONG)a[2],
                                         (PVOID)(ULONG_PTR)a[3],
                                         (BOOLEAN)a[4]);
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

static BOOLEAN Wow64pRedirectPath(WOW64_UNICODE_STRING64_M *Name64, WCHAR *Scratch)
{
    static const WCHAR Sys32[] = {'\\','s','y','s','t','e','m','3','2','\\',0};
    static const WCHAR Wow64[] = {'\\','s','y','s','w','o','w','6','4','\\',0};
    const WCHAR *Src = (const WCHAR *)(ULONG_PTR)Name64->Buffer;
    ULONG Chars = Name64->Length / sizeof(WCHAR);
    ULONG i, j;
    LONG Hit = -1;

    if (!Src || Chars == 0)
        return FALSE;
    if (Chars > WOW64_REDIR_MAX)
    {
        /* A path too long for the scratch buffer used to return quietly, which
         * means a \system32\ path silently stayed \system32\ and the 32-bit
         * process opened the 64-BIT file. That is a wrong-file open reported as
         * success — exactly the failure this function exists to prevent — so it
         * says so rather than degrading in silence. */
        DbgPrint("[WOW64]   REDIRECT SKIPPED (path %lu chars > %u): '%.*S'\n",
                 Chars, (unsigned)WOW64_REDIR_MAX, (int)Chars, Src);
        return FALSE;
    }

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
        return FALSE;

    for (i = 0; i < Chars; i++)
        Scratch[i] = Src[i];
    for (j = 0; j < 10; j++)
        Scratch[Hit + j] = Wow64[j];
    Name64->Buffer = (ULONG_PTR)Scratch;
    return TRUE;
}

static void Wow64pCopyIosbDown(WOW64_IO_STATUS_BLOCK64_M *I64, ULONG Iosb32Ptr)
{
    WOW64_IO_STATUS_BLOCK32_M *I32 = (WOW64_IO_STATUS_BLOCK32_M *)(ULONG_PTR)Iosb32Ptr;
    if (!I32) return;
    I32->Status = (ULONG)I64->Status;
    I32->Information = (ULONG)I64->Information;
}

/* The third member of the Lock/Unlock in/out family (see the out-cell section
 * above); it is here because it needs the IOSB helper. */
static NTSTATUS Wow64Manual_NtFlushVirtualMemory(ULONG *a)
{
    ULONG *Base32 = (ULONG *)(ULONG_PTR)a[1];
    ULONG *Size32 = (ULONG *)(ULONG_PTR)a[2];
    PVOID Base = Base32 ? (PVOID)(ULONG_PTR)*Base32 : NULL;
    SIZE_T Size = Size32 ? *Size32 : 0;
    WOW64_IO_STATUS_BLOCK64_M Iosb;
    NTSTATUS St;

    Iosb.Status = 0;
    Iosb.Information = 0;
    /* Synchronous by contract, so a host IOSB on this frame is safe — unlike
     * the file family, this one can never come back STATUS_PENDING holding a
     * pointer into a frame that is about to die (field manual §9.36). */
    St = NtFlushVirtualMemory((HANDLE)(LONG_PTR)(LONG)a[0], &Base, &Size, &Iosb);
    Wow64pCopyIosbDown(&Iosb, a[3]);
    if (St >= 0)
    {
        if (Base32) *Base32 = (ULONG)(ULONG_PTR)Base;
        if (Size32) *Size32 = (ULONG)Size;
    }
    return St;
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
 * The real fix for async is kernel-side: I/O completion writes an
 * IO_STATUS_BLOCK32 when the requesting process is WoW64 (what NT's
 * IoIs32bitProcess() gates). That is DONE for NtReadFile/NtWriteFile (M15b)
 * and for the IOCTL family (M15c) — see Wow64pLatchIosb32. The functions still
 * using Wow64pFinishIosb below are the ones not yet converted; they keep the
 * refusal, which is why it is still here.
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

/* NtReadFile / NtWriteFile share their shape exactly.
 *
 * M15b — kernel-side IOSB32: these two no longer marshal through a host IOSB.
 * We latch the guest's IOSB address into TEB.Instrumentation[0]; the kernel's
 * NtReadFile/NtWriteFile entry consumes the latch (read-and-clear, honored
 * only when it matches the IoStatusBlock argument exactly), marks the IRP,
 * and I/O completion writes the 8-byte IO_STATUS_BLOCK32 layout — sync AND
 * async. That kills the dead-frame hazard outright (the kernel holds a
 * pointer into the GUEST's IOSB, which outlives this frame by construction),
 * so Event-based/polled overlapped I/O from guests is now allowed through.
 * A guest APC routine still can't be honored — that stays a loud refusal, but
 * NOT because guest entry is missing: Wow64pBuildLoaderApcFrame already builds
 * a KiUserApcDispatcher-ABI frame and runs on every guest thread start. What is
 * missing is interception. The kernel's KiInitializeUserApc builds a 64-bit
 * UAPC_FRAME aimed at the 64-bit dispatcher, so a guest's 32-bit routine passed
 * through would be called as native x64; wow64.dll has to substitute its own
 * host thunk as the NormalRoutine and carry the guest routine/context along,
 * and the frame builder has to take its routine as a parameter instead of
 * hardcoding the loader thunk. */
typedef NTSTATUS (WINAPI *WOW64_RW_FN)(HANDLE, HANDLE, PVOID, PVOID, PVOID,
                                       PVOID, ULONG, PLARGE_INTEGER, PULONG);

/* TEB.Instrumentation offset. Identical on both 64-bit hosts and pinned by a
 * layout assert on each (offset_check_amd64.rs and offset_check_arm64.rs:
 * _TEB.Instrumentation == 5816), so this is one constant rather than a
 * per-arch table. Drift degrades to "latch never matches" + the kernel's loud
 * [IOSB32] mismatch print + the async32.exe oracle failing its OVERLAPPED
 * clobber check — never to silent corruption. */
#define WOW64_TEB_INSTRUMENTATION0 5816

static void Wow64pLatchIosb32(ULONG Iosb32Ptr)
{
    *(ULONG_PTR *)((unsigned char *)NtCurrentTeb() + WOW64_TEB_INSTRUMENTATION0) =
        (ULONG_PTR)Iosb32Ptr;
}

static NTSTATUS Wow64pReadWriteFile(WOW64_RW_FN Fn, ULONG *a, const char *Who)
{
    HANDLE File = (HANDLE)(LONG_PTR)(LONG)a[0];
    HANDLE Event = (HANDLE)(LONG_PTR)(LONG)a[1];
    NTSTATUS St;

    if (a[2])
    {
        DbgPrint("[WOW64]   %s: guest ApcRoutine=%08lx is not supported yet "
                 "(needs a host thunk standing in as the NormalRoutine) — "
                 "refusing\n", Who, a[2]);
        return WOW64_STATUS_NOT_IMPLEMENTED;
    }

    /* ByteOffset is a LARGE_INTEGER* and Key a ULONG* — both layout-identical. */
    Wow64pLatchIosb32(a[4]);
    St = Fn(File, Event, NULL, (PVOID)(ULONG_PTR)a[3],
            (PVOID)(ULONG_PTR)a[4],
            (PVOID)(ULONG_PTR)a[5], a[6],
            (PLARGE_INTEGER)(ULONG_PTR)a[7], (PULONG)(ULONG_PTR)a[8]);
    /* The kernel consumed the latch at entry; clear defensively for paths
     * that fail before reaching it (bad handle, etc.). */
    Wow64pLatchIosb32(0);
    return St;
}

static NTSTATUS Wow64Manual_NtReadFile(ULONG *a)
{
    /* The kernel writes into the guest's buffer behind the CPU layer's back. If
     * that buffer is inside a range the layer write-traps for self-modifying
     * code (a loader reading a section straight into RWX memory does exactly
     * this), the trap has to come off for the duration — and the code cached
     * from those pages has to go. Buffer=a[5], Length=a[6]. */
    PVOID Buffer = (PVOID)(ULONG_PTR)a[5];
    SIZE_T Length = a[6];
    HANDLE File = (HANDLE)(LONG_PTR)(LONG)a[0];
    NTSTATUS St;

    Wow64NotifyReadFileBefore(File, Buffer, Length);
    St = Wow64pReadWriteFile(NtReadFile, a, "NtReadFile");
    Wow64NotifyReadFileAfter(File, Buffer, Length, St);
    return St;
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
    BOOLEAN Redirected;

    Wow64pMarshalOa(&Oa64, &Uname64, a[2]);
    Redirected = Wow64pRedirectPath(&Uname64, RedirScratch);
    St = NtOpenFile(&Handle, Access, &Oa64, &Iosb64, a[4], a[5]);
    /* Logged on SUCCESS too. Printing only failures means you can see every
     * path the process could not open and none of the ones it did — so a
     * successful open of the WRONG file (the 64-bit copy, because redirection
     * did not fire) is invisible, which is precisely the case this had to be
     * used to diagnose. */
    if (Oa64.ObjectName)
        DbgPrint("[WOW64]   NtOpenFile%s '%.*S' => %08lx\n",
                 Redirected ? " [redir]" : "",
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
    BOOLEAN Self;

    /* Keep guest allocations addressable from 32-bit: when the guest lets the
     * system choose (Base==0), force the result below 2GB (kernel ZeroBits>32
     * mask semantics). */
    if (!Base && ZeroBits == 0)
        ZeroBits = WOW64_ZEROBITS_BELOW_2G;

    /* Announce the new range to the CPU layer. Without this a guest that
     * generates code into the allocation cannot execute it at all on a JIT
     * host: FEX only decodes addresses inside the executable set it has been
     * told about. Before/After must stay paired around this one call. */
    Self = Wow64IsCurrentProcess(Process);
    if (Self) Wow64NotifyAllocBefore(Base, Size, Type, Protect);
    St = NtAllocateVirtualMemory(Process, &Base, ZeroBits, &Size, Type, Protect);
    if (Self) Wow64NotifyAllocAfter(Base, Size, Type, Protect, St);
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
    NTSTATUS St;
    BOOLEAN Self;

    /* A freed range must leave the CPU layer's executable set, or a later
     * allocation reusing the address executes the OLD translation. */
    Self = Wow64IsCurrentProcess(Process);
    if (Self) Wow64NotifyFreeBefore(Base, Size, FreeType);
    St = NtFreeVirtualMemory(Process, &Base, &Size, FreeType);
    if (Self) Wow64NotifyFreeAfter(Base, Size, FreeType, St);
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
    NTSTATUS St;
    BOOLEAN Self;

    /* The RW -> RWX transition every runtime code generator makes. This is what
     * puts the range in the CPU layer's executable set and arms its
     * self-modifying-code tracking. */
    Self = Wow64IsCurrentProcess(Process);
    if (Self) Wow64NotifyProtectBefore(Base, Size, NewProtect);
    St = NtProtectVirtualMemory(Process, &Base, &Size, NewProtect, &Old);
    if (Self) Wow64NotifyProtectAfter(Base, Size, NewProtect, St);
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

/* ---- NtUnmapViewOfSection ------------------------------------------------ *
 * ABI-flat, but the CPU layer has to drop the image and everything it cached
 * from it — a later map at the same address would otherwise run the old code. */
static NTSTATUS Wow64Manual_NtUnmapViewOfSection(ULONG *a)
{
    HANDLE Process = (HANDLE)(LONG_PTR)(LONG)a[0];
    PVOID Base = (PVOID)(ULONG_PTR)a[1];
    BOOLEAN Self = Wow64IsCurrentProcess(Process);
    NTSTATUS St;

    if (Self) Wow64NotifyUnmapBefore(Base);
    St = NtUnmapViewOfSection(Process, Base);
    if (Self) Wow64NotifyUnmapAfter(Base, St);
    return St;
}

/* ---- NtRead/NtWriteVirtualMemory ----------------------------------------- *
 * Two separate defects the flat thunk had: the trailing *ReturnSize is a
 * ULONG_PTR, so the host wrote 8 bytes into the guest's 4-byte cell and
 * clobbered whatever followed it; and a write into the guest's own address
 * space can land on code the CPU layer has already translated. */
static NTSTATUS Wow64Manual_NtReadVirtualMemory(ULONG *a)
{
    HANDLE Process = (HANDLE)(LONG_PTR)(LONG)a[0];
    ULONG *Read32 = (ULONG *)(ULONG_PTR)a[4];
    SIZE_T Read = 0;
    NTSTATUS St = NtReadVirtualMemory(Process, (PVOID)(ULONG_PTR)a[1],
                                      (PVOID)(ULONG_PTR)a[2], a[3], &Read);
    if (Read32) *Read32 = (ULONG)Read;
    return St;
}

static NTSTATUS Wow64Manual_NtWriteVirtualMemory(ULONG *a)
{
    HANDLE Process = (HANDLE)(LONG_PTR)(LONG)a[0];
    PVOID Base = (PVOID)(ULONG_PTR)a[1];
    SIZE_T Size = a[3];
    ULONG *Written32 = (ULONG *)(ULONG_PTR)a[4];
    SIZE_T Written = 0;
    NTSTATUS St = NtWriteVirtualMemory(Process, Base, (PVOID)(ULONG_PTR)a[2],
                                       Size, &Written);
    if (Written32) *Written32 = (ULONG)Written;
    /* After the fact on purpose: the layer only needs to know the bytes
     * changed, and Written is the range that actually did. */
    if (St >= 0 && Wow64IsCurrentProcess(Process))
        Wow64NotifyMemoryDirty(Base, Written);
    return St;
}

/* ---- NtFlushInstructionCache --------------------------------------------- *
 * On a native host this is close to a no-op; on a JIT host it is the guest
 * telling us, in the only way the architecture offers, that it just rewrote
 * code. Well-behaved runtime code generators call it and nothing else. */
static NTSTATUS Wow64Manual_NtFlushInstructionCache(ULONG *a)
{
    HANDLE Process = (HANDLE)(LONG_PTR)(LONG)a[0];
    PVOID Base = (PVOID)(ULONG_PTR)a[1];
    SIZE_T Length = a[2];

    if (Wow64IsCurrentProcess(Process))
        Wow64NotifyFlushInstructionCache(Base, Length);
    return NtFlushInstructionCache(Process, Base, Length);
}

/* ---- NtSetInformationProcess(ProcessExecuteFlags) ------------------------ *
 * DEP has two enforcers here and both have to hear about it: the kernel owns
 * the host page protections — which on a native compat-mode host ARE the
 * guest's — and the CPU layer owns whether the emulator will execute a page
 * the host would. The kernel's verdict is the one the guest gets back; the
 * layer is told only once the kernel has agreed. */
static NTSTATUS Wow64Manual_NtSetInformationProcess(ULONG *a)
{
    HANDLE Process = (HANDLE)(LONG_PTR)(LONG)a[0];
    ULONG Class = a[1];
    PVOID Buf32 = (PVOID)(ULONG_PTR)a[2];
    ULONG Len32 = a[3];
    NTSTATUS St;

    if (Class != WOW64_ProcessExecuteFlags)
        return NtSetInformationProcess(Process, Class, Buf32, Len32);

    if (Len32 != sizeof(ULONG) || !Buf32)
        return (NTSTATUS)0xC0000004L; /* STATUS_INFO_LENGTH_MISMATCH */

    St = NtSetInformationProcess(Process, Class, Buf32, Len32);
    if (St >= 0 && Wow64IsCurrentProcess(Process))
        Wow64SetExecuteFlags(*(ULONG *)Buf32);
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

/* The CSR client port view is the ONLY memory csrss and the guest share, and
 * csrss dereferences CsrCaptureData directly in its own address space
 * (CsrValidateMessageBuffer reads CaptureBuffer->Size). A host-layout capture
 * buffer must therefore live inside that view. So we ask the kernel for one
 * page more than the guest wanted and keep the tail as wow64 scratch; the guest
 * is told its original ViewSize and never sees it.
 *
 * Pointers inside a CSR message are SERVER-side: csrlib stores
 * CsrCaptureData = local + (ViewRemoteBase - ViewBase). Dereferencing one
 * locally faults -- that is what killed every 32-bit process at
 * Wow64pCsrWidenCaptureBuffer+0x1b. */
#define WOW64_CSR_SCRATCH_SIZE 0x1000

/* csrss publishes ReadOnlyStaticServerData as an array of PVOID and hands every
 * client its address. Those entries are 8 bytes wide in a 64-bit csrss, but a
 * 32-bit client indexes the same memory with a 4-byte stride -- so kernel32
 * reading [BASESRV_SERVERDLL_INDEX] got the HIGH half of entry 0, which is zero
 * for any pointer below 4GB. It then asserted and dereferenced NULL.
 *
 * Give the guest a narrowed copy instead. csrss fills the array while its
 * server DLLs register, which is complete long before any client connects, so a
 * snapshot taken at connect time is not racing a later write. */
/* Byte accessors, defined with the CSR message marshalling further down. */
static void Wow64pCpy(unsigned char *d, const unsigned char *s, unsigned n);
static ULONG Wow64pCsrGet32(const unsigned char *P, ULONG Off);
static VOID Wow64pCsrPut32(unsigned char *P, ULONG Off, ULONG V);

#define WOW64_CSR_SERVER_DLL_MAX 4   /* subsystems/csr/csrsrv/api.h:34 */
#define WOW64_BASESRV_SERVERDLL_INDEX 1 /* reactos/subsys/win/basemsg.h:15 */
/* CSR_MAKE_API_NUMBER(BASESRV_SERVERDLL_INDEX, BasepCreateThread) */
#define WOW64_CSRAPI_BASEP_CREATEPROCESS 0x00010000
#define WOW64_CSRAPI_BASEP_CREATETHREAD 0x00010001

/* BASE_STATIC_SERVER_DATA is the struct entry[BASESRV_SERVERDLL_INDEX] points
 * at, and it is no more width-safe than the array around it: it opens with
 * three UNICODE_STRINGs, 8 bytes here and 16 there, so a 32-bit reader takes
 * WindowsDirectory.Buffer out of the 64-bit struct's PADDING -- reads NULL, and
 * memmoves MaximumLength bytes from it. That was the last access violation.
 *
 * Offsets MEASURED with a host program on both widths, not derived; the replica
 * it measured reproduces the real headers' own C_ASSERTs (NLS_USER_INFO 0x1870,
 * BASE_STATIC_SERVER_DATA 0x1AC8 on i386) which is what makes the 64-bit
 * numbers trustworthy. sdk/include/reactos/subsys/win/base.h:119. */
#define WOW64_BSSD32_SIZE        0x1AC8
#define WOW64_BSSD_WINDIR32           0
#define WOW64_BSSD_WINDIR64           0
#define WOW64_BSSD_SYSDIR32           8
#define WOW64_BSSD_SYSDIR64          16
#define WOW64_BSSD_OBJDIR32          16
#define WOW64_BSSD_OBJDIR64          32
#define WOW64_BSSD_VERBLK32          24   /* MajorVersion .. CSDVersion[128] */
#define WOW64_BSSD_VERBLK64          48
#define WOW64_BSSD_VERBLK_LEN       268
#define WOW64_BSSD_SYSINFO32        292
#define WOW64_BSSD_SYSINFO64        320
#define WOW64_BSSD_TIMEOFDAY32      336
#define WOW64_BSSD_TIMEOFDAY64      384
#define WOW64_BSSD_TIMEOFDAY_LEN     48
#define WOW64_BSSD_INIMAP32         384
#define WOW64_BSSD_INIMAP64         432
#define WOW64_BSSD_NLS32            388
#define WOW64_BSSD_NLS64            440
#define WOW64_BSSD_NLS_LEN       0x1870
#define WOW64_BSSD_VDMFLAGS32      6644   /* DefaultSeparateVDM, IsWowTaskReady */
#define WOW64_BSSD_VDMFLAGS64      6696
#define WOW64_BSSD_SYS32X86_32     6648
#define WOW64_BSSD_SYS32X86_64     6704
#define WOW64_BSSD_TSTAIL32        6656   /* fTermsrvAppInstallMode .. end */
#define WOW64_BSSD_TSTAIL64        6720
#define WOW64_BSSD_TSTAIL_LEN       200

static unsigned char Wow64pCsrBssd32[WOW64_BSSD32_SIZE];

/* SYSTEM_BASIC_INFORMATION: 7 ULONGs, then 3 ULONG_PTRs, then a CCHAR
 * (ndk/extypes.h:909) -- the three pointer-width fields are the whole reason
 * this struct needs a hand conversion.
 *
 * The 64-bit pointer block starts at 32, NOT at 28: the 7 ULONGs end at 28 and
 * a ULONG_PTR wants 8-byte alignment there. Deriving 28 instead of measuring it
 * shifted every field by four bytes and produced a MinimumUserModeAddress of
 * 0x1000000000000. MEASURED both ways:
 *      i386   Min=28 Max=32 Aff=36 NumProc=40 sizeof=44
 *      amd64  Min=32 Max=40 Aff=48 NumProc=56 sizeof=64 */
#define WOW64_USER_CEILING32 0x7FFEFFFFULL /* i386 MM_HIGHEST_USER_ADDRESS */
#define WOW64_SBI_PTRBLK32 28
#define WOW64_SBI_PTRBLK64 32
#define WOW64_SBI_NUMPROC32 40
#define WOW64_SBI_NUMPROC64 56

static void Wow64pCsrNarrowSysInfo(unsigned char *D32, const unsigned char *S64)
{
    ULONG i;
    for (i = 0; i < 7; i++)
        Wow64pCsrPut32(D32, i * 4, Wow64pCsrGet32(S64, i * 4));
    {
        ULONGLONG Min = *(const ULONGLONG *)(S64 + WOW64_SBI_PTRBLK64 + 0);
        ULONGLONG Max = *(const ULONGLONG *)(S64 + WOW64_SBI_PTRBLK64 + 8);
        ULONGLONG Aff = *(const ULONGLONG *)(S64 + WOW64_SBI_PTRBLK64 + 16);

        /* MaximumUserModeAddress is the host's ceiling (0x7FFFFFFEFFFF on
         * amd64). Truncating it hands the guest 0xFFFEFFFF, which reads as a
         * KERNEL address -- clamp to the 32-bit user ceiling instead. */
        if (Max > WOW64_USER_CEILING32)
            Max = WOW64_USER_CEILING32;
        if (Min >> 32)
        {
            DbgPrint("[WOW64]   BSSD MinimumUserModeAddress %I64x is above 4GB\n", Min);
            Min = 0x10000;
        }
        if (Aff >> 32)
            DbgPrint("[WOW64]   BSSD affinity mask %I64x has processors a 32-bit "
                     "client cannot address\n", Aff);

        Wow64pCsrPut32(D32, WOW64_SBI_PTRBLK32 + 0, (ULONG)Min);
        Wow64pCsrPut32(D32, WOW64_SBI_PTRBLK32 + 4, (ULONG)Max);
        Wow64pCsrPut32(D32, WOW64_SBI_PTRBLK32 + 8, (ULONG)Aff);
    }
    D32[WOW64_SBI_NUMPROC32] = S64[WOW64_SBI_NUMPROC64];
}

static void Wow64pCsrNarrowUniStr(unsigned char *D32, ULONG O32,
                                  const unsigned char *S64, ULONG O64,
                                  const char *Name)
{
    ULONGLONG Buf = *(const ULONGLONG *)(S64 + O64 + 8);
    D32[O32 + 0] = S64[O64 + 0];
    D32[O32 + 1] = S64[O64 + 1];
    D32[O32 + 2] = S64[O64 + 2];
    D32[O32 + 3] = S64[O64 + 3];
    if (Buf >> 32)
    {
        DbgPrint("[WOW64]   BSSD %s buffer %I64x is above 4GB — guest-invisible\n",
                 Name, Buf);
        Wow64pCsrPut32(D32, O32 + 4, 0);
    }
    else
    {
        Wow64pCsrPut32(D32, O32 + 4, (ULONG)Buf);
    }
}

/* Snapshot, because there is no hook to re-narrow on change. csrss fills this
 * during server-DLL registration, before any client connects, so the fields the
 * guest reads at startup are stable; NlsUserInfo.ulCacheUpdateCount is the one
 * field csrss may bump later, and a 32-bit client will not see that bump. */
static ULONG Wow64pCsrNarrowBssd(ULONGLONG Wide)
{
    const unsigned char *S = (const unsigned char *)(ULONG_PTR)Wide;
    unsigned char *D = Wow64pCsrBssd32;
    ULONGLONG Ini;

    if (!Wide)
        return 0;
    if ((ULONGLONG)(ULONG_PTR)D >> 32)
    {
        DbgPrint("[WOW64]   BSSD shadow above 4GB — leaving the 64-bit struct\n");
        return (ULONG)Wide;
    }

    Wow64pCsrNarrowUniStr(D, WOW64_BSSD_WINDIR32, S, WOW64_BSSD_WINDIR64, "WindowsDirectory");
    Wow64pCsrNarrowUniStr(D, WOW64_BSSD_SYSDIR32, S, WOW64_BSSD_SYSDIR64, "WindowsSystemDirectory");
    Wow64pCsrNarrowUniStr(D, WOW64_BSSD_OBJDIR32, S, WOW64_BSSD_OBJDIR64, "NamedObjectDirectory");
    Wow64pCpy(D + WOW64_BSSD_VERBLK32, S + WOW64_BSSD_VERBLK64, WOW64_BSSD_VERBLK_LEN);
    Wow64pCsrNarrowSysInfo(D + WOW64_BSSD_SYSINFO32, S + WOW64_BSSD_SYSINFO64);
    Wow64pCpy(D + WOW64_BSSD_TIMEOFDAY32, S + WOW64_BSSD_TIMEOFDAY64, WOW64_BSSD_TIMEOFDAY_LEN);

    Ini = *(const ULONGLONG *)(S + WOW64_BSSD_INIMAP64);
    Wow64pCsrPut32(D, WOW64_BSSD_INIMAP32, (Ini >> 32) ? 0 : (ULONG)Ini);

    Wow64pCpy(D + WOW64_BSSD_NLS32, S + WOW64_BSSD_NLS64, WOW64_BSSD_NLS_LEN);
    D[WOW64_BSSD_VDMFLAGS32 + 0] = S[WOW64_BSSD_VDMFLAGS64 + 0];
    D[WOW64_BSSD_VDMFLAGS32 + 1] = S[WOW64_BSSD_VDMFLAGS64 + 1];
    Wow64pCsrNarrowUniStr(D, WOW64_BSSD_SYS32X86_32, S, WOW64_BSSD_SYS32X86_64,
                          "WindowsSys32x86Directory");
    Wow64pCpy(D + WOW64_BSSD_TSTAIL32, S + WOW64_BSSD_TSTAIL64, WOW64_BSSD_TSTAIL_LEN);

    return (ULONG)(ULONG_PTR)D;
}
static ULONG Wow64pCsrStaticServerData32[WOW64_CSR_SERVER_DLL_MAX];

static ULONG Wow64pCsrNarrowStaticServerData(ULONGLONG Wide)
{
    const ULONGLONG *Src = (const ULONGLONG *)(ULONG_PTR)Wide;
    ULONG_PTR Self = (ULONG_PTR)&Wow64pCsrStaticServerData32[0];
    ULONG i;

    if (!Wide)
        return 0;

    /* The narrowed array itself has to be reachable from 32-bit code. */
    if ((ULONGLONG)Self >> 32)
    {
        DbgPrint("[WOW64]   CSR static-server-data shadow at %p is above 4GB — "
                 "guest-invisible, leaving the 64-bit array in place\n", (PVOID)Self);
        return (ULONG)Wide;
    }

    for (i = 0; i < WOW64_CSR_SERVER_DLL_MAX; i++)
    {
        ULONGLONG E = Src[i];
        if (i == WOW64_BASESRV_SERVERDLL_INDEX)
        {
            /* The struct behind this entry needs narrowing too. */
            Wow64pCsrStaticServerData32[i] = Wow64pCsrNarrowBssd(E);
        }
        else if (E >> 32)
        {
            DbgPrint("[WOW64]   CSR static server data[%lu]=%I64x is above 4GB — "
                     "guest-invisible\n", i, E);
            Wow64pCsrStaticServerData32[i] = 0;
        }
        else
        {
            Wow64pCsrStaticServerData32[i] = (ULONG)E;
        }
    }
    return (ULONG)Self;
}

static ULONG_PTR Wow64pCsrPortDelta;     /* server address - local address */
static ULONG_PTR Wow64pCsrViewLocal;     /* the guest-visible view */
static ULONG     Wow64pCsrViewSize;
static ULONG_PTR Wow64pCsrScratchLocal;  /* our tail page, local address */
static ULONG     Wow64pCsrScratchSize;

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
    ULONG MaxLen = 0, ConnLen = 0, CvReq = 0;
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
        CvReq = Cv32->ViewSize;
        Cv64.ViewSize = CvReq;
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
    /* The client view is where csrlib builds CsrPortHeap, so every capture
     * buffer the guest ever allocates lives inside it. A base above 4GB is
     * invisible to the guest and silently truncates below. */
    if (Cv32)
        DbgPrint("[WOW64]   ClientView: base=%I64x size=%I64x remote=%I64x\n",
                 Cv64.ViewBase, Cv64.ViewSize, Cv64.ViewRemoteBase);
    if (Sv32)
        DbgPrint("[WOW64]   ServerView: base=%I64x size=%I64x\n",
                 Sv64.ViewBase, Sv64.ViewSize);

    if (St >= 0)
    {
        if (Handle32) *Handle32 = (ULONG)(ULONG_PTR)Handle;
        if (Cv32)
        {
            Cv32->ViewSize = (ULONG)Cv64.ViewSize;
            Cv32->ViewBase = (ULONG)Cv64.ViewBase;
            Cv32->ViewRemoteBase = (ULONG)Cv64.ViewRemoteBase;
        }
        if (Csr32 && Cv32)
        {
            Wow64pCsrPortDelta = (ULONG_PTR)(Cv64.ViewRemoteBase - Cv64.ViewBase);
            Wow64pCsrViewLocal = (ULONG_PTR)Cv64.ViewBase;
            if (Cv64.ViewSize > 2 * WOW64_CSR_SCRATCH_SIZE)
            {
                Wow64pCsrViewSize     = (ULONG)Cv64.ViewSize - WOW64_CSR_SCRATCH_SIZE;
                Wow64pCsrScratchLocal = (ULONG_PTR)Cv64.ViewBase + Wow64pCsrViewSize;
                Wow64pCsrScratchSize  = WOW64_CSR_SCRATCH_SIZE;
                /* The guest's heap must stop short of the tail we keep. */
                Cv32->ViewSize = Wow64pCsrViewSize;
                {
                    /* The port view is a RESERVE section -- its pages commit on
                     * demand, so the tail we just claimed is reserved only and
                     * writing it faults. Commit it before anyone uses it. */
                    PVOID Sc = (PVOID)Wow64pCsrScratchLocal;
                    SIZE_T ScSize = WOW64_CSR_SCRATCH_SIZE;
                    NTSTATUS CSt = NtAllocateVirtualMemory((HANDLE)(LONG_PTR)-1, &Sc, 0,
                                                           &ScSize, 0x1000 /* MEM_COMMIT */,
                                                           0x04 /* PAGE_READWRITE */);
                    if (CSt < 0)
                    {
                        Wow64pCsrScratchLocal = 0;
                        Wow64pCsrScratchSize = 0;
                        Cv32->ViewSize = (ULONG)Cv64.ViewSize;
                        DbgPrint("[WOW64]   CSR scratch commit FAILED %08lx — "
                                 "capture buffers cannot be widened\n", CSt);
                    }
                }
            }
            else
            {
                /* Too small to share. Say so rather than silently handing the
                 * guest a capture buffer csrss cannot read. */
                Wow64pCsrViewSize = (ULONG)Cv64.ViewSize;
                Wow64pCsrScratchLocal = 0;
                Wow64pCsrScratchSize = 0;
                DbgPrint("[WOW64]   CSR port view %I64x too small to reserve scratch — "
                         "capture buffers cannot be widened\n", Cv64.ViewSize);
            }
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
            DbgPrint("[WOW64]   CSR connectinfo: base=%I64x static=%I64x heap=%I64x "
                     "pid=%I64x ndlls=%lx\n",
                     Csr64.SharedSectionBase, Csr64.SharedStaticServerData,
                     Csr64.SharedSectionHeap, Csr64.ServerProcessId,
                     Csr64.NumberOfServerDllNames);
            Csr32->ObjectDirectory = (ULONG)Csr64.ObjectDirectory;
            Csr32->SharedSectionBase = (ULONG)Csr64.SharedSectionBase;
            Csr32->SharedStaticServerData =
                Wow64pCsrNarrowStaticServerData(Csr64.SharedStaticServerData);
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

/* E5 — NtRaiseException: dispatch a guest software raise into the guest's own
 * 32-bit SEH. The flat thunk handed the 64-bit kernel the guest's 32-bit
 * EXCEPTION_RECORD/CONTEXT to read at 64-bit layout — garbage. With no
 * debugger attached a first-chance user raise comes straight back to user
 * dispatch anyway, so skip the kernel round-trip: build the i386
 * KiUserExceptionDispatcher frame on the guest stack below the raiser's Esp
 * (Wow64pBuildGuestExceptionFrame, wow64.c) and point the CPU layer at it via
 * BTCpuSetContext — works on both arches (amd64 REPLACED-flag resume, arm64
 * FEX context reload). Second chance == the guest's dispatcher found no
 * handler: terminate the process with the exception code, loudly.
 *   NtRaiseException(ExceptionRecord, Context, FirstChance) */
#define WOW64_CTX32_SIZE 0x2CC /* i386 CONTEXT (through ExtendedRegisters) */
#define WOW64_CTX32_EIP  0xB8
#define WOW64_CTX32_ESP  0xC4
static NTSTATUS Wow64Manual_NtRaiseException(ULONG *a)
{
    extern LONG (WINAPI *Wow64pBTCpuSetContext)(HANDLE, HANDLE, PVOID, PVOID);
    extern LONG (WINAPI *Wow64pBTCpuGetContext)(HANDLE, HANDLE, PVOID, PVOID);
    extern PVOID Wow64pKiUserExceptionDispatcher32;
    extern ULONG Wow64pBuildGuestExceptionFrame(const void *Rec32,
                                                const void *Ctx32, ULONG Below);
    extern NTSTATUS WINAPI NtTerminateProcess(HANDLE Process, LONG ExitStatus);
    extern ULONG Wow64TraceSyscalls;
    const BYTE *Rec32 = (const BYTE *)(ULONG_PTR)a[0];
    const BYTE *Ctx32 = (const BYTE *)(ULONG_PTR)a[1];
    ULONG FirstChance = a[2];
    ULONG Resume[WOW64_CTX32_SIZE / 4];
    ULONG Live[WOW64_CTX32_SIZE / 4];
    ULONG Esp, i;

    if (!Rec32 || !Ctx32 || (ULONG_PTR)Rec32 >= 0x80000000u ||
        (ULONG_PTR)Ctx32 >= 0x80000000u)
        return (NTSTATUS)0xC000000DL;

    if (!FirstChance)
    {
        DbgPrint("[WOW64] second-chance exception %08lx eip=%08lx — unhandled in guest, terminating\n",
                 *(const ULONG *)Rec32, *(const ULONG *)(Ctx32 + WOW64_CTX32_EIP));
        NtTerminateProcess((HANDLE)(LONG_PTR)-1, (LONG)*(const ULONG *)Rec32);
        return (NTSTATUS)0xC000000DL; /* not reached */
    }

    if (!Wow64pKiUserExceptionDispatcher32 || !Wow64pBTCpuSetContext ||
        !Wow64pBTCpuGetContext)
        return (NTSTATUS)0xC000000DL;

    /* DBG_PRINTEXCEPTION(_WIDE)_C: natively the kernel's KD consumes these
     * (that's how OutputDebugString "returns" with no debugger). We bypass the
     * kernel, so consume them here the same way — print and resume at the
     * captured context. Dispatching them to guest SEH instead would kill the
     * process: the guest kernel32's __except is DUMMY_PSEH (never registers). */
    {
        ULONG Code = *(const ULONG *)Rec32;
        const ULONG *Prm = (const ULONG *)(Rec32 + 0x14); /* ExceptionInformation */
        if (Code == 0x40010006 /* DBG_PRINTEXCEPTION_C */ ||
            Code == 0x4001000A /* DBG_PRINTEXCEPTION_WIDE_C */)
        {
            ULONG Len = Prm[0], Str = Prm[1];
            if (Str && Str < 0x80000000u && Len && Len < 512)
            {
                if (Code == 0x40010006)
                    DbgPrint("[GUEST32] %.*s", (int)Len,
                             (const char *)(ULONG_PTR)Str);
                else
                    DbgPrint("[GUEST32] %.*ws", (int)Len,
                             (const WCHAR *)(ULONG_PTR)Str);
            }
            for (i = 0; i < WOW64_CTX32_SIZE / 4; i++)
                Resume[i] = ((const ULONG *)(ULONG_PTR)Ctx32)[i];
            return Wow64pBTCpuSetContext((HANDLE)(LONG_PTR)-2,
                                         (HANDLE)(LONG_PTR)-1, NULL, Resume);
        }
    }

    /* Frame placement: below the LIVE guest Esp (the syscall-stub esp from the
     * CPU layer), NOT below the captured Ctx32->Esp — the raiser's frames (and
     * the very Rec32/Ctx32 buffers being copied) live between the two. */
    for (i = 0; i < WOW64_CTX32_SIZE / 4; i++)
        Live[i] = 0;
    if (Wow64pBTCpuGetContext((HANDLE)(LONG_PTR)-2, (HANDLE)(LONG_PTR)-1,
                              NULL, Live) < 0)
        return (NTSTATUS)0xC000000DL;
    Esp = Wow64pBuildGuestExceptionFrame(Rec32, Ctx32,
                                         Live[WOW64_CTX32_ESP / 4] - 0x40);
    if (!Esp)
        return (NTSTATUS)0xC000000DL;

    if (Wow64TraceSyscalls)
        DbgPrint("[WOW64] NtRaiseException %08lx eip=%08lx -> 32-bit SEH dispatch (esp=%08lx)\n",
                 *(const ULONG *)Rec32, *(const ULONG *)(Ctx32 + WOW64_CTX32_EIP), Esp);

    for (i = 0; i < WOW64_CTX32_SIZE / 4; i++)
        Resume[i] = ((const ULONG *)(ULONG_PTR)Ctx32)[i];
    Resume[WOW64_CTX32_EIP / 4] = (ULONG)(ULONG_PTR)Wow64pKiUserExceptionDispatcher32;
    Resume[WOW64_CTX32_ESP / 4] = Esp;
    return Wow64pBTCpuSetContext((HANDLE)(LONG_PTR)-2, (HANDLE)(LONG_PTR)-1,
                                 NULL, Resume);
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
        Wow64pPfnMapSetPsi(Uc64.siClient.psi);
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

/*
 * Window/class procs crossing into the kernel. A guest system proc (Button's,
 * DefWindowProc, ...) is swapped for the host's proc in the same PFNCLIENT slot
 * (wow64_pfnmap.c), and a host proc coming back is swapped for the guest's.
 * The WNDPROC indices go through the *Ptr entry points because the 32-bit ones
 * narrow the value: the new host proc on the way in (SetClassLong's ULONG),
 * and the old one on the way out (SetWindowLong returns a LONG). Neither can
 * be mapped once it has lost its high half.
 */
extern LONG NtUserSetWindowLong(PVOID, ULONG, LONG, ULONG);
extern LONG_PTR NtUserSetWindowLongPtr(PVOID, ULONG, LONG_PTR, ULONG);
extern ULONG_PTR NtUserSetClassLong(PVOID, INT, ULONG, ULONG);
extern ULONG_PTR NtUserSetClassLongPtr(PVOID, INT, ULONG_PTR, ULONG);
extern ULONG_PTR NtUserGetCPD(PVOID, ULONG, ULONG_PTR);

#define WOW64_GWL_WNDPROC (-4)
#define WOW64_GCL_WNDPROC (-24)

/*
 * NtUserBuildPropList(hWnd, Buffer, BufferSize, Count). PROPLISTITEM is
 * {ATOM; HANDLE}: 8 bytes in the guest, 16 on the host. With a NULL buffer the
 * kernel reports the size NEEDED, in bytes of ITS items; with a buffer it
 * requires a whole number of items and reports how many it wrote. Flat, the
 * guest's EnumProps* read a 16-byte-stride array at an 8-byte stride.
 */
#define WOW64_PROPLISTITEM32_CB 8
#define WOW64_PROPLISTITEM64_CB 16
#define WOW64_PROPLIST_STACK 32

static NTSTATUS Wow64Manual_NtUserBuildPropList(ULONG *a)
{
    extern NTSTATUS NtUserBuildPropList(PVOID, PVOID, ULONG, ULONG *);
    typedef struct { USHORT Atom; ULONG Data; } ITEM32;      /* 8 bytes */
    typedef struct { USHORT Atom; ULONGLONG Data; } ITEM64;  /* 16 bytes */
    PVOID Wnd = (PVOID)(LONG_PTR)(LONG)a[0];
    ITEM32 *Out32 = (ITEM32 *)(ULONG_PTR)a[1];
    ULONG Size32 = a[2];
    ULONG *Count32 = (ULONG *)(ULONG_PTR)a[3];
    ITEM64 Stack[WOW64_PROPLIST_STACK], *Items = Stack;
    PVOID Heap = NULL;
    ULONG Count = 0, n, i;
    NTSTATUS St;

    C_ASSERT(sizeof(ITEM32) == WOW64_PROPLISTITEM32_CB);
    C_ASSERT(sizeof(ITEM64) == WOW64_PROPLISTITEM64_CB);

    if (!Out32)
    {
        St = NtUserBuildPropList(Wnd, NULL, 0, &Count);
        if (St >= 0 && Count32)
            *Count32 = Count / WOW64_PROPLISTITEM64_CB * WOW64_PROPLISTITEM32_CB;
        return St;
    }
    if (Size32 == 0 || Size32 % WOW64_PROPLISTITEM32_CB)
        return (NTSTATUS)0xC000000DL; /* STATUS_INVALID_PARAMETER, as the kernel says */

    n = Size32 / WOW64_PROPLISTITEM32_CB;
    if (n > WOW64_PROPLIST_STACK)
    {
        SIZE_T Size = (SIZE_T)n * WOW64_PROPLISTITEM64_CB;
        St = NtAllocateVirtualMemory((HANDLE)(LONG_PTR)-1, &Heap, 0, &Size,
                                     MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE);
        if (St < 0)
            return St;
        Items = (ITEM64 *)Heap;
    }

    St = NtUserBuildPropList(Wnd, Items, n * WOW64_PROPLISTITEM64_CB, &Count);
    if (St >= 0)
    {
        for (i = 0; i < Count && i < n; i++)
        {
            Out32[i].Atom = Items[i].Atom;
            Out32[i].Data = (ULONG)Items[i].Data; /* a handle-sized value */
        }
        if (Count32)
            *Count32 = Count;
    }

    if (Heap)
    {
        SIZE_T Free = 0;
        NtFreeVirtualMemory((HANDLE)(LONG_PTR)-1, &Heap, &Free, MEM_RELEASE);
    }
    return St;
}

/* NtUserSetWindowLong(hWnd, Index, NewValue, Ansi) */
static NTSTATUS Wow64Manual_NtUserSetWindowLong(ULONG *a)
{
    PVOID Wnd = (PVOID)(LONG_PTR)(LONG)a[0]; /* handle: sign-extend */

    if ((LONG)a[1] == WOW64_GWL_WNDPROC)
        return (NTSTATUS)Wow64pMapClientProc((ULONGLONG)NtUserSetWindowLongPtr(
            Wnd, a[1], (LONG_PTR)Wow64pUnmapClientProc(a[2]), a[3]));
    return (NTSTATUS)NtUserSetWindowLong(Wnd, a[1], (LONG)a[2], a[3]);
}

/* NtUserSetClassLong(hWnd, Offset, dwNewLong, Ansi) */
static NTSTATUS Wow64Manual_NtUserSetClassLong(ULONG *a)
{
    PVOID Wnd = (PVOID)(LONG_PTR)(LONG)a[0];

    if ((LONG)a[1] == WOW64_GCL_WNDPROC)
        return (NTSTATUS)Wow64pMapClientProc((ULONGLONG)NtUserSetClassLongPtr(
            Wnd, (INT)a[1], (ULONG_PTR)Wow64pUnmapClientProc(a[2]), a[3]));
    return (NTSTATUS)NtUserSetClassLong(Wnd, (INT)a[1], a[2], a[3]);
}

/* NtUserGetCPD(hWnd, Flags, ProcIn): returns a CPD handle (0xFFFFxxxx, which
 * the map leaves alone) or a proc. */
static NTSTATUS Wow64Manual_NtUserGetCPD(ULONG *a)
{
    PVOID Wnd = (PVOID)(LONG_PTR)(LONG)a[0];

    return (NTSTATUS)Wow64pMapClientProc(
        (ULONGLONG)NtUserGetCPD(Wnd, a[1], (ULONG_PTR)Wow64pUnmapClientProc(a[2])));
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
    /* A guest system proc becomes the host's (wow64_pfnmap.c): win32k picks an
     * ANSI window's A proc by comparing the class proc against gpsi. */
    W64.lpfnWndProc = Wow64pUnmapClientProc(W32->lpfnWndProc);
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

/* ---- E6: NtUserGetAtomName ----------------------------------------------- *
 * user32 GetClassInfo path (class.c) calls NtUserGetAtomName(atom,
 * PUNICODE_STRING) to resolve a class atom to its name. win32u.spec types it
 * flat as (long ptr), so the auto-thunk hands 64-bit win32k the guest's 8-byte
 * UNICODE_STRING32 read as a 16-byte UNICODE_STRING64 — Buffer is picked up
 * from the wrong offset (garbage), and the kernel writes the atom name through
 * it -> C0000005 -> BUGCHECK 0x1E under DUMMY_PSEH (calc's class-name lookup
 * killed the box). In/out marshal: widen US32->US64 (Buffer is a guest <4GB
 * pointer, zero-extended — the kernel writes the wide name straight into the
 * guest's own buffer), call, then narrow the updated Length back to the guest
 * US32.  NtUserGetAtomName(ATOM atom, PUNICODE_STRING pustrName) -> ULONG len */
static NTSTATUS Wow64Manual_NtUserGetAtomName(ULONG *a)
{
    extern ULONG NtUserGetAtomName(ULONG_PTR Atom, PVOID PUstrName);
    WOW64_UNICODE_STRING32_M *N32 = (WOW64_UNICODE_STRING32_M *)(ULONG_PTR)a[1];
    WOW64_UNICODE_STRING64_M N64;
    ULONG Ret;

    if (!N32)
        return 0;
    N64.Length = N32->Length;
    N64.MaximumLength = N32->MaximumLength;
    N64.__pad = 0;
    N64.Buffer = (ULONGLONG)(ULONG_PTR)N32->Buffer; /* guest buffer, kernel writes here */
    Ret = NtUserGetAtomName(a[0], &N64);
    N32->Length = N64.Length; /* kernel updated the byte count written */
    return Ret;
}

/* ---- E6: NtUserGetClassName ---------------------------------------------- *
 * user32 GetClassNameW/RealGetWindowClass — same UNICODE_STRING32<->64 gap as
 * GetAtomName (win32u.spec types it flat as (ptr long ptr)). The guest's
 * 4-byte-aligned US32 hit win32k's align-8 ProbeForWrite of a 16-byte US64 ->
 * STATUS_DATATYPE_MISALIGNMENT (0x80000002) -> BUGCHECK 0x1E (calc's window
 * class-name query). Widen IN, kernel writes the name into the guest Buffer,
 * narrow Length OUT.  NtUserGetClassName(HWND, BOOL Real, PUNICODE_STRING) */
static NTSTATUS Wow64Manual_NtUserGetClassName(ULONG *a)
{
    extern ULONG NtUserGetClassName(PVOID HWnd, ULONG Real, PVOID PClassName);
    WOW64_UNICODE_STRING32_M *N32 = (WOW64_UNICODE_STRING32_M *)(ULONG_PTR)a[2];
    WOW64_UNICODE_STRING64_M N64;
    ULONG Ret;

    if (!N32)
        return 0;
    N64.Length = N32->Length;
    N64.MaximumLength = N32->MaximumLength;
    N64.__pad = 0;
    N64.Buffer = (ULONGLONG)(ULONG_PTR)N32->Buffer; /* guest buffer, kernel writes here */
    Ret = NtUserGetClassName((PVOID)(ULONG_PTR)a[0], a[1], &N64);
    N32->Length = N64.Length;
    return Ret;
}

/* ---- E6: NtUserRegisterWindowMessage ------------------------------------- *
 * user32 RegisterWindowMessageW — every app that defines a private message
 * (comctl32, shell hooks, DDE) calls it. win32u.spec types it flat (ptr), so
 * the guest US32 message-name would be read as a US64 (garbage Buffer). IN
 * only (returns the message atom).  NtUserRegisterWindowMessage(PUNICODE_STRING) */
static NTSTATUS Wow64Manual_NtUserRegisterWindowMessage(ULONG *a)
{
    extern ULONG NtUserRegisterWindowMessage(PVOID PMessageName);
    WOW64_UNICODE_STRING32_M *N32 = (WOW64_UNICODE_STRING32_M *)(ULONG_PTR)a[0];
    WOW64_UNICODE_STRING64_M N64;

    if (!N32)
        return 0;
    N64.Length = N32->Length;
    N64.MaximumLength = N32->MaximumLength;
    N64.__pad = 0;
    N64.Buffer = (ULONGLONG)(ULONG_PTR)N32->Buffer; /* guest name buffer (read) */
    return NtUserRegisterWindowMessage(&N64);
}

/* ---- E6: NtUserGetKeyboardLayoutName ------------------------------------- *
 * user32 GetKeyboardLayoutNameW — same single-US in/out shape as GetAtomName;
 * the kernel writes the KLID (a 9-char string) into the guest Buffer.
 *   NtUserGetKeyboardLayoutName(_Inout_ PUNICODE_STRING) */
static NTSTATUS Wow64Manual_NtUserGetKeyboardLayoutName(ULONG *a)
{
    extern ULONG NtUserGetKeyboardLayoutName(PVOID PName);
    WOW64_UNICODE_STRING32_M *N32 = (WOW64_UNICODE_STRING32_M *)(ULONG_PTR)a[0];
    WOW64_UNICODE_STRING64_M N64;
    ULONG Ret;

    if (!N32)
        return 0;
    N64.Length = N32->Length;
    N64.MaximumLength = N32->MaximumLength;
    N64.__pad = 0;
    N64.Buffer = (ULONGLONG)(ULONG_PTR)N32->Buffer; /* guest buffer, kernel writes here */
    Ret = NtUserGetKeyboardLayoutName(&N64);
    N32->Length = N64.Length;
    return Ret;
}

/* ---- E7: NtUserBuildHwndList --------------------------------------------- *
 * EnumWindows/EnumChildWindows/EnumThreadWindows -> user32 -> this syscall,
 * which fills a caller HWND[] and returns the count. win32u.spec types the
 * array slot flat (ptr), so the auto-thunk hands 64-bit win32k the guest's
 * buffer verbatim — and the kernel writes 8-byte HWNDs into it. The guest
 * (user32 BuildHwndList) sized that buffer at sizeof(HWND)*count = 4 bytes
 * per entry, so the kernel's 8-bytes-per-entry write is a 2x OVERRUN that
 * clobbers the adjacent heap block; the damage surfaces minutes later as a
 * bad free-list link in RtlpRemoveFreeBlock (calc's button-enum killed the
 * guest heap this way — a NULL Blink -> BUGCHECK-shaped guest AV -> SEH
 * cascade). Marshal it: give the kernel a host 64-bit scratch array, then
 * narrow each handle's low 32 bits back into the guest's 4-byte-per-entry
 * buffer. The count slot (pcHwndNeeded) is a ULONG — same width 32/64 — so it
 * flat-passes. Signature:
 *   NtUserBuildHwndList(HDESK, HWND parent, BOOLEAN children, ULONG tid,
 *                       ULONG cHwnd, HWND *phwndList, ULONG *pcHwndNeeded) */
static NTSTATUS Wow64Manual_NtUserBuildHwndList(ULONG *a)
{
    extern NTSTATUS NtUserBuildHwndList(HANDLE hDesktop, HANDLE hwndParent,
                                        BOOLEAN bChildren, ULONG dwThreadId,
                                        ULONG cHwnd, PVOID phwndList,
                                        PULONG pcHwndNeeded);
    HANDLE hDesktop = (HANDLE)(ULONG_PTR)a[0];
    HANDLE hParent  = (HANDLE)(ULONG_PTR)a[1];
    BOOLEAN bChild  = (BOOLEAN)a[2];
    ULONG   dwTid   = a[3];
    ULONG   cHwnd   = a[4];
    ULONG  *pList32 = (ULONG *)(ULONG_PTR)a[5];       /* guest OUT array, 4B/entry */
    PULONG  pNeeded = (PULONG)(ULONG_PTR)a[6];        /* IN/OUT count, 4B both sides */
    PVOID   Tmp     = NULL;
    SIZE_T  Size;
    NTSTATUS St;
    ULONG   n, i;

    /* No array write (query-count call or empty buffer): flat-pass the guest
     * pointer — the kernel only touches pNeeded, which is width-identical. */
    if (!pList32 || cHwnd == 0)
        return NtUserBuildHwndList(hDesktop, hParent, bChild, dwTid, cHwnd,
                                   (PVOID)(ULONG_PTR)a[5], pNeeded);

    Size = (SIZE_T)cHwnd * sizeof(PVOID);             /* host: 8 bytes per HWND */
    St = NtAllocateVirtualMemory((HANDLE)(LONG_PTR)-1, &Tmp, 0, &Size,
                                 MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE);
    if (St < 0)
        return St;

    St = NtUserBuildHwndList(hDesktop, hParent, bChild, dwTid, cHwnd, Tmp, pNeeded);
    if (St >= 0)
    {
        n = pNeeded ? *pNeeded : 0;                   /* count kernel actually wrote */
        if (n > cHwnd)
            n = cHwnd;
        for (i = 0; i < n; i++)
            pList32[i] = (ULONG)((const ULONGLONG *)Tmp)[i]; /* narrow 64 -> 32 */
    }

    {
        SIZE_T Free = 0;
        NtFreeVirtualMemory((HANDLE)(LONG_PTR)-1, &Tmp, &Free, MEM_RELEASE);
    }
    return St;
}

/* ---- E4b: NtUserGetClassInfo --------------------------------------------- *
 * user32 GetClassInfoExW/A — hit by the stock create-window path (doom/testgui
 * probe their class before registering). The raw auto-thunk fed win32k the
 * guest's 8-byte UNICODE_STRING32 as 16 bytes (the garbage Buffer half
 * probe-raised C0000005 -> BUGCHECK 0x1E under DUMMY_PSEH on amd64; arm64 only
 * ever survived by stack-garbage luck) and then let the kernel RtlCopyMemory an
 * 80-byte WNDCLASSEXW over the guest's 48-byte struct — 32 bytes of guest
 * stack corruption even on success. Same WC32/WC64 layouts as
 * RegisterClassExWOW above, narrowed on the way OUT instead of widened IN. */
static NTSTATUS Wow64Manual_NtUserGetClassInfo(ULONG *a)
{
    extern ULONG NtUserGetClassInfo(PVOID, PVOID, PVOID, PVOID, ULONG);
    typedef struct { ULONG cbSize, style, lpfnWndProc; LONG cbClsExtra, cbWndExtra;
                     ULONG hInstance, hIcon, hCursor, hbrBackground,
                           lpszMenuName, lpszClassName, hIconSm; } WC32;
    typedef struct { ULONG cbSize, style; ULONGLONG lpfnWndProc;
                     LONG cbClsExtra, cbWndExtra;
                     ULONGLONG hInstance, hIcon, hCursor, hbrBackground,
                               lpszMenuName, lpszClassName, hIconSm; } WC64;
    WOW64_UNICODE_STRING32_M *CN32 = (WOW64_UNICODE_STRING32_M *)(ULONG_PTR)a[1];
    WC32 *W32 = (WC32 *)(ULONG_PTR)a[2];
    ULONG *MenuName32 = (ULONG *)(ULONG_PTR)a[3];
    WOW64_UNICODE_STRING64_M CN64;
    PVOID CN64p = NULL;
    WC64 W64;
    ULONGLONG MenuName64 = 0;
    ULONG Ret;

    if (!W32)
        return 0;
    if (CN32)
    {
        /* Buffer may be a raw atom (< 0x10000) — zero-extension keeps it one. */
        CN64.Length = CN32->Length; CN64.MaximumLength = CN32->MaximumLength;
        CN64.__pad = 0; CN64.Buffer = CN32->Buffer;
        CN64p = &CN64;
    }
    W64.cbSize = sizeof(W64);
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

    Ret = NtUserGetClassInfo((PVOID)(ULONG_PTR)a[0], CN64p, &W64, &MenuName64, a[4]);
    if (Ret)
    {
        /* Narrow back. A class whose members came from 64-bit code (a system
         * class wndproc, a >4GB menu-name mapping) cannot be represented for
         * the guest — trace it loudly instead of silently truncating. */
        if ((W64.lpszMenuName | W64.lpszClassName | MenuName64) >> 32 ||
            (W64.lpfnWndProc >> 32 &&
             Wow64pMapClientProc(W64.lpfnWndProc) == (ULONG)W64.lpfnWndProc))
            DbgPrint("[WOW64] NtUserGetClassInfo: 64-bit-only members "
                     "(proc=%I64x menu=%I64x cls=%I64x pmenu=%I64x) TRUNCATED\n",
                     W64.lpfnWndProc, W64.lpszMenuName, W64.lpszClassName,
                     MenuName64);
        W32->style = W64.style;
        W32->lpfnWndProc = Wow64pMapClientProc(W64.lpfnWndProc);
        W32->cbClsExtra = W64.cbClsExtra;
        W32->cbWndExtra = W64.cbWndExtra;
        W32->hInstance = (ULONG)W64.hInstance;
        W32->hIcon = (ULONG)W64.hIcon;
        W32->hCursor = (ULONG)W64.hCursor;
        W32->hbrBackground = (ULONG)W64.hbrBackground;
        W32->lpszMenuName = (ULONG)W64.lpszMenuName;
        W32->lpszClassName = (ULONG)W64.lpszClassName;
        W32->hIconSm = (ULONG)W64.hIconSm;
        if (MenuName32)
            *MenuName32 = (ULONG)MenuName64;
    }
    return (NTSTATUS)Ret;
}

/* ---- E4c: NtUserThunkedMenuItemInfo -------------------------------------- *
 * winmine's menu bar. user32 InsertMenuItemW/SetMenuItemInfoW pack the item as
 * MENUITEMINFOW (48 bytes on i386, 80 on the host — five pointer-width fields)
 * plus the caption as a separate PUNICODE_STRING. The auto-thunk handed win32k
 * the guest's 8-byte UNICODE_STRING32; its align-8 ProbeForRead raised
 * STATUS_DATATYPE_MISALIGNMENT -> BUGCHECK 0x1E under DUMMY_PSEH. */
static NTSTATUS Wow64Manual_NtUserThunkedMenuItemInfo(ULONG *a)
{
    extern ULONG NtUserThunkedMenuItemInfo(PVOID, ULONG, ULONG, ULONG, PVOID, PVOID);
    typedef struct { ULONG cbSize, fMask, fType, fState, wID,
                     hSubMenu, hbmpChecked, hbmpUnchecked,
                     dwItemData, dwTypeData, cch, hbmpItem; } MII32; /* 48 */
    typedef struct { ULONG cbSize, fMask, fType, fState, wID, __pad0;
                     ULONGLONG hSubMenu, hbmpChecked, hbmpUnchecked,
                               dwItemData, dwTypeData;
                     ULONG cch, __pad1;
                     ULONGLONG hbmpItem; } MII64; /* 80 */
    const MII32 *M32 = (const MII32 *)(ULONG_PTR)a[4];
    WOW64_UNICODE_STRING32_M *Cap32 = (WOW64_UNICODE_STRING32_M *)(ULONG_PTR)a[5];
    MII64 M64;
    WOW64_UNICODE_STRING64_M Cap64;
    PVOID M64p = NULL, Cap64p = NULL;

    if (M32)
    {
        M64.cbSize = sizeof(M64);
        M64.fMask = M32->fMask;
        M64.fType = M32->fType;
        M64.fState = M32->fState;
        M64.wID = M32->wID;
        M64.__pad0 = 0;
        M64.hSubMenu = M32->hSubMenu;
        M64.hbmpChecked = M32->hbmpChecked;
        M64.hbmpUnchecked = M32->hbmpUnchecked;
        M64.dwItemData = M32->dwItemData;
        M64.dwTypeData = M32->dwTypeData;
        M64.cch = M32->cch;
        M64.__pad1 = 0;
        M64.hbmpItem = M32->hbmpItem;
        M64p = &M64;
    }
    if (Cap32)
    {
        Cap64.Length = Cap32->Length;
        Cap64.MaximumLength = Cap32->MaximumLength;
        Cap64.__pad = 0;
        Cap64.Buffer = Cap32->Buffer;
        Cap64p = &Cap64;
    }
    return (NTSTATUS)NtUserThunkedMenuItemInfo((PVOID)(ULONG_PTR)a[0], a[1], a[2],
                                               a[3], M64p, Cap64p);
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

/* ---- E8: NtUserUnregisterClass ------------------------------------------- *
 * regedit/comctl32 class teardown. NtUserUnregisterClass(PUNICODE_STRING
 * ClassNameOrAtom, HINSTANCE, PCLSMENUNAME) is typed flat (ptr ptr ptr) in
 * win32u.spec, so wow64gen emits a silent-wrong FLAT thunk: 64-bit win32k reads
 * the guest's 8-byte UNICODE_STRING32 as a 16-byte UNICODE_STRING64, picks up
 * Buffer from the wrong offset (garbage), and ProbeAndCaptureUnicodeStringOrAtom
 * copies the "string" from that wild address -> C0000005 in win32k's raw_move
 * -> BUGCHECK 0x1E under DUMMY_PSEH (regedit killed the box on the amd64 E8
 * ladder). The class name marshals exactly like GetClassName/GetAtomName; the
 * atom form (Buffer < 0x10000, Length=0) survives the zero-extend + the kernel's
 * IS_ATOM check (Wow64pUS32to64 handles both). hInstance flat-passes.
 * pClassMenuName is IGNORED by the kernel (NtUserUnregisterClass passes NULL to
 * UserUnregisterClass — class.rs), so its guest pointer passes through untouched
 * (nothing is ever written back through it). */
static NTSTATUS Wow64Manual_NtUserUnregisterClass(ULONG *a)
{
    extern LONG NtUserUnregisterClass(PVOID ClassNameOrAtom, PVOID hInstance,
                                      PVOID pClassMenuName);
    WOW64_UNICODE_STRING64_M Name64;
    return (NTSTATUS)NtUserUnregisterClass(Wow64pUS32to64(a[0], &Name64),
                                           (PVOID)(ULONG_PTR)a[1],
                                           (PVOID)(ULONG_PTR)a[2]);
}

/* ---- E9: the flat-typed PUNICODE_STRING NtUser* sweep -------------------- *
 * A whole class of NtUser* syscalls take one or two PUNICODE_STRINGs but are
 * typed flat (ptr) in win32u.spec, so wow64gen emits a silent-wrong flat thunk:
 * 64-bit win32k reads the guest's 8-byte UNICODE_STRING32 as a 16-byte US64,
 * picks Buffer from the wrong offset (garbage) -> ProbeAndCapture* raw_move
 * C0000005 -> BUGCHECK 0x1E. osk.exe hit NtUserFindWindowEx this way (same
 * win32k raw_move PC as regedit's NtUserUnregisterClass). A spec-vs-prototype
 * scan (win32u.spec ptr args vs the real ntuser.h PUNICODE_STRING) found the
 * whole class; each is the same US32<->US64 widen (Wow64pUS32to64 handles NULL
 * + the atom form). Non-US args flat-pass; OUT DEVMODEW/DISPLAY_DEVICEW are
 * non-divergent (no pointers, identical 32/64) so the guest buffer passes
 * straight through and the kernel writes the shared layout. Handle returns
 * narrow to the guest's low 32 bits (all user handles are <4GB).
 * NtUserCreateDesktop ALSO carries a divergent struct (OBJECT_ATTRIBUTES) and
 * is listed in manual-win32k.txt WITHOUT a dispatch entry, so it falls through
 * to the generated table and comes back WOW64_AUTOMARSHAL_UNIMPL, which the
 * caller reports loudly by name (a clean error, NOT a bugcheck) until a guest
 * that needs it arrives. NtUserGetIconInfo was the other one; a guest reached
 * it (user32:class) and it has a marshaller now, further down. */
static NTSTATUS Wow64Manual_NtUserFindWindowEx(ULONG *a)
{
    extern HANDLE NtUserFindWindowEx(PVOID, PVOID, PVOID, PVOID, ULONG);
    WOW64_UNICODE_STRING64_M Cls64, Win64;
    HANDLE Ret = NtUserFindWindowEx((PVOID)(ULONG_PTR)a[0], (PVOID)(ULONG_PTR)a[1],
                                    Wow64pUS32to64(a[2], &Cls64),
                                    Wow64pUS32to64(a[3], &Win64), a[4]);
    return (NTSTATUS)(ULONG)(ULONG_PTR)Ret;
}

static NTSTATUS Wow64Manual_NtUserGetWOWClass(ULONG *a)
{
    extern PVOID NtUserGetWOWClass(PVOID, PVOID);
    WOW64_UNICODE_STRING64_M N64;
    PVOID Ret = NtUserGetWOWClass((PVOID)(ULONG_PTR)a[0], Wow64pUS32to64(a[1], &N64));
    return (NTSTATUS)(ULONG)(ULONG_PTR)Ret;
}

static NTSTATUS Wow64Manual_NtUserSetWindowsHookEx(ULONG *a)
{
    extern PVOID NtUserSetWindowsHookEx(PVOID, PVOID, ULONG, INT, PVOID, ULONG);
    WOW64_UNICODE_STRING64_M N64;
    PVOID Ret = NtUserSetWindowsHookEx((PVOID)(ULONG_PTR)a[0], Wow64pUS32to64(a[1], &N64),
                                       a[2], (INT)a[3], (PVOID)(ULONG_PTR)a[4], a[5]);
    return (NTSTATUS)(ULONG)(ULONG_PTR)Ret;
}

static NTSTATUS Wow64Manual_NtUserSetWinEventHook(ULONG *a)
{
    extern PVOID NtUserSetWinEventHook(ULONG, ULONG, PVOID, PVOID, PVOID, ULONG, ULONG, ULONG);
    WOW64_UNICODE_STRING64_M N64;
    PVOID Ret = NtUserSetWinEventHook(a[0], a[1], (PVOID)(ULONG_PTR)a[2],
                                      Wow64pUS32to64(a[3], &N64), (PVOID)(ULONG_PTR)a[4],
                                      a[5], a[6], a[7]);
    return (NTSTATUS)(ULONG)(ULONG_PTR)Ret;
}

static NTSTATUS Wow64Manual_NtUserRegisterUserApiHook(ULONG *a)
{
    extern LONG NtUserRegisterUserApiHook(PVOID, PVOID, ULONG, ULONG);
    WOW64_UNICODE_STRING64_M D64, F64;
    return (NTSTATUS)NtUserRegisterUserApiHook(Wow64pUS32to64(a[0], &D64),
                                               Wow64pUS32to64(a[1], &F64), a[2], a[3]);
}

static NTSTATUS Wow64Manual_NtUserDrawCaptionTemp(ULONG *a)
{
    extern LONG NtUserDrawCaptionTemp(PVOID, PVOID, PVOID, PVOID, PVOID, PVOID, ULONG);
    WOW64_UNICODE_STRING64_M S64;
    return (NTSTATUS)NtUserDrawCaptionTemp((PVOID)(ULONG_PTR)a[0], (PVOID)(ULONG_PTR)a[1],
                                           (PVOID)(ULONG_PTR)a[2], (PVOID)(ULONG_PTR)a[3],
                                           (PVOID)(ULONG_PTR)a[4], Wow64pUS32to64(a[5], &S64),
                                           a[6]);
}

static NTSTATUS Wow64Manual_NtUserLoadKeyboardLayoutEx(ULONG *a)
{
    extern PVOID NtUserLoadKeyboardLayoutEx(PVOID, ULONG, PVOID, PVOID, PVOID, ULONG, ULONG);
    WOW64_UNICODE_STRING64_M K64;
    PVOID Ret = NtUserLoadKeyboardLayoutEx((PVOID)(ULONG_PTR)a[0], a[1], (PVOID)(ULONG_PTR)a[2],
                                           (PVOID)(ULONG_PTR)a[3], Wow64pUS32to64(a[4], &K64),
                                           a[5], a[6]);
    return (NTSTATUS)(ULONG)(ULONG_PTR)Ret;
}

static NTSTATUS Wow64Manual_NtUserEnumDisplaySettings(ULONG *a)
{
    extern NTSTATUS NtUserEnumDisplaySettings(PVOID, ULONG, PVOID, ULONG);
    WOW64_UNICODE_STRING64_M N64;
    /* lpDevMode (a[2]) is DEVMODEW: no pointers, identical 32/64 -> flat pass. */
    return NtUserEnumDisplaySettings(Wow64pUS32to64(a[0], &N64), a[1],
                                     (PVOID)(ULONG_PTR)a[2], a[3]);
}

static NTSTATUS Wow64Manual_NtUserEnumDisplayDevices(ULONG *a)
{
    extern LONG NtUserEnumDisplayDevices(PVOID, ULONG, PVOID, ULONG);
    WOW64_UNICODE_STRING64_M N64;
    /* lpDisplayDevice (a[2]) is DISPLAY_DEVICEW: no pointers, identical 32/64. */
    return (NTSTATUS)NtUserEnumDisplayDevices(Wow64pUS32to64(a[0], &N64), a[1],
                                              (PVOID)(ULONG_PTR)a[2], a[3]);
}

/* ---- E10: the flat-thunk audit sweep ------------------------------------ *
 * E6/E8/E9 each found one member of this class the expensive way — a guest
 * called a syscall whose .spec types every arg flat, the flat thunk handed
 * 64-bit win32k a 32-bit payload, and the box bugchecked. `marshalgen audit`
 * (tools/wow64gen/marshalgen.py, wired into `vibeos-cli check`) now derives
 * the whole class mechanically: it diffs every LIVE flat thunk in the
 * generated table against its real header prototype using clang's own 32/64
 * record layouts. These are the survivors it found. */

/* NtGdiCreateDIBSection(hdc, hSectionApp, dwOffset, pbmi, iUsage, cjHeader,
 *                       fl, dwColorSpace, PVOID *ppvBits)
 * The E7 overrun shape, on a HOT path this time: guest gdi32 passes &bmBits —
 * a 4-byte stack local (gdi32-rust/src/objects/bitmap.rs) — and the flat thunk
 * let 64-bit win32k store an 8-byte PVOID there, smashing the 4 bytes that
 * follow it on the guest stack. Every DIB section (mspaint, magnify, any
 * double-buffered app) took that write. Give the kernel a host-width slot and
 * narrow it back. pbmi is a BITMAPINFO — pointer-free, identical 32/64 — so it
 * flat-passes, as does dwColorSpace (ULONG_PTR: one guest slot, zero-extended).
 *
 * The narrowed VA is the guest's only view of the mapping, so a bitmap mapped
 * above 4GB cannot be represented at all. Truncating silently would hand the
 * guest a pointer into nowhere and fault far from here, so fail the call
 * loudly instead — the guest sees a clean CreateDIBSection failure. */
static NTSTATUS Wow64Manual_NtGdiCreateDIBSection(ULONG *a)
{
    extern HANDLE NtGdiCreateDIBSection(HANDLE hdc, HANDLE hSectionApp, ULONG dwOffset,
                                        PVOID pbmi, ULONG iUsage, ULONG cjHeader,
                                        ULONG fl, ULONG_PTR dwColorSpace, PVOID *ppvBits);
    extern LONG NtGdiDeleteObjectApp(PVOID hobj);
    ULONG *pBits32 = (ULONG *)(ULONG_PTR)a[8];
    PVOID Bits64 = NULL;
    HANDLE Ret;

    Ret = NtGdiCreateDIBSection((HANDLE)(ULONG_PTR)a[0], (HANDLE)(LONG_PTR)(LONG)a[1],
                                a[2], (PVOID)(ULONG_PTR)a[3], a[4], a[5], a[6],
                                (ULONG_PTR)a[7], pBits32 ? &Bits64 : NULL);
    if (Ret && pBits32)
    {
        if ((ULONG_PTR)Bits64 >> 32)
        {
            /* Unrepresentable in the guest's 32-bit world — refuse rather than
             * hand back a truncated pointer. */
            DbgPrint("[WOW64]   NtGdiCreateDIBSection mapped bits at %p (above 4GB) — "
                     "unrepresentable to a 32-bit guest; failing the call\n", Bits64);
            NtGdiDeleteObjectApp((PVOID)Ret);
            *pBits32 = 0;
            return 0;
        }
        *pBits32 = (ULONG)(ULONG_PTR)Bits64;
    }
    return (NTSTATUS)(ULONG)(ULONG_PTR)Ret;
}

/* The rest of the MSG family. `tagMSG` is 28 bytes on i386 and 48 on the host
 * (HWND + ULONG_PTR wParam/lParam all widen), and the pump three —
 * GetMessage/PeekMessage/DispatchMessage — have been marshalled since M8. The
 * audit caught three siblings that were still flat, and every one of them is on
 * an ordinary app path: TranslateAccelerator (any app with an accelerator
 * table), CallMsgFilter (modal loops / MSGF_* hooks), RealInternalGetMessage
 * (the internal pump user32 uses behind some dialogs). They reuse the same
 * Wow64pMsg32To64 / Wow64pMsg64To32 pair. */
static NTSTATUS Wow64Manual_NtUserTranslateAccelerator(ULONG *a)
{
    extern LONG NtUserTranslateAccelerator(PVOID Window, PVOID Table, PVOID Message);
    WOW64_MSG32 *M32 = (WOW64_MSG32 *)(ULONG_PTR)a[2];
    WOW64_MSG64 M64;

    if (!M32)
        return 0;
    Wow64pMsg32To64(M32, &M64); /* IN only — the kernel reads, never writes */
    return (NTSTATUS)NtUserTranslateAccelerator((PVOID)(ULONG_PTR)a[0],
                                                (PVOID)(ULONG_PTR)a[1], &M64);
}

static NTSTATUS Wow64Manual_NtUserCallMsgFilter(ULONG *a)
{
    extern LONG NtUserCallMsgFilter(PVOID msg, LONG code);
    WOW64_MSG32 *M32 = (WOW64_MSG32 *)(ULONG_PTR)a[0];
    WOW64_MSG64 M64;
    LONG Ret;

    if (!M32)
        return 0;
    /* IN/OUT: a MSGF_* hook may rewrite the message before it is dispatched. */
    Wow64pMsg32To64(M32, &M64);
    Ret = NtUserCallMsgFilter(&M64, (LONG)a[1]);
    Wow64pMsg64To32(&M64, M32);
    return (NTSTATUS)Ret;
}

static NTSTATUS Wow64Manual_NtUserRealInternalGetMessage(ULONG *a)
{
    extern LONG NtUserRealInternalGetMessage(PVOID lpMsg, PVOID hWnd, ULONG Min,
                                             ULONG Max, ULONG Remove, ULONG bGMSG);
    WOW64_MSG32 *M32 = (WOW64_MSG32 *)(ULONG_PTR)a[0];
    WOW64_MSG64 M64;
    LONG Ret;

    if (!M32)
        return 0;
    Ret = NtUserRealInternalGetMessage(&M64, (PVOID)(ULONG_PTR)a[1], a[2], a[3], a[4], a[5]);
    if (Ret)
        Wow64pMsg64To32(&M64, M32);
    return (NTSTATUS)Ret;
}

/* NtUserResolveDesktop(ProcessHandle, PUNICODE_STRING DesktopPath, bInherit,
 *                      HWINSTA *phWinSta) — BOTH halves of the class in one
 * call: the path widens US32->US64, and the OUT window-station handle is a
 * 4-byte guest cell the kernel would fill with 8 bytes. */
static NTSTATUS Wow64Manual_NtUserResolveDesktop(ULONG *a)
{
    extern HANDLE NtUserResolveDesktop(HANDLE ProcessHandle, PVOID DesktopPath,
                                       ULONG bInherit, PVOID phWinSta);
    WOW64_UNICODE_STRING64_M Path64;
    ULONG *pWinSta32 = (ULONG *)(ULONG_PTR)a[3];
    HANDLE hWinSta = NULL;
    HANDLE Ret;

    Ret = NtUserResolveDesktop((HANDLE)(LONG_PTR)(LONG)a[0],
                               Wow64pUS32to64(a[1], &Path64), a[2],
                               pWinSta32 ? &hWinSta : NULL);
    if (pWinSta32)
        *pWinSta32 = (ULONG)(ULONG_PTR)hWinSta;
    return (NTSTATUS)(ULONG)(ULONG_PTR)Ret;
}

/* The USER OBJECT_ATTRIBUTES takers. OBJECT_ATTRIBUTES is 24 bytes on i386 and
 * 48 on the host, with a nested UNICODE_STRING — the same shape the whole
 * ntdll open family already marshals through Wow64pMarshalOa, so these reuse
 * that proven helper rather than a new one. */
static NTSTATUS Wow64Manual_NtUserOpenDesktop(ULONG *a)
{
    extern HANDLE NtUserOpenDesktop(PVOID ObjectAttributes, ULONG dwFlags, ACCESS_MASK Access);
    WOW64_OBJECT_ATTRIBUTES64_M Oa64;
    WOW64_UNICODE_STRING64_M Uname64;
    HANDLE Ret;

    if (!a[0])
        return (NTSTATUS)(ULONG)(ULONG_PTR)NtUserOpenDesktop(NULL, a[1], a[2]);
    Wow64pMarshalOa(&Oa64, &Uname64, a[0]);
    Ret = NtUserOpenDesktop(&Oa64, a[1], a[2]);
    return (NTSTATUS)(ULONG)(ULONG_PTR)Ret;
}

static NTSTATUS Wow64Manual_NtUserOpenWindowStation(ULONG *a)
{
    extern HANDLE NtUserOpenWindowStation(PVOID ObjectAttributes, ACCESS_MASK Access);
    WOW64_OBJECT_ATTRIBUTES64_M Oa64;
    WOW64_UNICODE_STRING64_M Uname64;
    HANDLE Ret;

    if (!a[0])
        return (NTSTATUS)(ULONG)(ULONG_PTR)NtUserOpenWindowStation(NULL, a[1]);
    Wow64pMarshalOa(&Oa64, &Uname64, a[0]);
    Ret = NtUserOpenWindowStation(&Oa64, a[1]);
    return (NTSTATUS)(ULONG)(ULONG_PTR)Ret;
}

static NTSTATUS Wow64Manual_NtUserCreateWindowStation(ULONG *a)
{
    extern HANDLE NtUserCreateWindowStation(PVOID ObjectAttributes, ACCESS_MASK Access,
                                            ULONG Unknown2, ULONG Unknown3, ULONG Unknown4,
                                            ULONG Unknown5, ULONG Unknown6);
    WOW64_OBJECT_ATTRIBUTES64_M Oa64;
    WOW64_UNICODE_STRING64_M Uname64;
    HANDLE Ret;

    if (!a[0])
        return (NTSTATUS)(ULONG)(ULONG_PTR)NtUserCreateWindowStation(NULL, a[1], a[2], a[3],
                                                                     a[4], a[5], a[6]);
    Wow64pMarshalOa(&Oa64, &Uname64, a[0]);
    Ret = NtUserCreateWindowStation(&Oa64, a[1], a[2], a[3], a[4], a[5], a[6]);
    return (NTSTATUS)(ULONG)(ULONG_PTR)Ret;
}

/* Shared-icon cache diagnosis: a WOW64 app whose LR_SHARED icons never match re-creates them on
 * every paint and exhausts the GDI quota. Log what the two cache calls actually carry. Behind the
 * syscall-trace switch: a line per icon lookup is ~1500 lines a sweep once the cache works. */
extern ULONG Wow64TraceSyscalls;

static void Wow64pTraceCurIcon(const char *What, ULONG Mod32, ULONG Rsrc32, LONG *Param,
                               ULONG Flags, ULONG Ret)
{
    if (!Wow64TraceSyscalls)
        return;
    WOW64_UNICODE_STRING32_M *M = (WOW64_UNICODE_STRING32_M *)(ULONG_PTR)Mod32;
    WOW64_UNICODE_STRING32_M *R = (WOW64_UNICODE_STRING32_M *)(ULONG_PTR)Rsrc32;
    DbgPrint("[WOW64-CURICON] %s mod=%.*S rsrc=%s%.*S%s id=%lx len=%u flags=%lx param=%ld,%ld,%ld ret=%lx\n",
             What,
             M && M->Buffer ? (int)(M->Length / 2) : 0, M && M->Buffer ? (PWSTR)(ULONG_PTR)M->Buffer : L"",
             R && R->Buffer >= 0x10000 ? "\"" : "",
             R && R->Buffer >= 0x10000 ? (int)(R->Length / 2) : 0,
             R && R->Buffer >= 0x10000 ? (PWSTR)(ULONG_PTR)R->Buffer : L"",
             R && R->Buffer >= 0x10000 ? "\"" : "",
             R ? (ULONG)R->Buffer : 0, R ? R->Length : 0, Flags,
             Param ? Param[0] : -1, Param ? Param[1] : -1, Param ? Param[2] : -1, Ret);
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
    Wow64pTraceCurIcon("FindExisting", a[0], a[1], a[2] ? ((LONG *)(ULONG_PTR)a[2]) : NULL, 0,
                       (ULONG)(ULONG_PTR)Ret);
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

    Wow64pTraceCurIcon("SetData", a[1], a[2], NULL, C32 ? C32->CURSORF_flags : 0xffffffff, a[0]);
    return NtUserSetCursorIconData((HANDLE)(LONG_PTR)(LONG)a[0],
                                   Wow64pUS32to64(a[1], &Mod64),
                                   Wow64pUS32to64(a[2], &Rsrc64),
                                   C32 ? &C64 : NULL);
}

/* NtUserGetIconInfo(hCurIcon, IconInfo, lpModule, lpResName, pbpp, bInternal):
 * ICONINFO carries two HBITMAPs (20 bytes guest, 32 host) and both strings
 * are IN/OUT: win32k reads MaximumLength/Buffer, then writes Length,
 * MaximumLength (the size needed, when too small) and, for an integer
 * resource, Buffer itself (the id). So each string goes up and comes back
 * down field by field. With no marshaller every guest GetIconInfo and
 * CopyImage failed, which is how user32:class lost the class small icon
 * win32k derives through the COPYIMAGE callback. */
typedef struct { ULONG fIcon, xHotspot, yHotspot, hbmMask, hbmColor; } WOW64_ICONINFO32_M;
typedef struct { ULONG fIcon, xHotspot, yHotspot, __pad;
                 ULONGLONG hbmMask, hbmColor; } WOW64_ICONINFO64_M;

static VOID Wow64pUSUp(ULONG U32Ptr, WOW64_UNICODE_STRING64_M *U64)
{
    const WOW64_UNICODE_STRING32_M *U32 = (const WOW64_UNICODE_STRING32_M *)(ULONG_PTR)U32Ptr;
    U64->Length = U32->Length;
    U64->MaximumLength = U32->MaximumLength;
    U64->__pad = 0;
    U64->Buffer = U32->Buffer;
}

static VOID Wow64pUSDown(const WOW64_UNICODE_STRING64_M *U64, ULONG U32Ptr)
{
    WOW64_UNICODE_STRING32_M *U32 = (WOW64_UNICODE_STRING32_M *)(ULONG_PTR)U32Ptr;
    U32->Length = U64->Length;
    U32->MaximumLength = U64->MaximumLength;
    U32->Buffer = (ULONG)U64->Buffer;
}

static NTSTATUS Wow64Manual_NtUserGetIconInfo(ULONG *a)
{
    extern BOOL NtUserGetIconInfo(HANDLE, PVOID, PVOID, PVOID, PVOID, BOOL);
    WOW64_ICONINFO64_M Ii64;
    WOW64_UNICODE_STRING64_M Mod64, Res64;
    BOOL Ret;

    if (a[2]) Wow64pUSUp(a[2], &Mod64);
    if (a[3]) Wow64pUSUp(a[3], &Res64);
    Ret = NtUserGetIconInfo((HANDLE)(LONG_PTR)(LONG)a[0],
                            a[1] ? &Ii64 : NULL,
                            a[2] ? &Mod64 : NULL,
                            a[3] ? &Res64 : NULL,
                            (PVOID)(ULONG_PTR)a[4], (BOOL)a[5]);
    if (Ret && a[1])
    {
        WOW64_ICONINFO32_M *Ii32 = (WOW64_ICONINFO32_M *)(ULONG_PTR)a[1];
        Ii32->fIcon = Ii64.fIcon;
        Ii32->xHotspot = Ii64.xHotspot;
        Ii32->yHotspot = Ii64.yHotspot;
        Ii32->hbmMask = (ULONG)Ii64.hbmMask;
        Ii32->hbmColor = (ULONG)Ii64.hbmColor;
    }
    /* A too-small string buffer is answered in the string itself (Length 0,
     * the needed MaximumLength), so the strings come back down whatever Ret
     * says. */
    if (a[2]) Wow64pUSDown(&Mod64, a[2]);
    if (a[3]) Wow64pUSDown(&Res64, a[3]);
    return (NTSTATUS)Ret;
}

/* NtGdiExtGetObjectW(h, cj, pvOut): win32k answers in its OWN layouts, and
 * three of them have a pointer-width field. BITMAP ends in LPVOID bmBits (24
 * bytes guest, 32 host), DIBSECTION adds a HANDLE (84 / 104), LOGBRUSH ends in
 * ULONG_PTR lbHatch (12 / 16), and EXTLOGPEN has ULONG_PTR elpHatch before its
 * style array (24+4n / 28+4n). BITMAP_GetObject refuses a buffer smaller than
 * its sizeof(BITMAP), so with the flat thunk every guest GetObject on a bitmap
 * returned 0: CopyImage could not copy an icon (user32:class, class.c:1214).
 * The rest (LOGPEN, LOGFONTW, a palette's WORD) are width-identical and pass
 * straight through. */
#define WOW64_GDI_TYPE_MASK   0x007f0000
#define WOW64_GDI_TYPE_BITMAP 0x00050000
#define WOW64_GDI_TYPE_BRUSH  0x00100000
#define WOW64_GDI_TYPE_PEN    0x00300000
#define WOW64_GDI_TYPE_EXTPEN 0x00500000

typedef struct { LONG bmType, bmWidth, bmHeight, bmWidthBytes;
                 USHORT bmPlanes, bmBitsPixel; ULONG bmBits; } WOW64_BITMAP32_M;
typedef struct { LONG bmType, bmWidth, bmHeight, bmWidthBytes;
                 USHORT bmPlanes, bmBitsPixel; ULONGLONG bmBits; } WOW64_BITMAP64_M;
typedef struct { WOW64_BITMAP32_M dsBm; BYTE dsBmih[40]; ULONG dsBitfields[3];
                 ULONG dshSection; ULONG dsOffset; } WOW64_DIBSECTION32_M;
typedef struct { WOW64_BITMAP64_M dsBm; BYTE dsBmih[40]; ULONG dsBitfields[3];
                 ULONGLONG dshSection; ULONG dsOffset; } WOW64_DIBSECTION64_M;
typedef struct { ULONG lbStyle, lbColor, lbHatch; } WOW64_LOGBRUSH32_M;
typedef struct { ULONG lbStyle, lbColor; ULONGLONG lbHatch; } WOW64_LOGBRUSH64_M;
/* EXTLOGPEN: four DWORDs, the hatch, elpNumEntries, then the style array. */
#define WOW64_EXTPEN32_HEAD 24
#define WOW64_EXTPEN64_HEAD 28
C_ASSERT(sizeof(WOW64_BITMAP32_M) == 24);
C_ASSERT(sizeof(WOW64_BITMAP64_M) == 32);
C_ASSERT(sizeof(WOW64_DIBSECTION32_M) == 84);
C_ASSERT(sizeof(WOW64_DIBSECTION64_M) == 104);
C_ASSERT(sizeof(WOW64_LOGBRUSH32_M) == 12);
C_ASSERT(sizeof(WOW64_LOGBRUSH64_M) == 16);

static VOID Wow64pBitmapDown(const WOW64_BITMAP64_M *B64, WOW64_BITMAP32_M *B32)
{
    B32->bmType = B64->bmType;
    B32->bmWidth = B64->bmWidth;
    B32->bmHeight = B64->bmHeight;
    B32->bmWidthBytes = B64->bmWidthBytes;
    B32->bmPlanes = B64->bmPlanes;
    B32->bmBitsPixel = B64->bmBitsPixel;
    B32->bmBits = (ULONG)B64->bmBits;
}

static NTSTATUS Wow64Manual_NtGdiExtGetObjectW(ULONG *a)
{
    extern INT NtGdiExtGetObjectW(HANDLE, INT, PVOID);
    HANDLE h = (HANDLE)(LONG_PTR)(LONG)a[0];
    INT cj = (INT)a[1];
    BYTE *Out = (BYTE *)(ULONG_PTR)a[2];
    union {
        WOW64_DIBSECTION64_M Ds;
        WOW64_LOGBRUSH64_M Lb;
        BYTE Pen[WOW64_EXTPEN64_HEAD + 64 * sizeof(ULONG)];
    } Host;
    INT r, i;

    switch (a[0] & WOW64_GDI_TYPE_MASK)
    {
        case WOW64_GDI_TYPE_BITMAP:
            if (!Out)
            {
                r = NtGdiExtGetObjectW(h, cj, NULL);
                return r == sizeof(WOW64_BITMAP64_M) ? sizeof(WOW64_BITMAP32_M) : r;
            }
            if (cj < (INT)sizeof(WOW64_BITMAP32_M))
                return 0;
            r = NtGdiExtGetObjectW(h, cj >= (INT)sizeof(WOW64_DIBSECTION32_M)
                                           ? sizeof(WOW64_DIBSECTION64_M)
                                           : sizeof(WOW64_BITMAP64_M), &Host);
            if (r < (INT)sizeof(WOW64_BITMAP64_M))
                return 0;
            Wow64pBitmapDown(&Host.Ds.dsBm, (WOW64_BITMAP32_M *)Out);
            if (r < (INT)sizeof(WOW64_DIBSECTION64_M))
                return sizeof(WOW64_BITMAP32_M);
            {
                WOW64_DIBSECTION32_M *D32 = (WOW64_DIBSECTION32_M *)Out;
                for (i = 0; i < 40; i++) D32->dsBmih[i] = Host.Ds.dsBmih[i];
                for (i = 0; i < 3; i++) D32->dsBitfields[i] = Host.Ds.dsBitfields[i];
                D32->dshSection = (ULONG)Host.Ds.dshSection;
                D32->dsOffset = Host.Ds.dsOffset;
            }
            return sizeof(WOW64_DIBSECTION32_M);

        case WOW64_GDI_TYPE_BRUSH:
            if (!Out)
            {
                r = NtGdiExtGetObjectW(h, cj, NULL);
                return r == sizeof(WOW64_LOGBRUSH64_M) ? sizeof(WOW64_LOGBRUSH32_M) : r;
            }
            if (cj < (INT)sizeof(WOW64_LOGBRUSH32_M))
                return 0;
            r = NtGdiExtGetObjectW(h, sizeof(WOW64_LOGBRUSH64_M), &Host);
            if (r < (INT)sizeof(WOW64_LOGBRUSH64_M))
                return 0;
            ((WOW64_LOGBRUSH32_M *)Out)->lbStyle = Host.Lb.lbStyle;
            ((WOW64_LOGBRUSH32_M *)Out)->lbColor = Host.Lb.lbColor;
            ((WOW64_LOGBRUSH32_M *)Out)->lbHatch = (ULONG)Host.Lb.lbHatch;
            return sizeof(WOW64_LOGBRUSH32_M);

        case WOW64_GDI_TYPE_PEN:
        case WOW64_GDI_TYPE_EXTPEN:
        {
            /* An old-style pen answers with a 16-byte LOGPEN (identical), an
             * extended one with an EXTLOGPEN 4 bytes longer than the guest's.
             * PEN_GetObject tells them apart by the size it is given, so the
             * guest's EXTLOGPEN-sized request is passed up 4 bytes larger. */
            INT cjHost = cj;
            if (!Out)
            {
                r = NtGdiExtGetObjectW(h, cj, NULL);
                return r >= WOW64_EXTPEN64_HEAD ? r - 4 : r;
            }
            if (cj >= WOW64_EXTPEN32_HEAD)
                cjHost = cj + 4;
            if (cjHost > (INT)sizeof(Host.Pen))
                cjHost = sizeof(Host.Pen);
            r = NtGdiExtGetObjectW(h, cjHost, Host.Pen);
            if (r < WOW64_EXTPEN64_HEAD)
            {
                for (i = 0; i < r; i++) Out[i] = Host.Pen[i];
                return r;
            }
            for (i = 0; i < 16; i++) Out[i] = Host.Pen[i];               /* 4 DWORDs */
            *(ULONG *)(Out + 16) = (ULONG)*(ULONGLONG *)(Host.Pen + 16); /* elpHatch */
            for (i = 24; i < r; i++) Out[i - 4] = Host.Pen[i];           /* count + styles */
            return r - 4;
        }

        default:
            return NtGdiExtGetObjectW(h, cj, Out);
    }
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

/* NtDeviceIoControlFile / NtFsControlFile — M15c.
 *
 * This comment used to say the in/out buffers are "OPAQUE to us ... We can't
 * know which from here", and predicted that a pointer-bearing IOCTL would show
 * up as a bad pointer inside the driver. The prediction was right; the premise
 * was wrong. The IoControlCode IS knowable — it is argument 5 — so the payload
 * type is derivable from it, and clang can settle whether that payload changes
 * shape. tools/wow64gen/ioctlaudit.py does exactly that against the LIVE driver
 * (afd-rust; the driver's C sources on disk are cohab-flipped dead source) and
 * records the verdicts in tools/lint-baseline/wow64_ioctl_audit.txt.
 *
 * Nine AFD payloads change shape between a 32-bit guest and this process, i.e.
 * every socket a 32-bit app opens was passing through here mis-shaped.
 *
 * Flat-passing them is worse than a truncation, because AFD is METHOD_NEITHER:
 * the driver dereferences the guest's buffer IN PLACE and writes back to it —
 *     afd-rust/src/write.rs:  (*SendReq).BufferArray = LockBuffers(...)
 * — so an 8-byte kernel pointer landed inside a 16-byte guest struct, clobbering
 * BufferCount and leaking a kernel address to the guest. Marshalling through a
 * host-side shadow fixes both halves: the driver reads a correctly-shaped
 * struct, and its write-back lands in the shadow, which is where a kernel
 * pointer belongs.
 *
 * Codes NOT in the table keep the old flat pass. That is deliberate: the audit
 * says which ones diverge, and inventing marshalling for a width-identical
 * payload would be its own bug. An audit row that is neither IDENTICAL nor
 * handled here is a gap, and `ioctlaudit.py check` is what refuses to let a new
 * one appear silently. */

#define WOW64_STATUS_INVALID_PARAMETER ((NTSTATUS)0xC000000D)
#define WOW64_STATUS_NO_MEMORY         ((NTSTATUS)0xC0000017)

/* FILE_DEVICE_NETWORK << 12 | op << 2 | method — afd/shared.h's
 * _AFD_CONTROL_CODE, mirrored so this DLL need not drag in a kernel header. The
 * audit reads the same codes out of the live driver, so a drift between these
 * and afd-rust turns up as a changed baseline row rather than a silent miss. */
#define WOW64_AFD_CTL(op, method) \
    (((ULONG)0x12 << 12) | ((ULONG)(op) << 2) | (ULONG)(method))

#define WOW64_IOCTL_AFD_ACCEPT              WOW64_AFD_CTL(4,  0)
#define WOW64_IOCTL_AFD_RECV                WOW64_AFD_CTL(5,  3)
#define WOW64_IOCTL_AFD_RECV_DATAGRAM       WOW64_AFD_CTL(6,  3)
#define WOW64_IOCTL_AFD_SEND                WOW64_AFD_CTL(7,  3)
#define WOW64_IOCTL_AFD_SEND_DATAGRAM       WOW64_AFD_CTL(8,  3)
#define WOW64_IOCTL_AFD_SELECT              WOW64_AFD_CTL(9,  0)
#define WOW64_IOCTL_AFD_GET_TDI_HANDLES     WOW64_AFD_CTL(13, 3)
#define WOW64_IOCTL_AFD_EVENT_SELECT        WOW64_AFD_CTL(33, 3)
#define WOW64_IOCTL_AFD_ENUM_NETWORK_EVENTS WOW64_AFD_CTL(34, 3)

/* Guest (32) and host (64) shapes. Sizes are asserted against the numbers the
 * audit baseline records, so a header change breaks the build here AND turns
 * the audit red — one of those alone could be argued away. */
#include <pshpack4.h>
typedef struct { ULONG len; ULONG buf; }                       WOW64_AFD_WSABUF32;
typedef struct { ULONG BufferArray, BufferCount, AfdFlags, TdiFlags; }
                                                               WOW64_AFD_XFER32;
typedef struct { ULONG BufferArray, BufferCount, AfdFlags, TdiFlags;
                 ULONG Address, AddressLength; }               WOW64_AFD_XFERUDP32;
typedef struct { ULONG BufferArray, BufferCount, AfdFlags;
                 ULONG TdiRequest[5];
                 LONG UserDataLength; ULONG UserData;
                 LONG OptionsLength;  ULONG Options;
                 LONG RemoteAddressLength; ULONG RemoteAddress; }
                                                               WOW64_AFD_SENDUDP32;
typedef struct { ULONG Handle, Events; LONG Status; }          WOW64_AFD_HANDLE32;
typedef struct { LONGLONG Timeout; ULONG HandleCount, Exclusive; }
                                                               WOW64_AFD_POLLHDR32;
typedef struct { ULONG EventObject, Events; }                  WOW64_AFD_EVSEL32;
typedef struct { ULONG Event, PollEvents; LONG EventStatus[10]; }
                                                               WOW64_AFD_ENUMEV32;
typedef struct { ULONG UseSAN, SequenceNumber, ListenHandle; } WOW64_AFD_ACCEPT32;
typedef struct { ULONG TdiAddressHandle, TdiConnectionHandle; } WOW64_AFD_TDIH32;
#include <poppack.h>

typedef struct { ULONG len; ULONG _p; ULONGLONG buf; }         WOW64_AFD_WSABUF64;
typedef struct { ULONGLONG BufferArray; ULONG BufferCount, AfdFlags, TdiFlags, _p; }
                                                               WOW64_AFD_XFER64;
typedef struct { ULONGLONG BufferArray; ULONG BufferCount, AfdFlags, TdiFlags, _p;
                 ULONGLONG Address, AddressLength; }           WOW64_AFD_XFERUDP64;
typedef struct { ULONGLONG BufferArray; ULONG BufferCount, AfdFlags;
                 ULONGLONG TdiRequest[5];
                 LONG UserDataLength; ULONG _p0; ULONGLONG UserData;
                 LONG OptionsLength;  ULONG _p1; ULONGLONG Options;
                 LONG RemoteAddressLength; ULONG _p2; ULONGLONG RemoteAddress; }
                                                               WOW64_AFD_SENDUDP64;
typedef struct { ULONGLONG Handle; ULONG Events; LONG Status; } WOW64_AFD_HANDLE64;
typedef struct { LONGLONG Timeout; ULONG HandleCount, _p; ULONGLONG Exclusive; }
                                                               WOW64_AFD_POLLHDR64;
typedef struct { ULONGLONG EventObject; ULONG Events, _p; }    WOW64_AFD_EVSEL64;
typedef struct { ULONGLONG Event; ULONG PollEvents; LONG EventStatus[10]; ULONG _p; }
                                                               WOW64_AFD_ENUMEV64;
typedef struct { ULONG UseSAN, SequenceNumber; ULONGLONG ListenHandle; }
                                                               WOW64_AFD_ACCEPT64;
typedef struct { ULONGLONG TdiAddressHandle, TdiConnectionHandle; }
                                                               WOW64_AFD_TDIH64;

C_ASSERT(sizeof(WOW64_AFD_WSABUF32) == 8);  C_ASSERT(sizeof(WOW64_AFD_WSABUF64) == 16);
C_ASSERT(sizeof(WOW64_AFD_XFER32) == 16);   C_ASSERT(sizeof(WOW64_AFD_XFER64) == 24);
C_ASSERT(sizeof(WOW64_AFD_XFERUDP32) == 24);C_ASSERT(sizeof(WOW64_AFD_XFERUDP64) == 40);
C_ASSERT(sizeof(WOW64_AFD_SENDUDP32) == 56);C_ASSERT(sizeof(WOW64_AFD_SENDUDP64) == 104);
C_ASSERT(sizeof(WOW64_AFD_HANDLE32) == 12); C_ASSERT(sizeof(WOW64_AFD_HANDLE64) == 16);
C_ASSERT(sizeof(WOW64_AFD_EVSEL32) == 8);   C_ASSERT(sizeof(WOW64_AFD_EVSEL64) == 16);
C_ASSERT(sizeof(WOW64_AFD_ENUMEV32) == 48); C_ASSERT(sizeof(WOW64_AFD_ENUMEV64) == 56);
C_ASSERT(sizeof(WOW64_AFD_ACCEPT32) == 12); C_ASSERT(sizeof(WOW64_AFD_ACCEPT64) == 16);
C_ASSERT(sizeof(WOW64_AFD_TDIH32) == 8);    C_ASSERT(sizeof(WOW64_AFD_TDIH64) == 16);
/* AFD_POLL_INFO is split into header + element because the element stride
 * changes too. The audit's 32 -> 40 are whole-struct sizes WITH the declared
 * Handles[1] and rounded to the struct's 8-byte alignment: 16+12 rounds to 32,
 * 24+16 is already 40. Handles[i] therefore lives at header + stride*i, and the
 * trailing pad the 32-bit size carries is not addressable. */
C_ASSERT(sizeof(WOW64_AFD_POLLHDR32) == 16);
C_ASSERT(sizeof(WOW64_AFD_POLLHDR64) == 24);

/* Guest-supplied counts size a host allocation, so they are bounded. The caps
 * are far above any real use (winsock's FD_SETSIZE is 64; scatter/gather sends
 * run to a handful of buffers) and refusing loudly beats trusting a count that
 * a buggy guest computed. */
#define WOW64_AFD_MAX_BUFFERS 1024
#define WOW64_AFD_MAX_HANDLES 1024

PVOID   WINAPI RtlAllocateHeap(PVOID HeapHandle, ULONG Flags, SIZE_T Size);
BOOLEAN WINAPI RtlFreeHeap(PVOID HeapHandle, ULONG Flags, PVOID BaseAddress);

static PVOID Wow64pIoctlAlloc(SIZE_T Size)
{
    return RtlAllocateHeap(Wow64pProcessHeap(), 0x00000008 /* ZERO_MEMORY */, Size);
}

static VOID Wow64pIoctlFree(PVOID P)
{
    if (P)
        RtlFreeHeap(Wow64pProcessHeap(), 0, P);
}

/* ---- M19: LPC — the PORT_MESSAGE header ---------------------------------- *
 * PORT_MESSAGE is 24 bytes to a 32-bit guest and 40 to the host: CLIENT_ID
 * carries two pointers (8 vs 16) and ClientViewSize is a SIZE_T (4 vs 8).
 * Measured, not derived -- i386 24 {ClientId@8 MessageId@16 ViewSize@20},
 * amd64 40 {ClientId@8 MessageId@24 ViewSize@32}.
 *
 * That divergence alone fails EVERY call. ntoskrnl checks
 * `DataLength + sizeof(PORT_MESSAGE) > TotalLength` (lpc-rust/send.rs:413)
 * before it even looks at the handle, and a guest sets TotalLength to
 * DataLength + 24 while the host kernel wants + 40 -- so the test is true every
 * time and the syscall returns STATUS_INVALID_PARAMETER without touching a port.
 * That is the c000000d this lane has been staring at.
 *
 * ndk's USE_LPC6432 looks purpose-built to make the two agree and does NOT:
 * measured, it takes i386 to 32, not 40, because the i386 ABI aligns a 64-bit
 * field to 4 where amd64 aligns to 8. There is no way around marshalling it.
 *
 * NOTE the payload is a SEPARATE problem, deliberately not solved here. The body
 * after the header is a per-server union discriminated by CSR_API_MESSAGE's
 * ApiNumber (Data sits at 40 on i386 and 64 on the host, and is ULONG_PTR[39],
 * so it is 156 bytes of guest data against 312 of host). Widening that blindly
 * would corrupt every message. This rung fixes the HEADER so calls stop failing
 * before they start, and LOGS ApiNumber so the APIs a guest actually uses can be
 * counted instead of guessed at -- ~100 message layouts exist; the set a guest
 * touches is much smaller. */
#include <pshpack4.h>
typedef struct {
    union { struct { SHORT DataLength, TotalLength; } s1; ULONG Length;   } u1;
    union { struct { SHORT Type, DataInfoOffset;   } s2; ULONG ZeroInit;  } u2;
    ULONG  ClientIdProcess, ClientIdThread;
    ULONG  MessageId;
    ULONG  ClientViewSize;
} WOW64_PORT_MESSAGE32;
#include <poppack.h>
C_ASSERT(sizeof(WOW64_PORT_MESSAGE32) == 24);

typedef struct {
    union { struct { SHORT DataLength, TotalLength; } s1; ULONG Length;   } u1;
    union { struct { SHORT Type, DataInfoOffset;   } s2; ULONG ZeroInit;  } u2;
    ULONGLONG ClientIdProcess, ClientIdThread;
    ULONG     MessageId;
    ULONG     __pad0;
    ULONGLONG ClientViewSize;
} WOW64_PORT_MESSAGE64;
C_ASSERT(sizeof(WOW64_PORT_MESSAGE64) == 40);

/* CSR_API_MESSAGE's ApiNumber sits right after the header on both sides
 * (guest +4 past the 24-byte header, host +8 past the 40-byte one); it encodes
 * the server index and the API within it. Only read for the census below. */
#define WOW64_CSR_APINUMBER_OFF32 (24 + 4)

static VOID Wow64pLpcHeader32To64(const WOW64_PORT_MESSAGE32 *In32,
                                  WOW64_PORT_MESSAGE64 *Out64, ULONG DataLength)
{
    Out64->u1.s1.DataLength     = In32->u1.s1.DataLength;
    Out64->u1.s1.TotalLength    = (SHORT)(sizeof(WOW64_PORT_MESSAGE64) + DataLength);
    Out64->u2.s2.Type           = In32->u2.s2.Type;
    Out64->u2.s2.DataInfoOffset = In32->u2.s2.DataInfoOffset;
    Out64->ClientIdProcess      = In32->ClientIdProcess;
    Out64->ClientIdThread       = In32->ClientIdThread;
    Out64->MessageId            = In32->MessageId;
    Out64->__pad0               = 0;
    Out64->ClientViewSize       = In32->ClientViewSize;
}

static VOID Wow64pLpcHeader64To32(const WOW64_PORT_MESSAGE64 *In64,
                                  WOW64_PORT_MESSAGE32 *Out32, ULONG DataLength)
{
    Out32->u1.s1.DataLength     = In64->u1.s1.DataLength;
    Out32->u1.s1.TotalLength    = (SHORT)(sizeof(WOW64_PORT_MESSAGE32) + DataLength);
    Out32->u2.s2.Type           = In64->u2.s2.Type;
    Out32->u2.s2.DataInfoOffset = In64->u2.s2.DataInfoOffset;
    Out32->ClientIdProcess      = (ULONG)In64->ClientIdProcess;
    Out32->ClientIdThread       = (ULONG)In64->ClientIdThread;
    Out32->MessageId            = In64->MessageId;
    Out32->ClientViewSize       = (ULONG)In64->ClientViewSize;
}

/* A guest's own ceiling is PORT_MAXIMUM_MESSAGE_LENGTH, which is itself
 * divergent: 256 for a 32-bit caller, 512 for the host. A guest message
 * therefore always fits after widening (256 + 16 of header growth = 272 < 512),
 * and anything past the guest ceiling is a bad message, not a big one --
 * refuse it loudly rather than truncate into a host allocation. */
#define WOW64_LPC_MAX_DATA32 256

/* Byte copy in the shape wow64_automarshal.c's WowAmCpy already uses and links
 * with in this module -- the marshallers here move small, possibly unaligned
 * guest payloads, and this keeps the dependency surface identical to the
 * generated sibling rather than introducing a new CRT import. */
static void Wow64pCpy(unsigned char *d, const unsigned char *s, unsigned n)
{
    while (n--) *d++ = *s++;
}

static NTSTATUS Wow64pLpcBadArg(const char *Who, const char *Why)
{
    DbgPrint("[WOW64] %s REFUSED: %s\n", Who, Why);
    return WOW64_STATUS_INVALID_PARAMETER;
}

/* The CSR_API_MESSAGE body does NOT survive a verbatim copy. Its first member
 * is PCSR_CAPTURE_BUFFER -- a pointer -- so everything after it shifts by four:
 * body-relative, a guest has ApiNumber@4 Status@8 Reserved@12 Data@16 while the
 * host has 8 / 12 / 16 / 24. Copying the body straight through therefore feeds
 * csrss an ApiNumber read from the wrong half of a pointer, and reads the reply
 * Status from the wrong word. Marshal the prologue explicitly; it is identical
 * for EVERY api, so this part is universal rather than per-API. */
#define WOW64_CSRMSG_BODY32_APINUMBER   4
#define WOW64_CSRMSG_BODY32_STATUS      8
#define WOW64_CSRMSG_BODY32_RESERVED   12
#define WOW64_CSRMSG_BODY32_DATA       16
#define WOW64_CSRMSG_BODY64_APINUMBER   8
#define WOW64_CSRMSG_BODY64_STATUS     12
#define WOW64_CSRMSG_BODY64_RESERVED   16
#define WOW64_CSRMSG_BODY64_DATA       24

/* sizeof(BASE_CREATE_PROCESS) per arch, measured with offsetof/sizeof against
 * the real basemsg.h and cross-checked against the size_check_*.rs asserts the
 * tree already carries (140 i386 / 232 amd64). */
#define WOW64_BCP32_SIZE               140
#define WOW64_BCP64_SIZE               232

/* Narrow one field of the BASE_CREATE_PROCESS reply back to the guest. */
#define WOW64_BCP_NARROW(o32, o64)                                             \
            Wow64pCsrPut32(R32, WOW64_CSRMSG_BODY32_DATA + (o32),              \
                        Wow64pCsrGet32(R64, WOW64_CSRMSG_BODY64_DATA + (o64)))

static void Wow64pZeroBytes(unsigned char *d, unsigned n)
{
    while (n--)
        *d++ = 0;
}

static ULONG Wow64pCsrGet32(const unsigned char *P, ULONG Off)
{
    ULONG V; Wow64pCpy((unsigned char *)&V, P + Off, 4); return V;
}
static VOID Wow64pCsrPut32(unsigned char *P, ULONG Off, ULONG V)
{
    Wow64pCpy(P + Off, (const unsigned char *)&V, 4);
}
static VOID Wow64pCsrPut64(unsigned char *P, ULONG Off, ULONGLONG V)
{
    Wow64pCpy(P + Off, (const unsigned char *)&V, 8);
}

/* CSR_CAPTURE_BUFFER, measured both ways: the header is 16 bytes to a guest and
 * 32 to the host (PreviousCaptureBuffer and BufferEnd are pointers, PointerCount
 * gets padded), and every PointerOffsetsArray entry is a ULONG_PTR -- 4 bytes
 * against 8. csrss reads this structure out of the CLIENT's address space with
 * host field offsets, so a guest-built one fails CsrValidateMessageBuffer's
 * bounds arithmetic and the api returns c000000d. wow64.dll runs INSIDE the
 * guest process, so a host-layout copy allocated from our own heap lands in the
 * very address space csrss reads from -- no cross-process work needed.
 *
 * The entries are message-relative offsets of pointer FIELDS, so they have to be
 * restated in host coordinates too. That mapping is per-api in general; only the
 * one api a guest actually sends is mapped here, and anything else is refused
 * loudly rather than relocated to a guessed offset. */
#define WOW64_CAPBUF32_HDR            16
#define WOW64_CAPBUF64_HDR            32
#define WOW64_CAPBUF32_SIZE            0
#define WOW64_CAPBUF32_POINTERCOUNT    8
#define WOW64_CAPBUF64_SIZE            0
#define WOW64_CAPBUF64_PREV            8
#define WOW64_CAPBUF64_POINTERCOUNT   16
#define WOW64_CAPBUF64_BUFFEREND      24

/* CSR_CLIENT_CONNECT's ConnectionInfo pointer, message-relative:
 *   guest 24(hdr) + 16(Data) + 4(ServerId)      = 44
 *   host  40(hdr) + 24(Data) + 8(ServerId+pad)  = 72  */
#define WOW64_CSR_CLIENTCONNECT_PTR32 44
#define WOW64_CSR_CLIENTCONNECT_PTR64 72

static ULONG Wow64pCsrMapMsgOffset(ULONG Api, ULONG Off32, const char **Why)
{
    if (Api == 0 && Off32 == WOW64_CSR_CLIENTCONNECT_PTR32)
        return WOW64_CSR_CLIENTCONNECT_PTR64;
    *Why = "no host offset known for this api's pointer slot";
    return 0;
}

/* Build a host-layout capture buffer from the guest's. Returns the host buffer
 * (caller frees) or NULL with *Why set. *InfoOut receives the relocated value
 * for the api's data pointer. */
static unsigned char *Wow64pCsrWidenCaptureBuffer(ULONG Api, ULONG Cap32,
                                                  ULONG Info32,
                                                  ULONGLONG *InfoOut,
                                                  ULONG *Off64Out,
                                                  unsigned char **BackTo,
                                                  unsigned char **BackFrom,
                                                  ULONG *BackLen,
                                                  const char **Why)
{
    const unsigned char *C32;
    unsigned char *C64;
    ULONG_PTR Local;
    ULONG Size32, Count, Data32Off, DataLen, Total64, i;

    if (!Wow64pCsrScratchSize)
    {
        *Why = "no scratch reserved in the CSR port view";
        return NULL;
    }

    /* Cap32 is a SERVER address. Bring it back into our own view before any
     * dereference, and refuse anything that does not land inside the view the
     * guest actually shares with csrss. */
    Local = (ULONG_PTR)Cap32 - Wow64pCsrPortDelta;
    if (Local < Wow64pCsrViewLocal ||
        Local >= Wow64pCsrViewLocal + Wow64pCsrViewSize)
    {
        DbgPrint("[WOW64] LPC: capture buffer %08lx (local %p) is outside the port view\n",
                 Cap32, (PVOID)Local);
        *Why = "capture buffer is outside the CSR port view";
        return NULL;
    }
    C32 = (const unsigned char *)Local;

    Size32 = Wow64pCsrGet32(C32, WOW64_CAPBUF32_SIZE);
    Count  = Wow64pCsrGet32(C32, WOW64_CAPBUF32_POINTERCOUNT);

    /* Guest-supplied sizes bound a host write: refuse absurd ones loudly rather
     * than trusting a count a broken guest computed. */
    if (Count > 64 || Size32 < WOW64_CAPBUF32_HDR + Count * 4 || Size32 > 0x10000 ||
        Local + Size32 > Wow64pCsrViewLocal + Wow64pCsrViewSize)
    {
        *Why = "capture buffer Size/PointerCount out of range";
        return NULL;
    }

    Data32Off = WOW64_CAPBUF32_HDR + Count * 4;
    DataLen   = Size32 - Data32Off;
    Total64   = WOW64_CAPBUF64_HDR + Count * 8 + DataLen;

    if (Total64 > Wow64pCsrScratchSize)
    {
        *Why = "widened capture buffer does not fit the port-view scratch";
        return NULL;
    }

    if (Info32 < Cap32 + Data32Off || Info32 >= Cap32 + Size32)
    {
        *Why = "api data pointer is outside the capture buffer";
        return NULL;
    }

    /* The widened buffer goes in the view tail we reserved at connect time, so
     * csrss can reach it at its own address. */
    C64 = (unsigned char *)Wow64pCsrScratchLocal;

    Wow64pCsrPut32(C64, WOW64_CAPBUF64_SIZE, Total64);
    Wow64pCsrPut64(C64, WOW64_CAPBUF64_PREV, 0);
    Wow64pCsrPut32(C64, WOW64_CAPBUF64_POINTERCOUNT, Count);
    Wow64pCsrPut64(C64, WOW64_CAPBUF64_BUFFEREND,
                (ULONGLONG)(Wow64pCsrScratchLocal + Wow64pCsrPortDelta + Total64));

    for (i = 0; i < Count; i++)
    {
        ULONG Off32 = Wow64pCsrGet32(C32, WOW64_CAPBUF32_HDR + i * 4);
        ULONG Off64;
        if (Off32 == 0)
        {
            Wow64pCsrPut64(C64, WOW64_CAPBUF64_HDR + i * 8, 0);
            continue;
        }
        Off64 = Wow64pCsrMapMsgOffset(Api, Off32, Why);
        if (Off64 == 0)
            return NULL;
        Wow64pCsrPut64(C64, WOW64_CAPBUF64_HDR + i * 8, Off64);
        *Off64Out = Off64;
    }

    Wow64pCpy(C64 + WOW64_CAPBUF64_HDR + Count * 8, C32 + Data32Off, DataLen);

    /* csrss writes its answer INTO the buffer, so the reply path has to copy
     * the data area back where the guest will look for it. */
    *BackTo   = (unsigned char *)(Local + Data32Off);
    *BackFrom = C64 + WOW64_CAPBUF64_HDR + Count * 8;
    *BackLen  = DataLen;

    /* Every pointer the message hands csrss is a server address too. */
    *InfoOut = (ULONGLONG)(Wow64pCsrScratchLocal + Wow64pCsrPortDelta
                           + WOW64_CAPBUF64_HDR + Count * 8
                           + (Info32 - (Cap32 + Data32Off)));
    return C64;
}

/* NtRequestWaitReplyPort(PortHandle, LpcRequest, LpcReply) — the call every CSR
 * API rides on. Header-only for now; see the block comment above on why the
 * payload is a separate job. */
static NTSTATUS Wow64Manual_NtRequestWaitReplyPort(ULONG *a)
{
    extern NTSTATUS WINAPI NtRequestWaitReplyPort(HANDLE, PVOID, PVOID);
    const WOW64_PORT_MESSAGE32 *Req32 = (const WOW64_PORT_MESSAGE32 *)(ULONG_PTR)a[1];
    WOW64_PORT_MESSAGE32 *Rep32 = (WOW64_PORT_MESSAGE32 *)(ULONG_PTR)a[2];
    HANDLE Port = (HANDLE)(LONG_PTR)(LONG)a[0];
    WOW64_PORT_MESSAGE64 *Req64 = NULL, *Rep64 = NULL;
    unsigned char *Cap64 = NULL;
    unsigned char *CapBackTo = NULL, *CapBackFrom = NULL;
    ULONG CapBackLen = 0;
    ULONG Cap32Orig = 0, Info32Orig = 0;
    int IsCreateThread = 0;
    int IsCreateProcess = 0;
    ULONG DataLength, RepData;
    NTSTATUS St;


    if (!Req32 || !Rep32)
        return Wow64pLpcBadArg("NtRequestWaitReplyPort", "NULL request or reply buffer");

    /* DataLength is a CSHORT: a negative value would widen into a huge ULONG. */
    if (Req32->u1.s1.DataLength < 0 || Req32->u1.s1.TotalLength < 0)
        return Wow64pLpcBadArg("NtRequestWaitReplyPort", "negative DataLength/TotalLength");

    DataLength = (ULONG)Req32->u1.s1.DataLength;
    if (DataLength > WOW64_LPC_MAX_DATA32)
        return Wow64pLpcBadArg("NtRequestWaitReplyPort", "DataLength past the guest's own ceiling");

    /* The guest's own TotalLength must at least cover its header + data, or the
     * message was malformed before we ever touched it. */
    if ((ULONG)Req32->u1.s1.TotalLength < sizeof(WOW64_PORT_MESSAGE32) + DataLength)
        return Wow64pLpcBadArg("NtRequestWaitReplyPort", "TotalLength shorter than header + DataLength");

    Req64 = (WOW64_PORT_MESSAGE64 *)Wow64pIoctlAlloc(sizeof(*Req64) + WOW64_LPC_MAX_DATA32);
    Rep64 = (WOW64_PORT_MESSAGE64 *)Wow64pIoctlAlloc(sizeof(*Rep64) + WOW64_LPC_MAX_DATA32);
    if (!Req64 || !Rep64)
    {
        Wow64pIoctlFree(Req64);
        Wow64pIoctlFree(Rep64);
        return WOW64_STATUS_NO_MEMORY;
    }

    {
        const unsigned char *B32 = (const unsigned char *)Req32 + sizeof(*Req32);
        unsigned char *B64 = (unsigned char *)Req64 + sizeof(*Req64);
        ULONG Api = Wow64pCsrGet32(B32, WOW64_CSRMSG_BODY32_APINUMBER);
        ULONG Data32Len = (DataLength > WOW64_CSRMSG_BODY32_DATA)
                        ? DataLength - WOW64_CSRMSG_BODY32_DATA : 0;

        /* Prologue, explicit and universal across every api. */
        Wow64pCsrPut64(B64, 0, (ULONGLONG)Wow64pCsrGet32(B32, 0)); /* CsrCaptureData */
        Wow64pCsrPut32(B64, WOW64_CSRMSG_BODY64_APINUMBER, Api);
        Wow64pCsrPut32(B64, WOW64_CSRMSG_BODY64_STATUS,
                    Wow64pCsrGet32(B32, WOW64_CSRMSG_BODY32_STATUS));
        Wow64pCsrPut32(B64, WOW64_CSRMSG_BODY64_RESERVED,
                    Wow64pCsrGet32(B32, WOW64_CSRMSG_BODY32_RESERVED));

        /* The per-api Data union is NOT width-safe in general. Measured on a
         * real boot, the only api a guest sends here is 0 (CsrSrvClientConnect),
         * whose CSR_CLIENT_CONNECT is {ULONG ServerId, PVOID ConnectionInfo,
         * ULONG ConnectionInfoSize} -- one pointer to widen. Anything else is
         * copied verbatim AND named on serial, so a new api shows up as a loud
         * line rather than as silent corruption. */
        if (Api == 0 && Data32Len >= 12)
        {
            ULONG Cap32  = Wow64pCsrGet32(B32, 0);
            Cap32Orig = Cap32;
            ULONG Info32 = Wow64pCsrGet32(B32, WOW64_CSRMSG_BODY32_DATA + 4);

            ULONGLONG Info64 = 0;
            ULONG Off64Unused = 0;
            const char *Why = "unknown";

            Wow64pCsrPut32(B64, WOW64_CSRMSG_BODY64_DATA + 0,
                        Wow64pCsrGet32(B32, WOW64_CSRMSG_BODY32_DATA + 0));   /* ServerId */
            Wow64pCsrPut32(B64, WOW64_CSRMSG_BODY64_DATA + 16,
                        Wow64pCsrGet32(B32, WOW64_CSRMSG_BODY32_DATA + 8));   /* ConnectionInfoSize */
            Req64->u1.s1.DataLength = (SHORT)(WOW64_CSRMSG_BODY64_DATA + 20);

            /* csrss reads the capture buffer out of THIS process with host field
             * offsets, so the guest's 16-byte-header copy fails its bounds check
             * and the api answers c000000d. Hand it a host-layout one. */
            if (Cap32)
            {
                Info32Orig = Info32;
                Cap64 = Wow64pCsrWidenCaptureBuffer(Api, Cap32, Info32,
                                                    &Info64, &Off64Unused,
                                                    &CapBackTo, &CapBackFrom,
                                                    &CapBackLen, &Why);
                if (!Cap64)
                {
                    Wow64pIoctlFree(Req64);
                    Wow64pIoctlFree(Rep64);
                    return Wow64pLpcBadArg("NtRequestWaitReplyPort", Why);
                }
                /* csrss dereferences this in ITS address space. */
                Wow64pCsrPut64(B64, 0,
                            (ULONGLONG)((ULONG_PTR)Cap64 + Wow64pCsrPortDelta));
                Wow64pCsrPut64(B64, WOW64_CSRMSG_BODY64_DATA + 8, Info64);
            }
            else
            {
                Wow64pCsrPut64(B64, WOW64_CSRMSG_BODY64_DATA + 8,
                            (ULONGLONG)Info32);
            }
        }
        else if (Api == WOW64_CSRAPI_BASEP_CREATEPROCESS &&
                 Data32Len >= WOW64_BCP32_SIZE)
        {
            /* BASE_CREATE_PROCESS -- the message CreateProcess sends basesrv
             * once the child and its initial thread exist. It opens with
             * HANDLE, HANDLE, CLIENT_ID, i.e. FOUR pointer-width fields in a
             * row, so the verbatim copy handed basesrv a ProcessHandle and a
             * ClientId assembled from the wrong words and it answered
             * c0000008; the guest reported CreateProcessW err 0x6
             * (ERROR_INVALID_HANDLE). Identical failure to BASEP_CREATETHREAD
             * below, one struct larger.
             *
             * Offsets MEASURED, not derived -- offsetof() dumps from the real
             * basemsg.h structs built with the llvm-mingw i686 and x86_64
             * targets. The control on that measurement is sizeof: 140/96 on
             * i386 and 232/160 on amd64, which is exactly what the tree
             * already asserts in ntdll-sys/src/size_check_{i386,amd64}.rs, so
             * a bad transcription would have shown up as a size mismatch
             * rather than as quietly wrong offsets.
             *
             *   field                  i386    amd64
             *   ProcessHandle          0x00    0x00
             *   ThreadHandle           0x04    0x08
             *   ClientId.UniqueProcess 0x08    0x10
             *   ClientId.UniqueThread  0x0c    0x18
             *   CreationFlags          0x10    0x20
             *   VdmBinaryType          0x14    0x24
             *   VdmTask                0x18    0x28
             *   hVDM                   0x1c    0x30
             *   Sxs                    0x20    0x38
             *   PebAddressNative       0x80    0xd8
             *   PebAddressWow64        0x84    0xe0
             *   ProcessorArchitecture  0x88    0xe4
             *   sizeof                 0x8c    0xe8
             *
             * ProcessHandle is NOT purely a handle: kernel32 packs up to two
             * flag bits into its low bits (AddToHandle/RemoveFromHandle) and
             * BaseSrvCreateProcess reads them back out as `Handle & 3` before
             * masking. Zero-extension preserves them, which is the only
             * reason the naive widening is right here.
             *
             * Sxs is the one field NOT marshalled. BaseSrvCreateProcess never
             * reads it -- the SxS work belongs to a different API
             * (BaseSrvSxsCreateActivationContext) -- so widening thirty fields
             * including four UNICODE_STRINGs whose Buffers are guest pointers
             * would buy nothing today. It is ZEROED rather than left as
             * whatever the verbatim copy put there, and the guest is told when
             * it mattered, so the day the server starts reading it this
             * announces itself instead of going quiet. */
            IsCreateProcess = 1;

            /* Zero first: every byte the server might read is then either a
             * field written below or a deliberate zero, never leftover. */
            Wow64pZeroBytes(B64 + WOW64_CSRMSG_BODY64_DATA, WOW64_BCP64_SIZE);

#define WOW64_BCP_WIDEN(o32, o64)                                              \
            Wow64pCsrPut64(B64, WOW64_CSRMSG_BODY64_DATA + (o64),              \
                        (ULONGLONG)Wow64pCsrGet32(B32,                         \
                                    WOW64_CSRMSG_BODY32_DATA + (o32)))
#define WOW64_BCP_COPY32(o32, o64)                                             \
            Wow64pCsrPut32(B64, WOW64_CSRMSG_BODY64_DATA + (o64),              \
                        Wow64pCsrGet32(B32, WOW64_CSRMSG_BODY32_DATA + (o32)))

            WOW64_BCP_WIDEN (0x00, 0x00);   /* ProcessHandle (+ flag bits)   */
            WOW64_BCP_WIDEN (0x04, 0x08);   /* ThreadHandle                  */
            WOW64_BCP_WIDEN (0x08, 0x10);   /* ClientId.UniqueProcess        */
            WOW64_BCP_WIDEN (0x0c, 0x18);   /* ClientId.UniqueThread         */
            WOW64_BCP_COPY32(0x10, 0x20);   /* CreationFlags        (ULONG)  */
            WOW64_BCP_COPY32(0x14, 0x24);   /* VdmBinaryType        (ULONG)  */
            WOW64_BCP_COPY32(0x18, 0x28);   /* VdmTask              (ULONG)  */
            WOW64_BCP_WIDEN (0x1c, 0x30);   /* hVDM                          */
            WOW64_BCP_WIDEN (0x80, 0xd8);   /* PebAddressNative              */
            WOW64_BCP_COPY32(0x84, 0xe0);   /* PebAddressWow64      (ULONG)  */
            WOW64_BCP_COPY32(0x88, 0xe4);   /* ProcessorArchitecture(USHORT) */

            if (Wow64pCsrGet32(B32, WOW64_CSRMSG_BODY32_DATA + 0x20) != 0)
                DbgPrint("[WOW64] LPC BasepCreateProcess: Sxs.Flags=%08lx is "
                         "NOT marshalled (basesrv ignores Sxs today) -- if this "
                         "starts mattering, widen BASE_SXS_CREATEPROCESS_MSG\n",
                         Wow64pCsrGet32(B32, WOW64_CSRMSG_BODY32_DATA + 0x20));

            Req64->u1.s1.DataLength = (SHORT)(WOW64_CSRMSG_BODY64_DATA +
                                              WOW64_BCP64_SIZE);
        }
        else if (Api == WOW64_CSRAPI_BASEP_CREATETHREAD && Data32Len >= 12)
        {
            /* BASE_CREATE_THREAD { HANDLE ThreadHandle; CLIENT_ID ClientId; } --
             * three pointer-width fields in a row, so the verbatim copy handed
             * csrss a ClientId built from the wrong words and it answered
             * c0000001; the guest reported CreateThread err 0x1f. MEASURED:
             *   i386   ThreadHandle=0 UniqueProcess=4  UniqueThread=8  sizeof=12
             *   amd64  ThreadHandle=0 UniqueProcess=8  UniqueThread=16 sizeof=24
             * These are handle/id VALUES, not guest pointers, so zero-extending
             * is the whole conversion. */
            IsCreateThread = 1;
            Wow64pCsrPut64(B64, WOW64_CSRMSG_BODY64_DATA + 0,
                        (ULONGLONG)Wow64pCsrGet32(B32, WOW64_CSRMSG_BODY32_DATA + 0));
            Wow64pCsrPut64(B64, WOW64_CSRMSG_BODY64_DATA + 8,
                        (ULONGLONG)Wow64pCsrGet32(B32, WOW64_CSRMSG_BODY32_DATA + 4));
            Wow64pCsrPut64(B64, WOW64_CSRMSG_BODY64_DATA + 16,
                        (ULONGLONG)Wow64pCsrGet32(B32, WOW64_CSRMSG_BODY32_DATA + 8));
            Req64->u1.s1.DataLength = (SHORT)(WOW64_CSRMSG_BODY64_DATA + 24);
        }
        else
        {
            /* A verbatim body copy is CORRECT when the payload holds no
             * pointer-width field, and shouting about those trains the reader
             * to ignore the line that matters. Each entry here was read against
             * its struct, not assumed:
             *   0x00010003 BasepExitProcess            { UINT uExitCode; }
             *   0x0001000c BasepSetProcessShutdownParam{ ULONG Level, Flags; }
             * (reactos/subsys/win/basemsg.h:113, :219) */
            if (Api != 0x00010003 && Api != 0x0001000C)
                DbgPrint("[WOW64] LPC: api %08lx has NO payload marshaller — body copied verbatim, "
                         "fields past CsrCaptureData may be wrong\n", Api);
            Wow64pCpy(B64 + WOW64_CSRMSG_BODY64_DATA,
                      B32 + WOW64_CSRMSG_BODY32_DATA, Data32Len);
            Req64->u1.s1.DataLength = (SHORT)(WOW64_CSRMSG_BODY64_DATA + Data32Len);
        }
        Req64->u1.s1.TotalLength = (SHORT)(sizeof(*Req64) + Req64->u1.s1.DataLength);
    }

    /* The census this rung exists for: which CSR APIs does a guest actually
     * send? ApiNumber encodes server index + API. Counting beats guessing —
     * ~100 message layouts exist and the reachable set is far smaller. */
    if (DataLength >= 8)
    {
        const unsigned char *Bl = (const unsigned char *)Req32 + sizeof(*Req32);
        /* CsrCaptureData decides which arm of CsrValidateMessageBuffer runs:
         * NULL means "same-process only", and a cross-process client is then
         * rejected -- which is the c000000d the api is returning. Print it with
         * the connect args instead of inferring which case this is. */
        DbgPrint("[WOW64] LPC request: Api=%08lx DataLen=%lu Capture=%08lx "
                 "ServerId=%08lx Info=%08lx InfoSize=%08lx\n",
                 Wow64pCsrGet32(Bl, WOW64_CSRMSG_BODY32_APINUMBER), DataLength,
                 Wow64pCsrGet32(Bl, 0),
                 Wow64pCsrGet32(Bl, WOW64_CSRMSG_BODY32_DATA + 0),
                 Wow64pCsrGet32(Bl, WOW64_CSRMSG_BODY32_DATA + 4),
                 Wow64pCsrGet32(Bl, WOW64_CSRMSG_BODY32_DATA + 8));
    }

    St = NtRequestWaitReplyPort(Port, Req64, Rep64);

    if (St >= 0)
    {
        const unsigned char *R64 = (const unsigned char *)Rep64 + sizeof(*Rep64);
        unsigned char *R32 = (unsigned char *)Rep32 + sizeof(*Rep32);
        ULONG ApiStatus;

        RepData = (Rep64->u1.s1.DataLength < 0) ? 0 : (ULONG)Rep64->u1.s1.DataLength;
        if (RepData > WOW64_LPC_MAX_DATA32)
            RepData = WOW64_LPC_MAX_DATA32;

        /* Narrow the prologue the same way, or the guest reads its Status out of
         * the wrong word -- the transport status below is NOT the api's answer. */
        /* csrss echoes the message back, so CsrCaptureData in the reply is the
         * SCRATCH buffer we substituted, not the guest's. Handing that back
         * makes the guest compute scratch = reply - CsrPortMemoryDelta and
         * RtlFreeHeap a block its heap never owned. Give it its own pointer. */
        Wow64pCsrPut32(R32, 0, Cap32Orig ? Cap32Orig : Wow64pCsrGet32(R64, 0));
        Wow64pCsrPut32(R32, WOW64_CSRMSG_BODY32_APINUMBER,
                    Wow64pCsrGet32(R64, WOW64_CSRMSG_BODY64_APINUMBER));
        ApiStatus = Wow64pCsrGet32(R64, WOW64_CSRMSG_BODY64_STATUS);
        Wow64pCsrPut32(R32, WOW64_CSRMSG_BODY32_STATUS, ApiStatus);
        Wow64pCsrPut32(R32, WOW64_CSRMSG_BODY32_RESERVED,
                    Wow64pCsrGet32(R64, WOW64_CSRMSG_BODY64_RESERVED));
        if (RepData > WOW64_CSRMSG_BODY64_DATA)
            Wow64pCpy(R32 + WOW64_CSRMSG_BODY32_DATA,
                      R64 + WOW64_CSRMSG_BODY64_DATA,
                      RepData - WOW64_CSRMSG_BODY64_DATA);

        if (IsCreateProcess)
        {
            /* basesrv passes &ClientId to CsrCreateProcess by reference, and
             * the verbatim reply copy above laid 64-bit words into a struct the
             * guest reads with 32-bit offsets. kernel32 happens to fill
             * PROCESS_INFORMATION from its OWN local ClientId rather than from
             * the reply, so nothing reads this today -- narrow it anyway, so
             * the guest is not left holding a struct that is wrong in a way
             * only the next reader discovers. */
            WOW64_BCP_NARROW(0x00, 0x00);   /* ProcessHandle                 */
            WOW64_BCP_NARROW(0x04, 0x08);   /* ThreadHandle                  */
            WOW64_BCP_NARROW(0x08, 0x10);   /* ClientId.UniqueProcess        */
            WOW64_BCP_NARROW(0x0c, 0x18);   /* ClientId.UniqueThread         */
            WOW64_BCP_NARROW(0x10, 0x20);   /* CreationFlags                 */
            WOW64_BCP_NARROW(0x14, 0x24);   /* VdmBinaryType                 */
            WOW64_BCP_NARROW(0x18, 0x28);   /* VdmTask                       */
            WOW64_BCP_NARROW(0x1c, 0x30);   /* hVDM                          */
        }

        if (IsCreateThread)
        {
            Wow64pCsrPut32(R32, WOW64_CSRMSG_BODY32_DATA + 0,
                        Wow64pCsrGet32(R64, WOW64_CSRMSG_BODY64_DATA + 0));
            Wow64pCsrPut32(R32, WOW64_CSRMSG_BODY32_DATA + 4,
                        Wow64pCsrGet32(R64, WOW64_CSRMSG_BODY64_DATA + 8));
            Wow64pCsrPut32(R32, WOW64_CSRMSG_BODY32_DATA + 8,
                        Wow64pCsrGet32(R64, WOW64_CSRMSG_BODY64_DATA + 16));
        }

        if (Cap32Orig)
        {
            /* The Data union is NOT width-safe, so the verbatim copy above put
             * the 64-bit padding word where the guest reads ConnectionInfo --
             * it then computed NULL - CsrPortMemoryDelta and faulted reading
             * there. Narrow the three fields by hand, the way the request did,
             * and give the guest back its OWN pointers rather than our scratch. */
            Wow64pCsrPut32(R32, WOW64_CSRMSG_BODY32_DATA + 0,
                        Wow64pCsrGet32(R64, WOW64_CSRMSG_BODY64_DATA + 0)); /* ServerId */
            Wow64pCsrPut32(R32, WOW64_CSRMSG_BODY32_DATA + 4, Info32Orig);
            Wow64pCsrPut32(R32, WOW64_CSRMSG_BODY32_DATA + 8,
                        Wow64pCsrGet32(R64, WOW64_CSRMSG_BODY64_DATA + 16)); /* InfoSize */

            if (CapBackLen)
                Wow64pCpy(CapBackTo, CapBackFrom, CapBackLen);
        }

        Wow64pLpcHeader64To32(Rep64, Rep32,
                              WOW64_CSRMSG_BODY32_DATA +
                              ((RepData > WOW64_CSRMSG_BODY64_DATA)
                                   ? RepData - WOW64_CSRMSG_BODY64_DATA : 0));

        /* The api's OWN result. A green transport status over a rejected api is
         * exactly the false-green this rung must not report as working. */
        DbgPrint("[WOW64] LPC reply: api Status=%08lx\n", ApiStatus);
    }

    DbgPrint("[WOW64] NtRequestWaitReplyPort => %08lx\n", St);
    /* Cap64 lives in the port-view scratch, not the ioctl heap — nothing to free. */
    Wow64pIoctlFree(Req64);
    Wow64pIoctlFree(Rep64);
    return St;
}

static NTSTATUS Wow64pIoctlBadArg(const char *Who, const char *Why)
{
    DbgPrint("[WOW64]   %s: malformed 32-bit IOCTL payload: %s\n", Who, Why);
    return WOW64_STATUS_INVALID_PARAMETER;
}

typedef NTSTATUS (WINAPI *WOW64_IOCTL_FN)(HANDLE, HANDLE, PVOID, PVOID, PVOID,
                                          ULONG, PVOID, ULONG, PVOID, ULONG);

/* Issue a MARSHALLED IOCTL and do not come back until the driver is done.
 *
 * M15c pairs with the kernel-side IOSB32 work below: a plain (unmarshalled)
 * IOCTL now latches the guest's own 8-byte IOSB and may pend for real. A
 * marshalled one cannot, and the reason is the PAYLOAD, not the IOSB — the
 * widened input, the WSABUF array and the AFD_POLL_INFO shadow are all
 * host-heap blocks this frame frees on the way out, and for METHOD_BUFFERED
 * the kernel copies its SystemBuffer back into that shadow at completion
 * time. Returning while the request is live would hand the kernel a freed
 * pointer — the same dead-frame class as the old stack IOSB, just with a
 * bigger blast radius. So we block, and only then copy back and free.
 *
 * Blocking is invisible to the caller that matters: msafd's own idiom is
 * "issue, and if STATUS_PENDING immediately MsafdWaitForAlert(SockEvent)".
 * But its SockEvent is a SYNCHRONIZATION (auto-reset) event, so OUR wait
 * consumes the very signal that wait is about to look for — hence the
 * re-signal. Getting that wrong is a guest hang, not a visible error. */
static NTSTATUS Wow64pIoctlCall(WOW64_IOCTL_FN Fn, HANDLE File, HANDLE Event,
                                WOW64_IO_STATUS_BLOCK64_M *I64, ULONG Code,
                                PVOID In, ULONG InLen, PVOID Out, ULONG OutLen,
                                const char *Who)
{
    NTSTATUS St = Fn(File, Event, NULL, NULL, I64, Code, In, InLen, Out, OutLen);
    NTSTATUS Wait;

    if (St != WOW64_STATUS_PENDING)
        return St;

    Wait = NtWaitForSingleObject(Event ? Event : File, FALSE, NULL);
    if (Wait < 0)
    {
        /* The shadow is about to be freed under a request the driver still
         * owns. Nothing here can make that safe, so say so rather than free
         * quietly and corrupt later. */
        DbgPrint("[WOW64]   %s: wait on a pended marshalled IOCTL failed %08lx — "
                 "the driver may still own our shadow buffer\n", Who, Wait);
        return Wait;
    }
    if (Event)
        NtSetEvent(Event, NULL);
    return (NTSTATUS)(ULONG)I64->Status;
}

/* The AFD_WSABUF array every transfer IOCTL points at: {len, ptr} per element,
 * 8 bytes wide in the guest and 16 here. IN only — AFD reads it to find the
 * guest's data buffers, and reports what it moved through the IOSB. */
static WOW64_AFD_WSABUF64 *Wow64pAfdWidenBufs(ULONG Buf32Ptr, ULONG Count,
                                              const char *Who, NTSTATUS *St)
{
    const WOW64_AFD_WSABUF32 *B32 = (const WOW64_AFD_WSABUF32 *)(ULONG_PTR)Buf32Ptr;
    WOW64_AFD_WSABUF64 *B64;
    ULONG i;

    *St = 0;
    if (Count == 0)
        return NULL;                    /* legal: a zero-buffer transfer */
    if (Count > WOW64_AFD_MAX_BUFFERS)
    {
        *St = Wow64pIoctlBadArg(Who, "BufferCount above the marshalling cap");
        return NULL;
    }
    if (!B32)
    {
        *St = Wow64pIoctlBadArg(Who, "non-zero BufferCount with a NULL BufferArray");
        return NULL;
    }
    B64 = (WOW64_AFD_WSABUF64 *)Wow64pIoctlAlloc((SIZE_T)Count * sizeof(*B64));
    if (!B64)
    {
        *St = WOW64_STATUS_NO_MEMORY;
        return NULL;
    }
    for (i = 0; i < Count; i++)
    {
        B64[i].len = B32[i].len;
        B64[i].buf = (ULONGLONG)(ULONG_PTR)B32[i].buf;   /* guest VA == host VA */
    }
    return B64;
}

/* IOCTL_AFD_SEND / IOCTL_AFD_RECV — AFD_SEND_INFO / AFD_RECV_INFO, 16 -> 24.
 *
 * The shadow is sized to the LARGEST payload this code can be read as, not to
 * AFD_SEND_INFO. AfdConnectedSocketWriteData re-reads its buffer as the 104-byte
 * AFD_SEND_INFO_UDP when the endpoint is connectionless, so a 24-byte shadow
 * would be over-read exactly as the guest's 16-byte buffer is today. Zeroed and
 * oversized, that read stays in bounds. (The connectionless case still lands on
 * the SEND_DATAGRAM refusal below for its own payload; this only keeps the
 * overload from reading off the end.) */
static NTSTATUS Wow64pIoctlAfdXfer(WOW64_IOCTL_FN Fn, ULONG *a, const char *Who)
{
    const WOW64_AFD_XFER32 *In32 = (const WOW64_AFD_XFER32 *)(ULONG_PTR)a[6];
    WOW64_IO_STATUS_BLOCK64_M Iosb64 = {0};
    WOW64_AFD_WSABUF64 *B64;
    WOW64_AFD_XFER64 *In64;
    HANDLE File = (HANDLE)(LONG_PTR)(LONG)a[0];
    HANDLE Event = (HANDLE)(LONG_PTR)(LONG)a[1];
    NTSTATUS St;

    if (!In32 || a[7] < sizeof(*In32))
        return Wow64pIoctlBadArg(Who, "input shorter than AFD_SEND_INFO/AFD_RECV_INFO");

    B64 = Wow64pAfdWidenBufs(In32->BufferArray, In32->BufferCount, Who, &St);
    if (St)
        return St;

    In64 = (WOW64_AFD_XFER64 *)Wow64pIoctlAlloc(104);   /* AFD_SEND_INFO_UDP */
    if (!In64)
    {
        Wow64pIoctlFree(B64);
        return WOW64_STATUS_NO_MEMORY;
    }
    In64->BufferArray = (ULONGLONG)(ULONG_PTR)B64;
    In64->BufferCount = In32->BufferCount;
    In64->AfdFlags    = In32->AfdFlags;
    In64->TdiFlags    = In32->TdiFlags;

    St = Wow64pIoctlCall(Fn, File, Event, &Iosb64, a[5], In64, sizeof(WOW64_AFD_XFER64),
                         (PVOID)(ULONG_PTR)a[8], a[9], Who);
    Wow64pIoctlFree(In64);
    Wow64pIoctlFree(B64);
    return Wow64pFinishIosb(St, File, &Iosb64, a[4], Who);
}

/* IOCTL_AFD_RECV_DATAGRAM — AFD_RECV_INFO_UDP, 24 -> 40. Address and
 * AddressLength point at guest buffers the driver WRITES, but their pointees
 * (TRANSPORT_ADDRESS, INT) are width-identical, so widening the pointers is the
 * whole job — the driver writes straight into guest memory. */
static NTSTATUS Wow64pIoctlAfdRecvFrom(WOW64_IOCTL_FN Fn, ULONG *a, const char *Who)
{
    const WOW64_AFD_XFERUDP32 *In32 = (const WOW64_AFD_XFERUDP32 *)(ULONG_PTR)a[6];
    WOW64_IO_STATUS_BLOCK64_M Iosb64 = {0};
    WOW64_AFD_WSABUF64 *B64;
    WOW64_AFD_XFERUDP64 In64 = {0};
    HANDLE File = (HANDLE)(LONG_PTR)(LONG)a[0];
    HANDLE Event = (HANDLE)(LONG_PTR)(LONG)a[1];
    NTSTATUS St;

    if (!In32 || a[7] < sizeof(*In32))
        return Wow64pIoctlBadArg(Who, "input shorter than AFD_RECV_INFO_UDP");

    B64 = Wow64pAfdWidenBufs(In32->BufferArray, In32->BufferCount, Who, &St);
    if (St)
        return St;

    In64.BufferArray   = (ULONGLONG)(ULONG_PTR)B64;
    In64.BufferCount   = In32->BufferCount;
    In64.AfdFlags      = In32->AfdFlags;
    In64.TdiFlags      = In32->TdiFlags;
    In64.Address       = (ULONGLONG)(ULONG_PTR)In32->Address;
    In64.AddressLength = (ULONGLONG)(ULONG_PTR)In32->AddressLength;

    St = Wow64pIoctlCall(Fn, File, Event, &Iosb64, a[5], &In64, sizeof(In64),
                         (PVOID)(ULONG_PTR)a[8], a[9], Who);
    Wow64pIoctlFree(B64);
    return Wow64pFinishIosb(St, File, &Iosb64, a[4], Who);
}

/* BEFORE blaming any marshaller below for an IOCTL that "succeeded" with an
 * out-buffer that looks untouched: there is an arm64 kernel defect that presents
 * IDENTICALLY from the guest side. METHOD_BUFFERED's output copy-back on IRP
 * completion (SystemBuffer -> UserBuffer, length from Irp->IoStatus.Information)
 * intermittently does not land, and DeviceIoControl still returns ERROR_SUCCESS
 * — the kmtest that caught it passes its `== ERROR_SUCCESS` check and fails only
 * the value checks that follow. A width or thunking bug here and that defect are
 * indistinguishable without checking, and only one of them is ours. It never
 * appears in an error path, so nothing will point at it for you.
 * Localised by the conformance-ratchet lane (n=1, no rate); see its `Example`
 * entry. Check it first, then come back here.
 *
 * IOCTL_AFD_SEND_DATAGRAM — AFD_SEND_INFO_UDP, 56 -> 104. The one AFD payload
 * with nested structs, and the nesting is shallower than the declaration looks.
 * AfdPacketSocketWriteData reads only BufferArray, BufferCount and
 * TdiConnection.RemoteAddress; msafd's WSPSendTo fills only those plus AfdFlags
 * and RemoteAddressLength. TdiRequest is written by NEITHER side, so it is
 * carried as opaque zero padding rather than translated — synthesizing a
 * host-shaped TDI_REQUEST there would invent content no one supplies, and the
 * driver would read it back as if the guest had meant it.
 *
 * RemoteAddress points at a TRANSPORT_ADDRESS the guest built on its own stack
 * (WSPSendTo's Buffer[128]). That pointee is LONG + TA_ADDRESS{USHORT, USHORT,
 * UCHAR[]} — no pointers, identical on both widths — so widening the pointer is
 * the whole job, exactly as for RECV_DATAGRAM's Address. The driver
 * dereferences it in our own process context, so the guest address stays valid.
 *
 * The NULL check is ours, not the driver's: AfdPacketSocketWriteData
 * dereferences RemoteAddress unconditionally (AddressType, TAAddressCount,
 * TdiBuildConnectionInfo), so a guest passing NULL would take the kernel
 * through a null deref. Native i386 guests can do that too — this refuses it on
 * the one path we are already inspecting rather than widening a trap. */
static NTSTATUS Wow64pIoctlAfdSendTo(WOW64_IOCTL_FN Fn, ULONG *a, const char *Who)
{
    const WOW64_AFD_SENDUDP32 *In32 = (const WOW64_AFD_SENDUDP32 *)(ULONG_PTR)a[6];
    WOW64_IO_STATUS_BLOCK64_M Iosb64 = {0};
    WOW64_AFD_WSABUF64 *B64;
    WOW64_AFD_SENDUDP64 In64 = {0};
    HANDLE File = (HANDLE)(LONG_PTR)(LONG)a[0];
    HANDLE Event = (HANDLE)(LONG_PTR)(LONG)a[1];
    NTSTATUS St;

    if (!In32 || a[7] < sizeof(*In32))
        return Wow64pIoctlBadArg(Who, "input shorter than AFD_SEND_INFO_UDP");
    if (!In32->RemoteAddress)
        return Wow64pIoctlBadArg(Who, "AFD_SEND_INFO_UDP with a NULL RemoteAddress");

    B64 = Wow64pAfdWidenBufs(In32->BufferArray, In32->BufferCount, Who, &St);
    if (St)
        return St;

    In64.BufferArray         = (ULONGLONG)(ULONG_PTR)B64;
    In64.BufferCount         = In32->BufferCount;
    In64.AfdFlags            = In32->AfdFlags;
    In64.UserDataLength      = In32->UserDataLength;
    In64.UserData            = (ULONGLONG)(ULONG_PTR)In32->UserData;
    In64.OptionsLength       = In32->OptionsLength;
    In64.Options             = (ULONGLONG)(ULONG_PTR)In32->Options;
    In64.RemoteAddressLength = In32->RemoteAddressLength;
    In64.RemoteAddress       = (ULONGLONG)(ULONG_PTR)In32->RemoteAddress;

    St = Wow64pIoctlCall(Fn, File, Event, &Iosb64, a[5], &In64, sizeof(In64),
                         (PVOID)(ULONG_PTR)a[8], a[9], Who);
    Wow64pIoctlFree(B64);
    return Wow64pFinishIosb(St, File, &Iosb64, a[4], Who);
}

/* IOCTL_AFD_SELECT — AFD_POLL_INFO, 32 -> 40 for one handle. Doubly divergent:
 * the header grows AND the Handles[] element stride goes 12 -> 16, so the array
 * cannot be shared. IN/OUT: the driver writes Events and Status per handle, so
 * both halves are copied. */
static NTSTATUS Wow64pIoctlAfdSelect(WOW64_IOCTL_FN Fn, ULONG *a, const char *Who)
{
    const WOW64_AFD_POLLHDR32 *In32 = (const WOW64_AFD_POLLHDR32 *)(ULONG_PTR)a[6];
    WOW64_AFD_POLLHDR32 *Out32 = (WOW64_AFD_POLLHDR32 *)(ULONG_PTR)(a[8] ? a[8] : a[6]);
    const WOW64_AFD_HANDLE32 *H32;
    WOW64_AFD_HANDLE32 *HOut32;
    WOW64_IO_STATUS_BLOCK64_M Iosb64 = {0};
    WOW64_AFD_POLLHDR64 *P64;
    WOW64_AFD_HANDLE64 *H64;
    HANDLE File = (HANDLE)(LONG_PTR)(LONG)a[0];
    HANDLE Event = (HANDLE)(LONG_PTR)(LONG)a[1];
    NTSTATUS St;
    ULONG Count, i, Need;

    if (!In32 || a[7] < sizeof(*In32))
        return Wow64pIoctlBadArg(Who, "input shorter than AFD_POLL_INFO's header");
    Count = In32->HandleCount;
    if (Count > WOW64_AFD_MAX_HANDLES)
        return Wow64pIoctlBadArg(Who, "HandleCount above the marshalling cap");
    Need = (ULONG)sizeof(*In32) + Count * (ULONG)sizeof(WOW64_AFD_HANDLE32);
    if (a[7] < Need)
        return Wow64pIoctlBadArg(Who, "input shorter than HandleCount handles");
    /* The OUTPUT buffer is checked too, and up front rather than at copy-back
     * time. HandleCount comes from the guest and sizes the write below; taking
     * it on trust would walk off the end of the guest's buffer, which is the
     * very bug class this marshaller exists to close. Checking after the call
     * would also be too late to refuse — the select would already have run. */
    if ((a[8] ? a[9] : a[7]) < Need)
        return Wow64pIoctlBadArg(Who, "output shorter than HandleCount handles");

    P64 = (WOW64_AFD_POLLHDR64 *)Wow64pIoctlAlloc(sizeof(*P64) +
                                                  (SIZE_T)Count * sizeof(*H64));
    if (!P64)
        return WOW64_STATUS_NO_MEMORY;
    H64 = (WOW64_AFD_HANDLE64 *)(P64 + 1);
    H32 = (const WOW64_AFD_HANDLE32 *)(In32 + 1);

    P64->Timeout     = In32->Timeout;
    P64->HandleCount = Count;
    P64->Exclusive   = In32->Exclusive;
    for (i = 0; i < Count; i++)
    {
        H64[i].Handle = (ULONGLONG)(ULONG_PTR)H32[i].Handle;
        H64[i].Events = H32[i].Events;
        H64[i].Status = H32[i].Status;
    }

    St = Wow64pIoctlCall(Fn, File, Event, &Iosb64, a[5],
                         P64, (ULONG)(sizeof(*P64) + Count * sizeof(*H64)),
                         P64, (ULONG)(sizeof(*P64) + Count * sizeof(*H64)), Who);
    if (St >= 0 && Out32)
    {
        HOut32 = (WOW64_AFD_HANDLE32 *)(Out32 + 1);
        Out32->HandleCount = P64->HandleCount;
        for (i = 0; i < Count; i++)
        {
            HOut32[i].Handle = (ULONG)H64[i].Handle;
            HOut32[i].Events = H64[i].Events;
            HOut32[i].Status = H64[i].Status;
        }
    }
    Wow64pIoctlFree(P64);
    return Wow64pFinishIosb(St, File, &Iosb64, a[4], Who);
}

/* IOCTL_AFD_EVENT_SELECT — AFD_EVENT_SELECT_INFO, 8 -> 16. IN only. */
static NTSTATUS Wow64pIoctlAfdEventSelect(WOW64_IOCTL_FN Fn, ULONG *a, const char *Who)
{
    const WOW64_AFD_EVSEL32 *In32 = (const WOW64_AFD_EVSEL32 *)(ULONG_PTR)a[6];
    WOW64_IO_STATUS_BLOCK64_M Iosb64 = {0};
    WOW64_AFD_EVSEL64 In64 = {0};
    HANDLE File = (HANDLE)(LONG_PTR)(LONG)a[0];
    HANDLE Event = (HANDLE)(LONG_PTR)(LONG)a[1];
    NTSTATUS St;

    if (!In32 || a[7] < sizeof(*In32))
        return Wow64pIoctlBadArg(Who, "input shorter than AFD_EVENT_SELECT_INFO");
    In64.EventObject = (ULONGLONG)(LONG_PTR)(LONG)In32->EventObject;  /* HANDLE: sign-extend */
    In64.Events      = In32->Events;

    St = Wow64pIoctlCall(Fn, File, Event, &Iosb64, a[5], &In64, sizeof(In64),
                         (PVOID)(ULONG_PTR)a[8], a[9], Who);
    return Wow64pFinishIosb(St, File, &Iosb64, a[4], Who);
}

/* IOCTL_AFD_ENUM_NETWORK_EVENTS — AFD_ENUM_NETWORK_EVENTS_INFO, 48 -> 56.
 * Event is IN; PollEvents and the EventStatus[10] array are OUT. */
static NTSTATUS Wow64pIoctlAfdEnumEvents(WOW64_IOCTL_FN Fn, ULONG *a, const char *Who)
{
    WOW64_AFD_ENUMEV32 *In32 = (WOW64_AFD_ENUMEV32 *)(ULONG_PTR)a[6];
    WOW64_IO_STATUS_BLOCK64_M Iosb64 = {0};
    WOW64_AFD_ENUMEV64 In64 = {0};
    HANDLE File = (HANDLE)(LONG_PTR)(LONG)a[0];
    HANDLE Event = (HANDLE)(LONG_PTR)(LONG)a[1];
    NTSTATUS St;
    ULONG i;

    if (!In32 || a[7] < sizeof(*In32))
        return Wow64pIoctlBadArg(Who, "input shorter than AFD_ENUM_NETWORK_EVENTS_INFO");
    In64.Event = (ULONGLONG)(LONG_PTR)(LONG)In32->Event;

    St = Wow64pIoctlCall(Fn, File, Event, &Iosb64, a[5], &In64, sizeof(In64),
                         (PVOID)(ULONG_PTR)a[8], a[9], Who);
    if (St >= 0)
    {
        In32->PollEvents = In64.PollEvents;
        for (i = 0; i < 10; i++)
            In32->EventStatus[i] = In64.EventStatus[i];
    }
    return Wow64pFinishIosb(St, File, &Iosb64, a[4], Who);
}

/* IOCTL_AFD_ACCEPT — AFD_ACCEPT_DATA, 12 -> 16. IN only. */
static NTSTATUS Wow64pIoctlAfdAccept(WOW64_IOCTL_FN Fn, ULONG *a, const char *Who)
{
    const WOW64_AFD_ACCEPT32 *In32 = (const WOW64_AFD_ACCEPT32 *)(ULONG_PTR)a[6];
    WOW64_IO_STATUS_BLOCK64_M Iosb64 = {0};
    WOW64_AFD_ACCEPT64 In64 = {0};
    HANDLE File = (HANDLE)(LONG_PTR)(LONG)a[0];
    HANDLE Event = (HANDLE)(LONG_PTR)(LONG)a[1];
    NTSTATUS St;

    if (!In32 || a[7] < sizeof(*In32))
        return Wow64pIoctlBadArg(Who, "input shorter than AFD_ACCEPT_DATA");
    In64.UseSAN         = In32->UseSAN;
    In64.SequenceNumber = In32->SequenceNumber;
    In64.ListenHandle   = (ULONGLONG)(LONG_PTR)(LONG)In32->ListenHandle;

    St = Wow64pIoctlCall(Fn, File, Event, &Iosb64, a[5], &In64, sizeof(In64),
                         (PVOID)(ULONG_PTR)a[8], a[9], Who);
    return Wow64pFinishIosb(St, File, &Iosb64, a[4], Who);
}

/* IOCTL_AFD_GET_TDI_HANDLES — AFD_TDI_HANDLE_DATA, 8 -> 16. OUT only: the
 * driver writes two handles into Irp->UserBuffer, i.e. the OUTPUT buffer. */
static NTSTATUS Wow64pIoctlAfdTdiHandles(WOW64_IOCTL_FN Fn, ULONG *a, const char *Who)
{
    WOW64_AFD_TDIH32 *Out32 = (WOW64_AFD_TDIH32 *)(ULONG_PTR)a[8];
    WOW64_IO_STATUS_BLOCK64_M Iosb64 = {0};
    WOW64_AFD_TDIH64 Out64 = {0};
    HANDLE File = (HANDLE)(LONG_PTR)(LONG)a[0];
    HANDLE Event = (HANDLE)(LONG_PTR)(LONG)a[1];
    NTSTATUS St;

    if (!Out32 || a[9] < sizeof(*Out32))
        return Wow64pIoctlBadArg(Who, "output shorter than AFD_TDI_HANDLE_DATA");

    St = Wow64pIoctlCall(Fn, File, Event, &Iosb64, a[5], (PVOID)(ULONG_PTR)a[6], a[7],
                         &Out64, sizeof(Out64), Who);
    if (St >= 0)
    {
        Out32->TdiAddressHandle    = (ULONG)Out64.TdiAddressHandle;
        Out32->TdiConnectionHandle = (ULONG)Out64.TdiConnectionHandle;
    }
    return Wow64pFinishIosb(St, File, &Iosb64, a[4], Who);
}

static NTSTATUS Wow64pIoControl(WOW64_IOCTL_FN Fn, ULONG *a, const char *Who)
{
    HANDLE File = (HANDLE)(LONG_PTR)(LONG)a[0];
    HANDLE Event = (HANDLE)(LONG_PTR)(LONG)a[1];
    NTSTATUS St;

    /* M15c: an Event is no longer a refusal. It used to be, because the only
     * IOSB we could offer the kernel was a host one on this frame; the kernel
     * now writes the GUEST's 8-byte block directly (Wow64pLatchIosb32 above), which
     * outlives us by construction. An APC routine is still refused — that needs
     * a host-side thunk to stand in as the kernel's NormalRoutine (see the
     * M15b note above — the guest-entry half already exists). AFD is inherently
     * async and msafd always passes an Event, so this refusal was blocking
     * every socket operation a 32-bit guest could make. */
    if (a[2])
    {
        DbgPrint("[WOW64]   %s: guest ApcRoutine=%08lx is not supported yet "
                 "(needs a host thunk standing in as the NormalRoutine) — "
                 "refusing\n", Who, a[2]);
        return WOW64_STATUS_NOT_IMPLEMENTED;
    }

    switch (a[5])
    {
    case WOW64_IOCTL_AFD_SEND:
    case WOW64_IOCTL_AFD_RECV:
        return Wow64pIoctlAfdXfer(Fn, a, Who);
    case WOW64_IOCTL_AFD_RECV_DATAGRAM:
        return Wow64pIoctlAfdRecvFrom(Fn, a, Who);
    case WOW64_IOCTL_AFD_SELECT:
        return Wow64pIoctlAfdSelect(Fn, a, Who);
    case WOW64_IOCTL_AFD_EVENT_SELECT:
        return Wow64pIoctlAfdEventSelect(Fn, a, Who);
    case WOW64_IOCTL_AFD_ENUM_NETWORK_EVENTS:
        return Wow64pIoctlAfdEnumEvents(Fn, a, Who);
    case WOW64_IOCTL_AFD_ACCEPT:
        return Wow64pIoctlAfdAccept(Fn, a, Who);
    case WOW64_IOCTL_AFD_GET_TDI_HANDLES:
        return Wow64pIoctlAfdTdiHandles(Fn, a, Who);
    case WOW64_IOCTL_AFD_SEND_DATAGRAM:
        return Wow64pIoctlAfdSendTo(Fn, a, Who);
    default:
        break;
    }

    /* No shadow on this path, so nothing of ours has to outlive the call: latch
     * the guest's IOSB and let the request pend for real if it wants to. This
     * is the one IOCTL path that is genuinely asynchronous under WoW64. */
    Wow64pLatchIosb32(a[4]);
    St = Fn(File, Event, NULL, (PVOID)(ULONG_PTR)a[3], (PVOID)(ULONG_PTR)a[4],
            a[5], (PVOID)(ULONG_PTR)a[6], a[7], (PVOID)(ULONG_PTR)a[8], a[9]);
    /* Consumed by the kernel at entry; cleared here for the paths that fail
     * before reaching it (bad handle, failed probe). */
    Wow64pLatchIosb32(0);
    return St;
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

/* ---- E13: NtUserMessageCall — the guest->kernel message boundary ---------
 *
 * NtUserMessageCall(hWnd, Msg, wParam, lParam, ResultInfo, FNID, Ansi) carries
 * THREE independent 32/64 divergences, and all three were live:
 *
 *  1. The lParam payload. For the pointer-lParam messages in win32k's
 *     g_MsgMemory table, co_IntDoSendMessage copies MsgMemorySize() bytes out
 *     of the SENDER's buffer (CopyMsgToKernelMem) and back (CopyMsgToUserMem).
 *     That size is the HOST's sizeof(), so a 32-bit sender's smaller struct is
 *     over-READ on the way in — BUGCHECK 0x1E when it sits at the end of a page,
 *     which is what eventvwr/spider hit — and over-WRITTEN on the way out.
 *     M15a/M16/M18 had covered only the kernel->guest callback direction.
 *
 *  2. ResultInfo under FNID_SENDMESSAGE is a bare ULONG_PTR out-slot. The guest
 *     hands over a 4-byte stack local (user32 SendMessageW's `LRESULT Result`)
 *     and win32k stores sizeof(ULONG_PTR) = 8 into it: an UNCONDITIONAL 4-byte
 *     guest-stack smash on every SendMessage a 32-bit guest ever makes. Same
 *     shape as E7, on a hotter path.
 *
 *  3. ResultInfo under FNID_SENDMESSAGEFF / FNID_SENDMESSAGEWTOOPTION is a
 *     DOSENDMESSAGE {UINT uFlags; UINT uTimeout; ULONG_PTR Result;} — 12 bytes
 *     on i386, 16 on the host. win32k reads 16 from the 12-byte guest buffer,
 *     then writes 16 back over it.
 *
 * Every size above is a clang record-layout fact (marshalgen's own engine), not
 * arithmetic: DRAWITEMSTRUCT 48/64, MEASUREITEMSTRUCT 24/32, HELPINFO 28/40,
 * MDINEXTMENU 12/24, WINDOWPOS 28/40, DOSENDMESSAGE 12/16. MINMAXINFO, RECT and
 * STYLESTRUCT were checked the same way and are width-IDENTICAL — they need no
 * marshalling, and assuming otherwise would corrupt them.
 *
 * The payload converters are GENERATED (wow64_cb_automarshal.c, keyed by Msg)
 * and shared with the callback direction, so the two directions cannot drift
 * apart. This subsumes M16's hand-written FNID_DEFWINDOWPROC/WINDOWPOS case,
 * which is now just one row of that table.
 */
#include "wow64_cb_automarshal.h"

/* Values from win32ss/include/ntuser.h ("FNID's for NtUserSetWindowFNID,
 * NtUserMessageCall"). This TU is deliberately minimal (windef/winbase only),
 * so they are restated here — check them against that header when touching. */
#define WOW64_MC_FNID_SCROLLBAR            0x029A
#define WOW64_MC_FNID_MENU                 0x029C
#define WOW64_MC_FNID_DESKTOP              0x029D
#define WOW64_MC_FNID_DEFWINDOWPROC        0x029E
#define WOW64_MC_FNID_MESSAGEWND           0x029F
#define WOW64_MC_FNID_SENDMESSAGE          0x02B1
#define WOW64_MC_FNID_SENDMESSAGEFF        0x02B2
#define WOW64_MC_FNID_SENDMESSAGEWTOOPTION 0x02B3

/* Divergent pointer-lParam messages the flat table cannot express: an embedded
 * guest pointer (NCCALCSIZE_PARAMS.lppos, COPYDATASTRUCT.lpData) or a packed
 * string tail (CREATESTRUCTW). Marshalling these needs the pointee too. */
#define WOW64_MC_WM_CREATE     0x0001
#define WOW64_MC_WM_COPYDATA   0x004A
#define WOW64_MC_WM_NCCREATE   0x0081
#define WOW64_MC_WM_NCCALCSIZE 0x0083

/* DOSENDMESSAGE (win32ss/include/ntuser.h) at both widths. */
typedef struct { ULONG uFlags, uTimeout, Result; }            WOW64_MC_DSM32; /* 12 */
typedef struct { ULONG uFlags, uTimeout; ULONGLONG Result; }  WOW64_MC_DSM64; /* 16 */

/* Largest Size64 in the generated table; asserted at runtime below. */
#define WOW64_MC_PAYLOAD64_MAX 64

extern ULONG Wow64TraceSyscalls;

/* FNIDs for which lParam IS this message's payload buffer and win32k consumes
 * it synchronously — the SendMessage family (which copies it into pool around
 * the send) and the server-side WndProcs (which read it in place). Deliberately
 * NOT the hook FNIDs (CALLWNDPROC/CALLWNDPROCRET repack lParam into a CWPSTRUCT
 * that the callback path marshals separately), NOT BROADCASTSYSTEMMESSAGE, and
 * NOT the async posts (SENDNOTIFYMESSAGE / SENDMESSAGECALLBACK), where a host
 * stack buffer would outlive this call. */
static int Wow64pMcLParamIsPayload(ULONG FNID)
{
    return FNID == WOW64_MC_FNID_SENDMESSAGE ||
           FNID == WOW64_MC_FNID_SENDMESSAGEFF ||
           FNID == WOW64_MC_FNID_SENDMESSAGEWTOOPTION ||
           FNID == WOW64_MC_FNID_DEFWINDOWPROC ||
           FNID == WOW64_MC_FNID_SCROLLBAR ||
           FNID == WOW64_MC_FNID_DESKTOP ||
           FNID == WOW64_MC_FNID_MENU ||
           FNID == WOW64_MC_FNID_MESSAGEWND;
}

/* Reaches co_IntDoSendMessage, i.e. win32k sizes the payload copy from
 * g_MsgMemory using HOST widths. An unmarshallable divergent payload here is
 * the box-killing over-read, so we refuse rather than let it through. */
static int Wow64pMcGoesThroughMsgMemory(ULONG FNID)
{
    return FNID == WOW64_MC_FNID_SENDMESSAGE ||
           FNID == WOW64_MC_FNID_SENDMESSAGEFF ||
           FNID == WOW64_MC_FNID_SENDMESSAGEWTOOPTION;
}

static int Wow64pMcPayloadNeedsPointee(ULONG Msg)
{
    return Msg == WOW64_MC_WM_NCCALCSIZE || Msg == WOW64_MC_WM_COPYDATA ||
           Msg == WOW64_MC_WM_CREATE || Msg == WOW64_MC_WM_NCCREATE;
}

static NTSTATUS Wow64Manual_NtUserMessageCall(ULONG *a)
{
    extern NTSTATUS NtUserMessageCall(PVOID, ULONG_PTR, PVOID, PVOID, PVOID,
                                      ULONG_PTR, ULONG_PTR);
    ULONG Msg  = a[1];
    ULONG lp32 = a[3];
    ULONG ri32 = a[4];
    ULONG FNID = a[5];
    const WOW64_CB_MARSHAL *Cbm = NULL;
    unsigned char Payload64[WOW64_MC_PAYLOAD64_MAX];
    WOW64_MC_DSM64 Dsm64;
    ULONG_PTR Result64 = 0;
    PVOID lParamArg = (PVOID)(LONG_PTR)(LONG)a[3];
    PVOID ResultArg = (PVOID)(LONG_PTR)(LONG)a[4];
    NTSTATUS St;
    ULONG i;

    /* -- (1) lParam payload: guest 32-bit layout -> host 64-bit ------------ */
    if (lp32 && Wow64pMcLParamIsPayload(FNID))
    {
        Cbm = Wow64CbFindMarshal(Msg);
        if (Cbm)
        {
            if (Cbm->Size64 > sizeof(Payload64))
            {
                /* Generated table outgrew the buffer — a codegen change, not a
                 * guest input. Loud, never a silent truncation. */
                DbgPrint("[WOW64] NtUserMessageCall: msg 0x%lx payload %lu > "
                         "WOW64_MC_PAYLOAD64_MAX %u — raise it\n",
                         (unsigned long)Msg, (unsigned long)Cbm->Size64,
                         (unsigned)sizeof(Payload64));
                return 0;
            }
            /* Up() writes only the fields it knows; padding it skips must not
             * reach win32k as stack garbage. */
            for (i = 0; i < Cbm->Size64; i++)
                Payload64[i] = 0;
            Cbm->Up((const unsigned char *)(ULONG_PTR)lp32, Payload64);
            lParamArg = Payload64;
        }
        else if (Wow64pMcPayloadNeedsPointee(Msg) &&
                 Wow64pMcGoesThroughMsgMemory(FNID))
        {
            /* win32k would size the copy from the host struct and read past the
             * end of the guest's buffer. Failing the call loses the message;
             * letting it through can lose the machine. */
            DbgPrint("[WOW64] NtUserMessageCall: msg 0x%lx (FNID 0x%lx) has a "
                     "divergent lParam payload with an embedded pointer and no "
                     "marshaller — failing the call rather than over-reading "
                     "the guest buffer\n",
                     (unsigned long)Msg, (unsigned long)FNID);
            return 0;
        }
    }

    /* -- (2)/(3) ResultInfo out-slot --------------------------------------- */
    if (ri32)
    {
        if (FNID == WOW64_MC_FNID_SENDMESSAGE)
        {
            ResultArg = &Result64;
        }
        else if (FNID == WOW64_MC_FNID_SENDMESSAGEFF ||
                 FNID == WOW64_MC_FNID_SENDMESSAGEWTOOPTION)
        {
            const WOW64_MC_DSM32 *d32 = (const WOW64_MC_DSM32 *)(ULONG_PTR)ri32;
            Dsm64.uFlags   = d32->uFlags;
            Dsm64.uTimeout = d32->uTimeout;
            Dsm64.Result   = d32->Result;
            ResultArg = &Dsm64;
        }
    }

    St = NtUserMessageCall((PVOID)(LONG_PTR)(LONG)a[0], (ULONG_PTR)(LONG_PTR)(LONG)a[1],
                           (PVOID)(LONG_PTR)(LONG)a[2], lParamArg, ResultArg,
                           (ULONG_PTR)(LONG_PTR)(LONG)a[5],
                           (ULONG_PTR)(LONG_PTR)(LONG)a[6]);

    /* -- copy back: host 64-bit -> guest 32-bit ---------------------------- */
    if (Cbm)
        Cbm->Down(Payload64, (unsigned char *)(ULONG_PTR)lp32);

    if (ri32)
    {
        if (FNID == WOW64_MC_FNID_SENDMESSAGE)
        {
            *(ULONG *)(ULONG_PTR)ri32 = (ULONG)Result64;
        }
        else if (FNID == WOW64_MC_FNID_SENDMESSAGEFF ||
                 FNID == WOW64_MC_FNID_SENDMESSAGEWTOOPTION)
        {
            WOW64_MC_DSM32 *d32 = (WOW64_MC_DSM32 *)(ULONG_PTR)ri32;
            d32->uFlags   = Dsm64.uFlags;
            d32->uTimeout = Dsm64.uTimeout;
            d32->Result   = (ULONG)Dsm64.Result;
        }
    }

    if (Wow64TraceSyscalls && ResultArg != (PVOID)(LONG_PTR)(LONG)a[4])
        DbgPrint("[WOW64] NtUserMessageCall msg=0x%lx fnid=0x%lx result-slot %s\n",
                 (unsigned long)Msg, (unsigned long)FNID,
                 FNID == WOW64_MC_FNID_SENDMESSAGE ? "ULONG_PTR 8->4"
                                                   : "DOSENDMESSAGE 16->12");
    if (Wow64TraceSyscalls && Cbm)
        DbgPrint("[WOW64] NtUserMessageCall msg=0x%lx fnid=0x%lx payload "
                 "32<->64 (%u<->%u bytes)\n",
                 (unsigned long)Msg, (unsigned long)FNID,
                 (unsigned)Cbm->Size32, (unsigned)Cbm->Size64);
    return St;
}

/* Name-dispatched entry point. An unhandled name tails into the generated
 * table below, so it returns WOW64_AUTOMARSHAL_UNIMPL, not the manual one. */
/* ---- M18: NtCreateIoCompletion (PHANDLE out-slot + optional OA) ----------
 * Flushed out by marshal32.exe: the .spec typed it flat, so the PHANDLE arg
 * got a flat thunk and the kernel's 8-byte handle write clobbered the guest's
 * 4-byte slot + neighbor (CreateIoCompletionPort returned NULL). Same shape
 * as NtCreateKeyedEvent. */

static NTSTATUS Wow64Manual_NtCreateIoCompletion(ULONG *a)
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
    St = NtCreateIoCompletion(&Handle, Access, Oa64Ptr, a[3]);
    if (St >= 0 && Handle32)
        *Handle32 = (ULONG)(ULONG_PTR)Handle;
    return St;
}

/* ---- M18: NtQueryInformationThread (THREADINFOCLASS 32/64 marshal) ------ */

#define WOW64_ThreadBasicInformation          0
#define WOW64_ThreadTimes                     1
#define WOW64_ThreadQuerySetWin32StartAddress 9
#define WOW64_ThreadPerformanceCount          11
#define WOW64_ThreadAmILastThread             12
#define WOW64_ThreadPriorityBoost             14
#define WOW64_ThreadIsIoPending               16
#define WOW64_TEBSLOT_WOWTEBOFFSET            0x180c /* MmCreateTeb raw slot */

typedef struct _WOW64_TBI64 {
    ULONG ExitStatus; ULONG Pad0;
    ULONGLONG TebBaseAddress;
    ULONGLONG UniqueProcess; ULONGLONG UniqueThread;
    ULONGLONG AffinityMask;
    LONG Priority; LONG BasePriority;
} WOW64_TBI64;

typedef struct _WOW64_TBI32 {
    ULONG ExitStatus; ULONG TebBaseAddress;
    ULONG UniqueProcess; ULONG UniqueThread;
    ULONG AffinityMask; LONG Priority; LONG BasePriority;
} WOW64_TBI32;

static NTSTATUS Wow64Manual_NtQueryInformationThread(ULONG *a)
{
    HANDLE Thread = (HANDLE)(LONG_PTR)(LONG)a[0];
    ULONG Class = a[1];
    PVOID Buf32 = (PVOID)(ULONG_PTR)a[2];
    ULONG Len32 = a[3];
    ULONG *RetLen32 = (ULONG *)(ULONG_PTR)a[4];

    switch (Class)
    {
        case WOW64_ThreadBasicInformation:
        {
            WOW64_TBI64 T64;
            WOW64_TBI32 *T32 = Buf32;
            ULONG Ret64 = 0;
            NTSTATUS St;
            if (Len32 < sizeof(*T32))
            {
                if (RetLen32) *RetLen32 = sizeof(*T32);
                return (NTSTATUS)0xC0000004L; /* INFO_LENGTH_MISMATCH */
            }
            St = NtQueryInformationThread(Thread, Class, &T64, sizeof(T64), &Ret64);
            if (St < 0) return St;
            T32->ExitStatus = T64.ExitStatus;
            /* TebBaseAddress must be the 32-bit TEB. Same-process threads:
             * TEB32 = TEB64 + WowTebOffset (the kernel MmCreateTeb slot we
             * already use in Wow64LdrpInitialize). Cross-process handles
             * would need NtReadVirtualMemory; report 0 rather than a 64-bit
             * TEB the guest would misread. */
            T32->TebBaseAddress = 0;
            /* TEB64.ClientId.UniqueProcess is at +0x40 (TEB incomplete here) */
            if (T64.UniqueProcess ==
                *(ULONGLONG *)((PBYTE)NtCurrentTeb() + 0x40) &&
                T64.TebBaseAddress)
            {
                LONG WowOff = *(LONG *)((PBYTE)(ULONG_PTR)T64.TebBaseAddress +
                                        WOW64_TEBSLOT_WOWTEBOFFSET);
                if (WowOff)
                    T32->TebBaseAddress = (ULONG)(T64.TebBaseAddress + WowOff);
            }
            T32->UniqueProcess = (ULONG)T64.UniqueProcess;
            T32->UniqueThread = (ULONG)T64.UniqueThread;
            T32->AffinityMask = (ULONG)T64.AffinityMask;
            T32->Priority = T64.Priority;
            T32->BasePriority = T64.BasePriority;
            if (RetLen32) *RetLen32 = sizeof(*T32);
            return St;
        }

        case WOW64_ThreadQuerySetWin32StartAddress:
        {
            ULONGLONG Addr64 = 0;
            ULONG Ret64 = 0;
            NTSTATUS St;
            if (Len32 < sizeof(ULONG))
            {
                if (RetLen32) *RetLen32 = sizeof(ULONG);
                return (NTSTATUS)0xC0000004L;
            }
            St = NtQueryInformationThread(Thread, Class, &Addr64, sizeof(Addr64),
                                          &Ret64);
            if (St < 0) return St;
            *(ULONG *)Buf32 = (ULONG)Addr64;
            if (RetLen32) *RetLen32 = sizeof(ULONG);
            return St;
        }

        /* Width-identical info classes: LARGE_INTEGERs / ULONGs only. */
        case WOW64_ThreadTimes:            /* KERNEL_USER_TIMES: 4x LARGE_INTEGER */
        case WOW64_ThreadPerformanceCount: /* LARGE_INTEGER */
        case WOW64_ThreadAmILastThread:    /* ULONG */
        case WOW64_ThreadPriorityBoost:    /* ULONG */
        case WOW64_ThreadIsIoPending:      /* ULONG */
            return NtQueryInformationThread(Thread, Class, Buf32, Len32, RetLen32);

        default:
            /* Fail loud: an unmarshalled class passed through silently would
             * hand the guest 64-bit-laid-out data. */
            DbgPrint("[WOW64] NtQueryInformationThread class %lu NEEDS MANUAL "
                     "MARSHALLER — failing loud\n", Class);
            return (NTSTATUS)0xC0000003L; /* INVALID_INFO_CLASS */
    }
}

/* ---- M18: NtRemoveIoCompletion (IOCP dequeue: 2 out-slots + IOSB) -------- */

static NTSTATUS Wow64Manual_NtRemoveIoCompletion(ULONG *a)
{
    HANDLE Port = (HANDLE)(LONG_PTR)(LONG)a[0];
    ULONG *Key32 = (ULONG *)(ULONG_PTR)a[1];
    ULONG *Ctx32 = (ULONG *)(ULONG_PTR)a[2];
    PVOID Key64 = NULL, Ctx64 = NULL;
    WOW64_IO_STATUS_BLOCK64_M Iosb64 = {0};
    NTSTATUS St;

    /* Timeout is a LARGE_INTEGER* — width-identical, guest pointer passes. */
    St = NtRemoveIoCompletion(Port, &Key64, &Ctx64, &Iosb64,
                              (PLARGE_INTEGER)(ULONG_PTR)a[4]);
    if (St == 0)
    {
        if (Key32) *Key32 = (ULONG)(ULONG_PTR)Key64;
        if (Ctx32) *Ctx32 = (ULONG)(ULONG_PTR)Ctx64;
        Wow64pCopyIosbDown(&Iosb64, a[3]);
    }
    return St;
}

/* ---- M18: NtUserEnumDisplayMonitors (count-aware HMONITOR out-array) -----
 * The ROS syscall is FIVE args (the MSDN-shaped 4-arg prototype above it in
 * ntuser.h is commented out — a trap marshal32.exe caught):
 *   (HDC, LPCRECT, HMONITOR *hMonitorList, LPRECT monitorRectList, DWORD listSize)
 * Only hMonitorList diverges (4 vs 8 per element); monitorRectList is RECTs
 * (width-identical, guest pointer passes straight through). Probe call
 * (both lists NULL) just returns the count. */

#define WOW64_MON_MAX 16

static NTSTATUS Wow64Manual_NtUserEnumDisplayMonitors(ULONG *a)
{
    HANDLE Dc = (HANDLE)(LONG_PTR)(LONG)a[0];
    PVOID Rect = (PVOID)(ULONG_PTR)a[1];      /* LPCRECT: width-identical */
    ULONG *List32 = (ULONG *)(ULONG_PTR)a[2];
    PVOID RectList = (PVOID)(ULONG_PTR)a[3];  /* LPRECT[]: width-identical */
    ULONG Want = a[4];
    ULONGLONG List64[WOW64_MON_MAX];
    INT Got;
    ULONG i;

    if (!List32 || !Want)
    {
        /* Probe/count call (user32 makes this first with both lists NULL). */
        return (NTSTATUS)NtUserEnumDisplayMonitors(Dc, Rect, NULL, RectList, 0);
    }
    if (Want > WOW64_MON_MAX)
    {
        DbgPrint("[WOW64] NtUserEnumDisplayMonitors: %lu monitors > cap %u — "
                 "failing loud\n", Want, WOW64_MON_MAX);
        return (NTSTATUS)0xC0000095L; /* INTEGER_OVERFLOW: never silently truncate */
    }
    Got = NtUserEnumDisplayMonitors(Dc, Rect, List64, RectList, Want);
    for (i = 0; Got > 0 && i < Want && i < (ULONG)Got; i++)
        List32[i] = (ULONG)List64[i];
    return (NTSTATUS)Got;
}

/* ---- E22: NtUserThunkedMenuInfo — the declared type is NOT the real payload
 *
 * The prototype says `LPCMENUINFO`, and `marshalgen` believed it: MENUINFO is
 * 28 bytes in the guest and 40 on the host, a clean WIDENING diff. But win32k
 * casts the pointer to `PROSMENUINFO` and `UserMenuInfo` reads **`cbSize`
 * bytes** out of the caller, accepting anything in the range
 * `[sizeof(MENUINFO), sizeof(ROSMENUINFO)]` — ROSMENUINFO being MENUINFO plus
 * a dozen internal fields. The size is therefore CALLER-DECLARED and variable,
 * which is precisely the shape a layout-diff marshaller cannot see: a
 * generated thunk sized from the MENUINFO diff hands the kernel a 40-byte
 * bounce and then lets it read `cbSize` bytes out of it. A guest declaring a
 * ROSMENUINFO would have the kernel read ~36 bytes past the end of wow64.dll's
 * own stack buffer — a bug the marshaller would have INTRODUCED, since the
 * flat thunk it replaced at least handed over a real, correctly-sized guest
 * pointer.
 *
 * So marshal exactly the MENUINFO size class and pass everything else through
 * untouched. user32's SetMenuInfo (the only in-tree caller) always sends
 * exactly `sizeof(MENUINFO)`, so the class that matters is covered; a direct
 * caller declaring more gets today's behaviour, not a fresh overread.
 *
 * IN-only: NtUserThunkedMenuInfo always calls UserMenuInfo with SetOrGet=1,
 * and only the Get branch copies back. GetMenuInfo never reaches the kernel at
 * all — user32 answers it from the client-side PMENU. */

static NTSTATUS Wow64Manual_NtUserThunkedMenuInfo(ULONG *a)
{
    extern ULONG NtUserThunkedMenuInfo(PVOID, PVOID);
    typedef struct { ULONG cbSize, fMask, dwStyle, cyMax, hbrBack,
                     dwContextHelpID, dwMenuData; } MI32;         /* 28 */
    typedef struct { ULONG cbSize, fMask, dwStyle, cyMax;
                     ULONGLONG hbrBack;
                     ULONG dwContextHelpID, __pad;
                     ULONGLONG dwMenuData; } MI64;                /* 40 */
    const MI32 *M32 = (const MI32 *)(ULONG_PTR)a[1];
    MI64 M64;

    if (!M32 || M32->cbSize != sizeof(MI32))
        return (NTSTATUS)NtUserThunkedMenuInfo((PVOID)(ULONG_PTR)a[0],
                                               (PVOID)(ULONG_PTR)a[1]);

    M64.cbSize = sizeof(M64);      /* the whole point: the host's own sizeof */
    M64.fMask = M32->fMask;
    M64.dwStyle = M32->dwStyle;
    M64.cyMax = M32->cyMax;
    M64.hbrBack = M32->hbrBack;
    M64.dwContextHelpID = M32->dwContextHelpID;
    M64.__pad = 0;
    M64.dwMenuData = M32->dwMenuData;
    return (NTSTATUS)NtUserThunkedMenuInfo((PVOID)(ULONG_PTR)a[0], &M64);
}

/* ---- E22: NtUserSendInput (the size-as-an-ARGUMENT half of the cbSize class)
 *
 * SendInput is the one member of the self-describing-size family that states
 * its size in a SYSCALL ARGUMENT rather than in a leading struct field, so
 * marshalgen's WowAmSizeIn/Out cannot reach it — it has no idea that arg 2
 * describes arg 1. win32k's check is
 *     `cbSize as usize != size_of::<INPUT>()`  (input.rs)
 * so a 32-bit guest offering its own sizeof (28) is rejected against the host
 * 40 and EVERY SendInput from EVERY 32-bit process returned 0 with
 * ERROR_INVALID_PARAMETER. That is not a crash, which is why nothing found it:
 * apps that synthesize input (installers, automation, accessibility, games
 * that warp the pointer) just quietly did nothing.
 *
 * The payload diverges twice over: the union sits at offset 4 in the guest and
 * 8 on the host (8-alignment for ULONG_PTR), and each arm's dwExtraInfo widens
 * 4->8. So the stride is 28 vs 40 and a flat pass would have the kernel walk
 * the guest array with 40-byte strides — reading 12 bytes past the end of a
 * single-element array even before the union offsets are considered.
 *
 * dwExtraInfo is ZERO-extended, not sign-extended like a handle: it is
 * app-defined data the guest reads back through GetMessageExtraInfo, and
 * sign-extending would corrupt any value with bit 31 set. */

#define WOW64_INPUT32_CB   28
#define WOW64_INPUT64_CB   40
#define WOW64_SENDINPUT_STACK 64   /* fast path; larger batches go to the heap */

/* winuser.h is not in this TU's include set (windef.h + winbase.h only). */
#define WOW64_INPUT_MOUSE     0
#define WOW64_INPUT_KEYBOARD  1

/* Byte-wise throughout: the guest array has a 28-byte stride, so element N is
 * only 4-aligned if the guest's base pointer was, and this runs on arm64 too. */
static ULONG Wow64pGet32(const unsigned char *p)
{
    ULONG v;
    unsigned i;
    for (i = 0; i < 4; i++)
        ((unsigned char *)&v)[i] = p[i];
    return v;
}

static void Wow64pPut64(unsigned char *p, ULONGLONG v)
{
    unsigned i;
    for (i = 0; i < 8; i++)
        p[i] = ((const unsigned char *)&v)[i];
}

static void Wow64pInput32To64(const unsigned char *g, unsigned char *h)
{
    unsigned i;

    for (i = 0; i < WOW64_INPUT64_CB; i++)
        h[i] = 0;
    for (i = 0; i < 4; i++)
        h[i] = g[i];                       /* type, offset 0 on both sides */

    switch (Wow64pGet32(g))
    {
    case WOW64_INPUT_MOUSE:
        /* dx,dy,mouseData,dwFlags,time identical; dwExtraInfo 20 -> 24 */
        for (i = 0; i < 20; i++)
            h[8 + i] = g[4 + i];
        Wow64pPut64(h + 8 + 24, (ULONGLONG)Wow64pGet32(g + 4 + 20));
        break;
    case WOW64_INPUT_KEYBOARD:
        /* wVk,wScan,dwFlags,time identical; dwExtraInfo 12 -> 16 */
        for (i = 0; i < 12; i++)
            h[8 + i] = g[4 + i];
        Wow64pPut64(h + 8 + 16, (ULONGLONG)Wow64pGet32(g + 4 + 12));
        break;
    default:
        /* INPUT_HARDWARE (8 bytes, identical) and anything the guest made up.
         * Copy the guest union verbatim and let win32k reject the type — a
         * marshaller must not turn an invalid type into a valid one. */
        for (i = 0; i < WOW64_INPUT32_CB - 4; i++)
            h[8 + i] = g[4 + i];
        break;
    }
}

static NTSTATUS Wow64Manual_NtUserSendInput(ULONG *a)
{
    extern ULONG NtUserSendInput(ULONG nInputs, PVOID pInput, INT cbSize);
    ULONG nInputs = a[0];
    const unsigned char *In32 = (const unsigned char *)(ULONG_PTR)a[1];
    ULONG cbGuest = a[2];
    unsigned char Stack[WOW64_SENDINPUT_STACK * WOW64_INPUT64_CB];
    unsigned char *In64 = Stack;
    PVOID Heap = NULL;
    ULONG Ret, i;

    /* Pass a wrong cbSize straight through so the kernel still rejects it:
     * only the guest's own sizeof means "I am a 32-bit INPUT". */
    if (!In32 || !nInputs || cbGuest != WOW64_INPUT32_CB)
        return (NTSTATUS)NtUserSendInput(nInputs, (PVOID)(ULONG_PTR)a[1], (INT)cbGuest);

    if (nInputs > WOW64_SENDINPUT_STACK)
    {
        SIZE_T Size = (SIZE_T)nInputs * WOW64_INPUT64_CB;
        NTSTATUS St = NtAllocateVirtualMemory((HANDLE)(LONG_PTR)-1, &Heap, 0, &Size,
                                              MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE);
        if (St < 0)
            return St;
        In64 = (unsigned char *)Heap;
    }

    for (i = 0; i < nInputs; i++)
        Wow64pInput32To64(In32 + (SIZE_T)i * WOW64_INPUT32_CB,
                          In64 + (SIZE_T)i * WOW64_INPUT64_CB);

    /* IN-only: win32k copies each entry into its own SafeInput and never
     * writes back through pInput, so there is no copy-down. */
    Ret = NtUserSendInput(nInputs, In64, WOW64_INPUT64_CB);

    if (Heap)
    {
        SIZE_T Free = 0;
        NtFreeVirtualMemory((HANDLE)(LONG_PTR)-1, &Heap, &Free, MEM_RELEASE);
    }
    return (NTSTATUS)Ret;
}

/*
 * NtUserInitializeClientPfnArrays — INTERCEPT. The guest's user32 calls this once
 * at process init with three PFNCLIENT/PFNCLIENTWORKER structs: arrays of
 * FUNCTION POINTERS, 4 bytes each in the guest and 8 in the kernel.
 *
 * What makes this one different from the rest of the divergence family is the
 * destination. win32k does not keep these per-process — it RtlCopyMemory's them
 * into gpsi->apfnClientA/W/Worker and aStoCidPfn, the GLOBAL server info every
 * process shares, and sets the global hModClient. So a 32-bit guest reaching
 * that body would (a) be read for the HOST struct size, ~2x its own buffer, and
 * (b) overwrite the SYSTEM-WIDE client-callback table with pairwise-merged
 * garbage, breaking callbacks for native 64-bit processes too. A WoW64 process
 * corrupting global kernel state is a different blast radius from one corrupting
 * itself.
 *
 * Today it is harmless for exactly one reason: win32k early-outs on
 * `ClientPfnInit != 0`, and native user32 (csrss/winlogon) always wins that race
 * long before any guest starts. That is initialization ORDER, not a guarantee —
 * so the call is intercepted here rather than left resting on it.
 *
 * Returning SUCCESS without calling the kernel preserves the observable
 * behaviour exactly (the kernel's early-out already made it a no-op) and drops
 * the ordering dependency. The guest needs nothing from it: 32-bit callbacks are
 * dispatched by the WoW64 callback bridge through
 * Wow64pKiUserCallbackDispatcher32, not through gpsi.
 *   NtUserInitializeClientPfnArrays(pfnClientA, pfnClientW, pfnClientWorker, hmodUser)
 */
static NTSTATUS Wow64Manual_NtUserInitializeClientPfnArrays(ULONG *a)
{
    static int Announced;

    /* Not installed globally, but remembered: wow64_pfnmap.c maps the host's
     * built-in class procs onto these slots before they reach the guest. */
    Wow64pPfnMapSetGuest(a[0], a[1]);
    if (!Announced)
    {
        Announced = 1;
        DbgPrint("[WOW64] NtUserInitializeClientPfnArrays: INTERCEPTED - guest "
                 "32-bit PFNCLIENT arrays are never installed into the GLOBAL "
                 "gpsi callback table (they would be read at host width and "
                 "corrupt it for native processes). Guest callbacks go through "
                 "the WoW64 callback bridge.\n");
    }
    return 0; /* STATUS_SUCCESS — matches win32k's own ClientPfnInit early-out */
}

/*
 * NtOpenProcessTokenEx / NtOpenThreadTokenEx — an OUT PHANDLE is 4 bytes in the
 * guest and 8 on the host, so the flat thunk handed the 64-bit kernel a 4-byte
 * cell to write 8 bytes into. It failed TWO ways, chosen by the address, because
 * the kernel's probe is itself width-typed (probe_write_handle is
 * probe_write_user::<PVOID>, i.e. 8-byte alignment) while a guest HANDLE is only
 * 4-aligned:
 *   - cell at ≡4 (mod 8): rejected outright, STATUS_DATATYPE_MISALIGNMENT — a
 *     status the guest's ABI says this call cannot return;
 *   - cell at ≡0 (mod 8): probe passes, 8 bytes written. Little-endian keeps
 *     this half quiet: the low dword lands correctly so the guest reads a valid
 *     handle, while the high dword (and the probe's own NULL touch-test) zeroes
 *     the adjacent 4 bytes — E7's class, surfacing far from the write.
 * Measured 12 calls in one amd64 boot (8 + 4); the tell was NtOpenThreadTokenEx
 * appearing TWICE per logical open, the first returning 0x80000002. Marshalling
 * through a host-width local makes it 1:1.
 *
 * Narrowing the result is not a truncation risk: a WoW64 process's handles must
 * fit in 32 bits by construction, or the guest could never name them at all.
 *
 * NtOpenProcessTokenEx(ProcessHandle, DesiredAccess, HandleAttributes, TokenHandle)
 * NtOpenThreadTokenEx(ThreadHandle, DesiredAccess, OpenAsSelf, HandleAttributes,
 *                     TokenHandle)
 */
static NTSTATUS Wow64Manual_NtOpenProcessTokenEx(ULONG *a)
{
    ULONG *Handle32 = (ULONG *)(ULONG_PTR)a[3];
    HANDLE Handle = NULL;
    NTSTATUS St;

    St = NtOpenProcessTokenEx((HANDLE)(LONG_PTR)(LONG)a[0], (ACCESS_MASK)a[1],
                              a[2], &Handle);
    if (St >= 0 && Handle32)
        *Handle32 = (ULONG)(ULONG_PTR)Handle;
    return St;
}

static NTSTATUS Wow64Manual_NtOpenThreadTokenEx(ULONG *a)
{
    ULONG *Handle32 = (ULONG *)(ULONG_PTR)a[4];
    HANDLE Handle = NULL;
    NTSTATUS St;

    St = NtOpenThreadTokenEx((HANDLE)(LONG_PTR)(LONG)a[0], (ACCESS_MASK)a[1],
                             (BOOLEAN)(a[2] != 0), a[3], &Handle);
    if (St >= 0 && Handle32)
        *Handle32 = (ULONG)(ULONG_PTR)Handle;
    return St;
}

/*
 * NtGetContextThread / NtSetContextThread — an i386 CONTEXT is not widenable,
 * and the flat thunk did not try: it handed the guest's 32-bit CONTEXT pointer
 * to the 64-bit kernel to use at HOST layout. Get then wrote a 1232-byte amd64
 * CONTEXT into a 716-byte (0x2CC) guest buffer — a >500-byte overflow of guest
 * memory — and Set loaded the native thread's register file out of i386 fields
 * read at the wrong offsets. Both are catastrophic; both returned SUCCESS.
 *
 * No kernel path can help, because the 64-bit kernel has no notion of the
 * guest's registers. Guest CPU state lives in the CPU layer, and
 * BTCpuGet/SetContext already speak exactly the i386 CONTEXT this syscall
 * carries — the same primitives NtContinue and the callback bridge use. So
 * these are real marshallers for the case that matters, not refusals.
 *
 * SELF-THREAD ONLY. BTCpuGet/SetContext ignore their Thread argument and act on
 * the calling thread's CPU area; a cross-thread query needs a suspend handshake
 * with the target's CPU layer (task #64). Cross-thread is REFUSED rather than
 * silently answered with the caller's own registers — which is what makes this
 * safe to add before #64 exists.
 *
 * Two amd64-only quirks, both consequences of wow64cpu.S's contract (arm64's
 * CPU layer is FEX, which tracks full live guest state and needs neither):
 *   - The saved Esp is the LIVE one, still pointing at the stub return address,
 *     while the saved Eip is where the guest will RESUME. Wow64CpuDispatch
 *     reconciles that pair with `Esp += 4` on the normal return, so a snapshot
 *     handed to the guest has to apply it or Eip and Esp disagree by one slot.
 *   - Only the SegGs..SegSs window is maintained. A request for FloatSave, Dr*
 *     or ExtendedRegisters gets those bits CLEARED from the returned
 *     ContextFlags — the API's own way of saying "not supplied" — plus a log
 *     line, instead of handing back a buffer that looks filled in and isn't.
 */
#define WOW64_CTX32_SEG_LO      0x8C   /* SegGs, SegFs, SegEs, SegDs          */
#define WOW64_CTX32_SEG_HI      0x98
#define WOW64_CTX32_INT_LO      0x9C   /* Edi, Esi, Ebx, Edx, Ecx, Eax        */
#define WOW64_CTX32_INT_HI      0xB0
#define WOW64_CTX32_EBX         0xA4
#define WOW64_CTX32_EAX         0xB0
#define WOW64_CTX32_CTL_LO      0xB4   /* Ebp, Eip, SegCs, EFlags, Esp, SegSs */
#define WOW64_CTX32_CTL_HI      0xC8

#define WOW64_CTX32_ARCH        0x00010000u /* CONTEXT_i386                   */
#define WOW64_CTX32_F_CONTROL   0x00000001u
#define WOW64_CTX32_F_INTEGER   0x00000002u
#define WOW64_CTX32_F_SEGMENTS  0x00000004u
/* CONTEXT_FLOATING_POINT | CONTEXT_DEBUG_REGISTERS | CONTEXT_EXTENDED_REGISTERS */
#define WOW64_CTX32_F_UNTRACKED 0x00000038u
#define WOW64_CTX32_WANTS(f, g) \
    (((f) & (WOW64_CTX32_ARCH | (g))) == (WOW64_CTX32_ARCH | (g)))

/* The guest CONTEXT must lie wholly in the 32-bit user half and be aligned. */
static int Wow64pCtx32Valid(ULONG Addr)
{
    return Addr >= 0x1000u && Addr < 0x80000000u && (Addr & 3) == 0 &&
           (0x80000000u - Addr) >= WOW64_CTX32_SIZE;
}

/* The CPU-area context as the guest will observe it once this service returns
 * (see the Esp/Eip note above). */
static LONG Wow64pCpuGetContext32(PVOID Ctx32)
{
    extern LONG (WINAPI *Wow64pBTCpuGetContext)(HANDLE, HANDLE, PVOID, PVOID);
    LONG St;

    if (!Wow64pBTCpuGetContext)
        return (LONG)0xC0000002L;
    St = Wow64pBTCpuGetContext((HANDLE)(LONG_PTR)-2, WOW64_PROCESS_CURRENT,
                               NULL, Ctx32);
#ifndef _M_ARM64
    if (St >= 0)
        *(volatile ULONG *)((PBYTE)Ctx32 + WOW64_CTX32_ESP) += 4;
#endif
    return St;
}

/* THREAD_BASIC_INFORMATION prefix at host layout; only TebBaseAddress is read.
 * Comparing TEB bases identifies the calling thread through a real handle, not
 * just the (HANDLE)-2 pseudo-handle. */
typedef struct _WOW64_THREAD_BASIC_INFO
{
    NTSTATUS  ExitStatus;
    PVOID     TebBaseAddress;
    ULONG_PTR UniqueProcess;
    ULONG_PTR UniqueThread;
    ULONG_PTR AffinityMask;
    LONG      Priority;
    LONG      BasePriority;
} WOW64_THREAD_BASIC_INFO;

static int Wow64pIsCurrentThread(ULONG Handle32)
{
    WOW64_THREAD_BASIC_INFO Self, Target;
    HANDLE Thread = (HANDLE)(LONG_PTR)(LONG)Handle32;

    if (Thread == (HANDLE)(LONG_PTR)-2)
        return 1;
    /* A handle without THREAD_QUERY_INFORMATION lands here as "not self" and is
     * refused — a wrong answer would be worse than a refused one. */
    if (NtQueryInformationThread((HANDLE)(LONG_PTR)-2, 0 /*ThreadBasicInformation*/,
                                 &Self, sizeof(Self), NULL) < 0 ||
        NtQueryInformationThread(Thread, 0, &Target, sizeof(Target), NULL) < 0)
        return 0;
    return Self.TebBaseAddress == Target.TebBaseAddress;
}

/*   NtGetContextThread(ThreadHandle, Context) */
static NTSTATUS Wow64Manual_NtGetContextThread(ULONG *a)
{
    ULONG Ctx32 = a[1];
    ULONG Want;
    LONG St;

    if (!Wow64pCtx32Valid(Ctx32))
        return (NTSTATUS)0xC000000DL; /* STATUS_INVALID_PARAMETER */
    if (!Wow64pIsCurrentThread(a[0]))
    {
        DbgPrint("[WOW64] NtGetContextThread: REFUSED - cross-thread query "
                 "(handle %08lx). The CPU layer only exposes the CALLING "
                 "thread's guest state (task #64).\n", a[0]);
        return (NTSTATUS)0xC0000002L; /* STATUS_NOT_IMPLEMENTED */
    }

    Want = *(volatile ULONG *)(ULONG_PTR)Ctx32;
    St = Wow64pCpuGetContext32((PVOID)(ULONG_PTR)Ctx32);
    if (St < 0)
        return (NTSTATUS)St;
#ifndef _M_ARM64
    if (Want & WOW64_CTX32_F_UNTRACKED)
    {
        DbgPrint("[WOW64] NtGetContextThread: ContextFlags %08lx asked for "
                 "FP/Dr/extended state wow64cpu does not maintain; clearing "
                 "those bits in the reply, fields left untouched.\n", Want);
        Want &= ~WOW64_CTX32_F_UNTRACKED;
    }
#endif
    *(volatile ULONG *)(ULONG_PTR)Ctx32 = Want;
    return 0;
}

/*   NtSetContextThread(ThreadHandle, Context) */
static NTSTATUS Wow64Manual_NtSetContextThread(ULONG *a)
{
    extern LONG (WINAPI *Wow64pBTCpuSetContext)(HANDLE, HANDLE, PVOID, PVOID);
    ULONG Ctx32 = a[1];
    const BYTE *Src;
    BYTE Merged[WOW64_CTX32_SIZE];
    ULONG Want, i;
    LONG St;

    if (!Wow64pCtx32Valid(Ctx32))
        return (NTSTATUS)0xC000000DL;
    if (!Wow64pBTCpuSetContext)
        return (NTSTATUS)0xC0000002L;
    if (!Wow64pIsCurrentThread(a[0]))
    {
        DbgPrint("[WOW64] NtSetContextThread: REFUSED - cross-thread set "
                 "(handle %08lx). Installing it on the CALLING thread instead "
                 "would corrupt the wrong thread (task #64).\n", a[0]);
        return (NTSTATUS)0xC0000002L;
    }

    Src = (const BYTE *)(ULONG_PTR)Ctx32;
    Want = *(const volatile ULONG *)(const void *)Src;

    /* A partial ContextFlags must leave the unrequested groups ALONE, but
     * BTCpuSetContext installs the whole SegGs..SegSs window unconditionally
     * and flags it as the state to RESUME. So start from the live context and
     * overlay only the groups the guest actually asked for. */
    for (i = 0; i < WOW64_CTX32_SIZE; i++)
        Merged[i] = 0;
    *(ULONG *)Merged = Want;
    St = Wow64pCpuGetContext32(Merged);
    if (St < 0)
        return (NTSTATUS)St;

    if (WOW64_CTX32_WANTS(Want, WOW64_CTX32_F_SEGMENTS))
        for (i = WOW64_CTX32_SEG_LO; i <= WOW64_CTX32_SEG_HI; i += 4)
            *(ULONG *)(Merged + i) = *(const ULONG *)(Src + i);
    if (WOW64_CTX32_WANTS(Want, WOW64_CTX32_F_INTEGER))
        for (i = WOW64_CTX32_INT_LO; i <= WOW64_CTX32_INT_HI; i += 4)
            *(ULONG *)(Merged + i) = *(const ULONG *)(Src + i);
    else
        /* Resuming a replaced context bypasses Wow64CpuDispatch's
         * `Eax = result` fixup, so the return value has to be planted by hand.
         * If the guest asked to set INTEGER state it chose its own Eax, and
         * that wins — same as a real self-directed SetThreadContext. */
        *(ULONG *)(Merged + WOW64_CTX32_EAX) = 0; /* STATUS_SUCCESS */
    if (WOW64_CTX32_WANTS(Want, WOW64_CTX32_F_CONTROL))
        for (i = WOW64_CTX32_CTL_LO; i <= WOW64_CTX32_CTL_HI; i += 4)
            *(ULONG *)(Merged + i) = *(const ULONG *)(Src + i);
#ifndef _M_ARM64
    if (Want & WOW64_CTX32_F_UNTRACKED)
        DbgPrint("[WOW64] NtSetContextThread: ContextFlags %08lx asked to set "
                 "FP/Dr/extended state wow64cpu does not maintain; those groups "
                 "were IGNORED (control/integer/segments applied).\n", Want);
#endif

    return (NTSTATUS)Wow64pBTCpuSetContext((HANDLE)(LONG_PTR)-2,
                                           WOW64_PROCESS_CURRENT, NULL, Merged);
}

/*
 * NtWaitForMultipleObjects — the wait ARRAY is 4-byte handles in the guest and
 * 8-byte on the host, so the flat thunk had the kernel read every handle from
 * the wrong offset and reject the call with STATUS_INVALID_HANDLE (c0000008).
 *
 * It was in the audit baseline as a cold SCALAR_WIDEN entry and became live the
 * moment 32-bit guests could create threads: WaitForMultipleObjects is how a
 * threaded app joins its workers (task #64 found it this way). Single-object
 * waits pass a handle by value and were never affected, which is why nothing
 * noticed until now.
 */
#define WOW64_MAXIMUM_WAIT_OBJECTS 64

static NTSTATUS Wow64Manual_NtWaitForMultipleObjects(ULONG *a)
{
    ULONG Count = a[0];
    const ULONG *Handles32 = (const ULONG *)(ULONG_PTR)a[1];
    HANDLE Handles64[WOW64_MAXIMUM_WAIT_OBJECTS];
    ULONG i;

    /* The kernel enforces the same ceiling; matching its status keeps the
     * guest's error path identical to a native one. */
    if (Count == 0 || Count > WOW64_MAXIMUM_WAIT_OBJECTS)
        return (NTSTATUS)0xC00000EFL; /* STATUS_INVALID_PARAMETER_1 */
    if (!Handles32)
        return (NTSTATUS)0xC00000F0L; /* STATUS_INVALID_PARAMETER_2 */

    for (i = 0; i < Count; i++)
        Handles64[i] = (HANDLE)(LONG_PTR)(LONG)Handles32[i];

    return NtWaitForMultipleObjects(Count, Handles64, a[2],
                                    (BOOLEAN)(a[3] & 0xFF),
                                    (PLARGE_INTEGER)(ULONG_PTR)a[4]);
}

/* i386 INITIAL_TEB: five 4-byte pointers — PreviousStackBase,
 * PreviousStackLimit, StackBase, StackLimit, AllocatedStackBase. */
#define WOW64_ITEB32_STACKBASE  0x08
#define WOW64_ITEB32_STACKLIMIT 0x0C
/* AllocatedStackBase -- the BOTTOM of the guest's stack reservation, which the
 * guest's BaseCreateStack already filled in and this thunk used to discard.
 * It becomes TEB32.DeallocationStack, without which the kernel cannot grow a
 * guest stack at all (see Wow64pThreadInit). */
#define WOW64_ITEB32_ALLOCBASE  0x10

/* wow64.c — #64: creates a NATIVE thread that runs 32-bit guest code, and
 * establishes the per-thread WoW64 state on it (CPU area, TEB32 NtTib stack
 * bounds, BTCpuThreadInit) before entering the guest through its own loader. */
typedef struct _WOW64_CLIENT_ID {
    HANDLE UniqueProcess;
    HANDLE UniqueThread;
} WOW64_CLIENT_ID;

LONG Wow64CreateGuestThread(ULONG Eip, ULONG Esp, ULONG Eax, ULONG Ebx,
                            ULONG StackBase, ULONG StackLimit,
                            ULONG DeallocationStack,
                            BOOLEAN CreateSuspended,
                            HANDLE *ThreadHandle, WOW64_CLIENT_ID *ClientId);
VOID Wow64FreeThreadCpuArea(VOID);
NTSTATUS WINAPI NtTerminateThread(HANDLE ThreadHandle, NTSTATUS ExitStatus);
VOID WINAPI RtlExitUserThread(NTSTATUS Status);

/* ---- PEB64 process parameters for a child a 32-BIT parent created --------
 *
 * The mirror image of wow64.c's M3b down-conversion, and needed for the
 * opposite reason. When a NATIVE parent spawns an i386 child, its 64-bit
 * kernel32 writes 64-bit RTL_USER_PROCESS_PARAMETERS into PEB64, the child's
 * LdrpInitializeProcess reads them, and wow64.dll then DERIVES the 32-bit copy
 * for the guest. A 32-bit parent cannot do that first step: it can only build
 * a 32-bit block, and since Wow64pPublishRemotePeb32 it correctly writes that
 * into the child's PEB32. Nothing fills PEB64 -- and PEB64 is what runs first,
 * so the child died in LdrpInitializeProcess on its very first dereference,
 * `(*(*Peb).ProcessParameters).ImagePathName.Buffer`, before printing a line.
 *
 * So we widen the block the guest already wrote and point PEB64 at it. Note
 * what this does NOT do: no string bytes are copied. Every Buffer the guest
 * built already lives in the CHILD's address space, below 2GB, which a 64-bit
 * pointer addresses perfectly well -- so the 64-bit strings point at the very
 * same characters and only the fixed header is rebuilt. The environment block
 * is shared the same way.
 *
 * The output is marked NORMALIZED and carries absolute pointers, which is why
 * the denormalized case costs nothing extra: a denormalized Buffer is an
 * offset from the block base, so resolving it is one addition done here
 * instead of by the reader. Which fields that applies to is not a guess --
 * rtl-ppb's RtlDeNormalizeProcessParams adjusts exactly the eight
 * UNICODE_STRINGs below and deliberately leaves Environment alone, because the
 * environment is a separate allocation rather than part of the block.
 *
 * Layout MEASURED with the same offsetof-plus-control method as
 * BASE_CREATE_PROCESS, and the control EARNED ITS KEEP here: transcribing the
 * struct from wine/winternl.h gave sizeof 676/1040 against the 656/1008 that
 * ntdll-sys/src/size_check_{i386,amd64}.rs asserts. The five trailing fields
 * that header carries (EnvironmentSize, EnvironmentVersion,
 * PackageDependencyData, ProcessGroupId, LoaderThreads) are not in the struct
 * this tree builds against. Nothing above RuntimeData moved, so a derivation
 * would have produced correct field offsets and the WRONG BLOCK SIZE.
 *
 *   field                  i386    amd64          field        i386   amd64
 *   MaximumLength          0x00    0x00           Environment  0x48   0x80
 *   Length                 0x04    0x04           dwX..Show    0x4c   0x88
 *   Flags                  0x08    0x08           WindowTitle  0x70   0xb0
 *   DebugFlags             0x0c    0x0c           DesktopInfo  0x78   0xc0
 *   ConsoleHandle          0x10    0x10           ShellInfo    0x80   0xd0
 *   ConsoleFlags           0x14    0x18           RuntimeData  0x88   0xe0
 *   StandardInput          0x18    0x20           sizeof       0x290  0x3f0
 *   StandardOutput         0x1c    0x28
 *   StandardError          0x20    0x30
 *   CurDir.DosPath         0x24    0x38
 *   CurDir.Handle          0x2c    0x48
 *   DllPath                0x30    0x50
 *   ImagePathName          0x38    0x60
 *   CommandLine            0x40    0x70
 */
#define WOW64_UPP32_SIZE        0x290
#define WOW64_UPP64_SIZE        0x3f0
#define WOW64_UPP32_HDR         0x90   /* everything up to DLCurrentDirectory */
#define WOW64_PEB32_PARAMS_OFF  0x10
#define WOW64_PEB64_PARAMS_OFF  0x20
#define WOW64_PPF_NORMALIZED    0x00000001

static NTSTATUS Wow64pPublishPeb64Params(HANDLE Process, ULONG_PTR Peb32,
                                         ULONGLONG Peb64)
{
    unsigned char Hdr32[WOW64_UPP32_HDR];
    unsigned char Blk64[WOW64_UPP64_SIZE];
    ULONG Upp32 = 0;
    ULONG_PTR StrBase;
    PVOID Remote = NULL;
    SIZE_T Size = WOW64_UPP64_SIZE, Done = 0;
    ULONGLONG Remote64;
    NTSTATUS St;
    unsigned i;

    /* Where the guest put its own block. */
    St = NtReadVirtualMemory(Process,
                             (PVOID)(Peb32 + WOW64_PEB32_PARAMS_OFF),
                             &Upp32, sizeof(Upp32), &Done);
    if (St < 0 || Done != sizeof(Upp32) || !Upp32)
    {
        DbgPrint("[WOW64] PEB64 params: child PEB32->ProcessParameters is "
                 "empty (st=%08lx) -- nothing to widen\n", St);
        return (NTSTATUS)0xC000000D; /* STATUS_INVALID_PARAMETER */
    }

    St = NtReadVirtualMemory(Process, (PVOID)(ULONG_PTR)Upp32,
                             Hdr32, sizeof(Hdr32), &Done);
    if (St < 0 || Done != sizeof(Hdr32))
        return St < 0 ? St : (NTSTATUS)0xC0000018; /* CONFLICTING_ADDRESSES */

    /* A denormalized Buffer is an offset from the block base; a normalized one
     * is already absolute. Resolve to absolute either way, once. */
    StrBase = (*(ULONG *)(Hdr32 + 0x08) & WOW64_PPF_NORMALIZED)
                  ? 0 : (ULONG_PTR)Upp32;

    for (i = 0; i < sizeof(Blk64); i++)
        Blk64[i] = 0;

#define UPP_U32(o32, o64)  (*(ULONG *)(Blk64 + (o64)) = *(ULONG *)(Hdr32 + (o32)))
#define UPP_PTR(o32, o64)  (*(ULONGLONG *)(Blk64 + (o64)) =                    \
                            (ULONGLONG)*(ULONG *)(Hdr32 + (o32)))
/* UNICODE_STRING32{Length,Max,Buffer} -> UNICODE_STRING64{Length,Max,pad,Buffer}.
 * A zero Buffer stays zero rather than becoming StrBase, which would hand the
 * reader the block header as if it were a string. */
#define UPP_STR(o32, o64)                                                      \
    do {                                                                       \
        ULONG _b = *(ULONG *)(Hdr32 + (o32) + 4);                              \
        *(USHORT *)(Blk64 + (o64) + 0) = *(USHORT *)(Hdr32 + (o32) + 0);       \
        *(USHORT *)(Blk64 + (o64) + 2) = *(USHORT *)(Hdr32 + (o32) + 2);       \
        *(ULONGLONG *)(Blk64 + (o64) + 8) =                                    \
            _b ? (ULONGLONG)((ULONG_PTR)_b + StrBase) : 0ULL;                  \
    } while (0)

    /* The block describes itself as fully used: the strings it points at live
     * outside it, so there is no arena tail to account for. */
    *(ULONG *)(Blk64 + 0x00) = WOW64_UPP64_SIZE;   /* MaximumLength */
    *(ULONG *)(Blk64 + 0x04) = WOW64_UPP64_SIZE;   /* Length        */
    *(ULONG *)(Blk64 + 0x08) =
        *(ULONG *)(Hdr32 + 0x08) | WOW64_PPF_NORMALIZED;
    UPP_U32(0x0c, 0x0c);                            /* DebugFlags    */
    UPP_PTR(0x10, 0x10);                            /* ConsoleHandle */
    UPP_U32(0x14, 0x18);                            /* ConsoleFlags  */
    UPP_PTR(0x18, 0x20);                            /* StandardInput */
    UPP_PTR(0x1c, 0x28);                            /* StandardOutput*/
    UPP_PTR(0x20, 0x30);                            /* StandardError */
    UPP_STR(0x24, 0x38);                            /* CurDir.DosPath*/
    UPP_PTR(0x2c, 0x48);                            /* CurDir.Handle */
    UPP_STR(0x30, 0x50);                            /* DllPath       */
    UPP_STR(0x38, 0x60);                            /* ImagePathName */
    UPP_STR(0x40, 0x70);                            /* CommandLine   */
    UPP_PTR(0x48, 0x80);                            /* Environment   */
    for (i = 0; i < 9; i++)                         /* dwX..wShowWindow */
        UPP_U32(0x4c + i * 4, 0x88 + i * 4);
    UPP_STR(0x70, 0xb0);                            /* WindowTitle   */
    UPP_STR(0x78, 0xc0);                            /* DesktopInfo   */
    UPP_STR(0x80, 0xd0);                            /* ShellInfo     */
    UPP_STR(0x88, 0xe0);                            /* RuntimeData   */
#undef UPP_U32
#undef UPP_PTR
#undef UPP_STR

    St = NtAllocateVirtualMemory(Process, &Remote, 0, &Size,
                                 MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
    if (St < 0)
    {
        DbgPrint("[WOW64] PEB64 params: allocate in child failed %08lx\n", St);
        return St;
    }

    St = NtWriteVirtualMemory(Process, Remote, Blk64, sizeof(Blk64), &Done);
    if (St < 0 || Done != sizeof(Blk64))
        return St < 0 ? St : (NTSTATUS)0x8000000D; /* STATUS_PARTIAL_COPY */

    Remote64 = (ULONGLONG)(ULONG_PTR)Remote;
    St = NtWriteVirtualMemory(Process,
                              (PVOID)(ULONG_PTR)(Peb64 + WOW64_PEB64_PARAMS_OFF),
                              &Remote64, sizeof(Remote64), &Done);
    if (St < 0 || Done != sizeof(Remote64))
        return St < 0 ? St : (NTSTATUS)0x8000000D;

    DbgPrint("[WOW64] PEB64->ProcessParameters published at %p (widened from "
             "PEB32 block %08lx, strings %s)\n", Remote, Upp32,
             StrBase ? "denormalized" : "normalized");
    return 0;
}

LONG Wow64CreateForeignGuestThread(HANDLE Process, ULONGLONG Peb64,
                                   ULONG Eip, ULONG Esp, ULONG Eax, ULONG Ebx,
                                   ULONG StackBase, ULONG StackLimit,
                                   ULONG DeallocationStack,
                                   BOOLEAN CreateSuspended,
                                   HANDLE *ThreadHandle, WOW64_CLIENT_ID *ClientId);

/*
 * Wow64pCreateForeignThread — both halves of task #64.
 *
 * `NtCreateThread` with a ProcessHandle that is not -1 arrives here, and the two
 * callers that get there are not alike. The split is what made the gap
 * tractable: they were refused together on the harder one's reason.
 *
 *   CreateRemoteThread into a RUNNING process needs the target's wow64.dll to
 *   establish the guest side of a thread it did not create. In-process
 *   creation does that by handing RtlCreateUserThread a WOW64_THREAD_SEED on
 *   the local heap and Wow64pGuestThreadStart as the start routine; neither
 *   address means anything in another process, which is why that path cannot
 *   simply be pointed at a foreign handle. Wow64CreateForeignGuestThread
 *   (wow64.c) fixes both ends -- seed allocated in the target, start routine
 *   translated through the target's own wow64.dll base -- and in doing so it
 *   gets the target's code running the setup, on the new thread, which is the
 *   property Win10's Wow64ProcessPendingCrossProcessItems work list exists to
 *   provide. That is why the drain is still empty and does not need to be.
 *
 *   The INITIAL thread of a brand-new child needs none of it, because the child
 *   establishes its own guest side from data the kernel already wrote. The
 *   child's Wow64LdrpInitialize reads PEB32->ImageBaseAddress and takes the
 *   entry point out of the image's own NT headers (wow64.c); it never looks at
 *   a CONTEXT the parent supplied. And it is reached from LdrpInitializeProcess
 *   on the strength of ProcessWow64Information on ITSELF (ntdll-rust
 *   ldrinit.rs), so it does not matter who created the thread or how. The
 *   existence proof is that winlogon spawns all ten ladder guests this way
 *   today; the only difference for a 32-bit parent is that its NtCreateThread
 *   passes through this thunk, where a native parent's does not.
 *
 * So the guest's CONTEXT32/INITIAL_TEB32 are deliberately IGNORED on this path.
 * They describe a 32-bit start (kernel32!BaseProcessStartThunk with the entry in
 * Eax and PEB32 in Ebx) that the child will recompute for itself. The guest
 * stack its kernel32 already allocated cross-process in BaseCreateStack is left
 * where it is -- unused, in the child's own address space, freed when the child
 * dies. Reproducing that allocation here would not make it any more used.
 *
 * Two conditions, and both are load-bearing rather than defensive:
 *
 *   The target must be a WoW64 process. A 32-bit parent spawning a SIXTY-FOUR-bit
 *   child is a real case and a different one -- nothing would hand that child's
 *   initial thread a guest context because it does not want one -- and it stays
 *   refused rather than silently taking this path.
 *
 *   The target must not have started. PEB->Ldr is NULL until the child's own
 *   LdrpInitializeProcess fills it, so a NULL there is the kernel's word for
 *   "this process has not run any user code yet", which is exactly the
 *   precondition the paragraph above relies on. A running target falls through
 *   to the refusal, which is the CreateRemoteThread case and still owed.
 */
static NTSTATUS Wow64pCreateForeignThread(HANDLE Process,
                                          BOOLEAN CreateSuspended,
                                          PBYTE Ctx32, PBYTE ITeb32,
                                          ULONG *Handle32, ULONG *ClientId32)
{
    WOW64_PROCESS_BASIC_INFORMATION64 Pbi = {0};
    ULONG_PTR TargetPeb32 = 0;
    ULONGLONG Ldr = (ULONGLONG)-1;
    SIZE_T Read = 0;
    HANDLE Thread = NULL;
    WOW64_CLIENT_ID Cid = { NULL, NULL };
    NTSTATUS St;

    St = NtQueryInformationProcess(Process, ProcessWow64Information,
                                   &TargetPeb32, sizeof(TargetPeb32), NULL);
    if (St < 0 || !TargetPeb32)
    {
        DbgPrint("[WOW64] NtCreateThread: REFUSED - cross-process into a "
                 "non-WoW64 target (ProcessHandle=%p st=%08lx) (task #64).\n",
                 Process, St);
        return (NTSTATUS)0xC0000002; /* STATUS_NOT_IMPLEMENTED */
    }

    St = NtQueryInformationProcess(Process, ProcessBasicInformation,
                                   &Pbi, sizeof(Pbi), NULL);
    if (St >= 0 && Pbi.PebBaseAddress)
    {
        /* PEB64.Ldr is at +0x18 on every 64-bit host we build for. */
        St = NtReadVirtualMemory(Process,
                                 (PVOID)(ULONG_PTR)(Pbi.PebBaseAddress + 0x18),
                                 &Ldr, sizeof(Ldr), &Read);
        if (St < 0 || Read != sizeof(Ldr))
            Ldr = (ULONGLONG)-1; /* unreadable: treat as "already running" */
    }

    if (Ldr != 0)
    {
        /* A RUNNING target: this is CreateRemoteThread, and unlike the initial
         * thread it really does need the target's wow64.dll to establish the
         * guest side. It gets it by BEING the start routine -- see
         * Wow64CreateForeignGuestThread. The guest's CONTEXT32 describes a
         * 32-bit thread start exactly as it does in-process, and the guest
         * stack in ITeb32 was allocated cross-process by the caller's
         * BaseCreateStack, so both already refer to the target. */
        if (!Ctx32 || !ITeb32)
            return (NTSTATUS)0xC000000D; /* STATUS_INVALID_PARAMETER */
        St = Wow64CreateForeignGuestThread(
                 Process, Pbi.PebBaseAddress,
                 *(ULONG *)(Ctx32 + WOW64_CTX32_EIP),
                 *(ULONG *)(Ctx32 + WOW64_CTX32_ESP),
                 *(ULONG *)(Ctx32 + WOW64_CTX32_EAX),
                 *(ULONG *)(Ctx32 + WOW64_CTX32_EBX),
                 *(ULONG *)(ITeb32 + WOW64_ITEB32_STACKBASE),
                 *(ULONG *)(ITeb32 + WOW64_ITEB32_STACKLIMIT),
                 *(ULONG *)(ITeb32 + WOW64_ITEB32_ALLOCBASE),
                 CreateSuspended, &Thread, &Cid);
        if (St < 0)
            return St;
        if (Handle32)
            *Handle32 = (ULONG)(ULONG_PTR)Thread;
        if (ClientId32)
        {
            ClientId32[0] = (ULONG)(ULONG_PTR)Cid.UniqueProcess;
            ClientId32[1] = (ULONG)(ULONG_PTR)Cid.UniqueThread;
        }
        return St;
    }

    /* The child's 64-bit loader runs before its wow64.dll and reads PEB64's
     * process parameters, which only a 64-bit parent would have filled. Widen
     * the block the guest wrote into PEB32 and point PEB64 at it, BEFORE the
     * thread exists -- once it is running the loader has already dereferenced
     * that pointer. A failure here is fatal to the child, so it fails the
     * creation rather than starting a process that cannot survive its own
     * first dereference. */
    St = Wow64pPublishPeb64Params(Process, (ULONG_PTR)TargetPeb32,
                                  Pbi.PebBaseAddress);
    if (St < 0)
    {
        DbgPrint("[WOW64] NtCreateThread: PEB64 params not published %08lx -- "
                 "refusing rather than starting a child that cannot load\n", St);
        return St;
    }

    /* StartAddress is never reached: the child's LdrpInitializeProcess diverts
     * into Wow64LdrpInitialize, which ends in BTCpuSimulate and does not
     * return. RtlExitUserThread is named anyway so that if that assumption is
     * ever wrong the thread exits instead of running whatever is at the
     * address -- the failure that costs a bugcheck is the one worth aiming. */
    St = RtlCreateUserThread(Process, NULL, CreateSuspended, 0, 0, 0,
                             (PVOID)RtlExitUserThread, NULL, &Thread, &Cid);
    if (St < 0)
    {
        DbgPrint("[WOW64] NtCreateThread: cross-process initial thread failed "
                 "%08lx\n", St);
        return St;
    }

    if (Handle32)
        *Handle32 = (ULONG)(ULONG_PTR)Thread;
    if (ClientId32)
    {
        ClientId32[0] = (ULONG)(ULONG_PTR)Cid.UniqueProcess;
        ClientId32[1] = (ULONG)(ULONG_PTR)Cid.UniqueThread;
    }
    DbgPrint("[WOW64] NtCreateThread: cross-process INITIAL thread pid=%lx "
             "tid=%lx suspended=%u (task #64 initial-thread half)\n",
             (ULONG)(ULONG_PTR)Cid.UniqueProcess,
             (ULONG)(ULONG_PTR)Cid.UniqueThread, CreateSuspended);
    return St;
}

/*
 * NtTerminateThread — pass-through, except that a thread terminating ITSELF is
 * the last moment wow64.dll holds control on it: the kernel call never returns,
 * and heap memory is not reclaimed by thread exit. Without this, every guest
 * thread leaks its CPU area (WOW64_CPURESERVED + a WOW64_CONTEXT) for the life
 * of the process. The guest's own exit path lands here — i386
 * RtlExitUserThread ends in NtTerminateThread(NULL, status) — and there is no
 * native DLL_THREAD_DETACH to hook, because that call kills the native thread
 * outright without running the 64-bit loader's shutdown.
 */
static NTSTATUS Wow64Manual_NtTerminateThread(ULONG *a)
{
    HANDLE Thread = (HANDLE)(LONG_PTR)(LONG)a[0];
    NTSTATUS St;

    /* Terminating some OTHER thread: nothing of ours on this one to release,
     * and we must not touch the target's CPU area from here (it may be mid
     * mode-switch). Its area is released when it runs its own exit path. */
    if (Thread && Thread != (HANDLE)(LONG_PTR)-2)
        return NtTerminateThread(Thread, (NTSTATUS)a[1]);

    /* Safe to free here: we are host-side inside the guest's own syscall, so
     * the CPU layer is not reading the area, and control never comes back. */
    Wow64FreeThreadCpuArea();
    St = NtTerminateThread(Thread, (NTSTATUS)a[1]);

    /* Returned from a self-terminate. The CPU area is gone, so returning into
     * the dispatcher would have it write the syscall result into freed memory.
     * Say so and make sure the thread dies instead of limping on. */
    DbgPrint("[WOW64] NtTerminateThread: self-terminate returned %08lx - forcing "
             "thread exit (CPU area already released)\n", St);
    RtlExitUserThread(St);
    return St;
}

/*
 * NtCreateThread — #64: real 32-bit guest thread creation.
 *
 * The flat thunk this replaces returned STATUS_SUCCESS while handing the 64-bit
 * kernel an i386 CONTEXT/INITIAL_TEB/OBJECT_ATTRIBUTES/CLIENT_ID reinterpreted at
 * host width (5 divergent params). The kernel then built a thread whose trap
 * frame was effectively zeroed, and the very first user APC to it computed
 * (Rsp=0) - sizeof(UAPC_FRAME) and, before the #67 fix, BUGCHECKed the machine.
 * Observed for real: osk.exe, the ONLY NtCreateThread in a whole amd64 boot.
 *
 * Marshalling the five structs would NOT have been enough, which is why this
 * spent a while as a loud refusal: the host kernel cannot build a 32-bit thread
 * at all. What it CAN build is a native thread, which is what we do — the guest
 * side of it is then established by wow64.dll's per-thread entry, exactly as
 * process startup establishes the initial thread.
 *
 * So the guest's CONTEXT32 is not marshalled to a host CONTEXT; it is READ as
 * the description of a guest thread it is:
 *   Eip = kernel32's BaseThreadStartupThunk, Esp = the guest stack it already
 *   allocated in BaseCreateStack, Eax = the thread routine, Ebx = its parameter
 * (kernel32!BaseInitializeContext, i386 branch). INITIAL_TEB32 carries the
 * guest stack bounds that TEB32's NtTib needs.
 */
static NTSTATUS Wow64Manual_NtCreateThread(ULONG *a)
{
    ULONG *Handle32   = (ULONG *)(ULONG_PTR)a[0];
    ULONG *ClientId32 = (ULONG *)(ULONG_PTR)a[4];
    PBYTE  Ctx32      = (PBYTE)(ULONG_PTR)a[5];
    PBYTE  ITeb32     = (PBYTE)(ULONG_PTR)a[6];
    HANDLE Thread = NULL;
    WOW64_CLIENT_ID Cid = { NULL, NULL };
    NTSTATUS St;

    /* Cross-process creation splits in two -- a brand-new child's initial
     * thread and CreateRemoteThread into a running one -- and they need
     * different things. See Wow64pCreateForeignThread. */
    if ((HANDLE)(LONG_PTR)(LONG)a[3] != (HANDLE)(LONG_PTR)-1)
        return Wow64pCreateForeignThread((HANDLE)(LONG_PTR)(LONG)a[3],
                                        (BOOLEAN)(a[7] & 0xFF),
                                        Ctx32, ITeb32, Handle32, ClientId32);

    if (!Ctx32 || !ITeb32)
        return (NTSTATUS)0xC000000D; /* STATUS_INVALID_PARAMETER */

    St = Wow64CreateGuestThread(*(ULONG *)(Ctx32 + WOW64_CTX32_EIP),
                                *(ULONG *)(Ctx32 + WOW64_CTX32_ESP),
                                *(ULONG *)(Ctx32 + WOW64_CTX32_EAX),
                                *(ULONG *)(Ctx32 + WOW64_CTX32_EBX),
                                *(ULONG *)(ITeb32 + WOW64_ITEB32_STACKBASE),
                                *(ULONG *)(ITeb32 + WOW64_ITEB32_STACKLIMIT),
                                *(ULONG *)(ITeb32 + WOW64_ITEB32_ALLOCBASE),
                                (BOOLEAN)(a[7] & 0xFF),
                                &Thread, &Cid);
    if (St < 0)
        return St;

    if (Handle32)
        *Handle32 = (ULONG)(ULONG_PTR)Thread;
    if (ClientId32)
    {
        /* CLIENT_ID32: two 4-byte ids, against the host's two 8-byte ones. */
        ClientId32[0] = (ULONG)(ULONG_PTR)Cid.UniqueProcess;
        ClientId32[1] = (ULONG)(ULONG_PTR)Cid.UniqueThread;
    }
    DbgPrint("[WOW64] NtCreateThread: guest thread tid=%lx eip=%08lx esp=%08lx "
             "routine=%08lx suspended=%lu\n",
             (ULONG)(ULONG_PTR)Cid.UniqueThread,
             *(ULONG *)(Ctx32 + WOW64_CTX32_EIP),
             *(ULONG *)(Ctx32 + WOW64_CTX32_ESP),
             *(ULONG *)(Ctx32 + WOW64_CTX32_EAX), a[7] & 0xFF);
    return St;
}

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

    /* The out-cell class — every one of these used to hand the kernel a 4-byte
     * guest cell for a pointer-sized store. Driven off `marshalgen audit`. */
    if (WOW64_IS("NtOpenProcessToken"))
        return Wow64Manual_NtOpenProcessToken(Arguments);
    if (WOW64_IS("NtOpenThreadToken"))
        return Wow64Manual_NtOpenThreadToken(Arguments);
    if (WOW64_IS("NtDuplicateObject"))
        return Wow64Manual_NtDuplicateObject(Arguments);
    if (WOW64_IS("NtDuplicateToken"))
        return Wow64Manual_NtDuplicateToken(Arguments);
    if (WOW64_IS("NtCreateEventPair"))
        return Wow64pOpenByOa(NtCreateEventPair, Arguments);
    if (WOW64_IS("NtOpenEventPair"))
        return Wow64pOpenByOa(NtOpenEventPair, Arguments);
    if (WOW64_IS("NtCreateJobObject"))
        return Wow64pOpenByOa(NtCreateJobObject, Arguments);
    if (WOW64_IS("NtOpenJobObject"))
        return Wow64pOpenByOa(NtOpenJobObject, Arguments);
    if (WOW64_IS("NtOpenTimer"))
        return Wow64pOpenByOa(NtOpenTimer, Arguments);
    if (WOW64_IS("NtCreateTimer"))
        return Wow64pOpenByOaU32(NtCreateTimer, Arguments);
    if (WOW64_IS("NtCreateDebugObject"))
        return Wow64pOpenByOaU32(NtCreateDebugObject, Arguments);
    if (WOW64_IS("NtCreatePort"))
        return Wow64pCreatePort(NtCreatePort, Arguments);
    if (WOW64_IS("NtCreateWaitablePort"))
        return Wow64pCreatePort(NtCreateWaitablePort, Arguments);
    if (WOW64_IS("NtCreateProcess"))
        return Wow64Manual_NtCreateProcess(Arguments);
    if (WOW64_IS("NtCreateProcessEx"))
        return Wow64Manual_NtCreateProcessEx(Arguments);
    if (WOW64_IS("NtCreateProfile"))
        return Wow64Manual_NtCreateProfile(Arguments);
    if (WOW64_IS("NtLockVirtualMemory"))
        return Wow64pLockUnlockVm(NtLockVirtualMemory, Arguments);
    if (WOW64_IS("NtUnlockVirtualMemory"))
        return Wow64pLockUnlockVm(NtUnlockVirtualMemory, Arguments);
    if (WOW64_IS("NtFlushVirtualMemory"))
        return Wow64Manual_NtFlushVirtualMemory(Arguments);

    /* Bare PUNICODE_STRING / POBJECT_ATTRIBUTES args — flat-thunked, the kernel
     * read the string pointer from PAST THE END of the guest's 8-byte struct.
     * NtSetValueKey is the hot one: it BUGCHECKs the box (0x1E) on the garbage
     * pointer, so every RegSetValueEx from a 32-bit guest was a local DoS. */
    if (WOW64_IS("NtSetValueKey"))
        return Wow64Manual_NtSetValueKey(Arguments);
    if (WOW64_IS("NtDeleteValueKey"))
        return Wow64Manual_NtDeleteValueKey(Arguments);
    if (WOW64_IS("NtRenameKey"))
        return Wow64Manual_NtRenameKey(Arguments);
    if (WOW64_IS("NtDeleteFile"))
        return Wow64pCallOa1(NtDeleteFile, Arguments);
    if (WOW64_IS("NtUnloadKey"))
        return Wow64pCallOa1(NtUnloadKey, Arguments);
    if (WOW64_IS("NtDisplayString"))
        return Wow64pCallUstr1(NtDisplayString, Arguments);
    if (WOW64_IS("NtLoadDriver"))
        return Wow64pCallUstr1(NtLoadDriver, Arguments);
    if (WOW64_IS("NtUnloadDriver"))
        return Wow64pCallUstr1(NtUnloadDriver, Arguments);
    if (WOW64_IS("NtQuerySystemEnvironmentValue"))
        return Wow64Manual_NtQuerySystemEnvironmentValue(Arguments);
    if (WOW64_IS("NtSetSystemEnvironmentValue"))
        return Wow64Manual_NtSetSystemEnvironmentValue(Arguments);
    if (WOW64_IS("NtQuerySystemEnvironmentValueEx"))
        return Wow64Manual_NtQuerySystemEnvironmentValueEx(Arguments);
    if (WOW64_IS("NtSetSystemEnvironmentValueEx"))
        return Wow64Manual_NtSetSystemEnvironmentValueEx(Arguments);
    if (WOW64_IS("NtCreatePagingFile"))
        return Wow64Manual_NtCreatePagingFile(Arguments);
    if (WOW64_IS("NtLoadKey"))
        return Wow64Manual_NtLoadKey(Arguments);
    if (WOW64_IS("NtLoadKey2"))
        return Wow64Manual_NtLoadKey2(Arguments);
    if (WOW64_IS("NtLoadKeyEx"))
        return Wow64Manual_NtLoadKeyEx(Arguments);
    if (WOW64_IS("NtReplaceKey"))
        return Wow64Manual_NtReplaceKey(Arguments);
    if (WOW64_IS("NtUnloadKey2"))
        return Wow64Manual_NtUnloadKey2(Arguments);
    if (WOW64_IS("NtUnloadKeyEx"))
        return Wow64Manual_NtUnloadKeyEx(Arguments);
    if (WOW64_IS("NtQueryOpenSubKeys"))
        return Wow64Manual_NtQueryOpenSubKeys(Arguments);
    if (WOW64_IS("NtQueryOpenSubKeysEx"))
        return Wow64Manual_NtQueryOpenSubKeysEx(Arguments);
    if (WOW64_IS("NtAccessCheckAndAuditAlarm"))
        return Wow64Manual_NtAccessCheckAndAuditAlarm(Arguments);
    if (WOW64_IS("NtOpenObjectAuditAlarm"))
        return Wow64Manual_NtOpenObjectAuditAlarm(Arguments);
    if (WOW64_IS("NtCloseObjectAuditAlarm"))
        return Wow64pAuditClose(NtCloseObjectAuditAlarm, Arguments);
    if (WOW64_IS("NtDeleteObjectAuditAlarm"))
        return Wow64pAuditClose(NtDeleteObjectAuditAlarm, Arguments);
    if (WOW64_IS("NtPrivilegeObjectAuditAlarm"))
        return Wow64Manual_NtPrivilegeObjectAuditAlarm(Arguments);
    if (WOW64_IS("NtPrivilegedServiceAuditAlarm"))
        return Wow64Manual_NtPrivilegedServiceAuditAlarm(Arguments);
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
    if (WOW64_IS("NtUnmapViewOfSection"))
        return Wow64Manual_NtUnmapViewOfSection(Arguments);
    if (WOW64_IS("NtReadVirtualMemory"))
        return Wow64Manual_NtReadVirtualMemory(Arguments);
    if (WOW64_IS("NtWriteVirtualMemory"))
        return Wow64Manual_NtWriteVirtualMemory(Arguments);
    if (WOW64_IS("NtFlushInstructionCache"))
        return Wow64Manual_NtFlushInstructionCache(Arguments);
    if (WOW64_IS("NtSetInformationProcess"))
        return Wow64Manual_NtSetInformationProcess(Arguments);
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
    if (WOW64_IS("NtRequestWaitReplyPort"))
        return Wow64Manual_NtRequestWaitReplyPort(Arguments);
    if (WOW64_IS("NtConnectPort"))
        return Wow64Manual_NtConnectPort(Arguments);
    if (WOW64_IS("NtContinue"))
        return Wow64Manual_NtContinue(Arguments);
    if (WOW64_IS("NtRaiseException"))
        return Wow64Manual_NtRaiseException(Arguments);
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
    if (WOW64_IS("NtUserBuildPropList"))
        return Wow64Manual_NtUserBuildPropList(Arguments);
    if (WOW64_IS("NtUserSetWindowLong"))
        return Wow64Manual_NtUserSetWindowLong(Arguments);
    if (WOW64_IS("NtUserSetClassLong"))
        return Wow64Manual_NtUserSetClassLong(Arguments);
    if (WOW64_IS("NtUserGetCPD"))
        return Wow64Manual_NtUserGetCPD(Arguments);
    if (WOW64_IS("NtUserRegisterClassExWOW"))
        return Wow64Manual_NtUserRegisterClassExWOW(Arguments);
    if (WOW64_IS("NtUserGetClassInfo"))
        return Wow64Manual_NtUserGetClassInfo(Arguments);
    if (WOW64_IS("NtUserGetAtomName"))
        return Wow64Manual_NtUserGetAtomName(Arguments);
    if (WOW64_IS("NtUserGetClassName"))
        return Wow64Manual_NtUserGetClassName(Arguments);
    if (WOW64_IS("NtUserRegisterWindowMessage"))
        return Wow64Manual_NtUserRegisterWindowMessage(Arguments);
    if (WOW64_IS("NtUserGetKeyboardLayoutName"))
        return Wow64Manual_NtUserGetKeyboardLayoutName(Arguments);
    if (WOW64_IS("NtUserUnregisterClass"))
        return Wow64Manual_NtUserUnregisterClass(Arguments);
    if (WOW64_IS("NtUserFindWindowEx"))
        return Wow64Manual_NtUserFindWindowEx(Arguments);
    if (WOW64_IS("NtUserGetWOWClass"))
        return Wow64Manual_NtUserGetWOWClass(Arguments);
    if (WOW64_IS("NtUserSetWindowsHookEx"))
        return Wow64Manual_NtUserSetWindowsHookEx(Arguments);
    if (WOW64_IS("NtUserSetWinEventHook"))
        return Wow64Manual_NtUserSetWinEventHook(Arguments);
    if (WOW64_IS("NtUserRegisterUserApiHook"))
        return Wow64Manual_NtUserRegisterUserApiHook(Arguments);
    if (WOW64_IS("NtUserDrawCaptionTemp"))
        return Wow64Manual_NtUserDrawCaptionTemp(Arguments);
    if (WOW64_IS("NtUserLoadKeyboardLayoutEx"))
        return Wow64Manual_NtUserLoadKeyboardLayoutEx(Arguments);
    if (WOW64_IS("NtUserEnumDisplaySettings"))
        return Wow64Manual_NtUserEnumDisplaySettings(Arguments);
    if (WOW64_IS("NtUserEnumDisplayDevices"))
        return Wow64Manual_NtUserEnumDisplayDevices(Arguments);
    /* E10 (marshalgen audit) */
    if (WOW64_IS("NtGdiCreateDIBSection"))
        return Wow64Manual_NtGdiCreateDIBSection(Arguments);
    if (WOW64_IS("NtUserTranslateAccelerator"))
        return Wow64Manual_NtUserTranslateAccelerator(Arguments);
    if (WOW64_IS("NtUserCallMsgFilter"))
        return Wow64Manual_NtUserCallMsgFilter(Arguments);
    if (WOW64_IS("NtUserRealInternalGetMessage"))
        return Wow64Manual_NtUserRealInternalGetMessage(Arguments);
    if (WOW64_IS("NtUserResolveDesktop"))
        return Wow64Manual_NtUserResolveDesktop(Arguments);
    if (WOW64_IS("NtUserOpenDesktop"))
        return Wow64Manual_NtUserOpenDesktop(Arguments);
    if (WOW64_IS("NtUserOpenWindowStation"))
        return Wow64Manual_NtUserOpenWindowStation(Arguments);
    if (WOW64_IS("NtUserCreateWindowStation"))
        return Wow64Manual_NtUserCreateWindowStation(Arguments);
    if (WOW64_IS("NtUserBuildHwndList"))
        return Wow64Manual_NtUserBuildHwndList(Arguments);
    if (WOW64_IS("NtUserThunkedMenuItemInfo"))
        return Wow64Manual_NtUserThunkedMenuItemInfo(Arguments);
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
    if (WOW64_IS("NtUserGetIconInfo"))
        return Wow64Manual_NtUserGetIconInfo(Arguments);
    if (WOW64_IS("NtGdiExtGetObjectW"))
        return Wow64Manual_NtGdiExtGetObjectW(Arguments);
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
    if (WOW64_IS("NtCreateThread"))
        return Wow64Manual_NtCreateThread(Arguments);
    if (WOW64_IS("NtWaitForMultipleObjects"))
        return Wow64Manual_NtWaitForMultipleObjects(Arguments);
    if (WOW64_IS("NtTerminateThread"))
        return Wow64Manual_NtTerminateThread(Arguments);
    if (WOW64_IS("NtUserInitializeClientPfnArrays"))
        return Wow64Manual_NtUserInitializeClientPfnArrays(Arguments);
    if (WOW64_IS("NtOpenProcessTokenEx"))
        return Wow64Manual_NtOpenProcessTokenEx(Arguments);
    if (WOW64_IS("NtOpenThreadTokenEx"))
        return Wow64Manual_NtOpenThreadTokenEx(Arguments);
    if (WOW64_IS("NtGetContextThread"))
        return Wow64Manual_NtGetContextThread(Arguments);
    if (WOW64_IS("NtSetContextThread"))
        return Wow64Manual_NtSetContextThread(Arguments);

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
    if (WOW64_IS("NtCreateIoCompletion"))
        return Wow64Manual_NtCreateIoCompletion(Arguments);
    if (WOW64_IS("NtOpenIoCompletion"))
        return Wow64pOpenByOa(NtOpenIoCompletion, Arguments);
    if (WOW64_IS("NtQueryInformationThread"))
        return Wow64Manual_NtQueryInformationThread(Arguments);
    if (WOW64_IS("NtRemoveIoCompletion"))
        return Wow64Manual_NtRemoveIoCompletion(Arguments);
    if (WOW64_IS("NtUserEnumDisplayMonitors"))
        return Wow64Manual_NtUserEnumDisplayMonitors(Arguments);
    if (WOW64_IS("NtUserSendInput"))
        return Wow64Manual_NtUserSendInput(Arguments);
    if (WOW64_IS("NtUserThunkedMenuInfo"))
        return Wow64Manual_NtUserThunkedMenuInfo(Arguments);
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
