use crate::dsound::capture::DSOUND_CaptureCreate;
use crate::dsound::capture::DSOUND_CaptureCreate8;
use crate::dsound::dsound::DSOUND_Create;
use crate::dsound::dsound::DSOUND_Create8;
use crate::dsound::capture::DSOUND_capture;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_260153EE262B: [u16; 29] = [
    0x0050u16, 0x0072, 0x0069, 0x006D, 0x0061, 0x0072, 0x0079, 0x0020, 0x0053, 0x006F,
    0x0075, 0x006E, 0x0064, 0x0020, 0x0043, 0x0061, 0x0070, 0x0074, 0x0075, 0x0072,
    0x0065, 0x0020, 0x0044, 0x0072, 0x0069, 0x0076, 0x0065, 0x0072, 0x0000,
];
pub static RUSTCONV_WSTR_B996700897BA: [u16; 21] = [
    0x0050u16, 0x0072, 0x0069, 0x006D, 0x0061, 0x0072, 0x0079, 0x0020, 0x0053, 0x006F,
    0x0075, 0x006E, 0x0064, 0x0020, 0x0044, 0x0072, 0x0069, 0x0076, 0x0065, 0x0072,
    0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut DSOUND_renderer: [*mut DirectSoundDevice; 10] =
    [::core::ptr::null::<DirectSoundDevice>() as *mut DirectSoundDevice; 10];

#[no_mangle]
pub static mut DSOUND_renderer_guids: [GUID; 10] = [_GUID {
    Data1: 0,
    Data2: 0,
    Data3: 0,
    Data4: [0; 8],
}; 10];

#[no_mangle]
pub static mut DSOUND_capture_guids: [GUID; 10] = [_GUID {
    Data1: 0,
    Data2: 0,
    Data3: 0,
    Data4: [0; 8],
}; 10];

#[no_mangle]
pub static mut ds_emuldriver: ::core::ffi::c_int = 0 as ::core::ffi::c_int;

#[no_mangle]
pub static mut ds_hel_buflen: ::core::ffi::c_int =
    32768 as ::core::ffi::c_int * 2 as ::core::ffi::c_int;

#[no_mangle]
pub static mut ds_snd_queue_max: ::core::ffi::c_int = 10 as ::core::ffi::c_int;

#[no_mangle]
pub static mut ds_snd_queue_min: ::core::ffi::c_int = 6 as ::core::ffi::c_int;

#[no_mangle]
pub static mut ds_snd_shadow_maxsize: ::core::ffi::c_int = 2 as ::core::ffi::c_int;

#[no_mangle]
pub static mut ds_hw_accel: ::core::ffi::c_int = (DS_HW_ACCEL_FULL as i32);

#[no_mangle]
pub static mut ds_default_sample_rate: ::core::ffi::c_int = 44100 as ::core::ffi::c_int;

#[no_mangle]
pub static mut ds_default_bits_per_sample: ::core::ffi::c_int = 16 as ::core::ffi::c_int;

static mut ds_default_playback: ::core::ffi::c_int = 0;

static mut ds_default_capture: ::core::ffi::c_int = 0;

static mut instance: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

static mut DSOUND_CF: [IClassFactoryImpl; 7] = unsafe {
    [
        IClassFactoryImpl {
            IClassFactory_iface: IClassFactory {
                lpVtbl: &raw const DSCF_Vtbl,
            },
            rclsid: &raw const CLSID_DirectSound,
            pfnCreateInstance: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*const IID, *mut LPDIRECTSOUND) -> HRESULT>,
                FnCreateInstance,
            >(Some(
                DSOUND_Create as unsafe extern "C" fn(*const IID, *mut LPDIRECTSOUND) -> HRESULT,
            )),
        },
        IClassFactoryImpl {
            IClassFactory_iface: IClassFactory {
                lpVtbl: &raw const DSCF_Vtbl,
            },
            rclsid: &raw const CLSID_DirectSound8,
            pfnCreateInstance: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*const IID, *mut LPDIRECTSOUND8) -> HRESULT>,
                FnCreateInstance,
            >(Some(
                DSOUND_Create8 as unsafe extern "C" fn(*const IID, *mut LPDIRECTSOUND8) -> HRESULT,
            )),
        },
        IClassFactoryImpl {
            IClassFactory_iface: IClassFactory {
                lpVtbl: &raw const DSCF_Vtbl,
            },
            rclsid: &raw const CLSID_DirectSoundCapture,
            pfnCreateInstance: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*const IID, *mut LPDIRECTSOUNDCAPTURE) -> HRESULT>,
                FnCreateInstance,
            >(Some(
                DSOUND_CaptureCreate
                    as unsafe extern "C" fn(*const IID, *mut LPDIRECTSOUNDCAPTURE) -> HRESULT,
            )),
        },
        IClassFactoryImpl {
            IClassFactory_iface: IClassFactory {
                lpVtbl: &raw const DSCF_Vtbl,
            },
            rclsid: &raw const CLSID_DirectSoundCapture8,
            pfnCreateInstance: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*const IID, *mut LPDIRECTSOUNDCAPTURE8) -> HRESULT>,
                FnCreateInstance,
            >(Some(
                DSOUND_CaptureCreate8
                    as unsafe extern "C" fn(*const IID, *mut LPDIRECTSOUNDCAPTURE8) -> HRESULT,
            )),
        },
        IClassFactoryImpl {
            IClassFactory_iface: IClassFactory {
                lpVtbl: &raw const DSCF_Vtbl,
            },
            rclsid: &raw const CLSID_DirectSoundFullDuplex,
            pfnCreateInstance: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*const IID, *mut LPDIRECTSOUNDFULLDUPLEX) -> HRESULT>,
                FnCreateInstance,
            >(Some(
                DSOUND_FullDuplexCreate
                    as unsafe extern "C" fn(*const IID, *mut LPDIRECTSOUNDFULLDUPLEX) -> HRESULT,
            )),
        },
        IClassFactoryImpl {
            IClassFactory_iface: IClassFactory {
                lpVtbl: &raw const DSCF_Vtbl,
            },
            rclsid: &raw const CLSID_DirectSoundPrivate,
            pfnCreateInstance: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*const IID, *mut *mut IKsPropertySet) -> HRESULT>,
                FnCreateInstance,
            >(Some(
                IKsPrivatePropertySetImpl_Create
                    as unsafe extern "C" fn(*const IID, *mut *mut IKsPropertySet) -> HRESULT,
            )),
        },
        IClassFactoryImpl {
            IClassFactory_iface: IClassFactory {
                lpVtbl: ::core::ptr::null::<IClassFactoryVtbl>(),
            },
            rclsid: ::core::ptr::null::<CLSID>(),
            pfnCreateInstance: None,
        },
    ]
};

