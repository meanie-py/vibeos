//! Rust translation of `drivers/base/beep/beep.c` — the Beep device driver
//! (\Device\Beep, IOCTL_BEEP_SET → HalMakeBeep, with a KTIMER/DPC to stop the
//! tone after the requested duration). Faithful to the C; IRP completion and
//! StartIo/cancel packet handling preserved exactly.

use crate::beep_h::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ddk::{
    PDEVICE_OBJECT, PDRIVER_OBJECT, PIRP, PUNICODE_STRING, UNICODE_STRING,
    PIO_STACK_LOCATION, _IRP, PKDEVICE_QUEUE_ENTRY, LARGE_INTEGER,
    IoCreateDevice, IoDeleteDevice, IoStartPacket, IoStartNextPacket,
    IoAcquireCancelSpinLock, IoReleaseCancelSpinLock,
    KeInitializeDpc, KeInitializeTimer, KeSetTimer, KeCancelTimer,
    KeRemoveDeviceQueue, KeRemoveEntryDeviceQueue,
    MmLockPagableDataSection, MmUnlockPagableImageSection, MmPageEntireDriver,
    HalMakeBeep,
};
use ntkrnl_sys::inline::tier1::{
    IoGetCurrentIrpStackLocation, IoMarkIrpPending, IoSetCancelRoutine,
};
use ntkrnl_sys::inline::ex::ExInitializeFastMutex;
use ntkrnl_sys::interlocked::{InterlockedIncrement, InterlockedDecrement};
use ntkrnl_sys::ddk::PFAST_MUTEX;

// IRQL primitives differ per-arch. On i386/arm64 KeRaiseIrql(new, *old) +
// KeLowerIrql(new) are real stdcall HAL exports. On amd64 they are FORCEINLINE
// (IRQL == CR8), so the C beep.sys imports no IRQL symbol — we inline the CR8
// read/write directly here to match. (BeepCleanup raises to DISPATCH_LEVEL to
// fence the cancel/StartIo races while draining the device queue.)
#[cfg(not(target_arch = "x86_64"))]
extern "system" {
    fn KeRaiseIrql(NewIrql: KIRQL, OldIrql: *mut KIRQL);
    fn KeLowerIrql(NewIrql: KIRQL);
}
#[inline]
unsafe fn beep_raise_to_dispatch(old: *mut KIRQL) {
    #[cfg(not(target_arch = "x86_64"))]
    {
        KeRaiseIrql(DISPATCH_LEVEL as KIRQL, old);
    }
    #[cfg(target_arch = "x86_64")]
    {
        let cr8: u64;
        core::arch::asm!("mov {}, cr8", out(reg) cr8, options(nomem, nostack, preserves_flags));
        *old = cr8 as KIRQL;
        core::arch::asm!("mov cr8, {}", in(reg) DISPATCH_LEVEL as u64, options(nomem, nostack, preserves_flags));
    }
}
#[inline]
unsafe fn beep_lower_irql(old: KIRQL) {
    #[cfg(not(target_arch = "x86_64"))]
    {
        KeLowerIrql(old);
    }
    #[cfg(target_arch = "x86_64")]
    {
        core::arch::asm!("mov cr8, {}", in(reg) old as u64, options(nomem, nostack, preserves_flags));
    }
}

// ExAcquireFastMutex / ExReleaseFastMutex are exported by ntoskrnl, but with a
// per-arch convention: amd64/arm64 export them directly (stdcall = "system");
// i386 exports the fastcall `Exi*` variants (the public names are FORCEINLINE
// there). ExInitializeFastMutex is FORCEINLINE on every arch (not exported),
// so it stays inlined via ntkrnl_sys::inline::ex.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    #[link_name = "ExiAcquireFastMutex"]
    fn ExAcquireFastMutex(FastMutex: PFAST_MUTEX);
    #[link_name = "ExiReleaseFastMutex"]
    fn ExReleaseFastMutex(FastMutex: PFAST_MUTEX);
}
#[cfg(not(target_arch = "x86"))]
extern "system" {
    fn ExAcquireFastMutex(FastMutex: PFAST_MUTEX);
    fn ExReleaseFastMutex(FastMutex: PFAST_MUTEX);
}

