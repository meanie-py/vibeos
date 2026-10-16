// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_2B3E7AFAF5DE: [u16; 18] = [
    0x0075u16, 0x004D, 0x0073, 0x0067, 0x0020, 0x0025, 0x0078, 0x0020, 0x0046, 0x006C,
    0x0061, 0x0067, 0x0073, 0x0020, 0x0025, 0x0078, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_37DADC391C83: [u16; 61] = [
    0x0057u16, 0x0044, 0x004D, 0x0041, 0x0055, 0x0044, 0x0020, 0x002D, 0x0020, 0x0047,
    0x0065, 0x0074, 0x0057, 0x0064, 0x006D, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063,
    0x0065, 0x0043, 0x0061, 0x0070, 0x0061, 0x0062, 0x0069, 0x006C, 0x0069, 0x0074,
    0x0069, 0x0065, 0x0073, 0x0020, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065,
    0x0054, 0x0079, 0x0070, 0x0065, 0x0020, 0x0025, 0x0075, 0x0020, 0x0044, 0x0065,
    0x0076, 0x0069, 0x0063, 0x0065, 0x0049, 0x0064, 0x0020, 0x0025, 0x0075, 0x000A,
    0x0000,
];
pub static RUSTCONV_WSTR_7AE505E71B2A: [u16; 28] = [
    0x004Fu16, 0x0070, 0x0065, 0x006E, 0x0069, 0x006E, 0x0067, 0x0020, 0x0077, 0x0064,
    0x006D, 0x0061, 0x0075, 0x0064, 0x0020, 0x0064, 0x0065, 0x0076, 0x0069, 0x0063,
    0x0065, 0x0020, 0x0027, 0x0025, 0x0073, 0x0027, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_9C538FEECA8F: [u16; 38] = [
    0x0043u16, 0x0061, 0x006C, 0x006C, 0x0020, 0x0074, 0x006F, 0x0020, 0x0049, 0x004F,
    0x0043, 0x0054, 0x004C, 0x005F, 0x0047, 0x0045, 0x0054, 0x004E, 0x0055, 0x004D,
    0x0044, 0x0045, 0x0056, 0x0053, 0x005F, 0x0054, 0x0059, 0x0050, 0x0045, 0x0020,
    0x0066, 0x0061, 0x0069, 0x006C, 0x0065, 0x0064, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_CB2B95ECFA77: [u16; 29] = [
    0x0046u16, 0x0072, 0x0061, 0x006D, 0x0065, 0x0053, 0x0069, 0x007A, 0x0065, 0x0020,
    0x0025, 0x0075, 0x0020, 0x0042, 0x0075, 0x0066, 0x0066, 0x0065, 0x0072, 0x0043,
    0x006F, 0x0075, 0x006E, 0x0074, 0x0020, 0x0025, 0x0075, 0x000A, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut KernelHandle: HANDLE = unsafe { INVALID_HANDLE_VALUE };

#[no_mangle]
pub static mut OpenCount: DWORD = 0 as DWORD;

pub const NDEBUG: () = ();

pub const KERNEL_DEVICE_NAME: &[u16] = &[
    b'\\' as u16, b'\\' as u16, b'.' as u16, b'\\' as u16,
    b'w' as u16, b'd' as u16, b'm' as u16, b'a' as u16, b'u' as u16, b'd' as u16, 0u16,
];


#[no_mangle]
pub unsafe extern "system" fn MixerEventThreadRoutine(Parameter: LPVOID) -> DWORD {
    let mut WaitObjects: [HANDLE; 2] = [core::ptr::null_mut(); 2];
    let mut dwResult: DWORD;
    let mut Result: MMRESULT;
    let mut DeviceInfo: WDMAUD_DEVICE_INFO = core::mem::zeroed();
    let Instance: PSOUND_DEVICE_INSTANCE = Parameter as PSOUND_DEVICE_INSTANCE;

    /* setup wait objects */
    WaitObjects[0 as usize] = (*Instance).hNotifyEvent;
    WaitObjects[1 as usize] = (*Instance).hStopEvent;

    /* zero device info */
    ZeroMemory!(
        core::ptr::addr_of_mut!(DeviceInfo) as *mut core::ffi::c_void,
        (core::mem::size_of::<WDMAUD_DEVICE_INFO>()) as usize,
    );

    DeviceInfo.hDevice = (*Instance).Handle;
    DeviceInfo.DeviceType = MIXER_DEVICE_TYPE as SOUND_DEVICE_TYPE;

    loop {
        dwResult = WaitForMultipleObjects(
            2 as DWORD,
            core::ptr::addr_of_mut!(WaitObjects) as *mut HANDLE,
            FALSE as BOOL,
            INFINITE as DWORD,
        );

        if dwResult == (WAIT_OBJECT_0 + 1) as DWORD {
            /* stop event was signalled */
            break;
        }

        loop {
            Result = SyncOverlappedDeviceIoControl(
                KernelHandle,
                IOCTL_GET_MIXER_EVENT as DWORD,
                core::ptr::addr_of_mut!(DeviceInfo) as LPVOID,
                (core::mem::size_of::<WDMAUD_DEVICE_INFO>()) as DWORD,
                core::ptr::addr_of_mut!(DeviceInfo) as LPVOID,
                (core::mem::size_of::<WDMAUD_DEVICE_INFO>()) as DWORD,
                core::ptr::null_mut::<DWORD>(),
            );

            if Result == MMSYSERR_NOERROR as MMRESULT {
                DriverCallback(
                    (*Instance).WinMM.ClientCallback,
                    (HIWORD!((*Instance).WinMM.Flags)) as DWORD,
                    (*Instance).WinMM.Handle,
                    DeviceInfo.u.MixerEvent.NotificationType,
                    (*Instance).WinMM.ClientCallbackInstanceData,
                    DeviceInfo.u.MixerEvent.Value as DWORD_PTR,
                    0 as DWORD_PTR,
                );
            }

            if !(Result == MMSYSERR_NOERROR as MMRESULT) {
                break;
            }
        }

        if !((TRUE) != 0) {
            break;
        }
    }

    /* done */
    return 0 as DWORD;
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudCleanupByLegacy() -> MMRESULT {
    if KernelHandle != INVALID_HANDLE_VALUE {
        CloseHandle(KernelHandle);
        KernelHandle = INVALID_HANDLE_VALUE;
    }

    return MMSYSERR_NOERROR;
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudGetNumWdmDevsByLegacy(
    mut DeviceType: MMDEVICE_TYPE,
    mut DeviceCount: *mut DWORD,
) -> MMRESULT {
    let mut Result: MMRESULT;
    let mut DeviceInfo: WDMAUD_DEVICE_INFO = core::mem::zeroed();
    if !(KernelHandle != INVALID_HANDLE_VALUE) {
        return 11 as MMRESULT; // MMSYSERR_INVALPARAM
    }
    if (((IS_VALID_SOUND_DEVICE_TYPE!(DeviceType))) == 0) {
        return 11 as MMRESULT; // MMSYSERR_INVALPARAM
    }
    if DeviceCount.is_null() {
        return 11 as MMRESULT; // MMSYSERR_INVALPARAM
    }

    ZeroMemory!(
        &raw mut DeviceInfo as *mut core::ffi::c_void,
        core::mem::size_of::<WDMAUD_DEVICE_INFO>() as usize,
    );
    DeviceInfo.DeviceType = (DeviceType) as _;

    Result = SyncOverlappedDeviceIoControl(
        KernelHandle,
        (IOCTL_GETNUMDEVS_TYPE) as DWORD,
        &raw mut DeviceInfo as LPVOID,
        core::mem::size_of::<WDMAUD_DEVICE_INFO>() as DWORD,
        &raw mut DeviceInfo as LPVOID,
        core::mem::size_of::<WDMAUD_DEVICE_INFO>() as DWORD,
        core::ptr::null_mut(),
    );

    if (((MMSUCCESS!(Result))) == 0) {
        SND_ERR!(RUSTCONV_WSTR_9C538FEECA8F.as_ptr());
        *DeviceCount = 0;
        return TranslateInternalMmResult(Result);
    }

    *DeviceCount = DeviceInfo.DeviceCount;

    return MMSYSERR_NOERROR;
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudGetCapabilitiesByLegacy(
    SoundDevice: PSOUND_DEVICE,
    DeviceId: DWORD,
    Capabilities: PVOID,
    CapabilitiesSize: DWORD,
) -> MMRESULT {
    let mut Result: MMRESULT = 0;
    let mut DeviceType: MMDEVICE_TYPE = 0;
    let mut DeviceInfo: WDMAUD_DEVICE_INFO = core::mem::zeroed();

    SND_ASSERT!(SoundDevice);
    SND_ASSERT!(Capabilities);

    Result = GetSoundDeviceType(SoundDevice, core::ptr::addr_of_mut!(DeviceType));
    SND_ASSERT!(Result == MMSYSERR_NOERROR);

    if MMSUCCESS!(Result) == 0 {
        return Result;
    }

    SND_TRACE!(RUSTCONV_WSTR_37DADC391C83.as_ptr(), DeviceType, DeviceId);

    ZeroMemory!(
        core::ptr::addr_of_mut!(DeviceInfo) as LPVOID,
        core::mem::size_of::<WDMAUD_DEVICE_INFO>(),
    );
    DeviceInfo.DeviceType = (DeviceType) as _;
    DeviceInfo.DeviceIndex = (DeviceId) as _;

    Result = SyncOverlappedDeviceIoControl(
        KernelHandle,
        (IOCTL_GETCAPABILITIES) as DWORD,
        core::ptr::addr_of_mut!(DeviceInfo) as LPVOID,
        core::mem::size_of::<WDMAUD_DEVICE_INFO>() as DWORD,
        core::ptr::addr_of_mut!(DeviceInfo) as LPVOID,
        core::mem::size_of::<WDMAUD_DEVICE_INFO>() as DWORD,
        core::ptr::null_mut(),
    );

    if MMSUCCESS!(Result) == 0 {
        return TranslateInternalMmResult(Result);
    }

    /* This is pretty much a big hack right now */
    match (DeviceType) as u32 {
        MIXER_DEVICE_TYPE => {
            let mut MixerCaps: LPMIXERCAPSW = Capabilities as LPMIXERCAPSW;

            DeviceInfo.u.MixCaps.szPname[(MAXPNAMELEN - 1) as usize] =
                0x0000u16;
            CopyWideString!(
                (*MixerCaps).szPname.as_mut_ptr(),
                DeviceInfo.u.MixCaps.szPname.as_mut_ptr(),
            );

            (*MixerCaps).cDestinations = DeviceInfo.u.MixCaps.cDestinations;
            (*MixerCaps).fdwSupport = DeviceInfo.u.MixCaps.fdwSupport;
            (*MixerCaps).vDriverVersion = DeviceInfo.u.MixCaps.vDriverVersion;
            (*MixerCaps).wMid = DeviceInfo.u.MixCaps.wMid;
            (*MixerCaps).wPid = DeviceInfo.u.MixCaps.wPid;
            break;
        }
        WAVE_OUT_DEVICE_TYPE => {
            let mut WaveOutCaps: LPWAVEOUTCAPSW = Capabilities as LPWAVEOUTCAPSW;

            DeviceInfo.u.WaveOutCaps.szPname[(MAXPNAMELEN - 1) as usize] =
                0x0000u16;
            (*WaveOutCaps).wMid = DeviceInfo.u.WaveOutCaps.wMid;
            (*WaveOutCaps).wPid = DeviceInfo.u.WaveOutCaps.wPid;

            (*WaveOutCaps).vDriverVersion = DeviceInfo.u.WaveOutCaps.vDriverVersion;
            CopyWideString!(
                (*WaveOutCaps).szPname.as_mut_ptr(),
                DeviceInfo.u.WaveOutCaps.szPname.as_mut_ptr(),
            );

            (*WaveOutCaps).dwFormats = DeviceInfo.u.WaveOutCaps.dwFormats;
            (*WaveOutCaps).wChannels = DeviceInfo.u.WaveOutCaps.wChannels;
            (*WaveOutCaps).dwSupport = DeviceInfo.u.WaveOutCaps.dwSupport;
            break;
        }
        WAVE_IN_DEVICE_TYPE => {
            let mut WaveInCaps: LPWAVEINCAPSW = Capabilities as LPWAVEINCAPSW;

            DeviceInfo.u.WaveInCaps.szPname[(MAXPNAMELEN - 1) as usize] =
                0x0000u16;

            (*WaveInCaps).wMid = DeviceInfo.u.WaveInCaps.wMid;
            (*WaveInCaps).wPid = DeviceInfo.u.WaveInCaps.wPid;

            (*WaveInCaps).vDriverVersion = DeviceInfo.u.WaveInCaps.vDriverVersion;
            CopyWideString!(
                (*WaveInCaps).szPname.as_mut_ptr(),
                DeviceInfo.u.WaveInCaps.szPname.as_mut_ptr(),
            );

            (*WaveInCaps).dwFormats = DeviceInfo.u.WaveInCaps.dwFormats;
            (*WaveInCaps).wChannels = DeviceInfo.u.WaveInCaps.wChannels;
            (*WaveInCaps).wReserved1 = 0;
            break;
        }
        MIDI_IN_DEVICE_TYPE => {
            let mut MidiInCaps: LPMIDIINCAPSW = Capabilities as LPMIDIINCAPSW;

            DeviceInfo.u.MidiInCaps.szPname[(MAXPNAMELEN - 1) as usize] =
                0x0000u16;

            (*MidiInCaps).vDriverVersion = DeviceInfo.u.MidiInCaps.vDriverVersion;
            (*MidiInCaps).wMid = DeviceInfo.u.MidiInCaps.wMid;
            (*MidiInCaps).wPid = DeviceInfo.u.MidiInCaps.wPid;
            (*MidiInCaps).dwSupport = DeviceInfo.u.MidiInCaps.dwSupport;

            CopyWideString!(
                (*MidiInCaps).szPname.as_mut_ptr(),
                DeviceInfo.u.MidiInCaps.szPname.as_mut_ptr(),
            );
            break;
        }
        MIDI_OUT_DEVICE_TYPE => {
            let mut MidiOutCaps: LPMIDIOUTCAPSW = Capabilities as LPMIDIOUTCAPSW;

            DeviceInfo.u.MidiOutCaps.szPname[(MAXPNAMELEN - 1) as usize] =
                0x0000u16;

            (*MidiOutCaps).vDriverVersion = DeviceInfo.u.MidiOutCaps.vDriverVersion;
            (*MidiOutCaps).wMid = DeviceInfo.u.MidiOutCaps.wMid;
            (*MidiOutCaps).wPid = DeviceInfo.u.MidiOutCaps.wPid;
            (*MidiOutCaps).dwSupport = DeviceInfo.u.MidiOutCaps.dwSupport;

            CopyWideString!(
                (*MidiOutCaps).szPname.as_mut_ptr(),
                DeviceInfo.u.MidiOutCaps.szPname.as_mut_ptr(),
            );
            break;
        }
        _ => {}
    }

    return MMSYSERR_NOERROR;
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudOpenSoundDeviceByLegacy(
    SoundDevice: PSOUND_DEVICE,
    Handle: *mut PVOID,
) -> MMRESULT {
    let mut hDevInfo: HDEVINFO = core::ptr::null_mut();
    let mut DeviceInterfaceData: SP_DEVICE_INTERFACE_DATA = core::mem::zeroed();
    let mut SWBusGuid: GUID = STATIC_KSCATEGORY_WDMAUD;
    let mut DeviceInterfaceDetailData: PSP_DEVICE_INTERFACE_DETAIL_DATA_W = core::ptr::null_mut();

    if KernelHandle == INVALID_HANDLE_VALUE {
        hDevInfo = SetupDiGetClassDevsW(
            &mut SWBusGuid,
            core::ptr::null(),
            core::ptr::null_mut(),
            DIGCF_DEVICEINTERFACE | DIGCF_PRESENT,
        );
        if hDevInfo.is_null() {
            // failed
            return MMSYSERR_ERROR;
        }

        DeviceInterfaceData.cbSize = core::mem::size_of::<SP_DEVICE_INTERFACE_DATA>() as DWORD;
        if SetupDiEnumDeviceInterfaces(
            hDevInfo,
            core::ptr::null_mut(),
            &mut SWBusGuid,
            0,
            &mut DeviceInterfaceData,
        ) == 0
        {
            // failed
            SetupDiDestroyDeviceInfoList(hDevInfo);
            return MMSYSERR_ERROR;
        }

        DeviceInterfaceDetailData = HeapAlloc(
            GetProcessHeap(),
            0,
            ((MAX_PATH as usize) * core::mem::size_of::<WCHAR>()
                + core::mem::size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>()) as SIZE_T,
        ) as PSP_DEVICE_INTERFACE_DETAIL_DATA_W;
        if DeviceInterfaceDetailData.is_null() {
            // failed
            SetupDiDestroyDeviceInfoList(hDevInfo);
            return MMSYSERR_ERROR;
        }

        (*DeviceInterfaceDetailData).cbSize =
            core::mem::size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>() as DWORD;
        if SetupDiGetDeviceInterfaceDetailW(
            hDevInfo,
            &mut DeviceInterfaceData,
            DeviceInterfaceDetailData,
            ((MAX_PATH as usize) * core::mem::size_of::<WCHAR>()
                + core::mem::size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>()) as DWORD,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) == 0
        {
            // failed
            HeapFree(
                GetProcessHeap(),
                0,
                DeviceInterfaceDetailData as PVOID,
            );
            SetupDiDestroyDeviceInfoList(hDevInfo);
            return MMSYSERR_ERROR;
        }
        SND_TRACE!(
            RUSTCONV_WSTR_7AE505E71B2A.as_ptr(),
            core::ptr::addr_of_mut!((*DeviceInterfaceDetailData).DevicePath) as *mut WCHAR,
        );
        KernelHandle = CreateFileW(
            core::ptr::addr_of_mut!((*DeviceInterfaceDetailData).DevicePath) as *mut WCHAR as LPCWSTR,
            GENERIC_READ | GENERIC_WRITE,
            0,
            core::ptr::null_mut(),
            OPEN_EXISTING,
            FILE_FLAG_OVERLAPPED,
            core::ptr::null_mut(),
        );

        HeapFree(
            GetProcessHeap(),
            0,
            DeviceInterfaceDetailData as PVOID,
        );
        SetupDiDestroyDeviceInfoList(hDevInfo);
    }

    if KernelHandle == INVALID_HANDLE_VALUE {
        return MMSYSERR_ERROR;
    }

    OpenCount = OpenCount.wrapping_add(1);
    return MMSYSERR_NOERROR;
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudCloseSoundDeviceByLegacy(
    SoundDeviceInstance: *mut _SOUND_DEVICE_INSTANCE,
    Handle: PVOID,
) -> MMRESULT {
    let mut DeviceInfo: WDMAUD_DEVICE_INFO = core::mem::zeroed();
    let mut Result: MMRESULT;
    let mut DeviceType: MMDEVICE_TYPE = 0;
    let mut SoundDevice: PSOUND_DEVICE = core::ptr::null_mut();

    Result = GetSoundDeviceFromInstance(SoundDeviceInstance, core::ptr::addr_of_mut!(SoundDevice));

    if ((MMSUCCESS!(Result)) == 0) {
        return TranslateInternalMmResult(Result);
    }

    if OpenCount == 0 {
        return MMSYSERR_NOERROR;
    }

    SND_ASSERT!(KernelHandle != INVALID_HANDLE_VALUE);

    Result = GetSoundDeviceType(SoundDevice, core::ptr::addr_of_mut!(DeviceType));
    SND_ASSERT!(Result == MMSYSERR_NOERROR);

    if (*SoundDeviceInstance).Handle != KernelHandle as PVOID {
        ZeroMemory!(
            core::ptr::addr_of_mut!(DeviceInfo) as PVOID,
            core::mem::size_of::<WDMAUD_DEVICE_INFO>() as usize,
        );

        DeviceInfo.DeviceType = DeviceType as SOUND_DEVICE_TYPE;
        DeviceInfo.hDevice = (*SoundDeviceInstance).Handle as HANDLE;

        /* First stop the stream */
        if (DeviceType) as u32 != MIXER_DEVICE_TYPE {
            DeviceInfo.u.State = KSSTATE_PAUSE;
            SyncOverlappedDeviceIoControl(
                KernelHandle,
                (IOCTL_SETDEVICE_STATE) as DWORD,
                core::ptr::addr_of_mut!(DeviceInfo) as LPVOID,
                core::mem::size_of::<WDMAUD_DEVICE_INFO>() as DWORD,
                core::ptr::addr_of_mut!(DeviceInfo) as LPVOID,
                core::mem::size_of::<WDMAUD_DEVICE_INFO>() as DWORD,
                core::ptr::null_mut(),
            );

            DeviceInfo.u.State = KSSTATE_ACQUIRE;
            SyncOverlappedDeviceIoControl(
                KernelHandle,
                (IOCTL_SETDEVICE_STATE) as DWORD,
                core::ptr::addr_of_mut!(DeviceInfo) as LPVOID,
                core::mem::size_of::<WDMAUD_DEVICE_INFO>() as DWORD,
                core::ptr::addr_of_mut!(DeviceInfo) as LPVOID,
                core::mem::size_of::<WDMAUD_DEVICE_INFO>() as DWORD,
                core::ptr::null_mut(),
            );

            DeviceInfo.u.State = KSSTATE_STOP;
            SyncOverlappedDeviceIoControl(
                KernelHandle,
                (IOCTL_SETDEVICE_STATE) as DWORD,
                core::ptr::addr_of_mut!(DeviceInfo) as LPVOID,
                core::mem::size_of::<WDMAUD_DEVICE_INFO>() as DWORD,
                core::ptr::addr_of_mut!(DeviceInfo) as LPVOID,
                core::mem::size_of::<WDMAUD_DEVICE_INFO>() as DWORD,
                core::ptr::null_mut(),
            );
        }

        SyncOverlappedDeviceIoControl(
            KernelHandle,
            (IOCTL_CLOSE_WDMAUD) as DWORD,
            core::ptr::addr_of_mut!(DeviceInfo) as LPVOID,
            core::mem::size_of::<WDMAUD_DEVICE_INFO>() as DWORD,
            core::ptr::addr_of_mut!(DeviceInfo) as LPVOID,
            core::mem::size_of::<WDMAUD_DEVICE_INFO>() as DWORD,
            core::ptr::null_mut(),
        );
    }

    if (DeviceType) as u32 == MIXER_DEVICE_TYPE {
        SetEvent((*SoundDeviceInstance).hStopEvent);
        CloseHandle((*SoundDeviceInstance).hStopEvent);
        CloseHandle((*SoundDeviceInstance).hNotifyEvent);
    }

    OpenCount = OpenCount.wrapping_sub(1);

    if OpenCount < 1 {
        CloseHandle(KernelHandle);
        KernelHandle = INVALID_HANDLE_VALUE;
    }

    return MMSYSERR_NOERROR;
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudSetMixerDeviceFormatByLegacy(
    mut Instance: PSOUND_DEVICE_INSTANCE,
    mut DeviceId: DWORD,
    mut WaveFormat: PWAVEFORMATEX,
    mut WaveFormatSize: DWORD,
) -> MMRESULT {
    let mut Result: MMRESULT;
    let mut DeviceInfo: WDMAUD_DEVICE_INFO = core::mem::zeroed();
    let mut hThread: HANDLE;

    (*Instance).hNotifyEvent = CreateEventW(
        core::ptr::null_mut(),
        (FALSE) as BOOL,
        (FALSE) as BOOL,
        core::ptr::null(),
    );
    if !(*Instance).hNotifyEvent.is_null() == false {
        return MMSYSERR_NOMEM;
    }

    if (*Instance).Handle != core::ptr::null_mut() {
        /* device is already open */
        return MMSYSERR_NOERROR;
    }

    (*Instance).hStopEvent = CreateEventW(
        core::ptr::null_mut(),
        (FALSE) as BOOL,
        (FALSE) as BOOL,
        core::ptr::null(),
    );
    if !(*Instance).hStopEvent.is_null() == false {
        return MMSYSERR_NOMEM;
    }

    ZeroMemory!(
        core::ptr::addr_of_mut!(DeviceInfo) as *mut core::ffi::c_void,
        core::mem::size_of::<WDMAUD_DEVICE_INFO>(),
    );
    DeviceInfo.DeviceType = MIXER_DEVICE_TYPE;
    DeviceInfo.DeviceIndex = DeviceId as ULONG_PTR;
    DeviceInfo.u.hNotifyEvent = (*Instance).hNotifyEvent;

    Result = SyncOverlappedDeviceIoControl(
        KernelHandle,
        (IOCTL_OPEN_WDMAUD) as DWORD,
        core::ptr::addr_of_mut!(DeviceInfo) as LPVOID,
        core::mem::size_of::<WDMAUD_DEVICE_INFO>() as DWORD,
        core::ptr::addr_of_mut!(DeviceInfo) as LPVOID,
        core::mem::size_of::<WDMAUD_DEVICE_INFO>() as DWORD,
        core::ptr::null_mut(),
    );

    if !(MMSUCCESS!(Result) != 0) {
        CloseHandle((*Instance).hNotifyEvent);
        CloseHandle((*Instance).hStopEvent);
        return TranslateInternalMmResult(Result);
    }

    hThread = CreateThread(
        core::ptr::null_mut(),
        0,
        Some(MixerEventThreadRoutine),
        Instance as LPVOID,
        0,
        core::ptr::null_mut(),
    );
    if !hThread.is_null() {
        CloseHandle(hThread);
    }

    /* Store sound device handle instance handle */
    (*Instance).Handle = DeviceInfo.hDevice as PVOID;

    return MMSYSERR_NOERROR;
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudSetWaveDeviceFormatByLegacy(
    mut Instance: PSOUND_DEVICE_INSTANCE,
    mut DeviceId: DWORD,
    mut WaveFormat: PWAVEFORMATEX,
    mut WaveFormatSize: DWORD,
) -> MMRESULT {
    let mut Result: MMRESULT = 0;
    let mut SoundDevice: PSOUND_DEVICE = core::ptr::null_mut();
    let mut Identifier: PVOID = core::ptr::null_mut();
    let mut DeviceInfo: WDMAUD_DEVICE_INFO = core::mem::zeroed();
    let mut DeviceType: MMDEVICE_TYPE = 0;

    Result = GetSoundDeviceFromInstance(Instance, &raw mut SoundDevice);

    if ((MMSUCCESS!(Result)) == 0) {
        return TranslateInternalMmResult(Result);
    }

    Result = GetSoundDeviceIdentifier(SoundDevice, &raw mut Identifier);

    if ((MMSUCCESS!(Result)) == 0) {
        return TranslateInternalMmResult(Result);
    }

    if !(*Instance).Handle.is_null() {
        /* device is already open */
        return MMSYSERR_NOERROR as MMRESULT;
    }

    Result = GetSoundDeviceType(SoundDevice, &raw mut DeviceType);

    SND_ASSERT!(Result == MMSYSERR_NOERROR as MMRESULT);

    ZeroMemory!(
        &raw mut DeviceInfo as *mut core::ffi::c_void,
        (core::mem::size_of::<WDMAUD_DEVICE_INFO>()) as usize,
    );
    DeviceInfo.DeviceType = DeviceType as SOUND_DEVICE_TYPE;
    DeviceInfo.DeviceIndex = DeviceId as ULONG_PTR;
    DeviceInfo.u.WaveFormatEx.cbSize = (*WaveFormat).cbSize;
    DeviceInfo.u.WaveFormatEx.wFormatTag = (*WaveFormat).wFormatTag;
    #[cfg(usermode_mixer)]
    {
        DeviceInfo.u.WaveFormatEx.nChannels = 2;
        DeviceInfo.u.WaveFormatEx.nSamplesPerSec = 44100;
        DeviceInfo.u.WaveFormatEx.nBlockAlign = 4;
        DeviceInfo.u.WaveFormatEx.nAvgBytesPerSec = 176400;
        DeviceInfo.u.WaveFormatEx.wBitsPerSample = 16;
    }
    #[cfg(not(usermode_mixer))]
    {
        DeviceInfo.u.WaveFormatEx.nChannels = (*WaveFormat).nChannels;
        DeviceInfo.u.WaveFormatEx.nSamplesPerSec = (*WaveFormat).nSamplesPerSec;
        DeviceInfo.u.WaveFormatEx.nBlockAlign = (*WaveFormat).nBlockAlign;
        DeviceInfo.u.WaveFormatEx.nAvgBytesPerSec = (*WaveFormat).nAvgBytesPerSec;
        DeviceInfo.u.WaveFormatEx.wBitsPerSample = DeviceInfo
            .u
            .WaveFormatEx
            .nAvgBytesPerSec
            .wrapping_mul(8 as DWORD)
            .wrapping_div(
                DeviceInfo
                    .u
                    .WaveFormatEx
                    .nSamplesPerSec
                    .wrapping_mul(DeviceInfo.u.WaveFormatEx.nChannels as DWORD),
            ) as WORD;
    }

    Result = SyncOverlappedDeviceIoControl(
        KernelHandle,
        IOCTL_OPEN_WDMAUD as DWORD,
        &raw mut DeviceInfo as LPVOID,
        (core::mem::size_of::<WDMAUD_DEVICE_INFO>()) as DWORD,
        &raw mut DeviceInfo as LPVOID,
        (core::mem::size_of::<WDMAUD_DEVICE_INFO>()) as DWORD,
        core::ptr::null_mut::<DWORD>(),
    );

    if ((MMSUCCESS!(Result)) == 0) {
        return TranslateInternalMmResult(Result);
    }

    if WaveFormatSize as usize >= (core::mem::size_of::<WAVEFORMAT>()) as usize {
        /* Store format */
        (*Instance).WaveFormatEx.wFormatTag = (*WaveFormat).wFormatTag;
        (*Instance).WaveFormatEx.nChannels = (*WaveFormat).nChannels;
        (*Instance).WaveFormatEx.nSamplesPerSec = (*WaveFormat).nSamplesPerSec;
        (*Instance).WaveFormatEx.nBlockAlign = (*WaveFormat).nBlockAlign;
        (*Instance).WaveFormatEx.nAvgBytesPerSec = (*WaveFormat).nAvgBytesPerSec;
    }

    /* store details */
    (*Instance).WaveFormatEx.cbSize = (*WaveFormat).cbSize;
    (*Instance).WaveFormatEx.wBitsPerSample = DeviceInfo
        .u
        .WaveFormatEx
        .nAvgBytesPerSec
        .wrapping_mul(8 as DWORD)
        .wrapping_div(
            DeviceInfo
                .u
                .WaveFormatEx
                .nSamplesPerSec
                .wrapping_mul(DeviceInfo.u.WaveFormatEx.nChannels as DWORD),
        ) as WORD;

    /* Store sound device handle instance handle */
    (*Instance).Handle = DeviceInfo.hDevice as PVOID;

    /* Now determine framing requirements */
    Result = SyncOverlappedDeviceIoControl(
        KernelHandle,
        IOCTL_GETFRAMESIZE as DWORD,
        &raw mut DeviceInfo as LPVOID,
        (core::mem::size_of::<WDMAUD_DEVICE_INFO>()) as DWORD,
        &raw mut DeviceInfo as LPVOID,
        (core::mem::size_of::<WDMAUD_DEVICE_INFO>()) as DWORD,
        core::ptr::null_mut::<DWORD>(),
    );

    if ((MMSUCCESS!(Result)) != 0) {
        if DeviceInfo.u.FrameSize != 0 {
            (*Instance).FrameSize = DeviceInfo
                .u
                .FrameSize
                .wrapping_mul(2 as ULONG) as DWORD;
            (*Instance).BufferCount = (*WaveFormat)
                .nAvgBytesPerSec
                .wrapping_div((*Instance).FrameSize);
            SND_TRACE!(
                RUSTCONV_WSTR_CB2B95ECFA77.as_ptr(),
                (*Instance).FrameSize,
                (*Instance).BufferCount
            );
        }
    } else {
        // use a default of 100 buffers
        (*Instance).BufferCount = 100 as DWORD;
    }

    /* Now acquire resources */
    DeviceInfo.u.State = KSSTATE_ACQUIRE as _;
    SyncOverlappedDeviceIoControl(
        KernelHandle,
        IOCTL_SETDEVICE_STATE as DWORD,
        &raw mut DeviceInfo as LPVOID,
        (core::mem::size_of::<WDMAUD_DEVICE_INFO>()) as DWORD,
        &raw mut DeviceInfo as LPVOID,
        (core::mem::size_of::<WDMAUD_DEVICE_INFO>()) as DWORD,
        core::ptr::null_mut::<DWORD>(),
    );

    /* pause the pin */
    DeviceInfo.u.State = KSSTATE_PAUSE as _;
    SyncOverlappedDeviceIoControl(
        KernelHandle,
        IOCTL_SETDEVICE_STATE as DWORD,
        &raw mut DeviceInfo as LPVOID,
        (core::mem::size_of::<WDMAUD_DEVICE_INFO>()) as DWORD,
        &raw mut DeviceInfo as LPVOID,
        (core::mem::size_of::<WDMAUD_DEVICE_INFO>()) as DWORD,
        core::ptr::null_mut::<DWORD>(),
    );

    /* start the pin */
    DeviceInfo.u.State = KSSTATE_RUN as _;
    SyncOverlappedDeviceIoControl(
        KernelHandle,
        IOCTL_SETDEVICE_STATE as DWORD,
        &raw mut DeviceInfo as LPVOID,
        (core::mem::size_of::<WDMAUD_DEVICE_INFO>()) as DWORD,
        &raw mut DeviceInfo as LPVOID,
        (core::mem::size_of::<WDMAUD_DEVICE_INFO>()) as DWORD,
        core::ptr::null_mut::<DWORD>(),
    );

    return MMSYSERR_NOERROR as MMRESULT;
}
#[no_mangle]
pub unsafe extern "system" fn LegacyCompletionRoutine(
    dwErrorCode: DWORD,
    dwNumberOfBytesTransferred: DWORD,
    lpOverlapped: LPOVERLAPPED,
) {
    let mut Overlap: PSOUND_OVERLAPPED = core::ptr::null_mut::<_SOUND_OVERLAPPED>();
    let mut DeviceInfo: PWDMAUD_DEVICE_INFO = core::ptr::null_mut::<WDMAUD_DEVICE_INFO>();

    Overlap = lpOverlapped as PSOUND_OVERLAPPED;
    DeviceInfo = (*Overlap).CompletionContext as PWDMAUD_DEVICE_INFO;

    /* Call mmebuddy overlap routine */
    (*Overlap)
        .OriginalCompletionRoutine
        .unwrap_unchecked()(
        dwErrorCode,
        (*DeviceInfo).Header.DataUsed as DWORD,
        lpOverlapped,
    );

    HeapFree(GetProcessHeap(), 0, DeviceInfo as PVOID);
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudCommitWaveBufferByLegacy(
    SoundDeviceInstance: PSOUND_DEVICE_INSTANCE,
    OffsetPtr: PVOID,
    Length: DWORD,
    Overlap: PSOUND_OVERLAPPED,
    CompletionRoutine: LPOVERLAPPED_COMPLETION_ROUTINE,
) -> MMRESULT {
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut Result: MMRESULT;
    let mut DeviceInfo: PWDMAUD_DEVICE_INFO;
    let mut SoundDevice: PSOUND_DEVICE = core::mem::zeroed();
    let mut DeviceType: MMDEVICE_TYPE = core::mem::zeroed();
    let mut Ret: BOOL;

    // VALIDATE_MMSYS_PARAMETER!( SoundDeviceInstance );
    if SoundDeviceInstance.is_null() {
        return MMSYSERR_INVALPARAM;
    }
    // VALIDATE_MMSYS_PARAMETER!( OffsetPtr );
    if OffsetPtr.is_null() {
        return MMSYSERR_INVALPARAM;
    }
    // VALIDATE_MMSYS_PARAMETER!( Overlap );
    if Overlap.is_null() {
        return MMSYSERR_INVALPARAM;
    }
    // VALIDATE_MMSYS_PARAMETER!( CompletionRoutine );
    if CompletionRoutine.is_none() {
        return MMSYSERR_INVALPARAM;
    }

    GetSoundDeviceInstanceHandle(SoundDeviceInstance, core::ptr::addr_of_mut!(Handle));
    SND_ASSERT!(Handle);

    Result = GetSoundDeviceFromInstance(SoundDeviceInstance, core::ptr::addr_of_mut!(SoundDevice));

    if ((MMSUCCESS!(Result)) == 0) {
        return TranslateInternalMmResult(Result);
    }

    Result = GetSoundDeviceType(SoundDevice, core::ptr::addr_of_mut!(DeviceType));
    SND_ASSERT!(Result == MMSYSERR_NOERROR);

    DeviceInfo = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<WDMAUD_DEVICE_INFO>() as SIZE_T,
    ) as PWDMAUD_DEVICE_INFO;
    if DeviceInfo.is_null() {
        // no memory
        return MMSYSERR_NOMEM;
    }

    (*DeviceInfo).Header.FrameExtent = Length as ULONG;
    if (DeviceType) as u32 == WAVE_OUT_DEVICE_TYPE {
        (*DeviceInfo).Header.DataUsed = Length as ULONG;
    }
    (*DeviceInfo).Header.Data = OffsetPtr;
    (*DeviceInfo).Header.Size = core::mem::size_of::<WDMAUD_DEVICE_INFO>() as ULONG;
    (*DeviceInfo).Header.PresentationTime.Numerator = 1 as ULONG;
    (*DeviceInfo).Header.PresentationTime.Denominator = 1 as ULONG;
    (*DeviceInfo).hDevice = Handle;
    (*DeviceInfo).DeviceType = DeviceType as SOUND_DEVICE_TYPE;

    // create completion event
    Handle = CreateEventW(
        core::ptr::null_mut(),
        (FALSE) as BOOL,
        (FALSE) as BOOL,
        core::ptr::null(),
    );
    (*Overlap).Standard.hEvent = Handle;
    if (*Overlap).Standard.hEvent.is_null() {
        // no memory
        HeapFree(GetProcessHeap(), 0, DeviceInfo as PVOID);
        return MMSYSERR_NOMEM;
    }

    (*Overlap).OriginalCompletionRoutine = CompletionRoutine;
    (*Overlap).CompletionContext = DeviceInfo as PVOID;

    if (DeviceType) as u32 == WAVE_OUT_DEVICE_TYPE {
        Ret = WriteFileEx(
            KernelHandle,
            DeviceInfo as LPCVOID,
            core::mem::size_of::<WDMAUD_DEVICE_INFO>() as DWORD,
            Overlap as LPOVERLAPPED,
            Some(LegacyCompletionRoutine),
        );
        if Ret != 0 {
            WaitForSingleObjectEx(KernelHandle, INFINITE as DWORD, (TRUE) as BOOL);
        }
    } else if (DeviceType) as u32 == WAVE_IN_DEVICE_TYPE {
        Ret = ReadFileEx(
            KernelHandle,
            DeviceInfo as PVOID,
            core::mem::size_of::<WDMAUD_DEVICE_INFO>() as DWORD,
            Overlap as LPOVERLAPPED,
            Some(LegacyCompletionRoutine),
        );
        if Ret != 0 {
            WaitForSingleObjectEx(KernelHandle, INFINITE as DWORD, (TRUE) as BOOL);
        }
    }

    // close event handle
    CloseHandle(Handle);

    return MMSYSERR_NOERROR;
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudSetWaveStateByLegacy(
    SoundDeviceInstance: *mut _SOUND_DEVICE_INSTANCE,
    bStart: BOOL,
) -> MMRESULT {
    let mut Result: MMRESULT;
    let mut SoundDevice: PSOUND_DEVICE = core::ptr::null_mut();
    let mut DeviceInfo: WDMAUD_DEVICE_INFO = core::mem::zeroed();
    let mut DeviceType: MMDEVICE_TYPE = core::mem::zeroed();
    let mut Handle: HANDLE = core::ptr::null_mut();

    Result = GetSoundDeviceFromInstance(
        SoundDeviceInstance as PSOUND_DEVICE_INSTANCE,
        core::ptr::addr_of_mut!(SoundDevice),
    );

    if !(MMSUCCESS!(Result) != 0) {
        return TranslateInternalMmResult(Result);
    }

    Result = GetSoundDeviceType(SoundDevice, core::ptr::addr_of_mut!(DeviceType));
    SND_ASSERT!(Result == MMSYSERR_NOERROR);

    Result = GetSoundDeviceInstanceHandle(
        SoundDeviceInstance as PSOUND_DEVICE_INSTANCE,
        core::ptr::addr_of_mut!(Handle),
    );
    SND_ASSERT!(Result == MMSYSERR_NOERROR);

    ZeroMemory!(
        core::ptr::addr_of_mut!(DeviceInfo) as *mut core::ffi::c_void,
        core::mem::size_of::<WDMAUD_DEVICE_INFO>(),
    );
    DeviceInfo.hDevice = Handle;
    DeviceInfo.DeviceType = (DeviceType) as _;

    if bStart != 0 {
        DeviceInfo.u.State = KSSTATE_RUN;
    } else {
        DeviceInfo.u.State = KSSTATE_PAUSE;
    }
    Result = SyncOverlappedDeviceIoControl(
        KernelHandle,
        (IOCTL_SETDEVICE_STATE) as DWORD,
        core::ptr::addr_of_mut!(DeviceInfo) as LPVOID,
        core::mem::size_of::<WDMAUD_DEVICE_INFO>() as DWORD,
        core::ptr::addr_of_mut!(DeviceInfo) as LPVOID,
        core::mem::size_of::<WDMAUD_DEVICE_INFO>() as DWORD,
        core::ptr::null_mut(),
    );

    return Result;
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudGetDeviceInterfaceStringByLegacy(
    mut DeviceType: MMDEVICE_TYPE,
    mut DeviceId: DWORD,
    mut Interface: LPWSTR,
    mut InterfaceLength: DWORD,
    mut InterfaceSize: *mut DWORD,
) -> MMRESULT {
    let mut DeviceInfo: WDMAUD_DEVICE_INFO = core::mem::zeroed();
    let mut Result: MMRESULT = 0;

    ZeroMemory!(
        core::ptr::addr_of_mut!(DeviceInfo) as LPVOID,
        core::mem::size_of::<WDMAUD_DEVICE_INFO>(),
    );
    DeviceInfo.DeviceType = DeviceType as SOUND_DEVICE_TYPE;
    DeviceInfo.DeviceIndex = DeviceId as ULONG_PTR;

    Result = SyncOverlappedDeviceIoControl(
        KernelHandle,
        (IOCTL_QUERYDEVICEINTERFACESTRING) as DWORD,
        core::ptr::addr_of_mut!(DeviceInfo) as LPVOID,
        (core::mem::size_of::<WDMAUD_DEVICE_INFO>()) as DWORD,
        core::ptr::addr_of_mut!(DeviceInfo) as LPVOID,
        (core::mem::size_of::<WDMAUD_DEVICE_INFO>()) as DWORD,
        core::ptr::null_mut::<DWORD>(),
    );

    if !(MMSUCCESS!(Result) != 0) {
        return TranslateInternalMmResult(Result);
    }

    if Interface.is_null() {
        SND_ASSERT!(InterfaceSize);

        *InterfaceSize = DeviceInfo.u.Interface.DeviceInterfaceStringSize as DWORD;
        return MMSYSERR_NOERROR;
    }

    if InterfaceLength < DeviceInfo.u.Interface.DeviceInterfaceStringSize {
        /* buffer is too small */
        return MMSYSERR_MOREDATA;
    }

    DeviceInfo.u.Interface.DeviceInterfaceStringSize = InterfaceLength as ULONG;
    DeviceInfo.u.Interface.DeviceInterfaceString = Interface;

    Result = SyncOverlappedDeviceIoControl(
        KernelHandle,
        (IOCTL_QUERYDEVICEINTERFACESTRING) as DWORD,
        core::ptr::addr_of_mut!(DeviceInfo) as LPVOID,
        (core::mem::size_of::<WDMAUD_DEVICE_INFO>()) as DWORD,
        core::ptr::addr_of_mut!(DeviceInfo) as LPVOID,
        (core::mem::size_of::<WDMAUD_DEVICE_INFO>()) as DWORD,
        core::ptr::null_mut::<DWORD>(),
    );

    if MMSUCCESS!(Result) != 0 && InterfaceLength > 2 {
        *Interface.offset(1 as isize) = 0x005Cu16;
        *Interface.offset((InterfaceLength - 1) as isize) = 0x0000u16;
    }

    return Result;
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudGetWavePositionByLegacy(
        SoundDeviceInstance: *mut _SOUND_DEVICE_INSTANCE,
        Time: *mut MMTIME) -> MMRESULT {
    let mut Result: MMRESULT;
    let mut SoundDevice: PSOUND_DEVICE = core::ptr::null_mut();
    let mut DeviceInfo: WDMAUD_DEVICE_INFO = core::mem::zeroed();
    let mut DeviceType: MMDEVICE_TYPE = core::mem::zeroed();
    let mut Handle: HANDLE = core::ptr::null_mut();

    Result = GetSoundDeviceFromInstance(SoundDeviceInstance, core::ptr::addr_of_mut!(SoundDevice));

    if ((MMSUCCESS!(Result)) == 0) {
        return TranslateInternalMmResult(Result);
    }

    Result = GetSoundDeviceType(SoundDevice, core::ptr::addr_of_mut!(DeviceType));
    SND_ASSERT!(Result == MMSYSERR_NOERROR);

    Result = GetSoundDeviceInstanceHandle(SoundDeviceInstance, core::ptr::addr_of_mut!(Handle));
    SND_ASSERT!(Result == MMSYSERR_NOERROR);

    ZeroMemory!(core::ptr::addr_of_mut!(DeviceInfo) as *mut core::ffi::c_void,
               core::mem::size_of::<WDMAUD_DEVICE_INFO>());
    DeviceInfo.hDevice = Handle;
    DeviceInfo.DeviceType = (DeviceType) as _;

    Result = SyncOverlappedDeviceIoControl(KernelHandle,
                                    (IOCTL_GETPOS) as DWORD,
                                    core::ptr::addr_of_mut!(DeviceInfo) as LPVOID,
                                    core::mem::size_of::<WDMAUD_DEVICE_INFO>() as DWORD,
                                    core::ptr::addr_of_mut!(DeviceInfo) as LPVOID,
                                    core::mem::size_of::<WDMAUD_DEVICE_INFO>() as DWORD,
                                    core::ptr::null_mut());

    if ((MMSUCCESS!(Result)) == 0) {
        return TranslateInternalMmResult(Result);
    }

    (*Time).wType = TIME_BYTES;
    (*Time).u.cb = DeviceInfo.u.Position as DWORD;

    return MMSYSERR_NOERROR;
}

#[no_mangle]
pub unsafe extern "C" fn WdmAudGetVolumeByLegacy(
    SoundDeviceInstance: PSOUND_DEVICE_INSTANCE,
    DeviceId: DWORD,
    pdwVolume: PDWORD,
) -> MMRESULT {
    MMSYSERR_NOTSUPPORTED
}

#[no_mangle]
pub unsafe extern "C" fn WdmAudSetVolumeByLegacy(
    SoundDeviceInstance: PSOUND_DEVICE_INSTANCE,
    DeviceId: DWORD,
    dwVolume: DWORD,
) -> MMRESULT {
    MMSYSERR_NOTSUPPORTED
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudResetStreamByLegacy(
    SoundDeviceInstance: *mut _SOUND_DEVICE_INSTANCE,
    DeviceType: MMDEVICE_TYPE,
    bStartReset: BOOLEAN,
) -> MMRESULT {
    let mut Result: MMRESULT;
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut DeviceInfo: WDMAUD_DEVICE_INFO = core::mem::zeroed::<WDMAUD_DEVICE_INFO>();

    Result = GetSoundDeviceInstanceHandle(
        SoundDeviceInstance as PSOUND_DEVICE_INSTANCE,
        core::ptr::addr_of_mut!(Handle),
    );
    SND_ASSERT!(Result == MMSYSERR_NOERROR);

    ZeroMemory!(
        core::ptr::addr_of_mut!(DeviceInfo) as *mut core::ffi::c_void,
        core::mem::size_of::<WDMAUD_DEVICE_INFO>(),
    );
    (*core::ptr::addr_of_mut!(DeviceInfo)).hDevice = Handle;
    (*core::ptr::addr_of_mut!(DeviceInfo)).DeviceType = DeviceType as SOUND_DEVICE_TYPE;
    (*core::ptr::addr_of_mut!(DeviceInfo)).u.ResetStream = (if bStartReset as core::ffi::c_int != 0 {
        KSRESET_BEGIN as core::ffi::c_int
    } else {
        KSRESET_END as core::ffi::c_int
    }) as KSRESET;

    Result = SyncOverlappedDeviceIoControl(
        KernelHandle,
        IOCTL_RESET_STREAM as DWORD,
        core::ptr::addr_of_mut!(DeviceInfo) as LPVOID,
        core::mem::size_of::<WDMAUD_DEVICE_INFO>() as DWORD,
        core::ptr::addr_of_mut!(DeviceInfo) as LPVOID,
        core::mem::size_of::<WDMAUD_DEVICE_INFO>() as DWORD,
        core::ptr::null_mut::<DWORD>(),
    );
    return Result;
}
#[no_mangle]
pub unsafe extern "C" fn WdmAudQueryMixerInfoByLegacy(
    SoundDeviceInstance: *mut _SOUND_DEVICE_INSTANCE,
    DeviceId: DWORD,
    uMsg: UINT,
    Parameter: LPVOID,
    Flags: DWORD,
) -> MMRESULT {
    let mut Result: MMRESULT;
    let mut DeviceInfo: WDMAUD_DEVICE_INFO = core::mem::zeroed();
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut IoControlCode: DWORD;
    let mut MixLine: LPMIXERLINEW;
    let mut MixControls: LPMIXERLINECONTROLSW;
    let mut MixDetails: LPMIXERCONTROLDETAILS;

    SND_TRACE!(RUSTCONV_WSTR_2B3E7AFAF5DE.as_ptr(), uMsg, Flags);

    Result = GetSoundDeviceInstanceHandle(SoundDeviceInstance, core::ptr::addr_of_mut!(Handle));
    SND_ASSERT!(Result == MMSYSERR_NOERROR);

    ZeroMemory!(
        core::ptr::addr_of_mut!(DeviceInfo) as LPVOID,
        core::mem::size_of::<WDMAUD_DEVICE_INFO>() as usize,
    );
    DeviceInfo.hDevice = Handle;
    DeviceInfo.DeviceIndex = (DeviceId) as _;
    DeviceInfo.DeviceType = MIXER_DEVICE_TYPE;
    DeviceInfo.Flags = Flags;

    MixLine = Parameter as LPMIXERLINEW;
    MixControls = Parameter as LPMIXERLINECONTROLSW;
    MixDetails = Parameter as LPMIXERCONTROLDETAILS;

    match uMsg {
        MXDM_GETLINEINFO => {
            RtlCopyMemory!(
                core::ptr::addr_of_mut!(DeviceInfo.u.MixLine) as *mut core::ffi::c_void,
                MixLine as *const core::ffi::c_void,
                core::mem::size_of::<MIXERLINEW>() as usize,
            );
            IoControlCode = (IOCTL_GETLINEINFO as u32);
        }
        MXDM_GETLINECONTROLS => {
            RtlCopyMemory!(
                core::ptr::addr_of_mut!(DeviceInfo.u.MixControls) as *mut core::ffi::c_void,
                MixControls as *const core::ffi::c_void,
                core::mem::size_of::<MIXERLINECONTROLSW>() as usize,
            );
            IoControlCode = (IOCTL_GETLINECONTROLS as u32);
        }
        MXDM_SETCONTROLDETAILS => {
            RtlCopyMemory!(
                core::ptr::addr_of_mut!(DeviceInfo.u.MixDetails) as *mut core::ffi::c_void,
                MixDetails as *const core::ffi::c_void,
                core::mem::size_of::<MIXERCONTROLDETAILS>() as usize,
            );
            IoControlCode = (IOCTL_SETCONTROLDETAILS as u32);
        }
        MXDM_GETCONTROLDETAILS => {
            RtlCopyMemory!(
                core::ptr::addr_of_mut!(DeviceInfo.u.MixDetails) as *mut core::ffi::c_void,
                MixDetails as *const core::ffi::c_void,
                core::mem::size_of::<MIXERCONTROLDETAILS>() as usize,
            );
            IoControlCode = (IOCTL_GETCONTROLDETAILS as u32);
        }
        _ => {
            SND_ASSERT!(0);
            return MMSYSERR_NOTSUPPORTED;
        }
    }

    Result = SyncOverlappedDeviceIoControl(
        KernelHandle,
        IoControlCode,
        core::ptr::addr_of_mut!(DeviceInfo) as LPVOID,
        core::mem::size_of::<WDMAUD_DEVICE_INFO>() as DWORD,
        core::ptr::addr_of_mut!(DeviceInfo) as LPVOID,
        core::mem::size_of::<WDMAUD_DEVICE_INFO>() as DWORD,
        core::ptr::null_mut::<DWORD>(),
    );

    if ((MMSUCCESS!(Result)) == 0) {
        return Result;
    }

    match uMsg {
        MXDM_GETLINEINFO => {
            RtlCopyMemory!(
                MixLine as *mut core::ffi::c_void,
                core::ptr::addr_of_mut!(DeviceInfo.u.MixLine) as *const core::ffi::c_void,
                core::mem::size_of::<MIXERLINEW>() as usize,
            );
        }
        _ => {}
    }

    return Result;
}