#[repr(C)]
pub struct morecontext {
    pub callA: LPDSENUMCALLBACKA,
    pub data: LPVOID,
}
pub type Pmorecontext = *mut morecontext;

pub type FnCreateInstance = Option<unsafe extern "C" fn(riid: REFIID, ppobj: *mut LPVOID) -> HRESULT>;

#[repr(C)]
pub struct IClassFactoryImpl {
    pub IClassFactory_iface: IClassFactory,
    pub rclsid: REFCLSID,
    pub pfnCreateInstance: FnCreateInstance,
}
pub type PIClassFactoryImpl = *mut IClassFactoryImpl;

// MAXWAVEDRIVERS is not defined in this preamble; value assumed 10.
// DS_HW_ACCEL_FULL value assumed from dsound.h.
#[no_mangle]
pub unsafe extern "C" fn mmErr(err: UINT) -> HRESULT {
    match err {
        MMSYSERR_NOERROR => {
            return (DS_OK) as HRESULT;
        }
        MMSYSERR_ALLOCATED => {
            return DSERR_ALLOCATED;
        }
        MMSYSERR_ERROR | MMSYSERR_INVALHANDLE | WAVERR_STILLPLAYING => {
            return DSERR_GENERIC; /* FIXME */
        }
        MMSYSERR_BADDEVICEID | MMSYSERR_NODRIVER => {
            return DSERR_NODRIVER;
        }
        MMSYSERR_NOMEM => {
            return DSERR_OUTOFMEMORY;
        }
        MMSYSERR_INVALPARAM | WAVERR_BADFORMAT | WAVERR_UNPREPARED => {
            return DSERR_INVALIDPARAM;
        }
        MMSYSERR_NOTSUPPORTED => {
            return DSERR_UNSUPPORTED;
        }
        _ => {
            FIXME!("Unknown MMSYS error %d\n", err);
            return DSERR_GENERIC;
        }
    }
}

