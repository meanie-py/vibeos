use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut keyboard_device: dinput_device = unsafe {
    dinput_device {
        name: b"Wine keyboard driver\0" as *const u8 as *const ::core::ffi::c_char,
        enum_deviceA: Some(
            keyboarddev_enum_deviceA
                as unsafe extern "C" fn(
                    DWORD,
                    DWORD,
                    LPDIDEVICEINSTANCEA,
                    DWORD,
                    ::core::ffi::c_int,
                ) -> HRESULT,
        ),
        enum_deviceW: Some(
            keyboarddev_enum_deviceW
                as unsafe extern "C" fn(
                    DWORD,
                    DWORD,
                    LPDIDEVICEINSTANCEW,
                    DWORD,
                    ::core::ffi::c_int,
                ) -> HRESULT,
        ),
        create_device: Some(
            keyboarddev_create_device
                as unsafe extern "C" fn(
                    *mut IDirectInputImpl,
                    *const GUID,
                    *const IID,
                    *mut LPVOID,
                    ::core::ffi::c_int,
                ) -> HRESULT,
        ),
    }
};

// WINE_DEFAULT_DEBUG_CHANNEL(dinput): debug-channel macro, no Rust declaration emitted here.

pub const WINE_DINPUT_KEYBOARD_MAX_KEYS: usize = 256;

#[repr(C)]
pub struct SysKeyboardImpl {
    pub base: IDirectInputDeviceImpl,
    pub DInputKeyState: [BYTE; WINE_DINPUT_KEYBOARD_MAX_KEYS],
    pub subtype: DWORD,
}

unsafe extern "C" fn impl_from_IDirectInputDevice8A(iface: *mut IDirectInputDevice8A) -> *mut SysKeyboardImpl {
    return ((iface as *mut u8).wrapping_sub(core::mem::offset_of!(IDirectInputDeviceImpl, IDirectInputDevice8A_iface)) as *mut IDirectInputDeviceImpl as *mut u8).wrapping_sub(core::mem::offset_of!(SysKeyboardImpl, base)) as *mut SysKeyboardImpl;
}

unsafe extern "C" fn impl_from_IDirectInputDevice8W(iface: *mut IDirectInputDevice8W) -> *mut SysKeyboardImpl {
    let Inner: *mut IDirectInputDeviceImpl = (iface as *mut u8).wrapping_sub(core::mem::offset_of!(IDirectInputDeviceImpl, IDirectInputDevice8W_iface)) as *mut IDirectInputDeviceImpl;
    return (Inner as *mut u8).wrapping_sub(core::mem::offset_of!(SysKeyboardImpl, base)) as *mut SysKeyboardImpl;
}

unsafe extern "C" fn IDirectInputDevice8A_from_impl(This: *mut SysKeyboardImpl) -> *mut IDirectInputDevice8A {
    return core::ptr::addr_of_mut!((*This).base.IDirectInputDevice8A_iface);
}

unsafe extern "C" fn IDirectInputDevice8W_from_impl(This: *mut SysKeyboardImpl) -> *mut IDirectInputDevice8W {
    return core::ptr::addr_of_mut!((*This).base.IDirectInputDevice8W_iface);
}

unsafe extern "C" fn map_dik_code(mut scanCode: DWORD, mut vkCode: DWORD, mut subType: DWORD) -> BYTE {
    if scanCode == 0 {
        scanCode = MapVirtualKeyW(vkCode as UINT, MAPVK_VK_TO_VSC as UINT) as DWORD;
    }
    if subType == DIDEVTYPEKEYBOARD_JAPAN106 {
        match scanCode {
            0x0d => {
                scanCode = DIK_CIRCUMFLEX as DWORD;
            }
            0x1a => {
                scanCode = DIK_AT as DWORD;
            }
            0x1b => {
                scanCode = DIK_LBRACKET as DWORD;
            }
            0x28 => {
                scanCode = DIK_COLON as DWORD;
            }
            0x29 => {
                scanCode = DIK_KANJI as DWORD;
            }
            0x2b => {
                scanCode = DIK_RBRACKET as DWORD;
            }
            0x73 => {
                scanCode = DIK_BACKSLASH as DWORD;
            }
            _ => {}
        }
    }
    return scanCode as BYTE;
}

