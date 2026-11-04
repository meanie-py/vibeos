use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_13 {
    pub mask: DWORD,
    pub name: *const ::core::ffi::c_char,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_4 {
    pub Flags: DWORD,
    pub CreatorBackTraceIndexHigh: WORD,
    pub SpareWORD: WORD,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_3 {
    pub WineDebugString: DWORD_PTR,
    pub Spare: [DWORD_PTR; 1],
    pub c2rust_unnamed: C2RustUnnamed_4,
}

static mut dinput_devices: [*const dinput_device; 5] = unsafe {
    [
        &raw const mouse_device,
        &raw const keyboard_device,
        &raw const joystick_linuxinput_device,
        &raw const joystick_linux_device,
        &raw const joystick_osx_device,
    ]
};

#[no_mangle]
pub static mut DINPUT_instance: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

static mut direct_input_list: list = unsafe {
    list {
        next: &raw const direct_input_list as *mut list,
        prev: &raw const direct_input_list as *mut list,
    }
};

static mut DINPUT_CF: IClassFactoryImpl = unsafe {
    IClassFactoryImpl {
        IClassFactory_iface: IClassFactory {
            lpVtbl: &raw const DICF_Vtbl,
        },
        ref_: 1 as LONG,
    }
};

static mut hook_thread_id: DWORD = 0;

static mut hook_thread_event: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

static mut dinput_critsect_debug: CRITICAL_SECTION_DEBUG = unsafe { _RTL_CRITICAL_SECTION_DEBUG {
        Type: 0 as WORD,
        CreatorBackTraceIndex: 0 as WORD,
        CriticalSection: &raw mut dinput_hook_crit,
        ProcessLocksList: _LIST_ENTRY {
            Flink: &raw mut dinput_critsect_debug.ProcessLocksList,
            Blink: &raw mut dinput_critsect_debug.ProcessLocksList,
        },
        EntryCount: 0 as DWORD,
        ContentionCount: 0 as DWORD,
        __bindgen_anon_1: _RTL_CRITICAL_SECTION_DEBUG__bindgen_ty_1 { Spare: [0; 1] },
    }};

static mut dinput_hook_crit: CRITICAL_SECTION = unsafe {
    _RTL_CRITICAL_SECTION {
        DebugInfo: &raw const dinput_critsect_debug as PRTL_CRITICAL_SECTION_DEBUG,
        LockCount: -(1 as ::core::ffi::c_int) as LONG,
        RecursionCount: 0 as LONG,
        OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        SpinCount: 0 as ULONG_PTR,
    }
};

// WINE_DEFAULT_DEBUG_CHANNEL(dinput) -- debug-channel macro, no Rust declaration emitted
// COBJMACROS / NONAMELESSUNION -- configuration defines, no declaration emitted
// Forward prototypes (check_hook_thread, initialize_directinput_instance,
// uninitialize_directinput_instance) and enum_callback_wrapper are functions; emitted by the function pass.
// Vtables ddi7avt, ddi7wvt, ddi8avt, ddi8wvt, JoyConfig8vt, DICF_Vtbl are skipped (vtbl_gen.py).

#[repr(C)]
pub struct IClassFactoryImpl {
    pub IClassFactory_iface: IClassFactory,
    pub ref_: LONG,
}
pub type PIClassFactoryImpl = *mut IClassFactoryImpl;

pub const DIRECTINPUT_VERSION_300: i32 = 0x0300;
pub const DIRECTINPUT_VERSION_500: i32 = 0x0500;
pub const DIRECTINPUT_VERSION_50A: i32 = 0x050A;
pub const DIRECTINPUT_VERSION_5B2: i32 = 0x05B2;
pub const DIRECTINPUT_VERSION_602: i32 = 0x0602;
pub const DIRECTINPUT_VERSION_61A: i32 = 0x061A;
pub const DIRECTINPUT_VERSION_700: i32 = 0x0700;

// Pointers to other translation units' statics: not Sync, so declared `static mut`.

unsafe extern "C" fn impl_from_IDirectInput7A(iface: *mut IDirectInput7A) -> *mut IDirectInputImpl {
    return (iface as *mut u8).sub(core::mem::offset_of!(IDirectInputImpl, IDirectInput7A_iface))
        as *mut IDirectInputImpl;
}

unsafe extern "C" fn impl_from_IDirectInput7W(iface: *mut IDirectInput7W) -> *mut IDirectInputImpl {
    return CONTAINING_RECORD!(iface, IDirectInputImpl, IDirectInput7W_iface) as *mut IDirectInputImpl;
}

unsafe extern "C" fn impl_from_IDirectInput8A(iface: *mut IDirectInput8A) -> *mut IDirectInputImpl {
    return (iface as *mut u8).sub(core::mem::offset_of!(IDirectInputImpl, IDirectInput8A_iface))
        as *mut IDirectInputImpl;
}

unsafe extern "C" fn impl_from_IDirectInput8W(mut iface: *mut IDirectInput8W) -> *mut IDirectInputImpl {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(IDirectInputImpl, IDirectInput8W_iface)) as *mut IDirectInputImpl;
}

unsafe extern "C" fn impl_from_IDirectInputDevice8W(iface: *mut IDirectInputDevice8W) -> *mut IDirectInputDeviceImpl {
    return CONTAINING_RECORD!(iface, IDirectInputDeviceImpl, IDirectInputDevice8W_iface);
}

unsafe extern "C" fn create_directinput_instance(riid: *const IID, ppDI: *mut LPVOID, out: *mut *mut IDirectInputImpl) -> HRESULT {
    let mut This: *mut IDirectInputImpl = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY as DWORD, ::core::mem::size_of::<IDirectInputImpl>() as SIZE_T) as *mut IDirectInputImpl;
    let mut hr: HRESULT = 0;

    if This.is_null() {
        return E_OUTOFMEMORY;
    }

    (*This).IDirectInput7A_iface.lpVtbl = &raw const ddi7avt as *const IDirectInput7AVtbl;
    (*This).IDirectInput7W_iface.lpVtbl = &raw const ddi7wvt as *const IDirectInput7WVtbl;
    (*This).IDirectInput8A_iface.lpVtbl = &raw const ddi8avt as *const IDirectInput8AVtbl;
    (*This).IDirectInput8W_iface.lpVtbl = &raw const ddi8wvt as *const IDirectInput8WVtbl;
    (*This).IDirectInputJoyConfig8_iface.lpVtbl = &raw const JoyConfig8vt as *const IDirectInputJoyConfig8Vtbl;

    hr = IDirectInput_QueryInterface!(&raw mut (*This).IDirectInput7A_iface, riid, ppDI as *mut ::core::ffi::c_void);
    if hr < 0 {
        HeapFree(GetProcessHeap(), 0 as DWORD, This as PVOID);
        return hr;
    }

    if !out.is_null() {
        *out = This;
    }
    return DI_OK;
}

#[no_mangle]
pub unsafe extern "system" fn DirectInputCreateEx(
    hinst: HINSTANCE,
    dwVersion: DWORD,
    riid: *const IID,
    ppDI: *mut LPVOID,
    punkOuter: LPUNKNOWN,
) -> HRESULT {
    let mut This: *mut IDirectInputImpl = core::ptr::null_mut();
    let mut hr: HRESULT = 0;
    WINE_TRACE!("(%p,%04x,%s,%p,%p)\n", hinst, dwVersion, debugstr_guid(riid as *const _GUID), ppDI, punkOuter);

    if IsEqualGUID!(&raw const IID_IDirectInputA, riid)
        || IsEqualGUID!(&raw const IID_IDirectInput2A, riid)
        || IsEqualGUID!(&raw const IID_IDirectInput7A, riid)
        || IsEqualGUID!(&raw const IID_IDirectInputW, riid)
        || IsEqualGUID!(&raw const IID_IDirectInput2W, riid)
        || IsEqualGUID!(&raw const IID_IDirectInput7W, riid)
    {
        hr = create_directinput_instance(riid, ppDI, &raw mut This);
        if hr < 0 {
            return hr;
        }
    } else {
        return DIERR_NOINTERFACE;
    }

    hr = IDirectInput_Initialize!(&raw mut (*This).IDirectInput7A_iface, hinst, dwVersion);
    if hr < 0 {
        IDirectInput_Release!(&raw mut (*This).IDirectInput7A_iface);
        *ppDI = core::ptr::null_mut();
        return hr;
    }

    return DI_OK;
}

