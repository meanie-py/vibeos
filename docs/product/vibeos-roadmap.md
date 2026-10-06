# vibeos Roadmap — Future Enhancements

*"An operating system for nobody"* — built by AI, verified by compiler, stress-tested at kernel level.

Items below are drawn from ChatGPT planning sessions and ongoing work. Scoped with t-shirt sizes: **S** (days), **M** (1-2 weeks), **L** (weeks-months), **XL** (quarter+).

---

## Phase 0 — The near-term (recalibrated 2026-07-07)

The kernel + win32k + the core client-DLL/boot spine + the driver fleet are **~90–100% translated and boot to desktop on all three arches** (single source of truth: [`../engineering/rust-port-status.md`](../engineering/rust-port-status.md); completed history: [`vibeos-roadmap-complete.md`](vibeos-roadmap-complete.md)). So the near-term is **not** "finish the translation before this roadmap" — it's a deliberate interleave of *finishing the systemic unlocks* with *resuming translation*.

### Why priorities look "pulled forward" — the Fable window

**Claude Fable** (a higher-frontier model) was bundled in the subscription **through 2026-07-07**, so we used that window to have it tackle the hard systemic/enhancement work, **set the reusable patterns**, and prove concepts — pulling that work *ahead* of translation because a stronger model was briefly available and excelled. From today we are **Opus-only**: the job now is to **follow Fable's patterns** to land the burndowns/unlocks, then **resume translation as the through-line**. Priority *un*-shifts back to translation — *except* where a Fable-unlocked thread is high-value enough to see through.

**Fable-window ledger (state of each pulled-forward thread):**

| Thread | State | Disposition |
|---|---|---|
| ntoskrnl **crate-split** (build velocity) | ~63% (21 crates, monolith 456K→170K); ex parked on x86 hang; ke/mm remain | **continue** (Opus-doable) — [`../engineering/ntoskrnl-crate-split-plan.md`](../engineering/ntoskrnl-crate-split-plan.md) |
| **`--allow-multiple-definition`** elimination | spike + docs proven | **see through** (high value) — [`../engineering/build-system.md`](../engineering/build-system.md) |
| **DUMMY_PSEH** resolution | spike + docs, direction open | **see through** (high value) |
| **Doom on vibeos** / screen-painting / perf | showcase landed | done — winding down |
| **IRQL/spinlock RAII** audit | 0 leaks; 130 candidates catalogued | **park** — adopt incrementally ([`../engineering/archive/irql-raii-conversion-queue.md`](../engineering/archive/irql-raii-conversion-queue.md)) |
| **sync-instrumentation** (lockdep/waitdep/cswitchdep) | landed — cswitchdep all 4 phases done, lane closed 2026-07-11 (×3 arches, all 5 taps + decoder + farm) | keep as always-on tripwires; follow-ons in the ledger |
| the **linter suite** (ghost-tax Tier 1) | landed + 0-NEW gated | burn stock opportunistically |

### Tier 1 — Finish the high-value unlocks

- **`--allow-multiple-definition` elimination** — kill the load-bearing link flag categorically (spike proven; see the ratchet doc + `tools/scan_duplicate_symbols.py`). **2026-10-04: off for the five CRT DLLs** (`link_strict`, build-system.md) -- one Rust graph each, no C, no clang_rt; the pattern for the next modules (a module qualifies once its C is gone and its Rust archives are one graph).
- **DUMMY_PSEH resolution** — decide + execute the path off the SEH-is-dead-code hack across
  all arches (spike + open-question doc landed). **It does NOT block the system-cursor bug.**
  **Update 2026-09-23:** the cursor/icon block this note defended skipping now runs where
  upstream runs it (`co_IntInitializeDesktopGraphics`), on all arches. The skip was ours
  (`5802b15ed60`, 2026-03), not a C `_USE_DUMMY_PSEH` guard, and its stated reason was wrong.
  The thread does have a user-callback table: all three arches log
  `cursors/icons loaded at desktop-graphics init`, and the gates passed 9/9. The i386 hang
  the skip avoided was the padded `KAPC_STATE` copy that made threads read as suspended.
  `co_IntDeferredCursorsAndIcons()` remains as the fallback for a thread without a callback
  table.

### Tier 2 — Systemic Rust-native fixes (above Wine-DLL translation)

- **⭐ Exported-symbol trampoline refactor** *(new; Size: M, mechanical)* — split every exported symbol into a snake_case **native `nt_foo()`** (plain Rust ABI, holds the logic) + a generated **`NtFoo` trampoline** that is the *only* place `extern "system"/"fastcall"/stdcall` lives. Done mechanically in Python across all ~1,700+ monolith exports (thousands fleet-wide). **This is the capstone of the ghost-tax "two copies of the truth" program**: internal callers hit the native fn, so there is no hand-written `extern` decl left to drift — it kills the ABI-mismatch bug class the crate-split flushes cross-crate, `scan_cc_mismatch` catches same-crate (316 baseline), and `verify_ffi` guards (161). Also the natural substrate for idiomation (§3) and for **generating** crate-split's cross-crate decls instead of hand-writing them — so do it *aware of* the crate-split. **Re-scoped 2026-07-25 — Size is L-XL, not M: the real surface is 24,823 `#[no_mangle]` exports fleet-wide** (the ~1,700 figure was ntoskrnl-only, and is accurate for it; win32k alone is 3,788). It also cannot be an edit campaign over the decls that exist — it must GENERATE them from one authority and retype call sites in the same pass. Two shortcuts were probed and closed as dead ends (decl-follows-def bakes in wrong ABIs; `verify_ffi`'s 6 ABI findings are correctly baselined local shims, not bugs) — see the warnings-burndown lane plan, 2026-07-25, before restarting.
- **msvcrt / ucrtbase from pieces** *(Size: M-L; msvcrt DONE; ucrtbase DONE 2026-10-03 -- no Microsoft source or headers left)* — the CRT floor is still ~581 `.c`. Rather than translate it from scratch, **assemble** the Rust CRT from what already exists — `cnt-rust` (libcntpr kernel CRT subset) + the 39-crate `rtl-rust` umbrella — and NT-ify the gaps. Displaces the last big userland C floor. **ucrtbase now links Rust**: its shared TUs live in `crt-shared-rust` and it links `crt-shared-cargo`, deliberately NOT msvcrt's archive — those two collide with ucrtbase.spec on 891 names of which only 35 are ours, so the obvious wiring would have silently shadowed 856 genuine Microsoft UCRT bodies under `--allow-multiple-definition`. `tools/verify_ucrtbase_cohab.py` gates it; see rust-port-status.md's CRT row.
- **ntoskrnl crate-split continuation** *(Size: M)* — debug + land **ex** (x86 hang, branch `ex-crate-split-x86-hang`), then split **ke** and **mm/ARM3**; then **win32k** (flat 257K single crate, textbook USER/GDI seam, `freetype` a free leaf — one coordinated pass in a quiet window). Detail + recipe: the crate-split plan.

### Tier 3 — Resume translation (the through-line)

Genuine remnants only — the boot path is already Rust. Measured 2026-07-07 from `media/doc/WINESYNC.txt` provenance + `wine/`-subdir + the live `*-rust` crate set.

**ROS-native userland — ~140 components, ~240K native LOC (the priority queue).** Batched by shape for rustconv (a name-collision caveat: a few like `smss`→`smss-csrss` are already done; the count is a mild over-estimate):

| Batch | Top targets (native LOC) |
|---|---|
| **Winsock / net client** | ws2_32 (14K) ✅, wshtcpip ✅, ws2help ✅, mswsock ✅, wshirda ✅ — all **cohab-flipped to Rust** (2026-07-17/18, recipe in the wshtcpip commit); **iphlpapi ✅** (FULL flip 2026-07-18 — all 11 TUs; only the generated `iphlpapi_stubs.c` is C), **dnsapi ✅** (FULL flip — all 8 TUs; `adns` staticlib + the one `dnsrslvr` MIDL interface stay C), **dhcpcsvc ✅** (FULL flip — all 12 TUs, the DhcpC* API + the ISC dhclient state machine; MIDL `*_c/_s/_stubs.c` + a varargs trampoline stay C), **netapi32 ✅** (FULL flip — the 7 RPC-stub tail landed 2026-07-18, completing 21/21 TUs; the long-standing "6 conflicting MIDL interfaces" blocker was a MISDIAGNOSIS — the real conflict was `<lm.h>` vs the WIDL stubs, field manual §9.108 — resolved with two bindgen islands via the new `TYPES_STEM` mechanism). **The winsock/net client batch is COMPLETE — every DLL in it is fully Rust apart from generated MIDL/spec stubs.** Remaining networking work is the services tier (dnsrslvr, wkssvc, browser, tcpsvcs, telnetd, wlansvc), the untouched client DLLs (wsock32, wlanapi, snmpapi, mprapi, ras*, netcfgx, netshell, wldap32), and the ~28K-line driver tier (ndis + the netkvm/e1000/rtl8139 miniports + ndisuio/netio/tdi) — the latter deliberately deferred as a real-NIC milestone, since netstack drives virtio-net directly and bypasses NDIS today. |
| **Setup / PnP / device** | devmgr (10K), netcfgx (9K), syssetup (9K), umpnpmgr (7K), newdev, msports |
| **Control panels (`dll/cpl`)** | desk (11.5K), sysdm (7.9K), intl (5.2K), mmsys (3.8K), timedate, access, main, usrmgr, powercfg, input — ~15 CPLs, uniform shape, very batchable |
| **Services (`base/services`)** | eventlog (3.5K), netshell (4.5K), svchost, wkssvc, dnsrslvr, audiosrv, wlansvc |
| **Security / auth** | msv1_0 (3.3K), samlib (2.4K), ntmarta (1.8K), aclui (5.7K), authz — completes the LSA/SAM chain already in flight |
| **Shell exts / apps / audio** | zipfldr, shimgvw, fontext, ntobjshex, stobject; console (3K), diskpart (9.3K); wdmaud.drv, mmdrv, the `*.acm` codecs |
| **misc system DLLs** | userenv (6.5K) ✅ (cohab-flipped 2026-07-18), psapi (1.7K), framedyn, hhctrl.ocx (5.2K), fmifs |

**Wine-derived userland — ~154 DLLs, ~1.05M LOC (lower priority than Tier 2).** kernelbase (41K, `kernelbase/wine/`), comctl32, comdlg32, ole32, oleaut32, gdiplus, riched20, urlmon, wininet, mshtml, jscript/vbscript, msxml*, windowscodecs, crypt32, wldap32, rpcrt4, the whole `dll/directx/wine/` stack. Re-implementing these from *current* Wine via an LLM pipeline (§7b) beats translating the frozen fork — that's why they sit below the native queue.

**Microsoft DDK storage class drivers — 3 drivers, ~90.5K LOC (added to the queue 2026-08-08).** `classpnp` (43.7K), `cdrom` (34.1K), `disk` (12.7K). Foreign-copyright but **ours**, and recorded as such in `vendor.yaml`'s `ours:` half — MS-PL grants derivative works, ReactOS has already diverged them (13 of classpnp's 24 files carry local edits), and unlike WDF next door they expose no ABI that third-party binaries bind to, so there is no compatibility contract that would make owning them a liability. They were in no count before the classification sweep found them (task #62). Pointer-heavy IRP dispatch on the boot path for every disk, so they rank with the native queue rather than below it.

**Already translated (frontier boundary):** kernel/win32k, kernel32, ntdll, user32, gdi32, advapi32, secur32, shell32's complete activatable COM surface (its procedural C++ tail remains), msafd, mlang, mscms, version, cabinet + the boot/server spine (winsrv, smss-csrss, lsasrv, samsrv, msgina, services, winlogon, userinit, explorer) + the driver fleet.

### Tier 4 — Debt burndown (gated at the door; burn opportunistically)

Every class below is already **0-NEW-gated** by `vibeos-cli check`; the *stock* is owed, none of it is urgent. Framework: [`../engineering/archive/ghost-tax-diagnosis-and-program.md`](../engineering/archive/ghost-tax-diagnosis-and-program.md).

- **`scan_silent_failures` (2086)** — bare `loop{}` spins, `unwrap`/`expect`, swallowed `Err` in kernel/driver code.
- **`scan_cc_mismatch` (316)** + **`verify_ffi` (161)** — ABI-decl drift; **largely retired for free by the Tier-2 trampoline refactor.**
- **`lint_kernel` (1473 `write_volatile`)** — **likely mostly obsolete**: the captures(none) miscompile it defends against is toolchain-fixed on LLVM ≥ 21 (rustc 1.92 / LLVM 21.1.3). Decision owed: retire-and-revert the defensive laundering vs keep as belt-and-suspenders (`tools/scan_captures.py` is the post-toolchain-bump tripwire).
- **Duplicate-symbol cleanup after the link guard (2026-10-04)** — in this order:
  (1) ~~the ~160 kernel probe sites~~ DONE (bug registry, "Kernel user-pointer probes");
  (2) ~~delete the per-module `--allow-multiple-definition` copies~~ DONE (35 removed;
  browseui's upstream LTCG-only one stays); (3) **one Rust crate graph per binary**, so the Rust
  panic runtime stops being an allowed duplicate. Scope, inventory and tiers:
  [`../engineering/one-rust-graph-plan.md`](../engineering/one-rust-graph-plan.md).
- **IRQL/spinlock RAII (130 candidates)** — 0 leaks today; adopt hand-verified as files are touched (block-scope discipline — the "trivial" labels are ~50% wrong).

---

## 1. SQLite-Backed Registry

Replace the NT registry hive format (binary blobs with transaction logs) with a SQLite database.

**Why:** The registry is a hierarchical KV store with transactional semantics — almost a textbook SQLite workload. SQLite gives atomic commits, checksummed pages, crash recovery via WAL, and eliminates the entire hive flush/repair subsystem. No more chkdsk-style hive repair. Rollback to last known good becomes `ROLLBACK`.

**What changes:**
- New `sqlitereg.sys` (or integrate into cmlib) that implements `CmRegisterCallback`-compatible registry operations backed by SQLite
- Schema: `nodes(id, parent_id, name, type)` + `values(node_id, name, type, data)`
- Boot path: FreeLDR loads the `.db` file into ramdisk alongside ntoskrnl; kernel opens it during CmInit
- WAL mode with periodic checkpointing; `-shm` eliminated via exclusive locking (single writer in kernel)
- Migrate existing `.inf`-based hive generation to produce SQLite DB instead

**NOT a filesystem.** Just the registry. The system volume stays as-is (FAT32 ramdisk for ARM64, LiveCD ISO for x86/amd64).

**Risks:** SQLite in ring 0 means a SQL parser + B-tree + pager in kernel space. Bugs become bugchecks. Mitigated by SQLite's battle-tested robustness and the fact that we already run Rust in the kernel.

**Size: L** — Core registry operations are well-scoped, but the boot-time integration and hive migration are significant. Probably 2-3 weeks of focused work.

---

## 2. Rust Translation Pipeline Evolution

The rustconv pipeline has evolved from file-level translation to dependency-aware per-function semantic lowering. Next steps:

### 2a. Header Semantic Reconstruction — Size: M

The next scaling bottleneck. C's semantic information lives outside translation units (headers, macros, build system, conventions). The pipeline needs to reconstruct this context more completely:
- Expand the ntkrnl-sys catalog beyond bindgen (FORCEINLINE functions, macros, preprocessor constants)
- Auto-detect `gap_functions.toml` entries from compile errors
- Parse `#ifdef` conditionals to understand arch-specific code paths

### 2b. `cargo check` → Auto-Repair Loop — Size: M

We're ~80% there. Compiler errors are structured data. The postprocessor already fixes type names and imports. Next:
- Classify remaining error patterns (lifetime, mutability, unsafe boundary)
- Generate targeted fixes per error class
- Run iterative `translate → check → fix → check` until green (with a cycle limit)

### 2c. C-Shaped Rust → Idiomatic Rust — Size: L

Current translations are direct C transliterations (`unsafe` everywhere, raw pointers, manual resource management). Eventually:
- Replace manual `ExFreePool` patterns with RAII guards (`PoolAllocGuard`, `ObjRefGuard`)
- Replace `Status < 0` chains with `Result<T, NTSTATUS>` + `?` operator
- Replace raw pointer arithmetic with typed wrapper structs
- This is a LATER concern — correctness first, idioms second

> **Concrete design → [`../engineering/idiomatic-rust-idiomization-design.md`](../engineering/idiomatic-rust-idiomization-design.md)** (2026-07-16). The executable plan: a **two-phase** program — (1) a *mechanical* trampoline breakout (`rustconv breakout`, an AST transform, free + behavior-preserving, splits every `#[no_mangle] extern fn` into a thin ABI trampoline + a `_rs()` impl, lands gate-once across ~20–24K exports); (2) *incremental* idiomization of `_rs` to `Result<(T,NTSTATUS),NTSTATUS>` + references. Includes the **reachability ceiling** (public DDK ABI stays raw — `DeviceExtension` 15.6K, `IoStatus`, stack-locations; private plumbing idiomizes now — MMVAD/MMPFN/KWAIT_BLOCK/handle-table), the **IRP façade/DTO experiment** (idiomatic interior + `#[repr(C)]` mailbox, reconcile at owned API seams; soundness test = *async observation must be API-gated*), the **compile-collapse design rule** (hot-path primitives generic/`#[inline]` to cross the `lto=false`/`cgu=256` firewall; `#[no_mangle]` only at the ABI wall), and the **arm64/x86 asymmetry** (shared driver source shackles arm64 to x86's WDM ABI; the internal/external split is what lets it diverge). Sequencing: (a) finish translating → (b) idiomize *ABI-preserving* [this] → (c) driver-model XP→Win10 [§7-adjacent, separate later project, the only ABI-moving one].

### 2d. Cross-Model Review Integration — Size: S

Already proven with Codex. Formalize into the pipeline:
- After translation, run verify_constants.py automatically
- Optionally run a different model (Codex, Gemini) for review/audit pass
- Track which model caught which bug class for pipeline improvement

---

## 3. SEH → Result/RAII Idiomation (Pipeline Stage 3)

Already designed, partially implemented. Transform `_SEH2_TRY`/`_SEH2_EXCEPT` patterns (which are dead code under DUMMY_PSEH) into proper Rust error handling.

**What exists:**
- `probe.rs` API: `probe_read_user`, `probe_write_user` → `Result<T, NTSTATUS>`
- SE (12 files, 62 sites) + OB (6 files, 35 sites) already migrated
- Pipeline: `rustconv probe-migrate`

**What's next:**
- Full SEH idiomation: function-level C+Rust paired transformation
- RAII guards for spinlocks, resources, object references
- `safe_copy_to/from_user` replacing bare `__try` blocks
- Dependency graph (SQLite cache) for function-level context
- Target PS subsystem next (13 files, heavy SEH usage)

**Size: L** — The infrastructure exists but applying it across all subsystems and handling the edge cases (nested SEH, cross-function exception flow) is significant work.

> **Note:** the `_rs()` output shape and the two-phase mechanical-breakout-then-idiomize structure are unified with §2c in [`../engineering/idiomatic-rust-idiomization-design.md`](../engineering/idiomatic-rust-idiomization-design.md). Stage 3 (SEH/Result/RAII) and §2c (raw-ptr → idiomatic) are the same `_rs` de-transliteration seen from two angles.

---

## 4. Testing Infrastructure Modernization

> **Inline-asm safety — design → [`../engineering/asm-safety-design.md`](../engineering/asm-safety-design.md)** (2026-07-16, Size: M). ~850 asm blocks split into two regimes: **pure** (barriers/atomics/portio — already Kani/loom-tested in `slist`) and **privileged** (CR/MSR/cli-sti/context-switch — `#GP` in ring 3, untestable in userspace). Three moves: (1) an **asm-constraint linter** (the `verify_ffi` of asm — cross-check instructions vs declared `options`/clobbers; catches the CMPXCHG16B-`rbx` under-declared-clobber class that cost a multi-hour 0x50); (2) **one-wrapper-per-instruction** consolidation (~850 sites → ~40 primitives) + prefer `core::arch` intrinsics; (3) test the privileged half by **shrinking the asm to its irreducible instruction** (test the pure logic, the trampoline lever) + the existing **QEMU ring-0 harness** (`test_kernel.py`) + extending the **`verify_sacred_primitives` "survived optimization"** gate.

Current testing: QEMU smoke tests (boot to desktop) + Rust host tests (unit + Kani + Loom). This is necessary but insufficient.

### 4a. Extract-and-Test Kernel Algorithms — Size: M

Many kernel subsystems are "just algorithms + data structures" wrapped in pointer manipulation. The logic can be extracted and tested in isolation:
- MM page table management (algorithmic core extractable)
- VAD tree operations (AVL tree, pure logic)
- Pool allocator free list management
- Object manager handle table (hash table + reference counting)
- PnP device node state machine

**Approach:** Create `#[cfg(test)]` modules that can run the algorithm layer without kernel context. Use the existing Kani/Loom pattern from SList as the template. AI can generate these tests in bulk — matching current behavior (including bugs) matters more than finding bugs at this stage.

### 4b. Property-Based Testing with Kani — Size: M

Expand beyond SList. Good candidates for exhaustive verification:
- Pool allocator: allocation/free roundtrips, coalescing, alignment
- Object reference counting: no double-free, no use-after-free
- Handle table: insert/lookup/delete consistency
- Spinlock state machine: acquire/release pairing

### 4c. Structured Kernel Test Framework — Size: M

The existing 28 QEMU kernel tests use a serial dispatch loop. Expand:
- More subsystem coverage (MM, IO, PS tests)
- Automated test generation from C test suites
- Per-commit regression gate (CI-like, even if manual)

---

## 5. Semantic Plateau Mitigation

The "semantic plateau" is approaching: compile success stops correlating with runtime correctness. The constant-fix work (154 wrong values!) is a preview — code compiled fine but used wrong values at runtime.

### 5a. Runtime Invariant Checking — Size: M

Add lightweight runtime checks that catch semantic bugs:
- IRQL assertions at function entry/exit
- Pool tag validation on free
- Object type checks on handle dereference
- Lock ordering validation (detect ABBA deadlocks)

### 5b. Differential Testing (C vs Rust) — Size: L

For subsystems with both C and Rust versions available:
- Run both, compare outputs at function boundary
- Log divergences for investigation
- Use `--allow-multiple-definition` to keep C as shadow implementation

### 5c. Boot Trace Catalog Expansion — Size: S

Expand debugcon trace points beyond KE init:
- Add trace points to IO init, PnP enumeration, PS init
- Generate trace catalogs for amd64 (currently only i386 has 48 trace points)
- Automated triage reports on boot failure

---

## 6. Modernize Core Data Structures

NT kernel internals use 1990s-era synchronization primitives. Some can be modernized without breaking the DDI contract.

### 6a. Atomic Reference Counting — Size: M

Object manager reference counting should use atomic reference counts everywhere instead of spinlock-protected increments. Already partially done (SList uses atomics). Extend to:
- `ObfReferenceObject` / `ObfDereferenceObject` — already atomic on amd64
- Handle table reference management

### 6b. Lock-Free / Wait-Free Structures — Size: L

Replace selected spinlock-protected structures with lock-free alternatives:
- MFN (page frame number) database: lock-free per-page state transitions
- Pool allocator lookaside lists: already interlocked SList (done!)
- PFN free lists: could use lock-free queues

**Caution:** NT spinlocks are deeply tied to IRQL semantics. Cannot casually introduce blocking primitives where IRQL raise was the synchronization model. Each replacement must preserve IRQL contracts.

### 6c. Futex-Style Primitives — Size: XL

Replace kernel pushlock/fast mutex internals with futex-like wait primitives. This is a deep architectural change:
- Pushlocks already use a form of this (spin → wait transition)
- Real futexes need kernel wait queue infrastructure
- Benefits: better power efficiency (no spinning), cache-friendly
- Risk: high — changes fundamental synchronization model

---

## 7. Upstream Tracking + LLM Re-implementation Pipeline — Size: XL

Broader than just watching ReactOS commits — this is about keeping the entire dependency graph fresh without manual porting labor.

### 7a. ReactOS Upstream Patch Tracking

AI watches `reactos/reactos` commits, understands intent, ports patches to the Rust codebase.

- Monitor commits, parse message + diff for intent
- Map affected C functions to their Rust equivalents
- Generate Rust patch, run `cargo check` + smoke tests
- Human review gate before merge

**Risks:** Semantic drift — upstream commits may rely on subtle invariants the Rust rewrite no longer obeys. Pipeline must track subsystem contracts, not just syntax.

**The triage half of this is now a TOOL, not a reading habit: `tools/upstream_triage.py`**
(2026-09-26). It answers the one question that repeats per commit — *does this touch C that
no arch compiles any more?* — because that is what separates a bug we inherited from an
ordinary merge. It imports `port_audit.compiled_sources()` for the live set and
`vendored.load()` for third-party rather than re-deriving either, so it cannot drift from
the published coverage ledger.

Buckets, and a verdict is a READING ORDER not a work order: **ORACLE** (uncompiled C we
still have on disk → read against our Rust), **HEADER** (never a compile edge, but a struct
member added upstream moves every field after it and our Rust binds those), **MERGE**
(C we still compile), **VENDORED**, **TESTS** (`rostests`/`rosapps` — absent from the build
graph by CONFIGURATION, not because anyone translated them), **OTHER**.

That TESTS bucket is the whole reason to distrust a first number here: without it the tool
called **31 of 65** commits oracle hits, and 18 were driven entirely by
`modules/rostests/winetests/**`, so every Wine-sync commit looked like an inherited bug.
The real figure is **13 of 65**, and of those, three are comment/formatting-only
(`4ab7fb63af7` is literally "Initailize" → "Initialize"). Absence from the build graph is
evidence of translation only for code the build is SUPPOSED to compile.

Current queue (65 unmerged, `merge-base..upstream/master`): 13 ORACLE, 26 HEADER, 8 MERGE,
5 TESTS, 13 OTHER. Ported so far: `091855fc4f9` (i386 NPX ownership flush). The live one
worth doing next is `396019984c9` (`CmpRemoveFromHiveFileList` ordering + a cached
`CMHIVE::HiveRootPath`), which is also the owed `CmUnloadKey` item — it touches
`cmapi.rs`, `cmhvlist.rs`, `cminit.rs` AND changes `CMHIVE` layout in `sdk/lib/cmlib/cmlib.h`,
so it is a header-layout change as well as a logic port. Then the two HIDCLASS fixes, then
`[MSAFD][AFD] AcceptEx` and the HALX86 Legacy PC Arbiter as features, and the four
Wine-10.0 syncs that land on DLLs we have translated (large, and each needs its own read).

### 7b. Wine DLL Re-implementation in Rust — Size: XXL

ReactOS currently ships ported Wine code (user32, kernel32, gdi32, shell32, advapi32, msvcrt, comctl32, ole32, etc.) — hundreds of DLLs, millions of lines. This code is:
- C, ported from Wine's Linux/POSIX environment to NT
- Maintained as a frozen fork with manual backports
- Increasingly stale as Wine evolves rapidly

**The vision:** Delete the ported Wine C code entirely. Use an LLM pipeline (similar to rustconv but targeting user-mode DLLs) to re-implement from Wine's current source directly into Rust, with NT-ification done by the pipeline rather than by hand. The pipeline would:
- Read Wine's current C source for a DLL
- Understand the POSIX→NT translation patterns (Wine already has NT backend paths for many things)
- Generate Rust with proper NT API calls (NtCreateFile not open(), NtAllocateVirtualMemory not mmap())
- Apply NT-specific conventions (UNICODE_STRING, OBJECT_ATTRIBUTES, IRP-based I/O)
- Be quasi-repeatable: when Wine updates a DLL, re-run the pipeline against the new source

This turns Wine from a "port once, maintain forever" burden into a "regenerate on demand" dependency.

### 7c. Linux-Origin Subsystem Refresh — Size: XL

Same pattern for other code ReactOS imported from Linux and is now stuck maintaining:
- **NTFS driver** — ReactOS's NTFS implementation is a port from Linux's ntfs3/ntfs-3g. Rather than maintaining the C port, re-implement from current Linux source via LLM pipeline with IFS (Installable File System) NT-ification
- **ext2/btrfs drivers** — similar story
- **Network stack components** — lwIP or Linux TCP/IP pieces adapted to NT's TDI/WinSock kernel interface
- **USB stack** — portions derived from Linux USB core

The common theme: stop maintaining manual C ports of Linux code. Instead, build pipelines that can regenerate NT-native Rust implementations from current upstream source on demand.

**Size: XXL total** — Each subsystem is its own project. But the pipeline infrastructure is reusable — rustconv's architecture (tree-sitter parse → catalog resolve → LLM translate → postprocess → compile check) generalizes beyond ntoskrnl. The Wine DLL pipeline would be the largest single effort.

---

## 8. Homegrown Rosetta — WoW64 + FEX/TCG x86 Emulation — Size: XL

Run x86/x64 Windows binaries on ARM64 (and eventually RISC-V, LoongArch, etc.).

**Architecture — three layers:**

```
x86 app → wow64.dll (thunking) → X86Translator trait → host kernel
                                       │
                          ┌─────────────┼─────────────┐
                          │             │             │
                     FexTranslator  TcgTranslator   (future)
                     (ARM64 host)   (any host)
```

**Layer 1: WoW64 thunk auto-generation**
- Auto-generate thunk DLLs from ntkrnl-sys bindgen output (all types, all function signatures, all struct layouts per arch already available)
- For each Nt* syscall: classify params (scalar/pointer-to-struct/callback), generate layout converters, handle 32↔64-bit pointer translation
- Callbacks are the hard case but statically knowable from headers — generate trampoline table per function pointer type
- ReactOS already has partial wow64 infrastructure (wow64.dll, wow64win.dll, wow64cpu.dll skeleton)

**Layer 2: X86Translator trait — pluggable JIT backend**
```rust
trait X86Translator {
    fn init_process_context() -> TranslatorContext;
    fn run(ctx: &mut TranslatorContext, entry: GuestVA) -> ExitReason;
    fn handle_exception(ctx: &mut TranslatorContext, exc: &ExceptionRecord);
}

#[cfg(target_arch = "aarch64")]
struct FexTranslator;      // FEXCore JIT — fast, ARM64-only

#[cfg(not(target_arch = "aarch64"))]
struct TcgTranslator;      // QEMU TCG — portable to any host
```

**Layer 3a: FEX-Emu (primary, ARM64 hosts)**
- Custom IR JIT (x86 → FEX IR → optimization → ARM64 codegen), much faster than TCG
- **Already builds as PE DLLs** (`libwow64fex.dll`, `libarm64ecfex.dll`) with no Linux syscall layer
- Wine's WoW64 already uses FEX via a clean plugin interface (`BTCpuProcessInit`, `BTCpuSimulate`, etc.)
- Porting surface to VibeOS: ~500 lines of platform shim (mmap→NtAllocateVirtualMemory, pthread→CriticalSection, signal→SEH)
- Valve-sponsored, used in Steam Frame (Snapdragon 8 Gen 3). Production-trajectory.
- Handles both x86 (32-bit) and x86-64 guest
- Includes TSO memory model emulation (x86 strong ordering on ARM64 relaxed)

**Layer 3b: QEMU TCG (fallback, any host)**
- Vendor only: `tcg/` (core IR), `target/i386/` (x86 frontend), `accel/tcg/` (execution loop)
- Delete: all other `target/` dirs, `hw/`, `system/`, `linux-user/`
- Porting: replace `mmap(PROT_EXEC)` with `NtAllocateVirtualMemory`, replace `cpu_signal_handler` with SEH filter
- Larger porting surface than FEX (~2-4 months) but runs on RISC-V, LoongArch, MIPS hosts
- Not needed on day one — ship FEX on ARM64 first, TCG when exotic hardware exists

**Sequencing:**
1. Implement WoW64 plugin interface in VibeOS (BTCpu* entry points)
2. Vendor FEXCore, write platform shim (~500 lines)
3. Wire thunk auto-generator from ntkrnl-sys bindgen
4. Smoke test with simple x86 binary
5. Later: TCG backend for non-ARM64 hosts

**Size: XL** — Thunk generation is a few weeks (AI-automatable, mechanical). FEX integration is a few weeks more. TCG is months but deferred. Total for MVP: probably 1-2 months.

---

## 9. x86 (i386) Deprecation — Size: L

After WoW64/WoWARM64 thunking is working:
- Stop maintaining native i386 kernel builds
- x86 apps run via WoW64 on amd64 or WoWARM64 on ARM64
- Remove i386-specific assembly, calling convention hacks, 2-level paging code
- Simplify the entire codebase (no more `#[cfg(target_arch = "x86")]` special cases)

**Blocked by:** WoW64/WoWARM64 (item 8) being functional enough to run existing x86 test suites.

**Size: L** — The removal itself is straightforward once thunking works. The risk is discovering implicit i386 dependencies scattered across win32ss, user-mode DLLs, etc.

---

## 10. Real-Hardware Device Path — kill the VM-only shortcuts — Size: L

Today the port leans on **paravirtual, VM-only devices** and dev shortcuts that won't
survive real hardware: virtio-input/blk/net, an arm64 **RAM disk** (the whole OS in
memory), and **polling instead of interrupts** on arm64 (its HAL `HalGetInterruptVector`
is a stub). The unifying insight: **USB is the one bus that carries input, storage (and
more) on real machines across every arch**, and the entire USB+HID+mass-storage stack is
*already translated to Rust* — possibly never exercised. So the work is a shared beachhead
(does the Rust USB host stack enumerate a QEMU device?) that unlocks input **and** storage
together, plus a foundational arm64-interrupts effort that ends the polling class.

| # | Quest | Doc | Size | Status (2026-07-10) |
|---|-------|-----|------|-----------|
| 10a | **USB HID input** — beachhead ✅ **COMPLETE** (USB kbd/mouse enumerate + deliver keystrokes on ALL arches; both walls closed); the **unification migration is owed**: USB HID as the default smoke input transport, retire `arm64input`+`i8042prt`, xHCI gap | [`../engineering/archive/input-architecture-and-unification.md`](../engineering/archive/input-architecture-and-unification.md) | M (migration) | migration in the Check-back ledger |
| 10b | **USB mass storage / real storage** (kill the arm64 RAM disk) — 🔓 **UNBLOCKED**: the shared USB host stack is proven live | [`../engineering/archive/usb-mass-storage-real-storage-quest.md`](../engineering/archive/usb-mass-storage-real-storage-quest.md) | M | ready to start |
| 10c | **arm64 GIC interrupts** — ✅ **LANDED 2026-07-05** (real vectoring, GIC SPI routing, hardware IRQL via GICC_PMR, arm64input → ISR+DPC) | [`../engineering/archive/arm64-interrupts-quest.md`](../engineering/archive/arm64-interrupts-quest.md) | — | done |
| 10d | **Real NIC** (replace virtio-net; interrupt-driven miniport → netstack). First target: the Pi 4's GENET (BCM2711). Next: the Pi 5's Cadence GEM (behind RP1) and the OptiPlex's Intel I217-LM (e1000e class; `e1000` covers only 8254x) | [`../engineering/archive/real-nic-networking-quest.md`](../engineering/archive/real-nic-networking-quest.md) | M–L | **Pi 4 GENET works** (2026-10-05: DHCP, ping, TCP on the LAN; polled). Next: its interrupts, then GEM and I217 |
| 10e | **Verified network boot in freeldr** — freeldr fetches the kernel/ramdisk itself over HTTP through UEFI's HTTP/TCP protocols, and **refuses to boot any payload whose SHA-256 does not match** a signed/pinned manifest (treat it as prod: a netboot that trusts whatever answers on the LAN is a remote-code-execution path). Motivation: on the Pi 4 the 140 MB ramdisk costs ~17 min per kernel iteration, and the network is not the limit (iPXE pulls it in 19 s): it is writing it to and reading it from the SD card through the firmware (`docs/engineering/pi4-bench.md`). Fetching into RAM skips the card entirely. Interim: iPXE `sanboot` of the boot image | — | M | proposed 2026-09-26 |
| 10f | **Maybe: bring legacy BIOS boot back (loader only)** for i386 and amd64. The BIOS prune (`85102c111a3`..`5ca4d482d32`, 2026-05-29) assumed BIOS no longer mattered. That holds for amd64 PCs from about 2012 on, but almost every real 32-bit PC is BIOS-only, so without it i386 boots on no physical hardware; SeaBIOS is also the default in plain QEMU and Proxmox, and on coreboot/Libreboot machines such as the owner's OptiPlex 9020 (a candidate x64 bench: COM1, NVMe boot through SeaBIOS). **Scope: the loader, not the emulator.** The PC-AT machine layer survives in `freeldr-rust` (`machpc`, `pcmem`, `pcdisk`, `pcvesa`, `pchw`, `pxe`); restore the real-mode entry (`arch/realmode/*`) and `pcat.cmake` from git, wire `freeldr.sys` into the cargo-driven loader link (`ab763550984`), and repair four months of drift in the BIOS-only paths. The loader hands the kernel a VESA framebuffer like UEFI's GOP one, so fast486, HAL x86bios and videoprt INT10 stay retired. **Add a SeaBIOS smoke cell for i386 and amd64**, or it rots again. Quick experiment first: OpenCore's OpenDuet (UEFI loaded from disk by a legacy BIOS) runs today's `uefildr.efi` with no OS change. | — | M | considered 2026-10-05 |

Not trash, leave alone: the GOP linear-framebuffer display driver (`arm64vid`, shared all
arches) and the PL011 UART (a real ARM peripheral).

---

## 11. Retire MSVC support — Size: M (CMake + stlport DONE 2026-10-03; tail owed)

**MSVC is not a supported toolchain** (owner, 2026-09-26): the project builds with
clang/llvm-mingw, and that was a settled early call. Upstream ReactOS still carries an
MSVC build, and everything that exists only for it is dead weight here. Owner said "kill
it" on 2026-10-03.

**Done 2026-10-03:**

- **`tools/strip_msvc_cmake.py`** partially evaluates every CMake `if`/`elseif` with the
  MSVC facts fixed (`MSVC`/`MSVC_IDE` false, compiler id never "MSVC") and resolves what
  that decides: dead arms deleted with their attached comments, surviving arms dedented,
  compound conditions rewritten (`MSVC OR X` -> `X`). Its first run resolved 203
  conditions in 144 files. `vibeos-cli check` runs it as `strip_msvc_cmake` with an EMPTY
  baseline, so **after an upstream merge, run `python3 tools/strip_msvc_cmake.py --write`**
  when the check names new sites. Vendored trees are skipped (their CMake is the tree's).
- Deleted: `vendor/stlport` (116K lines; registry entry, patch and notices with it),
  `sdk/cmake/msvc.cmake`, `sdk/cmake/baseaddress_msvc.cmake`, `toolchain-msvc.cmake`,
  `overrides-msvc.cmake`. The MSVC-only rostests went with their `if(MSVC)` arms, and
  `vendor/ms-compiler-tests` is now built only by its clang i386 arm.

**Still owed:**

- **Orphaned MSVC-only sources**, now referenced by no CMake: `cpprt` (`ehvec.cpp`,
  `new_nothrow.cpp`, `terminate.cpp`, `typeinfo.cpp`, `{i386,amd64}/cpprt.s`), the Wine
  `msvc.S`/`msvc-thiscall.c` thunks (mshtml, oleaut32, rpcrt4 + `thunks-msvc.s`, vssapi,
  ole32, windowscodecs, ks), `crt/oldnames-{common,msvcrt}.S`, msvcrt's
  `{i386,amd64}/cpp_alias.s`, `cpp_vtables.s`, `except_*.s` (CRT lane: delete when it
  finishes), and the winetests `get_fpu_cw.asm` / `msvc-x64.asm`. Dead-but-present, like a
  cohab-flipped `.c`. Deleting a file upstream still edits makes a modify/delete conflict on
  every sync, so take them when their module is translated or upstream stops touching them.
- **Upstream's Windows/CI entry points** still name the MSVC toolchain and are left as
  upstream ships them: `.github/workflows/build.yml` (its `build-msvc` jobs),
  `CMakePresets.json` (`base-msvc`/`clang-cl` presets) and `configure.cmd`. We run none of
  them; editing them buys only merge conflicts.
- **`_MSC_VER` in ~589 non-vendored sources**: retire opportunistically in files we own or
  have translated. Many are portability guards that also select the clang path, so read
  each one; do not delete by regex.

---

## 12. Final bosses: Windows 10 drivers, real GPUs, latest Chrome — Size: XXL

The point where vibeos runs **today's Windows binaries unchanged** on real hardware.
Full plan, oracles and safety rules:
[`../engineering/modern-windows-compat-roadmap.md`](../engineering/modern-windows-compat-roadmap.md)
(owner discussion, 2026-10-05). Test bench: the homelab OptiPlex 7010 (NVMe, Libreboot).

| # | Boss | First rung | Size | Status (2026-10-05) |
|---|------|-----------|------|--------|
| 12a | **Windows 10 kernel export surface**, shared by everything below | An import-gap census tool: missing exports across real Windows 10 drivers and apps, ranked | L | proposed |
| 12b | **Modern hardware drivers**: KMDF (built, 1.17) on real drivers, NVMe (missing), NDIS 6, Vista audio stack, Wi-Fi, HID-over-I²C | NVMe StorPort miniport on the 7010 | XL | proposed |
| 12c | **Real GPU**: XP x64 driver (XDDM), then WDDM 1.1, then WDDM 2.x (dxgkrnl + a CDD-style adapter so win32k keeps its interface) | Radeon HD 5450 on AMD's XP x64 driver | XXL | proposed |
| 12d | **Latest Google Chrome**: Windows 10 user-mode surface (mostly Wine), then the sandbox's four kernel primitives (syscall filter, integrity levels, job limits, namespace isolation) plus Windows-exact ntdll stubs | `chrome --headless --no-sandbox --dump-dom` as a smoke ladder rung | XL | proposed |

**Blocked by:** amd64 booting on real hardware at all (the 7010, via firmware graphics),
§10. Chrome rungs 1–3 are not blocked by any of the kernel work.

---

## Check-back ledger — owed items from closed docs/lanes (opened 2026-07-10)

When a plan/quest/handoff doc closes, its *outstanding* items land here (one line + a
pointer) instead of rotting in an archived doc nobody re-reads. **Remove items as they
land; date every addition.** Source docs live in `docs/engineering/archive/`.

### Verify-if-fixed (cheap checks — later work likely already fixed these)
- [x] **Delay-load 0x1E blocker — CLEARED, VERIFIED** (2026-07-11): i386 built with
  `VIBEOS_ENABLE_DELAYLOAD=ON` (557 helper refs = delay-load live) boots to `shell ready` +
  `SwitchDesktop OK` in 33s, **0 `BAD gate` hits** (§9.13/§9.24 fixed the family). **Now-
  actionable follow-on:** flip `VIBEOS_ENABLE_DELAYLOAD` default ON in `sdk/cmake/gcc.cmake`,
  ×3-gate, delete the arm64 `arm64_*_stubs.c` optional-DLL hacks. *(archive/delay-import-llvm-mingw-roadmap)*
  **Flipped on all three arches, 2026-09-24:** `VIBEOS_ENABLE_DELAYLOAD` is now an option
  defaulting ON. The WoW64 `user32:class` found the eager loads, which put comctl32 in every
  RPC-using process. arm64 needed an 8-aligned IAT in `gen-delaylib.py`, then exposed #43
  (below), which it was held OFF for until that was fixed. The four arm64 `arm64_*_stubs.c`
  optional-DLL hacks were deleted 2026-09-25 (advapi32 LSA logon, rpcrt4, user32 libpng,
  consrv ole32/shell32), and every arch now uses the same delay-import lists. rpcrt4's stubs
  had included `VARIANT_UserMarshal` and `BSTR_UserMarshal` returning nothing, which silently
  broke OLE automation marshalling on arm64 alone.
- [x] **amd64 whpx net 13/15 — A/B done, inbound failure is GENUINE** (verified 2026-07-11):
  wedge fixed → VM boots fully under whpx, but both guest listeners never announce (echo +
  TDI `:8888`) — a whpx-specific inbound-listen bug (passes on TCG). Moved from "verify" to a
  real open issue in the whpx spike doc §Remaining; not a quick fix, deferred.
- [x] **arm64 taskbar 0x50 — NOT reproducing** (verified 2026-07-11): arm64 gates green
  through the taskbar path (gate `20260710-233422` OK) with the tray clock rendering (§9.22).
  A green gate = shell-ready reached without the child-control fault. Closed.
- [x] **gdb-engine i386-under-WHPX arch mismatch — FIXED** (verified 2026-07-11):
  `gdbengine.py` sets `set architecture i386:x86-64` for i386+whpx. Whpx spike doc updated.
- [ ] **arm64 reboot stub — CONFIRMED still stubbed** (2026-07-11): `HalReturnToFirmware`
  (`hal-rust/src/arm64/halstubs.rs:551`) just `wfi`-loops, no PSCI `SYSTEM_RESET`. Small owed
  task — matters for harnesses that reboot the guest (conformance crash-recovery). Not yet done.

### Owed work (moved from closed docs)
- [ ] **Input unification migration** *(input-architecture, roadmap §10a)*: USB HID as the
  default smoke input transport ×3 arches; retire `arm64input` + `i8042prt` (or demote to
  explicit fallback). The usbxhci gap is closed: USB HID is the Pi 4's keyboard and mouse
  (2026-09-26, `docs/engineering/usbxhci-design.md`), and arm64input now yields
  `KeyboardClass0`/`PointerClass0` to kbdclass/mouclass wherever it finds no virtio-input.
- [ ] **Warm-build driver-relink infra bug** *(input lane)*: warm `vibeos-cli build` doesn't
  relink driver `.sys` after an `ntkrnl-sys` edit — false-fresh family; cost 3 debug rounds.
- [ ] **KD continuation** *(windbg handoff)*: amd64 same-CPL `int 2Dh` machine-frame
  reconstruction corrupts the resumed context. Low priority; repro recipe in the archived doc.
- [ ] **Ghost-tax Tier-2 residue** *(ghost-tax §3T2)*: panic→serial on arm64 (PL011 silent
  today); cold-cadence policy (needs user decision); verify_abi internal-symbol coverage
  (the #1 ABI blind spot, R6/R7 class); boot-duration trending; STALE_MEDIA auto-heal
  (`smoke --rebuild-if-stale`); net-stage cost fix (`--skip-clean-build` in folded net suite).
- [ ] **Ghost-tax Tier-3 workstreams** *(ghost-tax §3T3)*: toolchain-UB kill program
  (split-debuginfo bootable kernel, arm64 trap-unreachable, xchg-width scanner); XLAT lint
  pack (`lint_xlat.py` from the field-manual §4 greps).
- [ ] **Deferred lint stock** *(ghost-tax landing)*: 136 SILENT_LOOP driver panic-prints;
  lint_kernel 1473 write_volatile stock; 6 RtlFill/MoveMemory macro-decls;
  verify_signatures `#[cfg]` FP; ~150 benign dead-decl deletions.
- [ ] **Tooling-consolidation residue** *(tooling plan T0/T1/T2/LT)*: `.responses/` GC policy
  (paid-artifact-safe); gdb-helpers `list-threads.py` archive-and-rewire; verify_constants +
  scan_inline_flag_literals merge; 4 cache-reassemblers → one `rustconv reassemble`;
  one-arch-source-of-truth (archcfg feeds scanners + run.sh); committed_drift `.res` check
  must run inside the canon mount (currently SKIPs); long-term: unify the FFI cluster on
  verify_signatures.
- [ ] **Driver-local enum const verification** *(input lane, task #14)*: extend
  verify_constants to driver-local enums (enum-drift class).
- [ ] **cdfs FSCTL / classpnp bind-all nests** *(input lane, task #9)*: 185
  unreachable-pattern warnings, orthogonal cleanup.
- [ ] **Reproducible builds — final gate wiring** *(deterministic-builds-plan)*: rungs 1–5 are
  essentially DONE (canonical path, PE-timestamp, SOURCE_DATE_EPOCH, build-id, gauge all ✅);
  only loose ends remain — rungs 2a/2b are *gate-pending* (implemented, not yet enforced), an
  optional cross-worktree `diff-dirs`/`diffoscope` proof, and the C `trim-paths` add. Hardlink-
  dedup is BANNED (fleet-isolation law). Gauge: `vibeos-cli repro`.
- [ ] **Payload-harness follow-ons** *(payload-harness-design)*: Workstream A (delete the
  winlogon spawn hack → fixed `ProcessStartupItems`); Workstream B (vibeagent guest-exec
  channel — highest-value); PuTTY payload (pure-YAML generalization exam).
- [ ] **arm64 harness waits sweep**: boot_timeout right-sized (540→180); net-suite/snapshot/
  per-milestone waits likely still oversized.
- [ ] **cswitchdep follow-ons** *(cswitchdep lane closed 2026-07-11, `cswitchdep-design.md`)*:
  (a) per-check subset arming `CSWITCH=irql,parity` — deliberately not built (would add a
  runtime-mask branch to every hot-path tap for a single-digit-% overhead the cost model
  already accepts); cheap to add if a measured soak overhead ever justifies it. (b) The
  overnight cswitch/lockdep farm soak — run `vibeos-cli farm ... ` when the host is quiet and
  read the new `farm-report` "tripwire hits" section (clusters green-boot violations).
- [x] **Stable-everything toolchain program** — **DONE 2026-07-12** (lane `stable-toolchain`;
  writeup `../engineering/stable-toolchain-plan.md`). All 4 nightly deps killed:
  (1) `link_llvm_intrinsics` → `code_addr!()`. (2) `-Z build-std` + the custom JSON targets →
  amd64/arm64 now build the **builtin triples on pinned stable**; 80 target JSONs deleted;
  `abi-return-struct-as-int` audit discharged (proven inert off 32-bit x86); ntoskrnl `.text`
  measured **+0.4% amd64 / −2.9% arm64** (the feared ~2.4× never materialised). (3) `c_variadic`
  → C `va_start` shims + a `varargs-rust` wrapper; zero of the 184 call sites changed.
  (4) `linkage` → the `hal-cargo` use was vestigial, just deleted. Toolchain now pinned
  (`rust-toolchain.toml` + `.rust-nightly`). **i386 kernel is the sole remaining nightly cell**,
  hard-blocked upstream on `rustc-abi=x86-softfloat` — it RETIRES with the wow64 lane, not fixed.
- [ ] **30 pre-existing infinite self-jump functions** *(found by the new
  `tools/scan_self_recursive.py`; baselined in `tools/lint-baseline/scan_self_recursive.txt`,
  gate fails on NEW ones)*: functions whose first instruction jumps to their own entry — the CPU
  spins forever with **no bugcheck and no serial output**. 23× a C `exp2()` that LLVM folded into
  a call to itself (same idiom-recursion trap as field manual §9.65, in C — needs
  `-fno-builtin-exp2` on those TUs); classpnp's local `KeRaiseIrql`/`KeLowerIrql` stubs (hang if
  ever called); `ftfd.dll`'s `rust_begin_unwind` written as `loop {}` (a panic that hangs
  silently — should adopt `panic-common`); `cdrom.sys: abort`. Tracked and owed, not accepted.
- [ ] **KeGetPcr x18 cohab dedup** *(cswitchdep arm64 first-light, task #2)*: the x18
  `ddk::KeGetPcr` copies are unsafe in switch context — dedup to the single TPIDR-backed path.

## Known bugs & flakes (live registry)

- **Pi 4 (real hardware): a loopback UDP `sendto()` never returned** (FIXED 2026-10-04,
  `e0674df2c66`). With no NIC netstack drives, the engine worker exited while TDI kept pending
  IRPs into its dead inbox. They now fail fast with STATUS_NETWORK_UNREACHABLE. QEMU never showed
  it because every smoke boot has QEMU's default NIC. **Still owed:** loopback with no NIC fails
  rather than WORKS (Windows always has a loopback interface). That wants a smoltcp loopback
  device in the engine, worth doing alongside the Pi's GENET driver.
- **amd64 RAM above 4 GiB: the kernel cannot use it yet** (OPEN, 2026-10-04). The loader's
  `MM_MAX_PAGE` (freeldr-rust `frldr.rs`) cuts amd64's memory map at 4 GiB, as the Rust has
  always done; mm.h's 8 GiB would expose the rest. With it raised, `VIBEOS_QEMU_HOSTILE=mem`
  (6 GiB) bugchecks 0x19 (pool header, P1=5) and hostile `all` hangs in uniata's AHCI
  start. Repro: runs `20261004-194815-lrtpx0w2` and `20261004-194452-8j2swl96` (smoke-amd64,
  graviton lane). Raise the cap once both are green. arm64 already describes all RAM: its
  pool-sizing underflow was the 4 GiB Pi 5's silent hang (fixed in `arm/init.rs`).
- **amd64: a user-mode Rust panic report can bugcheck the kernel (0x1E in the debug print
  path)** (OPEN, 2026-10-04; kd lane). A panic in explorer's shell32 printed a stack walk that
  ran into garbage, and the kernel faulted (C0000005) at `ntoskrnl+0xC64C0` reading it, which
  symbolizes to the first line of kd-rust `KdIoPrintf` (`kdio.rs:822`), so the exact faulting
  function is not confirmed. User mode must not be able to take the kernel down through a
  debug print: the copy from the caller's buffer needs a probe under SEH. Repro: runs
  `20261004-080222-cpyv4l2x` / `20261004-080920-w43f482z` (smoke-amd64, msvcrt-homebrew,
  before the DllEntryPoint fix that removed the panic).
- **fastfat: the free-dirent bitmap goes stale, then every create in that directory raises
  STATUS_FILE_CORRUPT_ERROR** (OPEN, 2026-10-04). Wine's ole32 `storage32` test (olewinetest
  payload, amd64) drives about 100,000 raises in 120 s. They are caught, so the test times out
  rather than crashing, but the files are never created.
  - Raiser: `FatCreateNewDirent`'s "assert that the dirents are in fact unused" check
    (dirsup.rs:396). `FreeDirentBitmap` calls a slot free that is in use on disk.
  - `FatCreateNewDirent` and `FatDeleteDirent` both read faithful to dirsup.c. The suspect is
    another path that writes a dirent without setting its bit: rename, tunnelling, or the
    rescan.
  - Found with `RtlRaiseStatus`'s new backtrace (image base + RVA per frame). For an amd64
    kernel frame, symbolize against `ntkrnlmp.exe`.
- **rostests i386 build: 5 targets fail under llvm-mingw** (OPEN, 2026-10-04). After the
  shell32/shlwapi/ole32 export fixes (`--rostests`, `ninja -k 0`): `dllimport_test.dll`
  (lld: framedyn `CHString` "should not refer to special section 0"), `menuaccels.rc` (llvm-rc
  rejects its ACCELERATORS statement), `interop_unittest` LocaleTests.cpp (libc++ `<__locale>`
  `enum result { ok, ... }` collides with the test `ok` macro), `wine.combase.test.dll`
  (`-llibcombase` not found), `user32_winetest` monitor.c (`ddk/d3dkmthk.h` not on its include
  path; it lives in sdk/include/wine/ddk).
- **i386 smoke QEMU mis-executes SAHF after a logic op** (OPEN, 2026-10-04; host tooling,
  not vibeos). The windows-host backend's QEMU is 11.0.50, a development snapshot
  (`v11.0.0-12631-g54e84cdc7a`, installed 2026-05-01). After `or`/`xor`/`test`/`mov`, `sahf`
  sets CF wrong, so `fucom; fnstsw; sahf; jcc` -- every x87 float compare `-march=pentium`
  emits -- can branch the wrong way in i386 guests. Seen as Wine msvcrt `string.c:3522`'s 5,535
  failures (`_atodbl` is right in all of them). `tools/payloads/atodbl` checks it and goes
  green on a fixed emulator. Owed: a released QEMU for the windows-host backend, then rerun
  i386 float-heavy tests. Field manual: "i386 guest float compares go the wrong way".
- **msvcrt.dll Wine winetest `misc`: FIXED 2026-10-04, was "exits 128"** (i386). The
  subtest died after misc.c:375's `_popen` failed with EACCES and the test ran `fgets` on the
  NULL stream. Root cause in kernel32-rust, not msvcrt: `GetEnvironmentVariableA/W` kept the
  pre-242937b1229 sizing (reserve one char for the NUL) while rtl-strings' query follows
  upstream ef70bdf9907 (needs room for the NUL), so every size-then-fetch caller got an empty
  string; msvcrt's COMSPEC came back `""` and `_popen` tried to execute `X:\reactos\bin\`.
  Now 832 tests, 11 failures: all `I10_OUTPUT` rows whose expected trailing bytes are native's
  rounding residue (`...9`), which Wine marks `todo_wine_if`. `tools/payloads/popen` checks
  `_popen`. msvcrt_apitest `ieee` is 33 failures of 286 (was 35), mostly `_j0`/`_j1` value
  rows that want native's bits.
- **fastfat bugchecks on rename and on a run of creates (arm64)** (FIXED 2026-10-03; was
  "bugchecks in `FsRtlCheckOplock`"). Two bugs, both 0x23 via `FatBugCheckExceptionFilter`:
  - rename: `FatSetRenameInfo` read the shim's 4-byte `MAX_LFN_CHARACTERS` as `usize`, so every
    name buffer carved from `UnicodeBuffer` had a garbage high half and the first
    `RtlCopyUnicodeString` faulted. Not arm64-only: amd64 bugchecks the same way once the
    rename is on a writable FAT volume (its `%TEMP%` is the read-only CD; run
    `20261003-125533-far1ln8_`, `tools/payloads/fsprobe`). Gated by `scan_extern_static_width`.
  - creates/deletes: the translation ran fastfat's `NTDDI >= WIN8` blocks on a pre-WIN8 layout,
    so parent-directory oplock checks read the DCB's free-dirent bitmap as an oplock pointer
    (and DCB init/teardown wrote into it). Those blocks sit behind `cfg(ntddi_ge_ntddi_win8)`
    now; the field manual's "version block translated UNGATED" entry has the audit.
  - a third, found by `tools/payloads/fsprobe` on amd64's scratch FAT16 disk: `UCHAR
    FatIndexBitSize()` declared `-> u32`; the undefined register bits became the divisor in
    `ClustersDescribableByFat`, the 64 MB volume mounted with 14 clusters, and the 29th file in
    a directory failed `ERROR_DISK_FULL`. Gated by `scan_extern_decl_drift`'s return widths.
  `tools/payloads/fatprobe` (rename, 64 create/delete, rename with a second handle open)
  passes on arm64; `fsprobe` (renames of each kind, 52 live long names) on arm64 and amd64.
  `UCRTDIFF_FASTFAT_OPLOCK_BUGCHECKS` is now 0: rename/_wrename/_mktemp/_wmktemp/_mktemp_s/
  _wmktemp_s all grade MATCH. The plain arm64 ucrtdiff build stops at pecheck
  (`__doserrno`/`_open_osfhandle` graded but not imported); grade through
  `ucrt_tier2_sweep.py --takeover-from d888eadd919^`, narrowed with `UCRTDIFF_ONLY=`.
- **fastfat's `NTDDI >= WIN7` blocks are still translated ungated — OPEN (audit owed)**
  (2026-10-03). The WIN8 audit above does not cover them. Ten are oplock calls on FILE FCBs
  (valid storage, so no corruption; a behaviour divergence: key checks, `FsRtlOplockBreakH`,
  atomic create-with-oplock in `FatOpenExistingFcb`/`FatOpenExistingFile`/`FatCreateNewFile`/
  `FatSetRenameInfo`); fsctrl.c has 12 more. Owner: fastfat lane.
- **ucrtbase's ~70 C99 math stubs** (FIXED, 2026-10-03): `acosh` ... `trunc`, their `f`
  forms, `_fpclassf`, `_dpcomp`/`_fdpcomp`, `_Cbuild`/`creal` are `ucrt-rust/src/c99math.rs`
  -- the algorithms from the `libm` crate (musl's, MIT; the code Wine itself runs), the UCRT
  errno contract ours. Graded by `tools/payloads/c99math` (value vs glibc within a stated ulp
  budget, errno vs Wine's tables) on all three arches. Two things it took: i386 ucrtbase moved
  to the SSE2 baseline (libm cannot compile under `-sse`), which in turn exposed
  `sdk/lib/crt`'s x87-only i386 `_control87`/`_statusfp`/`_clearfp` -- replaced by Wine's
  both-unit versions in `fenv.rs`; and `rint`/`nearbyint` on arm64 are `frintx`/`frinti`,
  because `libm` ignores rounding modes. `pow`/`exp`/`log`/`exp2`/`log2` (and f32 forms) come
  from `sdk/lib/rust/musl-math` instead of `libm` -- a port of musl's current algorithms,
  bit-exact against the musl C -- because libm's are FreeBSD's older ones (see the field
  manual). i386 msvcrt.dll moved to SSE2 too (2026-10-03), and cnt-rust's exp/log/pow/log10/
  fmod now carry Wine's errno/`_matherr` contract in both CRTs (`__cnt_math_error`). Still
  owed: the rest of cnt-rust's C89 set (trig, sqrt, ldexp, _logb, ...) onto libm with the
  same contract, and the ieee754 Bessel C onto libm. Still `-stub`: the `_d*`/`_fd*` internal helpers
  (`_dexp`, `_dnorm`, `_dpoly`, ...) and `_except1`. Owner: ucrt-rust.
- **spec2def's generated `*_stubs.c` are the last C in the CRT DLLs** (DEFERRED, 2026-10-04).
  msvcrt.dll and ucrtbase.dll link no hand-written C or assembly on any arch; each one's only C
  object is its spec2def-generated stubs TU (msvcrt 24 stub bodies, ucrtbase ~120, msvcrt20/40
  ~440 each, crtdll 9). Retiring them means spec2def emitting Rust (or asm) stubs, which changes
  every DLL in the tree, not just the CRT -- a tooling project of its own, parked here by choice.
- **Delay-loaded imports are not delay-loaded** (OPEN, 2026-10-04; all arches). `VIBEOS_ENABLE_DELAYLOAD`
  is ON, yet no module carries a delay-import directory (`DelayImportDescriptorRVA` 0 in shell32,
  comdlg32, setup's reactos.exe on i386 and amd64): every `add_delay_importlibs` DLL is bound as
  a regular import, and the delay-load runtime (`__delayLoadHelper2`, Rust since 2026-10-04 in
  `sdk/lib/rust/delayimp-rust`) is linked everywhere and called nowhere. The sdk apitests say so
  themselves -- `tools/payloads/delayimp` (i386): all three skip with "No
  IMAGE_DIRECTORY_ENTRY_DELAY_IMPORT found", nohook exits 0 and globalhook/runtimehook exit 128,
  identically against the C and the Rust runtime (A/B). Owed: find where the `_delayed`
  import libraries stop producing delay thunks (gcc.cmake `add_delay_importlibs`,
  docs/engineering/archive/delay-import-llvm-mingw-design.md), then grade the helper for real.
- **Wine's ucrtbase conformance suite: the failures the all-Rust ucrtbase SHARES with
  Microsoft's C** (OPEN, 2026-10-03; i386). `tools/payloads/ucrtwine` runs
  `ucrtbase_winetest` + `ucrtbase_apitest` in the guest; `tools/payloads/ucrtwineab` runs
  the same binaries against the pre-takeover ucrtbase.dll (`24089de2c88`, Microsoft's C)
  staged beside them. The two agree failure for failure, so none of these is a Rust
  regression, but each is owed: `file` 10 failures of 120 (identical on both; was an AV
  while the sweep ran it from the read-only CDFS); `misc` dies on an unmasked FP exception
  (0xC0000090) or times out after calling the `fesetround`/`fegetenv` spec stubs, and its
  math-errno checks (misc.c:774/776) fail on ours past the point where the baseline has
  already crashed, so they are UNCOMPARED. The HANG is ROOT-CAUSED and FIXED (2026-10-03):
  `log10(INFINITY)` never returned -- cnt-rust's `log10`/`pow`/`fmod` were ports of upstream's
  placeholder C (`pow(2, 0.5) == 0` shipped in msvcrt.dll and ucrtbase.dll on every arch); they
  are the `libm` crate's now, and the x87 asm on no-SSE i386 (`tools/payloads/matherrs` traces
  Wine's test per call). The math-errno checks are FIXED (2026-10-04): cnt-rust's C89 set
  reports through `__cnt_math_error` at Wine's `math_error` points, and `tools/payloads/matherrs`
  (Wine's `test_math_errors`) is 304 checks, 0 failed on all three arches; misc.c:363 (`_EM_AMBIGUOUS`) is FIXED by the
  i386 `_control87` in fenv.rs; `cpp` hangs past 120s; `string` fails two `mbstowcs` UTF-8 checks
  (string.c:831/835) -- ROOT-CAUSED, expected: kernel32 exports `IsValidLocaleName` only at
  `-version=0x600+` and this build is NT 5.2, so ucrtbase takes its downlevel locale path,
  where ".UTF-8" is not a code page and `en_US.UTF-8` resolves by LCID to ANSI 1252 (UCRT
  on Server 2003 does the same; kernel32 itself converts CP_UTF8 correctly); `ucrtbase_apitest` dies on an invalid handle (0xC0000008). `misc` now runs to the end (198,593 checks, 2026-10-04) with 3 failures, identical with and without that day's vcstartup/TLS move (A/B on one tree): misc.c:1126/1127 (a `_mbstowcs`-class codepage conversion, `ret = 4`, `"a?"` for `"\x96\xd8"`) and misc.c:1355 (`clock()` ~3 s below its window measured from the process creation time, both runs; TCG start-up latency or the clock base, not yet split).
  `environ`, `printf` (1,950), `scanf` and `thread` (60) are clean on both. `thread` was
  an AV at EIP 0 until two fixes the same day: kernel32's `FindResourceA` asked for
  language 4 (LANG_CHINESE) instead of LANG_NEUTRAL, so EVERY ANSI resource lookup in the
  OS failed with 1815, and %TEMP% sat on the read-only boot image. Only ucrtbase
  divergence found: the i386 `_invoke_watson` report-fault stub wrote CONTEXT out of
  order (fixed the same day).
  Owner: ucrt-rust lane.
- **arm64 ucrtbase forwards `_clearfp`/`_control87`/`_controlfp`/`_statusfp` to msvcrt.dll**
  (FIXED 2026-10-03: ucrt-rust `fpctrl_arm64.rs`, graded by `tools/payloads/fpctrl` against
  msvcrt.dll's bodies -- 671 cases, 0 diffs; the allowlist entry is gone. Was OPEN, 2026-09-28). No arm64 body exists in this tree (`sdk/lib/crt` builds the
  control-word family for i386/amd64 only), so the link resolves all four through the msvcrt
  import library and ucrtbase ships a forward into msvcrt.dll. Found by
  `verify_ucrtbase_cohab.py`'s export/import check, which allowlists these four as OWED
  (`EXPORT_IMPORT_ALLOWED`); remove them there when an arm64 FPCR-backed port lands.
  Owner: ucrt-rust lane.
- **arm64 boots the livecd file set and runs first-boot setup** (2026-10-03). arm64's
  media was ~250 hand-staged files against the livecd's ~1100; `qemu/mkramdisk.sh` now lays out
  the same `livecd.<config>.lst` x86's mkisofs reads (tools/ramdisk_tree.py) and arm64 builds
  the same `livecd` target, so `setup -mini` runs on every boot as on x86 (COM registrations,
  Tcpip/Dhcp/Dnscache, device installs). In the same pass arm64 stopped skipping: winlogon's
  RPC server, notifications, GINA and lsass wait; the bulk Start=1 driver load (was a fixed
  three-driver list); `/MININT` (and the kernel now creates `Control\MiniNT`, dropped by the
  port on every arch); the per-arch window-station/desktop security descriptors; bugchecks on
  IoInitSystem/PsInitSystem/SeRmInitPhase1 failure. Bugs the full media exposed, fixed:
  setupapi did not know PROCESSOR_ARCHITECTURE_ARM64; netkvm/e1000 miniports bound under
  netstack (see smoltcp-netstack-design.md M11); fastfat enumeration lost entries
  (`NextVbo` reset per iteration; Marlett never loaded); ks pin lists, fltmgr context
  terminator, pcix power/IPI spins. appstorm arm64: 40/43, cpl-appwiz and cpl-ncpa now pass.
  OPEN:
  - **rapps on arm64**: database download FIXED 2026-10-04 (netprobe payload: DNS, TCP and
    wininet https to rapps.reactos.org, 2.4 MB rappmgr2.cab, VERDICT=PASS). Four layers were
    broken: netstack never published its DHCP DNS servers to `Tcpip\Parameters\Interfaces`
    (iphlpapi found none), smoltcp's single interface-wide ARP silence starved later lookups
    (netstack now warms the router and on-link DNS servers), AFD select wrote poll status
    into the IRP's copy rather than the caller's buffer, ws2_32 dropped `LUP_RETURN_BLOB` so
    gethostbyname got no hostent; and bcrypt's ECDSA was a stub, so Let's Encrypt's chain never
    verified (upstream bcrypt restored over mbedTLS 3.6). SO_RCVTIMEO/SO_SNDTIMEO FIXED 2026-10-04: msafd
    waited forever (upstream's own FIXME; a 3s timeout blocked 9 minutes), and now cancels the
    request and fails with WSAETIMEDOUT (netprobe `rcvtimeo`: 2010ms on all arches). Still
    OPEN: the OK-only error box ignoring WM_CLOSE; smoltcp's ARP silence is still global
    (a per-address limit is a vendor patch, deferred).
  - **arm64 appstorm, 2026-10-04 (after the network fixes; 5 soak boots).** rapps now really
    downloads, which exposed and fixed a netstack bug (TDI IRPs could not be cancelled: BUGCHECK
    0x1E when a socket closed mid-connect, see smoltcp-netstack-design.md). What is left:
    - **rapps "close timeout" while "Updating Database"**: FIXED 2026-10-04, two bugs. (1)
      appstorm judged the startup download dialog; closing it makes rapps start the download
      again from its main window (upstream gui.cpp CheckAvailable), so it never closed in 10s.
      appstorm now waits past that dialog (`wait_past`, ~45s on arm64 TCG) and judges the main
      window. (2) The extraction that then runs crashed: cabinet-rust's `FDINOTIFICATION` lacked
      fdi.h's `pshpack4`, so on 64-bit rapps' fnNotify read `pv` as NULL/garbage. Packed and
      offset-asserted; arm64 rapps now opens "ReactOS Applications Manager" and closes OK.
      Two pre-fix boots hung on the old close path, unexplained: 20261004-161901-c5mgnl_5 went
      silent right after "shot rapps" (likely the garbage-`pv` extraction, unproven), and
      20261004-163220-whatl493 stopped inside appstorm's close-timeout thread dump, after the
      UI thread and before the download thread (blocked in an ntdll syscall), with no
      FAULTLOOP report. Inference, not shown: GetThreadContext waiting on a thread whose
      kernel wait blocks the suspend APC.
    - **cpl-appwiz then opens no window** (OPEN): rapps /appwiz stays alive with no window.
      Failed again 2026-10-04 (20261004-173345-lbmi6pz6) after passing 5 boots; not mutex
      abandonment. appstorm's no-window dump missed it (rundll32 exits and orphans rapps before
      the family is grown); fixed, so the next failure dumps rapps' threads.
    - **OWED: hand-declared user-mode FFI structs have no layout gate.** `verify_offsets.py`
      covers ntkrnl-sys only; a struct a crate declares by hand (cabinet's FDI types, the
      `IMAGE_*`/`DROPFILES`/`SHFILEINFO` copies in shell32/version/crt-startup) is checked by
      nothing unless someone wrote an `offset_of!` assert. Wanted: the verify_offsets recipe
      (clang offsets with the module's flags -> generated rustc asserts) for user-mode crates.
    - cpl-joy passed every run today.
  - **arm64 `-smp 2`: BUGCHECK 0x1E in `CmpCreateKeyControlBlock`** (FIXED 2026-10-05). Two
    threads setting values under different keys of one hive hold the registry lock only
    shared, so `HvAllocateCell` ran on both CPUs at once: both edited the free lists and both
    could `HvpAddBin`, which reallocates `BlockList`, frees the old one and publishes its own.
    The 0x1E read a garbage `CM_KEY_NODE` (0x0014000000000026; not the KCB hash chain, as first
    written here) while the other CPU looped in `HvFreeCell` on a NULL `BinAddress`. Windows
    serializes this under `CMHIVE.HiveLock`, which ReactOS declares and never takes; cmlib-rust
    now takes it (`hivelock.rs`). Proven with the new `regstress` payload (4 writers resizing
    values and subkeys in one hive, 2 readers verifying self-describing contents): with the lock
    compiled out, 2/2 `-smp 2` boots trip `ASSERT (*Free).Size < 0` and bugcheck 0x50 in
    `HvpAddBin`'s copy of a freed `BlockList`; with it, 0 failures (50,764 writes at `-smp 1`,
    19,733 at `-smp 2`). Artifacts: `~/vibeos-flake-farm/fails/regstress-smp2-20261005/`.
  - **arm64 `-smp 2`: push-lock release livelock** (FIXED 2026-10-05). The port of upstream
    f42ee0e23bb adopted the failed CMPXCHG's value only in the not-Waking arm of
    `ExfReleasePushLock[Shared]`, so a release that found another thread already waking spun
    forever on a stale value. It was the unexplained HANG_TIMEOUT 20261005-002201-1b2o74m7
    (CPU1 in `ExfReleasePushLock+308`) and hung every regstress `-smp 2` boot until fixed.
  - **arm64 `-smp 2` under regstress: timed waits stop expiring** (OPEN, SMP lane, 2/2 boots
    with both fixes above). After ~15 s of clean stress, regstress's own `Sleep` heartbeat and
    the CENSUS thread both go silent; the hang capture shows both vCPUs FROZEN in `KiIdleLoop`'s
    `wfi`, i.e. no tick wakes either CPU. Not the registry (0 failures up to the stall). Next
    step: read each CPU's GICC_PMR / CNTV_CTL_EL0 from a wedged guest over the gdbstub. One
    debug-VM boot instead died BUGCHECK 0x19 (BAD_POOL_HEADER, P1=3, `ExFreePoolWithTag`) during
    regstress thread startup, no caller stack. Repro: `VIBEOS_PAYLOAD=regstress ./vibeos-cli build
    --arch arm64 && ./vibeos-cli smoke --arch arm64 --payload regstress --smp 2`.
  - **arm64 FAULTLOOP was the detector, not a loop** (RESOLVED 2026-10-04). The report
    fingerprinted a fault as (VA, PC, TTBR0); a stream of DISTINCT faults sharing those --
    Cc reading many small files (rapps' cab extraction), each new VACB recycled onto the same
    view VA and taking one first-touch soft fault in memcpy, or the heap re-committing one
    page -- counted as "the pager succeeded 64 times on the same fault". Measured before the
    fix: every round entered with AT(VA) faulting and left it valid (a fresh view, not a
    stale TLB or a lying walker), the reader's VACB was live and correct each time, the PFN
    differed per round, and MM freed/recreated the area between rounds with no reader
    registered. The fingerprint now includes a hash of the faulting registers: a real loop
    re-faults with identical state, churn does not. Two appstorm boots after: zero reports.
    The probes stay: AT on entry/report, faulting args, frames named module+offset, and an
    unmap tripwire armed only while a loop is live. Whether the DFSC 0x06 Thread32Next case
    was a real hang is not settled -- the two silent appstorm hangs today printed no report.
  - **The dropped RpcTryExcept class**: FIXED 2026-10-04 for the user-mode RPC clients.
    ntdll now exports the kernel's SEH trampolines (`RtlSehTryExcept`, sehtry.c built
    `-DSEHTRY_USER_MODE`) and `ntdll_sys::seh::rpc_try` restores each C handler. 245 stub
    calls go through it: dnsapi's 6 by hand, the rest in advapi32 (scm, sctrl, eventlog,
    lsa), netapi32, samlib and wlanapi rewritten by `tools/rpc_try_wrap.py` from the C's own
    `RpcExcept` bodies. netprobe's
    rpc-raise step: OpenSCManagerW/OpenEventLogW on a missing host return 1722. Other
    user-mode crates with DUMMY_PSEH-dropped handlers (non-RPC) are not yet audited.
  - **Safe boot is missing from the Rust kernel on every arch**: ex-rust neither parses
    `SAFEBOOT:` nor creates `CONTROL\SAFEBOOT\Option` (init.c Phase1InitializationDiscard).
  - **pcix-rust predates upstream's arbiters** (#9502, 023d8af5566): five arbiter bodies are
    still `UNIMPLEMENTED; loop {}`, baselined in scan_ud2's KNOWN_DRIVER_UD2. arm64 binds
    pci.sys, so they are unreached there.
  - Remaining arm64 divergences: winlogon still creates `\Windows\WindowStations` and csrsrv
    skips CsrConnectToUser on arm64; the csrss stack-commit patch in mkramdisk's overlay;
    shell32 CDefView skips SHChangeNotifyRegister on arm64; winlogon's arm64-only notepad and
    cmd spawns. (i386 `cpl-appwiz` opens "Add/Remove Programs" again: appstorm 20261005-184453.)
- **Posted messages to other threads' windows were dropped** (FIXED 2026-09-27). The
  win32k-sys regeneration (c53b7807aad) turned THRDESKHEAD's `[u64; 3]` padding into
  `[u8; 24]`; user32 read `head.pti` as `__bindgen_padding_0[2]`, so IntCallMessageProc saw a
  thread mismatch on every window and dropped every posted message it dispatched. No app
  closed on WM_CLOSE on any arch; every gate stayed green because appstorm scored a close
  timeout as PASS (it fails now). `scan_bindgen_padding_index` gates the class.
- **Hand-written constants vs the bindgen islands** (DONE 2026-09-27). All 89
  `scan_const_vs_island` entries triaged: 28 real bugs fixed (among them winsrv's
  IsConsoleMode calling NtUserCreateMenu, amd64 MAX_SYSCALL_PARAM_SIZE, smss's RESERVE_1MB
  setting NX, fastfat/cdfs MDL flags, TOKEN_HAS_ADMIN_GROUP), the legitimate ones (freeldr's
  ARC codes, driver-local ERROR levels) carry a `nolint` with their reason. The baseline is
  empty: any new disagreement fails `vibeos-cli check`. Still owed from the sweep: fastfat's
  `KeExpandKernelStackAndCalloutEx` extern takes `StackSize: u32` where the C has SIZE_T
  (harmless while the shim ignores it).
- **Hardware faults in a Rust try body: all three arches catch a breakpoint; arm64's
  access violation too** (2026-09-27). The per-boot
  `[SEH-TRY] rust hw faults` self-test (ex-rust init.rs: an int3/brk, an access violation
  and a pool-tracker breakpoint, each under try_finally inside try_except) runs everywhere.
  **i386 FIXED:** a kernel-mode int3 is dispatched with interrupts still disabled and the
  hand-built catch in ntoskrnl/sehtry.c resumed by `__builtin_longjmp`, which restores no
  flags, so the next page fault bugchecked 0x0A with P2 = -1; `RtlSehTryExcept` now records
  IF at try entry and restores it on the catch. **arm64 brk FIXED:** every kernel brk was
  skipped in the trap handler, because DbgBreakPoint and DebugService shared `brk #0xF000`
  and only x0 told them apart. DebugService now uses `#0xF002` (Windows ARM64's service
  immediate), a kernel `#0xF000` is dispatched as STATUS_BREAKPOINT, and an unhandled one
  steps past the brk and continues (the amd64 policy). User-mode brk is still skipped, and
  `#0xF001` (assertion) is unchanged. **arm64 AV FIXED (2026-09-27):** VA 0x10 read physical page 0 through FreeLDR's
  never-torn-down identity map. `MmFreeLoaderBlock` now unmaps VA 0-64K
  (`MiUnmapLowGuardArm64`; serial `[MM] arm64: low guard ... MmIsAddressValid(0x10)=0`,
  QEMU and Pi 4). The Pi4 spin table was not a reason to keep it: APs start by PSCI CPU_ON
  with their own identity map, and hal-rust never touches PA 0xd8-0xf0.
- **amd64: an exception caught in fastfat looped until the stack overflowed (0x7F/8) —
  FIXED** (2026-09-27). The trigger was `FatPreallocateCloseContext`'s pool tag 0 (the
  translation's invented constant; C has `TAG_FAT_CLOSE_CONTEXT`), which equals
  `PoolHitTag`'s default and hit the pool tracker's `DbgBreakPoint`. That tag was fixed on
  2026-09-26, along with a sweep of every kernel crate for a literal-0 tag. The unwinder was
  innocent. `KiBreakpointTrap` dispatched a kernel int3 with IF=0, and the except block that
  caught it resumed with interrupts still off. FatMountVolume's retry then sent
  `IOCTL_DISK_GET_PARTITION_INFO_EX`: it completed synchronously, but its
  `IopCompleteRequest` APC (a self-IPI) could not fire, so `FatPerformDevIoCtrl` returned
  without waiting. The APC ran later and set a KEVENT in a dead frame, overwriting the
  retry's saved RBP. The next dispatch walked into that value and looped.
  `KiBreakpointTrap` and `KiGeneralProtectionFault` now dispatch with the trapped frame's IF
  (as `KiPageFault` already did). The new check found the same bug on arm64 (`irqs=1/0`),
  where the sync-exception path keeps DAIF masked on purpose. There `RtlSehTryExcept`
  restores PSTATE.I on the catch, as the i386 catch already restored IF. Native C
  `__try`/`__except` on arm64 still resumes masked; the trap-level fix is owed to the arm64
  lanes. With the tag-0 repro restored, the boot survives and the
  retried mount succeeds. The per-boot `[SEH-TRY] rust hw faults` test now checks IRQs
  after each catch (`irqs=1/1`), and smoke fails any self-test `(FAIL)` line
  (`SELFTEST_FAIL`); until then nothing read those verdicts. Found with `gdb probe`, whose
  hardware watchpoint armed at function entry named `KeSetEvent` as the writer.
- **i386 with a hard disk attached BUGCHECKs 0x1E after the volume mounts — FIXED**
  (2026-09-26). The volume was not the cause; loading fastfat was. When fs_rec saw the FAT
  volume it demand-loaded fastfat, and on i386 its DriverEntry returned into its caller's
  stack. `rust_cohab_forceinline.c` bridged the bare cdecl names of eight wdm FORCEINLINE
  helpers (`KeInitializeSpinLock`, `ExInitializeFastMutex`, the `Io*IrpStackLocation`
  family, `IoSetCompletionRoutine`) to their stdcall bodies with `.set _X, _X@N` ALIASES,
  so every cdecl call popped its arguments twice. DriverEntry made two (+8 bytes): its
  ESP-relative epilogue popped the return address into EBX and DriverObject into EBP, and
  `ret 8` jumped to RegistryPath on `IopInitializeDriverModule`'s stack. Every register in
  the trap frame matches that arithmetic. Fixed with real cdecl->stdcall thunks;
  `scan_cdecl_alias` (in `check`, baseline empty) forbids the alias shape, and the field
  manual's convention-mismatch list has it as manifestation 7. Verified: `smoke --arch i386
  --repeat 3` with the disk, 3/3, `C:\TEMP` found each boot. The disk is now attached by
  default on i386 as well. Found with a gdb probe once two tool gaps were closed: `gdb
  start` never attached the disk (now shares `vm.temp_disk_args`), and triage resolved
  i386 MP faults against the UP kernel (the loader names ntkrnlmp `ntoskrnl.exe`).
- **fastfat never marked a new directory entry as used — FIXED** (2026-09-26). Seen as
  an i386 `FatDeleteDirent` ASSERT (`FreeDirentBitmap` bits not set, twice per boot), but
  the cause was on every arch: the Rust `FatCreateNewDirent` had lost the tail of the C
  (`dirsup.c:414-534`) — the orphaned-LFN check, the "dirents really are unused" verify, and
  `RtlSetBits` on the free-dirent bitmap. So every new dirent stayed FREE in memory, and a
  later create that searches for deleted slots (`RtlFindClearBits`) could be handed a live
  entry and overwrite another file's dirent. arm64's system volume is fastfat. Restored;
  smoke 2/2 on all three arches, ASSERT gone, no FILE_CORRUPT raises.
- **Kernel user-pointer probes: a bad pointer bugchecks, and some never check — FIXED**
  (2026-10-04). The C syscalls probe inside `_SEH2_TRY` and return the exception code;
  this kernel has no SEH, so a raising probe was an unhandled exception (BUGCHECK 0x1E)
  wherever the C returns `STATUS_ACCESS_VIOLATION`, and several hand-ported helpers did
  no `MmUserProbeAddress` check at all (a user caller could make the kernel read or
  write a kernel address). Every raising `ProbeForRead`/`ProbeForWrite`/typed-probe call
  in Rust (~160, 20 crates) now uses `ntkrnl_sys::probe`'s `Result` probes with the C
  handler's cleanup; `ProbeAndCaptureUnicodeString` and `DefaultQueryInfoBufferCheck`
  were fixed centrally. `scan_raising_probes` gates the class at zero. Found on the way:
  open-coded 32-bit range tests (`ptr < 0x80000000 { check }`, i.e. no check for a kernel
  pointer; `0x7FFFFFFF` rejecting valid 64-bit user pointers) in config-rust and
  `ExQueueWorkItem`; WMI's capture writing a kernel stack address into the caller's user
  struct (a C bug); `KsSetInformationFile` probing its own pool buffer (a C bug: every
  call failed). Drivers outside ntoskrnl that use the probes need ntkrnl-sys feature
  `import_mm_user_probe_address`.
- **Post-probe user access: the probe passes, the access bugchecks — PARTLY FIXED, OWED**
  (2026-10-05). A probe proves where a buffer is, never that it stays mapped; the C
  touches it inside `_SEH2_TRY`, Rust touched it with plain loads and stores.
  `MmProbeAndLockPages` raises the same way. Done: `MmTryProbeAndLockPages` exported and
  used by every Rust caller outside a real `seh::` frame (`scan_raising_probes` now gates
  that name too); the query syscalls run against a pool copy of the caller's buffer
  (`ntkrnl_sys::probe::OutputBounce`, rebasing the pointers that point into it):
  NtQuerySystemInformation, NtQueryInformationToken/Process/Thread, NtQueryObject,
  NtQueryVirtualMemory, NtQueryInformationAtom, the Cm query/enumerate calls; the se
  captures (SID, SID array, ACL, LUID array, security descriptor, object type list) copy
  fault-tolerantly and pin the count/size they were sized by, so a racing caller cannot
  grow a SID or ACL past its allocation; win32k's captured strings
  (`safe_user_io::capture_string_buffer`); `ntkrnl_sys::probe::write_back` for the
  `IoStatusBlock`/handle/length write-backs. Real bugs found on the way:
  **NtQueryValueKey** "probed" by raw-storing into its output pointers with no range
  check, then had CmQueryValueKey write the value there — a user-chosen KERNEL address
  received registry data; **NtUserDefSetText** never probed the text buffer (kernel
  memory became the window title); **NtQuerySystemEnvironmentValue** had lost its whole
  probe block; NtAdjustPrivilegesToken/NtAdjustGroupsToken leaked a token reference per
  call; SeCaptureSidAndAttributesArray leaked its validation array per call and returned
  a freed array on failure; SeCaptureLuidAndAttributesArray never copied into a
  caller-supplied buffer and left `*Dest` dangling (a double free from NtFilterToken;
  the dangling pointer is the C's too). **Owed:** `tools/scan_raw_user_access.py` lists
  every Nt* syscall that still loads or stores through a pointer argument raw; its
  baseline (`tools/lint-baseline/scan_raw_user_access.txt`, 114 rows at filing) is the
  burndown list. Some rows are kernel-mode-only branches (read them, then `nolint`);
  most are real, led by win32k NtGdi*/NtUser*, NtSetInformationProcess/Thread reading
  their input in place, LPC's captures, and NtAllocateVirtualMemory/NtMapViewOfSection's
  write-backs. Callees are not covered by that scanner.
- **cdrom FeatureProfileList: the correct value takes the boot down — OPEN** (2026-10-04).
  `cdrom-rust/src/init.rs` held `FeatureProfileList = 0x0003` (an LLM "example value";
  cdrom.h says 0x0000). Correcting it (body-audit medium pass) made QEMU's DVD-ROM fail
  `DeviceGetMmcSupportInfo`'s TYPE_ONE probe (it answers with more than the one feature),
  and the device start then tears down through `DeviceClaimRelease(Release)` ->
  `WdfObjectDelete` -> `FxRequest::Release`, which reads past its object: amd64/i386
  BUGCHECK 0x19 at the next free, arm64 hang. `--tripwire "SPECIALPOOL=CDRO"` (KMDF tags a
  driver's objects with its service name) faults at the read: `wdf01000.sys+0x197FE`
  under `cdrom-rust init.rs:313`. Bisected to the constant alone (the commit's registry
  reads and CREATE context are innocent). The constant is held at 0x0003 with a comment;
  owed: why start fails on a TYPE_ALL-style answer, and why releasing a request the
  driver just created reads past it. Not a context-size mismatch: the three
  `CDROM_REQUEST_CONTEXT` type infos all use `size_of::<CDROM_REQUEST_CONTEXT>()`, and the
  same `DeviceClaimRelease` claims successfully on every boot, so suspect an object freed
  earlier on the new failure path. x64 lane.
- **Pre-seal translations lost statements: `rustconv body-audit` leads — OPEN** (2026-09-26,
  updated 2026-09-27). Crates translated before call sealing (fastfat, cdrom and the other
  early DeepSeek lanes) were never checked for dropped calls. `rustconv body-audit --all`
  (about a minute, $0) pairs every flipped-out C function with its Rust: 130 modules, 46
  clean, **9 silent stubs**, 249 `high` (state-changing callee missing), 988 `medium`, 110
  `low`. Leads, not verdicts: read each against the C. FIXED from the audit so far:
  - fastfat `FatCreateNewDirent` never marked new dirents used (entry above).
  - fastfat `FatFsdPnp` stopped at `return Status` (its source was truncated): every PnP
    IRP left the thread inside `FsRtlEnterFileSystem`'s critical region, kernel APCs
    disabled for good, plus a stale top-level IRP. Fixed independently the same day by the
    fastfat real-SEH lane (`bf8bbd0d86c`), whose version is the one in the tree.
  - fastfat `FatLockVolumeInternal` dropped `FatCloseEaFile`: a FAT12/16 volume with an
    `EA DATA. SF` kept an extra Vpb reference, so FSCTL_LOCK_VOLUME failed ACCESS_DENIED.
  - shell32 `SHUpdateImageW`/`A` and `SHHandleUpdateImage` had taken Wine's `#else` FIXME
    arm: icon changes were never announced or decoded.
  - classpnp `ClasspScanForSpecialInRegistry` was empty: registry quirk bits never applied.
  - kernel32 `BaseComputeProcessSearchPath` ignored `SafeProcessSearchMode` and
    `SetSearchPathMode()`.
  Checked and FALSE (each became a rule in the tool): shell32 `ILFindChild` (dead `#else`
  arm), cdrom `DeviceEvtFileClose` (delegates to live C glue), hal
  `HalpQueryPciRegistryInfo` (cfg twin), fastfat `FatAddToWorkque` (macro expansion),
  `FatAllocate/DeallocateDiskSpace` (macros defined in the .c), `FatCommonCreate`'s
  `if (_SEH2_AbnormalTermination())` unwinds (dead under DUMMY_PSEH), `FatMountVolume`'s
  `NTDDI_VERSION > NTDDI_WIN8` block. Checked, divergent but benign: fastfat
  `FatConstructNamesInFcb` stores an extended-character LFN Unicode-only, skipping the
  C's OEM round-trip attempt (no leak).
  `FatProcessException` was completed by the real-SEH lane. A crate on real SEH
  (`ntkrnl_sys::seh`) owes its except handlers and `AbnormalTermination` unwinds, and the
  audit now detects that per crate instead of assuming DUMMY_PSEH everywhere.
  **Second pass (2026-09-27): 48 clean, 2 silent stubs, 195 `high`** (was 46 / 9 / 249).
  The silent stubs split into two classes. First, **upstream work that landed on C already
  flipped to Rust**, which nothing re-ports:
  - win32k raw input: upstream #9498 added `rawinput.c` plus a device list in `input.c`
    after ntuser went Rust-only, so `GetRawInputDeviceList` and `GetRawInputDeviceInfoW`
    (forwarded straight to the syscalls by user32.spec) always reported no devices. Ported
    (`win32k-rust/src/rawinput.rs`, the list in `input.rs`, user32's
    `GetRawInputDeviceInfoA`/`GetRawInputBuffer`). This needed the win32k-sys island
    regenerated, whose generator had broken four ways (`rust-in-kernel.md`).
  - gdi32 `GetCharWidth*`/`GetGlyphIndicesA` lacked the DC validation of upstream #8690 and
    #8747, found by upstream-sync (`upstream-sync-diff-shape-wave2.md`) and never applied:
    a stale or non-DC handle went straight to the syscall. Ported; the ntgdi (kernel) half
    of both commits stays with upstream-sync.
  - netcfgx `ApplyPnpChanges` is one face of upstream's nine-commit `tcpipconf_notify.c`
    rewrite (adapter list instead of `pCurrentConfig`), tracked by upstream-sync as the
    "netcfgx cluster". OPEN there.
  Second, faithful stubs the audit misread, now rules in the tool: wlansvc
  `_RpcEnumInterfaces` (its body sits under a commented-out `GET_IF_ENTRY2_IMPLEMENTED`);
  gdi32's `EMFDRV_*` (Wine's old `enhmfdrv/` is on disk but in no CMake list, and the live
  `emfdrv.c` stubs replacing it are FIXME by design; a name defined by live C is no
  longer a translation pair). Further rules: `NTDDI_VISTA`-style version aliases (fastfat
  `FatOpenVolume`), an inline-expanded `IoSetCompletionRoutine` (`FatSingleAsync`), and a
  nested macro the Rust calls by name (`FatUnpinBcb`, `CdFreePool`,
  `RtlInsertElementGenericTable`). fastfat and cdfs are down to one benign lead each. condrv
  `ConDrvDispatch` is FALSE (the C only traces). Re-translate (with sealing) only what the
  audit and a reading of the C confirm.
  **Burn-down (2026-10-04): every high and medium lead read. 0 open high, 0 open medium**
  (from 198 / 951); 214 owed, 329 false/benign in `data/body_audit_reviewed.toml`, 55 `low`
  unread. About 70 real bugs fixed, among them: hal DMA locks that only raised IRQL (no
  exclusion on halmp) and S/G transfers over 64 KiB failing; cdrom completions skipping
  `RequestCompletion` (no verify-required marking); classpnp registry overrides passed as
  narrow strings (every override kept its default) and hotplug info a stub (every disk
  non-removable); scsiport parsing the service key as the DeviceN key; disk's invented SMART
  codes; kernel32 `UnhandledExceptionFilter` with scrambled constants (no post-mortem
  debugger, wrong crash box, a debugger-attached crash re-faulting forever) and
  `GetModuleHandle` misses returning stale errors; userenv shortcuts never written; csrss
  creating `\Session` instead of `\Sessions`; gdi32 never loading lpk.dll; ks stream IRPs
  with a NULL header; user32 button state overwriting the font handle on 64-bit; ole32
  property dictionaries unreadable. Owed, by cluster (upstream-sync): pciidex rewrite
  (~15K lines), PCIX arbiters/ECAM, netcfgx and dhcpcsvc alternate configuration, freeldr
  disk registration and boot menu, the Wine 10 ole32/oleaut32/mlang/crypt syncs, msgina and
  syssetup branding. **The msgina branding commit (6ae2d5ab43f) deleted IDI_ROSLOGO/IDI_BAR,
  so every logon, status and shutdown dialog shows no banner today: visible, on the boot
  path.** Still to decide: cdrom carries a live C-glue `DeviceEvtIoInCallerContext` that may
  shadow the Rust one under `--allow-multiple-definition` (check `nm`); the Rust masks the
  APIC error LVT where the C unmasks it.
  **ntkrnl-sys was stale against the post-merge headers — FIXED** (2026-09-27): regenerated
  the kernel and HAL islands. Nine structs had drifted: `KPRCB` (upstream `144fe39589b`'s
  `StartCycles`/`CycleTime`), `EJOB`/`PS_JOB_TOKEN_FILTER` (Rust still had the leaked
  `pack(1)` layout — upstream-sync wave-2 finding 19(b), which was INCONCLUSIVE), the
  arbiter's 64-bit `Length`/`Alignment`, `RTL_BSD_DATA` (three new items, ported into
  `BsdItemTable`), `EPROCESS`/`ETHREAD`'s `CycleTime` tails, `IO_BUS_TYPE_GUID_LIST`,
  `KIPCR`. The NT6 OB callbacks (`07217d32458`) no longer need the transmutes ob, iomgr,
  config and lpc carried past the old typedefs, and bindgen 0.73 now places a union's
  bitfields (`RTL_BALANCED_NODE`'s `Red`/`Balance`) at bit 0 as C does. Nothing caught
  the drift because `verify_offsets` was not gated and could not run on arm64 at all;
  `check_committed_drift` now regenerates the size/offset asserts in every gate (`check --strict`).
  The regen also let `scan_const_vs_island` see amd64 `MI_NUMBER_SYSTEM_PTES = 22000 * 22`
  (upstream `4cba65d760a`, #8613) against the Rust's 22000. Baselined, not ported: that MM
  layout change spans i386/amd64 init and is upstream-sync's (phase-4 review queue).
- **arm64: CPUs 2 and 3 never schedule a thread at `-smp 4`. FIXED** (landed
  `c141e7f6f8de`, 2026-09-26). Two causes: a hand-written `KNODE` stand-in put
  `ProcessorMask` at offset 0 (the real field is at 32), so every thread's ideal
  processor was CPU 0; and the System process kept the affinity captured before the APs
  started, where NT widens it with `KeSetAffinityProcess`. Fixing them exposed a third:
  arm64 `KeAcquireSpinLockRaiseToDpc` only raised IRQL and never took the lock, which
  under real concurrency corrupted the PFN lists (`0x4E`). `smoke --smp 4` 3/3, and the
  Pi runs four cores to the desktop.
- **Notepad's menu bar renders blank. FIXED on master, cause not traced** (seen
  2026-09-26 morning on arm64 QEMU and the Pi). The same evening the Pi drew the full
  `File Edit Format View Help` bar on build `f5abd8cee6a4`; the fix came in with a
  master merge and was not identified.
- **`ntkrnl-sys` had DRIFTED from the C headers on all three arches, and three tools that
  would have caught it were broken by one cause and all three exited 0. FIXED 2026-09-26**
  -- regenerated, plus the one translation the stale island had been hiding (below). i386
  boots to desktop; arm64/amd64 are verified by the landing gate.

  **The regen surfaced a stale TRANSLATION, not a bindings bug.** i386 went to BUGCHECK 0x19
  (pool header corrupted just past a 56-byte `TAG_NONE` block, identical down to
  flight-record `seq=522` every run) while the baseline booted clean. `SPECIALPOOL=*`
  turned it into a 0x50 whose trap frame put the write in `pnpmgr-rust`
  `IopGetBusTypeGuidIndex`. Upstream `a7f658e3221` (CORE-12791) had redesigned
  `IO_BUS_TYPE_GUID_LIST` from a struct with an inline `Guids[]` tail into a global struct
  holding a separately allocated `PGUID` array; it arrived with the 1,269-commit merge into
  `pnpmgr.c`/`pnpinit.c`, both already cohab-flipped, so the fix went dead on arrival.
  The Rust kept the old algorithm, and the stale island kept describing the old struct, so
  the two agreed with each other and nothing complained. Against the real header the first
  GUID was written over the `Guids` pointer and 12 bytes past a 44-byte block: 8 bytes into
  the next pool header on i386, and silently into the 16-byte rounding slack on 64-bit
  (so amd64/arm64 carried the corruption without crashing). Ported; the old body also freed
  the list and then released a mutex inside it on allocation failure, which upstream's
  shape removes.

  **That points at a much larger owed queue:** `upstream_triage.py` flags **428 ORACLE
  commits inside the already-merged range** (`28a26f3b664^1..^2`) -- upstream changes to C a
  CMake comment has switched off, i.e. files we translated -- and the oracle work so far
  read only the unmerged tail. Each landed as dead C next to Rust nobody re-read. (A first
  count said 465; the tool then learned to separate cohab-flipped files from modules this
  port never builds, which moved 25 to UNBUILT. It cannot tell a fix from a feature, so 428
  is a reading list, not a bug count -- in the unmerged tail 3 of 13 hits were real
  correctness bugs, and `a7f658e3221` was one of these 428.)

  Ruled out along the way, by tool: no extern arity mismatch in any kernel crate on i386
  (`verify_signatures`, after fixing its HIGH false positives from comments inside param
  lists); the four arity changes the regen made all match the existing Rust definitions.

  **`ntdll-sys`, `winlogon-sys` and `winsrv-sys` were stale too -- REGENERATED 2026-09-30.**
  A different root cause: each generator lifts flags from one ninja rule, and that rule's `.c`
  went to Rust when the module was flipped (`csrsrv/init.c`, `winlogon.c`, `consrv/alias.c`).
  The grep then found nothing and bindgen ran with no `-I`/`-D` at all, so these islands could
  not have been regenerated since the flip; `verify_sizes.py` did the same and exited 0 having
  checked nothing. Both now lift from a still-compiled stub TU and hard-fail on empty flags.
  winsrv-sys's anonymous-member inlining (`CONSOLE_START_INFO` embeds `CONSOLE_PROPERTIES`)
  silently stopped too -- bindgen 0.73 spells the padding `__BindgenOpaqueArray8<...>` and a
  miss was a WARN -- now fatal, with a layout assert. The regen surfaced one real ABI bug:
  `ConDrvCreateScreenBuffer` is cdecl in `conoutput.h` but the Rust was `extern "system"`, and
  once the island said so LLVM turned the i386 call into `ud2`. Upstream's
  `OBJECT_HANDLE_ATTRIBUTE_INFORMATION` -> `..._FLAG_INFORMATION` rename followed in three crates.

  What was wrong: `EPROCESS` measured 1000 bytes in C and 992 in Rust on arm64 (the Rust
  struct stopped at `Cookie` and omitted `ULONGLONG CycleTime`, which the header adds under
  `#if defined(__REACTOS__)` as a "Temp HACK until we switch to NTDDI_VISTA"); `ETHREAD` 1032
  vs 1016; also `EJOB`, `KIPCR`, `ARBITER_ALTERNATIVE`, `IO_BUS_TYPE_GUID_LIST`,
  `PS_JOB_TOKEN_FILTER`, `RTL_BSD_DATA`, `STACK_TRACE_DATABASE` -- ~10-11 structs per arch,
  and 5,623 lines / 55 new + 10 removed structs between the committed arm64 island and a
  fresh run.
  Root cause is ONE line in two places: the bindgen wrapper and `verify_sizes.py` both
  define `__wfi()`/`__yield()` as zero-arg function-like macros, and since
  `intrin_arm64.h` landed (`28a26f3b664`, 2026-09-20) `intrin.h`'s `void __yield(void);`
  prototype parses as a one-argument invocation of that macro. So `bindgen` ABORTS on arm64
  (no arm64 regeneration has been possible since 2026-09-20) and `verify_sizes.py` emitted
  ZERO arm64 assertions -- and, because the error is in a header rather than in the
  generated file, its retry loop that blanks offending type lines could never clear it.
  All three scripts reported success. All three are FIXED now (stubs dropped -- a real clang
  handles the asm bodies that only libclang could not -- plus real `error:` diagnostics
  instead of the include chain, and `verify_sizes.py` now honours `all_ok` in GENERATE mode,
  which is what let it lose a whole arch and still print "Done."). A fourth tool bug sat
  behind them: `postprocess_bindgen.py`'s two helper extractors used a non-greedy match
  bounded only by `\n#\[repr`, so they ran past the NEXT type's `#[derive(...)]` line and
  swallowed it -- `__BindgenOpaqueArray` arrived derive-less and broke `#[derive(Copy, Clone)]`
  on every struct containing one, `_KPRCB` included. `\n#\[derive` is now in both boundary
  sets. Note the
  `_INTRIN_H_`/`_INTRIN_X86_H_` defines beside the stubs are inert -- `intrin.h` is
  `#pragma once` + an unconditional `#define __INTRIN_H_` with no `#ifndef`, and
  `_INTRIN_X86_H_` matches nothing in the tree.

  **`tools/verify_offsets.py` has the SAME bug** (same `intrin.h:1029` failure, same rc=0,
  same silent arm64 skip -- and it ignores `--arch` and runs all three anyway), and its
  regenerated amd64/i386 files moved 80 of 132 lines, which is independent corroboration
  from a second implementation. It also makes the drift FIELD-LEVEL, not just trailing
  size: 43 `offset_of!` assertions fail on amd64 across 7 structs. Earliest drifted field
  per struct, which is what decides severity:
  `_IO_BUS_TYPE_GUID_LIST.Lock @ 0` (a field inserted at the FRONT -- every field after it
  is displaced), `_PS_JOB_TOKEN_FILTER.CapturedSids @ 8`, `_ARBITER_ALTERNATIVE.Alignment
  @ 24`, `_RTL_BSD_DATA.ErrorInfo @ 68`, `_EJOB.UIRestrictionsClass @ 248`, and -- benignly
  -- `_KPRCB.FeatureBitsHigh @ 11416` and `_KIPCR.ContextSwitches @ 11808`, both new tail
  fields on structs of 11824 bytes. So the reason the OS still boots is that the two
  boot-critical structs (KPRCB/KIPCR) drifted only at the TAIL; the front-loaded drift is in
  structs the boot path does not reach. That is luck, not safety. (I first wrote that all
  the drift was tail padding. That was wrong, and I had not yet run the offset verifier
  when I wrote it.)

  **What the regen actually cost, versus my estimate.** I sized it from the size assertions
  (7 structs, 66 consumer references) and called the risk "concentrated in a handful of
  files". The consumer counts were right and the conclusion was right, but the reasoning was
  luck: severity and blast radius are inversely correlated here. Consumer files outside the
  generated files themselves --
  `KPRCB` 22, `KIPCR` 16, `RTL_BSD_DATA` 15, `EJOB` 6, `PS_JOB_TOKEN_FILTER` 5,
  `IO_BUS_TYPE_GUID_LIST` 2, `ARBITER_ALTERNATIVE` 0 (66 references total). The two with the
  most consumers are exactly the two that drifted only at the tail, so regenerating them
  APPENDS fields and leaves every existing access at its current offset; the one with
  offset-0 drift has 2 references.

  Seven real defects fell out, every one structurally invisible beforehand: four
  `CloseProcedure` callbacks declared with `ULONG` handle counts where `OB_CLOSE_METHOD`
  passes `ULONG_PTR` (`config-rust` `CmpCloseKeyObjectCallback`, `mm/section.rs`
  `MmpCloseSection`, `iomgr-rust` `IopCloseFile`, `lpc-rust` `LpcpClosePort`) -- **two of them
  hidden behind `core::mem::transmute`, with comments citing each other as precedent for
  "the identical drift"**; `ob-rust` importing `ObpSetHandleAttributes` from the substrate
  when it is `static` in C and the crate defines its own; and three MISSING
  `rtl-bootdata::BsdItemTable` rows after `RtlBsdItemMax` grew 16 -> 19, an out-of-bounds
  index that nothing could detect because the array length and its bound came from the same
  stale file. Fixing the callbacks needs `ntkrnl_sys::ddk::ULONG_PTR` specifically, not
  `base::ULONG_PTR` (`usize`): same width, distinct type, and only the declared alias coerces
  into the slot. See the field manual, "A self-consistent pair of DERIVED values cannot
  detect its own staleness".

- **`SPECIALPOOL=*` cannot get an i386 boot past `wdf01000.sys`. OPEN, 2026-09-26.** It
  faults (0x50, READ) in vendored KMDF `FxRequest::Release`: the source reads `m_Completed`
  only behind `retValue == 1 &&`, but clang hoists the load of `this+0x6a` above the check
  (it treats `this` as dereferenceable), so after `FinalRelease` deleted the object it
  reads freed memory. Harmless on a normal pool (the value is discarded), fatal under
  special pool because the freed page is unmapped. Not a corruption and not new. Until the
  hoist is suppressed for that TU, arm special pool by TAG on i386, not `*`, or every
  wildcard run stops here and reads as a new bug.

- **The ucrt Tier 2 case generator reads the WRONG headers, and drops what they lack
  without a word. OPEN, owed** (2026-09-26). `gen_ucrt_tier2_cases.ast_functions` parses
  `#include <mbstring.h>` etc. with no `-I`, so clang resolves the llvm-mingw toolchain's
  own mingw-w64 headers, not `sdk/include/ucrt`. Any UCRT export mingw does not declare
  is never generated and is not in the "held out" announcement either: found because
  `_ismbcblank`/`_ismbcblank_l` (declared at `sdk/include/ucrt/mbstring.h:1134`) were
  missing from the ismbc takeover check. Fix: parse the UCRT headers the DLL is built
  against, then regrade the whole table, since the case set will move. Until then the
  table's coverage is "what mingw declares", not "what ucrtbase exports".
  2026-10-03: `sdk/include/ucrt` is deleted, so "the headers the DLL is built against" is
  now sdk/include/crt, which declares less, not more. The honest source of the export
  list is `ucrtbase.spec`; the declarations for the ones mingw lacks come from git
  history at the `--takeover-from` revision, the same place the reference C now lives.

- **`test_payload_launched` misses a launch marker that IS in its own serial log.
  ROOT-CAUSED + FIXED 2026-09-26: `restore_snapshot`'s flush ate it.** A fast payload
  finishes inside the settle window; a CHATTY one never lets serial go idle, so the settle
  timed out and kept none of its window (unconsumed, as read below -- correctly); the
  `ready` snapshot was taken with the payload already done; then the first test's
  `restore_vm` ran `restore_snapshot`, whose `read_available` flush discarded the unread
  bytes, marker included, and rewound the guest past the payload's exit so it never printed
  it again. Reproduced live on a ucrtdiff shard (marker at 02:28, pytest still draining at
  02:57). Fix: `QemuVM._absorb_pending()` moves pre-snapshot output into `_boot_output` on
  the settle timeout and before `savevm`; unit-tested both ways. Original report
  (2026-09-25): `smoke --arch i386 --payload oldcrt`: the payload's
  `payload/i386-uefi_serial.log` carries `[OLDCRT] done PASS` at byte 78,018, pytest
  reported `Boot complete (70445 bytes)` and then `drain-end: budget` -- 300s of draining
  without matching the marker -- and failed the run
  (`/tmp/vibeos-cli/20260925-210919-beje4luf-smoke-i386`). ucrtdiff showed a relative
  the same day: pytest never finished after `[UCRTDIFF] done` reached its log, and the
  main thread sat in `poll`. Read and ruled out, NOT reproduced: `read_until` and
  `read_until_idle` both leave `_pos` correct (the settle's TimeoutError path does not
  consume), and the drainer appends the same text to `_buf` that it tees to the file.
  So the in-memory stream and the tee diverge somewhere not yet found. Until it is, a
  payload's verdict must be read from its serial log, not from the smoke exit code.
- **Pyright does not cover the harness core -- RESOLVED (2026-10-05).** `_PYRIGHT_ROOTS` now
  covers all of `tests/qemu`. The first pass found 18 errors, none a live bug: unchecked
  `Optional` `vm._proc`, PIL's loose `getdata()` typing (now a histogram count), net-port
  reads off the base `ArchConfig` (now narrowed to the net configs, failing loudly
  otherwise), and a dead write of `vm._idle_cpu_pct` that nothing read.
- **`test_bindgen_generation` fails under PARALLEL check-tooling and passes alone — it
  blocked a land. OPEN** (2026-09-25). Five of its eleven cases failed inside
  `./vibeos-cli check-tooling` with
  `bindgen: refusing output in another worktree: /tmp`, which is
  `tools/bindgen-stage.sh`'s ancestry walk deciding that `/tmp/.git` exists. Run on its own
  the same file is 11/11, and the very next full `check-tooling` was 69/69, so the guard is
  reacting to something transient rather than to the test's own tmpdir — **what creates a
  `/tmp/.git` is NOT identified**, and no such path exists at rest. Same class as the two
  load-sensitive tests below: the cheap discriminator is running the single file, and it
  should be run before believing a lane broke this.
  It is not free to ignore: this is a `land` PREFLIGHT, so a transient failure rejects the
  submission of whichever lane happens to hit it.

- **THREE user32 winetest subtests BUGCHECK i386 0x1E, and two of them land on
  probe-then-raw-access leads the entry below already lists as OPEN. FIXED** (2026-09-26,
  `d295a7227d3` + `79b1a131ad7`). All 24 user32 subtests now sweep to completion with zero
  quarantined and zero BUGCHECKs. Four distinct faults, not three -- fixing `win`'s
  `NtUserFlashWindowEx` merely uncovered a second one behind it at `win32k.sys+0x91F6E`,
  which disassembles to an INLINED ProbeForWrite followed by `movl %eax, (%edi)`:
  `ONEPARAM_ROUTINE_GETPROCDEFLAYOUT`'s `*pdwLayout = ppi->dwLayout` in
  `NtUserCallOneParam`. `scan_probe_then_deref.py` had reported that file clean because the
  probe is on `Param` and the store goes through `pdwLayout = Param as PDWORD` -- an ALIAS.
  One assignment hid a live unprivileged kernel crash from the tool written to enumerate
  this exact class; the scanner now propagates taint across such casts (+8 leads elsewhere).
  `input` was correctly filed as a different shape and is a different shape: there is no raw
  dereference in the function at all. It hands a UNICODE_STRING whose `Buffer` is a user
  pointer to `RtlIntegerToUnicodeString` / `RtlAppendUnicodeToString`, which write through it
  with a bare `RtlCopyMemory`, so the fault lands INSIDE ntoskrnl with our frame as the
  caller. Fixed by building into a kernel buffer and publishing, the way
  `NtUserLoadKeyboardLayoutEx` in the same file already does for the read direction.
  The original diagnosis, kept because the reasoning was sound and one part of it was wrong:
  The first sweep of a whole conformance suite (`--winetest-sweep`, native i386, no WoW64
  anywhere) found all three; each one blinds every subtest after it, so they had to be
  quarantined one at a time to see the next:
  | subtest | BUGCHECK PC | lead |
  |---|---|---|
  | `win` | `NtUserFlashWindowEx+0x6f` | `painting.rs:2006` — probed `pfwi`, then `RtlCopyMemory` off it |
  | `msg` | `NtUserPeekMessage+0xf3` | `message.rs:3275` — probed `pMsg`, then `RtlCopyMemory` into it |
  | `input` | PC in ntoskrnl, `NtUserGetKeyboardLayoutName+0x158` as caller | not characterized |
  **What this changes is the priority, not the inventory.** The 152 bulk-copy leads below are
  already known and already open; what they lacked was any evidence they were reachable. Two
  are now demonstrated machine-killers, reached by a stock Wine subtest doing nothing exotic.
  A scanner count is audit debt; a reproducer is a bug.
  `input` is a different shape and should not be assumed to be the same one: its PC is in
  ntoskrnl rather than win32k, and it follows a run of
  `(win32k-rust/src/keyboard.rs:1689) err: Cannot copy key state` — the `capture_from_user`
  of the 256-byte key-state array failing repeatedly, which is the SAFE helper, not a raw
  access.
  Repro: `build --arch i386 --rostests` once for the binary, then
  `build --arch i386 --winetest-sweep` and boot `smoke --arch i386 --ladder` with
  `VIBEOS_LADDER_DRAIN_FLOOR=1500`. Add `--winetest-skip "input msg"` to get past the first
  two.

- **The user32 winetest A/B, whole suite, both sides: 17 identical and 7 guest-only.
  REFERENCE, ALL SEVEN RESOLVED** (2026-09-25; `combo`, `winstation`,
  `scroll`, `dde`, `uitools` FIXED and `monitor` no longer diverging, 2026-09-26, see below). First time the WoW64 control has run across a whole
  suite instead of two subtests. Same 21 subtests both sides, same quarantine
  (`input msg win`, all three BUGCHECK — see above), so the columns are comparable.
  Seventeen matching is the control doing its job; those are OUR user32, not the seam.
  | subtest | native i386 | WoW64 | |
  |---|---|---|---|
  | `combo` | 23 failures | ~~CRASH `0xc0000094`~~ **23 failures** | **FIXED 2026-09-26** — a width bug, as the shape said; now identical assertion-for-assertion |
  | `winstation` | 13 failures | ~~CRASH `0xc0000005`~~ **13 failures** | **FIXED 2026-09-26** — three bugs, one of them not WoW64's; see below |
  | `dde` | ~~TIMED OUT~~ **24** | ~~CRASH `0xc0000005`~~ **24** | **FIXED 2026-09-26**; the guest now PASSES one assertion native fails (929) |
  | `monitor` | 20 failures | ~~23~~ **20** | **NO LONGER DIVERGES 2026-09-26** — cause unattributed, see below |
  | `uitools` | 0 failures | ~~1~~ **0** | **FIXED 2026-09-26** — executing the guest GDI batch; 42 tests both sides |
  | `scroll` | ~~140~~ **14** | ~~17~~ **14** | FIXED 2026-09-26 on BOTH sides; now identical line for line |
  | `dialog` | **CRASH `0xc0000005`** | 78 failures | the NATIVE side is worse; this one is not the seam's |
  **`combo` was win32k reading guest user32's private `HEADCOMBO` at the amd64 layout.**
  `NtUserGetComboBoxInfo` does not ask the control; it follows the pointer in the window's
  extra bytes into user32's own struct. A WoW64 guest's is 32-bit, so `hwndList` came from
  16 bytes too far, the test sent `LB_GETITEMHEIGHT` to a non-window, got 0, and divided by
  it. Fixed by reading at the OWNER's layout (`window.rs` `CaptureComboState`; same fix for
  `NtUserGetListBoxInfo`'s `LB_DESCR.page_size`), probed and captured, which also closes an
  unprobed kernel read: the process controls that pointer via `SetWindowLongPtr`. Verified:
  guest 460 tests / 23 failures, native i386 the same 23, and the 28 output lines diff
  IDENTICAL once handle values are normalized. Repro loop is now
  `build --arch amd64 --winetest-sweep --winetest-only combo` (seconds, not a 40-min sweep).
  **`winstation` was three bugs stacked.** (1) `NtUserCreateDesktop` had no marshaller, and
  its loud refusal returned a non-zero status that the guest read as a valid `HDESK`. (2)
  Behind it, `NtQueryObject` was a flat thunk, so `ObjectNameInformation` came back in the
  host's 16-byte `UNICODE_STRING` layout and the test dereferenced a junk `Name.Buffer`.
  That one affects every guest: kernel32 (`dosdev`, `GetFinalPathNameByHandle`) and
  advapi32's HKCR code ask for object names too. (3) `EnumWindowStations` failing with
  `ERROR_INTERNAL_ERROR` was NOT the seam: win32k sized its fallback buffer from a
  `NtQueryDirectoryObject` NULL/0 query, which reports bytes written rather than bytes needed,
  and only a 64-bit kernel's 32-byte entries overflow into that path. Now 13 failures on
  both sides. Two differences remain, neither attributable: the foreground-window asserts
  (a timing-sensitive loop, failing on both sides at different iterations) and 1054
  (`WM_NCHITTEST` on an invisible desktop), which native never reached because its child
  launch failed first with `CreateProcessA` error 2 on `argv[0]`. FIXED the same day:
  winlogon passed a bare exe name as argv[0], and an lpApplicationName resolves against
  the CWD only. The guest had worked by accident, via WoW64's system32 -> syswow64
  redirection. With full paths, 1054 fails on BOTH sides (not the seam), and only one
  foreground-loop iteration differs.
  **`monitor` re-measured 2026-09-26: 20 and 20**, and the assertion lists match line for line
  except one native-only `WaitForSingleObject returned 102` (3181), a native timing miss.
  The +3 does not reproduce, and WHICH three assertions they were was never recorded (the
  09-25 log died with `/tmp`), so the fix cannot be attributed: that day's `NtQueryObject`
  marshaller and the upstream merges since are both candidates, and neither is proven.
  Record per-assertion diffs, not counts, or a disappearing divergence cannot be explained.
  **`dde` was three things, and the middle one was never about DDE.** (1) The DDE
  post/get user callbacks (13/14) had no WoW64 bridge: `[WOW64CB] callback 13 NEEDS
  64->32 MARSHALLER`, so every cross-process DDE data message failed to post. (2) With
  the bridge in, the posted `WM_DDE_DATA` still carried `lParam 0`, because the guest's
  `PackDDElParam` got NULL from `GlobalLock` on a handle `GlobalAlloc` had just returned.
  kernel32 fills `SystemRangeStart` at startup with
  `NtQuerySystemInformation(SystemRangeStartInformation, &x, 4)`, which the WoW64
  marshaller passed through to a host that answers 8 bytes. It failed
  `STATUS_INFO_LENGTH_MISMATCH` in EVERY guest (19 per boot), left the value 0, and
  `GlobalLock`'s `hMem >= SystemRangeStart` then refused every FIXED global in every
  32-bit process. Class 50 is now marshalled (0x80000000, matching the guest's
  0x7FFEFFFF user ceiling). (3) Found on the way: gui32 had improved to a full `0x600D`
  (desktop readback now works) and the ladder failed it for not being the soft-pass
  `0x600E`. The queue's integration tree does not stage the guests, so only lanes saw
  it; `ladder.ALSO_ACCEPTED` now names the upgrade. Now 24 on both sides; against a quiet
  native run the guest differs only by PASSING 929, plus one extra trailing byte in the
  2510 failure both sides share.
  **`uitools` FIXED 2026-09-26: guest 42 tests, 0 failures, same as native**, re-measured
  with `build --arch amd64 --winetest-sweep --winetest-only uitools` once the guest batch
  was executed rather than reset. The diagnosis, kept because it was the first measured
  cost of the discard: it was the discarded guest GDI batch. `uitools.c:58` does `SelectObject(GetStockObject(BLACK_BRUSH))` then
  `FillRect(hdc, &r, NULL)` — a NULL brush means "use the DC's current brush", so user32
  correctly goes straight to `PatBlt(PATCOPY)`. The traced syscalls for that subtest show
  `NtGdiGetStockObject`, `NtGdiCreateBitmap`, `NtGdiSelectBitmap`, `NtGdiGetPixel` — and **no
  `NtGdiPatBlt` and no brush select at all**. Both are batched, the batch is reset rather
  than executed, the fill never happens and `GetPixel` returns white. The GDI-batch entry
  says executing them needs four 32-bit record layouts and names `GDIBSPATBLT.hbrush`
  first; this is that record. It also says the discard "costs no drawing that was not
  already dropped" — true, and now known to cost a conformance assertion as well.
  **`scroll` diffed 2026-09-26, and the 140 -> 17 was neither a win nor a loss.** The whole
  gap is the `write_process` child: native's reports "128 failures in child process" with
  its output invisible, the guest's runs 12 tests with 2 failures (905/911). The parents
  differ by exactly three guest-only failures, 161/170/179 (`GetScrollPos(SB_CTL)` wrong
  after a `SetScrollPos` that worked). FIXED: guest user32 read the control's `SBCALC` as
  `pwnd as PSBWND`, i.e. off the end of its 32-bit WND SNAPSHOT instead of the window's real
  extra bytes. It now goes through `wnd_extra_base` (scrollbar.rs `SBCalcOf`), the fix M11
  made for EDIT. Guest now 14. **The native 140 was a crash, not 140 failures**: winetest
  reports a child's EXIT CODE as its failure count, and native's `write_process` child
  died with `0x80` (csrss terminating a second-chance exception). i386 could not show it,
  since it had neither the `[EXIT-THR]` trace nor the second-chance `[KI-EXC]` tripwire amd64
  has had for months. Both are now on i386, and the first run named
  `uxtheme!ThemeSetScrollInfo`: `ThemeGetWndData` returned the OWNER's heap pointer for
  another process's window (the property is global, the pointer is not), and
  `ThemeDestroyWndData` had always guarded this while the getter never did. Also fixed: its
  default path passed an uninitialized `SCROLLINFO`. Native 140 -> 14, identical to the
  guest line for line, 905/911 included. Also fixed: i386 `wow64_unsupported` fired in
  NATIVE processes ("SBINFO ... results will be garbage" on bare i386), a false alarm now
  gated on `IsWow64()`.
  **The original caution stands as method:** a conformance suite reporting fewer failures
  usually means it stopped asking. A conformance suite reporting fewer failures usually means it stopped
  asking, not that we got better, and the byte counts (896 native vs 1271 guest) do not
  settle it either way.
  Cost, for budgeting a wider sweep: most subtests are under 3s natively; the outliers are
  `dde` 180s (the timeout itself), `sysparams` 128s, `class` 21s, `clipboard` 20s,
  `monitor` 14s. Under WoW64 the same suite is roughly 3x that — `class` 115s, `sysparams`
  80s, `clipboard` 73s — so a 21-subtest guest sweep needs a drain floor near 2400s.
  Repro: `build --arch <a> --winetest-sweep --winetest-skip "input msg win"` on both
  arches, then `smoke --arch amd64 --ladder` and `smoke --arch i386 --ladder`.
  **Stated because it would otherwise read as a green run: the amd64 sweep boot FAILED.**
  `BUGCHECK 0x19 (BAD_POOL_HEADER, param1 0xa)` at serial line 996885 — 348k lines AFTER
  `sweep complete` at 648787 — during the stock-applet phase with doom running, with a
  free-list dump whose entries mostly carry `tag=00000000`. The A/B numbers above all
  predate it and are unaffected, but the boot is not green and this is NOT attributed:
  whether 21 extra guest processes provoked it or it is the pre-existing amd64 pool flake
  was unmeasured. **Discriminator run 2026-09-26: a plain non-sweep ladder boot, stock-applet
  phase included, is clean** (no BUGCHECK;
  `/tmp/vibeos-cli/20260926-102824-75mdh3kl-smoke-amd64`). That leans toward the sweep's
  extra processes, but it is ONE boot and a pool flake is by nature intermittent, so it
  lowers the prior rather than settling it. The next sweep boot is the real test.
  (Original artifact lost to a `wsl --shutdown`.)

- **arm64 media did not stage ucrtbase.dll, so no arm64 process could import it. FIXED**
  (2026-09-23). Symptom: an exe importing ucrtbase.dll never reached its first
  instruction and printed NOTHING on serial, while the boot itself reached
  `shell ready`. `LoadLibraryA("ucrtbase.dll")` from a bare-entry exe returned
  **err=126 ERROR_MOD_NOT_FOUND** (msvcrt.dll loaded fine in the same probe), which was
  the first error code this failure ever produced -- a static import fails before any
  code runs, which is why it was silent.
  Cause: `qemu/mkramdisk.sh` staged ucrtbase.dll only under `VIBEOS_FEX=1`. The DLL is
  built, and it IS in the livecd graft list -- but arm64 boots `boot.img` from the
  ramdisk script and never the livecd, so nothing noticed. Fixed by staging it
  unconditionally beside msvcrt.dll.
  Worth remembering as a shape: the i386 and arm64 media are assembled by DIFFERENT
  pipelines, so "it is in the CD file list" proves nothing about arm64. Check the media
  the arch actually boots (`arm64_launch.json` names it).
  This had blocked Tier-2 ucrt difftesting, which must run on arm64 because VEH
  first-chance dispatch is wired only there.
- **`SHObjectProperties` on a file hangs in ole32 marshalling. FIXED 2026-09-25**, and it was
  five bugs deep. Right-click > Properties now opens a correct sheet (name, type, location, size,
  dates, attributes) on i386; `tools/payloads/comprobe` checks it end to end with a screenshot.
  1. No interface had a proxy/stub registration: `CoGetPSClsid` gave `REGDB_E_IIDNOTREG` for
     IDataObject, IStream, IUnknown... The livecd registers COM from `setup -mini`
     (syssetup `InstallLiveCD`), which never got that far: kernel32's INI parser computed every
     length in BYTES (`PROFILE_Load`), so `GetPrivateProfileStringW` on `unattend.inf` never
     returned. Same file: `CurProfile` was its own static instead of `MRUProfile[0]`, and a
     `return TRUE` was dropped.
  2. Even with setup unstuck, ole32's `DllRegisterServer` wrote nothing: the upstream sync kept
     `7675017f60d`'s `.rc` change but lost its `add_idl_reg_scripts` line
     (`tools/test_wine_registry_resources.py` guards it now).
  3. Every value on the sheet drew as one glyph: `NtGdiGetCharABCWidthsW` read the LFONT as
     the realized FONTGDI, so its SharedFace guard refused every call and every
     `SS_*ELLIPSIS` static truncated. Same fix repairs dxdiag's values and Regional Options.
  With setup's LiveCD phase now running, PlugPlay starts too, which is what devmgmt and dxdiag
  were failing on (`RPC_S_SERVER_UNAVAILABLE`). setup no longer launches userinit when a shell
  is already up (vibeos's winlogon starts it directly).
- **App sweep findings, 2026-09-25 (`tools/payloads/appstorm`). OPEN.** 38 stock apps and
  applets launched, checked and screenshotted per arch. Fixed on the way: an i386 0x1E in
  `ExAcquirePushLockShared` from `CmpQueryKeyName` (a local lock helper skipped the KCB hash);
  `GetClassNameW/A` dropping the terminator; user-mode `DbgPrint` returning failure; Toolhelp
  snapshots with their counts overwritten; every kernel-built file-object name two characters
  short (`IopQueryNameInternal` used 8 for `FIELD_OFFSET(FILE_NAME_INFORMATION, FileName)`, so
  process names read `svchost.e`). The kmtest sweep now waits for setup's PnP device-install
  burst (~25-35s, measured) before its first test: started inside it, the load-sensitive tests
  failed on every attempt. Still open:
  - FIXED: folder windows opened from another process never appeared (`explorer.exe <path>`,
    Control Panel, `ncpa.cpl`). Two bugs. (1) `gen-delaylib.py` bound browseui's `NONAME` export
    `SHOnCWMCommandLine` by name, so explorer's `CWM_COMMANDLINE` handler raised `0xc06d007f`
    (all arches). (2) CControlPanelFolder handed its IShellFolder2 pointer to the view as the
    IShellFolderViewCB, so the view's first `MessageSFVCB` ran `ParseDisplayName` and faulted in
    CRegFolder. appstorm on amd64: explorer-folder, Control Panel (18 applets drawn) and
    Network Connections all PASS.
  - FIXED: closing a folder window faulted at RIP 0 in `browseui!CShellBrowser::OnDestroy`
    (`fCurrentShellFolder.Release()` on a freed folder). `CDefView_OnCreate_body` released
    `pdt` and `ppf2` explicitly AND through their CComPtr guards, so every view creation cost
    its folder one reference. Found with a gdb probe logging each AddRef/Release of the folder.
  - OPEN: with that fixed, browseui warns `Unexpected RefCount > 0 (1)` for the view at close.
    The double `pdt` release (the view's own IDropTarget) had been hiding a +1 leak on CDefView.
  - FIXED: the i386 Control Panel "hang" was a thread busy loading applets while boot-time
    `setup -mini` was still installing devices (7.8s to answer). appstorm now waits for setup
    to exit and gives a slow window 60s, sampling its PC; on amd64 and i386 explorer-folder,
    Control Panel and Network Connections all PASS with titled windows.
  - FIXED: a process that loaded and FreeLibrary'd its only user32 importer (Control Panel's
    applet enumeration in a console process) unloaded user32; the reload failed DllMain on the
    thread's stale `CI_INITTHREAD` (1114), every later user32 importer failed (1450), and
    `powercfg.cpl` hung on i386. user32 now pins itself once connected. `tools/payloads/cplreuse`
    replays it: 12/12 loads on amd64 and i386.
  - FIXED: amd64 setup.exe faulted (RIP `0x1b85ac0`) as registration began. A gdb write-watch
    showed `LdrpManifestProberRoutine` being set by the real setter, from a SECOND kernel32
    mapped at `0x1b50000`: ucrtbase's api-set load went through a redirect that returns a bare
    host name while claiming `RedirectedDll`, so the already-loaded check compared it against
    full paths and never matched. The loader now looks bare redirected names up by base name.
    Registration then reached the csrss crash below.
  - FIXED: registration crashed csrss in `winsrv!ConSrvGetConsole` (0xF4):
    `GetThreadConsoleDesktop` read `ServerData[1]` (basesrv's slot) instead of
    `CONSRV_SERVERDLL_INDEX` (2). amd64 now logs `registration done`.
  - amd64, OPEN: dxdiag faults in `RtlFreeHeap`; explorer raises `0xc06d007f` (a delay-load
    import that does not resolve; arm64 too).
  - FIXED: amd64 bugchecks under long GUI runs: 0x1E `c000009a` from a pool allocation, and
    0x1E `c0000005` in win32k (`REGION_Delete(NULL)` from `IntSelectClipRgn` and
    `VIS_ComputeVisibleRegion`). All one bug. `NtUserFindExistingCursorIcon` had a stray
    capture of the module name INTO the resource-name slot, so no LR_SHARED icon ever matched
    (IDC_ARROW looked up 4139 times, never found). Each LoadCursor/LoadIcon made a new icon + two
    bitmaps and leaked a pool string; a repainting WOW64 app reached the 10K GDI quota, region
    allocations returned NULL, and the pool ran dry. Found by counting create vs delete per
    handle type in the WOW64 syscall trace, then logging the icon-cache calls
    (`[WOW64-CURICON]`).
  - OPEN, harness: `smoke --payload rpmprobe` on amd64 FAILED with "`[RPMPROBE] done` never
    appeared" although the serial file has it (line 4201, byte 236K, `VERDICT=PASS`). Boot
    output was 78K bytes; the drain then ran its whole 600s budget over a 39.8MB serial (WOW64
    trace) without matching. The file shows `shell ready` three times for one payload run, so the
    `ready` snapshot restore is the first suspect: the reader may be following a timeline the
    marker is not in. Read the serial for the verdict until this is fixed.
  - ADDRESSED, pending recurrence (2026-09-26): cdfs now has real exception handling
    (`ntkrnl_sys::seh`), so a `STATUS_CANT_WAIT` raise reaches `CdProcessException` and is
    retried or posted instead of bugchecking. Original report: BUGCHECK 0x1E `0xC00000D8` (`STATUS_CANT_WAIT`) during
    boot, in the WoW64 `async32` guest's overlapped `ReadFile` right after
    `[async32] start`, before appstorm launched. `RtlRaiseStatus(0xC00000D8)`, which is the
    field manual's "DUMMY_PSEH defeats cdfs's raise->catch->post async model" class: a
    non-blocking read that has to wait raises, and the catch that would post it to a worker
    never runs. The `[RAISE]` dump held only stack residue, so the raiser (cdfs or Cc) is
    unconfirmed. Run `/tmp/vibeos-cli/20260926-110052-gc3j9quv-smoke-amd64`. Not seen in any
    other serial log on disk.
  - FIXED, harness (a different miss): an amd64 appstorm sweep with 38/38 PASS failed "output
    lacks `[APPSTORM] done VERDICT=PASS`", and the tail it quoted was `[APPSTORM] done VER`.
    `drain_for_marker` returned as soon as the launch marker (a PREFIX of the verdict line)
    appeared, even mid-line. It now finishes the line, bounded at 10s.
  - FIXED: the pushlock `ASSERT FAILED: (*WaitBlock).Signaled != 0` (pushlock.rs 495/643, on
    UP i386 too) was a false positive: the waker's DBG-only `Signaled = 1` was compiled out
    while the ASSERT was live. appstorm i386 38/38 with 0 assertions (was 4).
  - FIXED: amd64 app sweeps were starved, not failing. `Wow64TraceSyscalls` was still 1, and
    every amd64 boot runs user32_winetest under WoW64, so the serial reached ~40MB and apps
    missed their window timeouts (then the run was reaped). Off: serial 2.2MB, 17 min, 36/38.
  - NOT REPRODUCING (2026-09-26), amd64: `sysdm.cpl` and `powercfg.cpl` open but did not close
    (appstorm close=TIMEOUT). Both closed in three sweeps after the WoW64 trace went off and master
    was merged. appstorm now dumps every thread of a process that ignores WM_CLOSE, so a return
    brings its stacks.
  - FIXED: the amd64 print spooler never started. localspl failed `_OpenEnvironment` with 1805,
    because the `Windows x64` environment is in `[AddReg.NTamd64]`, which mkhive (the livecd hive
    builder) never applied. Under dummy PSEH, every winspool RPC to the dead spooler then ended in
    `RpcRaiseException` → `ExitProcess(1)`, so wordpad died before its window and winver died on
    close. mkhive now takes `-a:<arch>`, like setup. arm64 got a `Windows ARM64` environment and a
    winprint destination as well, but its ramdisk does not ship spoolsv yet.
    amd64 appstorm: 38/38.
  - FIXED: amd64 BUGCHECK 0x1E in `REGION_bOffsetRgn` from `co_WinPosSetWindowPos`, after
    ~8,000 `failed to free. pvAlloc=0` lines (2 of 3 appstorm sweeps once `--repeat` soaked the
    payload). A GDI handle leak: a census at the first failed insert showed all 65,536 entries
    live, ~63,000 of them REGIONS owned by `osk.exe`, the WoW64 i386 osk winlogon starts and
    leaves running. Its gdi32 deletes regions through the TEB32 GDI batch, which
    `NtGdiFlushUserBatch` reset without executing, so none was ever deleted; once the table
    filled, every Create* returned NULL. The NULL frees were only pens failing to insert.
    Fixed by executing the guest batch (see the WoW64 GDI-batch entry). `GDIOBJ_hInsertObject`
    now says `Could not get a free entry` as the C did, and dumps `[GDI-CENSUS]` by type and
    owner once. Harness side note: appstorm's own `osk` FAILs with `windows=0` because
    winlogon's instance holds the single-instance mutex, not because osk is broken.
  - FIXED: dxdiag's heap corruption (`RtlpRemoveFreeBlock`, amd64). kernel32-rust `GlobalSize`
    ran its translated `_SEH2_EXCEPT` arm unconditionally, so GlobalSize/LocalSize returned 0
    for EVERY block on every arch. The edit control sizes its undo buffer from
    `LocalSize()/2 - 1`, which wrapped to a 0-byte allocation it then wrote into. Found with
    `APPSTORM_PAGEHEAP=1`, which faulted at the write (`EDIT_WM_Create`, `*es->undo_text = 0`).
  - OPEN, page heap: under `APPSTORM_PAGEHEAP=1` wordpad faults in comctl32
    `TOOLBAR_AddBitmap` storing through the NULL that `ReAlloc` (LocalReAlloc) returned. Without
    page heap wordpad passes, so the suspect is page-heap allocation failing (it spends >= 2
    pages per block), met by comctl32 not checking ReAlloc. Also: upstream's page heap never
    implemented its normal-heap fallbacks (`FIXME` + `ASSERT(FALSE)`); the translation dropped
    those ASSERTs, so hitting one returns NULL silently instead of stopping.
  - OPEN, SMP lane (2026-09-25, handed off): `tools/payloads/rpmprobe` has a race phase (per CPU,
    one thread copies across a page boundary with Read/WriteProcessMemory while another
    commits/decommits that page). Run with `VIBEOS_QEMU_SMP=N ./vibeos-cli smoke --arch X
    --payload rpmprobe` and read `[RPMPROBE] ... race` in the serial.
    i386 `-smp 4`: PASS (46,273 copies ok, 686,336 clean errors, 0 silent, no bugcheck).
    arm64 `-smp 2`: the 2026-09-25 race-phase wedge is GONE (re-run 2026-10-04: PASS, 84,671
    copies, 0 silent; presumably fixed by the 09-26 KNODE/affinity/spinlock work). arm64
    `-smp 4`: PASS 3 of 4 (59,153 / 63,700 / 902 copies, 0 silent). The fourth died BEFORE the
    probe ran, in first-boot setup: `BUGCHECK 0x50` on `0xfffffa800041d560`, faulting in
    `CmpFindSubKeyByName` entry (`cmlib-rust/src/cmindex.rs:1201`) under setupapi's install
    work, i.e. a key-node pointer that was no longer mapped. OPEN, 1 in 8 arm64 `-smp 4`
    boots that ran past setup, never seen at `-smp 1` or `-smp 2` in today's runs; the shape
    fits a registry-lock hole (a cell pointer held across another CPU's bin change), not yet
    shown. 2026-10-05: the hive cell-map race (HvpAddBin freeing a `BlockList` other CPUs
    still index) was found and fixed under `-smp 2` and matches this shape; not re-run at
    `-smp 4`, so this stays OPEN until it is. Artifact: `~/vibeos-flake-farm/fails/smp-arm64-cm0x50-20261004/`.
    The `ke/ipi.rs` `== SYNCH_LEVEL` assert was already re-contracted to `>= DISPATCH_LEVEL`
    and no longer prints.
    amd64 `-smp 2` and `-smp 4`: FIXED 2026-10-04. The boot death seen 2026-09-25 was
    intermittent (2/8 and 9/16 `-smp 2` boots reached the desktop), with five unrelated-looking
    faces: csrss `0xC000001D` twice at the same basesrv address, `0x6B`/`0xC0000032`, a kernel
    `0x1E` at a garbage PC, `0x4C`/`STATUS_INVALID_IMAGE_FORMAT`, and double faults in the
    unwinder. Cause: amd64's lazy-idle TLB let a CPU woken from `hlt` by a DEVICE interrupt
    run the ISR with deferred shootdowns unserviced; the IDE ISR wrote PIO sector data through
    a stale translation of a recycled system VA, so successful reads left their pages zero
    and some other page took the data. `KiInterruptDispatch` now services the deferral first.
    After: 8/8 `-smp 2` boots to desktop with zero corrupt reads; rpmprobe passes the race at
    `-smp 2` (84,240 copies) and `-smp 4` (76,114), 0 silent, where `-smp 4` had hung. Field
    manual: "Successful reads that leave their pages ZERO".
  - FIXED: `ReadProcessMemory` over an uncommitted page bugchecked (0x1E from
    `MmProbeAndLockPages` in `MiDoMappedCopy`). See the `MmCopyVirtualMemory` entry below.
  - FIXED: `SystemProcessorPerformanceInformation` reported no idle time. The ex-rust helpers
    that read the idle thread's KernelTime/UserTime were placeholders returning 0. They now read
    it, and cpuprof on i386 shows 10000ms idle per 10s once the PnP burst ends.
  - Small: Date/Time's analog clock is not drawn; Add/Remove Programs' info pane draws text as
    a column; the Properties size line loses its closing parenthesis; comprobe hangs after
    closing the sheet (not yet shown to be a product bug).
  - Kernel `ASSERT FAILED: (*WaitBlock).Signaled != 0` at `ex-rust/src/pushlock.rs:643`,
    seen once during an i386 sweep.
- **Retire the other modules' printf/varargs C shims the way shell32's went. USER-MODE SIX
  DONE 2026-09-25; the rest OWED.** desk, msgina, sysdm, netcfgx, userenv and syssetup compile no
  shim now. The strsafe printf family is `strsafe_rust::StringCchPrintfW!`/`StringCbPrintfW!`/
  `StringCbPrintfExW!`, which CALL msvcrt's variadic `_snwprintf`. The modules' own variadics
  (sysdm `ResourceMessageBox`, syssetup's exported `pSetupDebugPrint` and `FatalError`) RECEIVE
  their arguments the va-slots way. userenv's two data globals are `#[no_mangle]` Rust statics
  now; its shim's `StringCchPrintfW` also passed `cchDest` where strsafe passes `cchDest - 1`.
  Graded by `strsafe-difftest` against the genuine strsafe.h: 59,853 cases over every size, flag
  and NULL out-param, and it catches a planted off-by-one in either the bound or the remaining
  count. **Still owed**, each its own class:
  - variadic RECEIVERS: aclui `LoadAndFormatString`, wldap32 `ber_printf`/`ber_scanf`, dhcpcsvc,
    msvcrt's 30 CRT shells;
  - the kernel drivers' debug-print shims (pcix, pciidex, uniata, scsiport, storport, usbhub,
    usbport, usbstor, acpi). These cannot call msvcrt, so they would be va-slots receivers;
  - usersrv's `usersrv_rust_strsafe_shim.c`.
- **arm64 `test_start_menu_opens` on fresh media: bare wallpaper, no taskbar. FIXED
  2026-09-23 (`5e2fb0703c2`); re-verified 2026-09-24 on fresh arm64 media: the start menu
  opens over the desktop with the taskbar and tray drawn. Two bugs, both needed.**
  Arm64 winlogon spawns `cmd.exe` while explorer starts, and cmd's console connect wedged
  csrss:
  1. **No console font could exist.** win32k never enumerated `\SystemRoot\Fonts`. It
     loaded a hardcoded list instead: 6 fonts on arm64, and nothing on x86 ("deferred to
     winlogon", which added 8 by name). Neither list held consrv's default face "VGA"
     (`UniVGA16.ttf`) or its "Terminal" fallback. So `InitFonts` failed, `WM_NCCREATE`
     failed, and `GuiInitFrontEnd` fell into `GuiDeinitFrontEnd`. That waits forever on
     `hGuiTermEvent`, because the input thread skips `NtSetEvent` when `hWindow` is NULL.
     The API thread serving cmd's connect was left parked there for good. Stacks taken
     via the census:
     - tid 148: `GuiDeinitFrontEnd` ← `GuiInitFrontEnd` ← `ConSrvConnect` ←
       `CsrApiRequestThread`.
     - The input thread: idle in `NtUserGetMessage`.
     Fix: upstream's directory enumeration is restored in `IntLoadSystemFonts` on all
     arches (enumerating once and filtering the extension, because our FAT driver ignores
     the search pattern). winlogon's font list is gone, and arm64 stages `UniVGA16.ttf` and
     `lucon.ttf`.
  2. **`KAPC_STATE_ACTUAL_LENGTH` was `size_of` (48/24), not the unpadded 43/23**
     (`ke/apc.rs`, since the Feb KE translation). Every attach/detach copied `ApcState`'s
     tail over `SavedApcState`'s, i.e. into `FreezeCount` (and on 64-bit `SuspendCount`).
     Any thread that ever attached to another process, which includes every thread that
     created one, then read as suspended. win32k refused all sends to it with "Sending to
     Suspended Thread". This is the same predicate as the i386 hang behind the desktop-thread
     cursor-callback hack (Tier 1 above).
  - **Measured:**
    - Both fixes: arm64 gate 7/7 with `cmd.exe` present; 0 suspended sends; cmd's
      `PROCESS_ATTACH` completes.
    - Fonts only: 2 passes, then 1 boot where explorer never started.
    - amd64: passes.
  - **Exposed, then FIXED (i8042prt):** fix 2 lets win32k's RIT really block in the
    WH_KEYBOARD_LL hook send, so keystrokes queued up. That exposed an upstream race in
    `i8042KbdDpcRoutine`:
    - It hands kbdclass `KeyboardBuffer[0..n]` outside the interrupt lock, then only does
      `KeysInBuffer -= transferred`.
    - A key the ISR appended during the callback sits behind a stale slot 0. The next DPC
      re-delivered the old key and lost the new one: one Esc make twice, its break never
      delivered.
    - Ctrl+Esc then toggled the start menu open and shut. i386 went from 4/4 on master to
      about 1/3 with fix 2.
    The DPC now compacts the tail as the mouse DPC always did. i386 is 8/8; amd64 3/3.
  - **Not caused by these fixes:** the arm64 PC/LR/FP=0 fetch fault (`ESR=0x86000007`,
    during services' `ScmStartUserModeService`), which is the registered smp-bringup flake.
    A/B under 5-way parallel load: 9/10 boots with fix 2, 9/10 with it reverted, one
    PC=0 fault in each. That is well above the registered 1.7%.
  - **Correction:** an earlier version of this entry said winsrv's `DbgPrint`s never reach
    serial. That was false. It was read from a stale `qemu/arm64_smoke_live.log` dated
    2026-08-06. In a worktree, pytest's serial lives under `/tmp/pytest-of-*/`. The kmtest
    and stale-payload inferences above it were refuted as well.
- **shell32's DllRegisterServer returns 0x80070002 (ERROR_FILE_NOT_FOUND) on the livecd, all
  three arches. UNOWNED** (2026-09-23). Measured by `tools/payloads/regdiff`, identically for
  the C++ build (`bf5c0d1573f`) and the Rust one, so it predates retiring shell32.cpp. Every
  object-map script and the typelib registered (the dump holds `HKCR\TypeLib\{50A7E9B0-…}`),
  and registration stops at the first failure, so the failing step is one of the three extra
  scripts (IDR_FOLDEROPTIONS / IDR_EXPLORER / IDR_SYSTEMFILEASSOC) or
  `SHELL_RegisterShellFolders` — the latter is the likelier, on a read-only system drive. No
  boot notices: the media's hives are prebuilt. Narrow it by printing the HRESULT after each
  step of `DllRegisterServer` in `shell32/registration.rs` under the regdiff payload.
- **amd64 payload gate misses a launch marker that IS in its own serial log. UNOWNED**
  (2026-09-23). `smoke --arch amd64 --payload regdiff` failed twice with "'[REGDIFF] done'
  never appeared" after draining its full budget, while `payload/amd64-uefi_serial.log` from
  the SAME run holds that line (line 1820, after the last `shell ready` at 1285). So the
  bytes reached the log file but not `vm.serial_output()`, which the test reads; i386 and
  arm64 see the same payload's marker fine. The log also carries three `shell ready` lines —
  check whether the fixture rebooted and the reader stayed attached to a dead stream.
  Evidence: `/tmp/vibeos-cli/*-smoke-amd64/payload/{pytest.log,amd64-uefi_serial.log}`.

- **`afd` BUGCHECK 0x1E on a pending receive: `FCB->ReceiveIrp.InFlightRequest != Irp`.
  OPEN** (2026-09-21). Found by the new apitest harness on its third boot, running
  `afd_apitest send`. Signature: `*** ASSERT FAILED: FCB->ReceiveIrp.InFlightRequest == Irp
  at read.rs:662`, then `BUGCHECK 0x0000001E (0xc0000005, ...)` with
  `TrapFrame.Rip = afd.sys+0x89C6`. **The translation is FAITHFUL** — `read.c:279` does the
  same `ASSERT` followed by the same unconditional `InFlightRequest = NULL`, in both
  `AfdConnectedSocketReadComplete` and its datagram sibling at `:558`. So the defect is not
  that shape; the defect is that the invariant is FALSE by the time we get here, i.e. the IRP
  completing is not the one recorded in flight. That is the lead: who sets
  `ReceiveIrp.InFlightRequest`, and which path completes a receive without going through it.
  (I first wrote this entry blaming read.rs's structure, having read only our side — the C
  was one grep away and says the same thing.) Reproduce:
  `tools/payloads/apitest/quarantine-amd64.txt` currently skips
  `afd_apitest.exe send`; remove that line and run the apitest payload.
  Quarantined only so the other 61 suites can be characterised, NOT because the test is wrong.

- **`advapi32_apitest RtlEncryptMemory` crash-loops `services.exe`. OPEN** (2026-09-21).
  Serial shows `[EXIT-THR] image=SERVICES.EXE` and `eventlog.exe` repeating and the guest
  never advances. Subtest-level skip in the apitest quarantine.

- **`opengl32_apitest` wedges the guest during subtest enumeration. OPEN** (2026-09-21).
  It never reaches the `Valid test names:` output, so the whole suite is quarantined rather
  than one subtest.

- **Process SPAWN and CRT TEARDOWN can wedge the whole guest. OPEN, and it probably subsumes
  several existing apitest quarantine entries** (2026-09-23). Nine quarantined subtests across
  three suites cluster on one theme rather than nine defects: `msvcrt_apitest` `system` /
  `_wsystem` (which spawn `cmd.exe`), `atexit` / `static_construct` (CRT teardown at process
  exit), and `cmd_apitest` `attrib` / `exit` / `pushd` (`cmd.exe` spawning children).
  **The discriminator that makes this a GUEST bug and not a harness one:** the sweep's own
  120-second per-subtest timeout never fires at these wedges — verified in four separate
  rounds. The sweep is an ordinary user-mode process, so if it cannot even time out and
  terminate its child, it is not running either; the box is dead, not the harness waiting.
  A few `services.exe` restarts follow the wedge and then the serial goes silent for good, and
  the wedged child emits no test output at all. Unlike the `kernel32_apitest` cluster, the
  PREDECESSOR differs every time (`_wcsupr`, `atan`, `sqrt`, `strtoul`), so this is not the
  blame-the-next-one artifact. Worth chasing as ONE investigation into process
  creation/teardown rather than as per-subtest quarantine entries — note `msvcrt` is fully
  translated, so our Rust CRT is in the frame alongside the kernel's process paths.

- **`cmd_apitest` wedges the guest — three of its six subtests, reproducibly. OPEN**
  (2026-09-23). `attrib`, `exit` and `pushd` each stop the apitest sweep dead; one boot sat
  silent for 77 minutes after `[APITEST-SWEEP] launch cmd_apitest.exe attrib` with QEMU alive
  and the serial not growing. For `pushd` the sweep's own 120s per-subtest timeout DID fire and
  terminated the child (`[EXIT-THR] image=CMD_APITEST.EXE exit_status=0xdead`, which is
  sweep.c's `TerminateProcess(.., 0xDEAD)`), and then the sweep still never printed its
  `TIMEOUT`/`exit` line — so at least that one wedges the sweep PARENT, not only the child.
  Plausible shared cause with the two quarantined `iphlpapi GetOwnerModuleFrom*Entry` stalls:
  a grandchild inheriting the pipe's write end, so the pump never sees EOF (`cmd_apitest`
  spawns `cmd.exe`; the iphlpapi tests talk to services). That is an inference from the shape,
  not observed. All five are quarantined at subtest granularity so the other ~60 suites can be
  characterised.

- **`CmUnloadKey` removes a hive from the hive-file list AFTER closing its files and
  unlinking it, reading an already-invalidated `LinkData` node. OPEN** (2026-09-22).
  `config-rust/src/cmapi.rs` has `CmpUnlockRegistry(); CmpCloseHiveFiles(CmHive);
  CmpRemoveFromHiveFileList(CmHive);` — the exact pre-fix order upstream corrected in
  `396019984c9` ("[NTOS:CONFIG] Fix CmpRemoveFromHiveFileList() management", #9564, CORE-6491),
  which is one of the 7 commits we are currently behind. Upstream's symptom is a hive that stays
  listed under `HKLM\SYSTEM\CurrentControlSet\Control\hivelist` after being unloaded, seen in
  1st-stage setup with `\Registry\Machine\USetup_Software`; the deduced `LinkName` can be read
  from an invalidated `KeyNode.Flags`, so `KEY_COMP_NAME` is misread and the constructed path is
  garbage. Note the full upstream fix is NOT just the two-line reorder: it caches the path in a
  new `CMHIVE::HiveRootPath` member (a struct layout change, Vista+ semantics) and rewrites
  `cmhvlist.c`. Found by the bug-oracle sweep while measuring how far behind upstream we are —
  the newest 7 commits already contained a live hit against our Rust.

- **`NtUserScrollDC` probes three user RECTs and then dereferences them raw — BUGCHECK 0x1E.
  FIXED 2026-09-23**, verified by boot: `user32_apitest ScrollDC` now exits **code=0**, a clean
  pass, where it previously took the machine down. Took THREE attempts, and why is the lesson:
  the first captured only the reads, the second patched the write-back of the SIBLING
  (`NtUserScrollWindowEx`) because that block matched the search text uniquely, and only the
  third reached `NtUserScrollDC`'s own store at `scrollex.rs:709`. Each time a `count == 1`
  assertion passed and I read that as "the crashing instruction changed"; the BUGCHECK PC never
  moving was the falsifying evidence. The check that settles it is about the FAULT, not the
  edit: 0 raw 16-byte stores and 3 `KmProtectedCopyMemory` calls between `NtUserScrollDC` and
  its sibling. Originally (2026-09-23). `win32k.sys+0xF8970` = `NtUserScrollDC+0x180`,
  `win32k-rust/src/scrollex.rs`; found by `user32_apitest ScrollDC`, whose name corroborates
  the symbol. Identical shape to `NtUserSetSysColors` below: `SafeProbeForRead` validates the
  RANGE and not the MAPPING (it cannot touch-test under DUMMY_PSEH), the C wraps probe AND
  copy in one `_SEH2_TRY`, and our Rust does `rcScroll = *(prcUnsafeScroll as *const RECTL)`
  outside any guard. Same fix: capture through `KmProtectedCopyMemory`
  (`win32k-rust/src/copy.rs`). **Third instance of this class, which makes it systematic
  rather than incidental** — a scanner for "`SafeProbeFor*` followed by a raw dereference of
  the probed pointer" would enumerate the rest for free, and is owed.

- **`msvcrt log10(0.0)` returns a large finite negative instead of `-Inf`. OPEN**
  (2026-09-23, found by the apitest verification sweep). Exact values, from the test's
  own output: `log10(0.0)` and `log10(-0.0)` both return `0xffe1ccf385ebc8a0`
  (approximately `-1.0e308`) where the expected bit pattern is `0xfff0000000000000`.
  Both signed zeros give the identical wrong answer, so the zero case is not being
  special-cased to infinity at all -- it looks like a finite stand-in for `-HUGE_VAL`
  rather than a sign or rounding problem. `log10.c:49` in the apitest is the assertion.
  Separate from, and simpler than, the wedge that follows it (below).

- **THE APITEST WEDGE CLUSTER: the "bounded `WaitForSingleObject` that never times out"
  root cause is FALSIFIED. The timeout DOES fire. OPEN, and back to un-attributed**
  (2026-09-24, corrected 2026-09-25).

  The previous entry here claimed the cluster was a 120s `WaitForSingleObject` failing to
  expire, and unified it with the winlogon hang as one kernel bug. **One isolated boot
  with the quarantine disarmed refuted it**, and the refutation was one command away the
  whole time:

      $ gen_suites.py --arch amd64 --only msvcrt_apitest.exe --ignore-quarantine
      $ characterize.py --arch amd64 --rounds 1 --only msvcrt_apitest.exe --ignore-quarantine
      [APITEST-SWEEP] exit    msvcrt_apitest.exe _wsystem code=0          <- PASSES
      [APITEST-SWEEP] exit    msvcrt_apitest.exe system  code=3           <- PASSES
      [APITEST-SWEEP] TIMEOUT msvcrt_apitest.exe log10 (120s) - terminated <- TIMEOUT FIRES
      [APITEST-SWEEP] done total=1 launched=1 subtests=55 failstart=0 timeouts=1

  **`log10` is the control the earlier run had and did not use.** It is the same
  `WaitForSingleObject(pi.hProcess, 120000)`, on the same object type, in the same boot,
  and it expired on schedule. A bounded wait that demonstrably times out 120s after a
  hung child cannot also be the mechanism that never times out.

  What survives from the earlier investigation, and what does not:

  - **FALSIFIED — "the wait never expires".** `log10` times out in the same boot. The
    argument was "`sweep.c` bounds every wait, so it must advance within ~145s"; that is
    true of the *waits* but the stalled span also holds `CloseHandle(wr)` and
    `CreateThread`, which are not waits and are not bounded either. The reasoning
    enumerated two candidates and there were four.
  - **FALSIFIED — "the wedge is SELF-CONTAINED".** Isolated msvcrt with every quarantine
    entry ARMED reaches `[APITEST-SWEEP] done`: 55 subtests, 3 bad, 4m49s. The wedge needs
    something the isolated run lacks — cross-suite residue, or it is a rare race. Note the
    earlier isolated boot *did* stall at `_wsystem` under the same restriction, so it is
    intermittent, which is also why one green isolated run must not un-quarantine anything.
  - **UNSUPPORTED — the unification with the winlogon hang.** That rested entirely on the
    shared never-expiring-wait mechanism. Same call and same object type is a shared
    *predicate*, not a shared cause; the winlogon entry itself asked whether it was "a
    coincidence or the link" and the answer is still unknown. They are two entries again.
  - **STANDS — the box is ALIVE while the sweep is stuck.** The serial grew 779 bytes in
    25s (eventlog, LSASS) during the stall. That measurement is independent of the wait
    hypothesis and still overturns `quarantine-amd64.txt`'s "the box is dead, not the
    harness waiting". It is one stuck process, not a dead guest.
  - **STANDS — `CreateProcessA` returned.** The `spawned` line printed in the observed
    stall, so process creation is excluded from that instance.

  **Next:** the stall is somewhere in `spawned` → `closed` → `waiting` → the wait, and
  `sweep.c` now prints all three, so the next occurrence localises itself with no new
  build. Do NOT go looking at timer insertion/expiry first: the whole chain
  (`KeWaitForSingleObject`, `KxSetTimerForThreadWait`, `KiComputeDueTime`,
  `KiInsertTimerTable`, `KiComputeTimerTableIndex`, `KiCheckForTimerExpiration`,
  `KiTimerExpiration`, `KiTimerListExpire`) was read against the C it was translated from
  and is faithful, and upstream had made **zero** commits to `ntoskrnl/ke/` or
  `ntoskrnl/ob/` **as of its 2026-09-22 tip `9dc3ca87209`**, so the bug oracle was silent
  on the wait path. That measurement went stale the same night: upstream `091855fc4f9`
  (2026-09-25, `ke/i386/thrdini.c`, NPX ownership flush) arrived in a later fetch and has
  since been ported. It does not touch the wait path, so the conclusion here stands — but
  re-run the range check rather than trusting this sentence, which is a timestamp and not
  a property. One known latent
  quirk, shared with upstream and therefore not our regression: `TIMER_TABLE_SIZE` is 512
  while `DISPATCHER_HEADER.Hand` is a `UCHAR`, so hands 256-511 truncate on store and
  `KiRemoveEntryTimer` does its bucket-empty bookkeeping against the wrong slot.

  Proximate context, offered as context and NOT as a cause: `_wsystem`'s grandchild fails
  `co_UserCreateWindowEx(ConsoleWindowClass): NCCREATE` just before the stall — but that
  message is AMBIENT (counted 11-16 times in all 42 archived serials including healthy
  ones), so its position there is suggestive only.

  **Method note, because this cost a wrong root cause in the repo:** the refuting artifact
  was a run I was already going to do, and I wrote the conclusion first. The specific trap
  was circular and is now closed in the tool: the quarantine entry that RECORDS a wedge is
  also what SUPPRESSES it, so `--only msvcrt_apitest.exe` printed `SKIP ... _wsystem` and
  completed clean — a green run that proves nothing about the bug it was launched to
  reproduce. `gen_suites.py --ignore-quarantine` (and `characterize.py --only /
  --ignore-quarantine`, which implies `--no-quarantine`) exist so that reproducing a
  quarantined wedge does not mean hand-editing the file that records it.

- **`NtGdiCreateBitmap` BUGCHECKed 0x1E in a naked `memcpy`. FIXED 2026-09-24**, verified
  by boot (`exit win32u_apitest.exe NtGdiCreateBitmap code=2` — real test failures, no
  crash). Symbolicated stack, top and bottom frames agreeing:
  `NtGdiCreateBitmap+0x1F9` → `memcpy+0xD` → `cnt_rust::cabi::raw_move+0x5C`, exception
  0xC0000005. **The site carried a comment correctly diagnosing the bug and a fix that
  did not address it**: someone restored the dropped `ProbeForRead` and left
  `UnsafeSetBitmapBits` copying `cjSize` bytes off the caller's pointer with a plain
  `RtlCopyMemory`. The probe is validate-only, so this is the field manual's "a restored
  probe does NOT re-protect the USE", written down and walked into anyway. Upstream puts
  `ProbeForRead` AND `UnsafeSetBitmapBits` in one `_SEH2_TRY` whose handler deletes the
  surface and returns NULL; both call sites now honour that, using a `-1` sentinel rather
  than 0 because a legitimate call can copy zero bytes and `NtGdiSetBitmapBits` returns
  that count to user mode. **First confirmed-live member of the 152-lead bulk-copy
  class**, and it needed BOTH remaining scanner blind spots at once (bulk copy, one frame
  down in a callee) — now closed, with a regression test.

- **`NtGdiGetBitmapBits` BUGCHECKs 0x1E and the ntoskrnl address is a RED HERRING.
  OPEN** (2026-09-24). `0x0000001E (0xffffffffc0000005, ntoskrnl.exe+0x2B6B0, ...)`,
  reproducing at the identical address across runs days apart. The quarantine recorded
  this as "the fault is in ntoskrnl rather than win32k, so it may be a different cause".
  Resolved now: `ntoskrnl.exe+0x2B6B0` = VA `0x42B6B0` (ImageBase 0x400000), which lands
  inside `NtShutdownSystem` (`0x42B6A0`–`0x42B6C3`) — **but MID-INSTRUCTION**, at the
  last byte of the `mov $0x4,%edx` at `0x42b6ac`. A PC that is not an instruction
  boundary is not a legitimate execution point, so this is a WILD CONTROL TRANSFER, not a
  fault inside `NtShutdownSystem`. The ntoskrnl symbol says where control landed, not
  where the bug is, and chasing ntoskrnl here would be chasing the landing site.
  Redirect: find the indirect call on the `NtGdiGetBitmapBits` path (a surface or DIB
  engine function pointer is the obvious candidate) and check it for NULL/garbage. Needs
  `vibeos-cli debug start amd64 --gdb`; a bare serial cannot show which call transferred.

- **`umkm_apitest SystemCall` BUGCHECKs 0x1E at PC=0 — the user-mode callback
  dispatch jumps through a NULL pointer. ROOT-CAUSE LOCATED, fix pending**
  (2026-09-23, round 2 of the verification sweep).
  **Observed:** `BUGCHECK 0x0000001E (0xffffffffc0000005, 0x0000000000000000,
  0xfffff88004613a50, 0x0)`. By this tree's convention the second argument is the
  faulting PC, so the PC is **zero** — a jump/call through a NULL pointer, not a data
  access. It fires immediately after the subtest's DLL init, with no output from the
  test itself. The third argument is a win32k address.
  **What the test does** (`modules/rostests/apitests/umkm/SystemCall.c`): on amd64 it
  issues win32k syscall `0x1000` *deliberately without user32.dll loaded*, having
  installed a vectored exception handler, and expects the resulting
  `KiUserCallbackDispatcher` attempt to raise an exception in USER mode that the VEH
  catches (`ok_eq_ulong(g_HandlerCalled, 1UL)`). So an exception here is the test's
  intent; taking it in KERNEL mode is ours.
  **The guard that should have caught it, and its hole:**
  `ntoskrnl-rust/src/ke/amd64/usercall.rs` already has a check added for precisely this
  scenario, whose comment reads "If user32.dll hasn't loaded yet,
  PEB->KernelCallbackTable is NULL and KiUserCallbackDispatcher will ACCESS_VIOLATION
  trying to call through it." It returns `STATUS_UNSUCCESSFUL` when
  `PEB->KernelCallbackTable` is NULL — but it is wrapped in `if !teb.is_null()`, so a
  NULL TEB **skips the guard entirely** and proceeds to dispatch. Separately, the
  pointer actually loaded into the trap frame is a different one:
  `CallbackTrapFrame.Rip = KeUserCallbackDispatcher`, and that symbol is defined as
  `static mut KeUserCallbackDispatcher: PVOID = null_mut()`
  (`ntoskrnl-rust/src/ke/procobj.rs:177`), populated later during init. Nothing checks
  it before it becomes `Rip`.
  **Inferred, not yet measured:** which of the two holes fires here. The discriminating
  measurement is cheap — print `KeUserCallbackDispatcher` and the TEB pointer at that
  call site on this boot. Do that before fixing, because "check the other pointer too"
  and "the guard is being skipped" have different fixes and only one of them is real.
  **Why it matters beyond this test:** a NULL `Rip` handed to `KiUserCallbackExit` is
  unrecoverable under `USE_DUMMY_PSEH`, and any win32k callback raced against user32
  load takes the same path.

- **The apitest parser scored 14 real results as broken, because wine/test.h
  PLURALISES. FIXED 2026-09-23.** `SUMMARY` required `(\d+) failures\)`, so a subtest
  with exactly one failing assertion prints `1 failure)` and matched nothing. The row
  then kept its initial `NO-END` status, its pass/fail counts were discarded, and
  `NO-END` is in the ratchet's BAD set — so an entirely ordinary
  one-assertion-failed result read as a broken subtest. Measured on the first
  verification sweep: `NO-END` 31 → 17, `FAIL-EXIT` 76 → 90, and 65,324 passing
  assertions became visible that had been thrown away. **This one would have poisoned
  the baseline**: the floor is a SET OF ASSERTION IDENTITIES, so banking it would have
  omitted 14 subtests' identities permanently and silently. `1 test executed` has the
  same trap and is covered too. The remaining 17 `NO-END` rows are genuine — they exit
  0x80 having printed no summary at all — and are a separate open question.

- **`msvcrt_apitest log10` HANGS (120s child timeout) but does not stop the sweep.**
  On the verification sweep it was launched, hit the sweep's 120s cap, was terminated,
  and the sweep carried on to 229 completed subtests. So it is a hang plus the wrong
  `log10(0)` result above, not a crasher.

- **THE SELECTION CRITERION FOR THE 0x1E UN-QUARANTINE BATCH WAS TOO COARSE**
  (2026-09-23). Commit 25c0c884afa6 un-quarantined 13 entries on the grounds that they
  were recorded as `BUGCHECK 0x0000001E`, which is what the probe-then-raw-access class
  produces. Round 1 of the verification sweep re-crashed at `powrprof_apitest.exe`
  with `0x0000001E (0xffffffffc0000047, ...)` -- stop code 0x1E, but exception code
  **0xC0000047, not 0xC0000005**. The probe-then-deref class is *always* an access
  violation (0xC0000005); 0x1E is merely KMODE_EXCEPTION_NOT_HANDLED and says nothing
  about which exception. So `powrprof` was never a candidate for that fix, and the
  quarantine file's habit of recording "BUGCHECK 0x0000001E" without the argument list
  is what made the batch look homogeneous when it is not.
  **Fix the record, not just this entry:** quarantine comments should carry the full
  bugcheck argument tuple, because the first argument is the discriminator and the stop
  code alone is a category containing everything.

- **`user32_apitest GetUserObjectInformation` triggers BUGCHECK 0xF4
  (CRITICAL_OBJECT_TERMINATION). OPEN** (2026-09-23). `P4 = ntoskrnl.exe+0x198FF0`. A
  critical process died; distinct from the ScrollDC crash above in both bugcheck class and
  faulting module, so `user32_apitest`'s failures are genuinely independent bugs rather than
  one suite-wide cause — which is why it is quarantined per-subtest and NOT as a suite.

- **`NtGdiCreateDIBSection` read `(*bmi).bmiHeader.biSize` raw after probing it — BUGCHECK
  0x1E. FIXED 2026-09-23**, verified by boot: `win32u_apitest NtGdiCreateDIBSection` now runs to
  completion (`code=29`, real test failures, no crash). The faulting instruction was
  `win32k.sys+0xFAD1A` = `+0x4A`, `movl (%r9), %r10d`, now a `KmProtectedCopyMemory` call. The
  size is also captured into a local, which stops it being re-read from user memory after
  validation. `tools/scan_probe_then_deref.py` finds this site statically at `dibobj.rs:1822`.

- **`gdi32_apitest SetSysColors` BUGCHECKs 0x1E — `NtUserSetSysColors` probes a user array
  and then reads it unguarded. FIXED 2026-09-23** (root-caused 2026-09-22).
  `win32k.sys+0xFADD8` = `NtUserSetSysColors+0xE8`, the instruction
  `movl (%rdi,%r14,4), %eax` reading `lpaElements[0]`; fault address `0`. The test's second
  call is `SetSysColors(1, NULL, crColors)` and it asserts `FALSE`/`ERROR_NOACCESS`, so the
  NULL is deliberate. `SafeProbeForRead` is validate-only by design (it cannot touch-test
  under `USE_DUMMY_PSEH`), so NULL passes the range check; the C reads the array inside the
  same `_SEH2_TRY` as the probe and catches the AV, while our Rust calls `IntSetSysColors`
  outside any guard. A new member of the probe-then-raw-read cluster documented in
  `ntkrnl-sys/src/probe.rs`, and the first found in win32k rather than ntoskrnl. Fix is the
  one `win32k-rust/src/copy.rs` already uses for `MmCopyFromCaller` — capture through
  `KmProtectedCopyMemory` — but **chunked**, since `cElements` is caller-controlled to
  `0x3FFFFFFF`. Write-up: `docs/engineering/active-lanes/upstream-sync-phase4-review-queue.md`.
  Fixed as described: a 256-element chunk loop in `NtUserSetSysColors` feeds `IntSetSysColors`
  kernel memory. **The capture is in the syscall, not the helper** — `IntSetSysColors` is also
  reachable with kernel pointers, so protecting it there would have been wrong.

- **The probe-then-raw-access class in win32k is CLOSED except two sites:
  `scan_probe_then_deref.py` reports 2 leads, both the same defect** (2026-09-26, measured on
  the merged tree). Two lanes worked this class in parallel and their counts have to be read
  together: 108 syntactic-`*p` leads were cleared across 31 files on 2026-09-23; fixing the
  scanner's blind spots then opened the bulk-copy and interprocedural half (165 leads, a
  further 26 files); the auto-fixer took its share down to 69 refusals and the hand sweep took
  the rest to 0 on its own scanner. Re-running the merged scanner over the merged tree gives
  the real number: **2**, and they are the ime atomics below. Earlier "0 leads" and
  "69 remaining" in this registry were each true of one lane's tree and are superseded by that
  measurement. All three arches build and boot to the desktop (`_tWinMain entered` +
  `shell ready`).

  **The last two, the ime atomics, are FIXED (2026-09-26).** ime's `nCntInIMEProc`
  `InterlockedIncrement`/`Decrement` on a user counter had a `_SEH2_TRY` per access in the C,
  and no protected primitive could express them: an atomic RMW routed through
  `KmProtectedCopyMemory` stops being atomic. `KmProtectedInterlockedAdd` now sits beside it —
  native SEH in `ntoskrnl/kmsafecopy.c` on amd64/arm64, and on i386 a second extable key (the
  `lock xadd`) in `ke/i386/safecopy.rs`. Its self-test twin prints
  `[KMSAFECOPY] fault-tolerant interlocked add: valid=1 landed=1 invalid_caught=1 (PASS)` on
  all three arches, and `scan_probe_then_deref` reports 0 leads.

  Two of the scanner's own numbers along the way were wrong in the direction that reads as more
  work, not less, and both are pinned by tests now: `RtlCopyMemory` was reported by both the
  direct and the interprocedural rule (281 rows for 165 defects, and 281 got quoted as the
  workload), and the `addr_of_mut!` skip ran before the taint-end check, so sites fixed with the
  standard `p = addr_of_mut!(local)` idiom kept reporting — including an already-correct
  `NtUserClipCursor`. Details and the batching hazard (`RtlCopyMemory` is safe to rewrite
  mechanically, `copy_nonoverlapping` is NOT — reversed arguments and element counts) are in the
  field manual under the probe-then-raw entry.

  Three sites were double fetches rather than merely unprotected, and are the part worth
  re-reading: `NtGdiCreateDIBSection` validated user memory with a length obtained by WALKING
  that same user memory (`DIB_BitmapInfoSize`); `NtGdiSetDIBitsToDeviceInternal` handed raw
  pixel bits to `IntGdiCreateMaskFromRLE`, which walks an RLE stream; and
  `NtGdiCreateDIBitmapInternal` forwarded a raw `pbmi` to a callee that dereferences it outside
  the probe — a hole **upstream has too**, so not a translation divergence.

  This class got worked down by TWO lanes independently, in the same week, without either
  knowing: upstream-sync's `21b349ccb63` ("70 more probe-then-raw-copy sites") and the other
  lane's `66fa9466bbc` (62 sites) overlapped heavily and collided as a merge conflict in
  `message.rs`/`stretchblt.rs`/`winpos.rs`, where both had written the same `capture_from_user`
  call with different line wrapping. Resolved in master's favour at every conflicting site,
  then re-scanned to prove nothing unique was dropped. **The duplicated effort is the lesson,
  not the merge:** a scanner that prints a lead list is an invitation for two lanes to work
  it at once, and nothing in the list says who holds it. The three BUGCHECKs above were NOT
  duplicated -- `NtUserFlashWindowEx`, `NtUserGetKeyboardLayoutName` and the two
  `NtUserCallOneParam` aliases were untouched on master -- which is the useful signal: the
  reproducer-driven sites and the list-driven sites barely intersect.
  The mechanization (`tools/fix_probe_then_deref.py`):
  `tools/fix_probe_then_deref.py` is the counterpart to the scanner. It never invents an
  error path -- it replays the probe's OWN failure block, which is the only reason a rewrite
  of 62 kernel syscall entry points is safe to automate at all: no new semantics are
  introduced anywhere. What it REFUSES is the part worth reading, and each guard is a wrong
  rewrite it emitted first (three of the four BUILD CLEANLY and are silently wrong on the
  error path):
  the positive form `if NT_SUCCESS(s) { <copy> }` has no failure block -- its body IS the
  copy; a failure block belonging to an EARLIER probe is the wrong error path, not a smaller
  one; a block whose transfer predates work the site now depends on (the tell is a reassigned
  status or an acquired resource, NOT line distance -- distance refused one out-parameter of
  `NtUserGetLayeredWindowAttributes` while fixing the two above it); and a multi-line probe
  inside the condition, where the brace scan scoops the probe's arguments up as the error
  body. Pinned in `tools/test_fix_probe_then_deref.py`, one test per wrong rewrite.
  Leads the fixer refuses are listed rather than silently dropped; each needs a human to
  decide what its cleanup does, and at most of them the answer is
  `Status = STATUS_ACCESS_VIOLATION` plus fall-through.

- **winlogon stops dead after a guest exits, and a 60-SECOND WaitForSingleObject timeout
  never fires. OPEN, and it invalidates the amd64 `--ladder` verdict** (2026-09-21).
  Signature: the kernel prints `[EXIT-THR] image=marshal32.exe exit_status=0x600d` — the
  guest PASSED — but winlogon's own `[WINLOGON] marshal32.exe exit = 0x600d` never
  appears, no later guest is launched, and `ladder.check` then reports the eight
  remaining guests as "never reported a verdict". Measured over a 956-second boot: the
  machine stays ALIVE the whole time (the netstack self-tests keep running and logging
  afterwards), so this is winlogon wedged, not the VM dead or starved.
  **The load-bearing detail is the timeout.** `TryStartWow64Marshal` waits
  `WaitForSingleObject(Pi.hProcess, 60_000)`; more than 500 seconds elapsed after the
  guest exited and it never returned, so the anomaly is not "the process did not signal"
  — it is a bounded wait that does not time out. Alongside it, something sends
  `WM_ENTERIDLE` to a thread `IsThreadSuspended` calls suspended, about 2.4 times a
  second forever (1202 sends in the tail); if that thread is winlogon's, "suspended" is
  the truth and the question becomes who suspended it.
  Same predicate as the console hang above, which is either a coincidence or the link.
  Diagnosis notes so the next attempt does not re-walk this: it is NOT host starvation
  (three earlier boots were misread that way — by rate all boots run 27-39 serial
  lines/s, and doubling the drain from 470s to 956s bought 670 more lines, not more
  progress), and it is NOT a tight spin (1202 sends in ~500s is idle, not CPU-bound).
  A boot at 2026-09-21 00:45 ran the same guests 11/11, so it is recent; the amd64 media
  changed only by winlogon.exe between the two, which is suspicious but unproven —
  nothing in that diff runs before marshal32.
  **Rate (2026-10-04):** 1 miss in 25 queue-batch gates (04:11-15:09, `smoke-amd64-a1`
  serial, `[WINLOGON] marshal32.exe exit =` present or not). The miss (gate
  `20261004-145947-0qwjkolk`) held master as LADDER_FAIL beside a real DIFFTEST_FAIL; a
  lone LADDER_FAIL with this signature is this entry, not the batch under test.

- **Wine's conformance suites now run under WoW64, and the first two subtests found
  four separate bugs. OPEN** (2026-09-21). `user32:wsprintf` passes clean (46 tests, 0
  failures). `user32:class` is the interesting one, and the native i386 A/B run of the
  SAME binary splits its failures three ways:
  - **Ours, not WoW64** (identical on bare i386, same source lines): `GetModuleFileName
    failed` (class.c:1286/1299), `IME window proc implemented in` (1290/1303),
    `CreateDialogParam returned NULL` (919).
  - **WoW64, FIXED**: all eight `GetClassLongW value doesn't match what was set`
    (class.c:164) — class extra bytes read off the WoW64 snapshot instead of the class.
  - **UNATTRIBUTED, and the tempting answer is wrong**: the wndproc-identity cluster
    (class.c:700-784, `Wrong ANSI wndproc: FFFF02BE vs 77A4A570`, where `FFFF02xx` is a
    winproc HANDLE where a real address belongs). It does not appear in the native run —
    but the native run HANGS in the child-process section before reaching line 700, and
    the WoW64 run CRASHES 0xC0000005 before reaching the child-process section. The two
    never overlap on that block, so "absent" means "not reached". A first theory that the
    SERVERINFO `apfnClientA/W` arrays are misread under WoW64 is DISPROVEN by the log:
    the values are valid *guest* user32 addresses (77A4A570 = the guest's DefWindowProcA),
    so the widths are fine and the A/W decision is what is wrong.
  Unblocking it needs the console hang below fixed first — that is what stops the native
  control from ever reaching line 700.

- **A winetest's child processes each hang for 30s on bare i386. OPEN** (2026-09-21).
  `class.c:1252` spawns `<exe> class <classname>` fifteen times with
  `bInheritHandles=FALSE` and waits; every wait times out
  (`winetest_wait_child_process`, 30s each) and is scored as one failure. Reached only by
  the NATIVE half of the winetest A/B, so it is not a WoW64 bug. Note the child gets no
  console and no inherited std handles, and winlogon launches the parent
  `DETACHED_PROCESS` — so before blaming CreateProcess, re-run the native control WITHOUT
  DETACHED_PROCESS (a console is legal there; it is only the WoW64 guest that cannot
  create one). That is a one-line experiment and it also A/Bs the console bug below.

- **Console creation HANGS a CUI process launched by winlogon — FIXED 2026-09-23.** Five
  bugs, the last three found only once console creation began to work:
  1. win32k never loaded consrv's "VGA" font.
  2. `KAPC_STATE_ACTUAL_LENGTH` made attached threads look suspended.
  3. consrv's handle table was indexed through winsrv-sys's opaque 1-byte
     `_CONSOLE_IO_HANDLE`, so `.add(i)` stepped one byte and entries overlapped. csrss
     died on the first console close: AV in `ConSrvCloseHandle`, BUGCHECK 0xF4 on i386.
  4. 22 of `GuiVtbl`'s 25 slots, plus `GuiConsoleInputThread`, were Rust-ABI fns
     transmuted into NTAPI slots, under a comment claiming they were `extern "system"`.
     Six of them returned u8 for C's BOOL.
  5. The same ABI shape was in msgina's dialog procs and advapi32's crypt callbacks.
  Two new gates, `scan_opaque_ptr_arith` and `scan_fnptr_abi_transmute`, keep 3–5 at zero.
  The i386 native user32 `class` winetest now runs end to end with a real console: winlogon
  no longer detaches it, and since the follow-ups below neither is the WoW64 guest.
  Follow-ups, 2026-09-23:
  - **WoW64 guest console (WORKS).** On amd64 the guest `user32:wsprintf` runs with a
    console: 58 tests, 0 failures, the same as the native i386 control. The ladder stays green.
  - **SxS (FIXED).** `CreateActCtxW` failed about 130 times per x86 boot. rtl-actctx queried
    `FileEndOfFileInformation`, which is a SET-only class on NT (upstream ReactOS has the same
    bug; Wine allows the query). It now uses `FileStandardInformation`, and the count is 0.
  - **Class-test 0xc0150010 (ENVIRONMENTAL) — FIXED on amd64 2026-09-26.** The winetest
    wrote its manifest to `GetTempPathA` = `X:\reactos\TEMP`, read-only on the livecd. The
    harness now attaches a FAT disk holding `\TEMP` (`vm.py _temp_disk_args`, default ON for
    amd64) and winlogon points the winetest children's TEMP/TMP at `C:\TEMP` when it exists.
    Guest `user32:class` now runs to completion (32 failures) instead of crashing. Attaching
    that disk found two classpnp bugs that had hung or crashed EVERY boot with a hard disk:
    `ClassCompleteRequest` spinning on a lock the release build never initializes, and a cdecl
    `ClasspSendSynchronousCompletion` in an NTAPI slot (i386). i386 stays opt-in
    (`VIBEOS_QEMU_TEMP_DISK=1`) until the bug below is fixed.
  - **arm64 winsxs enabled.** comctl32 had no arm64 `ARCH` or manifests. Staging them
    exposed a fastfat byte-vs-u16 copy that BUGCHECKed 0x50 at smss. That bug and six more of
    its shape (cmvalche, mountmgr, iomgr) are fixed, and `scan_wide_byte_copy` gates the class.
  - **The 50 ms `KeDelayExecutionThread` in `co_IntInitializeDesktopGraphics` is REMOVED.**
    It was a March workaround for a race fixed since; 12 boots passed without it.

- **WoW64 guest user32 read 64-bit shared structures with its 32-bit layout — FIXED
  2026-09-24; guest `user32:class` now fails exactly what the native control fails.** The first real conformance
  suite under WoW64 found this. The guest `user32:class` used to die with an execute AV
  after ~1.4 KB of output. It now runs to the same ending as the native i386 control (the
  read-only-TEMP 0xc0150010) with no faults. Fixed:
  1. **`gpsi` was never normalized.** Every read after `dwSRVIFlags` (offset 0 in both
     layouts) was misplaced: system metrics, colours, brushes, icons, event-hook bits, the
     class-proc tables. `wow64.rs` now has `ServerInfo64` with 64-bit layout guards and a
     `psi` accessor module that reads it live. There is no snapshot, because several fields
     change at runtime. Every guest call site goes through it.
  2. **Host class procs reached the guest.** win32k keeps the 64-bit user32's
     `apfnClientA/W` as the system procs. Such a proc is now swapped for the guest's proc
     in the same PFNCLIENT slot wherever one crosses over: the WND/CLS/CPD normalizers, the
     WINDOWPROC callback, and GetClassInfo. The guest's procs are swapped back going in:
     RegisterClassExWOW, and new manual SetWindowLong/SetClassLong(WNDPROC) and GetCPD
     thunks (`dll/win32/wow64/wow64_pfnmap.c`).
  3. **win32k refused `SetWindowLong(GWL_WNDPROC)` from every WoW64 caller**, even through
     the Ptr entry point. It compared the op size to `sizeof(LONG)` rather than to the
     caller's pointer size (`UserCallerPtrSize()`). No 32-bit app could subclass a window.
  4. `CALLPROCDATA`, `PROPERTY` (client-side `GetProp`), and `NtUserBuildPropList`'s
     `PROPLISTITEM` had no 32-bit shape. comctl32's IP-address control crashed on the
     NULL `GetProp` returned once subclassing started working.
  Second pass, same day. It took the guest-only failures from 17 to 1:
  5. **`GETPFNSERVER` was truncated.** `aStoCidPfn` holds the 64-bit user32's `apfnClientW`,
     so `psi::pfn_server` now maps it like any other proc. Server-side windows had read
     `GWLP_WNDPROC` as 0 (class.c:731/732, ScrollBar 774/778).
  6. **The WINDOWPROC callback discarded `WM_GETTEXT` output.** It is the one
     `MMS_FLAG_WRITE` message the copy-back list lacked (719).
  7. **Icons:** the COPYIMAGE callback, `NtUserGetIconInfo` and `NtGdiExtGetObjectW` had
     no marshallers. Guest `GetIconInfo`, `CopyImage` and `GetObject` on a bitmap all failed,
     so win32k could not derive a class small icon (1190–1219).
  8. **comctl32 was loaded in every guest at startup.** Delay-load had never been on:
     rpcrt4's delay imports were ordinary ones, loading wininet → shell32 → comctl32. Now
     ON by default on i386/amd64 (the guest's DLLs are i386); see the ledger for arm64
     (1292–1346, 1632/1643). The native run's
     "comctl32 not loaded" failures (1315/1322) are shared now.
  Still OPEN, guest-only:
  - ~~`SysAnimate32` window creation (class.c:1357)~~ FIXED 2026-09-24. Animate
    `LoadLibrary`s `msvfw32.dll`, a literal inside comctl32 that the roots-only literal scan
    missed. The amd64 livecd now passes `wow64-closure.py --library-literals`: 180 modules,
    +25 MB stripped, ISO 295 → 334 MB. The guest failure list is now IDENTICAL to the native
    control's. The arm64 FEX ramdisk (fixed 140 MB) stays roots-only.
  Also fixed with it: amd64 media had staged only amd64 `winsxs` assemblies, so the guest's
  comctl32 failed `CreateActCtxW` for both of its manifests. The x86 common-controls
  assemblies are now staged beside syswow64 (`dll/win32/wow64cpu/CMakeLists.txt`).
  Repro: amd64 media with `build-fex-hello` guests and an i386 `--rostests`
  user32_winetest, then
  `smoke --arch amd64 --until "[WT] user32:class: exit" --fail-on "user32:class: CRASHED"`.

- **Stock WoW64 applets (taskmgr, magnify, osk) died at `exit_status=0x80` about one boot
  in three — FIXED 2026-09-20.** Root cause was a native-stack LEAK, not recursion and not
  the 1 MB reserve: `BTCpuSimulate` stashes the 64-bit dispatcher RSP at `CpuArea+8`, a
  user callback runs the guest through a nested `BTCpuSimulate` that overwrites it, and the
  callback exits by `NtCallbackReturn` rather than returning through the asm, so the outer
  value was never restored. Every callback then cost ~4 KB permanently; taskmgr exhausted
  its 1 MB at ~950 callbacks. Fixed by saving/restoring the slot around the nesting in
  `Wow64pUserCallbackCommon`. Measured after: `exit_status=0x80` 3 -> **0** per boot, user
  stack pre-faults 2 -> **0**, `STACK EXHAUSTED` 1 -> **0**, and taskmgr reaching 4915
  callbacks with its dispatcher RSP flat. Two earlier fixes were prerequisites for even
  seeing it — guest stacks could not grow at all (TEB32 `DeallocationStack` was never
  published), and a genuine overflow could not be REPORTED (the exception record went below
  the faulting Rsp, which on overflow is already the floor), which is what laundered every
  stack bug into an anonymous `0x80`. Field manual: "A nested BTCpuSimulate clobbers the
  saved dispatcher RSP".

- **`ExPools` on arm64 intermittently drops one assertion under load, and it can eat a
  whole merge-queue submission. UNOWNED** (2026-09-20). Signature:
  `ExPools: pass 14 -> 13 (dropped)`, the extra failure being `assert ExPools.c:181` —
  the check that a `ExAllocatePoolWithQuotaTag(PagedPool|...)` allocation comes back
  `PagedPool` rather than promoted to `NonPagedPoolMustSucceed`. The test's own comment
  says "Vista+ promotes the memory allocation", i.e. **the assertion is memory-pressure
  sensitive by construction**, and the conductor runs three arches building and booting
  at once. Frequency across saved artifacts: **8 runs at pass=13 vs 101 at pass=14** (~7%).
  It is NOT new and NOT caused by whatever lane it rejects — the same signature appears in
  four earlier batches (2026-09-19 11:57, 12:06, 19:20 and 2026-09-20 00:29) that all
  **landed green**, having been retried away.
  **THE HEURISTIC IN THE ENTRY ABOVE HAS A FAILURE MODE, and this is it.** That entry says
  a failure whose signature is IDENTICAL across attempts is by definition not a flake.
  Here all three retries produced byte-identical `pass=13 fail=5`, which reads as
  deterministic — but the load condition *persisted across the whole batch*, so the retries
  never sampled a different environment. Retry-vs-retry cannot discriminate a
  load-sensitive assertion. **The discriminators that do work** are cheap and were what
  settled it: re-run the identical merged tree somewhere quiet (gave pass=14, ratchet
  holds), and grep the artifact history for the same signature
  (`grep -rl '^ExPools OK pass=13' /tmp/vibeos-cli/*/kmtest-results.txt`) to see whether it
  predates the change. Do both before either resubmitting *or* believing a lane is at
  fault.
  RESOLVED 2026-09-20 — and it was never pool promotion. The test reads the pool header
  through its own `POOL_HEADER` in `kmtests/include/kmt_test_kernel.h`, which gated the
  64-bit layout on `_M_AMD64` while ntoskrnl's own copy (`mm/ARM3/miarm.h`) gates on
  `_WIN64`. **On ARM64 the kernel wrote the 64-bit header and the test read the 32-bit
  one** — wrong field widths, wrong offsets, wrong union member — so
  `KmtGetPoolTag`/`KmtGetPoolType` returned garbage that moved with heap layout (`PoolType`
  was 2 on a quiet box and 0x54 under load; neither is a valid POOL_TYPE). Fixed by gating
  on `_WIN64` to match the kernel. ExPools went **14/4 -> 17/1** on arm64, now identical to
  the amd64 baseline, and IoDeviceObject 180/9 -> 184/5 and ExCallback 12/1 -> 13/0 came
  with it — those three are exactly the kmtests that call the pool helpers. Baseline
  hand-edited for those three rows only; a `--write-baseline` reseed was tried first and
  REVERTED because it deletes this file's provenance comments and banks unrelated bistable
  rows from one sweep, which the baseline's own header warns about.

- **Two more load-sensitive tests that impersonate lane failures. UNOWNED** (2026-09-20).
  Same class as the entry above, logged so the next person does not re-diagnose them.
  `tests/qemu/cli/test_cli_units.py::test_hostlock_announce_pair_on_queued_acquire` drives a
  real `threading.Timer(0.8, ...)` release mid-wait and asserts WAIT+ACQUIRED both print; run
  inside a gate that is itself saturating the host, the announce never happens and it fails.
  Seen in 3 gate runs (2026-09-19 11:22, 2026-09-20 01:23 and 12:32), i.e. it predates any
  one lane. `tests/qemu/cli/test_recordreplay.py::test_trace_position_finds_the_process_reading_the_trace`
  is the other: ~4 minutes of process scanning, fine standalone, times out under parallel
  load. **Both pass standalone**, which is the cheap discriminator — run the single test
  before believing a lane broke it.

- **win32k: 32 dropped `_SEH2_EXCEPT` handlers across 11 files, in `NtUser*` syscall
  paths — FIXED 2026-09-20** (`d89e8bd99ee`, `c5877c15d97`, `b6c0dde1ca3`).
  All 32 restored with `SafeProbeForRead`/`SafeProbeForWrite` + `safe_copy_raw`
  rather than the `MmUserProbeAddress`-dependent helpers (a plain `extern static`
  for that DATA export is unsatisfiable from a separate driver, and arm64 LINK_FAILs
  on it). The scanner that found them now reports **0 files, 0 handlers**:

        python3 tools/scan_dropped_seh_pairs.py --deleted win32k-rust:win32ss/user/ntuser

  Two caveats kept deliberately. `UserGetClassName` is NOT a literal restoration:
  it writes into the caller's buffer from inside opaque RTL calls, and real SEH
  catches a fault raised inside a *callee* while `safe_copy_raw` only protects a
  copy we perform — there is no protected-CALL primitive, so it queries into a
  kernel temp and copies out. And the restored `NtUserEnumDisplaySettings` body is
  **unvalidated**: the boot never calls it (`grep -ci EnumDisplaySettings serial.log`
  → 0), so green means "nothing regressed", not "this works".
  Still open, registered here rather than silently dropped: the `lpDevMode`
  zero-length probe in `NtUserEnumDisplaySettings`, and 14 lower-confidence
  scanner pairs the tool prints as "confirm by hand".

  Original survey kept below, because the two findings in it generalise: Same class as the LPC bug closed today and
  larger. Reproduce with:

        python3 tools/scan_dropped_seh_pairs.py --deleted win32k-rust:win32ss/user/ntuser

  | file | handlers in the deleted C |
  |---|---|
  | `class.rs` | 10 |
  | `display.rs` | 6 |
  | `ime.rs`, `kbdlayout.rs` | 3 each |
  | `cursoricon.rs`, `menu.rs`, `message.rs` | 2 each |
  | `accelerator.rs`, `console.rs`, `desktop.rs`, `sysparams.rs` | 1 each |

  **Why no scanner found this before.** The ordinary pairing needs the C on disk.
  These flips DELETED `win32ss/user/ntuser/*.c` (commit `8d3c33ee41e`), so basename
  matching silently fell through to `win32ss/user/user32/...` — a different
  component — and reported nothing about the kernel side. The oracle was never
  gone, only out of the working tree: `git log --diff-filter=D` finds the deletion
  and `git show <sha>^:<path>` prints the C back. That recovery is now the
  scanner's `--deleted` mode, opt-in because it shells out to git per file.

  **`class.rs` is the worst and is a DIFFERENT defect, not just a missing handler.**
  It defines its own local `probe_read_generic<T>(ptr, default) -> T` that
  range-checks and, on failure, **returns `default`** — where the C macro RAISES.
  So an out-of-range user pointer yields a zeroed `UNICODE_STRING` that the caller
  consumes as real data: a silent wrong answer rather than an error. And when the
  range check passes it does a bare `read_volatile`, which still faults on an
  in-range-but-unmapped page. Both halves are wrong, in opposite directions.

  That local helper also **defeats name-based detection**: it collides exactly with
  `ntkrnl_sys::probe::probe_read_generic`, which has different semantics, so any
  scan keyed on the bare name scores the file as protected. It hid the highest-count
  file in the tree from this very survey until the collision was noticed. Match
  QUALIFIED paths only.

  *Was, when surveyed:* "32 sites across the kernel's user-facing surface is not
  unsupervised overnight work." It was done overnight after all, by four
  concurrent agents writing one file each with the owner's explicit go-ahead,
  batched and gated centrally. The LPC pass was the worked template —
  `LpcpMoveMessageChecked` / `LpcpVerifyMessageDataInfoChecked` show the
  checked-variant-alongside-original shape, and the handler taxonomy (returns /
  goto-cleanup / falls-through / swallows) is in the field manual.
- **The vendor registry did not see Rust crates — 58 of them, including the TCP/IP
  stack. DONE** (found and fixed 2026-09-20, the finding prompted by the owner asking
  whether smoltcp was on the vendor list; it was not). `vendor.yaml` models vendored **C trees**, and its
  question — "is this C ours to translate?" — is meaningless for a crate, because crates
  are the translation DESTINATION. But the registry's other job, provenance and
  licensing for what we ship, applies to `smoltcp` exactly as much as to zlib, and that
  job quietly went unscoped. `THIRD-PARTY-NOTICES.md` is a shipped legal artifact with
  58 crates' worth of nothing in it.
  **`--suspects` cannot find this either**: it walks directories for foreign copyright
  headers, and crates.io dependencies live in `~/.cargo/registry`, outside the repo. So
  both halves of the registry are blind to the same category, which is why nothing
  surfaced it.
  Shipped and unrecorded, from `cargo metadata`: **smoltcp 0.13.1 (0BSD)** behind
  `netstack.sys`, **virtio-drivers 0.13.0 (MIT)**, **iced-x86 1.21.0 (MIT)**,
  **zlib-rs + libz-rs-sys 0.6.3 (Zlib)**, `managed` (0BSD), plus heapless / defmt /
  zerocopy / safe-mmio and the rest. All 58 are permissive — 33 `MIT OR Apache-2.0`, 11
  `MIT`, 3 `Unlicense OR MIT`, 3 `MIT/Apache-2.0`, 2 `Zlib`, 2 `0BSD`, 2 `BSD-2-Clause
  OR Apache-2.0 OR MIT`, 1 `Apache-2.0 OR MIT`, 1 `(MIT OR Apache-2.0) AND Unicode-3.0`
  — so this is a recording gap, not a licence problem.
  FIXED: `tools/rust-crates.yaml`, generated by `tools/vendored.py --write-crates` and
  gated by `check` against `sdk/lib/rust/Cargo.lock` — against the lockfile and not
  live `cargo metadata`, so `check` still needs no cargo and no registry cache.
  18 of the 58 ship; the notices file gained a Rust crates table.
  **The shipped/host split is best-effort and the file says so**, because cargo's graph
  cannot settle it here: `netstack-cargo` does not depend on `netstack-rust` in cargo
  terms at all — it pulls only panic-common and the link helper, and CMake links the
  real crate in afterwards, so walking edges from the 34 crates `add_cargo_module`
  builds reaches ZERO external crates. Only the negative is provable (proc-macros and
  crates reachable solely from harnesses or build helpers), so `ships` is false only
  when certain and true otherwise — over-listing costs a line, under-listing is the
  failure that matters.
  ALSO SURFACED BY THIS: we now ship **two zlibs** — the vendored C one (1.3.2) and
  `zlib-rs` 0.6.3 via `libz-rs-sys`. Possibly deliberate, never recorded.
- **Mbed TLS is five years past end-of-life, and three other vendored trees are stale.
  UNOWNED** (2026-09-20). `tools/vendored.py --emit` prints the full BEHIND UPSTREAM
  section; read that, not this line, because it is generated from `vendor.yaml` and this
  is not. The headline: **ours is 2.16.11, and the 2.16 LTS branch stopped receiving
  upstream security fixes at the end of 2021** — in the library behind `bcrypt` and
  `schannel`. Sized so it is not a vague someday: two consumers, 77 distinct `mbedtls_*`
  symbols, and a 2.x -> 3.x migration (the `*_ret` hash suffixes are gone, fields moved
  behind `MBEDTLS_PRIVATE`, several `ssl_conf` entry points changed shape). ~~**The trap:
  schannel LAZY-LOADS mbedtls, so a missed rename fails at RUNTIME, not at link time**~~
  — DISPROVEN 2026-09-20. That lazy-load header is inside `#ifndef __REACTOS__`; our
  CMakeLists does `add_importlibs(schannel mbedtls ...)` and `objdump -x` on the built
  DLL shows `mbedtls.dll` in the import table, so every symbol break is a LINK error.
  The real cost driver is different: the glue includes `mbedtls/md_internal.h` and
  `mbedtls/ssl_internal.h` and reads ten struct fields that 3.x puts behind
  `MBEDTLS_PRIVATE`. **"There is no TLS test in this tree" WAS true and is now fixed**
  — `sdk/lib/rust/schannel-tls-difftest` drives a real TLS 1.2 handshake through the
  genuine backend and runs in the `vibeos-cli difftest` gate.
  Also open: **libtirpc 0.1.11 -> 1.3.8** (largest version gap in the registry) and
  **FreeType 2.10.0 -> 2.14.3**. FreeType is NOT exposed to the famous one
  (CVE-2020-15999, the sbix embedded-PNG overflow exploited in the wild) — checked in the
  tree, not inferred: `FT_CONFIG_OPTION_USE_PNG` is undefined, so `pngshim.c` compiles to
  `typedef int _pngshim_dummy;`. The remaining CVEs are in the sfnt/cff/type1 parsers we
  DO build, so it is wanted but not urgent. libtiff (4.1.0 -> 4.7.0), libpng
  (1.6.39 -> 1.6.58) and the zlib audit are DONE.

- **A deterministic failure was classified `CONFORMANCE_FLAKE`, and the wrong LABEL cost
  more than the retries. LABEL FIXED 2026-09-26; the discriminator is still OPEN** (2026-09-19).
  Renamed `CONFORMANCE_RED` (owner-approved) — it names the observation, not a cause. It
  recurred the day of the rename: `rustconv-speedrun` @ `e50515abfab5` was rejected
  "CONFORMANCE_FLAKE, 3 attempts" for an i386 BUGCHECK 0x1E identical on all 9 surviving
  rounds (an un-gated empty stub shadowing a self-patching asm routine). What is still owed is
  the signature diff below: the final verdict still cannot say "reproduced identically". Verbatim from `land-status`:

        REJECTED: shell32-com @ 3607b8d766e0 (CONFORMANCE_FLAKE)
                  conformance:i386: CONFORMANCE_FLAKE, 3 attempts

  It was not a flake. It reproduced **9/9** — three arches x three rounds — and again on the
  lane's own machine first try; root cause was 58 `#[no_mangle]` names capturing live C++
  symbols (entry above). It blocked the merge queue for **75 minutes**.
  **The retry count is the symptom, not the defect.** `--retries 2` was behaving correctly;
  it was obeying a classifier that had already decided the failure was nondeterministic. So
  raising or lowering the retry number fixes nothing. **A failure whose signature is IDENTICAL
  across attempts is by definition not a flake, and the harness holds both attempts already** —
  retry once, diff the signature, and bail as deterministic when it matches. That costs 2x
  instead of 3x *and* produces a truthful verdict. Compare something stabler than elapsed
  time; `launched: 0 / sweep_rc: 1` is the shape that works here.
  **The label is the expensive part**, and this is the reason to fix it: `CONFORMANCE_FLAKE`
  is precisely the verdict that invites a blind resubmit. The lane's own read of "flake, 3
  attempts" would reasonably have been "unlucky, try again" — another three rounds of blocked
  fleet before suspecting its own tree. It only went the other way because someone hand-
  delivered the artifacts. A classification is what a lane reads to decide whether to
  investigate or requeue, so a wrong one buys a second outage.

- **WIRING a translated TU is already a partial cohab flip, and the linker is silent about
  it. UNOWNED (gap), lane-owned (instance)** (2026-09-19). `shell32/utils.rs` defines **58**
  `#[no_mangle]` names that the LIVE C++ shell32 objects also define as unmangled externs —
  including 16 `*_RustBridge` wrappers other Rust modules call, plus
  `SHGetShellStyleHInstance` / `SHGetUserDisplayName`, both on the taskbar `OnCreate` path
  that froze. **No duplicate-symbol error was produced**: lld resolves archive members
  against object files in a defined order and simply picks one, so 58 live entry points
  silently rerouted into Rust that had never executed. Bisect confirmed the mechanism rather
  than a bad body — un-wiring `utils` alone cleared the taskbar hang and exposed a SECOND
  independent blocker in the other wired module, which is what symbol capture predicts and a
  single bad function does not.
  **`scan_duplicate_symbols` cannot catch this and says so in its own docstring**: it
  ratchets Rust `#[no_mangle]` names defined in more than one *kernel-linked crate* —
  Rust-vs-Rust. The dangerous direction is **Rust shadowing C++ across the cohab boundary**,
  where the linker is silent by design. The check is cheap and belongs in `check`:
  `llvm-nm --defined-only` over the module's C++ objects, keep `$2=="T"`, drop `^__Z`, strip
  a leading `_` and any `@N`, intersect with the crate's `#[no_mangle]` set. **A non-empty
  intersection means wiring and flipping are ONE change** and must be gated as one.
  **Confirmed by bisect, and the instance is closed**: tip froze in `OnCreate` (launched=0);
  `utils` un-wired moved the freeze to after `shell window ready` (still launched=0); BOTH
  un-wired ran full startup to `kmtest-sweep.exe`, **launched=91 ok=88, ratchet holds** —
  identical to a same-host control. One blocker per wired module. Every compile repair and
  real bug the work found is innocent and stays (the `NM_*` catch-all that had made
  CDefView's whole `OnNotify` dead, a cdecl/stdcall mismatch, the substrate consolidation).
  Note *why* it happened: the deleted banner read "ORPHAN-BY-DESIGN: the module is not wired
  into the crate **yet**", **a guard that reads like a to-do item** to anyone holding "wired"
  as the goal. Reworded to open "DO NOT WIRE THIS MODULE. That is the invariant this file is
  held to, not a status line", and to name the old phrasing so the next reader sees their own
  reading anticipated rather than a bare prohibition.

  **FOLLOW-UP, unowned and the real fix: an orphan module is NOT COMPILED, so the guard buys
  correctness and pays in rot.** That is precisely how CDefView reached 799 errors before
  anyone looked. The banners now carry a manual re-verify recipe (wire temporarily, `cargo
  check` all three targets for both `shell32-rust` and `menuband-rust` — which `#[path]`
  -includes `types.rs`/`shellcom.rs` — then un-wire before building or landing, with a
  last-verified date), and that is a weak manual ratchet by construction. The mechanism that
  would actually close it is a **check-only cfg/feature that COMPILES orphans without LINKING
  them**, so `vibeos-cli check` covers them continuously and the staticlib never sees their
  symbols. Deliberately not invented inside the change repairing the incident.

- **`vibeos-cli conformance` exits 0 on media that launched ZERO tests. UNOWNED**
  (2026-09-19). Verified in an artifact: `launched: 0`, `sweep_rc: 1`, and every baseline row
  reported as the literal string **`in baseline (OK) but never launched`**. A test that never
  ran is scored **OK**, so zero launches means zero regressions means
  `ratchet holds: no regressions vs baseline`, exit 0. Two guards exist and neither covers
  this: the run is **report-only unless `--gate`** (`conformance.py:8-9,900`), and the
  zero-launch abort needs **three CONSECUTIVE** no-launch rounds (`NOLAUNCH_ABORT = 3`), which
  a single hand run can never reach. In the merge queue it does fail — correctly — so the
  consequence is narrow but nasty: **anyone reproducing a queue failure by hand gets a green
  shell and a red reality**, which is exactly backwards for the one workflow where you are
  trying to tell whether you fixed it. Minimum fix is to make `launched: 0` its own loud
  verdict independent of `--gate`; "no tests ran" is not a ratchet result at all.

- **The hang-capture symbolicator resolves against the WRONG IMAGE — a TOOL path, not just
  the hand path. UNOWNED** (2026-09-19). The kernel-symbol entry below says the helper fix
  "does not close the hand path". It does not close this tool path either. An i386
  conformance hang wrote, into `summary.json` itself:

        CPU#0 0x80211b45 -> 0014bb45    XHF_NOEXECUTE+310085    -

  An offset of 310,085 bytes past a symbol is not a resolution, and `XHF_NOEXECUTE` is not a
  kernel symbol — yet it is presented in the same field, same format, as a real answer. This
  is the entry below's own warning ("the wrong answer is PLAUSIBLE, not empty") arriving
  through a tool rather than a person, which is worse: a human reading `summary.json` has no
  cue to distrust it. Found by the lane that owned the hang, after being warned off the hand
  version of the same trap. **Anything that symbolicates a guest PC must resolve against
  `ntkrnlmp.exe`** — the booted image — and the conformance hang capture evidently does not.

- **`smoke` structurally cannot catch a hang that happens after its marker. OPEN, by design**
  (2026-09-19). A batch whose i386 guest froze inside the taskbar's `OnCreate` passed
  `smoke` on **all three arches** in that same batch. Not a flake and not a harness fault:
  `[WIN32K] shell ready` is at line **1266 in both** the failing tree and a green control,
  and the hang is at line **1891**. Smoke's contract is "reached a graphical desktop", and the
  guest genuinely did. AGENTS.md already warns that the screenshot is captured *at* the marker
  before the taskbar renders; this is the same gap with teeth — **everything explorer does
  after the marker is unguarded by the north-star test**, including `ProcessStartupItems`,
  which is what launches the conformance sweep. The failure surfaced only in `conformance`,
  ~400s later, as `launched: 0`. Worth deciding whether a post-marker liveness check belongs
  in smoke, or whether conformance's `launched` count is the intended guard and simply needs
  to be a louder verdict than "0 tests ran, in baseline (OK)".

- **Dropped `_SEH2_EXCEPT` in LPC — FIXED 2026-09-20 across all six files; a partial
  second pass remains.** Every `ProbeFor*` + capture site in `lpc-rust` whose C handler
  was plainly `return _SEH2_GetExceptionCode()` now goes through the validate-first
  substrate: `port.rs`, `reply.rs` (R1/R3/R4/R5/R8), `create.rs`, `complete.rs`,
  `connect.rs`, `send.rs`. Both arches boot 11/11 on the WoW64 ladder, which is a real
  test of this code rather than an adjacent one — every CSR call in the system goes
  through `NtRequestWaitReplyPort` and `NtSecureConnectPort`.

  **`tools/scan_dropped_seh_pairs.py` now gates the class at ZERO**, with a control in
  `test_scanner_controls.py`. It is the cohab-flip half that `scan_dropped_probes`
  structurally cannot see: that one reads the commented-out C rustconv leaves inline in
  the same `.rs`, and a cohab flip puts the two sides in different files. It caught its
  author's own omission — `connect.rs` was audited and then never converted, and the
  scanner is what noticed.

  **STILL OWED (second pass): the sites whose C handler does CLEANUP or falls through.**
  `reply.rs` R2/R6/R7/R9, `create.rs` C5, `connect.rs` #8/#9/#10, `complete.rs`'s copy and
  write-back, `send.rs` :522/:843/:1000. These are not mechanical: `reply.c:644`'s handler
  assigns `Status` and falls through to a pool-free and an `LpcpLock` release, so a naive
  `return Err` leaks the message AND leaves the lock held. Also owed:
  `LpcpVerifyMessageDataInfo` walks past the probed region and is documented in-place as
  unprotected, with a note on why blanket-probing `TotalLength` is the wrong fix.

  *Original filing, kept because it is why the entry existed:* **UNOWNED, VERIFIED**
  (2026-09-19). Found by generalising the research
  lane's `MiDoPoolCopy` discovery (below) into a tree sweep. `ntoskrnl/lpc/*.c` is
  commented out in `ntos.cmake:211-217`, `lpc-rust` is linked into `kernel-rust`, so the
  Rust IS the shipping LPC. Side-by-side at the same site:

        C   reply.c:205   if (PreviousMode != KernelMode) {
                            _SEH2_TRY { ProbeForRead(...); Captured = *(volatile ...)ReplyMessage; }
                            _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
                              { _SEH2_YIELD(return _SEH2_GetExceptionCode()); } }
        Rust reply.rs:249 if previous_mode != 0i8 {
                            ProbeForRead(...);
                            captured_reply_message = *(ReplyMessage as *const PORT_MESSAGE); }

  `NtReplyPort` (`reply.rs:236`) is `#[no_mangle] extern "system"` — a syscall entry point.
  The C returns the exception code to the caller; the Rust lets `ProbeForRead` raise into
  nothing. **Any unprivileged user process passing a bad `ReplyMessage` crashes the
  kernel.** Far wider reach than the MM case, which needs a cross-process write. (The
  `volatile` on the capture read was also dropped — separate and minor, but same site.)
  **Verified: `reply.rs` only.** Five more LPC files show the same shape and are NOT
  individually confirmed: `send.rs`, `connect.rs`, `complete.rs`, `create.rs`, `port.rs`.
  Nothing in the whole `lpc-rust` crate matches any catch idiom (`probe::`,
  `Result<_, NTSTATUS>`, `catch_unwind`, `_SEH2`).
  Fix is the same validate-first `probe.rs` idiom `dummy-pseh-verdict.md` prescribes.

  **On the sweep, because the method matters more than this one bug.** The research lane
  proposed the tell "an unreachable error path is the fingerprint of a deleted handler"
  (`_FailedInProbe` set, reset, then tested) and hoped it would find the rest of the class.
  Implemented, it finds **only `virtual.rs`** — and the reason is worth keeping: that
  fingerprint exists only where the C happened to use a *flag*. A dropped handler that
  guarded a plain `return` leaves no trace at all, which is exactly the LPC shape. A naive
  version of the flag sweep also produced 8 false positives, because `X = 1` in an `if`
  arm and `X = 0` in the `else` arm looks identical to set-then-reset unless you reject any
  brace between them (fastfat `RecursiveClose` is the clean example).
  What DOES generalise: **pair each C file containing `_SEH2_EXCEPT` with its translated
  `.rs`, and flag the ones whose Rust can raise (`ProbeForRead`/`ProbeForWrite`/
  `ExRaiseStatus`/`ExRaiseAccessViolation`) with no catch idiom.** 19 hits, of which the
  same-subsystem ones are the real leads; basename matching mis-pairs the rest
  (`fastfat/create.c` to `lpc-rust/create.rs`), so read the pairs before believing them.
  Still to triage from that list: `mm/section.rs`, `cc-rust/copy.rs`, `win32k-rust`
  {`class`,`hook`,`desktop`}`.rs`. **This wants to become a real scanner** — it is a
  source-only check that needs no boot, and it is currently a shell one-liner in a commit
  message, which is the definition of a rule that will be forgotten.

- **`NtWriteVirtualMemory`/`NtReadVirtualMemory` BUGCHECK instead of returning
  STATUS_ACCESS_VIOLATION — a dropped `_SEH2_EXCEPT` in the Rust MM translation**
  (found 2026-09-19 by `research`, via the new `spawn32` WoW64 guest). `virtual.c:876`
  wraps both probes in `_SEH2_TRY { … } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) { Status =
  _SEH2_GetExceptionCode(); }`. The Rust port kept the body and **dropped the handler**, at
  four probe sites across `MiDoMappedCopy` and `MiDoPoolCopy`
  (`ntoskrnl-rust/src/ARM3/virtual.rs`, around lines 842/869 and 961/986). `ProbeForWrite`
  raises via `ExRaiseAccessViolation()`, nothing catches it, and the machine takes
  **BUGCHECK 0x1E (`c0000005`)** — KMODE_EXCEPTION_NOT_HANDLED — where the contract is to
  return a status.
  **Reproduced on amd64**, deterministic: `ProbeForWrite` (`ex-rust/src/exintrin.rs:267`)
  called from `MiDoPoolCopy` (`virtual.rs:987`). Repro is
  `VIBEOS_WOW64_SPAWN=1 tools/fex/build-hello-compute.sh` then an amd64 `smoke --ladder` —
  the guest is opt-in precisely because it kills the boot.
  **NOT a WoW64 bug**, despite how it was found: any caller writing into another process
  with a bad target address and `PreviousMode != KernelMode` reaches the same sites. A
  32-bit guest's `CreateProcess` was simply the first thing in the tree to do it, and it
  gets there only AFTER `NtCreateProcessEx` has already succeeded.
  **The tell, for finding the rest of this class:** a recovery branch that nothing can
  reach any more. Here `_FailedInProbe` is set to 1, the probe runs, it is set back to 0,
  and a later `if Status < 0 && _FailedInProbe != 0` tests a state no line between them can
  now produce. **An unreachable error path is the fingerprint of a deleted handler**, and it
  is visible in the Rust alone without diffing the C.
  **PROBE HALF FIXED 2026-09-19 by `research`; the LOCK half is still open and is now what
  this entry tracks.** All four probe sites moved to validate-first
  `ntkrnl_sys::probe::probe_user_buffer` (`Result<NTSTATUS>`), which also makes the
  `if _FailedInProbe != 0` branch reachable again — setting `Status` is exactly what the
  deleted handler did, so the C's discriminator between "probe failed" (return the code)
  and "copy faulted" (`STATUS_PARTIAL_COPY`) works once more.
  **The copy needed fixing too, and the tree PROVED it rather than my predicting it.** With
  only the probes converted, the same BUGCHECK 0x1E came back one frame along, now inside
  `memcpy` — because the C's TRY also wrapped `RtlCopyMemory`, and a range check proves
  where a buffer IS, never that it stays mapped. All three copies now go through
  `ntkrnl_sys::probe::safe_copy_raw` (`KmProtectedCopyMemory`, genuinely fault-protected on
  x86_64/aarch64/x86). Leaving `_FailedInProbe` at 0 on that path is what selects the C's
  copy-faulted arm over its probe-failed one.
  **Verified end to end on amd64:** zero bugchecks, and the third `NtWriteVirtualMemory`
  into a child process now returns **`8000000d` (STATUS_PARTIAL_COPY)** — exactly the C's
  `MiGetExceptionInfo`/`HaveBadAddress` arm, the branch that had been unreachable. A
  syscall that killed the machine now returns a status.
  **And restoring that path immediately exposed a SECOND bug in the same three lines:**
  they returned `0xC000009B`, which is `STATUS_DFS_EXIT_PATH_FOUND`, not
  `STATUS_PARTIAL_COPY` (`0x8000000D`) — the numeric neighbour of
  `STATUS_INSUFFICIENT_RESOURCES` (`0xC000009A`) used a few lines above. It was invisible
  while the path was dead code, and while it was live it made a caller report
  `ERROR_PATH_NOT_FOUND` for a memory fault. `ntkrnl-sys::base` now has a named
  `STATUS_PARTIAL_COPY`; the guest reports `ERROR_PARTIAL_COPY` (299). `vibeos-cli check`
  passes both before and after, so `scan_invented_constants` does not catch a wrong bare
  hex in an `as NTSTATUS` cast.
  **FIXED 2026-09-25:** `MiDoMappedCopy` now calls `MiProbeAndLockPagesStatus`, the
  status-returning body of `MmProbeAndLockPages` (the export still raises). Its page touch is
  fault-protected (it had been a bare, non-volatile read), and two `MmAccessFault` failure
  paths no longer release the PFN/working-set lock a second time. appstorm's stack dumper
  bugchecked amd64 through this; `tools/payloads/rpmprobe` pins it on amd64 and i386.

- **`test_nested_cargo_deps` is load-sensitive and bounces `land` preflights —
  REPRODUCED, OPEN** (2026-09-19). **Three independent observations, three DIFFERENT steps
  of the same test, two different parametrisations** — so this is one invalidation bug that
  hits whichever step lands unluckily, not a bug in a particular crate-type or input kind:

        research  ["staticlib"]          build(15) got 6    <- shared/src/lib.rs edit
        msvcrt    ["staticlib","rlib"]   build(33) got 15   <- body/asset.txt edit
        control   ["staticlib"]          build(60) got 150  <- shared/Cargo.toml feature edit

  Read the assertion correctly: `build(N)` asserts the *program prints* N, so a low number is
  the PREVIOUS value still linked. It is not a count of cargo invocations.
  **Load-sensitive, now measured:** 1 failure in 16 test instances with the fleet saturated
  (a conductor gating + 2 lane builds); **0 in 36** once load dropped; 0 in 18 earlier at
  moderate load. `land` runs `check-tooling` at preflight, so at ~1-in-8 under load this
  rejects innocent lanes — and it does it worst exactly when the queue is deepest.
  **Not a code difference:** the reporting lane's branch had `test_nested_cargo_deps.py`,
  `cargo_deps.rs` and `retarget_cargo_depfile.py` byte-identical to master. Master is **not**
  deterministically red — it passes here 54/56 instances — so do not chase a phantom hard
  failure.
  **Disconfirmed:** mtime granularity, the obvious candidate. Both `/tmp` and the repo are
  ext4 with nanosecond stamps (~0.7ms measured, 6/6 distinct writes).
  **Best current hypothesis, untested:** that a manifest-only edit can also fail points at
  `outer/build.rs` not re-running at all — which would starve every input kind uniformly,
  since the nested cargo only runs from inside it. Emitting ANY `rerun-if-changed` disables
  cargo's default whole-package watching, so an incomplete watch list from
  `cargo_deps::watch` would do exactly this. Next step: catch a failure and read the
  leftover `invocations` file under `/tmp/pytest-of-*/` — it counts `build.rs` executions and
  settles which layer went stale in one look.


- **Kernel symbols resolve against the WRONG image since the MP promotion — TOOLING FIXED,
  HAND RESOLUTION STILL SUSPECT** (2026-09-13). Every medium carries both kernels and boots the MP one
  (`livecd.ini:59` → `/KERNEL=ntkrnlmp.exe /HAL=halmp.dll`), so anything that resolves a
  kernel symbol against `ntoskrnl.exe` is answering for an image the machine is not running.
  Hit independently by two lanes within hours, by different routes and neither looking for it:
  `qemu/gdb-helpers/find-kbase.py` relocates by matching the export Name against
  `VIBEOS_KERNEL`'s basename and matched the UP image, so `run-until.sh --break <sym>` set
  breakpoints the guest never executes and timed out silently (smp-bringup; fixed by
  `0280df67353`, **verified on origin/master 2026-09-13**). And by hand, with no script
  involved: `build-arm64/ntoskrnl/*.exe`
  lists only `ntoskrnl.exe` while the MP image is a directory deeper in `ntkrnlmp/`, so a
  glob steers you to the wrong file.
  **The danger is that the wrong answer is PLAUSIBLE, not empty.** The same fault address
  resolved two ways:

        ntoskrnl.exe  -> KeSetIdealProcessorThread +0x1c     (wrong image, real symbol)
        ntkrnlmp.exe  -> RtlRaiseStatus +0x44                 (booted image)

  Only incoherence caught it — BUGCHECK 0x1E is KMODE_EXCEPTION_NOT_HANDLED with
  P1 = STATUS_FILE_INVALID, and an ideal-processor setter raising a file-status exception fits
  nothing, while `RtlRaiseStatus+0x44` fits exactly. Had the fault been anywhere near
  scheduling, the wrong symbol would have been believed.
  **The helper fix does not close the hand path, which is why this entry stays open.** No
  script was involved in the second route, so no script fix reaches it: any
  `llvm-symbolizer`/`nm`/`objdump` resolution done between the 2026-09-12 promotion and the
  fix is still suspect, and a backtrace or fault symbol from that window should be re-derived
  rather than trusted. The durable defence is the directory layout, not the tool — reach for
  `ntoskrnl/ntkrnlmp/ntkrnlmp.exe`, and confirm against what the boot actually loaded
  (`ramdisk(0)\reactos\system32\ntkrnlmp.exe` appears in the serial).

- **54 i386 crates build on an SSE/ABI combination rustc is phasing out — OPEN, latent**
  (2026-09-13, research lane). They pass `-C target-feature=-sse,-sse2` while targeting the
  BUILTIN `i686-pc-windows-gnu`, whose spec declares `rustc-abi: x86-sse2`. Both the pinned
  stable (1.92.0) and the pinned nightly (2025-12-14) emit: *"target feature `sse2` must be
  enabled to ensure that the ABI of the current target can be implemented correctly … this was
  previously accepted by the compiler but is being phased out; **it will become a hard error in
  a future release**"* (rust-lang#116344). **Not broken today and not urgent — the trigger is a
  toolchain bump**, which is exactly when it will fire, and the pin exists so bumps are
  deliberate. Verified: 55 CMakeLists carry the flag and **all 55** use the builtin triple, none
  the custom `i686-kernel.json`. The fix already exists in-tree: that json sets
  `rustc-abi: x86-softfloat`, which is the sanctioned way to disable SSE on i386, and the kernel
  `*-cargo` cells are on it. **Answered 2026-09-13 by reading the link graph, which kills the
  obvious remedy:** at least 52 of the 55 genuinely need SSE off, so "split by KIND and let
  user-mode drop the flag" buys almost nothing. 42 are plainly kernel-side
  (`drivers/`, `hal/`, `ntoskrnl/`, `win32ss/drivers|reactx`). Ten more are `sdk/lib/*` STATIC
  LIBRARIES whose kind is decided by their consumers, not their path — `ntoskrnl/CMakeLists.txt:89`
  links `csq arbiter cmlib ntlsalib rtl`, so those land inside the kernel image and must be
  no-SSE however user-ish `sdk/lib/rtl` looks. **Classifying these by directory is the wrong
  oracle**; the question is link-graph reachability into a kernel PE. Only `concfg`, `msvcrt`
  and `ucrtbase` are candidates for simply dropping the flag (no kernel-side consumer found);
  ucrtbase DID drop it (2026-10-03, for the `libm` crate -- see its CMakeLists).
  So the fix for the other 52 is the softfloat ABI the kernel json already carries, not a KIND
  split. **Still unverified:** whether a newer rustc actually hard-errors — that is rustc's own
  claim, not a test of mine. Whoever bumps
  the pin should re-run `rustc --target i686-pc-windows-gnu -C target-feature=-sse,-sse2` first;
  it is a one-command check.

- **2026-10-04: x86 guests run under FEX on a real Raspberry Pi 4.** The FEX arm64 media
  (`VIBEOS_FEX=1`, MP kernel, Cortex-A72 x4) netbooted to the desktop and ran the whole ladder:
**13 of 14 x86 guests pass** plus `fexprobe` — including both stress guests, `lua32` and
  `sqlite32` — with no bugcheck. The one miss is `marshal32`, which hangs in a loopback
  `sendto()` that is a board-side networking bug, not FEX (registry: "Pi 4 (real hardware): a
  loopback UDP `sendto()` never returns"). Run `/tmp/vibeos-cli/20261004-202912-qddv97oj-pi4-boot-arm64`.
  - **FEX takes ~1,200 alignment faults per ladder boot, on the board AND in QEMU.** FEX emits
    `STLR`/`LDAR` for x86-TSO ordering; on ARMv8.0 (the A72 has no LSE2) an unaligned one takes
    an alignment fault (`ESR=0x92000061`, raised as `STATUS_DATATYPE_MISALIGNMENT`, e.g. opcode
    `0x889FFE9F` = `STLR wzr,[x20]`), and FEX recovers it. 1,229 on the Pi; 1,190 in a QEMU
    boot of the same tree (`20261004-204131-hsbqj3g1`), so TCG enforces it too. (This entry
    first said QEMU does not fault and the path "had never run" — an inference from the board
    log alone, refuted by counting the QEMU log.)
  - **What differs on the board is what each fault COSTS.** `ke/arm64/trap.rs`'s `[ALIGN]` decode
    and the `[UEXC]` dispatch dump wrote ~1 KB per fault byte-by-byte to the PL011 inside the
    trap: instant on QEMU's virtual UART, ~90 ms at 115200 baud (computed from the line rate,
    not measured). Now capped: those two codes print in full for their first 32, then as a
    count at each power of two; tripwire `UEXCVERBOSE` restores everything (`133142f2ff9`).
  - **Timed on the board (`pi4 boot --ladder`, `20261004-210344-k4a1c7i7-pi4-boot-arm64`):**
    every guest runs in **1–5 s** launch-to-verdict — `lua32` 2.6 s, `sqlite32` 2.6 s, `gui32`
    4.5 s — and the whole ladder takes 121.6 s, 67.6 s of which is winlogon waiting out the
    hung `marshal32`. Power-on to desktop was 347 s; most of it is sanbooting the 564 MB FEX
    image (a plain arm64 image reached the desktop in ~100 s on 2026-09-26), unmeasured split.
  - **Then 11/11 + all four non-parity guests, fully green, on the board** once netstack stopped
    pending TDI IRPs into a dead engine (`e0674df2c66`):
    `/tmp/vibeos-cli/20261004-221515-_d97sb6p-pi4-boot-arm64`, ladder drained in 61 s.
  - QEMU on the same tree: the ladder was red on `spawn32` (0xBAE4) — a lost-wakeup race in the
    guest itself (it closed the named DONE event before the child opened it), fixed in
    `509f9d7939f`; green UP and `-smp 4` after.

- **RETRACTED 2026-10-03: "arm64 WoW64 guests fault INSIDE FEX's code cache" was never a bug.**
  Settled by rebuilding the tree it was measured on (`c3bacfab2d4`, worktree
  `vibeos-fex-sept`, same `lua32.exe` by sha256) and booting it. Demonstrated:

  1. **`lua32` passes on the September tree** — all six rungs, `exit = 0x600d` — once the boot
     is not cut short (`smoke --until "sqlite32.exe exit"`; twice, artifacts
     `20261003-205955-c25w6b__` and `20261003-210513-ias8s8nj`).
  2. **The September "death" was the ladder harness.** That tree's `watch()` computes
     in-flight optional guests from the serial drained *since the call*, so it breaks the
     moment the last parity guest reports if `lua32`'s launch line has not arrived yet — and
     winlogon only launches `lua32` after the last parity guest exits. Reproduced with
     `--ladder` on the September tree (`20261003-205536-fvua4odz`): drain stopped at 73.7s of a
     420s ceiling, `LADDER_FAIL: lua32.exe never reported`, while the serial after the stop
     shows `lua32` still alive, making syscalls. Fixed `8756f3fbe4a` (2026-09-26, "ladder drain
     stops racing launches") — after the September measurement, before the October farm.
  3. **The symbolicated `[UEXC]` records are the normal path.** The September tree logs 3
     LookupCache AVs in *every* FEX process, including the nine that pass, at the same RVAs and
     the same first address `0x7FFFBEFB7858` the September entry quoted as the crash. They are
     FEX's designed overcommit emulation (below) and they recover.
  4. **`sqlite32`'s failure is real but is a KERNEL bug, not FEX:** `BUGCHECK 0x1E`
     (`0xC0000098`, `STATUS_FILE_INVALID`) raised via `RtlRaiseStatus` on sqlite32's own thread,
     raising frames outside ntoskrnl's symbol table (a driver; fastfat is the obvious suspect for
     an on-disk database, unproven). **`2252abfbb61` — the fastfat dirent-bitmap fix — is NOT
     the cure:** cherry-picked onto the September tree it removes the `FatDeleteDirent` ASSERT
     (3 hits → 0) but `sqlite32` still bugchecks identically. Something else between 09-13 and
     10-03 fixed it; `sqlite32` passes 10/10 at HEAD. Owed to the fastfat/IO lane as a
     bisect, not to FEX.

  Inferred (consistent with all of the above, not separately tested): the "size correlation"
  was a coincidence of which guests were OPTIONAL — `lua32` and `sqlite32` were the only two
  subject to the drain race, and both happen to be big; the size-matched control was optional
  too. None of the four arm64 exception-path fixes is needed to explain anything. The
  overcommit analysis and the 10-boot farm below remain accurate as descriptions of FEX's
  behaviour; they are no longer evidence of a bug.

  **History, kept because the retraction is only as good as the record it corrects.**

- **arm64 WoW64 guests fault INSIDE FEX's code cache — DOES NOT REPRODUCE at
  `87944d5e281`+ (2026-10-03), and the measurement CONFIRMS the overcommit mechanism rather
  than clearing it.** One `smoke --arch arm64 --ladder` run, artifact
  `/tmp/vibeos-cli/20261003-100335-xc8cw6oq-smoke-arm64`: **all 11 parity guests PASS, and both
  stress guests PASS** — `lua32.exe exit = 0x600d` with all six rungs (parse/GC/4000 longjmp
  unwinds/400-deep recursion/500 coroutine switches/format round-trip), `sqlite32.exe exit =
  0x600d` with all five. In September these two were the only reds and `lua32` died *before*
  printing `[lua32] start`.

  **The faults did NOT go away — they are now RECOVERED, which is the point.** The same run
  logged **55 access violations inside `libwow64fex`, at exactly two RVAs**:

        30x  RVA 0x10740  DFSC 0x7 (translation fault, L3)
        25x  RVA 0x0FB88  DFSC 0x6/0x7 (translation fault, L2/L3)

  Symbolicated **against the binary that actually ran** (`sha256
  2ea16e3c702120611e35bf9d1e979d3e9dfa2c9e62198f573ef808d2e5092edb`, ImageBase `0x180000000`,
  built 2026-10-03 — NOT the September binary, per the warning below), using the nearest
  preceding `T` symbol from `llvm-nm`:

        0x0FB88 +0x20   FEXCore::LookupCache::FindBlock(InternalThreadState*, unsigned long long)
        0x10740 +0x158  FEXCore::LookupCache::AddBlockMapping(InternalThreadState*, ...)

  Four processes loaded the plugin at four different bases (`0x410000`, `0x480000`, `0x4D0000`,
  `0x510000`) and every AV resolves to one of those two RVAs against *its own* base. So: the
  read side and the write side of the lookup structure still take translation faults on cold
  pages, at the same rate and the same two sites as when the guests were dying — **and the
  guests now pass.** That is precisely what the designed overcommit emulation predicts (see the
  mechanism section below): the AV is step one, `OvercommitTracker::HandleAccessViolation`
  commits 64K and returns `true`, the instruction retries. September's bug was never "a fault
  happened"; it was that the recovery did not complete.

  **10-boot farm, 2026-10-03: 0/10 failures, and 539 recovered faults.**
  `smoke --arch arm64 --ladder --repeat 10` (sequential, one boot slot — artifact
  `/tmp/vibeos-cli/20261003-200522-wtpis3_x-smoke-arm64`, 21m09s). Every boot: **14 guests
  pass**, `lua32` and `sqlite32` both `0x600d`. **140/140 guest verdicts green.** Per boot the
  same two sites still fault and still recover:

        LookupCache AVs/boot   51-56   (total 539: FindBlock 239, AddBlockMapping 300)
        other AVs/boot         ~200    (total 1998, also recovered)
        alignment fixups/boot  ~1192   (total 11920, x86-on-arm64, expected)

  The narrow spread (range 5 over 10 boots) says this is **not a rare race** — it is a
  deterministic function of how many cold pages the cache touches, firing ~54x per boot and
  recovering every time. 539 demand-commits, 539 recoveries, 0 deaths.

  **The commit-and-retry chain is verified end to end — from logs we already had, no new
  instrumentation.** The kernel's `[UEXC]` dump carries `P1` = `ExceptionInformation[1]`, the
  fault address. Attributing each AV to its process by load base, across all 10 boots:
  **538 LookupCache AVs, 0 repeat faults on the same page, 0 faults inside a 64K window an
  earlier fault would have committed.** Every AV is the first touch of a fresh region, and
  that process's next fault always lands outside the 64K `HandleAccessViolation` commits — so
  handler reached, commit succeeded, instruction retried, 538/538. Reads 238 / writes 300,
  matching `FindBlock` (reads) and `AddBlockMapping` (writes). The first fault in a process is
  at `0x7FFFBEFB7858`, **byte-identical to the September address above** — `MEM_TOP_DOWN`
  places the reservation identically every time, so the two measurements are directly
  comparable.

  **What that says about September.** A failed *commit* would show as a repeat fault at the
  same address — a retry loop, i.e. a hang. September's guests instead **died with
  `c0000005`**, which means the AV went *unhandled*: either `Query.Enclosed` was false (the
  address was not in a registered overcommit interval) or the plugin's handler was never
  reached. No September serial survives the disk failure to tell those apart, so the only way
  forward is to reproduce it: build the 2026-09-13 tree and boot it.

  **What 0/10 does and does not bound.** It is strong evidence against the JULY rate and weak
  evidence against a low one: at July's 3/6 (50%) a 10/10 green run is p=0.001; the 95%
  confidence upper bound from 0/10 is only ~26%. **And the farm was deliberately polite** —
  sequential, one slot, so as not to disturb other lanes. July's 3/6 came from a 6-VM soak and
  its mechanism is described as hitting "whichever guest is mid-actctx when starved", i.e. it
  is host-contention-sensitive. A sequential farm is the condition LEAST likely to reproduce
  it, so this result rules out a frequent contention-free failure and says little about the
  contended case. **The remaining experiment is a deliberately-contended soak**, run when it
  will not collide with other lanes — not more sequential boots, which buy more of the same.

  **What this does NOT establish.** One green run does not retire a nondeterministic bug — this
  one was always intermittent (`file32` faulted once and passed later), so a single pass is
  weak evidence on its own and the honest status is "not reproducing", not "fixed". Nor does it
  identify a fix: four arm64 exception-path changes landed between the September measurement
  and this one (NEON through `NtContinue` `501f350f3d0`, PSTATE.I on a Rust SEH catch
  `1a06337252d`, `brk #0xF000` dispatch `1406ebb8d77`, VA 0-64K unmap `a828512ac9b`) and any,
  none, or a combination could be responsible. **Owed: a `farm` soak for a rate**, which is the
  only thing that can distinguish "fixed" from "got lucky", plus a bisect if it stays green.
  Note also 200 further AVs outside any FEX image, 181 of them at a single PC
  `0x7FFFDFFA02B0` — unidentified, also recovered, probably another designed-AV mechanism
  (FEX's `CallRetStack` grows on fault the same way); not investigated.

  **The September evidence, kept because it is what the above is measured against.**

- **arm64 WoW64 guests faulted INSIDE FEX's code cache — the 2026-09-13 measurement, and it is the best candidate
  yet for the "unstable class" below** (2026-09-13). Found by `lua32.exe`, the second
  third-party stress guest (`tools/stress/lua/build.sh`), on its first arm64 ladder run.
  Two different guests in the SAME boot died with `c0000005`, and both faulting PCs resolve
  into the same FEXCore subsystem:

        lua32.exe   PC=0x0048FB88  base=0x480000  RVA 0xFB88
                    -> FEXCore::LookupCache::FindBlock
                       ESR=0x92000006 (data abort, translation fault), addr 0x7FFFBEFB7858
        file32.exe  PC=0x00420FD0  base=0x410000  RVA 0x10FD0
                    -> FEXCore::Context::ContextImpl::InvalidateThreadCachedCodeRange
                       ESR=0x92000004, then 0x56000000

  One looks a block up in the cache; the other invalidates a cached code range. That is one
  subsystem, approached from its two opposite sides, which is what a corrupted or raced code
  cache looks like -- and a racy code cache produces exactly the *variety* of nondeterministic
  outcomes recorded in the entry below (lock conflict / execute-at-NULL / BUGCHECK / hang)
  rather than one clean repro. Treat those two entries as possibly the same root.

  **Why the symbolication is trustworthy here** (it is the whole finding, so it was checked
  rather than assumed): the two processes loaded `libwow64fex.dll` at DIFFERENT bases --
  0x480000 and 0x410000, read off each process's own `CPU layer libwow64fex.dll loaded @` line,
  not guessed -- and two independent base subtractions both landing in FEXCore's cache code is
  not something a wrong base produces; a wrong base yields unrelated symbols. Resolved against
  `build-fex-arm64/Bin/libwow64fex.dll`, which is the exact binary `mkramdisk.sh` stages
  (PE ImageBase 0x180000000). **Do not symbolicate this against a rebuilt FEX**: a fresh
  `tools/fex/build-fex.sh` of the same vendored source produces a DIFFERENT binary
  (md5 differs), so its addresses do not transfer.

  `file32.exe` is a PARITY guest that passes normally, so this is not a stress-guest-only
  problem and not a lua32 defect -- lua32 merely reached it first and made it legible. It also
  means a red `file32` on arm64 should be read as a suspected code-cache fault before anything
  file-related.

  **DISCRIMINATED 2026-09-13: it is NOT translated-code volume. The hypothesis below is
  FALSIFIED, and the prediction was committed before the run.** A size-matched control was
  built — the same Lua 5.4 objects linked in, 505,344 bytes against the real guest's 508,928
  (0.7% apart), but with `main()` returning the PASS code immediately so that not one Lua opcode
  ever executes. Staged via `VIBEOS_LUA32_EXE` and verified by hash that the control, not the
  real guest, reached the media. Result: **the control faults identically**, at the same PC, in
  the same function, while all nine parity guests PASS in the same boot:

        control  PC=0x0048FB88  base=0x480000  RVA 0xFB88  -> FEXCore::LookupCache::FindBlock
        control  PC=0x00490740  base=0x480000  RVA 0x10740 -> FEXCore::LookupCache::AddBlockMapping

  So a guest that runs essentially no guest code fails the same way as one that runs six rungs of
  interpreter, GC and coroutine switching. Code volume, `setjmp`/`longjmp`, the block cache being
  *invalidated* — none of it is the trigger. The naive image/address-range reading is what
  survives.

  **The mechanism this points at, and it is on OUR side of the contract.**
  `FEXCore/Source/Interface/Core/LookupCache.cpp:27` sizes one large allocation as
  `VirtualMemSize / PAGE_SIZE * 8 + CODE_SIZE + MAX_L1_SIZE` and indexes it by GUEST ADDRESS:
  the header comment is `PageMemoryMap[VirtualMemoryRegion >> 12]`. Its own comment notes "at
  64GB of virtual memory this will allocate 128MB of virtual memory space" — i.e. it is a large
  SPARSE reservation, and `CTX->SyscallHandler->MarkOvercommitRange(PagePointer, TotalCacheSize)`
  says out loud that FEX expects Linux-style overcommit: reserve big, touch lazily, fault pages in
  on demand. Both faulting functions are the read side and the write side of exactly that
  structure, and the fault is ESR `0x92000006` — a translation fault, i.e. touching a page that
  is not backed.
  **INFERENCE, not yet observed:** if our MM does not demand-commit that sparse range the way
  FEX assumes, then a guest whose code spans more pages indexes further into the map and reaches
  an unbacked page, while a tiny guest never does. That fits every datum — the size correlation,
  the two fault sites, the nondeterminism, and `file32` (8K) faulting once and passing later.
  It is testable directly: dump the region's `MEM_COMMIT`/`MEM_RESERVE` state around
  `PagePointer` at the moment of the fault.

  **READ THE CODE 2026-10-03, and it reframes the question: the fault is step one of a
  DESIGNED mechanism, so the bug is that the recovery does not complete.** FEX does not assume
  the OS overcommits. It *emulates* overcommit on Windows, explicitly, in three parts:

  1. `LookupCache.cpp:40` calls `FEXCore::Allocator::VirtualAlloc(TotalCacheSize, false, false)`
     — the third argument is `Commit`, and it is **false**. `AllocatorHooks.h:49` turns that into
     `MEM_RESERVE | MEM_TOP_DOWN` with **no `MEM_COMMIT`**, `PAGE_READWRITE`.
  2. `LookupCache.cpp:49` registers the range: `MarkOvercommitRange(PagePointer, TotalCacheSize)`
     → `OvercommitTracker::MarkRange` (`Source/Windows/Common/OvercommitTracker.h`), whose class
     comment is literally *"Emulates memory overcommit of reserved regions with exceptions"*.
  3. The first touch of a reserved page raises `EXCEPTION_ACCESS_VIOLATION`.
     `BTCpuResetToConsistentStateImpl` (`WOW64/Module.cpp:893`, and :879 on the pre-thread path)
     calls `OvercommitTracker::HandleAccessViolation(FaultAddress)`, which commits and returns
     `true` so the faulting instruction is retried.

  So **an AV inside `LookupCache` is expected, by design, on every cold page.** `FindBlock` and
  `AddBlockMapping` are the read and write side of that structure and are exactly where the
  first touch of a new region happens — which is why both faulting PCs land there, and why a
  never-executed control faults identically: it is page-count, not code, that decides how far
  into the map you reach. What our evidence shows is not "FEX touched an unbacked page" (normal)
  but **"the page was still unbacked when the instruction retried"** — the handler did not run,
  or ran and failed, or ran and succeeded but we resumed wrong.

  **Three candidate break points, and the non-Wine path is the demanding one:**

        if (IsWine)  -> commit the WHOLE allocation from Info.AllocationBase
        else         -> VirtualAlloc(AlignedFaultAddress, min(Query.Size, 64KB),
                                     MEM_COMMIT, PAGE_READWRITE)

  We are not Wine, so we take the `else`: **repeated partial `MEM_COMMIT` at an arbitrary
  page-aligned offset inside an existing `MEM_RESERVE`, 64K at a time.** That is a specific MM
  capability, and it is the first thing to check — does our `NtAllocateVirtualMemory` support
  committing a sub-range of a reservation at a non-base address? The other two: does our arm64
  exception dispatch deliver a user-mode data abort on a reserved page to the plugin's handler
  at all, and do we resume correctly when it returns `true`? Note the `IsWine` split means
  upstream has *two* tested paths and we exercise the one Wine never does.

  **Candidate (b) is already WEAKENED by reading our side (2026-10-03).** `NtAllocateVirtualMemory`
  is Rust (`sdk/lib/rust/ntoskrnl-rust/src/ARM3/virtual.rs`) and its "MEM_COMMIT on top of an
  existing reservation" path (from :2552) does implement the private demand-zero case properly:
  it page-aligns the caller's range, requires only that it *fit inside* the found VAD rather
  than start at its base, charges commit, then walks `MiAddressToPte(Starting)..Last` calling
  `MiMakePdeExistAndMakeValid` on the first PDE and at every PDE boundary, writing a demand-zero
  invalid PTE per page. A 64K commit at an arbitrary page-aligned offset is exactly what that
  loop handles. So the MM capability FEX needs appears present; that pushes suspicion onto (a)
  delivery and (c) resume — the same arm64 exception machinery that has taken four fixes since
  these faults were measured (NEON through `NtContinue` `501f350f3d0`, PSTATE.I on a Rust SEH
  catch `1a06337252d`, `brk #0xF000` dispatch `1406ebb8d77`, and the VA 0-64K unmap
  `a828512ac9b`). Reading further is lower value than one boot: **re-measure before theorising.**

  This supersedes "dump `MEM_COMMIT`/`MEM_RESERVE` at fault time" as the next experiment — that
  would confirm the page is uncommitted, which the design already predicts. **Instrument
  `HandleAccessViolation` instead**: whether it is reached, what `Query.Enclosed` says, and what
  the `VirtualAlloc` returns. A probe outside the vendored tree can do it (see
  `docs/engineering/wow64-x64-guest-design.md`), or the kernel side can log reserved-page AVs in
  a WoW64 process.

  **The `TPIDRRO_EL0` lead is FALSIFIED as an explanation of this fault — but it uncovered a
  real, separate bug (checked 2026-10-03).** The lead was: `WOW64/Module.cpp:527` computes
  `IsWine = !!GetProcAddress(NtDll, "wine_get_version")`, `CPUFeatures.cpp:84` sets
  `HostFeatures.SupportsCPUIndexInTPIDRRO = !IsWine`, so FEX takes the real-Windows path and
  assumes our kernel publishes a CPU index in `TPIDRRO_EL0`; I reasoned that a garbage index
  into a per-CPU L1 structure "lands exactly in `LookupCache`". **It does not.** `LookupCache`
  has no CPU-index dependency whatsoever — `grep -niE 'processorid|cpuindex|tpidr|percpu|CPUID'`
  over both `LookupCache.cpp` and `LookupCache.h` returns NOTHING. The feature bit has exactly
  three consumers, and none is the block cache:

        FEXCore/Source/Interface/Core/JIT/MiscOps.cpp:233  DEF_OP(ProcessorID) -> mrs TPIDRRO_EL0
        FEXCore/Source/Interface/Core/CPUID.cpp:1354       GetCPUID = GetCPUID_TPIDRRO
        FEXCore/Source/Interface/Core/CPUID.cpp:1029       SUPPORTS_RDTSCP = SupportsCPUIndexInTPIDRRO

  So it cannot explain a translation fault in the code cache, and the entry no longer claims it
  can. I wrote "a garbage index ... lands exactly in `LookupCache`" without reading either file;
  the disconfirming grep took one command. (AGENTS.md, "a confident prediction about directly
  observable behaviour, made by someone who did not observe it".)

  **What IS true, and is its own bug: our arm64 kernel never writes `TPIDRRO_EL0`.** An
  exhaustive search of our source (excluding `vendor/`, bindgen islands and offset checks) finds
  the name in exactly one place — the `ULONG64 Tpidrro_El0` field of `KSPECIAL_REGISTERS` in
  `sdk/include/ndk/arm64/ketypes.h:266`. A declaration, never an `msr`. For contrast we DO write
  `TPIDR_EL1` (the PCR pointer: `ntoskrnl/ke/arm64/kiinit.c:198`, read by `ctxswitch.S`,
  `KeGetPcr`, `ARM3/helpers.rs`). Consequences, both guest-visible and neither yet observed in a
  boot: FEX **advertises RDTSCP to the guest** in CPUID on the strength of that bit, and lowers
  the guest's processor-ID op to a bare `mrs TPIDRRO_EL0` — so a 32-bit guest calling RDTSCP or
  reading its CPU index gets whatever the firmware left in a register we never initialise or
  context-switch, uniform across threads and architecturally UNKNOWN at reset. Note FEX's own
  comment at `MiscOps.cpp:239`: on Windows without this feature it is `ERROR_AND_DIE_FMT`, i.e.
  upstream treats "Windows host that does not publish the CPU index" as a programming error.
  Worth a guest rung of its own; it is not what kills lua32.

  **NOT corroborated after all — the July analysis names two mechanisms that do not operate in
  our build (checked 2026-10-03). See `m14-startup-flake/README.md` for the full working.**
  Short version: `GuardSignalDeferringSection` is `#else`-branched to a bare lock with no
  deferred-signal refcount on `_WIN32` (`SignalScopeGuards.h:259`), so `CompileBlock` defers
  nothing and a fault inside it reaches the VEH normally; and `MmWorkingSetManager`
  (`ARM3/wslist.rs:504`) has no live caller — `ke/balmgr.rs` has its calls commented out under
  "NO WS Management Yet!" — so nothing trims a working set and no page can be "evicted between
  the prefault and `RtlCreateActivationContext`". The empirical numbers may still be real; the
  explanation is refuted, and **this entry previously cited it as independent corroboration on
  the strength of its prose alone.** Kept below as the historical record.

  **Recovered July work, which reached A mechanism FIRST and appeared to explain the
  size correlation.** `docs/engineering/active-lanes/m14-startup-flake/` holds three patches
  recovered from a stale subagent worktree (`fex-arm-m4`, last touched 2026-07-17) before it was
  removed. Their analysis, arrived at independently and two months earlier: FEX's block compiler
  reads guest code at a call target **inside a signal-deferring section** (`CompileBlock` →
  `GuardSignalDeferringSection`); if that target page is **not yet resident**, the compile yields
  an empty block whose entrypoint is 0 and `ExitFunctionLink` branches straight to it. Measured
  then: **~3 of 6 guests died under a 6-VM soak**, victims varying (gui32/file32/hello32/doom)
  "because it hits whichever guest is mid-actctx when starved" — every WoW64 process's
  `LdrInitializeThunk` runs `RtlCreateActivationContext`, which calls `__alloca` /
  `get_manifest_in_module` on ntdll32 pages *other* than the caller's, so the demand-fault loses
  the race under host contention.

  **That is what makes the control experiment's result make sense.** The never-executed 505K
  control failing identically says the trigger is PAGES MAPPED AND RESIDENT, not code executed —
  a bigger image spans more pages, so more of them can be non-resident when `CompileBlock` reads
  them, and the control has the same page count as the real guest. It also states the
  Linux-vs-Windows assumption from the other direction: *"Upstream FEX assumes CompileBlock never
  fails for a mapped RIP (true on Linux, where the frontend read faults the page in); the
  Windows/WOW64 compile path can observe the transient."* Note the July fault signature is host
  `PC=0` via a null block, while 2026-09 sees faults *inside* `LookupCache` — related family,
  possibly the same root at different stages; **do not assume they are identical without
  evidence.** All of it predates the disk failure and none has been re-verified on this box.

  **Both leads say the same thing about provenance**: the fault executes inside vendored FEXCore,
  but the suspicious inputs are ours — our memory manager's handling of a large sparse
  reservation, and a CPU register our kernel may not set. "Unmodified locates the faulting code,
  not the cause."

  **Superseded hypothesis, kept because the prediction was on the record before the test.**

  **Size correlation, and it reframes the hypothesis.** lua32 was added expecting its
  `setjmp`/`longjmp` rungs to stress the block cache -- but it faults BEFORE printing
  `[lua32] start`, so it never reached `main` and no rung ran. What the two failing guests
  actually have in common is SIZE: `lua32` 509K and `sqlite32` 1.1M, against <=19K for every
  parity guest that passes (hello32 3K ... marshal32 19K). That is a 25-55x gap with nothing in
  between, which makes code-cache capacity/eviction on a large image a better candidate than any
  control-flow pattern. Cheap disconfirming test for whoever picks this up: build a guest that
  is LARGE but control-flow trivial (pad hello32 to ~1M of never-executed code, or link Lua and
  call nothing). If it faults, this is about image size and cache pressure; if it passes, size is
  a red herring and the trigger is what the big guests DO. Note the one exception that keeps this
  honest: `file32` is 8K and faulted too -- but at a different site
  (`InvalidateThreadCachedCodeRange`), earlier in the same boot, and it passes in other boots, so
  it may be the first corruption rather than a counterexample to the size pattern.

  **Next step**: the fault is INSIDE vendored FEXCore -- but that locates the faulting code, not
  the cause, and the two must not be conflated. "We did not patch the file, so the bug is
  upstream's" is an unwarranted inference, and this tree has just produced the counterexample:
  every blocker on running x64 guests turned out to be OURS, in a tree where FEX is equally
  unmodified (`docs/engineering/wow64-x64-guest-design.md`). So "what do we HAND FEXCore" is as
  live a hypothesis as "FEXCore has a bug" -- the guest binaries we build, the memory layout we
  give them, how our loader stages them, our threading. Keep both doors open. Either way the
  useful work is to characterise the trigger from our side -- lua32's rungs are ordered by control-flow violence (`pcall`/`error` longjmp, then
  deep recursion after stack reallocation, then coroutine switching), so bisecting which rung
  first faults localises it to a specific invalidation pattern. lua32 currently faults before
  printing `[lua32] start` at all, which points at process/thread startup rather than at any
  rung, so start there.

- **A 32-bit guest doing real file I/O BUGCHECKs the arm64 kernel — OPEN** (2026-09-13).
  Found on the first writable-volume run of `sqlite32.exe`, the third-party stress guest
  (`tools/stress/sqlite/build.sh`). Real SQLite creating and writing a database on arm64:

        *** BUGCHECK: 0x0000001E (0xffffffffc0000098, 0xfffff800421cbba4, ...)
        P1 = 0xC0000098 STATUS_FILE_INVALID
        P2 = ntkrnlmp.exe+0x3BBA4 = RtlRaiseStatus+0x44

  KMODE_EXCEPTION_NOT_HANDLED: a kernel path raised `STATUS_FILE_INVALID` with nothing to
  catch it. `NtCreateFile` had already returned SUCCESS on
  `\??\X:\reactos\syswow64\sqlite32.db`, so the file was created and the fault is in what
  follows, not in the open.
  **Symbolicate against `ntkrnlmp.exe`, not `ntoskrnl.exe`** — the MP kernel is the default
  boot on all three arches since 2026-09-12, and the UP image resolves the same RVA to
  `KeSetIdealProcessorThread`, which is nonsense that fits nothing. The deeper stack frames
  do NOT symbolicate reliably from the export table (they land past the last symbol and come
  back as `HalAdjustResourceList +0x24d4d8`), so the caller is UNIDENTIFIED — a GDB break on
  `RtlRaiseStatus` with the guest running is the next step, not more log reading.
  **amd64 does not show it, and that is not evidence of an arch difference**: the amd64
  livecd volume is read-only, so the guest takes its in-memory fallback there and never
  exercises the file path at all. An earlier build of the same guest, writing to a different
  path, got as far as `NtLockFile => c0000054` (FILE_LOCK_CONFLICT) and
  `NtUnlockFile => c000007e` (RANGE_NOT_FOUND) on ranges SQLite picks not to overlap — the
  wow64 marshaller for both looks correct (`ByteOffset`/`Length` are layout-identical
  `LARGE_INTEGER*`), so FsRtl byte-range locking is a second, separate lead.
  **The BUGCHECK is one sample of an UNSTABLE path, not a reproducible failure.** Four
  consecutive arm64 runs of the same guest produced four different outcomes: the lock
  conflict above, an execute-at-NULL before `main()`, this BUGCHECK, and a hang at guest
  thread startup. Do not tune a fix to any single one of them. The last runs were also made
  while the fleet saturated the host (a conducting 3-arch gate plus conformance on two
  arches, all TCG), which is exactly the condition under which a slow TCG boot starves, so
  determinism has NOT been established either way -- re-run this on a quiet host before
  drawing conclusions. What IS solid: on-disk SQLite on arm64 does not survive, and one of
  the ways it fails takes the kernel down.
  **REPRODUCED (2026-09-13, later): the BUGCHECK arm is not a one-off.** A separate arm64
  ladder boot on a different build hit the SAME stop with the SAME parameters -- 0x1E,
  P1 `0xC0000098` STATUS_FILE_INVALID, P2 resolving to `RtlRaiseStatus+68` at
  `sdk/lib/rust/rtl-diag/src/exception.rs:48` -- again immediately after `NtOpenFile` on
  `sqlite32.db` returned `00000000`. So of the four outcomes, this one at least recurs
  identically, which makes it the arm to chase first. Two caveats that keep this honest:
  the host was again NOT quiet (load 45-50 on 24 cores), so this does not settle the
  starvation question; and two samples is a recurrence, not a determinism claim -- the owed
  measurement is still a quiet-host run.
  **Lead: the raiser is almost certainly `fastfat-rust`, and the stop is SECOND-ORDER.**
  `STATUS_FILE_INVALID` is not a status the kernel raises casually -- grepping every raiser in
  the tree, the reachable ones on this path are `FatVerifyFcb` (`FcbBad`, and
  `FcbNeedsToBeVerified` that fails to re-verify) and `FatVerifyVcb` (`VcbBad`) in
  `sdk/lib/rust/fastfat-rust/src/verfysup.rs`, plus the cdfs equivalents. arm64 boots a FAT
  ramdisk (`mkramdisk.sh` stages with mtools) so fastfat is the live filesystem; amd64's livecd
  is ISO/cdfs AND read-only, which is the same reason the guest takes its in-memory fallback
  there. **Every one of those raisers fires because the Fcb/Vcb was ALREADY marked bad**, so
  the BUGCHECK is the symptom of an earlier I/O failure, not the failure itself -- which means
  a GDB break on `RtlRaiseStatus` lands one whole event too late and will show a stack that
  merely reports the damage. Break instead on the FIRST transition to `FcbBad`/`VcbBad`
  (`FatDetermineAndMarkFcbCondition`, `FatMarkFcbCondition`), the same "instrument the first
  event, not the loudest one" shape the stock-app NULL cascade below needs. This is a LEAD from
  reading raisers, NOT an observed stack: nothing has yet confirmed the fault goes through
  fastfat at all.
  **2026-09-26: the BUGCHECK arm should now be a failed request.** fastfat runs on real SEH
  (dummy-pseh-verdict.md step 3), so a `FatVerifyFcb`/`FatVerifyVcb` raise reaches
  `FatExceptionFilter` as an expected status and `FatProcessException` fails the Irp instead
  of the raise finding no handler. The lead above still stands: if sqlite32 now opens and
  gets `STATUS_FILE_INVALID` back, the first `FcbBad` transition is the bug to chase.

  **Ordering consequence, now fixed:** this guest can take the whole boot down, so anything
  winlogon launches after it loses its verdict silently. That is exactly what happened when
  `lua32` was added behind it -- the run graded 9/9 parity PASS and said nothing about lua32,
  because the machine was gone. `sqlite32` is now LAST among the optional guests
  (`winlogon.rs`); keep it there while this bug is open.

- **Stock 32-bit GUI apps cascade into an execute-at-NULL fault under WoW64 — ROOT-CAUSED
  AND FIXED 2026-09-20.** It was the GDI TEB batch, never flushed for a guest, overrunning
  `GdiTebBatch.Buffer` into `CLIENTINFO.pDeskInfo`. **This is field manual §9.41 a second
  time** — same bug, and two of that entry's four recorded symptoms are literally
  "CLIENTINFO corruption" and "a user32 `ValidateHandle` AV" — reached through the wrong
  **TEB** instead of the wrong **offset**, which is why `scan_magic_offsets`, the detector
  added for §9.41, cannot see it.

  Every link measured rather than argued:

  1. A guest's gdi32 bumps `TEB32->GdiBatchCount` (`win32ss/gdi/gdi32/include/gdi32p.h:517`).
  2. Every arch's trap path tested **TEB64**'s, so the flush never fired for a guest.
     Measured with per-case latches across a full boot: only ever `wow64=0`.
  3. So nothing reset the batch. Measured: count ran past `GDI_BatchLimit` (20) to 0x2d and
     `Offset` climbed monotonically to 0x628.
  4. Over the limit, gdi32 calls `NtGdiFlush()` on EVERY append and then writes at
     `Buffer + Offset` **without re-checking** (`gdi32p.h:498-515`), trusting the kernel to
     have zeroed `Offset` — true for a native process, never for a guest.
  5. `Buffer` spans TEB32+0x1DC..0x6B4 (`GDIBATCHBUFSIZE` 0x4D8); `Win32ClientInfo` is
     0x6CC and `pDeskInfo` 0x6E4, i.e. **0x508 from the buffer base**. Measured: `Offset`
     passed 0x4D8 and hit exactly 0x508.
  6. `user32!ValidateHandleNoErr` reads it by absolute displacement
     (`mov %fs:0x18,%eax; mov 0x6e4(%eax),%edx`), null-checks it, PASSES (a batch record
     byte is not NULL), dereferences, AVs.

  **Fix:** read the TEB the guest actually writes — `wow64_teb32()` in
  `ntoskrnl-rust/src/ke/amd64/traphandler.rs`, and `KiWow64GdiBatchCount()` in
  `ntoskrnl/ke/arm64/trap.c` shared with `ke/arm64/usercall.c` so the syscall and
  user-callback return paths cannot disagree — plus `Wow64ResetGuestGdiBatch()` from
  `NtGdiFlushUserBatch`.

  **Verified:** notepad faulted 58 times in every one of six boots; with the fix it is
  **0** and reports `still running = PASS`. Whole-boot guest faults 177+ -> 2, both
  survivors being `fault32`/`thread32`'s own deliberate test faults. Zero faults remain
  anywhere in user32's range.

  **The owed half is PAID (2026-09-26):** the guest batch is now EXECUTED, not reset.
  `flush_guest_batch()` in `win32k-rust/src/gdibatch.rs` widens each i386 record into a
  kernel scratch buffer (sign-extending the four HANDLE fields, as the generated thunks do)
  and runs it through the native executor. "Costs no drawing that was not already dropped"
  turned out to be the wrong cost to count: gdi32's `DeleteObject` on a region or brush with
  a user attribute is itself batched (`GdiBCDelRgn`/`GdiBCDelObj`), so every such delete was
  dropped and its handle leaked. See the osk entry under Known bugs.

  **Why `--ladder` could not see any of this for a week**, which is the reusable part:
  `ladder.watch()` stops draining once every GRADED guest reports, and the stock applets
  run after the parity set and are not graded. So the flag whose job is watching the ladder
  ended at 86.8s with the applet phase unobserved. Fixed with
  `VIBEOS_LADDER_DRAIN_FLOOR=<seconds>` (`tests/qemu/cli/ladder.py`), which only ever
  extends the drain; set it above `LADDER_TIMEOUT_SECONDS` (420) or it does nothing.

- **The stock-applet ladder exists ONLY on amd64, which biases every WoW64 finding toward
  "amd64-only" — OPEN** (2026-09-20). `wow64cpu/CMakeLists.txt` stages notepad, mspaint,
  charmap, taskmgr, winver, regedit and the rest onto the amd64 **livecd**; arm64 stages via
  `qemu/mkramdisk.sh`, whose syswow64 set is the purpose-built probe guests plus
  testgui/doom/lua/sqlite and **no stock applets at all** (measured: a FEX-staged arm64
  ladder boot logs `STOCK i386 app` zero times). So the entire applet robustness surface is
  unobservable on arm64.

  **Why this matters beyond coverage:** this registry has now mis-filed WoW64 findings as
  amd64-specific three times, and the x64-primary plan flags the same habit in itself. Part
  of that is not bias in the reader — it is *structural*: the only place that runs stock
  32-bit GUI apps is amd64, so any bug in that class can only ever be seen there, and
  "observed on amd64, not on arm64" is guaranteed regardless of whether the bug is
  arch-specific. The GDI-batch fix above is the live example: its arm64 half is written and
  compiles, and arm64 CANNOT confirm or deny it — a FEX-staged arm64 boot never reaches the
  batch-full path, so `Wow64ResetGuestGdiBatch` logged nothing. Not a green and not a red.
  Staging even two or three applets into the arm64 ramdisk would turn the largest remaining
  WoW64 blind spot into a measurable one.

  **The arm64 run itself: `ladder: 11/11 guests PASS` (+2 non-parity), boot PASS, 0
  bugchecks — so the fix's arm64 half causes no regression, which is the strongest claim
  the media supports.**

  *(Noticed while trying to validate the GDI-batch fix on arm64 rather than assuming
  parity.)* Observed in the same boot and NOT investigated: **`testgui.exe EXITED =
  0xc0000005`** while the ladder reported 11/11, with 61 guest faults in the boot. testgui
  is a STOCK unmodified i386 GUI binary and arm64 already stages it — so it is a better
  arm64 hook for the applet class than adding notepad would be, and the
  "green ladder next to an unhealthy stock-app surface" shape is exactly what started the
  amd64 investigation.

- **`WOW64_TEB32_OFFSET` is hand-rolled in FOUR places — OPEN cleanup** (2026-09-20).
  `ntoskrnl-rust/src/ARM3/procsup.rs:675` (the allocator that defines the layout),
  `ke/amd64/thrdini.rs:235`, `ke/amd64/traphandler.rs` (added by the GDI-batch fix) and
  `win32k-rust/src/wow64.rs`. Each is `const WOW64_TEB32_OFFSET: usize = 0x2000` plus its
  own `(p as *const u8).add(..)`, and each needs its own `// nolint: magic-offset`
  annotation to get past `scan_magic_offsets` — which is the smell: the scanner is being
  told four times that the same thing is fine. It wants ONE typed accessor in `ntkrnl-sys`
  (reachable from both crates) so the layout has a single definition and a single
  annotation. Noticed because the strict deferred lint rejected the fourth copy at
  land preflight; the annotation is the convention the other three already use, so this is
  consistency, not a new exception.

- **A guest fault is AMPLIFIED into process death by a NULL SEH handler — FIXED
  2026-09-20** (`617e235048a`). charmap went 55 guest faults -> 1; whole-boot guest faults
  115 -> 5 (two of which are fault32/thread32's deliberate ones).
  `RtlDispatchException` validated the registration RECORD's stack bounds and alignment but
  never the handler — there was an explicit `RtlIsValidHandler`/SafeSEH TODO at that exact
  line — so a null handler was CALLED, faulted at address 0, re-entered the dispatcher,
  which re-walked the same chain and called it again one frame deeper.
  `rtl-except/src/i386/except.rs` now rejects a handler in the low 64K or inside
  `[StackLow, StackHigh]` (a handler pointing into the thread's own stack is never code —
  that was the cascade's first frame, executing at `001ff49b`). Full SafeSEH still needs
  the image's handler table and remains the TODO.

  **The exit code is what shows it is a fix rather than a suppression:** charmap went from
  `EXITED = 0x80` — csrss terminating after the cascade reached second chance, which
  laundered the real cause — to `EXITED = 0xc0000005`, the actual AV propagating cleanly.
  Measured chains, healthy vs broken, are in the commit. Lands in the i386 ntdll, so it
  protects native i386 too.
  *(Superseded text below.)*
  (2026-09-20). Separate from the entry above and now the more important of the two: it
  turns ANY single guest fault into ~55 and a dead process.
  `ntdll32!RtlpExecuteHandler2`'s `mov 0x18(%ebp),%ecx; call *%ecx` runs with `ecx == 0`,
  once per nesting level at descending `esp`, with `edx` pinned at
  `ntdll32!RtlpExceptionProtector` (ntdll32+0x32d0) throughout — which is how the frames
  are identified. Fixing the CLIENTINFO corruption removed one TRIGGER and left the
  amplifier: `charmap.exe` still shows the same 55-fault cascade from a different first
  fault. Start from the guest-SEH frame the WoW64 layer builds
  (`Wow64pBuildGuestExceptionFrame`) and the handler chain handed to the guest dispatcher.
  The `EXITED = 0x80` verdict on a stock applet is this: csrss terminating the process
  after an exception reached second chance.

- **The initial WoW64 guest stack was a hardcoded 64K instead of the image's
  SizeOfStackReserve — FIXED 2026-09-20.** `dll/win32/wow64/wow64.c` allocated
  `Wow64pAllocBelow2G(0x10000, ...)` for the guest run stack while every stock applet
  declares `SizeOfStackReserve = 0x100000`. Any guest with frames deeper than 64K walked
  off the bottom of its own stack, and the compiler's stack-probe helper found it first:
  charmap died at `__alloca+0x12` — `test %ecx,(%ecx)`, the page probe — reading `001efd84`
  with the stack mapped `001f0000..00200000`, one page BELOW the allocation.

  **It also explains the "corrupt SEH chain" filed separately: same bug, other end.**
  `Wow64pThreadInit` publishes those bounds as TEB32 `NtTib.StackBase/StackLimit`, and i386
  `RtlDispatchException` validates `fs:[0]` registration records against them — so once
  `esp` left the region the chain was out of bounds and read as zeros. The all-zero record
  0x44 below `esp` was a symptom, not a second defect.

  Now reads `SizeOfStackReserve` from the guest's own `IMAGE_NT_HEADERS32` (floor 64K, cap
  16MB, rounded to 64K). Verified: `stack=001f0000..002f0000` (1MB) and **charmap goes
  55 faults -> 1 -> 0 and PASSES**. Whole boot: 13 PASS / 1 EXITED, 4 guest faults (two
  deliberate), 0 bugchecks.

  **Known simplification:** `Wow64pAllocBelow2G` is `MEM_RESERVE|MEM_COMMIT`, so this
  commits the whole reserve where Windows reserves it and commits one page plus a
  `PAGE_GUARD` that grows on demand. ~1MB per guest, no guard-page growth path needed, and
  it cannot fault where the real thing would have grown. On-demand growth is the
  refinement, not the fix.

- **A WoW64 guest's thread stacks CANNOT GROW — the guard-page fault reaches the guest and
  kills it. ROOT-CAUSED, NOT FIXED** (2026-09-20). This is the "a random applet dies each
  boot" behaviour, and it was never several app bugs.

  Every applet death is `EXITED = 0x80`, and the fault before it is **`80000001` =
  STATUS_GUARD_PAGE_VIOLATION**, not an access violation:

        guest fault 80000001 eip=7e017b4c -> 32-bit SEH dispatch (esp=01d8ec38)

  A guard-page hit is how a thread stack GROWS: the kernel should commit the next page,
  move the guard down and resume, so the guest never sees it. Instead it is dispatched into
  guest SEH, nothing handles it, it reaches second chance, and csrss terminates the process.
  MEASURED: exactly **3 guard-page faults per ladder boot**, 1-2 dead applets, and a
  DIFFERENT victim each boot (eventvwr / taskmgr / wordpad) — whichever thread happened to
  need more stack. That rotation is why it looked like unrelated app bugs; the uniform 0x80
  was the tell that they all die the same way.

  Same class as the GDI batch bugs above, a THIRD time:
  `MiCheckForUserStackOverflow` (`ntoskrnl-rust/src/ARM3/pagfault.rs`) reads `Tcb.Teb` — the
  **64-bit** TEB — and tests the faulting address against its
  `NtTib.StackBase`/`DeallocationStack`, i.e. the HOST stack. A guest address is never in
  that range, so the test rejects it and returns `STATUS_GUARD_PAGE_VIOLATION`.

  **A direct fix was tried and REVERTED — but the measurement that condemned it was
  CONFOUNDED, and the revert should not be read as a verdict.** Pointing the bounds test at
  TEB32 gave **0 PASS / 16 EXITED** on both of its two boots, which I compared against a
  12-13 PASS baseline and called a severe regression. Three repeat boots on the *unmodified*
  media then produced 13 / 12 / **0 PASS 16 EXITED** — the total wipeout occurs on stock
  master roughly one boot in three, for an unrelated reason (see the CPU-layer entry below).
  Two wipeouts out of two is barely distinguishable from a 1-in-3 base rate; the change was
  reverted on evidence that could not support the conclusion. Re-measure before judging it.
  The reasoning below still stands on its own as a static defect, independently of that run.
  Cause: **nothing ever populates
  TEB32's `DeallocationStack`.** `Wow64pThreadInit` (`dll/win32/wow64/wow64.c`) writes only
  `NtTib.StackBase` (+0x04) and `NtTib.StackLimit` (+0x08); `grep -c DeallocationStack` over
  wow64.c and wow64_manual.c is **0**. The test degenerated to `Address < 0`, the lower
  bound vanished, and `(NextStackAddress - PAGE_SIZE) <= DeallocationStack(0)` never
  tripped — so the handler committed a fresh guard page below ANY guest guard-page fault,
  walking down the address space with no floor. Stashed as `research: BROKEN
  MiCheckForUserStackOverflow TEB32 attempt 2026-09-20`.

  **So this needs a complete TEB32 stack descriptor, not a kernel one-liner.**
  `Wow64pThreadInit` must publish `DeallocationStack`, and the guest stack wants a real
  reserve plus guard page rather than the fully-committed block the initial stack now gets,
  BEFORE the kernel can trust those bounds. Sequence it in that order.

- **An amd64 boot could lose EVERY 32-bit guest at one address, during 64-bit loader init.
  NOT REPRODUCIBLE since 2026-09-20 — 0 in 29 boots. Kept because the mechanism was never
  proven and it was the confounder behind several of this lane's verdicts**
  (2026-09-20). Symptom: `PASS=0 EXITED=16`, every ladder guest AND every stock applet
  exiting `0x80`, including ones that never otherwise fail (winmine) and the non-GUI parity
  guests (sqlite32). MEASURED 13 / 12 / 0 PASS across three boots of *identical* media
  (`/tmp/vibeos-cli/20260920-144018-b4b1xg22-smoke-amd64`).

  It is not sixteen application failures, it is one boot-level failure. The tells:

  - A wipeout boot's serial log is **~14k lines against ~180-250k for a healthy one** — the
    applets die instantly, they do not run and then crash.
  - **Not one `[FEX-WOW64C]` line is printed all boot.** `BTCpuProcessInit`,
    `Wow64LdrpInitialize`, `entering guest` — the entire CPU-layer bring-up is absent, so
    no guest ever begins executing i386 code.
  - But the CPU layer is not where it breaks. `ldrinit.rs` prints `[LDR] WoW64 process:
    init routines done, loading wow64.dll` immediately before it loads wow64.dll, and a
    wipeout boot prints that **0 times against 30 in a healthy one**. So every guest dies
    during ordinary 64-bit loader init — the import walk and the dependencies' DllMain —
    *before* WoW64 enters the picture at all. The missing `[FEX-WOW64C]` output is the
    consequence, not the cause. (Checked the Rust: `dll/ntdll/ldr/ldrinit.c` is
    cohab-flipped and inert, so its `DPRINT1`s never fire and their absence means nothing.)
  - Every victim faults at the **same address in the same instruction**:
    `rip=addr=000007ffb7030093`, which is ntdll.dll (64-bit, based 0x7ffb7000000) at
    `RtlpUnwindInternal+0x13` — `movq %r9, 0x70(%rsp)`, the first store into the 0xA88-byte
    frame that the immediately preceding `subq $0xa88, %rsp` just allocated. So the 64-bit
    loader is already unwinding an earlier exception, and runs out of stack doing it.
  - `pci.pDeskInfo=0000000000000000` on every one, against real per-process values in a
    healthy boot.
  - The kernel then prints `user dispatch FAILED: pre-fault of stack page ... = 0xc0000005`
    and the process is terminated `0x80`, which is why the original cause never surfaces.

  Distinct from the guard-page entry above in WHERE it lands: those are **guest** faults
  (`80000001`, 3 per boot, rotating victim, late, after real GUI work); this is a **host**
  64-bit fault at process startup, identical across every guest, with `guardpage=0` all boot.

  **But symbolicating both sides says they are one bug, not two.** The healthy-boot applet
  deaths land at `rip=7e018191`, which is i386 `ntdll.dll` (based 0x7e000000) at
  `RtlpAllocateMemory@8+0x11` — `pushl $0x0`, a stack store. The wipeout deaths land at
  64-bit `RtlpUnwindInternal+0x13` — `movq %r9,0x70(%rsp)`, also a stack store, the first
  into a frame `sub rsp,0xA88` just opened. Same shape on both sides of the 32/64 boundary:
  a store just past the stack limit that the kernel declines to grow, `0xc0000005`, then
  `user dispatch FAILED: pre-fault of stack page`. Both therefore point at
  `MiCheckForUserStackOverflow` — which is the function the reverted change was aimed at,
  and is a second reason to re-derive that verdict rather than leave it standing.

  Note also how much stack growth a WoW64 process needs before it can do anything: the guest
  image (winmine.exe) declares `SizeOfStackReserve=0x100000` with `SizeOfStackCommit=0x1000`,
  so the 64-bit loader runs its whole import walk and every dependency's DllMain on a stack
  that starts at ONE committed page and must grow by guard-page fault the entire way. If
  growth is unreliable, a WoW64 process is the most exposed thing in the system.

  **It stopped happening, and the honest summary is "not reproducible", not "fixed".**
  Measured across four media: 1/3 before the TEB32 work, then 0/5, 0/4, and a 24-boot
  `farm` soak at **0/24** — pooled **0 of 29** post-change. Under the 1-in-3 originally
  quoted that outcome has probability ~1e-5, so the rate really did drop; but the
  disappearance coincides with `7261000a5110` (GdiBatchCount read from TEB64 on the
  user-callback path), and coincidence across media is not attribution. A bisect would
  settle it and has not been run. The mechanism that FITS: a guest's TEB64 GdiBatchCount
  is a field nobody writes, so a garbage count drives user-mode callbacks to flush a batch
  that does not exist, and recursive callbacks eating a megabyte of native stack is exactly
  the observed signature. Plausible, unproven; do not repeat it as the cause.

  A cheap way to detect a recurrence without a ladder run, which is how the 24 boots were
  scored: a wipeout kills the LADDER guests at startup, long before the desktop marker, so
  a plain 45s `farm` boot shows it as **zero `[LDR] WoW64 process` lines**. No `--ladder`
  needed, and n=24 costs half an hour instead of four.

  **The rate was not 1 in 3 and I should not have quoted one.** That figure came from a
  single 3-boot run (13 / 12 / 0). A 5-boot run on current media saw **none**: PASS
  13 / 14 / 11 / 14 / 14, EXITED 1 / 0 / 3 / 0 / 0, every boot reaching the desktop
  (`/tmp/vibeos-cli/20260920-154737-z5kel22e-smoke-amd64`). Pooled that is 1 in 8, and the
  two runs are not even the same media — `7261000a5110` (the TEB32 GDI-batch fix on the
  user-callback path) landed in between and is a live candidate for having reduced it.
  Nothing here is yet enough to claim a fix; it IS enough to say the number was soft, and
  it is the third estimate of mine today that needed softening once n grew.

  **Why it matters beyond itself:** a bug that wipes the entire applet population, in
  exactly the units this lane measures, will confirm whatever you were hoping to see. It
  already did once — see the corrected postmortem above. Until it is understood, no
  single-boot amd64 applet comparison means anything; use `--repeat` and report the
  distribution, not a boot.

  **Usable baseline as of 2026-09-20**, for whoever grades the stack-growth fix: 5 boots,
  PASS 13 / 14 / 11 / 14 / 14, and `pre-fault of stack page` failures 4 / 4 / 6 / 4 / 4 —
  present in EVERY boot, which makes that count the sharper before/after signal than the
  PASS count, since it does not depend on which applet happened to be unlucky.

- **`taskmgr.exe` EXITED = 0xc0000005 on one boot — OPEN, likely a flake or a distinct
  bug** (2026-09-20). Seen on the stack-fix boot while charmap (the applet that fix
  targeted) passed; the PREVIOUS boot was the exact reverse — taskmgr PASS, charmap dead.
  The applet population has been race-dependent throughout this investigation, so this is
  **not** filed as a regression from the stack change, but it is also not yet shown to be
  unrelated. Needs a repeat boot before anyone theorises.

- **`charmap.exe` faults in its OWN code under WoW64 — FIXED 2026-09-20** by the guest
  stack sizing above; it was never charmap's bug. Original text kept because the
  misattribution is instructive: the fault WAS in charmap's own image, which made it look
  app-specific, but the faulting function was the compiler-inserted `__alloca` probe and
  the real defect was the stack it was probing. (2026-09-20). Previously
  masked by the CLIENTINFO cascade, visible now that it is fixed. First fault is
  `eip=0040335a` — charmap's own image at base 0x400000, NOT user32 — reading `001efd84`
  while `esp=001fea38`, i.e. ~0xCB4 BELOW the stack pointer: a stack-guard shape rather
  than a bad pointer. **Now the LAST failing applet** (8 PASS / 1 EXITED) and, since the
  amplifier fix, it reports its real cause: `EXITED = 0xc0000005` instead of `0x80`, and
  takes 1 fault instead of 55. Its SEH chain is genuinely corrupt at that point —
  `fs:[0]` points 0x44 bytes below its own `esp` at an all-zero record — so "why is
  charmap's registration chain abandoned" is the question, and the stack-guard read
  suggests looking at stack depth first.

- **`NtGdiFlushUserBatch` reads and writes the 64-bit TEB for a 32-bit guest — FIXED
  2026-09-20**, as part of the GDI-batch root cause above; it was not latent after all, it
  was the bug. Kept because the reasoning below is how it was found, including the wrong
  turn.
  (2026-09-20). `win32k-rust/src/gdibatch.rs:625` does
  `let pTeb = NtCurrentTeb() as *mut ntkrnl_sys::ddk::_TEB` — the **64-bit** TEB — then reads
  `GdiBatchCount`, `GdiTebBatch.HDC` and `GdiTebBatch.Buffer` from it and walks the batch
  records. A WoW64 guest's gdi32 batches into **TEB32** (`GdiTebBatch` at 0x1D4, 32-bit
  layout), which is a different structure at a different address, so for a guest this reads
  one TEB and the writer used the other.

  **It is REACHABLE, and it is the leading suspect for the `pDeskInfo` corruption above.**
  Being commented out of `win32ss/win32u/win32u.spec:158` only removes the user-mode syscall
  export; `win32k-rust/src/main.rs:890` registers the same function as the `BatchFlushRoutine`
  callout, and every arch's trap path calls it through `KeGdiFlushUserBatch` — amd64 at
  `ntoskrnl-rust/src/ke/amd64/traphandler.rs:210`, i386 at `ke/i386/traphdlr.c:1854`, arm64 at
  `ke/arm64/trap.c:200`. None of that logs.

  The two halves that make this a WoW64 bug are both checked in the tree: the guest's gdi32
  does `pTeb->GdiBatchCount++` on ITS TEB (`win32ss/gdi/gdi32/include/gdi32p.h:517`), i.e.
  TEB32, and the kernel tests `TEB64->GdiBatchCount` before flushing. So **a 32-bit guest's
  GDI batch can never be flushed**, and "the GDI TEB batch was never flushed" is the exact
  title of field manual **§9.41**, whose four recorded symptoms include *CLIENTINFO
  corruption* and *a user32 `ValidateHandle` AV* — which is the entry above, precisely. The
  adjacency that made §9.41 corrupt CLIENTINFO exists on TEB32 too: `GdiTebBatch.Buffer` runs
  0x1D4..0x6B4 and `Win32ClientInfo` starts at 0x6CC.

  **Not proven, and the missing step is named:** each append gdi32 makes is bounded by
  `Offset + cjSize <= GDIBATCHBUFSIZE`, so an unflushed batch should stop batching rather
  than overflow. Either some path skips that check, or the corruption arrives another way.
  §9.41 was a wrong OFFSET (0x74 vs 0x1740) and is gated by `scan_magic_offsets`; this would
  be the same class through a wrong TEB, which that scanner cannot see.

- **`NtGdiGetCharABCWidthsW` BUGCHECKs 0x1E when called from a 32-bit guest — ROOT-CAUSED 2026-09-25**: it took `FontGDI` from the LFONT instead of `TextObj->Font` (see the SHObjectProperties entry); the guard below stays as defense. Original report:
  (2026-09-20). Found by the drain floor above: with the applet phase actually observed, a
  `--ladder` boot does not merely storm with first-chance faults, it **dies**.

        [WOW64>] NtGdiGetCharABCWidthsW(0x109f) enter arg0=4d010381 arg1=00000013 arg2=00000001
        *** BUGCHECK: 0x0000001E (0xffffffffc0000005, 0xfffff88004ea0204, ...)
        ***   P2 = win32k.sys+0x79204

  The syscall logs `enter` and never returns. `win32k.sys+0x79204` symbolicates to
  **`NtGdiGetCharABCWidthsW+0x1a4`**, which is the store in an inlined exclusive-unlock
  sequence:

        cmpw $0x0,0xc(%r12)
        movl $0x0,0x8(%r12)      ; <-- faults (write)
        lock xadd $-1,0x10(%r12)

  **REPRODUCED 2/2, and the second sample names the pointer.** The other boot died in the
  same function at `win32k.sys+0x79254` = `NtGdiGetCharABCWidthsW+0x1f4`:

        test %r15,%r15 / je      ; FontGDI null-checked, and it passes
        mov  0x50(%r15),%rax     ; rax = FontGDI->SharedFace   -- loads fine
        mov  (%rax),%r12         ; <-- faults: SharedFace->Face

  which is the source line `face = (*(*FontGDI).SharedFace).Face;`. So **`SharedFace` is
  invalid** — `FontGDI` itself is null-checked and survives, and nothing checks `SharedFace`
  (upstream does not either). That also explains the FIRST sample rather than competing with
  it: at `+0x1a4` the faulting store is an inlined exclusive unlock through `r12`, and `r12`
  is exactly what this instruction loads. One boot faults loading the bad `Face`, the other
  gets a garbage `Face` and dies later unlocking through it. Same root.

  *(Supersedes a lead recorded here earlier the same day. From sample 1 alone I argued the
  page must be readable-but-not-writable — the `cmpw` at `0xc(%r12)` completes and the
  `movl` at `0x8(%r12)` faults — and reasoned toward a user-side-view/delta mix-up. The
  read/write asymmetry is still a correct reading of sample 1, but a garbage `Face` explains
  both samples and needs no read-only mapping, so the delta theory is dropped. Note the
  BUGCHECK's P3/P4 cannot settle read-vs-write directly: P3 is a kernel address, not the
  0/1/8 an AV's ExceptionInformation[0] would carry.)*

  **The class is 24 more derefs, and the file is already inconsistent about it.** Surveyed
  by walking `freetype.rs` for `(*(*FontGDI).SharedFace)` / `(*SharedFace)` uses with no
  `SharedFace.is_null()` within the preceding 25 lines:

        guarded = 2   unguarded = 24

  One of the two guarded sites (`IntGetOutlineTextMetrics`, ~line 1013) predates this work,
  which is the point: somebody already knew `SharedFace` can be NULL and guarded exactly one
  place. The other 24 are the same latent BUGCHECK, reachable from guest GDI calls
  (`NtGdiGetGlyphIndicesW`, `NtGdiGetTextMetricsW`, the ExtTextOut paths, ...). Only the
  measured one is fixed — each needs its own unlock/free cleanup path, so this is a real
  batch of work rather than a sed. Worth a scanner before hand-editing 24 sites.

  **Do not record this as WoW64-specific either.** It was observed twice from 32-bit guests
  and not natively — but no native app in these boots calls `GetCharABCWidths` at all, so
  that is an absence of a trigger, not evidence of an arch split. `RealizeFontInit()`
  returning a TEXTOBJ whose FONTGDI has no realized `SharedFace` is the thing to check
  first, and it is not obviously a 32-bit question. Preceded by a long, healthy run of guest GDI text calls
  (`NtGdiGetOutlineTextMetricsInternalW`, `NtGdiGetGlyphIndicesW`) that all return correctly,
  so this is one syscall's unlock path, not general GDI breakage. Reproduce with
  `VIBEOS_LADDER_DRAIN_FLOOR=700 ./vibeos-cli smoke --arch amd64 --ladder`; it lands in the
  last applet launched (wordpad, PID 852 in that run), which is consistent with wordpad being
  the text-metrics-heaviest guest. **Not yet root-caused**, and the arch split is untested —
  do not record it as amd64-only, which is the mistake this registry has already made three
  times on WoW64 findings.

- **Nothing caps `/tmp/vibeos-cli` retention — OPEN tooling gap** (2026-09-06). A long soak
  writes one artifact dir per round with a ~7MB `serial.log` in it, forever; `pytest`'s
  `tmp_path_factory` keeps its last 3 runs but the CLI keeps everything. Found while the host
  root filesystem hit 100% (the actual weight that day was ~400G of sibling worktrees plus a
  148G sccache, NOT this — but the gap is real and unbounded). The failure mode is worth
  recognising: at 0 bytes free the agent harness cannot run ANY shell command, because it
  creates each command's stdout file under `/tmp` first, so even `true` fails with ENOSPC and
  it presents as a broken tool rather than a full disk. Retention belongs in the CLI, not in
  each lane remembering to clean up (`vibeos-cli prune` is delete-only and manual).
  **Half-addressed 2026-09-12 by the MP promotion, and the half that is left is still
  the retention cap.** A failed boot no longer depends on `/tmp` surviving: `Run.finalize()`
  copies the serial, summary, cpustate and screenshot to `~/vibeos-flake-farm/fails/auto/`
  and parks the kernel, HAL and symbol index content-addressed under `kernels/<sha12>/`
  (one copy per build, not per flake), because a preserved serial without its index is
  unreadable after the next build. So evidence loss on a RED run is closed. Everything a
  GREEN run writes is still uncapped and still grows forever.

- **MP kernel promoted to the DEFAULT boot on all three arches — 2026-09-12, user's call.**
  `DefaultOS=` now selects `ntkrnlmp.exe` + `halmp.dll`; both UP entries still ship as the
  opt-out. The UP:=smp1 collapse criterion had been met on all three arches since July; what
  held the flip afterwards was boot reliability, and that number did not move in months of
  soaking. Rates over `conformance.jsonl` rows with `arch`, `smp == 1`, `mp_kernel` TRUE —
  **quote that filter**, since the file also holds 169 arm64 rounds predating the `mp_kernel`
  field whose kernel is unrecorded, and blending them in yields a meaningless 2.72% spanning
  two kernels: **amd64 0/203, i386 1/204 (0.49%), arm64 22/1375 (1.60%)**. The decision was
  explicitly to stop measuring and get more eyes on it: a thousand identical soak rounds
  sample one condition a thousand times, while fleet usage samples many for free. **Do not
  propose more soak rounds as a precondition for anything here.** Two narrow signatures in
  `triage.KNOWN_FLAKES` made the gate retry these once (~1-in-42 → ~1-in-3200 disruption;
  both retired 2026-09-25, below);
  across all 1375 MP rounds `0x1E` and `0xE3` are the only codes that have ever appeared, so
  the pair is exhaustive, not merely sufficient. **#43 (`0xE3`) FIXED 2026-09-24**: fastfat's
  C shim tested `IRP_CONTEXT_FLAG_DISABLE_DIRTY` (0x1) as the WAIT bit, so FCB acquires ran
  non-blocking and `FatCommonCleanup` released a page fault's FCB lock; field manual, "0xE3
  RESOURCE_NOT_OWNED (#43) was a fastfat shim". Measured with delay-load ON (which made it
  hit 4 in 5 kmtest boots): 10/10 clean after. **Re-measured 2026-09-25: 102 arm64 MP kmtest
  rounds at `e2928af9a3c`, ZERO bugchecks of any code**, against 0xE3 in 18/336 and 0x1E in
  10/336 in the same corpus before the fix. Chance of 0/102 at those rates: 0.4% (0xE3) and
  4.5% (0x1E). At #39's long-run fleet rate of 1.3% it is 26%, so #39 is UNREPRODUCED, not
  proven fixed. One capture's stack ran through `ExReleaseResourceForThreadLite` under
  `NtFlushKey`, which fits the same corruption. **Both KNOWN_FLAKES entries are RETIRED**
  (the 0xF4 precedent), so a recurrence now BLOCKS as a fresh `BUGCHECK 0x1E`/`0xE3` with its
  capture preserved. That is how the next lead arrives, and the signatures are kept in
  triage.py's retirement note for a re-add.

- **No scanner catches a C `ASSERT` that VANISHES in translation — CLOSED (2026-09-06) by
  `tools/scan_dropped_asserts.py`.** 89 findings baselined; gated at 0 NEW. It pairs each C
  `FORCEINLINE` under `ntoskrnl/include/internal/**` + `ntoskrnl/mm/ARM3/*.h` against its
  same-named Rust twin and reports FEWER asserts on the Rust side. Covered by 11 unit tests
  (`tools/test_scan_dropped_asserts.py`) and a bad/good control in `test_scanner_controls.py`.
  Registered in the **deferred** source partition, not `source-early` as scoped below: the
  registry's contract is that unaudited new scanners stay deferred, and five of the six early
  scanners currently have a live rglob-vs-cargo-delete defect. It prunes `target` during the
  walk and tolerates `OSError`, so promotion is a deliberate audit somebody should do, not a
  drive-by. The residual 89 are real owed work, mostly the MM working-set lock family
  (`MiLockWorkingSet` and kin, 4-7 asserts each, all shipping zero) and the PTE writers.
  Original finding, kept because the reasoning is what made the scanner correct:
  `MI_MAKE_HARDWARE_PTE_USER` was translated without the
  four ASSERTs the C original opens with (`ASSERT(ProtectionMask & MM_PROTECT_ACCESS)` and
  friends, commented *"check that we are not setting valid a page that should not be"*). On
  arm64 those asserts were the only thing between a zeroed protection mask and a `Valid = 1`
  descriptor the MMU rejects, so their absence turned a loud precondition failure into a pager
  reporting success forever with interrupts masked — an unobservable freeze. (Restoring them
  did NOT fix #39: with the guard in place and silent, the wedge still reproduces at the same
  site. The dropped assert is a real defect that hid a real bad PTE; it was not the wedge's
  cause.)
  `scan_assert_to_ub` does **not** cover it: that gate catches an assert lowered to
  `unreachable_unchecked()` (present and wrong), not one deleted outright.
  **Do not file this as "clone `scan_dropped_prints`".** That scanner's oracle is rustconv's
  trailing C comments in the same `.rs`, and the two implementations that carried this bug
  (`ntoskrnl-rust/src/ARM3/helpers.rs`, `ntkrnl-sys/src/inline/mm.rs`) are HAND-WRITTEN bridge
  modules with no trailing C — a same-file diff has nothing to compare and reports clean on the
  exact function that cost the boot. The oracle has to be cross-file: the C `FORCEINLINE` in
  `ntoskrnl/mm/ARM3/miarm.h` (and `ke.h`) paired by name against its Rust twin.
  Shape precedent already exists — `scan_extern_decl_drift.py`, `scan_extern_arity.py` and
  `verify_signatures` all pair C declarations against Rust. Unlike `verify_signatures` this one
  reads **no generated headers** (source `.h` + source `.rs` only), so it is not build-dependent
  and is a `source-early` candidate (see the promotion caveat above).
  Two scoping rules: the finding must be *"fewer asserts than the C"*, not *"no asserts"* (a
  partial drop is the same class; #39 dropped 4 of 4, the easy end — and the shipped scanner
  does find real partial drops, e.g. `MiDereferencePfnAndDropLockCount` at 7 of 8); and it need not generalise
  beyond the `miarm.h`/`ke.h` FORCEINLINE surface, where hardware-precondition asserts live and
  the bridge modules deliberately mirror C names. Closest model: `scan_invented_constants`.

- **Every 32-bit guest is refused by CSR — the shared section can't map into a WoW64 process**
  (2026-09-11, wow64-amd64 lane, **CLOSED 2026-09-13**). Closed by `c277b0000d4` (M19 — CSR/LPC
  works and a 32-bit binary reaches PASS) and `a154628337b` (`BasepCreateThread` marshalled;
  **all nine WoW64 probe guests PASS on amd64**). The diagnosis below is kept because it is the
  reasoning that found the bug, and because the "fixing the downstream marshallers first would
  change nothing" sequencing argument is the reusable part. Read it as history, not as state. A trace probe on a passing `smoke:amd64` boot
  (`/tmp/vibeos-cli/20260911-234915-smoke-amd64/serial.log`) shows the real CSR/LPC blocker, and
  it is **not** a marshalling bug. Per 32-bit guest process, in order:
  `NtSecureConnectPort '\Windows\ApiPort' => c0000041` (PORT_CONNECTION_REFUSED, handle NULL),
  then `NtRegisterThreadTerminatePort => c0000008` (INVALID_HANDLE, because there is no handle).
  The server-side reason is already instrumented and names itself:
  `[CSRSRV] AttachSharedSection: map at fixed base 000007fffebf0000 FAILED st=c0000018`
  — **STATUS_CONFLICTING_ADDRESSES**, 12 times across **6 distinct pids**, with no successful
  attach in the whole boot. `CsrSrvAttachSharedSection` (`csrsrv/server.rs:403`) passes the
  GLOBAL `CsrSrvSharedSectionBase` as the *requested* base to `NtMapViewOfSection`, demanding
  that every client map the section at the same address csrss itself got. In a WoW64 process
  that address is occupied, so the attach fails, `AllowConnection` stays 0, and
  `NtAcceptConnectPort` is called with Accept=FALSE — which is the `c0000041` the guest sees.
  **What is NOT yet established:** what actually occupies that VA in a WoW64 process. That is
  the next measurement, not an assumption.
  **The predicted second blocker is now OBSERVED and FIXED (2026-09-12).** It was recorded here
  as derived-not-observed; once the double-attach fix let guests get further, wow64's own
  pre-existing diagnostic fired **25 times per boot**:
  `[WOW64] CSR connect: shared section ABOVE 4GB (base=7fffebf0000 …) — guest-invisible!`
  `CSR_API_CONNECTINFO.SharedSectionBase` is narrowed with a plain `(ULONG)` cast in
  `wow64_manual.c`, so `0x7fffebf0000` truncated to `0xfebf0000` and kernel32 dereferenced
  that. **Root cause was not wow64 but the kernel's based-section ceiling:** `SEC_BASED` means
  one system-wide address for every mapping, `MiFindEmptyAddressRangeDownBasedTree` searches
  downward from `MmHighSectionBase`, and `mminit.rs` set that to
  `MmHighestUserAddress - 0x800000` — ~2 GB on i386 but ~8 TB on a 64-bit kernel. Fix: cap the
  ceiling at the 32-bit user ceiling (`0x7F7EFFFF`), since a based address must be nameable by
  every client the section can reach. No-op on i386; csrss is the only `SEC_BASED` creator in
  the tree, so the address space given up is unused. Verified: ABOVE-4GB warnings 25 → 0,
  desktop still reached. NOTE this did **not** fix `hello32.exe`, which still exits
  `0xc0000005` exactly as it did before — that is a separate, still-unexplained fault, and the
  MM fix neither caused nor cured it.
  **Consequence for sequencing:** the `PORT_MESSAGE` 24→40 marshaller and the CSR payload work
  are real (`NtRequestWaitReplyPort => c000000d` fires 5x, exactly as the kernel's
  `DataLength + sizeof(PORT_MESSAGE) > TotalLength` check predicts) but they are DOWNSTREAM.
  Fixing them first would change nothing, because no guest ever gets a port handle. Note the
  kernel checks the header size BEFORE `ObReferenceObjectByHandle` (`lpc-rust/send.rs:407-420`),
  which is why a guest with no connection still reports `c000000d` and not `c0000008`.
  The desktop boots green throughout — this is confined to the 32-bit guest path.
- **ExResource waiter-count "race" — ROOT-CAUSED 2026-09-26: a 10 ms timing assumption in the
  test, not the ERESOURCE implementation.** Every recorded failure at the three moving sites
  (`ExResource.c:432`, `:469`, and `:570` in our own `TestSharedReleasePreemption`, 197 hits in
  the 2026-09-25 gates) read exactly `= 0, expected 1`. Each site starts a thread with a 10 ms
  timeout, takes `STATUS_TIMEOUT` as "it blocked", and asserts the waiter count at once. But
  ERESOURCE is RUST (`sdk/lib/rust/ex-rust/src/resource.rs`; `ntoskrnl/ex/resource.c` is
  commented out of `ntos.cmake` -- the entry below saying no port had taken it over is stale),
  and it increments `NumberOf{Exclusive,Shared}Waiters` under the resource lock BEFORE blocking,
  on a uniprocessor guest. So a 0 can only mean the new thread had not been SCHEDULED yet --
  which on TCG under host load (load average 22-27 that night) takes longer than 10 ms. It went
  from arm64-only to i386 as load rose, and it livelocked the land queue: each red batch
  started another 3-build/4-boot gate, re-creating the load. Fix: `WaitForWaiters()` waits
  (bounded, ~5 s) for the counts the test is about to assert, then asserts them UNCHANGED; an
  implementation that never counts a waiter still fails, one that over-counts fails at once.
  The two ExResource failures the baseline KEEPS (`:441`/`:488`, renumbered from 409/455 by the
  helper) are NT6 `ActiveCount` bookkeeping checked against the test's WS03 branch -- upstream
  semantics, accepted, not this race.
- **`NpfsFileInfo.c:57` "bistable" — ROOT-CAUSED 2026-09-26: the conformance PARSER, not npfs.**
  npfs returns `Information = 158` (Windows: 154 -- it subtracts the padded
  `sizeof(FILE_NAME_INFORMATION)`, 8, where Windows uses `FIELD_OFFSET(.., FileName)`, 4; upstream
  C does the same) on EVERY boot, EVERY arch. But `:56`'s message prints a raw `0xFFFF` WCHAR that
  often eats its newline, gluing `:57` onto the end of the line, and `parse_kmtest_serial.py`
  matched identities only at the line start. So `:57` surfaced on some boots; the 596-round soak
  banked it `assert?` on amd64/arm64, and the i386 seeding never saw it at all -- until a quiet
  host flipped the byte and i386 went "NEW failing" in the gate. Parser now finds identities
  anywhere on a line; `:57` is a stable `assert` on all three arches. The 4-byte npfs divergence
  itself is real Windows-vs-ReactOS behaviour and still OWED.
- **`conformance:arm64` wedges on an ExResource waiter-count race — OPEN, NOT a code regression**
  (2026-09-06, found by the wow64-amd64 lane, NOT owned by it). Two independent arm64 runs on
  the same tree both failed the kmtest sweep, and the line number MOVED between them:
  `ExResource.c:432` in gate `20260906-221252`, `ExResource.c:469` in the standalone control
  run. Both sites are the identical construct — start a thread that must BLOCK acquiring an
  ERESOURCE, assert it times out, then assert a waiter count at that instant:
  `StartThread(&ThreadData…, &Timeout, TRUE, TRUE)` → `ok_eq_hex(Status, STATUS_TIMEOUT)` →
  `CheckResourceStatus(Res, …)`. A deterministic regression fails at the same line every time;
  a race wanders, which is what this does. Both runs dropped exactly one assertion (399 → 398),
  and i386/amd64 conformance PASSED in the same gate — only arm64 (TCG, the slowest target)
  fails, on a host that was saturated all evening.
  **Read the verdict carefully, because it overstates itself twice.** The headline is
  `RATCHET REGRESSIONS (63)`, but 55 of those are `in baseline (OK) but never launched` — the
  failing assertion wedges the sweep, so every test after ExResource in sweep order cascades.
  One real regression is reported as 63. The headline naming ExResource IS correct (it is the
  first thing that broke and it did produce results, 399 → 398); it is the *count* that
  misrepresents the blast radius, not the file. Separately the assertion text offers "media was
  not built with VIBEOS_PAYLOAD=kmtest" as the first explanation — that branch was checked and
  ELIMINATED: `opt:kmtest_all`, `mkramdisk` and `payloads` are all present in the gate's
  `build:arm64`.
  Eliminated as direct causes: no commit in the last 80 touches `ntoskrnl/ex/resource.c`, and
  no Rust port has taken ERESOURCE over (grepping `sdk/lib/rust` for
  `ExAcquireResourceExclusiveLite`/`ExfAcquireResourceExclusive` finds only consumers and
  bindgen types), so AGENTS.md's "check for a Rust port FIRST" does not redirect here. **That
  eliminates the obvious candidate, not the class:** a waiter count asserted at an instant moves
  if anything shifts timing — scheduler, timer resolution, KE wait paths, or an unrelated
  translation that changed a hot path's cost. Do NOT weaken or quarantine the assertion; it is
  asserting something true.
  **Update 2026-09-11 — a third and fourth site, and a GREEN run.** `conformance:arm64` on a
  quiet host (queue empty, no conductor) came back `launched=91 ok=88 bad=3 regressions=[]`,
  i.e. PASS, in 74.5s. ExResource itself reported `OK pass=399 fail=2`, failing at
  `ExResource.c:409` and `ExResource.c:455` — **two further line numbers**, so four distinct
  sites across four runs (432, 469, 409, 455), all the same block-then-assert-a-waiter-count
  construct. That settles the race question: a deterministic regression does not move four
  times. It also sharpens the mechanism — the baseline **already accepts two failing ExResource
  assertions**, so the ratchet does not trip on a failure here at all; it trips only when the
  *pass count* moves (399 → 398), which is what a wedge mid-sweep does. Under load a third
  assertion flips and the sweep stops producing. Still OPEN, still not owned by the wow64 lane,
  and still do not weaken the assertion.
  **Update 2026-09-26 — it reaches i386 now, and it LIVELOCKED the merge queue.** The
  2026-09-06 entry recorded "i386/amd64 conformance PASSED in the same gate — only arm64";
  that is no longer true. Four consecutive batches went red on `conf:i386` between 00:02
  and 00:30, and **nothing landed for the 45 minutes from 23:45:28 to 00:30:18** — from the
  green gate `20260925-234529-6v80rtj1` that landed `05c6ab03b3c1` to `46f777f0d5a`.
  **It then broke on its own, and the first version of this entry claimed otherwise.** That
  version said "nothing landed between 23:45 and at least 00:33"; it was written at 00:34:15,
  four minutes AFTER `46f777f0d5a` landed at 00:30:18. The error was checking whether MY
  submission had landed (it had not, and still had not) and generalising that to "nothing
  landed" without looking at master's tip for other lanes. The stall is therefore
  LOAD-CORRELATED AND SELF-LIMITING, not a true livelock: as lanes drained, load fell 27.3 →
  6.07 (1-min) and the queue started moving again without anyone touching the ratchet. That
  is a materially better conclusion than the one it replaces, and it changes the advice —
  wait it out rather than reaching for a lever. The i386 counts
  wander exactly as the mechanism above predicts, across four runs of essentially the same
  tree: `pass=879 fail=2` (the baseline-accepted state), then `878/3`, `876/5`, `878/3`.
  The reported "regressions" wander with them — `ExResource.c:432`, `:469`, `:570` and
  `NpfsFileInfo.c:57` in varying combinations, 1 to 4 per batch. A deterministic regression
  does not vary its own identity set between runs on the same tree.
  **The trigger is measured, not guessed: host load average 27.3 (15-min 22.5) with six
  lanes queued and a conductor gating.** That is the load-sensitivity the entry already
  names, arriving on the second-slowest target once the host is busy enough.
  **The queue shape makes it self-sustaining, which is the new part.** A red batch is
  bounced, the next conductor immediately starts another 3-build + 4-boot batch, that load
  re-trips the race, and so on — a batch every ~10 minutes, none landing. The retry logic is
  not wrong (it classifies correctly as `CONFORMANCE_FLAKE`) and the ratchet is not wrong
  (the assertion is asserting something true); they are simply both correct while the queue
  makes no progress.
  **What NOT to do: do not weaken the ratchet or quarantine the assertion to drain the
  queue.** The cheap correct lever is LOAD — an agent that stops adding builds and boots
  while the queue is stalled is doing more for it than one that keeps working — and the
  queue recovers by itself once load falls, so waiting is a real option.
  **Root-caused 2026-09-26 by the msvcrt-homebrew lane, and it is the TEST, not ERESOURCE:**
  each of `:432`, `:469`, `:570` starts a thread with a 10 ms timeout, treats
  `STATUS_TIMEOUT` as proof it blocked, and checks the waiter count immediately — on TCG at
  load 22-27 the new thread has not been scheduled yet, so the count is legitimately 0. Fix
  is a bounded `WaitForWaiters()` that waits for the asserted counts and then asserts them
  UNCHANGED, so never counting a waiter still fails. Assertions not weakened.
  Two corrections to this entry's older text, both verified independently here: ERESOURCE
  **is** Rust (`ex-rust/src/resource.rs`, 27 `#[no_mangle]`; `ntoskrnl/ex/resource.c` is
  commented out of `ntos.cmake`), so the line claiming "no Rust port has taken ERESOURCE
  over" was stale and pointed readers away from the live code. And `:409`/`:455` are NOT part
  of this race: they read `Res->ActiveCount = 1, expected 2`, fire 129 times each on all
  three arches, and are the baseline's two accepted failures — the test asks for the WS03
  bookkeeping model (`ActiveCount == 2`) while the implementation is NT6-style
  (`ActiveCount == 1`, `ActiveEntries++`) in BOTH the C (`resource.c:958-971`) and our Rust
  (`resource.rs:651-655`). An upstream version/semantics mismatch, not a translation bug, and
  the allowance is correct rather than masking something. I first attributed it to our
  translation and relayed that; the refutation was one `grep -a` away (see the field manual's
  binary-grep entry — plain grep prints NOTHING for `resource.c`).- **Fleet disk exhaustion interrupted landing — OPEN, admission slice implemented locally**
  (2026-09-06; owner: bughunt/Codex). The shared WSL filesystem reached zero
  available bytes during `land`'s tooling preflight; the failure summary itself
  then failed with `ENOSPC`. User reclaimed space (163 GB available afterward).
  `disk-status --sizes` and cooperative build/tooling disk reservations now expose
  footprint and refuse new work without headroom. Disk refusals are inconclusive
  infrastructure failures, not lane regressions. Not a physical quota: legacy
  lanes, other writers, and larger-than-estimated growth remain unaccounted for.
  Follow-up: measure growth and tune reservations, establish bounded retention, and make cleanup
  targeted, host-locked, and safe for active lanes and paid translation output.
  Preserve enough headroom to record failure evidence. Do not treat manual
  cleanup as prevention or silently skip checks when space runs out.

- **Binary-lint corpus depends on existing artifacts — OPEN** (2026-09-06;
  owner: bughunt/Codex). `tools/scan_ud2.py::find_rust_binaries` discovers existing
  `.sys/.dll/.exe` files beneath the selected build directory and filters with
  `nm`; missing optional outputs are not an inventory failure. Claude flagged
  the ARM64 corpus gap; the exact missing binary/example is still requested.
  Follow-up: derive required scan inputs from the build/media contract, report
  expected/scanned/missing inventories, fail on missing required inputs or failed
  symbol/disassembly tools, and test identical verdicts in fresh and populated
  worktrees. Preserve separate kernel scanning and staging-copy exclusions.
  This is separate from the source-partition migration; no corpus fix is claimed
  by moving lints earlier.

- **`test_build_invalidation.py` bounced conductor batches — FIXED (2026-09-06), root cause was the HOST CLOCK.**
  (2026-09-06). Two different tests in the file have failed the same way:
  `test_generated_rust_tracks_directory_input_add_edit_remove[files-depfile]` (line 522) with
  `ninja explain: restat of output leaf older than most recent input .../generated.rs`, and
  `test_lockfile_only_git_pin_change_requires_explicit_edge[False]` (line 472). Both are the
  *no-op rebuild* assertion — `assert fixture.invocations() == before` — failing with
  `Left contains one more item: 'leaf'`, i.e. a spurious rebuild.
  **It is load-dependent, not random**: the file passed 3/3 run in isolation, and failed
  roughly 1 run in 3 inside `check-tooling`, which runs 27 checks on a machine already carrying
  the build fleet. That is precisely when the conductor runs it, so it preferentially fires
  where it costs the most — it already cost one two-lane batch a TOOLING_FAIL, which the
  conductor then split to find a culprit that did not exist, and it blocked this lane's `land`
  preflight three times running.
  **Neither the fixture nor cargo was at fault. `CLOCK_REALTIME` on this host steps BACKWARDS
  ~2.4s every 32s**, so an output is stamped older than the input it was just built from. See
  the consolidated entry below and the field manual, "The HOST CLOCK steps backwards, so an
  output is older than its own input".

- **FIXED pending gate (2026-09-06): the LAST five varargs-discarding printf entry points, plus
  four format-string bugs the fix itself created.** Follow-on to the freeldr `DbgPrint2` work.
  Same defect: stable Rust cannot DEFINE a C-variadic, so each of these took a fixed parameter
  list and printed its format string LITERALLY, dropping every argument. Converted to the
  slot-capture technique: freeldr `TuiPrintf` (tui.c:39) and the `printf`/`TRACE_printf` aliases
  over it, `rosload_shims.rs`'s cross-importlib `printf`, ntoskrnl's `KdIoPrintf` (kdio.c:574),
  and classpnp/disk's `TracePrint`.
  **Ported ONCE into a new `va-slots-rust` crate** rather than a third and fourth copy — the
  VA_SLOTS/VA_STAGE bounds are only safe because they were MEASURED, so each copy is a chance to
  guess them. rlib, no dependencies, exports no `#[no_mangle]` symbol so `drvtrace-shim` keeps the
  LLVM-module isolation that is its whole reason to exist (§9.99).
  **The fix makes a second bug, and this is the transferable part:** while a printer emits its
  format verbatim, passing message TEXT as the format is harmless — a `%` comes out as a `%`. The
  moment formatting is real, those sites consume slots nobody pushed. Four were already in the
  tree, all of them divergences from the C (which uses `printf("%s\n", Text)` at every one):
  `directui.rs` `UiInfoBox`/`UiMessageBox`/`UiMessageBoxCritical`, and `rosload_shims.rs`'s `puts`,
  which handed an arbitrary string straight to `TuiPrintf` as its format. All four routed through
  `%s`. Also clamped `KdIoPrintf`'s truncation path: the C is `Length = (ULONG)_vsnprintf(...)`
  with no check, so a >512-byte format hands `KdIoPrintString` a length of 0xFFFFFFFF — tui.c:52
  clamps for the identical idiom, and a 4GB overread inside the debug printer is the last thing
  anyone would successfully debug.
  Validated on a two-TU i386 host harness (variadic caller, non-variadic callee — the real shape):
  11/11 including `%*s`, the 11-slot widest live format, NULL `%s`, the over-run-reads-zeros
  degradation, and the truncation clamp. Field manual entry extended with the grep that finds the
  data-as-format sites before you flip a verbatim printer.

The committed home for open bugs (was: agent MEMORY.md, which other agents never see).
Field-manual § pointers hold the full writeups. **Update on state change.**

- **RVA arithmetic on a PE mapped as DATA crashed the WoW64 layer — FIXED** (2026-09-05).
  `Wow64pPatchTransitionSlot` runs on every successful `NtMapViewOfSection` and guarded only on
  DOS magic + PE signature + `Size >= 0x1000`, none of which separates an IMAGE mapping
  (sections at RVAs) from a raw DATA mapping of the same file (sections at FILE offsets). ws2_32
  maps msafd.dll as data before loading it properly, so the export walk read RVAs into raw file
  bytes and **wow64.dll took the access violation on the guest's behalf** — which is why the
  faulting address was inside no guest module. The discriminator is the ViewSize: an image view
  equals SizeOfImage, a data view is the file page-rounded (msafd: file 0x55400 -> 0x56000
  observed, vs SizeOfImage 0x49000). `Wow64pFindExport32` was also written for one trusted
  input (our own ntdll32) and now sees every module a guest maps, so all of its RVAs are
  range-checked and the name compare is bounded. Symptom to recognise again: `rip` inside
  wow64.dll after a map, with a guest that never reached its DllMain.
- **32-bit `socket()` returned WSAEINVAL because the WSH helper was unstaged — FIXED,
  BOOT-VERIFIED** (2026-09-05, ladder run `20260905-233102-smoke-amd64`: `socket()` succeeds,
  `select()` is reached and issues four AFD IOCTLs where there were previously zero). With the
  RVA-on-a-data-mapping crash above fixed, msafd.dll loaded for real and `socket()` STILL
  failed with `0x2726`. The cause was the SAME
  dynamic-edge blind spot one hop deeper: msafd reads `HelperDllName` from the registry and
  `LoadLibraryW`s the WSH_* helper (msafd-rust `misc/helpers.rs`), which nettcpip.inf declares
  as `%SystemRoot%\System32\wshtcpip.dll`. That DLL was BUILT and never staged into SysWOW64,
  `LoadLibraryW` returned NULL, and helpers.rs returns WSAEINVAL straight out of WSPSocket.
  Visible in the log as two `NtQueryAttributesFile => c0000034` immediately after msafd's
  DllMain. **The one-hop fix is what disguised it**: staging the provider made every symptom of
  the previous bug vanish while the module it loads at runtime was still missing, because the
  error surfaces a full frame from the absent DLL. `wow64-closure.py` now keys the derivation
  by READER (ws2_32 reads LibraryPath/PackedCatalogItem, msafd reads HelperDllName) and follows
  dynamic edges transitively. Lesson: a runtime-load edge is never known to be the last one.
- **AFD IOCTLs were REFUSED because async I/O was unsupported under WoW64 — FIXED,
  BOOT-VERIFIED** (2026-09-06, ladder `20260906-132106-smoke-amd64`: **9/9 guests PASS**,
  marshal32 `0xbb04` -> `0x600d`, AFD refusals 4 -> 0, IOCTL statuses `4x c0000002` ->
  `3x SUCCESS + 1x 00000102`). This was the E25 blocker: msafd always passes an Event because
  AFD is inherently async, and `Wow64pRefuseAsync` refused on sight, so the E25 marshallers had
  never once run. The refusal was CORRECT when written — the only IOSB wow64.dll could give the
  kernel was a host one on its own frame, and pending on that is the dead-frame-IOSB class
  (field manual §9.36). Fixed the way that comment already said was owed: the kernel writes the
  GUEST's 8-byte block, which outlives the thunk by construction. The machinery existed already
  (M15b, built for `NtReadFile`/`NtWriteFile`); the IOCTL family was simply never wired into it.
  Three obligations in `IopDeviceFsIoControl`, each **silent** rather than loud if skipped:
  probe the guest block at 8 bytes not 16, mark the IRP, and narrow the write on the fast-I/O
  path — which completes inline, never builds an IRP, and so can never see the flag. Marshalled
  IOCTLs still complete synchronously and the reason is the PAYLOAD, not the IOSB: the widened
  input, WSABUF array and AFD_POLL_INFO shadow are host-heap blocks the thunk frees on the way
  out, and METHOD_BUFFERED copies SystemBuffer back into that shadow AT COMPLETION. The
  unmarshalled path has no shadow and is genuinely async now. Blocking has one trap worth
  remembering: msafd's `SockEvent` is a SYNCHRONIZATION event, so the thunk's own wait consumes
  the very signal `MsafdWaitForAlert` is about to wait for — missing the re-signal is a guest
  hang with no error printed anywhere. `IOCTL_AFD_SEND_DATAGRAM` was the last refusal in this
  family and is now marshalled too (see the entry below). Still OPEN: a guest `ApcRoutine`.
- **A guest `ApcRoutine` is still refused, but NOT for the reason previously recorded — OPEN**
  (2026-09-06). This entry said "no 64->32 user-APC dispatch"; that was written from the
  refusal's own comment and is wrong. The guest-entry half EXISTS and is load-bearing:
  `Wow64pBuildLoaderApcFrame` (`wow64.c`) builds a `KiUserApcDispatcher`-ABI frame and
  `Wow64pKiUserApcDispatcher32` is resolved out of ntdll32 at startup, and that pair runs on
  EVERY guest thread start (`Wow64pGuestThreadStart`), not on some experimental path. What is
  actually missing is two smaller things. First, INTERCEPTION: the kernel's `KiInitializeUserApc`
  builds a 64-bit `UAPC_FRAME` and points the trap frame at the 64-bit `KiUserApcDispatcher`, so
  a guest's 32-bit `ApcRoutine` handed through would eventually be `call`ed as native x64 — x86
  bytes executed as x86-64. wow64.dll needs to substitute its own host-side thunk as the
  `NormalRoutine`, carrying the guest routine and context as arguments. Second, the frame builder
  hardcodes `Wow64pLdrInitThunk32` at `[E+0]`; it needs the routine as a parameter. Neither is
  architecture. Nothing on the ladder needs it today (msafd uses Events, not APCs), so it stays
  a loud refusal by choice rather than by blockage — but whoever picks it up should not go
  looking for machinery that is already there.
- **FIXED (2026-09-06): `IOCTL_AFD_SEND_DATAGRAM` was the last refused AFD payload, so 32-bit
  guests could not send UDP at all.** `AFD_SEND_INFO_UDP` is 56 -> 104 and the audit calls it
  SEMANTIC because it embeds `TDI_REQUEST_SEND_DATAGRAM` and `TDI_CONNECTION_INFORMATION`.
  Reading both consumers shrank the job a long way: `AfdPacketSocketWriteData` reads only
  `BufferArray`, `BufferCount` and `TdiConnection.RemoteAddress`, and msafd's `WSPSendTo` fills
  only those plus `AfdFlags` and `RemoteAddressLength`. **`TdiRequest` is written by NEITHER
  side**, so it is carried as opaque zero padding — synthesizing a host-shaped `TDI_REQUEST`
  would invent content nobody supplies and the driver would read it back as though the guest
  had meant it. `RemoteAddress` points at a `TRANSPORT_ADDRESS` (LONG + `TA_ADDRESS`{USHORT,
  USHORT, UCHAR[]}) which is pointer-free and width-identical, so widening the pointer is the
  whole job, exactly as `RECV_DATAGRAM` already does for its `Address`. Three independent
  witnesses agreed on 56/104: the bindgen types, a host layout program, and the audit baseline.
  A NULL `RemoteAddress` is now refused because `AfdPacketSocketWriteData` dereferences it
  unconditionally — a native i386 guest can still walk the kernel through that null deref, which
  is a separate pre-existing hazard. Oracle: marshal32's new sendto rung, whose teeth are not
  that `sendto` succeeded but that the socket comes back IMPLICITLY BOUND — AFD only reaches
  `WarmSocketForBind` by dereferencing `RemoteAddress` for `Address[0].AddressType`, so AF_INET
  plus a real port proves the driver read the family through the pointer we widened.
- **msafd interprets a FAILED `AFD_SELECT` as success — upstream wart, OPEN but no longer
  reachable here** (2026-09-05, re-checked 2026-09-06). The `select()` = 5 on a single-socket
  `fd_set` is explained: `WSPSelect` never checks the IOCTL status before walking `PollInfo`, so
  a failed IOCTL leaves the guest's own REQUEST mask sitting in `Handles[].Events` and the loop
  reads it back as the RESULT, `FD_SET`ting sockets that were never ready. It returns the count
  rather than `SOCKET_ERROR`, though it does set `*lpErrno` — so a caller checking only for
  failure sees five ready sockets. Byte-faithful to the C, so this is upstream behaviour and is
  left alone. With the refusal above fixed the precondition is gone (the rung now returns 0 and
  passes), but the wart would resurface behind any future IOCTL failure.
- **EVERY CSR call from a WoW64 guest fails — `NtRequestWaitReplyPort => c000000d` — OPEN**
  (2026-09-05). Found while chasing the socket() bug above; it is NOT its cause (there is no
  LPC call inside the socket() window at all — that lead was wrong). It is unconditional and
  the arithmetic is exact: `PORT_MESSAGE` is **24 bytes on i386 and 40 on amd64** (the
  `ClientId` union is 2 pointers, and `ClientViewSize` is a `SIZE_T`), and the syscall has NO
  manual marshaller — the log line prints without `[manual]`, so it takes the flat
  pointer-widening auto thunk. The guest sets `TotalLength = DataLength + 24`; the kernel
  (`lpc-rust/src/send.rs`) checks `DataLength + sizeof(PORT_MESSAGE) > TotalLength`, i.e.
  `DataLength + 40 > DataLength + 24`, which is **always true**. Hence 100% failure, every
  call, deterministically. `NtConnectPort`/`NtSecureConnectPort` are already on the manual list
  and thunk `PORT_VIEW` + `CSR_API_CONNECTINFO`, so the connect succeeds and only the calls
  fail. There is already a workaround papering over one symptom — `WOW64: CreateThread
  continuing without CSRSS thread notify (0xc000000d)`. Fixing the header is mechanical
  (translate 24<->40, payload follows contiguously, rewrite TotalLength); the real work is the
  **payload**, a `CSR_API_MESSAGE` full of 32-bit guest pointers that 64-bit csrss would
  misread, so this needs per-API marshalling and is its own task, not a bolt-on. Same family:
  `NtReplyPort`, `NtReplyWaitReceivePort`, `NtRequestPort`, `NtReplyWaitReplyPort`.
- **FIXED (2026-09-05): freeldr printed NOTHING — all ~2200 `ERR`/`FIXME` sites were silenced
  by a dropped disjunct, and `/DEBUG=` was dead code.** A boot log contained zero `DbgPrint2`
  output; the `[freeldr]`/`[FLDR]` stamps that *do* appear ride a separate hand-added path, which
  is why the gap was never obvious. Three independent defects in `freeldr-rust/src/debug.rs`,
  all in one function: (1) the port kept only the first half of the C's gate
  `if (!(DbgChannels[Mask] & Level) && !(Level & DBG_DEFAULT_LEVELS)) return;` — the second term
  is channel-independent and is what makes ERR and FIXME always-on, so requiring an open channel
  silenced every one of them; (2) the `err:`/`fixme:`/`warn:`/`trace:` level tag was dropped, so
  nothing was distinguishable from anything else; (3) `DbgPrint2`/`DbgPrint` were non-variadic
  (stable Rust cannot *define* a C-variadic) and printed their format string literally, so `%s`
  and `%lu` came out as those two characters. Separately `DbgParseDebugChannels` was a no-op stub
  and is the ONLY caller of `DbgAddDebugChannel`, so no boot option could ever set a channel bit
  — WARN/TRACE were unreachable rather than merely off — and `DebugDumpBuffer` was a stub too.
  All four fixed. The formatter takes varargs on stable by declaring the argument slots as fixed
  parameters and handing the resulting array to `_vsnprintf` as a `va_list`, which works because
  `va_list` is `*mut c_char` on all three targets; **no call site changed** (3038 of them) since
  the callee's arity is invisible to a cdecl/win64 caller. Field manual: "Which of the four is
  silent is a property of the HEADER" (whose freeldr row this corrects) and "A printf-style
  callee can take varargs on stable Rust".
- **UNREPRODUCED since #43's fix (task #39), 2026-09-25: 0 in 102 arm64 MP rounds; flake
  entry retired, see the #43 entry above. History follows. PARTLY FIXED: the arm64 `--smp 1`
  wedge reaches an EL1 user PC;
  exception handling hid the first fault, and the jumper is not yet found.** A 300-round
  soak on fixed media hit it 5 times (1.7%), and every capture showed the same shape: a vCPU at
  **EL1h with `PC=0x10000`**, register file byte-identical across samples, every timed wait
  dead, serial stopped one `DbgPrint` before `CreateProcessW`. `KiSynchronousExceptionHandler`
  sent the same-EL instruction abort to `MmAccessFault`, whose valid-PTE path only bugchecks an
  instruction fetch when the PTE is **U**XN — a user-executable page passes and returns
  `STATUS_SUCCESS`, so the handler eret'd to the same PC forever. AArch64 makes every
  EL0-writable page implicitly PXN, so that retry could never succeed. Fixed: EC 0x21 below
  `MmUserProbeAddress`, or on a permission IFSC, now dispatches as a kernel AV and bugchecks
  `0x1E` with the frame plus a new `[KSTKW]` kernel-stack-word dump. **Still open: who sets an
  EL1 PC to `0x10000`.** The captured frame had `LR=0` and every callee-saved register zero,
  with `X18` a TEB and `X28` a user start-context, so the stack words are the lead. This is the
  last flip blocker for `DefaultOS`. Field manual: "An EL1 instruction fetch on a user page
  loops silently forever". OWNED BY the smp-bringup lane.
  **Correction 2026-09-06:** dispatching the fetch AV did not close the whole path.
  `KiDispatchException` then treated execute faults as data probes, advanced PC by four,
  and could zero registers. The later `0x250000`/zero-register captures therefore cannot
  identify the original jump. The dispatcher now excludes execute faults from that fixup
  and dumps the original integer registers, thread, and process. Fresh patched captures
  (`20260906-220136`, `221936`, `222818`, `223301`, `224348`) now stop at **PC/LR/FP zero**,
  an EL1 instruction-fetch AV (`ESR=0x86000007`) in `lsass.exe`; the semaphore-release frame
  and waiter registers survive. The same-commit control reproduces the old `0x250000`
  signature. Artifacts and exact binaries: `~/vibeos-flake-farm/fails/smp39-20260906-first-fault/`
  and `smp39-20260906-control/`. The earlier fault is exposed; the underlying return/branch
  defect remains open. Next: watch the saved return and record any preceding exception recovery.
  **Update 2026-09-11:** the software-PTE Protection shift is fixed (`bc2c4d3810d`,
  `MM_PTE_SOFTWARE_PROTECTION_BITS` 6 -> 1 on arm64; writers disagreed with the reader, so
  `MmDecommittedPte` came out as the Prototype bit and decommitted faults fell through to
  demand-zero with Protection 0). That removes the SILENT-WEDGE mechanism, not the fault: a
  60-round `--smp 1` soak on top of it still took `BUGCHECK 0x1E` three times, all during boot
  before any kmtest launched. Those three captures are LOST — they were left in
  `/tmp/vibeos-cli`, which is cleaned within days; the durable `conformance.jsonl` rows prove
  the hits and nothing else. Re-run and copy each fail into `~/vibeos-flake-farm/fails/` at
  capture time. A per-thread flight-recorder dump (`flightrec_dump_thread`, wired into the
  bugcheck sidecar) is in the tree to answer whether the crashing thread took an EARLIER
  kernel fault it recovered from -- the last-32 tail window can neither show nor exclude that.
  **Update 2026-09-13 — the hunt has a tool now.** `./vibeos-cli rr` records one boot under
  QEMU's instruction-counted record mode and replays it instruction-for-instruction, with gdb
  instruction-for-instruction. Recording is verified on arm64 `--smp 1` (the MP kernel on one
  CPU — precisely this bug's configuration): boot to desktop in ~90s vs ~40s normally, 10MB
  trace, and a replay whose guest serial matched the recording line for line as far as it ran.
  **But the two things that would close #39 with it do not work yet**: under gdb a replay can be
  run but never STOPPED — stepping, attaching mid-run and even a breakpoint HIT all abort
  QEMU 9.0.2 with `Playback shouldn't have to iowait` — so `reverse-continue` from the zeroed
  frame is unreachable, not merely unattempted; and replays WEDGE — they stop consuming the
  trace about two thirds through, at a near-deterministic point, while burning CPU (verified on
  a correctly finalised trace: read position frozen at byte 2,342,912 of 10,666,166 for ten
  minutes, and at the IDENTICAL offset through two different harnesses -- so it is one specific
  recorded event that replay cannot pass, not a race). A separate,
  now-fixed record-finalisation bug produced TRUNCATED traces that abort rather than wedge;
  the two were conflated for a while.
  So this is not yet a route to "who set PC to 0" — it is a deterministic capture of a
  crashing boot, which is still more than a soak ever gave us.
  Both limitations, their measurements, and what to re-test on a QEMU upgrade are in
  `docs/engineering/qemu-record-replay.md`.
  **Update 2026-10-03 — half of that is fixed, and #39 closed by another route.** A replay
  CAN be stopped: `./vibeos-cli rr replay --break-at <icount>` arms a QMP `replay-break`
  over the harness's own QMP connection and the guest pauses at the requested instruction.
  The constraint is arithmetic, not a QEMU defect. Each `EVENT_INSTRUCTION` in the log means
  "run N more instructions, then read the log again", so only the cumulative sums are legal
  targets; a mid-chunk target aborts with `Playback shouldn't have to iowait (insn total
  <end>/<remainder> left)`, which is QEMU reporting the remainder it was asked to skip.
  `rr breaks <trace>` prints the legal set and `--break-at` snaps forward to the next one.
  So the earlier "even a breakpoint HIT aborts QEMU" reading was a mid-chunk stop, not an
  inability to stop — verified prospectively, with the prediction written before the run:
  the boundary target PAUSED and the mid-chunk target aborted with the exact predicted
  remainder (`insn total 2100388618/624674 left`).
  The **wedge** is still open and is what still blocks replaying a whole boot. It is
  structural, not a guest point: two independently recorded traces diverge at guest line
  1,120 yet stall within ~1% of each other, and each replay passes the other's stall point.
  Log-position quantities cluster tight (trace bytes 0.85%, lines 1.21%) while guest-work
  quantities spread (icount 6.69%, exceptions 23.01%). Not block I/O — all 2,277
  `EVENT_ASYNC` records fall in the first 1.5% of the trace (ramdisk boot).
  And the motivation has moved: **#43's fix retired #39, so rr did not close it.** rr stands
  on its own as a deterministic-capture tool, and its next customer is the #19/#23
  physical-page-reuse cluster rather than this bug.
- **FIXED (2026-09-06): system cursors load, and the residual NULLs are correct behaviour
  wearing an alarming message.** `SYSTEMCUR(ARROW) == NULL, should not happen!!` was firing **78x** per boot (6 system  classes x 13 GUI processes) and kept firing to within 300 lines of `shell ready` —
  `gasyscur[ROCR_ARROW].handle` was NULL for the *entire* boot, so every server-side class
  registered with `hCursor = NULL`. Chain: `co_IntInitializeDesktopGraphics` normally loads
  cursors/icons via `KeUserModeCallback`, but that block is `#if !defined(_USE_DUMMY_PSEH)` and
  `-D_USE_DUMMY_PSEH=1` is on all 5924 i386 TUs, so it is compiled out and deferred to
  `co_IntDeferredCursorsAndIcons()` on a real GUI thread. Instrumentation showed that deferred
  path was entered **exactly once per boot**, on the desktop thread, where its own guard returns
  immediately — so it never ran at all. Fixed by also calling it from `co_int_graphics_check`,
  **after** `co_add_gui_app`: ordering is load-bearing, because `NtUserSetSystemCursor` only
  stores into `gasyscur` for a process that already has `W32PF_CREATEDWINORDC`, and calling
  before that loads every cursor in user mode and then silently drops it in the kernel (the
  syscall just returns FALSE — a first attempt did exactly this and moved the count not at all).
  Result: 78 -> 24, boot green at 37s, and the feared `LoadImageW`/`RtlVirtualUnwind` AV from
  letting the callback finally run did **not** materialise.
  **Read the count as `6 x (processes that missed)`, not as a raw number.** Observed 78 (13x6),
  24 (4x6) and, on a later boot with no win32k change in between, 30 (5x6) — every value seen so
  far is an exact multiple of the six cursor classes. So movement in units of 6 is just how many
  GUI processes happened to register before the deferred load on that boot, and only a
  **non-multiple of 6** would mean something structural changed. Worth knowing before someone
  reads 24 -> 30 as a regression and goes looking for one; verified here that neither the
  origin/master merge nor the varargs work touched the win32k cursor path at all. **The residual 24 is 4 processes x 6
  classes that register system classes without ever passing through `co_int_graphics_check`, so
  the hook cannot fire for them**: every one of the 24 is emitted strictly *before* the load
  (serial lines 1263-1513 vs the load at 1552), and there are zero in the 750 lines after it.
  **CLOSED 2026-09-06 — the residual is correct behaviour, and the message was the bug.**
  An instrumented boot logged every `UserRegisterSystemClasses` with its pid and whether cursors
  had loaded, plus which of the three bail branches the deferred load took. It resolved the one
  question that mattered — whether those early processes ever draw anything — and they do not.
  That boot showed 42 (**7**x6, the rule holding at a new multiple): pids 0x8c (csrss) and six
  background services (eventlog, dcomlaunch, srvsvc, wuauserv, spooler…), each of which pulls in
  user32/comctl32 for message-only windows and never renders. The deferred load then ran
  (two `BAIL=desktopthread`, then `LOADING`), and **every process that draws visible UI
  registers after it with `loaded=1` and takes the real cursor** — winlogon (0x9c, line 1802),
  msgina's status dialog, explorer (0x238, line 1899, `_tWinMain` at 1910). Zero NULLs after the
  load, as before.
  So there is no gap to close, and the Tier-1 DUMMY_PSEH pointer that used to sit here was
  wrong twice over: `co_IntInitializeDesktopGraphics` is already Rust (no C `#if` reaches it),
  and the block is skipped because the callback would run on the system desktop thread, whose
  process has no `KernelCallbackTable` — a constraint DUMMY_PSEH resolution does not touch.
  Fixed by making the diagnostic honest instead: `SYSTEMCUR(ARROW) == NULL` is only reported
  when `gbDesktopCursorsAndIconsLoaded != 0`, i.e. when it genuinely should not happen. Before
  the load it is the expected state, and 42 lines/boot of `should not happen!!` for something
  that must happen is a diagnostic that trains people to ignore it.
  Not a translation defect: every link (`SYSTEMCUR_ARROW`, `LoadSystemCursors`'s
  `!gpsi || !gpsi->hIconSmWindows` guard, `co_add_gui_app`) was diffed against the retired C and
  is byte-faithful.
  Still genuinely skipped and NOT covered by this entry: `MenuInit: skipping co_IntSetupOBM
  (DUMMY_PSEH)` — a separate OBM bitmap path, unmeasured here.
- **FIXED (2026-09-06), was: blocks every lane's landing ~50% of the time — `tools/test_build_invalidation.py`
  is flaky.** Landed tonight (`2e016c6595f` "Cover native source and header invalidation layers",
  in as `41589edd0c53`). Measured 5 consecutive full-file runs on an IDLE host: **3 failed, 2
  passed** (see the fuller 6-run table below). No
  `pytest-randomly`/`xdist` installed, so it is not test-order shuffling; every failure is in the
  "did ninja re-run `leaf`" assertion class, in both directions, which points at mtime/timestamp
  sensitivity rather than a missing dependency edge. Every test passes in isolation in ~5s.
  **Frequency and signature, 6 full-file runs on an idle host: 4 failed, 2 passed, and FOUR
  DIFFERENT tests** — `test_generated_rust_tracks_directory_input_add_edit_remove[directory-depfile]`,
  `test_native_inputs_reach_linked_rust_executable[source-only-source-S]` and
  `[source-only-header-c]`, and `test_lockfile_only_git_pin_change_requires_explicit_edge[False]`.
  Never more than one per run. **The dominant signature is a SPURIOUS `leaf` rebuild**: 3 of the 4
  are `Left contains one more item: 'leaf'` — an extra `leaf` invocation on a `compile()` that
  should have been a no-op — and only the first was the opposite (a rebuild that failed to
  happen). `leaf` is the one target with extra inputs (`include_str!("../payload.txt")` +
  `env!("SCHEMA")`), so it is the most-dependent target in the fixture and the one most likely to
  trip a timestamp comparison.
  Impact while it was open: `vibeos-cli land` runs `check-tooling` as a pre-queue gate, so it
  aborted landings that had nothing to do with it, and the conductor runs the same check inside
  its gate (a red batch there costs 3 builds + 4 boots of fleet load before the split exonerates
  the innocent lanes).
  **ROOT-CAUSED AND FIXED (2026-09-06). It was never the fixture, cargo, load, or a missing
  dependency edge: `CLOCK_REALTIME` on this host is NON-MONOTONIC.** `ninja -d explain` names it
  outright — `restat of output leaf older than most recent input ../leaf/src/main.rs` with the
  input **1.86s NEWER** than the output built from it. Measured directly: over 120s of
  `CLOCK_MONOTONIC_RAW` the host lost **9.57s** of realtime in **4 backward steps of ~2.4s**, and
  `CLOCK_MONOTONIC` runs **8.09% fast**. Reproduced independently by the control-plane lane.
  Cause is a **runaway feedback loop between `systemd-timesyncd` and the kernel's own slew**:
  timesyncd steps the clock back, the kernel slews harder to catch up, the slew saturates at its
  `tick` ceiling of **11000** (nominal 10000) where it has no headroom left and can never
  converge, so timesyncd steps it back again — with the poll pinned at its **32s floor**
  (`timedatectl show-timesync --all`, `PacketCount=3966`) and its own `NTPMessage` showing our
  clock 3s ahead of the server. WSL2 adds a second time authority on the same clock:
  `/proc/cmdline` carries `hv_utils.timesync_implicit=1`.
  **My first attribution was wrong and is worth recording as the trap:** I read the oversized tick
  as the kernel compensating for a miscalibrated guest TSC, on the strength of `dmesg`'s
  `Adjusting tsc more than 11% (4482949 vs 4434052)`. That line is real but **incidental**, and it
  points at hardware — which is how you end up costing the fleet a `wsl --shutdown` for something
  a `systemctl` fixes. Two pieces of evidence settle it: `tick` at 11000 is **saturated, not
  merely large** (my own 10773 sample was mid-adjustment, not the steady state), and after
  stopping timesyncd the kernel returns to `tick 10000` with `freq` ≈ **-0.14 ppm** — a genuinely
  11%-off TSC would have left a large residual correction behind, and there is none.
  **This is bigger than the flake, and the other direction is a FALSE GREEN**: a source edited
  just after a step gets an mtime OLDER than an artifact built before it, so ninja/cmake/cargo
  skip the rebuild and a stale artifact ships. Rule out the clock first for any "impossible"
  staleness on this fleet.
  Fix landed in the fixture: `CLOCK_MONOTONIC` carries the same NTP slew as `CLOCK_REALTIME`, so
  realtime advancing *less* than monotonic isolates a step and nothing else — no sleep, retry, or
  threshold tuning, honouring `write()`'s "No sleeps or retries hide invalidation failures". On a
  proven step the fixture **skips loudly with the measured step size** instead of reporting a
  dependency-edge failure it cannot observe; a real invalidation bug fails on EVERY run, not only
  when the clock steps, so the guard cannot mask one. Validated 4/4 full-file runs green (was 4/6
  FAILING), 3-4 skips per run, rotating to a different test each time so nothing goes permanently
  unexercised.
  **Host clock FIXED (2026-09-06), by the control-plane lane with the user's approval:**
  `systemctl disable --now systemd-timesyncd` — one service, persistent across WSL restarts, and
  **no `wsl --shutdown`**, so no lane lost context and nothing paid was at risk. Verified here
  independently over 90s: REALTIME/MONOTONIC/MONOTONIC_RAW agree to five decimals, **0 backward
  steps**, `adjtimex tick 10000 freq -0.138 ppm offset 0`. What remains is a constant ~1.55s
  absolute offset (the frozen residue of the last backward step), stable within noise and harmless
  for build correctness since every mtime carries it equally. Two things measured NOT to work:
  switching `current_clocksource` to `hyperv_clocksource_tsc_page` (slew is reapplied), and the
  WSL kernel cmdline.
  **The fixture guard STAYS** — it defends against a step from any cause and re-arms the moment
  timesyncd is ever re-enabled. One documented limit: it catches STEPS, not slew; a pure frequency
  error moves both clocks together and slips past it.
- **FIXED pending validation (2026-09-06): `ks.sys`, `pcix.sys` and `ramdisk.sys` imported a
  nonexistent `ntoskrnl.dll`.** Found by the freeldr debug channel on its FIRST boot after being
  un-silenced — `PeLdrLoadImage('...\drivers\ntoskrnl.dll') failed` ->
  `PeLdrpLoadAndScanReferencedDll() failed` -> `PeLdrScanImportDescriptorTable() failed`, three
  times per boot, immediately after `ks.sys` and `ramdisk.sys` load. Cause:
  `#[link(name = "ntoskrnl", kind = "raw-dylib")]` appends `.dll` to the link name, so the
  descriptor names a file that does not exist. rustc appends it **unconditionally** unless
  `modifiers = "+verbatim"` is set, so `name = "ntoskrnl.exe"` alone is NOT the fix — it yields
  `ntoskrnl.exe.dll` (confirmed on the binaries and in a boot log before being corrected). The
  correct form, which the 15 working crates all use, is
  `name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim"`. The bare form existed in
  exactly the 3 crates that carry
  raw-dylib kernel DATA imports (ramdisk's own comment, "the first cohab driver to import kernel
  data", explains the copy path). Confirmed against the binaries: those 3 `.sys` carry both
  `ntoskrnl.dll` and `ntoskrnl.exe` strings while healthy drivers carry only `.exe`. **Not
  cosmetic** — everything on that descriptor is a DATA import (`ExEventObjectType`,
  `ExSemaphoreObjectType`, `IoFileObjectType`, `KeLoaderBlock`), so its IAT slots are never
  filled and any path reading them gets an unresolved pointer; the boot stays green only because
  those paths are not exercised (ramdisk is not the boot device here). 9 sites in 8 files fixed.
  Field manual: "raw-dylib in a freeldr-loaded DLL hangs the boot". **Owed lint DELIVERED
  (2026-09-06): `tools/scan_rawdylib_linkname.py`**, wired into `vibeos-cli check` with an EMPTY
  baseline. It resolves each name the way rustc does and reports DOUBLE_EXT (`ntoskrnl.exe` with
  no `+verbatim` -> `ntoskrnl.exe.dll`), WRONG_EXT (bare `ntoskrnl` -> `ntoskrnl.dll`, the
  original bug) and NO_EXT. It deliberately does NOT demand `+verbatim` everywhere — bare `ntdll`
  -> `ntdll.dll` is correct, and a scanner flagging it would teach the next agent to write
  `ntdll.dll.dll`. Validated by replaying `2a89c05d763^` through it: flags all four sites naming
  `ntoskrnl.exe.dll`, and the pre-repair spelling naming `ntoskrnl.dll` — the two filenames that
  actually appeared in the boot log. Positive AND negative controls in
  `tools/test_scanner_controls.py`.
- **`Cannot open winsta` x229 per boot is winlogon thread teardown — ROOT-CAUSED, benign, OPEN**
  (2026-09-05). Instrumented boot: 228 of 229 come from PID 0x9c (winlogon), all with
  `hWinSta = NULL` and `STATUS_INVALID_HANDLE` (not an access denial), all from one call site —
  `UserCloseClipboard()`, reached from `ExitThreadCallback` (win32k `main.c:798`), which calls it
  **unconditionally for every exiting GUI thread**. Byte-identical in the C, so faithful. Two
  takeaways: the ERR is upstream false-alarm noise on a cleanup path (~10% of the boot log), and
  the number means **winlogon tears down 228 GUI threads during boot**, which is worth a look in
  its own right.
- **`Sending to Suspended Thread Msg 121` x333 per boot — benign, OPEN** (2026-09-05). `0x121` is
  `WM_ENTERIDLE`, which a modal loop sends purely to say it is idle; dropping it with
  `STATUS_UNSUCCESSFUL` costs nothing. `IsThreadSuspended` and both its accessors
  (`KTHREAD.SuspendCount`, `PsGetThreadFreezeCount`) were diffed against the C and are faithful.
  Noted because winsta.rs ties this exact message to the DUMMY_PSEH cursor path historically.
- **freeldr's `FREELDR_HEAP_VERIFIER` is DEAD in Rust — OPEN** (2026-09-04). `heap.c:25` defines
  it **unconditionally**, so the C always compiles the redzone allocation, the redzone writes
  and the three `FrLdrHeapVerify` checks. The translation turned it into
  `#[cfg(feature = "FREELDR_HEAP_VERIFIER")]` at five sites in `freeldr-rust/src/heap.rs`, and
  the feature is not declared in that crate's `Cargo.toml` (`default = []`, only `uefi` and
  `rosload` exist) — so all five are permanently off. It is self-consistent (alloc and free
  both skip the redzone), which is why nothing has failed, but freeldr has shipped with its
  heap verifier silently disabled. Fix is to declare the feature and enable it by default,
  which needs a boot behind it since it changes every freeldr allocation's layout.
- **A green smoke on amd64 can HIDE a freeldr assertion storm** (2026-09-05). `DbgBreakPoint`
  is a real `brk #0xF000` on arm64 and an EMPTY STUB on i386/amd64, so the same failing
  assertion wedges one arch and merely prints on another while the boot reaches the desktop
  marker and the verdict says PASS. amd64 printed the bad-tag assertion 38 times — once per
  file loaded — under a green smoke. When touching freeldr assertions, grep the serial log for
  `Assertion`, do not read the verdict. Field manual, "The same-named constant with a DIFFERENT
  value per driver".
- **`ASSERT!` is FATAL in the boot loader, print-and-continue in the kernel** (2026-09-04).
  `ntkrnl_sys::ASSERT!` calls `RtlAssert`, and there are two of them: the kernel's
  (`rtl-diag/src/assert.rs` — prints, capped at 20, returns) and freeldr's
  (`freeldr-rust/src/debug.rs` — prints, then `DbgBreakPoint()`). On arm64 that breakpoint is a
  real `brk #0xF000` (`arm64_debug.rs:48`) and wedges the boot; on i386/amd64 it is an empty
  stub (`frldr.rs:719`) so the same assert prints and sails on. So an assertion's severity
  depends on the LINK, not the crate — and `uefildr-cargo/build.rs` shows the loader link pulls
  in `freeldr-rust`, `cmlib-rust`, `cnt-rust-lib`, `cport-rust` and `hal-rust`, of which
  `hal-rust` is ALSO in the kernel link. Any future tool that makes assertions live must read
  that link set rather than filter by crate name.
- **75 newly-visible same-name/different-value constants — baselined work queue** (2026-09-04).
  `scan_const_contradictions` could only parse integer literals, so every
  `u32::from_*_bytes(*b"..")` pool tag was invisible to it. Now evaluated; the backlog includes
  `TAG_CM` at four values inside `config-rust`, `TAG_HW_RESOURCE_LIST` at five inside
  `freeldr-rust`, `TAG_HAL` at three, and byte-reversed pairs in `pcix-rust`, `win32k-rust`,
  `fltmgr-rust` and `classpnp-rust`. One of this family wedged the arm64 boot (field manual,
  "The same-named constant with a DIFFERENT value per driver"). Burn down per crate.
  > **Pool-tag tier CLEARED, 770 -> 731** (2026-09-05). Six constants, 15 wrong definitions,
  > every one evaluated against its C macro rather than eyeballed. Two were real defects, not
  > accounting noise: `config-rust` ALLOCATED the hive `ViewLock`/`FlusherLock` in `cminit.rs`
  > under one `TAG_CMHIVE` and FREED them in `cmapi.rs` under another (`ExFreePoolWithTag`
  > mismatch on the registry path, 4 values for one macro); and classpnp's
  > `DICTIONARY_SIGNATURE` was WRITTEN with the byte-reversed value while
  > `TestDictionarySignature` compared against a function-local shadow holding the correct one,
  > so every dictionary validity check failed. `TAG_CM` was wrong at 7 of 14 sites, `TAG_USTR`
  > 3 of 13, `TAG_DIB` and `TAG_FAT_IO_CONTEXT` 1 each. Fixing `TAG_USTR`/`TAG_DIB` also
  > repaired latent CROSS-CRATE mismatches (rtl-strings vs ntdll/win32k; dibobj vs
  > dib_helpers). 3 further entries cleared by another lane's `ICIF_SET_SIZE_VARIABLE` fix
  > arriving in the same merge. Remaining 731 are Tier 2 semantic constants (~25: oleaut32's
  > `VT_*_PRIO` table, samsrv, `PARTITION_FAT_16`, `SP_UNTAGGED`) and Tier 3 debug/trace enums
  > (~32 cdrom/disk `TRACE_*`, serial `DPFLTR_*`) which corrupt only log filtering.
  > **Tier 2 (semantic) CLEARED** (2026-09-05): 10 constants, 16 wrong definitions, each
  > checked against its SDK header. Three had real behaviour behind them.
  > `SRB_FUNCTION_STORAGE_REQUEST_BLOCK` (srb.h `0x28`) was WRITTEN into `srbEx.Function` as
  > `0x10` and TESTED against `0x16` in the same file — the SRB-Ex branch could never be
  > taken; a third copy in `disk.rs` invented `0xF0`. `RTL_ENCRYPT_OPTION_*` had all three
  > values PERMUTED in one block (`SAME_PROCESS=1, CROSS_PROCESS=0, SAME_LOGON=3` vs the
  > header's `0/1/2`) — the recall failure a seal cannot catch, because the SET is right and
  > only the slots are wrong. A `VK_DBE_CODEINPUT`/`NOCODEINPUT` shadow held `0xF8`/`0xF9`,
  > which are the real `ENTERIMECONFIGMODE`/`FLUSHSTRING`: not wrong numbers but two
  > DIFFERENT keys aliased. Plus `SAM_SERVER_INITIALIZE` (both copies wrong; `0x04`),
  > `PARTITION_FAT_16` (`0x06` is `PARTITION_HUGE`) and `SP_UNTAGGED` (wrong in three files).
  > **And 10 entries were FALSE POSITIVES, annotated not fixed**: oleaut32's `VT_*_PRIO`
  > ladders look like one table shifted by one, but variant.c declares TWO function-local
  > `enum coerceprio`s that reuse the same member names with different values, and both Rust
  > ladders match their own enum member-for-member. Now carry
  > `// nolint: const-contradiction (function-local C enum)` with the reasoning, so the next
  > agent does not re-investigate. **The baseline is a work queue, not a defect list** —
  > classifying is as much of the job as fixing.
- **Constants that are INVENTED and say so — FIXED, and now gated** (2026-09-05). Found while chasing
  `SP_UNTAGGED` into `disk-rust`. Tree-wide, 127 `const` definitions carry `// placeholder`,
  `// typical`, `// guess` or `// adjust as needed`, and `disk.rs:617` has a whole block under
  the header *"Constants used by the function (values typical of Windows DDK; adjust as
  needed)"*. These are not drift from a real value — they are numbers the model made up, and
  they compile. Two were live in that one block: `SRB_FUNCTION_STORAGE_REQUEST_BLOCK = 0xF0`
  (real `0x28`, and not any `SRB_FUNCTION_*`) and `SP_UNTAGGED = 0xFFFFFFFF` (real `(UCHAR)~0`
  == `0xFF`; the `((ULONG)~0)` one is the neighbouring `SP_UNINITIALIZED_VALUE`). This is a
  DISTINCT class from `scan_const_contradictions`, which can only see a name that disagrees
  with ITSELF — a constant invented once, consistently, is invisible to it. A gate is cheap
  here because the sites are self-labelled: grep the annotation, resolve each against its
  header, and fail on any that survives. Worth doing before the class grows.
  Done: `tools/scan_invented_constants.py` (wired into `vibeos-cli check`, baseline
  `tools/lint-baseline/scan_invented_constants.txt`) scrapes the self-accusing comment, then
  resolves each name against a NARROWLY scoped oracle — the C module the crate translates plus
  the SDK headers — and classifies MISMATCH / AMBIGUOUS / UNRESOLVED. A repo-wide catalog was
  deliberately not used: it collides on short names and its findings are overwhelmingly false
  (§9.113b). 209 self-labelled sites, 29 MISMATCH, all fixed; **11** remain, none of them a
  mismatch. The interesting half is that ~65 of the labelled constants were CORRECT — the
  comment was the only thing wrong — which is why the annotation alone was never a defect list.
  What the 29 were: fastfat `VIRTUAL_VOLUME_FILE = 1` where `fatprocs.h`'s `_TYPE_OF_OPEN`
  starts at 1 and 1 is `UnopenedFileObject`, so every later decode of the virtual volume file
  said "not open"; fastfat `TAG_VPB` frozen as `0x20425056` (`" BPV"`, invented) against
  `nodetype.h`'s `'vtaF'`, a tag no free could ever match; cabinet's LZX decoder sizing its main
  tree with a function-local `LZX_NUM_CHARS = 0` while the decode loop split literal-from-match
  on the correct file-scope 256; scsiport numbering `CmResourceType*` from 0, which shifts every
  resource type down one and collides `CmResourceTypePort` with `CmResourceTypeNull`; disk's
  `SRB_EX_DATA_TYPE_SCSI_CDB16 = 0` (real `0x40`) and a PERMUTED
  `FAILURE_PREDICTION_SMART`/`_IOCTL` pair; `SRBEX_DATA_SCSI_CDB16_LENGTH` frozen at 32 where
  the C is `20 + sizeof(ULONG) + sizeof(PVOID)`, i.e. correct on 64-bit and four bytes long on
  i386 — now computed. Plus 156 `TRACE_LEVEL_*`/`TRACE_FLAG_*` declarations across
  cdrom/disk/classpnp normalised to `storswtr.h`: the flags are an ORDINAL enum (0..17) that the
  translation had spread across ~40 distinct bitmask-looking values, and even the two `glue.rs`
  "canonical" copies were off by one. Those are log-filtering only — the ReactOS
  `StorDebugPrint` marks the flag `UNREFERENCED_PARAMETER` — but they now have one oracle.
- **Translations that ABANDONED a function mid-body — the shipping two are FIXED** (2026-09-06).
  Found by the invented-constant sweep, not by the sweep's own findings: a preamble of 22
  placeholder constants turned out to be marking a body the model had given up on. The class
  announces itself in English, in the tree, which makes it the cheapest defect there is to find.
  - `scsiport` `SpiParseDeviceInfo` — C `scsiport.c:2634-3014` is ~380 lines walking the
    device node's registry values; the Rust enumerated them and stopped, leaving the model's
    note *"Actually let's rewrite the whole block with explicit structure. I'll rebuild."* The
    loop still terminated (`ZwEnumerateValueKey` runs out), so it was a silent no-op parse, not
    a hang: **scsiport applied no registry tuning at all** — MaximumLogicalUnit,
    InitiatorTargetId, the four Disable* knobs, DriverParameters, MaximumSGList,
    NumberOfRequests, and the whole ResourceList/Configuration Data resource walk. Hand-
    translated; all 22 placeholders resolved to the crate's real substrate. Two constants were
    genuinely absent and are now in `glue.rs` beside their siblings: `CmResourceTypeDeviceSpecific`
    (cmtypes.h:14) and `REG_FULL_RESOURCE_DESCRIPTOR` (cmtypes.h:33) — the latter localized
    rather than imported because only `hal_bindgen_{i386,amd64}` carry it and **there is no
    `hal_bindgen_arm64`**, so the hal-island import that rustc suggests breaks arm64.
  - `classpnp` `ClasspPowerUpCompletion` — *"[Error handling block same as C, omitted for
    brevity but will be included]"*, ~55 C lines. START_UNIT failures were never interpreted
    and **never retried**: no `InterpretSenseInfoWithoutHistory`, no watchdog-aware timeout,
    no `RetryPowerRequest`. Two further defects fell out of writing it: the surviving guard
    compared the RAW `SrbStatus` instead of `SRB_STATUS()` (srb.h:149 masks off
    `AUTOSENSE_VALID|QUEUE_FROZEN`, so a success carrying sense data read as a failure), and
    the state machine had collapsed C's `goto ClasspPowerUpCompletionFailure` into a plain
    re-dispatch — the label sits *inside* `case PowerUpDeviceStarted`, past its error check,
    so the goto path was wrongly running that check. Now carried by an explicit flag.
  - **Two preserved C quirks, deliberately not "fixed":** `sizeof(REG_FULL_RESOURCE_DESCRIPTOR)`
    is `sizeof(9)` == 4, because the identifier is a macro and not a type; and
    `CM_RESOURCE_DMA_8` is **0**, so `Flags & CM_RESOURCE_DMA_8` is always false and the
    8-bit DMA branch is dead upstream too.
- **disk-rust: 4 hand-inlined asserts that HANG instead of RtlAssert — OPEN** (2026-09-06).
  `scan_ud2`'s arm64 driver sweep flags three branch-to-self spins in `DiskIoctlIsWritable`
  (`disk.rs:10780`). They are **not** the §9.99 CC/sig-mismatch fold the scanner files them
  under — reading the instruction before the trap, as the manual's triage table says to, gives
  `brk #0xf001` (the deliberate breakpoint), then `b .`. These are the PAGED_CODE / CHECK_IRQL
  expansions written out longhand as `DbgPrint` + `int 0x2C`/`brk #0xF001` + `loop {}`. That is
  the exact pattern the 2026-07-26 "kernel asserts that hung the box" pass replaced with
  `ntkrnl_sys::ASSERT!`/`NT_ASSERT!` → `RtlAssert` (loud, non-fatal, matching C under `-DDBG=1`);
  these 4 survived it because they are inlined macro expansions, not `assert!` calls, so the
  migration's grep never saw them. Only 4 sites, both files in `disk-rust`
  (`disk.rs`, `ddk_shims.rs`), and `disk-rust` uses `ASSERT!` zero times. Small and mechanical.
- **`scan_ud2`'s arm64 driver coverage is INCIDENTAL, not defined — OPEN** (2026-09-06).
  Found while chasing the above. arm64 has no `livecd` DAG; `archcfg.py:209` builds a
  hand-picked staged set plus a ramdisk, and `disk.sys` is in neither the required nor the
  optional target list. So the driver sweep grades **whatever binaries happen to be sitting in
  `build-arm64/`** — which in a fresh integration worktree is a handful of staged files, and in
  a long-lived lane worktree is however much someone once ran `ninja` for. Consequences both
  ways: the disk.sys bug above has never blocked a gate (the conductor's worktree does not
  build the file), and conversely a lane can eat a red arm64 `scan_ud2` for a driver it never
  touched. A verdict that depends on which artifacts a worktree happens to carry is the same
  shape as the AGENTS.md §7-8 fleet-isolation rule about resolving generated artifacts from
  siblings — the sweep should name its corpus, not discover it.
- **49 more bindgen flexible-array bounds checks are live in built objects — OPEN** (2026-09-06).
  Found by generalising the cmlib fix above into `tools/scan_flex_array_panic.py`, which reads
  the i386 disassembly and keeps only `panic_bounds_check` sites whose pushed ARRAY LENGTH is 1
  — the bindgen `[T; 1]` shape — while leaving real fixed arrays alone. The method validated
  itself: run against the pre-fix artifact it independently found exactly the three
  `CmpSelectLeaf` sites that were fixed by reading. **49 sites across 20 modules**, including
  the kernel (`LpcpCopyRequestData`, `MiFillSystemPageDirectory`, `SePrivilegePolicyCheck`,
  `IopEnumerateDetectedDevices`, two NUMA `QSI_*` queries), `rtl-rust`, `oleaut32`'s
  `SafeArray{CopyData,GetLBound,GetUBound}`, `umpnpmgr`'s log-conf pair, and `usbaudio`'s
  topology builders.
  **Graded so far: 3 confirmed bugs, 3 confirmed false positives, 43 ungraded.** Confirmed and
  FIXED: `SePrivilegePolicyCheck` (`PRIVILEGE_SET.Privilege`, over-allocated as
  `size_of + (count-1)*size_of::<LUID_AND_ATTRIBUTES>`; `index` reaches 1 whenever a caller is
  granted both `WRITE_OWNER` and `ACCESS_SYSTEM_SECURITY`) and `LpcpCopyRequestData`
  (`LPCP_DATA_INFO.Entries[1]` guarded by `NumberOfEntries`, on the `NtReadRequestData` path
  CSRSS uses). Confirmed NOT a bug: `KeNodeBlock[Node]` in the two NUMA `QSI_*` queries — the C
  really is `PKNODE KeNodeBlock[1] = { &KiNode0 };`, a one-element array whose check merely
  survives because the guard is a runtime variable (the msvcrt `_setmaxstdio` face already in
  the manual), and `MmSystemPageDirectory[PPE_PER_PAGE]` in `MiFillSystemPageDirectory` for a
  sharper reason: that constant is 512 on amd64/arm64 and **1 on i386** (`mm.rs:1265`), so an
  arch-sized real array masquerades as a flexible one in a reader that only looks at i386.
  **That ratio is the point: the tool finds the SHAPE, not the bug.** Two discriminators, in
  order — does the C over-allocate, and is the declared length a LITERAL `1` or a named
  arch-dependent constant? Read the declaration before touching a site.
  Not wired into the gate yet, deliberately: **89 of the 200 declared staticlib crates are not
  built for i386**, so a baseline captured today would bless whatever happens to be missing.
  The tool reports those as NOT BUILT (no coverage) rather than skipping them. Wiring it wants
  the coverage gap closed first, or an explicit per-arch corpus like the one `scan_ud2` still
  lacks.
- **cmlib `CmpSelectLeaf` panicked on any multi-leaf index root — FIXED** (2026-09-06).
  It wrote `*RootCell` through `addr_of_mut!((*IndexKey).List[i])` at three sites
  (i = SubKeyIndex, +1, -1). An array index inside a raw-ref still emits a bounds check, and
  bindgen renders the C flexible array as `[HCELL_INDEX; 1]`, so each panicked for every
  i >= 1 — i.e. as soon as a key's subkey list has more than one leaf. The file's own accessor
  comment forbids exactly this (`NEVER index List[i] for i>=1`); the sites read as
  address-taking rather than indexing, which is how they slipped past. Measured in the shipped
  i386 object: 4 `panic_bounds_check` sites in the function, 3 pushing an array length of 1;
  after routing them through `index_list_ptr`, only the legitimate length-2
  `SubKeyLists[Type]` check remains. Small hives boot, which is why smoke never caught it.
  **This entry previously said the opposite, twice over, and the correction matters more than
  the fix.** It said both functions were truncated and that they *did not ship*. Both false:
  `CmpAddToLeaf` and `CmpSelectLeaf` are complete (call-site table-diff vs the C: HvGetCell
  8/8, CmpMaxIndexPerHblock 5/5, HCELL_NIL 13/13), and the kernel links the Rust —
  `cmindex.c` is in `CMLIB_RUST_FLIPPED` and REMOVE_ITEM'd, so its appearance at
  `CMakeLists.txt:10` is the shared list feeding the *bootloader*. The C `libcmlib.a` in
  `build-i386` is a 248-byte placeholder. The entry came from `cmindex.rs`'s 2026-06-07 review
  header, whose four "REAL BUGS" had all been fixed by `85479bd0878` without the text being
  updated. **A stale review header reads as current and is worse than none**; the ABI
  complaint it also carried is likewise dead (the five NTAPI callees are defined locally as
  `extern "system"`). Header rewritten in place.
- **classpnp `ClasspAccessAlignmentProperty` spun forever on arm64 — FIXED** (2026-09-05).
  An empty `_ => {}` catch-all as the last arm of a `match` that IS the body of a
  goto-elimination `'work: loop`. utils.c's switch has no `default:`, so an unmatched value
  falls out of the switch onto `Exit:`; the empty arm instead reaches the end of the loop and
  re-matches the unchanged scrutinee forever. arm64 emitted a literal `b .` INSIDE the
  function. Found only because this lane linked `classpnp.sys` for arm64 — **it is not in
  arm64's default target set**, so no arm64 sweep had ever scanned that binary (exactly the
  ucrtbase/`exp2` blind spot, one driver over). Swept the tree for the shape: 0 others.
  Worth fixing separately: a `build-arm64` that silently omits a driver every other arch
  builds means every post-link gate is blind to it.
- **classpnp's `KeRaiseIrql`/`KeLowerIrql`/`IoWMIDeviceObjectToProviderId` shims called
  THEMSELVES — FIXED** (2026-07-26 found, 2026-09-05 fixed). Each `#[no_mangle]` body in
  `classpnp-rust/src/ddk_shims.rs` declared an `extern` block for its OWN name and called it.
  A local strong definition beats an importlib thunk, so the call bound to the enclosing
  function; LLVM tail-folded all three to a single instruction — `4fb90 <KeRaiseIrql>: eb fe
  jmp 4fb90` on amd64, `b .` on arm64. Any classpnp path that touched IRQL spun that CPU
  forever with no bugcheck and no serial output. Same family as the "never hand-write memcpy
  in Rust" rule in AGENTS.md: **a `#[no_mangle]` shim may never call the symbol it defines.**
  The fix is per-arch `cfg`, not a better forwarding body — define a symbol ONLY on the arch
  where none is importable: i386 keeps the `KfRaiseIrql`/`KfLowerIrql` FASTCALL inlines
  (already correct), amd64 gets real CR8 bodies for both, arm64 defines neither and binds
  `hal.spec:84,86`. **The trap that cost a link:** `ntoskrnl.spec:635` really does export
  amd64 `KeLowerIrql`, but `-private`, and `spec2def.c:750` keeps private exports out of the
  *import lib* — so the question is never "is it exported" but "is there an importlib entry".
  Verified in the emitted code on all three arches (CR8 on amd64, `adrp/ldr/br x16` import
  thunk on arm64, `call @KfRaiseIrql@4` unchanged on i386), all three smoke green with zero
  assertions, and the three entries are out of `tools/lint-baseline/scan_self_recursive.txt`.
  Note classpnp.sys is not in arm64's default target set, so it was linked explicitly to
  verify. Field manual: "NEVER hand-write memcpy…", third route to the same `jmp .`.
  > Found via `scan_ud2`'s driver sweep. The other 11 spins in the same report are **faithful**:
  > upstream ReactOS really does write `UNIMPLEMENTED; while (TRUE);` in pcix's arbiter and
  > power paths, so `loop {}` is correct there and must NOT be "fixed" — check the C before
  > touching one (`ario_Initializer` has it commented out, `ario_Constructor` does not).
- **Kernel asserts that hung the box — 92 FIXED, remaining work is mechanical** (2026-07-26).
  A Rust panic in kernel mode reaches a `#[panic_handler]` that prints to debugcon and loops
  forever: a silent hang, no bugcheck, no dump. Fixed in two passes — **31** spelled
  `panic!`/`unreachable!` (now gated by `scan_assert_to_ub.py`, baseline empty) and **61**
  spelled `assert!`, migrated to the new `ntkrnl_sys::ASSERT!`/`NT_ASSERT!` macros which expand
  to `RtlAssert`: loud, non-fatal, matching C under `-DDBG=1`. Deliberately left alone: 599
  `size_of`/`offset_of` layout checks, 8 uniata `const _: () = {}` register-layout locks, and 4
  kdbg `#[cfg(test)]` asserts — all of which *should* be compile-time or fatal.
  **`debug_assert!` — DONE for kernel crates, 454 sites** (2026-07-26, `migrate_debug_assert.py`).
  The population was never 1962: that counted bindgen boilerplate (16 `__BindgenBitfieldUnit`
  bounds checks per island × 3 arches × ~30 crates), 149 `cfg(debug_assertions)` gates and 8
  comments. **490** real invocations — 454 kernel, 36 usermode. Kernel is migrated to
  `ntkrnl_sys::ASSERT!`; usermode needs an ntdll-side macro (`RtlAssert` is exported there too,
  with `PCHAR` rather than `PSTR`) and is the remaining slice, along with the two crates whose
  local `macro_rules! ASSERT` expands to `debug_assert!` (usbaudio, mouhid).
  Two things made this safe rather than a risk decision: the shipping `RtlAssert` is **Rust**
  (`rtl-diag/src/assert.rs` — `assert.c` is inert cohab source), and it prints only the first
  **20** failures then returns, so no assertion can hang or flood the log however many go live.
  Making them live immediately paid: `hidusb.sys` failed to link on amd64 because
  `KeGetCurrentIrql` is HAL-exported on i386/arm/arm64 but a CR8 FORCEINLINE on amd64 — the C
  `ASSERT` at `hidusb.c:410` had been dead in Rust, so nothing had ever referenced the symbol.
  Field manual, "`NT_ASSERT`/`ASSERT`/`PAGED_CODE` are DIAGNOSTICS" + §9.135.
- **`!x` translated as bitwise NOT — 8 sites, 5 of them live branches** (2026-07-26). C's `!` is
  logical; Rust's `!` on an integer is bitwise, so `!(flags & FLAG) != 0` is true for every value
  a flag test can produce — a constant condition that compiles clean and trips no rustc lint. The
  class was *already in the field manual* and still shipped: classpnp's `ADJUST_FUA_FLAG`
  collapsed to its first term (forcing Force Unit Access onto devices that had reported it
  unsupported), an always-true `||` arm disabled win32k's CJK line-break rule, kernel32's
  `DuplicateConsoleHandle` lost a validation arm, and scsiport kept only the null half of an
  assert. Now gated: `scan_bitwise_not_bool.py`, baseline EMPTY, with controls covering the two
  idioms it must never flag (`x & !MASK != 0`, and the macro bang in `offset_of!`) — a first
  draft that missed those reported 4710 sites.
- **`.try_into().unwrap()` — FIXED where it mattered; the rest is cosmetic** (2026-07-26).
  All 234 sites in the Rust tree were audited. Exactly one was reachable and it was a
  usermode-triggerable **kernel** panic: `handle_internal_message`'s `WM_ASYNC_SHOWWINDOW` arm
  (`win32k-rust/src/message.rs`) narrowed a 64-bit `WPARAM` to `INT` with `.unwrap()`.
  `NtUserShowWindowAsync` takes `nCmdShow` as a `LONG` straight from usermode and range-checks
  only the top (`> SW_MAX`), so `ShowWindow(hwnd, -1)` sign-extends to
  `0xFFFF_FFFF_FFFF_FFFF` and panics in win32k. The C — recovered from history, the file was
  retired in `8d3c33ee41e` — is `co_WinPosShowWindow(pWnd, wparam)`, whose implicit truncation
  round-trips `-1` correctly. Fixed to `wparam as INT`.
  The other 233 cannot fail: 146 have compile-time-constant receivers, and the two big runtime
  families are same-width identity conversions (allocation sizes into `RtlAllocateHeap`'s
  `SIZE_T = ULONG_PTR`; message args into externs declared `wParam: usize` / `lParam: isize`).
  **So task #19 is mechanical cleanup, not a backlog of live panics** — worth doing for the
  named-anti-pattern reason, but it needs the compiler as the per-site oracle for the target
  type, so it is a build-bound batch. Owed: a gate so new ones cannot land (`crate-repair`
  refuses to *generate* the pattern, but nothing stops one being hand-written). Field manual,
  "Semantic rewrites that flip meaning".
- **FIXED (E22, 2026-08-01): every 32-bit process silently lost the `cbSize`-validated USER32
  APIs.** `TrackMouseEvent` (so no 32-bit app ever received `WM_MOUSELEAVE`/`WM_MOUSEHOVER`),
  `GetCursorInfo`, `GetGUIThreadInfo`, `GetComboBoxInfo`, `GetMenuInfo` and `SendInput` (all
  synthesized input) returned `FALSE`/`ERROR_INVALID_PARAMETER` under WoW64 and nowhere else.
  Each struct states its own `sizeof` and win32k version-checks it against the **host** size,
  so the call was refused before the payload was read — a *field-perfect* marshaller does not
  help, which is why five of the six were waiting on a generator that copied `cbSize`
  verbatim. `marshalgen` now emits a value-guarded size rewrite (`WowAmSizeIn`/`WowAmSizeOut`)
  and `NtUserSendInput`, whose size is an argument rather than a field, is hand-written.
  Audit baseline 209 → 203. Field manual: "A struct that carries its own `sizeof` is rejected
  before a single field is read".
- **FIXED (E23, 2026-08-02, task #82): every error code win32k set was invisible to a 32-bit
  guest.** `win32k-rust/src/error.rs` — `EngSetLastError` writes
  `NtCurrentTeb()->LastErrorValue`, which in kernel mode is the **64-bit** TEB, while a 32-bit
  guest reads `TEB32` at `WowTebOffset`. win32k reports failure by setting last-error and
  returning FALSE, so a 32-bit app got the FALSE and never the reason; anything branching on
  the code (`ERROR_CLASS_ALREADY_EXISTS`, `ERROR_NO_MORE_ITEMS` to end an enumeration loop)
  misbehaved. The whole USER/GDI surface; ntdll was never affected (NTSTATUS returns,
  converted by the guest's own ntdll32). wow64.dll now mirrors last-error at the layer
  boundaries — **into the kernel copies up, into the guest copies down** — at four sites
  (`wow64_win32k.c` syscall entry/exit, `wow64_callback.c` callback entry/return), so TEB32
  and TEB64 behave as the single variable a native 64-bit process has. The callback pair is
  not optional: it closes a hole the syscall pair creates, where a guest window proc's own
  syscall would overwrite the `ERROR_TIMEOUT` that `co_IntSendMessageTimeout` sets and
  `BroadcastSystemMessage` reads back. Found by E22's unfixed-media control run
  (`err = 0x00000000` where win32k had just set `ERROR_INVALID_PARAMETER`); `marshal32` rung 8
  now drives one API two ways so a *code* has to travel, not merely appear. Field manual:
  "The kernel sets last-error in the 64-bit TEB and the 32-bit guest reads TEB32".
- **OPEN: `NtUserGetMouseMovePointsEx` never fills its output buffer** (`win32k-rust/src/
  ntmouse.rs`, `GMMP_USE_DISPLAY_POINTS`) — it probes `lpptOut` and returns `nBufPoints`
  without writing a single point, so `GetMouseMovePointsEx` hands every caller an
  uninitialized array. Pre-existing and **not** WoW64-specific; found while scoping E22, which
  deliberately left the call unmarshalled rather than convert an honest failure into a
  success with garbage. It stays on the marshalgen-audit baseline until the kernel side is
  real (2 entries).
- **FIXED (2026-08-05, task #48): msgina's `StringCchPrintfW`/`StringCbPrintfW` dropped their
  bound — a stack overrun.**
  `msgina-rust/src/lib.rs` renders both as macros that DISCARD the count argument and forward to
  user32's variadic `wsprintfW`, which bounds to 1024 wchars internally. `shutdown.rs:1091`
  formats a resource-loaded format string plus `gContext.UserName` into `szBuffer2: [u16; 256]`,
  so a long user name writes up to 768 wchars past the end of a stack array; the C
  (`StringCchPrintfW(szBuffer2, ARRAYSIZE(szBuffer2), ...)`) truncates safely and returns
  `STRSAFE_E_INSUFFICIENT_BUFFER`. Introduced by the msgina translation, not present upstream.
  Two other msgina strsafe entry points are hand-approximations of the same header
  (`StringCbCopyNExW` drops the fill paths, the `IGNORE_NULLS` handling, and computes
  `pcbRemaining` differently from the worker). The same call site had a second bug the macro
  hid: `UserName` is a `[WCHAR; 256]` and was passed to the variadic BY VALUE, so `%ws` read the
  first pointer-width bytes of the name text as an address — C decays an array to a pointer at a
  call and Rust does not. Fixed by `sdk/lib/rust/strsafe-rust`, one faithful header port that
  msgina/syssetup/desk (and sysdm next) all consume; the four divergent copies are what made
  this possible, so the crate is the fix for the class and not just the instance.
- **FIXED (2026-08-01): the registry's `HIVE_*` flag family was open-coded and half wrong.**
  `scan_mask_drift`'s first sweep found eight bare literals across five config-rust files, four
  of them with the wrong value. `CmLoadKey` set `0x4` and `CmUnloadKey` set `0x40` for
  `HIVE_IS_UNLOADING` (**0x20**); the two guards that consume it tested `0x4` (cmparse) and
  `0x200` (cmkcbncb). Only the load-side set and the cmparse test agreed, *by coincidence*, so
  the C's "don't resolve a key in a hive that is unloading" protection was inert on the unload
  path — `CmpParseKey` and `CmpCreateKeyControlBlock` would happily open keys and build KCBs in
  a hive mid-unload. `CmpLoadHiveThread` cleared `0x80` instead of `HIVE_NOLAZYFLUSH` (**0x2**),
  so that hive stayed excluded from the lazy flusher permanently, and `cmdata`'s SAM entry was
  tagged `0x8` (`HIVE_HAS_BEEN_FREED`) for the same flag. All eight now import one
  `config_rust::hiveflags`. Field manual: "A hand-typed bit-flag literal that names the wrong
  member".
- **FIXED (E21, 2026-08-01): `NtOpenKey` denied every `KEY_WOW64_32KEY` open.** The Rust port
  masked a bare `0x0100` where the C masks `KEY_WOW64_RES` (0x0300), so the 32KEY bit reached
  `ObCheckObjectAccess` and came back `STATUS_ACCESS_DENIED` — while the identical
  `RegCreateKeyEx`, which masks correctly, succeeded. Hit **native 64-bit** callers too (rapps,
  `reg /reg:32`, apphelp), not only WoW64 guests. Proven by positive control: unfixed media
  reported `wow64flags: KEY_WOW64_32KEY refused, status = 0xc0000022`; fixed media 9/9 ladder.
  The same commit fixed two more in that file: `NtOpenKey` stored `*KeyHandle = NULL` **before**
  any probe and only ever checked alignment, never the user/kernel boundary (kernel-mode write
  to a caller-supplied address — the #67/#68 class), and `ProbeAndCaptureObjectAttributes`
  open-coded `0x12F2` for `~OBJ_VALID_KERNEL_ATTRIBUTES` (0x107F2), rejecting `OBJ_OPENLINK` /
  `OBJ_FORCE_ACCESS_CHECK` (latent — every `NtLoadKey*` caller passes `OBJ_CASE_INSENSITIVE`
  alone). Regression-guarded by marshal32 rung `CheckRegistryWow64Flags` (0xBAE7–0xBAEA), so
  `smoke --ladder` now catches it. **This is NOT registry redirection** — that landed
  separately as E24 below, which also moved this strip out of `NtOpenKey`/`NtCreateKey` and
  into `CmpParseKey`, because once the two views differ the flags have to select one before
  being cleared.
- **FIXED (E24, 2026-08-02): Wow6432Node registry redirection was entirely absent (task #66).**
  A 32-bit process shared the 64-bit `HKLM\Software`, so nothing it wrote was isolated and a
  COM `InprocServer32` it read named a 64-bit DLL it could not load. Implemented in CM, as one
  wrapper around `CmpParseKey` (`config-rust/src/cmwow64.rs`): every key open funnels through
  that parse procedure — Ob's only bypass is for directory objects, which a key never is — so
  it catches direct `NtOpenKey` callers as well as advapi32. Rewriting paths in `wow64.dll`
  (where FILESYSTEM redirection lives) was rejected: registry opens are overwhelmingly
  RELATIVE, so it would have cost a handle-to-path syscall on every open. Gated on
  `EPROCESS.Wow64Process`, so the all-64-bit boot path cannot be affected, and compiled to
  `false` on i386. **Deliberate deviation from Windows:** a redirected open that misses falls
  back to the 64-bit key (Wine's model) instead of hiding it, because nothing populates
  Wow6432Node here and redirecting reads into an empty node would have hidden every OS key a
  32-bit app reads. Writes still land in the private node. Revisit when we ship 32-bit-only
  components. Proven by marshal32 rung 9 (0xBAFA–0xBAFE), which asserts the two views
  *disagree* — the default handle's own `KeyNameInformation` says Wow6432Node AND the same
  path under `KEY_WOW64_64KEY` cannot see the value just written.
- **FIXED (2026-08-01): i386 `KeFlushCurrentTb` under-flushed the TLB.** It dropped the C's
  `KF_GLOBAL_PAGE` branch, so the CR4.PGE toggle never ran and global TLB entries survived
  every flush — and `patpge.rs` does enable PGE. Found by the commented-C mask oracle with no
  symptom to go on; amd64's C genuinely is CR3-only, so only the i386 copy had drifted.
- **`msvcrt-difftest`'s `the_fd_layer_works_and_reports_length` is FLAKY under cargo's
  parallel test threads → spurious DIFFTEST_FAIL that convicts an innocent lane — NEW, OPEN**
  (2026-08-01). A shell32-com batch whose only delta was a **parked** `shlexec.rs` plus
  comments was rejected `DIFFTEST_FAIL`; the queue's own base-alone control run then came back
  `✓ msvcrt-difftest` on the same base. Same code, one red, one green — so it is a flake, and
  difftest is treated as a DETERMINISTIC stage, which is exactly why it convicts the lane
  rather than being retried like a smoke.
  The failure is a **mangled filename**, not a missing one: the test asks for
  `/tmp/msvcrt-ref-<pid>-fd` and `siblings_of` reports `/tmp` holds `msvcrt-ref-<pid>-fdek`.
  `fd` is written over the tail of the concurrently-running `seek` test's path (`…-seek` is
  exactly two chars longer) without re-terminating, so the mangle is offset-aligned — shared
  or stale buffer, not a random corruption. Ruled OUT so nobody re-walks them: `path_from_wide`
  and `msvcrt_wstrdupa` both `malloc`, `_sopen_dispatch` frees its `wstrdupa_utf8`, and
  nlsimpl's iconv descriptor caches are `__thread`. The remaining suspect is stale heap bytes
  in a freshly-`malloc`'d wide-path buffer that a short conversion failed to terminate.
  **ROOT CAUSE, and the crate already documents its own fix.** Every `[[test]]` target in
  `msvcrt-difftest/Cargo.toml` carries `harness = false`, for the reason stated there: the crate
  exports a `#[no_mangle] getenv` that shadows glibc's, so *"libtest reads RUST_TEST_THREADS
  through it, gets NULL, and misbehaves. A plain single-threaded `fn main()` runner never calls
  getenv and never spawns threads."* `tests/file_substrate.rs` has **no `[[test]]` entry at
  all**, so it is cargo-AUTO-DISCOVERED and runs on the default libtest harness — which spawns
  threads — and its five tests then race on process-global CRT state. Four of them drive real
  files, which is why `fd` and `seek` collide.
  (Three other targets are auto-discovered the same way — `nls_thread_safety`, `printf_port`,
  `printf_probe` — so "the only threaded binary" is NOT the discriminator, and an earlier
  revision of this entry saying so was wrong. The discriminator is that `file_substrate` is the
  only auto-discovered one whose tests touch the fd table at all; the other three make no file
  or `_open` call, and `nls_thread_safety` threads DELIBERATELY as its whole subject.)
  Fix is the one the file's own author already prescribes: give `file_substrate` a `[[test]]`
  entry with `harness = false` and a single-threaded `fn main()` runner, like its ten siblings.
  Do NOT "fix" it by renaming the tags to equal length — that hides the mangle instead of
  removing the race, and equally do NOT exempt difftest from the queue's deterministic-stage
  conviction, which would let real rust-vs-C divergences through.
  Cost: a full ~25-minute gate plus a ~25-minute control gate per hit, and it blocks the
  merge queue for every other lane while it runs. Owner: `msvcrt-homebrew` lane
  (`sdk/lib/rust/msvcrt-difftest/`).

- **`gate` races `marshalgen_audit` against a half-built `build-amd64` → spurious LINT_FAIL.**
  Seen 2026-07-26 (`/tmp/vibeos-cli/20260726-021528-gate-*`): every build green, all four
  smokes green, and `lint:i386` FAILED with *"the BUILT thunk table disagrees with the
  checked-in sources … is STALE"*. It was not stale — `lint:i386` starts the moment
  **build:i386** lands (gate.py pipelines per-arch lanes on purpose), and `marshalgen_audit`
  reads `build-amd64`, whose build finished 19 minutes later. Re-running `check --arch i386`
  afterwards: OK, 31/31, *"built table agrees"*. So the lint is right to hard-fail on a stale
  artifact — the bug is that a lint reading ANOTHER arch's build dir is scheduled off THIS
  arch's build. Costs a full 35-minute gate each time it hits, and the merge-queue conductor
  runs the same suite. Fix is a scheduling one (run cross-arch lints after all builds, or gate
  `marshalgen_audit` to the arch whose build dir it reads — wow64 is amd64-only, so the second
  is also semantically right). Owner: whoever holds `tools/wow64gen/` + `tests/qemu/cli/`.

- **x86 `-smp 2` MP conformance deltas, separated into deterministic vs flaky — NEW, OPEN**
  (2026-07-29, 37 i386 + 16 amd64 recorded rounds against the `kmtest-smp2` baselines). These
  are MP-vs-MP-baseline regressions, i.e. the MP kernel got *worse* than its own recorded
  floor, so they gate the flip independently of the arm64 blocker.
  **SPLIT 2026-07-29 by baseline archaeology (no host time needed).** These were recorded as
  one class of "MP regression". They are three different things, and only one is real:
  - **`FsRtlLegal` 2504/24 → 2496/32 — CLOSED 2026-07-30. It was never a flake: it was
    out-of-bounds READ UB, deterministic per BUILD.** `fsrtl-rust`'s three DBCS legality
    helpers dropped the `(SCHAR)(C) < 0 ? DEFAULT_RET` short-circuit that opens the
    `FsRtlTestAnsiCharacter` macro (fsrtlfuncs.h:1632) and indexed
    `LEGAL_ANSI_CHARACTER_ARRAY_DATA` — a `[u8; 128]` — with the raw byte, so every character
    ≥ 0x80 answered from up to 128 bytes past the end of a static. The 32 failing assertions
    are exactly the names `"\x80"` and `"\xff"` × 8 flag combinations × {Fat, Hpfs}; the 24
    variant is the same set with one of those four groups happening to read a byte with the
    right bit set. Fixed → **arm64 measured 2496/32 → 2528/0**, a pure conversion of failures
    into passes. The earlier archaeology was sound about the *numbers* and wrong about the
    *cause*: counting run dirs cannot distinguish "rare per boot" from "fixed per build" when
    the corpus is dominated by one build. **Recognition rule: before calling a two-valued
    conformance row a flake, check whether the two values partition by BUILD rather than by
    boot — UB whose answer comes from the link layout looks exactly like a rare coin.**
    This also re-frames the co-occurrence lead below: at least the `FsRtlLegal` half of it does
    not vary boot-to-boot at all, so "whatever varies boot-to-boot moves both" is the wrong
    question for that half. Whether `ZwAllocateVirtualMemory`'s 843 is the same shape is
    untested and still owed.
  - **`IoDeviceObject` 185/4 vs 184/5 — a ~50/50 FLAKE, not a regression** (closes the old
    "blessed with no rationale" framing, task #48). The baselines disagree only because each
    is a single sample of one flaky assertion: at `049840353ee` the i386 smp2 baseline was
    seeded reading 185/4 while the i386 UP baseline on the SAME TREE read 184/5, and today
    amd64 UP records 184/5 against amd64 smp2's 185/4. Measured: 52/100 i386 rounds and 24/52
    amd64 rounds land on 184/5. **Re-seeding any of these baselines just re-samples the coin**
    — name the assertion first.
    **UPDATED 2026-07-30 — it is NOT x86-only, and on arm64 it is not one assertion.**
    Tallying every banked run dir per arch:
    - i386 — 184/5 ×75, 185/4 ×68 (two outcomes, ~52/48)
    - amd64 — 184/5 ×40, 185/4 ×28 (two outcomes, ~59/41)
    - arm64 — 180/9 ×19, **181/8 ×66**, 182/7 ×26, 183/6 ×2 (**four** outcomes)
    Two values on x86 is consistent with one racing assertion; a four-wide spread on arm64 is
    not — that is ~3 independent flaky assertions, or one race with a variable victim count.
    So the old "one test, one assertion, both x86 arches ⇒ one genuine race" reading was an
    artifact of only ever having looked at x86. The arm64 baseline does at least sit on the
    mode (181/8). Naming the assertions is still the prerequisite for any re-seed.
  - **`ExPools` on arm64 — an arm64-ONLY flake, ~29%, NEW 2026-07-30.** Surfaced by a 10-round
    3-arch soak on `dd3411c76d45` (arm64 round 3 reported `ExPools: pass 14 -> 13`), then
    quantified against the banked corpus: arm64 reads 14/4 in 105 runs and 13/5 in 42
    (~71/29), while **i386 is 19/1 in 144 of 144 runs and amd64 17/1 in 68 of 68** — both
    perfectly deterministic. One flaky assertion, one arch. Unlike the rows above this is not
    just a baseline-sampling artifact: a pool test that is stable on two arches and not on the
    third is an arm64 lead. The baseline holds the lucky side (14/4), so it reads as a
    regression ~29% of the time.
    **ROOT-CAUSED 2026-09-13, and it is ONE event, not "one flaky assertion".** The two modes
    are `{119,135,178,210}` at 14/4 and `{119,135,176,181,210}` at 13/5, never any other
    combination — `:178` and the `{:176,:181}` pair are mutually exclusive, which is not a
    noise shape. `ExPools.c:175-181` reads `PoolType = KmtGetPoolType(Memory); ok(PoolType !=
    0); PoolType--; ok(PoolType & QUOTA_POOL_MASK); ok((PoolType & BASE_POOL_TYPE_MASK) ==
    PagedPool || ...)` with `PoolType` a `USHORT`. `KmtGetPoolType` returns **0 as a sentinel**
    for "page-aligned, so no POOL_HEADER to read" (kmt_test_kernel.h:136), so on that path
    `:176` fails, `0--` underflows to `0xFFFF`, `:178` **passes spuriously** because
    `0xFFFF & QUOTA_POOL_MASK` is nonzero, and `:181` fails on the garbage. Observed in serial,
    not inferred: the bad mode prints `:176: PoolType is 0` then `:181: PoolType = ffff`, the
    good mode prints `:178: PoolType = 2` (raw header value 3). Consequences for the floor:
    `:178` is **not** bistable — it is anti-correlated with `:176` by construction, and its
    `assert?` mark records a consequence as independent debt; `:176` is the only line carrying
    signal. The residual kernel question is narrow: `KmtGetPoolType` returned 0 for a 16-byte
    `ExAllocatePoolWithQuotaTag` block, meaning either the block was page-aligned (the
    sentinel) or its header's `PoolType` field was zeroed. The printed values cannot separate
    those — the test prints no pointer on the success path — but both say the arm64 pool
    intermittently hands back a small allocation whose `POOL_HEADER` does not describe it, and
    the good mode proves the header is normally readable there. **Rate is worse than the ~29%
    banked here:** 2 of 4 arm64 sweeps on 2026-09-13 took the bad mode and it took a full
    3-arch gate RED through all 3 conformance retries (`20260913-021807-fbitjxpy`). **Not the
    same bug as `IoDeviceObject_drv.c:198/206`**, despite the identical `KmtGetPoolType(x) - 1`
    idiom and adjacency at the top of the intermittent list: tested, they fire independently,
    and their `KmtGetPoolTag` returns `0x0062005c` / `0x0073006f` — UTF-16 `\b` and `os`, i.e.
    those Buffers point into the middle of an allocation rather than at its start.
    **SUPERSEDED SAME NIGHT — it IS the same bug, and it is in the HARNESS, not the pool.**
    `modules/rostests/kmtests/include/kmt_test_kernel.h` keeps a private `POOL_HEADER` whose
    64-bit layout is gated on **`_M_AMD64`**; the kernel's (`ntoskrnl/mm/ARM3/miarm.h:372`)
    is gated on **`_WIN64`**. arm64 is `_WIN64` and not `_M_AMD64`, so the kernel writes a
    16-byte header (its own `C_ASSERT(sizeof(POOL_HEADER) == POOL_BLOCK_SIZE ==
    sizeof(LIST_ENTRY))` compiles on arm64) while kmtest's copy is 8 bytes. `KmtGetPoolType`
    and `KmtGetPoolTag` do `Header = Memory; Header--`, so on arm64 they step back HALF a
    header and read `PoolTag`/`ProcessBilled` as `Ulong1`. That predicts every observation:
    `PoolType` is bits of the quota slot that `ExAllocatePoolWithQuotaTag` sets to the process
    or NULL (hence the two modes), `:178` fails stably because the quota bit is never where
    kmtest looks, `KmtGetPoolTag` returning `0xfffffa00`/`0xfffff800` is the high half of a
    kernel pointer, and IoDeviceObject's `0x0062005c`/`0x0073006f` is the previous block's
    payload. **There is no pool corruption and no arm64 kernel defect here.** The arm64-only
    pattern recorded above — i386 144/144, amd64 68/68, arm64 alone unstable — was the tell:
    an arch-selective macro produces exactly it. Fix is `_M_AMD64` -> `_WIN64` in that header,
    affecting three call sites (ExPools.c:175, IoDeviceObject_drv.c:198/206), but it is a
    CONFORMANCE TEST change that will move many arm64 assertions at once and wants a
    deliberate arm64 re-seed — owner's call, not a drive-by.
    **CLOSED 2026-09-13 — fixed, re-seeded, and confirmed.** Owner approved and scoped the
    rule that had been holding it: a build-flag / arch-gate correction is not a semantic
    change to a conformance test and needs no approval; approval is for changes to what a test
    MEANS in a direction that favours us, i.e. that would pass here and fail on real Windows.
    The three `#ifdef _M_AMD64` in `kmt_test_kernel.h`'s `POOL_HEADER` are now `#ifdef _WIN64`
    (real Windows' header is 16 bytes on every 64-bit target; the union's `ProcessBilled`
    pointer forces it). arm64 ExPools: `pass=14 fail=4` `{119,135,178,210}` ->
    `pass=17 fail=1` `{210}`, stable across four boots — **identical to amd64's baseline row**,
    which is the real confirmation: the two 64-bit arches now agree. `:210` is the shared
    64-bit 2GB-quota case, not this bug. Baseline re-seeded with `--only ExPools`, and the
    stale `:176`/`:178`/`:181` marks removed — they described the misread, so keeping them
    would have left the floor tolerating a failure that can no longer occur. A
    `C_ASSERT(sizeof(POOL_HEADER) == sizeof(LIST_ENTRY))` now guards the struct; restoring
    `_M_AMD64` fails it at compile time on arm64, so this was verified as a discriminator
    rather than assumed. Six weeks of "arm64 kernel lead" was a one-word arch gate.
    **A PREDICTION, recorded before it can be confirmed.** The fleet-wide census
    (`5255a32d74e4`, 196 real gate runs) counts ExPools as the LARGEST single contributor to
    arm64 conformance redness at 30 distinct batches. That census predates this fix, so the
    next one must show ExPools dropping out of the top of that list. If it does not, the fix
    is not what closed it and this entry is wrong -- which is the point of writing the
    prediction down while it is still falsifiable rather than noting the agreement afterwards.
    **Confirmed against the toolchain, not just read out of the headers.** `_WIN64` IS a
    clang builtin: `.build-tools/llvm-mingw/bin/clang -target <t> -dM -E </dev/null` gives it
    for both `aarch64-w64-mingw32` and `x86_64-w64-mingw32` and not for `i686-w64-mingw32`.
    **`_M_AMD64` is NOT a clang builtin on any mingw target** — a bare `-dM -E` reports it
    absent even for x86_64, so read the `_M_AMD64` column below as "in a real TU that includes
    the SDK", not as something `-dM -E` will show you.
    It cannot reach arm64 by any path, structurally rather than incidentally: both definition
    sites are gated on x86_64 — `sdk/include/reactos/msvctarget.h` defines it at :16 inside an
    `#elif defined(__x86_64__)` arm of a mutually exclusive chain whose `#elif
    defined(__arm64__)` arm (:28) is the one arm64 takes, and `sdk/include/vcruntime/
    _mingw_mac.h:85` gates its copy on `defined(__x86_64)`. `CMakeLists.txt:279` passes
    `-D_ARM64_ -D_M_ARM64 -D__arm64__ -D__aarch64__ -D_WIN64` for arm64.
    | arch | `_WIN64` (compiler) | `_M_AMD64` (with SDK) | kmtest | kernel | agree |
    | i386 | no | no | 32-bit | 32-bit | yes |
    | amd64 | yes | yes | 64-bit | 64-bit | yes |
    | arm64 | yes | **no** | **32-bit** | **64-bit** | **NO** |
    That is exactly the i386-144/144, amd64-68/68, arm64-only-unstable shape recorded above,
    and the amd64 row is the one to confirm before touching the header: it is why the fix is
    demonstrably arm64-only across the three call sites.
  - **MEASURED 2026-09-13: an arm64 conformance sweep goes RED 3 times in 9, from THREE
    distinct causes.** 9 sweeps of the same media at `11669999159` (arm64, smp=0, i.e. the MP
    kernel on one vcpu), banked in `~/vibeos-flake-farm/conformance.jsonl`: **6 clean, 2
    `ExPools`, 1 `Example`**. Per-sweep red rate **33%**. That is the number the retry budget
    and the `CONFORMANCE_FLAKE` gating question below turn on, and it had never been measured —
    the ~29% figure in the ExPools entry is the rate for ONE test, not for the lane.
    **Correcting a claim made earlier the same night from n=4:** ExPools was reported at ~50%;
    over 9 sweeps it is **2/9 = 22%**, so the banked ~29% stands and the 50% was small-sample
    noise. The lane-level 33% is higher than any single test's rate precisely because the
    causes are independent and compose — which is the real finding, and it survives the
    correction that prompted it.
  - **CENSUS of the fleet's real gate artifacts, n=201 (2026-09-13). SELF-CORRECTED: the
    "ten distinct tests" first written here was WRONG — inflated by exactly the two-mode
    conflation the same census identified.** Counts every `conformance-arm64`
    `summary.json` on disk, rather than one lane's flake farm.

    **Two modes, and they must not share a denominator.** 8 of the 201 runs are a LOST BOOT:
    `BUGCHECK 0x1E`, `ok`/`bad` both 0, and every baseline test reported `in baseline (OK) but
    never launched`. The other 193 are ordinary sweeps. **Mode-1 red rate: 45 of 193 = 23.3%**
    (the 27% first written here pooled both modes; use 23.3%).

    **Why the cause list was wrong.** A mode-2 run names the WHOLE baseline as regressed, so it
    adds a constant +8 batches to *every* test's count. Splitting by mode:

    | test | mode-1 batches | mode-2 batches |
    |---|---|---|
    | `ExPools` | 25 | 8 |
    | `NpfsConnect` | 9 | 8 |
    | `KeEvent` | 4 | 8 |
    | `IoInterrupt` | 4 | 8 |
    | `KeDpc` · `Example` · `IoDeviceObject` | 1 each | 8 |
    | `CmSecurity` · `ExCallback` · `ExDoubleList` · `ExUuid` · `IoFilesystem` · `ZwMapViewOfSection` · `HalSystemInfo` · `FsRtlTunnel` · `RtlCaptureContextKM` | **0** | 8 |

    So the genuine mode-1 set is **`ExPools`, `NpfsConnect`, and marginally `KeEvent` /
    `IoInterrupt`** — not ten. Everything in the bottom row is mode-2 collateral: those tests
    have never once been observed regressing in an ordinary sweep, and filing them as flakes
    would file nine tests nobody has seen fail. **`CmSecurity` in particular has ZERO mode-1
    occurrences** and should not be registered as a flake.

    **The caveat that must travel with any of these (from the conformance lane's own scar):
    batch spread proves NOT-ONE-LANE'S-DEFECT. It does not prove flaky.** `ExPools` had 30
    batches of perfect spread and was a deterministic HARNESS bug — the `POOL_HEADER` arch-gate
    — reproducible on every run. "Fleet-wide, cause unknown, determinism unknown" is the honest
    filing; deterministic failure is evidence AGAINST a kernel bug.

    **Determinism is mostly UNDECIDABLE from this data, and saying so beats leaving it open by
    omission:** 151 of 170 batches contain only ONE mode-1 run, so there is nothing to compare.
    Where a batch has 2-3: `ExPools` regressed in ALL runs of 10 batches and SOME of 15;
    `NpfsConnect` ALL of 6, SOME of 3; `KeEvent` and `IoInterrupt` ALL of 4, SOME of 0. The
    `ExPools` split spans the `POOL_HEADER` fix, so it is not clean evidence either way.

    **Consequence for the retry budget.** Three attempts discriminates mode 1 — a `-1` subtest
    is an independent trial, so independent 23.3% ⇒ ~1% spurious rejections, matching the
    observed trickle. It does NOT discriminate mode 2: the test never RAN, so re-booting a
    media that dies the same way is not an independent trial. Mode 2 wants a coverage check —
    *did the sweep launch what the baseline says exists* — computable from ONE run with no rate
    at all. **No single averaged rate should appear in the design.**

    **THE CORPUS THIS WAS COMPUTED FROM NO LONGER EXISTS, and these figures cannot be
    re-derived — this entry is the record.** The 2026-09-14 disk loss took `/tmp/vibeos-cli`
    with it: zero conformance `summary.json` survive, and `~/vibeos-flake-farm/` is gone.
    Verified 2026-09-14.

    **The durability lesson is about WHERE, not whether — and the obvious diagnosis is wrong.**
    It is tempting to say conformance kept no record: `vibeos-ledger.jsonl` holds **19,711 rows
    with ZERO conformance rows** (build 6152, smoke 4741, check 3543, gate 1265, farm 1194,
    land 1087, difftest 692), because `_ledger_append` was a `Run` method and `conformance.py`
    does not use `Run`. But conformance was recording all along, via `_append_round`
    (`conformance.py:299`) into `~/vibeos-flake-farm/conformance.jsonl`.

    **Two ledgers; the difference was which DIRECTORY the restore covered.** `vibeos-ledger.jsonl`
    lives beside the worktrees under `Projects/os-playground/` and came back whole.
    `~/vibeos-flake-farm/` is under `$HOME` and did not come back at all. Not a flush, fsync or
    format problem — conformance was writing diligently, to the location that evaporates.
    **Anything else written under `$HOME` rather than into the project tree is in exactly that
    position right now**, and that is the transferable half of this.

    Fixed in `b10ce2351ee`: `ledger_append` extracted to module level so a command with no `Run`
    can record an outcome at all, and conformance now appends a durable row carrying `shape` —
    the mode classification — alongside launched/ok/bad/bugcheck/regressions. `shape` is the
    load-bearing field: it separates a lost boot from a flaky assertion AT WRITE TIME, so the
    next census never has to reconstruct the two-mode partition this one had to discover.

    **Consequence: the `ExPools` prediction below is currently UNGRADEABLE** — it stands as
    stated, but the baseline it would be graded against is gone and rebuilding one needs gates.
    Saying so plainly beats leaving it looking falsifiable.

    **Falsifiable once a corpus exists again:** if the `POOL_HEADER` fix is what closed
    `ExPools`, the next census must show it dropping off the top of the mode-1 column. If it
    does not, that fix is not what closed it. Note what such a census still could NOT settle:
    **"regressed in SOME runs of a batch" is not "nondeterministic cause".** Before its fix
    `ExPools` misread the header on EVERY boot; what varied was which of `ProcessBilled`'s bits
    happened to be set. A deterministic fault with a variable observable looks exactly like a
    flake in a pass/fail tally.

  - **`CmSecurity` and `KeDpc` on arm64 — NOT flakes. Mode-2 collateral, filed so nobody
    registers them as intermittents.** Both were nominated for this registry off the first
    (wrong) version of the census above, which pooled the two failure modes; both had zero
    prior mentions anywhere in this document, and that absence was correct.

    Split by mode over the same n=201: **`CmSecurity` has ZERO mode-1 occurrences** — all 7 of
    its batches are lost boots — and **`KeDpc` has exactly 1**. Neither has been observed
    failing in an ordinary sweep in any quantity that means anything. The apparent batch counts
    came from the arithmetic, not from the tests: a mode-2 run (`BUGCHECK 0x1E`, `ok`/`bad`
    both 0) reports the ENTIRE baseline as `in baseline (OK) but never launched`, so it adds a
    constant +8 batches to *every* test in the baseline at once. Seven more names arrived the
    same way — `ExCallback`, `ExDoubleList`, `ExUuid`, `IoFilesystem`, `ZwMapViewOfSection`,
    `HalSystemInfo`, `RtlCaptureContextKM`, `FsRtlTunnel` — all at 0 mode-1.

    **The generalisable trap: a failure that reports per-ITEM but fails per-RUN will manufacture
    N findings out of one event, and they look like N independent corroborated findings because
    each carries its own count.** The tell is a constant: every one of these tests showed
    *exactly* 8 mode-2 batches, because there were exactly 8 lost boots. Any per-item tally
    drawn from run-level artifacts should be checked for a shared constant before it is
    believed. Registering these would have filed nine tests nobody has ever seen fail, and each
    would then have cost someone an investigation.

  - **`Example` on arm64 — a THIRD undocumented intermittent, found 2026-09-13.** One
    occurrence in 9: `pass 31 -> 29`, NEW failing `Example_user.c:39` and `:40`. `Example` is
    **31/15 on all three floors** and neither line is marked `assert?` anywhere. Unrelated to
    the ExPools POOL_HEADER mismatch — `Example_user.c` does not touch `KmtGetPoolType`.
    **LOCALISED, and unlike ExPools this one is a REAL KERNEL DEFECT, not a harness artifact.**
    `:39`/`:40` check that an `IOCTL_SEND_MYSTRUCT` round-trip came back MODIFIED — the driver
    writes `{456, "!!!"}` into the second half of the buffer and sets `*OutLength`
    (`Example_drv.c:181-185`). In the failing round user mode read `MyStruct[1] = {0, ""}`.
    The driver side was entirely healthy: **both rounds executed exactly 46 assertions**
    (`Example: 46 tests executed`, 17 failures vs 15), and none of the handler's own
    diagnostics printed in either — not `ok_eq_ulong(*OutLength, 2 * sizeof)` at `:177`, not
    the `skip("Cannot write to buffer!")` guard at `:181`. Equal assertion counts are what
    rules out the competing explanation: had the handler not run, its `ok_*` calls would be
    missing from the total. So the handler ran, saw correct lengths, and performed the
    `RtlCopyMemory` write-back — and the data still did not reach the user buffer. That
    localises it to the **METHOD_BUFFERED output copy-back on IRP completion** (SystemBuffer
    -> UserBuffer, length from `Irp->IoStatus.Information`), intermittent on arm64, with
    `DeviceIoControl` returning ERROR_SUCCESS throughout — `:36`/`:37`/`:38` all still pass, so
    every caller believes it worked. n=1, so no rate; the localisation is what is solid.
    **Narrowed further by source inspection (no new run needed).** The copy-back is
    `iomgr-rust/src/irp.rs:555-577`: under `IRP_BUFFERED_IO | IRP_INPUT_OPERATION` and a
    non-error status it does `safe_copy_raw(UserBuffer, SystemBuffer, IoStatus.Information)`
    and, **on failure, sets `STATUS_ACCESS_VIOLATION`**. `Example_user.c:38` asserts the
    `DeviceIoControl` returned `ERROR_SUCCESS` and it PASSES in the failing round — so the copy
    did not fault, and the fault-handling branch is not involved. That leaves exactly two
    shapes, both of which are silent and both of which match zeros in the user buffer:
    (a) `IoStatus.Information` was **0** at completion, so a zero-byte copy "succeeded" and the
    user buffer was never touched — note the driver sets `*OutLength = 2 * sizeof` at
    `Example_drv.c:184`, so this would mean that value failed to reach `Information`; or
    (b) the copy wrote to a **valid but wrong address** — the buffered copy-back runs in a
    special-kernel APC, and in the wrong process context the same user VA can be mapped and
    writable, so the write silently lands in another address space rather than faulting.
    **One datum separates them**: `IoStatus.Information` and the current process at the moment
    of the copy. The serial log cannot supply it; a breakpoint at `irp.rs:568` on a live
    soak can, at the observed ~1-in-9 rate — once `0280df67353` lands, per the trap below.
    **Record/replay can now too (2026-10-05):** an arm64 kmtest recording replays the whole
    sweep deterministically (docs/engineering/qemu-record-replay.md), so a recording that
    catches the bad round is a session: `rr replay --break-at` near it and read the IRP.
    **Two traps for whoever picks this up, both of them absences that mean nothing.** First,
    `Example_drv.c:227`'s `IRP!` trace does NOT print for these device-control IRPs in either
    round, so its absence is not evidence the IOCTL never arrived — the clean round looks
    identical there and passes. Two independent reasons, both worth knowing before re-digging
    it: it is a `DPRINT` (debug-only, unlike the `trace()` calls beside it that DO print), and
    `TestIrpHandler` only ever sees the Example device's CREATE/CLOSE anyway — the three IOCTLs
    go to **kmtest_drv's** device, whose `DriverIoControl` sets `Information = Length` at
    `kmtest_drv.c:541` and completes. That is the `Information` in shape (a).
    **Third trap, if you go near `qemu/symbols-*.tsv`:** it holds **linked-image addresses
    with preferred base 0x400000, not RVAs**, so treating an entry as an RVA breaks 4MB past
    the function and the breakpoint never fires — another absence that means nothing. Found by
    smp-bringup, which spent two rounds attributing it to replay before the arithmetic.
    **Instrument note, because the obvious one does not fit:** a breakpoint at `irp.rs:568`
    fires on EVERY buffered IRP completion — thousands per boot — so `run-until.sh --break` is
    not usable there without a condition, and the ~1-in-9 rate means a lot of boots. The
    instrument that fits is a filtered debug print in the copy-back (log `Information` and the
    current process when the completing device is kmtest's), which is a kernel change plus a
    soak rather than a debugger session. Second, **every kernel symbol resolution on this
    path is against the wrong image unless you name `ntkrnlmp.exe` explicitly** — by hand as
    much as by tool. Since the 2026-09-12 MP promotion the media boots `ntkrnlmp.exe` +
    `halmp.dll` (`qemu/freeldr.ini:2`/`:51`), but `build-arm64/ntoskrnl/*.exe` contains ONLY
    the UP `ntoskrnl.exe`; the MP image is one directory deeper at
    `build-arm64/ntoskrnl/ntkrnlmp/ntkrnlmp.exe`. A glob or a tab-complete in the obvious
    directory therefore hands you the wrong kernel, and **it answers instead of erroring**.
    `0280df67353` fixes `find-kbase.py` only, so gdb breakpoints will time out silently until
    it lands — looking exactly like "the copy-back path is never reached", the wrong
    conclusion — but hand symbolication stays exposed afterwards. Break on something you know
    executes before believing any silence, and pass the MP binary by name.
  - **amd64/i386 count drifts in `NpfsReadWrite` and `KeEvent` — ROOT-CAUSED and FIXED
    2026-10-03.** The worst live flake at the time: 16 amd64 NpfsReadWrite hits in 8 days,
    plus 10 KeEvent hits on i386, all at a different assertion each time. The cause was
    winlogon's boot-time ladder (WoW64 guests, then stock i386 applets under FEX) running during
    the sweep and holding a uniprocessor guest's only CPU past the tests' 100 ms windows.
    `[NPFS-WAIT]` shows the miss as `idle 0`, against `idle 109` for an expected block.
    Conformance boots now pass the per-boot tripwire token `QUIET`, and winlogon skips the
    ladder. Smoke and the gate's ladder lane still run it from the same media. amd64: 4 quiet
    rounds with 0 misses, against 10 loud rounds with misses in 8. Field manual: "A kmtest
    count drift on a UP guest is the boot-time ladder, not the scheduler". MmMdl's amd64 drift
    was a separate thing, already settled by `c8228e5` (the count depends on free system VA).
    **`KeEvent` RECURRED after the fix (found 2026-10-05):** 7 count drops (6 or 8 fails, once
    1) in sweeps whose serial log carries `QUIET`, so the ladder is not the whole cause. i386:
    `20261003-225707-54bluib3`, `20261004-092328-21kqrgdc`, `20261004-111018-zzsx3q0o`,
    `20261004-165018-vyz3y237`, `20261005-193425-lqtlcoq8` (a queue gate attempt); amd64:
    `20261004-073303-11khe67v`, `20261004-171841-f8gc6gdx`. Failing asserts are the
    timeout/thread ones (`KeEvent.c:201/205/207/223`). Counted with the 2026-10-05 parser,
    which no longer mistakes serial interleaving for regressions.
  - **`NpfsConnect` on arm64 — an UNDOCUMENTED intermittent, found 2026-09-13.** Did not recur
    in the 9-sweep soak, so it is rarer than the other two. `pass 679 -> 678, fail 0 -> 1` on
    `20260913-022245-fw6sw4_g-conformance-arm64`.
    **RECURRED 2026-09-23, and the second occurrence carries the lead the first could not:
    it looks LOAD-CORRELATED, not merely rare.** Identical signature, in
    `queue-batch-01790201087033273531/20260923-180447-8li01qbu-gate-i386+amd64+arm64`
    (`conf-arm64-a1/conformance.log`). It failed **3 of 3 gate attempts** while the gate was
    running 3 builds + 4 boots, *and* while a characterisation sweep was booting concurrently
    in another worktree — then passed **2 of 2** on the same tree on a quiet host, including
    the exact merge the gate tested. 3/3 under load vs 0/2 quiet is not the profile of a rare
    intermittent; it is the profile of a race that needs contention to lose. Next step is to
    reproduce it deliberately under load (`farm` alongside a full gate) rather than wait for
    another sweep to catch it.
    **Cost of NOT knowing this:** the conductor rejected an innocent 17-commit batch and
    concluded "the BASE is GREEN — the submission really is the culprit" from a **single**
    base sample against three lane samples, for a failure its own classifier had already
    named `CONFORMANCE_FLAKE`. That attribution is the same shape as the harness bugs this
    lane spent the day fixing: blame what was in flight rather than what was proven. `NpfsConnect` is **679/0 on all three
    floors with no `assert?` anywhere**, so this is exactly the case the ratchet quest warned
    about — an intermittent that never fired during the 15 seeding boots is indistinguishable
    from a regression on any single sweep.
    **DIAGNOSED 2026-09-13, from source, at zero machine cost — and it is a TIMING DEADLINE,
    not an npfs defect.** Now 2 hits in 8 arm64 rounds the same day (~25%, so not rare after
    all — the "did not recur in 9 sweeps" above was luck). Both hits are the identical
    identity, `assert NpfsConnect.c:109`, and that line is
    `ok_bool_true(Okay, "CheckConnectPipe returned")` for the SECOND client connect.
    `CheckConnectPipe(..., 100)` -> `TriggerWork` -> `WaitForWork` (`NpfsHelpers.c:692-707`)
    is a **relative 100 ms** `KeWaitForSingleObject`, and it returns FALSE *only* on
    `STATUS_TIMEOUT`. **The discriminator is the neighbour, and it is the ExPools lesson
    inverted:** `:110` (`ok_eq_hex(Status, STATUS_PIPE_NOT_AVAILABLE)`) PASSES on the same
    run, and `Status` held `STATUS_SUCCESS` going in from the `:97` connect — so the worker
    thread did signal, wrote the CORRECT status, and merely did it later than 100 ms. There an
    assertion that passed on garbage hid a defect; here an assertion that passed on good data
    proves the work was right. **Deliberately NOT baselined and NOT widened.** Widening a
    timeout is the classic weaken-the-test move, and baselining it would bury the one thing
    that might be real: 100 ms for a thread wake plus a pipe connect is generous even under
    TCG, so exceeding it may be genuine scheduler latency in our kernel rather than an
    over-tight suite deadline. **Those two are not separable by more sweeps** — the pass/fail
    bit saturates at "over 100 ms" either way. The discriminator is the ACTUAL delay, which
    needs an instrument (record the wait's elapsed time on the timeout path), not more boots.
    That is the next step whenever this is picked up; do not re-soak it.
    **INSTRUMENTED 2026-09-26 (`66668384be1`, diagnostic only) — and the first catch is a
    SCHEDULER STARVATION, not a slow pipe.** `[NPFS-LATE]` stamps trigger / worker-woke /
    work-done and prints when an item finishes past its caller's deadline. arm64 round 3 of
    3 at that commit regressed NpfsConnect harder than ever recorded (pass 679 -> 606, fail
    0 -> 73) and printed `work done 149890 ms after trigger (deadline 100 ms): woke +149890
    ms, work 0 ms`, followed immediately by `[CENSUS] wait profile: 29 thread(s) parked
    >20s`. Two artifact readings were checked and ruled out: it was the FIRST late item of
    the test on a fresh worker (so not a previous diagnostic print still running), and the
    round ran `KeStartAllProcessors: 1 processor(s) online` (so not cross-CPU clock skew,
    and not the CPUs-2/3 bug). A ready kernel thread did not run for ~150 s on ONE CPU.
    Evidence: `/tmp/vibeos-cli/20260926-175208-lvwp5qhv-conformance-arm64/` (preserved to
    `~/vibeos-flake-farm/fails/auto/`).
    **ROOT-CAUSED 2026-09-26: arm64 had no preemption, and the zero-page thread held the
    CPU.** Host-stamped serial (conformance now stamps every round) showed the stall in
    EVERY arm64 round, not only the failing one: ~95 s inside NpfsConnect with the guest
    clock ticking and nothing scheduled. The 150 s round just had a worker item straddling
    it. vCPU samples taken during the quiet (`silence-samples.txt`) sat in
    `MmZeroPageThread` -> byte-wise `memset`. arm64's DISPATCH softint parked readied threads
    in `Prcb->NextThread` and never switched to them, so the priority-0 thread ran until the
    free list was empty. Fixed on `conformance-ratchet` (arm64 `KiDispatchPreemptArm64` from
    `KfLowerIrql`, DAIF restored on resume, `KeZeroPages` via `dc zva`). arm64 conformance
    rounds went from 175-255 s to 72-107 s, with zero silences, zero kmtest AWAIT timeouts,
    smoke 3/3 and the ratchet holding. **Owed:** the arm64-only ReadySummary recheck in
    `ke/wait.rs` and `ke/thrdschd.rs` (×2) existed because there was no preemption.
    Instrument it for dead, then remove (archive/arm64-interrupts-quest.md). Preemption at
    IRQ exit for a spinner that never crosses DISPATCH: **FIXED 2026-09-30 for EL0**
    (`KiEL0IrqPreempt`, run from `KiEL0IrqEntry`: unmask, then raise to DISPATCH and lower
    back to PASSIVE). Found by the `tpprobe` payload, where an NT threadpool worker spun on
    one CPU for a condvar lock held by the thread it had preempted, and the guest froze. A
    spinner at EL1 below DISPATCH is still unpreempted: that IRQ exit runs in exception
    context, which is the §9.19 hazard.
    **Known noise in the instrument:** it cannot tell a MISSED deadline from an item the
    test EXPECTS to block (a listen with no client yet), so clean sweeps print ~38 lines of
    `woke +0 ms, work 109 ms` (x86) / `work 200 ms` (arm64). The `woke` field is the signal;
    a large `work` on a clean run is by design.
    **Scope, so this table is not misread:** the 3-in-9 is the rate for a **conformance sweep**
    (`conformance:<arch>`), not for a gate. Since `9f73d184fea1` an arm64 gate ASKS for the
    WoW64 ladder (`gate_argv=['--ladder']`, previously silently empty), so `LADDER_FAIL` is a
    fourth possible source of arm64 gate redness that is **not** one of these intermittents and
    not a new defect — a real check newly switched on, whose salient lines name the guest.
    **But it does not run on default media, and the measured cost is zero there.** Verified on
    a full 3-arch gate with `9f73d184fea1` in HEAD: all three arches SKIPPED at 0.0s —
    `ladder:arm64` and `ladder:i386` for *"no WoW64 CPU layer on this arch's media"*, and
    `ladder:amd64` for *"no staged syswow64/ guest in this arch's media manifest"*. arm64's
    WoW64 layer is the `VIBEOS_FEX=1` opt-in, so a default arm64 gate declines the ladder and
    pays nothing. The `ladder drain: 84.7s` figure is from a tree built WITH that opt-in — real,
    and not the default case. So expect the fourth red surface only on a FEX-staged tree; on
    default media the stage is a skip with a reason string.
    **Why it matters more than one more flaky row: arm64 conformance intermittents COMPOSE,
    and the retry budget is sized for one.** Gate `20260913-021807-fbitjxpy` burned all three
    arm64 attempts on TWO different bugs — `ExPools` (a1), `NpfsConnect` (r1, clean on
    ExPools), `ExPools` (r2). The retry does not need the same test to fail three times, only
    SOME regression each time, so independent intermittents multiply the all-attempts-fail
    probability well past what a 3-attempt budget was designed to absorb. Nothing in the final
    verdict shows that the three attempts disagreed, which is the information that would
    distinguish this from a real regression: **a lane's diff reproduces the SAME regression
    every attempt; independent intermittents produce a different one each time.** That
    distinction is computable from the per-attempt `salient` lines the gate already captures,
    and is the discriminator `CONFORMANCE_FLAKE` currently throws away — `gate.py:319` assigns
    that classification as a CONSTANT to every ratchet failure, so a regression reproducing
    3/3 carries the same label as a coin flip, and `queue.py`'s `INCONCLUSIVE` set (which does
    not contain it) therefore rejects the lane and blames its diff either way. Open gating-policy
    question for the owner; do not "fix" it by adding `CONFORMANCE_FLAKE` to `INCONCLUSIVE`,
    which would make every genuine conformance regression halt the drain as infrastructure and
    leave the ratchet unenforceable through the queue.
  - **`ZwAllocateVirtualMemory` 843 → 839 — a real UP-vs-MP DIFFERENCE, not a regression, and
    its baseline is a 1-in-101 sample.** Correlating pass count against `smp` over every banked
    i386 run dir: **smp1 scores 843 in 4/4 runs; smp2 scores 839 in 100 runs and 843 in exactly
    one.** That one is `20260726-005733-conformance-i386` — the same day `b957fb2229d` re-seeded
    the i386 baselines. So the smp2 baseline captured the rare outcome and the MP kernel has
    been losing these 4 assertions all along; nothing got worse over time. (An earlier revision
    of this entry called it "the only genuine one" and framed it as a regression since
    2026-07-26. That was wrong, and it was wrong because the first comparison used one smp1 run
    against one smp2 run — **always correlate against `smp` before calling an i386 delta a
    regression.**) The SHAPE is still the interesting part: pass drops by 4 while fail stays at
    2, so 4 assertions stop EXECUTING rather than start failing. Counts alone do not say
    why: the arm64 `KeEvent` 132-pass drop below comes from an explicit MP test exclusion.

  **ONE BOOT SEEDED TWO OF THESE ROWS.** `20260726-005733-conformance-i386` is the only smp2
  run in the whole corpus that produced the rare side of `FsRtlLegal` (2504) *and* the rare
  side of `ZwAllocateVirtualMemory` (843), and it is the run behind `b957fb2229d`'s re-seed on
  2026-07-26. Two rows, one lucky boot, and both have read as regressions in every sweep since.
  (The two rare outcomes co-occur — `FsRtlLegal`=2504 never appears without `ZwAlloc`=843 — so
  whatever varies boot-to-boot moves both, which is itself a lead worth pulling.)

  **The meta-finding, and it is the actionable one: every row in these baselines is a SINGLE
  SAMPLE of a nondeterministic quantity, and the ratchet treats it as a floor.** All three
  entries above reduce to that. `FsRtlLegal`'s smp2 row, both `IoDeviceObject` rows and
  `ZwAllocateVirtualMemory`'s smp2 row were each seeded from one boot of a test that does not
  return the same number every boot — twice from the lucky side, once from the unlucky one.
  `--write-baseline` cannot see this because it runs the sweep once. Until a baseline can be
  seeded from N rounds and record a *distribution* (or the underlying nondeterminism is fixed),
  re-seeding is not a fix for any of these — it just re-rolls the dice.
  - amd64 also showed `MmMdl: OK -> NO-END` once in 16 (a hang, not a fail) — worth watching,
    single occurrence, not yet a claim.
  Deterministic-vs-flaky is only separable because rounds are repeated on one frozen media and
  recorded per-round; a single sweep reports all five of these identically.
- **arm64 sweep failures — SUPERSEDED by the `MmDeleteRmap` root cause below; kept only for the
  corpus statistics.** (2026-07-29, from the 85 banked arm64 smp1 rounds in
  `~/vibeos-flake-farm/conformance.jsonl`.) **The concentration on
  `ZwWaitForMultipleObjects` is NOT the bug**: `747d5a28559` moved that test from position 71 to
  position 2, it ran clean there, and a round still died right after `ZwMapViewOfSection` — so
  the victim is incidental and section teardown is the trigger. Anything below that reads as
  "the failures concentrate on one test" is an artifact of position, not a lead. Only 42/85
  rounds finish. The distribution:
  - 20 rounds stop at `launched=71`, 4 at 75, 2 at 73, 1 each at 80 and 90 — i.e. clustered at
    whatever test happens to follow the section work, which is the point above.
  - **13 rounds have `launched=0`** — the sweep never started at all. A separate face; do not
    pool it with the crashers.
  - Bugchecks across the cluster: `0x1E` ×10, `0x1A` ×8, `0x50`, `0xE3`.
  - **The faulting PCs say these are DETERMINISTIC fault sites, not memory corruption.**
    Reading P1/P2 off all 20 banked crash serials, with `[KRNLBASE] ntoskrnl runtime
    base=0xFFFFF80042190000` from the same logs:
    - `0x1E` / P1=`0xC0000005` (ACCESS_VIOLATION) at `ntoskrnl+0x84df8` (twice, byte-identical),
      `+0xc2b60`, `+0x7da74`, `+0x89184` — legitimate in-ntoskrnl addresses, and repeats.
    - `0x1E` / P1=`0xC000001D` (ILLEGAL_INSTRUCTION) at `0xd4dfd99b5dde7df2` — twice, the SAME
      non-canonical value, which reads as a specific poison pattern rather than arbitrary
      stack damage; plus three more at `0xffff8000043xxxxx`.
    - `0x1A` ×8 with all four parameters ZERO — that is the double-map probe, which passed no
      parameters before 2026-07-29.
    Repeated identical PCs are the point: random stack corruption does not land on the same
    address twice. **Do not chase this as a smashed return address** (I did, briefly, off a
    single `0xd4dfd99b...` sample before checking the rest — `0xfffff800…` is a perfectly
    normal kernel VA here and a naive "is this a kernel address" test mislabels it).
    **Symbolicate against the SMP LANE's build, not yours** — those offsets come from that
    lane's tree at its own commit, and `qemu/addr2sym.sh` against a different build answers
    confidently and wrongly (it returned `KTRAP_FRAME_LENGTH+543896`, the task #32 shape).
  - The `0x1A` face is `MmCreateVirtualMapping`'s double-map probe (`[PAGE] BUG: old PTE was
    non-zero!`). That probe printed one string and died, so all 8 taught us nothing; as of
    2026-07-29 it prints the VA, old/new PTE, old/new PFN with a SAME=YES/NO discriminator,
    process and thread, and dumps that VA's own map/unmap ledger from the flight recorder
    (whole-ring tail dumps are useless here — on the motivating crash the last 32 of 643,270
    events were 100% working-set lock churn).
  This is the §9.98 physical-page-reuse cluster, task #19/#23, **OWNED BY the smp-bringup lane
  — and CLOSED 2026-10-03.** The root cause was fixed on 2026-08-01 (`MiPurgeStaleUserPtes`,
  see "stale FreeLDR identity PTEs in recycled user page tables" below); what was still owed was
  evidence that the SYMPTOM had gone, because that fix proved only that the oracle went quiet and
  that `ZwMapViewOfSection` gained two passes.

  Measured now: **639 arm64 `-smp 1` kmtest rounds (2026-09-14 → 10-03) carry 0 `BUGCHECK 0x50`
  and 0 `0x1A`** — the cluster's two stop codes — **and 0 short sweeps out of the 582 with a
  known total**, against the pre-fix 42/85 that finished. One fresh sweep on probe-bearing media
  launched 93/93 with the ratchet holding and **0 hits on every armed MM site** (`MMGUARD`,
  `FREEMAPPED`, `PFNMISMATCH`, `KSTACK`) across 15,144 serial lines, with the probe's presence in
  `qemu/ramdisk.img` verified first — the July retraction is the reason that check is not
  optional.

  Two methodological notes from doing it, both of which cost time:
  - **The flake ledger cannot answer this by grep.** `conformance.jsonl` stores `launched`, `ok`,
    `bad`, `bugcheck` and `shape` plus a *pointer* to the serial log — no serial text. Grepping
    it for `C000001D`/`MMGUARD` returns 0 for every input and means nothing. Use `launched` vs
    `sweep_done total=`, and `bugcheck`.
  - **Still genuinely owed, and small:** `MmInsertRmap`'s two early returns (`mm/rmap.rs:453`,
    `:476`) still `return` where the C `KeBugCheck`s. They are loud now (`mm_guard_trip!`) and
    probed as never firing, so they are not this bug — but they remain silent degrades.
- **`KeEvent`'s 132-pass UP→smp1 drop is intentional test coverage, not an event failure**
  (corrected 2026-09-06). `modules/rostests/kmtests/ntos_ke/KeEvent.c:85` explicitly excludes
  its `Wait=TRUE` block when `KmtIsMultiProcessorBuild` is true: the test's manual cleanup
  would leave the MP dispatcher lock held. Each of its two operations runs nine assertions
  in `CheckEvent` plus two direct assertions, for **3 IRQLs × 2 event types × 2 operations
  × 11 assertions = 132**. This exactly explains `42733/0 → 42601/0`; it is not an early
  loop termination. Verified against `/tmp/vibeos-cli/20260906-211400-conformance-arm64/`
  and the committed UP baseline. Tests and baseline remain unchanged. The separate
  `ExPools` fluctuation remains tracked in its entry above.
  The same run's `IoInterrupt` delta is different: **156 assertions execute and fail** at
  `IoInterrupt.c:42`, expecting the held lock to equal `1`. The observed value is
  `0xfffffa000610d8d1`, the running kmtest thread `0xfffffa000610d8d0 | 1`, matching the
  checked-build owner stamp in both `ntkrnl-sys/src/inline/ke.rs` and C `spinlock.h`.
  Preserve this reported discrepancy; it is not evidence that the lock was left free.
- **i386 does not LINK on the merged tree — three fastfat ABI breaks, TRAIN BLOCKER, 2 of 3
  OPEN** (2026-07-28). `ld.lld: undefined symbol: _FatAddMcbEntry@20`, `_FatStringTo8dot3@12`,
  `_FatBreakOnInterestingIoCompletion`. Present in every i386 build tonight including the
  first, and the defining commits (`2cd4e781d1c` and the fastfat batch) are from
  **origin/master**, so it arrived with the merge, not from the smp lane. *Not* verified
  against master standalone — that check is owed.
  **The generalisable part:** i386's stdcall name decoration bakes the argument-list size into
  the symbol (`@N`), so it is a **free ABI checker that the 64-bit arches do not have**.
  amd64/arm64 link all three of these GREEN while passing a wrong argument layout. When an
  i386-only `undefined symbol: _Foo@N` appears, the bug is nearly always a real cross-arch ABI
  mismatch, not an i386 quirk — read the `@N` delta as the byte-count delta it is.
  - `FatAddMcbEntry` — `fsctrl.rs` widened `Lbo` u32→i64 (LBO is LONGLONG; "for Fat32, LBO is
    >32 bits", fat.h:34) and fixed `allocsup.rs`, but `fileinfo.rs:147` and `strucsup.rs:95`
    kept `i32`/`u32` ⇒ callers want `@20`, definition exports `@24`. **FIXED** (both decls +
    the call-site casts).
  - `FatStringTo8dot3` — `fileinfo.rs:100` declares `OemName: *mut OEM_STRING` where
    `namesup.rs:96` defines it **by value** (4+4+4=12 vs 4+8+4=16), and the third parameter
    disagrees as well: `*mut OEM_STRING` vs the definition's `*mut [u8; 11]` (an 8.3 name
    buffer, `&Dirent->FileName` in the C). That is a translation question, not a decl typo.
    **OPEN.**
  - `FatBreakOnInterestingIoCompletion` (and its `…ExceptionStatus`/`…IrpCompletion` siblings)
    — `fatdata.rs` defines them under `#[cfg(debug_assertions)]` **and without `#[no_mangle]`**,
    so they export no C symbol under any configuration, while `deviosup.rs:76/:1811` declares
    and reads one unconditionally. **OPEN.**
- **`NtWriteVirtualMemory` reports `STATUS_SUCCESS` + a full byte count while writing NOTHING —
  OPEN** (2026-07-26, task #76, found by E17's `selfmod32.exe` rung 7). `MiDoPoolCopy`
  (`ARM3/virtual.rs`) translated the C's `_SEH2_TRY/_SEH2_EXCEPT` as straight-line code:
  `Status` is initialised to `STATUS_SUCCESS` and **never assigned anywhere in the function**,
  so both `if Status >= 0 { copy }` guards always pass and the tail unconditionally does
  `*ReturnSize = BufferSize`. Proven on arm64: a write to a page FEX had write-protected for
  SMC detection (`VirtualQuery` → `PAGE_EXECUTE_READ`) returned `0x00000000` with 6 bytes
  written and the target bytes unchanged. *Refusing* the write is correct — that is exactly why
  `WriteProcessMemory` wraps it in `NtProtectVirtualMemory` — reporting success is not.
  `MiDoMappedCopy` probably shares the shape. The real fix needs probe/copy faults to be
  observable in translated Rust (`docs/engineering/dummy-pseh-verdict.md`;
  `ntkrnl-sys/src/probe.rs` only range/alignment-checks today). `selfmod32` prints
  `rung7 BUG: …` on serial but deliberately does **not** gate on it, so the WoW64 ladder stays
  a WoW64 signal.
- **PFN `CacheAttribute` was never initialised → `VirtualProtect` made private pages uncached —
  FIXED 2026-07-26** (task #75, field manual "An uninitialised PFN `CacheAttribute`…").
  `MiNonCached` is 0, so every page off the free list read as deliberately-uncached and
  `MiFlushTbAndCapture` force-ORed `MM_NOCACHE` on every protection change. Harmless-ish on x86
  (a perf loss); on arm64 `MM_NOCACHE` = MAIR AttrIndx 1 = Device-nGnRnE, where every unaligned
  access alignment-faults — a livelock with the PC pinned. Upstream ReactOS has the same gap.
  Fixed by setting `MiCached` in the three `MiInitializePfn*` paths, as Windows does. Very
  likely also the source of the long-standing `[ALIGN] FAR=0x1D11Bxx / 0x7E0B377E` diagnostics
  in other WoW64 guests, which FEX's unaligned handler had been absorbing.
- **`mscms-rust` — being integrated on the `integration` lane, NOT a bug.** Surfaced
  2026-07-25 when `scan_orphan_modules` learned to see ROOTLESS crates: 6 files / 4,782 lines
  with no `Cargo.toml` and no `lib.rs`, which every source gate had been skipping silently
  (`crate_root_of` walks up to the nearest Cargo.toml — for a crate with none that is the
  *workspace* manifest, and the "unparseable crate → fail open" branch swallowed the lot).
  Baselined in `tools/lint-baseline/scan_orphan_modules.txt`, and **the ROOTLESS finding
  deliberately carries no line count** so their in-flight edits don't read as NEW findings.
  The 6 baseline entries expire on their own the moment a `lib.rs` lands — delete them then.
  Another lane owns these files; don't edit them from here.
- **`BUILD_INCOMPLETE` — OPEN, recurring ~5×/day, halts the merge-queue train; NOT arm64-only
  and NOT one bug** (2026-07-25; **triaged + instrumented same day**, task #21). `artifact-check`
  fails after ninja exited 0. Three archived occurrences are **three different oracles**:
  - `20260725-021909-build-arm64` — *"libkernel32_rust.a is newer than the built binary"*:
    `cohab_staleness_reasons` check B (artifact mtime vs the **copied** staticlib).
  - `20260725-022237-build-i386` — *"shell32-rust/…/shldataobject.rs newer than the
    cargo-linked binary"* ×4, for **lsass/services/smss/userinit**: `cargo_link_staleness_reasons`
    compares every cargo-linked artifact against a **global newest-workspace-`.rs` watermark**
    (`_newest_ws_src`). That `.rs` belongs to another lane and to a crate none of those four
    exes link — so one lane's new file marks another lane's untouched binaries stale.
  - `20260725-053107-build-arm64` — *"build source changed since build"*: `build_src_hash`
    differs from a stamp written **milliseconds earlier in the same function**.

  **Hypotheses tested and KILLED — do not re-run:** (a) *git calls hit the 120s timeout under
  load* — `git diff HEAD` is 10 ms and `git status --porcelain` 60 ms over 37,587 tracked files;
  (b) *`index.lock` contention fails the git calls* — held the lock and ran all three, every one
  returned rc=0; (c) the **previously recorded leading hypothesis** (*cargo re-emits the `.a` and
  bumps its mtime even when content is unchanged*) is aimed at the **wrong file** — check B
  compares against the **copied** staticlib, which cmake `copy_if_different` only touches when
  the bytes actually changed.

  **Real defect found and FIXED regardless of trigger:** `provenance._run` swallowed *both*
  failure modes — it caught `OSError`/`SubprocessError` returning `""` **and never checked
  `returncode`**. All three git calls behind `git_fingerprint` went through it, as did the
  `git grep` behind `cargo_module_rels`. So a degraded git call either shifted the fingerprint
  (false RED = this bug) or emptied the cargo-cell list (false GREEN = the staleness net
  silently off) — and logged nothing either way, which is why 5 occurrences a day left no
  evidence. Now raises `ProvenanceError` naming command + rc + stderr; `check_staleness`
  reports that as *"provenance oracle FAILED (verdict unknown, not stale)"* instead of
  laundering it into a confident "source changed". `git grep`'s rc=1 "no matches" is declared
  via `ok_codes`.

  **The next occurrence self-diagnoses.** The stamp now records `head_sha`, so a HEAD move
  under the build is named as such; the bare "source changed" reason now appends *what*
  changed, and says **"suspect the oracle, not the tree"** when the fingerprint moved while
  HEAD held still and nothing is modified. Both mtime verdicts now print the gap: **under ~2 s
  = same-run ordering (a race, i.e. a false positive), minutes = a genuinely missed rebuild** —
  the discriminator these messages never carried. Stays in `queue.INCONCLUSIVE` (halts the
  train rather than rejecting an innocent lane) until an instrumented occurrence names the
  cause. Artifacts: `/tmp/vibeos-cli/queue-batch-*/2026*-build-*/summary.json`.
- **Tree-wide bindgen callback-slot normalization — SCOPED, OWED** (task #14). The tool exists
  and is proven end-to-end on buslogic: `tools/rustconv/normalize_island_abi.py` spells callback
  slots `extern "system"` across a crate's three islands, projecting from the i386 island (the
  only one that still records which callbacks are stdcall). Applying it repo-wide would touch
  **17,612 slots across 37 crates** and surfaces **86 E0308s in ntoskrnl alone** — every one a
  real convention-spelling mismatch that is currently invisible, so it is a batch of its own,
  not a rider. The payoff is measured: on buslogic it let 6 `transmute(f as usize)` callback
  installs become plain `Some(f)`, and reverting a definition to `extern "C"` now fails to
  compile instead of leaking 8 bytes per SRB. Remaining exposure is ~500 transmute-shaped
  installs — dxg-rust 92, win32k `ntddraw_dxeng` 42, cdfs `cdinit` 33, winsrv console frontends
  ~90, hal `bussupp`/`dma` 42, cdrom 20, scsiport 15. Full writeup: field manual, "A `transmute`
  at a callback-install site erases the CALLING CONVENTION".

  **Generation is now wired (2026-07-26).** `gen_driver_types.sh` calls the pass
  unconditionally after all three `run_arch` calls — placement is forced, since i386 is the
  witness and a per-arch hook cannot see it — scoped by a new `--stem` so a run normalizes only
  the islands it wrote (netapi32 carries `types`, `types_browser`, `types_rpc`; the old glob
  would have rewritten all three). `CALLBACK_SYSTEM=1` is documented as superseded but NOT
  removed: the projection keys on `Option<...>` aliases and struct fields, while the old sed is
  a blanket rewrite that may be carrying fn-pointer shapes the projection does not match, so
  dropping it could reintroduce E0308 in the seven crates that opt in. They compose safely (the
  sed leaves no `stdcall` on i386, so the projection no-ops there). Retiring it means
  regenerating those seven and fixing the fallout — still owed.

  **The bug yield is measured, and it is small (2026-07-26).** The alarming part of this task
  was never the 17,612 slots, it was not knowing how many hid a real convention bug. Audited
  statically — no build needed, which matters because the E0308s were only ever a *detection
  mechanism* for something checkable directly: **884 transmute-shaped installs plus 129 `cb!`
  installs across the four crates that still define that macro (usbaudio, ks, usbhub, usbport),
  yielding exactly ONE real bug.** `ApphelpShimCacheCompareRoutine` (ps-rust) was defined
  `extern "C"` against four agreeing authorities — `NTAPI` in `ntoskrnl/ps/apphelp.c`, `NTAPI`
  on `RTL_AVL_COMPARE_ROUTINE` in `ndk/rtltypes.h`, `extern "stdcall"` in bindgen's i386
  transcript, and the two sibling routines passed to the *same* `RtlInitializeGenericTableAvl`
  call. On i386 the AVL code expects the callee to pop 12 bytes; ours popped none, once per node
  visited on every shim-cache lookup. Fixed.

  The other seven candidates were false positives, and both classes are worth knowing because
  they are what a careless mechanical pass would have corrupted: storahci's `PHW_DPC_ROUTINE`
  and `PAHCI_COMPLETION_ROUTINE` are **genuinely cdecl** (undecorated in `storport.h` and the
  driver's private header; bindgen's i386 island independently says `extern "C"`), and ks-rust's
  four clock routines are a correct per-arch `fastcall`/`C` cfg split of a C `FASTCALL`. Both
  vindicate the projection's rule that i386-says-`C` stays `C` — and both are exactly what
  `CALLBACK_SYSTEM`'s blanket sed would have broken.

  **The sweep was under-counting by 10% (2026-07-26).** The pass looked for a hardcoded stem
  list, `("types", "bindgen")`, but `gen_driver_types.sh` names islands from a per-crate
  `TYPES_STEM` — so three stems were skipped in silence while the tool still printed a grand
  total: `hal_bindgen` (ntkrnl-sys, 642), `island` (shell32-rust, 342 — landed the same day) and
  `win_sys` (oleaut32-rust, 831). Stems are now discovered from the filenames, which also
  removed a latent soundness bug: one witness set was pooled across every island in a crate, so
  a `stdcall` entry in one could promote a same-named cdecl entry in another, projecting across
  a boundary where the key means different things. Per-stem grouping lowered no crate's count.
  **Tree-wide total: 17,717 → 19,532 slots / 39 crates.** The pass was also write-capable, wired
  into generation, and had **zero tests**; it now has eight, each validated by breaking the tool
  and confirming the intended control fires. Field manual, "An oracle needs an oracle".
- **8 `scan_cc_mismatch` entries in `ntkrnl-sys/src/bindgen_i386.rs` — benign, baselined**
  (`Ke386SanitizeFlags`, `Ke386SanitizeSeg`, `Ke{Query,Register}InterruptHandler`,
  `KiFillTrapFrameDebug`, `KiGetThreadNpxArea`, `KiSetTebBase`, `KiUserTrap`). The island
  declares them `extern "C"` (faithful — the C headers make them `FORCEINLINE` and
  undecorated, i.e. cdecl); our out-of-line definitions and all **19** hand-written callers
  across the i386 trap/interrupt/thread-init path consistently say `extern "system"`. Nothing
  calls through the island decl, so the Rust side is self-consistent and safe. Flipping the 8
  definitions to `"C"` was tried and reverted — it makes 19 call sites disagree in order to
  match a declaration with no callers. Resolves naturally when task #14 normalizes bindgen
  conventions.

- **`ExHardError` 6 failures — OPEN, NOT SMP-related, NOT amd64-specific, regressed on master
  since 2026-07-17** (found 2026-07-24 by the UP:=smp1 conformance experiment; **arch scope
  widened 2026-07-25** — arm64 reports the identical **24/6** split once its BuildType bug is
  fixed and the test runs at all, so the same six subtests fail on both arches and the cause is
  in the shared `ex-rust`/`winsrv-rust` hard-error path, not in anything amd64-specific. That
  also kills the "it's an amd64 LPC-path change" framing below as the *whole* story.)
  `ExHardError.c:139-144`
  — the six `CheckHardError(0x40000009..0x4000000d, …)` cases that raise a hard error with an
  **out-of-range `ValidResponseOptions`** (9, 10, 11, 12, `MAXULONG/2+1`, `MAXULONG`) and expect
  `ResponseReturnToCaller` (0). We return **1 = `ResponseNotHandled`** (the test's sentinel is 27,
  so the kernel really writes the 1). **Control-proven not-MP:** the UP kernel on today's tree
  reports the identical `pass 30 -> 24 / fail 0 -> 6`, and BOTH the amd64 UP and `smp2` baselines
  (seeded 2026-07-17) record a clean `pass=30 fail=0`. Mechanism is visible but the *trigger*
  commit is not yet identified: `winsrv-rust/src/usersrv/harderror.rs:1108-1115` sets
  `Response = ResponseNotHandled` then **early-returns** for `ValidResponseOptions >
  OptionCancelTryContinue`, and the kernel accepts that verbatim (`ex-rust/src/harderr.rs:214-229`
  lists `ResponseNotHandled` as valid — faithful to `ntoskrnl/ex/harderr.c:262-276`, so NOT a
  translation bug).

  **TRANSLATION-BUG HYPOTHESIS RULED OUT ON BOTH HALVES, by source comparison (2026-07-25).**
  Read line-by-line against the C originals:
  - `winsrv-rust/src/usersrv/harderror.rs:1097-1115` is a faithful translation of
    `win32ss/user/winsrv/usersrv/harderror.c:1097-1110`: the C *also* sets
    `Response = ResponseNotHandled` and *also* returns early for
    `ValidResponseOptions > OptionCancelTryContinue`. Identical behaviour, not a divergence.
  - The kernel's `ValidResponseOptions` validation (`harderr.c:586` switch /
    `ex-rust/src/harderr.rs:377` match) sits inside `if (PreviousMode != KernelMode)` in BOTH
    languages. kmtest is a **driver**, so it is a kernel-mode caller and skips that validation
    entirely — out-of-range values are passed through to the port BY DESIGN, in C too.
  - No duplicate-symbol trap: `ntos.cmake:69` has the C `ex/harderr.c` commented out (cohab
    flipped) and the Rust owns `#[no_mangle] ExpDefaultErrorPort` — exactly one symbol.

  **THIRD HALF ALSO RULED OUT, and the "only one path yields 0" claim above was wrong
  (2026-07-26).** The `PortHandle` computation itself (`ex-rust/src/harderr.rs:145-167`) is
  byte-faithful to `ntoskrnl/ex/harderr.c:155-181` — same `DefaultHardErrorProcessing &
  SEM_FAILCRITICALERRORS || ErrorStatus & HARDERROR_OVERRIDE_ERRORMODE` gate, same
  ExceptionPort→ExpDefaultErrorPort fallback, same two `PortHandle = NULL` overrides. All three
  halves of the translation are now checked; none diverges.

  There are in fact **four** paths that write `ResponseReturnToCaller`, not one. **Cite the
  RUST line numbers when instrumenting** — `ntos.cmake:69` has `ex/harderr.c` commented out
  (noted above), so the C file is inert dead source and a probe added to it fires never. Earlier
  revisions of this entry listed only `harderr.c:210/:280/:284`, which is exactly the trap
  AGENTS.md warns about. Live sites, all in `sdk/lib/rust/ex-rust/src/harderr.rs`:
  1. `:171` — `PortHandle == NULL` — early return, no LPC at all.
  2. `:231` — `LpcRequestWaitReplyPort` **fails** — kernel writes 0, returns the failure status.
  3. `:226` — LPC succeeds but the reply's `Response` is **not one of the 11 accepted values**,
     so the kernel clamps it to 0 — i.e. the server never wrote `Response`.
  4. `:185` — `Process == ExpDefaultErrorPortProcess && ErrorStatus < 0`, which routes to
     `ExpSystemErrorHandler` and then answers 0. Not previously listed here; unlikely for a
     kmtest driver (the default error port process is csrss) but it is a real fourth writer, so
     an instrumented boot must distinguish it rather than assume three-way.

  **The failure COUNT discriminates.** `CheckHardError` asserts twice per case
  (`ok_eq_hex(Status, STATUS_SUCCESS)` *and* `ok_eq_ulong(Response, ...)`), and we see exactly
  **6 failures across 6 cases** — one assertion each. So `Status` is `STATUS_SUCCESS` and path 2
  is ruled out *today*; the 2026-07-17 baseline passed BOTH assertions, so path 2 is ruled out
  *then* as well. **A broken LPC was never the explanation in either direction.** The baseline's
  clean pass was path 1 or path 3, both of which mean the port round-trip was not delivering a
  valid server answer. Today winsrv writes `ResponseNotHandled` (1), which *is* in the accept
  list, so the kernel passes it through unclamped.

  **This reframes the entry: the six failures are the OS getting MORE correct, and #18 is NOT a
  flip blocker.** Current behaviour is exactly what both the C and the Rust winsrv specify. The
  residual divergence is upstream ReactOS vs Windows — the test is calibrated on Windows, whose
  csrss answers `ResponseReturnToCaller` for out-of-range `ValidResponseOptions` where ROS's
  `UserServerHardError` answers `ResponseNotHandled` — and it only became observable once our
  dispatch started delivering the reply. Trigger commit still unnamed; the 2026-07-17 winsrv
  harderror commit (`b8185deb011`) is mechanical `mut`/unused-import noise and is NOT it, and
  the 2026-07-16 LPC reply-lifetime fix (`ce89f8d6af1`) predates the clean baseline. The
  remaining suspects are the 07-18 calling-convention canon passes (`e33a810c9ad`,
  `992af657162` — 360 decl fixes + 38 shim-def bugs across 40 crates), which is precisely the
  change class that would repair a server reply that previously never landed.

  **The BASELINES have already absorbed this on two arches, and amd64's is the odd one out.**
  As of 2026-07-26 the UP baselines read: i386 `pass=24 fail=6`, arm64 `pass=24 fail=6`, amd64
  `pass=30 fail=0`. The i386 and arm64 rows are freshly measured and accepted; amd64's is the
  **untouched 2026-07-17 row** — `afc4aff8277` was an additive hand-edit (`amd64 | 1 +`, adding
  only `NtCreateSection`), not a `--write-baseline` regeneration, so nothing re-measured
  ExHardError there. (Do not read the file mtimes as evidence of a re-seed; they are checkout
  timestamps. Read the diff.) Consequence: **amd64 will keep reporting a phantom ExHardError
  regression until its UP baseline is re-seeded to 24/6** — the measured amd64 value has been
  24/6 since 2026-07-24. Also note `i386 never runs this test` elsewhere in this entry is now
  stale: the native-SEH quarantine clearance (i386 76 → 89) added it.

  One instrumented boot still settles path 1 vs path 3 definitively — log `PortHandle`, the
  `Status` from `LpcRequestWaitReplyPort`, and `Message.Response` immediately after the send.
  Note the six are the ONLY out-of-range cases outside the `InteractivePart1/2` guards (every
  case that would pop a message box is gated off on ROS), so they are the only ones whose answer
  the sweep sees. Repro: `./vibeos-cli conformance --arch amd64` (no `--smp`) — 110s after the
  build. i386 never runs this test (74-test list vs amd64's 87).

- **WoW64: 32-bit guests could not create THREADS — FIXED on amd64, PARTIAL on arm64,
  2026-07-25** (task #64, wow64-plan Phase E15). wow64.dll had no per-thread entry at all:
  `Wow64LdrpInitialize` fused process-init with first-thread-init and kept the per-thread CPU
  area (the guest register file) in function-local `static`s. The kernel side was already
  correct per-thread and needed no changes. Four PRE-EXISTING bugs each independently blocked
  it, all of the "can only exist while the capability is missing" kind:
  (1) `RtlpThrImageStackReserve` read the WoW64 process's **PE32** image at native width, so
  `SizeOfStackReserve` came back as `(commit<<32)|reserve` = **16 TB** and every
  `RtlCreateUserThread` failed `STATUS_INVALID_PARAMETER_4`;
  (2) `LdrpProcessInitialized` never left 1 because `LdrpInitializeProcess` never returns in a
  WoW64 process, so every later thread spun forever in the loader's wait — a thread that
  created and resumed cleanly and then printed nothing;
  (3) `CreateRemoteThread`'s csrss notify always failed (the guest runs CSR-less by design) and
  the cleanup path tore down a perfectly good thread;
  (4) `NtWaitForMultipleObjects` was a flat thunk whose handle ARRAY is 4-byte guest / 8-byte
  host — a *cold* audit-baseline entry that went live the instant threads existed (279 → 278).
  Verified by `thread32.exe`: 9 rungs incl. TLS and guest SEH on the new thread and 4
  concurrent threads. amd64 `0x600d`, all 8 probe guests green.

- **WoW64/FEX: four CONCURRENT guest threads AV on arm64 — FIXED 2026-07-26** (task #73,
  wow64-plan Phase E16). arm64 `thread32.exe exit = 0x600d`, all nine rungs; the whole arm64
  WoW64 probe ladder is now 7/7, i.e. parity with amd64. **FEX was never at fault** — three
  independent bugs, each masking the next:
  (1) `MiInsertSharedUserPageVad`, which reserves `MM_SHARED_USER_DATA_VA` with a VAD so the
  allocator knows the page is taken, was `#[cfg(target_arch = "x86_64")]` — amd64 had it, arm64
  did not, which is exactly why arch-neutral threading code passed on amd64 and died on arm64.
  The 5th WoW64 TEB landed on 0x7FFE0000; its `StackBase`/`StackLimit` read back as the *clock*,
  and `MmCreateTeb`'s zeroing wiped KUSER_SHARED_DATA process-wide;
  (2) `Wow64pVeh`/`Wow64pPrepareForExceptionHook` dereferenced the CPU area with no NULL check,
  so the handler faulted *inside itself* and buried the real exception — that is where E15's
  bogus "libwow64fex fault PCs" came from;
  (3) `NtSetInformationThread(ThreadZeroTlsCell)` zeroed `TEB64->TlsSlots[i]` for every thread,
  but a 32-bit guest's TLS lives in TEB32 and TEB64 slot 1 is `WOW64_TLS_CPURESERVED` — so a
  guest `TlsFree(1)` wiped wow64.dll's per-thread guest register file process-wide. Needed **no**
  marshalling change: same `ULONG` at the same offset in both worlds, only the target structure
  differs, so no layout-diff tool would flag it.
  Still owed on its own merits (NOT this bug): we wire only `BTCpuNotifyMapViewOfSection` of
  FEX's memory-notification contract, though it exports `BTCpuNotifyMemoryAlloc`/`Free`/
  `Protect`/`Dirty`. **→ closed by #74 below, and it was a missing capability, not tidy-up.**

- **WoW64: every `RegSetValueEx` from a 32-bit guest BUGCHECKed the box — FIXED 2026-07-26**
  (wow64-plan Phase E20). A **local denial of service from any 32-bit app**, not a correctness
  wart. `UNICODE_STRING` is 8 bytes in the guest and 16 on the host (`Buffer` at offset 4 vs
  8), so a flat-thunked `PUNICODE_STRING` made the kernel read the string pointer from *past
  the end* of the guest's struct — a wholly unrelated address assembled from the two words
  that followed. `Length`/`MaximumLength` share offsets and arrived intact, which is what made
  it look plausible. `NtSetValueKey` was flat while `NtQueryValueKey` one line away was already
  marshalled: registry **reads worked and writes killed the machine**. Serial signature:
  `[PROBE-AV-R] ABOVE-USERPROBE Addr=0040400000580056 Len=38` → `RtlRaiseStatus(0xC0000005)`
  → `BUGCHECK 0x1E` (`Len=38` = 19 WCHARs = the exact name the guest passed, so only `Buffer`
  was wrong). Fixed across all 27 ntdll services whose only divergence is a bare
  `PUNICODE_STRING`/`OBJECT_ATTRIBUTES`. Baseline 246 → **209** (+0 added); ntdll table now
  172 auto / 124 manual. Proof obtained in the right order: the `marshal32` rung was written
  and run against **unfixed** media first (it bugchecked), then against the fix (it passes), so
  the test is known to be able to see the bug rather than merely being green.

- **WoW64: 19 syscalls wrote FOUR BYTES PAST a 32-bit guest's variable — FIXED 2026-07-26**
  (wow64-plan Phase E19). Each takes a guest pointer to a cell the kernel fills with a
  *pointer-sized* value (`HANDLE`, base address, region size). Flat-thunked, the guest's
  4-byte cell went straight to a host kernel that stores 8, so every **successful** call
  scribbled the next four bytes of guest memory — the adjacent local, or the next struct
  field. `NtOpenProcessToken` was among them, so every `OpenProcessToken()` from a 32-bit
  guest did it. Invisible because the call succeeds and returns the right value in the right
  place; only the collateral write is wrong, so damage surfaces later and elsewhere (same
  signature as the calc heap corruption and the `IO_STATUS_BLOCK` overflow). Found by mining
  the `marshalgen audit` baseline rather than by reading code — a flat thunk over a divergent
  payload *is* the absence of code. Tell that it must come from the tool: `NtOpenProcessTokenEx`
  was already marshalled while plain `NtOpenProcessToken` beside it was not. Baseline 278 →
  **246** (+0 added). Proof: `marshal32.exe`'s new out-cell rung (canary after the handle
  cell, call required to succeed, canary `volatile` so the compiler cannot fold the check
  away), green on both arches. **Still open:** the array-payload group (AWE, `NtGetWriteWatch`,
  `NtCompactKeys`, `NtRaiseHardError`) — their kernel side returns `STATUS_NOT_IMPLEMENTED`
  today, so nothing corrupts and no marshaller can be tested; and the five `SEMANTIC` cases
  (`PORT_MESSAGE`, `TOKEN_GROUPS`), which belong with the CSR-LPC-32 layer.

- **`selfmod32.exe` was RED on amd64 since E17 and the gate could not see it — FIXED
  2026-07-26** (wow64-plan Phase E19). Its rung-5 fixture writes a scratch file under
  `%WINDIR%`: a writable ramdisk on arm64, a **read-only CD** on the amd64 livecd, so it
  exited `0xBAD9`. E17 verified it on arm64 and staged it for amd64 without watching it run
  there. The durable point is about the harness: **the probe ladder runs AFTER the desktop
  marker, and the ordinary smoke stops at that marker**, so any late-rung, single-arch
  failure is invisible to `gate`. Rung 5 now skips loudly with the errno on a read-only volume
  (file32's existing precedent) and rungs 6-7 still run; arm64 keeps full rung-5 coverage. Both
  ladders now 9/9. **Mechanised the same day so it cannot recur: `vibeos-cli smoke --arch X
  --ladder`** drains serial past the desktop marker until all nine guests report and fails the
  boot on a wrong or missing verdict (`tests/qemu/cli/ladder.py`, 7 unit tests including the
  selfmod32 case). This blind spot had already been *written down* once — in the
  `_MmCopyFromCaller` entry above, "the smoke gate stays GREEN through this: the boot marker
  fires long before the ladder runs" — and still cost a day, which is the argument for a flag
  over a note.

- **WoW64/FEX: a 32-bit guest could not execute code it generated at runtime — FIXED
  2026-07-26** (task #74, wow64-plan Phase E17). wow64.dll wired one of the CPU layer's eight
  memory notifications. FEX's decoder **refuses to translate any guest address outside the
  executable set it has been told about** (`Frontend.cpp` `CheckRangeExecutable`), and that set
  is filled only by those notifications plus a one-time address-space sweep at
  `BTCpuProcessInit` — so images ran and **everything the guest allocated afterwards was
  un-executable**: ATL/MFC thunks, VB6/Delphi trampolines, packers, self-extractors, any JIT.
  Now wires `BTCpuNotifyMemoryAlloc`/`Protect`/`Free`/`Dirty`/`UnmapViewOfSection`/`ReadFile`/
  `FlushInstructionCache2`/`ProcessExecuteFlagsChange`, with the Before/After pairing the
  contract requires (`After` is a lock/unlock flag inside FEX, not a status report) and gated on
  the target being the current process. Five syscalls became manual to carry the side effects;
  `NtRead`/`NtWriteVirtualMemory` had a second, independent flat-thunk bug — the trailing
  `*ReturnSize` is a `ULONG_PTR`, so the host wrote 8 bytes into the guest's 4-byte cell.
  Verified by the new `selfmod32.exe` (7 rungs; most fail *silently wrong*, so it checks values,
  not survival), with **amd64 as the control**: `wow64cpu` exports none of these, so the same
  guest must pass there with 0/8 wired.

- **WoW64/FEX: a guest hardware fault never reached the guest's own SEH on arm64 — FIXED
  2026-07-26** (task #77, wow64-plan Phase E18). The last arm64 ladder gap: `fault32.exe` was
  amd64-only and `thread32.exe` rung 7 ran behind an explicit `noseh` skip, both now dropped.
  On arm64 a guest fault is a **host** fault inside FEX's JIT; when
  `BTCpuResetToConsistentState` cannot handle it, it hands the fault back in guest terms
  (registers reconstructed into the CPU area, `EXCEPTION_RECORD` rewritten with
  `ExceptionAddress` = guest `Eip`, host `CONTEXT` replaced with the pre-JIT-entry one) and
  returns — leaving wow64.dll to dispatch. `Wow64pRedirectGuestFault` builds the i386
  `KiUserExceptionDispatcher` frame on the guest stack (E5's builder, verbatim), points the
  guest context at it, and returns `EXCEPTION_CONTINUE_EXECUTION`, which resumes inside
  `BTCpuSimulate` and re-enters the guest at its own dispatcher. Before it, the fault had
  nowhere to go and the simulate loop re-faulted forever. Breakpoints deliberately keep the
  pre-E18 behaviour — the discriminator matches `int 2d` DebugService, not a real `int3`.
  **The arm64 WoW64 ladder is now 9/9** (hello32/win32/file32/async32/marshal32/fault32/
  thread32/selfmod32 `0x600D`, gui32 `0x600E`).

- **WoW64 probe guests are NOT staged on arm64 by default** (2026-07-25, noted while closing
  #64). `dll/win32/wow64cpu/CMakeLists.txt` (the amd64 livecd path) is entirely
  `if(ARCH STREQUAL "amd64")`; arm64 stages from `qemu/mkramdisk.sh` and only under an opt-in
  `VIBEOS_FEX=1`. So a default `./vibeos-cli build --arch arm64` carries a minimal, possibly
  weeks-stale syswow64 set and the arm64 gate exercises almost no WoW64. Use
  `VIBEOS_FEX=1 ./vibeos-cli build --arch arm64` when the WoW64 ladder matters.

- **amd64 WoW64 `g_MsgMemory` SendMessage payload over-read — FIXED 2026-07-24**
  (§9.116 diagnosis, §9.118 fix, task E13). Fixed in the **WoW64 thunk layer**, not win32k:
  §9.116 had concluded "the fix is win32k-side", but every `co_IntDoSendMessage` call sits
  inside one syscall, `NtUserMessageCall`, which is where real Windows thunks messages.
  `Wow64Manual_NtUserMessageCall` now marshals the lParam payload through the **generated**
  `wow64_cb_automarshal.c` table the callback direction already used, so the two directions
  cannot disagree about a layout. Auditing that boundary found **two more** divergences on
  the same syscall, one of them live since the first 32-bit GUI guest: `ResultInfo` under
  `FNID_SENDMESSAGE` is a `ULONG_PTR` out-slot, so win32k stored **8** bytes into user32's
  **4**-byte `LRESULT` local — a guest-stack smash on *every* SendMessage (the E7 shape on a
  much hotter path); and `DOSENDMESSAGE` (12 vs 16) under `FNID_SENDMESSAGE{FF,WTOOPTION}`.
  Verified: old signature absent from the log entirely, **spider and eventvwr now PASS**
  (both are re-enabled in the winlogon ladder as the regression test), **21 applets + 7
  probe guests green on amd64**.
- **`ProbeForWrite` raises unhandled in kernel mode → BUGCHECK 0x1E — FIXED 2026-07-25**
  (task #67, §9.133). Chain was `ProbeForWrite+0x90` → `ExRaiseAccessViolation+0x14` →
  `RtlRaiseStatus(0xC0000005)` → no handler under `USE_DUMMY_PSEH` → 0x1E. **The raise site
  was `KiInitializeUserApc` (`ke/amd64/usercall.rs`), whose C `_SEH2_TRY`/`_SEH2_EXCEPT` had
  been translated to a BARE BLOCK with the handler body dropped** — `exception_occurred` was
  declared and tested but never assigned, so the recovery arm was provably dead code. Fixed
  per dummy-pseh-verdict Tier 1 (validate-first `probe_user_buffer` + the C `__except` body
  restored); `ke/i386/usercall.rs` already carried the identical conversion, so amd64 was
  simply missed. Found by instrumenting the previously-undiagnosed ACCESS_VIOLATION arm of
  the probe: `Addr=-1280 = -sizeof(UAPC_FRAME)` identified the caller by arithmetic alone,
  and **refuted** the leading E12/wrong-width-marshal hypothesis in one boot.
  **Neither doom nor devmgmt was the culprit — `osk.exe` was**; they were downstream victims
  of the box-kill. Root of the bad pointer is the flat `NtCreateThread` thunk (§9.134, task
  #64): it returns SUCCESS while building a thread with a zeroed trap frame. Verified same
  boot command before → after: BUGCHECK 1→0, PROBE-AV 1→0, doom frames died→2022.
  Still-relevant sibling is task #68 (win32k's
  `_MmCopyFromCaller`/`_MmCopyToCaller` copy user buffers unguarded — `SafeProbeFor*` is
  validate-only and does not touch-test, so any user process, 32- or 64-bit, can kill the
  box with a partially-unmapped buffer; `KmProtectedCopyMemory` is
  the available fix and it is **no longer amd64-only** — that parenthetical was stale on BOTH
  counts: arm64 has shipped it since 2026-07-17 (the §9.93 `save_next` fix cleared the §9.69
  wedge; `ntoskrnl-cargo/build.rs` pushes `kmsafecopy.c` for aarch64), and i386 got it
  2026-07-24 as an exception-table fault-fixup primitive (§9.123). **FIXED 2026-07-25:** both
  `_MmCopy{From,To}Caller` in `win32k-rust/src/copy.rs` now route through it on all three
  arches, and it is exported from `ntoskrnl.spec` because win32k.sys is a separate PE that
  cannot carry its own i386 copy (the extable keys on a link-time PC in the kernel's
  `KiTrap0EHandler`). The probe deliberately stays validate-only.)
  devmgmt/msconfig/sndrec32/kbswitch stay staged with their winlogon launch held so the 21
  passing rungs keep reporting. Note the smoke gate stays GREEN through this: the boot
  marker fires long before the ladder runs.
- **All 1962 Rust-kernel `debug_assert!` are compiled OUT, while the C `ASSERT` they were
  translated from is LIVE — OPEN** (2026-07-25, task #72, §9.135). `debug_assert!` is gated on
  `cfg(debug_assertions)`; `sdk/lib/rust/Cargo.toml`'s `[profile.release]` never sets it, so
  cargo's release default (off) applies and every kernel crate builds `--release`. Meanwhile
  `CMakeLists.txt:233` sets `-DDBG=1`, so the C original calls `RtlAssert`. **The mirror of
  §9.120** — there asserts were lowered to something *stronger* than C (UB / an invented
  BUGCHECK), here to something *weaker* (nothing). Proof by execution, not inference:
  `ke/amd64/usercall.rs:96` asserts `(SegCs & MODE_MASK) != 0` and a boot printed
  `[KI-APC] ... SegCs=0x0` — a forbidden value walked straight past it, and had it fired it
  would have caught #67/#71 at the source rather than as a bugcheck in another function.
  Fix is `debug-assertions = true` **plus an explicit `overflow-checks = false`** (it defaults
  to follow debug-assertions, and kernel-wide overflow panics are a separate, riskier change C
  never had) — but MEASURE FIRST: 1962 assertions going live at once needs a triage batch.

  **MEASURED 2026-07-28, and "measure first" was right for a reason nobody had named.**
  The count that matters is not the assertions. `debug_assert!(x)` expands to
  `if cfg!(debug_assertions) { assert!(x) }` — the `cfg!` **macro** — so its expression is
  type-checked on every build already, and turning the flag on cannot break it. The risk is
  entirely in the **attribute** form: enabling the flag compiles, for the first time in the
  life of this tree, **108 `#[cfg(debug_assertions)]` blocks** — and deletes **14
  `#[cfg(not(debug_assertions))]` blocks**, which is how a `not()`-only stub definition turns
  into an undefined symbol. Those 108 regions have never had a type check, a borrow check, a
  lint, or an execution. Three were broken on the first attempt:
  - `freeldr-rust/src/heap.rs` — `FrLdrHeapCleanupAll`'s statistics block deref'd the `PVOID`
    heap handle with no `as PHEAP`: **14 x E0609**. FIXED.
  - `ex-rust/src/init.rs` — `KeServiceDescriptorTable[0] = core::mem::zeroed()` (and the same
    for the shadow table), **invented by the translation**: `ntoskrnl/ex/init.c:1415` assigns
    only `.Count`, and `ke/krnlinit.c:104-108` fills `Base`/`Limit`/`Number` earlier. It
    compiles clean and would have wiped **the syscall dispatch table** at Phase 0, every
    system call after that dispatching through a zeroed descriptor. FIXED. This is the one to
    remember: not a compile error — a boot-fatal runtime bug sitting in the kernel's
    most-read init file, invisible because the compiler had never been asked to look.
  - `ke/{amd64,i386}/kiinit.rs` — called `KiReportCpuFeatures`, which is **not translated**
    (`ke/*/cpu.rs` carries only its signature as a comment) and whose C is cohab-dead, so the
    symbol exists in no artifact: `LINK_FAIL` on both x86 arches. Call sites disabled with a
    note; translating it is owed.

  **And there is a SECOND half, which is why this is still not enabled.** On pinned stable,
  `debug-assertions` does not only enable `debug_assert!` — it also enables rustc's **UB
  checks**, which share the same cfg (splitting them needs nightly `-Zub-checks`). Scoped to
  four kernel-core crates and booted, they found real **alignment UB in 2 of the 2 crates
  exercised, one per arch, on the first boot of each**:
  - amd64 — `hal-rust/src/x86/halacpi.rs:864`, an 8-byte read of an ACPI structure that is
    only 4-byte aligned (`misaligned pointer dereference: address must be a multiple of 0x8
    but is 0xfffff8000010232c`). **FIXED** with `addr_of!` + `read_unaligned`, which is the
    correct way to read a field out of a packed firmware structure, and is right regardless
    of whether the checks are on.
  - arm64 — `ntoskrnl-rust/src/ke/time.rs:56`, `write_volatile` of a `u64` through a
    `*mut KSYSTEM_TIME`. `KSYSTEM_TIME` is `{ULONG, LONG, LONG}` — 4-aligned, and
    `SystemTime` sits at offset **0x14** of `KUSER_SHARED_DATA`, so the pointer genuinely is
    not 8-aligned. **NOT fixed, deliberately:** the misalignment also means that store was
    never atomic, so the torn-read guard the surrounding comment documents (`High1Time` /
    `LowPart` / `High2Time` ordering) does not actually hold on 64-bit. Repairing it means
    reworking the KSYSTEM_TIME write protocol against the C, not silencing a checker.

  **So: staged, commented-out, NOT enabled.** Two of two crates tried had a UB site, on top of
  the 108 never-compiled blocks — that is a migration, and it must go one crate at a time with
  a boot per crate, not a flag flip. **Owed:** `ke/time.rs`'s KSYSTEM_TIME protocol; the other
  79 blocks (se-rust 11, win32k dclife 8, cc-rust view 7, fastfat 12, cdrom 9, config-rust 6,
  classpnp 6, the freeldr tail, …); translating `KiReportCpuFeatures`.

  Two behaviour changes ride along with the four armed crates and are expected, not bugs:
  `ex-rust`'s `NtBuildNumber` gains the `0xC0000000` checked-build bits (which makes it AGREE
  with the `PRCB_BUILD_DEBUG` the kernel already published unconditionally on all three
  arches), and `ke/spinlock.rs` starts stamping the owning thread into the lock word under
  DBG. Watch the conformance ratchet for kmtest tests that branch on checked-build.
- **`NtCreateThread` flat thunk returns SUCCESS with a zeroed thread context — REFUSED LOUDLY
  2026-07-25** (task #64, §9.134). Five divergent params incl. a `CONTEXT` whose field lists
  don't correspond across widths. Now returns `STATUS_NOT_IMPLEMENTED` instead of a silently
  corrupt thread; marshalling the structs alone would be *worse* (a more convincing corpse,
  still with no CPU-context setup — that needs the per-thread WoW64 entry #64 tracks). Blast
  radius measured: exactly one call in a full amd64 boot, from `osk.exe`.
- **`NtUserInitializeClientPfnArrays` would poison the GLOBAL win32k callback table —
  INTERCEPTED 2026-07-25** (task #63/E12). The one live divergence whose payload lands in
  *global* kernel state, not the guest: win32k copies the caller's `PFNCLIENT` arrays (function
  pointers, 4 bytes guest / 8 host) into `gpsi->apfnClient*`, shared by every process, so a
  32-bit guest reaching that body is read at host width and installs garbage callbacks for
  **native 64-bit processes too**. Never fired only because win32k early-outs on
  `ClientPfnInit != 0` and native user32 always initializes first — ordering, not a guarantee.
  Now returns SUCCESS from the thunk layer without calling the kernel (identical observable
  behaviour to the early-out, minus the dependency). 4 calls/boot, one per WoW64 GUI process.
- **OUT `PHANDLE` flat thunks: `NtOpenProcessTokenEx`/`NtOpenThreadTokenEx` — MARSHALLED
  2026-07-25** (task #63/E12). 12 calls per amd64 boot, failing two ways at once: the kernel's
  `probe_write_handle` is `probe_write_user::<PVOID>` (8-byte alignment) while a guest `HANDLE`
  is 4-aligned, so a cell at ≡4 (mod 8) was rejected with a spurious
  `STATUS_DATATYPE_MISALIGNMENT` and one at ≡0 passed and wrote 8 bytes over a 4-byte cell.
  Verified in a boot log: `NtOpenThreadTokenEx` used to appear twice per logical open (first
  `=> 80000002`), now once, `[manual]`, 1:1 with `NtOpenProcessTokenEx => 00000000`.
  **Triage lesson worth more than the fix:** intersecting the audit baseline with the syscalls
  a real boot dispatches collapses the queue — of 195 divergent syscalls only **4** are called
  in a full boot, and of 41 out-scalar-widen findings only **2**. Burning the baseline down in
  file order would have spent most of the effort on cold entries. The remaining 39 in this
  class are real but unreached; they matter for arbitrary apps, as a tail not a queue.
- **`NtGetContextThread`/`NtSetContextThread` flat thunks overflow guest memory / load the
  native register file from i386 fields — MARSHALLED 2026-07-25** (task #63/E12). Same
  un-widenable `_CONTEXT` as `NtCreateThread`, but the disposition is the opposite one: the
  kernel could never answer these (it has no notion of guest registers), while the CPU layer
  already tracks them and `BTCpuGet/SetContext` speak exactly this i386 `CONTEXT` — so they got
  real marshallers, not a refusal. Get was writing a 1232-byte host `CONTEXT` into the guest's
  716-byte buffer. Self-thread only (cross-thread refused loudly — it would confidently return
  the *wrong* thread's registers); control/integer/segments only, with unsupported
  `ContextFlags` bits cleared in the reply rather than left looking answered.
  Verified by two new `fault32.exe` rungs — a canary behind the CONTEXT (the standing
  negative control for the overrun) and a Get→Set Eip redirect, with `0x600D` moved
  into the landing site so the exit code *only* means the redirect worked.
  **Owed: arm64 has ZERO functional coverage of this.** The probe guests are staged
  by `dll/win32/wow64cpu/CMakeLists.txt`, which is `if(ARCH STREQUAL "amd64")`, and
  fault32's launcher is `#[cfg(target_arch = "x86_64")]` because its rung 1 (hardware
  fault → guest SEH) is still owed on arm64. Rungs 4-5 are arch-neutral but trapped
  behind rung 1 in one binary, so closing this means a separate ctx32 guest **plus**
  an arm64 staging path. It matters because arm64's CPU layer is FEX, not
  `wow64cpu.S`: exactly the `#ifndef _M_ARM64` branches that go untested.
- **arm64 warm build can end with a STALE cohab DLL — OPEN, not reproduced** (2026-07-25,
  task #70). A gate's arm64 build shipped `kernel32.dll` linked 331ms *before* its own
  `libkernel32_rust.a`; `artifact-check` caught it and a plain warm rebuild fixed it with no
  source change, so it was real staleness, not a flaky checker. Two hypotheses eliminated
  (sccache stop-on-entry — already replaced by pinned per-worktree ports; and port collision
  — all 11 worktrees hash distinct). Condition present at failure and absent from the repro:
  concurrent sibling-lane builds. The ninja graph is correct, so it is an ordering violation,
  not a missing edge.
- **C assertion macros lowered to UB / to a BUGCHECK — FIXED 2026-07-25** (§9.120).
  Found by a fresh-eyes catalog sweep, not by a crash. `PAGED_CODE()`/`NT_ASSERT()` are
  DIAGNOSTICS (KdPrint+RtlAssert under DBG, **nothing** in retail) and are never
  `__assume(0)`; three translations had lowered them to `unreachable_unchecked()` (UB —
  cdrom, fastfat) or to an unconditional retail `KeBugCheckEx` with an **invented** stop
  code (classpnp, `0x35u32` = NO_MORE_IRP_STACK_LOCATIONS, unrelated to IRQL). All latent,
  none had fired. **The §9.113 gate reported a clean 0 the whole time**: its regex required
  the digits to be followed directly by `,`/`)`, so the Rust type suffix in `0x35u32` hid
  the site from 2026-06-09 through the entire 07-24 sweep — *a lint that under-matches is
  indistinguishable from one that passes.* Regex fixed; new gate
  `tools/scan_assert_to_ub.py` (empty baseline) keeps the class at zero.
- **arm64 0x1E / `[WAITDEP]` ALL-ONES residual — CLOSED 2026-07-17** (§9.92 item 4). The
  "~2 hits / 2 days" residual was the SAME netstack-TDI wait-block bug as §9.36 on the
  *synchronous*-completion path the 07-08 fix didn't cover (`IofCompleteRequestInner`'s
  deferred special kernel APC landing on the freed block during the next self-test's IRP
  build). Fixed by leaking the wait-block on the sync path too; arm64 conformance boot went
  `BUGCHECK 0x109` → clean `sweep=done`. `ke/waitdep.rs` stays always-on — a fresh
  `[WAITDEP]` 0x109 is NEW evidence, triaged from scratch (per §9.54: symbolicate the FIRST
  banner in a log, never the last). Deeper owed root, still open: arm64 does not promptly
  deliver a self-targeted special kernel APC queued at PASSIVE_LEVEL (real NT delivers it
  before `IofCallDriver` returns — what makes the classic stack-event sync-IRP idiom safe).
- **winlogon 0xC000021A→0x4C init race — PREMISE REFUTED + NOT REPRODUCING** (2026-07-14,
  §9.82). The "owed publish-last ordering fix" is already in the tree at three nested levels
  (csrsrv step-ordering, CsrLoadServerDll, basesrv SharedSection-last), and the tripwire
  evidence (identical kernel VA across hits) never fit a race. 20/20 arm64 boots clean with
  the detector proven visible; zero hits in 1254 retained run dirs. Re-open ONLY on a fresh
  `[kernel32] BASE_STATIC_SERVER_DATA CORRUPT` print, starting from its printed fields.
- **win32k `DialogBoxParamW` dialog-creation race — NOT REPRODUCING, tripwire armed**
  (2026-07-17, §9.12 addendum). 0 hits / 20 boots across three regimes (contended-parallel
  TCG / sequential TCG / native KVM) with the path proven exercised in all 20. The old
  mitigation had NO working detector (its `ERR!` was compiled dark behind Wine debug
  channels) — replaced by a permanent unconditional pair in msgina `gui.rs`:
  `[MSGINA] status dialog up` each boot + `[MSGINA] 9.12-TRIPWIRE … worker tid=N` on a hit.
  A hit leaves the worker wedged in win32k with the tid printed — dump its kernel stack
  (`inspect.sh dump-thread`) for the exact blocking site. Re-open only on a tripwire line.
- **`PsLocateSystemDll` failed on EVERY arm64 boot, invisibly — FIXED 2026-07-26.** Found by
  arming the arm64 silent-suppression family (below) and booting once. The serial said
  `[PS] PsLocateSystemDll FAILED subcode=8 status=0xC000007A` on a **green** boot.
  Subcode 8 is `PspLookupKernelUserEntryPoints`; `0xC000007A` is `STATUS_PROCEDURE_NOT_FOUND`.
  Cause: it looked up `ExpInterlockedPopEntrySList{End,Fault,Resume}`, which `ntdll.spec`
  declares **`-arch=i386` and `-arch=x86_64` only** — there is no arm64 row, and the built
  arm64 `ntdll.dll` exports none of the three (0 occurrences, vs 3 each for the four
  `Ki*Dispatcher`s looked up just above). They exist to fix up a fault inside the *x86*
  user-mode SLIST pop; the sole consumer in the tree is `ke/i386/traphdlr.rs`, which already
  NULL-guards. Fixed by gating the three lookups to the arches that export them.
  **Why it went unseen for so long is the real lesson:** the failure is reported through
  `iomgr`'s `io_trace`, which was `#[inline(always)] fn io_trace(_s: &[u8]) {}` on aarch64 —
  a no-op, so all 40 of its boot-milestone and FAILED reports were discarded on the only arch
  that needed them, and LLVM dropped the strings from the binary entirely
  (`"[IO] IopInitializeBootDrivers"`: 0 occurrences before, 2 after). `IoInitSystem` then
  *continued booting* ("continuing on ARM64"), so this escape hatch was **permanently**
  exercised and could never have signalled a genuine system-DLL failure. `io_trace` now routes
  to `ARM64_UART_TRACE`; 26 `[IO]` lines reach the serial where every prior soak log has zero.
  Verified: the `FAILED` line is gone and `[IO] IoInitSystem returning TRUE` appears instead.
  **Impact on the four faces is NOT established** — the NULL pointers were harmless on arm64,
  so this is a spurious failure made real-looking, not (yet) a known cause of B/C/D. What it
  does buy: the hatch means something again, and 26 boot milestones are now visible.
- **`ex/init.c`'s whole 54-call `[EXP]`/`[PHASE1]` trace ladder was dropped in translation —
  RESTORED 2026-07-26.** Ran the `io_trace` lesson tree-wide as a count-vs-count audit of every
  cohab-flipped pair, and `ex/init.c` → `ex-rust/src/init.rs` came back **54 → 0**: not one of
  its boot milestones survived, in the single file that owns Phase 1. That is why every arm64
  soak log looks featureless — a hang anywhere in Phase 1 produced a bare `HANG_TIMEOUT` with no
  way to name the step it died on. 40 sites restored via a `uart_trace!` **macro** (never a
  no-op fn — see the field manual entry). Measured on a green 54s boot: **6 `[EXP]` + 26
  `[PHASE1]`**, a complete ladder from `ExpInitializeExecutive entered` through
  `IoInitSystem succeeded!` to `ExpLoadInitialProcess (SMSS)` / `[SMSS] Status=0x0`.
  Three of the restored sites are the silent `#[cfg(not(target_arch = "aarch64"))]` bugcheck
  suppressions (`IoInitSystem`, `PsInitSystem`, `SeRmInitPhase1` — same tolerate-and-continue
  shape as `PsLocateSystemDll` above), and a fourth is worse: an arm64-only
  `if IoInitSucceeded == 0 { ExpInitializationPhase += 1; free; return; }` that **abandons Phase 1
  outright** — no smss, no desktop, and not one byte on the serial. That path presented as an
  indistinguishable hang; it now announces itself.
  The audit's other two shortfalls were judged and deliberately **left dropped**:
  `mm/section.c` (32→0) and `ARM3/procsup.c` (33→10) lost per-page / per-process hot-path
  forensics that would drown the serial. The one exception restored from section.c is
  `MiGrabDataSection`'s rate-limited `INCREATE/INDELETE` spin probe — it fired **0 times** on the
  green boot, so it costs nothing and only speaks when that loop is wedged.
  **Noticed, not fixed (pre-existing, not this lane's change):** the arm64 serial carries
  **1,116 `[PS]` lines / 243 KB per boot**, ~2,250 of them per-thread `PspCreateThread` /
  `PspUserThreadStartup` traces that fire on *every* thread creation. Whether that volume
  perturbs TCG boot timing is untested — flagging it as a candidate, not a claim.
- **arm64 kmtest-sweep late-run user-process corruption** (~2/4 sweeps since the §9.96
  un-quarantine of MmSection/ZwMapViewOfSection, 2026-07-17; deeply characterized 2026-07-18,
  §9.97) — a *later* kmtest_.exe launch (77+ deep) dies with `0xC000001D` (executing garbage
  bytes in its still-mapped image `.text` — `Instr=0x00B000B0` vs a totally different on-disk
  word) or `0xC0000005` at `ntdll RtlpCoalesceFreeBlocks/RemoveEntryList` during process-init
  heap-extend (page-aligned FAR); manifests as `IoReadWrite`/`NtSystemDebugControl`/
  `IoDeviceObject` NO-END/FAIL-EXIT ratchet noise. **PROVEN: a physical-page-reuse bug** — a
  frame backing a still-mapped user page is freed/re-handed-out (image face = remapped-and-
  overwritten; heap face = unmapped → not-present). Disproven: rtl_heap translation bug (heap
  is faithful/victim), and the §9.96 section path (that fix works correctly). Same PFN family
  as `ZwAllocateVirtualMemory` 0x4E + `MmMapLockedPagesSpecifyCache` (task #7) — and as of
  2026-07-25 that is no longer a guess, it is **the same check** (§9.124). Task #7's recorded
  "BUGCHECK 0x1000 (PFN_LIST_CORRUPT-class, P1=0x401) in the Rust MDL-map path" was poisoned by
  §9.113: `0x1000` was an INVENTED stop code appearing at exactly four sites in the whole pre-fix
  kernel, all in `ARM3/virtual.rs`, subcodes `0x61940/0x61940/0x400/0x401`. So `P1=0x401`
  uniquely identifies `MiDeletePte`'s PFN back-pointer check — a different file and subsystem
  from the note. That same check is `#ifdef`'d OUT on arm64 and replaced by a silent
  `Pfn1->PteAddress = PointerPte` repair, added during Milestone-11 bring-up with the comment
  "pages are reused across session/process boundaries and the PFN database doesn't track the new
  PTE correctly" — a verbatim description of this bug. **arm64 has been suppressing the one
  detector that fires AT the reuse.** 2026-07-25: the repair is kept (removing it turns a boot
  into a bugcheck) but is now LOUD — a `[PFNMISMATCH]` line prints the PTE, the stale
  back-pointer, the PTE value and the PFN index at the moment of divergence. Silence is also a
  result: it eliminates MiDeletePte as the reuse path. Owed next step: a "free-while-mapped" tripwire at the RosMM page-free site
  (assert `MmGetRmapListHeadPage(pfn)==NULL`) to catch the premature free. Section tests stay
  ACTIVE (the canary). Field manual §9.97 (method + full evidence); task #19.

  **Tripwire site pinned down (2026-07-26), with the ordering trap that would have made it
  always-NULL.** The free site is `mm/freelist.rs:391 MmDereferencePage` — the
  `if rc - 1 == 0` branch, which ends in `MiInsertPageInFreeList(pfn)`. The check must go at the
  **TOP of that branch**: `MmGetRmapListHeadPage` returns NULL unless `awe_allocation()` is set
  (`freelist.rs:333`), and the branch clears `awe_allocation` two lines before the free — so a
  tripwire placed anywhere after that reads NULL on every page and silently proves nothing.
  Make it a LOUD PRINT, not an assert: `MmAllocPage` (`freelist.rs:428`) defensively clears
  `rmap_list_head` on *allocation*, which is itself evidence that stale non-NULL heads survive
  frees today, so a fatal assert would fire on benign leftovers as well as real
  free-while-mapped. Same call as the `[PFNMISMATCH]` precedent — made loud, deliberately not
  fatal.

  **It is not one suppressed detector — it is TWELVE, and eleven were still silent
  (2026-07-26).** The `[PFNMISMATCH]` note above treats `MiDeletePte`'s back-pointer check as
  *the* detector arm64 turned off. Enumerating the MM page-lifecycle paths mechanically says
  otherwise: **12 sites** hard-fail on x86 with `KeBugCheck(MEMORY_MANAGEMENT)` and have an
  arm64 arm that recovers silently instead (7 of them a bare `return STATUS_UNSUCCESSFUL`) —
  `mm/rmap.rs` ×3, `mm/pagefile.rs` ×6, `mm/balance.rs`, `ARM3/virtual.rs` ×2. So arm64 is not
  looking in twelve of the places x86 looks, all inside the subsystem this bug lives in, which
  is the structural reason the corruption only ever surfaces as a distant victim.

  All eleven remaining are now armed with `mm_guard_trip!` (`lib.rs`) — bounded prints, every
  recovery kept, `[MMGUARD] <site> hit#=N` plus operands. **Silence from an armed site is a
  result**: it eliminates that site.

  **QUALIFIED 2026-10-03 — only on arm64. `mm_guard_trip!` is `#[cfg(target_arch =
  "aarch64")]` and expands to NOTHING on i386/amd64**, because its printer
  `ARM64_UART_TRACE_HEX` is an arm64 header macro. Verified against the binaries, not the
  source: `strings build-<arch>/ntoskrnl/ntoskrnl.exe | grep -c MMGUARD` gives **arm64 43,
  amd64 0, i386 0**. So all 14 armed sites are blind on x86 and their silence there is
  uninterpretable — the same "probe whose silence has a hidden precondition" trap this entry
  already names, arriving from the direction nobody checked. Any reasoning of the form "x86
  shows nothing at these sites" is vacuous; I was one step from resting a kernel change on it.

  **Owed, with its risk stated so the deferral is not silent:** make the guard portable, then
  decide fatality. The obvious printer is `ntkrnl_sys::driver_ddk::DbgPrint` (used portably in
  `mm/section.rs`), but the arm64 path is a raw UART write that is safe at any IRQL and in
  early boot, and several of the 14 sites sit in PFN-lock/high-IRQL context — so a naive swap
  risks a deadlock in the subsystem these guards exist to watch. Needs a per-site context audit
  plus a 3-arch smoke, which is why it is not bundled with the #19 closure.

  **RESOLVED on every arch (2026-10-03/04).** `mm_guard_trip!` went portable (11 sites,
  `DbgPrint` on x86) on 10-03; the three spinlock-held sites use `mm_guard_trip_atomic!`, which
  now writes through `raw_trace_hex` -- the arm64 UART tracer, or on x86 a polled COM1 write
  that takes no lock and is safe at any IRQL. Verified on the binaries: `MMGUARD` strings in
  `ntkrnlmp.exe` are now **amd64 42, i386 42** (from 0 on 10-03). All 14 sites report on all
  three arches, so x86 silence at them now means something.

  Related and also owed: `MmInsertRmap`'s two early returns (`mm/rmap.rs:453`, `:476`) still
  `return` where the C `KeBugCheck(MEMORY_MANAGEMENT)`s (`ntoskrnl/mm/rmap.c:363`, `:380`), and
  the returns are **not** arch-gated even though their prints are. Restoring the bugcheck is
  correct on the contract and the arm64 evidence says they never fire (639 rounds + a
  probe-verified sweep, 0 hits) — but there is no x86 evidence at all, per the paragraph above,
  so instrument first and escalate second. The one to read first is
  **`rmap.MmDeleteRmap mapping-not-in-list`**: it means `MmDeleteRmap` walked the entire rmap
  list and did not find the `(Process, Address)` mapping it was told to delete, i.e. the
  caller believes the page is mapped there and the PFN database disagrees — *the* precondition
  for freeing a still-mapped frame, arrived at from the rmap side instead of the PFN side.

  **Also settled, and it redirects face C:** `ex-rust/src/pushlock.rs` was audited line-for-line
  against the original C (the file embeds it as comments) and is a **faithful translation** —
  `ExfWakePushLock`, `ExfReleasePushLock`, `ExfReleasePushLockExclusive` and the exclusive
  acquire all match, including an inherited upstream livelock (a failed CAS never refreshes
  `OldValue`) that does not fit face C anyway, since face C's thread is *parked*, not spinning.
  Two apparent bugs were checked and are not bugs: `debug_assert!(!x != 0)` is a bitwise-not
  that makes the assert always true (real, cosmetic, release-inert), and the spin loop's
  `EX_PUSH_LOCK_WAITING` where the C says `EX_PUSH_LOCK_FLAGS_WAIT` — **different names, both
  equal 2**. With the algorithm exonerated, HOLDER-STUCK points at a wait block made
  unreachable by memory corruption, which is where face A already pointed. Do not re-audit the
  pushlock; go to the `[MMGUARD]` output.

  **BOTH PROBES ARE NOW IMPLEMENTED AND IN THE SHIPPED KERNEL (2026-07-26).**
  - `[FREEMAPPED]` — `mm/freelist.rs` `MmDereferencePage`, at the TOP of the `rc - 1 == 0`
    branch exactly as pinned above, bounded at 32 prints plus a running total so it cannot
    bury the serial log it exists to inform.
  - `[KSTACK]` — `ke/arm64/stubs.rs`, inside the existing `[KEXC]` kernel-mode-unhandled block,
    so it costs nothing until the boot is already dying. Prints `SP`, `StackBase`, `StackLimit`,
    `InitialStack`, `FaultVA` and `Arm64DumpPte(FaultVA)` — the three-way discriminator below.

  Both were confirmed present with `strings ntoskrnl.exe | grep -c` **before** any boot
  (field manual §9.87). That check earned itself immediately: the first `[KSTACK]` attempt did
  not compile at all (`E0659` — `stubs.rs` glob-imports both `crate::ke::ke_defs` and
  `ntkrnl_sys::mm`, and both export a `KeGetCurrentThread`; take ke_defs', which returns
  PKTHREAD so the stack fields are reachable), and a soak would otherwise have run for hours
  against a kernel with no probe in it. **And then one did anyway** — the check was pointed at
  the built module while the soak booted a frozen `.iso` cut four hours earlier (see the
  retraction below). Checking the binary is necessary and not sufficient; check the media.

  > **RETRACTION (2026-07-26): the soak below was NOT instrumented — the probes were never in
  > the media.** The image was frozen at 22:42 for run-to-run comparability; `[FREEMAPPED]` and
  > `[KSTACK]` landed at 02:43, four hours later. `strings livecd-smp.iso | grep -c FREEMAPPED`
  > = **0**, same for `KSTACK`. The §9.87 check was run against
  > `build-arm64/ntoskrnl/ntoskrnl.exe` — the wrong artifact. What SURVIVES: the face taxonomy,
  > the frequencies, and the `0x50` operand reading, all of which come from bugcheck operands and
  > the census, and both of those *were* in that media. What is VACUOUS: every claim below
  > phrased as "under the probes", and any reading of the tripwires' silence — nothing was
  > listening. Re-run on freshly-cut media before treating a probe's silence as a result.

  **THE HEADLINE FROM 9 SOAK ROUNDS (2026-07-26): this is NOT one bug.** Two soaks
  on fixed media, `-smp 1`, same binary within each run:

  | face | n | signature |
  |---|---|---|
  | clean / ratchet-noise | 6 | `sweep=done`, 85–86 ok of 89, ~75–130s |
  | **A — memory/stack** | 1 | `BUGCHECK 0x50`, died 69/89 on `ZwWaitForMultipleObjects`, 77s |
  | **B — sweep no-launch** | 1 | desktop reached, `launched=0`, 87min, **no** `CENSUS STUCK` |
  | **C — holder-stuck** | 1 | `CENSUS … verdict=HOLDER-STUCK(no signal ever)`, 79/89, 12 bad, 78min |
  | **D — invisible hang** | 1 | 77/89, 10 bad, 78min, repeated 180s test timeouts, **census sees NOTHING** |

  ~33% of rounds fail, spread over **three mechanisms that are demonstrably different** (B vs C
  disproven below; A is a bugcheck while B and C never crash at all). **This reframes the flip
  blocker:** "fix the arm64 flake" is not one fix, and a conformance baseline cannot be seeded
  until each face is either fixed or the sweep is made robust to it. It also explains why the
  historical notes read inconsistently — `ZwWaitForMultipleObjects NO-END`, `0xC0000005`,
  `0x1E`, `0x50`, hangs — they were sightings of different bugs filed under one heading.
  Each face's evidence is below; the per-round data is in the soak ledgers, and the two
  reproducing serials are kept out of `/tmp` rotation.

  **FIRST REPRODUCTION (2026-07-26) — no probe fired (see the retraction: none were in the
  media), but the OPERANDS alone point hard at the STACK.**
  A 4-round `-smp 1` kmtest soak on fixed media reproduced once (rounds 1/3/4 clean or
  ratchet-noise; ~25%, in line with the prior base rate). Round 2 died at test **69 of 89, on
  `ZwWaitForMultipleObjects`** — immediately after the `ZwCreateSection` / `ZwMapViewOfSection`
  canaries, and the same test the earlier `-smp 2` run C failed with `0xC0000005`:

      *** BUGCHECK: 0x00000050 (0xffff8000050f4030, 0x2, 0xffff8000050f3e20, 0x0)

  Read the operands against the raise site (`ARM3/pagfault.rs`, the `P4=0` arm, which fires
  when `TempPte.soft_protection() == MM_ZERO_ACCESS`): **P3 is `TrapInformation`, a KTRAP_FRAME
  pointer — not a PC.** A trap frame lives ON the kernel stack, and the faulting address P1 is
  only **0x210 above it**, across the `...f4000` page boundary. Stacks grow down, so that is a
  reference to *already-used* stack — the interrupted frame's own data — whose page is no longer
  present, while the page holding the trap frame (`...f3000`) still is. A kernel stack is
  contiguous and fully mapped, so **a live kernel stack lost a backing page.** The immediately
  preceding serial is `PspUserThreadStartup: returning` for a `kmtest_.exe` thread.
  That is the same shape as the `0x1E` face (a store into the trap frame faulting on a
  kernel-stack VA), which makes stack-page loss the unifying explanation for both, and it is a
  near relative of the already-fixed §9.94 reaper bug (a stack freed without the SwapBusy spin).
  Fact vs inference: the operands and the raise arm are measured; "a live stack lost a page" is
  the reading of them, and the `[KSTACK]` dump added below is what turns it into a fact.

  **FACE A REPRODUCED ON PROBE-BEARING MEDIA (2026-07-29) — and `[KSTACK]` REFUTES the
  "live stack lost a page" reading.** Round 4 of the instrumented soak, ~15 minutes in,
  `-smp 1`, died at test **71** (the earlier reproduction died at 69 — same neighbourhood):

      [MMGUARD] rmap.MmDeleteRmap mapping-not-in-list hit#=0x1
      [MMGUARD]   Page=0x50  Process=0xFFFFFA000639AC20
      [MMGUARD]   Address=0x50000  listhead=0x0
      [PS] PspExitThread entered
      [KSTACK] SP=0xFFFF8000048C8FA0        TrapFrame=0xFFFF8000048C8E40
      [KSTACK] StackBase=0xFFFF8000048EF000 StackLimit=0xFFFF8000048DD000
      [KSTACK] InitialStack=0xFFFF8000048EEE00
      [KSTACK] FaultVA=0xFFFF8000048C9050
      *** BUGCHECK: 0x50 (0xffff8000048c9050, 0x2, 0xffff8000048c8e40, 0x0)

  **Apply this entry's own sanity check first:** `StackLimit < InitialStack < StackBase`
  holds, and `StackBase - StackLimit = 0x12000`, the forced large stack. So the bounds are
  self-consistent and this is **NOT** the mis-mapped-structure failure mode — the `KTHREAD` is
  being read correctly, and the numbers can be trusted.

  **Given that, the finding is that `SP` is 0x14060 (~80 KB) BELOW `StackLimit`.** `TrapFrame`
  and `FaultVA` are down there too, and `FaultVA` is again exactly **0x210 above the trap
  frame**, matching the earlier reproduction. The thread is not executing on the stack its own
  `KTHREAD` describes.
  - That **refutes "a live kernel stack lost a backing page"**: the stack it names is intact
    and correctly described; execution is happening ~80 KB outside it.
  - It is also not a plain overflow — running off the bottom faults at the guard page
    immediately below `StackLimit`, not 80 KB further down.
  - The reading it *does* fit: the thread is running on a **stale kernel stack** — a
    previously-allocated, since-freed one (kernel stacks come from adjacent system PTEs, so a
    prior stack plausibly lived at `0x…48C8000`). That puts it back in the §9.94 family (a
    stack freed while still in use) rather than the paging family, and the two lines
    immediately before it — a first-ever `MmDeleteRmap mapping-not-in-list` with a **NULL
    listhead**, then `PspExitThread` — are both on the thread-teardown path.

  **Fact vs inference, kept separate:** the register/bounds values, their mutual consistency,
  the 0x210 offset, and the MMGUARD hit are *measured*. "Running on a stale stack" is the
  reading of them, and "the MMGUARD rmap hit is causally upstream of the 0x50" is **not
  established** — it fired 68 serial lines earlier, with `PspExitThread` in between, and one
  co-occurrence is not a link. **Next:** dump `Arm64DumpPte(FaultVA)` (already in `[KSTACK]`'s
  repertoire) and, more decisively, record the thread's stack at *switch* time — if SP belongs
  to a stack that `MmDeleteKernelStack` already released, the reaper/SwapBusy path is the
  target, not paging.

  > ## CLOSED (2026-08-01): stale FreeLDR identity PTEs in recycled user page tables
  >
  > **Root cause.** FreeLDR's identity-mapped page-table *pages* get recycled as user-space page
  > tables while still holding FreeLDR's own entries (VA==PA, global, EL1-only), and on ARM64
  > TTBR0 **is** the user half. A freshly reserved user range therefore sits on valid-looking
  > descriptors, so (a) the fault that should demand-page it never fires, and (b) teardown
  > decodes the stale descriptor into a PFN that names no frame — VA `0x50000` → pte `0x50703`
  > → PFN `0x50`, far below QEMU virt's `0x40000000` DRAM base. `MmDeleteRmap` then reports
  > mapping-not-in-list, and `MmReleasePageMemoryConsumer(MC_USER, 0x50)` hands the allocator a
  > frame that was **never allocated** — the deterministic corruption behind all four stop codes.
  >
  > Why arm64 only: amd64 carries the equivalent guard in `ARM3/sysldr.rs`
  > (`#[cfg(target_arch = "x86_64")]`), whose comment describes this exact failure mode. 148 x86
  > sweep rounds could not reproduce it.
  >
  > **Fix.** `MiPurgeStaleUserPtes` (`arm64/page.rs`), called from `MmCreateMemoryArea` — the one
  > choke point every user VA reservation passes through, so VirtualAlloc and section views are
  > covered together instead of path by path. Skips 2MB at a time when the PDE is absent, so the
  > common case is not O(pages). Also restored the ARM64 block of `NtAllocateVirtualMemory` that
  > the port had skipped behind an expired premise (field manual, "A deferral note outlives the
  > premise that justified it") — correct on its own merits, though it does **not** cover the
  > section-view path that this bug took.
  >
  > **Evidence.** `[MMGUARD] rmap.MmDeleteRmap mapping-not-in-list` fired once per run on every
  > run across 3+ media builds; after the fix, **0 hits**, and `ZwMapViewOfSection` went
  > **pass=121 fail=13 → pass=123 fail=11**. The extra passes are the independent corroboration:
  > silencing a guard cannot make test assertions succeed. Ratchet holds, sweep 91/88 in 67s (no
  > boot slowdown from the purge).
  >
  > **What cracked it** (method worth reusing): the raw descriptor. `Page` alone was ambiguous
  > for days, and two intermediate readings were wrong — `balance.rs`'s `RMAP_IS_SEGMENT` bug was
  > real but not this, and `valid()` is only bit 0 so it cannot distinguish a hardware descriptor
  > from a software PTE. Logging `OldPte.Long` into the always-on flight recorder, keyed by VA,
  > settled it in one boot. A probe gated on `pfn < MmLowestPhysicalPage` fired zero times and was
  > **uninterpretable** — never use a probe whose silence has a hidden precondition.
  >
  > <details><summary>Original 2026-07-29 characterization (kept — the reasoning that got here)</summary>
  >
  > ## ROOT CAUSE CANDIDATE FOUND (2026-07-29): `MmDeleteRmap` loses the mapping, on EVERY run
  >
  > **The arm64 blocker's crash face is not a flake. The corruption is deterministic; only its
  > CONSEQUENCE is probabilistic.** Across 20 instrumented `-smp 1` rounds:
  >
  > | rounds | `MmDeleteRmap mapping-not-in-list` |
  > |---|---|
  > | 14 that ran the sweep and finished **clean** | **1 each** |
  > | 2 that stalled | **1 each** |
  > | 3 that BUGCHECKed (`0x50`, `0x1A`, `0x1E`) | **1 each** |
  > | 3 `B:no-launch` (never ran a test) | **0** |
  >
  > It fires **once per run, every run that maps a section** — and never when no test ran,
  > which is exactly the control. The operands are identical every time, across two different
  > media builds and three different stop codes:
  >
  >     [MMGUARD] rmap.MmDeleteRmap mapping-not-in-list hit#=0x1
  >     [MMGUARD]   Page=0x50   Process=0xFFFFFA000639AC20
  >     [MMGUARD]   Address=0x50000   listhead=0x0
  >
  > `listhead=0` — the PFN's rmap list is **empty** when the unmap tries to remove its entry.
  > So either the rmap was never inserted, or it was already removed. Downstream, a frame whose
  > reverse mapping was lost is a frame that can be freed and re-handed-out while still mapped,
  > which is precisely the "distant innocent victim" shape this whole entry has been chasing.
  >
  > **Why it looked like a flake:** the bookkeeping breaks on every run; whether the damaged
  > frame gets *reused* before the sweep ends is the coin flip. That single fact reconciles
  > face A's three different stop codes, the stalls, and the clean rounds — one cause, symptom
  > selected by timing.
  >
  > **This gives a DETERMINISTIC ORACLE, which changes the economics completely.** A candidate
  > fix no longer needs a soak: boot once, and ask whether `[MMGUARD] MmDeleteRmap
  > mapping-not-in-list` still appears. Seconds, not hours.
  >
  > **Note the probe that caught this** is one of the eleven arm64 MM invariant suppressions
  > armed on 2026-07-26 — it had never run on probe-bearing media until tonight.
  >
  > ### Hunt log — two hypotheses raised and KILLED by instrumenting instead of guessing
  >
  > **1. "`MmInsertRmap`'s silent returns skip the insert" — REFUTED.** `MmInsertRmap` has two
  > early returns commented *"defensive: skip instead of bugcheck"* where the C
  > (`mm/rmap.c`) calls `KeBugCheck(MEMORY_MANAGEMENT)`: on lookaside-allocation failure, and
  > on `MmGetPfnForProcess(Process, Address) != Page`. That is a textbook silent-degrade and
  > was the obvious suspect. Both were instrumented with `mm_guard_trip!` and **neither fires**
  > in a round where `MmDeleteRmap mapping-not-in-list` does. The insert path is not being
  > skipped. (The two silent returns are still wrong and should be restored to loud failures,
  > but they are not this bug.)
  >
  > **2. "The `MmFreeSectionPage` callback's parameters are swapped" — REFUTED.** The Rust
  > signature `(Context, MemoryArea, Address, Page, SwapEntry, Dirty)` matches
  > `ntoskrnl/mm/section.c:3533` argument for argument.
  >
  > ### The live lead: `Page` looks like it was derived from the VIRTUAL address
  >
  > The operands are `Address=0x50000`, `Page=0x50` — and **`0x50000 >> PAGE_SHIFT == 0x50`
  > exactly**, in every occurrence, across media builds. That is the signature of a physical
  > frame number computed from a *virtual* address. It would explain `listhead=0` perfectly:
  > nothing was ever inserted under that fabricated PFN, because the insert used the real one.
  > **Caveat, not yet excluded:** VA `0x50000` could legitimately be identity-mapped to PFN
  > `0x50`, which would make the relation a coincidence. Check that before building on it.
  >
  > **UPDATE — the `Page` fabrication theory is DEAD, and with it my own lead.**
  > `MmDeleteVirtualMapping` (`arm64/page.rs`) derives the out-param honestly:
  > `*Page = OldPte.page_frame_number()`, straight from the PTE it just exchanged out. So
  > `Page=0x50` is the **real** frame, VA `0x50000` → PFN `0x50` is simply an identity mapping,
  > and `Page == Address >> PAGE_SHIFT` was the coincidence the caveat warned about. Do not
  > spend time re-deriving this.
  >
  > **So the trail is now narrow and the question is sharp: was `MmInsertRmap` ever CALLED for
  > this page?** Four candidates eliminated by evidence, not opinion:
  > 1. `MmInsertRmap`'s two silent early-returns — probed, neither fires.
  > 2. `MmFreeSectionPage`'s callback signature — matches the C argument for argument.
  > 3. `MmFreeMemoryArea`'s page walk — zero-inits `Page` per iteration, fills it as an
  >    out-param from the unmap, never from the VA.
  > 4. `MmDeleteVirtualMapping`'s PFN derivation — reads the PTE (above).
  >
  > What is left is the **map** side, not the unmap side: find who should call `MmInsertRmap`
  > when a section view is mapped at a user VA, and prove it runs for this page. A missing or
  > mis-keyed insert (wrong `Process`, or an `Address` that was not `PAGE_ROUND_DOWN`ed the
  > same way on both sides — note `MmInsertRmap` rounds and `MmDeleteRmap` does **not**) fits
  > every observation including `listhead=0`. Probe `MmInsertRmap`'s SUCCESS path for this VA
  > and compare the key against what the delete looks up; the one-boot MMGUARD oracle makes
  > that a single cheap experiment.
  >
  > *(historical, superseded by the above)* **Next hop — the walk is EXONERATED.** `Page` reaches
  > `MmFreeSectionPage` as a parameter from `MmFreeMemoryArea`'s page walk (`mm/marea.rs`), and
  > that walk is correct: it zero-initialises `Page` per iteration and fills it as an **out
  > parameter** of `MmDeleteVirtualMapping` (or `MmDeletePhysicalMapping`), i.e. from the PTE,
  > never from the virtual address. So if `Page` really is `VA >> PAGE_SHIFT`, the fabrication
  > happens *inside* `MmDeleteVirtualMapping` — that is the next function to read, and the
  > alternative (the VA is genuinely identity-mapped and the relation is coincidence) has to be
  > excluded there too. The one-boot MMGUARD oracle makes each attempt cheap.
  >
  > Two things noticed in `marea.rs` while reading it, neither the bug but both worth an issue:
  > the callback parameter is *named* `_KWAIT_REASON_FreePage` (a bindgen constant name leaked
  > into a parameter name by the translator), and the phys-vs-virt unmap branch is selected by
  > `FreePage.is_none()`, with the function pointers laundered through `usize` + transmute to
  > stop LLVM folding them — check that indirection still matches the C's intent.
  >
  > </details>

  **THE TRIGGER IS LOCALISED (2026-07-29, 21 rounds in 44 min): the process teardown between
  `ZwMapViewOfSection` and `ZwWaitForMultipleObjects`.** Three failing arm64 rounds, three
  *different* symptoms, all at the identical point:

  | round | symptom | elapsed |
  |---|---|---|
  | 4 | `BUGCHECK 0x50` | 71.6s |
  | 8 | stall, `DID-NOT-FINISH` | 866.8s |
  | 15 | `BUGCHECK 0x1A` (MEMORY_MANAGEMENT) | 76.2s |

  Every one ends with `Finished test ZwMapViewOfSection` and **`ZwWaitForMultipleObjects`
  never starts**. In a clean round the order is 68 `ZwCreateSection` → 69 `ZwMapViewOfSection`
  → 70 `ZwWaitForMultipleObjects`. The sweep runs **one `kmtest_.exe` process per test**, so
  the failure window is the *gap between two tests*: the teardown of the process that just
  mapped sections, or the creation of the next one. That is precisely where the `[KSTACK]`
  round's other evidence sits — `[PS] PspExitThread entered`, and an `MmDeleteRmap`
  mapping-not-in-list for a **user** VA (`Address=0x50000`), which is what unmapping a view at
  process exit does.

  **This unifies face A with the stall**: same trigger point, symptom varying with timing
  (fault vs. wedge). It also matches the earlier reproduction, which the notes recorded as
  "died on `ZwWaitForMultipleObjects`, immediately after the `ZwCreateSection` /
  `ZwMapViewOfSection` canaries" — the same gap, described from the other side.

  **REORDER EXPERIMENT RUN, and the victim is incidental.** `ZwWaitForMultipleObjects` was
  moved from position 71 to position 2 (all 91 tests kept, so round time stayed ~63s). It then
  ran at position 2 **without incident**, and a round still died — `BUGCHECK 0x1E`
  (`P1=0xC000001D`, STATUS_ILLEGAL_INSTRUCTION) — with the serial's last event again
  `Finished test ZwMapViewOfSection` and the *new* next test (`RtlCaptureContextKM`) never
  starting. So the crash follows **`ZwMapViewOfSection`**, not the test that happens to be
  behind it: the victim is whatever runs next, which is why the stop code moves around.
  (A prior attempt to cut the list to 3 tests was abandoned — truncating it breaks the sweep's
  completion detection, so each round drained to the full timeout at 576s versus 63s for the
  whole 91-test list. Reordering, not truncating, is the right lever here.)

  **A FOURTH face — HOLDER-STUCK, and the kernel's own classifier names it.**
  Soak run 2 round 4: `launched=79 ok=66 bad=12`, **no bugcheck**, 4674s (round 3 swept the same
  media in 75s — 62× faster). The sweep degraded into repeated per-test `TIMEOUT … (180s) -
  terminating` (NtSystemDebugControl, PoIrp, …) until the drain budget ran out. The cause is on
  serial:

      [CENSUS] STUCK kmtest_.exe pid=816 tid=812 GateWait obj=ffff8000049044e0 parked=1 sat=0 sig=0
      [FLIGHTREC]   no events for obj=ffff8000049044e0 in the last 16384

  **Read the verdict field, not the symptom.** `census.rs` classifies `sig == 0` as
  `HOLDER-STUCK(no signal ever)` and reserves lost-wakeup for `signals >= parked` with the
  waiter still parked. All 97 census lines in this run say HOLDER-STUCK. `synctrace_signal()` is
  called at the TOP of `KeSignalGateBoostPriorityImpl`, before any lock, so it counts every
  signal *intent* — `sig=0` therefore means **nobody ever called the signal**, not that a signal
  was dropped. (An earlier revision of this entry called it a lost wakeup; that was the author
  overwriting the instrumentation's own conclusion with a guess, and it points the investigation
  in the opposite direction.)
  So the gate primitives are NOT implicated: look at whatever owns that gate and was supposed to
  complete. The `KeWaitForGate` / `KeSignalGateBoostPriority` pair in `ke/gate.rs` was read and
  both sides take the dispatcher-object lock, re-check `SignalState`, and pop under the thread
  lock — no window found. Only ONE thread is stuck, so the would-be signaller is not itself in
  GateWait. Note `obj=ffff8000049044e0` sits in the same `0xffff8000_04/05` VA family as the
  fault addresses, which is suggestive and nothing more.

  **A FIFTH face, D — a hang the instrumentation cannot see, which is itself the finding.**
  Round 8: `launched=77 ok=66 bad=10`, no bugcheck, 4652s, degrading into repeated
  `TIMEOUT … (180s) - terminating` (IoReadWrite, NtCreateSection, NtSystemDebugControl — the
  §9.97 quarantine family). Superficially identical to face C, but the serial carries exactly
  ONE census line: `[CENSUS] sync watchdog up (10s cadence, gate waits)` and **zero STUCK
  reports**. So no thread was parked in `GateWait` at all, and whatever the tests are blocked on
  is a primitive the watchdog does not observe — its own banner says *gate waits*.
  **Do not read C's absence as health here:** a silent census means "not a gate wait", not
  "nothing is stuck". That is the same not-looking-vs-looked-and-clean ambiguity §9.122 is about,
  and it currently splits one symptom (a stalled sweep) across two very different diagnoses.
  ~~**Owed:** widen the census to report ANY thread stuck >N seconds with its `State` and
  `WaitReason`, not only `GateWait`.~~ **DONE 2026-07-29** (`ke/census.rs`). The scan now keys
  on `(thread, object, State, WaitReason, WaitTime)` for every parked thread, not just
  `GateWait`:
  - `WaitTime` is in the key deliberately. It is written only at state transitions (the
    `KiSetWaitTime` inlines on park, `thrdschd.rs:479` on going Ready), never periodically, so
    it stays constant while a thread is genuinely parked — and it stops a thread that wakes and
    re-waits on the same object from reading as continuously stuck, which `(thread, object)`
    alone could not distinguish.
  - Threads on **kernel-internal lock/resource** wait reasons (`WrPushLock`, `WrFastMutex`,
    `WrGuardedMutex`, `WrResource`, `WrKeyedEvent`, `WrMutex`, `PageIn`, `PoolAllocation`, …)
    get a named per-thread `[CENSUS] STUCK … reason=WrFastMutex(34)` line, capped at 8/scan.
    NT's *generic* reasons (`Executive`, `WrQueue`, `WrLpcReceive`, `UserRequest`, …) are
    deliberately excluded from per-thread reporting — server and worker threads park on those
    forever by design and would drown the log.
  - Everything parked lands in a **wait-reason histogram** that prints only when the profile
    *shifts* (a bucket moving by ≥4, or a reason appearing at all), so a healthy system is
    silent and threads piling into one reason — face D's signature — prints once as they pile
    in. Capped at 200 prints/boot.
  - The startup banner now states the coverage (`gate waits + all parked waits w/ reason`),
    because the old banner's honest reading was *"nothing is stuck **in a gate wait**"* and it
    was being read as "nothing is stuck". That string doubles as a `--require-probe` literal so
    a silent census can never again be confused with an absent one.
  Verified live on a green 47s arm64 boot: banner present, `[EXP]`×6 + `[PHASE1]`×26, and zero
  STUCK/histogram output — silent when healthy, which is the design.

  **The histogram works, and here is the healthy baseline it produces** (amd64 `-smp 2`,
  clean round, kmtest sweep running):

      [CENSUS] wait profile: 65 thread(s) parked >20s (scan #3)
      [CENSUS]   Executive=7   UserRequest=43   WrQueue=8   WrLpcReceive=7

  That is precisely the picture face D never had: 65 parked threads, all in the benign
  population (message loops, worker pools, LPC servers). A stall to compare against this now
  has something to be different *from* — which is the whole point, since face D's evidence was
  previously the absence of output.

  **KNOWN FALSE POSITIVE in the first soak's data (2026-07-29).** The initial
  `is_kernel_lock_wait` set wrongly included NT's *generic* reasons `Executive`(0) and
  `WrExecutive`(7). Those are the default `KeWaitForSingleObject` reason, so `System` and
  `lsass.exe` service threads that park indefinitely by design report as STUCK — the very
  soak rounds show `[CENSUS] STUCK System pid=4 … reason=Executive(0)` ~4×/round. They are
  harmless and exactly identifiable, but **`census_stuck` in `conformance.jsonl` over-counts
  for that soak: filter `reason=Executive` before reading it as a wedge signal.** The two
  reasons are removed from per-thread reporting (they stay in the histogram, which is where a
  population of generic waiters belongs); the soak was deliberately NOT restarted for it.

  **Face C's next step is a two-line diagnostic, and it is already half-instrumented.** The only
  callers of `KeWaitForGate` in the tree are `ke/wait.rs:328` and **`ex-rust/src/pushlock.rs:487`
  and `:631`** — so a thread in `GateWait` is, in practice, blocked on a pushlock
  (`WrPushLock`), and the signaller that never ran is `pushlock.rs:198`'s
  `KeSignalGateBoostPriority(&WaitBlock->WakeGate)`. That means **HOLDER-STUCK here reads as: a
  pushlock holder never released, or released without waking.** Which pushlock is the question,
  and it is the one thing the report omits:
  - `gate.rs` accounts `synctrace_wait_park` under the **gate** address;
  - `pushlock.rs` accounts `synctrace_pl_park` under the **pushlock** address, deliberately —
    its own comment says "the wake gate is a per-wait stack object, useless as a key";
  - `census.rs:176` prints only the gate address, so the pushlock identity is *recorded and
    never surfaced*. The run's serial contains zero `WrPushLock` mentions for the stuck thread.

  **Do this next:** when a stuck `GateWait` thread's `WaitReason` is `WrPushLock`, have the
  census also print the pushlock address and *its* synctrace counters. That names the contended
  lock instead of a stack-allocated gate, and turns "something never signalled" into "this lock
  was held by that thread".

  **Suggestive, unproven:** because `WaitBlock` lives on the waiter's stack, `obj=ffff8000049044e0`
  IS a kernel-stack address — the same `0xffff8000_04/05` VA family as face A's fault addresses.
  That is consistent with A and C sharing a root cause (a stack whose backing goes away would
  also make a wake gate unreachable), and equally consistent with coincidence, since every kernel
  stack lives in that range. Do not merge the faces on this alone.

  **B is NOT C — tested, not assumed.** The obvious unifying story ("the launcher itself parked
  on a gate, hence `launched=0`") is **wrong**: the no-launch round has **zero** `CENSUS STUCK`
  lines, while the stalled round has them throughout. Two distinct faces.

  **FACE B IS NOT ARM64, AND NOT A KERNEL BUG — it is a harness/launcher race (2026-07-29).**
  The entry below asked for evidence before folding B in with the crash faces. Here it is,
  and it points the other way — *out* of the arm64 blocker entirely:
  - It reproduced **on amd64** (`-smp 2`, round 12, `launched=0`, 711s). Face B is therefore
    not arch-specific and cannot be part of an arm64-only blocker.
  - The two arm64 occurrences took **179.2s each — identical to the tenth of a second**. That
    is a fixed timeout expiring (the sweep's 180s launch budget), not a hang with random
    duration. A kernel wedge does not reproduce its wall-clock twice.
  - Both arm64 occurrences carry `census_stuck=0`: nothing parked, consistent with "the
    launcher never got going" rather than "the system is stuck".
  So B should be tracked as a **sweep-harness defect** (the launcher failing to start
  `kmtest_.exe` and timing out), not as a face of the kernel blocker. It still costs a soak
  round each time, so it is worth fixing, but it does not gate the MP flip.

  **A THIRD observable face — "sweep no-launch" (1 occurrence, cause unknown).** Soak run 2
  round 2: the guest **booted to desktop** (3 shell markers on serial) and then launched
  **zero** of the 89 tests, draining the full 87-minute budget with `sweep_rc=1`, no bugcheck
  and no probe hits. Serial ends mid-stream in netstack TDI teardown and `PspExitThread` churn.
  Host load was ~10 with a single QEMU running, so this is **not** simple starvation, and the
  same media had swept cleanly in round 1 and again in round 3. Recorded as an observable, not
  a diagnosis: it may be this bug wearing a hang instead of a crash, or an independent harness
  race. **Do not fold it into the crash faces without evidence** — and note the soak ledger
  renders it identically to a crash (`NO-VERDICT`), so read `summary.json`'s `launched`/`bugcheck`
  rather than the ledger line to tell them apart.

  **ELIMINATED (2026-07-26): a create/delete size mismatch on the kernel stack.** Worth writing
  down because the shape is suspicious and costs ~20 minutes to re-derive.
  `MmCreateKernelStack` (`ARM3/procsup.rs:280`) *forces* `GuiStack = 1` on arm64 and commits
  every page (`StackPages == StackPtes == BYTES_TO_PAGES(MmLargeStackSize)`), where x86 reserves
  large but commits only `KERNEL_LARGE_STACK_COMMIT`. `MmDeleteKernelStack` (`:210`), by
  contrast, takes `GuiStack` from its CALLER and sizes the unmap from it — so a caller passing 0
  would unmap `KERNEL_STACK_SIZE` pages of an `MmLargeStackSize` allocation and hand
  `MiReleaseSystemPtes` the wrong count. It would also take the dead-stack SList push, which on
  arm64 nothing ever drains (create's pop lives in the `GuiStack == 0` branch that arm64 never
  reaches). **All five call sites are nevertheless correct:** `thrdobj.rs:869/:895` and
  `kill.rs:341/:472` pass the thread's own `LargeStack`, which `thrdobj.rs:846` sets to 1 on
  arm64 next to the large `StackLimit`; and `win32.rs:103`'s hardcoded `MmDeleteKernelStack(old_stack, 0)`
  is inside `#[cfg(not(target_arch = "aarch64"))]` — arm64 skips the stack switch entirely
  because its stacks are already large. Both `kill.rs` sites also carry the §9.94 SwapBusy spin.
  So arm64 stacks are fully committed at creation and torn down at the right size: whatever
  removes the page, it is not this.

  **Reading the `[KSTACK]` numbers: implausible bounds may mean a MIS-MAPPED structure, not a
  corrupt stack.** The wow64-amd64 lane hit exactly that failure mode the same night
  (`335f2009cd6`): a WoW64 thread's TEB landed on top of `KUSER_SHARED_DATA`, so
  `Teb.StackBase = 0x000000000FD4BC00` / `Teb.StackLimit = 0x90A70AA000000000` read back as
  *the clock* — `InterruptTime.LowPart` and `High2Time | SystemTime.LowPart << 32`. Not corrupt
  bounds; the wrong page. That was a user TEB and `[KSTACK]` reads the kernel `KTHREAD`, so it
  is not the same bug — but the lesson transfers: if `StackBase`/`StackLimit` come back as
  nonsense, first ask whether the structure we are reading is where we think it is, and only
  then conclude the stack is damaged. Sanity check they bracket `InitialStack` and are
  `KERNEL_STACK_SIZE` (0xC000 on arm64) apart before drawing any conclusion from them.

  **Two probe-coverage lessons from that run, both now fixed.**
  1. `[KSTACK]` printed nothing, because it lives in `stubs.rs`'s `[KEXC]`
     kernel-mode-*unhandled-exception* path, which ends in `0x1E`. A `0x50` is raised straight
     out of `MmArmAccessFault` and never reaches it. The same dump is now called at all four
     live `PAGE_FAULT_IN_NONPAGED_AREA` sites (`arm64_dump_fault_stack`), so the next
     reproduction of *either* face reports StackBase/StackLimit.
  2. `[FREEMAPPED]` printed nothing, and **cannot** see this face by construction:
     `MmDereferencePage` is the rmap-tracked *user* page path, whereas kernel stacks come from
     `MmCreateKernelStack` via system PTEs and are released by a different route. Its silence
     is therefore not evidence about the stack face at all — only about the user-page face. If
     the `[KSTACK]` dump confirms a stack-page loss, the next tripwire belongs at the kernel
     stack free site, not at the page free site.

  **Earlier measurement — a HEALTHY arm64 UP boot to desktop (59s) fires NOTHING:**
  `FREEMAPPED 0`, `KSTACK 0`, `PFNMISMATCH 0`, `BUGCHECK 0`. That silence is a result, not a
  non-event: the worry that benign stale rmap heads would make `[FREEMAPPED]` chatty (the
  reason it is a print and not an assert) does **not** materialise on a normal boot, so the
  probe is precise and any future hit is signal rather than noise. It also means a clean boot
  cannot be used as evidence the probe works — only a run that reproduces the bug can.

  **First symbolicated kernel-side instance (2026-07-26) — and it points at the STACK, not at
  the page-reuse face.** `-smp 1` run F: `BUGCHECK 0x1E (0xC0000005, 0xfffff80042193788,
  0xffff8000048e8e50, 0)`, `P2 = ntoskrnl.exe+0x3788` = **`KiSynchronousExceptionEntry+4`**
  (`ntoskrnl/ke/arm64/trap.S:371`). `+0` is `sub sp, sp, #KTRAP_FRAME_LENGTH`; **`+4` is
  `stp x0, x1, [sp, #TfX0]` — the FIRST store into the freshly-allocated trap frame** — and P3,
  the faulting address, is `0xffff8000048e8e50`, a kernel-stack VA. So a synchronous exception
  arrived and the handler prolog could not write the stack underneath it.

  A stack-exhaustion suspect exists but is **NARROWER than it first looks — check the scope
  before chasing it.** `ke/arm64/kiinit.rs:63` has a file-local `KERNEL_STACK_SIZE = 0x6000`
  against `xdk/arm64/ke.h:49`'s `0xC000`, and the quest doc flags it. But it governs ONLY the
  two static P0 arrays it is declared next to — `KiP0BootStackData` and
  `KiP0DoubleFaultStackData`. **Per-thread kernel stacks are NOT half-size:**
  `ARM3/helpers.rs:400` defines arm64 `KERNEL_STACK_SIZE = 0xC000` and that is what
  `MmCreateKernelStack` (`ARM3/procsup.rs:200`) allocates from. So a kmtest thread taking an
  exception is on a correct 48KB stack, and the half-size const can only be implicated for a
  fault taken while running on **CPU0's boot/idle stack** — possible for an interrupt-time
  exception, but not the default explanation.

  Distinguishing test, in order: log `StackBase`/`StackLimit` of the current thread against P3
  at the 0x1E — that says in one boot whether the faulting SP was even inside a thread stack, an
  idle/P0 stack, or neither (the last would point back at the page-reuse face rather than
  exhaustion). Only if it lands in the P0 range is rebuilding with `0xC000` worth a sweep. Note
  this is a DIFFERENT face from the proven user-mode page-reuse corruption above
  (`0xC000001D` garbage `.text`, `0xC0000005` in ntdll heap init) — do not assume one fix
  closes both.

  **This is now the arm64 blocker for the MP-default flip** (2026-07-26): not because MP is
  implicated — it is not — but because a ratchet floor cannot be seeded from a
  non-deterministic sweep. Measured this session on arm64, all on the SAME media: `-smp 2` run
  A clean (86 ok / 3 bad), `-smp 2` run C `ZwWaitForMultipleObjects` NO-END `exit=0xC0000005`,
  `-smp 1` run B BUGCHECK `0x1E` at `FileAttributes` (71 in), `-smp 1` run D clean to the end
  (89 launched). Both CPU counts, different victims each time — the registered signature, not
  an SMP defect.
  **2026-07-25 — the `[PFNMISMATCH]` probe was RUN, and its silence is now a measured result.**
  A quarantine probe booted the arm64 candidates one list at a time (`tools/conformance_probe.py`).
  THREE of this family crashed with a recorded verdict, and the detector fired **zero** times
  across every boot: `NtCreateSection` → `0x50` PAGE_FAULT_IN_NONPAGED_AREA (read of
  `0xffff800004544040` from PC `0xffff8000048595e0`), `ZwAllocateVirtualMemory` → `0x4E`
  PFN_LIST_CORRUPT, `IoCreateFile` → `0x1E` (AV at `0xfffff800421c960c`, a driver-range PC).
  (An earlier revision said FOUR and named `MmMapLockedPagesSpecifyCache` on no evidence — the
  probe mis-attributed truncation and dropped it unrun, fixed in `fa0b294e3e4`.) **A round-2
  probe with the fixed tool has now settled every one of those:**
  `MmMapLockedPagesSpecifyCache` **CRASH** (own verdict — so the original claim was right, but
  only by luck, and only now evidenced), `IoFilesystem` **CRASH**, `MmReservedMapping` **CRASH**,
  `NpfsFileInfo` **CRASH**, `SeTokenFiltering` **INCOMPLETE** (it kills the VM outright — no
  bugcheck, QEMU exits, see the quest doc), and `ExFastMutex` **OK** — un-quarantined, it was
  arm64's last arch-specific entry. `[PFNMISMATCH]` still fired **zero** times across all of it.
  The probe strings are compiled into this exact kernel (verified present in
  `build-arm64/ntoskrnl/ntoskrnl.exe`, absent on i386/amd64), so this is genuine silence, not a
  missing probe — which **confirms the entry's own prediction** that `MiDeletePte` is not the
  reuse path. The `ZwAllocateVirtualMemory` crash is newly localised: `MiInsertStandbyListAtFrontImpl`
  (`ARM3/pfnlist.rs`) subcode `0xAD`, params `(pfn=0x40006, ref=0, frame=0x4259f)`; `ref==0` rules
  out the reference-count clause, so a page reaches the standby-list-front insert **without
  `PrototypePte` set**. That guard is a mistranslated `ASSERT` (C no-ops it in retail) but must
  NOT be silenced — `pfnlist.rs` is arch-generic and amd64 never trips it, so it is detecting the
  real arm64 divergence (§9.136). Blocked on offline symbolication of the `0x50` PC: the kernel
  runtime base is not printed on serial, so `addr2sym --reloc` cannot be used (task #33).
- **net suite one-test timeout (`test_tdi_selftest_roundtrip` et al) — FIXED 2026-08-09, root
  cause was the HARNESS.** Three independent guest consumers (netstack `engine.rs`'s outbound
  client, `tditest.rs`'s TDI roundtrip, `nettest.exe`) all dialled the same slirp guestfwd
  endpoint `10.0.2.100:9999`, and slirp hands an echo to the FIRST socket matching (addr, port)
  — so one consumer got the reply and the others timed out. `-cmd:cat` masks this on the native
  backend by forking a fresh `cat` per CONNECTION; the windows-host rewrite bound the whole
  forward to ONE `-chardev` byte stream, making it a coin flip. Fixed by giving each consumer
  its own port + chardev + echo server (`vm.GUESTFWD_ECHO_PORTS`). **Every earlier diagnosis
  here was wrong** — host contention, a VPN, the host resolver, a red base — and two of the
  discriminators this registry recommended were backwards: the ~4× wall clock is just the loser
  waiting out its marker timeout, and "the engine-level echo to the very same endpoint succeeds
  in the same boot" identified the thief, not an alibi. Field manual, "`NET_FLAKE` that is not
  a flake". Cost: ~90% gate failure and a day of wedged merge queue.
- **A lane-independent flaky STAGE costs one gate per lane, and the conductor cannot tell it
  from a bad lane** (observed 2026-08-09 on the net flake above; kept because it is a property
  of the queue, not of that bug). A red batch splits recursively to find the culprit, so a
  single flaky shared stage rejects each lane SEPARATELY and burns a full gate on each — in one
  drain all four queued lanes (`shell32-com` ×2, `smp-bringup`, `msvcrt-homebrew`, `research`)
  were rejected in succession and the queue emptied with nothing landed for ~100 minutes. Both
  cases look identical from the outside: a red gate against the same base. A lane whose diff
  cannot possibly affect the stage — docs, `tools/rustconv` Python, tests — still pays full
  freight. If this bites again: retry the flaky stage against a quiet host rather than
  mid-batch, raise the retry count for a stage already classified a known flake, or let a lane
  touching no build input skip it. One reason it stayed unowned for a day: `_net_once` labels
  the FIRST failure `NET_FLAKE` by construction, so the retries are the only thing separating a
  one-off from a permanent red — and the lane owner is told "flake" either way, before that
  distinction reaches them.
- **amd64 CLIENTINFO-on-Ctrl+Esc** (§9.41) — NOT reproducing since the §9.40 DesktopHeapReAlloc
  fix (4/4 start-menu opens; `test_start_menu_opens` gates it). Watch item only.
- **i386 "session-pool gate" `BAD gate Type=0x80`** (§9.24) — REFRAMED to pushlock-class (stack
  `_EX_PUSH_LOCK_WAIT_BLOCK.WakeGate`, not session pool); not reproducing, likely closed by the
  §9.13 WS-pushlock fixes. Re-open only on a live recurrence → GDB watchpoint on the WakeGate byte.
- **arm64 BUGCHECK 0x3 — IT IS `RESOURCE_NOT_OWNED`, NOT `INVALID_AFFINITY_SET`** (renamed
  2026-07-25, field manual §9.124). Proved by enumeration, not inference: across the ENTIRE
  pre-`d3ca121cb09` Rust kernel exactly **two** sites could raise stop code 3, both in
  `ex-rust/src/resource.rs` — `ExReleaseResourceForThreadLite` and `ExSetResourceOwnerPointer`,
  whose `RESOURCE_NOT_OWNED` (0xE3) the LLM had invented as `0x03` (§9.113). The only C raiser,
  `KeBugCheck(INVALID_AFFINITY_SET)`, passes **all-zero** parameters. Every historical hit
  carried FOUR non-zero ones, e.g. `0x3 (0xfffffa0006343870, 0xfffffa0006221c00, 0, 3)` —
  which is `(Resource, Thread, Resource->OwnerTable, 3)` exactly, down to the literal trailing
  `3`, with two plausible arm64 pool pointers and **OwnerTable == NULL**. So: a thread released
  (or reassigned ownership of) an ERESOURCE it does not own, on a resource with no owner table.
  The Rust is a faithful translation of the C here (the C's `ASSERT(Owner != NULL)` is a retail
  no-op), so this is a real ownership bug, not a translation bug. **The `[AFFPROBE]`
  instrumentation in `ke/thrdobj.rs` is aimed at the wrong subsystem and its silence is not
  evidence** — comment updated in place. Hunt from the ERESOURCE side now. NOTE the search space
  cuts both ways: from `d3ca121cb09` on, `KiSetAffinityThread` raises a real 0x3 again, so a
  FRESH hit with all-zero params is a genuine affinity bug and a different animal.
  *(Superseded entry, kept for its reasoning:)*
- **arm64 BUGCHECK 0x3 (INVALID_AFFINITY_SET) — rare non-deterministic** (long-standing;
  latest hit 2026-07-18 during an arm64 smoke, mid service-autostart, no WoW64 involvement:
  `0x3 (0xfffffa0006343870, 0xfffffa0006221c00, 0, 3)`). `KeSetSystemAffinityThread` carries
  an `[AFFPROBE]` caller-LR probe (ke/thrdobj.rs, "task#10") hunting the root caller that
  passes a mask excluding all active CPUs; the probe did NOT fire on the 07-18 hit, so the
  bad mask reaches the scheduler by another route. Not in the gate's auto-retry flake list —
  consider adding once the signature is pinned.
  **2026-07-24 — the search space CHANGED, re-read old evidence with care.** Until commit
  `d3ca121cb09`, `KiSetAffinityThread` (ke/thrdschd.rs) raised **0x48**, not 0x3 — the
  translation had invented the number (field manual §9.113). So every 0x3 observed BEFORE
  that commit provably came from somewhere *other* than that site, and any 0x48 seen in the
  old corpus was really this affinity check. From `d3ca121cb09` on, `KiSetAffinityThread` is
  a live 0x3 candidate again, so a fresh hit is NOT necessarily the same bug as the historical
  ones. Note the historical hits carry 4 non-zero parameters, which `KeBugCheck(code)` (all
  params 0) cannot produce — so the raiser is a `KeBugCheckEx` site and the hunt should start
  by enumerating those, not by re-reading the setter.
- **KD continuation** (windbg handoff) — amd64 same-CPL `int 2Dh` resumed-context corruption;
  attach itself works. Low priority; also in the Check-back ledger.

## Priority Order (suggested)

**Near-term is the Phase 0 tiers above** (Opus-only, post-Fable): T1 finish unlocks (allow-multiple-def, DUMMY_PSEH) → T2 systemic Rust-native (trampoline refactor, msvcrt-from-pieces, crate-split ke/mm/win32k) → T3 resume translation (ROS-native DLL batches, then Wine) → T4 debt burndown (opportunistic). The table below orders the **longer-horizon architecture & future** items (§1–§9), which sit *after* the Phase 0 tiers:

| # | Item | Size | Blocked By | Impact |
|---|------|------|-----------|--------|
| 1 | Testing infrastructure (4a-c) | M | Nothing | Enables everything else safely |
| 2 | SEH idiomation Stage 3 (3) | L | Trampoline refactor (§0 T2) helps | Correctness + code quality |
| 3 | Pipeline auto-repair (2b) | M | Nothing | Translation velocity (feeds §0 T3) |
| 4 | Boot trace expansion (5c) | S | Nothing | Debug velocity |
| 5 | Semantic plateau mitigation (5a) | M | Testing infra | Runtime correctness |
| 6 | SQLite registry (1) | L | Nothing | Architecture simplification |
| 7 | Atomic refcounting (6a) | M | Testing infra | Performance + correctness |
| 8 | Header reconstruction (2a) | M | Nothing | Translation coverage |
| 9 | Idiomatic Rust (2c) | L | SEH idiomation | Code quality |
| 10 | ReactOS upstream tracking (7a) | XL | Pipeline maturity | Sustainability |
| 11 | Lock-free structures (6b) | L | Testing infra | Performance |
| 12 | WoW64/WoWARM64 + FEX (8) | XL | Stable kernel | App compatibility |
| 13 | x86 deprecation (9) | L | WoWARM64 working | Simplification |
| 14 | Futex primitives (6c) | XL | Lock-free work | Architecture |
| 15 | Wine DLL re-implementation (7b) | XXL | Pipeline + Rosetta | Full Rust userland (the §0 T3 Wine bucket) |
| 16 | Linux-origin subsystem refresh (7c) | XL | Pipeline maturity | Remove stale ports |
| 17 | Final bosses: Windows 10 drivers, GPUs, Chrome (12) | XXL | Real-hardware boot (10) | Runs today's Windows binaries unchanged |
