//! Grading rows for `dll/win32/msvcrt/except_i386.c` — reference TU #13, and the first one
//! that is 32-bit.
//!
//! WHAT IS GRADED
//! --------------
//! Seven exports (`_except_handler4_common`, `_local_unwind4`, `_seh_longjmp_unwind`,
//! `_seh_longjmp_unwind4`, `__CxxLongjmpUnwind`, `cxx_frame_handler`, `handle_fpieee_flt`)
//! and, through them, six of the TU's `static` bodies: `get_scopetable_v4`,
//! `msvcrt_local_unwind2`, `msvcrt_local_unwind4`, `cxx_local_unwind`, `check_noexcept`, and
//! `MSVCRT_nested_handler`.  The last one is reached the only way a `static` handler can be:
//! a row asks a callback, mid-unwind, to call the current head of the SEH chain — which is
//! the registration record the unwinder just pushed, whose `Handler` IS that side's own
//! `MSVCRT_nested_handler`.
//!
//! WHAT IS NOT, AND WHY
//! --------------------
//! * The six `__ASM_GLOBAL_FUNC` thunks.  They are irreducible machine code proven
//!   byte-for-byte against the C objects by objdump, and seh_i386.rs emits them from ONE
//!   instruction text under two symbol decorations, so a row would compare a transcription
//!   with itself.  They are substrate for both sides here.
//! * `call_catch_block` / `catch_function_nested_handler` / `se_translation_filter` — the
//!   catch-dispatch half.  Reaching them needs a real thrown C++ object matched by
//!   `find_catch_handler`, and the arm ends in `continue_after_catch`, which restores %esp
//!   from the throwing frame.  The rows below drive `cxx_frame_handler` right up to that
//!   boundary (magic validation, the synchronous gate, the unwinding arm, the rethrow copy,
//!   the custom_handler dispatch, `processing_throw` accounting) and stop there.  Recorded
//!   as owed rather than quietly skipped.
//!
//! COMPARING BY ROLE, NOT BY VALUE
//! -------------------------------
//! Three of the values in flight are addresses of C LOCALS, and the two sides have different
//! stack frames, so comparing them raw would report a divergence in correct code:
//!
//!   * `*((DWORD *)frame - 1)` receives `&exceptPtrs`, a local of the handler.
//!   * the SEH chain head, mid-unwind, is `&reg`, a local of `msvcrt_local_unwind*`.
//!   * the `%ebp` a thunk installs is `&frame->_ebp`, which IS shared — so that one is
//!     compared directly, and is the check that the right frame was handed over.
//!
//! So the snapshot normalises the first two into the relations that carry the meaning: did
//! the scratch slot get the pointer the filter was handed, and was a frame pushed at all.
//! This is the same lesson process.rs's `label_handle` records.

#[cfg(not(all(target_arch = "x86", target_os = "linux")))]
fn main() {
    // Not a skip that could pass for a run: `vibeos-cli difftest` treats a harness that
    // grades zero cases as a failure, so an accidental host-target build cannot look green.
    eprintln!(
        "msvcrt-seh-difftest: except_i386.c is 32-bit-only; build with \
         --target i686-unknown-linux-gnu"
    );
    std::process::exit(2);
}

/// Pull the subject into the link.
///
/// Nothing in this test calls the seven exports through Rust — sehimpl.c's drivers do — and
/// a static archive is resolved left-to-right, with msvcrt-crt-rust's rlib ahead of the
/// harness's `.a` on the command line. So by the time the linker reaches the C reference to
/// `_except_handler4_common`, the rlib member that defines it has already been passed over
/// and the symbol comes back undefined. Naming them here puts the reference in the test's
/// OWN object, which comes first.
///
/// Declared with no parameters on purpose: this is an address-taking anchor and never a
/// call, and giving it a second copy of each signature would be a second thing to keep in
/// step with the C.
#[cfg(all(target_arch = "x86", target_os = "linux"))]
mod anchor {
    // And the rlib has to be ON the link line before any of that matters: an unused
    // dependency is not passed to the linker at all, which is why this is an `extern crate`
    // and not a comment saying the dependency exists.
    extern crate msvcrt_crt_rust;

    unsafe extern "C" {
        fn _except_handler4_common();
        fn _local_unwind4();
        fn _seh_longjmp_unwind();
        fn _seh_longjmp_unwind4();
        fn __CxxLongjmpUnwind();
        fn cxx_frame_handler();
        fn handle_fpieee_flt();
    }

