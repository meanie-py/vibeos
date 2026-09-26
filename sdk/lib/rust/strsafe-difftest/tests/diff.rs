//! strsafe-rust's `StringCchPrintfW!` / `StringCbPrintfW!` / `StringCbPrintfExW!` against the
//! genuine strsafe.h, over every destination size 0..=MAX_CB bytes (odd sizes included, which is
//! where cb % sizeof(WCHAR) matters), every flag combination, NULL end/remaining out-params, a
//! NULL destination under IGNORE_NULLS, and formats that fit, fill exactly, and overflow.
//! Each run compares the HRESULT, EVERY byte of the destination (pre-filled with a sentinel,
//! with guard bytes beyond cbDest), the end pointer's offset, and the remaining count.
use strsafe_rust::{StringCbPrintfExW, StringCbPrintfW, StringCchPrintfW};

use strsafe_difftest::{HRESULT, ref_StringCbPrintfExW, ref_StringCbPrintfW, ref_StringCchPrintfW};

const MAX_CB: usize = 24;
const GUARD: usize = 8;

fn w(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

struct Out {
    hr: HRESULT,
    buf: Vec<u8>,
    end: Option<isize>,
    rem: Option<usize>,
}

fn run(f: impl FnOnce(*mut u16, *mut *mut u16, *mut usize) -> HRESULT, want_end: bool, want_rem: bool,
       null_dest: bool) -> Out {
    // u16-aligned backing store, viewed as bytes so an odd cbDest's last byte is observable.
    let mut store = vec![0xA5A5u16; (MAX_CB + GUARD) / 2 + 1];
    let base = store.as_mut_ptr();
    let mut end: *mut u16 = 0x1 as *mut u16;
    let mut rem: usize = 0xDEAD;
    let hr = f(
        if null_dest { core::ptr::null_mut() } else { base },
        if want_end { &mut end } else { core::ptr::null_mut() },
        if want_rem { &mut rem } else { core::ptr::null_mut() },
    );
    let buf = unsafe { core::slice::from_raw_parts(base as *const u8, MAX_CB + GUARD) }.to_vec();
    Out {
        hr,
        buf,
        end: want_end.then(|| if end.is_null() { -1 } else if end as usize == 1 { -2 }
                                else { unsafe { end.offset_from(base) } }),
        rem: want_rem.then_some(rem),
    }
}

fn same(what: &str, a: &Out, b: &Out) {
    assert_eq!(a.hr, b.hr, "{what}: HRESULT");
    assert_eq!(a.buf, b.buf, "{what}: destination bytes");
    assert_eq!(a.end, b.end, "{what}: end pointer");
    assert_eq!(a.rem, b.rem, "{what}: remaining");
}

fn formats() -> Vec<(Vec<u16>, i32, Vec<u16>)> {
    vec![
        (w(""), 0, w("")),
        (w("ab"), 0, w("")),
        (w("%d"), 7, w("")),
        (w("%d"), -12345, w("")),
        (w("%ls (%ls)"), 0, w("x")),        // desk's filter-string shape (arg order aside)
        (w("%d:%ls!"), 42, w("hello")),
        (w("%%%c%d"), 'Q' as i32, w("")),
    ]
}

fn main() {
    let mut n = 0u32;
    // The flag space StringCbPrintfExW accepts, the invalid-flag case, and fill bytes.
    let flag_bits = [0x100u32, 0x200, 0x400, 0x800, 0x1000];
    let mut flag_sets = vec![0x2000u32 /* not a valid flag */];
    for m in 0..(1u32 << flag_bits.len()) {
        let f: u32 = flag_bits.iter().enumerate().filter(|(i, _)| m & (1 << i) != 0).map(|(_, b)| b).sum();
        flag_sets.push(f);
        if f & (0x200 | 0x400) != 0 {
            flag_sets.push(f | 0x5A); // a non-zero fill byte
        }
    }

    for (fmt, a, s) in formats() {
        let (f, sp) = (fmt.as_ptr(), s.as_ptr());
        for cb in 0..=MAX_CB {
            let cch = cb / 2;
            same(&format!("Cch cch={cch} fmt={:?}", String::from_utf16_lossy(&fmt)),
                 &run(|d, _, _| unsafe { StringCchPrintfW!(d, cch, f, a, sp) }, false, false, false),
                 &run(|d, _, _| unsafe { ref_StringCchPrintfW(d, cch, f, a, sp) }, false, false, false));
            same(&format!("Cb cb={cb}"),
                 &run(|d, _, _| unsafe { StringCbPrintfW!(d, cb, f, a, sp) }, false, false, false),
                 &run(|d, _, _| unsafe { ref_StringCbPrintfW(d, cb, f, a, sp) }, false, false, false));
            n += 2;
            for &flags in &flag_sets {
                for (we, wr) in [(true, true), (false, true), (true, false), (false, false)] {
                    for null_dest in [false, true] {
                        if null_dest && flags & 0x100 == 0 {
                            continue; // a NULL dest is only defined under IGNORE_NULLS
                        }
                        let what = format!("CbEx cb={cb} flags={flags:#x} end={we} rem={wr} \
                                            nulldest={null_dest} fmt={:?}", String::from_utf16_lossy(&fmt));
                        let cbx = if null_dest { 0 } else { cb };
                        same(&what,
                             &run(|d, e, r| unsafe { StringCbPrintfExW!(d, cbx, e, r, flags, f, a, sp) }, we, wr, null_dest),
                             &run(|d, e, r| unsafe { ref_StringCbPrintfExW(d, cbx, e, r, flags, f, a, sp) }, we, wr, null_dest));
                        n += 1;
                    }
                }
            }
        }
    }
    // Over-large counts: STRSAFE_MAX_CCH is the boundary, checked before any write, so
    // both sides can be handed a count far beyond the real buffer.
    let fmt = w("x");
    for cch in [2147483647usize, 2147483648, usize::MAX] {
        let fp = fmt.as_ptr();
        same(&format!("Cch cch={cch}"),
             &run(|d, _, _| unsafe { StringCchPrintfW!(d, cch, fp, 0, fp) }, false, false, false),
             &run(|d, _, _| unsafe { ref_StringCchPrintfW(d, cch, fp, 0, fp) }, false, false, false));
        n += 1;
    }
    println!("strsafe-difftest: Rust == genuine strsafe.h");
    println!("\ndifftest result: ok. {n} passed; 0 failed");
}
