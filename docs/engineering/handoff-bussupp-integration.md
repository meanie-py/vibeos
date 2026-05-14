# Handoff: integrate `hal-rust/src/x86/bussupp.rs` on i386 without breaking boot

## TL;DR

`sdk/lib/rust/hal-rust/src/x86/bussupp.rs` is a 1528-LOC i386-only Rust
translation of `hal/halx86/legacy/bussupp.c` (1701 LOC). It compiles cleanly
(0 rustc errors, no clippy warnings of note). The file is **committed but
disabled**: it is not listed in `hal/halx86/CMakeLists.txt` DEPENDS and the
module decl is commented out in `sdk/lib/rust/hal-rust/src/x86/mod.rs`.

The compile-clean checkpoint is commit `06b86140`. The cascade fix journey
(184 → 0 errors) is documented in that commit message.

**Your job:** enable `bussupp.rs` and remove `legacy/bussupp.c` from the C
build without breaking the i386 LiveCD boot. The i386 smoke test must pass
7/7 with the Rust file taking over. amd64 and arm64 must continue to pass
(bussupp.rs is `#![cfg(target_arch = "x86")]`, so they should be unaffected
— but verify).

## The failure mode

When `bussupp.rs` is enabled (uncommented in `mod.rs` + added to CMakeLists
DEPENDS), and `legacy/bussupp.c` is removed (or `--allow-multiple-definition`
lets the linker pick winners), **i386 boot wedges before the first debugcon
trace marker fires**. The smoke run times out at 90s with no serial output
beyond freeldr.

Concretely the last smoke run with bussupp.rs integrated returned:

```
ERROR tests/qemu/test_smoke.py::test_boot_and_serial - Failed: Timeout (>90.0s) from pytest-timeout.
[pytest] WARNING: ud2 scanner: 2 LLVM miscompilations detected:
  _MmPageOutPhysicalAddress@4, _PopSetSystemPowerState@8
```

(Those two ud2s are pre-existing and unrelated — they fire in the no-bussupp
baseline too. The smoke harness reports them as warnings, not failures.)

The current `mod.rs` comment captures the suspicion succinctly:

```
// pub mod bussupp; — compile-clean but causes i386 boot regression when
// cohabiting with C bussupp.c via --allow-multiple-definition (linker picks
// C side for duplicates, but something in the Rust side still gets pulled
// in and breaks boot before debugcon emits any markers).
```

## Repro

The 1-line toggle that breaks boot:

1. Edit `sdk/lib/rust/hal-rust/src/x86/mod.rs` — uncomment `pub mod bussupp;`
   (currently line 17, inside a `// pub mod bussupp; — ...` comment block).
2. Edit `hal/halx86/CMakeLists.txt` — add bussupp.rs to the
   `add_custom_command(... DEPENDS ...)` list around line 53 (alphabetically:
   between `bushndlr.rs` and `clock.rs`).
3. Edit `hal/halx86/legacy.cmake` — comment out `legacy/bussupp.c` on line 10
   the same way bushndlr.c / cmosbus.c / isabus.c are commented out (mark it
   with `# legacy/bussupp.c — replaced by Rust ...`).
4. Build + smoke:

```bash
source .venv/bin/activate
cd build-i386 && cmake . && ninja livecd && cd ..
pytest tests/qemu/test_smoke.py -v --arch i386 --timeout 300 --skip-clean-build
```

Expect the timeout listed above.

## What's known

### bussupp.rs surface (Rust exports)

`grep -n '^pub static\|^#\[no_mangle\]\|^#\[unsafe(no_mangle)\]' sdk/lib/rust/hal-rust/src/x86/bussupp.rs`

Statics:
- `HalpPciIrqMask: ULONG` — read+written by `HalpInitializePciBus`; defined
  in C as plain `ULONG HalpPciIrqMask;` at `hal/halx86/legacy/bussupp.c:104`