#[no_mangle]
pub unsafe extern "system" fn DirectInput8Create(
    hinst: HINSTANCE,
    version: DWORD,
    iid: REFIID,
    out: *mut *mut core::ffi::c_void,
    outer: *mut IUnknown,
) -> HRESULT {
    let mut This: *mut IDirectInputImpl = core::ptr::null_mut();
    let mut hr: HRESULT;

    if out.is_null() {
        return E_POINTER;
    }

    if !IsEqualGUID!(core::ptr::addr_of!(IID_IDirectInput8A), iid)
        && !IsEqualGUID!(core::ptr::addr_of!(IID_IDirectInput8W), iid)
        && !IsEqualGUID!(core::ptr::addr_of!(IID_IUnknown), iid)
    {
        *out = core::ptr::null_mut();
        return DIERR_NOINTERFACE;
    }

    hr = create_directinput_instance(iid, out as *mut LPVOID, core::ptr::addr_of_mut!(This));

    if hr < 0 {
        ERR!("Failed to create DirectInput, hr %#x.\n", hr);
        return hr;
    }

    /* When aggregation is used, the application needs to manually call Initialize(). */
    if outer.is_null() && IsEqualGUID!(core::ptr::addr_of!(IID_IDirectInput8A), iid) {
        hr = IDirectInput8_Initialize!(
            core::ptr::addr_of_mut!((*This).IDirectInput8A_iface),
            hinst,
            version,
        );
        if hr < 0 {
            IDirectInput8_Release!(core::ptr::addr_of_mut!((*This).IDirectInput8A_iface));
            *out = core::ptr::null_mut();
            return hr;
        }
    }

    if outer.is_null() && IsEqualGUID!(core::ptr::addr_of!(IID_IDirectInput8W), iid) {
        hr = IDirectInput8_Initialize!(
            core::ptr::addr_of_mut!((*This).IDirectInput8W_iface),
            hinst,
            version,
        );
        if hr < 0 {
            IDirectInput8_Release!(core::ptr::addr_of_mut!((*This).IDirectInput8W_iface));
            *out = core::ptr::null_mut();
            return hr;
        }
    }

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn DirectInputCreateA(hinst: HINSTANCE, dwVersion: DWORD, ppDI: *mut LPDIRECTINPUTA, punkOuter: LPUNKNOWN) -> HRESULT {
    return DirectInputCreateEx(hinst, dwVersion, core::ptr::addr_of!(IID_IDirectInput7A), ppDI as *mut LPVOID, punkOuter);
}

#[no_mangle]
pub unsafe extern "system" fn DirectInputCreateW(
    hinst: HINSTANCE,
    dwVersion: DWORD,
    ppDI: *mut LPDIRECTINPUTW,
    punkOuter: LPUNKNOWN,
) -> HRESULT {
    return DirectInputCreateEx(
        hinst,
        dwVersion,
        core::ptr::addr_of!(IID_IDirectInput7W),
        ppDI as *mut LPVOID,
        punkOuter,
    );
}

unsafe extern "C" fn _dump_DIDEVTYPE_value(mut dwDevType: DWORD, mut dwVersion: DWORD) -> *const i8 {
    if dwVersion < 0x0800 {
        match dwDevType {
            0 => return b"All devices\0".as_ptr() as *const i8,
            DIDEVTYPE_MOUSE => return b"DIDEVTYPE_MOUSE\0".as_ptr() as *const i8,
            DIDEVTYPE_KEYBOARD => return b"DIDEVTYPE_KEYBOARD\0".as_ptr() as *const i8,
            DIDEVTYPE_JOYSTICK => return b"DIDEVTYPE_JOYSTICK\0".as_ptr() as *const i8,
            DIDEVTYPE_DEVICE => return b"DIDEVTYPE_DEVICE\0".as_ptr() as *const i8,
            _ => return b"Unknown\0".as_ptr() as *const i8,
        }
    } else {
        match dwDevType {
            DI8DEVCLASS_ALL => return b"All devices\0".as_ptr() as *const i8,
            DI8DEVCLASS_POINTER => return b"DI8DEVCLASS_POINTER\0".as_ptr() as *const i8,
            DI8DEVCLASS_KEYBOARD => return b"DI8DEVCLASS_KEYBOARD\0".as_ptr() as *const i8,
            DI8DEVCLASS_DEVICE => return b"DI8DEVCLASS_DEVICE\0".as_ptr() as *const i8,
            DI8DEVCLASS_GAMECTRL => return b"DI8DEVCLASS_GAMECTRL\0".as_ptr() as *const i8,
            _ => return b"Unknown\0".as_ptr() as *const i8,
        }
    }
}

unsafe extern "C" fn _dump_EnumDevices_dwFlags(dwFlags: DWORD) {
    if TRACE_ON!(__wine_dbch_dinput) != 0 {
        let mut i: core::ffi::c_uint = 0;
        static mut flags: [C2RustUnnamed_13; 6] = [
            C2RustUnnamed_13 {
                mask: DIEDFL_ALLDEVICES as DWORD,
                name: b"DIEDFL_ALLDEVICES\0" as *const u8 as *const core::ffi::c_char,
            },
            C2RustUnnamed_13 {
                mask: DIEDFL_ATTACHEDONLY as DWORD,
                name: b"DIEDFL_ATTACHEDONLY\0" as *const u8 as *const core::ffi::c_char,
            },
            C2RustUnnamed_13 {
                mask: DIEDFL_FORCEFEEDBACK as DWORD,
                name: b"DIEDFL_FORCEFEEDBACK\0" as *const u8 as *const core::ffi::c_char,
            },
            C2RustUnnamed_13 {
                mask: DIEDFL_INCLUDEALIASES as DWORD,
                name: b"DIEDFL_INCLUDEALIASES\0" as *const u8 as *const core::ffi::c_char,
            },
            C2RustUnnamed_13 {
                mask: DIEDFL_INCLUDEPHANTOMS as DWORD,
                name: b"DIEDFL_INCLUDEPHANTOMS\0" as *const u8 as *const core::ffi::c_char,
            },
            C2RustUnnamed_13 {
                mask: 0x40000 as DWORD,
                name: b"DIEDFL_INCLUDEHIDDEN\0" as *const u8 as *const core::ffi::c_char,
            },
        ];

        if dwFlags == 0 as DWORD {
            return;
        }
        i = 0 as core::ffi::c_uint;
        while (i as usize) < ARRAY_SIZE!(flags) as usize {
            if flags[i as usize].mask & dwFlags != 0 {
                TRACE!("%s ", flags[i as usize].name);
            }
            i = i.wrapping_add(1);
        }
    }
}

unsafe extern "C" fn dump_semantic(semantic: DWORD) -> *const core::ffi::c_char {
    if (semantic & 0xff000000) == 0xff000000 {
        return b"Any AXIS\0".as_ptr() as *const core::ffi::c_char;
    } else if (semantic & 0x82000000) == 0x82000000 {
        return b"Mouse\0".as_ptr() as *const core::ffi::c_char;
    } else if (semantic & 0x81000000) == 0x81000000 {
        return b"Keybaord\0".as_ptr() as *const core::ffi::c_char;
    } else if (semantic & DIVIRTUAL_FLYING_HELICOPTER) == DIVIRTUAL_FLYING_HELICOPTER {
        return b"Helicopter\0".as_ptr() as *const core::ffi::c_char;
    }

    return b"Unknown\0".as_ptr() as *const core::ffi::c_char;
}

unsafe extern "C" fn _dump_diactionformatA(mut lpdiActionFormat: LPDIACTIONFORMATA) {
    let mut i: ::core::ffi::c_uint = 0;
    WINE_TRACE!("diaf.dwSize = %d\n", (*lpdiActionFormat).dwSize);
    WINE_TRACE!("diaf.dwActionSize = %d\n", (*lpdiActionFormat).dwActionSize);
    WINE_TRACE!("diaf.dwDataSize = %d\n", (*lpdiActionFormat).dwDataSize);
    WINE_TRACE!("diaf.dwNumActions = %d\n", (*lpdiActionFormat).dwNumActions);
    WINE_TRACE!("diaf.rgoAction = %p\n", (*lpdiActionFormat).rgoAction);
    WINE_TRACE!("diaf.guidActionMap = %s\n", debugstr_guid(&raw mut (*lpdiActionFormat).guidActionMap));
    WINE_TRACE!("diaf.dwGenre = 0x%08x\n", (*lpdiActionFormat).dwGenre);
    WINE_TRACE!("diaf.dwBufferSize = %d\n", (*lpdiActionFormat).dwBufferSize);
    WINE_TRACE!("diaf.lAxisMin = %d\n", (*lpdiActionFormat).lAxisMin);
    WINE_TRACE!("diaf.lAxisMax = %d\n", (*lpdiActionFormat).lAxisMax);
    WINE_TRACE!("diaf.hInstString = %p\n", (*lpdiActionFormat).hInstString);
    WINE_TRACE!("diaf.ftTimeStamp ...\n");
    WINE_TRACE!("diaf.dwCRC = 0x%x\n", (*lpdiActionFormat).dwCRC);
    WINE_TRACE!("diaf.tszActionMap = %s\n", debugstr_a(&raw mut (*lpdiActionFormat).tszActionMap as *mut CHAR));
    i = 0 as ::core::ffi::c_uint;
    while (i as DWORD) < (*lpdiActionFormat).dwNumActions {
        WINE_TRACE!("diaf.rgoAction[%u]:\n", i);
        WINE_TRACE!("\tuAppData=0x%lx\n", (*(*lpdiActionFormat).rgoAction.offset(i as isize)).uAppData);
        WINE_TRACE!("\tdwSemantic=0x%08x (%s)\n", (*(*lpdiActionFormat).rgoAction.offset(i as isize)).dwSemantic, dump_semantic( (*(*lpdiActionFormat).rgoAction.offset(i as isize)).dwSemantic, ));
        WINE_TRACE!("\tdwFlags=0x%x\n", (*(*lpdiActionFormat).rgoAction.offset(i as isize)).dwFlags);
        WINE_TRACE!("\tszActionName=%s\n", debugstr_a( (*(*lpdiActionFormat).rgoAction.offset(i as isize)).u.lptszActionName as *const ::core::ffi::c_char, ));
        WINE_TRACE!("\tguidInstance=%s\n", debugstr_guid( &raw mut (*(*lpdiActionFormat).rgoAction.offset(i as isize)) .guidInstance, ));
        WINE_TRACE!("\tdwObjID=0x%x\n", (*(*lpdiActionFormat).rgoAction.offset(i as isize)).dwObjID);
        WINE_TRACE!("\tdwHow=0x%x\n", (*(*lpdiActionFormat).rgoAction.offset(i as isize)).dwHow);
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn _copy_diactionformatAtoW(mut to: LPDIACTIONFORMATW, mut from: LPDIACTIONFORMATA) {
    let mut i: ::core::ffi::c_int = 0;
    (*to).dwSize = ::core::mem::size_of::<DIACTIONFORMATW>() as DWORD;
    (*to).dwActionSize = ::core::mem::size_of::<DIACTIONW>() as DWORD;
    (*to).dwDataSize = (*from).dwDataSize;
    (*to).dwNumActions = (*from).dwNumActions;
    (*to).guidActionMap = (*from).guidActionMap;
    (*to).dwGenre = (*from).dwGenre;
    (*to).dwBufferSize = (*from).dwBufferSize;
    (*to).lAxisMin = (*from).lAxisMin;
    (*to).lAxisMax = (*from).lAxisMax;
    (*to).dwCRC = (*from).dwCRC;
    (*to).ftTimeStamp = (*from).ftTimeStamp;
    while (i as DWORD) < (*to).dwNumActions {
        (*(*to).rgoAction.offset(i as isize)).uAppData =
            (*(*from).rgoAction.offset(i as isize)).uAppData;
        (*(*to).rgoAction.offset(i as isize)).dwSemantic =
            (*(*from).rgoAction.offset(i as isize)).dwSemantic;
        (*(*to).rgoAction.offset(i as isize)).dwFlags =
            (*(*from).rgoAction.offset(i as isize)).dwFlags;
        (*(*to).rgoAction.offset(i as isize)).guidInstance =
            (*(*from).rgoAction.offset(i as isize)).guidInstance;
        (*(*to).rgoAction.offset(i as isize)).dwObjID =
            (*(*from).rgoAction.offset(i as isize)).dwObjID;
        (*(*to).rgoAction.offset(i as isize)).dwHow =
            (*(*from).rgoAction.offset(i as isize)).dwHow;
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn _copy_diactionformatWtoA(mut to: LPDIACTIONFORMATA, mut from: LPDIACTIONFORMATW) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    (*to).dwSize = ::core::mem::size_of::<DIACTIONFORMATA>() as DWORD;
    (*to).dwActionSize = ::core::mem::size_of::<DIACTIONA>() as DWORD;
    (*to).dwDataSize = (*from).dwDataSize;
    (*to).dwNumActions = (*from).dwNumActions;
    (*to).guidActionMap = (*from).guidActionMap;
    (*to).dwGenre = (*from).dwGenre;
    (*to).dwBufferSize = (*from).dwBufferSize;
    (*to).lAxisMin = (*from).lAxisMin;
    (*to).lAxisMax = (*from).lAxisMax;
    (*to).dwCRC = (*from).dwCRC;
    (*to).ftTimeStamp = (*from).ftTimeStamp;
    while (i as DWORD) < (*to).dwNumActions {
        (*(*to).rgoAction.offset(i as isize)).uAppData =
            (*(*from).rgoAction.offset(i as isize)).uAppData;
        (*(*to).rgoAction.offset(i as isize)).dwSemantic =
            (*(*from).rgoAction.offset(i as isize)).dwSemantic;
        (*(*to).rgoAction.offset(i as isize)).dwFlags =
            (*(*from).rgoAction.offset(i as isize)).dwFlags;
        (*(*to).rgoAction.offset(i as isize)).guidInstance =
            (*(*from).rgoAction.offset(i as isize)).guidInstance;
        (*(*to).rgoAction.offset(i as isize)).dwObjID =
            (*(*from).rgoAction.offset(i as isize)).dwObjID;
        (*(*to).rgoAction.offset(i as isize)).dwHow =
            (*(*from).rgoAction.offset(i as isize)).dwHow;
        i += 1;
    }
}

unsafe extern "C" fn diactionformat_priorityA(lpdiaf: LPDIACTIONFORMATA, genre: DWORD) -> DWORD {
    let mut i: ::core::ffi::c_int = 0;
    let mut priorityFlags: DWORD = 0;

    /* If there's at least one action for the device it's priority 1 */
    while (i as DWORD) < (*lpdiaf).dwNumActions {
        if ((*(*lpdiaf).rgoAction.offset(i as isize)).dwSemantic & genre) == genre {
            priorityFlags |= DIEDBS_MAPPEDPRI1 as DWORD;
        }
        i += 1;
    }

    return priorityFlags;
}

unsafe extern "C" fn diactionformat_priorityW(lpdiaf: LPDIACTIONFORMATW, genre: DWORD) -> DWORD {
    let mut i: ::core::ffi::c_int;
    let mut priorityFlags: DWORD = 0 as DWORD;

    /* If there's at least one action for the device it's priority 1 */
    i = 0 as ::core::ffi::c_int;
    while (i as DWORD) < (*lpdiaf).dwNumActions {
        if ((*(*lpdiaf).rgoAction.offset(i as isize)).dwSemantic & genre) == genre {
            priorityFlags |= DIEDBS_MAPPEDPRI1 as DWORD;
        }
        i += 1;
    }

    return priorityFlags;
}
unsafe extern "system" fn IDirectInputAImpl_EnumDevices(
    mut iface: LPDIRECTINPUT7A,
    mut dwDevType: DWORD,
    mut lpCallback: LPDIENUMDEVICESCALLBACKA,
    mut pvRef: LPVOID,
    mut dwFlags: DWORD,
) -> HRESULT {
    let mut This: *mut IDirectInputImpl = impl_from_IDirectInput7A(iface as *mut IDirectInput7A);
    let mut devInstance: DIDEVICEINSTANCEA = DIDEVICEINSTANCEA {
        dwSize: 0,
        guidInstance: _GUID {
            Data1: 0,
            Data2: 0,
            Data3: 0,
            Data4: [0; 8],
        },
        guidProduct: _GUID {
            Data1: 0,
            Data2: 0,
            Data3: 0,
            Data4: [0; 8],
        },
        dwDevType: 0,
        tszInstanceName: [0; 260],
        tszProductName: [0; 260],
        guidFFDriver: _GUID {
            Data1: 0,
            Data2: 0,
            Data3: 0,
            Data4: [0; 8],
        },
        wUsagePage: 0,
        wUsage: 0,
    };
    let mut i: ::core::ffi::c_uint = 0;
    let mut j: ::core::ffi::c_int = 0;
    let mut r: HRESULT = 0;
    WINE_TRACE!("(this=%p,0x%04x '%s',%p,%p,0x%04x)\n", This, dwDevType, _dump_DIDEVTYPE_value(dwDevType, (*This).dwVersion), lpCallback, pvRef, dwFlags);
    _dump_EnumDevices_dwFlags(dwFlags);
    if lpCallback.is_none()
        || dwFlags
            & !(DIEDFL_ATTACHEDONLY
                | DIEDFL_FORCEFEEDBACK
                | DIEDFL_INCLUDEALIASES
                | DIEDFL_INCLUDEPHANTOMS
                | DIEDFL_INCLUDEHIDDEN) as DWORD
            != 0
        || dwDevType > DI8DEVCLASS_GAMECTRL as DWORD && dwDevType < DI8DEVTYPE_DEVICE as DWORD
        || dwDevType > DI8DEVTYPE_SUPPLEMENTAL as DWORD
    {
        return DIERR_INVALIDPARAM;
    }
    if (*This).initialized == 0 {
        return DIERR_NOTINITIALIZED;
    }
    i = 0 as ::core::ffi::c_uint;
    while (i as usize) < ARRAY_SIZE!(dinput_devices) as usize {
        if !(*dinput_devices[i as usize]).enum_deviceA.is_none() {
            WINE_TRACE!(" Checking device %u ('%s')\n", i, (*dinput_devices[i as usize]).name);
            j = 0 as ::core::ffi::c_int;
            r = S_OK as HRESULT;
            while r >= 0 as HRESULT {
                devInstance.dwSize = ::core::mem::size_of::<DIDEVICEINSTANCEA>() as DWORD;
                r = (*dinput_devices[i as usize])
                    .enum_deviceA
                    .unwrap_unchecked()(
                    dwDevType,
                    dwFlags,
                    &raw mut devInstance,
                    (*This).dwVersion,
                    j,
                );
                if r == S_OK as HRESULT {
                    if enum_callback_wrapper!(
                        lpCallback.unwrap_unchecked(),
                        &raw mut devInstance,
                        pvRef,
                    ) == DIENUM_STOP as BOOL
                    {
                        return S_OK;
                    }
                }
                j += 1;
            }
        }
        i = i.wrapping_add(1);
    }
    return S_OK;
}

unsafe extern "system" fn IDirectInputWImpl_EnumDevices(
    mut iface: LPDIRECTINPUT7W,
    mut dwDevType: DWORD,
    mut lpCallback: LPDIENUMDEVICESCALLBACKW,
    mut pvRef: LPVOID,
    mut dwFlags: DWORD,
) -> HRESULT {
    let mut This: *mut IDirectInputImpl = impl_from_IDirectInput7W(iface as *mut IDirectInput7W);
    let mut devInstance: DIDEVICEINSTANCEW = core::mem::zeroed();
    let mut i: ::core::ffi::c_uint = 0;
    let mut j: ::core::ffi::c_int = 0;
    let mut r: HRESULT = 0;
    WINE_TRACE!("(this=%p,0x%04x '%s',%p,%p,0x%04x)\n", This, dwDevType, _dump_DIDEVTYPE_value(dwDevType, (*This).dwVersion), lpCallback, pvRef, dwFlags);
    _dump_EnumDevices_dwFlags(dwFlags);

    if lpCallback.is_none()
        || (dwFlags
            & !(DIEDFL_ATTACHEDONLY | DIEDFL_FORCEFEEDBACK | DIEDFL_INCLUDEALIASES | DIEDFL_INCLUDEPHANTOMS | DIEDFL_INCLUDEHIDDEN))
            != 0
        || (dwDevType > DI8DEVCLASS_GAMECTRL && dwDevType < DI8DEVTYPE_DEVICE)
        || dwDevType > DI8DEVTYPE_SUPPLEMENTAL
    {
        return DIERR_INVALIDPARAM;
    }
    if (*This).initialized == 0 {
        return DIERR_NOTINITIALIZED;
    }
    i = 0 as ::core::ffi::c_uint;
    while (i as usize) < ARRAY_SIZE!(dinput_devices) as usize {
        'body: {
            if (*dinput_devices[i as usize]).enum_deviceW.is_none() {
                break 'body;
            }
            j = 0 as ::core::ffi::c_int;
            r = S_OK as HRESULT;
            while r >= 0 as HRESULT {
                devInstance.dwSize = ::core::mem::size_of::<DIDEVICEINSTANCEW>() as DWORD;
                WINE_TRACE!("  - checking device %u ('%s')\n", i, (*dinput_devices[i as usize]).name);
                r = (*dinput_devices[i as usize])
                    .enum_deviceW
                    .unwrap_unchecked()(
                    dwDevType,
                    dwFlags,
                    core::ptr::addr_of_mut!(devInstance),
                    (*This).dwVersion,
                    j,
                );
                if r == S_OK as HRESULT {
                    if enum_callback_wrapper!(
                        lpCallback.unwrap_unchecked(),
                        core::ptr::addr_of_mut!(devInstance),
                        pvRef,
                    ) == DIENUM_STOP as BOOL
                    {
                        return S_OK as HRESULT;
                    }
                }
                j += 1;
            }
        }
        i = i.wrapping_add(1);
    }

    return S_OK as HRESULT;
}

unsafe extern "system" fn IDirectInputAImpl_AddRef(iface: LPDIRECTINPUT7A) -> ULONG {
    let mut This: *mut IDirectInputImpl = impl_from_IDirectInput7A(iface as *mut IDirectInput7A);
    let mut ref_0: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_) as *mut LONG) as ULONG;

    return ref_0;
}

unsafe extern "system" fn IDirectInputWImpl_AddRef(iface: LPDIRECTINPUT7W) -> ULONG {
    let mut This: *mut IDirectInputImpl = impl_from_IDirectInput7W(iface as *mut IDirectInput7W);
    return IDirectInputAImpl_AddRef(core::ptr::addr_of_mut!((*This).IDirectInput7A_iface));
}

unsafe extern "system" fn IDirectInputAImpl_Release(iface: LPDIRECTINPUT7A) -> ULONG {
    let mut This: *mut IDirectInputImpl = impl_from_IDirectInput7A(iface as *mut IDirectInput7A);
    let mut ref_0: ULONG = InterlockedDecrement(&raw mut (*This).ref_ as *mut ::core::ffi::c_long) as ULONG;
    if ref_0 == 0 as ULONG {
        uninitialize_directinput_instance(This);
        HeapFree(GetProcessHeap(), 0 as DWORD, This as PVOID);
    }
    return ref_0;
}

unsafe extern "system" fn IDirectInputWImpl_Release(iface: LPDIRECTINPUT7W) -> ULONG {
    let mut This: *mut IDirectInputImpl = impl_from_IDirectInput7W(iface as *mut IDirectInput7W);
    return IDirectInputAImpl_Release(core::ptr::addr_of_mut!((*This).IDirectInput7A_iface));
}

unsafe extern "system" fn IDirectInputAImpl_QueryInterface(iface: LPDIRECTINPUT7A, riid: REFIID, ppobj: *mut LPVOID) -> HRESULT {
    let This: *mut IDirectInputImpl = impl_from_IDirectInput7A(iface);

    if riid.is_null() || ppobj.is_null() {
        return E_POINTER;
    }

    *ppobj = core::ptr::null_mut();

    if IsEqualGUID!(core::ptr::addr_of!(IID_IUnknown), riid) != false
        || IsEqualGUID!(core::ptr::addr_of!(IID_IDirectInputA), riid) != false
        || IsEqualGUID!(core::ptr::addr_of!(IID_IDirectInput2A), riid) != false
        || IsEqualGUID!(core::ptr::addr_of!(IID_IDirectInput7A), riid) != false
    {
        *ppobj = core::ptr::addr_of_mut!((*This).IDirectInput7A_iface) as LPVOID;
    } else if IsEqualGUID!(core::ptr::addr_of!(IID_IDirectInputW), riid) != false
        || IsEqualGUID!(core::ptr::addr_of!(IID_IDirectInput2W), riid) != false
        || IsEqualGUID!(core::ptr::addr_of!(IID_IDirectInput7W), riid) != false
    {
        *ppobj = core::ptr::addr_of_mut!((*This).IDirectInput7W_iface) as LPVOID;
    }

    if IsEqualGUID!(core::ptr::addr_of!(IID_IDirectInputJoyConfig8), riid) != false {
        *ppobj = core::ptr::addr_of_mut!((*This).IDirectInputJoyConfig8_iface) as LPVOID;
    }

    if !(*ppobj).is_null() {
        IUnknown_AddRef((*ppobj) as *mut IUnknown);
        return DI_OK;
    }

    WARN!("Unsupported interface: %s\n", debugstr_guid(riid));
    return E_NOINTERFACE;
}
unsafe extern "system" fn IDirectInputWImpl_QueryInterface(
    iface: LPDIRECTINPUT7W,
    riid: REFIID,
    ppobj: *mut LPVOID,
) -> HRESULT {
    let mut This: *mut IDirectInputImpl = impl_from_IDirectInput7W(iface as *mut IDirectInput7W);
    return IDirectInputAImpl_QueryInterface(core::ptr::addr_of_mut!((*This).IDirectInput7A_iface), riid, ppobj);
}

unsafe extern "C" fn initialize_directinput_instance(This: *mut IDirectInputImpl, dwVersion: DWORD) -> HRESULT {
    if (*This).initialized == 0 {
        (*This).dwVersion = dwVersion;
        (*This).evsequence = 1;

        InitializeCriticalSection(core::ptr::addr_of_mut!((*This).crit));
        (*(*This).crit.DebugInfo).__bindgen_anon_1.Spare[0 as usize] = b"__FILE__: IDirectInputImpl*->crit\0".as_ptr() as *const ::core::ffi::c_char as DWORD_PTR;

        list_init(core::ptr::addr_of_mut!((*This).devices_list));
        list_init(core::ptr::addr_of_mut!((*This).device_players));

        /* Add self to the list of the IDirectInputs */
        EnterCriticalSection(core::ptr::addr_of_mut!(dinput_hook_crit));
        list_add_head((core::ptr::addr_of_mut!(direct_input_list)).cast(), core::ptr::addr_of_mut!((*This).entry));
        LeaveCriticalSection(core::ptr::addr_of_mut!(dinput_hook_crit));

        (*This).initialized = (TRUE as i32);

        if check_hook_thread() == 0 {
            uninitialize_directinput_instance(This);
            return DIERR_GENERIC;
        }
    }

    return DI_OK;
}

unsafe extern "C" fn uninitialize_directinput_instance(This: *mut IDirectInputImpl) {
    if (*This).initialized != 0 {
        let mut device_player: *mut DevicePlayer = core::ptr::null_mut::<DevicePlayer>();
        let mut device_player2: *mut DevicePlayer = core::ptr::null_mut::<DevicePlayer>();
        /* Remove self from the list of the IDirectInputs */
        EnterCriticalSection(core::ptr::addr_of_mut!(dinput_hook_crit));
        list_remove(core::ptr::addr_of_mut!((*This).entry));
        LeaveCriticalSection(core::ptr::addr_of_mut!(dinput_hook_crit));

        /* LIST_FOR_EACH_ENTRY_SAFE( device_player, device_player2, &This->device_players, struct DevicePlayer, entry ) */
        device_player = ((*This).device_players.next as *mut u8)
            .offset(-(core::mem::offset_of!(DevicePlayer, entry) as isize))
            as *mut DevicePlayer;
        device_player2 = ((*device_player).entry.next as *mut u8)
            .offset(-(core::mem::offset_of!(DevicePlayer, entry) as isize))
            as *mut DevicePlayer;
        while core::ptr::addr_of_mut!((*device_player).entry)
            != core::ptr::addr_of_mut!((*This).device_players)
        {
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                device_player as PVOID,
            );
            device_player = device_player2;
            device_player2 = ((*device_player).entry.next as *mut u8)
                .offset(-(core::mem::offset_of!(DevicePlayer, entry) as isize))
                as *mut DevicePlayer;
        }

        check_hook_thread();

        (*(*This).crit.DebugInfo).__bindgen_anon_1.Spare[0 as usize] =
            0 as DWORD_PTR;
        DeleteCriticalSection(core::ptr::addr_of_mut!((*This).crit));

        (*This).initialized = FALSE as BOOL;
    }
}

