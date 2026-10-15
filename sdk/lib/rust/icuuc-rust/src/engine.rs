//! Converter state and the streaming conversion loops.
use core::ffi::c_void;
use core::ptr;

use crate::codec::{self, Dec, Enc};
use crate::names::Entry;
use crate::plat::{Plat as P, Platform};
use crate::*;

pub const POOL: usize = 32;

/// Opaque to clients (they only hold `UConverter*`). Internal layout is ours.
pub struct UConverter {
    pub entry: &'static Entry,
    pub to_cb: UToCb,
    pub to_ctx: *const c_void,
    pub from_cb: UFromCb,
    pub from_ctx: *const c_void,
    // toUnicode: incomplete input bytes, and output units that did not fit.
    pub to_in: [u8; 4],
    pub to_in_len: u8,
    pub to_out: [u16; POOL],
    pub to_out_len: u8,
    pub to_out_pos: u8,
    // fromUnicode: a pending high surrogate, and output bytes that did not fit.
    pub from_in: [u16; 2],
    pub from_in_len: u8,
    pub from_out: [u8; POOL],
    pub from_out_len: u8,
    pub from_out_pos: u8,
}

fn addr<T>(p: *const T) -> usize { p as usize }
fn room16(t: *const u16, l: *const u16) -> usize { (addr(l) - addr(t)) / 2 }
fn room8(t: *const u8, l: *const u8) -> usize { addr(l) - addr(t) }

pub unsafe fn cstr_len(p: *const CChar) -> usize {
    let mut n = 0;
    // volatile: keeps LLVM from turning this into a libc strlen call
    while ptr::read_volatile(p.add(n)) != 0 { n += 1; }
    n
}

fn reset_state(c: &mut UConverter) {
    c.to_in_len = 0; c.to_out_len = 0; c.to_out_pos = 0;
    c.from_in_len = 0; c.from_out_len = 0; c.from_out_pos = 0;
}

pub unsafe fn open(name: *const CChar, err: *mut i32) -> *mut UConverter {
    if err.is_null() || *err > 0 { return ptr::null_mut(); }
    let entry = if name.is_null() || *name == 0 {
        let acp = P::acp();
        crate::names::find_by_cp(acp).or_else(|| crate::names::find(b"UTF-8"))
    } else {
        let b = core::slice::from_raw_parts(name as *const u8, cstr_len(name));
        crate::names::find(b)
    };
    let entry = match entry {
        Some(e) if e.kind != crate::names::Kind::Cp || P::cp_info(e.cp).is_some() => e,
        _ => { *err = U_FILE_ACCESS_ERROR; return ptr::null_mut(); }
    };
    let p = sys::alloc::<UConverter>();
    if p.is_null() { *err = U_MEMORY_ALLOCATION_ERROR; return ptr::null_mut(); }
    ptr::write(p, UConverter {
        entry,
        to_cb: default_to_u, to_ctx: ptr::null(),
        from_cb: default_from_u, from_ctx: ptr::null(),
        to_in: [0; 4], to_in_len: 0, to_out: [0; POOL], to_out_len: 0, to_out_pos: 0,
        from_in: [0; 2], from_in_len: 0, from_out: [0; POOL], from_out_len: 0, from_out_pos: 0,
    });
    p
}

/// ICU calls a non-default callback with UCNV_RESET / UCNV_CLOSE so it can drop its state.
unsafe fn notify(cnv: *mut UConverter, reason: i32) {
    if (*cnv).to_cb as usize != default_to_u as usize {
        let mut a = UConverterToUnicodeArgs {
            size: core::mem::size_of::<UConverterToUnicodeArgs>() as u16, flush: 1, converter: cnv,
            source: ptr::null(), source_limit: ptr::null(), target: ptr::null_mut(),
            target_limit: ptr::null(), offsets: ptr::null_mut(),
        };
        let mut e = 0;
        ((*cnv).to_cb)((*cnv).to_ctx, &mut a, ptr::null(), 0, reason, &mut e);
    }
    if (*cnv).from_cb as usize != default_from_u as usize {
        let mut a = UConverterFromUnicodeArgs {
            size: core::mem::size_of::<UConverterFromUnicodeArgs>() as u16, flush: 1, converter: cnv,
            source: ptr::null(), source_limit: ptr::null(), target: ptr::null_mut(),
            target_limit: ptr::null(), offsets: ptr::null_mut(),
        };
        let mut e = 0;
        ((*cnv).from_cb)((*cnv).from_ctx, &mut a, ptr::null(), 0, 0, reason, &mut e);
    }
}