- `WarningsGiven: [BOOLEAN; 5]` — defined in C as
  `static BOOLEAN WarningsGiven[5];` at `bussupp.c:425` (note: `static` in
  C, but the Rust translation exports it as `#[no_mangle] pub static mut`
  — that's a behavioral change worth examining)

Functions (all `extern "system"` = stdcall on i386):
- `HalpAllocateBusHandler`, `HalpRegisterInternalBusHandlers`,
  `HalpMarkChipsetDecode`, `HalpAllocateAndInitPciBusHandler`,
  `HalpIsValidPCIDevice`, `HalpGetChipHacks`, `HalpIsRecognizedCard`,
  `HalpIsIdeDevice`, `HalpIsBridgeDevice`, `HalpGetPciBridgeConfig`,
  `HalpFixupPciSupportedRanges`, `ShowSize`, `HalpDebugPciDumpBus`
  (body stubbed — sprintf variadic), `HalpInitializePciBus`,
  `HalpInitBusHandlers`, `HalpRegisterKdSupportFunctions`,
  `HalpAssignSlotResources`, `HaliFindBusAddressTranslation`,
  `HaliTranslateBusAddress`, `HalAdjustResourceList`,
  `HalAssignSlotResources`, `HalGetBusData`, `HalGetBusDataByOffset`,
  `HalGetInterruptVector`, `HalSetBusData`, `HalSetBusDataByOffset`,
  `HalTranslateBusAddress`.

### Entry points (call edges from C-side ntoskrnl/HAL)

- `HalpInitBusHandlers` — called from `hal/halx86/generic/halinit.c:164`
  inside HalInitSystem (Phase 0 boot).
- `HalpInitializePciBus` — called from `hal/halx86/legacy/halpcat.c:97`
  (HalpInitNonBusHandler) during HalInitSystem Phase 1.
- Public Hal* exports (`HalGetBusData`, `HalGetInterruptVector`,
  `HalTranslateBusAddress`, etc.) — referenced via hal.spec/hal.def,
  consumed by ntoskrnl + drivers.

The integration question: **which of these is the first one called, and
does our Rust version do the right thing at that call?** Boot wedges
*before* the first debugcon trace, which means somewhere in the kernel-
entry → HalInitSystem → HalpInitBusHandlers path on i386.

### Smoke baseline (bussupp.rs disabled — currently committed state)

```
tests/qemu/test_smoke.py::test_boot_and_serial PASSED                    [ 14%]
tests/qemu/test_smoke.py::test_milestones PASSED                         [ 28%]
tests/qemu/test_smoke.py::test_qmp_status PASSED                         [ 42%]
tests/qemu/test_smoke.py::test_snapshot_restore PASSED                   [ 57%]
tests/qemu/test_smoke.py::test_screenshot PASSED                         [ 71%]
tests/qemu/test_smoke.py::test_no_unexpected_ud2 PASSED                  [ 85%]
tests/qemu/test_smoke.py::test_sacred_primitives PASSED                  [100%]
============================== 7 passed in 52.31s ==============================
```

### Files of interest

| Path | What's in it |
|---|---|
| `sdk/lib/rust/hal-rust/src/x86/bussupp.rs` | the Rust translation (1528 LOC, 0 errors) |
| `hal/halx86/legacy/bussupp.c` | the C original (1701 LOC) |
| `sdk/lib/rust/hal-rust/src/x86/mod.rs` | toggle line: `// pub mod bussupp;` (line 17) |
| `hal/halx86/CMakeLists.txt` | DEPENDS list to add bussupp.rs to (line ~53) |
| `hal/halx86/legacy.cmake` | comment out `legacy/bussupp.c` (line 10) |
| `sdk/lib/rust/hal-rust/src/x86/bushndlr.rs` | sibling — already integrated, provides `HalpAllBusHandlers`, `HalpInitBusHandler`, etc. that bussupp.rs imports |
| `sdk/lib/rust/hal-rust/src/x86/legacy_halpnpdd.rs` | sibling — already integrated |
| `hal/halx86/include/bus.h` | C decls for HalpReadPCIConfig / HalpWritePCIConfig / HalpQueryPciRegistryInfo etc. — used by Rust extern blocks |
| `hal/halx86/include/halp.h` | HalpInitBusHandlers decl (line 536) |

## Suspected root causes (ranked)

### 1. `WarningsGiven` storage class regression (most likely)

C: `static BOOLEAN WarningsGiven[5];` (file-local at `bussupp.c:425`).
Rust: `#[unsafe(no_mangle)] pub static mut WarningsGiven: [BOOLEAN; 5]` at
`bussupp.rs:80`.

If anything else in the C tree happens to have a *different* `WarningsGiven`
symbol — or if our `no_mangle` makes the static collide with something —
the linker (with `--allow-multiple-definition`) could pick the wrong one and
silently corrupt unrelated memory. **Fix:** drop `#[no_mangle]` on
`WarningsGiven` and let it stay crate-private. The C original is
file-static; nothing outside bussupp.c uses it.

Check: `nm` the hal.dll baseline (C-only build, bussupp.c in) for
`WarningsGiven`. If absent (because static in C), and the Rust version
exports it, that's a new symbol in hal.dll's export table — investigate.

### 2. `HalpPciIrqMask` linker pick (very likely if both definitions live)

C: `ULONG HalpPciIrqMask;` (global, zero-init) at `bussupp.c:104`.
Rust: `#[unsafe(no_mangle)] pub static mut HalpPciIrqMask: ULONG = 0;`.

When both definitions exist (because legacy/bussupp.c isn't yet removed
from the build) and `--allow-multiple-definition` is in effect, the linker
picks one — likely the C version. The Rust code then reads/writes a
*different* zero-initialized symbol than the C HalpInitializePciBus path
which the kernel actually drives. **Fix:** make sure step 3 of the repro
(commenting `legacy/bussupp.c` out of `legacy.cmake`) actually happens.
If we *don't* remove the C file, the Rust functions never get called (the
C ones win at link time for duplicates *and* take their statics' addresses
internally) so bussupp.rs is effectively dead — except for whatever subset
*does* get pulled in by another path. Confirm with `objdump`:

