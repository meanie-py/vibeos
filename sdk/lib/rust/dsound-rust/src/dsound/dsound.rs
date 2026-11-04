use crate::dsound::dsound_main::DSOUND_renderer;
use crate::dsound::dsound_main::DSOUND_renderer_guids;
use crate::dsound::buffer::IDirectSoundBufferImpl_Create;
use crate::dsound::buffer::IDirectSoundBufferImpl_Destroy;
use crate::dsound::buffer::IDirectSoundBufferImpl_Duplicate;
use crate::dsound::dsound_main::ds_default_bits_per_sample;
use crate::dsound::dsound_main::ds_default_sample_rate;
use crate::dsound::dsound_main::ds_emuldriver;
use crate::dsound::dsound_main::ds_snd_queue_max;
use crate::prelude::*;
use core::ffi::c_void;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_30 {
    pub mask: DWORD,
    pub name: *const ::core::ffi::c_char,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_31 {
    pub mask: DWORD,
    pub name: *const ::core::ffi::c_char,
}

// ---- file-scope preamble (translated) ----





#[no_mangle]
pub unsafe extern "C" fn dumpCooperativeLevel(mut level: DWORD) -> *const ::core::ffi::c_char {
    match level {
        DSSCL_NORMAL => return concat!(stringify!(DSSCL_NORMAL), "\0").as_ptr() as *const ::core::ffi::c_char,
        DSSCL_PRIORITY => return concat!(stringify!(DSSCL_PRIORITY), "\0").as_ptr() as *const ::core::ffi::c_char,
        DSSCL_EXCLUSIVE => return concat!(stringify!(DSSCL_EXCLUSIVE), "\0").as_ptr() as *const ::core::ffi::c_char,
        DSSCL_WRITEPRIMARY => return concat!(stringify!(DSSCL_WRITEPRIMARY), "\0").as_ptr() as *const ::core::ffi::c_char,
        _ => {}
    }
    return wine_dbg_sprintf(
        b"Unknown(%08x)\0" as *const u8 as *const ::core::ffi::c_char,
        level,
    );
}

unsafe extern "C" fn _dump_DSCAPS(mut xmask: DWORD) {
    let mut flags: [C2RustUnnamed_30; 11] = [
        C2RustUnnamed_30 {
            mask: 0x1 as DWORD,
            name: b"DSCAPS_PRIMARYMONO\0" as *const u8 as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_30 {
            mask: 0x2 as DWORD,
            name: b"DSCAPS_PRIMARYSTEREO\0" as *const u8 as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_30 {
            mask: 0x4 as DWORD,
            name: b"DSCAPS_PRIMARY8BIT\0" as *const u8 as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_30 {
            mask: 0x8 as DWORD,
            name: b"DSCAPS_PRIMARY16BIT\0" as *const u8 as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_30 {
            mask: 0x10 as DWORD,
            name: b"DSCAPS_CONTINUOUSRATE\0" as *const u8 as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_30 {
            mask: 0x20 as DWORD,
            name: b"DSCAPS_EMULDRIVER\0" as *const u8 as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_30 {
            mask: 0x40 as DWORD,
            name: b"DSCAPS_CERTIFIED\0" as *const u8 as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_30 {
            mask: 0x100 as DWORD,
            name: b"DSCAPS_SECONDARYMONO\0" as *const u8 as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_30 {
            mask: 0x200 as DWORD,
            name: b"DSCAPS_SECONDARYSTEREO\0" as *const u8 as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_30 {
            mask: 0x400 as DWORD,
            name: b"DSCAPS_SECONDARY8BIT\0" as *const u8 as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_30 {
            mask: 0x800 as DWORD,
            name: b"DSCAPS_SECONDARY16BIT\0" as *const u8 as *const ::core::ffi::c_char,
        },
    ];
    let mut i: ::core::ffi::c_uint = 0;
    i = 0 as ::core::ffi::c_uint;
    while (i as usize)
        < (::core::mem::size_of::<[C2RustUnnamed_30; 11]>() as usize)
            .wrapping_div(
                ::core::mem::size_of_val(&flags[0 as usize]) as usize,
            )
    {
        if (flags[i as usize].mask & xmask) == flags[i as usize].mask {
            WINE_TRACE!("%s ", flags[i as usize].name);
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn _dump_DSBCAPS(mut xmask: DWORD) {
    let mut flags: [C2RustUnnamed_31; 13] = [
        C2RustUnnamed_31 {
            mask: 0x1 as DWORD, // DSBCAPS_PRIMARYBUFFER
            name: b"DSBCAPS_PRIMARYBUFFER\0" as *const u8 as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_31 {
            mask: 0x2 as DWORD, // DSBCAPS_STATIC
            name: b"DSBCAPS_STATIC\0" as *const u8 as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_31 {
            mask: 0x4 as DWORD, // DSBCAPS_LOCHARDWARE
            name: b"DSBCAPS_LOCHARDWARE\0" as *const u8 as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_31 {
            mask: 0x8 as DWORD, // DSBCAPS_LOCSOFTWARE
            name: b"DSBCAPS_LOCSOFTWARE\0" as *const u8 as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_31 {
            mask: 0x10 as DWORD, // DSBCAPS_CTRL3D
            name: b"DSBCAPS_CTRL3D\0" as *const u8 as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_31 {
            mask: 0x20 as DWORD, // DSBCAPS_CTRLFREQUENCY
            name: b"DSBCAPS_CTRLFREQUENCY\0" as *const u8 as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_31 {
            mask: 0x40 as DWORD, // DSBCAPS_CTRLPAN
            name: b"DSBCAPS_CTRLPAN\0" as *const u8 as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_31 {
            mask: 0x80 as DWORD, // DSBCAPS_CTRLVOLUME
            name: b"DSBCAPS_CTRLVOLUME\0" as *const u8 as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_31 {
            mask: 0x100 as DWORD, // DSBCAPS_CTRLPOSITIONNOTIFY
            name: b"DSBCAPS_CTRLPOSITIONNOTIFY\0" as *const u8 as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_31 {
            mask: 0x4000 as DWORD, // DSBCAPS_STICKYFOCUS
            name: b"DSBCAPS_STICKYFOCUS\0" as *const u8 as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_31 {
            mask: 0x8000 as DWORD, // DSBCAPS_GLOBALFOCUS
            name: b"DSBCAPS_GLOBALFOCUS\0" as *const u8 as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_31 {
            mask: 0x10000 as DWORD, // DSBCAPS_GETCURRENTPOSITION2
            name: b"DSBCAPS_GETCURRENTPOSITION2\0" as *const u8 as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_31 {
            mask: 0x20000 as DWORD, // DSBCAPS_MUTE3DATMAXDISTANCE
            name: b"DSBCAPS_MUTE3DATMAXDISTANCE\0" as *const u8 as *const ::core::ffi::c_char,
        },
    ];
    let mut i: ::core::ffi::c_uint = 0;
    i = 0 as ::core::ffi::c_uint;
    while (i as usize)
        < (::core::mem::size_of_val(&flags) as usize)
            .wrapping_div(::core::mem::size_of_val(&flags[0]) as usize)
    {
        if (flags[i as usize].mask & xmask) == flags[i as usize].mask {
            TRACE!("%s ", flags[i as usize].name);
        }
        i += 1;
    }
}

unsafe extern "C" fn DSOUND_QueryInterface(
    iface: LPDIRECTSOUND8,
    riid: REFIID,
    ppobj: *mut LPVOID,
) -> HRESULT {
    let This: *mut IDirectSoundImpl = iface as *mut IDirectSoundImpl;

    if ppobj.is_null() {
        WARN!("invalid parameter\n");
        return E_INVALIDARG;
    }

    if IsEqualIID!(riid, &raw const IID_IUnknown) != 0 {
        if (*This).pUnknown.is_null() {
            IDirectSound_IUnknown_Create(iface, &raw mut (*This).pUnknown);
            if (*This).pUnknown.is_null() {
                WARN!("IDirectSound_IUnknown_Create() failed\n");
                *ppobj = core::ptr::null_mut();
                return E_NOINTERFACE;
            }
        }
        IDirectSound_IUnknown_AddRef((*This).pUnknown);
        *ppobj = (*This).pUnknown as LPVOID;
        return S_OK;
    } else if IsEqualIID!(riid, &raw const IID_IDirectSound) != 0 {
        if (*This).pDS.is_null() {
            IDirectSound_IDirectSound_Create(iface, &raw mut (*This).pDS);
            if (*This).pDS.is_null() {
                WARN!("IDirectSound_IDirectSound_Create() failed\n");
                *ppobj = core::ptr::null_mut();
                return E_NOINTERFACE;
            }
        }
        IDirectSound_IDirectSound_AddRef((*This).pDS);
        *ppobj = (*This).pDS as LPVOID;
        return S_OK;
    }

    *ppobj = core::ptr::null_mut();
    WARN!("Unknown IID %s\n", debugstr_guid(riid));
    return E_NOINTERFACE;
}

unsafe extern "C" fn DSOUND_QueryInterface8(
    iface: LPDIRECTSOUND8,
    riid: REFIID,
    ppobj: *mut LPVOID,
) -> HRESULT {
    let This: *mut IDirectSoundImpl = iface as *mut IDirectSoundImpl;

    if ppobj.is_null() {
        WARN!("invalid parameter\n");
        return E_INVALIDARG;
    }

    if IsEqualIID!(riid, &raw const IID_IUnknown) != 0 {
        if (*This).pUnknown.is_null() {
            IDirectSound8_IUnknown_Create(iface, &raw mut (*This).pUnknown);
            if (*This).pUnknown.is_null() {
                WARN!("IDirectSound8_IUnknown_Create() failed\n");
                *ppobj = core::ptr::null_mut();
                return E_NOINTERFACE;
            }
        }
        IDirectSound8_IUnknown_AddRef((*This).pUnknown);
        *ppobj = (*This).pUnknown as LPVOID;
        return S_OK;
    } else if IsEqualIID!(riid, &raw const IID_IDirectSound) != 0 {
        if (*This).pDS.is_null() {
            IDirectSound8_IDirectSound_Create(iface, &raw mut (*This).pDS);
            if (*This).pDS.is_null() {
                WARN!("IDirectSound8_IDirectSound_Create() failed\n");
                *ppobj = core::ptr::null_mut();
                return E_NOINTERFACE;
            }
        }
        IDirectSound8_IDirectSound_AddRef((*This).pDS);
        *ppobj = (*This).pDS as LPVOID;
        return S_OK;
    } else if IsEqualIID!(riid, &raw const IID_IDirectSound8) != 0 {
        if (*This).pDS8.is_null() {
            IDirectSound8_IDirectSound8_Create(iface, &raw mut (*This).pDS8);
            if (*This).pDS8.is_null() {
                WARN!("IDirectSound8_IDirectSound8_Create() failed\n");
                *ppobj = core::ptr::null_mut();
                return E_NOINTERFACE;
            }
        }
        IDirectSound8_IDirectSound8_AddRef((*This).pDS8);
        *ppobj = (*This).pDS8 as LPVOID;
        return S_OK;
    }

    *ppobj = core::ptr::null_mut();
    WARN!("Unknown IID %s\n", debugstr_guid(riid));
    return E_NOINTERFACE;
}

unsafe extern "C" fn IDirectSoundImpl_AddRef(iface: LPDIRECTSOUND8) -> ULONG {
    let This: *mut IDirectSoundImpl = iface as *mut IDirectSoundImpl;
    let ref_0: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_) as *mut LONG) as ULONG;
    return ref_0;
}

unsafe extern "C" fn IDirectSoundImpl_Release(mut iface: LPDIRECTSOUND8) -> ULONG {
    let mut This: *mut IDirectSoundImpl = iface as *mut IDirectSoundImpl;
    let mut ref_0: ULONG =
        InterlockedDecrement(&raw mut (*This).ref_ as *mut ::core::ffi::c_long) as ULONG;
    WINE_TRACE!("(%p) ref was %d\n", This, ref_0.wrapping_add(1 as ULONG));
    if ref_0 == 0 {
        if !(*This).device.is_null() {
            DirectSoundDevice_Release((*This).device);
        }
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            This as LPVOID,
        );
        WINE_TRACE!("(%p) released\n", This);
    }
    return ref_0;
}

unsafe extern "C" fn IDirectSoundImpl_Create(ppDS: *mut LPDIRECTSOUND8) -> HRESULT {
    let mut pDS: *mut IDirectSoundImpl = core::ptr::null_mut::<IDirectSoundImpl>();

    pDS = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<IDirectSoundImpl>() as SIZE_T,
    ) as *mut IDirectSoundImpl;
    if pDS.is_null() {
        WARN!("out of memory\n");
        *ppDS = core::ptr::null_mut();
        return DSERR_OUTOFMEMORY;
    }

    (*pDS).ref_ = 0;
    (*pDS).device = core::ptr::null_mut();

    *ppDS = pDS as LPDIRECTSOUND8;

    return (DS_OK) as HRESULT;
}
unsafe extern "system" fn IDirectSound_IUnknown_QueryInterface(
    iface: LPUNKNOWN,
    riid: REFIID,
    ppobj: *mut LPVOID,
) -> HRESULT {
    let This: *mut IDirectSound_IUnknown = iface as *mut IDirectSound_IUnknown;
    return DSOUND_QueryInterface((*This).pds, riid, ppobj);
}
unsafe extern "system" fn IDirectSound_IUnknown_AddRef(iface: LPUNKNOWN) -> ULONG {
    let This: *mut IDirectSound_IUnknown = iface as *mut IDirectSound_IUnknown;
    let ref_: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_) as *mut LONG) as ULONG;
    return ref_;
}

unsafe extern "system" fn IDirectSound_IUnknown_Release(iface: LPUNKNOWN) -> ULONG {
    let This: *mut IDirectSound_IUnknown = iface as *mut IDirectSound_IUnknown;
    let ref_0: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_) as *mut LONG) as ULONG;
    if ref_0 == 0 {
        (*((*This).pds as *mut IDirectSoundImpl)).pUnknown = core::ptr::null_mut();
        IDirectSoundImpl_Release((*This).pds);
        HeapFree(GetProcessHeap(), 0, This as PVOID);
    }
    return ref_0;
}

unsafe extern "C" fn IDirectSound_IUnknown_Create(
    pds: LPDIRECTSOUND8,
    ppunk: *mut LPUNKNOWN,
) -> HRESULT {
    let mut pdsunk: *mut IDirectSound_IUnknown = core::ptr::null_mut();

    if ppunk.is_null() {
        ERR!("invalid parameter: ppunk == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    if pds.is_null() {
        ERR!("invalid parameter: pds == NULL\n");
        *ppunk = core::ptr::null_mut();
        return DSERR_INVALIDPARAM;
    }

    pdsunk = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<IDirectSound_IUnknown>() as SIZE_T,
    ) as *mut IDirectSound_IUnknown;
    if pdsunk.is_null() {
        WARN!("out of memory\n");
        *ppunk = core::ptr::null_mut();
        return DSERR_OUTOFMEMORY;
    }

    (*pdsunk).lpVtbl = core::ptr::addr_of!(DirectSound_Unknown_Vtbl);
    (*pdsunk).ref_ = 0;
    (*pdsunk).pds = pds;

    IDirectSoundImpl_AddRef(pds);
    *ppunk = pdsunk as LPUNKNOWN;

    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn IDirectSound_IDirectSound_QueryInterface(
    mut iface: LPDIRECTSOUND,
    riid: REFIID,
    mut ppobj: *mut LPVOID,
) -> HRESULT {
    let mut This: *mut IDirectSound_IDirectSound = iface as *mut IDirectSound_IDirectSound;
    WINE_TRACE!("(%p,%s,%p)\n", This, debugstr_guid(riid as *const _GUID), ppobj);
    return DSOUND_QueryInterface((*This).pds, riid, ppobj);
}
unsafe extern "system" fn IDirectSound_IDirectSound_AddRef(iface: LPDIRECTSOUND) -> ULONG {
    let This: *mut IDirectSound_IDirectSound = iface as *mut IDirectSound_IDirectSound;
    let ref_0: ULONG =
        InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_) as *mut core::ffi::c_long)
            as ULONG;
    TRACE!("(%p) ref was %d\n", This, ref_0.wrapping_sub(1 as ULONG));
    return ref_0;
}

unsafe extern "system" fn IDirectSound_IDirectSound_Release(iface: LPDIRECTSOUND) -> ULONG {
    let This: *mut IDirectSound_IDirectSound = iface as *mut IDirectSound_IDirectSound;
    let ref_0: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_) as *mut LONG) as ULONG;
    if ref_0 == 0 {
        (*((*This).pds as *mut IDirectSoundImpl)).pDS = core::ptr::null_mut();
        IDirectSoundImpl_Release((*This).pds);
        HeapFree(GetProcessHeap(), 0, This as LPVOID);
    }
    return ref_0;
}

unsafe extern "system" fn IDirectSound_IDirectSound_CreateSoundBuffer(
    iface: LPDIRECTSOUND,
    dsbd: LPCDSBUFFERDESC,
    ppdsb: LPLPDIRECTSOUNDBUFFER,
    lpunk: LPUNKNOWN,
) -> HRESULT {
    let This: *mut IDirectSound_IDirectSound = iface as *mut IDirectSound_IDirectSound;
    return DirectSoundDevice_CreateSoundBuffer(
        (*((*This).pds as *mut IDirectSoundImpl)).device,
        dsbd,
        ppdsb,
        lpunk,
        (FALSE) as BOOL,
    );
}

unsafe extern "system" fn IDirectSound_IDirectSound_GetCaps(
    iface: LPDIRECTSOUND,
    lpDSCaps: LPDSCAPS,
) -> HRESULT {
    let This: *mut IDirectSound_IDirectSound = iface as *mut IDirectSound_IDirectSound;
    return DirectSoundDevice_GetCaps((*((*This).pds as *mut IDirectSoundImpl)).device, lpDSCaps);
}

unsafe extern "system" fn IDirectSound_IDirectSound_DuplicateSoundBuffer(
    iface: LPDIRECTSOUND,
    psb: LPDIRECTSOUNDBUFFER,
    ppdsb: LPLPDIRECTSOUNDBUFFER,
) -> HRESULT {
    let This: *mut IDirectSound_IDirectSound = iface as *mut IDirectSound_IDirectSound;
    return DirectSoundDevice_DuplicateSoundBuffer(
        (*((*This).pds as *mut IDirectSoundImpl)).device,
        psb,
        ppdsb,
    );
}

unsafe extern "system" fn IDirectSound_IDirectSound_SetCooperativeLevel(
    iface: LPDIRECTSOUND,
    hwnd: HWND,
    level: DWORD,
) -> HRESULT {
    let This: *mut IDirectSound_IDirectSound = iface as *mut IDirectSound_IDirectSound;
    WINE_TRACE!("(%p,%p,%s)\n", This, hwnd, dumpCooperativeLevel(level));
    return DirectSoundDevice_SetCooperativeLevel(
        (*((*This).pds as *mut IDirectSoundImpl)).device,
        hwnd,
        level,
    );
}

unsafe extern "system" fn IDirectSound_IDirectSound_Compact(iface: LPDIRECTSOUND) -> HRESULT {
    let mut This: *mut IDirectSound_IDirectSound = iface as *mut IDirectSound_IDirectSound;
    return DirectSoundDevice_Compact((*((*This).pds as *mut IDirectSoundImpl)).device);
}
unsafe extern "system" fn IDirectSound_IDirectSound_GetSpeakerConfig(
    iface: LPDIRECTSOUND,
    lpdwSpeakerConfig: LPDWORD,
) -> HRESULT {
    let This: *mut IDirectSound_IDirectSound = iface as *mut IDirectSound_IDirectSound;
    return DirectSoundDevice_GetSpeakerConfig(
        (*((*This).pds as *mut IDirectSoundImpl)).device,
        lpdwSpeakerConfig,
    );
}

unsafe extern "system" fn IDirectSound_IDirectSound_SetSpeakerConfig(
    iface: LPDIRECTSOUND,
    config: DWORD,
) -> HRESULT {
    let This: *mut IDirectSound_IDirectSound = iface as *mut IDirectSound_IDirectSound;
    return DirectSoundDevice_SetSpeakerConfig((*((*This).pds as *mut IDirectSoundImpl)).device, config);
}

unsafe extern "system" fn IDirectSound_IDirectSound_Initialize(
    iface: LPDIRECTSOUND,
    lpcGuid: LPCGUID,
) -> HRESULT {
    let This: *mut IDirectSound_IDirectSound = iface as *mut IDirectSound_IDirectSound;
    return DirectSoundDevice_Initialize(
        &raw mut (*((*This).pds as *mut IDirectSoundImpl)).device as *mut *mut DirectSoundDevice,
        lpcGuid,
    );
}

unsafe extern "C" fn IDirectSound_IDirectSound_Create(
    mut pds: LPDIRECTSOUND8,
    mut ppds: *mut LPDIRECTSOUND,
) -> HRESULT {
    let mut pdsds: *mut IDirectSound_IDirectSound = core::ptr::null_mut();

    if ppds.is_null() {
        ERR!("invalid parameter: ppds == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    if pds.is_null() {
        ERR!("invalid parameter: pds == NULL\n");
        *ppds = core::ptr::null_mut();
        return DSERR_INVALIDPARAM;
    }

    pdsds = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<IDirectSound_IDirectSound>() as SIZE_T,
    ) as *mut IDirectSound_IDirectSound;
    if pdsds.is_null() {
        WARN!("out of memory\n");
        *ppds = core::ptr::null_mut();
        return DSERR_OUTOFMEMORY;
    }

    (*pdsds).lpVtbl = core::ptr::addr_of!(DirectSound_DirectSound_Vtbl);
    (*pdsds).ref_ = 0;
    (*pdsds).pds = pds;

    IDirectSoundImpl_AddRef(pds);
    *ppds = pdsds as LPDIRECTSOUND;

    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn IDirectSound8_IUnknown_QueryInterface(
    iface: LPUNKNOWN,
    riid: REFIID,
    ppobj: *mut LPVOID,
) -> HRESULT {
    let mut This: *mut IDirectSound_IUnknown = iface as *mut IDirectSound_IUnknown;
    return DSOUND_QueryInterface8((*This).pds, riid, ppobj);
}

unsafe extern "system" fn IDirectSound8_IUnknown_AddRef(iface: LPUNKNOWN) -> ULONG {
    let This: *mut IDirectSound_IUnknown = iface as *mut IDirectSound_IUnknown;
    let ref_0: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_) as *mut LONG) as ULONG;
    WINE_TRACE!("(%p) ref was %d\n", This, ref_0.wrapping_sub(1 as ULONG));
    return ref_0;
}

unsafe extern "system" fn IDirectSound8_IUnknown_Release(iface: LPUNKNOWN) -> ULONG {
    let This: *mut IDirectSound_IUnknown = iface as *mut IDirectSound_IUnknown;
    let ref_0: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_) as *mut LONG) as ULONG;
    if ref_0 == 0 {
        (*((*This).pds as *mut IDirectSoundImpl)).pUnknown = core::ptr::null_mut();
        IDirectSoundImpl_Release((*This).pds);
        HeapFree(GetProcessHeap(), 0 as DWORD, This as PVOID);
    }
    return ref_0;
}

unsafe extern "C" fn IDirectSound8_IUnknown_Create(
    pds: LPDIRECTSOUND8,
    ppunk: *mut LPUNKNOWN,
) -> HRESULT {
    let mut pdsunk: *mut IDirectSound8_IUnknown = core::ptr::null_mut();

    if ppunk.is_null() {
        ERR!("invalid parameter: ppunk == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    if pds.is_null() {
        ERR!("invalid parameter: pds == NULL\n");
        *ppunk = core::ptr::null_mut();
        return DSERR_INVALIDPARAM;
    }

    pdsunk = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        core::mem::size_of::<IDirectSound8_IUnknown>() as SIZE_T,
    ) as *mut IDirectSound8_IUnknown;
    if pdsunk.is_null() {
        WARN!("out of memory\n");
        *ppunk = core::ptr::null_mut();
        return DSERR_OUTOFMEMORY;
    }

    (*pdsunk).lpVtbl = core::ptr::addr_of!(DirectSound8_Unknown_Vtbl);
    (*pdsunk).ref_ = 0 as LONG;
    (*pdsunk).pds = pds;

    IDirectSoundImpl_AddRef(pds);
    *ppunk = pdsunk as LPUNKNOWN;

    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn IDirectSound8_IDirectSound_QueryInterface(
    iface: LPDIRECTSOUND,
    riid: REFIID,
    ppobj: *mut LPVOID,
) -> HRESULT {
    let mut This: *mut IDirectSound8_IDirectSound = iface as *mut IDirectSound8_IDirectSound;
    WINE_TRACE!("(%p,%s,%p)\n", This, debugstr_guid(riid as *const _GUID), ppobj);
    return DSOUND_QueryInterface8((*This).pds, riid, ppobj);
}

unsafe extern "system" fn IDirectSound8_IDirectSound_AddRef(mut iface: LPDIRECTSOUND) -> ULONG {
    let mut This: *mut IDirectSound8_IDirectSound = iface as *mut IDirectSound8_IDirectSound;
    let mut ref_0: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_) as *mut LONG) as ULONG;
    return ref_0;
}
unsafe extern "system" fn IDirectSound8_IDirectSound_Release(mut iface: LPDIRECTSOUND) -> ULONG {
    let mut This: *mut IDirectSound8_IDirectSound = iface as *mut IDirectSound8_IDirectSound;
    let mut ref_0: ULONG =
        InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_) as *mut ::core::ffi::c_long)
            as ULONG;
    WINE_TRACE!("(%p) ref was %d\n", This, ref_0.wrapping_add(1 as ULONG));
    if ref_0 == 0 {
        let ref mut fresh18 = (*((*This).pds as *mut IDirectSoundImpl)).pDS;
        *fresh18 = ::core::ptr::null_mut::<IDirectSound>();
        IDirectSoundImpl_Release((*This).pds);
        HeapFree(
            GetProcessHeap(),
            0,
            This as PVOID,
        );
        WINE_TRACE!("(%p) released\n", This);
    }
    return ref_0;
}

unsafe extern "system" fn IDirectSound8_IDirectSound_CreateSoundBuffer(
    iface: LPDIRECTSOUND,
    dsbd: LPCDSBUFFERDESC,
    ppdsb: LPLPDIRECTSOUNDBUFFER,
    lpunk: LPUNKNOWN,
) -> HRESULT {
    let This: *mut IDirectSound8_IDirectSound = iface as *mut IDirectSound8_IDirectSound;
    TRACE!("(%p,%p,%p,%p)\n", This, dsbd, ppdsb, lpunk);
    return DirectSoundDevice_CreateSoundBuffer(
        (*((*This).pds as *mut IDirectSoundImpl)).device,
        dsbd,
        ppdsb,
        lpunk,
        (TRUE) as BOOL,
    );
}
unsafe extern "system" fn IDirectSound8_IDirectSound_GetCaps(
    iface: LPDIRECTSOUND,
    lpDSCaps: LPDSCAPS,
) -> HRESULT {
    let mut This: *mut IDirectSound8_IDirectSound = iface as *mut IDirectSound8_IDirectSound;
    return DirectSoundDevice_GetCaps((*((*This).pds as *mut IDirectSoundImpl)).device, lpDSCaps);
}

unsafe extern "system" fn IDirectSound8_IDirectSound_DuplicateSoundBuffer(
    iface: LPDIRECTSOUND,
    psb: LPDIRECTSOUNDBUFFER,
    ppdsb: LPLPDIRECTSOUNDBUFFER,
) -> HRESULT {
    let mut This: *mut IDirectSound8_IDirectSound = iface as *mut IDirectSound8_IDirectSound;
    return DirectSoundDevice_DuplicateSoundBuffer(
        (*((*This).pds as *mut IDirectSoundImpl)).device,
        psb,
        ppdsb,
    );
}

unsafe extern "system" fn IDirectSound8_IDirectSound_SetCooperativeLevel(
    mut iface: LPDIRECTSOUND,
    mut hwnd: HWND,
    mut level: DWORD,
) -> HRESULT {
    let mut This: *mut IDirectSound8_IDirectSound = iface as *mut IDirectSound8_IDirectSound;
    WINE_TRACE!("(%p,%p,%s)\n", This, hwnd, dumpCooperativeLevel(level));
    return DirectSoundDevice_SetCooperativeLevel(
        (*((*This).pds as *mut IDirectSoundImpl)).device,
        hwnd,
        level,
    );
}

unsafe extern "system" fn IDirectSound8_IDirectSound_Compact(iface: LPDIRECTSOUND) -> HRESULT {
    let This: *mut IDirectSound8_IDirectSound = iface as *mut IDirectSound8_IDirectSound;
    WINE_TRACE!("(%p)\n", This);
    return DirectSoundDevice_Compact((*((*This).pds as *mut IDirectSoundImpl)).device);
}

unsafe extern "system" fn IDirectSound8_IDirectSound_GetSpeakerConfig(
    iface: LPDIRECTSOUND,
    lpdwSpeakerConfig: LPDWORD,
) -> HRESULT {
    let This: *mut IDirectSound8_IDirectSound = iface as *mut IDirectSound8_IDirectSound;
    return DirectSoundDevice_GetSpeakerConfig((*((*This).pds as *mut IDirectSoundImpl)).device, lpdwSpeakerConfig);
}

unsafe extern "system" fn IDirectSound8_IDirectSound_SetSpeakerConfig(
    iface: LPDIRECTSOUND,
    config: DWORD,
) -> HRESULT {
    let mut This: *mut IDirectSound8_IDirectSound = iface as *mut IDirectSound8_IDirectSound;
    WINE_TRACE!("(%p,0x%08x)\n", This, config);
    return DirectSoundDevice_SetSpeakerConfig(
        (*((*This).pds as *mut IDirectSoundImpl)).device,
        config,
    );
}

unsafe extern "system" fn IDirectSound8_IDirectSound_Initialize(
    iface: LPDIRECTSOUND,
    lpcGuid: LPCGUID,
) -> HRESULT {
    let mut This: *mut IDirectSound8_IDirectSound = iface as *mut IDirectSound8_IDirectSound;
    return DirectSoundDevice_Initialize(
        &raw mut (*((*This).pds as *mut IDirectSoundImpl)).device,
        lpcGuid,
    );
}

unsafe extern "C" fn IDirectSound8_IDirectSound_Create(
    pds: LPDIRECTSOUND8,
    ppds: *mut LPDIRECTSOUND,
) -> HRESULT {
    let mut pdsds: *mut IDirectSound8_IDirectSound = core::ptr::null_mut();

    if ppds.is_null() {
        ERR!("invalid parameter: ppds == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    if pds.is_null() {
        ERR!("invalid parameter: pds == NULL\n");
        *ppds = core::ptr::null_mut();
        return DSERR_INVALIDPARAM;
    }

    pdsds = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        core::mem::size_of::<IDirectSound8_IDirectSound>() as SIZE_T,
    ) as *mut IDirectSound8_IDirectSound;
    if pdsds.is_null() {
        WARN!("out of memory\n");
        *ppds = core::ptr::null_mut();
        return DSERR_OUTOFMEMORY;
    }

    (*pdsds).lpVtbl = core::ptr::addr_of!(DirectSound8_DirectSound_Vtbl);
    (*pdsds).ref_ = 0 as LONG;
    (*pdsds).pds = pds;

    IDirectSoundImpl_AddRef(pds);
    *ppds = pdsds as LPDIRECTSOUND;

    return (DS_OK) as HRESULT;
}

unsafe extern "system" fn IDirectSound8_IDirectSound8_QueryInterface(
    iface: LPDIRECTSOUND8,
    riid: REFIID,
    ppobj: *mut LPVOID,
) -> HRESULT {
    let mut This: *mut IDirectSound8_IDirectSound8 = iface as *mut IDirectSound8_IDirectSound8;
    return DSOUND_QueryInterface8((*This).pds, riid, ppobj);
}
unsafe extern "system" fn IDirectSound8_IDirectSound8_AddRef(mut iface: LPDIRECTSOUND8) -> ULONG {
    let mut This: *mut IDirectSound8_IDirectSound8 = iface as *mut IDirectSound8_IDirectSound8;
    let mut ref_0: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_) as *mut LONG) as ULONG;
    TRACE!("(%p) ref was %d\n", This, ref_0.wrapping_sub(1 as ULONG));
    return ref_0;
}

unsafe extern "system" fn IDirectSound8_IDirectSound8_Release(iface: LPDIRECTSOUND8) -> ULONG {
    let This: *mut IDirectSound8_IDirectSound8 = iface as *mut IDirectSound8_IDirectSound8;
    let ref_0: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_) as *mut LONG) as ULONG;
    if ref_0 == 0 {
        (*((*This).pds as *mut IDirectSoundImpl)).pDS8 = core::ptr::null_mut();
        IDirectSoundImpl_Release((*This).pds);
        HeapFree(GetProcessHeap(), 0, This as LPVOID);
    }
    return ref_0;
}

unsafe extern "system" fn IDirectSound8_IDirectSound8_CreateSoundBuffer(
    iface: LPDIRECTSOUND8,
    dsbd: LPCDSBUFFERDESC,
    ppdsb: LPLPDIRECTSOUNDBUFFER,
    lpunk: LPUNKNOWN,
) -> HRESULT {
    let This: *mut IDirectSound8_IDirectSound8 = iface as *mut IDirectSound8_IDirectSound8;
    return DirectSoundDevice_CreateSoundBuffer(
        (*((*This).pds as *mut IDirectSoundImpl)).device,
        dsbd,
        ppdsb,
        lpunk,
        (TRUE) as BOOL,
    );
}

unsafe extern "system" fn IDirectSound8_IDirectSound8_GetCaps(
    iface: LPDIRECTSOUND8,
    lpDSCaps: LPDSCAPS,
) -> HRESULT {
    let mut This: *mut IDirectSound8_IDirectSound = iface as *mut IDirectSound8_IDirectSound;
    WINE_TRACE!("(%p,%p)\n", This, lpDSCaps);
    return DirectSoundDevice_GetCaps((*((*This).pds as *mut IDirectSoundImpl)).device, lpDSCaps);
}

unsafe extern "system" fn IDirectSound8_IDirectSound8_DuplicateSoundBuffer(
    iface: LPDIRECTSOUND8,
    psb: LPDIRECTSOUNDBUFFER,
    ppdsb: LPLPDIRECTSOUNDBUFFER,
) -> HRESULT {
    let This: *mut IDirectSound8_IDirectSound8 = iface as *mut IDirectSound8_IDirectSound8;
    return DirectSoundDevice_DuplicateSoundBuffer(
        (*((*This).pds as *mut IDirectSoundImpl)).device,
        psb,
        ppdsb,
    );
}

unsafe extern "system" fn IDirectSound8_IDirectSound8_SetCooperativeLevel(
    mut iface: LPDIRECTSOUND8,
    mut hwnd: HWND,
    mut level: DWORD,
) -> HRESULT {
    let mut This: *mut IDirectSound8_IDirectSound8 = iface as *mut IDirectSound8_IDirectSound8;
    WINE_TRACE!("(%p,%p,%s)\n", This, hwnd, dumpCooperativeLevel(level));
    return DirectSoundDevice_SetCooperativeLevel(
        (*((*This).pds as *mut IDirectSoundImpl)).device,
        hwnd,
        level,
    );
}

unsafe extern "system" fn IDirectSound8_IDirectSound8_Compact(iface: LPDIRECTSOUND8) -> HRESULT {
    let This: *mut IDirectSound8_IDirectSound8 = iface as *mut IDirectSound8_IDirectSound8;
    return DirectSoundDevice_Compact((*((*This).pds as *mut IDirectSoundImpl)).device);
}

unsafe extern "system" fn IDirectSound8_IDirectSound8_GetSpeakerConfig(
    iface: LPDIRECTSOUND8,
    lpdwSpeakerConfig: LPDWORD,
) -> HRESULT {
    let mut This: *mut IDirectSound8_IDirectSound8 = iface as *mut IDirectSound8_IDirectSound8;
    WINE_TRACE!("(%p, %p)\n", This, lpdwSpeakerConfig);
    return DirectSoundDevice_GetSpeakerConfig(
        (*((*This).pds as *mut IDirectSoundImpl)).device,
        lpdwSpeakerConfig,
    );
}

unsafe extern "system" fn IDirectSound8_IDirectSound8_SetSpeakerConfig(
    iface: LPDIRECTSOUND8,
    config: DWORD,
) -> HRESULT {
    let mut This: *mut IDirectSound8_IDirectSound8 = iface as *mut IDirectSound8_IDirectSound8;
    return DirectSoundDevice_SetSpeakerConfig((*((*This).pds as *mut IDirectSoundImpl)).device, config);
}
unsafe extern "system" fn IDirectSound8_IDirectSound8_Initialize(
    mut iface: LPDIRECTSOUND8,
    mut lpcGuid: LPCGUID,
) -> HRESULT {
    let mut This: *mut IDirectSound8_IDirectSound8 = iface as *mut IDirectSound8_IDirectSound8;
    WINE_TRACE!("(%p, %s)\n", This, debugstr_guid(lpcGuid as *const _GUID));
    return DirectSoundDevice_Initialize(
        core::ptr::addr_of_mut!((*((*This).pds as *mut IDirectSoundImpl)).device),
        lpcGuid,
    );
}
unsafe extern "system" fn IDirectSound8_IDirectSound8_VerifyCertification(
    iface: LPDIRECTSOUND8,
    pdwCertified: LPDWORD,
) -> HRESULT {
    let mut This: *mut IDirectSound8_IDirectSound8 = iface as *mut IDirectSound8_IDirectSound8;
    return DirectSoundDevice_VerifyCertification((*((*This).pds as *mut IDirectSoundImpl)).device, pdwCertified);
}

unsafe extern "C" fn IDirectSound8_IDirectSound8_Create(
    pds: LPDIRECTSOUND8,
    ppds: *mut LPDIRECTSOUND8,
) -> HRESULT {
    let mut pdsds: *mut IDirectSound8_IDirectSound8;

    if ppds.is_null() {
        ERR!("invalid parameter: ppds == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    if pds.is_null() {
        ERR!("invalid parameter: pds == NULL\n");
        *ppds = core::ptr::null_mut();
        return DSERR_INVALIDPARAM;
    }

    pdsds = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        core::mem::size_of::<IDirectSound8_IDirectSound8>() as SIZE_T,
    ) as *mut IDirectSound8_IDirectSound8;
    if pdsds.is_null() {
        WARN!("out of memory\n");
        *ppds = core::ptr::null_mut();
        return DSERR_OUTOFMEMORY;
    }

    (*pdsds).lpVtbl = core::ptr::addr_of!(DirectSound8_DirectSound8_Vtbl);
    (*pdsds).ref_ = 0 as LONG;
    (*pdsds).pds = pds;

    IDirectSoundImpl_AddRef(pds);
    *ppds = pdsds as LPDIRECTSOUND8;

    return (DS_OK) as HRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn DSOUND_Create(riid: *const IID, ppDS: *mut LPDIRECTSOUND) -> HRESULT {
    let mut pDS: LPDIRECTSOUND8 = core::ptr::null_mut();
    let mut hr: HRESULT = 0;
    TRACE!("(%s, %p)\n", debugstr_guid(riid), ppDS);
    if ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown))) == 0)
        && ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IDirectSound))) == 0)
    {
        *ppDS = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    /* Get dsound configuration */
    setup_dsound_options();

    hr = IDirectSoundImpl_Create(core::ptr::addr_of_mut!(pDS));
    if hr == (DS_OK as HRESULT) {
        hr = IDirectSound_IDirectSound_Create(pDS, ppDS);
        if !(*ppDS).is_null() {
            IDirectSound_IDirectSound_AddRef(*ppDS);
        } else {
            WARN!("IDirectSound_IDirectSound_Create failed\n");
            IDirectSound8_Release(pDS);
        }
    } else {
        WARN!("IDirectSoundImpl_Create failed\n");
        *ppDS = core::ptr::null_mut();
    }

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn DirectSoundCreate(
    lpcGUID: LPCGUID,
    ppDS: *mut LPDIRECTSOUND,
    pUnkOuter: *mut IUnknown,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut pDS: LPDIRECTSOUND = core::mem::zeroed();

    if ppDS.is_null() {
        WARN!("invalid parameter: ppDS == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    if !pUnkOuter.is_null() {
        WARN!("invalid parameter: pUnkOuter != NULL\n");
        *ppDS = core::ptr::null_mut();
        return DSERR_INVALIDPARAM;
    }

    hr = DSOUND_Create(core::ptr::addr_of!(IID_IDirectSound), core::ptr::addr_of_mut!(pDS));
    if hr == (DS_OK as HRESULT) {
        hr = IDirectSound_Initialize(pDS, lpcGUID);
        if hr != (DS_OK as HRESULT) {
            if hr != DSERR_ALREADYINITIALIZED {
                IDirectSound_Release(pDS);
                pDS = core::ptr::null_mut();
            } else {
                hr = (DS_OK as i32);
            }
        }
    }

    *ppDS = pDS;

    return hr;
}

#[no_mangle]
pub unsafe extern "C" fn DSOUND_Create8(riid: *const IID, ppDS: *mut LPDIRECTSOUND8) -> HRESULT {
    let mut pDS: LPDIRECTSOUND8 = core::ptr::null_mut();
    let mut hr: HRESULT = 0;
    if ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown))) == 0)
        && ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IDirectSound))) == 0)
        && ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IDirectSound8))) == 0)
    {
        *ppDS = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    /* Get dsound configuration */
    setup_dsound_options();

    hr = IDirectSoundImpl_Create(core::ptr::addr_of_mut!(pDS));
    if hr == (DS_OK as HRESULT) {
        hr = IDirectSound8_IDirectSound8_Create(pDS, ppDS);
        if !(*ppDS).is_null() {
            IDirectSound8_IDirectSound8_AddRef(*ppDS);
        } else {
            WARN!("IDirectSound8_IDirectSound8_Create failed\n");
            IDirectSound8_Release(pDS);
        }
    } else {
        WARN!("IDirectSoundImpl_Create failed\n");
        *ppDS = core::ptr::null_mut();
    }

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn DirectSoundCreate8(
    lpcGUID: LPCGUID,
    ppDS: *mut LPDIRECTSOUND8,
    pUnkOuter: *mut IUnknown,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut pDS: LPDIRECTSOUND8 = core::ptr::null_mut();

    if ppDS.is_null() {
        WARN!("invalid parameter: ppDS == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    if !pUnkOuter.is_null() {
        WARN!("invalid parameter: pUnkOuter != NULL\n");
        *ppDS = core::ptr::null_mut();
        return DSERR_INVALIDPARAM;
    }

    hr = DSOUND_Create8(
        core::ptr::addr_of!(IID_IDirectSound8),
        core::ptr::addr_of_mut!(pDS),
    );
    if hr == (DS_OK as HRESULT) {
        hr = IDirectSound8_Initialize(pDS, lpcGUID);
        if hr != (DS_OK as HRESULT) {
            if hr != DSERR_ALREADYINITIALIZED {
                IDirectSound8_Release(pDS);
                pDS = core::ptr::null_mut();
            } else {
                hr = (DS_OK as i32);
            }
        }
    }

    *ppDS = pDS;

    return hr;
}

unsafe extern "C" fn DirectSoundDevice_Create(ppDevice: *mut *mut DirectSoundDevice) -> HRESULT {
    let mut device: *mut DirectSoundDevice;

    /* Allocate memory */
    device = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<DirectSoundDevice>() as SIZE_T,
    ) as *mut DirectSoundDevice;
    if device.is_null() {
        WARN!("out of memory\n");
        return DSERR_OUTOFMEMORY as HRESULT;
    }

    (*device).ref_ = 1 as LONG;
    (*device).priolevel = DSSCL_NORMAL as DWORD;
    (*device).state = STATE_STOPPED as DWORD;
    (*device).speaker_config = (DSSPEAKER_STEREO
        | (DSSPEAKER_GEOMETRY_NARROW << 16)) as DWORD;

    /* 3D listener initial parameters */
    (*device).ds3dl.dwSize = core::mem::size_of::<DS3DLISTENER>() as DWORD;
    (*device).ds3dl.vPosition.x = 0.0f32;
    (*device).ds3dl.vPosition.y = 0.0f32;
    (*device).ds3dl.vPosition.z = 0.0f32;
    (*device).ds3dl.vVelocity.x = 0.0f32;
    (*device).ds3dl.vVelocity.y = 0.0f32;
    (*device).ds3dl.vVelocity.z = 0.0f32;
    (*device).ds3dl.vOrientFront.x = 0.0f32;
    (*device).ds3dl.vOrientFront.y = 0.0f32;
    (*device).ds3dl.vOrientFront.z = 1.0f32;
    (*device).ds3dl.vOrientTop.x = 0.0f32;
    (*device).ds3dl.vOrientTop.y = 1.0f32;
    (*device).ds3dl.vOrientTop.z = 0.0f32;
    (*device).ds3dl.flDistanceFactor = DS3D_DEFAULTDISTANCEFACTOR as D3DVALUE;
    (*device).ds3dl.flRolloffFactor = DS3D_DEFAULTROLLOFFFACTOR as D3DVALUE;
    (*device).ds3dl.flDopplerFactor = DS3D_DEFAULTDOPPLERFACTOR as D3DVALUE;

    (*device).prebuf = ds_snd_queue_max as UINT;
    (*device).guid = GUID_NULL;

    /* Set default wave format (may need it for waveOutOpen) */
    (*device).pwfx = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<WAVEFORMATEX>() as SIZE_T,
    ) as PWAVEFORMATEX;
    if (*device).pwfx.is_null() {
        WARN!("out of memory\n");
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            device as PVOID,
        );
        return DSERR_OUTOFMEMORY as HRESULT;
    }

    /* We rely on the sound driver to return the actual sound format of
     * the device if it does not support 22050x8x2 and is given the
     * WAVE_DIRECTSOUND flag.
     */
    (*(*device).pwfx).wFormatTag = WAVE_FORMAT_PCM as WORD;
    (*(*device).pwfx).nSamplesPerSec = ds_default_sample_rate as DWORD;
    (*(*device).pwfx).wBitsPerSample = ds_default_bits_per_sample as WORD;
    (*(*device).pwfx).nChannels = 2 as WORD;
    (*(*device).pwfx).nBlockAlign = (((*(*device).pwfx).wBitsPerSample as ::core::ffi::c_int
        * (*(*device).pwfx).nChannels as ::core::ffi::c_int)
        / 8 as ::core::ffi::c_int) as WORD;
    (*(*device).pwfx).nAvgBytesPerSec = (*(*device).pwfx)
        .nSamplesPerSec
        .wrapping_mul((*(*device).pwfx).nBlockAlign as DWORD);
    (*(*device).pwfx).cbSize = 0 as WORD;

    InitializeCriticalSection(&raw mut (*device).mixlock);
    (*(*device).mixlock.DebugInfo).__bindgen_anon_1.Spare[0 as usize] = concat!(
        file!(),
        ": DirectSoundDevice.mixlock\0"
    )
    .as_ptr() as DWORD_PTR;

    RtlInitializeResource(&raw mut (*device).buffer_list_lock);

    *ppDevice = device;

    return DS_OK as HRESULT;
}

