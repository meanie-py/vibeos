use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut MMixerLibraryInitialized: BOOL = (FALSE as i32);

#[no_mangle]
pub static mut MixerContext: MIXER_CONTEXT = unsafe {
    MIXER_CONTEXT {
        SizeOfStruct: ::core::mem::size_of::<MIXER_CONTEXT>() as ULONG,
        MixerContext: NULL,
        Alloc: Some(Alloc as unsafe extern "C" fn(ULONG) -> PVOID),
        Control: Some(
            Control
                as unsafe extern "C" fn(
                    HANDLE,
                    ULONG,
                    PVOID,
                    ULONG,
                    PVOID,
                    ULONG,
                    PULONG,
                ) -> MIXER_STATUS,
        ),
        Free: Some(Free as unsafe extern "C" fn(PVOID) -> ()),
        Open: Some(Open as unsafe extern "C" fn(LPWSTR, PHANDLE) -> MIXER_STATUS),
        Close: Some(Close as unsafe extern "C" fn(HANDLE) -> MIXER_STATUS),
        Copy_0: Some(Copy as unsafe extern "C" fn(PVOID, PVOID, ULONG) -> ()),
        OpenKey: Some(
            OpenKey as unsafe extern "C" fn(HANDLE, LPWSTR, ULONG, PHANDLE) -> MIXER_STATUS,
        ),
        QueryKeyValue: Some(
            QueryKeyValue
                as unsafe extern "C" fn(HANDLE, LPWSTR, *mut PVOID, PULONG, PULONG) -> MIXER_STATUS,
        ),
        CloseKey: Some(CloseKey as unsafe extern "C" fn(HANDLE) -> MIXER_STATUS),
        AllocEventData: Some(AllocEventData as unsafe extern "C" fn(ULONG) -> PVOID),
        FreeEventData: Some(FreeEventData as unsafe extern "C" fn(PVOID) -> ()),
    }
};