unsafe extern "system" fn IDirectInputAImpl_Initialize(iface: LPDIRECTINPUT7A, hinst: HINSTANCE, version: DWORD) -> HRESULT {
    let mut This: *mut IDirectInputImpl = impl_from_IDirectInput7A(iface);

    if hinst.is_null() {
        return DIERR_INVALIDPARAM;
    } else if version == 0 {
        return DIERR_NOTINITIALIZED;
    } else if version > (DIRECTINPUT_VERSION_700 as DWORD) {
        return DIERR_OLDDIRECTINPUTVERSION;
    } else if version != (DIRECTINPUT_VERSION_300 as DWORD) && version != (DIRECTINPUT_VERSION_500 as DWORD) &&
             version != (DIRECTINPUT_VERSION_50A as DWORD) && version != (DIRECTINPUT_VERSION_5B2 as DWORD) &&
             version != (DIRECTINPUT_VERSION_602 as DWORD) && version != (DIRECTINPUT_VERSION_61A as DWORD) &&
             version != (DIRECTINPUT_VERSION_700 as DWORD) && version != DIRECTINPUT_VERSION {
        return DIERR_BETADIRECTINPUTVERSION;
    }

    return initialize_directinput_instance(This, version);
}

unsafe extern "system" fn IDirectInputWImpl_Initialize(iface: LPDIRECTINPUT7W, hinst: HINSTANCE, x: DWORD) -> HRESULT {
    let mut This: *mut IDirectInputImpl = impl_from_IDirectInput7W(iface as *mut IDirectInput7W);
    return IDirectInputAImpl_Initialize(&raw mut (*This).IDirectInput7A_iface, hinst, x);
}
unsafe extern "system" fn IDirectInputAImpl_GetDeviceStatus(iface: LPDIRECTINPUT7A, rguid: REFGUID) -> HRESULT {
    let mut This: *mut IDirectInputImpl = impl_from_IDirectInput7A(iface as *mut IDirectInput7A);
    let mut hr: HRESULT = 0;
    let mut device: LPDIRECTINPUTDEVICEA = core::ptr::null_mut();
    WINE_TRACE!("(%p)->(%s)\n", This, debugstr_guid(rguid as *const _GUID));
    if rguid.is_null() {
        return E_POINTER;
    }
    if (*This).initialized == 0 {
        return DIERR_NOTINITIALIZED;
    }
    hr = IDirectInput_CreateDevice!(
        iface,
        rguid,
        core::ptr::addr_of_mut!(device),
        core::ptr::null_mut(),
    );
    if hr != DI_OK {
        return DI_NOTATTACHED;
    }
    IUnknown_Release((device).cast());
    return DI_OK;
}

unsafe extern "system" fn IDirectInputWImpl_GetDeviceStatus(iface: LPDIRECTINPUT7W, rguid: REFGUID) -> HRESULT {
    let mut This: *mut IDirectInputImpl = impl_from_IDirectInput7W(iface as *mut IDirectInput7W);
    return IDirectInputAImpl_GetDeviceStatus(core::ptr::addr_of_mut!((*This).IDirectInput7A_iface), rguid);
}

unsafe extern "system" fn IDirectInputAImpl_RunControlPanel(
    mut iface: LPDIRECTINPUT7A,
    mut hwndOwner: HWND,
    mut dwFlags: DWORD,
) -> HRESULT {
    let mut control_exeW: [WCHAR; 12] = [
    0x0063u16, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x002E, 0x0065, 0x0078,
    0x0065, 0x0000,
];
    let mut si: STARTUPINFOW = core::mem::zeroed();
    let mut pi: PROCESS_INFORMATION = core::mem::zeroed();
    let mut This: *mut IDirectInputImpl = impl_from_IDirectInput7A(iface as *mut IDirectInput7A);
    WINE_TRACE!("(%p)->(%p, %08x)\n", This, hwndOwner, dwFlags);
    if !hwndOwner.is_null() && IsWindow(hwndOwner) == 0 {
        return E_HANDLE;
    }
    if dwFlags != 0 {
        return DIERR_INVALIDPARAM;
    }
    if (*This).initialized == 0 {
        return DIERR_NOTINITIALIZED;
    }
    if CreateProcessW(
        core::ptr::null_mut(),
        &raw mut control_exeW as LPWSTR,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        (FALSE) as BOOL,
        DETACHED_PROCESS,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &raw mut si,
        &raw mut pi,
    ) == 0
    {
        return HRESULT_FROM_WIN32!(GetLastError());
    }
    return DI_OK;
}

