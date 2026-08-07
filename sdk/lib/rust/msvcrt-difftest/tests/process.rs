//! Grades process.c — reference TU #12. All 41 exports, over processimpl.c's recording
//! CreateProcessW and a real temp-directory filesystem.
//!
//! WHAT THIS FILE ACTUALLY COMPARES, AND WHY IT IS NOT THE RETURN VALUE. process.c does
//! not create processes. It MARSHALS: an `argv` array or a `...` list into one
//! space-separated command line, an `envp` array into a double-NUL-terminated block, a
//! bare program name into a full path by walking `PATH` and trying `.com`/`.exe`/`.bat`/
//! `.cmd`, and a `_P_*` mode into a creation-flags word plus a wait-or-return decision.
//! Then it calls CreateProcessW. Everything the file is FOR happens before that call — so
//! a harness that only compared return values would grade two -1s against each other and
//! report a confident green over the entire module. processimpl.c's CreateProcessW is
//! therefore a RECORDER, and every row compares the recorded call: application name,
//! command line, the whole environment block including its embedded NULs, the creation
//! flags, the inherit flag, and the STARTUPINFOW fields msvcrt fills.
//!
//! THE TWO va_list MODELS NEVER MEET. Sixteen of the 41 exports are WINAPIV. The subject
//! cannot supply them — `va_start` is not on pinned stable, so the shipping msvcrt has
//! them as C trampolines in `rust_process_varargs_shim.c` over `_v`-prefixed Rust twins,
//! and those twins walk a flat `*mut u8` (the Windows-x64 va_list) where this host's is a
//! SysV register-save-area struct. So the REFERENCE is called with real host varargs and
//! the SUBJECT's `_v*` twin is handed the same logical arguments hand-packed into 8-byte
//! slots, and what gets compared is the recorded spawn. Neither side ever sees the other's
//! cursor. (The scanf TU solved the same collision by compiling the reference against a
//! flat `<stdarg.h>`; that does not work here, because these bodies genuinely START their
//! own lists and the flat header's `va_start(ap, last)` expands to `ap = NULL`.)
//!
//! THE `_exec*` SUCCESS PATH CANNOT BE A ROW, AND THAT IS A FINDING NOT A GAP. Every
//! `_exec*` passes `_P_OVERLAY`, and msvcrt_spawn's `_P_OVERLAY` arm is `_exit(0)` — on
//! success the harness process terminates. So exec rows preset CreateProcessW to FAIL,
//! which still grades everything up to and including the errno mapping; only the two
//! instructions after the call are out of reach. processimpl.c redirects `_exit` to a
//! terminator that exits 70 with a message, so if a future row ever does reach that arm it
//! fails loudly instead of exiting zero having compared nothing.
//!
//! WHAT IS SHARED AND WHAT IS NOT. The ledger is ONE recorder, and that is not the §9.73
//! trap: nothing in process.c ever reads it, and the row protocol is reset → call one side
//! → snapshot → reset → call the other → snapshot → compare the snapshots. The filesystem
//! and `PATH` are shared for the same reason fileimpl.c's are — they are what the
//! environment IS. Everything process.c OWNS is already per-side: `popen_handles` is a
//! file-static in the reference TU and a Rust static in the subject, and the two fd tables
//! and FILE rings belong to file.c and file.rs respectively.

#![allow(non_snake_case)]

extern crate msvcrt_crt_rust;

use core::ffi::{c_char, c_int, c_uint, c_void};
use std::io::Write;

// ══════════════════════════════════════════════════════════════════════════════════════
// The substrate: processimpl.c's knobs and its ledger
// ══════════════════════════════════════════════════════════════════════════════════════

extern "C" {
    fn ref_proc_reset();
    fn ref_proc_set_create_result(ok: c_int, last_error: u32, pid: u32, tid: u32);
    fn ref_proc_set_wait_result(result: u32, last_error: u32);
    fn ref_proc_set_exit_code(ok: c_int, value: u32);
    fn ref_proc_set_module_results(load_ok: c_int, free_ok: c_int, proc_ok: c_int);
    fn ref_proc_set_comspec(value: *const u16);

    fn ref_proc_calls() -> c_int;
    fn ref_proc_truncated() -> c_int;
    fn ref_proc_app_null() -> c_int;
    fn ref_proc_app() -> *const u16;
    fn ref_proc_cmdline_null() -> c_int;
    fn ref_proc_cmdline() -> *const u16;
    fn ref_proc_env_null() -> c_int;
    fn ref_proc_env() -> *const u16;
    fn ref_proc_env_len() -> c_uint;
    fn ref_proc_cwd_null() -> c_int;
    fn ref_proc_flags() -> u32;
    fn ref_proc_inherit() -> c_int;
    fn ref_proc_attrs_null() -> c_int;
    fn ref_proc_si_cb() -> u32;
    fn ref_proc_si_flags() -> u32;
    fn ref_proc_si_cbreserved2() -> c_uint;
    fn ref_proc_si_reserved2_null() -> c_int;
    fn ref_proc_si_inherit_len() -> c_uint;
    fn ref_proc_si_inherit() -> *const u8;
    fn ref_proc_si_stdin() -> i64;
    fn ref_proc_si_stdout() -> i64;
    fn ref_proc_si_stderr() -> i64;

    fn ref_proc_loadlibrary_calls() -> c_int;
    fn ref_proc_loadlibrary_null() -> c_int;
    fn ref_proc_loadlibrary_name() -> *const u16;
    fn ref_proc_freelibrary_calls() -> c_int;
    fn ref_proc_freelibrary_arg() -> i64;
    fn ref_proc_getprocaddr_calls() -> c_int;
    fn ref_proc_getprocaddr_mod() -> i64;
    fn ref_proc_getprocaddr_raw() -> u64;
    fn ref_proc_getprocaddr_is_name() -> c_int;
    fn ref_proc_getprocaddr_name() -> *const c_char;

    fn ref_proc_hprocess() -> i64;
    fn ref_proc_hmodule() -> i64;
    fn ref_proc_procaddr() -> i64;
}

// The errno slots, one per side. errno.c is reference TU #7, so these are two genuine
// msvcrt mappers and not a shared stand-in — which matters here because
// `msvcrt_set_errno(GetLastError())` is what every failed spawn returns through.
extern "C" {
    fn subj_errno_get() -> c_int;
    fn subj_errno_set(v: c_int);
    fn subj_invalid_pmt_count() -> c_int;
    fn subj_invalid_pmt_reset();
    fn ref_errno_get() -> c_int;
    fn ref_errno_set(v: c_int);
    fn ref_invalid_pmt_count() -> c_int;
    fn ref_invalid_pmt_reset();
}

// ══════════════════════════════════════════════════════════════════════════════════════
// The two sides
// ══════════════════════════════════════════════════════════════════════════════════════

type W = *const u16;
type A = *const c_char;

// ── the SUBJECT: msvcrt-crt-rust's process.rs ──
extern "C" {
    fn _cwait(status: *mut c_int, pid: isize, action: c_int) -> isize;

    fn _wexecv(name: W, argv: *const W) -> isize;
    fn _execv(name: A, argv: *const A) -> isize;
    fn _wexecve(name: W, argv: *const W, envv: *const W) -> isize;
    fn _execve(name: A, argv: *const A, envv: *const A) -> isize;
    fn _wexecvp(name: W, argv: *const W) -> isize;
    fn _execvp(name: A, argv: *const A) -> isize;
    fn _wexecvpe(name: W, argv: *const W, envv: *const W) -> isize;
    fn _execvpe(name: A, argv: *const A, envv: *const A) -> isize;

    fn _wspawnv(flags: c_int, name: W, argv: *const W) -> isize;
    fn _spawnv(flags: c_int, name: A, argv: *const A) -> isize;
    fn _wspawnve(flags: c_int, name: W, argv: *const W, envv: *const W) -> isize;
    fn _spawnve(flags: c_int, name: A, argv: *const A, envv: *const A) -> isize;
    fn _wspawnvp(flags: c_int, name: W, argv: *const W) -> isize;
    fn _spawnvp(flags: c_int, name: A, argv: *const A) -> isize;
    fn _wspawnvpe(flags: c_int, name: W, argv: *const W, envv: *const W) -> isize;
    fn _spawnvpe(flags: c_int, name: A, argv: *const A, envv: *const A) -> isize;

    // The `_v` twins the WINAPIV shells trampoline into. `va` is the flat byte cursor.
    fn _vwexecl(name: W, arg0: W, va: *mut u8) -> isize;
    fn _vexecl(name: A, arg0: A, va: *mut u8) -> isize;
    fn _vwexecle(name: W, arg0: W, va: *mut u8) -> isize;
    fn _vexecle(name: A, arg0: A, va: *mut u8) -> isize;
    fn _vwexeclp(name: W, arg0: W, va: *mut u8) -> isize;
    fn _vexeclp(name: A, arg0: A, va: *mut u8) -> isize;
    fn _vwexeclpe(name: W, arg0: W, va: *mut u8) -> isize;
    fn _vexeclpe(name: A, arg0: A, va: *mut u8) -> isize;
    fn _vwspawnl(flags: c_int, name: W, arg0: W, va: *mut u8) -> isize;
    fn _vspawnl(flags: c_int, name: A, arg0: A, va: *mut u8) -> isize;
    fn _vwspawnle(flags: c_int, name: W, arg0: W, va: *mut u8) -> isize;
    fn _vspawnle(flags: c_int, name: A, arg0: A, va: *mut u8) -> isize;
    fn _vwspawnlp(flags: c_int, name: W, arg0: W, va: *mut u8) -> isize;
    fn _vspawnlp(flags: c_int, name: A, arg0: A, va: *mut u8) -> isize;
    fn _vwspawnlpe(flags: c_int, name: W, arg0: W, va: *mut u8) -> isize;
    fn _vspawnlpe(flags: c_int, name: A, arg0: A, va: *mut u8) -> isize;

    fn _wpopen(command: W, mode: W) -> *mut c_void;
    fn _popen(command: A, mode: A) -> *mut c_void;
    fn _pclose(file: *mut c_void) -> c_int;
    fn msvcrt_free_popen_data();

    fn _loaddll(dllname: A) -> isize;
    fn _unloaddll(dll: isize) -> c_int;
    fn _getdllprocaddr(dll: isize, name: A, ordinal: c_int) -> *mut c_void;
    fn _getpid() -> c_int;

    // file.rs's, for `label_handle` — the SUBJECT's fd table, which is a different table
    // from the reference's and caches different OS handles for the same CRT descriptor.
    fn _get_osfhandle(fd: c_int) -> isize;
}

