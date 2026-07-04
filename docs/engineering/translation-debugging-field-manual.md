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
**Source:** 2026-07-04 dispatcher audit + constant sweep (`docs/engineering/sync-instrumentation-design.md` §8).

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

---

## LLVM captures(none) / ud2 / codegen-unit miscompiles

The partial-Rust kernel trips a family of LLVM optimizer bugs where interprocedural analysis sees enough to optimize but not enough to be correct; these entries cover the symptoms, the `lto=false`+`codegen-units=256` firewall, and the workarounds that are (and aren't) still load-bearing.

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

---

## rustconv translation-output bug catalog

The specific ways the LLM C→Rust output is wrong — each entry is a distinct defect class, how it manifests at compile/link/runtime, and the cheapest oracle to catch it before it ships.

### Wrong INLINED flag/constant literal (comment names the right flag, literal is a different one)
**Symptom:** `SetFlag(f, 0x2 /* IRP_CONTEXT_FLAG_TOP_LEVEL */)` (0x2 is MORE_PROCESSING; TOP_LEVEL=0x10); `PALETTE_AllocPalette(0x4 /* PAL_BITFIELDS */ ...)` (PAL_BITFIELDS=0x2, 0x4=PAL_RGB); `RtlRaiseStatus(0xC0000002 /* STATUS_DATATYPE_MISALIGNMENT */)` (0xC0000002=NOT_IMPLEMENTED; MISALIGN=0x80000002). The LLM hardcodes a literal it labels with the intended name but the value is wrong/placeholder. Compiles & links clean; pure runtime/semantic. cdfs alone had 7 (TOP_LEVEL/TOP_LEVEL_CDFS/DISABLE_POPUPS/CDFS_NTC_IRP_CONTEXT/FSP_FLAGS/CCB_FLAG_DISMOUNT/CDFS_RESIDUAL); the codebase sweep found more in scsiport (LUNEX, HIGH), ntoskrnl se/srm (OBJ_OPENIF), win32k dibobj, disk, cabinet, smss.
**Oracle:** `tools/scan_inline_flag_literals.py <crate/src/*.rs> --headers <driver C dir>` — **`verify_constants.py` is BLIND to this** (it only checks `const NAME = VALUE;` defs, not bare literals). The tool is tree-sitter/AST-based (regex can't pair a literal with its naming-comment across nested `|`/parens/casts — not a regular language); catalog = kernel headers + sdk/include/{xdk,ndk,psdk} + driver `--headers`. It's a TRIAGE AID not a verifier — residual FP classes need a human eye: flag-MASK unions (`A|B|C // N1|N2|N3` where the union is right but order differs), informational args (KWAIT_REASON `Wr*`, WPP TRACE_LEVEL), and value-tables labeled by index (COLOR_*, suppressed).

#### Sub-class: correct literal for the WRONG named constant — a range BOUND, not a flag (scanner-invisible)
**Symptom (2026-06-21, user32-rust):** `IS_ATOM(x)` coded `v > 0 && v < 0xC000` with a comment "MAXINTATOM is 0xC000". The C macro (`user32p.h`) is `(x > 0) && (x < 0x10000)` — the upper bound is **0x10000** (high 48 bits clear), NOT `MAXINTATOM`. MAXINTATOM (0xC000) is the *base* of registered-class atoms; they start at 0xC000 and increment. So once the shell registers ~100 window classes, a real class atom (e.g. `0xC06A`) is `>= 0xC000` and `IS_ATOM` returns false → it flows into `RtlInitUnicodeString`, whose `wcslen` derefs the tiny integer → **AV 0xC0000005 in explorer at "taskbar: creating tray window"**, crash-looping the shell post-`shell ready` (so the smoke gate stays green; only the *desktop* is broken). Two copies (`window.rs:42` `User32CreateWindowEx`, `class.rs:111`) had the bug; `message.rs:224` had it right (0x10000). Fixed in `67c9e5849c7`.
**Why the literal-scanner misses it:** the literal `0xC000` *matches* its naming comment (`MAXINTATOM` really is 0xC000) — so `scan_inline_flag_literals.py` sees a consistent literal/label pair and stays silent. The defect is one level up: the translator picked the **wrong named constant** for an *unnamed* numeric bound in the source (the C `0x10000` had no symbol). This is the residual "correct value, wrong concept" class. **Tell:** a range/threshold compare (`< CONST`, `>= CONST`, `>> N`) where the C original used a bare numeric literal — re-derive the bound from the C, don't trust a plausible-looking named-constant comment. **Cross-arch:** `user32-rust` (and most usermode `*-rust` crates) are cohab'd on all 3 arches, so a usermode-DLL translation bug breaks amd64/i386 identically — it just hides *past* their `shell ready` gate. To see it, boot past the gate and watch for `[UEXC]`/`PspExitThread ExitStatus=0xC0000005`.

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

### 9.13 ✅ The Mm WS-pushlock SELF-DEADLOCK — PROVEN LIVE 2026-07-04 via the sync-instrumentation stack (2-line trace, ~30 min, no GDB archaeology)

The amd64 service-startup wedge's hypothesized root cause (§9 wedge notes /
MEMORY open-item: "a page fault inside the WS-locked teardown re-acquires the
working-set lock") is now a **filmed mechanism**, captured the same night the
instrumentation landed (`860452569ef` + the WS-lock tap):

**What the census showed (every boot, even green ones):** 3–6 freshly-created
processes (eventlog.exe, svchost, rundll32, setup.exe…) each have their FIRST
thread permanently parked acquiring their own `EPROCESS.Vm.WorkingSetMutex`
(+0x368 amd64; = ActiveProcessLinks+0xd8 + 0x290). Those processes never
execute one user instruction — **eventlog.exe is silently dead on every boot**;
historically services.exe wedged waiting on it via LPC, which is why the wedge
read as "service-startup" and was probabilistic (~50%): the stranding is
deterministic, the boot only died when a boot-critical thread was among the
caught. (11/11 boots green after the 2026-07-04 constants campaign — the LPC
STATUS/ViewUnmap fixes apparently de-fatalized the propagation — but the
stranding itself persisted, invisible until the census.)

**The 2-line proof** (`vibeos-flightrec obj=<stuck WS pushlock>` over the
temporary `ws-lock-EXCL` tap on `MiLockProcessWorkingSet*`):
```
seq=170109  ws-lock-EXCL   thr=0xfffffa80012e4040  data=1   ; MiLockProcessWorkingSetUnsafe
seq=170110  pushlock-park  thr=0xfffffa80012e4040           ; SAME thread parks on the SAME lock
```
And the parked thread's kernel stack (symbolicated via raw stack scan —
`dump-thread.py`'s pinned offsets have rotted; use the debug block):
```
MmAccessFault+356 → MiLockProcessWorkingSetShared+47 → ExfAcquirePushLockShared+1077
  → KeWaitForGate → KiSwapThread   (with MiDeletePte+788 on the stack)
```
I.e. **inside the WS-EXCLUSIVE(Unsafe) section, `MiDeletePte`-path code touched
a pageable address → #PF → MmAccessFault re-acquires the WS lock SHARED →
self-deadlock** (pushlocks are non-recursive). Matches the earlier csrss
analysis (`MiDecrementShareCount+0x71` reading a pageable prototype PTE);
upstream avoids it by running this under the PFN spinlock at DISPATCH
(non-faultable). Suspect list unchanged: MMPFN accessor layout /
`prototype_pte()`/`pte_address()` mis-decode sending the code down a
pageable-deref branch, or genuinely pageable prototype PTEs that upstream's
IRQL discipline tolerates.

**Reusable recipe (the point of this entry):** for ANY stuck-thread report —
1. Boot with `vibeos-cli gdb start <arch>`; wait for `[CENSUS] STUCK` (the
   wedge self-reports; the report already contains the lost-wakeup vs
   holder-stuck verdict + recent events).
2. `vibeos-cli gdb inspect <run> "raw vibeos-synctrace"` → stuck object list.
3. `... "raw vibeos-flightrec obj=0x..."` → who locked/signalled last.
4. `... "raw vibeos-threads gatewait"` → all parked threads, offset-free.
5. Stack-scan the parked KTHREAD (KernelStack via debug block) + `info symbol`.
Total cost tonight: ~30 min. The identical conclusion previously took days.

**Fix status:** NOT yet fixed — mechanism proven, faulting site bracketed
(MiDeletePte path under `MiLockProcessWorkingSetUnsafe`). Next: pin the exact
faulting VA/instruction (trap frame lives in the parked thread's stack above
MmAccessFault) and check the MMPFN accessors, then either fix the layout bug
or make the teardown path non-faultable like upstream.
