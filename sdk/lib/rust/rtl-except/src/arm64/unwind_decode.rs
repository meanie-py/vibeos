//! ARM64 `.xdata` unwind-code interpreter + packed-unwind expansion.
//!
//! This replaces the historical leaf-only stub (`Pc = Lr`) in `except.rs`'s
//! `RtlVirtualUnwind` with a full decoder per the Windows ARM64 exception
//! model (MS "ARM64 exception handling" spec). It is authored fresh — upstream
//! ReactOS `sdk/lib/rtl/arm64/except.c` never implemented it, so there is no C
//! to translate.
//!
//! The interpreter runs in BOTH kernel and user mode (librtl is shared), so it
//! must be allocation-free, panic-free, and tolerant of malformed data (it
//! bounds every array access rather than trusting the producer).
//!
//! Model: unwind codes describe the prolog's saves/allocations, stored in
//! teardown order (reverse-prolog). We execute them front-to-back against a
//! mutable CONTEXT: `alloc_*`/pre-indexed saves advance SP toward the caller;
//! `save_*` codes reload callee-saved registers from the frame; `set_fp`/
//! `add_fp` recover SP from the frame pointer. When the sequence ends, `Pc` is
//! set from the recovered `Lr` and SP holds the caller's SP.
//!
//! Partial prolog/epilog unwinding (fault mid-prolog or mid-epilog) is handled
//! via the epilog-scope table and the E-bit single-epilog fast path, matching
//! the spec's start-index rules.

use crate::types::{UNW_FLAG_EHANDLER, UNW_FLAG_UHANDLER};
use ntkrnl_sys::ddk::*;

/// One decoded save target — where a register was stored, so we can reload it
/// and (optionally) record the save slot into KNONVOLATILE_CONTEXT_POINTERS.
#[inline(always)]
unsafe fn ctx_x(Context: PCONTEXT) -> *mut ULONG64 {
    (*Context).__bindgen_anon_1.X.as_mut_ptr()
}

#[inline(always)]
unsafe fn set_x(Context: PCONTEXT, reg: usize, val: ULONG64) {
    if reg < 31 {
        *ctx_x(Context).add(reg) = val;
    }
}

#[inline(always)]
unsafe fn set_v_low(Context: PCONTEXT, reg: usize, val: ULONG64) {
    if reg < 32 {
        (*Context).V[reg].__bindgen_anon_1.Low = val;
    }
}

/// Read a u64 from guest/host stack memory. In kernel-mode dispatch the frame
/// is trusted kernel stack; in user mode the caller has already validated the
/// establisher-frame bounds. A raw read is what the C unwinder does; we keep it
/// but callers gate on RtlpGetStackLimits like the amd64/i386 paths.
#[inline(always)]
unsafe fn read_u64(addr: ULONG64) -> ULONG64 {
    core::ptr::read_unaligned(addr as *const ULONG64)
}

/// NV-context slot recorder (optional): records the address a register was
/// restored from, used by debuggers / collided unwind. `first` is the X index.
#[inline(always)]
unsafe fn record_int(Nv: PKNONVOLATILE_CONTEXT_POINTERS, reg: usize, slot: ULONG64) {
    if Nv.is_null() {
        return;
    }
    // X19..X28 are contiguous PULONG64 fields, then Fp(29), Lr(30).
    match reg {
        19..=28 => {
            let base = &raw mut (*Nv).X19;
            *base.add(reg - 19) = slot as *mut ULONG64;
        }
        29 => (*Nv).Fp = slot as *mut ULONG64,
        30 => (*Nv).Lr = slot as *mut ULONG64,
        _ => {}
    }
}

#[inline(always)]
unsafe fn record_fp(Nv: PKNONVOLATILE_CONTEXT_POINTERS, reg: usize, slot: ULONG64) {
    if Nv.is_null() {
        return;
    }
    // D8..D15 are contiguous PULONG64 fields after Lr.
    if (8..=15).contains(&reg) {
        let base = &raw mut (*Nv).D8;
        *base.add(reg - 8) = slot as *mut ULONG64;
    }
}

