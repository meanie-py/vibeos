# Handoff: complete pcibus.rs integration on i386

## TL;DR

`sdk/lib/rust/hal-rust/src/x86/pcibus.rs` is a 1500-LOC i386 Rust translation
of `hal/halx86/legacy/bus/pcibus.c` (1307 LOC).  After ~227 cascade fixes
(227 → 0 rustc errors) the crate compiles cleanly with `ninja rust_hal` and
`ninja hal` links the full `hal.dll` cleanly.  The file is **checked in,
enabled in mod.rs, and in the CMakeLists DEPENDS list**.  i386 smoke 7/7
PASS in 52s unchanged.

**Current state — cohabitation:** Both `pcibus.c` and `pcibus.rs` are
compiled into hal.dll.  Per `--allow-multiple-definition`, the C symbols
win (they're earlier in the link order).  pcibus.rs's `#[no_mangle]`
exports get duplicate-symbol warnings at link time but are otherwise
dead code.  The smoke 7/7 PASS confirms this state boots correctly.

**Your job:** swap pcibus.rs into the active position by removing
pcibus.c from the C build.

## What's blocking the swap

`pcibus.c` defines **8 file-scope statics** that other HAL files
reference and that pcibus.rs currently *imports* (declares as `extern`).
When pcibus.c goes away, those definitions need to live in Rust.

The eight statics (with C-side line numbers in `pcibus.c`):

| Static | C line | C definition |
|---|---|---|
| `HalpBusType: ULONG` | 18 | `ULONG HalpBusType;` (zero-init BSS, written by halinit.c) |
| `HalpPCIConfigInitialized: BOOLEAN` | 20 | `BOOLEAN HalpPCIConfigInitialized;` (zero-init BSS) |
| `HalpMinPciBus, HalpMaxPciBus: ULONG` | 21 | `ULONG HalpMinPciBus, HalpMaxPciBus;` (zero-init BSS) |
| `HalpPCIConfigLock: KSPIN_LOCK` | 22 | `KSPIN_LOCK HalpPCIConfigLock;` (zero-init BSS) |
| `PCIConfigHandler: PCI_CONFIG_HANDLER` | 23 | `PCI_CONFIG_HANDLER PCIConfigHandler;` (zero-init BSS — initialized at runtime in HalpInitializePciStubs) |
| `PCIConfigHandlerType1: PCI_CONFIG_HANDLER` | 35 | Static initializer with 8 function pointers (4 Sync/Release + 6 ConfigIO read/write fns) |
| `PCIConfigHandlerType2: PCI_CONFIG_HANDLER` | 57 | Same shape as Type1 with Type2 fns |
| `HalpFakePciBusData: PCIPBUSDATA` | 77 | Static initializer with nested PCI_CONFIG_HANDLER inline-init (bindgen union access via `__bindgen_anon_1`) |
| `HalpFakePciBusHandler: BUS_HANDLER` | 94 | Static initializer with 16 fields including 4 function pointers + reference to `HalpFakePciBusData` |

Reference these by reading the C source at the line numbers above.  Easy
ones (BSS-zeroed statics): just translate as `#[no_mangle] pub static mut
X: T = T_DEFAULT_VALUE;`.  Hard ones (PCIConfigHandlerType1/2,
HalpFakePciBusData, HalpFakePciBusHandler): need bindgen `__bindgen_anon_1`
union-access syntax for the nested initializers.

## Recipe to finish integration

1. **Translate the 8 statics.**  pcibus.rs currently has them in the top
   `unsafe extern "C" { ... }` block around line 99.  Move them out of
   that block and define them with `#[no_mangle] pub static mut`.  Use
   bindgen's `_PCI_CONFIG_HANDLER` shape:

   ```rust
   #[no_mangle]
   pub static mut HalpBusType: ULONG = 0;

   #[no_mangle]
   pub static mut HalpPCIConfigInitialized: BOOLEAN = 0;

   // ... etc for the BSS-zeroed ones.

   #[no_mangle]
   pub static PCIConfigHandlerType1: PCI_CONFIG_HANDLER = PCI_CONFIG_HANDLER {
       Synchronize: Some(HalpPCISynchronizeType1 as unsafe extern "stdcall" fn(...)),
       ReleaseSynchronzation: Some(HalpPCIReleaseSynchronzationType1 as unsafe extern "stdcall" fn(...)),
       ConfigRead: [
           Some(HalpPCIReadUlongType1 as unsafe extern "stdcall" fn(...)),
           Some(HalpPCIReadUcharType1 as unsafe extern "stdcall" fn(...)),
           Some(HalpPCIReadUshortType1 as unsafe extern "stdcall" fn(...)),
       ],
       ConfigWrite: [
           Some(HalpPCIWriteUlongType1 as ...),
           // ... etc
       ],
   };
   ```

   The `as unsafe extern "stdcall" fn(...)` cast is needed because
   bindgen's FncConfigIO is `Option<unsafe extern "stdcall" fn(...)>` with
   a specific NTAPI signature.  Match it exactly.

2. **HalpFakePciBusData (PCIPBUSDATA).**  This is a nested struct with
   inline union initialization.  PCIPBUSDATA has fields like:

   ```c
   typedef struct _PCIPBUSDATA {
       PCIBUSDATA CommonData;          // (Tag, Version, ReadConfig, WriteConfig, ...)
       union { ... } Config;            // Type1/Type2/Type1Cardbus address-base union
       ULONG MaxDevice;                 // = 32
       /* + bitmap arrays */
   } PCIPBUSDATA;
   ```

   bindgen will name `Config.Type1` / `Config.Type2` / `Config.Type1Cardbus`
   through `__bindgen_anon_1` accessors.  Initialization requires writing
   into the correct union variant via `MaybeUninit<...>` if you want a
   `static`.  Easier path: declare as `pub static mut HalpFakePciBusData:
   PCIPBUSDATA = unsafe { core::mem::zeroed() }` (compile-time zeroed) and
   initialize the runtime fields in `HalpInitializePciStubs`.

3. **HalpFakePciBusHandler (BUS_HANDLER).**  Same approach as #2 — declare
   zero-init via `core::mem::zeroed()` then have `HalpInitializePciStubs`
   fill in the fields at runtime (RtlCopyMemory or per-field).  The C
   version is `static` zero-initialized data + a constant initializer; the
   runtime path in pcibus.c at line 1186-1206 does the per-field init.

4. **Remove pcibus.c from legacy.cmake.**  Comment it out the same way
   bushndlr.c / cmosbus.c / isabus.c are:

   ```cmake
   # legacy/bus/pcibus.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/pcibus.rs)
   ```

5. **Build + smoke.** `ninja livecd` from build-i386, then
   `pytest tests/qemu/test_smoke.py --arch i386 --skip-clean-build` should
   stay 7/7 PASS.  If boot wedges, the most likely culprit is a static
   data layout mismatch in HalpFakePciBusHandler — check that the BUS_HANDLER
   bindgen layout matches what other HAL files expect.

## Files of interest

| Path | What's in it |
|---|---|
| `sdk/lib/rust/hal-rust/src/x86/pcibus.rs` | the Rust translation (1500 LOC, 0 errors, linkable) |
| `hal/halx86/legacy/bus/pcibus.c` | the C original (1307 LOC) — keep until step 4 |
| `hal/halx86/include/bus.h` | C decls — `BUS_HANDLER`, `PCIPBUSDATA`, `PCI_CONFIG_HANDLER`, `FncSync`/`FncReleaseSync`/`FncConfigIO` typedefs |
| `sdk/lib/rust/ntkrnl-sys/src/hal_bindgen_i386.rs:63225` | bindgen layout of `FncConfigIO`/`FncSync`/`FncReleaseSync` (Option<unsafe extern "stdcall" fn(...)>) |
| `sdk/lib/rust/ntkrnl-sys/src/hal_bindgen_i386.rs:63245` | bindgen layout of `PCI_CONFIG_HANDLER` |
| `hal/halx86/legacy.cmake:6` | toggle line: `legacy/bus/pcibus.c` |
| `sdk/lib/rust/hal-rust/src/x86/mod.rs` | `pub mod pcibus;` is already enabled |
| `hal/halx86/CMakeLists.txt` | pcibus.rs is already in DEPENDS list |

## Smoke baseline

```
tests/qemu/test_smoke.py::test_boot_and_serial PASSED      [ 14%]
tests/qemu/test_smoke.py::test_milestones PASSED           [ 28%]
tests/qemu/test_smoke.py::test_qmp_status PASSED           [ 42%]
tests/qemu/test_smoke.py::test_snapshot_restore PASSED     [ 57%]
tests/qemu/test_smoke.py::test_screenshot PASSED           [ 71%]
tests/qemu/test_smoke.py::test_no_unexpected_ud2 PASSED    [ 85%]
tests/qemu/test_smoke.py::test_sacred_primitives PASSED    [100%]
============================== 7 passed in 52.32s ==============================
```

## Cascade fix journey (227 → 0)

Errors evolved: 227 → 90 → 67 → 65 → 55 → 46 → 40 → 33 → 32 → 28 → 11 →
10 → 4 → 3 → 0.

The cascade roots (extends [[rustconv-stubbed-bodies]] /
[[bindgen-union-access]] / [[write-bytes-element-count]]):

1. **`use ntkrnl_sys::ddk::*` pulls in too much.**  Several invalid imports
   (`BusNumber`, `Interrupt`) are enum variants, not module symbols.
   Plus the transitive cgu effect from pic_pic — bringing in ddk pulls all
   bindgen ddk extern decls into the HAL link unit.  Fix: use
   `ntkrnl_sys::hal_ddk::*` for HAL types, and declare ntoskrnl-exported
   helpers locally via `#[link(name = "ntoskrnl", kind = "raw-dylib")]`.

2. **rustconv imports types it also defines in-file.**  ~31 function names
   from pcibus.c (HalpAdjustPCIResourceList, HalpAssignPCISlotResources,
   etc.) appeared in the use clause `use ntkrnl_sys::hal_ddk::{...}` but
   are also `#[no_mangle] pub unsafe extern "system" fn X(...)` *defined
   below*.  This causes E0255 (name defined multiple times) — the file
   imports its own exports.  Fix: remove from the use clause.

3. **rustconv duplicates intra-file fn defs.**  `HalpXboxBlacklistedPCISlot`
   and `write_port_uchar`/`ushort`/`ulong` each appeared TWICE in the
   draft (lines 358/535 and 219/549).  rustconv emitted helper functions
   in multiple places.  Strip the second occurrence.

4. **`Vec<_>` from std is unavailable in `no_std` kernel.**  rustconv
   translated C wide string literals as
   `"...".encode_utf16().chain(core::iter::once(0)).collect::<Vec<_>>().as_ptr()`.
   Fix: define a `macro_rules! u16cstr` that builds a `[u16; N]` static
   array at compile time and returns its pointer.

5. **CmResourceType* are plain `pub const`, not enum-variant.**
   Bindgen emits the CmResourceType values as bare `pub const NAME: u32 =
   1;` (not `_CM_RESOURCE_TYPE_NAME`).  rustconv used the enum-variant
   path that doesn't exist.

6. **PCIConfigHandler is a value, not a pointer.**  rustconv emitted
   `(*PCIConfigHandler).field` calls (deref) but PCIConfigHandler is a
   global of type PCI_CONFIG_HANDLER (struct, not pointer).  Fix:
   `PCIConfigHandler.field.unwrap()(args)` for the Option<fn> fields.

7. **Bindgen FncConfigIO is Option<fn>, not bare fn.**  Same as the
   pic_pic SWInterruptHandlerTable lesson.  Calls must `.unwrap()()`.

8. **Bitfield setter widths vary per parent struct.**  bindgen generates
   `set_FunctionNumber(val: ULONG)` for PCI_SLOT_NUMBER but
   `set_FunctionNumber(val: UCHAR)` for PCI_TYPE2_CSE_BITS.  Cast per
   call site to match the destination width.

9. **Pointer field assignments need explicit casts.**  `(*BusData).Config.Type1.Address`
   is `*mut ULONG` (pointer); `PCI_TYPE1_ADDRESS_PORT = 0xCF8` is ULONG.
   rustconv assigned without cast.  Fix: `as *mut ULONG`.

10. **rustconv missed conditional function calls.**  Translated
    `if (HalpValidPCISlot(BusHandler, Slot))` to `if !(BusHandler, Slot)`
    — stripping the function name and leaving a bare tuple.  Six sites
    in pcibus.rs needed `HalpValidPCISlot(...)` restoration.

11. **`unsafe ` orphans + missing fn names.**  Same orphan keyword
    pattern as pic_pic (rustconv emitted bare `unsafe \n` keywords).
    Plus my own regex over-applied: when stripping
    `HalpFooBar` from import statements, it ALSO stripped the fn name
    from `pub unsafe extern "system" fn HalpFooBar(...)` definitions.
    Lesson: anchor the regex more tightly (require leading `,` or `{`).

## Methodology

Same playbook as bussupp (b7436f22) / pic_pic (b5608919):

1. Rewrite import block + `#![cfg(target_arch = "x86")]` header.
2. Strip orphan attributes (`#[inline]` between top-level decls).
3. Switch from `ntkrnl_sys::ddk::*` to local raw-dylib for ntoskrnl
   exports.
4. Strip function-local `unsafe extern { ... }` blocks; consolidate at
   file head.
5. Sweep bindgen `__bindgen_anon_1` for union access and `Option<fn>` for
   handler tables.
6. Cast widening: rustconv's `as u8` for u32 bitfields → `as u32` per
   call site.
7. Iterate `ninja rust_hal` until 0; then `ninja hal` for linking.
8. Smoke verifies cohabitation works.  Then incrementally remove the C
   file (which is where the integration risk lives).
