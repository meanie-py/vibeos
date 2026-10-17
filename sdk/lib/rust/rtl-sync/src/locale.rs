use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{LCID, LPCWSTR, NtQueryDefaultLocale, PCSTR, PCUNICODE_STRING, PLCID, PUNICODE_STRING, RtlInitUnicodeString, RtlIntegerToUnicodeString, _UNICODE_STRING};

type INT = i32;

// winnls.h / nls.h constants
const NLS_VALID_LOCALE_MASK: ULONG = 0x000F_FFFF;
const RTL_LOCALE_ALLOW_NEUTRAL_NAMES: ULONG = 0x0000_0002;
// Special LCIDs (MAKELCID of the well-known LANG/SUBLANG pairs)
const LOCALE_USER_DEFAULT: LCID = 0x0400;
const LOCALE_SYSTEM_DEFAULT: LCID = 0x0800;
const LOCALE_CUSTOM_DEFAULT: LCID = 0x0C00;
const LOCALE_CUSTOM_UI_DEFAULT: LCID = 0x1400;

// CRT helper (libcntpr / ntdll CRT) used by the locale lookup.
extern "C" {
    fn strlen(s: PCSTR) -> usize;
}

impl _LOCALE_ENTRY {
    /// Low 24 bits of the packed bitfield hold the LCID (top byte = flags).
    #[inline]
    fn Lcid(&self) -> LCID {
        (self._bitfield & 0x00FF_FFFF) as LCID
    }
}

// ---- file-scope preamble (translated) ----
pub const MAX_PRIMARY_LANGUAGE: ULONG = 0x3FF;

pub const MAX_BASIC_LCID: ULONG = 0x5FFF;

pub const LCID_ALT_NAME: ULONG = 0x00100000;

pub static mut RtlpUserDefaultLcid: LCID = 0;
pub static mut RtlpSystemDefaultLcid: LCID = 0;

#[repr(C)]
pub struct _LOCALE_ENTRY {
    pub Locale: [CHAR; 16],
    pub _bitfield: u32,
}
pub type LOCALE_ENTRY = _LOCALE_ENTRY;

const fn locale_str(s: &[u8]) -> [CHAR; 16] {
    let mut out = [0 as CHAR; 16];
    let mut i = 0;
    while i < s.len() && i < 16 {
        out[i] = s[i] as CHAR;
        i += 1;
    }
    out
}

const fn locale_bits(lcid: u32, flags: u32) -> u32 {
    (lcid & 0x00FF_FFFF) | ((flags & 0xFF) << 24)
}