unsafe extern "C" fn KeyboardCallback(
    mut iface: LPDIRECTINPUTDEVICE8A,
    mut wparam: WPARAM,
    mut lparam: LPARAM,
) -> ::core::ffi::c_int {
    let mut This: *mut SysKeyboardImpl = impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    let mut dik_code: ::core::ffi::c_int = 0;
    let mut ret: ::core::ffi::c_int = ((*This).base.dwCoopLevel & DISCL_EXCLUSIVE as DWORD) as ::core::ffi::c_int;
    let mut hook: *mut KBDLLHOOKSTRUCT = lparam as *mut KBDLLHOOKSTRUCT;
    let mut new_diks: BYTE = 0;
    if wparam != WM_KEYDOWN as WPARAM
        && wparam != WM_KEYUP as WPARAM
        && wparam != WM_SYSKEYDOWN as WPARAM
        && wparam != WM_SYSKEYUP as WPARAM
    {
        return 0 as ::core::ffi::c_int;
    }
    WINE_TRACE!("(%p) wp %08lx, lp %08lx, vk %02x, scan %02x\n", iface, wparam, lparam, (*hook).vkCode, (*hook).scanCode);
    if (*hook).vkCode == VK_RSHIFT as DWORD {
        dik_code = DIK_RSHIFT as ::core::ffi::c_int;
    } else if (*hook).vkCode == VK_PAUSE as DWORD {
        dik_code = DIK_PAUSE as ::core::ffi::c_int;
    } else if (*hook).vkCode == VK_NUMLOCK as DWORD {
        dik_code = DIK_NUMLOCK as ::core::ffi::c_int;
    } else if (*hook).vkCode == VK_SUBTRACT as DWORD {
        dik_code = DIK_SUBTRACT as ::core::ffi::c_int;
    } else {
        dik_code = map_dik_code(
            (*hook).scanCode & 0xff as DWORD,
            (*hook).vkCode,
            (*This).subtype,
        ) as ::core::ffi::c_int;
        if (*hook).flags & LLKHF_EXTENDED as DWORD != 0 {
            dik_code |= 0x80 as ::core::ffi::c_int;
        }
    }
    new_diks = (if ((*hook).flags & LLKHF_UP as DWORD) != 0 {
        0 as ::core::ffi::c_int
    } else {
        0x80 as ::core::ffi::c_int
    }) as BYTE;
    if new_diks as ::core::ffi::c_int
        == (*This).DInputKeyState[dik_code as usize] as ::core::ffi::c_int
    {
        return ret;
    }
    (*This).DInputKeyState[dik_code as usize] = new_diks;
    WINE_TRACE!(" setting %02X to %02X\n", dik_code, (*This).DInputKeyState[dik_code as usize] as ::core::ffi::c_int);
    EnterCriticalSection(&raw mut (*This).base.crit);
    let fresh16 = (*(*This).base.dinput).evsequence;
    (*(*This).base.dinput).evsequence = (*(*This).base.dinput).evsequence.wrapping_add(1);
    queue_event(
        iface,
        DIDFT_MAKEINSTANCE!(dik_code as DWORD) | (DIDFT_PSHBUTTON as DWORD) as i32,
        new_diks as DWORD,
        GetCurrentTime(),
        fresh16,
    );
    LeaveCriticalSection(&raw mut (*This).base.crit);
    return ret;
}

unsafe extern "C" fn get_keyboard_subtype() -> DWORD {
    let mut kbd_type: DWORD = 0;
    let mut kbd_subtype: DWORD = 0;
    let mut dev_subtype: DWORD = 0;
    kbd_type = GetKeyboardType(0) as DWORD;
    kbd_subtype = GetKeyboardType(1) as DWORD;
    if kbd_type == 4 as DWORD || kbd_type == 7 as DWORD && kbd_subtype == 0 as DWORD {
        dev_subtype = DIDEVTYPEKEYBOARD_PCENH as DWORD;
    } else if kbd_type == 7 as DWORD && kbd_subtype == 2 as DWORD {
        dev_subtype = DIDEVTYPEKEYBOARD_JAPAN106 as DWORD;
    } else {
        WINE_FIXME!("Unknown keyboard type=%u, subtype=%u\n", kbd_type, kbd_subtype);
        dev_subtype = DIDEVTYPEKEYBOARD_PCENH as DWORD;
    }
    return dev_subtype;
}

unsafe extern "C" fn fill_keyboard_dideviceinstanceA(mut lpddi: LPDIDEVICEINSTANCEA, mut version: DWORD, mut subtype: DWORD) {
    let mut dwSize: DWORD = 0;
    let mut ddi: DIDEVICEINSTANCEA = core::mem::zeroed();
    dwSize = (*lpddi).dwSize;

    memset(lpddi as *mut core::ffi::c_void, 0 as core::ffi::c_int, dwSize as usize);
    memset(core::ptr::addr_of_mut!(ddi) as *mut core::ffi::c_void, 0 as core::ffi::c_int, core::mem::size_of::<DIDEVICEINSTANCEA>() as usize);

    ddi.dwSize = dwSize;
    ddi.guidInstance = GUID_SysKeyboard;
    ddi.guidProduct = GUID_SysKeyboard;
    if version >= 0x0800 as DWORD {
        ddi.dwDevType = (DI8DEVTYPE_KEYBOARD as DWORD) | (subtype << 8);
    } else {
        ddi.dwDevType = (DIDEVTYPE_KEYBOARD as DWORD) | (subtype << 8);
    }
    strcpy(core::ptr::addr_of_mut!(ddi.tszInstanceName) as *mut i8, b"Keyboard\0".as_ptr() as *const i8);
    strcpy(core::ptr::addr_of_mut!(ddi.tszProductName) as *mut i8, b"Wine Keyboard\0".as_ptr() as *const i8);

    memcpy(
        lpddi as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(ddi) as *const core::ffi::c_void,
        if (dwSize as usize) < core::mem::size_of::<DIDEVICEINSTANCEA>() {
            dwSize as usize
        } else {
            core::mem::size_of::<DIDEVICEINSTANCEA>() as usize
        },
    );
}

