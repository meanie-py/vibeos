use crate::*;
use core::ffi::c_void;
use std::ffi::CStr;
use std::ptr::{null, null_mut};

fn open(name: &str) -> *mut UConverter {
    let mut e = 0;
    let c = std::ffi::CString::new(name).unwrap();
    let p = unsafe { ucnv_open(c.as_ptr(), &mut e) };
    assert_eq!(e, 0, "open {name}");
    assert!(!p.is_null());
    p
}
fn name_of(c: *mut UConverter) -> String {
    let mut e = 0;
    unsafe { CStr::from_ptr(ucnv_getName(c, &mut e)).to_str().unwrap().to_string() }
}
fn std_name(n: &str, s: &str) -> Option<String> {
    let (n, s) = (std::ffi::CString::new(n).unwrap(), std::ffi::CString::new(s).unwrap());
    let mut e = 0;
    let p = unsafe { ucnv_getStandardName(n.as_ptr(), s.as_ptr(), &mut e) };
    assert_eq!(e, 0);
    if p.is_null() { None } else { Some(unsafe { CStr::from_ptr(p) }.to_str().unwrap().to_string()) }
}

/// One toUnicode call. Returns (consumed bytes, produced units, err).
fn to_u(c: *mut UConverter, src: &[u8], cap: usize, flush: bool) -> (usize, Vec<u16>, i32) {
    let mut out = vec![0u16; cap];
    let mut t = out.as_mut_ptr();
    let mut s = src.as_ptr() as *const CChar;
    let mut e = 0;
    unsafe {
        ucnv_toUnicode(c, &mut t, out.as_ptr().add(cap), &mut s, (src.as_ptr() as *const CChar).add(src.len()),
                       null_mut(), flush as i8, &mut e);
        let n = t.offset_from(out.as_ptr()) as usize;
        out.truncate(n);
        (s.offset_from(src.as_ptr() as *const CChar) as usize, out, e)
    }
}
fn from_u(c: *mut UConverter, src: &[u16], cap: usize, flush: bool) -> (usize, Vec<u8>, i32) {
    let mut out = vec![0u8; cap];
    let mut t = out.as_mut_ptr() as *mut CChar;
    let mut s = src.as_ptr();
    let mut e = 0;
    unsafe {
        ucnv_fromUnicode(c, &mut t, (out.as_ptr() as *const CChar).add(cap), &mut s, src.as_ptr().add(src.len()),
                         null_mut(), flush as i8, &mut e);
        let n = t.offset_from(out.as_ptr() as *const CChar) as usize;
        out.truncate(n);
        (s.offset_from(src.as_ptr()) as usize, out, e)
    }
}
/// Convert everything with chunked input and a tiny output buffer, resuming on overflow.
fn decode_chunks(c: *mut UConverter, chunks: &[&[u8]], cap: usize) -> Vec<u16> {
    let mut all = vec![];
    for (i, ch) in chunks.iter().enumerate() {
        let last = i + 1 == chunks.len();
        let mut rest: &[u8] = ch;
        loop {
            let (n, out, e) = to_u(c, rest, cap, last);
            all.extend(out);
            rest = &rest[n..];
            if e == U_BUFFER_OVERFLOW_ERROR { continue; }
            assert_eq!(e, 0);
            assert!(rest.is_empty());
            break;
        }
    }
    all
}