pub static RtlpLocaleTable: [LOCALE_ENTRY; 448] = [
    _LOCALE_ENTRY { Locale: locale_str(b""),               _bitfield: locale_bits(0x0007F, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"af"),             _bitfield: locale_bits(0x00036, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"af-ZA"),          _bitfield: locale_bits(0x00436, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"am"),             _bitfield: locale_bits(0x0005E, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"am-ET"),          _bitfield: locale_bits(0x0045E, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ar"),             _bitfield: locale_bits(0x00001, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ar-AE"),          _bitfield: locale_bits(0x03801, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ar-BH"),          _bitfield: locale_bits(0x03C01, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ar-DZ"),          _bitfield: locale_bits(0x01401, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ar-EG"),          _bitfield: locale_bits(0x00C01, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ar-IQ"),          _bitfield: locale_bits(0x00801, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ar-JO"),          _bitfield: locale_bits(0x02C01, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ar-KW"),          _bitfield: locale_bits(0x03401, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ar-LB"),          _bitfield: locale_bits(0x03001, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ar-LY"),          _bitfield: locale_bits(0x01001, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ar-MA"),          _bitfield: locale_bits(0x01801, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ar-OM"),          _bitfield: locale_bits(0x02001, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ar-QA"),          _bitfield: locale_bits(0x04001, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ar-SA"),          _bitfield: locale_bits(0x00401, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ar-SY"),          _bitfield: locale_bits(0x02801, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ar-TN"),          _bitfield: locale_bits(0x01C01, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ar-YE"),          _bitfield: locale_bits(0x02401, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"arn"),            _bitfield: locale_bits(0x0007A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"arn-CL"),         _bitfield: locale_bits(0x0047A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"as"),             _bitfield: locale_bits(0x0004D, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"as-IN"),          _bitfield: locale_bits(0x0044D, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"az"),             _bitfield: locale_bits(0x0002C, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"az-Cyrl"),        _bitfield: locale_bits(0x0742C, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"az-Cyrl-AZ"),     _bitfield: locale_bits(0x0082C, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"az-Latn"),        _bitfield: locale_bits(0x0782C, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"az-Latn-AZ"),     _bitfield: locale_bits(0x0042C, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ba"),             _bitfield: locale_bits(0x0006D, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ba-RU"),          _bitfield: locale_bits(0x0046D, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"be"),             _bitfield: locale_bits(0x00023, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"be-BY"),          _bitfield: locale_bits(0x00423, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"bg"),             _bitfield: locale_bits(0x00002, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"bg-BG"),          _bitfield: locale_bits(0x00402, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"bin"),            _bitfield: locale_bits(0x00066, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"bin-NG"),         _bitfield: locale_bits(0x00466, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"bn"),             _bitfield: locale_bits(0x00045, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"bn-BD"),          _bitfield: locale_bits(0x00845, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"bn-IN"),          _bitfield: locale_bits(0x00445, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"bo"),             _bitfield: locale_bits(0x00051, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"bo-CN"),          _bitfield: locale_bits(0x00451, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"br"),             _bitfield: locale_bits(0x0007E, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"br-FR"),          _bitfield: locale_bits(0x0047E, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"bs"),             _bitfield: locale_bits(0x0781A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"bs-Cyrl"),        _bitfield: locale_bits(0x0641A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"bs-Cyrl-BA"),     _bitfield: locale_bits(0x0201A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"bs-Latn"),        _bitfield: locale_bits(0x0681A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"bs-Latn-BA"),     _bitfield: locale_bits(0x0141A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ca"),             _bitfield: locale_bits(0x00003, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ca-ES"),          _bitfield: locale_bits(0x00403, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ca-ES-valencia"), _bitfield: locale_bits(0x00803, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"chr"),            _bitfield: locale_bits(0x0005C, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"chr-Cher"),       _bitfield: locale_bits(0x07C5C, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"chr-Cher-US"),    _bitfield: locale_bits(0x0045C, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"co"),             _bitfield: locale_bits(0x00083, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"co-FR"),          _bitfield: locale_bits(0x00483, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"cs"),             _bitfield: locale_bits(0x00005, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"cs-CZ"),          _bitfield: locale_bits(0x00405, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"cy"),             _bitfield: locale_bits(0x00052, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"cy-GB"),          _bitfield: locale_bits(0x00452, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"da"),             _bitfield: locale_bits(0x00006, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"da-DK"),          _bitfield: locale_bits(0x00406, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"de"),             _bitfield: locale_bits(0x00007, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"de-AT"),          _bitfield: locale_bits(0x00C07, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"de-CH"),          _bitfield: locale_bits(0x00807, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"de-DE"),          _bitfield: locale_bits(0x00407, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"de-DE_phoneb"),   _bitfield: locale_bits(0x10407, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"de-LI"),          _bitfield: locale_bits(0x01407, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"de-LU"),          _bitfield: locale_bits(0x01007, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"dsb"),            _bitfield: locale_bits(0x07C2E, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"dsb-DE"),         _bitfield: locale_bits(0x0082E, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"dv"),             _bitfield: locale_bits(0x00065, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"dv-MV"),          _bitfield: locale_bits(0x00465, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"dz-BT"),          _bitfield: locale_bits(0x00C51, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"el"),             _bitfield: locale_bits(0x00008, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"el-GR"),          _bitfield: locale_bits(0x00408, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"en"),             _bitfield: locale_bits(0x00009, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"en-029"),         _bitfield: locale_bits(0x02409, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"en-AE"),          _bitfield: locale_bits(0x04C09, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"en-AU"),          _bitfield: locale_bits(0x00C09, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"en-BZ"),          _bitfield: locale_bits(0x02809, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"en-CA"),          _bitfield: locale_bits(0x01009, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"en-GB"),          _bitfield: locale_bits(0x00809, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"en-HK"),          _bitfield: locale_bits(0x03C09, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"en-ID"),          _bitfield: locale_bits(0x03809, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"en-IE"),          _bitfield: locale_bits(0x01809, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"en-IN"),          _bitfield: locale_bits(0x04009, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"en-JM"),          _bitfield: locale_bits(0x02009, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"en-MY"),          _bitfield: locale_bits(0x04409, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"en-NZ"),          _bitfield: locale_bits(0x01409, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"en-PH"),          _bitfield: locale_bits(0x03409, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"en-SG"),          _bitfield: locale_bits(0x04809, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"en-TT"),          _bitfield: locale_bits(0x02C09, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"en-US"),          _bitfield: locale_bits(0x00409, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"en-ZA"),          _bitfield: locale_bits(0x01C09, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"en-ZW"),          _bitfield: locale_bits(0x03009, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"es"),             _bitfield: locale_bits(0x0000A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"es-419"),         _bitfield: locale_bits(0x0580A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"es-AR"),          _bitfield: locale_bits(0x02C0A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"es-BO"),          _bitfield: locale_bits(0x0400A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"es-CL"),          _bitfield: locale_bits(0x0340A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"es-CO"),          _bitfield: locale_bits(0x0240A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"es-CR"),          _bitfield: locale_bits(0x0140A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"es-CU"),          _bitfield: locale_bits(0x05C0A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"es-DO"),          _bitfield: locale_bits(0x01C0A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"es-EC"),          _bitfield: locale_bits(0x0300A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"es-ES"),          _bitfield: locale_bits(0x00C0A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"es-ES_tradnl"),   _bitfield: locale_bits(0x0040A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"es-GT"),          _bitfield: locale_bits(0x0100A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"es-HN"),          _bitfield: locale_bits(0x0480A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"es-MX"),          _bitfield: locale_bits(0x0080A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"es-NI"),          _bitfield: locale_bits(0x04C0A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"es-PA"),          _bitfield: locale_bits(0x0180A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"es-PE"),          _bitfield: locale_bits(0x0280A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"es-PR"),          _bitfield: locale_bits(0x0500A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"es-PY"),          _bitfield: locale_bits(0x03C0A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"es-SV"),          _bitfield: locale_bits(0x0440A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"es-US"),          _bitfield: locale_bits(0x0540A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"es-UY"),          _bitfield: locale_bits(0x0380A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"es-VE"),          _bitfield: locale_bits(0x0200A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"et"),             _bitfield: locale_bits(0x00025, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"et-EE"),          _bitfield: locale_bits(0x00425, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"eu"),             _bitfield: locale_bits(0x0002D, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"eu-ES"),          _bitfield: locale_bits(0x0042D, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"fa"),             _bitfield: locale_bits(0x00029, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"fa-IR"),          _bitfield: locale_bits(0x00429, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ff"),             _bitfield: locale_bits(0x00067, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ff-Latn"),        _bitfield: locale_bits(0x07C67, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ff-Latn-NG"),     _bitfield: locale_bits(0x00467, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ff-Latn-SN"),     _bitfield: locale_bits(0x00867, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ff-NG"),          _bitfield: locale_bits(0x00467 | LCID_ALT_NAME, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"fi"),             _bitfield: locale_bits(0x0000B, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"fi-FI"),          _bitfield: locale_bits(0x0040B, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"fil"),            _bitfield: locale_bits(0x00064, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"fil-PH"),         _bitfield: locale_bits(0x00464, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"fo"),             _bitfield: locale_bits(0x00038, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"fo-FO"),          _bitfield: locale_bits(0x00438, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"fr"),             _bitfield: locale_bits(0x0000C, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"fr-029"),         _bitfield: locale_bits(0x01C0C, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"fr-BE"),          _bitfield: locale_bits(0x0080C, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"fr-CA"),          _bitfield: locale_bits(0x00C0C, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"fr-CD"),          _bitfield: locale_bits(0x0240C, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"fr-CH"),          _bitfield: locale_bits(0x0100C, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"fr-CI"),          _bitfield: locale_bits(0x0300C, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"fr-CM"),          _bitfield: locale_bits(0x02C0C, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"fr-FR"),          _bitfield: locale_bits(0x0040C, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"fr-HT"),          _bitfield: locale_bits(0x03C0C, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"fr-LU"),          _bitfield: locale_bits(0x0140C, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"fr-MA"),          _bitfield: locale_bits(0x0380C, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"fr-MC"),          _bitfield: locale_bits(0x0180C, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"fr-ML"),          _bitfield: locale_bits(0x0340C, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"fr-RE"),          _bitfield: locale_bits(0x0200C, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"fr-SN"),          _bitfield: locale_bits(0x0280C, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"fy"),             _bitfield: locale_bits(0x00062, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"fy-NL"),          _bitfield: locale_bits(0x00462, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ga"),             _bitfield: locale_bits(0x0003C, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ga-IE"),          _bitfield: locale_bits(0x0083C, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"gd"),             _bitfield: locale_bits(0x00091, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"gd-GB"),          _bitfield: locale_bits(0x00491, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"gl"),             _bitfield: locale_bits(0x00056, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"gl-ES"),          _bitfield: locale_bits(0x00456, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"gn"),             _bitfield: locale_bits(0x00074, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"gn-PY"),          _bitfield: locale_bits(0x00474, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"gsw"),            _bitfield: locale_bits(0x00084, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"gsw-FR"),         _bitfield: locale_bits(0x00484, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"gu"),             _bitfield: locale_bits(0x00047, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"gu-IN"),          _bitfield: locale_bits(0x00447, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ha"),             _bitfield: locale_bits(0x00068, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ha-Latn"),        _bitfield: locale_bits(0x07C68, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ha-Latn-NG"),     _bitfield: locale_bits(0x00468, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"haw"),            _bitfield: locale_bits(0x00075, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"haw-US"),         _bitfield: locale_bits(0x00475, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"he"),             _bitfield: locale_bits(0x0000D, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"he-IL"),          _bitfield: locale_bits(0x0040D, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"hi"),             _bitfield: locale_bits(0x00039, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"hi-IN"),          _bitfield: locale_bits(0x00439, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"hr"),             _bitfield: locale_bits(0x0001A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"hr-BA"),          _bitfield: locale_bits(0x0101A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"hr-HR"),          _bitfield: locale_bits(0x0041A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"hsb"),            _bitfield: locale_bits(0x0002E, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"hsb-DE"),         _bitfield: locale_bits(0x0042E, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"hu"),             _bitfield: locale_bits(0x0000E, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"hu-HU"),          _bitfield: locale_bits(0x0040E, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"hu-HU_technl"),   _bitfield: locale_bits(0x1040E, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"hy"),             _bitfield: locale_bits(0x0002B, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"hy-AM"),          _bitfield: locale_bits(0x0042B, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ibb"),            _bitfield: locale_bits(0x00069, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ibb-NG"),         _bitfield: locale_bits(0x00469, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"id"),             _bitfield: locale_bits(0x00021, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"id-ID"),          _bitfield: locale_bits(0x00421, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ig"),             _bitfield: locale_bits(0x00070, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ig-NG"),          _bitfield: locale_bits(0x00470, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ii"),             _bitfield: locale_bits(0x00078, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ii-CN"),          _bitfield: locale_bits(0x00478, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"is"),             _bitfield: locale_bits(0x0000F, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"is-IS"),          _bitfield: locale_bits(0x0040F, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"it"),             _bitfield: locale_bits(0x00010, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"it-CH"),          _bitfield: locale_bits(0x00810, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"it-IT"),          _bitfield: locale_bits(0x00410, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"iu"),             _bitfield: locale_bits(0x0005D, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"iu-Cans"),        _bitfield: locale_bits(0x0785D, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"iu-Cans-CA"),     _bitfield: locale_bits(0x0045D, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"iu-Latn"),        _bitfield: locale_bits(0x07C5D, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"iu-Latn-CA"),     _bitfield: locale_bits(0x0085D, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ja"),             _bitfield: locale_bits(0x00011, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ja-JP"),          _bitfield: locale_bits(0x00411, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ja-JP_radstr"),   _bitfield: locale_bits(0x040411, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ka"),             _bitfield: locale_bits(0x00037, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ka-GE"),          _bitfield: locale_bits(0x00437, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ka-GE_modern"),   _bitfield: locale_bits(0x10437, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"kk"),             _bitfield: locale_bits(0x0003F, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"kk-Cyrl"),        _bitfield: locale_bits(0x0003F | LCID_ALT_NAME, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"kk-KZ"),          _bitfield: locale_bits(0x0043F, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"kl"),             _bitfield: locale_bits(0x0006F, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"kl-GL"),          _bitfield: locale_bits(0x0046F, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"km"),             _bitfield: locale_bits(0x00053, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"km-KH"),          _bitfield: locale_bits(0x00453, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"kn"),             _bitfield: locale_bits(0x0004B, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"kn-IN"),          _bitfield: locale_bits(0x0044B, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ko"),             _bitfield: locale_bits(0x00012, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ko-KR"),          _bitfield: locale_bits(0x00412, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"kok"),            _bitfield: locale_bits(0x00057, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"kok-IN"),         _bitfield: locale_bits(0x00457, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"kr"),             _bitfield: locale_bits(0x00071, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"kr-Latn-NG"),     _bitfield: locale_bits(0x00471, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ks"),             _bitfield: locale_bits(0x00060, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ks-Arab"),        _bitfield: locale_bits(0x00460, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ks-Deva-IN"),     _bitfield: locale_bits(0x00860, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ku"),             _bitfield: locale_bits(0x00092, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ku-Arab"),        _bitfield: locale_bits(0x07C92, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ku-Arab-IQ"),     _bitfield: locale_bits(0x00492, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ky"),             _bitfield: locale_bits(0x00040, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ky-KG"),          _bitfield: locale_bits(0x00440, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"la"),             _bitfield: locale_bits(0x00076, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"la-001"),         _bitfield: locale_bits(0x00476, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"lb"),             _bitfield: locale_bits(0x0006E, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"lb-LU"),          _bitfield: locale_bits(0x0046E, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"lo"),             _bitfield: locale_bits(0x00054, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"lo-LA"),          _bitfield: locale_bits(0x00454, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"lt"),             _bitfield: locale_bits(0x00027, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"lt-LT"),          _bitfield: locale_bits(0x00427, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"lv"),             _bitfield: locale_bits(0x00026, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"lv-LV"),          _bitfield: locale_bits(0x00426, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"mi"),             _bitfield: locale_bits(0x00081, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"mi-NZ"),          _bitfield: locale_bits(0x00481, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"mk"),             _bitfield: locale_bits(0x0002F, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"mk-MK"),          _bitfield: locale_bits(0x0042F, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ml"),             _bitfield: locale_bits(0x0004C, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ml-IN"),          _bitfield: locale_bits(0x0044C, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"mn"),             _bitfield: locale_bits(0x00050, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"mn-Cyrl"),        _bitfield: locale_bits(0x07850, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"mn-MN"),          _bitfield: locale_bits(0x00450, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"mn-Mong"),        _bitfield: locale_bits(0x07C50, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"mn-Mong-CN"),     _bitfield: locale_bits(0x00850, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"mn-Mong-MN"),     _bitfield: locale_bits(0x00C50, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"mni"),            _bitfield: locale_bits(0x00058, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"mni-IN"),         _bitfield: locale_bits(0x00458, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"moh"),            _bitfield: locale_bits(0x0007C, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"moh-CA"),         _bitfield: locale_bits(0x0047C, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"mr"),             _bitfield: locale_bits(0x0004E, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"mr-IN"),          _bitfield: locale_bits(0x0044E, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ms"),             _bitfield: locale_bits(0x0003E, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ms-BN"),          _bitfield: locale_bits(0x0083E, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ms-MY"),          _bitfield: locale_bits(0x0043E, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"mt"),             _bitfield: locale_bits(0x0003A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"mt-MT"),          _bitfield: locale_bits(0x0043A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"my"),             _bitfield: locale_bits(0x00055, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"my-MM"),          _bitfield: locale_bits(0x00455, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"nb"),             _bitfield: locale_bits(0x07C14, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"nb-NO"),          _bitfield: locale_bits(0x00414, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ne"),             _bitfield: locale_bits(0x00061, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ne-IN"),          _bitfield: locale_bits(0x00861, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ne-NP"),          _bitfield: locale_bits(0x00461, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"nl"),             _bitfield: locale_bits(0x00013, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"nl-BE"),          _bitfield: locale_bits(0x00813, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"nl-NL"),          _bitfield: locale_bits(0x00413, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"nn"),             _bitfield: locale_bits(0x07814, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"nn-NO"),          _bitfield: locale_bits(0x00814, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"no"),             _bitfield: locale_bits(0x00014, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"nso"),            _bitfield: locale_bits(0x0006C, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"nso-ZA"),         _bitfield: locale_bits(0x0046C, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"oc"),             _bitfield: locale_bits(0x00082, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"oc-FR"),          _bitfield: locale_bits(0x00482, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"om"),             _bitfield: locale_bits(0x00072, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"om-ET"),          _bitfield: locale_bits(0x00472, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"or"),             _bitfield: locale_bits(0x00048, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"or-IN"),          _bitfield: locale_bits(0x00448, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"pa"),             _bitfield: locale_bits(0x00046, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"pa-Arab"),        _bitfield: locale_bits(0x07C46, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"pa-Arab-PK"),     _bitfield: locale_bits(0x00846, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"pa-IN"),          _bitfield: locale_bits(0x00446, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"pap"),            _bitfield: locale_bits(0x00079, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"pap-029"),        _bitfield: locale_bits(0x00479, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"pl"),             _bitfield: locale_bits(0x00015, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"pl-PL"),          _bitfield: locale_bits(0x00415, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"prs"),            _bitfield: locale_bits(0x0008C, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"prs-AF"),         _bitfield: locale_bits(0x0048C, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ps"),             _bitfield: locale_bits(0x00063, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ps-AF"),          _bitfield: locale_bits(0x00463, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"pt"),             _bitfield: locale_bits(0x00016, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"pt-BR"),          _bitfield: locale_bits(0x00416, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"pt-PT"),          _bitfield: locale_bits(0x00816, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"qps-ploc"),       _bitfield: locale_bits(0x00501, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"qps-ploca"),      _bitfield: locale_bits(0x005FE, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"qps-plocm"),      _bitfield: locale_bits(0x009FF, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"quc"),            _bitfield: locale_bits(0x00086, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"quc-Latn-GT"),    _bitfield: locale_bits(0x00486, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"qut"),            _bitfield: locale_bits(0x00086 | LCID_ALT_NAME, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"qut-GT"),         _bitfield: locale_bits(0x00486 | LCID_ALT_NAME, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"quz"),            _bitfield: locale_bits(0x0006B, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"quz-BO"),         _bitfield: locale_bits(0x0046B, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"quz-EC"),         _bitfield: locale_bits(0x0086B, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"quz-PE"),         _bitfield: locale_bits(0x00C6b, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"rm"),             _bitfield: locale_bits(0x00017, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"rm-CH"),          _bitfield: locale_bits(0x00417, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ro"),             _bitfield: locale_bits(0x00018, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ro-MD"),          _bitfield: locale_bits(0x00818, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ro-RO"),          _bitfield: locale_bits(0x00418, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ru"),             _bitfield: locale_bits(0x00019, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ru-MD"),          _bitfield: locale_bits(0x00819, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ru-RU"),          _bitfield: locale_bits(0x00419, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"rw"),             _bitfield: locale_bits(0x00087, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"rw-RW"),          _bitfield: locale_bits(0x00487, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"sa"),             _bitfield: locale_bits(0x0004F, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"sa-IN"),          _bitfield: locale_bits(0x0044F, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"sah"),            _bitfield: locale_bits(0x00085, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"sah-RU"),         _bitfield: locale_bits(0x00485, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"sd"),             _bitfield: locale_bits(0x00059, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"sd-Arab"),        _bitfield: locale_bits(0x07C59, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"sd-Arab-PK"),     _bitfield: locale_bits(0x00859, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"sd-Deva-IN"),     _bitfield: locale_bits(0x00459, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"se"),             _bitfield: locale_bits(0x0003B, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"se-FI"),          _bitfield: locale_bits(0x00C3B, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"se-NO"),          _bitfield: locale_bits(0x0043B, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"se-SE"),          _bitfield: locale_bits(0x0083B, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"si"),             _bitfield: locale_bits(0x0005B, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"si-LK"),          _bitfield: locale_bits(0x0045B, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"sk"),             _bitfield: locale_bits(0x0001B, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"sk-SK"),          _bitfield: locale_bits(0x0041B, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"sl"),             _bitfield: locale_bits(0x00024, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"sl-SI"),          _bitfield: locale_bits(0x00424, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"sma"),            _bitfield: locale_bits(0x0783B, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"sma-NO"),         _bitfield: locale_bits(0x0183B, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"sma-SE"),         _bitfield: locale_bits(0x01C3B, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"smj"),            _bitfield: locale_bits(0x07C3B, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"smj-NO"),         _bitfield: locale_bits(0x0103B, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"smj-SE"),         _bitfield: locale_bits(0x0143B, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"smn"),            _bitfield: locale_bits(0x0703B, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"smn-FI"),         _bitfield: locale_bits(0x0243B, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"sms"),            _bitfield: locale_bits(0x0743B, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"sms-FI"),         _bitfield: locale_bits(0x0203B, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"so"),             _bitfield: locale_bits(0x00077, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"so-SO"),          _bitfield: locale_bits(0x00477, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"sq"),             _bitfield: locale_bits(0x0001C, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"sq-AL"),          _bitfield: locale_bits(0x0041C, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"sr"),             _bitfield: locale_bits(0x07C1A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"sr-Cyrl"),        _bitfield: locale_bits(0x06C1A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"sr-Cyrl-BA"),     _bitfield: locale_bits(0x01C1A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"sr-Cyrl-CS"),     _bitfield: locale_bits(0x00C1A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"sr-Cyrl-ME"),     _bitfield: locale_bits(0x0301A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"sr-Cyrl-RS"),     _bitfield: locale_bits(0x0281A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"sr-Latn"),        _bitfield: locale_bits(0x0701A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"sr-Latn-BA"),     _bitfield: locale_bits(0x0181A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"sr-Latn-CS"),     _bitfield: locale_bits(0x0081A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"sr-Latn-ME"),     _bitfield: locale_bits(0x02C1A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"sr-Latn-RS"),     _bitfield: locale_bits(0x0241A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"st"),             _bitfield: locale_bits(0x00030, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"st-ZA"),          _bitfield: locale_bits(0x00430, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"sv"),             _bitfield: locale_bits(0x0001D, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"sv-FI"),          _bitfield: locale_bits(0x0081D, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"sv-SE"),          _bitfield: locale_bits(0x0041D, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"sw"),             _bitfield: locale_bits(0x00041, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"sw-KE"),          _bitfield: locale_bits(0x00441, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"syr"),            _bitfield: locale_bits(0x0005A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"syr-SY"),         _bitfield: locale_bits(0x0045A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ta"),             _bitfield: locale_bits(0x00049, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ta-IN"),          _bitfield: locale_bits(0x00449, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ta-LK"),          _bitfield: locale_bits(0x00849, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"te"),             _bitfield: locale_bits(0x0004A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"te-IN"),          _bitfield: locale_bits(0x0044A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"tg"),             _bitfield: locale_bits(0x00028, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"tg-Cyrl"),        _bitfield: locale_bits(0x07C28, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"tg-Cyrl-TJ"),     _bitfield: locale_bits(0x00428, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"th"),             _bitfield: locale_bits(0x0001E, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"th-TH"),          _bitfield: locale_bits(0x0041E, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ti"),             _bitfield: locale_bits(0x00073, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ti-ER"),          _bitfield: locale_bits(0x00873, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ti-ET"),          _bitfield: locale_bits(0x00473, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"tk"),             _bitfield: locale_bits(0x00042, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"tk-TM"),          _bitfield: locale_bits(0x00442, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"tn"),             _bitfield: locale_bits(0x00032, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"tn-BW"),          _bitfield: locale_bits(0x00832, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"tn-ZA"),          _bitfield: locale_bits(0x00432, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"tr"),             _bitfield: locale_bits(0x0001F, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"tr-TR"),          _bitfield: locale_bits(0x0041F, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ts"),             _bitfield: locale_bits(0x00031, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ts-ZA"),          _bitfield: locale_bits(0x00431, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"tt"),             _bitfield: locale_bits(0x00044, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"tt-RU"),          _bitfield: locale_bits(0x00444, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"tzm"),            _bitfield: locale_bits(0x0005F, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"tzm-Arab-MA"),    _bitfield: locale_bits(0x0045F, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"tzm-Latn"),       _bitfield: locale_bits(0x07C5F, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"tzm-Latn-DZ"),    _bitfield: locale_bits(0x0085F, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"tzm-Tfng"),       _bitfield: locale_bits(0x0785F, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"tzm-Tfng-MA"),    _bitfield: locale_bits(0x0105F, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ug"),             _bitfield: locale_bits(0x00080, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ug-CN"),          _bitfield: locale_bits(0x00480, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"uk"),             _bitfield: locale_bits(0x00022, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"uk-UA"),          _bitfield: locale_bits(0x00422, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ur"),             _bitfield: locale_bits(0x00020, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ur-IN"),          _bitfield: locale_bits(0x00820, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ur-PK"),          _bitfield: locale_bits(0x00420, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"uz"),             _bitfield: locale_bits(0x00043, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"uz-Cyrl"),        _bitfield: locale_bits(0x07843, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"uz-Cyrl-UZ"),     _bitfield: locale_bits(0x00843, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"uz-Latn"),        _bitfield: locale_bits(0x07C43, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"uz-Latn-UZ"),     _bitfield: locale_bits(0x00443, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ve"),             _bitfield: locale_bits(0x00033, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"ve-ZA"),          _bitfield: locale_bits(0x00433, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"vi"),             _bitfield: locale_bits(0x0002A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"vi-VN"),          _bitfield: locale_bits(0x0042A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"wo"),             _bitfield: locale_bits(0x00088, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"wo-SN"),          _bitfield: locale_bits(0x00488, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"xh"),             _bitfield: locale_bits(0x00034, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"xh-ZA"),          _bitfield: locale_bits(0x00434, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"yi"),             _bitfield: locale_bits(0x0003D, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"yi-001"),         _bitfield: locale_bits(0x0043D, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"yo"),             _bitfield: locale_bits(0x0006A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"yo-NG"),          _bitfield: locale_bits(0x0046A, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"zh"),             _bitfield: locale_bits(0x07804, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"zh-CN"),          _bitfield: locale_bits(0x00804, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"zh-CN_stroke"),   _bitfield: locale_bits(0x20804, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"zh-Hans"),        _bitfield: locale_bits(0x00004, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"zh-Hant"),        _bitfield: locale_bits(0x07C04, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"zh-HK"),          _bitfield: locale_bits(0x00C04, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"zh-HK_radstr"),   _bitfield: locale_bits(0x40C04, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"zh-MO"),          _bitfield: locale_bits(0x01404, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"zh-MO_radstr"),   _bitfield: locale_bits(0x41404, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"zh-SG"),          _bitfield: locale_bits(0x01004, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"zh-SG_stroke"),   _bitfield: locale_bits(0x21004, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"zh-TW"),          _bitfield: locale_bits(0x00404, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"zh-TW_pronun"),   _bitfield: locale_bits(0x30404, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"zh-TW_radstr"),   _bitfield: locale_bits(0x40404, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"zu"),             _bitfield: locale_bits(0x00035, 0) },
    _LOCALE_ENTRY { Locale: locale_str(b"zu-ZA"),          _bitfield: locale_bits(0x00435, 0) },
];