// ── the REFERENCE: the genuine process.c, compiled for the host ──
extern "C" {
    fn ref__cwait(status: *mut c_int, pid: isize, action: c_int) -> isize;

    fn ref__wexecv(name: W, argv: *const W) -> isize;
    fn ref__execv(name: A, argv: *const A) -> isize;
    fn ref__wexecve(name: W, argv: *const W, envv: *const W) -> isize;
    fn ref__execve(name: A, argv: *const A, envv: *const A) -> isize;
    fn ref__wexecvp(name: W, argv: *const W) -> isize;
    fn ref__execvp(name: A, argv: *const A) -> isize;
    fn ref__wexecvpe(name: W, argv: *const W, envv: *const W) -> isize;
    fn ref__execvpe(name: A, argv: *const A, envv: *const A) -> isize;

    fn ref__wspawnv(flags: c_int, name: W, argv: *const W) -> isize;
    fn ref__spawnv(flags: c_int, name: A, argv: *const A) -> isize;
    fn ref__wspawnve(flags: c_int, name: W, argv: *const W, envv: *const W) -> isize;
    fn ref__spawnve(flags: c_int, name: A, argv: *const A, envv: *const A) -> isize;
    fn ref__wspawnvp(flags: c_int, name: W, argv: *const W) -> isize;
    fn ref__spawnvp(flags: c_int, name: A, argv: *const A) -> isize;
    fn ref__wspawnvpe(flags: c_int, name: W, argv: *const W, envv: *const W) -> isize;
    fn ref__spawnvpe(flags: c_int, name: A, argv: *const A, envv: *const A) -> isize;

    // Real C variadics — see the module docs for why these are NOT called through a
    // flat cursor the way their subject twins are.
    fn ref__wexecl(name: W, arg0: W, ...) -> isize;
    fn ref__execl(name: A, arg0: A, ...) -> isize;
    fn ref__wexecle(name: W, arg0: W, ...) -> isize;
    fn ref__execle(name: A, arg0: A, ...) -> isize;
    fn ref__wexeclp(name: W, arg0: W, ...) -> isize;
    fn ref__execlp(name: A, arg0: A, ...) -> isize;
    fn ref__wexeclpe(name: W, arg0: W, ...) -> isize;
    fn ref__execlpe(name: A, arg0: A, ...) -> isize;
    fn ref__wspawnl(flags: c_int, name: W, arg0: W, ...) -> isize;
    fn ref__spawnl(flags: c_int, name: A, arg0: A, ...) -> isize;
    fn ref__wspawnle(flags: c_int, name: W, arg0: W, ...) -> isize;
    fn ref__spawnle(flags: c_int, name: A, arg0: A, ...) -> isize;
    fn ref__wspawnlp(flags: c_int, name: W, arg0: W, ...) -> isize;
    fn ref__spawnlp(flags: c_int, name: A, arg0: A, ...) -> isize;
    fn ref__wspawnlpe(flags: c_int, name: W, arg0: W, ...) -> isize;
    fn ref__spawnlpe(flags: c_int, name: A, arg0: A, ...) -> isize;

    fn ref__wpopen(command: W, mode: W) -> *mut c_void;
    fn ref__popen(command: A, mode: A) -> *mut c_void;
    fn ref__pclose(file: *mut c_void) -> c_int;
    fn ref_msvcrt_free_popen_data();

    fn ref__loaddll(dllname: A) -> isize;
    fn ref__unloaddll(dll: isize) -> c_int;
    fn ref__getdllprocaddr(dll: isize, name: A, ordinal: c_int) -> *mut c_void;
    fn ref__getpid() -> c_int;

    // file.c's — the reference's own fd table. See `label_handle`.
    fn ref__get_osfhandle(fd: c_int) -> isize;
}

// ══════════════════════════════════════════════════════════════════════════════════════
// Constants — derived from the same header the DLL compiles against
// ══════════════════════════════════════════════════════════════════════════════════════
//
// Spelled here rather than read from msvcrt_crt_consts.h because Rust cannot include a C
// header, and pinned against it by `pin_constants()` below rather than trusted. The
// `_P_*` order is the one that would be easy to guess wrong: `_P_OVERLAY` is 2, in the
// MIDDLE of the valid range, not at an end.
const P_WAIT: c_int = 0;
const P_NOWAIT: c_int = 1;
const P_OVERLAY: c_int = 2;
const P_NOWAITO: c_int = 3;
const P_DETACH: c_int = 4;

const CREATE_UNICODE_ENVIRONMENT: u32 = 0x0000_0400;
const DETACHED_PROCESS: u32 = 8;
const STARTF_USESTDHANDLES: u32 = 0x0000_0100;

const WAIT_OBJECT_0: u32 = 0;
const WAIT_FAILED: u32 = 0xFFFF_FFFF;

const ERROR_FILE_NOT_FOUND: u32 = 2;
const ERROR_ACCESS_DENIED: u32 = 5;
const ERROR_INVALID_HANDLE: u32 = 6;

// msvcrt's own <errno.h>, and the names carry a `CRT_` prefix on purpose: this file also
// talks in Win32 `ERROR_*` codes, and freeldr-rust numbers its OWN `EACCES`/`ENOENT`
// differently (2 and 14) — `scan_const_contradictions` flags the bare spellings for
// exactly that reason. Two error domains in one file is the situation the prefix exists
// for, not a naming preference.
const CRT_ENOENT: c_int = 2;
const CRT_EACCES: c_int = 13;
const CRT_ECHILD: c_int = 10;
const CRT_EINVAL: c_int = 22;

/// errno is seeded with THIS, not 0, so a call that CLEARS errno is observable — the same
/// reason tests/file.rs seeds it. `msvcrt_spawn`'s flags check writes EINVAL and returns
/// without touching anything else, and a zeroed start would make "wrote nothing" and
/// "wrote 0" the same observation.
const ERRNO_SEED: c_int = 0x5A5A;

/// The exit code processimpl.c's GetExitCodeProcess reports. Deliberately not 0 and not
/// the pid: `msvcrt_spawn`'s `_P_WAIT` arm writes the EXIT CODE into `pi.dwProcessId` and
/// returns THAT, so a row can only tell the two apart if they differ.
const EXIT_CODE: u32 = 0x2A;
const FAKE_PID: u32 = 4321;
const FAKE_TID: u32 = 8765;

// ══════════════════════════════════════════════════════════════════════════════════════
// Row plumbing
// ══════════════════════════════════════════════════════════════════════════════════════

static mut ROWS: u64 = 0;

fn row() {
    unsafe { ROWS += 1 };
}

fn rows() -> u64 {
    unsafe { *core::ptr::addr_of!(ROWS) }
}

fn both<T: PartialEq + core::fmt::Debug>(what: &str, cfg: &str, subj: T, refr: T) {
    row();
    assert_eq!(subj, refr, "{what} [{cfg}]: subject != reference");
}

// ── string plumbing ───────────────────────────────────────────────────────────────────

/// A NUL-terminated UTF-16 buffer that stays alive as long as the `Vec` does.
struct Wide(Vec<u16>);

impl Wide {
    fn new(s: &str) -> Wide {
        let mut v: Vec<u16> = s.encode_utf16().collect();
        v.push(0);
        Wide(v)
    }
    fn p(&self) -> W {
        self.0.as_ptr()
    }
}

struct Narrow(Vec<u8>);

impl Narrow {
    fn new(s: &str) -> Narrow {
        let mut v = s.as_bytes().to_vec();
        v.push(0);
        Narrow(v)
    }
    fn p(&self) -> A {
        self.0.as_ptr() as A
    }
}

unsafe fn read_wide(p: *const u16) -> String {
    let mut n = 0usize;
    while *p.add(n) != 0 {
        n += 1;
    }
    String::from_utf16_lossy(core::slice::from_raw_parts(p, n))
}

unsafe fn read_narrow(p: *const c_char) -> String {
    let mut n = 0usize;
    while *p.add(n) != 0 {
        n += 1;
    }
    String::from_utf8_lossy(core::slice::from_raw_parts(p as *const u8, n)).into_owned()
}

/// The environment BLOCK, rendered so a diff reads. Embedded NULs become `|`, which is
/// the whole point: `msvcrt_argvtos(envp, 0)`'s terminator rule ("`delim` ? overwrite the
/// last separator : append a NUL") is one of the two branches in the function, and it is
/// only visible if the NULs are.
unsafe fn read_block(p: *const u16, len: c_uint) -> String {
    let s = core::slice::from_raw_parts(p, len as usize);
    s.iter()
        .map(|&c| if c == 0 { '|' } else { char::from_u32(c as u32).unwrap_or('?') })
        .collect()
}

