//! ucrt-rust — our Rust bodies for ucrtbase.dll's own surface, replacing the
//! vendored Microsoft UCRT in `sdk/lib/ucrt` one family at a time.
//!
//! Every symbol here deliberately displaces a genuine Microsoft UCRT body, so
//! every symbol here must be declared in `tools/ucrt-takeover.yaml` with the
//! oracle that grades it. See Cargo.toml for why that declaration is the whole
//! difference between this crate and the accident `crt-shared-rust` guards
//! against, and for why this crate must never depend on `msvcrt-crt-rust`.
//!
//! Progress: `python3 tools/ucrt_burndown.py --arch i386`.
#![no_std]
// LOAD-BEARING, as in msvcrt-crt-rust and crt-portable-rust: this crate now
// defines strcmp, wcscpy, wcscat, _memccpy and friends -- the very symbols
// LLVM's loop-idiom pass rewrites compare/copy/scan loops INTO. Without this,
// such a body can compile to a call to itself. `_strrev`'s NUL scan was
// measured becoming a call to strlen on all three targets.
#![no_builtins]
// Each module declares the ucrt C it still calls in its own `extern` block, so
// two modules can disagree about one C function's signature and both compile.
// One of them is then wrong at the ABI, silently. Make that an error.
#![deny(clashing_extern_declarations)]

#[cfg(feature = "bits")]
pub mod bits;

#[cfg(feature = "ismbbyte")]
pub mod ismbbyte;

#[cfg(feature = "ctype")]
pub mod ctype;

#[cfg(feature = "ismbc")]
pub mod ismbc;

#[cfg(feature = "tocase")]
pub mod tocase;

#[cfg(feature = "strtox")]
pub mod strtox;

#[cfg(feature = "atox")]
pub mod atox;

#[cfg(feature = "xtoa")]
pub mod xtoa;

#[cfg(feature = "findfile")]
pub mod findfile;

#[cfg(feature = "output")]
pub mod output;

#[cfg(feature = "strtod")]
pub mod big_integer;
#[cfg(feature = "strtod")]
pub mod strtod;
#[cfg(feature = "stat")]
pub mod stat;

// The locale takeover, in pieces that share `locale_abi` (see its table).
#[cfg(feature = "locale-data")]
pub mod locale_data;
#[cfg(feature = "locale-init")]
pub mod locale_init;
#[cfg(feature = "locale-init")]
pub mod nls_wrappers;
#[cfg(feature = "mbctype")]
pub mod mbctype;
#[cfg(feature = "qualified-locale")]
pub mod qualified_locale;
#[cfg(feature = "setlocale")]
pub mod setlocale;

#[cfg(feature = "exec")]
pub mod exec;
#[cfg(feature = "atof")]
pub mod atof;
#[cfg(feature = "ctime")]
pub mod ctime;
#[cfg(feature = "timeset")]
pub mod timeset;
#[cfg(feature = "argv-data")]
pub mod argv_data;
#[cfg(feature = "align")]
pub mod align;
#[cfg(feature = "invalid-parameter")]
pub mod invalid_parameter;
#[cfg(feature = "lowio-open")]
pub mod lowio_open;
#[cfg(feature = "mbstring")]
pub mod mbstring;
#[cfg(feature = "string-ops")]
pub mod string_ops;
#[cfg(feature = "time")]
pub mod time;
#[cfg(feature = "stdio")]
pub mod stdio;

// Named, not just depended on: a staticlib drops a dependency nothing references,
// and its #[no_mangle] bodies never reach libcrt_shared_cargo.a.
#[cfg(feature = "portable-string")]
extern crate crt_portable_rust;

// Always compiled: it defines no symbols, only UCRT's locale layout and the
// compile-time assertions pinning it. Gating it would mean the pins only hold
// when some other family happens to be on.
pub mod force_include;
pub mod locale;
pub mod locale_abi;
pub mod locale_update;
pub mod ptd;
pub mod ptd_host;
pub mod stdio_layout;

// Captured ground truth, not yet consumed by any body. Both are DATA ONLY --
// no `#[no_mangle]`, so they take over nothing and the cohab gate has nothing
// to say about them. They are here rather than in staging because they were
// GENERATED from public fact sources (the Unicode Character Database; published
// DBCS codepage definitions) and verified against Microsoft's tables while
// those still exist in `sdk/lib/ucrt`. After those files are deleted there is
// nothing left in the tree to check them against, so landing them is what makes
// the verification durable instead of a thing that happened once.
pub mod ctype_table;
pub mod mbctype_table;
