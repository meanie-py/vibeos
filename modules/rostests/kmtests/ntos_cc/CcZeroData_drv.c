/*
 * PROJECT:         ReactOS kernel-mode tests
 * LICENSE:         LGPLv2.1+ - See COPYING.LIB in the top level directory
 * PURPOSE:         Test driver for CcZeroData's non-cached and cached paths
 *
 * A file with no shared cache map makes CcZeroData write the range itself, in
 * MAX_ZERO_LENGTH (256 KiB) paging writes of the zero page. This driver is that
 * file's file system: it counts those writes and checks each one, without storing
 * anything, so a range of 4 GiB and more costs ~16K no-op IRPs instead of 4 GiB of
 * I/O. That is the range the Rust port broke: it narrowed the LONGLONG length to a
 * ULONG before taking the min, so the chunk for a 4 GiB remainder was 0 bytes and
 * `Length -= 0` never ended. A zero-length write is failed here, so that regression
 * reports FALSE instead of hanging the sweep.
 *
 * The CACHED path had the same narrowing, per 256 KiB view: a 4 GiB remainder made
 * a 0-byte view chunk, which MmIsDataSectionResident calls resident, so nothing
 * advanced. Exercising it cheaply relies on Wait=FALSE: on a fresh 8 GiB stream
 * file the first view is not resident, so a correct CcZeroData over [0, 4 GiB)
 * returns FALSE at once without zeroing anything, while the narrowed one spins.
 * A small Wait=TRUE range then checks the cached path really zeroes.
 */

#include <kmt_test.h>

#define NDEBUG
#include <debug.h>

#define ZERO_CHUNK (256 * 1024)

typedef struct _TEST_FCB
{
    SECTION_OBJECT_POINTERS SectionObjectPointers;
} TEST_FCB, *PTEST_FCB;

typedef struct _TEST_CACHED_FCB
{
    FSRTL_ADVANCED_FCB_HEADER Header;
    SECTION_OBJECT_POINTERS SectionObjectPointers;
    FAST_MUTEX HeaderMutex;
} TEST_CACHED_FCB, *PTEST_CACHED_FCB;

static KMT_IRP_HANDLER TestIrpHandler;

/* The stream file the cached tests use, and the paging I/O Cc sent for it. */
static PFILE_OBJECT CachedFileObject;
static struct
{
    ULONG Reads;
    ULONG Writes;
    ULONG NotPaging;
} CachedSeen;

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
    KmtRegisterIrpHandler(IRP_MJ_READ, NULL, TestIrpHandler);
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

static BOOLEAN NTAPI AcquireForLazyWrite(_In_ PVOID Context, _In_ BOOLEAN Wait) { return TRUE; }
static VOID NTAPI ReleaseFromLazyWrite(_In_ PVOID Context) { }
static BOOLEAN NTAPI AcquireForReadAhead(_In_ PVOID Context, _In_ BOOLEAN Wait) { return TRUE; }
static VOID NTAPI ReleaseFromReadAhead(_In_ PVOID Context) { }

static CACHE_MANAGER_CALLBACKS Callbacks = {
    AcquireForLazyWrite,
    ReleaseFromLazyWrite,
    AcquireForReadAhead,
    ReleaseFromReadAhead,
};

#define FILL_BYTE 0xAB
#define SMALL_ZERO (64 * 1024)