/// The flat `va_list` the subject's `_v*` twins walk: one 8-byte slot per pointer, which
/// IS the Windows-x64 model (`printf::va_next`). The reference gets the same arguments as
/// real host varargs; see the module docs.
struct VaBuf(Vec<u64>);

impl VaBuf {
    fn of(ptrs: &[*const c_void]) -> VaBuf {
        VaBuf(ptrs.iter().map(|p| *p as u64).collect())
    }
    fn p(&mut self) -> *mut u8 {
        self.0.as_mut_ptr() as *mut u8
    }
}

// ══════════════════════════════════════════════════════════════════════════════════════
// The observation
// ══════════════════════════════════════════════════════════════════════════════════════

/// One side's complete footprint for a spawn row: what came back, what it did to errno,
/// and every argument it handed CreateProcessW.
#[derive(PartialEq, Debug)]
struct Spawn {
    ret: i64,
    errno: c_int,
    pmt: c_int,
    calls: c_int,
    app: Option<String>,
    cmdline: Option<String>,
    env: Option<String>,
    cwd_null: bool,
    flags: u32,
    inherit: bool,
    attrs_null: bool,
    si_cb: u32,
    si_flags: u32,
    si_cbreserved2: c_uint,
    si_reserved2_null: bool,
    si_inherit: Vec<u8>,
    /// The three std handles, as ROLES rather than numbers — see `label_handle`.
    si_std: (String, String, String),
}

/// `_wpopen` is the only caller that sets `si.hStdInput`/`hStdOutput`/`hStdError`, and what
/// it is deciding is WHICH END OF THE PIPE the child gets: `read_pipe` puts the write end
/// on stdout and leaves stdin alone, `!read_pipe` does the opposite. That decision is the
/// entire difference between `_popen(cmd, "r")` and `_popen(cmd, "w")`.
///
/// The raw handle NUMBERS cannot be compared, and it took a row to see why. Each side has
/// its own CRT fd table — file.c's for the reference, file.rs's for the subject — and each
/// table caches OS handles that the host allocated at whatever moment that side first
/// opened something. The subject runs first in every row, so its `_pipe` takes the lower
/// host descriptors and the reference's take the next ones: the two sides reported
/// `(4, 14, 8)` and `(5, 14, 9)` for identical, correct behaviour. Comparing the numbers
/// would have been a permanently red row that says nothing; comparing the ROLES says
/// exactly the thing the mode string decides.
unsafe fn label_handle(h: i64, osf: unsafe extern "C" fn(c_int) -> isize) -> String {
    if h == 0 {
        // msvcrt_spawn memsets its STARTUPINFOW, so every non-popen row lands here.
        return "unset".into();
    }
    for (fd, name) in [(0, "STDIN"), (1, "STDOUT"), (2, "STDERR")] {
        if osf(fd) as i64 == h {
            return name.into();
        }
    }
    "PIPE".into()
}

unsafe fn snapshot(
    ret: isize,
    errno: c_int,
    pmt: c_int,
    osf: unsafe extern "C" fn(c_int) -> isize,
) -> Spawn {
    assert_eq!(
        ref_proc_truncated(),
        0,
        "processimpl.c's ledger OVERFLOWED — a clipped command line makes two different \
         marshallings compare equal, which is the one failure this harness must not have. \
         Raise REF_PROC_CMDLINE_MAX/REF_PROC_ENV_MAX."
    );
    Spawn {
        ret: ret as i64,
        errno,
        pmt,
        calls: ref_proc_calls(),
        app: (ref_proc_app_null() == 0).then(|| read_wide(ref_proc_app())),
        cmdline: (ref_proc_cmdline_null() == 0).then(|| read_wide(ref_proc_cmdline())),
        env: (ref_proc_env_null() == 0).then(|| read_block(ref_proc_env(), ref_proc_env_len())),
        cwd_null: ref_proc_cwd_null() != 0,
        flags: ref_proc_flags(),
        inherit: ref_proc_inherit() != 0,
        attrs_null: ref_proc_attrs_null() != 0,
        si_cb: ref_proc_si_cb(),
        si_flags: ref_proc_si_flags(),
        si_cbreserved2: ref_proc_si_cbreserved2(),
        si_reserved2_null: ref_proc_si_reserved2_null() != 0,
        si_inherit: core::slice::from_raw_parts(
            ref_proc_si_inherit(),
            ref_proc_si_inherit_len() as usize,
        )
        .to_vec(),
        si_std: (
            label_handle(ref_proc_si_stdin(), osf),
            label_handle(ref_proc_si_stdout(), osf),
            label_handle(ref_proc_si_stderr(), osf),
        ),
    }
}

/// The row. Reset → subject → snapshot → reset → reference → snapshot → compare.
///
/// The reset between the two calls is what makes ONE ledger sound (see the module docs):
/// the recorder is a sink nothing in process.c reads, and each side's marshalling is read
/// back before the other side can write over it.
unsafe fn duel(cfg: &str, subj: impl FnOnce() -> isize, refr: impl FnOnce() -> isize) {
    ref_proc_reset();
    subj_errno_set(ERRNO_SEED);
    subj_invalid_pmt_reset();
    let s_ret = subj();
    let s = snapshot(s_ret, subj_errno_get(), subj_invalid_pmt_count(), _get_osfhandle);

    ref_proc_reset();
    ref_errno_set(ERRNO_SEED);
    ref_invalid_pmt_reset();
    let r_ret = refr();
    let r = snapshot(r_ret, ref_errno_get(), ref_invalid_pmt_count(), ref__get_osfhandle);

    both("spawn", cfg, s, r);
}

// ══════════════════════════════════════════════════════════════════════════════════════
// The environment: a real directory tree, because GetFileAttributesW is real
// ══════════════════════════════════════════════════════════════════════════════════════
//
// fileimpl.c backs GetFileAttributesW with `stat`, so `msvcrt_search_executable` walks an
// actual filesystem — which is what makes its four-extension probe, its PATH walk and its
// length arithmetic gradeable at all. A shim that answered "exists" or "does not exist"
// unconditionally would collapse the function to one branch, on both sides, green.
//
// The layout is chosen so each arm of the search is reachable AND distinguishable:
//
//   here.exe        an exact hit in the current directory, no probing needed
//   order.com       + order.exe: the suffix table's ORDER is `.com .exe .bat .cmd`, so a
//   order.exe       search for `order` must land on the `.com`. A probe that tried them
//                   in any other order still finds *a* file and still looks green.
//   only.cmd        the LAST suffix, so a probe that stops early misses it
//   noext           a file with no extension at all: found as an exact hit, never probed
//   dotted.name     contains a `.` that is not an extension, which makes the backward
//                   scan set `extension = 1` and SKIP the probe entirely
//   pathonly.exe    lives in the second PATH directory only — unreachable with use_path 0
static TREE: &[&str] =
    &["here.exe", "order.com", "order.exe", "only.cmd", "noext", "dotted.name"];
static PATH_TREE: &[&str] = &["pathonly.exe", "pathonly.bat", "shadow.exe"];

/// The names each search row tries, paired with a label. The interesting ones are the
/// misses and the near-misses, not the hits.
static SEARCH_NAMES: &[&str] = &[
    "here.exe",     // exact hit, current dir
    "here",         // probes; .exe is the second suffix
    "order",        // probes; .com must win over .exe
    "only",         // probes; .cmd is the LAST suffix
    "noext",        // exact hit with no extension
    "dotted.name",  // exact hit, and the `.` suppresses probing
    "dotted",       // MISS: probing `dotted.com` &c., none exist
    "pathonly",     // only findable with use_path
    "shadow",       // in PATH, and `shadow.exe` shadows nothing locally
    "nosuchthing",  // total miss — fullname stays empty
    "with space",   // a name containing a space, which the command line will re-split
    "sub/here.exe", // a separator before any dot: the backward scan stops at `/`
    "",             // the empty name: `name_len` 0, and the probe writes at offset 0
    ".",            // a bare dot
    "..",           // the parent directory, which EXISTS and is a DIRECTORY
];

/// argv shapes. `msvcrt_argvtos` has exactly two branches after the loop — `delim && p >
/// ret` overwrites the final separator, otherwise it appends a NUL — and they are only
/// distinguishable across an empty list and a non-empty one.
fn argv_shapes() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        ("one", vec!["prog"]),
        ("two", vec!["prog", "arg"]),
        ("many", vec!["prog", "a", "b", "c", "d", "e"]),
        ("empty-arg", vec!["prog", "", "after"]),
        ("all-empty", vec!["", "", ""]),
        ("spaces", vec!["prog", "a b", " lead", "trail "]),
        ("quotes", vec!["prog", "\"quoted\"", "back\\slash"]),
        ("long", vec!["prog", "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"]),
        ("none", vec![]),
    ]
}

/// envp shapes. The block's terminator rule is the branch, and the empty list is the case
/// that separates "one NUL" from "two".
fn envp_shapes() -> Vec<(&'static str, Option<Vec<&'static str>>)> {
    vec![
        ("null", None),
        ("empty", Some(vec![])),
        ("one", Some(vec!["A=1"])),
        ("three", Some(vec!["A=1", "BB=22", "CCC=333"])),
        ("empty-entry", Some(vec!["A=1", "", "C=3"])),
        ("no-equals", Some(vec!["JUSTNAME"])),
    ]
}

// ── array plumbing: a NULL-terminated vector of pointers, kept alive by its owner ──

struct WArgv {
    _items: Vec<Wide>,
    ptrs: Vec<*const u16>,
}