    pub fn keep() -> usize {
        _except_handler4_common as usize
            ^ _local_unwind4 as usize
            ^ _seh_longjmp_unwind as usize
            ^ _seh_longjmp_unwind4 as usize
            ^ __CxxLongjmpUnwind as usize
            ^ cxx_frame_handler as usize
            ^ handle_fpieee_flt as usize
    }
}

#[cfg(all(target_arch = "x86", target_os = "linux"))]
mod rows {
    use std::os::raw::{c_char, c_int, c_uchar, c_ulong};

    unsafe extern "C" {
        pub fn seh_reset();
        pub fn seh_set_cookie(c: c_ulong);
        pub fn seh_set_trylevel(t: c_int);
        pub fn seh_set_cxx_trylevel(t: c_int);
        pub fn seh_set_exception(code: u32, flags: u32);
        pub fn seh_set_exception_info(i: c_int, v: c_ulong);
        pub fn seh_set_filter_result(id: c_int, r: c_int);
        pub fn seh_probe_nested(flags: c_int);
        pub fn seh_row_label(s: *const c_char);
        pub fn seh_set_prev_exception(code: u32, flags: u32);
        pub fn seh_set_exc_type(mode: c_int);
        pub fn seh_add_scope(prev: c_int, filter_id: c_int, handler_kind: c_int, handler_id: c_int);
        pub fn seh_add_unwind(prev: c_int, handler_kind: c_int, handler_id: c_int);
        pub fn seh_add_tryblock(start: c_int, end: c_int, catch_level: c_int);
        pub fn seh_set_descr(magic: u32, bbt: u32, flags: u32);

        pub fn seh_run_handler4_common(side: c_int) -> c_int;
        pub fn seh_run_local_unwind4(side: c_int, trylevel: c_int) -> c_int;
        pub fn seh_run_seh_longjmp_unwind(side: c_int, trylevel: c_int) -> c_int;
        pub fn seh_run_seh_longjmp_unwind4(side: c_int, trylevel: c_int) -> c_int;
        pub fn seh_run_cxx_longjmp_unwind(side: c_int, trylevel: c_int) -> c_int;
        pub fn seh_run_cxx_frame_handler(side: c_int) -> c_int;

        pub fn seh_events() -> c_int;
        pub fn seh_event_overflow() -> c_int;
        pub fn seh_event_kind(i: c_int) -> c_int;
        pub fn seh_event_id(i: c_int) -> c_int;
        pub fn seh_event_ebp(i: c_int) -> u32;
        pub fn seh_event_trylevel(i: c_int) -> c_int;
        pub fn seh_event_chain(i: c_int) -> u32;
        pub fn seh_event_arg(i: c_int) -> u32;

        pub fn seh_frame_trylevel() -> c_int;
        pub fn seh_cxx_trylevel() -> c_int;
        pub fn seh_frame_scratch() -> u32;
        pub fn seh_cxx_scratch() -> u32;
        pub fn seh_dispatcher() -> u32;
        pub fn seh_left_by_jump() -> c_int;
        pub fn seh_processing_throw() -> c_int;
        pub fn seh_chain_head() -> u32;
        pub fn seh_ebp_addr() -> u32;
        pub fn seh_rec_code() -> u32;
        pub fn seh_rec_flags() -> u32;
        pub fn seh_rec_info(i: c_int) -> u32;

        pub fn seh_fp_setup(control: u32, status: u32, opcode: u32, operand2: f64, result: c_int);
        pub fn seh_run_fpieee(side: c_int, code: c_ulong) -> c_int;
        pub fn seh_fp_calls() -> c_int;
        pub fn seh_fp_seen_byte(i: c_int) -> c_uchar;
        pub fn seh_fp_seen_size() -> c_int;
        pub fn seh_fp_regarea_byte(i: c_int) -> c_uchar;
    }

