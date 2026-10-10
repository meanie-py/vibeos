use crate::prelude::*;
use core::ffi::c_void;

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct C2RustUnnamed {
    pub dwType: DWORD,
    pub dwDeviceID: DWORD,
    pub wMid: WORD,
    pub wPid: WORD,
    pub vDriverVersion: MMVERSION,
    pub szPname: [WCHAR; 32],
}

extern "C" {
    fn DbgPrint(Format: PCH, ...) -> ULONG;
}

// ---- file-scope preamble (translated) ----
pub const NO_MIXER_SELECTED: UINT = !0;

unsafe extern "C" fn ClearMixerCache(Mixer: PSND_MIXER) {
    let mut Line: PSND_MIXER_DESTINATION = core::ptr::null_mut();
    let mut NextLine: PSND_MIXER_DESTINATION = core::ptr::null_mut();
    let mut Con: PSND_MIXER_CONNECTION = core::ptr::null_mut();
    let mut NextCon: PSND_MIXER_CONNECTION = core::ptr::null_mut();
    Line = (*Mixer).Lines;
    while !Line.is_null() {
        if !(*Line).Controls.is_null() {
            HeapFree(GetProcessHeap(), 0, (*Line).Controls as LPVOID);
        }
        Con = (*Line).Connections;
        while !Con.is_null() {
            if !(*Con).Controls.is_null() {
                HeapFree(GetProcessHeap(), 0, (*Con).Controls as LPVOID);
            }
            NextCon = (*Con).Next as PSND_MIXER_CONNECTION;
            HeapFree(GetProcessHeap(), 0, Con as LPVOID);
            Con = NextCon;
        }
        NextLine = (*Line).Next as PSND_MIXER_DESTINATION;
        HeapFree(GetProcessHeap(), 0, Line as LPVOID);
        Line = NextLine;
    }
    (*Mixer).Lines = core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn SndMixerCreate(mut hWndNotification: HWND, mut MixerId: UINT) -> PSND_MIXER {
    let mut Mixer: PSND_MIXER = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        ::core::mem::size_of::<SND_MIXER>() as SIZE_T,
    ) as PSND_MIXER;
    if !Mixer.is_null() {
        (*Mixer).hWndNotification = hWndNotification;
        (*Mixer).MixersCount = mixerGetNumDevs();
        (*Mixer).MixerId = NO_MIXER_SELECTED as UINT;
        if (*Mixer).MixersCount > 0 as UINT {
            /* select the first mixer by default */
            SndMixerSelect(Mixer, MixerId);
        }
    }
    return Mixer;
}
#[no_mangle]
pub unsafe extern "C" fn SndMixerDestroy(Mixer: PSND_MIXER) {
    ClearMixerCache(Mixer);
    SndMixerClose(Mixer);
    HeapFree(GetProcessHeap(), 0 as DWORD, Mixer as LPVOID);
}
#[no_mangle]
pub unsafe extern "C" fn SndMixerClose(Mixer: PSND_MIXER) {
    if !(*Mixer).hmx.is_null() {
        mixerClose((*Mixer).hmx);
        (*Mixer).hmx = core::ptr::null_mut();
        (*Mixer).MixerId = NO_MIXER_SELECTED;
    }
}
#[no_mangle]
pub unsafe extern "C" fn SndMixerQueryControls(Mixer: PSND_MIXER, DisplayControls: PUINT, LineInfo: LPMIXERLINE, Controls: *mut LPMIXERCONTROL) -> BOOL {
    if (*LineInfo).cControls > 0 {
        *Controls = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY as DWORD,
            ((*LineInfo).cControls as SIZE_T)
                .wrapping_mul(::core::mem::size_of::<MIXERCONTROL>() as SIZE_T),
        ) as *mut MIXERCONTROL as LPMIXERCONTROL;
        if !(*Controls).is_null() {
            let mut LineControls: MIXERLINECONTROLS = core::mem::zeroed();
            let mut Result: MMRESULT = 0;
            let mut j: UINT = 0;

            LineControls.cbStruct = ::core::mem::size_of::<MIXERLINECONTROLS>() as DWORD;
            LineControls.dwLineID = (*LineInfo).dwLineID;
            LineControls.cControls = (*LineInfo).cControls;
            LineControls.cbmxctrl = ::core::mem::size_of::<MIXERCONTROL>() as DWORD;
            LineControls.pamxctrl = *Controls as *mut MIXERCONTROL as LPMIXERCONTROL;

            Result = mixerGetLineControls(
                (*Mixer).hmx as HMIXEROBJ,
                core::ptr::addr_of_mut!(LineControls),
                MIXER_GETLINECONTROLSF_ALL as DWORD,
            );
            if Result == MMSYSERR_NOERROR as MMRESULT {
                j = 0 as UINT;
                while (j as DWORD) < LineControls.cControls {
                    if SndMixerIsDisplayControl(
                        Mixer,
                        (*Controls).offset(j as isize) as LPMIXERCONTROL,
                    ) != 0
                    {
                        *DisplayControls = (*DisplayControls).wrapping_add(1);
                    }

                    DbgPrint(
                        b"SNDVOL32: %s:%i: \0" as *const u8 as *const ::core::ffi::c_char as PCH,
                        b"base/applications/sndvol32/mixer.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        147 as ::core::ffi::c_int,
                    );
                    DPRINT!(
                        b"Line control: %ws (0x%x, 0x%x)\n\0" as *const u8
                            as *const ::core::ffi::c_char as PCH,
                        core::ptr::addr_of_mut!((*(*Controls).offset(j as isize)).szName) as *mut WCHAR,
                        (*(*Controls).offset(j as isize)).fdwControl,
                        (*(*Controls).offset(j as isize)).dwControlType,
                    );

                    j = j.wrapping_add(1);
                }

                return (TRUE) as BOOL;
            } else {
                HeapFree(
                    GetProcessHeap(),
                    0 as DWORD,
                    *Controls as PVOID,
                );
                *Controls = core::ptr::null_mut::<MIXERCONTROLW>();

                DbgPrint(
                    b"SNDVOL32: %s:%i: \0" as *const u8 as *const ::core::ffi::c_char as PCH,
                    b"base/applications/sndvol32/mixer.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    158 as ::core::ffi::c_int,
                );
                DPRINT!(
                    b"Failed to get line (ID: 0x%x) controls: %d\n\0" as *const u8
                        as *const ::core::ffi::c_char as PCH,
                    (*LineInfo).dwLineID,
                    Result,
                );
            }
        } else {
            DbgPrint(
                b"SNDVOL32: %s:%i: \0" as *const u8 as *const ::core::ffi::c_char as PCH,
                b"base/applications/sndvol32/mixer.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                163 as ::core::ffi::c_int,
            );
            DPRINT!(
                b"Failed to allocate memory for %d line (ID: 0x%x) controls!\n\0" as *const u8
                    as *const ::core::ffi::c_char as PCH,
                (*LineInfo).dwLineID,
                (*LineInfo).cControls,
            );
        }

        return (FALSE) as BOOL;
    } else {
        return (TRUE) as BOOL;
    }
}