impl WArgv {
    fn new(items: &[&str]) -> WArgv {
        let owned: Vec<Wide> = items.iter().map(|s| Wide::new(s)).collect();
        let mut ptrs: Vec<*const u16> = owned.iter().map(|w| w.p()).collect();
        ptrs.push(core::ptr::null());
        WArgv { _items: owned, ptrs }
    }
    fn p(&self) -> *const W {
        self.ptrs.as_ptr()
    }
}

struct AArgv {
    _items: Vec<Narrow>,
    ptrs: Vec<*const c_char>,
}

impl AArgv {
    fn new(items: &[&str]) -> AArgv {
        let owned: Vec<Narrow> = items.iter().map(|s| Narrow::new(s)).collect();
        let mut ptrs: Vec<*const c_char> = owned.iter().map(|n| n.p()).collect();
        ptrs.push(core::ptr::null());
        AArgv { _items: owned, ptrs }
    }
    fn p(&self) -> *const A {
        self.ptrs.as_ptr()
    }
}

// ══════════════════════════════════════════════════════════════════════════════════════
// The gates: what must be true before any of the 40k rows mean anything
// ══════════════════════════════════════════════════════════════════════════════════════

/// The constants above are Rust copies of C values. Pin them against what the code
/// actually does, so a drifted copy fails here rather than making every row compare two
/// sides that both took the wrong branch.
///
/// `_P_DETACH` is the pin that carries the most: it is the upper bound of msvcrt_spawn's
/// range check AND the arm that adds DETACHED_PROCESS, so if this number is wrong the
/// "invalid flags" rows below stop testing the boundary and quietly test the interior.
unsafe fn pin_constants() {
    let prog = Wide::new("here.exe");
    let argv = WArgv::new(&["p"]);

    // The valid range is [0, _P_DETACH]. One past it must be rejected with EINVAL and no
    // CreateProcessW call at all; _P_DETACH itself must be accepted.
    ref_proc_set_create_result(1, 0, FAKE_PID, FAKE_TID);
    ref_proc_reset();
    ref_errno_set(ERRNO_SEED);
    let bad = ref__wspawnv(P_DETACH + 1, prog.p(), argv.p());
    assert_eq!(bad, -1, "flags past _P_DETACH must be rejected");
    assert_eq!(ref_errno_get(), CRT_EINVAL, "…with EINVAL");
    assert_eq!(ref_proc_calls(), 0, "…and without reaching CreateProcessW");

    ref_proc_reset();
    let ok = ref__wspawnv(P_DETACH, prog.p(), argv.p());
    assert_eq!(ref_proc_calls(), 1, "_P_DETACH must be INSIDE the accepted range");
    assert_eq!(
        ref_proc_flags(),
        CREATE_UNICODE_ENVIRONMENT | DETACHED_PROCESS,
        "the _P_DETACH arm must OR in DETACHED_PROCESS — if this constant is wrong, every \
         flags comparison below is two sides agreeing on our number"
    );
    assert_eq!(ok, 0, "_P_DETACH closes hProcess and returns the zeroed handle");

    ref_proc_reset();
    ref__wspawnv(P_WAIT, prog.p(), argv.p());
    assert_eq!(
        ref_proc_flags(),
        CREATE_UNICODE_ENVIRONMENT,
        "every spawn ORs in CREATE_UNICODE_ENVIRONMENT"
    );
    assert!(ref_proc_inherit() != 0, "msvcrt always passes bInheritHandles TRUE");
    assert!(ref_proc_attrs_null() != 0, "…and NULL for both SECURITY_ATTRIBUTES");
    assert!(ref_proc_cwd_null() != 0, "…and NULL for the working directory");
}

/// The rows above compare two sides. These assert what the ANSWER is — because two sides
/// that both return -1, both record an empty application name and both write the same
/// errno agree perfectly while proving nothing. Each pin below is a branch that a plausible
/// port could lose entirely without a single row going red.
unsafe fn pin_effects() {
    let prog = Wide::new("here.exe");
    let argv = WArgv::new(&["p"]);

    // `_P_WAIT` returns the EXIT CODE, not the pid — msvcrt writes GetExitCodeProcess's
    // output into `pi.dwProcessId` and returns that field. The two are different numbers
    // here precisely so this pin can tell them apart.
    ref_proc_set_create_result(1, 0, FAKE_PID, FAKE_TID);
    ref_proc_set_exit_code(1, EXIT_CODE);
    ref_proc_reset();
    let r = ref__wspawnv(P_WAIT, prog.p(), argv.p());
    assert_eq!(
        r as u32, EXIT_CODE,
        "_P_WAIT must return the child's EXIT CODE. It came back {r}, and FAKE_PID is {FAKE_PID} \
         — if those are the same number this pin is not testing anything."
    );
    // …and `_P_NOWAIT` returns the process HANDLE instead.
    ref_proc_reset();
    let r = ref__wspawnv(P_NOWAIT, prog.p(), argv.p());
    assert_eq!(
        r as i64,
        ref_proc_hprocess(),
        "_P_NOWAIT must return the process handle, not the exit code"
    );

    // The errno mapping. `msvcrt_spawn` funnels a failed CreateProcessW through
    // `msvcrt_set_errno(GetLastError())`, and errno.c is reference TU #7 — so this is a
    // genuine Win32-to-CRT mapping and not the harness agreeing with itself.
    ref_proc_set_create_result(0, ERROR_FILE_NOT_FOUND, FAKE_PID, FAKE_TID);
    ref_errno_set(ERRNO_SEED);
    ref__wspawnv(P_WAIT, prog.p(), argv.p());
    assert_eq!(ref_errno_get(), CRT_ENOENT, "ERROR_FILE_NOT_FOUND must map to ENOENT");
    ref_proc_set_create_result(0, ERROR_ACCESS_DENIED, FAKE_PID, FAKE_TID);
    ref_errno_set(ERRNO_SEED);
    ref__wspawnv(P_WAIT, prog.p(), argv.p());
    assert_eq!(ref_errno_get(), CRT_EACCES, "ERROR_ACCESS_DENIED must map to EACCES");

    // `_cwait`'s ECHILD arm, which fires ONLY for ERROR_INVALID_HANDLE — every other
    // failure goes through the ordinary mapper. A port that dropped the special case
    // would still return -1 and still set *an* errno.
    ref_proc_set_wait_result(WAIT_FAILED, ERROR_INVALID_HANDLE);
    ref_errno_set(ERRNO_SEED);
    ref__cwait(core::ptr::null_mut(), 0x7000_0001, 0);
    assert_eq!(ref_errno_get(), CRT_ECHILD, "_cwait's ERROR_INVALID_HANDLE arm must set ECHILD");
    ref_proc_set_wait_result(WAIT_OBJECT_0, ERROR_INVALID_HANDLE);

    // The argv marshaller actually joins with spaces and the env marshaller actually
    // produces a double-NUL block. Both are one `if` away from producing something that
    // still looks like a string.
    ref_proc_set_create_result(1, 0, FAKE_PID, FAKE_TID);
    let av = WArgv::new(&["prog", "one", "two"]);
    let ev = WArgv::new(&["A=1", "BB=22"]);
    ref_proc_reset();
    ref__wspawnve(P_WAIT, prog.p(), av.p(), ev.p());
    assert_eq!(
        read_wide(ref_proc_cmdline()),
        "prog one two",
        "msvcrt_argvtos must join argv with single spaces and drop the trailing one"
    );
    assert_eq!(
        read_block(ref_proc_env(), ref_proc_env_len()),
        "A=1|BB=22||",
        "msvcrt_argvtos(envp, 0) must produce a NUL-separated block ended by an empty string"
    );

    // The CRT's fd-inherit block reaches the child. `msvcrt_create_io_inherit_block` is
    // file.c's, and process.c only carries it — but carrying it is the part that can be
    // dropped, and a spawn that passed NULL would still create the process.
    assert!(
        ref_proc_si_cbreserved2() > 0 && ref_proc_si_reserved2_null() == 0,
        "the spawn must carry a non-empty fd inherit block (cbReserved2={}, ptr null={})",
        ref_proc_si_cbreserved2(),
        ref_proc_si_reserved2_null()
    );

    // `_getdllprocaddr`'s success path really returns what GetProcAddress gave it.
    ref_proc_set_module_results(1, 1, 1);
    let nm = Narrow::new("Anything");
    let got = ref__getdllprocaddr(ref_proc_hmodule() as isize, nm.p(), -1);
    assert_eq!(got as i64, ref_proc_procaddr(), "_getdllprocaddr must forward the address");
    // …and that an ordinal with a non-zero HIWORD is rejected WITHOUT a call.
    ref_proc_reset();
    let got = ref__getdllprocaddr(ref_proc_hmodule() as isize, core::ptr::null(), 0x1_0000);
    assert!(got.is_null() && ref_proc_getprocaddr_calls() == 0,
            "an ordinal above 0xFFFF must be rejected before GetProcAddress is reached");

    ref_proc_set_create_result(1, ERROR_FILE_NOT_FOUND, FAKE_PID, FAKE_TID);
}

