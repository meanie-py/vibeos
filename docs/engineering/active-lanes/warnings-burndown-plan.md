# Rust warnings burndown (lane: `warnings-burndown`)

**Status:** OPEN (opened 2026-07-16). Worktree `vibeos-warnings-burndown`, branch
`warnings-burndown`. Follow-on from the closed `rust-warnings` lane (which fixed the 3
dangling-pointer UAFs + landed `scan_dangling_transmute`); this lane burns down the *rest*.

**Deliverable:** the Rust workspace compiles with a dramatically smaller warning surface,
the genuine-signal warnings are audited and fixed, and — critically — the fixed classes
**stay** fixed (no blanket re-suppression). Incremental, per-crate, each cluster gated ×3
and landed on its own so conflicts stay localized and every step is revertible.

## RESUME HERE (updated 2026-10-17)

**State.** The 2026-10-17 census read **2,349 sites (48 UB, 1,072 LOGIC, 1,229 NOISE)**,
down from 7,215. It covered 695 fresh units; the only stale one was reactos-link, a host tool
whose mtime the merge bumped. `static_mut_refs` and `improper_ctypes_definitions` read 0 and are
now in `rust_ub_deny` next to `invalid_value` and `deref_nullptr`. Re-run the census with
`python3 tools/rust_warning_census.py build-i386 build-amd64 build-arm64`. Details are under
"2026-10-17" below.

**Final census, 2026-10-17 morning** (3-arch build of d7f6514670b): **2,014 sites, 0 UB,
913 LOGIC, 1,101 NOISE.** `clashing_extern` is 889, of which 880 are shell32's.
**Open, for the control-plane lane:** a unit the artifact store restores can carry diagnostics
older than its own metadata. conutils' fingerprint JSON has `declared_features: ["USE_CRT"]`,
yet its `output-lib` replays "no expected values for `feature`". Every file in the unit is
hardlinked (links=2) and carries the 06:35 restore time, against a Cargo.toml edited at 05:56.
That is why the census still shows conutils' 14 and ws2_32's 1 `unexpected_cfgs`, which a
plain `cargo check` reports as 0. The census now treats a manifest newer than the build as
stale, but that cannot catch this case, because the restore stamped the unit newer than the
edit.

**Later on 2026-10-17** (after that census; landed as b8f5a91889d, dd8d8701261 and the
`unexpected_cfgs` batch):
- **`improper_ctypes` 48 -> 0, now denied.** All of the UB tier is denied in `rust_ub_deny`.
- **More `clashing_extern` cleared.**
  - cdrom 72 -> 0, plus about 75 external-API clashes in 12 crates.
  - Remaining outside shell32: menuband's 4 come from the `SIZE_T = usize` in shell32's
    shellcom.rs, and the i386-only pairs are the same width (benign).
  - shell32's 880 stay with the shell32-com lane.
- **New gate `scan_fnclass_cc`:** a callback declared through an NTAPI function class and defined
  cdecl. It found 4 real i386 bugs.
