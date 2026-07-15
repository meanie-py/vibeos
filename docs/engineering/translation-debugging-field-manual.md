# VibeOS Translation & Kernel Debugging Field Manual

A symptom-indexed reference for debugging C→Rust translation regressions **and** for hunting live kernel crashes (BUGCHECKs, hangs, heap corruption) in the ntoskrnl/win32k/HAL/freeldr port. **When a flip crashes, hangs, links-undefined, or smokes red, find the matching symptom here BEFORE you start instrumenting** — most of these cost a multi-hour debug session the first time and the root cause is rarely where the crash points.

Each entry is **Symptom → Root cause → Fix**. The `**Source:**` slug names the original per-session debug memory the entry was distilled from; **most of those memories have been consolidated into this manual and removed** — this doc is now the canonical, permanent home for the learning (full commit-by-commit narrative lives in `git log`). Sections 1–8 are translation-bug categories; section 9 is the live-crash-hunting technique layer. Sections:

1. [ABI / FFI / calling-convention / decoration](#abi--ffi--calling-convention--decoration) — the #1 i386 crash class
2. [LLVM captures(none) / ud2 / codegen-unit miscompiles](#llvm-capturesnone--ud2--codegen-unit-miscompiles)
3. [bindgen / struct-layout / string-literal width traps](#bindgen--struct-layout--string-literal-width-traps)
4. [rustconv translation-output bug catalog](#rustconv-translation-output-bug-catalog)
5. [Per-arch crash patterns (arm64 + amd64-only)](#per-arch-crash-patterns-arm64--amd64-only)
6. [Cohabitation, cold-link & stale-artifact false failures](#cohabitation-cold-link--stale-artifact-false-failures)
7. [CRT/cnt runtime, harness flakes & kernel one-offs](#crtcnt-runtime-harness-flakes--kernel-one-offs)
8. [win32k flip-specific debugging](#win32k-flip-specific-debugging)
9. [Kernel crash-hunting techniques (live BUGCHECK / hang / heap hunts)](#9-kernel-crash-hunting-techniques-live-bugcheck--hang--heap-hunts)

**Three meta-rules that recur across every section:**
- **A green i386 does NOT exonerate a flip.** Most amd64/arm64-only deterministic crashes are real bugs i386's narrower pointers / no-SSE / different layout luck-masks. Build and boot all three.
- **Check ABI first (1 second via `nm`/`verify_abi.py`).** A `ud2` after a call or a folded success path is an `extern "C"` vs `extern "system"` mismatch far more often than a captures(none) bug.
- **Cargo-check green ≠ link-clean ≠ runtime-correct.** Many defects only surface at the `.dll`/`.sys` link or at runtime; integrate, link all three arches, and smoke.

---

## ABI / FFI / calling-convention / decoration

The #1 i386 crash class: on i386, `extern "C"` is cdecl and `extern "system"` is stdcall/NTAPI — divergent decoration and lowering — while on amd64/arm64 they coincide, so these bugs link clean and boot fine on 64-bit and only blow up on i386 (or, for return-width/register-width variants, the reverse).

### NTAPI fn declared `extern "C"` → ud2 after the call (i386-only crash)
**Symptom:** BUGCHECK 0x1E with exception code `0xC000001D`; the function is folded to a tiny `ud2` (`0f 0b`) stub right after a call. Clean on amd64/arm64.
**Root cause:** On i386 `extern "C"` is cdecl, NTAPI is stdcall; the ABI inconsistency at the callsite lets LLVM prove the continuation unreachable. (Was "the entire i386 pre-existing regression" — device.rs, iomgr.rs, arcname.rs.)
**Fix:** All hand-written extern blocks for NTAPI kernel fns use `extern "system"`, not `extern "C"`; only true C-runtime fns (sprintf/wcslen/memset) stay `extern "C"`. Prefer `ntkrnl_sys::ddk::` bindgen imports. Grep `extern "C"` and audit before enabling Rust on i386.
**Source:** `feedback_extern_c_abi.md`

### Sibling rlib NTAPI fn `extern "C"` → i386 link fails `undefined symbol: _Name`
**Symptom:** i386 link error `undefined symbol: _Name` while the kernel exports `_Name@N`. amd64/arm64 link fine. (e.g. `KeSetEvent`, `IopAcquireFileObjectLock` from io-inlines.)
**Root cause:** `extern "C"` emits a bare `_Name` reference; the stdcall export is `_Name@N` — decoration mismatch is fatal at i386 link time.
**Fix:** Sibling rlibs (`*-inlines`, `*-macros`) must declare NTAPI fns `extern "system"`. Check each fn's `.h` decl: `NTAPI`/`NTKERNELAPI...NTAPI` → `extern "system"`. i386 fastcall fns (`Obf*`, `KfRaiseIrql`) need `#[cfg(target_arch="x86")] extern "fastcall"` else `extern "C"`. Statics have no calling convention — any block is fine. Fixed commit `0965a48a`.
**Source:** `feedback_sibling_rlib_ntapi_abi.md`

### C **inline/macro** helper declared `extern` → phantom symbol, i386-only undefined (arcname `IsEqualGUID`, 2026-07-06)
**Symptom:** i386 kernel link `undefined symbol: _IsEqualGUID`, referenced from `iomgr/arcname.rs` (`IopVerifyDiskSignature`); amd64/arm64 link + boot fine. Intermittent onset — was green for weeks, surfaced only after an unrelated change stopped the caller being dead-stripped.
**Root cause:** `IsEqualGUID` is **not a real symbol** — in C (non-C++) guiddef.h it's a *macro* `(!memcmp(a,b,sizeof(GUID)))`. The translation declared it `extern "C" fn IsEqualGUID`, demanding a symbol that exists nowhere. It only ever "resolved" by accident: `wmi-rust` has a `#[no_mangle] extern "system" fn IsEqualGUID`. On amd64/arm64 `"system"`==cdecl so the export name (`IsEqualGUID`) coincides with the cdecl reference and links; on i386 `"system"`==stdcall → `_IsEqualGUID@8`, but the `extern "C"` reference wants `_IsEqualGUID` → mismatch → undefined. `--gc-sections` masked it on all three until the caller stopped being stripped, then it broke on i386 only.
**Fix:** Don't declare a C macro/`__inline` as an `extern` fn — **implement it inline in Rust** mirroring the macro (`arcname.rs` now has a private `#[inline(always)] unsafe fn IsEqualGUID` doing the `memcmp`). Kills the phantom dependency on all arches. General rule: before writing `extern { fn Foo }`, confirm `Foo` is a *real out-of-line symbol* (`nm`/grep for a def), not a header macro/`FORCEINLINE`. Fixed `8ac73dd02ce`.
**Detection gap:** `verify_ffi`/`verify_abi` miss it — the bad decl is a private extern in one crate, the accidental provider is a `no_mangle` export in another; neither is spec/bindgen-visible. Same family as the `DsSetupInit`/`TranslateStdHandle` internal-decl blind spots above.

### Array passed BARE to a C-variadic (`...`) extern — Rust arrays do NOT decay to pointers
**Symptom (i386 uefildr under WHPX, 2026-07-03):** boot "hangs at `[freeldr] Switching stack...`" forever — the wedged PC is `streamout`'s `%s` handler running `strnlen(0x6f636553, SIZE_MAX)`: the "pointer" is ASCII `"Seco"` — the first 4 BYTES of the string that should have been passed by reference. Caller disasm shows a `rep movsl` copying the whole array into the outgoing vararg area. Under TCG the bad scan terminates instantly (unmapped-phys reads return 0) so the bug is LATENT for months; under hardware virt (WHPX/KVM) every probe of unmapped guest-phys is a VM exit → a ~4G-iteration crawl ≈ hours = "hang".
**Root cause:** C array arguments decay to pointers; **Rust `[T; N]` passed to an `extern "C" fn(..., ...)` variadic is passed BY VALUE** — all N bytes land in the vararg area and `%s`/`%p` consume the first word as the pointer. rustc accepts it silently. Worse, on amd64 (Win64 ABI) >8-byte aggregates go by reference, so the SAME code accidentally works on 64-bit and only breaks i386.
**Fix:** `.as_ptr()` every array/static passed to a variadic (tuimenu.rs `UiTimeText`, fixed this round). **Sweep recipe:** collect every `static/let` ident with an array type crate-wide, then flag variadic call args that are a bare matching ident without `.as_ptr()`/`.as_mut_ptr()`; also disasm-grep for `rep movs` with const count feeding a call to a known printf. Watch every rustconv translation of a C file that printfs a global `char[]`.
**Source:** WHPX bring-up spike (docs/engineering/windows-host-qemu-whpx-spike.md), fix in `sdk/lib/rust/freeldr-rust/src/tuimenu.rs`

### Same fn name, two ABIs in one crate → ud2 stub even though linker is happy
**Symptom:** Function dramatically smaller than expected (0x30 vs 0x1df bytes), ends in `0f 0b` shortly after a call; `nm` shows BOTH `_Name` and `_Name@N` (smoking gun). IR marks outparams `readnone captures(none)`.
**Root cause:** Def is `extern "C"` (`_Name`), decl elsewhere is `extern "system"` (`_Name@N`) — linker sees distinct symbols, but LLVM shares the bare IR name `@Name`, detects the call-conv contradiction → `unreachable` folds the rest of the caller.
**Fix:** Pick ONE ABI matching the C contract (NTAPI=stdcall=`extern "system"`; `__cdecl`=`extern "C"`); if the C lib owns the symbol, delete the Rust dup and `pub use` from the sys-crate. Always check ABI via `nm` FIRST before chasing captures(none). Closed `0678cf5d` (frldr.rs `RtlZeroMemory`, libsupp.rs `FrLdrHeapFree`).
**Source:** `feedback_dup_abi_name_collision.md`

### At-scale variant: 76 decl↔def CC mismatches folded ~46 fns (classpnp, 2026-06-11)
**Symptom:** Whole driver's worth of functions fold to `push;mov;ud2` at opt-1 (BUGCHECK 0x1E at boot), yet the cohab link reports **0 undefined** (lld fuzzy-matches `_F` ↔ `_F@N`). Even clean raw-pointer fns with NO `&(*ptr).field` fold — disproving the reference-UB theory.
**Diagnostic that cracked it:** `RUSTFLAGS="… -C llvm-args=-print-changed=quiet -C llvm-args=-filter-print-funcs=<fn>"` → read the IR-dump sequence for the pass that inserts `store i1 true, ptr poison` (InstCombine's UB marker), then walk back to the offending instruction. Raw pre-opt IR (`-C no-prepopulate-passes --emit=llvm-ir`) shows rustc emits a NORMAL body — LLVM folds it. Build a def↔decl CC map (regex over `pub … extern "X" fn` defs vs `extern "Y" { fn }` decls); fix every decl CC := def CC. (`shims.rs` glob-imported everywhere with `extern "C"` decls of stdcall-defined fns was the systemic vector.)
**Two follow-on variants the un-folding exposed:**
- **Self-CC-collision wrapper:** a cdecl `#[no_mangle] fn ExFreePool` that calls `ntkrnl_sys::ddk::ExFreePool` (stdcall, *same IR name* `@ExFreePool`) calls itself with a mismatched CC → folds on every arch. NEVER wrap a kernel export under its own name; call the real stdcall export directly (`extern "system"`), no wrapper.
- **Inline-interaction fold:** a fn that's fold-free standalone can fold its *caller* when inlined — LLVM combines the inlinee's writes with the caller's `memcpy`/dereferenceable facts. Fix: `#[inline(never)]` (classpnp `ClassInitializeDispatchTables` inlined into `ClassInitialize` → ud2 after `RtlCopyUnicodeString`).
**Note:** i386-only — on amd64/arm64 `extern "C"` == `extern "system"` (both cdecl), so these never fold there. Fixed in classpnp `ec2ad2607db`; full writeup `classpnp-cohab-link-gap.md`.

### `verify_abi.py` blind spot: INTERNAL (non-exported) symbols — runtime ESP corruption, not a fold (kernel32, 2026-06-15)
**Symptom:** i386 runtime `STATUS_INVALID_HANDLE` from `WaitForMultipleObjectsEx` (winlogon StartRpcServer → rpcrt4 wait → WAIT_FAILED → black screen), **with `verify_abi ✓` AND `verify_ffi ✓` AND no `ud2`/fold.** The mismatched fn (`TranslateStdHandle`, cdecl per handle.c's no-`WINAPI` decl) was DEFINED `extern "C"` but one caller (`synch.rs`) DECLARED it `extern "system"`. Because it's a kernel32-**internal** helper (not in `ntoskrnl.spec` / bindgen), `verify_abi.py` — which cross-checks extern blocks *against the spec/bindgen* — never sees it. And it didn't fold (the two ABIs agree enough that LLVM emits a real call), so `scan_ud2` is silent too. It just **drifts ESP 4B per call**: harmless in a single-call caller (absorbed at the frame-pointer epilogue) but in `WaitForMultipleObjectsEx`'s per-handle translate **loop** the drift corrupts the ESP-relative handle buffer (release build omits the frame pointer) → garbage handles. WaitForSingleObject (one TranslateStdHandle call) survived; the loop didn't.
**Diagnostic:** the def↔decl CC scan must run over the **whole crate's own symbols**, not just spec-checked ones — find every fn name appearing under BOTH `extern "C"` and `extern "system"` (decls *and* `pub extern "X" fn` defs). Catches what verify_abi misses for internal helpers. (kernel32 had 4; only the *called* ones bite — `TranslateStdHandle`, `BaseFormatTimeOut`. Address-taken-only startup fns (BaseThreadStartup/BaseFiberStartup/BaseProcessStartup written into CONTEXT `.Eip/.Rip`) are harmless: never called via the decl, and i386 uses a separate `_BaseThreadStartupThunk@0`.)
**Fix:** decl CC := def CC := the C contract (no `WINAPI`/`NTAPI` ⇒ cdecl ⇒ `extern "C"`; `WINAPI`/`NTAPI` ⇒ stdcall ⇒ `extern "system"`). Fixed `f2f349d8a5d` (kernel32-port).

### Wrong-ABI **declaration** of an internal fn → i386 call *diverges* (not ESP-drift), even at 0 args (lsasrv, 2026-06-26)
**Symptom:** i386-only HANG during lsass init — `lsass.exe` wedges in `LsarStartRpcServer` right after `RpcServerRegisterIf`, before `SetEvent(LSA_RPC_SERVER_ACTIVE)`, so winlogon `WaitForLsass` (`WaitForSingleObject(event, INFINITE)`) waits forever → no desktop. No BUGCHECK, no `ud2`, **no Rust panic** (a panic-handler DbgPrint proved it), **links clean ×3**, amd64/arm64 boot fine. Deterministic.
**Root cause:** `lsarpc.rs` declared the internal helper `DsSetupInit` (C: `VOID DsSetupInit(VOID)` — **cdecl**, no `WINAPI`) inside an `extern "system"` block. On i386 that emits a reference to the stdcall-decorated `_DsSetupInit@0`, but the definition (`dssetup.rs`, correctly `extern "C"`) is `_DsSetupInit`. The `@0` reference **resolves to the wrong target** (the link does NOT error — it resolves to *something* via auto-import/fixup), so the call at runtime **diverges** (jumps to a bad address) and is swallowed by DUMMY_PSEH → the thread wedges. This is a THIRD ABI failure mode, distinct from the two famous ones: not a `ud2`/fold (line 40), not ESP-drift (TranslateStdHandle, line 54). The decoration-mismatch bites **even though `DsSetupInit` takes 0 args** (where stdcall and cdecl are behaviorally identical) — because the *symbol name* differs (`_F@0` vs `_F`), the bug is in symbol *resolution*, not stack cleanup.
**Why every existing check misses it:** (1) the def↔decl "appears under BOTH `extern "C"` and `extern "system"`" scan (line 56) does **NOT** catch it — `DsSetupInit` was declared *only once*, as `extern "system"`, never under both. (2) `--disable-stdcall-fixup` + 0-undefined ×3 does **not** rule it out — the `@0` ref resolves to *something*, so the link is clean (this is the exact false-confidence that makes you suspect contention/a 64-bit-only path instead). (3) `verify_abi`/`verify_ffi` miss it — `DsSetupInit` is lsasrv-internal, invisible to spec/bindgen (same blind spot as TranslateStdHandle).
**Detection recipe:** for every `extern "system"`/`extern "C"` **declaration** of an *internal* (translated, non-import) fn, cross-check the ABI against that fn's **actual Rust definition** AND the C convention (`VOID Foo(VOID)`/no-`WINAPI` ⇒ cdecl ⇒ `extern "C"`; `WINAPI`/`NTAPI`/`__stdcall`/`__RPC_USER` ⇒ `extern "system"`). The matching scan is **decl-ABI vs def-ABI per symbol**, not "both ABIs present." Worth teaching `verify_abi.py` to resolve internal-fn decls against their in-tree defs.
**Localization that cracked it (use for any i386-only early-init hang):** DbgPrint-bisect the init path with **real `DbgPrint`** (translated `TRACE!`/`WARN!` are no-op macros — their *absence* in the serial is NOT evidence of where execution stopped; that false signal cost a detour). Bracket each init step + each callee; the hang sits at the last printed trace's *next* call. Then audit that callee's decl ABI vs its def. (Boot a worktree i386 ISO directly — `qemu-system-i386 -M pc -accel kvm -cpu host -drive if=pflash,...OVMF32_CODE.fd... -device ide-cd,drive=cd0,bootindex=0 -serial file:...` — because `vibeos-cli smoke` from a worktree boots the **main** repo's media via the editable-install `tests` shadow.)
**Fix:** one line — declare `DsSetupInit` `extern "C"` to match its cdecl definition. i386 boots to desktop. Fixed on kernel32-port (lsasrv M5).

### disk-rust variants (2026-06-11): llvm.assume fold, noreturn-decl poison, per-symbol CC authority
Same class as the classpnp at-scale fold; four refinements from the disk.sys port:
- **Faster diagnostic than `-print-changed`:** `cargo rustc -p <crate> --target i686… --release -- --emit=llvm-ir`, then read the folded fn in the `.ll`. The smoking gun is `tail call void @llvm.assume(i1 %cond)` where the source had `if cond` — LLVM proved the OTHER branch UB and converted the branch to an assume. Whatever the surviving path calls next is NOT the culprit; the proven-UB call on the dead path is.
- **noreturn decl poisons a returning def:** one stray `fn __int2c() -> !;` decl (def returns) merges into the same IR name and marks all callers' fall-through unreachable → `ud2` right after the call, plus assert branches folded always-taken. Symptom at boot: spurious `Assertion failed … : 0` prints with a sane printed value (e.g. `IRQL (0)`), then BUGCHECK 0x1E/0xC000001D. Audit every `-> !` extern decl against its def.
- **The CC authority is per-symbol bindgen, not a blanket rule.** MS-style driver headers declare SOME fns `NTAPI` (the class-driver callbacks → stdcall) and most helpers without (→ cdecl). And ntkrnl-sys bindgen declares FORCEINLINE-derived names (`KeInitializeSpinLock`, `IoMarkIrpPending`, `IoGet*StackLocation`…) **`extern "C"`** even where a sibling real export is stdcall — in-crate shim defs must match the BINDGEN decl, because translated callers import those paths. Reconcile programmatically: regex `types_i386.rs`/`bindgen_i386.rs` block ABIs into a name→CC map, diff every `pub extern "X" fn` def and extern-block decl in the crate against it, fix the minority side (49 defs + 17 decls in disk-rust).
- **Callback param-type hallucination → kernel AV after the folds are gone:** translation gave `DiskBootDriverReinit` a `PDEVICE_OBJECT` param and deref'd `.DriverObject`; C takes `PDRIVER_OBJECT` directly. Garbage pointer into `IoRegisterDriverReinitialization` → BUGCHECK 0x1E/0xC0000005 inside `IopReinitializeBootDrivers`. When a crash lands INSIDE a kernel registration/iteration routine right after your driver's entry ran, diff the callback signatures against the C first.

### extern decl returns `u32` for a `BOOLEAN`(u8) impl → amd64 false-"true"
**Symptom:** amd64-only: a `return 0` (false) reads as nonzero/true; path-dependent. Real incident: BUGCHECK 0x4C (`0xc000021a`), "Load of WIN32K failed", SMSS `SmpInit` fail.
**Root cause:** Callee sets only `AL`; a `-> u32` caller reads full `EAX` with garbage upper 24 bits (clean only on `xor eax,eax` paths). `MiResolveImageReferences` declared `RtlEqualUnicodeString(..,u32)->u32` vs real `(..,u8)->u8`; false match made ftfd.dll snap against usbd.sys exports → `STATUS_DRIVER_ENTRYPOINT_NOT_FOUND`.
**Fix:** Declare booleans `u8` (BOOLEAN), never `u32`/`i32`, so the caller masks to AL. Audit all callers of the fn. Debug recipe: dump matched LDR `BaseDllName` as raw u16 hex (not `%ws`).
**Source:** `feedback_boolean_u8_return_as_u32_abi.md`
**i386 manifestation + def-side fix (cdfs-rust, 2026-06-14):** Same bug bites i386 too (LLM grind emits per-module externs that disagree: `FlagOn` defined `-> BOOLEAN` but declared `-> i32`/`-> u32` across files). `CdCleanupIrpContext` ran a `FlagOn(Flags, ALLOC_IO)`-guarded `ExFreePool` with the flag *clear* — `FlagOn`'s `setne %al` left garbage in the upper EAX, the `-> i32` caller did `testl %eax,%eax` → false-true → freed a stack `IoContext` → BUGCHECK 0x19 BAD_POOL_HEADER. When callers are inconsistent and you own the impl, fix the **definition** to return a zero-extended `u32` (`((a&b)!=0) as u32` → forces `movzbl`, valid AL *and* EAX for every caller) rather than chasing every decl. Detector: flag any fn whose **definition** is narrow (u8/BOOLEAN) with *any* wide (i32/u32) caller-decl; a fn DEFINED `-> i32` with BOOLEAN callers is SAFE (AL = low byte of 0/1). Fixed 9 such fns in cdfs-rust. Nondeterministic (depends on prior EAX) → reads as flaky boot.
**u32↔i64 variant (cdfs-rust SectorTruncate, 2026-06-14) — the same class, wider:** `SectorTruncate` DEFINED `-> u32` but DECLARED `-> i64` in two modules. On i386 an i64 return is **EDX:EAX**; a u32 callee sets only EAX → **EDX = garbage** leaks into the high dword. `current_base_offset = SectorTruncate(0)` came back `0x1_00000000`; that offset flowed through CcMapData → the path-table paging read reached CdCommonRead with `StartingOffset=0x100000000` → spurious `>= FileSize` STATUS_END_OF_FILE → MM zero-filled the page → cdfs parsed zeros → BUGCHECK 0x1E STATUS_DISK_CORRUPT_ERROR. **Generalize the detector to ANY def≠decl return-width mismatch, not just narrow-vs-wide** (u8/BOOLEAN↔i32/u32 *and* u32/i32↔i64/u64). Benign-alias pairs to ignore: NTSTATUS=i32, ULONG=u32, LONGLONG=i64, BOOLEAN=u8, PFCB=*mut FCB, POPLOCK=*mut OPLOCK. Fix the narrow side: match the definition's type and cast at the call site (`SectorTruncate(x) as i64`). Debug recipe: print hi/lo dwords as two `%x` (kernel DbgPrint mishandles `%I64x`); deterministic DbgPrint via the **smoke** harness (start.sh boots flake on KVM contention), grep `full.log`.

### ARGUMENT-width mismatch: i64 param declared u32 (or vice versa) → i386 garbage/shift
**Symptom (cdfs-rust, 2026-06-14):** i386-only BUGCHECK with a value that has a bogus high dword — e.g. an Mcb `ByteCount` of `0x80000000_00000800`, making `off < FileOffset+ByteCount` go negative so a lookup "fails" → STATUS_DISK_CORRUPT_ERROR.
**Root cause:** the sibling of the return-width bug, for *parameters*. A fn DEFINED with an `i64`/`LONGLONG` param but DECLARED with `u32`/`ULONG` (or swapped) in a caller module. On i386 cdecl the i64-reading callee picks up 4 garbage stack bytes for the high dword (if the param is last) or shifts every later arg. `CdAddInitialAllocation(.., StartingBlock: u32, DataLength: i64)` was declared `(.., i64, u32)` / `(.., u32, u32)` in two modules. Also hit `CdTruncateAllocation`, `CdSetFidPathTableOffset`, `CdLookupInitialFileDirent`.
**Fix:** make every per-module extern decl AND call site match the definition's param types; cast args at call sites (`x as i64` / `x as u32`). Detector: a param-by-param def-vs-decl 32-vs-64 width scan (one-liner in the 2026-06-14 cdfs transcript) — run it alongside the return-width scan.
**Source:** cdfs-rust commit `158560df26b`.

### Cross-DLL DATA import: extra pointer indirection (`*const *const` vs `*const`) → access violation
**Symptom (cdfs-rust FsRtlLegalAnsiCharacterArray, 2026-06-14):** BUGCHECK 0x1E STATUS_ACCESS_VIOLATION dereferencing a ntoskrnl data export (here in CdIsLegalName indexing the legal-char array).
**Root cause:** the exported symbol is a *single* pointer (`const UCHAR * const FsRtlLegalAnsiCharacterArray`, value = array base), but the driver declared it `*const *const u8` and did `*Sym` before indexing. With `#[link(kind="raw-dylib")]` the `__imp_` load already yields the symbol's value, so the extra `*` reads the array's first bytes AS a pointer → fault. (The `* const *` form exists in fsrtlfuncs.h but only under `__CREATE_NTOS_DATA_IMPORT_ALIAS`, which is NOT what ntoskrnl exports — don't copy it.)
**Fix:** declare the import with the SAME level of indirection ntoskrnl exports (check the ntoskrnl-rust `pub static` / the C `.c` definition, not just the header's import-alias variant) and index directly.
**Source:** cdfs-rust commit `747b0e50233`.

### `i64` substituted for `_LARGE_INTEGER` by-value → i386 boot wedge
**Symptom:** i386 hangs in win32k message loop, spamming `err: Sending to Suspended Thread Msg 121`; reaches WinLogon/GinaInit then wedges. amd64 7/7 pass.
**Root cause:** `_LARGE_INTEGER` is an 8-byte aggregate union; i386 cdecl lowers aggregates differently from scalar `i64`, so the `_rs_*` impl reads garbage off the stack. amd64 x64 ABI passes both in the same slot, hiding it. (`_rs_KxSetTimerForThreadWait` declared `interval: i64` vs impl `_LARGE_INTEGER`.)
**Fix:** Mirror the receiver's exact union type on both sides; pass `*ptr` (the struct), not `(*ptr).QuadPart`. Same for any bindgen union (`_ULARGE_INTEGER`, `_M128A`).
**Source:** `feedback_i64_vs_largeinteger_abi.md`

### c2rust direct union/bitfield access vs `__BindgenUnionField` repr → E0599/E0609/E0063 cluster
**Symptom (buslogic-rust, 2026-06-26):** a c2rust/LLM driver translation that touches C bitfield-union *registers* compiles to a wall of `no method 'X' found for __BindgenUnionField<T>` (E0599), `no field 'All'/'__bindgen_anon_1'` (E0609), and `missing fields 'Bits' and 'bindgen_union_field' in initializer` (E0063).
**Root cause:** c2rust emits C-native union access — `reg.All = x`, `reg.Bits.SomeFlag()`, `Reg { All: 0 }` — but the repo's gen_driver_types.sh / ntkrnl-sys bindgen renders **every** union as the legacy `__BindgenUnionField` struct (zero-sized accessor fields + a `bindgen_union_field` storage blob), never a native Rust `union`. The accessor fields aren't real fields, so direct read/write/init all fail.
**Fix (mechanical, sed-able):**
- init `Reg { All: 0 }` → `core::mem::zeroed()` (the let-binding's type annotation carries the type).
- read bit `reg.Bits.Flag()` → `reg.Bits.as_ref().Flag()`; read whole `reg.All` → `(*reg.All.as_ref())`; write `reg.All = v` → `*reg.All.as_mut() = v`. (`__BindgenUnionField<T>::as_ref/as_mut` transmute `&self`→`&T`.)
- nested c2rust anon names differ from bindgen's: `.c2rust_unnamed` → `.__bindgen_anon_1`; a C anon-union/struct member bindgen *flattened* (e.g. CCB `.Status`) drops the `.__bindgen_anon_1.` prefix entirely — check the substrate struct.
- a native bindgen `union` (e.g. ddk `_LARGE_INTEGER`) IS direct-access (`.__bindgen_anon_1.LowPart`), but a `__BindgenUnionField` one is NOT — grep the substrate to tell which.
Do the four register rewrites in any order (the suffixes `as`/`)`/`;`/` = ` are disjoint). atapi-rust's hand translation already used `.as_ref()` — the gap is c2rust/LLM output that didn't.

### gen_driver_types.sh enum members: bindgen names them `<EnumTag>_<member>`, the C source used the bare name
**Symptom:** ~50 `cannot find value 'BusLogic_VESA_Bus' / 'RequestComplete' / 'ScsiAdapterControlMax'` (E0425) after wiring a driver substrate — the translation references C enum members by their bare name.
**Root cause:** bindgen prefixes each enum constant with its enum *tag* (`BusLogic_HostAdapterBusType_T_BusLogic_VESA_Bus`, `_SCSI_NOTIFICATION_TYPE_RequestComplete`), and long-typed consts **wrap the `: <type> = N;` onto the next line**.
**Fix (automate it):** collect the missing names from the compiler, then for each grep the substrate for `pub const [A-Za-z0-9_]*<name>:` (anchor on the trailing `:` so it survives the line-wrap; type-on-next-line breaks a `pub const NAME: TYPE =` one-liner regex) and emit `pub use crate::types::<fullname> as <name>;` into glue (type-free `pub use` > `pub const X: T = ...`, no type to chase, arch-independent since enum names are identical on all 3). A handful of dup-value members bindgen drops, and #defines not in an allowlisted header (`SP_UNTAGGED`), need a manual `pub const`.
**Source:** buslogic-rust (BusLogic958.c → Rust, the last C-only storage driver; `drivers/storage/` 100% Rust). Companion to the `wstr!` compile-time `L"..."` macro (pure-LLM emits `…encode_utf16().collect::<Vec<u16>>()` — no allocator in-kernel).

### `PWCHAR HalName = L"..."` translated as `[u16; N]` → triple-fault at boot
**Symptom:** Kernel hangs at boot with no trace points (fault precedes HAL init); `cargo check` and the linker are both happy.
**Root cause:** The C export is a pointer *variable* (4/8 bytes holding the string's address), not the string bytes. As `pub static HalName: [u16; N]` the symbol sits at the buffer start, so consumers read `'A','C','P','I'` (`0x49004300...`) as the pointer and dereference a wild address.
**Fix:** `static HAL_NAME_STORAGE: [u16; N] = [...]; #[no_mangle] pub static mut HalName: *const u16 = HAL_NAME_STORAGE.as_ptr();`. Applies to any `PWCHAR`/`PCHAR`/`PSTR`/`LPSTR`/`PCWSTR` global initialized with a literal. Fixed-size `T[N]` arrays (e.g. `UCHAR HalpVectorToIndex[256]`) keep the array shape.
**Source:** `feedback_pwchar_pointer_export.md`

### i386 PE symbol lookups miss real symbols (three decoration schemes)
**Symptom:** `grep`/sym lookup returns nothing for a symbol that exists (e.g. `KiUnwaitThread` not found — actual symbol is `@KiUnwaitThread@12`).
**Root cause:** i386 PE uses three schemes: cdecl `_name`, stdcall `_name@N`, fastcall `@name@N`. amd64/arm64 emit bare names.
**Fix:** Try all four forms (`_X`, `_X@`, `@X@`, bare) or use `qemu/sym2addr.sh` which handles them. FFI shim files need explicit fastcall decoration via `fastcall_export!`.
**Source:** `feedback_i386_pe_decoration.md`

### verify_abi.py — automated extern "C"/"system" mismatch detector
**Symptom:** N/A — preventive tool for the ud2 crash classes above.
**Fix:** Run `.venv/bin/python tools/rustconv/rustconv/verify_abi.py [file.rs]` after any translation; cross-checks extern blocks vs ntoskrnl.spec (1,567 exports) + bindgen_i386.rs (2,921 fns). Current state: 61 mismatches (42 stdcall → fix via `extern "system"`; 19 fastcall → need C shims, no stable `extern "fastcall"`). Add to the post-integration checklist beside verify_constants.py.
**Source:** `feedback_verify_abi.md`

### i386 stdcall→cdecl `global_asm!` `.set` alias must live in the target's crate
**Symptom:** i386 link error `undefined symbol: _PsGetCurrentThread` (referenced by `_KdbpCmdProc`).
**Root cause:** GAS `.set _X, _X@0` evaluates at assembly time; if the target symbol isn't in the same `.o`, it emits `_X` as an *undefined reference* rather than an alias relocation, and lld won't resolve it across rlib archives. Happens when the wrapper def moves into a sibling rlib but its `.set` line stays behind.
**Fix:** When migrating a `shim_exports.rs` wrapper into a sibling rlib, move its `.set _name, _name@0` line into the destination rlib's `lib.rs` (`#[cfg(target_arch="x86")] core::arch::global_asm!`). Came up moving `PsGetCurrentThread` into common-inlines.
**Source:** `feedback_global_asm_alias_locality.md`

### LLVM COFF-i386 `global_asm!`: `push offset <label>` → "unsupported relocation type" (must `mov reg, offset`; bare `push <label>` is a MEMORY push)
**Symptom:** Transcribing MASM/GAS `.s` into `core::arch::global_asm!` for the i686 (`i686-w64-mingw32` / `i686-kernel.json`) target, the build dies with `error: unsupported relocation type` on a line like `push offset glu_return` / `push _unwind_handler` (pushing a code-label address as an imm32). Same source assembled fine with the original `ml`/GAS.
**Root cause:** LLVM's COFF-i386 backend won't emit a symbol relocation on the `push imm32` encoding (`0x68 + DIR32`), even though MASM does. The *mov-imm32* encoding (`0xB8 + IMAGE_REL_I386_DIR32`) **is** supported. Worse trap if you "simplify" to bare `push glu_return`: LLVM intel syntax treats that as a **memory** push (`ff 35` — reads the dword *at* the label), silently producing wrong code instead of an error. `offset` is mandatory for the address; `cmp dword ptr [..], offset <label>` (cmp-imm32) is supported and needs no workaround.
**Fix:** Materialize the address in a dead caller-saved scratch reg, then push the reg:
```
"    mov eax, offset glu_return",   // was: push offset glu_return
"    push eax",
```
Pick a register provably dead at that point (here `eax`: clobbered by the imminent `_RtlUnwind@16` / reloaded after). Verify the result with the disasm-diff oracle below.
**Source:** pseh-rust (seh.s → `sdk/lib/rust/pseh-rust/src/lib.rs`, the two push-address sites in `__global_unwind2` / `__local_unwind2`).

### Verify a `global_asm!` translation by disasm-diffing against the original `.obj` (a stronger oracle than smoke)
**Symptom:** N/A — preventive method for irreducible-asm translations (SEH/setjmp/CRT runtimes) whose paths a boot smoke barely or never exercises, so green smoke is weak evidence of correctness.
**Fix:** Build both, then disassemble and diff the function bodies — they should be instruction-for-instruction identical except for documented workarounds. The original cmake `.obj` lives under `build-<arch>/.../CMakeFiles/<tgt>.dir/<file>.s.obj`; the Rust output is in the crate's staticlib. `objdump -d` each (don't `llvm-ar x/p` a **thin archive** — `libpseh.a` references `.obj` on disk, extraction is unsupported; objdump the on-disk `.obj` directly). Any diff beyond your annotated `mov/push` reloc workarounds is a real divergence. This caught nothing-wrong on pseh-rust (clean match) but is the oracle that makes a cold/untestable asm port trustworthy without booting it.
**Source:** pseh-rust validation (disasm-equivalence vs `seh.s.obj`).

### New rlib's `#[no_mangle]` symbols absent from libkernel_rust.a (link fails)
**Symptom:** Staticlib builds OK, but ntoskrnl link fails with hundreds of undefined-symbol errors naming the new rlib's wrappers; `nm` shows them `U` in libkernel_rust.a.
**Root cause:** A Cargo dependency alone gives no reachability path; rustc only archives a dep's CGUs into a staticlib when explicitly reached.
**Fix:** Do all three: (1) add `name = { path = "../name" }` to kernel-rust/Cargo.toml + workspace members; (2) add `extern crate name;` to kernel-rust/src/lib.rs; (3) add the `_force_name_symbols` block with extern decls + `#[used] static REFS` (prevents DCE on the rlib side). With `extern crate` the symbols become `T`.
**Source:** `feedback_extern_crate_for_staticlib.md`

### Cross-crate `#[inline(always)]` inline-asm helper → undefined symbol on i386
**Symptom:** i386 link error `undefined symbol: _KeGetCurrentThread` even though the upstream def is `#[inline(always)] pub unsafe fn` with an `asm!` body.
**Root cause:** At `opt-level=1` rustc treats inline-asm bodies as opaque blobs and doesn't reliably fire cross-crate `#[inline(always)]`; it falls back to a call against the unmangled name, but the C-side helper is itself a FORCEINLINE with no exported symbol.
**Fix:** Replicate the asm into the consuming crate rather than importing it (the two-line duplicate is identical instructions). Applies to `KeGetCurrentThread`/`KeGetCurrentPrcb`/port-I/O helpers. Caught building cm-inlines, commit `646af953`.
**Source:** `feedback_cross_crate_inline_asm.md`

### DDK FORCEINLINE exported only on i386 → amd64/arm64 link fails (inverse of the usual)
**Symptom:** A standalone driver `.sys` links clean on i386 but `undefined symbol: KeGetCurrentIrql` (undecorated) on amd64/arm64 — the *opposite* of most FFI link bugs which fail on i386. cargo-check passes on all arches (the bindgen extern decl satisfies type-check); only the link breaks.
**Root cause:** `KeGetCurrentIrql` is a real ntoskrnl **export on i386** but a **FORCEINLINE on amd64/arm64** (reads CR8 / the PCR), so the bindgen `ddk` extern resolves on i386 and dangles on 64-bit. Two ways the driver reaches the bad extern: (a) the module imports it from `ntkrnl_sys::ddk`, or (b) — the partmgr trap — the LLM emitted a **local `extern "system" { fn KeGetCurrentIrql() -> u8; }`** phantom decl inside the translated file.
**Fix:** Import the per-arch inline explicitly: `use ntkrnl_sys::ke::KeGetCurrentIrql;` (a glob re-export through `crate::inlines` does NOT win over a local extern decl — must be explicit, pcix pattern), and **delete any LLM-emitted local `extern` decl of a DDK FORCEINLINE**. Same class as `IoGetCurrentIrpStackLocation`/`IoSkipCurrentIrpStackLocation` (→ `ntkrnl_sys::inline::tier1`) and `ExAllocatePoolZero`/`ExAllocatePoolUninitialized` (exfuncs.h FORCEINLINE wrapping `ExAllocatePoolWithTag` — define inline, use `core::ptr::write_bytes` for the zero). Caught landing partmgr-rust, 2026-06-12.
**Source:** `project_partmgr_rust_track.md`

### Three cross-arch link traps for kernel-time / IRQL / raise helpers (mup-rust, 2026-06-13)
A standalone `.sys` cohab that reads system time, raises status, or checks IRQL hits a per-arch trio. All cargo-check clean on every arch; only the link breaks, and on a *different* arch each.
1. **`ExRaiseStatus` → `undefined symbol: _RtlRaiseStatus@4` (i386).** `ntkrnl_sys::ex::ExRaiseStatus` is `#[link_name = "RtlRaiseStatus"]`, but ntoskrnl's spec exports **only `ExRaiseStatus`** (i386 `@4`), not `RtlRaiseStatus`. **Fix:** use the bindgen decl `use ntkrnl_sys::ddk::ExRaiseStatus;` (real `ExRaiseStatus` symbol). Both are `-> !`, so the diverging-call codegen is identical. (The `ex::` link-name aliasing only resolves inside the kernel image itself, where `RtlRaiseStatus` *is* present; a driver linking against the import lib has only the exported name.)
2. **amd64: `undefined symbol: SharedUserData`.** `KeQuerySystemTime` is a real export on i386/arm but a SharedUserData-reading inline on amd64; the ntkrnl-sys inline references `extern { static SharedUserData }`, a C macro for the fixed KI_USER_SHARED_DATA VA, not a real symbol. **Fix:** `#[cfg(not(target_arch="x86"))] #[no_mangle] pub static mut SharedUserData: *mut _KUSER_SHARED_DATA = 0xFFFFF78000000000usize as *mut _;` (classpnp-rust's shim), and cfg-split the import: i386 → `ddk::KeQuerySystemTime` (export), non-x86 → `inline::ke::KeQuerySystemTime`.
3. **arm64: `undefined symbol: KeGetCurrentPrcb`.** `KeGetCurrentIrql` resolves via the ntkrnl_sys inline, which references `KeGetCurrentPrcb` — not exported on arm64. **Fix:** `#[cfg(target_arch="aarch64")] #[no_mangle] pub unsafe extern "C" fn KeGetCurrentPrcb() -> PKPRCB { ntkrnl_sys::inline::ke::KeGetCurrentPrcb() }` — forward the export to the register-reading inline (distinct IR name), classpnp-rust pattern.

Also (worktree-build, not translation): when giving a worktree its own `build-<arch>` via the warm-dir copy+repoint recipe, the repoint sed must **NOT** rewrite `.build-tools/` paths — `.build-tools` is gitignored and lives only in main. Over-rewriting makes the `cc` crate's build.rs look for `…/vibeos-wt-X/.build-tools/llvm-mingw/bin/i686-w64-mingw32-clang` (ENOENT) → **every `*-cargo` Phase-2 crate fails its custom build command** ("failed to run custom build command", ToolNotFound), masquerading as a systemic cargo regression. Fix: `ln -sfn <main>/.build-tools <worktree>/.build-tools` (toolchain is read-only/shared). The C/link compiler path in the cache should also stay pointed at main's `.build-tools` (the recipe's "harmless" note assumed it was left alone).
**Source:** `project_mup_rust_track.md`

### Four multi-file driver-cohab integration traps (npfs-rust, 2026-06-20)
A 20-module driver translated by parallel subagents over a shared substrate hit these at central integration. All recur for any multi-`.c` driver-cohab lane — grep here before re-deriving.
1. **`use ntkrnl_sys::base::*;` + `use ntkrnl_sys::ddk::*;` together → a wall of `error[E0659]: X is ambiguous`** (UNICODE_STRING, ULONG, NTSTATUS, PVOID, LIST_ENTRY, STATUS_SUCCESS…). `base` and `ddk` both export those names. **Fix:** keep ONE glob (`use ntkrnl_sys::ddk::*;` — it has the types) and import base-only items explicitly (`use ntkrnl_sys::base::{NT_SUCCESS, InitializeListHead, InsertHeadList, InsertTailList, RemoveEntryList, RemoveHeadList, RemoveTailList, IsListEmpty};`). The glue/types/glob STATUS_* ambiguity vanishes too once the base glob is gone. (`AppendTailList` is NOT in base — don't list it.) Bake this into the per-module skeleton in the subagent contract, not the contract's "use both globs" default.
2. **`src/main.rs` is auto-detected as a CARGO BIN TARGET** — a module named `main` (from `main.c`) collides with cargo's convention, so cargo builds a `bin "npfs-rust"` and every `crate::glue`/`crate::types`/sibling import "fails to resolve" + `error[E0601]: main function not found`. **Fix:** `autobins = false` (and `autoexamples/autotests/autobenches = false`) in `[package]`. Keeps `pub mod main;` working as a lib module.
3. **`Io{SetCancelRoutine,SetCompletionRoutine,GetNextIrpStackLocation,SetNextIrpStackLocation,MarkIrpPending}` + `RtlInitEmptyUnicodeString` compile via `ddk::*` but are `undefined symbol` at link** (i386 `_IoSetCancelRoutine`, etc.). They're header **inlines** (FORCEINLINE), not ntoskrnl exports — bindgen still emits an extern decl, so the call compiles but has no symbol to bind. **Fix:** route them through the inline modules: `use ntkrnl_sys::inline::tier1::{IoSetCancelRoutine, IoSetCompletionRoutine, IoGetNextIrpStackLocation, IoSetNextIrpStackLocation, IoMarkIrpPending};` and `ntkrnl_sys::inline::common::RtlInitEmptyUnicodeString` (the explicit import beats the `ddk::*` glob). Same class as the `KeGetCurrentPrcb`/`KeQuerySystemTime` inline-vs-export traps above. (Pre-flight a cohab link to surface these — cargo-check is blind to them.)
4. **i386 FAST_IO (and any NTAPI callback) slot is `extern "stdcall"`, not `extern "C"`** → `error[E0308]: expected "stdcall" fn, found "C" fn` ONLY on i386 when a hand-written raw target type is `extern "C"`. **Fix:** don't name a raw inner fn type — transmute the `extern "system"` fn item straight to the bindgen `Option` slot type (`PFAST_IO_READ`/`PDRIVER_DISPATCH`/`PDRIVER_CANCEL`/`PKDEFERRED_ROUTINE`), which carries the correct per-arch ABI (stdcall on i386, C on amd64/arm64): `transmute::<FastIoReadFn, PFAST_IO_READ>(NpFastRead)`. Same null-niche fn→Option transmute as the dispatch-table pattern.

**Source:** `npfs` commit (filesystems lane); contract substrate `sdk/lib/rust/npfs-rust/src/{lib,glue,globals,helpers,types}.rs`.

### amd64 cohab driver: IRQL raise/lower can't use the tier1 `*Internal` wrappers (kbdhid/mouhid, 2026-06-21)
A cohab driver that raises to DISPATCH_LEVEL (input drivers calling a class `ServiceCallback`, anything taking a spinlock-equivalent) translates `KeRaiseIrql(DISPATCH_LEVEL, &Old)` / `KeLowerIrql(Old)`. Routing those through `ntkrnl_sys::inline::tier1::{KeRaiseIrqlInternal, KeLowerIrqlInternal}` compiles ×3 but **fails to link on amd64**: `undefined symbol: KeRaiseIrql` / `KeLowerIrql`. The tier1 `*Internal` wrappers' non-x86 branch calls the kernel-internal `extern KeRaiseIrql(NewIrql, *mut Old)` / `KeLowerIrql(NewIrql)` — symbols **only provided when linked *into* ntoskrnl** (ntoskrnl-rust `ARM3/helpers` / `shim_exports`, `#[no_mangle]` but NOT in `ntoskrnl.spec`). Per-arch reality (verify with `llvm-nm <build>/.../libntoskrnl.a` + `libhal.a`): **i386** ntoskrnl exports `@KfRaiseIrql@4` (fastcall) + `KeLowerIrql`/`KeRaiseIrql`/`KeRaiseIrqlToDpcLevel` (stdcall) → tier1 i386 path links; **arm64** *hal* exports `KeRaiseIrql`/`KeLowerIrql`/`KeRaiseIrqlToDpcLevel` (halstubs) → tier1 path links *because the driver also links hal*; **amd64** neither ntoskrnl nor hal exports any of them → no provider. **Fix:** per-crate arch-dispatched helper — amd64 uses the self-contained CR8 asm `ntkrnl_sys::inline::ke::{KfRaiseIrql_amd64, KeLowerIrql_amd64}` (`mov cr8`), i386/arm64 delegate to the tier1 wrappers. Same inline-vs-export class as the npfs trap #3, but the provider differs *by arch* so a single-arch link check misses it — **pre-flight the cohab `.sys` link on all 3 arches, not just cargo-check.** (`KeRaiseIrqlToDpcLevel` looks tempting as a uniform primitive but is also unexported from the amd64 import lib.)

### Cohab driver importing kernel DATA (`KeLoaderBlock`) → i386/arm64-only link fail, amd64 hides it (ramdisk, 2026-06-21)
The first cohab driver to reference a kernel **data** export (vs a function) — ramdisk reads `KeLoaderBlock` (the `PLOADER_PARAMETER_BLOCK` global, `ntoskrnl.spec @ extern KeLoaderBlock`). `ntkrnl_sys::globals` declares it as a plain `extern "C" { pub static mut KeLoaderBlock: ... }`. That works for the IN-KERNEL local symbol AND links on **amd64** (its import lib satisfies a direct data reference), but **i386/arm64 fail**: `undefined symbol: _KeLoaderBlock` / `KeLoaderBlock`. PE cross-DLL **data** imports need an `__imp_`-indirected thunk on i386/arm64; a plain extern static emits a direct symbol reference with no provider. **Fix (driver-local, don't touch shared globals.rs — the kernel needs the local form):** declare it in the crate's `glue.rs` with `#[link(name = "ntoskrnl", kind = "raw-dylib")] extern "C" { pub static mut KeLoaderBlock: ddk::PLOADER_PARAMETER_BLOCK; }` (raw-dylib generates the per-arch import thunk — same mechanism `postprocess_bindgen.py --hal` uses for the 41 HAL data externs). The translation pulls `KeLoaderBlock` from a `use ntkrnl_sys::globals::*` glob, so add an **explicit** `use crate::glue::KeLoaderBlock;` in the module — an explicit `use` shadows glob imports, so bare `KeLoaderBlock` resolves to the raw-dylib one with no E0659 ambiguity. Surfaces only at the central `.sys` link on i386/arm64 → **pre-flight the cohab link on all 3 arches**, cargo-check and an amd64-only link both miss it.

### Kept `extern "C"` block for NTAPI fns → i386-only `_Name` vs `_Name@N` link break (ramdisk, 2026-06-21)
A mechanical translation often keeps a hand-written `extern "C" { fn RtlInitUnicodeString(..); fn RtlCopyUnicodeString(..); fn RtlQueryRegistryValues(..); }` block for ntoskrnl APIs. On **amd64/arm64** NTAPI is cdecl → links fine. On **i386** NTAPI is **stdcall**: ntoskrnl exports `_RtlInitUnicodeString@8`, but `extern "C"` emits a reference to undecorated `_RtlInitUnicodeString` → `undefined symbol: _RtlInitUnicodeString`. Same root as the kernel32 RPC-wait ABI bug (TranslateStdHandle) — only here it's a *kept extern block* in the translated module, not a cross-rlib mismatch. **Fix:** split the block — NTAPI fns go in `extern "system" { }` (resolves to stdcall `_Name@N` on i386, cdecl elsewhere); leave genuine **cdecl CRT** helpers (`wcsncpy`/`_snwprintf`/`strlen`/`_strnicmp`/`strstr`) in `extern "C"`. Tell them apart: anything `NTAPI`/`WINAPI` in the C header → `extern "system"`; plain C runtime → `extern "C"`. (Keeping the translation's own signatures in an `extern "system"` block is lower-risk than re-importing from `ntkrnl_sys::ddk`, which can drift the param types — `PCWSTR` vs `PWSTR` — and cascade into call-site E0308s.) i386-only → an amd64/arm64 link passes while i386 breaks.

### i386 export-driver + mixed-convention link traps — three refinements (fltmgr/ksecdd, 2026-06-21)
Two multi-`.c` EXPORT-driver cohabs (fltmgr = Flt* SSPI-ish, ksecdd = SSPI) sharpened the kept-`extern "C"` rule above into three reusable sub-cases. All are **i386-only** (amd64/arm64 NTAPI=cdecl hides them) and surface only at the central `.sys` link, never in cargo:
1. **The driver's OWN spec exports must be `extern "system"` too, not just its callees.** A `spec2def` driver's `.def` declares each export `@ stdcall Foo` → the linker needs `_Foo@N`. But the translation defines the export bodies as `#[no_mangle] pub unsafe extern "C" fn Foo` → emits undecorated `_Foo` → `undefined symbol: _Foo@N`. **Fix:** flip every spec-exported definition to `extern "system"` (grep `extern "C" fn <Prefix>[A-Z]` for the API prefix — `Flt`/`Ksec`/SSPI names; leave internal `<Prefix>p*`/helpers alone). 22 Flt + 37 SSPI fns respectively. Safe on amd64/arm64 (system=cdecl there).
2. **`extern "system"` ≠ correct for *all* NTAPI — some kernel exports are FASTCALL.** Blanket-flipping a callee block C→system is WRONG for the fastcall variants: `IofCallDriver`/`IofCompleteRequest`/`ObfDereferenceObject` (the `f`-suffix forms) and, surprisingly, **`ExReleaseResourceLite`** (verify against `ntoskrnl.spec`: `@ fastcall`). stdcall (`extern "system"`) passes args on the stack; fastcall passes the first two in ecx/edx → silent corruption, not a link error. **Best fix: REMOVE the redundant `extern "C"` decl and import the callee from `ntkrnl_sys::ddk`** — the bindgen there carries the real per-arch convention (stdcall *and* fastcall) automatically. Only hand-qualify (`extern "system"`) when a ddk import drifts the signature into E0308.
3. **A vendored crypto lib gives ONE driver two conventions.** ksecdd links cryptlib: the **tomcrypt** primitives (`aes_*`/`des3_*`) are **cdecl** (`extern "C"`), but ReactOS's own crypt-lib **`MD5Init`/`MD5Update`/`MD5Final`** are **NTAPI stdcall** (`_MD5Init@4`, `extern "system"`). They sit in the same `crypt.c` — don't assume one block = one ABI; split by *which library* provides each symbol.
Plus two non-ABI link gotchas from the same drivers: (a) **x86 compiler intrinsics referenced on arm64** — `__rdtsc`/`_rotl` lose their C `#if defined(_M_IX86|_M_AMD64)` arch guard in translation → undefined on arm64; cfg-gate a glue def (`__rdtsc`: `rdtsc` on x86, `CNTVCT_EL0` on arm64). (b) **lld i386 cascade red-herrings** — an unresolved symbol in an object can make lld also report unrelated `Ki*`/`Hal*`/internal "undefined"s that **clear once the real undefined is fixed**; fix the obvious crypto/intrinsic undefineds and re-link *before* chasing the weird ones. (c) **git `core.ignorecase` collision** — a `Foo.c`→`Foo.rs` module whose name case-folds onto the crate-root `lib.rs` (e.g. `Lib.c`→`Lib.rs`) blocks `git add` (`will not add file alias`); rename the module (`flt_lib`).

### LLM appends `as u64` to every SIZE_T/ULONG_PTR value → i386-only E0308 (32-bit width)
**Symptom:** A driver crate is cargo-clean on amd64 but ~50 `error[E0308]: expected u32, found u64` on i386, all on `IoStatus.Information = X as u64`, `VerifyIrpOutBufferSize(Irp, X as u64)`, `ExAllocatePool*(_, size as u64, _)`.
**Root cause:** `SIZE_T`/`ULONG_PTR` are 64-bit on amd64 (so `as u64` matched) but **32-bit on i386**. The mechanical translation hardcodes `as u64` for every length/offset/`Information` value instead of the arch-width type.
**Fix:** Helpers that take a size take `usize`; assignments to arch-width fields use `as _` (infer the bindgen field type) — never hardcode `as u64` for a SIZE_T/ULONG_PTR/Information context. Note `SIZE_T`/`ULONG_PTR` in module scope often resolve to the hand-written `ntkrnl_sys::base` alias (`usize`), distinct from the bindgen `ddk` u64 — so prefer `as _`/`usize` over naming either. Caught landing partmgr-rust, 2026-06-12.
**Source:** `project_partmgr_rust_track.md`

### amd64 cpuid 32-bit `xchg ebx` dance silently truncates a caller's rbx pointer
**Symptom:** BUGCHECK 0x50 at a truncated address (e.g. `0xFFD03000` from `0xFFFFFFFFFFD03010`) near PsLocateSystemDll; multi-hour forward-search to pin.
**Root cause:** On amd64 every 32-bit register write zero-extends to 64 bits, so `xchg ebx_save:e, ebx` wipes the upper 32 bits of rbx — which is callee-saved and may hold a caller's 64-bit pointer across the inlined cpuid.
**Fix:** Split cpuid by arch: amd64 uses 64-bit `xchg {rbx_save}, rbx`; i386 keeps `xchg {ebx_save:e}, ebx`. Cast to u32 at the return-tuple boundary, not inside the asm (`hal-rust/src/x86/misc.rs:165-207`). Any amd64 inline asm touching rbx/r8d..r15d carrying pointers must use 64-bit ops.
**Source:** `feedback_cpuid_xchg_32bit_zext.md`

### PROCESS_ALL_ACCESS Vista value in access-mask storage → i386 SMSS hang
**Symptom:** i386 reaches KiInitializeKernel but loops before SMSS userspace; debugcon shows a periodic two-byte pattern (SAM/LSA wait loop). SMSS process-create silently fails.
**Root cause:** `ntkrnl_sys::constants::PROCESS_ALL_ACCESS` = `0x001FFFFF` (Vista+), but the C kernel is NT 5.x where it's `0x001F0FFF`. Using the Rust constant in `Psp{Process,Thread}Mapping.GenericAll` mismatches the C-built access-check callsite. (Surfaced when `kernel_c_helpers.c` was deleted and the Rust storage took effect.)
**Fix:** In `#[no_mangle] pub static mut` access-mask globals hard-code `STANDARD_RIGHTS_REQUIRED | SYNCHRONIZE | 0xFFF` (with a comment). `THREAD_ALL_ACCESS` (`...|0x03FF`) already matches — no workaround. Long-term: correct the Rust constant after a callers audit.
**Source:** `feedback_process_all_access_constant.md`

### Translated file re-declares C ENUM members as local consts with GUESSED values (SILENT scheduler/dispatcher deformation)
**Symptom:** none, at first — the kernel boots and works, but a whole behavior block is dead or misrouted. Found 2026-07-04: `ke/wait.rs` had `const AdjustUnwait: u8 = 3` (header: **1**) so `KiUnwaitThread` stamped a value `KiDeferredReadyThread` matches against nothing → the unwait priority-boost/quantum-recharge block was dead code for EVERY thread wakeup since the translation landed; `ke/thrdschd.rs` had Boost/Unwait swapped-and-shifted (1/2 vs the header's 2/1) so `KeSetEventBoostPriority` wakes took the wrong branch (over-boost to ~15 + bogus decrement). Sibling hits in the same sweep: `ThreadedDpcObject=7` (collides with `GateObject`!), `EventPairObject=2` (=MutantObject), swapped `NotificationEvent`/`SynchronizationEvent` in four files (an event created with the wrong type wakes ONE waiter instead of ALL — a lost-wakeup landmine), `SynchronizationTimer=0` (=NotificationTimer), wrong `KAPC_ENVIRONMENT` values.
**Root cause:** the translator re-declares enum members it needs as local `const NAME: T = <value>;` and *guesses* the value (often the intuitive-but-wrong one, or the value of a *different* enum's member with a similar name) instead of reading `ndk/ketypes.h`. Each file guesses independently, so setter and reader can disagree **while both compile clean and the reader is self-consistent within its own file**.
**Fix:** align to the header. **Detection is mechanical:** `PYTHONPATH=tools/rustconv python3 tools/rustconv/rustconv/verify_constants.py --enums <files>` — after 2026-07-04 the catalog includes the ndk type headers (ketypes/pstypes/obtypes/mmtypes/extypes/lpctypes/cmtypes: ADJUST_REASON, KTHREAD_STATE, KOBJECTS, KWAIT_REASON, EVENT_TYPE, TIMER_TYPE, KAPC_ENVIRONMENT, LPC_TYPE, PS_* pool/priority, MMVAD types, …). Run it over any kernel crate you're auditing; `--fix` auto-rewrites. **Anything it reports resolved to exactly ONE header value** (the ambiguity rule drops multi-enum names) → a strong real-bug signal, not the usual `--enums` noise. **Two deliberate blind spots you must review by hand:** (a) `TAG_*` pool tags are SKIPPED (Rust `from_le_bytes` vs the checker's big-endian multichar disagree on byte order — the tags need a byte-order-aware check; drift there is cosmetic pool-dump display); (b) untagged inline literals at call sites (`KeInitializeEvent(e, 1, 0)` with no `/* … */` comment) still escape — `scan_inline_flag_literals` only catches the commented ones. Prefer bindgen enums (`ddk::_EVENT_TYPE_*`) over local consts in new translations.
**Also caught by the same tool the same day** (once `pub const` / path-typed / suffixed-decimal parsing was added — the checker had been blind to ~27K `pub const`s): the **LPC message-type enum** shifted in `lpc/send.rs` (live `if MessageType < LPC_DATAGRAM || > LPC_CLIENT_DIED` validation window, csrss-CreateProcess reply path), `MEMORY_PRIORITY_FOREGROUND` writer(0)/reader(2) disagreement killing the scheduler foreground boost, swapped Ps pool-quota/priority enums, wrong MMVAD types, ~10 wrong hand-written STATUS codes. 41 total in one sweep.
**Source:** 2026-07-04 dispatcher audit + constant sweep (`docs/engineering/archive/sync-instrumentation-design.md` §8).

### Hardcoded HIGH_LEVEL=31 on amd64 → `mov cr8, 0x1F` #GP at boot
**Symptom:** BUGCHECK 0x1E with `c0000005` during boot-driver init / process create; reported faulting Rip unreliable (false symbol e.g. `MiAcquireExpansionLock+7`).
**Root cause:** HIGH_LEVEL is 31 on i386 but 15 on amd64/arm64; amd64 maps IRQL to CR8 which has only 4 valid bits, so writing 0x1F sets reserved bit 4 → #GP. (`pcibus.rs` `KeRaiseIrql(HIGH_LEVEL=31,...)`.)
**Fix:** `#[cfg(target_arch="x86")] const HIGH_LEVEL: KIRQL = 31;` else `= 15;`, or import per-arch from `hal_ddk`. Diagnosis: disassemble the suspect HAL fn and look for `mov cr8, <imm>` with imm > 15 — trust the instruction stream over the symbolicator for control-register #GPs.
**Source:** `feedback_irql_high_level_per_arch.md`

### i386 no-SSE float ABI: an `f32`/`f64` VALUE can't cross a fn boundary (translate C float helpers as macros; cfg-split float exports)
**Symptom:** a usermode-DLL Rust crate that builds with `-sse,-sse2` (the i686 CPU-baseline rule — gdi32/user32/etc., via `sdk/lib/rust/.cargo/config.toml`) fails to **compile** on i386 with `error: this function definition uses SIMD vector type 'f32'/'f64' which (with the chosen ABI) requires the 'sse' target feature, which is not enabled` — pointing at a fn *definition*, even with `#[inline(always)]`. amd64/arm64 build fine (SSE/NEON always present).
**Root cause:** without SSE there is no float ABI register, so rustc rejects any `f32`/`f64` **passed by value** across a function boundary (param or return) — it checks at the *definition*, so `#[inline]` does NOT help. What IS fine on no-SSE i386: float **math/values/stores inside a body** (x87), float **pointers** (`PFLOAT`/`*mut f32`/`LPABCFLOAT`), and float **struct fields**. Only the by-value ABI crossing is forbidden.
**Fixes (two flavors, both used in gdi32):**
1. **Internal C float helper → Rust `macro_rules!`, not a fn.** A `static __inline INT GDI_ROUND(FLOAT)` / `_fmul`/`_lrintf` / `fabs`-style helper translated as a Rust `fn(f32)->…` trips this. Convert it to a macro so the float math inlines at the call site and never crosses an ABI boundary: `macro_rules! GDI_ROUND { ($v:expr) => {{ let val: f64 = ($v) as f64; let t = val+0.5; let i = t as i64 as f64; (if i>t {i-1.0} else {i}) as INT }}; }`. **Keep the macro's internal type matching the C width** — C does world-to-viewport transforms in `double` then `(int)floor(v+0.5)`; narrowing to f32 in the macro silently loses precision. (gdi32 `coord.rs` `_fmul!`/`_lrintf!`; `font.rs` `GDI_ROUND!`/`dabs!`.)
2. **EXPORTED fn taking FLOAT by value → per-arch `#[cfg]` signature split.** e.g. `BOOL WINAPI SetMiterLimit(HDC, FLOAT eNewLimit, PFLOAT peOld)`. You can't drop the param and you can't make it `f32` on i386 — but the i386 stdcall ABI passes a 4-byte FLOAT **on the stack**, bit-identical to a `u32` on the stack, whereas amd64/arm64 pass it in a **vector register** (so they need the native `f32` sig — a `u32` there would corrupt the ABI). Split it:
   ```rust
   #[cfg(target_arch = "x86")]      // float arrives as stack bytes — take the bits directly
   pub unsafe extern "system" fn SetMiterLimit(hdc: HDC, a1: u32 /*FLOAT bits*/, a2: PFLOAT) -> BOOL { … NtGdiSetMiterLimit(hdc, a1 as LONG, …); *(a2 as *mut u32) = out_bits; … }
   #[cfg(not(target_arch = "x86"))] // float arrives in xmm/neon — native sig
   pub unsafe extern "system" fn SetMiterLimit(hdc: HDC, a1: FLOAT, a2: PFLOAT) -> BOOL { … NtGdiSetMiterLimit(hdc, a1.to_bits() as LONG, …); *a2 = f32::from_bits(out_bits); … }
   ```
   This mirrors the C, which reinterprets the float's bits as a LONG and passes the LONG to the kernel anyway (`gxf_long worker; worker.f = a1; …(worker.l)…`). The cfg-split is mandatory: a plain `f32` param won't compile on i386, a plain `u32` param corrupts amd64/arm64.
**Bonus (co-occurring): a `__forceinline`-from-a-header arithmetic helper is NOT a real export** — e.g. `RtlULongMult` (ntintsafe.h). Declaring it `extern` → undefined `_RtlULongMult@12` at link. Reimplement it locally as a small checked-arithmetic Rust fn (STATUS_SUCCESS / 0xC0000095 STATUS_INTEGER_OVERFLOW); see `win32k-rust/src/surface.rs` and gdi32 `font.rs`/`bitmap.rs`. (Sibling of "DDK FORCEINLINE exported only on i386" above, but here it's exported *nowhere*.)
**Source:** gdi32 Wave 2 (`coord.rs`), Wave 2b (`font.rs`), Wave 3a (`path.rs`/`bitmap.rs`) — `docs/engineering/archive/gdi32-translation-audit.md`.

### cohab flip drops a `.spec`-exported symbol that was a hidden OBJ-LEVEL alias in the flipped `.c`
**Symptom:** after a per-file cohab flip (`.c`→`.rs`), the DLL link fails with `undefined symbol: _Foo@N` for a symbol whose **literal name appears in NO `.c` file** — only in the `.spec` / a `.def`. The Rust crate compiles fine; it's a link-only break.
**Root cause:** the DLL's `.spec` exports `Foo` (e.g. gdi32 ordinal 273 `GdiEntry16`), and the flipped `.c` translation-unit was emitting `_Foo@N` *as an alias of a differently-named function it defines* (gdi32: `_GdiEntry16@12` aliased `DdSwapTextureHandles` in historic.c.obj — the C did it via a build/obj-level alias with no visible source macro). The `.rs` defines the real function but not the alias, so the spec export goes unsatisfied.
**Detection:** `undefined _Foo@N` + `grep -rn Foo **/*.c` returns nothing → it was an obj-level alias. Find the real target: grep the `.spec` (and any reference `.def` like ReactOS's `good api.def`, which had `GdiEntry16@12=DdSwapTextureHandles@12`) for the ordinal, and `nm` the **pre-flip** `.obj` to confirm `_Foo@N` lived there.
**Fix:** re-provide the alias explicitly in the `.rs` as a `#[no_mangle] extern "system" fn Foo(...)` forwarding to the real target — **all-arch** (the spec export is arch-independent; `_Foo@N` on i386, `Foo` on amd64/arm64). **Recurs across a family:** gdi32's `GdiEntry1..15` (gdientry.c, aliasing the `DdXxx` HAL fns) will each need this when that file flips. **Sibling bug (same gate):** a *blanket* "this family is cdecl/stdcall" rule that's wrong for one member — `EMFDC_*` were assumed cdecl but `wine/emfdc.c` declares `BOOL WINAPI EMFDC_GdiComment` (stdcall) → `undefined _EMFDC_GdiComment` until its decl was `extern "system"`. Verify each cross-file callee's *own* decoration against its C def.
**Source:** gdi32 Wave 3b (`historic.rs` GdiEntry16, `enhmfile.rs` EMFDC_GdiComment).

### a fn PARAMETER (or `let`) named like a glob-imported `const` becomes a const-PATTERN, not a binding → body sees the const's type
**Symptom:** a translated fn whose parameter is annotated as a pointer/struct type fails to compile with the parameter "found `u32`" (or whatever the const's type is) — e.g. `expected *mut _DDHAL_UNLOCKDATA, found u32`, plus cascade `error[E0614]: type u32 cannot be dereferenced` at every `(*param)` in the body. The annotation resolves correctly (the error even prints the right `*mut …::_DDHAL_UNLOCKDATA`), yet the binding is the const's type. Maddening because the *same* deref pattern compiles in a sibling fn whose param has a different name.
**Root cause:** Rust function parameters are irrefutable **patterns**. If an identifier in a pattern position matches a `const` in scope, it's parsed as a **constant pattern** (matching against that const's value), NOT as a fresh binding. The huge `win32k_sys::win32k::*` glob (and its `enum_aliases_*`) re-exports thousands of consts — e.g. `pub const Unlock: _SECURITY_LOGON_TYPE` (a u32 enum value). So `fn DdUnlock(Unlock: *mut _DDHAL_UNLOCKDATA)` binds nothing; every `Unlock` in the body resolves to that u32 const. (DirectDraw HAL params like `Unlock`/`Lock`/`Flip` are prime collision candidates against the LOGON_TYPE / DD_* / enum consts.)
**Detection:** when a param "is" the wrong scalar type despite a correct annotation, grep the glob for a `pub const <ParamName>:` — `grep -rn "pub const Unlock" sdk/lib/rust/win32k-sys/src/`. A sibling fn with the same body shape but a different param name compiling fine is the tell.
**Fix:** rename the parameter so it can't match a const — `Unlock` → `pUnlock` (update body refs; leave `.Unlock` *field* accesses alone). **⚠️ The dangerous variant is SILENT:** if the colliding const's type is *compatible* with the body's use, it compiles and the fn reads the const instead of its argument — a runtime bug with no error. So for translations against a big bindgen glob, prefer non-colliding param names (Hungarian `p*`/`lp*` prefixes, as the C often already has) rather than bare capitalized nouns.
**Source:** gdi32 Wave 4 (`gdientry.rs` DdUnlock/DdUnlockD3D — `Unlock` param vs `_SECURITY_LOGON_TYPE_Unlock` glob const). Cost ~an hour of "this param is u32 but it's annotated as a pointer?!" before spotting the const-pattern.

### a `#include`-built data TABLE has a different ROW SET under the wrong preprocessor config → wrong-but-compiling table (SILENT)
**Symptom:** a translated static table built from a header `#include` (the C does `static const T tbl[] = {\n#include "Data.h"\n};` where `Data.h` is rows of a `DEFINE_X(...)` macro) compiles and links clean, but has the **wrong number of rows** / wrong entries vs what the C build actually compiles. No error — the table is just silently wrong, so every lookup against it can return the wrong value at runtime.
**Root cause:** the rows in `Data.h` are guarded by `#if` on build macros (`_WIN32_WINNT`/`NTDDI_VERSION`/`WINE_FILEVERSION_MAJOR`/component defines like `_SHELL32_`). If the translator preprocesses with the *wrong* config, a different set of `#if` branches is active → a different row set. The classic miss: `_WIN32_WINNT=0x502` (NTDDI_WS03) instead of the module's real `-D_WIN32_WINNT=0x600` (NTDDI_WIN6) flips every `NTDDI_VERSION < NTDDI_LONGHORN` guard (note `NTDDI_LONGHORN == NTDDI_VISTA == NTDDI_WIN6`), and forgetting a component define (`-D_SHELL32_`) drops its guarded rows.
**Detection / fix:** translate the table against the **module's real CMake define set** (grep its `CMakeLists.txt` for `add_definitions`/`-D...`; shell32 forces `-D_WIN32_WINNT=0x600 -D_SHELL32_ -D__REACTOS__ -DUNICODE`). Verify the row set against **ground truth**, not a hand re-preprocess: `strings -el <the pre-flip .c.obj>` shows exactly which string literals (e.g. each row's key string) the real build compiled in — diff that against your table. A bare `clang -E` with hand-guessed `-D`s reproduces the bug (that's how the wrong count arose in the first place); the obj is authoritative.
**Source:** shell32 Wave 1 — `shpolicy.rs` `s_PolicyTable` (`#include "PolicyData.h"`, rows of `DEFINE_POLICY`). First pass preprocessed at WS03 → 169 rows; the real WIN6+`_SHELL32_` build is **159 rows**, confirmed byte-for-byte against `shpolicy.c.obj`'s UTF-16 strings (12 NTDDI-guarded rows dropped, 2 `>= NTDDI_LONGHORN` rows + 1 `_SHELL32_`-guarded row added). Same class to watch for the NLS/CSIDL/locale tables.

### NEVER translate a C debug MACRO into a no-op `fn` — make it a `macro_rules!`. An out-of-line no-op is a ud2 detonator (i386 BUGCHECK 0x1E / 0xC000001D)
**Symptom:** you take a translated **no-op debug fn** and give it an out-of-line body — e.g. move its variadic entry into a C trampoline so the crate can drop nightly `c_variadic` — touch *nothing* else, and i386 **BUGCHECKs 0x1E with exception `0xC000001D`** (illegal instruction) at a **`ud2` sitting immediately after `calll _TracePrint`**. amd64 and arm64 boot to desktop, clean, the whole time. You may also get a burst of `undefined symbol: _Name@N` link errors for functions unrelated to your change.
**Root cause — the no-op was a CLOAK, twice over.** The C original is a *macro* that expands to **nothing** in the retail build (`#define TracePrint(x)`, `storswtr.h:124`) — its arguments are never even evaluated. The translation rendered it as a no-op **`fn`**. That fn is trivially inlinable, so LLVM folded it away *along with every argument expression*, and with them any relocation reachable only through those arguments. Two whole bug classes were hiding under that fold:
  1. **§1 ABI mismatches.** Fns **defined** `extern "C"` (cdecl → `_Name`) but **declared** in an `extern "system"` block (stdcall → `_Name@N`). Clean on amd64/arm64 (the two coincide); on i386 they never even reached a link error because nothing referenced them.
  2. **C compile-time constants emitted as `extern "C" static`** (`static TRACE_LEVEL_ERROR: u32;` for what is a `#define`) — not linkable symbols at all.
Make the no-op **opaque** (an extern call LLVM cannot see through) and the arguments must materialise: the latents become `undefined symbol`s, **and** LLVM's inter-procedural reasoning across the now-visible mismatched-ABI callees starts proving continuations unreachable → the **ABI-mismatch `ud2` cascade**. Measured: cdrom.sys carried **100** `ud2` sites before (all on paths boot never hits — which is *why* master was green); the trampoline pushed it to **114**, and one of the new ones was on a live path.
**The trap:** the errors and the crash PC point at *innocent* code, so the instinct is "my change broke this driver". It didn't — it **uncloaked** what was already broken, and then added the detonator. Do not "fix" it by reverting to nightly.
**Fix (in this order):**
  1. **Make it a `macro_rules!`, not a fn** — the faithful shape (C never evaluates the args), zero cost, no dead call per site, and it does not create the opaque barrier. Consume args **by value** (`let _ = $arg;`): `&$arg` is E0793 on a `#[repr(packed)]` field. Do **not** type-bind the parameters — the per-module extern decls this replaces often *disagree* with each other (`u32` vs `i32` levels, `*const i8` vs `*const u8` formats), because nothing ever checked them against one definition.
  2. **Still fix the ABI mismatches you uncovered** — they are real. Split the mixed `extern "system"` block: kernel/WDF exports stay stdcall; the crate's own cdecl helpers move to `extern "C"`. Ground truth = the **definition** + the **C header** (no `NTAPI` ⇒ cdecl), and the linker itself (`undefined _Name` ⇒ it is stdcall; `undefined _Name@N` ⇒ it is cdecl). `verify_ffi.py` is the workspace-wide checker.
  3. `extern static` for a C constant ⇒ `const`.
**Source:** 2026-07-12 stable-toolchain lane, cdrom. `TracePrint` (962 call sites, empty body) given a C-trampoline variadic entry → i386 BUGCHECK `0x1E (0xc000001d)` at `cdrom.sys+0xBE58`, a `ud2` after `calll _TracePrint`; plus `undefined symbol: _DeviceSendRequestSynchronously@12` / `_DevicePowerSettingCallback@12` / `_RequestSenseInfoInterpret@24` (all defined `extern "C"`, all declared in `autorun.rs`'s `extern "system"` block, all declared NTAPI-less in `cdrom.h`) and `undefined symbol: TRACE_FLAG_IOCTL` / `TRACE_LEVEL_ERROR`. Converting `TracePrint` to a macro dropped ud2 114 → 100 and i386 booted. **amd64 and arm64 were green through every one of these states — i386 was the only arch that told the truth.**

---

## LLVM captures(none) / ud2 / codegen-unit miscompiles

The partial-Rust kernel trips a family of LLVM optimizer bugs where interprocedural analysis sees enough to optimize but not enough to be correct; these entries cover the symptoms, the `lto=false`+`codegen-units=256` firewall, and the workarounds that are (and aren't) still load-bearing.

**⚠ 2026-07-05 STATUS: the volatile-blind attribute-deduction root is FIXED in the shipping toolchains (LLVM 21.1.3 stable / 21.1.5 nightly — volatile stores now block `readnone captures(none)` deduction; plain out-writes deduce fold-proof `writeonly initializes(...)`), and a post-opt IR audit of the amd64 kernel body found ZERO committed folds** (every `ptr poison` callsite = a genuinely-unused source param). See rust-kernel-patterns.md "RESOLVED AT THE TOOLCHAIN". Layout-robustness is also empirically proven: the `VIBEOS_LAYOUT_CANARY=1` deliberately-perturbed kernel (451 symbols relocated) **booted 5/5** on amd64 — see the "✅ CLOSED" note under "Incomplete Rust cohab flip" below. The ABI-mismatch → `noreturn`/ud2 cascade entries below remain fully live (toolchain-independent). Detector: `tools/audit-captures.sh <arch>` (scan_captures.py over `VIBEOS_EMIT_LLVM_IR=1` builds) — rerun after every rustc/LLVM bump. Historical entries below are kept for the day a toolchain bump regresses this.

### ud2 in a NAMED function / success path folded away after a call
**Symptom:** `scan_ud2.py` reports `ud2` (0f 0b) in a named `#[no_mangle]` function; post-call code looks dead; cascade to 66+ functions made `noreturn`.
**Root cause:** LLVM's attribute pass deduces `readnone captures(none)` on output-pointer params (ignoring `write_volatile`), callers pass `ptr poison`, stores through poison are UB → body collapses to `ud2` → `unreachable` cascades. opt-level=1 introduces the attributes; opt-level=0 has none.
**Fix:** Workspace `lto=false` + `codegen-units=256` firewall (near-isolation per function); validated `opaque_ptr` asm barrier strips `captures(none)` from IR; reverted the global write_volatile sweep (tag `last-known-good`=a340b21c) — it reshuffled CGUs and made things worse (6→19 ud2). i386+amd64 6/6 smoke, 0 unexpected ud2.
**Source:** `feedback_cgu1.md`

### ud2 after a call is sometimes ABI mismatch, NOT captures(none)
**Symptom:** ud2 immediately after a call site in a fresh i386 translation; identical disasm to captures(none); amd64/arm64 link and run clean.
**Root cause:** Rust caller declared an i386 fn `extern "C"` (cdecl) but callee is `extern "system"` (stdcall); post-call ESP is wrong by arg_size → backend treats continuation as UB → unreachable → ud2. On amd64/arm64 the ABIs coincide so it links silently.
**Fix:** Diagnose ABI FIRST (1-second check) before captures(none) defenses — `verify_abi.py` or `nm | grep Fn`; `_FunctionName@N` decoration with an `extern "C"` decl = the bug. Move the decl into `extern "system" {}`. Caught: `IopInitializeDriverModule` calling `IopReadyDeviceObjects` (19a8676c).
**Source:** `feedback_ud2_abi_vs_captures.md`

### Stale/hoisted load through a static-mut table after a foreign call returns NULL/garbage
**Symptom:** Boot succeeds, early locking works, then a by-handle lookup wrongly rejects an object (e.g. `InitDcImpl` 0xc0000001, stock-object lookup NULL); a DbgPrint between the reads "fixes" it.
**Root cause:** Plain Rust loads through `static mut` base pointers (`gpentHmgr`/`gpaulRefCount`) get hoisted/cached across a foreign call into a CAS loop, so writes a sibling fn made aren't observed — same class as captures(none). Boot-time single-threaded init doesn't exercise it; only the cross-write path does → smoke catches it, cargo/links don't.
**Fix:** `core::ptr::read_volatile` at the read sites + `write_volatile` at the paired writes. PROVEN pre-emptive step for any win32k handle/object/refcount table (USER object.c: 6 read + 18 write_volatile → clean first boot, zero debug cycle). Apply during cargo-clean, before first boot.
**Source:** `feedback_win32k_handle_table_captures_none.md`

### i386 LLVM proves a pointer NULL even with volatile/black_box defenses
**Symptom:** On i386 only, LLVM hard-codes a pointer as null / the `else` branch; inline asm, `read_volatile`, and `black_box` all fail to stop it; same code works on amd64.
**Root cause:** i386 backend is more aggressive at null-pointer deduction/UB exploitation than amd64.
**Fix:** Go straight to structural fixes — C shim returns object address as `ULONG_PTR` (integer), Rust reconstructs via `with_exposed_provenance_mut` after a non-zero check; split large fns into `#[inline(never)]` helpers to break cross-fn deduction. Preference for outparams: integer-return C shim > PVOID C shim > inline asm. Emit `--emit=llvm-ir` and search for `unreachable` after the suspect call.
**Source:** `feedback_llvm_workarounds.md`

### #[no_mangle] alone doesn't stop inlining → boot hang
**Symptom:** Removing `#[inline(never)]` from shim_exports.rs causes an i386 boot hang.
**Root cause:** `#[no_mangle]` only preserves the symbol; LLVM can still inline the body into callers in the same CGU, then apply captures(none)/other opts that break kernel invariants.
**Fix:** Keep all 324 `#[inline(never)]` on shim_exports.rs — they are defense-in-depth alongside guards/write_volatile (one hides the body from LLVM, the other protects it if LLVM sees it).
**Source:** `feedback_inline_never.md`

### Volatile<T> on an exported static → boot hang
**Symptom:** Changing `static mut X: KIRQL = 0` to `static X: Volatile<KIRQL>` hangs i386 boot.
**Root cause:** `#[no_mangle]` non-`mut` statics may be placed in `.rdata` (read-only); writing them page-faults despite `UnsafeCell` interior mutability.
**Fix:** Don't wrap written `#[no_mangle] pub static` globals in `Volatile<T>`; keep `static mut` and wrap accesses in `read_volatile`/`write_volatile`. `Volatile<T>` is fine for internal or read-only statics.
**Source:** `feedback_volatile_static.md`

### Re-probe old C-shim captures(none) workarounds before keeping them
**Symptom:** A `*_shim` in `io_rust_shims.c`/`mm_rust_shims.c` carries a "captures(none)/black_box mis-fold" comment; some are dead code, some still load-bearing.
**Root cause:** Shims were authored before `codegen-units=256`+`lto=false` became default; the firewall makes many unnecessary. Comments ("even with volatile reads...") are stale because the original caller went through multiple shim hops.
**Fix:** Read the comment, grep callers, drop the trampoline and declare the Rust fn directly (`extern "system"` NTAPI / `extern "C"` cdecl), build 3 arches + `scan_ud2.py --check`, smoke. Every io workaround dissolved under the firewall (f5e37197, 2ba8dab4, 280782e5–fd141491, a1439467, 6ceb5f0f). Re-probing can also surface latent bugs by exercising newly-live paths. Must-stays: amd64 `KiSaveXState`/`KiRestoreXState` (KVM bug) + i386 `SepInitTokensShim` (fat-LTO ud2).
**Source:** `feedback_captures_none_reprobe.md`

### Keep volatile on MMIO accesses through translation/integration
**Symptom:** Translated/integrated driver enumerates garbage (no keyboard/mouse) though it compiles; `lint_kernel.py` flags nothing.
**Root cause:** `write_volatile`/`read_volatile` discipline applies to MMIO (`MmMapIoSpace MmNonCached`, C `volatile UCHAR *`) too, not just FFI outparams. A type error on `write_volatile(cfg.add(0), WIDE_CONST)` (cfg `*mut u8`) is "fixed" as `*cfg.offset(0) = WIDE_CONST as u8`, silently dropping volatile → LLVM reorders/coalesces/eliminates the select-write → size-read.
**Fix:** Cast the VALUE, keep volatile: `write_volatile(cfg.add(0), WIDE_CONST as u8)`. After any MMIO-touching driver, diff for removed `(read|write)_volatile` and compare `grep -c volatile` before/after; independently verify subagent self-reports (one was confidently wrong on 19 sites).
**Source:** `feedback_mmio_volatile_in_translation.md`

### Shared clock reads in spin-waits need volatile
**Symptom:** amd64/arm64-only driver boot hang immediately after entry begins; i386 works. A tight calibration loop like `KeQuerySystemTime(&t0); do { KeQuerySystemTime(&t1); } while (t0 == t1);` never observes the clock tick changing. uniata.sys hit this after returning from the initial `ScsiPortInitialize`: warm smoke stopped at `IopInitializeBuiltinDriver: uniata.sys`.
**Root cause:** Rust inline `KeQuerySystemTime` read `KUSER_SHARED_DATA.SystemTime` through ordinary raw-pointer loads. LLVM can reuse/hoist those non-volatile loads inside the spin-wait even though the kernel clock updates the page asynchronously. i386 called the real exported function, so it was immune.
**Fix:** Use `read_volatile` for `High1Time`, `LowPart`, and `High2Time` in the amd64/arm64 inline helper. Any fixed-VA shared page or MMIO-backed time source used as a polling condition needs volatile loads, not just a raw pointer.
**Source:** uniata-rust `KeQuerySystemTime` amd64 clean-smoke hang, 2026-06-13

### Incomplete Rust cohab flip — link breaks ONLY on a clean arm64 tree (stale x86 archives hide it)
**Symptom:** A C→Rust cohab flip (the cmake `libX.a` becomes a `rust_cohab_stub.c` placeholder; real symbols move to `libX_rust.a`) builds + boots fine on i386/amd64, but a clean/fresh arm64 tree fails at link with undefined symbols that "should" exist: `ld.lld: error: undefined symbol: CpInitialize / IoCsqInsertIrpEx / csStdIn / ConResPrintf / StringCchVPrintfExW`. Forcing a fresh relink of the *same* consumer on x86 still succeeds, so it reads as "arm64-only."
**Root cause:** the flip was only half-wired. Two failure shapes: (a) **cargo-consumer didn't add the rust archive** — e.g. `ntoskrnl-cargo/build.rs` listed the now-empty `libcportlib.a`/`libcsq.a`/`libarbiter.a` but never the sibling `libcport_rust.a`/&c.; (b) **a symbol the rust crate still references is inline/intrinsic on x86 but has no provider on arm64** — strsafe FORCEINLINEs (`StringCchVPrintfExW`), compiler intrinsics (`InterlockedCompareExchangePointer`), C variadic forwarders Rust can't define (`ConPrintf`→`ConPrintfV`), or a Rust `static` consumed by C but missing `#[no_mangle]` (`csStdIn`). **x86 only "works" because the stale pre-flip `libX.a` (or a weak inline copy clang emits on x86 but not arm64) still carries the symbol** — a truly clean x86 rebuild of `libX.a` would break identically. Verify with `llvm-nm libX.a`: a stub-only archive has 0–1 `T` symbols (just the cohab stub).
**Fix:** complete the flip so the rust crate is self-sufficient on every arch: (a) add `libX_rust.a` to each cargo consumer's `build.rs static_libs` (placed after the C placeholder; `--allow-multiple-definition` keeps C-first precedence and tolerates the transient stale dup); (b) make the crate provide its own symbols — `#[no_mangle]` on statics consumed by C; replace `extern` intrinsic decls (`InterlockedCompareExchangePointer`) with `core::sync::atomic`; replace strsafe-inline `extern`s with the msvcrt primitive they wrap (`StringCchVPrintfExW`→`_vsnwprintf`, cdecl `extern "C"`); add a small C variadic-shim TU to the cohab for the `...`→va_list forwarders. Audit ALL `static_libs` C archives on arm64 with `llvm-nm | grep -c ' T '` before trusting a flip.
**⚠ GATE THE `static_libs` ADDITION (a) TO `if arch == "aarch64"`.** On i386/amd64 the cmake C `libX.a` still carries the symbols (a rust-only `--clean` does NOT rebuild it to a stub), so the rust archive is REDUNDANT there — and adding it is NOT free: the extra in-group archives shift the kernel link LAYOUT, which tips a *pre-existing latent layout-sensitive miscompile on the cmlib `Hv*` hive-write path*. Symptom of getting this wrong: combined-tree **cold amd64 HANG** at `cmlib\hivewrt.c:360/522 "Failed to write the primary hive"`, deterministic across rebuilds, warm dodges it, stalls just after `[BASESRV] BaseInitializeVDM returning SUCCESS`. It reads as a uniata/storage bug but is NOT — the hang is in cmlib (a component you didn't touch), surfaced purely by your link-layout delta. Confirmed 2026-06-14: gating the 5 cohab-lib additions to arm64-only flipped amd64 cold from hivewrt-HANG → green, 0 hive failures (i386/amd64 stay byte-for-byte on the green baseline; arm64 still gets the libs it genuinely needs). See also "Workspace-composition change → warm-incremental FALSE-FAIL" + [[feedback_cargo_drive_reuse_a_redundancy]] (the latent hive miscompile is layout/cgu-sensitive and still lurks).
**🔑 PARTIAL FIX (cmlib hive site) — crp `d67f97bd87a`:** `HvpGetHiveHeader` (hiveinit.rs) wrote its `*HiveBaseBlock`/`*TimeStamp` output params with plain stores → captures(none)-elidable → DROPPED → caller (`HvLoadHive`) stores a garbage `Hive->BaseBlock`. **Fix = `write_volatile` the 6 output writes.** This DID cure the hive site (see validation below) — correct hardening, keep it. (NB: the "Failed to write the primary hive" `hivewrt.c:360/522` DPRINTs are **benign RO-media noise**, present on EVERY boot incl. green ones — they were a red herring, never the hang.)

**❌ VALIDATION RESULT (2026-06-14) — the fix is NECESSARY-BUT-NOT-SUFFICIENT; ungate stays gated.** Did the owed deterministic test: ungate the 5 cohab archives on amd64, cold build, smoke ×2 + GDB. Findings, all decisive:
- **Gated boots green; ungated deterministically faults.** Clean attribution (stash the 2 build.rs edits → gated → ✓ shell ready; restore → ✗). The earlier "compile-time incremental, not layout" theory is **DISPROVEN**: it reproduces on a *warm relink* with NO cmlib-rust recompile, and the hive site is already fixed.
- **It IS genuinely link-LAYOUT-sensitive, and EXTREME.** Bisected the 5 archives down to a **single** redundant archive (cport alone) → **byte-identical** fault. So it's not any archive's *code* — **ANY** perturbation of the kernel link tips ONE fixed latent miscompile. Params never move: `BUGCHECK 0x50 (referenced=0xfffff8a000093000, read, PC=0xfffff880055cd050)`.
- **The tip MOVED past the (now-fixed) hive site to the STORAGE stack.** Boot now sails through registry/hive init and faults during scsiport/uniata device enumeration (last builtin = `uniata.sys`; serial shows `scsiport/fdo.c:324 Found device …` right before the 0x50). PC `0xfffff880…` is in a *driver*; the bad pointer `0xfffff8a0…` is pool/hyperspace — a kernel fn handed the driver a garbage pointer when the layout shifted. Same captures(none) *class*, different (unnamed) site.
- **The ungate is the WRONG flip mechanism anyway.** `ar t libcportlib.a` = `cport.c.obj` + `rust_cohab_stub.c.obj` — the cmake archive **bundles the C original**, and `target_link_libraries(cportlib PUBLIC libcport_rust.a)` only propagates to *cmake* consumers, NOT to path-referencing `ntoskrnl-cargo`. So ungating adds a **redundant** rust archive that loses first-match to the bundled C and merely perturbs lazy-pull/layout. A *true* i386/amd64 flip must **replace** the C (drop `cport.c.obj` / the C archive), not add alongside.
- **Exact instruction BLOCKED by tooling.** `qemu/start.sh --gdb` `find-kbase` fails ("not in expected ranges") so symbol breakpoints resolve to unrelocated offsets and never fire; `addr2sym` only indexes ntoskrnl (driver PC → garbage offset); `debug=2` DWARF bloats the kernel so freeldr dies at "Switching stack…" (never boots). **Naming the site needs kernel-debug tooling repair (find-kbase) or split-debuginfo first.**

**Bottom line:** the deep-C cohab "ungate" cannot complete the flip — it's redundant AND it exposes a latent layout-fragile kernel miscompile that ANY link change re-triggers (the exact crp layout merely dodges it today). Keep the `if arch=="aarch64"` gate. Two real follow-ons, both bigger than a one-line fix: (1) repair `find-kbase`/symbolication, then name + `write_volatile` the storage-path site; (2) understand why the `cgu=256` firewall doesn't neutralize this within-crate captures(none) case (a true flip needs the kernel layout-robust, not just this one site patched).
**Source:** uniata arm64 bring-up 2026-06-14 — ntoskrnl-cargo + ntkrnlmp-cargo (cport/csq/arbiter/lsalib/rossym, arm64-gated) + conutils-rust (csStd*/StringCchVPrintfExW/InterlockedCompareExchangePointer/Con*printf forwarders); hive captures(none) candidate fix `d67f97bd87a`

**🔁 RE-CONFIRMED + the tip MIGRATED AGAIN (2026-06-15, staticlibs lane, on crp+LAPIC):** redid the amd64 ungate on a fresh crp merge (incl. the LAPIC clock fix `22356522c58` + the find-kbase PE32+ fix `5bba213f288`). Clean controlled A/B on one build dir: **baseline (no ungate) = 3/3 boot to desktop ~3m45s; ungate (+5 rust archives) = 3/3 HANG_TIMEOUT 6m01s.** Decisive new facts:
- **The miscompile MIGRATED a THIRD time.** 2026-06-14 it was the storage `0x50` (uniata enum); now it's a **late user-mode HANG** — PID 112 (csrss/services) wedges right at `[LDR] DllMain(crypt32.dll)` after loading the ws2_32→dhcpcsvc→iphlpapi→mbedtls→bcrypt→advapi32_vista chain. NO bugcheck; kernel alive (LAPIC `[CLKRATE]` ticking, `KiIdleLoop`); the user thread is blocked in a kernel call that never returns (classic captures(none) output-write elided → a wait/status never set). This **cements that it's a captures(none) CLASS, not one fixed site** — any link-layout perturbation tips whichever vulnerable site the current codegen exposes. So a per-site `write_volatile` is whack-a-mole; the durable fix is kernel layout-robustness (follow-on 2 below), not naming this one site.
- **find-kbase is now FIXED** (`5bba213f288`): `qemu/start.sh --gdb` resolves kbase (`find-kbase kbase=0x… reloc=0x…`). So symbolication of driver/late PCs works now — the 2026-06-14 "find-kbase fails" blocker is gone.
- **But a debuggable-yet-bootable kernel is STILL the wall.** `list-threads` needs Rust DWARF in the linked `ntoskrnl.exe`; the release kernel has `Rust=0`. Methods that DON'T get DWARF in: (i) `reactos-link` `compile()`'s `no_prepop`/RUSTFLAGS path — the **main body (`kernel-rust`) builds via `build_rust_crate` (lib.rs ~849), NOT `compile()`**, so editing the `no_prepop` match is a no-op on the body (this also means a `no-prepopulate-passes` systemic-guard test must go through `rust_build_rustflags`/`build_rust_crate`, NOT the `no_prepop` arm — the body was never actually tested with it); (ii) `[profile.release.package.{kernel-rust,ntoskrnl-rust}] debug=2` → `libkernel_rust.a` still has **0 DW_TAG** (per-package profile didn't apply under the `-Z build-std` json-target build). The method that DOES apply — workspace `[profile.release] debug=true` — **bloats the kernel so freeldr dies at "Switching stack…"** (per the 2026-06-14 note). **→ the real prerequisite for the GDB hunt is split-debuginfo: `-C debuginfo=2 -C split-debuginfo=packed` (or `objcopy --only-keep-debug` + strip the loaded image) so the booted PE stays lean but GDB has a sidecar `.dwp`/`.debug`.** That's a tooling sub-project, owed before the site can be named.

**Updated bottom line (2026-06-15):** the x86 cohab ungate is a confirmed layout-fragile *class* wall and remains **non-blocking** (baseline green; the 5 libs already run as Rust on arm64, as C on x86). Keep the `if arch=="aarch64"` gate. The durable path is two stacked sub-projects: **(A) split-debuginfo bootable-debug kernel** → walk PID 112's blocked thread → name the wait-site; **(B) kernel layout-robustness** for the captures(none) class (test `-C no-prepopulate-passes` on the kernel BODY via `build_rust_crate`/`rust_build_rustflags`, watching boot-size; understand why `cgu=256` doesn't neutralize within-crate/internal-helper captures(none)). Until (A)+(B), the ungate cannot land.

**✅ CLOSED (2026-07-05, whpx-bringup lane): the layout-fragility wall is GONE — proven empirically, tripwired permanently.** Three facts settled it:
1. **The flip completed by another route** (2026-06-20): the 5 deep-C cohabs went cargo-native `.rust_lib` on ALL arches — the "ungate" this entry blocks no longer exists on the link line, and that flip (a far bigger layout change than the 2026-06 perturbations) landed green.
2. **The captures(none) root is toolchain-fixed**: LLVM 21 (21.1.3 stable / 21.1.5 nightly — both shipping pipelines) respects volatile stores in attribute deduction, and plain out-writes deduce fold-proof `writeonly initializes(...)`. Post-opt IR audit of the shipping amd64 kernel (4335 CGU .ll files, `tools/scan_captures.py`): **zero live folds** — every poison callsite is a genuinely-unused source param. So sub-project (A) has no site left to name for this class, and the 2026-06 faults were probably never IR-committed folds at all (the class attribution was inferred from 2026-03 history, not IR-verified) — more likely an address-layout-sensitive **runtime** bug in the §9.13 Mm family, since fixed (or masked) by the constants/enum/LPC campaigns.
3. **Layout-robustness (B) proven empirically**: `VIBEOS_LAYOUT_CANARY=1` (ntoskrnl-cargo/build.rs) deliberately perturbs the kernel link with zero semantic delta — rtl-rust↔cnt-rust-lib order swap (the heavily-pulled pair; 451 symbols relocated, 0 provider flips per the `VIBEOS_LINK_MAP=1` lld-map diff) + cport/csq swap + a redundant rossym dup. **Canary kernel booted 5/5 to shell-ready** (2026-07-05, amd64 windows-host TCG). In the 2026-06 era this deterministically 0x50'd at uniata enum before the desktop.
Tripwires now standing: `tools/audit-captures.sh <arch> [--lint]` (rerun after every rustc/LLVM bump — a future LLVM could regress the volatile fix) and the canary build for any suspected layout-sensitivity report. Gotcha both share: env-only toggles don't re-run the cargo edge (§9.6b — ninja DEPENDS watches files, not env); touch `ntoskrnl-cargo/build.rs` and **verify the kernel md5 actually changed** (the first canary attempt no-op'd twice: once on stale-edge, once because cport/csq/rossym-dup extract 0 members — a duplicate archive AFTER the original pulls nothing).

### cnt-rust CRT shim returns garbage / hangs ONLY on amd64/arm64
**Symptom:** A `libcntpr` CRT symbol misbehaves only on amd64/arm64 — memset dead-store-eliminated into a guard-only spin (freeldr hung after "Starting freeldr from UEFI"); `_strnicmp` collapsed to "always equal"; or (cargo-drive) arm64 ntdll printf misformats (`0xc000003a`→`0xc`) → 0x6A bugcheck.
**Root cause:** After the per-fn cgu-split, tiny one-shim-per-CGU objects let LLVM's full pipeline miscompile isolated shims. i386 always carried `-C no-prepopulate-passes`; amd64/arm64 (and any cargo-driver scrubbing RUSTFLAGS) never got it. It's a class bug, not whack-a-mole.
**Fix:** Set `RUSTFLAGS=-C no-prepopulate-passes` for amd64+arm64 in `libcntpr.cmake` (c2b6df000d2) AND in `reactos-link`'s `build_rust_crate` when `crate_name=="cnt-rust-lib"` (i386 also `-C target-feature=-sse,-sse2`). Suspect this flag first for any amd64/arm64-only cnt-rust misbehavior.
**Source:** `feedback_cnt_rust_no_prepopulate_amd64_arm64.md`

### lld "X was replaced" relinking against fresh libcntpr_rust.a
**Symptom:** `ld.lld: error: X was replaced` (winsrv `__wcsicmp`, win32k `__itow`, ftfd `memmove`/`memset`) on a fresh module/exe relink — distinct from the benign `--allow-multiple-definition` warning.
**Root cause:** Import-vs-local-def collision from a MONOLITHIC codegen unit. `gen_cnt.py` emitted all ~100 CRT shims into one Rust module → one cgu object; referencing any symbol extracts the whole object, dragging strong defs that collide with ntoskrnl imports. C had one .obj per function (archive-member granularity). Deterministic per build, NOT a cgu-lottery; not the eh_personality stub.
**Fix:** Wrap each generated shim in its own `mod mX { ... } pub use mX::*;` in gen_cnt.py (rewrite `super::`→`super::super::`) → rustc partitions each into its own cgu (1 monolith → 110 i386 / 113 arm64). Semantically inert (a21d879c5cc). Diagnosis trap: a 1s relink "FAIL" = relink against a STALE `.a` — also rm the copied `.a` and the stale C `liblibcntpr.a` (ar never prunes).
**Source:** `feedback_cnt_monolithic_cgu_was_replaced.md`

### Adding a workspace member shifts unrelated crates' CGUs → "crp regression"
**Symptom:** A downstream worktree's cold-smoke hits a ud2 in a `KNOWN_CGU_UD2` fn (e.g. KiSystemStartup, PopSetSystemPowerState) → BUGCHECK 0x31, flagged as a continue-rust-port regression.
**Root cause:** rustc's `cgu=256` partitioner isn't deterministic across workspace shape; adding a crate to `sdk/lib/rust/Cargo.toml` `members` repartitions unrelated crates, potentially moving a ud2-replaced (parked, off-boot-path) body onto a live caller's path.
**Fix:** Reproduce on the REAL crp tree, deep-nuked (all 5 `rust-*-target/` + every .a/.exe/.sys/.dll + livecd.iso — conftest only nukes one of five), before accepting any crp-regression claim. Real instance (worktree-smss-csrss cf6637ca): crp itself was green 10/1. Structural fix: move the affected crate toward CGU=1 or hand-audit the KNOWN_CGU_UD2 fn for the real captures(none)/ABI bug.
**Source:** `feedback_workspace_member_cgu_shift.md`

### Parallel cold builds → false HANG at "[freeldr] Switching stack..."
**Symptom:** Three concurrent `vibeos-cli build --arch X --clean` then smoke; i386+amd64 both HANG_TIMEOUT dead at `[freeldr] Switching stack...` with zero kernel serial; the same build solo boots green.
**Root cause:** That line precedes the jump to `KiSystemStartup` (a `KNOWN_CGU_UD2` fn). Under memory/CPU pressure from 3 concurrent cold cargo builds, the non-deterministic `cgu=256` partitioner lands a ud2 body on the live early-boot path → kernel traps on entry. Build-time contention, not a code regression.
**Fix:** Build cold ONE arch at a time (smokes may run parallel — separate slot-capped VMs; cold builds must not). A zero-serial hang at "Switching stack..." after a loaded/parallel build is the prime lottery suspect — re-run the cold build solo before triaging. Escalate only if it reproduces SOLO and deterministically (then `scan_ud2.py` + deep-nuked crp repro).
**Source:** `feedback_parallel_cold_builds_cgu_lottery.md`

### Same-host contention false-red is NOT only `HANG_TIMEOUT` — it also shows as a transient BUGCHECK and a *late*-boot hang
**Symptom (variant A — transient BUGCHECK):** a single i386 smoke comes back **`BUGCHECK 0x1E` (e.g. 0xc0000006 IN_PAGE_ERROR)** while another arch is mid **cold-gate** on the same host; the *identical binary config* re-run on a quiet host boots green, reproducibly (user32 lane 2026-06-20: all-5-Rust config 0x1E'd once during a concurrent amd64 cold-gate + arm64 TCG boot, then booted green ×5 quiet). The contention class (previously documented only as `HANG_TIMEOUT` at freeldr) **also manifests as a transient BUGCHECK under memory/IO/KVM pressure** — don't root-cause a single contended red as a code regression.
**Symptom (variant B — late-boot HANG that reaches `shell ready`):** the i386 OS prints `[WIN32K] shell ready` (often ×3) but explorer never reaches `[EXPLORER] _tWinMain entered` within the 360s timeout. A normally-1m48s i386 boot drifts past 6min under concurrent boots. `shell ready` is an intermediate **milestone**, not the verdict marker — success = `[EXPLORER] _tWinMain entered`.
**Rule:** before declaring an i386/amd64 smoke red, confirm no other agent is cold-building or booting (`./vibeos-cli ps`, `#swarm`); a lone red (HANG *or* BUGCHECK) while another arch is mid-gate is prime contention-flake suspect. Re-run host-quiet to decide. (Distinct from the real pre-existing ~33% i386 flake where `winlogon` unconditionally spawns `nettest.exe` racing explorer's csrss/win32k handoff — that HANGs even host-quiet; serial tail shows `[WINLOGON] nettest.exe created` + no `_tWinMain`. Per-arch auto-serialization removes the contention variants.)
**Source:** user32 lane 2026-06-20 (Wave-1 cohab bring-up).

### usermode-DLL cohab integration traps (kernel32/user32 → Rust, against a bindgen `*-sys` glob)
Reusable footguns when integrating rustconv output into a usermode-DLL cohab crate that globs a bindgen `*-sys` crate (`win32k_sys::win32k::*`):
- **`extern "fastcall"` is i386-ONLY.** Rust mis-ABIs/rejects `fastcall` on amd64/arm64 (Win32 `FASTCALL` is the single standard convention there). Every `extern "fastcall"` block AND every `#[no_mangle] ... extern "fastcall"` export (cross-TU `@Name@N` symbols like `ValidateHwnd`, `IsMsgOverride`, `ClassNameToVersion`) MUST be cfg-split: `#[cfg(target_arch="x86")] extern "fastcall"` + `#[cfg(not(target_arch="x86"))] extern "C"`. i386's `@Name@N` decoration is load-bearing for the cohab link; only the non-x86 path changes. Use one private `_impl` body + two thin cfg'd `#[no_mangle]` wrappers to avoid duplicating bodies.
- **bindgen flattens nested-`_HEAD` structs into `__bindgen_padding_0`.** `THRDESKHEAD`/`THROBJHEAD`/`_PROCMARKHEAD` lose `head.h`/`head.pti` (collapsed into `[u32; N]`). C's `obj->head.h` → `addr_of!(head) as *const HANDLE` (h is offset 0); `head.pti` → the right padding index (`[2]` on THRDESKHEAD). Reuse one `UserHMGetHandle`/`wnd_pti` helper; grep the bindgen struct per arch.
- **A local `let Foo = ...` where `Foo` is also a glob-imported `const` silently becomes a refutable const-PATTERN match, not a binding** → `expected *mut X, found u32`. win32k-sys glob-exports PnP/enum-alias consts with innocuous names (`Root` = a `_PLUGPLAY_VIRTUAL_BUS_TYPE`); `let Root: PWND = ValidateHwnd(...)` breaks. Rename the local (`pRoot`).
- **Two bindgen `*-sys` crates can't both be glob-imported** (E0659 ambiguous `BOOL`/`DWORD`/`HWND` from distinct-but-layout-identical types). Pick ONE for the type bulk (win32k-sys); declare the other's functions as **local `extern` blocks using the chosen crate's types** (ntdll Rtl*/Zw* resolve via the importlib). Also drop `crate::types::*` (hand-rolled BOOL=i32) — it collides with the glob; shadow `const FALSE/TRUE: BOOL` locally (win32k-sys emits them as `u32`).
- **FORCEINLINE macros are NOT exports** — `RtlInitEmpty{Ansi,Unicode}String`, `NtCurrentTeb`, `RtlGetProcessHeap`, the `NtUserx*` ntwrapper inlines have no symbol; declaring them `extern` → undefined at link. Reproduce as `#[inline] pub unsafe fn`. rustconv opus routinely mis-emits these as `extern` AND as `ntkrnl_sys::ddk::` (kernel-crate) imports — both rewritten during integration (use a shared `internal.rs`).
**Source:** user32 lane 2026-06-20 (Wave-1: 9 .c→Rust, i386 cold-green).

### Dead "diagnostic" instrumentation in trampolines
**Symptom:** A C trampoline carries volatile `__builtin_return_address` captures + DPRINT1 tracebacks marked "load-bearing tracing"; deleting it breaks the build or leaves all-NULL output.
**Root cause:** The capture chain is often dead — its only reader (e.g. `MmDebugWsFaultCheck`) itself has no callers, so the diagnostic would print `(null)` regardless; the "load-bearing" comment is stale.
**Fix:** Trace the FULL consumer chain (what READS the state, not what writes it) via two-step verification (extern decl + actual call). If the reader has no callers, delete writer + reader + callers together. Genuine bug-catchers (IRQL violations, captures(none) checkers) belong in `ntkrnl_sys::inline::mm` with per-arch UART helpers, not in a C trampoline forever. Don't trust the in-source comment.
**Source:** `feedback_dead_debug_instrumentation.md`

---

## bindgen / struct-layout / string-literal width traps

Bugs that link clean and compile clean but corrupt memory or strings at runtime — derived from per-file translation footguns where the Rust mirror, literal width, or struct offset silently disagrees with C.

### Hand-built kernel wire struct as `#[repr(C, packed)]` mislocates a field vs the naturally-aligned C struct
**Symptom:** a value passed in/out via a hand-rolled mirror of a C struct comes out shifted by a few bytes (e.g. a TDI connect to 10.0.2.100 is parsed as 2.100.0.0); the operation silently fails (connection never establishes, request rejected). Compiles + links clean. The CONSUMER (real kernel/driver) builds the struct with natural alignment, so only the hand-built copy is wrong.
**Root cause:** declaring the mirror `#[repr(C, packed)]` removes the alignment padding the real `#[repr(C)]` struct has. Classic case: `TDI_ADDRESS_IP { USHORT sin_port; ULONG in_addr; ... }` — the `ULONG` forces 4-byte alignment, so C inserts **2 padding bytes after `sin_port`**, putting `in_addr` at offset 4. A packed mirror puts it at offset 2 → every reader using the real (aligned) layout reads 2 bytes early. The kernel's own parser was correct all along; only the hand-built test/FFI struct was wrong.
**Fix:** mirror the kernel's NATURAL alignment — either use `#[repr(C)]` (let Rust insert the same padding) or, if you need `packed` for a wire buffer, add the explicit `_pad` fields the aligned struct has. Cross-check field offsets against the consumer's struct, not just the field list. **Companion footgun:** a synchronous-IRP test helper that waits with `KeWaitForSingleObject` must check the WAIT return — on `STATUS_TIMEOUT` the IRP never completed and the stack `IO_STATUS_BLOCK` is still zero (== STATUS_SUCCESS), so reading it reports a FALSE "OK" that masks the real failure (this pair hid a netstack TDI round-trip that never actually connected, 2026-06-13).

### amd64 BUGCHECK 0x1E (STATUS_DATATYPE_MISALIGNMENT 0x80000002), i386/arm64 fine, after a flip with a UNICODE_STRING name
**Symptom:** amd64-only deterministic 0x1E at a fixed win32k/probe PC; P1 = `0xffffffff80000002`; probe arg register holds an ODD pointer (e.g. rcx=`0x7ff87028453`, rdx=0x14=10 WCHARs).
**Root cause:** rustconv emits `b"C\0S\0D\0V\0...".as_ptr() as *mut u16` for a wide-string `UNICODE_STRING.Buffer`. `b"..."` is `&[u8;N]` with alignment 1, so it can land at an odd address; amd64 `ProbeForRead(buf,len,sizeof(WCHAR))` enforces 2-byte alignment and raises, unhandled under DUMMY_PSEH.
**Fix:** declare a real `static X: [u16; N] = [...]` (alignment 2 ⇒ always even) and use `.as_ptr() as *mut u16`. Arch-green elsewhere does NOT exonerate — only amd64 trips when address is odd AND probe enforces.
**Source:** `feedback_byte_literal_wchar_misalign.md`

### Hand-encoded `[u16; N]` literal one short — silently truncated registry/object path (ERROR_FILE_NOT_FOUND from a "correct" call chain)
**Symptom:** an API fails with ERROR_FILE_NOT_FOUND / STATUS_OBJECT_NAME_NOT_FOUND on a path that provably exists; every function in the chain reads as a faithful translation; compiles + links clean; boots green (the broken path may simply never be exercised by boot — this one hid in `RCreateServiceW` until the kmtest conformance sweep called it, 2026-07-10).
**Root cause:** a wide-string constant hand-encoded as `static X: [u16; N] = [b'S' as u16, ...]` was sized one element short, so the LAST character was dropped to make room for the NUL — `SERVICES_KEY [u16; 33]` spelled `System\CurrentControlSet\Service` (no trailing `s`), and services-rust opened a nonexistent parent key. The intent comment above the array says the right string, so review eyes slide right past it; only decoding the bytes shows the truncation.
**Fix:** count = `chars + 1`; better, avoid hand arrays entirely (`w!()` / `wstr!` macro or a `const fn` UTF-16 builder). To sweep for siblings, decode every `static NAME: [u16; N]` against its `// L"..."` intent comment (one-shot scanner did this 2026-07-10; SERVICES_KEY was the only true hit). **The durable lint LANDED 2026-07-15**: `tools/scan_wide_literals.py`, gate-wired
(`vibeos-cli check`), baseline EMPTY. It decodes every `static/const NAME: [u16; N]` and diffs it
against the `// L"..."` comment above it; findings name the dropped characters and the needed
element count. It found one more real survivor immediately — hal `x86/kdpci.rs` `KEY_NAME [u16; 60]`
spelled `…\PCI\Debu` (missing the `g`), *dead* behind `let _ = KEY_NAME;` because the author hit
the bug, wrote "g missing — pad it", and worked around it with `build_wstr` instead of fixing it
(deleted). Two design notes worth keeping, both learned by the first fleet run reporting **20
findings of which 19 were its own gaps**:
* **The oracle is prose, so it must not over-read prose.** `L"..."` appears inside comments that are
  *about the syntax* ("Rust doesn't have `L"..."`, so spell them out") and inside *abbreviated*
  paths (`L"\Registry\Machine\...\Version"`). Neither is a specification — skip any intent
  containing `...`, and only take a CONTIGUOUS `//` run (a blank line means the comment is not this
  literal's spec).
* **No blanket "must end in NUL" rule.** UNICODE_STRING Buffers legitimately have none (`Length`
  carries the size) — five correct ones exist in iomgr/fsrtl/npfs. Fire only when the comment
  itself claims "NUL-terminated". Also strip *all* trailing NULs on both sides: hal `usage.rs`
  really has a double terminator, and rtl-rxact's comment spells the NUL (`L"RXACT\0"`).
Mutation-tested both directions (`tools/test_scan_wide_literals.py`, 12 cases, in `check-tooling`);
it reports coverage (`85/782 had an intent comment`) because "0 findings" over 0 comparisons is the
green that means nothing.
**Triangulation recipe that found it:** per-stage DbgPrint bisection in the failing RPC (one boot) + a parallel subagent diff-audit of the Rust vs upstream C (found the literal by byte-decode) + a second subagent proving the create call *cannot* return 2 (pinning it on the parent open). Three probes, one statement.

### STATUS_OBJECT_PATH_SYNTAX_BAD (0xC000003B) / garbage wide path from a bindgen `#define X L"..."`
**Symptom:** boot reaches CSRSS, `NtUserInitialize failed 0xc000003b`, "Cannot open winsta"; the wide string reads as garbage code units (`\W`,`in`,`do`,`ws`...). Deterministic on amd64.
**Root cause:** bindgen lowers a C wide-string `#define` to a NARROW `&[u8;N]` (drops the `L`); the translator then casts `X.as_ptr() as *const u16`, reinterpreting ASCII byte-pairs as UTF-16.
**Fix:** never cast the narrow const to `*const u16`. Build a real UTF-16 twin via `const fn ascii_to_u16<const N:usize>(s:&[u8;N])->[u16;N]` and use `X_W.as_ptr()`. Grep flipped files for `<CONST>.as_ptr() as *const u16` on any bindgen `&[u8]` wide-define.
**Source:** `feedback_bindgen_wide_define_narrowed.md`

### STATUS_OBJECT_NAME_INVALID (0xC0000033) → 0xc000021a → BUGCHECK 0x4C from a too-short name buffer
**Symptom:** truncated UNICODE_STRING (everything fits except the last literal); CsrClientConnectToServer fails, critical client dies.
**Root cause:** `size_of_val(X)` on a bindgen-mangled NARROW `&[u8;N]` (where C had `L"..."`) returns N, not N*2 — the alloc/length math undersizes by half. The append often already uses a hand-made `[u16;N]` twin, so only the size math is wrong.
**Fix:** use the wide twin in BOTH the append AND the size math: `size_of_val(&X_W)` (=N*2). Audit `size_of_val(<bindgen string const>)` near UNICODE_STRING / RtlAllocateHeap / MaximumLength after a flip.
**Source:** `feedback_sizeof_narrow_bindgen_wide_literal.md`

### Garbage path / EINVAL from `extern static Foo: *const i8` where C declared an array
**Symptom:** consumer passes the pointer to a string function that returns "not found"/EINVAL even though C knows the buffer is populated (e.g. FsOpenFile saw garbage DefaultPath, ArcOpen EINVAL, "Error opening freeldr.ini").
**Root cause:** rustconv translates C `extern CCHAR Foo[N]` (an array) as `extern static Foo: *const i8` (a pointer). Rust reads the first 4/8 buffer bytes (`"mult"`=0x746C756D) and uses them as a pointer.
**Fix:** mirror the array type — `extern { static Foo: [i8; 260]; }` then `Foo.as_ptr()`; or prefer a C accessor (`FrLdrGetBootPath()`). Grep `extern "C" { ... static .*: \*(const|mut) (i8|c_char)` against array-typed C symbols.
**Source:** `feedback_extern_array_as_pointer.md`

### BUGCHECK 0x1E in `wcsnlen`/`swprintf` from a wide string literal sized with no room for the NUL
**Symptom:** deterministic BUGCHECK 0x1E (0xC0000005 read fault) deep in the kernel's wide-string formatter — backtrace is `swprintf → wstreamout → streamout_impl::<u16> → case_string::<u16> → wcsnlen` faulting on a garbage `%ws` pointer; the immediate caller is your driver formatting a registry/path string. (uniata.sys C++→Rust, 2026-06-13: `AtapiRegCheckDevLunValue` → `swprintf(namex, FMT_CHAN, NamePrefix, chan)`.)
**Root cause:** the translation builds a wide C-string literal as a fixed array sized to the *exact* character count, dropping the NUL terminator — e.g. `const fn wlit<const N>(b:&[u8])->[u16;N]` called `wlit::<13>(b"%s\\Chan_%1.1d")` (13 chars → `[u16;13]`, no trailing 0). `swprintf`/`RtlQueryRegistryValues` scan for the NUL, walk off the end of the array into adjacent garbage, hit a bogus `%s`/`%ws` conversion, and `wcsnlen` faults. Inconsistent across sites: some `wlit::<N>` were sized `len+1` (NUL OK), some `len` (bug).
**Fix:** size every wide/narrow literal array to `byte_len + 1`. Harden the builder so it can't recur: `const fn wlit<const N>(b:&[u8])->[u16;N] { assert!(N > b.len(), "room for NUL"); ... }` turns every too-tight call into a build error. Audit: regex `wlit::<(\d+)>\(\s*b"((?:[^"\\]|\\.)*)"` and flag where `N <= decoded_len` (count `\\`/`\n` as 1 byte).
**Source:** uniata-rust `id_ata_part3.rs` (the `wlit` helper); 6 of 52 sites were undersized.

### Silent csrss/PnP hang or kernel BUGCHECK indexing a bindgen flexible array past 0
**Symptom:** enumeration processes element [0] then dies before [1]; csrss silently dies (no panic output) at i=1 → BUGCHECK 0x4C STATUS_SYSTEM_PROCESS_TERMINATED; amd64 silently corrupts/hangs (no bounds-check crash visible).
**Root cause:** bindgen models C `T arr[ANYSIZE_ARRAY]`/`T arr[1]` as `arr: [T; 1]`. Rust `arr[i]` for i≥1 inserts a bounds check that panic-aborts under `panic="abort"`; C intentionally over-allocates so the memory is valid.
**Fix:** use raw-pointer arithmetic — `arr.as_mut_ptr().add(i)` or `core::ptr::addr_of_mut!((*p).arr[0]).cast::<T>().add(i)`. Watch DEVICE_RELATIONS.Objects, CM_RESOURCE_LIST, IO_RESOURCE_REQUIREMENTS_LIST, KEY_VALUE_FULL_INFORMATION, CsrRootProcess.ServerData, **FILE_FULL_EA_INFORMATION.EaName** (`[CHAR;1]`, read the NUL terminator at `EaName[EaNameLength]`). Note: for `static mut FOO: [T;N]`, `addr_of_mut!(FOO).add(i)` strides by whole-array size — use `FOO.as_mut_ptr().add(i)`.
**Variant — SILENT THREAD HANG (not a bugcheck), message went to debugcon not serial (2026-06-13):** `IoCheckEaBufferValidity` (iomgr/util.rs) translated C `Current->EaName[EaNameLength] != 0` literally → `(*Current).EaName[16]` on `[CHAR;1]` → `panic_bounds_check`. The kernel-rust panic handler (kernel-rust/src/lib.rs) writes `PANIC:<file>:<line>` to **debugcon port 0xe9** (the `-debugcon file:...` sink, NOT the COM/serial log) then `loop {}`. On QEMU's single vCPU the timer still preempts the spinning thread, so OTHER threads keep running — symptom looked like one operation wedging forever (here: opening `\Device\Tcp` WITH an EA buffer, the AFD/winsock path) while the rest of the system was live, with no message in the serial log. **Diagnosis when a single op wedges but the box stays alive: check the debugcon capture for `PANIC:`** before assuming a lock/IO-manager hang. Triggered the FIRST time any EA-based create ran (AFD/named-pipes); fixed kernel-wide by `*(*Current).EaName.as_ptr().add(EaNameLength as usize)`.
**Source:** `feedback_bindgen_flexible_array_index.md`, `feedback_flexible_array.md`

### amd64/arm64 AV (BUGCHECK 0x1E 0xC0000005), i386 fine, from a hand-computed win32k flattened-head offset
**Symptom:** deterministic amd64 AV at a fixed win32k PC (e.g. explorer SetTimer → IntSetTimer → head_pti → deref `pti->Tcb.Win32Thread`); trapframe shows a dword-swapped garbage pointer (`Rcx = 0x0132201000000000`).
**Root cause:** bindgen flattens `HEAD{HANDLE h; DWORD cLockObj} pti` into `__bindgen_padding_0`, hiding `pti`. The wrong formula `size_of::<HANDLE>()+size_of::<DWORD>()` gives 8 (i386, correct) but 12 on 64-bit — it misses 4 bytes of tail padding before the 8-aligned `pti` (real offset 16).
**Fix:** head-pti offset = `2 * core::mem::size_of::<usize>()` (8 i386 / 16 64-bit). But do NOT blanket-apply `2*usize`: derive each offset from the actual field sequence and the target field's alignment — e.g. `PROCESSINFO.W32PF_flags` after `{PEPROCESS; DWORD}` with no tail padding = `size_of::<usize>()+size_of::<u32>()` (8/12).
**Source:** `feedback_bindgen_flattened_head_pti_offset_padding.md`

### Cross-DLL reads return garbage past the writer's own init; hang ~7 DLLs later in an unrelated consumer
**Symptom:** boot gets past the writer (writes only a few fields) and faults/hangs much later in a shared-section consumer; a known constant (e.g. 0x409 US LCID) appears where a string length should be.
**Root cause:** rustconv emits a broken `#[repr(C)]` mirror of a shared struct — scrambled field order, opaque `[u8;N]` blobs instead of real nested types, or missing fields. Also: a per-module mini-mirror placing a field at offset 0 clobbers the real struct, and `type X = c_void` ⇒ `size_of::<X>()==0` ⇒ ~0-byte alloc.
**Fix:** rebuild the mirror in exact C field order using real size-asserted bindgen nested types (UNICODE_STRING, NLS_USER_INFO, etc.), NOT `[u8;N]`. Pin with arch-gated `const _: () = assert!(size_of::<T>() == 0xNNNN);` matching the C's arch-gated `C_ASSERT` (basesrv BASE_STATIC_SERVER_DATA == 0x1AC8 i386). Never feed a helper a pointer to a private mirror — pass `addr_of_mut!((*real).Field)`. Replace `type X=c_void` with a real struct + size assert (INIFILE_MAPPING==16, BASESRV_API_CONNECTINFO==4).
**Source:** `feedback_struct_mirror_layout_repair.md`

### Critical process terminates (BUGCHECK 0xc000021a / 0x4C) with no obvious reason after a cohab flip
**Symptom:** a bit-test never matches / a session never found; a consumed field reads as a handle-shaped value (0x3D0, matching boot handles like 0x3DC).
**Root cause:** rustconv emits per-file LOCAL re-definitions that shadow the shared bindgen ones. (1) Scrambled const: smss had `SMP_SUBSYSTEM_FLAG=0x02` (real 0x08), so a writer's 0x08 never matched a reader's `&0x08`-via-`0x02`. (2) Truncated struct mirror: `SMP_SUBSYSTEM { Entry, MuSessionId }` put MuSessionId at offset 8 vs the real offset 36 — the reader read the Event HANDLE as MuSessionId. A truncated mirror is worse than a scrambled full one: no size to assert, and "...other fields not accessed" hides the offset bug.
**Fix:** delete the local re-defs; rely on the shared `*-sys` bindgen (file already has `use ntdll_sys::nt::*`). Grep `const SMP_\|struct .*SUBSYSTEM\|struct .*SESSION` and diff every per-file def against the bindgen + C header.
**Source:** `feedback_local_redef_shadows_bindgen.md`

### arm64 csrss CsrServerInitialization → STATUS_ACCESS_VIOLATION (0xc0000005) from hardcoded SD min length
**Symptom:** arm64 csrss faults in CsrSetProcessSecurity; the DACL lands inside the SD's own pointer fields; `Dacl - ProcSd == 0x14` where it should be 0x28.
**Root cause:** `SECURITY_DESCRIPTOR_MIN_LENGTH = sizeof(SECURITY_DESCRIPTOR)` is per-arch — 20 on 32-bit (header + four 4-byte ptrs), 40 on 64-bit (header + 4 pad + four 8-byte ptrs; bindgen asserts `_SECURITY_DESCRIPTOR == 40`). Both ntdll-sys and ntkrnl-sys hardcoded 20, halving it on 64-bit so `Dacl = ProcSd + 20` corrupts the Group pointer.
**Fix:** `#[cfg(target_pointer_width="32")] = 20; #[cfg(="64")] = 40;` in BOTH constants.rs. Tell: a known struct-size constant appearing where a per-arch sizeof should be.
**Source:** `feedback_security_descriptor_min_length_per_arch.md`

### amd64 BUGCHECK 0x7F Param1=8 (DOUBLE FAULT) in GUI-thread deep unwind chains
**Symptom:** GUI threads in RtlWalkFrameChain → RtlVirtualUnwind hit a guard page; page-fault delivery itself faults → double fault. On-screen "*** STOP: 0x0000007F" is a real bugcheck, not a cirrus text artifact.
**Root cause:** kernel stack-size constants in `ARM3/helpers.rs` were generic, not per-arch. Wrong amd64 LARGE_COMMIT (16KB vs 48KB) made `MmCreateKernelStack(GuiStack=TRUE)` map only 4 of 18 reserved pages. A shadow `const KERNEL_STACK_SIZE=12288` in `ke/thrdobj.rs` understated amd64 StackLimit by 36KB.
**Fix:** `#[cfg(target_arch=...)]` per-arch from `sdk/include/xdk/<arch>/ke.h`: amd64 48KB/72KB/48KB; i386 12KB/60KB/**16KB** (LARGE_COMMIT intentionally 4KB above C's 12KB — Rust call chains are deeper); arm64 12KB/60KB/12KB. Never introduce a shadow stack const — import from `crate::ARM3::helpers::*`.
**Source:** `feedback_kernel_stack_constants.md`

### win32k files invent thousands of inconsistent signatures/struct layouts (cross-file ABI disagreement)
**Symptom:** `rustconv resolve` reports 138–709 unresolved win32k-internal symbols per file; without a shared FFI surface the LLM invents each `#[repr(C)]` layout per-file → silent cross-file offset corruption at scale.
**Root cause:** ntkrnl-sys bindgen covers `ntoskrnl.h` only (0 win32k symbols); no FFI crate for win32k's internal surface.
**Fix:** build `sdk/lib/rust/win32k-sys` (ntkrnl-sys recipe clone, master header `win32k.h`/`pch.h`). bindgen flags: `-fms-extensions` YES (anonymous typedef members) but `-fms-compatibility` NO (defines `_MSC_VER` → breaks mingw `__CRT_INLINE`); `--allowlist-var '.+'` required (any allowlist drops unreferenced `#define` consts); register de-prefixed aliases for bindgen's tagged-enum member prefixing (`SimpleCallRoutines_HWNDLOCK_...` → also `HWNDLOCK_...`; `--default-enum-style=consts` is a no-op for tagged enums). Self-derive defines/includes/flags per arch from `build-<arch>/build.ninja`.
**Source:** `feedback_win32k_sys_bindgen.md`

### A redundant *-sys crate that re-bindgens types the parent master header already pulls → E0659 mass-ambiguity
**Symptom:** You build a focused `foo-sys` crate (e.g. ft-sys for FreeType) for a lib used by win32k; integrating the flip explodes into hundreds of `E0659` ambiguity errors at every struct-field→foo-call boundary (freetype: 235).
**Root cause:** The parent master header (`win32k.h`/`ntoskrnl.h`) ALREADY `#include`s that lib, so the parent's own bindgen (`win32k-sys`) embeds the lib's types as struct FIELDS (e.g. `SHARED_FACE.Face: FT_Face`, 131 such fields). A separate `foo-sys` defines the SAME types in a different crate → the field type and the `foo`-fn argument type are now two distinct paths for the "same" type → cross-crate ambiguity everywhere.
**Fix:** before building a separate FFI crate, check whether the parent master header already pulls the lib. If it does, FOLD the complete lib surface into the existing `-sys` crate instead: add the lib's headers to `wrapper.h` (under any required build defines like `#define FT2_BUILD_LIBRARY`), extend the allowlist (`FT_[A-Za-z0-9_]*|TT_...`), hand-add the fn-like macros bindgen drops (`FT_HAS_*`/`IS_*` → return `c_int`, callers use `!= 0`), regenerate ×3, then delete the redundant crate.
**Source:** `project_freetype_ftsys_track.md`

### Per-file hand-rolled struct view of a global reads a BITFIELD at offset 0 → feature silently disabled
**Symptom:** A whole feature is silently off with no crash. fastfat: every long-file-name (>8.3) open returned STATUS_OBJECT_NAME_NOT_FOUND though the file was on the volume; 8.3 names worked. On arm64 this made the loader report `kernel32_vista.dll`/`advapi32_vista.dll`/`windowscodecs.dll` "missing" (c0000135), msvcrt's imports went unsnapped, and csrss jumped through an unresolved thunk (arm64 instruction abort at a raw RVA → looks like a reloc bug).
**Root cause:** the file declared its OWN view of a shared global — `extern { static FatData: FatGlobalData; }` with a hand-rolled `struct FatGlobalData { ChicagoMode: u8 }` putting the field at offset 0. The real `FAT_DATA` has `ChicagoMode` as a packed `BOOLEAN:1` bitfield far past offset 0; offset 0 is `NodeTypeCode` (FAT_NTC_DATA_HEADER `0x0500`), whose little-endian low byte is `0x00` → `FatData.ChicagoMode != 0` is always FALSE. Two files (dirsup.rs, volinfo.rs) each had their own wrong stub; a third (strucsup.rs) correctly used the real type's `.ChicagoMode()` accessor — so the bug was invisible by grep-for-consistency.
**Fix:** import the real generated type (`crate::types::FAT_DATA`) and read bitfields through the generated accessor (`FatData.ChicagoMode()`), never as a plain offset-0 field. Delete the hand-rolled stub struct. **Tell:** an `extern static <global>: <LocalStubStruct>` next to a `#[repr(C)] struct <LocalStubStruct> { one_field }` — the LLM fabricates a minimal view of a big global and guesses offset 0. Cross-check every such stub against the real type, especially when the field is a C bitfield.
**Source:** fastfat-port `b2de393afb5` (2026-06-15), `project_fastfat_rust_track.md`

### C `:31` bitfield read as a full DWORD → high flag bit corrupts the value (wild pointer/offset)
**Symptom:** i386-only BUGCHECK 0x1E (KMODE AV, `0xc0000005`) deep in a kernel list-walk (`ExfAcquirePushLockShared`) on a bogus session-space pointer (`0xf6f65700`) — fired as explorer/children read version resources after the version.dll C→Rust flip. amd64/arm64 booted green (luck-masked).
**Root cause:** `IMAGE_RESOURCE_DIRECTORY_ENTRY.OffsetToDirectory` is an `OffsetToDirectory:31` bitfield in C (ntimage.h) whose **top bit is `DataIsDirectory`**. The translation declared it a full `DWORD` and used it raw. Every intermediate resource-directory level has `DataIsDirectory=1`, so the Rust computed `root + (0x80000000 | offset)` → a pointer far outside the `resSection` heap buffer; the garbage offset/length cascaded into a kernel deref. `find_pe_resource()` is the *primary* path for any on-disk PE, so it fired on essentially every PE version query. Bug class: **bitfield-not-masked** (sibling of cast-before-offset / element-vs-byte and the FatData offset-0 bitfield above).
**Fix:** mask before use — `OffsetToDirectory & 0x7FFF_FFFF` (and read the `:31`/USHORT siblings `& 0x7FFF_FFFF` / `& 0xFFFF`). **Tell / grep:** translations that use `.OffsetToDirectory`/`.OffsetToData`/`.NameOffset` (any C `:31`/`:N` bitfield) directly as an offset without masking the high flag bit. When the source field is a C bitfield, never read it as the full storage word.
**Source:** version-rust `79d7680b685` (2026-06-18), dc/version rescue → mainline.

### Hand-rolled accessors over a union's byte view, with two fields TRANSPOSED → a test that fails only 3 boots in 4
**Symptom:** kmtest `ExUuid` asserts RFC4122 compliance — `(Uuid.Data4[0] & 0xC0) == 0x80` — and saw `Data4[0] == 0xd0`. Filed as a "ratchet regression introduced by a merge". **It was neither a regression nor merge-related.**
**Root cause:** C's `UUID_CACHED_VALUES_STRUCT` (`ntoskrnl/include/internal/ex.h`) is a union whose byte view exists *precisely* to alias `GUID.Data4` (the header even says `/* Match GUID.Data4 */`):
```c
union { struct { UCHAR ClockSeqHiAndReserved;  /* == GuidInit[0] == Data4[0] */
                 UCHAR ClockSeqLow;            /* == GuidInit[1] */
                 UCHAR NodeId[6]; };
        UCHAR GuidInit[8]; };
```
The Rust port (`ex-rust/src/uuid.rs`) replaced the union with a bare `GuidInit: [u8; 8]` plus **hand-written accessors — and transposed the two ClockSeq fields** (`ClockSeqLow` → `[0]`, `ClockSeqHiAndReserved` → `[1]`). `ExpUuidGetValues` was otherwise a statement-for-statement faithful port, so it dutifully stamped the RFC4122 variant bits (`|= 0x80`) into the byte that now landed at `Data4[1]`, while `Data4[0]` received the raw low byte of the sequence number.
**Why it hid — the lesson.** The sequence number is seeded from `KeQueryPerformanceCounter` XOR a stack address whenever the registry seed is absent (i.e. every LiveCD boot), so `Data4[0]` was effectively **random**. `(random & 0xC0) == 0x80` is true **exactly 1 time in 4** — so the test *passed 25% of boots*, and the committed baseline had recorded one of those lucky runs as `ExUuid OK pass=3000 fail=0`. A latent bug can hide inside a green baseline as a **probabilistic pass**; the "it regressed at commit X" story was pure coincidence.
**Fix:** swap the two indices (and pin them to the C header in a comment). Post-fix `Data4[0]` is always `0x80 | (seq >> 8 & 0x3F)` → deterministically variant-compliant.
**Tell / grep:** a Rust struct that flattens a C **union** into a byte array plus hand-written named accessors. The union's whole purpose is a specific byte↔field mapping, and nothing in Rust re-checks it — a transposition compiles, links, and passes review. Prefer the bindgen union type, or `offset_of!`; if you must hand-roll, cite the C field order in a comment. Sibling of the `MMPFN`/`KTHREAD.Teb` numeric-offset drift class (`tools/scan_magic_offsets.py`).
**Corollary (test hygiene):** when a conformance test "regresses" but `git diff baseline..HEAD -- <file>` is **empty**, do not hunt the merge — ask whether the assertion is *probabilistic*. Reconciling that contradiction is what cracked this one.
**Source:** conformance-ratchet (2026-07-12).

---

## rustconv translation-output bug catalog

The specific ways the LLM C→Rust output is wrong — each entry is a distinct defect class, how it manifests at compile/link/runtime, and the cheapest oracle to catch it before it ships.

### Dangling pointer to a by-value `transmute` temporary (the WCHAR[] → storageless-temporary trap, 2026-07-12)
**Symptom:** intermittent corruption / garbage strings on a path that reads a stored `*const u16`. Confirmed live: `disk-rust/pnp.rs` — the dangling pointer was the **printf FORMAT STRING** passed to `RtlStringCchPrintfW` when creating a disk's `\Device\Harddisk%d\Partition0` / `\DosDevices\PhysicalDrive%d` symbolic links; `lsasrv-rust/authpackage.rs` — a dangling `Name` field in an `RTL_QUERY_REGISTRY_TABLE` read later by `RtlQueryRegistryValues`.
**Cause:** C's `WCHAR foo[] = L"..."` is a stack array with real storage, so `&foo[0]` stays valid for the array's whole scope. The mechanical translation becomes `core::mem::transmute::<[u8; N], [u16; M]>(bytes).as_ptr()` — but the transmute produces a **by-value temporary** `[u16; M]` with no binding; `.as_ptr()` borrows it; the temporary is dropped at the end of the *statement*; the stored pointer then dangles. Two escaping shapes: `let p = transmute(...).as_ptr()` (let-bound) and `Field: transmute(...).as_ptr()` (struct field that outlives the initializer).
**Why it hid:** the `dangling_pointers_from_temporaries` warning fires precisely on it — but nobody surfaced Rust warnings (105/255 crates carry crate-root `#![allow(...)]`, and the rest's warnings were never read until the 2026-07-12 survey). On amd64/arm64 the freed stack slot often still holds the right bytes (nothing reused it yet), so it "works" until timing changes — classic Heisenbug.
**NOT a bug (don't flag):** the *same transmute as a direct call argument* — `func(transmute(x).as_ptr())` — is sound (the temporary lives to the end of the call statement, valid during the call). And the *reference* form `transmute::<&[u8; N], &[u16; M]>(b"...")` is sound: it yields a reference into the `b"..."` **'static** literal, so `.as_ptr()` is 'static. Only the by-VALUE array escaping form dangles.
**Fix:** bind the transmuted array to a scoped local, then take `.as_ptr()` of the *binding*: `let foo_wide: [u16; M] = transmute::<[u8; N], [u16; M]>(bytes); let p = foo_wide.as_ptr();`.
**Oracle:** rustc's own `dangling_pointers_from_temporaries` lint (surface warnings and it's exact), or the grep-time gate `tools/scan_dangling_transmute.py --lint` (wired into `vibeos-cli check`, empty baseline — the class is fully fixed, any hit is a NEW regression). Suppress a verified FP with `// nolint: dangling-transmute (reason)`.
### C post-inc/dec value expression rewritten with wrong read order (`x = y--` reads post value)
**Symptom:** a count/index handed onward is off by one, and everything downstream of a threshold check silently misbehaves. Flagship instance (conformance catch #2, 2026-07-10): ob-rust `ObpDecrementHandleCount` translated C `ProcessHandleCount = HandleCount--` as decrement-then-read, so the CloseProcedure got the POST-decrement count — 0 instead of 1 on the last handle — and `IopCloseFile`'s `if (HandleCount != 1) return;` bailed, meaning **IRP_MJ_CLEANUP was never delivered to any driver in the system's history** (per-handle driver state leaked; second kmtest process got STATUS_ACCESS_DENIED on a stale MDL registration). Compiles clean, links clean, boots green — nothing in the boot path re-opens a MaintainHandleCount device twice.
**Root cause class:** C's value-bearing `x++`/`x--`/compound assignments inside larger expressions force the translator to split into statements, and it sometimes puts the read on the wrong side of the mutation. Post-forms yield the OLD value; pre-forms the NEW.
**Fix:** read first, then mutate (or vice versa for pre-forms) — match the C exactly; cite the C line in a comment when the ordering is load-bearing.
**Tell / grep:** in any translated function, a `X -= 1; y = X` or `X += 1; y = X` pair whose C original is `y = X--/X++` — cheapest audit is diffing count-bearing assignments against the C when a callee receives a count argument. A future lint could pair `-= 1`/`+= 1` statements adjacent to reads of the same place against the C original.
**Source:** ob-rust `obhandle.rs:505-517` vs `ntoskrnl/ob/obhandle.c:585/604`; found by kmtest sweep (back-to-back device opens) + subagent diff-audit.

### Wrong INLINED flag/constant literal (comment names the right flag, literal is a different one)
**Symptom:** `SetFlag(f, 0x2 /* IRP_CONTEXT_FLAG_TOP_LEVEL */)` (0x2 is MORE_PROCESSING; TOP_LEVEL=0x10); `PALETTE_AllocPalette(0x4 /* PAL_BITFIELDS */ ...)` (PAL_BITFIELDS=0x2, 0x4=PAL_RGB); `RtlRaiseStatus(0xC0000002 /* STATUS_DATATYPE_MISALIGNMENT */)` (0xC0000002=NOT_IMPLEMENTED; MISALIGN=0x80000002). The LLM hardcodes a literal it labels with the intended name but the value is wrong/placeholder. Compiles & links clean; pure runtime/semantic. cdfs alone had 7 (TOP_LEVEL/TOP_LEVEL_CDFS/DISABLE_POPUPS/CDFS_NTC_IRP_CONTEXT/FSP_FLAGS/CCB_FLAG_DISMOUNT/CDFS_RESIDUAL); the codebase sweep found more in scsiport (LUNEX, HIGH), ntoskrnl se/srm (OBJ_OPENIF), win32k dibobj, disk, cabinet, smss.
**Oracle:** `tools/scan_inline_flag_literals.py <crate/src/*.rs> --headers <driver C dir>` — **`verify_constants.py` is BLIND to this** (it only checks `const NAME = VALUE;` defs, not bare literals). The tool is tree-sitter/AST-based (regex can't pair a literal with its naming-comment across nested `|`/parens/casts — not a regular language); catalog = kernel headers + sdk/include/{xdk,ndk,psdk} + driver `--headers`. It's a TRIAGE AID not a verifier — residual FP classes need a human eye: flag-MASK unions (`A|B|C // N1|N2|N3` where the union is right but order differs), informational args (KWAIT_REASON `Wr*`, WPP TRACE_LEVEL), and value-tables labeled by index (COLOR_*, suppressed).

### Bind-all match arms + invented named constants — the 2026-07-05 sweep (killed USB enumeration)

Two adjacent defect classes, found together and best hunted together:

**(1) Bind-all match arm.** A translated `match` arm names a constant that is NOT in scope (`IRP_MN_QUERY_ID => ...` with no such const imported). Rust silently treats it as a *fresh binding that matches everything*; every later arm is dead code, and the linker GCs the dead arms' dependencies (hiding missing symbols). rustc only warns (`unreachable_patterns`/`unused_variables`/`non_snake_case`) and the crates' blanket `#[allow]`s ate the warnings. **Real impact:** usbport's PDO answered *every* PnP minor with the first arm's STATUS_NOT_SUPPORTED -> root hub never enumerated -> no USB at all. fastfat `cleanup.rs` imported every TYPE_OF_OPEN alias *except* `EaFile` -> the EaFile arm swallowed user file/dir cleanup. freeldr `iso.rs`/`fat.rs` treated every seek as absolute (`SEEKMODE_SeekAbsolute` unresolved).
**Oracle (ground truth, use this):** `python3 tools/scan_bind_all_matches.py` — runs `cargo check --release` with `--force-warn unreachable_patterns,unused_variables,non_snake_case` (overriding the blanket allows), parses JSON diagnostics, flags SCREAMING_CASE bindings in match arms. Wired into `vibeos-cli check` (strict empty baseline). The older `tools/verify_match_consts.py` (regex/heuristic) MISSED every real site — "exists somewhere in workspace + file has glob imports" demoted them all to LOW; treat it as a fast pre-filter only, the compiler is the truth.

**(2) Invented named-constant values** (the `const NAME = <guess>;` twin of the inline-literal class above). The translator declares local consts with confidently wrong values — sometimes marked `// typical` (grep for it: 30 hits, ~10 wrong), sometimes not. Found 2026-07-05: cdrom-rust had **28 wrong IOCTLs** (DVD ioctls invented as `0x2D00xx`, real `0x335xxx`; AACS ioctls colliding with real CDROM ioctls so arms went unreachable), usbport listened for `IOCTL_INTERNAL_USB_GET_ROOTHUB_PDO` on **0x220013** (= GET_PORT_STATUS!) while usbhub sent the correct 0x22000F — the request could never match; ke/bug.rs had 14 wrong bugcheck codes (`KERNEL_MODE_EXCEPTION_NOT_HANDLED` = 0x1E, duplicating KMODE -> unreachable OR-pattern; special-pool codes shifted so a special-pool BSOD would be mis-labeled); fastfat `UserFileOpen = 4 // typical` (4 is UserVolumeOpen; real = 2), cdfs `FSRTL_VOLUME_LOCK = 1` (real 3), classpnp `SRB_FUNCTION_STORAGE_REQUEST_BLOCK = 0` (real 0x28).
**Tells:** `// typical` / "actual value may differ" comments; the same IOCTL const defined with *different* values in different files of one crate; UNREACHABLE warnings on arms whose patterns are all real named consts (= two consts share a value). **Oracle:** compute CTL_CODE/NTSTATUS/bugcode ground truth from the C headers and diff every `const NAME: u32 = ...` against it (ad-hoc script per header family; `verify_constants.py` covers the ntoskrnl-side named consts, but driver crates' local consts need the header-diff pass). When an IRP flows down a stack, remember BOTH sides must agree: check the sender's const AND the handler's.

### Same-named constant, DIFFERENT value per driver — cross-driver value bleed (cdfs vs fastfat WAIT flag, 2026-07-06)

**Symptom:** LiveCD boots to `shell ready`, then BUGCHECK **0x1E** a few seconds later (during services/lsass CD I/O), failing amd64 smoke on a *post-marker* crash. Raise is `ExRaiseStatus(STATUS_CANT_WAIT 0xC00000D8)` amplified by a `RtlVirtualUnwind` fault (the translated unwinder derefs a wild slot dispatching the raise → messy 0x1E instead of a clean fatal). Presents as a "WDF crash" (cdrom.sys is KMDF and is on the stack) — it is NOT; the raiser is **cdfs**.
**Cause:** `cdfs-rust` defined `IRP_CONTEXT_FLAG_WAIT = 0x02`. That is **fastfat's** value (`fatstruc.h`); cdfs's own `cdstruc.h` defines it as **0x04**. The two filesystems reuse the same flag *names* with *different bit assignments*. So `CdCreateIrpContext(Wait=TRUE)` set bit 0x02, while `CdAcquireResource` tests 0x04 → the WAIT flag was **never effective** → every cdfs resource acquire ran non-blocking → failed under any contention → `CdRaiseStatus(CANT_WAIT)`, which under DUMMY_PSEH is fatal. (0x02 also collided with `MORE_PROCESSING`=0x02; `DISABLE_POPUPS` 0x20 and `IN_FSP` 0x200 were shifted too — a whole scrambled cluster.)
**Why it hid:** the wrong value is *internally consistent* within the crate (the LLM guessed one value and used it everywhere) AND *plausible* (it's a real IRP_CONTEXT flag value — just fastfat's). Master boots green because it either doesn't reach the contended CD-read window or its other-wrong constants alter the interleaving; a CORRECT storage-const fix elsewhere changed I/O timing and made the contention manifest. Classic latent-bug-unmasked-by-a-correct-fix.
**Tell / oracle:** any constant family shared by sibling drivers (`IRP_CONTEXT_FLAG_*`, `*_NTC_*` node types, driver-private `*_FLAG_*`) — verify each crate against ITS OWN header, never a sibling's. `verify_constants.py` flat catalog DROPS these (same name, ≥2 values = ambiguous), so they need a **per-crate targeted** check: `verify-constants sdk/lib/rust/cdfs-rust/src/*.rs` with only cdfs headers in scope. Localize a live one with the `[EXRAISE]` module-range stack scan (ntkrnl-sys `ExRaiseStatus`, gated on the status, filtered to the loaded-module VA window) → names the exact driver+offset that raised.

### Facility-status return-convention confusion — `return 0` where SUCCESS is a nonzero facility code (hidparser, 2026-07-06)

**Symptom:** a subsystem with its OWN status space (HIDP_STATUS_* = `HIDP_ERROR_CODES(sev,code)`, facility 0x11 — SUCCESS is **0x00110000, not 0**) silently misbehaves: hidclass got "NT success" with `collections=0` from real report descriptors, created no kbd/mouse PDOs, and USB HID input dead-ended after a fully successful enumeration. No warning, no error, all green.
**Cause:** the translation emitted `return 0; // HIDP_STATUS_SUCCESS` (hidparser-rust `context.rs` HidParser_BuildCollectionContext). Callers compare `!= HIDP_STATUS_SUCCESS` (0x110000), so the SUCCESS return read as FAILURE — and the error path then propagated the raw 0 upward, where NT_SUCCESS(0) reads as… success. One wrong literal produced "failed inside, succeeded outside" with zeroed out-params.
**Tell:** `return 0;` (or a literal) with a comment naming a facility-specific constant; mixed status spaces in one call chain (HIDP_STATUS vs NTSTATUS here — the boundary fn `TranslateStatusForUpperLayer` only maps a few codes and passes the rest through). **Oracle:** grep the crate for `return 0` / literal returns in fns whose C original returns a `FACILITY_*`/`*_STATUS_*` constant; also check every status-space boundary fn maps ALL codes (a passthrough `_ =>` arm leaks foreign facility codes into NTSTATUS space).

#### Sub-class: correct literal for the WRONG named constant — a range BOUND, not a flag (scanner-invisible)
**Symptom (2026-06-21, user32-rust):** `IS_ATOM(x)` coded `v > 0 && v < 0xC000` with a comment "MAXINTATOM is 0xC000". The C macro (`user32p.h`) is `(x > 0) && (x < 0x10000)` — the upper bound is **0x10000** (high 48 bits clear), NOT `MAXINTATOM`. MAXINTATOM (0xC000) is the *base* of registered-class atoms; they start at 0xC000 and increment. So once the shell registers ~100 window classes, a real class atom (e.g. `0xC06A`) is `>= 0xC000` and `IS_ATOM` returns false → it flows into `RtlInitUnicodeString`, whose `wcslen` derefs the tiny integer → **AV 0xC0000005 in explorer at "taskbar: creating tray window"**, crash-looping the shell post-`shell ready` (so the smoke gate stays green; only the *desktop* is broken). Two copies (`window.rs:42` `User32CreateWindowEx`, `class.rs:111`) had the bug; `message.rs:224` had it right (0x10000). Fixed in `67c9e5849c7`.
**Why the literal-scanner misses it:** the literal `0xC000` *matches* its naming comment (`MAXINTATOM` really is 0xC000) — so `scan_inline_flag_literals.py` sees a consistent literal/label pair and stays silent. The defect is one level up: the translator picked the **wrong named constant** for an *unnamed* numeric bound in the source (the C `0x10000` had no symbol). This is the residual "correct value, wrong concept" class. **Tell:** a range/threshold compare (`< CONST`, `>= CONST`, `>> N`) where the C original used a bare numeric literal — re-derive the bound from the C, don't trust a plausible-looking named-constant comment. **Cross-arch:** `user32-rust` (and most usermode `*-rust` crates) are cohab'd on all 3 arches, so a usermode-DLL translation bug breaks amd64/i386 identically — it just hides *past* their `shell ready` gate. To see it, boot past the gate and watch for `[UEXC]`/`PspExitThread ExitStatus=0xC0000005`.

### Translator ships the `#if !DBG` STUB branch of a dual-implementation file — the real body silently vanishes (freeldr DbgPrint, 2026-07-06)
**Symptom:** an entire diagnostic/feature channel goes quiet with zero errors: freeldr-rust's `debug.rs` translated C `lib/debug.c`'s `#else` (release, !DBG) branch — `DebugInit`/`DbgPrint`/`DbgPrint2` as literal no-ops — even though every build sets `-DDBG=1`. ALL freeldr serial debug output (TRACE/ERR/WARN and plain DbgPrint from uefildr, rosload, and every freeldr-rust module that declared `fn DbgPrint` expecting it to work) has been dead since the translation landed. Consequence for debugging: freeldr's whole load phase (~22s of every x86 boot) was one silent serial gap — the observability that would have flagged it was itself the casualty.
**Cause:** the C file has TWO complete implementations under `#if DBG` / `#else`; the LLM translated the (much shorter) stub half. Compiles, links, boots — nothing checks that a debug channel emits.
**Tell:** any translated file whose C original contains a `#if DBG`/`#if defined(NDEBUG)`-style dual implementation — diff the Rust against the *DBG* branch, not just "a" branch. Grep the crate for bodies that are pure `UNREFERENCED_PARAMETER`/`0`/`{}` where the C DBG branch is substantial.
**Fix:** real port restored in freeldr-rust `debug.rs` — DebugInit parses DEBUGPORT/BAUDRATE + lazy pre-init on first print; DbgPrint prints the string RAW (freeldr's cargo path pins STABLE, no `c_variadic` — constant messages intact, %-formats literal; full formatting owed on a nightly move); `[FLDR]` boot-phase stamps ride on it.

### C `switch` case labels → Rust or-pattern that silently swallows a neighboring case (LCMapString OS-wide failure, 2026-07-06)
**Symptom:** a translated function fails with an "invalid parameter/flags" error for a plainly valid call, on every arch, forever — no crash, no link error. Found because svchost logged `SvchostCharLowerW failed` at every service start: **every `LCMapStringW(LCMAP_LOWERCASE/UPPERCASE)` call in the OS had returned ERROR_INVALID_FLAGS since the 2026-06-14 kernel32 locale port**.
**Cause:** C fallthrough case labels `case NORM_IGNORENONSPACE: case NORM_IGNORESYMBOLS: case NORM_IGNORENONSPACE|NORM_IGNORESYMBOLS:` (the last one a BITWISE value, 6) were rendered as the Rust or-pattern `NORM_IGNORENONSPACE | NORM_IGNORESYMBOLS | 0 =>`. In a Rust pattern, `|` is ALTERNATION, not bitwise-or — this arm matches 2, 4, **and 0**, so it captured `switch_flags == 0` (the plain LOWERCASE/UPPERCASE path), whose flag-validation then rejected the call; the real `0 =>` arm below became dead code (kernel32-rust locale.rs `lcmap_string`; fixed on `performance-investigation`).
**Tells:** (a) any Rust match arm `CONST_A | CONST_B =>` where the C original had `case CONST_A|CONST_B:` as a single bitwise label — the translation changed its meaning from "value 6" to "2 or 4"; (b) a literal `0` alternative inside an or-pattern arm ABOVE a dedicated `0 =>` arm (rustc's unreachable-pattern lint flags the dead arm — don't let translated crates build with that warning suppressed); (c) `grep -rn ' | 0 =>' sdk/lib/rust` — two other hits (msgina `1 | 0`, win32k `MA_ACTIVATE | 0`) audited correct because their C really had `case X: case 0:` fallthrough.
**Oracle:** for every translated `match` on a flags/mask variable, diff the arm set against the C switch: each C `case A|B:` must become `_ if x == (A | B) =>` (or a named const), never a bare or-pattern.
### Match arm on an UNDEFINED constant — silently becomes an irrefutable catch-all binding, killing every later arm (win32k WM_CTLCOLOR, 2026-07-05)
**Symptom:** the entire desktop rendered with **no non-client chrome** (no captions/borders/buttons on any top-level window) and **no taskbar children** (no Start button, no clock), deterministic on all 3 arches, no crash, boot marker green. Root: `IntDefWindowProc`'s `match Msg` in `win32k-rust/src/defwnd.rs` had a `WM_CTLCOLOR => …` arm, but `WM_CTLCOLOR` (a 16-bit-compat windowsx.h macro, 0x19) does **not exist** in the win32k-sys bindgen constants — so Rust parsed it as a **variable-binding pattern that matches every value**. Every message after that arm in the match — WM_SETCURSOR, WM_ERASEBKGND, WM_NCHITTEST, WM_PAINT, WM_SYNCPAINT, WM_WINDOWPOSCHANGING/CHANGED, **WM_NCCALCSIZE, WM_NCACTIVATE, WM_NCPAINT** — was swallowed and returned a garbage brush handle. WM_NCCALCSIZE never inset `rcClient`, so `rcClient == rcWindow` for every window, which (correctly) no-ops all NC painting. Same class found same-day in `ntoskrnl-rust`: `pnpmgr/devaction.rs` (`STATUS_FAILED_DRIVER_ENTRY` arm — not in ntkrnl-sys → catch-all, misclassifying every driver-load failure) and `ke/balmgr.rs` (`STATUS_WAIT_0` arm — not imported → catch-all; benign only because all arms were empty). Related value-typo cousin: `ke/bug.rs` `KERNEL_MODE_EXCEPTION_NOT_HANDLED` defined as 0x1E (dup of KMODE) instead of 0x8E.
**Why it ships:** compiles and links clean — rustc only emits **warnings** (`unreachable pattern`, "this pattern matches any value", and `variable X should have a snake case name`), which scroll by unread in a 40-crate build.
**Oracle (cheap, run it):** grep the cargo output of a **clean** build for the fingerprints: `grep -E "matches any value|unreachable pattern|should have a snake case name" full.log`. "matches any value" on a SCREAMING_CASE identifier is ALWAYS this bug. Warm builds only re-emit warnings for recompiled crates — only a `--clean` build sweeps the whole workspace.
**Fix pattern:** define the missing const locally (with the real header value, e.g. `const WM_CTLCOLOR: UINT = 0x0019;`) or fully qualify the path (`ntkrnl_sys::mm::STATUS_WAIT_0 =>`). When translating any C `switch`, every `case` label MUST resolve to a const in scope — if bindgen lacks it, the translator must emit the const, not trust the bare name.
**Debug methodology that found it:** serial-marker bisection of the message round-trip (kernel send → user32 callback → app wndproc → DefWindowProc → `NtUserMessageCall` → kernel handler) with one `DbgPrint` per hop — the cut was "dispatch arm entered, handler never called", i.e. *inside one function*, which pointed straight at the match. User-mode `DbgPrint` from user32-rust lands on the same serial as kernel prints — instrument both sides.

### C param reassignment translated as a block-local `let` SHADOW — the new value silently dies at the block end (kernel32 CreateFileMappingW, 2026-07-06)
**Symptom:** a C function that *reassigns a parameter* inside a conditional (`if (hFile == INVALID_HANDLE_VALUE) hFile = NULL;`) behaves as if the branch never ran — downstream code sees the ORIGINAL value. Flagship: `kernel32-rust/src/file/filemap.rs` `CreateFileMappingW` — the translation emitted `let hFile: HANDLE = null_mut();` inside the if-block, so `NtCreateSection` received `INVALID_HANDLE_VALUE` (-1 = current-process pseudo-handle) as a *file* handle → **every anonymous/named pagefile-backed `CreateFileMapping(INVALID_HANDLE_VALUE, …)` in the OS failed** (kbswitch indicdll DllMain → the visible "SetHooks failed." messagebox; `SHAllocShared` → shell32 changenotify "Out of memory"/"Delivery failed" on every boot). Full-workspace sweep found 4 more: `win32k-rust/bitblt.rs` `EngBitBlt` ×2 cfg variants (`psoMask = BRUSHOBJ_psoMask(pbo)` + the no-mask `rop4` degrade both discarded → brush-mask blits ran with NULL mask), `ntoskrnl-rust/ARM3/section.rs` `MiFlushTbAndCapture` ×2 sites (`protection_mask` cache-attribute adjust discarded → the RebuildPte branch produced a byte-identical PTE, whole reconciliation a no-op), `freeldr-rust/tuimenu.rs` `TuiDisplayMenu` (`MenuTimeOut = -1` discarded).
**Why it ships:** compiles clean; the shadow is only a `unused_variables` warning — and the translation crates **blanket-`#![allow(unused_variables, unused_assignments)]`** in lib.rs, so not even a warning is emitted.
**Oracle (the sweep, reusable):** force the lint back on over the crate attrs and filter for shadows of *function params*: `RUSTFLAGS='--force-warn unused_variables' cargo check --release --keep-going --workspace --target <T> --target-dir <scratch> --message-format=short` (run under `tools/canon-run.sh` so the `*-stub`/link build.rs find `/tmp/vibeos`; exclude the `*-cargo` link crates + `cnt-difftest`), then keep only warnings whose line is a `let <name>` at indent ≥8 where `<name>` is also a param of the enclosing fn (the filter script lives in the 2026-07-06 session scratchpad; ~6,000 raw warnings → 5 real bugs, zero FPs). Run per-target — x86-only/arm64-only cfg variants each need their own pass.
**Fix pattern:** make the param binding `mut` in the signature (ABI-neutral) and turn the `let` back into an assignment. Never "fix" by underscoring the shadow — the C semantics *require* the reassignment.

### `write_bytes(ptr, v, size_of::<T>())` — count is in ELEMENTS, zeroes sizeof(T)² bytes (kernel32 QueueUserAPC, 2026-07-06)
**Symptom:** a process dies with `0xc0000005 at rip=0` and a **virgin-zero stack** at/above the fault rsp (no return address — nothing to symbolicate), OR unexplained pool corruption near a freshly-zeroed allocation. Flagship: explorer's change-notify thread died at the first `ret` after `NtQueueApcThread` — `QueueUserAPC` zeroed 256 bytes over its 16-byte `ACTIVATION_CONTEXT_BASIC_INFORMATION` local, wiping its saved regs, return address, and ~0xc8 of caller stack (`SHChangeNotifyRegister` → `CDirectoryWatcher::CreateAPCThread` → QueueUserAPC → exit 0x80 → no desktop icons on x86; arm64 skips SHChangeNotifyRegister, hence unaffected).
**Cause:** C `memset(&x, 0, sizeof(x))` / `RtlZeroMemory` translated as `core::ptr::write_bytes(&mut x as *mut T, 0, size_of::<T>())` — but `write_bytes` count is in UNITS OF THE POINTEE, so this writes `sizeof(T)²` bytes. 28 live sites found in one sweep (kernel32, classpnp SRB/fdo-data, disk geometry, fastfat IRP_CONTEXT/EA_RANGE, ks, mscms, mountmgr, pcix) — the kernel-driver ones are silent **pool overruns** (SCSI_REQUEST_BLOCK: 88² = 7.7KB of zeros over nonpaged pool).
**Debug methodology that found it:** kernel-side first-chance instrumentation in `KiDispatchException` (image/code/rip), then on the NULL-rip fault: dump the user stack from rsp up to the first non-zero qword (measures the zero swath) + PEB module list; `rcx` on amd64 holds the last `syscall` stub's return RIP (CPU clobber) — it named `ZwQueueApcThread` and the swath size matched the function's over-zeroing exactly. Confirmed by disassembling the built DLL: `sub rsp,0x58` prologue vs `movaps` stores out to `[rsp+0x130]`.
**Oracle (gated):** `scan_silent_failures.py` shape `WRITE_BYTES_SIZEOF` — flags 3-arg `write_bytes` with `size_of` in the count unless the pointee is size-1 (`*mut u8/i8/CHAR`, `c_void`/`PVOID` — c_void is size 1 in Rust) or the count does element math (`/`). Wired into `vibeos-cli check` (0-NEW baseline).
**Fix pattern:** byte-cast the pointer — `write_bytes(ptr as *mut u8, 0, size_of::<T>())` — never "fix" by dividing the count.

### Kernel Rust emitting x87 (`fild/fistp`) for 64-bit atomics — imprecise 0xc0000092 BSOD blamed on an innocent function (i386 STOP 0x1E, 2026-07-06)
**Symptom:** flaky i386 `STOP 0x1E (0xc0000092 …)` — STATUS_FLOAT_STACK_CHECK — with P2 pointing at a function with no floats in its source (flagship: `MiLockProcessWorkingSetUnsafe+0x87`). x87 faults are IMPRECISE: they fire at the *next* FP instruction executed, and the kernel doesn't own the FPU — the fault needs a user thread's dirty x87 state (full stack / pending unmasked exception) live at kernel entry, hence the flakiness.
**Cause:** the i686 kernel target JSONs set `"rustc-abi": "x86-softfloat"` (ABI only) with `cpu=pentium` (no SSE) but never disabled x87 CODEGEN — LLVM lowers `AtomicU64` load/store via `fild/fistp` when x87 is available and SSE2 isn't. 2479 x87 instructions in ntoskrnl, each a probabilistic landmine.
**Fix:** `"features": "-mmx,-sse,+soft-float"` in every kernel-mode `i686-kernel.json` (atomics lower to cmpxchg8b; f32/f64 become soft libcalls). ntoskrnl now has ZERO `fild/fistp`; residue is kdbg/CRT float paths + deliberate FPU probes (`KiIsNpxErrataPresent`, `KiVerifyCpuFeatures`). **win32k stays hard-float on purpose**: its FP is genuine geometry math under `EngSaveFloatingPointState`/`KeSaveFloatingPointState` guards (verified preserved in translation, zero fild/fistp), and `+soft-float` would break the f32-returns-in-ST0 ABI at its C libm calls (`atan2f` etc.).
**Toolchain trap (cost a full rebuild cycle):** cargo does NOT fingerprint custom target-JSON contents and sccache does NOT hash them either — editing a `*-kernel.json` silently keeps stale objects. A spec change needs a source edit in the crate or `rm -rf sdk/lib/rust/target/<stem>` to actually take. Verify by counting x87 mnemonics in the linked binary, not by trusting the build.
**Side effect to expect:** build-std pulls `compiler_builtins int/big` (u256/i256 for f128 soft ops) with 6 ud2s — panic tails (`shl/shr` ≥256 guards) and `unimplemented!()` `widen_hi/widen_mul` bodies; allowlisted in `scan_ud2.py` after disassembly verification.

### ANSI byte string cast to a WIDE pointer — every lookup silently falls back to defaults (workspace-wide sweep, 2026-07-06)
**Symptom:** no crash, no error — registry lookups/opens just fail and the code takes its default/fallback path. Found via the 31s-per-service SCM stall (ScConnectControlPipe's `b"\0"` as an "empty PWSTR" — a wide empty string needs TWO zero bytes) and then swept: ~140 sites in 121 files, all fixed 2026-07-06.
**Cause (4 sub-shapes, all from translating C `L"..."` literals):** (1) plain ANSI cast wide: `b"ScrollWidth\0".as_ptr() as PCWSTR` — a u16 read packs byte PAIRS, so the name never matches (win32k sysparams: all 27 window metrics silently ignored the registry). (2) hand-interleaved UTF-16LE (`b"F\0o\0o\0\0\0"`): content-correct but **align-1** — a `*const u16` into a byte literal is UB and has faulted before (basesrv CSDVersion). (3) phase-broken interleaving: one mis-escaped run (`\\0C` = '\', '0', 'C') desyncs the byte phase and the tail decodes as CJK garbage (msgina's `SYSTEM\CurrentControlSet\Control` — RegOpenKey always failed; cmboot's `System32\Drivers\` broken past "System3"; partmgr copied packed-ANSI `VOLMGR` as u16s). (4) unterminated: odd byte counts / missing final NUL (`b".\0l\0n\0k\0\0"`; SAM's `[u16; 5]` "%08lx" hand-fix with no terminator).
**Oracle:** `grep -rnP 'b"(?:[^"\\\\]|\\\\.)*"\.as_ptr\(\) as (PWSTR|PCWSTR|LPWSTR|LPCWSTR|\*const u16|\*mut u16)' sdk/lib/rust` plus the `as *const u8 as *const u16` variant. ZERO non-comment hits is the healthy state post-sweep. (`transmute::<[u8;N],[u16;N/2]>(*b"...")` sites are aligned and fine.)
**Fix pattern:** the per-file `w!` macro (const-eval NUL-terminated `&'static [u16]`, 2-aligned): `w!("ScrollWidth").as_ptr()`. For new translations the rule is absolute: a C `L"..."` literal NEVER becomes a `b"..."` byte string.

### Guard condition swapped for a DIFFERENT nearby variable — logic silently inverted per boot mode (CmpInitializeHiveList MININT, 2026-07-06)
**Symptom:** 5–9s boot stalls ×4 on livecd with `Failed to write the primary hive` on serial — the registry tried real writes to CD-backed hives on a MININT boot. No crash, boot marker green; pure latency + scary serial noise. First misclassified as "upstream-inherited" because the C `CmpNoWrite = FALSE` clobber in the same function *is* upstream — the actual bug was ours.
**Cause:** C `if (CmpShareSystemHives) CmpMachineHiveList[i].HHiveFlags |= HIVE_VOLATILE;` was translated as `if CmHive2 != null { HHiveFlags |= 1 }` — a *different variable* that happens to appear near the loop in the C source. `CmHive2` is only assigned later BY the load threads this loop spawns, so the condition is always false: MININT hives never got HIVE_VOLATILE, and `CmpLoadHiveThread` + every downstream write path (which all gate on HIVE_VOLATILE, not CmpNoWrite) attempted doomed CD writes (cmsysini.rs `CmpInitializeHiveList`; fixed on `performance-investigation`).
**Tells:** (a) a translated `if` whose tested variable does not appear in the corresponding C condition at all — worse than an inverted test because nothing looks wrong locally; (b) behavior that differs by BOOT MODE (MININT/setup/normal) when the C gates that mode via a flag the Rust never reads — `grep` the .rs for each mode flag the C file tests (`CmpShareSystemHives`, `CmpMiniNTBoot`, `ExpInTextModeSetup`, `InitIsWinPEMode`…) and diff the count against the C file; (c) a "should never fire" serial message (a doomed-write error, a hard-error popup) that fires reliably in one boot flavor only.
**Oracle:** for every translated condition, check the C original tests the SAME symbol — mechanical to spot-check per file: list `if (`-condition variables in the C, list them in the Rust, diff. A missing mode-flag read is this bug.

### Indexing a bindgen flexible-array extern (`static NAME: [T; 0]`) panics unconditionally (win32k NtGdiMaskBlt, 2026-07-05)
**Symptom:** deterministic **kernel RUST_PANIC** post-boot-marker (smoke `RUST_PANIC` classification), e.g. `win32k-rust/src/ntbitblt.rs:968` in `NtGdiMaskBlt`, the instant that code path first runs (here: notepad's window paint issuing a masked blit). The panic is a bounds-check: `panic_bounds_check(index, 0)`.
**Cause:** C `extern UCHAR gajBitsPerFormat[];` (unknown-size array) is emitted by bindgen as `pub static mut gajBitsPerFormat: [UCHAR; 0usize];`. A Rust file that pulls it in via a glob `use xxx_sys::...::*;` and does `gajBitsPerFormat[fmt]` or `.len()` bounds-checks against length **0** → every access panics. The real array is defined (sized) in some *other* Rust module (`surface.rs`: `[u8; 11]`), so files that `use crate::surface::gajBitsPerFormat;` or use `addr_of!`-pointer arithmetic are fine — only the glob-resolved sites are landmines. Same-day sweep of all zero-len bindgen statics (`gajBitsPerFormat`, `DibFunctionsForBitmapFormat`, `StockObjects`, `appalSurfaceDefault`, `gasyscur/gasysico`, `gpDxFuncs`, `KiBugCheckData`, …) found one more live site (`stretchblt.rs` `DibFunctionsForBitmapFormat`); most sites already shadow the extern with a local sized `extern "C"` decl or explicit import.
**Oracle:** `grep -hE "pub static (mut )?[A-Za-z_0-9]+: \[[^;]*; 0usize\]" sdk/lib/rust/*-sys/src/*.rs` to list the zero-len externs, then for each, check every `NAME[` / `NAME.len()` site: does the file import a real sized definition (or use `addr_of!`), or does it glob-resolve to the `[T;0]` extern? A `panic_bounds_check(_, 0)` in a named kernel fn is this bug.
**Fix:** `use crate::<realmodule>::NAME;` (the sized definition) at the top of the offending file — a one-liner, minimal blast radius. If no sized Rust definition exists anywhere (symbol is C/asm-only), convert the site to `addr_of!(NAME).cast::<T>().add(i)` pointer access. Also bounds-check explicitly if the index comes from untrusted data (C indexes unchecked — UB there but in practice rejects; Rust must not panic the kernel).

### `addr_of!(field)` where C passes the pointer-field's VALUE — buffer becomes struct innards (win32k clip-region 0x50, 2026-07-05)
**Symptom:** master-wide intermittent **BUGCHECK 0x50** post-boot-marker on ~20-25% of amd64 boots, `P1` a paged-pool VA constant across boots, `P4=0` (invalid PTE, `soft_protection == MM_ZERO_ACCESS` — i.e. a never-committed pool page), during explorer taskbar paint. `TrapFrame.Rip` = win32k's own `memcpy`; caller chain `NtGdiGetAppClipBox → GdiGetClipBox → CLIPPING_UpdateGCRegion → IntEngUpdateClipRegion`. On the ~75% of boots where the neighboring pool page happened to be committed: **no crash, silently garbage clip rects** (struct innards copied as RECTLs).
**Cause:** C `IntEngUpdateClipRegion(&pDC->co, nCount, pDC->prgnRao->Buffer, …)` passes the pointer-typed field's VALUE (PRECTL rect array). The translation passed `core::ptr::addr_of_mut!((*prgnRao).Buffer)` — the address OF the pointer field — so the rect copy sourced from inside the REGION struct and overran its paged-pool allocation by `nCount*16 - remaining` bytes (`cliprgn.rs` CLIPPING_UpdateGCRegion; fixed on master 2026-07-05). Same shape found in cabinet `fci.rs`: `&fci->pv as *mut i32` passed as a callback's `int* err` out-param (Wine C passes a local `&err`) — error writes corrupted the context pointer.
**Tell / oracle:** any `addr_of!/addr_of_mut!` of a field that is ITSELF `*mut T`/`*const T`, passed as a function argument the callee treats as a DATA buffer (memcpy source/dest, array base, string). Legit uses of the same shape: identity compares (`Rects != addr_of_mut!(rclBounds)`), pointer-slot writes (Interlocked/`write_volatile`/free-and-null), `T**` out-params, and addr_of of EMBEDDED structs/flexible arrays (`Data[1]`, `bString[1]`, `rdh.rcBound`) which are correct. A 2026-07-05 sweep of all translated crates found exactly the two instances above — but new translations can reintroduce it; check every addr_of-as-argument against the C original.
**Debug infra that cracked it (permanent):** `KeBugCheckEx` now self-symbolicates its params against `PsLoadedModuleList`, extracts `TrapFrame.Rip` for Mm bugchecks (P3 is TrapInformation, NOT a PC — decode P4: this port raises 0x50 as `(Address, FaultCode, TrapInformation, reason)` with reason 0=ZERO_ACCESS PTE, 1=NOACCESS, 2=invalid PXE/PPE/PDE), and prints a scan-based mini-backtrace — all on serial, no GDB (module bases vary per boot, so cross-boot symbolication of a saved PC is unreliable; catch it self-reported instead).

### Ported tree/list rebalance: an untested special case (predecessor = direct child) self-corrupts links — random model tests catch it, hand-built cases don't (splaytree, 2026-06-26)
**Symptom:** splaytree's two-children `delete` (`RtlDelete` → `subtree_predecessor` + `swap_splay_links`) corrupts the tree ONLY when the in-order predecessor is the node's *direct left child* — `verify_links` fails "left child's parent mismatch", or a later op derefs a wild/misaligned node pointer (a `SplayLinks` field holding key-like bytes). Compiles/links clean. The splay primitives are extern-linked from the splaytree crate, so any generic-table consumer (`generictable`, anything calling `RtlDelete`) hits it on a two-children-direct-predecessor delete. Latent — boot never landed on that exact shape.
**Cause:** in `swap_splay_links`, the "A was parent of B" special case wires A in under B (and repoints A's old parent at B), but the following `fixup_child_links(A, root=false, …)` *also* relinks A into "its parent" — and A's recorded parent (`tb.parent`) is A's *own* address (B was A's child), so it runs `set_left/right(A, A)`, self-corrupting A's child pointer and orphaning A's real child.
**Fix:** pass `a_was_parent_of_b = (tb.parent == link_a)` as fixup's `root` flag for A, skipping the bogus parent-relink (A's parent is already B). Only the special case changes; the deeper-predecessor path is byte-identical. (`e5ca403e206`)
**Methodology (the real lesson):** hand-built delete tests covered leaf / root-only / one-child and even a two-children case with a *deeper* predecessor — all green — and still missed this. A **random keyed model test** (insert/delete vs a `BTreeMap`; nodes never freed/reused so any corruption is a genuine link bug; `verify_links` + in-order-equals-sorted check after EVERY op) found it in <100 ops and shrank to a 4-node deterministic repro. For any ported/translated tree/list/heap: don't trust hand-built cases — model-test against a reference structure with a links-invariant check after each mutation, and make sure the merge/rebalance special cases (predecessor-is-direct-child, node-is-root, left-vs-right child) are actually exercised. The same gap let `swap_splay_links` ship untested for the entire two-children path.

### Hand-encoded UTF-16 (wide) byte-string literals — three silent mis-encodings
**Symptom (2026-06-21, user32-rust Wave 4):** rustconv emits `L"..."` wide strings as Rust byte-string literals (`b"i\0m\0m\03\02\0.\0d\0l\0l\0\0\0"` == `L"imm32.dll"`). Three ways it silently corrupts them, all compile clean (the bytes are well-formed Rust), all manifest only as a wrong *runtime* string:
- **(a) C-octal confusion.** Rust has **no octal escapes**: `\0` is NUL and any following digits are LITERAL. The translator writes a C octal (`\000`, `\0NN`) → `b"...m\0003\0002..."` is `m \0 '0' '0' '3' ...` = garbage, not `...m32...`. (A single `\0`+one-digit, e.g. `\03`, is *fine* — that's the NUL high-byte of one char then the next char's low byte; only `\0`+**two-or-more** digits is the bug.)
- **(b) Missing u16 NUL terminator.** A wide literal must be even-length and end in bytes `00 00`. `b"\\\0"` (intended `L"\\"`) is `[5C 00]` — the `'\'` char with **no terminator**; read as a `PCWSTR` it walks off the literal into adjacent `.rodata`. `b"\0"` (intended empty `L""`) is one byte `[00]` — odd, half a code unit. Correct: `b"\\\0\0\0"` and `b"\0\0"`.
- **(c) ANSI literal cast straight to a wide pointer.** `CreateWindowW(b"TuiConsoleWindowClass\0".as_ptr() as *const u16)` / `RegisterWindowMessageW(b"HardError\0".as_ptr() as *const u16)` — an ASCII byte string (no `\0` interleaving) cast to `*const u16` reads as UTF-16 garbage (`"Tui"` → `[0x7554, 0x6975, ...]`). The string should have been wide-encoded.
**Real boot impact:** (a)+(b) together broke `User32GetSystemFilePath` in `imm.rs`/`dllmain.rs` — the `b"\\\0"` path-separator appended garbage after `%SystemRoot%\system32` → `LoadLibraryW("imm32.dll")` failed → `User32InitializeImmEntryTable` returned FALSE → imm32 `DllMain` failed → `[LDR] FAILED` → **BUGCHECK 0x4C (0xc0000142)** on the first GUI process, on **all 3 arches** (the encoding is arch-independent). Cohab'd usermode DLLs hit it identically. Found via `[U32IMM]` `DbgPrint` probes (`ghImm32=NULL`, `LoadLibrary` returned NULL); fixed by `b"\\\0"`→`b"\\\0\0\0"`.
**Oracle:** `tools/lint_wide_strings.py [path…]` (`--check` for CI exit-1). Decodes every `b"..."`, flags: source `\0`+2-digits (octal); and — for literals **directly** cast `.as_ptr() as {PCWSTR,LPCWSTR,PWSTR,*const u16,…}` — odd length or missing `00 00` terminator. Binds the cast to *this* literal and skips `*`-deref'd tags (`*b"IpcA"`) so pool-tags/FourCCs/ANSI-used-as-ANSI don't false-positive. A 2026-06-21 project-wide run found **67 hits in 22 crates** (win32k, winsrv, smss-csrss, ntoskrnl, kernel32, classpnp, …) — mostly class (c) in non-boot-exercised paths, so they boot but are real defects. **`verify_constants.py`/`scan_inline_flag_literals.py` are blind to this** (it's string DATA, not a named const). Candidate rustconv `postprocess` transform: emit wide literals via a `wstr!()` helper instead of hand-encoding.

### C fixed-array field passed by-value to a variadic (`%s`) — missing decay

**Symptom:** compiles + links clean, then bugchecks at runtime in `strnlen`/`wcslen` (a `repne scasb/scasw` running off a buffer; faulting address just above `$esp`). Backtrace `strnlen` ← `swprintf`/`wstreamout` ← your fn. (In ACPI `buspdo.rs`: a 0x1E the instant PnP queried a device InstanceID.)

**Root cause:** a C **fixed array** field/local passed to a **variadic** `%s`/`%hs` decays to a pointer in C (`swprintf(t, L"%hs", dev->unique_id)` where `unique_id` is `char[9]`). c2rust/LLM emits `swprintf(t, w!("%hs"), (*dev).unique_id)` — but a Rust `[c_char; N]` does **not** decay; it's pushed **by value** into the varargs. The callee's `%hs` then reads the wrong 4 bytes as `char*` (usually an adjacent stack pointer) and scans into unmapped memory.

**Fix:** `.as_ptr()` (or `.as_mut_ptr()`) on the array arg to restore the decay: `swprintf(t, w!("%hs"), (*dev).unique_id.as_ptr())`.

**Why easy to miss:** only **variadic** calls hide it — non-variadic fns (`strcmp`/`strstr`) take `*const c_char`, so a bare array there is a compile error (caught); and the dest/first arg is already a pointer. **Grep the trailing/variadic args** of `swprintf|_snprintf|sprintf|swprintf!` for struct/local fields typed `[c_char; N]`/`[u16; N]`; check `types_*.rs` to distinguish array fields from pointer fields (pointer fields are fine as-is). Sibling compile-but-run-wrong c2rust footguns: the wide-literal mis-encodings above and the undefined-const→catch-all match binding.

### C `if (!Fn())` → Rust `if !Fn() != 0` — bitwise-NOT on an integer BOOLEAN makes the condition ALWAYS TRUE
**Symptom:** a guard that should fire "only when the predicate is false" fires unconditionally. Real case (fastfat write.rs:382): `if !FatIsIoRangeValid(Vcb, byte, count) != 0 { return STATUS_DISK_FULL; }` → **every** non-paging, non-write-to-EOF user-file write returned `STATUS_DISK_FULL`, which killed the registry hive flush (`cmlib hivewrt.c:360/649` "Failed to write the base block header / alternate hive"). Boots otherwise (the path is only one of several), so it reads as a narrow "some writes fail" bug.
**Root cause:** the callee returns a C `BOOLEAN`/`BOOL` (Rust `u8`/`i32`/`KPROCESSOR_MODE`, NOT `bool`). C's `!x` is logical; Rust's `!x` on an integer is **bitwise NOT**. So `!Fn()` is `0xFE`(from 1) or `0xFF`(from 0), and the appended `!= 0` (the LLM's truthiness cast) is then *always* satisfied: `(!Fn()) != 0` ≡ `true` for any value except all-ones. Net: the negation AND the truthiness-cast both got applied and the predicate is dead.
**Fix:** `!Fn(...) != 0` → `Fn(...) == 0` (C `!x` on a BOOLEAN is exactly `x == 0`). fastfat had 10 sites (write.rs FatIsIoRangeValid + FatLookupMcbEntry×3; read.rs FlagOn×5 DASD/MDL gates; dirsup.rs FatIsFat32) — commit `5525f825288`.
**Oracle/sweep:** `grep -rnE '!\s*[A-Za-z_][A-Za-z0-9_:]*\s*\([^;{]*\)\s*!=\s*0' --include=*.rs`. **Filter false positives** — fns that truly return Rust `bool` (`NT_SUCCESS`, `MI_IS_*`, `.is_null()`/`.is_some()`) make `!Fn()` a *correct* logical NOT and never have a trailing `!= 0` on the same call. The tell is specifically `!<call> != 0` (or `!<call> == 0`) where the call returns an integer BOOLEAN. Codebase-wide this also hit ntoskrnl `ke/i386/traphdlr.rs` (KiUserTrap), win32k (IsCJKT/co_IntMouseActivateWindow/IntCheckFontPathNames), cdfs (SectorOffset in an NT_ASSERT) — a recurring per-lane defect, worth a sweep on any LLM-translated crate.

### Translation-MODEL fidelity gap: DeepSeek output is markedly buggier on constants than Claude
**Observation (2026-06-14):** the inlined-flag-literal sweep + the cdfs debug grind correlate hard with the translating model. **Claude-translated** crates (ntoskrnl 405 files, win32k 166, ntdll, most of kernel32, rtl×20, the FS libs) came back essentially CLEAN. **DeepSeek-translated** crates (cdfs, scsiport, disk, kernel32 lcformat/nls) carried the wrong-literal + wrong-struct-flag + dropped-control-flow defects (cdfs needed ~18 fixes to boot; scsiport LUNEX HIGH; disk SMART/pool). DeepSeek is far cheaper but trades real fidelity on numeric constants, flag bits, and SEH/control-flow faithfulness.
**How to apply:** for constant/flag-heavy or boot-critical drivers, prefer Claude; if DeepSeek is used for cost, run `scan_inline_flag_literals.py` + `verify_constants.py` + `verify_ffi.py` HARD on the output and budget extra integration-debug time. Don't trust a DeepSeek cohab "compiles + links" as "correct".

### Functions silently missing from the flip (scanner skips macro-wrapped / SAL / callback / preproc defs)
**Symptom:** `translate` reports far fewer functions than the C has; flip-link fails `undefined symbol`, or the API dispatch table calls a function that was never emitted. concfg/font.c found 2 of 14; basesrv vdm.c found the helpers but NONE of the 12 `BaseSrv*` CSR handlers.
**Root cause:** scanner.py matches only `node.type == 'function_definition'`. `CSR_API(Name){...}` parses as expression_statement+block; SAL-with-args (`_Inout_updates_z_(N)`), calling-conv macros (`CALLBACK`/`APIENTRY`/`PASCAL`), and body-internal `#if 0`/`#ifndef` all make tree-sitter-c emit an ERROR node that cascade-swallows every following function.
**Fix / oracle:** pre-parse splice passes in `scan_file` — `_expand_csr_api` (CSR_API→real NTAPI sig), `_strip_sal`, `_normalize_callbacks` (→`__stdcall`), `_resolve_known_preproc`. Always diff the .rs's translated-fn list against the C's actual function/handler count; never trust "OK". `\bCSR_API\s*\(` must not match `CSR_API_NUMBER_TO_SERVER_ID`.
**Source:** `feedback_rustconv_csr_api_macro_skip.md`, `feedback_rustconv_scanner_sal_callback_preproc.md`

#### Variant: pointer-return-type functions silently dropped (dxeng.c, 2026-06-13)
**Symptom:** `translate` reports N functions but the C has N+k; the missing k all return **pointers**. dxeng.c: scanner found 40 of 42, dropping `DxEngEnumerateHdev` (`HDEV *`) and `DxEngAltLockSurface` (`SURFOBJ *`) — emitted as commented-out C in the stitched output. Flip-link would dangle if the dispatch table (`gaEngFuncs`) references them.
**Root cause:** the scanner's function-signature detection trips on a pointer return type written on its own line (`HDEV *\nAPIENTRY\nFoo(...)`) or inline (`SURFOBJ * APIENTRY Foo(...)`).
**Fix / oracle:** the catch is the **symbol-coverage diff** — `comm -23 <(llvm-nm --defined-only build-i386/.../foo.c.obj | grep ' T ' | sed 's/^_//;s/@.*//' | sort -u) <(grep -hA1 '#\[no_mangle\]' foo.rs | grep -oE 'fn [A-Za-z0-9_]+' | sed 's/fn //' | sort -u)` MUST be empty before flipping. Hand-translate the dropped pointer-returning fns (usually trivial). **c2rust anchoring can't backstop win32k**: c2rust 0.22.1 panics `Could not find CDeclId(N) in TypedAstContext` on win32k's heavy header AST (dxsdk/ddk/freetype) even after stripping the binary-PCH flags (`-Winvalid-pch -Xclang -include-pch -Xclang …pch`, keep the plain `-include cmake_pch.h`).

#### c2rust AST-unsupported → `--c2rust` HARD FAILS (no silent fallback); fix the C side first (msgina/shutdown.c, 2026-06-21)
**Symptom:** `rustconv translate <f>.c --c2rust` aborts with `C2RustError: no .rs produced for <f>.c (AST unsupported)` (or missing binary / no compile_commands entry / timeout). As of 2026-06-21 `--c2rust` is **strict**: a requested anchor that can't be produced is a HARD FAIL *before any paid call*, not a silent degrade to pure-LLM (AGENTS.md "fail loud"). msgina/shutdown.c (1450 LOC of dialog procs) hits the "AST unsupported" case on c2rust 0.22.1.
**Preferred remedy (do this first):** **adjust the C source into a c2rust-digestible form** so you KEEP the compiler anchor — c2rust AST breaks are usually one offending construct (an unusual initializer, a GNU-ism, a macro that expands to something c2rust's typed-AST chokes on). Massaging that (a behavior-preserving rewrite on the C side) is far better than losing the anchor for the whole file.
**Fallback (deliberate, only if the C can't be massaged):** re-run that single file **without** `--c2rust` to opt into a pure-LLM translation as an explicit choice — and review it harder (no compiler anchor = higher hallucination risk, esp. for pointer-math/crypto; fine-ish for dialog/UI).

### Split-TU translation covers only the first chunk
**Symptom:** crate compiles green ×3 but i386 `ninja win32k` flip-link fails `undefined symbol` for every split-chunk export — discovered only after wiring, not at `cargo check`.
**Root cause:** win32k files like `painting.c` end with `#include "painting.split2.c"` → one merged `.obj` with all chunks' exports. `rustconv translate foo.c` parses only foo.c's text and does NOT follow the `#include`.
**Fix / oracle:** `ls win32ss/user/ntuser/<f>.split*.c` BEFORE translating. Translate every `.splitN.c` separately, assemble ONE `foo.rs` keeping the chunk-1 preamble/use/macros once and deduping the rest. Authoritative export list: `llvm-nm --defined-only build-i386/.../foo.c.obj | grep ' T '`. Known split files: painting, winpos, message, window, keyboard.
**Source:** `feedback_win32k_split_tu_translation.md`

### Stubbed / degraded function bodies (constant-return or pass-through)
**Symptom:** a 20+-line C function comes out as `{ NewIrql }`, `{ 0 }`, `{ PASSIVE_LEVEL }`, or forwards to a non-formatting helper. pic_pic.rs IRQL fns returned PASSIVE_LEVEL always; `kdb_cli.c` via `batch` stubbed all 22 handlers as `1u8`/`0u8`.
**Root cause:** Sonnet (and `batch` especially) truncate complex bodies — inline asm + PCR access + bit-twiddling — without hitting max_tokens. `batch` is worst on >1000 LOC; stitcher oracle-injection can also mangle control flow (orphan `} else {`, raw-C fragments, literal `...`).
**Fix / oracle:** use `translate` not `batch` for >1000 LOC. `grep -cE '^\s+(0u8|1u8)\s*$' file.rs` (high vs fn count = stubbed). Read 2-3 hottest fns against C before integrating; always audit IRQL/DPC/interrupt/scheduler hot paths. Re-translate a single stub via `--fn <name>` (cents). Drop injected SmokeDeliveryOracle DbgPrint — it's instrumentation, not C.
**Source:** `feedback_rustconv_stubbed_bodies.md`, `feedback_rustconv_translate_gotchas.md`

### Placeholder constants — the LLM invents a value for an enum/#define it can't resolve (fastfat, 2026-06-14)
**Symptom:** logic is structurally faithful to the C but uses the *wrong number*, so a guard never fires / an enum never matches. Five distinct fastfat boot failures came from this one class, each unmasked by fixing the prior: `if NodeType==0/*FAT_NTC_ROOT_DCB*/` (real 0x504) → root-DCB guard dead → wild-ptr `KeAcquireGuardedMutex` **0x1E**; `FCB_CONDITION` (FcbGood=1/Bad=2/NeedsVerify=3) defined with placeholder values inconsistently across **9 files** (one file sets `FcbGood=0`, another checks `FcbGood=2`) → every fresh FCB mis-classified → **STATUS_FILE_INVALID 0xC0000098**; `FCB_LOOKUP_ALLOCATIONSIZE_HINT` `=0x4000`/`=0x7FFF..` vs real `((LONGLONG)-1)` → set-vs-check sentinel never matches.
**Root cause:** the constant lives in a *header* (`fastfat.h` enum, a `#define`), not the `.c` being translated, so the LLM emits a local `const FcbGood: u32 = 0; // placeholder` (or a bare literal `0/*FcbGood*/`) and the rest of the function is internally consistent with the guess. **Two reasons this slips every per-file behavioral check:** (1) it's *cross-file* — file A writes the field, file B reads it, each looks fine alone; (2) semdiff/semantic-diff compares the Rust to *the .c body it was generated from* — the value isn't in that body, and a dead guard / never-true match is structurally identical to the correct one. NT_ASSERTs that would catch it in C are no-ops under DUMMY_PSEH.
**Fix / oracle:** the LLM *self-flags* these — grep the crate for the tells: `grep -rnE '/\*placeholder\*/|// ?placeholder|// ?assumed|/\*Fcb|0/\*[A-Z]' src/`. Cross-check every such const against the C header value, and **run `verify_constants.py` over driver crates, not just ntoskrnl** (it cross-checks Rust consts vs C headers — it was never wired to fastfat, which is the actual process gap here). Prompt lever: instruct the translator to mark *any* invented value with `// placeholder` so it's greppable (turns the failure mode into a detectable signal — slightly lazier but far more findable). When a constant appears in N files, make ONE canonical def and re-export; N independent placeholder defs is the smell.
**Source:** `40dd6711a5d` (fastfat-port), [[project_fastfat_rust_track]]

#### Completing a dropped stub: verify the C vs PRISTINE upstream before "fixing" its apparent bugs
**Symptom:** you go to translate a recovered stub (e.g. cdfs `CdOpenByFileId`, emitted by the grind as `STATUS_UNSUCCESSFUL` + "body missing from the provided C snippet") and the source C looks broken — a `while (TRUE)` loop with NO success `break` (only an error `try_return` on overshoot/!Found), or pointer handling that seems to NULL-deref.
**Trap:** the instinct is "the repo's C was mangled, let me reconstruct the *correct* logic." Often it WASN'T — it's faithful upstream with a genuine long-standing latent bug on a rarely-hit path. cdfs `CdOpenByFileId` is verbatim reactos master: both the path-table and dirent `while(TRUE)` walks lack a success break, so CdLookupNext* always advances → an *uncached* open-by-id always returns `STATUS_INVALID_PARAMETER` (terminates via overshoot/!Found, NOT an infinite loop), making the Fcb-construction code after each loop effectively dead; only an already-cached Fcb completes the open. "Fixing" it would silently DEVIATE from upstream behavior.
**How to apply:** (1) confirm the repo file is pristine — `git log --oneline -- <file>` (cdfs/create.c had only one unrelated touch). (2) `WebFetch` the matching `raw.githubusercontent.com/reactos/reactos/master/...` and diff the exact function. (3) if it matches upstream, translate FAITHFULLY — preserve the structure incl. dead/buggy paths, add `#[allow(unreachable_code)]` if a divergent loop makes a tail unreachable, and DOCUMENT the upstream behavior in a header comment. A faithful mirror means a later upstream fix "just works" in the Rust. Only treat it as a mangle (and reconstruct) if it genuinely diverges from upstream.

### DeepSeek/grinder fabrication-audit checklist (compiles-but-wrong at scale)
**Symptom:** A DeepSeek-`--mode mechanical`/c2rust-anchored grind of a #define/macro-dense driver (HID parser, descriptor parsers) cargo-checks green but is silently wrong: parsing breaks, link fails, or a constant has the wrong facility.
**Root cause:** Three distinct fabrication classes the grinder hides behind a green check: (1) **bindgen can't eval fn-like macros** (`HIDP_ERROR_CODES(SEV,CODE)=((SEV<<28)|(0x11<<16)|CODE)` → ZERO `HIDP_STATUS_*` in bindgen output → the grinder FABRICATES them with a wrong facility (0x3FFE vs 0x3FEE) and wrong type (`i32`, bit-31 overflow)). (2) **undefined consts used in `match` arms** → each becomes a silent catch-all BINDING (only the first arm runs) → dispatch/parse silently broken, surfaced ONLY as "unreachable pattern" WARNINGS. (3) **dropped function bodies** emitted as `extern` STUBS → undefined symbol at LINK.
**Fix / oracle:** after integrating any grinder output run all three checks: (1) re-derive every `STATUS_*`/facility const from the real header into the `-sys` crate as the source of truth (`0x....u32 as i32` for bit-31 values), delete local fabrications; cross-check with `verify_constants.py` + a Sonnet-vs-C review tier (earns its keep on constant-dense code). (2) `grep -c "unreachable pattern"` in the cargo build — **must be 0**; define the missing consts matching the scrutinee type (HID item tags are a 4-bit nibble 0x8/0x9/0xa/0xb/0xc — the grinder's 0x80/0x90 were wrong). (3) recover dropped bodies from `tools/rustconv/.responses/` (durable bank) and hand-translate (watch for faithful C quirks like an early-return path that doesn't write outputs). Mechanical mode also declares all externs `extern "C"` → NTAPI imports break i386 link; the ABI sweep (`extern "system"` for NTAPI, inline DDK helpers from `ntkrnl_sys::inline::tier1` not `ddk`) is the engineer's job, forbidden to the grinder.
**Source:** `project_driver_residue_track.md`, `feedback_undefined_const_in_match_silent_binding.md`

### Constant linter as a mandatory driver-crate gate (verify_constants.py, now driver-aware)
**Symptom:** A DeepSeek/c2rust driver grind cargo-checks AND boots green, but carries
fabricated `#define` values on paths the smoke window never exercises (SCSI passthrough,
mount-notify, remote-DB, IOCTL tails). c2rust does NOT save you here — the hallucination is
in the hand-written `pub const` glue, not the transpiled bodies. These ship silently and bite
later: e.g. `STATUS_MORE_PROCESSING_REQUIRED=0x103` (=PENDING, not `0xC0000016`) wrong-returned
from a completion routine; `IRP_MJ_SCSI=0x0B` (=FLUSH, not `0x0F`) → SCSI passthrough mis-dispatch;
`FILE_OPEN_BY_FILE_ID=0x200` not `0x2000`; `VALUE_BUFFER_SIZE=512` not 2048 (undersized buffer).
**Oracle:** `verify_constants.py` is now driver-crate-aware — run it on EVERY driver crate before
and after landing, with the driver's own C dir as the extra catalog:
```
python3 tools/rustconv/rustconv/verify_constants.py sdk/lib/rust/<drv>-rust/src \
    --extra-headers drivers/<...>/<drv> [--skip <collision-names>] [--fix]
```
It parses `pub const NAME: T = V;` (not just bare `const`), globs `--extra-headers` dirs for the
driver-private `#define`s (pool tags, node types, IO flags), and is conflict-aware (won't auto-fix
a name with arch-divergent header values). One run across the 5 landed DeepSeek crates found
**19 real fabrications in disk + mountmgr** (boot drivers), 0 in the harvest_consts-built crates
(partmgr/fs_rec/pcix) — proof the gate belongs in the pipeline, not just at first-translate.
**Two traps learned wiring it:** (a) the `--fix` regex anchored after `rstrip()` only, so it
silently no-op'd fn-local (INDENTED) consts while printing "FIXED" — fixed to capture leading
whitespace; always re-run the linter after `--fix` to confirm `0 mismatches`, don't trust the
FIXED count. (b) Name-collisions are real false positives: mountmgr has a module-local
`LETTER_POSITION: isize = 4` (a `\??\C:` offset) distinct from the bindgen `LETTER_POSITION = 12`
(`\DosDevices\C:` offset) — `--fix` would wrongly rewrite it. Use `--skip NAME` to report-not-fix
those. `--unverifiable` (ALL_CAPS const not in ANY header) is too noisy to gate on (~4k hits);
opt-in investigative only.
**Source:** [[feedback_llm_invented_enum_constants]], [[project_disk_rust_track]], [[project_mountmgr_rust_track]]

### PSEH (`_SEH2_TRY/_FINALLY/_EXCEPT`) hand-lowering recipe (FS-driver translations)
**Symptom:** a PSEH-dense driver (fastfat: 276 try-blocks; cdfs/vfatfs similar) translates
with the SEH macros left as raw tokens in a fraction of functions — `cargo check` shows bare
`error:` PARSE errors ("found `_SEH2_EXCEPT`", "expected `{`, found `||`"), NOT E-codes, which
BLOCK the whole crate (resolve_imports can't run until it parses). Most blocks DO get lowered
(left as `// _SEH2_TRY` comments); only the live residue (fastfat: 19 of 276) needs hand-work,
concentrated in a few files. Find them: `cargo check 2>&1 | grep -oE "src/[a-z]+\.rs" | sort |
uniq -c` — the per-file parse-error histogram is the worklist.
**Lowering rules (DUMMY_PSEH — the whole kernel uses it; EXCEPT handlers are DEAD code, validate
pointers instead of catching):**
- `_SEH2_TRY{ body } _SEH2_EXCEPT(filter){ handler } _SEH2_END;` (and the `!`-macro form
  `_SEH2_TRY!{…}_SEH2_EXCEPT!(…){…}_SEH2_END!;`) → emit the **body only**, drop the handler. It
  references nonexistent `_SEH2_GetExceptionCode()/_GetExceptionInformation()` intrinsics and is
  unreachable anyway. This is every `FatFsd*`/dispatch wrapper.
- `_SEH2_TRY{ body w/ try_return/try_leave } _SEH2_FINALLY{ cleanup } _SEH2_END;` → the FINALLY
  must ALWAYS run, so: wrap body in a labeled block, run cleanup AFTER it:
  `'tryblk: { …body…; FINALLY-runs-after } ; { …cleanup… }`. `try_leave(NOTHING)` → `break 'tryblk;`;
  `try_return(Status = X)` → `{ Status = X; break 'tryblk; }`. NEVER drop the finally — a skipped
  cleanup (unpin BCB / release FCB / free pool) is a runtime leak or lock-imbalance, not a parse bug.
- bare `_SEH2_TRY{ body } _SEH2_END;` → `{ body }`.
- LLM-fabricated `extern { fn _SEH2_TRY()->i32; … }` + a `__seh_frame` if/else (ea.rs) → delete the
  fake externs; the `if __seh_frame==0 {try} else {handler}` collapses to the try body (handler dead).
Nested blocks lower inside-out. Validate per-file: `grep -nE "_SEH2_(TRY|FINALLY|EXCEPT|END|LEAVE)"
<f> | grep -v '//'` must be empty, then re-run cargo check. Did fastfat flush.rs first to validate
the recipe (3 blocks: 1 dispatch-EXCEPT, 1 triple-nested TRY/FINALLY+try_leave, 1 bare) → file clean.
**Source:** [[project_fastfat_rust_track]]

### Odd ATAPI PIO transfer uses dword I/O and wedges CD reads
**Symptom:** Boot reaches SMSS, then registry hive reads never complete; storage timeouts churn through `AtapiTimerDpc -> AtapiInterruptDpc -> AtapiResetController`, and the system eventually bugchecks because the boot CD is inaccessible. GDB/debugcon can prove real interrupts fire, so IRQ wiring is a red herring.
**Root cause:** On ATAPI PIO reads, QEMU/IDE can report an odd final word count with DRQ still set on large CD reads (e.g. READ(10) length over 64 KiB). The C has a 64K/odd-count special case: unaligned ATAPI PIO must fall through to the 16-bit transfer path. The Rust port gated that fallback on `g_opt_VirtualMachine == VM_BOCHS`, so this QEMU path kept dword I/O enabled and missed the final word/DRQ transition.
**Fix:** For ATAPI reads with an odd `wordCount`, force the 16-bit path regardless of VM detection: `if wordCount & 1 != 0 && atapiDev != 0 { ... } else if DFLAGS_DWORDIO_ENABLED { ... }`. When storage reads time out after interrupts are proven real, instrument opcode/count/status around `AtapiInterrupt__` before chasing reset loops.
**Source:** uniata-rust i386 boot-CD read hang, 2026-06-13

### rustconv drops ALL non-fn header content at scale (structs / const-tables / #defines / FuncTables)
**Symptom:** A bulk `translate` of a unit compiles per-function but is missing every file-local struct, const lookup table, `#define`, and static FuncTable — E0425/E0412 undefined-name storms, or worse, silent wrong behavior from a missing default table.
**Root cause:** rustconv translates function bodies only; the scanner's preamble captures the .c's top-of-file but `translate`'s stitcher never emits `scan.globals_text` — file-local typedefs/structs/`#define`-consts/`const DEVVTBL` FuncTables/static arrays are dropped entirely. (FreeLDR fs unit: bootsector/dirent/volume-info structs + const DEVVTBL tables + #defines all gone.)
**Fix / oracle:** restore the dropped structs/const-tables/#defines into the .rs (or a keystone module) FROM the .c/.h — put only UNIQUE types/consts in a shared keystone (cross-module Arc*/Fs*/Mm*/Ui* there → E0659 ambiguity; use a file-local `extern "C"` block instead). Recurring per-file traps: C `L"..."` is a Rust LEX ERROR → a `wide!()` const macro (align-2 `[u16;N]`); FuncTable fns referenced as `Option<extern "C" fn>` need `extern "C"`, not bare `unsafe fn` (fn-item vs fn-pointer); `((*p).Field)(a)` → `.Field.unwrap()(a)`; bindgen `LARGE_INTEGER` is `.u.LowPart` NOT `.__bindgen_anon_1.LowPart`; verify AI-emitted pool tags / sizes against C (it gets them wrong — `from_be_bytes`/`from_le_bytes` of the byte string).
**Source:** `project_freeldr_sweep.md`

### Header-only driver (kdgdb-class): rustconv leaves header content unimplemented
**Symptom:** A non-ntoskrnl/hal driver translated per-.c won't compile (~300 errors from few roots): unresolved cross-module globals, missing FORCEINLINEs, empty-`#define` macros externed as varargs C fns, opaque-vs-full struct mismatches.
**Root cause:** rustconv translates each .c in ISOLATION and never sees the driver's own .h — header content (the driver's own `extern` globals defined in one .c, `FORCEINLINE` helpers, empty `#define` no-ops, fixed-VA macros) has no .c body to translate.
**Fix / oracle:** (a) Cross-module globals: make the def `pub` (keep `#[no_mangle]`), `use crate::<mod>::<sym>` in consumers (grep the C for the defining file). (b) Header-only FORCEINLINEs (gdb_tid_to_handle etc.): reconstruct in a hand-written `<name>_h.rs` module. (c) Empty-macro `#define` (KDDBGPRINT no-op): replace the bogus varargs extern with `macro_rules! X { ($($a:tt)*)=>{{}} }` + rewrite calls to `!()`. (d) Fixed-VA macros that are NOT symbols: `SharedUserData = ((KUSER_SHARED_DATA*)KI_USER_SHARED_DATA)` → local `const` (not `static`: `*mut` isn't Sync) per arch (i386 `0xFFDF0000`/64-bit `0xFFFFF78000000000`), `use crate::SharedUserData` in EVERY module (a stray `use ntkrnl_sys::globals::*` re-pulls the unresolvable extern); `RtlEqualMemory` = `!memcmp` macro → local inline. (e) ntoskrnl DATA exports (`KdDebuggerNotPresent`) → `#[link(name="ntoskrnl.exe", kind="raw-dylib", modifiers="+verbatim")] extern { static mut }` (plain extern won't cross-DLL-link on i386). (f) `detect_target` ordering: register the longer-prefix target FIRST (`/kdvm/` before `/kdcom` else `/kdvm/` matches `/kdcom`). bindgen enum variants are bare consts (`KdPacketReceived` not `KDSTATUS::...`); intrinsics (`__writecr3`) → inline asm; CRT/cportlib fns → `extern "C"`/`extern "system"` per the actual ABI, resolve at final link.
**Source:** `project_kdgdb_translation.md`

### Dropped function-name → bare tuple no-op
**Symptom:** `(arg1, arg2, arg3);` where C had `Foo(arg1, arg2, arg3)` — valid Rust, semantically a no-op. Hides in cohabitation (lld picks the C symbol); surfaces as silent register no-op when the C twin is removed, BUGCHECKing far downstream (e.g. `PsLocateSystemDll → 0x6B`). pcibus.rs had 12+ dropped PCI config reads/writes.
**Root cause:** rustconv strips the function-name identifier on translation, leaving a side-effect-free tuple expression. Also: discarded direct calls translated to `core::ptr::null_mut()`.
**Fix / oracle:** `grep -nE '^\s+\([A-Za-z_][A-Za-z0-9_]*,' file.rs` — each hit is a candidate dropped call; verify against C. If a "swap C out for Rust" change links clean but BUGCHECKs far from the file in question while cohabitation worked, suspect this first.
**Source:** `feedback_rustconv_dropped_fn_calls.md`

### Stitcher comment-collapse drops whole fn bodies into a `//` line — and how to find them (fastfat, 2026-06-13)
**Symptom:** Crate is "cargo-clean ×3" but a cohab driver is non-functional: callers reference `FatCommonRead` etc. via `extern "C"`, the real body is nowhere, and a hand-written cohab `STUB_BUGCHECK()` provides the symbol → false-green link that bugchecks on first real I/O. fastfat had 7 core bodies (incl. the 29 KB `FatCommonRead`) gone this way.
**Root cause:** rustconv `stitcher.py::_merge_extern_blocks` space-JOINS multi-line extern decls (`current = current + " " + stripped` until `;`). When a banked decl line ends in a `// annotation` (e.g. `fn X(...) -> i32; // BOOLEAN`), the join concatenates every following definition onto that one physical line — so `pub unsafe extern "C" fn FatCommonRead(...) { body }` becomes trailing text of a `//` comment and is silently dead. Per-file `assemble_file` ALSO appends the whole original C source as `//` comments, so a 45–88% comment ratio is NORMAL — do not use comment ratio as the corruption signal.
**Fix / oracle:** NEVER decide "is fn X live?" by line-grepping `fn X(` (conflates extern-decls, multi-line sigs, comment-collapsed copies, embedded-C comments — gave 66 false "drops", 61 of which were already-`#[no_mangle]` dupes). AUTHORITATIVE = nm on the built `.a`: `nm` → defined set `D` (`T/t/D/d/R/r/B/b`) and undefined set `U`; the true external gap is `U − D` (a static archive resolves intra-archive U↔T, so raw U overcounts ~4×). A genuinely-dropped fn = `(U−D) ∩ {names with a bank file under .responses/fastfat__<file>/}`. Recover those (and ONLY those) from the bank with a targeted re-injection (`tools/rustconv/reinject_dropped.py`: extract the `#[no_mangle] pub fn …{}` span, move the bank's top-level placeholder consts function-local like the assembler does, strip the live self extern-decl to avoid E0428), then repair the reopened cascade. Raw injected bodies carry fabricated placeholder consts (`IRP_PAGING_IO=0x1`) — run verify_constants after.
**Source:** `project_fastfat_rust_track.md`

### Lock/RAII-pair epilogue dropped when inlining a macro
**Symptom:** acquire>release count imbalance; leaked console lock → timing-dependent 0xF4 critical-process-termination. coninput.rs had 5 `ConSrvGetConsole` / 0 `ConSrvReleaseConsole`. Heisenbug — any DbgPrint/gdb observation masks it.
**Root cause:** `CON_API(Name,...)` macro locks at entry and `ConSrvReleaseConsole` before every return; rustconv inlined the acquire but dropped the release epilogue. Generalizes to any lock/`__finally` the C expressed as a macro.
**Fix / oracle:** `grep -c <Acquire>(` vs `grep -c <Release>(` per file; acquire>release = leak. Count-balance ≠ per-path-balance — audit each early return. Fix by splitting into `<Name>Impl` + single-release entry. Note: a timing-changing fix that only *partially* helps a Heisenbug is a contributing factor, not THE cause — keep going.
**Source:** `feedback_con_api_lock_release_dropped.md`

### Match arm with out-of-scope CONST becomes catch-all binding
**Symptom:** compiles, links, boots, but runtime dispatch is broken — the first such arm eats every value, later arms unreachable. i8042prt: `CmResourceTypePort` arm ate the IRQ descriptor → interrupt never connected → no keys/mouse, yet still reached `shell ready`.
**Root cause:** C `switch(x){case CONST:}` → Rust `match x { CONST => }` where CONST isn't in scope as a const/unit-variant; Rust treats the bare ident as an irrefutable binding pattern. The consts simply weren't imported (IOCTL_*/IRP_MN_*/CmResourceType* live in headers bindgen never covered).
**Fix / oracle:** `cargo build 2>&1 | grep "unused variable: \`[A-Z]"` — every uppercase "unused variable" in a `match` is this bug; rustc also emits "matches some of the same values...unreachable". Zero warnings = all arms are genuine patterns. Define missing consts matching the SCRUTINEE TYPE exactly (MinorFunction→u8, IoControlCode→u32). Boot-to-shell-ready does NOT exonerate; the input-delivery smoke oracles caught this.
**Source:** `feedback_match_arm_binding_not_const.md`

### Wrong constant value invented (not truncation)
**Symptom:** a hand-defined magic const with a flat-out wrong number; same const exists with two different values in sibling modules. `CmResourceTypeDeviceSpecific = 0x81` (that's DevicePrivate; correct is 5) → arm64vid skipped the framebuffer descriptor → ScreenWidth=0, BUGCHECK 0xB4, three hops downstream.
**Root cause:** when a symbol isn't in the bindgen catalog the translator invents a local `pub const NAME = 0x..` and can pick the wrong number.
**Fix / oracle:** when two modules define the same `const NAME`, diff the VALUES — silent disagreement is a bug. For loader→kernel/driver handoff blobs, a raw hex-dump on the READER side shows the wrong byte directly. `verify_constants.py` does NOT yet cover CM_RESOURCE_TYPE — extend it.
**Source:** `feedback_rustconv_wrong_constant_value.md`

### Static-table complex-literal fields nulled out
**Symptom:** `null_mut()` where C had a string-literal/MULTI_SZ default. `RTL_QUERY_REGISTRY_TABLE.DefaultData` MULTI_SZ defaults all became `null_mut()`; on a fresh boot the registry values are absent so `RtlQueryRegistryValues` derefs the NULL default inside ntdll before any QueryRoutine → smss 0xF4.
**Root cause:** rustconv emits `null_mut()` for the hard string-literal initializers in a `static`/`const` table instead of the literal.
**Fix / oracle:** audit any translated `static` table for `null_mut()` fields that were non-NULL string literals in C. Use a `wstr!` const-macro (align-2 `[u16;N]`, preserves embedded `\0` for MULTI_SZ). Watch the byte-literal split: `b"D\0l\0l\0".as_ptr() as *const u16` (pre-widened) is CORRECT; `b"debug\0".as_ptr() as *const u16` (2+ consecutive non-`\` chars = plain ASCII) is a BUG → use `wstr!`.
**Source:** `feedback_rustconv_drops_table_default_data.md`

### Data-table-heavy file (rtl error.c-class): LLM silently drops table ENTRIES
**Symptom:** A file that is ~95% data (e.g. error.c = 19 NTSTATUS→DOS tables of 400+ entries + ~696 winerror.h/ntstatus.h constants) translates and compiles, but a table is short by one or more entries (one table had 71 of 72) and array sizes are miscounted → unreliable lookups.
**Root cause:** The LLM transcribes long numeric/string tables unreliably — it silently drops entries without hitting max_tokens. Compiles fine because nothing asserts the count.
**Fix / oracle:** for data-table-heavy files, EXTRACT the tables deterministically from the C/headers with a gen script, don't trust the LLM transcription. Two reusable patterns: `gen_winerror_consts.py` (scan referenced `ERROR_*/STATUS_*/...` idents in the table region only, resolve from headers across `plain` / `_HRESULT_TYPEDEF_(0x..)` / `((NTSTATUS)0x..L)` forms) and `gen_error_tables.py` (extract the tables + dispatch array verbatim with a `declared [N] == actual count` assertion). lib.rs then = generated mods + hand-verified dispatch fns.
**Source:** `project_rtl_misc_leaves.md`

### RTL_CONSTANT_STRING global UNICODE_STRINGs stubbed to empty (mountmgr, 2026-06-12)
**Symptom:** A driver with `UNICODE_STRING DeviceMount = RTL_CONSTANT_STRING(L"\\Device\\MountPointManager");` style globals translates to `pub static mut DeviceMount: _UNICODE_STRING = _UNICODE_STRING { Length: 0, MaximumLength: 0, Buffer: null }` — ALL fields zero. At runtime: `OB: Invalid symbolic link target - ` (empty) when the driver creates symlinks from these, then a downstream **BUGCHECK 0x19 BAD_POOL_HEADER (subtype 5)** because string-builder funcs compute `dst.Length = name.Length - DosDevices.Length(0) + DosGlobal.Length(0)` and `copy_nonoverlapping(DosGlobal.Buffer(NULL), …)` off the zeroed globals → wrong sizes / null derefs corrupt the pool.
**Root cause:** `RTL_CONSTANT_STRING` is a macro (`{ sizeof-2, sizeof, L"..." }`); the LLM can't expand the link-time string literal so it emits an all-zero `UNICODE_STRING` — the string-global analogue of "Static-table complex-literal fields nulled out", but it COMPILES and links clean (nothing asserts non-empty).
**Fix / oracle:** grep every `static mut <NAME>: _UNICODE_STRING = _UNICODE_STRING {\n Length: 0,` in the crate; for each, recover the C `RTL_CONSTANT_STRING(L"…")` (or its macro, e.g. `MOUNTMGR_DEVICE_NAME`) and emit `static <NAME>_WSTR: [u16; N] = [<utf16…>, 0]; … { Length: (N-1)*2, MaximumLength: N*2, Buffer: &<NAME>_WSTR as *const [u16;N] as *const u16 as *mut u16 }`. One agent-generated script does all of them from the harvested strings. (NB: some get translated correctly — diff against the C, don't blanket-replace.)
**Diagnostic heuristic that cracked it:** **byte-identical BUGCHECK args across multiple code-change+rebuild cycles ⇒ the bug is in INVARIANT data (a static initializer), not in any code path you're editing.** Three rebuilds with three different code fixes left `0x19 (5, 0x…96e50, 0xca, 0x…96d70)` byte-for-byte identical → stop editing code, audit the statics. (When the string fix finally changed the addresses, it confirmed the static-data root cause.)

### Driver cohab i386 CC-fold: types_*.rs (gen_driver_types) + ddk_shims are the per-symbol authority (mountmgr, 2026-06-12)
**Symptom:** Driver links clean ×3 but i386 has N `ud2` folds in named fns (amd64/arm64 fold-free). A crate-wide def↔decl CC consistency pass over the MODULE .rs files reports 0 mismatches, yet folds persist. IR shows a clean call (e.g. `call x86_fastcallcc void @ObfDereferenceObject(...)` / a driver-internal fn) immediately followed by `unreachable`.
**Root cause:** the per-symbol CC authority for a driver's OWN functions is **`types_{arch}.rs`** (gen_driver_types bindgen of the C prototypes in the private header) — NOT the module decls and NOT a blanket "NTAPI=stdcall" rule. Driver-internal helpers with NO `NTAPI` in their C prototype are **cdecl** (`extern "C"`); the translation often guesses `extern "system"` for cross-module decls. A def/decl that disagrees with the bindgen authority is an in-module CC contradiction → fold. The consistency check MUST include `types_*.rs` (authority), `ddk_shims.rs` (the cohab fn DEFINITIONS), and macro-shims (`AllocatePool`/`FreePool` — provided by ddk_shims, declared per-module, NOT in the C-prototype authority so reconcile to the shim's CC). My first sweep flipped DEFS the wrong way (to "system") because I only diffed modules-vs-modules; reconciling to the types_*.rs CC (mostly "C") fixed it.
**Fix / oracle:** build `auth = {fn → CC}` from `types_i386.rs` extern-block ABIs (stdcall→"system", else "C") PLUS the `ddk_shims.rs` `pub extern "X" fn` defs; reconcile every def and every extern-block decl across all module files to `auth[fn]`; then `objdump | awk '/<.*>:/{fn=$0}/ud2/{print fn}'` to confirm 0 named-fn folds. Also: a macro-shim with an inconsistent PARAM TYPE across decls (`AllocatePool(usize)` def vs `AllocatePool(u32)` decls) is a separate runtime pool-corruptor on amd64 (callee reads 64-bit reg, caller set low 32) — make the param type uniform too. And a cross-module DATA static declared with two TYPES (`DatabasePath: *mut u16` in 3 modules vs `_UNICODE_STRING` in 2) makes `.Buffer`/`.Length` read 8 bytes past the real symbol — grep `static (mut )?\w+:` and group by name to find type disagreements.

### Self-referential static initializer → {null, null}
**Symptom:** a `static X: LIST_ENTRY = LIST_ENTRY { Flink: null_mut(), Blink: null_mut() }` that walkers treat as empty/initialized; head-sentinel walks deref NULL or loop forever. fat::DirectoryBufferListHead → arm64 UEFI fault at FatOpen+0x230 (NULL+0x10).
**Root cause:** C `LIST_ENTRY LH = {&LH, &LH}` is a link-time constant; Rust statics can't reference their own address, so rustconv falls back to `{null, null}`.
**Fix / oracle:** grep `LIST_ENTRY = LIST_ENTRY {` in all .rs; verify each static has a one-shot `InitializeListHead` before first access, or add a lazy-init guard at every reader (`if X.Flink.is_null() { InitializeListHead(addr_of_mut!(X)) }`). Same hazard for sentinel-node trees and cache anchors.
**Source:** `feedback_rustconv_selfref_static.md`

### Narrowing cast truncates wider C operands
**Symptom:** silent value truncation. FAT: `NumberOfFats(u8) * SectorsPerFat(u32)` emitted as `(NumberOfFats * (SectorsPerFat as u8)) as u32` → 559 truncated to 47, DataSectorStart 1024 sectors short, file reads land in the FAT table → spurious ENOENT. MmAllocateMemoryWithType declared `size: u32` vs canonical SIZE_T → on 64-bit caller writes only low 32 bits, callee reads garbage in high bits.
**Root cause:** C auto-promotes `int` arithmetic operands; rustconv mis-derives the narrowest type and casts DOWN to match the smaller operand.
**Fix / oracle:** grep `as u8`/`as u16`/`as u32` in arithmetic contexts; cast UP not down (`(NumberOfFats as ULONG) * SectorsPerFat`). For widths use SIZE_T/`usize`/`as _`. Recognition: ENOENT on a verified-present file (dump 32 bytes — looks like a FAT chain = truncated data-sector-start); `%s` spam / MmAllocate weirdness.
**Source:** `feedback_rustconv_truncation_bugs.md`

### u64 sentinel cast as i64 inverts a bounds check
**Symptom:** `>= bar as i64` where `bar` is a u64 "no limit" sentinel. `SectorCount = 0xFFFFFFFFFFFFFFFF as i64 = -1` → `16 >= -1` TRUE → every Seek returns EINVAL → IsoMount can't reach PVD (sector 16) → boot dies at "Failed to load rosload.exe".
**Root cause:** C promotes the signed side to unsigned in mixed compares; rustconv "matches the types" with `as i64`, flipping u64::MAX to -1.
**Fix / oracle:** do the compare in u64 (cast the signed operand `as u64`). Grep `as i64.*SectorCount`, `as i64 - 1`, `>= .*\.SectorCount as i64` and any u64 sentinel (0xFFFFFFFFFFFFFFFF / ULONGLONG_MAX = "unbounded") compared against an i64/LONGLONG.
**Source:** `feedback_u64_sentinel_as_i64.md`

### write_bytes element-count vs byte-count confusion
**Symptom:** `write_bytes(typed_ptr, 0, size_of::<T>())` zeroes `size_of::<T>() * size_of::<T>()` bytes — a 96-byte struct zeros ~9KB, clobbering adjacent pool, wedging boot before any debugcon.
**Root cause:** `core::ptr::write_bytes::<T>(ptr, val, count)` writes `count * size_of::<T>()`; rustconv keeps C `memset(p, 0, sizeof(*p))`'s byte count in the 3rd arg without casting the pointer to `*mut u8`.
**Fix / oracle:** correct forms are `write_bytes(p as *mut u8, 0, byte_count)` or `write_bytes(p, 0, 1)`. Any `write_bytes(typed_ptr, _, size_of::<T>())` is a bug — flag it when reviewing memset/RtlZeroMemory/RtlFillMemory translations.
**Source:** `feedback_write_bytes_element_count.md`

### copy_nonoverlapping/copy element-count vs byte-count (the RtlCopyMemory sibling)
**Symptom:** a name/buffer copy writes 2× the intended bytes, silently corrupting the *adjacent* object. cdfs `CdFindPrefix` (prefxsup.rs) restored a matched path prefix's on-disk case via `copy_nonoverlapping(ExactCaseName.Buffer, FinalName.Buffer, ExactCaseName.Length)` on `*mut u16` — copying `Length` *elements* (= 2×Length bytes), reading past "reactos" into the adjacent `IgnoreCaseName` ("REACTOS") and clobbering the next component in the shared path buffer (`\SYSTEM32\` → `\EACTOS32\`). Every directory traversal after the first prefix hit then returned STATUS_OBJECT_PATH_NOT_FOUND → BUGCHECK 0x6B at PsLocateSystemDll opening `\reactos\System32\ntdll.dll`. amd64/arm64 unaffected by the *value* but the 2× overrun is arch-independent.
**Root cause:** `core::ptr::copy_nonoverlapping::<T>(src, dst, count)` / `core::ptr::copy` take `count` in **elements**; the LLM passes a `UNICODE_STRING`/`CD_NAME` `.Length`/`.MaximumLength` (a BYTE count) directly with `*mut u16` pointers. C's `RtlCopyMemory(dst, src, Length)` is byte-based.
**Fix / oracle:** cast both pointers to `*const u8`/`*mut u8` so `count` is bytes (matches RtlCopyMemory). Grep `copy_nonoverlapping`/`core::ptr::copy`/`copy_from` and flag any `*u16`-typed pointer paired with a `.Length`/`.MaximumLength`/`*Bytes` count and NO `as *u8` cast. (Inverse of the `.add()` on `*u16` case, same family as write_bytes above.)

### DUMMY_PSEH: `_SEH2_EXCEPT(...)` emitted as an UNCONDITIONAL bare block (handler runs on success)
**Symptom:** an exception handler's *side effects* fire on the normal/success path. cdfs `CdCreateUserMdl` (deviosup.rs) ran `IoFreeMdl(Mdl); Irp->MdlAddress = NULL;` after a *successful* `MmProbeAndLockPages`, sending cdrom.sys a read IRP with a freed/NULL MDL → BUGCHECK 0x1E in `MmGetMdlVirtualAddress`. `CdPerformVerify` (verfysup.rs) ran `CdProcessException(IrpContext, Irp, 0)` after the try body had already completed/posted the IRP → NULL-deref/double-complete.
**Root cause:** the grind translated `_SEH2_TRY { body } _SEH2_EXCEPT(filter) { handler } _SEH2_END` as no-op `_SEH2_TRY();`/`_SEH2_EXCEPT();` *function calls* followed by *bare Rust `{}` blocks* — so the handler block executes sequentially right after the try block, unconditionally. (`_SEH2_GetExceptionCode()` is a stub returning 0.) Under DUMMY_PSEH the handler should be DEAD (a raise bugchecks rather than transferring there). Distinct from the correct style (used in read.rs) that translates SEH into real gated Rust control flow with the keywords as `//` comments.
**Fix / oracle:** gate the handler so it can't run on success — `if Status != STATUS_SUCCESS { ... }` when the try body sets Status, or `if false { ... }` when the handler is purely exception-driven and dead under DUMMY_PSEH. Grep for `_SEH2_EXCEPT(` / `_SEH2_FINALLY(` as *statement calls* (not in comments, not the cdmacros.rs defs) followed by a bare `{}`; audit each handler body for destructive side effects (Io/Ex free, null-out, release, complete, raise). FINALLY blocks are usually fine (finally semantics run on both paths) unless gated on `_SEH2_AbnormalTermination()` (stub=0).

### RESOLVED — DUMMY_PSEH defeats cdfs's raise→catch→post async model (BUGCHECK 0x1E STATUS_CANT_WAIT 0xc00000d8)
**Resolution (2026-06-14):** return-threaded CANT_WAIT instead of raising — CdPrepareBuffers returns a CANT_WAIT *sentinel* (distinct from its 0/1 FoundUnaligned), CdNonCachedRead turns the sentinel + the insufficient-entries case into `Status = CANT_WAIT; break 'try_block` (FINALLY still cleans up the partial run via CleanupRunCount written through `&mut`), CdCommonRead breaks with CANT_WAIT and propagates CdNonCachedRead's return; the EXISTING post tail in CdCommonRead (`if Status==CANT_WAIT { CdFsdPostRequest } else { complete }`, the read.c:559 analog) posts to a worker, which re-runs WAIT-set (IRP_CONTEXT_FSP_FLAGS ⊇ WAIT) and does the scratch-buffer read inline. **Also fixed a co-located flag-bit bug:** CdPrepareBuffers tested `Flags & 0x1` (IRP_CONTEXT_FLAG_ON_STACK) instead of `& 0x04` (WAIT). i386/amd64 cold smoke green to shell ready. Commit `6e91ef3`. (Below is the original diagnosis.)
**Symptom:** first async (non-wait) non-cached cdfs read with a cold Fcb Mcb: `CdFsdDispatch→CdCommonRead→CdNonCachedRead→CdPrepareBuffers→CdLookupAllocation`→(deep callee during lazy allocation/dirent load)→`CdRaiseStatus(STATUS_CANT_WAIT)`→`ExRaiseStatus`→unhandled→BUGCHECK 0x1E. Reached on i386 boot right after `IoInitSystem returning TRUE`. Sync reads (Wait set) are unaffected, so it surfaces only once async paging reads begin.
**Root cause (architectural, not a single line):** C cdfs RAISES CANT_WAIT and relies on `CdFsdDispatch`'s `_SEH2_EXCEPT(CdExceptionFilter)→CdProcessException→CdFsdPostRequest` to catch it, post the IRP to a worker thread, and return PENDING (the worker re-runs with Wait=TRUE). Under DUMMY_PSEH the catch is dead, so the raise bugchecks. The Rust `CdFsdDispatch` was rewritten with a *retry-in-place* loop that (a) never calls CdProcessException and (b) couldn't help anyway because the deep code RAISES (bypassing any return). setjmp/longjmp is NOT a fix — it would skip the `_SEH2_FINALLY` lock-release cleanup between the raise site and the dispatch, deadlocking.
**Fix options (unresolved — needs a decision):** (1) thread a CANT_WAIT *return* up the async read/alloc path (each fn returns it and runs its inline finally cleanup), and make CdFsdDispatch call `CdProcessException` on a returned CANT_WAIT — precise but multi-function; (2) force-post all async non-cached reads upfront in CdCommonRead (extend the existing raw-sector `FORCE_POST`+`return CANT_WAIT` at read.rs ~153 to all `!Wait && NonCachedIo`) + fix dispatch to post on CANT_WAIT return — contained but a perf regression and read-only (write/dirctrl async paths could still raise). Detail in `project_cdfs_rust_track.md`.

### Variadic C function degraded to a non-formatting stub
**Symptom:** `RtlStringCbPrintfA` emitted as a forward to `RtlStringCbCopyA` — every `%u`/`%s`/`%x` left LITERAL. `multi(0)disk(0)cdrom(%u)` reached ArcOpen with literal `%u` → rosload.exe never loaded.
**Root cause:** stable Rust can't *define* a variadic C fn (`c_variadic` unstable on the def side); when the C is a `static __inline` with no out-of-line symbol, rustconv emits a degraded copy stub.
**Fix / oracle:** grep `pub unsafe extern "C" fn (Rtl|Cm|Db|Hal)\w*Printf[AW]\b` and any one-liner forwarding to a non-variadic helper with no format substitution. Fix with a tiny C shim TU delegating to libcntpr `_vsnprintf` (calling variadics FROM Rust is stable), then remove the Rust stub leaving only a comment so a re-run doesn't re-emit it.
**Source:** `feedback_degraded_variadic_stubs.md`

### ZST-array pointer `.add()` — bindgen flexible member steps 0 bytes
**Symptom:** loop over `cfg.AccessRanges.add(i)` always reads element 0; downstream a counted allocation overruns (BUGCHECK 0x19 BAD_POOL_HEADER) or per-element logic misbehaves with no crash at the access itself.
**Root cause:** bindgen renders C flexible/pointer-to-array members as `*mut [T; 0usize]`; `.add(i)` advances `i * size_of::<[T; 0]>() = 0` bytes. The LLM writes `field.add(i)).cast()` which compiles clean.
**Fix / oracle:** cast FIRST: `(x.field as *mut T).add(i)`. Sweep: `grep -P 'pub (\w+): \*mut \[\w+; 0usize\]' types_*.rs` for the field names, then grep `.FIELD.add(`/`.FIELD.offset(` in modules. Caught in scsiport SpiConfigToResource (descriptor-per-range overrun of an exactly-sized CM_RESOURCE_LIST), 2026-06-12.
**Source:** `project_scsiport_rust_track.md`

### Cast-before-offset: `(ptr as *T).offset(1)` overshoots by the size delta → heap overrun
**Symptom:** an allocation `sizeof(HEADER) + sizeof(BIGPAYLOAD)` is sub-divided so a sub-object lands near the END of the block; its large trailing field (here a 16KB `Buffer`) overruns into reserved-but-uncommitted heap; a later memset/fill (`NtQueryDirectoryFile` dir-fill) crosses the committed-page boundary → unhandled AV → **i386 BUGCHECK 0x1E** (the DUMMY_PSEH `_SEH2_EXCEPT` is dead code, so nothing catches it). Misleads everyone upstream: looks like a heap/Mm/session bug (the faulting addr `0xf6…/0xf7…` was just the trapframe), not the caller. (kernel32 `FindFirstFileExW`, surfaced only once the wave advanced boot to winlogon dir-enumeration.)
**Root cause:** C `(T*)(p + 1)` advances `p` by `sizeof(*p)` (the pointee's type) and THEN casts to `T*`. The literal Rust translation `(p as *T).offset(1)` casts FIRST, so `.offset(1)` advances by `sizeof(T)`. When the cast type `T` differs in size from `*p` (handle-header vs big payload struct), it overshoots by the size delta and the sub-object/its fields land past where C put them.
**Fix / oracle:** offset on the ORIGINAL pointer, then cast: `p.offset(1) as *mut T` (mirrors `(T*)(p+1)`). Sweep heuristic (staticlibs): `grep -nP '\(\w+ as P?[A-Z_]+\)\.(offset|add)\('` — flag any cast-immediately-before-`.offset`/`.add` where the cast type ≠ the pointer's own allocation/pointee type. Sibling class to fastfat's `!Fn()!=0` and the ZST-`.add()` entry above. Worked example: kernel32-port `e4925a9d720` (cdfs root-cause; cold i386 verify → shell ready, 0 bugchecks).
**Source:** kernel32 wave (find.rs), 2026-06-16.

### Plain Rust-ABI `unsafe fn` address handed to the kernel (timer/dispatch)
**Symptom:** i386 BUGCHECK 0x1E with PC INSIDE pool data (PC ≈ a DeviceExtension address) shortly after a timer starts or a device opens; works until the kernel first CALLS the pointer.
**Root cause:** fn defined `unsafe fn Foo(...)` (Rust ABI) but its ADDRESS escapes via `Foo as *mut c_void`/transmute into IoInitializeTimer/MajorFunction/etc. The kernel calls it as NTAPI (stdcall on i386) → callee-pop mismatch → caller's stack walks into its own args/locals and `ret`s into data.
**Fix / oracle:** every fn whose address escapes must be `extern "system"` (or the exact per-arch ABI). Sweep: list plain `unsafe fn` names, flag any appearing in `as *mut`/`as *const`/`transmute(`. Caught in scsiport (ScsiPortIoTimer = IO_TIMER_ROUTINE, ScsiPortCreateClose = DRIVER_DISPATCH), 2026-06-12.
**Source:** `project_scsiport_rust_track.md`

### Hand-rolled va_arg (`&last_fixed_param` walk) — amd64-only garbage
**Symptom:** variadic export works on i386 but reads garbage args on amd64/arm64 (scsiport: "Invalid PathId: 160" then hang — RequestComplete read a junk Srb pointer).
**Root cause:** the LLM "simulates va_start" by taking the address of the last fixed parameter and walking upward — valid only where all params live contiguously on the stack (i386 cdecl). On win64 fixed params are in registers; `&param` is a local spill slot.
**Fix / oracle:** real c_variadic: declare `mut args: ...` and read with `args.arg::<T>()` (pointers/i32/u32 are VaArgSafe). Sweep: grep `Simulate va_start|last_param_ptr|& *\w+ as \*const \*mut` inside variadic fns.
**Source:** `project_scsiport_rust_track.md`

### C `DPRINT` (NDEBUG-dead) translated as live `DbgPrint` — boot slowdown masquerading as a hang
**Symptom:** smoke HANG_TIMEOUT but the system is alive; EIP sampling lands in DbgPrint plumbing (`so_wstring`, `__DbgPortRead`, memcpy). Per-IO prints throttle boot below the smoke timeout.
**Root cause:** C's `#define NDEBUG` compiles DPRINT out entirely; the translation renders them as unconditional DbgPrint.
**Fix / oracle:** route prints whose format string matches a C `DPRINT(` (NOT `DPRINT1`) to a c_variadic no-op sink (`pub unsafe extern "C" fn dprint_noop(_f: *const c_char, mut _a: ...) -> i32 { 0 }`); keep DPRINT1-derived ones (they carry the `(%s:%d)` file/line prefix). Script it: harvest C DPRINT format strings, match Rust byte-literals, rename the callee.
**Source:** `project_scsiport_rust_track.md`

### #ifdef MACRO mapped to #[cfg(MACRO)] — always the wrong branch
**Symptom:** compiles and links clean; pure-runtime bug, often arch-asymmetric. winsta.c: `#if !defined(_USE_DUMMY_PSEH)` → `#[cfg(not(_USE_DUMMY_PSEH))]` (always TRUE) ran KeUserModeCallbacks on the SYSTEM desktop thread → i386-only hang at comctl32 InitializeClasses, flooding "Sending to Suspended Thread Msg 121". amd64/amd64-green did NOT exonerate it.
**Root cause:** Rust `#[cfg(X)]` knows only cargo/rustc cfg flags, NOT C `#define`s. A C-macro name inside `#[cfg]` is always its unconfigured value → the branch C compiled OUT always compiles IN.
**Fix / oracle:** grep `#[cfg(` for any NON-cargo token (not `target_arch`/`target_pointer_width`/`feature = `). A C-macro name there is a translation bug. Resolve to the value the project actually builds: `_USE_DUMMY_PSEH` is defined on all arches → delete the `!defined` branch, keep the `#else`.
**Source:** `feedback_c_ifdef_as_rust_cfg_always_true.md`

### Flip omits a C `#define` redirect/HACK → compiles + serial-greens but renders wrong
**Symptom:** OS boots to the explorer marker (serial smoke "✓ boot") but the screen renders WRONG — black desktop + a 40×30 px (= 640/16 × 480/16) fragment of desktop color. dclife.c defines `gmxWorldToDeviceDefault` as a 16:1 matrix then immediately `#define gmxWorldToDeviceDefault gmxWorldToPageDefault` (identity); the flip faithfully copied the 16:1 static *definition* but lost the `#define`, so every DC inited `mxWorldToDevice = 16×` → DPtoLP/clip box shrank to screen/16.
**Root cause:** A C `#define A B` HACK that redirects every *use* of a name to a different value. The translator copies the visible definitions but drops the redirect, so the Rust use-sites read the un-redirected (wrong) value. Links clean, compiles clean, serial-greens — it's a pixels-only defect.
**Fix / oracle:** mirror the `#define` at every Rust use-site (init the field from the redirect TARGET, e.g. `gmxWorldToPageDefault`). Grep the C for `#define <Ident> <OtherIdent>` near static definitions; verify the flip honors the redirect. **Debugging rule: serial marker ≠ rendered desktop.** When the screen is wrong but boot is green: instrument with raw `DbgPrint` and read `qemu/<arch>_smoke_live.log`, NOT the filtered `full.log` (full.log truncates — cost two false "function not called" conclusions). The screenshot (`qemu/<arch>_smoke.png`) is the check — give it time to settle (taskbar/tray render ~15s after the marker; a 0s-on-marker capture shows a bare desktop).
**Source:** `project_gdi_desktop_not_rendering.md`

### Arch-cfg (#ifdef _M_ARM64) branches flattened to the desktop path
**Symptom:** compiles ×3, boots i386/amd64; only arm64 breaks. ldrpe.rs: flattened "tolerate missing imports" branches always raised → arm64 BUGCHECK 0x4C on csrss import snap. ldrutils.rs: arm64 raised STATUS_DLL_NOT_FOUND → explorer never reached `_tWinMain` (silent HANG, not a bugcheck).
**Root cause:** rustconv collapses `#ifdef _M_ARM64` to the first/`#else` desktop branch, dropping arm64-specific "tolerate missing export/import" behavior.
**Fix / oracle:** grep the original C for `#ifdef _M_ARM64` / `defined(_M_ARM64)` and verify each arm64 branch survived as a `cfg!(target_arch="aarch64")` guard (use `cfg!()` not `#[cfg]` so both branches compile; dead branch folds to a const off-arm64 → no re-smoke there). Smoke all 3 arches; a flattened tolerance branch may present as a silent HANG (process never started), so check arm64 reaches the explorer marker, not just `shell ready`.
**Source:** `feedback_rustconv_drops_arch_cfg_branches.md`

### Integration: subsystem-private header types undefined in output
**Symptom:** a clean per-function translation that won't compile — undefined header structs/constants/globals/CRT fns. Some defects only surface at the .dll link (not `cargo check`): a missed header global, an inlined C macro the LLM made an extern.
**Root cause:** rustconv's preamble captures only the .c file's own top-of-file; subsystem-private header types (vdm.h VDM_* records/consts) aren't in `*-sys` bindgen. `translate`'s stitcher never reads `scan.globals_text` — typedefs/structs/`#define`-consts/lookup tables are dropped entirely.
**Fix / oracle:** build a layout-exact shared types module (`_NAME` tag + `NAME`/`PNAME` aliases; `use ntdll_sys::nt::*` for primitives), reconcile any earlier truncated mirror (offset disagreement footgun). Systematic cast tails: SIZE_T args `as _` not `as u32` (u64 on 64-bit); flat `_LUID`; CSR_API handlers come out `extern "system"`, cdecl exports promote to `#[no_mangle] pub extern "C"`. The working multi-stage workflow: keep `translate` bodies, run one `submit --assemble` to a scratch path, manually splice its preamble over the broken import block. `cargo check` ×3 (64-bit surfaces SIZE_T) THEN link the .dll (surfaces dropped-global/macro-extern) THEN smoke ×3.
**Source:** `feedback_rustconv_header_types_integration.md`, `feedback_rustconv_translate_gotchas.md`

### Do NOT delete _SEH2_TRY "dead-code" wrappers — they're an LLVM barrier
**Symptom:** bulk-deleting Pass-0 (d) BELT_SUSPENDERS `_SEH2_TRY` wrappers around bare user writes regresses i386 keyboard delivery: `test_post_shell_ready_input` oracle stops firing (worst in hook.split2.c hot path, writes straddling KeUserModeCallback).
**Root cause:** under DUMMY_PSEH the EXCEPT body is dead at runtime, but the macro expands an `extern volatile int _SEH2_Volatile0` read that is a load-bearing compiler memory barrier suppressing LLVM captures(none)/hoist on the TRY body. Remove it → user writes hoist across foreign-call boundaries.
**Fix / oracle:** "dead under DUMMY_PSEH" is NOT a free pass to delete the wrapper. Keep wrappers until callers get explicit `read_volatile`/`write_volatile` surgery (or translate the consumer first applying volatile, then delete the C wrapper). Any "delete after cross-check" audit item must include a post-edit i386 `test_post_shell_ready_input` smoke — the win32k message path is the canonical sentinel. Tool `rustconv seh-belt` retained but NOT `--apply`'d.
**Source:** `feedback_seh_belt_llvm_barrier.md`

### Invocation footguns that silently corrupt or misroute output
**Symptom:** "command not found" / ModuleNotFoundError; "file not found"; translations land in MAIN repo from a worktree; MAIN's lib.rs loses ~20 `pub mod X;` lines; a junk `--fn` run leaves the C un-compiled.
**Root cause:** bare `python`/system `python3` lack tree_sitter_c+anthropic (use `<repo>/.venv/bin/python`); `resolve_c_path` checks cwd=repo-root (pass ABSOLUTE C path); `config.py` `parents[3]` resolves to MAIN when rustconv was pip-installed editable from main (set `PYTHONPATH=<worktree>/tools/rustconv`); `translate` auto-mutates 3 places (output .rs + `pub mod` in mod.rs + cmake comment-out) even with `--fn`. `batch` is NOT the 50%-off batch API.
**Fix / oracle:** `python -c "import rustconv; print(rustconv.__file__)"` to confirm which copy loads. `.env` is at the MAIN checkout. Cleanup of a junk run must revert all 3 mutations (rm .rs is not enough — re-uncomment the cmake `.c` line, remove `pub mod` watching for stolen cfg attrs, drop the DEPENDS line). `git diff lib.rs` BEFORE rebuilding to catch assembler corruption. `opus` alias = `claude-opus-4-8`.
**Source:** `feedback_rustconv_invocation_gotchas.md`, `feedback_rustconv_translate_gotchas.md`

---

### Undefined const in a `match` arm → silent irrefutable catch-all

A `match` arm that names a constant the translation **forgot to define** does not fail to
compile — Rust parses the bare uppercase identifier as a **new binding pattern** (a
catch-all that binds the scrutinee to that name), so **that arm matches everything** and
every later arm is unreachable. The logic silently takes the "wrong" branch for all
inputs; you get an `unreachable-pattern` *warning*, never an error.

- **Real case:** msafd's status dispatch matched `STATUS_RECEIVE_PARTIAL`, which was never
  defined → the arm became a catch-all → **every** recv completion was reported as
  `MSG_PARTIAL` → ws2_32 returned `WSAEMSGSIZE` on all reads. (Same class hid
  `STATUS_FILE_CLOSED`/`FIONBIO`.)
- **Cause:** rustconv drops a `#define`/const the C `switch` relied on; the C `case FOO:`
  becomes a Rust binding `FOO =>` instead of a value match.
- **Fix / grep:** treat every `unreachable-pattern` warning as a *possible* undefined-const
  bug, not lint noise. Scan match arms for bare `SCREAMING_CASE` identifiers that resolve to
  a **binding**, not a path — `git grep -nE '^\s*[A-Z][A-Z0-9_]+\s*=>'` and confirm each name
  is a real `const`/imported path. A value match must be `path::FOO =>` or a guarded
  `x if x == FOO =>`.
- **The deletion variant (2026-07-06):** you can *create* this bug without touching the
  match — **deleting a `pub const` out from under a `use module::*` glob** turns every match
  arm naming it, possibly in a different file, into a catch-all. Real case: a const-dedup
  edit removed `IRP_MN_QUERY_DEVICE_TEXT`/`IRP_MN_QUERY_ID` from scsiport-rust `glue.rs`;
  `pdo.rs`'s PnP dispatch then swallowed **every unhandled minor code** into
  `PdoHandleQueryDeviceText`, so `IRP_MN_QUERY_RESOURCES` returned "success" with a *text
  string* as the resource list → PnP stored it as `BootResources` → deterministic BUGCHECK
  0x1E walking text as CM descriptors (i386 smoke; amd64 walked the same garbage without
  faulting — arch greenness is luck, not proof). Rule: after ANY const removal/rename,
  re-run `cargo check` on every crate that globs the module and grep `unreachable pattern`
  (`tools/scan_bind_all_matches.py` is the workspace-wide sweep).

### `write_bytes` typed-pointer element-count confusion → sizeof² memset

`core::ptr::write_bytes::<T>(dst, val, count)` fills `count * size_of::<T>()` BYTES —
`count` is in **elements of T**, not bytes. rustconv translates
`RtlZeroMemory(ptr, sizeof(X))` to `write_bytes(ptr_as_mut_X, 0, size_of::<X>())`,
which zeroes `sizeof(X)²` bytes. For small structs that's a silent overrun of
neighboring fields; for big ones it's a wild multi-GB memset.

- **Real case (2026-07-06 usbstor bring-up):** classpnp `ClassPnpStartDevice` zeroed
  the freshly-allocated 0x15470-byte `CLASS_PRIVATE_FDO_DATA` with a
  **0x1C4B9B100-byte (7.6 GB) memset** → ran off the pool block into unmapped VA →
  page-fault whose handling re-faulted → recursive `MmAccessFault` →
  `MiLockProcessWorkingSetShared` cycle → stack exhausted → **BUGCHECK 0x7F (double
  fault)** ~47KB of recursion frames deep. First-ever execution: no prior boot ran a
  disk-class AddDevice/START (LiveCD boots from CD+ramdisk) — never-executed-Rust
  landmine. Found via the 0x7F banner's tfstack + FIRST-FAULT KTRAP_FRAME dump
  (Rip=memset, FaultAddress page-aligned = overrun signature), then disasm of the
  call site showed `mov edx, 0x15470` (alloc) vs `movabs r8, 0x1C4B9B100` (memset).
- **Fix idiom:** cast the destination: `write_bytes(ptr as *mut u8, 0, size_of::<X>())`
  (byte elements → count is bytes). ~20 sites fixed workspace-wide same day
  (classpnp ×8, disk ×2, fastfat ×5, kernel32 res.rs ×7, ks ×2, mountmgr, pcix,
  mscms, freeldr).
- **Grep:** multi-line-aware scan for `write_bytes(` whose last arg contains
  `size_of` and whose first arg is NOT cast to `*mut u8`/`c_void`/`PVOID`.
  A `size_of::<T>() / size_of::<T>()` (=1) count with typed dst is the other
  correct spelling — don't "fix" it.

### `size_of_val` on a `__BindgenUnionField` marker → size 0

C `sizeof(x->UnionMember)` naively becomes `size_of_val(&(*x).UnionMember)` — but in
bindgen's **old-style** union layout (`#[repr(C)] struct` with `__BindgenUnionField<T>`
markers + one real `bindgen_union_field` storage array), each member field is a **ZST
marker**, so `size_of_val` returns **0**. The 0 then flows somewhere size-sensitive and
fails far away with a *parameter*-flavored status, not a crash.

- **Symptom (found 2026-07-06):** USB disk PnP START failed 0xC000000D; classpnp's
  `InitializeStorageRequestBlock` rejected `ByteSize=0` from
  `size_of_val(&PrivateFdoData.ReleaseQueueSrb.ReleaseQueueSrbBuffer)` (a
  `__BindgenUnionField<[UCHAR;144]>`). Sweep found 2 more live sites:
  classpnp `class.rs` SrbExBuffer (same call, power path) and partmgr
  `partition.rs` `sizeof(basicVolId->Mbr/Gpt)` → `UniqueIdLength=0` (breaks
  MOUNTDEV_QUERY_UNIQUE_ID / mountmgr bookkeeping). The partmgr site's comment even
  said "union field size, NOT a type" — knowing the C intent doesn't save you.
- **Fix:** size the **target type** (`size_of::<_BASIC_VOLUME_UNIQUE_ID__bindgen_ty_1>()`)
  or the **whole union struct** when the C took the largest member's size. Never
  `size_of_val` a bindgen union member.
- **Sweep recipe:** collect `pub X: __BindgenUnionField` field names per crate, then flag
  `size_of_val` callsites whose last `.Field)` token matches one (nested-paren tolerant).
  Note only *some* crates use old-style unions — verify the hit against the same crate's
  `types_*.rs` before "fixing" (new-style real `union` members are fine).
- **Related latent hazard:** `write_bytes(ptr_to_union_field, 0, size_of_val(...))` and
  friends — any `size_of_val` in a bindgen-heavy crate deserves a second look.

### Dropped term in a hand-derived `sizeof` `#define` → extension-layout overlap

C `#define FUNCTIONAL_EXTENSION_SIZE sizeof(FUNCTIONAL_DEVICE_EXTENSION) + sizeof(DISK_DATA)`
was hand-translated to `size_of::<FUNCTIONAL_DEVICE_EXTENSION>()` — the **second sizeof
dropped**. disk.sys registered a device extension with no room for DISK_DATA; classpnp
laid its PrivateCommonData + remove-lock rundown right after the FDE; disk's
`DISK_DATA.Mbr.{Signature,CheckSum}` write then corrupted the rundown's `RunRefs`.

- **Symptom (2026-07-06, USB-disk START):** AV in `ExfAcquireRundownProtectionCacheAware`
  minutes of call-chain away from the corrupting store. The kill shot: the faulting
  register held `0x<CheckSum><Signature>` — **the disk's literal MBR identifier**
  (`sfdisk` had printed `Disk identifier: 0x0a2da154`). When a "pointer" faults with
  recognizable DATA in it, ask *whose data is that* — it names the overwriter.
- **Recipe that found it:** serial 0x1E banner → `dump-trapframe <P3>` on the parked
  GDB run → recognize Rax as the disk signature → grep who writes `Mbr.Signature` →
  compare the C `#define` against the Rust const.
- **Rule:** any `glue.rs` "sizeof-based #define" must quote the FULL C macro text in a
  comment next to the Rust expression (this file's convention) — the reviewable unit is
  the C-to-Rust pair, not the Rust alone. When debugging extension-layout corruption
  between a class driver and classpnp, diff `DeviceExtensionSize` registration against
  the C macro FIRST — it's one line and this class lives there.

### BOOLEAN return declared `-> i32` in an extern block → garbage-upper-bytes boolean

On x86/x64 a function returning `BOOLEAN`/`u8` only defines **AL**; the upper 3 bytes of
EAX are whatever garbage the callee left there. A Rust extern decl that widens the return
(`fn FatDeviceIsFatFsdo(...) -> i32;` for a C/Rust definition returning `BOOLEAN`) makes
every `!= 0` test read those garbage bytes — the call **randomly returns true** even when
the function returned FALSE. The reverse direction (definition i32, decl u8) is safe for
0/non-0 semantics (truncation to AL preserves the boolean).

- **Symptom (2026-07-06, USB-disk mount):** fastfat `FatCreateIrpContext` raised
  `STATUS_INVALID_DEVICE_REQUEST` (→ BUGCHECK 0x1E under DUMMY_PSEH) on its own healthy
  volume-file read, because `FatDeviceIsFatFsdo` (declared `-> i32`, defined BOOLEAN)
  "matched" the volume device object as an FSDO. 55 decls in fastfat-rust alone; the
  workspace sweep found the same class live in mountmgr (`IsDriveLetter`/`HasDriveLetter`
  — drive-letter assignment!), win32k (`CLIPOBJ_bEnum`), amd64 kiinit (`KdPollBreakIn`),
  and driver crates declaring kernel BOOLEAN exports (`CcPurgeCacheSection`,
  `MmFlushImageSection`, `KeRemoveQueueDpc`) as i32.
- **Debug tell:** a branch that "can't be taken" fires anyway, non-deterministically, and
  the guard function is an extern-declared boolean. Print the operands: if the comparison
  inputs are visibly unequal yet the branch fired, suspect return-width garbage.
- **Sibling semantics bug the scan also catches:** BOOLEAN definition declared
  `-> NTSTATUS` and used as `if !NT_SUCCESS(x)` — TRUE(1) and FALSE(0) are both `>= 0`,
  so the failure path is dead code (`HvSyncHive` in cmlazy.rs; `RtlCreateUnicodeString`
  in scsiport-rust).
- **Detector:** `python3 tools/scan_bool_width.py` — cross-references every extern decl
  returning i32/u32/LONG/ULONG/NTSTATUS against the workspace's u8/BOOLEAN definitions
  (Rust `#[no_mangle]` + C cohab/header prototypes). Suppress verified FPs with
  `// nolint: bool-width (reason)`.
- **Fix pattern:** decl → `-> u8`, then let cargo E0308s name the call sites; append
  `as i32`/`as u32` where the result feeds a wide variable. For NTSTATUS-confused sites,
  fix the *semantics* (`if X(...) == 0 { fail }`), not just the type.

### `_Analysis_assume_` translated as live code → NULL deref the annotation "assumed away"

C's SAL annotations (`_Analysis_assume_`, `_Analysis_assume_lock_held_`, …) are compile-time
no-ops — their argument expression is **never evaluated**. A translation that turns them into
a Rust no-op *function call* still evaluates the argument at the call site. The argument is
routinely a pointer-deref guarded the WRONG way for runtime: fastfat's
`_Analysis_assume_(NonPagedFcb.is_null() && (*NonPagedFcb).OutstandingAsyncWrites != 1)`
dereferences the FCB **exactly when it is NULL** (the `&&` short-circuit runs the deref on
the is_null()==true side).

- **Symptom (2026-07-06, USB-disk mount):** BUGCHECK **0xB8 ATTEMPTED_SWITCH_FROM_DPC** —
  the NULL-page deref happened inside `FatSingleAsyncCompletionRoutine` (DISPATCH_LEVEL DPC),
  MmAccessFault took the process WS pushlock → `KeWaitForGate` → thread swap inside a DPC.
  The wait/swap frames are the messenger; the crime is the pageable/NULL touch at DISPATCH.
- **Recipe that found it:** 0xB8 has no auto-drill — `run-until --break KeBugCheckEx`, then
  `dump-thread <P1>` for the KernelStack, raw-dump the stack, symbolicate return addresses
  in ASCENDING address order (lowest = most recent): MmAccessFault sat directly above the
  completion routine's frame; the routine's line number landed inside the
  `_Analysis_assume_` block.
- **Fix/sweep:** delete every `_Analysis_assume_*(...)` *statement* (19 sites removed
  2026-07-06 across fastfat/cdfs/classpnp/cdrom/win32k — classpnp's
  `(*(*Irp).MdlAddress).ByteCount` variant was a live landmine for non-MDL IRPs). Grep:
  `grep -rn "_Analysis_assume_" sdk/lib/rust --include=*.rs | grep -v "^\s*//"`. rustconv
  should translate SAL annotations to nothing (or a comment), never to a call.

### Dropped `FIELD_OFFSET(T, Array[N])` array-subscript scaling → heap under-alloc

When a C size/offset uses an **array-subscripted** field — `FIELD_OFFSET(T, Addr.Address[N])`
or `... + Addr.Address[n]`-style arithmetic — the `[N]` contributes `N * sizeof(elem)` to the
computed size. A translation that keeps the field but **drops the subscript scaling**
under-computes the allocation → the subsequent write overruns the block → **heap
corruption** that manifests far away (a later `HeapAlloc`/free wedges or BUGCHECKs).

- **Real case:** msafd `WSPBind` computed `connectSize` from `Address.Address[N]` but lost the
  array-subscript scaling → a **2-byte heap overrun on every bind** → corrupted process heap
  → a later `HeapAlloc` hung.
- **Fix / grep:** any size derived from a `FIELD_OFFSET`/pointer-arith over an array member
  must scale by the element count. Grep translated size math for a struct array field whose
  `[N]` term vanished vs the C: compare each `*_size`/`connectSize`/`FIELD_OFFSET` against the
  pristine C expression, element by element.

## Per-arch crash patterns (arm64 + amd64-only)

Deterministic crashes that only fire on 64-bit (or only amd64, or only arm64) where i386's narrower pointers, lack of SSE, or different struct layout luck-mask a real bug.

### UEFI memory-map retry never converges on arm64 (UINTN hardcoded u32)
**Symptom:** arm64 freeldr hangs — `PUEFI_LoadMemoryMap`'s retry loop never converges, UEFI keeps returning `EFI_BUFFER_TOO_SMALL` with the same required size. amd64 "worked" by luck.
**Root cause:** `freeldr-rust` hardcoded `pub type UINTN = u32`. UEFI defines UINTN as pointer-width = UINT64 on amd64/arm64. UEFI reads/writes 8 bytes through a 4-byte Rust stack slot → reads adjacent garbage, write overflows into the next stack var (MapKey/DescriptorSize). amd64 survived because high 32 bits happened to be zero.
**Fix:** `#[cfg(target_pointer_width="64")] type UINTN=u64;` / `="32"` → u32. Add `as UINTN` casts at ULONG call sites (BufferSize, NumberOfPages, FrameBufferSize). Commit `50043b4b9b4`.
**Source:** `feedback_uefi_uintn_pointer_width.md`

### arm64 user-mode (explorer/win32k) never gets CPU — pegged in KeStallExecutionProcessor
**Symptom:** arm64 only; explorer/win32k never run, gdbstub shows PC at HAL `KeStallExecutionProcessor` 4/4 samples over 30s. Desktop never appears.
**Root cause:** arm64 has no GIC (`KiIrqHandler` is a return-stub) → no timer interrupts → no preemption. A kernel-side polling loop (csrss `SerialInputThread` → `ReadFile` COM1 → `Pl011Read`'s 5×1ms busy-wait) pegs the whole CPU; even a short stall burst blocks all threads.
**Fix:** never call `KeStallExecutionProcessor` from arm64 driver IRP handlers. Make the kernel path non-blocking (return 0 bytes when FIFO empty); let user mode poll via `Sleep` (works because the idle loop drains `Prcb->ReadySummary` directly). Edit `drivers/serial/pl011/pl011.c`.
**Source:** `feedback_arm64_pl011_busy_wait.md`

### arm64-only ACCESS_VIOLATION on every demand-zero fault → SMSS 0x6A
**Symptom:** arm64 only; `MmArmAccessFault` returns STATUS_ACCESS_VIOLATION for every demand-zero user page fault → SMSS dies BUGCHECK 0x6A on first stack write.
**Root cause:** `MMPTE_SOFTWARE.Protection` is at a different bit per arch: amd64/i386 = bit 5, arm64 = bit 1 (arm64 layout is `Valid:1, Protection:5, PageFileLow:4`). Rust accessors hardcoded `<< 5`. C writes PTE.Long=8 (Protection at bit 1), Rust check expects `MM_READWRITE << 5 = 0x80`. (Upstream `MM_PTE_SOFTWARE_PROTECTION_BITS=6` is also wrong — latent, only Rust reaches the check.)
**Fix:** route through `soft_protection()`/`set_soft_protection()` (now `SOFT_PROTECTION_SHIFT=1` on arm64). Files: `ntkrnl-sys/src/mm.rs` (arch-cfg accessors) + `ntoskrnl-rust/src/ARM3/pagfault.rs`.
**Source:** `feedback_arm64_soft_pte_layout.md`

### Editing the wrong arm64 trap handler — trap.c is dead code
**Symptom:** arm64 trap-handler edits (synchronous exceptions, MmAccessFault) have no effect; your literal string isn't in the binary even after rebuild.
**Root cause:** `ntoskrnl/ke/arm64/trap.c` is commented out in `ntos.cmake` (~line 435); the live handler is `ntoskrnl-rust/src/ke/arm64/trap.rs`. Same for arm64 stubs.c/kiinit.c/thrdini.c/usercall.c. The active version can flip between sessions.
**Fix:** edit the `.rs`. `touch` won't rebuild — `rm` the rust target output or `cargo build` directly. Verify with `strings build-arm64/ntoskrnl/ntoskrnl.exe | grep <literal>`. Always `grep -n "ke/arm64/" ntoskrnl/ntos.cmake` first.
**Source:** `feedback_arm64_trap_dead_code.md`

### arm64 Rust panic looks like an opaque boot hang (silent panic handler + slice-OOB)
**Symptom:** arm64 freeldr "hangs" with frozen serial + spinning CPU, indistinguishable from an infinite loop. Misdiagnosed as a "TCG-slow loop" (the 910 iter/sec was a per-iteration UART-print artifact).
**Root cause:** (1) `freeldr-rust` `#[panic_handler]` printed only under `cfg(any(x86,x86_64))`; on aarch64 it fell into `loop{wfe}` with zero output. (2) The hidden panic was a Rust slice-index OOB — `MemTypeDesc[MemoryType as usize]` where C silently reads adjacent garbage; QEMU virt's page table spans the MMIO hole below 0x40000000 RAM, yielding `MemoryType >= len`. Fires even when DbgPrint is disabled (Rust still bounds-checks the index expr).
**Fix:** make the arm64 panic handler emit file:line via PL011 DR @ 0x09000000 before spinning. Guard the index to match C's tolerate-garbage: `if i < MemTypeDesc.len() {…} else {b"Unknown\0"…}`. Audit rustconv output for `[…as usize]` slice indexing of device/enum fields. Commit `6662c9a4c5c`.

### arm64 "idle hang" that is really a kernel Rust panic — sample $pc, it's `rust_begin_unwind`
**Symptom:** arm64 boot freezes; serial stops, screen black, `info registers` shows `$pc` frozen at ONE kernel address across many samples → looks exactly like the idle WFI / a blocked wait. Reached deep (csrss/winsrv win32 init) after other fixes let boot advance.
**Root cause #1 (the tell):** the frozen PC was `__rustc::rust_begin_unwind` — a kernel **Rust panic**, not idle. The panic handler spins. A proper backtrace (`add-symbol-file ntoskrnl.exe -o <kbase - 0x400000>`, then `bt`) showed `panic_bounds_check` ← `KiDispatchException`. Read the `core::panic::Location` (the `x2`/3rd arg to `panic_bounds_check`: `file_ptr,len,line`) to get file:line for free.
**Root cause #2 (the actual bug):** `KiDispatchException`'s kernel-mode user-AV fixup (zero the faulting load's dest reg, skip the insn — the DUMMY_PSEH safe-probe substitute) wrote `TrapFrame.X[Rd]` for `Rd < 29`. But the arm64 **KTRAP_FRAME only holds X0..X18 (19 entries)**; X19..X28 are in the **KEXCEPTION_FRAME** and X29/X30 are `Fp`/`Lr` (see `KeTrapFrameToContext`, which already splits them). A faulting load whose dest is X19..X28 indexed `X[19..28]` OOB → `panic_bounds_check(len 19)`.
**Fix:** route the zeroed reg by range — X0..X18→`TrapFrame.X[]`, X19..X28→`KEXCEPTION_FRAME` (guard non-null), X29/X30→`Fp`/`Lr`, X31(xzr) no-op; apply to both the dest (Rd) and the STP second reg (Rt2). `ke/arm64/stubs.rs`, commit `23406d8c708`. **General tell:** any arm64 code that treats `TrapFrame.X[]` as a full 31-reg file is wrong — only X0..X18 live there.
**Source:** `project_fastfat_rust_track.md` (2026-06-15)
**Source:** `feedback_arm64_silent_panic_slice_oob.md`

### arm64-only deadlock from UB: `&immutable_local as *const T as *mut T` on an in/out FFI param (Heisenbug-via-instrumentation)
**Symptom:** arm64 only; csrss hangs forever inside `LdrLoadDll(winsrv.dll)` at kernel32 `ConnectConsole` → `NtWaitForMultipleObjects(MAX_INIT_EVENTS, ConsoleStartInfo.InitEvents, …)`. CPU idle (PC pinned at `KiIdleLoop`), thread blocked off-CPU on a dispatcher wait — a pure deadlock, no bugcheck/panic. i386+amd64 green (2/3). **The tell: adding an entry `DbgPrint` to the function makes it boot clean** — a Heisenbug, which means a UB-miscompile (instrumentation perturbs codegen and masks it), not a logic/race bug. (Bonus red herring: a non-fatal `KeSignalGateBoostPriority: BAD gate … caller=00000000` DbgPrint line shows up in PASSING runs too — ignore it.)
**Root cause:** `ConnectConsole` passed the IN/OUT size to `CsrClientConnectToServer` as `&ConnectInfoSize as *const ULONG as *mut ULONG`, where `ConnectInfoSize` was an immutable `let`. The callee writes the actual size back through it → **writing through a `*const`-derived pointer to an immutable local = UB.** arm64 gnullvm LLVM exploited the UB and miscompiled the *post-call reads of the same struct*: `ConnectInfo->ConsoleStartInfo.InitEvents` loaded STALE (the pre-connect zeroed values) → `NtWaitForMultipleObjects` waited on null handles consrv never signals → deadlock. i386/amd64 (gnu) codegen happened not to bite.
**Fix:** `let mut ConnectInfoSize` + `&mut ConnectInfoSize as *mut ULONG` (matches the C `ULONG ConnectInfoSize; … &ConnectInfoSize`). Commit `cc13aded8fd`.
**Grep:** `&\w+ as \*const .* as \*mut` — flags `&` of a local cast const→mut for an FFI out/in-out param. NOT the same as `ptr as *const _ as *mut _` round-trips on a caller-provided buffer pointer (those are fine); the UB is specifically taking `&`/`&mut`... no — taking `&` (shared) of an *immutable local* and writing through it. The c2rust output drops the `mut` on locals whose address is only taken once.
**Debugging technique that cracked it (arm64 has ZERO kernel DWARF → thread-walk fails):** instrument the suspect functions with `DbgPrint("[TAG] <fn>")` at entry, build the user DLL for arm64 + inject into ramdisk + boot, read the last serial line before the freeze. Here the instrument *masked* the bug — which was itself the decisive Heisenbug clue.
**Source:** `project_kernel32_nls_rust_track.md` (2026-06-19 cont.5)

### arm64 0xF4 winlogon death right after `[WIN32K] shell ready`
**Symptom:** arm64 only; deterministic BUGCHECK 0xF4 (critical-process death, P1=3, P2=winlogon EPROCESS) immediately after `shell ready` on the logoff KeUserModeCallback. amd64 immune.
**Root cause:** PEB is NULL in the arm64 user-callback context. ntdll `KiUserCallbackDispatcher` reads TEB->PEB via `[x18+0x60]` and gets NULL → AV at NULL+0x58; exception dispatch re-faults in `RtlPcToFileHeader` reading `PEB->Ldr` at NULL+0x18 → recursion → stack overflow. amd64 reaches TEB via gs (reloaded every kernel→user entry); arm64 keeps TEB in x18 and the callback trap-frame copy didn't propagate the X[18] slot. Probe: `cbtf.X18=0xFFFFF6FB40000000` (kernel garbage) vs correct `0x00007FFFFFFED000`.
**Fix (usercall.rs):** replace the union-struct `*dst=*src` with explicit `core::ptr::copy_nonoverlapping(src,dst,size_of::<KTRAP_FRAME>())`, then force `(*CallbackTrapFrame).__bindgen_anon_3.X[18] = (*CurrentThread).Teb as u64`. Fixed 2026-06-04, landed crp `ef8e5d2921c`. (X18 at TfX18=0x140 in the 0x160-byte frame.)
**Source:** `feedback_arm64_winlogon_0xf4_callback_peb.md`

### arm64 win32k flips boot "green" on stale binaries — harness skips the link
**Symptom:** arm64 win32k C→Rust flip passes smoke without ever running; latent arm64-only link breaks pile up across waves invisibly.
**Root cause:** `./vibeos-cli build --arch arm64` runs only `ninja ntoskrnl hal uefildr` — it does NOT build win32k.sys (nor user-mode), so smoke boots a stale win32k.sys. i386/amd64 link gates take the other cfg branch; an aarch64 branch calling a symbol-less bindgen extern (FORCEINLINE/intrinsic, e.g. `ddk::NtCurrentTeb`) links fine on x86 but is `undefined symbol` on arm64.
**Fix:** per flip, manually `cd build-arm64 && ninja win32k`, mcopy fresh win32k.sys into ramdisk.img + boot.img, then `smoke --arch arm64 --allow-stale`. Prefer `ntkrnl_sys::inline::ke::*` over `ddk::*` for arm64 intrinsics (read TEB via `asm!("mov {}, x18")`).
**Source:** `feedback_arm64_harness_skips_win32k_link.md`

### Same trap for user-mode DLLs (kernel32) — AND it hides arm64 COMPILE errors (kernel32, 2026-06-15)
**Symptom:** A kernel32-rust fix builds + smokes green on i386; `vibeos-cli build --arch arm64` reports WARN/green; arm64 smoke shows the **exact same BUGCHECK with identical params** as before the fix. The repeated-identical-bugcheck is the tell: the new code never ran.
**Root cause:** `vibeos-cli build --arch arm64`'s staged set builds only `ninja ntoskrnl hal uefildr` — **not kernel32 (nor any user-mode DLL)**. So it (a) boots a STALE `kernel32.dll` (mkramdisk --refresh copies the old one), and worse (b) **never compiles kernel32-rust for aarch64, so arm64-only compile errors in your edit are silently invisible** — cargo-check on the workspace may also skip the per-DLL staticlib unless you target it. My `BaseInitializeContext` edit had 3 `E0425` (the 3 startup-thunk externs were `#[cfg(x86_64)]`-only, no aarch64 block) and I didn't see them until I forced `ninja rust_kernel32`.
**Fix / workflow:** to test a kernel32 (or any user-mode DLL) change on arm64: (1) `cd build-arm64 && ninja rust_kernel32 dll/win32/kernel32/kernel32.dll` — this is where arm64 compile errors actually surface; (2) `llvm-objdump -d build-arm64/.../kernel32.dll` and eyeball the changed fn IS your new code, not a stale stub; (3) `mcopy -o -i qemu/ramdisk.img <dll> ::/reactos/system32/<dll>` + `mcopy -o -i qemu/boot.img qemu/ramdisk.img ::/ramdisk.img`; (4) `smoke --arch arm64 --allow-stale`. Generalizes the win32k entry above: the arm64 staged set excludes everything except ntoskrnl/hal/uefildr.

### arm64 cfg-gated stub left empty by translation → NULL thread-start 0xF4 (kernel32 BaseInitializeContext, 2026-06-15)
**Symptom:** arm64 boots far (winlogon spawns explorer/notepad/cmd) then BUGCHECK 0xF4 (CRITICAL_OBJECT_TERMINATION, p1=0x3); serial shows `PspUserThreadStartup: StartRoutine=0x0, StartContext=0x0` for a thread of a critical process.
**Root cause:** the C source has a real `_M_ARM64` branch but the translation emitted a no-op stub (`{ let _ = (args); }`) "because there's no upstream ARM64 path" — a misread (utils.c DID have `#elif defined(_M_ARM64)`). `CreateRemoteThread` zeroes a CONTEXT then calls `BaseInitializeContext`; with the stub, Pc/X0 stay 0 → `NtCreateThread` starts the thread at PC=0 → StartRoutine=0 → 0xF4. (i386/amd64 fine — they have real cfg branches.)
**Fix:** translate the `_M_ARM64` branch (utils.c:648): X0=StartAddress, X1=Parameter, Sp=Stack&~15, Pc=BaseThreadStartup/Fiber/Process per ContextType, ContextFlags=CONTEXT_FULL. Watch for arm64 bindgen CONTEXT layout (X0/X1 in `__bindgen_anon_1.__bindgen_anon_1`; Sp/Pc flat) and add the aarch64 extern block for symbols an x86-only block was declaring. **Lesson:** grep translated crates for empty arch-cfg stubs (`#[cfg(target_arch = "aarch64")] ... { let _ = (` ) — each is a silent unimplemented function. Fixed `c06848a9df2`.

### arm64 RtlCreateAtomTable divide-by-zero panic (NumberOfBuckets==0)
**Symptom:** arm64 only; Rust panic at `rtl-strings/src/core_rs/atom.rs:58` (`hash % number_of_buckets` divide-by-zero) AFTER `_tWinMain`/desktop. i386/amd64 never hit it. Masked because arm64 idle-detection smoke passes on reaching desktop and a HALT also reads as idle.
**Root cause:** RTL_ATOM_TABLE init reads NumberOfBuckets==0 on arm64 only. Prime suspect: the nested `#[repr(C)]` RTL_ATOM_TABLE mirror's AtmDispatcherHeader→NumberOfBuckets/Buckets field OFFSET on arm64, or RtlCreateAtomTable's TableSize-clamp landing in the wrong field. CONFIRMED pre-existing (reproduces on crp HEAD without the heap wave). Earlier, a related arm64 ntoskrnl link bug (`RtlDosApplyFileIsolationRedirection_Ustr` from path.c GC-rooted into the kernel) masked every "green" arm64 baseline by booting a stale kernel — fixed by `#ifndef _NTOSKRNL_` guard in path.c (commit `216a4efd053`).
**Fix:** (per the updated memory) the atom panic was subsequently FIXED+LANDED (crp `9660e4a5c2e`) — an atom union-arm mismatch + stale media, not heap. Method note: arm64 idle-smoke masks post-desktop panics — always grep serial for `BUGCHECK`/panic and confirm `_tWinMain`; a fast (~24s) arm64 OK is suspect.
**Source:** `feedback_rtl_atom_arm64_boot_regression.md`

### amd64-only 0xc0000024 in win32k ObReferenceObjectByHandle (ExEventObjectType)
**Symptom:** win32k flip calling `ObReferenceObjectByHandle(…, ExEventObjectType, …)` boots green on i386+arm64 but on amd64 `NtUserInitialize failed Status 0xc0000024` (OBJECT_TYPE_MISMATCH) → csrss dies → cmlib `hivewrt` "sequences DO NOT MATCH" noise (red herring — that's the dying system). Hit on power.c flip wave 6.
**Root cause:** per-arch pointer depth for the ntoskrnl DATA export `ExEventObjectType`. i386/arm64 hand-declare raw-dylib `static mut: *mut _OBJECT_TYPE` (single POBJECT_TYPE → pass directly). amd64 gets it from the win32k-sys bindgen glob typed `*mut POBJECT_TYPE` (pointer-to-pointer, matching win32k.h's `_WIN32K_` decl) → must deref once. Don't "fix" by switching amd64 to a plain extern — under `--disable-auto-import` that builds a broken `.refptr` single-deref of an unpopulated slot.
**Fix:** per-arch helper — `#[cfg(any(x86,aarch64))] ExEventObjectType as PVOID` vs `#[cfg(x86_64)] *ExEventObjectType as PVOID`. Diagnostic: `llvm-objdump -d win32k.sys` — correct amd64 form does THREE loads ending `mov r8,[rax]` (`.refptr.__imp_…` → IAT → ptr-to-ptr).
**Source:** `feedback_exeventobjecttype_amd64_ptr_to_ptr.md`

### amd64-only win32k float bugcheck 0x1E / 0xC0000090 (MXCSR unmasked)
**Symptom:** amd64 only; BUGCHECK 0x1E exc 0xC0000090 (STATUS_FLOAT_INVALID_OPERATION) during eventlog.exe desktop init, in win32k `xformobj::FLOATOBJ_IsLong`/`FLOATOBJ_GetFix`. Live MXCSR = 0x4131. i386 immune (FLOATOBJ is fixed-point, no SSE); arm64 has no MXCSR.
**Root cause:** kernel ran threads with FP exceptions UNMASKED (0x4131 masks only DM; healthy = 0x1F80, all six masked). With invalid-op unmasked, `cvttss2si` (float→int) of a NaN/out-of-range raises #IA instead of returning indefinite — and Rust saturating casts emit a bare cvttss2si that traps before the saturation fixup. `KiSystemCall64` loads live `Prcb.MxCsr` (`gs:[PcMxCsr]`) which drifted from INITIAL_MXCSR 0x1F80; the prior `and eax,0xFFFF` workaround cleared reserved bits but not the mask bits.
**Fix (commit `44a1127fdd2`, ke/amd64/trap.S):** in syscall-entry sanitize, `and eax,0x9FFF` (clear reserved + rounding) then `or eax,0x1F80` (force all six FP masks) before `ldmxcsr`. If a similar 0xC00000{8D..93} reappears via KiTrapXX/interrupt entries, they need the same mask-forcing (only KiSystemCall64 was patched).
**Source:** `feedback_amd64_mxcsr_unmasked_syscall.md`

### Method: i386 hides 64-bit bugs
A green i386 smoke does NOT exonerate a flip. amd64/arm64-specific deterministic crashes are almost always REAL bugs that i386 luck-masks: pointer width (UINTN u32 vs u64), struct/bitfield layout and field offsets (soft-PTE Protection bit 1 vs 5, RTL_ATOM_TABLE NumberOfBuckets offset), padding, FP/SSE behavior absent on i386 (MXCSR FLOATOBJ), per-arch pointer depth in DATA externs (ExEventObjectType ptr-to-ptr), and platform-register plumbing absent on x86 (arm64 TEB in x18, no GIC/timers). An arch-gated failure (2/3 green) is proof of an arch-gated bug — never trust 2/3 green, build and boot all three (and for arm64 specifically, grep serial for BUGCHECK/panic since idle-detection smoke masks post-desktop deaths and HALTs read as idle).

---

## Cohabitation, cold-link & stale-artifact false failures

The false-red/false-green class: cohabitation flips, stale archives, and cross-DLL data refs that make cold links lie, warm builds mask regressions, and merges look like fresh bugs.

### Workspace-composition change → warm-incremental FALSE-FAIL of a cohab smoke
**Symptom:** After adding/moving a crate in `sdk/lib/rust/Cargo.toml`, a sibling cohab smoke (e.g. cmlib) starts FAILING — i386 HANG_TIMEOUT at `cmlib\hivewrt.c:360 "Failed to write the primary hive."` 3 warm runs reproduced the failure, looking deterministic; the crate you added (rossym) seemed to "regress" cmlib.
**Root cause:** Editing `Cargo.toml` `members` is a declared build-dep of the sibling crate's `add_custom_command` → forces an INCREMENTAL rebuild whose cargo-incremental codegen tips a helper into a bad-codegen hazard (cmlib `Hv*` → precompiled-std-in-kernel hivewrt hang). Clean COLD codegen is correct. A tempting symbol-collision theory (1126 core `T`-syms shared across the two archives) is a RED HERRING — those collisions are byte-identical instantiations, benign under `--allow-multiple-definition` first-match-wins.
**Fix:** after ANY workspace-composition change, COLD-build before trusting (or debugging!) a cohab smoke — nuke ALL rust target dirs (`find build-<arch> -type d -name '*rust*target*' | xargs rm -rf`; do NOT `find … -prune -exec`, it misfires and leaves them). The cold rebuild boots green with zero hivewrt errors. Don't chase a byte-identical-core-symbol "collision" as the cause.
**⚠ `vibeos-cli build --clean` is NOT a sufficient cold nuke for this:** it removes `rust-kernel-target` + `lib*_rust.a` but LEAVES the per-crate sibling cohab target dirs (`build-<arch>/sdk/lib/cmlib/cmlib-rust-target`, `…/lsalib/rust-lsalib-target`, `…/rossym/rossym-rust-target`, etc.), so cmlib-rust still builds incrementally and the hivewrt hazard survives. Confirmed 2026-06-14 on the uniata rebase: after `--clean`, `find build-i386 -type d -name '*rust*target*'` still listed cmlib-rust-target; i386 cold smoke flaked HANG_TIMEOUT at hivewrt:360 (passed on re-run — the bad-codegen binary boots *sometimes*, which is what makes it read as flaky rather than deterministic). The manual `find … | xargs rm -rf` nuke (which DOES catch the sibling dirs) → green. Always do the manual nuke after a Cargo.toml `members` change; don't trust `--clean` alone.

**Stale C object tree after a big merge → undersized "Frankenstein" binary (diagnose by SIZE).** `--clean` nukes only the *Rust* target — NOT the C/cmake object tree (objects, generated headers, `.def`/`.spec` exports). After a large merge (a worktree 246 commits behind, then merged clean so sources are byte-identical to mainline), the C incremental tree is internally inconsistent → a linked-but-*wrong* `ntdll`/`ntoskrnl` that boots partway then dies. A 2026-06-08 case crashed i386 at SMSS (`priv.c:446 RtlAdjustPrivilege → STATUS_NO_TOKEN → 0x4C 0xc000021a`) and read exactly like an rtl/ntuser code regression — it was stale artifacts. **Diagnose by binary SIZE, not verdict:** with sources provably identical to a known-green ref, compare `stat -c%s build-<arch>/dll/ntdll/ntdll.dll` against that ref's build dir (the bad `ntdll` was ~170KB short, `ntoskrnl` ~53KB short). A big size delta = stale artifacts, not a code bug. **Full C rebuild:** `cd build-<arch> && cmake . && ninja -t clean && ninja livecd` (`cmake .` first to pick up crates/custom-commands the merge added). **`ninja -t clean` exposes a header-gen ordering gap:** RC compiles of `services.rc`/`winlogon.rc`/`userinit.rc` fail with `minwindef.h: 'winnt.h' file not found` — `winnt.h` is *generated* by the `xdk` target and the RC res targets don't hard-order-depend on it (incremental builds mask it). **Fix: `ninja xdk` FIRST, then `ninja livecd`.** Faster than a 20-min cold C build: copy a green same-mainline worktree's `build-<arch>` and repoint it (never `ninja -t clean` a freshly-repointed tree — re-triggers the xdk race).
**Source:** `project_cmlib_rossym_cohab_smoke.md`, uniata rebase 2026-06-14

### Cold-build from a fresh worktree: generate psdk/xdk headers BEFORE the cargo crates
**Symptom:** A from-scratch `ninja livecd` in a freshly-`configure.sh`'d build dir dies very early (≈ step 34/8748) in a `*-cargo` build script: `cc-rs` assembling e.g. `chkstk_asm.s` fails with `minwindef.h:171: fatal error: 'winnt.h' file not found`. Warm/`cp -a`-seeded builds never hit it.
**Root cause:** `winnt.h` (+ ~268 other psdk/xdk headers) is GENERATED by a cmake CUSTOM_COMMAND (only `winnt.rh` is tracked). The cargo build scripts' `cc-rs` invocations run at the very front of the ninja graph and lack a dependency edge on the generated-header targets, so on a cold tree they race ahead of header generation. Seeded trees already have the headers on disk, hiding the missing edge.
**Fix:** generate the headers first, then build: `ninja psdk xdk` (the per-dir phony aggregates) **before** `ninja livecd`. One-time per fresh build dir. (Also: `configure.sh` defaults to `toolchain-gcc.cmake`; override with `-DCMAKE_TOOLCHAIN_FILE:FILEPATH=toolchain-llvm-mingw.cmake` — last-`-D`-wins.)

### i386 cold-link `undefined symbol: _Foo@24` from an extern decl with the wrong arg WIDTH (kd64, 2026-06-17)
**Symptom:** An honest cold i386 kernel link fails `ld.lld: error: undefined symbol: _KdpCopyMemoryChunks@24` even though the function is `#[no_mangle] pub unsafe extern "system" fn` and present. `nm libkernel_rust.a` shows BOTH `T _KdpCopyMemoryChunks@28` (the def) AND `U _KdpCopyMemoryChunks@24` (a caller) — same name, two stdcall byte-counts.
**Root cause:** Two `extern "system"` declarations of the same fn disagree on a parameter's WIDTH. `KdpCopyMemoryChunks`'s first arg is `ULONG64` (8 bytes — the kd debugger protocol is 64-bit even on i386) in the definition + kdb.rs, but `kdbreak.rs`/`kdx86.rs` declared it `ULONG_PTR` (4 bytes on i386) → 24-byte stdcall decoration vs the real 28 → the `@24` reference is undefined. **Silent on amd64/arm64** (there `ULONG_PTR`==`ULONG64`==8 bytes, and NTAPI is cdecl/undecorated). **Masked on warm i386** by §9.6b (the kernel hadn't honestly relinked) and by DCE removing the unreachable `@24` callers under normal opt — only a true cold link (or `-C no-prepopulate-passes`, which keeps the dead code) forces the reference to the linker.
**Fix:** make every extern decl's arg widths match the definition (here `Address: u64` + call sites `as u64`). **Recipe:** when a cold i386 link reports `undefined _Foo@N`, `nm | grep Foo` — if you see both `@N` and `@M`, grep all `extern "system"` decls of `Foo` for a `ULONG_PTR`/`ULONG`-vs-`ULONG64`/`u64` mismatch on the first 64-bit arg. Sibling of the verify_abi.py internal-symbol blind spot (§ABI) — invisible to spec/bindgen because it's an internal kd fn.

### `-C no-prepopulate-passes` as a captures(none) class-guard: scope it to the offending CRATE, not the whole kernel body (#38/#35, 2026-06-17)
**Context:** The #35 x86 ungate of the 5 deep-C cohab archives (cportlib/csq/arbiter/lsalib/rossym) shifts the kernel link layout and tips a latent captures(none)/DCE miscompile. `-C no-prepopulate-passes` is the established class-guard (already on rtl-rust/i386 + cnt-rust-lib in `reactos-link` `no_prepop` match).
**Finding 1 — whole-body is too blunt:** adding `("kernel-rust", _)` to the match (guarding the entire 447K-LOC body) DOES build + boot to `shell ready`, but the kernel balloons 4.3MB→9.8MB (all opt passes off) and boots so slowly it blows the 6-min smoke window (`shell ready` emitted just *past* the cutoff → reads as HANG_TIMEOUT). Not shippable. (Earlier "no-op on the body" claim was the §9.6b relink-staleness trap — the body DOES route through `build_rust_crate`→the match at lib.rs:1462; force `rm ntoskrnl.exe ntkrnlmp.exe` to verify.)
**Finding 2 — targeted is the right scope:** guard only the small crate on the miscompiled path. For #35 that's `("cmlib-rust", _)` (cmlib is already 100% Rust on x86; its `Hv*` hive-write path is the documented tip site). Kernel stays 4.3MB/normal speed. With the cmlib guard + x86 ungate, a COLD i386 build boots PAST the hivewrt hang all the way to `[SMSS] _main entered` — but then BUGCHECK 0x1E (`0xc0000005, 0x00000020, …` → faulting EIP = `0x20`, i.e. a CALL through a ~null/poison function pointer; pool-poison `0xbadb0d00` on the faulting stack).
**Finding 3 — SOLVED: the smss 0x1E was an i386 stdcall/cdecl ABI mismatch in cmlib `cmcheck`/`cmheal`, NOT captures(none) and NOT a bad hive (2026-06-18, fixed `336d31be6ff`; i386 cold smoke OK, amd64 warm boot+net OK).** The crack, after three wrong guesses:
- *Tooling unlock.* The gdb-attached i386 boot hangs at `[freeldr] Switching stack…` under **KVM** (gdbstub + real→protected→paging mode-transition quirk); **`VIBEOS_NO_KVM=1` (TCG)** boots past it cleanly AND gives working SW breakpoints (meaniepy). The 0-DWARF kernel is symbolicated with `qemu/runtime_symbolicate.py` (meaniepy's amd64 tool, now i386-aware — nm-derived RVAs, 32-bit PsLoadedModuleList offsets, page-granular MZ-scan fallback). It MZ-scans the correct kbase, walks PsLoadedModuleList, and resolves via full `nm` → names **Rust-internal (non-export)** frames, which `addr2sym`/`info symbol` cannot. `catch_bugcheck.sh` needs `--skip-preflight` (preflight hard-fails on 0-DWARF). **This is the durable #37 answer: nm-based runtime symbolication, NOT a split-debuginfo kernel.**
- *Three wrong guesses, ruled out:* (1) "it's in the 5 cohab libs, not cmlib" — WRONG, it IS cmlib. (2) "captures(none) miscompile" — WRONG, no-prepop on body+cmlib didn't touch it. (3) "bad `GetCellRoutine`=0x20 / corrupt hive" — WRONG: the faulting hive is **VALID** (`Signature=0xbee0bee0`, `GetCellRoutine=0x83a11590`, all FileXxx/Allocate/Free ptrs good, `DirtyVector.SizeOfBitMap=0x40`). The `HvGetCell` `[hive+4]==0x20` bp correctly never fired.
- *The crack — catch the instant EIP=0x20 with a CLEAN stack.* `hbreak *0x20` under `VIBEOS_NO_KVM=1` TCG stops BEFORE the page-fault dispatch overwrites the deep frames (KiTrap0E builds its 0x8c-byte frame at the low ESP, clobbering the culprit's frame — so the post-mortem trap frame can't see it). At the hit: `[esp]` was **not** a return address (`0x00000002`) and `0x20`/`0x08`/`0x292` (EIP/CS/EFLAGS) sat just below — i.e. a **`ret`/iret consumed a stack DATA word `0x20` as its target**, classic ESP desync. Frame chain: `CmpInitializeHive → CmCheckRegistry → CmpValidateRegistryInternal → CmpValidateKey → ret-to-0x20`.
- *Root cause = stdcall/cdecl ABI mismatch.* `cmcheck.rs`/`cmheal.rs` declared `HvMarkCellDirty`/`HvIsCellAllocated`/`HvGetCellSize`/`CmpCompareCompressedName` as `extern "C"` (cdecl, caller-pops) but they're DEFINED `extern "system"` = NTAPI __stdcall on i386 (`_Name@N`, callee-pops). **LLD's fuzzy `_foo`↔`_foo@N` match links it silently**, but on i386 BOTH sides pop the args → ESP drifts 8–12 bytes per call. The deep `CmCheckRegistry` per-key tree-walk calls these in a tight loop, so the drift accumulates until a `ret` lands on a stale stack value (`0x20`) → 0x1E. Invisible on amd64/arm64 (`system`==cdecl) — exactly why it was i386-only and survived the kernel32/cdfs gates (standard smoke didn't exercise the cohab-cmlib path on i386). **Fix:** declarations → `extern "system"`. (cmse.rs `HvGetCell`/`HvReleaseCell` verified via `llvm-nm` as genuinely cdecl `_HvGetCell` — correctly left `extern "C"`; the cmlib API surface genuinely mixes ABIs, so each decl must match its own def.)
- *Reusable recipe (NEW, the highest-value generalization):* an `EIP` = a small constant (`0x20`, `0x8`, …) or a stack-data-looking value, with `[esp]` not a return address, ⇒ **ESP desync, almost always an i386 stdcall/cdecl ABI mismatch hidden by LLD fuzzy `_foo`↔`_foo@N` linking.** Scan each crate for `extern "C"` fn decls whose REAL symbol is `_Name@N` (stdcall): `llvm-nm <lib>.a | grep -E ' [Tt] _[A-Za-z]+@[0-9]+'`, then cross-check every `extern "C"` decl name against that list. Sibling of the kernel32 RPC-wait ABI bug.
- *Recipe banked (catch any deterministic kernel fault on a 0-DWARF i386 kernel):* `VIBEOS_NO_KVM=1 start.sh i386 --gdb --skip-preflight`, then `hbreak *<KiDispatchException> if *(int*)(*(int*)($esp+4)+0xc)==<faulting_eip>`; on hit, `TrapFrame=[esp+0xc]`, regs at KTRAP_FRAME offsets (i386: Eax+0x44, Edi+0x54, Esi+0x58, Ebx+0x5c, Ebp+0x60, Eip+0x68), ExceptionRecord=`[esp+4]` (Code+0, Addr+0xc, Params+0x14).

### Stale C `.obj`/`.a` member survives merge → false cold-LINK_FAIL on a commented-out `.c`
**Symptom:** Cold link fails `undefined symbol: _Foo@N` `referenced by bar.c.obj` where `bar.c` is already commented out (replaced by Rust) in its CMakeLists; or fresh cold-clean reports `duplicate symbol` from a flipped file. Warm builds are green.
**Root cause:** `ar` never prunes members; a static archive (e.g. `librtl.a`) still bundles the orphaned `.obj`. ninja won't re-`ar` when the input list only *shrinks*, and regenerates `build.ninja` mid-run using the OLD graph for that same run. `vibeos-cli build --clean` nukes only Rust targets.
**Fix:** `rm build-<arch>/sdk/lib/rtl/librtl.a` + the orphaned `.obj` to force a clean re-ar, then `ninja <target>`. `ninja -t clean` does NOT help (orphans left the graph). For a trustworthy post-merge verdict after C→Rust re-flips, prefer a full `rm -rf build-<arch>` + reconfigure over `--clean`.
**FALSE-GREEN facet (2026-06-11, rtl-except flip):** the same zombie members also cut the *other* way — they keep *defining* symbols whose Rust replacement forgot to export them, so every link since the original flip was green on borrowed time. Found when leaning `librtl.a` broke ntoskrnl/ntdll-cargo with `undefined: RtlpPageHeapEnabled / NtCurrentTeb / RtlFillMemoryUlong / _swprintf` — all four had been satisfied for *days* by zombie `heappage.c.obj`/`thread.c.obj`/`mem.c.obj` members. After ANY flip, `rm` the consumer archive, re-`ar`, and confirm `ar t` lists only live members; treat "links clean without touching the archive" as suspicious, not lucky. (Fix class: missing `#[no_mangle]` on cross-crate statics, extern decls of symbols nothing defines → inline them, "CRT provides it" claims that were really zombies.)
**Source:** `feedback_stale_c_archive_after_merge.md`, `feedback_freeldr_pc_layer_cold_dup.md`, rtl-translation-audit.md §4 (2026-06-11 note)

### freeldr `freeldr_pe`/`uefildr` cold dup (`_PcCons*`/`_Mach*`/`_FbCons*`) is a stale common-archive
**Symptom:** `duplicate symbol: _PcConsGetCh / _MachVtbl / _ArcGetTime / _FbCons* / _VidFb*` linking `freeldr_pe.exe`/`uefildr.efi` cold; the flip is actually complete.
**Root cause:** `libfreeldr_common.a` / `libuefifreeldr_common.a` still bundle `.obj`s for now-excluded C (`arch/arcemul.c`, `pccons.c`). Nuking only `rust-freeldr-target` + `libfreeldr_rust*.a` does NOT clear it — the stale objs live under `CMakeFiles/{freeldr_common,uefifreeldr_common,rosload_common}.dir`.
**Fix:** `rm -rf build-*/boot/freeldr/freeldr/CMakeFiles/{freeldr_common,uefifreeldr_common,rosload_common}.dir build-*/boot/freeldr/freeldr/lib{freeldr,uefifreeldr,rosload}_common.a` then `cmake . && ninja freeldr_pe.exe uefildr.efi`. Do NOT add `--allow-multiple-definition` (it just masks the stale archive).
**Source:** `feedback_freeldr_pc_layer_cold_dup.md`

### Smoke "green ×3" lies because conftest nukes only `rust-kernel-target`
**Symptom:** A freeldr/cnt-rust change smokes green and you're surprised; or HEAD "boots" but a truly fresh build of the same commit fails or bugchecks early.
**Root cause:** `conftest.py _ensure_clean_rust_build` nukes only `build-{arch}/ntoskrnl/rust-kernel-target/`; it does NOT touch `rust-freeldr-target/`, `rust-freeldr-uefi-target/`, or `build-<arch>/sdk/lib/crt/cnt-rust-target/`. Those incremental caches hold pre-change `.a` snapshots that mask the real cold-link / runtime state.
**Fix:** Before claiming cold green, also nuke the relevant target dirs: `rm -rf build-<arch>/sdk/lib/crt/cnt-rust-target` for cnt, and `rust-freeldr-target` + `rust-freeldr-uefi-target` + consumer `libfreeldr_*.a` for freeldr. Sanity check: `stat -c '%y' build-{arch}/.../libfreeldr_rust.a` — if it predates your change, the smoke means nothing.
**Source:** `feedback_freeldr_cargo_cache_warm.md`, `feedback_libcntpr_rust_stale_archive_false_breaks.md`, `feedback_cnt_rust_ntdll_export_replaced_coldlink.md`

### cnt-rust `.a` "build-breaks consumers" / "X was replaced" only on some worktrees
**Symptom:** A peer worktree reports `f5a44e61` build-breaks livecd: `undefined symbol: rust_eh_personality` in C consumers and `duplicate symbol _memset/_memcpy` in rosload.exe/uefildr.efi; one tree links green, another red on the SAME HEAD.
**Root cause:** Two distinct issues. (1) Stale `libcntpr.a`/`libcntpr_rust.a` member-set collision (stale-archive class). (2) A REAL link-order-dependent dup: EXEs link both the i386 asm CRT and the Rust CRT, but `--allow-multiple-definition` was only on `CMAKE_SHARED/MODULE_LINKER_FLAGS`, not `CMAKE_EXE_LINKER_FLAGS`; which member wins is nondeterministic across worktrees.
**Fix:** Stale side: `rm build-<arch>/sdk/lib/crt/{libcntpr.a,libcntpr_rust.a}` + `rm -rf .../cnt-rust-target`. Real side (`bf0d5e2c851`+`bfbc474eac1`): add `--allow-multiple-definition` to `CMAKE_EXE_LINKER_FLAGS`; use a PLAIN STRONG `#[no_mangle] fn rust_eh_personality(){}` (NOT `global_asm ".weak"` — its PE weak-external `.weak.<sym>.default.<hash>` perturbs CRT emission). "Fresh links green on MY tree" is NOT proof for link-order dups — check the actual EXE-vs-SHARED link flags.
**Source:** `feedback_libcntpr_rust_stale_archive_false_breaks.md`

### Fresh cnt-rust-target won't cold-link: "memset/`_wcsicmp`/`_itow` was replaced" (import vs local)
**Symptom:** A truly fresh `cnt-rust-target` fails cold link on i386 AND amd64: `__wcsicmp was replaced` (winsrv.dll), `__itow was replaced` (win32k.sys), `memset/memmove was replaced` (ftfd.dll). Green until now only because the stale cache predated the conflicting flips.
**Root cause:** cnt-rust cohab flips define CRT symbols ntdll ALSO exports; user-mode native modules link BOTH `libcntpr_rust.a` AND ntdll (carrying `__imp_<sym>` thunks). A local strong def for a symbol already bound to an ntdll import is the FATAL `was replaced` (regular dup is only a tolerated warning) — `--allow-multiple-definition` does NOT cover import-vs-local.
**Fix (RESOLVED `a21d879c5cc`→`32aa88f`):** `gen_cnt.py` was emitting all ~100 cabi shims into ONE module = one monolithic cgu object, so any reference dragged the whole thing. Fix = one `mod mX { ... }` per shim → per-function cgu objects (C archive-member granularity); linker pulls only the exact symbol. The strong `rust_eh_personality` was a red herring, not the fix.
**Source:** `feedback_cnt_rust_ntdll_export_replaced_coldlink.md`

### Companion DLL cold link cascades a wall of undefined symbols after a leaf flip
**Symptom:** Flipping one C fn makes a companion DLL (e.g. `ntdll_vista.dll` from `--features vista`) cold-fail with ~20 undefined (`LdrProcessRelocationBlockLongLong`, `find_entry_*`, `ApiSetResolveToHost`). Warm builds mask it.
**Root cause:** Before the flip the companion's `extern` resolved via the main DLL's import lib; after, the fn lives in the shared rust staticlib, and archive-def beats import-lib so lld pulls its static def into the companion, dragging the rest of the loader; the companion's link line lacks the C providers.
**Fix:** `#[cfg(not(feature = "vista"))] pub mod ldr;` in lib.rs — exclude the flipped subsystem from the companion's build so it imports the fn from the main DLL's import lib. Do NOT add `rtl` to the companion (→ `_RtlInitUnicodeString@8 was replaced`). raw-dylib `import_name_type="undecorated"` dodges the pull on i386 ONLY; gating is the real cross-arch fix.
**Source:** `feedback_cohab_flip_companion_dll_cascade.md`

### Completing a `rtl_vista` C→Rust flip needs a THIRD edit (else reactos-link panics)
**Symptom:** You enable an `rtl_vista` file's `.rs` (`#[cfg(feature="cabi")] pub mod X;`) and comment its `.c` in `sdk/lib/rtl/CMakeLists.txt` `SOURCE_VISTA`, but the ntdll cargo link confusingly still expects a stale C object — or, if you ALSO removed it from CMakeLists only, `reactos-link` PANICS at `lib.rs:991 "no compile_commands.json entry for X in target rtl_vista"`.
**Root cause:** the ntdll cargo link compiles the rtl_vista C files IN-CARGO via a hardcoded list in `sdk/lib/rust/ntdll-cargo/build.rs` → `.c_from_compile_db("rtl_vista", &[".../X.c", …])` — a SECOND place (besides `SOURCE_VISTA`) that names the C file. A vista flip therefore takes THREE edits, not two. (Main-SOURCE rtl leaves take only the two CMakeLists edits — NO build.rs touch.)
**Fix:** per vista file: (1) enable `pub mod X;` in `rtl-sync/src/lib.rs` (+ srcs to rtl/CMakeLists DEPENDS); (2) comment the `.c` in `SOURCE_VISTA`; (3) comment/remove the `.c` from `ntdll-cargo/build.rs` `.c_from_compile_db("rtl_vista", …)`. GOTCHA: `SOURCE_VISTA` can't go empty (`add_library(rtl_vista)` needs ≥1 source) — leave the last `.c` as harmless residue (librtl_vista.a feeds only the standalone ntdll_vista forwarder, never linked alongside librtl_rust.a). ntoskrnl does NOT link rtl_vista; only ntdll.
**Source:** `project_rtl_sync_integration.md`

### Middle-layer flip surfaces undefined symbols in WAVES (Rust `.o`s pulled for first time)
**Symptom:** Flipping a module called from above AND below (e.g. winsrv `condrv/`) is far noisier than a leaf flip; link errors cite `.c.obj` files you already commented out (phantom refs).
**Root cause:** With `--allow-multiple-definition` + C-first, the Rust `.o`s in the archive are never pulled while C provides the symbols; flip C away and they become sole providers, dragging all their undefined externs (CONTAINING_RECORD, macros). Stale flipped `*.c.obj` linger as archive members.
**Fix:** `ninja -t clean <module>` (e.g. `ninja -t clean consrv`) then rebuild for the TRUE undefined set. Decoration tells what the caller WANTS (`_Name@N`=stdcall, `_Name`=cdecl) but the C source line above the def is canonical (`NTAPI`=stdcall). Promote helpers to `#[no_mangle] pub unsafe extern "C"|"system"`, split mixed extern blocks rather than flipping a whole block. Force the rust `.a` rebuild after `.rs` ABI edits (`find build-<arch> -name 'lib<crate>_rust.a' -delete` + nuke cargo target — cmake DEPENDS doesn't track `.rs`).
**Source:** `feedback_cohab_middle_layer_flip_abi.md`

### Cohab link-clean ≠ runtime-correct: bugcheck mid-init after clean cargo/ninja
**Symptom:** A cohab flip (smss.exe, basesrv.dll) links clean and `_main`/`DllMain` return success, then bugchecks late (post-DllMain, mid-init) — e.g. smss 0xC000021A, or basesrv hangs in the winmm DLL chain. Silent in `cargo check` and `ninja`.
**Root cause:** rustconv defects that link clean: pointer-cast lies (PHANDLE↔HANDLE), dropped fn bodies, macro-as-extern stubs (CONTAINING_RECORD), ABI mismatches, and struct-layout mirrors with wrong field order/missing fields (e.g. `BASE_STATIC_SERVER_DATA` sizeof 0x1AC8). The smaller per-binary archive also repartitions cgus, exposing latent UB.
**Fix:** Never commit a cohab flip before smoke green ×3. Use `qemu/start.sh i386 --gdb` + `run-until.sh --break <suspect-fn> --then triage-bugcheck`; diff Rust call site against C SOURCE for cast lies / dropped args. Budget ~30 min per cascade layer (each fix exposes the next bug). If it regresses smoke, revert the CMAKE (not the Rust source) and reserve a `<binary>_only` feature.
**Source:** `feedback_cohab_flip_runtime_traps.md`

### `#[no_mangle] pub static mut` in a shared lib → per-consumer private data copy
**Symptom:** Cross-DLL data sharing silently breaks; e.g. csrss.exe writes a port handle that winsrv.dll (reading csrsrv.dll's export) sees as NULL → `STATUS_INVALID_HANDLE`, no shell ready.
**Root cause:** A `#[no_mangle] pub static mut FOO` is a definition. When lld pulls archive members into the .exe to satisfy a fn ref, it drags side-symbols including `_FOO`; the .exe's own static def beats the .dll's PE import (static > import in lld order). `--allow-multiple-definition` does NOT save you — archive first-match wins before import resolution.
**Fix:** Define the data global in EXACTLY ONE link unit via Cargo feature gate; non-definer consumers opt out (e.g. csrss builds `--no-default-features --features csrss_main_only,ntddi_ws03`, gating `csrsrv`/`csrlib`/`smlib`/`smss`/`basesrv` modules) and PE-import instead. Verify with `nm <consumer-exe> | grep <Sym>`: exactly one `D`/`B`, the rest `U`; two `D`s = bug.
**Source:** `feedback_static_lib_shared_globals_per_consumer.md`, `feedback_cohab_flip_runtime_traps.md`

### Dual-archive cohab: a consumer runs STALE Rust because its DEPENDS list doesn't mirror
**Symptom:** Edits to `csrsrv/init.rs` land in csrsrv.dll's binary (`strings` confirms) but never appear on serial; csrss.exe runs old code. `nm` shows the symbol (`_CsrServerInitialization@8`) in BOTH binaries.
**Root cause:** csrss.exe and csrsrv.dll each run `cargo build` with their OWN `CARGO_TARGET_DIR` (`.../csrss/rust-target` vs `.../csrsrv/rust-target`) → two physically distinct `libsmss_csrss_rust.a`, each gated by its own cmake DEPENDS. A shared `.rs` edit only invalidates the consumer whose DEPENDS lists it.
**Fix:** Mirror the canonical csrss/CMakeLists.txt DEPENDS (commit `032cf72`) for every sibling consumer of the shared crate; or better, move all consumers to one shared `add_custom_target` via `add_dependencies`. Always `nm <consumer-exe>` to confirm the binary holds the symbol you think you're editing before debugging.
**Source:** `feedback_csrss_cohabitation_dual_archive.md`

### Cohab unit is PARTIAL → un-cold-linkable by CMakeLists alone (dup↔undefined oracle)
**Symptom:** Cold link of a unit fails BOTH ways: leaving C in gives `duplicate` symbols, commenting C out gives `undefined` symbols; no `.c`-subset toggle fixes it. Warm binaries hide it because ninja never relinks an untouched component.
**Root cause:** The Rust port is partial — its `.rs` `#[no_mangle]`s SOME of the C unit's exports but not all (e.g. crp freeldr's mm port provides `_FrLdrHeap*`/`_MmInitializeHeap` but not `_ExAllocatePool*`/`_AddMemoryDescriptor`). Each unit must be all-C (pub mod un-wired) or all-Rust (.c commented).
**Fix:** Apply the oracle per unit: C-present+wired dup ⇒ Rust provides; C-commented undefined ⇒ Rust only consumes; BOTH ⇒ partial. A partial unit needs Rust-crate edits (un-wire the incomplete `pub mod` so C fully owns it, or finish the port) — keep partial units as C. If the break is in another agent's in-flight crate, root-cause and hand back; don't fix mid-collision or revert your own green work.
**Source:** `feedback_cohabitation_consume_vs_provide.md`

### Driver cohab: provide DDK/driver MACROS + FORCEINLINEs correctly via real headers, NOT bugcheck stubs (fastfat, 2026-06-13)
**Symptom:** A fully-translated driver crate references its own header macros (`FatIsFat32`, `FatGetLboFromIndex`, `FatLookup12BitEntry`) and DDK intrinsics (`KeGetCurrentIrql`, `KeQuerySystemTime`, `KeAcquireSpinLock`) via `extern "C"` — none have a linker symbol (they're macros / FORCEINLINEs). Tempting (and wrong) fixes: (a) hand-roll the struct offsets in a no-header TU (fragile, arch-specific — `VCB_RESOURCE_OFFSET=404` breaks on the next struct change / other arch); (b) `STUB_BUGCHECK()` them to force a link (false-green — the FS faults on first real I/O; FATAL where the driver is the boot volume, e.g. arm64 fastfat on the FAT32 ramdisk).
**Fix / oracle:** Write a cohab provider TU that **`#include`s the driver's real PCH** (`fatprocs.h` → ntifs.h + the private FAT headers) so every macro/inline/struct-layout is the compiler's own — ZERO hand offsets. It compiles standalone with the module's flags plus `-mcx16` and `-Isdk/lib/pseh/include`. Classify each needed symbol with a one-shot `#ifdef X` probe (compile a TU of `#ifdef X \n #warning MACRO:X \n #endif`), then:
  - **function-like MACRO** → POST-include capture: `static RET cap(args){ return X(args); }` then `#undef X` then real `RET X(args){ return cap(args); }`. (You can't write `RET X(decl-args){…}` directly — `X(decl-args)` is itself a macro invocation that expands to garbage; the capture-then-undef two-step is required.)
  - **FORCEINLINE function** → PRE-include rename: `#define X X_inl` BEFORE the include so the header emits its inline body as `X_inl`, then real-named `RET X(args){ return X_inl(args); }`. (Pre-rename does NOT work on macros — the header's `#define X(…)` re-defines and overrides your alias.)
  - **Vista+ (≥0x600) API #if'd out at the driver's 0x502 profile** (probe = not-a-macro AND header doesn't declare it → "implicit function declaration" on the alias) → direct impl on a 0x502 export: `ObReferenceObjectWithTag`→`ObfReferenceObject`, `KeExpandKernelStackAndCalloutEx`→`Callout(Param)` direct, `FsRtlCheckOplockEx`→`FsRtlCheckOplock`.
CRITICAL link-order rule: this provider TU is linked AFTER the bugcheck-stub TU, and `--allow-multiple-definition` is first-match-wins — so you MUST delete the shadowing bugcheck stubs from the stub `.c`, or the bugcheck still wins. Authoritative shadow set to remove = `cohab-defined ∩ Rust-`.a`-defined`. fastfat: `drivers/filesystems/fastfat/rust_cohab_fat.c` (17 intrinsics + 25 FAT macros), amd64 links clean; 18 untranslated fsctl FSCTL handlers deferred (off boot/mount/read path, still bugcheck-stubbed — tracked).
**Source:** `project_fastfat_rust_track.md`

### Shared `librtl.a` in BOTH ntoskrnl AND ntdll: kernel-VA read faults in user mode (silent idle hang)
**Symptom:** x86 boot reaches `[WIN32K] shell ready` but never `[EXPLORER] _tWinMain entered`; serial dead after `[LDR] DllMain(apphelp.dll) ret=1`; whole system goes idle (HLT) with NO bugcheck. Arch-neutral (i386+amd64).
**Root cause:** rtl-strings is built ONCE with `kernel` feature ON, and that single `librtl.a` links into BOTH ntoskrnl (kernel) and ntdll (user) — so `#[cfg(feature="kernel")]` cannot distinguish context. `RtlEncodeSystemPointer` read `KUSER_SHARED_DATA.Cookie` at the kernel VA `KI_USER_SHARED_DATA_VA` (i386 `0xFFDF0000`, amd64/arm64 `0xFFFFF78000000000`); in user mode it lives at `0x7FFE0000` → #PF, uncatchable under DUMMY_PSEH → thread wedges.
**Fix (`c65ad91`):** pick the VA at RUNTIME, not via cfg — sign-bit probe of a stack address: `fn in_kernel_mode() -> bool { let probe: u8 = 0; (addr_of!(probe) as isize) < 0 }`, then `kuser_shared_data_va()` returns `KI_USER_SHARED_DATA_VA` if kernel else `0x7FFE0000`. `RtlEncodePointer` (non-System) is fine (it routes via `ZwQueryInformationProcess`).
**Source:** `feedback_rtl_shared_librtl_user_kernel_va.md`

### Shared `librtl.a` alloc stubs return NO_MEMORY in the live kernel (3-mode gate needed)
**Symptom:** Fresh cold build bugchecks Phase 0 `ExpInitializeExecutive` at `RtlAnsiStringToUnicodeString` (0x31, STATUS_NO_MEMORY); after a 2-way fix, SMSS dies in `SmpCreateSecurityDescriptors` (0x4C). main's "GREEN 10/10" was stale `librtl.a` containing C `unicode.c.obj`.
**Root cause:** rtl-strings cabi modeled all `alloc=TRUE` paths as `return STATUS_NO_MEMORY` (difftest-harness model, no allocator). A 2-way kernel/else gate is ALSO wrong — the same `librtl.a` links into both ntoskrnl and ntdll, and only kernel sets the `kernel` feature, so the user-mode ntdll build fell into the NULL branch.
**Fix (`478e642d2b8`):** 3-way cfg — `feature="kernel"`→C `RtlpAllocateMemory`/`RtlpFreeMemory`; `not(kernel)+target_os="windows"`→`RtlAllocateHeap(RtlGetProcessHeap(),0,bytes)`; `not(kernel)+not(windows)`→host NULL stub. Also fix the `RtlFree*` stubs to actually free. `grep 'return STATUS_NO_MEMORY'` the cabi file for remaining unconditional-NO_MEMORY paths (e.g. `RtlStringFromGUID` was the amd64 0x6B root cause).
**Source:** `feedback_rtl_strings_alloc_stubs.md`

### libcntpr cohab swap by commenting the family `*_SOURCE` breaks msvcrt.dll
**Symptom:** Commenting a `.c` out of `LIBCNTPR_STRING_SOURCE` produces undefined symbols in user-mode (csrss/explorer fail; smoke never reaches shell).
**Root cause:** `sdk/lib/crt/**/*.cmake` build the CRT TWICE from the same vars — `LIBCNTPR_*_SOURCE` (kernel → libcntpr.a) AND `CRT_*_SOURCE` which does `list(APPEND CRT_STRING_SOURCE ${LIBCNTPR_STRING_SOURCE})` (user-mode msvcrt). cnt-rust is linked into libcntpr ONLY. The librtl pattern (private vars) does NOT transfer.
**Fix:** Scope the swap to libcntpr: in `libcntpr.cmake`, after the `list(APPEND LIBCNTPR_SOURCE …)` aggregation and before `add_library(libcntpr …)`, add `list(REMOVE_ITEM LIBCNTPR_SOURCE <paths>)` (no-op when absent, so per-arch entries are safe unconditionally). Before any cohab swap, grep the family cmake for the build var appearing in a second `list(APPEND OTHER_SOURCE ${THAT_VAR})`.
**Source:** `feedback_libcntpr_shared_source_crt.md`

### Smoke red right after merge/handoff is usually a stale worktree, not a fresh regression
**Symptom:** Smoke goes RED post-merge/handoff; you start kernel-debugging a "regression" and burn a session (e.g. attaching gdb to a services.exe spin at `[WIN32K] shell ready`, explorer never starting).
**Root cause:** The worktree is N commits BEHIND continue-rust-port and is missing a fix that already landed on crp (the input-stack case was missing `c65ad91` the RtlEncodeSystemPointer VA fix). Compounded by stale build-dir C archives, and cross-worktree KVM/cgu contention slowing boot past the timeout.
**Fix:** FIRST, before any debugging: `git rev-list --count HEAD..continue-rust-port` and `git log --oneline HEAD..continue-rust-port` — is one of them the fix? `git merge --ff-only continue-rust-port`. Then `git rev-list --count crp..HEAD` for your real delta; clean build + `rm` stale C `.a`s; `bash qemu/ps` for contention (contention shows serial *progress*, a real hang stops dead). The i386 success marker is `[EXPLORER] _tWinMain entered`, NOT "shell ready".
**Source:** `feedback_worktree_behind_crp_false_regression.md`

### Cold build fails `encountered diff marker` in a crate another track owns
**Symptom:** After merging crp, `cargo build` fails `error: encountered diff marker` → cold LINK_FAIL on every arch; the tree is clean and `git status`/`git diff --check` show nothing.
**Root cause:** A parallel same-user session committed unresolved stash-pop conflict markers (`<<<<<<< Updated upstream` / `=======` / `>>>>>>> Stashed changes`) straight into crp HEAD (`reactos-link/src/lib.rs`, `d8f15643bf3`) via a broad `git add` in the shared main checkout. A committed marker is invisible to `git diff --check` (it's in a commit, clean tree).
**Fix:** `git show continue-rust-port:<file> | grep -nE '^(<<<<<<<|=======|>>>>>>>)'` to confirm it's on crp HEAD; `git log -S'<<<<<<< '` names the culprit. Do NOT guess-resolve another session's WIP (the two sides are often non-equivalent) — hand off a paste-ready brief and `git reset --hard <your last green merge>` off the bad base. Prevention: never `git add -A` in the shared main checkout; stage by explicit path in a dedicated worktree.
**Source:** `feedback_crp_committed_conflict_marker.md`

### New DbgPrint/trace points produce nothing in serial.log — stale livecd.iso
**Symptom:** Added trace points return nothing in `qemu/runs/<id>/serial.log`, the function is present in the .dll (objdump confirms), and smoke passed — the code just never runs.
**Root cause:** i386/amd64 smoke boots from `build-<arch>/livecd.iso`, not the DLL directly. `ninja hal`/`ninja ntoskrnl` updates the DLL in the tree but does NOT re-roll the ISO; the VM boots whatever was last baked in. ninja reports "Finished" with no invalidation back to the ISO target.
**Fix:** After ANY ntoskrnl/hal change run `cd build-<arch> && ninja livecd` (or `bootcd` for stage-2) BEFORE smoke or `qemu/start.sh`. ARM64 uses the UEFI ramdisk path instead (mcopy into `boot.img` after every change).
**Source:** `feedback_stale_livecd_iso.md`

### raw-dylib in a freeldr-LOADED DLL hangs boot (second import descriptor)
**Symptom:** A Rust DLL mapped by freeldr (bootvid.dll, kdcom.dll) hangs at load with NO serial/panic — the DLL never runs. `objdump -p the.dll | grep "DLL Name"` shows `ntoskrnl.exe` twice.
**Root cause:** `#[link(name="ntoskrnl.exe", kind="raw-dylib")]` for a cross-DLL data import emits a SECOND `ntoskrnl.exe` import descriptor (on top of the import-lib one). The kernel loader tolerates two descriptors; freeldr's peloader does NOT. This is the OPPOSITE of kernel-loaded drivers.
**Fix (rule splits by loader):** freeldr-loaded (bootvid, kdcom) → use the import lib's `__imp_` IAT thunk directly: `extern "C" { #[cfg_attr(target_arch="x86", link_name="_imp__KeLoaderBlock")] #[cfg_attr(not(target_arch="x86"), link_name="__imp_KeLoaderBlock")] static __imp_KeLoaderBlock: *mut PLOADER_PARAMETER_BLOCK; }` then read via `*__imp_KeLoaderBlock`. (i386 import lib symbol is `__imp__KeLoaderBlock` two-underscore, but rustc prepends one, so write `_imp__KeLoaderBlock`; find exact name via `llvm-nm .../libntoskrnl.a | grep __imp_`.) kernel-loaded (MiLoadSystemImage) → raw-dylib OK/required.
**Source:** `feedback_freeldr_loaded_dll_no_raw_dylib.md`

### Cross-DLL DATA refs in a kernel-loaded Rust DLL need raw-dylib (i386), and the cgu>1 dup descriptor
**Symptom:** Without raw-dylib, i386 link fails `undefined symbol: _ExEventObjectType` (cross-DLL data ref under `--disable-auto-import`); or objdump shows duplicate `ntoskrnl.exe` IMPORT_DESCRIPTORs sharing one IAT.
**Root cause:** For DATA refs across DLLs the linker can't synthesize indirection — the source must opt in. hal.dll links with `--disable-auto-import`; functions auto-resolve via the import lib, data does not. With `codegen-units=256` lld emits multiple descriptors per DLL sharing the same IAT slice — harmless on a correct PE loader; ReactOS FreeLDR had a bug (`PeLdrpBindImportName` mutated OriginalFirstThunk in place) → "Ordinal number is invalid!".
**Fix:** `#[link(name="ntoskrnl.exe", kind="raw-dylib", modifiers="+verbatim")] unsafe extern "C" { pub static KdDebuggerNotPresent: u8; }` — `+verbatim` so the target is `ntoskrnl.exe` not `.dll` (FreeLDR does byte-compare with extension). `postprocess_bindgen.py --hal` wraps the matching externs from `ntoskrnl.spec`. Verify with `llvm-objdump -p`. The FreeLDR dup bug was fixed locally (`975766d9`, use a `ULONG_PTR ImportDataVa` local). Future-proof alternative is the `__imp_X` IAT symbol (ONE descriptor).
**Source:** `feedback_raw_dylib_cross_dll_data.md`

### win32k raw-dylib: fake-export phantom descriptor (freeldr OOM) + real-export per-arch bugcheck
**Symptom:** A win32k commit (`dc2ead8df35`) regressed i386+amd64 boot — freeldr OOM (`MmFindLocationForPageLookupTable`); or amd64 bugcheck 0x6B at PsLocateSystemDll (0xC000003A) when raw-dylib is applied unconditionally.
**Root cause:** Hazard 1: rustconv emitted `#[link(raw-dylib)] extern { static SharedUserData }`, but SharedUserData is NOT a real ntoskrnl export — it's the fixed-VA `KI_USER_SHARED_DATA` page; raw-dylib creates a phantom IMPORT_DESCRIPTOR freeldr chokes on. Hazard 2: `ExEventObjectType` IS a real export — i386 (`--disable-auto-import`) NEEDS raw-dylib for the `__imp_` thunk, but amd64/arm64 bugcheck WITH it.
**Fix (`7dfb9250fe5`):** Fake-export → VA-cast helper, not raw-dylib: `unsafe fn shared_user_data() -> *mut _KUSER_SHARED_DATA { KI_USER_SHARED_DATA as usize as *mut _ }`. Real-export → per-arch cfg gate: `#[cfg(target_arch="x86")]` gets `#[link(raw-dylib, +verbatim)]`, `#[cfg(not(target_arch="x86"))]` plain extern. Bisect lesson: don't bisect against a moving cgu=256 partition (toggling modules reshuffles codegen) — hold the module set constant, vary ONE extern decl / cfg attribute at a time; verify any rustconv `raw-dylib extern static X` is actually in `ntoskrnl.spec`/`libntoskrnl_implib.def`.
**Source:** `feedback_win32k_raw_dylib_freeldr_oom.md`

---

### Editing a kernel C SHIM silently ships the STALE object (no `rerun-if-changed`)
**Symptom:** you edit `ntoskrnl/kmsafecopy.c` (or `seh_poc.c`, `*_rust_shims.c`, `ehandler.c`, the CRT printf shells), rebuild, boot — and the kernel runs the PREVIOUS build. The build is green. Nothing warns you. Worse than a stale build: **it fabricates evidence** — the boot prints a verdict for code that is not running.
**Root cause:** `reactos-link` (`sdk/lib/rust/reactos-link/src/lib.rs`) emitted `cargo:rerun-if-changed` for `build.rs`, `rust_watch_dirs`, and every workspace crate's `src/` + `Cargo.toml` — but **not** for `c_shims`. The shims live OUTSIDE `sdk/lib/rust`, and because the engine emits *explicit* rerun-if-changed lines, cargo's default whole-package watching is **disabled**. So a shim edit triggers no build-script rerun, cc-rs never recompiles, and the link reuses the old object while cargo reports the module fresh.
**The cruel part — intermittency:** it only bites when you edit a `.c` **without** touching `build.rs`. Wire a new shim up (which edits `build.rs`) and it works; iterate on that same `.c` afterwards and you silently test the old binary. Cost a full arm64 boot + a wrong conclusion (an arm64 SEH "FAIL" that was really the previous build's output).
**This is the SAME class the engine already documents** at `lib.rs:1028` for Rust path-deps ("a false-fresh that cost a debug round-trip on the WHPX spike") — fixed there for Rust crates, missed for C. Adding the Rust watch is precisely what un-watched everything else.
**Partial fix + the REMAINING layer (corrected 2026-07-13):** `reactos-link` now emits `cargo:rerun-if-changed` for every `c_shims` path (commit e98089d11d0). Verified it emits (`…/build/ntoskrnl-cargo-*/output` contains `rerun-if-changed=/tmp/vibeos/ntoskrnl/kmsafecopy.c`). **But this alone is NOT sufficient**, and the original commit message overclaimed it: there is a SECOND gate one layer out. The kernel link is a cmake `add_cargo_module` → a ninja `CUSTOM_COMMAND` whose only input is the `sdk/lib/rust/ntoskrnl-cargo` **directory** (`build ntoskrnl.exe: CUSTOM_COMMAND ../sdk/lib/rust/ntoskrnl-cargo`, `restat=1`). A directory's mtime does NOT change when a shim's *content* changes (only on create/delete/rename), so **ninja never re-invokes cargo** — and if cargo isn't invoked, its `rerun-if-changed` is never consulted. Empirically proven: `touch ntoskrnl/kmsafecopy.c && ./vibeos-cli build --arch arm64` leaves `ntoskrnl.exe` mtime UNCHANGED. So the cargo-layer fix is necessary but not sufficient; the ninja layer still gates.
**Reliable workaround until the ninja layer is fixed:** `rm build-<arch>/ntoskrnl/ntoskrnl.exe` before the build (ninja must rebuild a missing output → invokes cargo → cargo's rerun-if-changed recompiles the edited shim). Touching a file *inside* `ntoskrnl-cargo/` does NOT work (file mtime ≠ dir mtime). A `--clean` build also works but is far slower.
**Proper fix OWED:** make the `add_cargo_module` ninja command depend on the actual c_shim files (they live in build.rs, invisible to cmake — so either surface the list to cmake, or glob the known shim dirs as extra DEPENDS). Until then, treat any hand-edit of a `.c` shim as needing the `rm` workaround.
**Tell:** any time a boot-visible change to a C shim "has no effect", or a print shows the value you had *before* your edit. Confirm with `stat -c %y build-<arch>/ntoskrnl/ntoskrnl.exe` before/after — if the mtime didn't move, the link was skipped. **General rule: when an experiment's result contradicts the code you just wrote, first prove the code you wrote is the code that RAN.**
**Source:** conformance-ratchet (2026-07-12, corrected 2026-07-13).

## CRT/cnt runtime, harness flakes & kernel one-offs

Hard-won lessons on libcntpr/CRT translation bugs, smoke-harness flakes that are NOT real regressions, and kernel-internal one-off translation traps — keep the three clusters distinct so you debug the right thing.

### cnt swprintf writes nothing → BUGCHECK 0x1E (signed/unsigned `_cnt` compare)
**Symptom:** Every wide `swprintf` write refused; ACPI `main.c` HardwareIds `NT_ASSERT(Length*sizeof(WCHAR)==HardwareIdsLength)` fails → BUGCHECK 0x1E on i386 boot.
**Root cause:** `streamout_char`'s C guard `if (stream->_cnt < sizeof(_TCHAR))` is an UNSIGNED compare (`sizeof`→`size_t` promotes `int _cnt`). Unbounded `_swprintf` sets `_cnt=(int)(INT_MAX*2)=-2`, which the unsigned promotion reads as huge → writes proceed. A naive signed Rust compare (`-2 < 2`) reads it as "full".
**Fix:** Replicate the unsigned promotion exactly: `if ((*s)._cnt as isize as usize) < size_of::<T>() { return 0; }`. WIDE-only bug (narrow `sprintf` gives positive `INT_MAX`). When porting C that compares a signed int against a `sizeof`/`size_t` expression, the compare is UNSIGNED — never write a signed Rust compare.
**Source:** `feedback_cnt_printf_swprintf_cnt_unsigned.md`

### cnt strncmp/strncpy/strncat source over-read → BUGCHECK 0x50
**Symptom:** Page-fault `0x50 (PAGE_FAULT_IN_NONPAGED_AREA)`, page-aligned READ at param1, on boot. Difftests stay green (heap slack hides it).
**Root cause:** cnt-rust `n`-bounded shims built the SOURCE slice with the UNBOUNDED `cstr_slice(src)`/`wcstr_slice(src)` (scans to NUL). The whole point of `n` is the source may be a non-NUL-terminated fixed field (OEM IDs, signatures) — the NUL-scan walks off the field into an unmapped page.
**Fix:** In the generator `tools/gen_cnt.py` (output is @generated), bound src to n via `cstr_slice_bounded(src, n)`/`wcstr_slice_bounded`; new `nul_in_n`/`wnul_in_n` patterns bound both compare operands. Rule: any C-ABI shim with an explicit count param must bound its pointer-derived slice to that count.
**Source:** `feedback_cnt_nbounded_source_overread.md`

### "SIMD f64 requires sse" pulling a float crate into the i386 kernel
**Symptom:** Hard rustc error "this function definition uses SIMD vector type `f64` which (with the chosen ABI) requires the `sse` target feature" when a kernel-linked crate gains a float-bearing dep.
**Root cause:** i386 kernel builds Rust no-SSE (`-C target-feature=-sse,-sse2`, since CR4.OSFXSR isn't set until Phase 1). Any extern/C-ABI signature passing `f64`/`f32` by value is uncompilable; `rtl-strings` reusing `cnt_rust::core_rs::mbcp` transitively pulled in `floatcvt`/`scanf`.
**Fix:** Gate the float modules behind a default-on Cargo feature (`default=["float"]`, `cabi=["float"]` so generated cabi/difftests stay byte-identical), and have the kernel consumer set `default-features=false`. Resolver-v2: per-`-p` builds don't feature-unify `float` back on — always verify the i386 kernel via the cmake `-p` path. This is correct modularization, not dodging a regression.
**Source:** `feedback_cnt_rust_float_no_sse_i386_kernel.md`

### Screenshot looks solid black (346-byte PNG) — NOT a boot failure
**Symptom:** `qemu/<arch>_smoke.png` is exactly 346 bytes, solid black in every viewer, while the raw PPM has full color.
**Root cause:** ImageMagick 6.x `convert *.ppm *.png` collapses a uniform-color frame (e.g. blue desktop RGB(58,110,165)) to a 1-bit-colormap PNG; viewers render index-0 as black. The OS is rendering fine.
**Fix:** Use PIL, not ImageMagick: `Image.open(ppm_path).save(png_path, "PNG")` → 8-bit RGB. Diagnostic before assuming OS broke: dump raw FB bytes (`monitor xp/64xb <GOP_base>`) or open the PPM directly.
**Source:** `feedback_smoke_screenshot_imagemagick.md`

### post_shell_ready input/mouse smoke fails — KNOWN FLAKE, not a kernel bug
**Symptom:** A full `pytest test_smoke.py` reports `3 failed, 7 passed` with `KeyError: 'run...'` on `test_post_shell_ready_input`/`_mouse`; an immediate clean re-run of the same binaries is `10 passed`.
**Root cause:** Harness/QMP snapshot-loadvm timing artifact, NOT the kernel. Separately, `test_arm64input_delivery` is the bespoke `arm64input-rust` virtio-mmio driver's own WIP first-coverage test with uncommitted tree changes — a red there is not an rtl/ntoskrnl signal.
**Fix:** Do NOT treat input/mouse failures as a regression until reproduced by a clean full re-run (and/or `-k post_shell_ready_input` in isolation). `boot_and_serial` + `shell ready` milestone + screenshot passing = the OS booted to desktop. For arch-neutral pointer-free rtl/cnt structs: amd64+i386 10/10 plus a fault-free arm64 kernel log (grep `BUGCHECK|PANIC:|0xC000|Rtl.*Acl|exception`, absent + `shell ready` present) proves soundness; commit, don't revert.
**Source:** `feedback_smoke_input_flake.md`

### 0xF4(0x3) at `[EXPLORER] _tWinMain entered` on smss-csrss branches — RETIRED 2026-07-03 (verify-gone by absence)
**Status:** The `tWinMain_0xF4_race` known-flake catalog entry (`tests/qemu/cli/triage.py`) was **removed 2026-07-03**. It did not reproduce in **36 desktop-reaching windows-host TCG boots** (sequential + 3-wide contended) on a warm amd64 build, nor in 10 native-KVM boots (wrong regime — nested-virt in WSL, discounted). Under heavy 3-wide contention boots die *earlier* — at the service-startup storage stall (`ScmStartUserModeService(DcomLaunch/EventLog)` + `[GAP] read# gap=~9–20M us`) and the `nettest.exe` explorer-handoff race — **before** ever reaching the post-explorer window this 0xF4 lived in, so contention masks rather than surfaces it (see `nettest_explorer_race` + the amd64-net EventLog stall). No live crash could be caught to root-cause, and code audit of the suspect paths (`msgqueue.rs` send/wait/cleanup, `ke/thrdobj.rs` + `ps/state.rs` suspend/resume/freeze/thaw, the `SuspendCount`/`FreezeCount` accessors) found them all **faithful to upstream ReactOS** — consistent with the "byte-identical binaries flip" property below (a genuine inherited timing race, not a localizable translation bug). Removing the entry means a recurrence now **blocks as a plain `BUGCHECK 0xF4`** instead of being auto-retried — we *want* the next one to surface so it can be caught live. Re-add a signature only if a real recurrence is captured.
**Symptom (historical):** Intermittent `BUGCHECK 0x000000F4 (0x3, <obj>, <obj>, 0x805adbd4)` right after a flood of `msgqueue.split2.c:87 'Sending to Suspended Thread' Msg 121` + `painting.split2.c` dirty-windows warning + a `[CSRSRV] LPC_REQ … dispatch` line. Byte-IDENTICAL binaries give green one run, 0xF4 the next. The suspended-thread flood is a genuine crash-*correlate* (absent in clean boots): a receiver thread is momentarily suspended (created-suspended, not yet resumed) so `co_MsqSendMessage` drops the message — the same mechanism as the still-open win32k `DialogBoxParamW`/`WM_INITDIALOG`-never-dispatched race (§9.12).
**Root cause:** A win32k/explorer/csrss startup-ordering RACE, not a build regression — verified by `cmp`-identical win32k.sys producing opposite outcomes. PC `0x805adbd4` is the ntoskrnl `PspCatchCriticalBreak` bugcheck caller (arg1=3 ⇒ a critical *process* terminated via `PspTerminateProcess`+`BreakOnTermination`); arg2 is the dying critical process.
**If it recurs — how to catch it live:** repro is *windows-host TCG* (fast + cold timing), NOT native KVM (nested-virt, warm, suppresses it). Enough contention to stretch timing but not so much boots die pre-explorer — sequential `./vibeos-cli smoke --arch amd64 --repeat N` or a 2-wide parallel gate is the sweet spot; 3-wide over-starves into the storage-stall hang. The `[CRIT-TERM] Process=%p ImageName=%s` print (`ps/kill.rs`) names the dying critical process; symbolicate the faulting PC via the offline index (`qemu/build-index.sh amd64`). Re-run the SAME build ≥3× before concluding a change caused or fixed it; one green or one red proves nothing.
**Source:** `feedback_smss_csrss_harness_smoke_twinmain_flake.md`

### Serial logfile stays at 0 bytes through entire boot
**Symptom:** `-chardev socket,...,logfile=<log>` captures nothing; logfile sits at 0 bytes the whole boot when no client is connected.
**Root cause:** QEMU buffers socket-bound serial internally with `wait=off`; `logfile=` only emits bytes that actually flow through the socket, and nothing flows until a client connects.
**Fix:** For batch capture use `-serial file:<path>` (flush-as-you-go; ~15–20s cold-cache before first byte). If both file-capture and live-attach are needed, drive interactive control via QMP `human-monitor-command`, not the socket+logfile combo.
**Source:** `feedback_qemu_serial_chardev.md`

### KeBalanceSetManager spins ~40% CPU, amd64 boots take 15+ min (timer type)
**Symptom:** amd64 boot extreme slowness (15+ min vs i386 ~30s); BalanceSet thread repeatedly sampled in `KxMultiThreadWait → KeRaiseIrqlToSynchLevel`, returning immediately.
**Root cause:** Rust port of `KeBalanceSetManager` (`ke/balmgr.rs`) called `KeInitializeTimerEx(&mut PeriodTimer, 0)` = `NotificationTimer` (manual-reset; waits return immediately once signaled), instead of the C original's `SynchronizationTimer` (auto-reset, each wait consumes one signal) → tight spin.
**Fix:** Use `SynchronizationTimer=1` for periodic-tick consumers. When auditing timer/event translations, check the naked `0`/`1` `TimerType` (and `KeInitializeEvent` Notification-vs-Synchronization) against C source — default-zero is dangerous for type-tagged enums. Trigger an audit when a light-workload thread keeps showing up in wait backtraces.
**Source:** `feedback_balmgr_timer_type.md`

### Folding ke-inlines/ke-macros back into ntoskrnl-rust → triple-fault, 0 serial bytes
**Symptom:** Kernel triple-faults before any output — 0 bytes serial AND 0 bytes debugcon.
**Root cause:** `ke-inlines` (KSPIN_LOCK/queued-spinlock helpers) and `ke-macros` (wait-list/trap-frame/arch intrinsics) take pointers and do atomic ops; LLVM marks them captures(none) and optimizes away no-op writes across the within-crate CGU boundary. The workspace `lto=false`+`cgu=256` firewall stops cross-CRATE propagation but not cross-CGU within a single crate — the rlib boundary is the only thing that defeats it for ke.
**Fix:** Treat `ke-inlines` and `ke-macros` as permanently-isolated rlibs; never fold them back. New KE-style spinlock wrappers go in `ke-inlines` (or a new sibling rlib), NOT in `ntoskrnl-rust/src/ke/`. The other 15 inline crates (cm/common/io/ob/mm-*/se/ps) fold back fine — the firewall holds for those.
**Source:** `feedback_ke_inlines_load_bearing.md`

### Don't retranslate a `*_rust_shims.c` function — a stale private Rust impl may exist
**Symptom:** About to spend an hour translating a parked C function that's already mostly done in Rust.
**Root cause:** The multi-stage rustconv pipeline often left private `unsafe fn <NAME>` impls in the `.rs` that were never promoted to `#[no_mangle] pub unsafe extern "system"` — dead code (C wins the link) but a real, near-complete starting point. mm_sysldr_shims.c had three (`MiLookupDataTableEntry`, `MiSnapThunk`, `NameToOrdinal`) plus a partial `MiLoadImageSection`.
**Fix:** Before any work on `<file>_rust_shims.c`, `grep -n "fn <name>\b"` the corresponding `.rs` per function. For each hit, diff against the C body (watch for missing arch-specific blocks / forwarder-recursion / error paths — "complete enough for the test that exists"), promote to `#[no_mangle] pub unsafe extern "system" fn`, verify with `nm` (Rust=`T`, C obj=`U`), smoke, then delete the C body.
**Source:** `feedback_stale_private_rust_impls.md`

### Intermittent csrss self-terminate → 0xF4 (thread-hash bucket off-by-one)
**Symptom:** ~1/3 raw boots: csrss AVs and self-terminates → BUGCHECK `0xF4 (CRITICAL_OBJECT_TERMINATION)`. Any instrumentation (DbgPrint, gdb bp) masks it. Last serial LPC was a non-returning `BaseSrvCreateThread` (ServerId=1=basesrv, ApiId=1).
**Root cause:** `CsrInitializeProcessStructure` (`smss-csrss-rust/src/csrsrv/procsup.rs`) hardcoded `for i in 0..256` but `NUMBER_THREAD_HASH_BUCKETS=257` (prime). Bucket 256 stayed `LIST_ENTRY{null,null}`; the first thread with TID ≡ 256 (mod 257) — e.g. `0x100` — hashes there → `InsertHeadList` derefs null Flink. Load/timing-dependent → Heisenbug. Thread CREATION touches the hash table (don't misattribute to a consrv console handler).
**Fix:** `for i in 0..NUMBER_THREAD_HASH_BUCKETS as usize`. When auditing rustconv output, grep loop bounds and array sizes for literals that should be named constants (hash-table sizes, +1 sentinels).
**Source:** `feedback_csrss_thread_hash_bucket_offbyone.md`

### Script-rewriting `#if/#elif` chains → "unterminated conditional directive" tree-wide
**Symptom:** `unterminated conditional directive` errors across the whole tree after a scripted dead-branch removal.
**Root cause:** A regex/state-machine script that drops one `#elif` in the middle of a chain and converts the next `#elif` to `#if` leaves the original outer `#endif` off-by-one → unbalanced `#if`/`#endif`.
**Fix:** For per-arch ifdef cleanup, edit by hand, OR use a real C preprocessor parser (pcpp full tree walker) — never regex. Safe scripted patterns: standalone `#ifdef MACRO ... #endif` blocks, and compound-condition simplification (substitute `defined(_M_ARM)`→`0` in `#if` expressions then simplify) — these never change the `#if/#elif/#endif` count. Dropping branches and rewriting elifs to ifs is what breaks.
**Source:** `feedback_preprocessor_elif_chain_rewriting.md`

### Don't `git rm` a tree from a prune table — verify the build graph first
**Symptom:** A doc "Pruning" table promised ~14K dead LOC; most entries were actually in the build (reactx/dxg, ntddraw, displays/vga, diblib).
**Root cause:** Prune tables are written from `ls`/LOC counts and routinely lie. A tree can be runtime-unexercised yet required for compilation/link: `ntddraw` compiles straight into win32k.sys's SOURCE list; `USE_DIBLIB FALSE` un-builds the library but `bitblt_new.c`/`dib_new.c` `#include` its header UNCONDITIONALLY.
**Fix:** Before `git rm` any tree, check ALL of: (1) `grep -rn add_subdirectory --include=CMakeLists.txt` up the parents (commented=dead, active=keep); (2) whether its `.c` are in a module's SOURCE/target list (compiled into the .sys/.dll directly); (3) `grep -rn '#include.*<dir>/'` for unconditional header pulls from still-compiled files; (4) reconfigure+build+smoke as the final gate. If reality contradicts the doc, surface it and prune only the provably-safe subset — never delete to hit a LOC number; then fix the stale table. (Only two genuinely-orphaned `vga_new` trees survived this check → 6,284 LOC.)
**Source:** `feedback_prune_verify_build_graph.md`

---

## win32k flip-specific debugging

Lessons unique to translating win32k.sys (gdi/eng, gdi/ntgdi, gdi/dib, user/ntuser) C→Rust — where the link is the real gate, ABI is the #1 i386 killer, and cargo-check green routinely hides link-undefined or runtime ud2 folds.

### win32k flip links clean ×3 but explorer BUGCHECKs 0x1E (0xc000001d) at first CreateDC → NtGdiOpenDCW (folded ud2)
**Symptom:** Kernel→SMSS→csrss→win32k→winlogon→explorer all reach desktop init; first `CreateDC` traps STATUS_ILLEGAL_INSTRUCTION. `NtGdiOpenDCW`/`GreOpenDCW`/`DC_vInitDc` each carry a bare `ud2` (no panic-location push) right after `RtlEqualUnicodeString`, BEFORE the callee call. Whole fn body = `push ebp/mov ebp,esp/ud2`.
**Root cause:** NOT cgu-shift (the original hypothesis was wrong). rustconv declared 10 fns in bare `extern "C"` blocks across dclife/dcobjs/dcutil whose definitions are stdcall/NTAPI/FASTCALL (e.g. `EngpGetPDEV` declared "C", defined "system" in pdevobj.rs). One `#[no_mangle]` symbol with two calling conventions in one crate → mismatched-CC call is UB → LLVM folds the *caller* whole-body to ud2.
**Fix:** Declare each extern with the ABI matching its def (system for stdcall/NTAPI; dual-cfg fastcall/"C" for FASTCALL); verify every defined fn matches the win32k_sys bindgen-glob ABI. ud2 dropped 9→2; CreateDC stopped bugchecking. When a WHOLE fn body is a single ud2 (no ret/calls), suspect ABI-decl collision FIRST (1s grep: def ABI vs every extern decl) before cgu/captures(none). Solved at HEAD `787820400d9`.
**Source:** `feedback_win32k_dc_lifecycle_wave.md`

### win32k flip links clean ×3 but explorer HANGS on CreateDC path (no bugcheck, 175 KB serial vs 676 KB green)
**Symptom:** After the ud2 fix above, CreateDC no longer traps but the boot hangs (timeout) deep on the DC path; not a clean trap.
**Root cause:** `DC_bIsBitmapCompatible` (dcobjs.rs) indexed the C extern array `gajBitsPerFormat` directly. bindgen models that flexible/extern array as `[UCHAR; 0]` → rustc inserts a bounds check whose panic landing pad spun (`unconditional_panic` at RELEASE, manifesting as a hang not a clean trap). Same class: `StockObjects[i]` and any direct index of a bindgen `[_;0]`.
**Fix:** Use the raw-pointer helper `crate::dib_helpers::BitsPerFormat(i)` / `(*StockObjects.as_ptr().add(i))` — never index a bindgen `[_;0]` directly. One line. Solved at HEAD `20bc456b6d0` (codex cross-model find).
**Source:** `feedback_win32k_dc_lifecycle_wave.md`

### win32k cargo-check GREEN but win32k.sys LINK fails `@RECTL_bUnionRect@12` / `@REGION_X@N` undefined on ONE arch only
**Symptom:** Per-file cargo check passes on all 3 targets; the win32k.sys link fails with one undefined decorated symbol, often only on i386. The fn silently doesn't exist; only external C references it.
**Root cause:** Two mechanisms. (1) rustconv #[no_mangle] UNDER-PROMOTION — it emits correct dual-cfg `extern "fastcall"/"C"` shells but adds `#[no_mangle]` to only ~1 of ~37 exports (and forgets data exports like `pub static mut prgnDefault`). (2) Dangling-cfg orphans: rustconv leaves an orphan `#[cfg(x86)]`+`#[cfg(not(x86))]` pair with blank lines above a fn; Rust stacks ALL attrs onto the next item regardless of blank lines → fn gets contradictory cfgs → never compiled.
**Fix:** Promote ALL cross-file-referenced fns (`grep -rl '\bFn\b' win32ss` >0 → must be #[no_mangle]) AND data exports. Delete orphan cfg lines: `perl -0777 -pe 's/#\[cfg\([^\]]*\)\]\n\n(?=#\[cfg)//g'`; find them with `grep -nA1 '^#\[cfg' f.rs | grep -B1 '^[0-9]*-$'`. Decide exported-ness by cross-file refs.
**Source:** `feedback_win32k_ntgdi_geometry.md`, `feedback_win32k_dc_lifecycle_wave.md`

### win32k table-referenced fn (DibFunctionsForBitmapFormat) link-undefined or wrong cdecl decoration
**Symptom:** `_DIB_32BPP_TransparentBlt`/`DIB_DoRop@16` undefined at link, or a renderer fn that the C DIB function table references is missing/mangled.
**Root cause:** DIB_* fns are referenced by-name from the C `DibFunctionsForBitmapFormat` table but are NOT win32k.spec exports → resolver marks `is_exported=false` → LLM emits private mangled `unsafe fn`. `enforce_win32k_abi` only rewrites *already*-no_mangle stdcall/fastcall fns, so it doesn't help. Table entries are uniform cdecl (verified: i386 `_DIB_1BPP_PutPixel`, no `@N`); some fns get spuriously emitted `extern "system"` (stdcall @N).
**Fix:** Mechanical promotion script keyed off the dib.h declared-fn list → `#[no_mangle] extern "C"`. Audit every table-referenced fn with `nm` for stray `@N`. Consumers must declare the table locally with the REAL `[DIB_FUNCTIONS; 11]` layout (PutPixel-first, 10 fn-ptr fields) — the win32k_sys glob is `[DIB_FUNCTIONS; 0]` (0-stride OOB) and rustconv emits truncated/wrong-order local structs. Local `#[no_mangle]` shadows the glob with no E0428.
**Source:** `feedback_win32k_dib_cluster.md`

### win32k flip cargo-check GREEN but link fails "SIMD vector type f64 requires sse" on i386 kernel target
**Symptom:** Per-file cargo check green on all arches; failure first appears at the win32k.sys LINK. The hard `error` hides amid `clashing_extern_declarations` WARN notes (grep `^error`, not `note`).
**Root cause:** A Rust fn returning f64 BY VALUE (e.g. a `DEGREES_TO_RADIANS` helper) triggers SSE codegen on the no-SSE i386 kernel target — but only at `cargo build` codegen, NOT `cargo check` (sse-on i686 check target). extern "C" math (tan/cos/sin) is fine; only your OWN f64-returning fns break.
**Fix:** Make such helpers `macro_rules!`, not fns. FLOATOBJ is per-arch: i386 kernel = struct {ul1,ul2} with real ENGAPI stdcall ops (`extern "system"`, predicates FORCEINLINE→inline by hand); amd64/arm64 = FLOAT(f32) with all-macro ops → inline f32. Build a shared cfg-split `floatobj.rs`. `Ke{Save,Restore}FloatingPointState` are real on i386/amd64 but FORCEINLINE no-ops on arm64 → cfg(not aarch64) import + arm64 inline.
**Source:** `feedback_win32k_ntgdi_geometry.md`

### win32k i386 crashes from LLM-guessed calling convention (the #1 i386 crash cause) — solved by deterministic ABI overwrite
**Symptom:** i386-only stack corruption / link-undefined `_X@N`; amd64/arm64 fine (variance is x86-only).
**Root cause:** rustconv lets the LLM infer convention from APIENTRY/FASTCALL/NTAPI macros and it flips the per-arch dual-cfg coin. But convention is deterministic ground truth: i386 NTAPI/APIENTRY→stdcall(`_X@N`), FASTCALL→`@X@N`, cdecl→`_X`; amd64/arm64 uniformly cdecl.
**Fix:** Never let the LLM infer it. `abi_map.build_win32k_abi_map()` fuses `win32k-sys/bindgen_i386.rs` (per-fn conv, incl 398 FASTCALL fns) with `win32k.spec` (spec wins for exports): 1698 fns = 1193 stdcall/398 fastcall/107 cdecl. `postprocess.enforce_win32k_abi` (generalizes `fix_fastcall_exports`) OVERWRITES the LLM extern with the canonical dual-cfg; conservative on unknowns, idempotent. Gate = `python -m rustconv.verify_abi`. win32k-rust verifies clean (`feedback_ntoskrnl_rust_abi_log` — the 65 remaining ABI mismatches are all in ntoskrnl-rust, NOT a win32k blocker).
**Source:** `feedback_win32k_abi_enforcement.md`, `feedback_ntoskrnl_rust_abi_log.md`

### win32k i386 NO symbol + "expected item after attributes" — cfg-blind dedup reverted enforce_win32k_abi
**Symptom:** i386 gets no symbol for an exported fn; build error "expected item after attributes"; `enforce changed: True` logged yet output silently reverted.
**Root cause:** `postprocess.fix_duplicate_fn_defs` was cfg-BLIND. `enforce_win32k_abi`/`fix_fastcall_exports` intentionally emit TWO complementary-cfg defs (`#[cfg(x86)] extern "<conv>"` + `#[cfg(not)] extern "C"`) — mutually-exclusive per-arch builds, NOT duplicates. Old dedup keyed only on fn NAME, deleted the x86 body, and (removal span started at `#[no_mangle]`, not the cfg line above) left a dangling `#[cfg(x86)]` with no item. Also latently broke every ntoskrnl fastcall export.
**Fix:** Key dedup by `(name, cfg-guard-line-above)`; absorb the preceding `#[cfg(...)]` into the removal span. Order matters: a later generic cleanup can revert an earlier authoritative pass. Caught BEFORE bulk spend by the eng/sort.c proving-slice (24 LOC, $0.03). Verify postprocess fixes with a synthetic reproduction (zero API cost).
**Source:** `feedback_postprocess_cfg_dual_def_dedup.md`

### win32k mid-size leaf links undefined — rustconv silently dropped 25-50% of functions
**Symptom:** Link-undefined for a fn that exists in the C file; no error in the translate-stage log — the fn just isn't in stitched output.
**Root cause:** rustconv `translate` (function-by-function) sometimes emits FEWER fns than the file has. Observed: float.c 1/2, mdevobj.c 1/4, semaphor.c 7/9.
**Fix:** Defer the file and re-translate (~$0.05). Instrument the stitcher to ASSERT `len(translated) == len(scan.functions)` and fail-loud. (Mode 3 of the Phase 2a quality lessons; device.rs's stubbed-bodies is the same family.)
**Source:** `feedback_win32k_phase2a_quality_lessons.md`

### win32k link-undefined `_DRIVEROBJ_AllocObjectWithHandle@0` — GDIOBJ_* family macros translated as externs
**Symptom:** Link fails on `_DRIVEROBJ_…@0`, `_BRUSHOBJ_…`, etc. — symbols that don't exist as functions.
**Root cause:** `DRIVEROBJ_AllocObjectWithHandle()`/`DRIVEROBJ_UnlockObject(pdo)` etc. are macros (driverobj.h) expanding to `GDIOBJ_AllocObjWithHandle(GDI_OBJECT_TYPE_…, sizeof(…))` / `GDIOBJ_vUnlockObject((POBJ)pdo)`. rustconv emitted them as `extern "C" { fn DRIVEROBJ_…; }`. Same hazard for every `DEFINE_GDIOBJ_FUNCS()`-style family (BRUSHOBJ_/PALETTE_/SURFACE_/REGION_/PATH_/DC_) — essentially every ntgdi file. Same class: FORCEINLINEs/macros imported as phantom externs (MASK1BPP, ROP4_USES_*, RECTL_lGetWidth, NT_SUCCESS, RECTL_bPointInRect, RGB, GDI_HANDLE_*, RTL_CONSTANT_STRING). Also `ExAllocatePoolZero` (FORCEINLINE in xdk/exfuncs.h, real symbol not in ntoskrnl.spec → not importable) → undefined `_ExAllocatePoolZero@12`.
**Fix:** Inline-expand the macro to the underlying GDIOBJ_*+type-tag form. Factor recurring FORCEINLINEs/macros into ONE shared `dib_helpers.rs` (pub(crate), copying bitblt.rs's proven bodies); replicate ExAllocatePoolZero as `ExAllocatePoolWithTag + write_bytes`. Teach the system prompt to recognize these families (postprocess upgrade would 2-3× yield).
**Source:** `feedback_win32k_phase2a_quality_lessons.md`, `feedback_win32k_dib_cluster.md`

### win32k typecheck `expected ntkrnl_sys::ddk::_X, found win32k_sys::win32k::_X` (cross-crate type-namespace collision)
**Symptom:** E0308 type mismatch at a call into ntkrnl_sys (KeQueryPerformanceCounter, KeRestoreFloatingPointState, NtCurrentTeb) even though the type is layout-identical; or E0659 ambiguity on every primitive (ULONG/EPROCESS/NTSTATUS).
**Root cause:** Both crates bindgen LARGE_INTEGER/KFLOATING_SAVE/TEB/etc. into their own namespaces → Rust sees them as DISTINCT types. The LLM emits a blanket `use win32k_sys::win32k::*;` that shadows ntkrnl_sys versions; adding a second `ntkrnl_sys::{base,ddk}::*` glob makes every shared primitive ambiguous and resolves shared structs to the wrong variant.
**Fix:** Use ONE glob `use win32k_sys::win32k::*` (the win32k-internal surface — PBYTE/SORTCOMP/_WND/Int*/Gre*/co_* lives in `win32k_sys::win32k`, NOT ntkrnl_sys); keep NAMED ntkrnl_sys imports only, cast `as *mut _` at the FFI boundary (layout-identical). `postprocess.fix_win32k_crate_path` + `normalize_win32k_imports` strip clashing globs. bindgen double-prefixes constified enums (`GDIObjType_GDIObjType_SURF_TYPE`) → `win32k-sys/gen_enum_aliases.py` emits ~3.5K de-prefixed `pub const` aliases/arch.
**Source:** `feedback_win32k_phase2a_quality_lessons.md`, `feedback_postprocess_cfg_dual_def_dedup.md`

### win32k 16bpp NICEPIXEL E0615 field-vs-method / E0740 union-not-Copy / E0381 partial-write
**Symptom:** `DstPixel16.col.red = X` errors E0615; `#[repr(C)] union NICEPIXEL*` errors E0740; uninitialized union local errors E0381.
**Root cause:** rustconv emits bitfield ACCESSOR METHODS (`red()`/`set_red()`) for the packed-u16 16bpp col struct but then writes as if fields. 24/32bpp dest is NICEPIXEL32 (real UCHAR fields → no surgery). rustconv also duplicates NICEPIXEL32/_col with a stray `#[inline]` on the union.
**Fix:** Rewrite 16bpp reads→`.col.red()`, writes→`.col.set_red(X)`; add `#[derive(Clone,Copy)]` to every union + col struct; delete the dup def; init union locals `= core::mem::zeroed()`. Cast u8 BLENDFUNCTION fields `as u32` before AND with AC_SRC_* consts; pool sizes `as SIZE_T` (NOT usize — distinct on i386). i386 GCC-asm fast paths (HLine/VLine/ColorFill under `#if _M_IX86 && !_MSC_VER`) are mistranslated wholesale → delete the cfg-split, keep the portable loop on all arches (smoke is the arbiter).
**Source:** `feedback_win32k_dib_cluster.md`

### win32k `#[no_mangle] fn XFORMOBJ_iSetXform` emits wrong symbol; callers ref EXFORMOBJ_*
**Symptom:** i386 link-undefined for the E-prefixed name; the win32k file's .h `#define`s the public name to an internal one.
**Root cause:** `#define XFORMOBJ_x EXFORMOBJ_x` (xformobj.h) → the real C symbols are E-prefixed. A `#[no_mangle] fn XFORMOBJ_iSetXform` emits the wrong symbol.
**Fix:** Rename the 7 exported fns to `EXFORMOBJ_*`. MX_*/UpdateAccel have no #define → stay. Audit any win32k file whose .h #defines the public name to an internal one.
**Source:** `feedback_win32k_ntgdi_geometry.md`

### win32k bindgen anonymous-member fields land in __bindgen_padding_N (BRUSHBODY/PERUSERSERVERINFO)
**Symptom:** Field access on `_BRUSH` (lWidth/eWidth/ulPenStyle) or SERVERINFO (argbSystem/ahbrSystem) fails — bindgen dropped the names into `__bindgen_padding_N`.
**Root cause:** bindgen flattens anonymous embedded structs (anon BRUSHBODY in _BRUSH, anon PERUSERSERVERINFO in SERVERINFO) into padding.
**Fix:** Reach them via a `#[repr(C)]` view built from REAL win32k_sys types (so per-arch offsets are correct — eWidth/pointers differ 4↔8) placed after BASEOBJECT (size_of), or `addr_of_mut!(x.__bindgen_padding_0) as *mut _RealStruct`. Shared `brushbody.rs` generalizes engbrush's view. Static-inline callee chains (IntLPtoDP→INTERNAL_LPTODP→INTERNAL_APPLY_MATRIX) have no symbol → inline the whole chain in the caller. DEVMODEW.dmDisplayFixedOutput is TWO anon-union levels deep (`.__bindgen_anon_1.__bindgen_anon_2.…`) — grep & count `__bindgen_anon_` prefixes before fixing access.
**Source:** `feedback_win32k_ntgdi_geometry.md`, `feedback_win32_user_mode_substrate_widening.md`

### INTEGRATED+FAILING: device.rs enabled → i386 hangs pre-`shell ready`, amd64 0x1E IN_PAGE_ERROR (resolved via PTE-walk fix)
**Symptom:** Enabling device.rs (with 14 DEFERRED-STUBs, then 3 boot-critical fns resurrected) made i386 hang before `shell ready` and amd64 BUGCHECK 0x1E (0xc0000006 STATUS_IN_PAGE_ERROR) right after PnP processes std-VGA `DISPLAY\RHT1234`; faulting code in an unsymbolized `.data` gap. cargo+link clean ×3 — kept INTEGRATED+FAILING in tree per AGENTS.md.
**Root cause:** NOT a pure cgu-shift (the hypothesis). `MiPteToPde` in ntkrnl-sys/src/mm.rs was mistranslated as `MiAddressToPde(pte)` for ALL arches, but amd64/i386 must be `MiAddressToPte(pte)` (self-map recursion); arm64 correctly stays MiAddressToPde. Wrong PDE pointer faulted in MM page-table code on the PnP path.
**Fix:** Per-arch `MiPteToPde`/`MiPteToPpe`/`MiPteToPxe` + `write_volatile` hardening across mm/avl,marea,section + pnpmgr/* + pool. Verified 3/3 GREEN, merged `96bc3b6dfa9`. Reusable device.rs machinery: per-arch extern ABI for Reg*/InitDisplayDriver (i386 stdcall else "C"); inline impls for RtlStringCchCopyW/RtlStringCbCopyNW (ntstrsafe.h static __inline, no symbol); stdcall/system fn-ptr `Some(transmute::<usize,_>(F as usize))` for PVIDEO_WIN32K_CALLOUT; raw-dylib gates `any(x86,aarch64)` for ExEventObjectType/MmSystemRangeStart (amd64 plain extern — raw-dylib bugchecks 0x6B).
**Source:** `feedback_win32k_device_rs_resurrection_state.md`, `feedback_win32k_wave2_deferred_device_ldevobj.md`

### INTEGRATED-but-DEFERRED: ldevobj.rs links clean ×3 but i386 NtUserInitialize returns 0xc0000001 (root cause TBD)
**Symptom:** cargo check ✓ + link ✓ (after fixing `LDEVOBJ_bBuildDevmodeList` extern decl AND def both `extern "system"`→`extern "C"`); i386 NtUserInitialize returns STATUS_UNSUCCESSFUL with ldevobj.rs replacing ldevobj.c. All LDEVOBJ_* bodies look faithful. Still unresolved.
**Root cause:** Suspect surface — `LDEVOBJ_bEnableDriver` indirect call through `Option<unsafe extern "C" fn() -> LONG_PTR>` (PFN/DRVFN union layout off vs bindgen); `LDEVOBJ_pvFindImageProcAddress` ULONG_PTR arithmetic on `__ImageBase` (if resolution differs from C, addresses point wrong); EngLoadImage/EngUnloadImage stdcall-on-i386 ZwSetSystemInformation interactions.
**Fix (resume path):** Boot i386 with `qemu/start.sh i386 --gdb`, break NtUserInitialize→UserInitialize→InitVideo, step Rust vs C side-effects, particularly pldev->pGdiDriverInfo population + pfn/DRVFN bindgen anon-union accessors. Note: device.rs's `LDEVOBJ_bBuildDevmodeList` extern decl was `extern "system"` but the C decl has no CC attr = cdecl (i386 symbol undecorated `_LDEVOBJ_bBuildDevmodeList`).
**Source:** `feedback_win32k_wave2_deferred_device_ldevobj.md`

### win32k USER critical section (618 sites) — over-engineered as a combined-guard problem
**Symptom:** A RAII translation pass for `UserEnterExclusive/UserEnterShared/UserLeave` looks like the ntoskrnl-class 2-level CR+ERESOURCE nest (with inverse release order — release ERESOURCE then leave CR).
**Root cause/finding:** ReactOS ALREADY factored the 2-level nest into the primitive pair (UserEnterExclusive/UserLeave in ntuser.c), exactly like ntoskrnl-rust's `ex/time.rs ExAcquireTimeRefreshLock` (no combined guard exists). So the primitive translates ONCE as plain direct calls, and the 618 callers collapse to a single-level `PfnLockGuard`-shaped problem — no novel machinery. IRQL/spinlock is a verified NON-issue in win32k proper: all 20 KeRaiseIrql sites are in videoprt (separate driver) + gdidbg.c (DBG-only); win32k.sys is PASSIVE_LEVEL.
**Fix/method:** Before assuming ntoskrnl patterns transfer, verify the IRQL/spinlock surface is real in THAT subsystem; check whether the codebase already factored the nest into a named primitive; research the proven ntoskrnl-rust guard + system_prompt rule first (usually shrinks the plan). Pass 2b = `user_lock_inventory.py` (clone of gdi_lock_inventory.py); classifier gotcha: conditional-enter test MUST precede the leave-then-reenter reentrancy test (sibling switch arms trip a naive straddle check).
**Source:** `feedback_win32k_user_critsec_pretranslation.md`

### win32k *-sys / user-mode-sys crate: 118 E0659+E0308 errors from a glob re-export
**Symptom:** First link of a user-mode file (e.g. concfg/settings.c) reports ~118 errors — E0659 ambiguity + E0308 mismatch on every HKEY/HANDLE/LPWSTR/Reg* interaction.
**Root cause:** windef.h types (HKEY/HANDLE/LPWSTR/MAX_PATH/REGSAM) are bindgen'd by BOTH the parent (ntdll-sys via win/base.h) and child (winsrv-sys via winsrv.h/consrv.h). A glob `pub use ntdll_sys::nt;` re-publishes ntdll's HKEY under the child namespace; `use child_sys::nt::*; use child_sys::child::*;` then ambiguates every site.
**Fix:** Broaden the child's FN_ALLOW to be a true superset of the NT/CSR/SM/BASE families (`(Nt|Zw)…|Rtl…|Ldr…|Dbg…|Csr…|Sm…|Sb…|Base…`) so NT calls fall out of the child bindgen; drop the glob re-export; namespace ntdll's hand-written inlines + STATUS_* under a NARROW `pub mod nt { pub use ::{…specific}; }`; translate with ONE glob `use child_sys::child::*`. Collapsed 118→7 (trivial `ERROR_SUCCESS as i32` casts).
**Source:** `feedback_user_mode_sys_superset_one_glob.md`

### win32k/Win32 user-mode flip: >50 E0425 (cannot find fn) then E0659 spam after widening FN_ALLOW
**Symptom:** Translating a Win32 GUI/USER/GDI file: first cargo check has >50 E0425; widening FN_ALLOW then produces E0659 ambiguity spam.
**Root cause:** NT/Rtl whitelist misses the ~80 Win32 verb roots (Empty/Select/Drag/Peek/Enable/Window/Cursor/Font/Mouse/Console/…). Over-widening grabs names that already exist as hand-written inlines/intrinsics (Enter/Leave/Initialize/_Interlocked*/RGB/MAKE*/LOWORD/HIWORD/MAKEINTRESOURCE).
**Fix:** Widen FN_ALLOW verb prefixes in generate_bindings.sh for E0425; EXCLUDE the already-hand-written names from FN_ALLOW (upstream, not per-file) for E0659. bindgen drops macros-as-expressions, `((HWND)-1)` consts, `static inline` bodies, and `L"..."` literals → hand-roll them in a sibling `helpers.rs` (canonical: winsrv-rust/src/consrv/frontends/gui/helpers.rs). WPARAM is c_uint on i386 / c_ulonglong on amd64+arm64 → write bit-extract helpers as `<T: Into<u64>>` to avoid per-arch casts. In cohab dirs, `sed -i 's/^unsafe fn /pub unsafe fn /'` every .rs (bindgen Rust→Rust calls bypass the C #[no_mangle] path → private fns invisible to siblings).
**Source:** `feedback_win32_user_mode_substrate_widening.md`

---

## Translating a shared C runtime library (Rtl-class API bodies)

Bugs that recur whenever you flip a C library whose functions are thin plumbing over syscalls, PEB/TEB accessors, and output-pointer contracts — i.e. any surface where the *shape* is "chain of `Zw*`/accessor calls" rather than a pure algorithm. A single flip routinely lets a *previously-unreached* code path run for the first time on a given arch, so the flip surfaces latent bugs in code that was already committed. Expect the flip to be correct and the downstream to be where the fault is.

### Symptom → cause → fix catalog

| Symptom | Root cause | Fix |
|---|---|---|
| **BUGCHECK 0x1E, write to VA 0** (`CR2=0`, page-table walk shows VA 0 `P=0`) after a flip reaches process/env init | An **accessor shim was left as a NULL stub** during translation. The C original does pure pointer arithmetic on a *child* structure's user-VA (`&((PPEB)base)->Field` — no deref, valid to compute in kernel), but the Rust/shim returned `NULL`, so the next `ZwWriteVirtualMemory` wrote to address 0. Uncatchable under DUMMY_PSEH. | Implement the real pointer arithmetic on the bindgen struct; never leave a "returns NULL for now" accessor stub in a booted path. Grep flipped libsupp/accessor shims for `-> ... { NULL }` / `return null_mut()`. |
| **BUGCHECK 0x1E, new thread's user `Rsp`/`Sp` = a kernel-stack address** (trap frame: entry `Rip` correct ✓, `Rsp` points into kernel stack ✗) | Passed **the address of a struct instead of a field of it**. A C API param is mistyped as a pointer (`PINITIAL_TEB`) but the callee actually consumes it as `(ULONG_PTR)StackBase`; the faithful C caller passes `teb.StackBase`, but the translation passed `&teb`. The struct address is then used as a stack pointer → AV on first user-APC probe. | Match what the C *caller* passes (the field), not what the *signature* nominally types. When a param's declared type disagrees with its use, translate the use. |
| **BUGCHECK 0x4C / 0x6B, `c000003a OBJECT_PATH_NOT_FOUND`** during smss/critical-process init; a registry `REG_EXPAND_SZ` path stays literally `%SystemRoot%\...` unexpanded | A **"NULL = use current-process default" contract was dropped**. The C function forwards a NULL `Environment`/context argument to a lower call that resolves NULL → `NtCurrentPeb()->ProcessParameters->Environment`. The translation re-implemented the lookup with a pure helper and mapped NULL → empty slice → every expansion silently no-ops. Historically misattributed to "upstream PnP/IDE-binding bug." | Preserve the NULL-means-default sentinel: NULL → resolve via the real PEB/context accessor, not `&[]`. Audit every translated function whose C body passes a possibly-NULL pointer straight through — the NULL has *meaning* downstream. |

**Cross-cutting lesson:** all three bugs are *latent in code that already links and passes earlier smoke*. A flip that "advances the boot marker and then crashes" is usually correct — it just reached virgin territory. Diagnose the crash site, not the flip.

**Stale fat-archive trap (build-system, same family):** if a flipped `.c` "links clean" suspiciously easily, suspect a zombie object. A cmake `ar` rule that *appends without truncating* leaves `lib*.a` accumulating `.obj`s of every previously-flipped file, which keep silently satisfying symbols that should now be undefined. Confirm with `rm lib*.a && ninja` then `ar t lib*.a | grep <should-be-gone>`.

### Byte-diff verification methodology

For **pointer-free / arch-neutral** translations (string, SID, ACL, SD, security, atom-table classes), don't rely on smoke — prove the Rust is bit-identical to the genuine C:

- **Compile the *real* C as a reference TU** inside a host test crate's `build.rs` (not a hand-copy — the actual source file), applying a **SAL-neutralization block** (`#define`-away `_In_`/`_Out_`/…) and a `scrape_protos()` pass so call-before-definition C still compiles.
- **Fuzz the Rust `#[no_mangle]` output byte-for-byte against the C** across generated inputs. Keep the pure logic in a `forbid(unsafe_code)` `core_rs`-style module so it's also Kani-verifiable; the `unsafe`/asm/FFI lives only in the thin `cabi` shim.
- **When byte-diff is the wrong shape** — pure syscall plumbing (chains of `Zw*`), stateful allocators, anything with side effects rather than a computed output — smoke is the only net. Don't force a byte-diff harness onto it; ship the flip and let boot exercise it. (Heap-class allocators get their *own* behavioral/property/Kani track, never byte-diff.)

Rule of thumb: **byte-diffable ⇔ pointer-free pure function.** If the function computes an output buffer from inputs with no VM/handle/PEB side effects, byte-diff it. Otherwise, smoke it. (A ready-made harness for this lives in `rust-in-kernel.md` → "Differential-fuzz parity harness".)

## Translating a COM DLL with a private header (ole32/oleaut32-class)

Bugs that recur whenever you flip a **user-mode COM DLL** — i.e. a surface whose shape is
"objects with vtables, a module-private header full of types and `static inline`s, and an
on-disk/wire format". Learned flipping all 29 TUs of ole32 (2026-07-12); every item below
is a bug that **compiled and linked cleanly** and would have shipped.

### The three things the translator silently omits

`rustconv` translates *functions*. It does not translate headers, and it does not translate
file-scope data. Before you start, grep the C and treat these as a **separate authoring
task**, budgeted up front:

| Missing | How to find it | Consequence if you don't notice |
|---|---|---|
| The module's private header | `dll/.../<mod>.h` — types, the class hierarchy, `static inline` dispatchers, on-disk offsets | The Rust *references* them (storage32.rs did, ~130×) as `extern` symbols that link to nothing. Looks like an import problem; is a missing-translation problem. |
| Every `static const ...Vtbl` table | `grep -nE '^static const .*Vtbl' <mod>.c` | The objects have **no methods at all**. 23 tables were missing from ole32's Tier-D. |
| An export's attributes | the `.spec` file is ground truth | `StgCreateDocfile` came back a module-private *Rust-ABI* fn → `_StgCreateDocfile@16` simply absent from the DLL. Link error, at least — but only because a `.def` forced it. |

### Symptom → cause → fix catalog

| Symptom | Root cause | Fix |
|---|---|---|
| **Stack corruption on the first call to any COM method — i386 only** | Every vtbl slot emitted as a plain `unsafe fn` (the **Rust** ABI) instead of `STDMETHODCALLTYPE`. `extern "C"` links on amd64/arm64 and is wrong on i386, where COM methods are `__stdcall`. ~130 slots across ole32's Tier-D. | Wrap every vtbl method in a `stdmethod!` macro emitting `extern "stdcall"` on x86 / `extern "C"` elsewhere. Note `extern "system"` does NOT unify with bindgen's vtbl slot types — bindgen spells the ABI concretely per-arch. |
| **A `match` arm silently swallows every input** | A **missing constant** turns a `match` arm into an irrefutable **binding**. rustc only whispers `unreachable pattern`. `StgOpenStorageEx` treated every `stgfmt` as `STGFMT_ANY`; datacache matched a `u16 cfFormat` against `DWORD` constants so *every* format took the first arm. | Never hand-transcribe a constant — `tools/harvest_consts.py` resolves it from the header the C compiles against and cites `file:line`. `verify_match_consts` in `vibeos-cli check` is the tripwire. |
| **Behaviour is exactly inverted / a flag never fires** | A constant transcribed from memory with the **wrong value**. Seven in one pass: `STG_E_INVALIDHEADER` that was really `STG_E_INVALIDHANDLE`; `DISCARDCACHE_SAVEIFDIRTY` as `1` when the header says `0`; `PROPSETFLAG_NONSIMPLE` tested as `& 2` (that is ANSI; NONSIMPLE is 1); `PROPSETFLAG_UNBUFFERED` as `& 0x1000` (real value 4 — auto-commit never fired). **No compile error, no link error.** | Same: harvest, never type. `scan_inline_flag_literals` catches the `0x1000 /* STGM_WRITE */` shape where the comment and the value disagree. |
| **Heap corruption a long way from the write** | `core::ptr::write_bytes(ptr, v, n)` / `copy_nonoverlapping` count **ELEMENTS, not bytes**, but the C's `memset`/`memcpy` count bytes. A `*mut ULONG` with `n = sizeof(array)` writes 4× the array. Also seen: a 64-*byte* `memcpy` that became 64 `WCHAR`s. | Cast the pointer to `*mut u8` whenever the count came from `sizeof`. `scan_silent_failures`' `WRITE_BYTES_SIZEOF` is the tripwire (suppress a verified FP with `// nolint: silent-failure (...)` **on the finding line**). |
| **Every record written to disk is corrupt, but the code looks right** | A `#pragma pack(N)` struct translated as plain `#[repr(C)]`. ole32's `meta_placeable` became 28 bytes instead of 22. | Check the C for `#pragma pack` around every struct that touches a stream/file, and use `#[repr(C, packed(N))]`. Wire-format structs deserve a `size_of` assertion. |
| **A loop never iterates, or skips its first element** | The C's list macros (`LIST_FOR_EACH_ENTRY`, wine's intrusive `struct list`) emitted as *function calls*, or hand-expanded wrong — `(*list.next).next` starts at the second element. `deleteStorageContents`' walk was rewritten into a loop that never iterates; `deleteStreamContents`' body was dropped entirely (→ use-after-free of the parent storage). | Provide the macros as real Rust macros and re-derive every hand-rolled walk from the C. Beware: a macro whose link-advance is *after* the body cannot host a `continue` — the C body may rely on that. |
| **An `extern "C"` decl that links but calls wrong on i386** | An item in a module **shadows a glob import**. A file that declares `extern "C" { fn Foo(); }` locally will use *that*, not the prelude's correct `extern "system"` one — undecorated on i386, where the real symbol is `_Foo@N`. | Strip local extern blocks at integration and route every call through one shared, correctly-typed surface. `scan_cc_mismatch` is the tripwire. Note the inverse too: `DECLSPEC_HIDDEN` helpers declared **without** `WINAPI` really are cdecl — don't "fix" them to stdcall. |
| **Two globs, one name → E0659 the day someone calls it** | A symbol declared `extern` in a shared "these still live in C" module AND defined by a flipped module. Both reachable through the prelude's globs. It stays *latent* until some file calls the name unqualified. | When you flip a TU into the crate, **delete its decl from the exports module**. 8 of ole32's were already latent before Tier-D made them fire. |
| **`extern "thiscall"` is unstable and the C is an asm thunk** | i386 C++ `this`-convention adapters (wine's `DEFINE_STDCALL_WRAPPER`). Calling the slot with the C ABI instead — `this` on the stack where the callee expects `ecx` — corrupts the stack every call. | Write the thunk as a **naked function** (`#[unsafe(naked)]` + `naked_asm!`, stable since 1.88) matching the C's asm instruction for instruction. Do not silently degrade to the wrong ABI. See `ring/stg_prop.rs`'s `Allocate_PMemoryAllocator`. |

**Cross-cutting lesson:** in a *kernel* flip the failure mode is a BUGCHECK — loud, and the
smoke harness catches it. In a **COM DLL** flip the failure mode is a function that returns
the wrong HRESULT, writes a subtly wrong byte to disk, or takes the wrong `match` arm. The
boot smoke will be **green**. The lint gate (`verify_match_consts`, `scan_cc_mismatch`,
`scan_inline_flag_literals`, `scan_silent_failures`) is doing most of the real work here —
run `vibeos-cli check` and take its 0-NEW gate seriously; on this flip it caught four real
bugs *after* the crate compiled clean and the OS booted on all three arches.

## 9. Kernel crash-hunting techniques (live BUGCHECK / hang / heap hunts)

The *how-to-hunt* layer: techniques for chasing a live kernel crash (BUGCHECK, hang, heap
corruption, wild write) in the QEMU boot, as opposed to the symptom→cause→fix categories
above. Companion toolkit *design* lives in `kernel-debugging-infra.md`; use that toolkit,
these are the techniques on top. **Append new crash-hunting techniques here**, not in
per-blocker memories (that's how the i386 desktopheap notes got lost).

### 9.1 KVM vs TCG — breakpoints and watchpoints only work under TCG

**KVM does NOT fire gdb breakpoints OR hardware watchpoints.** Under the default
KVM-accelerated boot, `break`/`hbreak`/`watch` never stop. Switch to TCG:
```bash
VIBEOS_NO_KVM=1   # qemu/start.sh honors this — drops -accel kvm, runs -accel tcg
```
- `hbreak <addr>` and `watch *(u32*)<addr>` work under TCG. Prefer `hbreak` over `break`
  on session/user pages not yet mapped.
- TCG is **slow** and a `watch`+`continue` can flake *across the bugcheck halt*. When a
  watch won't catch it, fall back to in-kernel instrumentation (§9.3).

### 9.2 Symbolication math (i386) — the Rust-in-ntoskrnl +0x291000 shift

Rust code in `ntoskrnl.exe` is runtime-shifted **+0x291000 vs its file VMA**; C is not.
- **C funcs:**   `file_VA = runtime - 0x7fe00000`
- **Rust funcs (heap/rtl/etc):** `file_VA = runtime - 0x7fe00000 - 0x291000` (= `nm` with `-o 0x80091000`).
- **One-shot:** `qemu/addr2sym.sh i386 --reloc 0x80091000 <runtime_addr>` resolves both Rust + C (ntoskrnl reloc base `0x80091000`, RB `0x80491000`).
- gdb `info symbol -o 0x7fe00000` only resolves early **C** `.text` — mis-resolves Rust.
- **win32k.sys is in SESSION space (~`0xf74xxxxx`), not `0x80xxxxxx`** (e.g. base `0xf73ef000`, `.text` `0xf73df000`; **re-derive after any win32k rebuild**). Stack-scan filters that only accept `0x80200000..0x80900000` MISS every win32k frame — also include `0xf7000000..0xf7700000`.

### 9.3 In-kernel free-list / heap validation instrumentation (beats watchpoints for heap desync)

When a heap free-list is corrupted by a stray write and a watch won't catch it, add a
**capped, scoped, heap-relative** validator that walks the free list per alloc/free:
- **Capped:** `static mut PRINTED: u32`, bail after ~20 (don't flood serial).
- **Kernel-heap-only:** gate `heap >= 0x80000000` — a naive `flink < 0x80000000` floods on user-mode heaps `0x130000`/`0x230000` (also in ntdll's *separate* Rust heap copy, §9.6).
- **Heap-relative sanity:** valid intra-heap link satisfies `flink.wrapping_sub(heap) < 0x4000000`.
- **Name the culprit:** on desync, stack-scan from ESP for return addresses in `0x80200000..0x80900000` *and* win32k `0xf7000000..0xf7700000` (§9.2).

Place at alloc-ENTRY/EXIT + free-ENTRY/EXIT, labeled (`a110`/`A11E`/`F1EE`/`F1FE`). If
**only alloc-ENTRY** trips, corruption isn't introduced at any heap exit → external stray
write to an already-free block, not a heap-impl bug. (This exonerated the Rust heap in the
i386 desktopheap crash — it's a win32k UAF-write.)

### 9.4 Hardware watchpoint, armed after the page is mapped

A session/desktop-heap address isn't mapped at boot. Sequence it:
1. `hbreak` a function that runs *after* the page maps but *before* the corruption (e.g. `hbreak <NtUserCreateWindowEx_runtime>`; first hit).
2. Then `watch *(u32*)0xbc9513a0 if ((*(u32*)0xbc9513a0)>>24)!=0xbc` and `continue` — condition on the **corruption signature** (flink's high byte no longer the heap page, or a known constant the bad write carries, e.g. `(*(u32*)ADDR>>16)==0x0505`), not the address alone.

### 9.5 Trap-frame decode (i386) + Rust-release stack walking

- BUGCHECK **0x1E**: param **P3 = KTRAP_FRAME ptr**. i386 offsets: `Esi@+0x58`, `Edi@+0x54`, `Eip@+0x68`, `Ebp@+0x60`.
- **Rust release omits frame pointers** → EBP-chain walk fails, don't trust `bt`. **Scan the raw stack upward from fault ESP** for values in the kernel/win32k `.text` ranges (§9.2) — those are the call chain.

### 9.6 Multi-copy staleness — rtl/heap exists in more than one archive

`rtl-heap`/`heap.rs` links into **both** `librtl.a` (ntoskrnl) **and** a separate ntdll
copy. A source edit that "doesn't take" is a stale archive:
```bash
rm build-i386/sdk/lib/rtl/librtl.a                              # ntoskrnl copy
rm -rf sdk/lib/rust/target/ntdll-cargo-rtl_rust-rustlib         # ntdll copy
touch sdk/lib/rust/ntdll-cargo/build.rs
```
See `feedback_stale_fat_librtl_masks_rust` + §6 (cohab/cold-link) for the general thin-archive trap.

### 9.6b ⚠️ The amd64 (cargo-driven) kernel does NOT relink on a `.rs` edit — `ninja livecd` silently ships a STALE kernel

The single most dangerous staleness trap, and it **invalidates any amd64 kernel-Rust
debugging done without checking for it.** `ntoskrnl.exe`/`ntkrnlmp.exe` are built by a
ninja **CUSTOM_COMMAND** that shells out to `cargo build` (target dir
`build-amd64/ntoskrnl/rust-kernel-target`, crate `ntkrnlmp-cargo`). Ninja's dep graph
for that command does **not** list the `.rs` source files — so editing
`ntoskrnl-rust/src/**.rs` and running `./vibeos-cli build --arch amd64` (even
**`--clean`**) **does not rerun the command** → the kernel on the ISO is whatever it
was last forced to. A 2026-06-14 session burned multiple smoke cycles (and a prior
session reached a *wrong root cause*) analysing a 5-hour-stale kernel whose `mtime`
never moved.

- **Detect:** `stat -c %y build-amd64/ntoskrnl/ntoskrnl.exe` vs your edit's mtime; or
  put a unique string in your edit and `strings ntoskrnl.exe | grep` for it **before
  smoking**. If absent, the kernel is stale.
- **Force a relink — you MUST nuke the cargo *target dir*, not just the exes.** Deleting
  only the outputs makes ninja relink, but cargo's incremental cache in the per-arch kernel
  target reuses the **stale rlib body** — so the relink reproduces the old kernel invisibly
  (the exact "cargo incremental masks regressions" trap). Nuke the target dir first, then the
  outputs:
  ```bash
  rm -rf sdk/lib/rust/target/x86_64-kernel      # amd64 (i686-kernel / aarch64-kernel per arch)
  rm -f build-amd64/ntoskrnl/{ntoskrnl.exe,libkernel_rust.a,ntkrnlmp/ntkrnlmp.exe}
  ninja -C build-amd64 livecd                   # arm64: ninja (full) + mkramdisk --refresh
  strings build-amd64/ntoskrnl/ntoskrnl.exe | grep '<your-marker>'   # verify BEFORE smoke
  ls -la build-amd64/ntoskrnl/ntoskrnl.exe       # mtime MUST advance past your nuke
  ```
- **Applies to all 3 arches** — the cargo kernel target is `sdk/lib/rust/target/<arch>-kernel`
  (`x86_64`/`i686`/`aarch64`). **2026-06-16 netstack-landing incident:** arm64 hung at
  `PspExitThread → LpcExitThread` (ABI mismatch — new-ABI drivers + stale kernel) because
  `rm exes && ninja` relinked the **Jun-5 `aarch64-kernel` rlib**; `ntoskrnl.exe` mtime never
  moved. `rm -rf …/aarch64-kernel` + rm exes + `ninja` (mtime advanced) → clean boot. amd64/i386
  dodged it because their `--clean` path nukes the target; the standalone arm64 staged path does not.
- **NOT affected:** driver/usermode **cohab** crates (e.g. `scsiport-rust`, `conutils-rust`)
  link via the **main-workspace** cargo step which *does* rerun on `.rs` edits — only
  the **kernel** (separate `rust-kernel-target`) has the missing edge. (Smoke's
  `--clean` nukes `rust-kernel-target` but the staleness is in the *ninja relink trigger*,
  not the cargo cache, so even clean can skip it.) Always verify-by-marker for kernel work.
- **⚠️ DO affected (a separate target dir): cargo-DRIVEN `reactos_link` drivers**
  (`netstack.sys`, `beep.sys`, `null.sys` — the `Linker::new(...).rust_staticlib("<body>")`
  ones, *not* cohab `.a` crates). Their body crate is built by `reactos_link` inside the
  driver's `build.rs` into its **own** dir `sdk/lib/rust/target/<driver>-cargo-<body>_rust-rustlib`
  — **NOT** `target/<arch>-kernel`. So the §9.6b recipe (`rm -rf target/<arch>-kernel`) does
  **not** rebuild the driver body: `ninja <driver>.sys` reuses the stale body rlib and the
  `.sys` comes out **byte-identical**. Recipe (netstack/arm64, 2026-06-20 M7 — cost two stale
  cycles):
  ```bash
  rm -rf sdk/lib/rust/target/netstack-cargo-netstack_rust-rustlib          # THE body crate
  rm -f build-arm64/drivers/network/netstack/netstack.sys \
        sdk/lib/rust/target/aarch64-kernel/release/netstack.exe
  rm -rf sdk/lib/rust/target/aarch64-kernel/release/.fingerprint/netstack-cargo-*  # re-run build.rs (it drives the body build)
  ninja -C build-arm64 drivers/network/netstack/netstack.sys
  strings build-arm64/drivers/network/netstack/netstack.sys | grep '<marker>'  # verify BEFORE smoke
  ```
  🔑 **Never judge freshness by PE file size:** netstack.sys stayed **720896 = 0xB0000** across
  a stale *and* a fresh build (PE is section-padded), so adding several `nsprint!` calls didn't
  move the byte count. The only reliable signal is `strings … | grep <unique-marker>` (or a
  fresh source string in the binary), exactly like the kernel case above.

- ✅ **STRUCTURALLY FIXED 2026-07-04** (after one more gate false-green: a kernel-constants
  batch's i386+amd64 smokes passed on a **pre-fix ntoskrnl.exe inside a freshly-rolled ISO**
  — 37 cargo-linked binaries stale on x86, 18 on arm64, including lsass/services/smss/
  winlogon/csrss/win32k; only arm64's binary-vs-`.rs` staleness probe caught it). Two fixes:
  1. **Build side (the root):** `add_cargo_module` (sdk/cmake/gcc.cmake) now DEPENDS on a
     **whole-workspace glob** (`sdk/lib/rust/**.rs|toml` minus `target/`, cached in a global
     property) — the workspace is one dependency graph, so any `.rs` edit re-runs every cargo
     link (a cheap cargo freshness no-op for unaffected modules). This is the same medicine
     `rust_kernel`'s DEPENDS got earlier; the cargo-LINK layer had been left on hand-listed
     `BODY_CRATES` (for ntoskrnl: the one-file `kernel-rust` umbrella — hence no kernel edge).
  2. **Detection net (defense-in-depth):** `cargo_link_staleness_reasons` in
     `tests/qemu/cli/provenance.py` — self-maintaining scan of `add_cargo_module` call sites;
     every boot-relevant cargo-linked artifact (CD manifest on x86, boot set on arm64) must be
     NEWER than the newest workspace `.rs`/`.toml`. Gates both staleness tiers (the build
     stamp is blind: a build RAN and re-stamped; ninja just did no work).
  The hand recipes above remain useful for *cargo-internal* staleness (incremental-cache
  masking — nuke the target dir), but the "ninja never re-ran the link" leg is closed.

### 9.6c ⚠️ A kernel ABI change needs ALL drivers rebuilt — the staged arm64 smoke build is insufficient

Sibling of §9.6b, different mechanism. When a commit changes a **kernel ABI that
drivers depend on** (e.g. the arm64 `IRP`/`IO_STACK_LOCATION` layout —
`IoGetCurrentIrpStackLocation` math), *every* driver must be recompiled against the new
layout. But `./vibeos-cli build --arch arm64` (and the smoke harness) build only the
**staged set** (`ntoskrnl`/`hal`/`uefildr`) + `mkramdisk` — they do **not** rebuild
drivers (`fastfat.sys`, `scsiport.sys`, the storage stack, etc.). So after pulling such a
change you get a ramdisk with a **new-ABI ntoskrnl + old-ABI drivers** → the mismatch
manifests as: **boot reaches the desktop, then idle-detection times out** (win32k/user32
come up, apps launch, but the box never settles — looks exactly like a hang/livelock, not
a crash).

- **Detect:** arm64 smoke `HANG_TIMEOUT` whose serial shows the desktop reached
  (`PspUserThreadStartup` for explorer/notepad/cmd, `NtUserProcessConnect`) — i.e. it got
  *far*, didn't BugCheck. Pair that with a recent kernel-ABI commit.
- **Fix:** a **full** `ninja -C build-arm64` (not the staged set) to rebuild all `.sys`
  against the new ABI, then `bash qemu/mkramdisk.sh --refresh`, then re-smoke. Cleared an
  arm64 idle-timeout to green in one shot (2026-06-15, fastfat cohab-flip merge: the merge
  carried the arm64 IRP-stack-location ABI fix).
- **i386/amd64 unaffected** when the ABI fix is arch-cfg-gated (`cfg(target_arch)`), and
  they boot from a LiveCD ISO that `ninja livecd` repackages with freshly-built drivers
  anyway — only arm64's staged-ramdisk path has the gap.

**⚠️ Sharper, more dangerous variant (2026-06-20): the skew can wedge EARLY and impersonate
a layout-tip miscompile in the exact change you're gating.** A *targeted-subset* arm64 build
(`ninja ntoskrnl hal uefildr win32k kernel32` — what agents reach for to gate a single
module fast) restages stale old-ABI drivers under the new-ABI kernel just like the staged
smoke path. But the wedge it produced was **not** "desktop-then-idle-timeout" — it was a
**deterministic 2/2 wedge at `PspUserThreadStartup → KiInitializeUserApc done → returning`,
PC=0x7FFB7001018, `_tWinMain=0`, explorer never starts** (the *first* usermode thread never
progresses). That signature is indistinguishable from a **#35-class layout-tip latent
miscompile** (links clean, wedges at runtime at a user-thread-startup site) — so when it
appeared while gating a kernel **link-layout change** (the `librtl.a`+`libhal.a` static-lib
drops), it looked exactly like "your link change tipped a latent captures(none) miscompile."
It hadn't: **bare mainline built the same subset-way wedged at the identical PC.** A full
`ninja -C build-arm64` + `mkramdisk --refresh` on mainline reached explorer (34s) → the
"regression" was 100% build-completeness artifact.
- **Rule:** NEVER gate an arm64 **link/layout or static-lib** change with a targeted-subset
  build post-fastfat-merge — *always* full `ninja` + `mkramdisk --refresh`, or the driver
  ABI skew fakes a usermode wedge that frames your change.
- **Decisive disambiguation (do this BEFORE symbolicating / reaching for a `no-prepop`
  guard):** rebuild **bare mainline** (without your change) the *same subset way* and boot.
  Same wedge on mainline ⇒ build artifact, your change is innocent, stop. Only if mainline
  is clean and your change wedges **under a FULL ninja** is it a real regression worth the
  symbolicate→targeted-guard path (§"#38/#35" Findings 2–3). Cost of skipping this step:
  a full symbolication + no-prepop fork put to the user on a phantom regression.

### 9.6c.1 ⚠️ A boot-critical driver MISSING from the arm64 staged set false-greens the gate on a STALE pre-cohab-flip binary — masking a broken Rust translation indefinitely

Distinct from §9.6c (which is an ABI *skew* of drivers that ARE built). Here a Rust-cohab
driver is **absent from `tests/qemu/cli/archcfg.py`'s arm64 `ninja_targets`/`staged_bins_rel`
entirely**, so `vibeos-cli build --arch arm64` + `mkramdisk --refresh` **never rebuild it** —
the gate ships whatever `.sys` was last on disk, which for a driver cohab-flipped C→Rust is the
**pre-flip STALE C binary**. The committed Rust translation is then **never boot-exercised**:
the gate stays green while a *clean* build (which does build the Rust `.sys`) crashes.

- **2026-06-26 case — `ramdisk`:** the 06-21 `ramdisk.c`→Rust cohab flip (`1c492f8adb9`,
  "link-clean x3" bar, never arm64-boot-tested) sat broken on mainline for days. Every arm64
  gate was green (36s) because `ramdisk` was in **neither** `ninja_targets` nor
  `staged_bins_rel` → the gate booted a stale **C** `ramdisk.sys` (13.7KB `.text`, 0 Rust
  syms). A clean build's **Rust** `ramdisk.sys` (140KB `.text`, 651 Rust syms) BUGCHECK'd
  **0x64** at the first file open (`ZwOpenFile(ntdll.dll)=0xc000003a`, IopParseDevice never
  reached). Root cause was a cluster of **garbage machine-translation consts used as Rust
  `match` patterns / FSCTL codes** — the const-as-pattern silent-dead-code class (sibling of
  §"const placeholder" entries): `FSCTL_CREATE_RAM_DISK`=`0x900c4` (×3 local shadows, correct
  `0x240000`=`CTL_CODE(FILE_DEVICE_VIRTUAL_DISK,0,BUFFERED,ANY)`) → the FSCTL dispatch arm was
  dead code → device never created; `RAMDISK_BOOT_DISK`/`WIM`/`MEMORY_MAPPED`=garbage ASCII
  (correct 3/4/2) → boot disk failed `DiskType >= RAMDISK_BOOT_DISK`. After those: a deeper
  **0x23 (FAT_FILE_SYSTEM)** because `IOCTL_DISK_CHECK_VERIFY`=`0x70000` **collided** with
  `IOCTL_DISK_GET_DRIVE_GEOMETRY`=`0x70000` and was listed FIRST in the match → geometry IOCTL
  returned success with a **zeroed `DISK_GEOMETRY`** (`BytesPerSector=0`) → fastfat FAT math
  divides by zero. Fix = all consts to header CTL_CODE values + add `ramdisk` **and** `pl011`
  (the 5th boot-critical Rust-cohab driver, also missing) to both archcfg lists.
- **Why x86 is immune:** i386/amd64 boot from a LiveCD ISO that `ninja livecd` repackages with
  freshly-built drivers; only arm64's **staged-ramdisk** path can ship a stale `.sys`. So an
  arm64-only `ramdisk`/`pl011`/`serial` cohab flip is invisible to the x86 gates entirely.
- **Detect / audit (do this whenever you cohab-flip ANY arm64 boot-loaded driver):** in a boot
  serial, `grep '\[BLT\] IopInitializeBuiltinDriver:'` lists *every* driver the kernel BLT-loads
  at boot (2026-06: `fastfat ramdisk pl011 arm64input arm64vid`). EVERY one that is a Rust-cohab
  workspace member MUST be in `archcfg.py` arm64 `ninja_targets` + `staged_bins_rel` (or
  `ninja_targets_optional`), or the gate false-greens on its stale pre-flip C binary. The
  comment block at the flip site (`*.cmake` "replaced by Rust") and `link-clean x3` bars do
  **not** imply boot-tested — only a clean arm64 boot of the actually-rebuilt Rust `.sys` does.
- **Mechanizing the const class — CTL_CODE half now CLOSED (2026-06-26):** these garbage consts
  slipped both static gates — `scan_inline_flag_literals` only catches *annotated bare literals*
  (`0x2 /* NAME */`), and `verify_constants` checked `const NAME = VALUE` against C `#define`
  *literals* only — but `IOCTL_*`/`FSCTL_*` are `#define NAME CTL_CODE(dev,fn,method,access)`
  *macro calls*, which all three of its literal/multichar/ntstatus parsers skip, so they never
  entered the catalog and any garbage Rust copy went unchecked. **`verify_constants.build_c_catalog`
  now evaluates the CTL_CODE macro** (`(dev<<16)|(access<<14)|(fn<<2)|method`), resolving the four
  operands from the header scope (incl. `IOCTL_*_BASE` aliases → `FILE_DEVICE_*`) and seeding the
  standardized base operands; it joins the DDK headers' multi-line continuation defs first. The
  catalog gained **326 IOCTL_/FSCTL_ codes** (was 0). It would have caught every ramdisk const
  above at the static gate. Locked by `tools/rustconv/tests/test_verify_constants_ctlcode.py`.
  - **Generalized to ALL wrapper-macro families (2026-06-28) — the robust upgrade:** many IOCTL
    families aren't `CTL_CODE(...)` directly but go through a *device-specific wrapper macro* —
    `_AFD_CONTROL_CODE`, `_TDI_CONTROL_CODE`, `_NDIS_CONTROL_CODE`, `USB_CTL`, … — and AFD's wrapper
    doesn't even use `CTL_CODE` (it has its own packing `(FSCTL_AFD_BASE)<<12 | (Op<<2) | Method`).
    Special-casing `CTL_CODE` missed all of these. Replaced the `CTL_CODE`-only parser with a
    **generic function-like-macro expander** (`_expand_func_macros`): it parses every `#define
    NAME(params) body` wrapper from the headers, recursively substitutes args (whole-word, so a
    `Method` param never matches inside `METHOD_NEITHER`) and expands nested calls until the body is
    pure arithmetic, then evaluates. `parse_ioctl_defines` now scans the whole `IOCTL_*`/`FSCTL_*`
    namespace through this one path — literal, direct-`CTL_CODE`, and any wrapper, family-agnostic.
    Catalog **326 → 390** (+AFD 38, TDI 14, NDIS, USB), verified against known answers (`IOCTL_AFD_BIND`
    `0x12003`, `IOCTL_AFD_SEND` `0x1201F`, `IOCTL_TDI_CONNECT` `0x210004`, `IOCTL_NDIS_QUERY_GLOBAL_STATS`
    `0x170002`). **This is exactly netstack's afd bug #3** (the `IOCTL_AFD_*` block mis-numbered
    sequentially instead of computed → `msafd` SEND `0x1201F` misrouted to the RECV arm) — the
    generalized gate computes the right value, so that class is now caught statically for the whole
    transport/USB stack, not just the disk/cdrom CTL_CODE ones. Bringing a new family under the net
    is a one-line header add to `CTL_CODE_HEADERS`; depth-guarded so a recursive macro bails to
    "unevaluable" (skipped) rather than fabricating. Self-referential/non-arithmetic wrapper macros
    are harmless — they only expand when an IOCTL body calls them and a non-integer result is skipped.
  - **First catch (fixed same commit):** 3 real wrong codes in `ntoskrnl-rust` — `fstub/disksup.rs`
    `IOCTL_DISK_GET_DRIVE_LAYOUT` `0x74004`→`0x7400C`, and `iomgr/rawfs.rs` oplock FSCTLs numbered
    *contiguously* but the real headers skip: `FSCTL_OPLOCK_BREAK_ACKNOWLEDGE` `0x90008`→`0x9000C`
    (fn 3 not 2), `FSCTL_OPLOCK_BREAK_NOTIFY` `0x9000C`→`0x90014` (fn 5 not 3).
  - **Gate scope now WHOLE-WORKSPACE (2026-06-26):** `verify_constants` main() default + the gate
    spec (`tests/qemu/cli/lints.py`) now scan **all of `sdk/lib/rust/**` (driver crates included)**,
    not just `ntoskrnl-rust/src` — so the prevention finally reaches where ramdisk actually lived.
    Its volatile catalog/file-count summary moved to **stderr** (gate reads stdout only) so the
    baseline stays stable as workspace files come and go. The tree-wide scan found **57 latent
    garbage IOCTL/FSCTL consts**: **cdrom-rust's 26 are FIXED mechanically** (`verify_constants
    --fix` — it rewrites each to the computed CTL_CODE value); the rest (fastfat 24, disk/mountmgr/
    classpnp/cdfs/usbhub) are **baselined** in `tools/lint-baseline/verify_constants.txt` (75 lines
    = 31 residual CTL + 44 non-CTL) as accepted debt for the owning lanes to burn down — the gate now
    blocks any NEW garbage const tree-wide.
  - **`--fix` now rewrites function-LOCAL consts too:** `parse_rust_const` strips the line so it
    *detected* `    const NAME = …` inside fns, but `fix_const_line` anchored its rewrite regex at
    column 0 → silently no-op'd on any indented const (e.g. cdrom's `IOCTL_CDROM_GET_INQUIRY_DATA`).
    Fixed by capturing leading whitespace in the prefix (indentation preserved). This closes the
    function-local half of the mechanization.
  - **Triage of the non-CTL findings (the holistic-fix question):** they split three ways —
    **CTL codes** are mechanically `--fix`-able (header-authoritative ABI values); **pool `TAG_*`**
    mismatches are *harmless translation divergences* (arbitrary 4-char debug labels, e.g. `TAG_KERNEL`
    "Kern" vs C "Ke  ") → baseline, do NOT auto-fix; **`STATUS_*`/flag** mismatches are *case-by-case*
    (a wrong NTSTATUS is a real bug, but the name can collide across status enums → FP risk) → review
    before fixing, never blanket `--fix`. So a blind tree-wide `--fix` is UNSAFE; fix per-class.

### 9.6c.2 ✅ The arm64 staged-build false-green is now STRUCTURALLY mechanized — a self-maintaining freshness net + build self-heal (2026-06-28)

§9.6c.1 mechanized the *const* half (a broken Rust binary). This closes the *other* half of
that bug — the **stale binary** that lets a broken (or even a *correct*) Rust cohab flip never
get boot-exercised because the arm64 staged build didn't rebuild it. The fix is two cooperating
pieces in `tests/qemu/cli/`, both keyed off the **actual boot manifest** (`mkramdisk.sh`'s
`REQUIRED_FILES`) rather than the hand-maintained `archcfg.py` lists — so neither can be
defeated by forgetting an archcfg entry (the exact failure mode that bit fastfat→ramdisk→pl011
→gdi32→user32→advapi32, six times):

- **The freshness net** (`provenance.cohab_staleness_reasons`, wired into `check_staleness` so
  `smoke` hard-fails). For every binary in the boot set (`mkramdisk.sh` `REQUIRED_FILES` ∪
  `archcfg.staged_bins_rel`) that maps to a `<stem>-rust` cohab crate, it asserts the built
  artifact is **newer than the crate's newest `.rs`**. A cohab flip that wasn't rebuilt leaves
  the artifact older than its source → loud `STALE` at the gate, instead of a silent stale-C (or
  stale-old-Rust) boot. **Crucially it runs in BOTH staleness tiers** — a flip that wasn't
  rebuilt leaves the build-stamp git-fingerprint *matching*, so the tier-1 fast path ("fingerprint
  unchanged ⇒ fresh") is itself unsound for these binaries and cannot be trusted; the net overrides
  it. Self-maintaining: a new `<x>-rust` crate that produces `<x>.sys` auto-joins the net the
  moment it's added to `mkramdisk.sh` — zero archcfg edits. (Mapping handles the `_`↔`-` skew:
  binary `kernel32_vista.dll` → crate `kernel32-vista-rust`.)
- **The build self-heal** (`build.py`): `boot_set_cohab_rels()` feeds those same cohab boot paths
  into the `ninja` invocation alongside `archcfg.ninja_targets`, so **every cohab boot binary is
  force-rebuilt regardless of whether anyone added it to `ninja_targets`**. This is the time-saver:
  no more "forgot archcfg → stale gate → hand-`ninja` the driver → full re-gate" loop; `build`
  just produces fresh binaries, and the net above confirms it.
- **Validation:** on a real `build-arm64` the net guards 20 cohab boot binaries (kernel, hal,
  win32k, kernel32(+vista), user32, winsrv, fastfat, ramdisk, pl011, netstack, ntdll, kdcom,
  bootvid, framebuf, mlang, version, winlogon, arm64input/vid) with zero false positives, and
  immediately flagged a genuinely-stale `user32.dll` (source newer than the built binary) that
  would otherwise have staged stale. `i386`/`amd64` return empty (a full `ninja livecd` can't ship
  a stale cohab binary), so the net is arm64-scoped by construction. Locked by 8 unit tests in
  `tests/qemu/cli/test_cli_units.py` (`test_cohab_*`, incl. the tier-1-override case).
- **Why mtime not content-hash:** the gate flow is `build`→`smoke` back-to-back, so mtimes are
  authoritative; the failure mode is conservative (over-warn on git-mtime churn between build and
  smoke, never false-green), and `--allow-stale` is the escape hatch. Same philosophy as the
  pre-existing re-stage trap (boot.img mtime), which this generalizes from "did mkramdisk re-run"
  to "was each cohab binary actually rebuilt from its current source."

**Takeaway:** you should no longer need the §9.6c.1 manual audit (`grep '\[BLT\]'` + cross-check
archcfg) for the *staleness* half — the gate now catches a stale cohab boot binary automatically,
and `build` stops producing them. The archcfg lists remain useful (they drive what `--clean`
nukes and the x86 path), but they are no longer the *only* thing standing between a cohab flip and
a false-green.

### 9.6d ⚠️ A bad mkramdisk run can hang arm64 late-boot — regenerate before suspecting code

Distinct from §9.6c (which is a real ABI mismatch). `mkramdisk.sh` occasionally produces a
**subtly-bad boot image** that hangs arm64 deterministically in late boot — but the *same
binaries* boot fine from a freshly-regenerated ramdisk. Seen 2026-06-16: one `mkramdisk
--refresh` output hung **3/3** arm64 smokes; a second `--refresh` (identical build inputs)
booted green in 30s. The boot binaries were byte-identical across all runs, so it was the
image, not the code.

- **Signature (vs §9.6c):** serial reaches csrss → user32 → winsrv → mpr → the `ufat.dll
  c0000135` load-fail lines, then **stops just before `[EXPLORER] _tWinMain entered`** —
  i.e. explorer never launches (black screen). §9.6c instead reaches the desktop *and*
  starts explorer/notepad user threads, then idle-times-out. If explorer's `PspUserThreadStartup`
  never appears and a recent kernel-ABI commit is *not* in play, suspect the image first.
- **Fix:** re-run `bash qemu/mkramdisk.sh --refresh` and re-smoke before spending any time
  in a debugger. A green on the fresh image confirms it was mkramdisk, not a regression.
- **Corollary:** when an arm64 smoke flips red, reproduce on a **freshly-regenerated**
  ramdisk before reporting a regression — a stale/bad boot image is a false signal.

### 9.6e ⚠️ A *reconstructed* build-arm64 dir → perpetual CMake reconfigure → permanent false `STALE_MEDIA`

Seen 2026-06-16 landing cdrom-rust. Symptom: `vibeos-cli smoke --arch arm64` hard-fails the
**staleness gate** (`verdict: FAIL, classification: STALE_MEDIA`) with salient
`"ninja reports work for ntoskrnl"` *even on genuinely-fresh media you just rebuilt*. Root:
in a build-arm64 that was reconstructed by hand (stale-dir cleanup, winnt.h-race fixes, etc.)
a `file(GLOB … CONFIGURE_DEPENDS)` enters a **perpetual reconfigure loop** — every `ninja -n`
prints `Re-checking globbed directories` → `Re-running CMake`, and a real `ninja` settles to
`no work to do` yet the *next* `ninja -n` still wants to reconfigure. The smoke staleness
check keys on `ninja -n <kernel>` being clean, so it can **never** pass for that build dir.

- **It is NOT a code/branch signal.** A fresh `git clone` + fresh `build-arm64` configures
  clean; the loop is local build-dir cruft. Don't chase it as a regression.
- **Get the real verdict:** verify the media is current by hand (ramdisk.img mtime newer than
  `ntoskrnl.exe`/`win32k.sys` etc.), then boot with `vibeos-cli smoke --arch arm64
  --allow-stale` and read the serial for `[WIN32K] shell ready` / `[KERNEL] SwitchDesktop OK`
  / `[EXPLORER] _tWinMain entered`. That bypasses the (false) gate, not a real check.
- **Media-tree prerequisite (related, same session):** `mkramdisk.sh --refresh` checks a long
  `REQUIRED_FILES` list under `build-arm64/{media,boot/bootdata}`. A reconstructed build dir
  often hasn't populated it (the SYSTEM hive `boot/bootdata/system`, the NLS files
  `media/nls/c_437.nls`/`c_1252.nls`). vibeos-cli's arm64 staged set (`ntoskrnl hal uefildr`)
  **assumes those already exist from a prior full build**. Populate once with `ninja livecd`
  (stages the bootdata hives + cd files) **plus** `ninja gen_nls` (the txt2nls outputs —
  these are NOT in the arm64 livecd DAG), then `mkramdisk --refresh` succeeds. (Do not pass
  `media/nls/l_intl.nls` to ninja — it's not a target; mkramdisk has a source-dir fallback.)

### 9.6f ⚠️ arm64 "uefildr StartImage: Unsupported" → PXE is CORRUPTED `qemu/efivars.fd`, not build-drift

Seen on arm64 UEFI boot. Symptom: the VM never loads the OS — it falls through to a **PXE/network
boot attempt**, and the serial shows `Error: Image at 0007FC7F000 start failed: Unsupported`. The
tempting (WRONG) read is a build-drift / bad-`uefildr.efi` regression, prompting a `rm -rf build-arm64`.

- **The `…start failed: Unsupported` line is firmware-internal EDK2 noise present in EVERY boot**
  (including green ones) — it is **NOT** the failure. Don't anchor on it.
- **Real root cause: a corrupted `qemu/efivars.fd`** (the UEFI NVRAM varstore) with a bad `BootOrder`
  → EDK2 BdsDxe skips the virtio disk entry and falls through to PXE. The kernel/uefildr build is fine.
- **Fix = restore a known-good `qemu/efivars.fd`** (copy from a working tree / a clean checkout), **NOT**
  `rm -rf build-arm64`. `efivars.fd` is gitignored boot state, easily corrupted by an interrupted boot
  or a stale varstore; it is not regenerated by the build. (A clean EDK2 `QEMU_EFI`/vars pair from the
  arm64 boot setup re-seeds a sane BootOrder.)
- **Related trap — a full `rm -rf build-arm64` (vs `vibeos-cli build --clean`) exposes a pre-existing
  missing-dep race:** the `.rc` resource compiles don't depend on the `xdk` header-gen target, so a
  from-scratch reconfigure races and fails with **`winnt.h: No such file or directory`**. Workaround:
  `ninja xdk` first (generates the headers), then the full `ninja`. So `rm -rf build-arm64` is doubly a
  bad reflex for the Unsupported/PXE symptom — it doesn't fix it (the cause is efivars) and it trips the
  xdk race.

### 9.7 Reliable-repro timing — slow ≠ hung

- A no-gdb KVM boot to a post-`shell ready` BUGCHECK is **~80–150s**. Boots are genuinely slow (CD load); **wait ≥150s** before calling it a hang. A trickle of early serial (e.g. 87/581 bytes) is slow CD load, not a hang.
- Non-deterministic crash: repeat the TCG hunt. A corruptor that **varies per boot** (different victim each run) is a **cascade** from an earlier first-cause — chase the earliest detectable corruption (e.g. `HEAP_TAIL_CHECKING` to catch the first overflow's victim), not the per-boot symptom.

### 9.8 Standard hunt recipe (toolkit)

From `kernel-debugging-infra.md` — the validated loop:
```bash
RUN=$(qemu/start.sh amd64 --gdb)                 # i386 hunts: prepend VIBEOS_NO_KVM=1
qemu/wait-for.sh $RUN --pattern "ACPI Timer" --timeout 60
qemu/run-until.sh $RUN --break KeBugCheckEx --timeout 180 --then triage-bugcheck
```
`triage-bugcheck` returns stop code, exception code, faulting PC + symbol, full
KTRAP_FRAME, and `identify-object` on bogus pointers in one call. Run
`qemu/build-index.sh <arch>` after each kernel build for offline symbol lookup.

### 9.9 Timer-table base skew — every relative KeDelay/KeWait-timeout over-waits ~4–60×

- **Symptom:** relative timed waits (`KeDelayExecutionThread`, relative `KeWaitForSingleObject`,
  Cc/Mm read-ahead pacing) take far longer than requested — a 2000ms delay waits ~8.7s
  (~4.37×), a 100ms delay waits ~6.4s (worse for shorter waits). UDP-style single-cycle ops and
  i386 are unaffected; multi-RTT operations (TDI TCP connect) time out. Box looks "idle, advancing
  on a coarse ~8s timer." Often **latent** and only exposed when an unrelated commit makes the
  clock actually tick (e.g. RTC→LAPIC switch `22356522c58`).
- **Root cause:** the NT timer table requires the invariant
  **`InterruptTime / KeMaximumIncrement == KeTickCount`** (both 0-based since boot). Insertion
  (`KiComputeTimerTableIndex`) hashes the *absolute* InterruptTime by KeMaximumIncrement;
  expiration sweeps `KeTickCount.LowPart & 511`. If the two clocks are seeded inconsistently at
  boot, the insertion bucket is offset from the sweeping hand by a constant N — a timer lands in a
  bucket the hand *just passed* and waits a full ~512-tick table revolution (~8s) before
  re-examination.
- **Oracle (deterministic, in-kernel):** drop a one-shot probe in `KxSetTimerForThreadWait`
  (inline/ke.rs — the REAL `KeDelayExecutionThread` hand path, NOT `KiComputeDueTime`) logging
  `now=KeQueryInterruptTime()`, `now_hand=(now/KeMaximumIncrement)&511`,
  `exp_hand=KeTickCount.LowPart&511`, `ins_hand`, `delta=(ins_hand-exp_hand)&511`,
  `expect=mag/KeMaximumIncrement`. **`now_hand != exp_hand` is the smoking gun** (the two bases are
  skewed). A first-tick probe in `KeUpdateSystemTimeImpl` shows whether InterruptTime/KeTickCount
  are pre-seeded huge before any tick.
- **Fix:** zero KUSER InterruptTime, KUSER TickCount and the `KeTickCount` global on the first
  `KeUpdateSystemTime` tick so they advance from 0 in lockstep. (`KeBootTime`/`SystemTime` are wall-
  clock and separate — leave them.) Verify: `now_hand == exp_hand`, `delta == expect`, KeDelay(100ms)
  → ~110–125ms.
- **Gotcha that cost ~6 build cycles:** the smoke `full.log` is a **truncated tail** (~140 lines of
  late boot) — early-boot `DbgPrint` probes (phase-1 self-tests, first ticks) are NOT in it. Grep the
  **live log** `qemu/<arch>-uefi_smoke_live.log` (full capture) instead.

### 9.10 A registry/hive addition that "shouldn't matter" deterministically tripping an early BUGCHECK 0x50

**Symptom:** You add benign data to a *baked* registry hive (e.g. an `add_registry_inf`
`.inf` with new `HKLM,...` values — in our case the Winsock2 protocol catalog
`netstack_winsock.inf`). The kernel code is untouched, yet boot now **deterministically**
BUGCHECKs `0x50` (PAGE_FAULT_IN_NONPAGED_AREA) in an early driver — read-fault on a
`0xfffff8a0...` paged-pool VA, faulting PC in the `0xfffff880...` driver region —
*before* `shell ready`, at the same log line every run. Backing the hive change out
restores a green boot.

**Root cause:** The hive is loaded early by the config manager; growing/altering it
shifts pool allocation order, which moves a **latent** memory bug (uninitialised read,
off-by-one, use-after-free, a hand-mirrored `repr(C)` struct with a wrong offset) from
"benign / lands on valid memory" to "lands on an unmapped page → 0x50." The registry
data is innocent; it is a **layout perturbation** that *exposes* a pre-existing driver
bug. This is the same family as the known amd64 taskbar 0x50 (`project_amd64_taskbar_
bugcheck_0x50`), but the hive shift makes it **deterministic and early** instead of
non-deterministic and post-`shell ready`.

**What to do:**
1. **Don't ship the hive change** — a baked `add_registry_inf` goes into the *shared*
   system hive, so it breaks boot on *all* arches/configs, not just the one you tested.
   Disable the `add_registry_inf(...)` line (keep the `.inf` artifact + a comment).
2. **Treat it as a gift:** a deterministic, same-line repro of an otherwise
   non-deterministic 0x50 is the easiest version to hunt. Symbolicate the faulting PC
   (`0xfffff880...`) against the per-arch build index (`qemu/build-index.sh amd64`,
   then `qemu/addr2sym.sh`) to name the driver, then bisect that driver's recent
   translations. The layout shift is the *trigger*, not the *bug* — fix the driver,
   not the hive.
3. **Confirm causation cheaply:** revert only the hive line and re-run. Boot green ⇒
   the addition is the trigger (not a flake). Two identical-line crashes ≠ the ~1/3
   flake; determinism + a *pre*-`shell ready` PC that the flake never reaches is the
   tell.

**Worked instance (2026-06-13, netstack Winsock2 catalog → 0x50, survives the crp merge):**
The faulting driver is the **uniata/scsiport IDE storage stack**, *not* netstack — the
catalog hive is innocent. Diagnosis chain that worked when the usual symbolication failed:
- **build-index/addr2sym is ntoskrnl-only.** The faulting PC `0xfffff880055cd0x0` is in a
  *driver* (`0xfffff880` region), which addr2sym can't resolve, and the Rust-cohab modules
  ship **no Rust DWARF** (cargo-drive strips it) so `qemu/start.sh --gdb` *preflight fails*
  — use `--skip-preflight`. Adding debug info is NOT an option: the bug is layout-sensitive,
  so the size change would move it (Heisenbug).
- **Caller-level bisect markers** (cheap `DbgPrint` at each call site, *not* in the hot path)
  localised it: the crash fires *after* the IDE `adapter-init complete` marker with no
  subsequent init call → it's an **async callback** (timer/DPC/ISR), reached because
  `FdoCallHWInitialize`'s `IoConnectInterrupt` fails (~33× "Could not connect interrupt") and
  the caller ignores the failure, starting the IoTimer on a half-set-up adapter.
- **The crash PC is STABLE across `scsiport-rust` rebuilds** (it does *not* shift when you add
  code to scsiport and recompile). That is the key tell: the faulting code is **not in the
  module you're editing** — here it points at `uniata.sys` (C miniport, never rebuilt) or a
  scsiport function linked before the edited init path. Instrumenting `ScsiPortIoTimer`
  *ruled it out* (its entry `DbgPrint` never fired before the fault). Lesson: when a PC
  refuses to move under your edits, you're editing the wrong module — bisect by *which
  module* moves the PC, not just which function.
**SOLVED (2026-06-14, commit `scsiport: fix uninitialized PortConfig on find-loop retries`):**
The earlier "IoConnectInterrupt fails ~33×" / "`Interrupt[0]` NULL → `KeSynchronizeExecution`"
hypotheses were **wrong** — instrumenting the connect path showed *both* IDE channels connect
fine (IRQ14→vec176, IRQ15→vec160, Status=0). The real bug is in **scsiport's `ScsiPortInitialize`
HwFindAdapter while-loop**, not uniata or the interrupt path:
- `ScsiPortInitialize` **re-allocates `PortConfig` fresh (non-zeroed pool) every loop iteration**,
  but `SpiCreatePortConfig` only *initializes* the struct when `ZeroStruct` (`FirstConfigCall`)
  is TRUE — i.e. only before the first device is found. On later iterations every PortConfig
  field is uninitialized garbage.
- The killer field is **`AdapterInterfaceType`**: when the garbage reads `0` (== `Internal`),
  `SpiCreatePortConfig` wrongly enters the Internal hardware-database branch and **resets
  `InternalConfigInfo->BusNumber = 0`**. The legacy per-channel find loop (uniata
  `AtapiFindIsaController`, `AdapterInterfaceType=Isa`) then never advances past bus 0, the
  `IoQueryDeviceDescription` bus bound never trips, and `ScsiPortInitialize` spins ~25 000×
  → 0x50. The catalog only perturbs the pool so the garbage is deterministically `0`.
- Secondary garbage field: `AtdiskPrimaryClaimed`/`AtdiskSecondaryClaimed` (uniata's
  "channel already claimed" check reads them).
- **Fix:** in `SpiCreatePortConfig`, seed `AdapterInterfaceType` from `HwInitData` before the
  Internal check; and re-seed the AT-disk claim state from `IoGetConfigurationInformation()`
  before every `HwFindAdapter` call. Both target the *fresh-PortConfig-not-reinitialized* root.
- **Diagnosis tooling that cracked it:** tagged `DbgPrint` probes walking the chain
  scsiport find-loop → `SpiCreatePortConfig`/`IoQueryDeviceDescription` → the BusNumber value
  per iteration (it was stuck at 0 — the smoking gun). Confirmed ×3 amd64 + i386 + arm64 boot to
  desktop **with the catalog enabled**.

**General lesson (recurs across the C→Rust port):** a C `for(;;){…}` loop body that re-allocates
a struct each pass and relies on a `ZeroStruct`/first-call flag to initialize it is a garbage-on-
retry trap — the "preserve across iterations" optimization assumes the buffer is *reused*, which
ReactOS violates by re-allocating. Any field read before it is set (here `AdapterInterfaceType`
gates a code path) becomes a non-deterministic, pool-layout-sensitive bug. When a loop "never
terminates", print the loop's *advancement variable* (here `BusNumber`) every iteration — if it
doesn't move, find who resets it.

- **Remaining (M5, separate bug):** with the 0x50 gone and the catalog in the hive, usermode
  `socket()` still returns `WSAEINVAL (10022)` — ws2_32/msafd isn't consuming the netstack
  Winsock2 catalog entries. Boot-unblocked; catalog-loading issue, tracked on M5/task-77.

### 9.x Symbolicating a cargo-built kernel crash address (when objdump/nm come back empty)

**Symptom:** you have a runtime crash/return address (e.g. an unhandled-exception PC, a
backtrace frame) on arm64 (or any cargo-linked module), but `aarch64-w64-mingw32-objdump
-d/-t/-h`, `llvm-nm`, and `llvm-objdump` all return **empty** on `ntoskrnl.exe` — the
cargo-linked PE is opaque to the GNU binutils, and the in-memory RVAs exceed the on-disk
file size (large virtual layout), so file-offset reasoning fails.

**Recipe:**
1. **Get a symbol index:** `qemu/build-index.sh <arch>` → `qemu/symbols-<arch>.tsv`
   (`<hex_RVA>\t<symbol>`). NB it covers **only that module's Rust+asm symbols** and tops
   out at the highest such symbol — **C-object functions and *other modules* (hal, fastfat,
   drivers) are absent**.
2. **Find the runtime load base** — the kernel is relocated, so you must calibrate. Print a
   *known* symbol's runtime address from inside the kernel (cheapest: temporarily
   `ARM64_UART_TRACE_HEX("kdx=", KiDispatchException as usize)` in a path you'll hit; tsv
   gives its RVA). Then **`base = runtime_addr − tsv_RVA`** (observed arm64 ≈
   `0xFFFFF80041D91xxx`, shifts a few bytes per rebuild — recalibrate each build).
3. **Resolve:** `rva = crash_addr − base`; nearest tsv symbol `≤ rva` names it.
4. **Key inference:** if `rva` is **past the tsv's max symbol**, the address is **not in this
   module** — it's in a module loaded *after* it (hal/fastfat/driver). That alone localizes
   a raiser (e.g. confirmed an unhandled `STATUS_FILE_CORRUPT_ERROR` was raised in
   *fastfat.sys*, not ntoskrnl Cc, because the `ExRaiseStatus` caller addr was past
   ntoskrnl's last symbol).
5. **Identify the raiser directly:** hook the **`ExRaiseStatus` shim** (`ntoskrnl-rust/src/
   shim_exports.rs` — it exists expressly for this) to print `x30` (caller LR) when
   `status == <the code>`. One clean address beats a stack scan.

**Gotchas:** `strings <exe> | grep <marker>` MISSES `TrapPuts(b"...")`-style traces — the
optimizer turns short byte-strings into **inline per-char immediate stores** (`mov w,#0x5b
'['; …`), not rodata; disassemble (or just trust the build) instead. A raw **stack scan**
for kernel-VA values works but is **polluted by stale frames** (the 100Hz clock leaves
`KiIrqEntry`/`KiIrqHandler`/`KeUpdateRunTime` return addrs on every stack); **frame-pointer
(x29) unwind from a big function like `KiDispatchException` does NOT work** (x29 isn't a
clean FP there). **Source:** arm64 clock-fix lane, `arm64-clock-and-lpcexitthread-findings.md`.

### 9.11 A nested `0x50` whose fault addr is one page above SP = STACK OVERFLOW — look for unbounded recursion (often a constant-placeholder), not a scheduler/LPC race

**Symptom:** a deterministic `BUGCHECK 0x50` (PAGE_FAULT_IN_NONPAGED_AREA) late in boot that
*looks* like a flaky scheduler/LPC race (it was mislabeled `arm64_lpcexitthread_race` for a
whole session because the last serial line before it was `PspExitThread → LpcExitThread`).
Tells that it's really a **kernel stack overflow**, not the thing the last log line names:
- **`arg2 == 2` (write)** and **`arg1` (fault addr) is ~one page ABOVE the trap-frame / SP**
  (here fault `0x..256050`, trapframe `0x..255e40`, SP `0x..255fa0` — all in the same
  `0x..255/256xxx` page-pair). A heap/pool wild-write lands *anywhere*; a stack overflow
  faults *adjacent to SP* in the guard page.
- The faulting **Pc is `KiSynchronousExceptionEntry+0x4`** (the first `stp x0,x1,[sp,#…]` of
  the exception prologue) while **Lr points into the real victim function**. That's a
  *nested* fault: the first exception fired in the victim, then the handler couldn't even
  save its own trapframe because the stack was already blown → the `0x50` you see is the
  handler's store, not the original fault.

**Why arm64 specifically:** `KERNEL_STACK_SIZE` is **12 KB on i386 *and* arm64** but **48 KB
(0xC000) on amd64**. A 64-bit recursion (bigger frames) that amd64 absorbs, and that i386
never reaches (it boots read-only media), blows arm64's inherited-from-i386 12 KB stack. So
arm64 is the first arch to expose any unbounded-recursion / deep-chain bug in a *writable*-FS
path (fastfat-on-ramdisk).

**Diagnosis recipe (no DWARF needed):**
1. From the bugcheck args, read the trap frame raw. arm64 `KTRAP_FRAME` offsets (from
   `ndk/arm64/ketypes.h`): `FaultAddress@0x10`, `Sp@0xa8`, `X[0]@0xb0`, `Lr@0x148`,
   `Fp@0x150`, `Pc@0x158`. `Esr@0xa4`: `EC = Esr>>26` (`0x25`=data abort), `WnR=(Esr>>6)&1`
   (1=write), `DFSC=Esr&0x3f` (`0x07`=translation fault = page not mapped).
2. **Walk the frame chain** off `Fp` (x29): `lr=*(fp+8)`, `next=*(fp)`. A short cycle of the
   **same 3–5 return addresses repeating** = the recursion; that *is* the bug.
3. **Symbolicate with the *right* binutils name** — it's `aarch64-w64-mingw32-nm`
   (`.build-tools/llvm-mingw/bin/`), NOT `llvm-nm`/plain `objdump` (those silently print
   nothing on the cargo PE). `qemu/symbols-<arch>.tsv` from `build-index.sh` is sparse/wrong
   above the last named symbol — build your own map: `nm <exe> | awk 'NF==3&&$2~/[Tt]/'`.
4. **If a frame's RVA is past ntoskrnl's `.text` end, it's in another module** (§9.x). Map
   runtime→module by **walking `PsLoadedModuleList`** in gdb (KLDR_DATA_TABLE_ENTRY:
   `DllBase@0x30`, `SizeOfImage@0x40`, `BaseDllName.Buffer@0x60`, `.Length@0x58`). Then for
   that module: `PE_VA = ImageBase + (runtime − module_base)`; resolve against *its* nm map.
   (Here the cycle resolved into **fastfat.sys**, not ntoskrnl.)

**Root cause (this instance, 2026-06-20):** constant-placeholder bug class again (sibling to
cdfs `BugCheckFileId`, scsiport LUNEX, fastfat `ChicagoMode`, find.rs cast-before-offset).
fastfat's recursion-breaker is `FatSetFatEntry` setting `IRP_CONTEXT_FLAG_DISABLE_DIRTY`
(canonical **0x00000001**, fatstruc.h) around its `FatPrepareWriteVolumeFile` writes, so the
nested `FatSetDirtyBcb` skips re-marking the volume dirty. The local const inside
`FatSetDirtyBcb` (cachesup.rs) was a placeholder **0x100**, so `Flags & DISABLE_DIRTY` never
matched the 0x1 bit actually set → guard never fired → infinite cycle
`FatSetDirtyBcb→FatMarkVolume→FatSetFatEntry→FatPrepareWriteVolumeFile→FatSetDirtyBcb`. Fix:
`0x100→0x01` (commit `3b3f9744c0a`). **Lesson:** when a flag-guarded recursion-breaker is
defeated, the *setter* and the *checker* of the flag are often in different files/functions —
grep every definition of the flag constant and confirm they agree with the C header. **Source:**
arm64-love lane; `arm64-clock-and-lpcexitthread-findings.md`.

---

*Maintenance: this manual is the fast-lookup index distilled from the project's debug
memories. When you solve a new translation bug, add a `### Symptom → Root cause → Fix`
entry to sections 1–8; when you discover a new live-crash-hunting technique, add it to
section 9. Goal: the durable learning lives HERE in the repo, not only in per-session
agent memory.*

### 9.x x86 UEFI boot bring-up recipe (OVMF) + the three UEFI-only gotchas

Non-obvious operational bits that make an x86 kernel boot under **UEFI (OVMF)** instead of legacy BIOS. These are latent in the C too — they only manifest once you actually boot via UEFI, so they bite whenever the UEFI boot path is touched or first enabled on an arch.

#### Gotcha 1 — i386 PAE→non-PAE paging transition (triple-fault if mis-ordered)

OVMF hands control to the loader with **paging ON and `CR4.PAE=1`**. A classic i386 kernel uses **non-PAE 2-level paging**. Doing `writecr3(PDE)` while `CR4.PAE=1` makes the MMU read the non-PAE PDE as a PAE PDPT → **instant triple-fault** (the #PF fires on the very next instruction fetch).

Order the switch so PAE is cleared while paging is OFF:

```
1. if CR0.PG set  → clear CR0.PG      # paging off
2. clear CR4.PAE                       # now safe to drop PAE
3. writecr3(non-PAE PDE)
4. set CR0.PG                          # paging back on
```

Steps 1–4 execute identity-mapped (loader + stack live in identity-mapped low memory), so they're safe with paging off. No-op on legacy BIOS (paging is already off at the equivalent point). Lives in the i386 `WinLdrSetProcessorContext`; the stock C `writecr3`/`writecr0` pair has **no PAE handling** — it's a latent bug that only shows under UEFI.

#### Gotcha 2 — Synthetic ACPI 2.0 RSDP (else no keyboard/PS-2 input)

Under UEFI the RSDP lives in **EFI-reclaim memory**, so a legacy BIOS-area RSDP scan (`0xE0000`–`0xFFFFF`) finds nothing → `AcpiInitializeTables` fails → no ACPI namespace → `PNP0303`/`PNP0F13` (PS/2 kbd/mouse) never enumerate → i8042prt gets no `AddDevice` → **no keyboard**.

Fix (in `AcpiOsGetRootPointer`): keep the BIOS-area scan first (legacy path byte-for-byte unchanged); **on failure**, read the RSDT/XSDT physical address the bootloader recorded in the ARC tree — the `"ACPI BIOS"` multifunction-adapter Configuration Data at `\HARDWARE\DESCRIPTION\System\MultifunctionAdapter\N` (`CM_FULL_RESOURCE_DESCRIPTOR` → device-specific descriptor → trailing `ACPI_BIOS_DATA.RSDTAddress`, which holds the XSDT address on ACPI 2.0+) — and **synthesize a valid ACPI 2.0 RSDP** (correct 1.0 checksum + extended checksum) pointing at it. Arch-agnostic — arm64 takes the same fallback safely.

**Diagnosis tell:** a working BIOS boot's serial shows `[[I8K]] AddDevice ... KBD ISR FIRED`. A broken UEFI boot has **zero `[[I8K]]` lines** and reports `Tables: [XSDT]` (vs BIOS `[RSDT]`) plus `Unable to AcpiInitializeTables`.

#### Gotcha 3 — freeldr-LOADED DLLs must not use raw-dylib for cross-DLL kernel-data imports

A DLL that FreeLDR itself loads (e.g. `kdcom.dll`, `bootvid.dll`) must **not** use `#[link(kind="raw-dylib")]` for cross-DLL kernel-data imports. The 2nd `ntoskrnl.exe` import descriptor that raw-dylib emits **hangs FreeLDR's peloader**. Use the import lib's `__imp_` thunk instead.

#### OVMF boot recipe (i440fx + ide-cd)

Boot the LiveCD ISO from OVMF on the **i440fx `-M pc`** machine — *not* q35. i386 and amd64 share the same shape (no AHCI/SATA; the IDE driver serves the CD):

- **Firmware:** i386 = `qemu/OVMF32_CODE.fd` + `OVMF32_VARS.fd`; amd64 = `/usr/share/OVMF/OVMF_CODE_4M.fd` + `OVMF_VARS_4M.fd`.
- **Boot device:** `-device ide-cd,drive=cd0,bootindex=0`. A bare `-cdrom` does **not** auto-boot — without an explicit `bootindex=0`, OVMF drops to the UEFI shell.
- **Writable VARS pflash must be qcow2** (convert the raw OVMF VARS → qcow2) for QMP `savevm`/snapshot to work; a raw VARS pflash blocks snapshotting.
- **Rebuild the ISO after touching the loader.** The LiveCD ISO's EFI El Torito entry loads `EFI/BOOT/boot<plat>.efi` (`EFI_PLATFORM_ID` ia32/x64/aa64), which folds in `uefildr` — re-run `ninja livecd` after `ninja uefildr` or you boot a **stale ISO**.

### 9.12 A cross-thread `WaitForSingleObject(..., INFINITE)` on a cosmetic UI element is a boot-wedge — the win32k dialog-creation race behind the amd64 status-window hang

**Symptom.** amd64 **cold** builds hung ~1-in-3 during winlogon init (i386/arm64 cold
booted fine). Last boot marker was between `[WINLOGON] GinaInit returned` and
`[WINLOGON] Init complete. Spawning explorer.exe...`, then dead. *Warm* amd64 builds
booted green — the classic "cargo-incremental masks it, the clean build reveals it"
(§North Star): the first genuinely-cold amd64 rebuild in a while (forced by the
canonical-build-path cutover) exposed a latent race that stale objects had been hiding.

**Localization technique (breadcrumb bracketing).** Because it was a *timing* race
(same binary passed run 1, hung run 3), single-step DbgPrints both perturbed it and
localized it. Bracket the suspect region with a print *before* and *after* each call;
the last one printed on a hang names the blocking call. Here that walked it down to
`msgina!GUIDisplayStatusMessage` → worker thread `StartupWindowThread` →
`DialogBoxParamW` **entered but `WM_INITDIALOG` never dispatched**. (`gui.rs`.)

**Root cause (two layers).**
1. *win32k (the real defect, still open):* `DialogBoxParamW` on the winlogon desktop
   occasionally never reaches `WM_INITDIALOG` — window/dialog creation stalls inside
   win32k under amd64-cold timing. Would also bite any real dialog (SAS/logon) outside
   autologon. **Tracked as an open win32k item; not yet root-caused.**
2. *msgina/winlogon (the wedge amplifier, FIXED):* the status window is created on a
   worker thread while the winlogon main thread blocks on
   `WaitForSingleObject(StartupEvent, INFINITE)`, and `StartupEvent` is only signaled
   from **inside `WM_INITDIALOG`**. So *any* failure to reach `WM_INITDIALOG` (the
   win32k race, or the two pre-`return 0` early exits in the worker) hangs the entire
   logon session **forever**. A cosmetic boot-splash must never be able to do that.

**The reusable bug class.** `INFINITE` cross-thread waits where the wakeup lives on a
*single* success path (one window message, one callback) are latent total-hangs: every
error/early-return/race that skips that one path wedges the waiter permanently. Faithful
C→Rust translations inherit this from upstream (ReactOS's C has the identical
`INFINITE` + shallow-pointer-copy shape) — it only stays invisible while the substrate
(here win32k) is reliable. When our translated substrate has a race the original didn't,
these INFINITE waits convert a soft failure into a boot-blocker.

**The fix (defensive, lifetime-correct).** In `GUIDisplayStatusMessage`:
- **Bounded wait**, not `INFINITE` (`STATUS_WINDOW_TIMEOUT_MS = 20s`): on timeout the
  session proceeds without the cosmetic splash instead of hanging.
- **Deep-copy the title/message** (`DuplicateString`) into worker-owned heap before
  releasing the caller — the originals are winlogon stack buffers
  (`DisplayStatusMessage`'s `StatusMsg[]`), so a bounded wait that lets the caller
  return-and-unwind would otherwise leave the worker reading freed stack (use-after-
  scope). The worker frees them via `FreeStatusMsg`.
- **Signal `StartupEvent` on *every* worker exit path** (both early `return 0`s), so a
  pre-window failure releases the caller immediately rather than after the full timeout.
- On timeout, do **not** close the event / free `msg` — the (possibly still-running)
  worker owns them; a one-time boot-only leak beats a double-free.

**Takeaway.** When a clean build newly-exposes a hang the warm build hid, suspect a
race the stale objects were masking — and audit any `WaitForSingleObject(…, INFINITE)`
whose signal sits on a lone happy path; make the wait bounded and the shared state
owned by the side that outlives the wait.

### 9.13 ✅ RESOLVED — the amd64 service-startup wedge: a Mm WS-pushlock self-deadlock, root-caused to the `OriginalPte`/`RmapListHead` PFN-union confusion (2026-07-04→05)

**Symptom (as it presented for weeks):** amd64 boot wedged at service startup ~50% of runs — `DcomLaunch done: 1053` on serial ⟺ wedge. A single vCPU idle in `KiIdleLoop` (scheduler alive) while 3–6 freshly-created processes (eventlog/svchost/rundll32/setup) each had their FIRST thread permanently parked acquiring their own `EPROCESS.Vm.WorkingSetMutex`. eventlog.exe was silently dead on *every* boot (deterministic stranding); the boot only died when a boot-critical thread was among the caught, hence the ~50%.

**Mechanism (proven live, not hypothesized):** a page fault *inside* the WS-EXCLUSIVE(`Unsafe`) section-unmap teardown re-acquires the WS lock SHARED → self-deadlock (pushlocks are non-recursive). Stack: `MiDeletePte → MiDecrementShareCount+0x71 → #PF → MmAccessFault → MiLockProcessWorkingSetShared → KeWaitForGate`. Upstream avoids this by running `MiDecrementShareCount` under the PFN spinlock at DISPATCH (non-faultable); this port's Mm locks are faultable pushlocks/gmutexes.

**Root cause — a chain of `OriginalPte` bugs, all in the PFN-union handling:**
1. **`OriginalPte.Long` dereferenced as a pointer** (7 sites, `pfnlist.rs`/`pagfault.rs`): `mmpte_software_*(&*(mmpfn_original_pte_long(Pfn) as PMMPTE))` read the PTE *value* as a *pointer*. Fixed to `&*mmpfn_original_pte_ptr(Pfn)` (take the field address). This turned the HANG into a deterministic **0xBE** — exposing the next layer.
2. **ROS PFNs have no `OriginalPte` — `RmapListHead` overlays it** (C `_MMPFN` union, mm.h:421-428). The GDI handle table is a ROS section; its pages come from `MmAllocPage`, which never writes `OriginalPte`. Our hybrid's *session* view routed through ARM3 `MiCompleteProtoPteFault`, which read `OriginalPte.u.Soft.Protection` → got 0 (`MM_ZERO_ACCESS`) → mapped the DevCaps view READ-ONLY → win32k's first `PDEVOBJ_vGetDeviceCaps` write → **0xBE**. Fix: `MiCompleteProtoPteFault` branches on `MI_IS_ROS_PFN` — derive protection from the authoritative valid *proto* PTE and never touch the union slot; plus a `MM_ZERO_ACCESS`-heal defensive path (all arches).
3. **One-slot offset shift in `ntkrnl-sys` `pfn_offsets_extra`** (mm.rs): the hand-rolled `NextLRU`/`PreviousLRU`/`RmapListHead` accessors missed the embedded `Wsle` field, so LRU links + WS-lock bits + rmap heads all landed on the wrong bytes (and fought the bindgen struct, which had C truth). This silently disabled rmaps on arm64 (papered over by a "reject non-page-aligned heads" hack). Offsets fixed (64-bit 56/64/32; 32-bit 28/32/16), hack removed. Adjacent: **`MI_IS_PAGE_WRITEABLE`/`_COW` were inverted on arm64** (tested hw AP/SH bits instead of the software Writable@55 / CoW@56 bits) — ~9 fault-path callers.

**Result:** amd64 boots green (5/5 → gate ×3). The old "PteFrame share-count chain" and "MMPFN accessor layout" hypotheses were red herrings — the real defect was the `OriginalPte`/`RmapListHead` union overlay + the offset/bit-test drift.

**Permanent tools this produced (the durable payoff):**
- **Lockdep self-deadlock detector** — `vibeos-cli build --arch amd64 --lockdep` builds a `-DVIBEOS_LOCKDEP=ON` kernel; every boot self-reports `[LOCKDEP] SELF-DEADLOCK … WAIT(shared) … already held(excl) site=…` on serial (held-pushlock side table at the `ExAcquire/ReleasePushLock*` funnel, checked at the park). `vibeos-lockdep [held|edges]` decodes the table / dumps the whole lock-order graph (ABBA detection) offline in GDB. (Design: `archive/sync-instrumentation-design.md` §4.)
- **Self-symbolicating `KeBugCheckEx`** — see §9.17's banner note (line ~537 of the rustconv catalog too): params symbolicated vs `PsLoadedModuleList`, `TrapFrame.Rip` extracted for Mm bugchecks, scan-based mini-backtrace on serial — no GDB, no cross-boot base guessing.

**Reusable debug recipe:** boot under `vibeos-cli gdb`; wait for `[CENSUS] STUCK` (the wedge self-reports, with a lost-wakeup-vs-holder-stuck verdict); `vibeos-cli gdb inspect <run> "raw vibeos-synctrace"` → stuck objects; `"raw vibeos-flightrec obj=0x…"` → who locked/signalled last; `"raw vibeos-threads gatewait"` → all parked threads offset-free. For the deterministic-bugcheck flavor, prefer `run-until KeBugCheckEx --then triage` + trap-frame + **self-reporting diagnostic `KeBugCheckEx(0x0BAD…)` codes** to bisect the fault path (survives VM reaping, unlike post-mortem GDB). ~30 min end-to-end; the identical conclusion previously took days.

**Durable lessons (recurring class):** (a) hand-rolled offset tables and bit tests DRIFT — anywhere a bindgen struct exists, use it; (b) when a hack "rejects garbage" reads of a field, suspect the OFFSET, not the data; (c) `OriginalPte` in this port is a **quad-duty union** (proto-template protection for active prototype pages / pagefile descriptor for private pages / color-list Flink for free pages / `RmapListHead` for ROS PFNs) — any writer must know which role the page is in; see §9.14 for the sibling standby-round-trip bug.

### 9.14 amd64 BUGCHECK 0xBE (write to read-only) on a session/GDI view — the `/* ReactOS Hack */` in `MiUnlinkPageFromList` destroys a prototype page's protection across a standby round-trip

**Symptom.** After the PFN-state cleanup landed, amd64 smoke deterministically
`BUGCHECK 0x000000BE (0xfffff97fde280000, 0x…6021, …, 0xC)` — a write to a **read-only**
session-space page (`0xfffff97f…` = win32k session view) that was created from a
`PAGE_READWRITE` section. The PTE was present+accessed but not writable. i386/arm64 were green.

**Root-cause technique (the decisive trace).** Session PTEs can't be read from a
halted GDB context (the bugcheck CPU isn't in the target session; session space is
unmapped there), so instrument the fault-completion path instead. Session-gated
`dbg_print!` in `MiResolveProtoPteFault` (compute site) and `MiCompleteProtoPteFault`
(consume site), logging the derived `Protection`, the `OriginalProtection` branch, and
`Pfn1->OriginalPte`, for `Address >= 0xFFFFF90000000000`. The faulting page stood out
against every healthy neighbour by exactly one field:
```
healthy:  Complete VA=…de290000 prot=4 origProt=1 origPTE=0x80   → MM_READWRITE ✓
FAULTING: Complete VA=…de280000 prot=0 origProt=1 origPTE=0x00   → MM_NOACCESS ✗
```
`MiCompleteProtoPteFault`'s else-branch (`PointerPte.PageFileHigh != MI_PTE_LOOKUP_NEEDED`,
i.e. a pure prototype-pointer PTE) reads protection from `Pfn1->OriginalPte`. Healthy
first-fault (demand-zero) pages carry `OriginalPte=0x80` (`MM_READWRITE<<5`); the faulting
page was a **transition** page (trimmed to standby, brought back — its Resolve `protoPTE`
was a transition PTE `…882`, Transition=1/Prototype=0/Prot=RW) whose `OriginalPte` had been
**zeroed to 0** → protection `MM_NOACCESS` → PTE mapped non-writable → first win32k/GDI write
faults.

**Root cause.** `MiUnlinkPageFromList` (pfnlist.rs) ends with a line the C literally labels
`/* ReactOS Hack */`: `Pfn->OriginalPte.u.Long = 0`. That is correct only for *private*
(modified-list) pages. **Standby/modified lists link via `u1.Flink`/`u2.Blink` (the
`flink()`/`share_count()` accessors) — `OriginalPte` is NOT the list linkage for them; only
the free/zeroed COLOR lists reuse `OriginalPte` as their Flink.** For a shared **prototype**
page, `OriginalPte` holds the segment prototype PTE that encodes the page protection, and
`MiCompleteProtoPteFault` reads it back every time the page is faulted in from transition.
Zeroing it on the standby→active unlink discards the protection.

**Fix.** Guard the hack so it skips prototype pages:
```rust
if !(*Pfn).prototype_pte() {
    mmpfn_set_original_pte_long(Pfn, 0);   // the /* ReactOS Hack */ — private pages only
}
```
(`sdk/lib/rust/ntoskrnl-rust/src/ARM3/pfnlist.rs`, `MiUnlinkPageFromList`.) Preserving
`OriginalPte` for prototype pages is safe: any later free-list insertion overwrites it
(`MiInsertPageInFreeList` sets it to `LIST_HEAD`; the color path re-links it), and
`MiInitializePfn` re-sets it on re-init. **Verified:** the previously-faulting page then
resolves `prot=4 origPTE=0x80`, no 0xBE, boot reaches `_tWinMain`; i386/arm64 gate-green.

**Takeaway.** `OriginalPte` in this port is triple-duty (proto-template protection for active
prototype pages / pagefile descriptor for private pages / color-list Flink for free pages).
Any code that writes it must know which role the page is in. A blanket "hack" that zeroes it
is a landmine for the prototype-page role.

### 9.15 (FIXED 2026-07-04) amd64 BUGCHECK 0x4E `PFN_LIST_CORRUPT` at process-exit teardown — `MmInitializeProcessAddressSpace` never init'd the hyperspace PDPT's PFN on amd64 (cfg-drift: the `_MI_PAGING_LEVELS == 4` PXE block was gated `aarch64`-only)

**Exposed by** the 9.14 fix (boot now reaches process exits it never used to).
**Symptom.** Deterministic `BUGCHECK 0x0000004E (0x98, 0x01ffffffffffffff, 0, 0)` shortly
after `_tWinMain`, during the first full user-process teardown (`setup.exe`/`winlogon.exe`
exit). Subcode `0x98` = `MiDecrementShareCount` entry validation; `Pfn1=0`,
`PageFrameIndex=0x01ffffffffffffff`.

**The COLORED_LIST_HEAD fingerprint was a red herring.** The bogus `0x01ffffffffffffff`
argument is not itself the corruption — it is whatever garbage sits in the *uninitialized*
`u4.PteFrame` after `MI_SET_PFN_DELETED` munges it. Live DIAG (guarded `dbg_print!` at the
three `(*Pfn).pte_frame()`-reading teardown sites — `MmDeleteProcessAddressSpace` ×2,
`MiRemoveWsle` shrink, valid-PTE-list drain) pinned the crashing read to
**`site=HYPER`** = `MmDeleteProcessAddressSpace`'s second decrement,
`Pfn1 = DirectoryTableBase[1]` (the process hyperspace **PDPT**). The DIAG printed
`loc=0 ref=0` — the PDPT page's PFN entry was **never initialized** (still reads as a free
zeroed-list page with a garbage frame), not double-freed.

**Root cause (cfg-drift translation bug).** `MmInitializeProcessAddressSpace`
(`ARM3/procsup.rs`) mirrors the C, which for `_MI_PAGING_LEVELS == 4` runs an *extra* block:
```c
#if (_MI_PAGING_LEVELS == 4)      /* amd64 AND arm64 */
    PointerPxe = MiAddressToPxe((PVOID)HYPER_SPACE);
    PageFrameNumber = PFN_FROM_PTE(PointerPxe);
    MiInitializePfn(PageFrameNumber, PointerPxe, TRUE);   /* inits the hyperspace PDPT PFN */
#endif
```
The port had gated this block **`#[cfg(target_arch = "aarch64")]` only** — dropping it for
amd64, which is *also* 4-level paging. So on amd64 the hyperspace PDPT (`DirectoryTableBase[1]`,
allocated in `MmCreateProcessAddressSpace` via `MiRemoveZeroPageSafe`/`MiRemoveAnyPage`, which
leave a page in a free-ish PFN state) never had `MiInitializePfn` set its `pte_frame` /
reference_count / `ActiveAndValid` location. Teardown then read that garbage `pte_frame` and
called `MiDecrementShareCount(MiGetPfnEntry(garbage)→NULL, garbage)` → 0x98. i386 (2-level in
this port's sense — no PXE level) legitimately skips the block, so it never bit x86-32; arm64
had the block and was fine. Classic per-arch `#if` that got collapsed to one arch in translation.

**Fix.** One line — extend the cfg to include amd64:
`#[cfg(target_arch = "aarch64")]` → `#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]`
on that PXE-init block in `MmInitializeProcessAddressSpace` (`ARM3/procsup.rs`). `MiAddressToPxe`
is already `#[cfg(any(x86_64, aarch64))]` in `ntkrnl_sys::mm`, so amd64 has the helper.
**Verified:** post-fix amd64 boots clean past the `setup.exe`/`winlogon` teardown to `shell
ready` / full desktop, DIAG fires 0×, no bugcheck.

**Lesson.** When a per-arch `#if (_MI_PAGING_LEVELS == N)` guards a block, remember BOTH amd64
and arm64 are 4-level — a `#[cfg(target_arch = "aarch64")]` that should have been
`any(x86_64, aarch64)` is an easy translation miss. Grep `MmInitializeProcessAddressSpace`-style
paging-level blocks for the same drift (see also §9.14 for the sibling Mm PFN-state fix).

**§9.15 addendum (2026-07-05): this drift is SYSTEMIC, not a one-off.** A workspace-wide
per-arch `#[cfg]` audit (`tools/scan_arch_cfg.py` +
[`per-arch-cfg-divergence-audit.md`](archive/per-arch-cfg-divergence-audit.md)) found the same
"4-level PXE block gated to ONE 4-level arch" pattern in **~8 more Mm functions, in both
directions** — `MiDeleteVirtualAddresses`/`MmProbeAndLockPages`/`MiCalculatePageCommitment`/
`MiQueryAddressState`/`MiIsEntireRangeCommitted`/`MiMakePdeExistAndMakeValid` drop the PXE
on **arm64** (x86_64-only), while `MiLock`/`MiUnlockVirtualMemory` drop it on **amd64**
(aarch64-only). All latent (the top-level entry is usually resident), all the same fix:
gate the PXE walk/validate/fault-in `any(x86_64, aarch64)`. **Durable rule: any PXE / top-
level-paging block must be `any(x86_64, aarch64)` — never one 4-level arch alone.** The
audit tool is the recurring check.

### 9.16 (FIXED 2026-07-04) arm64 BUGCHECK 0x4E `PFN_LIST_CORRUPT` (subcode 0x99) at `nettest.exe` teardown — arm64's `MiPteToPde` was one self-map recursion level too high (returned the **PPE**), so `MiDeletePte`'s proto branch decremented the page-**directory**'s share instead of the page table's

**RESOLUTION.** The whole `MiPteToX` navigation family was off by one level on aarch64 — in BOTH
copies (`sdk/lib/rust/ntkrnl-sys/src/mm.rs` and `ntoskrnl/include/internal/arm64/mm.h`):

| helper | correct (amd64, arch-independent recursion) | arm64 had (wrong) | actually returned |
|---|---|---|---|
| `MiPteToPde` | `MiAddressToPte(pte)` | `MiAddressToPde(pte)` | the **PPE** of X |
| `MiPteToPpe` | `MiAddressToPde(pte)` | `MiAddressToPpe(pte)` | the **PXE** of X |
| `MiPteToPxe` | `MiAddressToPpe(pte)` | `MiAddressToPxe(pte)` | self-map PXE junk |

The self-map recursion is pure VA-bit arithmetic — PDE-of-X **is** PTE-of-(PTE-of-X) — and the
PTE_BASE/PDE_BASE constants are identical to amd64, so there was never a reason for an arm64
branch. With the bug, `MiAddressToPde(PTE-of-X)` = `0xFFFFF6FB7DA00000 + (X>>30)*8` — for every
user VA under 1GB that's exactly `0xFFFFF6FB7DA00000`, whose `page_frame_number` is the **page
directory** for user [0,1GB). So:
- `MiCompleteProtoPteFault`'s PT-share `++` (via `MiAddressToPte`, correct) hit the true PT frame;
- `MiDeletePte`'s proto-branch `−−` (via `MiPteToPde`, wrong) hit the **PD** frame (`0x46720`) —
  one wrong decrement per deleted proto page, while the PT's share leaked high.
The PD's share crossed 0 with live PDEs remaining (`usedpte=2`) → PD listed on ModifiedPageList →
next `MiDeletePte` decrements a listed page → `0x4E (0x99)`. The "ledger" below is fully explained:
the `site=initpfn` INCs were **PT-page creations** (parent-bump on the PD — correct), the non-proto
DECs were PT deletions (correct), and the 3 proto DECs were the wrong-frame decrements.

**Proof.** (1) Arithmetic: `MiAddressToPde(PTE-of-X) ≠ PDE-of-X` for all X (it equals PPE-of-X);
(2) live: instrumented boot printed `[PDEC] proto pte=fffff68000001350 va=000000000026a000
pdeshare=ffffffffffffffff` right before the 0x99 — a plain user VA whose PTE round-tripped to the
PPE, and the PD's share already **underflowed to −1**. amd64/i386 were immune (their `MiPteToPde`
was correct); the bogus-looking "user VA 0xF680'00000000" that the culprit PDE index decodes to is
just the PPE region's self-map offset — recognizing that decode was the pivotal clue.

**Debugging lessons.** (a) When a `++`/`−−` ledger shows a *systematic* per-page-kind imbalance,
suspect the two sides *resolving different frames* before suspecting a missing call — print FULL
64-bit pointers (`%p`, not the 32-bit `%lx`) and *decode the self-map index back to a VA*; the
earlier session "ruled out" this exact divergence because two *different* self-map objects share
the same low-32 bits (`0x7da00000`). (b) An `#[inline]` chokepoint (`MI_WRITE_VALID_PTE`) +
`asm!("adr {}, .")` PC capture is a cheap way to attribute every PTE write when you need it.

---

> _(The pre-RESOLUTION open-lead analysis — the per-PFN share-count ledger, the ruled-out RosMm/COW paths, the reproduction recipe — was compressed away 2026-07-05; git history holds it. The RESOLUTION above is the full story: `MiPteToPde`/`Ppe`/`Pxe` were one self-map level too high on arm64, so the proto-branch `−−` decremented the page-directory instead of the page-table.)_

### 9.17 amd64 intermittent post-marker 0x50 with a driver-region PC that symbolicates to garbage = wild control transfer INTO a mapped system-space DATA VIEW (win32k/GDI dispatch family, NOT the layout class)

**Symptom:** rare amd64 gate flake — `BUGCHECK 0x50 (referenced=0xfffff8a0…,
read, PC=0xfffff880_03xxxxxx)` AFTER `shell ready` (taskbar already created,
fonts just loaded); binary byte-identical to a green baseline; `addr2sym` /
`info symbol` returns garbage for the PC. Observed 2026-07-05 ~01:52 (ghosts
lane): `0x50 (0xfffff8a0003c6000, read, PC=0xfffff88003fb3050)`.

**Attribution recipe (validated end-to-end, ~15 min):**
1. `./vibeos-cli gdb start amd64` → `wait-for --pattern "shell ready"` → walk
   `PsLoadedModuleList` (qemu/runtime_symbolicate.py with
   `SYMIDX_TARGETS=<pc>`, or a hand LDR walk: DllBase@0x30 SizeOfImage@0x40
   BaseDllName.Buffer@0x60). Verdict here: PC is in NO loaded module — it lies
   BELOW the lowest session image (framebuf/dxg/ftfd/win32k stack at
   `0xfffff88004278000+`), in dynamic system-PTE space.
2. `x/16bx <pc page>` on the live boot: the page IS mapped and holds DATA
   (font/section-view bytes `05 04 00 03 08 08 08…`, zeros at the PC itself).

**Reading:** the faulting boot *executed data* — a corrupted function pointer /
dispatch-table entry / return address transferred control into a
`MmMapViewInSystemSpace` view (font file via ftfd, or a section win32k maps);
the "instructions" there did a wild read of non-resident paged pool → 0x50.
The PC is meaningless as code; the bug is whoever published the wild pointer.

**What it is NOT:** the captures(none)/layout class — ruled out the same night
by (a) zero live folds in the shipping-pipeline IR audit (scan_captures) and
(b) the layout canary booting 5/5 (this section's banner + "Incomplete Rust
cohab flip" ✅ CLOSED note).

**Suspect family:** win32k session wild-pointer corruption — the amd64 cousin
of the documented i386 `KeSignalGateBoostPriority: BAD gate` session-pool
UAF/wild-write flake (same family, different victim: i386 corrupts a gate,
amd64 corrupts a code pointer). Timing (right after WINLOGON font loads +
explorer taskbar) and the mapped-font-view landing zone both point at the GDI
font/driver dispatch path (PFE/PDEV/DRVFN tables — cf. the long-standing
`LDEVOBJ_bEnableDriver` `Option<unsafe extern fn>` PFN/DRVFN layout suspect).
Next diagnostic on a repro: special-pool / GDB watchpoint on the dispatch
table slot, not PC symbolication.

**Candidate now fixed (re-verify before assuming this is still open):** the
`addr_of!`-on-pointer-field clip-region overrun landed on master 2026-07-05
(rustconv bug catalog, "§`addr_of!(field)` where C passes the pointer-field's
VALUE" — win32k `CLIPPING_UpdateGCRegion` overran a paged-pool REGION alloc by
`nCount*16 - remaining` bytes). That overrun corrupts *adjacent* paged pool in
exactly the GDI-region-activity window this 0x50 fires in — a very plausible
source of the "whoever published the wild pointer" above. A full ×3 gate on
2026-07-05 (post-fix, all arches green) did **not** reproduce this 0x50; treat
9.17 as *likely-closed-pending-recurrence* rather than confirmed-fixed (it was
rare to begin with, so one clean gate isn't proof). Re-open with the attribution
recipe if it recurs.

### 9.18 Special pool is LIVE — arm it per-boot via the fw_cfg tripwire channel (no media rebuild)

**The tool §9.17 asks for exists now (2026-07-05).** Special pool
(`ARM3/special.rs`) was fully translated but dead — `MmSpecialPoolTag` was
never set by anything. The tripwire channel arms it per boot:

```bash
# One suspect tag (as DISPLAYED in pool dumps, e.g. Gla5 / Usqu):
./vibeos-cli smoke --arch amd64 --tripwire "SPECIALPOOL=Gla5"
# Every eligible allocation (≤ PAGE_SIZE - header) — boots green to desktop:
./vibeos-cli smoke --arch amd64 --tripwire "SPECIALPOOL=*"
# Catch underruns instead of overruns:
./vibeos-cli smoke --arch amd64 --tripwire "SPECIALPOOL=Gla5 SPECIALPOOL_UNDERRUNS=1"
# Prove the detector end-to-end (deliberate corruption => bugcheck):
./vibeos-cli smoke --arch amd64 --tripwire "SPECIALPOOL=Vibe SPECIALPOOL_SELFTEST=guard"    # 0x50 AT the store
./vibeos-cli smoke --arch amd64 --tripwire "SPECIALPOOL=Vibe SPECIALPOOL_SELFTEST=pattern"  # 0xC1/0x23 at free
```

What you get on a hit, on serial, zero GDB: an overrun past the allocation
faults the **guard page at the corrupting instruction** — BUGCHECK 0x50 whose
banner symbolicates `TrapFrame.Rip` to the culprit; in-page slack corruption
bugchecks 0xC1 at free with case code (0x23 before-buffer / 0x24 after).
Underrun mode flips data to page start. Serial markers: `[TRIPWIRE] special
pool armed/ACTIVE` (assert these when scripting; `ERROR:` lines are the
fail-loud path).

Mechanics: harness passes `-fw_cfg name=opt/org.vibeos.tripwire,string=…`
(vm.py `_tripwire_args`, env `VIBEOS_TRIPWIRE`); the kernel reads it at
`MmArmInitSystem` Phase 0 (`ntoskrnl-rust/src/tripwire.rs`) BEFORE PTE sizing
+ `MiInitializeSpecialPool`. Transports: x86 port I/O 0x510/0x511; arm64 MMIO
0x09020000 (virt) via a Device-attr identity page the arm64 winldr maps
explicitly (`MempMapDevicePage`) — the signature probe guards non-QEMU.
Tokens are space/semicolon-separated; **no commas** (QEMU owns the
comma). Tags are `from_le_bytes` of the display form. Session-pool allocs are
NOT excluded — win32k allocations divert into (global) special pool, which is
exactly what the §9.17 / BAD-gate hunts want; watch for session-VA-assumption
side effects if a hunt behaves oddly.

Pair with the flake farm for rare flakes: `./vibeos-cli farm --arches amd64
--minutes 480 --tripwire "SPECIALPOOL=*"` soaks boots overnight into
`~/vibeos-flake-farm/corpus.jsonl` (stop code + symbolicated banner lines per
failure, artifacts preserved out of /tmp); `./vibeos-cli farm-report` clusters
failures into ranked (classification, stop, symbol) families.

### 9.19 (FIXED 2026-07-05) arm64 total-silence boot wedge = a context switch LEAKED exception-entry DAIF into a new thread — PSTATE.DAIF is NOT per-thread state

**Symptom:** boot wedges with serial completely silent (no BUGCHECK, no output at
all — the clock itself has stopped). Wedge point varies run-to-run (first hit:
right after `[WINLOGON] ARM64: cmd.exe created`). GDB shows the CPU parked with
`cpsr` DAIF bits ALL set (0x3C0-family = raw exception-entry mask) at a PC below a
`KiSwapContext` frame.

**Mechanism:** `KiSwapContext` does not save/restore PSTATE — whoever swaps in a
context hands it their own DAIF. Two facts make this lethal:
(1) on this port, **SVC handlers run DAIF-masked end-to-end**, so every syscall
wait parks a masked context (self-healing: it unmasks at ERET back to user);
(2) a **brand-new thread** (`KiThreadStartup`) inherits the swapper's DAIF and, if
it's a system thread, never ERETs — so if its first swap-in comes from a
masked-context wake chain it runs IRQ-masked *forever*. Once enough of the ready
set is masked, no clock tick ever fires again → machine dead, serial silent.
Trigger was the interrupt quest's IRQ-exit DPC retirement (`KiIrqHandler` →
`KiRetireDpcList` → `KiTimerExpiration` → thread wakes rooted in exception
context); before that, all wake chains were rooted in the idle loop, which
`daifclr`s before swapping — the landmine was unreachable.

**Fix (layered — the first alone is NOT sufficient):**
1. `KiThreadStartup` opens with `msr daifclr, #2` (ctxswitch.S) — threads start
   with IRQs enabled, matching NT. Necessary (closes the new-thread inheritance
   leak) but empirically insufficient: with exception-context timer wakes still
   enabled, boot still wedged at thread-lifecycle events (winlogon→cmd handoff
   on windows-host TCG; `LpcExitThread` during a System-thread exit on native) —
   masked wake/swap chains break more paths than new-thread startup.
2. **The IRQ-exit DPC retire excludes timer expiration entirely**
   (`TimerRequest == 0` guard in `KiIrqHandler`, trap.rs): `KiTimerExpiration` —
   the thread-wake machinery — must NOT run from exception context on this port
   until SVC entry re-enables IRQs (arm64-interrupts quest doc, landmine #2).
   Timer expiration stays with the idle loop (pre-quest semantics); only plain
   device DPCs retire at IRQ exit.

**Detector left in tree:** masked-swap tripwire in `KiSwapContextResume`
(thrdini.rs) self-reports `[SWAP-TRIPWIRE]` on serial if a masked swap ever
happens under the IRQ-exit retire again; the IRQ-exit retire must also use
`HalpEnterInterruptIrql`/`HalpExitInterruptIrql` (NOT `KeLowerIrql` — its
soft-interrupt drain delivers APCs in exception context).

**Debug recipe that cracked it** (generalizes to any silent arm64 wedge):
`RUN=$(qemu/start.sh arm64 --gdb)` → `qemu/wait-for.sh $RUN --pattern <last-line>`
→ `qemu/inspect.sh $RUN --command regs` — read `cpsr` FIRST: DAIF=0xF means "who
masked me" (this class), not "what crashed". Then instrument the save side
(tripwire print in `KiSwapContextResume`) rather than post-morteming the resume side.

### 9.20 (FIXED 2026-07-06) arm64 ISR-queued DPCs never ran at desktop — device-DPC delivery gated on a starved `TimerRequest` flag

**Symptom.** A device ISR fires and `KeInsertQueueDpc` returns 1 (queued), then
returns 0 forever after (`Dpc->DpcData` non-null = still queued): the DPC never
runs. Worked fine at DriverEntry time; broke only once the desktop was up.
First hit: virtio-input's ISR→DPC conversion — mouse IRQs delivered and acked,
`test_arm64input_delivery` red because the drain DPC never retired.

**Why (three stacked facts, each individually reasonable):**
1. On this port there were exactly TWO DPC-retire sites: the idle loop
   (`KiIdleLoop` → `KiRetireDpcList`) and the interrupt quest's IRQ-exception-exit
   drain (`KiIrqHandler`). The DISPATCH software interrupt is a no-op —
   `HalpDispatchSoftwareInterrupts` (arm64 halstubs) delivers only the APC bit,
   so `HalRequestSoftwareInterrupt(DISPATCH_LEVEL)` sets a bit nothing consumes.
2. At desktop a busy thread that never blocks (a user thread hammering
   `NtQueryPerformanceCounter` — 75% of PC samples inside hal
   `KeQueryPerformanceCounter`) starved the idle thread INDEFINITELY. Idle
   swap-out happened once at boot and it never resumed. Idle-loop retirement: gone.
3. The IRQ-exit drain was gated on `TimerRequest == 0` (to keep KiTimerExpiration
   out of exception context — the §9.19 wedge). But `TimerRequest`'s ONLY
   consumer is `KiRetireDpcList`, i.e. the idle loop — so with idle starved,
   `TimerRequest` sat non-zero FOREVER (it holds a stale TrapFrame pointer, not
   a bool!), and the gate turned "timer bookkeeping is stale" into "no device
   DPC ever runs".

**Fix.** `ke/dpc.rs::KiRetireDeviceDpcs` — drains ONLY `DpcData[DPC_NORMAL]`'s
list (never touches `TimerRequest`/`KiTimerExpiration`), called from
`KiIrqHandler` exit with the `TimerRequest == 0` gate REMOVED (guards kept:
interrupted IRQL < DISPATCH, `DpcRoutineActive == 0`). Device-DPC delivery must
never depend on timer bookkeeping. Timer expiration itself stays idle-loop-only
(exception-context KiTimerExpiration still wedges — §9.19 / quest landmine #2).

**Debug recipe (generalizes to "who eats my DPC/why is the queue stuck"):**
- Rate-limited prints at the three actors: insert result in the ISR, a
  first-N print in the DPC body, and a "skip-retire: irql=%u active=%u timer=%u
  depth=%d pc=%p" print in KiIrqHandler when the queue is non-empty but the
  drain is skipped. The `pc=` of the skip print names the loop the CPU parks
  in (it's the interrupted context of every IRQ) — one boot pinpointed both the
  stuck `TimerRequest` and the QPC busy-spinner.
- `qemu/sample-pc.sh <run-id>` (GDB toolkit) for a PC histogram of where the
  CPU actually lives; `[IDLE] enter/beat/swap-out/resume` prints in KiIdleLoop
  answer "does idle ever run" definitively.

**Both follow-ups CLOSED 2026-07-06 (same-day hunt, §9.21):** the
"unidentified QPC busy-spinner" turned out to be TWO stacked spinners with one
root — `KeDelayExecutionThread` was still stubbed to `NtYieldExecution()` on
arm64 ("no HAL timer interrupt yet" — dead lore), so EVERY kernel/user sleep
busy-yielded; and explorer's main thread separately spun on a stuck QS_PAINT
wake bit. The DISPATCH soft-interrupt delivery gap is also closed —
`KfLowerIrql` now delivers timer expiration + device-DPC drain in thread
context. Full writeup: §9.21.

### 9.21 (FIXED 2026-07-06) arm64 desktop timer freeze — `KeDelayExecutionThread` busy-yield stub + stuck QS_PAINT spin + no DISPATCH softint delivery

**Symptom.** At desktop: taskbar never populates / clock frozen / no caret,
70s-apart QMP screendumps byte-identical, `sample-pc.sh` shows ZERO idle-loop
samples — CPU 100% split between hal `KeQueryPerformanceCounter` (~70%, a
"System" thread) and one user PC (~20%, deterministic across boots).

**Three stacked bugs, one visible freeze:**
1. **`KeDelayExecutionThread` was a bring-up stub on arm64**
   (`ke/wait.rs`, inherited from C `#ifdef _M_ARM64`: "no HAL timer interrupt
   yet — would sleep forever"): `NtYieldExecution(); return SUCCESS`. Every
   kernel sleep (netstack's `delay_ms` poll loop → the hal-QPC-hot System
   thread) and every user `Sleep`/`NtDelayExecution` became a busy-yield.
   Ready threads yielded to each other forever → idle starved → idle-only
   timer expiration frozen → every real timer wait dead. The stub's premise
   died when the interrupts quest wired the vtimer: `KeTickCount` and
   `KUSER_SHARED_DATA.InterruptTime` demonstrably advance. Fix: deleted the
   stub; the real timer-wait path just works.
2. **Explorer's main thread spun on a stuck `QS_PAINT` wake bit** (the ~20%
   user PC = user32 `NtUserPeekMessage` syscall stub; identified via
   KTHREAD→ETHREAD walk + PEB module-list walk from a KiIrqHandler
   breakpoint). `pcti->fsChangeBits/fsWakeBits == 0x20 (QS_PAINT)` with
   `pti->cPaintsReady == 0`: `MsgWaitForMultipleObjects` returns "input
   available" immediately (fsChangeBits early-out) while `PeekMessage`
   retrieves nothing → infinite spin. Source of the stale state:
   `IntGetPaintMessage`'s upstream dirty-window-leak bandaid zeroes
   `cPaintsReady` WITHOUT clearing the wake bits — and fired invisibly
   because win32k-rust's `ERR!` macro is a NO-OP. Fixes (win32k-rust): the
   bandaid now clears the QS_PAINT bits too (painting.rs), co_IntPeekMessage
   reconciles stale QS_PAINT bits when `cPaintsReady == 0` (message.rs), and
   the bandaid prints via real `DbgPrint` so the underlying leak is visible.
3. **DISPATCH software-interrupt delivery implemented** (closes §9.20 (b)):
   `HalRequestSoftwareInterrupt(DISPATCH)`'s pended bit is now consumed by
   `KfLowerIrql` when the soft IRQL crosses below DISPATCH — hal calls the
   new ntoskrnl export `KiDispatchSoftwareIntArm64()` (ke/dpc.rs): TimerRequest
   → `KiTimerExpiration` (with `DpcRoutineActive=1` so wakes DEFER, mirroring
   the idle loop) + `KiRetireDeviceDpcs`. Timer expiration no longer depends
   on the idle loop — a busy thread expires timers from its own syscall/
   spinlock IRQL crossings.

**The §9.19 trap, round two (BUGCHECK 0x1E, EC=0x0E illegal-execution-state):**
the first delivery attempt ran timer expiration from EXCEPTION context,
because `KfLowerIrql` is reachable from inside IRQ windows — notably the
clock-tick branch (`HalClockInterruptArm64` → `KeUpdateSystemTime`) runs with
NO IRQL bracket at all, so any raise/lower pair inside it crosses below
DISPATCH. Fix: hal tracks `HalpInterruptContextDepth` — bumped vector-wide in
`HalClockInterruptArm64` (drop-guard) and by every
`HalpEnterInterruptIrql`/`HalpExitInterruptIrql` bracket — and
softint delivery requires depth == 0 (genuine thread/SVC context).

**Verified:** boot green ~36s; clock IRQ now interrupts the *Idle* process;
`KiTimerExpiration` + win32k `ProcessTimers` fire repeatedly (GDB breakpoints);
timer table shows live sub-second traffic; explorer main blocks in
`GetMessage`; netstack worker alive in real `delay_ms` waits (WaitReason=4);
`test_arm64input_delivery` green.

**Follow-up CLOSED 2026-07-06 (§9.23): tray clock / window repaint freeze was
the WRONG ACTIVE INPUT DESKTOP, not timer delivery.** See §9.23 — the timer
path was fine all along; `gpdeskInputDesktop` was stranded on "Winlogon" while
the shell runs on "Default", so win32k's paint retrieval walked the wrong
desktop tree. Fixed in winlogon (arm64 boot path). The `cTimersReady`-stays-0
observation was a red herring (measured mid-consume); WM_TIMER *does* post.

**Debug recipe additions** (beyond §9.20's): identify a user-PC spinner from a
`KiIrqHandler` breakpoint — read `KPCR(tpidr_el1)+0xD80 → Prcb.CurrentThread
(+8) → KTHREAD.Process(+512) → EPROCESS.ImageFileName(+608)`, offsets from
`ntkrnl-sys/src/offset_check_arm64.rs`; walk the user module list
(`TEB(+176)→PEB(+0x60)→Ldr(+0x18)`) to name the module at a user PC; walk
`PsLoadedModuleList` for kernel/driver PCs (module bases move per boot — never
reuse a previous boot's symbolication). `gdb` can't read kernel VAs while
stopped at EL0 — break on a kernel PC first. And REBUILD THE SYMBOL INDEX
after every build: a stale `sym2addr` address put a 5-minute breakpoint on
dead code and nearly "proved" timers were frozen when they weren't.

### 9.23 (FIXED 2026-07-06) arm64 desktop never repaints — wrong ACTIVE INPUT DESKTOP + SystemTime never seeded from the RTC

Two independent bugs behind "the arm64 desktop is frozen at boot": the tray
clock never ticked (both a wrong *value* and no *repaint*), and no window ever
redrew after its first paint (notepad title bar / taskbar bare).

**Bug A — window paint freeze: `gpdeskInputDesktop` stranded on "Winlogon".**
The shell (explorer.exe) is launched on `WinSta0\Default`, but the kernel's
*active input desktop* (`gpdeskInputDesktop`, win32k `desktop.rs`) was the
**Winlogon** desktop. `IntGetPaintMessage` → `IntFindWindowToRepaint(
UserGetDesktopWindow(), pti)` starts the repaint walk from the *global active*
desktop's window — so it walked the (empty) Winlogon tree and never found
explorer's dirty windows on Default. Every repaint silently no-op'd, and the
"`cPaintsReady=%u but no dirty window`" bandaid (§9.21) fired forever. The
taskbar you *see* is stale pixels drawn once — arm64 skips `IntHideDesktop`, so
switching away never cleared them.

*How it got stranded:* the arm64 winlogon shortcut path (`winlogon.rs`, no
GINA/logon) relies on `CreateWindowStationAndDesktops` switching to Default,
but `RunSetup`'s async thread (`setup.rs`) switches to Default then, on the
setup-shell `CreateProcess` **failure** path, `SwitchDesktop(WinlogonDesktop)`
— leaving the input desktop on Winlogon. Nothing switched it back before the
shell came up.

*Root-cause proof (the technique):* from GDB, walk the win32k timer list to the
explorer clock timer → its `pti` → `THREADINFO.rpdesk (+0x78)`; compare to
`gpdeskInputDesktop` (win32k `+0x131978` this build). They differed
(`rpdesk`=Default, active=Winlogon). Read each `_DESKTOP`'s object name by
scanning ~0x80 bytes before the body for a `UNICODE_STRING{len,max,ptr}` →
`[Winlogon]` vs `[Default]`. Then **overwrite** `gpdeskInputDesktop` with the
Default pointer and wait one 60 s timer tick — the clock repainted (`6:35`→
`6:47`) and the paint-leak count stopped climbing. That poke *is* the fix,
proven live before touching source.

*Fix (winlogon-rust):* after spawning explorer in the arm64 boot path,
`SwitchDesktop((*WLSession).ApplicationDesktop)` so the active input desktop is
the shell's desktop (what a normal logon reaches via `WlxSwitchDesktopToUser`);
and skip `setup.rs`'s failure-path `SwitchDesktop(WinlogonDesktop)` on arm64
(no GINA UI to fall back to) so the async setup thread can't re-strand it.

**Bug B — SystemTime never seeded: the "Query the clock" block was dropped in
translation.** `Phase1InitializationDiscard` (C `ex/init.c`) seeds boot time
from `HalQueryRealTimeClock` → `KeSetSystemTime`; the whole `if
((ExCmosClockIsSane) && (HalQueryRealTimeClock(&TimeFields)))` block was missing
from `ex/init.rs`, so `SharedUserData.SystemTime` accumulated from 0 ==
1601-01-01 (tray clock stuck at "12:00 AM"). This hit **all three arches**
(x86/amd64 too — no separate seed path existed). Fix: restored the block in
`ex/init.rs`. On arm64 the HAL RTC stub returned a hardcoded 2025-01-01;
`hal-rust/src/arm64/halstubs.rs::HalQueryRealTimeClock` now reads the QEMU virt
**PL031** RTC (`DR` @ `0x0901_0000`, seconds since the Unix epoch) with a
civil-date conversion, so arm64 shows real wall-clock time.

**Verified:** clean boot (no GDB), zero `no dirty window` leaks (was 4/boot),
tray clock shows real RTC time and ticks on its own (`6:56`→`6:58`), full
notepad title bar + Start button + taskbar button render. `IntGetPaintMessage`
walking the wrong desktop is the general shape — if windows draw once then
freeze, check `gpdeskInputDesktop` vs the shell thread's `rpdesk` before
suspecting the timer/paint bookkeeping.

### 9.22 Driver hardware-settle delays are pure loss in a VM — gate them on the driver's own VM detection (uniata cost 2.85s/boot)

**Symptom shape:** a boot phase is "slow" with zero serial output and no
faults — just wall-clock disappearing inside a driver's init/probe path.
Under TCG every `KeStallExecutionProcessor` microsecond is real (and costs
~1.7× nominal once the port-I/O in the polling loop is added).

**The class:** hardware drivers inherited from real-metal code are full of
fixed settle delays and worst-case polling budgets (wait-for-BUSY-assert,
post-reset mechanical settle, bus-settle stalls). Virtual devices complete
these operations instantly, so the *entire* budget burns on every boot.
The tell that it's this class and not a real wait: the delay is identical
run-to-run and the device works fine afterwards.

**Recipe:** (1) drop permanent `DbgPrint` phase markers at the suspect
driver's probe/init milestones (entry/exit of each stage), boot once with
`VIBEOS_SERIAL_TIMESTAMPS=1 ./vibeos-cli smoke`, read the per-stage deltas
from the live log. (2) For each hot stall, gate it on the driver's own VM
detection rather than deleting it — most storage/NIC drivers already detect
the hypervisor (uniata: `g_opt_VirtualMachine`, set from PCI subsys
1af4:1100 for QEMU).

**uniata instance (fixed 2026-07-06, amd64/i386):** 12 × `AtapiSoftReset`
each burned a 100ms wait for BUSY to *assert* after ATAPI RESET (QEMU never
asserts it; Bochs already had an `i=100` shortcut — QEMU is the same IDE
lineage, extend the shortcut: uniata-rust `id_ata_part1.rs`) ≈ 2.0s, plus an
unconditional 500ms "mechanical settle" per ATAPI lun in
`AtapiHwInitialize__` (`id_ata_part2.rs`) ≈ 0.88s real. Probe: 2.35s →
120ms; FindDevices→INQUIRY-complete: 880ms → 4ms. The `[UNIATA]` markers are
permanent — future regressions in this window are attributable from any
timestamped smoke log.

### 9.24 (FIXED 2026-07-06) i386 lost-wakeup wedge + "BAD gate" + BSOD roulette — bindgen DROPPED `__ALIGNED(16)` on `EX_PUSH_LOCK_WAIT_BLOCK`

**Symptom family (all one root):** post-`shell ready` i386 boots either (a)
bugcheck-roulette'd — `0x7F` #GP in `KiFlushNPXState`, `0xA Eip=0x10c`
(`KiSystemCallReturn` jmp through a corrupt kernel trap-frame Eip), `0x1E
c0000006` in `ExfWakePushLock`/`KeSignalGateBoostPriority` reading freed-stack
VAs, `0xF4` — or (b) wedged: `[CENSUS] STUCK winlogon/LSASS ... GateWait ...
verdict=HOLDER-STUCK(no signal ever)` with an empty desktop (explorer alive,
no icons/taskbar). Historic `KeSignalGateBoostPriority: BAD gate ... Type=<garbage>`
prints are the same bug.

**Root cause:** the pushlock word encodes 4 FLAG bits (Locked/Waiting/Waking/
MultipleShared) in the LOW NIBBLE of the wait-block pointer, so
`EX_PUSH_LOCK_WAIT_BLOCK` is `__ALIGNED(16)` in C (ndk/extypes.h). **bindgen
dropped the alignment attribute** (`#[repr(C)]` only, natural align 4 on
i386), so the Rust stack locals in `ExfAcquirePushLock{Exclusive,Shared}` /
`ExpBlockOnLockedHandleEntry` landed at `addr % 16 ∈ {4,8,12}`:

- block at `%16==4` → pointer bit2 reads as **Waking** → every releaser skips
  `ExfWakePushLock` ("already waking") → waiter parks forever = the wedge.
- chain walks decode the block base with `& ~0xF` → ±4/±8 off → garbage
  `Next/Last/WakeGate` → BAD-gate prints, wild reads of freed stacks, and the
  crash roulette above.
- amd64/arm64 natural align is 8 → `%16==8` → pointer bit3 reads as
  **MultipleShared** — subtler flake source on those arches too.

**Fix:** `#[repr(C, align(16))] AlignedPlWaitBlock(EX_PUSH_LOCK_WAIT_BLOCK)`
wrapper at all three stack-allocation sites (ex/pushlock.rs ×2, ex/handle.rs).
`ExBlockPushLock`'s `debug_assert!((ptr & 0xF) == 0)` was already there — it
compiles out in release; the C ASSERT does too.

**Class lesson + sweep:** bindgen silently drops `__attribute__((aligned)))`
on some structs — and the size-assert generator shares bindgen's view (both
said 52 where C sizeof is 64), so size asserts DON'T catch it. Swept
`sdk/include/ndk|xdk` for `__ALIGNED|DECLSPEC_ALIGN` struct decls:
`EX_PUSH_LOCK_WAIT_BLOCK` was the only kernel type affected. If you add such
a type, verify `align_of` in Rust matches, or wrap at the use site.

**Debug recipe that cracked it** (generic for any lost-wakeup): census/`STUCK`
lines give the parked GATE (per-wait stack object — useless as a key) — log
the PUSHLOCK address at park time (`synctrace_pl_park` + temp `[PLPARK]`
dbg_print), then `qemu/inspect.sh <run> --command raw "x/8wx <lock>"` on the
wedge: lock word `0` with a parked waiter = lost wakeup; lock word =
`<block>|flags` with a MISALIGNED block address = this bug. Also:
`dump-thread` → `WaitReason 0x23/0x1c` distinguishes guarded-mutex vs pushlock
chains, and `KGUARDED_MUTEX.Owner` (+4) names the holder for the chain walk.


### 9.25 (FIXED 2026-07-06) i386 NPX `Cr0NpxState` garbage → 0x7F GP in `KiFlushNPXState` — hand-rolled `FX_SAVE_AREA` local structs modelled the union U as 512 bytes, but the C `FXSAVE_FORMAT` is **520**

Same bindgen-drop *class* as §9.24, different struct. After the §9.24 pushlock fix
got i386 to `shell ready`, a `BUGCHECK 0x7F (0x0000000d GP)` fired in
`KiFlushNPXState` at the `mov cr0, <val>` — `val` was a pointer-shaped garbage
(observed `0x7ffea000`, a user Teb) OR'd into CR0's reserved bits.

**Root:** the C `_FXSAVE_FORMAT` (`sdk/include/ndk/i386/ketypes.h`) carries a trailing
`Align16Byte[8]` after `Reserved4[224]`, making it **520 bytes (0x208)**, not 512. So in
`FX_SAVE_AREA`, the union `U` is 520, putting `NpxSavedCpu@0x208` and `Cr0NpxState@0x20C`,
with `sizeof(FX_SAVE_AREA)=528 (0x210)` — which `KiGetThreadNpxArea` confirms
(`InitialStack - 0x210`) and bindgen models correctly (size-asserted 528, `Cr0NpxState`
offset-asserted 524). `traphdlr.rs`/`usercall.rs` correctly use the bindgen `_FX_SAVE_AREA`.

But **four files hand-rolled local `FX_SAVE_AREA`/`FXSAVE_FORMAT` with a short U**, so every
`Cr0NpxState`/`NpxSavedCpu` access landed 8 bytes low, inside the FXSAVE register/align
region → pointer garbage:
- `ke/i386/cpu.rs`   — `U:[u8;512]` → `Cr0NpxState@0x204` (this is the `KiFlushNPXState` reader that GP-faulted)
- `ke/i386/exp.rs`   — a hand-rolled `FXSAVE_FORMAT` of only **288 bytes** (truncated `Reserved4[128]`, missing `Reserved4[224]`+`Align16Byte[8]`) → `Cr0NpxState@0x124`
- `ke/i386/kiinit.rs` — `FX_AREA{_padding[24], MXCsrMask}` → read `MXCsrMask@0x18` (that's `MXCsr`), stashed `0x1F80` in `KiMXCsrMask`
- `ke/i386/v86vdm.rs` — `[u8;512]` → `sizeof` for a `RtlCopyMemory(FX_SAVE_AREA)` short by 16

**Fix (commit `88ee2c3461c`):** all four sized to the true 520/528 C layout. Boot confirms
the `[NPX] Cr0NpxState GARBAGE` prints are gone (0 hits). **Lesson:** whenever you see a
hand-rolled `FXSAVE_FORMAT` in a translation, check for the `Align16Byte[8]` tail — it is
easy to drop and size-asserts on the *local* struct won't catch it (there are none). Prefer
the bindgen `_FX_SAVE_AREA`/`_FXSAVE_FORMAT` (size-verified) over hand-rolled copies.

### 9.26 (RESOLVED 2026-07-06) i386 `0x0A @ Eip=0x10c` ~2s after `shell ready` — stdcall→fastcall shim tail-`jmp` leaks 4 bytes of stack per call; `KiCheckForApcDelivery`'s esp-relative loop `ret`s to a stale slot

**ROOT CAUSE (found by re-reading the victim frame, not by chasing the writer):** the
i386 stdcall wrappers `_KfRaiseIrql@4` / `_KfLowerIrql@4` in
`sdk/lib/rust/ntkrnl-sys/src/inline/ke_i386.rs` were

```asm
_KfRaiseIrql@4:
  mov ecx, [esp+4]
  jmp @KfRaiseIrql@4      ; ← BUG: tail-jump to the FASTCALL impl
```

A fastcall function whose args all fit in registers ends in plain `ret` (cleans 0
stack bytes) — but the stdcall shim's caller pushed 4 bytes and expects `ret 4`.
Tail-jumping hands that `ret` straight back to the caller, so **every call leaves ESP
4 bytes low**. Nearly all callers are compiler-generated (EBP-framed) and silently
self-heal at `mov esp, ebp`. The one that can't: `KiCheckForApcDelivery` (hand-asm,
esp-relative, same file) — its user-APC loop calls `_KfRaiseIrql@4` **and**
`_KfLowerIrql@4` (−8/iteration), then its exit `pop ebx/pop edi/pop esi; ret 4` pops
shifted slots and `ret`s to the saved-EDI slot, which held **0x10c** (a stale local —
NOT a written value; that's why every "find the 0x10c writer" experiment came up
empty). After the bogus ret, `[esp]` = `KiServiceExit+0x1b` — exactly the observed
post-crash stack. Fires only when a **user** APC is pending at syscall exit (rare;
first ~2s after `shell ready` during CDefView startup), hence the delayed, determinis-
tic crash. The suspend-APC association from the `KiLastApc*` recorder was a red
herring (it records the last *kernel* APC; the crashing pass is the user-APC loop).

**Fix:** shims now `call @Kf*Irql@4; ret 4` (clean their own frame). Sweep rule: a
stdcall→fastcall thunk may NEVER tail-jump unless the fastcall target also cleans
exactly the shim's stack bytes; audited all other `jmp @…@N` sites (`KiExceptionExit`,
`ctxswitch.S`) — noreturn tail-dispatches, fine.

**Debugging lessons (why this took multiple sessions):**
1. **A structured "garbage" value (small constant like 0x10c) on the stack is not
   necessarily *written* — check for an ESP-shifted `ret` popping a stale local
   FIRST.** ESP imbalance self-heals in every EBP-framed function, so the crash only
   manifests in esp-relative asm — audit the hand-asm in the return chain before
   hunting phantom memory writers.
2. The decisive datum was the `[T0E]` kernel-stack ret-chain scan (dump words above
   fault-ESP in the #PF handler, filter to kernel-code range, symbolicate offline):
   `[fault_esp]=KiServiceExit+0x1b` pinned the crashing `ret` to one call-depth below
   `KiServiceExit` with zero locals in between → `KiCheckForApcDelivery` itself.
3. For kernel-mode traps the CPU pushes no SS:ESP, so `TF+0x74` overlays the
   interrupted stack top — the popped slot itself (at `TF+0x70`) is destroyed by the
   CPU's EFlags push. Scan from `TF+0x74` upward.
4. Disassemble the *linked binary* (`llvm-objdump` at `ImageBase+RVA`), not the
   source: the bug was visible only as `jmp` vs `call/ret 4` in the emitted shim.

(Original OPEN triage, kept for the record:) a thread-SUSPEND APC delivered on the syscall-exit path was believed to corrupt `KiDeliverApc`'s return chain across the suspend/resume context switch

The LAST i386 boot blocker after §9.24 + §9.25. i386 reaches the real `shell ready`
boot marker (desktop renders), then within ~2s (during `shell32!CDefView` /
`RegisterDragDrop` / OLE marshaling startup) BUGCHECKs `0x0000000A
(0x0000010c, 0xffffffff, 0x00000000, 0x0000010c)`.

**What it is (fully triaged; the hard part is *why*):**
- The fault is a **kernel-mode (SegCs=0x8) instruction fetch at `0x10c`** with **IF=0**
  (`EFlags=0x10046`). `KiTrap0EHandler` bugchecks `0x0A` at line ~796 precisely *because*
  IF=0 (it treats an IF-clear fault as `IRQL_NOT_LESS_OR_EQUAL`), so `cr2=Eip=0x10c`.
- IF=0 + kernel = a `cli`-protected region: `KiServiceExit` does `__disable` (cli) then
  `call KiCheckForApcDelivery` → `KfRaiseIrql(APC)` → `KiDeliverApc` → `KfLowerIrql`.
- Raw hardware-frame capture at the asm `KiTrap0E` entry (break `*<KiTrap0E>` **if
  `*(unsigned*)($esp+4)==0x10c`**, then `x/40wx ($esp+16)`) shows the pre-fault stack top =
  `KiServiceExit+0x1b` (the return addr right after `call KiCheckForApcDelivery`). **No
  register holds 0x10c** (edx is a trap-frame ptr) → it was a `ret`/`ret N` to a **corrupted
  stack slot = 0x10c**, NOT a `jmp reg`.
- A `[T0E]` diagnostic in `KiTrap0EHandler` (fires on `Eip<0x1000`) plus a `KiLastApc*`
  recorder in `KiDeliverApc` names the culprit APC every boot: **the thread-suspend APC**
  — `KernelRoutine=KiSuspendNop@20`, `NormalRoutine=KiSuspendThread@12`, `type=0x12`
  (ApcObject). `KiSuspendThread` blocks on the suspend semaphore (`KeWaitForSingleObject`)
  → context-switch out, then resume.
- The `prevTF` (thread->TrapFrame) is a **normal user syscall** (`Eip=0x7e001785`, `cs=0x1b`
  — an ntdll SYSENTER return), consistent across every boot.

**Ruled OUT (all verified):**
- Bad routine pointers — `[APC]` bogus-pointer check never fires; both routines are valid.
- ABI/cleanup mismatch — `KiSuspendNop` compiles to `ret 0x14`, `KiSuspendThread` to
  `ret 0xc` (correct stdcall); `PKKERNEL_ROUTINE`/`PKNORMAL_ROUTINE` bindgen types are
  `extern "stdcall"`.
- `Esp0`/NPX frame math — `KiSwapContextExit` sets `Esp0 = InitialStack - 0x10 - NPX_FRAME_LENGTH`
  = `InitialStack - 0x220`, which places the trap frame adjacent to the (correct, 528-byte)
  NPX area. `NPX_FRAME_LENGTH = size_of::<_FX_SAVE_AREA>()` = 528 (bindgen).
- The `KiServiceExit`/`KiServiceExit2` guards — the `[KTF]`/`[KX2]` `Eip<0x8000_0000` checks
  are placed correctly but *cannot* be the jumper: the kernel-path `jmp edx` is guarded and
  edx≠0x10c at fault; the observed transfer is a `ret`, not that `jmp`. The `KiServiceExit`
  frames on the stack are partly **stale** (reused kernel stack) — do not over-trust them.

**Current best hypothesis:** the suspend/resume **context-switch stack save/restore**
(`KiSwapContextEntry`/`Exit` Rust + `ctxswitch.S` `KiSwitchThreads` `mov esp,edx`) leaves
`KiDeliverApc`/`KiCheckForApcDelivery`'s saved return address = `0x10c` after the round-trip,
so the epilogue `ret`s to `0x10c`. Next steps for whoever picks this up: capture ESP
immediately across the `KeWaitForSingleObject`/context-switch in `KiSuspendThread` (or
`KiDeliverApc`'s `NormalRoutine` call) to prove the resume shifts the stack; then audit
`KiSwapContextEntry` (`OldThread->KernelStack = SwitchFrame`) vs the asm push/pop balance and
the resumed-thread ESP restore. Diagnostics are staged (uncommitted) in
`ke/i386/traphdlr.rs` (`[T0E]`, `[KX2]`, `[KTF]`, `[FCH]`), `ke/i386/thrdini.rs` (`[TCTX]`),
and `ke/apc.rs` (`KiLastApc*` recorder) — strip before landing. amd64/arm64 are unaffected
(green), so this is i386-specific (asm or i386 Rust translation).
### 9.27 Ground-truthing the "3 open corruption bugs" (2026-07-06) — 2 don't reproduce, 1 reframed, 1 live arm64 lead

A corruption-bug sweep with the special-pool tripwire (§9.18) + the arm64 IRQL
self-checks, run against current master, materially changed the picture the
stale open-bugs notes painted. Recorded so nobody re-chases the closed ones or
re-derives the reframing.

**Tool validation first (always).** `--tripwire "SPECIALPOOL=* SPECIALPOOL_SELFTEST=guard"`
→ BUGCHECK 0x50 as designed: the detector barks, so its *silence* is trustworthy.
NB the selftest needs a tag armed too (`SPECIALPOOL=*`) — alone,
`SPECIALPOOL_SELFTEST=guard` early-returns (no pool initialized) and boots clean,
which looks like a broken detector but isn't.

1. **i386 "session-pool gate" corruption (`KeSignalGateBoostPriority: BAD gate
   Type=128`) — REFRAMED, not reproducing.** *(Since root-caused + FIXED: bindgen-dropped `__ALIGNED(16)` — see §9.24.)* The fingerprint `Type=128`=0x80 is
   *not* random garbage: `Type & KOBJECT_TYPE_MASK(0x7F)` on 0x80 = 0, so the
   whole low type byte was zeroed with bit 7 set. Crucially the corrupted gate is
   the `WakeGate` field of **`_EX_PUSH_LOCK_WAIT_BLOCK`** — a **stack-resident**
   pushlock wait block, NOT a session-pool allocation. So (a) special pool can
   never catch it (it guards *pool*, not stack), which is why armed boots come
   back clean, and (b) it's a **pushlock-class** bug — cousin of the fixed §9.13
   WS-pushlock family — most likely closed by those fixes. 5 boots (3 unarmed +
   2 `SPECIALPOOL=*`) reach desktop clean. Re-open only on a live `BAD gate`
   recurrence; if it recurs, the tool is a **GDB watchpoint on the wait-block's
   WakeGate.Type byte**, not special pool.
2. **amd64 post-marker 0x50 (wild code pointer, §9.17) — not reproducing.**
   Clean armed + unarmed. Consistent with §9.17's own "likely-closed by the
   2026-07-05 clip-region `addr_of!`-on-pointer-field fix" call. Treat closed
   pending recurrence.
3. **arm64 0x1E in the TDI self-test — RARE (≈1/4) but a LIVE, localized lead.**
   The one that still bites. Forensics from the 2026-07-06 gate failure:
   - Crash PC symbolicates to `InterlockedExchangeAdd64+0x10` with
     `0xC0000005` = an **atomic on a wild pointer** (classic post-corruption
     death: a stale/clobbered pointer handed to an interlocked op).
   - Immediately preceding it, the arm64 IRQL self-check (hal-rust
     `halstubs.rs`) spams **`[IRQL] VIOLATION: raise-DOWN 15 -> 2`** paired with
     **`lower-UP 2 -> 15`**. These are ABSENT in clean boots (verified across 3)
     — they *correlate with the crash*, they are not benign detector noise.
   - Reading: current soft IRQL is stuck at **15 (HIGH_LEVEL)** when a spinlock
     acquire raises to SYNCH (which is `DISPATCH_LEVEL`=2 on arm64, `halstubs.rs:23`)
     → the "raise" actually DROPS 15→2, opening a preemption window mid-critical-
     section → state corruption → the wild-pointer interlocked death. The 15 is a
     **leaked HIGH_LEVEL** (a raise via the ISR path / `HalBeginSystemInterrupt`
     never lowered — same family as §9.19's leaked-DAIF-across-context-switch).
   - The violation `caller near` addresses land PAST ntoskrnl's last symbol
     (calibrate base off a known symbol: 0x1E PC = `InterlockedExchangeAdd64+0x10`
     ⇒ tsv base `runtime_pc − tsv_rva`), i.e. in a later-loaded module — the
     net/TDI path (netstack.sys / arm64 HAL interrupt path). `build-index.sh`
     only indexes ntoskrnl, so symbolicating the caller needs a netstack index.
   - **The IRQL-leak hypothesis above is DISPROVEN (2026-07-06, caught live).**
     The `last raise-to-HIGH @ <ra>` diagnostic (hal-rust `irql_violation`, added
     this session) fired on a fresh gate repro and pointed the leaked HIGH at
     **`KeBugCheckWithTf+0x824`** — i.e. the HIGH_LEVEL is raised BY BUGCHECK
     PROCESSING, not by pre-crash net/TDI code. So the `raise-DOWN 15→2` /
     `lower-UP 2→15` spam is an ARTIFACT of already being inside the bugcheck
     path (KeBugCheck raises to HIGH, then its own spinlock acquires look like
     raise-DOWN); it is NOT the cause. The instrumentation did its job: it killed
     a plausible-but-wrong hypothesis cleanly.
   - **The REAL fault:** the first (root) bugcheck PC symbolicates to
     **`KiUnwaitThread+0x28`** with `0xC0000005` — an access violation in the
     SCHEDULER thread-wake path, hit during the netstack TDI self-test's
     `associate`. (The second, nested 0x1E at `KeGetBugMessageText+0x60` is just
     a fault while printing the first bugcheck.) This is the documented
     **KiUnwaitThread BUGCHECK flake** (AGENTS.md kernel-debug recipe) — a
     scheduler wakeup operating on a corrupted `KWAIT_BLOCK`/thread pointer, NOT
     an IRQL leak and NOT a pool overrun. Reclassify: this is a dispatcher/wait
     corruption bug in the thread-wake path, rare (~1/4 arm64 boots), surfaced by
     the TDI self-test's cross-thread wakeup.
   - **Next diagnostic on a repro:** break on `KiUnwaitThread` (kernel-debug
     toolkit `run-until.sh --break KiUnwaitThread`), dump the `KWAIT_BLOCK` /
     `KTHREAD` it faults on, and `identify-object` the bad pointer — the amd64
     KiUnwaitThread flake recipe in AGENTS.md returns stop code + trapframe +
     object identity in one call. Calibrate the symbol base off the bugcheck P2
     (`ntoskrnl.exe+<off>`): base = runtime_PC − off; tsv_rva = module_rva + 0x400000.

### 9.28 Static analysis for leaked IRQL raises / spinlock acquires — `scan_irql_balance.py`

The §9.27 arm64 IRQL-leak hunt raised the right question: we're almost entirely
Rust, so a raise/lower pair is a *lifecycle* we should be able to check
statically instead of catching at runtime. Two durable answers, both landed:

**RAII is the real fix.** `ntkrnl_sys::guards::{IrqlGuard, SpinLockGuard}` make
a leak impossible by construction (Drop lowers/releases on every exit path,
including `?` and early `return`). Adoption is partial — most translated sites
still use the raw C pattern `KeAcquireSpinLock(&l, &mut old); …; KeReleaseSpinLock(&l, old)`.
Converting a raw pair to a guard is the permanent cure; the lint below is the
bridge until adoption is complete.

**The lint: `tools/scan_irql_balance.py`** (tree-sitter, wired into
`vibeos-cli check` as a 0-baseline gate). Model: per function, walk statements
tracking a HELD set — `KeRaiseIrql(CONST_LEVEL,&mut v)` / `KfRaiseIrql`-into-
local / `KeAcquireSpinLock(l,&mut v)` add `v`; `KeLowerIrql(v)` /
`KeReleaseSpinLock(l,v)` remove it. An early `return`/`?` with a non-empty held
set = a leak, UNLESS the return hands the var back (`return old;` — ownership
transfer). Only functions that lower *something* internally are checked (a
raise-only function is the cross-function acquire-API contract, not a leak).

FP-avoidance that matters (a noisy lint gets ignored — these took it 140→1):
- **Strip lvalue wrapper noise**: `core::ptr::addr_of_mut!(OldIrql)` /
  `&mut OldIrql as *mut KIRQL` must reduce to `{OldIrql}` — drop the path/macro
  tokens (`core ptr addr_of_mut addr_of mut raw`) and cut the `as <type>` cast.
- **Field stores are cross-call, not intraprocedural**: `(*FastMutex).OldIrql =
  KfRaiseIrql(APC_LEVEL)` stashes IRQL for a PAIRED release fn — only a *plain
  local* lvalue is checked (`ExTryToAcquireFastMutex` returning TRUE-holding was
  the FP that proved this).
- **Raise-to-a-VARIABLE is a restore, not an acquire**: `KeRaiseIrql(OldIrql,
  &mut OldIrql)` is the lower-then-restore pattern (kdb.rs) — `KeRaiseIrql`
  counts only when its level arg is a `*_LEVEL` constant.

**First catch (2026-07-06):** `usbport-rust usbport.rs` `USBPORT_MiniportCompleteTransfer`
leaked `TransferSpinLock` + IRQL on the empty-`SplitTransfersList` path — and it's
an **upstream ReactOS bug** (usbport.c:2059 `goto Exit` jumps PAST the
`KeReleaseSpinLock`), faithfully reproduced as a bare `return`. The
goto-cleanup-mistranslated-as-return shape is the exact thing the lint exists to
catch. Fixed by releasing before the early return (divergence from stock).

### 9.29 A foreign binary parked in a wait, or spinning on `b .`, is usually one of three shapes (Doom bring-up)

Running a *foreign* Win32 binary (the Doom-on-vibeos quest — full writeup +
build recipe in **`doom-on-vibeos.md`**) surfaced a reusable triage ladder for
"a usermode process launched but nothing renders." The CPU is in `KiIdleLoop`
(everything blocked) or pinned at one usermode PC (spinning). Walk it:

1. **Sample the guest PC a dozen times** (QMP `info registers`). *All* samples at
   `KiIdleLoop` = the process is **parked in a wait** (not crashed, not spinning).
   *All* samples at one fixed usermode PC = a **busy spin** — disassemble that PC.
   A *mix* of usermode + varied kernel PCs = the process is **running fine**
   (syscalls cycling) and the problem is elsewhere (display/focus).
2. **Parked in a wait, no file I/O** (break on `NtReadFile`/`NtDelayExecution` for
   ~20s → zero hits): the process is past its reads, stuck on a dispatcher object.
   For a fresh Win32 GUI process, suspect a **console-subsystem exe spawned by a
   console-less parent** — CRT startup blocks on csrss/consrv console allocation
   before `main` ever runs. Check `objdump -p exe | grep Subsystem` (3 = CUI =
   the trap; want 2 = GUI). Fix at the *build*: `-Wl,--subsystem,windows
   -Wl,-e,mainCRTStartup`.
3. **Spinning at a fixed usermode PC that disassembles to `b .`** (opcode
   `0x14000000`, branch-to-self): a **compiler-emitted infinite loop**. In a
   pointer-chase loop (`while(p) p = p->next;`) this is LLVM hoisting `p` out as
   loop-invariant and dropping the reload — the same aliasing/LICM miscompile
   class as the kernel `captures(none)` bugs, but here in a stock-`-O2`/`-Os`
   foreign build. `objdump -d exe | grep -E '14000000'` finds every self-branch;
   **`-O0` is the workaround** until the pass is identified.

To backtrace a *blocked usermode thread* offline (gdb can't resolve the Rust
kernel's `_EPROCESS` type by name): hardcode `PsActiveProcessHead`'s relocated
address, walk `ActiveProcessLinks` scanning each node for a NUL-terminated
`ImageFileName`, then `ThreadListHead`→`ThreadListEntry`→`KTHREAD` using the
compile-time offsets asserted in `ntkrnl-sys/src/offset_check_arm64.rs` (validate
by checking `KTHREAD.Process` equals the EPROCESS base — a mismatch means an
offset is stale). The `WaitReason` byte alone (Executive vs UserRequest vs
DelayExecution) often localizes the block without a full unwind.

**Bugs #1–2 this quest also landed:** the `arm64input` avail-ring
`panic_bounds_check` (safe-indexing a `[u16; 0]` FAM → silent `loop{}` at
DISPATCH → first-keypress machine wedge; see `doom-on-vibeos.md` §Bug 1) and the
loud-panic-handler port for `arm64input-cargo` (bare `loop{}` handlers hide every
driver panic — ~40 `*-cargo` mains still have them, sweep owed).

### 9.27 (FIXED 2026-07-07) x86 PS/2 input was COMPLETELY dead — four stacked root causes, found by the Doom playable-gate

**Symptom:** amd64/i386 QMP-injected keys/mouse never reached any application.
Undetected for a long time: the cli gate's smoke never injects input, and the
pytest input test (`test_post_shell_ready_input`) only asserts the *class
driver* marker — which was ALSO dead, but nobody ran it. Surfaced the first
time an adversarial gate demanded a real app respond (`tests/qemu/test_payload.py`, née test_doom.py,
the "start a new game of Doom with injected keys" probe).

Four independent root causes, each masking the next (fix order = discovery
order; every one is arch-generic even though found on amd64):

1. **acpi-rust invented `CmResourceType*` table** (`buspdo.rs:418`): rustconv
   emitted a sequential 0..11 guess (Interrupt=0, Port=2, …) instead of the
   WDK values (Port=1, Interrupt=2, … cmtypes.h:224). Every ACPI-built
   CM_PARTIAL_RESOURCE_DESCRIPTOR was mistyped; IRQ descriptors were
   `CmResourceTypeNull` so pnpres translation and i8042prt's resource parse
   never saw an interrupt → `IoConnectInterrupt` never called → both PS/2 ISRs
   dead. Polled detection kept working, which hid it (devices enumerate fine).
   *Oracle:* GDB `hbreak` on `IoConnectInterrupt` + log callers → i8042prt
   absent. *Lint blind spot:* `verify_constants`/`--enums` missed it — local
   uncited `pub const` tables aren't verified (owed follow-up).

2. **i8042prt-rust mouse reset state machine stored INVALID ENUM DISCRIMINANTS**
   (`mouse.rs`, `MOUSE_RESET_SUBSTATE`): the C stores `I8042ReservedMinimum+N`
   scratch states (1001..1040) in the enum-typed field via plain int
   assignment; the translation kept the Rust enum and `transmute`d bare
   1001/1021/1040 into it. Transmuting a value that is not a declared
   discriminant is **UB, and LLVM used it**: knowing the legal value set, it
   *deleted the 1001..1040 match arms as unreachable*. The wheel-detect knock
   (F3 C8 …) stalled at state 1001 on every boot; the timeout DPC just clears
   MOUSE_PRESENT (no retry) → mouse dead. **New translation-bug class:**
   `transmute::<u32, CEnum>(out_of_range)` compiles clean and silently deletes
   the very code that uses the value. Fix = declare every reachable reserved
   value as a real variant (`I8042Reserved1001 = 1001`, …). *Oracle:* bounded
   ISR byte/state serial trace — conversation healthy to state 1001, then the
   response write never appears.

3. **winlogon only loaded keyboard layouts on arm64**: `InitKeyboardLayouts()`
   was `#[cfg(target_arch = "aarch64")]` ("required for RIT→WM_KEYDOWN
   delivery") — but the customized boot path skips the normal profile-based
   layout load on EVERY arch. With `gspklBaseLayout == NULL`, win32k's
   `UserProcessKeyboardInput` drops **every key** at "NO keyboard layout" —
   after kbdclass and the RIT already saw it. All the failure paths were
   no-op `ERR!` macros; §fix made kbdlayout.rs's ERR! real (DbgPrint) and
   added bounded `[WIN32K] KEYDROP:` prints at each silent drop site.

4. **The input desktop can be stranded on "Winlogon" on x86 too** (§9.23's
   second half): `SwitchDesktop(ApplicationDesktop)` post-explorer was also
   aarch64-only; RunSetup's async thread can leave `gpdeskInputDesktop` on the
   Winlogon desktop → keys route to the stranded desktop's foreground. De-cfg'd.

Also fixed en route: the shared i8042 packet state machine (`i8042PacketIsr`)
is consulted by BOTH ISRs but never checks which port the in-flight packet
targets — a keyboard packet in flight eats mouse bytes (and vice versa). This
is a latent race in the upstream C too; the Rust now gates on `PacketPort`.

**Test-oracle lesson:** the first playable-gate probe used F1 (Doom's help
screen = static frame). doomgeneric's `convertToDoomKey` doesn't map VK_F1 —
the game legitimately ignored the key and the gate kept failing AFTER the OS
was fixed. When probing a foreign binary, drive inputs it demonstrably maps
(ESC/ENTER/arrows) toward a state change it can't fake (new game started,
idle first-person view ≈ static vs attract demo ≈ 50% frame delta).

**i386 rare Mm-gate flake (open, REFRAMED 2026-07-07 — NOT a landing blocker):**
the acpi CmResourceType fix (#1 above) is correct and load-bearing for amd64,
and it connects the i8042 IRQ on i386 *for the first time* (previously
Null-typed → never connected). This can timing-expose a latent i386 Mm gate
deadlock: winlogon parks in `NtFreeVirtualMemory` → (Mm internals) →
`KeWaitForGate` (WaitReason 0x23 WrPushLock, HOLDER-STUCK "no signal ever"),
boot never reaches the desktop. **Empirically it is rare:** on a clean build,
i386 smoked 5/5 green + a full 3-arch gate ×3 green (~13 consecutive clean
boots) with no recurrence — the one live capture was under the
timing-perturbing `--lockdep` instrumented build. So it is a **pre-existing Mm
guarded-mutex race** (§9.13 WS-pushlock family), NOT introduced by the input
work and NOT a hard regression; the acpi fix is correct and stays. All 5 fixes
+ the amd64 playable win LANDED on master via ×3-green gate 2026-07-07. Left
open as a tracked rare flake; if it recurs, NEXT TOOL: `vibeos-cli build --arch
i386 --lockdep` names the holder + ordering inversion on serial (it's a stack
`_EX_PUSH_LOCK_WAIT_BLOCK` WakeGate class — special pool can't catch it; use a
GDB watchpoint on the wait-block WakeGate.Type byte, cf. §9.24).
### 9.30 (RESOLVED 2026-07-07) i386 desktop icons missing — decl/def calling-convention mismatch is UB that LLVM resolves by SILENTLY DELETING the calling branch (no ud2, no link error)

**Symptom:** i386 boots to a fully-working desktop (taskbar, ticking clock,
wallpaper) but ZERO desktop icons; amd64/arm64 render all four. No error output
anywhere.

**Root cause (four layers deep — record the chain, each layer was a real
finding):**
1. `CDefView::FillList` works: 5 items enumerated AND inserted into the
   listview. The *view window* stays 0×0/invisible — `CDesktopBrowser::
   Initialize` never reaches `_Resize()`/`ShowWindow`.
2. The desktop thread is parked forever in `RtlpWaitForCriticalSection` ←
   `LoadAcceleratorsW` ← `CDefView::OnCreate`. The CS (`U32AccelCacheLock`)
   is **zeroed, never initialized** (`LockCount=0→1, DebugInfo=0, Owner=0`) —
   a zeroed CS deadlocks its very first `EnterCriticalSection` (free is
   `LockCount=-1`). Systemic: winlogon's copy was zeroed too; only explorer
   ever calls `LoadAccelerators`.
3. `InitializeCriticalSection(&U32AccelCacheLock)` lives in user32-rust's
   `ClientThreadSetupHelper` behind `MessageInit() && MenuInit()`. Disassembly
   of the BUILT i386 user32.dll showed the entire branch **compiled to
   `return TlsAlloc() != -1`** — MessageInit/MenuInit/CS-init/LoadAppInitDlls
   all gone. On amd64 the same function has all the calls.
4. Why: `MessageInit`/`MessageCleanup` are **defined `extern "fastcall"`** on
   x86 (message.rs) and `MenuInit`/`MenuCleanup`/`DeleteFrameBrushes`
   **defined `extern "C"`**, but dllmain.rs **declared them all in one
   `extern "system"` block**. Same LLVM symbol, mismatched calling convention
   ⇒ the call is UB ⇒ LLVM deletes the branch containing it. **The fold
   removes the very ud2 that `scan_ud2` would have caught**, and the linker
   never sees the (wrong) symbol reference — completely silent.
   Confirmation: once the decls were fixed, the branch materialized and the
   link *immediately* broke on `_wcslen@4` — another wrong-convention decl in
   the same block that had been linkable only because its caller was folded.

**Fixes:** per-arch-correct extern blocks in user32-rust dllmain.rs (fastcall
x86 / C elsewhere for Message*, plain C for Menu*/DeleteFrameBrushes/wcslen),
plus the same class in win32k-rust (`DIB_1BPP_HLine`, `GreGetTextExtentExW`,
`UserExtTextOutW` — paint-path functions!) and user32-rust
`UserGetWindowBorders`. i386 desktop icons render.

**Permanent detector:** `tools/scan_cc_mismatch.py` — same-crate
extern-decl vs `#[no_mangle]`-def convention drift under i386 semantics
("system"→stdcall vs "C"/fastcall), cfg-aware. Wired into `vibeos-cli check`
(0-NEW baseline gate; stock ~330 findings, mostly driver crates = the
burn-down queue in `tools/lint-baseline/scan_cc_mismatch.txt`). Suppress a
verified FP with `// nolint: cc-mismatch (reason)`.

**Debugging lessons:**
1. **"Code path missing on one arch only, zero errors" → suspect a UB fold
   before anything else.** Disassemble the built binary and compare arches:
   a branch present on amd64 but absent on i386 with identical source is
   LLVM deleting provably-UB code. `scan_ud2` cannot see it when the whole
   branch (including the trap) folds away.
2. A **zeroed CRITICAL_SECTION deadlocks its first Enter** and looks like a
   lock-owner hang; check `LockCount/DebugInfo/OwningThread` before hunting a
   phantom owner (`LockCount=1,Owner=0,DebugInfo=0` = never initialized, one
   waiter).
3. QMP + manual page-table walk (`human-monitor-command xp` + 2-level i386
   PDE/PTE from the target EPROCESS's `DirectoryTableBase`) reads **any
   process's user memory** without breakpoints — module lists from PEB→Ldr,
   user-stack ret-chains, CS fields. Beats conditional breakpoints (which are
   glacial via the gdbstub) for cross-process user-mode inspection.
4. The `[T0E]`-style bisect-by-serial-print remains the fastest way to walk a
   wedge up a call chain (shell32 → LoadAcceleratorsW in 4 boots), but only
   ONCE the wedge is on a printable path — pair it with thread/wait-object
   dumps (`dump-thread`, `identify-object`) to find *which* path to print.
### 9.31 waitdep — always-on dispatcher wait-block integrity tripwire (`ke/waitdep.rs`)

**Built 2026-07-06 to attack the §9.27 arm64 `KiUnwaitThread+0x28` 0x1E flake**
(a KWAIT_BLOCK / dispatcher-list corruption that surfaces as an innocent-victim
0xC0000005 in the unwait path). Special pool can't see it (wait blocks live on
thread stacks, not pool) and a GDB watchpoint needs a repro in hand — so
instead the two chokepoints the crash funnels through validate the structures
BEFORE mutating them:

- **`KiUnlinkThread` entry** (every unwait goes through it): walks the thread's
  `WaitBlockList` chain and proves, per block — kernel-VA + aligned, `Thread`
  back-pointer matches, `WaitListEntry` doubly-linked invariants hold
  (`Flink->Blink == entry && Blink->Flink == entry` — exactly what the
  imminent `RemoveEntryList` relies on), `NextWaitBlock` cycle closes within
  68 blocks. Plus the thread's own `WaitListEntry` when linked.
- **`KiWaitTest` entry**: walks the object's `Header.WaitListHead` proving each
  waiter entry's list invariants, `Object` back-pointer, and `Thread` KVA.

**On violation:** `[WAITDEP] VIOLATION <kind>` on serial + a bounded dump of
the whole block chain (every block's Thread/Object/Next/Flink/Blink/WaitKey/
WaitType), then `KeBugCheckEx(0x109 CRITICAL_STRUCTURE_CORRUPTION,
P1=culprit, P2=owner, P3=check-count, P4=kind)`. So the flake now faults **at
detection with the corrupt structure on serial** instead of minutes of victim
distance later. Soft anomaly (State != Waiting at unwait) is print-only,
rate-limited ×8.

**Why always-on (unlike lockdep):** the flake fires ~1/4 arm64 gate boots in
the TDI self-test — gating the tripwire behind a build flag would mean it's
never armed when the crime happens. Checks are pure loads over memory the
caller dereferences on the next line anyway; no false-positive surface (every
hard check is an invariant the existing code's correctness already requires).
`VIBEOS_WAITDEP_CHECKS` (exported, atomic) is the liveness signal for GDB.

**When it fires:** the serial dump identifies WHICH invariant broke and on
which block — `thread-backptr`/`object-backptr` point at a stale/overwritten
block (stack reuse after an abandoned wait?), `list-integrity` at a
cross-linked or double-removed entry, `chain-overrun` at a clobbered
`NextWaitBlock`. Symbolicate nothing — the culprit address in P1 + the chain
dump is the whole story; feed the block address to `identify-object` /
`dump-thread` for the owner.

**§9.27 soak result (2026-07-07):** 16/16 dedicated arm64 `--repeat` boots +
3 gate arm64 boots all clean with waitdep armed — zero `[WAITDEP]` lines (not
even soft State anomalies), zero 0x1E. At the documented ≈1/4 rate, P(16
clean) ≈ 1% — the flake rate has COLLAPSED, most plausibly a side-benefit of
the §9.13 WS-pushlock family fixes (the last observed hit, 2026-07-06, was on
a pre-§9.26-era tree). Status: tripwire stays always-on; organic fleet gate
traffic continues the soak for free. The next occurrence anywhere will
self-report `[WAITDEP] VIOLATION <kind>` + chain dump + BUGCHECK 0x109
(P4=kind) instead of a victim 0xC0000005 — do NOT hand-loop boots chasing it.

**Correction — live catch hours later (2026-07-07 gate smoke, arm64, post-§9.26 tree): the flake is NOT collapsed.** `[WAITDEP] VIOLATION
bad-thread-ptr: owner=ffffffffffffffff culprit=ffffffffffffffff` fired during
the TDI self-test (right after `associate 0x00000000`) → 0x109. The wait-block
Thread pointer was ALL-ONES (a -1 poison/uninitialized pattern, not a stray
heap pointer) — consistent with a wait block consumed after its stack frame
was reused/poisoned. Same boot then cascaded IRQL raise-DOWN spam inside the
bugcheck path (KeBugCheckWithTf raising to HIGH; harmless secondary) and a
nested 0x1E in KeGetBugMessageText. This is the §9.27 arm64 TDI flake wearing
its new detector — one hit in ~20 armed boots today, so rarer than the historic ≈1/4 but alive; the waitdep chain dump above is the datum for the next debugging round.

**Coverage extension (2026-07-07, landed with the ex crate-split):** the live
catch was `bad-thread-ptr` fired from `check_unwait` at `KiUnlinkThread` —
which proves the -1 Thread arrived via a path that BYPASSES `check_waittest`
(KiWaitTest validates its object list at entry, so a corrupt block there
fires object-side first). Three wake paths consume `WaitBlock->Thread`
straight off a dispatcher wait list with no validation, and all three now run
`check_waittest` before consuming:
- `KeSetEventBoostPriority` (eventobj.rs — the pushlock event path; the event
  can be a STACK-resident `EX_PUSH_LOCK_WAIT_BLOCK.WakeEvent`) → `KiUnlinkThread`.
- `KiActivateWaiterQueue` (queue.rs — KQUEUE wake; worker queues). NB queue
  waiters (KeRemoveQueue) never set `WaitBlock->Object` (C nor Rust — only
  WaitListEntry is inserted), so this site passes `require_obj_backptr=false`
  or every contended queue wake false-positives K_OBJ_BACKPTR.
- `KeSignalGateBoostPriority` (gate.rs — no KiUnlinkThread in its path at
  ALL, so a garbage thread there previously crashed with no waitdep report).

The next hit therefore fires with **owner = the EVENT/QUEUE/GATE address**,
which names the funnel and (for a stack-resident event) the owning frame —
instead of the anonymous `check_unwait(-1)`.

**Hypotheses killed on paper (2026-07-07, don't re-derive):** (a) the
`ExfUnblockPushLock` WAIT-flag handshake is airtight for untimed waits — the
walker signals only blocks whose fetch_and saw WAIT already CLEAR (committed
sleeper), the canceller cases all synchronize correctly; (b) the timeout-path
hole (no handshake left after sleep-commit → canceller can return while a
walker is mid-signal on its stack event) is REAL in principle but unreachable:
no caller in the tree passes a non-NULL Timeout to
`ExTimedWaitForUnblockPushLock` (only `ExWaitForUnblockPushLock(NULL)`).

**Unifying lead for the TDI-window flake family (2026-07-08).** A second face
surfaced: winlogon dies user-mode (BUGCHECK 0x4C/C000021A cascade) with a
KERNEL VA (`0xFFFF8000050D4CB0`, identical across hits, kernel-stack region)
as `BASE_STATIC_SERVER_DATA.WindowsDirectory.Buffer` — caught by the
`[UEXC-BT]` user FP-chain backtrace (ke/arm64/stubs.rs, second-chance kill):
`LdrpRunInitializeRoutines → kernel32!DllMain → _snwprintf("%wZ;...")`.
basesrv heap-copies those strings into the CSR shared section correctly, and
the section maps at the SAME VA in server and clients — so the leading
hypothesis is a **wrong physical page behind the client's section view**: an
arm64 Mm page-identity race (direct successor of the fixed §9.14–§9.16
PFN/section family) briefly hands a client view a page still owned by (or
recycled from) a kernel thread stack. The same double-owned-page mechanism
would ALSO produce the §9.31 dispatcher face (KWAIT_BLOCK `Thread=-1` — stack
page scribbled by its other owner). Discriminators live on master:
- kernel32 DllMain (arm64) validates the shared-section strings and dumps
  `[kernel32] BASE_STATIC_SERVER_DATA CORRUPT ...` (+ falls back, so the boot
  survives diagnosed): whole-struct garbage ⇒ wrong page; only Buffer fields
  poisoned with sane Lengths ⇒ targeted overwrite.
- waitdep chain dump + `check_waittest` at the three direct-consume funnels
  (KeSetEventBoostPriority / KiActivateWaiterQueue / KeSignalGateBoostPriority)
  names the object and block on the dispatcher face.
Do NOT hand-loop boots; organic gate traffic is the soak — the next hit
self-reports enough to pick the branch.

### 9.36 (ROOT-CAUSED + FIXED 2026-07-08) the arm64 TDI-window flake family — netstack self-tests abandoned pending IRPs holding STACK event+IOSB; late completion scribbled reused frames

**Both long-standing faces, one bug.** The netstack self-tests (`tditest.rs
run_tdi_to`, `settest.rs`, `icmptest.rs`) all did:
stack `KEVENT` + stack `IO_STATUS_BLOCK` → `IoBuildDeviceIoControlRequest`
(IRP holds them as `UserEvent`/`UserIosb`) → `IofCallDriver` →
`KeWaitForSingleObject` **with a timeout** → on timeout, just return.
The frame dies with the IRP pending. Whenever the op later completes,
`IopCompleteRequest` (a) writes the IOSB through the stale stack VA and
(b) `KeSetEvent`s the dead event — whose wake path both READS whatever now
lives there (a reused frame) and WRITES self-referential wait-list pointers,
i.e. **addresses inside the dead frame** (deterministic kernel-stack VAs).

- **0x1E face (§9.27/§9.31, historic ~1/4):** `KeSetEvent → KxUnwaitThread`
  walks the dead event's "wait list" → `Thread=-1` (reused-local content) →
  `KiUnwaitThread` AV / waitdep 0x109. Caught live by the `[WAITDEP-BT]`
  kernel FP-chain backtrace: `IopCompleteRequest → KeSetEvent →
  KxUnwaitThread → KiUnwaitThread`.
- **0x4C face (winlogon death):** the dead-frame writes land in whichever
  thread reuses the stack — e.g. basesrv mid-build of its `UNICODE_STRING`
  locals; a written-in kernel-stack VA lands in a `Buffer` field, gets
  heap-copied into the CSR shared section, and winlogon's kernel32 DllMain
  `_snwprintf("%wZ")` later faults reading it (user AV in ntdll so_wstring →
  0xC000021A → 0x4C). Caught live by `[UEXC-BT]` naming
  `kernel32!DllMain → _snwprintf` and the fault address landing in the
  kernel-stack region, page-shifting between builds.

**Fix (netstack):** shared `tditest::alloc_wait_block`/`tdi_wait_finish` —
event+IOSB live in NonPagedPool; timely completion frees; timeout
`IoCancelIrp`s (hint only — netstack pends have no cancel routine, the
NT-textbook cancel+infinite-wait would hang the worker) and **loudly leaks**
the block (bounded, self-test-only) so the late completion writes owned pool.
`icmptest` additionally leaks `out_buf` on timeout (buffered-I/O completion
copies SystemBuffer back into `Irp->UserBuffer`).

**Validation (final, 2026-07-08):** pre-fix 3 waitdep hits/10 boots;
post-fix ZERO waitdep hits across 40 cumulative boots — the 0x1E dispatcher
face is DEAD (P(clean|unfixed) < 1e-4). The winlogon 0x4C face persisted
3/20 and was REFRAMED by the survivors' forensics: one hit's bad Buffer was
0x73 (string DATA read as a pointer half) and check-at-fetch-time validation
kept passing while the print faulted — so that face is winlogon's kernel32
DllMain RACING basesrv's population of BASE_STATIC_SERVER_DATA (csrss
accepts the connect before the shared section is fully populated), not the
dead-frame writer. Mitigation landed: kernel32 validates the SNAPSHOT the
%wZ actually uses and retargets to fallback strings on corruption (loud
'snapshot CORRUPT (raced basesrv?)' print = the confirmation signal from
organic traffic). The init-ordering race itself remains open.

**The reusable rule (grep for it after any driver/self-test translation):**
any `KeWaitForSingleObject(&stack_event, .., timeout)` on an IRP you built
with stack `UserEvent`/`UserIosb` MUST NOT return on timeout while the IRP
is pending — cancel+wait-forever (if cancellable) or pool-allocate + leak.
Sweep: `grep -rn "KeWaitForSingleObject" --include='*.rs' | grep -i timeout`
cross-checked against `IoBuildDeviceIoControlRequest(&mut event` stack args.

**⚠️ Addendum — the fix REDUCED but did not fully eliminate (organic recurrences
after the "DEAD" call).** The `[WAITDEP] VIOLATION bad-thread-ptr` face with
ALL-ONES poison (`owner=0xffffffffffffffff culprit=0xffff…`, the §9.31 signature)
has recurred on organic arm64 gate traffic *after* the 2026-07-08 fix — at least
one confirmed hit **2026-07-09** (gate `20260709-001019`, BUGCHECK 0x109,
P2=`ntoskrnl.exe+0x451B0`; serial chain dump preserved at
`~/vibeos-flake-farm/waitdep-evidence/20260709-gate-hit/full.log:72`), and an
overnight-gate arm64 0x1E (`ntoskrnl.exe+0x1CD8C`, silent-wedge face) seen
2026-07-10 that retried green. So the netstack-TDI source was A source, not the
ONLY source, of the ALL-ONES waitdep face — a residual arm64 dispatcher/stack
corruption persists at a low organic rate (~2 hits / 2 days of gate traffic).
`ke/waitdep.rs` stays ALWAYS-ON precisely to keep capturing these. **Do NOT read
§9.36 as "0x1E fully closed"** — if a `[WAITDEP]` 0x109 appears in a gate log,
that IS a live capture: read the chain dump (which invariant, which block/funnel),
compare the culprit PC against the two known offsets above, and treat it as the
still-open residual, not a re-derivation from scratch. Tracked in the roadmap
Known-bugs registry. *(Rescued 2026-07-10 from the performance lane's memory file,
which recorded the 07-09 recurrence but never routed it here.)*

### 9.32 Static analysis for leaked Ob references — `scan_ob_refcount.py`

Sibling of §9.28's IRQL scanner (same tree-sitter held-set engine), pointed at
the next lifecycle class: `ObReferenceObject*` ↔ `ObDereferenceObject`
balance. A ref leak never crashes a boot — it's pool exhaustion hours in — so
NO gate boot can ever catch one; static analysis is the only detector.

**First sweep (2026-07-07): 46 raw → 12 after engine iteration → 3 real
upstream leaks, 0 Rust-translation leaks** (adversarially verified per-finding
against the ReactOS C oracle by parallel subagents). Fixed the 2 reachable
ones (upstream ReactOS bugs, same class as the §9.26 usbport find):
- `config-rust/src/cmapi.rs` deleted-KCB path leaked the `KeyBody` ref
  (C `cmapi.c` bare `return Loaded;` while every other exit derefs);
- `dbgk-rust/src/dbgkobj.rs` `DbgkpPostFakeThreadMessages` leaked the
  last-thread ref on `STATUS_UNSUCCESSFUL` (C returns without releasing
  `pLastThread`, which only transfers to the caller on success).
The third (`sysldr.rs` session-load) is dead code here (flags!=0 rejected
earlier) — baselined, not fixed.

**Engine rules beyond §9.28** (each earned by a real FP shape from triage):
status-guard exemption (a return under the acquire's own `if
!NT_SUCCESS(status)` / `if ObRef..(..) != 0` check isn't a leak — the ref is
conditional there); alias groups (`Process = TargetProcess` — releasing either
name releases the ref; a name can be root AND alias across sibling branches);
overwrite-kills-hold (`x = (*p).Field` after an acquire under `x` — the var no
longer names the object); escape-on-any-use (passed to a call / stored through
a field = possible transfer → drop, precision over recall); single-ident
out-params only (field-expression out-args escape at birth). Known residue:
branch-merge FPs (acquire in one arm, return in the sibling — linear walk
merges them); those are verified + baselined
(`tools/lint-baseline/scan_ob_refcount.txt`), and the gate blocks NEW findings
only. Suppress a verified FP with `// nolint: ob-refcount (reason)`.

**Verify-fanout recipe (reusable):** one subagent per finding-file with the
finding, the trace rules (returned/stored/passed/failure-path = not leaked),
and the C-oracle question "does upstream deref on the corresponding path?" —
verdicts REAL_RUST_BUG / UPSTREAM_BUG / FP with line evidence. 7 agents
resolved 12 findings in ~1 min wall-clock; 2 engine bugs (wrong out-param
index for `ObReferenceFileObjectForWrite`, missing `ObReferenceObjectSafe`
statusfulness) were found BY the triage, not by inspection.

### 9.33 (RESOLVED 2026-07-07) black desktop shortcut icons — dropped `!` in `DIB_1BPP_BitBltSrcCopy_From1BPP` inverted `IgnoreSrc`; every identity 1bpp→1bpp SRCCOPY zero-filled its destination

**Symptom:** only the two .lnk desktop icons (Command Prompt, Read Me) rendered
as black squares with a white block in the corner; non-shortcut icons fine.
All arches.

**Diagnosis chain (three chained subagent audits, each exonerating a layer):**
exticon.rs (PE icon extraction) — faithful; user32 cursoricon.rs
(CreateIconFromResourceEx reconstruction + all BMP/mask helpers) — faithful;
the discriminator was the SHORTCUT-OVERLAY composite (`SIC_OverlayShortcutImage`,
C++ shell32) whose `ImageList_GetIcon(ILD_TRANSPARENT)` builds the overlay's
1bpp mask via a **1bpp→1bpp SRCCOPY BitBlt** — the only consumer of that DIB
path on the desktop. In `win32k-rust/src/dib1bpp.rs` the translation wrote
`IgnoreSrc = XorBit ^ (xlate(1) != 0)` where C is `XorBit ^ !xlate(1)` — one
dropped `!` inverting all four cases: the identity xlate took the
fill-with-XorBit(=0) branch, so the blit "succeeded" returning an all-zero
mask → SRCAND made the icon fully opaque → MaskBlt degenerated to SRCCOPY of
the overlay glyph → black square + white block. Also latently broke drag
images and ImageList_Merge.

**Fixes in the same change:** the `IgnoreSrc` inversion; `ROP4_MASK` in
dibobj.rs was `0x00AA0000` (C inteng.h: `0xAACC`) — RLE `SetDIBitsToDevice`
masked path; ten `copy_nonoverlapping` in dib32bpp.rs where C used
`RtlMoveMemory` (same-surface scroll blits overlap → UB) → `core::ptr::copy`;
user32 cursoricon.rs hotspot branch compared `rt == 2` where `RT_CURSOR == 1`
(scaled cursors silently got center hotspots).

**Lesson:** a boolean built from TWO C idioms (`!!x` and `x ^ !y`) is a prime
translation-drop site — the `!` inside an XOR is nearly invisible in review.
When only ONE consumer class of a shared primitive misrenders, enumerate what
that class does that others don't (here: 1bpp DESTINATION blits) before
auditing the shared machinery.

### 9.34 (RESOLVED 2026-07-07) arm64 input never reached apps — DO_BUFFERED_IO translated as 0x10 (= DO_DIRECT_IO); every input packet memcpy'd to NULL

**Symptom:** arm64 keys/mouse never reached any application. Driver-side
delivery looked healthy (`[ARM64INPUT] KBD-DELIVERED sc=0x0001`), but win32k
dropped every key at `KEYDROP: no VK for sc=0x0` — the RIT's
`KEYBOARD_INPUT_DATA` read back **all-zero**. Mouse "worked" only apparently:
zeroed `MOUSE_INPUT_DATA` is a silent no-op packet, so the cursor never left
screen centre and clicks never happened (test screenshots always show the
cursor at centre — that WAS the tell).

**Root cause (invented-constant class, same family as §9.27's CmResourceType):**
the C said `KbdDevice->Flags |= DO_BUFFERED_IO` (0x04); the translation wrote
`0x00000010` — which is **DO_DIRECT_IO**. `NtReadFile` therefore never
allocated a SystemBuffer (IRP arrives Flags=0x900, SystemBuffer=NULL,
UserBuffer=NULL, MDL path), and the driver's completion
`RtlCopyMemory(Irp->AssociatedIrp.SystemBuffer, &Data, 12)` **wrote each
packet to virtual NULL** in whatever process context the completion ran —
silent kernel memory corruption on every input event, plus data loss.

**Debug chain that found it:** bounded paired traces — driver `RingPush`
(sc=0x0001 healthy) vs win32k RIT receive (make=0x0) → completion-site trace
printed `sysbuf=0000000000000000` + IRP Flags without IRP_BUFFERED_IO → read
`NtReadFile` in iomgr-rust: SystemBuffer only allocated when
`DeviceObject.Flags & DO_BUFFERED_IO (0x04)`.

**Fix:** `arm64input.rs` device creation 0x10→0x04 (both KeyboardClass0 and
PointerClass0) + defensive `IrpOutputBuffer()` helper (SystemBuffer else
UserBuffer, skip-if-NULL) at all four completion memcpy sites. Result: the
`--doom` input gate goes green on arm64 — click-to-focus + ESC/ENTER starts a
new game (PLAYABLE Doom on arm64).

**Lesson:** when a reader sees ALL-ZERO structs that the writer provably
filled, suspect the *transport* (I/O manager buffering mode), not the data
path — and check every hand-written `Flags |= 0xNN` against the WDK value.
`verify_constants` can't see bare numeric flags ORs; grep for
`Flags |= 0x` in translated drivers is a cheap audit (owed sweep).

**Residual (task #21, still open):** an intermittent arm64 renders-freeze
(doom window black/whole-screen frame delta 0.0000) survives this fix at a
reduced rate — under investigation; NULL-write corruption was one component
but not the whole story. The dib1bpp/sprite-path suspicions from earlier
rounds were audited faithful-to-C (subagent audit, 2026-07-07) and are NOT
the cause.
### 9.35 (RESOLVED 2026-07-07) ex crate-split "x86 hang at ksecdd" — a `[T; 0]` extern-array decl: Rust bounds-checks the DECLARED length, so indexing through the C unsized-extern idiom panics at len 0

**Symptom:** after extracting `ex` into its own rlib (crate-split Phase 1b), i386
AND amd64 wedge deterministically in late boot right after
`MmGetSystemRoutineAddress: 'RtlQueryModuleInformation' NOT FOUND` +
`[BLT] IopInitializeBuiltinDriver: ksecdd.sys`; arm64 boots to desktop. On a
current tree the "hang" is actually loud:
`RUST PANIC (ntoskrnl) at ex-rust/src/sysinfo.rs:1368: index out of bounds: the len is 0 but the index is 0`.

**Root cause:** `ntkrnl-sys/src/globals.rs` declared
`pub static mut KiProcessorBlock: [ddk::PKPRCB; 0]` — the C idiom for an
unsized extern array (`extern PKPRCB KiProcessorBlock[];`). In C that's fine
(indexing is unchecked pointer math). In Rust the DECLARED type is the truth
the compiler enforces: `KiProcessorBlock[i]` through a `[T; 0]` decl is a
guaranteed bounds-check panic on the first index, regardless of the real
definition's size. Pre-split, `ex/sysinfo.rs` resolved `KiProcessorBlock` to
the in-crate sized definition (`ke/krnlinit.rs`, `[PKPRCB; MAXIMUM_PROCESSORS]`)
so indexing was fine; post-split it glob-imports `ntkrnl_sys::globals::*` and
silently binds the len-0 decl instead. Same symbol, same link, different
compile-time length → runtime panic.

**Why the exact i386+amd64-hang/arm64-clean split:** the panicking sites are
the `QSI_*` handlers (7 `KiProcessorBlock[i]` sites in sysinfo.rs). ksecdd's
init can't resolve `RtlQueryModuleInformation` (ntoskrnl has NEVER exported it,
any arch — that serial line is normal, not the failure) and falls back to
`NtQuerySystemInformation`, which lands in the panicking handlers on x86.
arm64's boot path never issues those QSI classes. The arch split encodes
*which boot path queries QSI*, nothing architectural.

**Fix:** declare the REAL per-arch size —
`[ddk::PKPRCB; crate::constants::MAXIMUM_PROCESSORS as usize]` (same constant
the definition uses; 32 on i386, 64 on amd64/arm64). Sizing UP a previously
`[T; 0]` decl is always safe for existing users (`.as_ptr()` unchanged;
indexing was impossible before).

**The class + how to sweep:** any `extern "C" { static mut X: [T; 0]; }` that
a crate-split suddenly rebinds is a landmine that only fires when the indexer
and the sized definition end up in DIFFERENT crates. Sweep:
`grep -rn 'static mut \w*: \[.*; 0\]' sdk/lib/rust --include='*.rs' | grep -v bindgen`
then check each name for `Name[` indexing anywhere. As of 2026-07-07 the only
other `[T; 0]` externs (kd64 KdpMessageBuffer/KdpPathBuffer/KdComponentTable/
TraceDataBuffer, iomgr KiProcessorBlock) are `.as_ptr()`-only in live code —
kd64's own sized-decl pattern
(`[PKPRCB; ntkrnl_sys::constants::MAXIMUM_PROCESSORS as usize]`, kdapi.rs) is
the correct template.

**Bonus finding from the same session (build infra, not kernel):**
`canon-run.sh`'s per-worktree sccache server port was `cksum(root) % 100` —
two live worktrees (vibeos-arm64-ex, vibeos-screen-painting) collided on port
4318, so one lane's rustc compiles were served by a daemon whose mount
namespace resolved `/tmp/vibeos` to the OTHER worktree (ENOENT on our build
dir at best; on a cache miss it would compile the sibling's sources —
phantom-build family). Now `md5 % 20000`. If a build dies with sccache
"No such file or directory" on paths that plainly exist in YOUR tree, suspect
a port/namespace collision: `pgrep -a sccache` + recompute the port hash per
worktree root.

### 9.36 (RESOLVED 2026-07-07) WriteFile hangs forever on USB volume — extern-`static` WIDTH DRIFT (`MmThrottleTop` declared u64, defined u32) permanently tripped the Cc write throttle

**Symptom:** first cached user write to a FAT volume never returns. No BUGCHECK,
no error, no FS activity — the writing process just parks. Everything read-path
works. (Surfaced as "FlushFileBuffers doesn't persist" / 0-byte files on the
medium; the real hang was *WriteFile*, one call earlier — the flush and close
never ran.)

**Root cause:** `cc-rust/src/copy.rs` declared

```rust
extern "system" { static mut MmThrottleTop: PFN_NUMBER; }   // u64 on amd64/arm64
```

but the definition (C `mminit.c:407` and Rust `ARM3/mminit.rs`) is `ULONG` (u32).
The 8-byte read pulls 4 bytes of the *neighboring* variable into the high half,
so the throttle limit is garbage-huge; `MmAvailablePages > MmThrottleTop` is
permanently false, so `CcCanIWrite` always says "throttled" and (Wait=TRUE path)
blocks in its deferred-write loop forever. LiveCD boots never do a cached write,
so this was latent until the first real FS write test.

**Why it was hard:** every probe in the FS write path showed *nothing wrong* —
the write IRP never reached fastfat at all. The unblocking insight came from
splitting the user-mode test into per-syscall markers (`created` printed,
`wrote` didn't → the hang is INSIDE WriteFile, upstream of the FS).

**The class — extern-`static` width drift:** same family as the VBO/LBO function
-arg width bugs, but on DATA symbols, and `verify_ffi.py` explicitly skips
extern statics. The linker gives no protection: a PE-COFF data import has no
type or size. Over-wide reads produce garbage; over-wide *writes* corrupt the
neighbor. A workspace sweep (2026-07-07) found and fixed the live cases:

- `MmThrottleTop`/`MmThrottleBottom` u64-vs-u32 (this bug),
- `MmLargeStackSize` declared `SIZE_T` in `ARM3/helpers.rs` + read as usize in
  `procsup.rs` GUI-stack paths (u32 in mminit — garbage large-stack size),
- `NlsMbCodePageTag` declared `i32` in pcix (C: `BOOLEAN`, 1 byte — over-read),
- `InitSafeBootMode` declared `BOOLEAN` in pcix (C: `ULONG` — under-read,
  LE-benign but wrong).

**Detection:** a first-pass scanner (group all extern-block `static NAME: TYPE`
decls by symbol, flag width conflicts after alias normalization) surfaced ~171
naming conflicts of which the above were real width bugs. Owed: wire an
extern-static width checker into `verify_ffi.py` (task: extend const/ABI
verification — most conflicts are alias noise like `LUID` vs `_LUID`, so the
checker must normalize through bindgen type aliases and compare *widths*, not
names).

**Lesson:** when a *user-mode* operation hangs with zero kernel-side probe
output, binary-search the syscall boundary FIRST (markf around each API call
costs one rebuild of a test app) before instrumenting ever-deeper kernel paths.
And any `extern static` whose type was guessed from the name (`Mm*` "looks like
a PFN count") is a width-drift candidate — the C definition is the only truth.

### 9.37 (RESOLVED 2026-07-07) i386+amd64 PS/2 keyboard/mouse fully dead since ACPI went Rust — hallucinated `CmResourceType*`/`ACPI_*` constants in acpi-rust broke PnP resource translation

**Symptom:** keyboard and mouse produced zero effect on the x86 desktop
(no `[I8042] StartDevice`, no kbd IRQ ever fired). Boot stayed green — input
is not on the boot-marker path — so it went unnoticed from 2026-06-09 until
the start-menu smoke work poked a key.

**Chain:** acpi-rust `buspdo.rs` `Bus_PDO_QueryResources` builds the
`CM_RESOURCE_LIST` the PnP manager feeds to `IopTranslateDeviceResources`.
The translated file had invented values for the `CmResourceType*` enum
(`Interrupt=0, Memory=1, Port=2` — real values: `Port=1, Interrupt=2,
Memory=3, Dma=4`) and for `ACPI_LEVEL_SENSITIVE` / `ACPI_*_RANGE` /
`ACPI_TRANSFER_*`. Descriptor `Type` bytes were therefore garbage →
`HalGetInterruptVector`/translation failed 0xC0000001 → `IRP_MN_START_DEVICE`
never reached i8042prt → no `IoConnectInterrupt` → dead input, silently.

**Fix:** correct the three constant blocks against `acrestyp.h` +
`sdk/include/xdk/cmtypes.h`. `verify_constants.py` only checks constants that
exist in C headers under the same name — locally-invented consts in a driver
crate dodge it; extending coverage to driver crates is the durable ask.

**Lesson:** when a device is dead but boot is green, walk the PnP resource
pipeline FIRST (`StartDevice` marker → translate status → descriptor types),
and treat every LLM-translated `pub const` block as guilty until diffed
against the C header. Permanent delivery-chain markers now exist:
`[I8042] StartDevice type=… -> status (irq vec=…)`, `[I8042] first kbd IRQ
fired`, `[KBDCLASS] first key delivered` — a smoke log shows exactly where
the input chain breaks.

### 9.38 (RESOLVED 2026-07-07) first keystroke BUGCHECK 0x1E on i386 — i8042prt DPC called kbdclass `ClassService` as `extern "C"` (caller-clean) instead of NTAPI stdcall

After §9.37's fix the first injected key crashed 0x1E. `i8042KbdDpcRoutine`
transmuted `KeyboardData.ClassService` to `unsafe extern "C" fn(...)`; the
target (kbdclass `ClassCallback`) is NTAPI = stdcall on i386, so callee `ret
0x10` + caller `add esp,16` double-cleaned the stack → ESP skew → the DPC's
epilogue returned into garbage. mouse.rs already used `extern "system"` —
keyboard.rs was the one-off. Same class as §9.30: on amd64/arm64 both
spellings coincide, so it links and runs everywhere except i386.
`extern "system"` is ALWAYS the right spelling for NTAPI function pointers.

### 9.39 (RESOLVED 2026-07-07) winlogon 0xF4 on first FAILING user-mode callback — i386 `KeUserModeCallback` conflated the SEH status with the callback's return status and skipped the user-ESP restore

**Symptom:** re-enabling `InitKeyboardLayouts` on x86 (see below) killed
winlogon with BUGCHECK 0xF4 the moment the keyboard-layout load completed —
kernel side of `NtUserLoadKeyboardLayoutEx` finished clean, then the process
died before executing a single further user-mode instruction. Fault state:
user EIP = a stack address, ESP = the spent callback-dispatch frame base.

**Root cause:** in `ntoskrnl-rust/src/ke/i386/usercall.rs`
`KeUserModeCallback`, the Rust translation modeled C's `_SEH2_TRY` block as a
labeled block whose value was BOTH the probe/copy error status AND
`KiCallUserMode`'s return, then did `if seh_status < 0 { return }`. C only
early-returns on a real SEH exception; the callback status — which may
legitimately be a failure NTSTATUS — must flow through to the epilogue
`*UserEsp = OldStack` that pops the callback dispatch frame off the USER
stack. The conflation meant any callback returning a failure status left the
thread's trap-frame ESP pointing at the consumed `KiUserCallbackDispatcher`
frame; on syscall exit the user `ret` popped garbage → unhandled AV →
critical-process death. First-ever failing callback on i386 was
GETCHARSETINFO (`TranslateCharsetInfo` → STATUS_UNSUCCESSFUL) during layout
load. Every prior callback (WNDPROC etc.) returned success, which is why the
desktop worked for weeks with this landmine armed. amd64's translation kept
the two statuses separate and was correct.

**Fix:** the 'seh block yields `Result<NTSTATUS, NTSTATUS>` — `Err` = SEH
path (early return, user mode never entered), `Ok` = callback status (falls
through to the ESP restore).

**Debug recipe that found it (all serial, no GDB):** bracket the layers —
kernel syscall wrapper entry/exit, each `co_` callout, user-mode handler
entry/exit, `KeUserModeCallback` enter/exit with old/new stack values, plus a
one-line user-exception trace in `KiDispatchException` (code/addr/EIP/ESP for
PreviousMode==User) and a `[CRIT-TERM] ImageName` print in
`PspCatchCriticalBreak`. The kill shot was noticing the KUMC *exit* print
fired for a SUCCESS callback but not for the FAILING one.

**Unblocked by this fix:** winlogon's `InitKeyboardLayouts` is re-enabled on
x86 (winlogon-rust; the C's `#if 0` cited a Rust-MM SEC_IMAGE deadlock fixed
2026-07-05 in §9.13). Without it `gspklBaseLayout` stays null on x86 and
win32k drops every keystroke at `UserProcessKeyboardInput` ("NO keyboard
layout"). With §9.37+§9.38+§9.39 fixed, i386 opens the Start menu via
Ctrl+Esc — now a smoke gate (`test_start_menu_opens`, saves
`qemu/<arch>_startmenu.png`).

### 9.40 (RESOLVED 2026-07-07) amd64 BUGCHECK 0x1E on first start-menu open — win32k `DesktopHeapReAlloc` passed the PDESKTOP as the HEAP pointer (and never freed the old block)

**Symptom:** the first Ctrl+Esc on the amd64 desktop BUGCHECKed 0x1E
(C0000005) at `RtlAllocateHeap+0x18d`; stack showed `NtUserDeleteMenu` →
`IntRemoveMenuItem`. Masked for a month because the keyboard was dead
(§9.37) — nobody could open the menu. In the smoke harness it painted as a
solid-navy screen (the BSOD backdrop) after the keypress.

**Root cause:** win32k-rust `menu.rs` reimplements the `desktop.h` inline
helpers. `DesktopHeapAlloc/Free` correctly use `(*Desktop).pheapDesktop`, but
`DesktopHeapReAlloc` took `Desktop: *mut c_void` and passed it STRAIGHT to
`RtlAllocateHeap`/`RtlSizeHeap` as the heap handle — so the allocator walked
the DESKTOP object as HEAP metadata (`heap->FreeLists` at +0x368 = garbage) →
AV. It also dropped the C helper's `RtlFreeHeap(old)` (its comment claimed
"caller frees old"; no caller does) and the `PrevSize == Bytes` early return.
The faulting site (`cmpw %bx, -0x8(%rax)` on `FreeLists.Blink`) looked like
classic heap-metadata corruption; it was actually "wrong object as heap".

**Lesson:** when a Rust file reimplements C header inlines, diff EVERY helper
against the header — a family of three near-identical helpers with one
diverging (wrong level of indirection, dropped free) is a signature
translation failure mode. A struct-as-heap mixup produces the same crash
signature as real heap corruption; check the heap pointer's provenance
before hunting for a corruptor.

### 9.41 (ROOT-CAUSED + FIXED 2026-07-10) amd64 Ctrl+Esc kills explorer + blank start-menu labels — GDI TEB batch never flushed (hand-rolled `TEB.GdiBatchCount` offset read 0x74 instead of 0x1740)

**THE ROOT CAUSE (one bug, four symptoms):** amd64 `KiSystemCallHandler`
(`ntoskrnl-rust ke/amd64/traphandler.rs:181-197`) is supposed to flush the
gdi32 TEB batch on every win32k syscall entry (C parity:
`ntoskrnl/ke/amd64/traphandler.c:224` `if (NtCurrentTeb()->GdiBatchCount)
KeGdiFlushUserBatch()`). The translation read the count with a hand-rolled
offset — `*(teb as *const ULONG).add(0x77 / 4)` = TEB+0x74 — but amd64
`TEB.GdiBatchCount` is at **0x1740** (`sdk/include/ndk/peb_teb.h`). The read
returned 0, the flush NEVER fired, and since ReactOS `NtGdiFlush` is
`SynchronizeDriver`-only BY DESIGN (the syscall-entry callout is the real
flush; gdi32's `GdiAllocBatchCommand` depends on it), the TEB batch buffer
(`GdiTebBatch.Buffer`, 0x300..0x7d8) grew monotonically and overwrote, in
order:
1. `RealClientId` / `GdiCachedProcessHandle` / `GdiClientPID/TID` /
   `GdiThreadLocalInfo` (0x7d8..0x800) → GDI handle-table userdata lookups
   break → shell32 `FillToolbar` item reads fail → **Run…/Shut Down… and
   separators dropped from the start menu**;
2. `Win32ClientInfo` (CLIENTINFO) at 0x800+ → `pci.pDeskInfo` (TEB+0x820) and
   `ulClientDelta` stomped with batch records (the structured "garbage"
   `0x??5?000000510008` / `0x001000000000?a04` = GDIBATCHHDR+GDIBSOBJECT) →
   first user32 `DesktopPtrToUser` deref AVs → **explorer dies** (csrss reaps
   via `LPC_EXCEPTION → NtTerminateProcess(STATUS_ABANDONED=0x80)`).
Additionally every batched-and-never-executed op silently vanished:
`GdiBCTextOut`/`GdiBCExtTextOut` (menu label text + flat separators — toolbar
paints on a plain window DC, so `DrawTextW→ExtTextOutW` batches), PatBlt,
SelObj, DelObj/DelRgn → **icons-without-labels menu** (icons survive because
`ImageList_Draw`'s source-ROP BitBlt goes straight to `NtGdiBitBlt`).

**Fix:** typed read `(*(teb as *const ntkrnl_sys::ddk::_TEB)).GdiBatchCount`
(traphandler.rs) — the i386 (`ke/i386/traphdlr.rs:998`) and arm64
(`ke/arm64/trap.rs:235`) translations were already typed and correct, which is
why this was amd64-only. Verified: `test_start_menu_opens` passes on amd64
with the COMPLETE menu (labels + arrows + separators + Run/Shut Down) and
explorer alive.

**Detectors added:** `scan_magic_offsets` shape `TYPED_PTR_HEX_ARITH`
(hex-literal `.add/.offset/.sub` on ANY typed pointer — would have flagged the
original line); permanent `[KI-EXC]` second-chance print in amd64
`KiDispatchException` (BEFORE the exception-port forward — csrss usually
"handles" a user second-chance by killing the client, so without the print a
shell death is 4x `[EXIT-THR] 0x80` with no cause). Debug recipe that cracked
it: conditional GDB break `NtTerminateProcess if $rdx==0x80` → caller was
csrss (offset-free EPROCESS name scan, walk-procs style) → `[KI-EXC]` teb=
print → crash TEB ≠ main-thread TEB → hardware watchpoint on the RIGHT TEB's
`pci.pDeskInfo` (`watch -l` + auto-continue + WHOHIT context print) → caught
`gdi32!GdiAllocBatchCommand`/`DeleteRegion` writing batch records at
Buffer+0x520 → C-ground-truth audit found the dead flush gate.

Likely also closed by the same fix (verify before re-opening): the
"freshly-CreateProcess'd GUI app AV" (notepad exit 0x80,
`ui-render-completeness-investigation.md` follow-up #1) — same face, and any
GDI-batching x86 thread without callback traffic was living on borrowed time.

**Historical writeup of the original hunt (2026-07-07/08), kept for the
watchpoint/diagnostic recipes:**

### 9.41-history (superseded by the root cause above) amd64 first Ctrl+Esc corrupts explorer's CLIENTINFO → shell AV in user32!ValidateHandle

**Symptom:** on amd64, the first Ctrl+Esc after the desktop settles kills
explorer: `[KI-EXC] EXPLORER.EXE code=0xc0000005 rip=user32!ValidateHandle+0x142`,
then 4× `[EXIT-THR] EXPLORER.EXE exit_status=0x80`. In the smoke harness it
paints as a solid-navy frame (BSOD-less shell death). i386 opens the menu
cleanly; this is amd64-only.

**What's proven:**
- Keyboard DELIVERY works (`[KBDCLASS] first key delivered`,
  `test_post_shell_ready_input` passes on amd64) — not an input-chain bug.
- The AV is at `user32!ValidateHandle+0x142`, the instruction
  `cmpq (%rdx),%rax` where `rdx = pci->pDeskInfo` (from `DesktopPtrToUser`).
  `pci` = TEB+0x800; `pDeskInfo` at +0x20, `ulClientDelta` at +0x28.
- Instrumenting `IntSetThreadDesktop` (win32k desktop.rs) shows it writes the
  CORRECT user value: `uDeskInfo=0x4f0dd0, ulClientDelta=0xfffff97f...` — a
  valid low-canonical user pointer.
- By crash time those same two fields read back as GARBAGE:
  `pci.pDeskInfo=0x02d0000000510008, pci.ulClientDelta=0x0010000000003a04` —
  i.e. a 16-byte blob was stomped over CLIENTINFO+0x20..0x30 AFTER the correct
  set. No win32k code writes `pci->pDeskInfo` except `IntSetThreadDesktop`, so
  this is a STRAY/overrun write (or a wrong-delta client-cache write) landing
  on CLIENTINFO+0x20, not an intentional bad assignment.

**Was masked by:** §9.40 (DesktopHeapReAlloc) — that 0x1E fired earlier in the
same Ctrl+Esc sequence (during menu-item build), so explorer never reached the
point of USING the corrupted CLIENTINFO. Fixing §9.40 unmasked this.

**Next tool (if it recurs):** a GDB hardware watchpoint on `&pci->pDeskInfo`
(TEB+0x820) in explorer's context, armed right after the `IntSetThreadDesktop`
correct-set, to catch the corrupting store. Suspect the client key-state cache
update (dwKeyCache/afKeyState written through ulClientDelta) or a callback
writing back through a stale/miscomputed client address during hotkey→menu
dispatch.

**Status 2026-07-08:** after the §9.40 fix + the §9.39 i386 usercall fix and a
CLEAN rebuild of both x86 arches, the crash NO LONGER REPRODUCES — 4/4
consecutive `test_start_menu_opens` runs on amd64 open the start menu with
explorer alive (screenshots verified). The temp CLIENTINFO diagnostics were
also removed in the same rebuild, so the diag itself perturbing timing can't
be excluded; treat as "likely a downstream echo of §9.40 debris on the warm
build" but re-open on any recurrence (the `test_start_menu_opens` gate now
runs FOR REAL on amd64 — a recurrence shows up as a hard failure with serial
tail + symbolicated BUGCHECK; re-add the xfail with `arch.startswith("amd64")`,
NOT `arch == "amd64"` — the fixture arch is `"amd64-uefi"`).

**RESIDUAL (RESOLVED 2026-07-10 — unflushed `GdiBCTextOut`/`GdiBCExtTextOut`
batches, see the root cause above):** the amd64 start menu renders DEGRADED vs i386:
Programs/Documents/Settings items show their icons but NO label text, and the
Run…/Shut Down… items are absent entirely (i386 renders the full menu). 100%
deterministic across 4 boots, so it's a real content/paint bug in the amd64
shell path (string-resource load or menu-item measurement), not capture
jitter. The panel-lit smoke gate passes (frame + Search/Help labels render);
label-level verification is future work.

### 9.42 (FIXED 2026-07-06) Kernel image shipped ZERO unwind info for Rust code — `requires-uwtable: false` in the custom target jsons; any exception walk died at the first Rust frame

**Symptom:** BUGCHECK **0x7F P1=8 (double fault)** with `TrapFrame.Rip =
RtlVirtualUnwind+0x165` (amd64), immediately after a `[RAISE]
RtlRaiseStatus(...)` line, followed by a cascade of secondary bugchecks
(0x50, 0x1E) as the bugcheck path itself walks the corrupted state.

**Found by:** the DUMMY_PSEH-verdict PoC (`ntoskrnl/seh_poc.c`, lane
`dummy-pseh-question`) — the first-ever kernel-mode raise with a REAL
`.pdata`/`.xdata` handler present. The compile side (native clang
`__try/__except`, `-fms-extensions`) is fine; the raise dispatches into
`RtlDispatchException → RtlpUnwindInternal → RtlVirtualUnwind`
(rtl-except/src/amd64/unwind.rs) and dies there.

**Mechanism:** `+0x165` is the `call memcpy` copying a 0x4D0-byte CONTEXT
per unwind step. A double fault AT a `call` = the push hit the stack guard
= **kernel stack overflow** — i.e. the unwinder faulted on some frame, that
fault raised a new exception, dispatch recursed into the unwinder again,
each level burning >1.2KB of CONTEXT copies until the stack died.

**Why it matters beyond the PoC:** today's *unhandled* raises (every
`ExRaiseStatus` under DUMMY_PSEH — §9.24's C00000D8 hunt) traverse this
same walk, so a raise that should end in a clean 0x1E can instead surface
as this 0x7F cascade ("RtlVirtualUnwind fault amplification"). If you see
0x7F/8 at RtlVirtualUnwind right after a `[RAISE]`, the raise is the
trigger, not the bug — root-cause the raiser AND note the unwinder defect.

**Root cause (CONFIRMED):** the custom rustc kernel targets
(`sdk/lib/rust/{ntkrnlmp,ntoskrnl}-cargo/*-kernel.json`) set
`"requires-uwtable": false` — so with `panic="abort"` rustc emitted NO
Win64 unwind info for ANY Rust function. The whole image `.pdata` was 380
entries, all from the cc-rs C shims. Every walk hit the leaf-fallback
(`Rip = [Rsp]`) on the first (non-leaf) Rust frame → garbage ControlPc →
the epilogue-scan's instruction read faulted → recursive dispatch → stack
exhaustion → 0x7F at the CONTEXT memcpy. The translated unwinder itself was
CORRECT.

**Fix:** `"requires-uwtable": true` (the value every real Windows rustc
target uses — Win64 ABI mandates `.pdata` for non-leaf functions). amd64 +
arm64 kernel targets flipped; `.pdata` 380 → ~7,900 entries (~1–3% image).
Validated end-to-end by the seh_poc.c boot self-test: `[SEH-POC] ...
(PASS)` — a native `__try/__except` catches `ExRaiseStatus` in kernel mode
and the system boots green. ALSO fixes: unhandled-raise diagnostics (clean
0x1E instead of this 0x7F cascade) and every stack-walk consumer on Rust
frames. **Sweep owed:** the other per-module `*-kernel.json` targets
(ntdll-cargo, driver cargos) carry the same latent `false`. Verdict
context + evidence: `docs/engineering/dummy-pseh-verdict.md`.

### 9.43 (RESOLVED 2026-07-08) usermode winsock servers: SYNs refused / responses truncated at 8KiB multiples — no TDI listen backlog + sends completed short under backpressure

**Symptom.** A usermode server (httpd payload) listens fine (`[NETSTACK] TDI
LISTEN conn N on :8080`) and serves *serial* requests, but under concurrent
load (tcp_hammer, 16 threads): (a) most connects get RST ("connection
refused"/"reset by peer" host-side), and (b) large responses arrive truncated
at **exact multiples of 8192 bytes** with a byte-correct prefix.

**Root causes (two, independent, both invisible to single-connection tests):**
1. **No listen backlog.** smoltcp has no SYN queue — one listening socket
   absorbs exactly one connection — and AFD keeps a single `TDI_LISTEN` in
   flight, re-posting only after the *usermode* `accept()` round-trip. Every
   SYN landing in that window was RST'd. Real Windows hides this behind the
   transport's kernel SYN backlog, so faithful-to-C AFD was still wrong here.
2. **Send truncation under backpressure.** netstack's `Cmd::Send` completed
   IRPs immediately: short count when the 8KiB tx buffer filled, refused when
   `can_send()` was false. AFD/msafd treated the completion as "all sent";
   the app's remaining bytes died silently. 8192-multiple truncation = N full
   buffer drains before the lie.

**Fixes (netstack tditrans.rs):** per-port shadow-listener backlog
(`LISTEN_BACKLOG=16` sockets armed per listened port; established handshakes
parked in a ready queue; `TDI_LISTEN` IRPs satisfied from the queue instantly;
torn down with the last address object on the port) + pended sends (per-conn
`send_q`; the bridge drains as tx space frees and completes each IRP only when
its whole buffer is queued — TDI semantics; dead socket fails the queue with
`STATUS_CONNECTION_RESET`, fail-loud).

**Debug recipe that cracked it (reusable):** paired breadcrumbs
`[AFD] ListenSocket/ListenComplete` (afd-rust listen.rs, permanent) +
`[NETSTACK] TDI LISTEN …` localize a listen break to msafd/AFD/netstack in one
boot. Host-side, the tcp_hammer control probe (dial the *known-good* netstack
echo port before blaming the payload's listener) discriminates harness/hostfwd
plumbing from guest bugs — it caught that the hammer was dialing `127.0.0.1`
where the windows-host backend needs `vm.net_host` (all-refused red herring).

**Bring-up landmines the httpd payload also documented (build.sh guards
enforce both):** console-subsystem apps spawned from explorer's startup walk
hang in kernel32 PROCESS_ATTACH allocating a CSRSS console (open bug — GUI
subsystem sidesteps); UCRT llvm-mingw's default `api-ms-win-crt-*` imports
aren't shipped by reactos, killing the child in the loader with ZERO serial
evidence — link classic msvcrt via `-lmsvcrt-os -Wl,-e,mainCRTStartup
-Wl,--subsystem,windows` (the doom recipe; bisect with a `-nostartfiles`
raw-entry probe exe when in doubt).

### 9.44 (OPEN 2026-07-08) arm64 user-mode SEH never delivers first-chance — `RtlRaiseException` unimplemented for arm64

**Symptom:** a user process that raises a software exception (or would want a
user `__except` to catch a hardware fault) on arm64 is killed instead of
caught. Serial (from `ke/arm64/stubs.rs` instrumentation): a single
`[UEXC] Code=… 1st=0` immediately followed by `[UEXC-KILL] … Proc=<app>` and
`PspExitThread ExitStatus=<code>` — the exception enters `KiDispatchException`
already **second-chance** (`FirstChance=0`); there is no preceding `1st=1` /
`[UEXC-DISPATCH]`, so first-chance user delivery never happened.

**Found by:** `base/system/sehtest` (crash-value lane) — the user-mode SEH
boot self-test, spawned by winlogon. amd64 catches all three cases
(sw-raise / `__finally` / hw-AV) `[SEH-USER] … PASS`; arm64 kills the process.
This is the FIRST time arm64 user SEH was exercised (the arm64 `rtl-except`
unwinder was "complete translation, never runtime-referenced").

**Root cause:** `RtlRaiseException` (`@ stdcall -register RtlRaiseException`,
ntdll.spec) has an amd64/i386 asm capture-and-call stub (captures CONTEXT,
calls `NtRaiseException(rec, ctx, FirstChance=TRUE)`), but **no arm64
implementation** — `rtl-except`'s arm64 module only *references* it
(`arm64/except.rs:403,420` for nested-exception re-raise). So a user raise on
arm64 never enters the first-chance path with `TRUE`; whatever reaches the
kernel arrives second-chance and the process is terminated.

**Fix (owed):** an arm64 asm/`#[naked]` capture stub mirroring
`rtl-except/src/amd64/except_asm.S` — snapshot the integer+control registers
into a CONTEXT, set `ContextFlags`, call `NtRaiseException(rec, ctx, TRUE)`;
also verify the arm64 `RtlpExecuteHandlerForException/Unwind` trampolines
exist. The kernel-side arm64 delivery is already in place:
`KiDispatchExceptionToUser` builds `KUSER_EXCEPTION_STACK_ARM64` and redirects
to `KeUserExceptionDispatcher` (`ke/arm64/stubs.rs:1079+`), and
`dll/ntdll/dispatch/arm64/dispatch.S` now has the DUMMY_PSEH swallow branch
(added this lane, amd64/i386 parity). Only the ntdll-side raise entry is
missing. Non-fatal to boot (sehtest is fire-and-forget; arm64 desktop still
reaches shell ready). Design: `docs/engineering/archive/crash-value-design.md`.

### 9.43 (2026-07-09) Un-gating a `-version=0x600+` export in a `.spec` shifts ALL later ORDINALS → breaks by-ordinal importers (BUGCHECK 0x4C `0xc0000139`)

**Symptom.** After making base `kernel32.dll` export a few previously
version-gated Vista symbols (dropped `-version=0x600+` from ~7 lines to unblock a
foreign binary's imports), **i386 AND amd64 deterministically BUGCHECK
`0x0000004C (0xc0000139, …)`** — `STATUS_ENTRYPOINT_NOT_FOUND` raised as a fatal
hard error — early, at csrss `LoadServerDll 'winsrv'`. arm64 was unaffected. The
build is green; only boot breaks; a re-smoke reproduces 3/3 (not a flake).

**Why.** `spec2def` assigns export ordinals **sequentially over the entries it
actually EMITS** — `Name=Name@N @<ordinal>` in the generated `.def`, packed with
no gaps. Version-excluded lines (those filtered out by `DLL_EXPORT_VERSION`,
default **`0x502`**) reserve **no** ordinal slot. So un-gating a line *inserts* it
at its alphabetical position and **shifts every later export's ordinal by +1**
(cumulatively +N). Any module that binds that DLL **by ordinal** (no name) now
calls the wrong function → wrong-signature call / entrypoint past the table →
`0xc0000139`. arm64 survived only because nothing binds kernel32 by ordinal
there; some x86 boot DLL does.

**Confirm.** `git stash push -m … <the.spec>` → rebuild that one arch → smoke. If
green with the spec reverted, it's the ordinal shift, not your logic. The `.def`
proves it: `grep '@[0-9]' build-<arch>/…/<dll>.def` shows the packed sequential
ordinals; diff the ordinal of any post-insertion symbol vs master.

**Fix menu (pick by intent).**
- **Only one arch needs the new exports** (the FEX case: fexprobe is arm64-only) →
  gate the added lines `-arch=arm64` so the *other* arches' ordinal tables stay
  byte-identical to master. Cheapest, zero-risk for the untouched arches.
- **All arches need them** → pin **explicit ordinals** on the new lines
  (`<N> stdcall Name(...)` with N above the current max) so nothing after them
  shifts; or add them at the very end of the spec; or rebuild the by-ordinal
  importer against the new implib. Never plain-ungate a mid-file line on an arch
  that has by-ordinal importers.

**Rule of thumb.** Adding a *named* export is NOT free — in an auto-ordinal
(`@ …`) spec it renumbers everything downstream. Treat any ungate/insert in a
widely-imported core DLL's spec (`kernel32`, `ntdll`, `user32`, `gdi32`) as an
ABI change and gate/pin it. Additive-by-name ≠ additive-by-ordinal.

### 9.44 (2026-07-09) A DLL "loads fine" but crashes at PC=0xFFBADD11 — our loader POISON-SNAPS missing imports instead of failing the load

**Symptom.** `LoadLibrary` + `GetProcAddress` on a foreign DLL (here:
`libwow64fex.dll`, wow64 Phase B) succeed, so the import surface looks
complete — then the first *call* to an unresolved import dies with
`STATUS_ILLEGAL_INSTRUCTION`/PC-alignment fault at **PC=0x00000000FFBADD11**,
X16 also `0xffbadd11` (arm64: the crash is a `br x16` through the import
veneer; on x86 you'd see a call to the same poison).

**Root.** Our ntdll loader intentionally snaps by-name import failures
NON-FATALLY: the IAT slot gets `0xffbadd11` and the load proceeds
(`sdk/lib/rust/ntdll-rust/src/ldr/ldrpe.rs:849`, inherited from ReactOS
`dll/ntdll/ldr/ldrpe.c:1150`). Great for booting a system with stub gaps,
terrible for diagnosing a new vendor DLL: "loads + resolves" proves nothing
about individual imports.

**Diagnose (one command).** Diff the DLL's imports against the exporter:
`llvm-objdump -p <dll> | <extract ntdll-imported names>` vs
`llvm-objdump -p build-<arch>/dll/ntdll/ntdll.dll` export list, `comm -23`.
Phase B's gap was `_errno`, `atan2`, `tan` — CRT helpers Wine/Win10 ntdll
exports that our 0x502 baseline never had.

**Fix pattern.** Arch-gated spec **forwarders to msvcrt** cost zero code:
`@ cdecl -arch=arm64 _errno() msvcrt._errno` (etc.). §9.43 ordinal discipline
applies — arch-gating keeps other arches' ordinal tables byte-identical.

**Trap within the trap.** ntdll ships via the CARGO link (`add_cargo_module`),
and its ninja DEPENDS didn't include `*.spec` — the spec edit regenerated the
cmake-side implib def but ninja never re-invoked cargo, silently shipping a
stale ntdll.dll ("forwarders added, still crashes"). Fixed by adding the spec
globs to `_REACTOS_CARGO_KERN_SRCS` (sdk/cmake/gcc.cmake); verify with
`llvm-readobj --coff-exports build-arm64/dll/ntdll/ntdll.dll | grep ForwardedTo`
before booting.

### 9.45 (RESOLVED 2026-07-09) arm64 BUGCHECK 0x23 during first-logon profile creation — untranslated fastfat `FatOpenExistingDirectory` stub

**Symptom.** arm64 boots to `shell ready`, then bugchecks *after the marker*:
```
(\dll\win32\userenv\profile.c:1841) Error: 2   ← GetUserProfileDirectoryW: no ProfileList\<SID>
HvHiveWillShrink is unimplemented              ← red herring (benign working stub)
FatAllocateDiskSpace -> (VOID)                 ← real, working
FASTFAT_STUB_HIT: FatOpenExistingDirectory
*** BUGCHECK: 0x00000023 (0xDEAD0000, 0, 0, 0)
```
Intermittent on mainline (timing-sensitive) — origin/master booted 9/9 green,
but a netstack change that made boot-time networking livelier nudged the timing
and reproduced it **6/6**. The `0xDEAD0000` is the deliberate stub marker, not a
corruption poison.

**Root cause.** fastfat is fully Rust-cohab (all 34 C bodies replaced), but
`FatOpenExistingDirectory` was never translated — only FFI-declared, backed by a
`STUB_BUGCHECK()` in `rust_cohab_stub.c`. It's reached from `IRP_MJ_CREATE` →
`FatCommonCreate` whenever a path resolves to an **existing directory with no
cached DCB** (first-time open). First-logon profile creation (userenv → config
manager creating the user hive → opening the profile's containing directory on
the FAT32 ramdisk) is one such path. Only bites arm64 because arm64's system
volume IS fastfat (x86 boots from cdfs), so it was latent + uncovered.

**Fix.** Translated the legacy C body (`create.c:4136`) to Rust in
`fastfat-rust/src/create.rs` as `#[no_mangle] FatOpenExistingDirectory`,
mirroring the already-translated sibling `FatOpenExistingDcb` (same
`'try_body`/`abnormal_termination` DUMMY_PSEH shape, `Iosb.__bindgen_anon_1`
status field, oplock/share-access/count idioms). All callees were already real
Rust (`FatCreateDcb`/`FatCreateCcb`/`FatSetFileObject`/`FatCheckFileAccess`/
`FatGetNeedEaCount`), so no new stubs underneath. C stub removed from
`rust_cohab_stub.c` so the Rust symbol wins.

**Reusable lesson.** A `FASTFAT_STUB_HIT: <fn>` on serial is an *un-translated
fastfat function*, not a corruption — grep `rust_cohab_stub.c` for the name; the
real C body is still on disk (`create.c` et al, kept per the never-delete rule)
and a sibling in `fastfat-rust` shows the translation idiom. These are latent
landmines on arm64 (the only fastfat-backed boot volume); a timing change
elsewhere can expose one that was dormant for months.

### 9.47 (2026-07-10) WoW64 process bring-up: two loader-ordering traps + a csrss-death-on-connect blocker

Context: launching a real i386 PE as a WoW64 process on arm64 (wow64-plan.md
Phase C M2). Three findings, in the order they bite as you push a wow64 process
through process init:

**(1) hello32 hangs in kernel32 `ConDllInitialize`; exit code 0x103.** A CUI
(console-subsystem) i386 image makes kernel32's DllMain call `ConDllInitialize`
which allocates a console via csrss. For a winlogon-spawned process with no
console to inherit, that connect blocks forever. The tell: winlogon's
`WaitForSingleObject(hProc, 30000)` times out, then `GetExitCodeProcess` returns
**0x103 = STILL_ACTIVE** (not a crash code — the process is still alive, hung).
Locate with the built-in `K32_DLLMAIN_TRACE` (kernel32-rust dllmain.rs): it
prints `pre-ConDllInitialize` but never `post-`. **Fix:** build the guest GUI,
not CUI (`-Wl,--subsystem,windows`). The FEX GUI probes work precisely because
GUI skips console init. General rule: a `0x103` "exit code" from a spawned
process means it never exited — look for a hang, not a fault.

**(2) A dependency that imports kernel32 must not be `LdrLoadDll`'d from early
process init.** The vendored CPU dll (libwow64fex.dll) imports kernel32.
Loading it from a divert placed BEFORE `LdrpRunInitializeRoutines` runs
kernel32's DllMain re-entrantly, under the loader lock, before the process is
registered — and hangs. **Fix:** do the wow64 handoff AFTER
`LdrpRunInitializeRoutines` (ldrinit.rs), where kernel32 has already initialized
at the normal time. Detect the wow64 process early (ProcessWow64Information
query → flag) but defer the actual `LdrLoadDll(wow64.dll)` + `Wow64LdrpInitialize`
call to the late point. Mirrors Win10's LdrpLoadWow64 timing.

**(3) [OPEN] csrss dies (BUGCHECK 0xF4) when a wow64 process connects to
basesrv.** After kernel32 DllMain completes, the wow64 process's
`CsrClientConnectToServer` to basesrv (serial: `[CSRSRV] CliCon: ServerId=2
InfoSize=1664`) makes **csrss raise exception 0x6BA via `RtlRaiseException`
(ntdll+0x6cf8) inside a CSR API handler**. With DUMMY_PSEH the handler's
`_SEH2_TRY` is dead code, so the exception is unhandled and csrss — a critical
process — dies → `BUGCHECK 0xF4 (0x3, <csrss EPROCESS>, ...)`. Serial signature:
`[UEXC] Code=0x6BA ... Proc=csrss.exe` immediately before the 0xF4. Note the
0xF4 is often FOLLOWED by a secondary `0x1E (0xc0000005, KeGetBugMessageText)` —
a fault while *printing* the first bugcheck; always find the FIRST bugcheck.
This is a csrss/basesrv wow64-connect bug (likely basesrv reading a wow64 field
— `PebAddressWow64` is hard-0, kernel32-rust proc_create.rs:2120 TODO — or the
client PEB), amplified by the CSR API loop lacking a DUMMY_PSEH-safe guard.
Whole path is behind VIBEOS_FEX staging → mainline boot unaffected (verified: a
clean arm64 smoke with the FEX artifacts stripped from the ramdisk is green).

**(4) STALE-RAMDISK TRAP — a non-FEX gate can hit (3) as a false-red.**
`mkramdisk --refresh` (what `vibeos-cli build`/`gate` use) re-stages freshly-built
artifacts into the *existing* `ramdisk.img` — it ADDS, it does NOT prune. So once a
`VIBEOS_FEX=1` run has staged `system32\{fexprobe,wow64probe}.exe` +
`syswow64\{hello32.exe,ntdll.dll}`, those binaries persist across every subsequent
plain `gate`/`smoke`. Winlogon's probes (`TryStartFexProbe`/`TryStartWow64Probe`/
`TryStartWow64Hello`) are gated on *file existence*, so they now fire on a normal
boot, launch the WoW64 chain, and death-spiral into the (3) csrss `0xF4` — making
arm64 look regressed when the code is fine. Tells: `[WINLOGON] fexprobe.exe
created` / `hello32.exe (i386 guest) created` on a boot you didn't build with
`VIBEOS_FEX=1`; the `0xF4` P2 EPROCESS resolves to csrss. **Fix:** rebuild the
ramdisk FRESH — `env -u VIBEOS_FEX bash qemu/mkramdisk.sh` (no `--refresh` ⇒
`rm -f ramdisk.img` + repopulate). Confirm clean with
`mdir -i qemu/ramdisk.img ::/reactos/system32 | grep -iE 'fexprobe|wow64probe'`
(want no hits). Then arm64 smokes 3/3 green. Cost this ~1 gate cycle on 2026-07-10
chasing a bogus `ntoskrnl+0x1CE20` (the smoke tool's PC annotation used the wrong
module base — real file addr = `PC - reloc`; gdb `find-kbase` gives `reloc`, then
symbolicate against `symbols-<arch>.tsv` which is ImageBase-relative).

### 9.46 (RESOLVED 2026-07-10) WHPX x86 bring-up: AHCI boot CD dead end-to-end — SIX stacked never-executed-translation bugs + UEFI firmware gaps

**Symptom.** WHPX (`VIBEOS_QEMU_WINDOWS_ACCEL=whpx`) was shelved: i386 spent
~5min in OVMF loading the livecd over IDE PIO (every 2-byte read = a VM exit),
then the boot wedged; amd64 BUGCHECKed 0x9 at uniata DMA. The fix direction —
attach the CD via `ich9-ahci` so OVMF DMAs it (`tests/qemu/backend.py`
rewrite + `PCI#CC_0106 → uniata` CriticalDeviceDatabase entry in
`hivesys.inf`) — then died in the guest: `CdRomCount=0` → BUGCHECK 0x6B.
Every one of the six blockers underneath was in a **never-executed
code path** (all prior boots used PIIX compat IDE, edge IRQ14/15, polled
probes): five translation defects + two firmware-completion gaps. Chain, in
kill order:

1. **uniata-rust `id_init.rs` Intel-AHCI unconditional drop.** The translator
   folded C's `if (AccessRanges[5].RangeStart) break; /* keep AHCI */` into an
   unconditional `HwFlags &= ~UNIATA_AHCI`, misapplying the CORE-13346
   "RangeStart assumed 0" note (true only *before* `UniataAhciDetect`
   self-heals the range from raw PCI config). Every Intel AHCI controller fell
   back to legacy IDE probing — dead on AHCI-only parts.
2. **freeldr UEFI: no `"PCI"` MultifunctionAdapter node** (upstream literal
   `// TODO: DetectPciBios`). `IoQueryDeviceDescription(PCIBus)` found nothing
   → scsiport's legacy native-PCI init (the ONLY path: ReactOS scsiport's
   `IRP_MN_START_DEVICE` is a stub) returned `STATUS_DEVICE_DOES_NOT_EXIST`
   before ever calling `HwFindAdapter`. Fix: `UefiFindPciBios` — PCI config
   mechanism #1 port probe (no INT 1A under UEFI) feeding the already-translated
   `DetectPciBios` (`freeldr-rust/src/uefihw.rs`).
3. **hal-rust `pcibus.rs` read 32 config bytes, not `PCI_COMMON_HDR_LENGTH`
   (64).** `HalpAssignPCISlotResources` stopped at BAR3 — BAR4, BAR5 (the AHCI
   ABAR!) and InterruptPin/Line all read as zero, so the resource list came
   back EMPTY and scsiport skipped the adapter (`BusInterruptLevel==0`).
4. **OVMF doesn't write per-device InterruptLine on all builds** — SeaBIOS-
   equivalent fixup in freeldr: `line = PIRQRC[(pin-1 + slot-1) & 3]` for bus-0
   functions with unprogrammed lines.
5. **OVMF never programs the ELCR** — PIRQ-routed IRQs stayed EDGE-triggered on
   the 8259. A held PCI INTx line delivers exactly one edge, then silence: the
   INQUIRY-completion interrupt never arrived (`FdoScanAdapter` waits forever).
   Fix: mark all valid PIRQ router targets level in ELCR (0x4D0/0x4D1),
   unconditional over PIRQRC.
6. **THE KILLER — hal-rust `pic_pic.rs` extern-declared `KiI8259MaskTable` as
   `[USHORT; 32]`; the real table (irql.rs / C irql.c) is `ULONG[32]`.** Every
   indexed read used half the true stride: `KiI8259MaskTable[16]` returned the
   low word of entry 8 (= 0), so `_HalpDismissIrqLevel`'s pend path EOI'd the
   slave PIC but **never masked IRQ11 at its own IRQL** — the held line refired
   instantly, forever (CPU livelocked in the dismiss; `pic1: isr=08` stuck
   in-service). Edge interrupts survived for months on garbage-but-lucky
   half-stride masks; IRQ11 for the AHCI HBA was the first level-triggered PIC
   interrupt the OS ever took.

**Result:** i386 boots to full desktop in **41s TCG / 52s WHPX** with the AHCI
CD; amd64 WHPX boots green (old 0x9 blocker gone — AHCI bypasses the IDE BM-DMA
path). Net suite 13/15 = the pre-existing §9.43-adjacent inbound-TDI issue.

**Debugging recipe that cracked it** (reusable for "interrupt never arrives"):
serial markers at each decision layer (uniata detect chain → scsiport scan →
`connect intr: level/vec/mapped/dirql/mode/share/st` → `AhciBeginTx:
GHC/hIS/PxIE/PxIS/CI` at SRB issue) + an **exponential ISR-entry counter**
(prints at 1/1k/100k/10M — a raw per-entry print would BE the livelock), then
`VIBEOS_AHCI_CD=1` (backend.py + qemu/start.sh) to reproduce the exact device
config under TCG/KVM natively, where **QMP `info pic` / `info irq`** told the
whole story in two lines: ioapic all-masked (⇒ PIC mode), `elcr=00`
(⇒ nothing level), later `pic1 irr/isr=08` stuck (⇒ EOI'd but unmasked
refire), and `kvm-i8259: 11: 2` (⇒ exactly two edges ever delivered). GDB
`inspect.sh --command bt` + hand symbolication against hal.dll file VAs
(runtime 0x8020xxxx − base + ImageBase 0x10000) pinned the livelock PC inside
`_HalpDismissIrqLevel`.

**Reusable rules.**
- A translated `extern "C" { static TABLE: [T; N] }` is an unchecked ABI
  boundary — the item TYPE is part of the contract. Wrong element width ⇒
  silent half/double-stride indexing. Audit candidates:
  `grep -rn 'static.*\[USHORT\|\[UCHAR' sdk/lib/rust/*/src/ | grep extern` and
  cross-check each against its C definition. (Same family as the
  bindgen-vs-hand-offset drift class — §scan_magic_offsets.)
- UEFI boots get NO BIOS-era side effects: no PCI description-tree node, no
  per-device InterruptLine (build-dependent), no ELCR. Anything NT's legacy
  paths "just assume the BIOS did" belongs in freeldr's firmware-completion
  pass (`UefiFindPciBios` + `pci_fixup_interrupt_lines`).
- "Interrupt never arrives" triage order: device regs armed (IE/IS/global-IE at
  issue time, printed in-path) → controller asserting (QMP `info pic`: IRR) →
  router/trigger config (ELCR for PIC, RTE for IOAPIC) → delivered-but-stuck
  (ISR in-service bit; livelock vs deadlock via a moving PC in repeated `bt`).

### 9.48 (2026-07-10) arm64 cross-module table-SEH silently drops every `__except` reached through a callee whose call site sits at its epilog boundary

**Symptom.** A standalone user exe (`base/system/sehtest/sehtest.exe`) died at its
FIRST test — `RaiseException(0xE0DEAD02)` inside a `__try` went **second-chance →
`[UEXC-KILL] Proc=sehtest.exe`, `exit=0xe0dead02`** — while the in-ntdll SEH tripwire
(`seh_poc_user`, both probes) passed. Same status code, same dispatch path; the only
difference was *whose frames the unwinder walked*.

**Oracle.** A per-frame probe in `RtlDispatchException` (arm64) gated to the raise code
showed the frame walk **stuck**: frame#1 = `kernel32!RaiseException`, and
`RtlVirtualUnwind` returned the *same* PC and *same* EstablisherFrame every iteration —
200 identical spins → "unhandled" → second chance. `interpret_codes` was a complete
no-op for that one function (never touched SP or X30), so `Pc = X[30]` fell back to the
stale return-into-RaiseException address.

**Root cause.** `kernel32!RaiseException`'s `.xdata`: `FuncLen=0xd0, E=1,
EpilogStartIndex=2`, codes `e2 15 | 55 | d0 14 | 0c | e4` (add_fp, save_fplr, save_reg
x19, alloc, end). The fault-return PC was at offset **0xc0 = exactly the epilog's first
instruction** (a `bl RtlRaiseException` immediately followed by the frame teardown —
common when the call is the last real work in the function). The E==1 single-epilog
branch in `unwind_decode::virtual_unwind_xdata` computed how many epilog codes had
"already executed" as `into = (func_len - pc)/4` — counting from the **wrong end**. At
the epilog's first instruction that gave `into=4` (skip the whole epilog to land on the
trailing `nop`/`end`) instead of `into=0` (apply the full epilog). The correct delta is
`(pc - epi_start_off)/4`, which the E==0 scope branch a few lines below **already used**.

**Fix.** One line in `sdk/lib/rust/rtl-except/src/arm64/unwind_decode.rs`:
`into = (pc_off_from_start - epi_start_off) / 4` where `epi_start_off = func_len_bytes -
epi_bytes`. Verified: `sehtest` sw-raise KILL→**PASS**, hw-fault **PASS** (closes the
§9.44-family callee-saved residual too — the multi-frame `__except` now resumes with the
right nonvolatiles).

**Why it hid so long.** `seh_poc_user` only ever unwinds *ntdll* frames whose call sites
are followed by post-return work, so their fault PC never lands on an epilog boundary —
the exact case the buggy formula got wrong. It takes a cross-module chain through a thin
`bl`-then-teardown shim (like `RaiseException`) to trip it. **Lesson:** any "the unwind
loops on one frame / `RtlVirtualUnwind` returns the same PC" symptom is an epilog-index
miscalc — dump that function's `.xdata` (`E`, `EpilogStartIndex`, codes) and check the
`into` delta against `pc - epi_start`, not `func_end - pc`.

**Follow-on (RESOLVED same day) — the `__finally` half: EstablisherFrame must be x29,
not SP, for frame-pointer functions.** After the epilog fix, `sehtest` was 2/3:
`finally: ran=0 caught=1`. A gated probe in `__C_specific_handler` proved the finally
funclet *was* called (`-> calling FINALLY scope[0] handler=0x…1248 frame=0x12FE90`) —
it just wrote `ran_finally` into a dead stack word. The funclet (`?dtor$9`) does
`mov x29, x1; … str w9, [x29-4]`, and `TestFinally` stores `ran_finally` at `x29-4`
where its `x29 = post-prologue-SP + 0x10` (`add x29, sp, #0x10`). But
`virtual_unwind_xdata` set `*EstablisherFrame = (*Context).Sp` unconditionally, so we
passed SP (`0x12FE90`) where the funclet needed x29 (`0x12FEA0`) — a 0x10 miss, wrong
slot. `__except` was immune because it resumes via `RtlUnwindEx` → `RtlRestoreContext`
(full target context, real x29), not via the establisher argument — which is why only
`__finally`/filters-that-read-parent-locals broke. **Fix:** if the prolog establishes a
frame pointer (scan unwind codes for `set_fp` 0xE1 / `add_fp` 0xE2), report
`EstablisherFrame = x29` (captured before `interpret_codes` restores the caller's x29),
else SP — mirrors Wine's amd64 `RtlVirtualUnwind` (`frame_reg ? frame_reg_value : Rsp`).
Packed frames need no equivalent: a packed entry (Flag==2) carries no language handler,
so its establisher is never handed to a funclet. Result: `sehtest` **RESULT: PASS**
(sw-raise + finally + hw-fault all green, `exit=0x0`). **Lesson:** "the funclet ran but
the local didn't change" ⇒ EstablisherFrame/frame-base mismatch, not a missing call —
disassemble the funclet, see what base+offset it writes, and check it against the
function's own frame-pointer offset.

### 9.49 (ROOT-CAUSED + FIXED 2026-07-10) arm64 BLACK desktop — desktops only ever paint via the hidden→visible show transition; arm64's early input-desktop switch broke the assumption for the whole window tree

**Symptom:** arm64 boots green on serial markers but the screen is BLACK except
stray window fragments (a notepad scrollbar + status bar); frozen thereafter.
Regressed on master ~Jul 5-6 (good `fc794e7c29e`, bad `b643a8ffbf9`+). The
boot-marker smoke stayed green for days of black desktops — the marker is
blind to paint.

**Root cause:** ReactOS only ever paints a desktop through the
`SWP_SHOWWINDOW` hidden→visible transition inside `co_IntShowDesktop` (that
path queues `WM_ERASEBKGND` → `IntPaintDesktop` fill). The explicit
`bRedraw=1` redraw passes `RDW_UPDATENOW|RDW_ALLCHILDREN|RDW_INVALIDATE` —
NO `RDW_ERASE` — and `RDW_INVALIDATE` alone never sets
`WNDS_SENDERASEBACKGROUND`, so `IntBeginPaint` runs with `erase=0` and fills
nothing. On x86 the Default desktop's FIRST show happens at the real
winlogon→shell `SwitchDesktop` (hidden→visible → erase → blue). On arm64 the
customized winlogon boot path makes Default the input desktop EARLY, while
the old input desktop's window is still invisible → `bRedrawDesktop=0` show
(visible, unpainted); every later `bRedraw=1` show (display-init
`winsta.rs:769`, the §9.22 switch) finds it ALREADY visible → no transition,
no erase, black — and via `RDW_ALLCHILDREN` the whole child tree (explorer
tray, notepad frame/client) shared the fate.

**Fix:** `co_IntShowDesktop_impl` (win32k-rust `desktop.rs`) adds `RDW_ERASE`
to the explicit redraw — an explicit "redraw the desktop" must erase to mean
anything. One flag restored wallpaper, taskbar+clock, notepad, version text
on arm64; x86 unaffected (its erase came from the transition anyway).

**Diagnostic recipe (kept as permanent bounded boot forensics):** serial
probes `[DESKSHOW]` (co_IntShowDesktop args), `[DESKPAINT]`
(IntPaintDesktop fill inputs: clip box, brush, wallpaper), `[DESKBP]`
(IntBeginPaint erase-decision: erase flag + clip type), `[DESKTHREAD]` /
`[DESKWNDPROC]` (desktop thread pump + wndproc liveness). The hunt pattern:
healthy-arch trace (amd64) vs broken-arch trace (arm64), diff the
DESKSHOW/DESKBP sequences — the divergence (second desktop's show missing /
erase=0 with a perfect clip box) named the bug. A commit bisect was
deliberately ABANDONED: warm-build masking makes per-commit verdicts
unreliable and the ~40-commit window would have cost hours; two instrumented
boots found it instead.

**Exonerated en route (do not re-chase):** the §9.33 dib1bpp/dib32bpp GDI
fixes (reverted-file boot still black); the arm64 GDI-batch flush sites
(predate the good commit, C-parity verified); wrong `gpdeskInputDesktop`
(§9.22's fix is present and fires — serial shows the switch succeeding).

**Still open on arm64 (next layer, honest-red in the smoke suite):**
`test_start_menu_opens` on arm64 fails because Ctrl+Esc never opens the menu
— winlogon's arm64 path logs `InitKeyboardLayouts failed (non-fatal)`, so
VK translation / hotkey dispatch has no layout. Also `runCmd FAILED (2)` for
`kbswitch.exe` + `rundll32 iernonce.dll` — files missing from the arm64
ramdisk (staging gap, `4fadba3d760` era). The gate's dual condition
(panel-lit AND region-delta) exists because the arm64 notepad demo pre-lights
the sample region — lit alone false-passed once.

### 9.50 (RESOLVED 2026-07-10) arm64 "unaligned access faults = Device memory" is only HALF a diagnosis — dump the OPCODE before blaming Mm

**Symptom (FEX/WoW64 lane, but general):** guest/user code takes DFSC 0x21
alignment faults on unaligned accesses even though `SCTLR_EL1.A=0`, and a
static audit of the AttrIndx path (MmProtectToPteMask, MAIR) says the pages
should be Normal-WB. The tempting conclusion — "user pages are secretly
Device-mapped" — sent a whole debug arc into Mm (link-order duplicate-symbol
theories, MAIR slot audits) when the Mm was innocent.

**The 3-in-1 discriminator (now permanent in `ke/arm64/trap.rs`):** on DFSC
0x21 the data-abort handler prints one line —
`[ALIGN] FAR=… PC=… PTE=<raw self-map PTE> MAIR=… OPC=<u32 at PC>`.
That single line separates ALL the hypotheses at once: PTE AttrIndx bits[3:2]
answer "is it Device?", MAIR answers "is the slot programmed right?", and the
opcode answers "does this INSTRUCTION require alignment regardless?"
Verdict in one boot: PTE AttrIndx=0 (Normal-WB, Mm fine), `OPC=0x88DFFCA7` =
`LDAR` — an acquire load, which alignment-faults **even on Normal memory**.

**Root cause + fix:** FEX emits LDAR/STLR/LDAPR for x86-TSO emulation and
ships its own unaligned-atomic backpatcher
(`ArchHelpers::Arm64::HandleUnalignedAccess` via
`BTCpuResetToConsistentState`), but it keys on
`EXCEPTION_DATATYPE_MISALIGNMENT`. Our kernel delivered alignment faults as
ACCESS_VIOLATION, so FEX dispatched them into the guest as real AVs (which is
also what the earlier "guest heap corruption" faces were). Fix = deliver DFSC
0x21 as `STATUS_DATATYPE_MISALIGNMENT` (0x80000002) in
`KiSynchronousExceptionHandler` — correct Windows semantics independently of
FEX. After the fix, every LDAR/STLR backpatch round-trips (fault → VEH →
backpatch → NtContinue) and execution resumes.

**Reusable rules:** (1) On arm64, alignment faults have TWO instruction-class
causes that no memory-attribute audit can see: acquire/release
(LDAR/STLR/LDAPR) and exclusive (LDXR/STXR) ops require alignment on ANY
memory type. Read the opcode first. (2) An emulator/JIT lane "impossible"
fault face is often the EXCEPTION CODE CONTRACT, not the fault itself — the
handler stack downstream (FEX here) switches behavior on the precise NTSTATUS.
(3) Related kernel landmine (OPEN, wow64-lane blocker §9.47 successor): a
user-supplied misaligned pointer to a PHANDLE-probing syscall reaches
`ProbeForWrite`'s RAISE path, and under DUMMY_PSEH that bugchecks (0x1E)
instead of returning 0x80000002 — `[PROBE-MISALIGN-R/W]` in ex-rust
`exintrin.rs` now prints addr/len/align + caller LR just before the raise.

### 9.51 (ROOT-CAUSED + FIXED 2026-07-11) First -smp 2 boot: two latent bugs the second CPU flushed out — uniata's nIEN ATAPI path under QEMU, and hand-rolled KUSER_SHARED_DATA shadow structs with fantasy offsets

**Context:** SMP bring-up M2 (first amd64 AP start). `-smp 1` green; `-smp 2`
died deterministically at BUGCHECK 0x6B (PROCESS1_INITIALIZATION_FAILED,
c000003a) and, after the first fix, at 0x4C (0xC000021A from SMSS). Neither
was an SMP bug per se — both were pre-existing latent bugs that only a second
CPU could expose. Expect more of this class as M3+ proceeds.

**Bug A — 0x6B: uniata `CPU_num > 1` switches to an nIEN-gated ATAPI send
that QEMU's IDE model cannot deliver.**
`uniata-rust/src/id_ata_part3.rs` (DriverEntry): `if CPU_num > 1 {
g_opt_AtapiSendDisableIntr = TRUE }` — an SMP workaround for real controllers
whose INTRQ stays pending across a device-side mask (nIEN). QEMU (and the
Bochs family it derives from) checks nIEN only at raise time and NEVER
replays a suppressed IRQ. Net effect: the INQUIRY packet command to the boot
CD loses its completion → scsiport scan reports no CdRom → no
`\ArcName\...cdrom(0)` → `IopReassignSystemRoot` can't resolve →
`PsLocateSystemDll`'s ZwOpenFile(ntdll) = STATUS_OBJECT_PATH_NOT_FOUND → 0x6B.
**Fix:** gate the override on bare metal only (`CPU_num > 1 &&
g_opt_VirtualMachine <= VM_NONE`), Rust + C reference copy both.
**Trap for the debugger:** uniata AND scsiport are Rust translations —
editing `drivers/storage/ide/uniata/id_ata.cpp` or
`drivers/storage/port/scsiport/*.c` does nothing (the shipped .sys comes from
`uniata-rust`/`scsiport-rust`; the C DPRINT file:line strings are baked into
the Rust, so serial output *looks* like the C is running). An entire
first-round "fix" went into the dead C copy and tested as a no-op.

**Bug B — 0x4C: five+ hand-rolled `struct KUSER_SHARED_DATA` shadow structs
with wrong field offsets; the AP's KiInitializeCpu chopped NtSystemRoot.**
`ke/amd64/kiinit.rs` declared a local `KUSER_SHARED_DATA {
ProcessorFeatures: [u8;64], NXSupportPolicy: u32 }` — placing writes at
USD+0x00..0x43 (real: ProcessorFeatures +0x274, NXSupportPolicy +0x2D5).
The `NXSupportPolicy = ALWAYSON` store writes 1 to USD+0x40 — bytes 16..19 of
`NtSystemRoot`. On the BSP this runs (KiSystemStartup) BEFORE Phase 0 writes
"C:\reactos" into USD, so UP boots never noticed — for years. The AP runs
KiInitializeCpu AFTER the string exists → "C:\reactos" became "C:\reacto\0"
→ smss's `%SystemRoot%` expanded to `X:\react` → KnownDlls dir open
c000003a → SmpInit fails → 0xC000021A → 0x4C. Found with a GDB hardware
watchpoint on USD+0x40 (TCG backend, `VIBEOS_NO_KVM=1` — KVM breakpoints
don't fire on this host); it caught CPU#1 mid-store with the culprit PC.
The audit then found the same landmine in: `ke/i386/kiinit.rs`
(ProcessorFeatures at +0x2D4, writes over ActiveConsoleId..Cookie),
`ke/amd64/xstate.rs` (XSTATE_CONFIGURATION at +0x00, over the tick/time
fields), `ke/amd64/thrdini.rs` (XState at +0x2E8 — a THIRD inconsistent
location), `rtlk-rust/misc.rs` (SuiteMask/NtProductType at wrong offsets and
wrong widths), `ex-rust/time.rs` (TimeZoneId at +0x2C, over
ImageNumberLow/High). All masked by ordering/self-consistency on UP.
**Fix:** all swapped to the size-verified bindgen
`ntkrnl_sys::ddk::_KUSER_SHARED_DATA` (NXSupportPolicy is the
`__bindgen_anon_1.__bindgen_anon_1.set_NXSupportPolicy()` bitfield).
`ke/clock.rs`'s shadow was audited correct (padding matches) and kept.
**Reusable rules:** (1) NEVER hand-roll a shadow struct for a type bindgen
already has — a partial local mirror with padding is a magic-offset bug in
struct clothing (`scan_magic_offsets` can't see it; a `struct
KUSER_SHARED_DATA`-redeclaration grep lint is OWED). (2) "Works on UP" proves
nothing about USD writes — boot ordering hides wrong offsets; the AP replays
per-CPU init at a time when the data is live. (3) For any "kernel global
string/field corrupts between phase X and Y" mystery: TCG + GDB hardware
watchpoint on the exact byte is a one-boot root-cause (`qemu/start.sh amd64
--gdb-wait` + `VIBEOS_NO_KVM=1`, `hbreak` a phase marker, `watch` the
address, `continue`).

### 9.52 (ROOT-CAUSED + FIXED 2026-07-11) M3 SMP bring-up: four lessons from un-parking the AP into the scheduler (-smp 2 now schedules on both CPUs to desktop)

**Context:** after M2 (AP parked pre-scheduler), M3 released the AP into
KiIdleLoop with KeActiveProcessors set at idle entry. Four distinct walls,
each generalizable:

**1. Per-processor Phase-1 loops must run AFTER KeStartAllProcessors.**
First `ObCreateObject` scheduled onto CPU1 AV'd in
`ObpAllocateObjectCreateInfoBuffer`: `ObInit2()`'s per-CPU
`PRCB.PPLookasideList` loop iterates KeNumberProcessors — and our Rust
Phase1 had `KeStartAllProcessors` wired AFTER `ObInitSystem(1)` (M2 put it
next to the boot-time block; the C reference order is boot-time → SMP start
→ Ob/Ex/Io phase-1 inits, precisely so every per-CPU loop covers the APs).
Moved to the C order. Rule: any `for i in 0..KeNumberProcessors` in a
phase-1 init is an ordering contract with KeStartAllProcessors.

**2. trap.S interrupt stubs: the HANDLER owns the IRQL protocol.**
Filling the 0xE1 `int 3` placeholder naively (raise cr8 + EOI in asm, plain
handler) trips ExitTrap's checked-build assert (c0000420 at
`KiIpiInterrupt+0xef`): ExitTrap requires cr8 restored to its entry value.
The pattern is KiDpcInterruptHandler's: asm = EnterTrap/call/ExitTrap ONLY;
the Rust handler does KfRaiseIrql → KiSendEOI → work → KeLowerIrql(old).

**3. TLB-shootdown senders MUST poll-service, never just wait.**
First shootdown (pure IPI + ack barrier) deadlocked exactly like the
textbook: a sender that entered with IF=0 spun for the sender lock while
the holder waited forever for its ack (caught live: GDB on both vCPUs, one
spinning with eflags.IF=0). NT-shaped fix: the request lives in a shared
pending mask; the interrupt is only a wake-up; EVERY spin loop in the
protocol services the CPU's own pending bit (`KiTbFlushServiceSelf`), so
progress never depends on interrupt delivery.

**4. Lazy-idle TLB — skip the round-trip for hlt'ed CPUs (huge).**
The correct-but-naive barrier made every MDL/syspte unmap a cross-CPU
round-trip: -smp 2 boot went 1m54 → 6m53 (and randomly tripped the
harness's 120s serial-silence abort — which looks like a HANG but is a
CRAWL; check whether serial resumes before calling it a wedge. Also: "both
CPUs at hal.dll+0xB766" = `HalProcessorIdle+6` = they're just idle — not a
deadlock). Fix: `KiTbLazyIdleSet` — idle loop enters lazy state around the
hlt; senders skip lazy CPUs (mark `PendingLazy`, no IPI, no wait — an idle
CPU touches only stable kernel mappings); the CPU flushes on idle-exit AND
at DPC/IPI ISR entry (DPCs run inside the hlt wake, before idle-exit).
Escape race handled sender-side: after marking PendingLazy, re-read the
lazy set; reclaim the bit of any escapee and treat it as awake (IPI+ack).
Result: -smp 2 boots 1m41 — FASTER than the parked-AP M2 baseline.
Removing the local-primitive hooks entirely was tried and is NOT safe: the
Rust Mm callers rely on `KeFlushProcessTb`/`KeInvalidateTlbEntry` for
mappings other CPUs cache (0xF4/wild-RIP user corruption returns).

Files: ke/amd64/ipi.rs (protocol), ke/amd64/stubs.rs KiIdleLoop (lazy
enter/exit), ke/amd64/traphandler.rs KiDpcInterruptHandler (flush-if-lazy),
ntkrnl-sys inline/mm.rs (broadcast hooks), ex-rust init.rs (Phase-1 order),
ntoskrnl/ke/amd64/trap.S (0xE1 dispatch). Owed next (M3b+): scheduled-on-
CPU1 visibility counter, targeted single-VA remote flushes, the packet
mailbox (staged wave-2 A), i386/arm64 parity, net-suite flake at -smp 2.

### 9.53 (ROOT-CAUSED + FIXED 2026-07-11) arm64 FindFirstFile returns FILE_NOT_FOUND for every FAT directory — a cohab `SEH_TRY_EXCEPT` glue stub was a no-op, so fastfat directory enumeration silently returned empty with status 0

**Symptom.** arm64 boots to desktop, but Ctrl+Esc opens no start menu. Serial shows
`CStartMenu.cpp:520/627: Unexpected failure (hr)=80070002` and `CDrivesFolder.cpp:760:
… 80070002` — `ERROR_FILE_NOT_FOUND`. The Start Menu shell folder can't be resolved
even though the physical directory (`X:\Profiles\All Users\Start Menu`) exists in the
FAT ramdisk. x86 (i386/amd64) is immune. **Boot-marker smoke stays green** — this is
a paint/shell-namespace failure the marker is blind to; `test_start_menu_opens` is the eye.

**The trace that cracked it (each step ruled out the "obvious" culprit).**
1. `FatQueryDirectory` (fastfat dirctrl.rs) had a *real* renumbered-enum bug (positional
   case labels 1..6 instead of the ntifs.h `FILE_INFORMATION_CLASS` values — cousin of the
   §-or-pattern class). Fixed it. **Start menu still red.** A bounded `[FFPROBE]` in
   `FatQueryDirectory` showed it was *never called* — 0 directory-control IRPs the whole boot.
2. `[FFPROBE-CR]` in fastfat `create.rs` proved CREATE + LFN both work: `All Users found=1
   lfnlen=18`, `Start Menu found=1 rem=0`, `advapi32_vista.dll found=1 lfnlen=36`. So the
   directory *opens*; only enumeration is dead.
3. `[FFF]` in kernel32-rust `FindFirstFileExW` showed `FilePattern.Length` was non-zero and
   the code reached `NtQueryDirectoryFile` — which returned `status=0x0` (SUCCESS) — yet 0
   fastfat dirctl IRPs. A `[NQDF]` in iomgr-rust `NtQueryDirectoryFile` proved the IRP *was*
   dispatched to `\FileSystem\fastfat` with a non-null `MajorFunction[IRP_MJ_DIRECTORY_CONTROL]`
   and came back SUCCESS — but `FatCommonDirectoryControl` never ran.
4. `FatFsdDirectoryControl` runs its `__try` body through the glue wrapper `SEH_TRY_EXCEPT(ctx,
   dirctrl_try, filter, except)`. It is the **only** `SEH_TRY_EXCEPT` caller in all of
   fastfat-rust (every other dispatcher — create, cleanup, … — inlines its try body as a
   `loop`/`catch_unwind`). `drivers/filesystems/fastfat/rust_cohab_stub.c` defined it as
   `VOID SEH_TRY_EXCEPT(PVOID,PVOID,PVOID,PVOID){}` — a **no-op**, mistakenly grouped with the
   genuinely-dead `_SEH2_*` macros. So `FatFsdDirectoryControl` built `ctx{Status=STATUS_SUCCESS}`,
   called the no-op (never invoking `dirctrl_try` → no `IrpContext`, no `FatQueryDirectory`),
   and returned its *init* SUCCESS. The IRP came back "successful" but empty → `FindFirstFileW`
   → `ERROR_FILE_NOT_FOUND` → shell namespace parse fails → no start menu.

**Fix.** `SEH_TRY_EXCEPT` now actually calls `try_fn(ctx)`. The filter/except paths stay dead
under `USE_DUMMY_PSEH` (exceptions don't fire), but the **try body must run**. The callbacks are
Rust `extern "system"` == `NTAPI` on every arch, so the pointer is typed `VOID (NTAPI *)(PVOID)`.

**Why CREATE worked but enumeration didn't** is the whole tell: single-file opens go through
`create.rs`'s inline try body; directory *enumeration* is the one path that routes through the
no-op wrapper. If a cohab glue file stubs a helper "because SEH is dead," check whether the
helper is a **dead exception macro** or a **live control-flow wrapper** — the latter must keep
executing its body. Grep the stub file for anything that takes a function pointer and returns
void; those are almost never safe no-ops.

**Detection.** A `[NQDF]`-style probe (IRP dispatched to the FS, returns SUCCESS, but the FS's
own common-dispatch probe never fires) localizes "handler returns without doing work" in one boot.
The class-agnostic lesson: **status 0 + zero downstream IRPs = the dispatcher short-circuited**;
don't trust a SUCCESS return, trust a probe *inside* the work function.

(Separately fixed in the same lane: the `FatQueryDirectory` renumbered-enum bug from step 1 —
real and now correct, though it only matters *after* this stub fix lets the query run at all.)

**Follow-on (same lane, same bug class): `try_return` was ALSO a no-op stub.**
Once SEH_TRY_EXCEPT let `FatQueryDirectory` actually run, its inner enumeration loop
never terminated: explorer crashed in the taskbar `OnCreate`/`CreateStartMenu` and
restarted forever. Root cause: `try_return(X)` (rust_cohab_stub.c) was a second empty
stub, but in the C source it is the control-flow macro `{ Status = X; goto try_exit; }`.
Every early exit in `FatQueryDirectory` — the `ReturnSingleEntry` loop terminator, the
"next entry won't fit" buffer-full guard, the no-more-files exit — did nothing, so
`CurrentVbo` oscillated between two dirents while `NextEntry` grew past the 16 KB user
buffer (out-of-bounds writes → crash). A bounded `[FFQD]` loop tracer (log `CurrentVbo`
/`NextEntry` per iteration, capped at 400) showed the runaway instantly: `NextEntry`
climbing 42920→46280 with `CurrentVbo` cycling 32↔128. Fix: a C `goto try_exit` can't be
a Rust function call — inline each site. Outer-closure sites `return;` straight to the
SEH_FINALLY cleanup; the per-entry **inner**-closure sites can only `return` from that
nested closure, so they set a `do_return` flag checked right after the inner
`catch_unwind` to break the enclosing loop (one faithful `try_exit`). `NOTHING` sites
preserve Status; `Status = X` sites set it. Post-fix: enumeration terminates (~32 bounded
queries to build the shell namespace vs 400+ runaway), explorer stops crashing, and the
arm64 desktop paints fully (notepad chrome, no torn strips).

**The general rule this lane hammered home:** a cohab `rust_cohab_stub.c` that stubs a
helper "because SEH/PSEH is dead" must distinguish a **dead exception macro**
(`_SEH2_TRY`, `_SEH2_EXCEPT`) from a **live control-flow wrapper** (`SEH_TRY_EXCEPT`
runs the try body; `try_return` is a `goto`). The dead ones are safe no-ops; the live
ones are NOT. Grep the stub file for any empty function that takes a function pointer or
whose C original expands to control flow — those are latent no-op landmines.

**Third layer (ROOT-CAUSED + FIXED 2026-07-11, same day): the `CreateStartMenu` "hang"
was ANOTHER fastfat translation bug — a dropped `_SEH2_FINALLY`, not a shell/COM bug.**
The initial framing ("deeper shell/COM-layer bug") was wrong. `FatQueryDirectory`'s Rust
translation ended its finally with only the Ccb update + `FatCompleteRequest`; the C
finally ALSO does `FatReleaseFcb`, `FatUnpinBcb`, and `FatFreeStringBuffer`
unconditionally. Every directory query therefore leaked a **shared hold on the directory
Fcb's ERESOURCE** (plus a pinned Bcb + a pool buffer for grown long names). Leaked shared
holds are invisible — enumeration keeps working (shared+shared stacks) — until any thread
queues an **exclusive** acquire on that Fcb; ERESOURCE fairness then blocks all NEW
shared acquirers too, wedging every subsequent open that traverses the directory.
Symptom shape: `SHGetSpecialFolderLocation(CSIDL_PROGRAMS)` succeeded the FIRST time,
then the identical second call (inside `GetProgramsFolder`) deadlocked opening
`...\Start Menu\Programs` — a *repeat-call-only* hang is a lock/cache-poisoning tell.
Diagnosis: `[CSM]` step tracer in `RSHELL_CStartMenu_CreateInstance` +
`GetMergedFolder`/`BindToDesktop` (NB: explorer prefers **rshell.dll**'s copy of
shellmenu over shell32's — instrument/rebuild `rshell`, not just shell32, or the probes
never run). Bisected to "user SHGetSpecialFolderLocation" → silence in one boot. Fix:
restore the three cleanup calls in the Rust finally (dirctrl.rs). Post-fix the arm64
taskbar appears (Start button, window buttons, live tray clock) and
`test_start_menu_opens` PASSES on arm64 for the first time — the full classic start menu
renders (Programs/Documents/Settings/Search/Help/Run/Shut Down).

**The second general rule from this lane: audit translated SEH functions for DROPPED
FINALLY BLOCKS.** The translation stitcher can emit the try body and lose the
`_SEH2_FINALLY` entirely (FatPnpQueryRemove: try block → `return`, no finally at all —
leaked Vcb+global locks AND never completed the IRP; latent, fixed same day), or emit a
partial finally (FatQueryDirectory: kept the C's `if (!AbnormalTermination())` tail,
dropped the unconditional resource releases above it). Cheap detector that found both:
count `FatAcquire*(Fcb|Vcb)` vs `FatRelease*(Fcb|Vcb)` call sites per .rs file and diff
against the same count in the inert C original — any file where Rust's acquire/release
delta differs from C's is a candidate. In fastfat only dirctrl.rs and pnp.rs diverged;
create.rs's finally had already been restored by an earlier fix of this same class (its
comments document the deliberately-dead DUMMY_PSEH abnormal-termination branches — that's
the correct pattern to imitate).

### 9.54 (ROOT-CAUSED + FIXED 2026-07-11) Every arm64 BUGCHECK wore a mask — the crash reporter crashed (KeGetBugMessageText), and underneath sat a callback-ERET IRQ-clobber race from a dropped `cli` in the C→Rust translation

**Triage rule first (the reusable part):** in any arm64 crash log, count the
`[KEXC]`/`*** BUGCHECK` pairs before symbolicating. If there are TWO, the
LAST one is a nested crash of the bugcheck display path itself and the
harness's `top_symbol` points at the MASK, not the bug. The mask's
signature: `0x1E (0xffffffffc0000005, ...)` with `ESR=0x96000005`
(EL1 data-abort, translation fault) + kernel `Spsr=0x300003C5`, PC in
`KeGetBugMessageText`. Both the 2026-07-09 waitdep gate hit
(`+0x451B0`, real stop = 0x109) and the 2026-07-11 armed-cswitchdep hit
(`+0x1D638`, real stop = 0x1E/C000001D) wore exactly this mask — the
recorded arm64 "top symbols" in older evidence are systematically suspect.

**Mask bug:** `KeGetBugMessageText` (ke/bug.rs) validated the
MESSAGE_RESOURCE_DATA header page, then walked `MessageEntry +=
entry.Length` hop-by-hop through the resource section with NO per-hop
validation — on arm64 the walk exits the mapped range and the crash
reporter takes its own 0x1E. Fix: `KiBugMsgRangeValid` on the blocks
array, on EVERY hop, on the final entry span; `Length==0` bails; bounded
strlen capped to the validated entry. A crash reporter must be fault-free
by construction — degrade to "no message text", never to a nested crash.
Companion noise fix: the arm64 HAL IRQL tripwire fired 8× raise-DOWN/
lower-UP lines during bugcheck display (bugcheck parks IRQL at HIGH by
design, display path acquires DISPATCH locks — expected, but it reads like
the crash signal and misled this triage). `KeBugCheckWithTf` now calls
`HalVibeosQuiesceIrqlTripwire()` (hal.spec arm64 export) at entry.

**The real bug under the mask (arm64-only flake, ~1/12 armed boots):**
original stop was `0x1E/C000001D` at `KiUserCallbackDispatcher`'s FIRST
instruction with `Spsr=0x102085` — M=EL1h + IL(bit 20) + RES0 bit 13. No
legitimate SPSR looks like that: it's ERET-from-a-corrupt-frame, and the
IL bit means the CPU itself flagged the restored state illegal
(ESR EC=0x0E, Illegal Execution State). Geometry: `KiUserModeCallout`
(ke/arm64/usercall.rs) carves `CallbackTrapFrame = CalloutFrame - 0x160`
BELOW its own live SP, then memcpys + field-writes it with IRQs enabled.
An IRQ in that window pushes ITS 0x160-byte trap frame at `[SP-0x160, SP)`
— inside the carve-out. With a function frame F ≤ 0xC0, the frame's
`Spsr` slot (+0xA0) is inside the clobber zone while `Pc` (+0x158)
survives — which is EXACTLY the observed crash (correct PC, garbage SPSR;
the 0x102085 is a 32-bit fragment of whatever register the nested IRQ
frame saved at that address). Smoking gun: the amd64 twin
(ke/amd64/usercall.rs) does `cli` before the frame copy and the i386 twin
does too — the arm64 translation DROPPED the interrupt-mask discipline.
Fix: `msr daifset, #3` before the copy (after the MmGrowKernelStack
early-return so the failure path isn't left masked); the final `eret`
reloads PSTATE.DAIF from the frame's Spsr(=0), so the mask never leaks
into user context — same argument as KiTrapReturn's daifset (§ the
2026-07-04 fix in trap.S, which protected the sysreg writes but not this
earlier window).

**Reusable rules:** (1) a trap frame carved below a live SP must be built
with interrupts masked — interrupts push at the CURRENT stack pointer on
both x86 and arm64; (2) when porting x86 kernel code, `cli`/`sti` pairs
are load-bearing — grep the x86 twin for interrupt discipline before
declaring an arm64 translation complete; (3) any code that runs INSIDE the
bugcheck path (message lookup, callbacks, display) must validate every
dereference — a fault there replaces the report with a mask; (4) partial
SPSR/context corruption where PC survives but low-offset fields die is a
frame-overlap signature — compute which offsets fall inside `[SP-0x160,
SP)` of the code that built the frame.
### 9.55 (ROOT-CAUSED + FIXED 2026-07-11) M3b: the -smp 2 "TDI dgram upcall TIMED OUT" NET_FLAKE — an "IRQL raise = mutual exclusion" UP assumption in a driver

**Symptom.** After M3a (AP scheduling live), `-smp 2` boots to desktop ~2/3
of the time; ~1/3 the netstack self-test prints
`[NETSTACK] TDI dgram self-test: datagram upcall TIMED OUT` and the smoke run
classifies NET_FLAKE. Never reproduced on the default UP gates (they run the
identical self-test). The `-smp N` knob is the only variable → a real
concurrency bug that a single vCPU masked.

**The trap: "on QEMU's single vCPU, DISPATCH_LEVEL is full mutual exclusion."**
This premise is written verbatim in several netstack comments, and it is *true
on UP*: raising IRQL to DISPATCH masks thread preemption, so a producer thread
and a consumer thread on the one CPU can never overlap. **On SMP it means
nothing** — raising IRQL (CR8 on amd64) is a *per-CPU* register write; it does
not stop a thread running on another CPU at any IRQL. Any driver shared
structure guarded only by an IRQL raise is unsynchronized under `-smp ≥2`.

**Root cause (the datagram flake).** `netstack-rust`'s TDI transport uses an
`INBOX`: an MPSC command queue (many TDI-dispatch threads push `Cmd`s; the one
poll worker drains via `mem::swap`) that was guarded by only
`IrqlGuard::raise_to_dpc()` over a bare `Vec`. Under `-smp 2` a `Vec::push`
that reallocates (moves the backing buffer) races the consumer's `mem::swap`
reading the `Vec`'s (ptr,len,cap) — and two producer threads race each other —
a torn read / lost element. The lost `Cmd` is a `SEND_DATAGRAM` (query never
goes out) or a `SET_EVENT(receive-datagram)` (handler never registered) → the
DNS reply's upcall never fires → the 10 s wait times out. (The comment even
named the hazard — "an unsynchronized Vec race between two threads = UB/heap
corruption" — but trusted the single-vCPU premise to prevent it.)

**Fix.** Guard the queue with a **real `KSPIN_LOCK`** via `SpinLockGuard`
(`ntkrnl_sys::guards`) → `KeAcquireSpinLock`, which raises to DISPATCH *and*
spins on the lock word. Because `KeAcquireSpinLock` is the exported kernel
symbol, a driver imports it and it resolves **per boot**: the SMP kernel's
`KxAcquireSpinLock` really spins (`feature=smp`, an `InterlockedBitTestAndSet`
loop); the UP kernel's is a no-op compiler fence — byte-identical behavior to
the old raise-only path there, so no UP regression and no driver-side feature
flag needed. **Do NOT use `KeAcquireSpinLockForDpc`** here — it is a *different*
function still stubbed to a no-op in this tree (returns IRQL without
raising/acquiring); reaching for it gives zero synchronization.

**Second instance, same class, same commit.** The smoke NIC is `virtio-net-pci`
(vm.py), so `hal.rs`'s `virtio_drivers::Hal` bounce path is live. Its
`static mut BOUNCES` table (used for buffers crossing a page boundary) was
scanned/mutated lock-free on the same "single vCPU → map/unmap never overlap"
premise: two concurrent `share`s could claim the same empty slot (one bounce
leaked + a descriptor pointing at the wrong physaddr → DMA to stale memory), or
a `share`/`unshare` pair could tear an entry. Guarded with the same
`SpinLockGuard` (share/unshare run on the poll worker, never the ISR, so a
DISPATCH-level lock is correct). NOT proven to be a *second* cause of the
timeout, but a genuine data race on a live path — fixed as defense-in-depth.

**Reusable rule.** In any driver/component that a `-smp N` boot exercises, a
comment of the form "QEMU runs a single vCPU, so DISPATCH_LEVEL is mutual
exclusion" marks a **UP-only** guard that is unsound under SMP. Grep the crate
for `single vCPU` / `raise_to_dpc` / `static mut` / `UnsafeCell` + `unsafe impl
Sync` and audit each: an SPSC ring with real acquire/release head/tail fences
(netstack `ndisphy::RxRing`) or a single disjoint volatile MMIO read
(`device::ack_interrupt`) is genuinely SMP-safe and stays; anything guarding a
`Vec`/shared table with only an IRQL raise needs a real `KSPIN_LOCK`.

Files: netstack-rust/src/tditrans.rs (INBOX lock), netstack-rust/src/hal.rs
(BOUNCE_LOCK), netstack-rust/src/lib.rs (comment). Owed next (M3+): scheduled-
on-CPU1 visibility counter, targeted single-VA remote flushes, packet mailbox,
i386/arm64 parity.

### 9.56 (OPEN — symptom confirmed, cause re-opened 2026-07-12) The -smp 2 residual: a real double-queue tears a DispatcherReadyListHead (KxQueueReadyThread called on an already-Ready thread). NOTE: the earlier "SwapBusy wait-side unimplemented" root cause is RETRACTED — the amd64 run-side spin already exists in ctxswitch.S (a `.rs`-only grep missed the asm); which call site issues the second queue is not yet pinned

**Faces (all from one corpus, 2026-07-11, amd64 `-smp 2`, TCG):**
- `[NETSTACK] TDI self-test: CONNECT FAILED 0xc00000b5` (STATUS_IO_TIMEOUT) — appears
  even in boots that go on to reach desktop green. Sometimes preceded by
  `TDI op timed out - IRP still pending; leaking wait block` (§9.36 bounded-leak path).
- Creeping service wedge: Themes/SamSs/SVCHOST(DcomLaunch) connects stop completing,
  serial goes silent (HANG_TIMEOUT, no bugcheck). The §9.53-fixed dgram-upcall
  timeout face can also appear earlier in the same boot.
- `BUGCHECK 0x19 BAD_POOL_HEADER (8, 0, <line>, <entry>)` = POOL_HEADER_IS_ZERO —
  a pool block header found zeroed mid-page.
- `BUGCHECK 0x1E (c0000006 STATUS_IN_PAGE_ERROR)` with cdfs.sys frames — a page-in
  from the boot CD failed.

**What it is NOT (bisected + controlled, method worth reusing):**
1. Receiver-side full-flush vs targeted invlpg in the 0xE1 TLB shootdown: identical
   failure rate (4/6 both) → the M3-tail invlpg narrowing is exonerated.
2. **Control experiment:** `git stash` the whole M3-tail delta, rebuild the SMP kernel
   at M3b-landed HEAD (`7b23f82f467`), smoke ×6 under the same host conditions →
   1/6 BUGCHECK 0x1E c0000006. The flake pre-exists the M3-tail work entirely.
   (M3b's 12/12 green was a favorable streak — treat any small-N -smp pass rate
   as ±30% until the M4 soak characterizes it.)
3. `--tripwire "SPECIALPOOL=*"` over 6 boots caught nothing at a store (2 hangs, no
   special-pool fault) — the 0x19 is more likely a *racing free/completion* zeroing a
   header than a classic overrun.
4. Pool locking audited: amd64 `KiInitSpinLocks` IS called per-CPU
   (ke/amd64/krnlinit.rs boot-stack — easy to miss; grep the FILE, the i386/arm64
   twins live in kiinit.rs), LockQueue points at shared globals, nonpaged pool uses the
   real queued spinlock, paged pool uses the real interlocked+gate guarded mutex.

**Suspect class:** lost wakeup / device-I/O completion race under real concurrency
(uniata/scsiport IRP completion, DPC targeting, or a KeSetEvent→schedule hole) —
the same family as §9.51's uniata `CPU_num>1` find, not the TLB layer. The
IN_PAGE_ERROR face says a CD read IRP completed with failure (or never completed);
the TDI c00000b5 face says the same for net IRPs; the creeping wedge is the
pipe-connect variant. One I/O-completion-shaped hole explains all three.

**M4 soak data (2026-07-12, REFRAMES the rate):** 453-boot scoped farm soak
(amd64 `-smp 2`, SMP-kernel media, SPECIALPOOL=* armed, one QEMU at a time,
[SMP] markers verified in the boot logs): **3 failures — ALL inside the first
hour, while landing-gate builds + x3 smokes were hammering the host; ~400
consecutive green boots after the host went quiet** (29s median boot, vs the
same evening's contention-inflated ~110s). All 3 failures wear the identical
service-connect creep-wedge face. So the residual is **host-timing-skew
sensitive**: extreme TCG starvation stretches guest timing until a real race
window lands. The evening's ~25% rates were contention-amplified, quiet-host
`-smp 2` is ~0/450 — but the 0x19 POOL_HEADER_IS_ZERO and 0x1E c0000006
crashes remain REAL guest bugs (host slowness cannot zero a pool header; it
can only widen the window). Don't re-litigate the rate; the bug stands.

**Next tools (in order):** the repro amplifier is now known — run the farm
WITH deliberate host load (a parallel build loop or a second farm arch) to
reopen the window; build `--cswitchdep` so the illegal hand-off pins the
culprit PC at switch time; for a live wedge, GDB both vCPUs
(`qemu/start.sh amd64 --gdb` + `VIBEOS_QEMU_SMP=2`, `info threads` / bt both).
Tracked as the gating item between M3 and M4.

**UPDATE 2026-07-12 — cswitchdep is a repro AMPLIFIER, not a detector, for
this bug (and the amplifier knob is now host-independent).** Soaked a
`--smp-kernel --cswitchdep` build at -smp 2 on a QUIET host: **46/202 failures
(23%)** vs 3/453 (0.9%) for the same config without cswitchdep. Mechanism:
cswitchdep takes a GLOBAL spinlock on every context switch (census
`spin_acq=666563`), so under -smp 2 both CPUs serialize through it at every
switch → a huge, deterministic timing perturbation that widens the real race
window — the same effect host contention gave us by accident, now generated
inside the guest and dial-able. This retires "host conditions" as the strategy:
**cswitchdep-overhead is the reproducible amplifier.**

Two cautions, both load-bearing:
- **cswitchdep's own violation output is NOISE under -smp 2** — census shows
  `violations=15821 / switches=19339` (80%) and `held_now=-2650` (a NEGATIVE
  held-lock count). Its held-count/"expected==actual at switch" invariant uses
  a single global counter inc/dec'd from both CPUs with no per-CPU separation,
  so it mis-fires on most switches. cswitchdep needs SMP-correct (per-CPU)
  accounting before its violation stream means anything at -smp N — a
  cswitchdep-lane follow-up, NOT this bug. It amplifies; it does not pin.
- The crashes it amplifies are REAL kernel faults, not cswitchdep-internal
  (its state is all-atomics — memory-safe). Dominant amplified faces:
  **0x1E c0000006 STATUS_IN_PAGE_ERROR at `KxQueueReadyThread+118`** (a
  corrupt/stale KTHREAD pointer walked out of a PRCB ready queue, or a
  demand-page-in IRP that failed under the -smp 2 I/O race) + the silent
  service-connect wedge. Both point at the dispatcher-ready-queue / IO-
  completion boundary — consistent with the §9.51 uniata I/O-completion class,
  NOT the TLB/IPI layer.

**Next (highest-signal, lowest-token):** with a 23% repro rate the live catch
is now practical — boot the cswitchdep build under GDB on both vCPUs
(`qemu/start.sh amd64 --gdb` + `VIBEOS_QEMU_SMP=2`), break on `KeBugCheckEx`,
and on the 0x1E examine the KTHREAD pointer KxQueueReadyThread was dereferencing
(is it a freed/zeroed/wild thread block? → dispatcher ready-queue race; is it a
valid VA whose page-in faulted? → I/O-completion race). The KxQueueReadyThread
face is the most diagnosable; chase it first.

**UPDATE 2026-07-12 (b) — root-cause AREA found by disassembling the pinned
instruction (no GDB): the residual is a DISPATCHER READY-QUEUE LIST RACE, not
device I/O.** 13/14 amplified 0x1E crashes fault at the SAME instruction,
KxQueueReadyThread+0x76 (file VA 0x462906) — a classic InsertHeadList into
Prcb->DispatcherReadyListHead[priority]:

    movq (%r10), %r9        ; r9 = ListHead->Flink
    movq %r8,  0x8(%r9)     ; r9->Blink = Entry     <-- FAULTS (r9 stale/wild)
    movq %r8,  (%r10)       ; ListHead->Flink = Entry

It faults writing ListHead->Flink->Blink because ListHead->Flink already points
at freed/garbage — this PRCB's ready list was corrupted by concurrent,
unserialized manipulation. (IN_PAGE_ERROR not ACCESS_VIOLATION = a wild pointer
landing in a transition/paged VA; ready lists are nonpaged so a legit entry is
never pageable. The earlier cdfs IN_PAGE_ERROR was the same wild pointer in
CD-backed pages, NOT a failing I/O — retires update (a)'s device-I/O theory.)

**It is a LOCKING-DISCIPLINE hole, not a no-op-stub bug.** KiAcquirePrcbLock
under feature=smp is a real test-and-set spin on Prcb->PrcbLock
(ke.rs smp_simple_lock_acquire). The bug: some path touches a PRCB's ready list
without holding THAT PRCB's PrcbLock. Suspects (thrdschd.rs): KiQueueReadyThread
is a lockless #[no_mangle] export straight into KxQueueReadyThread (does every
caller hold the lock?); KiProcessDeferredReadyListInner drains
Prcb->DeferredReadyListHead with NO lock; and any cross-CPU path that locks the
CURRENT Prcb but inserts into a DIFFERENT target Prcb's list.

**Next (focused audit, subagent-sized):** enumerate every reader/writer of
DispatcherReadyListHead / DeferredReadyListHead in thrdschd.rs + wait.rs +
dpc.rs and prove each holds the TARGET Prcb->PrcbLock across the whole
Flink/Blink update (NT's KiInsertDeferredReadyList / KiQueueReadyThread lock
contract). The first ready-list manipulation under the wrong/no PRCB lock is
the -smp 2 residual.

**UPDATE 2026-07-12 (c) — static lock audit + a CORRECTION to (b).** (b)
symbolicated the fault against the UP kernel's table (ntoskrnl.exe) — WRONG
binary for an SMP boot. Re-symbolicated against ntkrnlmp.exe: the faulting RIP
(0x462906) is **KiSelectReadyThread+0x46 (the ready-list REMOVER), not
KxQueueReadyThread** (which sits at a different address in the SMP build and
DOES carry its lock xchg's — the 'dropped KiReleasePrcbLock' idea from (b) is
dead; UP KiReleasePrcbLock is a no-op, which is what made the UP disasm look
release-less). Real fault: RemoveEntryList inlined in KiSelectReadyThread,
faulting on a stale Entry->Flink. Caller chain (stack re-symbolicated vs
ntkrnlmp): KeWaitForSingleObject -> KxSingleThreadWait -> KiSwapThread ->
KiSelectReadyThread — the wait/reschedule path, holding the per-PRCB lock.

Full static lock audit of every DispatcherReadyListHead / ReadySummary reader
and writer (thrdschd.rs KiDeferredReadyThreadImpl/KiSwapThread/KiSetPriority/
KiUpdateEffectiveAffinity, ke.rs KxQueueReadyThread/KiSelectReadyThread,
balmgr.rs KiScanReadyQueues): **all hold the target Prcb->PrcbLock** (balmgr
holds BOTH the global dispatcher lock and the per-PRCB lock). The lock
primitive is correct (SeqCst xchg acquire + SeqCst release; x86 TSO). So the
corruption is NOT an obviously-missing lock — it is a runtime interleaving
(or a path not yet examined: dpc.rs:477, thrdobj.rs:952/1035 KiSelectNextThread,
the KiCheckDeferredReadyList drain) that static reading can't localize.

**Fault MECHANISM identified:** the ReadySummary-vs-list consistency invariants
(balmgr.rs:85/93/97: 'bit set => list non-empty', 'entry->Priority == index')
are `debug_assert!` — **compiled out of the release kernel**. So a
ReadySummary bit set while its list is empty (or holding a wrong-priority /
already-freed entry) is never caught; KiSelectReadyThread then computes
`thread = ListHead.Flink - offsetof(WaitListEntry)` from the list HEAD itself
and RemoveEntryList walks a bogus node -> the 0x1E.

**Next (cheapest, idiomatic — arm the invariant in RELEASE):** add a release-mode
guard at the top of KiSelectReadyThread: if the chosen ReadySummary bit's list
is empty, or Entry->Flink->Blink != Entry, KeBugCheckEx immediately with
(Prcb, priority, ReadySummary, ListHead.Flink) — converting the 14 downstream
IN_PAGE_ERRORs into ONE precise 'ready list inconsistent on entry' report that
says whether the list was ALREADY corrupt when KiSelectReadyThread ran (someone
else corrupted it — then GDB-watchpoint that CPU's ReadySummary/list) or is
being torn concurrently. Same amplifier build (--cswitchdep) makes it fire fast.

**RETRACTED ROOT CAUSE (2026-07-12) — "SwapBusy wait-side unimplemented" was WRONG.**
The earlier conclusion here claimed the SwapBusy run-side handshake was missing
(31 writes, 0 spin-waits) and the fix was to add a Rust spin before committing
NextThread to Running. **That grep only covered `.rs` files and missed the
assembly.** The run-side spin already exists and is correct on amd64:
`ntoskrnl/ke/amd64/ctxswitch.S:179-184` (`KiSwapContextInternal`) spins
`cmp byte ptr [r8 + ThSwapBusy], 0 / pause` on the incoming thread, with
`ThSwapBusy = 0x95` (=149, matching `offset_check_amd64.rs`), and that file IS
compiled (`ntos.cmake:391`, not cohab-flipped — only `thrdini.c` is, because
Rust took over `KiSwapContextResume`). arm64 has the same spin
(`ctxswitch.S:161`, offset `0x95`). The C reference agrees the handshake has
exactly two halves and no more: `grep SwapBusy ntoskrnl/ke/*.c` shows only
`KiSetThreadSwapBusy` (set) + `SwapBusy=FALSE` in `thrdini.c`/`thrdobj.c`
(clear); **ReactOS has NO readier-side SwapBusy wait anywhere** — the only
consumer is the arch `ctxswitch.S` run-side spin. So adding a Rust run-side
spin would be dead/duplicate code and fix nothing. Lesson (reusable): when
grepping for a "missing" mechanism in this tree, `.rs` is not the whole kernel —
the arch context-switch primitives are still hand-written **assembly**
(`ke/{i386,amd64,arm64}/ctxswitch.S`), and offset symbols like `ThSwapBusy` come
from generated `ksamd64.inc` (genincdata), not the Rust offset_of.

**STILL OPEN — the confirmed symptom, mechanism NOT yet pinned.** The tripwire
chain is still valid up to the observation, only the *cause* was misattributed:
(1) release-mode ready-list invariant at KiSelectReadyThread (0x96) -> torn
Blink on a live list; (2) per-writer post-checks (0x11/0x22/0x33/0x44) ->
tearing writer = KxQueueReadyThread (0x11); (3) entry checks split already-torn
(0x51) vs double-queue (0x52) -> **0x52 fires: KxQueueReadyThread is called on a
thread already State=Ready.** That double-queue is real and re-inserts a
still-linked `WaitListEntry`, tearing `DispatcherReadyListHead[pri]` -> the 0x1E
IN_PAGE_ERROR / silent wedge. cswitchdep amplifies it 0.9%->~25% because its
global per-switch spinlock widens whatever the racing window is. What is NOT yet
known: **which of the ~9 KxQueueReadyThread call sites** (mostly reached via
`KiReadyThread`->`KiInsertDeferredReadyList`->deferred-ready drain, plus the
direct preempt paths in thrdschd/wait/dpc/traphandler) issues the *second* queue
on an already-Ready thread, and whether the bug is a genuine double-ready or a
deferred-ready-list / PRCB-lock drain gap. Candidate class: a thread sitting on
one CPU's `DeferredReadyListHead` while ALSO Ready on a PRCB list, or a
reprioritization path (`KiSetPriorityThread`/`KiReadyThread`) that re-queues
without first removing under the right lock.

**2026-07-12 ELIMINATION ROUND — the tear is NOT produced by the ready-list API.**
Four amplified bursts (`--smp-kernel --cswitchdep`, 16 boots each) with successively
sharper tripwires. THREE precise, state-independent guards ALL went SILENT:
- **0x52** (double-queue: `KxQueueReadyThread` on a `State==Ready` thread) — silent.
  It DID fire once early, but later bursts tore the list with 0x52 quiet =>
  **`State` is the WRONG PREDICATE** (a path can change State without unlinking).
- **0x53** (`KiInsertDeferredReadyList` on a still-LINKED thread) — silent =>
  **the WaitListEntry/SwapListEntry UNION-ALIAS theory is REFUTED.** (Good theory:
  `SwapListEntry.Next` really does alias `WaitListEntry.Flink`, and a deferred write
  would tear a list with NO lock on the victim PRCB — it just isn't happening.)
- **0x54** (`KxQueueReadyThread` entry: node ALREADY LINKED — the precise
  "do my neighbours point back at me?" test) — silent => **no still-linked node is
  ever re-inserted.**
Yet **site 0x11/0x22 keep firing**: `ki_rq_integrity` finds the list broken
*immediately after a legitimate insert, with the correct PRCB lock held*.
Lock discipline was audited and is SOUND: `KiAcquirePrcbLock` is a real SeqCst
test-and-set on `(*prcb).PrcbLock`; every mutator (`KxQueueReadyThread`,
`KiDeferredReadyThread`, `KiSelectReadyThread`, `KiSetPriorityThread`,
`KiSetAffinityThread`, balmgr) pairs with the correct PRCB's lock; `KiQuantumEnd`'s
lock hand-off into `KxQueueReadyThread` (release-inside-callee) is correct NT
semantics; all `KiSelectNextThread`/`KiSelectReadyThread` callers hold the lock.

**=> REFRAME: the ready-list tear is very likely a VICTIM, not the bug.** A wild
write / use-after-free scribbling kernel memory would corrupt whatever is nearby and
produce exactly this: a locked, API-correct list observed broken. **Corroborating: the
same corpus carries `0x19 BAD_POOL_HEADER` (x2) and `0xF4` (x4)** — independent
memory-corruption faces. Chasing scheduler *logic* further is likely wasted; the
scheduler keeps coming up clean under every precise test.

**NEXT TOOL (stop adding tripwires):** catch the corrupting STORE, not its victims.
A GDB **hardware watchpoint** on the `Prcb->DispatcherReadyListHead[pri]` that gets
torn (pri 8/9 observed) reports the culprit PC directly — use the kernel-debug
toolkit (`qemu/start.sh amd64 --gdb` + `run-until.sh`), which needs the native/KVM
backend. Note `SPECIALPOOL=*` has already come back empty here — expected, since it
guards POOL and the PRCB is not pool; that null result is consistent with the wild
writer living outside pool allocations (or writing through a stale/freed pointer).

**2026-07-12 OVERNIGHT — the invariant that's broken is now NAMED.** A linkage-based
guard (`0x54`, "is this node genuinely doubly-linked right now?") fired from
**`traphandler.rs:106` — `KiDpcInterruptHandler`'s preempt branch** — with param3
(state) showing **`0x2` = Running** (and one `0x1`=Ready). I.e. **`Prcb->CurrentThread`
is Running yet SIMULTANEOUSLY linked in a ready list**, and the dispatch path queues
it (`KxQueueReadyThread(OldThread=CurrentThread)`) → tears the list. Crucially this
path is a **NT-faithful exact copy** (`KiDpcInterruptHandler` == NT
`KiDispatchInterrupt`, `ntoskrnl/ke/i386/thrdini.c:460` — same structure, NO OldThread
linkage check), so the tear here is still a VICTIM; the corruption is **UPSTREAM** —
something leaves a Running/CurrentThread thread linked in a `DispatcherReadyListHead`.
That is a *specific, traceable* invariant violation (much sharper than "list torn").
Also this burst: **`0x19` absent** (weak — low base rate; the Codex pool-checker
hypothesis needs a farm to confirm/deny), **`[WAITDEP]` silent** (§9.36 not implicated
on amd64 SMP here).

**Now hunting: who leaves CurrentThread linked?** Candidates — a cross-CPU
reprioritization/affinity path (`KiSetPriorityThread`/`KiSetAffinityThread`/balmgr)
that manipulates a thread by its `NextProcessor` while it is actually Running on its
CPU, or a `KiSelectReadyThread` remove that doesn't fully unlink, or the unlocked
`NextThread` pre-check race at `KiDpcInterruptHandler` (checks `Prcb->NextThread`
before taking the PRCB lock). Enriched forensics added: `[RQLINKED]` (at the `0x54`
site) scans `KiProcessorBlock` to report WHICH cpu's `DispatcherReadyListHead[pri]`
the thread is wrongly linked into + its state + owning PRCB; `[RQTEAR]` (at
`ki_rq_integrity`) dumps the corrupted link VALUE. One amplified burst with these
should name the list + the value, from which the upstream writer follows.

**2026-07-13 FORENSIC BURST — WILD-WRITE THEORY DEAD; it's a scheduler logic bug.**
The `[RQLINKED]`/`[RQTEAR]` dumps are unambiguous:
- `[RQLINKED]`: EVERY hit shows `thread.Flink == thread.Blink == head` where `head`
  is the PRCB's `DispatcherReadyListHead[9]`, with `ownerCPU == thisCPU == 1`. That is
  a thread **correctly, currently the sole element of CPU1's priority-9 ready list**
  (head⇄X consistent) — being (re)queued from `traphandler.rs:106` (OldThread=
  CurrentThread) AND `wait.rs:393` (KiExitDispatcher, also the outgoing CurrentThread).
  Observed states: `2`=Running (the paradox) and `1`=Ready.
- `[RQTEAR]` (site 0x22): the DOWNSTREAM wreckage — `head.Flink=head.Blink=X` but
  `X.Flink=X.Blink=X` (self-looped), i.e. the one-element list after the botched
  re-insert. `BADPTR` is just the self-looped node, NOT a foreign fingerprint.
- List primitives (`RemoveEntryList`/`InsertTail/Head`/`InitializeListHead`) verified
  correct-standard. **No foreign value ever appears in a torn link => NOT a wild
  write.** The `0x19`/`0xF4` faces are downstream of the same scheduler damage (or the
  racy pool checker), not an independent corruptor.

**THE INVARIANT VIOLATION (precise):** `Prcb->CurrentThread` is simultaneously the
sole element of that same Prcb's `DispatcherReadyListHead`. Both crash sites just
re-queue the outgoing CurrentThread (NT-faithful), so the bug is whoever left it
linked. Two sub-hypotheses, and a discriminator was built to decide between them:
(A) a **set-NextThread path** promoted a still-linked thread to Running (so it was
linked *before* it became current); (B) a **cross-CPU queue-while-running** race
linked it *after*. New guard `ki_check_promote_not_linked(NextThread, site)` fires at
the 4 promotion points (NextThread->CurrentThread=Running): `0x01` KiExitDispatcher,
`0x02` KiDpcInterruptHandler, `0x03` KiQuantumEnd, `0x04` KiSwapThread. If it fires
(`0x55` bugcheck + `[RQPROMOTE]` print) => hypothesis A and the site names the swap
that saw it; if it stays SILENT while `0x54` keeps firing => hypothesis B (cross-CPU),
and the next step is a guard in `KxQueueReadyThread` when it queues to a REMOTE prcb
whose CurrentThread == the thread being queued.

**2026-07-13 DISCRIMINATOR RESULT = hypothesis B.** `[RQPROMOTE]`/`0x55` stayed
SILENT across the burst (NextThread is CLEAN at all 4 promotion points) while `0x54`
kept firing (state=`1`=Ready now). So the thread becomes CurrentThread cleanly and is
**readied/linked into the ready list WHILE it is the running thread** — a queue of a
running thread. `0x54` only catches the *second* (already-linked) queue; the *first*
illegal queue is the culprit. New catcher `ki_check_not_running(thread)` scans
`KiProcessorBlock`; if the thread being queued is CurrentThread of ANY cpu it prints
`[RQRUNNING]` + bugchecks `0x56` with the queuer's `#[track_caller]` line. Placed at
BOTH ready-list entry points: `KxQueueReadyThread` entry and `KiDeferredReadyThreadImpl`
entry. Legit dispatch/yield/quantum paths dethrone (Prcb->CurrentThread=NewThread)
BEFORE queuing the outgoing thread, so this is quiet on them and fires only on the
illegal readier. That `0x56`/`[RQRUNNING]` caller line == THE BUG.

**2026-07-13 ROOT CAUSE FOUND — `[RQRUNNING]` names it: `KiReadyThread`
(thrdschd.rs:555) with `state=5`=Waiting.** The readier-side of the SwapBusy
handshake is MISSING (ReactOS's SMP never implemented it; our UP-derived translation
inherited the omission). Full mechanism:
1. Thread T runs on CPU1 (`CurrentThread=T`). T commits a wait: sets `State=Waiting`,
   sets `SwapBusy=1`, calls `KiSwapThread` — but is **still `Prcb->CurrentThread`**
   until `KiSwapContext` actually switches away (`SwapBusy` clears in
   `KiSwapContextResume`, AFTER the dethrone).
2. CPU0 satisfies T's wait (`KiUnwaitThread` -> `KiReadyThread(T)` ->
   `KiInsertDeferredReadyList(T)`) in that window: T is `Waiting` but STILL running on
   CPU1. T gets `State=DeferredReady`, pushed to CPU0's deferred list.
3. CPU0 drains -> `KiDeferredReadyThreadImpl(T)` LINKS T into a ready list (or sets it
   `NextThread`) while T is **still CurrentThread on CPU1**.
4. T is now both running on CPU1 AND a schedulable ready-list element -> a later
   (re)queue double-links it -> the DispatcherReadyListHead tear (0x54/0x11/0x22), and
   the assorted `0x1E`/`0x19`/`0xF4`/wedge faces are all downstream of that.

**THE FIX (readier-side SwapBusy spin):** at `KiDeferredReadyThreadImpl` entry (the
deferred-drain link point), spin `while read_volatile(&Thread->SwapBusy) != 0 {
spin_loop() }` before making T schedulable. `SwapBusy` is set during the wait-commit
and cleared by `KiSwapContextResume` only AFTER T is dethroned, so the spin guarantees
T has left its CPU first. Mirrors the RUN-side spin already in `ctxswitch.S`.
**Lock-safety (critical):** the drain runs with NO PRCB lock held (called from
`KiCheckDeferredReadyList` after the dispatch/PRCB locks are released), and the
descheduling CPU clears `SwapBusy` in `KiSwapContextResume` without needing any lock
the spinner could hold -> no deadlock. (Do NOT put this spin inside
`KxQueueReadyThread` or `KiInsertDeferredReadyList` — those run under a PRCB lock the
descheduling CPU may need, which WOULD deadlock. The drain entry is the one lock-free
readier point.) The `ki_check_not_running` tripwire is kept armed right after the spin
as the validator: it must go SILENT.
**Residual window (known, likely negligible):** between `State=Waiting` (set first) and
`SwapBusy=1` (set a few lines later in the wait-commit), a readier that reaches the
drain that fast would see `SwapBusy=0` and pass. If the validator still fires post-fix,
tighten by setting `SwapBusy` BEFORE `State=Waiting` in the 6 wait-commit sites
(wait.rs 496/500, 557/561, 679/683, 771/775, 979/983, 1132/1136).
**Validation:** rebuild `--smp-kernel --cswitchdep`, burst ×16 — `0x56`/`[RQRUNNING]`
+ the `0x54`/`0x11`/`0x22` faces must go silent and the flake rate must drop toward 0.
A deadlock from a mis-placed spin would show as INCREASED hangs (revert if so).

**2026-07-13 FIX ATTEMPT #1 FAILED (drain-spin) + reverted — and it exposed a PARADOX
that is the real next lead.** The drain spin **deadlocked every boot** (0/16, wedge at
1 cswitch): **deferred-ready threads NORMALLY carry `SwapBusy` set** (it is part of the
readying transition), so `while SwapBusy` at `KiDeferredReadyThreadImpl` blocks the
normal path forever. The "silent tripwires" it produced were a FALSE negative (the
kernel hangs before reaching the corruption). Reverted (commit on branch). Lesson:
`SwapBusy` set ≠ "still CurrentThread"; do not gate the drain on it.

**THE PARADOX (this is where the next session starts):** the confirmed root — a thread
readied by a cross-CPU `KiReadyThread`/unwait *while still `CurrentThread` mid-wait-
commit* — **should be impossible under the dispatcher lock.** The wait-commit holds the
global queued dispatcher lock (`KiAcquireDispatcherLockAtSynchLevel`, wait.rs:461)
continuously through `State=Waiting` (496), `KiAddThreadToWaitList` (498),
`KiSetThreadSwapBusy` (500), and into `KiSwapThread` (502) — there is NO
`KiReleaseDispatcherLock` before the swap. The unwait path *also* holds the dispatcher
lock (`eventobj.rs:75` `KiAcquireDispatcherLock` -> `KiWaitTest` -> `KiUnwaitThread` ->
`KiReadyThread`). Two holders of the same global lock CANNOT overlap — so the unwait
must wait until the commit releases it, by which point `T` should be dethroned. Yet
`ki_check_not_running` (0x56) fires. **One of these must be true — resolve it first:**
1. **The dispatcher lock is released too EARLY in the swap** — before `KiSwapThread`
   dethrones `Prcb->CurrentThread`. TRACE where the queued dispatcher lock acquired at
   wait.rs:461 is actually released during `KiSwapThread`/`KiSwapContext`/
   `KiSwapContextResume` (asm `ctxswitch.S` + thrdini.rs). If it is released before the
   dethrone (or before `SwapBusy`/state settle), a cross-CPU unwait slips in. This is
   the prime suspect (an SMP dispatcher-lock-release-ordering translation bug).
2. **A readier bypasses the dispatcher lock.** Audit ALL `KiReadyThread` callers
   (eventobj:204, gate:268, queue:123, thrdobj:263, wait:204) — does every one hold the
   dispatcher lock? One that doesn't is the race.
3. **The queued dispatcher lock isn't actually global/serializing** in our SMP impl
   (e.g. `KiAcquireDispatcherLock` vs `KiAcquireDispatcherLockAtSynchLevel` resolve to
   different locks). Verify both resolve to the SAME `LOCK_QUEUE_DISPATCHER_LOCK` global.

The right fix follows from which of (1)/(2)/(3) is true — most likely (1), and the fix
is to release the dispatcher lock at the correct point (after dethrone), matching NT.
`ki_check_not_running` (0x56) stays armed as the exact oracle: the fix is right when it
goes silent AND the flake rate drops WITHOUT new hangs. Do this careful lock-ordering
work FRESH, not at the tail of a long run — attempt #1 shows how a plausible-but-wrong
placement makes things strictly worse.

**2026-07-13 — hypothesis (3) [LockQueue not wired] RULED OUT; paradox refined.**
Verified amd64 wires the dispatcher lock correctly on BOTH CPUs: the real amd64
`KiSystemStartupBootStack` lives in `sdk/lib/rust/ntoskrnl-rust/src/ke/amd64/
krnlinit.rs:146` (NOT kiinit.rs; the C `amd64/krnlinit.c` is cohab-commented at
ntos.cmake:410), and at line 164 it calls `KiInitSpinLocks(Prcb, Prcb->Number)`
BEFORE the `Number==0` branch — so both the BSP and the AP set
`LockQueue[DispatcherLock].Lock = &KiDispatcherLock` (one shared global). Both CPUs'
`KeAcquireQueuedSpinLock*` therefore contend the SAME lock word; the dispatcher lock
genuinely serializes. So (3) is out. Remaining, for the fresh session:
- **(2) a `KiReadyThread` caller bypasses the dispatcher lock.** Audit all callers
  (eventobj:204, gate:268, queue:123, thrdobj:263, wait:204) — one readying without
  the dispatcher lock held is the race. CHEAPEST to check; do first.
- **(4) the dispatcher lock is released too early across the swap.** Our wait-commit
  holds it from wait.rs:461 through `KiSwapThread` with NO release before the swap, so
  it is handed off across `KiSwapContext` (T swaps out holding it, NextThread swaps in
  holding it, NextThread releases it later). If our handoff releases it BEFORE the
  dethrone (`Prcb->CurrentThread = NextThread` in KiSwapThreadImpl) is visible, a
  cross-CPU unwait slips in. Trace exactly where `&KiDispatcherLock` is released
  relative to the dethrone in the swap.
- **(5) `ki_check_not_running` (0x56) may be catching a TOLERATED transient.** Take
  seriously: "readied while still `CurrentThread`" is arguably the NORMAL handoff
  window (T is `Prcb->CurrentThread` until dethroned; the run-side spin in ctxswitch.S
  protects actually RUNNING it). If NT tolerates linking such a thread (its
  `WaitListEntry` is free, so it's a clean single insert), then 0x56 is a red herring
  and the REAL bug is the DOUBLE-link (0x54) with a different cause. Before building
  another fix on 0x56, confirm it is not benign.
**DECISIVE NEXT INSTRUMENT (splits (2) vs (4)):** in `ki_check_not_running`, also read
and print whether `&KiDispatcherLock` (bit 0) is HELD when it fires. Held => a readier
is running under the dispatcher lock yet the race still happened => (4) [early release
/ handoff bug]. Not held => (2) [a readier bypasses the lock]. One armed burst decides.
State at this point: fix attempt #1 reverted, clean baseline restored, all diagnostics
(0x54/0x56/[RQRUNNING]/[RQLINKED]) committed + armed on branch `smp-bringup`. NOT on
master. The root-cause DIRECTION (readier vs wait-commit race) is solid; the precise
serialization break + the correct fix are the remaining work.

**2026-07-13 (fresh session) — THE PARADOX DISSOLVES: 0x56 was a FALSE ORACLE, and the
real defect is the SELF-SWAP WEDGE.** Two findings that rewrite the frame:
1. **"Readied while still CurrentThread" is NT's LEGAL, designed-for window — hypothesis
   (5) was right.** The earlier claim "wait.rs never releases the dispatcher lock before
   the swap" was WRONG: the C releases it in EVERY commit path *before* `KiSwapThread`
   (wait.c:567/859 explicitly; the timeout paths via **`KxInsertTimer`, which embeds
   `KiReleaseDispatcherLockFromSynchLevel()`** — ntkrnl-sys inline/ke.rs:1599 is
   faithful, and wait.rs mirrors all of it). So by design a cross-CPU readier may
   acquire the dispatcher lock, unwait T, and link/standby it while T is still
   `Prcb->CurrentThread` mid-commit; safety is delegated entirely to the per-PRCB locks
   + the run-side SwapBusy spin. BOTH splitter faces (dispLockHeld=1/Waiting and
   dispLockHeld=0/gate) are this legal transient. `ki_check_not_running` downgraded to
   print-only (`[RQRUNNING]` = breadcrumb marking the window, NOT a bug); its bugchecks
   were killing overnight bursts on legal behavior and masking the real fault downstream.
2. **The real defect: the legal window enables a SELF-TARGETING READY, and our swap
   wedges on it.** Sequence: T mid-commit on CPU1 (lock released, still CurrentThread,
   `SwapBusy=1`); CPU0's deferred drain — typically after the **AdjustUnwait priority
   boost** (thrdschd.rs:275-324; explains the observed pri-9/10) — targets CPU1 and
   either (a) preempt-tests `Thread->Priority > Prcb->CurrentThread->Priority` where
   CurrentThread==T itself, so boosted-T preempts ITSELF → `Prcb->NextThread = T`
   (self-standby), or (b) equal-pri falls through → T linked into CPU1's own ready list
   (self-queue, the `[RQLINKED]` sole-element face). CPU1's `KiSwapThread` then resolves
   **NextThread == CurrentThread == T** and calls `KiSwapContext(T, T)`; ctxswitch.S's
   run-side spin waits on `[T+ThSwapBusy]` — which only clears in `KiSwapContextResume`
   AFTER the spin, i.e. **T waits forever on its own flag → CPU dead at SYNCH → the
   silent HANG_TIMEOUT / service-connect creep-wedge face.** NT tolerates old==new
   (semantically "signaled before I left — resume in place"); ReactOS's never-validated
   SMP spin (inherited verbatim by the C at ke/{amd64,arm64}/ctxswitch.S — i386 has no
   spin at all) does not. Tripwire **0x57** armed at `KiSwapThreadImpl` post-resolution
   (`NextThread == CurrentThread` → 0x96, P2 tag 0x57|state|cpu, P3=SwapBusy, P4=pri) to
   convert the silent wedge into an attributed report. PREDICTION for the armed burst:
   0x57 replaces (most of) the HANG_TIMEOUTs. FIX once proven: old==new short-circuit of
   the SwapBusy spin in ctxswitch.S (`cmp rdx, r8 / je` past the spin on amd64; arm64
   `x1 vs x2` twin at ctxswitch.S:159; the rest of the self-swap path is benign —
   verified: stack save/reload is idempotent, resume clears SwapBusy, no CR3 change).

**2026-07-13 (b) — 0x57 burst result: self-swap NOT the (first) culprit; forensics
instead CONVICTED a lockless gate signal → ROOT CAUSE = `KiTryThreadLock` NEVER
ACQUIRES (an inherited UPSTREAM ReactOS SMP bug).** The 0x57-armed burst (16 boots,
12/16 green) produced ZERO 0x57 but 3× the site-0x22 tear with pristine forensics:
`[RQTEAR]` showed the JUST-INSERTED sole-element entry either self-looped
(`entry.F=entry.B=entry`, head still →entry) or half-foreign (`entry.F=our ready
head, entry.B=another thread's WaitListEntry`). Working the interleavings backward,
both shapes are the exact fingerprints of **two CPUs concurrently InsertTailList-ing
the SAME KTHREAD WaitListEntry union into two different lists** (the deferred
drain's ready insert racing the victim's own `KiAddThreadToWaitList` in its next
wait-commit; the interleave where writer B re-reads `head.Blink` after writer A's
store yields the self-loop exactly). A ready that early is only possible if a
signaler bypassed the wait-side serialization — and the only lockless readier is
`KeSignalGateBoostPriority`, whose sole mutual exclusion against the gate waiter's
commit is `KiTryThreadLock`. Reading it: **`ntkrnl-sys/inline/ke.rs` returned
0/"success" when the lock read FREE — WITHOUT ACQUIRING IT** (only the contended
path did the interlocked swap). Faithfully translated from **ReactOS's own
`ke_x.h:533`** (`if (!Thread->ThreadLock) return FALSE;`) — dead code upstream
(ReactOS never shipped SMP; the UP variant is a no-op), a live catastrophic hole at
`-smp 2`: on the COMMON (uncontended) path the gate signaler popped + readied the
waiter with NO lock, including the window BEFORE the waiter set `SwapBusy` — so the
waiter could be picked up and RUN on another CPU while still running its commit on
its own CPU (**double-run, one stack, two CPUs**), or its deferred ready could land
mid-next-wait-commit (the observed double-insert tears). Paged pool rides guarded
mutexes → gates, so the traffic is constant. FIX (both homes): try-ACQUIRE semantics
— visibly-held → fail(1); else interlocked swap(1), old!=0 → fail — in
`ntkrnl-sys/inline/ke.rs::KiTryThreadLock` (live) and `ke_x.h:528` (reference
parity). Lesson (reusable, add to the §1–8 canon): **"faithful to the C" is not
"correct" for SMP-only C paths — ReactOS's CONFIG_SMP code was NEVER validated
(it never booted SMP upstream); treat every inherited SMP-branch primitive as
unreviewed and verify it against NT semantics, not against ReactOS.** 0x57 stays
armed through validation: with the trylock fixed, a properly-serialized gate signal
still readies a pre-dethrone waiter (legal NT window), so the self-swap wedge may
now surface as the NEXT layer — if 0x57 fires, apply the old==new spin-skip in
ctxswitch.S. Validation burst running: expect 0x22/[RQTEAR] silent, flake → ~0.

**2026-07-13 (c) — trylock-fix validation: 12/16 → 14/16; tear NOT fully gone; two
more holes found and plugged.** Post-fix burst: 2 fails — 1× the same site-0x22
self-loop tear (pri 9), 1× 0x1E c0000006 post-marker. So the gate trylock was A
source (rate improved), not the only one. Re-decode of the self-loop: it does NOT
require two CPUs — a SINGLE sequential `InsertTailList(H, e)` when `H.Blink == e`
(thread ALREADY the tail/sole element of that same list) produces `e.F=e.B=e`
exactly. I.e. a plain SEQUENTIAL DOUBLE-READY of a parked thread — and the 0x22
drain-insert site has NO already-linked guard of its own. Two findings this round:
1. **The 0x53 "still-linked defer" oracle had a HOLE that invalidates the earlier
   elimination-round refutation:** its linkage check was gated on `State==Ready`,
   but `KeSignalGateBoostPriority` sets `State=DeferredReady` ITSELF before calling
   `KiReadyThread` — so a still-linked gate-woken victim arrived at
   `KiInsertDeferredReadyList` with state 7 and was never checked. Replaced with
   **0x58**: linkage check for ALL states + `[RQDOUBLE]` print (state, WaitReason,
   WaitStatus, DeferredProcessor — enough to name the waker class without
   track_caller). Also learned: `[RQRUNNING] state=7` prints are ROUTINE (gate wakes
   legitimately enter KiReadyThread with state pre-set to 7) — not evidence.
2. **The Rust `Kx{Single,Delay,Queue}ThreadWait` only carried HALF of the C's
   defensive timer-block init** (`ke_x.h` writes BOTH ends: `TimerBlock->
   WaitListEntry.F/B = &Timer->Header.WaitListHead` AND the head's F/B; the Rust
   inline wrote only the head side). A stale/self-referencing `TimerBlock->
   WaitListEntry` makes `KiUnlinkThread`'s ring-walk `RemoveEntryList` write through
   dangling pointers (tear class) or no-op (the documented KiTimerExpiration spin).
   Ported the both-ends init to all 3 Rust setups (KxMultiThreadWait already used
   InitializeListHead+InsertTailList — fine).

**2026-07-13 (d) — oracle-escalation ladder converged; first 16/16 GREEN amplified
burst.** Burst-by-burst (all `--smp-kernel --cswitchdep` ×16, same host):
- 0x59 burst (double-DEFER in-flight table): **0x59 SILENT**, 12/16, 4× tear —
  so no double-push either; the second insert reaches the drain-insert site
  without passing KiReadyThread/KiInsertDeferredReadyList.
- 0x5A/0x5B burst (pre-insert linked guard at the drain's own insert + chain-cycle
  cap): NO tears at all; 14/16 with 2× a NEW deterministic face — 0x1E c0000006 at
  the SAME PC both times = **KiSelectReadyThread+0xE2** (ntkrnlmp symbolication;
  the ready-list walk eating a wild entry — downstream victim, not culprit).
  Precursor print: a thread arriving AT THE DRAIN with **State==Ready** — i.e. it
  was KxQueueReadyThread'd (state=1 + linked, union Next overwritten) WHILE parked
  on a deferred SLIST → the drain then walks ready-list interiors as SLIST links.
- 0x5C/0x5D burst (drain pre-validates every chain node is DeferredReady-for-this-
  cpu; KxQueueReadyThread bugchecks on a DeferredReady input): **16/16 GREEN,
  both oracles silent.** Suspicious (green + no hits): either the cumulative fixes
  (trylock + timer-block init) close it and earlier 0x1E pair was residual timing,
  or 0x5C's per-drain walk perturbs the window (Heisenberg), or luck (~12% for a
  16-green at the prior rate). ×32 confirmation burst running.
Armed oracle inventory (all on branch, to be de-armed before landing): 0x54
(KxQueueReadyThread linked), 0x55 (promote-linked), 0x57 (self-swap), 0x58
(defer-linked, all states), 0x59 (double-defer table), 0x5A (drain-insert linked),
0x5B (chain>64), 0x5C (chain node not DeferredReady-for-cpu), 0x5D
(KxQueueReadyThread on DeferredReady); prints [RQRUNNING] (benign-window
breadcrumb) [RQTEAR] [RQDOUBLE] [RQDEFER2] [RQ5A] [RQ5C] [RQ5D].

**2026-07-13 (e) — evening rounds: every WAKER acquitted; the anomaly is now
"running thread in ready-flight" with all doors guarded.** Round-by-round:
- ×32 post-16/16: 29/32, **0x5D fired ×2** — threads found DEFERRED-PARKED while
  being the outgoing CurrentThread at the preempt/exit-dispatcher requeues
  (traphandler:109, wait.rs:396→now 427/429). So something wakes RUNNING threads.
- **0x5E** (KiUnwaitThread on non-Waiting = spurious unwait, WaitStatus names the
  waker class) + **0x5F** (gate pop of non-GateWait): both **SILENT** across ×32 —
  unwait and gate funnels are CLEAN. (Also: a 0x54 read as "state=GateWait" was a
  P2 MISREAD — that byte is the PRIORITY; the RQLINKED companion print showed
  state=Ready. Always cross-check P2 decodes against the companion dumps.)
- **0x60** (manual queue wake in KeInsertQueue — bypasses KiUnwaitThread, so 0x5E
  is blind to it; fingerprint WaitReason=0): **SILENT** — queue wake clean.
- **In-flight oracle lifecycle v2**: mark at defer-push, unmark at
  PROMOTION-to-Running (7 sites: KiSwapThreadImpl ×3, KiExitDispatcher,
  KiQuantumEnd, KiDpcInterruptHandler, amd64 idle loop — the idle loop was missed
  first and leaks marks), requeue variant (no collision check) for the legal
  repositioners: KiSetPriorityThread ×2, KiUpdateEffectiveAffinity ×2, balmgr
  boost re-defer (missed first → 0/32 false-0x59s, every boot, caller 114 —
  LESSON: enumerate ALL repositioners before arming a wake-to-run mark).
- v2 ×32: 29/32 — **0x59 STILL SILENT** (no double-push even wake→run), hits =
  0x5D ×2 + **0x5A** (drain-insert already-linked, state=Ready, reason=31
  =WrDispatchInt → last touched by the DPC-preempt requeue). Breadcrumb: the
  drain observed a chain node with **state=Running mid-walk**, and one drain ran
  with dispLockHeld=1. Reading: a wake races the victim's own preempt-requeue
  (both orderings caught), waker STILL unidentified — all guarded funnels pass.
- Current instrument: **waker identity stamping** — RQ_WAKER_LINE parallel array
  records each defer's caller line (Requeue=9000); 0x5A/0x5D dumps now print
  `deferred_by=<line>`, naming the waker retroactively at the catch point.
  Remaining unguarded readier if deferred_by comes back useless: KeReadyThread
  (thrdobj — stack-inswap; would need state==Transition guard).

**2026-07-13 SPLITTER RESULT — MIXED, so it is TWO faces, not one:**
`[RQRUNNING]` now prints `dispLockHeld`. Across a burst:
- **`dispLockHeld=1` AND `state=5`=Waiting** (both held cases): the readier HOLDS the
  global dispatcher lock, yet the thread is `Waiting` + still `CurrentThread` on the
  OTHER cpu. => **hypothesis (4) CONFIRMED as the primary face.** The wait-commit's
  DETHRONE is not serialized against the dispatcher lock: the descheduling cpu has let
  a lock-holding readier proceed while its thread is still `CurrentThread`. The
  wait-commit (wait.rs:461 acquires the dispatcher lock, holds it into `KiSwapThread`
  at 502) hands the lock off across `KiSwapContext`; the bug is that the handoff frees
  the lock (or the dethrone becomes visible) at the wrong point relative to
  `Prcb->CurrentThread = NextThread` in `KiSwapThreadImpl`. FIX = make the dethrone
  happen-before any other cpu can acquire the dispatcher lock (trace the exact
  release/handoff point of `&KiDispatcherLock` across the swap; align to NT).
- **`dispLockHeld=0` AND `state=7`=DeferredReady**: lockless readier / deferred-drain
  re-processing. => **hypothesis (2) also real.** Concrete instance:
  `gate.rs:268` (`KeSignalGateBoostPriority`) calls `KiReadyThread(WaitThread)` AFTER
  releasing the gate object lock (254), the thread lock (256), and the dispatcher lock
  (265) — a fully lockless ready. If that gate-waiter is still descheduling
  (`CurrentThread` mid-swap), it readies a running thread. Audit the gate/queue signal
  paths against NT for the missing serialization.
Both faces reduce to the same invariant: **a thread must be fully dethroned (no longer
any cpu's `CurrentThread`) before it can be readied/linked.** The correct fix enforces
that — most cleanly by fixing the dispatcher-lock/dethrone ordering (face 4) and the
lockless gate/queue readiers (face 2). `0x56`/`dispLockHeld` is the exact oracle for
both. This is careful SMP lock-protocol work for a FRESH session — the overnight run
took it from "wild-write mystery" to two precisely-characterized serialization faces
with a live oracle, a clean reverted baseline, and every step committed.

**Integrated from a Codex handoff attempt (its cyber filter tripped on the "wild
write" language, so it stalled — reword future handoffs):** (a) `expool.rs` removed
the post-unlock `ExpCheckPoolBlocks` page-walk on the theory it RACES (walks a shared
page after dropping the pool lock → can read a neighbour mid-split → **false** 0x19).
NT keeps that call (`ex/pool` ~line 2455) but only under `DBG`; our port runs it
always-on, so it's a plausible always-on-checked-build-assert-as-SMP-bug (cf. §9.57).
KEPT as a marked experiment; permanent fix if confirmed = gate to checked-builds, not
delete. (b) `#[cfg(not(smp))]` on the local `KxQueueReadyThread` externs in
`traphandler/dpc/wait.rs` — CORRECT, removes decls that were still shadowing the
`ke_defs` inline-repoint on SMP (this is why `0x54` now reports `traphandler.rs:106`
instead of the shim). (c) `[WAITDEP]` waitlist dump — harmless additive §9.36 signal.

**PRIOR (superseded) NEXT-DIAGNOSTIC note:** the 0x52 bugcheck currently reports
`(thread, (0x52<<8)|pri, state, prcb)` but NOT *who called*. Add the caller
return address (`code_addr!()` at each KxQueueReadyThread site, or capture the
return address in KxQueueReadyThread itself) into a spare 0x52 param, rebuild
`--smp-kernel --cswitchdep`, and run one ~16-boot amplified burst — that pins the
exact second-queuer call site in one cycle. THEN read that path against NT
ke_x.h/thrdschd.c for the missing lock/removal. Keep the 0x96/0x52 tripwires
armed (guarded, cheap) until a clean soak confirms the eventual fix.

**Separate latent (M5, not this flake): i386 `ctxswitch.S` has NO SwapBusy spin
at all** (matching ReactOS's incomplete i386 SMP) — a real gap for a future i386
SMP kernel, but the current flake is amd64 where the spin is present.

**2026-07-13 (f) — TWO more finds: the oracle's own hash table lied (false 0x59
flood), then the deduped oracle caught the REAL hole: `KiIdleLoop` consumes
`NextThread` with NO PRCB lock.**

*Round 1 — the false flood (a10c86602f3).* The attribution burst came back 11/32,
ALL nine 0x96 = tag 0x59 (double-defer) caller=708 (`KiReadyThread`), victims all
KQUEUE-woken ExWorker threads (decode tip: `WaitStatus` = a POINTER for queue waiters
— NT's queue wake stores the dequeued entry in WaitStatus; `0x100` = STATUS_KERNEL_APC).
Root cause was the ORACLE ITSELF: `rq_inflight_mark()` CAS-inserted at the first empty
probe slot *before finishing the 8-slot scan* for an existing entry. A legal Requeue
re-mark (worker priority boosts, constantly) after an earlier-probe slot freed created
a duplicate TWIN entry; promote-unmark cleared one, and the stale twin fired a false
0x59 on the next legal wake. **Reusable rule: in an open-addressed probe table,
insert-at-first-empty must only happen AFTER a full-window scan for the key** — and a
diagnostic that bugchecks must be held to the same correctness bar as the kernel it
polices (this is the SECOND false oracle of §9.56 after 0x56). Fix: full-scan-first
mark, clear-all unmark, and `stale_by` (the stale mark's waker stamp) printed + packed
into 0x59 P3.

*Round 2 — the real hole (466d66bf550).* Deduped burst: 28/32, three catches, full
attribution: 2× `[RQ5D]` caller=429 (= `wait.rs` KiExitDispatcher requeuing the OLD
running thread) with `deferred_by=9200` (the standby-DISPLACEMENT re-defer), 1×
`[RQ5A]` already-linked on the drain's private detached chain — and every victim
`runningOnCPU=1` with interleaved `[RQRUNNING] caller=8888` breadcrumbs proving the
drain processed a thread that was simultaneously CurrentThread on the other CPU.
Mechanism: **`KiIdleLoop` read/nulled/promoted `Prcb->NextThread` LOCK-FREE** —
inherited verbatim from ReactOS's never-validated CONFIG_SMP C (i386 AND amd64 both
lack the acquire; same class as the KiTryThreadLock find: *treat every inherited
CONFIG_SMP primitive as unreviewed*). The displacement branch of
`KiDeferredReadyThreadImpl` (correctly under the PRCB lock) read `NextThread=T` while
the idle CPU concurrently promoted T to Running, then set the RUNNING T DeferredReady
and re-deferred it → T later requeued itself run-side while still on a drain chain →
the double-link tear. Every OTHER promote site (KiExitDispatcher, KiQuantumEnd, yield,
KiDpcInterruptHandler) already held the lock — idle was the single hole. Fix: acquire
PRCB lock in the idle promote, RE-CHECK NextThread under it (a displacement may have
consumed it), promote, release before the SYNCH raise + swap. **Audit rule: a field
guarded by lock L at N−1 of N sites is not guarded** — grep every reader/writer of
`Prcb->NextThread` when touching scheduler code. Non-scheduler residuals in these
bursts are the known §9.36 family (csrss 0xF4 after rpcrt4 `handler continued
execution`; lsass user-AV hang) plus a cswitchdep tap miscalibration under SMP
(amd64 MP swaps at SYNCH_LEVEL=12, tap expects DISPATCH=2 — fold into the owed
per-CPU-cswitchdep work).

**2026-07-14 (g) — idle-lock v1 SELF-REGRESSED (a lesson in lock+IRQL coupling),
v2 lands the fix; oracles all-silent at 29/32; de-armed.**
- **v1 (acquire AFTER `sti`) wedged 12/32 + 8× 0xF4, zero oracle hits.** Holding the
  non-reentrant PrcbLock with interrupts ENABLED on the idle CPU lets any nesting
  that reaches scheduler code on that CPU re-acquire → self-deadlock → system-wide
  starvation (faces: post-marker taskbar-OnCreate wedge, csrss 0xF4 storm). **Rule:
  adding a lock is only half the fix — the acquire must also inherit the IRQL/
  interrupt discipline of the lock's other holders.** All other PrcbLock takers run
  with the dispatch path masked; idle's `sti` made it the odd one out.
- **v2 (4fc6a98c9b5): acquire→re-check→promote→release entirely inside the idle
  loop's existing `cli` window, `sti` only after release** (UP keeps the C-faithful
  early `sti`). Burst: **29/32, ZERO bugchecks, ZERO oracle hits, [IDLE-IRQL]
  tripwire silent** (idle was at DISPATCH — v1's nesting vector was interrupt-driven,
  not a below-DISPATCH bug). 1 of the 3 hangs = host contention ("serial still
  flowing at deadline").
- **New residual face to watch (NOT the tear): census `STUCK … GateWait …
  HOLDER-STUCK(no signal ever)`** — 77 hits incl. on green boots (previous bursts:
  0), SERVICES/setup/rpcss parked on session-space guarded-mutex gates whose holder
  never signals. Either instrumentation drag (cswitchdep taps + oracles slow the
  promote) or a real starvation the idle serialization unmasked. Census now prints
  the heuristic gm-owner (`gm? owner=… state=…`) so the next wedge names the HOLDER.
- **De-armed (db5b542a138):** `RQ_ORACLE_ARMED=false` turns the 15 protocol oracles
  (0x54–0x63) print-only; corruption fingerprints (torn ready-list links, null queue
  entries, ReadySummary inconsistency) stay armed. Fix set that closed the tear:
  gate trylock + timer-block both-ends init + in-flight table dedupe + idle
  NextThread lock v2.

### 9.57 (ROOT-CAUSED + FIXED 2026-07-11) A *correct* fix woke a latent panic: `NT_ASSERT` mistranslated as a hard `assert!`, dormant until IRP_MJ_CLEANUP started being delivered (arm64-only)

**Symptom:** After merging origin/master into the conformance-ratchet lane, the ×3 landing
gate was green on i386 + amd64 but **arm64 wedged during boot** — Rust-panic BUGCHECK
`0xE0525354` (`"RST"`), serial silent, last line a flight-record `ws-lock/unlock-excl`
livelock (a red herring — post-mortem ring-buffer history, not the cause). Deterministic 3/3.

**Two-layer root cause — this is the instructive part:**

1. **The trigger was one of *my own correct fixes.*** This lane's ob-rust
   `ObpDecrementHandleCount` fix (field-manual "post-inc/dec read-order" class) makes
   `IRP_MJ_CLEANUP` *actually get delivered* for the first time (it had been silently
   dropped by a `!= 1` guard reading the post-decrement count). Correct Windows behavior —
   but it makes a previously-**dead** code path **live**: `FatCommonCleanup` now runs.
2. **The latent bug it exposed:** `FatLocateDirent` (fastfat) begins with
   `NT_ASSERT(ExIsResourceAcquiredShared/Excl(ParentDcb->Header.Resource) || ...Vcb...)`.
   ReactOS `NT_ASSERT` is **debug-only** (a `DbgRaiseAssertionFailure` in checked builds,
   **compiled out in free/shipping builds**; the vibeos cohab shim `NT_ASSERT`,
   `rust_cohab_stub.c:353`, is a literal no-op). The C→Rust translation rendered *this one*
   as a hard `assert!(...)` — panics unconditionally — while every *other* NT_ASSERT in the
   same file correctly routes through the no-op `NT_ASSERT()` shim (cf. dirsup.rs:1157/1164).
   The full live path: `IRP_MJ_CLEANUP → FatFsdCleanup → FatCommonCleanup →
   FatNotifyReportChange (macro) → FatSetFullFileNameInFcb → FatGetUnicodeNameFromFcb →
   FatLocateDirent`. On the normal UserFileOpen cleanup path FatCommonCleanup legitimately
   holds **only the file's Fcb** (not the parent Dcb / Vcb) — exactly as the C does
   (cleanup.c grabs the Vcb only for delete-on-close / UserVolumeOpen). Free-build ReactOS:
   NT_ASSERT gone, read proceeds. Rust: `assert!` fires → BUGCHECK.

**Why arm64-only (the arch-specificity was the first real clue, not a distraction):**
arm64 boots from a **FAT ramdisk** → fastfat is exercised heavily at boot (driver `.sys`
loads open+close through it). i386/amd64 boot the **LiveCD ISO** → **cdfs**, and never
touch fastfat cleanup during boot. So an all-arch kernel change (IRP_MJ_CLEANUP delivery)
surfaces a fastfat bug on exactly one arch. **Lesson:** when a change is all-arch but the
failure is one-arch, ask *what does that arch exercise that the others don't* — here, the
boot filesystem, not anything CPU-specific.

**Isolation recipe that nailed "mine vs pre-existing" fast (~15 min):**
- **A/B against pristine mainline in a throwaway worktree.** `git worktree add -d /tmp/... origin/master`, symlink `.build-tools` in, copy the gitignored arm64 EFI assets, `vibeos-cli build --arch arm64`, smoke. Pristine master booted 2/2; my tree 3/3 red → **my change**, not a merge/pre-existing regression. This flipped my working hypothesis (I'd assumed pre-existing) — *run the A/B, don't reason about it.*
- **fastfat.sys was byte-identical (modulo build-stamp) across the two trees** → the bug is in how the *kernel drives* fastfat, not fastfat itself → points back at the kernel-side change.

**Getting the caller chain when the panic backtrace is "0 frames":** the Rust panic → bugcheck
path loses the stack (GDB `bt` at KeBugCheckEx unwinds only into the panic runtime). Instead,
**instrument the asserting function to walk the arm64 frame-pointer chain itself**: capture the
caller with `asm!("mov {}, x30")` as the *first* statement (before any `bl` clobbers lr), then
at the failing branch walk `x29`: `[x29]=saved fp`, `[x29+8]=return addr`, print N frames via
`DbgPrint`. Symbolize each `ret` with `llvm-symbolizer --obj=fastfat.sys (ImageBase+offset)`,
where `offset = ret − module_base` and `module_base = (bugcheck P1 absolute) − (P1 "+0xNNNN")`.
Module bases are stable across boots here (no KASLR), so you can reuse them.

**Fix:** route the assert through `NT_ASSERT((cond) as i32)` like the rest of the file — 13-line
diff, faithful to ReactOS, keeps the correct ob-rust IRP_MJ_CLEANUP fix. arm64 smoke green ×2.

**Owed sweep (same class) — DONE 2026-07-12.** Swept all 26 sibling hard `assert!(` sites in
fastfat-rust, cross-checked each against the C: **every one** mirrored a debug-only C
`NT_ASSERT`/`ASSERT` (allocsup 14, dirsup 6, create 2, resrcsup 2, fsctrl 1 — verified against
allocsup.c/dirsup.c/create.c/fsctrl.c/resrcsup.c). Converted 25 → `debug_assert!` (no-op in our
`--release` kernel: `[profile.release]` leaves `debug-assertions` off, so this matches ReactOS
free-build `ASSERT` semantics *exactly* and still fires in a debug kernel). Two were NOT dormant
panics and were left alone: `fileinfo.rs:1071` (already inside `if cfg!(debug_assertions)` — the C
wraps both the assert *and* the `PinCount -= 1` in `DbgDoit`, so the guarded block is faithful) and
`allocsup.rs:1807` (a `// ...could add assert!()` comment). One latent condition-mistranslation
noted-and-neutralized in passing: `create.rs` rendered `NT_ASSERT(i != 0)` as
`if i == 0 { assert!(false, "i != 0") }` — correct *semantics* (fires iff i==0), now `debug_assert!`
so it's a no-op in release regardless. **General rule for LLM C→Rust:** `ASSERT`/`NT_ASSERT`/`ASSERTMSG`
must translate to `debug_assert!` (or the no-op shim), never a hard `assert!` — a debug invariant is
not a runtime guarantee, and turning one into a panic plants a mine that a *future correctness fix*
will step on. **Detector idea (owed):** a lint that flags `assert!(` in no-std kernel crates whose
line traces to a C `ASSERT`/`NT_ASSERT` — the class is invisible to every existing gate until the
path goes hot.

### 9.58 (WoW64/FEX) Two window-path 32→64 divergences: (a) FIXED — cursor/icon UNICODE_STRING probe → 0x1E; (b) OPEN — the shared USER handle table has no 32-bit view, so client-side `ValidateHwnd` fails and `DispatchMessage` silently drops WM_PAINT

Context: the x86-on-arm64 WoW64 lane (M8 landed a real 32-bit *window* on the arm64
desktop, callbacks + pump working). M9's goal is WM_PAINT reaching the guest wndproc
so the app paints its client rect. Two bugs surfaced, both the same *class* (a win32k
syscall or shared struct laid out for 64-bit, consumed by a 32-bit guest), at two
different layers.

**(a) FIXED — cursor/icon load path, align-8 UNICODE_STRING probe → BUGCHECK 0x1E.**
`win32.exe` registers its class with `hCursor=NULL`, so window creation triggers
`LoadCursor(NULL, IDC_ARROW)`: user32 decodes the arrow bitmap from its own resources
(a run of `NtGdiStretchDIBitsInternal` on serial), then calls `NtUserSetCursorIconData`
(and `NtUserFindExistingCursorIcon`) with the module + resource names as `UNICODE_STRING`s.
Those were **passthrough** win32k thunks, so the guest's 32-bit `UNICODE_STRING`
(8 bytes, *4-aligned*) reached win32k's `ProbeAndCaptureUnicodeString` →
`ProbeForRead(ptr, 16, 8)` (a 64-bit `UNICODE_STRING` is 16 bytes / `TYPE_ALIGNMENT`=8).
The guest pointer being only 4-aligned raised `STATUS_DATATYPE_MISALIGNMENT`, which under
DUMMY_PSEH bugchecked `0x1E`. **The probe signature — `Len=16 Align=8` on a low (guest)
address — is the fingerprint of a raw-passthrough 64-bit `UNICODE_STRING` probe of a
WoW64 buffer.** Fix: manual marshallers (`dll/win32/wow64/wow64_manual.c`) that widen
both `UNICODE_STRING`s 8→16 and repack `CURSORDATA` 0x58→0x88 (pointer/HBITMAP fields
widen; the animated `CURSORF_ACON` frame arrays are owed). Added both to
`tools/wow64gen/manual-win32k.txt`. Post-fix: win32.exe creates the window and receives
its **full** message set through the callback bridge (WM_NCCREATE/WM_NCCALCSIZE/WM_CREATE/
WM_SHOWWINDOW/WM_NCPAINT/WM_ERASEBKGND/WM_ACTIVATE/WM_SIZE/WM_MOVE/...).

*Debug recipe that named the faulting syscall:* the win32k dispatch trace only prints
on syscall *return* (`[WOW64] Name => status`), so a thunk that faults the kernel prints
nothing. Add an **on-entry** trace (`[WOW64>] Name enter ...` in
`Wow64DispatchWin32kService`, gated on `Wow64TraceSyscalls`) — the last `[WOW64>]` with
no matching `=>` is the culprit, and its args point at the offending buffer.

**(b) OPEN — WM_PAINT never reaches the guest wndproc; root = no 32-bit view of the
shared USER handle table.** After (a), the pump calls `PeekMessageA` which returns
`WM_PAINT (0x000f)` with the correct hwnd **200×** (win32k's paint path is correct), but
the guest never dispatches it: no `NtUserDispatchMessage`, no `NtUserBeginPaint`, and the
window stays dirty so QS_PAINT re-fires forever (exit `0x600E` WARN). Root cause is
**client-side** `ValidateHwnd`: `user32!DispatchMessageA` calls `ValidateHwnd(hwnd)` first
and `return 0` silently if it's NULL (no syscall — matching the trace). `ValidateHwnd` →
`GetUser32Handle` indexes the **shared USER handle table** via `gSharedInfo.aheList`
(`gHandleTable->handles`, `->nb_handles`, then `gHandleEntries[Index]`). The shared heap
IS mapped below 4GB for WoW64 (`W32Wow64ZeroBits`; the marshalled USERCONNECT shows
`ahe=001da000 delta=fc240000`, all <4GB), **but the table itself is written by 64-bit
win32k in 64-bit layout**: `USER_HANDLE_TABLE` is 24 bytes on Win64 (`nb_handles` at
offset 16) vs 16 on Win32 (offset 8), and `USER_HANDLE_ENTRY` is 24 vs 12 bytes. The
32-bit guest reads `->handles` as the low 32 bits of a 64-bit kernel VA, misreads
`->nb_handles`, and strides the entry array by 12 not 24 — so every handle validation
fails. It's invisible until now because CreateWindow takes the hwnd straight from the
syscall return and the kernel→user callbacks pass the wndproc pointer directly — neither
uses `ValidateHwnd`; the message loop is the *first* client-side validation.
**This is a subsystem, not a marshaller:** WoW64 GUI needs a per-process **32-bit view of
the USER handle table** (real Windows keeps a parallel 32-bit `gSharedInfo`). The table is
session-global and can't be per-bitness in place, so win32k must maintain a 32-bit shadow
(header + entries) for WoW64 processes and hand its low VA back in `USERCONNECT.aheList`,
kept in sync on every handle alloc/free. Until then any WoW64 GUI app that validates an
HWND client-side (i.e. nearly all of them) silently no-ops the call.

**FIXED (2026-07-12, M9).** The first instinct — have win32k maintain a 32-bit *shadow*
of the handle table and publish it as `aheList` — is the **wrong layer**, and trying it is
instructive: with the shadow published, `ValidateHwnd` stops no-oping and **finds** the
entry, but the guest then dereferences the *validated object* and AVs (`0xfc6daab0`, in
guest `user32+0x46141`) during CreateWindow's WM_GETMINMAXINFO. Because the handle table is
only **one** of the things user32 reads straight out of shared memory with no syscall. The
full set is three:

  1. the shared **USER handle table** (`gSharedInfo.aheList`) — what `ValidateHwnd` indexes,
  2. the desktop heap's **WND / CLS** objects, reached via `DesktopPtrToUser()`,
  3. **DESKTOPINFO** (`pvDesktopBase`/`pvDesktopLimit`/`fsHooks`/`spwnd`).

Shadowing only (1) fixes validation and then walks straight into (2). All three are already
*addressable* by the guest — win32k maps them below 4GB for WoW64 procs (`W32Wow64ZeroBits`)
— they are simply **unparseable**, laid out by a 64-bit kernel with 8-byte pointers. So this
is a **layout** problem, not a mapping problem, and the fix belongs **client-side**, exactly
like the M6/M7 GDI fix (`g_GdiWow64` + `GDI_TABLE_ENTRY64` + a normalizing read helper):

* **`user32-rust/src/wow64.rs`** (new): `g_UserWow64` (detected once at connect via
  `ProcessWow64Information`, mirroring `GdiProcessSetup`), the `*64` shapes (`Wnd64`,
  `Cls64`, `DesktopInfo64`, `UserHandleTable64`, `UserHandleEntry64`), and a normalizing
  view over each. WND/CLS get cached **snapshots** — sound because the client's desktop-heap
  mapping is `PAGE_READONLY` (`IntMapDesktopView`), so user32 *never writes* through a
  `PWND` and a copy cannot lose an update. Snapshots are keyed by kernel object so `PWND`
  identity stays stable (user32 compares them) and refreshed per lookup so reads are current.
* Typed `desktop_wnd_to_user()` / `desktop_cls_to_user()` replace the ~25 raw
  `DesktopPtrToUser(x->spwnd*/pcls)` call sites. Pointer fields *inside* a snapshot keep the
  kernel value truncated to 32 bits (exactly as the native i386 path stores them), so the
  caller's next translation works unchanged — lossless below 4GB, and the delta arithmetic
  wraps mod 2^32.
* **Kernel half (`win32k-rust/src/wow64.rs`, new):** win32k writes CLIENTINFO into the
  **64-bit** TEB; a WoW64 guest reads it from **TEB32** (`GetWin32ClientInfo()` =
  `NtCurrentTeb()->Win32ClientInfo`, and under WoW64 `NtCurrentTeb()` *is* the TEB32). It
  was therefore **all zeros**: `ulClientDelta == 0`, `pDeskInfo == NULL` — which is why the
  guest computed a raw kernel pointer. `Wow64PublishClientInfo()` projects the 64-bit
  CLIENTINFO into a `ClientInfo32` in TEB32 (TEB64+0x2000; see `procsup.rs`) at thread init
  and desktop assignment. `CallbackWnd` is deliberately left **zero** — it caches a 64-bit
  `WND*` that is useless to the guest; zeroing it makes `ValidateHwnd`'s fast path miss and
  fall through to the (correct) normalizing lookup.

Result: `win32.exe` `0x600E → 0x600D` — the x86 guest's window receives WM_PAINT and paints
its client rect through the native arm64 win32k. `gui32` stays `0x600D`.

**Two reusable lessons.**

*(i) The two halves must ship together, and the failure in between is loud.* Landing the
kernel half alone (CLIENTINFO32 published) against a **stale** guest user32 turned `0x600E`
into `0xC000001D` — illegal instruction. That is not a mystery, it is the expected shape: a
non-zero `ulClientDelta` makes the *unmodified* client start actually translating pointers,
and it then walks 64-bit structs with 32-bit offsets straight into a garbage `lpfnWndProc`.
If you see a WoW64 guest go from "quietly does nothing" to "wild jump" after a win32k
change, suspect a client/kernel version skew before suspecting the kernel.
**Corollary (cost me a boot cycle):** the WoW64 guest's `user32.dll`/`gdi32.dll` are staged
from **`build-i386/`**, not from `build-arm64/` — an arm64-only rebuild silently ships a
stale client. Always rebuild **both** arches before booting a WoW64 change.

*(ii) Make the layout assumption a compile-time tripwire, not a comment.* The `*64` shapes
work because on i686-w64-mingw (MS ABI) a `u64` has 8-byte struct alignment exactly as on
x86_64 — verified: all 91 `Wnd64` fields land at identical offsets on both targets. But the
shapes must also *stay* equal to the real structs. `user32-rust` builds for i386 **and**
natively for amd64/arm64, so the `#[cfg(not(target_arch = "x86"))] mod layout_guard` asserts
`size_of::<Wnd64>() == size_of::<WND>()` + per-field `offset_of!` equality on every native
build. It earned its keep immediately: it caught a bogus padding word in `Cls64` on the
first compile. Any future drift in `ntuser.h` now breaks the amd64/arm64 build instead of
silently corrupting WoW64 clients.

**Still owed (loud, not silent):** `MENU`/`ITEM`, `HOOK` and `SBINFO` are desktop-heap
objects with pointers too, and have no `*64` shape yet — nothing on the M9 path reaches
them. `crate::wow64::wow64_unsupported()` prints a one-shot warning naming the type rather
than letting a 32-bit client silently walk a 64-bit `MENU`. When a WoW64 app needs menus,
that print is the work order.

### 9.59 (ROOT-CAUSED + FIXED 2026-07-12) The DUMMY_PSEH **capture-copy** class: every `_SEH2_TRY`-wrapped user-buffer capture became an unprotected `memcpy` — and how to get a *real* fault-tolerant copy out of clang's "calls-only" native SEH

**Symptom.** Several unrelated syscalls BUGCHECK 0x1E (0xC0000005) at the *identical* PC,
`memcpy+0x40`, whenever a caller passes a hostile buffer pointer (NULL, 0x42424242).
Observed via kmtest conformance: `ObSymbolicLink`, `ZwWaitForMultipleObjects`, `IoReadWrite`
(and `NtCreateSection` with a 0x50 face — same root, scalar deref instead of a bulk copy).

**Root (a whole CLASS, not three bugs).** In C, a kernel API captures a caller-supplied
buffer with a copy sitting inside `_SEH2_TRY`. Under `USE_DUMMY_PSEH` — still the default C
compile mode on **all** arches — that handler is dead code, and the Rust translations dropped
the SEH entirely. Two compounding gaps:
1. The `probe_user_buffer` guard is `PreviousMode`-gated, but the C's `_SEH2_TRY` wrapped the
   copy **unconditionally**. Kernel-mode callers (kmtest drivers, and any in-kernel client) skip
   the guard and hit a raw, unprotected copy.
2. A probe is a **range + alignment check only** — it never touches memory. NULL and 0x42424242
   are both *below* `MmUserProbeAddress`, so a range probe happily passes them. The safety was
   ALWAYS the SEH around the copy, never the probe.

Critically, `RtlpSafeCopyMemory` — the one primitive that was supposed to be the safe copy — had
been translated to a **plain `RtlCopyMemory`**. So the kernel had *no* fault-tolerant copy at all.

**The fix, and the non-obvious insight that makes it work.** `dummy-pseh-verdict.md` §C warns
that clang's native SEH is *"calls-only (no `-fasync-exceptions`)"* — a **direct deref** written
inside a `__try` body may not be covered. That reads like a blocker. It is actually the *recipe*:

> **A copy is a CALL.** Put the faulting access in a CALLEE and the `__try`'s scope table covers
> the call site, because the fault's return address lands back inside the `__try` range.

So `ntoskrnl/kmsafecopy.c` (native `__try`/`__except` via the `-fms-extensions` `c_shims` wiring,
same as `seh_poc.c`) does:

```c
__attribute__((noinline))              /* load-bearing: guarantees the CALL exists */
static void KmCopyRaw(void *D, const void *S, KMSIZE N) { memcpy(D, S, N); }

int KmProtectedCopyMemory(void *D, const void *S, KMSIZE N)
{ __try { KmCopyRaw(D, S, N); return 1; } __except (1) { return 0; } }
```

Chain: `KiPageFault` → `KiDispatchException` → `RtlDispatchException` unwinds `KmCopyRaw`'s frame
→ `__C_specific_handler` matches the scope table → `__except` → return 0 →
`Err(STATUS_ACCESS_VIOLATION)`. **`seh_poc.c` had only ever proven a *software raise* dispatches;
this proves the same chain catches a *hardware page fault*.**

Bonus, and the reason this beats the obvious alternative: the fault goes through `MmAccessFault`
**first**, so a valid-but-nonresident page is simply paged in and the copy proceeds. Only a truly
invalid address reaches the `__except`.

**Do NOT "fix" this with `MmIsAddressValid`.** It is the tempting portable stand-in and it is
wrong: it cannot distinguish *invalid* from *valid-but-paged-out*. Drop it on a hot path (e.g.
`IopCompleteRequest`'s buffered copy-back) and legitimate I/O starts failing with
STATUS_ACCESS_VIOLATION under memory pressure — a silent, subtly-wrong degradation far worse than
the crash it papers over. Prefer the crash + an honest gap.

**Two traps this flushed out:**
- **PERF: verify the copy is a real `memcpy`.** A hand-rolled `while (Len--) *Dst++ = *Src++;`
  compiled to a genuine byte-at-a-time loop — clang's loop-idiom pass did **not** lower it. On the
  buffered-I/O completion path that is a throughput regression. *Disassemble the shim* (`callq
  <memcpy>`) rather than assuming the optimizer saved you.
- **A fixed crash can UNMASK a second bug at the same test.** After the copy fix, `IoReadWrite`
  still crashed — but the PC had moved out of ntoskrnl to `IoReadWrite_drv.sys`. **Fast I/O hands
  the driver the RAW user buffer** (no SystemBuffer), and in C the fast-io dispatch is itself
  inside `_SEH2_TRY`. That is the same family needing a *different* primitive: a protected
  **call**, not a protected **copy**. Always re-symbolicate after a fix — "still crashes" is not
  "not fixed".

**Where it lives.** Primitive: `ntoskrnl/kmsafecopy.c` + `ntkrnl-sys/src/probe.rs`
(`safe_copy_from_user` / `safe_copy_to_user` / `safe_copy_raw`, all running the protected copy
UNCONDITIONALLY; only the range probe stays `PreviousMode`-gated). **amd64 only** — i386/arm64
fall through to a plain copy until clang's frame-based i386 / aarch64 kernel SEH gets the
`seh_poc.c` treatment, or an extable-style fault-fixup copy exists.

**Detector.** A `#[no_mangle]` syscall that does `copy_nonoverlapping` / `read_volatile` on a
caller pointer where the C had `_SEH2_TRY`. Grep the inert C for `_SEH2_TRY` around
`RtlCopyMemory`/`Probe*` and diff against the Rust port — if the Rust has the copy but no
`safe_copy_*`, it is a latent 0x1E waiting for a hostile caller.
kept in sync on every handle alloc/free. Owed as the M9 core (`wow64-plan.md`). Until then
any WoW64 GUI app that validates an HWND client-side (i.e. nearly all of them) silently
no-ops the call.

### 9.60 (2026-07-12) A differential harness can only falsify what it VARIES — two blind spots that hand you a green MATCH on code that is wrong in production

**Context.** The msvcrt "assemble the CRT from pieces" lane flips C CRT bodies to Rust one family at a time, gated by `msvcrt-difftest`: build.rs hands the **genuine** `dll/win32/msvcrt/string.c` to `cc` as the translation unit (106 symbols renamed `ref_*` via a generated `-include` prelude), and a fuzz driver compares the Rust against it, printing MATCH / SIGN-ONLY / DIVERGE per function. It works — it caught a real signed-vs-unsigned-char corruption before it shipped (below). But twice it would have cheerfully certified a function that is **wrong on the real target**, and both times the only thing that saved it was reading the C.

**The generalizable rule: a fuzz harness proves parity only across the axes it actually varies. Every axis it holds CONSTANT is a silent assumption baked into every MATCH it prints.** Before believing a verdict, enumerate what the reference TU has *pinned*.

**Blind spot 1 — ambient state the reference holds fixed (here: locale).**
`_stricmp` / `_strnicmp` / `_memicmp` looked like pure ASCII compares. They are not: msvcrt's bodies consult `get_locinfo()->lc_handle[LC_CTYPE]` and fold through `_tolower_l`. The harness's `refimpl.c` stubs the locale to "C" (`lc_handle[]` zeroed) — precisely the branch where a locale-blind ASCII fold coincides. So once the *unrelated* signedness bug was fixed, all three flipped to MATCH. Flipping them on that evidence would have been correct under the default locale and wrong the moment an app calls `setlocale()`. They were deferred to the locale phase instead, and the harness now prints them as an explicit DEFERRED footer so they can't be quietly forgotten.
> **Detector:** grep any flip candidate's C body for `get_locinfo` / `_l`-suffixed helpers / any read of process-global state. If it reaches ambient state the harness stubs, the harness cannot grade it.

**Blind spot 2 — the reference TU is compiled for the HOST, so target-conditional `#if`s are graded on the wrong arm.**
The reference is built for x86_64 **linux**, where `_WIN64` is undefined. So *every* `#if defined(_WIN64)` in the C is silently graded on its 32-bit arm. `strncmp` has exactly such a branch, and with msvcrt's `_MSVCR_VER=0` it reduces to plain `_WIN64` — meaning **the same function ships two different contracts**: i386 returns the raw unsigned difference, amd64/arm64 clamp to -1/0/1. A MATCH from the harness certifies only i386. (The tell was visible in the output and easy to miss: `msvcrt=-17` — an unclamped value.)
> **Detector:** before trusting a MATCH, `grep -nE '_WIN64|_UCRT|_MSVCR_VER'` the function. `_MSVCR_VER` is safe *if* build.rs defines it exactly as the real build does (here: `0`); host-vs-target macros like `_WIN64` are not.
> **Fix without a second reference compile:** if one `#if` arm is a pure function of the other's value, model it. Both `strncmp` arms run after an *identical* loop and then compare the *same two bytes*, so `_WIN64 == sign(#else)` — hence `clamp(ref_strncmp(..))` is an **exact** model of the 64-bit branch, and one host TU grades both shipped contracts.

**Blind spot 3 (the one that bites hardest) — grading a PROXY instead of the artifact.**
The harness originally linked `cnt-rust` (the kernel CRT crate whose cores the wrappers call). That was sound only while the wrappers were *verbatim copies* of cnt-rust's `cabi` shims. The moment a wrapper legitimately **adapts** the core (unsigned contract, clamped return, a msvcrt-specific port), the thing being graded is no longer the thing being shipped. Link **the shipping crate**.
> Two hard runtime gates keep this honest, and both earned their keep:
> - `strlen(NULL) == 0` — proves the shared libc names resolved to the Rust shims and not **glibc**. (Without a `extern crate` force-link, rustc drops the unused rlib and `strlen`/`strcmp`/`memcpy` bind to glibc; the harness then grades *glibc against msvcrt* and prints a meaningless MATCH.)
> - `_strnset(s,c,2) == s+2` — proves it's the **msvcrt wrapper** and not cnt-rust's cabi shim, whose contract returns the *start*. A contract difference between the two crates makes a perfect canary for "am I linked to the right one".

**The bug the harness DID catch (the reason to keep it).** cnt-rust's `core_rs` is faithful to libcntpr, which uses a **signed-char** convention (`Elem::diff` for `u8` = `(a as i8) - (b as i8)`; `mem::chr` matches `(b as i8 as i32) == c`). msvcrt/Wine uses **unsigned-char**. They agree on all of ASCII and **flip sign on any byte ≥ 0x80** — so `strcmp`/`strncmp`/`memchr`/`_memicmp`/`_memccpy` would have silently corrupted ordering and search on non-ASCII data (UTF-8 paths, accented filenames, binary buffers) while booting perfectly green. **Two CRT bodies in one tree can want genuinely different answers from the same-named function; that is a contract knob, not a bug** — resolved by adding `Elem::udiff` (default = `diff`; only `u8` overrides, since `wchar_t` is unsigned by definition and the wide family never had the bug) with each contract pinned by its own harness (`cnt-difftest` for the kernel, `msvcrt-difftest` for msvcrt).

**Corollary for cohab flips generally: VERIFY THE FLIP TOOK.** A green build and a "duplicate symbol" warning prove nothing about *which* body won. Parse the PE export table and resolve each export's RVA into the Rust `--whole-archive` cluster vs the C bodies — and assert the *deliberately-not-flipped* ones still land in C. (Plain link-order is NOT enough: the C monolith object still gets pulled in for its un-flipped symbols and drags its own copies of the flipped ones along, winning first-match. `--whole-archive` on the Rust `.a` is what forces the Rust definitions in first.)

### 9.61 (ROOT-CAUSED + FIXED 2026-07-12) The bug a value-comparing fuzz CANNOT see: an unbounded scan inside a bounded (`n`-suffixed) CRT function — right answer, wrong memory

**Symptom.** msvcrt wave-3 flipped 6 functions, all reported MATCH by the differential harness across 20,000 fuzz cases each, all 3 arch smokes green to desktop. But the **net suite** regressed hard: `nettest.exe` (the usermode winsock proof) printed `WSAStartup OK` and then went silent — no crash banner, no error, just gone — and the kernel's `[NETSTACK] dns OK` marker slipped past the boot snapshot. 4/4 fail. Control (previous commit, same host): 15/15 pass, twice. So: a real regression, not the documented NET_FLAKE.

**The trap.** Everything about the shape said "not my code": the failing tests were *kernel-side* netstack self-tests, and msvcrt.dll is **user-mode**. The functions were harness-exact against the genuine C. It was tempting to call it host contention (two other agents' QEMUs were live) and re-gate. **The A/B control is what forbade that** — build the previous commit, run the same suite on the same host: 15/15. Never accept "probably a flake" when a two-run control can settle it.

**Bisection** (rebuild + boot per step; the wrappers toggled one family at a time):

| variant | net |
|---|---|
| all 6 wrappers off (cnt-rust change + crate split still in) | 15 pass |
| `+strcmp +strncmp +memcmp` | 15 pass |
| `+memchr` | 15 pass |
| `+_memccpy` | 15 pass |
| `+_strnset` | **13/2 FAIL** |

**Root cause.** The Rust `_strnset` was:
```rust
let take = core::cmp::min(h::cstr_slice(p).len(), len);   // UNBOUNDED scan, clamped AFTER
```
against the C's
```c
if (len > 0 && str) while (*str && len--) *str++ = value;  /* reads at most len bytes */
```
`cstr_slice` is a `strlen`-style scan with **no bound**. On a buffer that an `n`-suffixed API is explicitly allowed to leave **unterminated**, it runs off the end. The `min` still bounds the *fill*, so **every byte written and every value returned is identical to the C** — which is precisely why 20k fuzz cases said MATCH: the harness's `gen_str` always NUL-terminates, so the scan always found a terminator inside the buffer. The only damage is the **read**, and it only kills you when it walks into an unmapped page.

**Why it reached the network stack (and why "nobody calls it" was wrong).** A first grep for callers found none — but the grep was truncated by `head`. `_strnset` IS called, from *inside msvcrt itself*: `mbcs.c`'s `u__strnset()` → `_mbsnset` / `_mbsnbset`. A silent fault there took the process down with no banner. **When a bisect says a symbol matters, believe the bisect over your grep — and re-run the grep without `head`.**

**The generalizable rule.**
> **An `n`-suffixed CRT function must never contain an unbounded scan.** Bound the scan by `n` (`cstr_slice_bounded`) — never scan-then-`min`. The clamp makes the *answer* right while the *access* is already out of bounds.

**The detector (new, permanent): `msvcrt-difftest/tests/oob_guard.rs`.** A value-comparing fuzz is *structurally* blind to an over-read that still returns the right answer — no amount of fuzzing finds this class. So probe the access, not the value: `mmap` two pages, `mprotect(PROT_NONE)` the second, and place a deliberately **NUL-free** buffer of `k` bytes flush against the boundary; call each bounded fn with `n <= k`. A correct impl never touches `p[k]`; an unbounded scan takes SIGSEGV. Each call runs in a forked child so the offender is *named* instead of taking the harness down. Validated both ways — it reports `_strnset ✗ OVER-READ` on the buggy version and passes on the fix. **Every new bounded flip gets a row here.**

**Meta-lesson (compounds with §9.60).** §9.60 catalogued the harness's blind spots in *what it varies* (locale, host-vs-target `#if`, proxy-vs-artifact). This is a blind spot in *what it observes*: it compares return values and destination bytes, so it can only ever catch wrong **answers** — never wrong **memory access** that yields a right answer. Two different questions, two different instruments. A green diff table is not a safety proof.
### 9.62 (ROOT-CAUSED + FIXED 2026-07-12) WoW64: a code *generator* silently mis-thunked the whole file-I/O family — `IO_STATUS_BLOCK` is 8 bytes in the guest and 16 on the host, and `NtReadFile` was never on the deep-marshal list

**Symptom (latent — nothing was visibly broken).** `hello32`/`gui32`/`win32` all passed;
the boot was green. But every `ReadFile` a 32-bit guest could ever issue was returning a
garbage byte count, and overflowing the guest's stack by 8 bytes while doing it. It had
simply never been *exercised*: no guest did file I/O yet.

**Root cause.** `tools/wow64gen/` generates 32→64 syscall thunks from `ntdll.spec`. A
`.spec` describes a pointer's *value*, not the shape of what it points at, so the generator
takes a hand-curated deep-marshal list (`manual.txt`) of syscalls whose pointed-to structs
differ in width; anything not on that list gets an **auto** thunk that passes the guest
pointer straight through. `IO_STATUS_BLOCK` ends in `ULONG_PTR Information` — **4 bytes on
i386, 8 on the host** — so the guest's block is 8 bytes and the host kernel writes 16.
`NtReadFile`, `NtWriteFile`, `NtQueryInformationFile`, `NtSetInformationFile`,
`NtQueryVolumeInformationFile` and `NtFlushBuffersFile` were **not on the list**. Two bugs
per call: an 8-byte overflow past the guest struct, and `Information` read from offset 4
when the kernel wrote it at offset 8. `wow64gen`'s own README names `IO_STATUS_BLOCK` as a
deep-marshal case — the *doc* was right and the *list* was incomplete.

**The reusable lesson (this is the point).** A generator that is safe-by-default only where
a human remembered to list something is **not safe by default**. The failure is invisible:
the build is green, the tool is "working as designed," and the wrongness only shows up as
corrupted data in a caller nobody has written yet. When a codegen tool has an escape hatch
list, the list needs a **periodic audit against the property it encodes** — here: "grep the
spec for every syscall taking a struct with a `ULONG_PTR`/pointer/HANDLE member, and diff
that against `manual.txt`." Missing entries are silent; extra entries are loud. **Always
err toward listing.**

**Fix** (`dll/win32/wow64/wow64_manual.c`, M10): hand-marshal the six + `NtQueryDirectoryFile`
(which also takes a `PUNICODE_STRING` mask). Every remaining IOSB taker (`NtLockFile`,
`NtQueryEaFile`, `NtCancelIoFile`, `NtNotifyChangeKey`, `NtDeviceIoControlFile`, …) went on
the manual list too, so they now fail **loud** instead of corrupting silently.

**Async is the trap, and it's the §9.36 class.** The host IOSB we marshal into is a *stack
local of the thunk*. Returning `STATUS_PENDING` hands the kernel a pointer into a frame that
is about to die — exactly the dead-frame-IOSB corruption that cost days in §9.36. So an
explicit async request (guest passed an `Event` or APC routine) is **refused before the call
is issued**, and if the host pends anyway we **block on the file handle** rather than return
over a live pointer into a dead frame. Never "just copy it back later" — there is no later.

**The proper async fix is kernel-side and is owed:** I/O completion should write an
`IO_STATUS_BLOCK32` when the requesting process is WoW64 (NT gates exactly this on
`IoIs32bitProcess(Irp)`); our single choke point is `*(*Irp).UserIosb = (*Irp).IoStatus;` in
`iomgr-rust/src/irp.rs`. Careful: 64-bit code (wow64.dll, the 64-bit ntdll) *also* runs
inside a WoW64 process and passes genuine 64-bit IOSBs, so the gate must be **per-IRP,
captured at syscall entry** — never merely "this process is WoW64."

**Regression oracle:** `file32.exe` (`tools/fex/hello-compute/File32.c`, spawned by
winlogon's `TryStartWow64File`). `ReadFile`'s returned byte count *is* `IOSB.Information`,
so asserting it catches this class by construction. Verdict 0x600D; `[file32]` lines on
serial.




### 9.63 (ROOT-CAUSED + FIXED 2026-07-12) A slice-based core cannot express a single-pass C string primitive — the CRT perf tax, and the gate that was blind to it

**Symptom.** A Rust CRT flip is byte-for-byte correct (a 20k-case differential fuzz
says MATCH on every input) and 2–5× slower than the C it replaced. Nothing in the
build, the lints, or the value-comparing harness says a word.

**Root cause — structural, not a typo.** The idiomatic way to wrap a safe Rust core
is to hand it slices:

```rust
pub unsafe extern "C" fn strcmp(a: *const u8, b: *const u8) -> i32 {
    clamp(cmp_u(cstr_slice(a), cstr_slice(b)))     // 3 passes
}
```

`cstr_slice` must find the NUL to build a `&[u8]`. So this scans `a`, scans `b`,
and only then walks them both to compare. The C is **one** pass and never computes
a length at all:

```c
while (*str1 && *str1 == *str2) { str1++; str2++; }
```

**The rule: a `&[T]` requires a length, and computing the length is exactly the scan
the C skips.** Any C string primitive whose loop *discovers* the terminator as it
works — `strcmp`, `strcpy`, `strcat`, `strchr`, `strstr`, `strspn`, and every wide
twin — pays an extra full pass per input when routed through a slice API. Measured
on msvcrt (2026-07-12):

| flip | slice form | single-pass form | why |
|---|---|---|---|
| `wcscmp` | **3.8–4.8×** | 0.83× | 2 length scans + 1 compare walk vs 1 walk |
| `strchr` | **2.01×** | 1.00× | scan-to-NUL then search vs one do-while |
| `strcat` | **2.5×** | within bar | scan dst + scan src + copy vs scan dst + copy |
| `strcmp` | *(unmeasured — see below)* | 1.00× | same 3-pass shape, shipped on master |

**Fix.** Transcribe the copy/compare/search families directly against their C bodies
as single-pass raw-pointer loops, in the *wrapper* crate. This is not a retreat from
the "assemble from cnt-rust" strategy — it is the same call the lane already made for
`memcmp` (hand-written word-wise, because cnt's byte-zip `mem::cmp` measured 10.3× off)
and `_memccpy` (msvcrt's algorithm genuinely differs). Where the core's *shape* is
wrong for the contract, the wrapper carries the body.

Narrow set-based searchers (`strspn`/`strcspn`/`strpbrk`) can go one better with a
256-bit `ByteSet` — which is precisely what msvcrt's own `strcspn` does (`BOOL
rejects[256]` + `memset`). The wide twins stay naive nested scans, because their C
is a naive nested scan (the u16 space is not cheaply tabulated, and msvcrt does not
try).

**The meta-lesson — a perf gate only defends what it BENCHES.** The msvcrt perf gate
existed (it was built *because* wave 2 shipped an unvectorizable `memset` and blew the
net suite's timeouts), and it was green. But it benched only
`memset/memcpy/memcmp/memchr/strlen`. `strcmp` — the same 3-pass shape, served to
every caller in the system — was never in the list, so it went slow through two waves
and was found only because its *wide twin* happened to be benched. When you add a
throughput gate, enumerate the flips and bench **all** of the hot ones; a gate with a
coverage hole reads as "fast" exactly like a gate with none.

**Two gate-hygiene corollaries found the same day:**
- **Bench best-of-3, not the mean.** This host runs other agents' builds and QEMUs. A
  scheduler steal inflates a sample, and under a mean that lands as a phantom
  regression at the small sizes where call overhead already dominates. The minimum
  time is the least-contended sample — the one that reflects the code, not the host.
- **A gate that cries wolf in the default profile is a gate people learn to ignore.**
  `opt-level = 3` for the CRT lives in `[profile.release.package.*]`, so a plain
  `cargo test` compared an *unoptimized* Rust body against `-O2` C and reported ~100×
  "regressions" that do not exist. Make it skip, loudly, with the reason and the right
  command — do not let a bogus red become the normal state.

**Related:** §9.60 (a differential harness can only falsify what it VARIES),
§9.61 (an unbounded scan inside a bounded `n`-suffixed CRT function). Together these
are the three orthogonal blind spots of a CRT flip: wrong **values** (`classify`),
right answer / wrong **memory** (`oob_guard`), right answer / too **slowly** (`perf`).
Each is blind to what the others catch.


### 9.64 (ROOT-CAUSED + FIXED 2026-07-12) A refactor can silently drop a *tolerance* the old code provided — NULL stopped behaving as `""`, and no value-comparing test could ever see it

**Symptom.** A pure-refactor of a CRT primitive (same contract, faster body) passes a
20,000-case differential fuzz at 45/45 MATCH, passes the OOB guard, passes the perf
gate — and then the OS boots to desktop on all three arches while the **net suite**
goes 15/15 → 13/15, with a user-mode winsock round-trip that never completes.

**Root cause.** The old wrappers reached their input through a helper that
**NULL-guards**:

```rust
unsafe fn cstr_slice<'a>(p: *const u8) -> &'a [u8] {
    if p.is_null() { return &[]; }      // <-- NULL behaves as ""
    ...
}
```

So the *shipped, gate-green* behaviour of `strcmp`/`strcpy`/`strcat`/`strspn`/… had
always been **"NULL is an empty string"** — deliberately more forgiving than the C,
which dereferences and dies. The single-pass perf rewrite (§9.63) walks raw pointers:

```rust
let (mut p, mut q) = (a, b);            // <-- NULL now faults
while *p != 0 && *p == *q { ... }
```

The contract for every *real* string is byte-identical. The only thing that changed is
an input the harness never generates.

**Why nothing caught it.** `gen_str()` produces real NUL-terminated buffers. It has no
notion of nullness, so the fuzz explored 20,000 points in a space that does not contain
the bug. This is §9.60's rule biting again in a new dimension — *a differential harness
can only falsify what it VARIES* — and the dimension here is not a value, a length, or a
locale. It is **the validity of the pointer itself**.

Note the asymmetry that makes this class nasty: being *more* forgiving than the C is
harmless and nobody notices. Silently becoming *less* forgiving than the **last shipped
build** is what breaks the boot — and the reference C is no help, because the C faults
too, so there is no `ref_*` to diff against. **The oracle is the previous build, not the
C.**

**Fix.** Restore the tolerance explicitly, and keep the single-pass body:

```rust
static EMPTY_A: [u8; 1] = [0];
#[inline] fn nz(p: *const u8) -> *const u8 {
    if p.is_null() { EMPTY_A.as_ptr() } else { p }
}
// strcmp: let (mut p, mut q) = (nz(a), nz(b));
```

Redirecting NULL to a static NUL reproduces `&[]` semantics **exactly** (empty compares,
copies, and spans identically) for one perfectly-predicted branch — measured cost: none
(`strcmp` 1.01x, `strchr` 1.00x, `strcat` 0.99x after the guards).

**Detector (permanent).** `classify::assert_null_tolerant()` — NULL is now a first-class
asserted input for every flipped reader, narrow and wide. Validated by NEGATIVE CONTROL:
delete one `nz()` and the harness dies with SIGSEGV. Because the genuine msvcrt faults
here, this is graded against **the crate's own contract**, not against a `ref_*`.

**The generalizable rule.**
> When you refactor a wrapper, enumerate the *implicit guarantees of the helper you are
> removing*, not just the contract of the function you are rewriting. A NULL guard, a
> bounds clamp, an alignment fixup, a zero-length early-out — these live in the helper,
> are invisible at the call site, and vanish silently when you inline the helper away.

Same shape as §9.61 (an unbounded scan inside a bounded function): the *values* were
right in both cases. What changed was something orthogonal to value — there, which
memory was touched; here, which inputs are survivable.

**How it was actually found.** Not by the harness. The net suite failed and every surface
signal said "flake" (NET_FLAKE is an auto-retried class; another agent was gating
concurrently; the failing test *set* varied run to run). The thing that settled it was
the **A/B control**: rebuild the previous commit, same host, same suite — 15/15 twice,
against 2-failed 5/5 for the new one. That is the same move that caught the wave-2 perf
regression, and it is the only reliable way to separate "my change" from "this host is
busy."

> **Run the A/B control before you accept a flake label.** It costs one build. The
> alternative is landing a regression that the retry logic will keep excusing.

### 9.65 (ROOT-CAUSED + FIXED 2026-07-12) A Rust `memcpy`/`memset` stub is a SELF-RECURSION trap — and `-Z build-std` had been cloaking a broken one in bootvid for months

**Symptom.** Dropping `-Z build-std` for the builtin triples (stable-toolchain Axis B) made
**both** amd64 and arm64 hang at boot with an EMPTY serial log. amd64 died at kernel entry
(one line past `[FLDR] phase: handoff to kernel`); arm64 got to `[KERNEL] Entering KiIdleLoop`
and never scheduled the Phase-1 thread. No bugcheck, no panic, no output — just a dead VM.
i386 (still on build-std) was unaffected.

**Root.** GDB showed the PC parked on a literal `jmp .` — an infinite self-jump. Walking back
to the module's PE header identified **bootvid.dll**, and the symbol was `memset`. bootvid-rust
defined its own libc mem*:

```rust
#[no_mangle] pub unsafe extern "C" fn memcpy(dst: *mut c_void, src: *const c_void, n: usize)
    -> *mut c_void {
    core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, n);   // ← lowers to
    dst                                                                     //   llvm.memcpy
}                                                                           //   → call memcpy
```

`copy_nonoverlapping` lowers to `llvm.memcpy`, and the backend lowers *that* back into a **call
to `memcpy`** — the function itself. Tail-call it and the entire body becomes `jmp .`. Same for
`write_bytes` → `memset`.

**Why it never fired before.** `-Z build-std-features=compiler-builtins-mem` linked
`compiler_builtins`' own mem* — compiled `#![no_builtins]`, so *its* loops don't get folded —
alongside the broken twins. Under `--allow-multiple-definition` lld took the FIRST definition,
which was compiler_builtins'. The broken code was compiled, shipped inside every bootvid.dll,
and simply **never selected**. Drop build-std → the competitor disappears → the broken twin is
the only definition → the kernel spins forever the instant it draws the boot screen.

**This is a CLOAK bug** — a real defect held harmless by a link-order accident, exactly like the
cdrom `TracePrint` no-op-fn cloak (Axis A). The lesson generalises: *when you remove a redundant
definition, you are not "cleaning up" — you are electing a new winner, and the new winner has
never run.*

**Rule: NEVER hand-write `memcpy`/`memset`/`memmove`/`memcmp` in Rust.** It cannot be done in
plain safe Rust — any byte-copy loop you write instead is re-recognised by LLVM's loop-idiom
pass and folded straight back into a self-call. The correct answers are:
- **Don't define them.** `ntoskrnl.exe` EXPORTS `memcpy`/`memset`/`memmove` on every arch
  (ntoskrnl.spec), so any module with an ntoskrnl import lib just gets them — which is how every
  C driver already works. `memcmp` is exported **only on x86_64**, which is what
  `sdk/lib/crt/libmemcmp.a` is for (reactos-link links it as a link-group fallback on the builtin
  targets; it's emitted LAST so anything that already defines memcmp keeps its own).
- If you genuinely must, use a crate built with `#![no_builtins]` (i.e. `compiler_builtins`).

**Detector (permanent).** `tools/scan_self_recursive.py` — flags any function whose FIRST
instruction jumps to its own entry. Nothing legitimate looks like that. Wired into the lint gate
as `scan_self_recursive` (a binary lint, beside `scan_ud2`), because a source-level lint CANNOT
catch this: the Rust reads perfectly. Only the emitted code tells the truth.

**Debug recipe that cracked it** (reusable for any silent hang):
1. `RUN=$(qemu/start.sh amd64 --gdb)`, wait, then `qemu/inspect.sh $RUN --command bt`.
2. PC symbolicated to nothing and sat *below* the kernel base → it's in another module.
3. `x/16i $pc` → `jmp <self>`, with an empty `ret` fn right after (`rust_eh_personality`).
4. Scan memory backwards from the PC for the `MZ` header, then read the module name out of its
   PE export directory → `bootvid.dll`. (`find-kbase` only knows about ntoskrnl.)

---

### 9.66 (ROOT-CAUSED + FIXED 2026-07-12) Your differential oracle can be MISCOMPILED by the host's own headers — glibc's `__nonnull`/`__attribute_pure__` deleted the reference C's guards

**Class:** differential-harness integrity. Applies to ANY harness that compiles genuine
target C for the host and renames its symbols to compare against a reimplementation.
**Symptom:** the *reference* segfaults, or silently reports the wrong answer, on inputs the
real target handles fine. **Blast radius:** every verdict the harness has ever issued.

#### The setup that causes it

`msvcrt-difftest` compiles the real `dll/win32/msvcrt/string.c` for the host and renames its
public symbols `ref_*` via a force-included prelude (`-include ref_prelude.h`), so the
reference and the Rust under test can be linked into one process and fuzzed against each
other. The rename deliberately lands **before** the system headers, because that makes glibc
declare the renamed symbols and the reference inherits correct prototypes for free:

```c
/* ref_prelude.h, force-included */
#define atoi ref_atoi
/* ...then <stdlib.h> is pulled, and glibc says: */
extern int atoi (const char *__nptr) __THROW __attribute_pure__ __nonnull ((1));
/*            ^^^^ ...which after the rename declares REF_ATOI, with glibc's attributes. */
```

That is the bug. **A declaration carries more than a prototype — it carries attributes**, and
glibc's are promises about *glibc's* implementation, not msvcrt's:

| leaked attribute | glibc promises | msvcrt actually does | consequence |
|---|---|---|---|
| `__nonnull((1))` | arg is never NULL | `atoi` has `if(!str) return 0;` (string.c:1720) | optimizer **DELETES the guard** |
| `__attribute_pure__` | no side effects | `strtol` family **sets errno** | call may be CSE'd or **elided** |

The `__nonnull` one crashes, so you find it. The `pure` one **doesn't crash** — it quietly
lets the compiler drop an errno-setting call whose result is unused, corrupting exactly the
side-channel an errno-comparing oracle is built on. That is the dangerous half.

#### How to detect it (the general recipe)

The C source and the machine code disagree. **Trust the machine code.**

```bash
# the source SAYS there is a NULL guard:
sed -n '1720p' dll/win32/msvcrt/string.c        # ->  if(!str) return 0;
# the emitted reference does NOT have one:
objdump -d --disassemble=ref_atoi <test-bin>    # ->  no `test %rdi,%rdi`; straight to a deref
```

Scoping rule: **the leak lands only on symbols whose names COLLIDE with the host libc.**
`ref_atoi`/`ref_strtol`/`ref_strcpy` are at risk; `ref__strtoi64_l` is not, because glibc
never declares `_strtoi64_l` and so never attaches anything to it. Guards that live inside
msvcrt-only helpers survive; guards in the ABI-shared names get deleted.

#### The fix

**Not** `-fno-delete-null-pointer-checks` — that patches the NULL symptom and leaves `pure`
armed. Instead pull glibc's macro header first and **neuter the attribute macros**, so every
subsequent declaration comes out attribute-free while the prototypes (the thing you actually
wanted) survive:

```c
/* prelude, AFTER the renames and BEFORE any libc header — build.rs "stage 1.5" */
#include <sys/cdefs.h>          /* glibc routes every attribute through these macros */
#undef  __nonnull            
#define __nonnull(params)
#undef  __attribute_pure__   
#define __attribute_pure__
#undef  __attribute_const__  
#define __attribute_const__
#undef  __attr_access        
#define __attr_access(x)
#undef  __wur                
#define __wur
```

#### The tripwire (do this, not just the fix)

`sdk/lib/rust/msvcrt-difftest/tests/ref_integrity.rs` — a test whose subject is **the oracle
itself**. For each reference function whose C source contains a guard, call it in a forked
child with the input the guard exists for, and assert it returns rather than faults. For
`pure`, call it **for its side effect only, discarding the result**, and assert the side
effect still happened — a `pure` call with an unused result is exactly what gets elided.

Negative-control it: back the disarm out and confirm `ref_atoi`/`ref_atol` fault again.

#### The two general lessons

1. **A harness improvement is worth nothing if you only point it at NEW code.** The errno
   oracle arrived in wave 5 and was used only on wave 5's flips. Re-running the *already
   shipped* functions through it convicted three of them (`_atoi64` on a wrapping parser where
   msvcrt saturates; `strtol` with no ERANGE/EINVAL and writing `*endptr` on a rejected base;
   `strtoul` likewise) — all live on master, all previously "MATCH". **When the oracle gets
   stronger, re-run everything through it.**

2. **A green from an unexercised branch is not a green.** `classify` had `gen_str`, a general
   string fuzzer, whose chance of emitting a 19-digit run is ~zero — so in 20,000 cases it
   never once drove a parse past `_I64_MAX`, and the whole saturate/ERANGE arm was untested.
   The graders now take a `require: &["ok","EINVAL","ERANGE","handler"]` list and fail the row
   as **UNEXERCISED CONTRACT** if the reference never produced those outcomes. Coverage is
   printed on every MATCH, so "did the fuzz reach the branch" is answerable at a glance:

   ```
   _ultoa_s   MATCH   [ok=4799 EINVAL=11049 ERANGE=4152 handler-fired=0]
   ```

   (That `handler-fired=0` is itself load-bearing: `_ultoa_s` sets errno by hand and never
   fires the invalid-parameter handler, while its sibling `_ltoa_s` does. On a bad radix the
   two are byte-, return- and errno-identical — the handler bit is the ONLY thing telling them
   apart, and a three-effect oracle calls them the same function.)

#### See also

- §9.60 — a differential harness can only falsify what it VARIES (the same disease, one level up)
- §9.61 — the bug a value-comparing fuzz cannot see (an unbounded scan inside a bounded fn)
- §9.64 — a refactor can silently drop a *tolerance* the old code provided
- `docs/engineering/active-lanes/msvcrt-from-pieces-plan.md` — the lane, and what it has refused to flip
---

### 9.67 (ROOT-CAUSED + FIXED 2026-07-12) WoW64 M11: what a *stock* binary flushes out that a hand-written test guest never can — five bugs, each invisible to every guest we wrote ourselves

**Setup.** Every WoW64 guest through M10 (`hello32`, `gui32`, `win32`, `file32`) was written *by us*. That is a hidden bias: a guest you author can only exercise what you thought to exercise, and — worse — it silently *avoids* everything the layer doesn't support yet, because you never wrote the call that would have tripped it. M11's forcing function was to stop authoring guests and run a **stock, unmodified binary** instead: `base/applications/testgui`, built for i386 exactly as the LiveCD ships it — real `msvcrt` CRT startup, a real 53-module import closure, a real window with a child EDIT control.

It failed five times in a row, each on a bug **no hand-written guest could have found**. That is the lesson, and it generalises past WoW64: *a test you wrote cannot falsify an assumption you didn't know you were making.*

**Bug 1 — TLS. The 64-bit loader read a 32-bit image's `IMAGE_TLS_DIRECTORY`.**
`IMAGE_TLS_DIRECTORY32` is 24 bytes with 4-byte fields; the 64-bit shape is 40 bytes with 8-byte fields. The process image of a WoW64 process *is* a 32-bit PE and it sits in the 64-bit loader's `InLoadOrderModuleList`, so `LdrpInitializeTls` walked it with the native struct, pulled a garbage `AddressOfIndex`, and stored the TLS index through it (`str w8, [x9]` → AV). **Invisible before M11 because no hand-written guest was CRT-linked, so none had a TLS directory at all** — the loop skipped them all on the null check. Fix: the native loader skips modules whose `OptionalHeader.Magic` is `PE32`; 32-bit modules' TLS belongs to the guest's own i386 ntdll (same source, right shape).

**Bug 2 — the environment block was never marshalled.** `PEB32->ProcessParameters->Environment` was simply never filled in, so the guest saw `NULL`, and kernel32's `GetEnvironmentStrings` walked from address 0. The block itself is width-agnostic (a double-NUL-terminated WCHAR run); only the *pointer to it* diverges, and we'd never written one. **Invisible before M11 because no no-CRT guest ever reads the environment** — a CRT does, during `msvcrt`'s `DLL_PROCESS_ATTACH`, before `main`.

**Bug 3 — `NtCreateSemaphore` was a loud stub, and that killed EVERY GUI app.** `wininet`'s `DllMain` calls `CreateSemaphoreW` (via `init_urlcache`); `rpcrt4` statically imports `wininet`; `user32` imports `rpcrt4`. So one unimplemented syscall → `wininet` `DllMain` returns FALSE → `STATUS_DLL_INIT_FAILED` → *every* GUI process dies. It had sat on the declared-but-undispatched list for ages, harmless, because nothing we wrote ever created a semaphore. **A single missing leaf in a transitive import graph is a whole-subsystem outage.**

**Bug 4 — `LARGE_STRING` is 12 bytes in the guest and 16 on the host, and `NtUserDefSetText` was auto-thunked.** `{ULONG Length; ULONG MaximumLength:31; ULONG bAnsi:1; PVOID Buffer;}` — `Buffer` is at +8 in both, but the host reads **eight** bytes there, so the high half is whatever follows the guest's 12-byte struct on its stack: a wild pointer. win32k dereferenced it **while holding the USER lock**, so it did not fault cleanly — it **wedged the entire desktop** (every thread that touches win32k blocked behind the lock; even winlogon's `WaitForSingleObject` never returned). `NtUserCreateWindowEx` — the *other* `LARGE_STRING` taker — was marshalled back in M8; this one was missed. Reached only via `DefWindowProcW(WM_NCCREATE)` setting the title from `CREATESTRUCT.lpszName`, which every stock window does and none of ours did.

**Bug 5 — window extra bytes, two bugs stacked.** user32's EDIT control keeps its `EDITSTATE*` in the window's extra bytes (`Get/SetWindowLongPtr(hwnd, 0)`). Both halves were broken:
- *Write:* win32k's `NtUserSetWindowLongPtr` used `size_of::<LONG_PTR>()` = **8** unconditionally. A WoW64 caller's `LONG_PTR` is **4**, and the guest's class registration sized `cbwndExtra` in 32-bit units, so the bounds check `0 + 8 > cbwndExtra(4)` **rejected the store outright**. Fix: `UserCallerPtrSize()` — gate on `EPROCESS.Wow64Process`.
- *Read:* user32's `IntGetWindowLong` did `wndPtr.add(1)` — "extra bytes follow the WND". But under WoW64 `wndPtr` is a **normalized 32-bit *copy*** of the real 64-bit WND (the M9 `normalize_wnd` snapshot), living in a heap `Snap`; nothing follows it. It read whatever the allocator put after the Snap. Fix: `wnd_extra_base()` recovers the snapped object and indexes off the *real* 64-bit WND.

**The payoff, and the meta-lesson.** After all five, the stock binary under WoW64 reaches `wWinMain` → `RegisterClassExW` → `CreateWindowExW` → `WM_CREATE` — *and then dies in exactly the same place as the NATIVE arm64 build of the same source* (a pre-existing user32 EDIT bug, `EDIT_LockBuffer`/`LocalLock`). **That identical failure is the actual proof of correctness**: the emulation layer is now faithful, reproducing native behaviour bug-for-bug.

Which is the reusable technique: **always run the native build of the same source as an A/B control.** Without it, every crash inside a shared library looks like an emulation bug, and you will burn hours debugging the emulator for a bug that lives in the library. The control costs one `CreateProcess` call and it partitions the entire search space in a single boot. (`winlogon-rust::TryStartNativeTestGui`.)

**Two harness traps this also flushed out, both of which cost real time:**
- **An incrementally-`mcopy`'d FAT ramdisk accumulates corruption.** `file32`'s directory-enumeration rung started failing (`FindFirstFileA` returning *empty* `cFileName` for every entry, including `.`, while the *wide* enumeration was perfect). It looked exactly like a wide→ANSI/NLS bug and it was not: rebuilding `ramdisk.img` from scratch made it vanish. If a FAT-backed test starts failing in a way that makes no sense, **`rm ramdisk.img` and rebuild before you debug the OS.**
- **A "correctness fix" with no oracle behind it is a regression waiting to happen.** While fixing the callback layer I also made `WM_GETMINMAXINFO`/`WM_NCCALCSIZE`/`WM_WINDOWPOSCHANGING` copy the guest's OUT-payload edits back up — which is what those messages are *for*. It regressed `win32.exe` from 0x600D to 0x600E: WM_PAINT/WM_NCPAINT/WM_ERASEBKGND vanished and WM_MOVE/WM_SIZE/WM_GETMINMAXINFO/WM_NCCALCSIZE each went 1× → 3×, i.e. win32k fell into repeated size renegotiation and the window never settled enough to paint. Reverted to dropping the edits (the pre-M11 behaviour) with the evidence recorded in-code; the real fix is owed. **Ship the fix the failing oracle demands; park the one it doesn't.**


---

### 9.68 (ROOT-CAUSED + FIXED 2026-07-12) When the BUG IS IN THE TEST HARNESS: the standalone kmtest drivers' own `_SEH2_TRY` was dead code, so three "kernel crashers" were the kernel behaving *correctly*

**Symptom.** Three amd64 kmtest quarantines, all BUGCHECK-class, all filed as kernel bugs:
`IoReadWrite` (0x1E), `NtCreateSection` (0x50), `MmMapLockedPagesSpecifyCache` (0x1000).

**The trap.** Two of the three were not kernel bugs at all. `USE_DUMMY_PSEH` is the
project-wide C compile mode, so *every* `_SEH2_TRY`/`_SEH2_EXCEPT` compiles to a **dead
handler**. `kmtest_drv` and `kmtest.exe` had opted into real clang SEH
(`_USE_NATIVE_SEH`, which pseh2.h maps straight to `__try`) — but the **17 STANDALONE
helper drivers never did**. Their handlers were silently dead.

That matters because these drivers *deliberately fault and expect to catch it*.
`IoReadWrite_drv` is the textbook case: fast I/O hands a driver the **raw user buffer**
(no SystemBuffer), `TestCommonWrite()` derefs it with no NULL guard, and its own
`_SEH2_EXCEPT` is supposed to convert the AV into `STATUS_ACCESS_VIOLATION` — which is
*exactly what the test asserts*. With a dead handler the AV escaped to the kernel and
bugchecked. **The kernel was innocent; it faithfully delivered a fault the test asked for.**

**The wrong turn worth remembering.** The first triage note read: *"NtReadFile/NtWriteFile in
C dispatch fast I/O from inside `_SEH2_TRY`, so the kernel needs a protected **call** around
the FastIoRead/FastIoWrite dispatch."* **That was wrong** — C does **not** wrap the fast-I/O
dispatch in SEH (`iofunc.c`: only the IOSB write-back is protected). Nor could the probes
have saved it: C's `ProbeForRead` is range+alignment only and never touches memory, so a
NULL buffer sails through by design. Reading the *C oracle's actual control flow* — instead
of pattern-matching "DUMMY_PSEH + fault ⇒ kernel needs a guard" — is what found the driver.

**Fix.** `kmt_enable_native_seh()` in `modules/rostests/kmtests/CMakeLists.txt`, applied to
**all 17** standalone drivers (not just the faulting ones — the invariant is "kmtest drivers
get real SEH wherever clang lowers it", so the next SEH-using test can't inherit a dead
handler). amd64 + arm64 only; i386's SEH lowering is MSVC-env-only (dummy-pseh-verdict §C).
Result: `IoReadWrite` CRASH → **OK (16192 pass / 151 fail)**.

**Why it works despite "calls-only".** Hardware-fault coverage under clang native SEH needs
the fault in a **CALLEE** of the `__try` body. That holds here for free: clang **refuses to
inline any callee into a `__try` region** (funclet-EH restriction — the same rule that forces
`kmt_native_seh_shims.c` to exist), so `TestCommonWrite` stays a real call. Same mechanism
`ntoskrnl/kmsafecopy.c` relies on (§9.59).

**The negative result is the valuable half.** `MmMapLockedPagesSpecifyCache` was
un-quarantined in the *same* experiment and **still** BUGCHECKs 0x1000 — proving native SEH
did not paper over it and its original triage (real PFN/PTE accounting bug in the Rust
MDL-map path) stands. Un-quarantining the whole suspected cluster at once is what cleanly
*separated* the harness artifacts from the genuine kernel bug.

**Detector.** For any kmtest quarantine on a **standalone** driver (`*_drv.sys` built by its
own `add_library`, not linked into `kmtest_drv`), check whether the driver source contains
`_SEH2_TRY` *before* blaming the kernel:
`grep -l _SEH2_TRY modules/rostests/kmtests/*/*_drv.c` — if it's there and the target lacks
`kmt_enable_native_seh()`, the handler is dead and the "kernel crash" is the harness.
**Generalization:** a test that deliberately faults is only a valid oracle if its *own*
exception handling actually compiles. Verify the harness before you debug the subject —
`objdump -p drv.sys | grep C_specific_handler` proves the handler is real.

---

### 9.69 (ROOT-CAUSED to the mechanism 2026-07-13, FIX OWED) arm64 kernel-mode SEH wedges on a real hardware fault: `RtlVirtualUnwind` decodes a GARBAGE frame size → a garbage SP → the transfer-to-handler faults on the bad SP → infinite exception-vector re-entry. NOT a fault storm, NOT the crutch, NOT a bad handler routine.

This entry was WRONG TWICE before landing on the truth; both wrong versions are kept as
cautionary notes, because each "explained the symptom" and each was falsified by an instrument.

**Symptom.** Enabling the §9.59 fault-tolerant copy on arm64 wedges the boot (`HANG_TIMEOUT`, no
desktop) when the self-test probes a genuinely-unmapped VA (`0x0000700000000000`).

**Wrong hypothesis #1 (falsified):** a DUMMY_PSEH "silent-skip crutch" in `KiDispatchException`
swallows it. → Instrumentation: the crutch is NEVER REACHED.
**Wrong hypothesis #2 (falsified):** a recursive FAULT STORM overflows the kernel stack. → A
fault-chain trace at the vector entry (`[FC]`, armed on the probe FAR, prints every kernel fault)
shows **exactly ONE** fault reaches the Rust handler. It is not a storm.

**What the evidence actually shows** (each datum from an instrument verified to emit — `[FC]` and
`[SEHCTX]` both printed through the same UART path that `[UEXC]` had already validated):
1. `[FC] n=1 EC=0x25 FAR=0x700000000000 PC=<in memcpy> SP=0xffff8000055ceb20` — the ONE fault; a
   clean data abort in `memcpy`, on a valid kernel stack.
2. `[SEHCTX] ctxPc=<in memcpy> ctxSp=0xffff...ceb20 ctxFp=… ctxLr=<in KmCopyRaw>` — the CONTEXT that
   `KeTrapFrameToContext` hands `RtlDispatchException` is SANE (so hypothesis "bad context" is also
   out). `RtlDispatchException` is entered and NEVER RETURNS (`[SEHDBG]`, which prints after it
   returns, never fires).
3. GDB at the hang: `PC = KiArm64VectorTable`, `ESR=0x96000045` (data abort, write), and
   `FAR ≈ SP + 0xb0` — the exception vector faulting while writing its OWN trap frame = the stack
   pointer is invalid. Crucially **`SP = 0xfffffffe35d96540`**, not the sane `0xffff8000055ceb20`
   it entered with. `SP_initial + framesz = garbage` ⟹ **`framesz ≈ 0x7ffe30419a20`** — an absurd
   frame size. So one frame's unwind decoded a garbage `FrameSize`.

**Mechanism.** `RtlDispatchException` finds `KmProtectedCopyMemory`'s `__except` (constant filter
`HandlerAddress==1`), so `__C_specific_handler` calls `RtlUnwindEx(EstablisherFrame, JumpTarget,…)`
(`rtl-except/src/arm64/except.rs:528`) to transfer control to the handler. `RtlUnwindEx` unwinds the
REAL context frame-by-frame via `RtlVirtualUnwind`; for one frame in the
`memcpy → KmCopyRaw → KmProtectedCopyMemory` chain it computes a garbage `FrameSize`, sets
`Context.Sp = sp + framesz` to `0xfffffffe35d96540`, and transfers there. The very next exception
(any) tries to save state at that garbage SP → data abort → the vector re-enters to save state →
still-garbage SP → data abort → … infinite re-entry in `KiArm64VectorTable`. Only the FIRST fault
reached Rust because after SP is trashed the vector's own asm prologue faults before it can call the
Rust handler — which is exactly why the fault-storm hypothesis looked plausible from the GDB "stuck
in the vector" view but is wrong.

**Why kernel-mode only / why the self-test was worth it.** `seh_poc.c` (a SOFTWARE raise) passes on
arm64 through the SAME `RtlDispatchException` — because it never traverses the `memcpy`/CRT-shim
frames whose unwind data trips the decoder. Shipping the `cfg` on the strength of "the symbol
exists + software raise works" would have turned a hostile-pointer bugcheck into a silent HANG.

**FIX OWED (task #10), now tractable.** Break at `RtlVirtualUnwind` (runtime addr = static +
reloc), dump `ControlPc` (the frame) and the resulting `Context.Sp` on each call; the call where Sp
jumps to `0xffff_fffe_…` is the culprit frame. Then decode that function's `.pdata`/`.xdata`
(`llvm-objdump -s -j .pdata / -j .xdata`) and compare against `unwind_decode::virtual_unwind_xdata`
/ the packed-format `FrameSize` reconstruction in `except.rs:205-285`. The bug is a `FrameSize`
(or code-word/word_cursor) mis-decode for a specific unwind-info shape the CRT shims emit. Interim:
arm64 keeps the plain copy + honest documented gap; i386 likewise. Arch-neutral escape (needs NO
working SEH, closes both): a Linux-extable-style fault-fixup table consulted in the page-fault
handler.

**Reusable lessons.** (a) A hypothesis that explains the symptom is not evidence — this one was
wrong twice. (b) Verify the instrument EMITS before trusting its silence (`[FC]:1` only meant
something because `[UEXC]` proved the macro). (c) Probe an address that ACTUALLY faults (`0x1` is
mapped on arm64). (d) "Stuck in the exception vector" can be an SP-corruption artifact, not a fault
storm — read the SP and compare to entry.

---

### 9.70 (ROOT-CAUSED + FIXED 2026-07-13) An SEH *filter expression* translated into a runtime value — `GlobalLock` returned NULL unconditionally, and every edit box in the OS was dead

**Symptom.** user32's EDIT control was completely broken on *every* arch: `EDIT_LockBuffer` left `es->text == NULL`, and the first thing that touched it (`EDIT_EM_ReplaceSel`, setting the initial text) dereferenced NULL. Notepad, every dialog text field, every edit box — all dead. It had been that way silently, because nothing in the boot path types into an edit control.

**The find.** In `kernel32-rust::client::heapmem::GlobalLock`:

```rust
let __except_val = EXCEPTION_EXECUTE_HANDLER;   // == 1. A CONSTANT.
let Ptr = if __except_val != 0 {                // ...so ALWAYS true
    SetLastError(ERROR_INVALID_HANDLE);
    core::ptr::null_mut()                       // the handler runs EVERY time
} else {
    __try_result                                // the real result: DEAD CODE
};
```

The C was the ordinary

```c
_SEH2_TRY { ...compute Ptr... }
_SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) { SetLastError(ERROR_INVALID_HANDLE); Ptr = NULL; }
_SEH2_END;
```

The translation turned the **filter expression** (`EXCEPTION_EXECUTE_HANDLER`, a compile-time constant meaning "if an exception reaches me, run my handler") into a **runtime value**, and then read `filter != 0` as "an exception occurred". So the handler body executed on every call and the try block's result was thrown away. `GlobalLock` returned NULL unconditionally; `LocalLock` forwards straight to it.

**Why it stayed hidden.** The two callers that matter both *look* like they're checking for failure and both quietly absorb it:
- `EDIT_LockBuffer` assigns `es->text = LocalLock(...)` and never checks it — the contract is "after a Lock, text is valid" — so the NULL surfaced three frames away as an AV at address 0.
- `EDIT_WM_NCCreate` computes `es->buffer_size = LocalSize(hloc)/sizeof(WCHAR) - 1`. `LocalSize` fails the same way and returns 0, so `buffer_size` became `0/2 - 1` = **0xFFFFFFFF** — a giant bogus size that nothing range-checked. That `4294967295` in a diagnostic print was the thread that unravelled it.

**The shape to grep for.** An SEH filter is a *predicate evaluated at throw time*, not a value. Any translation that materialises `EXCEPTION_EXECUTE_HANDLER` (or `EXCEPTION_CONTINUE_SEARCH`) into a local and then branches on it has inverted the control flow — the handler becomes the fast path. Sweep: `grep -rn "EXCEPTION_EXECUTE_HANDLER" --include=*.rs` and look for any `if <that> != 0`. Under `USE_DUMMY_PSEH` the correct lowering of a try/except whose body cannot actually fault is simply **the body, with no handler at all**.

**Meta-lesson (compounds with §9.67).** This is the third bug in a row that only a *stock* binary could reach — and the one with the widest blast radius, since it broke a control every real app uses. A hand-written test guest doesn't type into edit boxes. Neither does a boot-to-desktop smoke test. The OS booted green to a desktop for months with every text field in it non-functional; the marker `shell ready` cannot see that. **When a subsystem has no oracle, "green" only means the parts with oracles are green.**

---

### 9.71 (ROOT-CAUSED + FIXED 2026-07-13) A `#ifdef _WIN64` in a syscall list, parsed as flat text — 195 WoW64 syscalls silently misrouted onto their neighbours

**Symptom.** x86 Doom ran beautifully under WoW64 — CRT started, the 4 MB IWAD loaded, the window was created, the game loop pumped, and `NtGdiStretchDIBitsInternal` blitted **4000+ full frames**, each returning 400 scanlines copied. And nothing appeared on screen. The window was in the taskbar. Not one thing had failed.

**Root cause.** The WoW64 thunk table maps *the service number the 32-bit guest puts in EAX* to a host function, so its indices must be the **guest's** numbering. `win32ss/w32ksvc32.h` gates one entry on the host's word size:

```c
    SVC_(UserSetWindowLong, 4)
  #ifdef _WIN64
    SVC_(UserSetWindowLongPtr, 4)     /* 64-bit ONLY: on i386, SetWindowLongPtr IS SetWindowLong */
  #endif
    SVC_(UserSetWindowPlacement, 2)
```

`wow64gen.py` parsed that file with a line regex and **no preprocessor awareness**. On a 64-bit host it therefore included the `_WIN64`-only entry and every service number after it shifted by one. **195 of 736 syscalls dispatched to the wrong function** — off by exactly one slot.

Nothing crashed, because the neighbours are *plausible*: the guest's `NtUserShowWindow` (0x1227) landed on the host's `NtUserShowScrollBar`, which cheerfully accepted the arguments and did nothing useful. Doom's `ShowWindow(SW_SHOW)` became `ShowScrollBar`, so its window never became visible — and it spent the next ninety seconds rendering thousands of frames into a device context nobody could see.

**Why it survived M3–M11.** Every guest before Doom happened to use only syscalls *below* the shift. `NtUserGetDC` is index 400 and correct; the divergence starts at 541. A whole lane of milestones passed through a table that was wrong in its top quarter.

**The lesson, and the general shape.** *A generated table whose indices come from one side of a boundary and whose contents come from the other is only correct where the two agree — and it will not tell you where they stop agreeing.* Two specific traps compounded here:

1. **Parsing a C header with a regex is parsing it wrong.** `#ifdef` is not decoration. If a list is conditionally compiled, the generator has to expand it the same way the *consumer* does — here, with `_WIN64` **undefined**, because the consumer is always a 32-bit guest.
2. **An off-by-one in a dispatch table is invisible to every failure detector we have.** No fault, no bad status, no `NEEDS MANUAL` line, no ud2. The only oracle is *comparing the table against the thing that actually calls into it*.

**The permanent guard** is exactly that comparison: `tools/wow64gen/verify_svc_numbering.py` disassembles the **real i386 `user32.dll`/`gdi32.dll` syscall stubs** out of the build (the ground truth the guest executes — `movl $0x1227, %eax`), and hard-fails if any of them maps to a different name than the generated table has at that index. It is wired into `vibeos-cli check`. Ground truth beats a second parse of the same header.

### 9.72 (ROOT-CAUSED + FIXED 2026-07-13) The rustconv defect chain behind ole32's silent bugs: one regex hang bypassed the entire post-processor, and `abi_enforce` was throwing away the calling convention it had correctly derived

The ole32 translation shipped ~130 COM vtbl methods with the **Rust ABI** instead of
`__stdcall`, an export (`StgCreateDocfile`) with no `#[no_mangle]`, all 10 file-scope vtbl
tables missing, and seven wrong constants. Those were read as "LLM sloppiness." They were
not. They were **four tool bugs**, and they compound — this is the whole causal chain:

**1. A catastrophic-backtracking regex hung `postprocess`, and the hang looked like the
stitcher.** `drop_pseh_macro_externs` matched an extern block's body with an ambiguous
`(…fn…;\n)+` whose `[^;{]+` return-type class *also matches newlines*. Fine on its own —
the killer was that the stitcher's merged extern block legitimately contains `static`
decls among the fns, so the `+` ran out at the first `static` and **could never reach the
closing `}`**. The engine then re-partitioned the preceding 46 fn decls every possible
way: a clean 2^n (18 decls = 13 ms, 26 = 5.1 s, 30 = 37 s, **46 = never returns**). Only
big files have a long enough decl run, which is why it looked file-specific.
`drop_gdiobj_macro_externs` (win32k) carried the identical latent bomb.
- It printed nothing of its own, so the last message on screen was `Stitching N
  functions…` — and *the stitcher got blamed for years*. `stitch_file` takes **10 ms**.
- **The damage is the workaround.** The operator kills the wedged run and hand-assembles
  the `.responses/*.rs` pieces — which **bypasses `postprocess()` entirely**, and with it
  ABI enforcement *and* the already-paid-for translated preamble. Nearly every "LLM bug"
  on `storage32.c`/`moniker.c` traces to this one regex.
- Fix: a linear line-scan (`_filter_extern_block_decls`) — find the opener, brace-depth to
  the closer, filter per line. 2.3 ms, and *strictly more capable* (it can process the
  mixed fn/static blocks the old pattern could never match at all).
- **Lesson: a hang is a correctness bug.** It doesn't just cost time; it trains an operator
  into a workaround that silently disables your safety passes.

**2. `abi_enforce` derived the right convention and then discarded it.** It correctly
computed `('system', True)` for every `static … WINAPI` COM method — then
`_canonical_block()` did `if is_static: return f"unsafe fn {name}…"`, dropping the
`extern`. It **conflated linkage with calling convention**. In C, `static` controls
*linkage*; `WINAPI` controls the *convention*, and `static HRESULT WINAPI Foo()` is both.
Result: Rust-ABI fns in C vtbl slots — links clean on amd64/arm64, corrupts the i386 stack
at the first call. 130 in ole32; **153** in oleaut32's remaining six files.

**3. `_derive_convention` fell back to cdecl on a comment.** It scanned `)`→`{` for a
definition skipping *whitespace only*. wine annotates the last parameter:
```c
    IEnumSTATSTG**  ppenum)    /* [out] */
    {
```
It stopped on the `/`, concluded "not a definition", returned `None`, and the caller
**silently fell back to cdecl** — mis-deriving 17 more genuine WINAPI methods. Note this
survives the fix in (2) untouched: they'd come out `extern "C"`, still wrong on i386.

**4. `CALLBACK`/`PASCAL` were missing from the convention keyword list** (`minwindef.h`:
`#define CALLBACK __stdcall`). These mark exactly the fns C hands to the OS **as function
pointers** — an `EnumFontFamiliesEx` callback, a DlgProc. user32/gdi32 call them *through
the pointer with stdcall*, so a Rust-ABI body corrupts the i386 stack on a call **we never
make ourselves**: no call site in our code to inspect, nothing to grep for.

#### The three deterministic oracles that came out of this (use them; don't ask an LLM)
- **`vtbl_gen.py`** — COM vtbl tables generated from the C initializer + the *bindgen*
  struct's field order. The LLM dropped **all 10** of storage32's (a COM object with no
  methods at all) because `translate` is function-granular and file-scope data falls to a
  single preamble call that silently truncated. Arity mismatch **raises** rather than
  zip a table into the wrong methods. Independently agreed **149/149 slots** with the
  hand-written tables. *Do not let c2rust emit these either*: it hardcodes each slot as
  `f as unsafe extern "C" fn(..)`, the amd64 spelling, which is simply wrong on i386.
- **`table_audit.py`** — compares every file-scope C data table against the Rust,
  element by element. It caught `fmtLongTime` in `varformat.c`, the one file c2rust
  **cannot** anchor (upstream panic, `convert_type.rs:518`) and therefore the only file
  with no other oracle. **Model C's implicit zero-fill**: `BYTE t[256]` with 103 written
  elements is 256 bytes; Rust has no such rule and must spell all 256.
- **`export_audit.py`** — every `.spec` export must exist as a `#[no_mangle]` fn with the
  spec's convention. `make_user_dll_target` had `spec_file=None` for *every* user-mode DLL,
  so the pipeline never knew which fns were exports — it neither told the model nor
  enforced `#[no_mangle]`. That is how `_StgCreateDocfile@16` went missing from a DLL.

#### The meta-lesson
Every one of these produced code that **compiled, linked, and booted**. The smoke tests
cannot see a COM DLL with a wrong calling convention on a path the boot doesn't take, or a
date-format table with one wrong byte. When translating a *library* rather than a boot-path
component, the boot is not the oracle — **the C is**. Build the differential checks
(convention, exports, tables, vtbl slots) and run them in the pipeline, or you will ship
silent wrongness and only find it when a user formats a date.

**Corollary — audit the tool with the tool.** Every false positive `table_audit` produced
against *shipped, working* code was a bug in the auditor, and each one taught something
real: function-local statics collide by name (wine's `variant.c` defines a *different*
`prio2vt` in `VarAdd` and `VarMul`); `#ifdef __REACTOS__` means half the file-scope C is
dead and comparing against it reports 7 bogus diffs; a name can be legitimately defined at
two Rust scopes. Calibrate a new checker against known-good code **first** — a checker that
cries wolf is worse than no checker, because it trains you to ignore the one true finding.

### 9.73 (ROOT-CAUSED + FIXED 2026-07-13) The stub you wrote to make the reference *link* IS an oracle — and it will grade your code against your own model. Six shipped msvcrt functions were wrong; the differential harness said MATCH for four waves.

**Symptom.** None. That is the point. `classify` — a 20,000-case differential fuzzer comparing
Rust msvcrt bodies against msvcrt's genuine C, grading bytes + return + errno + invalid-parameter
handler — reported **MATCH** for `_atoi64`, `strtol`, `strtoul`, `_strtoi64`, `_strtoui64` and
`_memicmp` across waves 6–9. All six were shipping the wrong behaviour on master.

**What broke.** The harness compiles msvcrt's real `string.c` / `wcs.c` as reference TUs. Those
TUs *call* `_isspace_l` / `_tolower_l` / `_toupper_l`, which live in `ctype.c` — which was **not**
a reference TU. So nothing defined them, and `refimpl.c` (the harness's "externals" file) supplied
three convenient one-liners so the thing would link:

```c
int __cdecl _tolower_l(int c, _locale_t l) { (void)l; return ref_ascii_lower(c); }
```

That is not scaffolding. **That is the oracle.** Every reference call to `_tolower_l` was answered
by *our model of msvcrt*, and our model was an ASCII fold. The Rust under test also folded ASCII.
The two agreed, forever, about something neither of them was doing correctly.

**What msvcrt actually does.** `_tolower_l` is a locale-table lookup with a domain test:

```c
if((unsigned)c < 256) {                      /* FALSE for a NEGATIVE c */
    if(locinfo->pctype[c] & _LEADBYTE) return c;
    return locinfo->pclmap[c];
}
/* ... else: build a 1-2 byte string, and: */
*_errno() = EILSEQ;                          /* <-- a SIDE EFFECT, on the ordinary path */
switch(__crtLCMapStringA(...)) { ... }
```

and msvcrt's own callers hand it a **raw signed `char`**:

```c
while(_isspace_l((unsigned char)*nptr, locale)) nptr++;   /* CAST */
...
char cur = _tolower_l(*nptr, locale);                     /* NO CAST -- string.c:1651 */
```

So any byte ≥ 0x80 sign-extends negative, misses `(unsigned)c < 256`, lands in the NLS arm, and
**sets `errno = EILSEQ`**. Every one of the six functions inherits that. The *values* were fine
(the bad fold just ends the parse) — only `_memicmp`, which consumes the folded byte, was wrong in
its return value too. The entire defect lived in an effect the oracle had been taught not to have.

**The fix.** Make `ctype.c` a real reference TU (so the reference grades itself), delete the three
stubs, and *transcribe* the parsers so they call the same functions at the same points — rather
than bolting msvcrt's contract onto a foreign engine (`cnt_rust::radix::parse_i64_clamped`, which
has no way to express a side effect). Result: 6 DIVERGE → 0, and the same infrastructure unblocked
50 new flips (the whole locale-table ctype/case family).

**The rules that generalize.**

1. **Anything you define so the reference will link is part of the oracle.** Audit those definitions
   as carefully as the code under test. A one-line stub is a claim about msvcrt's semantics, and an
   un-reviewed claim is a lie you have arranged to have confirmed.
2. **Data you feed to BOTH sides can never be caught by a differential test.** A mistyped byte in a
   shared table makes both sides wrong the *same* way and still reports MATCH. So supply as little
   as possible, and take what you must supply from the real source (`build.rs` now *parses*
   `locale.c`'s `cloc_clmap`/`cloc_cumap` rather than transcribing them; ctype.c's `MSVCRT__ctype`
   is linked, not modelled).
3. **A side effect is not something you can bolt onto a foreign engine.** If the C's behaviour comes
   from *calling a particular function*, the faithful port calls that function. This is the same
   lesson the wave-5 float probe taught (`strtod` needs msvcrt's bignum, not a BSD parser) and the
   same one behind "cnt-rust supplies the ALGORITHM; msvcrt-crt-rust supplies msvcrt's SEMANTICS."
4. **Model the target's constants, not the host's.** The harness had silently inherited glibc's
   `EILSEQ` (84) where msvcrt says **42**. `EINVAL`(22)/`ERANGE`(34) happen to coincide between the
   two OSes, which is exactly why nobody noticed. A numeric coincidence between two operating
   systems is not a model — pin them.

**Detector.** There isn't a generic one, and that is worth saying plainly. What surfaced this was
*promoting a stub to a real TU* — i.e. deliberately reducing the amount of msvcrt the harness was
pretending to be. **Periodically ask of any differential harness: what am I still MODELLING, and
what would it cost to link the real thing instead?** Every model is a place the oracle can agree
with you about a shared mistake.

### 9.74 (2026-07-13) The corollary: `offset_of!` into a struct the harness defines differently is an oracle that agrees with a subject reading the wrong bytes

Wave 10's Rust reads `locinfo->pctype/pclmap/pcumap` and `thread_data->strtok_next` through
`msvcrt-sys` — a bindgen snapshot of msvcrt's **real** layout (`pcumap` @ 336 on amd64/arm64, @ **208**
on i386; `pctype` and `mb_cur_max` migrate to the *front* of the struct at `_MSVCR_VER >= 140`). The
differential harness links that same Rust and points it at a `threadlocinfo` **defined in the harness's
own refstub**, which was a convenient cut-down (6 fields, `lc_handle[16]`).

Two different layouts, one set of offsets. The Rust would have read its tables from past the end of the
object — and since *both* sides would then be fed the same nonsense, `classify` would have reported
MATCH. It does not crash. It just makes every verdict worthless.

Three guards now, and each was **negative-controlled** (perturb → it must fail):

| guard | checked by | catches |
|---|---|---|
| `msvcrt-sys/src/layout_<arch>.rs` | rustc, every Rust build | bindgen regenerated with wrong flags |
| `dll/win32/msvcrt/rust_layout_assert.c` | clang, with **msvcrt's real flags**, ×3 arches, every build | the C headers moved under the snapshot |
| `ref_integrity`'s `locale_layout_agrees_with_msvcrt_sys` | the harness, every run | the ORACLE's struct ≠ the shipping Rust's |

Two footguns found while building them, both worth knowing:

* **`compile_commands.json`'s `command` is a shell-quoted string.** CMake serialises the empty value
  of `-D_CRTIMP=` as `-D_CRTIMP=""`. A naive `.split()` hands clang a literal two-quote value, and
  since `_ACRTIMP` chains to `_CRTIMP`, every CRT declaration becomes `"" int __cdecl strcmp(...)`.
  **`shlex.split()`, never `.split()`.**
* **bindgen records the Windows spelling of a fixed-width type.** msvcrt's `__msvcrt_ulong` is 32-bit
  on *both* branches (`unsigned int` under `__LP64__`, `unsigned long` under LLP64 — corecrt.h goes out
  of its way to say so), but bindgen emits `c_ulong`, which is **64-bit on the linux host** that builds
  the harness. Pin such aliases to `u32`/`i32` in the generated bindings; the guardian above is what
  caught it, on day one, in anger.

### 9.75 (2026-07-14) Exercising a branch proves nothing if the branch cannot produce a DIFFERENT ANSWER — a differential harness's "non-default mode" that was a false green

**Context.** msvcrt has a whole family of functions that branch on whether a real locale is set:

```c
int __cdecl _strnicmp_l(const char *s1, const char *s2, size_t count, _locale_t locale) {
    if(!locinfo->lc_handle[LC_CTYPE]) {          /* path 1: the "C" locale */
        do { if ((c1 = *s1++) >= 'A' && c1 <= 'Z') c1 -= 'A' - 'a'; ... } while(...);
        return (unsigned char)c1 - (unsigned char)c2;
    }
    do { c1 = _tolower_l((unsigned char)*s1++, locale); ... } while(...);   /* path 2 */
    return c1 - c2;                              /* ...and a DIFFERENT return convention */
}
```

Our differential harness pinned the "C" locale, so **path 2 was never exercised** and a path-1-only
Rust implementation was reported MATCH forever. It was, and they were (`_stricmp`, `_strnicmp`,
`_wcsicmp`, `_wcsnicmp`, `_strlwr_s`, `_strupr_s` + wide twins — correct in the "C" locale, wrong the
moment `setlocale` is called).

**The trap.** The obvious fix is to plant a non-zero `lc_handle[LC_CTYPE]` and re-run the rows. That
**does** make msvcrt take path 2 — and it still proves nothing, because path 2 folds through
`_tolower_l`, which reads `locinfo->pclmap[]`, which was **still the C locale's map**: an ASCII fold.
Both paths therefore computed the *same answer on every input*. A path-1-only body was
indistinguishable from a correct one, and the new "mode" reported a confident green.

We only found out because we ran the negative control: **sabotage the subject to always take path 1,
and demand the new rows convict it.** They didn't move.

**The rule.**

> **Exercising a branch is not the same as testing it.** A branch is only tested when the two sides of
> it can produce *different observable results* for some input you actually generate. Before trusting a
> "new mode" in any harness, ask: *what input makes the mode's answer differ from the default mode's?*
> If you cannot name one, the mode is decorative.

The fix was to make the alternate locale's tables genuinely non-ASCII (Latin-1 case pairs,
`0xC0–0xDE ↔ 0xE0–0xFE`), so path 1 leaves `0xE9` alone while path 2 maps it to `0xC9`. Then the
negative control bit, and the *first honest run* immediately convicted `_wcsicmp` — which has its own
fold loop rather than delegating to the shared core, so an earlier fix had missed it.

**The corollary, which is §9.73 restated:** the alternate-mode tables must be **one copy, shared by
both sides**. Data you hand to both the reference and the subject can never be caught differentially —
a mistake makes both wrong the same way and classify still says MATCH. So supply as little as possible,
and share what you must. (Same reasoning retired the `msvcrt_ref_*` namespace for `LCMapStringA/W`,
`CompareStringA/W` and `GetStringTypeW`: kernel32 is the *environment*, never code under test.)

**Generalises to:** any harness with a "mode" (endianness, alignment, locale, SMP-vs-UP, a feature
flag). The mode is worthless until you have shown it can fail.

### 9.76 (ROOT-CAUSED + FIXED 2026-07-14) Five instruments passed and the OS booted 3× slower: a cheap harness stub hid a real COST, and no *ratio* can ever catch that — count, don't measure

**Symptom.** Wave 11 gave msvcrt's `_wcsicmp` its locale branch. Every instrument was green:

| instrument | verdict | what it grades |
|---|---|---|
| `classify` (20k-case differential fuzz, 4 effects) | **139 MATCH** | values |
| `oob_guard` | in bounds | access |
| `perf` (2× bar vs the genuine C) | **0.91×, PASSING** | speed |
| `ref_integrity` | oracle honest | the oracle |
| `link` | resolves | symbols |

and the gate went red: `test_tdi_selftest_roundtrip`, **deterministically, 3/3 retries**. Boot went
from **39s to 128s**. The *smoke tests* — the North Star — caught what the entire differential
harness could not.

**Root cause.** The Rust folded each character through `_towlower_l(c, NULL)`, and a NULL locale
makes that re-enter `get_locinfo()`. In the real msvcrt that is **not cheap**:

```c
thread_data_t *msvcrt_get_thread_data(void) {
    DWORD err = GetLastError();                     /* Win32 call */
    if (!(ptr = TlsGetValue( msvcrt_tls_index )))   /* Win32 call */
    ...                                            /* SetLastError */
```

**Three Win32 calls per character**, in a function on the boot's hot path (registry key
comparisons). The C never does this — it resolves the locale **once** and passes it down:

```c
INT CDECL _wcsicmp_l(const wchar_t *s1, const wchar_t *s2, _locale_t locale) {
    if(!locale) locale = get_current_locale_noalloc(&tmp);   /* ONCE */
    do { c1 = _towlower_l(*str1++, locale); ... }            /* ...then passed down */
```

**Why `perf` was blind — and always will be.** The harness stubs `get_locinfo()` as a static-address
return. **Both** the reference and the subject pay that same trivial cost, so the *ratio* stays
healthy while the real DLL burns three syscalls a character. This is §9.73's lesson in a third guise:
there, a cheap stub (`refimpl`'s ASCII `_tolower_l`) hid a real **behaviour**; here a cheap stub hides
a real **cost**. *Every stub is a claim about the environment, and a cheap stub claims the environment
is cheap.*

Tuning the stub to be "realistically slow" would be tuning a fake. So don't measure — **COUNT**.

**The fix, and the new instrument.** `sdk/lib/rust/msvcrt-difftest/tests/hot_path.rs` (the sixth
instrument) counts `get_locinfo()` calls and asserts:

> **A function that walks `n` elements may consult the environment O(1) times, not O(n).**

Deterministic, immune to machine noise, and exactly the invariant the C's own structure encodes. Run
each function at n=8 and n=512 and demand the count doesn't grow.

**It found two more instances on the FIRST run — both already landed on master:**

| fn | n=8 | n=512 | |
|---|---|---|---|
| `_memicmp` | 16 | **1024** | two `get_locinfo()` *per byte* |
| `strtol` | 9 | **513** | one *per digit* |

(The C is per-character in those two as well, so this was faithfulness, not divergence — but
`get_locinfo()` is idempotent, so hoisting is behaviourally identical and strictly better. msvcrt's own
hot comparer hoists.)

**Rules.**

1. **A ratio cannot see a cost your harness stubbed out.** When the subject and the reference both call
   the same cheap fake, the fake's real price is invisible. Ask of every stub: *what does this cost in
   the real system?*
2. **Count, don't measure, when the invariant is structural.** "O(1) environment lookups per call" is
   an exact, deterministic property. Asserting it beats benchmarking it.
3. **A perf gate only defends what it benches (§9.63, again).** The case-insensitive and case-mapping
   families were not benched — they are now. Every new family gets benched *and* hot-path-counted when
   it is flipped.
4. **When the North Star disagrees with your instruments, the instruments are wrong.** The smoke test
   is the source of truth for a reason. A green harness and a red boot means you are missing an
   instrument — go build it.

### 9.77 (2026-07-14) Two reading hazards that have now each cost a bug: a comment-stripping grep eats `*_errno() = ...`, and the host reference silently compiles the *i386* arm of every `#if defined(_WIN64)`

Both surfaced flipping msvcrt's collation family. Both are about **reading the C wrong**, not
writing the Rust wrong, and both were caught only by the differential harness.

#### 1. Never read C through a comment-stripping grep

To skim a function I used

```
sed -n '1478,1512p' string.c | grep -vE "^\s*\*|^/\*"
```

to drop the `*`-prefixed comment continuation lines. That pattern also drops **every statement
that begins with a dereference**:

```c
        *_errno() = ERANGE;      /* starts with whitespace-then-star */
        *p++ = *src++;
```

So the `_strxfrm_l` I read had **no errno in it at all**, and the flip I wrote had none either.
`classify` convicted it (`ret=17` on both sides, `errno=ERANGE` only on msvcrt's).

**This is the second time the same filter has eaten load-bearing C** — the first was `*p++ = *src++;`
in `wcsncpy_s` (wave 6c), where it nearly produced a copy loop with its copy removed. The rule is
absolute: **read the C unfiltered.** If it is long, page through it; do not filter it. A filter that
removes comments is indistinguishable, to a regex, from one that removes pointer writes.

#### 2. `_WIN64` — the host reference compiles the 32-bit arm

The differential harness compiles msvcrt's real `.c` for the **host** (x86-64 Linux), where `_WIN64`
is *not* defined. So every

```c
#if defined(_WIN64) || defined(_UCRT) || _MSVCR_VER >= 110
    if ((unsigned char)*str1 > (unsigned char)*str2) return 1;   /* CLAMP  — amd64/arm64 */
    ...
#else
    return (unsigned char)*str1 - (unsigned char)*str2;          /* RAW    — i386 */
#endif
```

is graded on its **i386** arm — while `msvcrt-crt-rust`, compiled for the same host, selects its arm
with `cfg!(target_pointer_width = "64")` and therefore ships the **amd64** one. *The oracle models
i386 while the subject models amd64.*

`strncmp` was already known and handled (two rows: the raw arm graded directly, the `_WIN64` arm
graded against `clamp(ref)` — an exact model, since both C branches run the identical loop and then
compare the same two bytes). But `_strncoll_l`'s "C"-locale path is literally

```c
return strncmp(str1, str2, count);
```

so it **inherits both contracts**, and the new flip walked straight into it: `cnt=1 msvcrt=47`.

**Rule:** *grep every new flip — and everything it CALLS — for `#if defined(_WIN64)` before writing
the row.* An arch-conditional contract propagates silently through delegation. classify's own header
already said this; the cost of not doing it is one DIVERGE per inherited branch, and the harness will
only catch it if the fuzz happens to produce a byte pair whose raw difference isn't ±1.

### 9.78 (ROOT-CAUSED + FIXED 2026-07-14) A parse failure in the *scanner* deletes a function from the plan, and every downstream check will pass — ws2_32 shipped a translation with no `bind`, `connect` or `sendto`

**Symptom.** ws2_32-rust translated (34 files, ~14 K LOC), compiled to zero errors on i386,
amd64 and arm64, passed the whole lint gate, linked, and booted to desktop on all three arches.
It was missing `bind`, `connect`, `sendto`, `WSAConnect`, `WSAJoinLeaf` and `WSASendTo`.

**Root cause.** `#define CONST const` (winnt.h). tree-sitter-c does not know `CONST`, so a
parameter spelled

    IN CONST struct sockaddr *name

reads as *two* type-identifiers, and the **enclosing function produces ZERO
`function_definition` nodes** — the function does not appear as unparseable, it simply does not
appear. All six lost functions take exactly that parameter. Fixed in `scanner.py::_blank_memmodel`
by normalising `CONST` → `const` alongside the existing `FAR`/`NEAR` blanking (§ the FAR fix that
lost `inet_addr` the same way, commit `12ca1606704`).

**Why nothing caught it — this is the transferable part.** A dropped export is invisible to every
check we run, and it is worth understanding *why each one* is blind:

| check | why it passes anyway |
|---|---|
| rustc | nothing in the crate calls the missing fn |
| the linker | a `.spec` export with no implementation is **not an error** — the DLL just ships a smaller export table |
| `verify_ffi` | compares a module's decls to its own `.spec`; it checks *conventions*, and has no opinion about a name that is simply absent |
| smoke boot | passes unless the boot path happens to call that exact API |
| the translation's own cost/coverage output | reports what it *planned*, and the function was never in the plan |

The plan is derived from the parse. **If the parse silently loses an item, every downstream signal
is computed over the reduced set and agrees with itself.** A pipeline cannot audit its own input.

**The oracle has to come from outside the pipeline** — here, the `.spec`. `tools/verify_lost_exports.py`
(gate-wired) reports a symbol that is (1) in the `.spec`, (2) defined in a `.c` that is **commented
out** in the cmake, and (3) absent from Rust. All three conditions are load-bearing; dropping any
one turns it into a wolf-crier, and this checker cried wolf **four separate times** before it was
trustworthy:

* `#if 0` bodies — kernel32's `IsValidLocaleName` lives in `#if 0 // See kernel32_vista`. The C
  never compiled it either.
* **prototypes with no body** — ReactOS puts the name at column 0 for a declaration *and* a
  definition, so `^sym(` cannot tell them apart. kernel32's four `Get/SetFirmwareEnvironmentVariableEx*`
  are declared in `client/sysinfo.c` and defined nowhere.
* `.spec` **forwarders** — `@ stdcall GetEnvironmentStringsA() GetEnvironmentStrings` is served.
* **assembly** implementations — ntdll's `KiUserExceptionDispatcher` is in `dispatch/*/dispatch.S`.

And a partial cohab legitimately has hundreds of still-C exports (shell32: 200). *Missing from Rust*
and *lost* are different claims; only the second is a defect.

**Rule.** After any cohab flip, audit the flipped DLL's `.spec` against the Rust — do not trust
"it compiled and booted". And when a *scanner* is the thing that failed, assume the failure is
silent: instrument the count (`Functions (N)`) against an independent list, because a parser that
drops an item does not raise, it just returns a shorter list.

**Sibling class, same lane (`tools/verify_c_signatures.py`, also gate-wired):** the model reaches
for the *pointer* flavour of a scalar with real regularity — `INT`→`LPINT` (6 fns), `SIZE_T`→`PSIZE_T`
(3), `HANDLE`→`LPHANDLE` (2 exported), `DWORD`→`DWORD_PTR` (`WPUModifyIFSHandle`, installed in the
provider upcall table). On 64-bit every one of these is the **same width**, so it links, runs and
returns the right answer — and breaks on i386. rustc cannot see it (the C prototype is not in the
Rust program) and the `.spec` records only the convention, so again the oracle must be the C source
itself. Compare **pointer depth**, not pointer-vs-scalar: a mutation test proved the first version
of that checker structurally blind to `HANDLE`-vs-`LPHANDLE` — the bug that motivated it — because
*both* are pointers.


### 9.79 (2026-07-14) Porting msvcrt's bignum float parser: a verbatim upstream bug that is provably INERT, a signed-char/WEOF trap that is load-bearing, an i386 x87 float-ABI I first read BACKWARDS, and a harness that could not see errno being *cleared*

**Context.** Wave 13 of the msvcrt-from-pieces lane ported msvcrt's own base-10 **bignum** float
parser (`dll/win32/msvcrt/bnum.h` + `string.c`'s `fpnum_parse`/`fpnum_double`/`fpnum_ldouble`) to
Rust — the first module in the lane that is a genuine *translation*, not an assembly. Wave 5 had
cancelled the "flip `strtod` from cnt-rust" plan after a probe showed cnt-rust's naive DJ-Delorie
accumulator disagreeing with msvcrt by **14 ULPs**; the honest path was always to port msvcrt's
machinery. The oracle already existed — `string.c`/`wcs.c` compile as reference TUs in
`msvcrt-difftest`, so every parsed `double` is graded on its exact **64-bit pattern**. It landed
**`strtod`/`_strtod_l`/`atof`/`_atof_l`/`_atoflt_l`/`_atodbl(_l)`/`wcstod`/`_wcstod_l`/`_wtof(_l)`/
`_atoldbl(_l)`/`__STRINGTOLD(_L)` — 15 functions on all three arches**, all bit-exact. Four durable
lessons, three of them found by *trying to break my own green*.

**1. A verbatim upstream bug can be provably inert — instrument reachability, don't assume.**
`fpnum_parse_bnum`'s round-decision has a real Wine bug: a `for(i=e-5; i>=b; i--)` sticky-scan whose
body reads `data[b]` (the lowest limb) every iteration instead of `data[i]`. I transcribed it
verbatim (the oracle is msvcrt, not IEEE) and then wrote a **negative control** — "fix" `b`→`i` and
demand a DIVERGE. *The control did not bite.* That is the exact signature of an unverified
transcription, so I did not shrug it off. I added an **exact-halfway-tie generator** (`gen_tie`:
build `(2m+1)·2^k` as a finite decimal with schoolbook arithmetic — random digits are NEVER ties, so
the entire `FP_ROUND_EVEN` arm was dead in the corpus) — still no bite. Then I *instrumented the
engine's reachability* and got the actual answer: the discriminating state is **structurally
unreachable**. `FP_ROUND_EVEN` requires a `500000000` (== `LIMB_MAX/2`) limb at position `e-4`; that
value is produced only by `bnum_rshift(1)` on an odd limb, whose remainder lands in a freshly
prepended *bottom* limb — so `EVEN ⟹ that limb is at b ⟹ b==e-4 ⟹ the scan range b..=e-5 is empty`.
Whenever the scan body *does* run, `round` is `ZERO` and the strip-loop guarantees `data[b]≠0`
(measured: **0 violations in 43,712 parses**), so buggy `data[b]` and correct `data[i]` both compute
`FP_ROUND_DOWN`. Same bytes, always. **The lesson isn't "reproduce upstream bugs" — it's that when a
negative control fails to bite, you have learned nothing until you find out WHY: either the corpus
can't reach the state, or the code is inert. Instrumentation tells you which; a comment guessing "can't
happen" does not.** (The tie-generator stays in the corpus as a live tripwire in case a future bignum
change ever creates the discriminating state.)

**2. The signed-char → WEOF trap is load-bearing and IS caught.** `strtod_str_get` does
`return *(*p)++;` on a **signed** `char` widened to `wchar_t`. `char` is signed on all three targets
(now asserted in `rust_layout_assert.c`), so byte `0xFF` sign-extends to `0xFFFF` — which *is* `WEOF`.
The parser sees end-of-input at a byte it has **already stepped past**, so the trailing-`unget` does
not put it back and the endptr lands one byte too far. Transcribed as `*p as i8 as i16 as u16`;
negative control (drop the sign extension) correctly moved `strtod` and `__STRINGTOLD_L` to DIVERGE —
and *only* the endptr-checking rows, since the value is unaffected. The wide side has no such trap
(`wchar_t` is unsigned), which is a genuine narrow/wide contract difference the harness grades.

**3. The harness could not see errno being *cleared* — the setup was part of the oracle (§9.73 again).**
`strtod_helper` clears errno on entry (`#if _MSVCR_VER==0: *_errno()=0`); its wide twin `_wcstod_l`,
written separately in `wcs.c`, does **not**. That is an observable CRT difference — a caller that sets
`errno=0`, calls one, and tests ERANGE gets different answers. The harness could not see it: `Effects`
**zeroed errno before every call**, so a redundant "clear errno" write changed nothing observable.
Sabotaging `_wcstod_l` to clear errno moved **zero rows**. Fix: seed errno with a **non-zero sentinel**
(`0x5A5A`) before each graded call instead of 0 — costs nothing (both sides get the same seed; a
function that leaves errno alone still matches) and turns "did you touch errno at all?" into an
observable. With the sentinel, the sabotage correctly reddens the whole wide family while `strtod`
(which *should* clear) stays green. **Zeroing a side channel before a differential call is a claim that
nobody clears it — and that claim was false.** This is the third variation in the lane of the same
root: a harness convenience (a stub, a cheap `get_locinfo`, a pre-zeroed errno) silently encodes an
assumption about the environment, and the assumption is itself untested until a negative control
attacks it.

**4. i386 (32-bit x86) works too — via x87 `ST(0)` — and my first read of the ABI wall was BACKWARDS.**
i386 msvcrt is built `-march=pentium` (pre-SSE — a ReactOS old-hardware-compat choice) and the whole
i686 user-mode target carries `-C target-feature=-sse,-sse2` (`.cargo/config.toml`). rustc *does* reject
some `f64`-across-a-function-boundary code on that no-SSE target with "SIMD vector type `f64` … requires
the `sse` target feature" — and I first read that as "`extern "C" fn -> f64` is impossible on i386" and
gated the whole family `#[cfg(not(target_arch = "x86"))]`. **That was exactly wrong, and the reviewer
caught it by pointing at prior art:** cnt-rust had already solved this, painstakingly, and its comments
state the rule precisely (`cabi/strtod.rs`, `core_rs/floatcvt.rs::fl`):

> the C ABI returns `f64` in the x87 `ST(0)` register — no SSE — so an **`extern "C"` `-> f64` compiles
> on no-SSE i386**. What actually hard-errors is a **Rust-ABI `-> f64`** (that one wants `xmm0`) and the
> SSE-pulling `f64`/`f32` *std methods* (`is_infinite`, `is_finite`, `abs`), whose operand crosses a
> Rust-ABI boundary by value.

So the fix is not to gate i386 out — it is three small, ABI-portable changes: (a) mark the one internal
`f64`-returning helper (`strtod_helper`) `extern "C"` so it returns via `ST(0)`; (b) replace the two
`is_infinite()` calls with bit tests that read the operand's *memory* as an integer (a plain load, no
float register); (c) replace `f64::from_bits` with a `store_bits` that writes the `u64` pattern straight
into the destination's storage. None of these is a hack: `extern "C" -> f64` and integer bit-ops are all
correct *with* SSE too, so if vibeos ever drops the pre-SSE baseline (it targets Vista-era+ hardware,
where SSE2 is universal) not one line here needs reverting. The family now flips on **all three arches**.

**The meta-lesson — and it is the important one:** a compiler error that names a feature (`requires the
`sse` target feature`) is describing *one* codegen path, not a universal impossibility. Before you gate a
whole family out "because the toolchain can't," check whether a sibling crate in the same tree already
ships the thing — the answer here was one `grep` away in cnt-rust, and the reviewer's "didn't we already
write softfloat painstakingly?" was worth more than my confident ABI paragraph. "Fail loud, don't
silently degrade" is right; but *gating a family out is itself a silent degrade dressed up as caution* if
the block was never real.

**Reusable takeaways.**
- When a **negative control does not bite**, you have found a hole in the *corpus or the instrument*,
  not a clean transcription. Instrument reachability to learn which; only then decide if the code is
  inert. (An inert upstream bug is a fine thing to ship verbatim — but *prove* the inertness.)
- A **coverage assertion must know the function's contract.** The float family splits on *how* it
  reports range errors — errno (`strtod`) vs an out-of-band return code (`_atoflt_l`). A blanket
  "require ERANGE" would have failed the return-code functions for behaving correctly; `check_fp`'s
  `require` list is therefore per-row.
- **Grep every new flip — and everything it CALLS — for arch-conditional (`_WIN64`) contracts and for
  side-channel writes (`errno`, the invalid-parameter handler) BEFORE trusting the harness**, because
  the harness only catches what its setup lets it observe.
- **A feature-named compiler error is a codegen constraint, not a law of nature.** `extern "C" -> f64`
  through x87 is the counter-example. Check for in-tree prior art before declaring a target unsupported.

### 9.80 (2026-07-14) An oracle that resolves a type wrongly reports *its own ignorance* as a defect — and the ambiguity check cannot save you when the parser is blind to the true declaration

**Symptom.** `verify_c_signatures` (§9.78's sibling) was landed with 14 baselined findings and a
note that `lsasrv LsapRegCreateKey` "looks genuinely real". On triage, **all 14 were the checker's
own bugs, and the one real defect in the tree was one it could not see.**

**Root causes (four, each a different flavour of the same mistake — a guess with no evidence):**
1. **`.idl` outvoted `.h`.** The harvest read `sdk/include/psdk/*.idl` as C. `cordebug.idl` says
   `typedef SIZE_T LPSTARTUPINFOW;` — an *RPC wire placeholder*, not the C type — so the correct
   advapi32 translation was reported as a bug. **An IDL typedef states a marshalling
   representation; the C compiler that built the .c never saw the .idl.**
2. **The true declaration was invisible, so the false one looked unanimous.** The real
   `} STARTUPINFOW, *LPSTARTUPINFOW;` is a *braced* typedef, excluded by the regex. The AMBIGUOUS
   machinery (which exists exactly to refuse guessing) fires only on *conflicting* declarations —
   **a parser blind to the true source sees no conflict and reports a confident wrong answer.**
   *A checker that can only see one source cannot tell that it is the wrong one.*
3. **A naming convention applied blind.** `^(?:LP|P)[A-Z]` read **`POINT` as a pointer to `OINT`** —
   and likewise PROCESS_INFORMATION, PALETTEENTRY, PIXELFORMATDESCRIPTOR, POLYTEXTW. Only trust the
   P/LP convention when the *remainder names a type you actually harvested*.
4. **Multi-word base types.** `typedef _W64 unsigned long ULONG_PTR` was parsed as base=`_W64`,
   harvesting **`unsigned`** as a typedef aliased to `_W64` — poisoning the whole integer tower
   (SIZE_T→ULONG_PTR→unsigned→_W64→unresolvable). Also `typedef void* PVOID` (star glued to the
   base) vs `typedef void *PVOID`: **C has no star on a base type; every star binds to a declarator.**

**The false-GREEN half, which is worse.** An unresolvable name silently returned depth 0. An unknown
C type and an unknown Rust type then both land on 0, compare **equal**, and the pair is reported as
*verified* having been checked not at all. Fixed by raising `Unknown` and counting it: unresolvable
is a **reported blind spot**, never a verdict. (`ole32` skips 308 types — now visible, previously
laundered into a green.)

**What the fixed checker found immediately:** `ws2_32 WPUCompleteOverlappedRequest(.., DWORD dwError,
DWORD cbTransferred, ..)` translated with **`DWORD_PTR`** — contradicted by the C, by `ws2spi.h`, and
by the `.spec` (`long`). 4 bytes vs 8 on amd64, on an exported SPI entry point, in the crate that had
just been landed "0 findings".

**Depth is not enough — width matters too.** `DWORD_PTR` is *not a pointer*; it is an integer that is
4 bytes on i386 and 8 on amd64. A depth comparison is structurally blind to `DWORD`→`DWORD_PTR`,
which the tool's own docstring **advertised catching**. Width cannot be compared numerically (the
headers carry both `#ifdef` branches and we do not preprocess) but it does not need to be: what makes
it a bug is that one side is **arch-variable** and the other **fixed**, which is decidable from the
name (`_PTR_SIZED`).

**Rule / recipe.** An oracle needs an oracle. **Mutation-test every checker in both directions** —
a "must catch" table of the bugs it exists for (catches false NEGATIVES / a checker gone blind) and a
"must not fire" table of correct code (catches false POSITIVES). Only the first table's absence lets a
tool *look* healthy while detecting nothing. Both live in `tools/test_verify_c_signatures.py`, wired
into `vibeos-cli check-tooling`. And **always print coverage next to findings**: "0 findings" from an
oracle that skipped everything is otherwise indistinguishable from one that checked everything.

**Triage, don't bless.** The 9 lines left in `tools/lint-baseline/verify_c_signatures.txt` were each
read against their C; the verdicts are in the tool's docstring. Two of them are **the C being wrong**:
`LsapRegCreateKey`/`LsapRegOpenKey` declare `OUT HANDLE KeyHandle` while every caller passes `&handle`
and `ZwCreateKey` takes `PHANDLE` — the C only compiles because `void*` implicitly converts to
`void**`. **Do not "fix" the Rust to match under-typed C.**

### 9.81 (2026-07-14, BOTH FIXED) The two staleness oracles: an mtime rule that false-REDs on a codegen-neutral edit, and a `ninja -n` oracle that is blind on every arch

Both live in `tests/qemu/cli/provenance.py` and both guard the same thing — "did the staged
binary come from the current sources?" — the class that false-greened arm64 six times
(fastfat→ramdisk→pl011→gdi32→user32→advapi32, §9.6c.1). Found while gating a one-word ws2_32
signature fix; **neither was found by the gate reporting a bug, both were found by the gate
reporting the WRONG THING and being chased down.**

#### (a) `cohab_staleness_reasons` false-REDs on a no-op-codegen edit — real, narrow, still there

**Symptom:** `artifact-check(boot media missing or stale)` → `ws2_32.dll: stale — ws2_32-rust
has .rs newer than the built binary (cohab flip not rebuilt; ...)`, on every arm64 build, while
the build was in fact perfectly up to date.

**Mechanism:** the rule is `artifact.mtime >= newest .rs mtime` — a *proxy*. The ws2_32 fix
(`WPUCompleteOverlappedRequest`: `DWORD_PTR`→`DWORD` on an unimplemented stub whose params are
unused) is **codegen-neutral on arm64**: cargo rebuilt, emitted a **byte-identical** `.a`,
cmake's `copy_if_different` correctly did NOT touch the staged copy, so nothing relinked (there
was nothing to relink) and `ws2_32.dll` stayed older than the `.rs` **permanently** — until an
unrelated ws2_32-rust change forced a relink. Any semantically-null-but-textual edit does this.

**A false RED is not the safe direction.** It is the mirror of the false green the check exists
to catch, and it teaches people to `touch` binaries or ignore artifact-check — which is how the
false greens got in.

**FIXED** by splitting the chain at the copy, which is where the mtime semantics change:
```
.rs --cargo--> <t>/release/lib<c>.a --copy_if_different--> lib<c>.a --link--> artifact
               [A: touched on EVERY cargo run]              [B: touched only on CHANGE]
```
  * **A.** cargo's own output >= newest `.rs`  => the crate really was rebuilt (catches the
    original never-built-for-this-arch bug: gdi32/user32/advapi32…).
  * **B.** artifact >= the COPIED staticlib   => the link consumed the current `.a` (catches
    the other half: crate rebuilt+changed, but the .dll wasn't in the staged target set).
Both compare files whose mtime moves exactly when it should, so an identical rebuild is silent
while a real one is still caught. Crates with NO staticlib in their chain are cargo-*linked*
modules (ntoskrnl-cargo/win32k produce the binary directly) — "staticlib absent" means
"different build shape", not "never built"; reading it as the latter reported ntoskrnl.exe and
win32k.sys as never-compiled on a green tree. They keep the old rule. Pinned by 5 tests in
`test_cli_units.py` (`test_cohab_*`), including the codegen-neutral case as an explicit
must-NOT-fire.

**Blind alley — do NOT "fix" it with `ninja -n`.** That swaps a false RED for a false GREEN:

#### (b) `_ninja_has_work` returned False even when a rebuild WAS pending — on ALL arches, so **editing any C file and running smoke booted the stale ISO and called it a pass**. FIXED.

**Verified:** with a genuine pending rebuild (proved by running real `ninja`, which compiled
ws2_32-rust and relinked), `_ninja_has_work` returned **False** for `dll/win32/ws2_32/ws2_32.dll`
AND for `ntoskrnl` (arm64) AND for **`livecd`** (amd64 — i.e. the x86 tier-1 target).

**Mechanism (structural, not a parsing bug):**
```
$ ninja -n dll/win32/ws2_32/ws2_32.dll
[0/2] Re-checking globbed directories...
[1/2] Re-running CMake...          <- and NOTHING else, ever
```
CMake's `CONFIGURE_DEPENDS` globs install a glob-verification edge that is dirty **by design on
every build** (that is how re-globbing works). Under `-n` no command executes, so ninja must
assume that edge changes the cmake stamp → the regen edge is dirty → and ninja **stops planning
there**, because regenerating `build.ninja` invalidates the rest of the graph. It never
evaluates the targets you asked about. `_ninja_has_work` filters `Re-running CMake` /
`Re-checking globbed directories` out as "not real work" and returns False = clean.

Running `cmake .` or `ninja build.ninja` for real **first does not help** — the glob edge is
always-dirty by construction, so the next `-n` stops in exactly the same place. The docstring's
premise ("cmake re-runs ... so the dirty set is evaluated against the updated graph") is simply
not what `-n` does. **`ninja -n` cannot be a staleness oracle in a CONFIGURE_DEPENDS project.**

**Consequence — this is the important part, and it is much worse than "a dead net".** Look at
where it sat in `check_staleness`. Tier 2 only runs when the git fingerprint moved, and it then
decides *whether the change was build-relevant*. `rust_src_hash` is scoped to `sdk/lib/rust`.
So the `ninja -n` call was **the only thing covering every non-Rust build input** — C, asm,
cmake, `.spec`, `.rc`, mkramdisk. With it blind, the chain ran:

    edit a tracked, compiled .c  ->  git fingerprint moved  ->  tier 2
      rust_src_hash unchanged (it is C)      -> no reason
      _ninja_has_work                        -> False (blind)
      boot media sha unchanged (not rebuilt) -> no reason
    -> no reasons at all -> "clean-diff (non-build change)" -> FRESH -> smoke boots the STALE iso

Measured on the real repo, not reasoned about: editing `dll/3rdparty/dxtn/txc_compress_dxtn.c`
gave `fresh=True, reasons=['clean-diff (non-build change)']`. A C source change was classified
exactly like a doc edit. The North Star's "smoke refuses to boot stale media" was defeated for
**every C-side change in the tree**, and had been for as long as the oracle existed.

**FIXED** with `build_src_hash()` — a git content fingerprint of everything that can change the
boot media, stamped at build time and compared in tier 2. Content is the only thing that can
answer this: an mtime cannot tell "not rebuilt" from "rebuilt to identical bytes" (that is (a)),
and `-n` cannot see past the regen edge. Two design points worth keeping:
  * **It is an EXCLUDE-list (`docs/`, `*.md`, `.github`, `.claude`), never an include-list.**
    An unknown path must land on the build-relevant side: over-including costs a no-op rebuild,
    under-including boots stale media and reports a pass. `tests/` and `tools/` are deliberately
    build-relevant — `archcfg.py` picks the arm64 staged set, `qemu/mkramdisk.sh` assembles the
    ramdisk, and `tools/libschily/*.c` is compiled.
  * **An absent stamp field is LOUD, not clean.** `build_src_hash` defaults to `""` so old
    stamps still load; `""` reports "stamp predates build-source fingerprinting — run `build`
    to re-stamp" rather than comparing equal to nothing and reading as fresh. A defaulted field
    that silently means "verified" is the same shape as everything else in §9.80/§9.81.

`_ninja_has_work` was **deleted**, not left unused, with a tombstone comment: a plausible helper
named "has work" is exactly what the next person reaches for.

**The fixture was lying too.** `staleness_repo` had no `.gitignore`, so its own stamp file (in
`build-i386/`) showed up as untracked and moved every fingerprint — which meant those tests never
matched tier 1 and always fell through to tier 2, where the blind oracle waved them through.
`test_staleness_fresh_cold` was **passing because of the bug**. A fixture that does not model the
real repo turns its tests into a green that means nothing.

**Rule.** When a gate check fires, find out whether it is *right* before you satisfy it — and
when you replace an oracle, test the replacement on a KNOWN-BAD input, not just a known-good
one. The `ninja -n` "fix" passed the benign case perfectly and silently failed to catch a real
stale cohab injected on purpose; only the deliberate known-bad probe exposed it. Same lesson as
§9.80's mutation tables, one layer down.

### 9.82 (2026-07-14) The winlogon 0x4C "init-ordering race": the owed fix was already in the tree — the hypothesis was wrong, and the evidence never fit it

**The owed item** (MEMORY.md + §9.36): *"winlogon 0x4C init-ordering race (all arches, ~3/20).
kernel32's DllMain `_snwprintf("%wZ")` RACING basesrv's population of BASE_STATIC_SERVER_DATA
(csrss accepts the connect before the CSR shared section is fully populated). Real fix owed:
publish-last ordering in csrsrv/basesrv (populate the shared section BEFORE accepting connects)."*

**That fix is already implemented, and was the whole time.** Read the live Rust (the C is inert —
both `csrsrv` and `basesrv` are full cohab flips into `smss-csrss-rust`):

* `csrsrv/init.rs` `CsrServerInitialization` — **step 5** `CsrParseServerCommandLine` (which calls
  `CsrLoadServerDll` → basesrv's `ServerDllInitialization`) runs at line 1095; **step 7**
  `CsrApiPortInitialize` — which *creates the API port* — runs at line 1107. No client can connect
  before the port name exists.
* `csrsrv/server.rs` `CsrLoadServerDll` — calls `init_proc(ServerDll)` **synchronously** and only
  then, `if Status >= 0`, publishes `CsrSrvSharedStaticServerData[ServerId] = ServerDll->SharedSection`.
* `basesrv/init.rs` `BaseInitializeStaticServerData` — its **last statement** is
  `(*LoadedServerDll).SharedSection = BaseStaticServerData` (the C's `/* Finally, set the pointer */`).

So it is publish-last at three nested levels. Implementing the "owed fix" would have been a no-op
edit to already-correct code — and the effort would have been spent looking for a race that the
control flow forbids.

**The evidence never fit a race either.** The live tripwire's own comment records the observation:
`WindowsDirectory.Buffer` reads as a **kernel VA `0xFFFF8000050D4CB0`, *identical across hits***.
A race produces *varying* garbage; an identical value across boots is a deterministic write or a
deterministic misread. (§9.36's other datum, `Buffer == 0x73`, is string DATA read as a pointer —
that is the offset-drift smell, §9.41's class, not a race either.) The word "race" was doing the
work that evidence should have been doing.

**Leads checked and closed while here** (all were plausible, none is the bug):
* **Struct layout drift** for `_BASE_STATIC_SERVER_DATA` between writer and reader — both
  `ntdll-sys` and `winsrv-sys` declare it identically, and both sides use `ntdll-sys` anyway.
* **`CSR_SERVER_DLL.SharedSection` at the wrong offset** — the field really is `#ifdef CSR_DBG`
  dependent (`csrsrv.h` C_ASSERTs 0x3C with, 0x38 without, on 32-bit), and the Rust struct carries
  `NameTable` unconditionally. But `csrsrv.h:18` does `#if DBG → #define CSR_DBG` and the build
  passes `-DDBG=1`, so CSR_DBG is ON and the Rust layout is right. (Two comments in `basesrv/init.rs`
  quote 0x3C and 0x38 respectively — they are describing the two C_ASSERT branches, not disagreeing.)
* **Shared-section view delta** — `CsrSrvAttachSharedSection` maps into every client at the same
  fixed base with `SEC_NO_CHANGE`, so server-side pointers are valid client-side; there is no delta
  to get wrong.

**Status: NOT REPRODUCING — downgrade from "open bug with a fix owed" to "re-open on evidence".**
The mitigation prints `[kernel32] BASE_STATIC_SERVER_DATA CORRUPT: …` / `… snapshot CORRUPT (raced
basesrv?): …` on every hit and reroutes to fallback strings. Measured 2026-07-14:
  * **20/20 arm64 boots green, ZERO tripwire hits** (`smoke --arch arm64 --repeat 20`, the
    sanctioned parallel-TCG repro path). Against the recorded rates that is
    P(0 hits | 1-in-10) ≈ 0.12 and P(0 hits | 3-in-20) ≈ 0.04 — so the face is either gone or far
    rarer than recorded.
  * **Zero hits across every retained log on the box** (1254 run dirs).
  * Zero `0x4C` / `0xC000021A` in the 20-boot run.

**The negative is only worth anything because the detector was proven visible first.** Two checks,
both necessary: the same arm64 serial logs carry user-mode `DbgPrint` output (`[EXPLORER] _tWinMain
entered`, `[WIN32K] shell ready`), so a user-mode print from kernel32 *would* reach the log; and the
marker appears **exactly 20 times** in the aggregate `full.log`, proving all 20 boots' serial was
captured and not just the first (per-boot dirs hold only screenshots — the serial is aggregated).
Without that second check, "zero hits in 20 boots" would really have been "zero hits in 1
observable boot". Confirm a detector can be *seen* before reading its silence as absence — the
§9.80/§9.81 lesson, one more time.

**Do NOT implement the owed "publish-last" fix.** Re-open only on a fresh tripwire print, and treat
the printed fields (not the word "race") as the starting evidence.

**Rule.** An owed item that names its own fix ("just do X") is a hypothesis wearing a fix's clothes.
Re-read the live code before implementing X: here X was already done, three times over, and had been
since before the item was written. And when a bug note says "race", check whether anything in the
evidence is actually *non-deterministic* — "identical across hits" is the opposite of a race.


### 9.83 (2026-07-14) The printf probe: a third-party C-vs-C differential that found a KERNEL HANG before writing any new Rust — and two ways a compiled-but-never-called oracle lies

**Context.** Wave 14 of the msvcrt lane needed an assemble-vs-port decision for the printf family:
is cnt-rust's already-Rust `streamout` engine close enough to msvcrt's `pf_printf_a` to sit behind
the msvcrt exports? Direct grading is impossible (linking cnt-rust's `cabi` into msvcrt-difftest
collides 200+ `#[no_mangle]`s with the subject), so the probe compared **two compiled C references
by transitivity**: ReactOS `printf/streamout.c` compiled under cnt-difftest's exact proven recipe
(flat va_list + bit-identical `floor/log10/pow` model — the configuration cnt-difftest proved
byte-equal to the Rust engine) vs the genuine msvcrt engine already compiled inert inside
msvcrt-difftest's wcs.c reference TU. `sdk/lib/rust/msvcrt-difftest/tests/printf_probe.rs`.

**Finding 0 — the probe found a real kernel hang before comparing anything.** The first run hung in
`fl::exp10_floor`'s C twin: for ±Inf the exponent-correction loop `while (pow10(e+1) <= f) e++;`
never terminates (`pow10(309)` saturates to +Inf; `Inf <= Inf` holds). The Rust engine shipping in
the KERNEL (`cabi/streamout.rs` → `fl::exp10_floor`, amd64/arm64 float path) had the identical
loop, unguarded — **any `%f/%e/%g` of an infinity through the kernel's Rust streamout was a silent
infinite loop**. The genuine C only "worked" via UB (`(int)floor(log10(inf))` → garbage exponent →
digits later overwritten by the `#INF` special path). Fix: `exp10_floor` returns **308**
(`DBL_MAX_10_EXP`, and exactly the value NaN already fell out with) for `biased == 0x7ff`, same
guard in every `cnt_log10` C-twin copy (3× cnt-difftest, 1× msvcrt-difftest), plus ±Inf/NaN rows in
cnt-difftest's `diff_streamout_float` as the permanent tripwire. Lesson: **a differential probe's
first product is often a crash/hang in the harness path itself — chase it before dismissing it as a
harness artifact.** This one was the shipped engine's bug reflected in its bit-identical twin.

**Finding 1 — the oracle answered with the wrong LIBC and only style told us.** The probe first
called `ref__snprintf`, whose body is `vsnprintf(...)` — a symbol wcs.c never *defines* under
`_MSVCR_VER=0`, so inside the harness it bound to **glibc**, and the whole "msvcrt" column was the
host libc. No crash, plausible outputs. The tells were stylistic fingerprints: `%p` → `(nil)`,
2-digit exponents (`e+00`), `%I64d` parsed as width-64, `%zd`/`%jd` accepted. Fix + permanent
guard: an **oracle-honesty canary** — `%I64d` must print `-5` (glibc right-pads to width 64);
HARD-FAIL the probe before any verdict if it doesn't. Generalizes §9.66 (ref-integrity): *a
reference that links is not yet a reference; assert a behavior only the intended engine has.*

**Finding 2 — Wine/ReactOS printf code is structurally UNCALLABLE when compiled under the SysV
va_list.** Second attempt called `ref__vsnprintf` via a varargs trampoline → SIGSEGV inside
`arg_clbk_valist`. The engine forwards `&valist` as `va_list*` — coherent only where `va_list` is a
plain pointer (all Windows targets). Under SysV's `__va_list_tag[1]`, a `va_list` *parameter*
decays to `tag*`, so `&valist` is `tag**`, and the callee's `va_arg(*valist, T)` reinterprets the
pointer variable itself as the register-save struct → wild read. (Real Wine avoids this on Linux
with `__ms_va_list`; this tree never needed to — it only builds for Windows targets.) So the
harness's ref wcs.c printf entries were *compiled* fine and *inert* fine, but no in-process caller
could ever legally drive them. Fix: compile wcs.c a SECOND time in a `pfflat_*` namespace under a
**flat `char*` <stdarg.h>** (msvcrt-difftest/refstub_printf/) — which is precisely the Windows-x64
va_list model, so the probe's drive is also the target-faithful one, and both engines consume the
SAME hand-packed 8-byte-slot buffer. Mechanics that made it work: prelude2 = prelude with only the
wcs.c-owned symbols retargeted `ref_→pfflat_` (string.c/ctype cross-calls stay `ref_*`); a separate
`cc` `out_dir` (cc hashes the SOURCE path for object names — two compilations of one file in one
OUT_DIR silently overwrite and double-link); the flat stdarg.h must also define `__gnuc_va_list`
(glibc headers consume gcc's <stdarg.h> contract).

**The verdict the probe produced (111 MATCH / 27 DIVERGE, decision-grade):**
- Integer / string / char / pointer formatting: **streamout ≡ msvcrt** (75/78 rows; deltas are
  `%#x` of 0 → msvcrt `0` vs streamout `0x0`, and `%jd`/`%td` unknown to streamout).
- Float formatting: **structural fidelity gap, msvcrt wins**: exact binary expansion at high
  precision (`%.30f` → `0.100000000000000005551115123126` vs zero-padded), denormals
  (`4.940656e-324` vs `0.000000e-308` garbage), `-0.0` sign kept, `%g` trailing-zero stripping +
  correct e/f-form threshold, `1.#INF00`-style precision padding, and a real `%a` hex-float
  implementation (streamout's `%a` is a FIXME falling through to `%f`).
- Decision: **PORT msvcrt's `printf.h` engine** (the wave-13 `bnum.rs` extraction was exactly the
  down-payment — parser and formatter share the limb engine); do NOT assemble streamout under
  msvcrt exports — it would silently downgrade the DLL's float output, violating never-weaken.
  The kernel keeps streamout (faithful to ReactOS crt by construction; its quirks are its C's).

### 9.84 (2026-07-15) `NET_FLAKE` that is not a flake: the net suite times out at 600s having never booted, because another lane's `farm` owns every host CPU lease

**Symptom.** A gate with `✓ build ×3  ✓ lint ×3  ✓ smoke:i386  ✓ smoke:amd64  ✓ smoke:arm64` and
`✗ smoke:net(NET_FLAKE, 3 attempts)` — `15 errors in 600.39s`. Every one of the 15 net tests errors
with an identical flat `Failed: Timeout (>600.0s)`, **including `test_boot_with_nic`, the first one**.

**It is not the network, and it is not your change.** The tell is in the captured setup output:

```
[hostlock] waiting for host CPU lease (boot:amd64-uefi-net)...
ERROR tests/qemu/test_net.py::test_boot_with_nic - Failed: Timeout (>600.0s)
```

The suite spent its entire 600s **queued on the hostlock (4 leases)** and never booted a VM. When
another worktree is running `vibeos-cli farm` (an all-night, whole-host soak) plus its own smokes,
load sits at ~24 and the leases never free. The gate's own retry (3 attempts) cannot help: it
re-queues into the same saturated lock.

**How to tell starvation from a real net regression, in ten seconds:**
* **All 15 error** (not fail) with the **same** 600s number, first test included → fixture never
  ran. A real regression fails a *specific* test with a *specific* assertion.
* `✓ smoke:i386/amd64/arm64` on the **same media** → the OS boots fine; only this lane starved.
* `uptime` (load ≫ cores) and `./vibeos-cli ps` (`slots: N/10 in use`, OWNER paths pointing at
  *other worktrees*) name the culprit.
* `grep hostlock` the pytest output. That line is the whole diagnosis.

**What to do.** NOT `--allow-stale`, not a re-gate (it will starve again), and above all **never
kill the other lane's QEMUs or farm** — they are someone's deliberate soak, and broad `pkill -f
qemu` matches your own process tree (AGENTS.md). Park the work committed-but-unpushed and wait:
`while kill -0 <farm-pid>; do sleep 120; done; ./vibeos-cli gate`. A gate that is red for a reason
outside the tree is not a green you may push on the strength of "it's probably fine" — the ×3-green
push rule exists exactly so that judgement is not yours to make at 1am.

**Related, and the reason this bit at all:** the standing rule is that the flake farm runs only
when no other agents are working. Two lanes each politely declining to soak, and one lane not
declining, produce this. If you start a farm, expect to be the reason another lane's gate is red.