/// Restore an integer register pair `x(first), x(first+1)` from `[addr]`.
#[inline(always)]
unsafe fn restore_int_pair(
    Context: PCONTEXT,
    Nv: PKNONVOLATILE_CONTEXT_POINTERS,
    first: usize,
    addr: ULONG64,
) {
    set_x(Context, first, read_u64(addr));
    record_int(Nv, first, addr);
    set_x(Context, first + 1, read_u64(addr + 8));
    record_int(Nv, first + 1, addr + 8);
}

#[inline(always)]
unsafe fn restore_int_single(
    Context: PCONTEXT,
    Nv: PKNONVOLATILE_CONTEXT_POINTERS,
    reg: usize,
    addr: ULONG64,
) {
    set_x(Context, reg, read_u64(addr));
    record_int(Nv, reg, addr);
}

#[inline(always)]
unsafe fn restore_fp_pair(
    Context: PCONTEXT,
    Nv: PKNONVOLATILE_CONTEXT_POINTERS,
    first: usize,
    addr: ULONG64,
) {
    set_v_low(Context, first, read_u64(addr));
    record_fp(Nv, first, addr);
    set_v_low(Context, first + 1, read_u64(addr + 8));
    record_fp(Nv, first + 1, addr + 8);
}

#[inline(always)]
unsafe fn restore_fp_single(
    Context: PCONTEXT,
    Nv: PKNONVOLATILE_CONTEXT_POINTERS,
    reg: usize,
    addr: ULONG64,
) {
    set_v_low(Context, reg, read_u64(addr));
    record_fp(Nv, reg, addr);
}

/// Length in bytes of the unwind code whose first byte is `b0` (used to skip
/// codes when starting mid-epilog / mid-prolog).
fn unwind_code_len(b0: u8) -> usize {
    match b0 {
        0x00..=0xBF => 1,                // alloc_s / save_r19r20_x / save_fplr / save_fplr_x
        0xC0..=0xC7 => 2,                // alloc_m
        0xC8..=0xDF => 2,                // save_regp/_x, save_reg[_x], save_lrpair, save_fregp[_x], save_freg[_x], alloc_z
        0xE0 => 4,                       // alloc_l
        0xE1 => 1,                       // set_fp
        0xE2 => 2,                       // add_fp
        0xE3 => 1,                       // nop
        0xE4 => 1,                       // end
        0xE5 => 1,                       // end_c
        0xE6 => 1,                       // save_next
        0xE7 => 3,                       // save_any_reg
        0xE8..=0xEC => 1,                // MSFT_OP_* custom stack frames
        0xFC => 1,                       // pac_sign_lr
        0xF8 => 2,
        0xF9 => 3,
        0xFA => 4,
        0xFB => 5,
        _ => 1,                          // reserved encodings — treat as 1-byte no-op
    }
}

