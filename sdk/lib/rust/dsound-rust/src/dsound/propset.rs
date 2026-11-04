use crate::dsound::dsound_main::DSOUND_capture_guids;
use crate::dsound::dsound_main::DSOUND_renderer_guids;
use crate::dsound::dsound_main::ds_hw_accel;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct IKsPrivatePropertySetImpl {
    pub IKsPropertySet_iface: IKsPropertySet,
    pub ref_: LONG,
}
unsafe extern "C" fn impl_from_IKsPropertySet(iface: *mut IKsPropertySet) -> *mut IKsPrivatePropertySetImpl {
    return CONTAINING_RECORD!(iface, IKsPrivatePropertySetImpl, IKsPropertySet_iface);
}

unsafe extern "system" fn IKsPrivatePropertySetImpl_QueryInterface(
    iface: LPKSPROPERTYSET,
    riid: REFIID,
    ppobj: *mut LPVOID,
) -> HRESULT {
    let mut This: *mut IKsPrivatePropertySetImpl = impl_from_IKsPropertySet(iface as *mut IKsPropertySet);

    if ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown))) != 0)
        || ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IKsPropertySet))) != 0)
    {
        *ppobj = iface as LPVOID;
        IUnknown_AddRef(iface as *mut IUnknown);
        return S_OK;
    }
    *ppobj = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn IKsPrivatePropertySetImpl_AddRef(iface: LPKSPROPERTYSET) -> ULONG {
    let This: *mut IKsPrivatePropertySetImpl =
        impl_from_IKsPropertySet(iface as *mut IKsPropertySet);
    let ref_0: ULONG =
        InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_) as *mut core::ffi::c_long)
            as ULONG;
    return ref_0;
}

unsafe extern "system" fn IKsPrivatePropertySetImpl_Release(iface: LPKSPROPERTYSET) -> ULONG {
    let This: *mut IKsPrivatePropertySetImpl =
        impl_from_IKsPropertySet(iface as *mut IKsPropertySet);
    let ref_0: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*This).ref_) as *mut ::core::ffi::c_long,
    ) as ULONG;

    if ref_0 == 0 {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            This as LPVOID,
        );
    }
    return ref_0;
}

unsafe extern "C" fn DSPROPERTY_WaveDeviceMappingW(
    mut pPropData: LPVOID,
    mut cbPropData: ULONG,
    mut pcbReturned: PULONG,
) -> HRESULT {
    let mut hr: HRESULT = DSERR_INVALIDPARAM;
    let mut ppd: PDSPROPERTY_DIRECTSOUNDDEVICE_WAVEDEVICEMAPPING_W_DATA =
        core::ptr::null_mut::<_DSPROPERTY_DIRECTSOUNDDEVICE_WAVEDEVICEMAPPING_W_DATA>();

    ppd = pPropData as PDSPROPERTY_DIRECTSOUNDDEVICE_WAVEDEVICEMAPPING_W_DATA;

    if ppd.is_null() {
        WARN!("invalid parameter: pPropData\n");
        return DSERR_INVALIDPARAM;
    }

    if (*ppd).DataFlow == DIRECTSOUNDDEVICE_DATAFLOW_RENDER {
        let mut wod: ULONG = 0;
        let mut wodn: core::ffi::c_uint = 0;

        wodn = waveOutGetNumDevs() as core::ffi::c_uint;
        wod = 0 as ULONG;
        while wod < wodn as ULONG {
            let mut capsW: WAVEOUTCAPSW = core::mem::zeroed();
            let mut res: MMRESULT = 0;

            res = waveOutGetDevCapsW(
                wod as UINT_PTR,
                core::ptr::addr_of_mut!(capsW),
                core::mem::size_of::<WAVEOUTCAPSW>() as UINT,
            );
            if res == MMSYSERR_NOERROR {
                if lstrcmpW(
                    core::ptr::addr_of_mut!(capsW.szPname) as *mut WCHAR as LPCWSTR,
                    (*ppd).DeviceName as LPCWSTR,
                ) == 0
                {
                    (*ppd).DeviceId = DSOUND_renderer_guids[wod as usize];
                    hr = (DS_OK as i32);

                    break;
                }
            }
            wod = wod.wrapping_add(1);
        }
    } else if (*ppd).DataFlow == DIRECTSOUNDDEVICE_DATAFLOW_CAPTURE {
        let mut wid: ULONG = 0;
        let mut widn: core::ffi::c_uint = 0;

        widn = waveInGetNumDevs() as core::ffi::c_uint;
        wid = 0 as ULONG;
        while wid < widn as ULONG {
            let mut capsW_0: WAVEINCAPSW = core::mem::zeroed();
            let mut res_0: MMRESULT = 0;

            res_0 = waveInGetDevCapsW(
                wid as UINT_PTR,
                core::ptr::addr_of_mut!(capsW_0),
                core::mem::size_of::<WAVEINCAPSW>() as UINT,
            );
            if res_0 == MMSYSERR_NOERROR {
                if lstrcmpW(
                    core::ptr::addr_of_mut!(capsW_0.szPname) as *mut WCHAR as LPCWSTR,
                    (*ppd).DeviceName as LPCWSTR,
                ) == 0
                {
                    (*ppd).DeviceId = DSOUND_capture_guids[wid as usize];
                    hr = (DS_OK as i32);

                    break;
                }
            }
            wid = wid.wrapping_add(1);
        }
    }

    if !pcbReturned.is_null() {
        *pcbReturned = cbPropData;
    }

    return hr;
}