// CONTAINING_RECORD(Packet, IRP, Tail.Overlay.DeviceQueueEntry): the device
// queue entry sits at the very start of IRP.Tail, so the offset is offset_of(Tail).
#[inline]
unsafe fn irp_from_device_queue_entry(entry: PKDEVICE_QUEUE_ENTRY) -> PIRP {
    (entry as *mut u8).sub(core::mem::offset_of!(_IRP, Tail)) as PIRP
}

unsafe extern "system" fn BeepDPC(
    _Dpc: PVOID,
    DeviceObject: PDEVICE_OBJECT,
    _SystemArgument1: PVOID,
    _SystemArgument2: PVOID,
) {
    let DeviceExtension: PDEVICE_EXTENSION = (*DeviceObject).DeviceExtension as PDEVICE_EXTENSION;

    /* Stop the beep */
    HalMakeBeep(0);

    /* Disable the timer */
    InterlockedDecrement(core::ptr::addr_of_mut!((*DeviceExtension).TimerActive));
}

unsafe extern "system" fn BeepCreate(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let DeviceExtension: PDEVICE_EXTENSION = (*DeviceObject).DeviceExtension as PDEVICE_EXTENSION;

    /* Acquire the mutex and increase reference count */
    ExAcquireFastMutex(core::ptr::addr_of_mut!((*DeviceExtension).Mutex));
    (*DeviceExtension).ReferenceCount += 1;
    if (*DeviceExtension).ReferenceCount == 1 {
        /* First reference, lock the data section */
        (*DeviceExtension).SectionHandle = MmLockPagableDataSection(BeepCreate as *const () as PVOID);
    }

    /* Release it */
    ExReleaseFastMutex(core::ptr::addr_of_mut!((*DeviceExtension).Mutex));

    /* Complete the request */
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    STATUS_SUCCESS
}

unsafe extern "system" fn BeepClose(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let DeviceExtension: PDEVICE_EXTENSION = (*DeviceObject).DeviceExtension as PDEVICE_EXTENSION;

    /* Acquire the mutex and decrease reference count */
    ExAcquireFastMutex(core::ptr::addr_of_mut!((*DeviceExtension).Mutex));
    (*DeviceExtension).ReferenceCount -= 1;
    if (*DeviceExtension).ReferenceCount == 0 {
        /* Check for active timer */
        if (*DeviceExtension).TimerActive != 0 {
            /* Cancel it */
            if KeCancelTimer(core::ptr::addr_of_mut!((*DeviceExtension).Timer)) != 0 {
                /* Mark it as cancelled */
                InterlockedDecrement(core::ptr::addr_of_mut!((*DeviceExtension).TimerActive));
            }
        }

        /* Page the driver */
        MmUnlockPagableImageSection((*DeviceExtension).SectionHandle);
    }

    /* Release the lock */
    ExReleaseFastMutex(core::ptr::addr_of_mut!((*DeviceExtension).Mutex));

    /* Complete the request */
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    STATUS_SUCCESS
}

unsafe extern "system" fn BeepCancel(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) {
    /* Check if this is the current request */
    if Irp == (*DeviceObject).CurrentIrp {
        /* Clear it */
        (*DeviceObject).CurrentIrp = core::ptr::null_mut();

        /* Release the cancel lock and start the next packet */
        IoReleaseCancelSpinLock((*Irp).CancelIrql);
        IoStartNextPacket(DeviceObject, 1u8);
    } else {
        /* Otherwise, remove the packet from the queue and release the lock */
        KeRemoveEntryDeviceQueue(
            core::ptr::addr_of_mut!((*DeviceObject).DeviceQueue),
            core::ptr::addr_of_mut!((*Irp).Tail.Overlay.__bindgen_anon_1.DeviceQueueEntry),
        );
        IoReleaseCancelSpinLock((*Irp).CancelIrql);
    }

    /* Complete the request */
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_CANCELLED;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
}

