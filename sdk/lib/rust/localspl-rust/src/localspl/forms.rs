// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_98C6C31426EF: [u16; 45] = [
    0x0053u16, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F,
    0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072,
    0x006F, 0x006C, 0x005C, 0x0050, 0x0072, 0x0069, 0x006E, 0x0074, 0x005C, 0x0046,
    0x006F, 0x0072, 0x006D, 0x0073, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut FormList: LIST_ENTRY = _LIST_ENTRY {
    Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
    Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
};

static mut _dwLastForm: DWORD = 0;

static mut dwFormInfo1Offsets: [DWORD; 2] = [
    core::mem::offset_of!(FORM_INFO_1W, pName) as DWORD,
    MAXDWORD as DWORD,
];

static mut dwFormInfo2Offsets: [DWORD; 5] = [
    core::mem::offset_of!(FORM_INFO_2W, pName) as DWORD,
    core::mem::offset_of!(FORM_INFO_2W, pKeyword) as DWORD,
    core::mem::offset_of!(FORM_INFO_2W, pMuiDll) as DWORD,
    core::mem::offset_of!(FORM_INFO_2W, pDisplayName) as DWORD,
    MAXDWORD as DWORD,
];

#[no_mangle]
pub static mut wszLetter: [WCHAR; 7] =
    unsafe { ::core::mem::transmute::<[u8; 14], [WCHAR; 7]>(*b"L\0e\0t\0t\0e\0r\0\0\0") };

#[no_mangle]
pub static mut wszLetterSmall: [WCHAR; 13] = unsafe {
    ::core::mem::transmute::<[u8; 26], [WCHAR; 13]>(*b"L\0e\0t\0t\0e\0r\0 \0S\0m\0a\0l\0l\0\0\0")
};

#[no_mangle]
pub static mut wszTabloid: [WCHAR; 8] =
    unsafe { ::core::mem::transmute::<[u8; 16], [WCHAR; 8]>(*b"T\0a\0b\0l\0o\0i\0d\0\0\0") };

#[no_mangle]
pub static mut wszLedger: [WCHAR; 7] =
    unsafe { ::core::mem::transmute::<[u8; 14], [WCHAR; 7]>(*b"L\0e\0d\0g\0e\0r\0\0\0") };

#[no_mangle]
pub static mut wszLegal: [WCHAR; 6] =
    unsafe { ::core::mem::transmute::<[u8; 12], [WCHAR; 6]>(*b"L\0e\0g\0a\0l\0\0\0") };

#[no_mangle]
pub static mut wszStatement: [WCHAR; 10] =
    unsafe { ::core::mem::transmute::<[u8; 20], [WCHAR; 10]>(*b"S\0t\0a\0t\0e\0m\0e\0n\0t\0\0\0") };

#[no_mangle]
pub static mut wszExecutive: [WCHAR; 10] =
    unsafe { ::core::mem::transmute::<[u8; 20], [WCHAR; 10]>(*b"E\0x\0e\0c\0u\0t\0i\0v\0e\0\0\0") };

#[no_mangle]
pub static mut wszA3: [WCHAR; 3] =
    unsafe { ::core::mem::transmute::<[u8; 6], [WCHAR; 3]>(*b"A\x003\0\0\0") };

#[no_mangle]
pub static mut wszA4: [WCHAR; 3] =
    unsafe { ::core::mem::transmute::<[u8; 6], [WCHAR; 3]>(*b"A\x004\0\0\0") };

#[no_mangle]
pub static mut wszA4Small: [WCHAR; 9] =
    unsafe { ::core::mem::transmute::<[u8; 18], [WCHAR; 9]>(*b"A\x004\0 \0S\0m\0a\0l\0l\0\0\0") };

#[no_mangle]
pub static mut wszA5: [WCHAR; 3] =
    unsafe { ::core::mem::transmute::<[u8; 6], [WCHAR; 3]>(*b"A\x005\0\0\0") };

#[no_mangle]
pub static mut wszB4JIS: [WCHAR; 9] =
    unsafe { ::core::mem::transmute::<[u8; 18], [WCHAR; 9]>(*b"B\x004\0 \0(\0J\0I\0S\0)\0\0\0") };

#[no_mangle]
pub static mut wszB5JIS: [WCHAR; 9] =
    unsafe { ::core::mem::transmute::<[u8; 18], [WCHAR; 9]>(*b"B\x005\0 \0(\0J\0I\0S\0)\0\0\0") };

#[no_mangle]
pub static mut wszFolio: [WCHAR; 6] =
    unsafe { ::core::mem::transmute::<[u8; 12], [WCHAR; 6]>(*b"F\0o\0l\0i\0o\0\0\0") };

#[no_mangle]
pub static mut wszQuarto: [WCHAR; 7] =
    unsafe { ::core::mem::transmute::<[u8; 14], [WCHAR; 7]>(*b"Q\0u\0a\0r\0t\0o\0\0\0") };

#[no_mangle]
pub static mut wsz10x14: [WCHAR; 8] =
    unsafe { ::core::mem::transmute::<[u8; 16], [WCHAR; 8]>(*b"1\x000\0 \0x\0 \x001\x004\0\0\0") };

#[no_mangle]
pub static mut wsz11x17: [WCHAR; 8] =
    unsafe { ::core::mem::transmute::<[u8; 16], [WCHAR; 8]>(*b"1\x001\0 \0x\0 \x001\x007\0\0\0") };

#[no_mangle]
pub static mut wszNote: [WCHAR; 5] =
    unsafe { ::core::mem::transmute::<[u8; 10], [WCHAR; 5]>(*b"N\0o\0t\0e\0\0\0") };

#[no_mangle]
pub static mut wszEnvelope9: [WCHAR; 12] = unsafe {
    ::core::mem::transmute::<[u8; 24], [WCHAR; 12]>(*b"E\0n\0v\0e\0l\0o\0p\0e\0 \0#\09\0\0\0")
};

#[no_mangle]
pub static mut wszEnvelope10: [WCHAR; 13] = unsafe {
    ::core::mem::transmute::<[u8; 26], [WCHAR; 13]>(
        *b"E\0n\0v\0e\0l\0o\0p\0e\0 \0#\x001\x000\0\0\0",
    )
};

#[no_mangle]
pub static mut wszEnvelope11: [WCHAR; 13] = unsafe {
    ::core::mem::transmute::<[u8; 26], [WCHAR; 13]>(
        *b"E\0n\0v\0e\0l\0o\0p\0e\0 \0#\x001\x001\0\0\0",
    )
};

#[no_mangle]
pub static mut wszEnvelope12: [WCHAR; 13] = unsafe {
    ::core::mem::transmute::<[u8; 26], [WCHAR; 13]>(
        *b"E\0n\0v\0e\0l\0o\0p\0e\0 \0#\x001\x002\0\0\0",
    )
};

#[no_mangle]
pub static mut wszEnvelope14: [WCHAR; 13] = unsafe {
    ::core::mem::transmute::<[u8; 26], [WCHAR; 13]>(
        *b"E\0n\0v\0e\0l\0o\0p\0e\0 \0#\x001\x004\0\0\0",
    )
};

#[no_mangle]
pub static mut wszCsizesheet: [WCHAR; 13] = unsafe {
    ::core::mem::transmute::<[u8; 26], [WCHAR; 13]>(*b"C\0 \0s\0i\0z\0e\0 \0s\0h\0e\0e\0t\0\0\0")
};

#[no_mangle]
pub static mut wszDsizesheet: [WCHAR; 13] = unsafe {
    ::core::mem::transmute::<[u8; 26], [WCHAR; 13]>(*b"D\0 \0s\0i\0z\0e\0 \0s\0h\0e\0e\0t\0\0\0")
};

#[no_mangle]
pub static mut wszEsizesheet: [WCHAR; 13] = unsafe {
    ::core::mem::transmute::<[u8; 26], [WCHAR; 13]>(*b"E\0 \0s\0i\0z\0e\0 \0s\0h\0e\0e\0t\0\0\0")
};

#[no_mangle]
pub static mut wszEnvelopeDL: [WCHAR; 12] = unsafe {
    ::core::mem::transmute::<[u8; 24], [WCHAR; 12]>(*b"E\0n\0v\0e\0l\0o\0p\0e\0 \0D\0L\0\0\0")
};

#[no_mangle]
pub static mut wszEnvelopeC5: [WCHAR; 12] = unsafe {
    ::core::mem::transmute::<[u8; 24], [WCHAR; 12]>(*b"E\0n\0v\0e\0l\0o\0p\0e\0 \0C\x005\0\0\0")
};

#[no_mangle]
pub static mut wszEnvelopeC3: [WCHAR; 12] = unsafe {
    ::core::mem::transmute::<[u8; 24], [WCHAR; 12]>(*b"E\0n\0v\0e\0l\0o\0p\0e\0 \0C\x003\0\0\0")
};

#[no_mangle]
pub static mut wszEnvelopeC4: [WCHAR; 12] = unsafe {
    ::core::mem::transmute::<[u8; 24], [WCHAR; 12]>(*b"E\0n\0v\0e\0l\0o\0p\0e\0 \0C\x004\0\0\0")
};

