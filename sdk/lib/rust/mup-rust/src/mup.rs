//! mup.rs — Rust translation of drivers/filesystems/mup/mup.c (Eric Kohl / Pierre
//! Schweitzer). The Multiple UNC Provider: muxes \\server\share opens across registered
//! network redirectors, caches accepted prefixes in a UNICODE_PREFIX_TABLE, and reroutes
//! IRP_MJ_CREATE to the winning provider via STATUS_REPARSE. Faithful hand translation.
//!
//! SEH (PSEH2) under USE_DUMMY_PSEH: _SEH2_EXCEPT handlers are dead code, so a
//! _SEH2_TRY/_SEH2_EXCEPT becomes the bare try body; _SEH2_LEAVE → `break 'try_blk`.
//! _SEH2_FINALLY bodies always run, so they follow the try block; _SEH2_AbnormalTermination()
//! is always false. ExRaiseStatus is declared `-> !`, so the OOM raise paths (and the dead
//! except arms) are genuinely unreachable — matching the C compiled under DUMMY_PSEH.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_variables, unused_mut, unused_assignments)]

use core::ptr::{addr_of_mut, null_mut};

use ntkrnl_sys::base::{BOOLEAN, LONG, NTSTATUS, NT_SUCCESS, PVOID, SIZE_T, ULONG, USHORT};
use ntkrnl_sys::base::{InitializeListHead, InsertHeadList, InsertTailList, RemoveEntryList};
use ntkrnl_sys::ddk::*;
use ntkrnl_sys::driver_ddk::{IoCallDriver, IoCompleteRequest};
use ntkrnl_sys::ex::{ExAllocatePoolWithTag, ExFreePoolWithTag};
// ExRaiseStatus: use the bindgen ddk decl (real "ExRaiseStatus" export). ex::ExRaiseStatus
// is #[link_name="RtlRaiseStatus"], which ntoskrnl does NOT export on i386 → undefined symbol.
use ntkrnl_sys::ddk::ExRaiseStatus;
use ntkrnl_sys::ob::ObDereferenceObject;
use ntkrnl_sys::rtl::RtlZeroMemory;
use ntkrnl_sys::ke::{KeBugCheckEx, KeGetCurrentIrql};
use ntkrnl_sys::interlocked::{InterlockedDecrement, InterlockedExchangeAdd};
use ntkrnl_sys::inline::tier1::{
    IoGetCurrentIrpStackLocation, IoGetNextIrpStackLocation, IoMarkIrpPending,
    IoSetCompletionRoutine, IoSetNextIrpStackLocation,
};
use ntkrnl_sys::inline::common::{
    FsRtlEnterFileSystem, FsRtlExitFileSystem, InitializeObjectAttributes, PsGetCurrentThread,
};
// KeQuerySystemTime: a real ntoskrnl export on i386, a SharedUserData header inline on
// amd64/arm64 (where ntkrnl-sys provides it).
#[cfg(target_arch = "x86")]
use ntkrnl_sys::ddk::KeQuerySystemTime;
#[cfg(not(target_arch = "x86"))]
use ntkrnl_sys::inline::ke::KeQuerySystemTime;
use ntkrnl_sys::inline::ex::ExInitializeWorkItem;

// ProbeForRead is a private extern in ntkrnl-sys::inline::common; redeclare it here (it's
// a real ntoskrnl export). ULONG_PTR == usize, ULONG == u32 on all three targets.
extern "system" {
}

use crate::dfs::*;
use crate::glue::*;
use crate::helpers::*;
use crate::types::*;
use crate::CONTAINING_RECORD;

const TRUE: BOOLEAN = 1;
const FALSE: BOOLEAN = 0;

// On amd64/arm64 KeQuerySystemTime is a header inline reading KI_USER_SHARED_DATA at its
// fixed kernel VA (0xFFFFF78000000000), via the `SharedUserData` symbol. That symbol is a
// C macro, not a real export, so provide it here (mirrors classpnp-rust's shim). On i386
// KeQuerySystemTime is a real ntoskrnl export and this is unused.
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub static mut SharedUserData: *mut _KUSER_SHARED_DATA =
    0xFFFFF78000000000usize as *mut _KUSER_SHARED_DATA;

// arm64: KeGetCurrentIrql resolves via the ntkrnl_sys inline, which references
// KeGetCurrentPrcb — not exported on arm64. Forward it to the register-reading inline
// (distinct IR name), as classpnp-rust does.
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn KeGetCurrentPrcb() -> ntkrnl_sys::ddk::PKPRCB {
    ntkrnl_sys::inline::ke::KeGetCurrentPrcb()
}

// Function-pointer ABI bridges. The kernel's dispatch/completion/unload/worker pointer
// types use the per-arch NTAPI ABI (extern "stdcall" on i386 via bindgen). Our routines
// are `extern "system"` (same ABI per arch) but a distinct Rust type, so we transmute on
// store — the field-manual "Rust-ABI fn-addr escape" pattern.
type DispatchFn = unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS;
type CompletionFn = unsafe extern "system" fn(PDEVICE_OBJECT, PIRP, PVOID) -> NTSTATUS;
type UnloadFn = unsafe extern "system" fn(PDRIVER_OBJECT);
type WorkerFn = unsafe extern "system" fn(PVOID);

// ---- globals (mup.c file scope) -------------------------------------------
static mut MupGlobalLock: ERESOURCE = unsafe { core::mem::zeroed() };
static mut MupPrefixTableLock: ERESOURCE = unsafe { core::mem::zeroed() };
static mut MupCcbListLock: ERESOURCE = unsafe { core::mem::zeroed() };
static mut MupVcbLock: ERESOURCE = unsafe { core::mem::zeroed() };
static mut MupProviderList: LIST_ENTRY = unsafe { core::mem::zeroed() };
static mut MupPrefixList: LIST_ENTRY = unsafe { core::mem::zeroed() };
static mut MupMasterQueryList: LIST_ENTRY = unsafe { core::mem::zeroed() };
static mut MupPrefixTable: UNICODE_PREFIX_TABLE = unsafe { core::mem::zeroed() };
static mut MupKnownPrefixTimeout: LARGE_INTEGER = unsafe { core::mem::zeroed() };
static mut MupProviderCount: ULONG = 0;
static mut MupOrderInitialized: BOOLEAN = 0;
static mut MupEnableDfs: BOOLEAN = 0;
static mut mupDeviceObject: PDEVICE_OBJECT = null_mut();
static mut MupOrderedErrorList: [NTSTATUS; 6] = [
    STATUS_UNSUCCESSFUL,
    STATUS_INVALID_PARAMETER,
    STATUS_REDIRECTOR_NOT_STARTED,
    STATUS_BAD_NETWORK_NAME,
    STATUS_BAD_NETWORK_PATH,
    0,
];

// ---- wide string literals (mup.c L"...") ----------------------------------
static MUP_REG_KEY: [u16; 56] = wsz(b"\\Registry\\Machine\\System\\CurrentControlSet\\Services\\Mup");
static DISABLE_DFS: [u16; 11] = wsz(b"DisableDfs");
static SERVICES_PREFIX: [u16; 53] = wsz(b"\\Registry\\Machine\\System\\CurrentControlSet\\Services\\");
static NETWORK_PROVIDER_SUFFIX: [u16; 17] = wsz(b"\\NetworkProvider");
static NETPROVIDER_ORDER_KEY: [u16; 73] =
    wsz(b"\\Registry\\Machine\\System\\CurrentControlSet\\Control\\NetworkProvider\\Order");
static PROVIDER_ORDER_VALUE: [u16; 14] = wsz(b"ProviderOrder");
static DEVICE_NAME_VALUE: [u16; 11] = wsz(b"DeviceName");
static MAILSLOT_NAME: [u16; 9] = wsz(b"Mailslot");
static MUP_DEVICE_NAME: [u16; 12] = wsz(b"\\Device\\Mup");

#[inline]
unsafe fn make_ustr(buf: &[u16], chars: u16) -> UNICODE_STRING {
    let mut s: UNICODE_STRING = core::mem::zeroed();
    s.Buffer = buf.as_ptr() as *mut u16;
    s.Length = chars * 2;
    s.MaximumLength = (buf.len() as u16) * 2;
    s
}

// ---------------------------------------------------------------------------

pub unsafe fn MupInitializeData() {
    ExInitializeResourceLite(addr_of_mut!(MupGlobalLock));
    ExInitializeResourceLite(addr_of_mut!(MupPrefixTableLock));
    ExInitializeResourceLite(addr_of_mut!(MupCcbListLock));
    ExInitializeResourceLite(addr_of_mut!(MupVcbLock));
    MupProviderCount = 0;
    InitializeListHead(addr_of_mut!(MupProviderList));
    InitializeListHead(addr_of_mut!(MupPrefixList));
    InitializeListHead(addr_of_mut!(MupMasterQueryList));
    RtlInitializeUnicodePrefix(addr_of_mut!(MupPrefixTable));
    (*addr_of_mut!(MupKnownPrefixTimeout)).QuadPart = 9000000000i64;
    MupOrderInitialized = FALSE;
}

pub unsafe fn MupUninitializeData() {
    ExDeleteResourceLite(addr_of_mut!(MupGlobalLock));
    ExDeleteResourceLite(addr_of_mut!(MupPrefixTableLock));
    ExDeleteResourceLite(addr_of_mut!(MupCcbListLock));
    ExDeleteResourceLite(addr_of_mut!(MupVcbLock));
}

pub unsafe fn MupInitializeVcb(Vcb: PMUP_VCB) {
    RtlZeroMemory(Vcb as PVOID, core::mem::size_of::<MUP_VCB>() as SIZE_T);
    (*Vcb).NodeType = NODE_TYPE_VCB;
    (*Vcb).NodeStatus = NODE_STATUS_HEALTHY;
    (*Vcb).NodeReferences = 1;
    (*Vcb).NodeSize = core::mem::size_of::<MUP_VCB>() as ULONG;
}

pub unsafe fn MuppIsDfsEnabled() -> BOOLEAN {
    let mut Key: HANDLE = null_mut();
    let mut Length: ULONG = 0;
    let mut Status: NTSTATUS;
    let mut KeyName: UNICODE_STRING;
    let mut ObjectAttributes: OBJECT_ATTRIBUTES = core::mem::zeroed();
    #[repr(C)]
    struct KeyQueryOutput {
        KeyInfo: KEY_VALUE_PARTIAL_INFORMATION,
        KeyValue: ULONG,
    }
    let mut KeyQueryOutput: KeyQueryOutput = core::mem::zeroed();

    /* You recognize FsRtlpIsDfsEnabled()! Congratz :-) */
    KeyName = make_ustr(&MUP_REG_KEY, 55);

    InitializeObjectAttributes(
        addr_of_mut!(ObjectAttributes),
        addr_of_mut!(KeyName),
        OBJ_CASE_INSENSITIVE,
        null_mut(),
        null_mut(),
    );

    Status = ZwOpenKey(addr_of_mut!(Key), KEY_READ, addr_of_mut!(ObjectAttributes));
    if !NT_SUCCESS(Status) {
        return TRUE;
    }

    KeyName = make_ustr(&DISABLE_DFS, 10);

    Status = ZwQueryValueKey(
        Key,
        addr_of_mut!(KeyName),
        _KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation,
        addr_of_mut!(KeyQueryOutput) as PVOID,
        core::mem::size_of::<KeyQueryOutput>() as ULONG,
        addr_of_mut!(Length),
    );
    ZwClose(Key);
    if !NT_SUCCESS(Status) || KeyQueryOutput.KeyInfo.Type != REG_DWORD {
        return TRUE;
    }

    // Faithful to the C: it compares the *address* of the Data[] array against 1 (a
    // ReactOS quirk), which is never 1 — so this returns TRUE. DFS ends up disabled
    // anyway via DfsDriverEntry returning STATUS_NOT_IMPLEMENTED.
    if (addr_of_mut!(KeyQueryOutput.KeyInfo.Data) as usize) != 1 {
        TRUE
    } else {
        FALSE
    }
}