```bash
i686-w64-mingw32-objdump -d build-i386/hal/halx86/hal.dll \
    | grep -E '^[0-9a-f]+ <_HalpInitBusHandlers' -A 20
```

If you see calls into bushndlr.rs symbols (`_rs_*` prefixes? or unmangled
names?) instead of pure C-style calls into `_HalpAllocateBusHandler@N`,
that tells you which side won.

### 3. CGU partitioning of `extern` callees (the io_rust_shims pattern)

The workspace `Cargo.toml` sets `codegen-units = 256, lto = false` precisely
to defeat LLVM's captures(none) deductions across CGUs. **But** if
`bussupp.rs` ends up in the *same* CGU as one of its callees and LLVM
proves a captures(none) violation, the entire success path of a function
becomes `ud2`. See `MEMORY.md` → `feedback_cgu1.md` and
`feedback_captures_none_reprobe.md`.

Check after build:

```bash
python3 tools/scan_ud2.py build-i386 2>&1 | grep -i bussupp
```

If anything from bussupp shows up as ud2'd, that's the smoking gun and the
fix is to scatter `unsafe { core::arch::asm!("", options(nostack, preserves_flags)) }`
or `core::hint::black_box(...)` around the suspect write paths.

### 4. Order-of-statics initialization

Rust statics are zero-init at PE load time same as C BSS — no constructor
calls. But if `bushndlr.rs` (`HalpAllBusHandlers` LIST_ENTRY) and
`bussupp.rs` both expect their statics zeroed before first use, and the
PE has them in different sections… unlikely but cheap to rule out by
inspecting the section assignment via `i686-w64-mingw32-objdump -h`.

