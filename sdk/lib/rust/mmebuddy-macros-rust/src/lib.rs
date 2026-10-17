//! `mmebuddy_debug.h` and `mmebuddy.h`'s statement macros, ported once for the four sound
//! crates that include them: mmebuddy, mment4, sndblst and wdmaud.drv.
//!
//! The header has two bodies, chosen by `#if DBG && !defined(NDEBUG)`, and the consumers do
//! not agree. mmebuddy (`add_compile_definitions(NDEBUG)` under DBG) and wdmaud.drv (`#define
//! NDEBUG` in every TU) compile the quiet one; mment4 and sndblst compile the DBG one, which
//! prints through `wsprintfW` + `OutputDebugStringW` and turns a failed `SND_ASSERT` into a
//! message box and `ExitProcess(1)`. So each flavour is exported under its own suffix and a
//! consumer's prelude picks one, e.g. `pub use mmebuddy_macros_rust::SND_TRACE_DBG as
//! SND_TRACE;`. A cfg/feature switch would not do: cargo unifies features across one build, so
//! the quiet crates would inherit the DBG flavour whenever both are compiled together.
//!
//! `rustconv port macros` cannot lower these: `__FILE__` expands to the probe's per-arch temp
//! path and `#condition` to the probe's placeholder, so every arch disagrees.
//!
//! As in win32macros-rust, the bodies spell core primitives and this crate's own externs; the
//! only names resolved at the call site are the caller's arguments.
#![cfg_attr(not(test), no_std)]

extern "C" {
    /// user32 `wsprintfW` (cdecl, variadic).
    pub fn wsprintfW(buf: *mut u16, fmt: *const u16, ...) -> i32;
}
extern "system" {
    pub fn OutputDebugStringW(s: *const u16);
    pub fn MessageBoxW(hwnd: *mut core::ffi::c_void, text: *const u16, caption: *const u16, ty: u32) -> i32;
    pub fn ExitProcess(code: u32) -> !;
}

/// mmsystem.h `MB_OK | MB_TASKMODAL`.
pub const MB_OK_TASKMODAL: u32 = 0x0000 | 0x2000;
/// mmsystem.h `MMSYSERR_INVALPARAM` = `MMSYSERR_BASE + 11`, `MMSYSERR_BASE` = 0.
pub const MMSYSERR_INVALPARAM: u32 = 11;

/// An ASCII `&str` as a NUL-terminated UTF-16 array, at compile time: `L"..."` for the
/// literals these macros build themselves (the callers' literals are already sealed wide).
pub const fn widen<const N: usize>(s: &str) -> [u16; N] {
    let b = s.as_bytes();
    let mut out = [0u16; N];
    let mut i = 0;
    while i < b.len() {
        assert!(b[i] < 0x80, "widen: ASCII only");
        out[i] = b[i] as u16;
        i += 1;
    }
    out
}

/// `L"..."` from an ASCII literal, as `*const u16`.
#[macro_export]
macro_rules! wlit {
    ($s:expr) => {{
        const S: &str = $s;
        const W: [u16; S.len() + 1] = $crate::widen::<{ S.len() + 1 }>(S);
        W.as_ptr()
    }};
}

/// A narrow NUL-terminated literal (`%hS` arguments), as `*const i8`.
#[macro_export]
macro_rules! nlit {
    ($s:expr) => {
        concat!($s, "\0").as_ptr() as *const i8
    };
}

// ================= quiet: `DBG && defined(NDEBUG)`, or !DBG ======================================

/// `#define SND_ERR(...) do {} while ( 0 )` — the arguments are not evaluated.
#[macro_export]
macro_rules! SND_ERR_NDEBUG {
    ($($arg:tt)*) => {{}};
}
/// `#define SND_WARN(...) do {} while ( 0 )`
#[macro_export]
macro_rules! SND_WARN_NDEBUG {
    ($($arg:tt)*) => {{}};
}
/// `#define SND_TRACE(...) do {} while ( 0 )`
#[macro_export]
macro_rules! SND_TRACE_NDEBUG {
    ($($arg:tt)*) => {{}};
}
/// `#define DUMP_WAVEHDR_QUEUE(sound_device_instance) do {} while ( 0 )`
#[macro_export]
macro_rules! DUMP_WAVEHDR_QUEUE_NDEBUG {
    ($($arg:tt)*) => {{}};
}
/// `#define SND_ASSERT(condition) do {(void)(condition);} while ( 0 )` — evaluated, discarded.
#[macro_export]
macro_rules! SND_ASSERT_NDEBUG {
    ($cond:expr $(,)?) => {{
        let _ = $cond;
    }};
}
/// mmebuddy.h `VALIDATE_MMSYS_PARAMETER`: return `MMSYSERR_INVALPARAM` from the CALLER when
/// the condition is false (SND_ERR is quiet here). The condition is C truthiness: the
/// translations pass pointers and integers as well as bools, so `$crate::truthy` decides.
#[macro_export]
macro_rules! VALIDATE_MMSYS_PARAMETER_NDEBUG {
    ($cond:expr $(,)?) => {
        if !$crate::Truthy::truthy($cond) {
            return $crate::MMSYSERR_INVALPARAM as _;
        }
    };
}