#[test]
fn open_and_names() {
    for (n, canon) in [
        ("UTF-8", "UTF-8"), ("utf8", "UTF-8"), ("utf_8", "UTF-8"), ("Unicode-1-1-UTF-8", "UTF-8"),
        ("windows-1252", "ibm-5348_P100-1997"), ("cp1252", "ibm-5348_P100-1997"), ("ibm-5348_P100-1997", "ibm-5348_P100-1997"),
        ("latin1", "ISO-8859-1"), ("iso-8859-1", "ISO-8859-1"), ("ISO_8859-1", "ISO-8859-1"), ("iso-8859-01", "ISO-8859-1"),
        ("ISO-8859-15", "ibm-923_P100-1998"), ("ascii", "US-ASCII"), ("US-ASCII", "US-ASCII"),
        ("UTF-16LE", "UTF-16LE"), ("utf16be", "UTF-16BE"), ("Shift_JIS", "ibm-943_P15A-2003"), ("sjis", "ibm-943_P15A-2003"),
        ("EUC-JP", "euc-jp-2007"), ("GBK", "windows-936-2000"), ("GB2312", "ibm-1383_P110-1999"), ("gb18030", "gb18030"),
        ("Big5", "windows-950-2000"), ("EUC-KR", "ibm-970_P110_P110-2006_U2"), ("KOI8-R", "ibm-878_P100-1996"),
        ("koi8-u", "ibm-1168_P100-2002"), ("cp437", "ibm-437_P100-1995"), ("windows-1251", "ibm-5347_P100-1998"),
        ("UTF-8,swaplfnl", "UTF-8"), ("windows-1256", "ibm-9448_X100-2005"),
    ] {
        let c = open(n);
        assert_eq!(name_of(c), canon, "{n}");
        unsafe { ucnv_close(c) };
    }
    // NULL / empty name = default (fake ACP is 28591)
    let mut e = 0;
    let c = unsafe { ucnv_open(null(), &mut e) };
    assert_eq!(name_of(c), "ISO-8859-1");
    unsafe { ucnv_close(c) };
    // unknown
    let mut e = 0;
    let c = unsafe { ucnv_open(b"no-such-thing\0".as_ptr() as *const CChar, &mut e) };
    assert!(c.is_null());
    assert_eq!(e, U_FILE_ACCESS_ERROR);
    // failure code in = untouched / NULL
    let mut e = U_ILLEGAL_ARGUMENT_ERROR;
    assert!(unsafe { ucnv_open(b"UTF-8\0".as_ptr() as *const CChar, &mut e) }.is_null());
}

#[test]
fn standard_names_and_max_size() {
    assert_eq!(std_name("utf8", "MIME").as_deref(), Some("UTF-8"));
    assert_eq!(std_name("utf8", "IANA").as_deref(), Some("UTF-8"));
    assert_eq!(std_name("ibm-5348_P100-1997", "IANA").as_deref(), Some("windows-1252"));
    assert_eq!(std_name("ibm-5348_P100-1997", "MIME"), None);
    assert_eq!(std_name("ibm-943_P15A-2003", "MIME").as_deref(), Some("Shift_JIS"));
    assert_eq!(std_name("latin1", "MIME").as_deref(), Some("ISO-8859-1"));
    assert_eq!(std_name("nonexistent", "MIME"), None);
    assert_eq!(std_name("utf8", "BOGUS"), None);
    for (n, m) in [("UTF-8", 3), ("UTF-16LE", 2), ("US-ASCII", 1), ("ISO-8859-1", 1), ("windows-1252", 1),
                   ("Shift_JIS", 2), ("EUC-JP", 3), ("GB18030", 4), ("Big5", 2)] {
        let c = open(n);
        assert_eq!(unsafe { ucnv_getMaxCharSize(c) }, m, "{n}");
        unsafe { ucnv_close(c) };
    }
}

#[test]
fn utf8_decode_split_at_every_boundary() {
    let s = "a\u{e9}\u{20ac}\u{1F600}z\u{10FFFF}";
    let bytes = s.as_bytes();
    let want: Vec<u16> = s.encode_utf16().collect();
    for cap in [64usize, 1, 2] {
        for split in 0..=bytes.len() {
            let c = open("UTF-8");
            let got = decode_chunks(c, &[&bytes[..split], &bytes[split..]], cap);
            assert_eq!(got, want, "split {split} cap {cap}");
            unsafe { ucnv_close(c) };
        }
    }
    // three-way splits too
    for a in 0..=bytes.len() {
        for b in a..=bytes.len() {
            let c = open("UTF-8");
            let got = decode_chunks(c, &[&bytes[..a], &bytes[a..b], &bytes[b..]], 64);
            assert_eq!(got, want, "split {a},{b}");
            unsafe { ucnv_close(c) };
        }
    }
}

