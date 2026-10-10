use crate::prelude::*;
use ntkrnl_sys::ddk::{_KWAIT_REASON_Executive, IoAllocateIrp, IoGetAttachedDeviceReference, KEVENT, KeInitializeEvent, KeWaitForSingleObject, _EVENT_TYPE_NotificationEvent, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP};
use core::ffi::c_void;

unsafe extern "C" fn AtaAcpiEvaluateObject(
    mut DeviceObject: PDEVICE_OBJECT,
    mut InputBuffer: PVOID,
    mut InputBufferLength: ULONG,
    mut OutputBuffer: PACPI_EVAL_OUTPUT_BUFFER,
    mut OutputBufferLength: ULONG,
) -> NTSTATUS {
    let mut Irp: PIRP = core::ptr::null_mut();
    let mut IoStack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Status: NTSTATUS = 0;
    let mut TopDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();

    TopDeviceObject = IoGetAttachedDeviceReference(DeviceObject);

    Irp = IoAllocateIrp((*TopDeviceObject).StackSize, 0);
    if Irp.is_null() {
        Status = STATUS_INSUFFICIENT_RESOURCES;
    } else {
        (*Irp).AssociatedIrp.SystemBuffer = InputBuffer;
        (*Irp).UserBuffer = OutputBuffer as PVOID;
        (*Irp).Flags |= IRP_BUFFERED_IO | IRP_INPUT_OPERATION;
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;

        IoStack = IoGetNextIrpStackLocation(Irp);
        (*IoStack).MajorFunction = (IRP_MJ_DEVICE_CONTROL as u8);
        (*IoStack).Parameters.DeviceIoControl.IoControlCode = IOCTL_ACPI_ASYNC_EVAL_METHOD;
        (*IoStack).Parameters.DeviceIoControl.InputBufferLength = InputBufferLength;
        (*IoStack).Parameters.DeviceIoControl.OutputBufferLength = OutputBufferLength;

        KeInitializeEvent(&mut Event, _EVENT_TYPE_NotificationEvent, FALSE);
        IoSetCompletionRoutine(
            Irp,
            Some(PciIdeXPdoCompletionRoutine),
            &mut Event as *mut _ as PVOID,
            TRUE,
            TRUE,
            TRUE,
        );

        Status = IoCallDriver(TopDeviceObject, Irp);
        if Status == STATUS_PENDING {
            KeWaitForSingleObject(
                &mut Event as *mut _ as PVOID,
                _KWAIT_REASON_Executive,
                (0i32) as KPROCESSOR_MODE,
                FALSE,
                core::ptr::null_mut(),
            );
            Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        }

        RtlCopyMemory!(
            OutputBuffer as *mut core::ffi::c_void,
            InputBuffer as *const core::ffi::c_void,
            min((*Irp).IoStatus.Information, OutputBufferLength as ULONG_PTR),
        );

        if !OutputBuffer.is_null() && NT_SUCCESS!(Status) {
            if (*OutputBuffer).Signature != ACPI_EVAL_OUTPUT_BUFFER_SIGNATURE
                || (*OutputBuffer).Count == 0
            {
                ERR!(b"Invalid ACPI output buffer\n\0".as_ptr() as *const i8);

                Status = STATUS_ACPI_INVALID_DATA;
            }
        }
    }

    ObDereferenceObject((TopDeviceObject).cast());
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn AtaAcpiGetTimingMode(
    mut DeviceObject: PDEVICE_OBJECT,
    mut TimingMode: PIDE_ACPI_TIMING_MODE_BLOCK,
) -> BOOLEAN {
    let mut Buffer: [UCHAR; (FIELD_OFFSET!(ACPI_EVAL_OUTPUT_BUFFER, Argument) as usize +
                 ACPI_METHOD_ARGUMENT_LENGTH!(core::mem::size_of::<_IDE_ACPI_TIMING_MODE_BLOCK>())) as usize] = [0; (FIELD_OFFSET!(ACPI_EVAL_OUTPUT_BUFFER, Argument) as usize +
                 ACPI_METHOD_ARGUMENT_LENGTH!(core::mem::size_of::<_IDE_ACPI_TIMING_MODE_BLOCK>())) as usize];
    let mut InputBuffer: ACPI_EVAL_INPUT_BUFFER = { let mut __lit: ACPI_EVAL_INPUT_BUFFER = core::mem::zeroed(); __lit.Signature = 0; __lit.__bindgen_anon_1.MethodNameAsUlong = 0; __lit };
    let mut Status: NTSTATUS = 0;
    let mut OutputBuffer: PACPI_EVAL_OUTPUT_BUFFER = Buffer.as_mut_ptr() as PACPI_EVAL_OUTPUT_BUFFER;

    RtlZeroMemory!(
        Buffer.as_mut_ptr() as *mut core::ffi::c_void,
        core::mem::size_of::<[UCHAR; (FIELD_OFFSET!(ACPI_EVAL_OUTPUT_BUFFER, Argument) as usize +
                 ACPI_METHOD_ARGUMENT_LENGTH!(core::mem::size_of::<_IDE_ACPI_TIMING_MODE_BLOCK>())) as usize]>(),
    );

    InputBuffer.__bindgen_anon_1.MethodNameAsUlong = 0x4D54475Fu32; // C 'MTG_': "_GTM" in memory
    InputBuffer.Signature = ACPI_EVAL_INPUT_BUFFER_SIGNATURE;

    Status = AtaAcpiEvaluateObject(
        DeviceObject,
        &mut InputBuffer as *mut ACPI_EVAL_INPUT_BUFFER as PVOID,
        core::mem::size_of::<ACPI_EVAL_INPUT_BUFFER>() as ULONG,
        OutputBuffer,
        core::mem::size_of::<[UCHAR; (FIELD_OFFSET!(ACPI_EVAL_OUTPUT_BUFFER, Argument) as usize +
                 ACPI_METHOD_ARGUMENT_LENGTH!(core::mem::size_of::<_IDE_ACPI_TIMING_MODE_BLOCK>())) as usize]>() as ULONG,
    );

    if !NT_SUCCESS!(Status) {
        return FALSE;
    }

    if ((*OutputBuffer).Argument[0].DataLength as usize) < core::mem::size_of::<_IDE_ACPI_TIMING_MODE_BLOCK>() {
        ERR!(
            b"Buffer too small, size %u/%u\n\0".as_ptr() as *const i8,
            (*OutputBuffer).Argument[0].DataLength as u32,
            core::mem::size_of::<_IDE_ACPI_TIMING_MODE_BLOCK>(),
        );
        return FALSE;
    }

    if (*OutputBuffer).Argument[0].Type != ACPI_METHOD_ARGUMENT_BUFFER as USHORT {
        ERR!(b"Unexpected method type %u\n\0".as_ptr() as *const i8, (*OutputBuffer).Argument[0].Type as u32);
        return FALSE;
    }

    RtlCopyMemory!(
        TimingMode as *mut core::ffi::c_void,
        (*OutputBuffer).Argument[0].__bindgen_anon_1.Data.as_ptr() as *const core::ffi::c_void,
        core::mem::size_of::<_IDE_ACPI_TIMING_MODE_BLOCK>(),
    );

    return TRUE;
}
#[no_mangle]
pub unsafe extern "C" fn AtaAcpiSetTimingMode(
    DeviceObject: PDEVICE_OBJECT,
    TimingMode: PIDE_ACPI_TIMING_MODE_BLOCK,
    IdBlock1: PIDENTIFY_DEVICE_DATA,
    IdBlock2: PIDENTIFY_DEVICE_DATA,
) -> NTSTATUS {
    unsafe {
        let mut InputBuffer: PACPI_EVAL_INPUT_BUFFER_COMPLEX = core::ptr::null_mut();
        let mut Argument: PACPI_METHOD_ARGUMENT = core::ptr::null_mut();
        let mut Status: NTSTATUS = 0;
        let mut InputSize: ULONG = 0;

        InputSize = (FIELD_OFFSET!(ACPI_EVAL_INPUT_BUFFER_COMPLEX, Argument) as usize
            + ACPI_METHOD_ARGUMENT_LENGTH!(core::mem::size_of::<IDE_ACPI_TIMING_MODE_BLOCK>())
            + ACPI_METHOD_ARGUMENT_LENGTH!(core::mem::size_of::<IDENTIFY_DEVICE_DATA>())
            + ACPI_METHOD_ARGUMENT_LENGTH!(core::mem::size_of::<IDENTIFY_DEVICE_DATA>())) as ULONG;

        InputBuffer = ExAllocatePoolUninitialized(
            NonPagedPool,
            (InputSize) as _,
            TAG_PCIIDEX,
        ) as PACPI_EVAL_INPUT_BUFFER_COMPLEX;

        if InputBuffer.is_null() {
            ERR!(b"Failed to allocate memory\n\0".as_ptr() as *const i8);
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        (*InputBuffer).__bindgen_anon_1.MethodNameAsUlong = 1297371999 as ULONG;
        (*InputBuffer).Signature = ACPI_EVAL_INPUT_BUFFER_COMPLEX_SIGNATURE;
        (*InputBuffer).ArgumentCount = 3;
        (*InputBuffer).Size = InputSize;

        Argument = core::ptr::addr_of_mut!((*InputBuffer).Argument) as PACPI_METHOD_ARGUMENT;
        ACPI_METHOD_SET_ARGUMENT_BUFFER!(
            Argument,
            TimingMode,
            core::mem::size_of::<IDE_ACPI_TIMING_MODE_BLOCK>()
        );

        Argument = ACPI_METHOD_NEXT_ARGUMENT!(Argument);
        (*Argument).Type = (ACPI_METHOD_ARGUMENT_BUFFER as u16);
        (*Argument).DataLength = core::mem::size_of::<IDENTIFY_DEVICE_DATA>() as USHORT;
        if !IdBlock1.is_null() {
            RtlCopyMemory!(
                core::ptr::addr_of_mut!((*Argument).__bindgen_anon_1.Data) as *mut core::ffi::c_void,
                IdBlock1 as *const core::ffi::c_void,
                core::mem::size_of::<IDENTIFY_DEVICE_DATA>(),
            );
        } else {
            RtlZeroMemory!(
                core::ptr::addr_of_mut!((*Argument).__bindgen_anon_1.Data) as *mut core::ffi::c_void,
                core::mem::size_of::<IDENTIFY_DEVICE_DATA>(),
            );
        }

        Argument = ACPI_METHOD_NEXT_ARGUMENT!(Argument);
        (*Argument).Type = (ACPI_METHOD_ARGUMENT_BUFFER as u16);
        (*Argument).DataLength = core::mem::size_of::<IDENTIFY_DEVICE_DATA>() as USHORT;
        if !IdBlock2.is_null() {
            RtlCopyMemory!(
                core::ptr::addr_of_mut!((*Argument).__bindgen_anon_1.Data) as *mut core::ffi::c_void,
                IdBlock2 as *const core::ffi::c_void,
                core::mem::size_of::<IDENTIFY_DEVICE_DATA>(),
            );
        } else {
            RtlZeroMemory!(
                core::ptr::addr_of_mut!((*Argument).__bindgen_anon_1.Data) as *mut core::ffi::c_void,
                core::mem::size_of::<IDENTIFY_DEVICE_DATA>(),
            );
        }

        Status = AtaAcpiEvaluateObject(
            DeviceObject,
            InputBuffer as PVOID,
            InputSize,
            core::ptr::null_mut(),
            0,
        );

        if !NT_SUCCESS!(Status) {
            if Status == STATUS_OBJECT_NAME_NOT_FOUND {
                INFO!(
                    b"Failed to set transfer timings, status 0x%lx\n\0".as_ptr() as *const i8,
                    Status,
                );
            } else {
                ERR!(
                    b"Failed to set transfer timings, status 0x%lx\n\0".as_ptr() as *const i8,
                    Status,
                );
            }
        }

        ExFreePoolWithTag(InputBuffer as PVOID, TAG_PCIIDEX);
        Status
    }
}
#[no_mangle]
pub unsafe extern "C" fn AtaAcpiGetTaskFile(
    mut DeviceObject: PDEVICE_OBJECT,
) -> PVOID {
    let mut InputBuffer: ACPI_EVAL_INPUT_BUFFER = core::mem::zeroed();
    let mut OutputBuffer: PACPI_EVAL_OUTPUT_BUFFER = core::ptr::null_mut();
    let mut RetryCount: ULONG = 0;
    let mut OutputSize: ULONG = 0;
    let mut Status: NTSTATUS = 0;

    PAGED_CODE!();

    InputBuffer.__bindgen_anon_1.MethodNameAsUlong = 1179928415u32;
    InputBuffer.Signature = ACPI_EVAL_INPUT_BUFFER_SIGNATURE;

    OutputSize = (core::mem::offset_of!(ACPI_EVAL_OUTPUT_BUFFER, Argument) as ULONG)
        .wrapping_add(
            ACPI_METHOD_ARGUMENT_LENGTH!(
                core::mem::size_of::<ATA_ACPI_TASK_FILE>()
                    .wrapping_mul(10),
            ) as ULONG,
        );

    RetryCount = 0;
    while RetryCount < 2 {
        OutputBuffer = ExAllocatePoolZero(
            NonPagedPool,
            OutputSize as usize,
            TAG_PCIIDEX,
        ) as PACPI_EVAL_OUTPUT_BUFFER;
        if OutputBuffer.is_null() {
            ERR!(b"Failed to allocate memory of size %lu\n\0".as_ptr() as *const i8, OutputSize);
            return core::ptr::null_mut();
        }

        Status = AtaAcpiEvaluateObject(
            DeviceObject,
            core::ptr::addr_of_mut!(InputBuffer) as PVOID,
            core::mem::size_of::<ACPI_EVAL_INPUT_BUFFER>() as ULONG,
            OutputBuffer,
            OutputSize,
        );

        if Status == STATUS_BUFFER_OVERFLOW {
            OutputSize = (*OutputBuffer).Length;
            ExFreePoolWithTag(OutputBuffer as PVOID, TAG_PCIIDEX);
            RetryCount = RetryCount.wrapping_add(1);
            continue;
        }

        break;
    }

    'cleanup: {
        if !NT_SUCCESS!(Status) {
            break 'cleanup;
        }

        if (*OutputBuffer).Argument[0].Type != (ACPI_METHOD_ARGUMENT_BUFFER as u16) {
            ERR!(b"Unexpected method type %u\n\0".as_ptr() as *const i8, (*OutputBuffer).Argument[0].Type as u32);
            break 'cleanup;
        }

        if ((*OutputBuffer).Argument[0].DataLength as usize)
            % core::mem::size_of::<ATA_ACPI_TASK_FILE>()
            != 0
        {
            ERR!(
                b"Incorrect command stream length %u\n\0".as_ptr() as *const i8,
                (*OutputBuffer).Argument[0].DataLength as u32,
            );
            break 'cleanup;
        }

        if (*OutputBuffer).Argument[0].DataLength == 0 {
            WARN!(b"Empty command list\n\0".as_ptr() as *const i8);
            break 'cleanup;
        }

        return OutputBuffer as PVOID;
    }

    ExFreePoolWithTag(OutputBuffer as PVOID, TAG_PCIIDEX);

    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn AtaAcpiSetDeviceData(
    mut DeviceObject: PDEVICE_OBJECT,
    mut IdBlock: PIDENTIFY_DEVICE_DATA,
) {
    let mut InputBuffer: PACPI_EVAL_INPUT_BUFFER_COMPLEX =
        core::ptr::null_mut::<ACPI_EVAL_INPUT_BUFFER_COMPLEX>();
    let mut Argument: PACPI_METHOD_ARGUMENT = core::ptr::null_mut::<ACPI_METHOD_ARGUMENT>();
    let mut Status: NTSTATUS = 0;
    let mut InputSize: ULONG = 0;

    PAGED_CODE!();

    InputSize = (FIELD_OFFSET!(ACPI_EVAL_INPUT_BUFFER_COMPLEX, Argument) as usize
        + ACPI_METHOD_ARGUMENT_LENGTH!(core::mem::size_of::<IDENTIFY_DEVICE_DATA>()))
        as ULONG;

    InputBuffer = ExAllocatePoolUninitialized(
        NonPagedPool,
        (InputSize) as _,
        TAG_PCIIDEX,
    ) as PACPI_EVAL_INPUT_BUFFER_COMPLEX;
    if InputBuffer.is_null() {
        ERR!(b"Failed to allocate memory\n\0".as_ptr() as *const i8);
        return;
    }

    (*InputBuffer).__bindgen_anon_1.MethodNameAsUlong = 0x4444535Fu32; // C 'DDS_': "_SDD" in memory
    (*InputBuffer).Signature = ACPI_EVAL_INPUT_BUFFER_COMPLEX_SIGNATURE;
    (*InputBuffer).ArgumentCount = 1;
    (*InputBuffer).Size = InputSize;

    Argument = &raw mut (*InputBuffer).Argument as *mut ACPI_METHOD_ARGUMENT;
    ACPI_METHOD_SET_ARGUMENT_BUFFER!(
        Argument,
        IdBlock,
        core::mem::size_of::<IDENTIFY_DEVICE_DATA>(),
    );

    Status = AtaAcpiEvaluateObject(
        DeviceObject,
        InputBuffer as PVOID,
        InputSize,
        core::ptr::null_mut::<ACPI_EVAL_OUTPUT_BUFFER>(),
        0,
    );
    if !NT_SUCCESS!(Status) {
        if Status == STATUS_OBJECT_NAME_NOT_FOUND {
            TRACE!("Failed to set device data, status 0x%lx\n", Status);
        } else {
            WARN!("Failed to set device data, status 0x%lx\n", Status);
        }
    }

    ExFreePoolWithTag(InputBuffer as PVOID, TAG_PCIIDEX);
}
