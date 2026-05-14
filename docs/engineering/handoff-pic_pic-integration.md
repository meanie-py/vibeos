# Handoff: integrate `hal-rust/src/x86/pic_pic.rs` on i386 without breaking link or boot

## TL;DR

`sdk/lib/rust/hal-rust/src/x86/pic_pic.rs` is a 1175-LOC i386 Rust translation
of `hal/halx86/pic/pic.c` (1300 LOC).  After ~100 cascade fixes (100 → 0 rustc
errors) the crate compiles cleanly with `ninja rust_hal`.  The file is
**checked in but parked** — `pub mod pic_pic;` is commented out in
`sdk/lib/rust/hal-rust/src/x86/mod.rs` and the file is NOT in
`hal/halx86/CMakeLists.txt` DEPENDS.  i386 smoke 7/7 PASS in 50s
unchanged.

The cleanup checkpoint commit replaces the historical rustconv draft
defects:
- Cascade roots fixed: bindgen union access (`__bindgen_anon_1.X` for
  `_PIC_MASK`, `_EISA_ELCR`, `_I8259_OCW2/3/ISR`); `Some(fn)` wrap for
  Option-typed handler tables; `Vector` widening for u8/u32 mismatches;
  table sizes corrected (`SWInterruptHandlerTable[20]`, not `[8]`;
  `SWInterruptHandlerTable2[3]`, not `[8]`; `SWInterruptLookUpTable[8]`,
  not `[4]`).
- IRQL primitive bodies re-translated from `pic.c`:309-455 (Sonnet had
  stubbed them — `KeGetCurrentIrql()` returned PASSIVE_LEVEL; `KfLowerIrql`
  was empty; `KfRaiseIrql` was identity).
- File-level scope: `#![cfg(target_arch = "x86")]` since `pic.c` is
  i386-only per `pic.cmake`.

**Your job:** finish the integration so `pic_pic.rs` replaces `pic.c` in the
i386 build with i386 smoke 7/7 still green.

## Why integration is blocked

`ninja hal` (linking the full hal.dll) fails with:

```
ld.lld: error: undefined symbol: _KiUserTrap
>>> referenced by libhal_rust.a(...)::__HalpApcInterruptHandler@4
ld.lld: error: undefined symbol: ... (KiEnterInterruptTrap implicitly via the
   ntkrnl_sys::ddk import — but KiEnterInterruptTrap is also FORCEINLINE)
```

The reason: `KiUserTrap` and `KiEnterInterruptTrap` are **`FORCEINLINE`** in
`ntoskrnl/include/internal/i386/trap_x.h` (lines 18 and 370 respectively):

```c
FORCEINLINE
BOOLEAN
KiUserTrap(IN PKTRAP_FRAME TrapFrame)
{
    return !!(TrapFrame->SegCs & MODE_MASK);
}

FORCEINLINE
VOID
KiEnterInterruptTrap(IN PKTRAP_FRAME TrapFrame)
{
    PVOID ExceptionList;

    ExceptionList = KeGetPcr()->NtTib.ExceptionList;
    ASSERTMSG(...);
    KeGetPcr()->NtTib.ExceptionList = EXCEPTION_CHAIN_END;
    TrapFrame->Dr7 = 0;
    if (KiUserTrap(TrapFrame) || (TrapFrame->EFlags & EFLAGS_V86_MASK)) {
        if (KeGetCurrentThread()->Header.DebugActive & 0xFF) {
            KiHandleDebugRegistersOnTrapEntry(TrapFrame);
        }
    }
    KiFillTrapFrameDebug(TrapFrame);
}
```