unsafe extern "C" fn DSPROPERTY_WaveDeviceMappingA(
    pPropData: LPVOID,
    cbPropData: ULONG,
    pcbReturned: PULONG,
) -> HRESULT {
    let ppd: *mut DSPROPERTY_DIRECTSOUNDDEVICE_WAVEDEVICEMAPPING_A_DATA =
        pPropData as *mut DSPROPERTY_DIRECTSOUNDDEVICE_WAVEDEVICEMAPPING_A_DATA;
    let mut data: DSPROPERTY_DIRECTSOUNDDEVICE_WAVEDEVICEMAPPING_W_DATA = core::mem::zeroed();
    let mut len: DWORD = 0;
    let mut hr: HRESULT;

    if ppd.is_null() || (*ppd).DeviceName.is_null() {
        WARN!("invalid parameter: ppd=%p\n", ppd);
        return DSERR_INVALIDPARAM;
    }

    data.DataFlow = (*ppd).DataFlow;
    len = MultiByteToWideChar(
        CP_ACP as UINT,
        0 as DWORD,
        (*ppd).DeviceName as LPCSTR,
        -(1 as ::core::ffi::c_int),
        core::ptr::null_mut::<WCHAR>(),
        0 as ::core::ffi::c_int,
    ) as DWORD;
    data.DeviceName = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        (len as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    if data.DeviceName.is_null() {
        return E_OUTOFMEMORY;
    }
    MultiByteToWideChar(
        CP_ACP as UINT,
        0 as DWORD,
        (*ppd).DeviceName as LPCSTR,
        -(1 as ::core::ffi::c_int),
        data.DeviceName,
        len as ::core::ffi::c_int,
    );

    hr = DSPROPERTY_WaveDeviceMappingW(
        core::ptr::addr_of_mut!(data) as LPVOID,
        cbPropData,
        pcbReturned,
    );
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        data.DeviceName as PVOID,
    );
    (*ppd).DeviceId = data.DeviceId;

    if !pcbReturned.is_null() {
        *pcbReturned = cbPropData;
    }

    return hr;
}