pub unsafe fn reset(cnv: *mut UConverter) {
    if cnv.is_null() { return; }
    notify(cnv, UCNV_RESET);
    reset_state(&mut *cnv);
}

pub unsafe fn close(cnv: *mut UConverter) {
    if cnv.is_null() { return; }
    notify(cnv, UCNV_CLOSE);
    sys::free(cnv);
}

// ---------------------------------------------------------------- pending output

unsafe fn drain_units(cnv: *mut UConverter, tgt: &mut *mut u16, tlimit: *const u16, offs: &mut *mut i32) -> bool {
    while (*cnv).to_out_pos < (*cnv).to_out_len {
        if *tgt as *const u16 >= tlimit { return false; }
        **tgt = (*cnv).to_out[(*cnv).to_out_pos as usize];
        *tgt = (*tgt).add(1);
        if !offs.is_null() { **offs = -1; *offs = (*offs).add(1); }
        (*cnv).to_out_pos += 1;
    }
    (*cnv).to_out_pos = 0; (*cnv).to_out_len = 0;
    true
}

unsafe fn drain_bytes(cnv: *mut UConverter, tgt: &mut *mut u8, tlimit: *const u8, offs: &mut *mut i32) -> bool {
    while (*cnv).from_out_pos < (*cnv).from_out_len {
        if *tgt as *const u8 >= tlimit { return false; }
        **tgt = (*cnv).from_out[(*cnv).from_out_pos as usize];
        *tgt = (*tgt).add(1);
        if !offs.is_null() { **offs = -1; *offs = (*offs).add(1); }
        (*cnv).from_out_pos += 1;
    }
    (*cnv).from_out_pos = 0; (*cnv).from_out_len = 0;
    true
}

/// Append units that did not fit to the converter's overflow pool. false = pool full.
unsafe fn push_units(cnv: *mut UConverter, u: &[u16]) -> bool {
    let c = &mut *cnv;
    if c.to_out_len as usize + u.len() > POOL { return false; }
    for &x in u { c.to_out[c.to_out_len as usize] = x; c.to_out_len += 1; }
    true
}

unsafe fn push_bytes(cnv: *mut UConverter, b: &[u8]) -> bool {
    let c = &mut *cnv;
    if c.from_out_len as usize + b.len() > POOL { return false; }
    for &x in b { c.from_out[c.from_out_len as usize] = x; c.from_out_len += 1; }
    true
}

/// Shared by the callbacks: write bytes at args->target, spill the rest into the pool.
pub unsafe fn write_bytes(args: *mut UConverterFromUnicodeArgs, b: &[u8], offset_index: i32, err: *mut i32) {
    let a = &mut *args;
    let mut i = 0;
    while i < b.len() && (a.target as *const u8) < a.target_limit as *const u8 {
        *a.target = b[i] as CChar; a.target = a.target.add(1);
        if !a.offsets.is_null() { *a.offsets = offset_index; a.offsets = a.offsets.add(1); }
        i += 1;
    }
    if i < b.len() {
        *err = if push_bytes(a.converter, &b[i..]) { U_BUFFER_OVERFLOW_ERROR } else { U_INTERNAL_PROGRAM_ERROR };
    }
}

pub unsafe fn write_uchars(args: *mut UConverterToUnicodeArgs, s: *const u16, length: i32, offset_index: i32, err: *mut i32) {
    let a = &mut *args;
    let mut i = 0usize;
    let len = if length < 0 { 0 } else { length as usize };
    while i < len && (a.target as *const u16) < a.target_limit {
        *a.target = *s.add(i); a.target = a.target.add(1);
        if !a.offsets.is_null() { *a.offsets = offset_index; a.offsets = a.offsets.add(1); }
        i += 1;
    }
    if i < len {
        let rest = core::slice::from_raw_parts(s.add(i), len - i);
        *err = if push_units(a.converter, rest) { U_BUFFER_OVERFLOW_ERROR } else { U_INTERNAL_PROGRAM_ERROR };
    }
}