#[no_mangle]
pub static mut wszEnvelopeC6: [WCHAR; 12] = unsafe {
    ::core::mem::transmute::<[u8; 24], [WCHAR; 12]>(*b"E\0n\0v\0e\0l\0o\0p\0e\0 \0C\x006\0\0\0")
};

#[no_mangle]
pub static mut wszEnvelope65: [WCHAR; 12] = unsafe {
    ::core::mem::transmute::<[u8; 24], [WCHAR; 12]>(*b"E\0n\0v\0e\0l\0o\0p\0e\0 \x006\x005\0\0\0")
};

#[no_mangle]
pub static mut wszEnvelopeB4: [WCHAR; 12] = unsafe {
    ::core::mem::transmute::<[u8; 24], [WCHAR; 12]>(*b"E\0n\0v\0e\0l\0o\0p\0e\0 \0B\x004\0\0\0")
};

#[no_mangle]
pub static mut wszEnvelopeB5: [WCHAR; 12] = unsafe {
    ::core::mem::transmute::<[u8; 24], [WCHAR; 12]>(*b"E\0n\0v\0e\0l\0o\0p\0e\0 \0B\x005\0\0\0")
};

#[no_mangle]
pub static mut wszEnvelopeB6: [WCHAR; 12] = unsafe {
    ::core::mem::transmute::<[u8; 24], [WCHAR; 12]>(*b"E\0n\0v\0e\0l\0o\0p\0e\0 \0B\x006\0\0\0")
};

#[no_mangle]
pub static mut wszEnvelope: [WCHAR; 9] =
    unsafe { ::core::mem::transmute::<[u8; 18], [WCHAR; 9]>(*b"E\0n\0v\0e\0l\0o\0p\0e\0\0\0") };

#[no_mangle]
pub static mut wszEnvelopeMonarch: [WCHAR; 17] = unsafe {
    ::core::mem::transmute::<[u8; 34], [WCHAR; 17]>(
        *b"E\0n\0v\0e\0l\0o\0p\0e\0 \0M\0o\0n\0a\0r\0c\0h\0\0\0",
    )
};

#[no_mangle]
pub static mut wsz634Envelope: [WCHAR; 15] = unsafe {
    ::core::mem::transmute::<[u8; 30], [WCHAR; 15]>(
        *b"6\0 \x003\0/\x004\0 \0E\0n\0v\0e\0l\0o\0p\0e\0\0\0",
    )
};

#[no_mangle]
pub static mut wszUSStdFanfold: [WCHAR; 15] = unsafe {
    ::core::mem::transmute::<[u8; 30], [WCHAR; 15]>(
        *b"U\0S\0 \0S\0t\0d\0 \0F\0a\0n\0f\0o\0l\0d\0\0\0",
    )
};

#[no_mangle]
pub static mut wszGermanStdFanfold: [WCHAR; 19] = unsafe {
    ::core::mem::transmute::<[u8; 38], [WCHAR; 19]>(
        *b"G\0e\0r\0m\0a\0n\0 \0S\0t\0d\0 \0F\0a\0n\0f\0o\0l\0d\0\0\0",
    )
};

#[no_mangle]
pub static mut wszGermanLegalFanfold: [WCHAR; 21] = unsafe {
    ::core::mem::transmute::<[u8; 42], [WCHAR; 21]>(
        *b"G\0e\0r\0m\0a\0n\0 \0L\0e\0g\0a\0l\0 \0F\0a\0n\0f\0o\0l\0d\0\0\0",
    )
};

#[no_mangle]
pub static mut wszB4ISO: [WCHAR; 9] =
    unsafe { ::core::mem::transmute::<[u8; 18], [WCHAR; 9]>(*b"B\x004\0 \0(\0I\0S\0O\0)\0\0\0") };

#[no_mangle]
pub static mut wszJapanesePostcard: [WCHAR; 18] = unsafe {
    ::core::mem::transmute::<[u8; 36], [WCHAR; 18]>(
        *b"J\0a\0p\0a\0n\0e\0s\0e\0 \0P\0o\0s\0t\0c\0a\0r\0d\0\0\0",
    )
};

#[no_mangle]
pub static mut wsz9x11: [WCHAR; 7] =
    unsafe { ::core::mem::transmute::<[u8; 14], [WCHAR; 7]>(*b"9\0 \0x\0 \x001\x001\0\0\0") };

#[no_mangle]
pub static mut wsz10x11: [WCHAR; 8] =
    unsafe { ::core::mem::transmute::<[u8; 16], [WCHAR; 8]>(*b"1\x000\0 \0x\0 \x001\x001\0\0\0") };

#[no_mangle]
pub static mut wsz15x11: [WCHAR; 8] =
    unsafe { ::core::mem::transmute::<[u8; 16], [WCHAR; 8]>(*b"1\x005\0 \0x\0 \x001\x001\0\0\0") };

#[no_mangle]
pub static mut wszEnvelopeInvite: [WCHAR; 16] = unsafe {
    ::core::mem::transmute::<[u8; 32], [WCHAR; 16]>(
        *b"E\0n\0v\0e\0l\0o\0p\0e\0 \0I\0n\0v\0i\0t\0e\0\0\0",
    )
};

#[no_mangle]
pub static mut wszReserved48: [WCHAR; 11] = unsafe {
    ::core::mem::transmute::<[u8; 22], [WCHAR; 11]>(*b"R\0e\0s\0e\0r\0v\0e\0d\x004\08\0\0\0")
};

#[no_mangle]
pub static mut wszReserved49: [WCHAR; 11] = unsafe {
    ::core::mem::transmute::<[u8; 22], [WCHAR; 11]>(*b"R\0e\0s\0e\0r\0v\0e\0d\x004\09\0\0\0")
};

#[no_mangle]
pub static mut wszLetterExtra: [WCHAR; 13] = unsafe {
    ::core::mem::transmute::<[u8; 26], [WCHAR; 13]>(*b"L\0e\0t\0t\0e\0r\0 \0E\0x\0t\0r\0a\0\0\0")
};

#[no_mangle]
pub static mut wszLegalExtra: [WCHAR; 12] = unsafe {
    ::core::mem::transmute::<[u8; 24], [WCHAR; 12]>(*b"L\0e\0g\0a\0l\0 \0E\0x\0t\0r\0a\0\0\0")
};

#[no_mangle]
pub static mut wszTabloidExtra: [WCHAR; 14] = unsafe {
    ::core::mem::transmute::<[u8; 28], [WCHAR; 14]>(*b"T\0a\0b\0l\0o\0i\0d\0 \0E\0x\0t\0r\0a\0\0\0")
};

#[no_mangle]
pub static mut wszA4Extra: [WCHAR; 9] =
    unsafe { ::core::mem::transmute::<[u8; 18], [WCHAR; 9]>(*b"A\x004\0 \0E\0x\0t\0r\0a\0\0\0") };

#[no_mangle]
pub static mut wszLetterTransverse: [WCHAR; 18] = unsafe {
    ::core::mem::transmute::<[u8; 36], [WCHAR; 18]>(
        *b"L\0e\0t\0t\0e\0r\0 \0T\0r\0a\0n\0s\0v\0e\0r\0s\0e\0\0\0",
    )
};

#[no_mangle]
pub static mut wszA4Transverse: [WCHAR; 14] = unsafe {
    ::core::mem::transmute::<[u8; 28], [WCHAR; 14]>(
        *b"A\x004\0 \0T\0r\0a\0n\0s\0v\0e\0r\0s\0e\0\0\0",
    )
};

#[no_mangle]
pub static mut wszLetterExtraTransverse: [WCHAR; 24] = unsafe {
    ::core::mem::transmute::<[u8; 48], [WCHAR; 24]>(
        *b"L\0e\0t\0t\0e\0r\0 \0E\0x\0t\0r\0a\0 \0T\0r\0a\0n\0s\0v\0e\0r\0s\0e\0\0\0",
    )
};

#[no_mangle]
pub static mut wszSuperA: [WCHAR; 8] =
    unsafe { ::core::mem::transmute::<[u8; 16], [WCHAR; 8]>(*b"S\0u\0p\0e\0r\0 \0A\0\0\0") };

#[no_mangle]
pub static mut wszSuperB: [WCHAR; 8] =
    unsafe { ::core::mem::transmute::<[u8; 16], [WCHAR; 8]>(*b"S\0u\0p\0e\0r\0 \0B\0\0\0") };

#[no_mangle]
pub static mut wszLetterPlus: [WCHAR; 12] = unsafe {
    ::core::mem::transmute::<[u8; 24], [WCHAR; 12]>(*b"L\0e\0t\0t\0e\0r\0 \0P\0l\0u\0s\0\0\0")
};

#[no_mangle]
pub static mut wszA4Plus: [WCHAR; 8] =
    unsafe { ::core::mem::transmute::<[u8; 16], [WCHAR; 8]>(*b"A\x004\0 \0P\0l\0u\0s\0\0\0") };

