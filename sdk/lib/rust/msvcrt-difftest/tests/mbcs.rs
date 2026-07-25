//! Runs the generated mbcs differential cases.
//!
//! The cases themselves are in `src/mbcs_cases.rs`, generated from mbcs.c's own
//! signatures by `rustconv difftest-gen`. This file only drives them — deliberately, so
//! that regenerating never touches hand-written code.
//!
//! `tests/mbcs_fixture.rs` must pass first: every case here reads its answer out of one
//! `threadmbcinfo`, and if that came back empty these all still run, both sides take the
//! single-byte path, and the suite is green having graded nothing.

/// Forces the shipping crate into the link.
///
/// Its mbcs entry points are `#[no_mangle]` and never referenced from Rust — the cases
/// call them through `extern "C"` declarations — so without this the linker has no
/// reason to pull those objects out of the rlib and every subject symbol comes back
/// undefined. Same reason classify.rs carries it.
extern crate msvcrt_crt_rust;

fn main() {
    eprintln!("\nmbcs differential cases\n");
    let n = msvcrt_difftest::mbcs_cases::run_all();
    println!("\n  {n} mbcs cases passed\n");
}
