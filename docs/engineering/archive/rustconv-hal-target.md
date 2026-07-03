# rustconv: HAL-aware target abstraction

**Status:** IMPLEMENTED (commit 1cb012104bf); HAL translation since COMPLETE — HISTORICAL.
**Owner:** TBD.
**Last updated:** 2026-05-12.

## TL;DR

`rustconv translate` currently bakes a single implicit target crate (`ntoskrnl-rust`) into config, the assembler, and the system prompt. When pointed at HAL C files (`hal/halx86/...`) it produces structurally-recognizable Rust but with so many integration gaps that hand-fixing takes as long as hand-translating from scratch.

This doc captures the failure modes observed translating `hal/halx86/generic/dma.c` (commit `68a2eb11` preserved the raw rustconv draft, `d549e9b8` is the hand-translation that replaced it), and proposes a small refactor — make the target crate a first-class `Target` abstraction with HAL-specific paths, catalog wiring, output routing, and system prompt — so future HAL translations land closer to drop-in.

The change is roughly **~560 LOC across 6 files** (1 new template, 5 files modified). Out-of-scope for v1: type-identity casts (`ddk` ↔ `hal_ddk`), iterative compile-check loop, FORCEINLINE helper crate. Those are tracked as follow-up work.

---

## Why bother

The `dma.c` rustconv→Rust→smoke pipeline cost ~$0.10 in API and produced 1618 LOC of output that I then largely discarded — the hand-translation that landed shares structure with the draft but had to rewrite every import block, every macro call site, every union/bitfield access, every type-identity cast, and every cross-arch primitive. Net useful information from the rustconv pass: ~30% (function ordering + arg shapes + control flow).

If we can lift that to 70–80% useful — i.e. the LLM's output compiles after some `as *mut _` fixups instead of needing structural rewrites — then rustconv becomes a real time-saver for HAL translations. The remaining HAL work (~12,000 LOC C across ~25 files) is the immediate target; ARM64 HAL and other ports benefit later.

## What we learned from `dma.c`

Ten failure categories observed, roughly ordered by frequency:

### 1. Wrong import-module paths (every import wrong)

The rustconv draft emitted:
```rust
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{ /* 30+ items */ };
```

Those `globals`/`constants`/`base`/`ke`/`mm`/`ob`/`rtl` modules are `ntoskrnl-rust` internal Rust modules — not part of `ntkrnl_sys` at all. HAL Rust crates use only `ntkrnl_sys::ddk` (kernel API) + `ntkrnl_sys::hal_ddk` (HAL-internal types). The system prompt teaches the wrong import skeleton because it was written for `ntoskrnl-rust`.

### 2. C macros declared as `extern fn` (5 macros)

```rust
extern "C" { fn PAGE_ROUND_UP(x: ULONG) -> ULONG; }
extern "system" {
    fn FIELD_OFFSET(struc: *const u8, field: *const u8) -> u32;
    fn UlongToPtr(value: u32) -> *mut core::ffi::c_void;
    fn WRITE_PORT_UCHAR(port: *mut u8, value: u8);
    fn READ_PORT_UCHAR(Port: *const u8) -> u8;
}
```

None of these are linkable symbols — they're C preprocessor macros or compiler intrinsics. The LLM treated them as unknown imports rather than inlining.

Worse, when `FIELD_OFFSET` arguments referenced structs the LLM couldn't preserve through the translation, it emitted **`FIELD_OFFSET(core::ptr::null(), core::ptr::null())`** placeholders — silent semantic loss, 4 sites in dma.c.

### 3. Bindgen bitfield/union accessor pattern wrong (every bitfield)

The LLM emitted direct field assignment:
```rust
(*ExtendedMode.__bindgen_anon_1.__bindgen_anon_1).ChannelNumber = ch;
(*DmaMode.__bindgen_anon_1.__bindgen_anon_1).TransferType = WRITE_TRANSFER as u8;
```