unsafe extern "C" fn DSPROPERTY_DescriptionW(
    pPropData: LPVOID,
    cbPropData: ULONG,
    pcbReturned: PULONG,
) -> HRESULT {
    let ppd: PDSPROPERTY_DIRECTSOUNDDEVICE_DESCRIPTION_W_DATA =
        pPropData as PDSPROPERTY_DIRECTSOUNDDEVICE_DESCRIPTION_W_DATA;
    let mut err: HRESULT = 0;
    let mut dev_guid: GUID = core::mem::zeroed();
    let mut wod: ULONG;
    let mut wid: ULONG;
    let mut wodn: ULONG;
    let mut widn: ULONG;
    let mut desc: DSDRIVERDESC = core::mem::zeroed();

    if IsEqualGUID!(
        core::ptr::addr_of!((*ppd).DeviceId),
        core::ptr::addr_of!(GUID_NULL),
    ) != false
    {
        /* default device of type specified by ppd->DataFlow */
        if (*ppd).DataFlow == DIRECTSOUNDDEVICE_DATAFLOW_CAPTURE {
            (*ppd).DeviceId = DSDEVID_DefaultCapture;
        } else if (*ppd).DataFlow == DIRECTSOUNDDEVICE_DATAFLOW_RENDER {
            (*ppd).DeviceId = DSDEVID_DefaultPlayback;
        } else {
            WARN!("DataFlow=Unknown(%d)\n", (*ppd).DataFlow);
            return E_PROP_ID_UNSUPPORTED;
        }
    }

    setup_dsound_options();

    GetDeviceID(
        core::ptr::addr_of_mut!((*ppd).DeviceId),
        core::ptr::addr_of_mut!(dev_guid),
    );

    wodn = waveOutGetNumDevs() as ULONG;
    widn = waveInGetNumDevs() as ULONG;
    wod = dev_guid.Data4[7] as ULONG;
    wid = wod;
    if (memcmp(
        core::ptr::addr_of_mut!(dev_guid) as *const core::ffi::c_void,
        core::ptr::addr_of!(DSOUND_renderer_guids[0]) as *const core::ffi::c_void,
        ((core::mem::size_of::<GUID>() as SIZE_T).wrapping_sub(1 as SIZE_T)) as _,
    ) == 0)
        && wod < wodn
    {
        (*ppd).DataFlow = DIRECTSOUNDDEVICE_DATAFLOW_RENDER;
        (*ppd).WaveDeviceId = wod;
    } else if (memcmp(
        core::ptr::addr_of_mut!(dev_guid) as *const core::ffi::c_void,
        core::ptr::addr_of!(DSOUND_capture_guids[0]) as *const core::ffi::c_void,
        ((core::mem::size_of::<GUID>() as SIZE_T).wrapping_sub(1 as SIZE_T)) as _,
    ) == 0)
        && wid < widn
    {
        (*ppd).DataFlow = DIRECTSOUNDDEVICE_DATAFLOW_CAPTURE;
        (*ppd).WaveDeviceId = wid;
    } else {
        WARN!("Device not found\n");
        return E_PROP_ID_UNSUPPORTED;
    }

    if (*ppd).DataFlow == DIRECTSOUNDDEVICE_DATAFLOW_RENDER {
        err = waveOutMessage(
            (UlongToHandle!(wod)).cast(),
            DRV_QUERYDSOUNDDESC,
            core::ptr::addr_of_mut!(desc) as DWORD_PTR,
            (ds_hw_accel) as DWORD_PTR,
        ) as HRESULT;
    } else {
        err = waveInMessage(
            (UlongToHandle!(wod)).cast(),
            DRV_QUERYDSOUNDDESC,
            core::ptr::addr_of_mut!(desc) as DWORD_PTR,
            (ds_hw_accel) as DWORD_PTR,
        ) as HRESULT;
    }

    if err != (MMSYSERR_NOERROR as HRESULT) {
        WARN!("waveMessage(DRV_QUERYDSOUNDDESC) failed!\n");
        return E_PROP_ID_UNSUPPORTED;
    } else {
        /* FIXME: Still a memory leak.. */
        let mut desclen: i32;
        let mut modlen: i32;
        static mut wInterface: [u16; 10] = [0x0049u16, 0x006E, 0x0074, 0x0065, 0x0072, 0x0066, 0x0061, 0x0063, 0x0065, 0x0000];

        modlen = MultiByteToWideChar(
            CP_ACP,
            0,
            core::ptr::addr_of_mut!(desc.szDrvname) as *mut CHAR as LPCSTR,
            -(1),
            core::ptr::null_mut(),
            0,
        );
        desclen = MultiByteToWideChar(
            CP_ACP,
            0,
            core::ptr::addr_of_mut!(desc.szDesc) as *mut CHAR as LPCSTR,
            -(1),
            core::ptr::null_mut(),
            0,
        );
        (*ppd).Module = HeapAlloc(
            GetProcessHeap(),
            0,
            (modlen as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as LPWSTR;
        (*ppd).Description = HeapAlloc(
            GetProcessHeap(),
            0,
            (desclen as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as LPWSTR;
        (*ppd).Interface = core::ptr::addr_of_mut!(wInterface) as *mut WCHAR as LPWSTR;
        if (*ppd).Description.is_null() || (*ppd).Module.is_null() {
            WARN!("Out of memory\n");
            HeapFree(
                GetProcessHeap(),
                0,
                (*ppd).Description as PVOID,
            );
            HeapFree(
                GetProcessHeap(),
                0,
                (*ppd).Module as PVOID,
            );
            (*ppd).Module = core::ptr::null_mut();
            (*ppd).Description = (*ppd).Module;
            return E_OUTOFMEMORY;
        }

        MultiByteToWideChar(
            CP_ACP,
            0,
            core::ptr::addr_of_mut!(desc.szDrvname) as *mut CHAR as LPCSTR,
            -(1),
            (*ppd).Module,
            modlen,
        );
        MultiByteToWideChar(
            CP_ACP,
            0,
            core::ptr::addr_of_mut!(desc.szDesc) as *mut CHAR as LPCSTR,
            -(1),
            (*ppd).Description,
            desclen,
        );
    }

    (*ppd).Type = DIRECTSOUNDDEVICE_TYPE_VXD;

    if !pcbReturned.is_null() {
        *pcbReturned =
            core::mem::size_of::<_DSPROPERTY_DIRECTSOUNDDEVICE_DESCRIPTION_W_DATA>() as ULONG;
    }

    return S_OK;
}

unsafe extern "C" fn DSPROPERTY_EnumerateW(
    pPropData: LPVOID,
    cbPropData: ULONG,
    pcbReturned: PULONG,
) -> HRESULT {
    let mut ppd: PDSPROPERTY_DIRECTSOUNDDEVICE_ENUMERATE_W_DATA =
        pPropData as PDSPROPERTY_DIRECTSOUNDDEVICE_ENUMERATE_W_DATA;
    let mut data: DSPROPERTY_DIRECTSOUNDDEVICE_DESCRIPTION_W_DATA = core::mem::zeroed();
    let mut ret: BOOL;
    let mut widn: ::core::ffi::c_int;
    let mut wodn: ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int;

    if !pcbReturned.is_null() {
        *pcbReturned = 0 as ULONG;
    }

    if ppd.is_null() || (*ppd).Callback.is_none() {
        WARN!("Invalid ppd %p\n", ppd);
        return E_PROP_ID_UNSUPPORTED;
    }

    wodn = waveOutGetNumDevs() as ::core::ffi::c_int;
    widn = waveInGetNumDevs() as ::core::ffi::c_int;

    data.DeviceId = DSOUND_renderer_guids[0 as usize];
    i = 0 as ::core::ffi::c_int;
    while i < wodn {
        let mut hr: HRESULT = 0;
        data.DeviceId.Data4[7 as usize] = i as u8;
        hr = DSPROPERTY_DescriptionW(
            core::ptr::addr_of_mut!(data) as LPVOID,
            core::mem::size_of::<DSPROPERTY_DIRECTSOUNDDEVICE_DESCRIPTION_W_DATA>() as ULONG,
            core::ptr::null_mut::<ULONG>(),
        );
        if hr < 0 as HRESULT {
            ERR!("DescriptionW failed!\n");
            return S_OK;
        }
        ret = ((*ppd).Callback.unwrap_unchecked())(
            core::ptr::addr_of_mut!(data),
            (*ppd).Context,
        );
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            data.Module as PVOID,
        );
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            data.Description as PVOID,
        );
        if ret == 0 {
            return S_OK;
        }
        i += 1;
    }

    data.DeviceId = DSOUND_capture_guids[0 as usize];
    i = 0 as ::core::ffi::c_int;
    while i < widn {
        let mut hr: HRESULT = 0;
        data.DeviceId.Data4[7 as usize] = i as u8;
        hr = DSPROPERTY_DescriptionW(
            core::ptr::addr_of_mut!(data) as LPVOID,
            core::mem::size_of::<DSPROPERTY_DIRECTSOUNDDEVICE_DESCRIPTION_W_DATA>() as ULONG,
            core::ptr::null_mut::<ULONG>(),
        );
        if hr < 0 as HRESULT {
            ERR!("DescriptionW failed!\n");
            return S_OK;
        }
        ret = ((*ppd).Callback.unwrap_unchecked())(
            core::ptr::addr_of_mut!(data),
            (*ppd).Context,
        );
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            data.Module as PVOID,
        );
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            data.Description as PVOID,
        );
        if ret == 0 {
            return S_OK;
        }
        i += 1;
    }
    return S_OK;
}

