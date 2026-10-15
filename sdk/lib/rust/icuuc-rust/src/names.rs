//! Converter name table + ICU-style alias matching.
//!
//! Canonical names, IANA/MIME tags and aliases were taken from ICU's
//! `icu4c/source/data/mappings/convrtrs.txt` (main branch, fetched 2026-10-15).
//! Strings that are returned to C carry an explicit trailing NUL.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Utf8,
    Utf16Le,
    Utf16Be,
    Ascii,
    Latin1,
    /// Anything that goes through the platform (kernel32) code page seam.
    Cp,
}

pub struct Entry {
    /// ICU canonical internal name (what ucnv_getName returns), NUL-terminated.
    pub name: &'static str,
    /// Alias tagged MIME* in convrtrs.txt (NUL-terminated), if any.
    pub mime: Option<&'static str>,
    /// Alias tagged IANA* in convrtrs.txt (NUL-terminated), if any.
    pub iana: Option<&'static str>,
    pub kind: Kind,
    /// Windows code page used for Kind::Cp (informational for native kinds).
    pub cp: u32,
    /// ucnv_getMaxCharSize value (max bytes per UChar), as ICU reports it.
    pub max: i8,
    pub aliases: &'static [&'static str],
}

macro_rules! e {
    ($name:expr, $mime:expr, $iana:expr, $kind:ident, $cp:expr, $max:expr, [$($a:expr),* $(,)?]) => {
        Entry { name: $name, mime: $mime, iana: $iana, kind: Kind::$kind, cp: $cp, max: $max, aliases: &[$($a),*] }
    };
}