pub static mut RtlpLocaleIndexTable: [USHORT; 448] = [0; 448];


#[no_mangle]
pub unsafe extern "system" fn RtlpInitializeLocaleTable() -> NTSTATUS {
    NtQueryDefaultLocale(1u8, core::ptr::addr_of_mut!(RtlpUserDefaultLcid));
    NtQueryDefaultLocale(0u8, core::ptr::addr_of_mut!(RtlpSystemDefaultLcid));

    // An insertion sort into the index table, in place (upstream 3cc9aa99292): the kernel
    // initializes this table too now and has no process heap for the old qsort copy. It
    // also retires the comparator's `Lcid - Lcid` on unsigned values.
    let count = RtlpLocaleTable.len();
    let mut i: USHORT = 0;
    while (i as usize) < count {
        let Lcid = RtlpLocaleTable[i as usize].Lcid();
        let mut j: USHORT = i;
        while j > 0 && RtlpLocaleTable[RtlpLocaleIndexTable[(j - 1) as usize] as usize].Lcid() > Lcid {
            RtlpLocaleIndexTable[j as usize] = RtlpLocaleIndexTable[(j - 1) as usize];
            j -= 1;
        }
        RtlpLocaleIndexTable[j as usize] = i;
        i += 1;
    }

    STATUS_SUCCESS
}

