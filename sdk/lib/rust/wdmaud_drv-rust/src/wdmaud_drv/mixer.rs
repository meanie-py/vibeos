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
/// Divergence from mixer.c (the same code as kmixer's pin.c), which is wrong for every
/// width but 8 bits: it allocates `Samples * NewChannels` BYTES whatever the sample
/// width, indexes the input as bytes (`Buffer[OldIndex]` is a PUCHAR) while the output
/// is USHORT/ULONG-indexed, and its
/// `do { } while (SubIndex++ < NewChannels - OldChannels)` writes one channel more than
/// it fills -- the first slot of the next frame, which on the last frame is past the end.
/// So a 16-bit mono stream reaching WriteFileEx_Remixer overran the heap and produced
/// garbage. Its down-mix writes `NewChannels * BitsPerSample/8` bytes at a USHORT stride
/// from an unscaled byte offset, and it reported `*ResultLength` in samples, not bytes.
/// An unhandled width returned ERROR_SUCCESS with `*Result` never written.
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
) -> DWORD {
    let Width = match BitsPerSample {
        8 | 16 | 24 | 32 => (BitsPerSample / 8) as usize,
        _ => {
            DPRINT1!("Not implemented channel conversion BitsPerSample %u\n", BitsPerSample);
            return ERROR_NOT_SUPPORTED;
        }
    };
    if OldChannels == 0 || NewChannels == 0 {
        return ERROR_INVALID_PARAMETER;
    }
    let Samples = (BufferLength as usize) / Width / (OldChannels as usize);
    let OldFrame = (OldChannels as usize) * Width;
    let NewFrame = (NewChannels as usize) * Width;
    let Size = match Samples.checked_mul(NewFrame) {
        Some(Size) if Size <= ULONG::MAX as usize => Size,
        _ => return ERROR_NOT_ENOUGH_MEMORY,
    };

    let BufferOut: PUCHAR = HeapAlloc(GetProcessHeap(), 0, Size as SIZE_T) as PUCHAR;
    if BufferOut.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY;
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
    return ERROR_SUCCESS;
}
/// Divergence from mixer.c on every pair but 16 -> 32. 8-bit PCM is unsigned with silence
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
) -> DWORD {
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
            return ERROR_NOT_SUPPORTED;
        }
    };
    let Samples = (BufferLength as usize) / InBytes;
    let Size = match Samples.checked_mul(OutBytes) {
        Some(Size) if Size <= ULONG::MAX as usize => Size,
        _ => return ERROR_NOT_ENOUGH_MEMORY,
    };

    let BufferOut: PUCHAR = HeapAlloc(GetProcessHeap(), 0, Size as SIZE_T) as PUCHAR;
    if BufferOut.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY;
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