pub static ENTRIES: &[Entry] = &[
    e!("UTF-8\0", Some("UTF-8\0"), Some("UTF-8\0"), Utf8, 65001, 3,
       ["utf8", "ibm-1208", "ibm-1209", "ibm-5304", "ibm-5305", "ibm-13496", "ibm-13497", "windows-65001",
        "cp1208", "x-UTF_8J", "unicode-1-1-utf-8", "unicode-2-0-utf-8"]),
    e!("UTF-16LE\0", Some("UTF-16LE\0"), Some("UTF-16LE\0"), Utf16Le, 1200, 2,
       ["x-utf-16le", "UnicodeLittleUnmarked", "ibm-1202", "ibm-13490", "ibm-17586", "UTF16_LittleEndian", "windows-1200"]),
    e!("UTF-16BE\0", Some("UTF-16BE\0"), Some("UTF-16BE\0"), Utf16Be, 1201, 2,
       ["x-utf-16be", "UnicodeBigUnmarked", "ibm-1200", "ibm-1201", "ibm-13488", "ibm-17584", "ibm-21680", "ibm-25776",
        "ibm-29872", "ibm-29873", "cp1200", "cp1201", "UTF16_BigEndian", "windows-1201"]),
    e!("US-ASCII\0", Some("US-ASCII\0"), Some("ANSI_X3.4-1968\0"), Ascii, 20127, 1,
       ["ASCII", "ANSI_X3.4-1968", "ANSI_X3.4-1986", "ISO_646.irv:1991", "iso_646.irv:1983", "ISO646-US", "us",
        "csASCII", "iso-ir-6", "cp367", "ascii7", "646", "windows-20127", "ibm-367", "IBM367"]),
    e!("ISO-8859-1\0", Some("ISO-8859-1\0"), Some("ISO_8859-1:1987\0"), Latin1, 28591, 1,
       ["ibm-819", "IBM819", "cp819", "latin1", "8859_1", "csISOLatin1", "iso-ir-100", "ISO_8859-1:1987", "l1", "819",
        "windows-28591"]),
    e!("ibm-912_P100-1995\0", Some("ISO-8859-2\0"), Some("ISO-8859-2\0"), Cp, 28592, 1,
       ["ibm-912", "cp912", "912", "latin2", "l2", "iso-ir-101", "ISO_8859-2:1987", "8859_2", "windows-28592"]),
    e!("ibm-913_P100-2000\0", Some("ISO-8859-3\0"), Some("ISO-8859-3\0"), Cp, 28593, 1,
       ["ibm-913", "cp913", "913", "latin3", "l3", "iso-ir-109", "ISO_8859-3:1988", "8859_3", "windows-28593"]),
    e!("ibm-914_P100-1995\0", Some("ISO-8859-4\0"), Some("ISO-8859-4\0"), Cp, 28594, 1,
       ["ibm-914", "cp914", "914", "latin4", "l4", "iso-ir-110", "ISO_8859-4:1988", "8859_4", "windows-28594"]),
    e!("ibm-915_P100-1995\0", Some("ISO-8859-5\0"), Some("ISO-8859-5\0"), Cp, 28595, 1,
       ["ibm-915", "cp915", "915", "cyrillic", "iso-ir-144", "ISO_8859-5:1988", "8859_5", "windows-28595"]),
    e!("ibm-1089_P100-1995\0", Some("ISO-8859-6\0"), Some("ISO-8859-6\0"), Cp, 28596, 1,
       ["ibm-1089", "arabic", "iso-ir-127", "ISO_8859-6:1987", "8859_6", "windows-28596"]),
    e!("ibm-9005_X110-2007\0", Some("ISO-8859-7\0"), Some("ISO-8859-7\0"), Cp, 28597, 1,
       ["ibm-9005", "greek", "greek8", "iso-ir-126", "ISO_8859-7:1987", "8859_7", "windows-28597"]),
    e!("ibm-5012_P100-1999\0", Some("ISO-8859-8\0"), Some("ISO-8859-8\0"), Cp, 28598, 1,
       ["ibm-5012", "hebrew", "iso-ir-138", "ISO_8859-8:1988", "8859_8", "windows-28598"]),
    e!("ibm-920_P100-1995\0", Some("ISO-8859-9\0"), Some("ISO-8859-9\0"), Cp, 28599, 1,
       ["ibm-920", "cp920", "920", "latin5", "l5", "iso-ir-148", "ISO_8859-9:1989", "8859_9", "windows-28599"]),
    e!("ibm-921_P100-1995\0", Some("ISO-8859-13\0"), Some("ISO-8859-13\0"), Cp, 28603, 1,
       ["ibm-921", "cp921", "921", "8859_13", "windows-28603"]),
    e!("ibm-923_P100-1998\0", Some("ISO-8859-15\0"), Some("ISO-8859-15\0"), Cp, 28605, 1,
       ["ibm-923", "cp923", "923", "latin9", "l9", "Latin-9", "8859_15", "windows-28605"]),
    e!("ibm-5346_P100-1998\0", None, Some("windows-1250\0"), Cp, 1250, 1, ["ibm-5346", "cp1250"]),
    e!("ibm-5347_P100-1998\0", None, Some("windows-1251\0"), Cp, 1251, 1, ["ibm-5347", "cp1251", "ANSI1251"]),
    e!("ibm-5348_P100-1997\0", None, Some("windows-1252\0"), Cp, 1252, 1, ["ibm-5348", "cp1252"]),
    e!("ibm-5349_P100-1998\0", None, Some("windows-1253\0"), Cp, 1253, 1, ["ibm-5349", "cp1253"]),
    e!("ibm-5350_P100-1998\0", None, Some("windows-1254\0"), Cp, 1254, 1, ["ibm-5350", "cp1254"]),
    e!("ibm-9447_P100-2002\0", None, Some("windows-1255\0"), Cp, 1255, 1, ["ibm-9447", "cp1255"]),
    e!("ibm-9448_X100-2005\0", None, Some("windows-1256\0"), Cp, 1256, 1, ["ibm-9448", "cp1256", "x-windows-1256S"]),
    e!("ibm-9449_P100-2002\0", None, Some("windows-1257\0"), Cp, 1257, 1, ["ibm-9449", "cp1257"]),
    e!("ibm-5354_P100-1998\0", None, Some("windows-1258\0"), Cp, 1258, 1, ["ibm-5354", "cp1258"]),
    e!("windows-874-2000\0", None, None, Cp, 874, 1, ["windows-874", "TIS-620", "MS874", "x-windows-874", "cp874"]),
    // Windows code page choices below: 932 for Shift_JIS, 51932 for EUC-JP, 936 for GBK and GB2312
    // (GBK is a superset of GB2312), 949 for EUC-KR (UHC is a superset of EUC-KR).
    e!("ibm-943_P15A-2003\0", Some("Shift_JIS\0"), Some("Shift_JIS\0"), Cp, 932, 2,
       ["ibm-943", "MS_Kanji", "csShiftJIS", "windows-31j", "csWindows31J", "x-sjis", "x-ms-cp932", "cp932",
        "windows-932", "cp943c", "ms932", "pck", "sjis"]),
    e!("euc-jp-2007\0", Some("EUC-JP\0"), Some("EUC-JP\0"), Cp, 51932, 3,
       ["Extended_UNIX_Code_Packed_Format_for_Japanese", "csEUCPkdFmtJapanese", "X-EUC-JP", "eucjis", "ujis"]),
    e!("windows-936-2000\0", None, Some("GBK\0"), Cp, 936, 2, ["CP936", "MS936", "windows-936"]),
    e!("ibm-1383_P110-1999\0", Some("GB2312\0"), Some("GB2312\0"), Cp, 936, 2,
       ["ibm-1383", "csGB2312", "cp1383", "1383", "EUC-CN", "euccn"]),
    e!("gb18030\0", Some("GB18030\0"), Some("GB18030\0"), Cp, 54936, 4,
       ["gb18030-2022", "ibm-1392", "windows-54936"]),
    e!("windows-950-2000\0", Some("Big5\0"), Some("Big5\0"), Cp, 950, 2,
       ["csBig5", "windows-950", "x-windows-950", "x-big5", "ms950", "cp950"]),
    e!("ibm-970_P110_P110-2006_U2\0", Some("EUC-KR\0"), Some("EUC-KR\0"), Cp, 949, 2,
       ["ibm-970", "KS_C_5601-1987", "windows-51949", "csEUCKR", "ibm-eucKR", "KSC_5601", "5601"]),
    e!("windows-949-2000\0", Some("KSC_5601\0"), None, Cp, 949, 2, ["windows-949", "ms949", "cp949"]),
    e!("ibm-878_P100-1996\0", Some("KOI8-R\0"), Some("KOI8-R\0"), Cp, 20866, 1,
       ["ibm-878", "koi8", "csKOI8R", "windows-20866", "cp878"]),
    e!("ibm-1168_P100-2002\0", None, Some("KOI8-U\0"), Cp, 21866, 1, ["ibm-1168", "windows-21866"]),
    e!("ibm-437_P100-1995\0", None, Some("IBM437\0"), Cp, 437, 1,
       ["ibm-437", "cp437", "437", "csPC8CodePage437", "windows-437"]),
    e!("ibm-850_P100-1995\0", Some("IBM850\0"), Some("IBM850\0"), Cp, 850, 1,
       ["ibm-850", "cp850", "850", "csPC850Multilingual", "windows-850"]),
    e!("ibm-852_P100-1995\0", None, Some("IBM852\0"), Cp, 852, 1,
       ["ibm-852", "cp852", "852", "csPCp852", "windows-852"]),
    e!("ibm-866_P100-1995\0", Some("IBM866\0"), Some("IBM866\0"), Cp, 866, 1,
       ["ibm-866", "cp866", "866", "csIBM866", "windows-866"]),
    // Test-only converters backed by the fake platform in plat.rs.
    #[cfg(test)]
    e!("fake-dbcs\0", None, None, Cp, 0xFFF1, 2, []),
    #[cfg(test)]
    e!("fake-sbcs\0", None, None, Cp, 0xFFF2, 1, []),
];

