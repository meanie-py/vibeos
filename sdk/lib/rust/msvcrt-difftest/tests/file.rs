#![no_builtins]
// Wave 18a — grade `msvcrt-crt-rust/src/msvcrt/file.rs` against the GENUINE
// `dll/win32/msvcrt/file.c` it replaces.
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
    // `_dup2` is deliberately absent: it REPLACES a descriptor rather than allocating
    // one, so a row over it has to keep both sides' tables in step by hand and
    // `balanced()` cannot help. It is on wave 18b's list with the rest.
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
        ref_msvcrt_init_io();
        msvcrt_init_io();
        assert_shims_live();
        run();
    }

    let res = unsafe { &*core::ptr::addr_of!(RESULTS) };
    let (mut m, mut d, mut u) = (0, 0, 0);
    println!("\n=== wave 18a: msvcrt-crt-rust file.rs (SHIPPING) vs GENUINE msvcrt file.c ===\n");
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

    let _ = std::fs::remove_file(&p);
}

unsafe fn run() {
    fd_layer_rows();
    stream_rows();
    path_rows();
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
