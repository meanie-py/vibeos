//! `_makepath`/`_splitpath`, `_access`, `_fullpath`, `remove`/`unlink`,
//! `rename`, `_mkdir`/`_rmdir`, `_chmod` -- all 26 exports, narrow and wide.
//! Replaces, in full, `sdk/lib/ucrt/filesystem/{makepath,splitpath,access,
//! waccess,fullpath,unlink,wunlink,chmod,wchmod,mkdir,wmkdir,rename,wrename,
//! rmdir,wrmdir}.cpp`.
//!
//! Every narrow export here is a thin conversion wrapper over its wide
//! sibling, the same shape as `_findfirst`/`_stat`: convert the path through
//! `__acrt_get_utf8_acp_compatibility_codepage`, call the wide body, done --
//! via `crate::findfile`'s `file_codepage`/`mbs_to_wcs`/`wcs_to_mbs`/
//! `os_errno`/`Buf` (the `filesystem` Cargo feature depends on `findfile`;
//! see `Cargo.toml`). `fullpath.rs` is the one file that cannot reuse all of
//! it as-is -- see its own module doc for exactly which piece is genuinely
//! different and why.
//!
//! `makepath`/`splitpath` are ported from UCRT's shared C++ templates
//! (`common_makepath_s<Character>`, `common_splitpath_internal<Character>`),
//! generic here over the same two character widths via the `PathChar`/
//! `SplitChar` traits in their own files.
//!
//! ## Details that are silent if wrong
//!
//! * **Two different "buffer too small" conventions, kept distinct.**
//!   `makepath`'s overflow resets the buffer to `""` and returns ERANGE
//!   THROUGH THE INVALID-PARAMETER HANDLER; `splitpath`'s overflow resets
//!   every requested component buffer and does `errno = ERANGE` DIRECTLY,
//!   with no handler call. See each file's own doc for the rest.
//! * **`splitpath`'s narrow form is MBCS-lead-byte aware** when scanning for
//!   the last `/`, `\` or `.`: it skips a DBCS trail byte via `_ismbblead` so
//!   a trail byte that happens to look like a separator is never misread as
//!   one. The wide form has no such concern.
//! * **`_access_s`/`_waccess_s` mode validation**: any bit outside `0b110`
//!   is EINVAL; a directory always passes (both read and write); a read-only
//!   FILE fails only when write access (bit 2) was requested, as EACCES.
//! * **`_fullpath` with an empty or NULL path returns the process's OWN cwd**
//!   (`_getcwd`/`_wgetcwd`, which itself allocates when the user buffer is
//!   NULL) rather than resolving `""` through `GetFullPathName`, and never
//!   grows a caller-supplied buffer -- only a NULL user buffer allocates,
//!   sized to the exact resolved length.
//! * **`_unlink`/`_mkdir`/`_chmod`/`_rmdir` on a NULL narrow path forward
//!   NULL to the wide form directly**; `rename` instead lets a NULL name
//!   convert to a NULL wide pointer through the SAME conversion path as a
//!   real string (no pre-check), then explicitly re-assigns `errno` from the
//!   conversion result before returning -1 -- redundant with what the
//!   conversion already set, but transcribed as Microsoft wrote it.
//! * **`_wchmod` only ever toggles `FILE_ATTRIBUTE_READONLY`**, via
//!   `_S_IWRITE`; every other mode bit is silently ignored.
//! * **`_wrename` allows a cross-volume move** (`MOVEFILE_COPY_ALLOWED`),
//!   which silently falls back to copy+delete instead of a plain rename when
//!   the two paths are on different volumes.

mod helpers;

mod access;
mod chmod;
mod fullpath;
mod makepath;
mod mkdir;
mod rename;
mod rmdir;
mod splitpath;
mod unlink;
