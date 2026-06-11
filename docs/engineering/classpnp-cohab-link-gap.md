# classpnp.sys cohab link gap (i386, 2026-06-10)

Status: classpnp-rust **compiles cold-clean ×3** (i386/amd64/arm64, committed
`fd07c31`+`HEAD`) and **`libclasspnp_rust.a` builds clean in release**. The
cohab **link** of `classpnp.sys` then fails with **166 undefined symbols** —
the DDK substrate + ABI + translation-gap layer.

Build infra: the classpnp worktree's `build-i386` was created by copying the
rtl-tpool worktree's warm `build-i386` and repointing paths (`sed
rtl-tpool→classpnp` across 1506 files) + symlinking `.build-tools` from main.
`ninja rust_classpnp` works; `ninja classpnp` is the failing link.

## The 166, bucketed

| Bucket | N | Nature | Fix |
|---|---|---|---|
| macro/inline-helper | 19 | `TEST_FLAG`,`FREE_POOL`,`NT_ASSERT`,`REVERSE_BYTES*`,`PAGED_CODE`,`PORT_ALLOCATED_SENSE`,`Int64ShrlMod32`… — C macros/FORCEINLINEs declared `extern "C"` in shims.rs with no symbol | implement as `#[no_mangle]` fns (or macro-ize call sites) |
| srb-accessor | 24 | `SrbGetCdb`,`SrbSetDataBuffer`… storport.h inlines | `#[no_mangle]` wrappers over srblib.rs |
| io/ke/mm-inline | 31 | `IoGetCurrentIrpStackLocation`,`KeGetCurrentIrql`,`InterlockedCompareExchange`,`MmGetMdlVirtualAddress`… | `#[no_mangle]` wrappers over `ntkrnl_sys::inline::{io,ke,mm}` |
| wpp-trace | 13 | `TracePrint`,`WPP_INIT_TRACING`,`DbgGet*Str`… | no-op / DbgPrint shims |
| sense-inline | 12 | `ScsiGetSenseDescriptor`,`IsDescriptorSenseDataFormat`… scsi.h inlines | `#[no_mangle]` impls |
| real-export | 19 | `ExFreePool`,`EtwRegister`,`PoRegisterPowerSettingCallback`… real ntoskrnl/hal exports | **ABI fix**: extern block is `extern "C"` (emits `_ExFreePool`) but i386 export is stdcall `_ExFreePool@4` → make `extern "system"`; verify importlibs on link line |
| intra-classpnp: ABI-fix | 20 | real defs in-crate (`ClasspGetCurrentTime`,`ClassRemoveChild`,`RetryRequest`…) but call decoration ≠ def decoration | `#[no_mangle]` + unify ABI (extern "system"); run verify_abi.py |
| **intra-classpnp: UNTRANSLATED** | **28** | **genuinely missing** — never translated | **needs paid translation ($)** |

~138 of 166 are **mechanical** (shim impls + ABI normalization, scriptable,
no API spend). The remaining **28 need paid translation**.

## The 28 untranslated (need $ + user approval)

Mostly class.c residue + untranslated data.c/guid.c/dictlib.c:

ClasspModeSelect, ClasspModeSense (class.c), ClasspStartIo@8, ClasspReleaseQueue,
ClassAddChild, ClassRemoveChild residue, ClassPnpQueryFdoRelations,
ClassDetermineTokenOperationCommandSupport, ClassGetPdoId,
ClasspBuildRequestEx, ClasspAllocateReleaseRequest, ClasspFreeReleaseRequest,
ClasspUpdateDiskProperties@8, ClasspRetryRequestDpc@16, ClassRetryRequest,
ClasspGetInquiryVpdSupportInfo, ClasspGetLBProvisioningInfo,
ClasspValidateOffloadSupported, ClasspBlockLimitsDataSnapshot,
ClasspRegisterMountedDeviceInterface, ClassDeviceControlDispatch@8,
ClassLogResourceExhaustionEvent@8, ClassLogThresholdEvent@8,
AllocateDictionaryEntry (dictlib.c),
ClassGuidQueryRegInfoEx, ClassGuidSenseInfo2, ClassGuidSrbSupport,
ClassGuidWorkingSet (data tables — data.c/guid.c).

## Plan

1. **(needs approval, $)** Translate the 28 missing fns — `rustconv translate`
   on the specific class.c functions + data.c/guid.c/dictlib.c. Est. small
   (most are short); the 4 ClassGuid* are static data tables.