/// Resolve a `save_next` against the pair-save that precedes it in the
/// *prologue*. Unwind codes are stored in reverse-prologue order, so that base
/// pair appears later in the byte stream, after the whole `save_next` run.
/// `ordinal` is one for the `save_next` nearest the base pair, two for the next
/// pair, and so on.
unsafe fn save_next_target(
    codes: *const u8,
    codes_len: usize,
    mut base_index: usize,
    ordinal: usize,
    sp: ULONG64,
) -> Option<(usize, ULONG64, bool)> {
    while base_index < codes_len && *codes.add(base_index) == 0xE6 {
        base_index += 1;
    }
    if base_index >= codes_len {
        return None;
    }

    let b0 = *codes.add(base_index);
    let (first, address, is_fp) = match b0 {
        0x20..=0x3F => (19usize, sp, false),
        0xC8..=0xCF => {
            if base_index + 1 >= codes_len {
                return None;
            }
            let b1 = *codes.add(base_index + 1);
            let x = ((((b0 & 0x03) as usize) << 2) | (b1 >> 6) as usize) & 0x0F;
            let z = (b1 & 0x3F) as ULONG64;
            let address = if b0 & 0x04 != 0 { sp } else { sp + z * 8 };
            (19 + x, address, false)
        }
        0xD8..=0xDB => {
            if base_index + 1 >= codes_len {
                return None;
            }
            let b1 = *codes.add(base_index + 1);
            let x = (((b0 & 0x01) << 2) | (b1 >> 6)) as usize;
            let z = (b1 & 0x3F) as ULONG64;
            let address = if b0 & 0x02 != 0 { sp } else { sp + z * 8 };
            (8 + x, address, true)
        }
        0xE7 => {
            if base_index + 2 >= codes_len {
                return None;
            }
            let b1 = *codes.add(base_index + 1);
            let b2 = *codes.add(base_index + 2);
            if (b1 >> 6) & 1 == 0 {
                return None;
            }
            let first = (b1 & 0x1F) as usize;
            let preindexed = (b1 >> 5) & 1 != 0;
            let mode = (b2 >> 6) & 0x03;
            let offset = (b2 & 0x3F) as ULONG64 * 16;
            (first, if preindexed { sp } else { sp + offset }, mode != 0)
        }
        _ => return None,
    };

    Some((first + 2 * ordinal, address + 16 * ordinal as ULONG64, is_fp))
}