#[no_mangle]
pub static mut wszA5Transverse: [WCHAR; 14] = unsafe {
    ::core::mem::transmute::<[u8; 28], [WCHAR; 14]>(
        *b"A\x005\0 \0T\0r\0a\0n\0s\0v\0e\0r\0s\0e\0\0\0",
    )
};

#[no_mangle]
pub static mut wszB5JISTransverse: [WCHAR; 20] = unsafe {
    ::core::mem::transmute::<[u8; 40], [WCHAR; 20]>(
        *b"B\x005\0 \0(\0J\0I\0S\0)\0 \0T\0r\0a\0n\0s\0v\0e\0r\0s\0e\0\0\0",
    )
};

#[no_mangle]
pub static mut wszA3Extra: [WCHAR; 9] =
    unsafe { ::core::mem::transmute::<[u8; 18], [WCHAR; 9]>(*b"A\x003\0 \0E\0x\0t\0r\0a\0\0\0") };

#[no_mangle]
pub static mut wszA5Extra: [WCHAR; 9] =
    unsafe { ::core::mem::transmute::<[u8; 18], [WCHAR; 9]>(*b"A\x005\0 \0E\0x\0t\0r\0a\0\0\0") };

#[no_mangle]
pub static mut wszB5ISOExtra: [WCHAR; 15] = unsafe {
    ::core::mem::transmute::<[u8; 30], [WCHAR; 15]>(
        *b"B\x005\0 \0(\0I\0S\0O\0)\0 \0E\0x\0t\0r\0a\0\0\0",
    )
};

#[no_mangle]
pub static mut wszA0: [WCHAR; 3] =
    unsafe { ::core::mem::transmute::<[u8; 6], [WCHAR; 3]>(*b"A\x000\0\0\0") };

#[no_mangle]
pub static mut wszA3Transverse: [WCHAR; 14] = unsafe {
    ::core::mem::transmute::<[u8; 28], [WCHAR; 14]>(
        *b"A\x003\0 \0T\0r\0a\0n\0s\0v\0e\0r\0s\0e\0\0\0",
    )
};

#[no_mangle]
pub static mut wszA3ExtraTransverse: [WCHAR; 20] = unsafe {
    ::core::mem::transmute::<[u8; 40], [WCHAR; 20]>(
        *b"A\x003\0 \0E\0x\0t\0r\0a\0 \0T\0r\0a\0n\0s\0v\0e\0r\0s\0e\0\0\0",
    )
};

#[no_mangle]
pub static mut wszJapaneseDoublePostcard: [WCHAR; 25] = unsafe {
    ::core::mem::transmute::<[u8; 50], [WCHAR; 25]>(
        *b"J\0a\0p\0a\0n\0e\0s\0e\0 \0D\0o\0u\0b\0l\0e\0 \0P\0o\0s\0t\0c\0a\0r\0d\0\0\0",
    )
};

#[no_mangle]
pub static mut wszA1: [WCHAR; 3] =
    unsafe { ::core::mem::transmute::<[u8; 6], [WCHAR; 3]>(*b"A\x001\0\0\0") };

#[no_mangle]
pub static mut wszJapaneseEnvelopeKaku2: [WCHAR; 26] = unsafe {
    ::core::mem::transmute::<[u8; 52], [WCHAR; 26]>(
        *b"J\0a\0p\0a\0n\0e\0s\0e\0 \0E\0n\0v\0e\0l\0o\0p\0e\0 \0K\0a\0k\0u\0 \0#\x002\0\0\0",
    )
};

#[no_mangle]
pub static mut wszJapaneseEnvelopeKaku3: [WCHAR; 26] = unsafe {
    ::core::mem::transmute::<[u8; 52], [WCHAR; 26]>(
        *b"J\0a\0p\0a\0n\0e\0s\0e\0 \0E\0n\0v\0e\0l\0o\0p\0e\0 \0K\0a\0k\0u\0 \0#\x003\0\0\0",
    )
};

#[no_mangle]
pub static mut wszJapaneseEnvelopeChou3: [WCHAR; 26] = unsafe {
    ::core::mem::transmute::<[u8; 52], [WCHAR; 26]>(
        *b"J\0a\0p\0a\0n\0e\0s\0e\0 \0E\0n\0v\0e\0l\0o\0p\0e\0 \0C\0h\0o\0u\0 \0#\x003\0\0\0",
    )
};

#[no_mangle]
pub static mut wszJapaneseEnvelopeChou4: [WCHAR; 26] = unsafe {
    ::core::mem::transmute::<[u8; 52], [WCHAR; 26]>(
        *b"J\0a\0p\0a\0n\0e\0s\0e\0 \0E\0n\0v\0e\0l\0o\0p\0e\0 \0C\0h\0o\0u\0 \0#\x004\0\0\0",
    )
};

#[no_mangle]
pub static mut wszLetterRotated: [WCHAR; 15] = unsafe {
    ::core::mem::transmute::<[u8; 30], [WCHAR; 15]>(
        *b"L\0e\0t\0t\0e\0r\0 \0R\0o\0t\0a\0t\0e\0d\0\0\0",
    )
};

#[no_mangle]
pub static mut wszA3Rotated: [WCHAR; 11] = unsafe {
    ::core::mem::transmute::<[u8; 22], [WCHAR; 11]>(*b"A\x003\0 \0R\0o\0t\0a\0t\0e\0d\0\0\0")
};

#[no_mangle]
pub static mut wszA4Rotated: [WCHAR; 11] = unsafe {
    ::core::mem::transmute::<[u8; 22], [WCHAR; 11]>(*b"A\x004\0 \0R\0o\0t\0a\0t\0e\0d\0\0\0")
};

#[no_mangle]
pub static mut wszA5Rotated: [WCHAR; 11] = unsafe {
    ::core::mem::transmute::<[u8; 22], [WCHAR; 11]>(*b"A\x005\0 \0R\0o\0t\0a\0t\0e\0d\0\0\0")
};

#[no_mangle]
pub static mut wszB4JISRotated: [WCHAR; 17] = unsafe {
    ::core::mem::transmute::<[u8; 34], [WCHAR; 17]>(
        *b"B\x004\0 \0(\0J\0I\0S\0)\0 \0R\0o\0t\0a\0t\0e\0d\0\0\0",
    )
};

#[no_mangle]
pub static mut wszB5JISRotated: [WCHAR; 17] = unsafe {
    ::core::mem::transmute::<[u8; 34], [WCHAR; 17]>(
        *b"B\x005\0 \0(\0J\0I\0S\0)\0 \0R\0o\0t\0a\0t\0e\0d\0\0\0",
    )
};

#[no_mangle]
pub static mut wszJapanesePostcardRotated: [WCHAR; 26] = unsafe {
    ::core::mem::transmute::<[u8; 52], [WCHAR; 26]>(
        *b"J\0a\0p\0a\0n\0e\0s\0e\0 \0P\0o\0s\0t\0c\0a\0r\0d\0 \0R\0o\0t\0a\0t\0e\0d\0\0\0",
    )
};

#[no_mangle]
pub static mut wszDoubleJapanPostcardRotated: [WCHAR; 30] = unsafe {
    ::core::mem::transmute::<
        [u8; 60],
        [WCHAR; 30],
    >(
        *b"D\0o\0u\0b\0l\0e\0 \0J\0a\0p\0a\0n\0 \0P\0o\0s\0t\0c\0a\0r\0d\0 \0R\0o\0t\0a\0t\0e\0d\0\0\0",
    )
};

#[no_mangle]
pub static mut wsA6Rotatedz: [WCHAR; 11] = unsafe {
    ::core::mem::transmute::<[u8; 22], [WCHAR; 11]>(*b"A\x006\0 \0R\0o\0t\0a\0t\0e\0d\0\0\0")
};

#[no_mangle]
pub static mut wszJapanEnvelopeKaku2Rotated: [WCHAR; 31] = unsafe {
    ::core::mem::transmute::<
        [u8; 62],
        [WCHAR; 31],
    >(
        *b"J\0a\0p\0a\0n\0 \0E\0n\0v\0e\0l\0o\0p\0e\0 \0K\0a\0k\0u\0 \0#\x002\0 \0R\0o\0t\0a\0t\0e\0d\0\0\0",
    )
};

#[no_mangle]
pub static mut wszJapanEnvelopeKaku3Rotated: [WCHAR; 31] = unsafe {
    ::core::mem::transmute::<
        [u8; 62],
        [WCHAR; 31],
    >(
        *b"J\0a\0p\0a\0n\0 \0E\0n\0v\0e\0l\0o\0p\0e\0 \0K\0a\0k\0u\0 \0#\x003\0 \0R\0o\0t\0a\0t\0e\0d\0\0\0",
    )
};

#[no_mangle]
pub static mut wszJapanEnvelopeChou3Rotated: [WCHAR; 31] = unsafe {
    ::core::mem::transmute::<
        [u8; 62],
        [WCHAR; 31],
    >(
        *b"J\0a\0p\0a\0n\0 \0E\0n\0v\0e\0l\0o\0p\0e\0 \0C\0h\0o\0u\0 \0#\x003\0 \0R\0o\0t\0a\0t\0e\0d\0\0\0",
    )
};