2. **(mechanical, no $)** Build the shim substrate: ~80 `#[no_mangle]` inline
   wrappers (io/ke/mm/srb/sense/macro) + no-op wpp + fix the 19 real-export
   extern blocks to `extern "system"` + `#[no_mangle]`-ify the 20 ABI-mismatch
   intra defs. Leverage `ntkrnl_sys::inline::*`, srblib.rs, and verify_abi.py.
3. Re-link `ninja classpnp`; iterate to 0 undefined.
4. Update boot media + 3-arch smoke (classpnp is boot-critical: IDE/CD path).

`/tmp/undef.txt` holds the full symbol list (regenerate via the link cmd with
`-Wl,--error-limit=0`).

## UPDATE 2026-06-10 (later): translations done, substrate pending

**Translation of the 28 untranslated fns: 20 DONE ($0.33), banked.**
- Output: `tools/rustconv/classpnp-gap-staging/{class_gap.rs(15), class_gap2.rs(4), ClasspBuildRequestEx.rs(1)}`
  and per-function `.responses/` (moved into this worktree).
- **Giant-file parser solved**: class.c tree-sitter truncates at the first PSEH
  `_SEH2_TRY` (~line 3768). Fix = extract target fns into a standalone "gap TU"
  with: (a) comment-aware def-finding (skip doc-comment `Fn()` matches inside
  `/*++ --*/`), (b) strip inline `/* */` (the `NTAPI /*…*/` trailing-comment
  broke the return-type back-walk), (c) strip `#ifdef _MSC_VER/#pragma` blocks,
  (d) strip standalone SAL lines (`_IRQL_requires_*`, `IN/OUT/OPTIONAL`).
  Each gap TU then catalogs cleanly and translates via plain LLM (no c2rust —
  the gap file isn't in compile_commands.json).

**4 fns still need hand-translation** (resist cataloging — PSEH `TRY/LEAVE` or
deep SAL): `AllocateDictionaryEntry` (dictlib.c:78, uses TRY/LEAVE),
`ClassLogThresholdEvent` (utils.c:5141, 211 ln), `ClassLogResourceExhaustionEvent`
(utils.c:5430, work-item logger), `ClasspValidateOffloadSupported` (utils.c:7092).
Plus 4 static **data tables** to transcribe: `ClassGuid{QueryRegInfoEx,SenseInfo2,
SrbSupport,WorkingSet}` (data.c/guid.c — GUID structs/arrays).

## Remaining work (in order)

1. **Hand-translate** the 4 fns + transcribe 4 data tables.
2. **Integrate** all 24 fns into a new `class_part_c.rs` module:
   - one import head (mirror `class_part_b.rs`), module-local extern union,
   - **set each fn's ABI from its undef decoration** (`@N` → `extern "system"`,
     bare → `extern "C"`): stdcall set = {ClassRemoveChild, ClasspStartIo,
     ClasspRetryRequestDpc, ClasspUpdateDiskProperties, ClassDeviceControlDispatch,
     ClassLogResourceExhaustionEvent, ClassLogThresholdEvent}; rest cdecl,
   - drop the duplicate DllUnload/preamble, `pub mod class_part_c;` in lib.rs,
   - cargo check cold ×3 → 0.
3. **DDK substrate** — implement the 95 (`/tmp/cls_B.txt`) as `#[no_mangle]`:
   - IRP-stack (7): wrap `ntkrnl_sys::inline::io::*`,
   - SRB accessors (24): impl over SRB_TYPE/storport layout (see srblib.rs),
   - SCSI sense (12): scsi.h inlines,
   - macros (15): TEST_FLAG/FREE_POOL/REVERSE_BYTES*/etc (some already in macros.rs — provide fn forms),
   - trace/ETW (18): no-op / DbgPrint,
   - Ex timer / lookaside / Po / Simple-SLIST / Rtl-safe / Mm / Ke-node (rest).
4. **13 real exports** (`/tmp/cls_A.txt`): flip extern decls to `extern "system"`
   so i386 gets `@N` (currently `extern "C"` → `_ExFreePool` ≠ export `_ExFreePool@4`).
5. **48 intra-classpnp**: 20 are ABI-mismatch on existing defs (`#[no_mangle]` +
   unify ABI via verify_abi.py); 28 satisfied by step 2's translations.
6. `ninja rust_classpnp && ninja classpnp` → iterate to 0 undefined → boot media → smoke ×3.

Build dir: this worktree's `build-i386` (copied+repointed from rtl-tpool, toolchain
symlinked from main). `ninja rust_classpnp` works.