/// Result of an unwind: the SP delta model is folded into the mutated CONTEXT.
/// Returns `true` on a clean `end`/`end_c`, `false` if the stream ran off.
///
/// `codes` is the byte slice of unwind codes (section 3 of `.xdata`).
/// `start` is the index of the first code to execute (0 for body unwind, or the
/// epilog/prolog partial start index).
unsafe fn interpret_codes(
    Context: PCONTEXT,
    Nv: PKNONVOLATILE_CONTEXT_POINTERS,
    codes: *const u8,
    codes_len: usize,
    start: usize,
) -> bool {
    let mut i = start;
    // save_next (0xE6) resolves FORWARD, not backward — see save_next_target.
    // Codes are stored in reverse-prologue order, so the explicit pair-save a
    // save_next run hangs off of appears AFTER the run in the byte stream, and
    // each step back toward lower indices is one register pair HIGHER and 16
    // bytes further up the frame. Being a plain (non-pre-indexed) stp/ldp, a
    // save_next contributes ZERO to SP; mutating SP by +16 per save_next was
    // the §9.69 / KeMutex-SeInheritance wild-SP bug.
    //
    // The superseded version chained backward off the previously-decoded save
    // (last_reg-2, last_off-16). That coincides with the truth whenever the
    // preceding code is itself part of the same consecutive run — which is the
    // common shape, and why it survived two rounds of review — but it breaks
    // the moment a NON-adjacent single save sits between. Real counterexample,
    // ntoskrnl 0x4014f0: `str x28,[sp,#0x140]` then save_next; chaining back
    // from x28@320 yields x26,x27@304, but the prologue actually stores
    // x21,x22 there. That mis-restore is silent — wrong callee-saved values
    // flow into the resumed context and surface later as a garbage indirect
    // branch. It was wrong at 1268 of 3945 save_next sites (32%) across
    // ntoskrnl+ntdll. `tools/verify_arm64_unwind.py` checks every save site in
    // a built ARM64 PE against the disassembled prologue; keep it green. §9.113
    //
    // Running SP cursor for the *offset-based* saves is just Context.Sp; the
    // pre-indexed / alloc codes mutate Context.Sp as they go.
    let mut guard = 0usize;
    while i < codes_len {
        guard += 1;
        if guard > 4096 {
            return false; // runaway — malformed stream
        }
        let b0 = *codes.add(i);
        let sp = (*Context).Sp;
        match b0 {
            // alloc_s: 000xxxxx — sp += x*16
            0x00..=0x1F => {
                (*Context).Sp = sp + ((b0 & 0x1F) as ULONG64) * 16;
            }
            // save_r19r20_x: 001zzzzz — load x19,x20 from [sp]; sp += z*8
            0x20..=0x3F => {
                let z = (b0 & 0x1F) as ULONG64;
                restore_int_pair(Context, Nv, 19, sp);
                (*Context).Sp = sp + z * 8;
            }
            // save_fplr: 01zzzzzz — load x29,lr from [sp + z*8]
            0x40..=0x7F => {
                let z = (b0 & 0x3F) as ULONG64;
                restore_int_pair(Context, Nv, 29, sp + z * 8);
            }
            // save_fplr_x: 10zzzzzz — load x29,lr from [sp]; sp += (z+1)*8
            0x80..=0xBF => {
                let z = (b0 & 0x3F) as ULONG64;
                restore_int_pair(Context, Nv, 29, sp);
                (*Context).Sp = sp + (z + 1) * 8;
            }
            // alloc_m: 11000xxx xxxxxxxx — sp += x*16
            0xC0..=0xC7 => {
                if i + 1 >= codes_len {
                    return false;
                }
                let x = (((b0 & 0x07) as ULONG64) << 8) | (*codes.add(i + 1) as ULONG64);
                (*Context).Sp = sp + x * 16;
            }
            // 110010xx..110111.. two-byte reg saves
            0xC8..=0xDF => {
                if b0 == 0xDF {
                    // alloc_z: 11011111 zzzzzzzz — sp += z * SVE-VL (no SVE on our
                    // hardware; treat VL as 16 for a best-effort SP recovery).
                    if i + 1 >= codes_len {
                        return false;
                    }
                    let z = *codes.add(i + 1) as ULONG64;
                    (*Context).Sp = sp + z * 16;
                } else if b0 == 0xDE {
                    // save_freg_x: 11011110 xxxzzzzz — d(8+x) at [sp-(z+1)*8]!
                    if i + 1 >= codes_len {
                        return false;
                    }
                    let b1 = *codes.add(i + 1);
                    let x = (b1 >> 5) as usize;
                    let z = (b1 & 0x1F) as ULONG64;
                    restore_fp_single(Context, Nv, 8 + x, sp);
                    (*Context).Sp = sp + (z + 1) * 8;
                } else {
                    if i + 1 >= codes_len {
                        return false;
                    }
                    let b1 = *codes.add(i + 1);
                    match b0 & 0xFE {
                        // save_regp: 110010xx xxzzzzzz
                        0xC8 | 0xCA => {
                            let x = ((((b0 & 0x03) as usize) << 2) | (b1 >> 6) as usize) & 0x0F;
                            let z = (b1 & 0x3F) as ULONG64;
                            restore_int_pair(Context, Nv, 19 + x, sp + z * 8);
                        }
                        // save_regp_x: 110011xx xxzzzzzz
                        0xCC | 0xCE => {
                            let x = ((((b0 & 0x03) as usize) << 2) | (b1 >> 6) as usize) & 0x0F;
                            let z = (b1 & 0x3F) as ULONG64;
                            restore_int_pair(Context, Nv, 19 + x, sp);
                            (*Context).Sp = sp + (z + 1) * 8;
                        }
                        // save_reg: 110100xx xxzzzzzz
                        0xD0 | 0xD2 => {
                            let x = ((((b0 & 0x03) as usize) << 2) | (b1 >> 6) as usize) & 0x0F;
                            let z = (b1 & 0x3F) as ULONG64;
                            restore_int_single(Context, Nv, 19 + x, sp + z * 8);
                        }
                        // save_reg_x: 1101010x xxxzzzzz  (4-bit reg: 1 bit in b0, 3 in
                        // b1>>5; 5-bit z. Was (<<2 | >>6) — dropped a reg bit for x>=4.)
                        0xD4 => {
                            let x = (((b0 & 0x01) << 3) | (b1 >> 5)) as usize;
                            let z = (b1 & 0x1F) as ULONG64;
                            restore_int_single(Context, Nv, 19 + x, sp);
                            (*Context).Sp = sp + (z + 1) * 8;
                        }
                        // save_lrpair: 1101011x xxzzzzzz
                        0xD6 => {
                            let x = (((b0 & 0x01) << 2) | (b1 >> 6)) as usize;
                            let z = (b1 & 0x3F) as ULONG64;
                            restore_int_single(Context, Nv, 19 + 2 * x, sp + z * 8);
                            restore_int_single(Context, Nv, 30, sp + z * 8 + 8);
                        }
                        // save_fregp: 1101100x xxzzzzzz
                        0xD8 => {
                            let x = (((b0 & 0x01) << 2) | (b1 >> 6)) as usize;
                            let z = (b1 & 0x3F) as ULONG64;
                            restore_fp_pair(Context, Nv, 8 + x, sp + z * 8);
                        }
                        // save_fregp_x: 1101101x xxzzzzzz
                        0xDA => {
                            let x = (((b0 & 0x01) << 2) | (b1 >> 6)) as usize;
                            let z = (b1 & 0x3F) as ULONG64;
                            restore_fp_pair(Context, Nv, 8 + x, sp);
                            (*Context).Sp = sp + (z + 1) * 8;
                        }
                        // save_freg: 1101110x xxzzzzzz
                        0xDC => {
                            let x = (((b0 & 0x01) << 2) | (b1 >> 6)) as usize;
                            let z = (b1 & 0x3F) as ULONG64;
                            restore_fp_single(Context, Nv, 8 + x, sp + z * 8);
                        }
                        _ => {}
                    }
                }
            }
            // alloc_l: 11100000 xxxxxxxx xxxxxxxx xxxxxxxx — sp += x*16
            0xE0 => {
                if i + 3 >= codes_len {
                    return false;
                }
                let x = ((*codes.add(i + 1) as ULONG64) << 16)
                    | ((*codes.add(i + 2) as ULONG64) << 8)
                    | (*codes.add(i + 3) as ULONG64);
                (*Context).Sp = sp + x * 16;
            }
            // set_fp: mov x29,sp → unwind: sp = x29
            0xE1 => {
                (*Context).Sp = *ctx_x(Context).add(29);
            }
            // add_fp: add x29,sp,#x*8 → unwind: sp = x29 - x*8
            0xE2 => {
                if i + 1 >= codes_len {
                    return false;
                }
                let x = *codes.add(i + 1) as ULONG64;
                (*Context).Sp = *ctx_x(Context).add(29) - x * 8;
            }
            // nop — home-register store; nothing to restore
            0xE3 => {}
            // end / end_c — terminate
            0xE4 | 0xE5 => {
                return true;
            }
            // save_next (0xE6) — the base pair is later in the unwind stream,
            // because codes are stored in reverse-prologue order. Each preceding
            // save_next restores the numerically next pair at the next stack slot.
            // It is offset-based and therefore does not change SP.
            0xE6 => {
                let mut run_end = i;
                while run_end < codes_len && *codes.add(run_end) == 0xE6 {
                    run_end += 1;
                }
                let ordinal = run_end - i;
                if let Some((first, address, is_fp)) =
                    save_next_target(codes, codes_len, run_end, ordinal, sp)
                {
                    if is_fp {
                        restore_fp_pair(Context, Nv, first, address);
                    } else {
                        restore_int_pair(Context, Nv, first, address);
                    }
                }
            }
            // save_any_reg: 3-byte general save — decode enough to restore + SP.
            0xE7 => {
                if i + 2 >= codes_len {
                    return false;
                }
                let b1 = *codes.add(i + 1);
                let b2 = *codes.add(i + 2);
                let p = (b1 >> 6) & 1; // pair
                let xneg = (b1 >> 5) & 1; // negative pre-indexed
                let r = (b1 & 0x1F) as usize;
                let mode = (b2 >> 6) & 0x03; // 00=X,01=D,10=Q
                let o = (b2 & 0x3F) as ULONG64;
                let scale: ULONG64 = if xneg == 1 || p == 1 || mode == 2 { 16 } else { 8 };
                let addr = if xneg == 1 { sp } else { sp + o * scale };
                match mode {
                    0 => {
                        restore_int_single(Context, Nv, r, addr);
                        if p == 1 {
                            restore_int_single(Context, Nv, r + 1, addr + 8);
                        }
                    }
                    1 => {
                        restore_fp_single(Context, Nv, r, addr);
                        if p == 1 {
                            restore_fp_single(Context, Nv, r + 1, addr + 8);
                        }
                    }
                    _ => {
                        // Q/Z/P regs: restore low 64 bits best-effort.
                        restore_fp_single(Context, Nv, r, addr);
                    }
                }
                if xneg == 1 {
                    let count: ULONG64 = if p == 1 { 2 } else { 1 };
                    (*Context).Sp = sp + o * scale * count.max(1);
                }
            }
            // MSFT_OP_* custom stack frames (trap/machine/context) — used only in
            // hand-written asm. Restoring their full state needs the frame layout;
            // for unwind-to-caller we stop and let the caller treat it as a leaf.
            0xE8..=0xEC => {
                return true;
            }
            // pac_sign_lr — strip the PAC (no pointer-auth on our targets: no-op)
            0xFC => {}
            _ => {
                // reserved / unhandled — skip by its declared length
            }
        }
        i += unwind_code_len(b0);
    }
    false
}

