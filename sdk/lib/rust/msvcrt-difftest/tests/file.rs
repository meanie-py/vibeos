#![no_builtins]
// Waves 18a and 18b — grade `msvcrt-crt-rust/src/msvcrt/file.rs` against the GENUINE
// `dll/win32/msvcrt/file.c` it replaces.
//
// 18a took the narrow fd and stream layers. 18b takes what that left: the entire UTF-16
// door, the nine-export stat family, the 64-bit positioning twins, descriptor plumbing
// (`_dup2`, `_pipe`, the fd↔HANDLE pair, `_locking`, `_umask`), the wide stream
// characters, and `setvbuf`/`_filbuf`/`_flsbuf` — the buffer selection every other stream
// call goes through. Each section carries its own banner explaining what makes it
// separable and, where something is NOT separable, saying so instead of implying coverage
// it does not have.
//
// file.c's 189 exports went live in msvcrt.dll ahead of any host oracle: the flip was
// gated by a link, `scan_ud2`, and three boots. Those catch what they catch — a missing
// symbol, an unprovable index, a deadlock — and none of them can see a return value that
// is right by one, an errno set to the wrong code, or a byte written to the wrong offset.
// This is that instrument.
//
// # Why this suite does not look like wave16's
//
// Every other graded row here is a pure function: hand it bytes, compare bytes back. A
// stream call has THREE observable surfaces and two of them live outside the process:
//
//   1. the scalar effects  — return value, errno, doserrno, invalid-parameter handler;
//   2. the CRT's own state — the FILE's position/EOF/error flags, the fd's table entry;
//   3. THE FILE ON DISK.
//
// So each row runs over TWO real files seeded identically, one per side, and the compared
// image carries all three. The disk half is read back through `std::fs` — by the HOST,
// never back through the implementation that wrote it. A round-trip that only consults
// itself passes just as well over a completely fictional filesystem, which is the trap
// file_substrate.rs was written to name.
//
// # What makes the two sides separable at all
//
// `MSVCRT___pioinfo` (the fd table), `MSVCRT__iob` and `MSVCRT___badioinfo` are renamed
// into `ref_*` for the reference TU, so each side owns its own fd table and its own first
// three FILEs. Without that the two implementations would share one table and this suite
// would be grading an implementation against itself — MATCH for free. Likewise
// `msvcrt_set_errno`: one definition wrote through the reference's `_errno()` for BOTH
// sides, so a subject failure was invisible AND clobbered the slot the next reference call
// would be graded on.
//
// Both halves of that are now closed. The store has been per-side since wave 18a, and the
// MAPPING — which this comment used to record as a limit the suite could not see past — is
// no longer shared either: errno.c is reference TU #7, so the 54 stores these rows drive
// run through msvcrt's genuine 42-case mapper on the reference side and through Rust's on
// the subject's. fileimpl.c's transcription, which both sides used to call, is deleted.
//
// What IS shared, deliberately, is the Win32 layer underneath (fileimpl.c) — kernel32 is
// the environment, not the code under test. Both sides therefore allocate handles from one
// pool and, being driven in lockstep from a single thread, allocate the same fd numbers.
// That is why fd numbers are compared rather than merely their sign: "lowest free
// descriptor" is part of the contract.
//
// # The fd numbers are compared, and the row must be balanced
//
// A row that leaks an fd or a FILE on one side and not the other desynchronises every
// later row's fd numbering and turns the whole suite red at a point unrelated to the bug.
// Each row closes what it opened on BOTH sides, unconditionally, and `balanced()` asserts
// the two tables agree at the end of every row rather than trusting that.

// Force the SHIPPING crate's objects into the link. Without it rustc drops the unused
// rlib, and this suite is the worst possible place for that: `fopen`, `fread`, `fwrite`,
// `fgetc`, `remove`, `rename` are all ALSO glibc exports, so they bind silently to the
// host libc and every row grades glibc against msvcrt while reporting MATCH. Only the
// msvcrt-only names (`_read`, `_lseek`, `_filelength`, `msvcrt_init_io`) fail loudly, and
// they are the minority. `assert_shims_live()` proves the binding at run time as well,
// because a link that half-succeeds this way is not a hypothetical — it is what this file
// did on its first build.
extern crate msvcrt_crt_rust;

use std::ffi::{c_int, c_void, CString};
use std::path::PathBuf;

type FILEP = *mut c_void;

// ══════════════════════════════════════════════════════════════════════════════════════
// The two implementations
// ══════════════════════════════════════════════════════════════════════════════════════

extern "C" {
    // ── the SUBJECT: msvcrt-crt-rust's file.rs, under its shipping names ──────────────
    fn msvcrt_init_io();

    fn _open(path: *const u8, flags: c_int, ...) -> c_int;
    fn _close(fd: c_int) -> c_int;
    fn _read(fd: c_int, buf: *mut c_void, count: u32) -> c_int;
    fn _write(fd: c_int, buf: *const c_void, count: u32) -> c_int;
    fn _lseek(fd: c_int, off: i32, whence: c_int) -> i32;
    fn _lseeki64(fd: c_int, off: i64, whence: c_int) -> i64;
    fn _tell(fd: c_int) -> i32;
    fn _eof(fd: c_int) -> c_int;
    fn _filelength(fd: c_int) -> i32;
    fn _filelengthi64(fd: c_int) -> i64;
    fn _dup(od: c_int) -> c_int;
    fn _commit(fd: c_int) -> c_int;
    fn _chsize(fd: c_int, size: i32) -> c_int;
    fn _setmode(fd: c_int, mode: c_int) -> c_int;
    fn _isatty(fd: c_int) -> c_int;

    fn fopen(path: *const u8, mode: *const u8) -> FILEP;
    fn fopen_s(pfile: *mut FILEP, path: *const u8, mode: *const u8) -> c_int;
    fn fclose(f: FILEP) -> c_int;
    fn fread(ptr: *mut c_void, size: usize, n: usize, f: FILEP) -> usize;
    fn fwrite(ptr: *const c_void, size: usize, n: usize, f: FILEP) -> usize;
    fn fgetc(f: FILEP) -> c_int;
    fn fputc(c: c_int, f: FILEP) -> c_int;
    fn fgets(s: *mut u8, size: c_int, f: FILEP) -> *mut u8;
    fn fputs(s: *const u8, f: FILEP) -> c_int;
    fn ungetc(c: c_int, f: FILEP) -> c_int;
    fn fseek(f: FILEP, off: i32, whence: c_int) -> c_int;
    fn ftell(f: FILEP) -> i32;
    fn rewind(f: FILEP);
    fn feof(f: FILEP) -> c_int;
    fn ferror(f: FILEP) -> c_int;
    fn clearerr(f: FILEP);
    fn fflush(f: FILEP) -> c_int;
    fn _fileno(f: FILEP) -> c_int;

    fn remove(path: *const u8) -> c_int;
    fn rename(old: *const u8, new: *const u8) -> c_int;
    fn _unlink(path: *const u8) -> c_int;
    fn _access(path: *const u8, mode: c_int) -> c_int;

    // ── the REFERENCE: the genuine file.c, renamed ────────────────────────────────────
    fn ref_msvcrt_init_io();

    fn ref__open(path: *const u8, flags: c_int, ...) -> c_int;
    fn ref__close(fd: c_int) -> c_int;
    fn ref__read(fd: c_int, buf: *mut c_void, count: u32) -> c_int;
    fn ref__write(fd: c_int, buf: *const c_void, count: u32) -> c_int;
    fn ref__lseek(fd: c_int, off: i32, whence: c_int) -> i32;
    fn ref__lseeki64(fd: c_int, off: i64, whence: c_int) -> i64;
    fn ref__tell(fd: c_int) -> i32;
    fn ref__eof(fd: c_int) -> c_int;
    fn ref__filelength(fd: c_int) -> i32;
    fn ref__filelengthi64(fd: c_int) -> i64;
    fn ref__dup(od: c_int) -> c_int;
    fn ref__commit(fd: c_int) -> c_int;
    fn ref__chsize(fd: c_int, size: i32) -> c_int;
    fn ref__setmode(fd: c_int, mode: c_int) -> c_int;
    fn ref__isatty(fd: c_int) -> c_int;

    fn ref_fopen(path: *const u8, mode: *const u8) -> FILEP;
    fn ref_fopen_s(pfile: *mut FILEP, path: *const u8, mode: *const u8) -> c_int;
    fn ref_fclose(f: FILEP) -> c_int;
    fn ref_fread(ptr: *mut c_void, size: usize, n: usize, f: FILEP) -> usize;
    fn ref_fwrite(ptr: *const c_void, size: usize, n: usize, f: FILEP) -> usize;
    fn ref_fgetc(f: FILEP) -> c_int;
    fn ref_fputc(c: c_int, f: FILEP) -> c_int;
    fn ref_fgets(s: *mut u8, size: c_int, f: FILEP) -> *mut u8;
    fn ref_fputs(s: *const u8, f: FILEP) -> c_int;
    fn ref_ungetc(c: c_int, f: FILEP) -> c_int;
    fn ref_fseek(f: FILEP, off: i32, whence: c_int) -> c_int;
    fn ref_ftell(f: FILEP) -> i32;
    fn ref_rewind(f: FILEP);
    fn ref_feof(f: FILEP) -> c_int;
    fn ref_ferror(f: FILEP) -> c_int;
    fn ref_clearerr(f: FILEP);
    fn ref_fflush(f: FILEP) -> c_int;
    fn ref__fileno(f: FILEP) -> c_int;

    fn ref_remove(path: *const u8) -> c_int;
    fn ref_rename(old: *const u8, new: *const u8) -> c_int;
    fn ref__unlink(path: *const u8) -> c_int;
    fn ref__access(path: *const u8, mode: c_int) -> c_int;

    // ══ wave 18b ══════════════════════════════════════════════════════════════════════
    //
    // `wchar_t` and `wint_t` are BOTH `unsigned short` here — msvcrt's, and the reference
    // TU's too since it is compiled `-fshort-wchar`. Spelling them `u16` rather than
    // importing a typedef is deliberate: a host `wchar_t` is 32 bits, and a declaration
    // that picked the host's width would pass half a string and grade the wreckage.
    //
    // The stat buffers are `*mut c_void`. The four structs (`_stat`, `_stat64`,
    // `_stati64`, and the 32/64 hybrids) differ only in the width of three fields, the
    // rows compare them as BYTES including padding, and a Rust mirror of each would be a
    // second opinion about a layout `refstub/msvcrt_crt_layout.h` already pins.

    // ── the SUBJECT ───────────────────────────────────────────────────────────────────
    fn _wfopen(path: *const u16, mode: *const u16) -> FILEP;
    fn _wfopen_s(pfile: *mut FILEP, path: *const u16, mode: *const u16) -> c_int;
    fn _wfsopen(path: *const u16, mode: *const u16, share: c_int) -> FILEP;
    fn _wfreopen(path: *const u16, mode: *const u16, f: FILEP) -> FILEP;
    fn _wfreopen_s(pf: *mut FILEP, path: *const u16, mode: *const u16, f: FILEP) -> c_int;
    fn _wsopen_s(
        fd: *mut c_int,
        path: *const u16,
        oflags: c_int,
        shflags: c_int,
        pmode: c_int,
    ) -> c_int;
    fn _wcreat(path: *const u16, pmode: c_int) -> c_int;
    fn _wfdopen(fd: c_int, mode: *const u16) -> FILEP;
    fn _wremove(path: *const u16) -> c_int;
    fn _wunlink(path: *const u16) -> c_int;
    fn _wrename(old: *const u16, new: *const u16) -> c_int;
    fn _waccess(path: *const u16, mode: c_int) -> c_int;
    fn _waccess_s(path: *const u16, mode: c_int) -> c_int;
    fn _wchmod(path: *const u16, flags: c_int) -> c_int;

    fn _fsopen(path: *const u8, mode: *const u8, share: c_int) -> FILEP;
    fn freopen(path: *const u8, mode: *const u8, f: FILEP) -> FILEP;
    fn freopen_s(pf: *mut FILEP, path: *const u8, mode: *const u8, f: FILEP) -> c_int;
    fn _sopen_s(
        fd: *mut c_int,
        path: *const u8,
        oflags: c_int,
        shflags: c_int,
        pmode: c_int,
    ) -> c_int;
    fn _creat(path: *const u8, pmode: c_int) -> c_int;
    fn _chmod(path: *const u8, flags: c_int) -> c_int;
    fn _access_s(path: *const u8, mode: c_int) -> c_int;
    fn _fdopen(fd: c_int, mode: *const u8) -> FILEP;

    fn _stat(path: *const u8, buf: *mut c_void) -> c_int;
    fn _stat64(path: *const u8, buf: *mut c_void) -> c_int;
    fn _stati64(path: *const u8, buf: *mut c_void) -> c_int;
    fn _wstat(path: *const u16, buf: *mut c_void) -> c_int;
    fn _wstat64(path: *const u16, buf: *mut c_void) -> c_int;
    fn _wstati64(path: *const u16, buf: *mut c_void) -> c_int;
    fn _fstat(fd: c_int, buf: *mut c_void) -> c_int;
    fn _fstat64(fd: c_int, buf: *mut c_void) -> c_int;
    fn _fstati64(fd: c_int, buf: *mut c_void) -> c_int;

    fn _fseeki64(f: FILEP, off: i64, whence: c_int) -> c_int;
    fn _ftelli64(f: FILEP) -> i64;
    fn _telli64(fd: c_int) -> i64;
    fn fgetpos(f: FILEP, pos: *mut i64) -> c_int;
    fn fsetpos(f: FILEP, pos: *mut i64) -> c_int;

    fn _dup2(od: c_int, nd: c_int) -> c_int;
    fn _get_osfhandle(fd: c_int) -> isize;
    fn _open_osfhandle(h: isize, oflags: c_int) -> c_int;
    fn _pipe(pfds: *mut c_int, size: u32, textmode: c_int) -> c_int;
    fn _locking(fd: c_int, mode: c_int, nbytes: i32) -> c_int;
    fn _umask(mask: c_int) -> c_int;
    fn _chsize_s(fd: c_int, size: i64) -> c_int;

    fn fgetwc(f: FILEP) -> u16;
    fn getwc(f: FILEP) -> u16;
    fn fputwc(wc: u16, f: FILEP) -> u16;
    fn fgetws(s: *mut u16, size: c_int, f: FILEP) -> *mut u16;
    fn fputws(s: *const u16, f: FILEP) -> c_int;
    fn ungetwc(wc: u16, f: FILEP) -> u16;

    fn setvbuf(f: FILEP, buf: *mut u8, mode: c_int, size: usize) -> c_int;
    fn setbuf(f: FILEP, buf: *mut u8);
    fn _filbuf(f: FILEP) -> c_int;
    fn _flsbuf(c: c_int, f: FILEP) -> c_int;
    fn getc(f: FILEP) -> c_int;
    fn putc(c: c_int, f: FILEP) -> c_int;
    fn _getw(f: FILEP) -> c_int;
    fn _putw(v: c_int, f: FILEP) -> c_int;
    fn clearerr_s(f: FILEP) -> c_int;

    // ── the REFERENCE ─────────────────────────────────────────────────────────────────
    fn ref__wfopen(path: *const u16, mode: *const u16) -> FILEP;
    fn ref__wfopen_s(pfile: *mut FILEP, path: *const u16, mode: *const u16) -> c_int;
    fn ref__wfsopen(path: *const u16, mode: *const u16, share: c_int) -> FILEP;
    fn ref__wfreopen(path: *const u16, mode: *const u16, f: FILEP) -> FILEP;
    fn ref__wfreopen_s(
        pf: *mut FILEP,
        path: *const u16,
        mode: *const u16,
        f: FILEP,
    ) -> c_int;
    fn ref__wsopen_s(
        fd: *mut c_int,
        path: *const u16,
        oflags: c_int,
        shflags: c_int,
        pmode: c_int,
    ) -> c_int;
    fn ref__wcreat(path: *const u16, pmode: c_int) -> c_int;
    fn ref__wfdopen(fd: c_int, mode: *const u16) -> FILEP;
    fn ref__wremove(path: *const u16) -> c_int;
    fn ref__wunlink(path: *const u16) -> c_int;
    fn ref__wrename(old: *const u16, new: *const u16) -> c_int;
    fn ref__waccess(path: *const u16, mode: c_int) -> c_int;
    fn ref__waccess_s(path: *const u16, mode: c_int) -> c_int;
    fn ref__wchmod(path: *const u16, flags: c_int) -> c_int;

    fn ref__fsopen(path: *const u8, mode: *const u8, share: c_int) -> FILEP;
    fn ref_freopen(path: *const u8, mode: *const u8, f: FILEP) -> FILEP;
    fn ref_freopen_s(pf: *mut FILEP, path: *const u8, mode: *const u8, f: FILEP) -> c_int;
    fn ref__sopen_s(
        fd: *mut c_int,
        path: *const u8,
        oflags: c_int,
        shflags: c_int,
        pmode: c_int,
    ) -> c_int;
    fn ref__creat(path: *const u8, pmode: c_int) -> c_int;
    fn ref__chmod(path: *const u8, flags: c_int) -> c_int;
    fn ref__access_s(path: *const u8, mode: c_int) -> c_int;
    fn ref__fdopen(fd: c_int, mode: *const u8) -> FILEP;

    fn ref__stat(path: *const u8, buf: *mut c_void) -> c_int;
    fn ref__stat64(path: *const u8, buf: *mut c_void) -> c_int;
    fn ref__stati64(path: *const u8, buf: *mut c_void) -> c_int;
    fn ref__wstat(path: *const u16, buf: *mut c_void) -> c_int;
    fn ref__wstat64(path: *const u16, buf: *mut c_void) -> c_int;
    fn ref__wstati64(path: *const u16, buf: *mut c_void) -> c_int;
    fn ref__fstat(fd: c_int, buf: *mut c_void) -> c_int;
    fn ref__fstat64(fd: c_int, buf: *mut c_void) -> c_int;
    fn ref__fstati64(fd: c_int, buf: *mut c_void) -> c_int;

    fn ref__fseeki64(f: FILEP, off: i64, whence: c_int) -> c_int;
    fn ref__ftelli64(f: FILEP) -> i64;
    fn ref__telli64(fd: c_int) -> i64;
    fn ref_fgetpos(f: FILEP, pos: *mut i64) -> c_int;
    fn ref_fsetpos(f: FILEP, pos: *mut i64) -> c_int;

    fn ref__dup2(od: c_int, nd: c_int) -> c_int;
    fn ref__get_osfhandle(fd: c_int) -> isize;
    fn ref__open_osfhandle(h: isize, oflags: c_int) -> c_int;
    fn ref__pipe(pfds: *mut c_int, size: u32, textmode: c_int) -> c_int;
    fn ref__locking(fd: c_int, mode: c_int, nbytes: i32) -> c_int;
    fn ref__umask(mask: c_int) -> c_int;
    fn ref__chsize_s(fd: c_int, size: i64) -> c_int;

    fn ref_fgetwc(f: FILEP) -> u16;
    fn ref_getwc(f: FILEP) -> u16;
    fn ref_fputwc(wc: u16, f: FILEP) -> u16;
    fn ref_fgetws(s: *mut u16, size: c_int, f: FILEP) -> *mut u16;
    fn ref_fputws(s: *const u16, f: FILEP) -> c_int;
    fn ref_ungetwc(wc: u16, f: FILEP) -> u16;

    fn ref_setvbuf(f: FILEP, buf: *mut u8, mode: c_int, size: usize) -> c_int;
    fn ref_setbuf(f: FILEP, buf: *mut u8);
    fn ref__filbuf(f: FILEP) -> c_int;
    fn ref__flsbuf(c: c_int, f: FILEP) -> c_int;
    fn ref_getc(f: FILEP) -> c_int;
    fn ref_putc(c: c_int, f: FILEP) -> c_int;
    fn ref__getw(f: FILEP) -> c_int;
    fn ref__putw(v: c_int, f: FILEP) -> c_int;
    fn ref_clearerr_s(f: FILEP) -> c_int;

    // ── the two sides' observable state ───────────────────────────────────────────────
    fn subj_errno_get() -> c_int;
    fn subj_errno_set(v: c_int);
    fn subj_doserrno_get() -> u32;
    fn subj_doserrno_set(v: u32);
    fn subj_invalid_pmt_count() -> c_int;
    fn subj_invalid_pmt_reset();
    fn ref_errno_get() -> c_int;
    fn ref_errno_set(v: c_int);
    fn ref_invalid_pmt_count() -> c_int;
    fn ref_invalid_pmt_reset();

    /// The reference's doserrno slot, straight from fileimpl.c. There is no getter/setter
    /// pair for it the way there is for errno — errno.c's contract IS the pointer — so the
    /// harness reads and seeds through the pointer itself. It has to be observed on BOTH
    /// sides or not at all: half of a compared field is just a row that always diverges,
    /// which is what it was until this declaration existed.
    fn msvcrt_ref___doserrno() -> *mut u32;

    // ══ wave 18c ══════════════════════════════════════════════════════════════════════
    //
    // `__utimbuf32`/`__utimbuf64` are two `__time32_t`/`__time64_t` in that order and
    // nothing else, so the rows pass `*const i32`/`*const i64` pairs directly rather than
    // mirroring a two-field struct whose layout `msvcrt_crt_layout.h` already pins.
    fn _utime32(path: *const u8, t: *const i32) -> c_int;
    fn _utime64(path: *const u8, t: *const i64) -> c_int;
    fn _futime32(fd: c_int, t: *const i32) -> c_int;
    fn _futime64(fd: c_int, t: *const i64) -> c_int;
    fn _wutime32(path: *const u16, t: *const i32) -> c_int;
    fn _wutime64(path: *const u16, t: *const i64) -> c_int;

    fn _flushall() -> c_int;
    fn _fcloseall() -> c_int;
    fn _rmtmp() -> c_int;
    fn _setmaxstdio(newmax: c_int) -> c_int;
    fn _getmaxstdio() -> c_int;

    fn ref__utime32(path: *const u8, t: *const i32) -> c_int;
    fn ref__utime64(path: *const u8, t: *const i64) -> c_int;
    fn ref__futime32(fd: c_int, t: *const i32) -> c_int;
    fn ref__futime64(fd: c_int, t: *const i64) -> c_int;
    fn ref__wutime32(path: *const u16, t: *const i32) -> c_int;
    fn ref__wutime64(path: *const u16, t: *const i64) -> c_int;

    fn ref__flushall() -> c_int;
    fn ref__fcloseall() -> c_int;
    fn ref__rmtmp() -> c_int;
    fn ref__setmaxstdio(newmax: c_int) -> c_int;

    // ── the temp-name family ──────────────────────────────────────────────────────────
    fn tmpnam(s: *mut u8) -> *mut u8;
    fn tmpnam_s(s: *mut u8, size: usize) -> c_int;
    fn _wtmpnam(s: *mut u16) -> *mut u16;
    fn _wtmpnam_s(s: *mut u16, size: usize) -> c_int;
    fn _mktemp(pattern: *mut u8) -> *mut u8;
    fn _mktemp_s(pattern: *mut u8, size: usize) -> c_int;
    fn _wmktemp(pattern: *mut u16) -> *mut u16;
    fn _wmktemp_s(pattern: *mut u16, size: usize) -> c_int;
    fn tmpfile() -> FILEP;
    fn tmpfile_s(f: *mut FILEP) -> c_int;
    fn _tempnam(dir: *const u8, prefix: *const u8) -> *mut u8;
    fn _wtempnam(dir: *const u16, prefix: *const u16) -> *mut u16;

    fn ref_tmpnam(s: *mut u8) -> *mut u8;
    fn ref_tmpnam_s(s: *mut u8, size: usize) -> c_int;
    fn ref__wtmpnam(s: *mut u16) -> *mut u16;
    fn ref__wtmpnam_s(s: *mut u16, size: usize) -> c_int;
    fn ref__mktemp(pattern: *mut u8) -> *mut u8;
    fn ref__mktemp_s(pattern: *mut u8, size: usize) -> c_int;
    fn ref__wmktemp(pattern: *mut u16) -> *mut u16;
    fn ref__wmktemp_s(pattern: *mut u16, size: usize) -> c_int;
    fn ref_tmpfile() -> FILEP;
    fn ref_tmpfile_s(f: *mut FILEP) -> c_int;
    fn ref__tempnam(dir: *const u8, prefix: *const u8) -> *mut u8;
    fn ref__wtempnam(dir: *const u16, prefix: *const u16) -> *mut u16;
    fn ref__getmaxstdio() -> c_int;

    // ── wave 18c part 3: the std-stream aliases and the FILE lock ─────────────────────
    //
    // `wint_t` is `unsigned short` here, so the wide getters and putters are `u16` and
    // their EOF is `WEOF` (0xFFFF), not -1. Declaring them `c_int` would make every
    // failure compare as 0x0000FFFF against a sign-extended -1 and pass by accident.
    fn __iob_func() -> FILEP;
    fn getchar() -> c_int;
    fn _fgetchar() -> c_int;
    fn getwchar() -> u16;
    fn _fgetwchar() -> u16;
    fn gets(buf: *mut u8) -> *mut u8;
    fn _getws(buf: *mut u16) -> *mut u16;
    fn putchar(c: c_int) -> c_int;
    fn _fputchar(c: c_int) -> c_int;
    fn _fputwchar(wc: u16) -> u16;
    fn puts(s: *const u8) -> c_int;
    fn _putws(s: *const u16) -> c_int;
    fn _lock_file(f: FILEP);
    fn _unlock_file(f: FILEP);

    fn ref___iob_func() -> FILEP;
    fn ref_getchar() -> c_int;
    fn ref__fgetchar() -> c_int;
    fn ref_getwchar() -> u16;
    fn ref__fgetwchar() -> u16;
    fn ref_gets(buf: *mut u8) -> *mut u8;
    fn ref__getws(buf: *mut u16) -> *mut u16;
    fn ref_putchar(c: c_int) -> c_int;
    fn ref__fputchar(c: c_int) -> c_int;
    fn ref__fputwchar(wc: u16) -> u16;
    fn ref_puts(s: *const u8) -> c_int;
    fn ref__putws(s: *const u16) -> c_int;
    fn ref__lock_file(f: FILEP);
    fn ref__unlock_file(f: FILEP);

    /// The `dwShareMode` the last `CreateFileW` was asked for (fileimpl.c).
    ///
    /// ONE definition serving both sides, so it has to be read immediately after each
    /// side's call rather than once at the end. That is the whole trick: Windows share
    /// modes have no POSIX behaviour to observe, so without this the `_SH_DENY*` mapping
    /// — the only thing distinguishing `_fsopen` from `fopen` — would be ungradeable.
    fn ref_get_last_share_mode() -> u32;

    /// Point the three standard handles at the caller's host fds, for the side that is
    /// about to run `msvcrt_init_io()`. See the long note at the definition (fileimpl.c):
    /// without it both sides' `stdout` IS the harness's own fd 1 and the eleven exports
    /// that write there cannot be compared at all.
    fn ref_set_std_fds(in_fd: c_int, out_fd: c_int, err_fd: c_int);

    /// Which global lock slot the last `_lock` / `_unlock` took, or -1 (fileimpl.c). The
    /// only window onto `_lock_file`'s non-critical-section arm; same read-it-immediately
    /// discipline as `ref_get_last_share_mode`.
    fn ref_get_last_lock() -> c_int;
    fn ref_get_last_unlock() -> c_int;
    fn ref_reset_last_locks();

    // ── wave 18c part 3: the `v*printf` shells, against the FLAT instantiation ────────
    //
    // **The reference here is `fileflat_*`, never `ref_*`, and that is not a preference.**
    // Every one of these forwards `&valist` as a `va_list*`, which is coherent only where
    // a `va_list` IS a plain pointer. `ref_*` is compiled under the host's SysV
    // register-save-area model, where `*valist` reinterprets the decayed pointer variable
    // as the tag struct — a wild read, not a wrong answer. `fileflat_*` is the same file.c
    // compiled under refstub_printf's flat `char*` `<stdarg.h>`, which IS the Windows
    // x64 / aarch64 / i386 model, so it is both the callable instantiation and the
    // target-faithful one. Identical reasoning to `pfflat_` for wcs.c; see build.rs.
    //
    // `Va` is `*mut u8` on BOTH sides: msvcrt-crt-rust's `va_list` is `*mut u8` on every
    // target (prelude.rs), so the two consume the SAME hand-packed 8-byte-slot buffer.
    fn vfprintf(f: FILEP, fmt: *const u8, v: *mut u8) -> c_int;
    fn vfprintf_s(f: FILEP, fmt: *const u8, v: *mut u8) -> c_int;
    fn _vfprintf_l(f: FILEP, fmt: *const u8, loc: *mut c_void, v: *mut u8) -> c_int;
    fn _vfprintf_s_l(f: FILEP, fmt: *const u8, loc: *mut c_void, v: *mut u8) -> c_int;
    fn _vfprintf_p(f: FILEP, fmt: *const u8, v: *mut u8) -> c_int;
    fn _vfprintf_p_l(f: FILEP, fmt: *const u8, loc: *mut c_void, v: *mut u8) -> c_int;
    fn vfwprintf(f: FILEP, fmt: *const u16, v: *mut u8) -> c_int;
    fn vfwprintf_s(f: FILEP, fmt: *const u16, v: *mut u8) -> c_int;
    fn _vfwprintf_l(f: FILEP, fmt: *const u16, loc: *mut c_void, v: *mut u8) -> c_int;
    fn _vfwprintf_s_l(f: FILEP, fmt: *const u16, loc: *mut c_void, v: *mut u8) -> c_int;
    fn _vfwprintf_p(f: FILEP, fmt: *const u16, v: *mut u8) -> c_int;
    fn _vfwprintf_p_l(f: FILEP, fmt: *const u16, loc: *mut c_void, v: *mut u8) -> c_int;
    fn vprintf(fmt: *const u8, v: *mut u8) -> c_int;
    fn vprintf_s(fmt: *const u8, v: *mut u8) -> c_int;
    fn vwprintf(fmt: *const u16, v: *mut u8) -> c_int;
    fn vwprintf_s(fmt: *const u16, v: *mut u8) -> c_int;

    fn fileflat_msvcrt_init_io();
    fn fileflat___iob_func() -> FILEP;
    fn fileflat__sopen_s(
        fd: *mut c_int,
        path: *const u8,
        oflags: c_int,
        shflags: c_int,
        pmode: c_int,
    ) -> c_int;
    fn fileflat__fdopen(fd: c_int, mode: *const u8) -> FILEP;
    fn fileflat_fclose(f: FILEP) -> c_int;
    fn fileflat_fflush(f: FILEP) -> c_int;
    fn fileflat_fseek(f: FILEP, off: i32, whence: c_int) -> c_int;
    fn fileflat_ftell(f: FILEP) -> i32;
    fn fileflat_feof(f: FILEP) -> c_int;
    fn fileflat_ferror(f: FILEP) -> c_int;
    fn fileflat__fileno(f: FILEP) -> c_int;

    fn fileflat_vfprintf(f: FILEP, fmt: *const u8, v: *mut u8) -> c_int;
    fn fileflat_vfprintf_s(f: FILEP, fmt: *const u8, v: *mut u8) -> c_int;
    fn fileflat__vfprintf_l(f: FILEP, fmt: *const u8, loc: *mut c_void, v: *mut u8) -> c_int;
    fn fileflat__vfprintf_s_l(f: FILEP, fmt: *const u8, loc: *mut c_void, v: *mut u8) -> c_int;
    fn fileflat__vfprintf_p(f: FILEP, fmt: *const u8, v: *mut u8) -> c_int;
    fn fileflat__vfprintf_p_l(f: FILEP, fmt: *const u8, loc: *mut c_void, v: *mut u8) -> c_int;
    fn fileflat_vfwprintf(f: FILEP, fmt: *const u16, v: *mut u8) -> c_int;
    fn fileflat_vfwprintf_s(f: FILEP, fmt: *const u16, v: *mut u8) -> c_int;
    fn fileflat__vfwprintf_l(f: FILEP, fmt: *const u16, loc: *mut c_void, v: *mut u8) -> c_int;
    fn fileflat__vfwprintf_s_l(f: FILEP, fmt: *const u16, loc: *mut c_void, v: *mut u8) -> c_int;
    fn fileflat__vfwprintf_p(f: FILEP, fmt: *const u16, v: *mut u8) -> c_int;
    fn fileflat__vfwprintf_p_l(f: FILEP, fmt: *const u16, loc: *mut c_void, v: *mut u8) -> c_int;
    fn fileflat_vprintf(fmt: *const u8, v: *mut u8) -> c_int;
    fn fileflat_vprintf_s(fmt: *const u8, v: *mut u8) -> c_int;
    fn fileflat_vwprintf(fmt: *const u16, v: *mut u8) -> c_int;
    fn fileflat_vwprintf_s(fmt: *const u16, v: *mut u8) -> c_int;
}