#[test]
fn pending_counts_and_flush() {
    let c = open("UTF-8");
    let mut e = 0;
    let (n, out, err) = to_u(c, &[b'x', 0xE2, 0x82], 8, false);
    assert_eq!((n, out, err), (3, vec![b'x' as u16], 0));
    assert_eq!(unsafe { ucnv_toUCountPending(c, &mut e) }, 2);
    // reset drops pending state
    unsafe { ucnv_reset(c) };
    assert_eq!(unsafe { ucnv_toUCountPending(c, &mut e) }, 0);
    // completion
    to_u(c, &[0xE2, 0x82], 8, false);
    let (_, out, err) = to_u(c, &[0xAC], 8, false);
    assert_eq!((out, err), (vec![0x20AC], 0));
    assert_eq!(unsafe { ucnv_toUCountPending(c, &mut e) }, 0);
    // truncated sequence at flush: U_TRUNCATED_CHAR_FOUND, default callback substitutes U+FFFD and clears err
    to_u(c, &[0xE2, 0x82], 8, false);
    let (n, out, err) = to_u(c, &[], 8, true);
    assert_eq!((n, out, err), (0, vec![0xFFFD], 0));
    assert_eq!(unsafe { ucnv_toUCountPending(c, &mut e) }, 0);
    unsafe { ucnv_close(c) };

    // fromUnicode: trailing high surrogate pends when !flush
    let c = open("UTF-8");
    let (n, out, err) = from_u(c, &[0x41, 0xD83D], 16, false);
    assert_eq!((n, out, err), (2, vec![0x41], 0));
    assert_eq!(unsafe { ucnv_fromUCountPending(c, &mut e) }, 1);
    let (_, out, err) = from_u(c, &[0xDE00], 16, true);
    assert_eq!((out, err), (vec![0xF0, 0x9F, 0x98, 0x80], 0));
    assert_eq!(unsafe { ucnv_fromUCountPending(c, &mut e) }, 0);
    // flush with lone high surrogate: truncated -> default substitute (EF BF BD)
    from_u(c, &[0xD83D], 16, false);
    let (_, out, err) = from_u(c, &[], 16, true);
    assert_eq!((out, err), (vec![0xEF, 0xBF, 0xBD], 0));
    unsafe { ucnv_close(c) };
}

#[test]
fn dbcs_lead_byte_pending() {
    // fake-dbcs: 0x81..0x9F lead; 0x81 0x40 -> U+4E00 ; 0xA1 -> U+FF61
    let want = vec![0x41u16, 0x4E00, 0xFF61, 0x4E00 + 0x40 + 0x3E];
    let bytes = [0x41u8, 0x81, 0x40, 0xA1, 0x82, 0x7E];
    for split in 0..=bytes.len() {
        let c = open("fake-dbcs");
        let got = decode_chunks(c, &[&bytes[..split], &bytes[split..]], 64);
        assert_eq!(got, want, "split {split}");
        unsafe { ucnv_close(c) };
    }
    let c = open("fake-dbcs");
    let mut e = 0;
    let (n, out, err) = to_u(c, &[0x41, 0x81], 8, false);
    assert_eq!((n, out, err), (2, vec![0x41], 0));
    assert_eq!(unsafe { ucnv_toUCountPending(c, &mut e) }, 1);
    // flush with lone lead: truncated, substituted
    let (_, out, err) = to_u(c, &[], 8, true);
    assert_eq!((out, err), (vec![0xFFFD], 0));
    // encode back
    let (_, out, err) = from_u(c, &[0x41, 0x4E00, 0xFF61, 0x4E7E], 16, true);
    assert_eq!((out, err), (bytes[..].iter().copied().filter(|_| true).collect::<Vec<_>>(), 0));
    unsafe { ucnv_close(c) };
}