/// `_popen`'s mode string decides which end of the pipe the child gets, and that is the
/// only thing separating `"r"` from `"w"`. Pin both directions: a port that ignored the
/// mode would hand the child the same handles either way, and every row would still match.
unsafe fn pin_popen_direction() {
    let cmd = Wide::new("x");
    ref_proc_set_create_result(1, 0, FAKE_PID, FAKE_TID);
    for (mode, want_in, want_out) in [("r", "STDIN", "PIPE"), ("w", "PIPE", "STDOUT")] {
        let m = Wide::new(mode);
        ref_proc_reset();
        let f = ref__wpopen(cmd.p(), m.p());
        assert!(!f.is_null(), "_wpopen({mode:?}) failed — the pipe rows below grade nothing");
        let got = (
            label_handle(ref_proc_si_stdin(), ref__get_osfhandle),
            label_handle(ref_proc_si_stdout(), ref__get_osfhandle),
        );
        assert_eq!(
            got,
            (want_in.to_string(), want_out.to_string()),
            "_wpopen({mode:?}) put the pipe on the wrong end"
        );
        assert_eq!(
            ref_proc_si_flags() & STARTF_USESTDHANDLES,
            STARTF_USESTDHANDLES,
            "_wpopen must set STARTF_USESTDHANDLES, or the child ignores all three handles"
        );
        ref__pclose(f);
    }
}

/// The search tree has to actually be findable, or every search row compares two empty
/// strings and reports MATCH. This is the mbcs_fixture lesson: assert the environment is
/// real before believing anything graded against it.
unsafe fn pin_environment() {
    let name = Wide::new("order");
    let argv = WArgv::new(&["p"]);
    ref_proc_set_create_result(1, 0, FAKE_PID, FAKE_TID);
    ref_proc_reset();
    ref__wspawnv(P_WAIT, name.p(), argv.p());
    assert_eq!(ref_proc_calls(), 1);
    let app = read_wide(ref_proc_app());
    assert_eq!(
        app, "order.com",
        "the executable search found {app:?} for `order` — it must find `order.com`, the \
         FIRST suffix in msvcrt's table. An empty string here means the temp tree is not \
         where the process thinks it is, and every search row below is comparing two \
         misses."
    );

    let name = Wide::new("pathonly");
    ref_proc_reset();
    ref__wspawnvp(P_WAIT, name.p(), argv.p());
    let app = read_wide(ref_proc_app());
    assert!(
        app.ends_with("pathonly.com") || app.ends_with("pathonly.exe"),
        "the PATH walk found {app:?} — `PATH` is not set the way this suite thinks, so the \
         whole use_path half of msvcrt_search_executable is unreachable and green"
    );

    ref_proc_reset();
    ref__wspawnv(P_WAIT, name.p(), argv.p());
    assert_eq!(
        read_wide(ref_proc_app()),
        "",
        "use_path=0 must NOT consult PATH — if it does, the `_spawnv` vs `_spawnvp` \
         distinction is not being tested at all"
    );
}

// ══════════════════════════════════════════════════════════════════════════════════════
// The rows
// ══════════════════════════════════════════════════════════════════════════════════════

/// The `v`-suffixed families: an argv ARRAY, every spawn mode, both widths, with and
/// without PATH. This is where `msvcrt_search_executable` and `msvcrt_argvtos` are graded.
unsafe fn grade_argv_families() {
    let modes: &[(&str, c_int)] = &[
        ("P_WAIT", P_WAIT),
        ("P_NOWAIT", P_NOWAIT),
        ("P_NOWAITO", P_NOWAITO),
        ("P_DETACH", P_DETACH),
        ("P_OVERLAY", P_OVERLAY),
        ("bad-5", P_DETACH + 1),
        ("bad-neg", -1),
        ("bad-max", c_int::MAX),
    ];

    for name in SEARCH_NAMES {
        let w_name = Wide::new(name);
        let a_name = Narrow::new(name);
        for (shape, items) in argv_shapes() {
            let w_argv = WArgv::new(&items);
            let a_argv = AArgv::new(&items);
            for &(mode_name, mode) in modes {
                // _P_OVERLAY must never SUCCEED: the arm is `_exit(0)`. See the module
                // docs — the marshalling is still fully recorded on the failure path.
                let succeed = mode != P_OVERLAY;
                ref_proc_set_create_result(
                    succeed as c_int,
                    ERROR_FILE_NOT_FOUND,
                    FAKE_PID,
                    FAKE_TID,
                );
                let cfg = format!("_wspawnv({mode_name}, {name:?}, {shape})");
                duel(
                    &cfg,
                    || _wspawnv(mode, w_name.p(), w_argv.p()),
                    || ref__wspawnv(mode, w_name.p(), w_argv.p()),
                );
                let cfg = format!("_spawnv({mode_name}, {name:?}, {shape})");
                duel(
                    &cfg,
                    || _spawnv(mode, a_name.p(), a_argv.p()),
                    || ref__spawnv(mode, a_name.p(), a_argv.p()),
                );
                let cfg = format!("_wspawnvp({mode_name}, {name:?}, {shape})");
                duel(
                    &cfg,
                    || _wspawnvp(mode, w_name.p(), w_argv.p()),
                    || ref__wspawnvp(mode, w_name.p(), w_argv.p()),
                );
                let cfg = format!("_spawnvp({mode_name}, {name:?}, {shape})");
                duel(
                    &cfg,
                    || _spawnvp(mode, a_name.p(), a_argv.p()),
                    || ref__spawnvp(mode, a_name.p(), a_argv.p()),
                );
            }

            // The exec family is _P_OVERLAY by construction, so it is always the failing
            // spawn — and the errno mapping is what comes back.
            ref_proc_set_create_result(0, ERROR_FILE_NOT_FOUND, FAKE_PID, FAKE_TID);
            let cfg = format!("_wexecv({name:?}, {shape})");
            duel(
                &cfg,
                || _wexecv(w_name.p(), w_argv.p()),
                || ref__wexecv(w_name.p(), w_argv.p()),
            );
            let cfg = format!("_execv({name:?}, {shape})");
            duel(&cfg, || _execv(a_name.p(), a_argv.p()), || ref__execv(a_name.p(), a_argv.p()));
            let cfg = format!("_wexecvp({name:?}, {shape})");
            duel(
                &cfg,
                || _wexecvp(w_name.p(), w_argv.p()),
                || ref__wexecvp(w_name.p(), w_argv.p()),
            );
            let cfg = format!("_execvp({name:?}, {shape})");
            duel(&cfg, || _execvp(a_name.p(), a_argv.p()), || ref__execvp(a_name.p(), a_argv.p()));
        }
    }
}

/// The `ve`/`vpe` families: an argv array AND an envp array. This is where
/// `msvcrt_argvtos(envp, 0)` — the double-NUL block — is graded, and the `_aw` twin's
/// `CP_ACP` choice with it.
unsafe fn grade_envp_families() {
    ref_proc_set_create_result(1, ERROR_FILE_NOT_FOUND, FAKE_PID, FAKE_TID);
    for name in ["here.exe", "order", "nosuchthing", "pathonly"] {
        let w_name = Wide::new(name);
        let a_name = Narrow::new(name);
        for (ashape, aitems) in argv_shapes() {
            let w_argv = WArgv::new(&aitems);
            let a_argv = AArgv::new(&aitems);
            for (eshape, eitems) in envp_shapes() {
                let (w_env, a_env) = match &eitems {
                    None => (None, None),
                    Some(v) => (Some(WArgv::new(v)), Some(AArgv::new(v))),
                };
                let wp = w_env.as_ref().map_or(core::ptr::null(), |e| e.p());
                let ap = a_env.as_ref().map_or(core::ptr::null(), |e| e.p());

                for &(mode_name, mode) in
                    &[("P_WAIT", P_WAIT), ("P_NOWAIT", P_NOWAIT), ("P_DETACH", P_DETACH)]
                {
                    let cfg =
                        format!("_wspawnve({mode_name}, {name:?}, {ashape}, env={eshape})");
                    duel(
                        &cfg,
                        || _wspawnve(mode, w_name.p(), w_argv.p(), wp),
                        || ref__wspawnve(mode, w_name.p(), w_argv.p(), wp),
                    );
                    let cfg =
                        format!("_spawnve({mode_name}, {name:?}, {ashape}, env={eshape})");
                    duel(
                        &cfg,
                        || _spawnve(mode, a_name.p(), a_argv.p(), ap),
                        || ref__spawnve(mode, a_name.p(), a_argv.p(), ap),
                    );
                    let cfg =
                        format!("_wspawnvpe({mode_name}, {name:?}, {ashape}, env={eshape})");
                    duel(
                        &cfg,
                        || _wspawnvpe(mode, w_name.p(), w_argv.p(), wp),
                        || ref__wspawnvpe(mode, w_name.p(), w_argv.p(), wp),
                    );
                    let cfg =
                        format!("_spawnvpe({mode_name}, {name:?}, {ashape}, env={eshape})");
                    duel(
                        &cfg,
                        || _spawnvpe(mode, a_name.p(), a_argv.p(), ap),
                        || ref__spawnvpe(mode, a_name.p(), a_argv.p(), ap),
                    );
                }

                ref_proc_set_create_result(0, ERROR_FILE_NOT_FOUND, FAKE_PID, FAKE_TID);
                let cfg = format!("_wexecve({name:?}, {ashape}, env={eshape})");
                duel(
                    &cfg,
                    || _wexecve(w_name.p(), w_argv.p(), wp),
                    || ref__wexecve(w_name.p(), w_argv.p(), wp),
                );
                let cfg = format!("_execve({name:?}, {ashape}, env={eshape})");
                duel(
                    &cfg,
                    || _execve(a_name.p(), a_argv.p(), ap),
                    || ref__execve(a_name.p(), a_argv.p(), ap),
                );
                let cfg = format!("_wexecvpe({name:?}, {ashape}, env={eshape})");
                duel(
                    &cfg,
                    || _wexecvpe(w_name.p(), w_argv.p(), wp),
                    || ref__wexecvpe(w_name.p(), w_argv.p(), wp),
                );
                let cfg = format!("_execvpe({name:?}, {ashape}, env={eshape})");
                duel(
                    &cfg,
                    || _execvpe(a_name.p(), a_argv.p(), ap),
                    || ref__execvpe(a_name.p(), a_argv.p(), ap),
                );
                ref_proc_set_create_result(1, ERROR_FILE_NOT_FOUND, FAKE_PID, FAKE_TID);
            }
        }
    }
}

