//! The `string/` family -- 61 C files, replaced one submodule at a time.
//!
//! House style follows `stat.rs`/`tocase.rs`: pinned struct layouts and Win32
//! externs transcribed from the header, C quirks kept and commented.
//!
//! ## Status (see the lead's yaml/report for the authoritative table)
//!
//! Done: [`plain`] (the arch-uniform and per-arch mechanical bodies --
//! `strdup`/`wcsdup`, `__strncnt`/`__wcsncnt`, `strnlen`/`wcsnlen`, the plain
//! `wcs*` family, and the six functions `string.cmake` builds from hand
//! asm on i386/amd64 and a C fallback on arm64: `memccpy`, `strcmp`,
//! `strncmp`, `_strnset`, `_strrev`, `_strset`), [`safe`] (the `_s` family
//! plus `memcpy_s`/`wmemcpy_s`/`wmemmove_s`), [`tok`] (`strtok`/`wcstok` and
//! their `_s` twins), [`icmp`] (`_memicmp`/`_stricmp`/`_strnicmp`/`_wcsicmp`/
//! `_wcsnicmp`, each with its `_l` and `__ascii_*` twin), [`case`] (`strlwr`/
//! `strupr`/`wcslwr`/`wcsupr`, each with its `_l`/`_s`/`_s_l` forms), [`coll`]
//! (`strcoll`/`stricoll`/`strncoll`/`strnicoll`/`wcscoll`/`wcsicoll`/
//! `wcsncoll`/`wcsnicoll` + `_l`, and `strxfrm`/`wcsxfrm` + `_l`).
//!
//! All 61 C files this batch owns are now ported (91 exports).
//!
//! ## Self-recursion
//!
//! This directory defines `strcmp`/`strncmp`/`wcscmp`/`wcsncmp`/`wcscpy`/
//! `wcscat`/`wcsncpy`/`wcsncat` and friends -- exactly the names LLVM's
//! loop-idiom pass rewrites *calls into*. Every loop here is either character-
//! at-a-time over a *non-recognized* shape (a `while` with an embedded
//! side-effecting comparison, not a bare copy-until-NUL) or, where the C
//! itself already calls a memory primitive (`memcpy_s`/`wmemcpy_s`/
//! `wmemmove_s`), forwarded to the *imported* `memcpy`/`memmove`, never to a
//! hand-rolled loop. Verified with `llvm-objdump -d` on all three targets:
//! see the lead's report for the disassembly evidence.
// `mod.rs` itself is only compiled under `feature = "string-ops"` (lib.rs);
// no need to re-gate each submodule.
pub mod case;
pub mod coll;
pub mod icmp;
pub mod plain;
pub mod safe;
pub mod tok;
