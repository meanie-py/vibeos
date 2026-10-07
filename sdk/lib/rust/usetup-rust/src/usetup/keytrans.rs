use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut KeyTable: [WORD; 128] = [
    0 as ::core::ffi::c_int as WORD,
    VK_ESCAPE as WORD,
    0x31 as ::core::ffi::c_int as WORD,
    0x32 as ::core::ffi::c_int as WORD,
    0x33 as ::core::ffi::c_int as WORD,
    0x34 as ::core::ffi::c_int as WORD,
    0x35 as ::core::ffi::c_int as WORD,
    0x36 as ::core::ffi::c_int as WORD,
    0x37 as ::core::ffi::c_int as WORD,
    0x38 as ::core::ffi::c_int as WORD,
    0x39 as ::core::ffi::c_int as WORD,
    0x30 as ::core::ffi::c_int as WORD,
    VK_OEM_MINUS as WORD,
    VK_OEM_PLUS as WORD,
    VK_BACK as WORD,
    VK_TAB as WORD,
    0x51 as ::core::ffi::c_int as WORD,
    0x57 as ::core::ffi::c_int as WORD,
    0x45 as ::core::ffi::c_int as WORD,
    0x52 as ::core::ffi::c_int as WORD,
    0x54 as ::core::ffi::c_int as WORD,
    0x59 as ::core::ffi::c_int as WORD,
    0x55 as ::core::ffi::c_int as WORD,
    0x49 as ::core::ffi::c_int as WORD,
    0x4f as ::core::ffi::c_int as WORD,
    0x50 as ::core::ffi::c_int as WORD,
    VK_OEM_4 as WORD,
    VK_OEM_6 as WORD,
    VK_RETURN as WORD,
    VK_CONTROL as WORD,
    0x41 as ::core::ffi::c_int as WORD,
    0x53 as ::core::ffi::c_int as WORD,
    0x44 as ::core::ffi::c_int as WORD,
    0x46 as ::core::ffi::c_int as WORD,
    0x47 as ::core::ffi::c_int as WORD,
    0x48 as ::core::ffi::c_int as WORD,
    0x4a as ::core::ffi::c_int as WORD,
    0x4b as ::core::ffi::c_int as WORD,
    0x4c as ::core::ffi::c_int as WORD,
    VK_OEM_1 as WORD,
    VK_OEM_7 as WORD,
    0xc0 as ::core::ffi::c_int as WORD,
    VK_LSHIFT as WORD,
    VK_OEM_5 as WORD,
    0x5a as ::core::ffi::c_int as WORD,
    0x58 as ::core::ffi::c_int as WORD,
    0x43 as ::core::ffi::c_int as WORD,
    0x56 as ::core::ffi::c_int as WORD,
    0x42 as ::core::ffi::c_int as WORD,
    0x4e as ::core::ffi::c_int as WORD,
    0x4d as ::core::ffi::c_int as WORD,
    VK_OEM_COMMA as WORD,
    VK_OEM_PERIOD as WORD,
    VK_OEM_2 as WORD,
    VK_RSHIFT as WORD,
    VK_MULTIPLY as WORD,
    VK_LMENU as WORD,
    VK_SPACE as WORD,
    VK_CAPITAL as WORD,
    VK_F1 as WORD,
    VK_F2 as WORD,
    VK_F3 as WORD,
    VK_F4 as WORD,
    VK_F5 as WORD,
    VK_F6 as WORD,
    VK_F7 as WORD,
    VK_F8 as WORD,
    VK_F9 as WORD,
    VK_F10 as WORD,
    VK_NUMLOCK as WORD,
    VK_SCROLL as WORD,
    VK_HOME as WORD,
    VK_UP as WORD,
    VK_PRIOR as WORD,
    VK_SUBTRACT as WORD,
    VK_LEFT as WORD,
    0 as ::core::ffi::c_int as WORD,
    VK_RIGHT as WORD,
    VK_ADD as WORD,
    VK_END as WORD,
    VK_DOWN as WORD,
    VK_NEXT as WORD,
    VK_INSERT as WORD,
    VK_DELETE as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    VK_F11 as WORD,
    VK_F12 as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
];