Bindgen actually generates `set_X(v)`/`X()` methods because the fields live inside a `__BindgenBitfieldUnit<[u8; N]>`:
```rust
ExtendedMode.__bindgen_anon_1.set_ChannelNumber(ch);
DmaMode.__bindgen_anon_1.set_TransferType(WRITE_TRANSFER as u8);
```

This pattern repeated for every `PCI_TYPE1_CFG_CYCLE_BITS`, `PCI_SLOT_NUMBER`, `DMA_MODE`, `DMA_EXTENDED_MODE` access in dma.c + kdpci.c.

### 4. Type identity ambiguity (`ddk` vs `hal_ddk`) — fundamental bindgen artifact

Bindgen generates `ntoskrnl.h`-based bindings into `ntkrnl_sys/src/bindgen_*.rs` (module `ddk`) and a separate HAL-internal-headers-based bindings into `ntkrnl_sys/src/hal_bindgen_*.rs` (module `hal_ddk`). Both contain `_RTL_BITMAP`, `_MDL`, `_KEVENT`, `_KDEVICE_QUEUE`, `_LARGE_INTEGER`, `_WORK_QUEUE_ITEM` etc. — these are *distinct Rust types* with identical layouts.

Pointer-typed function args at the `ddk` ↔ `hal_ddk` boundary therefore need `as *mut _` casts:
```rust
RtlFindClearBitsAndSet((*master).MapRegisters as *mut _, count, 0);
//                                              ^^^^^^^ master.MapRegisters is hal_ddk::_RTL_BITMAP,
//                                                       RtlFindClearBitsAndSet takes ddk::_RTL_BITMAP
KeInitializeDeviceQueue(&raw mut (*adapter).ChannelWaitQueue as *mut _);
ExQueueWorkItem(&raw mut (*work_item).work_queue_item as *mut _, queue_type);
```

About 15 such cast sites in dma.c. The system prompt doesn't teach this idiom because `ntoskrnl-rust` only uses `ddk` (it's the kernel itself, no cross-DLL HAL types).

### 5. Bindgen c_uint enums matched as variant names

C enums (without explicit `enum class` etc.) become bindgen `const` declarations + a `type _X_FOO = ::core::ffi::c_uint` alias:

```rust
pub const _DMA_SPEED_Compatible: _DMA_SPEED = 0;
pub const _DMA_SPEED_TypeA: _DMA_SPEED = 1;
pub type _DMA_SPEED = ::core::ffi::c_uint;
```

The LLM tried to match on them as if they were Rust `enum` variants:
```rust
match (*DeviceDescription).DmaSpeed {
    _DMA_SPEED_Compatible => ...,        // doesn't compile
    _DMA_SPEED_TypeA => ...,
}
```

Correct pattern:
```rust
match (*DeviceDescription).DmaSpeed {
    x if x == _DMA_SPEED_Compatible as u32 => ...,
    x if x == _DMA_SPEED_TypeA as u32 => ...,
    _ => return 0,
}
```

### 6. Per-arch availability surprises (4 cases)

- **`KfRaiseIrql`/`KfLowerIrql`**: i386 fastcall export, missing on amd64. Cross-arch translation needs `#[cfg(target_arch = ...)]` branches with `KeRaiseIrqlToDpcLevel`/`KeLowerIrql` raw-dylib on amd64.
- **`WRITE_PORT_UCHAR`/`READ_PORT_UCHAR`**: i386 NTAPI export only; amd64 expects inline intrinsics. Local inline-asm helpers work cross-arch.
- **`KernelMode`**: bare `KernelMode` const on i386 (in `hal_ddk`), but `_MODE_KernelMode` on amd64. Easiest: hard-code `0u32`.
- **`IoAllocateAdapterChannel`**: re-exported by `ntkrnl_sys::ddk` on i386, missing on amd64 — must raw-dylib import from `ntoskrnl`.

The catalog doesn't surface per-arch availability, so the LLM produces uniformly-arched code that fails to link on one side.