unsafe extern "C" fn SndMixerQueryConnections(
    mut Mixer: PSND_MIXER,
    mut Line: PSND_MIXER_DESTINATION,
) -> BOOL {
    let mut i: UINT = 0;
    let mut DispControls: UINT = 0;
    let mut LineInfo: MIXERLINE = tagMIXERLINEW {
        cbStruct: 0,
        dwDestination: 0,
        dwSource: 0,
        dwLineID: 0,
        fdwLine: 0,
        dwUser: 0,
        dwComponentType: 0,
        cChannels: 0,
        cConnections: 0,
        cControls: 0,
        szShortName: [0; 16],
        szName: [0; 64],
        Target: tagMIXERLINEW__bindgen_ty_1 {
            dwType: 0,
            dwDeviceID: 0,
            wMid: 0,
            wPid: 0,
            vDriverVersion: 0,
            szPname: [0; 32],
        },
    };
    let mut Result: MMRESULT = 0;
    let mut Ret: BOOL = TRUE as BOOL;
    LineInfo.cbStruct = ::core::mem::size_of::<MIXERLINE>() as DWORD;
    i = (*Line).Info.cConnections as UINT;
    while i > (0) as UINT {
        LineInfo.dwDestination = (*Line).Info.dwDestination;
        LineInfo.dwSource = i.wrapping_sub((1) as UINT) as DWORD;
        Result = mixerGetLineInfo(
            (*Mixer).hmx as HMIXEROBJ,
            &raw mut LineInfo,
            MIXER_GETLINEINFOF_SOURCE as DWORD,
        );
        if Result == MMSYSERR_NOERROR as MMRESULT {
            let mut Controls: LPMIXERCONTROL = ::core::ptr::null_mut::<MIXERCONTROLW>();
            let mut Con: PSND_MIXER_CONNECTION = ::core::ptr::null_mut::<_SND_MIXER_CONNECTION>();
            DbgPrint(
                b"SNDVOL32: %s:%i: \0" as *const u8 as *const ::core::ffi::c_char as PCH,
                b"base/applications/sndvol32/mixer.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                196 as ::core::ffi::c_int,
            );
            DPRINT!(
                b"++ Source: %ws\n\0" as *const u8 as *const ::core::ffi::c_char as PCH,
                &raw mut LineInfo.szName as *mut WCHAR,
            );
            DispControls = 0 as UINT;
            if SndMixerQueryControls(
                Mixer,
                &raw mut DispControls,
                &raw mut LineInfo,
                &raw mut Controls,
            ) == 0
            {
                DbgPrint(
                    b"SNDVOL32: %s:%i: \0" as *const u8 as *const ::core::ffi::c_char
                        as PCH,
                    b"base/applications/sndvol32/mixer.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    205 as ::core::ffi::c_int,
                );
                DPRINT!(
                    b"Failed to query connection controls\n\0" as *const u8
                        as *const ::core::ffi::c_char as PCH,
                );
                Ret = FALSE as BOOL;
                break;
            } else {
                Con = HeapAlloc(
                    GetProcessHeap(),
                    HEAP_ZERO_MEMORY as DWORD,
                    ::core::mem::size_of::<SND_MIXER_CONNECTION>() as SIZE_T,
                ) as *mut SND_MIXER_CONNECTION as PSND_MIXER_CONNECTION;
                if !Con.is_null() {
                    (*Con).Info = LineInfo;
                    (*Con).Controls = Controls;
                    (*Con).DisplayControls = DispControls;
                    (*Con).Next = (*Line).Connections as *mut _SND_MIXER_CONNECTION;
                    (*Line).Connections = Con;
                } else {
                    HeapFree(
                        GetProcessHeap(),
                        0 as DWORD,
                        Controls as PVOID,
                    );
                }
                i = i.wrapping_sub(1);
            }
        } else {
            DbgPrint(
                b"SNDVOL32: %s:%i: \0" as *const u8 as *const ::core::ffi::c_char as PCH,
                b"base/applications/sndvol32/mixer.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                230 as ::core::ffi::c_int,
            );
            DPRINT!(
                b"Failed to get connection information: %d\n\0" as *const u8
                    as *const ::core::ffi::c_char as PCH,
                Result,
            );
            Ret = FALSE as BOOL;
            break;
        }
    }
    return Ret;
}

