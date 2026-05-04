//! `CmControlVector` building blocks shared across crates that own
//! registry-config globals.
//!
//! `CmControlVector` is the dispatch table the registry uses to copy values
//! out of `HKLM\System\CurrentControlSet\Control` into kernel globals at
//! boot. The single concatenated vector lives in
//! `ntoskrnl-rust::config::cmdata::CmControlVector`, but the entries
//! themselves are owned by whichever subsystem owns the destination global
//! (see Phase B chop in `docs/engineering/cmdata-chopping-plan.md`).
//!
//! These types/macros live in `ntkrnl-sys` so any crate (pool, ntoskrnl-rust,
//! future kernel crates) can declare its own `FOO_CONTROL_VECTOR: SubVector<N>`
//! slice without needing to depend on ntoskrnl-rust.

use crate::ddk::_CM_SYSTEM_CONTROL_VECTOR;

/// Wrapper around a fixed-size `_CM_SYSTEM_CONTROL_VECTOR` array.
///
/// `_CM_SYSTEM_CONTROL_VECTOR` contains raw `*mut` pointers, so it isn't `Sync`.
/// All access goes through `CmGetSystemControlValues` which is single-threaded
/// at boot, so a manual `Sync` impl is sound.
///
/// Subsystem files declare:
/// ```ignore
/// pub static FOO_CONTROL_VECTOR: SubVector<N> = SubVector(unsafe { [...] });
/// ```
/// `cmdata.rs` then references the slice (`&FOO_CONTROL_VECTOR.0`) in its
/// `SLICES` array, and a `const fn` concatenates everything into the single
/// `CmControlVector` symbol that consumers see.
#[repr(transparent)]
pub struct SubVector<const N: usize>(pub [_CM_SYSTEM_CONTROL_VECTOR; N]);
unsafe impl<const N: usize> Sync for SubVector<N> {}

/// Wide-string literal helper. Returns `&'static [u16]` terminated with a NUL.
///
/// Used inside `cv!` to build the `KeyPath` / `ValueName` fields. Pattern lifted
/// from `config/cmconfig.rs`.
#[macro_export]
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}

/// `_CM_SYSTEM_CONTROL_VECTOR` entry constructor.
///
/// Three forms:
/// - 3-arg: `cv!(key, value, buf)` — NULL length+type
/// - 4-arg: `cv!(key, value, buf, len)` — NULL type
/// - 5-arg: `cv!(key, value, buf, len, ty)` — full
///
/// `$buf`/`$len`/`$ty` are raw pointer expressions
/// (e.g. `core::ptr::addr_of_mut!(crate::xxx::Foo)`).
#[macro_export]
macro_rules! cv {
    ($key:literal, $val:literal, $buf:expr) => {
        $crate::ddk::_CM_SYSTEM_CONTROL_VECTOR {
            KeyPath: $crate::w!($key).as_ptr() as $crate::ddk::PWCHAR,
            ValueName: $crate::w!($val).as_ptr() as $crate::ddk::PWCHAR,
            Buffer: $buf as $crate::ddk::PVOID,
            BufferLength: core::ptr::null_mut(),
            Type: core::ptr::null_mut(),
        }
    };
    ($key:literal, $val:literal, $buf:expr, $len:expr) => {
        $crate::ddk::_CM_SYSTEM_CONTROL_VECTOR {
            KeyPath: $crate::w!($key).as_ptr() as $crate::ddk::PWCHAR,
            ValueName: $crate::w!($val).as_ptr() as $crate::ddk::PWCHAR,
            Buffer: $buf as $crate::ddk::PVOID,
            BufferLength: $len as $crate::ddk::PULONG,
            Type: core::ptr::null_mut(),
        }
    };
    ($key:literal, $val:literal, $buf:expr, $len:expr, $ty:expr $(,)?) => {
        $crate::ddk::_CM_SYSTEM_CONTROL_VECTOR {
            KeyPath: $crate::w!($key).as_ptr() as $crate::ddk::PWCHAR,
            ValueName: $crate::w!($val).as_ptr() as $crate::ddk::PWCHAR,
            Buffer: $buf as $crate::ddk::PVOID,
            BufferLength: $len as $crate::ddk::PULONG,
            Type: $ty as $crate::ddk::PULONG,
        }
    };
}
