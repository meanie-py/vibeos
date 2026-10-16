//! Does the file.c reference actually RUN?
//!
//! Every other harness here grades a Rust shim against a reference. This one
//! grades the *substrate* — it is the acceptance test for the file.c reference
//! TU and the POSIX-backed Win32 layer under it (fileimpl.c), and it exists
//! because "it compiles and links" proves almost nothing about a stream layer.
//!
//! There is no Rust port of file.c yet; this is what has to be true BEFORE one
//! can be graded. Each assertion is chosen so that a plausible substrate bug
//! fails it:
//!
//!   * the FILE layout    — a wrong `_cnt`/`_base` order corrupts buffering, so
//!                          a buffered write/read round-trip catches it;
//!   * the HANDLE mapping — an fd/handle off-by-one makes every read return
//!                          the wrong descriptor's bytes;
//!   * the ERROR model    — the reason a shim may not simply always succeed
//!                          (§9.66): if `open` never failed, `_open`'s ENOENT
//!                          arm would be unreachable and a future difftest row
//!                          over it would report MATCH while proving nothing.
//!                          So the failure cases below are the load-bearing
//!                          ones, not the happy path.
//!
//! Contents are verified through `std::fs` — i.e. by the HOST, not by reading
//! back through the same reference that wrote them. A round-trip that only
//! consults itself would pass just as well with a completely fictional
//! filesystem underneath.

use std::ffi::CString;
use std::sync::{Mutex, MutexGuard, Once};

type FILEP = *mut core::ffi::c_void;

extern "C" {
    /// The real DLL calls this from DllMain; a test process has to call it
    /// itself, or the `__pioinfo` fd table and `_iob[0..2]` are never set up.
    fn ref_msvcrt_init_io();

    fn ref_fopen(path: *const u8, mode: *const u8) -> FILEP;
    fn ref_fclose(f: FILEP) -> i32;
    fn ref_fwrite(buf: *const u8, size: usize, count: usize, f: FILEP) -> usize;
    fn ref_fread(buf: *mut u8, size: usize, count: usize, f: FILEP) -> usize;
    fn ref_fseek(f: FILEP, off: i32, whence: i32) -> i32;
    fn ref_ftell(f: FILEP) -> i32;
    fn ref_fflush(f: FILEP) -> i32;
    fn ref_feof(f: FILEP) -> i32;

    fn ref__open(path: *const u8, flags: i32, ...) -> i32;
    fn ref__close(fd: i32) -> i32;
    fn ref__read(fd: i32, buf: *mut u8, n: u32) -> i32;
    fn ref__write(fd: i32, buf: *const u8, n: u32) -> i32;
    fn ref__lseek(fd: i32, off: i32, whence: i32) -> i32;
    fn ref__filelength(fd: i32) -> i32;
    fn ref_remove(path: *const u8) -> i32;

    /// The reference's OWN errno — refimpl.c's storage, deliberately separate
    /// from the subject's so the two sides can disagree.
    fn ref_errno_get() -> i32;

    /// NOT namespaced, deliberately: kernel32 is shared environment, not code
    /// under test (refimpl.c says so at the last-error slot). Read here only to
    /// widen a failure message — file.c's own errno mapping is what the
    /// assertions grade.
    fn GetLastError() -> u32;
    fn ref_errno_set(v: i32);

    /// The console knob and its reachability counter (fileimpl.c). The knob
    /// takes the HANDLE, not the CRT fd — see `ref_set_console_handle`.
    fn ref__get_osfhandle(crt_fd: i32) -> isize;
    fn ref_set_console_handle(h: isize, on: i32);
    fn ref_console_write_count() -> core::ffi::c_ulong;
    fn ref_console_write_reset();

    /// msvcrt's OWN errno numbering, derived by the cross-compiler and exported
    /// from fileimpl.c. NOT transcribed here: msvcrt and glibc disagree on part
    /// of <errno.h>, so a literal in a test is a second source of truth waiting
    /// to drift from the reference it grades.
    static ref_msvcrt_ENOENT: i32;
    static ref_msvcrt_EBADF: i32;
}

// msvcrt's <fcntl.h>, derived values (refstub/msvcrt_crt_consts.h).
const O_RDONLY: i32 = 0x0000;
const O_WRONLY: i32 = 0x0001;
const O_CREAT: i32 = 0x0100;
const O_TRUNC: i32 = 0x0200;
const O_BINARY: i32 = 0x8000;
const O_TEXT: i32 = 0x4000;