pub unsafe fn MupCalculateTimeout(EntryTime: PLARGE_INTEGER) {
    let mut CurrentTime: LARGE_INTEGER = core::mem::zeroed();
    KeQuerySystemTime(addr_of_mut!(CurrentTime));
    (*EntryTime).QuadPart = CurrentTime.QuadPart + (*addr_of_mut!(MupKnownPrefixTimeout)).QuadPart;
}

pub unsafe fn MupCreateCcb() -> PMUP_CCB {
    let Ccb = ExAllocatePoolWithTag(
        _POOL_TYPE_PagedPool,
        core::mem::size_of::<MUP_CCB>() as SIZE_T,
        TAG_MUP,
    ) as PMUP_CCB;
    if Ccb.is_null() {
        return null_mut();
    }
    (*Ccb).NodeStatus = NODE_STATUS_HEALTHY;
    (*Ccb).NodeReferences = 1;
    (*Ccb).NodeType = NODE_TYPE_CCB;
    (*Ccb).NodeSize = core::mem::size_of::<MUP_CCB>() as ULONG;
    Ccb
}

pub unsafe fn MupCreateFcb() -> PMUP_FCB {
    let Fcb = ExAllocatePoolWithTag(
        _POOL_TYPE_PagedPool,
        core::mem::size_of::<MUP_FCB>() as SIZE_T,
        TAG_MUP,
    ) as PMUP_FCB;
    if Fcb.is_null() {
        return null_mut();
    }
    (*Fcb).NodeStatus = NODE_STATUS_HEALTHY;
    (*Fcb).NodeReferences = 1;
    (*Fcb).NodeType = NODE_TYPE_FCB;
    (*Fcb).NodeSize = core::mem::size_of::<MUP_FCB>() as ULONG;
    InitializeListHead(addr_of_mut!((*Fcb).CcbList));
    Fcb
}

pub unsafe fn MupAllocateMasterIoContext() -> PMUP_MIC {
    let MasterIoContext = ExAllocatePoolWithTag(
        _POOL_TYPE_NonPagedPool,
        core::mem::size_of::<MUP_MIC>() as SIZE_T,
        TAG_MUP,
    ) as PMUP_MIC;
    if MasterIoContext.is_null() {
        return null_mut();
    }
    (*MasterIoContext).NodeStatus = NODE_STATUS_HEALTHY;
    (*MasterIoContext).NodeReferences = 1;
    (*MasterIoContext).NodeType = NODE_TYPE_MIC;
    (*MasterIoContext).NodeSize = core::mem::size_of::<MUP_MIC>() as ULONG;
    MasterIoContext
}

pub unsafe fn MupAllocateUncProvider(RedirectorDeviceNameLength: ULONG) -> PMUP_UNC {
    let UncProvider = ExAllocatePoolWithTag(
        _POOL_TYPE_PagedPool,
        (core::mem::size_of::<MUP_UNC>() as ULONG + RedirectorDeviceNameLength) as SIZE_T,
        TAG_MUP,
    ) as PMUP_UNC;
    if UncProvider.is_null() {
        return null_mut();
    }
    (*UncProvider).NodeReferences = 0;
    (*UncProvider).NodeType = NODE_TYPE_UNC;
    (*UncProvider).NodeStatus = NODE_STATUS_HEALTHY;
    (*UncProvider).NodeSize = core::mem::size_of::<MUP_UNC>() as ULONG + RedirectorDeviceNameLength;
    (*UncProvider).Registered = FALSE;
    UncProvider
}

pub unsafe fn MupAllocatePrefixEntry(PrefixLength: ULONG) -> PMUP_PFX {
    let PrefixSize = core::mem::size_of::<MUP_PFX>() as ULONG + PrefixLength;
    let Prefix =
        ExAllocatePoolWithTag(_POOL_TYPE_PagedPool, PrefixSize as SIZE_T, TAG_MUP) as PMUP_PFX;
    if Prefix.is_null() {
        return null_mut();
    }

    RtlZeroMemory(Prefix as PVOID, PrefixSize as SIZE_T);
    (*Prefix).NodeType = NODE_TYPE_PFX;
    (*Prefix).NodeStatus = NODE_STATUS_HEALTHY;
    (*Prefix).NodeReferences = 1;
    (*Prefix).NodeSize = PrefixSize;

    /* If we got a prefix, initialize the string */
    if PrefixLength != 0 {
        (*Prefix).AcceptedPrefix.Buffer =
            (Prefix as usize + core::mem::size_of::<MUP_PFX>()) as *mut u16;
        (*Prefix).AcceptedPrefix.Length = PrefixLength as USHORT;
        (*Prefix).AcceptedPrefix.MaximumLength = PrefixLength as USHORT;
    } else {
        /* Otherwise, mark the fact that the allocation will be external */
        (*Prefix).ExternalAlloc = TRUE;
    }

    (*Prefix).KeepExtraRef = FALSE;
    /* Already init timeout to have one */
    MupCalculateTimeout(addr_of_mut!((*Prefix).ValidityTimeout));

    Prefix
}

pub unsafe fn MupAllocateMasterQueryContext() -> PMUP_MQC {
    let MasterQueryContext = ExAllocatePoolWithTag(
        _POOL_TYPE_NonPagedPool,
        core::mem::size_of::<MUP_MQC>() as SIZE_T,
        TAG_MUP,
    ) as PMUP_MQC;
    if MasterQueryContext.is_null() {
        return null_mut();
    }
    (*MasterQueryContext).NodeStatus = NODE_STATUS_HEALTHY;
    (*MasterQueryContext).NodeReferences = 1;
    (*MasterQueryContext).NodeType = NODE_TYPE_MQC;
    (*MasterQueryContext).NodeSize = core::mem::size_of::<MUP_MQC>() as ULONG;
    InitializeListHead(addr_of_mut!((*MasterQueryContext).QueryPathList));
    InitializeListHead(addr_of_mut!((*MasterQueryContext).MQCListEntry));
    ExInitializeResourceLite(addr_of_mut!((*MasterQueryContext).QueryPathListLock));
    MasterQueryContext
}

pub unsafe fn MupDecodeFileObject(
    FileObject: PFILE_OBJECT,
    pFcb: *mut PMUP_FCB,
    pCcb: *mut PMUP_CCB,
) -> ULONG {
    ExAcquireResourceExclusiveLite(addr_of_mut!(MupGlobalLock), TRUE);
    *pFcb = (*FileObject).FsContext as PMUP_FCB;
    *pCcb = (*FileObject).FsContext2 as PMUP_CCB;
    let Fcb = *pFcb;

    if Fcb.is_null() {
        ExReleaseResourceLite(addr_of_mut!(MupGlobalLock));
        return 0;
    }

    let Type = (*Fcb).NodeType;
    if (Type != NODE_TYPE_VCB && Type != NODE_TYPE_FCB)
        || ((*Fcb).NodeStatus != NODE_STATUS_HEALTHY && (*Fcb).NodeStatus != NODE_STATUS_CLEANUP)
    {
        *pFcb = null_mut();
        ExReleaseResourceLite(addr_of_mut!(MupGlobalLock));
        return 0;
    }

    (*Fcb).NodeReferences += 1;
    ExReleaseResourceLite(addr_of_mut!(MupGlobalLock));

    Type
}

pub unsafe fn MupFreeNode(Node: PVOID) {
    ExFreePoolWithTag(Node, TAG_MUP);
}

pub unsafe fn MupDereferenceFcb(Fcb: PMUP_FCB) {
    if InterlockedDecrement(addr_of_mut!((*Fcb).NodeReferences)) == 0 {
        MupFreeNode(Fcb as PVOID);
    }
}

pub unsafe fn MupDereferenceCcb(Ccb: PMUP_CCB) {
    if InterlockedDecrement(addr_of_mut!((*Ccb).NodeReferences)) == 0 {
        ExAcquireResourceExclusiveLite(addr_of_mut!(MupCcbListLock), TRUE);
        RemoveEntryList(addr_of_mut!((*Ccb).CcbListEntry));
        ExReleaseResourceLite(addr_of_mut!(MupCcbListLock));
        ObDereferenceObject((*Ccb).FileObject as PVOID);
        MupDereferenceFcb((*Ccb).Fcb);
        MupFreeNode(Ccb as PVOID);
    }
}

pub unsafe fn MupDereferenceVcb(Vcb: PMUP_VCB) {
    /* We cannot reach the point where no references are left */
    if InterlockedDecrement(addr_of_mut!((*Vcb).NodeReferences)) == 0 {
        KeBugCheckEx(FILE_SYSTEM, 3, 0, 0, 0);
    }
}

pub unsafe fn MupDereferenceMasterIoContext(
    MasterIoContext: PMUP_MIC,
    NewStatus: PNTSTATUS,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if !NewStatus.is_null() {
        if !NT_SUCCESS(*NewStatus) {
            (*MasterIoContext).LastFailed = *NewStatus;
        } else {
            (*MasterIoContext).LastSuccess = STATUS_SUCCESS;
        }
    }

    if InterlockedDecrement(addr_of_mut!((*MasterIoContext).NodeReferences)) != 0 {
        return STATUS_PENDING;
    }

    let Irp = (*MasterIoContext).Irp;
    let Stack = IoGetCurrentIrpStackLocation(Irp);
    if (*Stack).MajorFunction as usize == IRP_MJ_WRITE {
        (*Irp).IoStatus.Information = (*Stack).Parameters.Write.Length as ULONG_PTR;
    } else {
        (*Irp).IoStatus.Information = 0;
    }

    if !NT_SUCCESS((*MasterIoContext).LastSuccess) {
        Status = (*MasterIoContext).LastFailed;
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_DISK_INCREMENT);
    MupDereferenceFcb((*MasterIoContext).Fcb);
    MupFreeNode(MasterIoContext as PVOID);

    Status
}

pub unsafe fn MupDereferenceUncProvider(UncProvider: PMUP_UNC) {
    InterlockedExchangeAdd(addr_of_mut!((*UncProvider).NodeReferences), -1);
}

pub unsafe fn MupDereferenceKnownPrefix(Prefix: PMUP_PFX) {
    if InterlockedDecrement(addr_of_mut!((*Prefix).NodeReferences)) == 0 {
        if (*Prefix).InTable != 0 {
            RtlRemoveUnicodePrefix(addr_of_mut!(MupPrefixTable), addr_of_mut!((*Prefix).PrefixTableEntry));
            RemoveEntryList(addr_of_mut!((*Prefix).PrefixListEntry));
        }

        if (*Prefix).ExternalAlloc != 0 && !(*Prefix).AcceptedPrefix.Buffer.is_null() {
            ExFreePoolWithTag((*Prefix).AcceptedPrefix.Buffer as PVOID, TAG_MUP);
        }

        if !(*Prefix).UncProvider.is_null() {
            MupDereferenceUncProvider((*Prefix).UncProvider);
        }

        MupFreeNode(Prefix as PVOID);
    }
}

pub unsafe fn MupRemoveKnownPrefixEntry(Prefix: PMUP_PFX) {
    RtlRemoveUnicodePrefix(addr_of_mut!(MupPrefixTable), addr_of_mut!((*Prefix).PrefixTableEntry));
    RemoveEntryList(addr_of_mut!((*Prefix).PrefixListEntry));
    (*Prefix).InTable = FALSE;
    MupDereferenceKnownPrefix(Prefix);
}

