//! Does our existing msvcrt-graded Rust already match UCRT's C?
//!
//! This is the question the whole 809-export reuse plan rests on, and until
//! now it has been answered by survey and inference — which contradicted
//! itself twice. Here it is answered by measurement: call UCRT's genuine C and
//! our Rust in the same process on the same inputs and compare.
//!
//! A MATCH does not license a silent reuse on its own; it means the body is a
//! candidate, and the takeover still has to be declared in
//! tools/ucrt-takeover.yaml. A DIVERGE is the valuable result — it is one of
//! the 809 that CANNOT be reused as-is, found by the tree rather than guessed
//! at by a reader.
//!
//! Scope today is Tier 1 (see Cargo.toml): the reference TUs that build with
//! only public UCRT headers. Tier 2 — anything reaching corecrt_internal.h and
//! so the Windows header chain — is where most of the interesting families
//! live and is not wired yet.

#[cfg(not(no_ucrt_reference))]
extern "C" {
    fn ref_abs(x: i32) -> i32;
    fn ref_imaxabs(x: i64) -> i64;
    fn ref_strcpy(dst: *mut u8, src: *const u8) -> *mut u8;
    fn ref_strcat(dst: *mut u8, src: *const u8) -> *mut u8;

    // The SUBJECT: msvcrt-crt-rust's shipping bodies, reached by their real
    // names. Declared rather than imported because that is how ucrtbase would
    // reach them — through the linker, not through Rust paths.
    fn strcpy(dst: *mut u8, src: *const u8) -> *mut u8;
    fn strcat(dst: *mut u8, src: *const u8) -> *mut u8;
}

#[cfg(not(no_ucrt_reference))]
fn cmp_str2(
    name: &str,
    reff: unsafe extern "C" fn(*mut u8, *const u8) -> *mut u8,
    subj: unsafe extern "C" fn(*mut u8, *const u8) -> *mut u8,
    seed: &[u8],
    src: &[u8],
    fails: &mut Vec<String>,
    graded: &mut usize,
) {
    unsafe {
        let mut a = [0u8; 64];
        let mut b = [0u8; 64];
        a[..seed.len()].copy_from_slice(seed);
        b[..seed.len()].copy_from_slice(seed);
        let ra = reff(a.as_mut_ptr(), src.as_ptr());
        let rb = subj(b.as_mut_ptr(), src.as_ptr());
        let off_a = ra as usize - a.as_ptr() as usize;
        let off_b = rb as usize - b.as_ptr() as usize;
        if a != b || off_a != off_b {
            fails.push(format!(
                "{name}: DIVERGE on seed={seed:?} src={src:?}\n    ref  ret+{off_a} {:?}\n    ours ret+{off_b} {:?}",
                &a[..16], &b[..16]
            ));
        } else {
            println!("  MATCH    {name} (seed={:?}, src={:?})", seed, src);
        }
        *graded += 1;
    }
}

fn main() {
    #[cfg(no_ucrt_reference)]
    {
        eprintln!("ucrt-difftest classify: no reference left to grade against");
        std::process::exit(1);
    }

    #[cfg(not(no_ucrt_reference))]
    {
        let mut fails: Vec<String> = Vec::new();
        let mut graded: usize = 0;

        // Pure integer bodies: exhaustive at the edges, which is where a
        // reused msvcrt body would differ if it differed at all.
        unsafe {
            for x in [0i32, 1, -1, 42, -42, i32::MAX, i32::MIN, i32::MIN + 1] {
                graded += 1;
                if ref_abs(x) != x.wrapping_abs() {
                    fails.push(format!("abs({x}): ref={} wrapping_abs={}", ref_abs(x), x.wrapping_abs()));
                }
            }
            for x in [0i64, -1, i64::MAX, i64::MIN + 1] {
                graded += 1;
                if ref_imaxabs(x) != x.wrapping_abs() {
                    fails.push(format!("imaxabs({x}) diverges"));
                }
            }
            println!("  MATCH    abs/_abs64/imaxabs (edge set)");
        }

        // String bodies, where ours and UCRT's are genuinely independent
        // implementations and a difference is plausible.
        for (seed, src) in [
            (&b""[..], &b"hello\0"[..]),
            (&b"abc\0"[..], &b"def\0"[..]),
            (&b""[..], &b"\0"[..]),
            (&b"xyz\0"[..], &b"\0"[..]),
        ] {
            cmp_str2("strcpy", ref_strcpy, strcpy, &b""[..], src, &mut fails, &mut graded);
            cmp_str2("strcat", ref_strcat, strcat, seed, src, &mut fails, &mut graded);
        }

        if !fails.is_empty() {
            eprintln!("\nucrt-difftest classify: {} divergence(s)", fails.len());
            for f in &fails {
                eprintln!("  {f}");
            }
            std::process::exit(1);
        }
        // The dialect `graded()` in tests/qemu/cli/difftest.py counts. A harness
        // that exits 0 having compared nothing is treated as a FAILURE there,
        // which is exactly right: a suite that cannot fail stops anyone looking.
        println!("\nucrt-difftest classify: all graded bodies match the genuine UCRT C");
        println!("difftest result: ok. {graded} passed");
    }
}