unsafe fn FindIndexByLcid(Lcid: LCID) -> ULONG {
    let mut Low: LONG = 0;
    let mut High: LONG = (RtlpLocaleTable.len() as LONG) - 1;

    while Low <= High {
        let Middle: LONG = (Low + High) / 2;

        let TableIndex: USHORT = RtlpLocaleIndexTable[Middle as usize];

        let CurrentLcid: LCID = RtlpLocaleTable[TableIndex as usize].Lcid();
        if CurrentLcid < Lcid {
            Low = Middle + 1;
        } else if CurrentLcid > Lcid {
            High = Middle - 1;
        } else {
            return RtlpLocaleIndexTable[Middle as usize] as ULONG;
        }
    }

    MAXULONG
}

/// Windows treats `_` and `-` as the same separator when it looks a locale name up
/// ("en_US" -> 0x409, "ja-JP-radstr" -> 0x40411; measured with RtlLocaleNameToLcid and
/// RtlIsValidLocaleName on Windows 10.0.26300). The table spells sort suffixes with
/// `_`, so both sides fold to `-`; the table stays sorted under the fold (a host test
/// checks every adjacent pair), so the binary search is unaffected.
#[inline]
fn SeparatorFold(c: u16) -> u16 {
    if c == b'_' as u16 { b'-' as u16 } else { c }
}