unsafe extern "C" fn fill_keyboard_dideviceinstanceW(lpddi: LPDIDEVICEINSTANCEW, version: DWORD, subtype: DWORD) {
    let mut dwSize: DWORD;
    let mut ddi: DIDEVICEINSTANCEW = core::mem::zeroed();

    dwSize = (*lpddi).dwSize;

    memset(lpddi as *mut core::ffi::c_void, 0, dwSize as size_t);
    memset(core::ptr::addr_of_mut!(ddi) as *mut core::ffi::c_void, 0, core::mem::size_of::<DIDEVICEINSTANCEW>() as size_t);

    ddi.dwSize = dwSize;
    ddi.guidInstance = GUID_SysKeyboard; /* DInput's GUID */
    ddi.guidProduct = GUID_SysKeyboard;
    if version >= 0x0800 {
        ddi.dwDevType = DI8DEVTYPE_KEYBOARD | (subtype << 8);
    } else {
        ddi.dwDevType = DIDEVTYPE_KEYBOARD | (subtype << 8);
    }
    MultiByteToWideChar(
        CP_ACP,
        0,
        b"Keyboard\0" as *const u8 as LPCSTR,
        -1,
        core::ptr::addr_of_mut!(ddi.tszInstanceName) as LPWSTR,
        (MAX_PATH as i32),
    );
    MultiByteToWideChar(
        CP_ACP,
        0,
        b"Wine Keyboard\0" as *const u8 as LPCSTR,
        -1,
        core::ptr::addr_of_mut!(ddi.tszProductName) as LPWSTR,
        (MAX_PATH as i32),
    );

    memcpy(
        lpddi as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(ddi) as *const core::ffi::c_void,
        if (dwSize as usize) < core::mem::size_of::<DIDEVICEINSTANCEW>() {
            dwSize as size_t
        } else {
            core::mem::size_of::<DIDEVICEINSTANCEW>() as size_t
        },
    );
}

unsafe extern "C" fn keyboarddev_enum_deviceA(dwDevType: DWORD, dwFlags: DWORD, lpddi: LPDIDEVICEINSTANCEA, version: DWORD, id: i32) -> HRESULT {
    if id != 0 as i32 {
        return E_FAIL;
    }

    if dwFlags & DIEDFL_FORCEFEEDBACK as DWORD != 0 {
        return S_FALSE;
    }

    if dwDevType == 0 as DWORD
        || (dwDevType == DIDEVTYPE_KEYBOARD as DWORD && version < 0x0800 as DWORD)
        || ((dwDevType == DI8DEVCLASS_KEYBOARD as DWORD || dwDevType == DI8DEVTYPE_KEYBOARD as DWORD)
            && version >= 0x0800 as DWORD)
    {
        TRACE!("Enumerating the Keyboard device\n");

        fill_keyboard_dideviceinstanceA(lpddi, version, get_keyboard_subtype());

        return S_OK;
    }

    return S_FALSE;
}

unsafe extern "C" fn keyboarddev_enum_deviceW(mut dwDevType: DWORD, mut dwFlags: DWORD, mut lpddi: LPDIDEVICEINSTANCEW, mut version: DWORD, mut id: ::core::ffi::c_int) -> HRESULT {
    if id != 0 {
        return E_FAIL;
    }
    if (dwFlags & DIEDFL_FORCEFEEDBACK) != 0 {
        return S_FALSE;
    }
    if (dwDevType == 0)
        || ((dwDevType == DIDEVTYPE_KEYBOARD) && (version < 0x0800))
        || (((dwDevType == DI8DEVCLASS_KEYBOARD) || (dwDevType == DI8DEVTYPE_KEYBOARD)) && (version >= 0x0800))
    {
        fill_keyboard_dideviceinstanceW(lpddi, version, get_keyboard_subtype());
        return S_OK;
    }
    return S_FALSE;
}

unsafe extern "C" fn alloc_device(rguid: *const GUID, dinput: *mut IDirectInputImpl) -> *mut SysKeyboardImpl {
    let mut newDevice: *mut SysKeyboardImpl = core::ptr::null_mut::<SysKeyboardImpl>();
    let mut df: LPDIDATAFORMAT = core::ptr::null_mut::<_DIDATAFORMAT>();
    let mut i: core::ffi::c_int = 0;
    let mut idx: core::ffi::c_int = 0;
    newDevice = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<SysKeyboardImpl>() as SIZE_T,
    ) as *mut SysKeyboardImpl;
    (*newDevice).base.IDirectInputDevice8A_iface.lpVtbl =
        core::ptr::addr_of!(SysKeyboardAvt) as *const IDirectInputDevice8AVtbl;
    (*newDevice).base.IDirectInputDevice8W_iface.lpVtbl =
        core::ptr::addr_of!(SysKeyboardWvt) as *const IDirectInputDevice8WVtbl;
    (*newDevice).base.ref_ = 1 as LONG;
    memcpy(
        core::ptr::addr_of_mut!((*newDevice).base.guid) as *mut core::ffi::c_void,
        rguid as *const core::ffi::c_void,
        core::mem::size_of::<GUID>() as size_t,
    );
    (*newDevice).base.dinput = dinput;
    (*newDevice).base.event_proc = Some(
        KeyboardCallback
            as unsafe extern "C" fn(LPDIRECTINPUTDEVICE8A, WPARAM, LPARAM) -> core::ffi::c_int,
    ) as DI_EVENT_PROC;
    InitializeCriticalSection(core::ptr::addr_of_mut!((*newDevice).base.crit));
    (*(*newDevice).base.crit.DebugInfo).__bindgen_anon_1.Spare[0 as usize] =
        concat!(file!(), ": SysKeyboardImpl*->base.crit\0").as_ptr() as DWORD_PTR;
    (*newDevice).subtype = get_keyboard_subtype();

    /* Create copy of default data format */
    'failed: {
        df = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            c_dfDIKeyboard.dwSize as SIZE_T,
        ) as LPDIDATAFORMAT;
        if df.is_null() {
            break 'failed;
        }
        memcpy(
            df as *mut core::ffi::c_void,
            core::ptr::addr_of!(c_dfDIKeyboard) as *const core::ffi::c_void,
            c_dfDIKeyboard.dwSize as size_t,
        );
        (*df).rgodf = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            (*df).dwNumObjs.wrapping_mul((*df).dwObjSize) as SIZE_T,
        ) as LPDIOBJECTDATAFORMAT;
        if (*df).rgodf.is_null() {
            break 'failed;
        }

        i = 0;
        while (i as DWORD) < (*df).dwNumObjs {
            let mut buf: [core::ffi::c_char; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
            let mut dik_code: BYTE = 0;

            if GetKeyNameTextA(
                ((i & 0x7f) << 16)
                    | ((i & 0x80) << 17),
                core::ptr::addr_of_mut!(buf) as LPSTR,
                core::mem::size_of::<[core::ffi::c_char; (MAX_PATH) as _]>()
                    as core::ffi::c_int,
            ) != 0
            {
                dik_code = map_dik_code(
                    i as DWORD,
                    0 as DWORD,
                    (*newDevice).subtype,
                );
                memcpy(
                    (*df).rgodf.offset(idx as isize) as *mut core::ffi::c_void,
                    c_dfDIKeyboard.rgodf.offset(dik_code as isize) as *const core::ffi::c_void,
                    (*df).dwObjSize as size_t,
                );
                let fresh15 = idx;
                idx = idx + 1;
                (*(*df).rgodf.offset(fresh15 as isize)).dwType =
                    (DIDFT_MAKEINSTANCE!(dik_code as DWORD) | (DIDFT_PSHBUTTON as i32)) as DWORD;
            }
            i += 1;
        }
        (*df).dwNumObjs = idx as DWORD;

        (*newDevice).base.data_format.wine_df = df;
        IDirectInput_AddRef!(core::ptr::addr_of_mut!(
            (*(*newDevice).base.dinput).IDirectInput7A_iface
        ));

        EnterCriticalSection(core::ptr::addr_of_mut!((*dinput).crit));
        list_add_tail(
            core::ptr::addr_of_mut!((*dinput).devices_list),
            core::ptr::addr_of_mut!((*newDevice).base.entry),
        );
        LeaveCriticalSection(core::ptr::addr_of_mut!((*dinput).crit));

        return newDevice;
    }

    /* failed: */
    if !df.is_null() {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            (*df).rgodf as PVOID,
        );
    }
    HeapFree(GetProcessHeap(), 0 as DWORD, df as PVOID);
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        newDevice as PVOID,
    );
    return core::ptr::null_mut();
}