// msvcrt's own <fcntl.h> / <io.h> values, derived by the pinned cross-compiler
// (refstub/msvcrt_crt_consts.h) — the same set file_substrate.rs uses. NOT the host's:
// msvcrt and glibc disagree on part of the space, and a literal here would be a second
// source of truth free to drift from the reference it grades.
const O_RDONLY: c_int = 0x0000;
const O_WRONLY: c_int = 0x0001;
const O_RDWR: c_int = 0x0002;
const O_APPEND: c_int = 0x0008;
const O_CREAT: c_int = 0x0100;
const O_TRUNC: c_int = 0x0200;
const O_TEXT: c_int = 0x4000;
const O_BINARY: c_int = 0x8000;

const SEEK_SET: c_int = 0;
const SEEK_CUR: c_int = 1;
const SEEK_END: c_int = 2;

/// `_access` modes. 0 = exists, 2 = write, 4 = read, 6 = both.
const ACCESS_MODES: [c_int; 4] = [0, 2, 4, 6];

// ── wave 18b's additions, from the same header and for the same reason ────────────────
const O_EXCL: c_int = 0x0400;
const O_NOINHERIT: c_int = 0x0080;

/// `_SH_DENYNO` / `_SH_DENYRD` / `_SH_DENYWR` / `_SH_DENYRW`.
///
/// Nothing on POSIX ENFORCES a share mode, so the substrate cannot make one observable by
/// behaving differently. What it can do is record the `dwShareMode` each side asked
/// `CreateFileW` for — see `ref_get_last_share_mode` — which turns the `_SH_DENY*`
/// mapping into something a row can compare. The rows grade the REQUEST, not its
/// enforcement, and say so.
const SHARE_MODES: [c_int; 4] = [0x0040, 0x0030, 0x0020, 0x0010];

const S_IWRITE: c_int = 0x0080;
const S_IREAD: c_int = 0x0100;

const LK_UNLCK: c_int = 0;
const LK_LOCK: c_int = 1;
const LK_RLCK: c_int = 3;

/// `setvbuf` modes. `_IONBF` is 4 here, not glibc's 2 — msvcrt aliases it to `MSVCRT__NOBUF`
/// at `_MSVCR_VER < 140`, which is the version this DLL is built at.
const IOFBF: c_int = 0;
const IOLBF: c_int = 0x0040;
const IONBF: c_int = 4;

/// `wint_t` is 16 bits in msvcrt, so its EOF is `0xFFFF` and not `-1`. Getting this wrong
/// is silent: the comparison simply never fires and every wide row grades as if the stream
/// never ended.
const WEOF: u16 = 0xFFFF;
const CEOF: c_int = -1;

// ══════════════════════════════════════════════════════════════════════════════════════
// The oracle
// ══════════════════════════════════════════════════════════════════════════════════════

/// errno is seeded with THIS, not 0, so a call that CLEARS errno is observable. msvcrt has
/// such calls — `_wsopen_dispatch` zeroes doserrno on its success path — and zeroing first
/// would make a zeroing write invisible.
const ERRNO_SEED: c_int = 0x5A5A;
const DOSERRNO_SEED: u32 = 0xA5A5_A5A5;

/// Filesystem work is orders of magnitude slower than the pure-function rows, so the
/// per-row count is chosen for coverage rather than for a big number: every row's corpus
/// is generated from a per-row seed and the `require` tags below are what actually assert
/// the interesting branches ran.
const ROW_ITERS: usize = 220;

#[derive(PartialEq, Debug)]
struct Effects {
    ret: i64,
    errno: c_int,
    doserrno: u32,
    pmt: c_int,
}

impl Effects {
    fn subject(f: impl FnOnce() -> i64) -> Effects {
        unsafe {
            subj_errno_set(ERRNO_SEED);
            subj_doserrno_set(DOSERRNO_SEED);
            subj_invalid_pmt_reset();
            let ret = f();
            Effects {
                ret,
                errno: subj_errno_get(),
                doserrno: subj_doserrno_get(),
                pmt: subj_invalid_pmt_count(),
            }
        }
    }
    fn reference(f: impl FnOnce() -> i64) -> Effects {
        unsafe {
            ref_errno_set(ERRNO_SEED);
            *msvcrt_ref___doserrno() = DOSERRNO_SEED;
            ref_invalid_pmt_reset();
            let ret = f();
            Effects {
                ret,
                errno: ref_errno_get(),
                doserrno: *msvcrt_ref___doserrno(),
                pmt: ref_invalid_pmt_count(),
            }
        }
    }
}

impl core::fmt::Display for Effects {
    fn fmt(&self, w: &mut core::fmt::Formatter) -> core::fmt::Result {
        let e = match self.errno {
            0 => "0".to_string(),
            ERRNO_SEED => "untouched".to_string(),
            2 => "ENOENT".to_string(),
            9 => "EBADF".to_string(),
            13 => "EACCES".to_string(),
            22 => "EINVAL".to_string(),
            24 => "EMFILE".to_string(),
            28 => "ENOSPC".to_string(),
            n => format!("errno{n}"),
        };
        let d = if self.doserrno == DOSERRNO_SEED {
            "untouched".to_string()
        } else {
            format!("{}", self.doserrno)
        };
        write!(
            w,
            "ret={} ({:#x}) errno={} doserrno={} handler={}",
            self.ret,
            self.ret,
            e,
            d,
            if self.pmt > 0 { format!("FIRED x{}", self.pmt) } else { "no".into() }
        )
    }
}

/// One side's complete footprint: the scalars, plus every byte the call could have
/// touched — the destination buffer with its guards, the CRT state it left behind, and
/// the file it wrote.
struct Obs {
    fx: Effects,
    img: Vec<u8>,
}

impl Obs {
    fn with(fx: Effects, img: Vec<u8>) -> Obs {
        Obs { fx, img }
    }
}

#[derive(PartialEq)]
enum V {
    Match,
    Diverge,
    Unexercised,
}

struct Res {
    name: String,
    v: V,
    note: String,
}

static mut RESULTS: Vec<Res> = Vec::new();
static mut CASES: u64 = 0;

fn record(name: String, v: V, note: String) {
    unsafe { (*core::ptr::addr_of_mut!(RESULTS)).push(Res { name, v, note }) }
}

/// Grade one function. `require` names the outcome classes the row MUST reach, counted
/// from the REFERENCE's answer — so it is a claim about msvcrt's contract rather than
/// about our implementation, and a row that reports MATCH having never run the branch it
/// exists to test fails as UNEXERCISED instead of passing quietly.
fn grade(
    name: &str,
    require: &[&'static str],
    iters: usize,
    mut f: impl FnMut(&mut Rng) -> (Obs, Obs, String, Vec<&'static str>),
) {
    let mut seed = 0x9E37_79B9_7F4A_7C15u64;
    for b in name.bytes() {
        seed = seed.wrapping_mul(0x100_0000_01B3).wrapping_add(b as u64);
    }
    let mut r = Rng(seed | 1);
    let mut seen: Vec<&'static str> = Vec::new();
    for _ in 0..iters {
        let (got, want, ctx, tags) = f(&mut r);
        unsafe { CASES += 1 };
        for t in tags {
            if !seen.contains(&t) {
                seen.push(t);
            }
        }
        if got.fx != want.fx || got.img != want.img {
            let mut note = format!("[{ctx}]\n        rust  = {}", got.fx);
            if got.img != want.img {
                note.push_str(&format!("\n                img={}", hex(&got.img)));
            }
            note.push_str(&format!("\n        msvcrt= {}", want.fx));
            if got.img != want.img {
                note.push_str(&format!("\n                img={}", hex(&want.img)));
                if let Some(i) = first_diff(&got.img, &want.img) {
                    note.push_str(&format!("\n        first differing byte: index {i}"));
                }
            }
            record(name.to_string(), V::Diverge, note);
            return;
        }
        // A leak on one side and not the other shifts every later fd number and turns a
        // row red a long way from its cause. Caught here, in the row that caused it.
        if let Err(e) = balanced() {
            record(name.to_string(), V::Diverge, format!("[{ctx}] {e}"));
            return;
        }
    }
    let missing: Vec<&&str> = require.iter().filter(|w| !seen.contains(w)).collect();
    if !missing.is_empty() {
        record(
            name.to_string(),
            V::Unexercised,
            format!(
                "the fuzz never reached {missing:?} — this MATCH is worthless, the branch \
                 never ran.\n        reached: {seen:?}"
            ),
        );
        return;
    }
    record(name.to_string(), V::Match, format!("{iters} cases; reached {seen:?}"));
}

/// Both sides must have the same number of descriptors in flight. Probed rather than
/// tracked: open one scratch fd on each side and compare the numbers the allocators hand
/// out. They agree iff the two tables are in the same state.
fn balanced() -> Result<(), String> {
    let (c, p) = tmp_path("balance");
    let _ = std::fs::write(&p, b"");
    unsafe {
        let a = _open(c.as_ptr() as *const u8, O_RDONLY | O_BINARY);
        let b = ref__open(c.as_ptr() as *const u8, O_RDONLY | O_BINARY);
        let out = if a != b {
            Err(format!(
                "fd tables have drifted apart — subject would next allocate {a}, reference {b}. \
                 A previous case leaked a descriptor on one side only."
            ))
        } else {
            Ok(())
        };
        if a >= 0 {
            _close(a);
        }
        if b >= 0 {
            ref__close(b);
        }
        let _ = std::fs::remove_file(&p);
        out
    }
}

fn hex(v: &[u8]) -> String {
    let mut s = String::new();
    for (i, b) in v.iter().enumerate() {
        if i > 0 && i % 32 == 0 {
            s.push('|');
        }
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn first_diff(a: &[u8], b: &[u8]) -> Option<usize> {
    for i in 0..a.len().min(b.len()) {
        if a[i] != b[i] {
            return Some(i);
        }
    }
    if a.len() != b.len() {
        Some(a.len().min(b.len()))
    } else {
        None
    }
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 17
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() as usize) % n.max(1)
    }
    /// `bytes(below(n))` and `bytes(k + below(n))`, as one call each. Spelled out because
    /// `r.bytes_upto(n)` borrows the generator twice and the obvious rewrite —
    /// hoisting a `let` at every site — buries the corpus shape in bookkeeping.
    fn bytes_upto(&mut self, n: usize) -> Vec<u8> {
        let k = self.below(n);
        self.bytes(k)
    }
    fn bytes_from(&mut self, least: usize, n: usize) -> Vec<u8> {
        let k = least + self.below(n);
        self.bytes(k)
    }
    fn bytes(&mut self, n: usize) -> Vec<u8> {
        // Deliberately includes 0x00, 0x0A and 0x1A: the NUL is what separates a
        // byte-count contract from a string one, the newline is the text-mode
        // translation, and 0x1A is msvcrt's text-mode EOF.
        (0..n)
            .map(|_| match self.below(8) {
                0 => 0x00,
                1 => 0x0A,
                2 => 0x0D,
                3 => 0x1A,
                _ => (self.next() % 256) as u8,
            })
            .collect()
    }
}

// ══════════════════════════════════════════════════════════════════════════════════════
// The two files a row runs over
// ══════════════════════════════════════════════════════════════════════════════════════

/// A path unique to this process. The CRT state under test is process-global.
fn tmp_path(tag: &str) -> (CString, PathBuf) {
    let p = std::env::temp_dir().join(format!("msvcrt-file-{}-{}", std::process::id(), tag));
    let _ = std::fs::remove_file(&p);
    (CString::new(p.to_str().unwrap()).unwrap(), p)
}

/// One file per side, seeded to identical bytes. The subject writes to `s`, the reference
/// to `r`, and the row compares what each left behind — read by `std::fs`, never back
/// through the implementation under test.
struct World {
    sc: CString,
    sp: PathBuf,
    rc: CString,
    rp: PathBuf,
}

impl World {
    fn new(tag: &str, seed: &[u8]) -> World {
        let (sc, sp) = tmp_path(&format!("{tag}-s"));
        let (rc, rp) = tmp_path(&format!("{tag}-r"));
        std::fs::write(&sp, seed).unwrap();
        std::fs::write(&rp, seed).unwrap();
        World { sc, sp, rc, rp }
    }
    /// No file at all on either side — for the failure rows.
    fn absent(tag: &str) -> World {
        let (sc, sp) = tmp_path(&format!("{tag}-s"));
        let (rc, rp) = tmp_path(&format!("{tag}-r"));
        World { sc, sp, rc, rp }
    }
    fn s(&self) -> *const u8 {
        self.sc.as_ptr() as *const u8
    }
    fn r(&self) -> *const u8 {
        self.rc.as_ptr() as *const u8
    }
    /// What is on disk now. `None` encodes "no such file", which is itself an answer —
    /// `remove` and `rename` rows turn on it.
    fn disk(&self) -> (Option<Vec<u8>>, Option<Vec<u8>>) {
        (std::fs::read(&self.sp).ok(), std::fs::read(&self.rp).ok())
    }
    /// The same two paths as msvcrt's 16-bit `wchar_t`, for the `_w*` rows.
    fn ws(&self) -> Vec<u16> {
        wpath(&self.sc)
    }
    fn wr(&self) -> Vec<u16> {
        wpath(&self.rc)
    }
    /// Whether each side's file is read-only, as the HOST sees it — the observation the
    /// `_chmod`/`_wchmod` rows turn on. Read through `std::fs` for the same reason the
    /// contents are: an implementation that reports its own permission bits back to itself
    /// passes over a filesystem where it never changed anything.
    fn readonly(&self) -> (Option<bool>, Option<bool>) {
        let ro = |p: &PathBuf| std::fs::metadata(p).ok().map(|m| m.permissions().readonly());
        (ro(&self.sp), ro(&self.rp))
    }
}

/// ONE file, read by BOTH sides.
///
/// The read-only rows — `_stat` and its eight relatives — compare a `struct _stat64` whose
/// `st_atime`/`st_mtime`/`st_ctime` come from the filesystem, and `_fstat64` additionally
/// reports `st_dev`/`st_rdev` as the fd number. Two files created microseconds apart carry
/// different timestamps, so a `World` would make every one of those rows diverge on fields
/// neither implementation computes. Nothing in these rows writes, so one file is the
/// honest fixture — and the fd numbers still agree, because `balanced()` keeps the two
/// tables in step.
struct Shared {
    c: CString,
    p: PathBuf,
}

impl Shared {
    fn new(tag: &str, seed: &[u8]) -> Shared {
        let (c, p) = tmp_path(tag);
        std::fs::write(&p, seed).unwrap();
        Shared { c, p }
    }
    fn absent(tag: &str) -> Shared {
        let (c, p) = tmp_path(tag);
        Shared { c, p }
    }
    fn n(&self) -> *const u8 {
        self.c.as_ptr() as *const u8
    }
    fn w(&self) -> Vec<u16> {
        wpath(&self.c)
    }
}

impl Drop for Shared {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.p);
    }
}

/// A path as msvcrt's 16-bit `wchar_t`, NUL-terminated.
///
/// The widening is byte-for-byte, which is exact for ASCII and wrong for anything else — a
/// real ANSI→UTF-16 conversion this is not. So it is ASSERTED rather than assumed: this
/// harness is entitled to require an ASCII temp directory, but it is not entitled to hand
/// the wide entry points a mangled path and grade the wreckage against the narrow rows.
fn wpath(c: &CString) -> Vec<u16> {
    let b = c.to_bytes();
    assert!(
        b.is_ascii(),
        "temp path {:?} is not ASCII — wpath()'s byte widening is not a real ANSI->UTF-16 \
         conversion, so the wide rows would run over a different path than the narrow ones. \
         Point TMPDIR at an ASCII path.",
        String::from_utf8_lossy(b)
    );
    b.iter().map(|&x| x as u16).chain(core::iter::once(0)).collect()
}

/// A narrow mode string (`b"rb\0"`) as its wide twin, NUL included.
fn wmode(m: &[u8]) -> Vec<u16> {
    m.iter().map(|&x| x as u16).collect()
}

impl Drop for World {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.sp);
        let _ = std::fs::remove_file(&self.rp);
    }
}

/// Fold a file's contents into the compared image. The `None` case gets a distinct marker
/// rather than an empty vector: "deleted" and "truncated to zero" are different answers
/// and a row over `remove` or `_chsize` must not confuse them.
fn push_disk(img: &mut Vec<u8>, d: Option<Vec<u8>>) {
    match d {
        None => img.push(0xDE),
        Some(b) => {
            img.push(0xFA);
            img.extend_from_slice(&(b.len() as u32).to_le_bytes());
            img.extend_from_slice(&b);
        }
    }
}

// ══════════════════════════════════════════════════════════════════════════════════════
// The two sides' standard streams
// ══════════════════════════════════════════════════════════════════════════════════════
//
// Eleven of file.c's exports take no FILE parameter — `putchar`, `puts`, `getchar`,
// `gets` and their kin all name `stdout` or `stdin` directly — so the ONLY way to compare
// what the two implementations did is for each side to own its standard streams. It does,
// as a consequence of the `_iob` rename wall: `stdout` is `&MSVCRT__iob[1]` on the subject
// and `&ref_MSVCRT__iob[1]` on the reference, two different FILEs.
//
// What they did NOT own is the fd underneath. `msvcrt_init_io` binds CRT fd 1 from
// `GetStdHandle(STD_OUTPUT_HANDLE)`, which the substrate answers with host fd 1 — so both
// sides' `putchar` wrote into the harness's own stdout, mixed with each other and with
// `println!`, and unreadable afterwards. `ref_set_std_fds` moves each side onto its own
// three files instead, once, before that side's init.

const SUBJ: char = 's';
const REF: char = 'r';
/// The THIRD world: file.c compiled a second time under the flat `va_list`, which is the
/// only instantiation the `v*printf` shells are callable in. It needs its own standard
/// streams for the same reason the other two do — `vprintf` names `stdout` and nothing
/// else. See the extern block's note, and build.rs.
const FLAT: char = 'f';

/// One side's stdin / stdout / stderr backing file.
fn std_path(side: char, which: &str) -> PathBuf {
    std::env::temp_dir().join(format!("msvcrt-file-{}-std{which}-{side}", std::process::id()))
}

/// Open the three files and hand their host fds to the substrate. The fds are LEAKED on
/// purpose: they live as long as the CRT's fd table does, which is the process.
unsafe fn bind_std_streams(side: char) {
    use std::os::fd::IntoRawFd;
    let mut fds = [-1 as c_int; 3];
    for (i, which) in ["in", "out", "err"].iter().enumerate() {
        let p = std_path(side, which);
        let f = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&p)
            .unwrap_or_else(|e| panic!("{}: {e}", p.display()));
        fds[i] = f.into_raw_fd();
    }
    ref_set_std_fds(fds[0], fds[1], fds[2]);
}

/// `sizeof(struct _iobuf)`, which is the stride of the `_iob` array. Asserted against the
/// pinned cross-compiler by refstub/msvcrt_crt_layout.h (`sizeof(FILE) == 48`), and
/// checked against both implementations at run time by `assert_shims_live`, which walks
/// this stride and demands `_fileno` answer 0, 1, 2.
const IOB_STRIDE: usize = core::mem::size_of::<Iobuf>();

/// msvcrt's `CRITICAL_SECTION`, and the `file_crit` a heap FILE really is.
///
/// Same status as `Iobuf` below, and declared for the same reason: `_lock_file`'s
/// critical-section arm leaves its mark in `RecursionCount`, and reaching it by byte
/// arithmetic would be a hand-rolled layout offset — the pattern `scan_magic_offsets`
/// forbids, and it caught this. Spelled as types instead, so the offsets come from the
/// declaration; every one of them is `_Static_assert`ed against the pinned cross-compiler
/// in refstub/msvcrt_crt_layout.h.
#[repr(C)]
struct Crit {
    debug_info: *mut c_void,
    lock_count: c_int,
    recursion_count: c_int,
    owning_thread: *mut c_void,
    lock_semaphore: *mut c_void,
    spin_count: usize,
}

/// file.c's `typedef struct { FILE file; CRITICAL_SECTION crit; } file_crit` — what
/// `file_get_cs` casts a FILE outside the `_iob` array to.
#[repr(C)]
struct FileCrit {
    file: Iobuf,
    crit: Crit,
}

const _: () = assert!(core::mem::size_of::<Crit>() == 40, "CRITICAL_SECTION size drifted");
const _: () = assert!(core::mem::size_of::<Iobuf>() == 48, "FILE size drifted");

/// `stdin`/`stdout`/`stderr` for the subject and the reference, through each side's own
/// `__iob_func` — which is itself one of the exports graded below.
unsafe fn s_std(i: usize) -> FILEP {
    (__iob_func() as *mut u8).add(i * IOB_STRIDE) as FILEP
}
unsafe fn r_std(i: usize) -> FILEP {
    (ref___iob_func() as *mut u8).add(i * IOB_STRIDE) as FILEP
}
unsafe fn f_std(i: usize) -> FILEP {
    (fileflat___iob_func() as *mut u8).add(i * IOB_STRIDE) as FILEP
}

/// msvcrt's `struct _iobuf`, as the harness sees it.
///
/// The layout is pinned by refstub/msvcrt_file_pre.h and asserted by
/// refstub/msvcrt_crt_layout.h against what the pinned cross-compiler measured, so this
/// is a view of the same bytes both sides manipulate rather than a second opinion about
/// them. It exists because the accessors alone cannot see the BUFFERING decision, and
/// buffering is a behaviour with no other window: `fwrite` of 18 bytes can buffer them or
/// write them straight through, return 9 either way, and leave identical bytes on disk —
/// while leaving the file position 9 apart. That is exactly what it did.
#[repr(C, packed(8))]
struct Iobuf {
    ptr: *mut u8,
    cnt: c_int,
    base: *mut u8,
    flag: c_int,
    file: c_int,
    charbuf: c_int,
    bufsiz: c_int,
    tmpfname: *mut u8,
}

/// The same fields, in words, for a failure note. A 32-byte image is a fine thing to
/// COMPARE and a poor thing to read, and every divergence in this suite is going to be
/// about buffering.
unsafe fn buffer_desc(f: FILEP) -> String {
    if f.is_null() {
        return "NULL".into();
    }
    let b = &*(f as *const Iobuf);
    let (ptr, base, cnt, flag, bufsiz) = (b.ptr, b.base, b.cnt, b.flag, b.bufsiz);
    let held = if base.is_null() || ptr.is_null() { 0i64 } else { ptr as i64 - base as i64 };
    let mut names = Vec::new();
    for (bit, name) in [
        (0x0001, "_IOREAD"),
        (0x0002, "_IOWRT"),
        (0x0008, "_IOMYBUF"),
        (0x0010, "_IOEOF"),
        (0x0020, "_IOERR"),
        (0x0040, "_IOSTRG"),
        (0x0080, "_IORW"),
        (0x0400, "_USERBUF"),
        (0x0002_0000, "_NOBUF"),
    ] {
        if flag & bit != 0 {
            names.push(name);
        }
    }
    format!(
        "held={held} cnt={cnt} bufsiz={bufsiz} base={} flag={flag:#x}[{}]",
        if base.is_null() { "NULL" } else { "set" },
        names.join("|")
    )
}

/// The buffering state, in the compared image. `_ptr` and `_base` are heap addresses and
/// differ by construction, so what is compared is their DIFFERENCE — how much is sitting
/// in the buffer — alongside the flags and the size.
unsafe fn buffer_state(f: FILEP) -> Vec<u8> {
    let b = &*(f as *const Iobuf);
    let (ptr, base, cnt, flag, bufsiz) = (b.ptr, b.base, b.cnt, b.flag, b.bufsiz);
    let held = if base.is_null() || ptr.is_null() { 0i64 } else { ptr as i64 - base as i64 };
    let mut v = Vec::new();
    v.extend_from_slice(&held.to_le_bytes());
    v.extend_from_slice(&cnt.to_le_bytes());
    v.extend_from_slice(&flag.to_le_bytes());
    v.extend_from_slice(&bufsiz.to_le_bytes());
    v.push(base.is_null() as u8);
    v
}

/// The FILE's observable state after the call, through each side's own accessors. Read
/// AFTER `Effects` has latched errno, because these are graded calls too and may set it.
unsafe fn stream_state_subject(f: FILEP) -> Vec<u8> {
    if f.is_null() {
        return vec![0xFF];
    }
    let mut v = vec![0x00];
    v.extend_from_slice(&ftell(f).to_le_bytes());
    v.push((feof(f) != 0) as u8);
    v.push((ferror(f) != 0) as u8);
    v.extend_from_slice(&_fileno(f).to_le_bytes());
    v.extend(buffer_state(f));
    v
}

unsafe fn stream_state_reference(f: FILEP) -> Vec<u8> {
    if f.is_null() {
        return vec![0xFF];
    }
    let mut v = vec![0x00];
    v.extend_from_slice(&ref_ftell(f).to_le_bytes());
    v.push((ref_feof(f) != 0) as u8);
    v.push((ref_ferror(f) != 0) as u8);
    v.extend_from_slice(&ref__fileno(f).to_le_bytes());
    v.extend(buffer_state(f));
    v
}

/// The fd's observable state, likewise.
unsafe fn fd_state_subject(fd: c_int) -> Vec<u8> {
    let mut v = Vec::new();
    v.extend_from_slice(&fd.to_le_bytes());
    if fd >= 0 {
        v.extend_from_slice(&_tell(fd).to_le_bytes());
        v.extend_from_slice(&_filelength(fd).to_le_bytes());
        v.push((_eof(fd) != 0) as u8);
    }
    v
}

unsafe fn fd_state_reference(fd: c_int) -> Vec<u8> {
    let mut v = Vec::new();
    v.extend_from_slice(&fd.to_le_bytes());
    if fd >= 0 {
        v.extend_from_slice(&ref__tell(fd).to_le_bytes());
        v.extend_from_slice(&ref__filelength(fd).to_le_bytes());
        v.push((ref__eof(fd) != 0) as u8);
    }
    v
}

// Guarded destination buffers: 8 bytes of poison on each side, compared with the payload,
// so a write past either end is a divergence rather than a silent corruption of the
// harness's own memory.
const GUARD: usize = 8;

struct Buf(Vec<u8>);

impl Buf {
    fn new(n: usize) -> Buf {
        Buf(vec![0xA7; n + 2 * GUARD])
    }
    fn p(&mut self) -> *mut u8 {
        unsafe { self.0.as_mut_ptr().add(GUARD) }
    }
    fn img(&self) -> Vec<u8> {
        self.0.clone()
    }
}

fn main() {
    unsafe {
        // Windows always has TMP set; this host does not, and the difference is not
        // cosmetic. `tmpfile()` calls `_tempnam(",", "t")`, which falls back to the literal
        // directory "," when TMP is absent — `GetTempFileNameW` then fails, `_wtempnam`
        // returns NULL, and `_tempnam` hands that straight to `astrdupw_utf8`. msvcrt's own
        // C does the same, with no guard between them:
        //
        //     retW = _wtempnam(dirW, prefixW);
        //     ret = astrdupw_utf8(retW);            /* <- retW may be NULL */
        //
        // So an unset TMP crashes BOTH implementations identically. That is an upstream
        // hazard, not a translation defect, and the row that would "expose" it is really
        // just modelling an environment Windows never presents. Setting it here is the
        // harness supplying the environment, in the same spirit as ref_set_console_fd.
        std::env::set_var("TMP", std::env::temp_dir());

        // Each side gets its OWN stdin/stdout/stderr, bound BEFORE that side's init
        // because `msvcrt_init_io` is the only reader of `GetStdHandle`. See
        // `ref_set_std_fds` (fileimpl.c) for why this is not `freopen`.
        bind_std_streams(REF);
        ref_msvcrt_init_io();
        bind_std_streams(FLAT);
        fileflat_msvcrt_init_io();
        bind_std_streams(SUBJ);
        msvcrt_init_io();
        assert_shims_live();
        run();
        for side in [SUBJ, REF, FLAT] {
            for which in ["in", "out", "err"] {
                let _ = std::fs::remove_file(std_path(side, which));
            }
        }
    }

    let res = unsafe { &*core::ptr::addr_of!(RESULTS) };
    let (mut m, mut d, mut u) = (0, 0, 0);
    println!(
        "\n=== wave 18a+b: msvcrt-crt-rust file.rs (SHIPPING) vs GENUINE msvcrt file.c ===\n"
    );
    for r in res {
        let tag = match r.v {
            V::Match => {
                m += 1;
                "MATCH       ✓"
            }
            V::Diverge => {
                d += 1;
                "DIVERGE     ✗"
            }
            V::Unexercised => {
                u += 1;
                "UNEXERCISED ✗"
            }
        };
        println!("  {:<26} {}", r.name, tag);
        if !r.note.is_empty() {
            println!("        {}", r.note);
        }
    }
    let cases = unsafe { CASES };
    println!("\n  {m} MATCH   {d} DIVERGE   {u} UNEXERCISED   (of {} rows)", res.len());
    if d > 0 || u > 0 {
        println!(
            "\n  ✗ FAIL — these functions are FLIPPED, so a divergence means msvcrt.dll is\n    \
             shipping a wrong answer, and an UNEXERCISED row means the green above is\n    \
             worthless for the branch it exists to test.\n"
        );
        println!("{cases} file cases run");
        std::process::exit(1);
    }
    println!("\n{cases} file cases passed\n");
}

/// The subject's symbols must be the SUBJECT's.
///
/// Every name this suite calls unprefixed — `fopen`, `fread`, `remove`, `rename` — is also
/// a glibc export. If one bound to glibc the row would still run, still compare two
/// answers, and grade the host libc against msvcrt while reporting MATCH. The check is
/// that the subject and the reference disagree about something only msvcrt does: msvcrt's
/// `_filelength` on a bad fd sets EBADF and returns -1, where glibc has no such call at
/// all, and `_fileno` of a FILE opened by msvcrt's `fopen` must be an fd THIS side's
/// `_read` understands.
unsafe fn assert_shims_live() {
    let (c, p) = tmp_path("shimcheck");
    std::fs::write(&p, b"vibeos").unwrap();

    let f = fopen(c.as_ptr() as *const u8, b"rb\0".as_ptr());
    assert!(!f.is_null(), "the subject's fopen could not open a file that exists");
    let fd = _fileno(f);
    let mut b = [0u8; 6];
    assert_eq!(
        _read(fd, b.as_mut_ptr() as *mut c_void, 6),
        6,
        "the subject's _read did not understand the fd its own _fileno returned — \
         one of the two is glibc's"
    );
    assert_eq!(&b, b"vibeos");
    assert_eq!(fclose(f), 0);

    let g = ref_fopen(c.as_ptr() as *const u8, b"rb\0".as_ptr());
    assert!(!g.is_null(), "the reference's fopen could not open a file that exists");
    assert_eq!(ref_fclose(g), 0);

    // The two sides' FILE tables are SEPARATE — that is the property the whole suite
    // rests on. If they were shared, the same slot would come back twice.
    let a = fopen(c.as_ptr() as *const u8, b"rb\0".as_ptr());
    let b2 = ref_fopen(c.as_ptr() as *const u8, b"rb\0".as_ptr());
    assert!(
        a != b2,
        "subject and reference fopen returned the SAME FILE* — the _iob/__pioinfo rename \
         wall is not in effect and this suite would be grading one implementation against \
         itself"
    );
    assert_eq!(fclose(a), 0);
    assert_eq!(ref_fclose(b2), 0);

    // Wave 18b brought a second batch of names glibc also exports — `getc`, `setvbuf`,
    // `freopen`, `fgetpos`, `fgetwc`. They bind through the same mechanism `fopen` just
    // demonstrated, but "the same mechanism" is an inference, and not inferring is this
    // function's entire job. `getc` is the discriminator: handed a msvcrt `FILE*`, glibc's
    // would read msvcrt's `_ptr` field as `_IO_FILE`'s flags and answer nonsense.
    // Run on BOTH sides, and that is not symmetry for its own sake. `getc` fills the
    // stream's buffer, and `_bufsiz` is slot state that `fopen` does NOT reset — it is
    // whatever the previous tenant left. Doing this to the subject alone left slot 3
    // carrying 4096 on one side and 0 on the other, and the next three rows to reach that
    // slot (`fopen`, `fopen_s`, `fread` — all wave 18a, all previously green) diverged on
    // a field none of them was about.
    let h = fopen(c.as_ptr() as *const u8, b"rb\0".as_ptr());
    let h2 = ref_fopen(c.as_ptr() as *const u8, b"rb\0".as_ptr());
    assert!(!h.is_null() && !h2.is_null(), "fopen could not reopen the fixture");
    assert_eq!(
        getc(h),
        b'v' as c_int,
        "`getc` did not read the first byte of a msvcrt FILE* — it is glibc's, and every \
         wave 18b row that calls an unprefixed name is grading the host libc"
    );
    assert_eq!(ref_getc(h2), b'v' as c_int);
    assert_eq!(fclose(h), 0);
    assert_eq!(ref_fclose(h2), 0);

    // Wave 18c part 3 rests on two claims the rows below cannot make for themselves.
    //
    // FIRST, that `IOB_STRIDE` is the real one. Every std-stream row addresses `stdin` and
    // `stdout` as `__iob_func() + i * IOB_STRIDE`, and a stride that were wrong by a field
    // would hand both sides a misaligned interior pointer — which does not crash, because
    // the array is contiguous. It reads a `_ptr` as a `_flag` and grades the rubble. The
    // check is msvcrt's own contract: `msvcrt_init_io` sets `_file = i` for the first
    // three, so walking the stride must produce exactly 0, 1, 2.
    for i in 0..3 {
        assert_eq!(
            _fileno(s_std(i)),
            i as c_int,
            "the subject's _iob[{i}] does not carry fd {i} — IOB_STRIDE is wrong, or \
             __iob_func did not answer the base of the array"
        );
        assert_eq!(ref__fileno(r_std(i)), i as c_int, "the reference's _iob[{i}] likewise");
    }

    // SECOND, that `putchar` and friends are the SUBJECT's. They are glibc names too, and
    // this batch is the worst case for that: glibc's `putchar` writes to the process's
    // real stdout, so a row grading it would compare two empty files and report MATCH for
    // every case. The discriminator is that the byte must land in the file THIS side's
    // standard output was bound to.
    assert_eq!(putchar(b'Q' as c_int), b'Q' as c_int);
    assert_eq!(fflush(s_std(1)), 0);
    assert_eq!(
        std::fs::read(std_path(SUBJ, "out")).unwrap(),
        b"Q",
        "the subject's `putchar` did not write to the stdout it was bound to — it is \
         glibc's, and every std-stream row would compare two files nothing ever wrote to"
    );
    assert_eq!(ref_putchar(b'Q' as c_int), b'Q' as c_int);
    assert_eq!(ref_fflush(r_std(1)), 0);
    assert_eq!(std::fs::read(std_path(REF, "out")).unwrap(), b"Q", "the reference's likewise");
    reset_stdout();

    let _ = std::fs::remove_file(&p);
}