static
VOID
Test_CcZeroDataCached(
    _In_ PDEVICE_OBJECT DeviceObject)
{
    const LONGLONG FourGiB = 0x100000000LL;
    PTEST_CACHED_FCB Fcb;
    LARGE_INTEGER Start, End, Zero = {{0}};
    CACHE_UNINITIALIZE_EVENT UninitEvent;
    IO_STATUS_BLOCK Iosb;
    PUCHAR Buffer;
    ULONG i, BadZero, BadFill;
    BOOLEAN Ret;

    CachedFileObject = IoCreateStreamFileObject(NULL, DeviceObject);
    if (skip(CachedFileObject != NULL, "IoCreateStreamFileObject failed\n"))
        return;

    Fcb = ExAllocatePoolWithTag(NonPagedPool, sizeof(*Fcb), 'FzcC');
    if (!skip(Fcb != NULL, "No FCB\n"))
    {
        RtlZeroMemory(Fcb, sizeof(*Fcb));
        ExInitializeFastMutex(&Fcb->HeaderMutex);
        FsRtlSetupAdvancedHeader(&Fcb->Header, &Fcb->HeaderMutex);
        CachedFileObject->FsContext = Fcb;
        CachedFileObject->SectionObjectPointer = &Fcb->SectionObjectPointers;
        Fcb->Header.AllocationSize.QuadPart = 2 * FourGiB;
        Fcb->Header.FileSize.QuadPart = 2 * FourGiB;
        Fcb->Header.ValidDataLength.QuadPart = 2 * FourGiB;

        KmtStartSeh();
        CcInitializeCacheMap(CachedFileObject, (PCC_FILE_SIZES)&Fcb->Header.AllocationSize, FALSE, &Callbacks, NULL);
        KmtEndSeh(STATUS_SUCCESS);

        if (!skip(CcIsFileCached(CachedFileObject), "CcInitializeCacheMap failed\n"))
        {
            /* Exactly 4 GiB, Wait=FALSE: the first 256 KiB view is not resident, so FALSE
             * with no read. Narrowed to a ULONG the first chunk is 0 bytes and never ends. */
            RtlZeroMemory(&CachedSeen, sizeof(CachedSeen));
            Start.QuadPart = 0;
            End.QuadPart = FourGiB;
            Ret = TRUE;
            KmtStartSeh();
            Ret = CcZeroData(CachedFileObject, &Start, &End, FALSE);
            KmtEndSeh(STATUS_SUCCESS);
            ok(Ret == FALSE, "Cached CcZeroData(0..4G, no wait) returned %u\n", Ret);
            ok_eq_ulong(CachedSeen.Reads, 0UL);

            /* Control: a small waited range is paged in (FILL_BYTE) and zeroed */
            End.QuadPart = SMALL_ZERO;
            Ret = FALSE;
            KmtStartSeh();
            Ret = CcZeroData(CachedFileObject, &Start, &End, TRUE);
            KmtEndSeh(STATUS_SUCCESS);
            ok(Ret == TRUE, "Cached CcZeroData(0..64K, wait) returned %u\n", Ret);
            ok(CachedSeen.Reads > 0, "No paging read for the zeroed view\n");
            ok_eq_ulong(CachedSeen.NotPaging, 0UL);

            Buffer = ExAllocatePoolWithTag(NonPagedPool, 2 * SMALL_ZERO, 'BzcC');
            if (!skip(Buffer != NULL, "No buffer\n"))
            {
                Ret = FALSE;
                KmtStartSeh();
                Ret = CcCopyRead(CachedFileObject, &Start, 2 * SMALL_ZERO, TRUE, Buffer, &Iosb);
                KmtEndSeh(STATUS_SUCCESS);
                ok(Ret == TRUE, "CcCopyRead returned %u\n", Ret);
                BadZero = BadFill = 0;
                for (i = 0; i < SMALL_ZERO; i++)
                    BadZero += (Buffer[i] != 0);
                for (i = SMALL_ZERO; i < 2 * SMALL_ZERO; i++)
                    BadFill += (Buffer[i] != FILL_BYTE);
                ok_eq_ulong(BadZero, 0UL);
                ok_eq_ulong(BadFill, 0UL);
                ExFreePoolWithTag(Buffer, 'BzcC');
            }

            KeInitializeEvent(&UninitEvent.Event, NotificationEvent, FALSE);
            CcUninitializeCacheMap(CachedFileObject, &Zero, &UninitEvent);
            KeWaitForSingleObject(&UninitEvent.Event, Executive, KernelMode, FALSE, NULL);
        }

        CachedFileObject->FsContext = NULL;
        CachedFileObject->SectionObjectPointer = NULL;
        ExFreePoolWithTag(Fcb, 'FzcC');
    }

    ObDereferenceObject(CachedFileObject);
    CachedFileObject = NULL;
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

    if (CachedFileObject != NULL && IoStack->FileObject == CachedFileObject)
    {
        /* Paging I/O for the cached tests' stream file: reads return FILL_BYTE,
         * writes (the flush at uninitialize) are accepted and dropped */
        ULONG Length = (IoStack->MajorFunction == IRP_MJ_READ) ?
            IoStack->Parameters.Read.Length : IoStack->Parameters.Write.Length;
        PVOID Buffer = Irp->MdlAddress ?
            MmGetSystemAddressForMdlSafe(Irp->MdlAddress, NormalPagePriority) : NULL;

        /* counted, not ok()'d per IRP: the ratchet baselines a fixed pass count */
        CachedSeen.NotPaging += !FlagOn(Irp->Flags, IRP_PAGING_IO);
        if (IoStack->MajorFunction == IRP_MJ_READ && Buffer != NULL)
        {
            CachedSeen.Reads++;
            RtlFillMemory(Buffer, Length, FILL_BYTE);
            Irp->IoStatus.Information = Length;
            Status = STATUS_SUCCESS;
        }
        else if (IoStack->MajorFunction == IRP_MJ_WRITE)
        {
            CachedSeen.Writes++;
            Irp->IoStatus.Information = Length;
            Status = STATUS_SUCCESS;
        }
    }
    else if (IoStack->MajorFunction == IRP_MJ_CREATE)
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
        Test_CcZeroDataCached(DeviceObject);
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