### 7. FORCEINLINE without exported symbol (~7 helpers needed in dma.c)

`KeInitializeSpinLock`, `RtlInitEmptyUnicodeString`, `MmGetMdlPfnArray`, `MmGetMdlVirtualAddress`, `MmGetSystemAddressForMdlSafe`, plus all `LIST_ENTRY` operations (`InitializeListHead`, `InsertTailList`, `InsertHeadList`, `RemoveEntryList`, `RemoveHeadList`, `IsListEmpty`) are FORCEINLINE in NDK headers — bindgen emits extern decls but no linker symbol exists.

Each translation re-inlines them locally. That's bug-bait (does my `insert_tail_list` match the C definition byte-for-byte?) and pure repetition. They belong in a shared crate.

### 8. Function-pointer ABI mismatch in dispatch tables (~14 sites)

`DMA_OPERATIONS` fields are typed `Option<unsafe extern "stdcall" fn(...)>` by bindgen. My exported functions are `extern "system"`. On i386 the ABIs are identical but Rust's type system treats `"stdcall"` and `"system"` as distinct function-pointer types. Direct assignment fails:
```rust
ops.PutDmaAdapter = Some(HalPutDmaAdapter);
//                       ^^^^^^^^^^^^^^^^ expected "stdcall" fn, found "system" fn
```

Workaround pattern (already used by `busemul.rs`):
```rust
ops.PutDmaAdapter = Some(core::mem::transmute(HalPutDmaAdapter as usize));
```

The LLM didn't know this pattern. It also tried to assign at static-init time, which fails because const eval can't transmute fn pointers — must defer to runtime in an init function.

### 9. Topo sort emits no forward declarations

rustconv's topological sort puts callees before callers — works for top-down call chains. But cycles (HalpInitDma → HalpDmaAllocateMasterAdapter → HalpGrowMapBuffers → HalpGetAdapterMaximumPhysicalAddress; also HalpScatterGatherAdapterControl → IoMapTransfer where IoMapTransfer is defined later) need either:
- A reordering pass that finds an acyclic ordering, OR
- Forward `extern "system" fn X(...);` declarations at the top of the file so bodies can appear in any order.

The latter is dramatically simpler and matches how C handles it.

### 10. Hex literal type-mismatch (`!0xFusize` vs `u32`)

Small but recurring: bindgen `ULONG` is `c_ulong` = `u32`, but `usize` literals don't auto-coerce. Translation produced:
```rust
if HalpEisaDma == 0 && ((p1 ^ p2) as ULONG_PTR) & !0xFusize != 0 { /* ... */ }
//                                                ^^^^^^^^^ usize, but lhs is u32 on i386
```

Need `!0xFu32 as ULONG_PTR` or unify the types differently.

---

## Design

### Core abstraction: `Target`

Add a dataclass to `tools/rustconv/rustconv/config.py`:

```python
@dataclass
class Target:
    name: str                              # "ntoskrnl" or "hal"
    source_dir: Path                       # NTOSKRNL_DIR or HAL_DIR
    output_crate: Path                     # ntoskrnl-rust or hal-rust
    bindgen_files: dict[str, Path]         # arch → bindgen_{arch}.rs (+ optionally hal_bindgen)
    spec_file: Path | None                 # ntoskrnl.spec (None for HAL — no exports surface)
    system_prompt: Path                    # templates/system_prompt.md or system_prompt_hal.md
    # Output-routing hooks
    src_subdir: Path | None                # "src/" or "src/x86/" — for hal-rust which gates by arch
    cmake_deps_file: Path | None           # hal/halx86/CMakeLists.txt for HAL (needs explicit DEPENDS list)
    cmake_source_files: list[Path]         # generic.cmake / acpi.cmake / pic.cmake / apic.cmake
                                           # (the .cmake files whose C-source list must be edited)


TARGETS = {
    "ntoskrnl": Target(
        name="ntoskrnl",
        source_dir=REACTOS_ROOT / "ntoskrnl",
        output_crate=RUST_DIR / "ntoskrnl-rust",
        bindgen_files={
            "i386":  NTKRNL_SYS_DIR / "bindgen_i386.rs",
            "amd64": NTKRNL_SYS_DIR / "bindgen_amd64.rs",
            "arm64": NTKRNL_SYS_DIR / "bindgen_arm64.rs",
        },
        spec_file=REACTOS_ROOT / "ntoskrnl" / "ntoskrnl.spec",
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=None,
        cmake_deps_file=None,
        cmake_source_files=[],
    ),
    "hal": Target(
        name="hal",
        source_dir=REACTOS_ROOT / "hal",
        output_crate=RUST_DIR / "hal-rust",
        bindgen_files={
            "i386":  NTKRNL_SYS_DIR / "bindgen_i386.rs",
            "amd64": NTKRNL_SYS_DIR / "bindgen_amd64.rs",
            # Additional hal_bindgen sources merged in catalog:
            "i386_hal":  NTKRNL_SYS_DIR / "hal_bindgen_i386.rs",
            "amd64_hal": NTKRNL_SYS_DIR / "hal_bindgen_amd64.rs",
        },
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt_hal.md",
        src_subdir=Path("src/x86"),
        cmake_deps_file=REACTOS_ROOT / "hal" / "halx86" / "CMakeLists.txt",
        cmake_source_files=[
            REACTOS_ROOT / "hal" / "halx86" / "generic.cmake",
            REACTOS_ROOT / "hal" / "halx86" / "acpi.cmake",
            REACTOS_ROOT / "hal" / "halx86" / "apic.cmake",
            REACTOS_ROOT / "hal" / "halx86" / "pic.cmake",
            REACTOS_ROOT / "hal" / "halx86" / "legacy.cmake",
        ],
    ),
}


def detect_target(c_path: Path) -> Target:
    """Pick a target by sniffing the C path. Falls back to ntoskrnl."""
    p = c_path.resolve()
    if "hal/halx86" in p.as_posix() or "/hal/" in p.as_posix():
        return TARGETS["hal"]
    return TARGETS["ntoskrnl"]
```

### Path resolution + assembler routing

In `__main__.py::cmd_translate`, after the early `args.path` parsing:
```python
target = detect_target(Path(args.path).resolve())
if target.name == "hal":
    # Path resolution: HAL files live under hal/halx86/{subsystem}/file.c
    # NOT ntoskrnl/{subsystem}/file.c
    pass
```

`assembler.assemble_file(..., target)` writes to:
- ntoskrnl: `target.output_crate / "src" / subsystem / f"{stem}.rs"`  (current behavior)
- HAL: `target.output_crate / target.src_subdir / f"{stem}.rs"`  (e.g. `hal-rust/src/x86/dma.rs`)

For HAL, the assembler **also** must:
- Append to `hal-rust/src/x86/mod.rs` (`pub mod dma;`)
- Append to `hal/halx86/CMakeLists.txt` DEPENDS list (`${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/x86/dma.rs`)
- Comment out the original `.c` in the right `*.cmake` file (`# generic/dma.c — replaced by Rust (...)`)

The third step (which subsystem .cmake) maps from the C path: `hal/halx86/{subsystem}/dma.c` → `{subsystem}.cmake` (or `generic.cmake` for `generic/`). Define a helper:
```python
def hal_cmake_for_c(c_path: Path) -> Path:
    sub = c_path.parent.name  # "generic", "acpi", "apic", "pic", "legacy"
    return REACTOS_ROOT / "hal" / "halx86" / f"{sub}.cmake"
```

### Catalog wiring

`catalog.build_catalog(arch, target)`: if `target.name == "hal"`, parse both `bindgen_{arch}.rs` (kernel side, what gets called) and `hal_bindgen_{arch}.rs` (HAL-internal types, what gets used in struct fields/locals). Symbol resolution must know which module a given identifier lives in so the LLM gets the right `use ntkrnl_sys::{ddk,hal_ddk}::...` shape.