unsafe fn run() {
    fd_layer_rows();
    stream_rows();
    path_rows();
    // ── wave 18b ──────────────────────────────────────────────────────────────────────
    wide_path_rows();
    narrow_open_rows();
    stat_rows();
    position_rows();
    fd_plumbing_rows();
    wide_stream_rows();
    buffering_rows();
    // ── wave 18c ──────────────────────────────────────────────────────────────────────
    utime_rows();
    global_stream_rows();
    tempname_rows();
    std_stream_rows();
    vprintf_rows();
}

// ══════════════════════════════════════════════════════════════════════════════════════
// The fd layer
// ══════════════════════════════════════════════════════════════════════════════════════

/// The oflag combinations a row draws from. Every one is reachable through `_open`, and
/// the text/binary pair matters more than it looks: msvcrt's text mode rewrites \n on the
/// way out and stops at 0x1A on the way in, so a row that only ever passed `_O_BINARY`
/// would grade half the layer.
fn gen_oflags(r: &mut Rng) -> c_int {
    let base = match r.below(3) {
        0 => O_RDONLY,
        1 => O_WRONLY,
        _ => O_RDWR,
    };
    let mut f = base;
    if r.below(2) == 0 {
        f |= O_BINARY;
    } else {
        f |= O_TEXT;
    }
    if r.below(3) == 0 {
        f |= O_APPEND;
    }
    if r.below(4) == 0 {
        f |= O_TRUNC;
    }
    if r.below(3) == 0 {
        f |= O_CREAT;
    }
    f
}

unsafe fn fd_layer_rows() {
    // `_open` — the whole flag matrix, over a file that exists and one that does not.
    grade("_open", &["ok", "enoent", "created"], ROW_ITERS, |r| {
        let exists = r.below(4) != 0;
        let seed = r.bytes_upto(24);
        let w = if exists { World::new("open", &seed) } else { World::absent("open") };
        let flags = gen_oflags(r);

        let g = Effects::subject(|| _open(w.s(), flags, 0o666) as i64);
        let n = Effects::reference(|| ref__open(w.r(), flags, 0o666) as i64);
        let (mut gi, mut ni) = (Vec::new(), Vec::new());
        gi.extend(fd_state_subject(g.ret as c_int));
        ni.extend(fd_state_reference(n.ret as c_int));
        if g.ret >= 0 {
            _close(g.ret as c_int);
        }
        if n.ret >= 0 {
            ref__close(n.ret as c_int);
        }
        let (sd, rd) = w.disk();
        let tags = vec![
            if n.ret >= 0 { "ok" } else { "enoent" },
            if !exists && n.ret >= 0 { "created" } else { "" },
        ];
        push_disk(&mut gi, sd);
        push_disk(&mut ni, rd);
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("exists={exists} flags={flags:#x}"),
            tags.into_iter().filter(|t| !t.is_empty()).collect(),
        )
    });

    // `_read` — short reads, exact reads, past-EOF reads, and a bad fd.
    grade("_read", &["ok", "eof", "badfd", "partial"], ROW_ITERS, |r| {
        let seed = r.bytes_from(1, 40);
        let w = World::new("read", &seed);
        let bad = r.below(6) == 0;
        let want = r.below(seed.len() + 8);
        let skip = r.below(seed.len() + 2);

        let (sfd, rfd) = if bad {
            (4096, 4096)
        } else {
            (_open(w.s(), O_RDONLY | O_BINARY), ref__open(w.r(), O_RDONLY | O_BINARY))
        };
        if !bad {
            _lseek(sfd, skip as i32, SEEK_SET);
            ref__lseek(rfd, skip as i32, SEEK_SET);
        }
        let (mut sb, mut rb) = (Buf::new(want), Buf::new(want));
        let (sp, rp) = (sb.p(), rb.p());
        let g = Effects::subject(|| _read(sfd, sp as *mut c_void, want as u32) as i64);
        let n = Effects::reference(|| ref__read(rfd, rp as *mut c_void, want as u32) as i64);
        let (mut gi, mut ni) = (sb.img(), rb.img());
        if !bad {
            gi.extend(fd_state_subject(sfd));
            ni.extend(fd_state_reference(rfd));
            _close(sfd);
            ref__close(rfd);
        }
        let avail = seed.len().saturating_sub(skip);
        let tags = vec![
            if bad {
                "badfd"
            } else if n.ret == 0 {
                "eof"
            } else if (n.ret as usize) < want {
                "partial"
            } else {
                "ok"
            },
        ];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("len={} skip={skip} want={want} avail={avail} bad={bad}", seed.len()),
            tags,
        )
    });

    // `_write` — including the zero-length call, which must not truncate, and a bad fd.
    grade("_write", &["ok", "badfd", "zero"], ROW_ITERS, |r| {
        let seed = r.bytes_upto(24);
        let w = World::new("write", &seed);
        let bad = r.below(6) == 0;
        let payload = r.bytes_upto(24);
        let append = r.below(3) == 0;
        let flags = O_WRONLY | O_BINARY | if append { O_APPEND } else { 0 };

        let (sfd, rfd) =
            if bad { (4096, 4096) } else { (_open(w.s(), flags), ref__open(w.r(), flags)) };
        let g =
            Effects::subject(|| _write(sfd, payload.as_ptr() as *const c_void, payload.len() as u32) as i64);
        let n = Effects::reference(|| {
            ref__write(rfd, payload.as_ptr() as *const c_void, payload.len() as u32) as i64
        });
        let (mut gi, mut ni) = (Vec::new(), Vec::new());
        if !bad {
            gi.extend(fd_state_subject(sfd));
            ni.extend(fd_state_reference(rfd));
            _close(sfd);
            ref__close(rfd);
        }
        let (sd, rd) = w.disk();
        push_disk(&mut gi, sd);
        push_disk(&mut ni, rd);
        let tags =
            vec![if bad { "badfd" } else if payload.is_empty() { "zero" } else { "ok" }];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("seed={} write={} append={append} bad={bad}", seed.len(), payload.len()),
            tags,
        )
    });

    // `_lseek` / `_lseeki64` — every whence, plus the negative-result case that must fail
    // rather than move the pointer.
    for (name, wide) in [("_lseek", false), ("_lseeki64", true)] {
        grade(name, &["ok", "einval", "past-eof"], ROW_ITERS, |r| {
            let seed = r.bytes_from(1, 40);
            let w = World::new("lseek", &seed);
            let whence = [SEEK_SET, SEEK_CUR, SEEK_END][r.below(3)];
            let off = r.below(seed.len() * 2 + 8) as i64 - (seed.len() as i64);

            let sfd = _open(w.s(), O_RDONLY | O_BINARY);
            let rfd = ref__open(w.r(), O_RDONLY | O_BINARY);
            let (g, n) = if wide {
                (
                    Effects::subject(|| _lseeki64(sfd, off, whence)),
                    Effects::reference(|| ref__lseeki64(rfd, off, whence)),
                )
            } else {
                (
                    Effects::subject(|| _lseek(sfd, off as i32, whence) as i64),
                    Effects::reference(|| ref__lseek(rfd, off as i32, whence) as i64),
                )
            };
            let (mut gi, mut ni) = (Vec::new(), Vec::new());
            gi.extend(fd_state_subject(sfd));
            ni.extend(fd_state_reference(rfd));
            _close(sfd);
            ref__close(rfd);
            let tags = vec![if n.ret < 0 {
                "einval"
            } else if n.ret as usize > seed.len() {
                "past-eof"
            } else {
                "ok"
            }];
            (
                Obs::with(g, gi),
                Obs::with(n, ni),
                format!("len={} off={off} whence={whence}", seed.len()),
                tags,
            )
        });
    }

    // `_eof` — the flag is latched by a READ that hit the end, not by the position, which
    // is why the row reads rather than just seeking.
    grade("_eof", &["yes", "no", "badfd"], ROW_ITERS, |r| {
        let seed = r.bytes_from(1, 16);
        let w = World::new("eof", &seed);
        let bad = r.below(6) == 0;
        let over = r.below(2) == 0;
        let (sfd, rfd) = if bad {
            (4096, 4096)
        } else {
            (_open(w.s(), O_RDONLY | O_BINARY), ref__open(w.r(), O_RDONLY | O_BINARY))
        };
        if !bad {
            let n = if over { seed.len() + 4 } else { seed.len() / 2 };
            let mut b = vec![0u8; n + 1];
            _read(sfd, b.as_mut_ptr() as *mut c_void, n as u32);
            ref__read(rfd, b.as_mut_ptr() as *mut c_void, n as u32);
        }
        let g = Effects::subject(|| _eof(sfd) as i64);
        let n = Effects::reference(|| ref__eof(rfd) as i64);
        if !bad {
            _close(sfd);
            ref__close(rfd);
        }
        let tags =
            vec![if bad { "badfd" } else if n.ret == 1 { "yes" } else { "no" }];
        (
            Obs::with(g, Vec::new()),
            Obs::with(n, Vec::new()),
            format!("len={} over={over} bad={bad}", seed.len()),
            tags,
        )
    });

    // `_filelength` / `_filelengthi64` — must RESTORE the position it moved to measure.
    for (name, wide) in [("_filelength", false), ("_filelengthi64", true)] {
        grade(name, &["ok", "empty", "badfd"], ROW_ITERS, |r| {
            let seed = r.bytes_upto(40);
            let w = World::new("flen", &seed);
            let bad = r.below(6) == 0;
            let pos = r.below(seed.len() + 1);
            let (sfd, rfd) = if bad {
                (4096, 4096)
            } else {
                (_open(w.s(), O_RDONLY | O_BINARY), ref__open(w.r(), O_RDONLY | O_BINARY))
            };
            if !bad {
                _lseek(sfd, pos as i32, SEEK_SET);
                ref__lseek(rfd, pos as i32, SEEK_SET);
            }
            let (g, n) = if wide {
                (
                    Effects::subject(|| _filelengthi64(sfd)),
                    Effects::reference(|| ref__filelengthi64(rfd)),
                )
            } else {
                (
                    Effects::subject(|| _filelength(sfd) as i64),
                    Effects::reference(|| ref__filelength(rfd) as i64),
                )
            };
            let (mut gi, mut ni) = (Vec::new(), Vec::new());
            if !bad {
                // The position is part of the answer: _filelength seeks to the end and
                // seeks back, and a body that forgot the second seek returns the right
                // length and leaves the fd somewhere else entirely.
                gi.extend(fd_state_subject(sfd));
                ni.extend(fd_state_reference(rfd));
                _close(sfd);
                ref__close(rfd);
            }
            let tags = vec![if bad {
                "badfd"
            } else if seed.is_empty() {
                "empty"
            } else {
                "ok"
            }];
            (
                Obs::with(g, gi),
                Obs::with(n, ni),
                format!("len={} pos={pos} bad={bad}", seed.len()),
                tags,
            )
        });
    }

    // `_dup` — the copy shares the file pointer, so the row moves one and reads the other.
    grade("_dup", &["ok", "badfd"], ROW_ITERS, |r| {
        let seed = r.bytes_from(1, 24);
        let w = World::new("dup", &seed);
        let bad = r.below(6) == 0;
        let pos = r.below(seed.len());
        let (sfd, rfd) = if bad {
            (4096, 4096)
        } else {
            (_open(w.s(), O_RDONLY | O_BINARY), ref__open(w.r(), O_RDONLY | O_BINARY))
        };
        if !bad {
            _lseek(sfd, pos as i32, SEEK_SET);
            ref__lseek(rfd, pos as i32, SEEK_SET);
        }
        let g = Effects::subject(|| _dup(sfd) as i64);
        let n = Effects::reference(|| ref__dup(rfd) as i64);
        let (mut gi, mut ni) = (Vec::new(), Vec::new());
        if g.ret >= 0 {
            gi.extend(fd_state_subject(g.ret as c_int));
            _close(g.ret as c_int);
        }
        if n.ret >= 0 {
            ni.extend(fd_state_reference(n.ret as c_int));
            ref__close(n.ret as c_int);
        }
        if !bad {
            _close(sfd);
            ref__close(rfd);
        }
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("len={} pos={pos} bad={bad}", seed.len()),
            vec![if bad { "badfd" } else { "ok" }],
        )
    });

    // `_commit` — a flush of the fd, and EBADF on a closed one.
    grade("_commit", &["ok", "badfd"], ROW_ITERS, |r| {
        let seed = r.bytes_upto(16);
        let w = World::new("commit", &seed);
        let bad = r.below(4) == 0;
        let (sfd, rfd) = if bad {
            (4096, 4096)
        } else {
            (_open(w.s(), O_WRONLY | O_BINARY), ref__open(w.r(), O_WRONLY | O_BINARY))
        };
        let g = Effects::subject(|| _commit(sfd) as i64);
        let n = Effects::reference(|| ref__commit(rfd) as i64);
        if !bad {
            _close(sfd);
            ref__close(rfd);
        }
        (
            Obs::with(g, Vec::new()),
            Obs::with(n, Vec::new()),
            format!("bad={bad}"),
            vec![if bad { "badfd" } else { "ok" }],
        )
    });

    // `_chsize` — grow (zero-filled) and shrink, both visible only on disk.
    grade("_chsize", &["grow", "shrink", "badfd"], ROW_ITERS, |r| {
        let seed = r.bytes_from(1, 24);
        let w = World::new("chsize", &seed);
        let bad = r.below(6) == 0;
        let size = r.below(seed.len() * 2 + 4) as i32;
        let (sfd, rfd) = if bad {
            (4096, 4096)
        } else {
            (_open(w.s(), O_RDWR | O_BINARY), ref__open(w.r(), O_RDWR | O_BINARY))
        };
        let g = Effects::subject(|| _chsize(sfd, size) as i64);
        let n = Effects::reference(|| ref__chsize(rfd, size) as i64);
        let (mut gi, mut ni) = (Vec::new(), Vec::new());
        if !bad {
            gi.extend(fd_state_subject(sfd));
            ni.extend(fd_state_reference(rfd));
            _close(sfd);
            ref__close(rfd);
        }
        let (sd, rd) = w.disk();
        push_disk(&mut gi, sd);
        push_disk(&mut ni, rd);
        let tags = vec![if bad {
            "badfd"
        } else if size as usize > seed.len() {
            "grow"
        } else {
            "shrink"
        }];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("len={} size={size} bad={bad}", seed.len()),
            tags,
        )
    });

    // `_setmode` — returns the PREVIOUS mode, which is the part an implementation gets
    // wrong by returning the new one.
    grade("_setmode", &["ok", "badfd", "einval"], ROW_ITERS, |r| {
        let seed = r.bytes_upto(8);
        let w = World::new("setmode", &seed);
        let bad = r.below(6) == 0;
        let opened_text = r.below(2) == 0;
        let mode = match r.below(3) {
            0 => O_TEXT,
            1 => O_BINARY,
            _ => 0x1234, // not a mode at all: must be EINVAL, not silently accepted
        };
        let of = O_RDONLY | if opened_text { O_TEXT } else { O_BINARY };
        let (sfd, rfd) =
            if bad { (4096, 4096) } else { (_open(w.s(), of), ref__open(w.r(), of)) };
        let g = Effects::subject(|| _setmode(sfd, mode) as i64);
        let n = Effects::reference(|| ref__setmode(rfd, mode) as i64);
        if !bad {
            _close(sfd);
            ref__close(rfd);
        }
        let tags = vec![if bad {
            "badfd"
        } else if n.ret < 0 {
            "einval"
        } else {
            "ok"
        }];
        (
            Obs::with(g, Vec::new()),
            Obs::with(n, Vec::new()),
            format!("opened_text={opened_text} mode={mode:#x} bad={bad}"),
            tags,
        )
    });

    // `_isatty` — reads the WX_TTY flag the open latched, never GetFileType.
    grade("_isatty", &["no", "badfd"], ROW_ITERS, |r| {
        let seed = r.bytes_upto(8);
        let w = World::new("isatty", &seed);
        let bad = r.below(4) == 0;
        let (sfd, rfd) = if bad {
            (4096, 4096)
        } else {
            (_open(w.s(), O_RDONLY | O_BINARY), ref__open(w.r(), O_RDONLY | O_BINARY))
        };
        let g = Effects::subject(|| _isatty(sfd) as i64);
        let n = Effects::reference(|| ref__isatty(rfd) as i64);
        if !bad {
            _close(sfd);
            ref__close(rfd);
        }
        (
            Obs::with(g, Vec::new()),
            Obs::with(n, Vec::new()),
            format!("bad={bad}"),
            vec![if bad { "badfd" } else { "no" }],
        )
    });

    // `_close` — the success path and the double close.
    grade("_close", &["ok", "badfd"], ROW_ITERS, |r| {
        let seed = r.bytes_upto(8);
        let w = World::new("close", &seed);
        let twice = r.below(3) == 0;
        let sfd = _open(w.s(), O_RDONLY | O_BINARY);
        let rfd = ref__open(w.r(), O_RDONLY | O_BINARY);
        if twice {
            _close(sfd);
            ref__close(rfd);
        }
        let g = Effects::subject(|| _close(sfd) as i64);
        let n = Effects::reference(|| ref__close(rfd) as i64);
        (
            Obs::with(g, Vec::new()),
            Obs::with(n, Vec::new()),
            format!("twice={twice}"),
            vec![if twice { "badfd" } else { "ok" }],
        )
    });
}

// ══════════════════════════════════════════════════════════════════════════════════════
// The stream layer
// ══════════════════════════════════════════════════════════════════════════════════════

/// The mode strings `fopen` takes. Text vs binary is the interesting axis, and `"a"` is
/// the one whose position rule (every write goes to the end, whatever the seek said)
/// nothing else covers.
const MODES: [&[u8]; 8] =
    [b"rb\0", b"r\0", b"wb\0", b"w\0", b"ab\0", b"a\0", b"r+b\0", b"w+b\0"];

unsafe fn stream_rows() {
    // `fopen` — over a file that exists and one that does not, across the mode matrix.
    grade("fopen", &["ok", "fail", "created"], ROW_ITERS, |r| {
        let exists = r.below(4) != 0;
        let seed = r.bytes_upto(24);
        let w = if exists { World::new("fopen", &seed) } else { World::absent("fopen") };
        let mode = MODES[r.below(MODES.len())];

        let mut sf: FILEP = core::ptr::null_mut();
        let mut rf: FILEP = core::ptr::null_mut();
        let g = Effects::subject(|| {
            sf = fopen(w.s(), mode.as_ptr());
            !sf.is_null() as i64
        });
        let n = Effects::reference(|| {
            rf = ref_fopen(w.r(), mode.as_ptr());
            !rf.is_null() as i64
        });
        let mut gi = stream_state_subject(sf);
        let mut ni = stream_state_reference(rf);
        if !sf.is_null() {
            fclose(sf);
        }
        if !rf.is_null() {
            ref_fclose(rf);
        }
        let (sd, rd) = w.disk();
        push_disk(&mut gi, sd);
        push_disk(&mut ni, rd);
        let tags = vec![
            if n.ret == 1 { "ok" } else { "fail" },
            if !exists && n.ret == 1 { "created" } else { "" },
        ];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("exists={exists} mode={}", String::from_utf8_lossy(&mode[..mode.len() - 1])),
            tags.into_iter().filter(|t| !t.is_empty()).collect(),
        )
    });

    // `fopen_s` — the `_s` shell: a NULL out-pointer must fire the invalid-parameter
    // handler and return EINVAL rather than crash, and the out-pointer must be zeroed on
    // failure. Both are contract, not implementation detail.
    grade("fopen_s", &["ok", "fail", "nullout"], ROW_ITERS, |r| {
        let exists = r.below(3) != 0;
        let seed = r.bytes_upto(16);
        let w = if exists { World::new("fopens", &seed) } else { World::absent("fopens") };
        let mode = MODES[r.below(MODES.len())];
        let nullout = r.below(8) == 0;

        let mut sf: FILEP = core::ptr::null_mut();
        let mut rf: FILEP = core::ptr::null_mut();
        let sp = if nullout { core::ptr::null_mut() } else { &mut sf as *mut FILEP };
        let rp = if nullout { core::ptr::null_mut() } else { &mut rf as *mut FILEP };
        let g = Effects::subject(|| fopen_s(sp, w.s(), mode.as_ptr()) as i64);
        let n = Effects::reference(|| ref_fopen_s(rp, w.r(), mode.as_ptr()) as i64);
        let mut gi = stream_state_subject(sf);
        let mut ni = stream_state_reference(rf);
        if !sf.is_null() {
            fclose(sf);
        }
        if !rf.is_null() {
            ref_fclose(rf);
        }
        let (sd, rd) = w.disk();
        push_disk(&mut gi, sd);
        push_disk(&mut ni, rd);
        let tags = vec![if nullout {
            "nullout"
        } else if n.ret == 0 {
            "ok"
        } else {
            "fail"
        }];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("exists={exists} nullout={nullout}"),
            tags,
        )
    });

    // `fread` — the count/size split is the trap: msvcrt returns whole ITEMS read, and a
    // partial final item is not counted but IS consumed.
    grade("fread", &["ok", "eof", "partial", "zero"], ROW_ITERS, |r| {
        let seed = r.bytes_from(1, 40);
        let w = World::new("fread", &seed);
        let size = 1 + r.below(4);
        let count = r.below(seed.len() / size + 3);
        let skip = r.below(seed.len());

        let sf = fopen(w.s(), b"rb\0".as_ptr());
        let rf = ref_fopen(w.r(), b"rb\0".as_ptr());
        fseek(sf, skip as i32, SEEK_SET);
        ref_fseek(rf, skip as i32, SEEK_SET);
        let (mut sb, mut rb) = (Buf::new(size * count), Buf::new(size * count));
        let (sbp, rbp) = (sb.p(), rb.p());
        let g = Effects::subject(|| fread(sbp as *mut c_void, size, count, sf) as i64);
        let n = Effects::reference(|| ref_fread(rbp as *mut c_void, size, count, rf) as i64);
        let (mut gi, mut ni) = (sb.img(), rb.img());
        gi.extend(stream_state_subject(sf));
        ni.extend(stream_state_reference(rf));
        fclose(sf);
        ref_fclose(rf);
        let avail = seed.len() - skip;
        let tags = vec![if count == 0 || size == 0 {
            "zero"
        } else if n.ret == 0 {
            "eof"
        } else if (n.ret as usize) < count {
            "partial"
        } else {
            "ok"
        }];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("len={} skip={skip} size={size} count={count} avail={avail}", seed.len()),
            tags,
        )
    });

    // `fwrite` — same split, and the result is only visible after the stream is closed,
    // which is what makes the buffering layer part of the answer.
    grade("fwrite", &["ok", "zero"], ROW_ITERS, |r| {
        let seed = r.bytes_upto(16);
        let w = World::new("fwrite", &seed);
        let size = 1 + r.below(4);
        let count = r.below(12);
        let payload = r.bytes(size * count + 4);
        let mi = r.below(3);
        let mode = [b"wb\0".as_ptr(), b"ab\0".as_ptr(), b"r+b\0".as_ptr()][mi];
        let mname = ["wb", "ab", "r+b"][mi];

        let sf = fopen(w.s(), mode);
        let rf = ref_fopen(w.r(), mode);
        let g = Effects::subject(|| fwrite(payload.as_ptr() as *const c_void, size, count, sf) as i64);
        let n = Effects::reference(|| {
            ref_fwrite(payload.as_ptr() as *const c_void, size, count, rf) as i64
        });
        let (spos, rpos) = (ftell(sf), ref_ftell(rf));
        let (sdesc, rdesc) = (buffer_desc(sf), buffer_desc(rf));
        let (mut gi, mut ni) = (Vec::new(), Vec::new());
        gi.extend(stream_state_subject(sf));
        ni.extend(stream_state_reference(rf));
        fclose(sf);
        ref_fclose(rf);
        let (sd, rd) = w.disk();
        push_disk(&mut gi, sd);
        push_disk(&mut ni, rd);
        let tags = vec![if count == 0 { "zero" } else { "ok" }];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!(
                "seed={} size={size} count={count} mode={mname}\n         ftell rust={spos} msvcrt={rpos}\n         rust  buf: {}\n         msvcrt buf: {}",
                seed.len(), sdesc, rdesc
            ),
            tags,
        )
    });

    // `fgetc` — one byte at a time to the end and one past it, in BOTH modes, because
    // text mode is where 0x0D disappears and 0x1A ends the file early.
    grade("fgetc", &["ok", "eof", "text"], ROW_ITERS, |r| {
        let seed = r.bytes_from(1, 24);
        let w = World::new("fgetc", &seed);
        let text = r.below(2) == 0;
        let n_reads = 1 + r.below(seed.len() + 3);
        let mode = if text { b"r\0".as_ptr() } else { b"rb\0".as_ptr() };

        let sf = fopen(w.s(), mode);
        let rf = ref_fopen(w.r(), mode);
        let (mut gs, mut ns) = (Vec::new(), Vec::new());
        let mut g = Effects::subject(|| 0);
        let mut n = Effects::reference(|| 0);
        for _ in 0..n_reads {
            g = Effects::subject(|| fgetc(sf) as i64);
            n = Effects::reference(|| ref_fgetc(rf) as i64);
            gs.extend_from_slice(&(g.ret as i32).to_le_bytes());
            ns.extend_from_slice(&(n.ret as i32).to_le_bytes());
        }
        gs.extend(stream_state_subject(sf));
        ns.extend(stream_state_reference(rf));
        fclose(sf);
        ref_fclose(rf);
        let tags = vec![if n.ret < 0 { "eof" } else { "ok" }, if text { "text" } else { "" }];
        (
            Obs::with(g, gs),
            Obs::with(n, ns),
            format!("len={} reads={n_reads} text={text}", seed.len()),
            tags.into_iter().filter(|t| !t.is_empty()).collect(),
        )
    });

    // `fputc` — in text mode a '\n' becomes two bytes on disk, which only the disk half
    // of the image can see.
    grade("fputc", &["ok", "text", "newline"], ROW_ITERS, |r| {
        let w = World::new("fputc", b"");
        let text = r.below(2) == 0;
        let payload = r.bytes_from(1, 12);
        let mode = if text { b"w\0".as_ptr() } else { b"wb\0".as_ptr() };

        let sf = fopen(w.s(), mode);
        let rf = ref_fopen(w.r(), mode);
        let (mut gs, mut ns) = (Vec::new(), Vec::new());
        let mut g = Effects::subject(|| 0);
        let mut n = Effects::reference(|| 0);
        for &c in &payload {
            g = Effects::subject(|| fputc(c as c_int, sf) as i64);
            n = Effects::reference(|| ref_fputc(c as c_int, rf) as i64);
            gs.extend_from_slice(&(g.ret as i32).to_le_bytes());
            ns.extend_from_slice(&(n.ret as i32).to_le_bytes());
        }
        gs.extend(stream_state_subject(sf));
        ns.extend(stream_state_reference(rf));
        fclose(sf);
        ref_fclose(rf);
        let (sd, rd) = w.disk();
        push_disk(&mut gs, sd);
        push_disk(&mut ns, rd);
        let tags = vec![
            "ok",
            if text { "text" } else { "" },
            if payload.contains(&b'\n') { "newline" } else { "" },
        ];
        (
            Obs::with(g, gs),
            Obs::with(n, ns),
            format!("n={} text={text}", payload.len()),
            tags.into_iter().filter(|t| !t.is_empty()).collect(),
        )
    });

    // `fgets` — stops at the newline AND keeps it, NUL-terminates, and returns NULL only
    // at EOF-with-nothing-read. A size of 1 must produce an empty string, not a byte.
    grade("fgets", &["ok", "eof", "newline", "size1"], ROW_ITERS, |r| {
        let seed = r.bytes_from(1, 40);
        let w = World::new("fgets", &seed);
        let size = r.below(seed.len() + 4) as c_int;
        let skip = r.below(seed.len());

        let sf = fopen(w.s(), b"rb\0".as_ptr());
        let rf = ref_fopen(w.r(), b"rb\0".as_ptr());
        fseek(sf, skip as i32, SEEK_SET);
        ref_fseek(rf, skip as i32, SEEK_SET);
        let (mut sb, mut rb) = (Buf::new(size.max(0) as usize + 4), Buf::new(size.max(0) as usize + 4));
        let (sbp, rbp) = (sb.p(), rb.p());
        let g = Effects::subject(|| !fgets(sbp, size, sf).is_null() as i64);
        let n = Effects::reference(|| !ref_fgets(rbp, size, rf).is_null() as i64);
        let (mut gi, mut ni) = (sb.img(), rb.img());
        gi.extend(stream_state_subject(sf));
        ni.extend(stream_state_reference(rf));
        fclose(sf);
        ref_fclose(rf);
        let tags = vec![
            if n.ret == 1 { "ok" } else { "eof" },
            if size == 1 { "size1" } else { "" },
            if seed[skip..].contains(&b'\n') { "newline" } else { "" },
        ];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("len={} skip={skip} size={size}", seed.len()),
            tags.into_iter().filter(|t| !t.is_empty()).collect(),
        )
    });

    // `fputs` — no newline appended, and the empty string must still be a success.
    grade("fputs", &["ok", "empty"], ROW_ITERS, |r| {
        let w = World::new("fputs", b"");
        let text = r.below(2) == 0;
        // NUL-free by construction: fputs takes a C string, so an embedded NUL would
        // shorten the payload rather than test anything.
        let n_bytes = r.below(20);
        let mut s: Vec<u8> = (0..n_bytes)
            .map(|_| match r.below(4) {
                0 => b'\n',
                1 => b'\r',
                _ => 1 + (r.next() % 255) as u8,
            })
            .collect();
        s.push(0);
        let mode = if text { b"w\0".as_ptr() } else { b"wb\0".as_ptr() };

        let sf = fopen(w.s(), mode);
        let rf = ref_fopen(w.r(), mode);
        let g = Effects::subject(|| fputs(s.as_ptr(), sf) as i64);
        let n = Effects::reference(|| ref_fputs(s.as_ptr(), rf) as i64);
        let (mut gi, mut ni) = (Vec::new(), Vec::new());
        gi.extend(stream_state_subject(sf));
        ni.extend(stream_state_reference(rf));
        fclose(sf);
        ref_fclose(rf);
        let (sd, rd) = w.disk();
        push_disk(&mut gi, sd);
        push_disk(&mut ni, rd);
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("n={n_bytes} text={text}"),
            vec![if n_bytes == 0 { "empty" } else { "ok" }],
        )
    });

    // `ungetc` — one slot, and the pushed byte must come back from the NEXT read while
    // the reported position goes BACK by one.
    grade("ungetc", &["ok", "eof-arg", "twice"], ROW_ITERS, |r| {
        let seed = r.bytes_from(2, 24);
        let w = World::new("ungetc", &seed);
        let skip = r.below(seed.len());
        let push = if r.below(6) == 0 { -1 } else { r.below(256) as c_int };
        let twice = r.below(4) == 0;

        let sf = fopen(w.s(), b"rb\0".as_ptr());
        let rf = ref_fopen(w.r(), b"rb\0".as_ptr());
        fseek(sf, skip as i32, SEEK_SET);
        ref_fseek(rf, skip as i32, SEEK_SET);
        // A read first: msvcrt's ungetc on a stream that has never been read has no
        // buffer to push into, and that arm is a different one.
        fgetc(sf);
        ref_fgetc(rf);
        let mut g = Effects::subject(|| ungetc(push, sf) as i64);
        let mut n = Effects::reference(|| ref_ungetc(push, rf) as i64);
        let (mut gi, mut ni) = (Vec::new(), Vec::new());
        gi.extend_from_slice(&(g.ret as i32).to_le_bytes());
        ni.extend_from_slice(&(n.ret as i32).to_le_bytes());
        if twice {
            g = Effects::subject(|| ungetc(push, sf) as i64);
            n = Effects::reference(|| ref_ungetc(push, rf) as i64);
        }
        // What comes back next IS the contract.
        gi.extend_from_slice(&(fgetc(sf) as i32).to_le_bytes());
        ni.extend_from_slice(&(ref_fgetc(rf) as i32).to_le_bytes());
        gi.extend(stream_state_subject(sf));
        ni.extend(stream_state_reference(rf));
        fclose(sf);
        ref_fclose(rf);
        let tags = vec![
            if push < 0 { "eof-arg" } else { "ok" },
            if twice { "twice" } else { "" },
        ];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("len={} skip={skip} push={push} twice={twice}", seed.len()),
            tags.into_iter().filter(|t| !t.is_empty()).collect(),
        )
    });

    // `fseek` / `ftell` — the pair, because a position is only meaningful if the other
    // one agrees with it. A seek must also CLEAR the EOF flag.
    grade("fseek+ftell", &["ok", "fail", "clears-eof"], ROW_ITERS, |r| {
        let seed = r.bytes_from(1, 40);
        let w = World::new("fseek", &seed);
        let whence = [SEEK_SET, SEEK_CUR, SEEK_END][r.below(3)];
        let off = r.below(seed.len() * 2 + 8) as i32 - (seed.len() as i32);
        let hit_eof = r.below(2) == 0;

        let sf = fopen(w.s(), b"rb\0".as_ptr());
        let rf = ref_fopen(w.r(), b"rb\0".as_ptr());
        if hit_eof {
            let mut b = vec![0u8; seed.len() + 4];
            fread(b.as_mut_ptr() as *mut c_void, 1, seed.len() + 4, sf);
            ref_fread(b.as_mut_ptr() as *mut c_void, 1, seed.len() + 4, rf);
        }
        let g = Effects::subject(|| fseek(sf, off, whence) as i64);
        let n = Effects::reference(|| ref_fseek(rf, off, whence) as i64);
        let mut gi = stream_state_subject(sf);
        let mut ni = stream_state_reference(rf);
        // ftell separately, so a body that answers from a stale cached position rather
        // than the fd is caught.
        gi.extend_from_slice(&ftell(sf).to_le_bytes());
        ni.extend_from_slice(&ref_ftell(rf).to_le_bytes());
        fclose(sf);
        ref_fclose(rf);
        let tags = vec![
            if n.ret == 0 { "ok" } else { "fail" },
            if hit_eof && n.ret == 0 { "clears-eof" } else { "" },
        ];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("len={} off={off} whence={whence} hit_eof={hit_eof}", seed.len()),
            tags.into_iter().filter(|t| !t.is_empty()).collect(),
        )
    });

    // `rewind` — a seek to 0 that ALSO clears the error flag, unlike fseek.
    grade("rewind", &["ok", "after-eof"], ROW_ITERS, |r| {
        let seed = r.bytes_from(1, 24);
        let w = World::new("rewind", &seed);
        let hit_eof = r.below(2) == 0;

        let sf = fopen(w.s(), b"rb\0".as_ptr());
        let rf = ref_fopen(w.r(), b"rb\0".as_ptr());
        let mut b = vec![0u8; seed.len() + 4];
        let n_read = if hit_eof { seed.len() + 4 } else { r.below(seed.len()) };
        fread(b.as_mut_ptr() as *mut c_void, 1, n_read, sf);
        ref_fread(b.as_mut_ptr() as *mut c_void, 1, n_read, rf);
        let g = Effects::subject(|| {
            rewind(sf);
            0
        });
        let n = Effects::reference(|| {
            ref_rewind(rf);
            0
        });
        let mut gi = stream_state_subject(sf);
        let mut ni = stream_state_reference(rf);
        gi.extend_from_slice(&(fgetc(sf) as i32).to_le_bytes());
        ni.extend_from_slice(&(ref_fgetc(rf) as i32).to_le_bytes());
        fclose(sf);
        ref_fclose(rf);
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("len={} hit_eof={hit_eof}", seed.len()),
            vec![if hit_eof { "after-eof" } else { "ok" }],
        )
    });

    // `feof` / `ferror` / `clearerr` — the flag trio. EOF is latched by a read that ran
    // out, not by the position, and `clearerr` must clear BOTH.
    grade("feof+clearerr", &["eof", "no-eof"], ROW_ITERS, |r| {
        let seed = r.bytes_from(1, 16);
        let w = World::new("feof", &seed);
        let over = r.below(2) == 0;

        let sf = fopen(w.s(), b"rb\0".as_ptr());
        let rf = ref_fopen(w.r(), b"rb\0".as_ptr());
        let n_read = if over { seed.len() + 4 } else { seed.len() / 2 };
        let mut b = vec![0u8; seed.len() + 8];
        fread(b.as_mut_ptr() as *mut c_void, 1, n_read, sf);
        ref_fread(b.as_mut_ptr() as *mut c_void, 1, n_read, rf);

        let g = Effects::subject(|| feof(sf) as i64);
        let n = Effects::reference(|| ref_feof(rf) as i64);
        let (mut gi, mut ni) = (Vec::new(), Vec::new());
        gi.extend_from_slice(&(ferror(sf) as i32).to_le_bytes());
        ni.extend_from_slice(&(ref_ferror(rf) as i32).to_le_bytes());
        clearerr(sf);
        ref_clearerr(rf);
        gi.extend_from_slice(&(feof(sf) as i32).to_le_bytes());
        ni.extend_from_slice(&(ref_feof(rf) as i32).to_le_bytes());
        gi.extend(stream_state_subject(sf));
        ni.extend(stream_state_reference(rf));
        fclose(sf);
        ref_fclose(rf);
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("len={} over={over}", seed.len()),
            vec![if over { "eof" } else { "no-eof" }],
        )
    });

    // `fflush` — the write must reach the disk WITHOUT the close, which is the only way
    // to tell a real flush from a body that relies on fclose to do it.
    grade("fflush", &["ok", "readonly"], ROW_ITERS, |r| {
        let w = World::new("fflush", b"");
        let readonly = r.below(4) == 0;
        let payload = r.bytes_from(1, 24);
        let mode = if readonly { b"rb\0".as_ptr() } else { b"wb\0".as_ptr() };

        let sf = fopen(w.s(), mode);
        let rf = ref_fopen(w.r(), mode);
        if !readonly {
            fwrite(payload.as_ptr() as *const c_void, 1, payload.len(), sf);
            ref_fwrite(payload.as_ptr() as *const c_void, 1, payload.len(), rf);
        }
        let g = Effects::subject(|| fflush(sf) as i64);
        let n = Effects::reference(|| ref_fflush(rf) as i64);
        // BEFORE the close: that is the whole point of the row.
        let (sd, rd) = w.disk();
        let (mut gi, mut ni) = (Vec::new(), Vec::new());
        push_disk(&mut gi, sd);
        push_disk(&mut ni, rd);
        gi.extend(stream_state_subject(sf));
        ni.extend(stream_state_reference(rf));
        fclose(sf);
        ref_fclose(rf);
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("readonly={readonly} n={}", payload.len()),
            vec![if readonly { "readonly" } else { "ok" }],
        )
    });

    // `fclose` — the return value, and that the buffered bytes landed.
    grade("fclose", &["ok"], ROW_ITERS, |r| {
        let w = World::new("fclose", b"");
        let payload = r.bytes_upto(24);
        let sf = fopen(w.s(), b"wb\0".as_ptr());
        let rf = ref_fopen(w.r(), b"wb\0".as_ptr());
        fwrite(payload.as_ptr() as *const c_void, 1, payload.len(), sf);
        ref_fwrite(payload.as_ptr() as *const c_void, 1, payload.len(), rf);
        let g = Effects::subject(|| fclose(sf) as i64);
        let n = Effects::reference(|| ref_fclose(rf) as i64);
        let (sd, rd) = w.disk();
        let (mut gi, mut ni) = (Vec::new(), Vec::new());
        push_disk(&mut gi, sd);
        push_disk(&mut ni, rd);
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("n={}", payload.len()),
            vec!["ok"],
        )
    });
}

