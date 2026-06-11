<!--
  HARDENED kernel-subsystem grind preset (distilled from the cmlib Phase-2 run,
  7 turns 282→0). Use for compile-grinding a banked DeepSeek-MECHANICAL translation
  of a kernel C lib/subsystem into a Rust cohab crate. Fill <<...>> and pass via
  `deepdrive run --prompt-file`. Single prompt, reused across turns — the ordered
  Method lets DeepCode self-phase (parse → imports → const/enum → casts → tail).

  ── FRONTIER PREP — do these BEFORE the first grind turn (collapses turn count) ──
   1. COMPLETENESS: count C fn defs vs translated (.responses/<sub>__<f>/). Fill gaps
      (re-translate the missing fns) OR plan extern decls. Mechanical mode SILENTLY
      SKIPS some functions.
   2. ABI: nm a built C .obj. `_Name@N` = stdcall → extern "system"; `_Name` = cdecl →
      extern "C". Tell the grinder which (and that it must NOT change ABI — you sweep
      + nm-verify in review). Mechanical defaults to extern "C".
   3. CONSTANT/ENUM CATALOG (the #1 lever): harvest the subsystem's #defines + file-local
      typedef enums into a `glue.rs` (real values from the C headers, MAXULONG-style macros
      resolved). The mechanical pass FABRICATES missing consts with WRONG values
      (cmlib: HCELL_NIL=0 not 0xFFFFFFFF; dropped a RESULT enum variant). Front-loading
      these into glue BEFORE grinding removes ~2 turns of stop-and-ask churn.
   4. MODEL LADDER: turns 1-2 (parse/import/structural) on deepseek-v4-flash (cheap);
      escalate to deepseek-v4-pro for the E0308/E0277 cast bulk + the finicky tail
      (v4-flash plateaus on casts, v4-pro crushes them).
   Expect the count to stay FLAT or RISE after a parse-clearing turn — clearing parses
   UNMASKS the real type-error surface. Judge a turn by its reply's class breakdown, not
   the raw count delta.
-->

# Grind `classpnp-rust` to `cargo check -p classpnp-rust --target i686-pc-windows-gnu` GREEN. GRIND ONLY — not a rewrite.

Banked MECHANICAL translation. Files: `autorun.rs, class.rs, classwmi.rs, clntirp.rs, create.rs, debug.rs, dictlib.rs, dispatch.rs, history.rs, lock.rs, obsolete.rs, power.rs, retry.rs, srblib.rs, utils.rs, xferpkt.rs`. ABI of this lib: **mixed — internal helpers extern "C" (cdecl); kernel-called entry points (DriverEntry, dispatch, completion/DPC/IO-workitem routines) extern "system". DO NOT change ABI on any definition.**.

## HARD RULES (never violate)
1. **NEVER rewrite a function wholesale.** Smallest edit per error; preserve structure/logic/naming.
2. **No stubs.** Never use `todo!()`/`unimplemented!()`/`panic!()`/dummy returns to dodge an error.
3. **NEVER fabricate a constant/enum/value.** If a const/enum/type is missing and NOT in `glue.rs`
   or `ntkrnl_sys`, **STOP and report its name + the C header it comes from** — the engineer adds the
   REAL value. The mechanical pass guesses WRONG values; a wrong const silently corrupts data. This
   rule is the whole point of the frontier/cheap split — respect it on turn 1.
4. **ONLY edit `autorun.rs, class.rs, classwmi.rs, clntirp.rs, create.rs, debug.rs, dictlib.rs, dispatch.rs, history.rs, lock.rs, obsolete.rs, power.rs, retry.rs, srblib.rs, utils.rs, xferpkt.rs`.** Don't touch other files/crates/lib.rs/Cargo.toml/glue.rs.
5. **Do NOT change `extern "C"`↔`extern "system"` on fn DEFINITIONS.** The engineer owns the ABI
   sweep. Ignore `clashing_extern_declarations` warnings.
6. **Batch edits, then re-check ONCE per batch** (cost ≈ rounds × context; don't cargo-check per edit).

## METHOD — work in this order (earlier classes mask later ones)
1. **Parse errors first.** Mechanical preambles are malformed: `#[no_mangle] pub fn ...;` lines
   illegally inside `extern { }` blocks (delete them — they're dup forward-decls), unbalanced braces
   (balance), mangled/merged function boundaries (separate; the per-fn `.responses` files are the
   clean reference), C-style `for`/comments swallowing `#[cfg]`/`if` (fix syntax). Re-check; repeat
   per file until it parses.
2. **Imports.** Missing types/values are in `ntkrnl_sys::{ddk,base,constants,mm,rtl}` or another
   module (`crate::<mod>::<sym>`) or `crate::glue::*`. Import the REAL symbol. (If truly missing →
   rule 3: STOP and report.)
3. **Then the cookbook classes below.** Largest class first.

## ERROR-CLASS COOKBOOK (exact fix per class — apply mechanically)
- **E0308 mismatched types:** add the `as` cast to the EXPECTED type (`as u32/i32/u16/u8/usize`).
  Pointer mismatch (`*mut _LARGE_INTEGER` vs `*mut i64`) → cast the pointer `as *mut _`. NEVER change
  a typedef/struct-field type — cast at the use site only.
- **E0277 trait not satisfied** (mixed-width arith/bitops `u16 & u32`, `usize * u32`): cast operands
  to the wider common type, matching the C: `(a as usize) * b`.
- **E0618 expected function** (`Option<extern fn>` field called directly): `(field.unwrap())(args)`
  (C calls it unconditionally → `.unwrap()` is faithful; use `if let Some(f)=field` only where the C
  null-checks).
- **E0609/E0614 no field / cannot deref** (bindgen anon-union rename): use the real
  `(*x).__bindgen_anon_N.field` or `(*x).u.field` path — inspect the struct in `ntkrnl-sys/src`.
- **E0608 cannot index** (it's a raw pointer/flex-array, not a slice): `*ptr.add(i)` not `ptr[i]`.
- **E0793 ref to packed/unaligned field** (`#[repr(packed)]`): `core::ptr::addr_of!((*p).field)` then
  `.read_unaligned()`; for packed flex-arrays `*(addr_of!((*x).u.List) as *const T).add(i)`. Do NOT
  remove `#[repr(packed)]`.
- **E0745 address of temporary** (`&(a+b)`, `&call()`, `&CONST`): bind to a local first
  (`let t = <expr>; ... &t`), value unchanged.
- **E0605 non-primitive cast** (fn pointers): `core::mem::transmute`, not `as`.
- **E0381 used-uninitialized:** init at declaration with the C-guaranteed default
  (`= core::mem::zeroed()` / `= 0`); don't restructure control flow.
- **E0428 defined multiple times:** stitching duplicated a fn/const — delete the redundant copy,
  keep one complete definition.
- **E0004 non-exhaustive match:** add the MISSING real arms with the C `switch`'s behavior — do NOT
  hide a case with a bare `_ => {}` unless the C `default:` truly does nothing.
- **FIELD_OFFSET(t,f) → `core::mem::offset_of!(t, f)`.** **C debug-assert macros (ASSERTMSG-style,
  compiled out in release) → no-op** (delete the call or `let _ = (args);`).

## STOP / DONE
If no progress on a class for 2 rounds, OR a genuinely-missing const/enum/type appears, STOP and
report the exact sites + errors verbatim so the engineer can supply the real definition or a global
rule. Otherwise drive `error_count` → 0 and report the final count + any residual classes.

## CLASSPNP-SPECIFIC (read before turn 1)
- **NEVER edit `types.rs`, `types_i386.rs`, `types_amd64.rs`, `types_arm64.rs`.** These are
  bindgen-generated EXACT struct layouts. They are correct. If a field access fails, the bug is in
  the MODULE's use site (wrong path, wrong cast, anon-union rename), never the struct. Re-stubbing
  these structs is the exact regression this baseline fixed.
- **Type sources:** driver-PRIVATE structs (FUNCTIONAL_DEVICE_EXTENSION, TRANSFER_PACKET, CLASS_*,
  CDB, SCSI_*, WNODE_*, MEDIA_CHANGE_DETECTION_INFO ...) come from `crate::types::*` (already
  imported in every module). COMMON kernel types (DEVICE_OBJECT, IRP, IO_STACK_LOCATION, MDL, GUID,
  KEVENT, DRIVER_OBJECT, HANDLE) and kernel FUNCTIONS come from `ntkrnl_sys::driver_ddk::*` or the
  curated `ntkrnl_sys::{ddk,base,ke,ex,rtl,interlocked,mm,ob,constants,globals}` modules.
- For "cannot find type DEVICE_OBJECT/IRP/..." → add `use ntkrnl_sys::driver_ddk::*;` (or the
  specific `ntkrnl_sys::ddk::{Type}`) to that module. Do NOT add a local definition.
- Enum variants (BusTypeAtapi, PowerDeviceD0, StorageDeviceProperty, GenericEqual ...) and DPFLTR_*
  ids that fail to import are bindgen enum constants — find them in `ntkrnl-sys/src/bindgen_i386.rs`
  (they may be assoc consts on the enum type, e.g. `_STORAGE_PROPERTY_ID_StorageDeviceProperty`).
- No `glue.rs` exists yet. If a constant/enum/static (TRACE_LEVEL_ERROR, ClassGlobalDispatch ...) is
  genuinely missing from ntkrnl_sys, STOP and report it (rule 3) — the engineer adds it.

## CRITICAL IMPORT RULES (learned from turns 1-2 — violating these REGRESSES the build)
1. **NEVER add a `use ...::*` glob import** (e.g. `use ntkrnl_sys::driver_ddk::*;`). The modules
   already glob several overlapping ntkrnl_sys submodules; ADDING another glob creates E0659
   "ambiguous" cascades (turn 2 added driver_ddk globs → +700 errors). To bring a missing symbol,
   add an **explicit named** import: `use ntkrnl_sys::ddk::{TheName};`. Explicit wins over globs → no
   ambiguity.
2. **Driver-private structs are in `crate::types`, NOT `ntkrnl_sys::ddk`.** For
   `cannot find type FUNCTIONAL_DEVICE_EXTENSION / CLASS_* / TRANSFER_PACKET / CDB(if private) / ...`,
   or `... in module ntkrnl_sys::ddk` for such a type, fix the path to `crate::types::TheName`
   (or rely on the module's existing `use crate::types::*`). NEVER define the struct locally.
3. **Kernel types** (DEVICE_OBJECT, IRP, KEVENT, MDL, IO_WORKITEM, STORAGE_*, WNODE_*) come from
   `ntkrnl_sys::ddk` — import them explicitly: `use ntkrnl_sys::ddk::{DEVICE_OBJECT, ...};`.
4. The generated `types_*.rs` are CORRECT (E0365 strip done). Never edit them; never re-stub structs.