/// The sixteen WINAPIV entry points, graded through the reference's real varargs against
/// the subject's flat cursor. `msvcrt_valisttos` is the function under the microscope
/// here, and its realloc growth loop (`size = max(256, size * 2)`) only fires past 256
/// wide characters — which is why one of the shapes below is deliberately enormous.
unsafe fn grade_varargs_families() {
    let long = "x".repeat(200);
    let shapes: Vec<(&str, Vec<String>)> = vec![
        ("arg0-only", vec!["a0".into()]),
        ("two", vec!["a0".into(), "a1".into()]),
        ("five", vec!["a0".into(), "a1".into(), "a2".into(), "a3".into(), "a4".into()]),
        ("empty-tail", vec!["a0".into(), "".into()]),
        // Past 256 wide chars in total, so the growth loop runs at least twice.
        ("grow", vec![long.clone(), long.clone(), long.clone()]),
    ];

    for name in ["here.exe", "order", "nosuchthing"] {
        let w_name = Wide::new(name);
        let a_name = Narrow::new(name);
        for (shape, items) in &shapes {
            let w: Vec<Wide> = items.iter().map(|s| Wide::new(s)).collect();
            let a: Vec<Narrow> = items.iter().map(|s| Narrow::new(s)).collect();
            // Slot buffer for the subject: everything AFTER arg0, then the NULL sentinel.
            let mut w_tail: Vec<*const c_void> =
                w[1..].iter().map(|x| x.p() as *const c_void).collect();
            w_tail.push(core::ptr::null());
            let mut a_tail: Vec<*const c_void> =
                a[1..].iter().map(|x| x.p() as *const c_void).collect();
            a_tail.push(core::ptr::null());

            // The `le`/`lpe` families take an envp array AFTER the NULL sentinel, and
            // their Rust twins walk that one snapshot TWICE to find it — the reasoning is
            // written out at the twins in process.rs. A row that never supplied a second
            // walk would leave that half unexercised.
            let envp = WArgv::new(&["A=1", "BB=22"]);
            let a_envp = AArgv::new(&["A=1", "BB=22"]);
            let mut w_tail_e = w_tail.clone();
            w_tail_e.push(envp.p() as *const c_void);
            let mut a_tail_e = a_tail.clone();
            a_tail_e.push(a_envp.p() as *const c_void);

            // Every `_spawnl*` shape takes an explicit mode; three cover the arms.
            for &(mode_name, mode) in
                &[("P_WAIT", P_WAIT), ("P_NOWAIT", P_NOWAIT), ("P_OVERLAY", P_OVERLAY)]
            {
                let succeed = mode != P_OVERLAY;
                ref_proc_set_create_result(
                    succeed as c_int,
                    ERROR_FILE_NOT_FOUND,
                    FAKE_PID,
                    FAKE_TID,
                );
                let _ = mode_name;

                // The variadic call must be spelled out per arity: Rust cannot forward a
                // slice into a C `...`. Five is the widest shape above.
                let cfg = format!("_vwspawnl({mode_name}, {name:?}, {shape})");
                let mut buf = VaBuf::of(&w_tail);
                let t: Vec<W> = w[1..].iter().map(|x| x.p()).collect();
                duel(
                    &cfg,
                    || _vwspawnl(mode, w_name.p(), w[0].p(), buf.p()),
                    || call_wspawnl(mode, w_name.p(), w[0].p(), &t),
                );

                let cfg = format!("_vspawnl({mode_name}, {name:?}, {shape})");
                let mut buf = VaBuf::of(&a_tail);
                let t: Vec<A> = a[1..].iter().map(|x| x.p()).collect();
                duel(
                    &cfg,
                    || _vspawnl(mode, a_name.p(), a[0].p(), buf.p()),
                    || call_spawnl(mode, a_name.p(), a[0].p(), &t),
                );

                let cfg = format!("_vwspawnlp({mode_name}, {name:?}, {shape})");
                let mut buf = VaBuf::of(&w_tail);
                let t: Vec<W> = w[1..].iter().map(|x| x.p()).collect();
                duel(
                    &cfg,
                    || _vwspawnlp(mode, w_name.p(), w[0].p(), buf.p()),
                    || call_wspawnlp(mode, w_name.p(), w[0].p(), &t),
                );

                let cfg = format!("_vspawnlp({mode_name}, {name:?}, {shape})");
                let mut buf = VaBuf::of(&a_tail);
                let t: Vec<A> = a[1..].iter().map(|x| x.p()).collect();
                duel(
                    &cfg,
                    || _vspawnlp(mode, a_name.p(), a[0].p(), buf.p()),
                    || call_spawnlp(mode, a_name.p(), a[0].p(), &t),
                );

                let cfg = format!("_vwspawnle({mode_name}, {name:?}, {shape})");
                let mut buf = VaBuf::of(&w_tail_e);
                let t: Vec<W> = w[1..].iter().map(|x| x.p()).collect();
                duel(
                    &cfg,
                    || _vwspawnle(mode, w_name.p(), w[0].p(), buf.p()),
                    || call_wspawnle(mode, w_name.p(), w[0].p(), &t, envp.p()),
                );

                let cfg = format!("_vspawnle({mode_name}, {name:?}, {shape})");
                let mut buf = VaBuf::of(&a_tail_e);
                let t: Vec<A> = a[1..].iter().map(|x| x.p()).collect();
                duel(
                    &cfg,
                    || _vspawnle(mode, a_name.p(), a[0].p(), buf.p()),
                    || call_spawnle(mode, a_name.p(), a[0].p(), &t, a_envp.p()),
                );

                let cfg = format!("_vwspawnlpe({mode_name}, {name:?}, {shape})");
                let mut buf = VaBuf::of(&w_tail_e);
                let t: Vec<W> = w[1..].iter().map(|x| x.p()).collect();
                duel(
                    &cfg,
                    || _vwspawnlpe(mode, w_name.p(), w[0].p(), buf.p()),
                    || call_wspawnlpe(mode, w_name.p(), w[0].p(), &t, envp.p()),
                );

                let cfg = format!("_vspawnlpe({mode_name}, {name:?}, {shape})");
                let mut buf = VaBuf::of(&a_tail_e);
                let t: Vec<A> = a[1..].iter().map(|x| x.p()).collect();
                duel(
                    &cfg,
                    || _vspawnlpe(mode, a_name.p(), a[0].p(), buf.p()),
                    || call_spawnlpe(mode, a_name.p(), a[0].p(), &t, a_envp.p()),
                );
            }

            // The eight `_execl*` shells, always on the failing spawn (_P_OVERLAY).
            ref_proc_set_create_result(0, ERROR_FILE_NOT_FOUND, FAKE_PID, FAKE_TID);

            let cfg = format!("_vwexecl({name:?}, {shape})");
            let mut buf = VaBuf::of(&w_tail);
            let t: Vec<W> = w[1..].iter().map(|x| x.p()).collect();
            duel(
                &cfg,
                || _vwexecl(w_name.p(), w[0].p(), buf.p()),
                || call_wexecl(w_name.p(), w[0].p(), &t),
            );

            let cfg = format!("_vexecl({name:?}, {shape})");
            let mut buf = VaBuf::of(&a_tail);
            let t: Vec<A> = a[1..].iter().map(|x| x.p()).collect();
            duel(
                &cfg,
                || _vexecl(a_name.p(), a[0].p(), buf.p()),
                || call_execl(a_name.p(), a[0].p(), &t),
            );

            let cfg = format!("_vwexeclp({name:?}, {shape})");
            let mut buf = VaBuf::of(&w_tail);
            let t: Vec<W> = w[1..].iter().map(|x| x.p()).collect();
            duel(
                &cfg,
                || _vwexeclp(w_name.p(), w[0].p(), buf.p()),
                || call_wexeclp(w_name.p(), w[0].p(), &t),
            );

            let cfg = format!("_vexeclp({name:?}, {shape})");
            let mut buf = VaBuf::of(&a_tail);
            let t: Vec<A> = a[1..].iter().map(|x| x.p()).collect();
            duel(
                &cfg,
                || _vexeclp(a_name.p(), a[0].p(), buf.p()),
                || call_execlp(a_name.p(), a[0].p(), &t),
            );

            let cfg = format!("_vwexecle({name:?}, {shape})");
            let mut buf = VaBuf::of(&w_tail_e);
            let t: Vec<W> = w[1..].iter().map(|x| x.p()).collect();
            duel(
                &cfg,
                || _vwexecle(w_name.p(), w[0].p(), buf.p()),
                || call_wexecle(w_name.p(), w[0].p(), &t, envp.p()),
            );

            let cfg = format!("_vexecle({name:?}, {shape})");
            let mut buf = VaBuf::of(&a_tail_e);
            let t: Vec<A> = a[1..].iter().map(|x| x.p()).collect();
            duel(
                &cfg,
                || _vexecle(a_name.p(), a[0].p(), buf.p()),
                || call_execle(a_name.p(), a[0].p(), &t, a_envp.p()),
            );

            let cfg = format!("_vwexeclpe({name:?}, {shape})");
            let mut buf = VaBuf::of(&w_tail_e);
            let t: Vec<W> = w[1..].iter().map(|x| x.p()).collect();
            duel(
                &cfg,
                || _vwexeclpe(w_name.p(), w[0].p(), buf.p()),
                || call_wexeclpe(w_name.p(), w[0].p(), &t, envp.p()),
            );

            let cfg = format!("_vexeclpe({name:?}, {shape})");
            let mut buf = VaBuf::of(&a_tail_e);
            let t: Vec<A> = a[1..].iter().map(|x| x.p()).collect();
            duel(
                &cfg,
                || _vexeclpe(a_name.p(), a[0].p(), buf.p()),
                || call_execlpe(a_name.p(), a[0].p(), &t, a_envp.p()),
            );
        }
    }
}

