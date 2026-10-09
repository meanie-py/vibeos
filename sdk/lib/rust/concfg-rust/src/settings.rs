use winsrv_sys::winsrv::*;

use core::ptr;

const PATH_SEPARATOR: u16 = b'\\' as u16;

const CSR_DEFAULT_CURSOR_SIZE: u32 = 25;
const DEFAULT_SCREEN_ATTRIB: u16 = 0x0007;
const DEFAULT_POPUP_ATTRIB: u16 = 0x00F5;

static S_COLORS: [COLORREF; 16] = [
    0x00000000, // RGB(0, 0, 0)
    0x00800000, // RGB(0, 0, 128)
    0x00008000, // RGB(0, 128, 0)
    0x00808000, // RGB(0, 128, 128)
    0x00000080, // RGB(128, 0, 0)
    0x00800080, // RGB(128, 0, 128)
    0x00008080, // RGB(128, 128, 0)
    0x00C0C0C0, // RGB(192, 192, 192)
    0x00808080, // RGB(128, 128, 128)
    0x00FF0000, // RGB(0, 0, 255)
    0x0000FF00, // RGB(0, 255, 0)
    0x00FFFF00, // RGB(0, 255, 255)
    0x000000FF, // RGB(255, 0, 0)
    0x00FF00FF, // RGB(255, 0, 255)
    0x0000FFFF, // RGB(255, 255, 0)
    0x00FFFFFF, // RGB(255, 255, 255)
];