unsafe extern "system" fn IDirectInputWImpl_RunControlPanel(iface: LPDIRECTINPUT7W, hwndOwner: HWND, dwFlags: DWORD) -> HRESULT {
    let mut This: *mut IDirectInputImpl = impl_from_IDirectInput7W(iface as *mut IDirectInput7W);
    return IDirectInputAImpl_RunControlPanel(core::ptr::addr_of_mut!((*This).IDirectInput7A_iface) as *mut _, hwndOwner, dwFlags);
}
unsafe extern "system" fn IDirectInput2AImpl_FindDevice(
    mut iface: LPDIRECTINPUT7A,
    rguid: *const GUID,
    mut pszName: LPCSTR,
    mut pguidInstance: LPGUID,
) -> HRESULT {
    let mut This: *mut IDirectInputImpl = impl_from_IDirectInput7A(iface as *mut IDirectInput7A);
    WINE_FIXME!("(%p)->(%s, %s, %p): stub\n", This, debugstr_guid(rguid as *const GUID), pszName, pguidInstance);
    return DI_OK;
}

unsafe extern "system" fn IDirectInput2WImpl_FindDevice(
    iface: LPDIRECTINPUT7W,
    rguid: REFGUID,
    pszName: LPCWSTR,
    pguidInstance: LPGUID,
) -> HRESULT {
    let mut This: *mut IDirectInputImpl = impl_from_IDirectInput7W(iface);
    WINE_FIXME!("(%p)->(%s, %s, %p): stub\n", This, debugstr_guid(rguid as *const _GUID), debugstr_w(pszName as *const WCHAR), pguidInstance);
    return DI_OK;
}

unsafe extern "C" fn create_device(This: *mut IDirectInputImpl, rguid: REFGUID, riid: REFIID, pvOut: *mut LPVOID, unicode: BOOL) -> HRESULT {
    let mut i: ::core::ffi::c_uint;

    if !pvOut.is_null() {
        *pvOut = core::ptr::null_mut();
    }

    if rguid.is_null() || pvOut.is_null() {
        return E_POINTER;
    }

    if (*This).initialized == 0 {
        return DIERR_NOTINITIALIZED;
    }

    /* Loop on all the devices to see if anyone matches the given GUID */
    i = 0;
    while (i as usize) < (ARRAY_SIZE!(dinput_devices) as usize) {
        let mut ret: HRESULT;

        if !(*dinput_devices[i as usize]).create_device.is_none() {
            ret = (*dinput_devices[i as usize])
                .create_device
                .unwrap_unchecked()(This, rguid, riid, pvOut, unicode);
            if ret == DI_OK {
                return DI_OK;
            }
        }
        i = i.wrapping_add(1);
    }

    WARN!("invalid device GUID %s\n", debugstr_guid(rguid));
    return DIERR_DEVICENOTREG;
}

unsafe extern "system" fn IDirectInput7AImpl_CreateDeviceEx(iface: LPDIRECTINPUT7A, rguid: REFGUID, riid: REFIID, pvOut: *mut LPVOID, lpUnknownOuter: LPUNKNOWN) -> HRESULT {
    let mut This: *mut IDirectInputImpl = impl_from_IDirectInput7A(iface as *mut IDirectInput7A);

    return create_device(This, rguid, riid, pvOut, (FALSE) as BOOL);
}
unsafe extern "system" fn IDirectInput7WImpl_CreateDeviceEx(
    iface: LPDIRECTINPUT7W,
    rguid: *const GUID,
    riid: *const IID,
    pvOut: *mut LPVOID,
    lpUnknownOuter: LPUNKNOWN,
) -> HRESULT {
    let This: *mut IDirectInputImpl = impl_from_IDirectInput7W(iface as *mut IDirectInput7W);
    let _ = lpUnknownOuter;
    return create_device(This, rguid, riid, pvOut, (TRUE) as BOOL);
}

unsafe extern "system" fn IDirectInputAImpl_CreateDevice(iface: LPDIRECTINPUT7A, rguid: REFGUID, pdev: *mut LPDIRECTINPUTDEVICEA, punk: LPUNKNOWN) -> HRESULT {
    return IDirectInput7AImpl_CreateDeviceEx(iface, rguid, core::ptr::null_mut(), pdev as *mut LPVOID, punk);
}
unsafe extern "system" fn IDirectInputWImpl_CreateDevice(
    iface: LPDIRECTINPUT7W,
    rguid: REFGUID,
    pdev: *mut LPDIRECTINPUTDEVICEW,
    punk: LPUNKNOWN,
) -> HRESULT {
    return IDirectInput7WImpl_CreateDeviceEx(
        iface,
        rguid,
        core::ptr::null_mut(),
        pdev as *mut LPVOID,
        punk,
    );
}
unsafe extern "system" fn IDirectInput8AImpl_AddRef(iface: LPDIRECTINPUT8A) -> ULONG {
    let mut This: *mut IDirectInputImpl = impl_from_IDirectInput8A(iface as *mut IDirectInput8A);
    return IDirectInputAImpl_AddRef(core::ptr::addr_of_mut!((*This).IDirectInput7A_iface));
}

unsafe extern "system" fn IDirectInput8WImpl_AddRef(iface: LPDIRECTINPUT8W) -> ULONG {
    let mut This: *mut IDirectInputImpl = impl_from_IDirectInput8W(iface as *mut IDirectInput8W);
    return IDirectInputAImpl_AddRef(core::ptr::addr_of_mut!((*This).IDirectInput7A_iface));
}
unsafe extern "system" fn IDirectInput8AImpl_QueryInterface(
    mut iface: LPDIRECTINPUT8A,
    riid: *const IID,
    mut ppobj: *mut LPVOID,
) -> HRESULT {
    let mut This: *mut IDirectInputImpl = impl_from_IDirectInput8A(iface as *mut IDirectInput8A);
    return IDirectInputAImpl_QueryInterface(&raw mut (*This).IDirectInput7A_iface, riid, ppobj);
}
unsafe extern "system" fn IDirectInput8WImpl_QueryInterface(iface: LPDIRECTINPUT8W, riid: REFIID, ppobj: *mut LPVOID) -> HRESULT {
    let mut This: *mut IDirectInputImpl = impl_from_IDirectInput8W(iface as *mut IDirectInput8W);
    return IDirectInputAImpl_QueryInterface(core::ptr::addr_of_mut!((*This).IDirectInput7A_iface), riid, ppobj);
}
unsafe extern "system" fn IDirectInput8AImpl_Release(iface: LPDIRECTINPUT8A) -> ULONG {
    let This: *mut IDirectInputImpl = impl_from_IDirectInput8A(iface as *mut IDirectInput8A);
    return IDirectInputAImpl_Release(core::ptr::addr_of_mut!((*This).IDirectInput7A_iface));
}

unsafe extern "system" fn IDirectInput8WImpl_Release(iface: LPDIRECTINPUT8W) -> ULONG {
    let This: *mut IDirectInputImpl = impl_from_IDirectInput8W(iface as *mut IDirectInput8W);
    return IDirectInputAImpl_Release(core::ptr::addr_of_mut!((*This).IDirectInput7A_iface));
}
unsafe extern "system" fn IDirectInput8AImpl_CreateDevice(iface: LPDIRECTINPUT8A, rguid: *const GUID, pdev: *mut LPDIRECTINPUTDEVICE8A, punk: LPUNKNOWN) -> HRESULT {
    let mut This: *mut IDirectInputImpl = impl_from_IDirectInput8A(iface as *mut IDirectInput8A);
    return IDirectInput7AImpl_CreateDeviceEx(core::ptr::addr_of_mut!((*This).IDirectInput7A_iface), rguid, core::ptr::null_mut(), pdev as *mut LPVOID, punk);
}
unsafe extern "system" fn IDirectInput8WImpl_CreateDevice(
    mut iface: LPDIRECTINPUT8W,
    rguid: *const GUID,
    mut pdev: *mut LPDIRECTINPUTDEVICE8W,
    mut punk: LPUNKNOWN,
) -> HRESULT {
    let mut This: *mut IDirectInputImpl = impl_from_IDirectInput8W(iface as *mut IDirectInput8W);
    return IDirectInput7WImpl_CreateDeviceEx(
        core::ptr::addr_of_mut!((*This).IDirectInput7W_iface),
        rguid,
        core::ptr::null_mut(),
        pdev as *mut LPVOID,
        punk,
    );
}

unsafe extern "system" fn IDirectInput8AImpl_EnumDevices(
    iface: LPDIRECTINPUT8A,
    dwDevType: DWORD,
    lpCallback: LPDIENUMDEVICESCALLBACKA,
    pvRef: LPVOID,
    dwFlags: DWORD,
) -> HRESULT {
    let This: *mut IDirectInputImpl = impl_from_IDirectInput8A(iface as *mut IDirectInput8A);
    return IDirectInputAImpl_EnumDevices(
        &raw mut (*This).IDirectInput7A_iface,
        dwDevType,
        lpCallback,
        pvRef,
        dwFlags,
    );
}

unsafe extern "system" fn IDirectInput8WImpl_EnumDevices(iface: LPDIRECTINPUT8W, dwDevType: DWORD, lpCallback: LPDIENUMDEVICESCALLBACKW, pvRef: LPVOID, dwFlags: DWORD) -> HRESULT {
    let mut This: *mut IDirectInputImpl = impl_from_IDirectInput8W(iface as *mut IDirectInput8W);
    return IDirectInputWImpl_EnumDevices(
        core::ptr::addr_of_mut!((*This).IDirectInput7W_iface),
        dwDevType,
        lpCallback,
        pvRef,
        dwFlags,
    );
}

unsafe extern "system" fn IDirectInput8AImpl_GetDeviceStatus(iface: LPDIRECTINPUT8A, rguid: REFGUID) -> HRESULT {
    let mut This: *mut IDirectInputImpl = impl_from_IDirectInput8A(iface as *mut IDirectInput8A);
    return IDirectInputAImpl_GetDeviceStatus(core::ptr::addr_of_mut!((*This).IDirectInput7A_iface), rguid);
}

unsafe extern "system" fn IDirectInput8WImpl_GetDeviceStatus(iface: LPDIRECTINPUT8W, rguid: REFGUID) -> HRESULT {
    let mut This: *mut IDirectInputImpl = impl_from_IDirectInput8W(iface as *mut IDirectInput8W);
    return IDirectInputAImpl_GetDeviceStatus(core::ptr::addr_of_mut!((*This).IDirectInput7A_iface) as LPDIRECTINPUT7A, rguid);
}

unsafe extern "system" fn IDirectInput8AImpl_RunControlPanel(iface: LPDIRECTINPUT8A, hwndOwner: HWND, dwFlags: DWORD) -> HRESULT {
    let mut This: *mut IDirectInputImpl = impl_from_IDirectInput8A(iface as *mut IDirectInput8A);
    return IDirectInputAImpl_RunControlPanel(core::ptr::addr_of_mut!((*This).IDirectInput7A_iface), hwndOwner, dwFlags);
}
unsafe extern "system" fn IDirectInput8WImpl_RunControlPanel(iface: LPDIRECTINPUT8W, hwndOwner: HWND, dwFlags: DWORD) -> HRESULT {
    let mut This: *mut IDirectInputImpl = impl_from_IDirectInput8W(iface as *mut IDirectInput8W);
    return IDirectInputAImpl_RunControlPanel(core::ptr::addr_of_mut!((*This).IDirectInput7A_iface), hwndOwner, dwFlags);
}

unsafe extern "system" fn IDirectInput8AImpl_Initialize(iface: LPDIRECTINPUT8A, hinst: HINSTANCE, version: DWORD) -> HRESULT {
    let mut This: *mut IDirectInputImpl = impl_from_IDirectInput8A(iface as *mut IDirectInput8A);
    WINE_TRACE!("(%p)->(%p, 0x%04x)\n", This, hinst, version);
    if hinst.is_null() {
        return DIERR_INVALIDPARAM;
    } else if version == 0 {
        return DIERR_NOTINITIALIZED;
    } else if version < DIRECTINPUT_VERSION {
        return DIERR_BETADIRECTINPUTVERSION;
    } else if version > DIRECTINPUT_VERSION {
        return DIERR_OLDDIRECTINPUTVERSION;
    }
    return initialize_directinput_instance(This, version);
}

unsafe extern "system" fn IDirectInput8WImpl_Initialize(iface: LPDIRECTINPUT8W, hinst: HINSTANCE, version: DWORD) -> HRESULT {
    let mut This: *mut IDirectInputImpl = impl_from_IDirectInput8W(iface as *mut IDirectInput8W);
    return IDirectInput8AImpl_Initialize(&raw mut (*This).IDirectInput8A_iface, hinst, version);
}
unsafe extern "system" fn IDirectInput8AImpl_FindDevice(iface: LPDIRECTINPUT8A, rguid: *const GUID, pszName: LPCSTR, pguidInstance: LPGUID) -> HRESULT {
    let mut This: *mut IDirectInputImpl = impl_from_IDirectInput8A(iface as *mut IDirectInput8A);
    return IDirectInput2AImpl_FindDevice(core::ptr::addr_of_mut!((*This).IDirectInput7A_iface), rguid, pszName, pguidInstance);
}

unsafe extern "system" fn IDirectInput8WImpl_FindDevice(iface: LPDIRECTINPUT8W, rguid: REFGUID, pszName: LPCWSTR, pguidInstance: LPGUID) -> HRESULT {
    let mut This: *mut IDirectInputImpl = impl_from_IDirectInput8W(iface as *mut IDirectInput8W);
    return IDirectInput2WImpl_FindDevice(core::ptr::addr_of_mut!((*This).IDirectInput7W_iface), rguid, pszName, pguidInstance);
}