unsafe fn CompareLocaleNames(
    AsciiLocaleName: PCSTR,
    UnicodeLocaleName: PCUNICODE_STRING,
) -> INT {
    let mut i: ULONG = 0;

    while i < (*UnicodeLocaleName).Length as ULONG / core::mem::size_of::<u16>() as ULONG {
        let RawChar1 = SeparatorFold(*AsciiLocaleName.offset(i as isize) as u8 as u16) as u8;
        let Char1: u8 = if RawChar1 >= b'A' && RawChar1 <= b'Z' {
            RawChar1 + (b'a' - b'A')
        } else {
            RawChar1
        };

        let RawChar2 = SeparatorFold(*(*UnicodeLocaleName).Buffer.offset(i as isize));
        let Char2: u16 = if RawChar2 >= b'A' as u16 && RawChar2 <= b'Z' as u16 {
            RawChar2 + (b'a' - b'A') as u16
        } else {
            RawChar2
        };

        if Char1 as u16 == Char2 {
            i += 1;
            continue;
        }

        if *AsciiLocaleName.offset(i as isize) == 0 {
            return -1;
        }

        return Char1 as INT - Char2 as INT;
    }

    if *AsciiLocaleName.offset(i as isize) == 0 {
        return 0;
    }

    1
}

unsafe fn FindIndexByLocaleName(LocaleName: PCUNICODE_STRING) -> ULONG {
    let mut Low: LONG = 0;
    let mut High: LONG = (RtlpLocaleTable.len() as LONG) - 1;
    let mut Middle: LONG;
    let mut CurrentLocaleName: PCSTR;
    let mut CompareResult: INT;

    while Low <= High {
        Middle = (Low + High) / 2;

        CurrentLocaleName = RtlpLocaleTable[Middle as usize].Locale.as_ptr() as PCSTR;
        CompareResult = CompareLocaleNames(CurrentLocaleName, LocaleName);
        if CompareResult < 0 {
            Low = Middle + 1;
        } else if CompareResult > 0 {
            High = Middle - 1;
        } else {
            return Middle as ULONG;
        }
    }

    MAXULONG
}

