use crate::prelude::*;
use core::ffi::c_void;

unsafe extern "C" fn GetDmiString(mut Header: PDMI_HEADER, mut FieldOffset: ULONG) -> PCHAR {
    let mut StringIndex: ULONG;
    let mut String: PCHAR;

    StringIndex = *(Header as PUCHAR).offset(FieldOffset as isize) as ULONG;
    if StringIndex == 0 {
        return core::ptr::null_mut();
    }

    String = (Header as PCHAR).offset((*Header).Length as isize);

    loop {
        StringIndex = StringIndex.wrapping_sub(1);
        if !(StringIndex != 0) {
            break;
        }
        while *String != 0 {
            String = String.offset(1);
        }

        String = String.offset(1);
    }

    return String;
}
#[no_mangle]
pub unsafe extern "C" fn ParseSMBiosTables(
    SMBiosTables: PVOID,
    TableSize: ULONG,
    Strings: *mut PCHAR,
) {
    let _ = TableSize;
    let BiosTablesHeader: PMSSmBios_RawSMBiosTables = SMBiosTables as PMSSmBios_RawSMBiosTables;
    let mut Header: PDMI_HEADER = core::ptr::null_mut();
    let mut Remaining: ULONG = 0;
    let mut Data: PCHAR = core::ptr::null_mut();

    Header = core::ptr::addr_of_mut!((*BiosTablesHeader).SMBiosData) as PDMI_HEADER;
    Remaining = (*BiosTablesHeader).Size;

    while Remaining >= core::mem::size_of::<_DMI_HEADER>() as ULONG {
        if (*Header).Type as core::ffi::c_int == DMI_ENTRY_END_OF_TABLE as core::ffi::c_int {
            break;
        }

        match ((*Header).Type as core::ffi::c_int) as u32 {
            DMI_ENTRY_BIOS => {
                if Remaining < (DMI_BIOS_SIZE) as ULONG {
                    return;
                }
                *Strings.add((BIOS_VENDOR) as usize) = GetDmiString(Header, (DMI_BIOS_VENDOR) as ULONG);
                *Strings.add((BIOS_VERSION) as usize) = GetDmiString(Header, (DMI_BIOS_VERSION) as ULONG);
                *Strings.add((BIOS_DATE) as usize) = GetDmiString(Header, (DMI_BIOS_DATE) as ULONG);
            }
            DMI_ENTRY_SYSTEM => {
                if Remaining < (DMI_SYS_SIZE) as ULONG {
                    return;
                }
                *Strings.add((SYS_VENDOR) as usize) = GetDmiString(Header, (DMI_SYS_VENDOR) as ULONG);
                *Strings.add((SYS_PRODUCT) as usize) = GetDmiString(Header, (DMI_SYS_PRODUCT) as ULONG);
                *Strings.add((SYS_VERSION) as usize) = GetDmiString(Header, (DMI_SYS_VERSION) as ULONG);
                *Strings.add((SYS_SERIAL) as usize) = GetDmiString(Header, (DMI_SYS_SERIAL) as ULONG);
                *Strings.add((SYS_SKU) as usize) = GetDmiString(Header, (DMI_SYS_SKU) as ULONG);
                *Strings.add((SYS_FAMILY) as usize) = GetDmiString(Header, (DMI_SYS_FAMILY) as ULONG);
            }
            DMI_ENTRY_BASEBOARD => {
                if Remaining < (DMI_BOARD_SIZE) as ULONG {
                    return;
                }
                *Strings.add((BOARD_VENDOR) as usize) = GetDmiString(Header, (DMI_BOARD_VENDOR) as ULONG);
                *Strings.add((BOARD_NAME) as usize) = GetDmiString(Header, (DMI_BOARD_NAME) as ULONG);
                *Strings.add((BOARD_VERSION) as usize) = GetDmiString(Header, (DMI_BOARD_VERSION) as ULONG);
                *Strings.add((BOARD_SERIAL) as usize) = GetDmiString(Header, (DMI_BOARD_SERIAL) as ULONG);
                *Strings.add((BOARD_ASSET_TAG) as usize) = GetDmiString(Header, (DMI_BOARD_ASSET_TAG) as ULONG);
            }
            // DMI_ENTRY_IPMI_DEV?
            // DMI_ENTRY_ONBOARD_DEV_EXT?
            DMI_ENTRY_CHASSIS | DMI_ENTRY_ONBOARD_DEVICE | DMI_ENTRY_OEMSTRINGS => {}
            _ => {}
        }

        Remaining = Remaining.wrapping_sub((*Header).Length as ULONG);
        Data = (Header as PCHAR).add((*Header).Length as usize);

        /* Now loop until we find 2 zeroes */
        while (Remaining >= (2) as ULONG)
            && (((*Data.add((0) as usize)) as core::ffi::c_int) != (0 as core::ffi::c_int)
                || ((*Data.add((1) as usize)) as core::ffi::c_int) != (0 as core::ffi::c_int))
        {
            Data = Data.add(1);
            Remaining = Remaining.wrapping_sub(1);
        }

        if Remaining < (2) as ULONG {
            break;
        }

        /* Go to the next header */
        Remaining = Remaining.wrapping_sub((2) as ULONG);
        Header = (Data as PUCHAR).add((2) as usize) as PDMI_HEADER;
    }
}