// ══════════════════════════════════════════════════════════════════════════════════════
// The path layer
// ══════════════════════════════════════════════════════════════════════════════════════

unsafe fn path_rows() {
    // `remove` / `_unlink` — remove IS _unlink in msvcrt (file.c), so both are graded to
    // pin that they stay the same answer rather than drifting apart.
    for (name, subj, refn) in [
        ("remove", remove as unsafe extern "C" fn(*const u8) -> c_int, ref_remove as unsafe extern "C" fn(*const u8) -> c_int),
        ("_unlink", _unlink, ref__unlink),
    ] {
        grade(name, &["ok", "enoent"], ROW_ITERS, |r| {
            let exists = r.below(3) != 0;
            let seed = r.bytes_upto(16);
            let w = if exists { World::new("rm", &seed) } else { World::absent("rm") };
            let g = Effects::subject(|| subj(w.s()) as i64);
            let n = Effects::reference(|| refn(w.r()) as i64);
            let (sd, rd) = w.disk();
            let (mut gi, mut ni) = (Vec::new(), Vec::new());
            push_disk(&mut gi, sd);
            push_disk(&mut ni, rd);
            let tags = vec![if n.ret == 0 { "ok" } else { "enoent" }];
            (Obs::with(g, gi), Obs::with(n, ni), format!("exists={exists}"), tags)
        });
    }

    // `rename` — the destination-exists case is the interesting one: Win32 refuses it
    // where POSIX would overwrite, and the shim has to reproduce the refusal or this
    // whole branch is unreachable.
    grade("rename", &["ok", "enoent", "dest-exists"], ROW_ITERS, |r| {
        let exists = r.below(4) != 0;
        let dest_exists = r.below(3) == 0;
        let seed = r.bytes_upto(16);
        let w = if exists { World::new("ren", &seed) } else { World::absent("ren") };
        let d = World::new("rendst", if dest_exists { b"x" } else { b"" });
        if !dest_exists {
            let _ = std::fs::remove_file(&d.sp);
            let _ = std::fs::remove_file(&d.rp);
        }

        let g = Effects::subject(|| rename(w.s(), d.s()) as i64);
        let n = Effects::reference(|| ref_rename(w.r(), d.r()) as i64);
        let (sd, rd) = w.disk();
        let (sd2, rd2) = d.disk();
        let (mut gi, mut ni) = (Vec::new(), Vec::new());
        push_disk(&mut gi, sd);
        push_disk(&mut gi, sd2);
        push_disk(&mut ni, rd);
        push_disk(&mut ni, rd2);
        let tags = vec![
            if n.ret == 0 { "ok" } else { "enoent" },
            if dest_exists { "dest-exists" } else { "" },
        ];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("exists={exists} dest_exists={dest_exists}"),
            tags.into_iter().filter(|t| !t.is_empty()).collect(),
        )
    });

    // `_access` — mode 0/2/4/6 against present and absent files.
    grade("_access", &["ok", "enoent"], ROW_ITERS, |r| {
        let exists = r.below(3) != 0;
        let seed = r.bytes_upto(8);
        let w = if exists { World::new("acc", &seed) } else { World::absent("acc") };
        let mode = ACCESS_MODES[r.below(ACCESS_MODES.len())];
        let g = Effects::subject(|| _access(w.s(), mode) as i64);
        let n = Effects::reference(|| ref__access(w.r(), mode) as i64);
        let tags = vec![if n.ret == 0 { "ok" } else { "enoent" }];
        (
            Obs::with(g, Vec::new()),
            Obs::with(n, Vec::new()),
            format!("exists={exists} mode={mode}"),
            tags,
        )
    });
}

// ══════════════════════════════════════════════════════════════════════════════════════
// Wave 18b — the wide path
// ══════════════════════════════════════════════════════════════════════════════════════
//
// Wave 18a graded the narrow entry points and nothing else, which left msvcrt's ENTIRE
// UTF-16 door ungraded: `_wfopen`, `_wsopen_s`, `_wrename`, `_wstat64` and their nine
// relatives went live in msvcrt.dll with a link and three boots behind them, exactly as
// the narrow ones did. That matters more here than it looks, because the wide path is not
// a thin shim over the narrow one — it is the OTHER WAY ROUND. msvcrt implements the wide
// call and reaches Win32's `*W` directly; the narrow entry points are the ones that
// convert (`wstrdupa_utf8`) and delegate. So the wide side is where the real bodies live,
// and until this section every one of them was covered only by its narrow caller.
//
// The row shapes are deliberately the same as their narrow twins — same corpus, same
// tags, same disk comparison. A wide row that diverges where its narrow twin matches is
// then a statement about the conversion, not about the fixture.

/// Wide rows that walk a string a character at a time do real I/O per character, so they
/// run fewer cases than the scalar rows. The `require` tags, not the count, are what
/// assert the interesting branches ran.
const WIDE_ITERS: usize = 120;

unsafe fn wide_path_rows() {
    // `_wfopen` — the wide `fopen`, across the mode matrix and both existence cases.
    grade("_wfopen", &["ok", "fail", "created"], ROW_ITERS, |r| {
        let exists = r.below(4) != 0;
        let seed = r.bytes_upto(24);
        let w = if exists { World::new("wfopen", &seed) } else { World::absent("wfopen") };
        let mode = wmode(MODES[r.below(MODES.len())]);
        let (sw, rw) = (w.ws(), w.wr());

        let mut sf: FILEP = core::ptr::null_mut();
        let mut rf: FILEP = core::ptr::null_mut();
        let g = Effects::subject(|| {
            sf = _wfopen(sw.as_ptr(), mode.as_ptr());
            !sf.is_null() as i64
        });
        let n = Effects::reference(|| {
            rf = ref__wfopen(rw.as_ptr(), mode.as_ptr());
            !rf.is_null() as i64
        });
        let mut gi = stream_state_subject(sf);
        let mut ni = stream_state_reference(rf);
        if !sf.is_null() {
            fclose(sf);
        }
        if !rf.is_null() {
            ref_fclose(rf);
        }
        let (sd, rd) = w.disk();
        push_disk(&mut gi, sd);
        push_disk(&mut ni, rd);
        let tags = vec![
            if n.ret == 1 { "ok" } else { "fail" },
            if !exists && n.ret == 1 { "created" } else { "" },
        ];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("exists={exists}"),
            tags.into_iter().filter(|t| !t.is_empty()).collect(),
        )
    });

    // `_wfopen_s` — the `_s` shell, including the NULL out-pointer the handler must catch.
    grade("_wfopen_s", &["ok", "fail", "nullout"], ROW_ITERS, |r| {
        let exists = r.below(3) != 0;
        let seed = r.bytes_upto(16);
        let w = if exists { World::new("wfopens", &seed) } else { World::absent("wfopens") };
        let mode = wmode(MODES[r.below(MODES.len())]);
        let nullout = r.below(8) == 0;
        let (sw, rw) = (w.ws(), w.wr());

        let mut sf: FILEP = core::ptr::null_mut();
        let mut rf: FILEP = core::ptr::null_mut();
        let sp = if nullout { core::ptr::null_mut() } else { &mut sf as *mut FILEP };
        let rp = if nullout { core::ptr::null_mut() } else { &mut rf as *mut FILEP };
        let g = Effects::subject(|| _wfopen_s(sp, sw.as_ptr(), mode.as_ptr()) as i64);
        let n = Effects::reference(|| ref__wfopen_s(rp, rw.as_ptr(), mode.as_ptr()) as i64);
        let mut gi = stream_state_subject(sf);
        let mut ni = stream_state_reference(rf);
        if !sf.is_null() {
            fclose(sf);
        }
        if !rf.is_null() {
            ref_fclose(rf);
        }
        let (sd, rd) = w.disk();
        push_disk(&mut gi, sd);
        push_disk(&mut ni, rd);
        let tags = vec![if nullout {
            "nullout"
        } else if n.ret == 0 {
            "ok"
        } else {
            "fail"
        }];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("exists={exists} nullout={nullout}"),
            tags,
        )
    });

    // `_wfsopen` — `_wfopen` plus a share mode, which is the only thing that makes it a
    // separate export. The mode is compared as the `dwShareMode` each side ASKED FOR,
    // read straight back out of the shim; nothing here enforces sharing, and a row that
    // pretended otherwise would be claiming coverage the substrate cannot provide.
    grade("_wfsopen", &["ok", "fail"], ROW_ITERS, |r| {
        let exists = r.below(3) != 0;
        let seed = r.bytes_upto(16);
        let w = if exists { World::new("wfsopen", &seed) } else { World::absent("wfsopen") };
        let mode = wmode(MODES[r.below(MODES.len())]);
        let share = SHARE_MODES[r.below(SHARE_MODES.len())];
        let (sw, rw) = (w.ws(), w.wr());

        let mut sf: FILEP = core::ptr::null_mut();
        let mut rf: FILEP = core::ptr::null_mut();
        let g = Effects::subject(|| {
            sf = _wfsopen(sw.as_ptr(), mode.as_ptr(), share);
            !sf.is_null() as i64
        });
        let sshare = ref_get_last_share_mode();
        let n = Effects::reference(|| {
            rf = ref__wfsopen(rw.as_ptr(), mode.as_ptr(), share);
            !rf.is_null() as i64
        });
        let rshare = ref_get_last_share_mode();
        let mut gi = stream_state_subject(sf);
        let mut ni = stream_state_reference(rf);
        gi.extend_from_slice(&sshare.to_le_bytes());
        ni.extend_from_slice(&rshare.to_le_bytes());
        if !sf.is_null() {
            fclose(sf);
        }
        if !rf.is_null() {
            ref_fclose(rf);
        }
        let (sd, rd) = w.disk();
        push_disk(&mut gi, sd);
        push_disk(&mut ni, rd);
        let tags = vec![if n.ret == 1 { "ok" } else { "fail" }];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("exists={exists} share={share:#x}"),
            tags,
        )
    });

    // `_wfreopen` — the interesting part is the FAILURE path: msvcrt closes the incoming
    // stream whether or not the new path opens, so a row that only tested success would
    // leave the "did it free the FILE" question — and with it every later row's stream
    // numbering — completely unexamined.
    grade("_wfreopen", &["ok", "fail"], ROW_ITERS, |r| {
        let seed = r.bytes_from(1, 16);
        let w = World::new("wfreopA", &seed);
        let dest_exists = r.below(3) != 0;
        let d = if dest_exists {
            World::new("wfreopB", &seed)
        } else {
            World::absent("wfreopB")
        };
        let mode = wmode(MODES[r.below(MODES.len())]);
        let (dw_s, dw_r) = (d.ws(), d.wr());

        let sf0 = fopen(w.s(), b"rb\0".as_ptr());
        let rf0 = ref_fopen(w.r(), b"rb\0".as_ptr());
        let mut sf: FILEP = core::ptr::null_mut();
        let mut rf: FILEP = core::ptr::null_mut();
        let g = Effects::subject(|| {
            sf = _wfreopen(dw_s.as_ptr(), mode.as_ptr(), sf0);
            !sf.is_null() as i64
        });
        let n = Effects::reference(|| {
            rf = ref__wfreopen(dw_r.as_ptr(), mode.as_ptr(), rf0);
            !rf.is_null() as i64
        });
        let mut gi = stream_state_subject(sf);
        let mut ni = stream_state_reference(rf);
        if !sf.is_null() {
            fclose(sf);
        }
        if !rf.is_null() {
            ref_fclose(rf);
        }
        let (sd, rd) = d.disk();
        push_disk(&mut gi, sd);
        push_disk(&mut ni, rd);
        let tags = vec![if n.ret == 1 { "ok" } else { "fail" }];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("dest_exists={dest_exists}"),
            tags,
        )
    });

    // `_wfreopen_s` — four NULL guards, each of which must fire the handler and return
    // EINVAL rather than dereference.
    grade("_wfreopen_s", &["ok", "fail", "guard"], ROW_ITERS, |r| {
        let seed = r.bytes_from(1, 16);
        let w = World::new("wfreopsA", &seed);
        // See the narrow twin: an absent destination opened for READING is the only route
        // into the failure arm, because every "w"/"a" mode creates it.
        let dest_exists = r.below(3) != 0;
        let d =
            if dest_exists { World::new("wfreopsB", &seed) } else { World::absent("wfreopsB") };
        let mode = wmode(if dest_exists { MODES[r.below(MODES.len())] } else { b"rb\0" });
        // 0 = no guard, 1 = NULL out, 2 = NULL path, 3 = NULL mode, 4 = NULL stream.
        let guard = if r.below(3) == 0 { 1 + r.below(4) } else { 0 };
        let (dw_s, dw_r) = (d.ws(), d.wr());
        let null_w: *const u16 = core::ptr::null();

        let sf0 = fopen(w.s(), b"rb\0".as_ptr());
        let rf0 = ref_fopen(w.r(), b"rb\0".as_ptr());
        let mut sf: FILEP = core::ptr::null_mut();
        let mut rf: FILEP = core::ptr::null_mut();
        let sp = if guard == 1 { core::ptr::null_mut() } else { &mut sf as *mut FILEP };
        let rp = if guard == 1 { core::ptr::null_mut() } else { &mut rf as *mut FILEP };
        let (spath, rpath) = if guard == 2 {
            (null_w, null_w)
        } else {
            (dw_s.as_ptr(), dw_r.as_ptr())
        };
        let pmode = if guard == 3 { null_w } else { mode.as_ptr() };
        let (sin, rin) = if guard == 4 {
            (core::ptr::null_mut(), core::ptr::null_mut())
        } else {
            (sf0, rf0)
        };

        let g = Effects::subject(|| _wfreopen_s(sp, spath, pmode, sin) as i64);
        let n = Effects::reference(|| ref__wfreopen_s(rp, rpath, pmode, rin) as i64);
        let mut gi = stream_state_subject(sf);
        let mut ni = stream_state_reference(rf);
        // A guard that fired left the INCOMING streams open — nothing consumed them.
        if guard != 0 {
            if !sf0.is_null() {
                fclose(sf0);
            }
            if !rf0.is_null() {
                ref_fclose(rf0);
            }
        }
        if !sf.is_null() {
            fclose(sf);
        }
        if !rf.is_null() {
            ref_fclose(rf);
        }
        let (sd, rd) = d.disk();
        push_disk(&mut gi, sd);
        push_disk(&mut ni, rd);
        let tags = vec![if guard != 0 {
            "guard"
        } else if n.ret == 0 {
            "ok"
        } else {
            "fail"
        }];
        (Obs::with(g, gi), Obs::with(n, ni), format!("guard={guard}"), tags)
    });

    // `_wsopen_s` — the whole oflag matrix through the SECURE dispatcher, whose two extra
    // guards (NULL fd out-pointer; a `pmode` carrying bits outside `_S_IREAD|_S_IWRITE`
    // when `_O_CREAT` is set) exist only on this path and are unreachable from `_wopen`.
    grade("_wsopen_s", &["ok", "fail", "nullfd", "badpmode"], ROW_ITERS, |r| {
        let exists = r.below(4) != 0;
        let seed = r.bytes_upto(16);
        let w = if exists { World::new("wsopens", &seed) } else { World::absent("wsopens") };
        let flags = gen_oflags(r) | (if r.below(6) == 0 { O_EXCL } else { 0 });
        let share = SHARE_MODES[r.below(SHARE_MODES.len())];
        let nullfd = r.below(10) == 0;
        let badpmode = r.below(6) == 0;
        let pmode = if badpmode { S_IREAD | 0x4000 } else { S_IREAD | S_IWRITE };
        let (sw, rw) = (w.ws(), w.wr());

        let (mut sfd, mut rfd) = (-1, -1);
        let sp = if nullfd { core::ptr::null_mut() } else { &mut sfd as *mut c_int };
        let rp = if nullfd { core::ptr::null_mut() } else { &mut rfd as *mut c_int };
        let g = Effects::subject(|| _wsopen_s(sp, sw.as_ptr(), flags, share, pmode) as i64);
        let sshare = ref_get_last_share_mode();
        let n =
            Effects::reference(|| ref__wsopen_s(rp, rw.as_ptr(), flags, share, pmode) as i64);
        let rshare = ref_get_last_share_mode();
        let mut gi = fd_state_subject(sfd);
        let mut ni = fd_state_reference(rfd);
        gi.extend_from_slice(&sshare.to_le_bytes());
        ni.extend_from_slice(&rshare.to_le_bytes());
        if sfd >= 0 {
            _close(sfd);
        }
        if rfd >= 0 {
            ref__close(rfd);
        }
        let (sd, rd) = w.disk();
        push_disk(&mut gi, sd);
        push_disk(&mut ni, rd);
        let tags = vec![
            if nullfd {
                "nullfd"
            } else if badpmode && (flags & O_CREAT) != 0 {
                "badpmode"
            } else if n.ret == 0 {
                "ok"
            } else {
                "fail"
            },
        ];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("exists={exists} flags={flags:#x} pmode={pmode:#x} nullfd={nullfd}"),
            tags,
        )
    });

    // `_wcreat` — `_wopen` with `_O_CREAT|_O_TRUNC|_O_RDWR` baked in. The truncation is
    // the observable part: over an existing file the contents must be GONE.
    grade("_wcreat", &["created", "truncated"], ROW_ITERS, |r| {
        let exists = r.below(2) == 0;
        let seed = r.bytes_from(1, 16);
        let w = if exists { World::new("wcreat", &seed) } else { World::absent("wcreat") };
        let pmode = if r.below(2) == 0 { S_IREAD | S_IWRITE } else { S_IWRITE };
        let (sw, rw) = (w.ws(), w.wr());

        let g = Effects::subject(|| _wcreat(sw.as_ptr(), pmode) as i64);
        let n = Effects::reference(|| ref__wcreat(rw.as_ptr(), pmode) as i64);
        let (mut gi, mut ni) = (fd_state_subject(g.ret as c_int), fd_state_reference(n.ret as c_int));
        if g.ret >= 0 {
            _close(g.ret as c_int);
        }
        if n.ret >= 0 {
            ref__close(n.ret as c_int);
        }
        // _wcreat can leave the file read-only; make it removable again on every host.
        make_writable(&[&w.sp, &w.rp]);
        let (sd, rd) = w.disk();
        push_disk(&mut gi, sd);
        push_disk(&mut ni, rd);
        let tags = vec![if exists { "truncated" } else { "created" }];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("exists={exists} pmode={pmode:#x}"),
            tags,
        )
    });

    // `_wfdopen` — wrap an existing descriptor in a FILE. The failure arm matters for the
    // bookkeeping as much as for the answer: on failure the caller still owns the fd, and
    // a row that closed it only on success would drift the two tables apart.
    grade("_wfdopen", &["ok", "badfd", "badmode"], ROW_ITERS, |r| {
        let seed = r.bytes_from(1, 16);
        let w = World::new("wfdopen", &seed);
        let bad = r.below(6) == 0;
        let badmode = r.below(6) == 0;
        let mode = if badmode { wmode(b"q\0") } else { wmode(MODES[r.below(MODES.len())]) };

        let (sfd, rfd) = if bad {
            (4096, 4096)
        } else {
            (_open(w.s(), O_RDWR | O_BINARY), ref__open(w.r(), O_RDWR | O_BINARY))
        };
        let mut sf: FILEP = core::ptr::null_mut();
        let mut rf: FILEP = core::ptr::null_mut();
        let g = Effects::subject(|| {
            sf = _wfdopen(sfd, mode.as_ptr());
            !sf.is_null() as i64
        });
        let n = Effects::reference(|| {
            rf = ref__wfdopen(rfd, mode.as_ptr());
            !rf.is_null() as i64
        });
        let gi = stream_state_subject(sf);
        let ni = stream_state_reference(rf);
        // fclose consumes the fd; if the wrap failed the fd is still ours to close.
        if !sf.is_null() {
            fclose(sf);
        } else if !bad {
            _close(sfd);
        }
        if !rf.is_null() {
            ref_fclose(rf);
        } else if !bad {
            ref__close(rfd);
        }
        let tags = vec![if bad {
            "badfd"
        } else if badmode {
            "badmode"
        } else {
            "ok"
        }];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("bad={bad} badmode={badmode}"),
            tags,
        )
    });

    // `_wremove` / `_wunlink` — `_wremove` IS `_wunlink` in file.c, graded separately so
    // that staying the same answer is a checked property rather than an assumption.
    for (name, subj, refn) in [
        (
            "_wremove",
            _wremove as unsafe extern "C" fn(*const u16) -> c_int,
            ref__wremove as unsafe extern "C" fn(*const u16) -> c_int,
        ),
        ("_wunlink", _wunlink, ref__wunlink),
    ] {
        grade(name, &["ok", "enoent"], ROW_ITERS, |r| {
            let exists = r.below(3) != 0;
            let seed = r.bytes_upto(16);
            let w = if exists { World::new("wrm", &seed) } else { World::absent("wrm") };
            let (sw, rw) = (w.ws(), w.wr());
            let g = Effects::subject(|| subj(sw.as_ptr()) as i64);
            let n = Effects::reference(|| refn(rw.as_ptr()) as i64);
            let (sd, rd) = w.disk();
            let (mut gi, mut ni) = (Vec::new(), Vec::new());
            push_disk(&mut gi, sd);
            push_disk(&mut ni, rd);
            let tags = vec![if n.ret == 0 { "ok" } else { "enoent" }];
            (Obs::with(g, gi), Obs::with(n, ni), format!("exists={exists}"), tags)
        });
    }

    // `_wrename` — same three cases as the narrow `rename`, including the destination
    // that already exists, which Win32 refuses where POSIX would overwrite.
    grade("_wrename", &["ok", "enoent", "dest-exists"], ROW_ITERS, |r| {
        let exists = r.below(4) != 0;
        let dest_exists = r.below(3) == 0;
        let seed = r.bytes_upto(16);
        let w = if exists { World::new("wren", &seed) } else { World::absent("wren") };
        let d = World::new("wrendst", if dest_exists { b"x" } else { b"" });
        if !dest_exists {
            let _ = std::fs::remove_file(&d.sp);
            let _ = std::fs::remove_file(&d.rp);
        }
        let (sw, rw, dsw, drw) = (w.ws(), w.wr(), d.ws(), d.wr());

        let g = Effects::subject(|| _wrename(sw.as_ptr(), dsw.as_ptr()) as i64);
        let n = Effects::reference(|| ref__wrename(rw.as_ptr(), drw.as_ptr()) as i64);
        let (sd, rd) = w.disk();
        let (sd2, rd2) = d.disk();
        let (mut gi, mut ni) = (Vec::new(), Vec::new());
        push_disk(&mut gi, sd);
        push_disk(&mut gi, sd2);
        push_disk(&mut ni, rd);
        push_disk(&mut ni, rd2);
        let tags = vec![
            if n.ret == 0 { "ok" } else { "enoent" },
            if dest_exists { "dest-exists" } else { "" },
        ];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("exists={exists} dest_exists={dest_exists}"),
            tags.into_iter().filter(|t| !t.is_empty()).collect(),
        )
    });

    // `_waccess` — the wide `_access`. This is the body; the narrow one delegates here.
    grade("_waccess", &["ok", "enoent", "readonly"], ROW_ITERS, |r| {
        let exists = r.below(3) != 0;
        let seed = r.bytes_upto(8);
        let w = if exists { World::new("wacc", &seed) } else { World::absent("wacc") };
        let readonly = exists && r.below(3) == 0;
        if readonly {
            set_readonly(&[&w.sp, &w.rp]);
        }
        let mode = ACCESS_MODES[r.below(ACCESS_MODES.len())];
        let (sw, rw) = (w.ws(), w.wr());
        let g = Effects::subject(|| _waccess(sw.as_ptr(), mode) as i64);
        let n = Effects::reference(|| ref__waccess(rw.as_ptr(), mode) as i64);
        make_writable(&[&w.sp, &w.rp]);
        let tags = vec![
            if n.ret == 0 { "ok" } else { "enoent" },
            if readonly { "readonly" } else { "" },
        ];
        (
            Obs::with(g, Vec::new()),
            Obs::with(n, Vec::new()),
            format!("exists={exists} readonly={readonly} mode={mode}"),
            tags.into_iter().filter(|t| !t.is_empty()).collect(),
        )
    });

    // `_waccess_s` / `_access_s` — two guards each, and both RETURN `*_errno()` rather
    // than a literal, so the row is as much about the errno slot as about the answer.
    grade("_waccess_s", &["ok", "enoent", "nullpath", "badmode"], ROW_ITERS, |r| {
        let exists = r.below(3) != 0;
        let seed = r.bytes_upto(8);
        let w = if exists { World::new("waccs", &seed) } else { World::absent("waccs") };
        let nullpath = r.below(8) == 0;
        let badmode = r.below(6) == 0;
        let mode = if badmode { 1 } else { ACCESS_MODES[r.below(ACCESS_MODES.len())] };
        let (sw, rw) = (w.ws(), w.wr());
        let null_w: *const u16 = core::ptr::null();
        let (sp, rp) = if nullpath { (null_w, null_w) } else { (sw.as_ptr(), rw.as_ptr()) };
        let g = Effects::subject(|| _waccess_s(sp, mode) as i64);
        let n = Effects::reference(|| ref__waccess_s(rp, mode) as i64);
        let tags = vec![if nullpath {
            "nullpath"
        } else if badmode {
            "badmode"
        } else if n.ret == 0 {
            "ok"
        } else {
            "enoent"
        }];
        (
            Obs::with(g, Vec::new()),
            Obs::with(n, Vec::new()),
            format!("exists={exists} nullpath={nullpath} mode={mode}"),
            tags,
        )
    });

    // `_wchmod` — the only row whose answer lives in the filesystem's permission bits
    // rather than in a return value, so it is read back through `std::fs`, out of process,
    // exactly like the file contents everywhere else in this suite.
    grade("_wchmod", &["readonly", "writable", "enoent"], ROW_ITERS, |r| {
        let exists = r.below(4) != 0;
        let seed = r.bytes_upto(8);
        let w = if exists { World::new("wchmod", &seed) } else { World::absent("wchmod") };
        let readonly = r.below(2) == 0;
        let flags = if readonly { S_IREAD } else { S_IREAD | S_IWRITE };
        let (sw, rw) = (w.ws(), w.wr());

        let g = Effects::subject(|| _wchmod(sw.as_ptr(), flags) as i64);
        let n = Effects::reference(|| ref__wchmod(rw.as_ptr(), flags) as i64);
        let (sro, rro) = w.readonly();
        let (mut gi, mut ni) = (Vec::new(), Vec::new());
        gi.push(ro_byte(sro));
        ni.push(ro_byte(rro));
        make_writable(&[&w.sp, &w.rp]);
        let tags = vec![if !exists {
            "enoent"
        } else if readonly {
            "readonly"
        } else {
            "writable"
        }];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("exists={exists} readonly={readonly}"),
            tags,
        )
    });
}

