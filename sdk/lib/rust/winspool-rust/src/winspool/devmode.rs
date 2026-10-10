use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut MinimumSizeA: [MINIMUM_SIZE_TABLE; 31] = [
        _MINIMUM_SIZE_TABLE {
            dwField: DM_PANNINGHEIGHT as DWORD,
            wSize: (152 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<DWORD>() as usize) as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_PANNINGWIDTH as DWORD,
            wSize: (148 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<DWORD>() as usize) as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_DITHERTYPE as DWORD,
            wSize: (136 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<DWORD>() as usize) as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_MEDIATYPE as DWORD,
            wSize: (132 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<DWORD>() as usize) as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_ICMINTENT as DWORD,
            wSize: (128 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<DWORD>() as usize) as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_ICMMETHOD as DWORD,
            wSize: (124 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<DWORD>() as usize) as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_DISPLAYFREQUENCY as DWORD,
            wSize: (120 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<DWORD>() as usize) as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_NUP as DWORD,
            wSize: (116 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<DWORD>() as usize) as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_DISPLAYFLAGS as DWORD,
            wSize: (116 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<DWORD>() as usize) as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_PELSHEIGHT as DWORD,
            wSize: (112 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<DWORD>() as usize) as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_PELSWIDTH as DWORD,
            wSize: (108 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<DWORD>() as usize) as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_BITSPERPEL as DWORD,
            wSize: (104 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<DWORD>() as usize) as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_LOGPIXELS as DWORD,
            wSize: (102 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<WORD>() as usize) as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_FORMNAME as DWORD,
            wSize: (70 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<[BYTE; 32]>() as usize)
                as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_COLLATE as DWORD,
            wSize: (68 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<::core::ffi::c_short>() as usize)
                as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_TTOPTION as DWORD,
            wSize: (66 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<::core::ffi::c_short>() as usize)
                as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_YRESOLUTION as DWORD,
            wSize: (64 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<::core::ffi::c_short>() as usize)
                as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_DUPLEX as DWORD,
            wSize: (62 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<::core::ffi::c_short>() as usize)
                as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_COLOR as DWORD,
            wSize: (60 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<::core::ffi::c_short>() as usize)
                as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_DISPLAYFIXEDOUTPUT as DWORD,
            wSize: (56 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<DWORD>() as usize) as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_DISPLAYORIENTATION as DWORD,
            wSize: (52 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<DWORD>() as usize) as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_POSITION as DWORD,
            wSize: (44 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<POINTL>() as usize) as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_PRINTQUALITY as DWORD,
            wSize: (58 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<::core::ffi::c_short>() as usize)
                as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_DEFAULTSOURCE as DWORD,
            wSize: (56 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<::core::ffi::c_short>() as usize)
                as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_COPIES as DWORD,
            wSize: (54 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<::core::ffi::c_short>() as usize)
                as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_SCALE as DWORD,
            wSize: (52 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<::core::ffi::c_short>() as usize)
                as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_PAPERWIDTH as DWORD,
            wSize: (50 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<::core::ffi::c_short>() as usize)
                as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_PAPERLENGTH as DWORD,
            wSize: (48 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<::core::ffi::c_short>() as usize)
                as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_PAPERSIZE as DWORD,
            wSize: (46 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<::core::ffi::c_short>() as usize)
                as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_ORIENTATION as DWORD,
            wSize: (44 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<::core::ffi::c_short>() as usize)
                as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: 0 as DWORD,
            wSize: 0 as WORD,
        },
    ];

