use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ex::*;

use ntkrnl_sys::ddk::{
    _ETHREAD, PETHREAD, PDEVICE_OBJECT, PDRIVER_OBJECT, PIRP, PVPB,
    POBJECT_NAME_INFORMATION, _OBJECT_NAME_INFORMATION, PUNICODE_STRING,
    PKAPC, PKDPC, PKNORMAL_ROUTINE, _KAPC, _KDPC, _KTIMER,
    WORK_QUEUE_ITEM, _WORK_QUEUE_ITEM, PORT_MESSAGE, _PORT_MESSAGE,
    IO_ERROR_LOG_PACKET, PIO_ERROR_LOG_PACKET, ExQueueWorkItem,
    ObQueryNameString, KeInitializeApc, KeInsertQueueApc, KeInitializeDpc,
    KeInitializeTimer, KeSetTimer, ZwConnectPort, ZwRequestPort, ZwClose,
    SECURITY_QUALITY_OF_SERVICE, _SECURITY_QUALITY_OF_SERVICE,
    ExAllocatePool, ExFreePool,
    PsGetCurrentThread, KeAreApcsDisabled, RtlInitEmptyUnicodeString,
    HANDLE, CSHORT,
    _UNICODE_STRING,
};

use ntkrnl_sys::ddk::_WORK_QUEUE_TYPE_DelayedWorkQueue as DelayedWorkQueue;
use ntkrnl_sys::ddk::_SECURITY_IMPERSONATION_LEVEL_SecurityIdentification as SecurityIdentification;

// C #define constants not in bindgen
const IO_TYPE_DEVICE: USHORT = 3;
const IO_TYPE_DRIVER: USHORT = 4;
const IO_TYPE_ERROR_LOG: USHORT = 11;
const IO_TYPE_ERROR_MESSAGE: USHORT = 12;

#[cfg(not(target_arch = "x86"))]
use ntkrnl_sys::ddk::ExInterlockedInsertHeadList;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


#[cfg(target_arch = "x86")]
unsafe fn ExInterlockedInsertHeadList(
    ListHead: PLIST_ENTRY,
    ListEntry: PLIST_ENTRY,
    Lock: *mut KSPIN_LOCK,
) -> PLIST_ENTRY {
    extern "fastcall" {
        fn ExfInterlockedInsertHeadList(
            ListHead: PLIST_ENTRY,
            ListEntry: PLIST_ENTRY,
            Lock: *mut KSPIN_LOCK,
        ) -> PLIST_ENTRY;
    }
    ExfInterlockedInsertHeadList(ListHead, ListEntry, Lock)
}

extern "system" {
    fn KeQuerySystemTime(CurrentTime: *mut LARGE_INTEGER);
    fn IoCompleteRequest(Irp: PIRP, PriorityBoost: i8);
}

// ExInitializeWorkItem is a FORCEINLINE macro
#[inline(always)]
unsafe fn ExInitializeWorkItem(Item: *mut WORK_QUEUE_ITEM, Routine: PVOID, Context: PVOID) {
    (*Item).WorkerRoutine = core::mem::transmute(Routine);
    (*Item).Parameter = Context;
    (*Item).List = core::mem::zeroed();
}

extern "system" {
}

const TAG_IO: ULONG = 0x20206F49;
const TAG_ERROR_LOG: ULONG = 0x72456F49;
const TAG_APC: ULONG = 0x4350414B;

const ELF_PORT_NAME: &[u16] = &[
    b'\\' as u16, b'E' as u16, b'r' as u16, b'r' as u16, b'o' as u16, b'r' as u16,
    b'L' as u16, b'o' as u16, b'g' as u16, b'P' as u16, b'o' as u16, b'r' as u16,
    b't' as u16, 0
];

const ERROR_LOG_MAXIMUM_SIZE: UCHAR = 255;
const IO_ERROR_OBJECT_NAMES_LENGTH: usize = 100;
const IO_ERROR_LOG_MESSAGE_LENGTH: usize = 256 + core::mem::size_of::<IO_ERROR_LOG_MESSAGE>();

#[repr(C)]
struct ERROR_LOG_ENTRY {
    Type: CSHORT,
    Size: USHORT,
    DeviceObject: PDEVICE_OBJECT,
    DriverObject: PDRIVER_OBJECT,
    TimeStamp: LARGE_INTEGER,
    ListEntry: LIST_ENTRY,
}
type PERROR_LOG_ENTRY = *mut ERROR_LOG_ENTRY;

#[repr(C)]
struct IO_ERROR_LOG_MESSAGE {
    Type: USHORT,
    Size: USHORT,
    DriverNameLength: USHORT,
    TimeStamp: LARGE_INTEGER,
    DriverNameOffset: ULONG,
    EntryData: IO_ERROR_LOG_PACKET,
}
type PIO_ERROR_LOG_MESSAGE = *mut IO_ERROR_LOG_MESSAGE;

#[repr(C)]
struct ELF_API_MSG {
    Header: PORT_MESSAGE,
    Unknown: [UCHAR; 2],
    IoErrorMessage: IO_ERROR_LOG_MESSAGE,
}
type PELF_API_MSG = *mut ELF_API_MSG;

#[repr(C)]
struct IOP_ERROR_LOG_WORKER_DPC {
    Dpc: _KDPC,
    Timer: _KTIMER,
}
type PIOP_ERROR_LOG_WORKER_DPC = *mut IOP_ERROR_LOG_WORKER_DPC;

const IOP_MAXIMUM_LOG_SIZE: LONG = 100 * 4096;

#[no_mangle]
pub static mut IopTotalLogSize: LONG = 0;
#[no_mangle]
pub static mut IopErrorLogListHead: LIST_ENTRY = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut IopLogListLock: KSPIN_LOCK = 0;

static mut IopLogWorkerRunning: BOOLEAN = 0u8;
static mut IopLogPortConnected: BOOLEAN = 0u8;
static mut IopLogPort: HANDLE = 0 as HANDLE;
#[no_mangle]
pub static mut IopErrorLogWorkItem: WORK_QUEUE_ITEM = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut IopErrorLogObject: PDEVICE_OBJECT = core::ptr::null_mut();

unsafe extern "system" fn IopLogDpcRoutine(
    Dpc: PKDPC,
    _DeferredContext: PVOID,
    _SystemArgument1: PVOID,
    _SystemArgument2: PVOID,
) {
    IopLogDpcRoutineInner(Dpc, _DeferredContext, _SystemArgument1, _SystemArgument2);
}
#[inline(always)]
unsafe fn IopLogDpcRoutineInner(
    Dpc: PKDPC,
    _DeferredContext: PVOID,
    _SystemArgument1: PVOID,
    _SystemArgument2: PVOID,
) {
    if !Dpc.is_null() {
        ExFreePool(Dpc as PVOID);
    }

    ExInitializeWorkItem(
        core::ptr::addr_of_mut!(IopErrorLogWorkItem),
        IopLogWorker as PVOID,
        core::ptr::null_mut(),
    );
    ExQueueWorkItem(
        core::ptr::addr_of_mut!(IopErrorLogWorkItem),
        DelayedWorkQueue,
    );
}