unsafe extern "C" fn keyboarddev_create_device(dinput: *mut IDirectInputImpl, rguid: REFGUID, riid: REFIID, pdev: *mut LPVOID, mut unicode: i32) -> HRESULT {
    *pdev = core::ptr::null_mut();

    if IsEqualGUID!(core::ptr::addr_of!(GUID_SysKeyboard), rguid) != false {
        let mut This: *mut SysKeyboardImpl;

        if riid.is_null() {
            /* nothing */
        } else if IsEqualGUID!(core::ptr::addr_of!(IID_IDirectInputDeviceA), riid) != false
            || IsEqualGUID!(core::ptr::addr_of!(IID_IDirectInputDevice2A), riid) != false
            || IsEqualGUID!(core::ptr::addr_of!(IID_IDirectInputDevice7A), riid) != false
            || IsEqualGUID!(core::ptr::addr_of!(IID_IDirectInputDevice8A), riid) != false
        {
            unicode = 0;
        } else if IsEqualGUID!(core::ptr::addr_of!(IID_IDirectInputDeviceW), riid) != false
            || IsEqualGUID!(core::ptr::addr_of!(IID_IDirectInputDevice2W), riid) != false
            || IsEqualGUID!(core::ptr::addr_of!(IID_IDirectInputDevice7W), riid) != false
            || IsEqualGUID!(core::ptr::addr_of!(IID_IDirectInputDevice8W), riid) != false
        {
            unicode = 1;
        } else {
            WARN!("no interface\n");
            return DIERR_NOINTERFACE;
        }

        This = alloc_device(rguid, dinput);

        if This.is_null() {
            return DIERR_OUTOFMEMORY;
        }

        if unicode != 0 {
            *pdev = core::ptr::addr_of_mut!((*This).base.IDirectInputDevice8W_iface) as LPVOID;
        } else {
            *pdev = core::ptr::addr_of_mut!((*This).base.IDirectInputDevice8A_iface) as LPVOID;
        }

        return DI_OK;
    }

    return DIERR_DEVICENOTREG;
}

unsafe extern "system" fn SysKeyboardWImpl_GetDeviceState(iface: LPDIRECTINPUTDEVICE8W, len: DWORD, ptr: LPVOID) -> HRESULT {
    let mut This: *mut SysKeyboardImpl = impl_from_IDirectInputDevice8W(iface as *mut IDirectInputDevice8W);

    if (*This).base.acquired == 0 {
        return DIERR_NOTACQUIRED;
    }

    if len != (*(*This).base.data_format.user_df).dwDataSize {
        return DIERR_INVALIDPARAM;
    }

    check_dinput_events();

    EnterCriticalSection(&raw mut (*This).base.crit);

    if ((TRACE_ON!(__wine_dbch_dinput)) != 0) {
        let mut i: i32;
        i = 0;
        while i < (WINE_DINPUT_KEYBOARD_MAX_KEYS as i32) {
            if (*This).DInputKeyState[i as usize] != 0x00 {
                TRACE!(" - %02X: %02x\n", i, (*This).DInputKeyState[i as usize] as i32);
            }
            i += 1;
        }
    }

    fill_DataFormat(
        ptr as *mut core::ffi::c_void,
        len,
        &raw mut (*This).DInputKeyState as *mut BYTE as *const core::ffi::c_void,
        &raw mut (*This).base.data_format,
    );
    LeaveCriticalSection(&raw mut (*This).base.crit);

    return DI_OK;
}

