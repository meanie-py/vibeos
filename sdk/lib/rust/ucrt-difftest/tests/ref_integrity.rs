//! The oracle auditing ITSELF, before anything is graded against it.
//!
//! Every other suite here treats the compiled UCRT C as ground truth. This one
//! asks whether that truth survived the trip: the reference is Microsoft's
//! Windows source pushed through a Linux compiler with a pile of shadowed
//! headers, and a reference that got miscompiled on the way in reports
//! confident nonsense. The sibling msvcrt-difftest learned this the hard way —
//! glibc's `__nonnull` leaked onto its renamed symbols and deleted msvcrt's
//! NULL guards, and the suite stayed green while grading against a rewritten C.
//!
//! It also fails loudly when there is NO reference left, rather than passing on
//! an empty corpus.

#[cfg(not(no_ucrt_reference))]
extern "C" {
    fn ref_abs(x: i32) -> i32;
    fn ref_abs64(x: i64) -> i64;
    fn ref_imaxabs(x: i64) -> i64;
    #[cfg(ucrt_ref_strcat)]
    fn ref_strcpy(dst: *mut u8, src: *const u8) -> *mut u8;
}

fn main() {
    #[cfg(no_ucrt_reference)]
    {
        eprintln!(
            "ucrt-difftest: every reference TU has been deleted, so this suite \
             grades NOTHING. That is not a pass. Either restore a reference or \
             retire the suite deliberately."
        );
        std::process::exit(1);
    }

    #[cfg(not(no_ucrt_reference))]
    unsafe {
        // Sanity, not coverage: if these are wrong the reference did not
        // survive compilation and no verdict built on it means anything.
        assert_eq!(ref_abs(-5), 5, "reference abs is not abs");
        assert_eq!(ref_abs(0), 0);
        assert_eq!(ref_abs(i32::MAX), i32::MAX);
        // INT_MIN is the interesting one: C says undefined, every real
        // implementation returns it unchanged, and a compiler that "optimised"
        // that into something else would be a rewritten reference.
        assert_eq!(ref_abs(i32::MIN), i32::MIN, "abs(INT_MIN) was rewritten");
        assert_eq!(ref_abs64(-9_000_000_000), 9_000_000_000);
        assert_eq!(ref_imaxabs(-42), 42);

        #[allow(unused_mut)]
        let mut checked = 6;
        // string/strcat.c is gone once the string family is taken over (its TU is
        // skipped by build.rs), and with it this reference.
        #[cfg(ucrt_ref_strcat)]
        {
            let src = b"hello\0";
            let mut dst = [0u8; 8];
            let ret = ref_strcpy(dst.as_mut_ptr(), src.as_ptr());
            assert_eq!(ret, dst.as_mut_ptr(), "strcpy must return dst");
            assert_eq!(&dst[..6], src, "reference strcpy did not copy");
            checked += 2;
        }

        println!("ucrt-difftest ref_integrity: reference compiled intact");
        // The reference assertions above; see classify.rs on why the count matters.
        println!("difftest result: ok. {checked} passed");
    }
}
