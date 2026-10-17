use crate::prelude::*;
use ntkrnl_sys::ddk::{ACCESS_MASK, GUID, LPWSTR, PHANDLE};
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

#[no_mangle]
pub static mut KSDATAFORMAT_SPECIFIER_WAVEFORMATEX: GUID = _GUID {
    Data1: 0x5589f81 as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0xc356 as ::core::ffi::c_ushort,
    Data3: 0x11ce as ::core::ffi::c_ushort,
    Data4: [
        0xbf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x55 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x59 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5a as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSDATAFORMAT_SUBTYPE_PCM: GUID = _GUID {
    Data1: 0x1 as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0x10 as ::core::ffi::c_ushort,
    Data4: [
        0x80 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x38 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x71 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSDATAFORMAT_TYPE_AUDIO: GUID = _GUID {
    Data1: 0x73647561 as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0x10 as ::core::ffi::c_ushort,
    Data4: [
        0x80 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x38 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x71 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSINTERFACESETID_Standard: GUID = _GUID {
    Data1: 0x1a8766a0 as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0x62ce as ::core::ffi::c_ushort,
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

#[no_mangle]
pub static mut KSMEDIUMSETID_Standard: GUID = _GUID {
    Data1: 0x4747b320 as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0x62ce as ::core::ffi::c_ushort,
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

static mut TestRange: [AUDIO_RANGE; 5] = [
    AUDIO_RANGE {
        SampleRate: 11025 as ULONG,
        Bit8Mono: WAVE_FORMAT_1M08 as ULONG,
        Bit8Stereo: WAVE_FORMAT_1S08 as ULONG,
        Bit16Mono: WAVE_FORMAT_1M16 as ULONG,
        Bit16Stereo: WAVE_FORMAT_1S16 as ULONG,
    },
    AUDIO_RANGE {
        SampleRate: 22050 as ULONG,
        Bit8Mono: WAVE_FORMAT_2M08 as ULONG,
        Bit8Stereo: WAVE_FORMAT_2S08 as ULONG,
        Bit16Mono: WAVE_FORMAT_2M16 as ULONG,
        Bit16Stereo: WAVE_FORMAT_2S16 as ULONG,
    },
    AUDIO_RANGE {
        SampleRate: 44100 as ULONG,
        Bit8Mono: WAVE_FORMAT_4M08 as ULONG,
        Bit8Stereo: WAVE_FORMAT_4S08 as ULONG,
        Bit16Mono: WAVE_FORMAT_4M16 as ULONG,
        Bit16Stereo: WAVE_FORMAT_4S16 as ULONG,
    },
    AUDIO_RANGE {
        SampleRate: 48000 as ULONG,
        Bit8Mono: WAVE_FORMAT_48M08 as ULONG,
        Bit8Stereo: WAVE_FORMAT_48S08 as ULONG,
        Bit16Mono: WAVE_FORMAT_48M16 as ULONG,
        Bit16Stereo: WAVE_FORMAT_48S16 as ULONG,
    },
    AUDIO_RANGE {
        SampleRate: 96000 as ULONG,
        Bit8Mono: WAVE_FORMAT_96M08 as ULONG,
        Bit8Stereo: WAVE_FORMAT_96S08 as ULONG,
        Bit16Mono: WAVE_FORMAT_96M16 as ULONG,
        Bit16Stereo: WAVE_FORMAT_96S16 as ULONG,
    },
];

#[repr(C)]
pub struct AUDIO_RANGE {
    pub SampleRate: ULONG,
    pub Bit8Mono: ULONG,
    pub Bit8Stereo: ULONG,
    pub Bit16Mono: ULONG,
    pub Bit16Stereo: ULONG,
}

pub const AUDIO_TEST_RANGE: usize = 5;

// WAVE_FORMAT_* values inlined (mmreg.h):
#[no_mangle]
pub unsafe extern "C" fn MMixerAllocatePinConnect(
    MixerContext: PMIXER_CONTEXT,
    DataFormatSize: ULONG,
) -> PKSPIN_CONNECT {
    return (*MixerContext).Alloc.unwrap_unchecked()(
        (core::mem::size_of::<KSPIN_CONNECT>() as usize).wrapping_add(DataFormatSize as usize)
            as ULONG,
    ) as PKSPIN_CONNECT;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetWaveInfoByIndexAndType(
    mut MixerList: PMIXER_LIST,
    mut DeviceIndex: ULONG,
    mut bWaveInType: ULONG,
    mut OutWaveInfo: *mut LPWAVE_INFO,
) -> MIXER_STATUS {
    let mut Index: ULONG = 0;
    let mut Entry: PLIST_ENTRY = ::core::ptr::null_mut::<_LIST_ENTRY>();
    let mut ListHead: PLIST_ENTRY = ::core::ptr::null_mut::<_LIST_ENTRY>();
    let mut WaveInfo: LPWAVE_INFO = ::core::ptr::null_mut::<WAVE_INFO>();
    if bWaveInType != 0 {
        ListHead = &raw mut (*MixerList).WaveInList as PLIST_ENTRY;
    } else {
        ListHead = &raw mut (*MixerList).WaveOutList as PLIST_ENTRY;
    }
    Entry = (*ListHead).Flink as PLIST_ENTRY;
    while Entry != ListHead {
        WaveInfo = (Entry as ULONG_PTR)
            .wrapping_sub(core::mem::offset_of!(WAVE_INFO, Entry) as ULONG_PTR)
            as *mut WAVE_INFO as LPWAVE_INFO;
        if Index == DeviceIndex {
            *OutWaveInfo = WaveInfo;
            return MM_STATUS_SUCCESS;
        }
        Index = Index.wrapping_add(1);
        Entry = (*Entry).Flink as PLIST_ENTRY;
    }
    return MM_STATUS_INVALID_PARAMETER;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerInitializeDataFormat(
    DataFormat: PKSDATAFORMAT_WAVEFORMATEX,
    WaveFormatEx: LPWAVEFORMATEX,
    cbSize: DWORD,
) {
    (*DataFormat).WaveFormatEx.wFormatTag = (*WaveFormatEx).wFormatTag;
    (*DataFormat).WaveFormatEx.nChannels = (*WaveFormatEx).nChannels;
    (*DataFormat).WaveFormatEx.nSamplesPerSec = (*WaveFormatEx).nSamplesPerSec;
    (*DataFormat).WaveFormatEx.nBlockAlign = (*WaveFormatEx).nBlockAlign;
    (*DataFormat).WaveFormatEx.nAvgBytesPerSec = (*WaveFormatEx).nAvgBytesPerSec;
    (*DataFormat).WaveFormatEx.wBitsPerSample = (*WaveFormatEx).wBitsPerSample;
    (*DataFormat).WaveFormatEx.cbSize = cbSize as WORD;
    (*DataFormat).DataFormat.c2rust_unnamed.FormatSize = (core::mem::size_of::<KSDATAFORMAT>() as usize)
        .wrapping_add(core::mem::size_of::<WAVEFORMATEX>() as usize)
        .wrapping_add(cbSize as usize) as ULONG;
    (*DataFormat).DataFormat.c2rust_unnamed.Flags = 0;
    (*DataFormat).DataFormat.c2rust_unnamed.Reserved = 0;
    (*DataFormat).DataFormat.c2rust_unnamed.MajorFormat = KSDATAFORMAT_TYPE_AUDIO;
    (*DataFormat).DataFormat.c2rust_unnamed.SubFormat = KSDATAFORMAT_SUBTYPE_PCM;
    (*DataFormat).DataFormat.c2rust_unnamed.Specifier = KSDATAFORMAT_SPECIFIER_WAVEFORMATEX;
    (*DataFormat).DataFormat.c2rust_unnamed.SampleSize = 4;

    /* Write additional fields for Extensible audio format */
    if (*WaveFormatEx).wFormatTag == (WAVE_FORMAT_EXTENSIBLE as u16) {
        let WaveFormatExt: PWAVEFORMATEXTENSIBLE =
            core::ptr::addr_of_mut!((*DataFormat).WaveFormatEx) as PWAVEFORMATEXTENSIBLE;
        (*WaveFormatExt).SubFormat = KSDATAFORMAT_SUBTYPE_PCM;
        (*WaveFormatExt).Samples.wValidBitsPerSample = (*WaveFormatEx).wBitsPerSample;
        if (*WaveFormatEx).nChannels == 0 {
            (*WaveFormatExt).dwChannelMask = KSAUDIO_SPEAKER_DIRECTOUT;
        } else if (*WaveFormatEx).nChannels == 1 {
            (*WaveFormatExt).dwChannelMask = KSAUDIO_SPEAKER_MONO;
        } else if (*WaveFormatEx).nChannels == 2 {
            (*WaveFormatExt).dwChannelMask = KSAUDIO_SPEAKER_STEREO;
        } else if (*WaveFormatEx).nChannels == 4 {
            (*WaveFormatExt).dwChannelMask = KSAUDIO_SPEAKER_QUAD;
        } else if (*WaveFormatEx).nChannels == 5 {
            (*WaveFormatExt).dwChannelMask = KSAUDIO_SPEAKER_5POINT1;
        } else if (*WaveFormatEx).nChannels == 7 {
            (*WaveFormatExt).dwChannelMask = KSAUDIO_SPEAKER_7POINT1;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetAudioPinDataRanges(
    mut MixerContext: PMIXER_CONTEXT,
    mut hDevice: HANDLE,
    mut PinId: ULONG,
    mut OutMultipleItem: *mut PKSMULTIPLE_ITEM,
) -> MIXER_STATUS {
    let mut PinProperty: KSP_PIN = core::mem::zeroed();
    let mut BytesReturned: ULONG = 0;
    let mut Status: MIXER_STATUS = core::mem::zeroed();
    let mut MultipleItem: PKSMULTIPLE_ITEM = core::ptr::null_mut();

    /* retrieve size of data ranges buffer */
    PinProperty.Reserved = 0;
    PinProperty.PinId = PinId;
    PinProperty.Property.Set = KSPROPSETID_Pin;
    PinProperty.Property.Id = KSPROPERTY_PIN_DATARANGES;
    PinProperty.Property.Flags = KSPROPERTY_TYPE_GET;

    Status = (*MixerContext).Control.unwrap_unchecked()(
        hDevice,
        IOCTL_KS_PROPERTY,
        core::ptr::addr_of_mut!(PinProperty) as PVOID,
        core::mem::size_of::<KSP_PIN>() as ULONG,
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(BytesReturned),
    );
    if Status != MM_STATUS_MORE_ENTRIES {
        return Status;
    }

    MultipleItem = (*MixerContext).Alloc.unwrap_unchecked()(BytesReturned)
        as PKSMULTIPLE_ITEM;
    if MultipleItem.is_null() {
        /* not enough memory */
        return MM_STATUS_NO_MEMORY;
    }

    Status = (*MixerContext).Control.unwrap_unchecked()(
        hDevice,
        IOCTL_KS_PROPERTY,
        core::ptr::addr_of_mut!(PinProperty) as PVOID,
        core::mem::size_of::<KSP_PIN>() as ULONG,
        MultipleItem as PVOID,
        BytesReturned,
        core::ptr::addr_of_mut!(BytesReturned),
    );
    if Status != MM_STATUS_SUCCESS {
        /* failed */
        (*MixerContext).Free.unwrap_unchecked()(MultipleItem as PVOID);
        return Status;
    }

    /* save result */
    *OutMultipleItem = MultipleItem;
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerFindAudioDataRange(
    mut MultipleItem: PKSMULTIPLE_ITEM,
    mut OutDataRangeAudio: *mut PKSDATARANGE_AUDIO,
) -> MIXER_STATUS {
    let mut Index: ULONG;
    let mut DataRangeAudio: PKSDATARANGE_AUDIO;
    let mut DataRange: PKSDATARANGE;

    DataRange = (MultipleItem.offset(1 as isize)) as PKSDATARANGE;
    Index = 0 as ULONG;
    while Index < (*MultipleItem).Count {
        if (*DataRange).c2rust_unnamed.FormatSize as usize == (core::mem::size_of::<KSDATARANGE_AUDIO>()) as usize {
            DataRangeAudio = DataRange as PKSDATARANGE_AUDIO;
            if IsEqualGUIDAligned(
                core::ptr::addr_of_mut!((*DataRangeAudio).DataRange.c2rust_unnamed.MajorFormat),
                core::ptr::addr_of!(KSDATAFORMAT_TYPE_AUDIO),
            ) != false
                && IsEqualGUIDAligned(
                    core::ptr::addr_of_mut!((*DataRangeAudio).DataRange.c2rust_unnamed.SubFormat),
                    core::ptr::addr_of!(KSDATAFORMAT_SUBTYPE_PCM),
                ) != false
                && IsEqualGUIDAligned(
                    core::ptr::addr_of_mut!((*DataRangeAudio).DataRange.c2rust_unnamed.Specifier),
                    core::ptr::addr_of!(KSDATAFORMAT_SPECIFIER_WAVEFORMATEX),
                ) != false
            {
                DPRINT!(
                    "Min Sample %u Max Sample %u Min Bits %u Max Bits %u Max Channel %u\n",
                    (*DataRangeAudio).MinimumSampleFrequency,
                    (*DataRangeAudio).MaximumSampleFrequency,
                    (*DataRangeAudio).MinimumBitsPerSample,
                    (*DataRangeAudio).MaximumBitsPerSample,
                    (*DataRangeAudio).MaximumChannels
                );
                *OutDataRangeAudio = DataRangeAudio;
                return MM_STATUS_SUCCESS;
            }
        }
        DataRange = (DataRange as ULONG_PTR)
            .wrapping_add((*DataRange).c2rust_unnamed.FormatSize as ULONG_PTR) as PKSDATARANGE;
        Index = Index.wrapping_add(1);
    }
    return MM_STATUS_UNSUCCESSFUL;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerOpenWavePin(
    MixerContext: PMIXER_CONTEXT,
    MixerList: PMIXER_LIST,
    DeviceId: ULONG,
    PinId: ULONG,
    WaveFormatEx: LPWAVEFORMATEX,
    DesiredAccess: ACCESS_MASK,
    CreateCallback: PIN_CREATE_CALLBACK,
    Context: PVOID,
    PinHandle: PHANDLE,
) -> MIXER_STATUS {
    let mut PinConnect: PKSPIN_CONNECT = core::ptr::null_mut();
    let mut DataFormat: PKSDATAFORMAT_WAVEFORMATEX = core::ptr::null_mut();
    let mut MixerData: LPMIXER_DATA = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;
    let mut MixerStatus: MIXER_STATUS = 0;
    let mut cbSize: DWORD = 0;

    MixerData = MMixerGetDataByDeviceId(MixerList, DeviceId);
    if MixerData.is_null() {
        return MM_STATUS_INVALID_PARAMETER;
    }

    /* Enforce 0 for WAVE_FORMAT_PCM, which ignores extra information size */
    cbSize = (if (*WaveFormatEx).wFormatTag as u32 == WAVE_FORMAT_PCM {
        0
    } else {
        (*WaveFormatEx).cbSize as u32
    }) as DWORD;

    /* allocate pin connect */
    PinConnect = MMixerAllocatePinConnect(
        MixerContext,
        (core::mem::size_of::<KSDATAFORMAT_WAVEFORMATEX>() as ULONG).wrapping_add(cbSize as ULONG),
    );
    if PinConnect.is_null() {
        /* no memory */
        return MM_STATUS_NO_MEMORY;
    }

    /* initialize pin connect struct */
    MMixerInitializePinConnect(PinConnect, PinId);

    /* get offset to dataformat */
    DataFormat = PinConnect.offset(1 as isize) as PKSDATAFORMAT_WAVEFORMATEX;
    /* initialize with requested wave format */
    MMixerInitializeDataFormat(DataFormat, WaveFormatEx, cbSize);

    if CreateCallback.is_some() {
        /* let the callback handle the creation */
        MixerStatus = (CreateCallback.unwrap_unchecked())(
            Context,
            DeviceId,
            PinId,
            (*MixerData).hDevice,
            PinConnect,
            DesiredAccess,
            PinHandle,
        );
    } else {
        /* now create the pin */
        Status = KsCreatePin(
            (*MixerData).hDevice,
            PinConnect,
            DesiredAccess,
            PinHandle,
        );

        /* normalize status */
        if Status == STATUS_SUCCESS {
            MixerStatus = MM_STATUS_SUCCESS;
        } else {
            MixerStatus = MM_STATUS_UNSUCCESSFUL;
        }
    }

    /* free create info */
    ((*MixerContext).Free.unwrap_unchecked())(PinConnect as PVOID);

    /* done */
    return MixerStatus;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerCheckFormat(
    mut DataRangeAudio: PKSDATARANGE_AUDIO,
    mut WaveInfo: LPWAVE_INFO,
    mut bInput: ULONG,
) {
    let mut Index: ULONG;
    let mut SampleFrequency: ULONG;
    let mut Result: ULONG = 0 as ULONG;

    Index = 0 as ULONG;
    while Index < AUDIO_TEST_RANGE as ULONG {
        SampleFrequency = TestRange[Index as usize].SampleRate;

        if (*DataRangeAudio).MinimumSampleFrequency <= SampleFrequency
            && (*DataRangeAudio).MaximumSampleFrequency >= SampleFrequency
        {
            /* the audio adapter supports the sample frequency */
            if (*DataRangeAudio).MinimumBitsPerSample <= 8 as ULONG
                && (*DataRangeAudio).MaximumBitsPerSample >= 8 as ULONG
            {
                Result |= TestRange[Index as usize].Bit8Mono;

                if (*DataRangeAudio).MaximumChannels > 1 as ULONG {
                    /* check if pin supports the sample rate in 8-Bit Stereo */
                    Result |= TestRange[Index as usize].Bit8Stereo;
                }
            }

            if (*DataRangeAudio).MinimumBitsPerSample <= 16 as ULONG
                && (*DataRangeAudio).MaximumBitsPerSample >= 16 as ULONG
            {
                /* check if pin supports the sample rate in 16-Bit Mono */
                Result |= TestRange[Index as usize].Bit16Mono;

                if (*DataRangeAudio).MaximumChannels > 1 as ULONG {
                    /* check if pin supports the sample rate in 16-Bit Stereo */
                    Result |= TestRange[Index as usize].Bit16Stereo;
                }
            }
        }
        Index = Index.wrapping_add(1);
    }

    if bInput != 0 {
        (*WaveInfo).u.InCaps.dwFormats = Result as DWORD;
    } else {
        (*WaveInfo).u.OutCaps.dwFormats = Result as DWORD;
    }

    DPRINT!("Format %lx bInput %u\n", Result, bInput);
}
#[no_mangle]
pub unsafe extern "C" fn MMixerInitializeWaveInfo(
    MixerContext: PMIXER_CONTEXT,
    MixerList: PMIXER_LIST,
    MixerData: LPMIXER_DATA,
    DeviceName: LPWSTR,
    bWaveIn: ULONG,
    PinCount: ULONG,
    Pins: PULONG,
) -> MIXER_STATUS {
    let mut Status: MIXER_STATUS;
    let mut MultipleItem: PKSMULTIPLE_ITEM = core::mem::zeroed();
    let mut DataRangeAudio: PKSDATARANGE_AUDIO = core::mem::zeroed();
    let mut WaveInfo: LPWAVE_INFO;

    WaveInfo = (*MixerContext).Alloc.unwrap_unchecked()(
        core::mem::size_of::<WAVE_INFO>() as ULONG,
    ) as LPWAVE_INFO;
    if WaveInfo.is_null() {
        return MM_STATUS_NO_MEMORY;
    }

    if PinCount > 1 {
        /* FIXME support multiple pins for wave device */
        DPRINT1!("Implement support for multiple pins\n");
        //ASSERT(PinCount == 1);
    }

    /* initialize wave info */
    (*WaveInfo).DeviceId = (*MixerData).DeviceId;
    (*WaveInfo).PinId = *Pins.offset(0 as isize);

    /* sanity check */
    ASSERT!(wcslen(DeviceName) < (MAXPNAMELEN as _));

    /* copy device name */
    if bWaveIn != 0 {
        wcscpy(
            core::ptr::addr_of_mut!((*WaveInfo).u.InCaps.szPname) as *mut u16,
            DeviceName as *const u16,
        );
    } else {
        wcscpy(
            core::ptr::addr_of_mut!((*WaveInfo).u.OutCaps.szPname) as *mut u16,
            DeviceName as *const u16,
        );
    }

    /* FIXME determine manufacturer / product id */
    if bWaveIn != 0 {
        (*WaveInfo).u.InCaps.wMid = MM_MICROSOFT as WORD;
        (*WaveInfo).u.InCaps.wPid = MM_PID_UNMAPPED as WORD;
        (*WaveInfo).u.InCaps.vDriverVersion = 1 as MMVERSION;
    } else {
        (*WaveInfo).u.OutCaps.wMid = MM_MICROSOFT as WORD;
        (*WaveInfo).u.OutCaps.wPid = MM_PID_UNMAPPED as WORD;
        (*WaveInfo).u.OutCaps.vDriverVersion = 1 as MMVERSION;
    }

    /* get audio pin data ranges */
    Status = MMixerGetAudioPinDataRanges(
        MixerContext,
        (*MixerData).hDevice,
        *Pins.offset(0 as isize) as ULONG,
        &mut MultipleItem,
    );
    if Status != MM_STATUS_SUCCESS {
        /* failed to get audio pin data ranges */
        (*MixerContext).Free.unwrap_unchecked()(WaveInfo as PVOID);
        return MM_STATUS_UNSUCCESSFUL;
    }

    /* find an KSDATARANGE_AUDIO range */
    Status = MMixerFindAudioDataRange(MultipleItem, &mut DataRangeAudio);
    if Status != MM_STATUS_SUCCESS {
        /* failed to find audio pin data range */
        (*MixerContext).Free.unwrap_unchecked()(MultipleItem as PVOID);
        (*MixerContext).Free.unwrap_unchecked()(WaveInfo as PVOID);
        return MM_STATUS_UNSUCCESSFUL;
    }

    /* store channel count */
    if bWaveIn != 0 {
        (*WaveInfo).u.InCaps.wChannels = (*DataRangeAudio).MaximumChannels as WORD;
    } else {
        (*WaveInfo).u.OutCaps.wChannels = (*DataRangeAudio).MaximumChannels as WORD;
    }

    /* get all supported formats */
    MMixerCheckFormat(DataRangeAudio, WaveInfo, bWaveIn);

    /* free dataranges buffer */
    (*MixerContext).Free.unwrap_unchecked()(MultipleItem as PVOID);

    if bWaveIn != 0 {
        InsertTailList(
            core::ptr::addr_of_mut!((*MixerList).WaveInList),
            core::ptr::addr_of_mut!((*WaveInfo).Entry),
        );
        (*MixerList).WaveInListCount += 1;
    } else {
        InsertTailList(
            core::ptr::addr_of_mut!((*MixerList).WaveOutList),
            core::ptr::addr_of_mut!((*WaveInfo).Entry),
        );
        (*MixerList).WaveOutListCount += 1;
    }

    return MM_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerOpenWave(
    MixerContext: PMIXER_CONTEXT,
    DeviceIndex: ULONG,
    bWaveIn: ULONG,
    WaveFormat: LPWAVEFORMATEX,
    CreateCallback: PIN_CREATE_CALLBACK,
    Context: PVOID,
    PinHandle: PHANDLE,
) -> MIXER_STATUS {
    let mut MixerList: PMIXER_LIST = core::ptr::null_mut();
    let mut Status: MIXER_STATUS;
    let mut WaveInfo: LPWAVE_INFO = core::ptr::null_mut();
    let mut DesiredAccess: ACCESS_MASK = 0;

    /* verify mixer context */
    Status = MMixerVerifyContext(MixerContext);

    if Status != MM_STATUS_SUCCESS {
        /* invalid context passed */
        return Status;
    }

    /* grab mixer list */
    MixerList = (*MixerContext).MixerContext as PMIXER_LIST;

    /* find destination wave */
    Status = MMixerGetWaveInfoByIndexAndType(
        MixerList,
        DeviceIndex,
        bWaveIn,
        core::ptr::addr_of_mut!(WaveInfo),
    );
    if Status != MM_STATUS_SUCCESS {
        /* failed to find wave info */
        return MM_STATUS_INVALID_PARAMETER;
    }

    /* get desired access */
    if bWaveIn != 0 {
        DesiredAccess |= GENERIC_READ;
    } else {
        DesiredAccess |= GENERIC_WRITE;
    }

    /* now try open the pin */
    return MMixerOpenWavePin(
        MixerContext,
        MixerList,
        (*WaveInfo).DeviceId,
        (*WaveInfo).PinId,
        WaveFormat,
        DesiredAccess,
        CreateCallback,
        Context,
        PinHandle,
    );
}
#[no_mangle]
pub unsafe extern "C" fn MMixerWaveInCapabilities(
       MixerContext: PMIXER_CONTEXT,
       DeviceIndex: ULONG,
        Caps: LPWAVEINCAPSW) -> MIXER_STATUS {
    let mut MixerList: PMIXER_LIST = core::ptr::null_mut();
    let mut Status: MIXER_STATUS;
    let mut WaveInfo: LPWAVE_INFO = core::ptr::null_mut();

    /* verify mixer context */
    Status = MMixerVerifyContext(MixerContext);

    if Status != MM_STATUS_SUCCESS
    {
        /* invalid context passed */
        return Status;
    }

    /* grab mixer list */
    MixerList = (*MixerContext).MixerContext as PMIXER_LIST;

    /* find destination wave */
    Status = MMixerGetWaveInfoByIndexAndType(MixerList, DeviceIndex, (TRUE) as ULONG, &mut WaveInfo);
    if Status != MM_STATUS_SUCCESS
    {
        /* failed to find wave info */
        return MM_STATUS_UNSUCCESSFUL;
    }

    /* copy capabilities */
    (*MixerContext).Copy.unwrap_unchecked()(
        Caps as PVOID,
        core::ptr::addr_of_mut!((*WaveInfo).u.InCaps) as PVOID,
        core::mem::size_of::<WAVEINCAPSW>() as ULONG,
    );

    return MM_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerWaveOutCapabilities(
    mut MixerContext: PMIXER_CONTEXT,
    mut DeviceIndex: ULONG,
    mut Caps: LPWAVEOUTCAPSW,
) -> MIXER_STATUS {
    let mut MixerList: PMIXER_LIST = core::ptr::null_mut();
    let mut Status: MIXER_STATUS = MM_STATUS_SUCCESS;
    let mut WaveInfo: LPWAVE_INFO = core::ptr::null_mut();

    /* verify mixer context */
    Status = MMixerVerifyContext(MixerContext);

    if Status != MM_STATUS_SUCCESS {
        /* invalid context passed */
        return Status;
    }

    /* grab mixer list */
    MixerList = (*MixerContext).MixerContext as PMIXER_LIST;

    /* find destination wave */
    Status = MMixerGetWaveInfoByIndexAndType(MixerList, DeviceIndex, (FALSE) as ULONG, &raw mut WaveInfo);
    if Status != MM_STATUS_SUCCESS {
        /* failed to find wave info */
        return MM_STATUS_UNSUCCESSFUL;
    }

    /* copy capabilities */
    (*MixerContext).Copy.unwrap_unchecked()(
        Caps as PVOID,
        core::ptr::addr_of_mut!((*WaveInfo).u.OutCaps) as PVOID,
        core::mem::size_of::<WAVEOUTCAPSW>() as ULONG,
    );

    return MM_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetWaveInCount(MixerContext: PMIXER_CONTEXT) -> ULONG {
    let mut MixerList: PMIXER_LIST;
    let mut Status: MIXER_STATUS;

    /* verify mixer context */
    Status = MMixerVerifyContext(MixerContext);

    if Status != MM_STATUS_SUCCESS {
        /* invalid context passed */
        return Status;
    }

    /* grab mixer list */
    MixerList = (*MixerContext).MixerContext as PMIXER_LIST;

    return (*MixerList).WaveInListCount;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetWaveOutCount(mut MixerContext: PMIXER_CONTEXT) -> ULONG {
    let mut MixerList: PMIXER_LIST = core::ptr::null_mut();
    let mut Status: MIXER_STATUS;
    Status = MMixerVerifyContext(MixerContext);
    if Status != MM_STATUS_SUCCESS {
        return Status as ULONG;
    }
    MixerList = (*MixerContext).MixerContext as PMIXER_LIST;
    return (*MixerList).WaveOutListCount;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetWavePosition(
    MixerContext: PMIXER_CONTEXT,
    PinHandle: HANDLE,
    Position: PDWORD,
) -> MIXER_STATUS {
    let mut AudioPosition: KSAUDIO_POSITION = core::mem::zeroed();
    let mut Property: KSPROPERTY = core::mem::zeroed();
    let mut Status: MIXER_STATUS;
    let mut Length: ULONG = core::mem::zeroed();

    /* Validate mixer context */
    Status = MMixerVerifyContext(MixerContext);

    if Status != MM_STATUS_SUCCESS {
        return Status;
    }

    Property.Id = KSPROPERTY_AUDIO_POSITION;
    Property.Set = KSPROPSETID_Audio;
    Property.Flags = KSPROPERTY_TYPE_GET;

    Status = ((*MixerContext).Control.unwrap_unchecked())(
        PinHandle,
        IOCTL_KS_PROPERTY,
        &raw mut Property as PVOID,
        core::mem::size_of::<KSPROPERTY>() as ULONG,
        &raw mut AudioPosition as PVOID,
        core::mem::size_of::<KSAUDIO_POSITION>() as ULONG,
        &raw mut Length,
    );
    if Status == MM_STATUS_SUCCESS {
        /* store audio position */
        *Position = AudioPosition.PlayOffset as DWORD;
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerSetWaveStatus(
    mut MixerContext: PMIXER_CONTEXT,
    mut PinHandle: HANDLE,
    mut State: KSSTATE,
) -> MIXER_STATUS {
    let mut Property: KSPROPERTY = core::mem::zeroed();
    let mut Length: ULONG = 0;
    let mut Status: MIXER_STATUS = core::mem::zeroed();

    /* verify mixer context */
    Status = MMixerVerifyContext(MixerContext);

    if Status != MM_STATUS_SUCCESS {
        /* invalid context passed */
        return Status;
    }

    /* setup property request */
    Property.Set = KSPROPSETID_Connection;
    Property.Id = KSPROPERTY_CONNECTION_STATE;
    Property.Flags = KSPROPERTY_TYPE_SET;

    return (*MixerContext).Control.unwrap_unchecked()(
        PinHandle,
        IOCTL_KS_PROPERTY,
        core::ptr::addr_of_mut!(Property) as PVOID,
        core::mem::size_of::<KSPROPERTY>() as ULONG,
        core::ptr::addr_of_mut!(State) as PVOID,
        core::mem::size_of::<KSSTATE>() as ULONG,
        core::ptr::addr_of_mut!(Length),
    );
}
#[no_mangle]
pub unsafe extern "C" fn MMixerSetWaveResetState(
    mut MixerContext: PMIXER_CONTEXT,
    mut PinHandle: HANDLE,
    mut bBegin: ULONG,
) -> MIXER_STATUS {
    let mut Length: ULONG = 0;
    let mut Status: MIXER_STATUS = MM_STATUS_SUCCESS;
    let mut Reset: KSRESET = KSRESET_BEGIN as KSRESET;

    /* verify mixer context */
    Status = MMixerVerifyContext(MixerContext);

    if Status != MM_STATUS_SUCCESS {
        /* invalid context passed */
        return Status;
    }

    /* begin / stop reset */
    Reset = (if bBegin != 0 {
        KSRESET_BEGIN
    } else {
        KSRESET_END
    }) as KSRESET;

    return (*MixerContext).Control.unwrap_unchecked()(
        PinHandle,
        IOCTL_KS_RESET_STATE as ULONG,
        core::ptr::addr_of_mut!(Reset) as PVOID,
        core::mem::size_of::<KSRESET>() as ULONG,
        core::ptr::null_mut(),
        0 as ULONG,
        core::ptr::addr_of_mut!(Length),
    );
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetWaveDevicePath(
    mut MixerContext: PMIXER_CONTEXT,
    mut bWaveIn: ULONG,
    mut DeviceId: ULONG,
    mut DevicePath: *mut LPWSTR,
) -> MIXER_STATUS {
    let mut MixerList: PMIXER_LIST = core::ptr::null_mut();
    let mut MixerData: LPMIXER_DATA = core::ptr::null_mut();
    let mut WaveInfo: LPWAVE_INFO = core::ptr::null_mut();
    let mut Length: SIZE_T = 0;
    let mut Status: MIXER_STATUS = MM_STATUS_SUCCESS;

    /* verify mixer context */
    Status = MMixerVerifyContext(MixerContext);

    if Status != MM_STATUS_SUCCESS {
        /* invalid context passed */
        return Status;
    }

    /* grab mixer list */
    MixerList = (*MixerContext).MixerContext as PMIXER_LIST;

    /* find destination wave */
    Status = MMixerGetWaveInfoByIndexAndType(
        MixerList,
        DeviceId,
        bWaveIn,
        core::ptr::addr_of_mut!(WaveInfo),
    );
    if Status != MM_STATUS_SUCCESS {
        /* failed to find wave info */
        return MM_STATUS_INVALID_PARAMETER;
    }

    /* get associated device id */
    MixerData = MMixerGetDataByDeviceId(MixerList, (*WaveInfo).DeviceId);
    if MixerData.is_null() {
        return MM_STATUS_INVALID_PARAMETER;
    }

    /* calculate length */
    Length = (wcslen((*MixerData).DeviceName as *const wchar_t)
        .wrapping_add(1 as _)) as SIZE_T;

    /* allocate destination buffer */
    *DevicePath = (*MixerContext).Alloc.unwrap_unchecked()(
        (Length as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as ULONG,
    ) as LPWSTR;

    if (*DevicePath).is_null() {
        /* no memory */
        return MM_STATUS_NO_MEMORY;
    }

    /* copy device path */
    (*MixerContext).Copy.unwrap_unchecked()(
        *DevicePath as PVOID,
        (*MixerData).DeviceName as PVOID,
        (Length as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as ULONG,
    );

    /* done */
    return MM_STATUS_SUCCESS;
}