/// Full `.xdata` unwind for one frame. Mutates `Context` to the caller's state
/// and returns the language handler routine (or None) plus the handler data
/// pointer written through `HandlerData`.
///
/// `ControlPcRva` is the ControlPc offset from the image base.
pub(crate) unsafe fn virtual_unwind_xdata(
    HandlerType: ULONG,
    ImageBase: ULONG64,
    ControlPc: ULONG64,
    FunctionEntry: PRUNTIME_FUNCTION,
    Context: PCONTEXT,
    HandlerData: *mut PVOID,
    EstablisherFrame: *mut ULONG64,
    Nv: PKNONVOLATILE_CONTEXT_POINTERS,
) -> PEXCEPTION_ROUTINE {
    let func_start = ImageBase + (*FunctionEntry).BeginAddress as ULONG64;
    let pc_off_from_start = ControlPc.wrapping_sub(func_start);

    let xdata = (ImageBase + (*FunctionEntry).__bindgen_anon_1.UnwindData as ULONG64) as *const u32;
    let hdr0 = core::ptr::read_unaligned(xdata);

    let func_len_words = hdr0 & 0x3FFFF; // FunctionLength / 4
    let has_handler = (hdr0 >> 20) & 1; // X
    let single_epilog = (hdr0 >> 21) & 1; // E
    let mut epilog_count = (hdr0 >> 22) & 0x1F;
    let mut code_words = (hdr0 >> 27) & 0x1F;

    // Establisher frame: for a function that establishes a frame pointer
    // (set_fp / add_fp in its prolog), the EstablisherFrame handed to language
    // handlers and __finally funclets is the FRAME-POINTER value (x29), NOT the
    // raw SP — clang keys funclet access to the parent's locals off x29 (the
    // funclet does `mov x29, x1; str .., [x29 + off]`), and TestFinally stores
    // `ran_finally` at `x29-4`. Passing SP (which is x29 minus the add_fp offset)
    // made the finally funclet write to the wrong slot — the "__finally never
    // runs" bug (it DID run, into a dead stack word). Mirrors Wine's amd64
    // RtlVirtualUnwind (frame_reg ? frame_reg_value : Rsp). Capture x29 BEFORE
    // interpret_codes restores the caller's x29 via save_fplr; pick below once we
    // know whether the prolog set up a frame pointer. Frameless functions keep SP.
    let sp_in = (*Context).Sp;
    let x29_in = *ctx_x(Context).add(29);
    let lr_in = *ctx_x(Context).add(30);
    *EstablisherFrame = sp_in;

    // Optional extension header word when both counts are zero.
    let mut word_cursor = 1usize;
    if epilog_count == 0 && code_words == 0 {
        let ext = core::ptr::read_unaligned(xdata.add(1));
        epilog_count = ext & 0xFFFF;
        code_words = (ext >> 16) & 0xFF;
        word_cursor = 2;
    }

    // Epilog scope words (present only when E == 0).
    let epilog_scope_words = if single_epilog != 0 { 0 } else { epilog_count };
    let epilog_scopes_at = word_cursor;
    word_cursor += epilog_scope_words as usize;

    // Unwind code bytes start here; there are `code_words` 32-bit words.
    let codes = xdata.add(word_cursor) as *const u8;
    let codes_len = (code_words as usize) * 4;

    // Determine the start index into the code stream:
    //  - body unwind (default): index 0
    //  - E==1 single epilog: header's EpilogCount field is the start index, but
    //    only if PC is within that epilog. We approximate epilog membership by
    //    "PC in the last `prolog-mirror` region"; without exact epilog offset in
    //    the E==1 header we conservatively use body unwind unless PC is clearly
    //    past function end minus an epilog. For correctness on the common case
    //    (fault in body), index 0 is right.
    //  - E==0 with scopes: find a scope whose [start, start+len) contains PC.
    let mut start_index = 0usize;
    let func_len_bytes = (func_len_words as ULONG64) * 4;

    if single_epilog != 0 {
        // E==1: the 5-bit EpilogCount field holds the first unwind-code *index*
        // of the sole epilog. We only jump into it when PC is actually inside
        // the epilog. The epilog's start offset isn't in the header, so we treat
        // any PC at/after (func_end - epilog_code_count*4) as in-epilog. This is
        // a conservative heuristic; body faults (the FEX case) take index 0.
        let epilog_start_index = ((hdr0 >> 22) & 0x1F) as usize;
        // Rough epilog byte length = number of codes from epilog_start_index to
        // the terminating end. Compute it by walking.
        let mut walk = epilog_start_index;
        let mut epi_instr = 0usize;
        while walk < codes_len {
            let b0 = *codes.add(walk);
            epi_instr += 1;
            if b0 == 0xE4 || b0 == 0xE5 {
                break;
            }
            walk += unwind_code_len(b0);
        }
        let epi_bytes = (epi_instr as ULONG64) * 4;
        let epi_start_off = func_len_bytes - epi_bytes;
        if func_len_bytes >= epi_bytes && pc_off_from_start >= epi_start_off {
            // Codes already "executed" = epilog instructions between the epilog's
            // start and the fault PC, i.e. (pc - epi_start)/4 — NOT (func_end -
            // pc)/4, which counts from the wrong end and, when PC sits exactly at
            // the epilog's first instruction (a `bl` immediately followed by the
            // frame teardown — e.g. kernel32!RaiseException calling ntdll!Rtl-
            // RaiseException), skipped the entire epilog to land on the trailing
            // `end`, yielding a no-op unwind that never restored LR. That spun the
            // dispatcher 200× and dropped every cross-module __except. This mirrors
            // the E==0 scope branch below, which already uses the correct delta.
            let into = ((pc_off_from_start - epi_start_off) / 4) as usize;
            // Skip `into` codes forward from epilog_start_index.
            start_index = skip_codes(codes, codes_len, epilog_start_index, into);
        }
    } else if epilog_scope_words > 0 {
        for s in 0..epilog_scope_words as usize {
            let scope = core::ptr::read_unaligned(xdata.add(epilog_scopes_at + s));
            let epi_start_off = (scope & 0x3FFFF) as ULONG64 * 4;
            let epi_start_index = ((scope >> 22) & 0x3FF) as usize;
            // Membership: PC within [epi_start_off, epi_start_off + epilog_len).
            let mut walk = epi_start_index;
            let mut epi_instr = 0usize;
            while walk < codes_len {
                let b0 = *codes.add(walk);
                epi_instr += 1;
                if b0 == 0xE4 || b0 == 0xE5 {
                    break;
                }
                walk += unwind_code_len(b0);
            }
            let epi_bytes = (epi_instr as ULONG64) * 4;
            if pc_off_from_start >= epi_start_off && pc_off_from_start < epi_start_off + epi_bytes {
                let into = ((pc_off_from_start - epi_start_off) / 4) as usize;
                start_index = skip_codes(codes, codes_len, epi_start_index, into);
                break;
            }
        }
    }

    // Does this prolog establish a frame pointer? Scan the body unwind codes
    // (index 0..) for set_fp (0xE1) or add_fp (0xE2). If so, the EstablisherFrame
    // is x29 (captured pre-interpret), not SP — see the note at *EstablisherFrame.
    let mut has_fp = false;
    {
        let mut i = 0usize;
        while i < codes_len {
            let b0 = *codes.add(i);
            if b0 == 0xE1 || b0 == 0xE2 {
                has_fp = true;
                break;
            }
            if b0 == 0xE4 || b0 == 0xE5 {
                break;
            }
            i += unwind_code_len(b0);
        }
    }
    if has_fp {
        *EstablisherFrame = x29_in;
    }

    // Interpret. On failure (malformed) fall back to a leaf unwind so we still
    // make forward progress rather than looping.
    let ok = interpret_codes(Context, Nv, codes, codes_len, start_index);
    if !ok {
        // Malformed / truncated code stream: interpret_codes may have left
        // Context.Sp and the callee-saved regs HALF-updated (a wild SP is the
        // §9.69 vector-re-entry wedge). Reset to a clean LEAF unwind — caller SP =
        // entry SP, return address = entry LR — so the dispatcher makes safe
        // forward progress instead of transferring control to a garbage SP.
        (*Context).Sp = sp_in;
        set_x(Context, 30, lr_in);
    }

    // Recovered return address: Pc <- Lr (X[30]).
    (*Context).Pc = *ctx_x(Context).add(30);

    // Handler extraction: after the code words (padded to word boundary), if
    // X==1, a single Exception Handler RVA followed by handler-specific data.
    *HandlerData = core::ptr::null_mut();
    if has_handler != 0 && (HandlerType & (UNW_FLAG_EHANDLER | UNW_FLAG_UHANDLER)) != 0 {
        let handler_word = word_cursor + code_words as usize;
        let handler_rva = core::ptr::read_unaligned(xdata.add(handler_word));
        let routine = (ImageBase + handler_rva as ULONG64) as PVOID;
        *HandlerData = xdata.add(handler_word + 1) as PVOID;
        return core::mem::transmute::<PVOID, PEXCEPTION_ROUTINE>(routine);
    }

    None
}

/// Skip `count` unwind codes forward from `start`, returning the new index
/// (clamped to `codes_len`). Used for mid-epilog partial unwinds.
fn skip_codes(codes: *const u8, codes_len: usize, start: usize, count: usize) -> usize {
    let mut i = start;
    let mut n = 0usize;
    while i < codes_len && n < count {
        let b0 = unsafe { *codes.add(i) };
        i += unwind_code_len(b0);
        n += 1;
    }
    i
}