Add a `module: str` field to the catalog's `Function`/`Type` records. HAL system prompt then can emit:
```rust
use ntkrnl_sys::ddk::{ExAllocatePoolWithTag, KeBugCheckEx, /* ...kernel functions... */};
use ntkrnl_sys::hal_ddk::{PADAPTER_OBJECT, KIRQL, ULONG, /* ...HAL types/locals... */};
```

The split rule is approximately: types defined inside HAL headers (ADAPTER_OBJECT, DEVICE_DESCRIPTION, EISA_CONTROL, DMA_PAGE, …) live in `hal_ddk`; kernel APIs (Ke*, Ex*, Mm*, Ob*, Rtl*, Io*, Zw*) live in `ddk`. The catalog already knows this — surface it to the prompt builder.

### System prompt: `templates/system_prompt_hal.md`

Distilled from the ntoskrnl prompt (~825 LOC) down to a HAL-focused ~500 LOC. Key sections to rewrite:

**1. Crate dependencies + import block** — replace ntoskrnl-rust-flavored prelude with the `ddk` + `hal_ddk` split. Show worked import examples.

**2. C macro pre-pass — list of macros to inline locally**

Show all of these patterns by example, with the wrong vs right form side-by-side:

| C macro | Rust inline |
|---|---|
| `FIELD_OFFSET(S, F)` | `core::mem::offset_of!(S, F)` |
| `PAGE_ROUND_UP(x)` | `(x + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)` |
| `BYTES_TO_PAGES(b)` | `(b + PAGE_SIZE - 1) >> PAGE_SHIFT` |
| `BYTE_OFFSET(va)` | `(va & (PAGE_SIZE - 1)) as ULONG` |
| `PAGE_ALIGN(p)` | `(p as ULONG_PTR & !(PAGE_SIZE - 1)) as *mut u8` |
| `UlongToPtr(v)` | `v as usize as *mut c_void` |
| `PtrToUlong(p)` | `p as usize as ULONG` |
| `WRITE_PORT_UCHAR(port, val)` | Local inline-asm `out dx, al` helper (cross-arch) |
| `READ_PORT_UCHAR(port)` | Local inline-asm `in al, dx` helper |
| `MmGetMdlPfnArray(Mdl)` | `Mdl.add(1) as PPFN_NUMBER` |
| `MmGetMdlVirtualAddress(Mdl)` | `((*Mdl).StartVa as ULONG_PTR + (*Mdl).ByteOffset as ULONG_PTR) as PVOID` |
| `MmGetSystemAddressForMdlSafe(Mdl, prio)` | conditional on `MdlFlags & MDL_MAPPED_TO_SYSTEM_VA` |
| `KeInitializeSpinLock(sl)` | `*sl = 0` |
| `InitializeListHead(h)` | `(*h).Flink = h; (*h).Blink = h;` |
| `InsertTailList(h, e)` / `InsertHeadList` / `RemoveEntryList` / `RemoveHeadList` / `IsListEmpty` | (textbook) |
| `CONTAINING_RECORD(p, T, F)` | `((p as ULONG_PTR) - offset_of!(T, F) as ULONG_PTR) as *mut T` |
| `RTL_CONSTANT_STRING(L"...")` | manually construct `UNICODE_STRING` with const-eval u16 buffer |
| `InitializeObjectAttributes(oa, name, attrs, root, sd)` | direct field writes |
| `NT_SUCCESS(s)` | `s >= 0` |
| `NT_ERROR(s)` | `(s as u32) >> 30 == 3` |

**3. Bindgen union/bitfield patterns**

```rust
// ✅ Bitfields: bindgen generates set_X()/X() methods, NOT direct fields
let mut mode: _DMA_MODE = core::mem::zeroed();
mode.__bindgen_anon_1.set_Channel(2);                    // ✅ correct
mode.__bindgen_anon_1.set_RequestMode(SINGLE_REQUEST_MODE as UCHAR);

// ❌ NEVER write to the inner bitfield struct directly
// (*mode.__bindgen_anon_1.__bindgen_anon_1).Channel = 2;
```