// ---------------------------------------------------------------- toUnicode

unsafe fn consume_in(cnv: *mut UConverter, src: &mut *const u8, used: usize, pl: usize) {
    if used <= pl {
        let c = &mut *cnv;
        c.to_in.copy_within(used..pl, 0);
        c.to_in_len = (pl - used) as u8;
    } else {
        *src = (*src).add(used - pl);
        (*cnv).to_in_len = 0;
    }
}

pub unsafe fn to_unicode(
    cnv: *mut UConverter, target: *mut *mut u16, tlimit: *const u16, source: *mut *const CChar,
    slimit: *const CChar, offsets: *mut i32, flush: i8, err: *mut i32,
) {
    if err.is_null() || *err > 0 { return; }
    if cnv.is_null() || target.is_null() || source.is_null() { *err = U_ILLEGAL_ARGUMENT_ERROR; return; }
    let mut tgt = *target;
    let mut src = *source as *const u8;
    let slim = slimit as *const u8;
    if tlimit < tgt as *const u16 || slim < src { *err = U_ILLEGAL_ARGUMENT_ERROR; return; }
    let src_start = src;
    let mut offs = offsets;
    let flush = flush != 0;

    if !drain_units(cnv, &mut tgt, tlimit, &mut offs) {
        *err = U_BUFFER_OVERFLOW_ERROR;
    } else {
        loop {
            let pl = (*cnv).to_in_len as usize;
            let rem = addr(slim) - addr(src);
            if pl + rem == 0 { break; }
            let mut view = [0u8; 4];
            let mut vl = 0;
            while vl < pl { view[vl] = (*cnv).to_in[vl]; vl += 1; }
            let mut k = 0;
            while vl < 4 && k < rem { view[vl] = *src.add(k); vl += 1; k += 1; }
            let base = if pl > 0 { -1 } else { (addr(src) - addr(src_start)) as i32 };
            match codec::decode((*cnv).entry, &view[..vl], flush) {
                Dec::More => {
                    (*cnv).to_in = view;
                    (*cnv).to_in_len = vl as u8;
                    src = slim;
                    break;
                }
                Dec::Char { used, units, n } => {
                    let room = room16(tgt, tlimit);
                    if room == 0 { *err = U_BUFFER_OVERFLOW_ERROR; break; }
                    consume_in(cnv, &mut src, used, pl);
                    let w = n.min(room);
                    for i in 0..w {
                        *tgt = units[i]; tgt = tgt.add(1);
                        if !offs.is_null() { *offs = base; offs = offs.add(1); }
                    }
                    if w < n {
                        push_units(cnv, &units[w..n]);
                        *err = U_BUFFER_OVERFLOW_ERROR;
                        break;
                    }
                }
                Dec::Bad { used, code } => {
                    let mut bad = [0u8; 4];
                    bad[..used].copy_from_slice(&view[..used]);
                    consume_in(cnv, &mut src, used, pl);
                    *err = code;
                    let reason = if code == U_INVALID_CHAR_FOUND { UCNV_UNASSIGNED } else { UCNV_ILLEGAL };
                    let mut args = UConverterToUnicodeArgs {
                        size: core::mem::size_of::<UConverterToUnicodeArgs>() as u16,
                        flush: flush as i8, converter: cnv,
                        source: src as *const CChar, source_limit: slimit,
                        target: tgt, target_limit: tlimit, offsets: offs,
                    };
                    let cb = (*cnv).to_cb;
                    cb((*cnv).to_ctx, &mut args, bad.as_ptr() as *const CChar, used as i32, reason, err);
                    tgt = args.target;
                    src = args.source as *const u8;
                    // Offsets written by a callback are relative to the start of the bad sequence.
                    let mut o = offs;
                    while !o.is_null() && o < args.offsets {
                        *o = if base < 0 { -1 } else { *o + base };
                        o = o.add(1);
                    }
                    offs = args.offsets;
                    if *err > 0 { break; }
                }
            }
        }
    }
    *target = tgt;
    *source = src as *const CChar;
}

