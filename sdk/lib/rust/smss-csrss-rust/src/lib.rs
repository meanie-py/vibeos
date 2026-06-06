//! smss-csrss-rust — Rust translation of the SMSS→CSRSS user-mode
//! spine (smss.exe, csrss.exe, csrsrv.dll, basesrv.dll, …).
//!
//! Per-binary cohabitation: each translated `.c` is commented out in
//! its CMakeLists and the `#[no_mangle]` symbol here takes over. FFI
//! surface is `ntdll-sys` (the user-mode Native API + SM/CSR/BASE
//! protocol). See docs/engineering/smss-csrss-translation-audit.md.
//!
//! NOT a standalone binary — an `rlib` linked into the cmake-built
//! native PEs (smss.exe / csrss.exe / csrsrv.dll …).

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code)]

use ntprintf::{format_into, Writer};

pub trait IntoSlot {
    fn into_slot(self) -> u64;
}

macro_rules! impl_intoslot_uint {
    ($($t:ty),*) => { $(
        impl IntoSlot for $t {
            #[inline(always)]
            fn into_slot(self) -> u64 { self as u64 }
        }
    )* };
}

macro_rules! impl_intoslot_sint {
    ($($t:ty),*) => { $(
        impl IntoSlot for $t {
            #[inline(always)]
            fn into_slot(self) -> u64 { self as i64 as u64 }
        }
    )* };
}

impl_intoslot_uint!(u8, u16, u32, u64, usize, bool);
impl_intoslot_sint!(i8, i16, i32, i64, isize);

impl<T> IntoSlot for *const T {
    #[inline(always)]
    fn into_slot(self) -> u64 {
        self as usize as u64
    }
}

impl<T> IntoSlot for *mut T {
    #[inline(always)]
    fn into_slot(self) -> u64 {
        self as usize as u64
    }
}

struct RawSink {
    dst: *mut u8,
    cap: usize,
    pos: usize,
    total: usize,
}

impl RawSink {
    fn new(dst: *mut u8, cap: usize) -> Self {
        Self {
            dst,
            cap,
            pos: 0,
            total: 0,
        }
    }
}

impl Writer for RawSink {
    fn write_bytes(&mut self, bytes: &[u8]) {
        self.total += bytes.len();
        let mut i = 0;
        while i < bytes.len() && self.pos < self.cap {
            unsafe {
                *self.dst.add(self.pos) = bytes[i];
            }
            self.pos += 1;
            i += 1;
        }
    }

    fn written(&self) -> usize {
        self.total
    }
}

#[inline]
fn strip_trailing_nul(fmt: &[u8]) -> &[u8] {
    match fmt.split_last() {
        Some((&0, rest)) => rest,
        _ => fmt,
    }
}

fn emit_dbgprint_no_varargs(bytes: &[u8]) {
    let mut start = 0usize;
    let mut i = 0usize;

    while i < bytes.len() {
        if bytes[i] == b'%' {
            emit_dbgprint_chunk(&bytes[start..i]);
            unsafe {
                crate::dbg_print!(b"%%\0");
            }
            i += 1;
            start = i;
        } else {
            i += 1;
        }
    }

    emit_dbgprint_chunk(&bytes[start..]);
}

fn emit_dbgprint_chunk(mut bytes: &[u8]) {
    while !bytes.is_empty() {
        let n = bytes.len().min(127);
        let mut chunk = [0u8; 128];
        let mut i = 0usize;
        while i < n {
            chunk[i] = bytes[i];
            i += 1;
        }
        unsafe {
            ntdll_sys::nt::DbgPrint(chunk.as_ptr() as *const i8);
        }
        bytes = &bytes[n..];
    }
}

pub fn dbg_print_impl(fmt: &[u8], slots: &[u64]) {
    let fmt = strip_trailing_nul(fmt);
    let mut buf = [0u8; 512];
    let mut sink = RawSink::new(buf.as_mut_ptr(), buf.len());
    format_into(&mut sink, fmt, slots);
    let len = sink.pos.min(buf.len());
    emit_dbgprint_no_varargs(&buf[..len]);
}

#[macro_export]
macro_rules! dbg_print {
    ($fmt:expr $(,)?) => {
        $crate::dbg_print_impl($fmt, &[])
    };
    ($fmt:expr, $($arg:expr),+ $(,)?) => {{
        let slots: &[u64] = &[$($crate::IntoSlot::into_slot($arg)),+];
        $crate::dbg_print_impl($fmt, slots)
    }};
}

// csrss.exe `_main` — clean rustconv translation after the P1a
// pipeline fixes. Linked into csrss.exe via the rtl-style cmake
// cargo→.a + --allow-multiple-definition swap-in (csrss.c kept until
// N green smokes). Gated OUT under smss_exe_only: csrss.rs and smss/smss.rs
// both define `#[no_mangle] _main`, which would collide in the smss.exe
// archive — smss.exe owns the entry there.
#[cfg(not(any(
    feature = "smss_exe_only",
    feature = "csrlib_only",
    feature = "smlib_only"
)))]
pub mod csrss;