### 5. PCI_COMMON_HEADER union/anon-field translation

The big risk in the translation work itself: bindgen split
`_PCI_COMMON_CONFIG` (opaque 192-byte buffer) from `_PCI_COMMON_HEADER`
(typed fields). The Rust file aliases them via
`_PCI_COMMON_HEADER as _PCI_COMMON_CONFIG` and uses
`_PCI_COMMON_HEADER__bindgen_ty_1` to reach `u.type0` / `u.type1` /
`u.type2`. **A field-offset slip here would corrupt PCI config-cycle
reads** silently — exactly the kind of bug that wedges the boot before
any tracing fires. Verify in particular:
- `(*PciData).HeaderType` — must be at offset 0x0E
- `(*PciData).u.type1.InterruptLine` — `bussupp.rs:1155` area
- `(*PciData).u.type0.SubsystemID` / `SubVendorID` — `bussupp.rs` HalpIsRecognizedCard
- `(*PciData).VendorID` / `DeviceID` — should be offsets 0/2

`ntkrnl-sys/src/size_check_i386.rs` has compile-time size assertions —
check there for `_PCI_COMMON_HEADER`.

## Investigation playbook

### Step 0 — establish baseline diffs

Two builds: one with bussupp.rs disabled (baseline, currently committed),
one with it enabled (the regression). Diff their hal.dll exports:

```bash
i686-w64-mingw32-objdump -p build-i386/hal/halx86/hal.dll | grep -E 'DLL Name|Ordinal|Name' > /tmp/hal-with-bussupp.txt
# (after reverting integration)
i686-w64-mingw32-objdump -p build-i386/hal/halx86/hal.dll > /tmp/hal-baseline.txt
diff /tmp/hal-baseline.txt /tmp/hal-with-bussupp.txt
```

Any symbol that *gained* an export in the bussupp.rs build is a candidate.

### Step 1 — try removing `#[no_mangle]` from `WarningsGiven` first

Cheapest possible test. The C original is file-static; the Rust version
shouldn't export it. Change `bussupp.rs:79–80` from:

```rust
#[unsafe(no_mangle)]
pub static mut WarningsGiven: [BOOLEAN; 5] = [0; 5];
```

to:

```rust
static mut WarningsGiven: [BOOLEAN; 5] = [0; 5];
```

Rebuild, run i386 smoke. If it passes, we're done with this hypothesis.

### Step 2 — bisect: enable bussupp.rs but mass-stub all but one fn

If step 1 doesn't fix it, replace every fn body in bussupp.rs with the
existing C equivalent's name as a thin forwarder. This requires bringing
back the C file *and* renaming the Rust functions to avoid the multiple-
definition. Too invasive — instead:

Disable individual `#[no_mangle]` decorations one by one (so the C
version wins via the spec/def) and binary-search which Rust function's
codegen alone is enough to wedge boot. Start with the public Hal* exports
since those are reached via hal.spec/hal.def name resolution and are the
most exposed to caller-side ABI assumptions.

### Step 3 — add debugcon markers inside Rust

`hal/halx86/include/halhw.h` (or equivalent) typically wraps the I/O port.
The simplest probe is to slap `WRITE_PORT_UCHAR((PUCHAR)0xE9, b'?')` at
the very top of `HalpInitBusHandlers` and `HalpRegisterInternalBusHandlers`
and watch debugcon.bin for the byte. If the byte appears, the issue is
*inside* bussupp's call graph; if it doesn't, the wedge is on the way
*to* bussupp (likely an earlier-loading static-init or PE-section corrupt).

`/tmp/pytest-of-curi/pytest-N/qemu-i3860/debugcon.bin` is where smoke
stores the dump.

### Step 4 — check ud2 scanner

```bash
python3 tools/scan_ud2.py build-i386
```