unsafe fn CopyAsciizToUnicodeString(
    UnicodeString: PUNICODE_STRING,
    AsciiString: PCSTR,
) -> u8 {
    let AsciiLength = strlen(AsciiString) as usize;

    if ((*UnicodeString).MaximumLength as usize) < (AsciiLength + 1) * core::mem::size_of::<u16>() {
        return 0u8;
    }

    let mut i: usize = 0;
    while i < AsciiLength {
        *(*UnicodeString).Buffer.add(i) = *(AsciiString as *const i8).add(i) as u16;
        i += 1;
    }

    *(*UnicodeString).Buffer.add(AsciiLength) = 0u16;
    (*UnicodeString).Length = (AsciiLength * core::mem::size_of::<u16>()) as u16;

    1u8
}

unsafe fn IsNeutralLocale(Lcid: LCID) -> u8 {
    if ((Lcid <= MAX_PRIMARY_LANGUAGE) && (Lcid != 0x007F)) ||
        (Lcid == 0x0460) ||
        ((Lcid & 0xFFFF) > MAX_BASIC_LCID)
    {
        return 1u8;
    }

    0u8
}

#[no_mangle]
pub unsafe extern "system" fn RtlLcidToLocaleName(
    mut Lcid: LCID,
    LocaleName: PUNICODE_STRING,
    Flags: ULONG,
    AllocateDestinationString: u8,
) -> NTSTATUS {
    let LocaleIndex: ULONG;

    /* Check for invalid flags */
    if (Flags & !0x2) != 0 {
        return STATUS_INVALID_PARAMETER_3;
    }

    /* Check if the LocaleName buffer is valid */
    if LocaleName.is_null() || (*LocaleName).Buffer.is_null() {
        return STATUS_INVALID_PARAMETER_2;
    }

    /* Validate LCID */
    if (Lcid & !NLS_VALID_LOCALE_MASK) != 0 {
        return STATUS_INVALID_PARAMETER_1;
    }

    /* Check if neutral locales were requested */
    if (Flags & RTL_LOCALE_ALLOW_NEUTRAL_NAMES) == 0 {
        /* Check if this is a neutral locale */
        if IsNeutralLocale(Lcid) != 0 {
            return STATUS_INVALID_PARAMETER_1;
        }
    }

    /* Handle special LCIDs */
    match Lcid {
        LOCALE_USER_DEFAULT => {
            Lcid = RtlpUserDefaultLcid;
        }
        LOCALE_SYSTEM_DEFAULT | LOCALE_CUSTOM_DEFAULT => {
            Lcid = RtlpSystemDefaultLcid;
        }
        LOCALE_CUSTOM_UI_DEFAULT => {
            return STATUS_UNSUCCESSFUL;
        }
        _ => {}
    }

    /* Try to find the locale by LCID */
    LocaleIndex = FindIndexByLcid(Lcid);
    if LocaleIndex == MAXULONG {
        return STATUS_INVALID_PARAMETER_1;
    }

    /* Copy the locale name to the buffer */
    if CopyAsciizToUnicodeString(
        LocaleName,
        RtlpLocaleTable[LocaleIndex as usize].Locale.as_ptr() as PCSTR,
    ) == 0
    {
        return STATUS_BUFFER_TOO_SMALL;
    }

    STATUS_SUCCESS
}