static INIT: Once = Once::new();
static SERIAL: Mutex<()> = Mutex::new(());

/// Initialise the reference CRT once, and hold this guard for the whole test. The CRT's
/// errno is process-global too: run in parallel, another test's failing call (EINVAL)
/// landed between `failures_propagate_all_the_way_to_errno`'s `ref_fopen` and its errno
/// check, and bounced a land preflight (2026-10-15). A test that panics poisons the
/// lock; the next one takes it anyway, so one failure stays one failure.
fn init() -> MutexGuard<'static, ()> {
    let guard = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    INIT.call_once(|| unsafe { ref_msvcrt_init_io() });
    guard
}

/// A path unique to this test AND this process. The CRT state under test is
/// process-global and cargo runs tests on parallel threads, so nothing here may
/// assume a particular fd number or a private working directory.
fn tmp_path(tag: &str) -> (CString, std::path::PathBuf) {
    let p = std::env::temp_dir().join(format!("msvcrt-ref-{}-{}", std::process::id(), tag));
    let _ = std::fs::remove_file(&p);
    (CString::new(p.to_str().unwrap()).unwrap(), p)
}

/// What the reference ACTUALLY created, for a failure that says a file is
/// missing.
///
/// "the file is not there" has two very different causes and the assertion
/// alone cannot tell them apart: the reference never created it, or it created
/// it under a mangled name. The second is invisible from inside the reference —
/// `_open(O_CREAT)` on a garbled path succeeds, and reading back through the
/// same garbled path round-trips perfectly — so only a listing from outside can
/// distinguish them. A 2026-07-29 merge-queue rejection was diagnosable down to
/// "one of those two" and no further, which is what this exists to prevent.
fn siblings_of(path: &std::path::Path) -> String {
    let (Some(dir), Some(stem)) = (path.parent(), path.file_name()) else {
        return "<no parent directory>".into();
    };
    let stem = stem.to_string_lossy();
    // The pid keeps this to THIS process's files however busy /tmp is.
    let prefix: String = stem.chars().take(stem.len().min(24)).collect();
    match std::fs::read_dir(dir) {
        Err(e) => format!("<{dir:?} unreadable: {e}>"),
        Ok(rd) => {
            let mut hits: Vec<String> = rd
                .flatten()
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .filter(|n| n.starts_with(&prefix))
                .collect();
            hits.sort();
            if hits.is_empty() {
                format!("nothing in {dir:?} starts with {prefix:?}")
            } else {
                format!("{dir:?} holds {hits:?}")
            }
        }
    }
}

#[test]
fn buffered_write_round_trips_to_the_real_filesystem() {
    let _serial = init();
    let (c_path, path) = tmp_path("buffered");
    let body = b"the quick brown fox\njumped over 0123456789\n";

    unsafe {
        let f = ref_fopen(c_path.as_ptr() as *const u8, b"wb\0".as_ptr());
        assert!(!f.is_null(), "ref_fopen(\"wb\") returned NULL");

        let n = ref_fwrite(body.as_ptr(), 1, body.len(), f);
        assert_eq!(n, body.len(), "ref_fwrite short write");
        assert_eq!(ref_fflush(f), 0, "ref_fflush failed");
        assert_eq!(ref_fclose(f), 0, "ref_fclose failed");
    }

    // Verified by the HOST. Reading it back through ref_fread would pass even
    // if the whole Win32 layer were a memory buffer pretending to be a disk.
    let on_disk = std::fs::read(&path).expect("the reference did not create the file");
    assert_eq!(on_disk, body, "bytes on disk differ from what the reference wrote");

    // ...and now read it back through the reference too, which is what proves
    // the read path and the buffer bookkeeping (_ptr/_cnt/_base).
    unsafe {
        let f = ref_fopen(c_path.as_ptr() as *const u8, b"rb\0".as_ptr());
        assert!(!f.is_null(), "ref_fopen(\"rb\") returned NULL");

        let mut buf = vec![0u8; body.len() + 8];
        let n = ref_fread(buf.as_mut_ptr(), 1, buf.len(), f);
        assert_eq!(n, body.len(), "ref_fread returned the wrong count");
        assert_eq!(&buf[..n], body, "ref_fread returned the wrong bytes");
        assert_ne!(ref_feof(f), 0, "ref_feof should be set after a short read");
        assert_eq!(ref_fclose(f), 0);
    }

    let _ = std::fs::remove_file(&path);
}