The C version of `pic.c` doesn't generate calls to these — they're inlined
by the C compiler.  Our Rust translation `use ntkrnl_sys::ddk::{KiUserTrap,
KiEnterInterruptTrap}` declares them as extern functions and emits real
calls.  At link time, `ld.lld` looks for the corresponding exports in
`ntoskrnl.dll` and finds nothing (they're not in `ntoskrnl/ntoskrnl.spec`).

There are also `~16K` worth of supporting symbols pulled in transitively
because `use ntkrnl_sys::ddk::*` brings the whole bindgen ddk module into
the HAL link unit.  `feedback_raw_dylib_cross_dll_data.md` documents the
established pattern for HAL-side ntoskrnl imports: declare them locally as
`#[link(name = "ntoskrnl", kind = "raw-dylib")]`.

## What needs to happen for integration

1. **Inline `KiUserTrap` in pic_pic.rs.**  One line: it returns
   `(*TrapFrame).SegCs & MODE_MASK != 0` cast to `BOOLEAN`.  `MODE_MASK = 1`
   on i386 (it's the RPL mask).  Replace every `KiUserTrap(TrapFrame)` call
   site with the inline expression.

2. **Inline `KiEnterInterruptTrap` in pic_pic.rs.**  This is harder — the
   body does:
   - `ExceptionList = KeGetPcr()->NtTib.ExceptionList;`
   - assert `ExceptionList == TrapFrame->ExceptionList` (DBG only, skip in
     release)
   - `KeGetPcr()->NtTib.ExceptionList = EXCEPTION_CHAIN_END;` (= -1u32)
   - `TrapFrame->Dr7 = 0;`
   - debug-register conditional (DBG only — skip; calls
     `KiHandleDebugRegistersOnTrapEntry` and `KiFillTrapFrameDebug` which
     are FORCEINLINE too in trap_x.h).

   Release build can skip the debug stuff.  The mandatory work is just
   `Pcr->NtTib.ExceptionList = EXCEPTION_CHAIN_END` + `TrapFrame->Dr7 = 0`.

3. **Switch from `use ntkrnl_sys::ddk::*` to local raw-dylib imports** for
   the few real ntoskrnl exports we need: `KeBugCheck`, `KeBugCheckEx`,
   `KiDeliverApc`, `KiDispatchInterrupt`, `KiEoiHelper`.  This stops the
   bindgen ddk CGU from being pulled into `libhal_rust.a` and fixes the
   transitive undefined-symbol cascade (`_KiDispatchExceptionFromTrapFrame@32`,
   `@KiServiceExit2@4`, `_KiSystemStartupBootStack@0`, `_HalVectorToIDTEntry@4`).

   Pattern to follow (already used elsewhere in hal-rust per
   `feedback_raw_dylib_cross_dll_data.md`):

   ```rust
   #[link(name = "ntoskrnl", kind = "raw-dylib")]
   unsafe extern "system" {
       fn KeBugCheck(BugCheckCode: ULONG);
       fn KeBugCheckEx(BugCheckCode: ULONG, P1: ULONG_PTR, P2: ULONG_PTR,
                       P3: ULONG_PTR, P4: ULONG_PTR);
       fn KiDeliverApc(PreviousMode: i8, ExceptionFrame: PVOID,
                       TrapFrame: PKTRAP_FRAME);
       fn KiDispatchInterrupt();
   }
   #[link(name = "ntoskrnl", kind = "raw-dylib")]
   unsafe extern "fastcall" {
       fn KiEoiHelper(TrapFrame: PKTRAP_FRAME);
   }
   ```

   Then drop the `use ntkrnl_sys::ddk::*` line entirely.  Remove the
   `core::mem::transmute` calls used to bridge the `ddk::_KTRAP_FRAME` vs
   `hal_ddk::_KTRAP_FRAME` mismatch — once we use local extern with
   `hal_ddk::PKTRAP_FRAME`, no transmute is needed.

4. **Port `HalpHardwareInterrupt[0-15]` to Rust** (or arrange for them to
   stay C-side via a partial pic.c removal).

   In `hal/halx86/pic/pic.c:158-205`, the macro `HalpDelayedHardwareInterrupt(N)`
   expands 16 times to define `HalpHardwareInterrupt0` through `Interrupt15`.
   These are the per-IRQ pending-interrupt stubs that go into
   `SWInterruptHandlerTable[4..20]`.

   Each one is a small wrapper.  Find the macro definition (probably in
   `hal/halx86/include/halhw.h` or `pic.c` itself) and either:
   - **(a)** translate the macro expansion into a Rust `macro_rules!` and
     emit the 16 stubs,
   - **(b)** keep `pic.c` building but with most functions `#if 0`'d out,
     leaving only the `HalpDelayedHardwareInterrupt(N)` expansions and the
     SWInterruptHandlerTable / SWInterruptHandlerTable2 / HalpEisaELCR /
     HalpDefaultInterruptAffinity statics + the macro expansions live there.
     This is the smaller-blast-radius option.

5. **Move static table definitions to Rust if you went with (a).**  When
   pic.c is fully removed, these defs need to come from somewhere:
   - `SWInterruptHandlerTable[20]` (pic.c:182)
   - `SWInterruptHandlerTable2[3]` (pic.c:207)
   - `HalpEisaELCR` (pic.c:214)
   - `HalpDefaultInterruptAffinity` (pic.c, search)
   - `HalpSpecialDismissTable` (pic.c, search)
   - `HalpSpecialDismissLevelTable` (pic.c, search)

   Currently pic_pic.rs declares all of these as `extern` (in the top
   `unsafe extern "C" { ... }` block at lines 71-79).  When pic.c goes
   away, change those `extern` decls to `#[no_mangle] pub static mut ...`
   definitions.  Match the C initializer values exactly (line numbers
   above).

6. **Remove pic.c from `hal/halx86/pic.cmake`** by commenting the
   `pic/pic.c` line the same way the other replaced files are commented out
   (see how `pic/halinit.c` etc. are commented).  Add pic_pic.rs to
   `hal/halx86/CMakeLists.txt` DEPENDS.  Uncomment `pub mod pic_pic;` in
   `sdk/lib/rust/hal-rust/src/x86/mod.rs`.

7. **Build and smoke.** `ninja livecd` from build-i386, then
   `pytest tests/qemu/test_smoke.py --arch i386 --skip-clean-build` should
   be 7/7 PASS.

## Files of interest

| Path | What's in it |
|---|---|
| `sdk/lib/rust/hal-rust/src/x86/pic_pic.rs` | the Rust translation (1175 LOC, 0 errors) |
| `hal/halx86/pic/pic.c` | the C original (1300 LOC) |
| `hal/halx86/pic/pic.S` | trap stubs that wrap pic.c functions; defines `HalpDispatchInterrupt`/`HalpHardwareInterruptLevel`/`HalpEndSoftwareInterrupt`/`HalEndSystemInterrupt` (the `_2`-suffix wrappers).  These stay C-side. |
| `ntoskrnl/include/internal/i386/trap_x.h` | FORCEINLINE bodies of `KiUserTrap` (line 18) and `KiEnterInterruptTrap` (line 370) — copy these into Rust |
| `sdk/lib/rust/hal-rust/src/x86/irql.rs` | already provides `KiI8259MaskTable` and `FindHigherIrqlMask` static tables — pic_pic.rs externs them correctly |
| `sdk/lib/rust/hal-rust/src/x86/mod.rs` | toggle line: `// pub mod pic_pic;` (after `pic_halinit;`) — uncomment when integrating |
| `hal/halx86/CMakeLists.txt` | DEPENDS list — add pic_pic.rs alphabetically between `pic_halinit.rs` and `portio.rs` |
| `hal/halx86/pic.cmake` | comment out `pic/pic.c` when integrating |

## Smoke baseline

```
tests/qemu/test_smoke.py::test_boot_and_serial PASSED      [ 14%]
tests/qemu/test_smoke.py::test_milestones PASSED           [ 28%]
tests/qemu/test_smoke.py::test_qmp_status PASSED           [ 42%]
tests/qemu/test_smoke.py::test_snapshot_restore PASSED     [ 57%]
tests/qemu/test_smoke.py::test_screenshot PASSED           [ 71%]
tests/qemu/test_smoke.py::test_no_unexpected_ud2 PASSED    [ 85%]
tests/qemu/test_smoke.py::test_sacred_primitives PASSED    [100%]
============================== 7 passed in 50.81s ==============================
```

## Rules of engagement

- **Do not revert `pic_pic.rs` to `.c`** to make the link succeed.
- **Do not delete `pic_pic.rs`** — API translation cost real money.
- **Do not regenerate the rustconv translation** without asking the user.
- **Do not weaken smoke tests.**

## Methodology lesson (for future you)

The four cascade-root patterns that ate ~95 of the 100 errors here:

1. **Bindgen `__bindgen_anon_1` wrappers** — When `union { struct {...}; T x; }`
   hits bindgen, the struct half becomes `__bindgen_anon_1: _Foo__bindgen_ty_1`
   and `T x` stays as a field.  Setters on the bitfield struct need to be
   reached through `__bindgen_anon_1.set_X(...)`; the `T x` field is
   accessed directly.  rustconv consistently emits the wrong access pattern.

2. **Option-typed handler tables** — `[Option<unsafe extern "C" fn()>; N]`
   needs `Some(fn)` to assign a function item.  rustconv emits bare `fn`.

3. **Local `unsafe extern { ... }` blocks shadow types** — when the same
   extern static is declared with a different type (e.g., `[PVOID; 16]`)
   inside a function and (e.g., `[Option<fn>; 20]`) at the top level, the
   local wins for the function body and creates type mismatches.  Strip
   ALL function-local extern blocks and consolidate at the top.

4. **Mixed `UCHAR`/`ULONG` vector arithmetic** — C does implicit int
   promotion in `Vector - PRIMARY_VECTOR_BASE`.  Rust does not.  The
   safest fix is to widen the constants to `ULONG` and use `wrapping_sub`
   with explicit casts at the boundaries.

## Quick reference

```bash
# Build hal-rust crate alone (validates pic_pic.rs compile state)
ninja -C build-i386 rust_hal

# Try linking hal.dll (currently fails until KiUserTrap inlined)
ninja -C build-i386 hal

# Inspect what's in the rust .a
i686-w64-mingw32-nm build-i386/hal/halx86/libhal_rust.a 2>/dev/null \
    | grep -E "pic_pic|KiUser|KiEnter"

# Smoke
source .venv/bin/activate
pytest tests/qemu/test_smoke.py -v --arch i386 --timeout 300 --skip-clean-build
```
