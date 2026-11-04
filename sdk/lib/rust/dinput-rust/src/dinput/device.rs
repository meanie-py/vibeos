use crate::prelude::*;
use core::ffi::c_void;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_3 {
    pub guid: *const GUID,
    pub name: *const ::core::ffi::c_char,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_4 {
    pub mask: DWORD,
    pub name: *const ::core::ffi::c_char,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_5 {
    pub mask: DWORD,
    pub name: *const ::core::ffi::c_char,
}

// ---- file-scope preamble (translated) ----
// WINE_DEFAULT_DEBUG_CHANNEL(dinput); -- debug channel registration macro, no Rust declaration emitted here.
// The trailing fragment (`if (guid == NULL) ... debugstr_guid(guid); }`) is the body of a function, not file-scope
// declarations, so it is not emitted in this preamble. It belongs to the function translation pass.

// WM_USER
pub const WM_WINE_NOTIFY_ACTIVITY: u32 = 0x0400;
unsafe extern "C" fn impl_from_IDirectInputDevice8A(
    iface: *mut IDirectInputDevice8A,
) -> *mut IDirectInputDeviceImpl {
    return (iface as *mut u8).sub(core::mem::offset_of!(IDirectInputDeviceImpl, IDirectInputDevice8A_iface))
        as *mut IDirectInputDeviceImpl;
}

unsafe extern "C" fn impl_from_IDirectInputDevice8W(
    iface: *mut IDirectInputDevice8W,
) -> *mut IDirectInputDeviceImpl {
    return (iface as usize).wrapping_sub(
        core::mem::offset_of!(IDirectInputDeviceImpl, IDirectInputDevice8W_iface),
    ) as *mut IDirectInputDeviceImpl;
}

unsafe extern "C" fn IDirectInputDevice8A_from_impl(This: *mut IDirectInputDeviceImpl) -> *mut IDirectInputDevice8A {
    return core::ptr::addr_of_mut!((*This).IDirectInputDevice8A_iface);
}

unsafe extern "C" fn IDirectInputDevice8W_from_impl(This: *mut IDirectInputDeviceImpl) -> *mut IDirectInputDevice8W {
    return core::ptr::addr_of_mut!((*This).IDirectInputDevice8W_iface);
}

unsafe extern "C" fn _dump_cooperativelevel_DI(dwFlags: DWORD) {
    if ((TRACE_ON!(__wine_dbch_dinput)) != 0) {
        let mut i: ::core::ffi::c_uint;
        let flags: [(DWORD, *const i8); 5] = [
            (DISCL_BACKGROUND as DWORD, b"DISCL_BACKGROUND\0".as_ptr() as *const i8),
            (DISCL_EXCLUSIVE as DWORD, b"DISCL_EXCLUSIVE\0".as_ptr() as *const i8),
            (DISCL_FOREGROUND as DWORD, b"DISCL_FOREGROUND\0".as_ptr() as *const i8),
            (DISCL_NONEXCLUSIVE as DWORD, b"DISCL_NONEXCLUSIVE\0".as_ptr() as *const i8),
            (0x10 as DWORD, b"DISCL_NOWINKEY\0".as_ptr() as *const i8), // DISCL_NOWINKEY
        ];
        i = 0 as ::core::ffi::c_uint;
        while (i as usize) < (ARRAY_SIZE!(flags) as usize) {
            if flags[i as usize].0 & dwFlags != 0 {
                TRACE!("%s ", flags[i as usize].1);
            }
            i = i.wrapping_add(1);
        }
    }
}

unsafe extern "C" fn _dump_ObjectDataFormat_flags(mut dwFlags: DWORD) {
    let mut i: ::core::ffi::c_uint;
    static mut flags: [C2RustUnnamed_4; 4] = [
        C2RustUnnamed_4 {
            mask: DIDOI_FFACTUATOR as DWORD,
            name: b"DIDOI_FFACTUATOR\0" as *const u8 as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_4 {
            mask: DIDOI_FFEFFECTTRIGGER as DWORD,
            name: b"DIDOI_FFEFFECTTRIGGER\0" as *const u8 as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_4 {
            mask: DIDOI_POLLED as DWORD,
            name: b"DIDOI_POLLED\0" as *const u8 as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_4 {
            mask: 0x10000 as DWORD,
            name: b"DIDOI_GUIDISUSAGE\0" as *const u8 as *const ::core::ffi::c_char,
        },
    ];
    if dwFlags == 0 {
        return;
    }
    WINE_TRACE!("Flags:");
    i = 0 as ::core::ffi::c_uint;
    while (i as usize) < ARRAY_SIZE!(flags) as usize {
        if flags[i as usize].mask & dwFlags != 0 {
            WINE_TRACE!(" %s", flags[i as usize].name);
        }
        i = i.wrapping_add(1);
    }
    match dwFlags & DIDOI_ASPECTMASK as DWORD {
        DIDOI_ASPECTACCEL => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_1,
                    __dbch_1,
                    b"dll/directx/wine/dinput/device.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"_dump_ObjectDataFormat_flags\0" as *const u8
                        as *const ::core::ffi::c_char,
                    118 as ::core::ffi::c_int,
                    concat!(" ", stringify!(DIDOI_ASPECTACCEL), "\0").as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        }
        DIDOI_ASPECTFORCE => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_2: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_2,
                    __dbch_2,
                    b"dll/directx/wine/dinput/device.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"_dump_ObjectDataFormat_flags\0" as *const u8
                        as *const ::core::ffi::c_char,
                    119 as ::core::ffi::c_int,
                    concat!(" ", stringify!(DIDOI_ASPECTFORCE), "\0").as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        }
        DIDOI_ASPECTPOSITION => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_3: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_3: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_3,
                    __dbch_3,
                    b"dll/directx/wine/dinput/device.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"_dump_ObjectDataFormat_flags\0" as *const u8
                        as *const ::core::ffi::c_char,
                    120 as ::core::ffi::c_int,
                    concat!(" ", stringify!(DIDOI_ASPECTPOSITION), "\0").as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        }
        DIDOI_ASPECTVELOCITY => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_4: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_4: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_4,
                    __dbch_4,
                    b"dll/directx/wine/dinput/device.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"_dump_ObjectDataFormat_flags\0" as *const u8
                        as *const ::core::ffi::c_char,
                    121 as ::core::ffi::c_int,
                    concat!(" ", stringify!(DIDOI_ASPECTVELOCITY), "\0").as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        }
        _ => {}
    }
}

unsafe extern "C" fn _dump_EnumObjects_flags(mut dwFlags: DWORD) {
    if TRACE_ON!(__wine_dbch_dinput) != 0 {
        let mut i: core::ffi::c_uint = 0;
        let mut type_0: DWORD = 0;
        let mut instance: DWORD = 0;
        static mut flags: [C2RustUnnamed_5; 13] = [
            C2RustUnnamed_5 {
                mask: DIDFT_RELAXIS as DWORD,
                name: concat!(stringify!(DIDFT_RELAXIS), "\0").as_ptr() as *const core::ffi::c_char,
            },
            C2RustUnnamed_5 {
                mask: DIDFT_ABSAXIS as DWORD,
                name: concat!(stringify!(DIDFT_ABSAXIS), "\0").as_ptr() as *const core::ffi::c_char,
            },
            C2RustUnnamed_5 {
                mask: DIDFT_PSHBUTTON as DWORD,
                name: concat!(stringify!(DIDFT_PSHBUTTON), "\0").as_ptr() as *const core::ffi::c_char,
            },
            C2RustUnnamed_5 {
                mask: DIDFT_TGLBUTTON as DWORD,
                name: concat!(stringify!(DIDFT_TGLBUTTON), "\0").as_ptr() as *const core::ffi::c_char,
            },
            C2RustUnnamed_5 {
                mask: DIDFT_POV as DWORD,
                name: concat!(stringify!(DIDFT_POV), "\0").as_ptr() as *const core::ffi::c_char,
            },
            C2RustUnnamed_5 {
                mask: DIDFT_COLLECTION as DWORD,
                name: concat!(stringify!(DIDFT_COLLECTION), "\0").as_ptr() as *const core::ffi::c_char,
            },
            C2RustUnnamed_5 {
                mask: DIDFT_NODATA as DWORD,
                name: concat!(stringify!(DIDFT_NODATA), "\0").as_ptr() as *const core::ffi::c_char,
            },
            C2RustUnnamed_5 {
                mask: DIDFT_FFACTUATOR as DWORD,
                name: concat!(stringify!(DIDFT_FFACTUATOR), "\0").as_ptr() as *const core::ffi::c_char,
            },
            C2RustUnnamed_5 {
                mask: DIDFT_FFEFFECTTRIGGER as DWORD,
                name: concat!(stringify!(DIDFT_FFEFFECTTRIGGER), "\0").as_ptr() as *const core::ffi::c_char,
            },
            C2RustUnnamed_5 {
                mask: DIDFT_OUTPUT as DWORD,
                name: concat!(stringify!(DIDFT_OUTPUT), "\0").as_ptr() as *const core::ffi::c_char,
            },
            C2RustUnnamed_5 {
                mask: DIDFT_VENDORDEFINED as DWORD,
                name: concat!(stringify!(DIDFT_VENDORDEFINED), "\0").as_ptr() as *const core::ffi::c_char,
            },
            C2RustUnnamed_5 {
                mask: DIDFT_ALIAS as DWORD,
                name: concat!(stringify!(DIDFT_ALIAS), "\0").as_ptr() as *const core::ffi::c_char,
            },
            C2RustUnnamed_5 {
                mask: DIDFT_OPTIONAL as DWORD,
                name: concat!(stringify!(DIDFT_OPTIONAL), "\0").as_ptr() as *const core::ffi::c_char,
            },
        ];
        type_0 = dwFlags & (0xFF0000FF as DWORD);
        instance = (dwFlags >> (8 as DWORD)) & (0xFFFF as DWORD);

        if type_0 == (DIDFT_ALL as DWORD) {
            TRACE!(" DIDFT_ALL");
        } else {
            i = 0 as core::ffi::c_uint;
            while (i as usize) < ARRAY_SIZE!(flags) as usize {
                if (flags[i as usize].mask & type_0) != 0 {
                    type_0 &= !flags[i as usize].mask;
                    TRACE!(" %s", flags[i as usize].name);
                }
                i = i.wrapping_add(1);
            }
            if type_0 != 0 {
                TRACE!(" (unhandled: %08x)", type_0);
            }
        }

        TRACE!(" / Instance: ");

        if instance
            == (((DIDFT_ANYINSTANCE as DWORD) >> (8 as DWORD))
                & (0xFFFF as DWORD))
        {
            TRACE!("DIDFT_ANYINSTANCE");
        } else {
            TRACE!("%3d", instance);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn _dump_DIPROPHEADER(diph: LPCDIPROPHEADER) {
    if ((TRACE_ON!(__wine_dbch_dinput)) != 0) {
        TRACE!("  - dwObj = 0x%08x\n", (*diph).dwObj);
        TRACE!(
            "  - dwHow = %s\n",
            if (*diph).dwHow == 0 as DWORD {
                b"DIPH_DEVICE\0" as *const u8 as *const i8
            } else if (*diph).dwHow == 1 as DWORD {
                b"DIPH_BYOFFSET\0" as *const u8 as *const i8
            } else if (*diph).dwHow == 2 as DWORD {
                b"DIPH_BYID\0" as *const u8 as *const i8
            } else {
                b"unknown\0" as *const u8 as *const i8
            }
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn _dump_OBJECTINSTANCEA(mut ddoi: *const DIDEVICEOBJECTINSTANCEA) {
    WINE_TRACE!("    - enumerating : %s ('%s') - %2d - 0x%08x - %s - 0x%x\n", debugstr_guid(core::ptr::addr_of!((*ddoi).guidType)), _dump_dinput_GUID(core::ptr::addr_of!((*ddoi).guidType)), (*ddoi).dwOfs, (*ddoi).dwType, core::ptr::addr_of!((*ddoi).tszName) as *const CHAR, (*ddoi).dwFlags);
}
#[no_mangle]
pub unsafe extern "C" fn _dump_OBJECTINSTANCEW(mut ddoi: *const DIDEVICEOBJECTINSTANCEW) {
    WINE_TRACE!("    - enumerating : %s ('%s'), - %2d - 0x%08x - %s - 0x%x\n", debugstr_guid(&raw const (*ddoi).guidType), _dump_dinput_GUID(&raw const (*ddoi).guidType), (*ddoi).dwOfs, (*ddoi).dwType, debugstr_w(&raw const (*ddoi).tszName as *const u16), (*ddoi).dwFlags);
}
#[no_mangle]
pub unsafe extern "C" fn _dump_dinput_GUID(mut guid: *const GUID) -> *const ::core::ffi::c_char {
    let mut i: ::core::ffi::c_uint = 0;
    static mut guids: [C2RustUnnamed_3; 26] = unsafe {
        [
            C2RustUnnamed_3 {
                guid: &raw const GUID_XAxis,
                name: b"GUID_XAxis\0" as *const u8 as *const ::core::ffi::c_char,
            },
            C2RustUnnamed_3 {
                guid: &raw const GUID_YAxis,
                name: b"GUID_YAxis\0" as *const u8 as *const ::core::ffi::c_char,
            },
            C2RustUnnamed_3 {
                guid: &raw const GUID_ZAxis,
                name: b"GUID_ZAxis\0" as *const u8 as *const ::core::ffi::c_char,
            },
            C2RustUnnamed_3 {
                guid: &raw const GUID_RxAxis,
                name: b"GUID_RxAxis\0" as *const u8 as *const ::core::ffi::c_char,
            },
            C2RustUnnamed_3 {
                guid: &raw const GUID_RyAxis,
                name: b"GUID_RyAxis\0" as *const u8 as *const ::core::ffi::c_char,
            },
            C2RustUnnamed_3 {
                guid: &raw const GUID_RzAxis,
                name: b"GUID_RzAxis\0" as *const u8 as *const ::core::ffi::c_char,
            },
            C2RustUnnamed_3 {
                guid: &raw const GUID_Slider,
                name: b"GUID_Slider\0" as *const u8 as *const ::core::ffi::c_char,
            },
            C2RustUnnamed_3 {
                guid: &raw const GUID_Button,
                name: b"GUID_Button\0" as *const u8 as *const ::core::ffi::c_char,
            },
            C2RustUnnamed_3 {
                guid: &raw const GUID_Key,
                name: b"GUID_Key\0" as *const u8 as *const ::core::ffi::c_char,
            },
            C2RustUnnamed_3 {
                guid: &raw const GUID_POV,
                name: b"GUID_POV\0" as *const u8 as *const ::core::ffi::c_char,
            },
            C2RustUnnamed_3 {
                guid: &raw const GUID_Unknown,
                name: b"GUID_Unknown\0" as *const u8 as *const ::core::ffi::c_char,
            },
            C2RustUnnamed_3 {
                guid: &raw const GUID_SysMouse,
                name: b"GUID_SysMouse\0" as *const u8 as *const ::core::ffi::c_char,
            },
            C2RustUnnamed_3 {
                guid: &raw const GUID_SysKeyboard,
                name: b"GUID_SysKeyboard\0" as *const u8 as *const ::core::ffi::c_char,
            },
            C2RustUnnamed_3 {
                guid: &raw const GUID_Joystick,
                name: b"GUID_Joystick\0" as *const u8 as *const ::core::ffi::c_char,
            },
            C2RustUnnamed_3 {
                guid: &raw const GUID_ConstantForce,
                name: b"GUID_ConstantForce\0" as *const u8 as *const ::core::ffi::c_char,
            },
            C2RustUnnamed_3 {
                guid: &raw const GUID_RampForce,
                name: b"GUID_RampForce\0" as *const u8 as *const ::core::ffi::c_char,
            },
            C2RustUnnamed_3 {
                guid: &raw const GUID_Square,
                name: b"GUID_Square\0" as *const u8 as *const ::core::ffi::c_char,
            },
            C2RustUnnamed_3 {
                guid: &raw const GUID_Sine,
                name: b"GUID_Sine\0" as *const u8 as *const ::core::ffi::c_char,
            },
            C2RustUnnamed_3 {
                guid: &raw const GUID_Triangle,
                name: b"GUID_Triangle\0" as *const u8 as *const ::core::ffi::c_char,
            },
            C2RustUnnamed_3 {
                guid: &raw const GUID_SawtoothUp,
                name: b"GUID_SawtoothUp\0" as *const u8 as *const ::core::ffi::c_char,
            },
            C2RustUnnamed_3 {
                guid: &raw const GUID_SawtoothDown,
                name: b"GUID_SawtoothDown\0" as *const u8 as *const ::core::ffi::c_char,
            },
            C2RustUnnamed_3 {
                guid: &raw const GUID_Spring,
                name: b"GUID_Spring\0" as *const u8 as *const ::core::ffi::c_char,
            },
            C2RustUnnamed_3 {
                guid: &raw const GUID_Damper,
                name: b"GUID_Damper\0" as *const u8 as *const ::core::ffi::c_char,
            },
            C2RustUnnamed_3 {
                guid: &raw const GUID_Inertia,
                name: b"GUID_Inertia\0" as *const u8 as *const ::core::ffi::c_char,
            },
            C2RustUnnamed_3 {
                guid: &raw const GUID_Friction,
                name: b"GUID_Friction\0" as *const u8 as *const ::core::ffi::c_char,
            },
            C2RustUnnamed_3 {
                guid: &raw const GUID_CustomForce,
                name: b"GUID_CustomForce\0" as *const u8 as *const ::core::ffi::c_char,
            },
        ]
    };
    if guid.is_null() {
        return b"null GUID\0" as *const u8 as *const ::core::ffi::c_char;
    }
    i = 0 as ::core::ffi::c_uint;
    while (i as usize)
        < (::core::mem::size_of::<[C2RustUnnamed_3; 26]>() as usize)
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed_3>() as usize)
    {
        if memcmp(
            guids[i as usize].guid as *const ::core::ffi::c_void,
            guid as *const ::core::ffi::c_void,
            ::core::mem::size_of::<GUID>() as usize,
        ) == 0
        {
            return guids[i as usize].name;
        }
        i = i.wrapping_add(1);
    }
    return debugstr_guid(guid);
}
#[no_mangle]
pub unsafe extern "C" fn _dump_DIDATAFORMAT(df: *const DIDATAFORMAT) {
    let mut i: u32;

    TRACE!("Dumping DIDATAFORMAT structure:\n");
    TRACE!("  - dwSize: %d\n", (*df).dwSize);
    if (*df).dwSize != core::mem::size_of::<DIDATAFORMAT>() as DWORD {
        WARN!("Non-standard DIDATAFORMAT structure size %d\n", (*df).dwSize);
    }
    TRACE!("  - dwObjsize: %d\n", (*df).dwObjSize);
    if (*df).dwObjSize != core::mem::size_of::<DIOBJECTDATAFORMAT>() as DWORD {
        WARN!("Non-standard DIOBJECTDATAFORMAT structure size %d\n", (*df).dwObjSize);
    }
    TRACE!("  - dwFlags: 0x%08x (", (*df).dwFlags);
    match (*df).dwFlags {
        DIDF_ABSAXIS => {
            TRACE!("DIDF_ABSAXIS");
        }
        DIDF_RELAXIS => {
            TRACE!("DIDF_RELAXIS");
        }
        _ => {
            TRACE!("unknown");
        }
    }
    TRACE!(")\n");
    TRACE!("  - dwDataSize: %d\n", (*df).dwDataSize);
    TRACE!("  - dwNumObjs: %d\n", (*df).dwNumObjs);

    i = 0;
    while i < (*df).dwNumObjs {
        TRACE!("  - Object %d:\n", i);
        TRACE!(
            "      * GUID: %s ('%s')\n",
            debugstr_guid((*(*df).rgodf.offset(i as isize)).pguid as *const _GUID),
            _dump_dinput_GUID((*(*df).rgodf.offset(i as isize)).pguid)
        );
        TRACE!("      * dwOfs: %d\n", (*(*df).rgodf.offset(i as isize)).dwOfs);
        TRACE!("      * dwType: 0x%08x\n", (*(*df).rgodf.offset(i as isize)).dwType);
        TRACE!("        ");
        _dump_EnumObjects_flags((*(*df).rgodf.offset(i as isize)).dwType);
        TRACE!("\n");
        TRACE!("      * dwFlags: 0x%08x\n", (*(*df).rgodf.offset(i as isize)).dwFlags);
        TRACE!("        ");
        _dump_ObjectDataFormat_flags((*(*df).rgodf.offset(i as isize)).dwFlags);
        TRACE!("\n");
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn get_app_key(defkey: *mut HKEY, appkey: *mut HKEY) -> BOOL {
    let mut buffer: [::core::ffi::c_char; (MAX_PATH + 16) as _] = [0; (MAX_PATH + 16) as _];
    let mut len: DWORD = 0;

    *appkey = core::ptr::null_mut();

    /* @@ Wine registry key: HKCU\Software\Wine\DirectInput */
    if RegOpenKeyA(HKEY_CURRENT_USER, b"Software\\Wine\\DirectInput\0".as_ptr() as LPCSTR, defkey as *mut HKEY) != 0 {
        *defkey = core::ptr::null_mut();
    }

    len = GetModuleFileNameA(core::ptr::null_mut(), &raw mut buffer as *mut ::core::ffi::c_char, MAX_PATH as DWORD);
    if len != 0 && len < MAX_PATH as DWORD {
        let mut tmpkey: HKEY = core::ptr::null_mut();

        /* @@ Wine registry key: HKCU\Software\Wine\AppDefaults\app.exe\DirectInput */
        if RegOpenKeyA(HKEY_CURRENT_USER, b"Software\\Wine\\AppDefaults\0".as_ptr() as LPCSTR, &raw mut tmpkey) == 0 {
            let mut p: *mut ::core::ffi::c_char = core::ptr::null_mut();
            let mut appname: *mut ::core::ffi::c_char = &raw mut buffer as *mut ::core::ffi::c_char;
            p = strrchr(appname, '/' as i32);
            if !p.is_null() {
                appname = p.offset(1 as isize);
            }
            p = strrchr(appname, '\\' as i32);
            if !p.is_null() {
                appname = p.offset(1 as isize);
            }
            strcat(appname, b"\\DirectInput\0".as_ptr() as *const ::core::ffi::c_char);

            if RegOpenKeyA(tmpkey, appname as LPCSTR, appkey as *mut HKEY) != 0 {
                *appkey = core::ptr::null_mut();
            }
            RegCloseKey(tmpkey);
        }
    }

    return (!(*defkey).is_null() || !(*appkey).is_null()) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn get_config_key(
    mut defkey: HKEY,
    mut appkey: HKEY,
    mut name: *const ::core::ffi::c_char,
    mut buffer: *mut ::core::ffi::c_char,
    mut size: DWORD,
) -> DWORD {
    if !appkey.is_null()
        && RegQueryValueExA(
            appkey,
            name as LPCSTR,
            0 as LPDWORD,
            ::core::ptr::null_mut::<DWORD>(),
            buffer as LPBYTE,
            &raw mut size,
        ) == 0
    {
        return 0 as DWORD;
    }

    if !defkey.is_null()
        && RegQueryValueExA(
            defkey,
            name as LPCSTR,
            0 as LPDWORD,
            ::core::ptr::null_mut::<DWORD>(),
            buffer as LPBYTE,
            &raw mut size,
        ) == 0
    {
        return 0 as DWORD;
    }

    return ERROR_FILE_NOT_FOUND as DWORD;
}
#[no_mangle]
pub unsafe extern "C" fn fill_DataFormat(
    out: *mut ::core::ffi::c_void,
    size: DWORD,
    in_: *const ::core::ffi::c_void,
    df: *const DataFormat,
) {
    let mut i: ::core::ffi::c_int = 0;
    let in_c: *const ::core::ffi::c_char = in_ as *const ::core::ffi::c_char;
    let out_c: *mut ::core::ffi::c_char = out as *mut ::core::ffi::c_char;

    memset(out, 0, size as usize);
    if (*df).dt.is_null() {
        /* This means that the app uses Wine's internal data format */
        memcpy(
            out,
            in_,
            min!(size, (*df).internal_format_size as DWORD) as usize,
        );
    } else {
        while i < (*df).size {
            let ent = (*df).dt.offset(i as isize);
            if (*ent).offset_in >= 0 {
                if (*ent).size == 1 {
                    TRACE!(
                        "Copying (c) to %d from %d (value %d)\n",
                        (*ent).offset_out,
                        (*ent).offset_in,
                        *in_c.offset((*ent).offset_in as isize) as ::core::ffi::c_int
                    );
                    *out_c.offset((*ent).offset_out as isize) =
                        *in_c.offset((*ent).offset_in as isize);
                } else if (*ent).size == 2 {
                    TRACE!(
                        "Copying (s) to %d from %d (value %d)\n",
                        (*ent).offset_out,
                        (*ent).offset_in,
                        *(in_c.offset((*ent).offset_in as isize) as *const ::core::ffi::c_short)
                            as ::core::ffi::c_int
                    );
                    *(out_c.offset((*ent).offset_out as isize) as *mut ::core::ffi::c_short) =
                        *(in_c.offset((*ent).offset_in as isize) as *const ::core::ffi::c_short);
                } else if (*ent).size == 4 {
                    TRACE!(
                        "Copying (i) to %d from %d (value %d)\n",
                        (*ent).offset_out,
                        (*ent).offset_in,
                        *(in_c.offset((*ent).offset_in as isize) as *const ::core::ffi::c_int)
                    );
                    *(out_c.offset((*ent).offset_out as isize) as *mut ::core::ffi::c_int) =
                        *(in_c.offset((*ent).offset_in as isize) as *const ::core::ffi::c_int);
                } else {
                    memcpy(
                        out_c.offset((*ent).offset_out as isize) as *mut ::core::ffi::c_void,
                        in_c.offset((*ent).offset_in as isize) as *const ::core::ffi::c_void,
                        (*ent).size as usize,
                    );
                }
            } else {
                if (*ent).size == 1 {
                    TRACE!(
                        "Copying (c) to %d default value %d\n",
                        (*ent).offset_out,
                        (*ent).value
                    );
                    *out_c.offset((*ent).offset_out as isize) =
                        (*ent).value as ::core::ffi::c_char;
                } else if (*ent).size == 2 {
                    TRACE!(
                        "Copying (s) to %d default value %d\n",
                        (*ent).offset_out,
                        (*ent).value
                    );
                    *(out_c.offset((*ent).offset_out as isize) as *mut ::core::ffi::c_short) =
                        (*ent).value as ::core::ffi::c_short;
                } else if (*ent).size == 4 {
                    TRACE!(
                        "Copying (i) to %d default value %d\n",
                        (*ent).offset_out,
                        (*ent).value
                    );
                    *(out_c.offset((*ent).offset_out as isize) as *mut ::core::ffi::c_int) =
                        (*ent).value as ::core::ffi::c_int;
                } else {
                    memset(
                        out_c.offset((*ent).offset_out as isize) as *mut ::core::ffi::c_void,
                        0,
                        (*ent).size as usize,
                    );
                }
            }
            i += 1;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn release_DataFormat(format: *mut DataFormat) {
    HeapFree(GetProcessHeap(), 0, (*format).dt as PVOID);
    (*format).dt = core::ptr::null_mut();
    HeapFree(GetProcessHeap(), 0, (*format).offsets as PVOID);
    (*format).offsets = core::ptr::null_mut();
    HeapFree(GetProcessHeap(), 0, (*format).user_df as PVOID);
    (*format).user_df = core::ptr::null_mut();
}

unsafe extern "C" fn dataformat_to_odf(df: LPCDIDATAFORMAT, idx: core::ffi::c_int) -> LPDIOBJECTDATAFORMAT {
    if idx < 0 || idx as DWORD >= (*df).dwNumObjs {
        return core::ptr::null_mut();
    }
    return ((*df).rgodf as LPBYTE).offset((idx as DWORD).wrapping_mul((*df).dwObjSize) as isize) as LPDIOBJECTDATAFORMAT;
}
#[no_mangle]
pub unsafe extern "C" fn dataformat_to_odf_by_type(df: LPCDIDATAFORMAT, n: ::core::ffi::c_int, type_: DWORD) -> LPDIOBJECTDATAFORMAT {
    let mut i: ::core::ffi::c_int = 0;
    let mut nfound: ::core::ffi::c_int = 0;
    i = 0;
    while (i as DWORD) < (*df).dwNumObjs {
        let mut odf: LPDIOBJECTDATAFORMAT = dataformat_to_odf(df, i);
        if ((*odf).dwType & type_) != 0 {
            if n == nfound {
                return odf;
            }
            nfound += 1;
        }
        i += 1;
    }
    return ::core::ptr::null_mut::<_DIOBJECTDATAFORMAT>();
}

unsafe extern "C" fn create_DataFormat(asked_format: LPCDIDATAFORMAT, format: *mut DataFormat) -> HRESULT {
    let mut dt: *mut DataTransform = core::ptr::null_mut();
    let mut i: u32 = 0;
    let mut j: u32 = 0;
    let mut same: i32 = 1;
    let mut done: *mut i32 = core::ptr::null_mut();
    let mut index: i32 = 0;
    let mut next: DWORD = 0;

    if (*format).wine_df.is_null() {
        return DIERR_INVALIDPARAM;
    }

    'failed: {
        done = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            ((*asked_format).dwNumObjs as usize * core::mem::size_of::<i32>()) as SIZE_T,
        ) as *mut i32;
        dt = HeapAlloc(
            GetProcessHeap(),
            0,
            ((*asked_format).dwNumObjs as usize * core::mem::size_of::<DataTransform>()) as SIZE_T,
        ) as *mut DataTransform;
        if dt.is_null() || done.is_null() {
            break 'failed;
        }

        (*format).offsets = HeapAlloc(
            GetProcessHeap(),
            0,
            ((*(*format).wine_df).dwNumObjs as usize * core::mem::size_of::<i32>()) as SIZE_T,
        ) as *mut i32;
        if (*format).offsets.is_null() {
            break 'failed;
        }

        (*format).user_df = HeapAlloc(
            GetProcessHeap(),
            0,
            ((*asked_format).dwSize) as SIZE_T,
        ) as LPDIDATAFORMAT;
        if (*format).user_df.is_null() {
            break 'failed;
        }
        memcpy(
            (*format).user_df as *mut core::ffi::c_void,
            asked_format as *const core::ffi::c_void,
            (*asked_format).dwSize as usize,
        );

        i = 0;
        while i < (*(*format).wine_df).dwNumObjs {
            *(*format).offsets.offset(i as isize) = -(1);

            j = 0;
            while j < (*asked_format).dwNumObjs {
                if !(*done.offset(j as isize) == 1) {
                    if ((*(*asked_format).rgodf.offset(j as isize)).pguid.is_null()
                        || (*(*(*format).wine_df).rgodf.offset(i as isize)).pguid.is_null()
                        || IsEqualGUID!(
                            (*(*(*format).wine_df).rgodf.offset(i as isize)).pguid,
                            (*(*asked_format).rgodf.offset(j as isize)).pguid,
                        ) != false)
                        && (((*(*asked_format).rgodf.offset(j as isize)).dwType
                            & DIDFT_INSTANCEMASK)
                            == DIDFT_ANYINSTANCE
                            || (DIDFT_GETINSTANCE!((*(*asked_format).rgodf.offset(j as isize)).dwType) as DWORD)
                                == 0x00FF
                            || (DIDFT_GETINSTANCE!((*(*asked_format).rgodf.offset(j as isize)).dwType) as DWORD)
                                == (DIDFT_GETINSTANCE!(
                                    (*(*(*format).wine_df).rgodf.offset(i as isize)).dwType,
                                ) as DWORD))
                        && (((DIDFT_GETTYPE!((*(*asked_format).rgodf.offset(j as isize)).dwType) as DWORD)
                            & (*(*(*format).wine_df).rgodf.offset(i as isize)).dwType)
                            != 0)
                    {
                        *done.offset(j as isize) = 1;

                        _dump_EnumObjects_flags((*(*asked_format).rgodf.offset(j as isize)).dwType);
                        _dump_ObjectDataFormat_flags((*(*asked_format).rgodf.offset(j as isize)).dwFlags);

                        _dump_EnumObjects_flags((*(*(*format).wine_df).rgodf.offset(i as isize)).dwType);
                        _dump_ObjectDataFormat_flags((*(*(*format).wine_df).rgodf.offset(i as isize)).dwFlags);

                        if ((*(*(*format).wine_df).rgodf.offset(i as isize)).dwType
                            & DIDFT_BUTTON)
                            != 0
                        {
                            (*dt.offset(index as isize)).size = core::mem::size_of::<BYTE>() as i32;
                        } else {
                            (*dt.offset(index as isize)).size = core::mem::size_of::<DWORD>() as i32;
                        }
                        (*dt.offset(index as isize)).offset_in =
                            (*(*(*format).wine_df).rgodf.offset(i as isize)).dwOfs as i32;
                        (*dt.offset(index as isize)).offset_out =
                            (*(*asked_format).rgodf.offset(j as isize)).dwOfs as i32;
                        *(*format).offsets.offset(i as isize) =
                            (*(*asked_format).rgodf.offset(j as isize)).dwOfs as i32;
                        (*dt.offset(index as isize)).value = 0;
                        next = next.wrapping_add((*dt.offset(index as isize)).size as DWORD);

                        if (*(*(*format).wine_df).rgodf.offset(i as isize)).dwOfs
                            != (*dt.offset(index as isize)).offset_out as DWORD
                        {
                            same = 0;
                        }

                        index += 1;
                        break;
                    }
                }
                j = j.wrapping_add(1);
            }
            i = i.wrapping_add(1);
        }

        j = 0;
        while j < (*asked_format).dwNumObjs {
            if *done.offset(j as isize) == 0 {
                _dump_EnumObjects_flags((*(*asked_format).rgodf.offset(j as isize)).dwType);
                _dump_ObjectDataFormat_flags((*(*asked_format).rgodf.offset(j as isize)).dwFlags);

                if ((*(*asked_format).rgodf.offset(j as isize)).dwType & DIDFT_BUTTON)
                    != 0
                {
                    (*dt.offset(index as isize)).size = core::mem::size_of::<BYTE>() as i32;
                } else {
                    (*dt.offset(index as isize)).size = core::mem::size_of::<DWORD>() as i32;
                }
                (*dt.offset(index as isize)).offset_in = -(1);
                (*dt.offset(index as isize)).offset_out =
                    (*(*asked_format).rgodf.offset(j as isize)).dwOfs as i32;
                if ((*(*asked_format).rgodf.offset(j as isize)).dwType & DIDFT_POV)
                    != 0
                {
                    (*dt.offset(index as isize)).value = -(1);
                } else {
                    (*dt.offset(index as isize)).value = 0;
                }
                index += 1;

                same = 0;
            }
            j = j.wrapping_add(1);
        }

        (*format).internal_format_size = (*(*format).wine_df).dwDataSize as i32;
        (*format).size = index;
        if same != 0 {
            HeapFree(GetProcessHeap(), 0, dt as LPVOID);
            dt = core::ptr::null_mut();
        }
        (*format).dt = dt;

        HeapFree(GetProcessHeap(), 0, done as LPVOID);

        return DI_OK;
    }

    // failed:
    HeapFree(GetProcessHeap(), 0, done as LPVOID);
    HeapFree(GetProcessHeap(), 0, dt as LPVOID);
    (*format).dt = core::ptr::null_mut();
    HeapFree(GetProcessHeap(), 0, (*format).offsets as LPVOID);
    (*format).offsets = core::ptr::null_mut();
    HeapFree(GetProcessHeap(), 0, (*format).user_df as LPVOID);
    (*format).user_df = core::ptr::null_mut();

    return DIERR_OUTOFMEMORY;
}

unsafe extern "C" fn offset_to_object(df: *const DataFormat, offset: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int;

    if (*df).offsets.is_null() {
        return -1;
    }

    i = 0;
    while (i as DWORD) < (*(*df).wine_df).dwNumObjs {
        if *(*df).offsets.offset(i as isize) == offset {
            return i;
        }
        i += 1;
    }

    return -1;
}
#[no_mangle]
pub unsafe extern "C" fn id_to_object(df: LPCDIDATAFORMAT, id: i32) -> i32 {
    let mut i: i32;
    let mut id: i32 = id;
    id &= 0x00ffffff;
    i = 0;
    while (i as DWORD) < (*df).dwNumObjs {
        if ((*dataformat_to_odf(df, i)).dwType & 0x00ffffff) == id as DWORD {
            return i;
        }
        i += 1;
    }
    return -1;
}

unsafe extern "C" fn id_to_offset(df: *const DataFormat, id: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let obj: ::core::ffi::c_int = id_to_object((*df).wine_df as LPCDIDATAFORMAT, id);

    return if obj >= 0 && !(*df).offsets.is_null() {
        *(*df).offsets.offset(obj as isize)
    } else {
        -1
    };
}
#[no_mangle]
pub unsafe extern "C" fn find_property(
    mut df: *const DataFormat,
    mut ph: LPCDIPROPHEADER,
) -> ::core::ffi::c_int {
    match (*ph).dwHow {
        DIPH_BYID => {
            return id_to_object(
                (*df).wine_df as LPCDIDATAFORMAT,
                (*ph).dwObj as ::core::ffi::c_int,
            );
        }
        DIPH_BYOFFSET => {
            return offset_to_object(df, (*ph).dwObj as ::core::ffi::c_int);
        }
        _ => {}
    }
    WINE_FIXME!("Unhandled ph->dwHow=='%04X'\n", (*ph).dwHow as ::core::ffi::c_uint);
    return -1;
}

unsafe extern "C" fn semantic_to_obj_id(This: *mut IDirectInputDeviceImpl, dwSemantic: DWORD) -> DWORD {
    let mut type_: DWORD = (0x0000ff00 & dwSemantic) >> 8;
    let offset: DWORD = 0x000000ff & dwSemantic;
    let mut obj_instance: DWORD = 0;
    let mut found: BOOL = (FALSE as i32);
    let mut i: core::ffi::c_int;

    i = 0;
    while (i as DWORD) < (*(*This).data_format.wine_df).dwNumObjs {
        let odf: LPDIOBJECTDATAFORMAT =
            dataformat_to_odf((*This).data_format.wine_df as LPCDIDATAFORMAT, i);

        if (*odf).dwOfs == offset {
            obj_instance = (DIDFT_GETINSTANCE!((*odf).dwType) as DWORD);
            found = (TRUE as i32);
            break;
        }
        i += 1;
    }

    if found == 0 {
        return 0;
    }

    if type_ & DIDFT_AXIS != 0 {
        type_ = DIDFT_RELAXIS;
    }
    if type_ & DIDFT_BUTTON != 0 {
        type_ = DIDFT_PSHBUTTON;
    }

    return type_ | (0x0000ff00 & (obj_instance << 8));
}

unsafe extern "C" fn del_mapping_key(device: *const WCHAR, username: *const WCHAR, guid: *const WCHAR) {
    static subkey: [WCHAR; 44] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0065, 0x005C, 0x0044, 0x0069, 0x0072, 0x0065, 0x0063, 0x0074,
    0x0049, 0x006E, 0x0070, 0x0075, 0x0074, 0x005C, 0x004D, 0x0061, 0x0070, 0x0070,
    0x0069, 0x006E, 0x0067, 0x0073, 0x005C, 0x0025, 0x0073, 0x005C, 0x0025, 0x0073,
    0x005C, 0x0025, 0x0073, 0x0000,
];
    let mut keyname: *mut WCHAR;

    keyname = heap_alloc(
        ((core::mem::size_of::<WCHAR>() as SIZE_T).wrapping_mul(
            (lstrlenW(core::ptr::addr_of!(subkey) as LPCWSTR) as SIZE_T)
                .wrapping_add(strlenW(username) as SIZE_T)
                .wrapping_add(strlenW(device) as SIZE_T)
                .wrapping_add(strlenW(guid) as SIZE_T),
        )) as _,
    ) as *mut WCHAR;
    sprintfW(
        keyname,
        core::ptr::addr_of!(subkey) as *const WCHAR,
        username,
        device,
        guid,
    );

    /* Remove old key mappings so there will be no overlapping mappings */
    RegDeleteKeyW(HKEY_CURRENT_USER, keyname as LPCWSTR);

    heap_free(keyname as *mut core::ffi::c_void);
}

unsafe extern "C" fn get_mapping_key(device: *const WCHAR, username: *const WCHAR, guid: *const WCHAR, create: BOOL) -> HKEY {
    static subkey: [u16; 44] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0065, 0x005C, 0x0044, 0x0069, 0x0072, 0x0065, 0x0063, 0x0074,
    0x0049, 0x006E, 0x0070, 0x0075, 0x0074, 0x005C, 0x004D, 0x0061, 0x0070, 0x0070,
    0x0069, 0x006E, 0x0067, 0x0073, 0x005C, 0x0025, 0x0073, 0x005C, 0x0025, 0x0073,
    0x005C, 0x0025, 0x0073, 0x0000,
];
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut keyname: *mut WCHAR = core::ptr::null_mut();

    keyname = HeapAlloc(
        GetProcessHeap(),
        0,
        (core::mem::size_of::<WCHAR>() as SIZE_T).wrapping_mul(
            (lstrlenW(core::ptr::addr_of!(subkey) as LPCWSTR) as SIZE_T)
                .wrapping_add(strlenW(username) as SIZE_T)
                .wrapping_add(strlenW(device) as SIZE_T)
                .wrapping_add(strlenW(guid) as SIZE_T),
        ),
    ) as *mut WCHAR;
    sprintfW(
        keyname,
        core::ptr::addr_of!(subkey) as *const WCHAR,
        username,
        device,
        guid,
    );

    /* The key used is HKCU\Software\Wine\DirectInput\Mappings\[username]\[device]\[mapping_guid] */
    if create != 0 {
        if RegCreateKeyW(HKEY_CURRENT_USER, keyname as LPCWSTR, core::ptr::addr_of_mut!(hkey)) != 0 {
            hkey = core::ptr::null_mut();
        }
    } else if RegOpenKeyW(HKEY_CURRENT_USER, keyname as LPCWSTR, core::ptr::addr_of_mut!(hkey)) != 0 {
        hkey = core::ptr::null_mut();
    }

    HeapFree(GetProcessHeap(), 0, keyname as PVOID);

    return hkey;
}
#[no_mangle]
pub unsafe extern "C" fn save_mapping_settings(iface: *mut IDirectInputDevice8W, lpdiaf: LPDIACTIONFORMATW, lpszUsername: LPCWSTR) -> HRESULT {
    let mut guid_str: *mut WCHAR = core::ptr::null_mut();
    let mut didev: DIDEVICEINSTANCEW = core::mem::zeroed();
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut i: i32 = 0;

    didev.dwSize = core::mem::size_of::<DIDEVICEINSTANCEW>() as DWORD;
    IDirectInputDevice8_GetDeviceInfo!(iface, &raw mut didev);

    if StringFromCLSID(&raw mut (*lpdiaf).guidActionMap, &raw mut guid_str) != S_OK {
        return DI_SETTINGSNOTSAVED;
    }

    del_mapping_key(&raw mut didev.tszInstanceName as *mut WCHAR, lpszUsername as *const WCHAR, guid_str);

    hkey = get_mapping_key(&raw mut didev.tszInstanceName as *mut WCHAR, lpszUsername as *const WCHAR, guid_str, (TRUE) as BOOL);

    if hkey.is_null() {
        CoTaskMemFree(guid_str as LPVOID);
        return DI_SETTINGSNOTSAVED;
    }

    /* Write each of the actions mapped for this device.
       Format is "dwSemantic"="dwObjID" and key is of type REG_DWORD
    */
    while (i as DWORD) < (*lpdiaf).dwNumActions {
        static mut format: [u16; 3] = [0x0025u16, 0x0078, 0x0000];
        let mut label: [WCHAR; 9] = [0; 9];

        if IsEqualGUID!(&raw mut didev.guidInstance, &raw mut (*(*lpdiaf).rgoAction.offset(i as isize)).guidInstance) != false
            && (*(*lpdiaf).rgoAction.offset(i as isize)).dwHow != DIAH_UNMAPPED
        {
            sprintfW(&raw mut label as *mut WCHAR, &raw const format as *const WCHAR, (*(*lpdiaf).rgoAction.offset(i as isize)).dwSemantic);
            RegSetValueExW(hkey, &raw mut label as *mut WCHAR as LPCWSTR, 0 as DWORD, REG_DWORD as DWORD, &raw mut (*(*lpdiaf).rgoAction.offset(i as isize)).dwObjID as *const BYTE, core::mem::size_of::<DWORD>() as DWORD);
        }
        i += 1;
    }

    RegCloseKey(hkey);
    CoTaskMemFree(guid_str as LPVOID);

    return DI_OK;
}
#[no_mangle]
pub unsafe extern "C" fn load_mapping_settings(
    mut This: *mut IDirectInputDeviceImpl,
    mut lpdiaf: LPDIACTIONFORMATW,
    mut username: *const WCHAR,
) -> BOOL {
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut guid_str: *mut WCHAR = core::ptr::null_mut();
    let mut didev: DIDEVICEINSTANCEW = core::mem::zeroed();
    let mut i: core::ffi::c_int;

    didev.dwSize = core::mem::size_of::<DIDEVICEINSTANCEW>() as DWORD;
    IDirectInputDevice8_GetDeviceInfo!(&raw mut (*This).IDirectInputDevice8W_iface, &raw mut didev);

    if StringFromCLSID(&raw mut (*lpdiaf).guidActionMap, &raw mut guid_str) != S_OK {
        return (FALSE) as BOOL;
    }

    hkey = get_mapping_key(
        &raw mut didev.tszInstanceName as *mut WCHAR,
        username,
        guid_str,
        (FALSE) as BOOL,
    );

    if hkey.is_null() {
        CoTaskMemFree(guid_str as LPVOID);
        return (FALSE) as BOOL;
    }

    /* Try to read each action in the DIACTIONFORMAT from registry */
    i = 0;
    while (i as DWORD) < (*lpdiaf).dwNumActions {
        static format: [WCHAR; 3] = [0x0025u16, 0x0078, 0x0000];
        let mut id: DWORD = 0;
        let mut size: DWORD = core::mem::size_of::<DWORD>() as DWORD;
        let mut label: [WCHAR; 9] = [0 as WCHAR; 9];

        sprintfW(
            &raw mut label as *mut WCHAR,
            &raw const format as *const WCHAR,
            (*(*lpdiaf).rgoAction.offset(i as isize)).dwSemantic,
        );

        if RegQueryValueExW(
            hkey,
            &raw mut label as *mut WCHAR as LPCWSTR,
            core::ptr::null_mut(),
            0 as *mut DWORD,
            &raw mut id as LPBYTE,
            &raw mut size,
        ) == 0
        {
            (*(*lpdiaf).rgoAction.offset(i as isize)).dwObjID = id;
            (*(*lpdiaf).rgoAction.offset(i as isize)).guidInstance = didev.guidInstance;
            (*(*lpdiaf).rgoAction.offset(i as isize)).dwHow = DIAH_USERCONFIG as DWORD;
        } else {
            memset(
                &raw mut (*(*lpdiaf).rgoAction.offset(i as isize)).guidInstance as *mut core::ffi::c_void,
                0,
                core::mem::size_of::<GUID>() as size_t,
            );
            (*(*lpdiaf).rgoAction.offset(i as isize)).dwHow = DIAH_UNMAPPED as DWORD;
        }

        i += 1;
    }

    RegCloseKey(hkey);
    CoTaskMemFree(guid_str as LPVOID);

    /* On Windows BuildActionMap can open empty mapping, so always return TRUE if get_mapping_key is success */
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn _build_action_map(
    mut iface: LPDIRECTINPUTDEVICE8W,
    mut lpdiaf: LPDIACTIONFORMATW,
    mut lpszUserName: LPCWSTR,
    mut dwFlags: DWORD,
    mut devMask: DWORD,
    mut df: LPCDIDATAFORMAT,
) -> HRESULT {
    let mut This: *mut IDirectInputDeviceImpl =
        impl_from_IDirectInputDevice8W(iface as *mut IDirectInputDevice8W);
    let mut username: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut username_size: DWORD = MAX_PATH as DWORD;
    let mut i: ::core::ffi::c_int = 0;
    let mut load_success: BOOL = FALSE as BOOL;
    let mut has_actions: BOOL = FALSE as BOOL;
    if dwFlags & DIDBAM_HWDEFAULTS as DWORD == 0 {
        if lpszUserName.is_null() {
            GetUserNameW(
                core::ptr::addr_of_mut!(username) as LPWSTR,
                core::ptr::addr_of_mut!(username_size),
            );
        } else {
            lstrcpynW(
                core::ptr::addr_of_mut!(username) as LPWSTR,
                lpszUserName,
                MAX_PATH as ::core::ffi::c_int,
            );
        }
        load_success = load_mapping_settings(
            This,
            lpdiaf,
            core::ptr::addr_of_mut!(username) as *mut WCHAR,
        );
    }
    if load_success != 0 {
        i = 0 as ::core::ffi::c_int;
        while (i as DWORD) < (*lpdiaf).dwNumActions {
            (*lpdiaf).dwCRC ^= (*(*lpdiaf).rgoAction.offset(i as isize)).dwObjID
                << i * 2 as ::core::ffi::c_int
                | (*(*lpdiaf).rgoAction.offset(i as isize)).dwObjID
                    >> (::core::mem::size_of::<DWORD>() as usize)
                        .wrapping_mul(8 as usize)
                        .wrapping_sub(
                            (i * 2 as ::core::ffi::c_int) as usize,
                        );
            (*lpdiaf).dwCRC ^= (*(*lpdiaf).rgoAction.offset(i as isize)).dwSemantic
                << i * 2 as ::core::ffi::c_int
                    + 5 as ::core::ffi::c_int
                | (*(*lpdiaf).rgoAction.offset(i as isize)).dwSemantic
                    >> (::core::mem::size_of::<DWORD>() as usize)
                        .wrapping_mul(8 as usize)
                        .wrapping_sub(
                            (i * 2 as ::core::ffi::c_int
                                + 5 as ::core::ffi::c_int)
                                as usize,
                        );
            i += 1;
        }
        return DI_OK;
    }
    i = 0 as ::core::ffi::c_int;
    while (i as DWORD) < (*lpdiaf).dwNumActions {
        if (*(*lpdiaf).rgoAction.offset(i as isize)).dwSemantic & devMask == devMask {
            let mut obj_id: DWORD = semantic_to_obj_id(
                This,
                (*(*lpdiaf).rgoAction.offset(i as isize)).dwSemantic,
            );
            let mut type_0: DWORD = (DIDFT_GETTYPE!(obj_id) as DWORD);
            let mut inst: DWORD = (DIDFT_GETINSTANCE!(obj_id) as DWORD);
            let mut odf: LPDIOBJECTDATAFORMAT = ::core::ptr::null_mut::<_DIOBJECTDATAFORMAT>();
            if type_0 == DIDFT_PSHBUTTON as DWORD {
                type_0 = DIDFT_BUTTON as DWORD;
            }
            if type_0 == DIDFT_RELAXIS as DWORD {
                type_0 = DIDFT_AXIS as DWORD;
            }
            odf = dataformat_to_odf_by_type(df, inst as ::core::ffi::c_int, type_0);
            if !odf.is_null() {
                (*(*lpdiaf).rgoAction.offset(i as isize)).dwObjID = obj_id;
                (*(*lpdiaf).rgoAction.offset(i as isize)).guidInstance = (*This).guid;
                (*(*lpdiaf).rgoAction.offset(i as isize)).dwHow = DIAH_DEFAULT as DWORD;
                has_actions = TRUE as BOOL;
            }
        } else if dwFlags & DIDBAM_PRESERVE as DWORD == 0 {
            memset(
                core::ptr::addr_of_mut!((*(*lpdiaf).rgoAction.offset(i as isize)).guidInstance)
                    as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<GUID>() as size_t,
            );
            (*(*lpdiaf).rgoAction.offset(i as isize)).dwHow = DIAH_UNMAPPED as DWORD;
        }
        i += 1;
    }
    (*lpdiaf).dwCRC = 0 as DWORD;
    i = 0 as ::core::ffi::c_int;
    while (i as DWORD) < (*lpdiaf).dwNumActions {
        (*lpdiaf).dwCRC ^= (*(*lpdiaf).rgoAction.offset(i as isize)).dwObjID
            << i * 2 as ::core::ffi::c_int
            | (*(*lpdiaf).rgoAction.offset(i as isize)).dwObjID
                >> (::core::mem::size_of::<DWORD>() as usize)
                    .wrapping_mul(8 as usize)
                    .wrapping_sub((i * 2 as ::core::ffi::c_int) as usize);
        (*lpdiaf).dwCRC ^= (*(*lpdiaf).rgoAction.offset(i as isize)).dwSemantic
            << i * 2 as ::core::ffi::c_int
                + 5 as ::core::ffi::c_int
            | (*(*lpdiaf).rgoAction.offset(i as isize)).dwSemantic
                >> (::core::mem::size_of::<DWORD>() as usize)
                    .wrapping_mul(8 as usize)
                    .wrapping_sub(
                        (i * 2 as ::core::ffi::c_int
                            + 5 as ::core::ffi::c_int)
                            as usize,
                    );
        i += 1;
    }
    if has_actions == 0 {
        return DI_NOEFFECT;
    }
    return IDirectInputDevice8WImpl_BuildActionMap(iface, lpdiaf, lpszUserName, dwFlags);
}
#[no_mangle]
pub unsafe extern "C" fn _set_action_map(
    mut iface: LPDIRECTINPUTDEVICE8W,
    mut lpdiaf: LPDIACTIONFORMATW,
    mut lpszUserName: LPCWSTR,
    mut dwFlags: DWORD,
    mut df: LPCDIDATAFORMAT,
) -> HRESULT {
    let mut This: *mut IDirectInputDeviceImpl =
        impl_from_IDirectInputDevice8W(iface as *mut IDirectInputDevice8W);
    let mut data_format: DIDATAFORMAT = core::mem::zeroed();
    let mut obj_df: *mut DIOBJECTDATAFORMAT = core::ptr::null_mut();
    let mut dp: DIPROPDWORD = core::mem::zeroed();
    let mut dpr: DIPROPRANGE = core::mem::zeroed();
    let mut dps: DIPROPSTRING = core::mem::zeroed();
    let mut username: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut username_size: DWORD = MAX_PATH as DWORD;
    let mut new_crc: DWORD = 0 as DWORD;
    let mut i: ::core::ffi::c_int = 0;
    let mut action: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut num_actions: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut offset: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;

    if (*This).acquired != 0 {
        return DIERR_ACQUIRED;
    }

    data_format.dwSize = ::core::mem::size_of::<DIDATAFORMAT>() as DWORD;
    data_format.dwObjSize = ::core::mem::size_of::<DIOBJECTDATAFORMAT>() as DWORD;
    data_format.dwFlags = DIDF_RELAXIS as DWORD;
    data_format.dwDataSize = (*lpdiaf).dwDataSize;

    /* Calculate checksum for actionformat */
    i = 0 as ::core::ffi::c_int;
    while (i as DWORD) < (*lpdiaf).dwNumActions {
        new_crc ^= (*(*lpdiaf).rgoAction.offset(i as isize))
            .dwObjID
            .wrapping_shl((i * (2 as ::core::ffi::c_int)) as u32)
            | (*(*lpdiaf).rgoAction.offset(i as isize)).dwObjID.wrapping_shr(
                ((::core::mem::size_of::<DWORD>() as ::core::ffi::c_int
                    * (8 as ::core::ffi::c_int))
                    - (i * (2 as ::core::ffi::c_int))) as u32,
            );
        new_crc ^= (*(*lpdiaf).rgoAction.offset(i as isize))
            .dwSemantic
            .wrapping_shl(
                ((i * (2 as ::core::ffi::c_int))
                    + (5 as ::core::ffi::c_int)) as u32,
            )
            | (*(*lpdiaf).rgoAction.offset(i as isize)).dwSemantic.wrapping_shr(
                ((::core::mem::size_of::<DWORD>() as ::core::ffi::c_int
                    * (8 as ::core::ffi::c_int))
                    - ((i * (2 as ::core::ffi::c_int))
                        + (5 as ::core::ffi::c_int))) as u32,
            );
        i += 1;
    }

    /* Count the actions */
    i = 0 as ::core::ffi::c_int;
    while (i as DWORD) < (*lpdiaf).dwNumActions {
        if IsEqualGUID!(
            &raw mut (*This).guid,
            &raw mut (*(*lpdiaf).rgoAction.offset(i as isize)).guidInstance,
        ) != false
            || (IsEqualGUID!(
                &raw const GUID_NULL,
                &raw mut (*(*lpdiaf).rgoAction.offset(i as isize)).guidInstance,
            ) != false
                && (((*(*lpdiaf).rgoAction.offset(i as isize)).dwSemantic & (*lpdiaf).dwGenre)
                    == (*lpdiaf).dwGenre
                    || ((*(*lpdiaf).rgoAction.offset(i as isize)).dwSemantic
                        & 0xff000000 as DWORD)
                        == 0xff000000 as DWORD))
        {
            num_actions += 1;
        }
        i += 1;
    }

    /* Should return DI_NOEFFECT if we dont have any actions and actionformat has not changed */
    if num_actions == 0 as ::core::ffi::c_int
        && (*lpdiaf).dwCRC == new_crc
        && (dwFlags & DIDSAM_FORCESAVE as DWORD) == 0
    {
        return DI_NOEFFECT;
    }

    /* update dwCRC to track if action format has changed */
    (*lpdiaf).dwCRC = new_crc;

    (*This).num_actions = num_actions;

    /* Construct the dataformat and actionmap */
    obj_df = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        ((::core::mem::size_of::<DIOBJECTDATAFORMAT>()).wrapping_mul(num_actions as usize)) as SIZE_T,
    ) as *mut DIOBJECTDATAFORMAT;
    data_format.rgodf = obj_df as LPDIOBJECTDATAFORMAT;
    data_format.dwNumObjs = num_actions as DWORD;

    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        (*This).action_map as LPVOID,
    );
    (*This).action_map = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        ((::core::mem::size_of::<ActionMap>()).wrapping_mul(num_actions as usize)) as SIZE_T,
    ) as *mut ActionMap;

    i = 0 as ::core::ffi::c_int;
    while (i as DWORD) < (*lpdiaf).dwNumActions {
        if IsEqualGUID!(
            &raw mut (*This).guid,
            &raw mut (*(*lpdiaf).rgoAction.offset(i as isize)).guidInstance,
        ) != false
        {
            let mut inst: DWORD =
                (DIDFT_GETINSTANCE!((*(*lpdiaf).rgoAction.offset(i as isize)).dwObjID) as DWORD);
            let mut type_0: DWORD =
                (DIDFT_GETTYPE!((*(*lpdiaf).rgoAction.offset(i as isize)).dwObjID) as DWORD);
            let mut obj: LPDIOBJECTDATAFORMAT = core::ptr::null_mut();

            if type_0 == DIDFT_PSHBUTTON as DWORD {
                type_0 = DIDFT_BUTTON as DWORD;
            }
            if type_0 == DIDFT_RELAXIS as DWORD {
                type_0 = DIDFT_AXIS as DWORD;
            }

            obj = dataformat_to_odf_by_type(df, inst as ::core::ffi::c_int, type_0);

            memcpy(
                obj_df.offset(action as isize) as *mut DIOBJECTDATAFORMAT
                    as *mut ::core::ffi::c_void,
                obj as *const ::core::ffi::c_void,
                (*df).dwObjSize as usize,
            );

            (*(*This).action_map.offset(action as isize)).uAppData =
                (*(*lpdiaf).rgoAction.offset(i as isize)).uAppData;
            (*(*This).action_map.offset(action as isize)).offset = offset;
            (*obj_df.offset(action as isize)).dwOfs = offset as DWORD;
            offset = offset.wrapping_add(
                (if type_0 & DIDFT_BUTTON as DWORD != 0 {
                    1
                } else {
                    4
                }) as ::core::ffi::c_uint,
            );

            action += 1;
        } else if ((*(*lpdiaf).rgoAction.offset(i as isize)).dwSemantic & (*lpdiaf).dwGenre)
            == (*lpdiaf).dwGenre
            || ((*(*lpdiaf).rgoAction.offset(i as isize)).dwSemantic
                & 0xff000000 as DWORD)
                == 0xff000000 as DWORD
        {
            let mut obj_id: DWORD = semantic_to_obj_id(
                This,
                (*(*lpdiaf).rgoAction.offset(i as isize)).dwSemantic,
            );
            let mut type_1: DWORD = (DIDFT_GETTYPE!(obj_id) as DWORD);
            let mut inst_0: DWORD = (DIDFT_GETINSTANCE!(obj_id) as DWORD);
            let mut obj_0: LPDIOBJECTDATAFORMAT = core::ptr::null_mut();

            if type_1 == DIDFT_PSHBUTTON as DWORD {
                type_1 = DIDFT_BUTTON as DWORD;
            } else if type_1 == DIDFT_RELAXIS as DWORD {
                type_1 = DIDFT_AXIS as DWORD;
            }

            obj_0 = dataformat_to_odf_by_type(df, inst_0 as ::core::ffi::c_int, type_1);

            if !obj_0.is_null() {
                memcpy(
                    obj_df.offset(action as isize) as *mut DIOBJECTDATAFORMAT
                        as *mut ::core::ffi::c_void,
                    obj_0 as *const ::core::ffi::c_void,
                    (*df).dwObjSize as usize,
                );

                (*(*This).action_map.offset(action as isize)).uAppData =
                    (*(*lpdiaf).rgoAction.offset(i as isize)).uAppData;
                (*(*This).action_map.offset(action as isize)).offset = offset;
                (*obj_df.offset(action as isize)).dwOfs = offset as DWORD;
                offset = offset.wrapping_add(
                    (if type_1 & DIDFT_BUTTON as DWORD != 0 {
                        1
                    } else {
                        4
                    }) as ::core::ffi::c_uint,
                );

                action += 1;
            }
        }
        i += 1;
    }

    if action == 0 as ::core::ffi::c_int {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            obj_df as LPVOID,
        );
        return DI_NOEFFECT;
    }
    data_format.dwNumObjs = action as DWORD;

    IDirectInputDevice8_SetDataFormat!(iface, &raw mut data_format as LPCDIDATAFORMAT);

    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        obj_df as LPVOID,
    );

    /* Set the device properties according to the action format */
    dpr.diph.dwSize = ::core::mem::size_of::<DIPROPRANGE>() as DWORD;
    dpr.lMin = (*lpdiaf).lAxisMin;
    dpr.lMax = (*lpdiaf).lAxisMax;
    dpr.diph.dwHeaderSize = ::core::mem::size_of::<DIPROPHEADER>() as DWORD;
    dpr.diph.dwHow = DIPH_DEVICE as DWORD;
    IDirectInputDevice8_SetProperty!(
        iface,
        (DIPROP_RANGE as usize) as *const GUID,
        &raw mut dpr.diph as LPCDIPROPHEADER,
    );

    if (*lpdiaf).dwBufferSize > 0 as DWORD {
        dp.diph.dwSize = ::core::mem::size_of::<DIPROPDWORD>() as DWORD;
        dp.dwData = (*lpdiaf).dwBufferSize;
        dp.diph.dwHeaderSize = ::core::mem::size_of::<DIPROPHEADER>() as DWORD;
        dp.diph.dwHow = DIPH_DEVICE as DWORD;
        IDirectInputDevice8_SetProperty!(
            iface,
            (DIPROP_BUFFERSIZE as usize) as *const GUID,
            &raw mut dp.diph as LPCDIPROPHEADER,
        );
    }

    /* Retrieve logged user name if necessary */
    if lpszUserName.is_null() {
        GetUserNameW(&raw mut username as LPWSTR, &raw mut username_size);
    } else {
        lstrcpynW(
            &raw mut username as LPWSTR,
            lpszUserName,
            MAX_PATH as ::core::ffi::c_int,
        );
    }

    dps.diph.dwSize = ::core::mem::size_of::<DIPROPSTRING>() as DWORD;
    dps.diph.dwHeaderSize = ::core::mem::size_of::<DIPROPHEADER>() as DWORD;
    dps.diph.dwObj = 0 as DWORD;
    dps.diph.dwHow = DIPH_DEVICE as DWORD;
    if dwFlags & DIDSAM_NOUSER as DWORD != 0 {
        dps.wsz[0 as usize] = '\0' as i32 as WCHAR;
    } else {
        lstrcpynW(
            &raw mut dps.wsz as LPWSTR,
            &raw mut username as *mut WCHAR as LPCWSTR,
            ARRAY_SIZE!(dps.wsz) as ::core::ffi::c_int,
        );
    }
    IDirectInputDevice8_SetProperty!(
        iface,
        (DIPROP_USERNAME as usize) as *const GUID,
        &raw mut dps.diph as LPCDIPROPHEADER,
    );

    /* Save the settings to disk */
    save_mapping_settings(
        iface as *mut IDirectInputDevice8W,
        lpdiaf,
        &raw mut username as *mut WCHAR as LPCWSTR,
    );

    return DI_OK;
}
#[no_mangle]
pub unsafe extern "C" fn queue_event(
    iface: LPDIRECTINPUTDEVICE8A,
    inst_id: ::core::ffi::c_int,
    data: DWORD,
    time: DWORD,
    seq: DWORD,
) {
    let This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    let mut next_pos: ::core::ffi::c_int = 0;
    let mut ofs: ::core::ffi::c_int = id_to_offset(core::ptr::addr_of_mut!((*This).data_format), inst_id);

    /* Event is being set regardless of the queue state */
    if !(*This).hEvent.is_null() {
        SetEvent((*This).hEvent);
    }

    PostMessageW(
        GetDesktopWindow(),
        WM_WINE_NOTIFY_ACTIVITY as UINT,
        0 as WPARAM,
        0 as LPARAM,
    );

    if (*This).queue_len == 0 || (*This).overflow != 0 || ofs < 0 {
        return;
    }

    next_pos = ((*This).queue_head + 1) % (*This).queue_len;
    if next_pos == (*This).queue_tail {
        (*This).overflow = (TRUE as i32);
        return;
    }

    (*(*This).data_queue.offset((*This).queue_head as isize)).dwOfs = ofs as DWORD;
    (*(*This).data_queue.offset((*This).queue_head as isize)).dwData = data;
    (*(*This).data_queue.offset((*This).queue_head as isize)).dwTimeStamp = time;
    (*(*This).data_queue.offset((*This).queue_head as isize)).dwSequence = seq;

    /* Set uAppData by means of action mapping */
    if (*This).num_actions > 0 {
        let mut i: ::core::ffi::c_int = 0;
        i = 0;
        while i < (*This).num_actions {
            if (*(*This).action_map.offset(i as isize)).offset == ofs as ::core::ffi::c_uint {
                (*(*This).data_queue.offset((*This).queue_head as isize)).uAppData =
                    (*(*This).action_map.offset(i as isize)).uAppData;
                break;
            } else {
                i += 1;
            }
        }
    }

    (*This).queue_head = next_pos;
    /* Send event if asked */
}

#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2WImpl_Acquire(iface: LPDIRECTINPUTDEVICE8W) -> HRESULT {
    let This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8W(iface as *mut IDirectInputDevice8W);
    let mut res: HRESULT;

    if (*This).data_format.user_df.is_null() {
        return DIERR_INVALIDPARAM;
    }
    if ((*This).dwCoopLevel & DISCL_FOREGROUND) != 0 && (*This).win != GetForegroundWindow() {
        return DIERR_OTHERAPPHASPRIO;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*This).crit));
    res = if (*This).acquired != 0 {
        S_FALSE
    } else {
        DI_OK
    };
    (*This).acquired = 1;
    LeaveCriticalSection(core::ptr::addr_of_mut!((*This).crit));
    if res == DI_OK {
        check_dinput_hooks(iface, (TRUE) as BOOL);
    }

    return res;
}

#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2AImpl_Acquire(iface: LPDIRECTINPUTDEVICE8A) -> HRESULT {
    let mut This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    return IDirectInputDevice2WImpl_Acquire(IDirectInputDevice8W_from_impl(This) as LPDIRECTINPUTDEVICE8W);
}