    // Constants, spelled once. The reference's copies are asserted against the real i386
    // headers by refstub/seh_layout.h, so these are checked against the same authority
    // rather than against a second transcription.
    pub const EXCEPTION_UNWINDING: u32 = 0x02;
    pub const EXCEPTION_EXIT_UNWIND: u32 = 0x04;
    pub const CXX_EXCEPTION: u32 = 0xe06d_7363;
    pub const CXX_FRAME_MAGIC_VC6: u32 = 0x1993_0520;
    pub const CXX_FRAME_MAGIC_VC7: u32 = 0x1993_0521;
    pub const CXX_FRAME_MAGIC_VC8: u32 = 0x1993_0522;
    pub const FUNC_DESCR_SYNCHRONOUS: u32 = 1;
    pub const FUNC_DESCR_NOEXCEPT: u32 = 4;
    pub const STATUS_FLOAT_DIVIDE_BY_ZERO: u32 = 0xc000_008e;
    pub const STATUS_FLOAT_INVALID_OPERATION: u32 = 0xc000_0090;
    pub const STATUS_ACCESS_VIOLATION: u32 = 0xc000_0005;
    pub const EXCEPTION_CONTINUE_SEARCH: c_int = 0;
    pub const EXCEPTION_EXECUTE_HANDLER: c_int = 1;
    pub const EXCEPTION_CONTINUE_EXECUTION: c_int = -1;
    pub const CHAIN_END: u32 = 0xffff_ffff;

    // The C's own end-of-scope-list sentinels: -1 for the v3 table, -2 for v4.
    pub const TRYLEVEL_END: c_int = -1;
    pub const TRYLEVEL_END_V4: c_int = -2;

    /// One event, with the two stack-local values already reduced to the relation that
    /// survives a change of stack frame. See the module docstring.
    #[derive(PartialEq, Eq, Debug)]
    pub struct Ev {
        kind: c_int,
        id: c_int,
        /// The %ebp the thunk installed, compared against `&frame->_ebp`, which is shared.
        ebp_is_frame: bool,
        ebp_raw_if_shared: u32,
        trylevel: c_int,
        /// A registration record was on the chain at this moment (its address is a local).
        chain_pushed: bool,
        arg: u32,
    }

    #[derive(PartialEq, Eq, Debug)]
    pub struct Snap {
        pub ret: c_int,
        pub events: Vec<Ev>,
        pub overflow: c_int,
        pub frame_trylevel: c_int,
        pub cxx_trylevel: c_int,
        /// The scratch word is `&exceptPtrs`, a local — only its role is comparable.
        pub scratch_set: bool,
        pub scratch_is_filter_arg: bool,
        pub cxx_scratch: u32,
        pub dispatcher: u32,
        pub left_by_jump: c_int,
        pub processing_throw: c_int,
        pub chain_head: u32,
        pub rec_code: u32,
        pub rec_flags: u32,
        pub rec_info: [u32; 3],
    }

    const KIND_FILTER: c_int = 1;
    const KIND_RTLUNWIND: c_int = 5;

    pub unsafe fn snap(ret: c_int) -> Snap {
        let ebp_addr = seh_ebp_addr();
        let scratch = seh_frame_scratch();
        let mut events = Vec::new();
        let mut scratch_is_filter_arg = false;
        for i in 0..seh_events() {
            let kind = seh_event_kind(i);
            let ebp = seh_event_ebp(i);
            let arg = seh_event_arg(i);
            if kind == KIND_FILTER && arg == scratch && scratch != 0 {
                scratch_is_filter_arg = true;
            }
            events.push(Ev {
                kind,
                id: seh_event_id(i),
                ebp_is_frame: ebp == ebp_addr,
                // Only RtlUnwind's `ebp` slot carries something shared (the record it was
                // handed); everywhere else it is a stack address and the boolean above is
                // the whole comparison.
                ebp_raw_if_shared: if kind == KIND_RTLUNWIND { ebp } else { 0 },
                trylevel: seh_event_trylevel(i),
                chain_pushed: seh_event_chain(i) != CHAIN_END,
                // A filter's `arg` is `&exceptPtrs`; reduced above. Everything else here is
                // a shared object's address.
                arg: if kind == KIND_FILTER { 0 } else { arg },
            });
        }
        Snap {
            ret,
            events,
            overflow: seh_event_overflow(),
            frame_trylevel: seh_frame_trylevel(),
            cxx_trylevel: seh_cxx_trylevel(),
            scratch_set: scratch != 0,
            scratch_is_filter_arg,
            cxx_scratch: seh_cxx_scratch(),
            dispatcher: seh_dispatcher(),
            left_by_jump: seh_left_by_jump(),
            processing_throw: seh_processing_throw(),
            chain_head: seh_chain_head(),
            rec_code: seh_rec_code(),
            rec_flags: seh_rec_flags(),
            rec_info: [seh_rec_info(0), seh_rec_info(1), seh_rec_info(2)],
        }
    }