unsafe extern "C" fn get_config_key(defkey: HKEY, appkey: HKEY, name: *const i8, buffer: *mut i8, mut size: DWORD) -> DWORD {
    if !appkey.is_null() && (RegQueryValueExA(appkey, name as LPCSTR, core::ptr::null_mut(), core::ptr::null_mut(), buffer as LPBYTE, core::ptr::addr_of_mut!(size)) == 0) {
        return 0;
    }
    if !defkey.is_null() && (RegQueryValueExA(defkey, name as LPCSTR, core::ptr::null_mut(), core::ptr::null_mut(), buffer as LPBYTE, core::ptr::addr_of_mut!(size)) == 0) {
        return 0;
    }
    return ERROR_FILE_NOT_FOUND;
}
#[no_mangle]
pub unsafe extern "C" fn setup_dsound_options() {
    let mut buffer: [core::ffi::c_char; (MAX_PATH + 16) as _] = [0 as core::ffi::c_char; (MAX_PATH + 16) as _];
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut appkey: HKEY = 0 as HKEY;
    let mut len: DWORD;

    buffer[MAX_PATH as usize] = 0 as core::ffi::c_char;

    /* @@ Wine registry key: HKCU\Software\Wine\DirectSound */
    if RegOpenKeyA(HKEY_CURRENT_USER as HKEY, b"Software\\Wine\\DirectSound\0".as_ptr() as *const core::ffi::c_char, core::ptr::addr_of_mut!(hkey)) != 0 {
        hkey = 0 as HKEY;
    }

    len = GetModuleFileNameA(0 as HINSTANCE, buffer.as_mut_ptr(), MAX_PATH as DWORD);
    if len != 0 && len < MAX_PATH as DWORD {
        let mut tmpkey: HKEY = core::ptr::null_mut();
        /* @@ Wine registry key: HKCU\Software\Wine\AppDefaults\app.exe\DirectSound */
        if RegOpenKeyA(HKEY_CURRENT_USER as HKEY, b"Software\\Wine\\AppDefaults\0".as_ptr() as *const core::ffi::c_char, core::ptr::addr_of_mut!(tmpkey)) == 0 {
            let mut p: *mut core::ffi::c_char = core::ptr::null_mut();
            let mut appname: *mut core::ffi::c_char = buffer.as_mut_ptr();
            p = strrchr(appname, '/' as core::ffi::c_int);
            if !p.is_null() {
                appname = p.offset(1 as isize);
            }
            p = strrchr(appname, '\\' as core::ffi::c_int);
            if !p.is_null() {
                appname = p.offset(1 as isize);
            }
            strcat(appname, b"\\DirectSound\0".as_ptr() as *const core::ffi::c_char);

            if RegOpenKeyA(tmpkey, appname as *const core::ffi::c_char, core::ptr::addr_of_mut!(appkey)) != 0 {
                appkey = 0 as HKEY;
            }
            RegCloseKey(tmpkey);
        }
    }

    /* get options */

    if get_config_key(hkey, appkey, b"EmulDriver\0".as_ptr() as *const core::ffi::c_char, buffer.as_mut_ptr(), MAX_PATH as DWORD) == 0 {
        ds_emuldriver = strcmp(buffer.as_mut_ptr() as *const core::ffi::c_char, b"N\0".as_ptr() as *const core::ffi::c_char);
    }

    if get_config_key(hkey, appkey, b"HelBuflen\0".as_ptr() as *const core::ffi::c_char, buffer.as_mut_ptr(), MAX_PATH as DWORD) == 0 {
        ds_hel_buflen = atoi(buffer.as_mut_ptr() as *const core::ffi::c_char);
    }

    if get_config_key(hkey, appkey, b"SndQueueMax\0".as_ptr() as *const core::ffi::c_char, buffer.as_mut_ptr(), MAX_PATH as DWORD) == 0 {
        ds_snd_queue_max = atoi(buffer.as_mut_ptr() as *const core::ffi::c_char);
    }

    if get_config_key(hkey, appkey, b"SndQueueMin\0".as_ptr() as *const core::ffi::c_char, buffer.as_mut_ptr(), MAX_PATH as DWORD) == 0 {
        ds_snd_queue_min = atoi(buffer.as_mut_ptr() as *const core::ffi::c_char);
    }

    if get_config_key(hkey, appkey, b"HardwareAcceleration\0".as_ptr() as *const core::ffi::c_char, buffer.as_mut_ptr(), MAX_PATH as DWORD) == 0 {
        if strcmp(buffer.as_mut_ptr() as *const core::ffi::c_char, b"Full\0".as_ptr() as *const core::ffi::c_char) == 0 as core::ffi::c_int {
            ds_hw_accel = DS_HW_ACCEL_FULL as core::ffi::c_int;
        } else if strcmp(buffer.as_mut_ptr() as *const core::ffi::c_char, b"Standard\0".as_ptr() as *const core::ffi::c_char) == 0 as core::ffi::c_int {
            ds_hw_accel = DS_HW_ACCEL_STANDARD as core::ffi::c_int;
        } else if strcmp(buffer.as_mut_ptr() as *const core::ffi::c_char, b"Basic\0".as_ptr() as *const core::ffi::c_char) == 0 as core::ffi::c_int {
            ds_hw_accel = DS_HW_ACCEL_BASIC as core::ffi::c_int;
        } else if strcmp(buffer.as_mut_ptr() as *const core::ffi::c_char, b"Emulation\0".as_ptr() as *const core::ffi::c_char) == 0 as core::ffi::c_int {
            ds_hw_accel = DS_HW_ACCEL_EMULATION as core::ffi::c_int;
        }
    }

    if get_config_key(hkey, appkey, b"DefaultPlayback\0".as_ptr() as *const core::ffi::c_char, buffer.as_mut_ptr(), MAX_PATH as DWORD) == 0 {
        ds_default_playback = atoi(buffer.as_mut_ptr() as *const core::ffi::c_char);
    }

    if get_config_key(hkey, appkey, b"MaxShadowSize\0".as_ptr() as *const core::ffi::c_char, buffer.as_mut_ptr(), MAX_PATH as DWORD) == 0 {
        ds_snd_shadow_maxsize = atoi(buffer.as_mut_ptr() as *const core::ffi::c_char);
    }

    if get_config_key(hkey, appkey, b"DefaultCapture\0".as_ptr() as *const core::ffi::c_char, buffer.as_mut_ptr(), MAX_PATH as DWORD) == 0 {
        ds_default_capture = atoi(buffer.as_mut_ptr() as *const core::ffi::c_char);
    }

    if get_config_key(hkey, appkey, b"DefaultSampleRate\0".as_ptr() as *const core::ffi::c_char, buffer.as_mut_ptr(), MAX_PATH as DWORD) == 0 {
        ds_default_sample_rate = atoi(buffer.as_mut_ptr() as *const core::ffi::c_char);
    }

    if get_config_key(hkey, appkey, b"DefaultBitsPerSample\0".as_ptr() as *const core::ffi::c_char, buffer.as_mut_ptr(), MAX_PATH as DWORD) == 0 {
        ds_default_bits_per_sample = atoi(buffer.as_mut_ptr() as *const core::ffi::c_char);
    }

    if !appkey.is_null() {
        RegCloseKey(appkey);
    }
    if !hkey.is_null() {
        RegCloseKey(hkey);
    }
}