#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2WImpl_Unacquire(mut iface: LPDIRECTINPUTDEVICE8W) -> HRESULT {
    let mut This: *mut IDirectInputDeviceImpl =
        impl_from_IDirectInputDevice8W(iface as *mut IDirectInputDevice8W);
    let mut res: HRESULT;
    WINE_TRACE!("(%p)\n", This);
    EnterCriticalSection(&raw mut (*This).crit);
    res = if (*This).acquired == 0 {
        DI_NOEFFECT
    } else {
        DI_OK
    };
    (*This).acquired = 0;
    LeaveCriticalSection(&raw mut (*This).crit);
    if res == DI_OK {
        check_dinput_hooks(iface, (FALSE) as BOOL);
    }

    return res;
}

#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2AImpl_Unacquire(iface: LPDIRECTINPUTDEVICE8A) -> HRESULT {
    let mut This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    return IDirectInputDevice2WImpl_Unacquire(IDirectInputDevice8W_from_impl(This) as LPDIRECTINPUTDEVICE8W);
}
#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2WImpl_SetDataFormat(iface: LPDIRECTINPUTDEVICE8W, df: LPCDIDATAFORMAT) -> HRESULT {
    let This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8W(iface as *mut IDirectInputDevice8W);
    let mut res: HRESULT = DI_OK;

    if df.is_null() {
        return E_POINTER;
    }

    TRACE!("(%p) %p\n", This, df);

    _dump_DIDATAFORMAT(df as *const DIDATAFORMAT);

    if (*df).dwSize as usize != core::mem::size_of::<DIDATAFORMAT>() as usize {
        return DIERR_INVALIDPARAM;
    }
    if (*This).acquired != 0 {
        return DIERR_ACQUIRED;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*This).crit));

    release_DataFormat(core::ptr::addr_of_mut!((*This).data_format));
    res = create_DataFormat(df, core::ptr::addr_of_mut!((*This).data_format));

    LeaveCriticalSection(core::ptr::addr_of_mut!((*This).crit));
    return res;
}