static mut KeyTableEnhanced: [WORD; 128] = [
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    VK_RETURN as WORD,
    VK_RCONTROL as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    VK_DIVIDE as WORD,
    0 as ::core::ffi::c_int as WORD,
    VK_SNAPSHOT as WORD,
    VK_RMENU as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    VK_HOME as WORD,
    VK_UP as WORD,
    VK_PRIOR as WORD,
    0 as ::core::ffi::c_int as WORD,
    VK_LEFT as WORD,
    0 as ::core::ffi::c_int as WORD,
    VK_RIGHT as WORD,
    0 as ::core::ffi::c_int as WORD,
    VK_END as WORD,
    VK_DOWN as WORD,
    VK_NEXT as WORD,
    VK_INSERT as WORD,
    VK_DELETE as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
];

static mut KeyTableNumlock: [WORD; 128] = [
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    VK_NUMPAD7 as WORD,
    VK_NUMPAD8 as WORD,
    VK_NUMPAD9 as WORD,
    0 as ::core::ffi::c_int as WORD,
    VK_NUMPAD4 as WORD,
    VK_NUMPAD5 as WORD,
    VK_NUMPAD6 as WORD,
    0 as ::core::ffi::c_int as WORD,
    VK_NUMPAD1 as WORD,
    VK_NUMPAD2 as WORD,
    VK_NUMPAD3 as WORD,
    VK_NUMPAD0 as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
    0 as ::core::ffi::c_int as WORD,
];