unsafe extern "C" fn get_device_id(pGuid: LPCGUID) -> *const i8 {
    if IsEqualGUID!(core::ptr::addr_of!(DSDEVID_DefaultPlayback), pGuid) != false {
        return b"DSDEVID_DefaultPlayback\0".as_ptr() as *const i8;
    } else if IsEqualGUID!(core::ptr::addr_of!(DSDEVID_DefaultVoicePlayback), pGuid) != false {
        return b"DSDEVID_DefaultVoicePlayback\0".as_ptr() as *const i8;
    } else if IsEqualGUID!(core::ptr::addr_of!(DSDEVID_DefaultCapture), pGuid) != false {
        return b"DSDEVID_DefaultCapture\0".as_ptr() as *const i8;
    } else if IsEqualGUID!(core::ptr::addr_of!(DSDEVID_DefaultVoiceCapture), pGuid) != false {
        return b"DSDEVID_DefaultVoiceCapture\0".as_ptr() as *const i8;
    }
    return debugstr_guid(pGuid as *const _GUID);
}

#[no_mangle]
pub unsafe extern "system" fn GetDeviceID(pGuidSrc: LPCGUID, pGuidDest: LPGUID) -> HRESULT {
    if pGuidSrc.is_null() {
        WARN!("invalid parameter: pGuidSrc == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    if pGuidDest.is_null() {
        WARN!("invalid parameter: pGuidDest == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    if IsEqualGUID!(core::ptr::addr_of!(DSDEVID_DefaultPlayback), pGuidSrc) != false
        || IsEqualGUID!(core::ptr::addr_of!(DSDEVID_DefaultVoicePlayback), pGuidSrc) != false
    {
        *pGuidDest = DSOUND_renderer_guids[ds_default_playback as usize];
        return (DS_OK) as HRESULT;
    }

    if IsEqualGUID!(core::ptr::addr_of!(DSDEVID_DefaultCapture), pGuidSrc) != false
        || IsEqualGUID!(core::ptr::addr_of!(DSDEVID_DefaultVoiceCapture), pGuidSrc) != false
    {
        *pGuidDest = DSOUND_capture_guids[ds_default_capture as usize];
        return (DS_OK) as HRESULT;
    }

    *pGuidDest = *pGuidSrc;

    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn a_to_w_callback(guid: LPGUID, descW: LPCWSTR, modW: LPCWSTR, data: LPVOID) -> BOOL {
    let context: *mut morecontext = data as *mut morecontext;
    let mut descA: [::core::ffi::c_char; (MAXPNAMELEN as _)] = [0; (MAXPNAMELEN as _)];
    let mut modA: [::core::ffi::c_char; (MAXPNAMELEN as _)] = [0; (MAXPNAMELEN as _)];

    WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        descW,
        -1,
        core::ptr::addr_of_mut!(descA) as LPSTR,
        core::mem::size_of::<[::core::ffi::c_char; (MAXPNAMELEN as _)]>() as ::core::ffi::c_int,
        core::ptr::null::<CHAR>(),
        core::ptr::null_mut::<BOOL>(),
    );
    WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        modW,
        -1,
        core::ptr::addr_of_mut!(modA) as LPSTR,
        core::mem::size_of::<[::core::ffi::c_char; (MAXPNAMELEN as _)]>() as ::core::ffi::c_int,
        core::ptr::null::<CHAR>(),
        core::ptr::null_mut::<BOOL>(),
    );

    return (*context).callA.unwrap_unchecked()(
        guid,
        core::ptr::addr_of_mut!(descA) as *mut ::core::ffi::c_char as LPCSTR,
        core::ptr::addr_of_mut!(modA) as *mut ::core::ffi::c_char as LPCSTR,
        (*context).data,
    );
}

#[no_mangle]
pub unsafe extern "system" fn DirectSoundEnumerateA(
    lpDSEnumCallback: LPDSENUMCALLBACKA,
    lpContext: LPVOID,
) -> HRESULT {
    let mut context: morecontext = morecontext {
        callA: None,
        data: core::ptr::null_mut(),
    };

    if lpDSEnumCallback.is_none() {
        WARN!("invalid parameter: lpDSEnumCallback == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    context.callA = lpDSEnumCallback;
    context.data = lpContext;

    return DirectSoundEnumerateW(
        Some(a_to_w_callback),
        core::ptr::addr_of_mut!(context) as LPVOID,
    );
}

#[no_mangle]
pub unsafe extern "system" fn DirectSoundEnumerateW(
    lpDSEnumCallback: LPDSENUMCALLBACKW,
    lpContext: LPVOID,
) -> HRESULT {
    let mut devs: u32;
    let mut wod: u32;
    let mut capsW: WAVEOUTCAPSW = core::mem::zeroed();
    let mut guid: GUID = core::mem::zeroed();
    let mut err: i32;

    if lpDSEnumCallback.is_none() {
        WARN!("invalid parameter: lpDSEnumCallback == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    setup_dsound_options();

    devs = waveOutGetNumDevs();
    if devs > 0 {
        if GetDeviceID(core::ptr::addr_of!(DSDEVID_DefaultPlayback), core::ptr::addr_of_mut!(guid)) == (DS_OK as i32) {
            wod = 0;
            while wod < devs {
                if IsEqualGUID!(core::ptr::addr_of!(guid), core::ptr::addr_of_mut!(DSOUND_renderer_guids[wod as usize]) as LPCGUID) != false {
                    err = mmErr(waveOutGetDevCapsW(wod as UINT_PTR, core::ptr::addr_of_mut!(capsW), core::mem::size_of::<WAVEOUTCAPSW>() as UINT)) as i32;
                    if err == (DS_OK as i32) {
                        if (lpDSEnumCallback.unwrap_unchecked())(core::ptr::null_mut(), core::ptr::addr_of_mut!(capsW.szPname) as *mut u16 as LPCWSTR, RUSTCONV_WSTR_B996700897BA.as_ptr(), lpContext) == (FALSE as i32) {
                            return (DS_OK) as HRESULT;
                        }
                    }
                }
                wod = wod.wrapping_add(1);
            }
        }
    }

    wod = 0;
    while wod < devs {
        err = mmErr(waveOutGetDevCapsW(wod as UINT_PTR, core::ptr::addr_of_mut!(capsW), core::mem::size_of::<WAVEOUTCAPSW>() as UINT)) as i32;
        if err == (DS_OK as i32) {
            if (lpDSEnumCallback.unwrap_unchecked())(core::ptr::addr_of_mut!(DSOUND_renderer_guids[wod as usize]) as LPGUID, core::ptr::addr_of_mut!(capsW.szPname) as *mut u16 as LPCWSTR, RUSTCONV_WSTR_B996700897BA.as_ptr(), lpContext) == (FALSE as i32) {
                return (DS_OK) as HRESULT;
            }
        }
        wod = wod.wrapping_add(1);
    }
    return (DS_OK) as HRESULT;
}

#[no_mangle]
pub unsafe extern "system" fn DirectSoundCaptureEnumerateA(
    lpDSEnumCallback: LPDSENUMCALLBACKA,
    lpContext: LPVOID,
) -> HRESULT {
    let mut context: morecontext = morecontext {
        callA: None,
        data: core::ptr::null_mut(),
    };

    if lpDSEnumCallback.is_none() {
        WARN!("invalid parameter: lpDSEnumCallback == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    context.callA = lpDSEnumCallback;
    context.data = lpContext;

    return DirectSoundCaptureEnumerateW(
        Some(a_to_w_callback),
        core::ptr::addr_of_mut!(context) as LPVOID,
    );
}

#[no_mangle]
pub unsafe extern "system" fn DirectSoundCaptureEnumerateW(
    lpDSEnumCallback: LPDSENUMCALLBACKW,
    lpContext: LPVOID,
) -> HRESULT {
    let mut devs: c_uint = 0;
    let mut wid: c_uint = 0;
    let mut capsW: WAVEINCAPSW = core::mem::zeroed();
    let mut guid: GUID = core::mem::zeroed();
    let mut err: c_int = 0;

    if lpDSEnumCallback.is_none() {
        WARN!("invalid parameter: lpDSEnumCallback == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    setup_dsound_options();

    devs = waveInGetNumDevs();
    if devs > 0 {
        if GetDeviceID(
            core::ptr::addr_of!(DSDEVID_DefaultCapture),
            core::ptr::addr_of_mut!(guid),
        ) == (DS_OK as i32)
        {
            wid = 0;
            while wid < devs {
                if IsEqualGUID!(
                    core::ptr::addr_of_mut!(guid) as LPCGUID,
                    (core::ptr::addr_of_mut!(DSOUND_capture_guids) as *mut GUID)
                        .offset(wid as isize) as LPCGUID,
                ) != false
                {
                    err = mmErr(waveInGetDevCapsW(
                        wid as UINT_PTR,
                        core::ptr::addr_of_mut!(capsW),
                        (core::mem::size_of::<WAVEINCAPSW>()) as UINT,
                    ) as UINT) as c_int;
                    if err == (DS_OK as c_int) {
                        if (lpDSEnumCallback.unwrap_unchecked())(
                            core::ptr::null_mut::<GUID>(),
                            core::ptr::addr_of_mut!(capsW.szPname) as LPCWSTR,
                            RUSTCONV_WSTR_260153EE262B.as_ptr(),
                            lpContext,
                        ) == (FALSE as i32)
                        {
                            return (DS_OK) as HRESULT;
                        }
                    }
                }
                wid = wid.wrapping_add(1);
            }
        }
    }

    wid = 0;
    while wid < devs {
        err = mmErr(waveInGetDevCapsW(
            wid as UINT_PTR,
            core::ptr::addr_of_mut!(capsW),
            (core::mem::size_of::<WAVEINCAPSW>()) as UINT,
        ) as UINT) as c_int;
        if err == (DS_OK as c_int) {
            if (lpDSEnumCallback.unwrap_unchecked())(
                (core::ptr::addr_of_mut!(DSOUND_capture_guids) as *mut GUID)
                    .offset(wid as isize) as LPGUID,
                core::ptr::addr_of_mut!(capsW.szPname) as LPCWSTR,
                RUSTCONV_WSTR_260153EE262B.as_ptr(),
                lpContext,
            ) == (FALSE as i32)
            {
                return (DS_OK) as HRESULT;
            }
        }
        wid = wid.wrapping_add(1);
    }

    return (DS_OK) as HRESULT;
}

unsafe extern "C" fn impl_from_IClassFactory(iface: *mut IClassFactory) -> *mut IClassFactoryImpl {
    return (iface as *mut u8).wrapping_sub(core::mem::offset_of!(IClassFactoryImpl, IClassFactory_iface)) as *mut IClassFactoryImpl;
}
unsafe extern "system" fn DSCF_QueryInterface(iface: LPCLASSFACTORY, riid: *const IID, ppobj: *mut LPVOID) -> HRESULT {
    let This: *mut IClassFactoryImpl = impl_from_IClassFactory(iface);
    TRACE!("(%p, %s, %p)\n", This, debugstr_guid(riid), ppobj);
    if ppobj.is_null() {
        return E_POINTER;
    }
    if ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown))) != 0)
        || ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IClassFactory))) != 0)
    {
        *ppobj = iface as LPVOID;
        IUnknown_AddRef((iface).cast());
        return S_OK;
    }
    *ppobj = core::ptr::null_mut() as LPVOID;
    return E_NOINTERFACE;
}
unsafe extern "system" fn DSCF_AddRef(iface: LPCLASSFACTORY) -> ULONG {
    2
}
unsafe extern "system" fn DSCF_Release(iface: LPCLASSFACTORY) -> ULONG {
    1
}

unsafe extern "system" fn DSCF_CreateInstance(
    iface: LPCLASSFACTORY,
    pOuter: LPUNKNOWN,
    riid: REFIID,
    ppobj: *mut LPVOID,
) -> HRESULT {
    let This: *mut IClassFactoryImpl = impl_from_IClassFactory(iface as *mut IClassFactory);

    if !pOuter.is_null() {
        return CLASS_E_NOAGGREGATION;
    }

    if ppobj.is_null() {
        WARN!("invalid parameter\n");
        return DSERR_INVALIDPARAM;
    }
    *ppobj = core::ptr::null_mut();
    return (((*This).pfnCreateInstance).unwrap_unchecked())(riid, ppobj);
}

unsafe extern "system" fn DSCF_LockServer(iface: LPCLASSFACTORY, dolock: BOOL) -> HRESULT {
    let mut This: *mut IClassFactoryImpl = impl_from_IClassFactory(iface as *mut IClassFactory);
    WINE_FIXME!("(%p, %d) stub!\n", This, dolock);
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(
    rclsid: *const GUID,
    riid: *const GUID,
    ppv: *mut LPVOID,
) -> HRESULT {
    let mut i: i32 = 0;

    if ppv.is_null() {
        WARN!("invalid parameter\n");
        return E_INVALIDARG;
    }

    *ppv = core::ptr::null_mut();

    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IClassFactory)) == 0
        && IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) == 0
    {
        WARN!("no interface for %s\n", debugstr_guid(riid));
        return E_NOINTERFACE;
    }

    while !(*(core::ptr::addr_of!(DSOUND_CF) as *const IClassFactoryImpl)
        .offset(i as isize))
    .rclsid
    .is_null()
    {
        if IsEqualGUID!(
            rclsid,
            (*(core::ptr::addr_of!(DSOUND_CF) as *const IClassFactoryImpl)
                .offset(i as isize))
            .rclsid,
        ) != false
        {
            let cf: *mut IClassFactoryImpl =
                (core::ptr::addr_of_mut!(DSOUND_CF) as *mut IClassFactoryImpl)
                    .offset(i as isize);
            DSCF_AddRef(core::ptr::addr_of_mut!((*cf).IClassFactory_iface));
            *ppv = (core::ptr::addr_of_mut!(DSOUND_CF) as *mut IClassFactoryImpl)
                .offset(i as isize) as LPVOID;
            return S_OK;
        }
        i += 1;
    }

    WARN!(
        "(%s, %s, %p): no class found.\n",
        debugstr_guid(rclsid),
        debugstr_guid(riid),
        ppv
    );
    return CLASS_E_CLASSNOTAVAILABLE;
}