#[test]
fn from_unicode_overflow_and_resume() {
    let c = open("UTF-8");
    let src: Vec<u16> = "a\u{e9}\u{20ac}\u{1F600}".encode_utf16().collect();
    let want = "a\u{e9}\u{20ac}\u{1F600}".as_bytes().to_vec();
    for cap in 1..=5 {
        unsafe { ucnv_reset(c) };
        let mut rest: &[u16] = &src;
        let mut got = vec![];
        let mut guard = 0;
        loop {
            let (n, out, e) = from_u(c, rest, cap, true);
            got.extend(out);
            rest = &rest[n..];
            guard += 1;
            assert!(guard < 100);
            if e == U_BUFFER_OVERFLOW_ERROR { continue; }
            assert_eq!(e, 0);
            break;
        }
        assert!(rest.is_empty());
        assert_eq!(got, want, "cap {cap}");
    }
    unsafe { ucnv_close(c) };
    // same for toUnicode with a supplementary char and cap 1
    let c = open("UTF-8");
    let got = decode_chunks(c, &[&want], 1);
    assert_eq!(got, src);
    unsafe { ucnv_close(c) };
}

#[test]
fn native_codecs() {
    // Latin-1
    let c = open("ISO-8859-1");
    assert_eq!(to_u(c, &[0x41, 0xE9, 0xFF], 8, true), (3, vec![0x41, 0xE9, 0xFF], 0));
    // unmappable U+20AC -> default substitute 0x1A
    assert_eq!(from_u(c, &[0x41, 0x20AC], 8, true), (2, vec![0x41, 0x1A], 0));
    unsafe { ucnv_close(c) };
    // US-ASCII: byte >= 0x80 is illegal -> U+FFFD
    let c = open("US-ASCII");
    assert_eq!(to_u(c, &[0x41, 0x80], 8, true), (2, vec![0x41, 0xFFFD], 0));
    unsafe { ucnv_close(c) };
    // UTF-16LE/BE with split surrogate pair and odd byte splits
    for (n, bytes) in [("UTF-16LE", vec![0x41, 0, 0x3D, 0xD8, 0x00, 0xDE]), ("UTF-16BE", vec![0, 0x41, 0xD8, 0x3D, 0xDE, 0x00])] {
        for split in 0..=bytes.len() {
            let c = open(n);
            let got = decode_chunks(c, &[&bytes[..split], &bytes[split..]], 8);
            assert_eq!(got, vec![0x41, 0xD83D, 0xDE00], "{n} {split}");
            let (_, out, _) = from_u(c, &[0x41, 0xD83D, 0xDE00], 16, true);
            assert_eq!(out, bytes);
            unsafe { ucnv_close(c) };
        }
    }
    // invalid UTF-8: maximal subpart, one U+FFFD each
    let c = open("UTF-8");
    assert_eq!(to_u(c, &[0xFF, 0xE2, 0x82, 0x41, 0xC0, 0xED, 0xA0, 0x80], 16, true).1,
               vec![0xFFFD, 0xFFFD, 0x41, 0xFFFD, 0xFFFD, 0xFFFD, 0xFFFD]);
    unsafe { ucnv_close(c) };
}

// ------------------------------------------------------------ callbacks

#[derive(Default)]
struct Rec {
    to: Vec<(i32, Vec<u8>, i32)>,        // reason, code units, err on entry
    from: Vec<(i32, Vec<u16>, i32, i32)>, // reason, code units, code point, err on entry
    reset: u32,
    close: u32,
}

unsafe extern "C" fn rec_to(ctx: *const c_void, args: *mut UConverterToUnicodeArgs, units: *const CChar, len: i32, reason: i32, err: *mut i32) {
    let r = &mut *(ctx as *mut Rec);
    assert_eq!((*args).size as usize, std::mem::size_of::<UConverterToUnicodeArgs>());
    match reason {
        UCNV_RESET => { r.reset += 1; return; }
        UCNV_CLOSE => { r.close += 1; return; }
        _ => {}
    }
    r.to.push((reason, std::slice::from_raw_parts(units as *const u8, len as usize).to_vec(), *err));
    *err = 0;
    let q = [b'?' as u16];
    ucnv_cbToUWriteUChars(args, q.as_ptr(), 1, 0, err);
}
unsafe extern "C" fn rec_from(ctx: *const c_void, args: *mut UConverterFromUnicodeArgs, units: *const u16, len: i32, cp: i32, reason: i32, err: *mut i32) {
    let r = &mut *(ctx as *mut Rec);
    match reason {
        UCNV_RESET => { r.reset += 1; return; }
        UCNV_CLOSE => { r.close += 1; return; }
        _ => {}
    }
    r.from.push((reason, std::slice::from_raw_parts(units, len as usize).to_vec(), cp, *err));
    *err = 0;
    let q = [b'?' as u16];
    let mut p = q.as_ptr();
    ucnv_cbFromUWriteUChars(args, &mut p, p.add(1), 0, err);
}
unsafe extern "C" fn stop_to(_: *const c_void, _: *mut UConverterToUnicodeArgs, _: *const CChar, _: i32, _: i32, _: *mut i32) {}
unsafe extern "C" fn stop_from(_: *const c_void, _: *mut UConverterFromUnicodeArgs, _: *const u16, _: i32, _: i32, _: i32, _: *mut i32) {}