unsafe fn IopGetErrorLogEntry() -> PLIST_ENTRY {
    let mut OldIrql: KIRQL = 0;
    let ListEntry: PLIST_ENTRY;

    // [guard] KeAcquireSpinLock(core::ptr::addr_of_mut!(IopLogListLock), &mut OldIrql);
    let spin_guard = unsafe { ntkrnl_sys::guards::SpinLockGuard::acquire(core::ptr::addr_of_mut!(IopLogListLock)) };
    if IsListEmpty(core::ptr::addr_of_mut!(IopErrorLogListHead)) {
        IopLogWorkerRunning = 0u8;
        ListEntry = core::ptr::null_mut();
    } else {
        ListEntry = RemoveHeadList(core::ptr::addr_of_mut!(IopErrorLogListHead));
    }

    drop(spin_guard);
    ListEntry
}

unsafe fn IopRestartLogWorker() {
    let WorkerDpc = ExAllocatePool(NonPagedPool, core::mem::size_of::<IOP_ERROR_LOG_WORKER_DPC>() as _) as PIOP_ERROR_LOG_WORKER_DPC;
    if WorkerDpc.is_null() {
        IopLogWorkerRunning = 0u8;
        return;
    }

    KeInitializeDpc(
        core::ptr::addr_of_mut!((*WorkerDpc).Dpc),
        Some(IopLogDpcRoutine),
        WorkerDpc as PVOID,
    );
    KeInitializeTimer(core::ptr::addr_of_mut!((*WorkerDpc).Timer));

    let mut Timeout = ntkrnl_sys::ddk::_LARGE_INTEGER { QuadPart: -300000000i64 };
    KeSetTimer(
        core::ptr::addr_of_mut!((*WorkerDpc).Timer),
        Timeout,
        core::ptr::addr_of_mut!((*WorkerDpc).Dpc),
    );
}