/// Read-only-ness as one image byte. `None` (no such file) gets a marker distinct from
/// both booleans — "the call deleted it" and "the call left it writable" are different
/// answers.
fn ro_byte(v: Option<bool>) -> u8 {
    match v {
        None => 0xDE,
        Some(true) => 1,
        Some(false) => 0,
    }
}

/// Clear the read-only bit so the fixture can be removed. `_wcreat`/`_chmod` rows can
/// leave a file the harness would otherwise leak on hosts where the bit blocks removal.
fn make_writable(paths: &[&PathBuf]) {
    for p in paths {
        if let Ok(m) = std::fs::metadata(p) {
            let mut perm = m.permissions();
            perm.set_readonly(false);
            let _ = std::fs::set_permissions(p, perm);
        }
    }
}

fn set_readonly(paths: &[&PathBuf]) {
    for p in paths {
        if let Ok(m) = std::fs::metadata(p) {
            let mut perm = m.permissions();
            perm.set_readonly(true);
            let _ = std::fs::set_permissions(p, perm);
        }
    }
}

// ══════════════════════════════════════════════════════════════════════════════════════
// Wave 18b — the narrow openers wave 18a did not reach
// ══════════════════════════════════════════════════════════════════════════════════════
//
// These are the `A` twins of the section above. Each one converts its path and delegates
// to the wide body already graded, so what these rows add is exactly the conversion and
// the delegation — which is where a narrow-only bug would have to live.

unsafe fn narrow_open_rows() {
    // `_fsopen` — `fopen` plus a share mode.
    grade("_fsopen", &["ok", "fail"], ROW_ITERS, |r| {
        let exists = r.below(3) != 0;
        let seed = r.bytes_upto(16);
        let w = if exists { World::new("fsopen", &seed) } else { World::absent("fsopen") };
        let mode = MODES[r.below(MODES.len())];
        let share = SHARE_MODES[r.below(SHARE_MODES.len())];

        let mut sf: FILEP = core::ptr::null_mut();
        let mut rf: FILEP = core::ptr::null_mut();
        let g = Effects::subject(|| {
            sf = _fsopen(w.s(), mode.as_ptr(), share);
            !sf.is_null() as i64
        });
        let sshare = ref_get_last_share_mode();
        let n = Effects::reference(|| {
            rf = ref__fsopen(w.r(), mode.as_ptr(), share);
            !rf.is_null() as i64
        });
        let rshare = ref_get_last_share_mode();
        let mut gi = stream_state_subject(sf);
        let mut ni = stream_state_reference(rf);
        gi.extend_from_slice(&sshare.to_le_bytes());
        ni.extend_from_slice(&rshare.to_le_bytes());
        if !sf.is_null() {
            fclose(sf);
        }
        if !rf.is_null() {
            ref_fclose(rf);
        }
        let (sd, rd) = w.disk();
        push_disk(&mut gi, sd);
        push_disk(&mut ni, rd);
        let tags = vec![if n.ret == 1 { "ok" } else { "fail" }];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("exists={exists} share={share:#x}"),
            tags,
        )
    });

    // `freopen` — including the failure arm, which still consumes the incoming stream.
    grade("freopen", &["ok", "fail"], ROW_ITERS, |r| {
        let seed = r.bytes_from(1, 16);
        let w = World::new("freopA", &seed);
        let dest_exists = r.below(3) != 0;
        let d = if dest_exists { World::new("freopB", &seed) } else { World::absent("freopB") };
        let mode = MODES[r.below(MODES.len())];

        let sf0 = fopen(w.s(), b"rb\0".as_ptr());
        let rf0 = ref_fopen(w.r(), b"rb\0".as_ptr());
        let mut sf: FILEP = core::ptr::null_mut();
        let mut rf: FILEP = core::ptr::null_mut();
        let g = Effects::subject(|| {
            sf = freopen(d.s(), mode.as_ptr(), sf0);
            !sf.is_null() as i64
        });
        let n = Effects::reference(|| {
            rf = ref_freopen(d.r(), mode.as_ptr(), rf0);
            !rf.is_null() as i64
        });
        let mut gi = stream_state_subject(sf);
        let mut ni = stream_state_reference(rf);
        if !sf.is_null() {
            fclose(sf);
        }
        if !rf.is_null() {
            ref_fclose(rf);
        }
        let (sd, rd) = d.disk();
        push_disk(&mut gi, sd);
        push_disk(&mut ni, rd);
        let tags = vec![if n.ret == 1 { "ok" } else { "fail" }];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("dest_exists={dest_exists}"),
            tags,
        )
    });

    // `freopen_s` — the four NULL guards.
    grade("freopen_s", &["ok", "fail", "guard"], ROW_ITERS, |r| {
        let seed = r.bytes_from(1, 16);
        let w = World::new("freopsA", &seed);
        // A destination that does not exist, opened for READING, is the only way into the
        // failure arm — every "w"/"a" mode creates it. Without both halves the row's
        // `fail` tag names a branch the fuzz cannot enter, which is an UNEXERCISED and
        // not a pass.
        let dest_exists = r.below(3) != 0;
        let d = if dest_exists { World::new("freopsB", &seed) } else { World::absent("freopsB") };
        let mode: &[u8] = if dest_exists { MODES[r.below(MODES.len())] } else { b"rb\0" };
        let guard = if r.below(3) == 0 { 1 + r.below(4) } else { 0 };
        let null_a: *const u8 = core::ptr::null();

        let sf0 = fopen(w.s(), b"rb\0".as_ptr());
        let rf0 = ref_fopen(w.r(), b"rb\0".as_ptr());
        let mut sf: FILEP = core::ptr::null_mut();
        let mut rf: FILEP = core::ptr::null_mut();
        let sp = if guard == 1 { core::ptr::null_mut() } else { &mut sf as *mut FILEP };
        let rp = if guard == 1 { core::ptr::null_mut() } else { &mut rf as *mut FILEP };
        let (spath, rpath) = if guard == 2 { (null_a, null_a) } else { (d.s(), d.r()) };
        let pmode = if guard == 3 { null_a } else { mode.as_ptr() };
        let (sin, rin) = if guard == 4 {
            (core::ptr::null_mut(), core::ptr::null_mut())
        } else {
            (sf0, rf0)
        };

        let g = Effects::subject(|| freopen_s(sp, spath, pmode, sin) as i64);
        let n = Effects::reference(|| ref_freopen_s(rp, rpath, pmode, rin) as i64);
        let mut gi = stream_state_subject(sf);
        let mut ni = stream_state_reference(rf);
        if guard != 0 {
            if !sf0.is_null() {
                fclose(sf0);
            }
            if !rf0.is_null() {
                ref_fclose(rf0);
            }
        }
        if !sf.is_null() {
            fclose(sf);
        }
        if !rf.is_null() {
            ref_fclose(rf);
        }
        let (sd, rd) = d.disk();
        push_disk(&mut gi, sd);
        push_disk(&mut ni, rd);
        let tags = vec![if guard != 0 {
            "guard"
        } else if n.ret == 0 {
            "ok"
        } else {
            "fail"
        }];
        (Obs::with(g, gi), Obs::with(n, ni), format!("guard={guard}"), tags)
    });

    // `_sopen_s` — the secure dispatcher's two extra guards, narrow side.
    grade("_sopen_s", &["ok", "fail", "nullfd", "badpmode"], ROW_ITERS, |r| {
        let exists = r.below(4) != 0;
        let seed = r.bytes_upto(16);
        let w = if exists { World::new("sopens", &seed) } else { World::absent("sopens") };
        let flags = gen_oflags(r) | (if r.below(6) == 0 { O_EXCL } else { 0 });
        let share = SHARE_MODES[r.below(SHARE_MODES.len())];
        let nullfd = r.below(10) == 0;
        let badpmode = r.below(6) == 0;
        let pmode = if badpmode { S_IREAD | 0x4000 } else { S_IREAD | S_IWRITE };

        let (mut sfd, mut rfd) = (-1, -1);
        let sp = if nullfd { core::ptr::null_mut() } else { &mut sfd as *mut c_int };
        let rp = if nullfd { core::ptr::null_mut() } else { &mut rfd as *mut c_int };
        let g = Effects::subject(|| _sopen_s(sp, w.s(), flags, share, pmode) as i64);
        let sshare = ref_get_last_share_mode();
        let n = Effects::reference(|| ref__sopen_s(rp, w.r(), flags, share, pmode) as i64);
        let rshare = ref_get_last_share_mode();
        let mut gi = fd_state_subject(sfd);
        let mut ni = fd_state_reference(rfd);
        gi.extend_from_slice(&sshare.to_le_bytes());
        ni.extend_from_slice(&rshare.to_le_bytes());
        if sfd >= 0 {
            _close(sfd);
        }
        if rfd >= 0 {
            ref__close(rfd);
        }
        let (sd, rd) = w.disk();
        push_disk(&mut gi, sd);
        push_disk(&mut ni, rd);
        let tags = vec![
            if nullfd {
                "nullfd"
            } else if badpmode && (flags & O_CREAT) != 0 {
                "badpmode"
            } else if n.ret == 0 {
                "ok"
            } else {
                "fail"
            },
        ];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("exists={exists} flags={flags:#x} nullfd={nullfd}"),
            tags,
        )
    });

    // `_creat` — truncation over an existing file is the observable part.
    grade("_creat", &["created", "truncated"], ROW_ITERS, |r| {
        let exists = r.below(2) == 0;
        let seed = r.bytes_from(1, 16);
        let w = if exists { World::new("creat", &seed) } else { World::absent("creat") };
        let pmode = if r.below(2) == 0 { S_IREAD | S_IWRITE } else { S_IWRITE };

        let g = Effects::subject(|| _creat(w.s(), pmode) as i64);
        let n = Effects::reference(|| ref__creat(w.r(), pmode) as i64);
        let (mut gi, mut ni) =
            (fd_state_subject(g.ret as c_int), fd_state_reference(n.ret as c_int));
        if g.ret >= 0 {
            _close(g.ret as c_int);
        }
        if n.ret >= 0 {
            ref__close(n.ret as c_int);
        }
        make_writable(&[&w.sp, &w.rp]);
        let (sd, rd) = w.disk();
        push_disk(&mut gi, sd);
        push_disk(&mut ni, rd);
        let tags = vec![if exists { "truncated" } else { "created" }];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("exists={exists} pmode={pmode:#x}"),
            tags,
        )
    });

    // `_chmod` — read back out of process, like its wide twin.
    grade("_chmod", &["readonly", "writable", "enoent"], ROW_ITERS, |r| {
        let exists = r.below(4) != 0;
        let seed = r.bytes_upto(8);
        let w = if exists { World::new("chmod", &seed) } else { World::absent("chmod") };
        let readonly = r.below(2) == 0;
        let flags = if readonly { S_IREAD } else { S_IREAD | S_IWRITE };

        let g = Effects::subject(|| _chmod(w.s(), flags) as i64);
        let n = Effects::reference(|| ref__chmod(w.r(), flags) as i64);
        let (sro, rro) = w.readonly();
        let (mut gi, mut ni) = (Vec::new(), Vec::new());
        gi.push(ro_byte(sro));
        ni.push(ro_byte(rro));
        make_writable(&[&w.sp, &w.rp]);
        let tags = vec![if !exists {
            "enoent"
        } else if readonly {
            "readonly"
        } else {
            "writable"
        }];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("exists={exists} readonly={readonly}"),
            tags,
        )
    });

    // `_access_s` — the narrow twin of the `_waccess_s` guards.
    grade("_access_s", &["ok", "enoent", "nullpath", "badmode"], ROW_ITERS, |r| {
        let exists = r.below(3) != 0;
        let seed = r.bytes_upto(8);
        let w = if exists { World::new("accs", &seed) } else { World::absent("accs") };
        let nullpath = r.below(8) == 0;
        let badmode = r.below(6) == 0;
        let mode = if badmode { 1 } else { ACCESS_MODES[r.below(ACCESS_MODES.len())] };
        let null_a: *const u8 = core::ptr::null();
        let (sp, rp) = if nullpath { (null_a, null_a) } else { (w.s(), w.r()) };
        let g = Effects::subject(|| _access_s(sp, mode) as i64);
        let n = Effects::reference(|| ref__access_s(rp, mode) as i64);
        let tags = vec![if nullpath {
            "nullpath"
        } else if badmode {
            "badmode"
        } else if n.ret == 0 {
            "ok"
        } else {
            "enoent"
        }];
        (
            Obs::with(g, Vec::new()),
            Obs::with(n, Vec::new()),
            format!("exists={exists} nullpath={nullpath} mode={mode}"),
            tags,
        )
    });

    // `_fdopen` — narrow twin of `_wfdopen`, same fd-ownership rule on the failure arm.
    grade("_fdopen", &["ok", "badfd", "badmode"], ROW_ITERS, |r| {
        let seed = r.bytes_from(1, 16);
        let w = World::new("fdopen", &seed);
        let bad = r.below(6) == 0;
        let badmode = r.below(6) == 0;
        let mode: &[u8] = if badmode { b"q\0" } else { MODES[r.below(MODES.len())] };

        let (sfd, rfd) = if bad {
            (4096, 4096)
        } else {
            (_open(w.s(), O_RDWR | O_BINARY), ref__open(w.r(), O_RDWR | O_BINARY))
        };
        let mut sf: FILEP = core::ptr::null_mut();
        let mut rf: FILEP = core::ptr::null_mut();
        let g = Effects::subject(|| {
            sf = _fdopen(sfd, mode.as_ptr());
            !sf.is_null() as i64
        });
        let n = Effects::reference(|| {
            rf = ref__fdopen(rfd, mode.as_ptr());
            !rf.is_null() as i64
        });
        let gi = stream_state_subject(sf);
        let ni = stream_state_reference(rf);
        if !sf.is_null() {
            fclose(sf);
        } else if !bad {
            _close(sfd);
        }
        if !rf.is_null() {
            ref_fclose(rf);
        } else if !bad {
            ref__close(rfd);
        }
        let tags = vec![if bad {
            "badfd"
        } else if badmode {
            "badmode"
        } else {
            "ok"
        }];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("bad={bad} badmode={badmode}"),
            tags,
        )
    });
}

// ══════════════════════════════════════════════════════════════════════════════════════
// Wave 18b — the stat family
// ══════════════════════════════════════════════════════════════════════════════════════
//
// Nine exports over four structs that differ only in the width of `st_size` and the three
// timestamps. That is the whole bug surface: every one of them is `_wstat64` plus a field
// narrowing (`msvcrt_stat64_to_stat`, `…_to_stat32i64`, `…_to_stat64i32`, `…_to_stati64`),
// and a narrowing that truncates the wrong field, or writes one field too many, produces
// a struct that is right in the fields anyone casually checks and wrong in the rest.
//
// So the rows compare the buffer as BYTES, guards included, and deliberately over-allocate
// past the struct: a converter that writes `_stat64`'s 56 bytes into a `_stat` is then a
// divergence in the trailing poison rather than an invisible overrun of the caller's stack.
//
// Both sides run over ONE file (see `Shared`) — the timestamps are the point of the struct
// and two files do not have the same ones.

/// Generous enough for every variant, and the slack is compared: it is where an
/// over-wide write lands.
const STAT_BUF: usize = 128;

/// A guarded destination buffer with 8-byte alignment.
///
/// `Buf` is a `Vec<u8>`, whose allocation is 1-aligned, and every stat struct here carries
/// `__int64` fields. Handing the C an interior pointer into a `Vec<u8>` would have it
/// perform unaligned 64-bit stores — which happens to work on x86-64 and is undefined
/// everywhere, including here. Backing this one with `Vec<u64>` makes the alignment right
/// by construction, and the leading and trailing `u64`s are the poison guards.
struct QBuf(Vec<u64>);

impl QBuf {
    fn new(n: usize) -> QBuf {
        QBuf(vec![0xA7A7_A7A7_A7A7_A7A7; n.div_ceil(8) + 2])
    }
    fn p(&mut self) -> *mut c_void {
        unsafe { (self.0.as_mut_ptr() as *mut u8).add(8) as *mut c_void }
    }
    fn img(&self) -> Vec<u8> {
        self.0.iter().flat_map(|w| w.to_le_bytes()).collect()
    }
}

unsafe fn stat_rows() {
    // ── the path-taking narrow trio ───────────────────────────────────────────────────
    for (name, subj, refn) in [
        (
            "_stat64",
            _stat64 as unsafe extern "C" fn(*const u8, *mut c_void) -> c_int,
            ref__stat64 as unsafe extern "C" fn(*const u8, *mut c_void) -> c_int,
        ),
        ("_stat", _stat, ref__stat),
        ("_stati64", _stati64, ref__stati64),
    ] {
        grade(name, &["ok", "enoent", "empty"], ROW_ITERS, |r| {
            let exists = r.below(4) != 0;
            let seed = r.bytes_upto(64);
            let f = if exists {
                Shared::new("stat", &seed)
            } else {
                Shared::absent("stat")
            };
            let (mut sb, mut rb) = (QBuf::new(STAT_BUF), QBuf::new(STAT_BUF));
            let (sp, rp) = (sb.p(), rb.p());
            let g = Effects::subject(|| subj(f.n(), sp) as i64);
            let n = Effects::reference(|| refn(f.n(), rp) as i64);
            let tags = vec![
                if n.ret == 0 { "ok" } else { "enoent" },
                if exists && seed.is_empty() { "empty" } else { "" },
            ];
            (
                Obs::with(g, sb.img()),
                Obs::with(n, rb.img()),
                format!("exists={exists} len={}", seed.len()),
                tags.into_iter().filter(|t| !t.is_empty()).collect(),
            )
        });
    }

    // ── the path-taking wide trio ─────────────────────────────────────────────────────
    for (name, subj, refn) in [
        (
            "_wstat64",
            _wstat64 as unsafe extern "C" fn(*const u16, *mut c_void) -> c_int,
            ref__wstat64 as unsafe extern "C" fn(*const u16, *mut c_void) -> c_int,
        ),
        ("_wstat", _wstat, ref__wstat),
        ("_wstati64", _wstati64, ref__wstati64),
    ] {
        grade(name, &["ok", "enoent", "empty"], ROW_ITERS, |r| {
            let exists = r.below(4) != 0;
            let seed = r.bytes_upto(64);
            let f = if exists {
                Shared::new("wstat", &seed)
            } else {
                Shared::absent("wstat")
            };
            let wp = f.w();
            let (mut sb, mut rb) = (QBuf::new(STAT_BUF), QBuf::new(STAT_BUF));
            let (sp, rp) = (sb.p(), rb.p());
            let g = Effects::subject(|| subj(wp.as_ptr(), sp) as i64);
            let n = Effects::reference(|| refn(wp.as_ptr(), rp) as i64);
            let tags = vec![
                if n.ret == 0 { "ok" } else { "enoent" },
                if exists && seed.is_empty() { "empty" } else { "" },
            ];
            (
                Obs::with(g, sb.img()),
                Obs::with(n, rb.img()),
                format!("exists={exists} len={}", seed.len()),
                tags.into_iter().filter(|t| !t.is_empty()).collect(),
            )
        });
    }

    // ── the fd-taking trio ────────────────────────────────────────────────────────────
    //
    // `_fstat64` reports `st_dev` and `st_rdev` as the DESCRIPTOR, so this row is also the
    // one that would notice the two fd tables drifting — the fd numbers are inside the
    // compared image, not merely alongside it.
    for (name, subj, refn) in [
        (
            "_fstat64",
            _fstat64 as unsafe extern "C" fn(c_int, *mut c_void) -> c_int,
            ref__fstat64 as unsafe extern "C" fn(c_int, *mut c_void) -> c_int,
        ),
        ("_fstat", _fstat, ref__fstat),
        ("_fstati64", _fstati64, ref__fstati64),
    ] {
        grade(name, &["ok", "badfd", "empty"], ROW_ITERS, |r| {
            let seed = r.bytes_upto(64);
            let f = Shared::new("fstat", &seed);
            let bad = r.below(5) == 0;
            let (sfd, rfd) = if bad {
                (4096, 4096)
            } else {
                (_open(f.n(), O_RDONLY | O_BINARY), ref__open(f.n(), O_RDONLY | O_BINARY))
            };
            let (mut sb, mut rb) = (QBuf::new(STAT_BUF), QBuf::new(STAT_BUF));
            let (sp, rp) = (sb.p(), rb.p());
            let g = Effects::subject(|| subj(sfd, sp) as i64);
            let n = Effects::reference(|| refn(rfd, rp) as i64);
            if !bad {
                _close(sfd);
                ref__close(rfd);
            }
            let tags = vec![
                if bad { "badfd" } else { "ok" },
                if !bad && seed.is_empty() { "empty" } else { "" },
            ];
            (
                Obs::with(g, sb.img()),
                Obs::with(n, rb.img()),
                format!("bad={bad} len={}", seed.len()),
                tags.into_iter().filter(|t| !t.is_empty()).collect(),
            )
        });
    }
}

// ══════════════════════════════════════════════════════════════════════════════════════
// Wave 18b — 64-bit positioning
// ══════════════════════════════════════════════════════════════════════════════════════
//
// Wave 18a graded `fseek`/`ftell`/`_lseek`/`_tell`, all of which take and return a 32-bit
// offset. The `i64` twins are separate exports with separate bodies, and the reason they
// exist is precisely the case a 32-bit row cannot construct. The rows below still run over
// small files — creating a 3 GB fixture per case is not a trade this suite should make —
// so what they pin is the CONTRACT (arguments threaded through at full width, the same
// error codes, the same interaction with the buffer), not the >4 GB arithmetic. Said
// plainly rather than left implied, because a green row here is weaker than it looks.

unsafe fn position_rows() {
    // `_fseeki64` + `_ftelli64` — the pair, since a seek is only observable through a tell.
    grade("_fseeki64+_ftelli64", &["ok", "fail", "clears-eof"], ROW_ITERS, |r| {
        let seed = r.bytes_from(1, 40);
        let w = World::new("seek64", &seed);
        let whence = [SEEK_SET, SEEK_CUR, SEEK_END][r.below(3)];
        let off = (r.below(seed.len() + 8) as i64) - 4;
        let drain = r.below(3) == 0;

        let sf = fopen(w.s(), b"rb\0".as_ptr());
        let rf = ref_fopen(w.r(), b"rb\0".as_ptr());
        if drain {
            // Latch EOF first: a seek must clear it, and nothing else in this row would.
            let mut b = [0u8; 64];
            fread(b.as_mut_ptr() as *mut c_void, 1, 64, sf);
            fgetc(sf);
            ref_fread(b.as_mut_ptr() as *mut c_void, 1, 64, rf);
            ref_fgetc(rf);
        }
        let g = Effects::subject(|| _fseeki64(sf, off, whence) as i64);
        let n = Effects::reference(|| ref__fseeki64(rf, off, whence) as i64);
        let mut gi = stream_state_subject(sf);
        let mut ni = stream_state_reference(rf);
        gi.extend_from_slice(&_ftelli64(sf).to_le_bytes());
        ni.extend_from_slice(&ref__ftelli64(rf).to_le_bytes());
        fclose(sf);
        ref_fclose(rf);
        let tags = vec![
            if n.ret == 0 { "ok" } else { "fail" },
            if drain { "clears-eof" } else { "" },
        ];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("len={} off={off} whence={whence} drain={drain}", seed.len()),
            tags.into_iter().filter(|t| !t.is_empty()).collect(),
        )
    });

    // `_telli64` — the fd-layer twin, read after a seek that `_lseeki64` performed.
    grade("_telli64", &["ok", "badfd"], ROW_ITERS, |r| {
        let seed = r.bytes_from(1, 40);
        let w = World::new("telli64", &seed);
        let bad = r.below(5) == 0;
        let skip = r.below(seed.len() + 4) as i64;

        let (sfd, rfd) = if bad {
            (4096, 4096)
        } else {
            (_open(w.s(), O_RDONLY | O_BINARY), ref__open(w.r(), O_RDONLY | O_BINARY))
        };
        if !bad {
            _lseeki64(sfd, skip, SEEK_SET);
            ref__lseeki64(rfd, skip, SEEK_SET);
        }
        let g = Effects::subject(|| _telli64(sfd));
        let n = Effects::reference(|| ref__telli64(rfd));
        if !bad {
            _close(sfd);
            ref__close(rfd);
        }
        let tags = vec![if bad { "badfd" } else { "ok" }];
        (
            Obs::with(g, Vec::new()),
            Obs::with(n, Vec::new()),
            format!("len={} skip={skip} bad={bad}", seed.len()),
            tags,
        )
    });

    // `fgetpos` + `fsetpos` — a save/restore round trip. The `fpos_t` is compared as the
    // OPAQUE VALUE it is: msvcrt defines it as an `__int64` byte offset, and a row that
    // only checked "reading twice from the restored position gives the same bytes" would
    // pass just as well over an implementation that stored something else entirely.
    //
    // NO null-`pos` case, deliberately. This row had one and it segfaulted — in the
    // SUBJECT first, which read like a missing guard until the C said otherwise:
    // msvcrt's `fgetpos` is `*pos = _ftelli64(file);` with nothing in front of it, and
    // `fsetpos` dereferences just as bare. Rust is faithful. A row over a NULL there
    // would be grading a crash against a crash, and the guard it seems to demand would
    // be a DIVERGENCE from the thing this file exists to match.
    grade("fgetpos+fsetpos", &["at-start", "mid-file"], ROW_ITERS, |r| {
        let seed = r.bytes_from(4, 40);
        let w = World::new("fpos", &seed);
        let skip = r.below(seed.len());

        let sf = fopen(w.s(), b"rb\0".as_ptr());
        let rf = ref_fopen(w.r(), b"rb\0".as_ptr());
        fseek(sf, skip as i32, SEEK_SET);
        ref_fseek(rf, skip as i32, SEEK_SET);
        let (mut spos, mut rpos) = (0i64, 0i64);
        let sp = &mut spos as *mut i64;
        let rp = &mut rpos as *mut i64;

        let g = Effects::subject(|| fgetpos(sf, sp) as i64);
        let n = Effects::reference(|| ref_fgetpos(rf, rp) as i64);
        let (mut gi, mut ni) = (Vec::new(), Vec::new());
        gi.extend_from_slice(&spos.to_le_bytes());
        ni.extend_from_slice(&rpos.to_le_bytes());
        // Walk away, then come back and prove we are where we said we were.
        fgetc(sf);
        fgetc(sf);
        ref_fgetc(rf);
        ref_fgetc(rf);
        gi.extend_from_slice(&(fsetpos(sf, sp) as i32).to_le_bytes());
        ni.extend_from_slice(&(ref_fsetpos(rf, rp) as i32).to_le_bytes());
        gi.extend_from_slice(&fgetc(sf).to_le_bytes());
        ni.extend_from_slice(&ref_fgetc(rf).to_le_bytes());
        gi.extend(stream_state_subject(sf));
        ni.extend(stream_state_reference(rf));
        fclose(sf);
        ref_fclose(rf);
        // Tagged by WHERE the position was taken. The obvious pair — "it worked" vs "it
        // failed" — is not available: `fgetpos` fails only when `_ftelli64` returns -1,
        // which this row's streams never do, so a `fail` tag would be an UNEXERCISED
        // forever and a `ok`/`restored` split would be one tag wearing two names.
        let tags = vec![if skip == 0 { "at-start" } else { "mid-file" }];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("len={} skip={skip}", seed.len()),
            tags,
        )
    });
}

// ══════════════════════════════════════════════════════════════════════════════════════
// Wave 18b — descriptor plumbing
// ══════════════════════════════════════════════════════════════════════════════════════