**4. Bindgen c_uint enum matching**

```rust
// ✅ Match c_uint enums via `if x == K as u32`
match speed {
    x if x == _DMA_SPEED_Compatible as u32 => COMPATIBLE_TIMING,
    x if x == _DMA_SPEED_TypeA as u32 => TYPE_A_TIMING,
    _ => return 0,
}

// ❌ NEVER use variant-name match arms — bindgen produces consts not enum variants
// match speed { _DMA_SPEED_Compatible => ..., }
```

**5. ddk/hal_ddk type-identity casts**

> When passing a struct pointer from a hal-internal field (in `hal_ddk`) to a kernel function (in `ddk`), insert `as *mut _`:
> ```rust
> RtlFindClearBitsAndSet((*master).MapRegisters as *mut _, count, 0);
> KeInitializeDeviceQueue(&raw mut (*adapter).ChannelWaitQueue as *mut _);
> ```
> Same with `core::mem::transmute(...)` for non-pointer types (e.g. LARGE_INTEGER passed by value).

**6. Function-pointer dispatch tables**

```rust
// Setting Option<extern "stdcall" fn(...)> fields with extern "system" functions
HalpDmaOperations.PutDmaAdapter = Some(core::mem::transmute(HalPutDmaAdapter as usize));
//                                     ^^^^^^^^^^^^^^^^^^^^^                ^^^^^^^^^
//                                     transmute via usize bridge — direct cast fails

// For static initializers, defer fn-pointer fields to a runtime init function;
// const-eval can't transmute fn pointers.
```

**7. Per-arch primitives**

Show the `#[cfg(target_arch = "x86")]` / `#[cfg(target_arch = "x86_64")]` split for:
- `KfRaiseIrql`/`KfLowerIrql` (i386 fastcall) vs `KeRaiseIrqlToDpcLevel`/`KeLowerIrql` (amd64 raw-dylib from ntoskrnl)
- `WRITE_PORT_UCHAR`/`READ_PORT_UCHAR` — local inline-asm cross-arch helpers
- Anything else surfaced by the catalog's per-arch availability flags

**8. Cross-DLL data — `#[link(name="ntoskrnl", kind="raw-dylib")]`**

When `hal-rust` references kernel data exports (`HalDispatchTable`, `HalPrivateDispatchTable`, `KdComPortInUse` if owned by ntoskrnl, etc.), use raw-dylib. The catalog should tag these; the system prompt should include the pattern.

**9. Forward declarations for in-file circular calls**

> If function A calls function B that's defined later in the same file (topo sort can't break cycles), emit a forward `extern "system" fn B(...);` block at the top. Don't reorder bodies.

---

## Implementation plan

### Day 1 — code plumbing

| Step | File | Effort |
|---|---|---|
| 1 | `config.py` — add `Target` dataclass, `TARGETS` map, `detect_target()` helper | 60 LOC, 1 hr |
| 2 | `__main__.py::cmd_translate` — wire `target` through to scanner/catalog/resolver/assembler. Same for `submit`, `batch`, `estimate`. | 20 LOC, 30 min |
| 3 | `catalog.py` — `build_catalog(arch, target)` merges hal_bindgen when target is HAL, tags each symbol with `module: str` ("ddk" or "hal_ddk"). | 30 LOC, 1 hr |
| 4 | `prompt_builder.py` / `function_prompt.py` — pick `target.system_prompt`, generate target-appropriate `use ntkrnl_sys::...` block using catalog `module` tags. | 20 LOC, 1 hr |
| 5 | `assembler.py` — write to `target.output_crate / target.src_subdir / file.rs`; auto-edit mod.rs + CMakeLists.txt DEPENDS + subsystem.cmake for HAL targets. | 30 LOC, 1.5 hr |
| 6 | Smoke test: `python -m rustconv translate <abs path>/hal/.../tiny_file.c --dry-run` — confirm Target detection picks hal, paths resolve, prompt loads. | 30 min |