unsafe extern "C" fn DirectSoundDevice_AddRef(device: *mut DirectSoundDevice) -> ULONG {
    let ref_0: ULONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*device).ref_) as *mut core::ffi::c_long,
    ) as ULONG;
    return ref_0;
}
#[no_mangle]
pub unsafe extern "C" fn DirectSoundDevice_Release(device: *mut DirectSoundDevice) -> ULONG {
    let mut hr: HRESULT;
    let ref_0: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*device).ref_)) as ULONG;
    if ref_0 == 0 {
        let mut i: core::ffi::c_int;
        timeKillEvent((*device).timerID);
        timeEndPeriod(DS_TIME_RES);

        /* The kill event should have allowed the timer process to expire
         * but try to grab the lock just in case. Can't hold lock because
         * IDirectSoundBufferImpl_Destroy also grabs the lock */
        RtlAcquireResourceShared(core::ptr::addr_of_mut!((*device).buffer_list_lock), TRUE as BYTE);
        RtlReleaseResource(core::ptr::addr_of_mut!((*device).buffer_list_lock));

        /* It is allowed to release this object even when buffers are playing */
        if !(*device).buffers.is_null() {
            WARN!("%d secondary buffers not released\n", (*device).nrofbuffers);
            i = 0 as core::ffi::c_int;
            while i < (*device).nrofbuffers {
                IDirectSoundBufferImpl_Destroy(*(*device).buffers.offset(i as isize));
                i += 1;
            }
        }

        if !(*device).primary.is_null() {
            WARN!("primary buffer not released\n");
            IDirectSoundBuffer8_Release((*device).primary as LPDIRECTSOUNDBUFFER8);
        }

        hr = DSOUND_PrimaryDestroy(device);
        if hr != DS_OK as HRESULT {
            WARN!("DSOUND_PrimaryDestroy failed\n");
        }

        if !(*device).driver.is_null() {
            IDsDriver_Close((*device).driver);
        }

        if (*device).drvdesc.dwFlags & DSDDESC_DOMMSYSTEMOPEN as DWORD != 0 {
            waveOutClose((*device).hwo);
        }

        if !(*device).driver.is_null() {
            IDsDriver_Release((*device).driver);
        }

        DSOUND_renderer[(*device).drvdesc.dnDevNode as usize] = core::ptr::null_mut();

        HeapFree(GetProcessHeap(), 0 as DWORD, (*device).tmp_buffer as PVOID);
        HeapFree(GetProcessHeap(), 0 as DWORD, (*device).mix_buffer as PVOID);
        if (*device).drvdesc.dwFlags & DSDDESC_USESYSTEMMEMORY as DWORD != 0 {
            HeapFree(GetProcessHeap(), 0 as DWORD, (*device).buffer as PVOID);
        }
        RtlDeleteResource(core::ptr::addr_of_mut!((*device).buffer_list_lock));
        (*(*device).mixlock.DebugInfo).__bindgen_anon_1.Spare[0 as usize] = 0 as DWORD_PTR;
        DeleteCriticalSection(core::ptr::addr_of_mut!((*device).mixlock));
        HeapFree(GetProcessHeap(), 0 as DWORD, device as PVOID);
    }
    return ref_0;
}