// Phase 3 (smss.exe) + Phase 4 (basesrv.dll) translations sit under
// src/smss/ and src/basesrv/. Both modules compile in default
// features and provide reusable `#[no_mangle]` data globals + macro
// inlines, but their **cohabitation cmake flips are deferred**
// (task #18). The naive smss.exe flip bugchecked SMSS at
// SmpAcquirePrivilege / NtWaitForMultipleObjects; the naive
// basesrv.dll flip hung csrss inside ServerDllInitialization
// (suspected BaseInitializeStaticServerData runtime divergence).
// Keeping the source in tree for the next debug session.
//
// `csrss_main_only`: when set, csrss.exe consumes ONLY the `_main`
// shim from `csrss.rs`. The rest of the modules below are gated OUT
// so they don't produce a duplicate static copy of csrsrv's globals
// (notably `_CsrApiPort`) in csrss.exe. csrss.exe instead resolves
// `CsrServerInitialization` (and friends) via PE imports from
// csrsrv.dll, which means the *real* csrsrv.dll instance of the data
// statics gets written by `NtCreatePort` and read back by
// `CsrQueryApiPort` from any other DLL (winsrv, basesrv, …).
// `lean_features` matches any per-binary opt-in (csrss.exe / basesrv.dll
// / smss.exe). When ANY lean feature is on, csrsrv / csrlib / smlib are
// gated OUT — each lean binary defines only the entry points it owns
// and PE-imports everything else from csrsrv.dll.
// smss.exe owns smss/* — include under default AND smss_exe_only;
// gate OUT only for the csrss.exe / basesrv.dll / csrlib / smlib lean binaries.
#[cfg(any(
    not(any(
        feature = "csrss_main_only",
        feature = "basesrv_dll_only",
        feature = "smss_exe_only",
        feature = "csrlib_only",
        feature = "smlib_only"
    )),
    feature = "basesrv_dll_only"
))]
pub mod basesrv;
#[cfg(not(any(
    feature = "csrss_main_only",
    feature = "basesrv_dll_only",
    feature = "csrlib_only",
    feature = "smlib_only"
)))]
pub mod smss;

// Phase 1 — SM/CSR LPC protocol contract (translated 2026-05-16).
// csrlib = client CSR API + capture-buffer marshalling. Lands in
// ntdll.dll via the `csrlib_only` lean archive (client flip 2026-06-02);
// also compiled in csrsrv.dll's default-features kitchen-sink build.
// smlib = SM client (consumed by csrsrv.dll + smss.exe + ntdll_vista.dll;
// ntdll_vista.dll uses the `smlib_only` lean archive). Modules carry the
// #[no_mangle] protocol symbols.
//
// csrlib: default (no lean feature) OR the csrlib_only lean binary.
#[cfg(any(
    not(any(
        feature = "csrss_main_only",
        feature = "basesrv_dll_only",
        feature = "smss_exe_only",
        feature = "csrlib_only",
        feature = "smlib_only"
    )),
    feature = "csrlib_only"
))]
pub mod csrlib;
// smlib: default OR smss_exe_only (smss.exe) OR smlib_only (ntdll_vista.dll).
#[cfg(any(
    not(any(
        feature = "csrss_main_only",
        feature = "basesrv_dll_only",
        feature = "csrlib_only",
        feature = "smlib_only"
    )),
    feature = "smlib_only"
))]
pub mod smlib;

// Phase 2 — csrsrv.dll server runtime (translated 2026-05-16).
// 35 csrsrv.spec exports; cohabited into csrsrv.dll (the 7 .c
// commented out in subsystems/csr/csrsrv/CMakeLists.txt).
#[cfg(not(any(
    feature = "csrss_main_only",
    feature = "basesrv_dll_only",
    feature = "smss_exe_only",
    feature = "csrlib_only",
    feature = "smlib_only"
)))]
pub mod csrsrv;

// Standalone `#![no_std]` staticlib → needs its own panic handler +
// eh_personality (same role as kdcom-rust/hal-rust). USER-MODE: must
// NOT use privileged `hlt`/`wfe` (csrss.exe runs in ring 3) — abort
// the process via the Native API instead.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        crate::dbg_print!(b"\n!!! [SMSS-CSRSS] RUST PANIC HIT !!!\n\0");
        if let Some(loc) = info.location() {
            // Filenames in core are NUL-terminated UTF-8 string literals;
            // location().file() returns a &str whose underlying bytes are
            // typically a static C-string. Use the slice's ptr directly +
            // print line/column separately.
            let mut buf = [0u8; 256];
            let s = loc.file().as_bytes();
            let n = s.len().min(buf.len() - 1);
            buf[..n].copy_from_slice(&s[..n]);
            buf[n] = 0;
            crate::dbg_print!(
                b"PANIC LOC: %s:%u:%u\n\0",
                buf.as_ptr() as *const i8,
                loc.line(),
                loc.column(),
            );
        }
    }
    loop {
        core::hint::spin_loop();
    }
}

// `core`/`compiler_builtins` emit `.eh_frame` refs to
// `rust_eh_personality`; panic=abort never unwinds but the symbol
// must resolve. Same stub as kdcom-rust/hal-rust/kernel-rust.
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