#[test]
fn seek_and_tell_agree_with_the_bytes() {
    let _serial = init();
    let (c_path, path) = tmp_path("seek");
    std::fs::write(&path, b"0123456789abcdef").unwrap();

    unsafe {
        let f = ref_fopen(c_path.as_ptr() as *const u8, b"rb\0".as_ptr());
        assert!(!f.is_null());

        assert_eq!(ref_fseek(f, 10, 0 /* SEEK_SET */), 0);
        assert_eq!(ref_ftell(f), 10, "ftell disagrees with the seek that preceded it");

        let mut b = [0u8; 3];
        assert_eq!(ref_fread(b.as_mut_ptr(), 1, 3, f), 3);
        assert_eq!(&b, b"abc", "seek landed at the wrong offset");
        assert_eq!(ref_ftell(f), 13);

        assert_eq!(ref_fseek(f, -4, 2 /* SEEK_END */), 0);
        assert_eq!(ref_ftell(f), 12);
        assert_eq!(ref_fclose(f), 0);
    }

    let _ = std::fs::remove_file(&path);
}

#[test]
fn the_fd_layer_works_and_reports_length() {
    let _serial = init();
    let (c_path, path) = tmp_path("fd");
    let body = b"low-level bytes";

    unsafe {
        let fd = ref__open(
            c_path.as_ptr() as *const u8,
            O_WRONLY | O_CREAT | O_TRUNC | O_BINARY,
            0o666,
        );
        assert!(fd >= 0, "ref__open for write failed, errno={}", ref_errno_get());
        assert_eq!(ref__write(fd, body.as_ptr(), body.len() as u32), body.len() as i32);
        assert_eq!(ref__filelength(fd), body.len() as i32);
        assert_eq!(ref__close(fd), 0);

        let fd = ref__open(c_path.as_ptr() as *const u8, O_RDONLY | O_BINARY);
        assert!(fd >= 0, "ref__open for read failed, errno={}", ref_errno_get());
        assert_eq!(ref__lseek(fd, 4, 0), 4, "_lseek returned the wrong offset");

        let mut buf = [0u8; 32];
        let n = ref__read(fd, buf.as_mut_ptr(), buf.len() as u32);
        assert_eq!(n, (body.len() - 4) as i32);
        assert_eq!(&buf[..n as usize], &body[4..]);
        assert_eq!(ref__close(fd), 0);
    }

    match std::fs::read(&path) {
        Ok(on_disk) => assert_eq!(on_disk, body),
        Err(e) => panic!(
            "the fd layer reported success but {} is not readable ({e}) — {}",
            path.display(),
            siblings_of(&path)
        ),
    }
    unsafe { assert_eq!(ref_remove(c_path.as_ptr() as *const u8), 0) };
    assert!(!path.exists(), "ref_remove did not unlink the file");
}

/// THE ONE THAT MATTERS. A shim that always succeeds makes its caller's failure
/// branch unreachable, and every future difftest row over that branch reports
/// MATCH having exercised nothing (§9.66 — the `_wctomb_s_l` lesson). These
/// assertions are what prove the error path is wired end to end: POSIX errno →
/// fileimpl.c's host_err_to_win32 → the last-error slot → file.c's
/// msvcrt_set_errno → the reference's own errno.
#[test]
fn failures_propagate_all_the_way_to_errno() {
    let _serial = init();
    let missing = CString::new(format!(
        "{}/definitely-not-here-{}",
        std::env::temp_dir().display(),
        std::process::id()
    ))
    .unwrap();

    unsafe {
        ref_errno_set(0);
        let f = ref_fopen(missing.as_ptr() as *const u8, b"rb\0".as_ptr());
        assert!(f.is_null(), "ref_fopen succeeded on a path that does not exist");
        assert_eq!(
            ref_errno_get(),
            ref_msvcrt_ENOENT,
            "the ENOENT did not survive the trip through the Win32 shim"
        );

        ref_errno_set(0);
        let fd = ref__open(missing.as_ptr() as *const u8, O_RDONLY | O_BINARY);
        assert_eq!(fd, -1, "ref__open succeeded on a missing path");
        assert_eq!(ref_errno_get(), ref_msvcrt_ENOENT);

        // A closed/never-open fd must be EBADF, which only works if the shim
        // actually validates the handle rather than trusting it.
        ref_errno_set(0);
        let mut b = [0u8; 4];
        assert_eq!(ref__read(4096, b.as_mut_ptr(), 4), -1, "read on a bogus fd succeeded");
        assert_eq!(ref_errno_get(), ref_msvcrt_EBADF);
    }
}