// ================= DBG: `DBG && !defined(NDEBUG)` ================================================

/// `{ WCHAR dbg_popup_msg[1024]; wsprintf(dbg_popup_msg, __VA_ARGS__); OutputDebugStringW(..); }`
#[macro_export]
macro_rules! SND_ERR_DBG {
    ($($arg:expr),+ $(,)?) => {{
        let mut dbg_popup_msg = [0u16; 1024];
        $crate::wsprintfW(dbg_popup_msg.as_mut_ptr(), $($arg),+);
        $crate::OutputDebugStringW(dbg_popup_msg.as_ptr());
    }};
}
/// Same body as SND_ERR.
#[macro_export]
macro_rules! SND_WARN_DBG {
    ($($arg:expr),+ $(,)?) => { $crate::SND_ERR_DBG!($($arg),+) };
}
/// Same body as SND_ERR.
#[macro_export]
macro_rules! SND_TRACE_DBG {
    ($($arg:expr),+ $(,)?) => { $crate::SND_ERR_DBG!($($arg),+) };
}
/// Walks `HeadWaveHeader` → `lpNext`, tracing each header's address, length and flags.
#[macro_export]
macro_rules! DUMP_WAVEHDR_QUEUE_DBG {
    ($inst:expr $(,)?) => {{
        let mut curr = (*$inst).HeadWaveHeader;
        $crate::SND_TRACE_DBG!($crate::wlit!("-- Current wave header list --\n"));
        while !curr.is_null() {
            $crate::SND_TRACE_DBG!(
                $crate::wlit!("%x | %d bytes | flags: %x\n"),
                curr,
                (*curr).dwBufferLength,
                (*curr).dwFlags
            );
            curr = (*curr).lpNext;
        }
    }};
}
/// On failure: trace `ASSERT FAILED: <condition> File <file> Line <line>`, pop the same up in
/// a task-modal message box titled `<file>(<line>)`, then `ExitProcess(1)`.
#[macro_export]
macro_rules! SND_ASSERT_DBG {
    ($cond:expr $(,)?) => {{
        if !$crate::Truthy::truthy($cond) {
            $crate::SND_ERR_DBG!(
                $crate::wlit!("ASSERT FAILED: %hS File %hS Line %u\n"),
                $crate::nlit!(stringify!($cond)),
                $crate::nlit!(file!()),
                line!()
            );
            let mut title = [0u16; 256];
            let mut msg = [0u16; 1024];
            $crate::wsprintfW(title.as_mut_ptr(), $crate::wlit!("%hS(%d)"), $crate::nlit!(file!()), line!());
            $crate::wsprintfW(msg.as_mut_ptr(), $crate::wlit!("ASSERT FAILED: %hS\n"), $crate::nlit!(stringify!($cond)));
            $crate::MessageBoxW(core::ptr::null_mut(), msg.as_ptr(), title.as_ptr(), $crate::MB_OK_TASKMODAL);
            $crate::ExitProcess(1);
        }
    }};
}
/// VALIDATE_MMSYS_PARAMETER with the DBG SND_ERR: logs the failed check, then returns
/// `MMSYSERR_INVALPARAM` from the caller.
#[macro_export]
macro_rules! VALIDATE_MMSYS_PARAMETER_DBG {
    ($cond:expr $(,)?) => {
        if !$crate::Truthy::truthy($cond) {
            $crate::SND_ERR_DBG!(
                $crate::wlit!("FAILED parameter check: %hS at File %S Line %lu\n"),
                $crate::nlit!(stringify!($cond)),
                $crate::nlit!(file!()),
                line!()
            );
            return $crate::MMSYSERR_INVALPARAM as _;
        }
    };
}

/// mmebuddy.h `#define AllocateStruct(thing) (thing*) AllocateMemory(sizeof(thing))`.
/// `AllocateMemory` is mmebuddy's own export (utility.c), resolved at the call site.
#[macro_export]
macro_rules! AllocateStruct {
    // The translations sometimes pass `sizeof(thing)` where the C passed `thing`.
    (core::mem::size_of::<$t:ty>() $(,)?) => {
        AllocateMemory(core::mem::size_of::<$t>() as u32) as *mut $t
    };
    ($t:ty $(,)?) => {
        AllocateMemory(core::mem::size_of::<$t>() as u32) as *mut $t
    };
}