unsafe fn TranslateConsoleName(
    DestString: LPWSTR,
    ConsoleName: LPCWSTR,
    MaxStrLen: u32,
) {
    if DestString.is_null() || ConsoleName.is_null() || MaxStrLen == 0 {
        return;
    }

    if *ConsoleName == 0 {
        return;
    }

    let wLength = GetSystemWindowsDirectoryW(DestString, MaxStrLen);
    if wLength > 0 {
        let mut matches = true;
        for i in 0..wLength as isize {
            let c1 = *DestString.offset(i);
            let c2 = *ConsoleName.offset(i);
            if (c1 | 0x20) != (c2 | 0x20) {
                matches = false;
                break;
            }
        }

        if matches {
            static SYSTEMROOT: [u16; 13] = [
                b'%' as u16, b'S' as u16, b'y' as u16, b's' as u16, b't' as u16,
                b'e' as u16, b'm' as u16, b'R' as u16, b'o' as u16, b'o' as u16,
                b't' as u16, b'%' as u16, 0,
            ];
            core::ptr::copy_nonoverlapping(
                SYSTEMROOT.as_ptr(),
                DestString,
                SYSTEMROOT.len(),
            );

            let mut dest_len = SYSTEMROOT.len() - 1;
            let mut src_idx = wLength as isize;
            let mut dest_ptr = DestString.offset(dest_len as isize);
            
            while *ConsoleName.offset(src_idx) != 0 && (dest_len as u32) < MaxStrLen - 1 {
                *dest_ptr = *ConsoleName.offset(src_idx);
                dest_ptr = dest_ptr.offset(1);
                src_idx += 1;
                dest_len += 1;
            }
            *dest_ptr = 0;
        } else {
            let mut i = 0;
            while i < (MaxStrLen - 1) as isize && *ConsoleName.offset(i) != 0 {
                *DestString.offset(i) = *ConsoleName.offset(i);
                i += 1;
            }
            *DestString.offset(i) = 0;
        }
    } else {
        let mut i = 0;
        while i < (MaxStrLen - 1) as isize && *ConsoleName.offset(i) != 0 {
            *DestString.offset(i) = *ConsoleName.offset(i);
            i += 1;
        }
        *DestString.offset(i) = 0;
    }

    let mut p = DestString;
    while *p != 0 {
        if *p == PATH_SEPARATOR {
            *p = b'_' as u16;
        }
        p = p.offset(1);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ConCfgOpenUserSettings(
    ConsoleTitle: LPCWSTR,
    phSubKey: PHKEY,
    samDesired: REGSAM,
    Create: u8,
) -> u8 {
    let Success: u8;
    let mut hKey: HKEY = ptr::null_mut();

    let mut szBuffer: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    static CONSOLE_PREFIX: [u16; 9] = [
        b'C' as u16, b'o' as u16, b'n' as u16, b's' as u16, b'o' as u16,
        b'l' as u16, b'e' as u16, b'\\' as u16, 0,
    ];
    core::ptr::copy_nonoverlapping(
        CONSOLE_PREFIX.as_ptr(),
        szBuffer.as_mut_ptr(),
        CONSOLE_PREFIX.len(),
    );

    let mut szBuffer2: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];

    let Status = RtlOpenCurrentUser(MAXIMUM_ALLOWED, core::ptr::addr_of_mut!(hKey) as PHANDLE);
    if Status < 0 {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    TranslateConsoleName(szBuffer2.as_mut_ptr(), ConsoleTitle, MAX_PATH);

    let mut console_len = 0;
    while szBuffer[console_len] != 0 {
        console_len += 1;
    }

    let mut title_idx = 0;
    while szBuffer2[title_idx] != 0 && console_len < (MAX_PATH as usize - 1) {
        szBuffer[console_len] = szBuffer2[title_idx];
        console_len += 1;
        title_idx += 1;
    }
    szBuffer[console_len] = 0;

    if Create != 0 {
        let mut disposition: u32 = 0;
        Success = if RegCreateKeyExW(
            hKey,
            szBuffer.as_ptr(),
            0,
            ptr::null_mut(),
            REG_OPTION_NON_VOLATILE,
            samDesired,
            ptr::null_mut(),
            phSubKey,
            core::ptr::addr_of_mut!(disposition),
        ) == ERROR_SUCCESS as i32
        {
            1
        } else {
            0
        };
    } else {
        Success = if RegOpenKeyExW(hKey, szBuffer.as_ptr(), 0, samDesired, phSubKey)
            == ERROR_SUCCESS as i32
        {
            1
        } else {
            0
        };
    }

    NtClose(hKey as HANDLE);
    Success
}

#[no_mangle]
pub unsafe extern "C" fn ConCfgReadUserSettings(
    ConsoleInfo: PCONSOLE_STATE_INFO,
    DefaultSettings: u8,
) -> u8 {
    let mut Success: u8 = 0;
    let mut hKey: HKEY = ptr::null_mut();
    let mut dwNumValues: u32 = 0;
    let mut dwIndex: u32;
    let mut dwColorIndex: u32;
    let mut dwType: u32 = 0;
    let mut szValueName: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut dwValueName: u32;
    let mut szValue: [u16; LF_FACESIZE as usize] = [0; LF_FACESIZE as usize];
    let mut Value: u32;
    let mut dwValue: u32;

    let title = if DefaultSettings != 0 {
        static EMPTY: [u16; 1] = [0];
        EMPTY.as_ptr()
    } else {
        (*ConsoleInfo).ConsoleTitle.as_ptr()
    };

    if ConCfgOpenUserSettings(title, core::ptr::addr_of_mut!(hKey), KEY_READ, 0) == 0 {
        return 0;
    }

    if RegQueryInfoKeyW(
        hKey,
        ptr::null_mut(),
        ptr::null_mut(),
        ptr::null_mut(),
        ptr::null_mut(),
        ptr::null_mut(),
        ptr::null_mut(),
        core::ptr::addr_of_mut!(dwNumValues),
        ptr::null_mut(),
        ptr::null_mut(),
        ptr::null_mut(),
        ptr::null_mut(),
    ) != ERROR_SUCCESS as i32
    {
        RegCloseKey(hKey);
        return 0;
    }

    dwIndex = 0;
    while dwIndex < dwNumValues {
        dwValue = core::mem::size_of::<u32>() as u32;
        dwValueName = szValueName.len() as u32;
        Value = 0;

        let mut result = RegEnumValueW(
            hKey,
            dwIndex,
            szValueName.as_mut_ptr(),
            core::ptr::addr_of_mut!(dwValueName),
            ptr::null_mut(),
            core::ptr::addr_of_mut!(dwType),
            core::ptr::addr_of_mut!(Value) as PBYTE,
            core::ptr::addr_of_mut!(dwValue),
        );

        if result != ERROR_SUCCESS as i32 {
            if dwType == REG_DWORD {
                break;
            }

            dwValue = (szValue.len() * 2) as u32;
            dwValueName = szValueName.len() as u32;
            szValue = [0; LF_FACESIZE as usize];

            result = RegEnumValueW(
                hKey,
                dwIndex,
                szValueName.as_mut_ptr(),
                core::ptr::addr_of_mut!(dwValueName),
                ptr::null_mut(),
                ptr::null_mut(),
                szValue.as_mut_ptr() as PBYTE,
                core::ptr::addr_of_mut!(dwValue),
            );

            if result != ERROR_SUCCESS as i32 {
                break;
            }
        }

        static COLORTABLE: [u16; 11] = [
            b'C' as u16, b'o' as u16, b'l' as u16, b'o' as u16, b'r' as u16,
            b'T' as u16, b'a' as u16, b'b' as u16, b'l' as u16, b'e' as u16,
            0,
        ];

        let mut matches = true;
        for i in 0..10 {
            if szValueName[i] != COLORTABLE[i] {
                matches = false;
                break;
            }
        }

        if matches {
            dwColorIndex = 0;
            if szValueName[10] >= b'0' as u16 && szValueName[10] <= b'9' as u16 {
                dwColorIndex = (szValueName[10] - b'0' as u16) as u32;
            }
            if szValueName[11] >= b'0' as u16 && szValueName[11] <= b'9' as u16 {
                dwColorIndex = dwColorIndex * 10 + (szValueName[11] - b'0' as u16) as u32;
            }
            if (dwColorIndex as usize) < (*ConsoleInfo).ColorTable.len() {
                (*ConsoleInfo).ColorTable[dwColorIndex as usize] = Value;
                Success = 1;
            }
        }

        static NAMES: [(&[u16], usize); 19] = [
            (
                &[b'C' as u16, b'o' as u16, b'd' as u16, b'e' as u16, b'P' as u16, b'a' as u16, b'g' as u16, b'e' as u16, 0],
                8,
            ),
            (
                &[b'F' as u16, b'a' as u16, b'c' as u16, b'e' as u16, b'N' as u16, b'a' as u16, b'm' as u16, b'e' as u16, 0],
                8,
            ),
            (
                &[b'F' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'F' as u16, b'a' as u16, b'm' as u16, b'i' as u16, b'l' as u16, b'y' as u16, 0],
                10,
            ),
            (
                &[b'F' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'S' as u16, b'i' as u16, b'z' as u16, b'e' as u16, 0],
                8,
            ),
            (
                &[b'F' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'W' as u16, b'e' as u16, b'i' as u16, b'g' as u16, b'h' as u16, b't' as u16, 0],
                10,
            ),
            (
                &[b'H' as u16, b'i' as u16, b's' as u16, b't' as u16, b'o' as u16, b'r' as u16, b'y' as u16, b'B' as u16, b'u' as u16, b'f' as u16, b'f' as u16, b'e' as u16, b'r' as u16, b'S' as u16, b'i' as u16, b'z' as u16, b'e' as u16, 0],
                17,
            ),
            (
                &[b'N' as u16, b'u' as u16, b'm' as u16, b'b' as u16, b'e' as u16, b'r' as u16, b'O' as u16, b'f' as u16, b'H' as u16, b'i' as u16, b's' as u16, b't' as u16, b'o' as u16, b'r' as u16, b'y' as u16, b'B' as u16, b'u' as u16, b'f' as u16, b'f' as u16, b'e' as u16, b'r' as u16, b's' as u16, 0],
                22,
            ),
            (
                &[b'H' as u16, b'i' as u16, b's' as u16, b't' as u16, b'o' as u16, b'r' as u16, b'y' as u16, b'N' as u16, b'o' as u16, b'D' as u16, b'u' as u16, b'p' as u16, 0],
                12,
            ),
            (
                &[b'Q' as u16, b'u' as u16, b'i' as u16, b'c' as u16, b'k' as u16, b'E' as u16, b'd' as u16, b'i' as u16, b't' as u16, 0],
                9,
            ),
            (
                &[b'I' as u16, b'n' as u16, b's' as u16, b'e' as u16, b'r' as u16, b't' as u16, b'M' as u16, b'o' as u16, b'd' as u16, b'e' as u16, 0],
                10,
            ),
            (
                &[b'S' as u16, b'c' as u16, b'r' as u16, b'e' as u16, b'e' as u16, b'n' as u16, b'B' as u16, b'u' as u16, b'f' as u16, b'f' as u16, b'e' as u16, b'r' as u16, b'S' as u16, b'i' as u16, b'z' as u16, b'e' as u16, 0],
                16,
            ),
            (
                &[b'F' as u16, b'u' as u16, b'l' as u16, b'l' as u16, b'S' as u16, b'c' as u16, b'r' as u16, b'e' as u16, b'e' as u16, b'n' as u16, 0],
                10,
            ),
            (
                &[b'W' as u16, b'i' as u16, b'n' as u16, b'd' as u16, b'o' as u16, b'w' as u16, b'P' as u16, b'o' as u16, b's' as u16, b'i' as u16, b't' as u16, b'i' as u16, b'o' as u16, b'n' as u16, 0],
                14,
            ),
            (
                &[b'W' as u16, b'i' as u16, b'n' as u16, b'd' as u16, b'o' as u16, b'w' as u16, b'S' as u16, b'i' as u16, b'z' as u16, b'e' as u16, 0],
                10,
            ),
            (
                &[b'C' as u16, b'u' as u16, b'r' as u16, b's' as u16, b'o' as u16, b'r' as u16, b'S' as u16, b'i' as u16, b'z' as u16, b'e' as u16, 0],
                10,
            ),
            (
                &[b'S' as u16, b'c' as u16, b'r' as u16, b'e' as u16, b'e' as u16, b'n' as u16, b'C' as u16, b'o' as u16, b'l' as u16, b'o' as u16, b'r' as u16, b's' as u16, 0],
                12,
            ),
            (
                &[b'P' as u16, b'o' as u16, b'p' as u16, b'u' as u16, b'p' as u16, b'C' as u16, b'o' as u16, b'l' as u16, b'o' as u16, b'r' as u16, b's' as u16, 0],
                11,
            ),
            (
                &[0],
                0,
            ),
            (
                &[0],
                0,
            ),
        ];

        for (name_idx, &(name, len)) in NAMES.iter().enumerate() {
            if len == 0 {
                break;
            }

            let mut name_matches = true;
            for i in 0..len {
                if szValueName[i] != name[i] {
                    name_matches = false;
                    break;
                }
            }
            if !name_matches || szValueName[len] != 0 {
                continue;
            }

            match name_idx {
                0 => {
                    if IsValidCodePage(Value) != 0 {
                        (*ConsoleInfo).CodePage = Value;
                    }
                    Success = 1;
                }
                1 => {
                    if szValue[0] != 0 {
                        for i in 0..LF_FACESIZE as usize {
                            if i < (*ConsoleInfo).FaceName.len() {
                                (*ConsoleInfo).FaceName[i] = szValue[i];
                            }
                        }
                    }
                    Success = 1;
                }
                2 => {
                    if Value != 0 {
                        (*ConsoleInfo).FontFamily = Value;
                    }
                    Success = 1;
                }
                3 => {
                    if Value != 0 {
                        (*ConsoleInfo).FontSize.X = (Value & 0xFFFF) as i16;
                        (*ConsoleInfo).FontSize.Y = ((Value >> 16) & 0xFFFF) as i16;
                    }
                    Success = 1;
                }
                4 => {
                    if Value != 0 {
                        (*ConsoleInfo).FontWeight = Value;
                    }
                    Success = 1;
                }
                5 => {
                    (*ConsoleInfo).HistoryBufferSize = Value;
                    Success = 1;
                }
                6 => {
                    (*ConsoleInfo).NumberOfHistoryBuffers = Value;
                    Success = 1;
                }
                7 => {
                    (*ConsoleInfo).HistoryNoDup = if Value != 0 { 1 } else { 0 };
                    Success = 1;
                }
                8 => {
                    (*ConsoleInfo).QuickEdit = if Value != 0 { 1 } else { 0 };
                    Success = 1;
                }
                9 => {
                    (*ConsoleInfo).InsertMode = if Value != 0 { 1 } else { 0 };
                    Success = 1;
                }
                10 => {
                    (*ConsoleInfo).ScreenBufferSize.X = (Value & 0xFFFF) as i16;
                    (*ConsoleInfo).ScreenBufferSize.Y = ((Value >> 16) & 0xFFFF) as i16;
                    Success = 1;
                }
                11 => {
                    (*ConsoleInfo).FullScreen = Value as i32;
                    Success = 1;
                }
                12 => {
                    (*ConsoleInfo).AutoPosition = 0;
                    (*ConsoleInfo).WindowPosition.x = (Value & 0xFFFF) as i32;
                    (*ConsoleInfo).WindowPosition.y = ((Value >> 16) & 0xFFFF) as i32;
                    Success = 1;
                }
                13 => {
                    (*ConsoleInfo).WindowSize.X = (Value & 0xFFFF) as i16;
                    (*ConsoleInfo).WindowSize.Y = ((Value >> 16) & 0xFFFF) as i16;
                    Success = 1;
                }
                14 => {
                    let mut cursor_size = Value;
                    if cursor_size > 100 {
                        cursor_size = 100;
                    }
                    (*ConsoleInfo).CursorSize = cursor_size;
                    Success = 1;
                }
                15 => {
                    (*ConsoleInfo).ScreenAttributes = Value as u16;
                    Success = 1;
                }
                16 => {
                    (*ConsoleInfo).PopupAttributes = Value as u16;
                    Success = 1;
                }
                _ => {}
            }
            break;
        }

        dwIndex += 1;
    }

    RegCloseKey(hKey);
    Success
}

#[no_mangle]
pub unsafe extern "C" fn ConCfgWriteUserSettings(
    ConsoleInfo: PCONSOLE_STATE_INFO,
    DefaultSettings: u8,
) -> u8 {
    let mut hKey: HKEY = ptr::null_mut();
    let mut Storage: u32;

    let title = if DefaultSettings != 0 {
        static EMPTY: [u16; 1] = [0];
        EMPTY.as_ptr()
    } else {
        (*ConsoleInfo).ConsoleTitle.as_ptr()
    };

    if ConCfgOpenUserSettings(title, core::ptr::addr_of_mut!(hKey), KEY_WRITE, 1) == 0 {
        return 0;
    }

    let mut szValueName: [u16; 15] = [0; 15];
    for i in 0..(*ConsoleInfo).ColorTable.len() {
        static FMT: [u16; 14] = [
            b'C' as u16, b'o' as u16, b'l' as u16, b'o' as u16, b'r' as u16,
            b'T' as u16, b'a' as u16, b'b' as u16, b'l' as u16, b'e' as u16,
            b'0' as u16, b'0' as u16, 0, 0,
        ];
        core::ptr::copy_nonoverlapping(FMT.as_ptr(), szValueName.as_mut_ptr(), 12);

        szValueName[10] = (b'0' + ((i / 10) % 10) as u8) as u16;
        szValueName[11] = (b'0' + (i % 10) as u8) as u16;
        szValueName[12] = 0;

        if DefaultSettings != 0 || (*ConsoleInfo).ColorTable[i] != S_COLORS[i] {
            RegSetValueExW(
                hKey,
                szValueName.as_ptr(),
                0,
                REG_DWORD,
                core::ptr::addr_of!((*ConsoleInfo).ColorTable[i]) as PBYTE,
                core::mem::size_of::<u32>() as u32,
            );
        } else {
            RegDeleteValueW(hKey, szValueName.as_ptr());
        }
    }

    static CODEPAGE: [u16; 9] = [b'C' as u16, b'o' as u16, b'd' as u16, b'e' as u16, b'P' as u16, b'a' as u16, b'g' as u16, b'e' as u16, 0];
    if DefaultSettings != 0 || (*ConsoleInfo).CodePage != CP_ACP {
        RegSetValueExW(hKey, CODEPAGE.as_ptr(), 0, REG_DWORD, core::ptr::addr_of!((*ConsoleInfo).CodePage) as PBYTE, 4);
    } else {
        RegDeleteValueW(hKey, CODEPAGE.as_ptr());
    }

    static FACENAME: [u16; 9] = [b'F' as u16, b'a' as u16, b'c' as u16, b'e' as u16, b'N' as u16, b'a' as u16, b'm' as u16, b'e' as u16, 0];
    let mut facename_len = 0;
    while facename_len < (*ConsoleInfo).FaceName.len() && (*ConsoleInfo).FaceName[facename_len] != 0 {
        facename_len += 1;
    }
    if DefaultSettings != 0 || (*ConsoleInfo).FaceName[0] != 0 {
        RegSetValueExW(hKey, FACENAME.as_ptr(), 0, REG_SZ, (*ConsoleInfo).FaceName.as_ptr() as PBYTE, ((facename_len + 1) * 2) as u32);
    } else {
        RegDeleteValueW(hKey, FACENAME.as_ptr());
    }

    static FONTFAMILY: [u16; 11] = [b'F' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'F' as u16, b'a' as u16, b'm' as u16, b'i' as u16, b'l' as u16, b'y' as u16, 0];
    if DefaultSettings != 0 || (*ConsoleInfo).FontFamily != FF_DONTCARE {
        RegSetValueExW(hKey, FONTFAMILY.as_ptr(), 0, REG_DWORD, core::ptr::addr_of!((*ConsoleInfo).FontFamily) as PBYTE, 4);
    } else {
        RegDeleteValueW(hKey, FONTFAMILY.as_ptr());
    }

    static FONTSIZE: [u16; 9] = [b'F' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'S' as u16, b'i' as u16, b'z' as u16, b'e' as u16, 0];
    Storage = (((*ConsoleInfo).FontSize.X as u32) & 0xFFFF) | ((((*ConsoleInfo).FontSize.Y as u32) & 0xFFFF) << 16);
    if DefaultSettings != 0 || Storage != 0 {
        RegSetValueExW(hKey, FONTSIZE.as_ptr(), 0, REG_DWORD, core::ptr::addr_of!(Storage) as PBYTE, 4);
    } else {
        RegDeleteValueW(hKey, FONTSIZE.as_ptr());
    }

    static FONTWEIGHT: [u16; 11] = [b'F' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'W' as u16, b'e' as u16, b'i' as u16, b'g' as u16, b'h' as u16, b't' as u16, 0];
    if DefaultSettings != 0 || (*ConsoleInfo).FontWeight != FW_DONTCARE {
        RegSetValueExW(hKey, FONTWEIGHT.as_ptr(), 0, REG_DWORD, core::ptr::addr_of!((*ConsoleInfo).FontWeight) as PBYTE, 4);
    } else {
        RegDeleteValueW(hKey, FONTWEIGHT.as_ptr());
    }

    static HISTORYBUFFERSIZE: [u16; 18] = [b'H' as u16, b'i' as u16, b's' as u16, b't' as u16, b'o' as u16, b'r' as u16, b'y' as u16, b'B' as u16, b'u' as u16, b'f' as u16, b'f' as u16, b'e' as u16, b'r' as u16, b'S' as u16, b'i' as u16, b'z' as u16, b'e' as u16, 0];
    if DefaultSettings != 0 || (*ConsoleInfo).HistoryBufferSize != 50 {
        RegSetValueExW(hKey, HISTORYBUFFERSIZE.as_ptr(), 0, REG_DWORD, core::ptr::addr_of!((*ConsoleInfo).HistoryBufferSize) as PBYTE, 4);
    } else {
        RegDeleteValueW(hKey, HISTORYBUFFERSIZE.as_ptr());
    }

    static NUMBEROFHISTORYBUFFERS: [u16; 23] = [b'N' as u16, b'u' as u16, b'm' as u16, b'b' as u16, b'e' as u16, b'r' as u16, b'O' as u16, b'f' as u16, b'H' as u16, b'i' as u16, b's' as u16, b't' as u16, b'o' as u16, b'r' as u16, b'y' as u16, b'B' as u16, b'u' as u16, b'f' as u16, b'f' as u16, b'e' as u16, b'r' as u16, b's' as u16, 0];
    if DefaultSettings != 0 || (*ConsoleInfo).NumberOfHistoryBuffers != 4 {
        RegSetValueExW(hKey, NUMBEROFHISTORYBUFFERS.as_ptr(), 0, REG_DWORD, core::ptr::addr_of!((*ConsoleInfo).NumberOfHistoryBuffers) as PBYTE, 4);
    } else {
        RegDeleteValueW(hKey, NUMBEROFHISTORYBUFFERS.as_ptr());
    }

    static HISTORYNODUP: [u16; 13] = [b'H' as u16, b'i' as u16, b's' as u16, b't' as u16, b'o' as u16, b'r' as u16, b'y' as u16, b'N' as u16, b'o' as u16, b'D' as u16, b'u' as u16, b'p' as u16, 0];
    Storage = (*ConsoleInfo).HistoryNoDup as u32;
    if DefaultSettings != 0 || Storage != 0 {
        RegSetValueExW(hKey, HISTORYNODUP.as_ptr(), 0, REG_DWORD, core::ptr::addr_of!(Storage) as PBYTE, 4);
    } else {
        RegDeleteValueW(hKey, HISTORYNODUP.as_ptr());
    }

    static QUICKEDIT: [u16; 10] = [b'Q' as u16, b'u' as u16, b'i' as u16, b'c' as u16, b'k' as u16, b'E' as u16, b'd' as u16, b'i' as u16, b't' as u16, 0];
    Storage = (*ConsoleInfo).QuickEdit as u32;
    if DefaultSettings != 0 || Storage != 0 {
        RegSetValueExW(hKey, QUICKEDIT.as_ptr(), 0, REG_DWORD, core::ptr::addr_of!(Storage) as PBYTE, 4);
    } else {
        RegDeleteValueW(hKey, QUICKEDIT.as_ptr());
    }

    static INSERTMODE: [u16; 11] = [b'I' as u16, b'n' as u16, b's' as u16, b'e' as u16, b'r' as u16, b't' as u16, b'M' as u16, b'o' as u16, b'd' as u16, b'e' as u16, 0];
    Storage = (*ConsoleInfo).InsertMode as u32;
    if DefaultSettings != 0 || Storage != 1 {
        RegSetValueExW(hKey, INSERTMODE.as_ptr(), 0, REG_DWORD, core::ptr::addr_of!(Storage) as PBYTE, 4);
    } else {
        RegDeleteValueW(hKey, INSERTMODE.as_ptr());
    }

    static SCREENBUFFERSIZE: [u16; 17] = [b'S' as u16, b'c' as u16, b'r' as u16, b'e' as u16, b'e' as u16, b'n' as u16, b'B' as u16, b'u' as u16, b'f' as u16, b'f' as u16, b'e' as u16, b'r' as u16, b'S' as u16, b'i' as u16, b'z' as u16, b'e' as u16, 0];
    Storage = (((*ConsoleInfo).ScreenBufferSize.X as u32) & 0xFFFF) | ((((*ConsoleInfo).ScreenBufferSize.Y as u32) & 0xFFFF) << 16);
    let default_screen_buffer = (80u32 & 0xFFFF) | ((300u32 & 0xFFFF) << 16) ;
    if DefaultSettings != 0 || Storage != default_screen_buffer {
        RegSetValueExW(hKey, SCREENBUFFERSIZE.as_ptr(), 0, REG_DWORD, core::ptr::addr_of!(Storage) as PBYTE, 4);
    } else {
        RegDeleteValueW(hKey, SCREENBUFFERSIZE.as_ptr());
    }

    static FULLSCREEN: [u16; 11] = [b'F' as u16, b'u' as u16, b'l' as u16, b'l' as u16, b'S' as u16, b'c' as u16, b'r' as u16, b'e' as u16, b'e' as u16, b'n' as u16, 0];
    Storage = (*ConsoleInfo).FullScreen as u32;
    if DefaultSettings != 0 || Storage != 0 {
        RegSetValueExW(hKey, FULLSCREEN.as_ptr(), 0, REG_DWORD, core::ptr::addr_of!(Storage) as PBYTE, 4);
    } else {
        RegDeleteValueW(hKey, FULLSCREEN.as_ptr());
    }

    static WINDOWPOSITION: [u16; 15] = [b'W' as u16, b'i' as u16, b'n' as u16, b'd' as u16, b'o' as u16, b'w' as u16, b'P' as u16, b'o' as u16, b's' as u16, b'i' as u16, b't' as u16, b'i' as u16, b'o' as u16, b'n' as u16, 0];
    if (*ConsoleInfo).AutoPosition == 0 {
        Storage = (((*ConsoleInfo).WindowPosition.x as u32) & 0xFFFF) | ((((*ConsoleInfo).WindowPosition.y as u32) & 0xFFFF) << 16);
        RegSetValueExW(hKey, WINDOWPOSITION.as_ptr(), 0, REG_DWORD, core::ptr::addr_of!(Storage) as PBYTE, 4);
    } else {
        RegDeleteValueW(hKey, WINDOWPOSITION.as_ptr());
    }

    static WINDOWSIZE: [u16; 11] = [b'W' as u16, b'i' as u16, b'n' as u16, b'd' as u16, b'o' as u16, b'w' as u16, b'S' as u16, b'i' as u16, b'z' as u16, b'e' as u16, 0];
    Storage = (((*ConsoleInfo).WindowSize.X as u32) & 0xFFFF) | ((((*ConsoleInfo).WindowSize.Y as u32) & 0xFFFF) << 16);
    let default_window_size = (80u32 & 0xFFFF) | ((25u32 & 0xFFFF) << 16) ;
    if DefaultSettings != 0 || Storage != default_window_size {
        RegSetValueExW(hKey, WINDOWSIZE.as_ptr(), 0, REG_DWORD, core::ptr::addr_of!(Storage) as PBYTE, 4);
    } else {
        RegDeleteValueW(hKey, WINDOWSIZE.as_ptr());
    }

    static CURSORSIZE: [u16; 11] = [b'C' as u16, b'u' as u16, b'r' as u16, b's' as u16, b'o' as u16, b'r' as u16, b'S' as u16, b'i' as u16, b'z' as u16, b'e' as u16, 0];
    if DefaultSettings != 0 || (*ConsoleInfo).CursorSize != CSR_DEFAULT_CURSOR_SIZE {
        RegSetValueExW(hKey, CURSORSIZE.as_ptr(), 0, REG_DWORD, core::ptr::addr_of!((*ConsoleInfo).CursorSize) as PBYTE, 4);
    } else {
        RegDeleteValueW(hKey, CURSORSIZE.as_ptr());
    }

    static SCREENCOLORS: [u16; 13] = [b'S' as u16, b'c' as u16, b'r' as u16, b'e' as u16, b'e' as u16, b'n' as u16, b'C' as u16, b'o' as u16, b'l' as u16, b'o' as u16, b'r' as u16, b's' as u16, 0];
    Storage = (*ConsoleInfo).ScreenAttributes as u32;
    if DefaultSettings != 0 || Storage != DEFAULT_SCREEN_ATTRIB as u32 {
        RegSetValueExW(hKey, SCREENCOLORS.as_ptr(), 0, REG_DWORD, core::ptr::addr_of!(Storage) as PBYTE, 4);
    } else {
        RegDeleteValueW(hKey, SCREENCOLORS.as_ptr());
    }

    static POPUPCOLORS: [u16; 12] = [b'P' as u16, b'o' as u16, b'p' as u16, b'u' as u16, b'p' as u16, b'C' as u16, b'o' as u16, b'l' as u16, b'o' as u16, b'r' as u16, b's' as u16, 0];
    Storage = (*ConsoleInfo).PopupAttributes as u32;
    if DefaultSettings != 0 || Storage != DEFAULT_POPUP_ATTRIB as u32 {
        RegSetValueExW(hKey, POPUPCOLORS.as_ptr(), 0, REG_DWORD, core::ptr::addr_of!(Storage) as PBYTE, 4);
    } else {
        RegDeleteValueW(hKey, POPUPCOLORS.as_ptr());
    }

    RegCloseKey(hKey);
    1
}

#[no_mangle]
pub unsafe extern "C" fn ConCfgInitDefaultSettings(ConsoleInfo: PCONSOLE_STATE_INFO) {
    if ConsoleInfo.is_null() {
        return;
    }

    static VGA: [u16; 4] = [b'V' as u16, b'G' as u16, b'A' as u16, 0];
    for i in 0..4.min((*ConsoleInfo).FaceName.len()) {
        (*ConsoleInfo).FaceName[i] = VGA[i];
    }

    (*ConsoleInfo).FontSize.X = 0;
    (*ConsoleInfo).FontSize.Y = 16;
    (*ConsoleInfo).FontFamily = FF_DONTCARE;
    (*ConsoleInfo).FontWeight = FW_NORMAL;

    (*ConsoleInfo).HistoryBufferSize = 50;
    (*ConsoleInfo).NumberOfHistoryBuffers = 4;
    (*ConsoleInfo).HistoryNoDup = 0;

    (*ConsoleInfo).QuickEdit = 0;
    (*ConsoleInfo).InsertMode = 1;

    (*ConsoleInfo).ScreenBufferSize.X = 80;
    (*ConsoleInfo).ScreenBufferSize.Y = 300;
    (*ConsoleInfo).WindowSize.X = 80;
    (*ConsoleInfo).WindowSize.Y = 25;

    (*ConsoleInfo).FullScreen = 0;
    (*ConsoleInfo).AutoPosition = 1;
    (*ConsoleInfo).WindowPosition.x = 0;
    (*ConsoleInfo).WindowPosition.y = 0;

    (*ConsoleInfo).CursorSize = CSR_DEFAULT_CURSOR_SIZE;

    (*ConsoleInfo).ScreenAttributes = DEFAULT_SCREEN_ATTRIB;
    (*ConsoleInfo).PopupAttributes = DEFAULT_POPUP_ATTRIB;

    core::ptr::copy_nonoverlapping(
        S_COLORS.as_ptr(),
        (*ConsoleInfo).ColorTable.as_mut_ptr(),
        S_COLORS.len(),
    );

    (*ConsoleInfo).CodePage = GetOEMCP();
}

#[no_mangle]
pub unsafe extern "C" fn ConCfgGetDefaultSettings(ConsoleInfo: PCONSOLE_STATE_INFO) {
    if ConsoleInfo.is_null() {
        return;
    }

    ConCfgInitDefaultSettings(ConsoleInfo);

    if ConCfgReadUserSettings(ConsoleInfo, 1) == 0 {
        ConCfgWriteUserSettings(ConsoleInfo, 1);
    }
}