pub unsafe fn MupInvalidatePrefixTable() {
    ExAcquireResourceExclusiveLite(addr_of_mut!(MupPrefixTableLock), TRUE);
    let head = addr_of_mut!(MupPrefixList);
    let mut Entry = (*head).Flink;
    while Entry != head {
        let Prefix = CONTAINING_RECORD!(Entry, MUP_PFX, PrefixListEntry);
        if (*Prefix).InTable != 0 {
            MupRemoveKnownPrefixEntry(Prefix);
        }
        Entry = (*Entry).Flink;
    }
    ExReleaseResourceLite(addr_of_mut!(MupPrefixTableLock));
}

pub unsafe fn MupCleanupVcb(DeviceObject: PDEVICE_OBJECT, Irp: PIRP, Vcb: PMUP_VCB) {
    ExAcquireResourceExclusiveLite(addr_of_mut!(MupVcbLock), TRUE);

    if (*Vcb).NodeStatus != NODE_STATUS_HEALTHY || (*Vcb).NodeType != NODE_TYPE_VCB {
        ExRaiseStatus(STATUS_INVALID_HANDLE);
    }

    IoRemoveShareAccess(
        (*IoGetCurrentIrpStackLocation(Irp)).FileObject,
        addr_of_mut!((*Vcb).ShareAccess),
    );

    ExReleaseResourceLite(addr_of_mut!(MupVcbLock));
}

pub unsafe fn MupCleanupFcb(DeviceObject: PDEVICE_OBJECT, Irp: PIRP, Fcb: PMUP_FCB) {
    ExAcquireResourceExclusiveLite(addr_of_mut!(MupGlobalLock), TRUE);
    if (*Fcb).NodeStatus != NODE_STATUS_HEALTHY || (*Fcb).NodeType != NODE_TYPE_FCB {
        ExRaiseStatus(STATUS_INVALID_HANDLE);
    }
    (*Fcb).NodeStatus = NODE_STATUS_CLEANUP;
    ExReleaseResourceLite(addr_of_mut!(MupGlobalLock));

    ExAcquireResourceExclusiveLite(addr_of_mut!(MupCcbListLock), TRUE);
    let head = addr_of_mut!((*Fcb).CcbList);
    let mut Entry = (*head).Flink;
    while Entry != head {
        let Ccb = CONTAINING_RECORD!(Entry, MUP_CCB, CcbListEntry);
        ExReleaseResourceLite(addr_of_mut!(MupCcbListLock));
        MupDereferenceCcb(Ccb);
        ExAcquireResourceExclusiveLite(addr_of_mut!(MupCcbListLock), TRUE);
        Entry = (*Entry).Flink;
    }
    ExReleaseResourceLite(addr_of_mut!(MupCcbListLock));
}

pub unsafe fn CommonForwardedIoCompletionRoutine(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    FwdCtxt: PFORWARDED_IO_CONTEXT,
) -> NTSTATUS {
    let Status = (*Irp).IoStatus.__bindgen_anon_1.Status;

    if !(*Irp).MdlAddress.is_null() {
        MmUnlockPages((*Irp).MdlAddress);
        IoFreeMdl((*Irp).MdlAddress);
    }

    if (*Irp).Flags & IRP_DEALLOCATE_BUFFER != 0 {
        ExFreePoolWithTag((*Irp).AssociatedIrp.SystemBuffer, TAG_MUP);
    }

    IoFreeIrp(Irp);

    MupDereferenceCcb((*FwdCtxt).Ccb);
    let mut s = Status;
    MupDereferenceMasterIoContext((*FwdCtxt).MasterIoContext, addr_of_mut!(s));
    ExFreePoolWithTag(FwdCtxt as PVOID, TAG_MUP);

    STATUS_MORE_PROCESSING_REQUIRED
}

pub unsafe extern "system" fn DeferredForwardedIoCompletionRoutine(Context: PVOID) {
    let FwdCtxt = Context as PFORWARDED_IO_CONTEXT;
    CommonForwardedIoCompletionRoutine((*FwdCtxt).DeviceObject, (*FwdCtxt).Irp, FwdCtxt);
}

pub unsafe extern "system" fn ForwardedIoCompletionRoutine(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    /* If we're at DISPATCH_LEVEL, we cannot complete, defer completion */
    if KeGetCurrentIrql() < DISPATCH_LEVEL {
        CommonForwardedIoCompletionRoutine(DeviceObject, Irp, Context as PFORWARDED_IO_CONTEXT);
    } else {
        let FwdCtxt = Context as PFORWARDED_IO_CONTEXT;
        ExInitializeWorkItem(
            addr_of_mut!((*FwdCtxt).WorkQueueItem),
            core::mem::transmute::<WorkerFn, _>(DeferredForwardedIoCompletionRoutine),
            Context,
        );
        ExQueueWorkItem(addr_of_mut!((*FwdCtxt).WorkQueueItem), _WORK_QUEUE_TYPE_CriticalWorkQueue);
    }

    STATUS_MORE_PROCESSING_REQUIRED
}

pub unsafe fn BuildAndSubmitIrp(Irp: PIRP, Ccb: PMUP_CCB, MasterIoContext: PMUP_MIC) -> NTSTATUS {
    let mut Mdl: PMDL = null_mut();
    let mut LowerIrp: PIRP = null_mut();
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut FwdCtxt: PFORWARDED_IO_CONTEXT = null_mut();

    'cleanup: {
        FwdCtxt = ExAllocatePoolWithTag(
            _POOL_TYPE_NonPagedPool,
            core::mem::size_of::<FORWARDED_IO_CONTEXT>() as SIZE_T,
            TAG_MUP,
        ) as PFORWARDED_IO_CONTEXT;
        if FwdCtxt.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'cleanup;
        }

        (*FwdCtxt).DeviceObject = null_mut();
        (*FwdCtxt).Irp = null_mut();

        let DeviceObject = IoGetRelatedDeviceObject((*Ccb).FileObject);
        LowerIrp = IoAllocateIrp((*DeviceObject).StackSize, TRUE);
        if LowerIrp.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'cleanup;
        }

        (*LowerIrp).Tail.Overlay.OriginalFileObject = (*Ccb).FileObject;
        (*LowerIrp).Tail.Overlay.Thread = (*Irp).Tail.Overlay.Thread;
        (*LowerIrp).RequestorMode = KernelMode;

        let Stack = IoGetNextIrpStackLocation(LowerIrp);
        RtlMoveMemory(
            Stack as PVOID,
            IoGetCurrentIrpStackLocation(Irp) as PVOID,
            core::mem::size_of::<IO_STACK_LOCATION>() as SIZE_T,
        );
        (*Stack).FileObject = (*Ccb).FileObject;
        if (*(*Ccb).FileObject).Flags & FO_WRITE_THROUGH != 0 {
            (*Stack).Flags = SL_WRITE_THROUGH as u8;
        }

        // C: _SEH2_TRY whose handler sets Status to the exception code and falls
        // through to the cleanup below. Each raise in it is a `break 'seh` here.
        'seh: {
            if (*DeviceObject).Flags & DO_BUFFERED_IO != 0 {
                (*LowerIrp).AssociatedIrp.SystemBuffer = null_mut();

                if (*Stack).Parameters.Write.Length == 0 {
                    (*LowerIrp).Flags = IRP_BUFFERED_IO;
                } else {
                    if (*Irp).RequestorMode == UserMode {
                        if let Err(status) = ntkrnl_sys::probe::probe_user_buffer(
                            (*Irp).UserBuffer,
                            (*Stack).Parameters.Write.Length as usize,
                            1,
                            1,
                        ) {
                            Status = status;
                            break 'seh;
                        }
                    }

                    (*LowerIrp).AssociatedIrp.SystemBuffer = ExAllocatePoolWithQuotaTag(
                        _POOL_TYPE_PagedPoolCacheAligned,
                        (*Stack).Parameters.Write.Length as ntkrnl_sys::ddk::SIZE_T,
                        TAG_MUP,
                    );
                    if (*LowerIrp).AssociatedIrp.SystemBuffer.is_null() {
                        Status = STATUS_INSUFFICIENT_RESOURCES;
                        break 'seh;
                    }

                    if let Err(status) = ntkrnl_sys::probe::safe_copy_raw(
                        (*LowerIrp).AssociatedIrp.SystemBuffer,
                        (*Irp).UserBuffer,
                        (*Stack).Parameters.Write.Length as usize,
                    ) {
                        Status = status;
                        break 'seh;
                    }
                    (*LowerIrp).Flags = IRP_BUFFERED_IO | IRP_DEALLOCATE_BUFFER;
                }
            } else if (*DeviceObject).Flags & DO_DIRECT_IO == 0 {
                (*LowerIrp).UserBuffer = (*Irp).UserBuffer;
            } else if (*Stack).Parameters.Write.Length != 0 {
                Mdl = IoAllocateMdl(
                    (*Irp).UserBuffer,
                    (*Stack).Parameters.Write.Length,
                    FALSE,
                    TRUE,
                    LowerIrp,
                );
                if Mdl.is_null() {
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    break 'seh;
                }
                // The C raises here into the _SEH2_EXCEPT below; the status form
                // reaches the same cleanup.
                Status = ntkrnl_sys::probe::MmTryProbeAndLockPages(
                    Mdl,
                    (*Irp).RequestorMode,
                    _LOCK_OPERATION_IoReadAccess,
                );
                if !NT_SUCCESS(Status) {
                    break 'seh;
                }
            }

            if (*(*Ccb).FileObject).Flags & FO_NO_INTERMEDIATE_BUFFERING != 0 {
                (*LowerIrp).Flags |= IRP_WRITE_OPERATION | IRP_NOCACHE;
            } else {
                (*LowerIrp).Flags |= IRP_WRITE_OPERATION;
            }

            (*FwdCtxt).Ccb = Ccb;
            (*FwdCtxt).MasterIoContext = MasterIoContext;

            ExAcquireResourceExclusiveLite(addr_of_mut!(MupGlobalLock), TRUE);
            (*MasterIoContext).NodeReferences += 1;
            ExReleaseResourceLite(addr_of_mut!(MupGlobalLock));

            IoSetCompletionRoutine(
                LowerIrp,
                core::mem::transmute::<CompletionFn, PIO_COMPLETION_ROUTINE>(
                    ForwardedIoCompletionRoutine,
                ),
                FwdCtxt as PVOID,
                TRUE,
                TRUE,
                TRUE,
            );
            Status = IoCallDriver((*Ccb).DeviceObject, LowerIrp);
        }
    }

    // Cleanup:
    if !NT_SUCCESS(Status) {
        if !FwdCtxt.is_null() {
            ExFreePoolWithTag(FwdCtxt as PVOID, TAG_MUP);
        }

        if !LowerIrp.is_null() {
            // The C tests `SystemBuffer == NULL` here, so it frees NULL and leaks the
            // buffered-write copy (IRP_DEALLOCATE_BUFFER is only honoured by
            // IoCompleteRequest, never by IoFreeIrp). Free it when it exists.
            if !(*LowerIrp).AssociatedIrp.SystemBuffer.is_null() {
                ExFreePoolWithTag((*LowerIrp).AssociatedIrp.SystemBuffer, TAG_MUP);
            }
            IoFreeIrp(LowerIrp);
        }

        if !Mdl.is_null() {
            IoFreeMdl(Mdl);
        }
    }

    Status
}