static mut MinimumSizeW: [MINIMUM_SIZE_TABLE; 31] = [
        _MINIMUM_SIZE_TABLE {
            dwField: DM_PANNINGHEIGHT as DWORD,
            wSize: (216 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<DWORD>() as usize) as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_PANNINGWIDTH as DWORD,
            wSize: (212 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<DWORD>() as usize) as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_DITHERTYPE as DWORD,
            wSize: (200 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<DWORD>() as usize) as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_MEDIATYPE as DWORD,
            wSize: (196 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<DWORD>() as usize) as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_ICMINTENT as DWORD,
            wSize: (192 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<DWORD>() as usize) as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_ICMMETHOD as DWORD,
            wSize: (188 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<DWORD>() as usize) as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_DISPLAYFREQUENCY as DWORD,
            wSize: (184 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<DWORD>() as usize) as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_NUP as DWORD,
            wSize: (180 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<DWORD>() as usize) as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_DISPLAYFLAGS as DWORD,
            wSize: (180 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<DWORD>() as usize) as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_PELSHEIGHT as DWORD,
            wSize: (176 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<DWORD>() as usize) as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_PELSWIDTH as DWORD,
            wSize: (172 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<DWORD>() as usize) as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_BITSPERPEL as DWORD,
            wSize: (168 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<DWORD>() as usize) as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_LOGPIXELS as DWORD,
            wSize: (166 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<WORD>() as usize) as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_FORMNAME as DWORD,
            wSize: (102 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<[WCHAR; 32]>() as usize)
                as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_COLLATE as DWORD,
            wSize: (100 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<::core::ffi::c_short>() as usize)
                as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_TTOPTION as DWORD,
            wSize: (98 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<::core::ffi::c_short>() as usize)
                as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_YRESOLUTION as DWORD,
            wSize: (96 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<::core::ffi::c_short>() as usize)
                as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_DUPLEX as DWORD,
            wSize: (94 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<::core::ffi::c_short>() as usize)
                as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_COLOR as DWORD,
            wSize: (92 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<::core::ffi::c_short>() as usize)
                as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_DISPLAYFIXEDOUTPUT as DWORD,
            wSize: (88 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<DWORD>() as usize) as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_DISPLAYORIENTATION as DWORD,
            wSize: (84 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<DWORD>() as usize) as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_POSITION as DWORD,
            wSize: (76 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<POINTL>() as usize) as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_PRINTQUALITY as DWORD,
            wSize: (90 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<::core::ffi::c_short>() as usize)
                as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_DEFAULTSOURCE as DWORD,
            wSize: (88 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<::core::ffi::c_short>() as usize)
                as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_COPIES as DWORD,
            wSize: (86 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<::core::ffi::c_short>() as usize)
                as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_SCALE as DWORD,
            wSize: (84 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<::core::ffi::c_short>() as usize)
                as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_PAPERWIDTH as DWORD,
            wSize: (82 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<::core::ffi::c_short>() as usize)
                as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_PAPERLENGTH as DWORD,
            wSize: (80 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<::core::ffi::c_short>() as usize)
                as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_PAPERSIZE as DWORD,
            wSize: (78 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<::core::ffi::c_short>() as usize)
                as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: DM_ORIENTATION as DWORD,
            wSize: (76 as ::core::ffi::c_ulonglong as LONG as usize)
                .wrapping_add(::core::mem::size_of::<::core::ffi::c_short>() as usize)
                as WORD,
        },
        _MINIMUM_SIZE_TABLE {
            dwField: 0 as DWORD,
            wSize: 0 as WORD,
        },
    ];

pub type PMINIMUM_SIZE_TABLE = *mut MINIMUM_SIZE_TABLE;


