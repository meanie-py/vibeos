use crate::prelude::*;
use ntkrnl_sys::constants::STATUS_INVALID_PARAMETER;
use ntkrnl_sys::ddk::{ExAllocatePool, ExFreePool, GUID, KFLOATING_SAVE, PDEVICE_OBJECT, PFILE_OBJECT, PIO_STACK_LOCATION, PIO_STATUS_BLOCK, PIRP, PLARGE_INTEGER, RtlAssert};
// ke.h: an import on x86, a FORCEINLINE no-op on amd64/arm64.
use ntkrnl_sys::inline::ke::{KeRestoreFloatingPointState, KeSaveFloatingPointState};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut KSPROPSETID_Connection: GUID = _GUID {
    Data1: 0x1d58c920 as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0xac9b as ::core::ffi::c_ushort,
    Data3: 0x11cf as ::core::ffi::c_ushort,
    Data4: [
        0xa5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x28 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

static mut PinTable: KSDISPATCH_TABLE = unsafe {
    KSDISPATCH_TABLE {
        DeviceIoControl: Some(
            Pin_fnDeviceIoControl,
        ),
        Read: Some(Pin_fnRead),
        Write: Some(Pin_fnWrite),
        Flush: Some(Pin_fnFlush),
        Close: Some(Pin_fnClose),
        QuerySecurity: Some(
            Pin_fnQuerySecurity,
        ),
        SetSecurity: Some(
            Pin_fnSetSecurity,
        ),
        FastDeviceIoControl: Some(
            Pin_fnFastDeviceIoControl,
        ),
        FastRead: Some(
            Pin_fnFastRead,
        ),
        FastWrite: Some(
            Pin_fnFastWrite,
        ),
    }
};

// #define NDEBUG (empty object-like macro, no value to emit)
#[no_mangle]
pub unsafe extern "C" fn PerformSampleRateConversion(
    mut Buffer: PUCHAR,
    mut BufferLength: ULONG,
    mut OldRate: ULONG,
    mut NewRate: ULONG,
    mut BytesPerSample: ULONG,
    mut NumChannels: ULONG,
    mut Result: *mut PVOID,
    mut ResultLength: PULONG,
) -> NTSTATUS {
    let mut FloatSave: KFLOATING_SAVE = core::mem::zeroed();
    let mut Status: NTSTATUS = 0;
    let mut Index: ULONG = 0;
    let mut State: *mut SRC_STATE = core::ptr::null_mut::<SRC_STATE>();
    let mut Data: SRC_DATA = SRC_DATA {
        data_in: core::ptr::null::<f32>(),
        data_out: core::ptr::null_mut::<f32>(),
        input_frames: 0,
        output_frames: 0,
        input_frames_used: 0,
        output_frames_gen: 0,
        end_of_input: 0,
        src_ratio: 0.,
    };
    let mut ResultOut: PUCHAR = core::ptr::null_mut::<u8>();
    let mut error: core::ffi::c_int = 0;
    let mut FloatIn: PFLOAT = core::ptr::null_mut::<FLOAT>();
    let mut FloatOut: PFLOAT = core::ptr::null_mut::<FLOAT>();
    let mut NumSamples: ULONG = 0;
    let mut NewSamples: ULONG = 0;

    DPRINT!(
        "PerformSampleRateConversion OldRate %u NewRate %u BytesPerSample %u NumChannels %u Irql %u\n",
        OldRate,
        NewRate,
        BytesPerSample,
        NumChannels,
        KeGetCurrentIrql()
    );

    ASSERT!(
        BytesPerSample == 1
            || BytesPerSample == 2
            || BytesPerSample == 4
    );

    /* first acquire float save context */
    Status = KeSaveFloatingPointState(core::ptr::addr_of_mut!(FloatSave));

    if !NT_SUCCESS!(Status) {
        DPRINT1!("KeSaveFloatingPointState failed with %x\n", Status);
        return Status;
    }

    NumSamples = BufferLength.wrapping_div(BytesPerSample.wrapping_mul(NumChannels));

    FloatIn = ExAllocatePool(
        NonPagedPool,
        (NumSamples.wrapping_mul(NumChannels) as SIZE_T)
            .wrapping_mul(core::mem::size_of::<FLOAT>() as SIZE_T),
    ) as PFLOAT;
    if FloatIn.is_null() {
        KeRestoreFloatingPointState(core::ptr::addr_of_mut!(FloatSave));
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    NewSamples = (((NumSamples as ULONG64).wrapping_mul(NewRate as ULONG64))
        .wrapping_add((OldRate.wrapping_div(2)) as ULONG64))
        .wrapping_div(OldRate as ULONG64)
        .wrapping_add(2 as ULONG64) as ULONG;

    FloatOut = ExAllocatePool(
        NonPagedPool,
        (NewSamples.wrapping_mul(NumChannels) as SIZE_T)
            .wrapping_mul(core::mem::size_of::<FLOAT>() as SIZE_T),
    ) as PFLOAT;
    if FloatOut.is_null() {
        ExFreePool(FloatIn as PVOID);
        KeRestoreFloatingPointState(core::ptr::addr_of_mut!(FloatSave));
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    ResultOut = ExAllocatePool(
        NonPagedPool,
        NewSamples
            .wrapping_mul(NumChannels)
            .wrapping_mul(BytesPerSample) as SIZE_T,
    ) as PUCHAR;
    if ResultOut.is_null() {
        ExFreePool(FloatIn as PVOID);
        ExFreePool(FloatOut as PVOID);
        KeRestoreFloatingPointState(core::ptr::addr_of_mut!(FloatSave));
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    State = src_new(
        SRC_SINC_FASTEST as core::ffi::c_int,
        NumChannels as core::ffi::c_int,
        core::ptr::addr_of_mut!(error),
    );
    if State.is_null() {
        DPRINT1!("src_new failed with %x\n", error);
        KeRestoreFloatingPointState(core::ptr::addr_of_mut!(FloatSave));
        ExFreePool(FloatIn as PVOID);
        ExFreePool(FloatOut as PVOID);
        ExFreePool(ResultOut as PVOID);
        return STATUS_UNSUCCESSFUL;
    }

    /* fixme use asm */
    if BytesPerSample == 1 {
        Index = 0;
        while Index < NumSamples.wrapping_mul(NumChannels) {
            *FloatIn.offset(Index as isize) = ((*Buffer.offset(Index as isize) as core::ffi::c_int
                as core::ffi::c_double)
                / (1.0f64 * 0x80 as core::ffi::c_double))
                as core::ffi::c_float as FLOAT;
            Index = Index.wrapping_add(1);
        }
    } else if BytesPerSample == 2 {
        src_short_to_float_array(
            Buffer as *mut core::ffi::c_short,
            FloatIn as *mut core::ffi::c_float,
            NumSamples.wrapping_mul(NumChannels) as core::ffi::c_int,
        );
    } else if BytesPerSample == 4 {
        src_int_to_float_array(
            Buffer as *mut core::ffi::c_int,
            FloatIn as *mut core::ffi::c_float,
            NumSamples.wrapping_mul(NumChannels) as core::ffi::c_int,
        );
    }

    Data.data_in = FloatIn as *const core::ffi::c_float;
    Data.data_out = FloatOut as *mut core::ffi::c_float;
    Data.input_frames = NumSamples as core::ffi::c_long;
    Data.output_frames = NewSamples as core::ffi::c_long;
    Data.src_ratio = NewRate as core::ffi::c_double / OldRate as core::ffi::c_double;

    error = src_process(State, core::ptr::addr_of_mut!(Data));
    if error != 0 {
        DPRINT1!("src_process failed with %x\n", error);
        KeRestoreFloatingPointState(core::ptr::addr_of_mut!(FloatSave));
        ExFreePool(FloatIn as PVOID);
        ExFreePool(FloatOut as PVOID);
        ExFreePool(ResultOut as PVOID);
        return STATUS_UNSUCCESSFUL;
    }

    if BytesPerSample == 1 {
        /* FIXME perform over/under clipping */

        Index = 0;
        while Index < (Data.output_frames_gen as ULONG).wrapping_mul(NumChannels) {
            *ResultOut.offset(Index as isize) =
                (lrintf(*FloatOut.offset(Index as isize) as core::ffi::c_float)
                    >> 24) as core::ffi::c_uchar;
            Index = Index.wrapping_add(1);
        }
    } else if BytesPerSample == 2 {
        let mut Res: PUSHORT = ResultOut as PUSHORT;

        src_float_to_short_array(
            FloatOut as *const core::ffi::c_float,
            Res as *mut core::ffi::c_short,
            (Data.output_frames_gen as ULONG).wrapping_mul(NumChannels) as core::ffi::c_int,
        );
    } else if BytesPerSample == 4 {
        let mut Res: PULONG = ResultOut as PULONG;

        src_float_to_int_array(
            FloatOut as *const core::ffi::c_float,
            Res as *mut core::ffi::c_int,
            (Data.output_frames_gen as ULONG).wrapping_mul(NumChannels) as core::ffi::c_int,
        );
    }

    *Result = ResultOut as PVOID;
    *ResultLength = (Data.output_frames_gen as ULONG)
        .wrapping_mul(BytesPerSample)
        .wrapping_mul(NumChannels) as core::ffi::c_ulong;
    ExFreePool(FloatIn as PVOID);
    ExFreePool(FloatOut as PVOID);
    src_delete(State);
    KeRestoreFloatingPointState(core::ptr::addr_of_mut!(FloatSave));
    return STATUS_SUCCESS;
}
/// Divergence from pin.c, which is wrong for every width but 8 bits: it allocates
/// `Samples * NewChannels` BYTES whatever the sample width, indexes the input as bytes
/// (`Buffer[OldIndex]` is a PUCHAR) while the output is USHORT/ULONG-indexed, and its
/// `do { } while (SubIndex++ < NewChannels - OldChannels)` writes one channel more than
/// it fills -- the first slot of the next frame, which on the last frame is past the end.
/// So a 16-bit mono stream reaching Pin_fnFastWrite overran the pool and produced
/// garbage. Its down-mix writes `NewChannels * BitsPerSample/8` bytes at a USHORT stride
/// from an unscaled byte offset, and it reported `*ResultLength` in samples, not bytes.
/// An unhandled width returned STATUS_SUCCESS with `*Result` never written.
///
/// Here every width is one frame copy: output channel `c` takes input channel
/// `c % OldChannels`, which is what the C does for 8-bit (2 channels stretched to 4 look
/// like LRLR), and down-mixing keeps the first `NewChannels` (the C's TODO: no mixing).
#[no_mangle]
pub unsafe extern "C" fn PerformChannelConversion(
    Buffer: PUCHAR,
    BufferLength: ULONG,
    OldChannels: ULONG,
    NewChannels: ULONG,
    BitsPerSample: ULONG,
    Result: *mut PVOID,
    ResultLength: PULONG,
) -> NTSTATUS {
    let Width = match BitsPerSample {
        8 | 16 | 24 | 32 => (BitsPerSample / 8) as usize,
        _ => {
            DPRINT1!("Not implemented channel conversion BitsPerSample %u\n", BitsPerSample);
            return STATUS_NOT_IMPLEMENTED;
        }
    };
    if OldChannels == 0 || NewChannels == 0 {
        return STATUS_INVALID_PARAMETER;
    }
    let Samples = (BufferLength as usize) / Width / (OldChannels as usize);
    let OldFrame = (OldChannels as usize) * Width;
    let NewFrame = (NewChannels as usize) * Width;
    let Size = match Samples.checked_mul(NewFrame) {
        Some(Size) if Size <= ULONG::MAX as usize => Size,
        _ => return STATUS_INSUFFICIENT_RESOURCES,
    };

    let BufferOut: PUCHAR = ExAllocatePool(NonPagedPool, Size as SIZE_T) as PUCHAR;
    if BufferOut.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    for Frame in 0..Samples {
        let In = Buffer.add(Frame * OldFrame);
        let Out = BufferOut.add(Frame * NewFrame);
        for Channel in 0..NewChannels as usize {
            let Source = Channel % OldChannels as usize;
            core::ptr::copy_nonoverlapping(In.add(Source * Width), Out.add(Channel * Width), Width);
        }
    }
    *Result = BufferOut as PVOID;
    *ResultLength = Size as ULONG;
    return STATUS_SUCCESS;
}
/// Divergence from pin.c on every pair but 16 -> 32. 8-bit PCM is unsigned with silence
/// at 0x80 while 16/32-bit are signed, and the C never re-centres: its 8 -> 32 maps
/// silence to 0x80000000, full-scale negative, and 8 -> 16 multiplies by 2 where its own
/// siblings scale by the width difference (8 -> 32 is `<< 24`, 16 -> 32 is `<< 16`).
/// Its 32 -> 16 stores the sample in a USHORT before dividing by 65536, so it is always
/// silence. And under `_X86_` it byte-swaps every sample, so the same little-endian
/// stream came out different on i386 than on amd64 from the identical source.
///
/// Here each sample is widened to a full-scale signed 32-bit value and narrowed by
/// truncation (no dither, as the C), with 8-bit samples' sign bit flipped both ways.
#[no_mangle]
pub unsafe extern "C" fn PerformQualityConversion(
    Buffer: PUCHAR,
    BufferLength: ULONG,
    OldWidth: ULONG,
    NewWidth: ULONG,
    Result: *mut PVOID,
    ResultLength: PULONG,
) -> NTSTATUS {
    ASSERT!(OldWidth != NewWidth);

    let (InBytes, OutBytes) = match (OldWidth, NewWidth) {
        (8, 16) | (8, 32) | (16, 8) | (16, 32) | (32, 8) | (32, 16) => {
            ((OldWidth / 8) as usize, (NewWidth / 8) as usize)
        }
        _ => {
            DPRINT1!(
                "Not implemented conversion OldWidth %u NewWidth %u\n",
                OldWidth,
                NewWidth
            );
            return STATUS_NOT_IMPLEMENTED;
        }
    };
    let Samples = (BufferLength as usize) / InBytes;
    let Size = match Samples.checked_mul(OutBytes) {
        Some(Size) if Size <= ULONG::MAX as usize => Size,
        _ => return STATUS_INSUFFICIENT_RESOURCES,
    };

    let BufferOut: PUCHAR = ExAllocatePool(NonPagedPool, Size as SIZE_T) as PUCHAR;
    if BufferOut.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    for Index in 0..Samples {
        let Full: ULONG = match InBytes {
            1 => ((*Buffer.add(Index) ^ 0x80) as ULONG) << 24,
            2 => (core::ptr::read_unaligned(Buffer.add(Index * 2) as *const USHORT) as ULONG) << 16,
            _ => core::ptr::read_unaligned(Buffer.add(Index * 4) as *const ULONG),
        };
        match OutBytes {
            1 => *BufferOut.add(Index) = ((Full >> 24) as UCHAR) ^ 0x80,
            2 => core::ptr::write_unaligned(BufferOut.add(Index * 2) as *mut USHORT, (Full >> 16) as USHORT),
            _ => core::ptr::write_unaligned(BufferOut.add(Index * 4) as *mut ULONG, Full),
        }
    }
    *Result = BufferOut as PVOID;
    *ResultLength = Size as ULONG;
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn Pin_fnDeviceIoControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut IoStack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut Property: PKSP_PIN = core::ptr::null_mut();
    //DPRINT1("Pin_fnDeviceIoControl called DeviceObject %p Irp %p\n", DeviceObject);

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    if (*IoStack).Parameters.DeviceIoControl.InputBufferLength as usize == core::mem::size_of::<KSP_PIN>()
        && (*IoStack).Parameters.DeviceIoControl.OutputBufferLength as usize == core::mem::size_of::<KSDATAFORMAT_WAVEFORMATEX>() as usize
    {
        Property = (*IoStack).Parameters.DeviceIoControl.Type3InputBuffer as PKSP_PIN;

        if IsEqualGUIDAligned(
            core::ptr::addr_of_mut!((*Property).Property.Set) as *const GUID,
            core::ptr::addr_of!(KSPROPSETID_Connection) as *const GUID,
        ) != false
        {
            if (*Property).Property.Id == KSPROPERTY_CONNECTION_DATAFORMAT && (*Property).Property.Flags == KSPROPERTY_TYPE_SET {
                let mut Formats: PKSDATAFORMAT_WAVEFORMATEX = core::ptr::null_mut();
                let mut WaveFormat: PKSDATAFORMAT_WAVEFORMATEX = core::ptr::null_mut();

                Formats = (*(*IoStack).FileObject).FsContext2 as PKSDATAFORMAT_WAVEFORMATEX;
                WaveFormat = (*Irp).UserBuffer as PKSDATAFORMAT_WAVEFORMATEX;

                ASSERT!((*Property).PinId == 0 || (*Property).PinId == 1);
                ASSERT!(!Formats.is_null());
                ASSERT!(!WaveFormat.is_null());

                (*Formats.offset((*Property).PinId as isize)).WaveFormatEx.nChannels =
                    (*WaveFormat).WaveFormatEx.nChannels;
                (*Formats.offset((*Property).PinId as isize)).WaveFormatEx.wBitsPerSample =
                    (*WaveFormat).WaveFormatEx.wBitsPerSample;
                (*Formats.offset((*Property).PinId as isize)).WaveFormatEx.nSamplesPerSec =
                    (*WaveFormat).WaveFormatEx.nSamplesPerSec;

                (*Irp).IoStatus.Information = 0;
                (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
                IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));
                return STATUS_SUCCESS;
            }
        }
    }
    DPRINT1!(
        "Size %u Expected %u\n",
        (*IoStack).Parameters.DeviceIoControl.OutputBufferLength,
        core::mem::size_of::<KSDATAFORMAT_WAVEFORMATEX>() as usize
    );
    (*Irp).IoStatus.Information = 0;
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_UNSUCCESSFUL;
    IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));
    return STATUS_UNSUCCESSFUL;
}

#[no_mangle]
pub unsafe extern "system" fn Pin_fnRead(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    UNIMPLEMENTED!();

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_UNSUCCESSFUL;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));
    return STATUS_UNSUCCESSFUL;
}
#[no_mangle]
pub unsafe extern "system" fn Pin_fnWrite(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    UNIMPLEMENTED!();

    (*Irp).IoStatus.Information = 0;
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn Pin_fnFlush(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    UNIMPLEMENTED!();

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_UNSUCCESSFUL;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
    return STATUS_UNSUCCESSFUL;
}
#[no_mangle]
pub unsafe extern "system" fn Pin_fnClose(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    UNIMPLEMENTED!();

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn Pin_fnQuerySecurity(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    UNIMPLEMENTED!();

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_UNSUCCESSFUL;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));
    return STATUS_UNSUCCESSFUL;
}
#[no_mangle]
pub unsafe extern "system" fn Pin_fnSetSecurity(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    UNIMPLEMENTED!();

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_UNSUCCESSFUL;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));
    return STATUS_UNSUCCESSFUL;
}