/// The console arm of `_write` is REACHABLE — measured, not argued.
///
/// file.c's `_write` splits at `_isatty(fd) && VerifyConsoleIoHandle(hand)`. A
/// substrate that always answered "not a console" would delete the
/// `WriteConsoleW` path from every run, and a future difftest row over it would
/// report MATCH having executed nothing — the exact failure `ref_set_console_fd`
/// exists to prevent. This test is what stops the knob from quietly rotting into
/// dead code.
///
/// It asserts the branch was TAKEN rather than the bytes produced: both arms end
/// in `write(fd, …)`, so the effect alone cannot tell them apart. Byte-level
/// grading of the text-mode/UTF-16 conversion belongs with the Rust port, which
/// does not exist yet.
///
/// The fd is `/dev/null` — a real character device — and that is not incidental.
/// `_isatty` does NOT ask GetFileType; it reads `wxflag & WX_TTY`, which
/// `_wsopen_dispatch` latched at OPEN time (file.c:2551, :862). So an ordinary
/// temp file can never take this arm however it is marked afterwards, and the
/// knob governs only the `VerifyConsoleIoHandle` half of the condition. That
/// cost a failing test to learn, which is the test doing its job.
#[test]
fn the_console_write_path_is_reachable() {
    let _serial = init();
    let dev_null = CString::new("/dev/null").unwrap();

    unsafe {
        // TEXT mode, deliberately: `_write`'s binary path short-circuits to
        // WriteFile and RETURNS before the `_isatty` check ever runs
        // (file.c:3584), so the console arm does not exist for a binary fd.
        let fd = ref__open(dev_null.as_ptr() as *const u8, O_WRONLY | O_TEXT);
        // Both slots, plus the host's own answer. errno alone cannot say whether
        // the shim really failed or whether `_open` bailed BEFORE reaching it and
        // left a previous call's value standing — file.c's `wstrdupa_utf8`
        // failure arm returns -1 without touching errno, so a stale ENOENT from
        // the test that deliberately opens a missing path reads identically.
        assert!(
            fd >= 0,
            "ref__open(/dev/null) failed: errno={}, GetLastError={:#x}, and the host \
             opens it {}",
            ref_errno_get(),
            GetLastError(),
            if std::fs::OpenOptions::new().write(true).open("/dev/null").is_ok() {
                "fine"
            } else {
                "no better — /dev/null is genuinely unavailable here"
            }
        );

        // Baseline: a chardev that is NOT marked stays on the WriteFile arm, so
        // the counter also proves the knob is what decides — not the chardev-ness.
        // The HANDLE, not `fd`: `_open` hands back an MSVCRT fd (an index into
        // file.c's `__pioinfo`), which is a different numbering space from the
        // POSIX fd the shim layer uses. Keying this on `fd` made the test pass
        // by coincidence and then fail — the two happen to agree only while the
        // process has opened exactly the right number of files.
        let h = ref__get_osfhandle(fd);
        assert!(h > 0, "_get_osfhandle failed for crt fd {fd}");

        ref_console_write_reset();
        ref_set_console_handle(h, 0);
        assert_eq!(ref__write(fd, b"plain\n".as_ptr(), 6), 6);
        assert_eq!(
            ref_console_write_count(),
            0,
            "WriteConsoleW ran for an fd that was never marked as a console"
        );

        // Marked: the same call must route through WriteConsoleW.
        ref_set_console_handle(h, 1);
        let n = ref__write(fd, b"console\n".as_ptr(), 8);
        assert!(n > 0, "console-path _write failed, errno={}", ref_errno_get());
        assert!(
            ref_console_write_count() > 0,
            "the knob did not reach the WriteConsoleW arm — the branch is unreachable, \
             which is the whole failure this test exists to catch"
        );

        ref_set_console_handle(h, 0);
        assert_eq!(ref__close(fd), 0);
    }
}
