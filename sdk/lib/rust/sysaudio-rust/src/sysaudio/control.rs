use crate::prelude::*;
use ntkrnl_sys::ddk::{GUID, LPWSTR, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP, RtlFreeUnicodeString, RtlStringFromGUID};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut KSPROPSETID_Sysaudio: GUID = _GUID {
    Data1: 0xcbe3faa0 as ::core::ffi::c_ulong as ::core::ffi::c_uint,
    Data2: 0xcc75 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xb4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x65 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x18 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x18 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSPROPSETID_Sysaudio_Pin: GUID = _GUID {
    Data1: 0xa3a53220 as ::core::ffi::c_ulong as ::core::ffi::c_uint,
    Data2: 0xc6e4 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xb4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x65 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x18 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x18 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSPROPSETID_General: GUID = _GUID {
    Data1: 0x1464eda5 as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0x6a8f as ::core::ffi::c_ushort,
    Data3: 0x11d1 as ::core::ffi::c_ushort,
    Data4: [
        0x9a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x22 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x31 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSPROPSETID_Pin: GUID = _GUID {
    Data1: 0x8c134960 as ::core::ffi::c_ulong as ::core::ffi::c_uint,
    Data2: 0x51ad as ::core::ffi::c_ushort,
    Data3: 0x11cf as ::core::ffi::c_ushort,
    Data4: [
        0x87 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x94 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

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
pub static mut KSPROPSETID_Topology: GUID = _GUID {
    Data1: 0x720d4ac0 as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0x7533 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
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
pub unsafe extern "C" fn SetIrpIoStatus(
    mut Irp: PIRP,
    mut Status: NTSTATUS,
    mut Length: ULONG,
) -> NTSTATUS {
    (*Irp).IoStatus.Information = Length as ULONG_PTR;
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    if Status != STATUS_PENDING {
        IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
    } else {
        IoMarkIrpPending(Irp);
    }
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn GetListEntry(
    mut Head: PLIST_ENTRY,
    mut Index: ULONG,
) -> PKSAUDIO_DEVICE_ENTRY {
    let mut Entry: PLIST_ENTRY = (*Head).Flink as PLIST_ENTRY;
    loop {
        let fresh15 = Index;
        Index = Index.wrapping_sub(1);
        if !(fresh15 != 0 && Entry != Head) {
            break;
        }
        Entry = (*Entry).Flink as PLIST_ENTRY;
    }
    if Entry == Head {
        return ::core::ptr::null_mut::<KSAUDIO_DEVICE_ENTRY>();
    }
    return ((Entry as ULONG_PTR).wrapping_sub(
        core::mem::offset_of!(KSAUDIO_DEVICE_ENTRY, Entry) as ULONG_PTR,
    ) as *mut KSAUDIO_DEVICE_ENTRY) as PKSAUDIO_DEVICE_ENTRY;
}
#[no_mangle]
pub unsafe extern "C" fn SysAudioOpenVirtualDevice(
    Irp: PIRP,
    DeviceNumber: ULONG,
    DeviceExtension: PSYSAUDIODEVEXT,
) -> NTSTATUS {
    let mut Entry: PKSAUDIO_DEVICE_ENTRY = core::ptr::null_mut();
    let mut IoStack: PIO_STACK_LOCATION = core::ptr::null_mut();

    /* get current irp stack */
    IoStack = IoGetCurrentIrpStackLocation(Irp);

    /* sanity check */
    ASSERT!(!(*IoStack).FileObject.is_null());

    if DeviceNumber >= (*DeviceExtension).NumberOfKsAudioDevices {
        /* invalid device index */
        return SetIrpIoStatus(Irp, STATUS_INVALID_PARAMETER, 0);
    }

    /* get device context */
    Entry = GetListEntry(core::ptr::addr_of_mut!((*DeviceExtension).KsAudioDeviceList), DeviceNumber);
    ASSERT!(!Entry.is_null());

    /* store device entry in FsContext
     * see pin.c DispatchCreateSysAudioPin for details
     */
    (*(*IoStack).FileObject).FsContext = Entry as PVOID;

    return SetIrpIoStatus(Irp, STATUS_SUCCESS, 0);
}
#[no_mangle]
pub unsafe extern "C" fn HandleSysAudioFilterPinProperties(
    Irp: PIRP,
    Property: PKSPROPERTY,
    DeviceExtension: PSYSAUDIODEVEXT,
) -> NTSTATUS {
    let mut IoStack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut Entry: PKSAUDIO_DEVICE_ENTRY;
    let mut BytesReturned: ULONG = 0;

    // in order to access pin properties of a sysaudio device
    // the caller must provide a KSP_PIN struct, where
    // Reserved member points to virtual device index

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    if ((*IoStack).Parameters.DeviceIoControl.InputBufferLength as usize)
        < core::mem::size_of::<KSP_PIN>()
    {
        /* too small buffer */
        return SetIrpIoStatus(
            Irp,
            STATUS_BUFFER_TOO_SMALL,
            (core::mem::size_of::<KSPROPERTY>() + core::mem::size_of::<ULONG>()) as ULONG,
        );
    }

    Entry = GetListEntry(
        &raw mut (*DeviceExtension).KsAudioDeviceList,
        (*(Property as *mut KSP_PIN)).Reserved,
    );
    if Entry.is_null() {
        /* invalid device index */
        return SetIrpIoStatus(Irp, STATUS_INVALID_PARAMETER, 0 as ULONG);
    }

    /* forward request to the filter implementing the property */
    Status = KsSynchronousIoControlDevice(
        (*Entry).FileObject,
        0i32 as ::core::ffi::c_int as KPROCESSOR_MODE,
        IOCTL_KS_PROPERTY as ULONG,
        (*IoStack).Parameters.DeviceIoControl.Type3InputBuffer,
        (*IoStack).Parameters.DeviceIoControl.InputBufferLength,
        (*Irp).UserBuffer,
        (*IoStack).Parameters.DeviceIoControl.OutputBufferLength,
        &raw mut BytesReturned,
    );

    return SetIrpIoStatus(Irp, Status, BytesReturned);
}
#[no_mangle]
pub unsafe extern "C" fn ComputeCompatibleFormat(
    Entry: PKSAUDIO_DEVICE_ENTRY,
    PinId: ULONG,
    ClientFormat: PKSDATAFORMAT_WAVEFORMATEX,
    MixerFormat: PKSDATAFORMAT_WAVEFORMATEX,
) -> NTSTATUS {
    let mut bFound: BOOL = 0;
    let mut BytesReturned: ULONG = 0;
    let mut PinRequest: PKSP_PIN = core::ptr::null_mut::<KSP_PIN>();
    let mut Status: NTSTATUS = 0;
    let mut MultipleItem: PKSMULTIPLE_ITEM = core::ptr::null_mut::<KSMULTIPLE_ITEM>();
    let mut Length: ULONG = 0;
    let mut AudioRange: PKSDATARANGE_AUDIO = core::ptr::null_mut::<KSDATARANGE_AUDIO>();
    let mut Index: ULONG = 0;

    Length = (core::mem::size_of::<KSP_PIN>()
        + core::mem::size_of::<KSMULTIPLE_ITEM>()
        + (*ClientFormat).DataFormat.c2rust_unnamed.FormatSize as usize) as ULONG;
    PinRequest = AllocateItem(NonPagedPool, Length as SIZE_T) as PKSP_PIN;
    if PinRequest.is_null() {
        return STATUS_UNSUCCESSFUL;
    }

    (*PinRequest).PinId = PinId;
    (*PinRequest).Property.Set = KSPROPSETID_Pin;
    (*PinRequest).Property.Flags = KSPROPERTY_TYPE_GET as ULONG;
    (*PinRequest).Property.Id = KSPROPERTY_PIN_DATAINTERSECTION as ULONG;

    MultipleItem = PinRequest.offset(1 as isize) as PKSMULTIPLE_ITEM;
    (*MultipleItem).Count = 1 as ULONG;
    (*MultipleItem).Size = (*ClientFormat).DataFormat.c2rust_unnamed.FormatSize;

    core::ptr::copy(
        ClientFormat as *const u8,
        MultipleItem.offset(1 as isize) as *mut u8,
        (*ClientFormat).DataFormat.c2rust_unnamed.FormatSize as usize,
    );
    /* Query the miniport data intersection handler */
    Status = KsSynchronousIoControlDevice(
        (*Entry).FileObject,
        0i32 as KPROCESSOR_MODE,
        IOCTL_KS_PROPERTY as ULONG,
        PinRequest as PVOID,
        Length,
        MixerFormat as PVOID,
        core::mem::size_of::<KSDATAFORMAT_WAVEFORMATEX>() as ULONG,
        core::ptr::addr_of_mut!(BytesReturned),
    );

    DPRINT!("Status %x\n", Status);

    if NT_SUCCESS!(Status) {
        FreeItem(PinRequest as PVOID);
        return Status;
    }

    /* Setup request block */
    (*PinRequest).Property.Id = KSPROPERTY_PIN_DATARANGES as ULONG;
    /* Query pin data ranges */
    Status = KsSynchronousIoControlDevice(
        (*Entry).FileObject,
        0i32 as KPROCESSOR_MODE,
        IOCTL_KS_PROPERTY as ULONG,
        PinRequest as PVOID,
        core::mem::size_of::<KSP_PIN>() as ULONG,
        core::ptr::null_mut(),
        0 as ULONG,
        core::ptr::addr_of_mut!(BytesReturned),
    );

    if Status != STATUS_MORE_ENTRIES {
        /* Failed to get data ranges */
        return Status;
    }

    MultipleItem = AllocateItem((NonPagedPool as SIZE_T) as POOL_TYPE, BytesReturned as SIZE_T) as PKSMULTIPLE_ITEM;
    if MultipleItem.is_null() {
        FreeItem(PinRequest as PVOID);
        return STATUS_NO_MEMORY;
    }

    Status = KsSynchronousIoControlDevice(
        (*Entry).FileObject,
        0i32 as KPROCESSOR_MODE,
        IOCTL_KS_PROPERTY as ULONG,
        PinRequest as PVOID,
        core::mem::size_of::<KSP_PIN>() as ULONG,
        MultipleItem as PVOID,
        BytesReturned,
        core::ptr::addr_of_mut!(BytesReturned),
    );
    if !NT_SUCCESS!(Status) {
        DPRINT!("Property Request KSPROPERTY_PIN_DATARANGES failed with %x\n", Status);
        FreeItem(MultipleItem as PVOID);
        FreeItem(PinRequest as PVOID);
        return STATUS_UNSUCCESSFUL;
    }

    AudioRange = MultipleItem.offset(1 as isize) as PKSDATARANGE_AUDIO;
    bFound = (FALSE as i32);
    Index = 0 as ULONG;
    while Index < (*MultipleItem).Count {
        if (*AudioRange).DataRange.c2rust_unnamed.FormatSize as usize
            != core::mem::size_of::<KSDATARANGE_AUDIO>() as usize
        {
            UNIMPLEMENTED!();
            AudioRange = (AudioRange as *mut u8)
                .offset((*AudioRange).DataRange.c2rust_unnamed.FormatSize as isize)
                as PKSDATARANGE_AUDIO;
            Index = Index.wrapping_add(1);
            continue;
        }
        /* Select best quality available */

        (*MixerFormat).DataFormat.c2rust_unnamed.FormatSize =
            (core::mem::size_of::<KSDATAFORMAT>()
                + core::mem::size_of::<WAVEFORMATEX>()) as ULONG;
        (*MixerFormat).DataFormat.c2rust_unnamed.Flags = 0 as ULONG;
        (*MixerFormat).DataFormat.c2rust_unnamed.Reserved = 0 as ULONG;
        (*MixerFormat).DataFormat.c2rust_unnamed.MajorFormat = KSDATAFORMAT_TYPE_AUDIO;
        (*MixerFormat).DataFormat.c2rust_unnamed.SubFormat = KSDATAFORMAT_SUBTYPE_PCM;
        (*MixerFormat).DataFormat.c2rust_unnamed.Specifier = KSDATAFORMAT_SPECIFIER_WAVEFORMATEX;
        (*MixerFormat).DataFormat.c2rust_unnamed.SampleSize = 4 as ULONG;
        (*MixerFormat).WaveFormatEx.wFormatTag = (*ClientFormat).WaveFormatEx.wFormatTag;
        /* HACK: AC97 does not support mono render / record */
        (*MixerFormat).WaveFormatEx.nChannels = 2 as WORD;
        /*HACK: AC97 only supports 16-Bit Bits */
        (*MixerFormat).WaveFormatEx.wBitsPerSample = 16 as WORD;

        (*MixerFormat).WaveFormatEx.nSamplesPerSec = max(
            (*AudioRange).MinimumSampleFrequency,
            min(
                (*ClientFormat).WaveFormatEx.nSamplesPerSec,
                (*AudioRange).MaximumSampleFrequency,
            ),
        ) as DWORD;

        (*MixerFormat).WaveFormatEx.cbSize = 0 as WORD;
        (*MixerFormat).WaveFormatEx.nBlockAlign = (((*MixerFormat).WaveFormatEx.nChannels as i32
            * (*MixerFormat).WaveFormatEx.wBitsPerSample as i32)
            / 8 as i32) as WORD;
        (*MixerFormat).WaveFormatEx.nAvgBytesPerSec = ((*MixerFormat).WaveFormatEx.nChannels
            as DWORD)
            .wrapping_mul((*MixerFormat).WaveFormatEx.nSamplesPerSec)
            .wrapping_mul(
                ((*MixerFormat).WaveFormatEx.wBitsPerSample as i32
                    / 8 as i32) as DWORD,
            );

        bFound = (TRUE as i32);
        break;

        AudioRange = (AudioRange as *mut u8)
            .offset((*AudioRange).DataRange.c2rust_unnamed.FormatSize as isize)
            as PKSDATARANGE_AUDIO;
        Index = Index.wrapping_add(1);
    }

    FreeItem(MultipleItem as PVOID);
    FreeItem(PinRequest as PVOID);

    if bFound != 0 {
        return STATUS_SUCCESS;
    } else {
        return STATUS_NOT_IMPLEMENTED;
    }
}
#[no_mangle]
pub unsafe extern "C" fn GetPinInstanceCount(
    mut Entry: PKSAUDIO_DEVICE_ENTRY,
    mut PinInstances: PKSPIN_CINSTANCES,
    mut PinConnect: PKSPIN_CONNECT,
) -> NTSTATUS {
    let mut PinRequest: KSP_PIN = core::mem::zeroed::<KSP_PIN>();
    let mut BytesReturned: ULONG = 0;

    /* query the instance count */
    PinRequest.PinId = (*PinConnect).PinId;
    PinRequest.Property.Set = KSPROPSETID_Pin;
    PinRequest.Property.Flags = KSPROPERTY_TYPE_GET;
    PinRequest.Property.Id = KSPROPERTY_PIN_CINSTANCES;
    ASSERT!(!(*Entry).FileObject.is_null());
    return KsSynchronousIoControlDevice(
        (*Entry).FileObject,
        0i32 as KPROCESSOR_MODE,
        IOCTL_KS_PROPERTY,
        core::ptr::addr_of_mut!(PinRequest) as PVOID,
        core::mem::size_of::<KSP_PIN>() as ULONG,
        PinInstances as PVOID,
        core::mem::size_of::<KSPIN_CINSTANCES>() as ULONG,
        core::ptr::addr_of_mut!(BytesReturned),
    );
}
#[no_mangle]
pub unsafe extern "C" fn SysAudioHandleProperty(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut IoStack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut Status: NTSTATUS = STATUS_NOT_IMPLEMENTED;
    let mut PropertyRequest: KSPROPERTY = core::mem::zeroed();
    let mut ComponentId: KSCOMPONENTID = core::mem::zeroed();
    let mut Index: PULONG = core::ptr::null_mut();
    let mut Property: PKSPROPERTY = core::ptr::null_mut();
    let mut DeviceExtension: PSYSAUDIODEVEXT = core::ptr::null_mut();
    let mut Entry: PKSAUDIO_DEVICE_ENTRY = core::ptr::null_mut();
    let mut InstanceInfo: PSYSAUDIO_INSTANCE_INFO = core::ptr::null_mut();
    let mut BytesReturned: ULONG = 0;
    let mut GuidString: _UNICODE_STRING = core::mem::zeroed();
    let mut Pin: PKSP_PIN = core::ptr::null_mut();
    let mut DeviceName: LPWSTR = core::ptr::null_mut();

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    if (*IoStack).Parameters.DeviceIoControl.InputBufferLength < (core::mem::size_of::<KSPROPERTY>()) as _ {
        /* buffer must be at least of sizeof KSPROPERTY */
        return SetIrpIoStatus(Irp, STATUS_BUFFER_TOO_SMALL, (core::mem::size_of::<KSPROPERTY>()) as ULONG);
    }

    Property = (*IoStack).Parameters.DeviceIoControl.Type3InputBuffer as PKSPROPERTY;
    DeviceExtension = (*DeviceObject).DeviceExtension as PSYSAUDIODEVEXT;

    if IsEqualGUIDAligned(
        core::ptr::addr_of_mut!((*Property).Set),
        core::ptr::addr_of!(KSPROPSETID_Pin),
    ) != false
    {
        return HandleSysAudioFilterPinProperties(Irp, Property, DeviceExtension);
    } else if IsEqualGUIDAligned(
        core::ptr::addr_of_mut!((*Property).Set),
        core::ptr::addr_of!(KSPROPSETID_Topology),
    ) != false
    {
        if (*IoStack).Parameters.DeviceIoControl.InputBufferLength < (core::mem::size_of::<KSP_PIN>()) as _ {
            /* too small buffer */
            return SetIrpIoStatus(Irp, STATUS_BUFFER_TOO_SMALL, (core::mem::size_of::<KSP_PIN>()) as ULONG);
        }
        Pin = (*IoStack).Parameters.DeviceIoControl.Type3InputBuffer as PKSP_PIN;
        Entry = GetListEntry(
            core::ptr::addr_of_mut!((*DeviceExtension).KsAudioDeviceList),
            (*Pin).Reserved,
        );
        ASSERT!(!Entry.is_null());

        /* forward request to the filter implementing the property */
        Status = KsSynchronousIoControlDevice(
            (*Entry).FileObject,
            (0i32) as KPROCESSOR_MODE,
            IOCTL_KS_PROPERTY,
            (*IoStack).Parameters.DeviceIoControl.Type3InputBuffer,
            (*IoStack).Parameters.DeviceIoControl.InputBufferLength,
            (*Irp).UserBuffer,
            (*IoStack).Parameters.DeviceIoControl.OutputBufferLength,
            core::ptr::addr_of_mut!(BytesReturned),
        );

        return SetIrpIoStatus(Irp, Status, BytesReturned);
    } else if IsEqualGUIDAligned(
        core::ptr::addr_of_mut!((*Property).Set),
        core::ptr::addr_of!(KSPROPSETID_Sysaudio),
    ) != false
    {
        if (*Property).Id == KSPROPERTY_SYSAUDIO_DEVICE_INTERFACE_NAME {
            if (*IoStack).Parameters.DeviceIoControl.InputBufferLength
                < (core::mem::size_of::<KSPROPERTY>() + core::mem::size_of::<ULONG>()) as ULONG
            {
                /* invalid request */
                return SetIrpIoStatus(
                    Irp,
                    STATUS_UNSUCCESSFUL,
                    (core::mem::size_of::<KSPROPERTY>() + core::mem::size_of::<ULONG>()) as ULONG,
                );
            }
            Index = Property.offset(1 as isize) as PULONG;

            if (*DeviceExtension).NumberOfKsAudioDevices <= *Index {
                /* invalid index */
                return SetIrpIoStatus(Irp, STATUS_INVALID_PARAMETER, 0);
            }

            Entry = GetListEntry(
                core::ptr::addr_of_mut!((*DeviceExtension).KsAudioDeviceList),
                *Index,
            );
            ASSERT!(!Entry.is_null());

            BytesReturned = ((*Entry).DeviceName.Length as usize
                + core::mem::size_of::<u16>())
                as ULONG;
            if (*IoStack).Parameters.DeviceIoControl.OutputBufferLength < BytesReturned {
                /* too small buffer */
                return SetIrpIoStatus(Irp, STATUS_BUFFER_TOO_SMALL, BytesReturned);
            }

            /* copy device name */
            DeviceName = (*Irp).UserBuffer as LPWSTR;

            RtlMoveMemory!(
                DeviceName as PVOID,
                (*Entry).DeviceName.Buffer as PVOID,
                (*Entry).DeviceName.Length as usize,
            );
            *DeviceName.offset(
                ((*Entry).DeviceName.Length as usize / core::mem::size_of::<u16>()) as isize,
            ) = 0x0000u16;
            return SetIrpIoStatus(Irp, STATUS_SUCCESS, BytesReturned);
        }

        if (*Property).Id == KSPROPERTY_SYSAUDIO_COMPONENT_ID {
            if (*IoStack).Parameters.DeviceIoControl.InputBufferLength
                < (core::mem::size_of::<KSPROPERTY>() + core::mem::size_of::<ULONG>()) as ULONG
            {
                /* too small buffer */
                return SetIrpIoStatus(
                    Irp,
                    STATUS_BUFFER_TOO_SMALL,
                    (core::mem::size_of::<KSPROPERTY>() + core::mem::size_of::<ULONG>()) as ULONG,
                );
            }

            if (*IoStack).Parameters.DeviceIoControl.OutputBufferLength < (core::mem::size_of::<KSCOMPONENTID>()) as _ {
                /* too small buffer */
                return SetIrpIoStatus(Irp, STATUS_BUFFER_TOO_SMALL, (core::mem::size_of::<KSCOMPONENTID>()) as ULONG);
            }

            Index = Property.offset(1 as isize) as PULONG;

            if (*DeviceExtension).NumberOfKsAudioDevices <= *Index {
                /* invalid index */
                return SetIrpIoStatus(Irp, STATUS_INVALID_PARAMETER, 0);
            }
            Entry = GetListEntry(
                core::ptr::addr_of_mut!((*DeviceExtension).KsAudioDeviceList),
                *Index,
            );
            ASSERT!(!Entry.is_null());

            PropertyRequest.Set = KSPROPSETID_General;
            PropertyRequest.Id = KSPROPERTY_GENERAL_COMPONENTID;
            PropertyRequest.Flags = KSPROPERTY_TYPE_GET;

            /* call the filter */
            Status = KsSynchronousIoControlDevice(
                (*Entry).FileObject,
                (0i32) as KPROCESSOR_MODE,
                IOCTL_KS_PROPERTY,
                addr_of_mut!(PropertyRequest) as PVOID,
                (core::mem::size_of::<KSPROPERTY>()) as ULONG,
                addr_of_mut!(ComponentId) as PVOID,
                (core::mem::size_of::<KSCOMPONENTID>()) as ULONG,
                core::ptr::addr_of_mut!(BytesReturned),
            );
            if !NT_SUCCESS!(Status) {
                DPRINT!(
                    "KsSynchronousIoControlDevice failed with %x for KSPROPERTY_GENERAL_COMPONENTID\n",
                    Status
                );
                return SetIrpIoStatus(Irp, Status, 0);
            }
            RtlMoveMemory!(
                (*Irp).UserBuffer,
                addr_of_mut!(ComponentId) as PVOID,
                core::mem::size_of::<KSCOMPONENTID>(),
            );
            return SetIrpIoStatus(Irp, STATUS_SUCCESS, (core::mem::size_of::<KSCOMPONENTID>()) as ULONG);
        } else if (*Property).Id == KSPROPERTY_SYSAUDIO_DEVICE_COUNT {
            if (*IoStack).Parameters.DeviceIoControl.OutputBufferLength
                < core::mem::size_of::<ULONG>() as ULONG
            {
                /* too small buffer */
                return SetIrpIoStatus(
                    Irp,
                    STATUS_BUFFER_TOO_SMALL,
                    core::mem::size_of::<ULONG>() as ULONG,
                );
            }

            *((*Irp).UserBuffer as PULONG) = (*DeviceExtension).NumberOfKsAudioDevices;
            return SetIrpIoStatus(
                Irp,
                STATUS_SUCCESS,
                core::mem::size_of::<ULONG>() as ULONG,
            );
        } else if (*Property).Id == KSPROPERTY_SYSAUDIO_DEVICE_INSTANCE {
            if (*IoStack).Parameters.DeviceIoControl.OutputBufferLength
                < core::mem::size_of::<ULONG>() as ULONG
            {
                /* too small buffer */
                return SetIrpIoStatus(
                    Irp,
                    STATUS_BUFFER_TOO_SMALL,
                    core::mem::size_of::<ULONG>() as ULONG,
                );
            }

            if ((*Property).Flags & KSPROPERTY_TYPE_SET) != 0 {
                Index = (*Irp).UserBuffer as PULONG;
                return SysAudioOpenVirtualDevice(Irp, *Index, DeviceExtension);
            }
        } else if (*Property).Id == KSPROPERTY_SYSAUDIO_INSTANCE_INFO {
            if (*IoStack).Parameters.DeviceIoControl.InputBufferLength < (core::mem::size_of::<SYSAUDIO_INSTANCE_INFO>()) as _ {
                /* too small buffer */
                return SetIrpIoStatus(Irp, STATUS_BUFFER_TOO_SMALL, (core::mem::size_of::<SYSAUDIO_INSTANCE_INFO>()) as ULONG);
            }

            /* get input parameter */
            InstanceInfo = Property as PSYSAUDIO_INSTANCE_INFO;

            if ((*Property).Flags & KSPROPERTY_TYPE_SET) != 0 {
                return SysAudioOpenVirtualDevice(Irp, (*InstanceInfo).DeviceNumber, DeviceExtension);
            }
        }
    }

    RtlStringFromGUID(
        core::ptr::addr_of_mut!((*Property).Set),
        core::ptr::addr_of_mut!(GuidString),
    );
    DPRINT1!(
        "Unhandled property Set |%S| Id %u Flags %x\n",
        GuidString.Buffer,
        (*Property).Id,
        (*Property).Flags
    );
    RtlFreeUnicodeString(core::ptr::addr_of_mut!(GuidString));
    return SetIrpIoStatus(Irp, STATUS_UNSUCCESSFUL, 0);
}