#[no_mangle]
pub unsafe extern "system" fn DllCanUnloadNow() -> HRESULT {
    S_FALSE
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(mut hInstDLL: HINSTANCE, fdwReason: DWORD, lpvReserved: LPVOID) -> BOOL {
    let mut i: ::core::ffi::c_int = 0;
    WINE_TRACE!("(%p %d %p)\n", hInstDLL, fdwReason, lpvReserved);
    match fdwReason {
        DLL_PROCESS_ATTACH => {
            WINE_TRACE!("DLL_PROCESS_ATTACH\n");
            i = 0 as ::core::ffi::c_int;
            while i < MAXWAVEDRIVERS as ::core::ffi::c_int {
                DSOUND_renderer[i as usize] = ::core::ptr::null_mut::<DirectSoundDevice>();
                DSOUND_capture[i as usize] = ::core::ptr::null_mut::<DirectSoundCaptureDevice>();
                INIT_GUID!(
                    DSOUND_renderer_guids[i as usize],
                    0xbd6dd71a,
                    0x3deb,
                    0x11d1,
                    0xb1,
                    0x71,
                    0x00,
                    0xc0,
                    0x4f,
                    0xc2,
                    0x00,
                    (0x00 + i) as u8
                );
                INIT_GUID!(
                    DSOUND_capture_guids[i as usize],
                    0xbd6dd71b,
                    0x3deb,
                    0x11d1,
                    0xb1,
                    0x71,
                    0x00,
                    0xc0,
                    0x4f,
                    0xc2,
                    0x00,
                    (0x00 + i) as u8
                );
                i += 1;
            }
            instance = hInstDLL;
            DisableThreadLibraryCalls(hInstDLL as HMODULE);
            /* Increase refcount on dsound by 1 */
            GetModuleHandleExW(
                GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS as DWORD,
                hInstDLL as LPCWSTR,
                core::ptr::addr_of_mut!(hInstDLL),
            );
        }
        DLL_PROCESS_DETACH => {
            WINE_TRACE!("DLL_PROCESS_DETACH\n");
        }
        _ => {
            WINE_TRACE!("UNKNOWN REASON\n");
        }
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn DllRegisterServer() -> HRESULT {
    return __wine_register_resources(instance as HMODULE);
}
#[no_mangle]
pub unsafe extern "system" fn DllUnregisterServer() -> HRESULT {
    return __wine_unregister_resources(instance as HMODULE);
}

// COM vtable tables — generated from dsound_main.c by vtbl_gen.py
/// dsound_main.c:530 `static const IClassFactoryVtbl DSCF_Vtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static DSCF_Vtbl: IClassFactoryVtbl = IClassFactoryVtbl {
    QueryInterface: Some(DSCF_QueryInterface),
    AddRef: Some(DSCF_AddRef),
    Release: Some(DSCF_Release),
    CreateInstance: Some(DSCF_CreateInstance),
    LockServer: Some(DSCF_LockServer),
};