unsafe extern "C" fn DSPROPERTY_descWtoA(
    dataW: *const DSPROPERTY_DIRECTSOUNDDEVICE_DESCRIPTION_W_DATA,
    dataA: *mut DSPROPERTY_DIRECTSOUNDDEVICE_DESCRIPTION_A_DATA,
) -> BOOL {
    let mut modlen: DWORD = 0;
    let mut desclen: DWORD = 0;
    static mut Interface: [u8; 10] = *b"Interface\0";
    modlen = WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        (*dataW).Module as LPCWSTR,
        -(1 as ::core::ffi::c_int),
        ::core::ptr::null_mut::<CHAR>(),
        0 as ::core::ffi::c_int,
        ::core::ptr::null::<CHAR>(),
        ::core::ptr::null_mut::<BOOL>(),
    ) as DWORD;
    desclen = WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        (*dataW).Description as LPCWSTR,
        -(1 as ::core::ffi::c_int),
        ::core::ptr::null_mut::<CHAR>(),
        0 as ::core::ffi::c_int,
        ::core::ptr::null::<CHAR>(),
        ::core::ptr::null_mut::<BOOL>(),
    ) as DWORD;
    (*dataA).Type = (*dataW).Type;
    (*dataA).DataFlow = (*dataW).DataFlow;
    (*dataA).DeviceId = (*dataW).DeviceId;
    (*dataA).WaveDeviceId = (*dataW).WaveDeviceId;
    (*dataA).Interface = core::ptr::addr_of_mut!(Interface) as *mut ::core::ffi::c_char as LPSTR;
    (*dataA).Module = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        modlen as SIZE_T,
    ) as LPSTR;
    (*dataA).Description = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        desclen as SIZE_T,
    ) as LPSTR;
    if (*dataA).Module.is_null() || (*dataA).Description.is_null() {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            (*dataA).Module as PVOID,
        );
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            (*dataA).Description as PVOID,
        );
        (*dataA).Description = ::core::ptr::null_mut::<CHAR>();
        (*dataA).Module = (*dataA).Description;
        return FALSE as BOOL;
    }

    WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        (*dataW).Module as LPCWSTR,
        -(1 as ::core::ffi::c_int),
        (*dataA).Module,
        modlen as ::core::ffi::c_int,
        ::core::ptr::null::<CHAR>(),
        ::core::ptr::null_mut::<BOOL>(),
    );
    WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        (*dataW).Description as LPCWSTR,
        -(1 as ::core::ffi::c_int),
        (*dataA).Description,
        desclen as ::core::ffi::c_int,
        ::core::ptr::null::<CHAR>(),
        ::core::ptr::null_mut::<BOOL>(),
    );
    return TRUE as BOOL;
}