#[no_mangle]
pub static mut wszJapanEnvelopeChou4Rotated: [WCHAR; 31] = unsafe {
    ::core::mem::transmute::<
        [u8; 62],
        [WCHAR; 31],
    >(
        *b"J\0a\0p\0a\0n\0 \0E\0n\0v\0e\0l\0o\0p\0e\0 \0C\0h\0o\0u\0 \0#\x004\0 \0R\0o\0t\0a\0t\0e\0d\0\0\0",
    )
};

#[no_mangle]
pub static mut wszB6JIS: [WCHAR; 9] =
    unsafe { ::core::mem::transmute::<[u8; 18], [WCHAR; 9]>(*b"B\x006\0 \0(\0J\0I\0S\0)\0\0\0") };

#[no_mangle]
pub static mut wszB6JISRotated: [WCHAR; 17] = unsafe {
    ::core::mem::transmute::<[u8; 34], [WCHAR; 17]>(
        *b"B\x006\0 \0(\0J\0I\0S\0)\0 \0R\0o\0t\0a\0t\0e\0d\0\0\0",
    )
};

#[no_mangle]
pub static mut wsz12x11: [WCHAR; 8] =
    unsafe { ::core::mem::transmute::<[u8; 16], [WCHAR; 8]>(*b"1\x002\0 \0x\0 \x001\x001\0\0\0") };

#[no_mangle]
pub static mut wszJapanEnvelopeYou4: [WCHAR; 22] = unsafe {
    ::core::mem::transmute::<[u8; 44], [WCHAR; 22]>(
        *b"J\0a\0p\0a\0n\0 \0E\0n\0v\0e\0l\0o\0p\0e\0 \0Y\0o\0u\0 \0#\x004\0\0\0",
    )
};

#[no_mangle]
pub static mut wszJapanEnvelopeYou4Rotated: [WCHAR; 30] = unsafe {
    ::core::mem::transmute::<
        [u8; 60],
        [WCHAR; 30],
    >(
        *b"J\0a\0p\0a\0n\0 \0E\0n\0v\0e\0l\0o\0p\0e\0 \0Y\0o\0u\0 \0#\x004\0 \0R\0o\0t\0a\0t\0e\0d\0\0\0",
    )
};

#[no_mangle]
pub static mut wszPRC16K: [WCHAR; 8] =
    unsafe { ::core::mem::transmute::<[u8; 16], [WCHAR; 8]>(*b"P\0R\0C\0 \x001\x006\0K\0\0\0") };

#[no_mangle]
pub static mut wszPRC32K: [WCHAR; 8] =
    unsafe { ::core::mem::transmute::<[u8; 16], [WCHAR; 8]>(*b"P\0R\0C\0 \x003\x002\0K\0\0\0") };

#[no_mangle]
pub static mut wszPRC32KBig: [WCHAR; 13] = unsafe {
    ::core::mem::transmute::<[u8; 26], [WCHAR; 13]>(
        *b"P\0R\0C\0 \x003\x002\0K\0(\0B\0i\0g\0)\0\0\0",
    )
};

#[no_mangle]
pub static mut wszPRCEnvelope1: [WCHAR; 16] = unsafe {
    ::core::mem::transmute::<[u8; 32], [WCHAR; 16]>(
        *b"P\0R\0C\0 \0E\0n\0v\0e\0l\0o\0p\0e\0 \0#\x001\0\0\0",
    )
};

#[no_mangle]
pub static mut wszPRCEnvelope2: [WCHAR; 16] = unsafe {
    ::core::mem::transmute::<[u8; 32], [WCHAR; 16]>(
        *b"P\0R\0C\0 \0E\0n\0v\0e\0l\0o\0p\0e\0 \0#\x002\0\0\0",
    )
};

#[no_mangle]
pub static mut wszPRCEnvelope3: [WCHAR; 16] = unsafe {
    ::core::mem::transmute::<[u8; 32], [WCHAR; 16]>(
        *b"P\0R\0C\0 \0E\0n\0v\0e\0l\0o\0p\0e\0 \0#\x003\0\0\0",
    )
};

#[no_mangle]
pub static mut wszPRCEnvelope4: [WCHAR; 16] = unsafe {
    ::core::mem::transmute::<[u8; 32], [WCHAR; 16]>(
        *b"P\0R\0C\0 \0E\0n\0v\0e\0l\0o\0p\0e\0 \0#\x004\0\0\0",
    )
};

#[no_mangle]
pub static mut wszPRCEnvelope5: [WCHAR; 16] = unsafe {
    ::core::mem::transmute::<[u8; 32], [WCHAR; 16]>(
        *b"P\0R\0C\0 \0E\0n\0v\0e\0l\0o\0p\0e\0 \0#\x005\0\0\0",
    )
};

#[no_mangle]
pub static mut wszPRCEnvelope6: [WCHAR; 16] = unsafe {
    ::core::mem::transmute::<[u8; 32], [WCHAR; 16]>(
        *b"P\0R\0C\0 \0E\0n\0v\0e\0l\0o\0p\0e\0 \0#\x006\0\0\0",
    )
};

#[no_mangle]
pub static mut wszPRCEnvelope7: [WCHAR; 16] = unsafe {
    ::core::mem::transmute::<[u8; 32], [WCHAR; 16]>(
        *b"P\0R\0C\0 \0E\0n\0v\0e\0l\0o\0p\0e\0 \0#\x007\0\0\0",
    )
};

#[no_mangle]
pub static mut wszPRCEnvelope8: [WCHAR; 16] = unsafe {
    ::core::mem::transmute::<[u8; 32], [WCHAR; 16]>(
        *b"P\0R\0C\0 \0E\0n\0v\0e\0l\0o\0p\0e\0 \0#\08\0\0\0",
    )
};

#[no_mangle]
pub static mut wszPRCEnvelope9: [WCHAR; 16] = unsafe {
    ::core::mem::transmute::<[u8; 32], [WCHAR; 16]>(
        *b"P\0R\0C\0 \0E\0n\0v\0e\0l\0o\0p\0e\0 \0#\09\0\0\0",
    )
};

#[no_mangle]
pub static mut wszPRCEnvelope10: [WCHAR; 17] = unsafe {
    ::core::mem::transmute::<[u8; 34], [WCHAR; 17]>(
        *b"P\0R\0C\0 \0E\0n\0v\0e\0l\0o\0p\0e\0 \0#\x001\x000\0\0\0",
    )
};

#[no_mangle]
pub static mut wszPRC16KRotated: [WCHAR; 16] = unsafe {
    ::core::mem::transmute::<[u8; 32], [WCHAR; 16]>(
        *b"P\0R\0C\0 \x001\x006\0K\0 \0R\0o\0t\0a\0t\0e\0d\0\0\0",
    )
};

#[no_mangle]
pub static mut wszPRC32KRotated: [WCHAR; 16] = unsafe {
    ::core::mem::transmute::<[u8; 32], [WCHAR; 16]>(
        *b"P\0R\0C\0 \x003\x002\0K\0 \0R\0o\0t\0a\0t\0e\0d\0\0\0",
    )
};

#[no_mangle]
pub static mut wszPRC32KBigRotated: [WCHAR; 21] = unsafe {
    ::core::mem::transmute::<[u8; 42], [WCHAR; 21]>(
        *b"P\0R\0C\0 \x003\x002\0K\0(\0B\0i\0g\0)\0 \0R\0o\0t\0a\0t\0e\0d\0\0\0",
    )
};

#[no_mangle]
pub static mut wszPRCEnvelope1Rotated: [WCHAR; 24] = unsafe {
    ::core::mem::transmute::<[u8; 48], [WCHAR; 24]>(
        *b"P\0R\0C\0 \0E\0n\0v\0e\0l\0o\0p\0e\0 \0#\x001\0 \0R\0o\0t\0a\0t\0e\0d\0\0\0",
    )
};

#[no_mangle]
pub static mut wszPRCEnvelope2Rotated: [WCHAR; 24] = unsafe {
    ::core::mem::transmute::<[u8; 48], [WCHAR; 24]>(
        *b"P\0R\0C\0 \0E\0n\0v\0e\0l\0o\0p\0e\0 \0#\x002\0 \0R\0o\0t\0a\0t\0e\0d\0\0\0",
    )
};

#[no_mangle]
pub static mut wszPRCEnvelope3Rotated: [WCHAR; 24] = unsafe {
    ::core::mem::transmute::<[u8; 48], [WCHAR; 24]>(
        *b"P\0R\0C\0 \0E\0n\0v\0e\0l\0o\0p\0e\0 \0#\x003\0 \0R\0o\0t\0a\0t\0e\0d\0\0\0",
    )
};

#[no_mangle]
pub static mut wszPRCEnvelope4Rotated: [WCHAR; 24] = unsafe {
    ::core::mem::transmute::<[u8; 48], [WCHAR; 24]>(
        *b"P\0R\0C\0 \0E\0n\0v\0e\0l\0o\0p\0e\0 \0#\x004\0 \0R\0o\0t\0a\0t\0e\0d\0\0\0",
    )
};

