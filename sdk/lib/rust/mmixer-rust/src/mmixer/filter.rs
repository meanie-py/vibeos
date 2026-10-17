use crate::prelude::*;
use ntkrnl_sys::ddk::{LPGUID};
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn MMixerGetFilterPinCount(
    mut MixerContext: PMIXER_CONTEXT,
    mut hMixer: HANDLE,
) -> ULONG {
    let mut Pin: KSPROPERTY = core::mem::zeroed();
    let mut Status: MIXER_STATUS = MM_STATUS_SUCCESS;
    let mut NumPins: ULONG = 0;
    let mut BytesReturned: ULONG = 0;

    /* setup property request */
    Pin.Flags = KSPROPERTY_TYPE_GET as ULONG;
    Pin.Set = KSPROPSETID_Pin;
    Pin.Id = KSPROPERTY_PIN_CTYPES as ::core::ffi::c_int as ULONG;

    /* query pin count */
    Status = (*MixerContext).Control.unwrap_unchecked()(
        hMixer,
        IOCTL_KS_PROPERTY as ULONG,
        &raw mut Pin as PVOID,
        ::core::mem::size_of::<KSPROPERTY>() as ULONG,
        &raw mut NumPins as PVOID,
        ::core::mem::size_of::<ULONG>() as ULONG,
        &raw mut BytesReturned as PULONG,
    );

    /* check for success */
    if Status as ::core::ffi::c_uint
        != MM_STATUS_SUCCESS as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return 0 as ULONG;
    }

    return NumPins;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetFilterTopologyProperty(
    mut MixerContext: PMIXER_CONTEXT,
    mut hMixer: HANDLE,
    mut PropertyId: ULONG,
    mut OutMultipleItem: *mut PKSMULTIPLE_ITEM,
) -> MIXER_STATUS {
    let mut Property: KSPROPERTY = core::mem::zeroed();
    let mut MultipleItem: PKSMULTIPLE_ITEM = core::ptr::null_mut();
    let mut Status: MIXER_STATUS = 0;
    let mut BytesReturned: ULONG = 0;

    /* setup property request */
    Property.Id = PropertyId;
    Property.Flags = KSPROPERTY_TYPE_GET as ULONG;
    Property.Set = KSPROPSETID_Topology;

    /* query for the size */
    Status = (*MixerContext).Control.unwrap_unchecked()(
        hMixer,
        IOCTL_KS_PROPERTY as ULONG,
        core::ptr::addr_of_mut!(Property) as PVOID,
        core::mem::size_of::<KSPROPERTY>() as ULONG,
        core::ptr::null_mut(),
        0 as ULONG,
        core::ptr::addr_of_mut!(BytesReturned),
    );

    if Status != MM_STATUS_MORE_ENTRIES {
        return Status;
    }

    /* sanity check */
    ASSERT!(BytesReturned);

    /* allocate an result buffer */
    MultipleItem = (*MixerContext).Alloc.unwrap_unchecked()(BytesReturned)
        as PKSMULTIPLE_ITEM;

    if MultipleItem.is_null() {
        /* not enough memory */
        return MM_STATUS_NO_MEMORY;
    }

    /* query again with allocated buffer */
    Status = (*MixerContext).Control.unwrap_unchecked()(
        hMixer,
        IOCTL_KS_PROPERTY as ULONG,
        core::ptr::addr_of_mut!(Property) as PVOID,
        core::mem::size_of::<KSPROPERTY>() as ULONG,
        MultipleItem as PVOID,
        BytesReturned,
        core::ptr::addr_of_mut!(BytesReturned),
    );

    if Status != MM_STATUS_SUCCESS {
        /* failed */
        (*MixerContext).Free.unwrap_unchecked()(MultipleItem as PVOID);
        return Status;
    }

    /* store result */
    *OutMultipleItem = MultipleItem;

    /* done */
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetPhysicalConnection(
    mut MixerContext: PMIXER_CONTEXT,
    mut hMixer: HANDLE,
    mut PinId: ULONG,
    mut OutConnection: *mut PKSPIN_PHYSICALCONNECTION,
) -> MIXER_STATUS {
    let mut Pin: KSP_PIN = KSP_PIN {
        Property: KSIDENTIFIER {
            Set: _GUID {
                Data1: 0,
                Data2: 0,
                Data3: 0,
                Data4: [0; 8],
            },
            Id: 0,
            Flags: 0,
        },
        PinId: 0,
        Reserved: 0,
    };
    let mut Status: MIXER_STATUS;
    let mut BytesReturned: ULONG = 0;
    let mut Connection: PKSPIN_PHYSICALCONNECTION;

    /* setup the request */
    Pin.Property.Flags = KSPROPERTY_TYPE_GET as ULONG;
    Pin.Property.Id = KSPROPERTY_PIN_PHYSICALCONNECTION as ULONG;
    Pin.Property.Set = KSPROPSETID_Pin;
    Pin.PinId = PinId;

    /* query the pin for the physical connection */
    Status = (*MixerContext).Control.unwrap_unchecked()(
        hMixer,
        IOCTL_KS_PROPERTY as ULONG,
        core::ptr::addr_of_mut!(Pin) as PVOID,
        core::mem::size_of::<KSP_PIN>() as ULONG,
        core::ptr::null_mut(),
        0 as ULONG,
        core::ptr::addr_of_mut!(BytesReturned),
    );

    if Status == MM_STATUS_UNSUCCESSFUL {
        /* pin does not have a physical connection */
        return Status;
    }
    DPRINT!("Status %u BytesReturned %lu\n", Status, BytesReturned);
    Connection = (*MixerContext).Alloc.unwrap_unchecked()(BytesReturned)
        as PKSPIN_PHYSICALCONNECTION;
    if Connection.is_null() {
        /* not enough memory */
        return MM_STATUS_NO_MEMORY;
    }

    /* query the pin for the physical connection */
    Status = (*MixerContext).Control.unwrap_unchecked()(
        hMixer,
        IOCTL_KS_PROPERTY as ULONG,
        core::ptr::addr_of_mut!(Pin) as PVOID,
        core::mem::size_of::<KSP_PIN>() as ULONG,
        Connection as PVOID,
        BytesReturned,
        core::ptr::addr_of_mut!(BytesReturned),
    );
    if Status != MM_STATUS_SUCCESS {
        /* failed to query the physical connection */
        (*MixerContext).Free.unwrap_unchecked()(Connection as PVOID);
        return Status;
    }

    // store connection
    *OutConnection = Connection;
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetControlTypeFromTopologyNode(mut NodeType: LPGUID) -> ULONG {
    if IsEqualGUIDAligned(NodeType, &raw const KSNODETYPE_AGC as LPGUID) != false {
        /* automatic gain control */
        return MIXERCONTROL_CONTROLTYPE_ONOFF;
    } else if IsEqualGUIDAligned(NodeType, &raw const KSNODETYPE_LOUDNESS as LPGUID) != false {
        /* loudness control */
        return MIXERCONTROL_CONTROLTYPE_LOUDNESS;
    } else if IsEqualGUIDAligned(NodeType, &raw const KSNODETYPE_MUTE as LPGUID) != false {
        /* mute control */
        return MIXERCONTROL_CONTROLTYPE_MUTE;
    } else if IsEqualGUIDAligned(NodeType, &raw const KSNODETYPE_TONE as LPGUID) != false {
        /* tone control
         * FIXME
         * MIXERCONTROL_CONTROLTYPE_ONOFF if KSPROPERTY_AUDIO_BASS_BOOST is supported
         * MIXERCONTROL_CONTROLTYPE_BASS if KSPROPERTY_AUDIO_BASS is supported
         * MIXERCONTROL_CONTROLTYPE_TREBLE if KSPROPERTY_AUDIO_TREBLE is supported
         */
        UNIMPLEMENTED!();
        return MIXERCONTROL_CONTROLTYPE_ONOFF;
    } else if IsEqualGUIDAligned(NodeType, &raw const KSNODETYPE_VOLUME as LPGUID) != false {
        /* volume control */
        return MIXERCONTROL_CONTROLTYPE_VOLUME;
    } else if IsEqualGUIDAligned(NodeType, &raw const KSNODETYPE_PEAKMETER as LPGUID) != false {
        /* peakmeter control */
        return MIXERCONTROL_CONTROLTYPE_PEAKMETER;
    } else if IsEqualGUIDAligned(NodeType, &raw const KSNODETYPE_MUX as LPGUID) != false {
        /* mux control */
        return MIXERCONTROL_CONTROLTYPE_MUX;
    } else if IsEqualGUIDAligned(NodeType, &raw const KSNODETYPE_STEREO_WIDE as LPGUID) != false {
        /* stero wide control */
        return MIXERCONTROL_CONTROLTYPE_FADER;
    } else if IsEqualGUIDAligned(NodeType, &raw const KSNODETYPE_CHORUS as LPGUID) != false {
        /* chorus control */
        return MIXERCONTROL_CONTROLTYPE_FADER;
    } else if IsEqualGUIDAligned(NodeType, &raw const KSNODETYPE_REVERB as LPGUID) != false {
        /* reverb control */
        return MIXERCONTROL_CONTROLTYPE_FADER;
    } else if IsEqualGUIDAligned(NodeType, &raw const KSNODETYPE_SUPERMIX as LPGUID) != false {
        /* supermix control
         * MIXERCONTROL_CONTROLTYPE_MUTE if KSPROPERTY_AUDIO_MUTE is supported
         */
        UNIMPLEMENTED!();
        return MIXERCONTROL_CONTROLTYPE_VOLUME;
    }
    /* TODO
     * check for other supported node types
     */
    //UNIMPLEMENTED;
    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerSetGetControlDetails(
    MixerContext: PMIXER_CONTEXT,
    hMixer: HANDLE,
    NodeId: ULONG,
    bSet: ULONG,
    PropertyId: ULONG,
    Channel: ULONG,
    InputValue: PLONG,
) -> MIXER_STATUS {
    let mut Property: KSNODEPROPERTY_AUDIO_CHANNEL = core::mem::zeroed();
    let mut Status: MIXER_STATUS;
    let mut Value: LONG = 0;
    let mut BytesReturned: ULONG = 0;

    if bSet != 0 {
        Value = *InputValue;
    }

    /* setup the request */
    core::ptr::write_bytes(
        core::ptr::addr_of_mut!(Property) as *mut u8,
        0,
        core::mem::size_of::<KSNODEPROPERTY_AUDIO_CHANNEL>(),
    );

    Property.NodeProperty.NodeId = NodeId;
    Property.NodeProperty.Property.Id = PropertyId;
    Property.NodeProperty.Property.Flags = KSPROPERTY_TYPE_TOPOLOGY as ULONG;
    Property.NodeProperty.Property.Set = KSPROPSETID_Audio;
    Property.Channel = Channel as LONG;
    Property.Reserved = 0 as ULONG;

    if bSet != 0 {
        Property.NodeProperty.Property.Flags |= KSPROPERTY_TYPE_SET as ULONG;
    } else {
        Property.NodeProperty.Property.Flags |= KSPROPERTY_TYPE_GET as ULONG;
    }

    /* send the request */
    Status = (*MixerContext).Control.unwrap_unchecked()(
        hMixer,
        IOCTL_KS_PROPERTY as ULONG,
        core::ptr::addr_of_mut!(Property) as PVOID,
        core::mem::size_of::<KSNODEPROPERTY_AUDIO_CHANNEL>() as ULONG,
        core::ptr::addr_of_mut!(Value) as PVOID,
        core::mem::size_of::<LONG>() as ULONG,
        core::ptr::addr_of_mut!(BytesReturned),
    );

    if bSet == 0 && Status == MM_STATUS_SUCCESS as MIXER_STATUS {
        *InputValue = Value;
    }

    DPRINT!(
        "Status %x bSet %u NodeId %u Value %d PropertyId %u\n",
        Status,
        bSet,
        NodeId,
        Value,
        PropertyId
    );
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetPinInstanceCount(
    mut MixerContext: PMIXER_CONTEXT,
    mut hFilter: HANDLE,
    mut PinId: ULONG,
) -> ULONG {
    let mut PinRequest: KSP_PIN = KSP_PIN {
        Property: KSIDENTIFIER {
            Set: _GUID {
                Data1: 0,
                Data2: 0,
                Data3: 0,
                Data4: [0; 8],
            },
            Id: 0,
            Flags: 0,
        },
        PinId: 0,
        Reserved: 0,
    };
    let mut PinInstances: KSPIN_CINSTANCES = KSPIN_CINSTANCES {
        PossibleCount: 0,
        CurrentCount: 0,
    };
    let mut BytesReturned: ULONG = 0;
    let mut Status: MIXER_STATUS = MM_STATUS_SUCCESS;
    PinRequest.Reserved = 0;
    PinRequest.PinId = PinId;
    PinRequest.Property.Set = KSPROPSETID_Pin;
    PinRequest.Property.Flags = KSPROPERTY_TYPE_GET;
    PinRequest.Property.Id = KSPROPERTY_PIN_CINSTANCES;
    Status = (*MixerContext).Control.unwrap_unchecked()(
        hFilter,
        IOCTL_KS_PROPERTY,
        &raw mut PinRequest as PVOID,
        ::core::mem::size_of::<KSP_PIN>() as ULONG,
        &raw mut PinInstances as PVOID,
        ::core::mem::size_of::<KSPIN_CINSTANCES>() as ULONG,
        &raw mut BytesReturned,
    );
    ASSERT!(Status == MM_STATUS_SUCCESS);
    return PinInstances.CurrentCount;
}