unsafe extern "C" fn DSPROPERTY_descWto1(
    dataW: *const DSPROPERTY_DIRECTSOUNDDEVICE_DESCRIPTION_W_DATA,
    data1: *mut DSPROPERTY_DIRECTSOUNDDEVICE_DESCRIPTION_1_DATA,
) {
    (*data1).DeviceId = (*dataW).DeviceId;
    lstrcpynW(
        core::ptr::addr_of_mut!((*data1).ModuleW) as LPWSTR,
        core::ptr::addr_of!((*dataW).Module) as LPCWSTR,
        (core::mem::size_of::<[WCHAR; 260]>() / core::mem::size_of::<WCHAR>()) as ::core::ffi::c_int,
    );
    lstrcpynW(
        core::ptr::addr_of_mut!((*data1).DescriptionW) as LPWSTR,
        core::ptr::addr_of!((*dataW).Description) as LPCWSTR,
        (core::mem::size_of::<[WCHAR; 256]>() / core::mem::size_of::<WCHAR>()) as ::core::ffi::c_int,
    );
    WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        core::ptr::addr_of_mut!((*data1).DescriptionW) as *mut WCHAR as LPCWSTR,
        -(1 as ::core::ffi::c_int),
        core::ptr::addr_of_mut!((*data1).DescriptionA) as LPSTR,
        (core::mem::size_of::<[CHAR; 256]>() as usize).wrapping_sub(1 as usize)
            as ::core::ffi::c_int,
        ::core::ptr::null::<CHAR>(),
        ::core::ptr::null_mut::<BOOL>(),
    );
    WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        core::ptr::addr_of_mut!((*data1).ModuleW) as *mut WCHAR as LPCWSTR,
        -(1 as ::core::ffi::c_int),
        core::ptr::addr_of_mut!((*data1).ModuleA) as LPSTR,
        (core::mem::size_of::<[CHAR; 260]>() as usize).wrapping_sub(1 as usize)
            as ::core::ffi::c_int,
        ::core::ptr::null::<CHAR>(),
        ::core::ptr::null_mut::<BOOL>(),
    );
    (*data1).DescriptionA[(core::mem::size_of::<[CHAR; 256]>() as usize)
        .wrapping_sub(1 as usize) as usize] = 0 as CHAR;
    (*data1).ModuleA[(core::mem::size_of::<[CHAR; 260]>() as usize)
        .wrapping_sub(1 as usize) as usize] = 0 as CHAR;
    (*data1).Type = (*dataW).Type;
    (*data1).DataFlow = (*dataW).DataFlow;
    (*data1).Devnode = (*dataW).WaveDeviceId;
    (*data1).WaveDeviceId = (*data1).Devnode;
}

unsafe extern "system" fn DSPROPERTY_enumWtoA(
    descW: *mut DSPROPERTY_DIRECTSOUNDDEVICE_DESCRIPTION_W_DATA,
    data: *mut core::ffi::c_void,
) -> BOOL {
    let mut descA: DSPROPERTY_DIRECTSOUNDDEVICE_DESCRIPTION_A_DATA = core::mem::zeroed();
    let ppd: *mut DSPROPERTY_DIRECTSOUNDDEVICE_ENUMERATE_A_DATA =
        data as *mut DSPROPERTY_DIRECTSOUNDDEVICE_ENUMERATE_A_DATA;
    let mut ret: BOOL;

    ret = DSPROPERTY_descWtoA(descW, core::ptr::addr_of_mut!(descA));
    if ret == 0 {
        return (FALSE) as BOOL;
    }
    ret = (*ppd).Callback.unwrap_unchecked()(
        core::ptr::addr_of_mut!(descA),
        (*ppd).Context,
    );
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        descA.Module as PVOID,
    );
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        descA.Description as PVOID,
    );
    return ret;
}