#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2AImpl_SetDataFormat(iface: LPDIRECTINPUTDEVICE8A, df: LPCDIDATAFORMAT) -> HRESULT {
    let This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    return IDirectInputDevice2WImpl_SetDataFormat(IDirectInputDevice8W_from_impl(This) as LPDIRECTINPUTDEVICE8W, df);
}
#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2WImpl_SetCooperativeLevel(iface: LPDIRECTINPUTDEVICE8W, mut hwnd: HWND, dwflags: DWORD) -> HRESULT {
    let mut This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8W(iface);
    _dump_cooperativelevel_DI(dwflags);
    if (dwflags & (DISCL_EXCLUSIVE as DWORD | DISCL_NONEXCLUSIVE as DWORD)) == 0 as DWORD
        || (dwflags & (DISCL_EXCLUSIVE as DWORD | DISCL_NONEXCLUSIVE as DWORD)) == (DISCL_EXCLUSIVE as DWORD | DISCL_NONEXCLUSIVE as DWORD)
        || (dwflags & (DISCL_FOREGROUND as DWORD | DISCL_BACKGROUND as DWORD)) == 0 as DWORD
        || (dwflags & (DISCL_FOREGROUND as DWORD | DISCL_BACKGROUND as DWORD)) == (DISCL_FOREGROUND as DWORD | DISCL_BACKGROUND as DWORD)
    {
        return DIERR_INVALIDPARAM;
    }
    if !hwnd.is_null() && (GetWindowLongW(hwnd, GWL_STYLE) & (WS_CHILD as LONG)) != 0 {
        return E_HANDLE;
    }
    if hwnd.is_null() && dwflags == (DISCL_NONEXCLUSIVE as DWORD | DISCL_BACKGROUND as DWORD) {
        hwnd = GetDesktopWindow();
    }
    if IsWindow(hwnd) == 0 {
        return E_HANDLE;
    }
    /* For security reasons native does not allow exclusive background level
       for mouse and keyboard only */
    if (dwflags & DISCL_EXCLUSIVE as DWORD) != 0
        && (dwflags & DISCL_BACKGROUND as DWORD) != 0
        && (IsEqualGUID!(core::ptr::addr_of_mut!((*This).guid), core::ptr::addr_of!(GUID_SysMouse)) != false
            || IsEqualGUID!(core::ptr::addr_of_mut!((*This).guid), core::ptr::addr_of!(GUID_SysKeyboard)) != false)
    {
        return DIERR_UNSUPPORTED;
    }
    /* Store the window which asks for the mouse */
    EnterCriticalSection(core::ptr::addr_of_mut!((*This).crit));
    (*This).win = hwnd;
    (*This).dwCoopLevel = dwflags;
    LeaveCriticalSection(core::ptr::addr_of_mut!((*This).crit));
    return DI_OK;
}