// ── the variadic call sites, one per arity ────────────────────────────────────────────
//
// Rust cannot splat a slice into a C `...`, so each family gets a dispatcher over the
// arities the shapes above actually use (1..=3 tail arguments). The `unreachable!` arm is
// deliberate: a new shape with a wider tail must FAIL here rather than silently pass the
// reference fewer arguments than the subject got, which would look like a port bug.
macro_rules! variadic_dispatch {
    ($fn_name:ident, $target:ident, $ptr:ty, $($pre:ident : $pre_ty:ty),*) => {
        unsafe fn $fn_name($($pre: $pre_ty,)* arg0: $ptr, tail: &[$ptr]) -> isize {
            let n: $ptr = core::ptr::null();
            match tail.len() {
                0 => $target($($pre,)* arg0, n),
                1 => $target($($pre,)* arg0, tail[0], n),
                2 => $target($($pre,)* arg0, tail[0], tail[1], n),
                3 => $target($($pre,)* arg0, tail[0], tail[1], tail[2], n),
                4 => $target($($pre,)* arg0, tail[0], tail[1], tail[2], tail[3], n),
                k => unreachable!("no variadic dispatcher for {k} tail arguments"),
            }
        }
    };
}

macro_rules! variadic_dispatch_e {
    ($fn_name:ident, $target:ident, $ptr:ty, $envp:ty, $($pre:ident : $pre_ty:ty),*) => {
        unsafe fn $fn_name(
            $($pre: $pre_ty,)* arg0: $ptr, tail: &[$ptr], envp: $envp
        ) -> isize {
            let n: $ptr = core::ptr::null();
            match tail.len() {
                0 => $target($($pre,)* arg0, n, envp),
                1 => $target($($pre,)* arg0, tail[0], n, envp),
                2 => $target($($pre,)* arg0, tail[0], tail[1], n, envp),
                3 => $target($($pre,)* arg0, tail[0], tail[1], tail[2], n, envp),
                4 => $target($($pre,)* arg0, tail[0], tail[1], tail[2], tail[3], n, envp),
                k => unreachable!("no variadic dispatcher for {k} tail arguments"),
            }
        }
    };
}

variadic_dispatch!(call_wexecl, ref__wexecl, W, name: W);
variadic_dispatch!(call_execl, ref__execl, A, name: A);
variadic_dispatch!(call_wexeclp, ref__wexeclp, W, name: W);
variadic_dispatch!(call_execlp, ref__execlp, A, name: A);
variadic_dispatch_e!(call_wexecle, ref__wexecle, W, *const W, name: W);
variadic_dispatch_e!(call_execle, ref__execle, A, *const A, name: A);
variadic_dispatch_e!(call_wexeclpe, ref__wexeclpe, W, *const W, name: W);
variadic_dispatch_e!(call_execlpe, ref__execlpe, A, *const A, name: A);
variadic_dispatch!(call_wspawnl, ref__wspawnl, W, flags: c_int, name: W);
variadic_dispatch!(call_spawnl, ref__spawnl, A, flags: c_int, name: A);
variadic_dispatch!(call_wspawnlp, ref__wspawnlp, W, flags: c_int, name: W);
variadic_dispatch!(call_spawnlp, ref__spawnlp, A, flags: c_int, name: A);
variadic_dispatch_e!(call_wspawnle, ref__wspawnle, W, *const W, flags: c_int, name: W);
variadic_dispatch_e!(call_spawnle, ref__spawnle, A, *const A, flags: c_int, name: A);
variadic_dispatch_e!(call_wspawnlpe, ref__wspawnlpe, W, *const W, flags: c_int, name: W);
variadic_dispatch_e!(call_spawnlpe, ref__spawnlpe, A, *const A, flags: c_int, name: A);

/// `_cwait` — the only export that is pure wait-and-map. Its ECHILD arm is reachable ONLY
/// when GetLastError says ERROR_INVALID_HANDLE, and its `status` out-parameter has three
/// distinct behaviours (written on success, written with -1 on failure, or not written at
/// all when NULL) that a row must be able to tell apart.
unsafe fn grade_cwait() {
    let outcomes: &[(&str, u32, u32, bool, u32)] = &[
        // label, wait result, last error, GetExitCodeProcess ok, exit code
        ("signalled", WAIT_OBJECT_0, 0, true, EXIT_CODE),
        ("signalled-zero", WAIT_OBJECT_0, 0, true, 0),
        ("signalled-huge", WAIT_OBJECT_0, 0, true, 0xFFFF_FFFF),
        ("failed-badhandle", WAIT_FAILED, ERROR_INVALID_HANDLE, false, 0),
        ("failed-denied", WAIT_FAILED, ERROR_ACCESS_DENIED, false, 0),
        ("timeout", 0x102, ERROR_ACCESS_DENIED, false, 0),
    ];
    for &(label, wres, werr, ok, code) in outcomes {
        ref_proc_set_wait_result(wres, werr);
        ref_proc_set_exit_code(ok as c_int, code);
        for &(pid_label, pid) in &[("h", 0x7000_0001isize), ("null", 0isize)] {
            // With a status pointer, and without.
            let mut s_status: c_int = -12345;
            let mut r_status: c_int = -12345;
            let cfg = format!("_cwait({label}, pid={pid_label}, status=&x)");
            ref_proc_reset();
            subj_errno_set(ERRNO_SEED);
            let sr = _cwait(&mut s_status, pid, 0);
            let se = subj_errno_get();
            ref_proc_reset();
            ref_errno_set(ERRNO_SEED);
            let rr = ref__cwait(&mut r_status, pid, 0);
            let re = ref_errno_get();
            both("cwait", &cfg, (sr as i64, se, s_status), (rr as i64, re, r_status));

            let cfg = format!("_cwait({label}, pid={pid_label}, status=NULL)");
            ref_proc_reset();
            subj_errno_set(ERRNO_SEED);
            let sr = _cwait(core::ptr::null_mut(), pid, 0);
            let se = subj_errno_get();
            ref_proc_reset();
            ref_errno_set(ERRNO_SEED);
            let rr = ref__cwait(core::ptr::null_mut(), pid, 0);
            let re = ref_errno_get();
            both("cwait-nostatus", &cfg, (sr as i64, se), (rr as i64, re));
        }
    }
    // Restore the defaults the spawn rows assume.
    ref_proc_set_wait_result(WAIT_OBJECT_0, ERROR_INVALID_HANDLE);
    ref_proc_set_exit_code(1, EXIT_CODE);
}