unsafe extern "C" fn SndMixerQueryDestinations(Mixer: PSND_MIXER) -> BOOL {
    let mut i: UINT;
    let mut Ret: BOOL = (TRUE) as i32;

    i = (*Mixer).Caps.cDestinations;
    while i > 0 {
        let mut Line: PSND_MIXER_DESTINATION = core::ptr::null_mut();

        Line = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY as DWORD,
            (core::mem::size_of::<SND_MIXER_DESTINATION>()) as SIZE_T,
        ) as *mut SND_MIXER_DESTINATION as PSND_MIXER_DESTINATION;
        if !Line.is_null() {
            (*Line).Info.cbStruct = core::mem::size_of::<MIXERLINE>() as DWORD;
            (*Line).Info.dwDestination = (i - 1) as DWORD;
            if mixerGetLineInfo(
                (*Mixer).hmx as HMIXEROBJ,
                core::ptr::addr_of_mut!((*Line).Info),
                MIXER_GETLINEINFOF_DESTINATION as DWORD,
            ) == MMSYSERR_NOERROR
            {
                DbgPrint(
                    b"SNDVOL32: %s:%i: \0".as_ptr() as *mut i8,
                    b"base/applications/sndvol32/mixer.c\0"
                        .as_ptr() as *const i8,
                    260 as i32,
                );
                DPRINT!(
                    b"+ Destination: %ws (0x%x, %d)\n\0".as_ptr() as *const i8,
                    core::ptr::addr_of_mut!((*Line).Info.szName) as *mut WCHAR,
                    (*Line).Info.dwLineID,
                    (*Line).Info.dwComponentType,
                );

                if SndMixerQueryControls(
                    Mixer,
                    core::ptr::addr_of_mut!((*Line).DisplayControls),
                    core::ptr::addr_of_mut!((*Line).Info),
                    core::ptr::addr_of_mut!((*Line).Controls),
                ) == 0
                {
                    DbgPrint(
                        b"SNDVOL32: %s:%i: \0".as_ptr() as *mut i8,
                        b"base/applications/sndvol32/mixer.c\0"
                            .as_ptr() as *const i8,
                        267 as i32,
                    );
                    DPRINT!(
                        b"Failed to query mixer controls!\n\0".as_ptr() as *const i8,
                    );
                    Ret = (FALSE) as i32;
                    break;
                }

                if SndMixerQueryConnections(Mixer, Line) == 0 {
                    DbgPrint(
                        b"SNDVOL32: %s:%i: \0".as_ptr() as *mut i8,
                        b"base/applications/sndvol32/mixer.c\0"
                            .as_ptr() as *const i8,
                        274 as i32,
                    );
                    DPRINT!(
                        b"Failed to query mixer connections!\n\0".as_ptr() as *const i8,
                    );
                    Ret = (FALSE) as i32;
                    break;
                }

                (*Line).Next = (*Mixer).Lines;
                (*Mixer).Lines = Line;
            } else {
                DbgPrint(
                    b"SNDVOL32: %s:%i: \0".as_ptr() as *mut i8,
                    b"base/applications/sndvol32/mixer.c\0"
                        .as_ptr() as *const i8,
                    284 as i32,
                );
                DPRINT!(
                    b"Failed to get line information for id %d!\n\0".as_ptr() as *const i8,
                    i,
                );
                HeapFree(
                    GetProcessHeap(),
                    0 as DWORD,
                    Line as LPVOID,
                );
                Ret = (FALSE) as i32;
                break;
            }
        } else {
            DbgPrint(
                b"SNDVOL32: %s:%i: \0".as_ptr() as *mut i8,
                b"base/applications/sndvol32/mixer.c\0"
                    .as_ptr() as *const i8,
                294 as i32,
            );
            DPRINT!(
                b"Allocation of SND_MIXER_DEST structure for id %d failed!\n\0".as_ptr() as *const i8,
                i,
            );
            Ret = (FALSE) as i32;
            break;
        }
        i = i.wrapping_sub(1);
    }

    return Ret;
}
#[no_mangle]
pub unsafe extern "C" fn SndMixerSelect(Mixer: PSND_MIXER, MixerId: UINT) -> BOOL {
    if MixerId >= (*Mixer).MixersCount {
        return (FALSE) as BOOL;
    }

    SndMixerClose(Mixer);

    if mixerOpen(
        core::ptr::addr_of_mut!((*Mixer).hmx),
        MixerId,
        (*Mixer).hWndNotification as DWORD_PTR,
        0,
        CALLBACK_WINDOW | MIXER_OBJECTF_MIXER,
    ) == MMSYSERR_NOERROR
        || mixerOpen(
            core::ptr::addr_of_mut!((*Mixer).hmx),
            MixerId,
            (*Mixer).hWndNotification as DWORD_PTR,
            0,
            CALLBACK_WINDOW,
        ) == MMSYSERR_NOERROR
        || mixerOpen(
            core::ptr::addr_of_mut!((*Mixer).hmx),
            MixerId,
            0,
            0,
            0,
        ) == MMSYSERR_NOERROR
    {
        if mixerGetDevCaps(
            (MixerId) as _,
            core::ptr::addr_of_mut!((*Mixer).Caps),
            core::mem::size_of::<MIXERCAPS>() as UINT,
        ) == MMSYSERR_NOERROR
        {
            let mut Ret: BOOL = (FALSE) as i32;

            (*Mixer).MixerId = MixerId;

            ClearMixerCache(Mixer);

            Ret = SndMixerQueryDestinations(Mixer);

            if Ret == 0 {
                ClearMixerCache(Mixer);
            }

            return Ret;
        } else {
            mixerClose((*Mixer).hmx);
        }
    }

    (*Mixer).hmx = core::ptr::null_mut::<HMIXER__>();
    (*Mixer).MixerId = NO_MIXER_SELECTED;
    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn SndMixerGetSelection(Mixer: PSND_MIXER) -> UINT {
    return (*Mixer).MixerId;
}
#[no_mangle]
pub unsafe extern "C" fn SndMixerGetProductName(Mixer: PSND_MIXER, lpBuffer: LPTSTR, uSize: UINT) -> INT {
    if !(*Mixer).hmx.is_null() {
        let mut lnsz: UINT = lstrlen(
            core::ptr::addr_of_mut!((*Mixer).Caps.szPname) as *mut WCHAR as LPCWSTR
        ) as UINT;
        if lnsz.wrapping_add(1 as UINT) > uSize {
            return lnsz.wrapping_add(1 as UINT) as INT;
        } else {
            memcpy(
                lpBuffer as *mut core::ffi::c_void,
                core::ptr::addr_of!((*Mixer).Caps.szPname) as *mut WCHAR as *const core::ffi::c_void,
                (lnsz as usize).wrapping_mul(core::mem::size_of::<TCHAR>() as usize),
            );
            *lpBuffer.offset(lnsz as isize) = ('\0' as u16);
            return lnsz as INT;
        }
    }

    return -(1 as INT);
}
#[no_mangle]
pub unsafe extern "C" fn SndMixerGetLineName(mut Mixer: PSND_MIXER, mut LineID: DWORD, mut lpBuffer: LPTSTR, mut uSize: UINT, mut LongName: BOOL) -> INT {
    if !(*Mixer).hmx.is_null() {
        let mut lnsz: UINT = 0;
        let mut Line: PSND_MIXER_DESTINATION = core::ptr::null_mut();
        let mut lpl: LPMIXERLINE = core::ptr::null_mut();
        Line = (*Mixer).Lines;
        while !Line.is_null() {
            if (*Line).Info.dwLineID == LineID {
                lpl = &raw mut (*Line).Info as LPMIXERLINE;
                break;
            } else {
                Line = (*Line).Next as PSND_MIXER_DESTINATION;
            }
        }
        if !lpl.is_null() {
            lnsz = lstrlen(
                (if LongName != 0 {
                    &raw mut (*lpl).szName as LPTSTR
                } else {
                    &raw mut (*lpl).szShortName as LPTSTR
                }) as LPCTSTR,
            ) as UINT;
            if lnsz.wrapping_add(1 as UINT) > uSize {
                return lnsz.wrapping_add(1 as UINT) as INT;
            } else {
                memcpy(
                    lpBuffer as *mut core::ffi::c_void,
                    (if LongName != 0 {
                        &raw mut (*lpl).szName as LPTSTR
                    } else {
                        &raw mut (*lpl).szShortName as LPTSTR
                    }) as *const core::ffi::c_void,
                    (lnsz as size_t).wrapping_mul(
                        core::mem::size_of::<TCHAR>() as size_t,
                    ),
                );
                *lpBuffer.offset(lnsz as isize) = '\0' as TCHAR;
                return lnsz as INT;
            }
        }
    }
    return -(1 as INT);
}
#[no_mangle]
pub unsafe extern "C" fn SndMixerEnumProducts(Mixer: PSND_MIXER, EnumProc: PFNSNDMIXENUMPRODUCTS, Context: PVOID) -> BOOL {
    let mut Caps: MIXERCAPS = core::mem::zeroed();
    let mut hMixer: HMIXER = core::ptr::null_mut();
    let mut i: UINT;
    let mut Ret: BOOL = (TRUE) as i32;

    i = 0;
    while i < (*Mixer).MixersCount {
        if mixerOpen(
            &raw mut hMixer,
            i,
            0,
            0,
            0,
        ) == MMSYSERR_NOERROR
        {
            if mixerGetDevCaps(
                (i) as _,
                &raw mut Caps,
                core::mem::size_of::<MIXERCAPS>() as UINT,
            ) == MMSYSERR_NOERROR
            {
                if (EnumProc.unwrap_unchecked())(
                    Mixer,
                    i,
                    &raw mut Caps.szPname as *mut WCHAR as LPCTSTR,
                    Context,
                ) == 0
                {
                    mixerClose(hMixer);
                    Ret = (FALSE) as i32;
                    break;
                }
            } else {
                DbgPrint(
                    b"SNDVOL32: %s:%i: \0".as_ptr() as *const i8 as PCH,
                    b"base/applications/sndvol32/mixer.c\0"
                        .as_ptr() as *const i8,
                    464 as core::ffi::c_int,
                );
                DPRINT!(
                    b"Failed to get device capabilities for mixer id %d!\n\0".as_ptr() as *const i8 as PCH,
                    i,
                );
            }
            mixerClose(hMixer);
        }
        i = i.wrapping_add(1);
    }

    return Ret;
}
#[no_mangle]
pub unsafe extern "C" fn SndMixerSetVolumeControlDetails(Mixer: PSND_MIXER, dwControlID: DWORD, cChannels: DWORD, cbDetails: DWORD, paDetails: LPVOID) -> INT {
    let mut MixerDetails: MIXERCONTROLDETAILS = core::mem::zeroed();

    if !(*Mixer).hmx.is_null() {
        MixerDetails.cbStruct = core::mem::size_of::<MIXERCONTROLDETAILS>() as DWORD;
        MixerDetails.dwControlID = dwControlID;
        MixerDetails.cChannels = cChannels;
        MixerDetails.__bindgen_anon_1.cMultipleItems = 0 as DWORD;
        MixerDetails.cbDetails = cbDetails;
        MixerDetails.paDetails = paDetails as PVOID;

        if mixerSetControlDetails(
            (*Mixer).hmx as HMIXEROBJ,
            core::ptr::addr_of_mut!(MixerDetails),
            (MIXER_SETCONTROLDETAILSF_VALUE | MIXER_OBJECTF_HMIXER) as DWORD,
        ) == MMSYSERR_NOERROR as MMRESULT
        {
            return 1 as INT;
        }
    }

    return -(1 as INT);
}
#[no_mangle]
pub unsafe extern "C" fn SndMixerGetVolumeControlDetails(Mixer: PSND_MIXER, dwControlID: DWORD, cChannels: DWORD, cbDetails: DWORD, paDetails: LPVOID) -> INT {
    let mut MixerDetails: MIXERCONTROLDETAILS = core::mem::zeroed();

    if !(*Mixer).hmx.is_null() {
        MixerDetails.cbStruct = core::mem::size_of::<MIXERCONTROLDETAILS>() as DWORD;
        MixerDetails.dwControlID = dwControlID;
        MixerDetails.cChannels = cChannels;
        MixerDetails.__bindgen_anon_1.cMultipleItems = 0;
        MixerDetails.cbDetails = cbDetails;
        MixerDetails.paDetails = paDetails as PVOID;

        if mixerGetControlDetails(
            (*Mixer).hmx as HMIXEROBJ,
            core::ptr::addr_of_mut!(MixerDetails),
            (MIXER_GETCONTROLDETAILSF_VALUE | MIXER_OBJECTF_HMIXER) as DWORD,
        ) == MMSYSERR_NOERROR as MMRESULT
        {
            return 1 as INT;
        }
    }
    return -(1 as INT);
}
#[no_mangle]
pub unsafe extern "C" fn SndMixerGetDestinationCount(mut Mixer: PSND_MIXER) -> INT {
    return if !(*Mixer).hmx.is_null() {
        (*Mixer).Caps.cDestinations as INT
    } else {
        -(1 as INT)
    };
}
#[no_mangle]
pub unsafe extern "C" fn SndMixerEnumLines(mut Mixer: PSND_MIXER, mut EnumProc: PFNSNDMIXENUMLINES, mut Context: PVOID) -> BOOL {
    if !(*Mixer).hmx.is_null() {
        let mut Line: PSND_MIXER_DESTINATION = core::ptr::null_mut();
        Line = (*Mixer).Lines;
        while !Line.is_null() {
            if !(EnumProc.unwrap_unchecked())(
                Mixer,
                &raw mut (*Line).Info,
                (*Line).DisplayControls,
                Context,
            ) != 0
            {
                return (FALSE) as BOOL;
            }
            Line = (*Line).Next as PSND_MIXER_DESTINATION;
        }
        return (TRUE) as BOOL;
    }
    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn SndMixerEnumConnections(mut Mixer: PSND_MIXER, mut LineID: DWORD, mut EnumProc: PFNSNDMIXENUMCONNECTIONS, mut Context: PVOID) -> BOOL {
    if !(*Mixer).hmx.is_null() {
        let mut Line: PSND_MIXER_DESTINATION = ::core::ptr::null_mut::<_SND_MIXER_DESTINATION>();
        Line = (*Mixer).Lines;
        while !Line.is_null() {
            if (*Line).Info.dwLineID == LineID {
                let mut Connection: PSND_MIXER_CONNECTION =
                    ::core::ptr::null_mut::<_SND_MIXER_CONNECTION>();
                if (*Line).DisplayControls != 0 {
                    if (EnumProc.unwrap_unchecked())(
                        Mixer,
                        LineID,
                        &raw mut (*Line).Info,
                        Context,
                    ) == 0
                    {
                        return (FALSE) as BOOL;
                    }
                }
                Connection = (*Line).Connections;
                while !Connection.is_null() {
                    if (EnumProc.unwrap_unchecked())(
                        Mixer,
                        LineID,
                        &raw mut (*Connection).Info,
                        Context,
                    ) == 0
                    {
                        return (FALSE) as BOOL;
                    }
                    Connection = (*Connection).Next as PSND_MIXER_CONNECTION;
                }
                return (TRUE) as BOOL;
            }
            Line = (*Line).Next as PSND_MIXER_DESTINATION;
        }
    }
    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn SndMixerIsDisplayControl(
    mut Mixer: PSND_MIXER,
    mut Control: LPMIXERCONTROL,
) -> BOOL {
    if !(*Mixer).hmx.is_null()
        && (*Control).fdwControl & MIXERCONTROL_CONTROLF_DISABLED as DWORD == 0
    {
        match (*Control).dwControlType & MIXERCONTROL_CT_CLASS_MASK as DWORD {
            MIXERCONTROL_CT_CLASS_FADER | MIXERCONTROL_CT_CLASS_SWITCH => return (TRUE) as i32,
            _ => {}
        }
    }
    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn SndMixerGetLineByName(Mixer: PSND_MIXER, LineID: DWORD, LineName: LPWSTR) -> LPMIXERLINE {
    let mut Line: PSND_MIXER_DESTINATION = core::ptr::null_mut();
    let mut Connection: PSND_MIXER_CONNECTION = core::ptr::null_mut();

    if (*Mixer).hmx == core::ptr::null_mut() {
        return core::ptr::null_mut();
    }

    Line = (*Mixer).Lines;
    while !Line.is_null() {
        if (*Line).Info.dwLineID == LineID {
            if (*Line).DisplayControls != 0 {
                if _wcsicmp(core::ptr::addr_of_mut!((*Line).Info.szName) as *mut WCHAR, LineName as *const wchar_t) == 0 {
                    return core::ptr::addr_of_mut!((*Line).Info);
                }
            }

            Connection = (*Line).Connections;
            while !Connection.is_null() {
                if _wcsicmp(core::ptr::addr_of_mut!((*Connection).Info.szName) as *mut WCHAR, LineName as *const wchar_t) == 0 {
                    return core::ptr::addr_of_mut!((*Connection).Info);
                }
                Connection = (*Connection).Next as PSND_MIXER_CONNECTION;
            }

            return core::ptr::null_mut();
        }
        Line = (*Line).Next as PSND_MIXER_DESTINATION;
    }

    return core::ptr::null_mut();
}