pub unsafe extern "system" fn MupForwardIoRequest(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let mut Fcb: PMUP_FCB = null_mut();
    let mut Ccb: PMUP_CCB = null_mut();
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut CcbLockAcquired: BOOLEAN = FALSE;

    /* If DFS is enabled, check if that's for DFS and is so relay */
    if MupEnableDfs != 0 && (*DeviceObject).DeviceType == FILE_DEVICE_DFS {
        return DfsVolumePassThrough(DeviceObject, Irp);
    }

    let Stack = IoGetCurrentIrpStackLocation(Irp);

    FsRtlEnterFileSystem();

    'complete: {
        MupDecodeFileObject((*Stack).FileObject, addr_of_mut!(Fcb), addr_of_mut!(Ccb));
        if Fcb.is_null() || (*Fcb).NodeType != NODE_TYPE_FCB {
            FsRtlExitFileSystem();
            Status = STATUS_INVALID_DEVICE_REQUEST;
            break 'complete;
        }

        let MasterIoContext = MupAllocateMasterIoContext();
        if MasterIoContext.is_null() {
            FsRtlExitFileSystem();
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'complete;
        }

        IoMarkIrpPending(Irp);
        (*MasterIoContext).Irp = Irp;
        (*MasterIoContext).LastSuccess = STATUS_UNSUCCESSFUL;
        (*MasterIoContext).LastFailed = STATUS_BAD_NETWORK_PATH;
        (*MasterIoContext).Fcb = Fcb;

        // _SEH2_TRY/_SEH2_EXCEPT: handler releases the CCB lock if held. Dead under
        // DUMMY_PSEH (no fault expected on list walk); inline the try body.
        {
            ExAcquireResourceExclusiveLite(addr_of_mut!(MupCcbListLock), TRUE);
            CcbLockAcquired = TRUE;

            let head = addr_of_mut!((*Fcb).CcbList);
            let mut Entry = (*head).Flink;
            while Entry != head {
                let FcbListCcb = CONTAINING_RECORD!(Entry, MUP_CCB, CcbListEntry);
                ExReleaseResourceLite(addr_of_mut!(MupCcbListLock));
                CcbLockAcquired = FALSE;

                ExAcquireResourceExclusiveLite(addr_of_mut!(MupGlobalLock), TRUE);
                (*FcbListCcb).NodeReferences += 1;
                ExReleaseResourceLite(addr_of_mut!(MupGlobalLock));

                BuildAndSubmitIrp(Irp, FcbListCcb, MasterIoContext);
                ExAcquireResourceExclusiveLite(addr_of_mut!(MupCcbListLock), TRUE);
                CcbLockAcquired = TRUE;

                Entry = (*Entry).Flink;
            }

            ExReleaseResourceLite(addr_of_mut!(MupCcbListLock));
            CcbLockAcquired = FALSE;
        }

        MupDereferenceMasterIoContext(MasterIoContext, null_mut());
        FsRtlExitFileSystem();

        return STATUS_PENDING;
    }

    // Complete:
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_DISK_INCREMENT);
    Status
}

pub unsafe fn AddUnregisteredProvider(DeviceName: *const u16, ProviderOrder: ULONG) -> PMUP_UNC {
    let NameLen = wcslen(DeviceName) as ULONG;
    let StrLen = NameLen * core::mem::size_of::<u16>() as ULONG;
    let UncProvider = MupAllocateUncProvider(StrLen);
    if UncProvider.is_null() {
        return null_mut();
    }

    (*UncProvider).DeviceName.MaximumLength = StrLen as USHORT;
    (*UncProvider).DeviceName.Length = StrLen as USHORT;
    (*UncProvider).DeviceName.Buffer =
        (UncProvider as usize + core::mem::size_of::<MUP_UNC>()) as *mut u16;
    (*UncProvider).ProviderOrder = ProviderOrder;
    RtlMoveMemory(
        (*UncProvider).DeviceName.Buffer as PVOID,
        DeviceName as PVOID,
        StrLen as SIZE_T,
    );

    ExAcquireResourceExclusiveLite(addr_of_mut!(MupGlobalLock), TRUE);
    InsertTailList(addr_of_mut!(MupProviderList), addr_of_mut!((*UncProvider).ProviderListEntry));
    ExReleaseResourceLite(addr_of_mut!(MupGlobalLock));

    UncProvider
}

pub unsafe fn InitializeProvider(ProviderName: *const u16, ProviderOrder: ULONG) {
    let mut Status: NTSTATUS;
    let mut KeyHandle: HANDLE = null_mut();
    let mut Key: UNICODE_STRING = core::mem::zeroed();
    let mut Value: UNICODE_STRING = core::mem::zeroed();
    let mut Info: PKEY_VALUE_FULL_INFORMATION = null_mut();
    let mut ObjectAttributes: OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut ResultLength: ULONG = 0;

    let NameLen = wcslen(ProviderName) as ULONG;
    let StrLen = NameLen * core::mem::size_of::<u16>() as ULONG
        + (SERVICES_PREFIX.len() * 2) as ULONG
        + (NETWORK_PROVIDER_SUFFIX.len() * 2) as ULONG;
    Key.Buffer = ExAllocatePoolWithTag(_POOL_TYPE_PagedPool, StrLen as SIZE_T, TAG_MUP) as *mut u16;
    if Key.Buffer.is_null() {
        return;
    }

    RtlMoveMemory(
        Key.Buffer as PVOID,
        SERVICES_PREFIX.as_ptr() as PVOID,
        (SERVICES_PREFIX.len() * 2) as SIZE_T,
    );
    Key.MaximumLength = StrLen as USHORT;
    Key.Length = ((SERVICES_PREFIX.len() - 1) * 2) as USHORT;
    RtlAppendUnicodeToString(addr_of_mut!(Key), ProviderName);
    RtlAppendUnicodeToString(addr_of_mut!(Key), NETWORK_PROVIDER_SUFFIX.as_ptr());

    InitializeObjectAttributes(
        addr_of_mut!(ObjectAttributes),
        addr_of_mut!(Key),
        OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE,
        null_mut(),
        null_mut(),
    );
    Status = ZwOpenKey(addr_of_mut!(KeyHandle), KEY_QUERY_VALUE, addr_of_mut!(ObjectAttributes));
    ExFreePoolWithTag(Key.Buffer as PVOID, TAG_MUP);
    if !NT_SUCCESS(Status) {
        return;
    }

    RtlInitUnicodeString(addr_of_mut!(Value), DEVICE_NAME_VALUE.as_ptr());
    Status = ZwQueryValueKey(
        KeyHandle,
        addr_of_mut!(Value),
        _KEY_VALUE_INFORMATION_CLASS_KeyValueFullInformation,
        null_mut(),
        0,
        addr_of_mut!(ResultLength),
    );
    if Status == STATUS_BUFFER_TOO_SMALL {
        Info = ExAllocatePoolWithTag(
            _POOL_TYPE_PagedPool,
            (ResultLength + core::mem::size_of::<u16>() as ULONG) as SIZE_T,
            TAG_MUP,
        ) as PKEY_VALUE_FULL_INFORMATION;
        if Info.is_null() {
            ZwClose(KeyHandle);
            return;
        }

        Status = ZwQueryValueKey(
            KeyHandle,
            addr_of_mut!(Value),
            _KEY_VALUE_INFORMATION_CLASS_KeyValueFullInformation,
            Info as PVOID,
            ResultLength,
            addr_of_mut!(ResultLength),
        );
    } else {
        Info = null_mut();
    }

    ZwClose(KeyHandle);

    if NT_SUCCESS(Status) {
        AddUnregisteredProvider(
            (Info as usize + (*Info).DataOffset as usize) as *const u16,
            ProviderOrder,
        );
    }

    if !Info.is_null() {
        ExFreePoolWithTag(Info as PVOID, TAG_MUP);
    }
}

pub unsafe fn MupGetProviderInformation() {
    let mut Status: NTSTATUS;
    let mut KeyHandle: HANDLE = null_mut();
    let mut Info: PKEY_VALUE_FULL_INFORMATION = null_mut();
    let mut ResultLength: ULONG = 0;
    let mut ObjectAttributes: OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut NetworkProvider: UNICODE_STRING = core::mem::zeroed();
    let mut ProviderOrder: UNICODE_STRING = core::mem::zeroed();

    RtlInitUnicodeString(addr_of_mut!(NetworkProvider), NETPROVIDER_ORDER_KEY.as_ptr());
    InitializeObjectAttributes(
        addr_of_mut!(ObjectAttributes),
        addr_of_mut!(NetworkProvider),
        OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE,
        null_mut(),
        null_mut(),
    );
    Status = ZwOpenKey(addr_of_mut!(KeyHandle), KEY_QUERY_VALUE, addr_of_mut!(ObjectAttributes));
    if !NT_SUCCESS(Status) {
        return;
    }

    RtlInitUnicodeString(addr_of_mut!(ProviderOrder), PROVIDER_ORDER_VALUE.as_ptr());
    Status = ZwQueryValueKey(
        KeyHandle,
        addr_of_mut!(ProviderOrder),
        _KEY_VALUE_INFORMATION_CLASS_KeyValueFullInformation,
        null_mut(),
        0,
        addr_of_mut!(ResultLength),
    );
    if Status == STATUS_BUFFER_TOO_SMALL {
        Info = ExAllocatePoolWithTag(
            _POOL_TYPE_PagedPool,
            (ResultLength + core::mem::size_of::<u16>() as ULONG) as SIZE_T,
            TAG_MUP,
        ) as PKEY_VALUE_FULL_INFORMATION;
        if Info.is_null() {
            ZwClose(KeyHandle);
            return;
        }

        Status = ZwQueryValueKey(
            KeyHandle,
            addr_of_mut!(ProviderOrder),
            _KEY_VALUE_INFORMATION_CLASS_KeyValueFullInformation,
            Info as PVOID,
            ResultLength,
            addr_of_mut!(ResultLength),
        );
    } else {
        Info = null_mut();
    }

    ZwClose(KeyHandle);

    if NT_SUCCESS(Status) {
        let mut Providers = (Info as usize + (*Info).DataOffset as usize) as *mut u16;
        let mut End: BOOLEAN = FALSE;
        let mut ProviderCount: ULONG = 0;

        loop {
            let Coma = wcschr(Providers, ',' as u16);
            if !Coma.is_null() {
                *Coma = UNICODE_NULL;
            } else {
                End = TRUE;
            }

            InitializeProvider(Providers, ProviderCount);
            ProviderCount += 1;

            Providers = Coma.add(1);

            if End != 0 {
                break;
            }
        }
    }

    if !Info.is_null() {
        ExFreePoolWithTag(Info as PVOID, TAG_MUP);
    }
}

pub unsafe fn MupCheckForUnregisteredProvider(RedirectorDeviceName: PUNICODE_STRING) -> PMUP_UNC {
    let mut UncProvider: PMUP_UNC = null_mut();

    ExAcquireResourceExclusiveLite(addr_of_mut!(MupGlobalLock), TRUE);
    let head = addr_of_mut!(MupProviderList);
    let mut Entry = (*head).Flink;
    while Entry != head {
        UncProvider = CONTAINING_RECORD!(Entry, MUP_UNC, ProviderListEntry);

        if (*UncProvider).Registered == 0
            && RtlEqualUnicodeString(RedirectorDeviceName, addr_of_mut!((*UncProvider).DeviceName), TRUE) != 0
        {
            (*UncProvider).NodeStatus = NODE_STATUS_HEALTHY;
            break;
        }
        Entry = (*Entry).Flink;
    }

    if Entry == head {
        UncProvider = null_mut();
    }
    ExReleaseResourceLite(addr_of_mut!(MupGlobalLock));

    UncProvider
}