/// The dll trio. `_getdllprocaddr` is the interesting one and its whole body is a
/// two-way decision: a non-NULL `name` is used only when `ordinal == -1`, and a NULL
/// `name` is replaced by the ordinal cast to a pointer — but only if `HIWORD(ordinal)` is
/// zero. Four combinations, and three of them return NULL without calling anything.
unsafe fn grade_dll_family() {
    for &(load_ok, free_ok, proc_ok) in
        &[(1, 1, 1), (0, 1, 1), (1, 0, 1), (1, 1, 0), (0, 0, 0)]
    {
        ref_proc_set_module_results(load_ok, free_ok, proc_ok);
        let cfgtag = format!("load={load_ok} free={free_ok} proc={proc_ok}");

        for dllname in ["kernel32.dll", "", "a/b\\c.dll"] {
            let n = Narrow::new(dllname);
            let cfg = format!("_loaddll({dllname:?}) [{cfgtag}]");
            ref_proc_reset();
            subj_errno_set(ERRNO_SEED);
            let s = _loaddll(n.p());
            let s_obs = (s as i64, subj_errno_get(), ref_proc_loadlibrary_calls(), {
                let nm = read_wide(ref_proc_loadlibrary_name());
                (ref_proc_loadlibrary_null() != 0, nm)
            });
            ref_proc_reset();
            ref_errno_set(ERRNO_SEED);
            let r = ref__loaddll(n.p());
            let r_obs = (r as i64, ref_errno_get(), ref_proc_loadlibrary_calls(), {
                let nm = read_wide(ref_proc_loadlibrary_name());
                (ref_proc_loadlibrary_null() != 0, nm)
            });
            both("loaddll", &cfg, s_obs, r_obs);
        }

        // NULL dllname: msvcrt passes NULL straight through to LoadLibraryW rather than
        // failing early, which is a branch a `if (!name) return 0` port would lose.
        let cfg = format!("_loaddll(NULL) [{cfgtag}]");
        ref_proc_reset();
        subj_errno_set(ERRNO_SEED);
        let s = _loaddll(core::ptr::null());
        let s_obs = (s as i64, subj_errno_get(), ref_proc_loadlibrary_calls(),
                     ref_proc_loadlibrary_null());
        ref_proc_reset();
        ref_errno_set(ERRNO_SEED);
        let r = ref__loaddll(core::ptr::null());
        let r_obs = (r as i64, ref_errno_get(), ref_proc_loadlibrary_calls(),
                     ref_proc_loadlibrary_null());
        both("loaddll-null", &cfg, s_obs, r_obs);

        for handle in [ref_proc_hmodule(), 0, -1] {
            let cfg = format!("_unloaddll({handle:#x}) [{cfgtag}]");
            ref_proc_reset();
            subj_errno_set(ERRNO_SEED);
            let s = _unloaddll(handle as isize);
            let s_obs = (s, subj_errno_get(), ref_proc_freelibrary_calls(),
                         ref_proc_freelibrary_arg());
            ref_proc_reset();
            ref_errno_set(ERRNO_SEED);
            let r = ref__unloaddll(handle as isize);
            let r_obs = (r, ref_errno_get(), ref_proc_freelibrary_calls(),
                         ref_proc_freelibrary_arg());
            both("unloaddll", &cfg, s_obs, r_obs);
        }

        let names: [(&str, Option<Narrow>); 3] = [
            ("named", Some(Narrow::new("GetProcAddress"))),
            ("empty", Some(Narrow::new(""))),
            ("null", None),
        ];
        for (nlabel, nm) in &names {
            let np = nm.as_ref().map_or(core::ptr::null(), |n| n.p());
            for ordinal in [-1i32, 0, 1, 7, 0xFFFF, 0x1_0000, 0x7FFF_FFFF] {
                let cfg =
                    format!("_getdllprocaddr(name={nlabel}, ord={ordinal:#x}) [{cfgtag}]");
                ref_proc_reset();
                let s = _getdllprocaddr(ref_proc_hmodule() as isize, np, ordinal);
                let s_obs = (
                    s as i64,
                    ref_proc_getprocaddr_calls(),
                    ref_proc_getprocaddr_mod(),
                    ref_proc_getprocaddr_raw(),
                    ref_proc_getprocaddr_is_name(),
                    if ref_proc_getprocaddr_is_name() != 0 {
                        read_narrow(ref_proc_getprocaddr_name())
                    } else {
                        String::new()
                    },
                );
                ref_proc_reset();
                let r = ref__getdllprocaddr(ref_proc_hmodule() as isize, np, ordinal);
                let r_obs = (
                    r as i64,
                    ref_proc_getprocaddr_calls(),
                    ref_proc_getprocaddr_mod(),
                    ref_proc_getprocaddr_raw(),
                    ref_proc_getprocaddr_is_name(),
                    if ref_proc_getprocaddr_is_name() != 0 {
                        read_narrow(ref_proc_getprocaddr_name())
                    } else {
                        String::new()
                    },
                );
                both("getdllprocaddr", &cfg, s_obs, r_obs);
            }
        }
    }
    ref_proc_set_module_results(1, 1, 1);

    both("_getpid", "getpid", _getpid(), ref__getpid());
}

/// `_popen` / `_wpopen` / `_pclose`. These stand on the two fd tables and FILE rings —
/// file.c's for the reference, file.rs's for the subject — so the row protocol has to keep
/// them in lockstep: every open is matched by a close on the same side before the next
/// row, or the fd numbers drift apart and the recorded std handles stop comparing.
///
/// The mode string is the branch worth the rows: `w` flips which end of the pipe the
/// child gets, and `b`/`t` fight over the same two flag bits with LAST-ONE-WINS semantics
/// that a port using `|=` alone would get wrong.
unsafe fn grade_popen() {
    let comspec = Wide::new("cmd.exe");
    for (cs_label, cs) in
        [("set", Some(&comspec)), ("unset", None), ("long", Some(&comspec))]
    {
        ref_proc_set_comspec(cs.map_or(core::ptr::null(), |c| c.p()));
        for &(create_ok, err) in &[(1u8, 0u32), (0u8, ERROR_ACCESS_DENIED)] {
            ref_proc_set_create_result(create_ok as c_int, err, FAKE_PID, FAKE_TID);
            for mode in ["r", "w", "rb", "wb", "rt", "wt", "rbt", "rw", "", "X"] {
                let cmd = Wide::new("dir /w");
                let m = Wide::new(mode);
                let cfg = format!("_wpopen(mode={mode:?}, comspec={cs_label}, ok={create_ok})");

                ref_proc_reset();
                subj_errno_set(ERRNO_SEED);
                let sf = _wpopen(cmd.p(), m.p());
                let s = snapshot(sf.is_null() as isize, subj_errno_get(), 0, _get_osfhandle);
                if !sf.is_null() {
                    _pclose(sf);
                }

                ref_proc_reset();
                ref_errno_set(ERRNO_SEED);
                let rf = ref__wpopen(cmd.p(), m.p());
                let r = snapshot(rf.is_null() as isize, ref_errno_get(), 0, ref__get_osfhandle);
                if !rf.is_null() {
                    ref__pclose(rf);
                }
                both("wpopen", &cfg, s, r);

                // The narrow twin, which is `wstrdupa_utf8` + `msvcrt_wstrdupa` over the
                // wide one — two DIFFERENT converters for two arguments of the same call,
                // which is exactly the kind of asymmetry a translation flattens.
                let cmd_a = Narrow::new("dir /w");
                let m_a = Narrow::new(mode);
                let cfg = format!("_popen(mode={mode:?}, comspec={cs_label}, ok={create_ok})");
                ref_proc_reset();
                subj_errno_set(ERRNO_SEED);
                let sf = _popen(cmd_a.p(), m_a.p());
                let s = snapshot(sf.is_null() as isize, subj_errno_get(), 0, _get_osfhandle);
                if !sf.is_null() {
                    _pclose(sf);
                }
                ref_proc_reset();
                ref_errno_set(ERRNO_SEED);
                let rf = ref__popen(cmd_a.p(), m_a.p());
                let r = snapshot(rf.is_null() as isize, ref_errno_get(), 0, ref__get_osfhandle);
                if !rf.is_null() {
                    ref__pclose(rf);
                }
                both("popen", &cfg, s, r);
            }

            // The NULL guards, both arguments, both widths.
            let cmd = Wide::new("x");
            let m = Wide::new("r");
            for (label, c, mm) in [
                ("cmd=NULL", core::ptr::null(), m.p()),
                ("mode=NULL", cmd.p(), core::ptr::null()),
                ("both=NULL", core::ptr::null(), core::ptr::null()),
            ] {
                let cfg = format!("_wpopen({label})");
                ref_proc_reset();
                subj_errno_set(ERRNO_SEED);
                let sf = _wpopen(c, mm);
                let s = (sf.is_null(), subj_errno_get(), ref_proc_calls());
                ref_proc_reset();
                ref_errno_set(ERRNO_SEED);
                let rf = ref__wpopen(c, mm);
                let r = (rf.is_null(), ref_errno_get(), ref_proc_calls());
                both("wpopen-null", &cfg, s, r);
                assert!(sf.is_null() && rf.is_null(), "a NULL argument must not open a pipe");
            }
        }
    }

    // `_pclose` on a FILE that was never opened by popen: the EBADF arm, plus the
    // invalid-parameter handler for NULL.
    ref_proc_set_create_result(1, 0, FAKE_PID, FAKE_TID);
    let cfg = "_pclose(NULL)";
    subj_errno_set(ERRNO_SEED);
    subj_invalid_pmt_reset();
    let s = (_pclose(core::ptr::null_mut()), subj_errno_get(), subj_invalid_pmt_count());
    ref_errno_set(ERRNO_SEED);
    ref_invalid_pmt_reset();
    let r = (ref__pclose(core::ptr::null_mut()), ref_errno_get(), ref_invalid_pmt_count());
    both("pclose-null", cfg, s, r);

    // …and the teardown both sides own.
    msvcrt_free_popen_data();
    ref_msvcrt_free_popen_data();
    row();
}

// ══════════════════════════════════════════════════════════════════════════════════════

fn main() {
    // The temp tree, and the working directory the "try current dir first" arm reads.
    let base = std::env::temp_dir().join(format!("msvcrt-difftest-process-{}", std::process::id()));
    let cwd = base.join("cwd");
    let path1 = base.join("path1");
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&cwd).expect("temp tree");
    std::fs::create_dir_all(&path1).expect("temp tree");
    for f in TREE {
        std::fs::File::create(cwd.join(f)).unwrap().write_all(b"x").unwrap();
    }
    for f in PATH_TREE {
        std::fs::File::create(path1.join(f)).unwrap().write_all(b"x").unwrap();
    }
    std::fs::create_dir_all(cwd.join("sub")).unwrap();
    std::fs::File::create(cwd.join("sub/here.exe")).unwrap().write_all(b"x").unwrap();
    std::env::set_current_dir(&cwd).expect("chdir");
    // ONE PATH for both sides: `_wgetenv` is fileimpl.c's, the environment layer, and a
    // per-side PATH would be two different worlds rather than one comparison.
    std::env::set_var("PATH", path1.to_str().unwrap());

    unsafe {
        ref_proc_set_create_result(1, ERROR_FILE_NOT_FOUND, FAKE_PID, FAKE_TID);
        ref_proc_set_wait_result(WAIT_OBJECT_0, ERROR_INVALID_HANDLE);
        ref_proc_set_exit_code(1, EXIT_CODE);
        ref_proc_set_module_results(1, 1, 1);
        ref_proc_set_comspec(Wide::new("cmd.exe").p());

        pin_constants();
        pin_environment();
        pin_effects();
        pin_popen_direction();

        grade_argv_families();
        grade_envp_families();
        grade_varargs_families();
        grade_cwait();
        grade_dll_family();
        grade_popen();

        println!("process: {} rows", rows());
    }

    let _ = std::env::set_current_dir(std::env::temp_dir());
    let _ = std::fs::remove_dir_all(&base);
}