- **`unexpected_cfgs` is LOGIC now.** It found desk's `unicode` bug. The remaining 4 are
  crt-shared (msvcrt-homebrew's) and wdmaud_drv (the audio lane).

**Owed, roughly in priority order:**
- **The rest of `clashing_extern`:** shell32 (880, the shell32-com lane's), and menuband's
  shellcom pair. `extern_to_use` gaps:
  - it does not collapse two declarations in the SAME file under different inline mods
    (rtl-strings' `cabi/mod.rs` was done by hand);
  - it does not see that a definition is cfg-gated (cdrom's i386-only `KeQueryTickCount`
    extern).
- **LOGIC smalls:** `unused_assignments` (29), `unreachable_code` (6), `useless_ptr_null_checks`,
  `unused_comparisons`, `unused_must_use`, and the obname/psnotify fn-pointer comparison.
- **Noise:**
  - `non_snake_case` (334), `unused_labels` (172), `unused_unsafe` (142), `unused_imports` (97),
    `dead_code` (90).
  - `tools/apply_rustc_suggestions.py --skip-crate` keeps a sweep off hot crates. Rustc offers
    no machine fix for most of what is left.
  - The 12 float `unnecessary_transmutes` are msvcrt-homebrew's.
- **Islands:**
  - the committed mspaint and msctfime islands have drifted far from what the generator now
    writes (thousands of lines). A real regen is its own job.
  - bindgen declares a function-class declaration `extern "C"` on i386, which is wrong for
    NTAPI classes. `scan_fnclass_cc` covers the Rust side.
- **Upstream drift:** explorer #9332, oleaut32 VarMod and serializeBMP.
- **pcix arbiters**, and the preserved upstream-C bugs under "Owed follow-ups".

**How this lane works** (so a fresh session doesn't rediscover it):
- Haiku 5.5 subagents (`model: haiku`), at most 2-3 at a time.
- Agents are read-only. They write audits or JSONL edits to the scratchpad; I review the edits
  and apply them centrally with `tools/apply_agent_edits.py`.
- Verify an agent's "UB" or "bug" claim against the code before acting on it. On 2026-10-11 the
  external-symbol grader's 14 "UB" calls were all benign or latent once checked: it missed a
  `cfg` split, a trampoline, and a C-side truncation. By contrast the FlagOn-truncation and
  constant audits were right, and found live bugs.
- Landing (since 2026-10-16): commit, run `./vibeos-cli check` and the tests for what you
  touched, then message the `vibeos-integration-f2` session with
  `merge warnings-burndown @ <sha> — <one line>`. Keep working; it merges, gates and reports
  back (docs/engineering/active-lanes/integration.md). No `land`, and no warm builds ×3 just
  to submit. Don't touch the tree while a smoke you started is running.

## 2026-10-17 — UB tier closed except mspaint; clashing_extern 1,325 -> 1,031

- **UB:**
  - crt-startup-rust's `static mut` refs became `&raw mut`.
  - rapps and drwtsn32 had Rust-only helpers declared `extern "C"` with Rust types. They are
    plain `unsafe fn` now.
  - Both lints are denied now.
- **`clashing_extern`.** `tools/extern_to_use.py` grew several things:
  - a real mod-tree walk (`#[path]`, inline `mod x {}`);
  - `pub use` for a pub declaration;
  - generated-island and bindgen-declaration skips;
  - `--canonical` for externals.

  It also learned that a declaration's attributes travel with it. menuband's orphaned
  `link_name` would have rebound an i386 import. 83 external APIs (254 declarations) were
  graded by two Haiku agents and checked against the headers; none passed a wrong width. Real
  bugs fixed:
  - kernel32's `ExitThread`/`ExitProcess` declared as returning;
  - cdfs's FSP dispatcher discarding a status;
  - private struct copies: user32 DDE, acpi `POWER_STATE`, menuband `IUnknown`, ole32
    `GUID`/`BITMAP`.
- **`crate-repair` `c_integer_widen`:** under an arithmetic, bitwise or shift operator, `as _`
  cannot be inferred, so the rule traded an E0308 for an E0282 that its pass guard kept. It now
  names the other operand's declared type, or declines with `uninferable_operand`. Comparisons
  infer, and keep `as _`.
- **`scan_version_vs_c`** now also reads ALL_CAPS constants in a dead branch, counted only where
  the Rust decides on them. This was the "flag test is invisible" follow-up. It found:
  - cdfs refusing `FILE_OPEN_REQUIRING_OPLOCK` (WIN7-only);
  - classpnp parsing the WINBLUE LBP parameter block on a WIN8 build;
  - pnpmgr answering `PNP_PROPERTY_CONTAINERID` (WIN7) on a WS03 kernel;
  - cdfs's `cfg!(not(reactos))`.

  Field names were tried and dropped, because every hit was an eager pure read. classpnp's
  `#[cfg(any())]` stays: a named cfg would need check-cfg plumbing to say what the
  `// dead: C is #if ...` comment already says.
- **Noise:** the bindgen generator now writes `allow(unnecessary_transmutes)` into its islands
  (4,590 sites; all 915 islands updated). `apply_rustc_suggestions` applied about 120 fixes in
  cold crates, and gained `--skip-crate` plus the census's renamed-lib dep-info lookup.

## 2026-10-16 — four owed items closed, and the version/null classes gated

- **crate-repair `width_alias_cast`** no longer writes a crate-private alias into a module that
  cannot see it (`harvest_alias_scope`, decline `alias_not_in_scope`). Measured over the tree,
  it agrees with all 181 compiling uses of such aliases.
- **`scan_macro_fn_shadow` and fatdata.h:** its discard harvest did not know `{NOTHING;}`. Now it
  does, and that found cdfs's 28 `ASSERT_*` shadows and fastfat's `DebugDoit`. Those are crate
  macros now, and fastfat's dead C debug stubs (`DebugTrace` … `TimerStop`) are gone.
- **`tools/scan_version_vs_c.py`** (new lint, empty baseline, control). Its first run found 7
  real sites beyond fastfat: classpnp `ClasspDeleteTimer` (zeroed a live KTIMER header) and its
  power callback, comctl32 syslink (deleted the parent's brush) and tooltips, cmd's Win7
  variable, threadpool's thread naming, and shell32's open-verb spelling. It also found fastfat's
  four disk-accounting calls and `FsRtlAreThereWaitingFileLocks`, which the 2026-10-11 pass had
  left ungated. Lesson: classpnp builds at WIN8, not WS03; the cutover erased that from its
  CMakeLists, so the scanner reads build.ninja first and the CMakeLists history second.
- **`deref_nullptr`:** 78 offsetof-through-null sites became `core::mem::offset_of!`
  (`tools/null_offset_to_offset_of.py`; `--lint` gates the class at 0).
- **`invalid_value` deny:** the fresh census reads 0, so `build` now has a `rust_ub_deny` stage
  (`rust_warning_census.py --deny invalid_value,deref_nullptr` over that arch's cargo cache).
  The census also learned to find a renamed lib's dep-info (atlclasses-rust builds `atl_rust`),
  which had made it refuse the whole total as stale.
- `c_twin.rust_functions` counted lines quadratically. Fixing that took the version scanner
  from 218 s to 71 s and sped up `scan_cfg_vs_c` too.

## The census finding that reframes everything (2026-07-16)

**201 of 219 workspace crates (92%) carry a crate-root `#![allow(...)]`** — most of them
blanket `dead_code`/`non_snake_case`/`unused` suppressions parked at `lib.rs` top. So:

- The "~9,555 warnings" figure from the rust-warnings survey was measured **with those
  blanket allows in place** — it is an *undercount*. The real surface is hidden.
- A burndown that only fixes what currently *leaks past* the allows is cosmetic. The
  durable version **peels the blanket allow off a crate**, fixes what that exposes, and
  lands the crate clean **without** re-adding the blanket — so future warnings can't hide.
- These allows exist because the c2rust/LLM translation output is noisy by construction
  (non-snake-case FFI names, dead translated-but-unused helpers). Much of it is legitimately
  suppressible — but **at the item/module level with a reason**, not a crate-root sledgehammer
  that also hides real bugs (an unused `Result`, an unreachable arm, a shadowed write).

## Three tiers — sort every warning into one

### Tier 1 — NOISE (mechanical, subagent-fannable, the bulk)
Auto-fixable style: `unused_imports`, `unused_mut`, `unused_variables` (→ `_`-prefix),
redundant clones, `unused_parens`, needless `return`. **`cargo fix` handles these.**
- This is the **subagent work** the user green-lit ("sonnet or even haiku to burn down
  simple things"). Fan-out model per AGENTS.md: one crate per agent, agent runs `cargo fix`
  + hand-cleans the residual in its crate ONLY, writes the patched `.rs` to
  `~/vibeos-rust-staging/warnings-<crate>/`, does **not** edit shared files or build. **You**
  integrate + build + gate ×3 centrally, per cluster.
- Keep legitimate suppressions, but **narrow them**: crate-root `#![allow(non_snake_case)]`
  on an FFI crate is fine to keep (the whole crate is FFI); crate-root `#![allow(dead_code)]`
  is NOT — dead code should be deleted or `#[allow(dead_code)]`'d per-item with a reason.

### Tier 2 — SIGNAL (manual audit — NOT subagent work)
Warnings that can indicate real bugs. Do these yourself, carefully:
- **`clashing_extern_declarations`** (~60 in the last census) — the same extern symbol
  declared with two different signatures across crates. This is an ABI-mismatch class; it's
  what `verify_ffi.py` also hunts. Each one is a potential silent-miscall on amd64/arm64
  (links fine, wrong ABI) that only i386 would catch. Reconcile to one declaration.
- **`unpredictable_function_pointer_comparisons`** — `ob-rust/src/obname.rs`,
  `ob-rust/src/obsecure.rs`, `ps-rust/src/psnotify.rs`. Comparing fn pointers is
  optimization-fragile (LLVM may merge/split identical bodies). Verify each compares what the
  C intended; replace with a stable discriminant if the identity matters.

### Tier 3 — TOOLCHAIN TIMEBOMB (do FIRST — it's the only time-sensitive one)
**`deref_nullptr` / FIELD_OFFSET-via-null: 55 sites and GROWING** (~32 at the last census;
ole32/oleaut32/msvcrt landing added ~23). Shape: `addr_of!((*null::<T>()).field) as usize` —
the C `FIELD_OFFSET(T,f)` idiom translated literally. **On pinned rustc 1.92 it's a warning;
on a newer toolchain it's a HARD ERROR.** The stable-toolchain lane just landed; the next
rustc bump breaks the build on all 55.
- **Fix = `core::mem::offset_of!(T, field)`** — this is `scan_magic_offsets.py`'s domain
  (the offset-drift lint). Coordinate with it: the fix is exactly what that gate wants.
- Do this tier first and land it — it's the one with an external deadline (the toolchain pin).

## ⚠️ Coordination fence — do NOT churn crates active lanes are editing

A workspace-wide `cargo fix` touches ~200 crates. Landing that under an active lane's feet
creates brutal merge conflicts. **Before touching any crate, check `git worktree list` +
recent commits.** Known-HOT crates as of 2026-07-16 — **skip these until their lane lands**:

| Active lane | Hot crates — off-limits |
|---|---|
| `shell32-com` | `shell32-rust`, `tools/rustconv/**` |
| `smp-bringup` | `ntoskrnl-rust` (ke/mm), `ntkrnlmp-cargo`, `hal-rust`, `ke-*` |
| `msvcrt-homebrew` | `msvcrt-*`, `cnt-rust`, `libm`-side crates |
| `research` | `ole32-rust`, `oleaut32-rust` |

**Start with cold crates** — the driver fleet, the smaller algorithm crates, the settled
user-DLLs. There are ~200; there is plenty to do without touching a hot one. The timebomb
(Tier 3) is the exception where you may need to touch a hot crate's file — if so, hand that
lane a tested one-line `offset_of!` diff rather than landing it yourself.

## Ordering

1. **Census first** — real per-crate warning counts on the current tree (the 9,555 is stale;
   run `cargo build` capturing warnings, or per-crate `cargo check`, across all 3 arches).
   Record the baseline so progress is measurable. Sort into the three tiers.
2. **Tier 3 (timebomb)** — the 55 `deref_nullptr` → `offset_of!`. External deadline. Land it.
3. **Tier 2 (signal)** — clashing externs + fn-ptr comparisons. Manual. Land per-cluster.
4. **Tier 1 (noise)** — subagent fan-out over COLD crates, cluster-integrated + gated by you.
5. As hot lanes land, fold their crates into the Tier-1 sweep.

## Progress log

- **2026-07-18 — clashing_extern Phase 2: the ORACLE pass — full mechanical CC canon, no agents (~570 decl
  sites + 20 shim-def bugs; the latent-link-bomb class defused wholesale).** The Phase-1 gate FAILED
  overnight (build:i386 LINK_FAIL + lint NEW): each incremental i386 build surfaced ANOTHER wrong-ABI decl
  as CGU reshuffling pulled previously-dead objects into the link (lsasrv `ConvertSidToStringSidW`/`LocalFree`
  declared cdecl → `_Name` undefined, import lib has `_Name@8`; next build: usbhub `USBD_ParseConfigurationDescriptorEx`).
  These are **consistently-declared-but-wrong externs — zero warnings, pure link bombs**; the clash lint
  never sees them. Killed the whole class at once with two new oracles (tools in scratchpad, recipe here):
  1. **nm-decoration oracle** (`ced_nm_oracle.py`): `llvm-nm --defined-only` over every `.a` in build-i386
     → symbol → `_N@x`(stdcall)/`_N`(cdecl). 30.5K platform symbols, 27 ambiguous. The linker contract,
     free, zero judgment. Split platform-vs-rust archives (rust side stale-risk, low priority).
  2. **`ced_canon3.py`** (supersedes v2): audits EVERY extern decl (not just clash-warned) in non-fenced
     crates. Truth: in-crate def > workspace def > platform-nm > rust-nm > majority(dead-safe). Def-FLIPS
     restricted to driver shim files (`ddk_shims|glue|stubs`) where the .sys link context makes platform
     truth authoritative — 20 real shim bugs fixed (cdrom/ks `KeQueryTickCount` cdecl shims shadowing the
     stdcall kernel export = the verify_ffi findings, disk/mountmgr `_InterlockedExchange`, usb* `KeInitializeSpinLock`…).
  **THE key semantic rule learned (cost 3 iterations):** flips are only meaningful where CDECL-vs-STDCALL
  is ABI-real = **x86-visible code**. Per-arch mirror files (`types_amd64.rs`/`types_arm64.rs`) and
  `#[cfg(not(x86))]` blocks deliberately use `"C"` strings for TYPE-identity with per-arch bindgen
  (`bindgen_i386` slots = `"stdcall"`, `bindgen_amd64` = `"C"`; see the po-rust power.rs:152 NOTE and
  serial.rs DriverUnload transmute idiom). Flipping them is cosmetic ABI-wise but breaks fn-ptr type
  matching → 144 E0308s. canon3 now skips non-x86-visible blocks (7355 skipped). Other traps fixed in-tool:
  edit-collision at insertion==next-block-flip line (sort tie-break), `#[inline] fn` on one line matching
  the attr-carry regex (pure-attr regex), kernel-rust `#[used]` force-arrays re-partitioned per element
  decl ABI (REFS/REFS_C split). Final: **0 NEW errors ×3 targets**; clashing 1335→1327/1445→1431/1331→1323
  (the lint barely moves because its remaining population is different — see next). Remaining map:
  **CC 359** = mostly per-arch-mirror string pairs (CORRECT code the lint hates; root fix = normalize
  bindgen fn-ptr ABI strings to `"system"` via the i386-twin projection — a systemic ntkrnl-sys/postprocess
  move, needs user green-light); **TYPE 1038 + ARITY 437 + SIGN 131** = decl-signature drift (Pass C:
  unify decls to canonical sig + compiler-driven cast repair at E0308 sites); **iaps 42** = duplicate
  type defs (dedup via `pub use`). All mechanical, no agent fan-out needed.
  **usbstor-rust EXCLUDED from this batch** (restored to HEAD): its i386 rust archive is
  link-lottery-fragile — glue shims (`IoCopyCurrentIrpStackLocationToNext`,
  `ExAllocatePoolUninitialized`, `RtlStringCchPrintfW`) are gated `#[cfg(not(x86))]` with
  NO i386 provider anywhere (the varargs-shim comment "i386 resolves from the import lib"
  is STALE LORE — nm proves no archive exports them), so usbstor.sys-i386 links only while
  the referencing CGUs stay unpulled; any glue.o symbol change (e.g. a def-flip creating
  interposition) reshuffles member selection and detonates the dead refs. Owed: complete
  the i386 shim set (drop the not(x86) gates; RtlStringCchPrintfW needs the `_v` varargs
  trampoline mirrored for i386), THEN re-run the ABI canon on it.
- **2026-07-17 — `clashing_extern_declarations` Phase 1: the safe ABI-class canon (−122 amd64/arm64, −66 i386).**
  The prior POC (below) proved the *type* re-point (`use crate::mod::SYM`) hits an 80× E0308 wall —
  call sites were written to the loose forward-decl sigs. This batch takes the **narrower, safe cut
  the POC missed: flip only the ABI, never the types.** Structure (via `ced_cluster.py`): all 2299
  clashes are **INTRA-crate** (0 cross-crate); the bug-bearing subset is **798 calling-convention
  clashes** (`extern "C"`/cdecl vs `"system"`/`"stdcall"`/stdcall — real i386 stack-corruption, harmless
  on amd64/arm64 where the conventions coincide). Truth = the in-crate **definition** (`extern "X" fn
  NAME(){`); the lint fires decl-vs-decl but the linker uses the def, so canonicalizing each
  declaration's ABI to the def's clears the lint AND fixes the ABI. Tool `ced_canon2.py` (cfg-aware +
  ABI-class-aware): flips single-fn extern blocks / extracts from multi-fn blocks, but **only** for
  (a) real cross-CLASS clashes (CDECL vs STDCALL — `system`↔`stdcall` is the same convention, never
  touched), (b) symbols with a single in-crate def class, (c) **ungated** blocks — it SKIPS every
  `#[cfg]`-gated block (422 of them; win32k alone has 257). **v1 was cfg-BLIND** and extracted fns out
  of `#[cfg(target_arch="x86")]` blocks, dropping the gate → cross-arch duplicate-decl E0428 (fastfat,
  win32k); reverted, rebuilt cfg-aware. Final: **17 crates edited (64 files; 58 non-fenced processed),
  0 new build errors ×3**; clashing
  1457→1335 / 1511→1445 / 1453→1331. **Real bug surfaced (the goal):** flipping exposed 4 E0308 sites
  in serial/afd — `SerialSendByte` (DPC), `SerialAddDevice`/`ForwardIrpAndForget`/`SerialCreate`
  (dispatch), `AfdCancelHandler` (cancel) are **defined `extern "C"` (cdecl) but registered as kernel
  callbacks the kernel invokes stdcall on i386** — a latent i386 stack-corruption bug a papering local
  `extern "system"` decl was hiding. Reverted those 2 crates from this mechanical batch (they need a
  *definition*-ABI fix + call-site check = Phase 2). See Owed-follow-ups for the Phase-2 tail
  (no-def externals ~half of CC, cfg-gated decls, the benign type/arity majority, serial/afd def-audit).
- **2026-07-17 — `win32k-rust` 118 unsuppressed `static_mut_refs` → ZERO (the boot-path side-finding, closed).**
  win32k-rust never carried the `static_mut_refs` allow, so its 118 sites leaked raw as
  warnings (nobody gates rustc warning *counts*, so they sat unfixed). Census ×3: 118 per
  arch, **146 unique** across arches (the 28-site delta is `#[cfg(target_arch)]` code compiled
  only on specific targets — source bytes identical, so the union applies to the one tree
  cleanly). win32k is autoref-heavy: only **10** plain `&STATIC` sites, **108** method/field-autoref
  (big global-state structs — `gafAsyncKeyState.as_mut_ptr()`, `gpsi.field`, `size_of_val(&gKeyboardInfo)`).
  Applied the **type-preserving raw-deref wrap uniformly to all 146** (`smr_wrap_residual.py`):
  `&X`→`&*(&raw const X)`, `&mut X`→`&mut *(&raw mut X)`, `X.rest`→`(*(&raw kw X)).rest` (kw from
  rustc's shared/mutable message). Unlike the pure-`&raw` prefix (which yields a *raw pointer* and
  breaks the `size_of_val`/ref-needed sites with E0308), the wrap yields a *reference to a deref
  expression* — silences the lint AND preserves the reference/place type at every shape, so **zero
  ref-needed breakage**. Oracle-guarded ×3: `rc=0, static_mut_refs=0, errors=0` on all arches; own-crate
  warning histogram delta is exactly `static_mut_refs 118→0` with **every other class +0** (the 6
  pre-existing `unused_parens` etc. unchanged — no new lints). 16 files, boot-path so full 3-arch gate
  before landing. No allow removed (win32k never had one) — the class is simply at zero now.
- **2026-07-16 — Tier-3 (timebombs) LANDED** (`5125e242d40` on master): deref_nullptr
  128 sites → `offset_of!`; unsupported_calling_conventions 164 sites (fastcall/stdcall,
  per-arch ABI preserved via a `fastcall_fn!` dual-emit macro); bindgen
  unnecessary_transmutes 345 sites → casts, **fixed at the generator** (`fix_transmutes`
  in all 5 postprocess_bindgen.py + standalone for usb/acpica) so regen stays clean.
  All three classes now **0 on i386/amd64/arm64**. Gate ×3 green.
- **2026-07-16 — Tier-2b (fn-ptr comparisons) + Tier-1 (cargo-fix noise sweep) batched.**
  T2b: 65 sites / 34 fns annotated `#[allow(unpredictable_function_pointer_comparisons)]`
  with a greppable `FNPTR-CMP(audited)` tag (all verified intentional NT callback/sentinel
  identity compares mirroring the C). T1: central `cargo fix` over ~80 cold crates, 3
  targets (~5k noise warnings). Also fixed a real swallowed-error bug found by triage
  (lsasrv `LsapLogonUser`, see below). Landed `eda1e0e9bc8` after a full gate-build
  recovery (see the "gate-build-is-the-oracle" lane rule for the cfg-blind botch recipe).
- **2026-07-17 — Tier-1 noise sweep batches 2-4 (the un-swept cold crates).** Same
  `cargo fix` + central-integration process, now with the full trap-handling recipe
  (3-arch **clean** build as oracle, committed_drift ×3, per-arch baseline reconcile).
  - **Batch 2** (`79256ccfb10`): win32k-rust 73 files (644 warnings) + samsrv + rtl-*.
    Over-removal reverted: rtl-diag/exception.rs (arm64 `ZwContinue`).
  - **Batch 3** (`87e41e59a4e`): fastfat/cmlib/afd + drivers, 71 files (~500). Two
    over-removals: cmlib/hiveinit.rs (i386/arm64 hive-recovery types) + smss-csrss/csrss.rs
    (`let mut Status` — a WARM-build-masked i386-cfg-mut regression the gate's clean build
    caught; hence the clean-build rule).
  - **Batch 4** (`b8185deb011`): winsrv-rust console, 30 files (~440). The deferred
    cfg-fragile crate — swept cleanly by running `cargo fix` on the **i386 target** (keeps
    the `winsrv_sys::nt::*` glob i386 needs for `SetWindowLongPtrW`); 0 over-removals.
  Real bug also fixed in batch 1: disk-rust 6 wrong STATUS_* constants (verify_constants).
- **2026-07-17 — `static_mut_refs` mechanical batch (the first "riskier" Tier-1.5 class).**
  ~272 `&STATIC`→`&raw const STATIC` / `&mut STATIC`→`&raw mut STATIC` conversions across
  56 files / 21 cold crates (samsrv, ks, win32k, smss-csrss, fastfat, classpnp, usbaudio,
  lsasrv, acpi, mountmgr, rtl-threadpool, uniata, …). **`cargo fix` cannot do this** (the
  suggestion is `MaybeIncorrect`), so the transform is driven by rustc's own byte-span
  JSON suggestions (`smr_apply_json.py`): capture `--message-format=json` from a
  `--keep-going` workspace build on all 3 targets, dedup convertible (pure `&raw` prefix)
  edits by byte-span, apply once per file descending. **The build is a total oracle here:**
  a `&raw` site genuinely *used as a reference* (`for x in &STATIC`, `&STATIC` passed to a
  `&T` param) fails to compile — never silently miscompiles. 22 such ref-needed sites
  (kdgdb, ks/pin, lsasrv/privileges iterators, services/database, win32k/keyboard+input+
  msgqueue, winsrv/terminal) were caught by the post-apply build and **reverted to
  reference form** (they stay as warnings — the honest state; converting them needs real
  refactor). All 3 arches build clean (0 real breakage). No generated files, no line-shift
  → **no baseline reconciliation needed** (every edit is a same-line in-place rewrite).
  Fenced (untouched): ntoskrnl-rust 53, oleaut32-rust 19, hal-rust, etc.
- **2026-07-17 — leftover-noise re-sweep ABANDONED (negative result, worth recording).**
  A `--keep-going` amd64 workspace census (past the freeldr/netstack plain-build bail that
  stops `cargo fix --workspace`) showed ~265 standard-fixable warnings still present in
  editable crates. Ran `cargo fix` per-crate over 33 of them → it yielded only 9 files and,
  critically, **re-introduced the exact cfg-blind over-removals already fixed in batches 1/3/4**
  (cmlib/hiveinit `#[cfg(not(bldr))]` imports, crt-startup/excpt x86_64 re-export, smss-csrss
  `let mut Status`, winsrv `use winsrv_sys::nt::*` glob). Stashed + dropped. **Lesson: the
  cargo-fix noise sweep IS complete.** What survives is the cfg-fragile tail that cargo fix
  *cannot* fix correctly (that's *why* it survived) — do not re-run `cargo fix` hoping for more;
  it will only re-botch the known-fragile files. The true remaining noise (unused_unsafe 219,
  unused_labels 156 cfg-gated, unused_macros 71 = intentional debug shims) needs per-site hand
  work or `#[allow]`, not autofix.
- **2026-07-17 — clashing_extern ARGCOUNT-6 investigated.** 5 unique syms: DrawStateW +
  FsRtlNotifyFullChangeDirectory are classifier false-positives (nested-callback commas
  miscounted; actually same arg-count, `Option<fn>` vs `*mut c_void` = ABI-identical); TracePrint
  is variadic-vs-fixed (benign on Windows x64 ABI); the two genuine ones — cdrom
  `DeviceErrorHandlerForMmc` (0 vs 4) + `DevicePowerSettingCallback` (3 vs 4) — are the
  same-crate-redecl pattern (fixable by `use`, address-of only so no live miscall). This is what
  surfaced the 520-symbol fix vector documented below. No live bug found; all latent.
- **2026-07-17 — clashing_extern CONV bucket POC: real i386 bug CONFIRMED, but NOT a burndown
  win — reverted.** Isolated the *hard* CONV signal (`clash_hardsignal.py`): of the i686 CONV
  clashes, 36 are the clean shape — a driver crate's local `extern "C" {}` forward-decl of a fn
  the same crate **defines** `pub unsafe extern "system" fn`. On i386 `"system"`=stdcall
  (callee-clean), `"C"`=cdecl (caller-clean), and grep confirmed these are used for **real calls**
  (`return ForwardIrpAndForget(DeviceObject, Irp)`, `USBH_Wait(10)`, `PciReadDeviceConfig(..)`,
  `PortFdoScsi(..)`, …), not address-of → a genuine **double-cleanup + `_Name@N`-decoration
  mismatch** latent in serial/usbhub/pcix/storport/usbstor/usbccgp/classpnp. Applied the fix
  (`clash_conv_apply.py`: relocate each clashing decl out of its `extern "C"` block — which also
  holds genuinely-cdecl variadics like `DbgPrint` that must stay `C` — into a new
  `extern "system"` block). Keep-going census ×3: i686 CONV **42→6** (the 6 = the def-is-C or
  external-USBD skips), **0 compile errors in edited crates**, amd64/arm64 a codegen no-op
  (`system`==`C` there; the ±28 total wobble is keep-going build non-determinism, not the change).
  **BUT edited-file clash count 72→72 — ZERO net burndown.** The CONV clashes don't *clear*, they
  **reclassify to pointer-type-diffs**: the local decls also carry loose `PVOID`/`*mut c_void`
  typing vs the canonical typed decl (same benign clash the redecl POC hit). So flipping the ABI
  is a strict i386 *correctness* improvement but reduces the warning count by nothing. Reverted:
  it's real driver-ABI work, but **(a)** zero burndown (the lane's goal), and **(b)**
  **unvalidatable here** — these drivers load on no smoke path, so the fix can't be demonstrated;
  landing a 14-file driver-ABI change on reasoning alone is exactly the smoke-can't-reach-it churn
  to avoid. **Disposition: the CONV-i386 bucket is a real latent-bug find for a proper, *validated*
  driver-ABI lane (align call decls to def convention AND boot an i386 config that loads the
  driver), NOT a warnings-lane drive-by.** Tools kept in scratchpad (`clash_hardsignal.py`,
  `clash_conv_apply.py`, `conv_fixable.json`). Net conclusion across both POCs: **every
  clashing_extern bucket that looked mechanical (same-crate-redecl AND CONV-i386) is not — the
  loose FFI typing that co-occurs with the flagged difference is load-bearing.**

- **2026-07-17 — allow-peel: `unused_unsafe` — 24 crates peeled, all own sites → ZERO.**
  Peeled `#![allow(unused_unsafe)]` from 24 crates (ntoskrnl-rust fenced). **Key scoping finding:
  the class is tiny in hand-written code and huge in generated bindgen** — a `-p <crate>` census
  shows most `unused_unsafe` coming from the `ntkrnl-sys` bindgen dependency (66 in `bindgen_amd64.rs`
  alone; those are generated + suppressed elsewhere, never hand-edit), while the crates' OWN sites
  total just **29** (msgina 8, config/ps/kd64/ex 4 each, ob 3, kdbg 2). **17 of the 24 allows were
  DEAD** (0 own sites — suppressing nothing, pure removal). All 29 own sites were the translation
  pattern `unsafe { <safe array/ptr literal> }` (`SubVector(unsafe {[..]})`, `= unsafe {
  addr_of_mut!(..)}`) — redundant in *any* edition (2024 requires `unsafe` around unsafe *ops*, not
  safe array/ptr construction, so **no edition-migration hostility** despite 3 sitting inside an
  `unsafe fn`). Removed the 29 `unsafe` keywords (byte-span from the ×3 census); that surfaced **9
  `unused_braces`** (single-line `= { expr };` initializers), debraced to `= expr;`. Final ×3:
  **0 unused_unsafe, 0 unused_braces, 0 errors**; full own-crate warning histogram delta is `unused_unsafe
  29→0` and **every other class +0** (no new warnings). Multi-line `= { [..] }` initializers keep
  their (non-warning) braces — `unused_braces`' multi-line heuristic doesn't flag them, goal is 0
  warnings not 0 braces. Removes 24 blanket suppressions (durability). **Side finding (NOW CLOSED — see the
  2026-07-17 win32k entry at the top of this log): `win32k-rust` leaked 118 `static_mut_refs`
  UNSUPPRESSED** — it never carried the `static_mut_refs` allow so it was never in the peel campaign;
  driven to zero via the type-preserving raw-deref wrap (boot-path, full-gated). Does not affect the
  "30/35 allow-carrying crates → zero" claim (win32k was never allow-carrying — it's a separate axis).
- **2026-07-17 — FNPTR-CMP follow-up CLOSED (no code change; see Owed-follow-ups entry).**
  Peel-and-census of all 34 audited allows showed every one of the 63 fn-ptr comparisons is an
  `Option<fn>` compare (bindgen renders Windows fn-ptr typedefs as `Option<extern fn>`), so
  `fn_addr_eq` (needs bare `FnPtr`) applies to **zero** sites. The only lint-removal available is a
  behavior-identical `.map(|f| f as usize)` rewrite that buys no correctness and churns boot-path
  crates; user chose to keep the audited per-item allows (already the sanctioned end-state). Docs-only.
- **2026-07-17 — allow-peel batch 2: 22 more cold crates driven to ZERO static_mut_refs.**
  Peeled `#![allow(static_mut_refs)]` from the remaining 22 unfenced suppressing crates
  (config/crt-startup/dbgk/fsrtl/fstub/gdi32/inbv/iomgr/kd64/kdbg/kernel32/kernel32-vista/
  msgina/pnpmgr/random/rtl-actctx/rtl-diag/rtlk/user32/vdm/vf/wmi — **shell32-rust stays
  fenced** for the shell32-com lane). Surfaced **518 hidden sites** (amd64/i386; 441 arm64)
  and eliminated all: **310 via the pure `&raw` prefix** (`smr_apply_json.py`), **258 via the
  raw-deref wrap** (`smr_wrap_residual.py`: method/field-autoref → `(*(&raw const/mut X)).rest`),
  **23 ref-needed sites** (`size_of_val`/`discriminant`/`RTL_NUMBER_OF` args wanting `&T`) fixed
  with the **shared-ref-through-raw-deref idiom** `&*(&raw const X)` / `&mut *(&raw mut X)` — a
  reference to a *deref expression*, not the static path, so it satisfies the lint while keeping
  the reference the call site needs. Two manual sites: kd64 `kddata.rs` had ONE `let d = &mut
  Static` binding used as a struct place 109× (`d.field=…`) — plain `&raw` cascaded to 109 E0609
  "no field on raw ptr"; fixed at the binding with `&mut *(&raw mut …)` (1 line, all 109 uses
  unchanged). And user32 `appswitch.rs` `windowText.len()` on a `static mut [WCHAR;1024]` (array
  `.len()` autoref) → `(*(&raw const windowText)).len()`. **Oracle-guarded ×3:** the transform
  can only fail by *not compiling* (E0308/E0609), never silently — final = 0 static_mut_refs,
  **0 dangerous_implicit_autorefs**, 0 errors on all 3 arches; `unused_unsafe` count unchanged
  (164→164) proving the wrap didn't perturb other lints. Removes 22 blanket suppressions.
  **Running total: 30/35 static_mut_refs-suppressing crates peeled to zero.** Remaining 5:
  shell32-rust (fenced) + the hot-lane fenced set (ntoskrnl/hal/ole32/oleaut32 sites live under
  *their* crate allows — do when those lanes are quiet).
- **2026-07-17 — allow-peel started: 8 kernel crates driven to ZERO static_mut_refs.**
  `#![allow(static_mut_refs)]` at crate level (35 crates carry it; only shell32 is fenced) was
  HIDING real static_mut_refs from the census. Peeled the allow from the 8 census-reachable,
  unfenced kernel-subsystem crates (cc/ex/ob/po/se/lpc/kd/ps-rust) — surfaced **185** hidden
  sites — and eliminated all of them: 152 via the proven `&raw` transform (`smr_apply_json.py`),
  33 via a raw-ptr-deref wrap for the method/field-autoref sub-pattern (`X.as_ptr()`,
  `X.get_unchecked(i)`, `size_of_val(&X)`, `X.is_some()` → `(*(&raw const/mut X)).rest`, tool
  `smr_wrap_residual.py`). The wrap is behavior-preserving (same memory place via raw deref) and,
  verified across ×3 arches, introduces **no** `dangerous_implicit_autorefs` (the feared lint-trade
  didn't happen on 1.92). Oracle-guarded throughout: 3 ref-needed sites that broke plain `&raw`
  (E0308) were fixed by the wrap form; final = 0 errors, 0 static_mut_refs, 0 new lints. Removes 8
  blanket suppressions (durability). **Key process finding: the `vibeos-cli check` gate tracks the
  custom `scan_*`/`verify_*` scanners, NOT rustc lint counts** — so rustc warnings like
  static_mut_refs have no hard gate, and peeling an allow (exposing hidden warnings) has no gate
  consequence; the constraint is purely code-quality. **Remaining allow-peel targets:** the other
  ~26 static_mut_refs-suppressing crates show 0 in the plain census (don't plain-build or fully
  suppressed) — they need per-crate `cargo build -p X` cycles (some, e.g. advapi32/kernel32/user32
  big usermode DLLs, may plain-build; config/fstub/kdbg need the reactos-link env). Same two tools
  apply. This is the model for the rest of task #9.

## Remaining work (2026-07-17, prioritized) — the plain-buildable cold-crate noise is DONE

The `cargo fix`-reachable Tier-1 noise is largely burned down. What's left:
- **`static_mut_refs` — cold-crate `&raw` batch DONE (2026-07-17, ~272 sites, see log).**
  Residual owed: **(a)** the 22 ref-needed sites reverted to reference form — these need a
  real fix, and the cleanest is often `static mut`→`static` where the datum is provably
  read-only (e.g. lsasrv `WellKnownPrivileges`/`WellKnownRights` are `static const` tables in
  the C source, only ever indexed/iterated) — but that cascades into `unused_unsafe` cleanup,
  so it's a separate semantic pass, not mechanical. **(b)** fenced hot-lane sites (ntoskrnl
  53, oleaut32 19, hal) — do when those lanes are quiet. **(c)** freeldr/netstack sites (don't
  plain-build). The transform driver is reusable: `smr_apply_json.py` + `smr_revert_broken.py`.
- **`clashing_extern_declarations` (1457) — TRIAGED 2026-07-17 (classifier: `clash_classify.py`).**
  Parses the two `expected/found` fn-signature notes and buckets each clash by ABI impact:
  - **CONV `C`↔`system` — 277 sites / 207 syms — the priority, i386-REAL.** On i386
    `"system"`=stdcall (callee cleans stack), `"C"`=cdecl (caller cleans) → calling one
    through the other's decl is a stack-cleanup mismatch = corruption. On amd64/arm64 the two
    conventions are identical, so it's *invisible there* and the ×3 gate stays green — a
    latent i386-only footgun on less-exercised paths. Fix = make each local re-decl's `extern`
    string match the true export (NTAPI → `"system"`; genuine cdecl helpers → `"C"`); build
    oracle catches wrong guesses. **POC 2026-07-17 (see progress log): flipping the `extern`
    string alone is a real i386 correctness fix but does NOT clear the warning** — the same local
    decls also carry loose `PVOID`/`*mut c_void` typing, so the clash reclassifies to a benign
    pointer-type-diff (edited-file count 72→72, zero burndown). And it's *unvalidatable* in this
    lane (drivers load on no smoke path). Treat the CONV-i386 bucket as a latent-bug find for a
    proper validated driver-ABI lane, not a burndown item.
  - **ARGCOUNT differs — 6 sites — the highest-value needles.** A different *arg count* is
    almost always a real bug (a caller passing the wrong number of args) or a macro-shim
    artifact. Investigate these 6 first — cheap, potentially real.
  - scalar size/class 295 (BOOLEAN `u8` vs BOOL `i32` etc. — mostly benign return-register
    width), ret void↔value 181 (benign, void side ignores), ABI-identical 681 (pure
    pointer-name/const-mut noise). These ~1157 are reconcile-for-tidiness, not bug-hunts.
  Distribution: fastfat 390, win32k 185, cdrom 135, cdfs 118, user32 73, classpnp 66, disk 46.
  - **THE big safe fix vector — same-crate-redecl: 520 of 828 clashing syms (2026-07-17).**
    The dominant shape is NOT cross-crate: it's a driver crate **forward-declaring, in its own
    `extern "C" {}`/`extern "system" {}` block, a function that the SAME crate defines** as
    `pub (unsafe) extern fn`. The translator emitted a cross-module forward-decl with a
    *different* (often wrong) signature — e.g. cdrom `init.rs:79` `fn DeviceErrorHandlerForMmc();`
    (0-arg) vs the real 4-arg `pub unsafe extern "C" fn DeviceErrorHandlerForMmc(..)` in
    `sense.rs:253`; the forward-decl is used only for `SYM as *const c_void` (address-of, so no
    live miscall today — but a latent footgun if ever *called* through the wrong decl).
    Per-crate hit rate: cdrom 83/85, cdfs 63/65, fastfat 62/142, win32k 36/51, usbhub 25/28,
    user32 26/44, mountmgr 15/17, … (full table via the census one-liner in the progress note).
    **Fix = delete the clashing `fn SYM(..);` line from the extern block, add
    `use crate::<module>::SYM;`** (module resolved by grepping the crate for `pub .. fn SYM(`).
    **POC RESULT 2026-07-17: NOT mechanically fixable — the loose forward-decl sigs are
    load-bearing.** Ran the full transform on cdrom (83 decls → `use crate::<mod>::SYM`, clean
    tool run, `clashing_extern` dropped 83→0). Build ×3 = **80 × E0308 mismatched-types.** The
    forward-decls are NOT used only for address-of as first assumed — they're used for **calls**,
    and the call sites were written to the forward-decl's *deliberately-loose* signature
    (`*mut c_void`, generic WDF types) while the real fn takes specific struct pointers. The clash
    is ABI-compatible (pointer=pointer, so runtime-correct *today*), but re-pointing the name at
    the real typed fn makes every call site fail Rust type-checking. So the "fix" is not
    remove-decl-+-`use`; it's **per-call-site** (cast each arg to the real type, or loosen the
    real fn — both real work). Reverted (stashed+dropped). **Lesson: clashing_extern is a genuine
    per-symbol AND per-call-site audit; there is no mechanical decl-replacement sweep.** The tool
    (`clash_fix_crate.py`) + this result are the starting point for a *supervised* audit that also
    fixes call sites. Do the true-signal buckets (CONV i386, the handful of real ARGCOUNT) by hand
    first; the ABI-benign majority is low-value churn that needs call-site work per symbol. Task #4.
  - The rest (CONV, ARGCOUNT, scalar/void/noise buckets) as above. NOT a safe unsupervised bulk
    edit — a per-symbol audit (verify_ffi's domain). Do the same-crate-redecl POC + 6 ARGCOUNT
    + a CONV pass next. Task #4.
- **`unused_assignments` (~1436)** — triaged in batch-1 as verified-safe noise; `cargo fix`
  has no autofix for it. Low priority (deferred). Task #8 done as triage; suppression owed.
- **Non-plain-buildable crate noise** (freeldr/netstack/cdfs/config/kdbg) — these need the
  reactos-link/cmake build env, so plain `cargo fix` can't reach them; a cmake-env fix path
  or manual sweep is owed.
- **`-sys` crate bindgen noise** — mostly under crate-root `#![allow]` already (low-value).
- **Blanket-`#![allow]` peel** (task #9) — the durability endgame; still owed.

## 2026-07-25 — orphaned modules gated; two Tier-2 trampoline shortcuts closed as dead ends

**Landed.** `tools/scan_orphan_modules.py` (EMPTY baseline) + `tools/rust_module_tree.py`
(one reachability walk, shared with `verify_match_consts`) + `crate-repair`'s new
`panicking_cast` rule. Details in the commit; the durable lessons:

- **An orphan `.rs` is a decoy, and the reason it is parked belongs IN THE FILE.** All 12
  orphans in the tree were legitimate parks whose reasons existed — in a CMakeLists, a lane
  doc, a commit message — just nowhere near the file, so the classpnp `class_gap3.rs` trap
  (edits pass `cargo check` because cargo never opens the file; the link keeps failing) was
  reachable for every one of them. Gate is "no UNDECLARED orphans": an
  `//! ORPHAN-BY-DESIGN: <reason>` banner satisfies it.
- **A reachability walk that fails toward "dead" hides bugs**, because `verify_match_consts`
  SUPPRESSES findings in files it calls dead. Four defects fixed, all in that direction; the
  worst was stripping block comments before line comments, so `/*` inside a `//` comment
  (freeldr's `arch/*/multiboot.S`) opened a phantom block comment that ate 53 of 87 `mod`
  declarations. 20+ crate roots carry that shape. Also: `mod r#async;`,
  `#[cfg_attr(target_arch=.., path=..)]`, and Cargo.toml `[[bin]] path =` roots.

**Tier-2 trampoline refactor — scoped, NOT started. Two shortcuts probed and both are wrong;
do not re-probe them.**

- Surface is **24,823 `#[no_mangle]` exports fleet-wide**, not the ~1,700 in the roadmap —
  that figure was ntoskrnl-only (ntoskrnl-rust measures 1,877, so it was right about
  ntoskrnl). win32k alone is 3,788. This is a program, not a task.
- **Shortcut A — "make the same-crate extern decl's convention match its definition"
  (`scan_cc_mismatch`, 305 baselined): WRONG DIRECTION.** buslogic's `BT958Hw*` are SCSI
  miniport callbacks *declared* stdcall and *defined* `extern "C"`; on i386 the miniport ABI
  is stdcall, so the DEFINITION is the wrong one. A decl-follows-def sweep would bake the bug
  in. The authority is the C prototype / `.spec`, never the local def. (cdrom is 187 of the
  305, and its 159 unique symbols are the crate-internal loose-forward-decl shape from
  task #4 — those need call-site retyping, measured at 80 × E0308 on cdrom alone.)
- **Shortcut B — "use `verify_ffi`'s `.spec` authority to fix the ABI findings": all 6 are
  correctly baselined, none is a bug.** Each crate ships a LOCAL `#[no_mangle]` shim that the
  linker resolves in preference to the kernel export, and each decl correctly matches its
  local shim. cdrom defines its own `InterlockedCompareExchange`/`InterlockedIncrement`
  (verified: real `AtomicI32` SeqCst bodies, not stubs) though ntoskrnl.spec exports them
  `fastcall -arch=i386,arm`; classpnp and disk do the same for
  `IoWMIDeviceObjectToProviderId` / `KeFlushIoBuffers`, whose spec entries do not even cover
  i386. **`verify_ffi` compares a decl against the spec without checking for an in-crate
  definition that outranks it** — worth teaching it, and worth knowing before anyone reads
  those 6 lines as a work queue.
- **So the refactor has to GENERATE decls from one authority and retype call sites in the
  same pass** — it cannot be an edit campaign over the decls that exist. That matches the
  roadmap's own framing (the trampoline as the substrate for generating crate-split's
  cross-crate decls) and it wants a quiet fleet: 12 worktrees were active on 2026-07-25.

## 2026-10-08 — census by RISK, not by count

`tools/rust_warning_census.py <build full.log>...` reads the warnings out of
`vibeos-cli build` logs and sorts every site into **UB** (undefined or miscompiles),
**LOGIC** (the code plainly does something its author did not intend), **NOISE**, or
**UNCLASSIFIED** (a lint nobody ranked yet -- never defaulted to noise). Pass one log per
arch; a site seen on several is one site. It reports coverage, because a warm build's log
only carries the crates it recompiled. `--tier UB`, `--lint <name>` list the sites.

Fresh arm64 + i386 logs, before this pass: **5,773 sites -- 192 UB, 2,461 LOGIC, 3,120
NOISE**. The raw count is dominated by three buckets (`unnecessary_transmutes` 1,782,
`clashing_extern_declarations` 1,358, `unused_assignments` 1,048); outside them, about 40
sites were worth a human read, and they were read against the C:

- **Fixed, real:** pcix's `swprintf` format was a pointer into a dead temporary
  (`dangling_pointers_from_temporaries`); shell32's `SHELL_GetShellExtensionRegCLSID` tested
  an uncast DWORD `>= 0` for `SUCCEEDED()`, so it never consulted a key's default value; five
  structs built by `MaybeUninit::uninit().assume_init()` (`invalid_value`, UB on integer
  fields -- cdrom, shell32 x3, wkssvc); scsiport dropped both `KeFlushIoBuffers` calls, and
  cdfs and disk shimmed it as empty on every arch -- the header makes it a `dsb sy` on arm64,
  so DMA reads were unordered on real hardware (fastfat was already right, via a C capture);
  wshtcpip's 11 `deref_nullptr` FIELD_OFFSETs became `offset_of!` (the toolchain timebomb);
  secur32 and gdi32 rebuilt pointers from integers (`integer_to_ptr_transmutes`).
- **Residue, deleted:** six `unused_must_use` statements computing and discarding a
  condition. Each was graded first, because a discarded `!NT_SUCCESS(status)` can be a lost
  `goto` -- and two were (netapi32, scsiport), but each sat last in its success block, so
  falling through reached the label's cleanup anyway. The rest were dropped `ASSERT`s and
  `FAILED_UNEXPECTEDLY` traces. Also fastfat's `static PAGED_CODE: ()` extern and an lsasrv
  `drop(None::<()>)`.
- **Graded faithful, left:** `RtlRaiseStatus` really does end in a self-call, as the C does
  (reached only when `ZwRaiseException` fails) -- item-level allow with the reason;
  kernel32's `Result > 0xFFFFFFFF` is the same dead check in path.c:1521; disk's
  `if !"msg".as_ptr().is_null()` is `NT_ASSERT(!"msg")`, i.e. assert-false, now spelled so.

**Owed** (tracked here until done): the `clashing_extern` Phase 2 below; `unused_assignments` 1,048 (some are lost writes -- grade, don't bulk-allow); 13
`path_statements` that are `STUB;` / `UNIMPLEMENTED;` debug macros translated as constants.
(The flexible-array reader's cargo-linked corpus, owed when this was written, closed the same
day: ten boot-path sites fixed -- roadmap.)

**Closed later the same day:**

- **`PAGED_CODE()`** is a `macro_rules!` in all 12 crates that used it (acpi, cdfs, cdrom,
  classpnp, cmlib, disk, fastfat, fs_rec, partmgr, pciidex-legacy, pcix, scsiport). Before,
  59 files declared it `extern` and called it out of line, resolved by whichever crate in the
  image exported a `#[no_mangle]` no-op (fastfat's 29 declarations had no definition in
  fastfat at all), and the rest had local empty fns. `scan_macro_fn_shadow` now owns the
  class through `DDK_MACROS`: an empty fn or an `extern` declaration of a DDK macro name is
  a finding with no `macro_rules!` in the crate needed. Against the old tree it reports 68.
- **`unreachable_code` (19).** All were read; 14 were dead tails (after `return`, after a
  noreturn `ExitProcess`/`ExitThread`/`FatBugCheck`, or after a `while(TRUE)` whose every exit
  is a `try_leave`, which is just as dead in cdfs's C) and are removed. The rest:
  - kernel32 `LIST_FOR_EACH_ENTRY!` ran the caller's `$body` and THEN advanced, so a
    `continue` spun on one entry forever: `add_resource_dir_entry` / `add_resource_data_entry`
    `continue` past every entry that sorts first, so UpdateResource hung on its second
    resource. ole32's `LIST_FOR_EACH_ENTRY!` and `_SAFE` had the same shape (datacache
    already hand-rolled one walk around it). All three advance first now, and
    `scan_dowhile_continue` flags any loop macro that advances after its spliced body.
  - cdrom `RequestHandleMountQuerySuggestedLinkName` returned `STATUS_NOT_FOUND` before its
    body, a bring-up short-circuit from `370a754c85a`, so no CD-ROM ever got its stored drive
    letter back. The C body is live, minus an invented early `BUFFER_OVERFLOW` that returned
    before writing the `NameLength` the caller sizes its retry from.
  - msafd `WSPSendDisconnect` and `WSPCancelBlockingCall` were `unimplemented!()` (a panic in
    the calling process) where the C is `UNIMPLEMENTED; return 0;`.
  - pcix `ario_ApplyBrokenVideoHack`'s `loop {}` stays: it is the pre-#9502 arbiter stub
    already tracked in the roadmap ("pcix-rust predates upstream's arbiters").

- **`static_mut_refs` 159 -> 0** (clean builds, all three arches) across 17 crates. Each
  site took rustc's migration: `&mut X` / `&X` became `&raw mut X` / `&raw const X`,
  `X.as_ptr()` became a typed cast of `&raw const X`, and a method or bitfield accessor
  became `(*&raw const X).m()` or `(*&raw mut X).m()` (mut only for `set_*` / `add` /
  `remove_all`). crt-shared's `ARRAY_SIZE!` goes through `addr_of!` like msvcrt file.rs's
  copy. That leaves the UB tier empty.

**Newly owed from that sweep:**

- The same skipped-advance shape written by hand: a `while` whose last statement is the
  step and which `continue`s without re-running it. A prototype finds 62 such loops (shell32's
  19 `cpp_com_string_cb_printf_w` copies, pci `PdoQueryResources`, se token filtering, kdbg,
  ...). Unlike the macro case each needs the C beside it, since a C `while` with a trailing
  update is faithful and only a C `for` is a bug. Spot checks (win32k `ProcessTimers`, cc
  `CcRosFlushDirtyPages`) were both fine, because they advance before every `continue`.
  **Closed 2026-10-09** (below): one bug, `scan_dowhile_continue` owns the shape.
- `UNREFERENCED_PARAMETER` as an empty fn: 42 sites. Same class as PAGED_CODE, but
  `scan_macro_fn_shadow` only sees the copies in crates that also define the macro.
  **Closed 2026-10-09** (below).
- msafd `WSPAcceptEx` / `WSPGetAcceptExSockaddrs` are still the old stubs.rs bodies
  (`return 0` / `unimplemented!()`), but upstream has since given both real bodies in
  `dll/win32/msafd/misc/dllmain.c` and neither is ported. This is an upstream-sync gap,
  not a warning: port them from dllmain.c.

**`unused_assignments` is ranked by the C, not by the warning (2026-10-08).**
`tools/rank_unused_assignments.py <clean full.log ×3>` pairs every site with its C write
(same callee, literal, or same-key ordinal; refuses when the counts differ, e.g. a hoisted
Rust call) and runs a structured liveness over the C function: branches, loops to a fixed
point, `switch` fall-through, `goto`, `#if` arms. `UNREFERENCED_PARAMETER(x)` is not a read,
`list_for_each(x, ..)` writes its cursor, `sizeof` reads nothing, `#ifdef __REACTOS__` takes
its own arm, `while (TRUE)` never exits through its condition, and reads that exist only
through `&x` or only inside DPRINT/ASSERT (or an `if` guarding nothing else) get their own
verdicts. Names match EXACTLY, falling back to a case/underscore-insensitive spelling only when
it is unique: the insensitive match had made the type `FEATURE_HEADER` a read of
`featureHeader`. Of the 1,025 sites (clean ×3 logs of 2026-10-08):

| verdict | n | meaning |
|---|---:|---|
| `C_READS` | 4 | the C reads it and the Rust does not |
| `C_READS_ADDR` | 2 | the C reads it only by passing `&x` |
| `C_READS_DEBUG` | 12 | only an ASSERT/DPRINT reads it |
| `NO_MATCH` / `NO_C_FN` | 57 / 92 | no confident pairing: read by hand |
| `C_DEAD` / `C_NO_INIT` | 538 / 320 | the C has the same dead store, or no matching initializer: harmless |

**The first 23 graded `C_READS*` are read (2026-10-08)**, before the rules above sorted most
of them into the debug and harmless verdicts. Two real bugs, both fixed:

- ole32 `StorageImpl_LockRegionSync` had turned the C's
  `do { .. } while (hr == STG_E_ACCESSDENIED || hr == STG_E_LOCKVIOLATION)` into
  `if (contended) { .. } else break`, so a sanity probe that failed some other way kept
  the wait going for the full 20 seconds instead of returning that error.
- partmgr `VolumeDeleteMountPoints` freed `OutputBuffer` on `STATUS_BUFFER_OVERFLOW` and
  looped; a second overflow ends the loop and the cleanup freed it again. The C has the same
  double free; the Rust now nulls the pointer after the free.

The rest: cdrom `RequestSynchronizeProcessWithSerialQueue` returns `STATUS_ACCESS_DENIED`
where the C falls through and returns the blocked-query status, which every caller discards.
cdrom `DeviceSendSrbSynchronously` sets `ioctlCode` early on purpose, fixing a C bug where the
first send went out with 0, and says so in a comment. ole32 `create_storagefile` spelled out a
`goto end`. rtl `RtlpInitEnvironment`'s reset of `base_address` matches the C, whose 1 MB
reserve at `(PVOID)4` rounds the base to 0. The 12 debug ones are ASSERTs and prints that the
Rust dropped.

**The debug-only reads and the 149 unpaired sites (2026-10-08).** The C's ASSERTs are back at
the eight ASSERT-only sites and heappage's two `ASSERT(FALSE)`s, as RtlAssert-and-continue.
The prints stay dropped. The 149 were read against the C by hand: 121 faithful, 7 debug,
16 benign, 2 Rust-only, and four real bugs, now fixed:

- uniata `UniataFindBusMasterController` ran the `#ifdef UNIATA_INIT_ON_PROBE` arm, a macro
  nothing defines. That ran an extra `FindDevices` on every channel at FindAdapter time and
  never cleared a latched `BM_STATUS_INTR`. It now runs the `#else` arm the C compiles. Smoke
  is green on amd64 and i386 with the disk and CD found.
- atapi-legacy `FindDevices` indexed `DeviceFlags[deviceNumber + Channel*2]` after the probe
  loop had left `deviceNumber == 2`: index 4 of 4 on channel 1, a panic (the C reads past the
  array). It indexes by `i` now.
- explorer `ProcessRunKeys` reported success on out-of-memory, because the C's `goto end`
  skips `res = ERROR_SUCCESS` and the Rust's `break` did not.
- lsasrv `LsarQueryAuditEvents` with no audit data returned success, a NULL output, and a
  leaked buffer.

Two stores were a write through the variable after a raw pointer to it had been taken
(rtl-sync `RtlWaitOnAddress`, explorer `tw_track_menu`). They now go through the pointer.
**Do not delete** cdfs's SEH cleanup flags (`UnlockVcb`, `UnlockFcb`,
`CleanupCompoundPathEntry`), the 10 benign ones in strucsup/allocsup/create: the finally
block reads them through `*p_X` pointers that rustc cannot see.

Why the ranker could not pair these, which is owed as rules:
- The C is gone from the tree for uniata (`50c1dd7f8d1^`), NT4 atapi (`a3a7746beed^`) and
  pciidex-legacy (`5213cf717cf`). Read them with `git show`.
- C++ methods became prefixed snake fns (`tw_track_menu` = `CTrayWindow::TrackMenu`).
- `_impl` / `_v` / `_rs` wrapper suffixes.
- Writes hidden in macros: `WaitOnBusy`, `GetBaseStatus`, `NEED_SPACE`, `CON_STREAM_WRITE2`.
- `goto` emulated with a flag or a labelled block.

Upstream drift found on the way, not a warning: pcix arbiters (already tracked), afd
AcceptEx's `SatisfySuperAccept` branch in `ListenComplete`, explorer's syspager balloon
crash fix (#9332), and an oleaut32 Wine resync (`VarMod`, `serializeBMP`).

**The 858 harmless sites (2026-10-09).** `tools/apply_dead_assignments.py <rank.json>` takes
the ranker's `C_DEAD`/`C_NO_INIT` rows and removes them: a pure initializer is dropped
(`let mut x: T;`), a pure dead store is deleted, a dead store of a call keeps the call. It
looks inside `stdmethod! { fn .. }`, which tree-sitter sees as one opaque token tree, by
re-parsing the body in place. Then, for each of the 200 sites where the dead initializer
was the only reason for `mut`, rustc flagged `unused_mut` on all three arches, and `mut`
was dropped. It applied 817 and left 41 for review:

- **13 initializers stay, because the removal compiles and is still wrong.**
  `addr_of_mut!(x)` / `&raw mut x` of an uninitialized local is NOT E0381, and the callee
  it is handed to may read the initial value (samsrv's in/out `&raw mut Length`). So
  "rustc's definite-init proves it" holds only for a variable never raw-borrowed in its fn.
  The tool now skips those. The first run did not, and the audit caught it before any boot.
- **Three stores are live, and rustc's warning was a false positive.** cdrom
  `fourByte = &allocationLength as *mut FOUR_BYTE` (DVD READ KEY),
  `addr_of_mut!(blockAddress)` (SET READ AHEAD), and `srb.SenseInfoBuffer = &mut sense`
  (autorun) each take the pointer first, write the local directly, then read through the
  pointer. That direct write is aliasing UB, the same class as rtl-sync and traywnd. Each
  now takes the pointer after the write, so the UB is gone, and so is the warning, because
  rustc sees the borrow as the read.
- 7 cdfs SEH flags, as above. 18 out-param and "NULL after free" stores (`subKey`, `Spb`,
  `SidString`, version `len`, scsiport `Again` x2 and `Offset += CopyField(..)` x6, heappage
  `Flags |=`), read
  and removed by hand.

The skip rule for a store is position-aware: a raw-pointer read rustc cannot see needs a
pointer that EXISTS at the write, i.e. one taken earlier in the fn or anywhere in a loop
around it. A borrow after the write is itself a read rustc counts.

The ranker relocated rows by snippet when the tree had moved since the build, and after a
13-line shift one `hRes = IStream_Write(` site took its neighbour's line (two rows on one
line). It now maps each log's lines through `git diff` against the tree that build compiled,
which the `warm-receipt` stage records, and calls a non-unique snippet STALE.

**The last 175, and the crates the census had not covered (2026-10-09): `unused_assignments` → 0.**
Two Haiku 5.5 subagents read the 175 against the C. They were read-only on the repo, and
each wrote one JSONL row per site: a verdict plus an exact `old`/`new` edit. I reviewed and
applied the edits centrally. Results: 147 faithful dead writes, 11 debug-only, 17 cdfs SEH
flags written through their `p_X` pointer, and no bugs. The earlier hand read's bugs were
already fixed. I also moved 14 more cdfs flag writes rustc never warned about onto their
pointers. Haiku proposed one meaningless `let _ = (*p).field;` (dropped). It also removed
four initializers that my raw-borrow rule would have kept. Each turned out to be written
before its borrow, so the rule is stricter than it needs to be: the condition that matters
is a raw borrow BEFORE the first write (owed: refine `apply_dead_assignments`).

**A clean log does not carry every crate's warnings.** The 000251 clean ×3 logs held no
msvcrt-crt-rust or crt-shared-rust warning at all, the next clean build (150818) held 27
`unused_assignments` there, and the one after it (154317) held none again for unchanged
content. A crate's warnings reach `full.log` only from the build that really compiles its
content, probably because sccache serves the rest without them. (`NOT BUILT (no coverage)`
in the same log is `scan_flex_array_panic`'s binary corpus, NOT cargo's, and says nothing
about this.) **Owed: the census must check per-crate coverage** (closed 2026-10-09, below), by compiling each crate
it reports on or by refusing a log that lacks a crate's compile, before any total is
trusted. The zero below was established crate by crate from the build that compiled each
one's current content. The 27 were:
- i386 `cxx_frame_handler` had dropped `__EXCEPT_CTX(se_translation_filter, &ctx)` and
  left `ctx` built and never read. It is restored as the `if _SEH2_Volatile0` guard that
  x64 `except.rs` already uses. That is inert under DUMMY_PSEH, as in the C.
- `_statusfp` / `_control87`: the `flags = 0` is dead on one arch set and in/out to
  `_setfp` / `__control87_2` on the other, so each cfg arm now owns its initializer.
- `create_locinfo`: current Wine overwrites `i = GetLocaleInfoEx(..)` with
  `i = WideCharToMultiByte(..)` at once. The tree's `locale.c` is the older A-version and
  is not the oracle here.

**Four owed items closed (2026-10-09).**

- **The census reads cargo's cache, not the log.** `tools/rust_warning_census.py build-i386
  build-amd64 build-arm64` reads every unit's `.fingerprint/<crate>-<hash>/output-lib-*`.
  That file is rustc's JSON, the one cargo replays for a fresh unit, and it names the lint,
  so nothing is inferred. A unit counts only if no file in its dep-info is newer than its
  `invoked.timestamp`, which is cargo's own freshness rule. A crate with no fresh unit is
  refused (exit 3, `--allow-stale` to report the rest). msvcrt-crt-rust's cache held 357
  warnings for the same content one clean log showed none of. Logs still work, with the
  caveat printed. The first verified census (fresh ×3, 653 arch:crate units): **8,447
  sites, 0 UB, 1,720 LOGIC, 6,727 NOISE**. `unnecessary_transmutes` is 5,322, not the
  logs' 1,782. Seven LOGIC sites no log had ever carried were graded noise and fixed:
  version-rust's NUL for a dropped WARN; an atapi `NT_VERIFY` and shell32's
  `FAILED_UNEXPECTEDLY(hr2)`, both "report but don't fail" in the C; syssetup's
  `DateTime_SetSystemtime` comparing where the C casts to BOOL; diskpart's three
  `while true`. LOGIC is now `clashing_extern` (Phase 2), 12 fn-pointer comparisons
  outside the audited allows (owed), and the graded kernel32 and pcix sites.
- **Hand-written `while` that `continue`s past its step: 61 graded, 1 bug.** Two Haiku
  agents read all 61 against the C, and I re-read the four they judged without it, plus
  partmgr and se. The bug was cmlib `HvValidateHive`, whose C `for` runs `BlockIndex++;
  continue;` on a NULL bin. The Rust spun forever there, under a comment calling it
  "faithful to C". partmgr's restart-after-match is right and the C's `for` is not: the
  step skips the head it just reset to, which can hand out a PDO number already in use.
  The Rust now says so. Everything else is a C `while` with the same `continue` (17) or
  advances before every `continue` (43). `scan_dowhile_continue` now flags the shape. An
  update covers a `continue` only from a block still open at it, so a step in a sibling
  `if` arm does not count. That rule cut the 61 to 26, all baselined as graded. Reading
  each file once also made the scan about twice as fast.
- **`UNREFERENCED_PARAMETER` is a `macro_rules!` in all 13 crates** that had it as an
  empty fn or an extern declaration: 40 definitions and declarations gone, 102 calls now
  `!`. Three of them were `#[no_mangle]` exports resolving the other crates' externs.
  `scan_macro_fn_shadow` lists it in `DDK_MACROS`, so an extern declaration fires with no
  in-crate macro. Before, the widened rule reported 35 files; now it reports 0.
- **The `path_statements` were not all noise.** kernel32.h's `STUB` is
  `SetLastError(ERROR_CALL_NOT_IMPLEMENTED); DPRINT1(..)`, but kernel32-rust had it as
  `const STUB: () = ()`. So 11 live `STUB;` stubs, plus 13 more commented out as
  "debug no-op", returned FALSE without saying why, and a caller's `GetLastError()` read
  whatever came before. All 24, and `GetActiveProcessorGroupCount` (which is `STUB;
  return 1;` in the C), now go through a `STUB!()` that sets it. `UNIMPLEMENTED` (debug.h,
  a print only) and `NOTHING` really are empty. win32k's `STUB` is a `DbgPrint` alone.
- **`clashing_extern` return mismatches: 78 graded, 0 bugs.** Every declaration that
  disagrees on whether the function returns (40 functions) has its value discarded at
  every Rust call and at the matching C call: statement calls, `let _ =`, `!` vs `()` on a
  real noreturn. Still owed: 16 fastfat declarations give FatBugCheck 32-bit
  arguments where the C passes `ULONG_PTR`, so on amd64 the upper halves of the bugcheck
  parameters are garbage. Retyping it hits the call-site E0308 wall described above.

**`clashing_extern` Phase 2 by width, the AcceptEx port, and the noise applier (2026-10-09).**

- **Classified from the cache, not by eye:** 1,703 clashing sites. 1,531 differ only in
  pointer target or signedness (same width), 14 differ in calling convention only on 64-bit
  (where "C" and "system" agree), and 55 are the already-graded return set. Two Haiku
  agents graded the other 102 (argument or return WIDTH, by-value struct, i386 ABI)
  against the C. 73 were benign, 12 cosmetic, and these were real:
  - **fastfat `CanFsdWait` returned garbage.** The macro (`IoIsOperationSynchronous(Irp)`,
    fatprocs.h) was a `VOID` no-op in `rust_cohab_stub.c`, fastfat.sys's only definition.
    So all 14 FSD entry points set their IrpContext's Wait flag from a leftover register.
    The stub now returns the real answer, and the 11 Rust declarations read a BOOLEAN.
  - **cdfs `CdIs8dot3Name`** is defined taking `UNICODE_STRING` by value, but dirsup.rs
    declared a pointer and passed `addr_of_mut!`. It worked on amd64 by luck (a 16-byte
    struct goes by hidden pointer) and read garbage on i386 (8 bytes on the stack) and
    arm64 (two registers).
  - **Width bugs:** fastfat `FatBugCheck` (16 declarations, 28 calls) and cdfs
    `CdBugCheck` (the definition itself) took `u32` where the C passes `ULONG_PTR`, which
    truncates bugcheck parameters on 64-bit. oleaut32's `_wcsnicmp` count was `u32`, not
    `size_t`. user32 clipboard's `GlobalSize` returned `DWORD`, not `SIZE_T`.
- **CLOSED 2026-10-11 (see "fastfat flag macros" below).** Was owed: fastfat's `SetFlag` / `ClearFlag` are also
  out-of-line stubs, `*(PULONG_PTR)x |= f`. On 64-bit that is an 8-byte read-modify-write
  of a 4-byte (sometimes 1-byte) field and its neighbour: not atomic, and past the end at
  the end of an allocation. `FlagOn` / `BooleanFlagOn` declarations narrow their arguments.
  The fix is the PAGED_CODE treatment for the whole family (about 480 calls and 70
  declarations): crate `macro_rules!` on the real lvalue, then delete the stubs. The
  generated cdfs `types_i386.rs` also declares `CdCreateFcb`'s `FILE_ID` as a pointer,
  where it is a `LARGE_INTEGER` by value; no live caller uses it, so the fix belongs in
  the generator.
- **AcceptEx (upstream a68bfa2fb41) is ported.** A Haiku agent drafted the afd half: 18
  edits covering every C hunk (`SatisfySuperAccept`, `AfdSuperAccept`,
  `AcceptExReceiveComplete`, the PREACCEPT dispatch and cancel paths, bind/connect state
  checks). I reviewed it against the C. It compiled first time once `TDI_ADDRESS_INFO`
  had a home (afd glue.rs). The afd islands were regenerated with gen_driver_types.sh
  rather than hand-edited. That picked up `AcceptIrp` in `_AFD_FCB`, so the Rust FCB
  layout matches the header again. msafd's `WSPAcceptEx` / `WSPGetAcceptExSockaddrs` are
  ported by hand, `SIO_GET_EXTENSION_FUNCTION_POINTER` returns success for
  GetAcceptExSockaddrs, and `SO_UPDATE_ACCEPT_CONTEXT` is accepted.
- **`tools/apply_rustc_suggestions.py`** applies rustc's machine-applicable fixes from the
  cache. It only applies a fix every arch that compiled the file agrees on, never inside a
  macro expansion, and never in a generated file. It also refuses rustc's float
  `from_bits` rewrite, which does not compile on the i386 no-SSE cell (that cost one
  build). Applied: `unused_mut` 100 to 6 (the rest disagree by arch) and `unused_parens`
  160 to 0. Not applied: `unused_imports` (an import used only under `#[cfg(test)]` reads
  as unused here). Owed: regenerating the bindgen islands, which carry about 4,590 of the 4,602
  `unnecessary_transmutes`; the afd regen (bindgen 0.73) emits none.
- **The census counted a file twice** when a crate reaches another's source by path
  (menuband: `../../shell32-rust/src/island_*.rs`). Paths are normalized now, and the
  applier would otherwise have applied the same byte-offset edit twice. The verified total
  is **7,530 sites: 0 UB, 1,699 LOGIC, 5,831 NOISE**. The 12 function-pointer
  comparisons got audited allows, except ks's `SupportHandler != None`, which is now
  `.is_some()`.
- `apply_dead_assignments` keeps an initializer only if a raw borrow can see it: a plain
  `x = ..;` in the `let`'s own block before every raw borrow now makes it removable.

## fastfat flag macros and the bugs behind them (2026-10-11)

- **`SetFlag` / `ClearFlag` are crate macros on the real lvalue** (185 live calls), so the
  `rust_cohab_stub.c` stubs are gone. Each call used to be an 8-byte read-modify-write
  through a `PULONG_PTR`, and the pointer casts at the call sites hid the true width: the
  IO_STACK_LOCATION's UCHAR `Flags` (`.cast()`), the VPB's USHORT `Flags`
  (`as *mut u16 as *mut u32`), `VcbState as *mut u8`. A compound assignment can't infer
  `as _`, so the macros go through a `Flag` trait (`flags.rs`) that truncates like C.
  `FlagOn` / `BooleanFlagOn` are now module-local inline fns with each module's declared
  signature, computed at u64. The C stub took ULONG_PTR, so it read undefined upper bits
  of a 32-bit argument.
- **A Haiku audit of about 300 live `FlagOn` calls found a truncation class.**
  `FlagOn((X as u8) as u32, F)` narrowed the field before the test, at 21 sites. It was
  live where F >= 0x100: cachesup's `IRP_CONTEXT_FLAG_PARENT_BY_CHILD` (0x80000000)
  never fired, deviosup's `VCB_STATE_FLAG_REMOVAL_PREVENTED`, create's `DELETE` /
  `MAXIMUM_ALLOWED` access mask, strucsup's `DISMOUNT_IN_PROGRESS` asserts. Also
  `FlagOn(..) as u8|BOOLEAN` returned 0 for USER_IO (0x400), at 3 sites; those are now
  `!= 0`.
- **A second Haiku audit checked about 1,140 fastfat consts against the C.** Real:
  - flush.rs `VolumeClean = 2`, which is `VolumeDirtyWithSurfaceTest`, so the flush
    path marked the volume for a surface test instead of clean.
  - `IOCTL_VOLSNAP_FLUSH_AND_HOLD_WRITES = 1` (real 0x53C000).
  - `MdlMappingNoExecute` / `POOL_NX_ALLOCATION` carried the Windows values, where
    fatprocs.h zeroes both under `__REACTOS__`. classpnp's `POOL_NX_ALLOCATION` was the
    same bug.
  - Dead but wrong: `DOS_QM`, a second `FAT_CLUSTER_BAD`, fatdata's `Dbg`.
- **`verify_constants` couldn't see either.** `MdlMappingNoExecute` and
  `POOL_NX_ALLOCATION` are now applied after the headers, like the `NonPagedPoolNx`
  seed. `ntifs.template.h` joined the CTL_CODE headers (FSCTL / volsnap), so the 1 now
  fails.
- **Owed items from this entry, all closed 2026-10-11:** see the next section.

## fastfat `clashing_extern` 284 -> 0, the trace macros, and the NTDDI finding (2026-10-11)

- **fastfat's own functions** (64 symbols, 311 redeclarations) are now `use crate::<mod>::<fn>`,
  so every call type-checks against the real definition. The 242 resulting type errors were
  graded: crate-repair took the mechanical ones, and two Haiku agents graded the 40 left
  against the C. The real ones were mistyped pointer-sized locals (fileinfo's BCBs were
  `*mut PBCB` where the C has `PBCB`), plus flush.rs's opaque `struct IRP_CONTEXT { _unused }`
  leaking into its pub API. `FatData` is the real static everywhere now.
- **External symbols** (56) have ONE declaration in `externs.rs`: kernel exports re-exported
  from ntkrnl-sys (bindgen of the real headers), and the cohab shims transcribed from
  rust_cohab_stub.c / rust_cohab_fat.c. A Haiku grader called 14 of the old declarations UB.
  Each was checked against the code, and none was live:
  - FastMutex was cfg-split.
  - IoSetCompletionRoutine has a cdecl trampoline.
  - FatGetIndexFromLbo's caller truncates to USHORT in the C.
  - FsRtlCheckOplockEx's calls pass NULLs or the C argument order.
  - DbgDoit's callee ignores its argument.
- **fatdata.h's `DebugTrace` / `DebugUnwind` / `DebugDump` / `DbgDoit` are `{NOTHING;}` in a
  free build.** They were about 880 FFI calls into no-op C stubs, the "debug macro as an
  out-of-line fn" class. They are crate macros now, and so is `NodeType` (nodetype.h). That
  also closes the 0/1 `Dbg` consts item: nothing reads them anymore.
- **crate-repair's `raw_pointer_cast` now refuses an indirection-depth change**
  (`*mut *mut *mut c_void` for `*mut *mut c_void`) unless one side is a bare void pointer. It
  had been `.cast()`ing them silently. That is the shape the fileinfo BCB locals had.
- **cdfs `CdCreateFcb`:** 94898d07b5a hand-edited the generated islands from bindgen's
  by-value `FILE_ID` to `*mut c_void`. That edit is reverted, and create.rs uses the
  definition. The same commit's five BOOLEAN-to-u32 edits are consistent: no C caller
  remains.
- **NTDDI: the "invented" constants were WIN8-only code.** fastfat builds at
  `_WIN32_WINNT=0x502` (NTDDI WS03), so every `NTDDI_VERSION >= VISTA/WIN7/WIN8` block is
  compiled out of the C. Two Haiku audits of all ~100 gated sites found about 40 where the
  Rust runs the compiled-out side. Examples: `FSCTL_REQUEST_OPLOCK` dispatched,
  `FILE_OPEN_REQUIRING_OPLOCK` honored, oplock-H breaks on sharing violations, the
  purge-failure and zero-on-dealloc FSCTLs, FAT prefetch. Method: evaluate each C `#if` at
  NTDDI 0x05020000, then diff against the same-stem Rust fn. **Fixed in the next commit.**
  27 gates use the crate's never-set cfgs (`ntddi_ge_ntddi_{vista,win7,win8,winthreshold}`, now
  all declared), or `cfg!()` inside a condition. Where the C has an `#else`, it is translated:
  the control-code shared test, and `FsRtlAreThereCurrentFileLocks`, a header macro, inline.
  The same pass found `#[cfg(not(reactos))]` for the C's `#ifndef __REACTOS__`. `reactos` is
  never set, so that code compiled IN; it is now an explicit never-set `not_reactos`. With
  `DBG`, `alpha` and the never-enabled wow64/FASTFATDBG features declared, **fastfat-rust has
  0 warnings on all three arches.**
- **Owed:** a scanner for the NTDDI class (like `scan_cfg_vs_c` for arch). Every MS-architecture
  driver with `#if NTDDI_VERSION` (cdfs, classpnp, disk, cdrom) can carry the same drift.
- **Still owed:** crate-repair's `width_alias_cast` named a callee-module-local alias
  (`FAT_FLUSH_TYPE`) that the call site cannot see (3 sites, fixed by hand).

## Owed follow-ups

- **`clashing_extern_declarations` Phase 2 (task #4) — the audit-heavy tail after the safe ABI-class
  canon landed.** Phase 1 fixed the mechanical, def-known, ungated cross-class CC clashes (−122/−66/−122).
  What remains (clashing still ~1335 amd64):
  - **No-def external CC clashes (~half of the 798 CC).** atapi/buslogic/storahci/iomgr/... have their
    clashing symbols declared (not defined) in-crate — two decls of an *external* framework callback
    (scsiport/storport/NT) that disagree. Truth = the external binding's ABI (the `-sys` crate /
    ntkrnl-sys / NT convention = `system`), not an in-crate def. Needs an external-ABI lookup per symbol
    (bounded per-crate subagents fit here: 3-5 at a time, sonnet, NO recursion, one crate each).
  - **cfg-gated decls (422 blocks skipped, win32k 257).** Per-arch `#[cfg]` extern blocks (x86 `stdcall`
    vs non-x86 `system` — often already correct, just clash cosmetically across the arch union). Need a
    cfg-aware, per-arch canon (flip carrying the gate); the v1 cfg-blind extraction is the anti-pattern.
  - **The benign type/arity majority (SIGN i32/u32, pointer-target, `*mut`/`*const`, missing-return).**
    Mostly cosmetic, BUT the *missing-return* ones are real dropped-NTSTATUS bugs (e.g. cdrom
    `init.rs:69` `fn DeviceSetParameter(...);` drops the `-> NTSTATUS` the def returns). Reconciling
    types is the 80×-E0308 wall from the POC below — per-call-site work, supervised.
  - **CLOSED (checked 2026-10-09): every definition and declaration of these is `extern "system"` now.**
    **serial/afd callback def-ABI bug (found this batch, reverted).** `SerialSendByte`/`SerialAddDevice`/
    `ForwardIrpAndForget`/`SerialCreate`/`AfdCancelHandler` defined cdecl but registered as stdcall
    callbacks → latent i386 stack bug. Fix = change the *definitions* to `extern "system"` + verify no
    direct-call site depends on cdecl. Real bug, not noise.
  - Tooling in scratchpad: `ced_cluster.py` (categorize CC vs type/arity, intra/inter), `ced_canon2.py`
    (cfg+class-aware ABI canon, reads HEAD → idempotent, `--skip SYM` for oracle-flagged fn-ptr sites).
- **FNPTR-CMP re-architecture — CLOSED 2026-07-17 (audited allows are the correct end-state;
  `fn_addr_eq` is inapplicable).** Investigated all 34 `#[allow(unpredictable_function_pointer_comparisons)]`
  functions → 63 actual comparison sites (peel the allows, `--keep-going` census ×3, primary span
  = exactly the `A == B` expr). **Finding that closes the follow-up: NOT ONE site is a bare
  `fn == fn`.** Every fn-ptr typedef in these crates is `Option<fn>` — `WNDPROC`, `PHANDLER_ROUTINE`,
  `PFNCALLBACK`, `FARPROC`, `PDRIVER_FS_NOTIFICATION`, `PSE_LOGON_SESSION_TERMINATED_ROUTINE`, … are
  all `::core::option::Option<unsafe extern fn(..)>` (how bindgen renders a nullable Windows fn
  pointer). `core::ptr::fn_addr_eq` requires a bare `FnPtr` operand and rejects `Option<fn>`, so the
  "convert the clean `fn == fn` ones" plan has **zero applicable sites**. The three shapes present —
  `Option<fn> == Option<fn>` (~48), `Option<fn> == Some(fn)`/sentinel `old == sig_ign()` (~12),
  `X == transmute(fn)` (ps psnotify, 3) — could only be de-linted via an inline Option-aware
  `a.map(|f| f as usize) == b.map(|f| f as usize)` rewrite, which is **behavior-identical** (same
  address compare, same "addresses vary across CGUs" caveat the lint warns of) and buys **no
  correctness** — it just trades a documented per-item allow for `.map()` boilerplate on every
  comparison and churns four boot-path crates (ob/ps/se/win32k). **Disposition (user decision
  2026-07-17): keep the audited per-item allows.** They are already the lane's *sanctioned* end-state
  for a genuine intentional-identity compare (`#[allow]` per-item WITH a documented reason — the
  `// FNPTR-CMP(audited)` tag records intent + the C-source mirror), not a stopgap. A generic
  `fn_opt_addr_eq` helper is also impossible on stable — `core::marker::FnPtr` (the bound
  `fn_addr_eq` needs) is perma-unstable and unnameable. Re-open only if a future need makes the
  `.map(|f| f as usize)` conversion worth its cost; the census tooling is in scratchpad
  (`fnptr_convert.py`).
- **Upstream ReactOS-C bugs faithfully preserved by translation** (found during
  unused_assignments triage, NOT translation-introduced — tracked here, not fixed in
  this lane): `pcix-rust/enumr.rs:1575`, `pcix-rust/init.rs:367`, `pcix-rust/utils.rs:714`
  (unchecked status / computed-but-unused / hardcoded wrong return status — each matches
  the C original, incl. a commented-out `//if (!NT_SUCCESS(Status)) break;` in init.c:752).

## Lane rules

- **Batch big, gate once** (2026-07-16 user directive, overrides the old per-cluster rule):
  roll mechanical work up into big commits and run ONE `vibeos-cli gate` at the end — at
  thousands of commits/month nobody reads commit-level history, and per-cluster gating
  serializes the fleet on the host CPU-lease pool. Validate cheaply as you go
  (`cargo check` per crate × 3 targets). Auto-push on ×3-green still applies.
- **cargo-fix `--broken-code` hazard (central-integration duty):** a workspace `cargo fix`
  mis-handles macro/cfg-blind cases — it removes imports/renames params used only inside
  macro invocations (e.g. `RTL_CONSTANT_STRING!(...)`), over-qualifies enum paths with
  unimported modules (`types::types_arch::X`), strips `mut` off a binding reassigned only
  under *another arch's* cfg arm (`let mut OldOffset` in freeldr i386 code), removes a
  `pub use win64::{…}` re-export the crate needs (crt-startup amd64), and drops a
  `use winsrv_sys::nt::*;` glob that on i386 is the *only* source of `SetWindowLongPtrW`
  (bindgen_i386 has just the 32-bit `SetWindowLongW`).
- **The oracle is the 3-arch `-k0` gate BUILD, not `cargo check` (2026-07-16, learned the
  hard way — a batch landed a green-looking commit that failed the gate build on i386+amd64).**
  `cargo check -p X` on the *default* target sees only ONE arch's cfg, so it green-lights
  edits that break a different arch's gated arm. Worse: crates that "don't build standalone"
  under `cargo check` (freeldr/uefildr/winsrv/kdgdb/netstack) are **not** exempt — they build
  fine in the gate via `reactos-link` (a build.rs-driven `cargo build -p X --target …`), so
  EXCLUDING them from validation hides exactly the cfg-blind botches. Do NOT trust "excluded
  standalone failure" as a reason to skip a crate. The good news: the cargo-fix noise class
  (drop `mut` / remove import / remove parens) can only fail by *not compiling* — so a
  `bash ../tools/canon-run.sh ninja -k0 livecd` on each x86 arch (+ the gate's arm64) is a
  **complete** oracle. Recipe when a batch breaks the gate: run `ninja -k0 livecd` per broken
  arch to enumerate the FULL casualty set in one pass (don't whack-a-mole one error at a
  time), then for each broken file either surgically restore the one botched line (`let mut`,
  the re-export) or, if the botch is dense/cfg-fragile (winsrv's whole console frontend),
  revert the *entire crate* to the pre-batch parent — `git checkout <parent> -- <crate>/`.
  **`<parent>` is `HEAD~1` once the batch is committed, NOT `HEAD`** (a `git checkout HEAD`
  restores the committed botch and no-ops).
- **Two per-arch gate checks a single-arch `cargo check` misses entirely (2026-07-16):**
  (1) **the lint baseline-diff runs PER ARCH** — `check amd64` reads `bindgen_amd64.rs`,
  whose line numbers differ from `bindgen_arm64.rs`, so a line-shift re-baseline verified only
  on arm64 can still fail `lint:amd64`. Re-baseline / verify on the SAME arch mix the gate runs
  (or just run `vibeos-cli check` for each arch). (2) **`committed_drift`**: cargo fix will
  happily "clean up" a committed **generated fallback** — an `unused_parens` removal in
  `winreg-stub/data/winreg_s.amd64.rs` (a widl-generated RPC server stub) byte-diverged it from
  regeneration → `lint:amd64` DRIFT. Generated/committed-fallback files (`*_s.<arch>.rs` RPC
  stubs, `cflags/*.flags`, `*.rc.res`, `bugcodes.h`, `ks*.inc`) must NEVER be hand- or
  cargo-fix-edited; revert any the sweep touched to `HEAD~1`. Detector: `python3
  tools/check_committed_drift.py --arch <a>` (13-14 artifacts, byte-compare vs cmake regen).
- **Validate cargo-fix batches with a CLEAN per-arch build, not a warm `-k0` (2026-07-17).**
  A warm/incremental `ninja -k0` recompiles the files you edited but may reuse a cached
  artifact for a crate it deems unchanged — so it can MISS an i386-cfg-`mut` or import
  regression the gate's clean build then catches. Batch 3 shipped a green warm i386 build
  but the gate's clean build failed: cargo-fix-amd64 stripped `mut` off
  `smss-csrss-rust/csrss.rs`'s `let mut Status` (i386 reassigns it in the
  `#[cfg(target_arch="x86")]` IOPL block; amd64 assigns once — the SAME csrss.rs case as
  batch 1). Restore from `HEAD~1` and re-validate with `vibeos-cli build --arch i386
  --clean` (full lint suite + stamp). Do the clean build BEFORE the gate so you don't burn
  a ~12-min gate cycle. This `mut`-strip class is the most common cargo-fix cfg hazard
  after import-removal — grep the batch for `^-.*let mut ` and spot-check any binding
  reassigned under an arch cfg.
- **Never suppress to green.** Removing a real warning by fixing it ✅; re-adding a blanket
  `#![allow]` to silence it ❌. If a warning is a genuine false-positive, `#[allow(...)]` it
  **per-item with a `// reason`**, never crate-root.
- **A fix that changes behavior is a bug, not a cleanup.** `cargo fix` is *supposed* to be
  behavior-preserving, but review every hunk — an "unused" `Result` it drops may be a
  swallowed error (`scan_silent_failures`' domain). When in doubt, keep the value and bind it.
- **Consider a gate at the end** — once a crate is clean, a lint that keeps it clean (deny the
  fixed classes for that crate, or a workspace `-D` for the ones fully eliminated) is what
  makes the burndown *stay* burned down. Design it so it can't be defeated by a new blanket allow.


## 2026-07-18 — the ud2-bomb detour (Phase 2 landing fallout) + boot green again

Landing Phase 2 surfaced a CLASS: same-module CC-mismatch calls fold to ud2 AND
drop their symbol ref (link passes BECAUSE the code is broken); scan_ud2 never
looked at drivers. Field manual **§9.99** (6 root patterns + detection recipe) and
**§9.100** (the cdrom 32-vs-18 sense-buffer pool overrun, found via the new
ex-rust `[POOL]` page-walk forensics). Truth-model addendum for the CC canon:
**bindgen_i386 decl class outranks nm/import-lib decoration for any symbol with
an in-crate shim def** (Phase 2 had this backwards for KeInitializeSpinLock ×7
crates). New crate `drvtrace-shim` = the fold-proof home for variadic-called
non-variadic debug shims (lto=false module isolation).

Owed from the detour: extend scan_ud2 to drivers (+KNOWN_DRIVER_UD2 baseline:
win32k UserRedrawDesktop/NtGdiGetRealizationInfo/co_IntSendActivateMessages,
cdfs CdPerformVerify, usbehci EHCI_OpenInterruptEndpoint, winsrv
ConDrvInitConsole — cold-branch residuals to root-cause); kernel-island
def-vs-bindgen mismatches (iomgr bootlog SD/CD, se SepDump* SD/CD, byteswap
FC/CD, config CmpAssignSecurityDescriptor — all link+boot today, audit before
touching); SPECIALPOOL tripwire never reaches the kernel on the windows-host
backend (no -fw_cfg on the qemu cmdline — harness gap).

Pass C (TYPE/SIGN unification) is built + dry-run-validated (362 syms / 517
rewrites / 49 crates, signal bucket 311 syms preserved): scratchpad ced_passc.py
+ ced_passc_repair.py; applies next.


## 2026-07-18 (late) — Pass C landed; T2a campaign state

Pass C (992af657162): 40 crates' in-crate decl signatures unified + 3 manual
ARITY fixes, all-arch clean, i386 boots. amd64 clashing 1363→1303. The
surviving ~1300 decomposes into exactly THREE root classes, none reachable by
text-level tooling:
1. **Import-scope resolution drift** (the bulk): identical decl TEXT resolving
   to different types per module (each module's `use` set differs). Needs
   resolution-aware rewriting (rust-analyzer/HIR) — a fully-qualified-path
   rewriter would also collapse most of class 2.
2. **Per-arch mirror pairs / CC strings**: root fix = bindgen fn-ptr ABI-string
   normalization to `"system"` via the i386-twin projection (ntkrnl-sys,
   NEEDS USER GREEN-LIGHT).
3. **Real-signal residue**: BOOLEAN-size crossings, fn-ptr-vs-ptr params,
   9 reverted usermode crates (advapi32/classpnp/cmlib/kernel32/lsasrv/msafd/
   samsrv/secur32/user32 — their unification bounced off compile errors),
   fastfat FsRtlNotifyFullChangeDirectory 7-variant knot. Reviewable set,
   ~passc-signal.json (session 57d7315b scratchpad).

Also owed from tonight: scan_ud2 driver extension (§9.99 has the recipe +
KNOWN_DRIVER_UD2 candidates); SPECIALPOOL tripwire not plumbed on windows-host
backend; kernel-island def-vs-bindgen audit (iomgr bootlog, se SepDump*,
byteswap fastcall, config CmpAssignSecurityDescriptor).

## 2026-07-18 (day): the IR-mismatch campaign — §9.99 tooled at source level, driver fleet swept

User green-lit both owed lanes (scan_ud2 driver extension; bindgen "system"
normalization). The scan_ud2 work immediately escalated: prototyping the binary
driver sweep surfaced ~10 named residual ud2s, and root-causing them produced a
**new detector** — `tools/scan_ir_mismatch.py` (§9.103) — that catches the whole
§9.99 class pre-fold in dev IR. The full i386 workspace sweep: **346 findings
across 44 crates** (refined to 250 true landmines once decl-vs-decl SIG was
proven non-folding).

Fix batches (all deterministic, zero subagents):
- **Proven-armed bombs fixed by hand:** winsrv `ConDrvCreateScreenBuffer`
  (bindgen_i386 decl class "C" vs "system" def — the ConDrvInitConsole ud2) +
  3 sig drifts; fastfat 5-symbol CC set (`ExReleaseResourceLite` fastcall truth,
  `IoGet*/IoSet*/IoCopy*` cdecl truth per nm+bindgen — 2-decl-class mixes);
  se `SepDump*` ×4 + config `CmpAssignSecurityDescriptor` def-flips to bindgen
  class; cdrom one-liner `KeInitializeSpinLock` (the §9.99 scanner-gotcha form,
  AGAIN); kd `KdIoPrintf` variadic-decl misuse in kdprompt (echo now via
  KdIoPuts — was printing raw "%c" format strings).
- **Batch B (VAR relocations):** 13 DPRINT-family no-ops + serial `TRACE_` moved
  to drvtrace-shim; new **winetrace-shim** crate for kernel32's Wine
  TRACE/FIXME/WARN/ERR (names too generic for the shared shim).
- **Batch A2 (scripted def-authority SIG alignment):** align decls to the def's
  IR shape with primitive-only tokens (the Batch A source-text transplant died
  on import-scope drift — see §9.102 recipe); compiler-driven cast repair +
  crate-revert convergence.

Late-day closeout: Batch A2 (IR-shape decl morphs, byte-mode span-guarded cast
repair) + hand-finish drove the driver fleet to ZERO unexpected binary ud2s.
The last 5 survivors were a NEW class — provable-null-deref folds (§9.103):
cdfs eager SEH-filter evaluation (both CdPerformVerify ud2s from one line) and
win32k unguarded null-blind derefs (UserRedrawDesktop, NtGdiGetRealizationInfo)
— all fixed with real guards, not baselines. Repair-tooling scars worth keeping:
rustc spans are BYTES (a `§` in a comment corrupted 3 classpnp files via char
offsets — stashed, guards added), and span-text verification before splicing is
mandatory (§9.103).

First-ever amd64/arm64 driver sweeps (gate leg 2) surfaced 15 more pre-existing
latents: ExitProcess/Fatal* terminal spins classified EXPECTED (deliberate
`for(;;);` after NtTerminateProcess); the win32k amd64 NtGdiGetRealizationInfo
twin got the same §9.102 null-guard; OWED root-causes baselined in
KNOWN_DRIVER_UD2 with writeups: user32 SwitchWndProc_common ×2 (error-path
folds after SetLastError), kernel32 arm64 time/locale spins ×7 (suspected
non-volatile KUSER_SHARED_DATA tick-capture retry loops folded to `b .` —
would hang if the subpath runs; POTENTIAL REAL arm64 HANG CLASS, prioritize),
JapaneseEra_Find, win32k EngInitializeSafeSemaphore.

**2026-07-18 (later): the fenced reviewed batch is PAID.** Def-authority
ret-drift / arg-width findings fixed individually (ntoskrnl-rust + hal-rust,
plus a bonus thrdschd CONTAINING_RECORD) — scan_ir_mismatch clean.

**CAUGHT BY THE GATE — a §9.99 fix in the WRONG DIRECTION (field manual §9.107).**
The batch first "fixed" MmDeleteVirtualMapping by flipping it to cdecl "C"
(trusting the no-NTAPI decl in internal/mm.h + bindgen). That is a boot-breaker:
its real i386 ABI is stdcall (@16) — the def and the hot caller MmFreeMemoryArea
reach it via a `type VirtFn = extern "system" fn(..)` transmute and call it
stdcall, so a cdecl callee desyncs ESP → exec-at-0 → BUGCHECK 0x1E during driver
init (i386-only, 3/3 deterministic; KTRAP_FRAME Eip=0 + pool-poison 0xbadb0d00 +
return addr MmFreeMemoryArea+408 nailed it). REVERTED to extern "system" at all
4 sites. The last live KNOWN_CGU_UD2 entry (MmPageOutPhysicalAddress) was a
side-effect of the SAME symbol: mm/rmap.rs imported it from the bindgen ddk
module (extern "C") while the def is stdcall → an intra-crate CC disagreement
that folds MmPageOutPhysicalAddress to a cold-path ud2. FIXED AT ROOT by giving
rmap.rs a LOCAL extern "system" decl (no ABI change) — kills the fold without
the cdecl boot-breaker. i386 build ✓ scan_ud2, i386 smoke ×3 green. All ten
KNOWN_CGU_UD2 entries verified 0-hit on i386/amd64/arm64 UP + amd64 ntkrnlmp SMP,
and the whole allow-set is RETIRED from scan_ud2.py.

Lesson banked: a missing NTAPI in a header is NOT proof of cdecl; when
scan_ir_mismatch flags "def system vs decl C", make the DECL match the def
(stdcall), not the def match the header — and a ud2 that vanishes when you
change an ABI is a red flag (you moved the fold), not a win.

The kernel32 arm64 time-fn spins were root-caused as
the non-volatile KUSER_SHARED_DATA seqlock class (field manual §9.106) —
read_volatile/write_volatile applied across kernel32/kernel32-vista/ke-inlines/
ntdll/ntoskrnl clock+time, all 8 kernel32 spins verified dead by arm64
re-sweep, KNOWN_DRIVER_UD2 kernel32 entry deleted. win32k
EngInitializeSafeSemaphore (same class, wait-loop flavor) and user32
SwitchWndProc_common ×2 (§9.103 null-blind deref of inlined GetWindowLongPtrW
error return) fixed the same day; baseline entries removed after binary
re-verification.

**#14 (bindgen "system" normalization) — ATTEMPTED, DEFERRED 2026-07-18.** The
green-lit generated-file normalizer (`tools/rustconv/normalize_bindgen_abi.py`,
preserved in scratchpad `DEFER-*`) is sound and takes i386 clashing to ZERO
(the arch with the real §9.99 ud2-fold risk). But applying it forces a
consumer migration (every fn item / fn-ptr type that meets a normalized value
must also be "system"), and that ripple is UNBOUNDED across the workspace ABI
surface AND collides with genuinely-cdecl callback populations (ACPICA's
`acpi_bus_*_handler`, CRT `rand_s`, driver `DRIVER_DISPATCH` indirection) that
CANNOT be "system" without breaking i386 ABI. rustc keeps `extern "system"`
and `extern "C"` nominally distinct for fn-pointer coercion even where
machine-identical, so there is no partial landing: an arch is either fully
normalized or fully un-normalized. i386 reached green; amd64/arm64 kept
surfacing new cdecl-collision categories with no convergence. Reset to HEAD +
re-applied only the solid #15/spin work. NEXT: redesign #14 as **i386-bindgen-
only** (normalize just `bindgen_i386`/`types_i386` stdcall→system — kills the
i386 clash + ud2 risk with ZERO consumer migration, since i386 hand-written is
already all-"system"; leaves the harmless amd64/arm64 cosmetic clashes), OR a
per-crate staged migration that hand-separates NTAPI from cdecl. Do NOT re-run
the blanket amd64/arm64 projection.

Still owed: classpnp/disk Shape-2 VAR items (real fns with
drifted arity: CreateStorageRequestBlock, RtlStringCbPrintfW, sprintfW…) plus
the revert-bounced decl files in fastfat/classpnp/cdrom/cabinet (landmines, not
armed folds — IR count 250→~30); freeldr-rust IR emission fails for i686
(rosload halstub mangled ud2 unexplored).

## 2026-07-24 — the catch-all dispatch-arm sweep (task #16) + two gate-integrity fixes

Post-merge regroup (24 commits of mainline: tier-2 services, iphlpapi/netapi32/dhcpcsvc
full flips, shell32, wow64 E9, arm64 fast-mutex). Fresh amd64 census: **5,193 warnings /
136 crates**. `unused_assignments` 1435 · `clashing_extern` 1269 · `unused_imports` 278 ·
`static_mut_refs` 261 · `unused_unsafe` 255 · **`unreachable_patterns` 226** ·
non_snake_case 235 · unexpected_cfgs 173.

**`unreachable_patterns` turned out to be a live-bug class, not noise.** Full writeup:
field manual **§9.112a**. Summary of what this lane did:

- **The oracle was reporting ✓ 0 on a tree with 273 real hazards.** `verify_match_consts.py`
  had two independent scope defects: it scanned **`ntoskrnl-rust/src` only** (the driver fleet
  and every usermode DLL — where §9.112 says the class recurs — were never looked at), and it
  resolved constants **workspace-wide** (a name defined in any crate counted as reachable from
  every crate, so real catch-alls were filed LOW and suppressed).
- **Tool rewritten** (`tools/verify_match_consts.py`): default scope = whole workspace;
  resolution = each crate's own sources + transitive **path deps**; glob imports resolved to
  their actual modules (following `#[path]` + `pub use types_arch::*` chains) instead of
  "has a glob → probably fine"; unresolvable glob targets fail OPEN (lax), never closed.
  `--workspace-scope` keeps the old laxer behaviour. Validated against a `--force-warn`
  rustc oracle: **0 false positives, 5 conservative false negatives** on 175 confirmed sites.
- **Shape A (constant exists in-crate, file never imports it) — 175 → 2, all fixed:**
  classpnp 126, fs_rec 18, partmgr 16, pcix 10, scsiport 5. classpnp went from **140
  `unreachable_patterns` and 127 catch-alls to ZERO**, clean ×3 targets. Notable: classpnp's
  WMI dispatcher needed the five `IRP_MN_*` codes defined locally (they exist only in the
  x86-only `ntkrnl_sys::hal_ddk`, and classpnp also builds arm64); `class.rs` was missing
  `DeviceUsageTypeHibernation` from its import list, which killed the three arms after it.
  The residual 2 are `#[cfg(feature = "NTDDI_WINTRHESHOLD")]`-gated firmware IOCTLs — latent,
  no build compiles them, and their only known value in-tree is a `// typical` guess.
- **Shape B (constant does not exist anywhere in the crate closure) — 271 owed, baselined.**
  samsrv 121, serial 37, lsasrv 28, iphlpapi 17, dnsapi 17, scsiport 8, mscms 6, i8042prt 6,
  dhcpcsvc 5, arm64input 5, ramdisk 4, pciidex 4, … These need the constants defined — the
  §9.112 root fix (allowlist the header owning them in the crate's `gen_driver_types.sh`
  profile), not an import edit. `serial.sys` is the headline: every `DeviceIoControl` currently
  runs the `IOCTL_SERIAL_CLEAR_STATS` handler. **Baseline `verify_match_consts.txt` recorded at
  273** — this is tracked debt, NOT acceptance; it is the next batch of this task.

**Two gate-integrity fixes found on the way (both AGENTS.md "fail loud" violations):**
- **`verify_signatures` reported 987 phantom NEW findings.** It needs generated headers from
  `build-<arch>/`; this worktree has none, so it fell back to a sibling checkout — which only
  had `build-arm64-cross`. With `-I <nonexistent>/build-amd64/ntoskrnl`, clang emits
  implicit-int decls (`int KiSetProcessorType(VOID *)` for a `VOID f(VOID)`) and every export
  looks mismatched. It degraded silently instead of failing.
- **The sibling fallback itself is the deeper footgun** (user call, and it proved itself
  immediately): making the search arch-aware made it pick `vibeos-conformance/build-amd64` —
  another lane's tree at another commit — and the count moved **987 → 9 with no source
  change**. Removed the sibling fallback for generated headers in `verify_signatures.py`,
  `verify_offsets.py` and `verify_offsets_win32k.py`; all three now HARD FAIL with
  `fix: ./vibeos-cli build --arch <a>`. The toolchain (`.build-tools`, a pinned clang binary)
  fallback stays — it is commit-independent. Durable rule recorded as **AGENTS.md worktree
  discipline #10**: fleet isolation applies to READS, not just writes.

**Next in this task:** Shape B, biggest-first (samsrv 121, serial 37, lsasrv 28), via the
bindgen-island allowlist fix rather than per-file imports.

## 2026-07-25 — Shape B: 273 reported → 88 rustc-confirmed → 0 live

Continuation of task #16. The headline is a correction: **most of the 273 were my own tool's
false positives.** Cross-checking `verify_match_consts.py` against a rustc oracle
(`RUSTFLAGS="--force-warn unreachable_patterns" cargo check --workspace`) BEFORE editing any
source showed samsrv (121) and lsasrv (28) emitting zero unreachable patterns — they were fine.

### Four scanner defects, all fixed (field manual §9.112b)

| defect | effect | delta |
|---|---|---|
| explicit `pub use … as Alias` re-exports not harvested | reported the bug the prelude already fixed | -190 |
| attribute on the same line hid the glob (`#[allow(unused_imports)] use crate::glue::*;`) | an *unseen* glob leaves `all_resolved` true → **confident** HIGH | -13 |
| enum-variant regex consumed its own delimiter | every OTHER variant dropped (`Unknown` ✓ `Keyboard` ✗ `Mouse` ✓) | -6 |
| dead files scanned as live | dhcpcsvc `dhcp/tree.rs` is upstream-dead, excluded in lib.rs | -6 → new ORPHAN class |

And rustc caught a class the tool was structurally blind to: **`_FSCTL_LOCK_VOLUME`**. A leading
underscore still binds — it only silences `unused_variables`, which is exactly the lint §9.112a's
oracle was built on. cdfs had 8 FSCTL dispatches dead behind one.

### The live bugs actually fixed (88 rustc-confirmed → 0)

- **serial.sys** — all 37 `DeviceIoControl` arms dead behind `IOCTL_SERIAL_CLEAR_STATS`.
- **cdfs** — 8 volume FSCTLs dead behind `_FSCTL_LOCK_VOLUME`; 2 more IRP_MJ arms in `fspdisp.rs`.
- **classpnp** — the `IOCTL_STORAGE_FIRMWARE_ACTIVATE` arm (a function-local const in class.rs,
  invisible here). NOT the 8 storage-property arms I first reported: those were already imported,
  and the finding was an artifact of my own edit landing inside the multi-line `use` list (a
  `const … ;` there truncates the scanner's import parse). rustc never flagged them.
- **kernel32** — Ctrl-Close handler ran for Last-Close/Logoff/Shutdown.
- **i8042prt** — the FDO took the keyboard path for a mouse.
- **arm64input** (5), **oleaut32** (6), **services** (4), **netapi32** (4), **mscms** (6),
  **smss-csrss** (3), **msafd** (2), **winsrv**, **winlogon**, **dhcpcsvc**, **cdfs volinfo**.

### Two new deterministic transcribers (field manual §9.112c)

`tools/gen_ioctl_consts.py` and `tools/gen_enum_consts.py`. Values are COMPUTED from the header,
never hand-typed (§9.113), and both have a `--check` mode that re-verifies committed constants.
That mode paid for itself immediately: it confirmed the 9 existing `FSCTL_*`, all 18 `IRP_MJ_*`,
and classpnp's hand-guessed `// typical` firmware IOCTL are correct.

Completed two incomplete constant families in `ntkrnl-sys` along the way (8 volume FSCTLs, 11
IRP_MJ codes), deliberately skipping the value-aliases `IRP_MJ_SCSI` / `IRP_MJ_PNP_POWER` —
defining a second name for the same value turns a sibling arm unreachable.

### Oracle policy going forward

**rustc is the gate, the scanner is the pre-build hint.** `unreachable_patterns` is precise, free
(we are already compiling) and sees through every glob/alias/cfg the scanner must approximate. Its
blind spot is code not compiled for the checked arch (arm64input's 5), which is where the scanner
earns its keep. Residual in-tree `unreachable_patterns`: **3**, all benign redundant `_ =>` after
an exhaustive match on a real Rust enum (inbv ×2, kernel32 `find.rs` ×1).

`tools/lint-baseline/verify_match_consts.txt`: **273 → 27** (all 27 ORPHAN, dead-file entries),
and since then **→ 0: the baseline is EMPTY and the gate is strict.** Zero live catch-alls remain.

### Side finding: orphaned translated files

The module-tree walk added for ORPHAN classification doubles as an orphan detector — files that
grep as live but no `mod` reaches, so `cargo check` passes while the link still fails (the
`class_gap3.rs` trap from the previous batch). Current set: classpnp `class_gap3.rs`/`class_gap4.rs`,
dhcpcsvc `dhcp/tree.rs`, ws2_32 `src/async.rs`, win32k `bitblt_new.rs`/`drvdbg.rs`,
ntoskrnl `ARM3/virtual.rs`/`global_barrier.rs`, afd `event.rs`. Not triaged in this batch.

### Landing note: two gate cycles lost to a type gap

`cargo check -p <crate> --target <t>` passed oleaut32 and smss-csrss; the real build failed both
with `E0308`, because the `*-cargo` wrapper compiles them with a different feature set and the
scrutinees are narrow C fields (`LOWORD(...)` → u16, `USHORT EntryIndex` → u16) while a
transcribed constant defaults to `u32`. Fixed per the §9.112a rule: widen the scrutinee where the
match has no narrow siblings (smss-csrss, oleaut32's MSHCTX), join the siblings where it does
(oleaut32's `VT_CARRAY` → `consts.rs` as `u16`, next to `VT_PTR`). Verified all touched crates
against all three targets afterwards. Recorded in the field manual under §9.112c.

### Baseline delta: `scan_silent_failures` +78/-57, and only 57 of it is mine

The gate runs `check --strict`, where NEW findings are fatal (a plain `check` shows them as ⚠ and
still exits 0 — do not mistake a green standalone `check` for a green gate). Re-blessing this
baseline needs the delta explained, so:

- **57 are pure line shifts from this batch** — 36 msafd `dllmain.rs`, 17 winlogon `sas.rs`,
  4 classpnp `class_part_b.rs`, each with a matching removal. Balanced +N/−N per file is the
  established proof of a shift rather than a new finding.
- **21 are PRE-EXISTING drift, not introduced here** (plus one non-finding count line the
  baseline does not carry): msgina `shutdown.rs` 7 / `gui.rs` 5, explorer `appbar.rs` 3,
  ks `api.rs` 2, kernel32 `client/heapmem.rs` + `client/debugger.rs`, win32k `drvdbg.rs`,
  cdfs `deviosup.rs`. Verified by stashing this batch and re-running against a clean HEAD:
  **22 added / 0 removed**, deterministic across two runs. They arrived with the mainline merge
  and were never blessed. They are real `.unwrap()`/`.expect()` findings, so blessing them adds
  them to the campaign work queue this baseline doubles as — but they are *not* this lane's work
  and should not be read as such.