// ================= sndtypes.h — device-type tests ================================================
//
// `SOUND_DEVICE_TYPE` is an enum (C `int` constants 1..=6) and callers pass it as a UCHAR
// `MMDEVICE_TYPE`, so C compares in `int` after promotion. `rustconv port macros` lowered these
// per crate with the comparison at whichever width the caller had (`u8` against the island's
// `u32` constant: E0308 in every crate); the shared spelling promotes like C and returns `int`.

/// sndtypes.h `MIN_SOUND_DEVICE_TYPE` / `MAX_SOUND_DEVICE_TYPE` and the members they bound.
pub const WAVE_IN_DEVICE_TYPE: i32 = 1;
pub const WAVE_OUT_DEVICE_TYPE: i32 = 2;
pub const MIDI_IN_DEVICE_TYPE: i32 = 3;
pub const MIDI_OUT_DEVICE_TYPE: i32 = 4;
pub const AUX_DEVICE_TYPE: i32 = 5;
pub const MIXER_DEVICE_TYPE: i32 = 6;
pub const MIN_SOUND_DEVICE_TYPE: i32 = 1;
pub const MAX_SOUND_DEVICE_TYPE: i32 = 6;

/// `( ( x >= MIN_SOUND_DEVICE_TYPE ) && ( x <= MAX_SOUND_DEVICE_TYPE ) )`
#[macro_export]
macro_rules! IS_VALID_SOUND_DEVICE_TYPE {
    ($x:expr $(,)?) => {
        ((($x) as i32 >= $crate::MIN_SOUND_DEVICE_TYPE && ($x) as i32 <= $crate::MAX_SOUND_DEVICE_TYPE) as i32)
    };
}
/// mmebuddy.h `#define IsValidSoundDeviceType IS_VALID_SOUND_DEVICE_TYPE`
#[macro_export]
macro_rules! IsValidSoundDeviceType {
    ($x:expr $(,)?) => { $crate::IS_VALID_SOUND_DEVICE_TYPE!($x) };
}
/// `( ( x == WAVE_IN_DEVICE_TYPE ) || ( x == WAVE_OUT_DEVICE_TYPE ) )`
#[macro_export]
macro_rules! IS_WAVE_DEVICE_TYPE {
    ($x:expr $(,)?) => {
        ((($x) as i32 == $crate::WAVE_IN_DEVICE_TYPE || ($x) as i32 == $crate::WAVE_OUT_DEVICE_TYPE) as i32)
    };
}
/// `( ( x == MIDI_IN_DEVICE_TYPE ) || ( x == MIDI_OUT_DEVICE_TYPE ) )`
#[macro_export]
macro_rules! IS_MIDI_DEVICE_TYPE {
    ($x:expr $(,)?) => {
        ((($x) as i32 == $crate::MIDI_IN_DEVICE_TYPE || ($x) as i32 == $crate::MIDI_OUT_DEVICE_TYPE) as i32)
    };
}
/// `( x == AUX_DEVICE_TYPE )`
#[macro_export]
macro_rules! IS_AUX_DEVICE_TYPE {
    ($x:expr $(,)?) => { ((($x) as i32 == $crate::AUX_DEVICE_TYPE) as i32) };
}
/// `( x == MIXER_DEVICE_TYPE )`
#[macro_export]
macro_rules! IS_MIXER_DEVICE_TYPE {
    ($x:expr $(,)?) => { ((($x) as i32 == $crate::MIXER_DEVICE_TYPE) as i32) };
}
/// mmebuddy.h `( x - MIN_SOUND_DEVICE_TYPE )`, an `int`.
#[macro_export]
macro_rules! SOUND_DEVICE_TYPE_TO_INDEX {
    ($x:expr $(,)?) => { (($x) as i32 - $crate::MIN_SOUND_DEVICE_TYPE) };
}
/// mmebuddy.h `( x + MIN_SOUND_DEVICE_TYPE )`, an `int`.
#[macro_export]
macro_rules! INDEX_TO_SOUND_DEVICE_TYPE {
    ($x:expr $(,)?) => { (($x) as i32 + $crate::MIN_SOUND_DEVICE_TYPE) };
}

// ================= mmebuddy.h — wide-string helpers ================================================
// Lowered per crate, `( string_length + 1 ) * sizeof(chartype)` multiplied the caller's length
// type by `usize` (mment4: `u64 * usize`). C computes it in size_t; so does this.

