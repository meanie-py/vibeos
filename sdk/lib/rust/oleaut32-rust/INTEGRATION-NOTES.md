# oleaut32-rust integration burn-down — running notes

Lane: drive the assembled crate (rustconv hash/oleaut/safearray/variant/usrmarshal
+ DeepSeek vartype + fable-lane olesys/dispatch/typelib) to 0 errors ×3 targets,
faithfully to `dll/win32/oleaut32/*.c`.

## Baseline
- Start of lane: 853 errors (x86_64-pc-windows-gnu), vartype.rs not yet landed.
- The crate requires `RUSTC_BOOTSTRAP=1` (or nightly) for `#![feature(c_variadic)]`
  (debug_shims variadic TRACE/WARN/ERR/FIXME) — same as the real build path
  (sdk/cmake/gcc.cmake RUSTUP_TOOLCHAIN=nightly). A plain-stable `cargo check`
  reports E0554; that is NOT a crate bug.

## Shared-surface work (done centrally)
- **consts.rs fixes (generated values that couldn't compile):**
  - `R4_MAX/R4_MIN/R8_MAX/R8_MIN` → `f64` with the exact C literals
    (variant.h:71-74; the generator had typed them i64/u32).
  - `UI8_MAX` (overflowed i64) and `DECIMAL_POS` (dup of winemacros' u8 version)
    removed — winemacros.rs carries the correct ones.
  - Hand additions below the generated block: `VT_HRESULT/VT_PTR/VT_UINT_PTR`,
    `DESCKIND_FUNCDESC/VARDESC/TYPECOMP/IMPLICITAPPOBJ` (values from com-sys
    bindgen VARENUM_/tagDESCKIND_ consts).
- **commacros_gen.rs (NEW, generated):** COBJMACROS-shaped call helpers
  (`IFoo_Method(p, ...)`) for the 26 interfaces / 55 methods the translations
  call as free functions. Generated from com-sys bindgen vtbl structs by
  scratchpad `gen_commacros.py`; bodies call through the vtbl slot with
  `unwrap_unchecked` (Option<fn> niche = NULL slot faults at the call, exactly
  like C). `IRunningObjectTable_GetObject` maps to bindgen slot `GetObjectA`
  (wingdi's `#define GetObject GetObjectA` leaked into bindgen).
- **prelude.rs additions:** `LPCSTR` alias; CRT externs (memcpy/memmove/memset/
  memcmp/wcslen); kernel32/user32/gdi32/advapi32/ole32 externs (all WINAPI =
  `extern "system"`, prototypes from sdk/include/psdk); in-DLL C-side symbols
  (OLEAUTPS_* from the widl proxy unit, CreateProxy/StubFromTypeInfo from
  tmarshal.c, `_get_STDFONT_CF`/`_get_STDPIC_CF` cdecl per oleaut.c:1056);
  Wdtp InterfacePointer usrmarshal helpers (ole32, __RPC_USER = stdcall).
- **winemacros.rs additions:** V_RECORD_ref/V_RECORDINFO_ref (oleauto.h:208-209),
  IsEqualGUID/IID/CLSID inline fns, MAKELONG, CopyCursor (winuser macro → CopyIcon),
  DOS_DATE/DOS_TIME (variant.c:1104-1105), `ALIGN_LENGTH!`/`ALIGN_POINTER!`
  macro_rules (usrmarshal.c:42-45 are assignment macros).

## vartype.rs wiring (DeepSeek translation landed 2026-07-11 23:50)
- Copied from ~/vibeos-rust-staging/oleaut32-rustconv/vartype.rs via
  scratchpad `integrate_vartype.py`: header swap, bindgen anon-path
  normalization (DeepSeek used windows-sys-flavor single-level `.__bindgen_anon_1.X`
  paths; VARIANT fields → `.n1.n2.n3.X`, DECIMAL scale/sign/Lo32/Mid32 get their
  extra union level, CY `.int64` is direct), extern-strip of in-crate-provided
  names, V_*/DEC_* lvalue transform, VT-pair match-literal transform.
- **DeepSeek defects found + fixed at integration:**
  - The entire `_Var*` inline conversion family (vartype.c:150-262, generated in
    C by SIMPLE/NEGTST/POSTST/BOTHTST macros — 78 functions) was left as extern
    decls. Generated faithfully by the integrate script; comparisons emulate
    C usual-arithmetic-conversions via i128 (verified per instance: all bounds
    non-negative or compared signed-vs-signed in C, so i128 math == C result).
  - `VarR8FromDec` was emitted as an orphaned body with no signature (top-level
    `unsafe { ... }` block) — re-wrapped by hand against the C prototype,
    DECIMAL field paths fixed, `highPart` made mut.

## Per-file burn-down
- **safearray.rs — 0 errors ×3 targets (done, by lane owner).** Notable fixes:
  local extern block deleted (CC_MISMATCH sweep — Sys*/VariantCopy in-crate
  extern "system", IUnknown_*/IRecordInfo_* now commacros wrappers, CoTaskMemAlloc
  had wrong ULONG prototype); `rgsabound` ([T;1] flexible array) decays via
  `.as_ptr()/.as_mut_ptr()` where C used it as a pointer (SafeArrayPtrOfIndex,
  SafeArrayRedim `oldBounds`); C implicit conversions made explicit
  (cDims u16→UINT returns/args, DWORD→VARTYPE truncation in SafeArrayGetVartype,
  ULONG→SIZE_T at CoTaskMemAlloc, byte counts → usize at ZeroMemory);
  RecordCopy const→mut arg (PVOID in the COM signature).
- **variant.rs / vartype.rs / usrmarshal.rs / hash.rs+oleaut.rs** — delegated to
  four parallel burn-down agents (same faithfulness rules; file-scoped; shared-
  surface requests routed back here). Status: in flight.

## Status snapshot at shutdown (2026-07-12, stopped early for token budget)
Per-file errors, x86_64-pc-windows-gnu (from 853 total at lane start):
- **safearray.rs: 0** (done ×3 targets — shared files also clean on i686/aarch64).
- **hash.rs: ~0** (agent done: ~95 file-local `LANG_*` u16 consts added, verified
  vs ntbasedef.h incl. __REACTOS__ extensions; all 12 Lookup tables verified
  byte-for-byte vs hash.c, Lookup_64 zero-pad confirmed; SUBLANGID/HeapAlloc casts).
- **variant.rs: 89** (was 238; agent partial). CC_MISMATCH swept, V_UNION leftovers
  → real brecVal accesses, VARIANT_NUMBER_CHARS + VT_X_VT_BYREF case consts defined
  (2 were silently-wrong catch-all patterns), FITS_AS_* inlined 64-bit to avoid
  truncation, several compile-clean semantic bugs fixed (SafeArrayCopy value-cast,
  VarSub UI1 wine quirk preserved). Remaining ~89 mapped: &mut sweep in
  VarMul/Sub/Idiv/Imp, deref-drop out-params, I*_MIN/R4_MIN typed compares, VarMod
  `resT: i32`. FIDELITY GAP (pre-existing, compile-clean, unfixed): VarAnd/VarImp
  translate C `goto X_Exit` as no-op `loop{break}` — failure paths fall through
  instead of jumping to cleanup; needs a labeled-block rewrite.
- **oleaut.rs: 72** (was 51; agent partial, mid-edit churn — externs swept,
  HKEY_CLASSES_ROOT sign-extended per winreg.h:10 (0xFFFFFFFF80000000, NOT
  zero-extended), 11 external GUID statics kept). ~13 remaining fixes fully
  diagnosed in the agent's plan (vtbl `Some(...)` wrap + `*mut` lpVtbl cast,
  bstr-cache offset_of arithmetic, month-array size-by-one corrections vs C,
  pointer-table statics → `static mut` + addr_of).
- **vartype.rs: 100 / usrmarshal.rs: 110** — agents still running at shutdown;
  output lands in-tree (work preserved per repo rules).
- **winemacros.rs: 1** — ALIGN_POINTER! expansion needs the usrmarshal call sites'
  first arg to be a `mut` place (the agent's job — the C macro mutates its by-value
  local `Buffer`/`Pos`, so making the fn param `mut` is the faithful fix).

## SHARED-SURFACE REQUESTS still owed (from agents)
- `VarBstrFromDec` and `VarCyFromUI2` are called by variant.rs (VARIANT_Coerce) but
  translated NOWHERE — vartype.c has them; the DeepSeek vartype.rs may be missing
  those two `#[no_mangle]` exports. Verify vartype.rs defines them; if not, they need
  translating/adding. (2 variant.rs errors block on this.)

## write_bytes(sizeof²) lint hits — full inventory (fix to match C memset byte count)
scan_silent_failures flags 6 WRITE_BYTES_SIZEOF: usrmarshal.rs:407,438 (task brief)
PLUS vartype.rs:2430,2829,2866,2887 (4 more — relayed to the vartype agent).

## Verified count at handoff: 853 → 372 (x86_64), per file:
usrmarshal 110, vartype 100, variant 89, oleaut 72, winemacros 1 (=usrmarshal's
ALIGN_POINTER! `mut` param); safearray 0, hash ~0.

## SIGNSCALE is CORRECT (do not "fix"): variant.h:85 is the little-endian branch
`((scale) << 8) | sign` — winemacros.rs matches it; all 3 targets are LE. (A
burn-down agent misread the BE branch at :87.) The DecFromDI signscale-order
question is a vartype.rs call-site issue, not a winemacros bug.

## usrmarshal.rs — NOT YET EDITED (agent did analysis only). Resume plan:
- `LPSAFEARRAY_UserMarshal`, `elem_wire_size`, `elem_mem_size` were DROPPED by the
  translation (C lines 798-862, 972-1118) — must be WRITTEN, not just declared.
- The 3 write_bytes sites here (407/438/1407) are actually already byte-correct
  (pointee u8 / count 1) — the lint fires but re-check each vs C before "fixing".
- Real semantic bugs to fix while translating: line ~1118 uses `& 0x0040` where C
  uses FADF_HAVEVARTYPE (0x80; 0x40 is FADF_HAVEIID); unmarshal `sftype` match uses
  0x20 for SF_HAVEIID (real 0x800D); V_DISPATCH unmarshal casts pointer *values* to
  `**` instead of taking addresses (~800/808).
- `interface_variant_size` → `__REACTOS__` `interface_user_size(pFlags,Start,riid,punk)`
  form; collapse the cfg-dup to the REACTOS branch. `IPropertyBag_RemoteRead_Proxy`
  takes `DWORD varType` not VARTYPE. Missing *_Proxy externs are all extern "system"
  (prototypes in build-amd64/sdk/include/psdk/oaidl.h/ocidl.h). SafeArrayCreateEx is
  still C-side (safearray.c) — file-local extern "system".

## vartype.rs — partial edits landed; resume plan (from agent):
- `.ceil()/.floor()` method→free-fn; `_CY`/`_DECIMAL`→`CY`/`DECIMAL`; CY `.s.Lo/.Hi`
  → `.__bindgen_anon_1` paths; `std::`→`core::`.
- MISSING translations to add (block variant.rs too): `VARIANT_DI_normalize`
  (vartype.c:5260), and confirm `VarBstrFromDec`/`VarCyFromUI2` exist as exports.
- Semantic bugs found: `VarI4FromDisp` passes vt=VT_I2 (should VT_I4); `VarUI4FromStr`
  passes VT_UI2 (should VT_UI4); a 5th write_bytes at ~2636 (`n*4` should be `n`).
- VarDecInt/Neg/Round, VarCyFromDec have garbled DEC accessor paths — rewrite vs C.

## variant.rs — partial (238→89). Remaining mechanical: `&x`→`&mut x` in
VarMul/Sub/Idiv/Imp; deref-drop out-params; I*_MIN/R4_MIN typed compares; VarMod
`resT: i32`. FIDELITY GAP: VarAnd/VarImp render C `goto X_Exit` as no-op
`loop{break}` — failure paths skip cleanup; needs labeled-block rewrite.

## oleaut.rs — partial (churn to 72). ~13 fixes diagnosed: vtbl `Some(...)` + `*mut`
lpVtbl cast; bstr-cache offset_of arithmetic; month-array size-by-one vs C;
pointer-table statics → `static mut` + addr_of. HKEY_CLASSES_ROOT already added
SIGN-EXTENDED (0xFFFFFFFF80000000 per winreg.h:10) — keep that, not zero-extend.

## Known-real translation bugs tracked from the task brief
- usrmarshal.rs:407/:438 `write_bytes(ptr, 0, size_of::<T>())` — zeroes size_of²
  bytes (write_bytes counts elements). Assigned to the usrmarshal agent; verify
  the fix matches the C memset byte counts.
- CC_MISMATCH class (module-local `extern "C"` decls of in-crate
  `extern "system"` definitions — i386-real stdcall/cdecl break): swept in
  safearray (done), assigned per-file to the agents; widl-generated *_Proxy
  externs stay but as `extern "system"`.