#[no_mangle]
pub static mut wszPRCEnvelope5Rotated: [WCHAR; 24] = unsafe {
    ::core::mem::transmute::<[u8; 48], [WCHAR; 24]>(
        *b"P\0R\0C\0 \0E\0n\0v\0e\0l\0o\0p\0e\0 \0#\x005\0 \0R\0o\0t\0a\0t\0e\0d\0\0\0",
    )
};

#[no_mangle]
pub static mut wszPRCEnvelope6Rotated: [WCHAR; 24] = unsafe {
    ::core::mem::transmute::<[u8; 48], [WCHAR; 24]>(
        *b"P\0R\0C\0 \0E\0n\0v\0e\0l\0o\0p\0e\0 \0#\x006\0 \0R\0o\0t\0a\0t\0e\0d\0\0\0",
    )
};

#[no_mangle]
pub static mut wszPRCEnvelope7Rotated: [WCHAR; 24] = unsafe {
    ::core::mem::transmute::<[u8; 48], [WCHAR; 24]>(
        *b"P\0R\0C\0 \0E\0n\0v\0e\0l\0o\0p\0e\0 \0#\x007\0 \0R\0o\0t\0a\0t\0e\0d\0\0\0",
    )
};

#[no_mangle]
pub static mut wszPRCEnvelope8Rotated: [WCHAR; 24] = unsafe {
    ::core::mem::transmute::<[u8; 48], [WCHAR; 24]>(
        *b"P\0R\0C\0 \0E\0n\0v\0e\0l\0o\0p\0e\0 \0#\08\0 \0R\0o\0t\0a\0t\0e\0d\0\0\0",
    )
};

#[no_mangle]
pub static mut wszPRCEnvelope9Rotated: [WCHAR; 24] = unsafe {
    ::core::mem::transmute::<[u8; 48], [WCHAR; 24]>(
        *b"P\0R\0C\0 \0E\0n\0v\0e\0l\0o\0p\0e\0 \0#\09\0 \0R\0o\0t\0a\0t\0e\0d\0\0\0",
    )
};

#[no_mangle]
pub static mut wszPRCEnvelope10Rotated: [WCHAR; 25] = unsafe {
    ::core::mem::transmute::<[u8; 50], [WCHAR; 25]>(
        *b"P\0R\0C\0 \0E\0n\0v\0e\0l\0o\0p\0e\0 \0#\x001\x000\0 \0R\0o\0t\0a\0t\0e\0d\0\0\0",
    )
};