pub unsafe fn RegisterUncProvider(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let mut New: BOOLEAN = FALSE;
    let mut Fcb: PMUP_FCB = null_mut();
    let mut Ccb: PMUP_CCB = null_mut();
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut UncProvider: PMUP_UNC;
    let mut ListEntry: PMUP_UNC;
    let mut IoStatusBlock: IO_STATUS_BLOCK = core::mem::zeroed();
    let mut ObjectAttributes: OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut RedirectorDeviceName: UNICODE_STRING = core::mem::zeroed();
    let mut HandleInfo: OBJECT_HANDLE_INFORMATION = core::mem::zeroed();

    /* Check whether providers order was already initialized */
    ExAcquireResourceExclusiveLite(addr_of_mut!(MupGlobalLock), TRUE);
    if MupOrderInitialized != 0 {
        ExReleaseResourceLite(addr_of_mut!(MupGlobalLock));
    } else {
        MupOrderInitialized = TRUE;
        ExReleaseResourceLite(addr_of_mut!(MupGlobalLock));
        MupGetProviderInformation();
    }

    let Stack = IoGetCurrentIrpStackLocation(Irp);

    if MupDecodeFileObject((*Stack).FileObject, addr_of_mut!(Fcb), addr_of_mut!(Ccb)) != NODE_TYPE_VCB {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_HANDLE;
        IoCompleteRequest(Irp, IO_DISK_INCREMENT);
        return STATUS_INVALID_HANDLE;
    }

    let RegInfo =
        (*Irp).AssociatedIrp.SystemBuffer as crate::types::PMUP_PROVIDER_REGISTRATION_INFO;

    // _SEH2_TRY ... _SEH2_FINALLY (the finally always completes the IRP)
    'try_blk: {
        RedirectorDeviceName.Length = (*RegInfo).RedirectorDeviceNameLength as USHORT;
        RedirectorDeviceName.MaximumLength = RedirectorDeviceName.Length;
        RedirectorDeviceName.Buffer =
            (RegInfo as usize + (*RegInfo).RedirectorDeviceNameOffset as usize) as *mut u16;

        UncProvider = MupCheckForUnregisteredProvider(addr_of_mut!(RedirectorDeviceName));
        if UncProvider.is_null() {
            New = TRUE;
            UncProvider = MupAllocateUncProvider((*RegInfo).RedirectorDeviceNameLength);
            if UncProvider.is_null() {
                Status = STATUS_INVALID_USER_BUFFER;
                break 'try_blk;
            }

            (*UncProvider).DeviceName.Length = RedirectorDeviceName.Length;
            (*UncProvider).DeviceName.MaximumLength = RedirectorDeviceName.MaximumLength;
            (*UncProvider).DeviceName.Buffer =
                (UncProvider as usize + core::mem::size_of::<MUP_UNC>()) as *mut u16;

            (*UncProvider).ProviderOrder = MAXLONG;
            RtlMoveMemory(
                (*UncProvider).DeviceName.Buffer as PVOID,
                (RegInfo as usize + (*RegInfo).RedirectorDeviceNameOffset as usize) as PVOID,
                (*RegInfo).RedirectorDeviceNameLength as SIZE_T,
            );
        }

        (*UncProvider).MailslotsSupported = (*RegInfo).MailslotsSupported;
        (*UncProvider).NodeReferences += 1;

        InitializeObjectAttributes(
            addr_of_mut!(ObjectAttributes),
            addr_of_mut!((*UncProvider).DeviceName),
            OBJ_CASE_INSENSITIVE,
            null_mut(),
            null_mut(),
        );
        Status = NtOpenFile(
            addr_of_mut!((*UncProvider).DeviceHandle),
            FILE_TRAVERSE,
            addr_of_mut!(ObjectAttributes),
            addr_of_mut!(IoStatusBlock),
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            FILE_DIRECTORY_FILE,
        );
        if NT_SUCCESS(Status) {
            Status = IoStatusBlock.__bindgen_anon_1.Status;
        }

        if NT_SUCCESS(Status) {
            (*(*Stack).FileObject).FsContext2 = UncProvider as PVOID;
            Status = ObReferenceObjectByHandle(
                (*UncProvider).DeviceHandle,
                0,
                null_mut(),
                KernelMode,
                addr_of_mut!((*UncProvider).FileObject) as *mut PVOID,
                addr_of_mut!(HandleInfo),
            );
            if !NT_SUCCESS(Status) {
                NtClose((*UncProvider).DeviceHandle);
            }
        }

        if !NT_SUCCESS(Status) {
            MupDereferenceUncProvider(UncProvider);
        } else {
            (*UncProvider).DeviceObject = IoGetRelatedDeviceObject((*UncProvider).FileObject);

            ExAcquireResourceExclusiveLite(addr_of_mut!(MupGlobalLock), TRUE);
            MupProviderCount += 1;
            if New != 0 {
                let head = addr_of_mut!(MupProviderList);
                let mut Entry = (*head).Flink;
                while Entry != head {
                    ListEntry = CONTAINING_RECORD!(Entry, MUP_UNC, ProviderListEntry);
                    if (*UncProvider).ProviderOrder < (*ListEntry).ProviderOrder {
                        break;
                    }
                    Entry = (*Entry).Flink;
                }
                InsertTailList(Entry, addr_of_mut!((*UncProvider).ProviderListEntry));
            }
            (*UncProvider).Registered = TRUE;
            ExReleaseResourceLite(addr_of_mut!(MupGlobalLock));
            Status = STATUS_SUCCESS;
        }
    }

    // _SEH2_FINALLY (AbnormalTermination() is always false under DUMMY_PSEH)
    MupDereferenceVcb(Fcb as PMUP_VCB);
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_DISK_INCREMENT);

    Status
}