#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2AImpl_SetCooperativeLevel(iface: LPDIRECTINPUTDEVICE8A, hwnd: HWND, dwflags: DWORD) -> HRESULT {
    let This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    return IDirectInputDevice2WImpl_SetCooperativeLevel(IDirectInputDevice8W_from_impl(This) as LPDIRECTINPUTDEVICE8W, hwnd, dwflags);
}

#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2WImpl_SetEventNotification(iface: LPDIRECTINPUTDEVICE8W, event: HANDLE) -> HRESULT {
    let mut This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8W(iface as *mut IDirectInputDevice8W);

    EnterCriticalSection(core::ptr::addr_of_mut!((*This).crit));
    (*This).hEvent = event;
    LeaveCriticalSection(core::ptr::addr_of_mut!((*This).crit));
    return DI_OK;
}

#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2AImpl_SetEventNotification(iface: LPDIRECTINPUTDEVICE8A, event: HANDLE) -> HRESULT {
    let mut This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    return IDirectInputDevice2WImpl_SetEventNotification(IDirectInputDevice8W_from_impl(This) as LPDIRECTINPUTDEVICE8W, event);
}

#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2WImpl_Release(iface: LPDIRECTINPUTDEVICE8W) -> ULONG {
    let mut This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8W(iface);
    let mut ref_0: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_) as *mut LONG) as ULONG;

    TRACE!("(%p) ref %d\n", This, ref_0);

    if ref_0 != 0 {
        return ref_0;
    }

    IDirectInputDevice_Unacquire!(iface);
    /* Reset the FF state, free all effects, etc */
    IDirectInputDevice8_SendForceFeedbackCommand!(iface, DISFFC_RESET as DWORD);

    HeapFree(GetProcessHeap(), 0 as DWORD, (*This).data_queue as LPVOID);

    /* Free data format */
    HeapFree(GetProcessHeap(), 0 as DWORD, (*(*This).data_format.wine_df).rgodf as LPVOID);
    HeapFree(GetProcessHeap(), 0 as DWORD, (*This).data_format.wine_df as LPVOID);
    release_DataFormat(core::ptr::addr_of_mut!((*This).data_format));

    /* Free action mapping */
    HeapFree(GetProcessHeap(), 0 as DWORD, (*This).action_map as LPVOID);

    EnterCriticalSection(core::ptr::addr_of_mut!((*(*This).dinput).crit));
    list_remove(core::ptr::addr_of_mut!((*This).entry));
    LeaveCriticalSection(core::ptr::addr_of_mut!((*(*This).dinput).crit));

    IDirectInput_Release!(core::ptr::addr_of_mut!((*(*This).dinput).IDirectInput7A_iface));
    (*(*This).crit.DebugInfo).__bindgen_anon_1.Spare[0 as usize] = 0 as DWORD_PTR;
    DeleteCriticalSection(core::ptr::addr_of_mut!((*This).crit));

    HeapFree(GetProcessHeap(), 0 as DWORD, This as LPVOID);

    return ref_0;
}
#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2AImpl_Release(iface: LPDIRECTINPUTDEVICE8A) -> ULONG {
    let mut This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    return IDirectInputDevice2WImpl_Release(IDirectInputDevice8W_from_impl(This) as LPDIRECTINPUTDEVICE8W);
}