    /// Hand the row's own name to the watchdog before running it. A scenario that spins
    /// touches nothing the harness can see, so the label recorded up front is the only thing
    /// that can name the culprit when the alarm fires.
    unsafe fn label(what: &str, cfg: &str) {
        let mut buf = format!("{what} [{cfg}]").into_bytes();
        buf.truncate(150);
        buf.push(0);
        seh_row_label(buf.as_ptr() as *const c_char);
    }

    pub struct Grader {
        pub rows: u64,
        pub fails: u32,
    }

    impl Grader {
        pub fn new() -> Grader {
            Grader { rows: 0, fails: 0 }
        }

        /// reset -> subject -> snapshot -> reset -> reference -> snapshot -> compare.
        /// The setup closure runs twice, so the two sides see structurally identical inputs
        /// built from the same code rather than from two hand-matched copies.
        pub unsafe fn duel(
            &mut self,
            what: &str,
            cfg: &str,
            setup: &dyn Fn(),
            run: &dyn Fn(c_int) -> c_int,
        ) {
            label(what, &format!("{cfg} SUBJECT"));
            seh_reset();
            setup();
            let s = snap(run(0));

            label(what, &format!("{cfg} REFERENCE"));
            seh_reset();
            setup();
            let r = snap(run(1));

            self.rows += 1;
            if s != r {
                self.fails += 1;
                if self.fails <= 12 {
                    println!("DIVERGE {what} [{cfg}]");
                    println!("   subject: {s:?}");
                    println!(" reference: {r:?}");
                }
            }
        }

        pub unsafe fn duel_fp(&mut self, cfg: &str, setup: &dyn Fn(), code: u32) {
            label("handle_fpieee_flt", &format!("{cfg} SUBJECT"));
            seh_reset();
            setup();
            let sr = seh_run_fpieee(0, code as c_ulong);
            let s = fp_snap(sr);

            label("handle_fpieee_flt", &format!("{cfg} REFERENCE"));
            seh_reset();
            setup();
            let rr = seh_run_fpieee(1, code as c_ulong);
            let r = fp_snap(rr);

            self.rows += 1;
            if s != r {
                self.fails += 1;
                if self.fails <= 12 {
                    println!("DIVERGE handle_fpieee_flt [{cfg}]");
                    println!("   subject ret={} calls={}", s.0, s.1);
                    println!(" reference ret={} calls={}", r.0, r.1);
                    for i in 0..s.2.len() {
                        if s.2[i] != r.2[i] {
                            println!("   _FPIEEE_RECORD byte {i}: {:#04x} vs {:#04x}", s.2[i], r.2[i]);
                        }
                    }
                    for i in 0..s.3.len() {
                        if s.3[i] != r.3[i] {
                            println!("   RegisterArea byte {i}: {:#04x} vs {:#04x}", s.3[i], r.3[i]);
                        }
                    }
                }
            }
        }
    }

    /// `handle_fpieee_flt`'s whole output is the `_FPIEEE_RECORD` its handler receives, so
    /// the row compares all 112 bytes — the C memsets it first, so even the padding is
    /// defined and a byte that differs is a real difference.
    fn fp_snap(ret: c_int) -> (c_int, c_int, Vec<u8>, Vec<u8>) {
        unsafe {
            let n = seh_fp_seen_size();
            let seen = (0..n).map(|i| seh_fp_seen_byte(i)).collect();
            let reg = (0..80).map(|i| seh_fp_regarea_byte(i)).collect();
            (ret, seh_fp_calls(), seen, reg)
        }
    }
}