#[no_mangle]
pub unsafe extern "C" fn DirectSoundDevice_GetCaps(
    device: *mut DirectSoundDevice,
    lpDSCaps: LPDSCAPS,
) -> HRESULT {
    if device.is_null() {
        WARN!("not initialized\n");
        return DSERR_UNINITIALIZED;
    }

    if lpDSCaps.is_null() {
        WARN!("invalid parameter: lpDSCaps = NULL\n");
        return DSERR_INVALIDPARAM;
    }

    /* check if there is enough room */
    if ((*lpDSCaps).dwSize as usize) < core::mem::size_of::<DSCAPS>() {
        WARN!("invalid parameter: lpDSCaps->dwSize = %d\n", (*lpDSCaps).dwSize);
        return DSERR_INVALIDPARAM;
    }

    (*lpDSCaps).dwFlags = (*device).drvcaps.dwFlags;
    if ((TRACE_ON!(__wine_dbch_dsound)) != 0) {
        _dump_DSCAPS((*lpDSCaps).dwFlags);
    }
    (*lpDSCaps).dwMinSecondarySampleRate = (*device).drvcaps.dwMinSecondarySampleRate;
    (*lpDSCaps).dwMaxSecondarySampleRate = (*device).drvcaps.dwMaxSecondarySampleRate;
    (*lpDSCaps).dwPrimaryBuffers = (*device).drvcaps.dwPrimaryBuffers;
    (*lpDSCaps).dwMaxHwMixingAllBuffers = (*device).drvcaps.dwMaxHwMixingAllBuffers;
    (*lpDSCaps).dwMaxHwMixingStaticBuffers = (*device).drvcaps.dwMaxHwMixingStaticBuffers;
    (*lpDSCaps).dwMaxHwMixingStreamingBuffers = (*device).drvcaps.dwMaxHwMixingStreamingBuffers;
    (*lpDSCaps).dwFreeHwMixingAllBuffers = (*device).drvcaps.dwFreeHwMixingAllBuffers;
    (*lpDSCaps).dwFreeHwMixingStaticBuffers = (*device).drvcaps.dwFreeHwMixingStaticBuffers;
    (*lpDSCaps).dwFreeHwMixingStreamingBuffers = (*device).drvcaps.dwFreeHwMixingStreamingBuffers;
    (*lpDSCaps).dwMaxHw3DAllBuffers = (*device).drvcaps.dwMaxHw3DAllBuffers;
    (*lpDSCaps).dwMaxHw3DStaticBuffers = (*device).drvcaps.dwMaxHw3DStaticBuffers;
    (*lpDSCaps).dwMaxHw3DStreamingBuffers = (*device).drvcaps.dwMaxHw3DStreamingBuffers;
    (*lpDSCaps).dwFreeHw3DAllBuffers = (*device).drvcaps.dwFreeHw3DAllBuffers;
    (*lpDSCaps).dwFreeHw3DStaticBuffers = (*device).drvcaps.dwFreeHw3DStaticBuffers;
    (*lpDSCaps).dwFreeHw3DStreamingBuffers = (*device).drvcaps.dwFreeHw3DStreamingBuffers;
    (*lpDSCaps).dwTotalHwMemBytes = (*device).drvcaps.dwTotalHwMemBytes;
    (*lpDSCaps).dwFreeHwMemBytes = (*device).drvcaps.dwFreeHwMemBytes;
    (*lpDSCaps).dwMaxContigFreeHwMemBytes = (*device).drvcaps.dwMaxContigFreeHwMemBytes;

    /* driver doesn't have these */
    (*lpDSCaps).dwUnlockTransferRateHwBuffers = 4096; /* But we have none... */
    (*lpDSCaps).dwPlayCpuOverheadSwBuffers = 1; /* 1% */

    return (DS_OK) as HRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn DirectSoundDevice_Initialize(
    ppDevice: *mut *mut DirectSoundDevice,
    mut lpcGUID: LPCGUID,
) -> HRESULT {
    let mut hr: HRESULT = (DS_OK as i32);
    let mut wod: ::core::ffi::c_uint = 0;
    let mut wodn: ::core::ffi::c_uint = 0;
    let mut found: BOOLEAN = (FALSE as u8);
    let mut devGUID: GUID = core::mem::zeroed();
    let mut device: *mut DirectSoundDevice = *ppDevice;

    if !(*ppDevice).is_null() {
        WARN!("already initialized\n");
        return DSERR_ALREADYINITIALIZED;
    }

    /* Default device? */
    if lpcGUID.is_null() || IsEqualGUID!(lpcGUID, &raw const GUID_NULL) {
        lpcGUID = &raw const DSDEVID_DefaultPlayback;
    }

    if GetDeviceID(lpcGUID, &raw mut devGUID) != (DS_OK as i32) {
        WARN!("invalid parameter: lpcGUID\n");
        return DSERR_INVALIDPARAM;
    }

    /* Enumerate WINMM audio devices and find the one we want */
    wodn = waveOutGetNumDevs() as ::core::ffi::c_uint;
    if wodn == 0 {
        WARN!("no driver\n");
        return DSERR_NODRIVER;
    }

    wod = 0;
    while wod < wodn {
        if IsEqualGUID!(
            &raw const devGUID,
            (&raw mut DSOUND_renderer_guids as *mut GUID).offset(wod as isize) as *const GUID,
        ) {
            found = (TRUE as u8);
            break;
        }
        wod = wod.wrapping_add(1);
    }

    if found == (FALSE as BOOLEAN) {
        WARN!("No device found matching given ID!\n");
        return DSERR_NODRIVER;
    }

    if !DSOUND_renderer[wod as usize].is_null() {
        if IsEqualGUID!(
            &raw const devGUID,
            &raw const (*DSOUND_renderer[wod as usize]).guid,
        ) {
            device = DSOUND_renderer[wod as usize];
            DirectSoundDevice_AddRef(device);
            *ppDevice = device;
            return (DS_OK) as HRESULT;
        } else {
            ERR!("device GUID doesn't match\n");
            hr = DSERR_GENERIC;
            return hr;
        }
    } else {
        hr = DirectSoundDevice_Create(&raw mut device);
        if hr != (DS_OK as HRESULT) {
            WARN!("DirectSoundDevice_Create failed\n");
            return hr;
        }
    }

    *ppDevice = device;
    (*device).guid = devGUID;
    (*device).driver = core::ptr::null_mut();

    (*device).drvdesc.dnDevNode = wod as DWORD;
    hr = DSOUND_ReopenDevice(device, (FALSE) as BOOL);
    if hr < 0 {
        WARN!("DSOUND_ReopenDevice failed: %08x\n", hr);
        return hr;
    }

    if !(*device).driver.is_null() {
        /* the driver is now open, so it's now allowed to call GetCaps */
        hr = IDsDriver_GetCaps((*device).driver as *mut IDsDriver, &raw mut (*device).drvcaps);
        if hr != (DS_OK as HRESULT) {
            WARN!("IDsDriver_GetCaps failed\n");
            return hr;
        }
    } else {
        let mut woc: WAVEOUTCAPSA = core::mem::zeroed();
        hr = mmErr(waveOutGetDevCapsA(
            (*device).drvdesc.dnDevNode as UINT_PTR,
            &raw mut woc,
            core::mem::size_of::<WAVEOUTCAPSA>() as UINT,
        ) as UINT);
        if hr != (DS_OK as HRESULT) {
            WARN!("waveOutGetDevCaps failed\n");
            return hr;
        }
        ZeroMemory!(
            &raw mut (*device).drvcaps as *mut ::core::ffi::c_void,
            core::mem::size_of::<DSDRIVERCAPS>() as size_t,
        );
        if (woc.dwFormats & WAVE_FORMAT_1M08) != 0
            || (woc.dwFormats & WAVE_FORMAT_2M08) != 0
            || (woc.dwFormats & WAVE_FORMAT_4M08) != 0
            || (woc.dwFormats & WAVE_FORMAT_48M08) != 0
            || (woc.dwFormats & WAVE_FORMAT_96M08) != 0
        {
            (*device).drvcaps.dwFlags |= DSCAPS_PRIMARY8BIT;
            (*device).drvcaps.dwFlags |= DSCAPS_PRIMARYMONO;
        }
        if (woc.dwFormats & WAVE_FORMAT_1M16) != 0
            || (woc.dwFormats & WAVE_FORMAT_2M16) != 0
            || (woc.dwFormats & WAVE_FORMAT_4M16) != 0
            || (woc.dwFormats & WAVE_FORMAT_48M16) != 0
            || (woc.dwFormats & WAVE_FORMAT_96M16) != 0
        {
            (*device).drvcaps.dwFlags |= DSCAPS_PRIMARY16BIT;
            (*device).drvcaps.dwFlags |= DSCAPS_PRIMARYMONO;
        }
        if (woc.dwFormats & WAVE_FORMAT_1S08) != 0
            || (woc.dwFormats & WAVE_FORMAT_2S08) != 0
            || (woc.dwFormats & WAVE_FORMAT_4S08) != 0
            || (woc.dwFormats & WAVE_FORMAT_48S08) != 0
            || (woc.dwFormats & WAVE_FORMAT_96S08) != 0
        {
            (*device).drvcaps.dwFlags |= DSCAPS_PRIMARY8BIT;
            (*device).drvcaps.dwFlags |= DSCAPS_PRIMARYSTEREO;
        }
        if (woc.dwFormats & WAVE_FORMAT_1S16) != 0
            || (woc.dwFormats & WAVE_FORMAT_2S16) != 0
            || (woc.dwFormats & WAVE_FORMAT_4S16) != 0
            || (woc.dwFormats & WAVE_FORMAT_48S16) != 0
            || (woc.dwFormats & WAVE_FORMAT_96S16) != 0
        {
            (*device).drvcaps.dwFlags |= DSCAPS_PRIMARY16BIT;
            (*device).drvcaps.dwFlags |= DSCAPS_PRIMARYSTEREO;
        }
        if ds_emuldriver != 0 {
            (*device).drvcaps.dwFlags |= DSCAPS_EMULDRIVER;
        }
        (*device).drvcaps.dwMinSecondarySampleRate = DSBFREQUENCY_MIN;
        (*device).drvcaps.dwMaxSecondarySampleRate = DSBFREQUENCY_MAX;
        ZeroMemory!(
            &raw mut (*device).volpan as *mut ::core::ffi::c_void,
            core::mem::size_of::<DSVOLUMEPAN>() as size_t,
        );
    }

    hr = DSOUND_PrimaryCreate(device);
    if hr == (DS_OK as HRESULT) {
        let mut triggertime: UINT = DS_TIME_DEL;
        let mut res: UINT = DS_TIME_RES;
        let mut id: UINT;
        let mut time: TIMECAPS = core::mem::zeroed();

        DSOUND_renderer[(*device).drvdesc.dnDevNode as usize] = device;
        timeGetDevCaps(
            &raw mut time,
            core::mem::size_of::<TIMECAPS>() as UINT,
        );

        if triggertime < time.wPeriodMin {
            triggertime = time.wPeriodMin;
        }
        if res < time.wPeriodMin {
            res = time.wPeriodMin;
        }
        if timeBeginPeriod(res) == TIMERR_NOCANDO {
            WARN!("Could not set minimum resolution, don't expect sound\n");
        }
        id = timeSetEvent(
            triggertime,
            res,
            Some(
                DSOUND_timer,
            ),
            device as DWORD_PTR,
            (TIME_PERIODIC | TIME_KILL_SYNCHRONOUS) as UINT,
        ) as UINT;
        if id == 0 {
            WARN!("Timer not created! Retrying without TIME_KILL_SYNCHRONOUS\n");
            id = timeSetEvent(
                triggertime,
                res,
                Some(
                    DSOUND_timer,
                ),
                device as DWORD_PTR,
                TIME_PERIODIC as UINT,
            ) as UINT;
            if id == 0 {
                ERR!("Could not create timer, sound playback will not occur\n");
            }
        }
        (*DSOUND_renderer[(*device).drvdesc.dnDevNode as usize]).timerID = id;
    } else {
        WARN!("DSOUND_PrimaryCreate failed\n");
    }

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn DirectSoundDevice_CreateSoundBuffer(
    mut device: *mut DirectSoundDevice,
    mut dsbd: LPCDSBUFFERDESC,
    mut ppdsb: LPLPDIRECTSOUNDBUFFER,
    mut lpunk: LPUNKNOWN,
    mut from8: BOOL,
) -> HRESULT {
    let mut hres: HRESULT = DS_OK as HRESULT;

    if device.is_null() {
        WARN!("not initialized\n");
        return DSERR_UNINITIALIZED;
    }

    if dsbd.is_null() {
        WARN!("invalid parameter: dsbd == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    if ((*dsbd).dwSize as usize) != core::mem::size_of::<DSBUFFERDESC>()
        && ((*dsbd).dwSize as usize) != core::mem::size_of::<DSBUFFERDESC1>()
    {
        WARN!("invalid parameter: dsbd\n");
        return DSERR_INVALIDPARAM;
    }

    if ppdsb.is_null() {
        WARN!("invalid parameter: ppdsb == NULL\n");
        return DSERR_INVALIDPARAM;
    }
    *ppdsb = core::ptr::null_mut::<IDirectSoundBuffer>();

    if ((TRACE_ON!(__wine_dbch_dsound)) != 0) {
        _dump_DSBCAPS((*dsbd).dwFlags);
    }

    if (*dsbd).dwFlags & DSBCAPS_PRIMARYBUFFER != 0 {
        if !(*dsbd).lpwfxFormat.is_null() {
            WARN!("invalid parameter: dsbd->lpwfxFormat must be NULL for primary buffer\n");
            return DSERR_INVALIDPARAM;
        }

        if !(*device).primary.is_null() {
            WARN!("Primary Buffer already created\n");
            IDirectSoundBuffer_AddRef(((*device).primary as LPDIRECTSOUNDBUFFER8).cast());
            *ppdsb = (*device).primary as LPDIRECTSOUNDBUFFER as *mut IDirectSoundBuffer;
        } else {
            hres = primarybuffer_create(
                device,
                &raw mut (*device).primary,
                dsbd as *const DSBUFFERDESC,
            );
            if !(*device).primary.is_null() {
                *ppdsb = &raw mut (*(*device).primary).IDirectSoundBuffer8_iface
                    as *mut IDirectSoundBuffer;
                (*(*device).primary).dsbd.dwFlags &=
                    !(DSBCAPS_LOCHARDWARE | DSBCAPS_LOCSOFTWARE) as DWORD;
                if !(*device).hwbuf.is_null() {
                    (*(*device).primary).dsbd.dwFlags |= DSBCAPS_LOCHARDWARE as DWORD;
                } else {
                    (*(*device).primary).dsbd.dwFlags |= DSBCAPS_LOCSOFTWARE as DWORD;
                }
            } else {
                WARN!("primarybuffer_create() failed\n");
            }
        }
    } else {
        let mut dsb: *mut IDirectSoundBufferImpl =
            core::ptr::null_mut::<IDirectSoundBufferImpl>();
        let mut pwfxe: *mut WAVEFORMATEXTENSIBLE =
            core::ptr::null_mut::<WAVEFORMATEXTENSIBLE>();

        if (*dsbd).lpwfxFormat.is_null() {
            WARN!("invalid parameter: dsbd->lpwfxFormat can't be NULL for secondary buffer\n");
            return DSERR_INVALIDPARAM;
        }
        pwfxe = (*dsbd).lpwfxFormat as *mut WAVEFORMATEXTENSIBLE;

        if (*pwfxe).Format.wBitsPerSample != 16
            && (*pwfxe).Format.wBitsPerSample != 8
            && (*pwfxe).Format.wFormatTag != (WAVE_FORMAT_EXTENSIBLE as u16)
        {
            WARN!(
                "wBitsPerSample=%d needs a WAVEFORMATEXTENSIBLE\n",
                (*(*dsbd).lpwfxFormat).wBitsPerSample as i32
            );
            return DSERR_CONTROLUNAVAIL;
        }
        if (*pwfxe).Format.wFormatTag == (WAVE_FORMAT_EXTENSIBLE as u16) {
            /* check if cbSize is at least 22 bytes */
            if ((*pwfxe).Format.cbSize as usize)
                < (core::mem::size_of::<WAVEFORMATEXTENSIBLE>())
                    .wrapping_sub(core::mem::size_of::<WAVEFORMATEX>())
            {
                WARN!("Too small a cbSize %u\n", (*pwfxe).Format.cbSize as u32);
                return DSERR_INVALIDPARAM;
            }

            /* cbSize should be 22 bytes, with one possible exception */
            if ((*pwfxe).Format.cbSize as usize)
                > (core::mem::size_of::<WAVEFORMATEXTENSIBLE>())
                    .wrapping_sub(core::mem::size_of::<WAVEFORMATEX>())
                && !((IsEqualGUID!(
                    &raw mut (*pwfxe).SubFormat,
                    &raw const KSDATAFORMAT_SUBTYPE_PCM,
                ) != false
                    || IsEqualGUID!(
                        &raw mut (*pwfxe).SubFormat,
                        &raw const KSDATAFORMAT_SUBTYPE_IEEE_FLOAT,
                    ) != false)
                    && ((*pwfxe).Format.cbSize as usize)
                        == core::mem::size_of::<WAVEFORMATEXTENSIBLE>())
            {
                WARN!("Too big a cbSize %u\n", (*pwfxe).Format.cbSize as u32);
                return DSERR_CONTROLUNAVAIL;
            }

            if (IsEqualGUID!(
                &raw mut (*pwfxe).SubFormat,
                &raw const KSDATAFORMAT_SUBTYPE_PCM,
            ) == false)
                && (IsEqualGUID!(
                    &raw mut (*pwfxe).SubFormat,
                    &raw const KSDATAFORMAT_SUBTYPE_IEEE_FLOAT,
                ) == false)
            {
                if IsEqualGUID!(
                    &raw mut (*pwfxe).SubFormat,
                    &raw const GUID_NULL,
                ) == false
                {
                    FIXME!(
                        "SubFormat %s not supported right now.\n",
                        debugstr_guid(&raw mut (*pwfxe).SubFormat)
                    );
                }
                return DSERR_INVALIDPARAM;
            }
            if (*pwfxe).Samples.wValidBitsPerSample > (*(*dsbd).lpwfxFormat).wBitsPerSample {
                WARN!(
                    "Samples.wValidBitsPerSample(%d) > Format.wBitsPerSample (%d)\n",
                    (*pwfxe).Samples.wValidBitsPerSample as i32,
                    (*pwfxe).Format.wBitsPerSample as i32
                );
                return DSERR_INVALIDPARAM;
            }
            if (*pwfxe).Samples.wValidBitsPerSample != 0
                && (*pwfxe).Samples.wValidBitsPerSample < (*(*dsbd).lpwfxFormat).wBitsPerSample
            {
                FIXME!(
                    "Non-packed formats not supported right now: %d/%d\n",
                    (*pwfxe).Samples.wValidBitsPerSample as i32,
                    (*(*dsbd).lpwfxFormat).wBitsPerSample as i32
                );
                return DSERR_CONTROLUNAVAIL;
            }
        }

        if from8 != 0
            && ((*dsbd).dwFlags & DSBCAPS_CTRL3D) != 0
            && ((*(*dsbd).lpwfxFormat).nChannels != 1)
        {
            WARN!("invalid parameter: 3D buffer format must be mono\n");
            return DSERR_INVALIDPARAM;
        }

        hres = IDirectSoundBufferImpl_Create(device, &raw mut dsb, dsbd);
        if !dsb.is_null() {
            *ppdsb = &raw mut (*dsb).IDirectSoundBuffer8_iface as *mut IDirectSoundBuffer;
        } else {
            WARN!("IDirectSoundBufferImpl_Create failed\n");
        }
    }

    return hres;
}
#[no_mangle]
pub unsafe extern "C" fn DirectSoundDevice_DuplicateSoundBuffer(
    device: *mut DirectSoundDevice,
    psb: LPDIRECTSOUNDBUFFER,
    ppdsb: LPLPDIRECTSOUNDBUFFER,
) -> HRESULT {
    let mut hres: HRESULT = (DS_OK as i32);
    let mut dsb: *mut IDirectSoundBufferImpl = core::ptr::null_mut();

    if device.is_null() {
        WARN!("not initialized\n");
        return DSERR_UNINITIALIZED;
    }

    if psb.is_null() {
        WARN!("invalid parameter: psb == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    if ppdsb.is_null() {
        WARN!("invalid parameter: ppdsb == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    /* make sure we have a secondary buffer */
    if psb == core::ptr::addr_of_mut!((*(*device).primary).IDirectSoundBuffer8_iface) as *mut IDirectSoundBuffer {
        WARN!("trying to duplicate primary buffer\n");
        *ppdsb = core::ptr::null_mut();
        return DSERR_INVALIDCALL;
    }

    /* duplicate the actual buffer implementation */
    hres = IDirectSoundBufferImpl_Duplicate(device, core::ptr::addr_of_mut!(dsb), psb as *mut IDirectSoundBufferImpl);
    if hres == (DS_OK as HRESULT) {
        *ppdsb = core::ptr::addr_of_mut!((*dsb).IDirectSoundBuffer8_iface) as *mut IDirectSoundBuffer;
    } else {
        WARN!("IDirectSoundBufferImpl_Duplicate failed\n");
    }

    return hres;
}
#[no_mangle]
pub unsafe extern "C" fn DirectSoundDevice_SetCooperativeLevel(
    mut device: *mut DirectSoundDevice,
    mut hwnd: HWND,
    mut level: DWORD,
) -> HRESULT {
    TRACE!(
        "(%p,%p,%s)\n",
        device,
        hwnd,
        dumpCooperativeLevel(level)
    );

    if device.is_null() {
        WARN!("not initialized\n");
        return DSERR_UNINITIALIZED;
    }

    if level == DSSCL_PRIORITY as DWORD || level == DSSCL_EXCLUSIVE as DWORD {
        WARN!(
            "level=%s not fully supported\n",
            if level == DSSCL_PRIORITY as DWORD {
                b"DSSCL_PRIORITY\0".as_ptr() as *const i8
            } else {
                b"DSSCL_EXCLUSIVE\0".as_ptr() as *const i8
            }
        );
    }

    (*device).priolevel = level;
    return (DS_OK) as HRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn DirectSoundDevice_Compact(device: *mut DirectSoundDevice) -> HRESULT {
    if device.is_null() {
        WARN!("not initialized\n");
        return DSERR_UNINITIALIZED;
    }

    if (*device).priolevel < DSSCL_PRIORITY {
        WARN!("incorrect priority level\n");
        return DSERR_PRIOLEVELNEEDED;
    }

    return (DS_OK) as HRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn DirectSoundDevice_GetSpeakerConfig(
    device: *mut DirectSoundDevice,
    lpdwSpeakerConfig: LPDWORD,
) -> HRESULT {
    if device.is_null() {
        WARN!("not initialized\n");
        return DSERR_UNINITIALIZED;
    }

    if lpdwSpeakerConfig.is_null() {
        WARN!("invalid parameter: lpdwSpeakerConfig == NULL\n");
        return DSERR_INVALIDPARAM;
    }

    WARN!("not fully functional\n");
    *lpdwSpeakerConfig = (*device).speaker_config;
    return (DS_OK) as HRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn DirectSoundDevice_SetSpeakerConfig(
    device: *mut DirectSoundDevice,
    config: DWORD,
) -> HRESULT {
    TRACE!("(%p,0x%08x)\n", device, config);
    if device.is_null() {
        WARN!("not initialized\n");
        return DSERR_UNINITIALIZED;
    }

    (*device).speaker_config = config;
    WARN!("not fully functional\n");
    return (DS_OK) as HRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn DirectSoundDevice_VerifyCertification(
    mut device: *mut DirectSoundDevice,
    mut pdwCertified: LPDWORD,
) -> HRESULT {
    TRACE!("(%p, %p)\n", device, pdwCertified);

    if device.is_null() {
        WARN!("not initialized\n");
        return DSERR_UNINITIALIZED as HRESULT;
    }

    if (*device).drvcaps.dwFlags & DSCAPS_CERTIFIED as DWORD != 0 {
        *pdwCertified = DS_CERTIFIED as DWORD;
    } else {
        *pdwCertified = DS_UNCERTIFIED as DWORD;
    }

    return DS_OK as HRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn DirectSoundDevice_AddBuffer(
    device: *mut DirectSoundDevice,
    pDSB: *mut IDirectSoundBufferImpl,
) -> HRESULT {
    let mut newbuffers: *mut *mut IDirectSoundBufferImpl = core::ptr::null_mut();
    let mut hr: HRESULT = (DS_OK as i32);

    TRACE!("(%p, %p)\n", device, pDSB);

    RtlAcquireResourceExclusive(core::ptr::addr_of_mut!((*device).buffer_list_lock), (TRUE) as BYTE);

    if !(*device).buffers.is_null() {
        newbuffers = HeapReAlloc(
            GetProcessHeap(),
            0,
            (*device).buffers as LPVOID,
            (core::mem::size_of::<*mut IDirectSoundBufferImpl>() as SIZE_T)
                .wrapping_mul(((*device).nrofbuffers + 1) as SIZE_T),
        ) as *mut *mut IDirectSoundBufferImpl;
    } else {
        newbuffers = HeapAlloc(
            GetProcessHeap(),
            0,
            (core::mem::size_of::<*mut IDirectSoundBufferImpl>() as SIZE_T)
                .wrapping_mul(((*device).nrofbuffers + 1) as SIZE_T),
        ) as *mut *mut IDirectSoundBufferImpl;
    }

    if !newbuffers.is_null() {
        (*device).buffers = newbuffers;
        *(*device).buffers.offset((*device).nrofbuffers as isize) = pDSB;
        (*device).nrofbuffers += 1;
        TRACE!("buffer count is now %d\n", (*device).nrofbuffers);
    } else {
        ERR!("out of memory for buffer list! Current buffer count is %d\n", (*device).nrofbuffers);
        hr = DSERR_OUTOFMEMORY;
    }

    RtlReleaseResource(core::ptr::addr_of_mut!((*device).buffer_list_lock));

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn DirectSoundDevice_RemoveBuffer(mut device: *mut DirectSoundDevice, mut pDSB: *mut IDirectSoundBufferImpl) -> HRESULT {
    let mut i: ::core::ffi::c_int = 0;
    let mut hr: HRESULT = DS_OK as HRESULT;
    WINE_TRACE!("(%p, %p)\n", device, pDSB);
    RtlAcquireResourceExclusive(&raw mut (*device).buffer_list_lock, TRUE as BYTE);
    i = 0 as ::core::ffi::c_int;
    while i < (*device).nrofbuffers {
        if *(*device).buffers.offset(i as isize) == pDSB {
            break;
        }
        i += 1;
    }
    if i < (*device).nrofbuffers {
        let ref mut fresh21 = *(*device).buffers.offset(i as isize);
        *fresh21 = *(*device)
            .buffers
            .offset(((*device).nrofbuffers - 1 as ::core::ffi::c_int) as isize);
        (*device).nrofbuffers -= 1;
        (*device).buffers = HeapReAlloc(
            GetProcessHeap(),
            0 as DWORD,
            (*device).buffers as PVOID,
            (::core::mem::size_of::<LPDIRECTSOUNDBUFFER8>() as SIZE_T)
                .wrapping_mul((*device).nrofbuffers as SIZE_T),
        ) as *mut *mut IDirectSoundBufferImpl;
        WINE_TRACE!("buffer count is now %d\n", (*device).nrofbuffers);
    }
    if (*device).nrofbuffers == 0 as ::core::ffi::c_int {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            (*device).buffers as PVOID,
        );
        (*device).buffers = ::core::ptr::null_mut::<*mut IDirectSoundBufferImpl>();
    }
    RtlReleaseResource(&raw mut (*device).buffer_list_lock);
    return hr;
}

// COM vtable tables — generated from dsound.c by vtbl_gen.py
/// dsound.c:346 `static const IUnknownVtbl DirectSound_Unknown_Vtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static DirectSound_Unknown_Vtbl: IUnknownVtbl = IUnknownVtbl {
    QueryInterface: Some(IDirectSound_IUnknown_QueryInterface),
    AddRef: Some(IDirectSound_IUnknown_AddRef),
    Release: Some(IDirectSound_IUnknown_Release),
};

/// dsound.c:500 `static const IDirectSoundVtbl DirectSound_DirectSound_Vtbl`  (slot order: rust:types_amd64.rs)
pub static DirectSound_DirectSound_Vtbl: IDirectSoundVtbl = IDirectSoundVtbl {
    QueryInterface: Some(IDirectSound_IDirectSound_QueryInterface),
    AddRef: Some(IDirectSound_IDirectSound_AddRef),
    Release: Some(IDirectSound_IDirectSound_Release),
    CreateSoundBuffer: Some(IDirectSound_IDirectSound_CreateSoundBuffer),
    GetCaps: Some(IDirectSound_IDirectSound_GetCaps),
    DuplicateSoundBuffer: Some(IDirectSound_IDirectSound_DuplicateSoundBuffer),
    SetCooperativeLevel: Some(IDirectSound_IDirectSound_SetCooperativeLevel),
    Compact: Some(IDirectSound_IDirectSound_Compact),
    GetSpeakerConfig: Some(IDirectSound_IDirectSound_GetSpeakerConfig),
    SetSpeakerConfig: Some(IDirectSound_IDirectSound_SetSpeakerConfig),
    Initialize: Some(IDirectSound_IDirectSound_Initialize),
};

/// dsound.c:587 `static const IUnknownVtbl DirectSound8_Unknown_Vtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static DirectSound8_Unknown_Vtbl: IUnknownVtbl = IUnknownVtbl {
    QueryInterface: Some(IDirectSound8_IUnknown_QueryInterface),
    AddRef: Some(IDirectSound8_IUnknown_AddRef),
    Release: Some(IDirectSound8_IUnknown_Release),
};

/// dsound.c:741 `static const IDirectSoundVtbl DirectSound8_DirectSound_Vtbl`  (slot order: rust:types_amd64.rs)
pub static DirectSound8_DirectSound_Vtbl: IDirectSoundVtbl = IDirectSoundVtbl {
    QueryInterface: Some(IDirectSound8_IDirectSound_QueryInterface),
    AddRef: Some(IDirectSound8_IDirectSound_AddRef),
    Release: Some(IDirectSound8_IDirectSound_Release),
    CreateSoundBuffer: Some(IDirectSound8_IDirectSound_CreateSoundBuffer),
    GetCaps: Some(IDirectSound8_IDirectSound_GetCaps),
    DuplicateSoundBuffer: Some(IDirectSound8_IDirectSound_DuplicateSoundBuffer),
    SetCooperativeLevel: Some(IDirectSound8_IDirectSound_SetCooperativeLevel),
    Compact: Some(IDirectSound8_IDirectSound_Compact),
    GetSpeakerConfig: Some(IDirectSound8_IDirectSound_GetSpeakerConfig),
    SetSpeakerConfig: Some(IDirectSound8_IDirectSound_SetSpeakerConfig),
    Initialize: Some(IDirectSound8_IDirectSound_Initialize),
};

/// dsound.c:912 `static const IDirectSound8Vtbl DirectSound8_DirectSound8_Vtbl`  (slot order: rust:types_amd64.rs)
pub static DirectSound8_DirectSound8_Vtbl: IDirectSound8Vtbl = IDirectSound8Vtbl {
    QueryInterface: Some(IDirectSound8_IDirectSound8_QueryInterface),
    AddRef: Some(IDirectSound8_IDirectSound8_AddRef),
    Release: Some(IDirectSound8_IDirectSound8_Release),
    CreateSoundBuffer: Some(IDirectSound8_IDirectSound8_CreateSoundBuffer),
    GetCaps: Some(IDirectSound8_IDirectSound8_GetCaps),
    DuplicateSoundBuffer: Some(IDirectSound8_IDirectSound8_DuplicateSoundBuffer),
    SetCooperativeLevel: Some(IDirectSound8_IDirectSound8_SetCooperativeLevel),
    Compact: Some(IDirectSound8_IDirectSound8_Compact),
    GetSpeakerConfig: Some(IDirectSound8_IDirectSound8_GetSpeakerConfig),
    SetSpeakerConfig: Some(IDirectSound8_IDirectSound8_SetSpeakerConfig),
    Initialize: Some(IDirectSound8_IDirectSound8_Initialize),
    VerifyCertification: Some(IDirectSound8_IDirectSound8_VerifyCertification),
};