#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2WImpl_QueryInterface(iface: LPDIRECTINPUTDEVICE8W, riid: *const IID, ppobj: *mut LPVOID) -> HRESULT {
    let mut This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8W(iface as *mut IDirectInputDevice8W);

    if IsEqualGUID!(core::ptr::addr_of!(IID_IUnknown), riid) != false ||
        IsEqualGUID!(core::ptr::addr_of!(IID_IDirectInputDeviceA), riid) != false ||
        IsEqualGUID!(core::ptr::addr_of!(IID_IDirectInputDevice2A), riid) != false ||
        IsEqualGUID!(core::ptr::addr_of!(IID_IDirectInputDevice7A), riid) != false ||
        IsEqualGUID!(core::ptr::addr_of!(IID_IDirectInputDevice8A), riid) != false
    {
        IDirectInputDevice2_AddRef!(iface as *mut IDirectInputDevice8W);
        *ppobj = IDirectInputDevice8A_from_impl(This) as LPVOID;
        return DI_OK;
    }
    if IsEqualGUID!(core::ptr::addr_of!(IID_IDirectInputDeviceW), riid) != false ||
        IsEqualGUID!(core::ptr::addr_of!(IID_IDirectInputDevice2W), riid) != false ||
        IsEqualGUID!(core::ptr::addr_of!(IID_IDirectInputDevice7W), riid) != false ||
        IsEqualGUID!(core::ptr::addr_of!(IID_IDirectInputDevice8W), riid) != false
    {
        IDirectInputDevice2_AddRef!(iface as *mut IDirectInputDevice8W);
        *ppobj = IDirectInputDevice8W_from_impl(This) as LPVOID;
        return DI_OK;
    }

    WARN!("Unsupported interface!\n");
    return E_NOINTERFACE;
}

