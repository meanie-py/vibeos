use crate::mf::main::create_activation_object;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut DEVPKEY_Device_FriendlyName: DEVPROPKEY = _DEVPROPKEY {
    fmtid: _GUID {
        Data1: 0xa45c254e as ::core::ffi::c_uint,
        Data2: 0xdf1c as ::core::ffi::c_ushort,
        Data3: 0x4efd as ::core::ffi::c_ushort,
        Data4: [
            0x80 as ::core::ffi::c_int as ::core::ffi::c_uchar,
            0x20 as ::core::ffi::c_int as ::core::ffi::c_uchar,
            0x67 as ::core::ffi::c_int as ::core::ffi::c_uchar,
            0xd1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
            0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
            0xa8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
            0x50 as ::core::ffi::c_int as ::core::ffi::c_uchar,
            0xe0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        ],
    },
    pid: 14 as DEVPROPID,
};

static mut audio_capture_activate_funcs: activate_funcs = unsafe {
    activate_funcs {
        create_object: Some(
            audio_capture_create_object
                as unsafe extern "C" fn(
                    *mut IMFAttributes,
                    *mut ::core::ffi::c_void,
                    *mut *mut IUnknown,
                ) -> HRESULT,
        ),
        shutdown_object: Some(
            audio_capture_shutdown_object
                as unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut IUnknown) -> (),
        ),
        free_private: Some(
            audio_capture_free_private as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> (),
        ),
    }
};

// #define COBJMACROS: flag-only macro with no value, nothing to emit.
// WINE_DEFAULT_DEBUG_CHANNEL(mfplat): debug channel registration for this file, no Rust declaration emitted here.

unsafe extern "C" fn audio_capture_create_object(
    attributes: *mut IMFAttributes,
    user_context: *mut core::ffi::c_void,
    obj: *mut *mut IUnknown,
) -> HRESULT {
    // FIXME("%p, %p, %p stub.\n", attributes, user_context, obj);
    E_NOTIMPL
}

unsafe extern "C" fn audio_capture_shutdown_object(
    user_context: *mut core::ffi::c_void,
    obj: *mut IUnknown,
) {
    // TRACE("%p %p.\n", user_context, obj);
}

unsafe extern "C" fn audio_capture_free_private(user_context: *mut core::ffi::c_void) {
    // TRACE("%p.\n", user_context);
}
#[no_mangle]
pub unsafe extern "C" fn enum_audio_capture_sources(
    attributes: *mut IMFAttributes,
    ret_sources: *mut *mut *mut IMFActivate,
    ret_count: *mut UINT32,
) -> HRESULT {
    let mut devenum: *mut IMMDeviceEnumerator = core::ptr::null_mut();
    let mut devices: *mut IMMDeviceCollection = core::ptr::null_mut();
    let mut i: UINT32 = 0;
    let mut count: UINT32 = 0;
    let mut created_count: UINT32 = 0;
    let mut sources: *mut *mut IMFActivate = core::ptr::null_mut();
    let mut role: ERole = eConsole;
    let mut hr: HRESULT = 0;

    'done: {
        hr = CoCreateInstance(
            core::ptr::addr_of!(CLSID_MMDeviceEnumerator),
            core::ptr::null_mut(),
            CLSCTX_INPROC_SERVER,
            core::ptr::addr_of!(IID_IMMDeviceEnumerator),
            core::ptr::addr_of_mut!(devenum) as *mut LPVOID,
        );
        if hr < 0 {
            break 'done;
        }
        if IMFAttributes_GetUINT32(
            attributes,
            core::ptr::addr_of!(MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_AUDCAP_ROLE),
            core::ptr::addr_of_mut!(role) as *mut UINT32,
        ) >= 0
        {
            FIXME!("Specifying role (%d) is not supported.\n", role);
        }
        hr = IMMDeviceEnumerator_EnumAudioEndpoints(
            devenum,
            eCapture,
            DEVICE_STATE_ACTIVE,
            core::ptr::addr_of_mut!(devices),
        );
        if hr < 0 {
            break 'done;
        }
        hr = IMMDeviceCollection_GetCount(devices, core::ptr::addr_of_mut!(count));
        if hr < 0 {
            break 'done;
        }
        sources = CoTaskMemAlloc(
            ((count as usize).wrapping_mul(core::mem::size_of::<IMFActivate>())) as SIZE_T,
        ) as *mut *mut IMFActivate;
        if sources.is_null() {
            hr = E_OUTOFMEMORY;
            break 'done;
        }
        i = 0;
        while i < count {
            let mut device: *mut IMMDevice = core::ptr::null_mut();
            let mut ps: *mut IPropertyStore = core::ptr::null_mut();
            let mut str: *mut WCHAR = core::ptr::null_mut();
            let mut pv: PROPVARIANT = core::mem::zeroed();

            hr = create_activation_object(
                core::ptr::null_mut(),
                core::ptr::addr_of!(audio_capture_activate_funcs),
                sources.offset(i as isize),
            );
            if hr < 0 {
                break 'done;
            }
            created_count = created_count.wrapping_add(1);

            hr = IMFActivate_SetGUID(
                *sources.offset(i as isize),
                core::ptr::addr_of!(MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE),
                core::ptr::addr_of!(MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_AUDCAP_GUID),
            );
            if hr >= 0 {
                hr = IMMDeviceCollection_Item(devices, i, core::ptr::addr_of_mut!(device));
            }
            if hr >= 0 {
                hr = IMMDevice_GetId(device, core::ptr::addr_of_mut!(str));
            }
            if hr >= 0 {
                hr = IMFActivate_SetString(
                    *sources.offset(i as isize),
                    core::ptr::addr_of!(MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_AUDCAP_ENDPOINT_ID),
                    str as LPCWSTR,
                );
            }
            CoTaskMemFree(str as LPVOID);

            PropVariantInit!(core::ptr::addr_of_mut!(pv));
            let mut cond: bool = hr >= 0;
            if cond {
                hr = IMMDevice_OpenPropertyStore(
                    device,
                    STGM_READ,
                    core::ptr::addr_of_mut!(ps),
                );
                cond = hr >= 0;
            }
            if cond {
                hr = IPropertyStore_GetValue(
                    ps,
                    core::ptr::addr_of!(DEVPKEY_Device_FriendlyName) as *const PROPERTYKEY,
                    core::ptr::addr_of_mut!(pv),
                );
                IPropertyStore_Release(ps);
            }
            if !device.is_null() {
                IMMDevice_Release(device);
            }

            if hr >= 0 && pv.vt as ::core::ffi::c_int != (VT_LPWSTR as i32) {
                hr = E_FAIL;
            }

            if hr >= 0 {
                hr = IMFActivate_SetString(
                    *sources.offset(i as isize),
                    core::ptr::addr_of!(MF_DEVSOURCE_ATTRIBUTE_FRIENDLY_NAME),
                    pv.__bindgen_anon_1.pwszVal as LPCWSTR,
                );
            }
            PropVariantClear(core::ptr::addr_of_mut!(pv));
            if hr < 0 {
                break 'done;
            }
            i = i.wrapping_add(1);
        }
        *ret_count = count;
        *ret_sources = sources;
    }

    if hr < 0 {
        i = 0;
        while i < created_count {
            IMFActivate_Release(*sources.offset(i as isize));
            i = i.wrapping_add(1);
        }
        CoTaskMemFree(sources as LPVOID);
    }
    if !devices.is_null() {
        IMMDeviceCollection_Release(devices);
    }
    if !devenum.is_null() {
        IMMDeviceEnumerator_Release(devenum);
    }
    return hr;
}