unsafe extern "C" fn DSPROPERTY_EnumerateA(
    pPropData: LPVOID,
    cbPropData: ULONG,
    pcbReturned: PULONG,
) -> HRESULT {
    let mut ppd: *mut DSPROPERTY_DIRECTSOUNDDEVICE_ENUMERATE_A_DATA =
        pPropData as *mut DSPROPERTY_DIRECTSOUNDDEVICE_ENUMERATE_A_DATA;
    let mut data: DSPROPERTY_DIRECTSOUNDDEVICE_ENUMERATE_W_DATA =
        DSPROPERTY_DIRECTSOUNDDEVICE_ENUMERATE_W_DATA {
            Callback: None,
            Context: core::ptr::null_mut(),
        };

    if ppd.is_null() || (*ppd).Callback.is_none() {
        WARN!("Invalid ppd %p\n", ppd);
        return E_PROP_ID_UNSUPPORTED;
    }

    data.Callback = Some(DSPROPERTY_enumWtoA);
    data.Context = ppd as LPVOID;

    return DSPROPERTY_EnumerateW(&raw mut data as LPVOID, cbPropData, pcbReturned);
}
unsafe extern "system" fn DSPROPERTY_enumWto1(
    mut descW: *mut DSPROPERTY_DIRECTSOUNDDEVICE_DESCRIPTION_W_DATA,
    mut data: *mut core::ffi::c_void,
) -> BOOL {
    let mut desc1: DSPROPERTY_DIRECTSOUNDDEVICE_DESCRIPTION_1_DATA = core::mem::zeroed();
    let mut ppd: *mut DSPROPERTY_DIRECTSOUNDDEVICE_ENUMERATE_1_DATA =
        data as *mut DSPROPERTY_DIRECTSOUNDDEVICE_ENUMERATE_1_DATA;
    let mut ret: BOOL = 0;
    DSPROPERTY_descWto1(descW, core::ptr::addr_of_mut!(desc1));
    ret = ((*ppd).Callback.unwrap_unchecked())(
        core::ptr::addr_of_mut!(desc1),
        (*ppd).Context,
    );
    return ret;
}

unsafe extern "C" fn DSPROPERTY_Enumerate1(
    pPropData: LPVOID,
    cbPropData: ULONG,
    pcbReturned: PULONG,
) -> HRESULT {
    let ppd: *mut DSPROPERTY_DIRECTSOUNDDEVICE_ENUMERATE_1_DATA =
        pPropData as *mut DSPROPERTY_DIRECTSOUNDDEVICE_ENUMERATE_1_DATA;
    let mut data: DSPROPERTY_DIRECTSOUNDDEVICE_ENUMERATE_W_DATA =
        DSPROPERTY_DIRECTSOUNDDEVICE_ENUMERATE_W_DATA {
            Callback: None,
            Context: core::ptr::null_mut(),
        };

    if ppd.is_null() || (*ppd).Callback.is_none() {
        WARN!("Invalid ppd %p\n", ppd);
        return E_PROP_ID_UNSUPPORTED;
    }

    data.Callback = Some(DSPROPERTY_enumWto1);
    data.Context = ppd as LPVOID;

    return DSPROPERTY_EnumerateW(
        core::ptr::addr_of_mut!(data) as LPVOID,
        cbPropData,
        pcbReturned,
    );
}

unsafe extern "C" fn DSPROPERTY_DescriptionA(
    mut pPropData: LPVOID,
    mut cbPropData: ULONG,
    mut pcbReturned: PULONG,
) -> HRESULT {
    let mut data: DSPROPERTY_DIRECTSOUNDDEVICE_DESCRIPTION_W_DATA = core::mem::zeroed();
    let mut ppd: *mut DSPROPERTY_DIRECTSOUNDDEVICE_DESCRIPTION_A_DATA =
        pPropData as *mut DSPROPERTY_DIRECTSOUNDDEVICE_DESCRIPTION_A_DATA;
    let mut hr: HRESULT = 0;
    if !pcbReturned.is_null() {
        *pcbReturned =
            ::core::mem::size_of::<DSPROPERTY_DIRECTSOUNDDEVICE_DESCRIPTION_A_DATA>() as ULONG;
    }
    if pPropData.is_null() {
        return S_OK;
    }
    data.DeviceId = (*ppd).DeviceId;
    data.DataFlow = (*ppd).DataFlow;
    hr = DSPROPERTY_DescriptionW(
        core::ptr::addr_of_mut!(data) as LPVOID,
        ::core::mem::size_of::<DSPROPERTY_DIRECTSOUNDDEVICE_DESCRIPTION_W_DATA>() as ULONG,
        ::core::ptr::null_mut::<ULONG>(),
    );
    if hr < 0 {
        return hr;
    }
    if DSPROPERTY_descWtoA(core::ptr::addr_of_mut!(data), ppd) == 0 {
        hr = E_OUTOFMEMORY;
    }
    HeapFree(
        GetProcessHeap(),
        0,
        data.Module as PVOID,
    );
    HeapFree(
        GetProcessHeap(),
        0,
        data.Interface as PVOID,
    );
    return hr;
}

