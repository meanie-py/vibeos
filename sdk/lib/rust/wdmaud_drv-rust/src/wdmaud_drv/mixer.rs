// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_00FB6900C11A: [u16; 28] = [
    0x0057u16, 0x0072, 0x0069, 0x0074, 0x0065, 0x0046, 0x0069, 0x006C, 0x0065, 0x0045,
    0x0078, 0x0020, 0x0066, 0x0061, 0x0069, 0x006C, 0x0065, 0x0064, 0x0020, 0x0077,
    0x0069, 0x0074, 0x0068, 0x0020, 0x0025, 0x0078, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_113E6451A8DB: [u16; 33] = [
    0x0050u16, 0x0065, 0x0072, 0x0066, 0x006F, 0x0072, 0x006D, 0x0051, 0x0075, 0x0061,
    0x006C, 0x0069, 0x0074, 0x0079, 0x0043, 0x006F, 0x006E, 0x0076, 0x0065, 0x0072,
    0x0073, 0x0069, 0x006F, 0x006E, 0x0020, 0x0066, 0x0061, 0x0069, 0x006C, 0x0065,
    0x0064, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_201283C65888: [u16; 36] = [
    0x0050u16, 0x0065, 0x0072, 0x0066, 0x006F, 0x0072, 0x006D, 0x0053, 0x0061, 0x006D,
    0x0070, 0x006C, 0x0065, 0x0052, 0x0061, 0x0074, 0x0065, 0x0043, 0x006F, 0x006E,
    0x0076, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x0020, 0x0066, 0x0061,
    0x0069, 0x006C, 0x0065, 0x0064, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_4683ED71C250: [u16; 33] = [
    0x0050u16, 0x0065, 0x0072, 0x0066, 0x006F, 0x0072, 0x006D, 0x0043, 0x0068, 0x0061,
    0x006E, 0x006E, 0x0065, 0x006C, 0x0043, 0x006F, 0x006E, 0x0076, 0x0065, 0x0072,
    0x0073, 0x0069, 0x006F, 0x006E, 0x0020, 0x0066, 0x0061, 0x0069, 0x006C, 0x0065,
    0x0064, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_D1041FF85E08: [u16; 56] = [
    0x0050u16, 0x0065, 0x0072, 0x0066, 0x006F, 0x0072, 0x006D, 0x0043, 0x0068, 0x0061,
    0x006E, 0x006E, 0x0065, 0x006C, 0x0043, 0x006F, 0x006E, 0x0076, 0x0065, 0x0072,
    0x0073, 0x0069, 0x006F, 0x006E, 0x0020, 0x004F, 0x006C, 0x0064, 0x0043, 0x0068,
    0x0061, 0x006E, 0x006E, 0x0065, 0x006C, 0x0073, 0x0020, 0x0025, 0x0075, 0x0020,
    0x004E, 0x0065, 0x0077, 0x0043, 0x0068, 0x0061, 0x006E, 0x006E, 0x0065, 0x006C,
    0x0073, 0x0020, 0x0025, 0x0075, 0x000A, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[no_mangle]
pub unsafe extern "C" fn PerformSampleRateConversion(
    Buffer: PUCHAR,
    BufferLength: ULONG,
    OldRate: ULONG,
    NewRate: ULONG,
    BytesPerSample: ULONG,
    NumChannels: ULONG,
    Result: *mut PVOID,
    ResultLength: PULONG,
) -> DWORD {
    let mut Index: ULONG;
    let mut State: *mut SRC_STATE;
    let mut Data: SRC_DATA = core::mem::zeroed();
    let mut ResultOut: PUCHAR;
    let mut error: ::core::ffi::c_int = core::mem::zeroed();
    let mut FloatIn: PFLOAT;
    let mut FloatOut: PFLOAT;
    let mut NumSamples: ULONG;
    let mut NewSamples: ULONG;

    ASSERT!(BytesPerSample == 1 || BytesPerSample == 2 || BytesPerSample == 4);

    NumSamples = BufferLength.wrapping_div(BytesPerSample.wrapping_mul(NumChannels));

    FloatIn = HeapAlloc(GetProcessHeap(), 0, (NumSamples.wrapping_mul(NumChannels) as SIZE_T).wrapping_mul(core::mem::size_of::<FLOAT>() as SIZE_T)) as PFLOAT;
    if FloatIn.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    NewSamples = ((((NumSamples as ULONG64).wrapping_mul(NewRate as ULONG64)).wrapping_add((OldRate.wrapping_div(2)) as ULONG64)).wrapping_div(OldRate as ULONG64)).wrapping_add(2 as ULONG64) as ULONG;

    FloatOut = HeapAlloc(GetProcessHeap(), 0, (NewSamples.wrapping_mul(NumChannels) as SIZE_T).wrapping_mul(core::mem::size_of::<FLOAT>() as SIZE_T)) as PFLOAT;
    if FloatOut.is_null() {
        HeapFree(GetProcessHeap(), 0, FloatIn as PVOID);
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    ResultOut = HeapAlloc(GetProcessHeap(), 0, (NewSamples.wrapping_mul(NumChannels).wrapping_mul(BytesPerSample)) as SIZE_T) as PUCHAR;
    if ResultOut.is_null() {
        HeapFree(GetProcessHeap(), 0, FloatIn as PVOID);
        HeapFree(GetProcessHeap(), 0, FloatOut as PVOID);
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    State = src_new((SRC_SINC_FASTEST as i32), NumChannels as ::core::ffi::c_int, core::ptr::addr_of_mut!(error));
    if State.is_null() {
        HeapFree(GetProcessHeap(), 0, FloatIn as PVOID);
        HeapFree(GetProcessHeap(), 0, FloatOut as PVOID);
        HeapFree(GetProcessHeap(), 0, ResultOut as PVOID);
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    /* fixme use asm */
    if BytesPerSample == 1 {
        Index = 0;
        while Index < NumSamples.wrapping_mul(NumChannels) {
            *FloatIn.add(Index as usize) = ((*Buffer.add(Index as usize) as f64) / (1.0f64 * (0x80) as f64)) as f32;
            Index = Index.wrapping_add(1);
        }
    } else if BytesPerSample == 2 {
        src_short_to_float_array(Buffer as *mut ::core::ffi::c_short, FloatIn as *mut f32, NumSamples.wrapping_mul(NumChannels) as ::core::ffi::c_int);
    } else if BytesPerSample == 4 {
        src_int_to_float_array(Buffer as *mut ::core::ffi::c_int, FloatIn as *mut f32, NumSamples.wrapping_mul(NumChannels) as ::core::ffi::c_int);
    }

    Data.data_in = FloatIn as *const f32;
    Data.data_out = FloatOut as *mut f32;
    Data.input_frames = NumSamples as ::core::ffi::c_long;
    Data.output_frames = NewSamples as ::core::ffi::c_long;
    Data.src_ratio = NewRate as f64 / OldRate as f64;

    error = src_process(State, core::ptr::addr_of_mut!(Data));
    if error != 0 {
        DPRINT1!("src_process failed with %x\n", error);
        HeapFree(GetProcessHeap(), 0, FloatIn as PVOID);
        HeapFree(GetProcessHeap(), 0, FloatOut as PVOID);
        HeapFree(GetProcessHeap(), 0, ResultOut as PVOID);
        return ERROR_INVALID_DATA;
    }

    if BytesPerSample == 1 {
        /* FIXME perform over/under clipping */

        Index = 0;
        while Index < (Data.output_frames_gen as ULONG).wrapping_mul(NumChannels) {
            *ResultOut.add(Index as usize) = ((lrintf(*FloatOut.add(Index as usize) as f32) >> 24) as UCHAR);
            Index = Index.wrapping_add(1);
        }
    } else if BytesPerSample == 2 {
        let Res: PUSHORT = ResultOut as PUSHORT;

        src_float_to_short_array(FloatOut as *const f32, Res as *mut ::core::ffi::c_short, (Data.output_frames_gen as ULONG).wrapping_mul(NumChannels) as ::core::ffi::c_int);
    } else if BytesPerSample == 4 {
        let Res: PULONG = ResultOut as PULONG;

        src_float_to_int_array(FloatOut as *const f32, Res as *mut ::core::ffi::c_int, (Data.output_frames_gen as ULONG).wrapping_mul(NumChannels) as ::core::ffi::c_int);
    }

    *Result = ResultOut as PVOID;
    *ResultLength = (Data.output_frames_gen as ULONG).wrapping_mul(BytesPerSample).wrapping_mul(NumChannels);
    HeapFree(GetProcessHeap(), 0, FloatIn as PVOID);
    HeapFree(GetProcessHeap(), 0, FloatOut as PVOID);
    src_delete(State);
    return ERROR_SUCCESS;
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
) -> DWORD {
    let mut Samples: ULONG;
    let mut NewIndex: ULONG;
    let mut OldIndex: ULONG;

    Samples = BufferLength
        .wrapping_div(BitsPerSample.wrapping_div(8))
        .wrapping_div(OldChannels);

    SND_TRACE!(RUSTCONV_WSTR_D1041FF85E08.as_ptr(), OldChannels, NewChannels);

    if NewChannels > OldChannels {
        if BitsPerSample == 8 {
            let BufferOut: PUCHAR = HeapAlloc(
                GetProcessHeap(),
                0,
                Samples.wrapping_mul(NewChannels) as SIZE_T,
            ) as PUCHAR;
            if BufferOut.is_null() {
                return ERROR_NOT_ENOUGH_MEMORY as DWORD;
            }

            NewIndex = 0;
            OldIndex = 0;
            while OldIndex < Samples.wrapping_mul(OldChannels) {
                let mut SubIndex: ULONG = 0;

                core::ptr::copy(
                    Buffer.offset(OldIndex as isize) as *const UCHAR as *const ::core::ffi::c_void,
                    BufferOut.offset(NewIndex as isize) as *mut UCHAR as *mut ::core::ffi::c_void,
                    (OldChannels as usize).wrapping_mul(core::mem::size_of::<UCHAR>() as usize),
                );

                loop {
                    /* 2 channel stretched to 4 looks like LRLR */
                    *BufferOut.offset(
                        NewIndex.wrapping_add(OldChannels).wrapping_add(SubIndex) as isize,
                    ) = *Buffer
                        .offset(OldIndex.wrapping_add(SubIndex.wrapping_rem(OldChannels)) as isize);
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
            let BufferOut: PUSHORT = HeapAlloc(
                GetProcessHeap(),
                0,
                Samples.wrapping_mul(NewChannels) as SIZE_T,
            ) as PUSHORT;
            if BufferOut.is_null() {
                return ERROR_NOT_ENOUGH_MEMORY as DWORD;
            }

            NewIndex = 0;
            OldIndex = 0;
            while OldIndex < Samples.wrapping_mul(OldChannels) {
                let mut SubIndex: ULONG = 0;

                core::ptr::copy(
                    Buffer.offset(OldIndex as isize) as *const UCHAR as *const ::core::ffi::c_void,
                    BufferOut.offset(NewIndex as isize) as *mut USHORT as *mut ::core::ffi::c_void,
                    (OldChannels as usize).wrapping_mul(core::mem::size_of::<USHORT>() as usize),
                );

                loop {
                    *BufferOut.offset(
                        NewIndex.wrapping_add(OldChannels).wrapping_add(SubIndex) as isize,
                    ) = *Buffer
                        .offset(OldIndex.wrapping_add(SubIndex.wrapping_rem(OldChannels)) as isize)
                        as USHORT;
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
            let BufferOut: PUCHAR = HeapAlloc(
                GetProcessHeap(),
                0,
                Samples.wrapping_mul(NewChannels) as SIZE_T,
            ) as PUCHAR;
            if BufferOut.is_null() {
                return ERROR_NOT_ENOUGH_MEMORY as DWORD;
            }

            NewIndex = 0;
            OldIndex = 0;
            while OldIndex < Samples.wrapping_mul(OldChannels) {
                let mut SubIndex: ULONG = 0;

                core::ptr::copy(
                    Buffer.offset(OldIndex as isize) as *const UCHAR as *const ::core::ffi::c_void,
                    BufferOut.offset(NewIndex as isize) as *mut UCHAR as *mut ::core::ffi::c_void,
                    (OldChannels as usize).wrapping_mul(3 as usize),
                );

                loop {
                    core::ptr::copy(
                        Buffer.offset(
                            OldIndex
                                .wrapping_add(SubIndex.wrapping_rem(OldChannels))
                                .wrapping_mul(3) as isize,
                        ) as *const UCHAR as *const ::core::ffi::c_void,
                        BufferOut.offset(
                            NewIndex
                                .wrapping_add(OldChannels)
                                .wrapping_add(SubIndex)
                                .wrapping_mul(3) as isize,
                        ) as *mut UCHAR as *mut ::core::ffi::c_void,
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
            let BufferOut: PULONG = HeapAlloc(
                GetProcessHeap(),
                0,
                Samples.wrapping_mul(NewChannels) as SIZE_T,
            ) as PULONG;
            if BufferOut.is_null() {
                return ERROR_NOT_ENOUGH_MEMORY as DWORD;
            }

            NewIndex = 0;
            OldIndex = 0;
            while OldIndex < Samples.wrapping_mul(OldChannels) {
                let mut SubIndex: ULONG = 0;

                core::ptr::copy(
                    Buffer.offset(OldIndex as isize) as *const UCHAR as *const ::core::ffi::c_void,
                    BufferOut.offset(NewIndex as isize) as *mut ULONG as *mut ::core::ffi::c_void,
                    (OldChannels as usize).wrapping_mul(::core::mem::size_of::<ULONG>()),
                );

                loop {
                    *BufferOut.offset(
                        NewIndex.wrapping_add(OldChannels).wrapping_add(SubIndex) as isize,
                    ) = *Buffer
                        .offset(OldIndex.wrapping_add(SubIndex.wrapping_rem(OldChannels)) as isize)
                        as ULONG;
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
        let BufferOut: PUSHORT = HeapAlloc(
            GetProcessHeap(),
            0,
            Samples.wrapping_mul(NewChannels) as SIZE_T,
        ) as PUSHORT;
        if BufferOut.is_null() {
            return ERROR_NOT_ENOUGH_MEMORY as DWORD;
        }

        NewIndex = 0;
        OldIndex = 0;
        while OldIndex < Samples.wrapping_mul(OldChannels) {
            /* TODO
             * mix stream instead of just dumping part of it ;)
             */
            core::ptr::copy(
                Buffer.offset(OldIndex as isize) as *const UCHAR as *const ::core::ffi::c_void,
                BufferOut.offset(NewIndex as isize) as *mut USHORT as *mut ::core::ffi::c_void,
                (NewChannels.wrapping_mul(BitsPerSample.wrapping_div(8)))
                    as usize,
            );

            NewIndex = NewIndex.wrapping_add(NewChannels);
            OldIndex = OldIndex.wrapping_add(OldChannels);
        }

        *Result = BufferOut as PVOID;
        *ResultLength = Samples.wrapping_mul(NewChannels);
    }
    return ERROR_SUCCESS as DWORD;
}
#[no_mangle]
pub unsafe extern "C" fn PerformQualityConversion(
    Buffer: PUCHAR,
    BufferLength: ULONG,
    OldWidth: ULONG,
    NewWidth: ULONG,
    Result: *mut PVOID,
    ResultLength: PULONG,
) -> DWORD {
    let mut Samples: ULONG;
    let mut Index: ULONG;

    ASSERT!(OldWidth != NewWidth);

    Samples = BufferLength.wrapping_div(OldWidth.wrapping_div(8));
    //DPRINT("Samples %u BufferLength %u\n", Samples, BufferLength);

    //SND_TRACE!(L"PerformQualityConversion OldWidth %u NewWidth %u\n", OldWidth, NewWidth);

    if OldWidth == 8 && NewWidth == 16 {
        let mut Sample: USHORT = 0;
        let mut BufferOut: PUSHORT = HeapAlloc(
            GetProcessHeap(),
            0,
            (Samples as SIZE_T).wrapping_mul(core::mem::size_of::<USHORT>() as SIZE_T),
        ) as PUSHORT;
        if BufferOut.is_null() {
            return ERROR_NOT_ENOUGH_MEMORY;
        }

        Index = 0;
        while Index < Samples {
            Sample = *Buffer.offset(Index as isize) as USHORT;
            Sample = (Sample as ::core::ffi::c_int * 2) as USHORT;
            if cfg!(target_arch = "x86") {
                Sample = _byteswap_ushort(Sample);
            }
            *BufferOut.offset(Index as isize) = Sample;
            Index = Index.wrapping_add(1);
        }
        *Result = BufferOut as PVOID;
        *ResultLength = (Samples as usize)
            .wrapping_mul(core::mem::size_of::<USHORT>() as usize) as ULONG;
    } else if OldWidth == 8 && NewWidth == 32 {
        let mut Sample: ULONG = 0;
        let mut BufferOut: PULONG = HeapAlloc(
            GetProcessHeap(),
            0,
            (Samples as SIZE_T).wrapping_mul(core::mem::size_of::<ULONG>() as SIZE_T),
        ) as PULONG;
        if BufferOut.is_null() {
            return ERROR_NOT_ENOUGH_MEMORY;
        }

        Index = 0;
        while Index < Samples {
            Sample = *Buffer.offset(Index as isize) as ULONG;
            Sample = Sample.wrapping_mul(16777216);
            if cfg!(target_arch = "x86") {
                Sample = _byteswap_ulong(Sample);
            }
            *BufferOut.offset(Index as isize) = Sample;
            Index = Index.wrapping_add(1);
        }
        *Result = BufferOut as PVOID;
        *ResultLength =
            (Samples as usize).wrapping_mul(core::mem::size_of::<ULONG>() as usize) as ULONG;
    } else if OldWidth == 16 && NewWidth == 32 {
        let mut Sample: ULONG = 0;
        let mut BufferIn: PUSHORT = Buffer as PUSHORT;
        let mut BufferOut: PULONG = HeapAlloc(
            GetProcessHeap(),
            0,
            (Samples as SIZE_T).wrapping_mul(core::mem::size_of::<ULONG>() as SIZE_T),
        ) as PULONG;
        if BufferOut.is_null() {
            return ERROR_NOT_ENOUGH_MEMORY;
        }

        Index = 0;
        while Index < Samples {
            Sample = *BufferIn.offset(Index as isize) as ULONG;
            Sample = Sample.wrapping_mul(65536);
            if cfg!(target_arch = "x86") {
                Sample = _byteswap_ulong(Sample);
            }
            *BufferOut.offset(Index as isize) = Sample;
            Index = Index.wrapping_add(1);
        }
        *Result = BufferOut as PVOID;
        *ResultLength =
            (Samples as usize).wrapping_mul(core::mem::size_of::<ULONG>() as usize) as ULONG;
    } else if OldWidth == 16 && NewWidth == 8 {
        let mut Sample: USHORT = 0;
        let mut BufferIn: PUSHORT = Buffer as PUSHORT;
        let mut BufferOut: PUCHAR = HeapAlloc(
            GetProcessHeap(),
            0,
            (Samples as SIZE_T).wrapping_mul(core::mem::size_of::<UCHAR>() as SIZE_T),
        ) as PUCHAR;
        if BufferOut.is_null() {
            return ERROR_NOT_ENOUGH_MEMORY;
        }

        Index = 0;
        while Index < Samples {
            Sample = *BufferIn.offset(Index as isize);
            if cfg!(target_arch = "x86") {
                Sample = _byteswap_ushort(Sample);
            }
            Sample = (Sample as ::core::ffi::c_int / 256) as USHORT;
            *BufferOut.offset(Index as isize) =
                (Sample as ::core::ffi::c_int & 0xFF) as UCHAR;
            Index = Index.wrapping_add(1);
        }
        *Result = BufferOut as PVOID;
        *ResultLength = (Samples as usize)
            .wrapping_mul(core::mem::size_of::<UCHAR>() as usize) as ULONG;
    } else if OldWidth == 32 && NewWidth == 8 {
        let mut Sample: ULONG = 0;
        let mut BufferIn: PULONG = Buffer as PULONG;
        let mut BufferOut: PUCHAR = HeapAlloc(
            GetProcessHeap(),
            0,
            (Samples as SIZE_T).wrapping_mul(core::mem::size_of::<UCHAR>() as SIZE_T),
        ) as PUCHAR;
        if BufferOut.is_null() {
            return ERROR_NOT_ENOUGH_MEMORY;
        }

        Index = 0;
        while Index < Samples {
            Sample = *BufferIn.offset(Index as isize);
            if cfg!(target_arch = "x86") {
                Sample = _byteswap_ulong(Sample);
            }
            Sample = Sample.wrapping_div(16777216);
            *BufferOut.offset(Index as isize) =
                (Sample & 0xFF) as UCHAR;
            Index = Index.wrapping_add(1);
        }
        *Result = BufferOut as PVOID;
        *ResultLength = (Samples as usize)
            .wrapping_mul(core::mem::size_of::<UCHAR>() as usize) as ULONG;
    } else if OldWidth == 32 && NewWidth == 16 {
        let mut Sample: USHORT = 0;
        let mut BufferIn: PULONG = Buffer as PULONG;
        let mut BufferOut: PUSHORT = HeapAlloc(
            GetProcessHeap(),
            0,
            (Samples as SIZE_T).wrapping_mul(core::mem::size_of::<USHORT>() as SIZE_T),
        ) as PUSHORT;
        if BufferOut.is_null() {
            return ERROR_NOT_ENOUGH_MEMORY;
        }

        Index = 0;
        while Index < Samples {
            // Divergence from mixer.c: it stores the 32-bit sample in its USHORT `Sample`
            // first, so `Sample /= 65536` is always 0 and this branch emits silence. Keep the
            // full ULONG and take its high half, as the scaling in the other branches intends.
            let mut Wide: ULONG = *BufferIn.offset(Index as isize);
            if cfg!(target_arch = "x86") {
                Wide = _byteswap_ulong(Wide);
            }
            Wide /= 65536;
            Sample = (Wide & 0xFFFF) as USHORT;
            *BufferOut.offset(Index as isize) = Sample;
            Index = Index.wrapping_add(1);
        }
        *Result = BufferOut as PVOID;
        *ResultLength = (Samples as usize)
            .wrapping_mul(core::mem::size_of::<USHORT>() as usize) as ULONG;
    } else {
        DPRINT1!(
            "Not implemented conversion OldWidth %u NewWidth %u\n",
            OldWidth,
            NewWidth
        );
        return ERROR_NOT_SUPPORTED;
    }

    return ERROR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn MixerCompletionRoutine(
    mut dwErrorCode: DWORD,
    mut dwNumberOfBytesTransferred: DWORD,
    mut lpOverlapped: LPOVERLAPPED,
) {
    let mut Overlap: PSOUND_OVERLAPPED = lpOverlapped as PSOUND_OVERLAPPED;
    (*Overlap)
        .OriginalCompletionRoutine
        .unwrap_unchecked()(
        dwErrorCode,
        PtrToUlong!((*Overlap).CompletionContext as *const ::core::ffi::c_void) as DWORD,
        lpOverlapped,
    );
}
#[no_mangle]
pub unsafe extern "C" fn WriteFileEx_Remixer(
    SoundDeviceInstance: PSOUND_DEVICE_INSTANCE,
    OffsetPtr: PVOID,
    Length: DWORD,
    Overlap: PSOUND_OVERLAPPED,
    CompletionRoutine: LPOVERLAPPED_COMPLETION_ROUTINE,
) -> MMRESULT {
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut DeviceInfo: WDMAUD_DEVICE_INFO = core::mem::zeroed();
    let mut BufferLength: DWORD = 0;
    let mut BufferLengthTemp: DWORD = 0;
    let mut BufferOut: PVOID = core::ptr::null_mut();
    let mut BufferOutTemp: PVOID = core::ptr::null_mut();
    let mut Status: DWORD = 0;
    let mut Result: BOOL = 0;

    VALIDATE_MMSYS_PARAMETER!(SoundDeviceInstance);
    VALIDATE_MMSYS_PARAMETER!(OffsetPtr);
    VALIDATE_MMSYS_PARAMETER!(Overlap);
    VALIDATE_MMSYS_PARAMETER!(CompletionRoutine);

    GetSoundDeviceInstanceHandle(SoundDeviceInstance, core::ptr::addr_of_mut!(Handle));

    SND_ASSERT!(Handle);

    BufferOut = OffsetPtr;
    BufferLength = Length;

    if (*SoundDeviceInstance).WaveFormatEx.wBitsPerSample != 16
    {
        Status = PerformQualityConversion(OffsetPtr as PUCHAR,
                                          Length,
                                          (*SoundDeviceInstance).WaveFormatEx.wBitsPerSample as ULONG,
                                          16,
                                          core::ptr::addr_of_mut!(BufferOut),
                                          core::ptr::addr_of_mut!(BufferLength));
        if Status != 0
        {
            SND_TRACE!(RUSTCONV_WSTR_113E6451A8DB.as_ptr());
            return MMSYSERR_NOERROR;
        }
    }

    if (*SoundDeviceInstance).WaveFormatEx.nChannels != 2
    {
        Status = PerformChannelConversion(BufferOut as PUCHAR,
                                          BufferLength,
                                          (*SoundDeviceInstance).WaveFormatEx.nChannels as ULONG,
                                          2,
                                          16,
                                          core::ptr::addr_of_mut!(BufferOutTemp),
                                          core::ptr::addr_of_mut!(BufferLengthTemp));

        if BufferOut != OffsetPtr
        {
            HeapFree(GetProcessHeap(), 0, BufferOut);
        }

        if Status != 0
        {
            SND_TRACE!(RUSTCONV_WSTR_4683ED71C250.as_ptr());
            return MMSYSERR_NOERROR;
        }

        BufferOut = BufferOutTemp;
        BufferLength = BufferLengthTemp;
    }

    if (*SoundDeviceInstance).WaveFormatEx.nSamplesPerSec != 44100
    {
        Status = PerformSampleRateConversion(BufferOut as PUCHAR,
                                             BufferLength,
                                             (*SoundDeviceInstance).WaveFormatEx.nSamplesPerSec as ULONG,
                                             44100,
                                             2,
                                             2,
                                             core::ptr::addr_of_mut!(BufferOutTemp),
                                             core::ptr::addr_of_mut!(BufferLengthTemp));

        if BufferOut != OffsetPtr
        {
            HeapFree(GetProcessHeap(), 0, BufferOut);
        }

        if Status != 0
        {
            SND_TRACE!(RUSTCONV_WSTR_201283C65888.as_ptr());
            return MMSYSERR_NOERROR;
        }

        BufferOut = BufferOutTemp;
        BufferLength = BufferLengthTemp;
    }

    ZeroMemory!(core::ptr::addr_of_mut!(DeviceInfo) as *mut core::ffi::c_void, core::mem::size_of::<WDMAUD_DEVICE_INFO>() as usize);
    DeviceInfo.hDevice = Handle;
    DeviceInfo.DeviceType = WAVE_OUT_DEVICE_TYPE; //FIXME
    DeviceInfo.Header.FrameExtent = BufferLength;
    DeviceInfo.Header.DataUsed = BufferLength;
    DeviceInfo.Header.Data = BufferOut;
    DeviceInfo.Header.Size = core::mem::size_of::<KSSTREAM_HEADER>() as ULONG;
    DeviceInfo.Header.PresentationTime.Numerator = 1;
    DeviceInfo.Header.PresentationTime.Denominator = 1;

    (*Overlap).CompletionContext = UlongToPtr!(Length);
    (*Overlap).OriginalCompletionRoutine = CompletionRoutine;

    (*Overlap).Standard.hEvent = CreateEventW(core::ptr::null_mut(), (FALSE) as BOOL, (FALSE) as BOOL, core::ptr::null_mut());

    //SND_TRACE!(L"OriginalLength %u NewLength %u\n", Length, BufferLength);

    Result = WriteFileEx(KernelHandle, core::ptr::addr_of_mut!(DeviceInfo) as LPCVOID, core::mem::size_of::<WDMAUD_DEVICE_INFO>() as DWORD, Overlap as LPOVERLAPPED, Some(MixerCompletionRoutine));

    if Result == 0
    {
        SND_TRACE!(RUSTCONV_WSTR_00FB6900C11A.as_ptr(), GetLastError());
        return MMSYSERR_NOERROR;
    }

    WaitForSingleObjectEx(KernelHandle, INFINITE, (TRUE) as BOOL);

    // #ifdef USERMODE_MIXER
    //    // if (BufferOut != OffsetPtr)
    //    //     HeapFree(GetProcessHeap(), 0, BufferOut);
    // #endif

    return MMSYSERR_NOERROR;
}