// ---------------------------------------------------------------- fromUnicode

pub enum Sc {
    More,
    Cp { used: usize, cp: u32 },
    Bad { used: usize, cp: i32, code: i32 },
}

/// `v` non-empty. `flush` decides whether a trailing high surrogate is truncated or pending.
pub fn next_scalar(v: &[u16], flush: bool) -> Sc {
    let u = v[0];
    if (0xD800..0xDC00).contains(&u) {
        if v.len() < 2 {
            return if flush { Sc::Bad { used: 1, cp: u as i32, code: U_TRUNCATED_CHAR_FOUND } } else { Sc::More };
        }
        if (0xDC00..0xE000).contains(&v[1]) {
            return Sc::Cp { used: 2, cp: 0x10000 + (((u as u32) & 0x3FF) << 10) + (v[1] as u32 & 0x3FF) };
        }
        return Sc::Bad { used: 1, cp: u as i32, code: U_ILLEGAL_CHAR_FOUND };
    }
    if (0xDC00..0xE000).contains(&u) {
        return Sc::Bad { used: 1, cp: u as i32, code: U_ILLEGAL_CHAR_FOUND };
    }
    Sc::Cp { used: 1, cp: u as u32 }
}

unsafe fn consume_from(cnv: *mut UConverter, src: &mut *const u16, used: usize, pl: usize) {
    if pl > 0 {
        (*cnv).from_in_len = 0;
        *src = (*src).add(used - 1);
    } else {
        *src = (*src).add(used);
    }
}

pub unsafe fn from_unicode(
    cnv: *mut UConverter, target: *mut *mut u8, tlimit: *const CChar, source: *mut *const u16,
    slimit: *const u16, offsets: *mut i32, flush: i8, err: *mut i32,
) {
    if err.is_null() || *err > 0 { return; }
    if cnv.is_null() || target.is_null() || source.is_null() { *err = U_ILLEGAL_ARGUMENT_ERROR; return; }
    let mut tgt = *target;
    let tlim = tlimit as *const u8;
    let mut src = *source;
    if tlim < tgt as *const u8 || slimit < src { *err = U_ILLEGAL_ARGUMENT_ERROR; return; }
    let src_start = src;
    let mut offs = offsets;
    let flush = flush != 0;

    if !drain_bytes(cnv, &mut tgt, tlim, &mut offs) {
        *err = U_BUFFER_OVERFLOW_ERROR;
    } else {
        loop {
            let pl = (*cnv).from_in_len as usize;
            let rem = room16(src, slimit);
            if pl + rem == 0 { break; }
            let mut view = [0u16; 2];
            let mut vl = 0;
            if pl > 0 { view[0] = (*cnv).from_in[0]; vl = 1; }
            let mut k = 0;
            while vl < 2 && k < rem { view[vl] = *src.add(k); vl += 1; k += 1; }
            let base = if pl > 0 { -1 } else { room16(src_start, src) as i32 };
            let sc = match next_scalar(&view[..vl], flush) {
                Sc::Cp { used, cp } => match codec::encode((*cnv).entry, cp) {
                    Enc::Bytes { buf, n } => {
                        let room = room8(tgt, tlim);
                        if room == 0 { *err = U_BUFFER_OVERFLOW_ERROR; break; }
                        consume_from(cnv, &mut src, used, pl);
                        let w = n.min(room);
                        for i in 0..w {
                            *tgt = buf[i]; tgt = tgt.add(1);
                            if !offs.is_null() { *offs = base; offs = offs.add(1); }
                        }
                        if w < n {
                            push_bytes(cnv, &buf[w..n]);
                            *err = U_BUFFER_OVERFLOW_ERROR;
                            break;
                        }
                        continue;
                    }
                    Enc::Unmapped => Sc::Bad { used, cp: cp as i32, code: U_INVALID_CHAR_FOUND },
                },
                Sc::More => {
                    (*cnv).from_in[0] = view[0];
                    (*cnv).from_in_len = 1;
                    src = slimit;
                    break;
                }
                bad => bad,
            };
            if let Sc::Bad { used, cp, code } = sc {
                let mut bad = [0u16; 2];
                bad[..used].copy_from_slice(&view[..used]);
                consume_from(cnv, &mut src, used, pl);
                *err = code;
                let reason = if code == U_INVALID_CHAR_FOUND { UCNV_UNASSIGNED } else { UCNV_ILLEGAL };
                let mut args = UConverterFromUnicodeArgs {
                    size: core::mem::size_of::<UConverterFromUnicodeArgs>() as u16,
                    flush: flush as i8, converter: cnv,
                    source: src, source_limit: slimit,
                    target: tgt as *mut CChar, target_limit: tlimit as *const CChar, offsets: offs,
                };
                let cb = (*cnv).from_cb;
                cb((*cnv).from_ctx, &mut args, bad.as_ptr(), used as i32, cp, reason, err);
                tgt = args.target as *mut u8;
                src = args.source;
                let mut o = offs;
                while !o.is_null() && o < args.offsets {
                    *o = if base < 0 { -1 } else { *o + base };
                    o = o.add(1);
                }
                offs = args.offsets;
                if *err > 0 { break; }
            }
        }
    }
    *target = tgt;
    *source = src;
}