#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2AImpl_QueryInterface(iface: LPDIRECTINPUTDEVICE8A, riid: *const IID, ppobj: *mut LPVOID) -> HRESULT {
    let This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    return IDirectInputDevice2WImpl_QueryInterface(
        IDirectInputDevice8W_from_impl(This) as LPDIRECTINPUTDEVICE8W,
        riid,
        ppobj,
    );
}

#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2WImpl_AddRef(iface: LPDIRECTINPUTDEVICE8W) -> ULONG {
    let This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8W(iface as *mut IDirectInputDevice8W);
    let ref_0: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_) as *mut ::core::ffi::c_long) as ULONG;
    return ref_0;
}
#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2AImpl_AddRef(iface: LPDIRECTINPUTDEVICE8A) -> ULONG {
    let mut This: *mut IDirectInputDeviceImpl =
        impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    return IDirectInputDevice2WImpl_AddRef(
        IDirectInputDevice8W_from_impl(This) as LPDIRECTINPUTDEVICE8W
    );
}
#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2AImpl_EnumObjects(iface: LPDIRECTINPUTDEVICE8A, lpCallback: LPDIENUMDEVICEOBJECTSCALLBACKA, lpvRef: LPVOID, dwFlags: DWORD) -> HRESULT {
    let This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8A(iface);
    let mut ddoi: DIDEVICEOBJECTINSTANCEA = core::mem::zeroed();
    let mut i: i32;

    _dump_EnumObjects_flags(dwFlags);

    /* Only the fields till dwFFMaxForce are relevant */
    memset(
        &raw mut ddoi as *mut core::ffi::c_void,
        0 as i32,
        core::mem::size_of::<DIDEVICEOBJECTINSTANCEA>(),
    );
    ddoi.dwSize = FIELD_OFFSET!(DIDEVICEOBJECTINSTANCEA, dwFFMaxForce) as DWORD;

    i = 0 as i32;
    'items: while (i as DWORD) < (*(*This).data_format.wine_df).dwNumObjs {
        let odf: LPDIOBJECTDATAFORMAT =
            dataformat_to_odf((*This).data_format.wine_df as LPCDIDATAFORMAT, i);
        'body: {
            if dwFlags != DIDFT_ALL
                && (dwFlags & (DIDFT_GETTYPE!((*odf).dwType) as DWORD)) == 0
            {
                break 'body;
            }
            if IDirectInputDevice_GetObjectInfo!(
                iface,
                &raw mut ddoi,
                (*odf).dwType,
                DIPH_BYID,
            ) != DI_OK
            {
                break 'body;
            }
            if (lpCallback.unwrap_unchecked())(&raw mut ddoi as LPCDIDEVICEOBJECTINSTANCEA, lpvRef)
                != (DIENUM_CONTINUE as i32)
            {
                break 'items;
            }
        }
        i += 1;
    }

    return DI_OK;
}
#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2WImpl_EnumObjects(iface: LPDIRECTINPUTDEVICE8W, lpCallback: LPDIENUMDEVICEOBJECTSCALLBACKW, lpvRef: LPVOID, dwFlags: DWORD) -> HRESULT {
    let This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8W(iface);
    let mut ddoi: DIDEVICEOBJECTINSTANCEW = core::mem::zeroed();
    let mut i: ::core::ffi::c_int;

    _dump_EnumObjects_flags(dwFlags);

    /* Only the fields till dwFFMaxForce are relevant */
    memset(&raw mut ddoi as *mut ::core::ffi::c_void, 0, ::core::mem::size_of::<DIDEVICEOBJECTINSTANCEW>() as size_t);
    ddoi.dwSize = core::mem::offset_of!(DIDEVICEOBJECTINSTANCEW, dwFFMaxForce) as DWORD;

    i = 0;
    'objects: while (i as DWORD) < (*(*This).data_format.wine_df).dwNumObjs {
        let odf: LPDIOBJECTDATAFORMAT = dataformat_to_odf((*This).data_format.wine_df, i);

        'next: {
            if dwFlags != DIDFT_ALL && (dwFlags & (DIDFT_GETTYPE!((*odf).dwType) as DWORD)) == 0 {
                break 'next;
            }
            if IDirectInputDevice_GetObjectInfo!(iface, &raw mut ddoi, (*odf).dwType, DIPH_BYID) != DI_OK {
                break 'next;
            }

            if (lpCallback.unwrap_unchecked())(&raw mut ddoi, lpvRef) != (DIENUM_CONTINUE as i32) {
                break 'objects;
            }
        }
        i += 1;
    }

    return DI_OK;
}
#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2WImpl_GetProperty(iface: LPDIRECTINPUTDEVICE8W, rguid: REFGUID, pdiph: LPDIPROPHEADER) -> HRESULT {
    let This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8W(iface);

    _dump_DIPROPHEADER(pdiph);

    if !IS_DIPROP!(rguid) {
        return DI_OK;
    }

    let sel: DWORD_PTR = LOWORD!(rguid) as DWORD_PTR;
    match sel {
        x if x == (DIPROP_BUFFERSIZE as DWORD_PTR) => {
            let pd: LPDIPROPDWORD = pdiph as LPDIPROPDWORD;

            if (*pdiph).dwSize as usize != core::mem::size_of::<DIPROPDWORD>() {
                return DIERR_INVALIDPARAM;
            }

            (*pd).dwData = (*This).queue_len as DWORD;
        }
        x if x == (DIPROP_USERNAME as DWORD_PTR) => {
            let ps: LPDIPROPSTRING = pdiph as LPDIPROPSTRING;
            let mut device_player: *mut DevicePlayer;

            if (*pdiph).dwSize as usize != core::mem::size_of::<DIPROPSTRING>() {
                return DIERR_INVALIDPARAM;
            }

            device_player = ((*(*This).dinput).device_players.next as *mut core::ffi::c_char)
                .offset(-(core::mem::offset_of!(DevicePlayer, entry) as isize))
                as *mut DevicePlayer;
            while &raw mut (*device_player).entry != &raw mut (*(*This).dinput).device_players {
                if IsEqualGUID!(
                    &raw mut (*device_player).instance_guid as *const GUID,
                    &raw mut (*This).guid as *const GUID,
                ) != false
                {
                    if *(&raw mut (*device_player).username as *mut WCHAR) != 0 {
                        lstrcpynW(
                            &raw mut (*ps).wsz as LPWSTR,
                            &raw mut (*device_player).username as *mut WCHAR as LPCWSTR,
                            ARRAY_SIZE!((*ps).wsz) as core::ffi::c_int,
                        );
                        return DI_OK;
                    } else {
                        break;
                    }
                }
                device_player = ((*device_player).entry.next as *mut core::ffi::c_char)
                    .offset(-(core::mem::offset_of!(DevicePlayer, entry) as isize))
                    as *mut DevicePlayer;
            }
            return S_FALSE;
        }
        x if x == (DIPROP_VIDPID as DWORD_PTR) => {
            FIXME!("DIPROP_VIDPID not implemented\n");
            return DIERR_UNSUPPORTED;
        }
        _ => {
            FIXME!("Unknown property %s\n", debugstr_guid(rguid));
            return DIERR_INVALIDPARAM;
        }
    }

    return DI_OK;
}
#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2AImpl_GetProperty(iface: LPDIRECTINPUTDEVICE8A, rguid: REFGUID, pdiph: LPDIPROPHEADER) -> HRESULT {
    let mut This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8A(iface);
    return IDirectInputDevice2WImpl_GetProperty(IDirectInputDevice8W_from_impl(This), rguid, pdiph);
}
#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2WImpl_SetProperty(iface: LPDIRECTINPUTDEVICE8W, rguid: *const GUID, pdiph: LPCDIPROPHEADER) -> HRESULT {
    let This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8W(iface);

    _dump_DIPROPHEADER(pdiph);

    if !IS_DIPROP!(rguid) {
        return DI_OK;
    }

    match LOWORD!(rguid as DWORD_PTR) as DWORD_PTR {
        v if v == (DIPROP_AXISMODE as DWORD_PTR) => {
            let pd: LPCDIPROPDWORD = pdiph as LPCDIPROPDWORD;

            if (*pdiph).dwSize as usize != core::mem::size_of::<DIPROPDWORD>() {
                return DIERR_INVALIDPARAM;
            }
            if (*pdiph).dwHow == (DIPH_DEVICE as DWORD) && (*pdiph).dwObj != 0 {
                return DIERR_INVALIDPARAM;
            }
            if (*This).acquired != 0 {
                return DIERR_ACQUIRED;
            }
            if (*pdiph).dwHow != (DIPH_DEVICE as DWORD) {
                return DIERR_UNSUPPORTED;
            }
            if (*This).data_format.user_df.is_null() {
                return DI_OK;
            }

            EnterCriticalSection(core::ptr::addr_of_mut!((*This).crit));
            (*(*This).data_format.user_df).dwFlags &= !(DIDFT_AXIS as DWORD);
            (*(*This).data_format.user_df).dwFlags |= (if (*pd).dwData == (DIPROPAXISMODE_ABS as DWORD) {
                DIDF_ABSAXIS as DWORD
            } else {
                DIDF_RELAXIS as DWORD
            }) as DWORD;
            LeaveCriticalSection(core::ptr::addr_of_mut!((*This).crit));
        }
        v if v == (DIPROP_BUFFERSIZE as DWORD_PTR) => {
            let pd: LPCDIPROPDWORD = pdiph as LPCDIPROPDWORD;

            if (*pdiph).dwSize as usize != core::mem::size_of::<DIPROPDWORD>() {
                return DIERR_INVALIDPARAM;
            }
            if (*This).acquired != 0 {
                return DIERR_ACQUIRED;
            }

            EnterCriticalSection(core::ptr::addr_of_mut!((*This).crit));
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                (*This).data_queue as PVOID,
            );

            (*This).data_queue = (if (*pd).dwData == 0 {
                core::ptr::null_mut()
            } else {
                HeapAlloc(
                    GetProcessHeap(),
                    0 as DWORD,
                    ((*pd).dwData as SIZE_T)
                        .wrapping_mul(core::mem::size_of::<DIDEVICEOBJECTDATA>() as SIZE_T),
                )
            }) as LPDIDEVICEOBJECTDATA;
            (*This).overflow = 0 as BOOL;
            (*This).queue_tail = (*This).overflow as ::core::ffi::c_int;
            (*This).queue_head = (*This).queue_tail;
            (*This).queue_len = (*pd).dwData as ::core::ffi::c_int;

            LeaveCriticalSection(core::ptr::addr_of_mut!((*This).crit));
        }
        v if v == (DIPROP_USERNAME as DWORD_PTR) => {
            let ps: LPCDIPROPSTRING = pdiph as LPCDIPROPSTRING;
            let mut device_player: *mut DevicePlayer = core::ptr::null_mut();
            let mut found: BOOL = FALSE as BOOL;

            if (*pdiph).dwSize as usize != core::mem::size_of::<DIPROPSTRING>() {
                return DIERR_INVALIDPARAM;
            }

            // LIST_FOR_EACH_ENTRY(device_player, &This->dinput->device_players, struct DevicePlayer, entry)
            device_player = ((*(*This).dinput).device_players.next as *mut u8)
                .sub(core::mem::offset_of!(DevicePlayer, entry)) as *mut DevicePlayer;
            while core::ptr::addr_of_mut!((*device_player).entry)
                != core::ptr::addr_of_mut!((*(*This).dinput).device_players)
            {
                if IsEqualGUID!(
                    core::ptr::addr_of!((*device_player).instance_guid),
                    core::ptr::addr_of!((*This).guid),
                ) != false
                {
                    found = TRUE as BOOL;
                    break;
                }
                device_player = ((*device_player).entry.next as *mut u8)
                    .sub(core::mem::offset_of!(DevicePlayer, entry)) as *mut DevicePlayer;
            }
            if found == 0 && {
                device_player = HeapAlloc(
                    GetProcessHeap(),
                    0 as DWORD,
                    core::mem::size_of::<DevicePlayer>() as SIZE_T,
                ) as *mut DevicePlayer;
                !device_player.is_null()
            } {
                list_add_tail(
                    core::ptr::addr_of_mut!((*(*This).dinput).device_players),
                    core::ptr::addr_of_mut!((*device_player).entry),
                );
                (*device_player).instance_guid = (*This).guid;
            }
            if !device_player.is_null() {
                lstrcpynW(
                    core::ptr::addr_of_mut!((*device_player).username) as LPWSTR,
                    core::ptr::addr_of!((*ps).wsz) as LPCWSTR,
                    (core::mem::size_of::<[WCHAR; 260]>() / core::mem::size_of::<WCHAR>())
                        as ::core::ffi::c_int,
                );
            }
        }
        _ => {
            WARN!(
                "Unknown property %s\n",
                debugstr_guid(rguid)
            );
            return DIERR_UNSUPPORTED;
        }
    }

    return DI_OK;
}
#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2AImpl_SetProperty(iface: LPDIRECTINPUTDEVICE8A, rguid: *const GUID, pdiph: LPCDIPROPHEADER) -> HRESULT {
    let This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    return IDirectInputDevice2WImpl_SetProperty(IDirectInputDevice8W_from_impl(This) as LPDIRECTINPUTDEVICE8W, rguid, pdiph);
}