unsafe extern "C" fn should_enumerate_device(
    username: *const WCHAR,
    dwFlags: DWORD,
    device_players: *mut list,
    guid: REFGUID,
) -> BOOL {
    let mut should_enumerate: BOOL = (TRUE as i32);
    let mut device_player: *mut DevicePlayer;

    /* Check if user owns this device */
    if (dwFlags & DIEDBSFL_THISUSER) != 0 && !username.is_null() && *username != 0 {
        should_enumerate = (FALSE as i32);
        device_player = CONTAINING_RECORD!((*device_players).next, DevicePlayer, entry);
        while core::ptr::addr_of_mut!((*device_player).entry) != (device_players).cast() {
            if IsEqualGUID!(core::ptr::addr_of!((*device_player).instance_guid), guid) {
                if *(core::ptr::addr_of_mut!((*device_player).username) as *mut WCHAR) != 0
                    && lstrcmpW(
                        username,
                        core::ptr::addr_of_mut!((*device_player).username) as *mut WCHAR,
                    ) == 0
                {
                    return (TRUE) as BOOL; /* Device username matches */
                }
                break;
            }
            device_player = CONTAINING_RECORD!((*device_player).entry.next, DevicePlayer, entry);
        }
    }

    /* Check if this device is not owned by anyone */
    if (dwFlags & DIEDBSFL_AVAILABLEDEVICES) != 0 {
        let mut found: BOOL = (FALSE as i32);
        should_enumerate = (FALSE as i32);
        device_player = CONTAINING_RECORD!((*device_players).next, DevicePlayer, entry);
        while core::ptr::addr_of_mut!((*device_player).entry) != (device_players).cast() {
            if IsEqualGUID!(core::ptr::addr_of!((*device_player).instance_guid), guid) {
                if *(core::ptr::addr_of_mut!((*device_player).username) as *mut WCHAR) != 0 {
                    found = (TRUE as i32);
                }
                break;
            }
            device_player = CONTAINING_RECORD!((*device_player).entry.next, DevicePlayer, entry);
        }
        if found == 0 {
            return (TRUE) as BOOL; /* Device does not have a username */
        }
    }

    return should_enumerate;
}

unsafe extern "system" fn IDirectInput8AImpl_EnumDevicesBySemantics(
    iface: LPDIRECTINPUT8A,
    ptszUserName: LPCSTR,
    lpdiActionFormat: LPDIACTIONFORMATA,
    lpCallback: LPDIENUMDEVICESBYSEMANTICSCBA,
    pvRef: LPVOID,
    dwFlags: DWORD,
) -> HRESULT {
    static mut guids: [*const GUID; 2] = [&raw const GUID_SysKeyboard, &raw const GUID_SysMouse];
    static mut actionMasks: [DWORD; 2] = [DIKEYBOARD_MASK, DIMOUSE_MASK];
    let mut This: *mut IDirectInputImpl = impl_from_IDirectInput8A(iface as *mut IDirectInput8A);
    let mut didevi: DIDEVICEINSTANCEA = core::mem::zeroed();
    let mut lpdid: LPDIRECTINPUTDEVICE8A = core::ptr::null_mut();
    let mut callbackFlags: DWORD = 0;
    let mut i: core::ffi::c_int = 0;
    let mut j: core::ffi::c_int = 0;
    let mut device_count: core::ffi::c_int = 0;
    let mut remain: core::ffi::c_int;
    let mut didevis: *mut DIDEVICEINSTANCEA = core::ptr::null_mut();
    let mut username_w: *mut WCHAR = core::ptr::null_mut();

    WINE_FIXME!("(this=%p,%s,%p,%p,%p,%04x): semi-stub\n", This, debugstr_a(ptszUserName as *const core::ffi::c_char), lpdiActionFormat, lpCallback, pvRef, dwFlags);
    if (dwFlags & (0 as DWORD)) != 0 {
        WINE_FIXME!("\tdwFlags |= DIEDBSFL_ATTACHEDONLY\n");
    }
    if (dwFlags & (0x10 as DWORD)) != 0 {
        WINE_FIXME!("\tdwFlags |= DIEDBSFL_THISUSER\n");
    }
    if (dwFlags & (0x100 as DWORD)) != 0 {
        WINE_FIXME!("\tdwFlags |= DIEDBSFL_FORCEFEEDBACK\n");
    }
    if (dwFlags & (0x1000 as DWORD)) != 0 {
        WINE_FIXME!("\tdwFlags |= DIEDBSFL_AVAILABLEDEVICES\n");
    }
    if (dwFlags & (0x2000 as DWORD)) != 0 {
        WINE_FIXME!("\tdwFlags |= DIEDBSFL_MULTIMICEKEYBOARDS\n");
    }
    if (dwFlags & (0x4000 as DWORD)) != 0 {
        WINE_FIXME!("\tdwFlags |= DIEDBSFL_NONGAMINGDEVICES\n");
    }

    _dump_diactionformatA(lpdiActionFormat);

    didevi.dwSize = core::mem::size_of::<DIDEVICEINSTANCEA>() as DWORD;

    if !ptszUserName.is_null() {
        let len: core::ffi::c_int = MultiByteToWideChar(
            CP_ACP,
            0,
            ptszUserName,
            -1,
            core::ptr::null_mut(),
            0,
        );

        username_w = HeapAlloc(
            GetProcessHeap(),
            0,
            (core::mem::size_of::<WCHAR>() as SIZE_T).wrapping_mul(len as SIZE_T),
        ) as *mut WCHAR;
        MultiByteToWideChar(
            CP_ACP,
            0,
            ptszUserName,
            -1,
            username_w,
            len,
        );
    }

    /* Enumerate all the joysticks */
    i = 0;
    while (i as usize) < ARRAY_SIZE!(dinput_devices) as usize {
        let mut enumSuccess: HRESULT = 0;

        if !(*dinput_devices[i as usize]).enum_deviceA.is_none() {
            j = 0;
            enumSuccess = S_OK;
            while enumSuccess >= 0 {
                WINE_TRACE!(" - checking device %u ('%s')\n", i, (*dinput_devices[i as usize]).name);

                /* Default behavior is to enumerate attached game controllers */
                enumSuccess = (*dinput_devices[i as usize])
                    .enum_deviceA
                    .unwrap_unchecked()(
                    DI8DEVCLASS_GAMECTRL,
                    DIEDFL_ATTACHEDONLY | dwFlags,
                    &raw mut didevi,
                    (*This).dwVersion,
                    j,
                );
                if enumSuccess == S_OK
                    && (should_enumerate_device(
                        username_w,
                        dwFlags,
                        (&raw mut (*This).device_players).cast(),
                        &raw mut didevi.guidInstance,
                    ) != 0)
                {
                    let fresh = device_count;
                    device_count += 1;
                    if fresh != 0 {
                        didevis = HeapReAlloc(
                            GetProcessHeap(),
                            0,
                            didevis as *mut core::ffi::c_void,
                            (core::mem::size_of::<DIDEVICEINSTANCEA>() as SIZE_T)
                                .wrapping_mul(device_count as SIZE_T),
                        ) as *mut DIDEVICEINSTANCEA;
                    } else {
                        didevis = HeapAlloc(
                            GetProcessHeap(),
                            0,
                            (core::mem::size_of::<DIDEVICEINSTANCEA>() as SIZE_T)
                                .wrapping_mul(device_count as SIZE_T),
                        ) as *mut DIDEVICEINSTANCEA;
                    }
                    *didevis.offset((device_count - 1) as isize) = didevi;
                }
                j += 1;
            }
        }
        i += 1;
    }

    remain = device_count;
    /* Add keyboard and mouse to remaining device count */
    if (dwFlags & DIEDBSFL_FORCEFEEDBACK) == 0 {
        i = 0;
        while (i as usize) < ARRAY_SIZE!(guids) as usize {
            if should_enumerate_device(
                username_w,
                dwFlags,
                (&raw mut (*This).device_players).cast(),
                guids[i as usize],
            ) != 0
            {
                remain += 1;
            }
            i += 1;
        }
    }

    i = 0;
    while i < device_count {
        callbackFlags = diactionformat_priorityA(lpdiActionFormat, (*lpdiActionFormat).dwGenre);
        IDirectInput_CreateDevice!(
            iface,
            &raw mut (*didevis.offset(i as isize)).guidInstance,
            &raw mut lpdid,
            core::ptr::null_mut(),
        );

        remain -= 1;
        if (lpCallback.unwrap_unchecked())(
            didevis.offset(i as isize) as *mut DIDEVICEINSTANCEA as LPCDIDEVICEINSTANCEA,
            lpdid,
            callbackFlags,
            remain as DWORD,
            pvRef,
        ) == (DIENUM_STOP as i32)
        {
            IDirectInputDevice_Release!(lpdid);
            HeapFree(GetProcessHeap(), 0, didevis as *mut core::ffi::c_void);
            HeapFree(GetProcessHeap(), 0, username_w as *mut core::ffi::c_void);
            return DI_OK;
        }
        IDirectInputDevice_Release!(lpdid);
        i += 1;
    }

    HeapFree(GetProcessHeap(), 0, didevis as *mut core::ffi::c_void);

    if (dwFlags & DIEDBSFL_FORCEFEEDBACK) != 0 {
        HeapFree(GetProcessHeap(), 0, username_w as *mut core::ffi::c_void);
        return DI_OK;
    }

    /* Enumerate keyboard and mouse */
    i = 0;
    while (i as usize) < ARRAY_SIZE!(guids) as usize {
        if should_enumerate_device(
            username_w,
            dwFlags,
            (&raw mut (*This).device_players).cast(),
            guids[i as usize],
        ) != 0
        {
            callbackFlags = diactionformat_priorityA(lpdiActionFormat, actionMasks[i as usize]);

            IDirectInput_CreateDevice!(
                iface,
                guids[i as usize],
                &raw mut lpdid,
                core::ptr::null_mut(),
            );
            IDirectInputDevice_GetDeviceInfo!(lpdid, &raw mut didevi);

            remain -= 1;
            if (lpCallback.unwrap_unchecked())(
                &raw mut didevi as LPCDIDEVICEINSTANCEA,
                lpdid,
                callbackFlags,
                remain as DWORD,
                pvRef,
            ) == (DIENUM_STOP as i32)
            {
                IDirectInputDevice_Release!(lpdid);
                HeapFree(GetProcessHeap(), 0, username_w as *mut core::ffi::c_void);
                return DI_OK;
            }
            IDirectInputDevice_Release!(lpdid);
        }
        i += 1;
    }

    HeapFree(GetProcessHeap(), 0, username_w as *mut core::ffi::c_void);
    return DI_OK;
}

unsafe extern "system" fn IDirectInput8WImpl_EnumDevicesBySemantics(
    iface: LPDIRECTINPUT8W,
    ptszUserName: LPCWSTR,
    lpdiActionFormat: LPDIACTIONFORMATW,
    lpCallback: LPDIENUMDEVICESBYSEMANTICSCBW,
    pvRef: LPVOID,
    dwFlags: DWORD,
) -> HRESULT {
    static mut guids: [*const GUID; 2] =
        unsafe { [&raw const GUID_SysKeyboard, &raw const GUID_SysMouse] };
    static mut actionMasks: [DWORD; 2] = [DIKEYBOARD_MASK as DWORD, DIMOUSE_MASK as DWORD];
    let mut This: *mut IDirectInputImpl = impl_from_IDirectInput8W(iface as *mut IDirectInput8W);
    let mut didevi: DIDEVICEINSTANCEW = core::mem::zeroed();
    let mut lpdid: LPDIRECTINPUTDEVICE8W = core::ptr::null_mut();
    let mut callbackFlags: DWORD = 0;
    let mut i: ::core::ffi::c_int = 0;
    let mut j: ::core::ffi::c_int = 0;
    let mut device_count: ::core::ffi::c_int = 0;
    let mut remain: ::core::ffi::c_int = 0;
    let mut didevis: *mut DIDEVICEINSTANCEW = core::ptr::null_mut();

    DPRINT1!(
        "(this=%p,%s,%p,%p,%p,%04x): semi-stub\n",
        This,
        debugstr_w(ptszUserName),
        lpdiActionFormat,
        lpCallback,
        pvRef,
        dwFlags
    );

    didevi.dwSize = ::core::mem::size_of::<DIDEVICEINSTANCEW>() as DWORD;

    /* Enumerate all the joysticks */
    i = 0;
    while (i as usize) < ARRAY_SIZE!(dinput_devices) as usize {
        let mut enumSuccess: HRESULT = 0;

        if (*dinput_devices[i as usize]).enum_deviceW.is_some() {
            j = 0;
            enumSuccess = S_OK;
            while enumSuccess >= 0 {
                DPRINT1!(
                    " - checking device %u ('%s')\n",
                    i,
                    (*dinput_devices[i as usize]).name
                );

                /* Default behavior is to enumerate attached game controllers */
                enumSuccess = (*dinput_devices[i as usize])
                    .enum_deviceW
                    .unwrap_unchecked()(
                    DI8DEVCLASS_GAMECTRL,
                    (DIEDFL_ATTACHEDONLY as DWORD) | dwFlags,
                    &raw mut didevi,
                    (*This).dwVersion,
                    j,
                );
                if enumSuccess == S_OK
                    && should_enumerate_device(
                        ptszUserName as *const WCHAR,
                        dwFlags,
                        (&raw mut (*This).device_players).cast(),
                        &raw mut didevi.guidInstance,
                    ) != 0
                {
                    let fresh15 = device_count;
                    device_count = device_count + 1;
                    if fresh15 != 0 {
                        didevis = HeapReAlloc(
                            GetProcessHeap(),
                            0,
                            didevis as PVOID,
                            (::core::mem::size_of::<DIDEVICEINSTANCEW>() as SIZE_T)
                                .wrapping_mul(device_count as SIZE_T),
                        ) as *mut DIDEVICEINSTANCEW;
                    } else {
                        didevis = HeapAlloc(
                            GetProcessHeap(),
                            0,
                            (::core::mem::size_of::<DIDEVICEINSTANCEW>() as SIZE_T)
                                .wrapping_mul(device_count as SIZE_T),
                        ) as *mut DIDEVICEINSTANCEW;
                    }
                    *didevis.offset((device_count - 1) as isize) = didevi;
                }
                j += 1;
            }
        }
        i += 1;
    }

    remain = device_count;
    /* Add keyboard and mouse to remaining device count */
    if (dwFlags & (DIEDBSFL_FORCEFEEDBACK as DWORD)) == 0 {
        i = 0;
        while (i as usize) < ARRAY_SIZE!(guids) as usize {
            if should_enumerate_device(
                ptszUserName as *const WCHAR,
                dwFlags,
                (&raw mut (*This).device_players).cast(),
                guids[i as usize],
            ) != 0
            {
                remain += 1;
            }
            i += 1;
        }
    }

    i = 0;
    while i < device_count {
        callbackFlags = diactionformat_priorityW(lpdiActionFormat, (*lpdiActionFormat).dwGenre);
        IDirectInput_CreateDevice!(
            iface,
            &raw mut (*didevis.offset(i as isize)).guidInstance,
            &raw mut lpdid,
            core::ptr::null_mut(),
        );

        remain -= 1;
        if (lpCallback.unwrap_unchecked())(
            didevis.offset(i as isize) as LPCDIDEVICEINSTANCEW,
            lpdid,
            callbackFlags,
            remain as DWORD,
            pvRef,
        ) == (DIENUM_STOP as i32)
        {
            HeapFree(GetProcessHeap(), 0, didevis as PVOID);
            return DI_OK;
        }
        i += 1;
    }

    HeapFree(GetProcessHeap(), 0, didevis as PVOID);

    if (dwFlags & (DIEDBSFL_FORCEFEEDBACK as DWORD)) != 0 {
        return DI_OK;
    }

    /* Enumerate keyboard and mouse */
    i = 0;
    while (i as usize) < ARRAY_SIZE!(guids) as usize {
        if should_enumerate_device(
            ptszUserName as *const WCHAR,
            dwFlags,
            (&raw mut (*This).device_players).cast(),
            guids[i as usize],
        ) != 0
        {
            callbackFlags = diactionformat_priorityW(lpdiActionFormat, actionMasks[i as usize]);

            IDirectInput_CreateDevice!(
                iface,
                guids[i as usize],
                &raw mut lpdid,
                core::ptr::null_mut(),
            );
            IDirectInputDevice_GetDeviceInfo!(lpdid, &raw mut didevi);

            remain -= 1;
            if (lpCallback.unwrap_unchecked())(
                &raw mut didevi as LPCDIDEVICEINSTANCEW,
                lpdid,
                callbackFlags,
                remain as DWORD,
                pvRef,
            ) == (DIENUM_STOP as i32)
            {
                return DI_OK;
            }
        }
        i += 1;
    }

    return DI_OK;
}

