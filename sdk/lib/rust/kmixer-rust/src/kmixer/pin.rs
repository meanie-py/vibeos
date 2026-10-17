use crate::prelude::*;
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
    let mut Samples: ULONG;
    let mut NewIndex: ULONG;
    let mut OldIndex: ULONG;

    Samples = BufferLength / (BitsPerSample / 8) / OldChannels;

    if NewChannels > OldChannels {
        if BitsPerSample == 8 {
            let BufferOut: PUCHAR = ExAllocatePool(
                NonPagedPool,
                Samples.wrapping_mul(NewChannels) as SIZE_T,
            ) as PUCHAR;
            if BufferOut.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            NewIndex = 0;
            OldIndex = 0;
            while OldIndex < Samples.wrapping_mul(OldChannels) {
                let mut SubIndex: ULONG = 0;

                core::ptr::copy(
                    Buffer.offset(OldIndex as isize) as *const u8,
                    BufferOut.offset(NewIndex as isize) as *mut u8,
                    (OldChannels as usize).wrapping_mul(core::mem::size_of::<UCHAR>()),
                );

                loop {
                    *BufferOut.offset(
                        NewIndex.wrapping_add(OldChannels).wrapping_add(SubIndex) as isize,
                    ) = *Buffer.offset(
                        OldIndex.wrapping_add(SubIndex % OldChannels) as isize,
                    );
                    let fresh0 = SubIndex;
                    SubIndex = SubIndex.wrapping_add(1);
                    if !(fresh0 < NewChannels.wrapping_sub(OldChannels)) {
                        break;
                    }
                }
                NewIndex = NewIndex.wrapping_add(NewChannels);
                OldIndex = OldIndex.wrapping_add(OldChannels);
            }
            *Result = BufferOut as PVOID;
            *ResultLength = Samples.wrapping_mul(NewChannels);
        } else if BitsPerSample == 16 {
            let BufferOut: PUSHORT = ExAllocatePool(
                NonPagedPool,
                Samples.wrapping_mul(NewChannels) as SIZE_T,
            ) as PUSHORT;
            if BufferOut.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            NewIndex = 0;
            OldIndex = 0;
            while OldIndex < Samples.wrapping_mul(OldChannels) {
                let mut SubIndex: ULONG = 0;

                core::ptr::copy(
                    Buffer.offset(OldIndex as isize) as *const u8,
                    BufferOut.offset(NewIndex as isize) as *mut u8,
                    (OldChannels as usize).wrapping_mul(core::mem::size_of::<USHORT>()),
                );

                loop {
                    *BufferOut.offset(
                        NewIndex.wrapping_add(OldChannels).wrapping_add(SubIndex) as isize,
                    ) = *Buffer.offset(
                        OldIndex.wrapping_add(SubIndex % OldChannels) as isize,
                    ) as USHORT;
                    let fresh1 = SubIndex;
                    SubIndex = SubIndex.wrapping_add(1);
                    if !(fresh1 < NewChannels.wrapping_sub(OldChannels)) {
                        break;
                    }
                }
                NewIndex = NewIndex.wrapping_add(NewChannels);
                OldIndex = OldIndex.wrapping_add(OldChannels);
            }
            *Result = BufferOut as PVOID;
            *ResultLength = Samples.wrapping_mul(NewChannels);
        } else if BitsPerSample == 24 {
            let BufferOut: PUCHAR = ExAllocatePool(
                NonPagedPool,
                Samples.wrapping_mul(NewChannels) as SIZE_T,
            ) as PUCHAR;
            if BufferOut.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            NewIndex = 0;
            OldIndex = 0;
            while OldIndex < Samples.wrapping_mul(OldChannels) {
                let mut SubIndex: ULONG = 0;

                core::ptr::copy(
                    Buffer.offset(OldIndex as isize) as *const u8,
                    BufferOut.offset(NewIndex as isize) as *mut u8,
                    OldChannels.wrapping_mul(3) as usize,
                );

                loop {
                    core::ptr::copy(
                        Buffer.offset(
                            OldIndex
                                .wrapping_add(SubIndex % OldChannels)
                                .wrapping_mul(3) as isize,
                        ) as *const u8,
                        BufferOut.offset(
                            NewIndex
                                .wrapping_add(OldChannels)
                                .wrapping_add(SubIndex)
                                .wrapping_mul(3) as isize,
                        ) as *mut u8,
                        3 as usize,
                    );
                    let fresh2 = SubIndex;
                    SubIndex = SubIndex.wrapping_add(1);
                    if !(fresh2 < NewChannels.wrapping_sub(OldChannels)) {
                        break;
                    }
                }
                NewIndex = NewIndex.wrapping_add(NewChannels);
                OldIndex = OldIndex.wrapping_add(OldChannels);
            }
            *Result = BufferOut as PVOID;
            *ResultLength = Samples.wrapping_mul(NewChannels);
        } else if BitsPerSample == 32 {
            let BufferOut: PULONG = ExAllocatePool(
                NonPagedPool,
                Samples.wrapping_mul(NewChannels) as SIZE_T,
            ) as PULONG;
            if BufferOut.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            NewIndex = 0;
            OldIndex = 0;
            while OldIndex < Samples.wrapping_mul(OldChannels) {
                let mut SubIndex: ULONG = 0;

                core::ptr::copy(
                    Buffer.offset(OldIndex as isize) as *const u8,
                    BufferOut.offset(NewIndex as isize) as *mut u8,
                    (OldChannels as usize).wrapping_mul(core::mem::size_of::<ULONG>()),
                );

                loop {
                    *BufferOut.offset(
                        NewIndex.wrapping_add(OldChannels).wrapping_add(SubIndex) as isize,
                    ) = *Buffer.offset(
                        OldIndex.wrapping_add(SubIndex % OldChannels) as isize,
                    ) as ULONG;
                    let fresh3 = SubIndex;
                    SubIndex = SubIndex.wrapping_add(1);
                    if !(fresh3 < NewChannels.wrapping_sub(OldChannels)) {
                        break;
                    }
                }
                NewIndex = NewIndex.wrapping_add(NewChannels);
                OldIndex = OldIndex.wrapping_add(OldChannels);
            }
            *Result = BufferOut as PVOID;
            *ResultLength = Samples.wrapping_mul(NewChannels);
        }
    } else {
        let BufferOut: PUSHORT = ExAllocatePool(
            NonPagedPool,
            Samples.wrapping_mul(NewChannels) as SIZE_T,
        ) as PUSHORT;
        if BufferOut.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        NewIndex = 0;
        OldIndex = 0;
        while OldIndex < Samples.wrapping_mul(OldChannels) {
            /* TODO
             * mix stream instead of just dumping part of it ;)
             */
            core::ptr::copy(
                Buffer.offset(OldIndex as isize) as *const u8,
                BufferOut.offset(NewIndex as isize) as *mut u8,
                NewChannels.wrapping_mul(BitsPerSample / 8) as usize,
            );
            NewIndex = NewIndex.wrapping_add(NewChannels);
            OldIndex = OldIndex.wrapping_add(OldChannels);
        }

        *Result = BufferOut as PVOID;
        *ResultLength = Samples.wrapping_mul(NewChannels);
    }
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn PerformQualityConversion(
    Buffer: PUCHAR,
    BufferLength: ULONG,
    OldWidth: ULONG,
    NewWidth: ULONG,
    Result: *mut PVOID,
    ResultLength: PULONG,
) -> NTSTATUS {
    let mut Samples: ULONG;
    let mut Index: ULONG;

    ASSERT!(OldWidth != NewWidth);

    Samples = BufferLength.wrapping_div(OldWidth.wrapping_div(8 as ULONG));
    //DPRINT("Samples %u BufferLength %u\n", Samples, BufferLength);

    if OldWidth == 8 as ULONG && NewWidth == 16 as ULONG {
        let mut Sample: USHORT = 0;
        let mut BufferOut: PUSHORT = ExAllocatePool(
            NonPagedPool,
            ((Samples as usize).wrapping_mul(core::mem::size_of::<USHORT>() as usize)) as SIZE_T,
        ) as PUSHORT;
        if BufferOut.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        Index = 0 as ULONG;
        while Index < Samples {
            Sample = *Buffer.offset(Index as isize) as USHORT;
            Sample = (Sample as ULONG).wrapping_mul(2 as ULONG) as USHORT;
            if cfg!(target_arch = "x86") {
                Sample = _byteswap_ushort(Sample);
            }
            *BufferOut.offset(Index as isize) = Sample;
            Index = Index.wrapping_add(1);
        }
        *Result = BufferOut as PVOID;
        *ResultLength = (Samples as usize).wrapping_mul(core::mem::size_of::<USHORT>() as usize) as ULONG;
    } else if OldWidth == 8 as ULONG && NewWidth == 32 as ULONG {
        let mut Sample: ULONG = 0;
        let mut BufferOut: PULONG = ExAllocatePool(
            NonPagedPool,
            ((Samples as usize).wrapping_mul(::core::mem::size_of::<ULONG>())) as SIZE_T,
        ) as PULONG;
        if BufferOut.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        Index = 0 as ULONG;
        while Index < Samples {
            Sample = *Buffer.offset(Index as isize) as ULONG;
            Sample = Sample.wrapping_mul(16777216 as ULONG);
            if cfg!(target_arch = "x86") {
                Sample = _byteswap_ulong(Sample);
            }
            *BufferOut.offset(Index as isize) = Sample;
            Index = Index.wrapping_add(1);
        }
        *Result = BufferOut as PVOID;
        *ResultLength = (Samples as usize).wrapping_mul(::core::mem::size_of::<ULONG>()) as ULONG;
    } else if OldWidth == 16 as ULONG && NewWidth == 32 as ULONG {
        let mut Sample: ULONG = 0;
        let mut BufferIn: PUSHORT = Buffer as PUSHORT;
        let mut BufferOut: PULONG = ExAllocatePool(
            NonPagedPool,
            ((Samples as usize).wrapping_mul(::core::mem::size_of::<ULONG>())) as SIZE_T,
        ) as PULONG;
        if BufferOut.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        Index = 0 as ULONG;
        while Index < Samples {
            Sample = *BufferIn.offset(Index as isize) as ULONG;
            Sample = Sample.wrapping_mul(65536 as ULONG);
            if cfg!(target_arch = "x86") {
                Sample = _byteswap_ulong(Sample);
            }
            *BufferOut.offset(Index as isize) = Sample;
            Index = Index.wrapping_add(1);
        }
        *Result = BufferOut as PVOID;
        *ResultLength = (Samples as usize).wrapping_mul(::core::mem::size_of::<ULONG>()) as ULONG;
    }

    else if OldWidth == 16 as ULONG && NewWidth == 8 as ULONG {
        let mut Sample: USHORT = 0;
        let mut BufferIn: PUSHORT = Buffer as PUSHORT;
        let mut BufferOut: PUCHAR = ExAllocatePool(
            NonPagedPool,
            ((Samples as usize).wrapping_mul(core::mem::size_of::<UCHAR>() as usize)) as SIZE_T,
        ) as PUCHAR;
        if BufferOut.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        Index = 0 as ULONG;
        while Index < Samples {
            Sample = *BufferIn.offset(Index as isize) as USHORT;
            if cfg!(target_arch = "x86") {
                Sample = _byteswap_ushort(Sample);
            }
            Sample = Sample / (256 as USHORT);
            *BufferOut.offset(Index as isize) = (Sample & (0xFF as USHORT)) as UCHAR;
            Index = Index.wrapping_add(1);
        }
        *Result = BufferOut as PVOID;
        *ResultLength = (Samples as usize).wrapping_mul(core::mem::size_of::<UCHAR>() as usize) as ULONG;
    } else if OldWidth == 32 as ULONG && NewWidth == 8 as ULONG {
        let mut Sample: ULONG = 0;
        let mut BufferIn: PULONG = Buffer as PULONG;
        let mut BufferOut: PUCHAR = ExAllocatePool(
            NonPagedPool,
            ((Samples as usize).wrapping_mul(core::mem::size_of::<UCHAR>() as usize)) as SIZE_T,
        ) as PUCHAR;
        if BufferOut.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        Index = 0 as ULONG;
        while Index < Samples {
            Sample = *BufferIn.offset(Index as isize) as ULONG;
            if cfg!(target_arch = "x86") {
                Sample = _byteswap_ulong(Sample);
            }
            Sample = Sample / (16777216 as ULONG);
            *BufferOut.offset(Index as isize) = (Sample & (0xFF as ULONG)) as UCHAR;
            Index = Index.wrapping_add(1);
        }
        *Result = BufferOut as PVOID;
        *ResultLength = (Samples as usize).wrapping_mul(core::mem::size_of::<UCHAR>() as usize) as ULONG;
    } else if OldWidth == 32 as ULONG && NewWidth == 16 as ULONG {
        let mut Sample: USHORT = 0;
        let mut BufferIn: PULONG = Buffer as PULONG;
        let mut BufferOut: PUSHORT = ExAllocatePool(
            NonPagedPool,
            ((Samples as usize).wrapping_mul(core::mem::size_of::<USHORT>() as usize)) as SIZE_T,
        ) as PUSHORT;
        if BufferOut.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        Index = 0 as ULONG;
        while Index < Samples {
            // Divergence from pin.c: it stores the 32-bit sample in its USHORT `Sample`
            // first, so `Sample /= 65536` is always 0 and this branch emits silence. Keep the
            // full ULONG and take its high half, as the scaling in the other branches intends.
            let mut Wide: ULONG = *BufferIn.offset(Index as isize);
            if cfg!(target_arch = "x86") {
                Wide = _byteswap_ulong(Wide);
            }
            Wide /= 65536 as ULONG;
            Sample = (Wide & (0xFFFF as ULONG)) as USHORT;
            *BufferOut.offset(Index as isize) = Sample;
            Index = Index.wrapping_add(1);
        }
        *Result = BufferOut as PVOID;
        *ResultLength = (Samples as usize).wrapping_mul(core::mem::size_of::<USHORT>() as usize) as ULONG;
    } else {
        DPRINT1!(
            "Not implemented conversion OldWidth %u NewWidth %u\n",
            OldWidth,
            NewWidth
        );
        return STATUS_NOT_IMPLEMENTED;
    }

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
