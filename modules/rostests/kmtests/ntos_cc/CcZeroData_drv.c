/*
 * PROJECT:         ReactOS kernel-mode tests
 * LICENSE:         LGPLv2.1+ - See COPYING.LIB in the top level directory
 * PURPOSE:         Test driver for CcZeroData's non-cached path
 *
 * A file with no shared cache map makes CcZeroData write the range itself, in
 * MAX_ZERO_LENGTH (256 KiB) paging writes of the zero page. This driver is that
 * file's file system: it counts those writes and checks each one, without storing
 * anything, so a range of 4 GiB and more costs ~16K no-op IRPs instead of 4 GiB of
 * I/O. That is the range the Rust port broke: it narrowed the LONGLONG length to a
 * ULONG before taking the min, so the chunk for a 4 GiB remainder was 0 bytes and
 * `Length -= 0` never ended. A zero-length write is failed here, so that regression
 * reports FALSE instead of hanging the sweep.
 */

#include <kmt_test.h>

#define NDEBUG
#include <debug.h>

#define ZERO_CHUNK (256 * 1024)

typedef struct _TEST_FCB
{
    SECTION_OBJECT_POINTERS SectionObjectPointers;
} TEST_FCB, *PTEST_FCB;

static KMT_IRP_HANDLER TestIrpHandler;

/* What the non-cached writes of one CcZeroData call looked like. */
static struct
{
    BOOLEAN Recording;
    ULONG Writes;
    ULONG ZeroLength;
    ULONG BadOffset;
    ULONG BadMdl;
    ULONG LastLength;
    LONGLONG NextOffset;
    LONGLONG Total;
} Seen;

NTSTATUS
TestEntry(
    _In_ PDRIVER_OBJECT DriverObject,
    _In_ PCUNICODE_STRING RegistryPath,
    _Out_ PCWSTR *DeviceName,
    _Inout_ INT *Flags)
{
    PAGED_CODE();

    UNREFERENCED_PARAMETER(DriverObject);
    UNREFERENCED_PARAMETER(RegistryPath);

    *DeviceName = L"CcZeroData";
    *Flags = TESTENTRY_NO_EXCLUSIVE_DEVICE |
             TESTENTRY_BUFFERED_IO_DEVICE |
             TESTENTRY_NO_READONLY_DEVICE;

    KmtRegisterIrpHandler(IRP_MJ_CLEANUP, NULL, TestIrpHandler);
    KmtRegisterIrpHandler(IRP_MJ_CREATE, NULL, TestIrpHandler);
    KmtRegisterIrpHandler(IRP_MJ_WRITE, NULL, TestIrpHandler);

    return STATUS_SUCCESS;
}

VOID
TestUnload(
    _In_ PDRIVER_OBJECT DriverObject)
{
    PAGED_CODE();
}

static
VOID
ZeroRange(
    _In_ PFILE_OBJECT FileObject,
    _In_ LONGLONG Start,
    _In_ LONGLONG End,
    _In_ ULONG ExpectWrites,
    _In_ ULONG ExpectLast)
{
    LARGE_INTEGER StartOffset, EndOffset;
    BOOLEAN Ret;

    RtlZeroMemory(&Seen, sizeof(Seen));
    Seen.NextOffset = Start;
    Seen.Recording = TRUE;

    StartOffset.QuadPart = Start;
    EndOffset.QuadPart = End;
    Ret = CcZeroData(FileObject, &StartOffset, &EndOffset, TRUE);

    Seen.Recording = FALSE;
    ok(Ret == TRUE, "CcZeroData(%I64x..%I64x) returned %u\n", Start, End, Ret);
    ok_eq_ulong(Seen.ZeroLength, 0UL);
    ok_eq_ulong(Seen.BadOffset, 0UL);
    ok_eq_ulong(Seen.BadMdl, 0UL);
    ok_eq_ulong(Seen.Writes, ExpectWrites);
    ok_eq_ulong(Seen.LastLength, ExpectLast);
    ok_eq_longlong(Seen.Total, End - Start);
}