/// ucnv_cbFromUWriteUChars: convert UChars through this converter into args->target, without
/// invoking callbacks. Stops with an error code at the first unmappable/illegal unit.
pub unsafe fn cb_from_u_write_uchars(
    args: *mut UConverterFromUnicodeArgs, source: *mut *const u16, slimit: *const u16, offset_index: i32, err: *mut i32,
) {
    if err.is_null() || *err > 0 { return; }
    if args.is_null() || source.is_null() || (*args).converter.is_null() { *err = U_ILLEGAL_ARGUMENT_ERROR; return; }
    let entry = (*(*args).converter).entry;
    while *source < slimit {
        let rem = room16(*source, slimit).min(2);
        let v = [**source, if rem > 1 { *(*source).add(1) } else { 0 }];
        match next_scalar(&v[..rem], true) {
            Sc::Cp { used, cp } => match codec::encode(entry, cp) {
                Enc::Bytes { buf, n } => {
                    *source = (*source).add(used);
                    write_bytes(args, &buf[..n], offset_index, err);
                    if *err > 0 { return; }
                }
                Enc::Unmapped => { *err = U_INVALID_CHAR_FOUND; return; }
            },
            Sc::Bad { code, .. } => { *err = code; return; }
            Sc::More => { *err = U_TRUNCATED_CHAR_FOUND; return; }
        }
    }
}

// ---------------------------------------------------------------- built-in callbacks

/// ICU's UCNV_TO_U_CALLBACK_SUBSTITUTE: ignore RESET/CLOSE/CLONE, otherwise write U+FFFD.
pub unsafe extern "C" fn default_to_u(
    _ctx: *const c_void, args: *mut UConverterToUnicodeArgs, _units: *const CChar, _len: i32, reason: i32, err: *mut i32,
) {
    if reason > UCNV_IRREGULAR { return; }
    *err = U_ZERO_ERROR;
    let r = [0xFFFDu16];
    write_uchars(args, r.as_ptr(), 1, 0, err);
}

/// ICU's UCNV_FROM_U_CALLBACK_SUBSTITUTE: write the converter's substitution bytes.
pub unsafe extern "C" fn default_from_u(
    _ctx: *const c_void, args: *mut UConverterFromUnicodeArgs, _units: *const u16, _len: i32, _cp: i32, reason: i32, err: *mut i32,
) {
    if reason > UCNV_IRREGULAR { return; }
    *err = U_ZERO_ERROR;
    let (b, n) = codec::sub_bytes((*(*args).converter).entry);
    write_bytes(args, &b[..n], 0, err);
}