struct Norm<'a> {
    s: &'a [u8],
    i: usize,
    after_digit: bool,
}

impl Norm<'_> {
    /// ICU ucnv_compareNames rules: ASCII case-insensitive, ignore '-', '_', ' ',
    /// and ignore a '0' that follows a non-digit and precedes a digit.
    fn next(&mut self) -> Option<u8> {
        loop {
            let c = *self.s.get(self.i)?;
            if c == 0 {
                return None;
            }
            self.i += 1;
            if c == b'-' || c == b'_' || c == b' ' {
                self.after_digit = false;
                continue;
            }
            if c == b'0' && !self.after_digit {
                if let Some(&n) = self.s.get(self.i) {
                    if n.is_ascii_digit() {
                        continue;
                    }
                }
            }
            self.after_digit = c.is_ascii_digit();
            return Some(c.to_ascii_lowercase());
        }
    }
}

pub fn names_match(a: &[u8], b: &[u8]) -> bool {
    let mut x = Norm { s: a, i: 0, after_digit: false };
    let mut y = Norm { s: b, i: 0, after_digit: false };
    loop {
        match (x.next(), y.next()) {
            (None, None) => return true,
            (Some(p), Some(q)) if p == q => {}
            _ => return false,
        }
    }
}

fn entry_matches(e: &Entry, name: &[u8]) -> bool {
    names_match(e.name.as_bytes(), name)
        || e.mime.map_or(false, |m| names_match(m.as_bytes(), name))
        || e.iana.map_or(false, |m| names_match(m.as_bytes(), name))
        || e.aliases.iter().any(|a| names_match(a.as_bytes(), name))
}

/// Look up a converter by any name/alias. Anything after a ',' (ICU option suffix such
/// as ",swaplfnl") is ignored.
pub fn find(name: &[u8]) -> Option<&'static Entry> {
    let mut end = 0;
    while end < name.len() && name[end] != b',' {
        end += 1;
    }
    let name = &name[..end];
    if name.is_empty() {
        return None;
    }
    ENTRIES.iter().find(|e| entry_matches(e, name))
}

pub fn find_by_cp(cp: u32) -> Option<&'static Entry> {
    ENTRIES.iter().find(|e| e.cp == cp)
}