#[no_mangle]
pub static mut ScanToAscii: [SCANTOASCII; 67] = [
    _SCANTOASCII {
        ScanCode: 0x1e as USHORT,
        Enhanced: 0 as USHORT,
        Normal: 'a' as i32 as UCHAR,
        Shift: 'A' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: TRUE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x30 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: 'b' as i32 as UCHAR,
        Shift: 'B' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: TRUE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x2e as USHORT,
        Enhanced: 0 as USHORT,
        Normal: 'c' as i32 as UCHAR,
        Shift: 'C' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: TRUE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x20 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: 'd' as i32 as UCHAR,
        Shift: 'D' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: TRUE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x12 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: 'e' as i32 as UCHAR,
        Shift: 'E' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: TRUE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x21 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: 'f' as i32 as UCHAR,
        Shift: 'F' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: TRUE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x22 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: 'g' as i32 as UCHAR,
        Shift: 'G' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: TRUE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x23 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: 'h' as i32 as UCHAR,
        Shift: 'H' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: TRUE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x17 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: 'i' as i32 as UCHAR,
        Shift: 'I' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: TRUE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x24 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: 'j' as i32 as UCHAR,
        Shift: 'J' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: TRUE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x25 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: 'k' as i32 as UCHAR,
        Shift: 'K' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: TRUE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x26 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: 'l' as i32 as UCHAR,
        Shift: 'L' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: TRUE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x32 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: 'm' as i32 as UCHAR,
        Shift: 'M' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: TRUE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x31 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: 'n' as i32 as UCHAR,
        Shift: 'N' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: TRUE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x18 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: 'o' as i32 as UCHAR,
        Shift: 'O' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: TRUE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x19 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: 'p' as i32 as UCHAR,
        Shift: 'P' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: TRUE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x10 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: 'q' as i32 as UCHAR,
        Shift: 'Q' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: TRUE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x13 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: 'r' as i32 as UCHAR,
        Shift: 'R' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: TRUE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x1f as USHORT,
        Enhanced: 0 as USHORT,
        Normal: 's' as i32 as UCHAR,
        Shift: 'S' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: TRUE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x14 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: 't' as i32 as UCHAR,
        Shift: 'T' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: TRUE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x16 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: 'u' as i32 as UCHAR,
        Shift: 'U' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: TRUE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x2f as USHORT,
        Enhanced: 0 as USHORT,
        Normal: 'v' as i32 as UCHAR,
        Shift: 'V' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: TRUE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x11 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: 'w' as i32 as UCHAR,
        Shift: 'W' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: TRUE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x2d as USHORT,
        Enhanced: 0 as USHORT,
        Normal: 'x' as i32 as UCHAR,
        Shift: 'X' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: TRUE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x15 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: 'y' as i32 as UCHAR,
        Shift: 'Y' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: TRUE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x2c as USHORT,
        Enhanced: 0 as USHORT,
        Normal: 'z' as i32 as UCHAR,
        Shift: 'Z' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: TRUE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x2 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: '1' as i32 as UCHAR,
        Shift: '!' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x3 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: '2' as i32 as UCHAR,
        Shift: '@' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x4 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: '3' as i32 as UCHAR,
        Shift: '#' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x5 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: '4' as i32 as UCHAR,
        Shift: '$' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x6 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: '5' as i32 as UCHAR,
        Shift: '%' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x7 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: '6' as i32 as UCHAR,
        Shift: '^' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x8 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: '7' as i32 as UCHAR,
        Shift: '&' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x9 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: '8' as i32 as UCHAR,
        Shift: '*' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0xa as USHORT,
        Enhanced: 0 as USHORT,
        Normal: '9' as i32 as UCHAR,
        Shift: '(' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0xb as USHORT,
        Enhanced: 0 as USHORT,
        Normal: '0' as i32 as UCHAR,
        Shift: ')' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x29 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: '\'' as i32 as UCHAR,
        Shift: '~' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0xc as USHORT,
        Enhanced: 0 as USHORT,
        Normal: '-' as i32 as UCHAR,
        Shift: '_' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0xd as USHORT,
        Enhanced: 0 as USHORT,
        Normal: '=' as i32 as UCHAR,
        Shift: '+' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x1a as USHORT,
        Enhanced: 0 as USHORT,
        Normal: '[' as i32 as UCHAR,
        Shift: '{' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x1b as USHORT,
        Enhanced: 0 as USHORT,
        Normal: ']' as i32 as UCHAR,
        Shift: '}' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x2b as USHORT,
        Enhanced: 0 as USHORT,
        Normal: '\\' as i32 as UCHAR,
        Shift: '|' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x27 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: ';' as i32 as UCHAR,
        Shift: ':' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x28 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: '\'' as i32 as UCHAR,
        Shift: '"' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x33 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: ',' as i32 as UCHAR,
        Shift: '<' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x34 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: '.' as i32 as UCHAR,
        Shift: '>' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x35 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: '/' as i32 as UCHAR,
        Shift: '?' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x4f as USHORT,
        Enhanced: 0 as USHORT,
        Normal: 0 as UCHAR,
        Shift: 0 as UCHAR,
        NumLock: '1' as i32 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x50 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: 0 as UCHAR,
        Shift: 0 as UCHAR,
        NumLock: '2' as i32 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x51 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: 0 as UCHAR,
        Shift: 0 as UCHAR,
        NumLock: '3' as i32 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x4b as USHORT,
        Enhanced: 0 as USHORT,
        Normal: 0 as UCHAR,
        Shift: 0 as UCHAR,
        NumLock: '4' as i32 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x4c as USHORT,
        Enhanced: 0 as USHORT,
        Normal: 0 as UCHAR,
        Shift: 0 as UCHAR,
        NumLock: '5' as i32 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x4d as USHORT,
        Enhanced: 0 as USHORT,
        Normal: 0 as UCHAR,
        Shift: 0 as UCHAR,
        NumLock: '6' as i32 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x47 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: 0 as UCHAR,
        Shift: 0 as UCHAR,
        NumLock: '7' as i32 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x48 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: 0 as UCHAR,
        Shift: 0 as UCHAR,
        NumLock: '8' as i32 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x49 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: 0 as UCHAR,
        Shift: 0 as UCHAR,
        NumLock: '9' as i32 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x52 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: 0 as UCHAR,
        Shift: 0 as UCHAR,
        NumLock: '0' as i32 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x4a as USHORT,
        Enhanced: 0 as USHORT,
        Normal: '-' as i32 as UCHAR,
        Shift: '-' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x4e as USHORT,
        Enhanced: 0 as USHORT,
        Normal: '+' as i32 as UCHAR,
        Shift: '+' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x37 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: '*' as i32 as UCHAR,
        Shift: '*' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x35 as USHORT,
        Enhanced: 1 as USHORT,
        Normal: '/' as i32 as UCHAR,
        Shift: '/' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x53 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: 0 as UCHAR,
        Shift: 0 as UCHAR,
        NumLock: '.' as i32 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x39 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: ' ' as i32 as UCHAR,
        Shift: ' ' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x1c as USHORT,
        Enhanced: 0 as USHORT,
        Normal: '\r' as i32 as UCHAR,
        Shift: '\r' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0x1c as USHORT,
        Enhanced: 1 as USHORT,
        Normal: '\r' as i32 as UCHAR,
        Shift: '\r' as i32 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0xe as USHORT,
        Enhanced: 0 as USHORT,
        Normal: 0x8 as UCHAR,
        Shift: 0x8 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
    _SCANTOASCII {
        ScanCode: 0 as USHORT,
        Enhanced: 0 as USHORT,
        Normal: 0 as UCHAR,
        Shift: 0 as UCHAR,
        NumLock: 0 as UCHAR,
        bCAPS: FALSE as UCHAR,
    },
];


pub type PSCANTOASCII = *mut SCANTOASCII;

unsafe extern "C" fn IntUpdateControlKeyState(
    hConsoleInput: HANDLE,
    State: LPDWORD,
    InputData: PKEYBOARD_INPUT_DATA,
) {
    let mut Value: DWORD = 0;
    let mut oldState: DWORD;
    let mut newState: DWORD;

    if (*InputData).Flags & (KEY_E1 as u16) != 0 {
        return;
    }

    oldState = *State;
    newState = oldState;

    if (*InputData).Flags & (KEY_E0 as u16) == 0 {
        match (*InputData).MakeCode {
            0x2a | 0x36 => {
                Value = SHIFT_PRESSED;
            }
            0x1d => {
                Value = LEFT_CTRL_PRESSED;
            }
            0x38 => {
                Value = LEFT_ALT_PRESSED;
            }
            0x3A => {
                if (*InputData).Flags & (KEY_BREAK as u16) == 0 {
                    newState ^= CAPSLOCK_ON;
                }
            }
            0x45 => {
                if (*InputData).Flags & (KEY_BREAK as u16) == 0 {
                    newState ^= NUMLOCK_ON;
                }
            }
            0x46 => {
                if (*InputData).Flags & (KEY_BREAK as u16) == 0 {
                    newState ^= SCROLLLOCK_ON;
                }
            }
            _ => return,
        }
    } else {
        match (*InputData).MakeCode {
            0x1d => {
                Value = RIGHT_CTRL_PRESSED;
            }
            0x38 => {
                Value = RIGHT_ALT_PRESSED;
            }
            _ => return,
        }
    }

    if (oldState ^ newState) & (NUMLOCK_ON | CAPSLOCK_ON | SCROLLLOCK_ON) != 0 {
        let mut IoStatusBlock: _IO_STATUS_BLOCK = { let mut __lit: _IO_STATUS_BLOCK = core::mem::zeroed(); __lit.__bindgen_anon_1.Status = 0; __lit.Information = 0; __lit };
        let mut Status: NTSTATUS;
        let mut kip: KEYBOARD_INDICATOR_PARAMETERS = KEYBOARD_INDICATOR_PARAMETERS {
            UnitId: 0,
            LedFlags: 0,
        };

        kip.LedFlags = 0;
        kip.UnitId = 0;

        if newState & NUMLOCK_ON != 0 {
            kip.LedFlags |= (KEYBOARD_NUM_LOCK_ON) as u16;
        }

        if newState & CAPSLOCK_ON != 0 {
            kip.LedFlags |= (KEYBOARD_CAPS_LOCK_ON) as u16;
        }

        if newState & SCROLLLOCK_ON != 0 {
            kip.LedFlags |= (KEYBOARD_SCROLL_LOCK_ON) as u16;
        }

        DPRINT!("NtDeviceIoControlFile dwLeds=%x\n", kip.LedFlags);

        Status = NtDeviceIoControlFile(
            hConsoleInput,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            &mut IoStatusBlock,
            IOCTL_KEYBOARD_SET_INDICATORS,
            &mut kip as *mut KEYBOARD_INDICATOR_PARAMETERS as PVOID,
            core::mem::size_of::<KEYBOARD_INDICATOR_PARAMETERS>() as u32,
            core::ptr::null_mut(),
            0,
        );

        if ((NT_SUCCESS!(Status)) == 0) {
            DPRINT1!("NtDeviceIoControlFile(IOCTL_KEYBOARD_SET_INDICATORS) failed (Status %lx)\n", Status);
        }
    } else if (*InputData).Flags & (KEY_BREAK as u16) != 0 {
        newState &= !Value;
    } else {
        newState |= Value;
    }

    *State = newState;
}

unsafe extern "C" fn IntVKFromKbdInput(
    InputData: PKEYBOARD_INPUT_DATA,
    KeyState: DWORD,
) -> DWORD {
    if KeyState & (ENHANCED_KEY as DWORD) == 0 {
        if KeyState & (NUMLOCK_ON as DWORD) != 0
            && KeyTableNumlock[((*InputData).MakeCode as i32
                & 0x7f as i32) as usize] as i32 != 0
        {
            let _ = (*InputData).MakeCode as i32 & 0x7f as i32;
            return KeyTableNumlock[((*InputData).MakeCode as i32
                & 0x7f as i32) as usize] as DWORD;
        }
        let _ = (*InputData).MakeCode as i32 & 0x7f as i32;
        return KeyTable[((*InputData).MakeCode as i32
            & 0x7f as i32) as usize] as DWORD;
    }
    let _ = (*InputData).MakeCode as i32 & 0x7f as i32;
    return KeyTableEnhanced[((*InputData).MakeCode as i32
        & 0x7f as i32) as usize] as DWORD;
}

unsafe extern "C" fn IntAsciiFromInput(
    InputData: PKEYBOARD_INPUT_DATA,
    mut KeyState: DWORD,
) -> UCHAR {
    let mut Counter: UINT = 0;
    let mut Enhanced: USHORT = 0;

    if KeyState & ENHANCED_KEY != 0 {
        Enhanced = 1;
    }

    while ScanToAscii[Counter as usize].ScanCode as i32 != 0 as i32 {
        if ScanToAscii[Counter as usize].ScanCode as i32 == (*InputData).MakeCode as i32
            && ScanToAscii[Counter as usize].Enhanced as i32 == Enhanced as i32
        {
            if ScanToAscii[Counter as usize].NumLock != 0 {
                if KeyState & NUMLOCK_ON != 0
                    && KeyState & SHIFT_PRESSED == 0
                {
                    return ScanToAscii[Counter as usize].NumLock;
                } else {
                    return ScanToAscii[Counter as usize].Normal;
                }
            }

            if KeyState & CAPSLOCK_ON != 0
                && ScanToAscii[Counter as usize].bCAPS != 0
            {
                KeyState ^= SHIFT_PRESSED;
            }

            if KeyState & SHIFT_PRESSED != 0 {
                return ScanToAscii[Counter as usize].Shift;
            }

            return ScanToAscii[Counter as usize].Normal;
        }
        Counter = Counter.wrapping_add(1);
    }

    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn IntTranslateKey(hConsoleInput: HANDLE, InputData: PKEYBOARD_INPUT_DATA, Event: *mut KEY_EVENT_RECORD) -> NTSTATUS {
    static mut dwControlKeyState: DWORD = 0;

    RtlZeroMemory!(Event as *mut core::ffi::c_void, core::mem::size_of::<KEY_EVENT_RECORD>());

    if (*InputData).Flags & KEY_BREAK as u16 == 0 {
        (*Event).bKeyDown = (TRUE) as i32;
    } else {
        (*Event).bKeyDown = (FALSE) as i32;
    }

    (*Event).wRepeatCount = 1;
    (*Event).wVirtualScanCode = (*InputData).MakeCode;

    DPRINT!("Translating: %x\n", (*InputData).MakeCode);

    IntUpdateControlKeyState(hConsoleInput, &raw mut dwControlKeyState, InputData);
    (*Event).dwControlKeyState = dwControlKeyState;

    if (*InputData).Flags & KEY_E0 as u16 != 0 {
        (*Event).dwControlKeyState |= ENHANCED_KEY;
    }

    (*Event).wVirtualKeyCode = (IntVKFromKbdInput(InputData, (*Event).dwControlKeyState)) as u16;

    DPRINT!("Result: %x\n", (*Event).wVirtualKeyCode);

    if (*Event).bKeyDown != 0 {
        (*Event).uChar.AsciiChar = (IntAsciiFromInput(InputData, (*Event).dwControlKeyState)) as i8;
        DPRINT!("Char: %x\n", (*Event).uChar.AsciiChar);
    } else {
        (*Event).uChar.AsciiChar = 0;
    }

    return STATUS_SUCCESS;
}