unsafe fn IopConnectLogPort() -> BOOLEAN {
    if IopLogPortConnected != 0 {
        return 1u8;
    }

    let mut PortName = _UNICODE_STRING {
        Length: ((ELF_PORT_NAME.len() - 1) * 2) as USHORT,
        MaximumLength: (ELF_PORT_NAME.len() * 2) as USHORT,
        Buffer: ELF_PORT_NAME.as_ptr() as *mut u16,
    };

    let SecurityQos = SECURITY_QUALITY_OF_SERVICE {
        Length: core::mem::size_of::<SECURITY_QUALITY_OF_SERVICE>() as ULONG,
        ImpersonationLevel: SecurityIdentification,
        ContextTrackingMode: SECURITY_DYNAMIC_TRACKING as u8,
        EffectiveOnly: 1u8,
    };

    let Status = ZwConnectPort(
        core::ptr::addr_of_mut!(IopLogPort),
        &mut PortName,
        &SecurityQos as *const _ as *mut _,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    if Status >= 0 {
        IopLogPortConnected = 1u8;
        return 1u8;
    }

    IopRestartLogWorker();
    0u8
}

unsafe extern "system" fn IopLogWorker(_Parameter: PVOID) {
    if IopConnectLogPort() == 0 {
        return;
    }

    let Message = ExAllocatePoolWithTag(PagedPool, IO_ERROR_LOG_MESSAGE_LENGTH, TAG_IO) as PELF_API_MSG;
    if Message.is_null() {
        IopRestartLogWorker();
        return;
    }

    core::ptr::write_bytes(Message as *mut u8, 0, core::mem::size_of::<ELF_API_MSG>());
    let ErrorMessage = core::ptr::addr_of_mut!((*Message).IoErrorMessage);

    loop {
        let ListEntry = IopGetErrorLogEntry();
        if ListEntry.is_null() {
            break;
        }

        let LogEntry = ((ListEntry as usize) - core::mem::offset_of!(ERROR_LOG_ENTRY, ListEntry)) as PERROR_LOG_ENTRY;
        let Packet = ((LogEntry as usize) + core::mem::size_of::<ERROR_LOG_ENTRY>()) as PIO_ERROR_LOG_PACKET;

        let mut MessageLength = core::mem::size_of::<IO_ERROR_LOG_MESSAGE>()
            - core::mem::size_of::<ERROR_LOG_ENTRY>()
            - core::mem::size_of::<IO_ERROR_LOG_PACKET>()
            + (*LogEntry).Size as usize;

        core::ptr::copy_nonoverlapping(
            Packet as *const u8,
            core::ptr::addr_of_mut!((*ErrorMessage).EntryData) as *mut u8,
            (*LogEntry).Size as usize - core::mem::size_of::<ERROR_LOG_ENTRY>(),
        );

        (*ErrorMessage).TimeStamp = (*LogEntry).TimeStamp;
        (*ErrorMessage).Type = IO_TYPE_ERROR_MESSAGE;

        let mut StringBuffer: *mut u8;
        if (*Packet).NumberOfStrings != 0 {
            StringBuffer = (core::ptr::addr_of_mut!((*ErrorMessage).EntryData) as *mut u8)
                .add((*Packet).StringOffset as usize);
        } else {
            StringBuffer = (ErrorMessage as *mut u8).add(MessageLength);
        }

        StringBuffer = ((StringBuffer as usize + core::mem::align_of::<u16>() - 1) & !(core::mem::align_of::<u16>() - 1)) as *mut u8;

        (*ErrorMessage).DriverNameOffset = (StringBuffer as usize - ErrorMessage as usize) as ULONG;

        let mut RemainingLength = (Message as usize + IO_ERROR_LOG_MESSAGE_LENGTH - StringBuffer as usize) as ULONG;

        let mut NameString: *mut u16 = core::ptr::null_mut();
        let mut DriverNameLength: ULONG = 0;
        let mut DeviceNameLength: ULONG = 0;
        let mut PoolObjectNameInfo: POBJECT_NAME_INFORMATION = core::ptr::null_mut();
        let mut Buffer = [0u8; core::mem::size_of::<_OBJECT_NAME_INFORMATION>() + IO_ERROR_OBJECT_NAMES_LENGTH];
        let mut ObjectNameInfo = Buffer.as_mut_ptr() as POBJECT_NAME_INFORMATION;

        let DriverObject = (*LogEntry).DriverObject;
        if !DriverObject.is_null() {
            if !(*DriverObject).DriverName.Buffer.is_null() {
                NameString = (*DriverObject).DriverName.Buffer;
                DriverNameLength = (*DriverObject).DriverName.Length as ULONG;
            } else {
                NameString = core::ptr::null_mut();
                DriverNameLength = 0;
            }

            if DriverNameLength == 0 {
                let mut ReturnedLength: ULONG = 0;
                let Status = ObQueryNameString(
                    DriverObject as PVOID,
                    ObjectNameInfo,
                    Buffer.len() as ULONG,
                    &mut ReturnedLength,
                );
                if Status < 0 || (*ObjectNameInfo).Name.Length == 0 {
                    DriverNameLength = 0;
                } else {
                    NameString = (*ObjectNameInfo).Name.Buffer;
                    DriverNameLength = (*ObjectNameInfo).Name.Length as ULONG;
                }
            }
        } else {
            const APP_POPUP: [u16; 18] = [
                b'A' as u16, b'p' as u16, b'p' as u16, b'l' as u16, b'i' as u16, b'c' as u16,
                b'a' as u16, b't' as u16, b'i' as u16, b'o' as u16, b'n' as u16, b' ' as u16,
                b'P' as u16, b'o' as u16, b'p' as u16, b'u' as u16, b'p' as u16, 0
            ];
            NameString = APP_POPUP.as_ptr() as *mut u16;
            DriverNameLength = 17 * 2;
        }

        if DriverNameLength != 0 {
            let mut p = NameString.add((DriverNameLength / 2) as usize);
            DriverNameLength = 2;
            p = p.sub(1);
            while *p != b'\\' as u16 && p != NameString {
                p = p.sub(1);
                DriverNameLength += 2;
            }

            if *p == b'\\' as u16 {
                p = p.add(1);
                DriverNameLength -= 2;
            }

            DriverNameLength = core::cmp::min(DriverNameLength, RemainingLength - 3 * 2);
            core::ptr::copy_nonoverlapping(
                p as *const u8,
                StringBuffer,
                DriverNameLength as usize,
            );
        }

        *(StringBuffer.add(DriverNameLength as usize) as *mut u16) = 0;
        DriverNameLength += 2;

        StringBuffer = StringBuffer.add(DriverNameLength as usize);
        RemainingLength -= DriverNameLength;

        (*ErrorMessage).EntryData.StringOffset = (StringBuffer as usize - ErrorMessage as usize) as USHORT;

        if !(*LogEntry).DeviceObject.is_null() {
            let mut ReturnedLength: ULONG = 0;
            let mut Status = ObQueryNameString(
                (*LogEntry).DeviceObject as PVOID,
                ObjectNameInfo,
                (Buffer.len() - DriverNameLength as usize) as ULONG,
                &mut ReturnedLength,
            );
            if Status < 0 || (*ObjectNameInfo).Name.Length == 0 {
                RtlInitEmptyUnicodeString(
                    core::ptr::addr_of_mut!((*ObjectNameInfo).Name),
                    w!("").as_ptr() as *mut u16,
                    0,
                );

                if Status == STATUS_INFO_LENGTH_MISMATCH {
                    PoolObjectNameInfo = ExAllocatePoolWithTag(PagedPool, ReturnedLength as usize, TAG_IO) as POBJECT_NAME_INFORMATION;
                    if !PoolObjectNameInfo.is_null() {
                        ObjectNameInfo = PoolObjectNameInfo;
                        Status = ObQueryNameString(
                            (*LogEntry).DeviceObject as PVOID,
                            ObjectNameInfo,
                            ReturnedLength,
                            &mut ReturnedLength,
                        );
                        if Status >= 0 {
                            (*ObjectNameInfo).Name.Length = (IO_ERROR_OBJECT_NAMES_LENGTH - DriverNameLength as usize) as USHORT;
                        }
                    }
                }
            }

            NameString = (*ObjectNameInfo).Name.Buffer;
            DeviceNameLength = (*ObjectNameInfo).Name.Length as ULONG;
        } else {
            NameString = w!("").as_ptr() as *mut u16;
            DeviceNameLength = 0;
        }

        DeviceNameLength = core::cmp::min(DeviceNameLength, RemainingLength - 2 * 2);
        core::ptr::copy_nonoverlapping(
            NameString as *const u8,
            StringBuffer,
            DeviceNameLength as usize,
        );

        *(StringBuffer.add(DeviceNameLength as usize) as *mut u16) = 0;
        DeviceNameLength += 2;

        if !PoolObjectNameInfo.is_null() {
            ExFreePoolWithTag(PoolObjectNameInfo as PVOID, TAG_IO);
            PoolObjectNameInfo = core::ptr::null_mut();
        }

        (*ErrorMessage).EntryData.NumberOfStrings += 1;
        StringBuffer = StringBuffer.add(DeviceNameLength as usize);
        RemainingLength -= DeviceNameLength;

        if (*Packet).NumberOfStrings != 0 {
            let mut ExtraStringLength = (*LogEntry).Size as ULONG
                - core::mem::size_of::<ERROR_LOG_ENTRY>() as ULONG
                - (*Packet).StringOffset as ULONG;

            ExtraStringLength = (ExtraStringLength + 1) & !1 ;

            if ExtraStringLength > RemainingLength - 2 {
                MessageLength -= (ExtraStringLength - RemainingLength) as usize;
                ExtraStringLength = RemainingLength - 2;
            }

            core::ptr::copy_nonoverlapping(
                (Packet as *const u8).add((*Packet).StringOffset as usize),
                StringBuffer,
                ExtraStringLength as usize,
            );

            *(StringBuffer.add(ExtraStringLength as usize) as *mut u16) = 0;
        }

        (*ErrorMessage).DriverNameLength = DriverNameLength as USHORT;

        MessageLength += (DriverNameLength + DeviceNameLength) as usize;
        (*ErrorMessage).Size = MessageLength as USHORT;

        MessageLength += core::mem::offset_of!(ELF_API_MSG, IoErrorMessage)
            - core::mem::offset_of!(ELF_API_MSG, Unknown);

        (*Message).Header.u1.s1.TotalLength = (core::mem::size_of::<PORT_MESSAGE>() + MessageLength) as CSHORT;
        (*Message).Header.u1.s1.DataLength = MessageLength as CSHORT;

        let Status = ZwRequestPort(IopLogPort, core::ptr::addr_of_mut!((*Message).Header));
        if Status < 0 {
            ZwClose(IopLogPort);
            IopLogPortConnected = 0u8;

            ExInterlockedInsertHeadList(
                core::ptr::addr_of_mut!(IopErrorLogListHead) as PLIST_ENTRY,
                core::ptr::addr_of_mut!((*LogEntry).ListEntry) as PLIST_ENTRY,
                core::ptr::addr_of_mut!(IopLogListLock) as *mut _,
            );

            IopRestartLogWorker();
            break;
        }

        if !(*LogEntry).DeviceObject.is_null() {
            ObDereferenceObject((*LogEntry).DeviceObject as PVOID);
        }
        if !(*LogEntry).DriverObject.is_null() {
            ObDereferenceObject((*LogEntry).DriverObject as PVOID);
        }

        InterlockedExchangeAdd(core::ptr::addr_of_mut!(IopTotalLogSize), -((*LogEntry).Size as LONG));
        ExFreePoolWithTag(LogEntry as PVOID, TAG_ERROR_LOG);
    }

    ExFreePoolWithTag(Message as PVOID, TAG_IO);
}

unsafe extern "system" fn IopFreeApc(
    Apc: PKAPC,
    _NormalRoutine: *mut PKNORMAL_ROUTINE,
    _NormalContext: *mut PVOID,
    _SystemArgument1: *mut PVOID,
    _SystemArgument2: *mut PVOID,
) {
    ExFreePoolWithTag(Apc as PVOID, TAG_APC);
}

unsafe extern "system" fn IopRaiseHardError(
    NormalContext: PVOID,
    _SystemArgument1: PVOID,
    _SystemArgument2: PVOID,
) {
    IopRaiseHardErrorInner(NormalContext, _SystemArgument1, _SystemArgument2);
}
#[inline(always)]
unsafe fn IopRaiseHardErrorInner(
    NormalContext: PVOID,
    _SystemArgument1: PVOID,
    _SystemArgument2: PVOID,
) {
    let Irp = NormalContext as PIRP;

    (*Irp).IoStatus.Information = 0;
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_IMPLEMENTED;
    IoCompleteRequest(Irp, IO_DISK_INCREMENT);
}

#[no_mangle]
pub unsafe extern "system" fn IoAllocateErrorLogEntry(
    IoObject: PVOID,
    EntrySize: UCHAR,
) -> PVOID {
    let IoObject = opaque_ptr(IoObject);
    if IoObject.is_null() {
        return core::ptr::null_mut();
    }

    let DeviceObject = IoObject as PDEVICE_OBJECT;
    let DriverObject: PDRIVER_OBJECT;
    let ActualDeviceObject: PDEVICE_OBJECT;

    if (*DeviceObject).Type == IO_TYPE_DEVICE as i16 {
        ActualDeviceObject = DeviceObject;
        DriverObject = (*DeviceObject).DriverObject;
    } else if (*DeviceObject).Type == IO_TYPE_DRIVER as i16 {
        ActualDeviceObject = core::ptr::null_mut();
        DriverObject = IoObject as PDRIVER_OBJECT;
    } else {
        return core::ptr::null_mut();
    }

    if (EntrySize as usize) < core::mem::size_of::<IO_ERROR_LOG_PACKET>()
        || EntrySize > ERROR_LOG_MAXIMUM_SIZE
    {
        return core::ptr::null_mut();
    }

    let RoundedEntrySize = ((EntrySize as usize + core::mem::size_of::<PVOID>() - 1) & !(core::mem::size_of::<PVOID>() - 1)) as UCHAR;
    let LogEntrySize = core::mem::size_of::<ERROR_LOG_ENTRY>() + RoundedEntrySize as usize;

    if IopTotalLogSize + LogEntrySize as LONG > IOP_MAXIMUM_LOG_SIZE {
        return core::ptr::null_mut();
    }

    let LogEntry = ExAllocatePoolWithTag(NonPagedPool, LogEntrySize, TAG_ERROR_LOG) as PERROR_LOG_ENTRY;
    if LogEntry.is_null() {
        return core::ptr::null_mut();
    }

    if !ActualDeviceObject.is_null() {
        ObReferenceObject(ActualDeviceObject as PVOID);
    }
    if !DriverObject.is_null() {
        ObReferenceObject(DriverObject as PVOID);
    }

    InterlockedExchangeAdd(core::ptr::addr_of_mut!(IopTotalLogSize), LogEntrySize as LONG);

    core::ptr::write_bytes(LogEntry as *mut u8, 0, LogEntrySize);
    (*LogEntry).Type = IO_TYPE_ERROR_LOG as i16;
    (*LogEntry).Size = LogEntrySize as USHORT;
    (*LogEntry).DeviceObject = ActualDeviceObject;
    (*LogEntry).DriverObject = DriverObject;

    (LogEntry as usize + core::mem::size_of::<ERROR_LOG_ENTRY>()) as PVOID
}

#[no_mangle]
pub unsafe extern "system" fn IoFreeErrorLogEntry(ElEntry: PVOID) {
    let ElEntry = opaque_ptr(ElEntry);
    if ElEntry.is_null() {
        return;
    }

    let LogEntry = ((ElEntry as usize) - core::mem::size_of::<ERROR_LOG_ENTRY>()) as PERROR_LOG_ENTRY;

    if !(*LogEntry).DeviceObject.is_null() {
        ObDereferenceObject((*LogEntry).DeviceObject as PVOID);
    }
    if !(*LogEntry).DriverObject.is_null() {
        ObDereferenceObject((*LogEntry).DriverObject as PVOID);
    }

    InterlockedExchangeAdd(core::ptr::addr_of_mut!(IopTotalLogSize), -((*LogEntry).Size as LONG));
    ExFreePoolWithTag(LogEntry as PVOID, TAG_ERROR_LOG);
}

#[no_mangle]
pub unsafe extern "system" fn IoWriteErrorLogEntry(ElEntry: PVOID) {
    let ElEntry = opaque_ptr(ElEntry);
    if ElEntry.is_null() {
        return;
    }

    let LogEntry = ((ElEntry as usize) - core::mem::size_of::<ERROR_LOG_ENTRY>()) as PERROR_LOG_ENTRY;

    KeQuerySystemTime(core::ptr::addr_of_mut!((*LogEntry).TimeStamp));

    let mut Irql: KIRQL = 0;
    // [guard] KeAcquireSpinLock(core::ptr::addr_of_mut!(IopLogListLock), &mut Irql);
    let spin_guard = unsafe { ntkrnl_sys::guards::SpinLockGuard::acquire(core::ptr::addr_of_mut!(IopLogListLock)) };
    InsertHeadList(
        core::ptr::addr_of_mut!(IopErrorLogListHead),
        core::ptr::addr_of_mut!((*LogEntry).ListEntry),
    );

    if IopLogWorkerRunning == 0 {
        IopLogWorkerRunning = 1u8;
        ExInitializeWorkItem(
            core::ptr::addr_of_mut!(IopErrorLogWorkItem),
            IopLogWorker as PVOID,
            core::ptr::null_mut(),
        );
        ExQueueWorkItem(
            core::ptr::addr_of_mut!(IopErrorLogWorkItem),
            DelayedWorkQueue,
        );
    }

    drop(spin_guard);
}

#[no_mangle]
pub unsafe extern "system" fn IoRaiseHardError(
    Irp: PIRP,
    Vpb: PVPB,
    RealDeviceObject: PDEVICE_OBJECT,
) {
    let Irp = opaque_ptr(Irp);
    let Vpb = opaque_ptr(Vpb);
    let RealDeviceObject = opaque_ptr(RealDeviceObject);
    let Thread = (*Irp).Tail.Overlay.Thread as PETHREAD;

    if (*(Thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.HardErrorsAreDisabled() != 0 {
        (*Irp).IoStatus.Information = 0;
        IoCompleteRequest(Irp, IO_DISK_INCREMENT);
        return;
    }

    // The C's TODO, done: "In case we were called ... for a synchronous operation, that
    // happens with APCs disabled, queue the hard-error call for later processing". A mount
    // is that case. IopMountVolume holds a critical region from before IoCallDriver until
    // after its wait for the mount IRP, and FastFat raises the hard error for a missing
    // medium on that same thread (FatProcessException, user-induced status), then returns
    // STATUS_PENDING. A normal kernel APC queued to a thread in a critical region is not
    // delivered until it leaves, and it leaves only once the IRP completes: the mount waited
    // forever, and with it every opener of the drive (stortest's GetVolumeInformationA on
    // the Pi 5, once the Pico's medium went away, 2026-10-16). The APC's only work is to
    // complete the IRP (IopRaiseHardError raises no pop-up), and the caller marked the IRP
    // pending so it may complete now, as the hard-errors-disabled path above does.
    if Thread == PsGetCurrentThread() && KeAreApcsDisabled() != 0 {
        ntkrnl_sys::driver_ddk::DbgPrint(
            b"[IO] hard error 0x%x completed inline: the requester has APCs disabled\n\0"
                .as_ptr() as _,
            (*Irp).IoStatus.__bindgen_anon_1.Status,
        );
        IopRaiseHardErrorInner(Irp as PVOID, Vpb as PVOID, RealDeviceObject as PVOID);
        return;
    }

    let ErrorApc = ExAllocatePoolWithTag(NonPagedPool, core::mem::size_of::<_KAPC>(), TAG_APC) as PKAPC;
    if ErrorApc.is_null() {
        IoCompleteRequest(Irp, IO_DISK_INCREMENT);
        return;
    }

    KeInitializeApc(
        ErrorApc,
        core::ptr::addr_of_mut!((*(Thread as *mut _ETHREAD)).Tcb),
        (*Irp).ApcEnvironment as u32,
        Some(IopFreeApc),
        None,
        Some(IopRaiseHardError),
        0i8,
        Irp as PVOID,
    );

    KeInsertQueueApc(ErrorApc, Vpb as PVOID, RealDeviceObject as PVOID, IO_NO_INCREMENT as i32);
}

#[no_mangle]
pub unsafe extern "system" fn IoRaiseInformationalHardError(
    _ErrorStatus: NTSTATUS,
    _String: PUNICODE_STRING,
    _Thread: *mut core::ffi::c_void,
) -> BOOLEAN {
    let _String = opaque_ptr(_String);
    let _Thread = opaque_ptr(_Thread);
    0u8
}

#[no_mangle]
pub unsafe extern "system" fn IoSetThreadHardErrorMode(HardErrorEnabled: BOOLEAN) -> BOOLEAN {
    let Thread = PsGetCurrentThread() as *mut _ETHREAD;

    let OldMode = if (*Thread).__bindgen_anon_7.__bindgen_anon_1.HardErrorsAreDisabled() != 0 { 0u8 } else { 1u8 };

    (*Thread).__bindgen_anon_7.__bindgen_anon_1.set_HardErrorsAreDisabled(if HardErrorEnabled != 0 { 0 } else { 1 });
    OldMode
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/io/iomgr/error.c
//  * PURPOSE:         I/O Error Functions and Error Log Support
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Eric Kohl
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #include <subsys/iolog/iolog.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* TYPES *********************************************************************/
//
// typedef struct _IOP_ERROR_LOG_WORKER_DPC
// {
//     KDPC Dpc;
//     KTIMER Timer;
// } IOP_ERROR_LOG_WORKER_DPC, *PIOP_ERROR_LOG_WORKER_DPC;
//
// /* GLOBALS *******************************************************************/
//
// #define IOP_MAXIMUM_LOG_SIZE    (100 * PAGE_SIZE)
// LONG IopTotalLogSize;
// LIST_ENTRY IopErrorLogListHead;
// KSPIN_LOCK IopLogListLock;
//
// BOOLEAN IopLogWorkerRunning;
// BOOLEAN IopLogPortConnected;
// HANDLE IopLogPort;
// WORK_QUEUE_ITEM IopErrorLogWorkItem;
//
// PDEVICE_OBJECT IopErrorLogObject;
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// VOID
// NTAPI
// IopLogDpcRoutine(IN PKDPC Dpc,
//                  IN PVOID DeferredContext,
//                  IN PVOID SystemArgument1,
//                  IN PVOID SystemArgument2)
// {
//     /* If we have a DPC, free it */
//     if (Dpc) ExFreePool(Dpc);
//
//     /* Initialize and queue the work item */
//     ExInitializeWorkItem(&IopErrorLogWorkItem, IopLogWorker, NULL);
//     ExQueueWorkItem(&IopErrorLogWorkItem, DelayedWorkQueue);
// }
//
// PLIST_ENTRY
// NTAPI
// IopGetErrorLogEntry(VOID)
// {
//     KIRQL OldIrql;
//     PLIST_ENTRY ListEntry;
//
//     /* Acquire the lock and check if the list is empty */
//     KeAcquireSpinLock(&IopLogListLock, &OldIrql);
//     if (IsListEmpty(&IopErrorLogListHead))
//     {
//         /* List is empty, disable the worker and return NULL */
//         IopLogWorkerRunning = FALSE;
//         ListEntry = NULL;
//     }
//     else
//     {
//         /* Otherwise, remove an entry */
//         ListEntry = RemoveHeadList(&IopErrorLogListHead);
//     }
//
//     /* Release the lock and return the entry */
//     KeReleaseSpinLock(&IopLogListLock, OldIrql);
//     return ListEntry;
// }
//
// VOID
// NTAPI
// IopRestartLogWorker(VOID)
// {
//     PIOP_ERROR_LOG_WORKER_DPC WorkerDpc;
//     LARGE_INTEGER Timeout;
//
//     /* Allocate a DPC Context */
//     WorkerDpc = ExAllocatePool(NonPagedPool, sizeof(IOP_ERROR_LOG_WORKER_DPC));
//     if (!WorkerDpc)
//     {
//         /* Fail */
//         IopLogWorkerRunning = FALSE;
//         return;
//     }
//
//     /* Initialize DPC and Timer */
//     KeInitializeDpc(&WorkerDpc->Dpc, IopLogDpcRoutine, WorkerDpc);
//     KeInitializeTimer(&WorkerDpc->Timer);
//
//     /* Restart after 30 seconds */
//     Timeout.QuadPart = (LONGLONG)-300000000;
//     KeSetTimer(&WorkerDpc->Timer, Timeout, &WorkerDpc->Dpc);
// }
//
// BOOLEAN
// NTAPI
// IopConnectLogPort(VOID)
// {
//     NTSTATUS Status;
//     UNICODE_STRING PortName = RTL_CONSTANT_STRING(ELF_PORT_NAME);
//     SECURITY_QUALITY_OF_SERVICE SecurityQos;
//
//     /* Make sure we're not already connected */
//     if (IopLogPortConnected) return TRUE;
//
//     /* Setup the QoS structure */
//     SecurityQos.Length = sizeof(SecurityQos);
//     SecurityQos.ImpersonationLevel = SecurityIdentification;
//     SecurityQos.ContextTrackingMode = SECURITY_DYNAMIC_TRACKING;
//     SecurityQos.EffectiveOnly = TRUE;
//
//     /* Connect the port */
//     Status = ZwConnectPort(&IopLogPort,
//                            &PortName,
//                            &SecurityQos,
//                            NULL,
//                            NULL,
//                            NULL,
//                            NULL,
//                            NULL);
//     if (NT_SUCCESS(Status))
//     {
//         /* Remember we're connected */
//         IopLogPortConnected = TRUE;
//         return TRUE;
//     }
//
//     /* We failed, try again */
//     IopRestartLogWorker();
//     return FALSE;
// }
//
// VOID
// NTAPI
// IopLogWorker(IN PVOID Parameter)
// {
// #define IO_ERROR_OBJECT_NAMES_LENGTH    100
//
//     NTSTATUS Status;
//     PELF_API_MSG Message;
//     PIO_ERROR_LOG_MESSAGE ErrorMessage;
//     PLIST_ENTRY ListEntry;
//     PERROR_LOG_ENTRY LogEntry;
//     PIO_ERROR_LOG_PACKET Packet;
//     PCHAR StringBuffer;
//     ULONG RemainingLength;
//     PDRIVER_OBJECT DriverObject;
//     PWCHAR NameString;
//     ULONG DriverNameLength, DeviceNameLength;
//     UCHAR Buffer[sizeof(OBJECT_NAME_INFORMATION) + IO_ERROR_OBJECT_NAMES_LENGTH];
//     POBJECT_NAME_INFORMATION ObjectNameInfo = (POBJECT_NAME_INFORMATION)&Buffer;
//     POBJECT_NAME_INFORMATION PoolObjectNameInfo;
//     ULONG ReturnedLength, MessageLength;
//     ULONG ExtraStringLength;
//     PWCHAR p;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER(Parameter);
//
//     /* Connect to the port */
//     if (!IopConnectLogPort()) return;
//
//     /* Allocate the message */
//     Message = ExAllocatePoolWithTag(PagedPool, IO_ERROR_LOG_MESSAGE_LENGTH, TAG_IO);
//     if (!Message)
//     {
//         /* Couldn't allocate, try again */
//         IopRestartLogWorker();
//         return;
//     }
//
//     /* Zero out the message and get the actual I/O structure */
//     RtlZeroMemory(Message, sizeof(*Message));
//     ErrorMessage = &Message->IoErrorMessage;
//
//     /* Start loop */
//     while (TRUE)
//     {
//         /* Get an entry */
//         ListEntry = IopGetErrorLogEntry();
//         if (!ListEntry) break;
//         LogEntry = CONTAINING_RECORD(ListEntry, ERROR_LOG_ENTRY, ListEntry);
//
//         /* Get pointer to the log packet */
//         Packet = (PIO_ERROR_LOG_PACKET)((ULONG_PTR)LogEntry +
//                                         sizeof(ERROR_LOG_ENTRY));
//
//         /* Calculate the total length of the message only */
//         MessageLength = sizeof(IO_ERROR_LOG_MESSAGE) -
//                         sizeof(ERROR_LOG_ENTRY) -
//                         sizeof(IO_ERROR_LOG_PACKET) +
//                         LogEntry->Size;
//
//         /* Copy the packet */
//         RtlCopyMemory(&ErrorMessage->EntryData,
//                       Packet,
//                       LogEntry->Size - sizeof(ERROR_LOG_ENTRY));
//
//         /* Set the timestamp and time */
//         ErrorMessage->TimeStamp = LogEntry->TimeStamp;
//         ErrorMessage->Type = IO_TYPE_ERROR_MESSAGE;
//
//         /* Check if this message has any strings */
//         if (Packet->NumberOfStrings)
//         {
//             /* String buffer is after the current strings */
//             StringBuffer = (PCHAR)&ErrorMessage->EntryData +
//                             Packet->StringOffset;
//         }
//         else
//         {
//             /* Otherwise, string buffer is at the end */
//             StringBuffer = (PCHAR)ErrorMessage + MessageLength;
//         }
//
//         /* Align the buffer */
//         StringBuffer = ALIGN_UP_POINTER(StringBuffer, WCHAR);
//
//         /* Set the offset for the driver's name to the current buffer */
//         ErrorMessage->DriverNameOffset = (ULONG)(StringBuffer -
//                                                 (PCHAR)ErrorMessage);
//
//         /* Check how much space we have left for the device string */
//         RemainingLength = (ULONG)((ULONG_PTR)Message +
//                                   IO_ERROR_LOG_MESSAGE_LENGTH -
//                                   (ULONG_PTR)StringBuffer);
//
//         NameString = NULL;
//         DriverNameLength = 0; DeviceNameLength = 0;
//         PoolObjectNameInfo = NULL;
//         ObjectNameInfo = (POBJECT_NAME_INFORMATION)&Buffer;
//
//         /* Now check if there is a driver object */
//         DriverObject = LogEntry->DriverObject;
//         if (DriverObject)
//         {
//             /* Check if the driver has a name, and use it if so */
//             if (DriverObject->DriverName.Buffer)
//             {
//                 NameString = DriverObject->DriverName.Buffer;
//                 DriverNameLength = DriverObject->DriverName.Length;
//             }
//             else
//             {
//                 NameString = NULL;
//                 DriverNameLength = 0;
//             }
//
//             /* Check if there isn't a valid name */
//             if (!DriverNameLength)
//             {
//                 /* Query the name directly */
//                 Status = ObQueryNameString(DriverObject,
//                                            ObjectNameInfo,
//                                            sizeof(Buffer),
//                                            &ReturnedLength);
//                 if (!NT_SUCCESS(Status) || (ObjectNameInfo->Name.Length == 0))
//                 {
//                     /* We don't have a name */
//                     DriverNameLength = 0;
//                 }
//                 else
//                 {
//                     NameString = ObjectNameInfo->Name.Buffer;
//                     DriverNameLength = ObjectNameInfo->Name.Length;
//                 }
//             }
//         }
//         else
//         {
//             /* Use default name */
//             NameString = L"Application Popup";
//             DriverNameLength = (ULONG)wcslen(NameString) * sizeof(WCHAR);
//         }
//
//         /* Check if we have a driver name */
//         if (DriverNameLength)
//         {
//             /* Skip to the end of the driver's name */
//             p = &NameString[DriverNameLength / sizeof(WCHAR)];
//
//             /* Now we'll walk backwards and assume the minimum size */
//             DriverNameLength = sizeof(WCHAR);
//             p--;
//             while ((*p != L'\\') && (p != NameString))
//             {
//                 /* No backslash found, keep going */
//                 p--;
//                 DriverNameLength += sizeof(WCHAR);
//             }
//
//             /* Now we probably hit the backslash itself, skip past it */
//             if (*p == L'\\')
//             {
//                 p++;
//                 DriverNameLength -= sizeof(WCHAR);
//             }
//
//             /*
//              * Now make sure that the driver name fits in the buffer, minus
//              * 3 NULL chars (driver name, device name, and remaining strings),
//              * and copy the driver name in the string buffer.
//              */
//             DriverNameLength = min(DriverNameLength,
//                                    RemainingLength - 3 * sizeof(UNICODE_NULL));
//             RtlCopyMemory(StringBuffer, p, DriverNameLength);
//         }
//
//         /* Null-terminate the driver name */
//         *((PWSTR)(StringBuffer + DriverNameLength)) = UNICODE_NULL;
//         DriverNameLength += sizeof(WCHAR);
//
//         /* Go to the next string buffer position */
//         StringBuffer += DriverNameLength;
//         RemainingLength -= DriverNameLength;
//
//         /* Update the string offset */
//         ErrorMessage->EntryData.StringOffset =
//             (USHORT)((ULONG_PTR)StringBuffer - (ULONG_PTR)ErrorMessage);
//
//         /* Check if we have a device object */
//         if (LogEntry->DeviceObject)
//         {
//             /* We do, query its name */
//             Status = ObQueryNameString(LogEntry->DeviceObject,
//                                        ObjectNameInfo,
//                                        sizeof(Buffer) - DriverNameLength,
//                                        &ReturnedLength);
//             if (!NT_SUCCESS(Status) || (ObjectNameInfo->Name.Length == 0))
//             {
//                 /* Setup an empty name */
//                 RtlInitEmptyUnicodeString(&ObjectNameInfo->Name, L"", 0);
//
//                 /* Check if we failed because our buffer wasn't large enough */
//                 if (Status == STATUS_INFO_LENGTH_MISMATCH)
//                 {
//                     /* Then we'll allocate one... we really want this name! */
//                     PoolObjectNameInfo = ExAllocatePoolWithTag(PagedPool,
//                                                                ReturnedLength,
//                                                                TAG_IO);
//                     if (PoolObjectNameInfo)
//                     {
//                         /* Query it again */
//                         ObjectNameInfo = PoolObjectNameInfo;
//                         Status = ObQueryNameString(LogEntry->DeviceObject,
//                                                    ObjectNameInfo,
//                                                    ReturnedLength,
//                                                    &ReturnedLength);
//                         if (NT_SUCCESS(Status))
//                         {
//                             /* Success, update the information */
//                             ObjectNameInfo->Name.Length =
//                                 IO_ERROR_OBJECT_NAMES_LENGTH - (USHORT)DriverNameLength;
//                         }
//                     }
//                 }
//             }
//
//             NameString = ObjectNameInfo->Name.Buffer;
//             DeviceNameLength = ObjectNameInfo->Name.Length;
//         }
//         else
//         {
//             /* No device object, setup an empty name */
//             NameString = L"";
//             DeviceNameLength = 0;
//         }
//
//         /*
//          * Now make sure that the device name fits in the buffer, minus
//          * 2 NULL chars (device name, and remaining strings), and copy
//          * the device name in the string buffer.
//          */
//         DeviceNameLength = min(DeviceNameLength,
//                                RemainingLength - 2 * sizeof(UNICODE_NULL));
//         RtlCopyMemory(StringBuffer, NameString, DeviceNameLength);
//
//         /* Null-terminate the device name */
//         *((PWSTR)(StringBuffer + DeviceNameLength)) = UNICODE_NULL;
//         DeviceNameLength += sizeof(WCHAR);
//
//         /* Free the buffer if we had one */
//         if (PoolObjectNameInfo)
//         {
//             ExFreePoolWithTag(PoolObjectNameInfo, TAG_IO);
//             PoolObjectNameInfo = NULL;
//         }
//
//         /* Go to the next string buffer position */
//         ErrorMessage->EntryData.NumberOfStrings++;
//         StringBuffer += DeviceNameLength;
//         RemainingLength -= DeviceNameLength;
//
//         /* Check if we have any extra strings */
//         if (Packet->NumberOfStrings)
//         {
//             /* Find out the size of the extra strings */
//             ExtraStringLength = LogEntry->Size -
//                                 sizeof(ERROR_LOG_ENTRY) -
//                                 Packet->StringOffset;
//
//             /* Round up the length */
//             ExtraStringLength = ROUND_UP(ExtraStringLength, sizeof(WCHAR));
//
//             /* Make sure that the extra strings fit in our buffer */
//             if (ExtraStringLength > (RemainingLength - sizeof(UNICODE_NULL)))
//             {
//                 /* They wouldn't, so set normalize the length */
//                 MessageLength -= ExtraStringLength - RemainingLength;
//                 ExtraStringLength = RemainingLength - sizeof(UNICODE_NULL);
//             }
//
//             /* Now copy the extra strings */
//             RtlCopyMemory(StringBuffer,
//                           (PCHAR)Packet + Packet->StringOffset,
//                           ExtraStringLength);
//
//             /* Null-terminate them */
//             *((PWSTR)(StringBuffer + ExtraStringLength)) = UNICODE_NULL;
//         }
//
//         /* Set the driver name length */
//         ErrorMessage->DriverNameLength = (USHORT)DriverNameLength;
//
//         /* Update the message length to include the driver and device names */
//         MessageLength += DriverNameLength + DeviceNameLength;
//         ErrorMessage->Size = (USHORT)MessageLength;
//
//         /* Now update it again for the size of the actual LPC */
//         MessageLength += (FIELD_OFFSET(ELF_API_MSG, IoErrorMessage) -
//                           FIELD_OFFSET(ELF_API_MSG, Unknown[0]));
//
//         /* Set the total and data lengths */
//         Message->Header.u1.s1.TotalLength =
//             (USHORT)(sizeof(PORT_MESSAGE) + MessageLength);
//         Message->Header.u1.s1.DataLength = (USHORT)MessageLength;
//
//         /* Send the message */
//         Status = ZwRequestPort(IopLogPort, &Message->Header);
//         if (!NT_SUCCESS(Status))
//         {
//             /*
//              * An error happened while sending the message on the port.
//              * Close the port, requeue the log message on top of the list
//              * and restart the worker.
//              */
//             ZwClose(IopLogPort);
//             IopLogPortConnected = FALSE;
//
//             ExInterlockedInsertHeadList(&IopErrorLogListHead,
//                                         &LogEntry->ListEntry,
//                                         &IopLogListLock);
//
//             IopRestartLogWorker();
//             break;
//         }
//
//         /* NOTE: The following is basically 'IoFreeErrorLogEntry(Packet)' */
//
//         /* Dereference both objects */
//         if (LogEntry->DeviceObject) ObDereferenceObject(LogEntry->DeviceObject);
//         if (LogEntry->DriverObject) ObDereferenceObject(LogEntry->DriverObject);
//
//         /* Decrease the total allocation size and free the entry */
//         InterlockedExchangeAdd(&IopTotalLogSize, -(LONG)LogEntry->Size);
//         ExFreePoolWithTag(LogEntry, TAG_ERROR_LOG);
//     }
//
//     /* Free the LPC Message */
//     ExFreePoolWithTag(Message, TAG_IO);
// }
//
// static
// VOID
// NTAPI
// IopFreeApc(IN PKAPC Apc,
//            IN OUT PKNORMAL_ROUTINE* NormalRoutine,
//            IN OUT PVOID* NormalContext,
//            IN OUT PVOID* SystemArgument1,
//            IN OUT PVOID* SystemArgument2)
// {
//     PAGED_CODE();
//
//     /* Free the APC */
//     ExFreePoolWithTag(Apc, TAG_APC);
// }
//
// static
// VOID
// NTAPI
// IopRaiseHardError(IN PVOID NormalContext,
//                   IN PVOID SystemArgument1,
//                   IN PVOID SystemArgument2)
// {
//     PIRP Irp = NormalContext;
//     //PVPB Vpb = SystemArgument1;
//     //PDEVICE_OBJECT DeviceObject = SystemArgument2;
//
//     UNIMPLEMENTED;
//
//     /* FIXME: UNIMPLEMENTED */
//     Irp->IoStatus.Status = STATUS_NOT_IMPLEMENTED;
//     Irp->IoStatus.Information = 0;
//     IoCompleteRequest(Irp, IO_DISK_INCREMENT);
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  */
// PVOID
// NTAPI
// IoAllocateErrorLogEntry(IN PVOID IoObject,
//                         IN UCHAR EntrySize)
// {
//     PERROR_LOG_ENTRY LogEntry;
//     ULONG LogEntrySize;
//     PDEVICE_OBJECT DeviceObject;
//     PDRIVER_OBJECT DriverObject;
//
//     /* Make sure we have an object */
//     if (!IoObject) return NULL;
//
//     /* Check if this is a device object or driver object */
//     DeviceObject = (PDEVICE_OBJECT)IoObject;
//     if (DeviceObject->Type == IO_TYPE_DEVICE)
//     {
//         /* It's a device, get the driver */
//         // DeviceObject = (PDEVICE_OBJECT)IoObject;
//         DriverObject = DeviceObject->DriverObject;
//     }
//     else if (DeviceObject->Type == IO_TYPE_DRIVER)
//     {
//         /* It's a driver, so we don't have a device */
//         DeviceObject = NULL;
//         DriverObject = (PDRIVER_OBJECT)IoObject;
//     }
//     else
//     {
//         /* Fail */
//         return NULL;
//     }
//
//     /* Check whether the size is too small or too large */
//     if ((EntrySize < sizeof(IO_ERROR_LOG_PACKET)) ||
//         (EntrySize > ERROR_LOG_MAXIMUM_SIZE))
//     {
//         /* Fail */
//         return NULL;
//     }
//
//     /* Round up the size and calculate the total size */
//     EntrySize = ROUND_UP(EntrySize, sizeof(PVOID));
//     LogEntrySize = sizeof(ERROR_LOG_ENTRY) + EntrySize;
//
//     /* Check if we're past our buffer */
//     // TODO: Improve (what happens in case of concurrent calls?)
//     if (IopTotalLogSize + LogEntrySize > IOP_MAXIMUM_LOG_SIZE) return NULL;
//
//     /* Allocate the entry */
//     LogEntry = ExAllocatePoolWithTag(NonPagedPool,
//                                      LogEntrySize,
//                                      TAG_ERROR_LOG);
//     if (!LogEntry) return NULL;
//
//     /* Reference the Objects */
//     if (DeviceObject) ObReferenceObject(DeviceObject);
//     if (DriverObject) ObReferenceObject(DriverObject);
//
//     /* Update log size */
//     InterlockedExchangeAdd(&IopTotalLogSize, LogEntrySize);
//
//     /* Clear the entry and set it up */
//     RtlZeroMemory(LogEntry, LogEntrySize);
//     LogEntry->Type = IO_TYPE_ERROR_LOG;
//     LogEntry->Size = LogEntrySize;
//     LogEntry->DeviceObject = DeviceObject;
//     LogEntry->DriverObject = DriverObject;
//
//     /* Return the entry data */
//     return (PVOID)((ULONG_PTR)LogEntry + sizeof(ERROR_LOG_ENTRY));
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoFreeErrorLogEntry(IN PVOID ElEntry)
// {
//     PERROR_LOG_ENTRY LogEntry;
//
//     /* Make sure there is an entry */
//     if (!ElEntry) return;
//
//     /* Get the actual header */
//     LogEntry = (PERROR_LOG_ENTRY)((ULONG_PTR)ElEntry - sizeof(ERROR_LOG_ENTRY));
//
//     /* Dereference both objects */
//     if (LogEntry->DeviceObject) ObDereferenceObject(LogEntry->DeviceObject);
//     if (LogEntry->DriverObject) ObDereferenceObject(LogEntry->DriverObject);
//
//     /* Decrease the total allocation size and free the entry */
//     InterlockedExchangeAdd(&IopTotalLogSize, -(LONG)LogEntry->Size);
//     ExFreePoolWithTag(LogEntry, TAG_ERROR_LOG);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoWriteErrorLogEntry(IN PVOID ElEntry)
// {
//     PERROR_LOG_ENTRY LogEntry;
//     KIRQL Irql;
//
//     /* Make sure there is an entry */
//     if (!ElEntry) return;
//
//     /* Get the actual header */
//     LogEntry = (PERROR_LOG_ENTRY)((ULONG_PTR)ElEntry - sizeof(ERROR_LOG_ENTRY));
//
//     /* Get time stamp */
//     KeQuerySystemTime(&LogEntry->TimeStamp);
//
//     /* Acquire the lock and insert this write in the list */
//     KeAcquireSpinLock(&IopLogListLock, &Irql);
//     InsertHeadList(&IopErrorLogListHead, &LogEntry->ListEntry);
//
//     /* Check if the worker is running */
//     if (!IopLogWorkerRunning)
//     {
//         /* It's not, initialize it and queue it */
//         IopLogWorkerRunning = TRUE;
//         ExInitializeWorkItem(&IopErrorLogWorkItem, IopLogWorker, NULL);
//         ExQueueWorkItem(&IopErrorLogWorkItem, DelayedWorkQueue);
//     }
//
//     /* Release the lock and return */
//     KeReleaseSpinLock(&IopLogListLock, Irql);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoRaiseHardError(IN PIRP Irp,
//                  IN PVPB Vpb,
//                  IN PDEVICE_OBJECT RealDeviceObject)
// {
//     PETHREAD Thread = Irp->Tail.Overlay.Thread;
//     PKAPC ErrorApc;
//
//     /* Don't do anything if hard errors are disabled on the thread */
//     if (Thread->HardErrorsAreDisabled)
//     {
//         /* Complete the request */
//         Irp->IoStatus.Information = 0;
//         IoCompleteRequest(Irp, IO_DISK_INCREMENT);
//         return;
//     }
//
//     // TODO: In case we were called in the context of a paging I/O or for
//     // a synchronous operation, that happens with APCs disabled, queue the
//     // hard-error call for later processing (see also IofCompleteRequest).
//
//     /* Setup an APC and queue it to deal with the error (see OSR documentation) */
//     ErrorApc = ExAllocatePoolWithTag(NonPagedPool, sizeof(*ErrorApc), TAG_APC);
//     if (!ErrorApc)
//     {
//         /* Fail */
//         IoCompleteRequest(Irp, IO_DISK_INCREMENT);
//         return;
//     }
//
//     KeInitializeApc(ErrorApc,
//                     &Thread->Tcb,
//                     Irp->ApcEnvironment,
//                     IopFreeApc,
//                     NULL,
//                     IopRaiseHardError,
//                     KernelMode,
//                     Irp);
//
//     KeInsertQueueApc(ErrorApc, Vpb, RealDeviceObject, IO_NO_INCREMENT);
// }
//
// /*
//  * @unimplemented
//  */
// BOOLEAN
// NTAPI
// IoRaiseInformationalHardError(IN NTSTATUS ErrorStatus,
//                               IN PUNICODE_STRING String,
//                               IN PKTHREAD Thread)
// {
//     DPRINT1("IoRaiseInformationalHardError: %lx, '%wZ'\n", ErrorStatus, String);
// #if DBG
//     ASSERT(ErrorStatus != STATUS_FILE_CORRUPT_ERROR); /* CORE-17587 */
// #endif
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// IoSetThreadHardErrorMode(IN BOOLEAN HardErrorEnabled)
// {
//     PETHREAD Thread = PsGetCurrentThread();
//     BOOLEAN OldMode;
//
//     /* Get the current value */
//     OldMode = !Thread->HardErrorsAreDisabled;
//
//     /* Set the new one and return the old */
//     Thread->HardErrorsAreDisabled = !HardErrorEnabled;
//     return OldMode;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