fn install(c: *mut UConverter, r: &mut Rec) {
    let mut e = 0;
    unsafe {
        ucnv_setToUCallBack(c, Some(rec_to), r as *mut Rec as *const c_void, null_mut(), null_mut(), &mut e);
        ucnv_setFromUCallBack(c, Some(rec_from), r as *mut Rec as *const c_void, null_mut(), null_mut(), &mut e);
    }
    assert_eq!(e, 0);
}

#[test]
fn callback_reasons_and_output() {
    let mut r = Rec::default();
    // UTF-8: illegal byte -> ILLEGAL; truncated at flush -> ILLEGAL with err TRUNCATED on entry
    let c = open("UTF-8");
    install(c, &mut r);
    let (n, out, e) = to_u(c, &[b'a', 0xFF, b'b', 0xE2, 0x82], 16, true);
    assert_eq!((n, out, e), (5, vec![b'a' as u16, b'?' as u16, b'b' as u16, b'?' as u16], 0));
    assert_eq!(r.to, vec![(UCNV_ILLEGAL, vec![0xFF], U_ILLEGAL_CHAR_FOUND), (UCNV_ILLEGAL, vec![0xE2, 0x82], U_TRUNCATED_CHAR_FOUND)]);
    // lone surrogate in -> ILLEGAL, cb writes '?' via cbFromUWriteUChars landing in target
    let (n, out, e) = from_u(c, &[0x41, 0xDC00, 0x42], 16, true);
    assert_eq!((n, out, e), (3, b"A?B".to_vec(), 0));
    assert_eq!(r.from, vec![(UCNV_ILLEGAL, vec![0xDC00], 0xDC00, U_ILLEGAL_CHAR_FOUND)]);
    unsafe { ucnv_close(c) };
    assert_eq!((r.reset, r.close), (0, 2)); // CLOSE is sent for both directions

    // unassigned: fake-sbcs 0x81 undefined; U+1234 unmappable; supplementary reports 2 units
    let mut r = Rec::default();
    let c = open("fake-sbcs");
    install(c, &mut r);
    assert_eq!(to_u(c, &[0x80, 0x81, 0xFF], 8, true), (3, vec![0x100, b'?' as u16, 0x17F], 0));
    assert_eq!(r.to, vec![(UCNV_UNASSIGNED, vec![0x81], U_INVALID_CHAR_FOUND)]);
    assert_eq!(from_u(c, &[0x100, 0x1234, 0xD83D, 0xDE00, 0x41], 16, true), (5, vec![0x80, b'?', b'?', 0x41], 0));
    assert_eq!(r.from, vec![
        (UCNV_UNASSIGNED, vec![0x1234], 0x1234, U_INVALID_CHAR_FOUND),
        (UCNV_UNASSIGNED, vec![0xD83D, 0xDE00], 0x1F600, U_INVALID_CHAR_FOUND),
    ]);
    unsafe { ucnv_reset(c); ucnv_close(c) };
    assert_eq!((r.reset, r.close), (2, 2)); // one RESET per direction, one CLOSE per direction
}