Target end-of-day-1: `rustconv translate <hal file> --dry-run` shows correct output paths, system prompt path, and `use` block in the planned prompt.

### Day 2 — system prompt

| Step | File | Effort |
|---|---|---|
| 7 | Write `templates/system_prompt_hal.md` from scratch (don't try to share the ntoskrnl prompt — they diverge enough that one inheritance hierarchy isn't worth it). Use the dma.rs hand-translation as the working example for each section. | 400-500 LOC, 4-5 hr |
| 8 | Spot-check by manually running prompt builder against a small HAL .c file; verify generated prompt looks sensible. | 30 min |

### Day 3 — regression test + iteration

| Step | What | Cost |
|---|---|---|
| 9 | Pick the regression test file: **`hal/halx86/legacy/bus/bushndlr.c`** (473 LOC, both arches, bus-handler dispatch). Different domain from dma (no DMA controllers, no MDLs, no scatter-gather), function-pointer-heavy (exercises the dispatch-table transmute pattern). Reasonable size — small enough to bound API cost, large enough to surface a representative range of issues. | — |
| 10 | `rustconv translate hal/halx86/legacy/bus/bushndlr.c -o /tmp/bushndlr_new.rs` and inspect. Score the output against the 10 failure categories: how many disappear vs persist? | ~$0.02 |
| 11 | Iterate on `system_prompt_hal.md` based on what bushndlr.c still gets wrong. | varies |
| 12 | (Optional) Integrate bushndlr.rs by hand using the rustconv output as a base. If the time-to-integrate is <1 hr, the system prompt is good enough; if it's >2 hr, more iteration needed. | 1-2 hr |

**Cost budget:** ~$0.05 in API calls over the whole regression cycle (bushndlr.c at ~half the size of dma.c).