unsafe extern "C" fn DSPROPERTY_Description1(
    mut pPropData: LPVOID,
    mut cbPropData: ULONG,
    mut pcbReturned: PULONG,
) -> HRESULT {
    let mut data: DSPROPERTY_DIRECTSOUNDDEVICE_DESCRIPTION_W_DATA = core::mem::zeroed();
    let mut ppd: *mut DSPROPERTY_DIRECTSOUNDDEVICE_DESCRIPTION_1_DATA =
        pPropData as *mut DSPROPERTY_DIRECTSOUNDDEVICE_DESCRIPTION_1_DATA;
    let mut hr: HRESULT;

    if !pcbReturned.is_null() {
        *pcbReturned =
            core::mem::size_of::<DSPROPERTY_DIRECTSOUNDDEVICE_DESCRIPTION_1_DATA>() as ULONG;
    }
    if pPropData.is_null() {
        return S_OK;
    }

    data.DeviceId = (*ppd).DeviceId;
    data.DataFlow = (*ppd).DataFlow;
    hr = DSPROPERTY_DescriptionW(
        core::ptr::addr_of_mut!(data) as LPVOID,
        core::mem::size_of::<DSPROPERTY_DIRECTSOUNDDEVICE_DESCRIPTION_W_DATA>() as ULONG,
        core::ptr::null_mut::<ULONG>(),
    );
    if hr < 0 {
        return hr;
    }
    DSPROPERTY_descWto1(core::ptr::addr_of_mut!(data), ppd);
    HeapFree(GetProcessHeap(), 0, data.Module as LPVOID);
    HeapFree(GetProcessHeap(), 0, data.Interface as LPVOID);
    return hr;
}

unsafe extern "system" fn IKsPrivatePropertySetImpl_Get(
    iface: LPKSPROPERTYSET,
    guidPropSet: *const GUID,
    dwPropID: ULONG,
    pInstanceData: LPVOID,
    cbInstanceData: ULONG,
    pPropData: LPVOID,
    cbPropData: ULONG,
    pcbReturned: PULONG,
) -> HRESULT {
    #[allow(unused_mut, unused_variables)]
    let mut This: *mut IKsPrivatePropertySetImpl = impl_from_IKsPropertySet(iface as *mut IKsPropertySet);

    if IsEqualGUID!(core::ptr::addr_of!(DSPROPSETID_DirectSoundDevice), guidPropSet) != false {
        match dwPropID {
            DSPROPERTY_DIRECTSOUNDDEVICE_WAVEDEVICEMAPPING_A => return DSPROPERTY_WaveDeviceMappingA(pPropData, cbPropData, pcbReturned),
            DSPROPERTY_DIRECTSOUNDDEVICE_DESCRIPTION_1 => return DSPROPERTY_Description1(pPropData, cbPropData, pcbReturned),
            DSPROPERTY_DIRECTSOUNDDEVICE_ENUMERATE_1 => return DSPROPERTY_Enumerate1(pPropData, cbPropData, pcbReturned),
            DSPROPERTY_DIRECTSOUNDDEVICE_WAVEDEVICEMAPPING_W => return DSPROPERTY_WaveDeviceMappingW(pPropData, cbPropData, pcbReturned),
            DSPROPERTY_DIRECTSOUNDDEVICE_DESCRIPTION_A => return DSPROPERTY_DescriptionA(pPropData, cbPropData, pcbReturned),
            DSPROPERTY_DIRECTSOUNDDEVICE_DESCRIPTION_W => return DSPROPERTY_DescriptionW(pPropData, cbPropData, pcbReturned),
            DSPROPERTY_DIRECTSOUNDDEVICE_ENUMERATE_A => return DSPROPERTY_EnumerateA(pPropData, cbPropData, pcbReturned),
            DSPROPERTY_DIRECTSOUNDDEVICE_ENUMERATE_W => return DSPROPERTY_EnumerateW(pPropData, cbPropData, pcbReturned),
            _ => {
                FIXME!("unsupported ID: %d\n", dwPropID);
            }
        }
    } else {
    }

    if !pcbReturned.is_null() {
        *pcbReturned = 0;
    }

    return E_PROP_ID_UNSUPPORTED;
}

