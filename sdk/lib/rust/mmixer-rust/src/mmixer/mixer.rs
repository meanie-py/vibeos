use crate::prelude::*;
use ntkrnl_sys::ddk::{LPWSTR, PHANDLE};
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn MMixerGetCount(MixerContext: PMIXER_CONTEXT) -> ULONG {
    let mut MixerList: PMIXER_LIST;
    let mut Status: MIXER_STATUS;

    /* verify mixer context */
    Status = MMixerVerifyContext(MixerContext);

    if Status != MM_STATUS_SUCCESS {
        /* invalid context passed */
        return Status;
    }

    /* grab mixer list */
    MixerList = (*MixerContext).MixerContext as PMIXER_LIST;

    // return number of mixers
    return (*MixerList).MixerListCount;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetCapabilities(
    mut MixerContext: PMIXER_CONTEXT,
    mut MixerIndex: ULONG,
    mut MixerCaps: LPMIXERCAPSW,
) -> MIXER_STATUS {
    let mut Status: MIXER_STATUS = core::mem::zeroed();
    let mut MixerInfo: LPMIXER_INFO = core::ptr::null_mut();

    /* verify mixer context */
    Status = MMixerVerifyContext(MixerContext);

    if Status != MM_STATUS_SUCCESS {
        /* invalid context passed */
        return Status;
    }

    /* get mixer info */
    MixerInfo = MMixerGetMixerInfoByIndex(MixerContext, MixerIndex);

    if MixerInfo.is_null() {
        // invalid device index
        return MM_STATUS_INVALID_PARAMETER;
    }

    (*MixerCaps).wMid = (*MixerInfo).MixCaps.wMid;
    (*MixerCaps).wPid = (*MixerInfo).MixCaps.wPid;
    (*MixerCaps).vDriverVersion = (*MixerInfo).MixCaps.vDriverVersion;
    (*MixerCaps).fdwSupport = (*MixerInfo).MixCaps.fdwSupport;
    (*MixerCaps).cDestinations = (*MixerInfo).MixCaps.cDestinations;

    ASSERT!((*MixerInfo).MixCaps.szPname[(MAXPNAMELEN - 1) as usize] == 0);
    wcscpy(
        core::ptr::addr_of_mut!((*MixerCaps).szPname) as *mut WCHAR,
        core::ptr::addr_of_mut!((*MixerInfo).MixCaps.szPname) as *mut WCHAR,
    );

    return MM_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerOpen(
    MixerContext: PMIXER_CONTEXT,
    MixerId: ULONG,
    MixerEventContext: PVOID,
    MixerEventRoutine: PMIXER_EVENT,
    MixerHandle: PHANDLE,
) -> MIXER_STATUS {
    let mut Status: MIXER_STATUS;
    let mut MixerInfo: LPMIXER_INFO;

    /* verify mixer context */
    Status = MMixerVerifyContext(MixerContext);

    if Status != MM_STATUS_SUCCESS {
        /* invalid context passed */
        DPRINT1!("invalid context\n");
        return Status;
    }

    /* get mixer info */
    MixerInfo = MMixerGetMixerInfoByIndex(MixerContext, MixerId) as LPMIXER_INFO;
    if MixerInfo.is_null() {
        /* invalid mixer id */
        DPRINT1!("invalid mixer id %lu\n", MixerId);
        return MM_STATUS_INVALID_PARAMETER;
    }

    /* add the event */
    Status = MMixerAddEvent(MixerContext, MixerInfo, MixerEventContext, MixerEventRoutine);

    /* store result */
    *MixerHandle = MixerInfo as HANDLE;
    return MM_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerClose(
    MixerContext: PMIXER_CONTEXT,
    MixerId: ULONG,
    MixerEventContext: PVOID,
    MixerEventRoutine: PMIXER_EVENT,
) -> MIXER_STATUS {
    let mut Status: MIXER_STATUS;
    let mut MixerInfo: LPMIXER_INFO;

    /* verify mixer context */
    Status = MMixerVerifyContext(MixerContext);

    if Status != MM_STATUS_SUCCESS {
        /* invalid context passed */
        DPRINT1!("invalid context\n");
        return Status;
    }

    /* get mixer info */
    MixerInfo = MMixerGetMixerInfoByIndex(MixerContext, MixerId);
    if MixerInfo.is_null() {
        /* invalid mixer id */
        DPRINT1!("invalid mixer id %lu\n", MixerId);
        return MM_STATUS_INVALID_PARAMETER;
    }

    /* remove event from list */
    return MMixerRemoveEvent(MixerContext, MixerInfo, MixerEventContext, MixerEventRoutine);
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetLineInfo(
    mut MixerContext: PMIXER_CONTEXT,
    mut MixerHandle: HANDLE,
    mut MixerId: ULONG,
    mut Flags: ULONG,
    mut MixerLine: LPMIXERLINEW,
) -> MIXER_STATUS {
    let mut Status: MIXER_STATUS = 0;
    let mut MixerInfo: LPMIXER_INFO = core::ptr::null_mut();
    let mut MixerLineSrc: LPMIXERLINE_EXT = core::ptr::null_mut();
    let mut DestinationLineID: ULONG = 0;

    /* verify mixer context */
    Status = MMixerVerifyContext(MixerContext);

    if Status != MM_STATUS_SUCCESS {
        /* invalid context passed */
        return Status;
    }
    if (Flags & (MIXER_OBJECTF_MIXER | MIXER_OBJECTF_HMIXER)) == MIXER_OBJECTF_MIXER {
        /* caller passed mixer id */
        MixerHandle = MMixerGetMixerInfoByIndex(MixerContext, MixerId) as HANDLE;

        if MixerHandle.is_null() {
            /* invalid parameter */
            return MM_STATUS_INVALID_PARAMETER;
        }
    }

    if (*MixerLine).cbStruct != core::mem::size_of::<MIXERLINEW>() as ULONG {
        DPRINT1!(
            "MixerLine Expected %lu but got %lu\n",
            core::mem::size_of::<MIXERLINEW>(),
            (*MixerLine).cbStruct
        );
        return MM_STATUS_INVALID_PARAMETER;
    }

    /* clear hmixer from flags */
    Flags &= !MIXER_OBJECTF_HMIXER;

    DPRINT!("MMixerGetLineInfo MixerId %lu Flags %lu\n", MixerId, Flags);

    if Flags == MIXER_GETLINEINFOF_DESTINATION {
        /* cast to mixer info */
        MixerInfo = MixerHandle as LPMIXER_INFO;

        /* calculate destination line id */
        DestinationLineID = (*MixerLine)
            .dwDestination
            .wrapping_add(DESTINATION_LINE as DWORD) as ULONG;

        /* get destination line */
        MixerLineSrc = MMixerGetSourceMixerLineByLineId(MixerInfo, DestinationLineID as DWORD);

        if MixerLineSrc.is_null() {
            DPRINT1!(
                "MixerCaps Name %S DestinationLineCount %lu dwDestination %lu not found\n",
                &raw mut (*MixerInfo).MixCaps.szPname as *mut WCHAR,
                (*MixerInfo).MixCaps.cDestinations,
                (*MixerLine).dwDestination
            );
            return MM_STATUS_UNSUCCESSFUL;
        }
        /* copy mixer line */
        (*MixerContext).Copy.unwrap_unchecked()(
            MixerLine as PVOID,
            &raw mut (*MixerLineSrc).Line as PVOID,
            core::mem::size_of::<MIXERLINEW>() as ULONG,
        );

        /* make sure it is null terminated */
        (*MixerLine).szName[(MIXER_LONG_NAME_CHARS - 1) as usize] =
            0x0000u16;
        (*MixerLine).szShortName[(MIXER_SHORT_NAME_CHARS - 1) as usize] =
            0x0000u16;
        (*MixerLine).Target.szPname[(MAXPNAMELEN - 1) as usize] =
            0x0000u16;

        /* done */
        return MM_STATUS_SUCCESS;
    } else if Flags == MIXER_GETLINEINFOF_SOURCE {
        /* cast to mixer info */
        MixerInfo = MixerHandle as LPMIXER_INFO;

        /* calculate destination line id */
        DestinationLineID = (*MixerLine)
            .dwDestination
            .wrapping_add(DESTINATION_LINE as DWORD) as ULONG;

        /* get destination line */
        MixerLineSrc = MMixerGetSourceMixerLineByLineId(MixerInfo, DestinationLineID as DWORD);

        if MixerLineSrc.is_null() {
            DPRINT1!(
                "MixerCaps Name %S DestinationLineCount %lu dwDestination %lu not found\n",
                &raw mut (*MixerInfo).MixCaps.szPname as *mut WCHAR,
                (*MixerInfo).MixCaps.cDestinations,
                (*MixerLine).dwDestination
            );
            return MM_STATUS_UNSUCCESSFUL;
        }

        /* check if dwSource is out of bounds */
        if (*MixerLine).dwSource >= (*MixerLineSrc).Line.cConnections {
            DPRINT1!(
                "MixerCaps Name %S MixerLineName %S Connections %lu dwSource %lu not found\n",
                &raw mut (*MixerInfo).MixCaps.szPname as *mut WCHAR,
                &raw mut (*MixerLineSrc).Line.szName as *mut WCHAR,
                (*MixerLineSrc).Line.cConnections,
                (*MixerLine).dwSource
            );
            return MM_STATUS_UNSUCCESSFUL;
        }

        /* calculate destination line id */
        DestinationLineID = (*MixerLine)
            .dwSource
            .wrapping_mul(SOURCE_LINE as DWORD)
            .wrapping_add((*MixerLine).dwDestination) as ULONG;

        DPRINT!(
            "MixerName %S cDestinations %lu MixerLineName %S cConnections %lu dwSource %lu dwDestination %lu ID %lx\n",
            &raw mut (*MixerInfo).MixCaps.szPname as *mut WCHAR,
            (*MixerInfo).MixCaps.cDestinations,
            &raw mut (*MixerLineSrc).Line.szName as *mut WCHAR,
            (*MixerLineSrc).Line.cConnections,
            (*MixerLine).dwSource,
            (*MixerLine).dwDestination,
            DestinationLineID
        );
        /* get target destination line id */
        MixerLineSrc = MMixerGetSourceMixerLineByLineId(MixerInfo, DestinationLineID as DWORD);

        /* sanity check */
        ASSERT!(!MixerLineSrc.is_null());

        DPRINT!(
            "Line %u Name %S\n",
            (*MixerLineSrc).Line.dwSource,
            &raw mut (*MixerLineSrc).Line.szName as *mut WCHAR
        );

        /* copy mixer line */
        (*MixerContext).Copy.unwrap_unchecked()(
            MixerLine as PVOID,
            &raw mut (*MixerLineSrc).Line as PVOID,
            core::mem::size_of::<MIXERLINEW>() as ULONG,
        );

        /* make sure it is null terminated */
        (*MixerLine).szName[(MIXER_LONG_NAME_CHARS - 1) as usize] =
            0x0000u16;
        (*MixerLine).szShortName[(MIXER_SHORT_NAME_CHARS - 1) as usize] =
            0x0000u16;
        (*MixerLine).Target.szPname[(MAXPNAMELEN - 1) as usize] =
            0x0000u16;

        /* done */
        return MM_STATUS_SUCCESS;
    } else if Flags == MIXER_GETLINEINFOF_LINEID {
        /* cast to mixer info */
        MixerInfo = MixerHandle as LPMIXER_INFO;

        /* try to find line */
        MixerLineSrc = MMixerGetSourceMixerLineByLineId(MixerInfo, (*MixerLine).dwLineID);
        if MixerLineSrc.is_null() {
            /* invalid parameter */
            DPRINT1!(
                "MMixerGetLineInfo: MixerName %S Line not found 0x%lx\n",
                &raw mut (*MixerInfo).MixCaps.szPname as *mut WCHAR,
                (*MixerLine).dwLineID
            );
            return MM_STATUS_INVALID_PARAMETER;
        }

        DPRINT!(
            "Line %u Name %S\n",
            (*MixerLineSrc).Line.dwSource,
            &raw mut (*MixerLineSrc).Line.szName as *mut WCHAR
        );

        /* copy mixer line*/
        (*MixerContext).Copy.unwrap_unchecked()(
            MixerLine as PVOID,
            &raw mut (*MixerLineSrc).Line as PVOID,
            core::mem::size_of::<MIXERLINEW>() as ULONG,
        );

        /* make sure it is null terminated */
        (*MixerLine).szName[(MIXER_LONG_NAME_CHARS - 1) as usize] =
            0x0000u16;
        (*MixerLine).szShortName[(MIXER_SHORT_NAME_CHARS - 1) as usize] =
            0x0000u16;
        (*MixerLine).Target.szPname[(MAXPNAMELEN - 1) as usize] =
            0x0000u16;

        return MM_STATUS_SUCCESS;
    } else if Flags == MIXER_GETLINEINFOF_COMPONENTTYPE {
        /* cast to mixer info */
        MixerInfo = MixerHandle as LPMIXER_INFO;

        /* find mixer line by component type */
        MixerLineSrc = MMixerGetSourceMixerLineByComponentType(MixerInfo, (*MixerLine).dwComponentType);
        if MixerLineSrc.is_null() {
            DPRINT1!(
                "Failed to find component type %x\n",
                (*MixerLine).dwComponentType
            );
            return MM_STATUS_UNSUCCESSFUL;
        }

        /* copy mixer line */
        (*MixerContext).Copy.unwrap_unchecked()(
            MixerLine as PVOID,
            &raw mut (*MixerLineSrc).Line as PVOID,
            core::mem::size_of::<MIXERLINEW>() as ULONG,
        );

        /* make sure it is null terminated */
        (*MixerLine).szName[(MIXER_LONG_NAME_CHARS - 1) as usize] =
            0x0000u16;
        (*MixerLine).szShortName[(MIXER_SHORT_NAME_CHARS - 1) as usize] =
            0x0000u16;
        (*MixerLine).Target.szPname[(MAXPNAMELEN - 1) as usize] =
            0x0000u16;

        /* done */
        return MM_STATUS_SUCCESS;
    } else if Flags == MIXER_GETLINEINFOF_TARGETTYPE {
        DPRINT1!("MIXER_GETLINEINFOF_TARGETTYPE handling is unimplemented\n");
    } else {
        DPRINT1!("Unknown Flags %lx handling is unimplemented\n", Flags);
    }

    return MM_STATUS_NOT_IMPLEMENTED;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetLineControls(
    mut MixerContext: PMIXER_CONTEXT,
    mut MixerHandle: HANDLE,
    mut MixerId: ULONG,
    mut Flags: ULONG,
    mut MixerLineControls: LPMIXERLINECONTROLSW,
) -> MIXER_STATUS {
    let mut MixerInfo: LPMIXER_INFO = core::ptr::null_mut();
    let mut MixerLineSrc: LPMIXERLINE_EXT = core::ptr::null_mut();
    let mut MixerControl: LPMIXERCONTROL_EXT = core::ptr::null_mut();
    let mut Status: MIXER_STATUS = core::mem::zeroed();
    let mut Entry: PLIST_ENTRY = core::ptr::null_mut();
    let mut Index: ULONG = 0;

    /* verify mixer context */
    Status = MMixerVerifyContext(MixerContext);

    if Status != MM_STATUS_SUCCESS {
        /* invalid context passed */
        return Status;
    }

    if (*MixerLineControls).cbStruct as usize != core::mem::size_of::<MIXERLINECONTROLSW>() as usize {
        DPRINT1!(
            "Invalid MixerLineControls cbStruct passed %lu expected %lu\n",
            (*MixerLineControls).cbStruct,
            core::mem::size_of::<MIXERLINECONTROLSW>()
        );
        /* invalid parameter */
        return MM_STATUS_INVALID_PARAMETER;
    }

    if (*MixerLineControls).cbmxctrl as usize != core::mem::size_of::<MIXERCONTROLW>() as usize {
        DPRINT1!(
            "Invalid MixerLineControls cbmxctrl passed %lu expected %lu\n",
            (*MixerLineControls).cbmxctrl,
            core::mem::size_of::<MIXERCONTROLW>()
        );
        /* invalid parameter */
        return MM_STATUS_INVALID_PARAMETER;
    }

    if (Flags & (MIXER_OBJECTF_MIXER | MIXER_OBJECTF_HMIXER)) == MIXER_OBJECTF_MIXER {
        /* caller passed mixer id */
        MixerHandle = MMixerGetMixerInfoByIndex(MixerContext, MixerId) as HANDLE;

        if MixerHandle.is_null() {
            /* invalid parameter */
            return MM_STATUS_INVALID_PARAMETER;
        }
    }

    Flags &= !MIXER_OBJECTF_HMIXER;

    DPRINT!("MMixerGetLineControls MixerId %lu Flags %lu\n", MixerId, Flags);

    if Flags == MIXER_GETLINECONTROLSF_ALL {
        /* cast to mixer info */
        MixerInfo = MixerHandle as LPMIXER_INFO;

        /* get mixer line */
        MixerLineSrc = MMixerGetSourceMixerLineByLineId(MixerInfo, (*MixerLineControls).dwLineID);

        if MixerLineSrc.is_null() {
            /* invalid line id */
            DPRINT!("MMixerGetLineControls Line not found %lx\n", (*MixerLineControls).dwLineID);
            return MM_STATUS_INVALID_PARAMETER;
        }

        if (*MixerLineSrc).Line.cControls != (*MixerLineControls).cControls {
            /* invalid parameter */
            DPRINT1!(
                "Invalid control count %lu expected %lu\n",
                (*MixerLineControls).cControls,
                (*MixerLineSrc).Line.cControls
            );
            return MM_STATUS_INVALID_PARAMETER;
        }

        /* copy line control(s) */
        Entry = (*MixerLineSrc).ControlsList.Flink as PLIST_ENTRY;
        Index = 0;
        while Entry != &raw mut (*MixerLineSrc).ControlsList as PLIST_ENTRY {
            /* get mixer control */
            MixerControl = CONTAINING_RECORD!(Entry, MIXERCONTROL_EXT, Entry) as LPMIXERCONTROL_EXT;

            /* copy mixer control */
            (*MixerContext).Copy.unwrap_unchecked()(
                (*MixerLineControls).pamxctrl.offset(Index as isize) as PVOID,
                &raw mut (*MixerControl).Control as PVOID,
                core::mem::size_of::<MIXERCONTROLW>() as ULONG,
            );

            /* move to next */
            Entry = (*Entry).Flink as PLIST_ENTRY;

            /* increment mixer control offset */
            Index = Index.wrapping_add(1);
        }
        return MM_STATUS_SUCCESS;
    } else if Flags == MIXER_GETLINECONTROLSF_ONEBYTYPE {
        /* cast to mixer info */
        MixerInfo = MixerHandle as LPMIXER_INFO;

        /* get mixer line */
        MixerLineSrc = MMixerGetSourceMixerLineByLineId(MixerInfo, (*MixerLineControls).dwLineID);

        if MixerLineSrc.is_null() {
            /* invalid line id */
            DPRINT1!("MMixerGetLineControls Line not found %lx\n", (*MixerLineControls).dwLineID);
            return MM_STATUS_INVALID_PARAMETER;
        }

        /* sanity checks */
        ASSERT!((*MixerLineControls).cControls == 1);
        ASSERT!((*MixerLineControls).cbmxctrl as usize == core::mem::size_of::<MIXERCONTROLW>() as usize);
        ASSERT!(!(*MixerLineControls).pamxctrl.is_null());

        Entry = (*MixerLineSrc).ControlsList.Flink as PLIST_ENTRY;
        while Entry != &raw mut (*MixerLineSrc).ControlsList as PLIST_ENTRY {
            MixerControl = CONTAINING_RECORD!(Entry, MIXERCONTROL_EXT, Entry) as LPMIXERCONTROL_EXT;
            if (*MixerLineControls).__bindgen_anon_1.dwControlType == (*MixerControl).Control.dwControlType {
                /* found a control with that type */
                (*MixerContext).Copy.unwrap_unchecked()(
                    (*MixerLineControls).pamxctrl as PVOID,
                    &raw mut (*MixerControl).Control as PVOID,
                    core::mem::size_of::<MIXERCONTROLW>() as ULONG,
                );
                return MM_STATUS_SUCCESS;
            }

            /* move to next entry */
            Entry = (*Entry).Flink as PLIST_ENTRY;
        }

        DPRINT!(
            "DeviceInfo->u.MixControls.dwControlType %x not found in Line %x cControls %u \n",
            (*MixerLineControls).dwControlType,
            (*MixerLineControls).dwLineID,
            (*MixerLineSrc).Line.cControls
        );
        return MM_STATUS_UNSUCCESSFUL;
    } else if Flags == MIXER_GETLINECONTROLSF_ONEBYID {
        /* cast to mixer info */
        MixerInfo = MixerHandle as LPMIXER_INFO;

        Status = MMixerGetMixerControlById(
            MixerInfo,
            (*MixerLineControls).__bindgen_anon_1.dwControlID,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(MixerControl),
            core::ptr::null_mut(),
        );

        if Status != MM_STATUS_SUCCESS {
            /* invalid parameter */
            DPRINT!("MMixerGetLineControls ControlID not found %lx\n", (*MixerLineControls).dwLineID);
            return MM_STATUS_INVALID_PARAMETER;
        }

        ASSERT!((*MixerLineControls).cControls == 1);
        ASSERT!((*MixerLineControls).cbmxctrl as usize == core::mem::size_of::<MIXERCONTROLW>() as usize);
        ASSERT!(!(*MixerLineControls).pamxctrl.is_null());

        DPRINT!(
            "MMixerGetLineControls ControlID %lx ControlType %lx Name %S\n",
            (*MixerControl).Control.dwControlID,
            (*MixerControl).Control.dwControlType,
            (*MixerControl).Control.szName
        );

        /* copy the controls */
        (*MixerContext).Copy.unwrap_unchecked()(
            (*MixerLineControls).pamxctrl as PVOID,
            &raw mut (*MixerControl).Control as PVOID,
            core::mem::size_of::<MIXERCONTROLW>() as ULONG,
        );
        (*(*MixerLineControls).pamxctrl).szName[(MIXER_LONG_NAME_CHARS - 1) as usize] =
            0x0000u16;
        (*(*MixerLineControls).pamxctrl).szShortName[(MIXER_SHORT_NAME_CHARS - 1) as usize] =
            0x0000u16;

        return MM_STATUS_SUCCESS;
    }
    UNIMPLEMENTED!();
    return MM_STATUS_NOT_IMPLEMENTED;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerSetControlDetails(
    mut MixerContext: PMIXER_CONTEXT,
    mut MixerHandle: HANDLE,
    mut MixerId: ULONG,
    mut Flags: ULONG,
    mut MixerControlDetails: LPMIXERCONTROLDETAILS,
) -> MIXER_STATUS {
    let mut Status: MIXER_STATUS = 0;
    let mut NodeId: ULONG = 0;
    let mut MixerInfo: LPMIXER_INFO = core::ptr::null_mut();
    let mut MixerLine: LPMIXERLINE_EXT = core::ptr::null_mut();
    let mut MixerControl: LPMIXERCONTROL_EXT = core::ptr::null_mut();

    /* verify mixer context */
    Status = MMixerVerifyContext(MixerContext);

    if Status != MM_STATUS_SUCCESS {
        /* invalid context passed */
        DPRINT1!("invalid context\n");
        return Status;
    }

    if (Flags & (MIXER_OBJECTF_MIXER | MIXER_OBJECTF_HMIXER)) == MIXER_OBJECTF_MIXER {
        /* caller passed mixer id */
        MixerHandle = MMixerGetMixerInfoByIndex(MixerContext, MixerId) as HANDLE;

        if MixerHandle.is_null() {
            /* invalid parameter */
            DPRINT1!("invalid handle\n");
            return MM_STATUS_INVALID_PARAMETER;
        }
    }

    /* get mixer info */
    MixerInfo = MixerHandle as LPMIXER_INFO;

    /* get mixer control */
    Status = MMixerGetMixerControlById(
        MixerInfo,
        (*MixerControlDetails).dwControlID,
        core::ptr::addr_of_mut!(MixerLine),
        core::ptr::addr_of_mut!(MixerControl),
        core::ptr::addr_of_mut!(NodeId),
    );

    /* check for success */
    if Status != MM_STATUS_SUCCESS {
        /* failed to find control id */
        DPRINT1!("invalid control id %lu\n", (*MixerControlDetails).dwControlID);
        return MM_STATUS_INVALID_PARAMETER;
    }

    DPRINT!(
        "MMixerSetControlDetails ControlType %lx MixerControlName %S MixerLineName %S NodeID %lu\n",
        (*MixerControl).Control.dwControlType,
        (*MixerControl).Control.szName,
        (*MixerLine).Line.szName,
        NodeId
    );
    match (*MixerControl).Control.dwControlType {
        MIXERCONTROL_CONTROLTYPE_MUTE => {
            Status = MMixerSetGetMuteControlDetails(
                MixerContext,
                MixerInfo,
                MixerControl,
                (*MixerLine).Line.dwLineID,
                MixerControlDetails,
                (TRUE) as ULONG,
            );
        }
        MIXERCONTROL_CONTROLTYPE_VOLUME => {
            Status = MMixerSetGetVolumeControlDetails(
                MixerContext,
                MixerInfo,
                NodeId,
                (TRUE) as ULONG,
                MixerControl,
                MixerControlDetails,
                MixerLine,
            );
        }
        MIXERCONTROL_CONTROLTYPE_MUX => {
            Status = MMixerSetGetMuxControlDetails(
                MixerContext,
                MixerInfo,
                NodeId,
                (TRUE) as ULONG,
                Flags,
                MixerControl,
                MixerControlDetails,
                MixerLine,
            );
        }
        _ => {
            Status = MM_STATUS_NOT_IMPLEMENTED;
        }
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetControlDetails(
    MixerContext: PMIXER_CONTEXT,
    mut MixerHandle: HANDLE,
    MixerId: ULONG,
    Flags: ULONG,
    MixerControlDetails: LPMIXERCONTROLDETAILS,
) -> MIXER_STATUS {
    let mut Status: MIXER_STATUS;
    let mut NodeId: ULONG = core::mem::zeroed();
    let mut MixerInfo: LPMIXER_INFO;
    let mut MixerLine: LPMIXERLINE_EXT = core::mem::zeroed();
    let mut MixerControl: LPMIXERCONTROL_EXT = core::mem::zeroed();

    /* verify mixer context */
    Status = MMixerVerifyContext(MixerContext);

    if Status != MM_STATUS_SUCCESS {
        /* invalid context passed */
        return Status;
    }

    if (Flags & (MIXER_OBJECTF_MIXER | MIXER_OBJECTF_HMIXER)) == MIXER_OBJECTF_MIXER {
        /* caller passed mixer id */
        MixerHandle = MMixerGetMixerInfoByIndex(MixerContext, MixerId) as HANDLE;

        if MixerHandle.is_null() {
            /* invalid parameter */
            return MM_STATUS_INVALID_PARAMETER;
        }
    }

    /* get mixer info */
    MixerInfo = MixerHandle as LPMIXER_INFO;

    /* get mixer control */
    Status = MMixerGetMixerControlById(
        MixerInfo,
        (*MixerControlDetails).dwControlID,
        core::ptr::addr_of_mut!(MixerLine),
        core::ptr::addr_of_mut!(MixerControl),
        core::ptr::addr_of_mut!(NodeId),
    );

    /* check for success */
    if Status != MM_STATUS_SUCCESS {
        /* failed to find control id */
        return MM_STATUS_INVALID_PARAMETER;
    }

    match (*MixerControl).Control.dwControlType {
        MIXERCONTROL_CONTROLTYPE_MUTE => {
            Status = MMixerSetGetMuteControlDetails(
                MixerContext,
                MixerInfo,
                MixerControl,
                (*MixerLine).Line.dwLineID,
                MixerControlDetails,
                (FALSE) as ULONG,
            );
        }
        MIXERCONTROL_CONTROLTYPE_VOLUME => {
            Status = MMixerSetGetVolumeControlDetails(
                MixerContext,
                MixerInfo,
                NodeId,
                (FALSE) as ULONG,
                MixerControl,
                MixerControlDetails,
                MixerLine,
            );
        }
        MIXERCONTROL_CONTROLTYPE_ONOFF => {
            DPRINT1!("Not Implemented MIXERCONTROL_CONTROLTYPE_ONOFF\n");
        }
        MIXERCONTROL_CONTROLTYPE_MUX => {
            Status = MMixerSetGetMuxControlDetails(
                MixerContext,
                MixerInfo,
                NodeId,
                (FALSE) as ULONG,
                Flags,
                MixerControl,
                MixerControlDetails,
                MixerLine,
            );
        }

        _ => {
            Status = MM_STATUS_NOT_IMPLEMENTED;
            DPRINT1!(
                "ControlType %lx not implemented\n",
                (*MixerControl).Control.dwControlType
            );
        }
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerPrintMixerLineControls(MixerLine: LPMIXERLINE_EXT) {
    let mut Entry: PLIST_ENTRY;
    let mut MixerControl: LPMIXERCONTROL_EXT;
    let mut Index: ULONG = 0;

    Entry = (*MixerLine).ControlsList.Flink;
    while Entry != &raw mut (*MixerLine).ControlsList {
        MixerControl = CONTAINING_RECORD!(Entry, MIXERCONTROL_EXT, Entry) as LPMIXERCONTROL_EXT;

        DPRINT1!("\n");
        DPRINT1!("Control Index: %lu\n", Index);
        DPRINT!("\n");
        DPRINT1!("cbStruct %u\n", (*MixerControl).Control.cbStruct);
        DPRINT1!("dwControlID %lu\n", (*MixerControl).Control.dwControlID);
        DPRINT1!("dwControlType %lx\n", (*MixerControl).Control.dwControlType);
        DPRINT1!("fdwControl %lu\n", (*MixerControl).Control.fdwControl);
        DPRINT1!("cMultipleItems %lu\n", (*MixerControl).Control.cMultipleItems);
        DPRINT1!("szShortName %S\n", &raw mut (*MixerControl).Control.szShortName as *mut WCHAR);
        DPRINT1!("szName %S\n", &raw mut (*MixerControl).Control.szName as *mut WCHAR);
        DPRINT1!("Bounds.dwMinimum %lu\n", (*MixerControl).Control.Bounds.__bindgen_anon_2.dwMinimum);
        DPRINT1!("Bounds.dwMaximum %lu\n", (*MixerControl).Control.Bounds.__bindgen_anon_2.dwMaximum);

        DPRINT1!("Metrics.Reserved[0] %lu\n", (*MixerControl).Control.Metrics.dwReserved[0 as usize]);
        DPRINT1!("Metrics.Reserved[1] %lu\n", (*MixerControl).Control.Metrics.dwReserved[1 as usize]);
        DPRINT1!("Metrics.Reserved[2] %lu\n", (*MixerControl).Control.Metrics.dwReserved[2 as usize]);
        DPRINT1!("Metrics.Reserved[3] %lu\n", (*MixerControl).Control.Metrics.dwReserved[3 as usize]);
        DPRINT1!("Metrics.Reserved[4] %lu\n", (*MixerControl).Control.Metrics.dwReserved[4 as usize]);
        DPRINT1!("Metrics.Reserved[5] %lu\n", (*MixerControl).Control.Metrics.dwReserved[5 as usize]);

        Entry = (*Entry).Flink;
        Index = Index.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn MMixerPrintMixers(
    mut MixerContext: PMIXER_CONTEXT,
    mut MixerList: PMIXER_LIST,
) {
    let mut Index: ULONG = 0;
    let mut SubIndex: ULONG = 0;
    let mut DestinationLineID: ULONG = 0;
    let mut SrcIndex: ULONG = 0;
    let mut MixerInfo: LPMIXER_INFO = core::ptr::null_mut();
    let mut DstMixerLine: LPMIXERLINE_EXT = core::ptr::null_mut();
    let mut SrcMixerLine: LPMIXERLINE_EXT = core::ptr::null_mut();

    DPRINT1!("MixerList %p\n", MixerList);
    DPRINT1!("MidiInCount %lu\n", (*MixerList).MidiInListCount);
    DPRINT1!("MidiOutCount %lu\n", (*MixerList).MidiOutListCount);
    DPRINT1!("WaveInCount %lu\n", (*MixerList).WaveInListCount);
    DPRINT1!("WaveOutCount %lu\n", (*MixerList).WaveOutListCount);
    DPRINT1!("MixerCount %p\n", (*MixerList).MixerListCount);

    Index = 0 as ULONG;
    while Index < (*MixerList).MixerListCount {
        /* get mixer info */
        MixerInfo = MMixerGetMixerInfoByIndex(MixerContext, Index);

        ASSERT!(!MixerInfo.is_null());
        DPRINT1!("\n");
        DPRINT1!("Name :%S\n", &raw mut (*MixerInfo).MixCaps.szPname as *mut WCHAR);
        DPRINT1!("cDestinations: %lu\n", (*MixerInfo).MixCaps.cDestinations);
        DPRINT1!("fdwSupport %lu\n", (*MixerInfo).MixCaps.fdwSupport);
        DPRINT1!("vDriverVersion %lx\n", (*MixerInfo).MixCaps.vDriverVersion);
        DPRINT1!("wMid %lx\n", (*MixerInfo).MixCaps.wMid as ::core::ffi::c_int);
        DPRINT1!("wPid %lx\n", (*MixerInfo).MixCaps.wPid as ::core::ffi::c_int);

        SubIndex = 0 as ULONG;
        while SubIndex < (*MixerInfo).MixCaps.cDestinations {
            /* calculate destination line id */
            DestinationLineID = SubIndex.wrapping_add(DESTINATION_LINE as ULONG);

            /* get destination line */
            DstMixerLine = MMixerGetSourceMixerLineByLineId(MixerInfo, DestinationLineID as DWORD);
            DPRINT1!("//----------------------------------------------------------------------------------------------\n");
            DPRINT1!("\n");
            DPRINT1!("Destination Index %lu\n", SubIndex);
            DPRINT1!("\n");
            DPRINT1!("cChannels %lu\n", (*DstMixerLine).Line.cChannels);
            DPRINT1!("cConnections %lu\n", (*DstMixerLine).Line.cConnections);
            DPRINT1!("cControls %lu\n", (*DstMixerLine).Line.cControls);
            DPRINT1!("dwComponentType %lx\n", (*DstMixerLine).Line.dwComponentType);
            DPRINT1!("dwDestination %lu\n", (*DstMixerLine).Line.dwDestination);
            DPRINT1!("dwLineID %lx\n", (*DstMixerLine).Line.dwLineID);
            DPRINT1!("dwSource %lx\n", (*DstMixerLine).Line.dwSource);
            DPRINT1!("dwUser %lu\n", (*DstMixerLine).Line.dwUser);
            DPRINT1!("fdwLine %lu\n", (*DstMixerLine).Line.fdwLine);
            DPRINT1!("szName %S\n", &raw mut (*DstMixerLine).Line.szName as *mut WCHAR);
            DPRINT1!("szShortName %S\n", &raw mut (*DstMixerLine).Line.szShortName as *mut WCHAR);
            DPRINT1!("Target.dwDeviceId %lu\n", (*DstMixerLine).Line.Target.dwDeviceID);
            DPRINT1!("Target.dwType %lu\n", (*DstMixerLine).Line.Target.dwType);
            DPRINT1!("Target.szName %S\n", &raw mut (*DstMixerLine).Line.Target.szPname as *mut WCHAR);
            DPRINT1!("Target.vDriverVersion %lx\n", (*DstMixerLine).Line.Target.vDriverVersion);
            DPRINT1!("Target.wMid %lx\n", (*DstMixerLine).Line.Target.wMid as ::core::ffi::c_int);
            DPRINT1!("Target.wPid %lx\n", (*DstMixerLine).Line.Target.wPid as ::core::ffi::c_int);
            MMixerPrintMixerLineControls(DstMixerLine);

            SrcIndex = 0 as ULONG;
            while SrcIndex < (*DstMixerLine).Line.cConnections {
                /* calculate destination line id */
                DestinationLineID = (SOURCE_LINE as ULONG)
                    .wrapping_mul(SrcIndex)
                    .wrapping_add(SubIndex);

                /* get source line */
                SrcMixerLine = MMixerGetSourceMixerLineByLineId(MixerInfo, DestinationLineID as DWORD);
                DPRINT1!("//==============================================================================================\n");
                DPRINT1!("\n");
                DPRINT1!("SrcLineIndex : %lu\n", SrcIndex);
                DPRINT1!("\n");
                DPRINT1!("cChannels %lu\n", (*SrcMixerLine).Line.cChannels);
                DPRINT1!("cConnections %lu\n", (*SrcMixerLine).Line.cConnections);
                DPRINT1!("cControls %lu\n", (*SrcMixerLine).Line.cControls);
                DPRINT1!("dwComponentType %lx\n", (*SrcMixerLine).Line.dwComponentType);
                DPRINT1!("dwDestination %lu\n", (*SrcMixerLine).Line.dwDestination);
                DPRINT1!("dwLineID %lx\n", (*SrcMixerLine).Line.dwLineID);
                DPRINT1!("dwSource %lx\n", (*SrcMixerLine).Line.dwSource);
                DPRINT1!("dwUser %lu\n", (*SrcMixerLine).Line.dwUser);
                DPRINT1!("fdwLine %lu\n", (*SrcMixerLine).Line.fdwLine);
                DPRINT1!("szName %S\n", &raw mut (*SrcMixerLine).Line.szName as *mut WCHAR);
                DPRINT1!("szShortName %S\n", &raw mut (*SrcMixerLine).Line.szShortName as *mut WCHAR);
                DPRINT1!("Target.dwDeviceId %lu\n", (*SrcMixerLine).Line.Target.dwDeviceID);
                DPRINT1!("Target.dwType %lu\n", (*SrcMixerLine).Line.Target.dwType);
                DPRINT1!("Target.szName %S\n", &raw mut (*SrcMixerLine).Line.Target.szPname as *mut WCHAR);
                DPRINT1!("Target.vDriverVersion %lx\n", (*SrcMixerLine).Line.Target.vDriverVersion);
                DPRINT1!("Target.wMid %lx\n", (*SrcMixerLine).Line.Target.wMid as ::core::ffi::c_int);
                DPRINT1!("Target.wPid %lx\n", (*SrcMixerLine).Line.Target.wPid as ::core::ffi::c_int);
                MMixerPrintMixerLineControls(SrcMixerLine);

                SrcIndex = SrcIndex.wrapping_add(1);
            }

            SubIndex = SubIndex.wrapping_add(1);
        }

        Index = Index.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn MMixerInitialize(
    MixerContext: PMIXER_CONTEXT,
    EnumFunction: PMIXER_ENUM,
    EnumContext: PVOID,
) -> MIXER_STATUS {
    let mut Status: MIXER_STATUS = 0;
    let mut hMixer: HANDLE = core::ptr::null_mut();
    let mut hKey: HANDLE = core::ptr::null_mut();
    let mut DeviceIndex: ULONG = 0;
    let mut Count: ULONG = 0;
    let mut DeviceName: LPWSTR = core::ptr::null_mut();
    let mut MixerData: LPMIXER_DATA = core::ptr::null_mut();
    let mut MixerList: PMIXER_LIST = core::ptr::null_mut();
    let mut Entry: PLIST_ENTRY = core::ptr::null_mut();

    if MixerContext.is_null() || EnumFunction.is_none() || EnumContext.is_null() {
        /* invalid parameter */
        return MM_STATUS_INVALID_PARAMETER;
    }

    if (*MixerContext).Alloc.is_none()
        || (*MixerContext).Control.is_none()
        || (*MixerContext).Free.is_none()
        || (*MixerContext).Open.is_none()
        || (*MixerContext).AllocEventData.is_none()
        || (*MixerContext).FreeEventData.is_none()
        || (*MixerContext).Close.is_none()
        || (*MixerContext).OpenKey.is_none()
        || (*MixerContext).QueryKeyValue.is_none()
        || (*MixerContext).CloseKey.is_none()
    {
        /* invalid parameter */
        return MM_STATUS_INVALID_PARAMETER;
    }

    /* allocate a mixer list */
    MixerList = (*MixerContext).Alloc.unwrap_unchecked()(
        core::mem::size_of::<MIXER_LIST>() as ULONG,
    ) as PMIXER_LIST;
    if MixerList.is_null() {
        /* no memory */
        return MM_STATUS_NO_MEMORY;
    }

    /* initialize mixer list */
    (*MixerList).MixerListCount = 0;
    (*MixerList).MixerDataCount = 0;
    (*MixerList).WaveInListCount = 0;
    (*MixerList).WaveOutListCount = 0;
    (*MixerList).MidiInListCount = 0;
    (*MixerList).MidiOutListCount = 0;
    InitializeListHead(core::ptr::addr_of_mut!((*MixerList).MixerList));
    InitializeListHead(core::ptr::addr_of_mut!((*MixerList).MixerData));
    InitializeListHead(core::ptr::addr_of_mut!((*MixerList).WaveInList));
    InitializeListHead(core::ptr::addr_of_mut!((*MixerList).WaveOutList));
    InitializeListHead(core::ptr::addr_of_mut!((*MixerList).MidiInList));
    InitializeListHead(core::ptr::addr_of_mut!((*MixerList).MidiOutList));

    /* store mixer list */
    (*MixerContext).MixerContext = MixerList as PVOID;

    /* enumerate all available devices */
    DeviceIndex = 0;
    loop {
        /* enumerate a device */
        Status = (EnumFunction.unwrap_unchecked())(
            EnumContext,
            DeviceIndex,
            core::ptr::addr_of_mut!(DeviceName),
            core::ptr::addr_of_mut!(hMixer),
            core::ptr::addr_of_mut!(hKey),
        );

        if Status != MM_STATUS_SUCCESS {
            /* check error code */
            if Status == MM_STATUS_NO_MORE_DEVICES {
                /* enumeration has finished */
                break;
            }

            DPRINT1!("EnumFunction() failed for device %lu, Status %x\n", DeviceIndex, Status);

            /* ignore error and continue */
        } else {
            /* create a mixer data entry */
            Status = MMixerCreateMixerData(MixerContext, MixerList, DeviceIndex, DeviceName, hMixer, hKey);
            if Status != MM_STATUS_SUCCESS {
                DPRINT1!("MMixerCreateMixerData() failed for device %lu, Status %x\n",
                        DeviceIndex, Status);
            }

            /* ignore error and continue */
        }

        DeviceIndex = DeviceIndex.wrapping_add(1);
    }

    /* now all filters have been pre-opened
     * lets enumerate the filters
     */
    Count = 0;
    Entry = (*MixerList).MixerData.Flink as PLIST_ENTRY;
    while Entry != core::ptr::addr_of_mut!((*MixerList).MixerData) {
        MixerData = CONTAINING_RECORD!(Entry, MIXER_DATA, Entry) as LPMIXER_DATA;
        MMixerSetupFilter(
            MixerContext,
            MixerList,
            MixerData,
            core::ptr::addr_of_mut!(Count),
        );
        Entry = (*Entry).Flink as PLIST_ENTRY;
    }

    Entry = (*MixerList).MixerData.Flink as PLIST_ENTRY;
    while Entry != core::ptr::addr_of_mut!((*MixerList).MixerData) {
        MixerData = CONTAINING_RECORD!(Entry, MIXER_DATA, Entry) as LPMIXER_DATA;

        /* now handle alternative mixer types */
        MMixerHandleAlternativeMixers(MixerContext, MixerList, MixerData, (*MixerData).Topology);
        Entry = (*Entry).Flink as PLIST_ENTRY;
    }

    //MMixerPrintMixers(MixerContext, MixerList);

    /* done */
    return MM_STATUS_SUCCESS;
}
