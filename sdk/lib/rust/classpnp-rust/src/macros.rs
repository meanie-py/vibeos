//! Crate-wide macros the rustconv translation emits in call form (NAME!(...)).
//! Loaded via `#[macro_use] mod macros;` at the top of lib.rs so the bare names
//! are in scope for every later module.

/// NT_ASSERT — debug assertion; a no-op in the Rust translation (the C asserts are
/// debug aids; failures elsewhere surface through the smoke harness).
#[macro_export]
macro_rules! NT_ASSERT { ($($t:tt)*) => {{}} }

/// FREE_POOL — the crate's pool-free helper (shims::FREE_POOL wraps ExFreePool).
#[macro_export]
macro_rules! FREE_POOL { ($p:expr) => { $crate::shims::FREE_POOL($p as PVOID) } }

/// FIELD_OFFSET(Type, Field) — byte offset of a field, like C's FIELD_OFFSET.
#[macro_export]
macro_rules! FIELD_OFFSET { ($t:ty, $f:tt) => { ::core::mem::offset_of!($t, $f) as ULONG } }

/// TEST_FLAG(Flags, Bit) — nonzero test, like C's TEST_FLAG.
#[macro_export]
macro_rules! TEST_FLAG { ($f:expr, $b:expr) => { (($f) & ($b)) != 0 } }

/// REVERSE_BYTES(Dest, Src) — reverse a 4-byte (ULONG) big/little-endian field in place.
#[macro_export]
macro_rules! REVERSE_BYTES {
    ($d:expr, $s:expr) => {{
        let d = ($d) as *mut u8;
        let s = ($s) as *const u8;
        *d.add(0) = *s.add(3);
        *d.add(1) = *s.add(2);
        *d.add(2) = *s.add(1);
        *d.add(3) = *s.add(0);
    }}
}

/// REVERSE_BYTES_QUAD(Dest, Src) — reverse an 8-byte (LARGE_INTEGER) field in place.
#[macro_export]
macro_rules! REVERSE_BYTES_QUAD {
    ($d:expr, $s:expr) => {{
        let d = ($d) as *mut u8;
        let s = ($s) as *const u8;
        let mut i = 0usize;
        while i < 8 { *d.add(i) = *s.add(7 - i); i += 1; }
    }}
}

/// SET_FLAG(Flags, Bit) / CLEAR_FLAG — in-place flag mutation, like the C macros.
#[macro_export]
macro_rules! SET_FLAG { ($f:expr, $b:expr) => { $f |= ($b) } }
#[macro_export]
macro_rules! CLEAR_FLAG { ($f:expr, $b:expr) => { $f &= !($b) } }

/// NT_ASSERTMSG(Msg, Expr) — debug assertion with message; no-op in the translation.
#[macro_export]
macro_rules! NT_ASSERTMSG { ($($t:tt)*) => {{}} }

/// REVERSE_BYTES_SHORT(Dest, Src) — reverse a 2-byte (USHORT) field in place.
#[macro_export]
macro_rules! REVERSE_BYTES_SHORT {
    ($d:expr, $s:expr) => {{
        let d = ($d) as *mut u8;
        let s = ($s) as *const u8;
        *d.add(0) = *s.add(1);
        *d.add(1) = *s.add(0);
    }}
}