#[test]
fn stop_callbacks_report_error_and_position() {
    let c = open("UTF-8");
    let mut e = 0;
    unsafe {
        ucnv_setToUCallBack(c, Some(stop_to), null(), null_mut(), null_mut(), &mut e);
        ucnv_setFromUCallBack(c, Some(stop_from), null(), null_mut(), null_mut(), &mut e);
    }
    let (n, out, e) = to_u(c, &[b'a', 0xFF, b'b'], 8, true);
    assert_eq!((n, out, e), (2, vec![b'a' as u16], U_ILLEGAL_CHAR_FOUND));
    let (n, out, e) = to_u(c, &[b'b'], 8, true);
    assert_eq!((n, out, e), (1, vec![b'b' as u16], 0));
    let c2 = open("US-ASCII");
    let mut e = 0;
    unsafe { ucnv_setFromUCallBack(c2, Some(stop_from), null(), null_mut(), null_mut(), &mut e) };
    let (n, out, e) = from_u(c2, &[0x41, 0xE9, 0x42], 8, true);
    assert_eq!((n, out, e), (2, b"A".to_vec(), U_INVALID_CHAR_FOUND));
    unsafe { ucnv_close(c); ucnv_close(c2) };
}

#[test]
fn callback_set_get_roundtrip() {
    let c = open("UTF-8");
        let mut e = 0;
    let (mut a, mut x): (UToCb, *const c_void) = (stop_to, null());
    unsafe {
        ucnv_getToUCallBack(c, &mut a, &mut x);
        let default = a;
        let marker = 5usize as *const c_void;
        ucnv_setToUCallBack(c, Some(rec_to), marker, &mut a, &mut x, &mut e);
        assert_eq!(a as usize, default as usize);
        let (old_to, old_ctx) = (Some(a), x);
        let (mut a2, mut x2): (UToCb, *const c_void) = (stop_to, null());
        ucnv_getToUCallBack(c, &mut a2, &mut x2);
        assert_eq!((a2 as usize, x2), (rec_to as usize, marker));
        // restoring the old pair makes substitute behaviour return
        ucnv_setToUCallBack(c, old_to, old_ctx, null_mut(), null_mut(), &mut e);
        assert_eq!(to_u(c, &[0xFF], 4, true).1, vec![0xFFFD]);
        // NULL action is an argument error
        ucnv_setToUCallBack(c, None, null(), null_mut(), null_mut(), &mut e);
        assert_eq!(e, U_ILLEGAL_ARGUMENT_ERROR);
        ucnv_close(c);
    }
}

#[test]
fn offsets() {
    let c = open("UTF-8");
    let src = "a\u{e9}\u{1F600}b".as_bytes();
    let mut out = [0u16; 16];
    let mut offs = [-9i32; 16];
    let (mut t, mut s, mut e) = (out.as_mut_ptr(), src.as_ptr() as *const CChar, 0);
    unsafe {
        ucnv_toUnicode(c, &mut t, out.as_ptr().add(16), &mut s, (src.as_ptr() as *const CChar).add(src.len()), offs.as_mut_ptr(), 1, &mut e);
    }
    assert_eq!(&offs[..5], &[0, 1, 3, 3, 7]);
    // offsets across a split: bytes pending from the first call report -1
    let c2 = open("UTF-8");
    let mut offs = [-9i32; 4];
    let (mut t, mut s) = (out.as_mut_ptr(), src.as_ptr() as *const CChar);
    unsafe {
        ucnv_toUnicode(c2, &mut t, out.as_ptr().add(16), &mut s, (src.as_ptr() as *const CChar).add(2), offs.as_mut_ptr(), 0, &mut e);
        assert_eq!(t.offset_from(out.as_ptr()), 1);
        let (mut t, mut s) = (out.as_mut_ptr(), src.as_ptr().add(2) as *const CChar);
        ucnv_toUnicode(c2, &mut t, out.as_ptr().add(16), &mut s, (src.as_ptr() as *const CChar).add(4), offs.as_mut_ptr(), 0, &mut e);
        assert_eq!(offs[0], -1);
        ucnv_close(c); ucnv_close(c2);
    }
}

#[test]
fn bad_arguments() {
    let mut e = 0;
    unsafe {
        assert!(ucnv_getName(null(), &mut e).is_null());
        assert_eq!(e, U_ILLEGAL_ARGUMENT_ERROR);
        let mut e = 0;
        assert_eq!(ucnv_toUCountPending(null(), &mut e), -1);
        ucnv_close(null_mut());
        ucnv_reset(null_mut());
    }
}