unsafe fn fd_plumbing_rows() {
    // `_dup2` — REPLACES a descriptor rather than allocating one, which is why wave 18a
    // left it out: `balanced()` probes the allocator, and an implementation that got
    // `_dup2` wrong could still allocate correctly. So the row opens two descriptors per
    // side and closes two, keeping the allocator's view identical, and puts BOTH
    // descriptors' state in the compared image — which is where a wrong `_dup2` shows.
    grade("_dup2", &["ok", "badsrc", "baddst"], ROW_ITERS, |r| {
        let a_seed = r.bytes_from(1, 24);
        let b_seed = r.bytes_from(1, 24);
        let wa = World::new("dup2a", &a_seed);
        let wb = World::new("dup2b", &b_seed);
        let badsrc = r.below(6) == 0;
        let baddst = r.below(6) == 0;

        let sa = _open(wa.s(), O_RDONLY | O_BINARY);
        let ra = ref__open(wa.r(), O_RDONLY | O_BINARY);
        let sb = _open(wb.s(), O_RDONLY | O_BINARY);
        let rb = ref__open(wb.r(), O_RDONLY | O_BINARY);
        let (ssrc, rsrc) = if badsrc { (4096, 4096) } else { (sa, ra) };
        let (sdst, rdst) = if baddst { (-1, -1) } else { (sb, rb) };

        let g = Effects::subject(|| _dup2(ssrc, sdst) as i64);
        let n = Effects::reference(|| ref__dup2(rsrc, rdst) as i64);
        let mut gi = fd_state_subject(sa);
        let mut ni = fd_state_reference(ra);
        gi.extend(fd_state_subject(sb));
        ni.extend(fd_state_reference(rb));
        _close(sa);
        _close(sb);
        ref__close(ra);
        ref__close(rb);
        let tags = vec![if badsrc {
            "badsrc"
        } else if baddst {
            "baddst"
        } else {
            "ok"
        }];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("badsrc={badsrc} baddst={baddst}"),
            tags,
        )
    });

    // `_get_osfhandle` + `_open_osfhandle` — the fd↔HANDLE door.
    //
    // The raw handle CANNOT be compared. Each side opens its own descriptor and the shim
    // derives the handle from the host fd, so the two are different numbers by
    // construction and always would be. What is compared is the VALIDITY CLASS, plus
    // everything downstream of it: the fd `_open_osfhandle` hands back for that handle,
    // and that fd's position/length/EOF. A row that compared the handle itself would have
    // to be deleted, not fixed.
    //
    // Note the cleanup: the new fd owns the same handle as the old one, so closing both
    // closes it twice. That is deliberate and symmetric — msvcrt frees the table slot
    // whether or not `CloseHandle` succeeds, and doing it identically on both sides keeps
    // the allocator in step while also grading the second close's errno.
    grade("_get/_open_osfhandle", &["ok", "badfd", "badhandle"], ROW_ITERS, |r| {
        let seed = r.bytes_from(1, 24);
        let w = World::new("osfh", &seed);
        let bad = r.below(6) == 0;
        let badhandle = r.below(6) == 0;
        let oflags = O_RDONLY
            | (if r.below(2) == 0 { O_BINARY } else { O_TEXT })
            | (if r.below(3) == 0 { O_NOINHERIT } else { 0 });

        let (sfd, rfd) = if bad {
            (4096, 4096)
        } else {
            (_open(w.s(), O_RDONLY | O_BINARY), ref__open(w.r(), O_RDONLY | O_BINARY))
        };
        let g = Effects::subject(|| (_get_osfhandle(sfd) == -1) as i64);
        let n = Effects::reference(|| (ref__get_osfhandle(rfd) == -1) as i64);
        let (sh, rh) = if badhandle {
            (-1isize, -1isize)
        } else {
            (_get_osfhandle(sfd), ref__get_osfhandle(rfd))
        };

        let (mut snew, mut rnew) = (-1, -1);
        let g2 = Effects::subject(|| {
            snew = _open_osfhandle(sh, oflags);
            snew as i64
        });
        let n2 = Effects::reference(|| {
            rnew = ref__open_osfhandle(rh, oflags);
            rnew as i64
        });
        let mut gi = fd_state_subject(snew);
        let mut ni = fd_state_reference(rnew);
        gi.extend_from_slice(&(g2.errno).to_le_bytes());
        ni.extend_from_slice(&(n2.errno).to_le_bytes());
        gi.extend_from_slice(&(g2.pmt).to_le_bytes());
        ni.extend_from_slice(&(n2.pmt).to_le_bytes());
        if snew >= 0 {
            _close(snew);
        }
        if rnew >= 0 {
            ref__close(rnew);
        }
        if !bad {
            _close(sfd);
            ref__close(rfd);
        }
        let tags = vec![if bad {
            "badfd"
        } else if badhandle {
            "badhandle"
        } else {
            "ok"
        }];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("bad={bad} badhandle={badhandle} oflags={oflags:#x}"),
            tags,
        )
    });

    // `_pipe` — two descriptors from one call, then a round trip through them. The
    // payload is kept small on purpose: a write larger than the host pipe buffer would
    // block forever with nothing to drain it, and a hung difftest is a worse failure mode
    // than an ungraded byte.
    grade("_pipe", &["ok", "roundtrip"], ROW_ITERS, |r| {
        let payload = r.bytes_from(1, 64);
        let textmode = if r.below(2) == 0 { O_BINARY } else { O_TEXT };
        let size = 512 + r.below(2048) as u32;

        let (mut sfds, mut rfds) = ([-1, -1], [-1, -1]);
        let g = Effects::subject(|| _pipe(sfds.as_mut_ptr(), size, textmode) as i64);
        let n = Effects::reference(|| ref__pipe(rfds.as_mut_ptr(), size, textmode) as i64);
        let (mut gi, mut ni) = (Vec::new(), Vec::new());
        gi.extend_from_slice(&sfds[0].to_le_bytes());
        gi.extend_from_slice(&sfds[1].to_le_bytes());
        ni.extend_from_slice(&rfds[0].to_le_bytes());
        ni.extend_from_slice(&rfds[1].to_le_bytes());
        let mut full_roundtrip = false;
        if g.ret == 0 && n.ret == 0 {
            let (mut sb, mut rb) = (Buf::new(payload.len()), Buf::new(payload.len()));
            let (sp, rp) = (sb.p(), rb.p());
            let sw = _write(sfds[1], payload.as_ptr() as *const c_void, payload.len() as u32);
            let rw = ref__write(rfds[1], payload.as_ptr() as *const c_void, payload.len() as u32);
            let sr = _read(sfds[0], sp as *mut c_void, payload.len() as u32);
            let rr = ref__read(rfds[0], rp as *mut c_void, payload.len() as u32);
            gi.extend_from_slice(&sw.to_le_bytes());
            ni.extend_from_slice(&rw.to_le_bytes());
            gi.extend_from_slice(&sr.to_le_bytes());
            ni.extend_from_slice(&rr.to_le_bytes());
            gi.extend(sb.img());
            ni.extend(rb.img());
            // Every byte written came back. Tagged separately from "the call returned 0"
            // because those are different claims, and a `require` list that spells the
            // same condition twice asserts nothing.
            full_roundtrip = rr == payload.len() as c_int && rw == payload.len() as c_int;
        }
        for fd in sfds {
            if fd >= 0 {
                _close(fd);
            }
        }
        for fd in rfds {
            if fd >= 0 {
                ref__close(fd);
            }
        }
        let tags = vec![
            if n.ret == 0 { "ok" } else { "" },
            if full_roundtrip { "roundtrip" } else { "" },
        ];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("len={} textmode={textmode:#x}", payload.len()),
            tags.into_iter().filter(|t| !t.is_empty()).collect(),
        )
    });

    // `_locking` — byte-range locks.
    //
    // The CONFLICT arm is not reachable from one process: POSIX `fcntl` locks are held per
    // process, so a second overlapping lock from this same process replaces the first
    // instead of failing. The row therefore grades the three arms it CAN reach — a
    // successful lock/unlock, an out-of-range mode, and a closed descriptor — and says so
    // rather than carrying a `require` tag that would fail forever.
    grade("_locking", &["ok", "badfd", "badmode"], ROW_ITERS, |r| {
        let seed = r.bytes_from(8, 40);
        let w = World::new("lock", &seed);
        let bad = r.below(6) == 0;
        let badmode = r.below(6) == 0;
        let mode = if badmode { 7 } else { [LK_UNLCK, LK_LOCK, LK_RLCK][r.below(3)] };
        let nbytes = 1 + r.below(seed.len()) as i32;

        let (sfd, rfd) = if bad {
            (4096, 4096)
        } else {
            (_open(w.s(), O_RDWR | O_BINARY), ref__open(w.r(), O_RDWR | O_BINARY))
        };
        let g = Effects::subject(|| _locking(sfd, mode, nbytes) as i64);
        let n = Effects::reference(|| ref__locking(rfd, mode, nbytes) as i64);
        let (mut gi, mut ni) = (Vec::new(), Vec::new());
        if !bad {
            gi.extend(fd_state_subject(sfd));
            ni.extend(fd_state_reference(rfd));
            // Release whatever was taken; the descriptor close would do it anyway, but
            // relying on that would leave the unlock path itself ungraded.
            _locking(sfd, LK_UNLCK, nbytes);
            ref__locking(rfd, LK_UNLCK, nbytes);
            _close(sfd);
            ref__close(rfd);
        }
        let tags = vec![if bad {
            "badfd"
        } else if badmode {
            "badmode"
        } else {
            "ok"
        }];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("bad={bad} mode={mode} nbytes={nbytes}"),
            tags,
        )
    });

    // `_umask` — a getter/setter over one `static int`, so on its own it is a tautology.
    // What makes the row worth having is the SECOND half: the mask is consulted by
    // `_wsopen_dispatch`, which marks a newly-created file read-only when the umask has
    // taken `_S_IWRITE` out of the mode. So the row sets the mask, creates a file through
    // it, and reads the resulting permission back out of process.
    grade("_umask", &["masked", "unmasked"], ROW_ITERS, |r| {
        let masked = r.below(2) == 0;
        let mask = if masked { S_IWRITE } else { 0 };
        let w = World::absent("umask");

        let g = Effects::subject(|| _umask(mask) as i64);
        let n = Effects::reference(|| ref__umask(mask) as i64);
        let sfd = _open(w.s(), O_CREAT | O_WRONLY | O_BINARY, S_IREAD | S_IWRITE);
        let rfd = ref__open(w.r(), O_CREAT | O_WRONLY | O_BINARY, S_IREAD | S_IWRITE);
        if sfd >= 0 {
            _close(sfd);
        }
        if rfd >= 0 {
            ref__close(rfd);
        }
        let (sro, rro) = w.readonly();
        let (mut gi, mut ni) = (Vec::new(), Vec::new());
        gi.push(ro_byte(sro));
        ni.push(ro_byte(rro));
        // Put the mask back before the next row inherits it.
        _umask(0);
        ref__umask(0);
        make_writable(&[&w.sp, &w.rp]);
        let tags = vec![if masked { "masked" } else { "unmasked" }];
        (Obs::with(g, gi), Obs::with(n, ni), format!("mask={mask:#x}"), tags)
    });

    // `_chsize_s` — the 64-bit `_chsize`, plus the negative-size guard that only exists
    // on this variant.
    grade("_chsize_s", &["grow", "shrink", "badfd", "negative"], ROW_ITERS, |r| {
        let seed = r.bytes_from(1, 32);
        let w = World::new("chsize_s", &seed);
        let bad = r.below(6) == 0;
        let negative = r.below(6) == 0;
        let size = if negative { -1 } else { r.below(seed.len() * 2 + 8) as i64 };

        let (sfd, rfd) = if bad {
            (4096, 4096)
        } else {
            (_open(w.s(), O_RDWR | O_BINARY), ref__open(w.r(), O_RDWR | O_BINARY))
        };
        let g = Effects::subject(|| _chsize_s(sfd, size) as i64);
        let n = Effects::reference(|| ref__chsize_s(rfd, size) as i64);
        let (mut gi, mut ni) = (Vec::new(), Vec::new());
        if !bad {
            gi.extend(fd_state_subject(sfd));
            ni.extend(fd_state_reference(rfd));
            _close(sfd);
            ref__close(rfd);
        }
        let (sd, rd) = w.disk();
        push_disk(&mut gi, sd);
        push_disk(&mut ni, rd);
        let tags = vec![if negative {
            "negative"
        } else if bad {
            "badfd"
        } else if size as usize > seed.len() {
            "grow"
        } else {
            "shrink"
        }];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("len={} size={size} bad={bad}", seed.len()),
            tags,
        )
    });
}

// ══════════════════════════════════════════════════════════════════════════════════════
// Wave 18b — the wide stream layer
// ══════════════════════════════════════════════════════════════════════════════════════
//
// `_fgetwc_nolock` forks on the descriptor's text mode and the two arms have nothing in
// common. In binary (or any Unicode text mode) it assembles the `wchar_t` from TWO
// `_fgetc_nolock` calls; in ANSI text mode it reads one byte, asks `isleadbyte` whether a
// second is coming, and converts the 1-2 byte sequence with `mbtowc`. Every row below
// therefore runs both modes and tags them, because a row that only opened `"rb"` would
// grade half a function and report a full MATCH.
//
// What is NOT separable here: `isleadbyte`/`mbtowc`/`wctomb` are ONE definition shared by
// both sides (they live in mbcs.c and locale.c, neither of which is a reference TU). A
// conversion bug is invisible to these rows by construction — §9.73. They grade the
// stream logic around the converter, which is the part file.c owns. mbcs.c's own converter
// is graded in `tests/mbcs.rs`.

/// A guarded destination buffer of `wchar_t`, not bytes.
///
/// `Buf` is a `Vec<u8>`, whose allocation is 1-aligned; handing its interior to a
/// `wchar_t*` parameter would have the C write 16-bit stores through a possibly-odd
/// address. Backing this one with `Vec<u16>` makes the alignment right by construction.
struct WBuf(Vec<u16>);

impl WBuf {
    fn new(n: usize) -> WBuf {
        WBuf(vec![0xA7A7; n + 2 * GUARD])
    }
    fn p(&mut self) -> *mut u16 {
        unsafe { self.0.as_mut_ptr().add(GUARD) }
    }
    fn img(&self) -> Vec<u8> {
        self.0.iter().flat_map(|w| w.to_le_bytes()).collect()
    }
}

/// The wide characters a row writes. ASCII and the two text-mode-significant bytes are
/// over-weighted deliberately; the >0x7F values are what reach `wctomb`'s failure path in
/// ANSI text mode, where the character has no representation and the write must report it
/// rather than emit a replacement.
fn gen_wchar(r: &mut Rng) -> u16 {
    match r.below(8) {
        0 => 0x000A,
        1 => 0x000D,
        2 => 0x001A,
        3 => 0x00E9,
        4 => 0x20AC,
        _ => 0x20 + (r.next() % 0x5F) as u16,
    }
}

unsafe fn wide_stream_rows() {
    // `fgetwc` / `getwc` — `getwc` IS `fgetwc` in file.c, so both names run the same row
    // and staying identical is a checked property.
    grade("fgetwc+getwc", &["binary", "text", "weof"], WIDE_ITERS, |r| {
        let seed = r.bytes_upto(24);
        let w = World::new("fgetwc", &seed);
        let text = r.below(2) == 0;
        let mode: &[u8] = if text { b"r\0" } else { b"rb\0" };
        let via_getwc = r.below(2) == 0;
        let take = 1 + r.below(12);

        let sf = fopen(w.s(), mode.as_ptr());
        let rf = ref_fopen(w.r(), mode.as_ptr());
        let (mut sw, mut rw): (Vec<u16>, Vec<u16>) = (Vec::new(), Vec::new());
        let g = Effects::subject(|| {
            for _ in 0..take {
                sw.push(if via_getwc { getwc(sf) } else { fgetwc(sf) });
            }
            0
        });
        let n = Effects::reference(|| {
            for _ in 0..take {
                rw.push(if via_getwc { ref_getwc(rf) } else { ref_fgetwc(rf) });
            }
            0
        });
        let mut gi: Vec<u8> = sw.iter().flat_map(|c| c.to_le_bytes()).collect();
        let mut ni: Vec<u8> = rw.iter().flat_map(|c| c.to_le_bytes()).collect();
        gi.extend(stream_state_subject(sf));
        ni.extend(stream_state_reference(rf));
        fclose(sf);
        ref_fclose(rf);
        let tags = vec![
            if text { "text" } else { "binary" },
            if rw.contains(&WEOF) { "weof" } else { "" },
        ];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("len={} take={take} getwc={via_getwc}", seed.len()),
            tags.into_iter().filter(|t| !t.is_empty()).collect(),
        )
    });

    // `fputwc` — the disk image is the answer here, and it is read back by the host.
    grade("fputwc", &["binary", "text", "unconvertible"], WIDE_ITERS, |r| {
        let w = World::new("fputwc", b"");
        let text = r.below(2) == 0;
        let mode: &[u8] = if text { b"w\0" } else { b"wb\0" };
        let chars: Vec<u16> = (0..1 + r.below(8)).map(|_| gen_wchar(r)).collect();

        let sf = fopen(w.s(), mode.as_ptr());
        let rf = ref_fopen(w.r(), mode.as_ptr());
        let (mut sr, mut rr): (Vec<u16>, Vec<u16>) = (Vec::new(), Vec::new());
        let g = Effects::subject(|| {
            for &c in &chars {
                sr.push(fputwc(c, sf));
            }
            0
        });
        let n = Effects::reference(|| {
            for &c in &chars {
                rr.push(ref_fputwc(c, rf));
            }
            0
        });
        let mut gi: Vec<u8> = sr.iter().flat_map(|c| c.to_le_bytes()).collect();
        let mut ni: Vec<u8> = rr.iter().flat_map(|c| c.to_le_bytes()).collect();
        gi.extend(stream_state_subject(sf));
        ni.extend(stream_state_reference(rf));
        fclose(sf);
        ref_fclose(rf);
        let (sd, rd) = w.disk();
        push_disk(&mut gi, sd);
        push_disk(&mut ni, rd);
        let tags = vec![
            if text { "text" } else { "binary" },
            if rr.contains(&WEOF) { "unconvertible" } else { "" },
        ];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("n={} text={text}", chars.len()),
            tags.into_iter().filter(|t| !t.is_empty()).collect(),
        )
    });

    // `fgetws` — the wide `fgets`. `size` of 1 must produce an empty string and not a
    // character, and a `size` of 0 must not write at all; both are guard bytes away from
    // an overrun, which is why the destination is guarded.
    grade("fgetws", &["ok", "eof", "size1", "newline"], WIDE_ITERS, |r| {
        let seed = r.bytes_from(1, 40);
        let w = World::new("fgetws", &seed);
        let text = r.below(2) == 0;
        let mode: &[u8] = if text { b"r\0" } else { b"rb\0" };
        let size = r.below(12) as c_int;

        let sf = fopen(w.s(), mode.as_ptr());
        let rf = ref_fopen(w.r(), mode.as_ptr());
        let (mut sb, mut rb) = (WBuf::new(size.max(0) as usize), WBuf::new(size.max(0) as usize));
        let (sp, rp) = (sb.p(), rb.p());
        let g = Effects::subject(|| !fgetws(sp, size, sf).is_null() as i64);
        let n = Effects::reference(|| !ref_fgetws(rp, size, rf).is_null() as i64);
        let mut gi = sb.img();
        let mut ni = rb.img();
        gi.extend(stream_state_subject(sf));
        ni.extend(stream_state_reference(rf));
        fclose(sf);
        ref_fclose(rf);
        let tags = vec![
            if size == 1 {
                "size1"
            } else if n.ret == 0 {
                "eof"
            } else {
                "ok"
            },
            if seed.contains(&0x0A) { "newline" } else { "" },
        ];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("len={} size={size} text={text}", seed.len()),
            tags.into_iter().filter(|t| !t.is_empty()).collect(),
        )
    });

    // `fputws` — the wide `fputs`.
    grade("fputws", &["binary", "text", "empty"], WIDE_ITERS, |r| {
        let w = World::new("fputws", b"");
        let text = r.below(2) == 0;
        let mode: &[u8] = if text { b"w\0" } else { b"wb\0" };
        let mut s: Vec<u16> = (0..r.below(10)).map(|_| gen_wchar(r)).collect();
        s.push(0);

        let sf = fopen(w.s(), mode.as_ptr());
        let rf = ref_fopen(w.r(), mode.as_ptr());
        let g = Effects::subject(|| fputws(s.as_ptr(), sf) as i64);
        let n = Effects::reference(|| ref_fputws(s.as_ptr(), rf) as i64);
        let mut gi = stream_state_subject(sf);
        let mut ni = stream_state_reference(rf);
        fclose(sf);
        ref_fclose(rf);
        let (sd, rd) = w.disk();
        push_disk(&mut gi, sd);
        push_disk(&mut ni, rd);
        let tags = vec![
            if text { "text" } else { "binary" },
            if s.len() == 1 { "empty" } else { "" },
        ];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("n={} text={text}", s.len() - 1),
            tags.into_iter().filter(|t| !t.is_empty()).collect(),
        )
    });

    // `ungetwc` — push a wide character back and read it again. The `WEOF` argument is
    // required to be rejected, and the second consecutive pushback is where msvcrt's
    // one-character limit shows.
    grade("ungetwc", &["ok", "weof-arg", "twice"], WIDE_ITERS, |r| {
        let seed = r.bytes_from(2, 32);
        let w = World::new("ungetwc", &seed);
        let text = r.below(2) == 0;
        let mode: &[u8] = if text { b"r\0" } else { b"rb\0" };
        let weof_arg = r.below(6) == 0;
        let twice = r.below(3) == 0;
        let push = if weof_arg { WEOF } else { gen_wchar(r) };

        let sf = fopen(w.s(), mode.as_ptr());
        let rf = ref_fopen(w.r(), mode.as_ptr());
        fgetwc(sf);
        ref_fgetwc(rf);
        let (mut sr, mut rr): (Vec<u16>, Vec<u16>) = (Vec::new(), Vec::new());
        let g = Effects::subject(|| {
            sr.push(ungetwc(push, sf));
            if twice {
                sr.push(ungetwc(push, sf));
            }
            sr.push(fgetwc(sf));
            0
        });
        let n = Effects::reference(|| {
            rr.push(ref_ungetwc(push, rf));
            if twice {
                rr.push(ref_ungetwc(push, rf));
            }
            rr.push(ref_fgetwc(rf));
            0
        });
        let mut gi: Vec<u8> = sr.iter().flat_map(|c| c.to_le_bytes()).collect();
        let mut ni: Vec<u8> = rr.iter().flat_map(|c| c.to_le_bytes()).collect();
        gi.extend(stream_state_subject(sf));
        ni.extend(stream_state_reference(rf));
        fclose(sf);
        ref_fclose(rf);
        let tags = vec![
            if weof_arg { "weof-arg" } else { "ok" },
            if twice { "twice" } else { "" },
        ];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("push={push:#x} twice={twice} text={text}"),
            tags.into_iter().filter(|t| !t.is_empty()).collect(),
        )
    });
}

// ══════════════════════════════════════════════════════════════════════════════════════
// Wave 18b — buffering and the byte-at-a-time entry points
// ══════════════════════════════════════════════════════════════════════════════════════
//
// `setvbuf` is the only public way to reach the buffer-selection code that wave 18a's
// header calls out as the behaviour with no other window, and `_filbuf`/`_flsbuf` are the
// refill and spill that every other stream call goes through. Wave 18a graded them only
// indirectly, through whatever buffering `fopen` happened to choose.

/// msvcrt's `BUFSIZ`. `setbuf` passes it to `setvbuf`, so a user buffer handed to `setbuf`
/// must be at least this large or the CRT writes past it.
const MSVCRT_BUFSIZ: usize = 512;

unsafe fn buffering_rows() {
    // `setvbuf` — the three modes, a user buffer and a CRT-allocated one, and all three
    // guards. The compared image is the FILE's buffer state, which is the only place the
    // decision is visible.
    grade("setvbuf", &["fbf", "lbf", "nbf", "userbuf", "guard"], ROW_ITERS, |r| {
        let seed = r.bytes_from(1, 40);
        let w = World::new("setvbuf", &seed);
        // 0 = none, 1 = NULL file, 2 = bad mode, 3 = size out of range.
        let guard = if r.below(4) == 0 { 1 + r.below(3) } else { 0 };
        let mode = match guard {
            2 => 99,
            // `_IONBF` is EXEMPT from the size check (`mode==_IONBF || (size>=2 && …)`),
            // so drawing it here would leave the row tagged "guard" over a case where no
            // guard fired — a coverage claim for a branch that never ran.
            3 => [IOFBF, IOLBF][r.below(2)],
            _ => [IOFBF, IOLBF, IONBF][r.below(3)],
        };
        let size = if guard == 3 { 1 } else { 2 + r.below(4096) };
        let userbuf = guard == 0 && r.below(2) == 0;
        let (mut sbuf, mut rbuf) = (vec![0u8; size + 64], vec![0u8; size + 64]);

        let sf = fopen(w.s(), b"rb\0".as_ptr());
        let rf = ref_fopen(w.r(), b"rb\0".as_ptr());
        let (sarg, rarg) = if userbuf {
            (sbuf.as_mut_ptr(), rbuf.as_mut_ptr())
        } else {
            (core::ptr::null_mut(), core::ptr::null_mut())
        };
        let (sfile, rfile) = if guard == 1 {
            (core::ptr::null_mut(), core::ptr::null_mut())
        } else {
            (sf, rf)
        };
        let g = Effects::subject(|| setvbuf(sfile, sarg, mode, size) as i64);
        let n = Effects::reference(|| ref_setvbuf(rfile, rarg, mode, size) as i64);
        let mut gi = stream_state_subject(sf);
        let mut ni = stream_state_reference(rf);
        // Drive one read through whatever buffering was just selected.
        let (mut sb, mut rb) = (Buf::new(8), Buf::new(8));
        let (sp, rp) = (sb.p(), rb.p());
        fread(sp as *mut c_void, 1, 8, sf);
        ref_fread(rp as *mut c_void, 1, 8, rf);
        gi.extend(sb.img());
        ni.extend(rb.img());
        gi.extend(stream_state_subject(sf));
        ni.extend(stream_state_reference(rf));
        fclose(sf);
        ref_fclose(rf);
        let tags = vec![if guard != 0 {
            "guard"
        } else if userbuf {
            "userbuf"
        } else if mode == IONBF {
            "nbf"
        } else if mode == IOLBF {
            "lbf"
        } else {
            "fbf"
        }];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("guard={guard} mode={mode} size={size} userbuf={userbuf}"),
            tags,
        )
    });

    // `setbuf` — `setvbuf` with the mode and size implied by whether a buffer was passed.
    // It returns nothing, so the FILE's buffer state IS the entire observation.
    grade("setbuf", &["userbuf", "nobuf"], ROW_ITERS, |r| {
        let seed = r.bytes_from(1, 40);
        let w = World::new("setbuf", &seed);
        let userbuf = r.below(2) == 0;
        let (mut sbuf, mut rbuf) = (vec![0u8; MSVCRT_BUFSIZ], vec![0u8; MSVCRT_BUFSIZ]);

        let sf = fopen(w.s(), b"rb\0".as_ptr());
        let rf = ref_fopen(w.r(), b"rb\0".as_ptr());
        let (sarg, rarg) = if userbuf {
            (sbuf.as_mut_ptr(), rbuf.as_mut_ptr())
        } else {
            (core::ptr::null_mut(), core::ptr::null_mut())
        };
        let g = Effects::subject(|| {
            setbuf(sf, sarg);
            0
        });
        let n = Effects::reference(|| {
            ref_setbuf(rf, rarg);
            0
        });
        let mut gi = stream_state_subject(sf);
        let mut ni = stream_state_reference(rf);
        let (mut sb, mut rb) = (Buf::new(8), Buf::new(8));
        let (sp, rp) = (sb.p(), rb.p());
        fread(sp as *mut c_void, 1, 8, sf);
        ref_fread(rp as *mut c_void, 1, 8, rf);
        gi.extend(sb.img());
        ni.extend(rb.img());
        gi.extend(stream_state_subject(sf));
        ni.extend(stream_state_reference(rf));
        fclose(sf);
        ref_fclose(rf);
        let tags = vec![if userbuf { "userbuf" } else { "nobuf" }];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("userbuf={userbuf} len={}", seed.len()),
            tags,
        )
    });

    // `_filbuf` / `_flsbuf` — the refill and the spill, called directly rather than
    // through a `getc`/`putc` that might not reach them.
    grade("_filbuf+_flsbuf", &["fil", "fil-eof", "fls"], ROW_ITERS, |r| {
        let seed = r.bytes_upto(24);
        let w = World::new("filbuf", &seed);
        let writing = r.below(2) == 0;
        let ch = (r.next() % 256) as c_int;

        let (mut gi, mut ni) = (Vec::new(), Vec::new());
        let (sf, rf) = if writing {
            (fopen(w.s(), b"wb\0".as_ptr()), ref_fopen(w.r(), b"wb\0".as_ptr()))
        } else {
            (fopen(w.s(), b"rb\0".as_ptr()), ref_fopen(w.r(), b"rb\0".as_ptr()))
        };
        let g = Effects::subject(|| (if writing { _flsbuf(ch, sf) } else { _filbuf(sf) }) as i64);
        let n = Effects::reference(|| {
            (if writing { ref__flsbuf(ch, rf) } else { ref__filbuf(rf) }) as i64
        });
        gi.extend(stream_state_subject(sf));
        ni.extend(stream_state_reference(rf));
        fclose(sf);
        ref_fclose(rf);
        let (sd, rd) = w.disk();
        push_disk(&mut gi, sd);
        push_disk(&mut ni, rd);
        let tags = vec![if writing {
            "fls"
        } else if n.ret == CEOF as i64 {
            "fil-eof"
        } else {
            "fil"
        }];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("writing={writing} len={} ch={ch}", seed.len()),
            tags,
        )
    });

    // `getc` / `putc` — msvcrt exports these as real functions, not macros, so they are
    // graded like any other export rather than assumed identical to `fgetc`/`fputc`.
    grade("getc+putc", &["get", "get-eof", "put"], ROW_ITERS, |r| {
        let seed = r.bytes_upto(24);
        let w = World::new("getcputc", &seed);
        let writing = r.below(2) == 0;
        let ch = (r.next() % 256) as c_int;
        let take = 1 + r.below(6);

        let (sf, rf) = if writing {
            (fopen(w.s(), b"wb\0".as_ptr()), ref_fopen(w.r(), b"wb\0".as_ptr()))
        } else {
            (fopen(w.s(), b"rb\0".as_ptr()), ref_fopen(w.r(), b"rb\0".as_ptr()))
        };
        let (mut sr, mut rr): (Vec<c_int>, Vec<c_int>) = (Vec::new(), Vec::new());
        let g = Effects::subject(|| {
            for _ in 0..take {
                sr.push(if writing { putc(ch, sf) } else { getc(sf) });
            }
            0
        });
        let n = Effects::reference(|| {
            for _ in 0..take {
                rr.push(if writing { ref_putc(ch, rf) } else { ref_getc(rf) });
            }
            0
        });
        let mut gi: Vec<u8> = sr.iter().flat_map(|c| c.to_le_bytes()).collect();
        let mut ni: Vec<u8> = rr.iter().flat_map(|c| c.to_le_bytes()).collect();
        gi.extend(stream_state_subject(sf));
        ni.extend(stream_state_reference(rf));
        fclose(sf);
        ref_fclose(rf);
        let (sd, rd) = w.disk();
        push_disk(&mut gi, sd);
        push_disk(&mut ni, rd);
        let tags = vec![if writing {
            "put"
        } else if rr.contains(&CEOF) {
            "get-eof"
        } else {
            "get"
        }];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("writing={writing} take={take} len={}", seed.len()),
            tags,
        )
    });

    // `_getw` / `_putw` — an `int` at a time, in host byte order. `_getw` on a file with
    // fewer than four bytes left must latch `_IOEOF` and return `EOF`, and "fewer than
    // four" is a partial read it has already consumed — which is why the row varies the
    // seed length around a multiple of four rather than only past the end.
    grade("_getw+_putw", &["get", "get-short", "put"], ROW_ITERS, |r| {
        let len = r.below(10);
        let seed = r.bytes(len);
        let w = World::new("getwputw", &seed);
        let writing = r.below(2) == 0;
        let val = r.next() as i32;

        let (sf, rf) = if writing {
            (fopen(w.s(), b"wb\0".as_ptr()), ref_fopen(w.r(), b"wb\0".as_ptr()))
        } else {
            (fopen(w.s(), b"rb\0".as_ptr()), ref_fopen(w.r(), b"rb\0".as_ptr()))
        };
        let g = Effects::subject(|| (if writing { _putw(val, sf) } else { _getw(sf) }) as i64);
        let n = Effects::reference(|| {
            (if writing { ref__putw(val, rf) } else { ref__getw(rf) }) as i64
        });
        let mut gi = stream_state_subject(sf);
        let mut ni = stream_state_reference(rf);
        fclose(sf);
        ref_fclose(rf);
        let (sd, rd) = w.disk();
        push_disk(&mut gi, sd);
        push_disk(&mut ni, rd);
        let tags = vec![if writing {
            "put"
        } else if len < 4 {
            "get-short"
        } else {
            "get"
        }];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("writing={writing} len={len} val={val:#x}"),
            tags,
        )
    });

    // `clearerr_s` — `clearerr` with a NULL guard and an `errno_t` return.
    grade("clearerr_s", &["cleared", "nullfile"], ROW_ITERS, |r| {
        let seed = r.bytes_upto(8);
        let w = World::new("clearerrs", &seed);
        let nullfile = r.below(4) == 0;

        let sf = fopen(w.s(), b"rb\0".as_ptr());
        let rf = ref_fopen(w.r(), b"rb\0".as_ptr());
        // Latch EOF so that clearing it is observable.
        let mut b = [0u8; 64];
        fread(b.as_mut_ptr() as *mut c_void, 1, 64, sf);
        fgetc(sf);
        ref_fread(b.as_mut_ptr() as *mut c_void, 1, 64, rf);
        ref_fgetc(rf);
        let (sarg, rarg) = if nullfile {
            (core::ptr::null_mut(), core::ptr::null_mut())
        } else {
            (sf, rf)
        };
        let g = Effects::subject(|| clearerr_s(sarg) as i64);
        let n = Effects::reference(|| ref_clearerr_s(rarg) as i64);
        let gi = stream_state_subject(sf);
        let ni = stream_state_reference(rf);
        fclose(sf);
        ref_fclose(rf);
        let tags = vec![if nullfile { "nullfile" } else { "cleared" }];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("nullfile={nullfile} len={}", seed.len()),
            tags,
        )
    });
}

// ══════════════════════════════════════════════════════════════════════════════════════
// Wave 18c — timestamps and the process-global stream ops
// ══════════════════════════════════════════════════════════════════════════════════════
//
// The utime family is six exports over one body: `_futime64` does the work, `_utime64` and
// `_wutime64` open a descriptor and delegate, and the three `*32` twins widen their
// `__time32_t` pair and delegate again. So the interesting axis is not the entry point but
// the VALUE — whether a time survives `time_to_filetime`, the 100ns-since-1601 conversion,
// `SetFileTime`, and the read back out through `_stat64`'s
// `RtlTimeToSecondsSince1970` unchanged.
//
// No NULL-`t` case. `_futime64` reads `_time64(NULL)` for it, so the two sides stamp their
// own wall clocks; they agree to the second almost always and straddle a boundary at
// random, which is a flake rather than a test. The row says so instead of carrying a tag
// that fails once a fortnight.