unsafe extern "system" fn IDirectInput8WImpl_ConfigureDevices(
    iface: LPDIRECTINPUT8W,
    lpdiCallback: LPDICONFIGUREDEVICESCALLBACK,
    lpdiCDParams: LPDICONFIGUREDEVICESPARAMSW,
    dwFlags: DWORD,
    pvRefData: LPVOID,
) -> HRESULT {
    let This: *mut IDirectInputImpl = impl_from_IDirectInput8W(iface as *mut IDirectInput8W);
    let _ = This;

    /* Call helper function in config.c to do the real work */
    return _configure_devices(
        iface as *mut IDirectInput8W,
        lpdiCallback,
        lpdiCDParams,
        dwFlags,
        pvRefData,
    );
}

unsafe extern "system" fn IDirectInput8AImpl_ConfigureDevices(
    iface: LPDIRECTINPUT8A,
    lpdiCallback: LPDICONFIGUREDEVICESCALLBACK,
    lpdiCDParams: LPDICONFIGUREDEVICESPARAMSA,
    dwFlags: DWORD,
    pvRefData: LPVOID,
) -> HRESULT {
    let This: *mut IDirectInputImpl = impl_from_IDirectInput8A(iface);
    let mut diafW: DIACTIONFORMATW = ::core::mem::zeroed();
    let mut diCDParamsW: DICONFIGUREDEVICESPARAMSW = ::core::mem::zeroed();
    let mut hr: HRESULT = 0;
    let mut i: ::core::ffi::c_int = 0;
    WINE_FIXME!("(this=%p,%p,%p,%04x,%p): stub\n", This, lpdiCallback, lpdiCDParams, dwFlags, pvRefData);

    /* Copy parameters */
    diCDParamsW.dwSize = ::core::mem::size_of::<DICONFIGUREDEVICESPARAMSW>() as DWORD;
    diCDParamsW.dwcUsers = (*lpdiCDParams).dwcUsers;
    diCDParamsW.dwcFormats = (*lpdiCDParams).dwcFormats;
    diCDParamsW.lprgFormats = ::core::ptr::addr_of_mut!(diafW) as LPDIACTIONFORMATW;
    diCDParamsW.hwnd = (*lpdiCDParams).hwnd;
    diCDParamsW.lptszUserNames = ::core::ptr::null_mut::<WCHAR>();

    if !(*lpdiCDParams).lptszUserNames.is_null() {
        let mut start: *mut ::core::ffi::c_char =
            (*lpdiCDParams).lptszUserNames as *mut ::core::ffi::c_char;
        let mut to: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
        let mut total_len: ::core::ffi::c_int = 0;
        i = 0;
        while (i as DWORD) < (*lpdiCDParams).dwcUsers {
            let mut end: *mut ::core::ffi::c_char =
                start.offset(1 as isize);
            let mut len: ::core::ffi::c_int = 0;
            loop {
                let fresh16 = end;
                end = end.offset(1);
                if !(*fresh16 != 0) {
                    break;
                }
            }
            len = MultiByteToWideChar(
                CP_ACP as UINT,
                0 as DWORD,
                start as LPCSTR,
                end.offset_from(start) as ::core::ffi::c_longlong as ::core::ffi::c_int,
                ::core::ptr::null_mut::<WCHAR>(),
                0,
            );
            total_len += len + 2; /* length of string and two null char */
            if !to.is_null() {
                to = HeapReAlloc(
                    GetProcessHeap(),
                    0 as DWORD,
                    to as PVOID,
                    (::core::mem::size_of::<WCHAR>() as SIZE_T)
                        .wrapping_mul(total_len as SIZE_T),
                ) as *mut WCHAR;
            } else {
                to = HeapAlloc(
                    GetProcessHeap(),
                    0 as DWORD,
                    (::core::mem::size_of::<WCHAR>() as SIZE_T)
                        .wrapping_mul(total_len as SIZE_T),
                ) as *mut WCHAR;
            }

            MultiByteToWideChar(
                CP_ACP as UINT,
                0 as DWORD,
                start as LPCSTR,
                end.offset_from(start) as ::core::ffi::c_longlong as ::core::ffi::c_int,
                to.offset((total_len - len - 2) as isize),
                len,
            );
            *to.offset(total_len as isize) = 0 as WCHAR;
            *to.offset((total_len - 1) as isize) =
                0 as WCHAR;
            i += 1;
        }
        diCDParamsW.lptszUserNames = to as LPWSTR;
    }

    diafW.rgoAction = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        (::core::mem::size_of::<DIACTIONW>() as SIZE_T)
            .wrapping_mul((*(*lpdiCDParams).lprgFormats).dwNumActions as SIZE_T),
    ) as LPDIACTIONW;
    _copy_diactionformatAtoW(
        ::core::ptr::addr_of_mut!(diafW),
        (*lpdiCDParams).lprgFormats,
    );

    /* Copy action names */
    i = 0;
    while (i as DWORD) < diafW.dwNumActions {
        let from: *const ::core::ffi::c_char =
            (*(*(*lpdiCDParams).lprgFormats).rgoAction.offset(i as isize))
                .u
                .lptszActionName as *const ::core::ffi::c_char;
        let len: ::core::ffi::c_int = MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            from as LPCSTR,
            -1,
            ::core::ptr::null_mut::<WCHAR>(),
            0,
        );
        let to: *mut WCHAR = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            (::core::mem::size_of::<WCHAR>() as SIZE_T).wrapping_mul(len as SIZE_T),
        ) as *mut WCHAR;

        MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            from as LPCSTR,
            -1,
            to as LPWSTR,
            len,
        );
        (*diafW.rgoAction.offset(i as isize)).u.lptszActionName = to as LPCWSTR;
        i += 1;
    }

    hr = IDirectInput8WImpl_ConfigureDevices(
        ::core::ptr::addr_of_mut!((*This).IDirectInput8W_iface),
        lpdiCallback,
        ::core::ptr::addr_of_mut!(diCDParamsW),
        dwFlags,
        pvRefData,
    );

    /* Copy back configuration */
    if hr >= 0 as HRESULT {
        _copy_diactionformatWtoA(
            (*lpdiCDParams).lprgFormats,
            ::core::ptr::addr_of_mut!(diafW),
        );
    }

    /* Free memory */
    i = 0;
    while (i as DWORD) < diafW.dwNumActions {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            (*diafW.rgoAction.offset(i as isize)).u.lptszActionName as PVOID,
        );
        i += 1;
    }

    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        diafW.rgoAction as PVOID,
    );

    heap_free(diCDParamsW.lptszUserNames as *mut ::core::ffi::c_void);

    return hr;
}

unsafe extern "C" fn impl_from_IDirectInputJoyConfig8(iface: *mut IDirectInputJoyConfig8) -> *mut IDirectInputImpl {
    return CONTAINING_RECORD!(iface, IDirectInputImpl, IDirectInputJoyConfig8_iface);
}

unsafe extern "system" fn JoyConfig8Impl_QueryInterface(iface: *mut IDirectInputJoyConfig8, riid: REFIID, ppobj: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut This: *mut IDirectInputImpl = impl_from_IDirectInputJoyConfig8(iface);
    return IDirectInputAImpl_QueryInterface(core::ptr::addr_of_mut!((*This).IDirectInput7A_iface), riid, ppobj as *mut LPVOID);
}

unsafe extern "system" fn JoyConfig8Impl_AddRef(iface: *mut IDirectInputJoyConfig8) -> ULONG {
    let mut This: *mut IDirectInputImpl = impl_from_IDirectInputJoyConfig8(iface);
    return IDirectInputAImpl_AddRef(core::ptr::addr_of_mut!((*This).IDirectInput7A_iface));
}
unsafe extern "system" fn JoyConfig8Impl_Release(iface: *mut IDirectInputJoyConfig8) -> ULONG {
    let mut This: *mut IDirectInputImpl = impl_from_IDirectInputJoyConfig8(iface);
    return IDirectInputAImpl_Release(core::ptr::addr_of_mut!((*This).IDirectInput7A_iface));
}
unsafe extern "system" fn JoyConfig8Impl_Acquire(
    iface: *mut IDirectInputJoyConfig8,
) -> HRESULT {
    // FIXME( "(%p): stub!\n", iface );
    E_NOTIMPL
}
unsafe extern "system" fn JoyConfig8Impl_Unacquire(
    iface: *mut IDirectInputJoyConfig8,
) -> HRESULT {
    // FIXME( "(%p): stub!\n", iface );
    E_NOTIMPL
}
unsafe extern "system" fn JoyConfig8Impl_SetCooperativeLevel(
    iface: *mut IDirectInputJoyConfig8,
    hwnd: HWND,
    flags: DWORD,
) -> HRESULT {
    // FIXME( "(%p)->(%p, 0x%08x): stub!\n", iface, hwnd, flags );
    E_NOTIMPL
}
unsafe extern "system" fn JoyConfig8Impl_SendNotify(
    iface: *mut IDirectInputJoyConfig8,
) -> HRESULT {
    // FIXME( "(%p): stub!\n", iface );
    E_NOTIMPL
}
unsafe extern "system" fn JoyConfig8Impl_EnumTypes(
    iface: *mut IDirectInputJoyConfig8,
    cb: LPDIJOYTYPECALLBACK,
    r#ref: *mut core::ffi::c_void,
) -> HRESULT {
    // FIXME( "(%p)->(%p, %p): stub!\n", iface, cb, ref );
    E_NOTIMPL
}
unsafe extern "system" fn JoyConfig8Impl_GetTypeInfo(
    iface: *mut IDirectInputJoyConfig8,
    name: LPCWSTR,
    info: LPDIJOYTYPEINFO,
    flags: DWORD,
) -> HRESULT {
    // FIXME( "(%p)->(%s, %p, 0x%08x): stub!\n", iface, debugstr_w(name), info, flags );
    E_NOTIMPL
}
unsafe extern "system" fn JoyConfig8Impl_SetTypeInfo(
    iface: *mut IDirectInputJoyConfig8,
    name: LPCWSTR,
    info: LPCDIJOYTYPEINFO,
    flags: DWORD,
    new_name: LPWSTR,
) -> HRESULT {
    // FIXME( "(%p)->(%s, %p, 0x%08x, %s): stub!\n", iface, debugstr_w(name), info, flags, debugstr_w(new_name) );
    E_NOTIMPL
}
unsafe extern "system" fn JoyConfig8Impl_DeleteType(
    iface: *mut IDirectInputJoyConfig8,
    name: LPCWSTR,
) -> HRESULT {
    // FIXME( "(%p)->(%s): stub!\n", iface, debugstr_w(name) );
    E_NOTIMPL
}

