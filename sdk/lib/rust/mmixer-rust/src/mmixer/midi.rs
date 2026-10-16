use crate::prelude::*;
use ntkrnl_sys::ddk::{ACCESS_MASK, GUID, LPWSTR, PHANDLE};
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn MMixerGetPinDataFlowAndCommunication(
    mut MixerContext: PMIXER_CONTEXT,
    mut hDevice: HANDLE,
    mut PinId: ULONG,
    mut DataFlow: PKSPIN_DATAFLOW,
    mut Communication: PKSPIN_COMMUNICATION,
) -> MIXER_STATUS {
    let mut Pin: KSP_PIN = core::mem::zeroed();
    let mut BytesReturned: ULONG = 0;
    let mut Status: MIXER_STATUS = 0;

    /* setup request */
    Pin.PinId = PinId;
    Pin.Reserved = 0 as ULONG;
    Pin.Property.Flags = KSPROPERTY_TYPE_GET as ULONG;
    Pin.Property.Id = KSPROPERTY_PIN_DATAFLOW as ULONG;
    Pin.Property.Set = KSPROPSETID_Pin;

    /* get pin dataflow */
    Status = (*MixerContext).Control.unwrap_unchecked()(
        hDevice,
        IOCTL_KS_PROPERTY as ULONG,
        core::ptr::addr_of_mut!(Pin) as PVOID,
        core::mem::size_of::<KSP_PIN>() as ULONG,
        DataFlow as PVOID,
        core::mem::size_of::<KSPIN_DATAFLOW>() as ULONG,
        core::ptr::addr_of_mut!(BytesReturned),
    );
    if Status != MM_STATUS_SUCCESS as MIXER_STATUS {
        /* failed to retrieve dataflow */
        return Status;
    }

    /* setup communication request */
    Pin.Property.Id = KSPROPERTY_PIN_COMMUNICATION as ULONG;

    /* get pin communication */
    Status = (*MixerContext).Control.unwrap_unchecked()(
        hDevice,
        IOCTL_KS_PROPERTY as ULONG,
        core::ptr::addr_of_mut!(Pin) as PVOID,
        core::mem::size_of::<KSP_PIN>() as ULONG,
        Communication as PVOID,
        core::mem::size_of::<KSPIN_COMMUNICATION>() as ULONG,
        core::ptr::addr_of_mut!(BytesReturned),
    );

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerAddMidiPin(
    mut MixerContext: PMIXER_CONTEXT,
    mut MixerList: PMIXER_LIST,
    mut DeviceId: ULONG,
    mut PinId: ULONG,
    mut bInput: ULONG,
    mut DeviceName: LPWSTR,
) -> MIXER_STATUS {
    let mut MidiInfo: LPMIDI_INFO = ::core::ptr::null_mut::<MIDI_INFO>();

    /* allocate midi info */
    MidiInfo = (*MixerContext).Alloc.unwrap_unchecked()(
        core::mem::size_of::<MIDI_INFO>() as ULONG,
    ) as LPMIDI_INFO;

    if MidiInfo.is_null() {
        /* no memory */
        return MM_STATUS_NO_MEMORY;
    }

    /* initialize midi info */
    (*MidiInfo).DeviceId = DeviceId;
    (*MidiInfo).PinId = PinId;

    /* sanity check */
    ASSERT!(
        DeviceName.is_null()
            || (wcslen(DeviceName as *const wchar_t)
                < (MAXPNAMELEN as ::core::ffi::c_ulonglong) as _)
    );

    /* copy device name */
    if bInput != 0 && !DeviceName.is_null() {
        wcscpy(
            &raw mut (*MidiInfo).u.InCaps.szPname as *mut wchar_t,
            DeviceName as *const wchar_t,
        );
    } else if bInput == 0 && !DeviceName.is_null() {
        wcscpy(
            &raw mut (*MidiInfo).u.OutCaps.szPname as *mut wchar_t,
            DeviceName as *const wchar_t,
        );
    }

    /* FIXME determine manufacturer / product id */
    if bInput != 0 {
        (*MidiInfo).u.InCaps.dwSupport = 0 as DWORD;
        (*MidiInfo).u.InCaps.wMid = MM_MICROSOFT as WORD;
        (*MidiInfo).u.InCaps.wPid = MM_PID_UNMAPPED as WORD;
        (*MidiInfo).u.InCaps.vDriverVersion = 1 as MMVERSION;
    } else {
        (*MidiInfo).u.OutCaps.dwSupport = 0 as DWORD;
        (*MidiInfo).u.OutCaps.wMid = MM_MICROSOFT as WORD;
        (*MidiInfo).u.OutCaps.wPid = MM_PID_UNMAPPED as WORD;
        (*MidiInfo).u.OutCaps.vDriverVersion = 1 as MMVERSION;
    }

    if bInput != 0 {
        /* insert into list */
        InsertTailList(
            &raw mut (*MixerList).MidiInList,
            &raw mut (*MidiInfo).Entry,
        );
        (*MixerList).MidiInListCount = (*MixerList).MidiInListCount.wrapping_add(1);
    } else {
        /* insert into list */
        InsertTailList(
            &raw mut (*MixerList).MidiOutList,
            &raw mut (*MidiInfo).Entry,
        );
        (*MixerList).MidiOutListCount = (*MixerList).MidiOutListCount.wrapping_add(1);
    }

    return MM_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerCheckFilterPinMidiSupport(
    MixerContext: PMIXER_CONTEXT,
    MixerList: PMIXER_LIST,
    MixerData: LPMIXER_DATA,
    PinId: ULONG,
    MultipleItem: PKSMULTIPLE_ITEM,
    szPname: LPWSTR,
) {
    let mut Index: ULONG;
    let mut DataRange: PKSDATARANGE;
    let mut Communication: KSPIN_COMMUNICATION = 0 as KSPIN_COMMUNICATION;
    let mut DataFlow: KSPIN_DATAFLOW = 0 as KSPIN_DATAFLOW;

    /* get first datarange */
    DataRange = MultipleItem.offset(1 as isize) as PKSDATARANGE;

    /* alignment assert */
    ASSERT!(((DataRange as ULONG_PTR) & 0x7) == 0);

    /* iterate through all data ranges */
    Index = 0;
    while Index < (*MultipleItem).Count {
        if IsEqualGUIDAligned(
            core::ptr::addr_of_mut!((*DataRange).MajorFormat),
            core::ptr::addr_of!(KSDATAFORMAT_TYPE_MUSIC),
        ) && IsEqualGUIDAligned(
            core::ptr::addr_of_mut!((*DataRange).SubFormat),
            core::ptr::addr_of!(KSDATAFORMAT_SUBTYPE_MIDI),
        ) && IsEqualGUIDAligned(
            core::ptr::addr_of_mut!((*DataRange).Specifier),
            core::ptr::addr_of!(KSDATAFORMAT_SPECIFIER_NONE),
        ) {
            /* pin supports midi datarange */
            if MMixerGetPinDataFlowAndCommunication(
                MixerContext,
                (*MixerData).hDevice,
                PinId,
                core::ptr::addr_of_mut!(DataFlow),
                core::ptr::addr_of_mut!(Communication),
            ) == MM_STATUS_SUCCESS
            {
                if DataFlow == KSPIN_DATAFLOW_IN && Communication == KSPIN_COMMUNICATION_SINK {
                    MMixerAddMidiPin(
                        MixerContext,
                        MixerList,
                        (*MixerData).DeviceId,
                        PinId,
                        (FALSE) as ULONG,
                        szPname,
                    );
                } else if DataFlow == KSPIN_DATAFLOW_OUT
                    && Communication == KSPIN_COMMUNICATION_SOURCE
                {
                    MMixerAddMidiPin(
                        MixerContext,
                        MixerList,
                        (*MixerData).DeviceId,
                        PinId,
                        (TRUE) as ULONG,
                        szPname,
                    );
                }
            }
        }

        /* move to next datarange */
        DataRange = ((DataRange as ULONG_PTR).wrapping_add((*DataRange).FormatSize as ULONG_PTR))
            as PKSDATARANGE;

        /* alignment assert */
        ASSERT!(((DataRange as ULONG_PTR) & 0x7) == 0);

        /* data ranges are 64-bit aligned */
        DataRange = (((DataRange as ULONG_PTR).wrapping_add(0x7))
            & !(0x7)) as PVOID as PKSDATARANGE;

        Index = Index.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn MMixerInitializeMidiForFilter(
    MixerContext: PMIXER_CONTEXT,
    MixerList: PMIXER_LIST,
    MixerData: LPMIXER_DATA,
    Topology: PTOPOLOGY,
) {
    let mut PinCount: ULONG = 0;
    let mut Index: ULONG;
    let mut Status: MIXER_STATUS;
    let mut MultipleItem: PKSMULTIPLE_ITEM = core::ptr::null_mut();
    let mut szPname: [WCHAR; (MAXPNAMELEN as _)] = [0; (MAXPNAMELEN as _)];

    /* get filter pin count */
    MMixerGetTopologyPinCount(Topology, core::ptr::addr_of_mut!(PinCount));

    /* get mixer name */
    if MMixerGetDeviceName(
        MixerContext,
        core::ptr::addr_of_mut!(szPname) as LPWSTR,
        (*MixerData).hDeviceInterfaceKey,
    ) != MM_STATUS_SUCCESS
    {
        /* clear name */
        szPname[0] = 0;
    }

    /* iterate all pins and check for KSDATARANGE_MUSIC support */
    Index = 0;
    while Index < PinCount {
        /* get audio pin data ranges */
        Status = MMixerGetAudioPinDataRanges(
            MixerContext,
            (*MixerData).hDevice,
            Index,
            core::ptr::addr_of_mut!(MultipleItem),
        );

        /* check for success */
        if Status == MM_STATUS_SUCCESS {
            /* check if there is support KSDATARANGE_MUSIC */
            MMixerCheckFilterPinMidiSupport(
                MixerContext,
                MixerList,
                MixerData,
                Index,
                MultipleItem,
                core::ptr::addr_of_mut!(szPname) as LPWSTR,
            );
        }
        Index = Index.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn MMixerOpenMidiPin(
    mut MixerContext: PMIXER_CONTEXT,
    mut MixerList: PMIXER_LIST,
    mut DeviceId: ULONG,
    mut PinId: ULONG,
    mut DesiredAccess: ACCESS_MASK,
    mut CreateCallback: PIN_CREATE_CALLBACK,
    mut Context: PVOID,
    mut PinHandle: PHANDLE,
) -> MIXER_STATUS {
    let mut PinConnect: PKSPIN_CONNECT = core::ptr::null_mut();
    let mut DataFormat: PKSDATAFORMAT = core::ptr::null_mut();
    let mut MixerData: LPMIXER_DATA = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;
    let mut MixerStatus: MIXER_STATUS = 0;

    MixerData = MMixerGetDataByDeviceId(MixerList, DeviceId);
    if MixerData.is_null() {
        return MM_STATUS_INVALID_PARAMETER;
    }

    /* allocate pin connect */
    PinConnect = MMixerAllocatePinConnect(
        MixerContext,
        core::mem::size_of::<KSDATAFORMAT>() as ULONG,
    );
    if PinConnect.is_null() {
        /* no memory */
        return MM_STATUS_NO_MEMORY;
    }

    /* initialize pin connect struct */
    MMixerInitializePinConnect(PinConnect, PinId);

    /* get offset to dataformat */
    DataFormat = PinConnect.offset(1 as isize) as PKSDATAFORMAT;

    /* initialize data format */
    core::ptr::copy_nonoverlapping(
        &raw const KSDATAFORMAT_TYPE_MUSIC as *const u8,
        &raw mut (*DataFormat).MajorFormat as *mut u8,
        core::mem::size_of::<GUID>(),
    );
    core::ptr::copy_nonoverlapping(
        &raw const KSDATAFORMAT_SUBTYPE_MIDI as *const u8,
        &raw mut (*DataFormat).SubFormat as *mut u8,
        core::mem::size_of::<GUID>(),
    );
    core::ptr::copy_nonoverlapping(
        &raw const KSDATAFORMAT_SPECIFIER_NONE as *const u8,
        &raw mut (*DataFormat).Specifier as *mut u8,
        core::mem::size_of::<GUID>(),
    );

    if CreateCallback.is_some() {
        /* let the callback handle the creation */
        MixerStatus = (CreateCallback.unwrap_unchecked())(
            Context,
            DeviceId,
            PinId,
            (*MixerData).hDevice,
            PinConnect,
            DesiredAccess,
            PinHandle,
        );
    } else {
        /* now create the pin */
        Status = KsCreatePin((*MixerData).hDevice, PinConnect, DesiredAccess, PinHandle);

        /* normalize status */
        if Status == STATUS_SUCCESS {
            MixerStatus = MM_STATUS_SUCCESS;
        } else {
            MixerStatus = MM_STATUS_UNSUCCESSFUL;
        }
    }

    /* free create info */
    (*MixerContext).Free.unwrap_unchecked()(PinConnect as PVOID);

    /* done */
    return MixerStatus;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetMidiInfoByIndexAndType(
    mut MixerList: PMIXER_LIST,
    mut DeviceIndex: ULONG,
    mut bMidiInputType: ULONG,
    mut OutMidiInfo: *mut LPMIDI_INFO,
) -> MIXER_STATUS {
    let mut Index: ULONG = 0;
    let mut Entry: PLIST_ENTRY = core::ptr::null_mut();
    let mut ListHead: PLIST_ENTRY = core::ptr::null_mut();
    let mut MidiInfo: LPMIDI_INFO = core::ptr::null_mut();
    if bMidiInputType != 0 {
        ListHead = &raw mut (*MixerList).MidiInList as PLIST_ENTRY;
    } else {
        ListHead = &raw mut (*MixerList).MidiOutList as PLIST_ENTRY;
    }
    /* get first entry */
    Entry = (*ListHead).Flink as PLIST_ENTRY;
    while Entry != ListHead {
        MidiInfo = (Entry as ULONG_PTR)
            .wrapping_sub(core::mem::offset_of!(MIDI_INFO, Entry))
            as LPMIDI_INFO;
        if Index == DeviceIndex {
            *OutMidiInfo = MidiInfo;
            return MM_STATUS_SUCCESS;
        }
        Index = Index.wrapping_add(1);
        Entry = (*Entry).Flink as PLIST_ENTRY;
    }
    return MM_STATUS_INVALID_PARAMETER;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerMidiOutCapabilities(
    mut MixerContext: PMIXER_CONTEXT,
    mut DeviceIndex: ULONG,
    mut Caps: LPMIDIOUTCAPSW,
) -> MIXER_STATUS {
    let mut MixerList: PMIXER_LIST = core::ptr::null_mut();
    let mut Status: MIXER_STATUS;
    let mut MidiInfo: LPMIDI_INFO = core::ptr::null_mut();

    /* verify mixer context */
    Status = MMixerVerifyContext(MixerContext);

    if Status != MM_STATUS_SUCCESS {
        /* invalid context passed */
        return Status;
    }

    /* grab mixer list */
    MixerList = (*MixerContext).MixerContext as PMIXER_LIST;

    /* find destination midi */
    Status = MMixerGetMidiInfoByIndexAndType(MixerList, DeviceIndex, (FALSE) as ULONG, &raw mut MidiInfo);
    if Status != MM_STATUS_SUCCESS {
        /* failed to find midi info */
        return MM_STATUS_UNSUCCESSFUL;
    }

    /* copy capabilities */
    (*MixerContext).Copy_0.unwrap_unchecked()(
        Caps as PVOID,
        core::ptr::addr_of_mut!((*MidiInfo).u.OutCaps) as PVOID,
        core::mem::size_of::<MIDIOUTCAPSW>() as ULONG,
    );

    return MM_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerMidiInCapabilities(
    MixerContext: PMIXER_CONTEXT,
    DeviceIndex: ULONG,
    Caps: LPMIDIINCAPSW,
) -> MIXER_STATUS {
    let mut MixerList: PMIXER_LIST = core::ptr::null_mut();
    let mut Status: MIXER_STATUS;
    let mut MidiInfo: LPMIDI_INFO = core::ptr::null_mut();

    /* verify mixer context */
    Status = MMixerVerifyContext(MixerContext);

    if Status != MM_STATUS_SUCCESS {
        /* invalid context passed */
        return Status;
    }

    /* grab mixer list */
    MixerList = (*MixerContext).MixerContext as PMIXER_LIST;

    /* find destination midi */
    Status = MMixerGetMidiInfoByIndexAndType(
        MixerList,
        DeviceIndex,
        (TRUE) as ULONG,
        core::ptr::addr_of_mut!(MidiInfo),
    );
    if Status != MM_STATUS_SUCCESS {
        /* failed to find midi info */
        return MM_STATUS_UNSUCCESSFUL;
    }

    /* copy capabilities */
    (((*MixerContext).Copy).unwrap_unchecked())(
        Caps as PVOID,
        core::ptr::addr_of_mut!((*MidiInfo).u.InCaps) as PVOID,
        core::mem::size_of::<MIDIINCAPSW>() as ULONG,
    );

    return MM_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetMidiDevicePath(
    MixerContext: PMIXER_CONTEXT,
    bMidiIn: ULONG,
    DeviceId: ULONG,
    DevicePath: *mut LPWSTR,
) -> MIXER_STATUS {
    let mut MixerList: PMIXER_LIST = core::ptr::null_mut();
    let mut MixerData: LPMIXER_DATA = core::ptr::null_mut();
    let mut MidiInfo: LPMIDI_INFO = core::ptr::null_mut();
    let mut Length: SIZE_T = 0;
    let mut Status: MIXER_STATUS;

    /* verify mixer context */
    Status = MMixerVerifyContext(MixerContext);

    if Status != MM_STATUS_SUCCESS {
        /* invalid context passed */
        return Status;
    }

    /* grab mixer list */
    MixerList = (*MixerContext).MixerContext as PMIXER_LIST;

    /* find destination midi */
    Status = MMixerGetMidiInfoByIndexAndType(
        MixerList,
        DeviceId,
        bMidiIn,
        core::ptr::addr_of_mut!(MidiInfo),
    );
    if Status != MM_STATUS_SUCCESS {
        /* failed to find midi info */
        return MM_STATUS_INVALID_PARAMETER;
    }

    /* get associated device id */
    MixerData = MMixerGetDataByDeviceId(MixerList, (*MidiInfo).DeviceId);
    if MixerData.is_null() {
        return MM_STATUS_INVALID_PARAMETER;
    }

    /* calculate length */
    Length = wcslen((*MixerData).DeviceName) as SIZE_T + 1;

    /* allocate destination buffer */
    *DevicePath = (*MixerContext).Alloc.unwrap_unchecked()(
        (Length as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as ULONG,
    ) as LPWSTR;

    if (*DevicePath).is_null() {
        /* no memory */
        return MM_STATUS_NO_MEMORY;
    }

    /* copy device path */
    (*MixerContext).Copy_0.unwrap_unchecked()(
        *DevicePath as PVOID,
        (*MixerData).DeviceName as PVOID,
        (Length as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as ULONG,
    );

    /* done */
    return MM_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerSetMidiStatus(
    mut MixerContext: PMIXER_CONTEXT,
    mut PinHandle: HANDLE,
    mut State: KSSTATE,
) -> MIXER_STATUS {
    let mut Property: KSPROPERTY = core::mem::zeroed();
    let mut Length: ULONG = 0;

    /* setup property request */
    Property.Set = KSPROPSETID_Connection;
    Property.Id = KSPROPERTY_CONNECTION_STATE as ::core::ffi::c_int as ULONG;
    Property.Flags = KSPROPERTY_TYPE_SET as ULONG;

    return (*MixerContext).Control.unwrap_unchecked()(
        PinHandle,
        IOCTL_KS_PROPERTY as ULONG,
        &raw mut Property as PVOID,
        core::mem::size_of::<KSPROPERTY>() as ULONG,
        &raw mut State as PVOID,
        core::mem::size_of::<KSSTATE>() as ULONG,
        &raw mut Length,
    );
}
#[no_mangle]
pub unsafe extern "C" fn MMixerOpenMidi(
    mut MixerContext: PMIXER_CONTEXT,
    mut DeviceIndex: ULONG,
    mut bMidiIn: ULONG,
    mut CreateCallback: PIN_CREATE_CALLBACK,
    mut Context: PVOID,
    mut PinHandle: PHANDLE,
) -> MIXER_STATUS {
    let mut MixerList: PMIXER_LIST = core::ptr::null_mut();
    let mut Status: MIXER_STATUS = core::mem::zeroed();
    let mut MidiInfo: LPMIDI_INFO = core::ptr::null_mut();
    let mut DesiredAccess: ACCESS_MASK = 0;

    /* verify mixer context */
    Status = MMixerVerifyContext(MixerContext);

    if Status != MM_STATUS_SUCCESS {
        /* invalid context passed */
        return Status;
    }

    /* grab mixer list */
    MixerList = (*MixerContext).MixerContext as PMIXER_LIST;

    /* find destination midi */
    Status = MMixerGetMidiInfoByIndexAndType(MixerList, DeviceIndex, bMidiIn, &raw mut MidiInfo);
    if Status != MM_STATUS_SUCCESS {
        /* failed to find midi info */
        return MM_STATUS_INVALID_PARAMETER;
    }

    /* get desired access */
    if bMidiIn != 0 {
        DesiredAccess |= GENERIC_READ;
    } else {
        DesiredAccess |= GENERIC_WRITE;
    }

    /* now try open the pin */
    return MMixerOpenMidiPin(
        MixerContext,
        MixerList,
        (*MidiInfo).DeviceId,
        (*MidiInfo).PinId,
        DesiredAccess,
        CreateCallback,
        Context,
        PinHandle,
    );
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetMidiInCount(mut MixerContext: PMIXER_CONTEXT) -> ULONG {
    let mut MixerList: PMIXER_LIST = core::ptr::null_mut();
    let mut Status: MIXER_STATUS;

    /* verify mixer context */
    Status = MMixerVerifyContext(MixerContext);

    if Status != MM_STATUS_SUCCESS {
        /* invalid context passed */
        return Status as ULONG;
    }

    /* grab mixer list */
    MixerList = (*MixerContext).MixerContext as PMIXER_LIST;

    return (*MixerList).MidiInListCount;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetMidiOutCount(mut MixerContext: PMIXER_CONTEXT) -> ULONG {
    let mut MixerList: PMIXER_LIST = core::ptr::null_mut();
    let mut Status: MIXER_STATUS;

    /* verify mixer context */
    Status = MMixerVerifyContext(MixerContext);

    if Status != MM_STATUS_SUCCESS {
        /* invalid context passed */
        return Status as ULONG;
    }

    /* grab mixer list */
    MixerList = (*MixerContext).MixerContext as PMIXER_LIST;

    return (*MixerList).MidiOutListCount;
}