unsafe extern "system" fn SysKeyboardAImpl_GetDeviceState(
    mut iface: LPDIRECTINPUTDEVICE8A,
    mut len: DWORD,
    mut ptr: LPVOID,
) -> HRESULT {
    let mut This: *mut SysKeyboardImpl =
        impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    return SysKeyboardWImpl_GetDeviceState(
        IDirectInputDevice8W_from_impl(This) as LPDIRECTINPUTDEVICE8W,
        len,
        ptr,
    );
}

unsafe extern "system" fn SysKeyboardWImpl_GetCapabilities(iface: LPDIRECTINPUTDEVICE8W, lpDIDevCaps: LPDIDEVCAPS) -> HRESULT {
    let mut This: *mut SysKeyboardImpl = impl_from_IDirectInputDevice8W(iface as *mut IDirectInputDevice8W);
    let mut devcaps: DIDEVCAPS = core::mem::zeroed();

    if ((*lpDIDevCaps).dwSize as usize != core::mem::size_of::<DIDEVCAPS>())
        && ((*lpDIDevCaps).dwSize as usize != core::mem::size_of::<DIDEVCAPS_DX3>())
    {
        WARN!("invalid parameter\n");
        return DIERR_INVALIDPARAM as HRESULT;
    }

    devcaps.dwSize = (*lpDIDevCaps).dwSize;
    devcaps.dwFlags = (DIDC_ATTACHED | DIDC_EMULATED) as DWORD;
    if (*(*This).base.dinput).dwVersion >= 0x0800 as DWORD {
        devcaps.dwDevType = DI8DEVTYPE_KEYBOARD as DWORD | ((*This).subtype << 8);
    } else {
        devcaps.dwDevType = DIDEVTYPE_KEYBOARD as DWORD | ((*This).subtype << 8);
    }
    devcaps.dwAxes = 0 as DWORD;
    devcaps.dwButtons = (*(*This).base.data_format.wine_df).dwNumObjs;
    devcaps.dwPOVs = 0 as DWORD;
    devcaps.dwFFSamplePeriod = 0 as DWORD;
    devcaps.dwFFMinTimeResolution = 0 as DWORD;
    devcaps.dwFirmwareRevision = 100 as DWORD;
    devcaps.dwHardwareRevision = 100 as DWORD;
    devcaps.dwFFDriverVersion = 0 as DWORD;

    memcpy(
        lpDIDevCaps as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(devcaps) as *const core::ffi::c_void,
        (*lpDIDevCaps).dwSize as usize,
    );

    return DI_OK as HRESULT;
}

unsafe extern "system" fn SysKeyboardAImpl_GetCapabilities(iface: LPDIRECTINPUTDEVICE8A, lpDIDevCaps: LPDIDEVCAPS) -> HRESULT {
    let This: *mut SysKeyboardImpl = impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    return SysKeyboardWImpl_GetCapabilities(IDirectInputDevice8W_from_impl(This) as LPDIRECTINPUTDEVICE8W, lpDIDevCaps);
}

unsafe extern "C" fn map_dik_to_scan(mut dik_code: DWORD, mut subtype: DWORD) -> DWORD {
    if dik_code == DIK_PAUSE || dik_code == DIK_NUMLOCK {
        dik_code ^= 0x80;
    }
    if subtype == DIDEVTYPEKEYBOARD_JAPAN106 {
        match dik_code {
            DIK_CIRCUMFLEX => {
                dik_code = 0x0d;
            }
            DIK_AT => {
                dik_code = 0x1a;
            }
            DIK_LBRACKET => {
                dik_code = 0x1b;
            }
            DIK_COLON => {
                dik_code = 0x28;
            }
            DIK_KANJI => {
                dik_code = 0x29;
            }
            DIK_RBRACKET => {
                dik_code = 0x2b;
            }
            DIK_BACKSLASH => {
                dik_code = 0x73;
            }
            _ => {}
        }
    }

    return dik_code;
}

unsafe extern "system" fn SysKeyboardAImpl_GetObjectInfo(iface: LPDIRECTINPUTDEVICE8A, pdidoi: LPDIDEVICEOBJECTINSTANCEA, dwObj: DWORD, dwHow: DWORD) -> HRESULT {
    let mut This: *mut SysKeyboardImpl = impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    let mut res: HRESULT = 0;
    let mut scan: LONG = 0;

    res = IDirectInputDevice2AImpl_GetObjectInfo(iface, pdidoi, dwObj, dwHow);
    if res != DI_OK {
        return res;
    }

    scan = map_dik_to_scan((DIDFT_GETINSTANCE!((*pdidoi).dwType)) as DWORD, (*This).subtype) as LONG;
    if GetKeyNameTextA(
        ((scan & 0x80) << 17)
            | ((scan & 0x7f) << 16),
        core::ptr::addr_of_mut!((*pdidoi).tszName) as LPSTR,
        core::mem::size_of::<[CHAR; 260]>() as i32,
    ) == 0
    {
        return DIERR_OBJECTNOTFOUND;
    }

    _dump_OBJECTINSTANCEA(pdidoi as *const DIDEVICEOBJECTINSTANCEA);
    return res;
}
unsafe extern "system" fn SysKeyboardWImpl_GetObjectInfo(
    mut iface: LPDIRECTINPUTDEVICE8W,
    mut pdidoi: LPDIDEVICEOBJECTINSTANCEW,
    mut dwObj: DWORD,
    mut dwHow: DWORD,
) -> HRESULT {
    let mut This: *mut SysKeyboardImpl = impl_from_IDirectInputDevice8W(iface as *mut IDirectInputDevice8W);
    let mut res: HRESULT = 0;
    let mut scan: LONG = 0;

    res = IDirectInputDevice2WImpl_GetObjectInfo(iface, pdidoi, dwObj, dwHow);
    if res != DI_OK {
        return res;
    }

    scan = map_dik_to_scan(
        (DIDFT_GETINSTANCE!((*pdidoi).dwType)) as DWORD,
        (*This).subtype,
    ) as LONG;
    if GetKeyNameTextW(
        ((scan & 0x80) << 17)
            | ((scan & 0x7f) << 16),
        core::ptr::addr_of_mut!((*pdidoi).tszName) as LPWSTR,
        ARRAY_SIZE!((*pdidoi).tszName) as core::ffi::c_int,
    ) == 0
    {
        return DIERR_OBJECTNOTFOUND;
    }

    _dump_OBJECTINSTANCEW(pdidoi as *const DIDEVICEOBJECTINSTANCEW);
    return res;
}