unsafe extern "system" fn BeepCleanup(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let mut OldIrql: KIRQL = 0;
    let mut CancelIrql: KIRQL = 0;

    /* Raise IRQL and acquire the cancel lock */
    beep_raise_to_dispatch(&mut OldIrql);
    IoAcquireCancelSpinLock(&mut CancelIrql);

    /* Get the current IRP */
    let mut CurrentIrp: PIRP = (*DeviceObject).CurrentIrp;
    (*DeviceObject).CurrentIrp = core::ptr::null_mut();
    while !CurrentIrp.is_null() {
        /* Clear its cancel routine */
        IoSetCancelRoutine(CurrentIrp, None);

        /* Cancel the IRP */
        (*CurrentIrp).IoStatus.__bindgen_anon_1.Status = STATUS_CANCELLED;
        (*CurrentIrp).IoStatus.Information = 0;

        /* Release the cancel lock and complete it */
        IoReleaseCancelSpinLock(CancelIrql);
        IoCompleteRequest(CurrentIrp, IO_NO_INCREMENT as i8);

        /* Reacquire the lock and get the next queue packet */
        IoAcquireCancelSpinLock(&mut CancelIrql);
        let Packet: PKDEVICE_QUEUE_ENTRY =
            KeRemoveDeviceQueue(core::ptr::addr_of_mut!((*DeviceObject).DeviceQueue));
        if !Packet.is_null() {
            /* Get the IRP */
            CurrentIrp = irp_from_device_queue_entry(Packet);
        } else {
            /* No more IRPs */
            CurrentIrp = core::ptr::null_mut();
        }
    }

    /* Release lock and go back to low IRQL */
    IoReleaseCancelSpinLock(CancelIrql);
    beep_lower_irql(OldIrql);

    /* Complete the IRP */
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);

    /* Stop and beep and return */
    HalMakeBeep(0);
    STATUS_SUCCESS
}

unsafe extern "system" fn BeepDeviceControl(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    /* Get the stack location and parameters */
    let Stack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let BeepParam: PBEEP_SET_PARAMETERS = (*Irp).AssociatedIrp.SystemBuffer as PBEEP_SET_PARAMETERS;
    let Status: NTSTATUS;

    /* We only support one IOCTL */
    if (*Stack).Parameters.DeviceIoControl.IoControlCode != IOCTL_BEEP_SET {
        /* Unsupported command */
        Status = STATUS_NOT_IMPLEMENTED;
    } else if ((*Stack).Parameters.DeviceIoControl.InputBufferLength as usize)
        < core::mem::size_of::<BEEP_SET_PARAMETERS>()
    {
        /* Invalid buffer */
        Status = STATUS_INVALID_PARAMETER;
    } else if (*BeepParam).Frequency != 0 && (*BeepParam).Duration == 0 {
        /* No duration, return immediately */
        Status = STATUS_SUCCESS;
    } else {
        /* We'll queue this request */
        Status = STATUS_PENDING;
    }

    /* Set packet information */
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    (*Irp).IoStatus.Information = 0;

    /* Check if we're completing or queuing a packet */
    if Status == STATUS_PENDING {
        /* Start the queue */
        IoMarkIrpPending(Irp);
        IoStartPacket(
            DeviceObject,
            Irp,
            core::ptr::null_mut(),
            Some(core::mem::transmute(BeepCancel as *const ())),
        );
    } else {
        /* Complete the request */
        IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    }

    /* Return */
    Status
}

unsafe extern "system" fn BeepUnload(DriverObject: PDRIVER_OBJECT) {
    /* Get DO and DE */
    let DeviceObject: PDEVICE_OBJECT = (*DriverObject).DeviceObject;
    let DeviceExtension: PDEVICE_EXTENSION = (*DeviceObject).DeviceExtension as PDEVICE_EXTENSION;

    /* Check if the timer is active */
    if (*DeviceExtension).TimerActive != 0 {
        /* Cancel it */
        if KeCancelTimer(core::ptr::addr_of_mut!((*DeviceExtension).Timer)) != 0 {
            /* All done */
            InterlockedDecrement(core::ptr::addr_of_mut!((*DeviceExtension).TimerActive));
        }
    }

    /* Delete the object */
    IoDeleteDevice(DeviceObject);
}