pub unsafe extern "system" fn MupFsControl(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let Stack = IoGetCurrentIrpStackLocation(Irp);

    'try_blk: {
        if (*Stack).Parameters.FileSystemControl.FsControlCode == FSCTL_MUP_REGISTER_PROVIDER {
            if (*Irp).RequestorMode == UserMode {
                Status = STATUS_ACCESS_DENIED;
                (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
                IoCompleteRequest(Irp, IO_DISK_INCREMENT);
                break 'try_blk;
            }
            Status = RegisterUncProvider(DeviceObject, Irp);
        } else {
            if MupEnableDfs == 0 {
                Status = STATUS_INVALID_PARAMETER;
                (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
                IoCompleteRequest(Irp, IO_DISK_INCREMENT);
                break 'try_blk;
            }
            Status = DfsFsdFileSystemControl(DeviceObject, Irp);
        }
    }

    Status
}

pub unsafe fn MupSetFileObject(FileObject: PFILE_OBJECT, Fcb: PMUP_FCB, Ccb: PMUP_CCB) {
    (*FileObject).FsContext = Fcb as PVOID;
    (*FileObject).FsContext2 = Ccb as PVOID;
}

pub unsafe fn MupRerouteOpen(FileObject: PFILE_OBJECT, UncProvider: PMUP_UNC) -> NTSTATUS {
    let TotalLength =
        (*UncProvider).DeviceName.Length as ULONG + (*FileObject).FileName.Length as ULONG;
    if TotalLength > MAXUSHORT {
        return STATUS_NAME_TOO_LONG;
    }

    let FullPath = ExAllocatePoolWithTag(_POOL_TYPE_PagedPool, TotalLength as SIZE_T, TAG_MUP) as *mut u16;
    if FullPath.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlMoveMemory(
        FullPath as PVOID,
        (*UncProvider).DeviceName.Buffer as PVOID,
        (*UncProvider).DeviceName.Length as SIZE_T,
    );
    RtlMoveMemory(
        (FullPath as usize + (*UncProvider).DeviceName.Length as usize) as PVOID,
        (*FileObject).FileName.Buffer as PVOID,
        (*FileObject).FileName.Length as SIZE_T,
    );

    ExFreePoolWithTag((*FileObject).FileName.Buffer as PVOID, 0);
    (*FileObject).FileName.Buffer = FullPath;
    (*FileObject).FileName.MaximumLength = TotalLength as USHORT;
    (*FileObject).FileName.Length = (*FileObject).FileName.MaximumLength;

    STATUS_REPARSE
}

pub unsafe fn BroadcastOpen(Irp: PIRP) -> NTSTATUS {
    let mut Handle: HANDLE = null_mut();
    let mut Ccb: PMUP_CCB = null_mut();
    let mut UncProvider: PMUP_UNC = null_mut();
    let mut FullPath: UNICODE_STRING = core::mem::zeroed();
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut TotalLength: ULONG;
    let mut LastOrder: ULONG;
    let mut LastFailed: NTSTATUS;
    let mut IoStatusBlock: IO_STATUS_BLOCK = core::mem::zeroed();
    let mut ObjectAttributes: OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut HandleInfo: OBJECT_HANDLE_INFORMATION = core::mem::zeroed();
    let mut Locked: BOOLEAN = FALSE;
    let mut Referenced: BOOLEAN = FALSE;
    let mut CcbInitialized: BOOLEAN = FALSE;

    let Fcb = MupCreateFcb();
    if Fcb.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let Stack = IoGetCurrentIrpStackLocation(Irp);
    let FileObject = (*Stack).FileObject;
    LastFailed = STATUS_NO_SUCH_FILE;
    LastOrder = ULONG::MAX;

    // _SEH2_TRY/_SEH2_EXCEPT (handler dead): inline the body.
    {
        ExAcquireResourceExclusiveLite(addr_of_mut!(MupGlobalLock), TRUE);
        Locked = TRUE;

        MupSetFileObject(FileObject, Fcb, null_mut());
        (*Fcb).FileObject = FileObject;

        let head = addr_of_mut!(MupProviderList);
        let mut Entry = (*head).Flink;
        while Entry != head {
            UncProvider = CONTAINING_RECORD!(Entry, MUP_UNC, ProviderListEntry);
            (*UncProvider).NodeReferences += 1;
            Referenced = TRUE;

            ExReleaseResourceLite(addr_of_mut!(MupGlobalLock));
            Locked = FALSE;

            TotalLength =
                (*UncProvider).DeviceName.Length as ULONG + (*FileObject).FileName.Length as ULONG;
            if (*UncProvider).MailslotsSupported != 0 && TotalLength <= MAXUSHORT {
                FullPath.Buffer =
                    ExAllocatePoolWithTag(_POOL_TYPE_PagedPool, TotalLength as SIZE_T, TAG_MUP) as *mut u16;
                if FullPath.Buffer.is_null() {
                    ExRaiseStatus(STATUS_INSUFFICIENT_RESOURCES);
                }

                FullPath.Length = TotalLength as USHORT;
                FullPath.MaximumLength = TotalLength as USHORT;
                RtlMoveMemory(
                    FullPath.Buffer as PVOID,
                    (*UncProvider).DeviceName.Buffer as PVOID,
                    (*UncProvider).DeviceName.Length as SIZE_T,
                );
                RtlMoveMemory(
                    (FullPath.Buffer as usize + (*UncProvider).DeviceName.Length as usize) as PVOID,
                    (*FileObject).FileName.Buffer as PVOID,
                    (*FileObject).FileName.Length as SIZE_T,
                );

                InitializeObjectAttributes(
                    addr_of_mut!(ObjectAttributes),
                    addr_of_mut!(FullPath),
                    OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE,
                    null_mut(),
                    null_mut(),
                );
                Status = IoCreateFile(
                    addr_of_mut!(Handle),
                    (*(*Stack).Parameters.Create.SecurityContext).DesiredAccess & FILE_SIMPLE_RIGHTS_MASK,
                    addr_of_mut!(ObjectAttributes),
                    addr_of_mut!(IoStatusBlock),
                    null_mut(),
                    (*Stack).Parameters.Create.FileAttributes as ULONG & FILE_ATTRIBUTE_VALID_FLAGS,
                    (*Stack).Parameters.Create.ShareAccess as ULONG & FILE_SHARE_VALID_FLAGS,
                    FILE_OPEN,
                    (*Stack).Parameters.Create.Options & FILE_VALID_SET_FLAGS,
                    null_mut(),
                    0,
                    _CREATE_FILE_TYPE_CreateFileTypeNone,
                    null_mut(),
                    IO_NO_PARAMETER_CHECKING,
                );

                ExFreePoolWithTag(FullPath.Buffer as PVOID, TAG_MUP);

                if NT_SUCCESS(Status) {
                    Status = IoStatusBlock.__bindgen_anon_1.Status;

                    Ccb = MupCreateCcb();
                    if Ccb.is_null() {
                        Status = STATUS_INSUFFICIENT_RESOURCES;
                    }

                    if NT_SUCCESS(Status) {
                        Status = ObReferenceObjectByHandle(
                            Handle,
                            0,
                            null_mut(),
                            0,
                            addr_of_mut!((*Ccb).FileObject) as *mut PVOID,
                            addr_of_mut!(HandleInfo),
                        );
                        ZwClose(Handle);
                    }
                }

                if !NT_SUCCESS(Status) {
                    if (*UncProvider).ProviderOrder <= LastOrder {
                        LastOrder = (*UncProvider).ProviderOrder;
                        LastFailed = Status;
                    }
                } else {
                    (*Ccb).DeviceObject = IoGetRelatedDeviceObject((*Ccb).FileObject);
                    (*Ccb).Fcb = Fcb;

                    ExAcquireResourceExclusiveLite(addr_of_mut!(MupGlobalLock), TRUE);
                    Locked = TRUE;
                    (*Fcb).NodeReferences += 1;
                    ExReleaseResourceLite(addr_of_mut!(MupGlobalLock));
                    Locked = FALSE;
                    CcbInitialized = TRUE;

                    InsertTailList(addr_of_mut!((*Fcb).CcbList), addr_of_mut!((*Ccb).CcbListEntry));
                }
            }

            ExAcquireResourceExclusiveLite(addr_of_mut!(MupGlobalLock), TRUE);
            Locked = TRUE;
            MupDereferenceUncProvider(UncProvider);
            Referenced = FALSE;

            Entry = (*Entry).Flink;
        }

        ExReleaseResourceLite(addr_of_mut!(MupGlobalLock));
        Locked = FALSE;
    }

    Status = if CcbInitialized != 0 { STATUS_SUCCESS } else { LastFailed };

    if Referenced != 0 {
        MupDereferenceUncProvider(UncProvider);
    }

    if Locked != 0 {
        ExReleaseResourceLite(addr_of_mut!(MupGlobalLock));
    }

    if !NT_SUCCESS(Status) && !Ccb.is_null() {
        MupFreeNode(Ccb as PVOID);
    }

    Status
}

pub unsafe fn MupBuildIoControlRequest(
    FileObject: PFILE_OBJECT,
    Context: PVOID,
    MajorFunction: ULONG,
    IoctlCode: ULONG,
    InputBuffer: PVOID,
    InputBufferSize: ULONG,
    OutputBuffer: PVOID,
    OutputBufferSize: ULONG,
    CompletionRoutine: PIO_COMPLETION_ROUTINE,
) -> PIRP {
    if InputBuffer.is_null() {
        return null_mut();
    }

    let DeviceObject = IoGetRelatedDeviceObject(FileObject);
    let Irp = IoAllocateIrp((*DeviceObject).StackSize + 1, FALSE);
    if Irp.is_null() {
        return null_mut();
    }

    IoSetNextIrpStackLocation(Irp);
    (*Irp).Tail.Overlay.OriginalFileObject = FileObject;
    (*Irp).Tail.Overlay.Thread = PsGetCurrentThread();
    IoSetCompletionRoutine(Irp, CompletionRoutine, Context, TRUE, TRUE, TRUE);

    let Stack = IoGetNextIrpStackLocation(Irp);
    (*Stack).MajorFunction = MajorFunction as u8;
    (*Stack).Parameters.DeviceIoControl.OutputBufferLength = OutputBufferSize;
    (*Stack).Parameters.DeviceIoControl.InputBufferLength = InputBufferSize;
    (*Stack).Parameters.DeviceIoControl.IoControlCode = IoctlCode;
    (*Stack).MinorFunction = 0;
    (*Stack).FileObject = FileObject;
    (*Stack).DeviceObject = DeviceObject;

    match IoctlCode & 0x00000003 {
        x if x == METHOD_BUFFERED => {
            (*Irp).MdlAddress = null_mut();
            (*Irp).AssociatedIrp.SystemBuffer = InputBuffer;
            (*Irp).UserBuffer = OutputBuffer;
            (*Irp).Flags = IRP_BUFFERED_IO;
            if !OutputBuffer.is_null() {
                (*Irp).Flags |= IRP_INPUT_OPERATION;
            }
        }
        x if x == METHOD_IN_DIRECT || x == METHOD_OUT_DIRECT => {
            if IoAllocateMdl(InputBuffer, InputBufferSize, FALSE, FALSE, Irp).is_null() {
                IoFreeIrp(Irp);
                return null_mut();
            }
            (*Irp).AssociatedIrp.SystemBuffer = InputBuffer;
            (*Irp).Flags = IRP_BUFFERED_IO;
            // The C raises on a bad buffer with nothing to catch it. Fail the build
            // the way the IoAllocateMdl failure above does instead.
            if !NT_SUCCESS(ntkrnl_sys::probe::MmTryProbeAndLockPages(
                (*Irp).MdlAddress,
                KernelMode,
                _LOCK_OPERATION_IoReadAccess,
            )) {
                IoFreeMdl((*Irp).MdlAddress);
                IoFreeIrp(Irp);
                return null_mut();
            }
        }
        x if x == METHOD_NEITHER => {
            (*Irp).UserBuffer = OutputBuffer;
            (*Irp).MdlAddress = null_mut();
            (*Irp).AssociatedIrp.SystemBuffer = null_mut();
            (*Stack).Parameters.DeviceIoControl.Type3InputBuffer = InputBuffer;
        }
        _ => {}
    }

    Irp
}

pub unsafe fn MupFreeMasterQueryContext(MasterQueryContext: PMUP_MQC) {
    ExDeleteResourceLite(addr_of_mut!((*MasterQueryContext).QueryPathListLock));
    ExFreePoolWithTag(MasterQueryContext as PVOID, TAG_MUP);
}

pub unsafe fn MupDereferenceMasterQueryContext(MasterQueryContext: PMUP_MQC) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let KeepExtraRef: BOOLEAN;

    ExAcquireResourceExclusiveLite(addr_of_mut!(MupGlobalLock), TRUE);
    (*MasterQueryContext).NodeReferences -= 1;
    let References = (*MasterQueryContext).NodeReferences;
    ExReleaseResourceLite(addr_of_mut!(MupGlobalLock));

    if References != 0 {
        return STATUS_PENDING;
    }

    if (*MasterQueryContext).Irp.is_null() {
        KeBugCheckEx(FILE_SYSTEM, 0, 0, 0, 0);
    }

    ExAcquireResourceExclusiveLite(addr_of_mut!(MupGlobalLock), TRUE);
    RemoveEntryList(addr_of_mut!((*MasterQueryContext).MQCListEntry));
    ExReleaseResourceLite(addr_of_mut!(MupGlobalLock));

    ExAcquireResourceExclusiveLite(addr_of_mut!(MupPrefixTableLock), TRUE);
    KeepExtraRef = (*(*MasterQueryContext).Prefix).KeepExtraRef;
    MupDereferenceKnownPrefix((*MasterQueryContext).Prefix);

    'complete: {
        if !(*MasterQueryContext).LatestProvider.is_null() {
            if (*MasterQueryContext).LatestStatus == STATUS_SUCCESS {
                if KeepExtraRef == 0 {
                    MupDereferenceKnownPrefix((*MasterQueryContext).Prefix);
                }

                ExReleaseResourceLite(addr_of_mut!(MupPrefixTableLock));
                Status = MupRerouteOpen(
                    (*MasterQueryContext).FileObject,
                    (*MasterQueryContext).LatestProvider,
                );
                break 'complete;
            } else {
                MupDereferenceUncProvider((*MasterQueryContext).LatestProvider);
            }
        }

        MupDereferenceKnownPrefix((*MasterQueryContext).Prefix);
        ExReleaseResourceLite(addr_of_mut!(MupPrefixTableLock));

        Status = (*MasterQueryContext).LatestStatus;
    }

    // Complete:
    (*(*MasterQueryContext).Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest((*MasterQueryContext).Irp, IO_DISK_INCREMENT);

    (*MasterQueryContext).Irp = null_mut();
    MupFreeMasterQueryContext(MasterQueryContext);

    Status
}

pub unsafe extern "system" fn QueryPathCompletionRoutine(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    let mut LatestPos: ULONG;
    let mut Pos: ULONG;
    let mut AcceptedPrefix: *mut u16;

    let QueryContext = Context as PQUERY_PATH_CONTEXT;
    let QueryResponse = (*QueryContext).QueryPathRequest as PQUERY_PATH_RESPONSE;
    let MasterQueryContext = (*QueryContext).MasterQueryContext;
    let Status = (*Irp).IoStatus.__bindgen_anon_1.Status;

    ExAcquireResourceExclusiveLite(addr_of_mut!((*MasterQueryContext).QueryPathListLock), TRUE);
    RemoveEntryList(addr_of_mut!((*QueryContext).QueryPathListEntry));

    'cleanup: {
        if NT_SUCCESS(Status) && (*QueryResponse).LengthAccepted > 0 {
            let Prefix = (*MasterQueryContext).Prefix;

            if !(*MasterQueryContext).LatestProvider.is_null() {
                if (*(*QueryContext).UncProvider).ProviderOrder
                    >= (*(*MasterQueryContext).LatestProvider).ProviderOrder
                {
                    MupDereferenceUncProvider((*QueryContext).UncProvider);
                    break 'cleanup;
                }

                ExAcquireResourceExclusiveLite(addr_of_mut!(MupPrefixTableLock), TRUE);
                if (*Prefix).InTable != 0 {
                    RtlRemoveUnicodePrefix(addr_of_mut!(MupPrefixTable), addr_of_mut!((*Prefix).PrefixTableEntry));
                    RemoveEntryList(addr_of_mut!((*Prefix).PrefixListEntry));
                    (*Prefix).InTable = FALSE;
                }
                ExReleaseResourceLite(addr_of_mut!(MupPrefixTableLock));

                (*Prefix).KeepExtraRef = FALSE;

                if (*Prefix).AcceptedPrefix.Length != 0 && !(*Prefix).AcceptedPrefix.Buffer.is_null() {
                    ExFreePoolWithTag((*Prefix).AcceptedPrefix.Buffer as PVOID, TAG_MUP);
                    (*Prefix).AcceptedPrefix.MaximumLength = 0;
                    (*Prefix).AcceptedPrefix.Length = 0;
                    (*Prefix).AcceptedPrefix.Buffer = null_mut();
                    (*Prefix).ExternalAlloc = FALSE;
                }

                if !(*Prefix).UncProvider.is_null() {
                    MupDereferenceUncProvider((*Prefix).UncProvider);
                    (*Prefix).UncProvider = null_mut();
                }
            }

            (*MasterQueryContext).LatestProvider = (*QueryContext).UncProvider;
            (*MasterQueryContext).LatestStatus = Status;

            if (*(*MasterQueryContext).FileObject).FsContext2 != DFS_DOWNLEVEL_OPEN_CONTEXT as PVOID {
                AcceptedPrefix = ExAllocatePoolWithTag(
                    _POOL_TYPE_PagedPool,
                    (*QueryResponse).LengthAccepted as SIZE_T,
                    TAG_MUP,
                ) as *mut u16;
                if AcceptedPrefix.is_null() {
                    (*Prefix).InTable = FALSE;
                } else {
                    RtlMoveMemory(
                        AcceptedPrefix as PVOID,
                        (*(*MasterQueryContext).FileObject).FileName.Buffer as PVOID,
                        (*QueryResponse).LengthAccepted as SIZE_T,
                    );
                    (*Prefix).UncProvider = (*MasterQueryContext).LatestProvider;
                    (*Prefix).AcceptedPrefix.Buffer = AcceptedPrefix;
                    (*Prefix).AcceptedPrefix.Length = (*QueryResponse).LengthAccepted as USHORT;
                    (*Prefix).AcceptedPrefix.MaximumLength = (*QueryResponse).LengthAccepted as USHORT;
                    (*Prefix).ExternalAlloc = TRUE;

                    ExAcquireResourceExclusiveLite(addr_of_mut!(MupPrefixTableLock), TRUE);
                    if RtlInsertUnicodePrefix(
                        addr_of_mut!(MupPrefixTable),
                        addr_of_mut!((*Prefix).AcceptedPrefix),
                        addr_of_mut!((*Prefix).PrefixTableEntry),
                    ) != 0
                    {
                        InsertHeadList(addr_of_mut!(MupPrefixList), addr_of_mut!((*Prefix).PrefixListEntry));
                        (*Prefix).InTable = TRUE;
                        (*Prefix).KeepExtraRef = TRUE;
                    } else {
                        (*Prefix).InTable = FALSE;
                    }
                    ExReleaseResourceLite(addr_of_mut!(MupPrefixTableLock));
                }
            }
        } else {
            MupDereferenceUncProvider((*QueryContext).UncProvider);

            if (*MasterQueryContext).LatestProvider.is_null() {
                if NT_SUCCESS((*MasterQueryContext).LatestStatus) {
                    (*MasterQueryContext).LatestStatus = Status;
                } else {
                    let mut TableStatus = MupOrderedErrorList[0];
                    LatestPos = 0;

                    while TableStatus != 0 && TableStatus != (*MasterQueryContext).LatestStatus {
                        LatestPos += 1;
                        TableStatus = MupOrderedErrorList[LatestPos as usize];
                    }

                    if LatestPos == 0 {
                        (*MasterQueryContext).LatestStatus = Status;
                    } else {
                        Pos = 0;
                        loop {
                            if Status == MupOrderedErrorList[Pos as usize] {
                                break;
                            }
                            Pos += 1;
                            if Pos >= LatestPos {
                                break;
                            }
                        }

                        if Pos >= LatestPos {
                            (*MasterQueryContext).LatestStatus = Status;
                        }
                    }
                }
            }
        }
    }

    // Cleanup:
    ExFreePoolWithTag(QueryResponse as PVOID, TAG_MUP);
    ExFreePoolWithTag(QueryContext as PVOID, TAG_MUP);
    IoFreeIrp(Irp);

    ExReleaseResourceLite(addr_of_mut!((*MasterQueryContext).QueryPathListLock));
    MupDereferenceMasterQueryContext(MasterQueryContext);

    STATUS_MORE_PROCESSING_REQUIRED
}

