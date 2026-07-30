//! The codepage converters are shared state, and cargo runs tests on threads.
//!
//! `nlsimpl.c` caches one `iconv_t` per codepage in a process-global array
//! (`to_wide_cache` / `to_mb_cache`) and hands the same descriptor to every
//! caller. An `iconv_t` is not a pure function handle — `iconv()` mutates the
//! conversion state inside it, and `iconv(cd, NULL, NULL, NULL, NULL)` resets
//! that state — so two threads converting at once is undefined behaviour.
//! glibc's failure mode is a short or garbled result, not an error return,
//! which is the worst possible shape: the caller sees success.
//!
//! `MultiByteToWideChar` is deliberately NOT namespaced into `ref_*` (kernel32
//! is shared environment, not code under test — refimpl.c says so at the
//! last-error slot), so this test can call the very function both sides use.
//!
//! The assertion is a comparison against the SAME conversion performed with no
//! other thread running. A conversion that is correct only when it has the
//! process to itself is the bug being described.

use std::sync::{Arc, Barrier};

extern "C" {
    fn MultiByteToWideChar(
        cp: u32,
        flags: u32,
        src: *const u8,
        srclen: i32,
        dst: *mut u16,
        dstlen: i32,
    ) -> i32;
}

/// Shift-JIS. A multi-byte codepage is the point: the single-byte ASCII path in
/// `MultiByteToWideChar` never opens an iconv descriptor at all, so a test that
/// only converts ASCII exercises none of this.
const CP932: u32 = 932;

fn convert(src: &[u8]) -> Vec<u16> {
    let mut out = vec![0u16; 64];
    let n = unsafe {
        MultiByteToWideChar(
            CP932,
            0,
            src.as_ptr(),
            src.len() as i32,
            out.as_mut_ptr(),
            out.len() as i32,
        )
    };
    assert!(n >= 0, "MultiByteToWideChar returned {n}");
    out.truncate(n as usize);
    out
}

#[test]
fn the_codepage_converter_is_safe_to_call_from_two_threads() {
    // Shift-JIS "日本語abc" — lead/trail byte pairs, so a descriptor that loses
    // its place mid-character produces visibly different output rather than
    // merely a different length.
    const SRC: &[u8] = b"\x93\xfa\x96\x7b\x8c\xea\x61\x62\x63";

    // Establish the truth with nothing else running.
    let expected = convert(SRC);
    assert!(!expected.is_empty(), "the fixture converted to nothing");

    const THREADS: usize = 8;
    const CONVERSIONS_PER_THREAD: usize = 2000;

    let gate = Arc::new(Barrier::new(THREADS));
    let expected = Arc::new(expected);

    let mut handles = Vec::new();
    for _ in 0..THREADS {
        let gate = Arc::clone(&gate);
        let expected = Arc::clone(&expected);
        handles.push(std::thread::spawn(move || {
            // Released together, so the conversions overlap instead of politely
            // queueing behind thread startup.
            gate.wait();
            for i in 0..CONVERSIONS_PER_THREAD {
                let got = convert(SRC);
                assert_eq!(
                    got, *expected,
                    "iteration {i}: the shared iconv descriptor produced a \
                     different answer under concurrency"
                );
            }
        }));
    }

    for h in handles {
        h.join().expect("a worker thread saw a corrupted conversion");
    }
}