**Validation criteria:** the hand-fixes needed on bushndlr.c should number <5 (vs ~50 on dma.c's first compile). Specifically watch for:
- Zero `extern fn` decls for C macros (FIELD_OFFSET, PAGE_ROUND_UP, etc.)
- Zero `_DMA_X_FOO => ...` enum-variant match arms
- Bindgen `set_X()` methods used for bitfields where applicable
- Correct `use ntkrnl_sys::{ddk,hal_ddk}::...` split
- Forward decls emitted where call cycles span the file

---

## What this does NOT fix (deferred to follow-up work)

### A. Type-identity casts (`ddk` ↔ `hal_ddk`)

The HAL prompt will *teach* the LLM the cast idiom, but bindgen-emitted struct fields will continue to surface this 10-20 times per HAL file. Reliable fix is mechanical:
- Post-processor reads `cargo check` errors of the form `expected.*ddk::_X.*found.*hal_ddk::_X` (or vice versa)
- Auto-inject `as *mut _` cast at the offending source location
- Loop until clean

Estimated effort: 2-3 days (compiler-output parser + sed-like injector). Probably worth doing right after the HAL prompt lands, because it's also useful for the ntoskrnl prompt's residual type errors.

### B. FORCEINLINE helper crate

`KeInitializeSpinLock`, `InitializeListHead`, the LIST_ENTRY family, `MmGetMdlPfnArray`, `MmGetMdlVirtualAddress`, `MmGetSystemAddressForMdlSafe`, `CONTAINING_RECORD`, `RTL_CONSTANT_STRING`, `InitializeObjectAttributes` — these would live cleaner in `sdk/lib/rust/ntkrnl-sys/src/forceinline.rs` (or a sibling crate), then HAL/ntoskrnl translations both `use ntkrnl_sys::forceinline::*;`.

Risk: the inlines have subtle correctness depending on whether the wrapping function holds a lock, runs at PASSIVE/DISPATCH, etc. Better to land them as a small audited library than re-derive them per file.

Estimated effort: 1-2 days. Tracked separately.

### C. Iterative compile-check loop

After translating each function, run `cargo check -p hal-rust --target i686-pc-windows-gnu` on the in-progress file. Feed errors back to the LLM for a fix pass. The compiler is a perfect oracle for type/ABI issues that prompt-engineering can't reliably catch.

Estimated effort: 3-4 days. Higher upfront cost but the payoff compounds across every future translation in both crates. Probably the right "next big rustconv investment" after this HAL doc lands.

### D. SEH (Structured Exception Handling)

HAL has minimal SEH usage (none in dma.c). When/if we hit a HAL file with `_SEH2_TRY` blocks, the existing ntoskrnl SEH-handling sections in the system prompt apply unchanged. Not a HAL-specific problem.

---

## Risks / open questions

1. **Catalog growth**: merging `hal_bindgen_*.rs` doubles the symbol index (both files are ~80K LOC). May slow `build_catalog` from seconds to tens of seconds. Mitigation: parse lazily, cache to disk (`catalog.pickle`).

2. **Subsystem detection for cmake editing**: HAL has 5 subsystem `.cmake` files (generic, acpi, apic, pic, legacy). Path-based mapping (parent dir name → cmake) works for `generic/` and `acpi/`, but `legacy/bus/` files map to `legacy.cmake` (grandparent + .cmake, not parent). Need to walk up the directory tree until we find a matching `*.cmake` listing the file. Add a defensive grep-first check before modifying.

3. **Auto-editing CMakeLists.txt is destructive**: if the script breaks mid-edit (e.g. user kills it after the .rs file is written but before the cmake list is updated), the build will fail. Mitigation: write CMakeLists changes via a temp file + atomic rename; print "manual review recommended" in the success message.

4. **HAL ↔ kernel coupling drift**: `ntkrnl_sys::hal_ddk` is hand-curated; if the HAL grows new C-side types that bindgen hasn't been re-run for, translations will silently fall back to "unknown symbol" placeholders. Mitigation: extend `verify_constants.py` / `verify_ffi.py` to run on HAL crate output too, surfacing missing bindgen coverage.

5. **arm64 HAL**: the design uses the same hal_bindgen + system prompt for both arches via `target.bindgen_files`. ARM64 isn't in the HAL config yet (no `hal_bindgen_arm64.rs`). When ARM64 HAL work begins, extend the dict; no design change needed.

---

## File-by-file change inventory

```
tools/rustconv/
├── rustconv/
│   ├── config.py              MOD  (+60 LOC: Target dataclass, TARGETS map, detect_target)
│   ├── __main__.py            MOD  (+20 LOC: target = detect_target(args.path); plumb through)
│   ├── catalog.py             MOD  (+30 LOC: merge hal_bindgen when target=hal; module tag)
│   ├── prompt_builder.py      MOD  (+10 LOC: pick target.system_prompt; use module tags)
│   ├── function_prompt.py     MOD  (+10 LOC: same plumbing)
│   ├── assembler.py           MOD  (+30 LOC: HAL output paths + CMakeLists/cmake auto-edit)
│   └── (no new files)
├── templates/
│   ├── system_prompt.md       (unchanged — ntoskrnl version)
│   └── system_prompt_hal.md   NEW  (~400 LOC: HAL imports, macro catalog, bitfields, casts, per-arch)
└── docs/engineering/
    └── rustconv-hal-target.md NEW  (this file)

Total: ~560 LOC across 7 files (6 modified + 1 new template + this doc).
```

## Success metrics

After landing, re-run rustconv against `bushndlr.c` (473 LOC, both arches). Hand-integration time should be **<1 hr** (vs ~3 hr for dma.c). The 10 failure categories from §2 should drop to ≤3 categories still appearing (most likely: residual ddk↔hal_ddk casts, which need the compile-check loop to fully eliminate).

If hand-integration time is still >2 hr, iterate on the system prompt before considering the work done.