pub unsafe fn CreateRedirectedFile(
    Irp: PIRP,
    FileObject: PFILE_OBJECT,
    SecurityContext: PIO_SECURITY_CONTEXT,
) -> NTSTATUS {
    let mut Len: LONG;
    let mut Cur: u16 = 0;
    let mut Name: *mut u16;
    let mut QueryIrp: PIRP;
    let mut Status: NTSTATUS;
    let mut Prefix: PMUP_PFX;
    let mut UncProvider: PMUP_UNC = null_mut();
    let mut CurrentTime: LARGE_INTEGER = core::mem::zeroed();
    let mut MasterQueryContext: PMUP_MQC;
    let mut QueryContext: PQUERY_PATH_CONTEXT;
    let mut QueryPathRequest: PQUERY_PATH_REQUEST;
    let mut Locked: BOOLEAN = FALSE;
    let mut Referenced: BOOLEAN = FALSE;
    let mut BreakOnFirst: BOOLEAN = TRUE;

    if (*FileObject).FileName.Length == 0 {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_DEVICE_REQUEST;
        IoCompleteRequest(Irp, IO_DISK_INCREMENT);
        return STATUS_INVALID_DEVICE_REQUEST;
    }

    Status = STATUS_BAD_NETWORK_PATH;

    ExAcquireResourceExclusiveLite(addr_of_mut!(MupPrefixTableLock), TRUE);
    let TableEntry = RtlFindUnicodePrefix(addr_of_mut!(MupPrefixTable), addr_of_mut!((*FileObject).FileName), 1);
    if !TableEntry.is_null() {
        Prefix = CONTAINING_RECORD!(TableEntry, MUP_PFX, PrefixTableEntry);

        KeQuerySystemTime(addr_of_mut!(CurrentTime));
        if (*Prefix).ValidityTimeout.QuadPart < CurrentTime.QuadPart {
            MupCalculateTimeout(addr_of_mut!((*Prefix).ValidityTimeout));
            Status = MupRerouteOpen(FileObject, (*Prefix).UncProvider);
            ExReleaseResourceLite(addr_of_mut!(MupPrefixTableLock));

            if Status == STATUS_REPARSE {
                (*Irp).IoStatus.Information = FILE_SUPERSEDED as ULONG_PTR;
            }

            (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
            IoCompleteRequest(Irp, IO_DISK_INCREMENT);
            return Status;
        }

        if (*Prefix).InTable != 0 {
            MupRemoveKnownPrefixEntry(Prefix);
        }
    }
    ExReleaseResourceLite(addr_of_mut!(MupPrefixTableLock));

    let Stack = IoGetCurrentIrpStackLocation(Irp);
    if *(*FileObject).FileName.Buffer == '\\' as u16
        && (*Stack).MajorFunction as usize != IRP_MJ_CREATE
    {
        Name = (*FileObject).FileName.Buffer.add(1);
        Len = (*FileObject).FileName.Length as LONG;

        loop {
            Len -= core::mem::size_of::<u16>() as LONG;
            if Len <= 0 {
                break;
            }
            Cur = *Name;
            Name = Name.add(1);
            if Cur == '\\' as u16 {
                break;
            }
        }
        Len -= core::mem::size_of::<u16>() as LONG;

        // sizeof(L"Mailslot") - sizeof(UNICODE_NULL) == 16 bytes
        if Len >= 16 {
            Len /= core::mem::size_of::<u16>() as LONG;
            if Len > 8 {
                Len = 8;
            }

            if _wcsnicmp(Name, MAILSLOT_NAME.as_ptr(), Len as usize) == 0 {
                Status = BroadcastOpen(Irp);
                if Status == STATUS_REPARSE {
                    (*Irp).IoStatus.Information = FILE_SUPERSEDED as ULONG_PTR;
                }
                (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
                IoCompleteRequest(Irp, IO_DISK_INCREMENT);
                return Status;
            }
        }
    }

    if MupEnableDfs == 0 || (*FileObject).FsContext2 == DFS_DOWNLEVEL_OPEN_CONTEXT as PVOID {
        IoMarkIrpPending(Irp);

        Prefix = MupAllocatePrefixEntry(0);
        if Prefix.is_null() {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INSUFFICIENT_RESOURCES;
            IoCompleteRequest(Irp, IO_DISK_INCREMENT);
            return STATUS_PENDING;
        }

        MasterQueryContext = MupAllocateMasterQueryContext();
        if MasterQueryContext.is_null() {
            MupFreeNode(Prefix as PVOID);
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INSUFFICIENT_RESOURCES;
            IoCompleteRequest(Irp, IO_DISK_INCREMENT);
            return STATUS_PENDING;
        }

        (*MasterQueryContext).Irp = Irp;
        (*MasterQueryContext).FileObject = FileObject;
        (*MasterQueryContext).LatestProvider = null_mut();
        (*MasterQueryContext).Prefix = Prefix;
        (*MasterQueryContext).LatestStatus = STATUS_BAD_NETWORK_PATH;
        ExAcquireResourceExclusiveLite(addr_of_mut!(MupGlobalLock), TRUE);
        InsertTailList(addr_of_mut!(MupMasterQueryList), addr_of_mut!((*MasterQueryContext).MQCListEntry));
        (*Prefix).NodeReferences += 1;
        ExReleaseResourceLite(addr_of_mut!(MupGlobalLock));

        // _SEH2_TRY/_SEH2_FINALLY
        'try_blk: {
            ExAcquireResourceExclusiveLite(addr_of_mut!(MupGlobalLock), TRUE);
            Locked = TRUE;

            let head = addr_of_mut!(MupProviderList);
            let mut Entry = (*head).Flink;
            while Entry != head {
                UncProvider = CONTAINING_RECORD!(Entry, MUP_UNC, ProviderListEntry);

                (*UncProvider).NodeReferences += 1;
                Referenced = TRUE;

                ExReleaseResourceLite(addr_of_mut!(MupGlobalLock));
                Locked = FALSE;

                if (*UncProvider).Registered != 0 {
                    QueryPathRequest = ExAllocatePoolWithTag(
                        _POOL_TYPE_PagedPool,
                        ((*FileObject).FileName.Length as ULONG
                            + core::mem::size_of::<QUERY_PATH_REQUEST>() as ULONG)
                            as SIZE_T,
                        TAG_MUP,
                    ) as PQUERY_PATH_REQUEST;
                    if QueryPathRequest.is_null() {
                        ExRaiseStatus(STATUS_INSUFFICIENT_RESOURCES);
                    }

                    QueryContext = ExAllocatePoolWithTag(
                        _POOL_TYPE_PagedPool,
                        core::mem::size_of::<QUERY_PATH_CONTEXT>() as SIZE_T,
                        TAG_MUP,
                    ) as PQUERY_PATH_CONTEXT;
                    if QueryContext.is_null() {
                        ExFreePoolWithTag(QueryPathRequest as PVOID, TAG_MUP);
                        ExRaiseStatus(STATUS_INSUFFICIENT_RESOURCES);
                    }

                    InitializeListHead(addr_of_mut!((*QueryContext).QueryPathListEntry));
                    (*QueryContext).MasterQueryContext = MasterQueryContext;
                    (*QueryContext).QueryPathRequest = QueryPathRequest;
                    (*QueryPathRequest).PathNameLength = (*FileObject).FileName.Length as ULONG;
                    (*QueryPathRequest).SecurityContext = SecurityContext;
                    RtlMoveMemory(
                        addr_of_mut!((*QueryPathRequest).FilePathName) as PVOID,
                        (*FileObject).FileName.Buffer as PVOID,
                        (*FileObject).FileName.Length as SIZE_T,
                    );

                    QueryIrp = MupBuildIoControlRequest(
                        (*UncProvider).FileObject,
                        QueryContext as PVOID,
                        IRP_MJ_DEVICE_CONTROL as ULONG,
                        IOCTL_REDIR_QUERY_PATH,
                        QueryPathRequest as PVOID,
                        (*FileObject).FileName.Length as ULONG
                            + core::mem::size_of::<QUERY_PATH_REQUEST>() as ULONG,
                        QueryPathRequest as PVOID,
                        core::mem::size_of::<QUERY_PATH_RESPONSE>() as ULONG,
                        core::mem::transmute::<CompletionFn, PIO_COMPLETION_ROUTINE>(
                            QueryPathCompletionRoutine,
                        ),
                    );
                    if QueryIrp.is_null() {
                        ExFreePoolWithTag(QueryContext as PVOID, TAG_MUP);
                        ExFreePoolWithTag(QueryPathRequest as PVOID, TAG_MUP);
                        ExRaiseStatus(STATUS_INSUFFICIENT_RESOURCES);
                    }

                    (*QueryIrp).RequestorMode = KernelMode;
                    (*QueryContext).UncProvider = UncProvider;
                    (*QueryContext).Irp = QueryIrp;

                    ExAcquireResourceExclusiveLite(addr_of_mut!(MupGlobalLock), TRUE);
                    (*UncProvider).NodeReferences += 1;
                    (*MasterQueryContext).NodeReferences += 1;
                    ExReleaseResourceLite(addr_of_mut!(MupGlobalLock));

                    ExAcquireResourceExclusiveLite(addr_of_mut!((*MasterQueryContext).QueryPathListLock), TRUE);
                    InsertTailList(
                        addr_of_mut!((*MasterQueryContext).QueryPathList),
                        addr_of_mut!((*QueryContext).QueryPathListEntry),
                    );
                    ExReleaseResourceLite(addr_of_mut!((*MasterQueryContext).QueryPathListLock));

                    Status = IoCallDriver((*UncProvider).DeviceObject, QueryIrp);
                }

                ExAcquireResourceExclusiveLite(addr_of_mut!(MupGlobalLock), TRUE);
                Locked = TRUE;

                MupDereferenceUncProvider(UncProvider);
                Referenced = FALSE;

                if BreakOnFirst != 0 && Status == STATUS_SUCCESS {
                    break;
                }

                BreakOnFirst = FALSE;

                Entry = (*Entry).Flink;
            }
        }

        // _SEH2_FINALLY (AbnormalTermination() == false under DUMMY_PSEH)
        if Referenced != 0 {
            MupDereferenceUncProvider(UncProvider);
        }
        if Locked != 0 {
            ExReleaseResourceLite(addr_of_mut!(MupGlobalLock));
        }
        MupDereferenceMasterQueryContext(MasterQueryContext);
        Status = STATUS_PENDING;
    } else {
        Status = STATUS_NOT_IMPLEMENTED;
    }

    Status
}

pub unsafe fn OpenMupFileSystem(
    Vcb: PMUP_VCB,
    FileObject: PFILE_OBJECT,
    DesiredAccess: ULONG,
    ShareAccess: USHORT,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    ExAcquireResourceExclusiveLite(addr_of_mut!(MupVcbLock), TRUE);
    // _SEH2_TRY/_SEH2_FINALLY
    {
        Status = IoCheckShareAccess(
            DesiredAccess,
            ShareAccess as ULONG,
            FileObject,
            addr_of_mut!((*Vcb).ShareAccess),
            TRUE,
        );
        if NT_SUCCESS(Status) {
            (*Vcb).NodeReferences += 1;
            MupSetFileObject(FileObject, Vcb as PMUP_FCB, null_mut());
            Status = STATUS_SUCCESS;
        }
    }
    // finally:
    ExReleaseResourceLite(addr_of_mut!(MupVcbLock));

    Status
}

pub unsafe extern "system" fn MupCreate(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    FsRtlEnterFileSystem();

    // _SEH2_TRY/_SEH2_EXCEPT
    {
        if MupEnableDfs != 0
            && ((*DeviceObject).DeviceType == FILE_DEVICE_DFS
                || (*DeviceObject).DeviceType == FILE_DEVICE_DFS_FILE_SYSTEM)
        {
            Status = DfsFsdCreate(DeviceObject, Irp);
        } else {
            let Stack = IoGetCurrentIrpStackLocation(Irp);
            let FileObject = (*Stack).FileObject;
            let RelatedFileObject = (*FileObject).RelatedFileObject;

            if (*FileObject).FileName.Length != 0
                || (!RelatedFileObject.is_null()
                    && (*((*RelatedFileObject).FsContext as PMUP_FCB)).NodeType != NODE_TYPE_VCB)
            {
                Status = CreateRedirectedFile(Irp, FileObject, (*Stack).Parameters.Create.SecurityContext);
            } else {
                Status = OpenMupFileSystem(
                    (*DeviceObject).DeviceExtension as PMUP_VCB,
                    FileObject,
                    (*(*Stack).Parameters.Create.SecurityContext).DesiredAccess,
                    (*Stack).Parameters.Create.ShareAccess,
                );

                (*Irp).IoStatus.Information = FILE_OPENED as ULONG_PTR;
                (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
                IoCompleteRequest(Irp, IO_DISK_INCREMENT);
            }
        }
    }

    FsRtlExitFileSystem();

    Status
}

pub unsafe fn MupCloseUncProvider(UncProvider: PMUP_UNC) {
    ExAcquireResourceExclusiveLite(addr_of_mut!(MupGlobalLock), TRUE);

    if (*UncProvider).NodeStatus == NODE_STATUS_HEALTHY {
        (*UncProvider).NodeStatus = NODE_STATUS_CLEANUP;
        (*UncProvider).Registered = FALSE;
        ExReleaseResourceLite(addr_of_mut!(MupGlobalLock));

        if !(*UncProvider).FileObject.is_null() {
            ZwClose((*UncProvider).DeviceHandle);
            ObDereferenceObject((*UncProvider).FileObject as PVOID);
        }
    } else {
        ExReleaseResourceLite(addr_of_mut!(MupGlobalLock));
    }
}

pub unsafe extern "system" fn MupCleanup(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let mut Type: ULONG;
    let mut Fcb: PMUP_FCB = null_mut();
    let mut Ccb: PMUP_CCB = null_mut();
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if MupEnableDfs != 0
        && ((*DeviceObject).DeviceType == FILE_DEVICE_DFS
            || (*DeviceObject).DeviceType == FILE_DEVICE_DFS_FILE_SYSTEM)
    {
        return DfsFsdCleanup(DeviceObject, Irp);
    }

    FsRtlEnterFileSystem();

    // _SEH2_TRY/_SEH2_EXCEPT
    {
        let Stack = IoGetCurrentIrpStackLocation(Irp);
        Type = MupDecodeFileObject((*Stack).FileObject, addr_of_mut!(Fcb), addr_of_mut!(Ccb));
        if Type == NODE_TYPE_VCB {
            MupCleanupVcb(DeviceObject, Irp, Fcb as PMUP_VCB);

            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            IoCompleteRequest(Irp, IO_DISK_INCREMENT);

            MupDereferenceVcb(Fcb as PMUP_VCB);

            if !Ccb.is_null() {
                MupCloseUncProvider(Ccb as PMUP_UNC);
                MupDereferenceUncProvider(Ccb as PMUP_UNC);
                ExAcquireResourceExclusiveLite(addr_of_mut!(MupGlobalLock), TRUE);
                MupProviderCount -= 1;
                ExReleaseResourceLite(addr_of_mut!(MupGlobalLock));
            }

            Status = STATUS_SUCCESS;
        } else if Type == NODE_TYPE_FCB {
            if (*Fcb).NodeStatus == NODE_STATUS_HEALTHY {
                MupCleanupFcb(DeviceObject, Irp, Fcb);
                Status = STATUS_SUCCESS;
            } else {
                Status = STATUS_INVALID_HANDLE;
            }

            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            IoCompleteRequest(Irp, IO_DISK_INCREMENT);

            MupDereferenceFcb(Fcb);
        } else {
            Status = STATUS_INVALID_HANDLE;

            (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
            IoCompleteRequest(Irp, IO_DISK_INCREMENT);
        }
    }

    FsRtlExitFileSystem();

    Status
}

pub unsafe fn MupCloseVcb(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Vcb: PMUP_VCB,
    FileObject: PFILE_OBJECT,
) -> NTSTATUS {
    ExAcquireResourceExclusiveLite(addr_of_mut!(MupGlobalLock), TRUE);
    MupSetFileObject(FileObject, null_mut(), null_mut());
    MupDereferenceVcb(Vcb);
    ExReleaseResourceLite(addr_of_mut!(MupGlobalLock));
    STATUS_SUCCESS
}

pub unsafe fn MupCloseFcb(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Fcb: PMUP_FCB,
    FileObject: PFILE_OBJECT,
) -> NTSTATUS {
    ExAcquireResourceExclusiveLite(addr_of_mut!(MupGlobalLock), TRUE);
    MupSetFileObject(FileObject, null_mut(), null_mut());
    MupDereferenceFcb(Fcb);
    ExReleaseResourceLite(addr_of_mut!(MupGlobalLock));
    STATUS_SUCCESS
}

pub unsafe extern "system" fn MupClose(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let mut Fcb: PMUP_FCB = null_mut();
    let mut Ccb: PMUP_CCB = null_mut();
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if MupEnableDfs != 0
        && ((*DeviceObject).DeviceType == FILE_DEVICE_DFS
            || (*DeviceObject).DeviceType == FILE_DEVICE_DFS_FILE_SYSTEM)
    {
        return DfsFsdClose(DeviceObject, Irp);
    }

    FsRtlEnterFileSystem();

    'try_blk: {
        let Stack = IoGetCurrentIrpStackLocation(Irp);
        MupDecodeFileObject((*Stack).FileObject, addr_of_mut!(Fcb), addr_of_mut!(Ccb));
        if Fcb.is_null() {
            Status = STATUS_INVALID_HANDLE;
            (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
            IoCompleteRequest(Irp, IO_DISK_INCREMENT);
            break 'try_blk;
        }

        if (*Fcb).NodeType == NODE_TYPE_VCB {
            Status = MupCloseVcb(DeviceObject, Irp, Fcb as PMUP_VCB, (*Stack).FileObject);
        } else if (*Fcb).NodeType == NODE_TYPE_FCB {
            MupDereferenceFcb(Fcb);
            Status = MupCloseFcb(DeviceObject, Irp, Fcb, (*Stack).FileObject);
        } else {
            Status = STATUS_INVALID_HANDLE;
            (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
            IoCompleteRequest(Irp, IO_DISK_INCREMENT);
            break 'try_blk;
        }

        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        IoCompleteRequest(Irp, IO_DISK_INCREMENT);
    }

    FsRtlExitFileSystem();

    Status
}

pub unsafe extern "system" fn MupUnload(DriverObject: PDRIVER_OBJECT) {
    IoDeleteDevice(mupDeviceObject);

    if MupEnableDfs != 0 {
        DfsUnload(DriverObject);
    }

    MupUninitializeData();
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    let mut MupString: UNICODE_STRING = core::mem::zeroed();
    let mut DeviceObject: PDEVICE_OBJECT = null_mut();

    MupInitializeData();

    MupEnableDfs = MuppIsDfsEnabled();
    if MupEnableDfs != 0 && !NT_SUCCESS(DfsDriverEntry(DriverObject, RegistryPath)) {
        MupEnableDfs = FALSE;
    }

    RtlInitUnicodeString(addr_of_mut!(MupString), MUP_DEVICE_NAME.as_ptr());
    let Status = IoCreateDevice(
        DriverObject,
        core::mem::size_of::<MUP_VCB>() as ULONG,
        addr_of_mut!(MupString),
        FILE_DEVICE_MULTI_UNC_PROVIDER,
        0,
        FALSE,
        addr_of_mut!(DeviceObject),
    );
    if !NT_SUCCESS(Status) {
        if MupEnableDfs != 0 {
            DfsUnload(DriverObject);
        }
        MupUninitializeData();
        return Status;
    }

    (*DriverObject).DriverUnload =
        core::mem::transmute::<UnloadFn, PDRIVER_UNLOAD>(MupUnload);
    (*DriverObject).MajorFunction[IRP_MJ_CREATE] =
        core::mem::transmute::<DispatchFn, PDRIVER_DISPATCH>(MupCreate);
    (*DriverObject).MajorFunction[IRP_MJ_CREATE_NAMED_PIPE] =
        core::mem::transmute::<DispatchFn, PDRIVER_DISPATCH>(MupCreate);
    (*DriverObject).MajorFunction[IRP_MJ_CREATE_MAILSLOT] =
        core::mem::transmute::<DispatchFn, PDRIVER_DISPATCH>(MupCreate);
    (*DriverObject).MajorFunction[IRP_MJ_WRITE] =
        core::mem::transmute::<DispatchFn, PDRIVER_DISPATCH>(MupForwardIoRequest);
    (*DriverObject).MajorFunction[IRP_MJ_FILE_SYSTEM_CONTROL] =
        core::mem::transmute::<DispatchFn, PDRIVER_DISPATCH>(MupFsControl);
    (*DriverObject).MajorFunction[IRP_MJ_CLEANUP] =
        core::mem::transmute::<DispatchFn, PDRIVER_DISPATCH>(MupCleanup);
    (*DriverObject).MajorFunction[IRP_MJ_CLOSE] =
        core::mem::transmute::<DispatchFn, PDRIVER_DISPATCH>(MupClose);

    mupDeviceObject = DeviceObject;
    MupInitializeVcb((*DeviceObject).DeviceExtension as PMUP_VCB);

    STATUS_SUCCESS
}