unsafe extern "system" fn JoyConfig8Impl_GetConfig(iface: *mut IDirectInputJoyConfig8, id: UINT, info: LPDIJOYCONFIG, flags: DWORD) -> HRESULT {
    let di: *mut IDirectInputImpl = impl_from_IDirectInputJoyConfig8(iface);
    let mut found: UINT = 0;
    let mut i: ::core::ffi::c_int = 0;
    let mut j: ::core::ffi::c_int = 0;
    let mut r: HRESULT = 0;

    // X(DIJC_GUIDINSTANCE)
    if flags & 0x1 as DWORD != 0 {
        FIXME!("\tflags |= DIJC_GUIDINSTANCE\n");
    }
    // X(DIJC_REGHWCONFIGTYPE)
    if flags & 0x2 as DWORD != 0 {
        FIXME!("\tflags |= DIJC_REGHWCONFIGTYPE\n");
    }
    // X(DIJC_GAIN)
    if flags & 0x4 as DWORD != 0 {
        FIXME!("\tflags |= DIJC_GAIN\n");
    }
    // X(DIJC_CALLOUT)
    if flags & 0x8 as DWORD != 0 {
        FIXME!("\tflags |= DIJC_CALLOUT\n");
    }

    /* Enumerate all joysticks in order */
    i = 0;
    while (i as usize) < ARRAY_SIZE!(dinput_devices) as usize {
        'outer: {
            if (*dinput_devices[i as usize]).enum_deviceA.is_none() {
                break 'outer;
            }

            j = 0;
            r = S_OK;
            while r >= 0 {
                let mut dev: DIDEVICEINSTANCEA = core::mem::zeroed();
                dev.dwSize = core::mem::size_of::<DIDEVICEINSTANCEA>() as DWORD;
                r = (*dinput_devices[i as usize])
                    .enum_deviceA
                    .unwrap_unchecked()(
                    DI8DEVCLASS_GAMECTRL,
                    0,
                    core::ptr::addr_of_mut!(dev),
                    (*di).dwVersion,
                    j,
                );
                if r == S_OK {
                    /* Only take into account the chosen id */
                    if found == id {
                        if flags & DIJC_GUIDINSTANCE != 0 {
                            (*info).guidInstance = dev.guidInstance;
                        }

                        return DI_OK;
                    }
                    found += 1;
                }
                j += 1;
            }
        }
        i += 1;
    }

    return DIERR_NOMOREITEMS;
}
unsafe extern "system" fn JoyConfig8Impl_SetConfig(
    iface: *mut IDirectInputJoyConfig8,
    id: UINT,
    info: LPCDIJOYCONFIG,
    flags: DWORD,
) -> HRESULT {
    FIXME!("(%p)->(%d, %p, 0x%08x): stub!\n", iface, id, info, flags);
    E_NOTIMPL
}
unsafe extern "system" fn JoyConfig8Impl_DeleteConfig(
    iface: *mut IDirectInputJoyConfig8,
    id: UINT,
) -> HRESULT {
    // FIXME( "(%p)->(%d): stub!\n", iface, id );
    E_NOTIMPL
}
unsafe extern "system" fn JoyConfig8Impl_GetUserValues(
    iface: *mut IDirectInputJoyConfig8,
    info: LPDIJOYUSERVALUES,
    flags: DWORD,
) -> HRESULT {
    // FIXME( "(%p)->(%p, 0x%08x): stub!\n", iface, info, flags );
    E_NOTIMPL
}
unsafe extern "system" fn JoyConfig8Impl_SetUserValues(
    iface: *mut IDirectInputJoyConfig8,
    info: LPCDIJOYUSERVALUES,
    flags: DWORD,
) -> HRESULT {
    // FIXME( "(%p)->(%p, 0x%08x): stub!\n", iface, info, flags );
    E_NOTIMPL
}
unsafe extern "system" fn JoyConfig8Impl_AddNewHardware(
    iface: *mut IDirectInputJoyConfig8,
    hwnd: HWND,
    guid: REFGUID,
) -> HRESULT {
    // FIXME( "(%p)->(%p, %s): stub!\n", iface, hwnd, debugstr_guid(guid) );
    E_NOTIMPL
}
unsafe extern "system" fn JoyConfig8Impl_OpenTypeKey(
    iface: *mut IDirectInputJoyConfig8,
    name: LPCWSTR,
    security: DWORD,
    key: PHKEY,
) -> HRESULT {
    // FIXME( "(%p)->(%s, 0x%08x, %p): stub!\n", iface, debugstr_w(name), security, key );
    E_NOTIMPL
}
unsafe extern "system" fn JoyConfig8Impl_OpenAppStatusKey(
    iface: *mut IDirectInputJoyConfig8,
    key: PHKEY,
) -> HRESULT {
    // FIXME( "(%p)->(%p): stub!\n", iface, key );
    E_NOTIMPL
}

unsafe extern "C" fn impl_from_IClassFactory(iface: *mut IClassFactory) -> *mut IClassFactoryImpl {
    return CONTAINING_RECORD!(iface, IClassFactoryImpl, IClassFactory_iface);
}

unsafe extern "system" fn DICF_QueryInterface(iface: LPCLASSFACTORY, riid: REFIID, ppobj: *mut LPVOID) -> HRESULT {
    let mut This: *mut IClassFactoryImpl = impl_from_IClassFactory(iface as *mut IClassFactory);
    WINE_FIXME!("(%p)->(%s,%p),stub!\n", This, debugstr_guid(riid as *const _GUID), ppobj);
    return E_NOINTERFACE;
}

unsafe extern "system" fn DICF_AddRef(iface: LPCLASSFACTORY) -> ULONG {
    let mut This: *mut IClassFactoryImpl = impl_from_IClassFactory(iface as *mut IClassFactory);
    return InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_) as *mut core::ffi::c_long) as ULONG;
}

unsafe extern "system" fn DICF_Release(iface: LPCLASSFACTORY) -> ULONG {
    let mut This: *mut IClassFactoryImpl = impl_from_IClassFactory(iface as *mut IClassFactory);
    /* static class, won't be  freed */
    return InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_) as *mut LONG) as ULONG;
}

unsafe extern "system" fn DICF_CreateInstance(
    iface: LPCLASSFACTORY,
    pOuter: LPUNKNOWN,
    riid: REFIID,
    ppobj: *mut LPVOID,
) -> HRESULT {
    let mut This: *mut IClassFactoryImpl = impl_from_IClassFactory(iface as *mut IClassFactory);

    if IsEqualGUID!(core::ptr::addr_of!(IID_IUnknown), riid) != false ||
        IsEqualGUID!(core::ptr::addr_of!(IID_IDirectInputA), riid) != false ||
        IsEqualGUID!(core::ptr::addr_of!(IID_IDirectInputW), riid) != false ||
        IsEqualGUID!(core::ptr::addr_of!(IID_IDirectInput2A), riid) != false ||
        IsEqualGUID!(core::ptr::addr_of!(IID_IDirectInput2W), riid) != false ||
        IsEqualGUID!(core::ptr::addr_of!(IID_IDirectInput7A), riid) != false ||
        IsEqualGUID!(core::ptr::addr_of!(IID_IDirectInput7W), riid) != false ||
        IsEqualGUID!(core::ptr::addr_of!(IID_IDirectInput8A), riid) != false ||
        IsEqualGUID!(core::ptr::addr_of!(IID_IDirectInput8W), riid) != false
    {
        return create_directinput_instance(riid, ppobj, core::ptr::null_mut());
    }

    return E_NOINTERFACE;
}

unsafe extern "system" fn DICF_LockServer(iface: LPCLASSFACTORY, dolock: BOOL) -> HRESULT {
    let mut This: *mut IClassFactoryImpl = impl_from_IClassFactory(iface as *mut IClassFactory);
    FIXME!("(%p)->(%d),stub!\n", This, dolock);
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn DllCanUnloadNow() -> HRESULT {
    S_FALSE
}

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(rclsid: *const CLSID, riid: *const IID, ppv: *mut LPVOID) -> HRESULT {
    if IsEqualCLSID!(core::ptr::addr_of!(IID_IClassFactory), riid) != 0 {
        *ppv = core::ptr::addr_of_mut!(DINPUT_CF) as LPVOID;
        IClassFactory_AddRef(*ppv as *mut IClassFactory);
        return S_OK;
    }
    return CLASS_E_CLASSNOTAVAILABLE;
}

#[no_mangle]
pub unsafe extern "system" fn DllRegisterServer() -> HRESULT {
    return __wine_register_resources(DINPUT_instance as HMODULE);
}

#[no_mangle]
pub unsafe extern "system" fn DllUnregisterServer() -> HRESULT {
    return __wine_unregister_resources(DINPUT_instance as HMODULE);
}

unsafe extern "system" fn LL_hook_proc(
    mut code: ::core::ffi::c_int,
    mut wparam: WPARAM,
    mut lparam: LPARAM,
) -> LRESULT {
    let mut dinput: *mut IDirectInputImpl = core::ptr::null_mut::<IDirectInputImpl>();
    let mut skip: ::core::ffi::c_int = 0;

    if code != (HC_ACTION as i32) {
        return CallNextHookEx(core::ptr::null_mut(), code, wparam, lparam);
    }

    EnterCriticalSection(core::ptr::addr_of_mut!(dinput_hook_crit));
    dinput = ((*core::ptr::addr_of_mut!(direct_input_list)).next as *mut u8)
        .offset(-(core::mem::offset_of!(IDirectInputImpl, entry) as isize))
        as *mut IDirectInputImpl;
    while core::ptr::addr_of_mut!((*dinput).entry)
        != core::ptr::addr_of_mut!(direct_input_list)
    {
        let mut dev: *mut IDirectInputDeviceImpl = core::ptr::null_mut::<IDirectInputDeviceImpl>();

        EnterCriticalSection(core::ptr::addr_of_mut!((*dinput).crit));
        dev = ((*dinput).devices_list.next as *mut u8)
            .offset(-(core::mem::offset_of!(IDirectInputDeviceImpl, entry) as isize))
            as *mut IDirectInputDeviceImpl;
        while core::ptr::addr_of_mut!((*dev).entry)
            != core::ptr::addr_of_mut!((*dinput).devices_list)
        {
            if (*dev).acquired != 0 && (*dev).event_proc.is_some() {
                WINE_TRACE!("calling %p->%p (%lx %lx)\n", dev, (*dev).event_proc, wparam, lparam);
                skip |= (*dev).event_proc.unwrap_unchecked()(
                    core::ptr::addr_of_mut!((*dev).IDirectInputDevice8A_iface),
                    wparam,
                    lparam,
                );
            }
            dev = ((*dev).entry.next as *mut u8)
                .offset(-(core::mem::offset_of!(IDirectInputDeviceImpl, entry) as isize))
                as *mut IDirectInputDeviceImpl;
        }
        LeaveCriticalSection(core::ptr::addr_of_mut!((*dinput).crit));
        dinput = ((*dinput).entry.next as *mut u8)
            .offset(-(core::mem::offset_of!(IDirectInputImpl, entry) as isize))
            as *mut IDirectInputImpl;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(dinput_hook_crit));

    return if skip != 0 {
        1
    } else {
        CallNextHookEx(core::ptr::null_mut(), code, wparam, lparam)
    };
}
unsafe extern "system" fn callwndproc_proc(
    mut code: ::core::ffi::c_int,
    mut wparam: WPARAM,
    mut lparam: LPARAM,
) -> LRESULT {
    let mut msg: *mut CWPSTRUCT = lparam as *mut CWPSTRUCT;
    let mut dinput: *mut IDirectInputImpl = ::core::ptr::null_mut::<IDirectInputImpl>();
    let mut foreground: HWND = ::core::ptr::null_mut::<HWND__>();
    if code != (HC_ACTION as i32)
        || (*msg).message != WM_KILLFOCUS as UINT
            && (*msg).message != WM_ACTIVATEAPP as UINT
            && (*msg).message != WM_ACTIVATE as UINT
    {
        return CallNextHookEx(
            0 as HHOOK,
            code,
            wparam,
            lparam,
        );
    }
    foreground = GetForegroundWindow();
    EnterCriticalSection(&raw mut dinput_hook_crit);
    dinput = (direct_input_list.next as *mut ::core::ffi::c_char).offset(
        -(core::mem::offset_of!(IDirectInputImpl, entry) as ::core::ffi::c_ulonglong
            as isize),
    ) as *mut IDirectInputImpl;
    while &raw mut (*dinput).entry != (&raw mut direct_input_list).cast() {
        let mut dev: *mut IDirectInputDeviceImpl =
            ::core::ptr::null_mut::<IDirectInputDeviceImpl>();
        EnterCriticalSection(&raw mut (*dinput).crit);
        dev = ((*dinput).devices_list.next as *mut ::core::ffi::c_char).offset(
            -(core::mem::offset_of!(IDirectInputDeviceImpl, entry)
                as ::core::ffi::c_ulonglong as isize),
        ) as *mut IDirectInputDeviceImpl;
        while &raw mut (*dev).entry != &raw mut (*dinput).devices_list {
            if !((*dev).acquired == 0) {
                if (*msg).hwnd == (*dev).win && (*msg).hwnd != foreground {
                    WINE_TRACE!("%p window is not foreground - unacquiring %p\n", (*dev).win, dev);
                    IDirectInputDevice_Unacquire!(&raw mut (*dev).IDirectInputDevice8A_iface);
                }
            }
            dev = ((*dev).entry.next as *mut ::core::ffi::c_char).offset(
                -(core::mem::offset_of!(IDirectInputDeviceImpl, entry)
                    as ::core::ffi::c_ulonglong as isize),
            ) as *mut IDirectInputDeviceImpl;
        }
        LeaveCriticalSection(&raw mut (*dinput).crit);
        dinput = ((*dinput).entry.next as *mut ::core::ffi::c_char).offset(
            -(core::mem::offset_of!(IDirectInputImpl, entry)
                as ::core::ffi::c_ulonglong as isize),
        ) as *mut IDirectInputImpl;
    }
    LeaveCriticalSection(&raw mut dinput_hook_crit);
    return CallNextHookEx(
        0 as HHOOK,
        code,
        wparam,
        lparam,
    );
}
unsafe extern "system" fn hook_thread_proc(mut param: *mut ::core::ffi::c_void) -> DWORD {
    static mut kbd_hook: HHOOK = ::core::ptr::null_mut();
    static mut mouse_hook: HHOOK = ::core::ptr::null_mut();
    let mut msg: MSG = ::core::mem::zeroed();

    /* Force creation of the message queue */
    PeekMessageW(
        ::core::ptr::addr_of_mut!(msg),
        core::ptr::null_mut(),
        0,
        0,
        PM_NOREMOVE
    );
    SetEvent(param);

    while GetMessageW(
        ::core::ptr::addr_of_mut!(msg),
        core::ptr::null_mut(),
        0,
        0
    ) != 0
    {
        let mut kbd_cnt: UINT = 0;
        let mut mice_cnt: UINT = 0;

        if msg.message == (WM_USER + 0x10) as UINT {
            let mut dinput: *mut IDirectInputImpl = ::core::ptr::null_mut::<IDirectInputImpl>();

            WINE_TRACE!("Processing hook change notification lp:%ld\n", msg.lParam);

            if msg.wParam == 0 && msg.lParam == 0 {
                if !kbd_hook.is_null() {
                    UnhookWindowsHookEx(kbd_hook);
                }
                if !mouse_hook.is_null() {
                    UnhookWindowsHookEx(mouse_hook);
                }
                mouse_hook = ::core::ptr::null_mut::<HHOOK__>();
                kbd_hook = mouse_hook;
                break;
            } else {
                EnterCriticalSection(core::ptr::addr_of_mut!(dinput_hook_crit));

                /* Count acquired keyboards and mice*/
                dinput = (direct_input_list.next as *mut ::core::ffi::c_char).offset(
                    -(core::mem::offset_of!(IDirectInputImpl, entry)
                        as ::core::ffi::c_ulonglong as isize),
                ) as *mut IDirectInputImpl;
                while core::ptr::addr_of_mut!((*dinput).entry)
                    != core::ptr::addr_of_mut!(direct_input_list)
                {
                    let mut dev: *mut IDirectInputDeviceImpl =
                        ::core::ptr::null_mut::<IDirectInputDeviceImpl>();
                    EnterCriticalSection(core::ptr::addr_of_mut!((*dinput).crit));
                    dev = ((*dinput).devices_list.next as *mut ::core::ffi::c_char).offset(
                        -(core::mem::offset_of!(IDirectInputDeviceImpl, entry)
                            as ::core::ffi::c_ulonglong as isize),
                    ) as *mut IDirectInputDeviceImpl;
                    while core::ptr::addr_of_mut!((*dev).entry)
                        != core::ptr::addr_of_mut!((*dinput).devices_list)
                    {
                        if !((*dev).acquired == 0 || (*dev).event_proc.is_none()) {
                            if IsEqualGUID!(
                                core::ptr::addr_of_mut!((*dev).guid),
                                core::ptr::addr_of!(GUID_SysKeyboard),
                            ) != false
                            {
                                kbd_cnt = kbd_cnt.wrapping_add(1);
                            } else if IsEqualGUID!(
                                core::ptr::addr_of_mut!((*dev).guid),
                                core::ptr::addr_of!(GUID_SysMouse),
                            ) != false
                            {
                                mice_cnt = mice_cnt.wrapping_add(1);
                            }
                        }
                        dev = ((*dev).entry.next as *mut ::core::ffi::c_char).offset(
                            -(core::mem::offset_of!(IDirectInputDeviceImpl, entry)
                                as ::core::ffi::c_ulonglong as isize),
                        ) as *mut IDirectInputDeviceImpl;
                    }
                    LeaveCriticalSection(core::ptr::addr_of_mut!((*dinput).crit));
                    dinput = ((*dinput).entry.next as *mut ::core::ffi::c_char).offset(
                        -(core::mem::offset_of!(IDirectInputImpl, entry)
                            as ::core::ffi::c_ulonglong as isize),
                    ) as *mut IDirectInputImpl;
                }
                LeaveCriticalSection(core::ptr::addr_of_mut!(dinput_hook_crit));

                if kbd_cnt != 0 && kbd_hook.is_null() {
                    kbd_hook = SetWindowsHookExW(
                        (WH_KEYBOARD_LL as i32),
                        Some(
                            LL_hook_proc,
                        ),
                        DINPUT_instance,
                        0,
                    );
                } else if kbd_cnt == 0 && !kbd_hook.is_null() {
                    UnhookWindowsHookEx(kbd_hook);
                    kbd_hook = ::core::ptr::null_mut::<HHOOK__>();
                }

                if mice_cnt != 0 && mouse_hook.is_null() {
                    mouse_hook = SetWindowsHookExW(
                        (WH_MOUSE_LL as i32),
                        Some(
                            LL_hook_proc,
                        ),
                        DINPUT_instance,
                        0,
                    );
                } else if mice_cnt == 0 && !mouse_hook.is_null() {
                    UnhookWindowsHookEx(mouse_hook);
                    mouse_hook = ::core::ptr::null_mut::<HHOOK__>();
                }
            }
        }
        TranslateMessage(core::ptr::addr_of_mut!(msg));
        DispatchMessageW(core::ptr::addr_of_mut!(msg));
    }

    FreeLibraryAndExitThread(DINPUT_instance, 0);
}

unsafe extern "C" fn check_hook_thread() -> BOOL {
    static mut hook_thread: HANDLE = core::ptr::null_mut();
    let mut module: HMODULE = core::ptr::null_mut();

    EnterCriticalSection(core::ptr::addr_of_mut!(dinput_hook_crit));

    if list_empty(core::ptr::addr_of_mut!(direct_input_list)) == 0 && hook_thread.is_null() {
        GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS as DWORD,
            DINPUT_instance as LPCWSTR,
            core::ptr::addr_of_mut!(module),
        );
        hook_thread_event = CreateEventW(
            core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
            FALSE as BOOL,
            FALSE as BOOL,
            core::ptr::null::<WCHAR>(),
        );
        hook_thread = CreateThread(
            core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
            0 as DWORD,
            Some(hook_thread_proc),
            hook_thread_event as LPVOID,
            0 as DWORD,
            core::ptr::addr_of_mut!(hook_thread_id),
        );
    } else if list_empty(core::ptr::addr_of_mut!(direct_input_list)) != 0 && !hook_thread.is_null() {
        let mut tid: DWORD = hook_thread_id;

        if !hook_thread_event.is_null() {
            WaitForSingleObject(hook_thread_event, INFINITE as DWORD);
            CloseHandle(hook_thread_event);
            hook_thread_event = core::ptr::null_mut();
        }

        hook_thread_id = 0 as DWORD;
        PostThreadMessageW(
            tid,
            (WM_USER + 0x10) as UINT,
            0 as WPARAM,
            0 as LPARAM,
        );
        CloseHandle(hook_thread);
        hook_thread = core::ptr::null_mut();
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(dinput_hook_crit));
    return (hook_thread_id != 0 as DWORD) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn check_dinput_hooks(mut iface: LPDIRECTINPUTDEVICE8W, mut acquired: BOOL) {
    static mut callwndproc_hook: HHOOK = core::ptr::null_mut::<HHOOK__>() as *mut HHOOK__;
    static mut foreground_cnt: ULONG = 0;
    let mut dev: *mut IDirectInputDeviceImpl =
        impl_from_IDirectInputDevice8W(iface as *mut IDirectInputDevice8W);

    EnterCriticalSection(core::ptr::addr_of_mut!(dinput_hook_crit));

    if ((*dev).dwCoopLevel & (DISCL_FOREGROUND as DWORD)) != 0 {
        if acquired != 0 {
            foreground_cnt = foreground_cnt.wrapping_add(1);
        } else {
            foreground_cnt = foreground_cnt.wrapping_sub(1);
        }
    }

    if foreground_cnt != 0 && callwndproc_hook.is_null() {
        callwndproc_hook = SetWindowsHookExW(
            WH_CALLWNDPROC as ::core::ffi::c_int,
            Some(
                callwndproc_proc,
            ),
            DINPUT_instance,
            GetCurrentThreadId(),
        );
    } else if foreground_cnt == 0 && !callwndproc_hook.is_null() {
        UnhookWindowsHookEx(callwndproc_hook);
        callwndproc_hook = core::ptr::null_mut::<HHOOK__>();
    }

    if !hook_thread_event.is_null() {
        WaitForSingleObject(hook_thread_event, INFINITE as DWORD);
        CloseHandle(hook_thread_event);
        hook_thread_event = core::ptr::null_mut();
    }

    PostThreadMessageW(
        hook_thread_id,
        (WM_USER + 0x10) as UINT,
        1 as WPARAM,
        0 as LPARAM,
    );

    LeaveCriticalSection(core::ptr::addr_of_mut!(dinput_hook_crit));
}
#[no_mangle]
pub unsafe extern "C" fn check_dinput_events() {
    // The C polls `MsgWaitForMultipleObjectsEx` only `#ifndef __REACTOS__` (winex11 needs it
    // to forward events); on ReactOS the function is empty.
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(inst: HINSTANCE, reason: DWORD, reserved: LPVOID) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(inst as HMODULE);
            DINPUT_instance = inst;
        }
        DLL_PROCESS_DETACH => {
            if !reserved.is_null() {
            } else {
                DeleteCriticalSection(core::ptr::addr_of_mut!(dinput_hook_crit));
            }
        }
        _ => {}
    }
    return (TRUE) as BOOL;
}