#[no_mangle]
pub static mut CategoryGuid: GUID = _GUID {
    Data1: 0x6994ad04 as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0x93ef as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xa3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xcc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x22 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x31 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

// #define NTOS_MODE_USER  (valueless flag, no Rust declaration emitted)
// #define NDEBUG          (valueless flag, no Rust declaration emitted)

// Function prototypes (Alloc, Close, Free, Copy, Open, Control, Enum, OpenKey,
// CloseKey, QueryKeyValue, AllocEventData, FreeEventData) are not emitted here;
// the functions are translated as separate items and referenced by name below.

// Field names are not given in the C preamble; they follow the order of the
// initializer and must match the MIXER_CONTEXT definition in the bindgen output.

// STATIC_KSCATEGORY_AUDIO = 6994AD04-93EF-11D0-A3CC-00A0C9223196
#[no_mangle]
pub unsafe extern "C" fn QueryKeyValue(
    hKey: HANDLE,
    KeyName: LPWSTR,
    ResultBuffer: *mut PVOID,
    ResultLength: PULONG,
    KeyType: PULONG,
) -> MIXER_STATUS {
    if RegQueryValueExW(
        hKey as HKEY,
        KeyName as LPCWSTR,
        core::ptr::null_mut::<DWORD>(),
        KeyType as LPDWORD,
        core::ptr::null_mut::<BYTE>(),
        ResultLength as LPDWORD,
    ) == (ERROR_FILE_NOT_FOUND as i32)
    {
        return MM_STATUS_UNSUCCESSFUL;
    }

    *ResultBuffer = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        *ResultLength as SIZE_T,
    );
    if (*ResultBuffer).is_null() {
        return MM_STATUS_NO_MEMORY;
    }

    if RegQueryValueExW(
        hKey as HKEY,
        KeyName as LPCWSTR,
        core::ptr::null_mut::<DWORD>(),
        KeyType as LPDWORD,
        *ResultBuffer as LPBYTE,
        ResultLength as LPDWORD,
    ) != (ERROR_SUCCESS as i32)
    {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            *ResultBuffer,
        );
        return MM_STATUS_UNSUCCESSFUL;
    }
    return MM_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn OpenKey(
    hKey: HANDLE,
    SubKey: LPWSTR,
    DesiredAccess: ULONG,
    OutKey: PHANDLE,
) -> MIXER_STATUS {
    if RegOpenKeyExW(
        hKey as HKEY,
        SubKey,
        0,
        DesiredAccess,
        OutKey as PHKEY,
    ) == (ERROR_SUCCESS as i32)
    {
        return MM_STATUS_SUCCESS;
    }

    return MM_STATUS_UNSUCCESSFUL;
}
#[no_mangle]
pub unsafe extern "C" fn CloseKey(hKey: HANDLE) -> MIXER_STATUS {
    RegCloseKey(hKey as HKEY);
    return MM_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn Alloc(NumBytes: ULONG) -> PVOID {
    return HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        NumBytes as SIZE_T,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Close(hDevice: HANDLE) -> MIXER_STATUS {
    if CloseHandle(hDevice) != 0 {
        return MM_STATUS_SUCCESS;
    } else {
        return MM_STATUS_UNSUCCESSFUL;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Free(Block: PVOID) {
    HeapFree(GetProcessHeap(), 0, Block);
}

#[no_mangle]
pub unsafe extern "C" fn Copy(Src: PVOID, Dst: PVOID, NumBytes: ULONG) {
    RtlMoveMemory!(Src, Dst, NumBytes);
}

#[no_mangle]
pub unsafe extern "C" fn Open(DevicePath: LPWSTR, hDevice: PHANDLE) -> MIXER_STATUS {
    *DevicePath.add(1) = 0x005Cu16;
    *hDevice = CreateFileW(
        DevicePath as LPCWSTR,
        GENERIC_READ | GENERIC_WRITE,
        0,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        FILE_FLAG_OVERLAPPED,
        core::ptr::null_mut(),
    ) as HANDLE;
    if *hDevice == INVALID_HANDLE_VALUE {
        return MM_STATUS_UNSUCCESSFUL;
    }

    return MM_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn Control(
    hMixer: HANDLE,
    dwIoControlCode: ULONG,
    lpInBuffer: PVOID,
    nInBufferSize: ULONG,
    lpOutBuffer: PVOID,
    nOutBufferSize: ULONG,
    lpBytesReturned: PULONG,
) -> MIXER_STATUS {
    let mut Overlapped: OVERLAPPED = core::mem::zeroed();
    let mut IoResult: BOOLEAN;
    let mut Transferred: DWORD = 0;

    /* Overlapped I/O is done here - this is used for waiting for completion */
    ZeroMemory!(core::ptr::addr_of_mut!(Overlapped) as LPVOID, core::mem::size_of::<OVERLAPPED>() as usize);
    Overlapped.hEvent = CreateEvent(
        core::ptr::null_mut(),
        (FALSE as i32),
        (FALSE as i32),
        core::ptr::null(),
    );

    if Overlapped.hEvent.is_null() {
        return MM_STATUS_NO_MEMORY;
    }

    /* Talk to the device */
    IoResult = DeviceIoControl(
        hMixer,
        dwIoControlCode as DWORD,
        lpInBuffer as LPVOID,
        nInBufferSize as DWORD,
        lpOutBuffer as LPVOID,
        nOutBufferSize as DWORD,
        core::ptr::addr_of_mut!(Transferred),
        core::ptr::addr_of_mut!(Overlapped),
    ) as BOOLEAN;

    /* If failure occurs, make sure it's not just due to the overlapped I/O */
    if IoResult == 0 {
        if GetLastError() != ERROR_IO_PENDING {
            CloseHandle(Overlapped.hEvent);

            if GetLastError() == ERROR_MORE_DATA || GetLastError() == ERROR_INSUFFICIENT_BUFFER {
                if !lpBytesReturned.is_null() {
                    *lpBytesReturned = Transferred as ULONG;
                }
                return MM_STATUS_MORE_ENTRIES;
            }

            return MM_STATUS_UNSUCCESSFUL;
        }
    }

    /* Wait for the I/O to complete */
    IoResult = GetOverlappedResult(
        hMixer,
        core::ptr::addr_of_mut!(Overlapped),
        core::ptr::addr_of_mut!(Transferred),
        (TRUE as i32),
    ) as BOOLEAN;

    /* Don't need this any more */
    CloseHandle(Overlapped.hEvent);

    if IoResult == 0 {
        return MM_STATUS_UNSUCCESSFUL;
    }

    if !lpBytesReturned.is_null() {
        *lpBytesReturned = Transferred as ULONG;
    }

    return MM_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn Enum(
    EnumContext: PVOID,
    DeviceIndex: ULONG,
    DeviceName: *mut LPWSTR,
    OutHandle: PHANDLE,
    OutKey: PHANDLE,
) -> MIXER_STATUS {
    let mut InterfaceData: SP_DEVICE_INTERFACE_DATA = core::mem::zeroed::<SP_DEVICE_INTERFACE_DATA>();
    let mut DeviceData: SP_DEVINFO_DATA = core::mem::zeroed::<SP_DEVINFO_DATA>();
    let mut DetailData: PSP_DEVICE_INTERFACE_DETAIL_DATA_W = core::ptr::null_mut();
    let mut Result: BOOL;
    let mut Length: DWORD;
    let mut Status: MIXER_STATUS;

    //printf("Enum EnumContext %p DeviceIndex %lu OutHandle %p\n", EnumContext, DeviceIndex, OutHandle);

    InterfaceData.cbSize = core::mem::size_of::<SP_DEVICE_INTERFACE_DATA>() as DWORD;
    InterfaceData.Reserved = 0 as ULONG_PTR;

    Result = SetupDiEnumDeviceInterfaces(
        EnumContext as HDEVINFO,
        core::ptr::null_mut::<SP_DEVINFO_DATA>(),
        core::ptr::addr_of_mut!(CategoryGuid),
        DeviceIndex as DWORD,
        core::ptr::addr_of_mut!(InterfaceData),
    );

    if Result == 0 {
        if GetLastError() == ERROR_NO_MORE_ITEMS as DWORD {
            return MM_STATUS_NO_MORE_DEVICES;
        }
        return MM_STATUS_UNSUCCESSFUL;
    }

    Length = (core::mem::size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>() as DWORD)
        .wrapping_add((MAX_PATH as DWORD).wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD));
    DetailData = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        Length as SIZE_T,
    ) as PSP_DEVICE_INTERFACE_DETAIL_DATA_W;
    (*DetailData).cbSize = core::mem::size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>() as DWORD;
    DeviceData.cbSize = core::mem::size_of::<SP_DEVINFO_DATA>() as DWORD;
    DeviceData.Reserved = 0 as ULONG_PTR;

    Result = SetupDiGetDeviceInterfaceDetailW(
        EnumContext as HDEVINFO,
        core::ptr::addr_of_mut!(InterfaceData),
        DetailData,
        Length,
        core::ptr::null_mut::<DWORD>(),
        core::ptr::addr_of_mut!(DeviceData),
    );

    if Result == 0 {
        DPRINT!(
            "SetupDiGetDeviceInterfaceDetailW failed with %lu\n",
            GetLastError()
        );
        return MM_STATUS_UNSUCCESSFUL;
    }

    *OutKey = SetupDiOpenDeviceInterfaceRegKey(
        EnumContext as HDEVINFO,
        core::ptr::addr_of_mut!(InterfaceData),
        0 as DWORD,
        KEY_READ as REGSAM,
    ) as *mut core::ffi::c_void;
    if *OutKey as HKEY == INVALID_HANDLE_VALUE as HKEY {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            DetailData as PVOID,
        );
        return MM_STATUS_UNSUCCESSFUL;
    }

    Status = Open(
        core::ptr::addr_of_mut!((*DetailData).DevicePath) as LPWSTR,
        OutHandle,
    );

    if Status as core::ffi::c_uint != MM_STATUS_SUCCESS as core::ffi::c_uint {
        RegCloseKey(*OutKey as HKEY);
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            DetailData as PVOID,
        );
        return Status;
    }

    *DeviceName = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        (wcslen(core::ptr::addr_of_mut!((*DetailData).DevicePath) as *mut WCHAR)
            as SIZE_T)
            .wrapping_add(1 as SIZE_T)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    if (*DeviceName).is_null() {
        CloseHandle(*OutHandle);
        RegCloseKey(*OutKey as HKEY);
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            DetailData as PVOID,
        );
        return MM_STATUS_NO_MEMORY;
    }
    wcscpy(*DeviceName, core::ptr::addr_of_mut!((*DetailData).DevicePath) as *mut WCHAR);
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        DetailData as PVOID,
    );

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn AllocEventData(ExtraSize: ULONG) -> PVOID {
    let Data: PKSEVENTDATA = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        (core::mem::size_of::<KSEVENTDATA>() as SIZE_T).wrapping_add(ExtraSize as SIZE_T),
    ) as PKSEVENTDATA;
    if Data.is_null() {
        return core::ptr::null_mut();
    }

    (*Data).__bindgen_anon_1.EventHandle.Event = CreateEventW(
        core::ptr::null_mut::<SECURITY_ATTRIBUTES>(),
        (FALSE) as BOOL,
        (FALSE) as BOOL,
        core::ptr::null::<WCHAR>(),
    );
    if (*Data).__bindgen_anon_1.EventHandle.Event.is_null() {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            Data as PVOID,
        );
        return core::ptr::null_mut();
    }

    (*Data).NotificationType = KSEVENTF_EVENT_HANDLE as ULONG;
    return Data as PVOID;
}
#[no_mangle]
pub unsafe extern "C" fn FreeEventData(mut EventData: PVOID) {
    let mut Data: PKSEVENTDATA = EventData as PKSEVENTDATA;
    CloseHandle((*Data).__bindgen_anon_1.EventHandle.Event);
    HeapFree(GetProcessHeap(), 0 as DWORD, Data as PVOID);
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudInitUserModeMixer() -> BOOL {
    let mut DeviceHandle: HDEVINFO = core::ptr::null_mut();
    let mut Status: MIXER_STATUS;

    if MMixerLibraryInitialized != 0 {
        /* library is already initialized */
        return (TRUE) as BOOL;
    }

    /* create a device list */
    DeviceHandle = SetupDiGetClassDevs(
        core::ptr::addr_of_mut!(CategoryGuid),
        core::ptr::null(),
        core::ptr::null_mut(),
        DIGCF_DEVICEINTERFACE | DIGCF_PRESENT,
    );

    if DeviceHandle == INVALID_HANDLE_VALUE {
        /* failed to create a device list */
        return (FALSE) as BOOL;
    }

    /* initialize the mixer library */
    Status = MMixerInitialize(
        core::ptr::addr_of_mut!(MixerContext),
        Some(Enum),
        DeviceHandle as PVOID,
    );

    /* free device list */
    SetupDiDestroyDeviceInfoList(DeviceHandle);

    if Status != MM_STATUS_SUCCESS {
        /* failed to initialize mixer library */
        DPRINT1!("Failed to initialize mixer library with %x\n", Status);
        return (FALSE) as BOOL;
    }

    /* library is now initialized */
    MMixerLibraryInitialized = (TRUE as i32);

    /* completed successfully */
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "C" fn WdmAudCleanupByMMixer() -> MMRESULT {
    MMSYSERR_NOERROR
}

#[no_mangle]
pub unsafe extern "C" fn WdmAudGetMixerCapabilities(
    DeviceId: ULONG,
    Capabilities: LPMIXERCAPSW,
) -> MMRESULT {
    if MMixerGetCapabilities(core::ptr::addr_of_mut!(MixerContext), DeviceId, Capabilities) == MM_STATUS_SUCCESS {
        return MMSYSERR_NOERROR;
    }

    return MMSYSERR_BADDEVICEID;
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudGetLineInfo(
    mut hMixer: HANDLE,
    mut MixerId: DWORD,
    mut MixLine: LPMIXERLINEW,
    mut Flags: ULONG,
) -> MMRESULT {
    if MMixerGetLineInfo(
        core::ptr::addr_of_mut!(MixerContext),
        hMixer,
        MixerId as ULONG,
        Flags,
        MixLine,
    ) == MM_STATUS_SUCCESS
    {
        return MMSYSERR_NOERROR as MMRESULT;
    }
    return MMSYSERR_ERROR as MMRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudGetLineControls(
    hMixer: HANDLE,
    MixerId: DWORD,
    MixControls: LPMIXERLINECONTROLSW,
    Flags: ULONG,
) -> MMRESULT {
    if MMixerGetLineControls(
        core::ptr::addr_of_mut!(MixerContext),
        hMixer,
        MixerId as ULONG,
        Flags,
        MixControls,
    ) == MM_STATUS_SUCCESS
    {
        return MMSYSERR_NOERROR as MMRESULT;
    }

    return MMSYSERR_ERROR as MMRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudSetControlDetails(
    mut hMixer: HANDLE,
    mut MixerId: DWORD,
    mut MixDetails: LPMIXERCONTROLDETAILS,
    mut Flags: ULONG,
) -> MMRESULT {
    if MMixerSetControlDetails(
        core::ptr::addr_of_mut!(MixerContext),
        hMixer,
        MixerId as ULONG,
        Flags,
        MixDetails,
    ) as ::core::ffi::c_uint == MM_STATUS_SUCCESS as ::core::ffi::c_uint {
        return MMSYSERR_NOERROR as MMRESULT;
    }

    return MMSYSERR_ERROR as MMRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudGetControlDetails(
    mut hMixer: HANDLE,
    mut MixerId: DWORD,
    mut MixDetails: LPMIXERCONTROLDETAILS,
    mut Flags: ULONG,
) -> MMRESULT {
    if MMixerGetControlDetails(
        core::ptr::addr_of_mut!(MixerContext),
        hMixer,
        MixerId as ULONG,
        Flags,
        MixDetails,
    ) as u32 == MM_STATUS_SUCCESS as u32
    {
        return MMSYSERR_NOERROR as MMRESULT;
    }

    return MMSYSERR_ERROR as MMRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudGetWaveOutCapabilities(
    mut DeviceId: ULONG,
    mut Capabilities: LPWAVEOUTCAPSW,
) -> MMRESULT {
    if MMixerWaveOutCapabilities(core::ptr::addr_of_mut!(MixerContext), DeviceId, Capabilities) as ::core::ffi::c_uint
        == MM_STATUS_SUCCESS as ::core::ffi::c_uint
    {
        return MMSYSERR_NOERROR as MMRESULT;
    }

    return MMSYSERR_ERROR as MMRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudGetWaveInCapabilities(
    DeviceId: ULONG,
    Capabilities: LPWAVEINCAPSW,
) -> MMRESULT {
    if MMixerWaveInCapabilities(core::ptr::addr_of_mut!(MixerContext), DeviceId, Capabilities) == MM_STATUS_SUCCESS {
        return MMSYSERR_NOERROR;
    }

    return MMSYSERR_ERROR;
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudSetWaveDeviceFormatByMMixer(Instance: PSOUND_DEVICE_INSTANCE, DeviceId: DWORD, WaveFormat: PWAVEFORMATEX, WaveFormatSize: DWORD) -> MMRESULT {
    let mut DeviceType: MMDEVICE_TYPE = 0;
    let mut SoundDevice: PSOUND_DEVICE = core::ptr::null_mut();
    let mut Result: MMRESULT;
    let mut bWaveIn: BOOL;

    Result = GetSoundDeviceFromInstance(Instance, core::ptr::addr_of_mut!(SoundDevice));

    if MMSUCCESS!(Result) == 0 {
        return TranslateInternalMmResult(Result);
    }

    Result = GetSoundDeviceType(SoundDevice, core::ptr::addr_of_mut!(DeviceType));
    SND_ASSERT!(Result == MMSYSERR_NOERROR);

    bWaveIn = if (DeviceType) as u32 == WAVE_IN_DEVICE_TYPE {
        (TRUE as i32)
    } else {
        (FALSE as i32)
    };

    if MMixerOpenWave(
        core::ptr::addr_of_mut!(MixerContext),
        DeviceId,
        (bWaveIn) as ULONG,
        WaveFormat,
        None,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!((*Instance).Handle),
    ) == MM_STATUS_SUCCESS
    {
        if (DeviceType) as u32 == WAVE_OUT_DEVICE_TYPE {
            MMixerSetWaveStatus(
                core::ptr::addr_of_mut!(MixerContext),
                (*Instance).Handle,
                KSSTATE_ACQUIRE,
            );
            MMixerSetWaveStatus(
                core::ptr::addr_of_mut!(MixerContext),
                (*Instance).Handle,
                KSSTATE_PAUSE,
            );
            MMixerSetWaveStatus(
                core::ptr::addr_of_mut!(MixerContext),
                (*Instance).Handle,
                KSSTATE_RUN,
            );
        }
        return MMSYSERR_NOERROR;
    }
    return MMSYSERR_ERROR;
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudGetCapabilitiesByMMixer(SoundDevice: PSOUND_DEVICE, DeviceId: DWORD, Capabilities: PVOID, CapabilitiesSize: DWORD) -> MMRESULT {
    let mut DeviceType: MMDEVICE_TYPE = 0;
    let mut Result: MMRESULT;

    Result = GetSoundDeviceType(SoundDevice, core::ptr::addr_of_mut!(DeviceType));
    SND_ASSERT!(Result == MMSYSERR_NOERROR);

    if (DeviceType) as u32 == MIXER_DEVICE_TYPE {
        return WdmAudGetMixerCapabilities(DeviceId as ULONG, Capabilities as LPMIXERCAPSW);
    } else if (DeviceType) as u32 == WAVE_OUT_DEVICE_TYPE {
        return WdmAudGetWaveOutCapabilities(DeviceId as ULONG, Capabilities as LPWAVEOUTCAPSW);
    } else if (DeviceType) as u32 == WAVE_IN_DEVICE_TYPE {
        return WdmAudGetWaveInCapabilities(DeviceId as ULONG, Capabilities as LPWAVEINCAPSW);
    } else {
        // not supported
        return MMSYSERR_ERROR;
    }
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudOpenSoundDeviceByMMixer(mut SoundDevice: *mut _SOUND_DEVICE, mut Handle: *mut PVOID) -> MMRESULT {
    if WdmAudInitUserModeMixer() != 0 {
        return MMSYSERR_NOERROR;
    } else {
        return MMSYSERR_ERROR;
    }
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudCloseSoundDeviceByMMixer(mut SoundDeviceInstance: *mut _SOUND_DEVICE_INSTANCE, mut Handle: PVOID) -> MMRESULT {
    let mut DeviceType: MMDEVICE_TYPE = 0;
    let mut SoundDevice: PSOUND_DEVICE = core::ptr::null_mut();
    let mut Result: MMRESULT = 0;

    Result = GetSoundDeviceFromInstance(SoundDeviceInstance, core::ptr::addr_of_mut!(SoundDevice));

    if MMSUCCESS!(Result) == 0 {
        return TranslateInternalMmResult(Result);
    }

    Result = GetSoundDeviceType(SoundDevice, core::ptr::addr_of_mut!(DeviceType));
    SND_ASSERT!(Result == MMSYSERR_NOERROR);

    if (DeviceType) as u32 == MIXER_DEVICE_TYPE {
        /* no op */
        return MMSYSERR_NOERROR;
    } else if (DeviceType) as u32 == WAVE_IN_DEVICE_TYPE || (DeviceType) as u32 == WAVE_OUT_DEVICE_TYPE {
        /* make sure the pin is stopped */
        MMixerSetWaveStatus(
            core::ptr::addr_of_mut!(MixerContext),
            (*SoundDeviceInstance).Handle as HANDLE,
            KSSTATE_PAUSE,
        );
        MMixerSetWaveStatus(
            core::ptr::addr_of_mut!(MixerContext),
            (*SoundDeviceInstance).Handle as HANDLE,
            KSSTATE_ACQUIRE,
        );
        MMixerSetWaveStatus(
            core::ptr::addr_of_mut!(MixerContext),
            (*SoundDeviceInstance).Handle as HANDLE,
            KSSTATE_STOP,
        );

        CloseHandle(Handle as HANDLE);
        return MMSYSERR_NOERROR;
    }

    /* midi is not supported */
    return MMSYSERR_ERROR;
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudGetNumWdmDevsByMMixer(
    DeviceType: MMDEVICE_TYPE,
    DeviceCount: *mut DWORD,
) -> MMRESULT {
    if ((DeviceType as ::core::ffi::c_int)) as u32 == MIXER_DEVICE_TYPE {
        *DeviceCount = MMixerGetCount(core::ptr::addr_of_mut!(MixerContext)) as DWORD;
    } else if ((DeviceType as ::core::ffi::c_int)) as u32 == WAVE_OUT_DEVICE_TYPE {
        *DeviceCount = MMixerGetWaveOutCount(core::ptr::addr_of_mut!(MixerContext)) as DWORD;
    } else if ((DeviceType as ::core::ffi::c_int)) as u32 == WAVE_IN_DEVICE_TYPE {
        *DeviceCount = MMixerGetWaveInCount(core::ptr::addr_of_mut!(MixerContext)) as DWORD;
    } else {
        *DeviceCount = 0;
    }
    return MMSYSERR_NOERROR;
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudQueryMixerInfoByMMixer(mut SoundDeviceInstance: *mut _SOUND_DEVICE_INSTANCE, mut MixerId: DWORD, mut uMsg: UINT, mut Parameter: LPVOID, mut Flags: DWORD) -> MMRESULT {
    let mut MixLine: LPMIXERLINEW = core::ptr::null_mut();
    let mut MixControls: LPMIXERLINECONTROLSW = core::ptr::null_mut();
    let mut MixDetails: LPMIXERCONTROLDETAILS = core::ptr::null_mut();
    let mut hMixer: HANDLE = core::ptr::null_mut();

    MixLine = Parameter as LPMIXERLINEW;
    MixControls = Parameter as LPMIXERLINECONTROLSW;
    MixDetails = Parameter as LPMIXERCONTROLDETAILS;

    /* FIXME param checks */

    if !SoundDeviceInstance.is_null() {
        hMixer = (*SoundDeviceInstance).Handle as HANDLE;
    }

    match uMsg {
        MXDM_GETLINEINFO => return WdmAudGetLineInfo(hMixer, MixerId, MixLine, Flags as ULONG),
        MXDM_GETLINECONTROLS => return WdmAudGetLineControls(hMixer, MixerId, MixControls, Flags as ULONG),
        MXDM_SETCONTROLDETAILS => return WdmAudSetControlDetails(hMixer, MixerId, MixDetails, Flags as ULONG),
        MXDM_GETCONTROLDETAILS => return WdmAudGetControlDetails(hMixer, MixerId, MixDetails, Flags as ULONG),
        _ => {
            DPRINT1!("MixerId %lu, uMsg %lu, Parameter %p, Flags %lu\n", MixerId, uMsg, Parameter, Flags);
            SND_ASSERT!(0);
            return MMSYSERR_NOTSUPPORTED;
        }
    };
}

#[no_mangle]
pub unsafe extern "C" fn WdmAudGetDeviceInterfaceStringByMMixer(DeviceType: MMDEVICE_TYPE, DeviceId: DWORD, Interface: LPWSTR, InterfaceLength: DWORD, InterfaceSize: *mut DWORD) -> MMRESULT {
    MMSYSERR_NOTSUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn MixerEventCallback(
    MixerEventContext: PVOID,
    hMixer: HANDLE,
    NotificationType: ULONG,
    Value: ULONG,
) {
    let Instance: PSOUND_DEVICE_INSTANCE = MixerEventContext as PSOUND_DEVICE_INSTANCE;

    DriverCallback(
        (*Instance).WinMM.ClientCallback,
        (HIWORD!((*Instance).WinMM.Flags)) as DWORD,
        (*Instance).WinMM.Handle,
        NotificationType,
        (*Instance).WinMM.ClientCallbackInstanceData,
        Value as DWORD_PTR,
        0,
    );
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudSetMixerDeviceFormatByMMixer(mut Instance: PSOUND_DEVICE_INSTANCE, mut DeviceId: DWORD, mut WaveFormat: PWAVEFORMATEX, mut WaveFormatSize: DWORD) -> MMRESULT {
    if MMixerOpen(
        core::ptr::addr_of_mut!(MixerContext),
        DeviceId as ULONG,
        Instance as PVOID,
        Some(MixerEventCallback),
        core::ptr::addr_of_mut!((*Instance).Handle),
    ) as ::core::ffi::c_uint == MM_STATUS_SUCCESS as ::core::ffi::c_uint
    {
        return MMSYSERR_NOERROR as MMRESULT;
    }
    return MMSYSERR_BADDEVICEID as MMRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudSetWaveStateByMMixer(SoundDeviceInstance: *mut _SOUND_DEVICE_INSTANCE, bStart: BOOL) -> MMRESULT {
    let mut DeviceType: MMDEVICE_TYPE = core::mem::zeroed();
    let mut SoundDevice: PSOUND_DEVICE = core::ptr::null_mut();
    let mut Result: MMRESULT;

    Result = GetSoundDeviceFromInstance(SoundDeviceInstance, core::ptr::addr_of_mut!(SoundDevice));
    SND_ASSERT!(Result == MMSYSERR_NOERROR);

    Result = GetSoundDeviceType(SoundDevice, core::ptr::addr_of_mut!(DeviceType));
    SND_ASSERT!(Result == MMSYSERR_NOERROR);

    if (DeviceType) as u32 == WAVE_IN_DEVICE_TYPE || (DeviceType) as u32 == WAVE_OUT_DEVICE_TYPE {
        if bStart != 0 {
            MMixerSetWaveStatus(
                core::ptr::addr_of_mut!(MixerContext),
                (*SoundDeviceInstance).Handle as HANDLE,
                KSSTATE_ACQUIRE,
            );
            MMixerSetWaveStatus(
                core::ptr::addr_of_mut!(MixerContext),
                (*SoundDeviceInstance).Handle as HANDLE,
                KSSTATE_PAUSE,
            );
            MMixerSetWaveStatus(
                core::ptr::addr_of_mut!(MixerContext),
                (*SoundDeviceInstance).Handle as HANDLE,
                KSSTATE_RUN,
            );
        } else {
            MMixerSetWaveStatus(
                core::ptr::addr_of_mut!(MixerContext),
                (*SoundDeviceInstance).Handle as HANDLE,
                KSSTATE_PAUSE,
            );
            MMixerSetWaveStatus(
                core::ptr::addr_of_mut!(MixerContext),
                (*SoundDeviceInstance).Handle as HANDLE,
                KSSTATE_ACQUIRE,
            );
            MMixerSetWaveStatus(
                core::ptr::addr_of_mut!(MixerContext),
                (*SoundDeviceInstance).Handle as HANDLE,
                KSSTATE_STOP,
            );
        }
    } else if (DeviceType) as u32 == MIDI_IN_DEVICE_TYPE || (DeviceType) as u32 == MIDI_OUT_DEVICE_TYPE {
        if bStart != 0 {
            MMixerSetMidiStatus(
                core::ptr::addr_of_mut!(MixerContext),
                (*SoundDeviceInstance).Handle as HANDLE,
                KSSTATE_ACQUIRE,
            );
            MMixerSetMidiStatus(
                core::ptr::addr_of_mut!(MixerContext),
                (*SoundDeviceInstance).Handle as HANDLE,
                KSSTATE_PAUSE,
            );
            MMixerSetMidiStatus(
                core::ptr::addr_of_mut!(MixerContext),
                (*SoundDeviceInstance).Handle as HANDLE,
                KSSTATE_RUN,
            );
        } else {
            MMixerSetMidiStatus(
                core::ptr::addr_of_mut!(MixerContext),
                (*SoundDeviceInstance).Handle as HANDLE,
                KSSTATE_PAUSE,
            );
            MMixerSetMidiStatus(
                core::ptr::addr_of_mut!(MixerContext),
                (*SoundDeviceInstance).Handle as HANDLE,
                KSSTATE_ACQUIRE,
            );
            MMixerSetMidiStatus(
                core::ptr::addr_of_mut!(MixerContext),
                (*SoundDeviceInstance).Handle as HANDLE,
                KSSTATE_STOP,
            );
        }
    }

    return MMSYSERR_NOERROR;
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudResetStreamByMMixer(mut SoundDeviceInstance: *mut _SOUND_DEVICE_INSTANCE, mut DeviceType: MMDEVICE_TYPE, mut bStartReset: BOOLEAN) -> MMRESULT {
    let mut Status: MIXER_STATUS;

    if (DeviceType) as u32 == WAVE_IN_DEVICE_TYPE || (DeviceType) as u32 == WAVE_OUT_DEVICE_TYPE {
        Status = MMixerSetWaveResetState(
            core::ptr::addr_of_mut!(MixerContext),
            (*SoundDeviceInstance).Handle as HANDLE,
            bStartReset as ULONG,
        );
        if Status == MM_STATUS_SUCCESS {
            /* completed successfully */
            return MMSYSERR_NOERROR;
        }
    }

    return MMSYSERR_NOTSUPPORTED;
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudGetWavePositionByMMixer(mut SoundDeviceInstance: *mut _SOUND_DEVICE_INSTANCE, mut Time: *mut MMTIME) -> MMRESULT {
    let mut SoundDevice: PSOUND_DEVICE = core::ptr::null_mut();
    let mut DeviceType: MMDEVICE_TYPE = 0;
    let mut Status: MIXER_STATUS = 0;
    let mut Result: MMRESULT = 0;
    let mut Position: DWORD = 0;

    Result = GetSoundDeviceFromInstance(
        SoundDeviceInstance as PSOUND_DEVICE_INSTANCE,
        core::ptr::addr_of_mut!(SoundDevice),
    );
    if ((MMSUCCESS!(Result)) == 0) {
        return TranslateInternalMmResult(Result);
    }

    Result = GetSoundDeviceType(SoundDevice, core::ptr::addr_of_mut!(DeviceType));
    SND_ASSERT!(Result == MMSYSERR_NOERROR as MMRESULT);

    if DeviceType == WAVE_IN_DEVICE_TYPE as MMDEVICE_TYPE
        || DeviceType == WAVE_OUT_DEVICE_TYPE as MMDEVICE_TYPE
    {
        Status = MMixerGetWavePosition(
            core::ptr::addr_of_mut!(MixerContext),
            (*SoundDeviceInstance).Handle as HANDLE,
            core::ptr::addr_of_mut!(Position),
        );
        if Status == MM_STATUS_SUCCESS as MIXER_STATUS {
            /* Store position */
            (*Time).wType = TIME_BYTES as UINT;
            (*Time).u.cb = Position;

            /* Completed successfully */
            return MMSYSERR_NOERROR as MMRESULT;
        }
    }
    return MMSYSERR_NOTSUPPORTED as MMRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudGetVolumeByMMixer(SoundDeviceInstance: PSOUND_DEVICE_INSTANCE, DeviceId: DWORD, pdwVolume: PDWORD) -> MMRESULT {
    let mut Result: MMRESULT;
    let mut MixLine: MIXERLINE = core::mem::zeroed();
    let mut MixControl: MIXERCONTROL = core::mem::zeroed();
    let mut MixLineControls: MIXERLINECONTROLS = core::mem::zeroed();
    let mut MixControlDetails: MIXERCONTROLDETAILS = core::mem::zeroed();
    let mut MixControlDetailsU: [MIXERCONTROLDETAILS_UNSIGNED; 2] = core::mem::zeroed(); // For 2 (stereo) channels

    MixLine.cbStruct = core::mem::size_of::<MIXERLINE>() as DWORD;
    MixLine.dwComponentType = MIXERLINE_COMPONENTTYPE_DST_SPEAKERS as DWORD;

    /* Get line info */
    Result = WdmAudGetLineInfo(
        (*SoundDeviceInstance).Handle as HANDLE,
        DeviceId,
        core::ptr::addr_of_mut!(MixLine),
        (MIXER_OBJECTF_MIXER | MIXER_GETLINEINFOF_COMPONENTTYPE) as ULONG,
    );
    if ((MMSUCCESS!(Result)) == 0) {
        return TranslateInternalMmResult(Result);
    }

    MixLineControls.cbStruct = core::mem::size_of::<MIXERLINECONTROLS>() as DWORD;
    MixLineControls.dwLineID = MixLine.dwLineID;
    MixLineControls.__bindgen_anon_1.dwControlType = MIXERCONTROL_CONTROLTYPE_VOLUME as DWORD;
    MixLineControls.cControls = 1 as DWORD;
    MixLineControls.cbmxctrl = core::mem::size_of::<MIXERCONTROL>() as DWORD;
    MixLineControls.pamxctrl = core::ptr::addr_of_mut!(MixControl) as LPMIXERCONTROLW;

    /* Get line controls */
    Result = WdmAudGetLineControls(
        (*SoundDeviceInstance).Handle as HANDLE,
        DeviceId,
        core::ptr::addr_of_mut!(MixLineControls),
        (MIXER_OBJECTF_MIXER | MIXER_GETLINECONTROLSF_ONEBYTYPE) as ULONG,
    );
    if ((MMSUCCESS!(Result)) == 0) {
        return TranslateInternalMmResult(Result);
    }

    MixControlDetails.cbStruct = core::mem::size_of::<MIXERCONTROLDETAILS>() as DWORD;
    MixControlDetails.dwControlID = MixControl.dwControlID;
    MixControlDetails.cChannels = MixLine.cChannels;
    MixControlDetails.__bindgen_anon_1.cMultipleItems = 0 as DWORD;
    MixControlDetails.cbDetails = core::mem::size_of::<MIXERCONTROLDETAILS_UNSIGNED>() as DWORD;
    MixControlDetails.paDetails = core::ptr::addr_of_mut!(MixControlDetailsU) as *mut MIXERCONTROLDETAILS_UNSIGNED as PVOID;

    /* Get volume control details */
    Result = WdmAudGetControlDetails(
        (*SoundDeviceInstance).Handle as HANDLE,
        DeviceId,
        core::ptr::addr_of_mut!(MixControlDetails),
        MIXER_OBJECTF_MIXER as ULONG,
    );
    if ((MMSUCCESS!(Result)) != 0) {
        *pdwVolume = (MAKELONG!(
            LOWORD!(MixControlDetailsU[0 as usize].dwValue),
            HIWORD!(MixControlDetailsU[1 as usize].dwValue),
        )) as u32;
    }

    return Result;
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudSetVolumeByMMixer(SoundDeviceInstance: PSOUND_DEVICE_INSTANCE, DeviceId: DWORD, dwVolume: DWORD) -> MMRESULT {
    let mut Result: MMRESULT;
    let mut MixLine: MIXERLINE = core::mem::zeroed();
    let mut MixControl: MIXERCONTROL = core::mem::zeroed();
    let mut MixLineControls: MIXERLINECONTROLS = core::mem::zeroed();
    let mut MixControlDetails: MIXERCONTROLDETAILS = core::mem::zeroed();
    let mut MixControlDetailsU: [MIXERCONTROLDETAILS_UNSIGNED; 2] = core::mem::zeroed();

    MixLine.cbStruct = core::mem::size_of::<MIXERLINE>() as DWORD;
    MixLine.dwComponentType = MIXERLINE_COMPONENTTYPE_DST_SPEAKERS as DWORD;

    /* Get line info */
    Result = WdmAudGetLineInfo((*SoundDeviceInstance).Handle as HANDLE,
                               DeviceId,
                               core::ptr::addr_of_mut!(MixLine),
                               (MIXER_OBJECTF_MIXER | MIXER_GETLINEINFOF_COMPONENTTYPE) as ULONG);
    if ((MMSUCCESS!(Result)) == 0) {
        return TranslateInternalMmResult(Result);
    }

    MixLineControls.cbStruct = core::mem::size_of::<MIXERLINECONTROLS>() as DWORD;
    MixLineControls.dwLineID = MixLine.dwLineID;
    MixLineControls.__bindgen_anon_1.dwControlType = MIXERCONTROL_CONTROLTYPE_VOLUME as DWORD;
    MixLineControls.cControls = 1 as DWORD;
    MixLineControls.cbmxctrl = core::mem::size_of::<MIXERCONTROL>() as DWORD;
    MixLineControls.pamxctrl = core::ptr::addr_of_mut!(MixControl) as LPMIXERCONTROLW;

    /* Get line controls */
    Result = WdmAudGetLineControls((*SoundDeviceInstance).Handle as HANDLE,
                                   DeviceId,
                                   core::ptr::addr_of_mut!(MixLineControls),
                                   (MIXER_OBJECTF_MIXER | MIXER_GETLINECONTROLSF_ONEBYTYPE) as ULONG);
    if ((MMSUCCESS!(Result)) == 0) {
        return TranslateInternalMmResult(Result);
    }

    /* Convert volume level to be set */
    MixControlDetailsU[0].dwValue = LOWORD!(dwVolume) as DWORD; // Left channel
    MixControlDetailsU[1].dwValue = HIWORD!(dwVolume) as DWORD; // Right channel

    MixControlDetails.cbStruct = core::mem::size_of::<MIXERCONTROLDETAILS>() as DWORD;
    MixControlDetails.dwControlID = MixControl.dwControlID;
    MixControlDetails.cChannels = MixLine.cChannels;
    MixControlDetails.__bindgen_anon_1.cMultipleItems = 0 as DWORD;
    MixControlDetails.cbDetails = core::mem::size_of::<MIXERCONTROLDETAILS_UNSIGNED>() as DWORD;
    MixControlDetails.paDetails = core::ptr::addr_of_mut!(MixControlDetailsU) as PVOID;

    /* Set volume control details */
    Result = WdmAudSetControlDetails((*SoundDeviceInstance).Handle as HANDLE,
                                     DeviceId,
                                     core::ptr::addr_of_mut!(MixControlDetails),
                                     MIXER_OBJECTF_MIXER as ULONG);
    return Result;
}

unsafe extern "system" fn CommitWaveBufferApc(
    mut ApcContext: PVOID,
    mut IoStatusBlock: PIO_STATUS_BLOCK,
    mut Reserved: ULONG,
) {
    let mut dwErrorCode: DWORD = 0;
    let mut Overlap: PSOUND_OVERLAPPED = core::ptr::null_mut();
    let mut lpHeader: *mut KSSTREAM_HEADER = core::ptr::null_mut();

    dwErrorCode = RtlNtStatusToDosError((*IoStatusBlock).__bindgen_anon_1.Status) as DWORD;
    Overlap = IoStatusBlock as PSOUND_OVERLAPPED;
    lpHeader = (*Overlap).CompletionContext as *mut KSSTREAM_HEADER;

    /* Call mmebuddy overlap routine */
    (*Overlap)
        .OriginalCompletionRoutine
        .unwrap_unchecked()(
        dwErrorCode,
        (*lpHeader).DataUsed as DWORD,
        &raw mut (*Overlap).Standard,
    );

    HeapFree(GetProcessHeap(), 0 as DWORD, lpHeader as PVOID);
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudCommitWaveBufferByMMixer(SoundDeviceInstance: PSOUND_DEVICE_INSTANCE, OffsetPtr: PVOID, Length: DWORD, Overlap: PSOUND_OVERLAPPED, CompletionRoutine: LPOVERLAPPED_COMPLETION_ROUTINE) -> MMRESULT {
    let mut SoundDevice: PSOUND_DEVICE = core::ptr::null_mut();
    let mut DeviceType: MMDEVICE_TYPE = 0;
    let mut Result: MMRESULT = 0;
    let mut IoCtl: ULONG = 0;
    let mut lpHeader: *mut KSSTREAM_HEADER = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;

    Result = GetSoundDeviceFromInstance(SoundDeviceInstance, core::ptr::addr_of_mut!(SoundDevice));

    if ((MMSUCCESS!(Result)) == 0) {
        return TranslateInternalMmResult(Result);
    }

    Result = GetSoundDeviceType(SoundDevice, core::ptr::addr_of_mut!(DeviceType));
    SND_ASSERT!(Result == MMSYSERR_NOERROR as MMRESULT);

    lpHeader = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<KSSTREAM_HEADER>() as SIZE_T,
    ) as *mut KSSTREAM_HEADER;
    if lpHeader.is_null() {
        /* no memory */
        return MMSYSERR_NOMEM as MMRESULT;
    }

    /* setup stream packet */
    (*lpHeader).Size = core::mem::size_of::<KSSTREAM_HEADER>() as ULONG;
    (*lpHeader).PresentationTime.Numerator = 1 as ULONG;
    (*lpHeader).PresentationTime.Denominator = 1 as ULONG;
    (*lpHeader).Data = OffsetPtr;
    (*lpHeader).FrameExtent = Length as ULONG;
    (*Overlap).CompletionContext = lpHeader as PVOID;
    (*Overlap).OriginalCompletionRoutine = CompletionRoutine;
    IoCtl = (if DeviceType as ::core::ffi::c_int == WAVE_OUT_DEVICE_TYPE as ::core::ffi::c_int {
        IOCTL_KS_WRITE_STREAM
    } else {
        IOCTL_KS_READ_STREAM
    }) as ULONG;

    if DeviceType as ::core::ffi::c_int == WAVE_OUT_DEVICE_TYPE as ::core::ffi::c_int {
        (*lpHeader).DataUsed = Length as ULONG;
    }

    Status = NtDeviceIoControlFile(
        (*SoundDeviceInstance).Handle as HANDLE,
        core::ptr::null_mut(),
        Some(CommitWaveBufferApc),
        core::ptr::null_mut(),
        Overlap as PIO_STATUS_BLOCK,
        IoCtl,
        core::ptr::null_mut(),
        0 as ULONG,
        lpHeader as PVOID,
        core::mem::size_of::<KSSTREAM_HEADER>() as ULONG,
    );

    if !NT_SUCCESS!(Status) {
        DPRINT1!("NtDeviceIoControlFile() failed with status %08lx\n", Status);
        return MMSYSERR_ERROR as MMRESULT;
    }

    return MMSYSERR_NOERROR as MMRESULT;
}