/// `#define StringLengthToBytes(chartype, string_length) ( ( string_length + 1 ) * sizeof(chartype) )`
#[macro_export]
macro_rules! StringLengthToBytes {
    ($t:ty, $len:expr $(,)?) => {
        ((($len) as usize + 1) * core::mem::size_of::<$t>())
    };
}
/// `#define AllocateWideString(string_length) (PWSTR) AllocateMemory(StringLengthToBytes(WCHAR, string_length))`
/// — `AllocateMemory(UINT Size)`, resolved at the call site like AllocateStruct.
#[macro_export]
macro_rules! AllocateWideString {
    ($len:expr $(,)?) => {
        AllocateMemory($crate::StringLengthToBytes!(u16, $len) as u32) as *mut u16
    };
}
/// `#define ZeroWideString(string) ZeroMemory(string, StringLengthToBytes(WCHAR, wcslen(string)))`
#[macro_export]
macro_rules! ZeroWideString {
    ($s:expr $(,)?) => {{
        let s = ($s) as *mut u16;
        core::ptr::write_bytes(s as *mut u8, 0, $crate::StringLengthToBytes!(u16, $crate::wide_len(s)));
    }};
}
/// `#define CopyWideString(dest, source) CopyMemory(dest, source, StringLengthToBytes(WCHAR, wcslen(source)))`
#[macro_export]
macro_rules! CopyWideString {
    ($d:expr, $src:expr $(,)?) => {{
        let src = ($src) as *const u16;
        core::ptr::copy_nonoverlapping(
            src as *const u8,
            ($d) as *mut u8,
            $crate::StringLengthToBytes!(u16, $crate::wide_len(src)),
        );
    }};
}

/// `wcslen`: the count of UTF-16 units before the terminating NUL.
///
/// # Safety
/// `s` must point at a NUL-terminated UTF-16 string.
pub unsafe fn wide_len(s: *const u16) -> usize {
    let mut n = 0;
    while *s.add(n) != 0 {
        n += 1;
    }
    n
}

/// C's `if (x)`: nonzero / non-null is true. The callers pass BOOLs, handles and counts.
pub trait Truthy {
    fn truthy(self) -> bool;
}
impl Truthy for bool {
    fn truthy(self) -> bool {
        self
    }
}
macro_rules! truthy_int {
    ($($t:ty),*) => { $(impl Truthy for $t { fn truthy(self) -> bool { self != 0 } })* };
}
truthy_int!(i8, u8, i16, u16, i32, u32, i64, u64, isize, usize);
impl<T> Truthy for *const T {
    fn truthy(self) -> bool {
        !self.is_null()
    }
}
/// A C function pointer (`SND_ASSERT( FunctionTable->Close )`) is an `Option<fn>` here.
impl<T> Truthy for Option<T> {
    fn truthy(self) -> bool {
        self.is_some()
    }
}
impl<T> Truthy for *mut T {
    fn truthy(self) -> bool {
        !self.is_null()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_type_tests_promote_like_c() {
        let t: u8 = 6;
        assert_eq!(IS_VALID_SOUND_DEVICE_TYPE!(t), 1);
        assert_eq!(IS_VALID_SOUND_DEVICE_TYPE!(0u8), 0);
        assert_eq!(IS_MIXER_DEVICE_TYPE!(t), 1);
        assert_eq!(SOUND_DEVICE_TYPE_TO_INDEX!(2u32), 1);
    }

    #[test]
    fn wide_string_helpers_count_the_nul() {
        let src: [u16; 4] = [b'a' as u16, b'b' as u16, b'c' as u16, 0];
        let mut dst = [0xFFFFu16; 5];
        unsafe {
            assert_eq!(wide_len(src.as_ptr()), 3);
            CopyWideString!(dst.as_mut_ptr(), src.as_ptr());
        }
        assert_eq!(dst, [b'a' as u16, b'b' as u16, b'c' as u16, 0, 0xFFFF]);
        unsafe { ZeroWideString!(dst.as_mut_ptr()) };
        assert_eq!(dst, [0, 0, 0, 0, 0xFFFF]);
        assert_eq!(StringLengthToBytes!(u16, 3u64), 8);
    }

    #[test]
    fn widen_terminates_and_matches_ascii() {
        const W: [u16; 4] = widen::<4>("abc");
        assert_eq!(W, [b'a' as u16, b'b' as u16, b'c' as u16, 0]);
    }

    fn validate(p: *const u8) -> u32 {
        VALIDATE_MMSYS_PARAMETER_NDEBUG!(p);
        0
    }

    #[test]
    fn validate_returns_invalparam_from_the_caller() {
        assert_eq!(validate(core::ptr::null()), 11);
        assert_eq!(validate(b"x".as_ptr()), 0);
    }
}