unsafe extern "system" fn SysKeyboardAImpl_GetDeviceInfo(
    iface: LPDIRECTINPUTDEVICE8A,
    pdidi: LPDIDEVICEINSTANCEA,
) -> HRESULT {
    let mut This: *mut SysKeyboardImpl = impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);

    TRACE!("(this=%p,%p)\n", This, pdidi);

    fill_keyboard_dideviceinstanceA(pdidi, (*(*This).base.dinput).dwVersion, (*This).subtype);

    return DI_OK;
}

unsafe extern "system" fn SysKeyboardWImpl_GetDeviceInfo(iface: LPDIRECTINPUTDEVICE8W, pdidi: LPDIDEVICEINSTANCEW) -> HRESULT {
    let mut This: *mut SysKeyboardImpl = impl_from_IDirectInputDevice8W(iface);

    if (*pdidi).dwSize as usize != core::mem::size_of::<DIDEVICEINSTANCEW>() {
        WARN!(" dinput3 not supported yet...\n");
        return DI_OK;
    }

    fill_keyboard_dideviceinstanceW(pdidi, (*(*This).base.dinput).dwVersion, (*This).subtype);

    return DI_OK;
}
unsafe extern "system" fn SysKeyboardWImpl_GetProperty(
    iface: LPDIRECTINPUTDEVICE8W,
    rguid: REFGUID,
    pdiph: LPDIPROPHEADER,
) -> HRESULT {
    let This: *mut SysKeyboardImpl = impl_from_IDirectInputDevice8W(iface);

    _dump_DIPROPHEADER(pdiph);

    if !IS_DIPROP!(rguid) {
        return DI_OK;
    }

    let sw_key: DWORD_PTR = LOWORD!(rguid) as DWORD_PTR;
    match sw_key {
        _ if sw_key == (DIPROP_KEYNAME as DWORD_PTR) => {
            let mut hr: HRESULT = 0;
            let ps: LPDIPROPSTRING = pdiph as LPDIPROPSTRING;
            let mut didoi: DIDEVICEOBJECTINSTANCEW = core::mem::zeroed();

            if (*pdiph).dwSize as usize != core::mem::size_of::<DIPROPSTRING>() as usize {
                return DIERR_INVALIDPARAM;
            }

            didoi.dwSize = core::mem::size_of::<DIDEVICEOBJECTINSTANCEW>() as DWORD;

            hr = SysKeyboardWImpl_GetObjectInfo(
                iface,
                core::ptr::addr_of_mut!(didoi),
                (*ps).diph.dwObj,
                (*ps).diph.dwHow,
            );
            if hr == DI_OK {
                memcpy(
                    core::ptr::addr_of_mut!((*ps).wsz) as *mut WCHAR as *mut core::ffi::c_void,
                    core::ptr::addr_of!(didoi.tszName) as *const WCHAR as *const core::ffi::c_void,
                    core::mem::size_of::<[WCHAR; 260]>() as usize,
                );
            }
            return hr;
        }
        _ if sw_key == (DIPROP_RANGE as DWORD_PTR) => {
            return DIERR_UNSUPPORTED;
        }
        _ => {
            return IDirectInputDevice2AImpl_GetProperty(
                IDirectInputDevice8A_from_impl(This) as LPDIRECTINPUTDEVICE8A,
                rguid,
                pdiph,
            );
        }
    }
    #[allow(unreachable_code)]
    return DI_OK;
}

unsafe extern "system" fn SysKeyboardAImpl_GetProperty(
    iface: LPDIRECTINPUTDEVICE8A,
    rguid: *const GUID,
    pdiph: LPDIPROPHEADER,
) -> HRESULT {
    let mut This: *mut SysKeyboardImpl = impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    return SysKeyboardWImpl_GetProperty(IDirectInputDevice8W_from_impl(This) as LPDIRECTINPUTDEVICE8W, rguid, pdiph);
}

unsafe extern "system" fn SysKeyboardWImpl_Acquire(iface: LPDIRECTINPUTDEVICE8W) -> HRESULT {
    let This: *mut SysKeyboardImpl = impl_from_IDirectInputDevice8W(iface);
    let mut res: HRESULT;

    res = IDirectInputDevice2WImpl_Acquire(iface);
    if res == DI_OK {
        memset(
            core::ptr::addr_of_mut!((*This).DInputKeyState) as *mut BYTE as *mut core::ffi::c_void,
            0,
            core::mem::size_of::<[BYTE; 256]>() as size_t,
        );
    }

    return res;
}

unsafe extern "system" fn SysKeyboardAImpl_Acquire(iface: LPDIRECTINPUTDEVICE8A) -> HRESULT {
    let mut This: *mut SysKeyboardImpl = impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    return SysKeyboardWImpl_Acquire(IDirectInputDevice8W_from_impl(This) as LPDIRECTINPUTDEVICE8W);
}