unsafe extern "system" fn IKsPrivatePropertySetImpl_Set(
    iface: LPKSPROPERTYSET,
    guidPropSet: *const GUID,
    dwPropID: u32,
    pInstanceData: LPVOID,
    cbInstanceData: u32,
    pPropData: LPVOID,
    cbPropData: u32,
) -> HRESULT {
    let mut This: *mut IKsPrivatePropertySetImpl =
        impl_from_IKsPropertySet(iface as *mut IKsPropertySet);
    WINE_FIXME!("(%p,%s,%d,%p,%d,%p,%d), stub!\n", This, debugstr_guid(guidPropSet as *const _GUID), dwPropID, pInstanceData, cbInstanceData, pPropData, cbPropData);
    return E_PROP_ID_UNSUPPORTED;
}

unsafe extern "system" fn IKsPrivatePropertySetImpl_QuerySupport(
    iface: LPKSPROPERTYSET,
    guidPropSet: REFGUID,
    dwPropID: ULONG,
    pTypeSupport: PULONG,
) -> HRESULT {
    let This: *mut IKsPrivatePropertySetImpl = impl_from_IKsPropertySet(iface as *mut IKsPropertySet);

    if IsEqualGUID!(core::ptr::addr_of!(DSPROPSETID_DirectSoundDevice), guidPropSet) != false {
        match dwPropID {
            DSPROPERTY_DIRECTSOUNDDEVICE_WAVEDEVICEMAPPING_A => {
                *pTypeSupport = KSPROPERTY_SUPPORT_GET;
                return S_OK;
            }
            DSPROPERTY_DIRECTSOUNDDEVICE_DESCRIPTION_1 => {
                *pTypeSupport = KSPROPERTY_SUPPORT_GET;
                return S_OK;
            }
            DSPROPERTY_DIRECTSOUNDDEVICE_ENUMERATE_1 => {
                *pTypeSupport = KSPROPERTY_SUPPORT_GET;
                return S_OK;
            }
            DSPROPERTY_DIRECTSOUNDDEVICE_WAVEDEVICEMAPPING_W => {
                *pTypeSupport = KSPROPERTY_SUPPORT_GET;
                return S_OK;
            }
            DSPROPERTY_DIRECTSOUNDDEVICE_DESCRIPTION_A => {
                *pTypeSupport = KSPROPERTY_SUPPORT_GET;
                return S_OK;
            }
            DSPROPERTY_DIRECTSOUNDDEVICE_DESCRIPTION_W => {
                *pTypeSupport = KSPROPERTY_SUPPORT_GET;
                return S_OK;
            }
            DSPROPERTY_DIRECTSOUNDDEVICE_ENUMERATE_A => {
                *pTypeSupport = KSPROPERTY_SUPPORT_GET;
                return S_OK;
            }
            DSPROPERTY_DIRECTSOUNDDEVICE_ENUMERATE_W => {
                *pTypeSupport = KSPROPERTY_SUPPORT_GET;
                return S_OK;
            }
            _ => {
                FIXME!("unsupported ID: %d\n", dwPropID);
                // break
            }
        }
    }

    return E_PROP_ID_UNSUPPORTED;
}
#[no_mangle]
pub unsafe extern "C" fn IKsPrivatePropertySetImpl_Create(riid: *const IID, piks: *mut *mut IKsPropertySet) -> HRESULT {
    let mut iks: *mut IKsPrivatePropertySetImpl = core::ptr::null_mut();

    if IsEqualIID!(riid as *const _, core::ptr::addr_of!(IID_IUnknown) as *const _) == 0
        && IsEqualIID!(riid as *const _, core::ptr::addr_of!(IID_IKsPropertySet) as *const _) == 0
    {
        *piks = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    iks = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<IKsPrivatePropertySetImpl>() as SIZE_T,
    ) as *mut IKsPrivatePropertySetImpl;
    (*iks).ref_ = 1;
    (*iks).IKsPropertySet_iface.lpVtbl = core::ptr::addr_of!(ikspvt) as *const IKsPropertySetVtbl;

    *piks = core::ptr::addr_of_mut!((*iks).IKsPropertySet_iface);
    return S_OK;
}

// COM vtable tables — generated from propset.c by vtbl_gen.py
/// propset.c:606 `static const IKsPropertySetVtbl ikspvt`  (slot order: rust:types_amd64.rs)
pub static ikspvt: IKsPropertySetVtbl = IKsPropertySetVtbl {
    QueryInterface: Some(IKsPrivatePropertySetImpl_QueryInterface),
    AddRef: Some(IKsPrivatePropertySetImpl_AddRef),
    Release: Some(IKsPrivatePropertySetImpl_Release),
    Get: Some(IKsPrivatePropertySetImpl_Get),
    Set: Some(IKsPrivatePropertySetImpl_Set),
    QuerySupport: Some(IKsPrivatePropertySetImpl_QuerySupport),
};