/// Times a utime row draws from, all inside the range a 32-bit `__time32_t` can hold so the
/// `*32` and `*64` variants are graded on identical values. 0 is the epoch itself, which is
/// where a sign or bias error in the 1601 conversion shows up.
const UTIMES: [i64; 6] = [0, 1, 1_000_000_000, 1_600_000_000, 2_000_000_000, 2_147_483_647];

unsafe fn utime_rows() {
    // `_utime32` / `_utime64` — the narrow path.
    grade("_utime32+_utime64", &["ok", "enoent", "epoch"], ROW_ITERS, |r| {
        let exists = r.below(4) != 0;
        let seed = r.bytes_upto(16);
        let w = if exists { World::new("utime", &seed) } else { World::absent("utime") };
        let wide64 = r.below(2) == 0;
        let at = UTIMES[r.below(UTIMES.len())];
        let mt = UTIMES[r.below(UTIMES.len())];
        let t64 = [at, mt];
        let t32 = [at as i32, mt as i32];

        let g = Effects::subject(|| {
            if wide64 {
                _utime64(w.s(), t64.as_ptr()) as i64
            } else {
                _utime32(w.s(), t32.as_ptr()) as i64
            }
        });
        let n = Effects::reference(|| {
            if wide64 {
                ref__utime64(w.r(), t64.as_ptr()) as i64
            } else {
                ref__utime32(w.r(), t32.as_ptr()) as i64
            }
        });
        let (gi, ni) = stat_both(&w);
        let tags = vec![
            if n.ret == 0 { "ok" } else { "enoent" },
            if at == 0 || mt == 0 { "epoch" } else { "" },
        ];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("exists={exists} 64={wide64} at={at} mt={mt}"),
            tags.into_iter().filter(|t| !t.is_empty()).collect(),
        )
    });

    // `_wutime32` / `_wutime64` — the wide path, which is the one that owns the body.
    grade("_wutime32+_wutime64", &["ok", "enoent", "epoch"], ROW_ITERS, |r| {
        let exists = r.below(4) != 0;
        let seed = r.bytes_upto(16);
        let w = if exists { World::new("wutime", &seed) } else { World::absent("wutime") };
        let wide64 = r.below(2) == 0;
        let at = UTIMES[r.below(UTIMES.len())];
        let mt = UTIMES[r.below(UTIMES.len())];
        let t64 = [at, mt];
        let t32 = [at as i32, mt as i32];
        let (sw, rw) = (w.ws(), w.wr());

        let g = Effects::subject(|| {
            if wide64 {
                _wutime64(sw.as_ptr(), t64.as_ptr()) as i64
            } else {
                _wutime32(sw.as_ptr(), t32.as_ptr()) as i64
            }
        });
        let n = Effects::reference(|| {
            if wide64 {
                ref__wutime64(rw.as_ptr(), t64.as_ptr()) as i64
            } else {
                ref__wutime32(rw.as_ptr(), t32.as_ptr()) as i64
            }
        });
        let (gi, ni) = stat_both(&w);
        let tags = vec![
            if n.ret == 0 { "ok" } else { "enoent" },
            if at == 0 || mt == 0 { "epoch" } else { "" },
        ];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("exists={exists} 64={wide64} at={at} mt={mt}"),
            tags.into_iter().filter(|t| !t.is_empty()).collect(),
        )
    });

    // `_futime32` / `_futime64` — straight at the body, over a descriptor.
    grade("_futime32+_futime64", &["ok", "badfd", "epoch"], ROW_ITERS, |r| {
        let seed = r.bytes_upto(16);
        let w = World::new("futime", &seed);
        let bad = r.below(5) == 0;
        let wide64 = r.below(2) == 0;
        let at = UTIMES[r.below(UTIMES.len())];
        let mt = UTIMES[r.below(UTIMES.len())];
        let t64 = [at, mt];
        let t32 = [at as i32, mt as i32];

        // Each side's own times before the call: a bad descriptor must leave them alone.
        let before = stat_both(&w);
        let (sfd, rfd) = if bad {
            (4096, 4096)
        } else {
            (_open(w.s(), O_WRONLY | O_BINARY), ref__open(w.r(), O_WRONLY | O_BINARY))
        };
        let g = Effects::subject(|| {
            if wide64 {
                _futime64(sfd, t64.as_ptr()) as i64
            } else {
                _futime32(sfd, t32.as_ptr()) as i64
            }
        });
        let n = Effects::reference(|| {
            if wide64 {
                ref__futime64(rfd, t64.as_ptr()) as i64
            } else {
                ref__futime32(rfd, t32.as_ptr()) as i64
            }
        });
        if !bad {
            _close(sfd);
            ref__close(rfd);
        }
        let (mut gi, mut ni) = stat_both(&w);
        if bad {
            // Nothing was stamped, so the times are each file's own creation time: two
            // files made on either side of a second boundary differ by one, and rejected a
            // lane that touched no msvcrt code (2026-10-11, mt=2147483647). What a bad fd
            // must NOT do is stamp, and that is a per-side fact: graded as "unchanged",
            // then the cross-side words are masked.
            let gu = stat_times(&before.0) == stat_times(&gi);
            let nu = stat_times(&before.1) == stat_times(&ni);
            mask_stat_times(&mut gi);
            mask_stat_times(&mut ni);
            gi.push(gu as u8);
            ni.push(nu as u8);
        }
        let tags = vec![
            if bad { "badfd" } else { "ok" },
            if at == 0 || mt == 0 { "epoch" } else { "" },
        ];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("bad={bad} 64={wide64} at={at} mt={mt}"),
            tags.into_iter().filter(|t| !t.is_empty()).collect(),
        )
    });
}

/// Each side's own file, through its own `_stat64`.
///
/// This is the only place in the suite where a `World` (two files) is stat'd rather than a
/// `Shared` (one). It is sound here for the reason it is not sound for the stat rows
/// themselves: those compare timestamps the FILESYSTEM chose, which two files created
/// microseconds apart do not share, while these rows SET both timestamps to a value the row
/// picked. What is left to differ is exactly what the utime call did with it.
unsafe fn stat_both(w: &World) -> (Vec<u8>, Vec<u8>) {
    let (mut sb, mut rb) = (QBuf::new(STAT_BUF), QBuf::new(STAT_BUF));
    let (sp, rp) = (sb.p(), rb.p());
    let sr = _stat64(w.s(), sp);
    let rr = ref__stat64(w.r(), rp);
    let mut gi = sb.img();
    let mut ni = rb.img();
    gi.extend_from_slice(&sr.to_le_bytes());
    ni.extend_from_slice(&rr.to_le_bytes());
    (gi, ni)
}

/// `st_atime`/`st_mtime`/`st_ctime` in a `stat_both` image: the leading guard `u64`, then
/// the `_stat64` offsets 32/40/48 that `refstub/msvcrt_crt_layout.h` asserts.
const STAT_TIMES: core::ops::Range<usize> = 8 + 32..8 + 56;

fn stat_times(img: &[u8]) -> Vec<u8> {
    img[STAT_TIMES].to_vec()
}

fn mask_stat_times(img: &mut [u8]) {
    img[STAT_TIMES].fill(0);
}

unsafe fn global_stream_rows() {
    // `_flushall` — every buffered stream, at once. The return is the COUNT flushed, so it
    // is also a statement about how many streams each side thinks it has open: the two
    // rings have to have stayed in step for this to match, which no other row asserts
    // directly.
    grade("_flushall", &["some", "none"], ROW_ITERS, |r| {
        let n_streams = r.below(4);
        let payload = r.bytes_from(1, 24);
        let worlds: Vec<World> =
            (0..n_streams).map(|i| World::new(&format!("flushall{i}"), b"")).collect();
        let mut sf: Vec<FILEP> = Vec::new();
        let mut rf: Vec<FILEP> = Vec::new();
        for w in &worlds {
            let a = fopen(w.s(), b"wb\0".as_ptr());
            let b = ref_fopen(w.r(), b"wb\0".as_ptr());
            fwrite(payload.as_ptr() as *const c_void, 1, payload.len(), a);
            ref_fwrite(payload.as_ptr() as *const c_void, 1, payload.len(), b);
            sf.push(a);
            rf.push(b);
        }

        let g = Effects::subject(|| _flushall() as i64);
        let n = Effects::reference(|| ref__flushall() as i64);
        // Read the files back BEFORE closing: fclose would flush them anyway, and then the
        // row would pass over an implementation whose _flushall did nothing at all.
        let (mut gi, mut ni) = (Vec::new(), Vec::new());
        for w in &worlds {
            let (sd, rd) = w.disk();
            push_disk(&mut gi, sd);
            push_disk(&mut ni, rd);
        }
        for f in sf {
            fclose(f);
        }
        for f in rf {
            ref_fclose(f);
        }
        let tags = vec![if n_streams > 0 { "some" } else { "none" }];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("streams={n_streams} len={}", payload.len()),
            tags,
        )
    });

    // `_fcloseall` — closes everything from index 3 up and returns the count. It consumes
    // the streams, so the row must NOT close them again; that it leaves both rings equally
    // empty is what `balanced()` then confirms.
    grade("_fcloseall", &["some", "none"], ROW_ITERS, |r| {
        let n_streams = r.below(4);
        let worlds: Vec<World> =
            (0..n_streams).map(|i| World::new(&format!("fcloseall{i}"), b"x")).collect();
        for w in &worlds {
            fopen(w.s(), b"rb\0".as_ptr());
            ref_fopen(w.r(), b"rb\0".as_ptr());
        }

        let g = Effects::subject(|| _fcloseall() as i64);
        let n = Effects::reference(|| ref__fcloseall() as i64);
        let tags = vec![if n_streams > 0 { "some" } else { "none" }];
        (
            Obs::with(g, Vec::new()),
            Obs::with(n, Vec::new()),
            format!("streams={n_streams}"),
            tags,
        )
    });

    // `_setmaxstdio` / `_getmaxstdio` — the stream-table ceiling, and its three refusals:
    // below `_IOB_ENTRIES`, above `MSVCRT_MAX_FILES`, or below the number of streams
    // already handed out. The row RESTORES the value it found, because every later row's
    // `msvcrt_get_file` reads it.
    grade("_setmaxstdio+_getmaxstdio", &["ok", "too-low", "too-high"], ROW_ITERS, |r| {
        let before_s = _getmaxstdio();
        let before_r = ref__getmaxstdio();
        let newmax = [512, 2048, 5, 99_999, 1][r.below(5)];

        let g = Effects::subject(|| _setmaxstdio(newmax) as i64);
        let n = Effects::reference(|| ref__setmaxstdio(newmax) as i64);
        let (mut gi, mut ni) = (Vec::new(), Vec::new());
        gi.extend_from_slice(&_getmaxstdio().to_le_bytes());
        ni.extend_from_slice(&ref__getmaxstdio().to_le_bytes());
        _setmaxstdio(before_s);
        ref__setmaxstdio(before_r);
        let tags = vec![if n.ret > 0 {
            "ok"
        } else if newmax < 20 {
            "too-low"
        } else {
            "too-high"
        }];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("newmax={newmax} before={before_s}/{before_r}"),
            tags,
        )
    });
}

// ══════════════════════════════════════════════════════════════════════════════════════
// Wave 18c part 2 — the temp-name family
// ══════════════════════════════════════════════════════════════════════════════════════
//
// Twelve exports that split cleanly in two, and the split is the whole design of this
// section rather than a caveat bolted onto it.
//
// EIGHT ARE FULLY DETERMINISTIC and are compared BYTE FOR BYTE. `tmpnam` and `_mktemp`
// build their candidate from `GetCurrentProcessId()` and a counter that is a file-static in
// each side's own TU, then PROBE for a free name with `GetFileAttributes*` without ever
// creating one. Two implementations starting from the same counter, over the same
// filesystem, that take nothing from it, must produce the identical string — so anything
// less than an exact comparison here would be throwing away the strongest evidence the
// suite can get about a name-building function.
//
// FOUR CANNOT BE, and it is worth being precise about why, because "compare less" is
// exactly the move that turns a row into decoration. `tmpfile`, `tmpfile_s`, `_tempnam` and
// `_wtempnam` all reach `GetTempFileNameW`, whose shim is `mkstemp` — a RANDOM suffix, and
// one that reserves the name by creating it. The two sides therefore get different names by
// construction, and no amount of care in the row changes that. What those rows compare is
// the class of answer (produced a name / failed), the errno, that the name lands under the
// directory that was asked for, and — for `tmpfile` — that the stream is actually usable by
// writing and reading it back. They do NOT claim the names match, and they say so.

/// A `_mktemp` pattern lives in the temp directory so the row is not writing into the
/// source tree, and ends in the six `X`s the contract requires.
fn mktemp_pattern(tag: &str) -> Vec<u8> {
    let p = std::env::temp_dir().join(format!("msvcrt-mkt-{}-{}-XXXXXX", std::process::id(), tag));
    let mut v = p.to_str().unwrap().as_bytes().to_vec();
    v.push(0);
    v
}

/// The NUL-terminated C string at `p`, or None. Used to compare a returned name.
unsafe fn cstr(p: *const u8) -> Option<Vec<u8>> {
    if p.is_null() {
        return None;
    }
    let mut out = Vec::new();
    let mut i = 0;
    while *p.add(i) != 0 && i < 512 {
        out.push(*p.add(i));
        i += 1;
    }
    Some(out)
}

unsafe fn wcstr(p: *const u16) -> Option<Vec<u16>> {
    if p.is_null() {
        return None;
    }
    let mut out = Vec::new();
    let mut i = 0;
    while *p.add(i) != 0 && i < 512 {
        out.push(*p.add(i));
        i += 1;
    }
    Some(out)
}

fn push_name(img: &mut Vec<u8>, name: Option<Vec<u8>>) {
    match name {
        None => img.push(0xDE),
        Some(b) => {
            img.push(0xFA);
            img.extend_from_slice(&b);
        }
    }
}

unsafe fn tempname_rows() {
    // `tmpnam` / `tmpnam_s` / `_wtmpnam` / `_wtmpnam_s` — deterministic, compared exactly.
    //
    // The counter is per-side and ADVANCES on every call, so the two sides must be driven
    // exactly the same number of times. That is why all four variants are one row rather
    // than four: a row that called `tmpnam` 220 times while another called `_wtmpnam` 220
    // times would leave the two counters in different places and every later comparison
    // would be off by a name.
    grade("tmpnam family", &["narrow", "wide", "secure", "erange"], ROW_ITERS, |r| {
        let which = r.below(4);
        let small = r.below(6) == 0;
        let size = if small { 2 } else { 64 };
        let (mut sb, mut rb) = (Buf::new(64), Buf::new(64));
        let (mut swb, mut rwb) = (WBuf::new(64), WBuf::new(64));
        let (sp, rp) = (sb.p(), rb.p());
        let (swp, rwp) = (swb.p(), rwb.p());
        let (mut sname, mut rname) = (None, None);
        let (mut swname, mut rwname) = (None, None);

        let g = Effects::subject(|| match which {
            0 => {
                sname = cstr(tmpnam(sp));
                0
            }
            1 => tmpnam_s(sp, size) as i64,
            2 => {
                swname = wcstr(_wtmpnam(swp));
                0
            }
            _ => _wtmpnam_s(swp, size) as i64,
        });
        let n = Effects::reference(|| match which {
            0 => {
                rname = cstr(ref_tmpnam(rp));
                0
            }
            1 => ref_tmpnam_s(rp, size) as i64,
            2 => {
                rwname = wcstr(ref__wtmpnam(rwp));
                0
            }
            _ => ref__wtmpnam_s(rwp, size) as i64,
        });
        let (mut gi, mut ni) = (sb.img(), rb.img());
        gi.extend(swb.img());
        ni.extend(rwb.img());
        push_name(&mut gi, sname);
        push_name(&mut ni, rname);
        push_name(&mut gi, swname.map(|v| v.iter().flat_map(|c| c.to_le_bytes()).collect()));
        push_name(&mut ni, rwname.map(|v| v.iter().flat_map(|c| c.to_le_bytes()).collect()));
        let tags = vec![
            match which {
                0 => "narrow",
                1 => "secure",
                2 => "wide",
                _ => "secure",
            },
            if small && (which == 1 || which == 3) { "erange" } else { "" },
        ];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("which={which} size={size}"),
            tags.into_iter().filter(|t| !t.is_empty()).collect(),
        )
    });

    // `_mktemp` / `_mktemp_s` / `_wmktemp` / `_wmktemp_s` — deterministic, compared
    // exactly. Unlike the tmpnam family these take no counter, so each call is independent;
    // what they DO consume is the filesystem, by probing 'a'..'z' for a free name. Neither
    // side creates anything, so both scans stop at the same letter.
    grade("_mktemp family", &["narrow", "wide", "secure", "einval"], ROW_ITERS, |r| {
        let which = r.below(4);
        // A pattern with too few Xs must be REFUSED, and that guard exists only on the
        // `_s` variants — the bare ones would happily index off the front.
        let broken = r.below(5) == 0;
        let mut pat = mktemp_pattern("a");
        if broken {
            let len = pat.len();
            pat[len - 3] = b'Q';
        }
        let wpat: Vec<u16> = pat.iter().map(|&b| b as u16).collect();
        let (mut sp_, mut rp_) = (pat.clone(), pat.clone());
        let (mut swp_, mut rwp_) = (wpat.clone(), wpat.clone());
        let size = sp_.len();

        let g = Effects::subject(|| match which {
            0 => !_mktemp(sp_.as_mut_ptr()).is_null() as i64,
            1 => _mktemp_s(sp_.as_mut_ptr(), size) as i64,
            2 => !_wmktemp(swp_.as_mut_ptr()).is_null() as i64,
            _ => _wmktemp_s(swp_.as_mut_ptr(), size) as i64,
        });
        let n = Effects::reference(|| match which {
            0 => !ref__mktemp(rp_.as_mut_ptr()).is_null() as i64,
            1 => ref__mktemp_s(rp_.as_mut_ptr(), size) as i64,
            2 => !ref__wmktemp(rwp_.as_mut_ptr()).is_null() as i64,
            _ => ref__wmktemp_s(rwp_.as_mut_ptr(), size) as i64,
        });
        // The pattern is mutated IN PLACE, so the buffer itself is the answer.
        let mut gi = sp_.clone();
        let mut ni = rp_.clone();
        gi.extend(swp_.iter().flat_map(|c| c.to_le_bytes()));
        ni.extend(rwp_.iter().flat_map(|c| c.to_le_bytes()));
        let tags = vec![
            match which {
                0 => "narrow",
                1 => "secure",
                2 => "wide",
                _ => "secure",
            },
            if broken && (which == 1 || which == 3) { "einval" } else { "" },
        ];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("which={which} broken={broken}"),
            tags.into_iter().filter(|t| !t.is_empty()).collect(),
        )
    });

    // `tmpfile` / `tmpfile_s` — the NAME is not comparable and the row does not pretend
    // otherwise. `GetTempFileNameW`'s shim is `mkstemp`, which picks a random suffix AND
    // reserves it by creating the file, so the second caller cannot receive the first
    // caller's name even in principle. What IS compared: whether a stream came back, its
    // errno, its buffering state, and — the part that would catch a stream that is a
    // handle to nothing — a write and read back through it.
    grade("tmpfile+tmpfile_s", &["ok", "secure", "nullout"], ROW_ITERS, |r| {
        let secure = r.below(2) == 0;
        let nullout = secure && r.below(6) == 0;
        let payload = r.bytes_from(1, 24);

        let mut sf: FILEP = core::ptr::null_mut();
        let mut rf: FILEP = core::ptr::null_mut();
        let sp = if nullout { core::ptr::null_mut() } else { &mut sf as *mut FILEP };
        let rp = if nullout { core::ptr::null_mut() } else { &mut rf as *mut FILEP };
        let g = Effects::subject(|| {
            if secure {
                tmpfile_s(sp) as i64
            } else {
                sf = tmpfile();
                !sf.is_null() as i64
            }
        });
        let n = Effects::reference(|| {
            if secure {
                ref_tmpfile_s(rp) as i64
            } else {
                rf = ref_tmpfile();
                !rf.is_null() as i64
            }
        });
        let mut gi = stream_state_subject(sf);
        let mut ni = stream_state_reference(rf);
        // Prove the stream is real: write, rewind, read it back.
        for (f, img, is_subject) in
            [(sf, &mut gi, true), (rf, &mut ni, false)]
        {
            if f.is_null() {
                continue;
            }
            let mut back = Buf::new(payload.len());
            let bp = back.p();
            if is_subject {
                fwrite(payload.as_ptr() as *const c_void, 1, payload.len(), f);
                rewind(f);
                fread(bp as *mut c_void, 1, payload.len(), f);
            } else {
                ref_fwrite(payload.as_ptr() as *const c_void, 1, payload.len(), f);
                ref_rewind(f);
                ref_fread(bp as *mut c_void, 1, payload.len(), f);
            }
            img.extend(back.img());
        }
        if !sf.is_null() {
            fclose(sf);
        }
        if !rf.is_null() {
            ref_fclose(rf);
        }
        let tags = vec![if nullout {
            "nullout"
        } else if secure {
            "secure"
        } else {
            "ok"
        }];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("secure={secure} nullout={nullout} len={}", payload.len()),
            tags,
        )
    });

    // `_tempnam` / `_wtempnam` — same non-comparable name, same reason. These read `TMP`
    // from the environment (through fileimpl.c's `_wgetenv`, which reads the real one), so
    // the row SETS it and then checks that the returned name actually lands under it. That
    // is the part worth grading: the directory argument is overridden by `TMP` whenever
    // `TMP` is set, and a translation that dropped the override would still return a
    // perfectly good name somewhere else.
    grade("_tempnam+_wtempnam", &["narrow", "wide", "tmp-honoured"], ROW_ITERS, |r| {
        let wide = r.below(2) == 0;
        let tmp = std::env::temp_dir();
        std::env::set_var("TMP", &tmp);
        let dir = CString::new("/nonexistent-dir-the-TMP-override-must-beat").unwrap();
        let prefix = CString::new("vb").unwrap();
        let wdir: Vec<u16> = wpath(&dir);
        let wprefix: Vec<u16> = wpath(&prefix);

        let (mut sname, mut rname): (Option<Vec<u8>>, Option<Vec<u8>>) = (None, None);
        let g = Effects::subject(|| {
            let p: Option<Vec<u8>> = if wide {
                let w = _wtempnam(wdir.as_ptr(), wprefix.as_ptr());
                wcstr(w).map(|v| v.iter().map(|&c| c as u8).collect())
            } else {
                cstr(_tempnam(dir.as_ptr() as *const u8, prefix.as_ptr() as *const u8))
            };
            sname = p;
            sname.is_some() as i64
        });
        let n = Effects::reference(|| {
            let p: Option<Vec<u8>> = if wide {
                let w = ref__wtempnam(wdir.as_ptr(), wprefix.as_ptr());
                wcstr(w).map(|v| v.iter().map(|&c| c as u8).collect())
            } else {
                cstr(ref__tempnam(dir.as_ptr() as *const u8, prefix.as_ptr() as *const u8))
            };
            rname = p;
            rname.is_some() as i64
        });
        // NOT the name — the SHAPE of the name. Both must be under TMP and carry the
        // prefix; the random suffix mkstemp chose is not a difference between the two
        // implementations and comparing it would fail every single case.
        let want = tmp.to_str().unwrap().as_bytes().to_vec();
        let shape = |nm: &Option<Vec<u8>>| -> Vec<u8> {
            match nm {
                None => vec![0xDE],
                Some(b) => vec![
                    0xFA,
                    b.starts_with(&want) as u8,
                    b.windows(2).any(|w| w == b"vb") as u8,
                ],
            }
        };
        let (gi, ni) = (shape(&sname), shape(&rname));
        let honoured = rname.as_ref().is_some_and(|b| b.starts_with(&want));
        let tags = vec![
            if wide { "wide" } else { "narrow" },
            if honoured { "tmp-honoured" } else { "" },
        ];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("wide={wide}"),
            tags.into_iter().filter(|t| !t.is_empty()).collect(),
        )
    });
}

// ══════════════════════════════════════════════════════════════════════════════════════
// Wave 18c part 3a — the std-stream aliases, __iob_func, and the FILE lock
// ══════════════════════════════════════════════════════════════════════════════════════
//
// Fourteen exports whose whole content is WHICH STREAM they name. `putchar(c)` is
// `fputc(c, stdout)` and nothing else; `getchar()` is `fgetc(stdin)`. `fputc` and `fgetc`
// are already graded (wave 18a), so what is left to grade here is precisely the part a
// reading of the C makes look trivial — and which is exactly where a translation goes
// wrong, because `stdout` is `&_iob[1]` and `stderr` is `&_iob[2]` and an off-by-one
// there produces a function that works perfectly until someone looks at the file.
//
// That is only checkable because each side now owns its three streams: see
// `bind_std_streams`. Every row below resets both sides' stdout, or seeds both sides'
// stdin, before the call — so a case grades what THIS case did rather than an accumulated
// history that would make the first divergence poison every later one.
//
// The standard streams come out of `msvcrt_init_io` in TEXT mode (`WX_TEXT`, because
// `GetFileType` on a regular file answers `FILE_TYPE_DISK`), which is what Windows gives a
// redirected console too. So `\n` really does become `\r\n` on the way out and `\r\n`
// really does collapse on the way in, and the rows draw those bytes deliberately rather
// than hoping the fuzz finds them.
//
// `gets` and `_getws` have NO destination bound — `gets(buf)` is `gets_s(buf, (size_t)-1)`
// — so their rows keep the seed short and the destination guarded. That is not defensive
// padding: an implementation that miscounted would overrun into the guard, and the guard
// bytes are in the compared image.

/// Both sides' stdin, seeded to the same bytes with both streams rewound. Written through
/// the PATH rather than the fd, so the CRT's buffer and offset are reset by `rewind` — the
/// call under test then starts from a known state instead of from the previous case's.
unsafe fn seed_stdin(bytes: &[u8]) {
    std::fs::write(std_path(SUBJ, "in"), bytes).unwrap();
    std::fs::write(std_path(REF, "in"), bytes).unwrap();
    rewind(s_std(0));
    ref_rewind(r_std(0));
}

/// One side's output stream (`1` = stdout, `2` = stderr), emptied and repositioned.
///
/// Three steps and the order is the whole point: `fflush` first, because a byte still in
/// the buffer would otherwise be written AFTER the truncation and land at offset 0 of the
/// next case; then truncate through the path; then `fseek` back to 0, because truncating
/// the file does not move the descriptor and the next write would leave a hole of NULs
/// where the previous case's bytes were.
unsafe fn reset_out(slot: usize) {
    let which = ["in", "out", "err"][slot];
    fflush(s_std(slot));
    ref_fflush(r_std(slot));
    std::fs::write(std_path(SUBJ, which), b"").unwrap();
    std::fs::write(std_path(REF, which), b"").unwrap();
    fseek(s_std(slot), 0, SEEK_SET);
    ref_fseek(r_std(slot), 0, SEEK_SET);
}

unsafe fn reset_stdout() {
    reset_out(1);
}

/// What each side's stdout holds, flushed first. Read by the HOST, never back through the
/// implementation that wrote it.
unsafe fn stdout_bytes() -> (Vec<u8>, Vec<u8>) {
    fflush(s_std(1));
    ref_fflush(r_std(1));
    (
        std::fs::read(std_path(SUBJ, "out")).unwrap_or_default(),
        std::fs::read(std_path(REF, "out")).unwrap_or_default(),
    )
}

/// A byte string safe to hand a NUL-terminated `puts`: no 0x00, and the three bytes text
/// mode treats specially over-weighted (`\n` becomes `\r\n` on the way out, `\r\n`
/// collapses on the way in, and 0x1A ends a text-mode read). The 0x80..0xFF draw is what
/// reaches the multibyte converter on the wide rows, where a byte with no `wchar_t` has to
/// be reported rather than substituted.
fn gen_line(r: &mut Rng, n: usize) -> Vec<u8> {
    (0..n)
        .map(|_| match r.below(8) {
            0 => 0x0A,
            1 => 0x0D,
            2 => 0x1A,
            3 => 0x80 + (r.next() % 0x80) as u8,
            _ => 0x20 + (r.next() % 0x5F) as u8,
        })
        .collect()
}