unsafe extern "system" fn SysKeyboardWImpl_BuildActionMap(
    iface: LPDIRECTINPUTDEVICE8W,
    lpdiaf: LPDIACTIONFORMATW,
    lpszUserName: LPCWSTR,
    dwFlags: DWORD,
) -> HRESULT {
    let mut This: *mut SysKeyboardImpl = impl_from_IDirectInputDevice8W(iface as *mut IDirectInputDevice8W);

    return _build_action_map(
        iface,
        lpdiaf,
        lpszUserName,
        dwFlags,
        DIKEYBOARD_MASK as DWORD,
        core::ptr::addr_of!(c_dfDIKeyboard),
    );
}

unsafe extern "system" fn SysKeyboardAImpl_BuildActionMap(
    iface: LPDIRECTINPUTDEVICE8A,
    lpdiaf: LPDIACTIONFORMATA,
    lpszUserName: LPCSTR,
    dwFlags: DWORD,
) -> HRESULT {
    let mut This: *mut SysKeyboardImpl = impl_from_IDirectInputDevice8A(iface);
    let mut diafW: DIACTIONFORMATW = core::mem::zeroed();
    let mut hr: HRESULT;
    let mut lpszUserNameW: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    let mut username_size: core::ffi::c_int;

    diafW.rgoAction = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        (core::mem::size_of::<DIACTIONW>().wrapping_mul((*lpdiaf).dwNumActions as usize)) as SIZE_T,
    ) as LPDIACTIONW;
    _copy_diactionformatAtoW(core::ptr::addr_of_mut!(diafW), lpdiaf);

    if !lpszUserName.is_null() {
        username_size = MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            lpszUserName,
            -1,
            core::ptr::null_mut::<WCHAR>(),
            0,
        );
        lpszUserNameW = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            (core::mem::size_of::<WCHAR>().wrapping_mul(username_size as usize)) as SIZE_T,
        ) as *mut WCHAR;
        MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            lpszUserName,
            -1,
            lpszUserNameW,
            username_size,
        );
    }

    hr = SysKeyboardWImpl_BuildActionMap(
        core::ptr::addr_of_mut!((*This).base.IDirectInputDevice8W_iface),
        core::ptr::addr_of_mut!(diafW),
        lpszUserNameW as LPCWSTR,
        dwFlags,
    );

    _copy_diactionformatWtoA(lpdiaf, core::ptr::addr_of_mut!(diafW));
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        diafW.rgoAction as LPVOID,
    );
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        lpszUserNameW as LPVOID,
    );

    return hr;
}

unsafe extern "system" fn SysKeyboardWImpl_SetActionMap(
    iface: LPDIRECTINPUTDEVICE8W,
    lpdiaf: LPDIACTIONFORMATW,
    lpszUserName: LPCWSTR,
    dwFlags: DWORD,
) -> HRESULT {
    let This: *mut SysKeyboardImpl = impl_from_IDirectInputDevice8W(iface as *mut IDirectInputDevice8W);

    return _set_action_map(
        iface,
        lpdiaf,
        lpszUserName,
        dwFlags,
        core::ptr::addr_of!(c_dfDIKeyboard),
    );
}

unsafe extern "system" fn SysKeyboardAImpl_SetActionMap(iface: LPDIRECTINPUTDEVICE8A, lpdiaf: LPDIACTIONFORMATA, lpszUserName: LPCSTR, dwFlags: DWORD) -> HRESULT {
    let mut This: *mut SysKeyboardImpl = impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    let mut diafW: DIACTIONFORMATW = core::mem::zeroed::<DIACTIONFORMATW>();
    let mut hr: HRESULT;
    let mut lpszUserNameW: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    let mut username_size: ::core::ffi::c_int;

    diafW.rgoAction = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        (core::mem::size_of::<DIACTIONW>() as SIZE_T).wrapping_mul((*lpdiaf).dwNumActions as SIZE_T),
    ) as *mut DIACTIONW;
    _copy_diactionformatAtoW(core::ptr::addr_of_mut!(diafW), lpdiaf);

    if !lpszUserName.is_null() {
        username_size = MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            lpszUserName,
            (-1) as ::core::ffi::c_int,
            core::ptr::null_mut::<WCHAR>(),
            0 as ::core::ffi::c_int,
        );
        lpszUserNameW = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            (core::mem::size_of::<WCHAR>() as SIZE_T).wrapping_mul(username_size as SIZE_T),
        ) as *mut WCHAR;
        MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            lpszUserName,
            (-1) as ::core::ffi::c_int,
            lpszUserNameW as LPWSTR,
            username_size,
        );
    }

    hr = SysKeyboardWImpl_SetActionMap(
        core::ptr::addr_of_mut!((*This).base.IDirectInputDevice8W_iface),
        core::ptr::addr_of_mut!(diafW),
        lpszUserNameW as LPCWSTR,
        dwFlags,
    );

    (*lpdiaf).dwCRC = diafW.dwCRC;

    HeapFree(GetProcessHeap(), 0 as DWORD, diafW.rgoAction as LPVOID);
    HeapFree(GetProcessHeap(), 0 as DWORD, lpszUserNameW as LPVOID);

    return hr;
}