#[no_mangle]
pub static mut BuiltInForms: [FORM_INFO_1W; 44] = unsafe {
    [
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszLetter as LPWSTR,
            Size: tagSIZE {
                cx: 215900 as LONG,
                cy: 279400 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 215900 as LONG,
                bottom: 279400 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszLetterSmall as LPWSTR,
            Size: tagSIZE {
                cx: 215900 as LONG,
                cy: 279400 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 215900 as LONG,
                bottom: 279400 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszTabloid as LPWSTR,
            Size: tagSIZE {
                cx: 279400 as LONG,
                cy: 431800 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 279400 as LONG,
                bottom: 431800 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszLedger as LPWSTR,
            Size: tagSIZE {
                cx: 431800 as LONG,
                cy: 279400 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 431800 as LONG,
                bottom: 279400 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszLegal as LPWSTR,
            Size: tagSIZE {
                cx: 215900 as LONG,
                cy: 355600 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 215900 as LONG,
                bottom: 355600 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszStatement as LPWSTR,
            Size: tagSIZE {
                cx: 139700 as LONG,
                cy: 215900 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 139700 as LONG,
                bottom: 215900 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszExecutive as LPWSTR,
            Size: tagSIZE {
                cx: 184150 as LONG,
                cy: 266700 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 184150 as LONG,
                bottom: 266700 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszA3 as LPWSTR,
            Size: tagSIZE {
                cx: 297000 as LONG,
                cy: 420000 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 297000 as LONG,
                bottom: 420000 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszA4 as LPWSTR,
            Size: tagSIZE {
                cx: 210000 as LONG,
                cy: 297000 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 210000 as LONG,
                bottom: 297000 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszA4Small as LPWSTR,
            Size: tagSIZE {
                cx: 210000 as LONG,
                cy: 297000 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 210000 as LONG,
                bottom: 297000 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszA5 as LPWSTR,
            Size: tagSIZE {
                cx: 148000 as LONG,
                cy: 210000 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 148000 as LONG,
                bottom: 210000 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszB4JIS as LPWSTR,
            Size: tagSIZE {
                cx: 257000 as LONG,
                cy: 364000 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 257000 as LONG,
                bottom: 364000 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszB5JIS as LPWSTR,
            Size: tagSIZE {
                cx: 182000 as LONG,
                cy: 257000 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 182000 as LONG,
                bottom: 257000 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszFolio as LPWSTR,
            Size: tagSIZE {
                cx: 215900 as LONG,
                cy: 330200 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 215900 as LONG,
                bottom: 330200 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszQuarto as LPWSTR,
            Size: tagSIZE {
                cx: 215000 as LONG,
                cy: 275000 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 215000 as LONG,
                bottom: 275000 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wsz10x14 as LPWSTR,
            Size: tagSIZE {
                cx: 254000 as LONG,
                cy: 355600 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 254000 as LONG,
                bottom: 355600 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wsz11x17 as LPWSTR,
            Size: tagSIZE {
                cx: 279400 as LONG,
                cy: 431800 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 279400 as LONG,
                bottom: 431800 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszNote as LPWSTR,
            Size: tagSIZE {
                cx: 215900 as LONG,
                cy: 279400 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 215900 as LONG,
                bottom: 279400 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszEnvelope9 as LPWSTR,
            Size: tagSIZE {
                cx: 98425 as LONG,
                cy: 225425 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 98425 as LONG,
                bottom: 225425 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszEnvelope10 as LPWSTR,
            Size: tagSIZE {
                cx: 104775 as LONG,
                cy: 241300 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 104775 as LONG,
                bottom: 241300 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszEnvelope11 as LPWSTR,
            Size: tagSIZE {
                cx: 114300 as LONG,
                cy: 263525 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 114300 as LONG,
                bottom: 263525 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszEnvelope12 as LPWSTR,
            Size: tagSIZE {
                cx: 120650 as LONG,
                cy: 279400 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 120650 as LONG,
                bottom: 279400 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszEnvelope14 as LPWSTR,
            Size: tagSIZE {
                cx: 127000 as LONG,
                cy: 292100 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 127000 as LONG,
                bottom: 292100 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszCsizesheet as LPWSTR,
            Size: tagSIZE {
                cx: 431800 as LONG,
                cy: 558800 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 431800 as LONG,
                bottom: 558800 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszDsizesheet as LPWSTR,
            Size: tagSIZE {
                cx: 558800 as LONG,
                cy: 863600 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 558800 as LONG,
                bottom: 863600 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszEsizesheet as LPWSTR,
            Size: tagSIZE {
                cx: 863600 as LONG,
                cy: 1117600 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 863600 as LONG,
                bottom: 1117600 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszEnvelopeDL as LPWSTR,
            Size: tagSIZE {
                cx: 110000 as LONG,
                cy: 220000 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 110000 as LONG,
                bottom: 220000 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszEnvelopeC5 as LPWSTR,
            Size: tagSIZE {
                cx: 162000 as LONG,
                cy: 229000 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 162000 as LONG,
                bottom: 229000 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszEnvelopeC3 as LPWSTR,
            Size: tagSIZE {
                cx: 324000 as LONG,
                cy: 458000 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 324000 as LONG,
                bottom: 458000 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszEnvelopeC4 as LPWSTR,
            Size: tagSIZE {
                cx: 229000 as LONG,
                cy: 324000 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 229000 as LONG,
                bottom: 324000 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszEnvelopeC6 as LPWSTR,
            Size: tagSIZE {
                cx: 114000 as LONG,
                cy: 162000 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 114000 as LONG,
                bottom: 162000 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszEnvelope65 as LPWSTR,
            Size: tagSIZE {
                cx: 114000 as LONG,
                cy: 229000 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 114000 as LONG,
                bottom: 229000 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszEnvelopeB4 as LPWSTR,
            Size: tagSIZE {
                cx: 250000 as LONG,
                cy: 353000 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 250000 as LONG,
                bottom: 353000 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszEnvelopeB5 as LPWSTR,
            Size: tagSIZE {
                cx: 176000 as LONG,
                cy: 250000 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 176000 as LONG,
                bottom: 250000 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszEnvelopeB6 as LPWSTR,
            Size: tagSIZE {
                cx: 176000 as LONG,
                cy: 125000 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 176000 as LONG,
                bottom: 125000 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszEnvelope as LPWSTR,
            Size: tagSIZE {
                cx: 110000 as LONG,
                cy: 230000 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 110000 as LONG,
                bottom: 230000 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszEnvelopeMonarch as LPWSTR,
            Size: tagSIZE {
                cx: 98425 as LONG,
                cy: 190500 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 98425 as LONG,
                bottom: 190500 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wsz634Envelope as LPWSTR,
            Size: tagSIZE {
                cx: 92075 as LONG,
                cy: 165100 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 92075 as LONG,
                bottom: 165100 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszUSStdFanfold as LPWSTR,
            Size: tagSIZE {
                cx: 377825 as LONG,
                cy: 279400 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 377825 as LONG,
                bottom: 279400 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszGermanStdFanfold as LPWSTR,
            Size: tagSIZE {
                cx: 215900 as LONG,
                cy: 304800 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 215900 as LONG,
                bottom: 304800 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszGermanLegalFanfold as LPWSTR,
            Size: tagSIZE {
                cx: 215900 as LONG,
                cy: 330200 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 215900 as LONG,
                bottom: 330200 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszDoubleJapanPostcardRotated as LPWSTR,
            Size: tagSIZE {
                cx: 148000 as LONG,
                cy: 200000 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 148000 as LONG,
                bottom: 200000 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: &raw const wszPRCEnvelope10Rotated as LPWSTR,
            Size: tagSIZE {
                cx: 458000 as LONG,
                cy: 324009 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 458000 as LONG,
                bottom: 324009 as LONG,
            },
        },
        _FORM_INFO_1W {
            Flags: FORM_USER as DWORD,
            pName: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            Size: tagSIZE {
                cx: 0 as LONG,
                cy: 0 as LONG,
            },
            ImageableArea: _RECTL {
                left: 0 as LONG,
                top: 0 as LONG,
                right: 0 as LONG,
                bottom: 0 as LONG,
            },
        },
    ]
};

#[no_mangle]
pub static mut hFormCKey: HKEY = ::core::ptr::null::<HKEY__>() as *mut HKEY__;

#[no_mangle]
pub static mut hFormsKey: HKEY = ::core::ptr::null::<HKEY__>() as *mut HKEY__;

static mut pfnGetFormLevels: [PLocalGetFormLevelFunc; 3] = unsafe {
    [
        None,
        ::core::mem::transmute::<
            Option<
                unsafe extern "C" fn(PFORM_INFO_LIST, *mut PFORM_INFO_1W, *mut PBYTE, PDWORD) -> (),
            >,
            PLocalGetFormLevelFunc,
        >(Some(
            _LocalGetFormLevel1
                as unsafe extern "C" fn(
                    PFORM_INFO_LIST,
                    *mut PFORM_INFO_1W,
                    *mut PBYTE,
                    PDWORD,
                ) -> (),
        )),
        ::core::mem::transmute::<
            Option<
                unsafe extern "C" fn(PFORM_INFO_LIST, *mut PFORM_INFO_2W, *mut PBYTE, PDWORD) -> (),
            >,
            PLocalGetFormLevelFunc,
        >(Some(
            _LocalGetFormLevel2
                as unsafe extern "C" fn(
                    PFORM_INFO_LIST,
                    *mut PFORM_INFO_2W,
                    *mut PBYTE,
                    PDWORD,
                ) -> (),
        )),
    ]
};

pub const FORMINFOSIG: DWORD = 0x2E32; // '.2' multi-char constant ('.' << 8 | '2')

pub const FORMMAXNAMESIZE: DWORD = 32 - 1; // CCHDEVICENAME-1


// FORM_INFO_2W is an anonymous member in C; its fields are flattened here.
#[repr(C)]
pub struct _FORM_INFO_LIST {
    pub List: LIST_ENTRY,
    pub Sig: DWORD,
    pub Index: DWORD,
    pub Flags: DWORD,
    pub pName: PWSTR,
    pub Size: SIZEL,
    pub ImageableArea: RECTL,
    pub pKeyword: PWSTR,
    pub StringType: DWORD,
    pub pMuiDll: PWSTR,
    pub ResourceId: DWORD,
    pub pDisplayName: PWSTR,
    pub wLangId: WORD,
}
pub type FORM_INFO_LIST = _FORM_INFO_LIST;
pub type PFORM_INFO_LIST = *mut FORM_INFO_LIST;

#[repr(C)]
pub struct _REGISTRYFORMINFO {
    pub Size: SIZEL,
    pub ImageableArea: RECTL,
    pub Index: DWORD,
    pub Flags: DWORD,
}
pub type REGISTRYFORMINFO = _REGISTRYFORMINFO;
pub type PREGISTRYFORMINFO = *mut REGISTRYFORMINFO;

pub type PLocalGetFormLevelFunc =
    Option<unsafe extern "C" fn(PFORM_INFO_LIST, PVOID, *mut PBYTE, PDWORD)>;
#[no_mangle]
pub unsafe extern "C" fn InitializeFormList() -> BOOL {
    let mut dwErrorCode: DWORD = 0;
    let mut pfil: PFORM_INFO_LIST = core::ptr::null_mut();
    let mut rfi: REGISTRYFORMINFO = core::mem::zeroed();

    'Cleanup: {
        dwErrorCode = RegCreateKeyExW(
            HKEY_LOCAL_MACHINE,
            RUSTCONV_WSTR_98C6C31426EF.as_ptr(),
            0,
            core::ptr::null_mut(),
            REG_OPTION_VOLATILE,
            KEY_ALL_ACCESS,
            core::ptr::null_mut(),
            &raw mut hFormCKey,
            core::ptr::null_mut(),
        ) as DWORD;

        if dwErrorCode != ERROR_SUCCESS && dwErrorCode != ERROR_ALREADY_EXISTS {
            break 'Cleanup;
        }

        // Open some registry keys and leave them open. We need them multiple times throughout the Local Spooler.
        dwErrorCode = RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            RUSTCONV_WSTR_98C6C31426EF.as_ptr(),
            0,
            KEY_ALL_ACCESS,
            &raw mut hFormsKey,
        ) as DWORD;
        if dwErrorCode != ERROR_SUCCESS {
            break 'Cleanup;
        }

        _dwLastForm = 1;

        InitializeListHead(&raw mut FormList);

        {
            let mut i: i32 = 0;
            let mut Size: i32;
            while !BuiltInForms[i as usize].pName.is_null() {
                Size = (core::mem::size_of::<FORM_INFO_LIST>() as i32)
                    + ((wcslen(BuiltInForms[i as usize].pName as *const u16) as i32
                        + 1)
                        * core::mem::size_of::<WCHAR>() as i32);

                pfil = DllAllocSplMem(Size as DWORD) as PFORM_INFO_LIST;

                (*pfil).pName = wcscpy(
                    pfil.add(1 as usize) as PWSTR,
                    BuiltInForms[i as usize].pName,
                ) as PWSTR;
                (*pfil).Flags = BuiltInForms[i as usize].Flags;
                (*pfil).Size = BuiltInForms[i as usize].Size;
                (*pfil).ImageableArea = BuiltInForms[i as usize].ImageableArea;
                (*pfil).Sig = FORMINFOSIG;
                let fresh = _dwLastForm;
                _dwLastForm = _dwLastForm.wrapping_add(1);
                (*pfil).Index = fresh;
                (*pfil).pKeyword = core::ptr::null_mut();
                (*pfil).StringType = STRING_NONE;
                (*pfil).pMuiDll = core::ptr::null_mut();
                (*pfil).ResourceId = 0;
                (*pfil).pDisplayName = core::ptr::null_mut();
                (*pfil).wLangId = 0;

                InsertTailList(&raw mut FormList, &raw mut (*pfil).List);

                rfi.Size = (*pfil).Size;
                rfi.ImageableArea = (*pfil).ImageableArea;
                rfi.Index = (*pfil).Index;
                rfi.Flags = (*pfil).Flags;

                dwErrorCode = RegSetValueExW(
                    hFormsKey,
                    (*pfil).pName as LPCWSTR,
                    0,
                    REG_BINARY,
                    &raw mut rfi as PBYTE,
                    core::mem::size_of::<REGISTRYFORMINFO>() as DWORD,
                ) as DWORD;
                if dwErrorCode == ERROR_SUCCESS {
                }

                i += 1;
            }
        }
    }

    return TRUE as BOOL;
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn FindForm(pFormName: *mut WCHAR, pKeyword: *mut WCHAR) -> PFORM_INFO_LIST {
    let mut ListEntry: PLIST_ENTRY;
    let mut pfil: PFORM_INFO_LIST;

    ListEntry = (*core::ptr::addr_of_mut!(FormList)).Flink;

    while ListEntry != core::ptr::addr_of_mut!(FormList) {
        pfil = CONTAINING_RECORD!(ListEntry, FORM_INFO_LIST, List);

        ListEntry = (*ListEntry).Flink;

        if !pFormName.is_null() && _wcsicmp((*pfil).pName, pFormName) == 0 {
            return pfil;
        }

        if !pKeyword.is_null() && _wcsicmp((*pfil).pKeyword as *mut WCHAR, pKeyword) == 0 {
            return pfil;
        }
    }
    core::ptr::null_mut()
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn FindForm(pFormName: *mut WCHAR, pKeyword: *mut WCHAR) -> PFORM_INFO_LIST {
    let mut ListEntry: PLIST_ENTRY;
    let mut pfil: PFORM_INFO_LIST;

    ListEntry = (*core::ptr::addr_of_mut!(FormList)).Flink;

    while ListEntry != core::ptr::addr_of_mut!(FormList) {
        pfil = CONTAINING_RECORD!(ListEntry, FORM_INFO_LIST, List);

        ListEntry = (*ListEntry).Flink;

        if !pFormName.is_null() && _wcsicmp((*pfil).pName, pFormName) == 0 {
            return pfil;
        }

        if !pKeyword.is_null() && _wcsicmp((*pfil).pKeyword as *mut WCHAR, pKeyword) == 0 {
            return pfil;
        }
    }
    core::ptr::null_mut()
}

unsafe extern "C" fn _LocalGetFormLevel1(pfil: PFORM_INFO_LIST, ppFormInfo: *mut PFORM_INFO_1W, ppFormInfoEnd: *mut PBYTE, pcbNeeded: PDWORD) {
    let mut n: DWORD;
    let mut pwszStrings: [PCWSTR; 1] = [core::ptr::null(); 1];

    pwszStrings[0 as usize] = (*pfil).pName;

    // Calculate the string lengths.
    if ppFormInfo.is_null() {
        n = 0 as DWORD;
        while n < _countof!(pwszStrings) {
            *pcbNeeded = (*pcbNeeded).wrapping_add(
                ((wcslen(pwszStrings[n as usize]) as DWORD)
                    .wrapping_add(1 as DWORD))
                    .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD),
            );
            n = n.wrapping_add(1);
        }

        *pcbNeeded = (*pcbNeeded).wrapping_add(core::mem::size_of::<FORM_INFO_1W>() as DWORD);
        return;
    }

    (**ppFormInfo).Flags = (*pfil).Flags;
    (**ppFormInfo).Size = (*pfil).Size;
    (**ppFormInfo).ImageableArea = (*pfil).ImageableArea;

    // Finally copy the structure and advance to the next one in the output buffer.
    *ppFormInfoEnd = PackStrings(
        core::ptr::addr_of_mut!(pwszStrings) as *mut PCWSTR,
        *ppFormInfo as PBYTE,
        core::ptr::addr_of_mut!(dwFormInfo1Offsets) as *mut DWORD,
        *ppFormInfoEnd,
    );
    *ppFormInfo = (*ppFormInfo).offset(1);
}

unsafe extern "C" fn _LocalGetFormLevel2(pfil: PFORM_INFO_LIST, ppFormInfo: *mut PFORM_INFO_2W, ppFormInfoEnd: *mut PBYTE, pcbNeeded: PDWORD) {
    let mut n: DWORD;
    let mut pwszStrings: [PCWSTR; 4] = [core::ptr::null(); 4];

    pwszStrings[0] = (*pfil).pName as PCWSTR;
    pwszStrings[1] = (*pfil).pKeyword as PCWSTR;
    pwszStrings[2] = (*pfil).pMuiDll as PCWSTR;
    pwszStrings[3] = (*pfil).pDisplayName as PCWSTR;

    // Calculate the string lengths.
    if ppFormInfo.is_null() {
        n = 0;
        while (n as usize) < (_countof!(pwszStrings) as usize) {
            if !pwszStrings[n as usize].is_null() {
                *pcbNeeded += ((wcslen(pwszStrings[n as usize]) as DWORD)
                    + 1)
                    * core::mem::size_of::<WCHAR>() as DWORD;
            }
            n += 1;
        }
        *pcbNeeded += core::mem::size_of::<FORM_INFO_2W>() as DWORD;
        return;
    }

    (**ppFormInfo).Flags = (*pfil).Flags;
    (**ppFormInfo).Size = (*pfil).Size;
    (**ppFormInfo).ImageableArea = (*pfil).ImageableArea;
    (**ppFormInfo).StringType = (*pfil).StringType; //// If caller is remote, set STRING_LANGPAIR always;
    (**ppFormInfo).dwResourceId = (*pfil).ResourceId;

    // Finally copy the structure and advance to the next one in the output buffer.
    *ppFormInfoEnd = PackStrings(
        core::ptr::addr_of_mut!(pwszStrings) as *mut PCWSTR,
        *ppFormInfo as PBYTE,
        core::ptr::addr_of_mut!(dwFormInfo2Offsets) as *mut DWORD,
        *ppFormInfoEnd,
    );
    *ppFormInfo = (*ppFormInfo).add(1);
}
#[no_mangle]
pub unsafe extern "system" fn LocalAddForm(hPrinter: HANDLE, Level: DWORD, pForm: PBYTE) -> BOOL {
    let mut dwErrorCode: DWORD = 0;
    let mut Size: DWORD = 0;
    let mut cbNeeded: DWORD = 0;
    let mut pfil: PFORM_INFO_LIST = core::ptr::null_mut();
    let mut pHandle: PLOCAL_HANDLE = core::ptr::null_mut();
    let mut rfi: REGISTRYFORMINFO = core::mem::zeroed();
    let pfi1w: PFORM_INFO_1W = pForm as PFORM_INFO_1W;
    let pfi2w: PFORM_INFO_2W = pForm as PFORM_INFO_2W;

    // Check if this is a printer handle.
    pHandle = hPrinter as PLOCAL_HANDLE;
    'Cleanup: {
        if (*pHandle).HandleType as DWORD != HandleType_Printer as DWORD {
            dwErrorCode = ERROR_INVALID_HANDLE as DWORD;
            break 'Cleanup;
        }

        // Only support 1 & 2
        if Level < 1 as DWORD || Level > 2 as DWORD {
            // The caller supplied an invalid level.
            dwErrorCode = ERROR_INVALID_LEVEL as DWORD;
            break 'Cleanup;
        }

        pfil = FindForm((*pfi1w).pName as *mut WCHAR, core::ptr::null_mut());
        if !pfil.is_null() {
            dwErrorCode = ERROR_FILE_EXISTS as DWORD;
            break 'Cleanup;
        }

        dwErrorCode = RegQueryValueExW(
            hFormsKey,
            (*pfi1w).pName as LPCWSTR,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &raw mut cbNeeded,
        ) as DWORD;
        if dwErrorCode == ERROR_SUCCESS as DWORD {
            dwErrorCode = ERROR_FILE_EXISTS as DWORD;
            break 'Cleanup;
        }

        if wcslen((*pfi1w).pName as *const WCHAR) > FORMMAXNAMESIZE as usize {
            // Limit REG Name size.
            dwErrorCode = ERROR_INVALID_PARAMETER as DWORD;
            break 'Cleanup;
        }

        Size = (core::mem::size_of::<FORM_INFO_LIST>() as usize).wrapping_add(
            ((MAX_PATH + 1) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>()),
        ) as DWORD;

        pfil = DllAllocSplMem(Size) as PFORM_INFO_LIST;
        if pfil.is_null() {
            dwErrorCode = ERROR_NOT_ENOUGH_MEMORY as DWORD;
            break 'Cleanup;
        }

        (*pfil).Sig = FORMINFOSIG as DWORD;
        let fresh_index = _dwLastForm;
        _dwLastForm = _dwLastForm.wrapping_add(1);
        (*pfil).Index = fresh_index;
        (*pfil).pName = wcscpy(
            pfil.offset(1 as isize) as PWSTR,
            (*pfi1w).pName,
        );
        (*pfil).Flags = (*pfi1w).Flags;
        (*pfil).Size = (*pfi1w).Size;
        (*pfil).ImageableArea = (*pfi1w).ImageableArea;
        (*pfil).StringType = STRING_NONE as DWORD;

        if Level > 1 as DWORD {
            (*pfil).pKeyword = (*pfi2w).pKeyword as PWSTR;
            (*pfil).pMuiDll = (*pfi2w).pMuiDll as PWSTR;
            (*pfil).pDisplayName = (*pfi2w).pDisplayName as PWSTR;
            (*pfil).StringType = (*pfi2w).StringType;
            (*pfil).ResourceId = (*pfi2w).dwResourceId;
        }

        rfi.Size = (*pfil).Size;
        rfi.ImageableArea = (*pfil).ImageableArea;
        rfi.Index = (*pfil).Index;
        rfi.Flags = (*pfil).Flags;

        dwErrorCode = RegSetValueExW(
            hFormsKey,
            (*pfil).pName as LPCWSTR,
            0 as DWORD,
            REG_BINARY as DWORD,
            (&raw mut rfi) as PBYTE,
            core::mem::size_of::<REGISTRYFORMINFO>() as DWORD,
        ) as DWORD;

        BroadcastChange(pHandle);

        InsertTailList(&raw mut FormList, &raw mut (*pfil).List);
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS as DWORD) as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn LocalDeleteForm(hPrinter: HANDLE, pFormName: PWSTR) -> BOOL {
    let mut dwErrorCode: DWORD = 0;
    let mut cbNeeded: DWORD = 0;
    let mut pfil: PFORM_INFO_LIST = core::ptr::null_mut();
    let mut rfi: REGISTRYFORMINFO = core::mem::zeroed();
    let mut pHandle: PLOCAL_HANDLE = core::ptr::null_mut();

    'Cleanup: {
        pHandle = hPrinter as PLOCAL_HANDLE;
        if (*pHandle).HandleType as ::core::ffi::c_uint
            != HandleType_Printer as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'Cleanup;
        }

        pfil = FindForm(pFormName, core::ptr::null_mut());
        if pfil.is_null() {
            dwErrorCode = ERROR_INVALID_PARAMETER;
            break 'Cleanup;
        }

        dwErrorCode = RegQueryValueExW(
            hFormsKey,
            pFormName,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &raw mut rfi as PBYTE,
            &raw mut cbNeeded,
        ) as DWORD;
        if dwErrorCode != ERROR_SUCCESS {
            break 'Cleanup;
        }

        dwErrorCode = RegDeleteValueW(hFormsKey, pFormName) as DWORD;
        if dwErrorCode != ERROR_SUCCESS {
            break 'Cleanup;
        }

        RemoveEntryList(&raw mut (*pfil).List);

        DllFreeSplMem(pfil as PVOID);

        BroadcastChange(pHandle);

        dwErrorCode = ERROR_SUCCESS;
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn LocalEnumForms(hPrinter: HANDLE, Level: DWORD, mut pForm: PBYTE, cbBuf: DWORD, pcbNeeded: PDWORD, pcReturned: PDWORD) -> BOOL {
    let mut dwErrorCode: DWORD = 0;
    let mut pfil: PFORM_INFO_LIST = core::ptr::null_mut();
    let mut pEnd: PBYTE = pForm.add(cbBuf as usize);
    let mut pHandle: PLOCAL_HANDLE = core::ptr::null_mut();
    let mut ListEntry: PLIST_ENTRY = core::ptr::null_mut();

    'Cleanup: {
        // Check if this is a printer handle.
        pHandle = hPrinter as PLOCAL_HANDLE;
        if (*pHandle).HandleType != HandleType_Printer {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'Cleanup;
        }

        // Only support 1 & 2
        if Level < 1 || Level > 2 {
            // The caller supplied an invalid level.
            dwErrorCode = ERROR_INVALID_LEVEL;
            break 'Cleanup;
        }

        // Count the required buffer size.
        *pcbNeeded = 0;

        ListEntry = FormList.Flink;

        if IsListEmpty(ListEntry) {
            dwErrorCode = ERROR_INVALID_PARAMETER;
            break 'Cleanup;
        }

        while ListEntry != core::ptr::addr_of_mut!(FormList) {
            pfil = (ListEntry as ULONG_PTR)
                .wrapping_sub(core::mem::offset_of!(FORM_INFO_LIST, List) as ULONG_PTR)
                as PFORM_INFO_LIST;
            ListEntry = (*ListEntry).Flink as PLIST_ENTRY;

            pfnGetFormLevels[Level as usize].unwrap_unchecked()(
                pfil,
                core::ptr::null_mut(),
                core::ptr::null_mut::<PBYTE>(),
                pcbNeeded,
            );
        }

        // Check if the supplied buffer is large enough.
        if cbBuf < *pcbNeeded {
            dwErrorCode = ERROR_INSUFFICIENT_BUFFER;
            break 'Cleanup;
        }

        // Copy over the information.
        pEnd = pForm.add(*pcbNeeded as usize);

        ListEntry = FormList.Flink;

        while ListEntry != core::ptr::addr_of_mut!(FormList) {
            pfil = (ListEntry as ULONG_PTR)
                .wrapping_sub(core::mem::offset_of!(FORM_INFO_LIST, List) as ULONG_PTR)
                as PFORM_INFO_LIST;
            ListEntry = (*ListEntry).Flink as PLIST_ENTRY;

            pfnGetFormLevels[Level as usize].unwrap_unchecked()(
                pfil,
                core::ptr::addr_of_mut!(pForm) as PVOID,
                core::ptr::addr_of_mut!(pEnd),
                core::ptr::null_mut::<DWORD>(),
            );
            *pcReturned = (*pcReturned).wrapping_add(1);
        }

        dwErrorCode = ERROR_SUCCESS;
    }

    // Cleanup:
    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn LocalGetForm(hPrinter: HANDLE, mut pFormName: PWSTR, mut Level: DWORD, mut pForm: PBYTE, mut cbBuf: DWORD, mut pcbNeeded: PDWORD) -> BOOL {
    let mut dwErrorCode: DWORD = 0;
    let mut pfil: PFORM_INFO_LIST = core::ptr::null_mut();
    let mut pEnd: PBYTE = pForm.add(cbBuf as usize);
    let mut pHandle: PLOCAL_HANDLE = core::ptr::null_mut();

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/printing/providers/localspl/forms.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LocalGetForm\0" as *const u8 as *const ::core::ffi::c_char,
            631 as ::core::ffi::c_int,
            b"GetForm(%p, %S, %lu, %p, %lu, %p)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            hPrinter,
            pFormName,
            Level,
            pForm,
            cbBuf,
            pcbNeeded,
        );
    }

    'cleanup: {
        // Check if this is a printer handle.
        pHandle = hPrinter as PLOCAL_HANDLE;
        if (*pHandle).HandleType as ::core::ffi::c_uint
            != HandleType_Printer as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'cleanup;
        }

        // Only support 1 & 2
        if Level < 1 || Level > 2 {
            // The caller supplied an invalid level.
            dwErrorCode = ERROR_INVALID_LEVEL;
            break 'cleanup;
        }

        pfil = FindForm(pFormName, core::ptr::null_mut());
        if pfil.is_null() {
            dwErrorCode = ERROR_INVALID_PARAMETER;
            break 'cleanup;
        }

        // Count the required buffer size.
        *pcbNeeded = 0;

        pfnGetFormLevels[Level as usize].unwrap_unchecked()(
            pfil,
            core::ptr::null_mut(),
            core::ptr::null_mut::<PBYTE>(),
            pcbNeeded,
        );

        // Check if the supplied buffer is large enough.
        if cbBuf < *pcbNeeded {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"win32ss/printing/providers/localspl/forms.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"LocalGetForm\0" as *const u8 as *const ::core::ffi::c_char,
                    664 as ::core::ffi::c_int,
                    b"Insuffisient Buffer size\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
            dwErrorCode = ERROR_INSUFFICIENT_BUFFER;
            break 'cleanup;
        }

        // Copy over the information.
        pEnd = pForm.add(*pcbNeeded as usize);

        pfnGetFormLevels[Level as usize].unwrap_unchecked()(
            pfil,
            core::ptr::addr_of_mut!(pForm) as PVOID,
            core::ptr::addr_of_mut!(pEnd),
            core::ptr::null_mut::<DWORD>(),
        );

        dwErrorCode = ERROR_SUCCESS;
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn LocalSetForm(hPrinter: HANDLE, pFormName: PWSTR, Level: DWORD, pForm: PBYTE) -> BOOL {
    let mut dwErrorCode: DWORD = 0;
    let mut cbNeeded: DWORD = 0;
    let mut pfil: PFORM_INFO_LIST = core::ptr::null_mut();
    let mut rfi: REGISTRYFORMINFO = core::mem::zeroed();
    let mut pHandle: PLOCAL_HANDLE = core::ptr::null_mut();
    let pfi1w: PFORM_INFO_1W = pForm as PFORM_INFO_1W;
    let pfi2w: PFORM_INFO_2W = pForm as PFORM_INFO_2W;

    'cleanup: {
        // Check if this is a printer handle.
        pHandle = hPrinter as PLOCAL_HANDLE;
        if (*pHandle).HandleType != HandleType_Printer {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'cleanup;
        }

        // Only support 1 & 2
        if Level < 1 || Level > 2 {
            // The caller supplied an invalid level.
            dwErrorCode = ERROR_INVALID_LEVEL;
            break 'cleanup;
        }

        pfil = FindForm(pFormName, core::ptr::null_mut());
        if pfil.is_null() {
            dwErrorCode = ERROR_INVALID_PARAMETER;
            break 'cleanup;
        }

        dwErrorCode = RegQueryValueExW(
            hFormsKey,
            pFormName,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(rfi) as PBYTE,
            core::ptr::addr_of_mut!(cbNeeded),
        ) as DWORD;
        if dwErrorCode != ERROR_SUCCESS {
            break 'cleanup;
        }

        (*pfil).Flags = (*pfi1w).Flags;
        (*pfil).Size = (*pfi1w).Size;
        (*pfil).ImageableArea = (*pfi1w).ImageableArea;

        if Level > 1 {
            (*pfil).pKeyword = (*pfi2w).pKeyword as PWSTR;
            (*pfil).pMuiDll = (*pfi2w).pMuiDll as PWSTR;
            (*pfil).pDisplayName = (*pfi2w).pDisplayName as PWSTR;
            (*pfil).StringType = (*pfi2w).StringType;
            (*pfil).ResourceId = (*pfi2w).dwResourceId;
        }

        rfi.Size = (*pfil).Size;
        rfi.ImageableArea = (*pfil).ImageableArea;
        rfi.Flags = (*pfil).Flags;

        dwErrorCode = RegSetValueExW(
            hFormsKey,
            (*pfil).pName,
            0,
            REG_BINARY,
            core::ptr::addr_of_mut!(rfi) as PBYTE,
            core::mem::size_of::<REGISTRYFORMINFO>() as DWORD,
        ) as DWORD;

        BroadcastChange(pHandle);
    }

    // Cleanup:
    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}