unsafe extern "system" fn BeepStartIo(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) {
    let DeviceExtension: PDEVICE_EXTENSION = (*DeviceObject).DeviceExtension as PDEVICE_EXTENSION;
    let mut CancelIrql: KIRQL = 0;
    let Status: NTSTATUS;

    /* Acquire the cancel lock and make sure the IRP is valid */
    IoAcquireCancelSpinLock(&mut CancelIrql);
    if Irp.is_null() {
        /* It's not, release the lock and quit */
        IoReleaseCancelSpinLock(CancelIrql);
        return;
    }

    /* Remove the cancel routine and release the lock */
    IoSetCancelRoutine(Irp, None);
    IoReleaseCancelSpinLock(CancelIrql);

    /* Get the I/O Stack and make sure the request is valid */
    let BeepParam: PBEEP_SET_PARAMETERS = (*Irp).AssociatedIrp.SystemBuffer as PBEEP_SET_PARAMETERS;
    let IoStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    if (*IoStack).Parameters.DeviceIoControl.IoControlCode == IOCTL_BEEP_SET {
        /* Check if we have an active timer */
        if (*DeviceExtension).TimerActive != 0 {
            /* Cancel it */
            if KeCancelTimer(core::ptr::addr_of_mut!((*DeviceExtension).Timer)) != 0 {
                /* Set the state */
                InterlockedDecrement(core::ptr::addr_of_mut!((*DeviceExtension).TimerActive));
            }
        }

        /* Make the beep */
        if HalMakeBeep((*BeepParam).Frequency) != 0 {
            /* Beep successful, queue a DPC to stop it */
            Status = STATUS_SUCCESS;
            let mut DueTime: LARGE_INTEGER = core::mem::zeroed();
            DueTime.QuadPart = (*BeepParam).Duration as i64 * -10000i64;
            InterlockedIncrement(core::ptr::addr_of_mut!((*DeviceExtension).TimerActive));
            KeSetTimer(
                core::ptr::addr_of_mut!((*DeviceExtension).Timer),
                DueTime,
                core::ptr::addr_of_mut!((*DeviceObject).Dpc),
            );
        } else {
            /* Beep has failed */
            Status = STATUS_INVALID_PARAMETER;
        }
    } else {
        /* Invalid request */
        Status = STATUS_INVALID_PARAMETER;
    }

    /* Complete the request and start the next packet */
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    (*Irp).IoStatus.Information = 0;
    IoStartNextPacket(DeviceObject, 1u8);
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    _RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    // L"\Device\Beep"
    static DEVICE_NAME_BUF: [u16; 12] = [
        0x5C, 0x44, 0x65, 0x76, 0x69, 0x63, 0x65, 0x5C, 0x42, 0x65, 0x65, 0x70,
    ];
    let mut DeviceName: UNICODE_STRING = core::mem::zeroed();
    DeviceName.Length = (DEVICE_NAME_BUF.len() * 2) as u16;
    DeviceName.MaximumLength = (DEVICE_NAME_BUF.len() * 2) as u16;
    DeviceName.Buffer = DEVICE_NAME_BUF.as_ptr() as *mut u16;

    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();

    /* Create the device */
    let Status = IoCreateDevice(
        DriverObject,
        core::mem::size_of::<DEVICE_EXTENSION>() as u32,
        &mut DeviceName,
        FILE_DEVICE_BEEP,
        0,
        0u8,
        &mut DeviceObject,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* Make it use buffered I/O */
    (*DeviceObject).Flags |= DO_BUFFERED_IO;

    /* Setup the Driver Object */
    (*DriverObject).MajorFunction[IRP_MJ_CREATE as usize] =
        Some(core::mem::transmute(BeepCreate as *const ()));
    (*DriverObject).MajorFunction[IRP_MJ_CLOSE as usize] =
        Some(core::mem::transmute(BeepClose as *const ()));
    (*DriverObject).MajorFunction[IRP_MJ_CLEANUP as usize] =
        Some(core::mem::transmute(BeepCleanup as *const ()));
    (*DriverObject).MajorFunction[IRP_MJ_DEVICE_CONTROL as usize] =
        Some(core::mem::transmute(BeepDeviceControl as *const ()));
    (*DriverObject).DriverUnload = Some(core::mem::transmute(BeepUnload as *const ()));
    (*DriverObject).DriverStartIo = Some(core::mem::transmute(BeepStartIo as *const ()));

    /* Set up device extension */
    let DeviceExtension: PDEVICE_EXTENSION = (*DeviceObject).DeviceExtension as PDEVICE_EXTENSION;
    (*DeviceExtension).ReferenceCount = 0;
    (*DeviceExtension).TimerActive = 0;
    // IoInitializeDpcRequest(DeviceObject, BeepDPC):
    KeInitializeDpc(
        core::ptr::addr_of_mut!((*DeviceObject).Dpc),
        Some(core::mem::transmute(BeepDPC as *const ())),
        DeviceObject as PVOID,
    );
    KeInitializeTimer(core::ptr::addr_of_mut!((*DeviceExtension).Timer));
    ExInitializeFastMutex(core::ptr::addr_of_mut!((*DeviceExtension).Mutex));

    /* Page the entire driver */
    MmPageEntireDriver(DriverEntry as *const () as PVOID);
    STATUS_SUCCESS
}
