use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut PID_XBOX_CONTROLLERS: [WORD; 13] = [
    0x202 as ::core::ffi::c_int as WORD,
    0x285 as ::core::ffi::c_int as WORD,
    0x289 as ::core::ffi::c_int as WORD,
    0x28e as ::core::ffi::c_int as WORD,
    0x28f as ::core::ffi::c_int as WORD,
    0x2d1 as ::core::ffi::c_int as WORD,
    0x2dd as ::core::ffi::c_int as WORD,
    0x2e0 as ::core::ffi::c_int as WORD,
    0x2e3 as ::core::ffi::c_int as WORD,
    0x2e6 as ::core::ffi::c_int as WORD,
    0x2ea as ::core::ffi::c_int as WORD,
    0x2fd as ::core::ffi::c_int as WORD,
    0x719 as ::core::ffi::c_int as WORD,
];

#[no_mangle]
pub static mut DInput_PIDVID_Product_GUID: GUID = _GUID {
    Data1: 0 as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0 as ::core::ffi::c_ushort,
    Data4: [
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x50 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x49 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x44 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x56 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x49 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x44 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub const VID_MICROSOFT: u32 = 0x045e;
unsafe extern "C" fn impl_from_IDirectInputDevice8A(iface: *mut IDirectInputDevice8A) -> *mut JoystickGenericImpl {
    let inner: *mut IDirectInputDeviceImpl = (iface as *mut u8)
        .sub(core::mem::offset_of!(IDirectInputDeviceImpl, IDirectInputDevice8A_iface)) as *mut IDirectInputDeviceImpl;
    return (inner as *mut u8).sub(core::mem::offset_of!(JoystickGenericImpl, base)) as *mut JoystickGenericImpl;
}

unsafe extern "C" fn impl_from_IDirectInputDevice8W(iface: *mut IDirectInputDevice8W) -> *mut JoystickGenericImpl {
    let inner: *mut IDirectInputDeviceImpl = (iface as *mut u8)
        .wrapping_sub(core::mem::offset_of!(IDirectInputDeviceImpl, IDirectInputDevice8W_iface))
        as *mut IDirectInputDeviceImpl;
    return (inner as *mut u8).wrapping_sub(core::mem::offset_of!(JoystickGenericImpl, base))
        as *mut JoystickGenericImpl;
}

unsafe extern "C" fn IDirectInputDevice8A_from_impl(This: *mut JoystickGenericImpl) -> *mut IDirectInputDevice8A {
    return core::ptr::addr_of_mut!((*This).base.IDirectInputDevice8A_iface);
}

unsafe extern "C" fn IDirectInputDevice8W_from_impl(This: *mut JoystickGenericImpl) -> *mut IDirectInputDevice8W {
    return core::ptr::addr_of_mut!((*This).base.IDirectInputDevice8W_iface);
}
#[no_mangle]
pub unsafe extern "C" fn typeFromGUID(guid: *const GUID) -> DWORD {
    if IsEqualGUID!(guid, core::ptr::addr_of!(GUID_ConstantForce)) != false {
        return DIEFT_CONSTANTFORCE as DWORD;
    } else if IsEqualGUID!(guid, core::ptr::addr_of!(GUID_Square)) != false
        || IsEqualGUID!(guid, core::ptr::addr_of!(GUID_Sine)) != false
        || IsEqualGUID!(guid, core::ptr::addr_of!(GUID_Triangle)) != false
        || IsEqualGUID!(guid, core::ptr::addr_of!(GUID_SawtoothUp)) != false
        || IsEqualGUID!(guid, core::ptr::addr_of!(GUID_SawtoothDown)) != false
    {
        return DIEFT_PERIODIC as DWORD;
    } else if IsEqualGUID!(guid, core::ptr::addr_of!(GUID_RampForce)) != false {
        return DIEFT_RAMPFORCE as DWORD;
    } else if IsEqualGUID!(guid, core::ptr::addr_of!(GUID_Spring)) != false
        || IsEqualGUID!(guid, core::ptr::addr_of!(GUID_Damper)) != false
        || IsEqualGUID!(guid, core::ptr::addr_of!(GUID_Inertia)) != false
        || IsEqualGUID!(guid, core::ptr::addr_of!(GUID_Friction)) != false
    {
        return DIEFT_CONDITION as DWORD;
    } else if IsEqualGUID!(guid, core::ptr::addr_of!(GUID_CustomForce)) != false {
        return DIEFT_CUSTOMFORCE as DWORD;
    } else {
        WARN!(
            "GUID (%s) is not a known force type\n",
            _dump_dinput_GUID(guid)
        );
        return 0 as DWORD;
    }
}
#[no_mangle]
pub unsafe extern "C" fn get_device_type(mut version: DWORD, mut is_joystick: BOOL) -> DWORD {
    if is_joystick != 0 {
        return if version >= 0x0800 {
            DI8DEVTYPE_JOYSTICK | (DI8DEVTYPEJOYSTICK_STANDARD << 8)
        } else {
            DIDEVTYPE_JOYSTICK | (DIDEVTYPEJOYSTICK_TRADITIONAL << 8)
        };
    }

    return if version >= 0x0800 {
        DI8DEVTYPE_GAMEPAD | (DI8DEVTYPEJOYSTICK_STANDARD << 8)
    } else {
        DIDEVTYPE_JOYSTICK | (DIDEVTYPEJOYSTICK_GAMEPAD << 8)
    };
}

unsafe extern "C" fn _dump_DIEFFECT_flags(dwFlags: DWORD) {
    #[allow(non_camel_case_types)]
    struct FlagEntry {
        mask: DWORD,
        name: *const ::core::ffi::c_char,
    }

    if ((TRACE_ON!(__wine_dbch_dinput)) != 0) {
        let mut i: ::core::ffi::c_uint;
        let flags: [FlagEntry; 5] = [
            FlagEntry {
                mask: DIEFF_CARTESIAN as DWORD,
                name: concat!(stringify!(DIEFF_CARTESIAN), "\0").as_ptr() as *const ::core::ffi::c_char,
            },
            FlagEntry {
                mask: DIEFF_OBJECTIDS as DWORD,
                name: concat!(stringify!(DIEFF_OBJECTIDS), "\0").as_ptr() as *const ::core::ffi::c_char,
            },
            FlagEntry {
                mask: DIEFF_OBJECTOFFSETS as DWORD,
                name: concat!(stringify!(DIEFF_OBJECTOFFSETS), "\0").as_ptr() as *const ::core::ffi::c_char,
            },
            FlagEntry {
                mask: DIEFF_POLAR as DWORD,
                name: concat!(stringify!(DIEFF_POLAR), "\0").as_ptr() as *const ::core::ffi::c_char,
            },
            FlagEntry {
                mask: 0x40 as DWORD, // DIEFF_SPHERICAL
                name: b"DIEFF_SPHERICAL\0" as *const u8 as *const ::core::ffi::c_char,
            },
        ];
        i = 0 as ::core::ffi::c_uint;
        while (i as usize) < ARRAY_SIZE!(flags) as usize {
            if flags[i as usize].mask & dwFlags != 0 {
                TRACE!("%s ", flags[i as usize].name);
            }
            i = i.wrapping_add(1);
        }
    }
}

unsafe extern "C" fn _dump_DIENVELOPE(env: LPCDIENVELOPE) {
    if (*env).dwSize as usize != core::mem::size_of::<DIENVELOPE>() {
        WARN!("Non-standard DIENVELOPE structure size %d.\n", (*env).dwSize);
    }
    TRACE!(
        "Envelope has attack (level: %d time: %d), fade (level: %d time: %d)\n",
        (*env).dwAttackLevel,
        (*env).dwAttackTime,
        (*env).dwFadeLevel,
        (*env).dwFadeTime
    );
}

unsafe extern "C" fn _dump_DICONSTANTFORCE(frc: LPCDICONSTANTFORCE) {
    // TRACE("Constant force has magnitude %d\n", frc->lMagnitude);
}

unsafe extern "C" fn _dump_DIPERIODIC(
    frc: LPCDIPERIODIC,
) {
    // TRACE("Periodic force has magnitude %d, offset %d, phase %d, period %d\n", frc->dwMagnitude, frc->lOffset, frc->dwPhase, frc->dwPeriod);
}

unsafe extern "C" fn _dump_DIRAMPFORCE(frc: LPCDIRAMPFORCE) {
    // TRACE("Ramp force has start %d, end %d\n", frc->lStart, frc->lEnd);
}

unsafe extern "C" fn _dump_DICONDITION(
    frc: LPCDICONDITION,
) {
    // TRACE("Condition has offset %d, pos/neg coefficients %d and %d, pos/neg saturations %d and %d, deadband %d\n", frc->lOffset, frc->lPositiveCoefficient, frc->lNegativeCoefficient, frc->dwPositiveSaturation, frc->dwNegativeSaturation, frc->lDeadBand);
}

unsafe extern "C" fn _dump_DICUSTOMFORCE(frc: LPCDICUSTOMFORCE) {
    let mut i: u32;
    if (*frc).cSamples % (*frc).cChannels != 0 {
        WARN!("Custom force has a non-integral samples-per-channel count!\n");
    }
    if ((TRACE_ON!(__wine_dbch_dinput)) != 0) {
        i = 1;
        while i <= (*frc).cSamples {
            if i % (*frc).cChannels == 0 {
                TRACE!("\n");
            }
            i += 1;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn dump_DIEFFECT(eff: LPCDIEFFECT, guid: REFGUID, dwFlags: DWORD) {
    let mut type_0: DWORD = typeFromGUID(guid);
    let mut i: core::ffi::c_uint = 0;

    if ((*eff).dwSize as usize != core::mem::size_of::<DIEFFECT>())
        && ((*eff).dwSize as usize != core::mem::size_of::<DIEFFECT_DX5>())
    {
        WARN!(
            b"Non-standard DIEFFECT structure size %d\n\0".as_ptr() as *const i8,
            (*eff).dwSize,
        );
    }

    _dump_DIEFFECT_flags((*eff).dwFlags);

    if (*eff).dwGain > (10000 as DWORD) {
        WARN!(b"dwGain is out of range (>10,000)\n\0".as_ptr() as *const i8);
    }

    TRACE!(b"  - dwDuration: %d\n\0".as_ptr() as *const i8, (*eff).dwDuration);
    TRACE!(b"  - dwGain: %d\n\0".as_ptr() as *const i8, (*eff).dwGain);
    TRACE!(b"  - dwTriggerButton: %d\n\0".as_ptr() as *const i8, (*eff).dwTriggerButton);
    TRACE!(
        b"  - dwTriggerRepeatInterval: %d\n\0".as_ptr() as *const i8,
        (*eff).dwTriggerRepeatInterval,
    );
    TRACE!(b"  - rglDirection: %p\n\0".as_ptr() as *const i8, (*eff).rglDirection);
    TRACE!(
        b"  - cbTypeSpecificParams: %d\n\0".as_ptr() as *const i8,
        (*eff).cbTypeSpecificParams,
    );
    TRACE!(
        b"  - lpvTypeSpecificParams: %p\n\0".as_ptr() as *const i8,
        (*eff).lpvTypeSpecificParams,
    );

    /* Only trace some members if dwFlags indicates they have data */
    if (dwFlags & (DIEP_AXES as DWORD)) != 0 {
        TRACE!(b"  - cAxes: %d\n\0".as_ptr() as *const i8, (*eff).cAxes);
        TRACE!(b"  - rgdwAxes: %p\n\0".as_ptr() as *const i8, (*eff).rgdwAxes);

        if ((TRACE_ON!(__wine_dbch_dinput)) != 0) && !(*eff).rgdwAxes.is_null() {
            TRACE!(b"    \0".as_ptr() as *const i8);
            i = 0 as core::ffi::c_uint;
            while (i as DWORD) < (*eff).cAxes {
                TRACE!(
                    b"%d \0".as_ptr() as *const i8,
                    *(*eff).rgdwAxes.add(i as usize),
                );
                i = i.wrapping_add(1);
            }
            TRACE!(b"\n\0".as_ptr() as *const i8);
        }
    }

    if (dwFlags & (DIEP_ENVELOPE as DWORD)) != 0 {
        TRACE!(b"  - lpEnvelope: %p\n\0".as_ptr() as *const i8, (*eff).lpEnvelope);
        if !(*eff).lpEnvelope.is_null() {
            _dump_DIENVELOPE((*eff).lpEnvelope as LPCDIENVELOPE);
        }
    }

    if (*eff).dwSize as usize > core::mem::size_of::<DIEFFECT_DX5>() {
        TRACE!(
            b"  - dwStartDelay: %d\n\0".as_ptr() as *const i8,
            (*eff).dwStartDelay,
        );
    }

    if type_0 == (DIEFT_CONSTANTFORCE as DWORD) {
        if (*eff).cbTypeSpecificParams as usize
            != core::mem::size_of::<DICONSTANTFORCE>()
        {
            WARN!(
                b"Effect claims to be a constant force but the type-specific params are the wrong size!\n\0"
                    .as_ptr() as *const i8,
            );
        } else {
            _dump_DICONSTANTFORCE((*eff).lpvTypeSpecificParams as LPCDICONSTANTFORCE);
        }
    } else if type_0 == (DIEFT_PERIODIC as DWORD) {
        if (*eff).cbTypeSpecificParams as usize
            != core::mem::size_of::<DIPERIODIC>()
        {
            WARN!(
                b"Effect claims to be a periodic force but the type-specific params are the wrong size!\n\0"
                    .as_ptr() as *const i8,
            );
        } else {
            _dump_DIPERIODIC((*eff).lpvTypeSpecificParams as LPCDIPERIODIC);
        }
    } else if type_0 == (DIEFT_RAMPFORCE as DWORD) {
        if (*eff).cbTypeSpecificParams as usize
            != core::mem::size_of::<DIRAMPFORCE>()
        {
            WARN!(
                b"Effect claims to be a ramp force but the type-specific params are the wrong size!\n\0"
                    .as_ptr() as *const i8,
            );
        } else {
            _dump_DIRAMPFORCE((*eff).lpvTypeSpecificParams as LPCDIRAMPFORCE);
        }
    } else if type_0 == (DIEFT_CONDITION as DWORD) {
        if (*eff).cbTypeSpecificParams as usize == core::mem::size_of::<DICONDITION>() {
            _dump_DICONDITION((*eff).lpvTypeSpecificParams as LPCDICONDITION);
        } else if (*eff).cbTypeSpecificParams as usize
            == (2 as usize)
                .wrapping_mul(core::mem::size_of::<DICONDITION>())
        {
            let mut condition: *mut DICONDITION =
                (*eff).lpvTypeSpecificParams as *mut DICONDITION;
            _dump_DICONDITION(
                condition.offset(0 as isize) as *mut DICONDITION
                    as LPCDICONDITION,
            );
            _dump_DICONDITION(
                condition.offset(1 as isize) as *mut DICONDITION
                    as LPCDICONDITION,
            );
        } else {
            WARN!(
                b"Effect claims to be a condition but the type-specific params are the wrong size!\n\0"
                    .as_ptr() as *const i8,
            );
        }
    } else if type_0 == (DIEFT_CUSTOMFORCE as DWORD) {
        if (*eff).cbTypeSpecificParams as usize
            != core::mem::size_of::<DICUSTOMFORCE>()
        {
            WARN!(
                b"Effect claims to be a custom force but the type-specific params are the wrong size!\n\0"
                    .as_ptr() as *const i8,
            );
        } else {
            _dump_DICUSTOMFORCE((*eff).lpvTypeSpecificParams as LPCDICUSTOMFORCE);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn device_disabled_registry(name: *const ::core::ffi::c_char) -> BOOL {
    static disabled_str: [u8; 9] = *b"disabled\0";
    static joystick_key: [u8; 10] = *b"Joysticks\0";
    let mut buffer: [::core::ffi::c_char; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut hkey: HKEY = ::core::ptr::null_mut();
    let mut appkey: HKEY = ::core::ptr::null_mut();
    let mut temp: HKEY = ::core::ptr::null_mut();
    let mut do_disable: BOOL = (FALSE as i32);

    get_app_key(core::ptr::addr_of_mut!(hkey), core::ptr::addr_of_mut!(appkey));

    /* Joystick settings are in the 'joysticks' subkey */
    if !appkey.is_null() {
        if RegOpenKeyA(
            appkey,
            core::ptr::addr_of!(joystick_key) as LPCSTR,
            core::ptr::addr_of_mut!(temp),
        ) != 0
        {
            temp = core::ptr::null_mut();
        }
        RegCloseKey(appkey);
        appkey = temp;
    }
    if !hkey.is_null() {
        if RegOpenKeyA(
            hkey,
            core::ptr::addr_of!(joystick_key) as LPCSTR,
            core::ptr::addr_of_mut!(temp),
        ) != 0
        {
            temp = core::ptr::null_mut();
        }
        RegCloseKey(hkey);
        hkey = temp;
    }

    /* Look for the "controllername"="disabled" key */
    if get_config_key(
        hkey,
        appkey,
        name,
        buffer.as_mut_ptr(),
        core::mem::size_of::<[::core::ffi::c_char; (MAX_PATH) as _]>() as DWORD,
    ) == 0
    {
        if strcmp(
            core::ptr::addr_of!(disabled_str) as *const ::core::ffi::c_char,
            buffer.as_mut_ptr(),
        ) == 0
        {
            TRACE!(
                "Disabling joystick '%s' based on registry key.\n",
                name
            );
            do_disable = (TRUE as i32);
        }
    }

    if !appkey.is_null() {
        RegCloseKey(appkey);
    }
    if !hkey.is_null() {
        RegCloseKey(hkey);
    }

    return do_disable;
}
#[no_mangle]
pub unsafe extern "C" fn is_xinput_device(devcaps: *const DIDEVCAPS, vid: WORD, pid: WORD) -> BOOL {
    let mut i: ::core::ffi::c_int;

    if vid as ::core::ffi::c_int == (VID_MICROSOFT as i32) {
        i = 0 as ::core::ffi::c_int;
        while i < ARRAY_SIZE!(PID_XBOX_CONTROLLERS) as ::core::ffi::c_int {
            if pid as ::core::ffi::c_int == PID_XBOX_CONTROLLERS[i as usize] as ::core::ffi::c_int {
                return (TRUE) as BOOL;
            }
            i += 1;
        }
    }

    return ((*devcaps).dwAxes == 6 && (*devcaps).dwButtons >= 14)
        as ::core::ffi::c_int;
}

unsafe extern "C" fn remap_init(
    mut This: *mut JoystickGenericImpl,
    mut obj: ::core::ffi::c_int,
    mut remap_props: *mut ObjProps,
) {
    (*remap_props).lMin = (*This).props[obj as usize].lMin;
    (*remap_props).lDevMin = (*remap_props).lMin;
    (*remap_props).lMax = (*This).props[obj as usize].lMax;
    (*remap_props).lDevMax = (*remap_props).lMax;
    (*remap_props).lDeadZone = (*This).props[obj as usize].lDeadZone;
    (*remap_props).lSaturation = (*This).props[obj as usize].lSaturation;
}

unsafe extern "C" fn remap_apply(
    mut This: *mut JoystickGenericImpl,
    mut obj: ::core::ffi::c_int,
    mut remap_props: *mut ObjProps,
) {
    let dwOfs: DWORD = (*(*(*This).base.data_format.wine_df)
        .rgodf
        .offset(obj as isize))
    .dwOfs;
    if dwOfs == (DIJOFS_X as DWORD) {
        (*This).js.lX = joystick_map_axis(remap_props, (*This).js.lX as ::core::ffi::c_int);
    } else if dwOfs == (DIJOFS_Y as DWORD) {
        (*This).js.lY = joystick_map_axis(remap_props, (*This).js.lY as ::core::ffi::c_int);
    } else if dwOfs == (DIJOFS_Z as DWORD) {
        (*This).js.lZ = joystick_map_axis(remap_props, (*This).js.lZ as ::core::ffi::c_int);
    } else if dwOfs == (DIJOFS_RX as DWORD) {
        (*This).js.lRx = joystick_map_axis(remap_props, (*This).js.lRx as ::core::ffi::c_int);
    } else if dwOfs == (DIJOFS_RY as DWORD) {
        (*This).js.lRy = joystick_map_axis(remap_props, (*This).js.lRy as ::core::ffi::c_int);
    } else if dwOfs == (DIJOFS_RZ as DWORD) {
        (*This).js.lRz = joystick_map_axis(remap_props, (*This).js.lRz as ::core::ffi::c_int);
    } else if dwOfs == (core::mem::offset_of!(DIJOYSTATE, rglSlider) + 0 * core::mem::size_of::<LONG>()) as DWORD {
        (*This).js.rglSlider[0 as usize] = joystick_map_axis(
            remap_props,
            (*This).js.rglSlider[0 as usize] as ::core::ffi::c_int,
        );
    } else if dwOfs == (core::mem::offset_of!(DIJOYSTATE, rglSlider) + 1 * core::mem::size_of::<LONG>()) as DWORD {
        (*This).js.rglSlider[1 as usize] = joystick_map_axis(
            remap_props,
            (*This).js.rglSlider[1 as usize] as ::core::ffi::c_int,
        );
    } else {
    }
}

unsafe extern "C" fn remap_range(
    mut This: *mut JoystickGenericImpl,
    mut obj: ::core::ffi::c_int,
    mut pr: LPCDIPROPRANGE,
) {
    let mut remap_props: ObjProps = ObjProps {
        lDevMin: 0,
        lDevMax: 0,
        lMin: 0,
        lMax: 0,
        lDeadZone: 0,
        lSaturation: 0,
    };
    remap_init(This, obj, &raw mut remap_props);
    remap_props.lMin = (*pr).lMin;
    remap_props.lMax = (*pr).lMax;
    remap_apply(This, obj, &raw mut remap_props);
    (*This).props[obj as usize].lMin = (*pr).lMin;
    (*This).props[obj as usize].lMax = (*pr).lMax;
}

unsafe extern "C" fn remap_deadzone(This: *mut JoystickGenericImpl, obj: ::core::ffi::c_int, pd: LPCDIPROPDWORD) {
    let mut remap_props: ObjProps = ObjProps {
        lDevMin: 0,
        lDevMax: 0,
        lMin: 0,
        lMax: 0,
        lDeadZone: 0,
        lSaturation: 0,
    };
    remap_init(This, obj, core::ptr::addr_of_mut!(remap_props));

    remap_props.lDeadZone = (*pd).dwData as LONG;

    remap_apply(This, obj, core::ptr::addr_of_mut!(remap_props));

    // Store new value
    (*This).props[obj as usize].lDeadZone = (*pd).dwData as LONG;
}

unsafe extern "C" fn remap_saturation(This: *mut JoystickGenericImpl, obj: i32, pd: LPCDIPROPDWORD) {
    let mut remap_props: ObjProps = core::mem::zeroed();
    remap_init(This, obj, core::ptr::addr_of_mut!(remap_props));

    remap_props.lSaturation = (*pd).dwData as LONG;

    remap_apply(This, obj, core::ptr::addr_of_mut!(remap_props));

    /* Store new value */
    (*This).props[obj as usize].lSaturation = (*pd).dwData as LONG;
}

#[no_mangle]
pub unsafe extern "system" fn JoystickWGenericImpl_SetProperty(iface: LPDIRECTINPUTDEVICE8W, rguid: REFGUID, ph: LPCDIPROPHEADER) -> HRESULT {
    let mut This: *mut JoystickGenericImpl = impl_from_IDirectInputDevice8W(iface as *mut IDirectInputDevice8W);
    let mut i: DWORD = 0;

    if ph.is_null() {
        WARN!("invalid parameter: ph == NULL\n");
        return DIERR_INVALIDPARAM;
    }

    if ((TRACE_ON!(__wine_dbch_dinput)) != 0) {
        _dump_DIPROPHEADER(ph);
    }

    if IS_DIPROP!(rguid) {
        let sw: DWORD_PTR = LOWORD!(rguid as DWORD_PTR) as DWORD_PTR;
        if sw == (DIPROP_RANGE as DWORD_PTR) {
            let pr: LPCDIPROPRANGE = ph as LPCDIPROPRANGE;
            if (*ph).dwHow == DIPH_DEVICE {
                i = 0;
                while i < (*(*This).base.data_format.wine_df).dwNumObjs {
                    remap_range(This, i as i32, pr);
                    i = i.wrapping_add(1);
                }
            } else {
                let obj: i32 = find_property(core::ptr::addr_of_mut!((*This).base.data_format), ph);
                if obj >= 0 {
                    remap_range(This, obj, pr);
                }
            }
        } else if sw == (DIPROP_DEADZONE as DWORD_PTR) {
            let pd: LPCDIPROPDWORD = ph as LPCDIPROPDWORD;
            if (*ph).dwHow == DIPH_DEVICE {
                i = 0;
                while i < (*(*This).base.data_format.wine_df).dwNumObjs {
                    remap_deadzone(This, i as i32, pd);
                    i = i.wrapping_add(1);
                }
            } else {
                let obj: i32 = find_property(core::ptr::addr_of_mut!((*This).base.data_format), ph);
                if obj >= 0 {
                    remap_deadzone(This, obj, pd);
                }
            }
        } else if sw == (DIPROP_SATURATION as DWORD_PTR) {
            let pd: LPCDIPROPDWORD = ph as LPCDIPROPDWORD;
            if (*ph).dwHow == DIPH_DEVICE {
                i = 0;
                while i < (*(*This).base.data_format.wine_df).dwNumObjs {
                    remap_saturation(This, i as i32, pd);
                    i = i.wrapping_add(1);
                }
            } else {
                let obj: i32 = find_property(core::ptr::addr_of_mut!((*This).base.data_format), ph);
                if obj >= 0 {
                    remap_saturation(This, obj, pd);
                }
            }
        } else if sw == (DIPROP_CALIBRATIONMODE as DWORD_PTR) {
            let _pd: LPCDIPROPDWORD = ph as LPCDIPROPDWORD;
        } else {
            return IDirectInputDevice2WImpl_SetProperty(iface, rguid, ph);
        }
    }

    return DI_OK;
}

#[no_mangle]
pub unsafe extern "system" fn JoystickAGenericImpl_SetProperty(
    iface: LPDIRECTINPUTDEVICE8A,
    rguid: *const GUID,
    ph: LPCDIPROPHEADER,
) -> HRESULT {
    let This: *mut JoystickGenericImpl = impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    return JoystickWGenericImpl_SetProperty(
        IDirectInputDevice8W_from_impl(This) as LPDIRECTINPUTDEVICE8W,
        rguid,
        ph,
    );
}
#[no_mangle]
pub unsafe extern "C" fn _dump_DIDEVCAPS(mut lpDIDevCaps: *const DIDEVCAPS) {
    let mut type_0: ::core::ffi::c_int = GET_DIDEVICE_TYPE!((*lpDIDevCaps).dwDevType) as ::core::ffi::c_int;
    let mut str: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut hid: *const ::core::ffi::c_char = b"\0" as *const u8 as *const ::core::ffi::c_char;
    WINE_TRACE!("dwSize: %d\n", (*lpDIDevCaps).dwSize);
    WINE_TRACE!("dwFlags: %08x\n", (*lpDIDevCaps).dwFlags);
    match (type_0) as u32 {
        DIDEVTYPE_DEVICE => {
            str = b"DIDEVTYPE_DEVICE\0" as *const u8 as *const ::core::ffi::c_char;
        }
        DIDEVTYPE_MOUSE => {
            str = b"DIDEVTYPE_MOUSE\0" as *const u8 as *const ::core::ffi::c_char;
        }
        DIDEVTYPE_KEYBOARD => {
            str = b"DIDEVTYPE_KEYBOARD\0" as *const u8 as *const ::core::ffi::c_char;
        }
        DIDEVTYPE_JOYSTICK => {
            str = b"DIDEVTYPE_JOYSTICK\0" as *const u8 as *const ::core::ffi::c_char;
        }
        DI8DEVTYPE_DEVICE => {
            str = b"DI8DEVTYPE_DEVICE\0" as *const u8 as *const ::core::ffi::c_char;
        }
        DI8DEVTYPE_MOUSE => {
            str = b"DI8DEVTYPE_MOUSE\0" as *const u8 as *const ::core::ffi::c_char;
        }
        DI8DEVTYPE_KEYBOARD => {
            str = b"DI8DEVTYPE_KEYBOARD\0" as *const u8 as *const ::core::ffi::c_char;
        }
        DI8DEVTYPE_JOYSTICK => {
            str = b"DI8DEVTYPE_JOYSTICK\0" as *const u8 as *const ::core::ffi::c_char;
        }
        DI8DEVTYPE_GAMEPAD => {
            str = b"DI8DEVTYPE_GAMEPAD\0" as *const u8 as *const ::core::ffi::c_char;
        }
        DI8DEVTYPE_DRIVING => {
            str = b"DI8DEVTYPE_DRIVING\0" as *const u8 as *const ::core::ffi::c_char;
        }
        DI8DEVTYPE_FLIGHT => {
            str = b"DI8DEVTYPE_FLIGHT\0" as *const u8 as *const ::core::ffi::c_char;
        }
        DI8DEVTYPE_1STPERSON => {
            str = b"DI8DEVTYPE_1STPERSON\0" as *const u8 as *const ::core::ffi::c_char;
        }
        DI8DEVTYPE_DEVICECTRL => {
            str = b"DI8DEVTYPE_DEVICECTRL\0" as *const u8 as *const ::core::ffi::c_char;
        }
        DI8DEVTYPE_SCREENPOINTER => {
            str = b"DI8DEVTYPE_SCREENPOINTER\0" as *const u8 as *const ::core::ffi::c_char;
        }
        DI8DEVTYPE_REMOTE => {
            str = b"DI8DEVTYPE_REMOTE\0" as *const u8 as *const ::core::ffi::c_char;
        }
        DI8DEVTYPE_SUPPLEMENTAL => {
            str = b"DI8DEVTYPE_SUPPLEMENTAL\0" as *const u8 as *const ::core::ffi::c_char;
        }
        _ => {
            str = b"UNKNOWN\0" as *const u8 as *const ::core::ffi::c_char;
        }
    }
    if ((*lpDIDevCaps).dwDevType & (DIDEVTYPE_HID as DWORD)) != 0 {
        hid = b" (HID)\0" as *const u8 as *const ::core::ffi::c_char;
    }
    WINE_TRACE!("dwDevType: %08x %s%s\n", (*lpDIDevCaps).dwDevType, str, hid);
    WINE_TRACE!("dwAxes: %d\n", (*lpDIDevCaps).dwAxes);
    WINE_TRACE!("dwButtons: %d\n", (*lpDIDevCaps).dwButtons);
    WINE_TRACE!("dwPOVs: %d\n", (*lpDIDevCaps).dwPOVs);
    if (*lpDIDevCaps).dwSize as usize > ::core::mem::size_of::<DIDEVCAPS_DX3>() as usize {
        WINE_TRACE!("dwFFSamplePeriod: %d\n", (*lpDIDevCaps).dwFFSamplePeriod);
        WINE_TRACE!("dwFFMinTimeResolution: %d\n", (*lpDIDevCaps).dwFFMinTimeResolution);
        WINE_TRACE!("dwFirmwareRevision: %d\n", (*lpDIDevCaps).dwFirmwareRevision);
        WINE_TRACE!("dwHardwareRevision: %d\n", (*lpDIDevCaps).dwHardwareRevision);
        WINE_TRACE!("dwFFDriverVersion: %d\n", (*lpDIDevCaps).dwFFDriverVersion);
    }
}
#[no_mangle]
pub unsafe extern "system" fn JoystickWGenericImpl_GetCapabilities(iface: LPDIRECTINPUTDEVICE8W, lpDIDevCaps: LPDIDEVCAPS) -> HRESULT {
    let This: *mut JoystickGenericImpl = impl_from_IDirectInputDevice8W(iface);
    let mut size: i32 = 0;

    if lpDIDevCaps.is_null() {
        WARN!("invalid pointer\n");
        return E_POINTER;
    }

    size = (*lpDIDevCaps).dwSize as i32;

    if !(size == core::mem::size_of::<DIDEVCAPS>() as i32 || size == core::mem::size_of::<DIDEVCAPS_DX3>() as i32) {
        WARN!("invalid parameter\n");
        return DIERR_INVALIDPARAM;
    }

    CopyMemory!(lpDIDevCaps as LPVOID, core::ptr::addr_of_mut!((*This).devcaps) as LPVOID, size as usize);
    (*lpDIDevCaps).dwSize = size as DWORD;

    if ((TRACE_ON!(__wine_dbch_dinput)) != 0) {
        _dump_DIDEVCAPS(lpDIDevCaps);
    }

    return DI_OK;
}

#[no_mangle]
pub unsafe extern "system" fn JoystickAGenericImpl_GetCapabilities(iface: LPDIRECTINPUTDEVICE8A, lpDIDevCaps: LPDIDEVCAPS) -> HRESULT {
    let This: *mut JoystickGenericImpl = impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    return JoystickWGenericImpl_GetCapabilities(IDirectInputDevice8W_from_impl(This) as LPDIRECTINPUTDEVICE8W, lpDIDevCaps);
}
#[no_mangle]
pub unsafe extern "system" fn JoystickWGenericImpl_GetObjectInfo(
    iface: LPDIRECTINPUTDEVICE8W,
    pdidoi: LPDIDEVICEOBJECTINSTANCEW,
    dwObj: DWORD,
    dwHow: DWORD,
) -> HRESULT {
    static axisW: [WCHAR; 8] = [0x0041u16, 0x0078, 0x0069, 0x0073, 0x0020, 0x0025, 0x0064, 0x0000];
    static povW: [WCHAR; 7] = [0x0050u16, 0x004F, 0x0056, 0x0020, 0x0025, 0x0064, 0x0000];
    static buttonW: [WCHAR; 10] = [0x0042u16, 0x0075, 0x0074, 0x0074, 0x006F, 0x006E, 0x0020, 0x0025, 0x0064, 0x0000];
    let mut res: HRESULT;

    res = IDirectInputDevice2WImpl_GetObjectInfo(iface, pdidoi, dwObj, dwHow);
    if res != DI_OK {
        return res;
    }

    if ((*pdidoi).dwType & DIDFT_AXIS) != 0 {
        sprintfW(
            core::ptr::addr_of_mut!((*pdidoi).tszName) as *mut WCHAR,
            core::ptr::addr_of!(axisW) as *const WCHAR,
            (((((*pdidoi).dwType >> 8) as DWORD_PTR) & (0xffff as DWORD_PTR)) as WORD)
                as ::core::ffi::c_int,
        );
        (*pdidoi).dwFlags |= DIDOI_ASPECTPOSITION;
    } else if ((*pdidoi).dwType & DIDFT_POV) != 0 {
        sprintfW(
            core::ptr::addr_of_mut!((*pdidoi).tszName) as *mut WCHAR,
            core::ptr::addr_of!(povW) as *const WCHAR,
            (((((*pdidoi).dwType >> 8) as DWORD_PTR) & (0xffff as DWORD_PTR)) as WORD)
                as ::core::ffi::c_int,
        );
    } else if ((*pdidoi).dwType & DIDFT_BUTTON) != 0 {
        sprintfW(
            core::ptr::addr_of_mut!((*pdidoi).tszName) as *mut WCHAR,
            core::ptr::addr_of!(buttonW) as *const WCHAR,
            (((((*pdidoi).dwType >> 8) as DWORD_PTR) & (0xffff as DWORD_PTR)) as WORD)
                as ::core::ffi::c_int,
        );
    }

    _dump_OBJECTINSTANCEW(pdidoi as *const DIDEVICEOBJECTINSTANCEW);
    return res;
}
#[no_mangle]
pub unsafe extern "system" fn JoystickAGenericImpl_GetObjectInfo(
    iface: LPDIRECTINPUTDEVICE8A,
    pdidoi: LPDIDEVICEOBJECTINSTANCEA,
    dwObj: DWORD,
    dwHow: DWORD,
) -> HRESULT {
    let This: *mut JoystickGenericImpl = impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    let mut res: HRESULT = 0;
    let mut didoiW: DIDEVICEOBJECTINSTANCEW = core::mem::zeroed();
    let dwSize: DWORD = (*pdidoi).dwSize;

    didoiW.dwSize = core::mem::size_of::<DIDEVICEOBJECTINSTANCEW>() as DWORD;
    res = JoystickWGenericImpl_GetObjectInfo(
        IDirectInputDevice8W_from_impl(This) as LPDIRECTINPUTDEVICE8W,
        core::ptr::addr_of_mut!(didoiW),
        dwObj,
        dwHow,
    );
    if res != DI_OK {
        return res;
    }

    memset(
        pdidoi as *mut core::ffi::c_void,
        0 as core::ffi::c_int,
        (*pdidoi).dwSize as usize,
    );
    memcpy(
        pdidoi as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(didoiW) as *const core::ffi::c_void,
        FIELD_OFFSET!(DIDEVICEOBJECTINSTANCEW, tszName) as usize,
    );
    (*pdidoi).dwSize = dwSize;
    WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        core::ptr::addr_of_mut!(didoiW.tszName) as *mut WCHAR as LPCWSTR,
        -(1) as core::ffi::c_int,
        core::ptr::addr_of_mut!((*pdidoi).tszName) as LPSTR,
        core::mem::size_of::<[CHAR; 260]>() as core::ffi::c_int,
        core::ptr::null::<CHAR>(),
        core::ptr::null_mut::<BOOL>(),
    );

    return res;
}
#[no_mangle]
pub unsafe extern "system" fn JoystickWGenericImpl_GetProperty(
    iface: LPDIRECTINPUTDEVICE8W,
    rguid: REFGUID,
    pdiph: LPDIPROPHEADER,
) -> HRESULT {
    let mut This: *mut JoystickGenericImpl = impl_from_IDirectInputDevice8W(iface);

    if ((TRACE_ON!(__wine_dbch_dinput)) != 0) {
        _dump_DIPROPHEADER(pdiph);
    }

    if IS_DIPROP!(rguid) {
        match LOWORD!(rguid) as DWORD_PTR {
            __c if __c == DIPROP_RANGE as _ => {
                let mut pr: LPDIPROPRANGE = pdiph as LPDIPROPRANGE;
                let mut obj: core::ffi::c_int =
                    find_property(&raw mut (*This).base.data_format, pdiph);

                /* The app is querying the current range of the axis
                 * return the lMin and lMax values */
                if obj >= 0 {
                    (*pr).lMin = (*This).props[obj as usize].lMin;
                    (*pr).lMax = (*This).props[obj as usize].lMax;

                    return DI_OK;
                }
            }
            __c if __c == DIPROP_DEADZONE as _ => {
                let mut pd: LPDIPROPDWORD = pdiph as LPDIPROPDWORD;
                let mut obj: core::ffi::c_int =
                    find_property(&raw mut (*This).base.data_format, pdiph);

                if obj >= 0 {
                    (*pd).dwData = (*This).props[obj as usize].lDeadZone as DWORD;

                    return DI_OK;
                }
            }
            __c if __c == DIPROP_SATURATION as _ => {
                let mut pd: LPDIPROPDWORD = pdiph as LPDIPROPDWORD;
                let mut obj: core::ffi::c_int =
                    find_property(&raw mut (*This).base.data_format, pdiph);

                if obj >= 0 {
                    (*pd).dwData = (*This).props[obj as usize].lSaturation as DWORD;

                    return DI_OK;
                }
            }
            __c if __c == DIPROP_PRODUCTNAME as _ || __c == DIPROP_INSTANCENAME as _ => {
                let mut ps: *mut DIPROPSTRING = pdiph as *mut DIPROPSTRING;
                let mut didev: DIDEVICEINSTANCEW = core::mem::zeroed::<DIDEVICEINSTANCEW>();

                didev.dwSize = core::mem::size_of::<DIDEVICEINSTANCEW>() as DWORD;

                IDirectInputDevice_GetDeviceInfo!(iface, &raw mut didev);
                if (LOWORD!(rguid) as DWORD_PTR) == (DIPROP_PRODUCTNAME as _) {
                    lstrcpynW(
                        &raw mut (*ps).wsz as LPWSTR,
                        &raw mut didev.tszProductName as *mut WCHAR as LPCWSTR,
                        (MAX_PATH as i32),
                    );
                } else {
                    lstrcpynW(
                        &raw mut (*ps).wsz as LPWSTR,
                        &raw mut didev.tszInstanceName as *mut WCHAR as LPCWSTR,
                        (MAX_PATH as i32),
                    );
                }

                return DI_OK;
            }
            _ => {
                return IDirectInputDevice2WImpl_GetProperty(iface, rguid, pdiph);
            }
        }
    }

    return DI_OK;
}

#[no_mangle]
pub unsafe extern "system" fn JoystickAGenericImpl_GetProperty(iface: LPDIRECTINPUTDEVICE8A, rguid: REFGUID, pdiph: LPDIPROPHEADER) -> HRESULT {
    let This: *mut JoystickGenericImpl = impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    return JoystickWGenericImpl_GetProperty(IDirectInputDevice8W_from_impl(This) as LPDIRECTINPUTDEVICE8W, rguid, pdiph);
}

#[no_mangle]
pub unsafe extern "system" fn JoystickAGenericImpl_GetDeviceInfo(
    iface: LPDIRECTINPUTDEVICE8A,
    pdidi: LPDIDEVICEINSTANCEA,
) -> HRESULT {
    let mut This: *mut JoystickGenericImpl =
        impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    let mut pd: DIPROPDWORD = DIPROPDWORD {
        diph: DIPROPHEADER {
            dwSize: 0,
            dwHeaderSize: 0,
            dwObj: 0,
            dwHow: 0,
        },
        dwData: 0,
    };
    let mut index: DWORD = 0;

    TRACE!("(%p,%p)\n", This, pdidi);

    if pdidi.is_null() {
        WARN!("invalid pointer\n");
        return E_POINTER;
    }

    if ((*pdidi).dwSize != core::mem::size_of::<DIDEVICEINSTANCE_DX3A>() as DWORD)
        && ((*pdidi).dwSize != core::mem::size_of::<DIDEVICEINSTANCEA>() as DWORD)
    {
        WARN!("invalid parameter: pdidi->dwSize = %d\n", (*pdidi).dwSize);
        return DIERR_INVALIDPARAM;
    }

    /* Try to get joystick index */
    pd.diph.dwSize = core::mem::size_of::<DIPROPDWORD>() as DWORD;
    pd.diph.dwHeaderSize = core::mem::size_of::<DIPROPHEADER>() as DWORD;
    pd.diph.dwObj = 0;
    pd.diph.dwHow = DIPH_DEVICE;
    if (IDirectInputDevice2_GetProperty!(
        iface,
        DIPROP_JOYSTICKID,
        core::ptr::addr_of_mut!(pd.diph),
    )) >= 0
    {
        index = pd.dwData;
    }

    /* Return joystick */
    (*pdidi).guidInstance = (*This).guidInstance;
    (*pdidi).guidProduct = (*This).guidProduct;
    /* we only support traditional joysticks for now */
    (*pdidi).dwDevType = (*This).devcaps.dwDevType;
    snprintf(
        core::ptr::addr_of_mut!((*pdidi).tszInstanceName) as *mut i8,
        MAX_PATH as usize,
        b"Joystick %d\0".as_ptr() as *const i8,
        index,
    );
    lstrcpynA(
        core::ptr::addr_of_mut!((*pdidi).tszProductName) as LPSTR,
        (*This).name as LPCSTR,
        (MAX_PATH as i32),
    );
    if (*pdidi).dwSize as usize > core::mem::size_of::<DIDEVICEINSTANCE_DX3A>() as usize {
        (*pdidi).guidFFDriver = GUID_NULL;
        (*pdidi).wUsagePage = 0;
        (*pdidi).wUsage = 0;
    }

    return DI_OK;
}
#[no_mangle]
pub unsafe extern "system" fn JoystickWGenericImpl_GetDeviceInfo(
    iface: LPDIRECTINPUTDEVICE8W,
    pdidi: LPDIDEVICEINSTANCEW,
) -> HRESULT {
    let mut This: *mut JoystickGenericImpl = impl_from_IDirectInputDevice8W(iface);
    let mut buffer: [CHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut pd: DIPROPDWORD = core::mem::zeroed();
    let mut index: DWORD = 0 as DWORD;

    if ((*pdidi).dwSize as usize != core::mem::size_of::<DIDEVICEINSTANCE_DX3W>() as usize)
        && ((*pdidi).dwSize as usize != core::mem::size_of::<DIDEVICEINSTANCEW>() as usize)
    {
        WARN!("invalid parameter: pdidi->dwSize = %d\n", (*pdidi).dwSize);
        return DIERR_INVALIDPARAM as HRESULT;
    }

    /* Try to get joystick index */
    pd.diph.dwSize = core::mem::size_of::<DIPROPDWORD>() as DWORD;
    pd.diph.dwHeaderSize = core::mem::size_of::<DIPROPHEADER>() as DWORD;
    pd.diph.dwObj = 0 as DWORD;
    pd.diph.dwHow = DIPH_DEVICE as DWORD;
    if (IDirectInputDevice2_GetProperty!(
        iface,
        DIPROP_JOYSTICKID as *const GUID,
        core::ptr::addr_of_mut!(pd.diph),
    ) as HRESULT) >= (0 as HRESULT)
    {
        index = pd.dwData;
    }

    /* Return joystick */
    (*pdidi).guidInstance = (*This).guidInstance;
    (*pdidi).guidProduct = (*This).guidProduct;
    /* we only support traditional joysticks for now */
    (*pdidi).dwDevType = (*This).devcaps.dwDevType;
    snprintf(
        core::ptr::addr_of_mut!(buffer) as *mut CHAR,
        core::mem::size_of::<[CHAR; (MAX_PATH) as _]>() as usize,
        b"Joystick %d\0" as *const u8 as *const CHAR,
        index,
    );
    MultiByteToWideChar(
        CP_ACP as UINT,
        0 as DWORD,
        core::ptr::addr_of_mut!(buffer) as *mut CHAR as LPCSTR,
        -(1 as i32),
        core::ptr::addr_of_mut!((*pdidi).tszInstanceName) as LPWSTR,
        MAX_PATH as i32,
    );
    MultiByteToWideChar(
        CP_ACP as UINT,
        0 as DWORD,
        (*This).name as LPCSTR,
        -(1 as i32),
        core::ptr::addr_of_mut!((*pdidi).tszProductName) as LPWSTR,
        MAX_PATH as i32,
    );
    if (*pdidi).dwSize as usize > core::mem::size_of::<DIDEVICEINSTANCE_DX3W>() as usize {
        (*pdidi).guidFFDriver = GUID_NULL;
        (*pdidi).wUsagePage = 0 as WORD;
        (*pdidi).wUsage = 0 as WORD;
    }

    return DI_OK as HRESULT;
}

#[no_mangle]
pub unsafe extern "system" fn JoystickWGenericImpl_Poll(iface: LPDIRECTINPUTDEVICE8W) -> HRESULT {
    let This: *mut JoystickGenericImpl = impl_from_IDirectInputDevice8W(iface as *mut IDirectInputDevice8W);

    if (*This).base.acquired == 0 {
        WARN!("not acquired\n");
        return DIERR_NOTACQUIRED;
    }

    (*This).joy_polldev.unwrap_unchecked()(
        IDirectInputDevice8A_from_impl(This) as LPDIRECTINPUTDEVICE8A
    );
    return DI_OK;
}
#[no_mangle]
pub unsafe extern "system" fn JoystickAGenericImpl_Poll(iface: LPDIRECTINPUTDEVICE8A) -> HRESULT {
    let mut This: *mut JoystickGenericImpl = impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    return JoystickWGenericImpl_Poll(IDirectInputDevice8W_from_impl(This) as LPDIRECTINPUTDEVICE8W);
}
#[no_mangle]
pub unsafe extern "system" fn JoystickWGenericImpl_GetDeviceState(iface: LPDIRECTINPUTDEVICE8W, len: DWORD, ptr: LPVOID) -> HRESULT {
    let mut This: *mut JoystickGenericImpl = impl_from_IDirectInputDevice8W(iface as *mut IDirectInputDevice8W);

    TRACE!("(%p,0x%08x,%p)\n", iface, len, ptr);

    if (*This).base.acquired == 0 {
        WARN!("not acquired\n");
        return DIERR_NOTACQUIRED;
    }

    /* update joystick state */
    ((*This).joy_polldev.unwrap_unchecked())(
        IDirectInputDevice8A_from_impl(This) as LPDIRECTINPUTDEVICE8A
    );

    /* convert and copy data to user supplied buffer */
    fill_DataFormat(
        ptr as *mut core::ffi::c_void,
        len,
        core::ptr::addr_of!((*This).js) as *const core::ffi::c_void,
        core::ptr::addr_of_mut!((*This).base.data_format),
    );

    return DI_OK;
}

#[no_mangle]
pub unsafe extern "system" fn JoystickAGenericImpl_GetDeviceState(iface: LPDIRECTINPUTDEVICE8A, len: DWORD, ptr: LPVOID) -> HRESULT {
    let This: *mut JoystickGenericImpl = impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    return JoystickWGenericImpl_GetDeviceState(IDirectInputDevice8W_from_impl(This) as LPDIRECTINPUTDEVICE8W, len, ptr);
}

#[no_mangle]
pub unsafe extern "system" fn JoystickWGenericImpl_BuildActionMap(
    iface: LPDIRECTINPUTDEVICE8W,
    lpdiaf: LPDIACTIONFORMATW,
    lpszUserName: LPCWSTR,
    dwFlags: DWORD,
) -> HRESULT {
    static object_types: [DWORD; 2] = [DIDFT_AXIS as DWORD, DIDFT_BUTTON as DWORD];
    static type_map: [DWORD; 2] = [DIDFT_RELAXIS as DWORD, DIDFT_PSHBUTTON as DWORD];
    let mut This: *mut JoystickGenericImpl = impl_from_IDirectInputDevice8W(iface as *mut IDirectInputDevice8W);
    let mut i: ::core::ffi::c_uint = 0;
    let mut j: ::core::ffi::c_uint = 0;
    let mut has_actions: BOOL = FALSE as BOOL;
    let mut username: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut size: DWORD = 0;
    let mut load_success: BOOL = FALSE as BOOL;

    FIXME!(
        "(%p)->(%p,%s,%08x): semi-stub !\n",
        This,
        lpdiaf,
        debugstr_w(lpszUserName),
        dwFlags
    );

    /* Unless asked the contrary by these flags, try to load a previous mapping */
    if (dwFlags & (DIDBAM_HWDEFAULTS as DWORD)) == 0 {
        if lpszUserName.is_null() {
            GetUserNameW(::core::ptr::null_mut::<WCHAR>(), core::ptr::addr_of_mut!(size));
        } else {
            size = (lstrlenW(lpszUserName) + (1 as ::core::ffi::c_int)) as DWORD;
        }

        username = heap_alloc(
            ((size as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T)) as _,
        ) as *mut WCHAR;
        if lpszUserName.is_null() {
            GetUserNameW(username as LPWSTR, core::ptr::addr_of_mut!(size));
        } else {
            lstrcpynW(username as LPWSTR, lpszUserName, size as ::core::ffi::c_int);
        }

        load_success = load_mapping_settings(core::ptr::addr_of_mut!((*This).base), lpdiaf, username);
        heap_free(username as *mut ::core::ffi::c_void);
    }

    if load_success != 0 {
        return DI_OK as HRESULT;
    }

    i = 0 as ::core::ffi::c_uint;
    while (i as DWORD) < (*lpdiaf).dwNumActions {
        let inst: DWORD = (0x000000ff as DWORD & (*(*lpdiaf).rgoAction.offset(i as isize)).dwSemantic)
            .wrapping_sub(1 as DWORD);
        let type_0: DWORD = 0x000000ff as DWORD
            & ((*(*lpdiaf).rgoAction.offset(i as isize)).dwSemantic >> (8 as DWORD));
        let genre: DWORD = 0xff000000 as DWORD & (*(*lpdiaf).rgoAction.offset(i as isize)).dwSemantic;

        /* Don't touch a user configured action */
        if !((*(*lpdiaf).rgoAction.offset(i as isize)).dwHow == (DIAH_USERCONFIG as DWORD)) {
            /* Only consider actions of the right genre */
            if !((*lpdiaf).dwGenre != genre && genre != (DIGENRE_ANY as DWORD)) {
                j = 0 as ::core::ffi::c_uint;
                while (j as usize) < ARRAY_SIZE!(object_types) as usize {
                    if (type_0 & object_types[j as usize]) != 0 {
                        /* Ensure that the object exists */
                        let odf: LPDIOBJECTDATAFORMAT = dataformat_to_odf_by_type(
                            (*This).base.data_format.wine_df as LPCDIDATAFORMAT,
                            inst as ::core::ffi::c_int,
                            object_types[j as usize],
                        );

                        if !odf.is_null() {
                            (*(*lpdiaf).rgoAction.offset(i as isize)).dwObjID = type_map[j as usize]
                                | (0x0000ff00 as DWORD & (inst << (8 as u32)));
                            (*(*lpdiaf).rgoAction.offset(i as isize)).guidInstance = (*This).base.guid;
                            (*(*lpdiaf).rgoAction.offset(i as isize)).dwHow = DIAH_DEFAULT as DWORD;

                            has_actions = TRUE as BOOL;

                            /* No need to try other types if the action was already mapped */
                            break;
                        }
                    }
                    j = j.wrapping_add(1);
                }
            }
        }
        i = i.wrapping_add(1);
    }

    if has_actions == 0 {
        return DI_NOEFFECT as HRESULT;
    }

    return IDirectInputDevice8WImpl_BuildActionMap(iface, lpdiaf, lpszUserName, dwFlags);
}
#[no_mangle]
pub unsafe extern "system" fn JoystickAGenericImpl_BuildActionMap(
    iface: LPDIRECTINPUTDEVICE8A,
    lpdiaf: LPDIACTIONFORMATA,
    lpszUserName: LPCSTR,
    dwFlags: DWORD,
) -> HRESULT {
    let mut This: *mut JoystickGenericImpl =
        impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    let mut diafW: DIACTIONFORMATW = core::mem::zeroed();
    let mut hr: HRESULT;
    let mut lpszUserNameW: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    let mut username_size: ::core::ffi::c_int;

    diafW.rgoAction = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        (::core::mem::size_of::<DIACTIONW>() as SIZE_T)
            .wrapping_mul((*lpdiaf).dwNumActions as SIZE_T),
    ) as LPDIACTIONW;
    _copy_diactionformatAtoW(&raw mut diafW, lpdiaf);

    if !lpszUserName.is_null() {
        username_size = MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            lpszUserName,
            -1,
            core::ptr::null_mut::<WCHAR>(),
            0 as ::core::ffi::c_int,
        );
        lpszUserNameW = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            (::core::mem::size_of::<WCHAR>() as SIZE_T).wrapping_mul(username_size as SIZE_T),
        ) as *mut WCHAR;
        MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            lpszUserName,
            -1,
            lpszUserNameW as LPWSTR,
            username_size,
        );
    }

    hr = JoystickWGenericImpl_BuildActionMap(
        &raw mut (*This).base.IDirectInputDevice8W_iface,
        &raw mut diafW,
        lpszUserNameW as LPCWSTR,
        dwFlags,
    );

    _copy_diactionformatWtoA(lpdiaf, &raw mut diafW);
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        diafW.rgoAction as PVOID,
    );
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        lpszUserNameW as PVOID,
    );

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn JoystickWGenericImpl_SetActionMap(
    iface: LPDIRECTINPUTDEVICE8W,
    lpdiaf: LPDIACTIONFORMATW,
    lpszUserName: LPCWSTR,
    dwFlags: DWORD,
) -> HRESULT {
    let mut This: *mut JoystickGenericImpl = impl_from_IDirectInputDevice8W(iface);

    return _set_action_map(
        iface,
        lpdiaf,
        lpszUserName,
        dwFlags,
        (*This).base.data_format.wine_df,
    );
}
#[no_mangle]
pub unsafe extern "system" fn JoystickAGenericImpl_SetActionMap(
    iface: LPDIRECTINPUTDEVICE8A,
    lpdiaf: LPDIACTIONFORMATA,
    lpszUserName: LPCSTR,
    dwFlags: DWORD,
) -> HRESULT {
    let mut This: *mut JoystickGenericImpl = impl_from_IDirectInputDevice8A(iface as *mut IDirectInputDevice8A);
    let mut diafW: DIACTIONFORMATW = _DIACTIONFORMATW {
        dwSize: 0,
        dwActionSize: 0,
        dwDataSize: 0,
        dwNumActions: 0,
        rgoAction: ::core::ptr::null_mut::<_DIACTIONW>(),
        guidActionMap: _GUID {
            Data1: 0,
            Data2: 0,
            Data3: 0,
            Data4: [0; 8],
        },
        dwGenre: 0,
        dwBufferSize: 0,
        lAxisMin: 0,
        lAxisMax: 0,
        hInstString: ::core::ptr::null_mut::<HINSTANCE__>(),
        ftTimeStamp: _FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        },
        dwCRC: 0,
        tszActionMap: [0; 260],
    };
    let mut hr: HRESULT = 0;
    let mut lpszUserNameW: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut username_size: ::core::ffi::c_int = 0;

    diafW.rgoAction = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        (::core::mem::size_of::<DIACTIONW>() as SIZE_T).wrapping_mul((*lpdiaf).dwNumActions as SIZE_T),
    ) as LPDIACTIONW;
    _copy_diactionformatAtoW(core::ptr::addr_of_mut!(diafW), lpdiaf);

    if !lpszUserName.is_null() {
        username_size = MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            lpszUserName,
            -(1 as ::core::ffi::c_int),
            ::core::ptr::null_mut::<WCHAR>(),
            0 as ::core::ffi::c_int,
        );
        lpszUserNameW = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            (::core::mem::size_of::<WCHAR>() as SIZE_T).wrapping_mul(username_size as SIZE_T),
        ) as *mut WCHAR;
        MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            lpszUserName,
            -(1 as ::core::ffi::c_int),
            lpszUserNameW as LPWSTR,
            username_size,
        );
    }

    hr = JoystickWGenericImpl_SetActionMap(
        core::ptr::addr_of_mut!((*This).base.IDirectInputDevice8W_iface),
        core::ptr::addr_of_mut!(diafW),
        lpszUserNameW as LPCWSTR,
        dwFlags,
    );

    (*lpdiaf).dwCRC = diafW.dwCRC;

    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        diafW.rgoAction as PVOID,
    );
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        lpszUserNameW as PVOID,
    );

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn joystick_map_axis(props: *mut ObjProps, mut val: ::core::ffi::c_int) -> LONG {
    let mut ret: LONG = 0;
    let mut dead_zone: LONG = MulDiv(
        (*props).lDeadZone as ::core::ffi::c_int,
        ((*props).lDevMax - (*props).lDevMin) as ::core::ffi::c_int,
        10000,
    ) as LONG;
    let mut dev_range: LONG = (*props).lDevMax - (*props).lDevMin - dead_zone;
    val = (val as LONG - ((*props).lDevMin + (*props).lDevMax) / 2) as ::core::ffi::c_int;
    if (abs(val) as LONG) <= dead_zone / 2 {
        val = 0;
    } else {
        val = (if (val < 0) {
            val as LONG + dead_zone / 2
        } else {
            val as LONG - dead_zone / 2
        }) as ::core::ffi::c_int;
    }
    ret = MulDiv(
        val,
        ((*props).lMax - (*props).lMin) as ::core::ffi::c_int,
        dev_range as ::core::ffi::c_int,
    ) as LONG
        + ((*props).lMin + (*props).lMax) / 2;
    if ret > (*props).lMax {
        ret = (*props).lMax;
    } else if ret < (*props).lMin {
        ret = (*props).lMin;
    }
    WINE_TRACE!("(%d <%d> %d) -> (%d <%d> %d): val=%d ret=%d\n", (*props).lDevMin, dead_zone, (*props).lDevMax, (*props).lMin, (*props).lDeadZone, (*props).lMax, val, ret);
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn joystick_map_pov(p: *const POINTL) -> DWORD {
    if (*p).x > 0 {
        return (if (*p).y < 0 {
            4500
        } else if (*p).y == 0 {
            9000
        } else {
            13500
        }) as DWORD;
    } else if (*p).x < 0 {
        return (if (*p).y < 0 {
            31500
        } else if (*p).y == 0 {
            27000
        } else {
            22500
        }) as DWORD;
    } else {
        return (if (*p).y < 0 {
            0
        } else if (*p).y == 0 {
            -1
        } else {
            18000
        }) as DWORD;
    }
}
#[no_mangle]
pub unsafe extern "C" fn setup_dinput_options(
    mut This: *mut JoystickGenericImpl,
    mut default_axis_map: *const core::ffi::c_int,
) -> HRESULT {
    let mut buffer: [core::ffi::c_char; (MAX_PATH + 16) as _] =
        [0; (MAX_PATH + 16) as _];
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut appkey: HKEY = core::ptr::null_mut();
    let mut tokens: core::ffi::c_int = 0;
    let mut axis: core::ffi::c_int = 0;
    let mut pov: core::ffi::c_int = 0;
    let mut button: core::ffi::c_int;

    get_app_key(core::ptr::addr_of_mut!(hkey), core::ptr::addr_of_mut!(appkey));

    /* get options */

    if get_config_key(
        hkey,
        appkey,
        b"DefaultDeadZone\0".as_ptr() as *const core::ffi::c_char,
        core::ptr::addr_of_mut!(buffer) as *mut core::ffi::c_char,
        core::mem::size_of::<[core::ffi::c_char; (MAX_PATH + 16) as _]>()
            as DWORD,
    ) == 0
    {
        (*This).deadzone =
            atoi(core::ptr::addr_of_mut!(buffer) as *mut core::ffi::c_char)
                as LONG;
        TRACE!(
            "setting default deadzone to: \"%s\" %d\n",
            core::ptr::addr_of_mut!(buffer) as *mut core::ffi::c_char,
            (*This).deadzone,
        );
    }

    button = 0;
    while button < (MAX_MAP_BUTTONS as core::ffi::c_int) {
        (*This).button_map[button as usize] = button;
        button += 1;
    }

    if get_config_key(
        hkey,
        appkey,
        b"ButtonMap\0".as_ptr() as *const core::ffi::c_char,
        core::ptr::addr_of_mut!(buffer) as *mut core::ffi::c_char,
        core::mem::size_of::<[core::ffi::c_char; (MAX_PATH + 16) as _]>()
            as DWORD,
    ) == 0
    {
        static mut delim: *const core::ffi::c_char = b",\0" as *const u8 as *const core::ffi::c_char;
        let mut button: core::ffi::c_int = 0;
        let mut token: *mut core::ffi::c_char = core::ptr::null_mut();

        TRACE!(
            "ButtonMap = \"%s\"\n",
            core::ptr::addr_of_mut!(buffer) as *mut core::ffi::c_char,
        );

        token = strtok(
            core::ptr::addr_of_mut!(buffer) as *mut core::ffi::c_char,
            delim,
        );
        while !token.is_null() && button < (MAX_MAP_BUTTONS as core::ffi::c_int) {
            let mut s: *mut core::ffi::c_char = core::ptr::null_mut();
            let mut value: core::ffi::c_int = strtol(
                token,
                &raw mut s,
                10,
            ) as core::ffi::c_int;
            if value < 0 || *s as core::ffi::c_int != '\0' as i32 {
                ERR!("invalid button number: \"%s\"", token);
            } else {
                TRACE!(
                    "mapping physical button %d to DInput button %d\n",
                    value,
                    button,
                );
                (*This).button_map[value as usize] = button;
            }
            token = strtok(core::ptr::null_mut(), delim);
            button += 1;
        }
    }

    (*This).axis_map = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        (((*This).device_axis_count as size_t)
            .wrapping_mul(core::mem::size_of::<core::ffi::c_int>() as size_t)) as SIZE_T,
    ) as *mut core::ffi::c_int;
    if (*This).axis_map.is_null() {
        return DIERR_OUTOFMEMORY;
    }

    if get_config_key(
        hkey,
        appkey,
        (*This).name,
        core::ptr::addr_of_mut!(buffer) as *mut core::ffi::c_char,
        core::mem::size_of::<[core::ffi::c_char; (MAX_PATH + 16) as _]>()
            as DWORD,
    ) == 0
    {
        static mut axis_names: [*const core::ffi::c_char; 12] = [
            b"X\0" as *const u8 as *const core::ffi::c_char,
            b"Y\0" as *const u8 as *const core::ffi::c_char,
            b"Z\0" as *const u8 as *const core::ffi::c_char,
            b"Rx\0" as *const u8 as *const core::ffi::c_char,
            b"Ry\0" as *const u8 as *const core::ffi::c_char,
            b"Rz\0" as *const u8 as *const core::ffi::c_char,
            b"Slider1\0" as *const u8 as *const core::ffi::c_char,
            b"Slider2\0" as *const u8 as *const core::ffi::c_char,
            b"POV1\0" as *const u8 as *const core::ffi::c_char,
            b"POV2\0" as *const u8 as *const core::ffi::c_char,
            b"POV3\0" as *const u8 as *const core::ffi::c_char,
            b"POV4\0" as *const u8 as *const core::ffi::c_char,
        ];
        let delim: *const core::ffi::c_char = b",\0" as *const u8 as *const core::ffi::c_char;
        let mut ptr: *mut core::ffi::c_char = core::ptr::null_mut();

        TRACE!(
            "\"%s\" = \"%s\"\n",
            (*This).name,
            core::ptr::addr_of_mut!(buffer) as *mut core::ffi::c_char,
        );

        ptr = strtok(core::ptr::addr_of_mut!(buffer) as *mut core::ffi::c_char, delim);
        if !ptr.is_null() {
            loop {
                let mut i: core::ffi::c_int = 0;
                while i < ARRAY_SIZE!(axis_names) as core::ffi::c_int {
                    if strcmp(ptr, axis_names[i as usize]) == 0 {
                        if strncmp(
                            ptr,
                            b"POV\0".as_ptr() as *const core::ffi::c_char,
                            3 as size_t,
                        ) == 0
                        {
                            if pov >= 4 {
                                WARN!("Only 4 POVs supported - ignoring extra\n");
                                i = -1;
                            } else {
                                /* Pov takes two axes */
                                *(*This).axis_map.offset(tokens as isize) = i;
                                tokens += 1;
                                pov += 1;
                            }
                        } else {
                            if axis >= 8 {
                                FIXME!("Only 8 Axes supported - ignoring extra\n");
                                i = -1;
                            } else {
                                axis += 1;
                            }
                        }
                        break;
                    }
                    i += 1;
                }

                if i == ARRAY_SIZE!(axis_names) as core::ffi::c_int {
                    ERR!("invalid joystick axis type: \"%s\"\n", ptr);
                    i = -1;
                }

                *(*This).axis_map.offset(tokens as isize) = i;
                tokens += 1;
                ptr = strtok(core::ptr::null_mut(), delim);
                if ptr.is_null() {
                    break;
                }
            }

            if tokens != (*This).device_axis_count {
                ERR!(
                    "not all joystick axes mapped: %d axes(%d,%d), %d arguments\n",
                    (*This).device_axis_count,
                    axis,
                    pov,
                    tokens,
                );
                while tokens < (*This).device_axis_count {
                    *(*This).axis_map.offset(tokens as isize) = -1;
                    tokens += 1;
                }
            }
        }
    } else {
        let mut i: core::ffi::c_int;

        if !default_axis_map.is_null() {
            /* Use default mapping from the driver */
            i = 0;
            while i < (*This).device_axis_count {
                'body: {
                    *(*This).axis_map.offset(i as isize) = *default_axis_map.offset(i as isize);
                    tokens = *default_axis_map.offset(i as isize);
                    if tokens < 0 {
                        break 'body;
                    }
                    if tokens < 8 {
                        axis += 1;
                    } else if tokens < 15 {
                        i += 1;
                        pov += 1;
                        *(*This).axis_map.offset(i as isize) =
                            *default_axis_map.offset(i as isize);
                    }
                }
                i += 1;
            }
        } else {
            /* No config - set default mapping. */
            i = 0;
            while i < (*This).device_axis_count {
                if i < 8 {
                    *(*This).axis_map.offset(i as isize) = axis;
                    axis += 1;
                } else if i < 15 {
                    let fresh_i = i;
                    i += 1;
                    *(*This).axis_map.offset(fresh_i as isize) = 8 + pov;
                    *(*This).axis_map.offset(i as isize) = 8 + pov;
                    pov += 1;
                } else {
                    *(*This).axis_map.offset(i as isize) = -1;
                }
                i += 1;
            }
        }
    }
    (*This).devcaps.dwAxes = axis as DWORD;
    (*This).devcaps.dwPOVs = pov as DWORD;

    if !appkey.is_null() {
        RegCloseKey(appkey);
    }
    if !hkey.is_null() {
        RegCloseKey(hkey);
    }

    return DI_OK;
}