#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2AImpl_GetObjectInfo(
    iface: LPDIRECTINPUTDEVICE8A,
    pdidoi: LPDIDEVICEOBJECTINSTANCEA,
    dwObj: DWORD,
    dwHow: DWORD,
) -> HRESULT {
    let This: *mut IDirectInputDeviceImpl =
        impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    let mut didoiW: DIDEVICEOBJECTINSTANCEW = core::mem::zeroed();
    let mut res: HRESULT = 0;

    if pdidoi.is_null()
        || ((*pdidoi).dwSize as usize != ::core::mem::size_of::<DIDEVICEOBJECTINSTANCEA>()
            && (*pdidoi).dwSize as usize
                != ::core::mem::size_of::<DIDEVICEOBJECTINSTANCE_DX3A>())
    {
        return DIERR_INVALIDPARAM;
    }

    didoiW.dwSize = ::core::mem::size_of::<DIDEVICEOBJECTINSTANCEW>() as DWORD;
    res = IDirectInputDevice2WImpl_GetObjectInfo(
        IDirectInputDevice8W_from_impl(This) as LPDIRECTINPUTDEVICE8W,
        &raw mut didoiW,
        dwObj,
        dwHow,
    );
    if res == DI_OK {
        let mut dwSize: DWORD = (*pdidoi).dwSize;

        memset(
            pdidoi as *mut ::core::ffi::c_void,
            0,
            (*pdidoi).dwSize as usize,
        );
        (*pdidoi).dwSize = dwSize;
        (*pdidoi).guidType = didoiW.guidType;
        (*pdidoi).dwOfs = didoiW.dwOfs;
        (*pdidoi).dwType = didoiW.dwType;
        (*pdidoi).dwFlags = didoiW.dwFlags;
    }

    return res;
}

#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2WImpl_GetObjectInfo(
    iface: LPDIRECTINPUTDEVICE8W,
    pdidoi: LPDIDEVICEOBJECTINSTANCEW,
    mut dwObj: DWORD,
    mut dwHow: DWORD,
) -> HRESULT {
    let mut This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8W(iface as *mut IDirectInputDevice8W);
    let mut dwSize: DWORD = 0;
    let mut odf: LPDIOBJECTDATAFORMAT = core::ptr::null_mut();
    let mut idx: ::core::ffi::c_int = -1;

    if pdidoi.is_null()
        || ((*pdidoi).dwSize as usize != ::core::mem::size_of::<DIDEVICEOBJECTINSTANCEW>() as usize
            && (*pdidoi).dwSize as usize != ::core::mem::size_of::<DIDEVICEOBJECTINSTANCE_DX3W>() as usize)
    {
        return DIERR_INVALIDPARAM;
    }

    if dwHow == DIPH_BYOFFSET {
        if (*This).data_format.offsets.is_null() {
            // break out of switch
        } else {
            idx = ((*(*This).data_format.wine_df).dwNumObjs.wrapping_sub(1 as DWORD))
                as ::core::ffi::c_int;
            while idx >= 0 as ::core::ffi::c_int {
                if (*(*This).data_format.offsets.offset(idx as isize)) as DWORD == dwObj {
                    break;
                }
                idx -= 1;
            }
        }
    } else if dwHow == DIPH_BYID {
        dwObj &= 0x00ffffff as DWORD;
        idx = ((*(*This).data_format.wine_df).dwNumObjs.wrapping_sub(1 as DWORD))
            as ::core::ffi::c_int;
        while idx >= 0 as ::core::ffi::c_int {
            if ((*dataformat_to_odf((*This).data_format.wine_df as LPCDIDATAFORMAT, idx)).dwType
                & 0x00ffffff as DWORD)
                == dwObj
            {
                break;
            }
            idx -= 1;
        }
    } else if dwHow == DIPH_BYUSAGE {
        FIXME!("dwHow = DIPH_BYUSAGE not implemented\n");
    } else {
        WARN!("invalid parameter: dwHow = %08x\n", dwHow);
        return DIERR_INVALIDPARAM;
    }

    if idx < 0 as ::core::ffi::c_int {
        return DIERR_OBJECTNOTFOUND;
    }

    odf = dataformat_to_odf((*This).data_format.wine_df as LPCDIDATAFORMAT, idx);
    dwSize = (*pdidoi).dwSize;
    memset(
        pdidoi as *mut ::core::ffi::c_void,
        0,
        (*pdidoi).dwSize as usize,
    );
    (*pdidoi).dwSize = dwSize;
    if !(*odf).pguid.is_null() {
        (*pdidoi).guidType = *(*odf).pguid;
    }
    (*pdidoi).dwOfs = if !(*This).data_format.offsets.is_null() {
        *(*This).data_format.offsets.offset(idx as isize) as DWORD
    } else {
        (*odf).dwOfs
    };
    (*pdidoi).dwType = (*odf).dwType;
    (*pdidoi).dwFlags = (*odf).dwFlags;

    return DI_OK;
}
#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2WImpl_GetDeviceData(iface: LPDIRECTINPUTDEVICE8W, dodsize: DWORD, dod: LPDIDEVICEOBJECTDATA, entries: LPDWORD, flags: DWORD) -> HRESULT {
    let This: *mut IDirectInputDeviceImpl =
        impl_from_IDirectInputDevice8W(iface as *mut IDirectInputDevice8W);
    let mut ret: HRESULT = DI_OK as HRESULT;
    let mut len: ::core::ffi::c_int = 0;
    WINE_TRACE!("(%p) %p -> %p(%d) x%d, 0x%08x\n", This, dod, entries, if !entries.is_null() { *entries } else { 0 as DWORD }, dodsize, flags);
    if (*(*This).dinput).dwVersion == 0x0800 as DWORD
        || dodsize as usize == ::core::mem::size_of::<DIDEVICEOBJECTDATA_DX3>() as usize
    {
        if (*This).queue_len == 0 {
            return DIERR_NOTBUFFERED as HRESULT;
        }
        if (*This).acquired == 0 {
            return DIERR_NOTACQUIRED as HRESULT;
        }
    }
    if (*This).queue_len == 0 {
        return DI_OK as HRESULT;
    }
    if (dodsize as usize) < ::core::mem::size_of::<DIDEVICEOBJECTDATA_DX3>() as usize {
        return DIERR_INVALIDPARAM as HRESULT;
    }
    IDirectInputDevice2_Poll!(iface);
    EnterCriticalSection(&raw mut (*This).crit);
    len = (*This).queue_head - (*This).queue_tail;
    if len < 0 as ::core::ffi::c_int {
        len += (*This).queue_len;
    }
    if (*entries != INFINITE as DWORD) && (len as DWORD > *entries) {
        len = *entries as ::core::ffi::c_int;
    }
    if !dod.is_null() {
        let mut i: ::core::ffi::c_int = 0;
        i = 0 as ::core::ffi::c_int;
        while i < len {
            let mut n: ::core::ffi::c_int = ((*This).queue_tail + i) % (*This).queue_len;
            memcpy(
                (dod as *mut ::core::ffi::c_char).offset(dodsize.wrapping_mul(i as DWORD) as isize)
                    as *mut ::core::ffi::c_void,
                (*This).data_queue.offset(n as isize) as *const ::core::ffi::c_void,
                dodsize as usize,
            );
            i += 1;
        }
    }
    *entries = len as DWORD;
    if (*This).overflow != 0 && (*(*This).dinput).dwVersion == 0x0800 as DWORD {
        ret = DI_BUFFEROVERFLOW as HRESULT;
    }
    if (flags & (DIGDD_PEEK as DWORD)) == 0 {
        (*This).queue_tail = ((*This).queue_tail + len) % (*This).queue_len;
        (*This).overflow = FALSE as BOOL;
    }
    LeaveCriticalSection(&raw mut (*This).crit);
    WINE_TRACE!("Returning %d events queued\n", *entries);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2AImpl_GetDeviceData(iface: LPDIRECTINPUTDEVICE8A, dodsize: DWORD, dod: LPDIDEVICEOBJECTDATA, entries: LPDWORD, flags: DWORD) -> HRESULT {
    let mut This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    return IDirectInputDevice2WImpl_GetDeviceData(IDirectInputDevice8W_from_impl(This) as LPDIRECTINPUTDEVICE8W, dodsize, dod, entries, flags);
}
#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2WImpl_RunControlPanel(mut iface: LPDIRECTINPUTDEVICE8W, mut hwndOwner: HWND, mut dwFlags: DWORD) -> HRESULT {
    let mut This: *mut IDirectInputDeviceImpl =
        impl_from_IDirectInputDevice8W(iface as *mut IDirectInputDevice8W);
    DPRINT1!(
        "%p)->(%p,0x%08x): stub!\n",
        This,
        hwndOwner,
        dwFlags
    );
    return DI_OK;
}