unsafe extern "C" fn _FixStringA(mut String: PBYTE, mut cbString: DWORD) {
    let pLastCharacter: PBYTE = String.offset(
        (cbString as usize)
            .wrapping_div(::core::mem::size_of::<BYTE>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) as PBYTE;
    let mut p: PBYTE = String;
    while *p != 0 {
        if p == pLastCharacter {
            *p = 0 as BYTE;
            break;
        } else {
            p = p.offset(1);
        }
    }
}

unsafe extern "C" fn _FixStringW(mut String: PWSTR, mut cbString: DWORD) {
    let pLastCharacter: PWSTR = String.offset(
        (cbString as usize)
            .wrapping_div(::core::mem::size_of::<WCHAR>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) as PWSTR;
    let mut p: PWSTR = String;
    while *p != 0 {
        if p == pLastCharacter {
            *p = 0 as WCHAR;
            break;
        } else {
            p = p.offset(1);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn IsValidDevmodeA(pDevmode: PDEVMODEA, DevmodeSize: usize) -> BOOL {
    let mut pTable: PMINIMUM_SIZE_TABLE = core::ptr::addr_of_mut!(MinimumSizeA) as PMINIMUM_SIZE_TABLE;
    let mut wRequiredSize: WORD;

    'fail: {
        // Check if a Devmode was given at all.
        if pDevmode.is_null() {
            break 'fail;
        }

        // Verify that DevmodeSize is large enough to hold the public and private members of the structure.
        if DevmodeSize < ((*pDevmode).dmSize as usize + (*pDevmode).dmDriverExtra as usize) {
            break 'fail;
        }

        // If the structure has private members, the public structure must be 32-bit packed.
        if (*pDevmode).dmDriverExtra != 0 && ((*pDevmode).dmSize as u32 % 4) != 0 {
            break 'fail;
        }

        // Now determine the minimum possible dmSize based on the given fields in dmFields.
        wRequiredSize = ((FIELD_OFFSET!(DEVMODEA, dmFields) as usize) + RTL_FIELD_SIZE!(DEVMODEA, dmFields)) as WORD;

        while (*pTable).dwField != 0 {
            if ((*pDevmode).dmFields & (*pTable).dwField) != 0 {
                wRequiredSize = (*pTable).wSize;
                break;
            }

            pTable = pTable.add(1);
        }

        // Verify that the value in dmSize is big enough for the used fields.
        if ((*pDevmode).dmSize as usize) < (wRequiredSize as usize) {
            break 'fail;
        }

        // Check if dmDeviceName and (if used) dmFormName are null-terminated.
        // Fix this if they aren't.
        _FixStringA(
            core::ptr::addr_of_mut!((*pDevmode).dmDeviceName) as PBYTE,
            core::mem::size_of::<[BYTE; CCHDEVICENAME as usize]>() as DWORD,
        );
        if ((*pDevmode).dmFields & DM_FORMNAME) != 0 {
            _FixStringA(
                core::ptr::addr_of_mut!((*pDevmode).dmFormName) as PBYTE,
                core::mem::size_of::<[BYTE; CCHDEVICENAME as usize]>() as DWORD,
            );
        }

        // Return success without setting the error code.
        return TRUE as BOOL;
    }

    // Failure:
    SetLastError(ERROR_INVALID_DATA as DWORD);
    return FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn IsValidDevmodeW(pDevmode: PDEVMODEW, DevmodeSize: size_t) -> BOOL {
    let mut pTable: PMINIMUM_SIZE_TABLE = core::ptr::addr_of_mut!(MinimumSizeW) as PMINIMUM_SIZE_TABLE;
    let mut wRequiredSize: WORD = 0;
    'Failure: {
        // Check if a Devmode was given at all.
        if pDevmode.is_null() {
            break 'Failure;
        }

        // Verify that DevmodeSize is large enough to hold the public and private members of the structure.
        if DevmodeSize < (((*pDevmode).dmSize as i32) + ((*pDevmode).dmDriverExtra as i32)) as size_t {
            break 'Failure;
        }

        // If the structure has private members, the public structure must be 32-bit packed.
        if (*pDevmode).dmDriverExtra != 0 && ((*pDevmode).dmSize as i32 % 4) != 0 {
            break 'Failure;
        }

        // Now determine the minimum possible dmSize based on the given fields in dmFields.
        wRequiredSize = (FIELD_OFFSET!(DEVMODEW, dmFields) as WORD)
            .wrapping_add(RTL_FIELD_SIZE!(DEVMODEW, dmFields) as WORD);

        while (*pTable).dwField != 0 {
            if (*pDevmode).dmFields & (*pTable).dwField != 0 {
                wRequiredSize = (*pTable).wSize;
                break;
            }

            pTable = pTable.add(1);
        }

        // Verify that the value in dmSize is big enough for the used fields.
        if (*pDevmode).dmSize < wRequiredSize {
            break 'Failure;
        }

        // Check if dmDeviceName and (if used) dmFormName are null-terminated.
        // Fix this if they aren't.
        _FixStringW(
            core::ptr::addr_of_mut!((*pDevmode).dmDeviceName) as PWSTR,
            core::mem::size_of::<[WCHAR; 32]>() as DWORD,
        );
        if (*pDevmode).dmFields & DM_FORMNAME != 0 {
            _FixStringW(
                core::ptr::addr_of_mut!((*pDevmode).dmFormName) as PWSTR,
                core::mem::size_of::<[WCHAR; 32]>() as DWORD,
            );
        }

        // Return success without setting the error code.
        return TRUE as BOOL;
    }
    // Failure:
    SetLastError(ERROR_INVALID_DATA);
    return FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn IsValidDevmodeNoSizeW(pDevmode: PDEVMODEW) -> BOOL {
    let mut pTable: PMINIMUM_SIZE_TABLE = core::ptr::addr_of_mut!(MinimumSizeW) as PMINIMUM_SIZE_TABLE;
    let mut wRequiredSize: WORD = 0;

    'failure: {
        // Check if a Devmode was given at all.
        if pDevmode.is_null() {
            break 'failure;
        }

        // If the structure has private members, the public structure must be 32-bit packed.
        if (*pDevmode).dmDriverExtra != 0 && (*pDevmode).dmSize % 4 != 0 {
            break 'failure;
        }

        // Now determine the minimum possible dmSize based on the given fields in dmFields.
        wRequiredSize = (FIELD_OFFSET!(DEVMODEW, dmFields) as usize + RTL_FIELD_SIZE!(DEVMODEW, dmFields)) as WORD;

        while (*pTable).dwField != 0 {
            if ((*pDevmode).dmFields & (*pTable).dwField) != 0 {
                wRequiredSize = (*pTable).wSize;
                break;
            }

            pTable = pTable.add(1);
        }

        // Verify that the value in dmSize is big enough for the used fields.
        if ((*pDevmode).dmSize as core::ffi::c_int) < (wRequiredSize as core::ffi::c_int) {
            break 'failure;
        }

        // Check if dmDeviceName and (if used) dmFormName are null-terminated.
        // Fix this if they aren't.
        _FixStringW(
            core::ptr::addr_of_mut!((*pDevmode).dmDeviceName) as PWSTR,
            core::mem::size_of::<[WCHAR; 32]>() as DWORD,
        );
        if ((*pDevmode).dmFields & DM_FORMNAME) != 0 {
            _FixStringW(
                core::ptr::addr_of_mut!((*pDevmode).dmFormName) as PWSTR,
                core::mem::size_of::<[WCHAR; 32]>() as DWORD,
            );
        }

        // Return success without setting the error code.
        return TRUE as BOOL;
    }

    // Failure:
    SetLastError(ERROR_INVALID_DATA);
    return FALSE;
}
#[no_mangle]
pub unsafe extern "C" fn RosConvertAnsiDevModeToUnicodeDevmode(mut pDevModeInput: PDEVMODEA, mut pDevModeOutput: *mut PDEVMODEW) {
    if pDevModeInput.is_null() || pDevModeOutput.is_null() {
        return;
    }

    *pDevModeOutput = GdiConvertToDevmodeW(pDevModeInput as *const DEVMODEA) as PDEVMODEW;
}
unsafe extern "C" fn _ConvertToDevmodeA(dmW: *const DEVMODEW) -> *mut DEVMODEA {
    let mut dmA: *mut DEVMODEA = core::ptr::null_mut();
    let mut dmA_size: WORD;
    let mut dmW_size: WORD;
    let mut BytesToCopy: usize;

    dmW_size = (*dmW).dmSize;

    /* this is the minimal dmSize that XP accepts */
    if (dmW_size as usize) < core::mem::offset_of!(DEVMODEW, dmFields) {
        return core::ptr::null_mut();
    }

    // Guard against callers that set dmSize incorrectly.
    if (dmW_size as usize) > core::mem::size_of::<DEVMODEW>() {
        dmW_size = core::mem::size_of::<DEVMODEW>() as WORD;
    }

    // dmA_size must become dmW_size without the additional 1 byte per character for each Unicode string (dmDeviceName and dmFormName).
    dmA_size = (dmW_size as i32 - CCHDEVICENAME as i32) as WORD;
    if (dmW_size as usize)
        >= core::mem::offset_of!(DEVMODEW, dmFormName)
            + (CCHFORMNAME as usize) * core::mem::size_of::<WCHAR>()
    {
        dmA_size = (dmA_size as i32 - CCHFORMNAME as i32) as WORD;
    }

    // Allocate the required bytes, that is dmSize for the ANSI DEVMODEA structure plus any extra bytes requested through dmDriverExtra.
    dmA = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        (dmA_size as i32 + (*dmW).dmDriverExtra as i32) as SIZE_T,
    ) as *mut DEVMODEA;
    if dmA.is_null() {
        return core::ptr::null_mut();
    }

    // Every valid DEVMODEW has a dmDeviceName, which we convert to ANSI here.
    WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        &raw const (*dmW).dmDeviceName as LPCWSTR,
        -(1 as i32),
        &raw mut (*dmA).dmDeviceName as *mut BYTE as LPSTR,
        CCHDEVICENAME as i32,
        core::ptr::null::<CHAR>(),
        core::ptr::null_mut::<BOOL>(),
    );

    // Copy everything up to dmFormName or the remaining dmW_size, whatever is smaller.
    BytesToCopy = min(
        core::mem::offset_of!(DEVMODEW, dmFormName)
            - core::mem::offset_of!(DEVMODEW, dmSpecVersion),
        (dmW_size as usize) - (CCHDEVICENAME as usize) * core::mem::size_of::<WCHAR>(),
    );
    memcpy(
        &raw mut (*dmA).dmSpecVersion as *mut core::ffi::c_void,
        &raw const (*dmW).dmSpecVersion as *const core::ffi::c_void,
        BytesToCopy,
    );

    // Handle dmFormName if the input DEVMODEW is large enough to contain one.
    if (dmW_size as usize)
        >= core::mem::offset_of!(DEVMODEW, dmFormName)
            + (CCHFORMNAME as usize) * core::mem::size_of::<WCHAR>()
    {
        if (*dmW).dmFields & DM_FORMNAME as DWORD != 0 {
            WideCharToMultiByte(
                CP_ACP as UINT,
                0 as DWORD,
                &raw const (*dmW).dmFormName as LPCWSTR,
                -(1 as i32),
                &raw mut (*dmA).dmFormName as *mut BYTE as LPSTR,
                CCHFORMNAME as i32,
                core::ptr::null::<CHAR>(),
                core::ptr::null_mut::<BOOL>(),
            );
        } else {
            (*dmA).dmFormName[0 as usize] = 0 as BYTE;
        }

        // Copy the remaining fields.
        if (dmW_size as usize) > core::mem::offset_of!(DEVMODEW, dmLogPixels) {
            memcpy(
                &raw mut (*dmA).dmLogPixels as *mut core::ffi::c_void,
                &raw const (*dmW).dmLogPixels as *const core::ffi::c_void,
                (dmW_size as usize)
                    - core::mem::offset_of!(DEVMODEW, dmLogPixels),
            );
        }
    }

    // Append dmDriverExtra if required.
    if (*dmW).dmDriverExtra != 0 {
        memcpy(
            (dmA as *mut core::ffi::c_char).offset(dmA_size as isize) as *mut core::ffi::c_void,
            (dmW as *const core::ffi::c_char).offset(dmW_size as isize) as *const core::ffi::c_void,
            (*dmW).dmDriverExtra as usize,
        );
    }

    // Set the corrected dmSize and we are done.
    (*dmA).dmSize = dmA_size;

    dmA
}
#[no_mangle]
pub unsafe extern "C" fn RosConvertUnicodeDevModeToAnsiDevmode(pDevModeInput: PDEVMODEW, pDevModeOutput: PDEVMODEA) {
    let mut pTmp: PDEVMODEA = core::ptr::null_mut();
    if pDevModeInput.is_null() || pDevModeOutput.is_null() {
        return;
    }
    pTmp = _ConvertToDevmodeA(pDevModeInput as *const DEVMODEW) as PDEVMODEA;
    memcpy(
        pDevModeOutput as *mut core::ffi::c_void,
        pTmp as *const core::ffi::c_void,
        ((*pTmp).dmSize as usize + (*pTmp).dmDriverExtra as usize),
    );
    HeapFree(hProcessHeap, 0 as DWORD, pTmp as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn DeviceMode(
    param_1: PVOID,
    param_2: PVOID,
    param_3: PCHAR,
    param_4: PVOID,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn ExtDeviceMode(
    hWnd: HWND,
    hInst: HANDLE,
    pDevModeOutput: LPDEVMODEA,
    pDeviceName: LPSTR,
    pPort: LPSTR,
    pDevModeInput: LPDEVMODEA,
    pProfile: LPSTR,
    fMode: DWORD,
) -> LONG {
    // UNIMPLEMENTED;
    0
}
