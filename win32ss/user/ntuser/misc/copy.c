#include "win32k.h"
#include <safe_user_io.h>

_IRQL_requires_max_(APC_LEVEL)
NTSTATUS
_MmCopyFromCaller(
    _Out_writes_bytes_all_(Bytes) PVOID Target,
    _In_reads_bytes_(Bytes) PVOID Source,
    _In_ UINT Bytes)
{
    NTSTATUS Status;

    PAGED_CODE();
    ASSERT(ExGetPreviousMode() == UserMode);

    Status = STATUS_SUCCESS;
    Status = SafeProbeForRead(Source, Bytes, 1);
    if (NT_SUCCESS(Status))
    {
        RtlCopyMemory(Target, Source, Bytes);
    }

    return Status;
}

_IRQL_requires_max_(APC_LEVEL)
NTSTATUS
_MmCopyToCaller(
    _Out_writes_bytes_all_(Bytes) PVOID Target,
    _In_reads_bytes_(Bytes) PVOID Source,
    _In_ UINT Bytes)
{
    NTSTATUS Status;

    PAGED_CODE();
    ASSERT(ExGetPreviousMode() == UserMode);

    Status = STATUS_SUCCESS;
    Status = SafeProbeForWrite(Target, Bytes, 1);
    if (NT_SUCCESS(Status))
    {
        RtlCopyMemory(Target, Source, Bytes);
    }

    return Status;
}