#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2AImpl_RunControlPanel(iface: LPDIRECTINPUTDEVICE8A, hwndOwner: HWND, dwFlags: DWORD) -> HRESULT {
    let mut This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    return IDirectInputDevice2WImpl_RunControlPanel(IDirectInputDevice8W_from_impl(This) as LPDIRECTINPUTDEVICE8W, hwndOwner, dwFlags);
}

#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2WImpl_Initialize(iface: LPDIRECTINPUTDEVICE8W, hinst: HINSTANCE, dwVersion: DWORD, rguid: *const GUID) -> HRESULT {
    let This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8W(iface as *mut IDirectInputDevice8W);
    let _ = (This, hinst, dwVersion, rguid);
    return DI_OK;
}

#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2AImpl_Initialize(iface: LPDIRECTINPUTDEVICE8A, hinst: HINSTANCE, dwVersion: DWORD, rguid: *const GUID) -> HRESULT {
    let mut This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    return IDirectInputDevice2WImpl_Initialize(IDirectInputDevice8W_from_impl(This) as LPDIRECTINPUTDEVICE8W, hinst, dwVersion, rguid);
}
#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2WImpl_CreateEffect(iface: LPDIRECTINPUTDEVICE8W, rguid: *const GUID, lpeff: LPCDIEFFECT, ppdef: *mut LPDIRECTINPUTEFFECT, pUnkOuter: LPUNKNOWN) -> HRESULT {
    let mut This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8W(iface as *mut IDirectInputDevice8W);
    WINE_FIXME!("(%p)->(%s,%p,%p,%p): stub!\n", This, debugstr_guid(rguid as *const _GUID), lpeff, ppdef, pUnkOuter);
    WINE_FIXME!("not available in the generic implementation\n");
    *ppdef = core::ptr::null_mut::<IDirectInputEffect>();
    return DIERR_UNSUPPORTED;
}

#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2AImpl_CreateEffect(iface: LPDIRECTINPUTDEVICE8A, rguid: *const GUID, lpeff: LPCDIEFFECT, ppdef: *mut LPDIRECTINPUTEFFECT, pUnkOuter: LPUNKNOWN) -> HRESULT {
    let This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    return IDirectInputDevice2WImpl_CreateEffect(
        IDirectInputDevice8W_from_impl(This) as LPDIRECTINPUTDEVICE8W,
        rguid,
        lpeff,
        ppdef,
        pUnkOuter,
    );
}
#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2AImpl_EnumEffects(mut iface: LPDIRECTINPUTDEVICE8A, mut lpCallback: LPDIENUMEFFECTSCALLBACKA, mut lpvRef: LPVOID, mut dwFlags: DWORD) -> HRESULT {
    let mut This: *mut IDirectInputDeviceImpl =
        impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    WINE_FIXME!("%p)->(%p,%p,0x%08x): stub!\n", This, lpCallback, lpvRef, dwFlags);
    return DI_OK;
}
#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2WImpl_EnumEffects(mut iface: LPDIRECTINPUTDEVICE8W, mut lpCallback: LPDIENUMEFFECTSCALLBACKW, mut lpvRef: LPVOID, mut dwFlags: DWORD) -> HRESULT {
    let mut This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8W(iface as *mut IDirectInputDevice8W);
    WINE_FIXME!("(%p)->(%p,%p,0x%08x): stub!\n", This, lpCallback, lpvRef, dwFlags);
    return DI_OK;
}

#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2AImpl_GetEffectInfo(mut iface: LPDIRECTINPUTDEVICE8A, mut lpdei: LPDIEFFECTINFOA, rguid: *const GUID) -> HRESULT {
    let mut This: *mut IDirectInputDeviceImpl =
        impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    WINE_FIXME!("(%p)->(%p,%s): stub!\n", This, lpdei, debugstr_guid(rguid));
    return DI_OK;
}
#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2WImpl_GetEffectInfo(mut iface: LPDIRECTINPUTDEVICE8W, mut lpdei: LPDIEFFECTINFOW, rguid: *const GUID) -> HRESULT {
    let mut This: *mut IDirectInputDeviceImpl =
        impl_from_IDirectInputDevice8W(iface as *mut IDirectInputDevice8W);
    WINE_FIXME!("(%p)->(%p,%s): stub!\n", This, lpdei, debugstr_guid(rguid as *const _GUID));
    return DI_OK;
}
#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2WImpl_GetForceFeedbackState(iface: LPDIRECTINPUTDEVICE8W, pdwOut: LPDWORD) -> HRESULT {
    let mut This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8W(iface as *mut IDirectInputDevice8W);
    DPRINT1!("(%p)->(%p): stub!\n", This, pdwOut);
    return DI_OK;
}
#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2AImpl_GetForceFeedbackState(iface: LPDIRECTINPUTDEVICE8A, pdwOut: LPDWORD) -> HRESULT {
    let mut This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    return IDirectInputDevice2WImpl_GetForceFeedbackState(IDirectInputDevice8W_from_impl(This) as LPDIRECTINPUTDEVICE8W, pdwOut);
}

#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2WImpl_SendForceFeedbackCommand(iface: LPDIRECTINPUTDEVICE8W, dwFlags: DWORD) -> HRESULT {
    let _ = dwFlags;
    let This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8W(iface as *mut IDirectInputDevice8W);
    let _ = This;
    return DI_NOEFFECT;
}

#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2AImpl_SendForceFeedbackCommand(iface: LPDIRECTINPUTDEVICE8A, dwFlags: DWORD) -> HRESULT {
    let This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    return IDirectInputDevice2WImpl_SendForceFeedbackCommand(IDirectInputDevice8W_from_impl(This) as LPDIRECTINPUTDEVICE8W, dwFlags);
}
#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2WImpl_EnumCreatedEffectObjects(mut iface: LPDIRECTINPUTDEVICE8W, mut lpCallback: LPDIENUMCREATEDEFFECTOBJECTSCALLBACK, mut lpvRef: LPVOID, mut dwFlags: DWORD) -> HRESULT {
    let mut This: *mut IDirectInputDeviceImpl =
        impl_from_IDirectInputDevice8W(iface as *mut IDirectInputDevice8W);
    DPRINT1!(
        "(%p)0>(%p,%p,0x%08x): stub!\n",
        This,
        lpCallback,
        lpvRef,
        dwFlags
    );
    return DI_OK;
}
#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2AImpl_EnumCreatedEffectObjects(iface: LPDIRECTINPUTDEVICE8A, lpCallback: LPDIENUMCREATEDEFFECTOBJECTSCALLBACK, lpvRef: LPVOID, dwFlags: DWORD) -> HRESULT {
    let mut This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    return IDirectInputDevice2WImpl_EnumCreatedEffectObjects(
        IDirectInputDevice8W_from_impl(This) as LPDIRECTINPUTDEVICE8W,
        lpCallback,
        lpvRef,
        dwFlags,
    );
}
#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2WImpl_Escape(iface: LPDIRECTINPUTDEVICE8W, lpDIEEsc: LPDIEFFESCAPE) -> HRESULT {
    let mut This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8W(iface as *mut IDirectInputDevice8W);
    WINE_FIXME!("(%p)->(%p): stub!\n", This, lpDIEEsc);
    return DI_OK;
}

#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2AImpl_Escape(iface: LPDIRECTINPUTDEVICE8A, lpDIEEsc: LPDIEFFESCAPE) -> HRESULT {
    let This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    return IDirectInputDevice2WImpl_Escape(IDirectInputDevice8W_from_impl(This) as LPDIRECTINPUTDEVICE8W, lpDIEEsc);
}

#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2WImpl_Poll(iface: LPDIRECTINPUTDEVICE8W) -> HRESULT {
    let mut This: *mut IDirectInputDeviceImpl =
        impl_from_IDirectInputDevice8W(iface as *mut IDirectInputDevice8W);

    if (*This).acquired == 0 {
        return DIERR_NOTACQUIRED;
    }

    check_dinput_events();
    return DI_OK;
}
#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2AImpl_Poll(iface: LPDIRECTINPUTDEVICE8A) -> HRESULT {
    let mut This: *mut IDirectInputDeviceImpl =
        impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    return IDirectInputDevice2WImpl_Poll(
        IDirectInputDevice8W_from_impl(This) as LPDIRECTINPUTDEVICE8W
    );
}
#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2WImpl_SendDeviceData(iface: LPDIRECTINPUTDEVICE8W, cbObjectData: DWORD, rgdod: LPCDIDEVICEOBJECTDATA, pdwInOut: LPDWORD, dwFlags: DWORD) -> HRESULT {
    let mut This: *mut IDirectInputDeviceImpl =
        impl_from_IDirectInputDevice8W(iface as *mut IDirectInputDevice8W);
    WINE_FIXME!("(%p)->(0x%08x,%p,%p,0x%08x): stub!\n", This, cbObjectData, rgdod, pdwInOut, dwFlags);
    return DI_OK;
}

#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice2AImpl_SendDeviceData(mut iface: LPDIRECTINPUTDEVICE8A, mut cbObjectData: DWORD, mut rgdod: LPCDIDEVICEOBJECTDATA, mut pdwInOut: LPDWORD, mut dwFlags: DWORD) -> HRESULT {
    let mut This: *mut IDirectInputDeviceImpl =
        impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    return IDirectInputDevice2WImpl_SendDeviceData(
        IDirectInputDevice8W_from_impl(This) as LPDIRECTINPUTDEVICE8W,
        cbObjectData,
        rgdod,
        pdwInOut,
        dwFlags,
    );
}

#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice7AImpl_EnumEffectsInFile(iface: LPDIRECTINPUTDEVICE8A, lpszFileName: LPCSTR, pec: LPDIENUMEFFECTSINFILECALLBACK, pvRef: LPVOID, dwFlags: DWORD) -> HRESULT {
    let mut This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    WINE_FIXME!("(%p)->(%s,%p,%p,%08x): stub !\n", This, lpszFileName, pec, pvRef, dwFlags);
    return DI_OK;
}

#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice7WImpl_EnumEffectsInFile(iface: LPDIRECTINPUTDEVICE8W, lpszFileName: LPCWSTR, pec: LPDIENUMEFFECTSINFILECALLBACK, pvRef: LPVOID, dwFlags: DWORD) -> HRESULT {
    let mut This: *mut IDirectInputDeviceImpl =
        impl_from_IDirectInputDevice8W(iface as *mut IDirectInputDevice8W);
    WINE_FIXME!("(%p)->(%s,%p,%p,%08x): stub !\n", This, debugstr_w(lpszFileName as *const WCHAR), pec, pvRef, dwFlags);
    return DI_OK;
}

#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice7AImpl_WriteEffectToFile(iface: LPDIRECTINPUTDEVICE8A, lpszFileName: LPCSTR, dwEntries: DWORD, rgDiFileEft: LPDIFILEEFFECT, dwFlags: DWORD) -> HRESULT {
    let This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8A(iface);
    DPRINT1!(
        "(%p)->(%s,%08x,%p,%08x): stub !\n",
        This,
        lpszFileName,
        dwEntries,
        rgDiFileEft,
        dwFlags
    );
    return DI_OK;
}
#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice7WImpl_WriteEffectToFile(mut iface: LPDIRECTINPUTDEVICE8W, mut lpszFileName: LPCWSTR, mut dwEntries: DWORD, mut rgDiFileEft: LPDIFILEEFFECT, mut dwFlags: DWORD) -> HRESULT {
    let mut This: *mut IDirectInputDeviceImpl =
        impl_from_IDirectInputDevice8W(iface as *mut IDirectInputDevice8W);
    WINE_FIXME!("(%p)->(%s,%08x,%p,%08x): stub !\n", This, debugstr_w(lpszFileName as *const WCHAR), dwEntries, rgDiFileEft, dwFlags);
    return DI_OK;
}
#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice8WImpl_BuildActionMap(
    iface: LPDIRECTINPUTDEVICE8W,
    lpdiaf: LPDIACTIONFORMATW,
    lpszUserName: LPCWSTR,
    dwFlags: DWORD,
) -> HRESULT {
    let This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8W(iface as *mut IDirectInputDevice8W);
    if dwFlags & 0x0 as DWORD != 0 { // DIDBAM_DEFAULT
        FIXME!("\tdwFlags =|DIDBAM_DEFAULT\n");
    }
    if dwFlags & 0x1 as DWORD != 0 { // DIDBAM_PRESERVE
        FIXME!("\tdwFlags =|DIDBAM_PRESERVE\n");
    }
    if dwFlags & 0x2 as DWORD != 0 { // DIDBAM_INITIALIZE
        FIXME!("\tdwFlags =|DIDBAM_INITIALIZE\n");
    }
    if dwFlags & 0x4 as DWORD != 0 { // DIDBAM_HWDEFAULTS
        FIXME!("\tdwFlags =|DIDBAM_HWDEFAULTS\n");
    }
    return DI_OK;
}

#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice8AImpl_GetImageInfo(iface: LPDIRECTINPUTDEVICE8A, lpdiDevImageInfoHeader: LPDIDEVICEIMAGEINFOHEADERA) -> HRESULT {
    let This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8A(iface);
    FIXME!("(%p)->(%p): stub !\n", This, lpdiDevImageInfoHeader);
    return DI_OK;
}

#[no_mangle]
pub unsafe extern "system" fn IDirectInputDevice8WImpl_GetImageInfo(iface: LPDIRECTINPUTDEVICE8W, lpdiDevImageInfoHeader: LPDIDEVICEIMAGEINFOHEADERW) -> HRESULT {
    let mut This: *mut IDirectInputDeviceImpl = impl_from_IDirectInputDevice8W(iface as *mut IDirectInputDevice8W);
    DPRINT1!(
        "(%p)->(%p): stub !\n",
        This,
        lpdiDevImageInfoHeader
    );
    return DI_OK;
}