// COM vtable tables — generated from keyboard.c by vtbl_gen.py
/// keyboard.c:701 `static const IDirectInputDevice8AVtbl SysKeyboardAvt`  (slot order: rust:types_amd64.rs)
pub static SysKeyboardAvt: IDirectInputDevice8AVtbl = IDirectInputDevice8AVtbl {
    QueryInterface: Some(IDirectInputDevice2AImpl_QueryInterface),
    AddRef: Some(IDirectInputDevice2AImpl_AddRef),
    Release: Some(IDirectInputDevice2AImpl_Release),
    GetCapabilities: Some(SysKeyboardAImpl_GetCapabilities),
    EnumObjects: Some(IDirectInputDevice2AImpl_EnumObjects),
    GetProperty: Some(SysKeyboardAImpl_GetProperty),
    SetProperty: Some(IDirectInputDevice2AImpl_SetProperty),
    Acquire: Some(SysKeyboardAImpl_Acquire),
    Unacquire: Some(IDirectInputDevice2AImpl_Unacquire),
    GetDeviceState: Some(SysKeyboardAImpl_GetDeviceState),
    GetDeviceData: Some(IDirectInputDevice2AImpl_GetDeviceData),
    SetDataFormat: Some(IDirectInputDevice2AImpl_SetDataFormat),
    SetEventNotification: Some(IDirectInputDevice2AImpl_SetEventNotification),
    SetCooperativeLevel: Some(IDirectInputDevice2AImpl_SetCooperativeLevel),
    GetObjectInfo: Some(SysKeyboardAImpl_GetObjectInfo),
    GetDeviceInfo: Some(SysKeyboardAImpl_GetDeviceInfo),
    RunControlPanel: Some(IDirectInputDevice2AImpl_RunControlPanel),
    Initialize: Some(IDirectInputDevice2AImpl_Initialize),
    CreateEffect: Some(IDirectInputDevice2AImpl_CreateEffect),
    EnumEffects: Some(IDirectInputDevice2AImpl_EnumEffects),
    GetEffectInfo: Some(IDirectInputDevice2AImpl_GetEffectInfo),
    GetForceFeedbackState: Some(IDirectInputDevice2AImpl_GetForceFeedbackState),
    SendForceFeedbackCommand: Some(IDirectInputDevice2AImpl_SendForceFeedbackCommand),
    EnumCreatedEffectObjects: Some(IDirectInputDevice2AImpl_EnumCreatedEffectObjects),
    Escape: Some(IDirectInputDevice2AImpl_Escape),
    Poll: Some(IDirectInputDevice2AImpl_Poll),
    SendDeviceData: Some(IDirectInputDevice2AImpl_SendDeviceData),
    EnumEffectsInFile: Some(IDirectInputDevice7AImpl_EnumEffectsInFile),
    WriteEffectToFile: Some(IDirectInputDevice7AImpl_WriteEffectToFile),
    BuildActionMap: Some(SysKeyboardAImpl_BuildActionMap),
    SetActionMap: Some(SysKeyboardAImpl_SetActionMap),
    GetImageInfo: Some(IDirectInputDevice8AImpl_GetImageInfo),
};

/// keyboard.c:737 `static const IDirectInputDevice8WVtbl SysKeyboardWvt`  (slot order: rust:types_amd64.rs)
pub static SysKeyboardWvt: IDirectInputDevice8WVtbl = IDirectInputDevice8WVtbl {
    QueryInterface: Some(IDirectInputDevice2WImpl_QueryInterface),
    AddRef: Some(IDirectInputDevice2WImpl_AddRef),
    Release: Some(IDirectInputDevice2WImpl_Release),
    GetCapabilities: Some(SysKeyboardWImpl_GetCapabilities),
    EnumObjects: Some(IDirectInputDevice2WImpl_EnumObjects),
    GetProperty: Some(SysKeyboardWImpl_GetProperty),
    SetProperty: Some(IDirectInputDevice2WImpl_SetProperty),
    Acquire: Some(SysKeyboardWImpl_Acquire),
    Unacquire: Some(IDirectInputDevice2WImpl_Unacquire),
    GetDeviceState: Some(SysKeyboardWImpl_GetDeviceState),
    GetDeviceData: Some(IDirectInputDevice2WImpl_GetDeviceData),
    SetDataFormat: Some(IDirectInputDevice2WImpl_SetDataFormat),
    SetEventNotification: Some(IDirectInputDevice2WImpl_SetEventNotification),
    SetCooperativeLevel: Some(IDirectInputDevice2WImpl_SetCooperativeLevel),
    GetObjectInfo: Some(SysKeyboardWImpl_GetObjectInfo),
    GetDeviceInfo: Some(SysKeyboardWImpl_GetDeviceInfo),
    RunControlPanel: Some(IDirectInputDevice2WImpl_RunControlPanel),
    Initialize: Some(IDirectInputDevice2WImpl_Initialize),
    CreateEffect: Some(IDirectInputDevice2WImpl_CreateEffect),
    EnumEffects: Some(IDirectInputDevice2WImpl_EnumEffects),
    GetEffectInfo: Some(IDirectInputDevice2WImpl_GetEffectInfo),
    GetForceFeedbackState: Some(IDirectInputDevice2WImpl_GetForceFeedbackState),
    SendForceFeedbackCommand: Some(IDirectInputDevice2WImpl_SendForceFeedbackCommand),
    EnumCreatedEffectObjects: Some(IDirectInputDevice2WImpl_EnumCreatedEffectObjects),
    Escape: Some(IDirectInputDevice2WImpl_Escape),
    Poll: Some(IDirectInputDevice2WImpl_Poll),
    SendDeviceData: Some(IDirectInputDevice2WImpl_SendDeviceData),
    EnumEffectsInFile: Some(IDirectInputDevice7WImpl_EnumEffectsInFile),
    WriteEffectToFile: Some(IDirectInputDevice7WImpl_WriteEffectToFile),
    BuildActionMap: Some(SysKeyboardWImpl_BuildActionMap),
    SetActionMap: Some(SysKeyboardWImpl_SetActionMap),
    GetImageInfo: Some(IDirectInputDevice8WImpl_GetImageInfo),
};