unsafe fn std_stream_rows() {
    // `getchar` / `_fgetchar` — `getchar` IS `fgetc(stdin)` and `_fgetchar` is the same
    // line again, so both names run one row and staying identical is a checked property.
    grade("getchar+_fgetchar", &["plain", "alias", "ok", "eof"], ROW_ITERS, |r| {
        let seed = r.bytes_upto(24);
        seed_stdin(&seed);
        let alias = r.below(2) == 0;
        let take = 1 + r.below(12);

        let (mut sv, mut rv): (Vec<c_int>, Vec<c_int>) = (Vec::new(), Vec::new());
        let g = Effects::subject(|| {
            for _ in 0..take {
                sv.push(if alias { _fgetchar() } else { getchar() });
            }
            0
        });
        let n = Effects::reference(|| {
            for _ in 0..take {
                rv.push(if alias { ref__fgetchar() } else { ref_getchar() });
            }
            0
        });
        let mut gi: Vec<u8> = sv.iter().flat_map(|c| c.to_le_bytes()).collect();
        let mut ni: Vec<u8> = rv.iter().flat_map(|c| c.to_le_bytes()).collect();
        gi.extend(stream_state_subject(s_std(0)));
        ni.extend(stream_state_reference(r_std(0)));
        let tags = vec![
            if alias { "alias" } else { "plain" },
            if rv.iter().any(|&c| c >= 0) { "ok" } else { "" },
            if rv.contains(&CEOF) { "eof" } else { "" },
        ];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("len={} take={take} alias={alias}", seed.len()),
            tags.into_iter().filter(|t| !t.is_empty()).collect(),
        )
    });

    // `getwchar` / `_fgetwchar` — the wide twin. stdin is an ANSI text descriptor, so
    // `_fgetwc_nolock` takes the `isleadbyte`/`mbtowc` arm rather than assembling two
    // bytes; that converter is shared substrate (§9.73) and what this grades is the stream
    // logic around it.
    grade("getwchar+_fgetwchar", &["plain", "alias", "ok", "weof"], WIDE_ITERS, |r| {
        let seed = r.bytes_upto(24);
        seed_stdin(&seed);
        let alias = r.below(2) == 0;
        let take = 1 + r.below(12);

        let (mut sv, mut rv): (Vec<u16>, Vec<u16>) = (Vec::new(), Vec::new());
        let g = Effects::subject(|| {
            for _ in 0..take {
                sv.push(if alias { _fgetwchar() } else { getwchar() });
            }
            0
        });
        let n = Effects::reference(|| {
            for _ in 0..take {
                rv.push(if alias { ref__fgetwchar() } else { ref_getwchar() });
            }
            0
        });
        let mut gi: Vec<u8> = sv.iter().flat_map(|c| c.to_le_bytes()).collect();
        let mut ni: Vec<u8> = rv.iter().flat_map(|c| c.to_le_bytes()).collect();
        gi.extend(stream_state_subject(s_std(0)));
        ni.extend(stream_state_reference(r_std(0)));
        let tags = vec![
            if alias { "alias" } else { "plain" },
            if rv.iter().any(|&c| c != WEOF) { "ok" } else { "" },
            if rv.contains(&WEOF) { "weof" } else { "" },
        ];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("len={} take={take} alias={alias}", seed.len()),
            tags.into_iter().filter(|t| !t.is_empty()).collect(),
        )
    });

    // `gets` / `_getws` — read a line from stdin into an UNBOUNDED destination.
    //
    // `gets` is `gets_s(buf, (size_t)-1)`, which makes the `!len` arm of gets_s dead from
    // this entry point and the row says so rather than tagging an outcome it cannot reach.
    // What IS reachable, and what the row is really for: `\r` is DROPPED rather than
    // stored, `\n` terminates without being stored, and a read that hits EOF having stored
    // nothing returns NULL — three separate decisions, none of them visible in the return
    // value alone, all of them visible in the destination image.
    grade("gets+_getws", &["narrow", "wide", "line", "nothing"], ROW_ITERS, |r| {
        let wide = r.below(2) == 0;
        // Short and NUL-free: an unbounded destination means the seed length IS the bound.
        let k = r.below(40);
        let seed = gen_line(r, k);
        seed_stdin(&seed);
        let (mut sb, mut rb) = (Buf::new(96), Buf::new(96));
        let (mut swb, mut rwb) = (WBuf::new(96), WBuf::new(96));
        let (sp, rp) = (sb.p(), rb.p());
        let (swp, rwp) = (swb.p(), rwb.p());

        // 0 = NULL, 1 = the destination it was handed, 2 = some OTHER pointer. Both
        // functions contractually return `buf` itself on success, and "returned non-NULL"
        // would grade a body that returned the END of what it wrote as correct.
        let ret_class = |got: *const u8, want: *const u8| -> i64 {
            if got.is_null() {
                0
            } else if got == want {
                1
            } else {
                2
            }
        };
        let g = Effects::subject(|| {
            if wide {
                ret_class(_getws(swp) as *const u8, swp as *const u8)
            } else {
                ret_class(gets(sp), sp)
            }
        });
        let n = Effects::reference(|| {
            if wide {
                ret_class(ref__getws(rwp) as *const u8, rwp as *const u8)
            } else {
                ret_class(ref_gets(rp), rp)
            }
        });
        let (mut gi, mut ni) = if wide { (swb.img(), rwb.img()) } else { (sb.img(), rb.img()) };
        gi.extend(stream_state_subject(s_std(0)));
        ni.extend(stream_state_reference(r_std(0)));
        let tags = vec![
            if wide { "wide" } else { "narrow" },
            if n.ret != 0 { "line" } else { "nothing" },
        ];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("wide={wide} len={}", seed.len()),
            tags,
        )
    });

    // `putchar` / `_fputchar` — one byte to stdout, and the answer is the FILE.
    grade("putchar+_fputchar", &["plain", "alias", "newline", "high"], ROW_ITERS, |r| {
        reset_stdout();
        let alias = r.below(2) == 0;
        let chars: Vec<c_int> =
            (0..1 + r.below(8)).map(|_| gen_line(r, 1)[0] as c_int).collect();

        let (mut sv, mut rv): (Vec<c_int>, Vec<c_int>) = (Vec::new(), Vec::new());
        let g = Effects::subject(|| {
            for &c in &chars {
                sv.push(if alias { _fputchar(c) } else { putchar(c) });
            }
            0
        });
        let n = Effects::reference(|| {
            for &c in &chars {
                rv.push(if alias { ref__fputchar(c) } else { ref_putchar(c) });
            }
            0
        });
        let mut gi: Vec<u8> = sv.iter().flat_map(|c| c.to_le_bytes()).collect();
        let mut ni: Vec<u8> = rv.iter().flat_map(|c| c.to_le_bytes()).collect();
        gi.extend(stream_state_subject(s_std(1)));
        ni.extend(stream_state_reference(r_std(1)));
        let (sd, rd) = stdout_bytes();
        push_disk(&mut gi, Some(sd));
        push_disk(&mut ni, Some(rd));
        let tags = vec![
            if alias { "alias" } else { "plain" },
            if chars.contains(&0x0A) { "newline" } else { "" },
            if chars.iter().any(|&c| c >= 0x7F) { "high" } else { "" },
        ];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("n={} alias={alias}", chars.len()),
            tags.into_iter().filter(|t| !t.is_empty()).collect(),
        )
    });

    // `_fputwchar` — the wide twin, and the only one of the eleven with no narrow partner.
    // stdout is an ANSI text descriptor, so an unconvertible character must be REPORTED
    // (WEOF) rather than emitted as a replacement; `gen_wchar` draws U+20AC for that arm.
    grade("_fputwchar", &["ok", "newline", "unconvertible"], WIDE_ITERS, |r| {
        reset_stdout();
        let chars: Vec<u16> = (0..1 + r.below(8)).map(|_| gen_wchar(r)).collect();

        let (mut sv, mut rv): (Vec<u16>, Vec<u16>) = (Vec::new(), Vec::new());
        let g = Effects::subject(|| {
            for &c in &chars {
                sv.push(_fputwchar(c));
            }
            0
        });
        let n = Effects::reference(|| {
            for &c in &chars {
                rv.push(ref__fputwchar(c));
            }
            0
        });
        let mut gi: Vec<u8> = sv.iter().flat_map(|c| c.to_le_bytes()).collect();
        let mut ni: Vec<u8> = rv.iter().flat_map(|c| c.to_le_bytes()).collect();
        gi.extend(stream_state_subject(s_std(1)));
        ni.extend(stream_state_reference(r_std(1)));
        let (sd, rd) = stdout_bytes();
        push_disk(&mut gi, Some(sd));
        push_disk(&mut ni, Some(rd));
        let tags = vec![
            if rv.iter().any(|&c| c != WEOF) { "ok" } else { "" },
            if chars.contains(&0x000A) { "newline" } else { "" },
            if rv.contains(&WEOF) { "unconvertible" } else { "" },
        ];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("n={}", chars.len()),
            tags.into_iter().filter(|t| !t.is_empty()).collect(),
        )
    });

    // `puts` / `_putws` — a whole string plus a newline, and NEITHER is a one-liner.
    //
    // `puts` measures with `strlen`, writes the body with `_fwrite_nolock`, checks that
    // count, then writes "\n" as a SECOND call and reports `0` or `EOF` — so the return is
    // not the byte count and an implementation that returned one would look right until a
    // caller compared against 0. `_putws` goes through `fputws` and turns any negative
    // into `WEOF`, whose value is 0xFFFF and not -1. Both bracket the pair of writes in
    // ONE `_lock_file`, which is the reason they are not simply `fputs` + `fputc`.
    grade("puts+_putws", &["narrow", "wide", "empty", "newline"], ROW_ITERS, |r| {
        reset_stdout();
        let wide = r.below(2) == 0;
        let k = r.below(24);
        let body = gen_line(r, k);
        let mut narrow = body.clone();
        narrow.push(0);
        let mut widestr: Vec<u16> = body.iter().map(|&b| b as u16).collect();
        widestr.push(0);

        let g = Effects::subject(|| {
            if wide {
                _putws(widestr.as_ptr()) as i64
            } else {
                puts(narrow.as_ptr()) as i64
            }
        });
        let n = Effects::reference(|| {
            if wide {
                ref__putws(widestr.as_ptr()) as i64
            } else {
                ref_puts(narrow.as_ptr()) as i64
            }
        });
        let mut gi = stream_state_subject(s_std(1));
        let mut ni = stream_state_reference(r_std(1));
        let (sd, rd) = stdout_bytes();
        push_disk(&mut gi, Some(sd));
        push_disk(&mut ni, Some(rd));
        let tags = vec![
            if wide { "wide" } else { "narrow" },
            if body.is_empty() { "empty" } else { "" },
            if body.contains(&0x0A) { "newline" } else { "" },
        ];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("wide={wide} len={}", body.len()),
            tags.into_iter().filter(|t| !t.is_empty()).collect(),
        )
    });

    // `__iob_func` — one line (`return iob_get_file(0)`) and NOT gradeable by comparison:
    // the two sides answer different addresses by construction, which is the entire point
    // of the `_iob` rename wall.
    //
    // What IS comparable is what the answer MEANS. Walk it by `IOB_STRIDE` and each slot
    // must be the standard stream that index names — proved by USING it: slot 0 must be
    // the stdin this side's `getchar` reads, slots 1 and 2 must be the files this side's
    // stdout and stderr were bound to. A pointer off by one entry, or a stride off by a
    // field, fails here; a pointer that merely differs between the sides does not.
    grade("__iob_func", &["stdin", "stdout", "stderr"], ROW_ITERS, |r| {
        let i = r.below(3);
        let byte = gen_line(r, 1)[0];
        reset_out(1);
        reset_out(2);
        let k = 1 + r.below(8);
        let seed = gen_line(r, k);
        seed_stdin(&seed);

        let g = Effects::subject(|| {
            let f = (__iob_func() as *mut u8).add(i * IOB_STRIDE) as FILEP;
            let ret = if i == 0 { fgetc(f) } else { fputc(byte as c_int, f) };
            fflush(f);
            ret as i64
        });
        let n = Effects::reference(|| {
            let f = (ref___iob_func() as *mut u8).add(i * IOB_STRIDE) as FILEP;
            let ret = if i == 0 { ref_fgetc(f) } else { ref_fputc(byte as c_int, f) };
            ref_fflush(f);
            ret as i64
        });
        let which = ["in", "out", "err"][i];
        let mut gi = stream_state_subject(s_std(i));
        let mut ni = stream_state_reference(r_std(i));
        push_disk(&mut gi, std::fs::read(std_path(SUBJ, which)).ok());
        push_disk(&mut ni, std::fs::read(std_path(REF, which)).ok());
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("slot={i}"),
            vec![["stdin", "stdout", "stderr"][i]],
        )
    });

    // `_lock_file` / `_unlock_file` — two arms, and the arm chosen is the whole function.
    //
    // A FILE inside the `_iob` array takes the GLOBAL lock `_STREAM_LOCKS + (file - _iob)`;
    // any other FILE enters its own CRITICAL_SECTION. Neither is visible in a return value
    // — both are `void` — so each arm needs its own window:
    //
    //   * the global-lock arm reports through `ref_get_last_lock` (fileimpl.c), which
    //     records the slot number `_lock` was handed. That number is `28 + index`, so it
    //     catches a wrong index as well as a wrong arm.
    //   * the critical-section arm reports through the section's own `RecursionCount`,
    //     which lives in the FILE's memory and is therefore per-side already.
    //
    // The heap FILE comes from `fopen`, not from `tmpfile`: it has to be a FILE OUTSIDE
    // the array, and the row asserts that by checking the arm the reference took.
    grade("_lock_file+_unlock_file", &["iob", "heap"], ROW_ITERS, |r| {
        // 20 = _IOB_ENTRIES. Beyond it fileimpl.c's ref_lock_slot aborts, which is the
        // correct response to a lock number msvcrt cannot produce.
        let iob = r.below(2) == 0;
        let idx = r.below(20);
        let w = World::new("lockfile", b"vibeos");

        let (sf, rf) = if iob {
            (s_std(idx), r_std(idx))
        } else {
            (fopen(w.s(), b"rb\0".as_ptr()), ref_fopen(w.r(), b"rb\0".as_ptr()))
        };
        let recursion = |f: FILEP| -> c_int {
            if iob {
                -1
            } else {
                (*(f as *const FileCrit)).crit.recursion_count
            }
        };

        // The `ret` channel carries the state WHILE the lock is held — the slot `_lock`
        // was handed, and the section's recursion depth — because both are gone by the
        // time `_unlock_file` returns. The image carries what is left afterwards.
        let held = |lock: c_int, rec: c_int| -> i64 { ((lock as i64) << 32) | (rec as u32) as i64 };

        ref_reset_last_locks();
        let g = Effects::subject(|| {
            _lock_file(sf);
            let inside = held(ref_get_last_lock(), recursion(sf));
            _unlock_file(sf);
            inside
        });
        let mut gi = Vec::new();
        gi.extend_from_slice(&ref_get_last_unlock().to_le_bytes());
        gi.extend_from_slice(&recursion(sf).to_le_bytes());
        ref_reset_last_locks();
        let n = Effects::reference(|| {
            ref__lock_file(rf);
            let inside = held(ref_get_last_lock(), recursion(rf));
            ref__unlock_file(rf);
            inside
        });
        let mut ni = Vec::new();
        ni.extend_from_slice(&ref_get_last_unlock().to_le_bytes());
        ni.extend_from_slice(&recursion(rf).to_le_bytes());
        if !iob {
            fclose(sf);
            ref_fclose(rf);
        }
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("iob={iob} idx={idx}"),
            vec![if iob { "iob" } else { "heap" }],
        )
    });
}

// ══════════════════════════════════════════════════════════════════════════════════════
// Wave 18c part 3b — the sixteen `v*printf` shells
// ══════════════════════════════════════════════════════════════════════════════════════
//
// These are the last of file.c's exports, and the reason they went last is that the
// reference is not callable. Every one forwards `&valist` to the engine as a `va_list*`,
// an idiom coherent only where a `va_list` IS a plain pointer — so under the host's SysV
// register-save-area model `*valist` reinterprets the decayed pointer variable as the tag
// struct and reads wild memory. `fileflat_*` is file.c compiled a SECOND time under
// refstub_printf's flat `char*` `<stdarg.h>`, which is the Windows x64 / aarch64 / i386
// model; it is both the callable instantiation and the target-faithful one. Same
// diagnosis and same fix as `pfflat_` for wcs.c — see build.rs.
//
// # What is left to grade once the engine is out of scope
//
// `printf_port.rs` grades the ENGINE and `printf_shells.rs` grades the string-writing
// shells around it. What neither can see is the two helpers these sixteen funnel through,
// `vfprintf_helper` and `vfwprintf_helper`, and their decisions are all about the STREAM:
//
//   * which `options` bits the shell contributes — `_s` adds
//     INVOKE_INVALID_PARAM_HANDLER, `_p` adds POSITIONAL_PARAMS on top of it, plain adds
//     neither, and a shell that mixed them up would still format perfectly;
//   * the NULL checks, which are `MSVCRT_CHECK_PMT` — so a NULL file or format must
//     return -1 AND fire the invalid-parameter handler, not just return -1;
//   * `create_positional_ctx`'s three outcomes: found positional specifiers (use them),
//     found none (drop the flag and re-run as ordinary), or malformed (fire the handler,
//     set EINVAL, and return its negative code);
//   * that the pair of writes is bracketed by ONE `_lock_file` and that `add_std_buffer`'s
//     temporary buffer, if taken, is removed again.
//
// # Two limits, stated rather than implied
//
// `add_std_buffer` is UNREACHABLE here and would be on any redirected stream: it requires
// `_isatty(file->_file)`, and these standard streams are regular files. That is the same
// answer a Windows CI job with redirected output gets, so it is a property of the
// environment rather than a gap in the harness — but it does mean `tmp_buf` is always
// FALSE and the `remove_std_buffer` arm never runs.
//
// The subject's engine is Rust's `pf_printf_a` and the reference's is `pfflat_pf_printf_a`;
// a divergence in FORMATTING would show up here as well as in printf_port, and printf_port
// is where it would be diagnosed. These rows keep a `%I64d` case in the corpus for the
// same reason printf_shells does — glibc prints `I64d` literally, so an oracle that ever
// escaped to the host libc fails loudly instead of grading quietly.

/// One argument, as it goes into the packed slot area.
#[derive(Clone, Copy)]
enum A {
    I(i32),
    L(i64),
    F(f64),
    S(&'static [u8]),
    W(&'static [u16]),
    P(u64),
}

/// Hand-packed argument area: one 8-byte slot per argument, which is the Windows-x64 and
/// aarch64 model and the one the flat reference was compiled for. The subject reads it
/// the same way — msvcrt-crt-rust's `va_list` is `*mut u8` on every target.
fn vpack(args: &[A]) -> Vec<u64> {
    let mut slots: Vec<u64> = args
        .iter()
        .map(|a| match *a {
            A::I(v) => v as u32 as u64,
            A::L(v) => v as u64,
            A::F(v) => v.to_bits(),
            A::S(b) => b.as_ptr() as u64,
            A::W(w) => w.as_ptr() as u64,
            A::P(p) => p,
        })
        .collect();
    // Slack: an engine that over-reads finds zeros rather than the harness's stack.
    slots.extend([0u64; 4]);
    slots
}

static VW_HELLO: &[u16] = &[b'v' as u16, b'i' as u16, b'b' as u16, b'e' as u16, b'o' as u16, 0];
static VW_POS: &[u16] = &[b'p' as u16, b'o' as u16, b's' as u16, 0];

/// `(format, args, kind)` — `kind` is what the format needs from the shell, so a row can
/// hand a positional format only to a `_p` variant and still reach the "no specifiers
/// found, drop the flag" arm on the others.
#[derive(Clone, Copy, PartialEq)]
enum K {
    Ordinary,
    Positional,
    MalformedPositional,
}

fn vprintf_corpus() -> Vec<(&'static str, Vec<A>, K)> {
    vec![
        ("plain text, no conversions", vec![], K::Ordinary),
        ("%d", vec![A::I(42)], K::Ordinary),
        ("%d %d %d", vec![A::I(-1), A::I(0), A::I(2147483647)], K::Ordinary),
        ("%u %x %X %o", vec![A::I(-1), A::I(0xDEAD), A::I(0xBEEF), A::I(64)], K::Ordinary),
        ("[%s]", vec![A::S(b"vibeos\0")], K::Ordinary),
        ("[%10s][%-10s]", vec![A::S(b"ab\0"), A::S(b"cd\0")], K::Ordinary),
        ("%c%c%c", vec![A::I(b'v' as i32), A::I(b'!' as i32), A::I(b'\n' as i32)], K::Ordinary),
        ("%f %e %g", vec![A::F(3.5), A::F(1e10), A::F(0.000125)], K::Ordinary),
        ("%.3f|%+.0f", vec![A::F(2.0 / 3.0), A::F(-0.5)], K::Ordinary),
        // The oracle canary: glibc has no %I64 conversion and prints it literally.
        ("%I64d", vec![A::L(-1_234_567_890_123)], K::Ordinary),
        ("%p", vec![A::P(0x1234_5678)], K::Ordinary),
        ("%ls", vec![A::W(VW_HELLO)], K::Ordinary),
        ("100%% done", vec![], K::Ordinary),
        ("%*d", vec![A::I(8), A::I(7)], K::Ordinary),
        ("%1$s-%2$d", vec![A::S(b"pos\0"), A::I(9)], K::Positional),
        ("%2$d/%1$s/%2$d", vec![A::S(b"twice\0"), A::I(5)], K::Positional),
        ("%0$d", vec![A::I(1)], K::MalformedPositional),
    ]
}

/// The wide corpus is the narrow one with `%s` taking a WIDE string, because that is what
/// `%s` means to `pf_printf_w`. Spelling it out rather than deriving it keeps the two
/// readable side by side.
fn vwprintf_corpus() -> Vec<(&'static str, Vec<A>, K)> {
    vec![
        ("plain text, no conversions", vec![], K::Ordinary),
        ("%d", vec![A::I(42)], K::Ordinary),
        ("%d %d %d", vec![A::I(-1), A::I(0), A::I(2147483647)], K::Ordinary),
        ("%u %x %X %o", vec![A::I(-1), A::I(0xDEAD), A::I(0xBEEF), A::I(64)], K::Ordinary),
        ("[%s]", vec![A::W(VW_HELLO)], K::Ordinary),
        ("[%10s][%-10s]", vec![A::W(VW_POS), A::W(VW_HELLO)], K::Ordinary),
        ("%c%c%c", vec![A::I(b'v' as i32), A::I(b'!' as i32), A::I(b'\n' as i32)], K::Ordinary),
        ("%f %e %g", vec![A::F(3.5), A::F(1e10), A::F(0.000125)], K::Ordinary),
        ("%.3f|%+.0f", vec![A::F(2.0 / 3.0), A::F(-0.5)], K::Ordinary),
        ("%I64d", vec![A::L(-1_234_567_890_123)], K::Ordinary),
        ("%p", vec![A::P(0x1234_5678)], K::Ordinary),
        ("%hs", vec![A::S(b"narrow\0")], K::Ordinary),
        ("100%% done", vec![], K::Ordinary),
        ("%*d", vec![A::I(8), A::I(7)], K::Ordinary),
        ("%1$s-%2$d", vec![A::W(VW_POS), A::I(9)], K::Positional),
        ("%2$d/%1$s/%2$d", vec![A::W(VW_HELLO), A::I(5)], K::Positional),
        ("%0$d", vec![A::I(1)], K::MalformedPositional),
    ]
}

fn wide_of(s: &str) -> Vec<u16> {
    let mut v: Vec<u16> = s.bytes().map(|b| b as u16).collect();
    v.push(0);
    v
}

/// A scratch destination FILE on the subject and on the flat reference.
///
/// Opened WITHOUT a variadic entry point, which is a hard requirement rather than a style
/// choice: `fopen` reaches `_wsopen`, and in the flat instantiation `va_start` yields NULL
/// so its `_O_CREAT` arm would read a pmode through a null pointer. `_sopen_s` + `_fdopen`
/// is the same descriptor and the same FILE with no `va_arg` anywhere on the path, and
/// both sides take it so the two FILEs start out identical.
struct VWorld {
    sf: FILEP,
    ff: FILEP,
    sfd: c_int,
    ffd: c_int,
    sp: PathBuf,
    fp: PathBuf,
}

impl VWorld {
    unsafe fn new(tag: &str) -> VWorld {
        let (sc, sp) = tmp_path(&format!("{tag}-s"));
        let (fc, fp) = tmp_path(&format!("{tag}-f"));
        let flags = O_CREAT | O_TRUNC | O_RDWR | O_BINARY;
        let (mut sfd, mut ffd) = (-1, -1);
        assert_eq!(
            _sopen_s(&mut sfd, sc.as_ptr() as *const u8, flags, SHARE_MODES[0], S_IREAD | S_IWRITE),
            0
        );
        assert_eq!(
            fileflat__sopen_s(
                &mut ffd,
                fc.as_ptr() as *const u8,
                flags,
                SHARE_MODES[0],
                S_IREAD | S_IWRITE
            ),
            0
        );
        let sf = _fdopen(sfd, b"wb+\0".as_ptr());
        let ff = fileflat__fdopen(ffd, b"wb+\0".as_ptr());
        assert!(!sf.is_null() && !ff.is_null(), "_fdopen refused a descriptor _sopen_s made");
        VWorld { sf, ff, sfd, ffd, sp, fp }
    }

    /// What each side wrote, flushed first and read back by the HOST.
    unsafe fn bytes(&self) -> (Vec<u8>, Vec<u8>) {
        fflush(self.sf);
        fileflat_fflush(self.ff);
        (
            std::fs::read(&self.sp).unwrap_or_default(),
            std::fs::read(&self.fp).unwrap_or_default(),
        )
    }
}

impl Drop for VWorld {
    fn drop(&mut self) {
        unsafe {
            fclose(self.sf);
            fileflat_fclose(self.ff);
        }
        let _ = std::fs::remove_file(&self.sp);
        let _ = std::fs::remove_file(&self.fp);
    }
}

/// The stream state a `v*printf` row compares. `_fileno` is checked against the descriptor
/// `_fdopen` was HANDED rather than compared across the sides: the flat world is a third
/// fd table and its numbering has no contract with the subject's, but "the FILE reports
/// the fd it was built on" is a contract each side owes on its own.
unsafe fn vstate_subject(f: FILEP, fd: c_int) -> Vec<u8> {
    let mut v = vec![(_fileno(f) == fd) as u8];
    v.extend_from_slice(&ftell(f).to_le_bytes());
    v.push((feof(f) != 0) as u8);
    v.push((ferror(f) != 0) as u8);
    v.extend(buffer_state(f));
    v
}

unsafe fn vstate_flat(f: FILEP, fd: c_int) -> Vec<u8> {
    let mut v = vec![(fileflat__fileno(f) == fd) as u8];
    v.extend_from_slice(&fileflat_ftell(f).to_le_bytes());
    v.push((fileflat_feof(f) != 0) as u8);
    v.push((fileflat_ferror(f) != 0) as u8);
    v.extend(buffer_state(f));
    v
}

/// The subject's stdout and the FLAT side's, emptied and repositioned. The reference's is
/// left alone — these rows never touch it.
unsafe fn reset_flat_stdout() {
    fflush(s_std(1));
    fileflat_fflush(f_std(1));
    std::fs::write(std_path(SUBJ, "out"), b"").unwrap();
    std::fs::write(std_path(FLAT, "out"), b"").unwrap();
    fseek(s_std(1), 0, SEEK_SET);
    fileflat_fseek(f_std(1), 0, SEEK_SET);
}

/// Which of the six narrow FILE shells a case drives, and what `options` it contributes.
/// The order is the declaration order in file.c, so the table reads against the source.
const NARROW_SHELLS: usize = 6;
const WIDE_SHELLS: usize = 6;

unsafe fn vprintf_rows() {
    // The six narrow shells that take a FILE.
    grade(
        "vfprintf family",
        &["plain", "secure", "positional", "pos-dropped", "malformed", "nullfile", "nullfmt"],
        ROW_ITERS,
        |r| {
            let corpus = vprintf_corpus();
            let (fmt, args, kind) = corpus[r.below(corpus.len())].clone();
            // A `_p` shell for a positional format, any shell for an ordinary one — so
            // the "found no specifiers, drop the flag" arm is reached on the _p variants
            // too, and the positional arm is never asked of a shell that cannot take it.
            let which = if kind == K::Ordinary { r.below(NARROW_SHELLS) } else { 4 + r.below(2) };
            let bad = r.below(8);
            let (nullfile, nullfmt) = (bad == 0, bad == 1);

            let mut cfmt = fmt.as_bytes().to_vec();
            cfmt.push(0);
            let fp: *const u8 = if nullfmt { core::ptr::null() } else { cfmt.as_ptr() };
            let mut slots = vpack(&args);
            let vp = slots.as_mut_ptr() as *mut u8;
            let w = VWorld::new("vfprintf");
            let (sf, ff) = if nullfile { (core::ptr::null_mut(), core::ptr::null_mut()) } else { (w.sf, w.ff) };
            let loc: *mut c_void = core::ptr::null_mut();

            let g = Effects::subject(|| {
                match which {
                    0 => vfprintf(sf, fp, vp),
                    1 => vfprintf_s(sf, fp, vp),
                    2 => _vfprintf_l(sf, fp, loc, vp),
                    3 => _vfprintf_s_l(sf, fp, loc, vp),
                    4 => _vfprintf_p(sf, fp, vp),
                    _ => _vfprintf_p_l(sf, fp, loc, vp),
                }
                .into()
            });
            let n = Effects::reference(|| {
                match which {
                    0 => fileflat_vfprintf(ff, fp, vp),
                    1 => fileflat_vfprintf_s(ff, fp, vp),
                    2 => fileflat__vfprintf_l(ff, fp, loc, vp),
                    3 => fileflat__vfprintf_s_l(ff, fp, loc, vp),
                    4 => fileflat__vfprintf_p(ff, fp, vp),
                    _ => fileflat__vfprintf_p_l(ff, fp, loc, vp),
                }
                .into()
            });
            let mut gi = vstate_subject(w.sf, w.sfd);
            let mut ni = vstate_flat(w.ff, w.ffd);
            let (sd, fd_) = w.bytes();
            push_disk(&mut gi, Some(sd));
            push_disk(&mut ni, Some(fd_));
            let tags = vec![
                match which {
                    0 | 2 => "plain",
                    1 | 3 => "secure",
                    _ if kind == K::Positional => "positional",
                    _ if kind == K::Ordinary => "pos-dropped",
                    _ => "malformed",
                },
                if nullfile { "nullfile" } else { "" },
                if nullfmt { "nullfmt" } else { "" },
            ];
            (
                Obs::with(g, gi),
                Obs::with(n, ni),
                format!("which={which} fmt={fmt:?} nullfile={nullfile} nullfmt={nullfmt}"),
                tags.into_iter().filter(|t| !t.is_empty()).collect(),
            )
        },
    );

    // The six wide shells that take a FILE. Same structure, and the destination is a
    // BINARY stream on both sides so `pf_printf_w`'s output goes through `wctomb` in one
    // place rather than being re-translated by the text layer underneath it.
    grade(
        "vfwprintf family",
        &["plain", "secure", "positional", "pos-dropped", "malformed", "nullfile", "nullfmt"],
        ROW_ITERS,
        |r| {
            let corpus = vwprintf_corpus();
            let (fmt, args, kind) = corpus[r.below(corpus.len())].clone();
            let which = if kind == K::Ordinary { r.below(WIDE_SHELLS) } else { 4 + r.below(2) };
            let bad = r.below(8);
            let (nullfile, nullfmt) = (bad == 0, bad == 1);

            let wfmt = wide_of(fmt);
            let fp: *const u16 = if nullfmt { core::ptr::null() } else { wfmt.as_ptr() };
            let mut slots = vpack(&args);
            let vp = slots.as_mut_ptr() as *mut u8;
            let w = VWorld::new("vfwprintf");
            let (sf, ff) = if nullfile { (core::ptr::null_mut(), core::ptr::null_mut()) } else { (w.sf, w.ff) };
            let loc: *mut c_void = core::ptr::null_mut();

            let g = Effects::subject(|| {
                match which {
                    0 => vfwprintf(sf, fp, vp),
                    1 => vfwprintf_s(sf, fp, vp),
                    2 => _vfwprintf_l(sf, fp, loc, vp),
                    3 => _vfwprintf_s_l(sf, fp, loc, vp),
                    4 => _vfwprintf_p(sf, fp, vp),
                    _ => _vfwprintf_p_l(sf, fp, loc, vp),
                }
                .into()
            });
            let n = Effects::reference(|| {
                match which {
                    0 => fileflat_vfwprintf(ff, fp, vp),
                    1 => fileflat_vfwprintf_s(ff, fp, vp),
                    2 => fileflat__vfwprintf_l(ff, fp, loc, vp),
                    3 => fileflat__vfwprintf_s_l(ff, fp, loc, vp),
                    4 => fileflat__vfwprintf_p(ff, fp, vp),
                    _ => fileflat__vfwprintf_p_l(ff, fp, loc, vp),
                }
                .into()
            });
            let mut gi = vstate_subject(w.sf, w.sfd);
            let mut ni = vstate_flat(w.ff, w.ffd);
            let (sd, fd_) = w.bytes();
            push_disk(&mut gi, Some(sd));
            push_disk(&mut ni, Some(fd_));
            let tags = vec![
                match which {
                    0 | 2 => "plain",
                    1 | 3 => "secure",
                    _ if kind == K::Positional => "positional",
                    _ if kind == K::Ordinary => "pos-dropped",
                    _ => "malformed",
                },
                if nullfile { "nullfile" } else { "" },
                if nullfmt { "nullfmt" } else { "" },
            ];
            (
                Obs::with(g, gi),
                Obs::with(n, ni),
                format!("which={which} fmt={fmt:?} nullfile={nullfile} nullfmt={nullfmt}"),
                tags.into_iter().filter(|t| !t.is_empty()).collect(),
            )
        },
    );

    // `vprintf` / `vprintf_s` / `vwprintf` / `vwprintf_s` — the four that name `stdout`.
    //
    // One line each (`return vfprintf(stdout, format, valist)`), and the line is the whole
    // function: the only thing to get wrong is WHICH `_iob` entry, which is invisible in a
    // return value and obvious in the file. Graded exactly as the aliases above are.
    grade("vprintf+vwprintf", &["narrow", "wide", "plain", "secure", "nullfmt"], ROW_ITERS, |r| {
        reset_flat_stdout();
        let wide = r.below(2) == 0;
        let secure = r.below(2) == 0;
        let corpus = if wide { vwprintf_corpus() } else { vprintf_corpus() };
        // The stdout four are `vfprintf`/`vfprintf_s`, neither of which carries
        // POSITIONAL_PARAMS — a positional format would take the "no specifiers" arm and
        // print the text verbatim on both sides, which is true but uninteresting here.
        let ordinary: Vec<_> = corpus.into_iter().filter(|c| c.2 == K::Ordinary).collect();
        let (fmt, args, _) = ordinary[r.below(ordinary.len())].clone();
        let nullfmt = r.below(8) == 0;

        let mut cfmt = fmt.as_bytes().to_vec();
        cfmt.push(0);
        let wfmt = wide_of(fmt);
        let np: *const u8 = if nullfmt { core::ptr::null() } else { cfmt.as_ptr() };
        let wp: *const u16 = if nullfmt { core::ptr::null() } else { wfmt.as_ptr() };
        let mut slots = vpack(&args);
        let vp = slots.as_mut_ptr() as *mut u8;

        let g = Effects::subject(|| {
            match (wide, secure) {
                (false, false) => vprintf(np, vp),
                (false, true) => vprintf_s(np, vp),
                (true, false) => vwprintf(wp, vp),
                (true, true) => vwprintf_s(wp, vp),
            }
            .into()
        });
        let n = Effects::reference(|| {
            match (wide, secure) {
                (false, false) => fileflat_vprintf(np, vp),
                (false, true) => fileflat_vprintf_s(np, vp),
                (true, false) => fileflat_vwprintf(wp, vp),
                (true, true) => fileflat_vwprintf_s(wp, vp),
            }
            .into()
        });
        let mut gi = buffer_state(s_std(1));
        let mut ni = buffer_state(f_std(1));
        fflush(s_std(1));
        fileflat_fflush(f_std(1));
        push_disk(&mut gi, std::fs::read(std_path(SUBJ, "out")).ok());
        push_disk(&mut ni, std::fs::read(std_path(FLAT, "out")).ok());
        let tags = vec![
            if wide { "wide" } else { "narrow" },
            if secure { "secure" } else { "plain" },
            if nullfmt { "nullfmt" } else { "" },
        ];
        (
            Obs::with(g, gi),
            Obs::with(n, ni),
            format!("wide={wide} secure={secure} fmt={fmt:?} nullfmt={nullfmt}"),
            tags.into_iter().filter(|t| !t.is_empty()).collect(),
        )
    });
}