static
VOID
Test_CcZeroData(
    _In_ PFILE_OBJECT FileObject)
{
    const LONGLONG FourGiB = 0x100000000LL;

    /* Small: one full chunk and a tail */
    ZeroRange(FileObject, 0, ZERO_CHUNK + 44 * 1024, 2, 44 * 1024);
    /* Offsets above 4 GiB travel as 64 bits */
    ZeroRange(FileObject, FourGiB, FourGiB + 8192, 1, 8192);
    /* Exactly 4 GiB: narrowed to a ULONG this is 0 */
    ZeroRange(FileObject, 0, FourGiB, 16384, ZERO_CHUNK);
    /* 4 GiB + 64 KiB: narrowed, the first chunk is 64 KiB and the 4 GiB left is 0 */
    ZeroRange(FileObject, 0, FourGiB + 65536, 16385, 65536);
}

static
NTSTATUS
TestIrpHandler(
    _In_ PDEVICE_OBJECT DeviceObject,
    _In_ PIRP Irp,
    _In_ PIO_STACK_LOCATION IoStack)
{
    NTSTATUS Status = STATUS_NOT_SUPPORTED;
    PTEST_FCB Fcb;

    UNREFERENCED_PARAMETER(DeviceObject);

    Irp->IoStatus.Information = 0;

    if (IoStack->MajorFunction == IRP_MJ_CREATE)
    {
        /* No CcInitializeCacheMap: SharedCacheMap stays NULL, so CcZeroData goes non-cached */
        Fcb = ExAllocatePoolWithTag(NonPagedPool, sizeof(*Fcb), 'FzcC');
        if (Fcb)
        {
            RtlZeroMemory(Fcb, sizeof(*Fcb));
            IoStack->FileObject->FsContext = Fcb;
            IoStack->FileObject->SectionObjectPointer = &Fcb->SectionObjectPointers;
            Irp->IoStatus.Information = FILE_OPENED;
            Status = STATUS_SUCCESS;
        }
        else
        {
            Status = STATUS_INSUFFICIENT_RESOURCES;
        }
    }
    else if (IoStack->MajorFunction == IRP_MJ_WRITE && !FlagOn(Irp->Flags, IRP_NOCACHE))
    {
        /* The user-mode part's write: run the test from a real open file object */
        ok_irql(PASSIVE_LEVEL);
        Test_CcZeroData(IoStack->FileObject);
        Irp->IoStatus.Information = IoStack->Parameters.Write.Length;
        Status = STATUS_SUCCESS;
    }
    else if (IoStack->MajorFunction == IRP_MJ_WRITE)
    {
        /* One of CcZeroData's paging writes: count it, never touch the data */
        ULONG Length = IoStack->Parameters.Write.Length;
        LONGLONG Offset = IoStack->Parameters.Write.ByteOffset.QuadPart;

        if (!Seen.Recording || Length == 0 || Seen.Writes > 20000)
        {
            Seen.ZeroLength += (Length == 0);
            Status = STATUS_INVALID_PARAMETER;
        }
        else
        {
            Seen.Writes++;
            Seen.BadOffset += (Offset != Seen.NextOffset);
            Seen.BadMdl += (Irp->MdlAddress == NULL ||
                            MmGetMdlByteCount(Irp->MdlAddress) != Length ||
                            Length > ZERO_CHUNK ||
                            !FlagOn(Irp->Flags, IRP_PAGING_IO));
            Seen.NextOffset = Offset + Length;
            Seen.Total += Length;
            Seen.LastLength = Length;
            Irp->IoStatus.Information = Length;
            Status = STATUS_SUCCESS;
        }
    }
    else if (IoStack->MajorFunction == IRP_MJ_CLEANUP)
    {
        Fcb = IoStack->FileObject->FsContext;
        if (Fcb)
            ExFreePoolWithTag(Fcb, 'FzcC');
        IoStack->FileObject->FsContext = NULL;
        Status = STATUS_SUCCESS;
    }

    Irp->IoStatus.Status = Status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    return Status;
}