unsafe fn RtlpLocaleNameToLcidInternal(
    LocaleName: PCUNICODE_STRING,
    Lcid: PLCID,
    Flags: ULONG,
) -> NTSTATUS {
    let LocaleIndex: ULONG;
    let FoundLcid: LCID;

    if LocaleName.is_null() || (*LocaleName).Buffer.is_null() {
        return STATUS_INVALID_PARAMETER_1;
    }

    if Lcid.is_null() {
        return STATUS_INVALID_PARAMETER_2;
    }

    if Flags & !0x3 != 0 {
        return STATUS_INVALID_PARAMETER_3;
    }

    LocaleIndex = FindIndexByLocaleName(LocaleName);
    if LocaleIndex == MAXULONG {
        return STATUS_INVALID_PARAMETER_1;
    }

    FoundLcid = RtlpLocaleTable[LocaleIndex as usize].Lcid() & NLS_VALID_LOCALE_MASK;

    if (Flags & RTL_LOCALE_ALLOW_NEUTRAL_NAMES) == 0 {
        if IsNeutralLocale(FoundLcid) != 0 {
            return STATUS_INVALID_PARAMETER_1;
        }
    }

    *Lcid = FoundLcid;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlLocaleNameToLcid(
    LocaleName: PCWSTR,
    Lcid: PLCID,
    Flags: ULONG,
) -> NTSTATUS {
    let mut LocaleNameString: _UNICODE_STRING = core::mem::zeroed();

    RtlInitUnicodeString(&mut LocaleNameString, LocaleName);

    RtlpLocaleNameToLcidInternal(&LocaleNameString, Lcid, Flags)
}

#[no_mangle]
pub unsafe extern "system" fn RtlLCIDToCultureName(
    Lcid: LCID,
    String: PUNICODE_STRING,
) -> u8 {
    let Status = RtlLcidToLocaleName(Lcid, String, RTL_LOCALE_ALLOW_NEUTRAL_NAMES, 0u8);
    if !NT_SUCCESS(Status) {
        return 0u8;
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn RtlCultureNameToLCID(
    String: PCUNICODE_STRING,
    Lcid: PLCID,
) -> BOOLEAN {
    if String.is_null()
        || (*String).Buffer.is_null()
        || *(*String).Buffer == 0
    {
        return 0u8;
    }

    let Status = RtlpLocaleNameToLcidInternal(String, Lcid, RTL_LOCALE_ALLOW_NEUTRAL_NAMES);
    if !NT_SUCCESS(Status) {
        return 0u8;
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn RtlIsValidLocaleName(
    LocaleName: LPCWSTR,
    Flags: ULONG,
) -> BOOLEAN {
    // Windows (measured): any flag other than RTL_LOCALE_ALLOW_NEUTRAL_NAMES makes the
    // answer FALSE — even 1, which RtlLocaleNameToLcid itself accepts. Otherwise a name is
    // valid exactly when RtlLocaleNameToLcid would resolve it with the same flags: so ""
    // (the invariant locale) is valid, and a neutral name ("en") only with flag 2.
    if (Flags & !RTL_LOCALE_ALLOW_NEUTRAL_NAMES) != 0 || LocaleName.is_null() {
        return 0u8;
    }
    let mut Lcid: LCID = 0;
    NT_SUCCESS(RtlLocaleNameToLcid(LocaleName, &mut Lcid, Flags)) as BOOLEAN
}

#[no_mangle]
pub unsafe extern "system" fn RtlConvertLCIDToString(
    LcidValue: LCID,
    Base: ULONG,
    Padding: ULONG,
    pResultBuf: PWSTR,
    Size: ULONG,
) -> NTSTATUS {
    // The contract, measured black-box on Windows 10.0.26300 (Size is in WCHARs):
    //  1. RtlIntegerToUnicodeString(Lcid, Base) into a string whose capacity is Size WCHARs.
    //     Its failures come back unchanged and NOTHING is written: a bad Base is
    //     STATUS_INVALID_PARAMETER, too small a Size (digits + NUL) STATUS_BUFFER_OVERFLOW.
    //  2. Left-pad with '0' up to Padding digits — written straight into pResultBuf and
    //     clipped at Size, so a pad wider than the buffer fills it with '0' and no NUL.
    //  3. Copy the digits after the pad StringCchCopyW-style: truncate to the room left,
    //     NUL-terminate when there is any room at all. Any truncation, or no room, is
    //     STATUS_UNSUCCESSFUL — with the partial text left in the buffer.
    // The digits are formed in a local buffer: 32 binary digits + NUL is the most any
    // Base can produce, so capping the capacity there changes no outcome.
    const MAX_DIGITS_WITH_NUL: ULONG = 33;
    let mut Digits = [0u16; MAX_DIGITS_WITH_NUL as usize];
    let Capacity = if Size < MAX_DIGITS_WITH_NUL { Size } else { MAX_DIGITS_WITH_NUL };
    let mut DigitString = _UNICODE_STRING {
        Length: 0,
        MaximumLength: (Capacity * core::mem::size_of::<u16>() as ULONG) as USHORT,
        Buffer: Digits.as_mut_ptr(),
    };
    let Status = RtlIntegerToUnicodeString(LcidValue, Base, &mut DigitString);
    if !NT_SUCCESS(Status) {
        return Status;
    }
    let DigitCount = DigitString.Length as ULONG / core::mem::size_of::<u16>() as ULONG;

    let mut Written: ULONG = 0;
    if Padding > DigitCount {
        let Zeros = Padding - DigitCount;
        Written = if Zeros < Size { Zeros } else { Size };
        let mut k: ULONG = 0;
        while k < Written {
            *pResultBuf.add(k as usize) = b'0' as u16;
            k += 1;
        }
    }

    let Room = Size - Written;
    if Room == 0 {
        return STATUS_UNSUCCESSFUL;
    }
    let Copied = if DigitCount < Room { DigitCount } else { Room - 1 };
    core::ptr::copy_nonoverlapping(Digits.as_ptr(), pResultBuf.add(Written as usize), Copied as usize);
    *pResultBuf.add((Written + Copied) as usize) = 0;
    if Copied < DigitCount {
        return STATUS_UNSUCCESSFUL;
    }
    STATUS_SUCCESS
}