#[no_mangle]
pub unsafe extern "system" fn Pin_fnFastDeviceIoControl(
    FileObject: PFILE_OBJECT,
    Wait: BOOLEAN,
    InputBuffer: PVOID,
    InputBufferLength: ULONG,
    OutputBuffer: PVOID,
    OutputBufferLength: ULONG,
    IoControlCode: ULONG,
    IoStatus: PIO_STATUS_BLOCK,
    DeviceObject: PDEVICE_OBJECT,
) -> BOOLEAN {
    // UNIMPLEMENTED;
    FALSE
}

#[no_mangle]
pub unsafe extern "system" fn Pin_fnFastRead(
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    Length: ULONG,
    Wait: BOOLEAN,
    LockKey: ULONG,
    Buffer: PVOID,
    IoStatus: PIO_STATUS_BLOCK,
    DeviceObject: PDEVICE_OBJECT,
) -> BOOLEAN {
    // UNIMPLEMENTED;
    FALSE
}
#[no_mangle]
pub unsafe extern "system" fn Pin_fnFastWrite(
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    Length: ULONG,
    Wait: BOOLEAN,
    LockKey: ULONG,
    Buffer: PVOID,
    IoStatus: PIO_STATUS_BLOCK,
    DeviceObject: PDEVICE_OBJECT,
) -> BOOLEAN {
    let mut StreamHeader: PKSSTREAM_HEADER = core::ptr::null_mut();
    let mut BufferOut: PVOID = core::ptr::null_mut();
    let mut BufferLength: ULONG = 0;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut Formats: PKSDATAFORMAT_WAVEFORMATEX = core::ptr::null_mut();
    let mut InputFormat: PKSDATAFORMAT_WAVEFORMATEX = core::ptr::null_mut();
    let mut OutputFormat: PKSDATAFORMAT_WAVEFORMATEX = core::ptr::null_mut();

    DPRINT!("Pin_fnFastWrite called DeviceObject %p Irp %p\n", DeviceObject);

    Formats = (*FileObject).FsContext2 as PKSDATAFORMAT_WAVEFORMATEX;

    InputFormat = Formats;
    OutputFormat = Formats.offset(1 as isize);
    StreamHeader = Buffer as PKSSTREAM_HEADER;

    DPRINT!(
        "Num Channels %u Old Channels %u\n SampleRate %u Old SampleRate %u\n BitsPerSample %u Old BitsPerSample %u\n",
        (*InputFormat).WaveFormatEx.nChannels,
        (*OutputFormat).WaveFormatEx.nChannels,
        (*InputFormat).WaveFormatEx.nSamplesPerSec,
        (*OutputFormat).WaveFormatEx.nSamplesPerSec,
        (*InputFormat).WaveFormatEx.wBitsPerSample,
        (*OutputFormat).WaveFormatEx.wBitsPerSample
    );

    if (*InputFormat).WaveFormatEx.wBitsPerSample != (*OutputFormat).WaveFormatEx.wBitsPerSample {
        Status = PerformQualityConversion(
            (*StreamHeader).Data as PUCHAR,
            (*StreamHeader).DataUsed,
            (*InputFormat).WaveFormatEx.wBitsPerSample as ULONG,
            (*OutputFormat).WaveFormatEx.wBitsPerSample as ULONG,
            core::ptr::addr_of_mut!(BufferOut),
            core::ptr::addr_of_mut!(BufferLength),
        );
        if NT_SUCCESS!(Status) {
            ExFreePool((*StreamHeader).Data);
            (*StreamHeader).Data = BufferOut;
            (*StreamHeader).DataUsed = BufferLength;
        }
    }

    if (*InputFormat).WaveFormatEx.nChannels != (*OutputFormat).WaveFormatEx.nChannels {
        Status = PerformChannelConversion(
            (*StreamHeader).Data as PUCHAR,
            (*StreamHeader).DataUsed,
            (*InputFormat).WaveFormatEx.nChannels as ULONG,
            (*OutputFormat).WaveFormatEx.nChannels as ULONG,
            (*OutputFormat).WaveFormatEx.wBitsPerSample as ULONG,
            core::ptr::addr_of_mut!(BufferOut),
            core::ptr::addr_of_mut!(BufferLength),
        );

        if NT_SUCCESS!(Status) {
            ExFreePool((*StreamHeader).Data);
            (*StreamHeader).Data = BufferOut;
            (*StreamHeader).DataUsed = BufferLength;
        }
    }

    if (*InputFormat).WaveFormatEx.nSamplesPerSec != (*OutputFormat).WaveFormatEx.nSamplesPerSec {
        Status = PerformSampleRateConversion(
            (*StreamHeader).Data as PUCHAR,
            (*StreamHeader).DataUsed,
            (*InputFormat).WaveFormatEx.nSamplesPerSec as ULONG,
            (*OutputFormat).WaveFormatEx.nSamplesPerSec as ULONG,
            ((*OutputFormat).WaveFormatEx.wBitsPerSample as ULONG / 8),
            (*OutputFormat).WaveFormatEx.nChannels as ULONG,
            core::ptr::addr_of_mut!(BufferOut),
            core::ptr::addr_of_mut!(BufferLength),
        );
        if NT_SUCCESS!(Status) {
            ExFreePool((*StreamHeader).Data);
            (*StreamHeader).Data = BufferOut;
            (*StreamHeader).DataUsed = BufferLength;
        }
    }

    (*IoStatus).__bindgen_anon_1.Status = Status;

    if NT_SUCCESS!(Status) {
        return TRUE;
    } else {
        return FALSE;
    }
}
#[no_mangle]
pub unsafe extern "C" fn CreatePin(Irp: PIRP) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut ObjectHeader: KSOBJECT_HEADER = core::ptr::null_mut();
    let mut DataFormat: PKSDATAFORMAT;
    let mut IoStack: PIO_STACK_LOCATION;

    DataFormat = ExAllocatePool(
        NonPagedPool,
        (core::mem::size_of::<KSDATAFORMAT_WAVEFORMATEX>() * 2) as SIZE_T,
    ) as PKSDATAFORMAT;
    if DataFormat.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlZeroMemory!(
        DataFormat as *mut core::ffi::c_void,
        core::mem::size_of::<KSDATAFORMAT_WAVEFORMATEX>() * 2,
    );

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    (*(*IoStack).FileObject).FsContext2 = DataFormat as PVOID;

    /* allocate object header */
    Status = KsAllocateObjectHeader(
        core::ptr::addr_of_mut!(ObjectHeader),
        0,
        core::ptr::null_mut(),
        Irp,
        core::ptr::addr_of_mut!(PinTable),
    );
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn calloc(Elements: size_t, ElementSize: size_t) -> *mut ::core::ffi::c_void {
    let mut Index: ULONG;
    let mut Block: PUCHAR = ExAllocatePool(
        NonPagedPool,
        (Elements as SIZE_T).wrapping_mul(ElementSize as SIZE_T),
    ) as PUCHAR;
    if Block.is_null() {
        return core::ptr::null_mut();
    }

    Index = 0 as ULONG;
    while (Index as size_t) < Elements.wrapping_mul(ElementSize) {
        *Block.offset(Index as isize) = 0 as ::core::ffi::c_uchar;
        Index = Index.wrapping_add(1);
    }

    return Block as *mut ::core::ffi::c_void;
}

#[no_mangle]
pub unsafe extern "C" fn free(Block: PVOID) {
    ExFreePool(Block);
}

#[no_mangle]
pub unsafe extern "C" fn malloc(Size: usize) -> *mut core::ffi::c_void {
    return ExAllocatePool(NonPagedPool, (Size) as SIZE_T) as *mut core::ffi::c_void;
}
#[no_mangle]
pub unsafe extern "C" fn _assert(
    mut Message: *const ::core::ffi::c_char,
    mut File: *const ::core::ffi::c_char,
    mut Line: ::core::ffi::c_uint,
) {
    RtlAssert(
        Message as PVOID,
        File as PVOID,
        Line as ULONG,
        ::core::ptr::null_mut::<CHAR>(),
    );
}
