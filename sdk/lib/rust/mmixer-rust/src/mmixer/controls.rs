// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_F8619DE16B4F: [u16; 8] = [0x004Eu16, 0x006F, 0x0020, 0x004E, 0x0061, 0x006D, 0x0065, 0x0000];

use crate::prelude::*;
use ntkrnl_sys::ddk::{GUID, LPGUID, LPWSTR};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut KSNODETYPE_DESKTOP_MICROPHONE: GUID = _GUID {
    Data1: 0xdff21be2 as ::core::ffi::c_uint,
    Data2: 0xf70f as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xb9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x17 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x22 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x31 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSNODETYPE_LEGACY_AUDIO_CONNECTOR: GUID = _GUID {
    Data1: 0xdff21fe4 as ::core::ffi::c_uint,
    Data2: 0xf70f as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xb9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x17 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x22 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x31 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSNODETYPE_TELEPHONE: GUID = _GUID {
    Data1: 0xdff21ee2 as ::core::ffi::c_uint,
    Data2: 0xf70f as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xb9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x17 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x22 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x31 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSNODETYPE_PHONE_LINE: GUID = _GUID {
    Data1: 0xdff21ee1 as ::core::ffi::c_uint,
    Data2: 0xf70f as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xb9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x17 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x22 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x31 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSNODETYPE_DOWN_LINE_PHONE: GUID = _GUID {
    Data1: 0xdff21ee3 as ::core::ffi::c_uint,
    Data2: 0xf70f as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xb9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x17 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x22 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x31 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSNODETYPE_DESKTOP_SPEAKER: GUID = _GUID {
    Data1: 0xdff21ce4 as ::core::ffi::c_uint,
    Data2: 0xf70f as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xb9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x17 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x22 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x31 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSNODETYPE_ROOM_SPEAKER: GUID = _GUID {
    Data1: 0xdff21ce5 as ::core::ffi::c_uint,
    Data2: 0xf70f as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xb9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x17 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x22 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x31 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSNODETYPE_COMMUNICATION_SPEAKER: GUID = _GUID {
    Data1: 0xdff21ce6 as ::core::ffi::c_uint,
    Data2: 0xf70f as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xb9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x17 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x22 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x31 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSNODETYPE_HEADPHONES: GUID = _GUID {
    Data1: 0xdff21ce2 as ::core::ffi::c_uint,
    Data2: 0xf70f as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xb9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x17 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x22 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x31 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSNODETYPE_HEAD_MOUNTED_DISPLAY_AUDIO: GUID = _GUID {
    Data1: 0xdff21ce3 as ::core::ffi::c_uint,
    Data2: 0xf70f as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xb9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x17 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x22 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x31 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSNODETYPE_MICROPHONE: GUID = _GUID {
    Data1: 0xdff21be1 as ::core::ffi::c_uint,
    Data2: 0xf70f as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xb9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x17 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x22 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x31 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSCATEGORY_AUDIO: GUID = _GUID {
    Data1: 0x6994ad04 as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0x93ef as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xa3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xcc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x22 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x31 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSNODETYPE_SPDIF_INTERFACE: GUID = _GUID {
    Data1: 0xdff21fe5 as ::core::ffi::c_uint,
    Data2: 0xf70f as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xb9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x17 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x22 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x31 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSNODETYPE_ANALOG_CONNECTOR: GUID = _GUID {
    Data1: 0xdff21fe1 as ::core::ffi::c_uint,
    Data2: 0xf70f as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xb9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x17 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x22 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x31 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSNODETYPE_SPEAKER: GUID = _GUID {
    Data1: 0xdff21ce1 as ::core::ffi::c_uint,
    Data2: 0xf70f as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xb9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x17 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x22 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x31 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSNODETYPE_CD_PLAYER: GUID = _GUID {
    Data1: 0xdff220e3 as ::core::ffi::c_uint,
    Data2: 0xf70f as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xb9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x17 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x22 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x31 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSNODETYPE_SYNTHESIZER: GUID = _GUID {
    Data1: 0xdff220f3 as ::core::ffi::c_uint,
    Data2: 0xf70f as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xb9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x17 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x22 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x31 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KSNODETYPE_LINE_CONNECTOR: GUID = _GUID {
    Data1: 0xdff21fe3 as ::core::ffi::c_uint,
    Data2: 0xf70f as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xb9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x17 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x22 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x31 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut PINNAME_VIDEO_CAPTURE: GUID = _GUID {
    Data1: 0xfb6c4281 as ::core::ffi::c_uint,
    Data2: 0x353 as ::core::ffi::c_ushort,
    Data3: 0x11d1 as ::core::ffi::c_ushort,
    Data4: [
        0x90 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xcc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x16 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xba as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};
#[no_mangle]
pub unsafe extern "C" fn MMixerAddMixerControl(
    MixerContext: PMIXER_CONTEXT,
    MixerInfo: LPMIXER_INFO,
    hMixer: HANDLE,
    Topology: PTOPOLOGY,
    NodeIndex: ULONG,
    MixerLine: LPMIXERLINE_EXT,
    MaxChannels: ULONG,
) -> MIXER_STATUS {
    let mut NodeType: LPGUID = core::ptr::null_mut();
    let mut Node: KSP_NODE = core::mem::zeroed();
    let mut BytesReturned: ULONG = 0;
    let mut Status: MIXER_STATUS = 0;
    let mut Name: LPWSTR = core::ptr::null_mut();
    let mut MixerControl: LPMIXERCONTROL_EXT = core::ptr::null_mut();

    /* allocate mixer control */
    MixerControl = (*MixerContext).Alloc.unwrap_unchecked()(
        core::mem::size_of::<MIXERCONTROL_EXT>() as ULONG,
    ) as LPMIXERCONTROL_EXT;
    if MixerControl.is_null() {
        /* no memory */
        return (MM_STATUS_NO_MEMORY) as MIXER_STATUS;
    }

    /* initialize mixer control */
    (*MixerControl).hDevice = hMixer;
    (*MixerControl).NodeID = NodeIndex;
    (*MixerControl).ExtraData = core::ptr::null_mut();

    (*MixerControl).Control.cbStruct =
        core::mem::size_of::<MIXERCONTROLW>() as DWORD;
    (*MixerControl).Control.dwControlID = (*MixerInfo).ControlId as DWORD;

    /* get node type */
    NodeType = MMixerGetNodeTypeFromTopology(Topology, NodeIndex);
    /* store control type */
    (*MixerControl).Control.dwControlType =
        MMixerGetControlTypeFromTopologyNode(NodeType) as DWORD;

    (*MixerControl).Control.fdwControl = if MaxChannels > (1) as ULONG {
        (0) as DWORD
    } else {
        (MIXERCONTROL_CONTROLF_UNIFORM) as DWORD
    };
    (*MixerControl).Control.cMultipleItems = (0) as DWORD;

    /* setup request to retrieve name */
    Node.NodeId = NodeIndex;
    Node.Property.Id = (KSPROPERTY_TOPOLOGY_NAME) as ULONG;
    Node.Property.Flags = (KSPROPERTY_TYPE_GET) as ULONG;
    Node.Property.Set = KSPROPSETID_Topology;
    Node.Reserved = (0) as ULONG;

    /* get node name size */
    Status = (*MixerContext).Control.unwrap_unchecked()(
        hMixer,
        (IOCTL_KS_PROPERTY) as ULONG,
        core::ptr::addr_of_mut!(Node) as PVOID,
        core::mem::size_of::<KSP_NODE>() as ULONG,
        core::ptr::null_mut(),
        (0) as ULONG,
        core::ptr::addr_of_mut!(BytesReturned),
    );

    if Status == (MM_STATUS_MORE_ENTRIES) as MIXER_STATUS {
        ASSERT!(BytesReturned != (0) as ULONG);
        Name = (*MixerContext).Alloc.unwrap_unchecked()(BytesReturned)
            as LPWSTR;
        if Name.is_null() {
            /* not enough memory */
            return (MM_STATUS_NO_MEMORY) as MIXER_STATUS;
        }

        /* get node name */
        Status = (*MixerContext).Control.unwrap_unchecked()(
            hMixer,
            (IOCTL_KS_PROPERTY) as ULONG,
            core::ptr::addr_of_mut!(Node) as PVOID,
            core::mem::size_of::<KSP_NODE>() as ULONG,
            Name as PVOID,
            BytesReturned,
            core::ptr::addr_of_mut!(BytesReturned),
        );

        if Status == (MM_STATUS_SUCCESS) as MIXER_STATUS {
            (*MixerContext).Copy.unwrap_unchecked()(
                core::ptr::addr_of_mut!((*MixerControl).Control.szShortName) as PVOID,
                Name as PVOID,
                (min(
                    (MIXER_SHORT_NAME_CHARS) as usize,
                    (wcslen(Name as *const WCHAR) as usize)
                        + ((1) as usize),
                ) * core::mem::size_of::<WCHAR>()) as ULONG,
            );
            (*MixerControl).Control.szShortName[(MIXER_SHORT_NAME_CHARS as usize)
                - (1 as usize)] = 0x0000u16;

            (*MixerContext).Copy.unwrap_unchecked()(
                core::ptr::addr_of_mut!((*MixerControl).Control.szName) as PVOID,
                Name as PVOID,
                (min(
                    (MIXER_LONG_NAME_CHARS) as usize,
                    (wcslen(Name as *const WCHAR) as usize)
                        + ((1) as usize),
                ) * core::mem::size_of::<WCHAR>()) as ULONG,
            );
            (*MixerControl).Control.szName[(MIXER_LONG_NAME_CHARS as usize)
                - (1 as usize)] = 0x0000u16;
        }

        /* free name buffer */
        (*MixerContext).Free.unwrap_unchecked()(Name as PVOID);
    }

    /* increment control count */
    (*MixerInfo).ControlId = (*MixerInfo).ControlId.wrapping_add(1);

    /* insert control */
    InsertTailList(
        core::ptr::addr_of_mut!((*MixerLine).ControlsList),
        core::ptr::addr_of_mut!((*MixerControl).Entry),
    );

    if (*MixerControl).Control.dwControlType == (MIXERCONTROL_CONTROLTYPE_MUX) as DWORD {
        let mut NodesCount: ULONG = 0;
        let mut Nodes: PULONG = core::ptr::null_mut();

        /* allocate topology nodes array */
        Status = MMixerAllocateTopologyNodeArray(
            MixerContext,
            Topology,
            core::ptr::addr_of_mut!(Nodes),
        );

        if Status != (MM_STATUS_SUCCESS) as MIXER_STATUS {
            /* out of memory */
            return (STATUS_NO_MEMORY) as MIXER_STATUS;
        }

        /* get connected node count */
        MMixerGetNextNodesFromNodeIndex(
            MixerContext,
            Topology,
            NodeIndex,
            (TRUE) as ULONG,
            core::ptr::addr_of_mut!(NodesCount),
            Nodes,
        );

        /* TODO */
        (*MixerContext).Free.unwrap_unchecked()(Nodes as PVOID);

        /* setup mux bounds */
        (*MixerControl).Control.Bounds.__bindgen_anon_2.dwMinimum = (0) as DWORD;
        (*MixerControl).Control.Bounds.__bindgen_anon_2.dwMaximum =
            NodesCount.wrapping_sub((1) as ULONG) as DWORD;
        (*MixerControl).Control.Metrics.dwReserved[(0) as usize] =
            NodesCount as DWORD;
        (*MixerControl).Control.cMultipleItems = NodesCount as DWORD;
        (*MixerControl).Control.fdwControl |= ((MIXERCONTROL_CONTROLF_UNIFORM) as DWORD)
            | ((MIXERCONTROL_CONTROLF_MULTIPLE) as DWORD);
    } else if (*MixerControl).Control.dwControlType == (MIXERCONTROL_CONTROLTYPE_MUTE) as DWORD {
        (*MixerControl).Control.Bounds.__bindgen_anon_2.dwMinimum = (0) as DWORD;
        (*MixerControl).Control.Bounds.__bindgen_anon_2.dwMaximum = (1) as DWORD;
    } else if (*MixerControl).Control.dwControlType == (MIXERCONTROL_CONTROLTYPE_ONOFF) as DWORD {
        /* only needs to set bounds */
        (*MixerControl).Control.Bounds.__bindgen_anon_2.dwMinimum = (0) as DWORD;
        (*MixerControl).Control.Bounds.__bindgen_anon_2.dwMaximum = (1) as DWORD;
    } else if (*MixerControl).Control.dwControlType == (MIXERCONTROL_CONTROLTYPE_VOLUME) as DWORD {
        let mut Property: KSNODEPROPERTY_AUDIO_CHANNEL = core::mem::zeroed();
        let mut Length: ULONG = 0;
        let mut Desc: PKSPROPERTY_DESCRIPTION = core::ptr::null_mut();
        let mut Members: PKSPROPERTY_MEMBERSHEADER = core::ptr::null_mut();
        let mut Range: PKSPROPERTY_STEPPING_LONG = core::ptr::null_mut();
        let mut VolumeData: LPMIXERVOLUME_DATA = core::ptr::null_mut();

        (*MixerControl).Control.Bounds.__bindgen_anon_2.dwMinimum = (0) as DWORD;
        (*MixerControl).Control.Bounds.__bindgen_anon_2.dwMaximum = (0xFFFF) as DWORD;
        (*MixerControl).Control.Metrics.cSteps = (0xC0) as DWORD; /* FIXME */

        Length = (core::mem::size_of::<KSPROPERTY_DESCRIPTION>() as usize)
            .wrapping_add(core::mem::size_of::<KSPROPERTY_MEMBERSHEADER>() as usize)
            .wrapping_add(core::mem::size_of::<KSPROPERTY_STEPPING_LONG>() as usize)
            as ULONG;
        Desc = (*MixerContext).Alloc.unwrap_unchecked()(Length)
            as PKSPROPERTY_DESCRIPTION;
        ASSERT!(!Desc.is_null());

        /* setup the request */
        RtlZeroMemory!(
            core::ptr::addr_of_mut!(Property) as PVOID,
            core::mem::size_of::<KSNODEPROPERTY_AUDIO_CHANNEL>() as usize,
        );

        Property.NodeProperty.NodeId = NodeIndex;
        Property.NodeProperty.Property.Id = (KSPROPERTY_AUDIO_VOLUMELEVEL) as ULONG;
        Property.NodeProperty.Property.Flags =
            ((KSPROPERTY_TYPE_BASICSUPPORT) as ULONG) | ((KSPROPERTY_TYPE_TOPOLOGY) as ULONG);
        Property.NodeProperty.Property.Set = KSPROPSETID_Audio;

        /* get node volume level info */
        Status = (*MixerContext).Control.unwrap_unchecked()(
            hMixer,
            (IOCTL_KS_PROPERTY) as ULONG,
            core::ptr::addr_of_mut!(Property) as PVOID,
            core::mem::size_of::<KSNODEPROPERTY_AUDIO_CHANNEL>() as ULONG,
            Desc as PVOID,
            Length,
            core::ptr::addr_of_mut!(BytesReturned),
        );

        VolumeData = (*MixerContext).Alloc.unwrap_unchecked()(
            core::mem::size_of::<MIXERVOLUME_DATA>() as ULONG,
        ) as LPMIXERVOLUME_DATA;
        if VolumeData.is_null() {
            return (MM_STATUS_NO_MEMORY) as MIXER_STATUS;
        }

        if Status == (MM_STATUS_SUCCESS) as MIXER_STATUS {
            Members = Desc.offset((1) as isize)
                as PKSPROPERTY_MEMBERSHEADER;
            Range = Members.offset((1) as isize)
                as PKSPROPERTY_STEPPING_LONG;

            DPRINT!("NodeIndex %u Range Min %d Max %d Steps %x UMin %x UMax %x\n", NodeIndex, (*Range).Bounds.SignedMinimum, (*Range).Bounds.SignedMaximum, (*Range).SteppingDelta, (*Range).Bounds.UnsignedMinimum, (*Range).Bounds.UnsignedMaximum);

            /* Store mixer control info there */
            (*VolumeData).Header.dwControlID = (*MixerControl).Control.dwControlID;
            (*VolumeData).SignedMinimum = (*Range).Bounds.__bindgen_anon_1.SignedMinimum;
            (*VolumeData).SignedMaximum = (*Range).Bounds.__bindgen_anon_1.SignedMaximum;

            /* Fallback to defaults if: 1) the range is not defined (typically is 0) */
            if (*VolumeData).SignedMinimum == (*VolumeData).SignedMaximum {
                (*VolumeData).SignedMinimum =
                    (-((96) as LONG) * ((0x10000) as LONG))
                        as LONG; // -96 DB
                (*VolumeData).SignedMaximum = (0) as LONG; // 0 DB
            }
        } else {
            /* or 2) when some failure occurs */
            (*VolumeData).Header.dwControlID = (*MixerControl).Control.dwControlID;
            (*VolumeData).SignedMinimum =
                (-((96) as LONG) * ((0x10000) as LONG))
                    as LONG; // -96 DB
            (*VolumeData).SignedMaximum = (0) as LONG; // 0 DB
        }
        (*MixerControl).ExtraData = VolumeData as PVOID;
        (*MixerContext).Free.unwrap_unchecked()(Desc as PVOID);
    }

    DPRINT!(
        "Status %x Name %S\n",
        Status,
        core::ptr::addr_of_mut!((*MixerControl).Control.szName) as PVOID
    );
    return (MM_STATUS_SUCCESS) as MIXER_STATUS;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerCreateDestinationLine(
    mut MixerContext: PMIXER_CONTEXT,
    mut MixerInfo: LPMIXER_INFO,
    mut bInputMixer: ULONG,
    mut LineName: LPWSTR,
) -> MIXER_STATUS {
    let mut DestinationLine: LPMIXERLINE_EXT = ::core::ptr::null_mut::<MIXERLINE_EXT>();

    /* allocate a mixer destination line */
    DestinationLine = (*MixerContext).Alloc.unwrap_unchecked()(
        core::mem::size_of::<MIXERLINE_EXT>() as ULONG,
    ) as LPMIXERLINE_EXT;
    if MixerInfo.is_null() {
        /* no memory */
        return MM_STATUS_NO_MEMORY as MIXER_STATUS;
    }

    /* initialize mixer destination line */
    (*DestinationLine).Line.cbStruct = core::mem::size_of::<MIXERLINEW>() as DWORD;
    (*DestinationLine).Line.cChannels = 2 as DWORD; /* FIXME */
    (*DestinationLine).Line.cConnections = 0 as DWORD;
    (*DestinationLine).Line.cControls = 0 as DWORD;
    (*DestinationLine).Line.dwComponentType = (if bInputMixer == 0 as ULONG {
        MIXERLINE_COMPONENTTYPE_DST_SPEAKERS
    } else {
        MIXERLINE_COMPONENTTYPE_DST_WAVEIN
    }) as DWORD;
    (*DestinationLine).Line.dwDestination = (*MixerInfo).MixCaps.cDestinations;
    (*DestinationLine).Line.dwLineID = (*MixerInfo)
        .MixCaps
        .cDestinations
        .wrapping_add(DESTINATION_LINE as DWORD);
    (*DestinationLine).Line.dwSource = MAXULONG as DWORD;
    (*DestinationLine).Line.dwUser = 0 as DWORD_PTR;
    (*DestinationLine).Line.fdwLine = MIXERLINE_LINEF_ACTIVE as DWORD;

    if !LineName.is_null() {
        (*MixerContext).Copy_0.unwrap_unchecked()(
            &raw mut (*DestinationLine).Line.szShortName as *mut WCHAR as PVOID,
            LineName as PVOID,
            (min(
                MIXER_SHORT_NAME_CHARS as usize,
                wcslen(LineName as *const wchar_t)
                    .wrapping_add(1 as usize),
            ))
            .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as ULONG,
        );
        (*DestinationLine).Line.szShortName
            [(MIXER_SHORT_NAME_CHARS - 1) as usize] = 0x0000u16;

        (*MixerContext).Copy_0.unwrap_unchecked()(
            &raw mut (*DestinationLine).Line.szName as *mut WCHAR as PVOID,
            LineName as PVOID,
            (min(
                MIXER_LONG_NAME_CHARS as usize,
                wcslen(LineName as *const wchar_t)
                    .wrapping_add(1 as usize),
            ))
            .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as ULONG,
        );
        (*DestinationLine).Line.szName
            [(MIXER_LONG_NAME_CHARS - 1) as usize] = 0x0000u16;
    }

    (*DestinationLine).Line.Target.dwType = (if bInputMixer == 0 as ULONG {
        MIXERLINE_TARGETTYPE_WAVEOUT
    } else {
        MIXERLINE_TARGETTYPE_WAVEIN
    }) as DWORD;
    (*DestinationLine).Line.Target.dwDeviceID = 0 as DWORD; //FIXME
    (*DestinationLine).Line.Target.wMid = (*MixerInfo).MixCaps.wMid;
    (*DestinationLine).Line.Target.wPid = (*MixerInfo).MixCaps.wPid;
    (*DestinationLine).Line.Target.vDriverVersion = (*MixerInfo).MixCaps.vDriverVersion;

    ASSERT!(
        (*MixerInfo).MixCaps.szPname
            [(MAXPNAMELEN - 1) as usize]
            == 0
    );
    wcscpy(
        &raw mut (*DestinationLine).Line.Target.szPname as *mut WCHAR,
        &raw mut (*MixerInfo).MixCaps.szPname as *mut WCHAR,
    );

    /* initialize extra line */
    InitializeListHead(&raw mut (*DestinationLine).ControlsList);

    /* insert into mixer info */
    InsertTailList(
        &raw mut (*MixerInfo).LineList,
        &raw mut (*DestinationLine).Entry,
    );

    /* increment destination count */
    (*MixerInfo).MixCaps.cDestinations = (*MixerInfo).MixCaps.cDestinations.wrapping_add(1);

    /* done */
    return MM_STATUS_SUCCESS as MIXER_STATUS;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetPinName(
    mut MixerContext: PMIXER_CONTEXT,
    mut MixerInfo: LPMIXER_INFO,
    mut hMixer: HANDLE,
    mut PinId: ULONG,
    mut OutBuffer: *mut LPWSTR,
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
    let mut BytesReturned: ULONG = 0;
    let mut Buffer: LPWSTR = ::core::ptr::null_mut::<WCHAR>();
    let mut Status: MIXER_STATUS = MM_STATUS_SUCCESS;
    Pin.PinId = PinId;
    Pin.Reserved = 0 as ULONG;
    Pin.Property.Flags = KSPROPERTY_TYPE_GET as ULONG;
    Pin.Property.Set = KSPROPSETID_Pin;
    Pin.Property.Id = KSPROPERTY_PIN_NAME as ULONG;
    Status = (*MixerContext).Control.unwrap_unchecked()(
        hMixer,
        IOCTL_KS_PROPERTY as ULONG,
        &raw mut Pin as PVOID,
        ::core::mem::size_of::<KSP_PIN>() as ULONG,
        NULL,
        0 as ULONG,
        &raw mut BytesReturned,
    );
    if Status as ::core::ffi::c_uint
        == MM_STATUS_MORE_ENTRIES as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        Buffer = (*MixerContext).Alloc.unwrap_unchecked()(BytesReturned) as LPWSTR;
        if Buffer.is_null() {
            return MM_STATUS_NO_MEMORY;
        }
        Status = (*MixerContext).Control.unwrap_unchecked()(
            hMixer,
            IOCTL_KS_PROPERTY as ULONG,
            &raw mut Pin as PVOID,
            ::core::mem::size_of::<KSP_PIN>() as ULONG,
            Buffer as PVOID,
            BytesReturned,
            &raw mut BytesReturned,
        );
        if Status as ::core::ffi::c_uint
            != MM_STATUS_SUCCESS as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*MixerContext).Free.unwrap_unchecked()(Buffer as PVOID);
            return Status;
        }
        *OutBuffer = Buffer;
        return MM_STATUS_SUCCESS;
    }
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerBuildMixerDestinationLine(
    mut MixerContext: PMIXER_CONTEXT,
    mut MixerInfo: LPMIXER_INFO,
    mut hMixer: HANDLE,
    mut PinId: ULONG,
    mut bInput: ULONG,
) -> MIXER_STATUS {
    let mut PinName: LPWSTR = core::ptr::null_mut();
    let mut Status: MIXER_STATUS = core::mem::zeroed();

    /* try get pin name */
    Status = MMixerGetPinName(MixerContext, MixerInfo, hMixer, PinId, core::ptr::addr_of_mut!(PinName));
    if Status == MM_STATUS_SUCCESS
    {
        /* create mixer destination line */

        Status = MMixerCreateDestinationLine(MixerContext, MixerInfo, bInput, PinName);

        /* free pin name */
        (*MixerContext).Free.unwrap_unchecked()(PinName as PVOID);
    }
    else
    {
        /* create mixer destination line unlocalized */
        Status = MMixerCreateDestinationLine(MixerContext, MixerInfo, bInput, RUSTCONV_WSTR_F8619DE16B4F.as_ptr() as LPWSTR);
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerBuildTopology(
    mut MixerContext: PMIXER_CONTEXT,
    mut MixerData: LPMIXER_DATA,
    mut OutTopology: *mut PTOPOLOGY,
) -> MIXER_STATUS {
    let mut PinsCount: ULONG = 0;
    let mut NodeTypes: PKSMULTIPLE_ITEM = core::ptr::null_mut();
    let mut NodeConnections: PKSMULTIPLE_ITEM = core::ptr::null_mut();
    let mut Status: MIXER_STATUS;

    if !(*MixerData).Topology.is_null() {
        /* re-use existing topology */
        *OutTopology = (*MixerData).Topology;

        return MM_STATUS_SUCCESS;
    }

    /* get connected filter pin count */
    PinsCount = MMixerGetFilterPinCount(MixerContext, (*MixerData).hDevice);

    if PinsCount == 0 {
        /* referenced filter does not have any pins */
        return MM_STATUS_UNSUCCESSFUL;
    }

    /* get topology node types */
    Status = MMixerGetFilterTopologyProperty(
        MixerContext,
        (*MixerData).hDevice,
        KSPROPERTY_TOPOLOGY_NODES,
        &raw mut NodeTypes,
    );
    if Status != MM_STATUS_SUCCESS {
        /* failed to get topology node types */
        return Status;
    }

    /* get topology connections */
    Status = MMixerGetFilterTopologyProperty(
        MixerContext,
        (*MixerData).hDevice,
        KSPROPERTY_TOPOLOGY_CONNECTIONS,
        &raw mut NodeConnections,
    );
    if Status != MM_STATUS_SUCCESS {
        /* failed to get topology connections */
        (*MixerContext).Free.unwrap_unchecked()(NodeTypes as PVOID);
        return Status;
    }

    /* create a topology */
    Status = MMixerCreateTopology(
        MixerContext,
        PinsCount,
        NodeConnections,
        NodeTypes,
        OutTopology,
    );

    /* free node types & connections */
    (*MixerContext).Free.unwrap_unchecked()(NodeConnections as PVOID);
    (*MixerContext).Free.unwrap_unchecked()(NodeTypes as PVOID);

    if Status == MM_STATUS_SUCCESS {
        /* store topology object */
        (*MixerData).Topology = *OutTopology;
    }

    /* done */
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerCountMixerControls(MixerContext: PMIXER_CONTEXT, Topology: PTOPOLOGY, PinId: ULONG, bInputMixer: ULONG, bUpStream: ULONG, OutNodesCount: PULONG, OutNodes: PULONG, OutLineTerminator: PULONG) -> MIXER_STATUS {
    let mut Nodes: PULONG = core::ptr::null_mut();
    let mut NodesCount: ULONG = 0;
    let mut NodeIndex: ULONG = 0;
    let mut Count: ULONG = 0;
    let mut bTerminator: ULONG = 0;
    let mut Status: MIXER_STATUS;

    /* allocate an array to store all nodes which are upstream of this pin */
    Status = MMixerAllocateTopologyNodeArray(MixerContext, Topology, core::ptr::addr_of_mut!(Nodes));

    if Status != MM_STATUS_SUCCESS {
        /* out of memory */
        return (STATUS_NO_MEMORY) as MIXER_STATUS;
    }

    /* mark result array as zero */
    *OutNodesCount = 0;

    /* get next nodes */
    MMixerGetNextNodesFromPinIndex(MixerContext, Topology, PinId, bUpStream, core::ptr::addr_of_mut!(NodesCount), Nodes);

    if NodesCount == 0 {
        /* a pin which is not connected from any node
         * a) it is a topology bug (driver bug)
         * b) the request is from an alternative mixer
              alternative mixer code scans all pins which have not been used and tries to build lines
         */
        DPRINT1!("MMixerCountMixerControls PinId %lu is not connected by any node\n", PinId);
        MMixerPrintTopology(Topology);
        return MM_STATUS_UNSUCCESSFUL;
    }

    /* assume no topology split before getting line terminator */
    ASSERT!(NodesCount == 1);

    /* get first node */
    NodeIndex = *Nodes.add(0 as usize);
    Count = 0;

    loop {
        /* check if the node is a terminator */
        MMixerIsNodeTerminator(Topology, NodeIndex, core::ptr::addr_of_mut!(bTerminator));

        if bTerminator != 0 {
            /* found terminator */
            if bInputMixer != 0 {
                /* add mux source for source destination line */
                *OutNodes.add(Count as usize) = NodeIndex;
                Count = Count.wrapping_add(1);
            }
            break;
        }

        /* store node id */
        *OutNodes.add(Count as usize) = NodeIndex;

        /* increment node count */
        Count = Count.wrapping_add(1);

        /* get next nodes upstream */
        MMixerGetNextNodesFromNodeIndex(MixerContext, Topology, NodeIndex, bUpStream, core::ptr::addr_of_mut!(NodesCount), Nodes);

        if NodesCount != 1 {
            DPRINT!("PinId %lu bInputMixer %lu bUpStream %lu NodeIndex %lu is not connected", PinId, bInputMixer, bUpStream, NodeIndex);
            break;
        }

        ASSERT!(NodesCount == 1);

        /* use first index */
        NodeIndex = *Nodes.add(0 as usize);

        if (TRUE) == 0 {
            break;
        }
    }

    /* free node index */
    ((*MixerContext).Free.unwrap_unchecked())(Nodes as PVOID);

    /* store nodes count */
    *OutNodesCount = Count;

    /* store line terminator */
    *OutLineTerminator = NodeIndex;

    /* done */
    return MM_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetChannelCountEnhanced(
    mut MixerContext: PMIXER_CONTEXT,
    mut MixerInfo: LPMIXER_INFO,
    mut hMixer: HANDLE,
    mut NodeId: ULONG,
    mut MaxChannels: PULONG,
) -> MIXER_STATUS {
    let mut Description: KSPROPERTY_DESCRIPTION = KSPROPERTY_DESCRIPTION {
        AccessFlags: 0,
        DescriptionSize: 0,
        PropTypeSet: KSIDENTIFIER {
            Set: _GUID {
                Data1: 0,
                Data2: 0,
                Data3: 0,
                Data4: [0; 8],
            },
            Id: 0,
            Flags: 0,
        },
        MembersListCount: 0,
        Reserved: 0,
    };
    let mut NewDescription: PKSPROPERTY_DESCRIPTION =
        ::core::ptr::null_mut::<KSPROPERTY_DESCRIPTION>();
    let mut Header: PKSPROPERTY_MEMBERSHEADER = ::core::ptr::null_mut::<KSPROPERTY_MEMBERSHEADER>();
    let mut BytesReturned: ULONG = 0;
    let mut Request: KSP_NODE = KSP_NODE {
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
        NodeId: 0,
        Reserved: 0,
    };
    let mut Status: MIXER_STATUS = MM_STATUS_SUCCESS;

    /* try #1 obtain it via description */
    Request.NodeId = NodeId;
    Request.Reserved = 0 as ULONG;
    Request.Property.Set = KSPROPSETID_Audio;
    Request.Property.Flags = (KSPROPERTY_TYPE_BASICSUPPORT | KSPROPERTY_TYPE_TOPOLOGY) as ULONG;
    Request.Property.Id = KSPROPERTY_AUDIO_VOLUMELEVEL as ULONG;

    /* get description */
    Status = (*MixerContext).Control.unwrap_unchecked()(
        hMixer,
        IOCTL_KS_PROPERTY as ULONG,
        &raw mut Request as PVOID,
        ::core::mem::size_of::<KSP_NODE>() as ULONG,
        &raw mut Description as PVOID,
        ::core::mem::size_of::<KSPROPERTY_DESCRIPTION>() as ULONG,
        &raw mut BytesReturned,
    );
    if Status == (MM_STATUS_SUCCESS as MIXER_STATUS) {
        if (Description.DescriptionSize as usize)
            >= (::core::mem::size_of::<KSPROPERTY_DESCRIPTION>() as usize)
                .wrapping_add(::core::mem::size_of::<KSPROPERTY_MEMBERSHEADER>() as usize)
            && (Description.MembersListCount > (0 as ULONG))
        {
            /* allocate new description */
            NewDescription = (*MixerContext).Alloc.unwrap_unchecked()(
                Description.DescriptionSize,
            ) as PKSPROPERTY_DESCRIPTION;

            if NewDescription.is_null() {
                /* not enough memory */
                return MM_STATUS_NO_MEMORY as MIXER_STATUS;
            }

            /* get description */
            Status = (*MixerContext).Control.unwrap_unchecked()(
                hMixer,
                IOCTL_KS_PROPERTY as ULONG,
                &raw mut Request as PVOID,
                ::core::mem::size_of::<KSP_NODE>() as ULONG,
                NewDescription as PVOID,
                Description.DescriptionSize,
                &raw mut BytesReturned,
            );
            if Status == (MM_STATUS_SUCCESS as MIXER_STATUS) {
                /* get header */
                Header = NewDescription.offset(1 as isize)
                    as PKSPROPERTY_MEMBERSHEADER;

                if ((*Header).Flags & (KSPROPERTY_MEMBER_FLAG_BASICSUPPORT_MULTICHANNEL as ULONG)) != 0 {
                    /* found enhanced flag */
                    ASSERT!((*Header).MembersCount > (1 as ULONG));

                    /* store channel count */
                    *MaxChannels = (*Header).MembersCount;

                    /* free description */
                    (*MixerContext).Free.unwrap_unchecked()(
                        NewDescription as PVOID,
                    );

                    /* done */
                    return MM_STATUS_SUCCESS as MIXER_STATUS;
                }
            }

            /* free description */
            (*MixerContext).Free.unwrap_unchecked()(NewDescription as PVOID);
        }
    }

    /* failed to get channel count enhanced */
    return MM_STATUS_UNSUCCESSFUL as MIXER_STATUS;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetChannelCountLegacy(
    mut MixerContext: PMIXER_CONTEXT,
    mut MixerInfo: LPMIXER_INFO,
    mut hMixer: HANDLE,
    mut NodeId: ULONG,
    mut MaxChannels: PULONG,
) {
    let mut BytesReturned: ULONG = 0;
    let mut Status: MIXER_STATUS = MM_STATUS_SUCCESS;
    let mut Channel: KSNODEPROPERTY_AUDIO_CHANNEL = KSNODEPROPERTY_AUDIO_CHANNEL {
        NodeProperty: KSNODEPROPERTY {
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
            NodeId: 0,
            Reserved: 0,
        },
        Channel: 0,
        Reserved: 0,
    };
    let mut Volume: LONG = 0;

    /* setup request */
    Channel.Reserved = 0 as ULONG;
    Channel.NodeProperty.NodeId = NodeId;
    Channel.NodeProperty.Reserved = 0 as ULONG;
    Channel.NodeProperty.Property.Flags = (KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_TOPOLOGY) as ULONG;
    Channel.NodeProperty.Property.Set = KSPROPSETID_Audio;
    Channel.Channel = 0 as LONG;
    Channel.NodeProperty.Property.Id = KSPROPERTY_AUDIO_VOLUMELEVEL as ULONG;

    loop {
        /* get channel volume */
        Status = (*MixerContext).Control.unwrap_unchecked()(
            hMixer,
            IOCTL_KS_PROPERTY as ULONG,
            core::ptr::addr_of_mut!(Channel) as PVOID,
            ::core::mem::size_of::<KSNODEPROPERTY_AUDIO_CHANNEL>() as ULONG,
            core::ptr::addr_of_mut!(Volume) as PVOID,
            ::core::mem::size_of::<LONG>() as ULONG,
            core::ptr::addr_of_mut!(BytesReturned),
        );
        if Status != MM_STATUS_SUCCESS {
            break;
        }

        /* increment channel count */
        Channel.Channel += 1;

        if !(TRUE != 0) {
            break;
        }
    }

    /* store channel count */
    *MaxChannels = Channel.Channel as ULONG;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetMaxChannelsForNode(
    mut MixerContext: PMIXER_CONTEXT,
    mut MixerInfo: LPMIXER_INFO,
    mut hMixer: HANDLE,
    mut NodeId: ULONG,
    mut MaxChannels: PULONG,
) {
    let mut Status: MIXER_STATUS;

    /* try to get it enhanced */
    Status = MMixerGetChannelCountEnhanced(MixerContext, MixerInfo, hMixer, NodeId, MaxChannels);

    if Status != MM_STATUS_SUCCESS {
        /* get it old-fashioned way */
        MMixerGetChannelCountLegacy(MixerContext, MixerInfo, hMixer, NodeId, MaxChannels);
    }
}
#[no_mangle]
pub unsafe extern "C" fn MMixerAddMixerControlsToMixerLineByNodeIndexArray(
    MixerContext: PMIXER_CONTEXT,
    MixerInfo: LPMIXER_INFO,
    hMixer: HANDLE,
    Topology: PTOPOLOGY,
    DstLine: LPMIXERLINE_EXT,
    NodesCount: ULONG,
    Nodes: PULONG,
) -> MIXER_STATUS {
    let mut Index: ULONG;
    let mut Count: ULONG;
    let mut bReserved: ULONG = core::mem::zeroed();
    let mut Status: MIXER_STATUS;
    let mut NodeType: LPGUID;
    let mut MaxChannels: ULONG = core::mem::zeroed();

    /* initialize control count */
    Count = 0;

    Index = 0;
    while Index < NodesCount {
        /* check if the node has already been reserved to a line */
        MMixerIsTopologyNodeReserved(Topology, *Nodes.offset(Index as isize) as ULONG, &raw mut bReserved);
        /* set node status as used */
        MMixerSetTopologyNodeReserved(Topology, *Nodes.offset(Index as isize) as ULONG);

        /* query node type */
        NodeType = MMixerGetNodeTypeFromTopology(Topology, *Nodes.offset(Index as isize) as ULONG);

        if IsEqualGUIDAligned(NodeType, &raw const KSNODETYPE_VOLUME as LPGUID) != false {
            /* calculate maximum channel count for node */
            MMixerGetMaxChannelsForNode(
                MixerContext,
                MixerInfo,
                hMixer,
                *Nodes.offset(Index as isize) as ULONG,
                &raw mut MaxChannels,
            );

            DPRINT!(
                "NodeId %lu MaxChannels %lu Line %S Id %lu\n",
                *Nodes.offset(Index as isize) as ULONG,
                MaxChannels,
                (*DstLine).Line.szName,
                (*DstLine).Line.dwLineID
            );
            /* calculate maximum channels */
            (*DstLine).Line.cChannels = min((*DstLine).Line.cChannels, MaxChannels) as DWORD;
        } else {
            /* use default of one channel */
            MaxChannels = 1;
        }

        /* now add the mixer control */
        Status = MMixerAddMixerControl(
            MixerContext,
            MixerInfo,
            hMixer,
            Topology,
            *Nodes.offset(Index as isize) as ULONG,
            DstLine,
            MaxChannels,
        );

        if Status == MM_STATUS_SUCCESS {
            /* increment control count */
            Count = Count.wrapping_add(1);
        }

        Index = Index.wrapping_add(1);
    }

    /* store control count */
    (*DstLine).Line.cControls = Count as DWORD;

    /* done */
    return MM_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetComponentAndTargetType(
    MixerContext: PMIXER_CONTEXT,
    MixerInfo: LPMIXER_INFO,
    hMixer: HANDLE,
    PinId: ULONG,
    ComponentType: PULONG,
    TargetType: PULONG,
) -> MIXER_STATUS {
    let mut DataFlow: KSPIN_DATAFLOW = 0 as KSPIN_DATAFLOW;
    let mut Communication: KSPIN_COMMUNICATION = 0 as KSPIN_COMMUNICATION;
    let mut Status: MIXER_STATUS = 0 as MIXER_STATUS;
    let mut Request: KSP_PIN = core::mem::zeroed();
    let mut BytesReturned: ULONG = 0;
    let mut Guid: GUID = core::mem::zeroed();
    let mut BridgePin: BOOLEAN = FALSE as BOOLEAN;
    let mut Connection: PKSPIN_PHYSICALCONNECTION = core::ptr::null_mut();

    /* first dataflow type */
    Status = MMixerGetPinDataFlowAndCommunication(
        MixerContext,
        hMixer,
        PinId,
        core::ptr::addr_of_mut!(DataFlow),
        core::ptr::addr_of_mut!(Communication),
    );

    if Status != MM_STATUS_SUCCESS as MIXER_STATUS {
        /* failed to get dataflow */
        return Status;
    }

    /* now get pin category guid */
    Request.PinId = PinId;
    Request.Reserved = 0 as ULONG;
    Request.Property.Flags = KSPROPERTY_TYPE_GET as ULONG;
    Request.Property.Set = KSPROPSETID_Pin;
    Request.Property.Id = KSPROPERTY_PIN_CATEGORY as ULONG;

    /* get pin category */
    Status = (*MixerContext).Control.unwrap_unchecked()(
        hMixer,
        IOCTL_KS_PROPERTY as ULONG,
        core::ptr::addr_of_mut!(Request) as PVOID,
        core::mem::size_of::<KSP_PIN>() as ULONG,
        core::ptr::addr_of_mut!(Guid) as PVOID,
        core::mem::size_of::<GUID>() as ULONG,
        core::ptr::addr_of_mut!(BytesReturned),
    );
    if Status != MM_STATUS_SUCCESS as MIXER_STATUS {
        /* failed to get dataflow */
        return Status;
    }

    /* check if it has a physical connection */
    Status = MMixerGetPhysicalConnection(
        MixerContext,
        hMixer,
        PinId,
        core::ptr::addr_of_mut!(Connection),
    );
    if Status == MM_STATUS_SUCCESS as MIXER_STATUS {
        /* pin is a brige pin */
        BridgePin = TRUE as BOOLEAN;

        /* free physical connection */
        (*MixerContext).Free.unwrap_unchecked()(Connection as PVOID);
    }

    if DataFlow == KSPIN_DATAFLOW_IN as KSPIN_DATAFLOW {
        if IsEqualGUIDAligned(
            core::ptr::addr_of!(Guid),
            core::ptr::addr_of!(KSNODETYPE_MICROPHONE),
        ) || IsEqualGUIDAligned(
            core::ptr::addr_of!(Guid),
            core::ptr::addr_of!(KSNODETYPE_DESKTOP_MICROPHONE),
        ) {
            /* type microphone */
            *TargetType = MIXERLINE_TARGETTYPE_WAVEIN as ULONG;
            *ComponentType = MIXERLINE_COMPONENTTYPE_SRC_MICROPHONE as ULONG;
        } else if IsEqualGUIDAligned(
            core::ptr::addr_of!(Guid),
            core::ptr::addr_of!(KSNODETYPE_LEGACY_AUDIO_CONNECTOR),
        ) || IsEqualGUIDAligned(
            core::ptr::addr_of!(Guid),
            core::ptr::addr_of!(KSCATEGORY_AUDIO),
        ) || IsEqualGUIDAligned(
            core::ptr::addr_of!(Guid),
            core::ptr::addr_of!(KSNODETYPE_SPEAKER),
        ) {
            /* type waveout */
            *TargetType = MIXERLINE_TARGETTYPE_WAVEOUT as ULONG;
            *ComponentType = MIXERLINE_COMPONENTTYPE_SRC_WAVEOUT as ULONG;
        } else if IsEqualGUIDAligned(
            core::ptr::addr_of!(Guid),
            core::ptr::addr_of!(KSNODETYPE_CD_PLAYER),
        ) {
            /* type cd player */
            *TargetType = MIXERLINE_TARGETTYPE_UNDEFINED as ULONG;
            *ComponentType = MIXERLINE_COMPONENTTYPE_SRC_COMPACTDISC as ULONG;
        } else if IsEqualGUIDAligned(
            core::ptr::addr_of!(Guid),
            core::ptr::addr_of!(KSNODETYPE_SYNTHESIZER),
        ) {
            /* type synthesizer */
            *TargetType = MIXERLINE_TARGETTYPE_MIDIOUT as ULONG;
            *ComponentType = MIXERLINE_COMPONENTTYPE_SRC_SYNTHESIZER as ULONG;
        } else if IsEqualGUIDAligned(
            core::ptr::addr_of!(Guid),
            core::ptr::addr_of!(KSNODETYPE_LINE_CONNECTOR),
        ) {
            /* type line */
            *TargetType = MIXERLINE_TARGETTYPE_UNDEFINED as ULONG;
            *ComponentType = MIXERLINE_COMPONENTTYPE_SRC_LINE as ULONG;
        } else if IsEqualGUIDAligned(
            core::ptr::addr_of!(Guid),
            core::ptr::addr_of!(KSNODETYPE_TELEPHONE),
        ) || IsEqualGUIDAligned(
            core::ptr::addr_of!(Guid),
            core::ptr::addr_of!(KSNODETYPE_PHONE_LINE),
        ) || IsEqualGUIDAligned(
            core::ptr::addr_of!(Guid),
            core::ptr::addr_of!(KSNODETYPE_DOWN_LINE_PHONE),
        ) {
            /* type telephone */
            *TargetType = MIXERLINE_TARGETTYPE_UNDEFINED as ULONG;
            *ComponentType = MIXERLINE_COMPONENTTYPE_SRC_TELEPHONE as ULONG;
        } else if IsEqualGUIDAligned(
            core::ptr::addr_of!(Guid),
            core::ptr::addr_of!(KSNODETYPE_ANALOG_CONNECTOR),
        ) {
            /* type analog */
            if BridgePin != 0 {
                *TargetType = MIXERLINE_TARGETTYPE_WAVEIN as ULONG;
            } else {
                *TargetType = MIXERLINE_TARGETTYPE_WAVEOUT as ULONG;
            }

            *ComponentType = MIXERLINE_COMPONENTTYPE_SRC_ANALOG as ULONG;
        } else if IsEqualGUIDAligned(
            core::ptr::addr_of!(Guid),
            core::ptr::addr_of!(KSNODETYPE_SPDIF_INTERFACE),
        ) {
            /* type analog */
            if BridgePin != 0 {
                *TargetType = MIXERLINE_TARGETTYPE_WAVEIN as ULONG;
            } else {
                *TargetType = MIXERLINE_TARGETTYPE_WAVEOUT as ULONG;
            }

            *ComponentType = MIXERLINE_COMPONENTTYPE_SRC_DIGITAL as ULONG;
        } else {
            /* unknown type */
            *TargetType = MIXERLINE_TARGETTYPE_UNDEFINED as ULONG;
            *ComponentType = MIXERLINE_COMPONENTTYPE_SRC_UNDEFINED as ULONG;
            DPRINT1!("Unknown Category for PinId %lu BridgePin %lu\n", PinId, BridgePin as core::ffi::c_uint);
        }
    } else {
        if IsEqualGUIDAligned(
            core::ptr::addr_of!(Guid),
            core::ptr::addr_of!(KSNODETYPE_SPEAKER),
        ) || IsEqualGUIDAligned(
            core::ptr::addr_of!(Guid),
            core::ptr::addr_of!(KSNODETYPE_DESKTOP_SPEAKER),
        ) || IsEqualGUIDAligned(
            core::ptr::addr_of!(Guid),
            core::ptr::addr_of!(KSNODETYPE_ROOM_SPEAKER),
        ) || IsEqualGUIDAligned(
            core::ptr::addr_of!(Guid),
            core::ptr::addr_of!(KSNODETYPE_COMMUNICATION_SPEAKER),
        ) {
            /* type waveout */
            *TargetType = MIXERLINE_TARGETTYPE_WAVEOUT as ULONG;
            *ComponentType = MIXERLINE_COMPONENTTYPE_DST_SPEAKERS as ULONG;
        } else if IsEqualGUIDAligned(
            core::ptr::addr_of!(Guid),
            core::ptr::addr_of!(KSCATEGORY_AUDIO),
        ) || IsEqualGUIDAligned(
            core::ptr::addr_of!(Guid),
            core::ptr::addr_of!(PINNAME_CAPTURE),
        ) {
            /* type wavein */
            *TargetType = MIXERLINE_TARGETTYPE_WAVEIN as ULONG;
            *ComponentType = MIXERLINE_COMPONENTTYPE_DST_WAVEIN as ULONG;
        } else if IsEqualGUIDAligned(
            core::ptr::addr_of!(Guid),
            core::ptr::addr_of!(KSNODETYPE_HEADPHONES),
        ) || IsEqualGUIDAligned(
            core::ptr::addr_of!(Guid),
            core::ptr::addr_of!(KSNODETYPE_HEAD_MOUNTED_DISPLAY_AUDIO),
        ) {
            /* type head phones */
            *TargetType = MIXERLINE_TARGETTYPE_WAVEOUT as ULONG;
            *ComponentType = MIXERLINE_COMPONENTTYPE_DST_HEADPHONES as ULONG;
        } else if IsEqualGUIDAligned(
            core::ptr::addr_of!(Guid),
            core::ptr::addr_of!(KSNODETYPE_TELEPHONE),
        ) || IsEqualGUIDAligned(
            core::ptr::addr_of!(Guid),
            core::ptr::addr_of!(KSNODETYPE_PHONE_LINE),
        ) || IsEqualGUIDAligned(
            core::ptr::addr_of!(Guid),
            core::ptr::addr_of!(KSNODETYPE_DOWN_LINE_PHONE),
        ) {
            /* type waveout */
            *TargetType = MIXERLINE_TARGETTYPE_UNDEFINED as ULONG;
            *ComponentType = MIXERLINE_COMPONENTTYPE_DST_TELEPHONE as ULONG;
        } else if IsEqualGUIDAligned(
            core::ptr::addr_of!(Guid),
            core::ptr::addr_of!(KSNODETYPE_ANALOG_CONNECTOR),
        ) {
            /* type analog */
            if BridgePin != 0 {
                *TargetType = MIXERLINE_TARGETTYPE_WAVEOUT as ULONG;
                *ComponentType = MIXERLINE_COMPONENTTYPE_DST_SPEAKERS as ULONG;
            } else {
                *TargetType = MIXERLINE_TARGETTYPE_WAVEIN as ULONG;
                *ComponentType = MIXERLINE_COMPONENTTYPE_DST_WAVEIN as ULONG;
            }
        } else if IsEqualGUIDAligned(
            core::ptr::addr_of!(Guid),
            core::ptr::addr_of!(KSNODETYPE_SPDIF_INTERFACE),
        ) {
            /* type spdif */
            if BridgePin != 0 {
                *TargetType = MIXERLINE_TARGETTYPE_WAVEOUT as ULONG;
                *ComponentType = MIXERLINE_COMPONENTTYPE_DST_SPEAKERS as ULONG;
            } else {
                *TargetType = MIXERLINE_TARGETTYPE_WAVEIN as ULONG;
                *ComponentType = MIXERLINE_COMPONENTTYPE_DST_WAVEIN as ULONG;
            }
        } else {
            /* unknown type */
            *TargetType = MIXERLINE_TARGETTYPE_UNDEFINED as ULONG;
            *ComponentType = MIXERLINE_COMPONENTTYPE_DST_UNDEFINED as ULONG;
            DPRINT1!("Unknown Category for PinId %lu BridgePin %lu\n", PinId, BridgePin as core::ffi::c_uint);
        }
    }

    /* done */
    return MM_STATUS_SUCCESS as MIXER_STATUS;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerBuildMixerSourceLine(
    mut MixerContext: PMIXER_CONTEXT,
    mut MixerInfo: LPMIXER_INFO,
    mut hMixer: HANDLE,
    mut Topology: PTOPOLOGY,
    mut PinId: ULONG,
    mut NodesCount: ULONG,
    mut Nodes: PULONG,
    mut DestinationLineID: ULONG,
    mut OutSrcLine: *mut LPMIXERLINE_EXT,
) -> MIXER_STATUS {
    let mut SrcLine: LPMIXERLINE_EXT = core::ptr::null_mut();
    let mut DstLine: LPMIXERLINE_EXT = core::ptr::null_mut();
    let mut PinName: LPWSTR = core::ptr::null_mut();
    let mut Status: MIXER_STATUS = 0;
    let mut ComponentType: ULONG = 0;
    let mut TargetType: ULONG = 0;

    /* get component and target type */
    Status = MMixerGetComponentAndTargetType(
        MixerContext,
        MixerInfo,
        hMixer,
        PinId,
        core::ptr::addr_of_mut!(ComponentType),
        core::ptr::addr_of_mut!(TargetType),
    );
    if Status != MM_STATUS_SUCCESS {
        /* failed to get component status */
        TargetType = MIXERLINE_TARGETTYPE_UNDEFINED as ULONG;
        ComponentType = MIXERLINE_COMPONENTTYPE_DST_UNDEFINED as ULONG;
    }

    /* construct source line */
    SrcLine = (*MixerContext).Alloc.unwrap_unchecked()(
        core::mem::size_of::<MIXERLINE_EXT>() as ULONG,
    ) as LPMIXERLINE_EXT;

    if SrcLine.is_null() {
        /* no memory */
        return MM_STATUS_NO_MEMORY;
    }

    /* get destination line */
    DstLine = MMixerGetSourceMixerLineByLineId(MixerInfo, DestinationLineID as DWORD);
    ASSERT!(!DstLine.is_null());

    /* initialize mixer src line */
    (*SrcLine).PinId = PinId;

    /* initialize mixer line */
    (*SrcLine).Line.cbStruct = core::mem::size_of::<MIXERLINEW>() as DWORD;
    (*SrcLine).Line.dwDestination =
        (*MixerInfo).MixCaps.cDestinations.wrapping_sub(1 as DWORD);
    (*SrcLine).Line.dwSource = (*DstLine).Line.cConnections;
    (*SrcLine).Line.dwLineID = (*DstLine)
        .Line
        .cConnections
        .wrapping_mul(SOURCE_LINE as DWORD)
        .wrapping_add(
            (*MixerInfo)
                .MixCaps
                .cDestinations
                .wrapping_sub(1 as DWORD),
        );
    (*SrcLine).Line.fdwLine =
        (MIXERLINE_LINEF_ACTIVE as DWORD) | (MIXERLINE_LINEF_SOURCE as DWORD);
    (*SrcLine).Line.dwComponentType = ComponentType as DWORD;
    (*SrcLine).Line.dwUser = 0 as DWORD_PTR;
    (*SrcLine).Line.cChannels = (*DstLine).Line.cChannels;
    (*SrcLine).Line.cConnections = 0 as DWORD;
    (*SrcLine).Line.Target.dwType = TargetType as DWORD;
    (*SrcLine).Line.Target.dwDeviceID = (*DstLine).Line.Target.dwDeviceID;
    (*SrcLine).Line.Target.wMid = (*MixerInfo).MixCaps.wMid;
    (*SrcLine).Line.Target.wPid = (*MixerInfo).MixCaps.wPid;
    (*SrcLine).Line.Target.vDriverVersion = (*MixerInfo).MixCaps.vDriverVersion;
    InitializeListHead(core::ptr::addr_of_mut!((*SrcLine).ControlsList));

    /* copy name */
    ASSERT!(
        (*MixerInfo).MixCaps.szPname
            [(MAXPNAMELEN - 1) as usize]
            == 0x0000u16
    );
    wcscpy(
        core::ptr::addr_of_mut!((*SrcLine).Line.Target.szPname) as *mut wchar_t,
        core::ptr::addr_of_mut!((*MixerInfo).MixCaps.szPname) as *mut WCHAR,
    );

    /* get pin name */
    Status = MMixerGetPinName(
        MixerContext,
        MixerInfo,
        hMixer,
        PinId,
        core::ptr::addr_of_mut!(PinName),
    );

    if Status == MM_STATUS_SUCCESS {
        /* store pin name as line name */
        (*MixerContext).Copy_0.unwrap_unchecked()(
            core::ptr::addr_of_mut!((*SrcLine).Line.szShortName) as PVOID,
            PinName as PVOID,
            (min(
                MIXER_SHORT_NAME_CHARS,
                (wcslen(PinName as *const wchar_t) + 1) as _,
            ) * (core::mem::size_of::<WCHAR>()) as _) as ULONG,
        );
        (*SrcLine).Line.szShortName
            [(MIXER_SHORT_NAME_CHARS - 1) as usize] = 0x0000u16;

        (*MixerContext).Copy_0.unwrap_unchecked()(
            core::ptr::addr_of_mut!((*SrcLine).Line.szName) as PVOID,
            PinName as PVOID,
            (min(
                MIXER_LONG_NAME_CHARS,
                (wcslen(PinName as *const wchar_t) + 1) as _,
            ) * (core::mem::size_of::<WCHAR>()) as _) as ULONG,
        );
        (*SrcLine).Line.szName[(MIXER_LONG_NAME_CHARS - 1) as usize] =
            0x0000u16;

        /* free pin name buffer */
        (*MixerContext).Free.unwrap_unchecked()(PinName as PVOID);
    }

    /* add the controls to mixer line */
    Status = MMixerAddMixerControlsToMixerLineByNodeIndexArray(
        MixerContext,
        MixerInfo,
        hMixer,
        Topology,
        SrcLine,
        NodesCount,
        Nodes,
    );
    if Status != MM_STATUS_SUCCESS {
        /* failed */
        return Status;
    }

    /* store result */
    *OutSrcLine = SrcLine;

    return MM_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerAddMixerSourceLines(
    mut MixerContext: PMIXER_CONTEXT,
    mut MixerInfo: LPMIXER_INFO,
    mut hMixer: HANDLE,
    mut Topology: PTOPOLOGY,
    mut DestinationLineID: ULONG,
    mut LineTerminator: ULONG,
) -> MIXER_STATUS {
    let mut AllNodes: PULONG = ::core::ptr::null_mut::<::core::ffi::c_ulong>();
    let mut AllPins: PULONG = ::core::ptr::null_mut::<::core::ffi::c_ulong>();
    let mut AllPinNodes: PULONG = ::core::ptr::null_mut::<::core::ffi::c_ulong>();
    let mut AllNodesCount: ULONG = 0;
    let mut AllPinsCount: ULONG = 0;
    let mut AllPinNodesCount: ULONG = 0;
    let mut Index: ULONG = 0;
    let mut SubIndex: ULONG = 0;
    let mut PinId: ULONG = 0;
    let mut CurNode: ULONG = 0;
    let mut bConnected: ULONG = 0;
    let mut Status: MIXER_STATUS = MM_STATUS_SUCCESS;
    let mut DstLine: LPMIXERLINE_EXT = ::core::ptr::null_mut::<MIXERLINE_EXT>();
    let mut SrcLine: LPMIXERLINE_EXT = ::core::ptr::null_mut::<MIXERLINE_EXT>();

    /* get destination line */
    DstLine = MMixerGetSourceMixerLineByLineId(MixerInfo, DestinationLineID as DWORD);
    ASSERT!(!DstLine.is_null());

    /* allocate an array to store all nodes which are upstream of the line terminator */
    Status = MMixerAllocateTopologyNodeArray(MixerContext, Topology, &raw mut AllNodes);

    /* check for success */
    if Status != MM_STATUS_SUCCESS {
        /* out of memory */
        return MM_STATUS_NO_MEMORY;
    }

    /* allocate an array to store all nodes which are downstream of a particular pin */
    Status = MMixerAllocateTopologyNodeArray(MixerContext, Topology, &raw mut AllPinNodes);

    /* allocate an array to store all pins which are upstream of this pin */
    Status = MMixerAllocateTopologyPinArray(MixerContext, Topology, &raw mut AllPins);

    /* check for success */
    if Status != MM_STATUS_SUCCESS {
        /* out of memory */
        (*MixerContext).Free.unwrap_unchecked()(AllNodes as PVOID);
        return MM_STATUS_NO_MEMORY;
    }

    /* get all nodes which indirectly / directly connect to this node */
    AllNodesCount = 0;
    MMixerGetAllUpOrDownstreamNodesFromNodeIndex(
        MixerContext,
        Topology,
        LineTerminator,
        (TRUE) as ULONG,
        &raw mut AllNodesCount,
        AllNodes,
    );

    /* get all pins which indirectly / directly connect to this node */
    AllPinsCount = 0;
    MMixerGetAllUpOrDownstreamPinsFromNodeIndex(
        MixerContext,
        Topology,
        LineTerminator,
        (TRUE) as ULONG,
        &raw mut AllPinsCount,
        AllPins,
    );

    DPRINT!("LineTerminator %lu\n", LineTerminator);
    DPRINT!("PinCount %lu\n", AllPinsCount);
    DPRINT!("AllNodesCount %lu\n", AllNodesCount);

    /* now construct the source lines which are attached to the destination line */
    Index = AllPinsCount;

    loop {
        /* get current pin id */
        PinId = *AllPins.offset(Index.wrapping_sub(1) as isize) as ULONG;

        /* reset nodes count */
        AllPinNodesCount = 0;

        /* now scan all nodes and add them to AllPinNodes array when they are connected to this pin */
        SubIndex = 0;
        while SubIndex < AllNodesCount {
            /* get current node index */
            CurNode = *AllNodes.offset(SubIndex as isize) as ULONG;

            if CurNode != MAXULONG && CurNode != LineTerminator {
                /* check if that node is connected in some way to the current pin */
                Status = MMixerIsNodeConnectedToPin(
                    MixerContext,
                    Topology,
                    CurNode,
                    PinId,
                    (TRUE) as ULONG,
                    &raw mut bConnected,
                );

                if Status != MM_STATUS_SUCCESS {
                    break;
                }

                if bConnected != 0 {
                    /* it is connected */
                    *AllPinNodes.offset(AllPinNodesCount as isize) = CurNode as ::core::ffi::c_ulong;
                    AllPinNodesCount = AllPinNodesCount.wrapping_add(1);

                    /* clear current index */
                    *AllNodes.offset(SubIndex as isize) = MAXULONG as ::core::ffi::c_ulong;
                }
            }
            SubIndex = SubIndex.wrapping_add(1);
        }

        /* decrement pin index */
        Index = Index.wrapping_sub(1);

        if AllPinNodesCount != 0 {
            /* now build the mixer source line */
            Status = MMixerBuildMixerSourceLine(
                MixerContext,
                MixerInfo,
                hMixer,
                Topology,
                PinId,
                AllPinNodesCount,
                AllPinNodes,
                DestinationLineID,
                &raw mut SrcLine,
            );

            if Status == MM_STATUS_SUCCESS {
                /* insert into line list */
                InsertTailList(&raw mut (*MixerInfo).LineList, &raw mut (*SrcLine).Entry);

                /* increment destination line count */
                (*DstLine).Line.cConnections = (*DstLine).Line.cConnections.wrapping_add(1);

                /* mark pin as reserved */
                MMixerSetTopologyPinReserved(Topology, PinId);
            }
        } else {
            DPRINT1!(
                "Discarding DestinationLineID %lu PinId %lu NO NODES!\n",
                DestinationLineID,
                PinId
            );
        }

        if Index == 0 {
            break;
        }
    }

    return MM_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerAddMixerControlsToDestinationLine(
    mut MixerContext: PMIXER_CONTEXT,
    mut MixerInfo: LPMIXER_INFO,
    mut hMixer: HANDLE,
    mut Topology: PTOPOLOGY,
    mut PinId: ULONG,
    mut bInput: ULONG,
    mut DestinationLineId: ULONG,
    mut OutLineTerminator: PULONG,
) -> MIXER_STATUS {
    let mut Nodes: PULONG = core::ptr::null_mut();
    let mut NodesCount: ULONG = 0;
    let mut LineTerminator: ULONG = 0;
    let mut Status: MIXER_STATUS;
    let mut DstLine: LPMIXERLINE_EXT;

    /* allocate nodes index array */
    Status = MMixerAllocateTopologyNodeArray(MixerContext, Topology, core::ptr::addr_of_mut!(Nodes));

    /* check for success */
    if Status != MM_STATUS_SUCCESS {
        /* out of memory */
        return Status;
    }

    /* get all destination line controls */
    Status = MMixerCountMixerControls(
        MixerContext,
        Topology,
        PinId,
        bInput,
        (TRUE) as ULONG,
        core::ptr::addr_of_mut!(NodesCount),
        Nodes,
        core::ptr::addr_of_mut!(LineTerminator),
    );

    /* check for success */
    if Status != MM_STATUS_SUCCESS {
        /* failed to count controls */
        (*MixerContext).Free.unwrap_unchecked()(Nodes as PVOID);
        return Status;
    }

    /* get destination mixer line */
    DstLine = MMixerGetSourceMixerLineByLineId(MixerInfo, DestinationLineId as DWORD);

    /* sanity check */
    ASSERT!(DstLine);

    if NodesCount > 0 {
        /* add all nodes as mixer controls to the destination line */
        Status = MMixerAddMixerControlsToMixerLineByNodeIndexArray(
            MixerContext,
            MixerInfo,
            hMixer,
            Topology,
            DstLine,
            NodesCount,
            Nodes,
        );
        if Status != MM_STATUS_SUCCESS {
            /* failed to add controls */
            (*MixerContext).Free.unwrap_unchecked()(Nodes as PVOID);
            return Status;
        }
    }

    /* store result */
    *OutLineTerminator = LineTerminator;

    /* return result */
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerApplyOutputFilterHack(
    mut MixerContext: PMIXER_CONTEXT,
    mut MixerData: LPMIXER_DATA,
    mut hMixer: HANDLE,
    mut PinsCount: PULONG,
    mut Pins: PULONG,
) {
    let mut Count: ULONG = 0;
    let mut Index: ULONG;
    let mut Status: MIXER_STATUS;
    let mut Connection: PKSPIN_PHYSICALCONNECTION = core::ptr::null_mut();

    Index = 0;
    while Index < *PinsCount {
        /* check if it has a physical connection */
        Status = MMixerGetPhysicalConnection(
            MixerContext,
            hMixer,
            *Pins.add(Index as usize),
            core::ptr::addr_of_mut!(Connection),
        );

        if Status == MM_STATUS_SUCCESS {
            /* remove pin */
            (((*MixerContext).Copy).unwrap_unchecked())(
                Pins.add(Index as usize) as PVOID,
                Pins.add(Index.wrapping_add(1) as usize) as PVOID,
                ((*PinsCount).wrapping_sub(Index.wrapping_add(1)) as usize
                    * core::mem::size_of::<ULONG>()) as ULONG,
            );

            /* free physical connection */
            (((*MixerContext).Free).unwrap_unchecked())(Connection as PVOID);

            /* decrement index */
            Index = Index.wrapping_sub(1);

            /* decrement pin count */
            *PinsCount = (*PinsCount).wrapping_sub(1);
        } else {
            /* simple pin */
            Count = Count.wrapping_add(1);
        }
        Index = Index.wrapping_add(1);
    }

    /* store result */
    *PinsCount = Count;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerHandleTopologyFilter(
    MixerContext: PMIXER_CONTEXT,
    MixerList: PMIXER_LIST,
    MixerData: LPMIXER_DATA,
    MixerInfo: LPMIXER_INFO,
    bInput: ULONG,
    Pin: ULONG,
) -> MIXER_STATUS {
    let mut Status: MIXER_STATUS;
    let mut PinsCount: ULONG;
    let mut LineTerminator: ULONG = core::mem::zeroed();
    let mut DestinationLineID: ULONG;
    let mut Pins: PULONG = core::ptr::null_mut();
    let mut Topology: PTOPOLOGY;

    /* re-use existing topology */
    Topology = (*MixerData).Topology;

    if bInput == 0 {
        /* allocate pin index array which will hold all referenced pins */
        Status = MMixerAllocateTopologyPinArray(MixerContext, Topology, core::ptr::addr_of_mut!(Pins));
        if Status != MM_STATUS_SUCCESS {
            /* failed to create topology */
            return Status;
        }

        /* the mixer is an output mixer
        * find end pin of the node path
        */
        PinsCount = 0;
        Status = MMixerGetAllUpOrDownstreamPinsFromPinIndex(
            MixerContext,
            Topology,
            Pin,
            (FALSE) as ULONG,
            core::ptr::addr_of_mut!(PinsCount),
            Pins,
        );

        /* check for success */
        if Status != MM_STATUS_SUCCESS {
            /* failed to get end pin */
            (*MixerContext).Free.unwrap_unchecked()(Pins as PVOID);
            //MMixerFreeTopology(Topology);

            /* return error code */
            return Status;
        }
        /* HACK:
        * some topologies do not have strict boundaries
        * WorkArround: remove all pin ids which have a physical connection
        * because bridge pins may belong to different render paths
        */
        MMixerApplyOutputFilterHack(
            MixerContext,
            MixerData,
            (*MixerData).hDevice,
            core::ptr::addr_of_mut!(PinsCount),
            Pins,
        );

        /* sanity checks */
        ASSERT!(PinsCount != 0);
        if PinsCount != 1 {
            DPRINT1!("MMixerHandlePhysicalConnection Expected 1 pin but got %lu\n", PinsCount);
        }

        /* create destination line */
        Status = MMixerBuildMixerDestinationLine(
            MixerContext,
            MixerInfo,
            (*MixerData).hDevice,
            *Pins.offset(0 as isize) as ULONG,
            bInput,
        );

        /* calculate destination line id */
        DestinationLineID = (DESTINATION_LINE as ULONG)
            .wrapping_add((*MixerInfo).MixCaps.cDestinations)
            .wrapping_sub(1 as ULONG);

        if Status != MM_STATUS_SUCCESS {
            /* failed to build destination line */
            (*MixerContext).Free.unwrap_unchecked()(Pins as PVOID);

            /* return error code */
            return Status;
        }

        /* add mixer controls to destination line */
        Status = MMixerAddMixerControlsToDestinationLine(
            MixerContext,
            MixerInfo,
            (*MixerData).hDevice,
            Topology,
            *Pins.offset(0 as isize) as ULONG,
            bInput,
            DestinationLineID,
            core::ptr::addr_of_mut!(LineTerminator),
        );

        if Status == MM_STATUS_SUCCESS {
            /* now add the rest of the source lines */
            Status = MMixerAddMixerSourceLines(
                MixerContext,
                MixerInfo,
                (*MixerData).hDevice,
                Topology,
                DestinationLineID,
                LineTerminator,
            );
        }

        /* mark pin as consumed */
        MMixerSetTopologyPinReserved(
            Topology,
            *Pins.offset(0 as isize) as ULONG,
        );

        /* free topology pin array */
        (*MixerContext).Free.unwrap_unchecked()(Pins as PVOID);
    } else {
        /* calculate destination line id */
        DestinationLineID = (DESTINATION_LINE as ULONG)
            .wrapping_add((*MixerInfo).MixCaps.cDestinations)
            .wrapping_sub(1 as ULONG);

        /* add mixer controls */
        Status = MMixerAddMixerControlsToDestinationLine(
            MixerContext,
            MixerInfo,
            (*MixerData).hDevice,
            Topology,
            Pin,
            bInput,
            DestinationLineID,
            core::ptr::addr_of_mut!(LineTerminator),
        );

        if Status == MM_STATUS_SUCCESS {
            /* now add the rest of the source lines */
            Status = MMixerAddMixerSourceLines(
                MixerContext,
                MixerInfo,
                (*MixerData).hDevice,
                Topology,
                DestinationLineID,
                LineTerminator,
            );
        }
    }
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerHandlePhysicalConnection(mut MixerContext: PMIXER_CONTEXT, mut MixerList: PMIXER_LIST, mut MixerData: LPMIXER_DATA, mut MixerInfo: LPMIXER_INFO, mut bInput: ULONG, mut OutConnection: PKSPIN_PHYSICALCONNECTION) -> MIXER_STATUS {
    let mut Status: MIXER_STATUS;
    let mut PinsCount: ULONG;
    let mut LineTerminator: ULONG = core::mem::zeroed();
    let mut DestinationLineID: ULONG;
    let mut Pins: PULONG = core::mem::zeroed();
    let mut Topology: PTOPOLOGY = core::mem::zeroed();

    /* first try to open the connected filter */
    (*core::ptr::addr_of_mut!((*OutConnection).SymbolicLinkName[0]).add((1) as usize)) = 0x005Cu16;
    MixerData = MMixerGetDataByDeviceName(
        MixerList,
        &raw mut (*OutConnection).SymbolicLinkName as LPWSTR,
    );

    /* check if the linked connection is found */
    if MixerData.is_null() {
        /* filter references invalid physical connection */
        return MM_STATUS_UNSUCCESSFUL;
    }

    DPRINT!(
        "Name %S, Pin %lu bInput %lu\n",
        &raw mut (*OutConnection).SymbolicLinkName as LPWSTR,
        (*OutConnection).Pin,
        bInput
    );

    /* sanity check */
    ASSERT!((*MixerData).MixerInfo.is_null() || (*MixerData).MixerInfo == MixerInfo);

    /* associate with mixer */
    (*MixerData).MixerInfo = MixerInfo;

    if (*MixerData).Topology.is_null() {
        /* construct new topology */
        Status = MMixerBuildTopology(MixerContext, MixerData, &raw mut Topology);
        if Status != MM_STATUS_SUCCESS {
            /* failed to create topology */
            return Status;
        }

        /* store topology */
        (*MixerData).Topology = Topology;
    } else {
        /* re-use existing topology */
        Topology = (*MixerData).Topology;
    }

    /* mark pin as consumed */
    MMixerSetTopologyPinReserved(Topology, (*OutConnection).Pin);

    if bInput == 0 {
        /* allocate pin index array which will hold all referenced pins */
        Status = MMixerAllocateTopologyPinArray(MixerContext, Topology, &raw mut Pins);
        if Status != MM_STATUS_SUCCESS {
            /* failed to create topology */
            return Status;
        }

        /* the mixer is an output mixer
         * find end pin of the node path
         */
        PinsCount = 0;
        Status = MMixerGetAllUpOrDownstreamPinsFromPinIndex(
            MixerContext,
            Topology,
            (*OutConnection).Pin,
            (FALSE) as ULONG,
            &raw mut PinsCount,
            Pins,
        );

        /* check for success */
        if Status != MM_STATUS_SUCCESS {
            /* failed to get end pin */
            (*MixerContext).Free.unwrap_unchecked()(Pins as PVOID);
            //MMixerFreeTopology(Topology);

            /* return error code */
            return Status;
        }
        /* HACK:
         * some topologies do not have strict boundaries
         * WorkArround: remove all pin ids which have a physical connection
         * because bridge pins may belong to different render paths
         */
        MMixerApplyOutputFilterHack(
            MixerContext,
            MixerData,
            (*MixerData).hDevice,
            &raw mut PinsCount,
            Pins,
        );

        /* sanity checks */
        ASSERT!(PinsCount != 0);
        if PinsCount != 1 {
            DPRINT1!("MMixerHandlePhysicalConnection Expected 1 pin but got %lu\n", PinsCount);
        }

        /* create destination line */
        Status = MMixerBuildMixerDestinationLine(
            MixerContext,
            MixerInfo,
            (*MixerData).hDevice,
            *Pins.offset(0 as isize) as ULONG,
            bInput,
        );

        /* calculate destination line id */
        DestinationLineID = (DESTINATION_LINE as DWORD)
            .wrapping_add((*MixerInfo).MixCaps.cDestinations)
            .wrapping_sub(1 as DWORD) as ULONG;

        if Status != MM_STATUS_SUCCESS {
            /* failed to build destination line */
            (*MixerContext).Free.unwrap_unchecked()(Pins as PVOID);

            /* return error code */
            return Status;
        }

        /* add mixer controls to destination line */
        Status = MMixerAddMixerControlsToDestinationLine(
            MixerContext,
            MixerInfo,
            (*MixerData).hDevice,
            Topology,
            *Pins.offset(0 as isize) as ULONG,
            bInput,
            DestinationLineID,
            &raw mut LineTerminator,
        );

        if Status == MM_STATUS_SUCCESS {
            /* now add the rest of the source lines */
            Status = MMixerAddMixerSourceLines(
                MixerContext,
                MixerInfo,
                (*MixerData).hDevice,
                Topology,
                DestinationLineID,
                LineTerminator,
            );
        }

        /* mark pin as consumed */
        MMixerSetTopologyPinReserved(
            Topology,
            *Pins.offset(0 as isize) as ULONG,
        );

        /* free topology pin array */
        (*MixerContext).Free.unwrap_unchecked()(Pins as PVOID);
    } else {
        /* calculate destination line id */
        DestinationLineID = (DESTINATION_LINE as DWORD)
            .wrapping_add((*MixerInfo).MixCaps.cDestinations)
            .wrapping_sub(1 as DWORD) as ULONG;

        /* add mixer controls */
        Status = MMixerAddMixerControlsToDestinationLine(
            MixerContext,
            MixerInfo,
            (*MixerData).hDevice,
            Topology,
            (*OutConnection).Pin,
            bInput,
            DestinationLineID,
            &raw mut LineTerminator,
        );

        if Status == MM_STATUS_SUCCESS {
            /* now add the rest of the source lines */
            Status = MMixerAddMixerSourceLines(
                MixerContext,
                MixerInfo,
                (*MixerData).hDevice,
                Topology,
                DestinationLineID,
                LineTerminator,
            );
        }
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerInitializeFilter(
    MixerContext: PMIXER_CONTEXT,
    MixerList: PMIXER_LIST,
    MixerData: LPMIXER_DATA,
    mut MixerInfo: LPMIXER_INFO,
    Topology: PTOPOLOGY,
    NodeIndex: ULONG,
    bInputMixer: ULONG,
    OutMixerInfo: *mut LPMIXER_INFO,
) -> MIXER_STATUS {
    let mut Index: ULONG;
    let mut Status: MIXER_STATUS;
    let mut OutConnection: PKSPIN_PHYSICALCONNECTION = core::mem::zeroed();
    let mut Pins: *mut ULONG = core::mem::zeroed();
    let mut PinsFound: ULONG;
    let mut NewMixerInfo: ULONG = (FALSE as u32);

    if MixerInfo.is_null() {
        /* allocate a mixer info struct */
        MixerInfo = (*MixerContext).Alloc.unwrap_unchecked()(
            core::mem::size_of::<MIXER_INFO>() as ULONG,
        ) as LPMIXER_INFO;
        if MixerInfo.is_null() {
            /* no memory */
            return MM_STATUS_NO_MEMORY;
        }

        /* new mixer info */
        NewMixerInfo = (TRUE as u32);

        /* intialize mixer caps */
        (*MixerInfo).MixCaps.wMid = (MM_MICROSOFT as u16); /* FIXME */
        (*MixerInfo).MixCaps.wPid = (MM_PID_UNMAPPED as u16); /* FIXME */
        (*MixerInfo).MixCaps.vDriverVersion = 1; /* FIXME */
        (*MixerInfo).MixCaps.fdwSupport = 0;
        (*MixerInfo).MixCaps.cDestinations = 0;

        /* get mixer name */
        Status = MMixerGetDeviceName(
            MixerContext,
            core::ptr::addr_of_mut!((*MixerInfo).MixCaps.szPname) as LPWSTR,
            (*MixerData).hDeviceInterfaceKey,
        );
        if Status != MM_STATUS_SUCCESS {
            /* try get name with component id */
            Status = MMixerGetDeviceNameWithComponentId(
                MixerContext,
                (*MixerData).hDevice,
                core::ptr::addr_of_mut!((*MixerInfo).MixCaps.szPname) as LPWSTR,
            );
            if Status != MM_STATUS_SUCCESS {
                (*MixerInfo).MixCaps.szPname[0 as usize] =
                    0x0000u16;
            }
        }

        /* initialize line list */
        InitializeListHead(core::ptr::addr_of_mut!((*MixerInfo).LineList));
        InitializeListHead(core::ptr::addr_of_mut!((*MixerInfo).EventList));

        /* associate with mixer data */
        (*MixerData).MixerInfo = MixerInfo;
    }

    /* store mixer info */
    *OutMixerInfo = MixerInfo;

    /* now allocate an array which will receive the indices of the pin
     * which has a ADC / DAC nodetype in its path
     */
    Status = MMixerAllocateTopologyPinArray(
        MixerContext,
        Topology,
        core::ptr::addr_of_mut!(Pins),
    );
    ASSERT!(Status == MM_STATUS_SUCCESS);

    PinsFound = 0;

    /* now get all sink / source pins, which are attached to the ADC / DAC node
     * For sink pins (wave out) search up stream
     * For source pins (wave in) search down stream
     * The search direction is always the opposite of the current mixer type
     */
    PinsFound = 0;
    MMixerGetAllUpOrDownstreamPinsFromNodeIndex(
        MixerContext,
        Topology,
        NodeIndex,
        (bInputMixer == 0) as ULONG,
        core::ptr::addr_of_mut!(PinsFound),
        Pins,
    );

    /* if there is no pin found, we have a broken topology */
    ASSERT!(PinsFound != 0);

    /* now create a wave info struct */
    Status = MMixerInitializeWaveInfo(
        MixerContext,
        MixerList,
        MixerData,
        core::ptr::addr_of_mut!((*MixerInfo).MixCaps.szPname) as LPWSTR,
        bInputMixer,
        PinsFound,
        Pins,
    );
    if Status != MM_STATUS_SUCCESS {
        /* failed to create wave info struct */
        (*MixerContext).Free.unwrap_unchecked()(MixerInfo as PVOID);
        (*MixerContext).Free.unwrap_unchecked()(Pins as PVOID);
        return Status;
    }

    /* mark all found pins as reserved */
    Index = 0;
    while Index < PinsFound {
        MMixerSetTopologyPinReserved(Topology, *Pins.add(Index as usize));
        Index = Index.wrapping_add(1);
    }

    if bInputMixer != 0 {
        /* pre create the mixer destination line for input mixers */
        Status = MMixerBuildMixerDestinationLine(
            MixerContext,
            MixerInfo,
            (*MixerData).hDevice,
            *Pins.add(0 as usize),
            bInputMixer,
        );

        if Status != MM_STATUS_SUCCESS {
            /* failed to create mixer destination line */
            return Status;
        }
    }

    /* now get the bridge pin which is at the end of node path
     * For sink pins (wave out) search down stream
     * For source pins (wave in) search up stream
     */
    (*MixerContext).Free.unwrap_unchecked()(Pins as PVOID);
    Status = MMixerAllocateTopologyPinArray(
        MixerContext,
        Topology,
        core::ptr::addr_of_mut!(Pins),
    );
    ASSERT!(Status == MM_STATUS_SUCCESS);

    PinsFound = 0;
    MMixerGetAllUpOrDownstreamPinsFromNodeIndex(
        MixerContext,
        Topology,
        NodeIndex,
        bInputMixer,
        core::ptr::addr_of_mut!(PinsFound),
        Pins,
    );

    /* if there is no pin found, we have a broken topology */
    ASSERT!(PinsFound != 0);

    /* there should be exactly one bridge pin */
    ASSERT!(PinsFound == 1);

    DPRINT!(
        "BridgePin %lu bInputMixer %lu\n",
        *Pins.add(0 as usize),
        bInputMixer
    );

    /* does the pin have a physical connection */
    Status = MMixerGetPhysicalConnection(
        MixerContext,
        (*MixerData).hDevice,
        *Pins.add(0 as usize),
        core::ptr::addr_of_mut!(OutConnection),
    );

    if Status == MM_STATUS_SUCCESS {
        /* mark pin as reserved */
        MMixerSetTopologyPinReserved(Topology, *Pins.add(0 as usize));

        /* topology on the topoloy filter */
        Status = MMixerHandlePhysicalConnection(
            MixerContext,
            MixerList,
            MixerData,
            MixerInfo,
            bInputMixer,
            OutConnection,
        );

        /* free physical connection data */
        (*MixerContext).Free.unwrap_unchecked()(OutConnection as PVOID);
    } else {
        /* topology is on the same filter */
        Status = MMixerHandleTopologyFilter(
            MixerContext,
            MixerList,
            MixerData,
            MixerInfo,
            bInputMixer,
            *Pins.add(0 as usize),
        );
    }

    /* free pins */
    (*MixerContext).Free.unwrap_unchecked()(Pins as PVOID);

    if NewMixerInfo != 0 {
        /* insert mixer */
        InsertHeadList(
            core::ptr::addr_of_mut!((*MixerList).MixerList),
            core::ptr::addr_of_mut!((*MixerInfo).Entry),
        );
        /* increment mixer count */
        (*MixerList).MixerListCount = (*MixerList).MixerListCount.wrapping_add(1);
    }

    /* done */
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerHandleAlternativeMixers(
    MixerContext: PMIXER_CONTEXT,
    MixerList: PMIXER_LIST,
    MixerData: LPMIXER_DATA,
    Topology: PTOPOLOGY,
) {
    let mut Index: ULONG;
    let mut PinCount: ULONG = 0;
    let mut Reserved: ULONG = 0;
    let mut Status: MIXER_STATUS;
    let mut DestinationLineID: ULONG = 0;
    let mut LineTerminator: ULONG = 0;
    let mut DstLine: LPMIXERLINE_EXT;

    DPRINT!("DeviceName %S\n", (*MixerData).DeviceName);

    /* get topology pin count */
    MMixerGetTopologyPinCount(Topology, core::ptr::addr_of_mut!(PinCount));

    Index = 0;
    while Index < PinCount {
        'body: {
            MMixerIsTopologyPinReserved(Topology, Index, core::ptr::addr_of_mut!(Reserved));

            /* check if it has already been reserved */
            if Reserved != 0 {
                /* pin has already been reserved */
                break 'body;
            }

            DPRINT!("MixerName %S Available PinID %lu\n", (*MixerData).DeviceName, Index);

            /* sanity check */
            //ASSERT(MixerData->MixerInfo);

            if (*MixerData).MixerInfo.is_null() {
                DPRINT1!("Expected mixer info\n");
                break 'body;
            }

            /* build the destination line */
            Status = MMixerBuildMixerDestinationLine(
                MixerContext,
                (*MixerData).MixerInfo,
                (*MixerData).hDevice,
                Index,
                (TRUE) as ULONG,
            );
            if Status != MM_STATUS_SUCCESS {
                /* failed to build destination line */
                break 'body;
            }

            /* calculate destination line id */
            DestinationLineID = ((DESTINATION_LINE as ULONG)
                .wrapping_add((*(*MixerData).MixerInfo).MixCaps.cDestinations))
                .wrapping_sub(1 as ULONG);

            /* add mixer controls to destination line */
            Status = MMixerAddMixerControlsToDestinationLine(
                MixerContext,
                (*MixerData).MixerInfo,
                (*MixerData).hDevice,
                (*MixerData).Topology,
                Index,
                (TRUE) as ULONG,
                DestinationLineID,
                core::ptr::addr_of_mut!(LineTerminator),
            );
            if Status == MM_STATUS_SUCCESS {
                /* now add the rest of the source lines */
                Status = MMixerAddMixerSourceLines(
                    MixerContext,
                    (*MixerData).MixerInfo,
                    (*MixerData).hDevice,
                    (*MixerData).Topology,
                    DestinationLineID,
                    LineTerminator,
                );
            }

            /* mark pin as consumed */
            MMixerSetTopologyPinReserved(Topology, Index);

            /* now grab destination line */
            DstLine = MMixerGetSourceMixerLineByLineId((*MixerData).MixerInfo, DestinationLineID);

            /* set type and target as undefined */
            (*DstLine).Line.dwComponentType = MIXERLINE_COMPONENTTYPE_DST_UNDEFINED;
            (*DstLine).Line.Target.dwType = MIXERLINE_TARGETTYPE_UNDEFINED;
            (*DstLine).Line.Target.vDriverVersion = 0;
            (*DstLine).Line.Target.wMid = 0;
            (*DstLine).Line.Target.wPid = 0;
        }
        Index = Index.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn MMixerSetupFilter(
    MixerContext: PMIXER_CONTEXT,
    MixerList: PMIXER_LIST,
    MixerData: LPMIXER_DATA,
    DeviceCount: PULONG,
) -> MIXER_STATUS {
    let mut Status: MIXER_STATUS = MM_STATUS_SUCCESS;
    let mut Topology: PTOPOLOGY = core::ptr::null_mut();
    let mut NodeIndex: ULONG;
    let mut MixerInfo: LPMIXER_INFO = core::ptr::null_mut();

    /* check if topology has already been built */
    if (*MixerData).Topology.is_null() {
        /* build topology */
        Status = MMixerBuildTopology(MixerContext, MixerData, core::ptr::addr_of_mut!(Topology));

        if Status != MM_STATUS_SUCCESS {
            /* failed to build topology */
            return Status;
        }

        /* store topology */
        (*MixerData).Topology = Topology;
    } else {
        /* re-use topology */
        Topology = (*MixerData).Topology;
    }

    /* check if the filter has an wave out node */
    NodeIndex = MMixerGetNodeIndexFromGuid(Topology, &KSNODETYPE_DAC);
    if NodeIndex != MAXULONG {
        /* it has */
        Status = MMixerInitializeFilter(
            MixerContext,
            MixerList,
            MixerData,
            core::ptr::null_mut(),
            Topology,
            NodeIndex,
            (FALSE) as ULONG,
            core::ptr::addr_of_mut!(MixerInfo),
        );

        /* check for success */
        if Status == MM_STATUS_SUCCESS {
            /* increment mixer count */
            *DeviceCount = (*DeviceCount).wrapping_add(1);
        } else {
            /* reset mixer info in case of error */
            MixerInfo = core::ptr::null_mut();
        }
    }

    /* check if the filter has an wave in node */
    NodeIndex = MMixerGetNodeIndexFromGuid(Topology, &KSNODETYPE_ADC);
    if NodeIndex != MAXULONG {
        /* it has */
        Status = MMixerInitializeFilter(
            MixerContext,
            MixerList,
            MixerData,
            MixerInfo,
            Topology,
            NodeIndex,
            (TRUE) as ULONG,
            core::ptr::addr_of_mut!(MixerInfo),
        );

        /* check for success */
        if Status == MM_STATUS_SUCCESS {
            /* increment mixer count */
            *DeviceCount = (*DeviceCount).wrapping_add(1);
        }
    }

    /* TODO: apply hacks for Wave source line */

    /* activate midi devices */
    //MMixerInitializeMidiForFilter(MixerContext, MixerList, MixerData, Topology);

    /* done */
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerAddEvent(
    MixerContext: PMIXER_CONTEXT,
    MixerInfo: LPMIXER_INFO,
    MixerEventContext: PVOID,
    MixerEventRoutine: PMIXER_EVENT,
) -> MIXER_STATUS {
    let mut EventNotification: PEVENT_NOTIFICATION_ENTRY = core::ptr::null_mut();

    EventNotification = (*MixerContext).Alloc.unwrap_unchecked()(
        core::mem::size_of::<EVENT_NOTIFICATION_ENTRY>() as ULONG,
    ) as PEVENT_NOTIFICATION_ENTRY;
    if EventNotification.is_null() {
        /* not enough memory */
        return MM_STATUS_NO_MEMORY;
    }

    /* FIXME: what is it supposed to happen with KSEVENTDATA ? */

    /* initialize notification entry */
    (*EventNotification).MixerEventContext = MixerEventContext;
    (*EventNotification).MixerEventRoutine = MixerEventRoutine;

    /* store event */
    InsertTailList(
        &raw mut (*MixerInfo).EventList,
        &raw mut (*EventNotification).Entry,
    );
    return MM_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerRemoveEvent(
    MixerContext: PMIXER_CONTEXT,
    MixerInfo: LPMIXER_INFO,
    MixerEventContext: PVOID,
    MixerEventRoutine: PMIXER_EVENT,
) -> MIXER_STATUS {
    let mut EventList: PLIST_ENTRY = core::ptr::null_mut();
    let mut NotificationEntry: PEVENT_NOTIFICATION_ENTRY = core::ptr::null_mut();

    /* Lookup through mixers */
    EventList = (*MixerInfo).EventList.Flink;
    while EventList != core::ptr::addr_of_mut!((*MixerInfo).EventList) {
        NotificationEntry = CONTAINING_RECORD!(EventList, EVENT_NOTIFICATION_ENTRY, Entry);
        EventList = (*EventList).Flink;
        /* TODO: find a better way to identify an event ? */
        if (*NotificationEntry).MixerEventRoutine == MixerEventRoutine
            && (*NotificationEntry).MixerEventContext == MixerEventContext
        {
            DPRINT1!("Freeing entry %p\n", NotificationEntry);
            /* We found the event to remove */
            RemoveEntryList(core::ptr::addr_of_mut!((*NotificationEntry).Entry));
            (*MixerContext).Free.unwrap_unchecked()(NotificationEntry as PVOID);
        }
    }
    return MM_STATUS_SUCCESS;
}