#[cfg(all(target_arch = "x86", target_os = "linux"))]
fn main() {
    use rows::*;
    use std::os::raw::c_int;

    // Keeps the seven subject exports in the link; see `mod anchor`.
    assert_ne!(anchor::keep(), 0, "the subject exports did not make it into the link");

    let mut g = Grader::new();

    // ── the scope-table shapes every unwind row is driven over ───────────
    //
    // Each entry is (previousTryLevel, filter slot or -1, handler kind, handler slot).
    // handler kind 0 = none, 1 = a handler that returns, 2 = a `__finally` arm that the
    // thunk JUMPS to and never returns from.
    //
    // `END` is a PLACEHOLDER, not a value, and that is the point: the v3 walk
    // (`msvcrt_local_unwind2`) stops on -1 and the v4 walk stops on -2. A shape written
    // with a literal terminator is correct for one of them and an unterminated chain for
    // the other — the walk then indexes `entries[-1]`, which in C reads the four header
    // words of the scope table and spins forever with nothing to observe. The first run of
    // this suite did exactly that. So every shape names its end abstractly and each caller
    // substitutes the sentinel its walk actually uses.
    const END: c_int = i32::MIN;
    type Scope = (c_int, c_int, c_int, c_int);
    let shapes: Vec<(&str, Vec<Scope>)> = vec![
        ("empty", vec![]),
        // one cleanup level: no filter, so the unwinder calls its handler
        ("cleanup1", vec![(END, -1, 1, 0)]),
        // three nested cleanups chained to the sentinel
        (
            "cleanup3",
            vec![(END, -1, 1, 0), (0, -1, 1, 1), (1, -1, 1, 2)],
        ),
        // A level that carries a FILTER always gets a kind-2 handler, and that is a
        // property of the C rather than a harness convenience: a filtered level is a
        // `__try/__except`, and `_except_handler4_common` enters its handler through
        // `call_finally_block`, which JUMPS. A kind-1 handler there would `ret` onto a
        // stack the jump never set up — which is exactly what wedged the first run.
        // Unfiltered levels are `__finally` cleanups reached by `call_handler`, which
        // calls and expects a return, so those stay kind 1.
        //
        // a filter level between two cleanups — the filter suppresses the handler call
        // during an unwind, and is the one entered on EXECUTE_HANDLER
        (
            "filter-mid",
            vec![(END, -1, 1, 0), (0, 3, 2, 0), (1, -1, 1, 2)],
        ),
        // every level has a filter: an unwind should call NO handler at all
        (
            "all-filters",
            vec![(END, 0, 2, 0), (0, 1, 2, 1), (1, 2, 2, 2)],
        ),
        // a `__finally` arm at the top, reached through call_finally_block
        ("finally-top", vec![(END, -1, 1, 0), (0, 4, 2, 3)]),
        // two independent chains that both end at the sentinel
        ("two-roots", vec![(END, -1, 1, 0), (END, -1, 1, 1)]),
        // a level pointing back two steps — the legal shape that most nearly tests the walk
        (
            "skip-back",
            vec![(END, -1, 1, 0), (0, -1, 1, 1), (0, -1, 1, 2), (2, -1, 1, 3)],
        ),
    ];

    // Substitute the sentinel this walk stops on, so the chain actually terminates.
    let with_end = |shape: &Vec<Scope>, end: c_int| -> Vec<Scope> {
        shape
            .iter()
            .map(|&(p, f, hk, hi)| (if p == END { end } else { p }, f, hk, hi))
            .collect()
    };

    let cookies: [u32; 4] = [0, 1, 0xdead_beef, 0xffff_ffff];
    let filter_results: [(&str, c_int); 3] = [
        ("search", EXCEPTION_CONTINUE_SEARCH),
        ("execute", EXCEPTION_EXECUTE_HANDLER),
        ("continue", EXCEPTION_CONTINUE_EXECUTION),
    ];

    // ── _except_handler4_common ──────────────────────────────────────────
    // Both arms: the unwinding branch (which runs msvcrt_local_unwind4 to the -2 sentinel)
    // and the hunting branch (which walks the scope table calling filters, and on
    // EXECUTE_HANDLER does _global_unwind2 + a partial unwind + call_finally_block).
    for (sname, shape) in &shapes {
        for &cookie in &cookies {
            // -1 is deliberately absent: it is the v3 sentinel, and a v4 walk handed it
            // indexes `entries[-1]` rather than stopping.
            for trylevel in [-2i32, 0, 1, 2] {
                for &(fname, fres) in &filter_results {
                    for &(flname, flags) in &[
                        ("none", 0u32),
                        ("unwinding", EXCEPTION_UNWINDING),
                        ("exit", EXCEPTION_EXIT_UNWIND),
                        ("both", EXCEPTION_UNWINDING | EXCEPTION_EXIT_UNWIND),
                    ] {
                        // Skip trylevels the shape cannot index: the C indexes the table
                        // with no bounds check, so an out-of-range level is undefined on
                        // BOTH sides and a row over it would grade a read of our own heap.
                        if trylevel >= 0 && (trylevel as usize) >= shape.len() {
                            continue;
                        }
                        let shape = with_end(shape, TRYLEVEL_END_V4);
                        let setup = move || unsafe {
                            seh_set_cookie(cookie as _);
                            seh_set_trylevel(trylevel);
                            seh_set_exception(STATUS_ACCESS_VIOLATION, flags);
                            for &(p, f, hk, hi) in &shape {
                                seh_add_scope(p, f, hk, hi);
                            }
                            for id in 0..8 {
                                seh_set_filter_result(id, fres);
                            }
                        };
                        unsafe {
                            g.duel(
                                "_except_handler4_common",
                                &format!(
                                    "{sname}/cookie={cookie:#x}/try={trylevel}/{fname}/{flname}"
                                ),
                                &setup,
                                &|side| seh_run_handler4_common(side),
                            );
                        }
                    }
                }
            }
        }
    }

    // ── _local_unwind4 and _seh_longjmp_unwind4 (the v4 table) ───────────
    for (sname, shape) in &shapes {
        for &cookie in &cookies {
            for trylevel in [-2i32, 0, 1, 2, 3] {
                for target in [-2i32, -1, 0, 1, 2] {
                    if trylevel >= 0 && (trylevel as usize) >= shape.len() {
                        continue;
                    }
                    let shape = with_end(shape, TRYLEVEL_END_V4);
                    let setup = move || unsafe {
                        seh_set_cookie(cookie as _);
                        seh_set_trylevel(trylevel);
                        for &(p, f, hk, hi) in &shape {
                            seh_add_scope(p, f, hk, hi);
                        }
                    };
                    let cfg = format!("{sname}/cookie={cookie:#x}/try={trylevel}/to={target}");
                    unsafe {
                        g.duel("_local_unwind4", &cfg, &setup, &|side| {
                            seh_run_local_unwind4(side, target)
                        });
                        g.duel("_seh_longjmp_unwind4", &cfg, &setup, &|side| {
                            seh_run_seh_longjmp_unwind4(side, target)
                        });
                    }
                }
            }
        }
    }

    // ── _seh_longjmp_unwind (the v3 table, via msvcrt_local_unwind2) ─────
    // The v3 walk stops on -1 rather than -2, and reads the scope table straight out of the
    // frame with no cookie — so the same shapes exercise a different sentinel and a
    // different indirection.
    for (sname, shape) in &shapes {
        for trylevel in [-1i32, 0, 1, 2, 3] {
            for target in [-1i32, 0, 1, 2] {
                if trylevel >= 0 && (trylevel as usize) >= shape.len() {
                    continue;
                }
                let shape = with_end(shape, TRYLEVEL_END);
                let setup = move || unsafe {
                    seh_set_trylevel(trylevel);
                    for &(p, f, hk, hi) in &shape {
                        seh_add_scope(p, f, hk, hi);
                    }
                };
                unsafe {
                    g.duel(
                        "_seh_longjmp_unwind",
                        &format!("{sname}/try={trylevel}/to={target}"),
                        &setup,
                        &|side| seh_run_seh_longjmp_unwind(side, target),
                    );
                }
            }
        }
    }

    // ── MSVCRT_nested_handler, reached mid-unwind ────────────────────────
    // While `msvcrt_local_unwind2/4` is walking, it has pushed a registration record whose
    // Handler is that side's own `static MSVCRT_nested_handler`. Asking a callback to call
    // it is the only way to grade a static — and the two answers it can give
    // (ContinueSearch, or CollidedUnwind with *dispatch set) are exactly its whole body.
    for (sname, shape) in &shapes {
        for &probe in &[0u32, EXCEPTION_UNWINDING, EXCEPTION_EXIT_UNWIND, 0x10] {
            for trylevel in [0i32, 1, 2] {
                if (trylevel as usize) >= shape.len() {
                    continue;
                }
                let shape = with_end(shape, TRYLEVEL_END);
                let setup = move || unsafe {
                    seh_set_trylevel(trylevel);
                    seh_probe_nested(probe as c_int);
                    for &(p, f, hk, hi) in &shape {
                        seh_add_scope(p, f, hk, hi);
                    }
                };
                unsafe {
                    g.duel(
                        "MSVCRT_nested_handler",
                        &format!("{sname}/try={trylevel}/probe={probe:#x}"),
                        &setup,
                        &|side| seh_run_seh_longjmp_unwind(side, -1),
                    );
                }
            }
        }
    }

    // ── __CxxLongjmpUnwind (and cxx_local_unwind under it) ───────────────
    //
    // `cxx_local_unwind` calls `terminate()` on a trylevel outside the unwind table, and
    // terminate is the SUBJECT's (cpp.rs's) on both sides — it would take the process down
    // rather than grade anything. So the rows stay inside the table, and the bounds arm is
    // recorded as ungraded rather than reached.
    let unwind_shapes: Vec<(&str, Vec<(c_int, c_int, c_int)>)> = vec![
        ("u-empty", vec![]),
        ("u1", vec![(-1, 1, 0)]),
        ("u3", vec![(-1, 1, 0), (0, 1, 1), (1, 1, 2)]),
        ("u3-holes", vec![(-1, 0, 0), (0, 1, 1), (1, 0, 0)]),
        ("u-skip", vec![(-1, 1, 0), (-1, 1, 1), (0, 1, 2), (1, 1, 3)]),
    ];
    for (uname, ushape) in &unwind_shapes {
        for trylevel in [-1i32, 0, 1, 2, 3] {
            for target in [-1i32, 0, 1, 2] {
                if trylevel >= 0 && (trylevel as usize) >= ushape.len() {
                    continue;
                }
                // `cxx_local_unwind` bounds-checks INSIDE the loop and calls `terminate()`
                // on a level outside the table — so unlike the scope-table walks, a target
                // that is not ON the chain from `trylevel` is not a slow row, it is a dead
                // process. Follow the chain and take only the levels it actually visits.
                let mut reachable = vec![trylevel];
                let mut walk = trylevel;
                while walk >= 0 && (walk as usize) < ushape.len() {
                    walk = ushape[walk as usize].0;
                    reachable.push(walk);
                }
                if !reachable.contains(&target) {
                    continue;
                }
                let ushape = ushape.clone();
                let setup = move || unsafe {
                    seh_set_cxx_trylevel(trylevel);
                    for &(p, hk, hi) in &ushape {
                        seh_add_unwind(p, hk, hi);
                    }
                    seh_set_descr(CXX_FRAME_MAGIC_VC8, 0, 0);
                };
                unsafe {
                    g.duel(
                        "__CxxLongjmpUnwind",
                        &format!("{uname}/try={trylevel}/to={target}"),
                        &setup,
                        &|side| seh_run_cxx_longjmp_unwind(side, target),
                    );
                }
            }
        }
    }

    // ── cxx_frame_handler's decision logic ───────────────────────────────
    for &(mname, magic) in &[
        ("below-VC6", CXX_FRAME_MAGIC_VC6 - 1),
        ("VC6", CXX_FRAME_MAGIC_VC6),
        ("VC7", CXX_FRAME_MAGIC_VC7),
        ("VC8", CXX_FRAME_MAGIC_VC8),
        ("above-VC8", CXX_FRAME_MAGIC_VC8 + 1),
    ] {
        for &(dfname, dflags) in &[
            ("f0", 0u32),
            ("sync", FUNC_DESCR_SYNCHRONOUS),
            ("noexcept", FUNC_DESCR_NOEXCEPT),
            ("sync|noexcept", FUNC_DESCR_SYNCHRONOUS | FUNC_DESCR_NOEXCEPT),
        ] {
            for &(cname, code) in &[
                ("cxx", CXX_EXCEPTION),
                ("av", STATUS_ACCESS_VIOLATION),
            ] {
                for &(exname, exflags) in &[
                    ("none", 0u32),
                    ("unwinding", EXCEPTION_UNWINDING),
                    ("exit", EXCEPTION_EXIT_UNWIND),
                ] {
                    for &(tname, exc_type_mode) in
                        &[("no-type", 0i32), ("plain-type", 1), ("custom", 2)]
                    {
                        for &(uname, ushape) in &[
                            ("u0", &vec![] as &Vec<(c_int, c_int, c_int)>),
                            ("u2", &vec![(-1, 1, 0), (0, 1, 1)]),
                        ] {
                            for &(bname, blocks) in &[
                                ("t0", &vec![] as &Vec<(c_int, c_int, c_int)>),
                                // a try block the frame's trylevel sits OUTSIDE of, so the
                                // catch search runs its loop and finds nothing without
                                // reaching find_catch_handler
                                ("t-out", &vec![(5, 7, 5)]),
                            ] {
                                for cxx_trylevel in [-1i32, 0, 1] {
                                    // `check_noexcept` terminates on a noexcept descriptor
                                    // propagating a C++ exception; terminate is shared and
                                    // fatal, so that combination stays out.
                                    if magic >= CXX_FRAME_MAGIC_VC8
                                        && (dflags & FUNC_DESCR_NOEXCEPT) != 0
                                        && code == CXX_EXCEPTION
                                        && (exflags & (EXCEPTION_UNWINDING | EXCEPTION_EXIT_UNWIND))
                                            == 0
                                    {
                                        continue;
                                    }
                                    // `cxx_local_unwind` on the unwinding arm walks from the
                                    // frame's trylevel; keep it inside the table for the
                                    // same reason as above.
                                    if (exflags & (EXCEPTION_UNWINDING | EXCEPTION_EXIT_UNWIND)) != 0
                                        && cxx_trylevel >= ushape.len() as i32
                                        && cxx_trylevel != -1
                                    {
                                        continue;
                                    }
                                    let ushape = ushape.clone();
                                    let blocks = blocks.clone();
                                    let setup = move || unsafe {
                                        seh_set_cxx_trylevel(cxx_trylevel);
                                        seh_set_exception(code, exflags);
                                        // The rethrow arm copies the thread's previous
                                        // record over this one; furnish it so the arm is
                                        // reachable rather than a null dereference.
                                        seh_set_prev_exception(STATUS_ACCESS_VIOLATION, 0);
                                        seh_set_exception_info(0, CXX_FRAME_MAGIC_VC8 as _);
                                        seh_set_exception_info(1, 0);
                                        seh_set_exc_type(exc_type_mode);
                                        for &(p, hk, hi) in &ushape {
                                            seh_add_unwind(p, hk, hi);
                                        }
                                        for &(s, e, c) in &blocks {
                                            seh_add_tryblock(s, e, c);
                                        }
                                        seh_set_descr(magic, 0, dflags);
                                    };
                                    unsafe {
                                        g.duel(
                                            "cxx_frame_handler",
                                            &format!(
                                                "{mname}/{dfname}/{cname}/{exname}/{tname}/\
                                                 {uname}/{bname}/try={cxx_trylevel}"
                                            ),
                                            &setup,
                                            &|side| seh_run_cxx_frame_handler(side),
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // ── handle_fpieee_flt ────────────────────────────────────────────────
    //
    // The one body in this TU that is pure arithmetic: 60 lines turning an x87 control and
    // status word into a `_FPIEEE_RECORD`. Every row compares all 112 bytes of that record
    // plus the return value and the register area the C may write back, so a single wrong
    // shift or a swapped Enable/Status is caught by construction.
    let controls: [u32; 12] = [
        0x0000, 0x027f, 0x037f, 0x003f, 0x0000_003c, 0x0000_0300, 0x0000_0200, 0x0000_0100,
        0x0000_0c00, 0x0000_ffff, 0x0000_0a3f, 0x0000_0555,
    ];
    let statuses: [u32; 10] = [
        0x0000, 0x0001, 0x0004, 0x0008, 0x0010, 0x0020, 0x003d, 0x003f, 0xffff, 0x0025,
    ];
    let codes: [(&str, u32); 4] = [
        ("div0", STATUS_FLOAT_DIVIDE_BY_ZERO),
        ("invalid", STATUS_FLOAT_INVALID_OPERATION),
        ("overflow", 0xc000_0091),
        ("other", 0),
    ];
    let opcodes: [(&str, u32); 3] = [("fdiv", 0x35dc), ("not-fdiv", 0x1234), ("zero", 0)];
    let results: [(&str, c_int); 3] = [
        ("continue-exec", EXCEPTION_CONTINUE_EXECUTION),
        ("execute", EXCEPTION_EXECUTE_HANDLER),
        ("search", EXCEPTION_CONTINUE_SEARCH),
    ];
    for &control in &controls {
        for &status in &statuses {
            for &(oname, opcode) in &opcodes {
                for &(cname, code) in &codes {
                    for &(rname, res) in &results {
                        let setup = move || unsafe {
                            seh_fp_setup(control, status, opcode, 3.5_f64, res);
                        };
                        unsafe {
                            g.duel_fp(
                                &format!(
                                    "cw={control:#06x}/sw={status:#06x}/{oname}/{cname}/{rname}"
                                ),
                                &setup,
                                code,
                            );
                        }
                    }
                }
            }
        }
    }

    if g.fails > 0 {
        println!("seh difftest FAILED: {} of {} rows diverged", g.fails, g.rows);
        std::process::exit(1);
    }
    println!("seh difftest result: ok. {} passed", g.rows);
}
