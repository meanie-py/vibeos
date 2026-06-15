//! k32_inlines.rs — `#[no_mangle]` bodies for the RTL FORCEINLINE helpers that
//! ntdll-sys *declares* as `extern "C"` but ntdll never actually exports (they
//! are FORCEINLINE in ndk/rtlfuncs.h). proc.c call sites resolve the names via
//! the `ntdll_sys::nt` glob (its extern decls); these definitions provide the
//! matching link symbols (`_RtlInitEmptyUnicodeString` / `_RtlInitEmptyAnsiString`
//! on i386 cdecl). This module is intentionally NOT re-exported through
//! `substrate` — only its symbols matter, and globbing it would make the names
//! ambiguous against the ntdll-sys decls.

use ntdll_sys::nt::*;

/// `RtlInitEmptyUnicodeString` (ndk/rtlfuncs.h FORCEINLINE).
#[no_mangle]
pub unsafe extern "C" fn RtlInitEmptyUnicodeString(
    UnicodeString: PUNICODE_STRING,
    Buffer: PWCHAR,
    BufferSize: USHORT,
) {
    (*UnicodeString).Length = 0;
    (*UnicodeString).MaximumLength = BufferSize;
    (*UnicodeString).Buffer = Buffer;
}

/// `RtlInitEmptyAnsiString` (ndk/rtlfuncs.h FORCEINLINE).
#[no_mangle]
pub unsafe extern "C" fn RtlInitEmptyAnsiString(
    AnsiString: PANSI_STRING,
    Buffer: PCHAR,
    BufferSize: USHORT,
) {
    (*AnsiString).Length = 0;
    (*AnsiString).MaximumLength = BufferSize;
    (*AnsiString).Buffer = Buffer;
}