Look for any new ud2 in functions whose names match bussupp exports. Per
`feedback_ud2_abi_vs_captures.md`: ud2 + folded success path can be
captures(none) OR an ABI mismatch — check `nm` for `_Name@N` vs `_Name`
decoration on suspect symbols first (1-second check).

### Step 5 — ABI audit on the import list

Run `verify_abi.py` (if it has HAL coverage) or hand-check each `extern
"system"` / `extern "C"` / `extern "fastcall"` block in `bussupp.rs`
against the C declarations:

- `hal/halx86/include/bus.h` for HalpReadPCIConfig / HalpWritePCIConfig
  etc. — these are NTAPI (stdcall).
- `hal/halx86/include/halp.h` for HalpOpenRegistryKey, HalpCheckPowerButton,
  etc.
- Anything declared `FASTCALL` in C must be `extern "fastcall"` in Rust
  (not `extern "system"`).
- Anything declared bare (cdecl) must be `extern "C"` in Rust.

The `feedback_extern_c_abi.md` memory documents this is the #1 i386 crash
cause in this codebase.

## Rules of engagement

These are project-wide and override the smoke-test imperative when in
conflict. From `AGENTS.md`:

- **Do not revert `bussupp.rs` to `.c`** to make the build link. The Rust
  file stays. Fix it in place.
- **Do not delete `bussupp.rs`** — API translation cost real money.
- **Do not regenerate the rustconv translation** without asking the user
  first.
- **Do not comment out `pub mod bussupp;`** to make smoke green if you've
  already enabled it as part of debugging. Leaving it broken and visible
  to smoke is the correct state. (The current commit `06b86140` is the
  exception — bussupp was *never enabled* in that commit; it was kept as
  a compile-clean checkpoint.)
- **Do not commit without asking first.**
- **No `#[allow]`-spamming or weakening smoke tests.** If you find a
  legit bug in `bussupp.rs`, fix the bug — don't paper over the symptom.

## What "done" looks like

1. `sdk/lib/rust/hal-rust/src/x86/mod.rs` has `pub mod bussupp;` enabled
   (no comment).
2. `hal/halx86/CMakeLists.txt` lists `bussupp.rs` in the DEPENDS block.
3. `hal/halx86/legacy.cmake` has `legacy/bussupp.c` commented out with a
   `# replaced by Rust ...` note matching the other commented-out entries.
4. `cd build-i386 && cmake . && ninja livecd` succeeds.
5. `python3 tools/scan_ud2.py --check build-i386` exits 0 (no new ud2 in
   bussupp symbols).
6. `pytest tests/qemu/test_smoke.py -v --arch i386 --timeout 300 --skip-clean-build` → 7/7 PASS.
7. amd64 smoke unchanged: 7/7 PASS (bussupp is `#![cfg(target_arch = "x86")]`,
   so amd64 shouldn't even compile it — but verify the workspace builds).
8. A short summary commit message explaining which root cause it was.

## Quick reference

```bash
# Build i386
cd build-i386 && cmake . && ninja livecd && cd ..

# Run i386 smoke (fast iteration)
source .venv/bin/activate
pytest tests/qemu/test_smoke.py -v --arch i386 --timeout 300 --skip-clean-build

# Inspect hal.dll exports
i686-w64-mingw32-objdump -p build-i386/hal/halx86/hal.dll | less

# Inspect symbols
i686-w64-mingw32-nm build-i386/hal/halx86/hal.dll | grep -i bussupp
i686-w64-mingw32-nm build-i386/hal/halx86/libhal_rust.a 2>/dev/null | grep -i bussupp
# (libhal_rust.a actually lives at build-i386/hal/halx86/libhal_rust.a)

# Scan for ud2 miscompiles
python3 tools/scan_ud2.py build-i386

# Debugcon dump from last failed boot
xxd /tmp/pytest-of-curi/pytest-*/qemu-i3860/debugcon.bin | head -40
```

Good hunting.