// COM vtable tables — generated from dinput_main.c by vtbl_gen.py
/// dinput_main.c:1491 `static const IDirectInput7AVtbl ddi7avt`  (slot order: rust:types_amd64.rs)
pub static ddi7avt: IDirectInput7AVtbl = IDirectInput7AVtbl {
    QueryInterface: Some(IDirectInputAImpl_QueryInterface),
    AddRef: Some(IDirectInputAImpl_AddRef),
    Release: Some(IDirectInputAImpl_Release),
    CreateDevice: Some(IDirectInputAImpl_CreateDevice),
    EnumDevices: Some(IDirectInputAImpl_EnumDevices),
    GetDeviceStatus: Some(IDirectInputAImpl_GetDeviceStatus),
    RunControlPanel: Some(IDirectInputAImpl_RunControlPanel),
    Initialize: Some(IDirectInputAImpl_Initialize),
    FindDevice: Some(IDirectInput2AImpl_FindDevice),
    CreateDeviceEx: Some(IDirectInput7AImpl_CreateDeviceEx),
};

/// dinput_main.c:1504 `static const IDirectInput7WVtbl ddi7wvt`  (slot order: rust:types_amd64.rs)
pub static ddi7wvt: IDirectInput7WVtbl = IDirectInput7WVtbl {
    QueryInterface: Some(IDirectInputWImpl_QueryInterface),
    AddRef: Some(IDirectInputWImpl_AddRef),
    Release: Some(IDirectInputWImpl_Release),
    CreateDevice: Some(IDirectInputWImpl_CreateDevice),
    EnumDevices: Some(IDirectInputWImpl_EnumDevices),
    GetDeviceStatus: Some(IDirectInputWImpl_GetDeviceStatus),
    RunControlPanel: Some(IDirectInputWImpl_RunControlPanel),
    Initialize: Some(IDirectInputWImpl_Initialize),
    FindDevice: Some(IDirectInput2WImpl_FindDevice),
    CreateDeviceEx: Some(IDirectInput7WImpl_CreateDeviceEx),
};

/// dinput_main.c:1517 `static const IDirectInput8AVtbl ddi8avt`  (slot order: rust:types_amd64.rs)
pub static ddi8avt: IDirectInput8AVtbl = IDirectInput8AVtbl {
    QueryInterface: Some(IDirectInput8AImpl_QueryInterface),
    AddRef: Some(IDirectInput8AImpl_AddRef),
    Release: Some(IDirectInput8AImpl_Release),
    CreateDevice: Some(IDirectInput8AImpl_CreateDevice),
    EnumDevices: Some(IDirectInput8AImpl_EnumDevices),
    GetDeviceStatus: Some(IDirectInput8AImpl_GetDeviceStatus),
    RunControlPanel: Some(IDirectInput8AImpl_RunControlPanel),
    Initialize: Some(IDirectInput8AImpl_Initialize),
    FindDevice: Some(IDirectInput8AImpl_FindDevice),
    EnumDevicesBySemantics: Some(IDirectInput8AImpl_EnumDevicesBySemantics),
    ConfigureDevices: Some(IDirectInput8AImpl_ConfigureDevices),
};

/// dinput_main.c:1531 `static const IDirectInput8WVtbl ddi8wvt`  (slot order: rust:types_amd64.rs)
pub static ddi8wvt: IDirectInput8WVtbl = IDirectInput8WVtbl {
    QueryInterface: Some(IDirectInput8WImpl_QueryInterface),
    AddRef: Some(IDirectInput8WImpl_AddRef),
    Release: Some(IDirectInput8WImpl_Release),
    CreateDevice: Some(IDirectInput8WImpl_CreateDevice),
    EnumDevices: Some(IDirectInput8WImpl_EnumDevices),
    GetDeviceStatus: Some(IDirectInput8WImpl_GetDeviceStatus),
    RunControlPanel: Some(IDirectInput8WImpl_RunControlPanel),
    Initialize: Some(IDirectInput8WImpl_Initialize),
    FindDevice: Some(IDirectInput8WImpl_FindDevice),
    EnumDevicesBySemantics: Some(IDirectInput8WImpl_EnumDevicesBySemantics),
    ConfigureDevices: Some(IDirectInput8WImpl_ConfigureDevices),
};

/// dinput_main.c:1545 `static const IDirectInputJoyConfig8Vtbl JoyConfig8vt`  (slot order: rust:types_amd64.rs)
pub static JoyConfig8vt: IDirectInputJoyConfig8Vtbl = IDirectInputJoyConfig8Vtbl {
    QueryInterface: Some(JoyConfig8Impl_QueryInterface),
    AddRef: Some(JoyConfig8Impl_AddRef),
    Release: Some(JoyConfig8Impl_Release),
    Acquire: Some(JoyConfig8Impl_Acquire),
    Unacquire: Some(JoyConfig8Impl_Unacquire),
    SetCooperativeLevel: Some(JoyConfig8Impl_SetCooperativeLevel),
    SendNotify: Some(JoyConfig8Impl_SendNotify),
    EnumTypes: Some(JoyConfig8Impl_EnumTypes),
    GetTypeInfo: Some(JoyConfig8Impl_GetTypeInfo),
    SetTypeInfo: Some(JoyConfig8Impl_SetTypeInfo),
    DeleteType: Some(JoyConfig8Impl_DeleteType),
    GetConfig: Some(JoyConfig8Impl_GetConfig),
    SetConfig: Some(JoyConfig8Impl_SetConfig),
    DeleteConfig: Some(JoyConfig8Impl_DeleteConfig),
    GetUserValues: Some(JoyConfig8Impl_GetUserValues),
    SetUserValues: Some(JoyConfig8Impl_SetUserValues),
    AddNewHardware: Some(JoyConfig8Impl_AddNewHardware),
    OpenTypeKey: Some(JoyConfig8Impl_OpenTypeKey),
    OpenAppStatusKey: Some(JoyConfig8Impl_OpenAppStatusKey),
};

/// dinput_main.c:1630 `static const IClassFactoryVtbl DICF_Vtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static DICF_Vtbl: IClassFactoryVtbl = IClassFactoryVtbl {
    QueryInterface: Some(DICF_QueryInterface),
    AddRef: Some(DICF_AddRef),
    Release: Some(DICF_Release),
    CreateInstance: Some(DICF_CreateInstance),
    LockServer: Some(DICF_LockServer),
};
