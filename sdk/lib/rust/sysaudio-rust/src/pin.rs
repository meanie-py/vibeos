use crate::types::*;
use core::ffi::c_void;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{
    IoGetRelatedDeviceObject, ObReferenceObjectByHandle, ZwClose, PDEVICE_OBJECT, PFILE_OBJECT,
    PHANDLE, PIO_STACK_LOCATION, PIRP,
};
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::inline::tier1::{
    IoGetCurrentIrpStackLocation, IoGetNextIrpStackLocation, IoSkipCurrentIrpStackLocation,
};
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;

use crate::types::KSPROPERTY_CONNECTION_KSPROPERTY_CONNECTION_DATAFORMAT as KSPROPERTY_CONNECTION_DATAFORMAT;
use crate::types::SYSAUDIO_IOCTL_KS_PROPERTY as IOCTL_KS_PROPERTY;
use ntkrnl_sys::ddk::{RtlAssert, KPROCESSOR_MODE, SIZE_T, ULONG_PTR};
// wdm.h aliases the driver spelling to the import-address table. raw-dylib
// supplies that indirection, so this names the exported POBJECT_TYPE value.
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    static IoFileObjectType: ntkrnl_sys::ddk::POBJECT_TYPE;
}
extern "C" {
    fn DbgPrint(Format: *const i8, ...) -> u32;
}

// debug.h: DPRINT1 remains enabled with DBG=1 even when this TU defines NDEBUG.
macro_rules! dprint1 {
    ($line:expr, $format:literal $(, $argument:expr)*) => {{
        if DbgPrint(
            concat!("(%s:%d) ", $format, "\0").as_ptr().cast(),
            b"drivers/wdm/audio/sysaudio/pin.c\0".as_ptr(),
            $line as i32 $(, $argument)*,
        ) != 0 {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr().cast(),
                b"drivers/wdm/audio/sysaudio/pin.c\0".as_ptr(),
                $line as i32,
            );
        }
    }};
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const KSSTRING_Pin: [::core::ffi::c_ushort; 39] = unsafe {
    ::core::mem::transmute::<
        [u8; 78],
        [::core::ffi::c_ushort; 39],
    >(
        *b"{\x001\x004\x006\0F\x001\0A\08\x000\0-\x004\x007\09\x001\0-\x001\x001\0D\x000\0-\0A\x005\0D\x006\0-\x002\08\0D\0B\x000\x004\0C\x001\x000\x000\x000\x000\0}\0\0\0",
    )
};

// Callback types are inferred from the generated KS dispatch table, preserving NTAPI
// on i386 as well as the unified amd64/arm64 ABIs.
static mut PinTable: KSDISPATCH_TABLE = KSDISPATCH_TABLE {
    DeviceIoControl: Some(Pin_fnDeviceIoControl),
    Read: Some(KsDispatchInvalidDeviceRequest),
    Write: Some(Pin_fnWrite),
    Flush: Some(KsDispatchInvalidDeviceRequest),
    Close: Some(Pin_fnClose),
    QuerySecurity: Some(KsDispatchInvalidDeviceRequest),
    SetSecurity: Some(KsDispatchInvalidDeviceRequest),
    FastDeviceIoControl: Some(KsDispatchFastIoDeviceControlFailure),
    FastRead: Some(KsDispatchFastReadFailure),
    FastWrite: Some(KsDispatchFastReadFailure),
};

