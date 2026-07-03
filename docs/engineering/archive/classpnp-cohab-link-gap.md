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

## UPDATE 2026-06-10 (evening): integration complete, link 166→45

**All 24 missing fns translated ($0.37) + integrated into `class_part_c.rs`;
DDK substrate built as `ddk_shims.rs` (95 inline/macro/srb/scsi helpers).
classpnp-rust compiles COLD-CLEAN on all 3 arches.** Cohab link of classpnp.sys
now has **45 undefined** (was 166):

- `class_part_c.rs` (19 fns) + `ddk_shims.rs` (substrate) integrated; per-fn ABI
  set from undef decoration; fix-tool suite (fix_field_paths/bitfield/type_casts)
  + ~60 hand-fixes (CommonExtension offset-0 cast, IoStatus/IRP anon-union paths,
  LARGE_INTEGER.QuadPart, bitfield setters, IncompleteArrayField indexing, etc).
- **rustc won't emit a `#[no_mangle]` fn whose name also has a `#[macro_export]`
  macro** (REVERSE_BYTES) — implemented those 3 in C in `rust_cohab_stub.c`.
- x86 `lock xchg`/`cmpxchg` in class_part_c → portable AtomicI32 (arm64-safe).
  NOTE: `debug.rs:146` still has an un-gated `lock xchg` → will fail arm64 BUILD
  (cargo check passes; codegen doesn't validate mnemonics). Gate it before arm64 smoke.

### Remaining 45 undefs (see /tmp/undef5.txt)
1. **Kernel intrinsics/exports (~14)**: ExFreePool (→ExFreePoolWithTag macro),
   Interlocked{CompareExchange,Decrement,Exchange} (i386 intrinsics), KeInitializeSpinLock
   (inline *lock=0), KeGetCurrentIrql/KeRaiseIrql/KeLowerIrql/KeQueryTickCount,
   IoFreeMdl/IoStartNextPacket/IoSetHardErrorOrVerifyDevice/IoWMIDeviceObjectToProviderId.
   ⚠️ i386 ABI quirk: relocating their decls to `extern "system"` did NOT change the
   reference from cdecl `_X` to stdcall `_X@N` — needs investigation (caller may resolve
   to a bindgen ntkrnl_sys::ddk decl, or these are genuinely cdecl/intrinsic). Provide as
   shims (ddk_shims or C stub) matching the EXACT undef decoration.
2. **Hand-translate (4)**: AllocateDictionaryEntry (dictlib.c, TRY/LEAVE), ClassLogThresholdEvent,
   ClassLogResourceExhaustionEvent (utils.c), ClasspValidateOffloadSupported (utils.c).
3. **Data tables (4)**: ClassGuid{QueryRegInfoEx,SenseInfo2,SrbSupport,WorkingSet} + the
   MaxTokenOperationListIdentifier data symbol — transcribe from data.c/guid.c.
4. **Intra-ABI / more-missing (~20)**: some DEFINED in-crate but decoration-mismatched
   (ClasspGetCurrentTime, ClassAcquireRemoveLock, RetryRequest, InterpretSenseInfoWithoutHistory,
   ClassFreeOrReuseSrb, IsKeyReadCopyNumber, ClasspIsIdleRequestSupported) → set def ABI to
   match undef decoration; others are MORE untranslated internal helpers not in the original 28
   (ClasspTransferPacketGetCdb/SenseInfoBuffer/Length, ClasspTimeDiffToMs, ClasspSrbSetOriginalIrp,
   ClasspMarkIrpAsIdle, ClasspIsIdleRequest, ClassRetrieveDeviceRelations, ClasspRetryDpcTimer,
   ClasspAllocateReleaseQueueIrp, ClasspGetBlockDeviceTokenLimitsInfo, ClasspMediaChangeRegistryCallBack,
   ClasspRefreshFunctionSupportInfo) → translate (gap-TU technique) or shim the small accessors.

Iterate: fix → `ninja rust_classpnp && ninja classpnp` → recount → boot media → smoke ×3.

## UPDATE 2026-06-11: LINK COMPLETE (166→0) + runtime BUGCHECK surfaced

**The cohab link is DONE: classpnp.sys links with 0 undefined symbols on i386.**
classpnp-rust compiles COLD-CLEAN on all 3 arches (i386/amd64/arm64). All 18 C
sources are now Rust. `ninja classpnp` → classpnp.sys (720KB).

How the final 45→0 was closed (this session, no reverts):
- **Substrate shims (ddk_shims.rs)**: 8 classp.h FORCEINLINE helpers
  (ClasspTransferPacketGet{Cdb,SenseInfoBuffer,SenseInfoBufferLength}, ClasspGetCurrentTime,
  ClasspTimeDiffToMs, ClasspMarkIrpAsIdle, ClasspIsIdleRequest{,Supported}, ClasspSrbSetOriginalIrp),
  IsKeyReadCopyNumber, ClassAcquireRemoveLock(macro→Ex); Interlocked{CompareExchange,Decrement,
  Exchange}+KeInitializeSpinLock (i386 intrinsics, AtomicI32/`*lock=0`); cdecl wrappers over the
  real stdcall exports ExFreePool/IoFreeMdl/IoStartNextPacket/IoSetHardErrorOrVerifyDevice/
  KeGetCurrentIrql/KeRaiseIrql/KeLowerIrql/KeQueryTickCount (IRQL via tier1 inline; KeQueryTickCount
  private extern; IoWMIDeviceObjectToProviderId stubbed on i386 — not exported there).
- **C cohab stub**: `_Analysis_assume_` (i386 no_mangle underscore for a leading-`_` name is
  unreliable; C gives the exact per-arch symbol). Joins the REVERSE_BYTES* family.
- **Data symbols**: `#[no_mangle]` added to data.rs ClassGuid{QueryRegInfoEx,SenseInfo2,SrbSupport,
  WorkingSet} + class.rs MaxTokenOperationListIdentifier (were defined but mangled).
- **11 untranslated fns** (gap-TU, $0.25 Sonnet): RetryRequest, InterpretSenseInfoWithoutHistory,
  ClassFreeOrReuseSrb, ClasspValidateOffloadSupported, ClassRetrieveDeviceRelations,
  ClasspGetBlockDeviceTokenLimitsInfo, ClasspRefreshFunctionSupportInfo,
  ClasspMediaChangeRegistryCallBack, ClassLog{ResourceExhaustion,Threshold}Event → class_part_d.rs;
  AllocateDictionaryEntry hand-translated into dictlib.rs (PSEH TRY/LEAVE).
  - **Gap-TU extractor lessons** (tools/rustconv, /tmp/gapextract.py + /tmp/salstrip.py): tree-sitter
    truncates on (a) the ReactOS doc-comment `Fn()` line inside `/*++ --*/` — must be comment-aware,
    (b) inline `/* */` on the `NTAPI /*…*/` line breaking the return-type walk-back — pre-strip
    single-line comments, (c) inline SAL tokens `_In_`/`_In_z_`/`_IRQL_requires_max_(...)`/
    `_Function_class_(...)` in signatures, (d) call-site false matches (reject `=`/`return`).
  - Integration: reuse class_part_c's import head; TracePrint→no-op macro; NT_ASSERT→`!`-macro;
    fix-tools (fix_bitfield_methods/type_casts/arg_casts) auto-cleared 52 cast errors via plain
    `cargo check` (default toolchain finds i686-pc-windows-gnu std; `+nightly` does NOT). Portable
    casts must be `as _` not `as u32`/`as SIZE_T` (SIZE_T alias ≠ the bindgen param's concrete int).

### ⚠️ NEXT PHASE — runtime BUGCHECK 0x1E in ClassInitialize (open)
i386 smoke boots kernel→`IopInitializeBuiltinDriver: disk.sys`→ (disk.sys DriverEntry calls
classpnp `ClassInitialize`) → **BUGCHECK 0x1E (0xc000001d ILLEGAL_INSTRUCTION)** at a **`ud2`**.
Faulting site pinpointed: classpnp.sys `_ClassInitialize@12` +0x113ba (PC&0xFFF=0x3ba; load base
≈0x84c0a000). The ud2 sits at the **tail of the `IoAllocateDriverObjectExtension` success branch**,
right after `call RtlCopyUnicodeString` — LLVM eliminated the branch continuation (RtlCopyMemory +
ClassInitializeDispatchTables) as UB-unreachable. `ClassInitialize` is in **class.rs** (a PRE-EXISTING
translation that never linked/booted until now). `lint_kernel.py class.rs` = clean (not a simple
unprotected-output-write). Next: GDB-step ClassInitialize / bisect the success-branch tail (likely a
captures(none) or type-layout/size_of mismatch around `(*driverExtension).InitData` RtlCopyMemory or
ClassInitializeDispatchTables). This is the correct integrated-but-failing state per AGENTS.md.

## UPDATE 2026-06-11 (cont.): ClassInitialize fixed; SYSTEMIC fold discovered

**Root-caused + fixed the boot BUGCHECK 0x1E ud2:** it was in `ClassInitializeDispatchTables`
(dispatch.rs), which release-opt folded to a whole-function `unreachable` (→ ud2). Mechanism:
- `&mut (*DriverExtension).DeviceMajorFunctionTable` inserts a `panic_misaligned_pointer_dereference`
  check, and the PAGED_CODE preamble `if KeGetCurrentIrql()>APC_LEVEL { … __int2c() }` interacts
  with `__int2c() -> !` (declared **noreturn** in retry.rs:29, a shared `#[no_mangle]` symbol) so
  the whole function folds; rustc then stamps `readnone captures(none)` on the (actually-written)
  pointer param.
- Fixes: (a) dispatch.rs uses raw-pointer `write_volatile` (no `&mut` reference) + dropped the
  debug-only IRQL/PAGED_CODE preamble; (b) retry.rs `__int2c()` decl changed from `-> !` to `()`
  (int 0x2c can return; noreturn poisons every caller); (c) translated `ClassShutdownFlush`
  (gap-TU $0.01) which became a LIVE reference once the dispatch table un-folded.
- Verified: ClassInitialize + ClassInitializeDispatchTables are now ud2-free; links 0 undefs ×3-clean.

### ⚠️ SYSTEMIC release-opt fold (~46 functions) — the real boot blocker, OPEN
Disassembly shows **~46 classpnp functions are whole-function `unreachable` (push;mov;ud2)** at the
shipped `opt-level=1` — including dispatch handlers ClassCreateClose, ClassSystemControl,
ClassDeviceControlDispatch, ClassAddDevice, ClassMinimalPowerHandler, ClasspMcnControl, the
property/DSM handlers, transfer-packet helpers, etc. Each crashes when called, so boot dies at the
first one reached after init. Cause = the same class of miscompile: translated code creates
`&(*ptr).field` references whose rustc reference-UB-checks (align-8 on i386 + null) combine with the
wrongly-inferred `readnone captures(none)` param attribute so LLVM folds the whole body. The
workspace firewall (lto=false, cgu=256) stops the CROSS-function exploit but NOT this intra-function
fold.

**Proven workaround + the catch:** a per-package `[profile.release.package.classpnp-rust] opt-level=0`
override un-folds ALL 46 (verified: 0 non-shim whole-fn-ud2). BUT un-folding makes ~**38 previously-
DEAD references live** → 38 new link undefs (so it no longer links). They split into: (a) ~15 shim
ABI-decoration mismatches now exposed (e.g. `_InterlockedCompareExchange@12`/`_SrbGetCdb` cdecl-vs-
stdcall, NT_ASSERT, SET_FLAG, GET_FDO_EXTENSON_SENSE_DATA_LENGTH) — fix the ddk_shims/C-stub
decoration; (b) ~15 more untranslated helpers (ClassAcquireChildLock, ClasspIsPortable,
ClasspScanForSpecialInRegistry, ClasspAllocatePowerProcessIrp, ClassReleaseQueue[Completion],
ClasspRetryDpcTimer, ClasspSendSynchronousCompletion, …) — translate; (c) ~5 data symbols
(ClassBadItems, CountOfSetBitsUChar, ClasspIsOffload/TokenOperation, DBGGETIOCTLSTR).

So the opt-1 "0 undefs" is partly an artifact of the fold hiding 38 references. Reverted the opt-0
override to keep the OS LINKABLE at opt-1 (folded but builds). **Next phase = the proper fix:** either
(i) rewrite the `&(*ptr).field` reference patterns to raw-pointer reads crate-wide (kills the UB-check
fold without opt-0), or (ii) keep opt-0 and close the 38 surfaced undefs. Path (i) is cleaner and
doesn't regress codegen. The fold list + 38-undef list are in /tmp/undef_opt0.txt.

## UPDATE 2026-06-11 (cont.): SYSTEMIC FOLD ROOT-CAUSED & FIXED (not the reference theory)

The "~46 whole-function folds" were **misattributed** to the `&(*ptr).field`
reference-UB-check class. Disproof: `ClassCreateClose` is clean raw-pointer code
(no `&(*ptr).field`) yet folded to `start: unreachable`. LLVM IR archaeology
(`-print-changed -filter-print-funcs`) showed InstCombine inserting
`store i1 true, ptr poison` right after a call — i.e. it proved the **call** UB.

**Real root cause: declaration/definition calling-convention mismatch.** On i386,
`extern "C"` = cdecl (`_F`), `extern "system"` = stdcall (`_F@N`). When a caller
declares `extern "C" fn F` but the in-crate definition is `extern "system"` (or
vice-versa), LLVM sees `call ccc @F` against an `x86_stdcallcc` callee in the same
module → undefined behaviour → it folds the **whole caller** to `unreachable`.
lld then *fuzzy-matches* the undecorated/decorated symbols so the cohab link still
succeeds (0 undefs) — which is exactly why the bug was invisible at link time and
only showed as ud2 at boot.

A def↔decl CC-consistency scan found **76 mismatches** across the crate (every
dispatch handler calls `ClassReleaseRemoveLock`/`ClassCompleteRequest`/`SrbGet*`
etc., many declared `extern "C"` in the glob-imported `shims.rs` while defined
`extern "system"`). Fixing all 76 (decl CC := def CC) un-folded 46 → 0.

Three follow-on classes the un-folding then exposed:
1. **Self-CC-collision wrappers.** A cdecl `#[no_mangle] fn ExFreePool` that calls
   `ntkrnl_sys::ddk::ExFreePool` (stdcall, *same IR name*) is calling itself with a
   mismatched CC → folds. The 6 such wrappers were removed; callers now call the
   real stdcall export directly. (Lesson: never wrap a kernel export under its own
   name; the IR symbol collides regardless of `@N` decoration differences.)
2. **Inlining interaction.** `ClassInitializeDispatchTables` is fold-free standalone
   but inlining it into `ClassInitialize` let LLVM combine its driverExtension
   dispatch-table writes with the caller's `RtlCopyMemory(InitData)` dereferenceable
   facts into a fold (ud2 after `RtlCopyUnicodeString`). Fix: `#[inline(never)]`
   (+ `extern "C"` to match the cdecl call site). Also made the `EtwHandle` (u64,
   align-8 on i386) access `read/write_volatile`.
3. **Newly-live references.** Un-folded bodies referenced 9 previously-dead symbols:
   6 hand-translated into `class_part_e.rs`, `IoQueueWorkItemEx` shimmed to
   `IoQueueWorkItem` (ReactOS lacks the Ex export), etc.

**Status: classpnp.sys links 0-undef ×3-clean; boot-critical path fold-free.**
Remaining: 1 fold in `ClassRemoveDevice` (PnP IRP_MN_REMOVE_DEVICE handler — off the
boot path; same inlining class, owed). The opt-level=0 escape hatch was NOT used —
the firewall stays opt-1.

## UPDATE 2026-06-11 (final): 3× SMOKE GREEN — classpnp Rust cohab boots on all arches

All three architectures boot to desktop with the all-Rust classpnp.sys
(`vibeos-cli smoke --arch <a>` = ✓ boot):
- **i386 ✓** (the systemic CC-mismatch fold fix was the unlock)
- **amd64 ✓** (substrate: 5 hand-translated class.c fns, SET_FLAG, cr8 KeGetCurrentIrql,
  KUSD KeQueryTickCount/SharedUserData; no folds — amd64 never had the i386 CC issue)
- **arm64 ✓** (KeGetCurrentPrcb inline forward + shared KUSD provisions)

Two *unrelated* stale-base build bugs were fixed along the way to build amd64/arm64
(the worktree is 178 commits behind continue-rust-port):
1. ntdll-rust `RtlInitEmptyUnicodeString` — #[inline(always)] didn't fire cross-cgu on
   amd64/arm64 under the workspace firewall, leaving an undefined out-of-line ref. Fixed
   by #[no_mangle]-exporting the rtl/libsupp.rs copy.
2. arm64 `livecd` ninja target has a `boot/freeldr/install/fat.c` self-dependency cycle —
   sidestepped (arm64 boots via mkramdisk over the staged set, not the ISO; build the
   media files via `ninja` + the 116 media/nls targets directly, then mkramdisk).

Smokes were warm (--allow-stale) with freshly-rebuilt classpnp.sys each arch. A --clean
×3 pass is the remaining final-verification step. Eventual merge to main needs a rebase
onto current continue-rust-port.
