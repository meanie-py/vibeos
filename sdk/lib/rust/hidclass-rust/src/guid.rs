// guid.rs — GUID definitions for hidclass (replaces guid.c's DEFINE_GUID set).
// guid.c pulls in <wdmguid.h> + <hidclass.h> with <initguid.h>; the only GUIDs
// actually referenced by hidclass code are GUID_BUS_TYPE_HID (pdo.c
// IRP_MN_QUERY_BUS_INFORMATION) and GUID_DEVINTERFACE_HID (pdo.c
// IRP_MN_START_DEVICE IoRegisterDeviceInterface).

use ntkrnl_sys::ddk::GUID;

// hidclass.h:
// DEFINE_GUID(GUID_DEVINTERFACE_HID,
//   0x4D1E55B2L, 0xF16F, 0x11CF, 0x88, 0xCB, 0x00, 0x11, 0x11, 0x00, 0x00, 0x30);
#[no_mangle]
pub static GUID_DEVINTERFACE_HID: GUID = GUID {
    Data1: 0x4D1E_55B2,
    Data2: 0xF16F,
    Data3: 0x11CF,
    Data4: [0x88, 0xCB, 0x00, 0x11, 0x11, 0x00, 0x00, 0x30],
};

// wdmguid.h:
// DEFINE_GUID(GUID_BUS_TYPE_HID,
//   0xeeaf37d0, 0x1963, 0x47c4, 0xaa, 0x48, 0x72, 0x47, 0x6d, 0xb7, 0xcf, 0x49);
#[no_mangle]
pub static GUID_BUS_TYPE_HID: GUID = GUID {
    Data1: 0xeeaf_37d0,
    Data2: 0x1963,
    Data3: 0x47c4,
    Data4: [0xaa, 0x48, 0x72, 0x47, 0x6d, 0xb7, 0xcf, 0x49],
};