#[no_mangle]
pub unsafe extern "system" fn Pin_fnDeviceIoControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let Context: PDISPATCH_CONTEXT;
    let mut Status: NTSTATUS;
    let mut BytesReturned: ULONG = 0;
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let IoStack: PIO_STACK_LOCATION;

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    Context = (*(*IoStack).FileObject).FsContext as PDISPATCH_CONTEXT;
    if Context.is_null() {
        RtlAssert(
            b"Context\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            b"drivers/wdm/audio/sysaudio/pin.c\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            35,
            core::ptr::null_mut::<i8>(),
        );
    }

    Status = ObReferenceObjectByHandle(
        (*Context).Handle,
        GENERIC_WRITE,
        IoFileObjectType,
        (0i32) as i8,
        core::ptr::addr_of_mut!(FileObject) as *mut *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );
    if !(Status >= 0) {
        (*Irp).IoStatus.Information = 0;
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return Status;
    }

    Status = KsSynchronousIoControlDevice(
        FileObject,
        (0i32) as KPROCESSOR_MODE,
        (*IoStack).Parameters.DeviceIoControl.IoControlCode,
        (*IoStack).Parameters.DeviceIoControl.Type3InputBuffer,
        (*IoStack).Parameters.DeviceIoControl.InputBufferLength,
        (*Irp).UserBuffer,
        (*IoStack).Parameters.DeviceIoControl.OutputBufferLength,
        &mut BytesReturned,
    );
    ObDereferenceObject((FileObject).cast());

    (*Irp).IoStatus.Information = BytesReturned as ULONG_PTR;
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);

    return Status;
}
#[no_mangle]
pub unsafe extern "system" fn Pin_fnWrite(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let Context: PDISPATCH_CONTEXT;
    let mut IoStack: PIO_STACK_LOCATION;
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    Context = (*(*IoStack).FileObject).FsContext as PDISPATCH_CONTEXT;

    if Context.is_null() {
        RtlAssert(
            b"Context\0".as_ptr() as *const u8 as *mut core::ffi::c_void,
            b"drivers/wdm/audio/sysaudio/pin.c\0".as_ptr() as *const u8 as *mut core::ffi::c_void,
            87 as u32,
            core::ptr::null_mut::<i8>(),
        );
    }

    if !(*Context).hMixerPin.is_null() {
        DbgPrint(
            b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
            b"Pin_fnWrite\0".as_ptr() as *const i8,
            b"drivers/wdm/audio/sysaudio/pin.c\0".as_ptr() as *const i8,
            93 as i32,
        );
        // UNIMPLEMENTED expands to the warning above (debug.h).
    }

    Status = ObReferenceObjectByHandle(
        (*Context).Handle,
        GENERIC_WRITE,
        IoFileObjectType,
        (0i32) as i8,
        &raw mut FileObject as *mut *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );

    if !NT_SUCCESS(Status) {
        dprint1!(100, "failed\n");
        (*Irp).IoStatus.Information = 0;
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return Status;
    }

    IoSkipCurrentIrpStackLocation(Irp);
    IoStack = IoGetNextIrpStackLocation(Irp);
    (*IoStack).FileObject = FileObject;
    (*IoStack).MajorFunction = IRP_MJ_DEVICE_CONTROL;

    Status = IoCallDriver(IoGetRelatedDeviceObject(FileObject), Irp);
    ObDereferenceObject((FileObject).cast());

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn Pin_fnClose(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let Context: PDISPATCH_CONTEXT;
    let IoStack: PIO_STACK_LOCATION;

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    Context = (*(*IoStack).FileObject).FsContext as PDISPATCH_CONTEXT;

    if !(*Context).Handle.is_null() {
        ZwClose((*Context).Handle);
    }

    if !(*Context).hMixerPin.is_null() {
        ZwClose((*Context).hMixerPin);
    }

    FreeItem(Context as *mut core::ffi::c_void);

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn SetMixerInputOutputFormat(
    FileObject: ntkrnl_sys::ddk::PFILE_OBJECT,
    InputFormat: crate::types::PKSDATAFORMAT,
    OutputFormat: crate::types::PKSDATAFORMAT,
) -> i32 {
    let mut PinRequest: crate::types::KSP_PIN = core::mem::zeroed();
    let mut BytesReturned: u32 = 0;
    let mut Status: i32;

    PinRequest.Property.Set = crate::types::KSPROPSETID_Connection;
    PinRequest.Property.Flags = KSPROPERTY_TYPE_SET;
    PinRequest.Property.Id = KSPROPERTY_CONNECTION_DATAFORMAT;

    PinRequest.PinId = 0;
    // DPRINT is disabled by NDEBUG in pin.c.
    Status = KsSynchronousIoControlDevice(
        FileObject,
        (0i32) as KPROCESSOR_MODE,
        IOCTL_KS_PROPERTY,
        core::ptr::addr_of_mut!(PinRequest) as *mut core::ffi::c_void,
        (core::mem::size_of::<KSP_PIN>()) as ULONG,
        InputFormat as *mut core::ffi::c_void,
        (*InputFormat).__bindgen_anon_1.as_ref().FormatSize,
        core::ptr::addr_of_mut!(BytesReturned),
    );
    if !ntkrnl_sys::base::NT_SUCCESS(Status) {
        return Status;
    }

    PinRequest.PinId = 1;
    // DPRINT is disabled by NDEBUG in pin.c.
    Status = KsSynchronousIoControlDevice(
        FileObject,
        (0i32) as KPROCESSOR_MODE,
        IOCTL_KS_PROPERTY,
        core::ptr::addr_of_mut!(PinRequest) as *mut core::ffi::c_void,
        (core::mem::size_of::<KSP_PIN>()) as ULONG,
        OutputFormat as *mut core::ffi::c_void,
        (*OutputFormat).__bindgen_anon_1.as_ref().FormatSize,
        core::ptr::addr_of_mut!(BytesReturned),
    );
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn CreateMixerPinAndSetFormat(
    KMixerHandle: HANDLE,
    PinConnect: *mut KSPIN_CONNECT,
    InputFormat: PKSDATAFORMAT,
    OutputFormat: PKSDATAFORMAT,
    MixerPinHandle: PHANDLE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut PinHandle: HANDLE = core::ptr::null_mut();
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();

    Status = KsCreatePin(
        KMixerHandle,
        PinConnect,
        GENERIC_READ | GENERIC_WRITE,
        &mut PinHandle,
    );

    if !NT_SUCCESS(Status) {
        dprint1!(233, "Failed to create Mixer Pin with %x\n", Status);
        return STATUS_UNSUCCESSFUL;
    }

    Status = ObReferenceObjectByHandle(
        PinHandle,
        GENERIC_READ | GENERIC_WRITE,
        IoFileObjectType,
        (0i32) as i8,
        (core::ptr::addr_of_mut!(FileObject) as *mut core::ffi::c_void).cast(),
        core::ptr::null_mut(),
    );

    if !NT_SUCCESS(Status) {
        dprint1!(243, "Failed to get file object with %x\n", Status);
        return STATUS_UNSUCCESSFUL;
    }

    Status = SetMixerInputOutputFormat(FileObject, InputFormat, OutputFormat);
    if !NT_SUCCESS(Status) {
        ObDereferenceObject((FileObject).cast());
        ZwClose(PinHandle);
        return Status;
    }

    ObDereferenceObject((FileObject).cast());

    *MixerPinHandle = PinHandle;
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn InstantiatePins(
    DeviceEntry: PKSAUDIO_DEVICE_ENTRY,
    Connect: PKSPIN_CONNECT,
    DispatchContext: PDISPATCH_CONTEXT,
    DeviceExtension: PSYSAUDIODEVEXT,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut RealPinHandle: *mut core::ffi::c_void = core::ptr::null_mut::<core::ffi::c_void>();
    let InputFormat: PKSDATAFORMAT_WAVEFORMATEX;
    let mut OutputFormat: PKSDATAFORMAT_WAVEFORMATEX =
        core::ptr::null_mut::<KSDATAFORMAT_WAVEFORMATEX>();
    let mut MixerPinConnect: PKSPIN_CONNECT = core::ptr::null_mut::<KSPIN_CONNECT>();
    let mut PinInstances: KSPIN_CINSTANCES = KSPIN_CINSTANCES {
        PossibleCount: 0,
        CurrentCount: 0,
    };

    Status = GetPinInstanceCount(DeviceEntry, core::ptr::addr_of_mut!(PinInstances), Connect);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if PinInstances.PossibleCount == 0 as u32 {
        return STATUS_UNSUCCESSFUL as NTSTATUS;
    }

    if PinInstances.CurrentCount == PinInstances.PossibleCount {
        return STATUS_UNSUCCESSFUL as NTSTATUS;
    }

    InputFormat = Connect.offset(1 as isize) as PKSDATAFORMAT_WAVEFORMATEX;

    Status = KsCreatePin(
        (*DeviceEntry).Handle,
        Connect,
        GENERIC_READ as u32 | GENERIC_WRITE as u32,
        core::ptr::addr_of_mut!(RealPinHandle),
    );

    if !NT_SUCCESS(Status) {
        return STATUS_UNSUCCESSFUL as NTSTATUS;
    }

    (*DispatchContext).Handle = RealPinHandle;
    (*DispatchContext).PinId = (*Connect).PinId;
    (*DispatchContext).AudioEntry = DeviceEntry;

    if !OutputFormat.is_null() {
        Status = CreateMixerPinAndSetFormat(
            (*DeviceExtension).KMixerHandle,
            MixerPinConnect,
            InputFormat as PKSDATAFORMAT,
            OutputFormat as PKSDATAFORMAT,
            core::ptr::addr_of_mut!((*DispatchContext).hMixerPin),
        );

        if !NT_SUCCESS(Status) {
            if DbgPrint(
                b"(%s:%d) Failed to create Mixer Pin with %x\n\0".as_ptr() as *const i8,
                b"drivers/wdm/audio/sysaudio/pin.c\0".as_ptr() as *const i8,
                383i32,
                Status,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                    b"drivers/wdm/audio/sysaudio/pin.c\0".as_ptr() as *const i8,
                    383i32,
                );
            }
            FreeItem(MixerPinConnect as *mut core::ffi::c_void);
        }
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn GetConnectRequest(Irp: PIRP, Result: *mut PKSPIN_CONNECT) -> NTSTATUS {
    let mut IoStack: PIO_STACK_LOCATION;
    let mut ObjectLength: ULONG;
    let mut ParametersLength: ULONG;
    let mut Buffer: PVOID;

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    ObjectLength = ((wcslen(KSSTRING_Pin.as_ptr() as *const u16) as usize)
        .wrapping_add(1)
        .wrapping_mul(core::mem::size_of::<WCHAR>())) as ULONG;

    if (ObjectLength as usize).wrapping_add(core::mem::size_of::<KSPIN_CONNECT>())
        > (*(*IoStack).FileObject).FileName.MaximumLength as usize
    {
        return STATUS_UNSUCCESSFUL;
    }

    ParametersLength =
        ((*(*IoStack).FileObject).FileName.MaximumLength as ULONG).wrapping_sub(ObjectLength);

    Buffer = AllocateItem(NonPagedPool, ParametersLength as SIZE_T);
    if Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    core::ptr::copy(
        (*(*IoStack).FileObject)
            .FileName
            .Buffer
            .offset((ObjectLength as usize).wrapping_div(core::mem::size_of::<WCHAR>()) as isize)
            as *const u8,
        Buffer as *mut u8,
        ParametersLength as usize,
    );

    *Result = Buffer as PKSPIN_CONNECT;

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn DispatchCreateSysAudioPin(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let IoStack: PIO_STACK_LOCATION;
    let DeviceEntry: PKSAUDIO_DEVICE_ENTRY;
    let mut Connect: PKSPIN_CONNECT = core::ptr::null_mut();
    let DispatchContext: PDISPATCH_CONTEXT;

    // DPRINT is disabled by NDEBUG in pin.c.

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    if (*IoStack).FileObject.is_null() {
        RtlAssert(
            b"IoStack->FileObject\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            b"drivers/wdm/audio/sysaudio/pin.c\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            447 as u32,
            core::ptr::null_mut::<i8>(),
        );
    }
    if (*(*IoStack).FileObject).RelatedFileObject.is_null() {
        RtlAssert(
            b"IoStack->FileObject->RelatedFileObject\0".as_ptr() as *const i8
                as *mut core::ffi::c_void,
            b"drivers/wdm/audio/sysaudio/pin.c\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            448 as u32,
            core::ptr::null_mut::<i8>(),
        );
    }
    if (*(*(*IoStack).FileObject).RelatedFileObject)
        .FsContext
        .is_null()
    {
        RtlAssert(
            b"IoStack->FileObject->RelatedFileObject->FsContext\0".as_ptr() as *const i8
                as *mut core::ffi::c_void,
            b"drivers/wdm/audio/sysaudio/pin.c\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            449 as u32,
            core::ptr::null_mut::<i8>(),
        );
    }

    DeviceEntry = (*(*(*IoStack).FileObject).RelatedFileObject).FsContext as PKSAUDIO_DEVICE_ENTRY;

    if !NT_SUCCESS(Status) {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return Status;
    }

    Status = GetConnectRequest(Irp, core::ptr::addr_of_mut!(Connect));

    if !NT_SUCCESS(Status) {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return Status;
    }

    DispatchContext = AllocateItem(
        NonPagedPool,
        core::mem::size_of::<DISPATCH_CONTEXT>() as SIZE_T,
    ) as PDISPATCH_CONTEXT;

    if DispatchContext.is_null() {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INSUFFICIENT_RESOURCES;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlZeroMemory(
        DispatchContext as *mut core::ffi::c_void,
        core::mem::size_of::<DISPATCH_CONTEXT>(),
    );

    Status = KsAllocateObjectHeader(
        core::ptr::addr_of_mut!((*DispatchContext).ObjectHeader),
        0,
        core::ptr::null_mut(),
        Irp,
        core::ptr::addr_of_mut!(PinTable),
    );

    if !NT_SUCCESS(Status) {
        FreeItem(DispatchContext as *mut core::ffi::c_void);
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return Status;
    }

    Status = InstantiatePins(
        DeviceEntry,
        Connect,
        DispatchContext,
        (*DeviceObject).DeviceExtension as PSYSAUDIODEVEXT,
    );

    if !NT_SUCCESS(Status) {
        KsFreeObjectHeader((*DispatchContext).ObjectHeader);
        FreeItem(DispatchContext as *mut core::ffi::c_void);
    } else {
        (*(*IoStack).FileObject).FsContext = DispatchContext as *mut core::ffi::c_void;
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    return Status;
}
