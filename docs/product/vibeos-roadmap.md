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

- **`--allow-multiple-definition` elimination** — kill the load-bearing link flag categorically (spike proven; see the ratchet doc + `tools/scan_duplicate_symbols.py`).
- **DUMMY_PSEH resolution** — decide + execute the path off the SEH-is-dead-code hack across all arches (spike + open-question doc landed).

### Tier 2 — Systemic Rust-native fixes (above Wine-DLL translation)

- **⭐ Exported-symbol trampoline refactor** *(new; Size: M, mechanical)* — split every exported symbol into a snake_case **native `nt_foo()`** (plain Rust ABI, holds the logic) + a generated **`NtFoo` trampoline** that is the *only* place `extern "system"/"fastcall"/stdcall` lives. Done mechanically in Python across all ~1,700+ monolith exports (thousands fleet-wide). **This is the capstone of the ghost-tax "two copies of the truth" program**: internal callers hit the native fn, so there is no hand-written `extern` decl left to drift — it kills the ABI-mismatch bug class the crate-split flushes cross-crate, `scan_cc_mismatch` catches same-crate (316 baseline), and `verify_ffi` guards (161). Also the natural substrate for idiomation (§3) and for **generating** crate-split's cross-crate decls instead of hand-writing them — so do it *aware of* the crate-split. **Re-scoped 2026-07-25 — Size is L-XL, not M: the real surface is 24,823 `#[no_mangle]` exports fleet-wide** (the ~1,700 figure was ntoskrnl-only, and is accurate for it; win32k alone is 3,788). It also cannot be an edit campaign over the decls that exist — it must GENERATE them from one authority and retype call sites in the same pass. Two shortcuts were probed and closed as dead ends (decl-follows-def bakes in wrong ABIs; `verify_ffi`'s 6 ABI findings are correctly baselined local shims, not bugs) — see the warnings-burndown lane plan, 2026-07-25, before restarting.
- **msvcrt / ucrtbase from pieces** *(Size: M-L; msvcrt DONE, ucrtbase's rail LANDED 2026-09-05)* — the CRT floor is still ~581 `.c`. Rather than translate it from scratch, **assemble** the Rust CRT from what already exists — `cnt-rust` (libcntpr kernel CRT subset) + the 39-crate `rtl-rust` umbrella — and NT-ify the gaps. Displaces the last big userland C floor. **ucrtbase now links Rust**: its shared TUs live in `crt-shared-rust` and it links `crt-shared-cargo`, deliberately NOT msvcrt's archive — those two collide with ucrtbase.spec on 891 names of which only 35 are ours, so the obvious wiring would have silently shadowed 856 genuine Microsoft UCRT bodies under `--allow-multiple-definition`. `tools/verify_ucrtbase_cohab.py` gates it; see rust-port-status.md's CRT row.
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
| 10d | **Real NIC** (replace virtio-net; interrupt-driven miniport → netstack) | [`../engineering/archive/real-nic-networking-quest.md`](../engineering/archive/real-nic-networking-quest.md) | M–L | parked; 10c landing opened the arm64 path |

Not trash, leave alone: the GOP linear-framebuffer display driver (`arm64vid`, shared all
arches) and the PL011 UART (a real ARM peripheral).

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
  explicit fallback); usbxhci driver gap for real hardware.
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

The committed home for open bugs (was: agent MEMORY.md, which other agents never see).
Field-manual § pointers hold the full writeups. **Update on state change.**

- **PARTLY FIXED (2026-09-05, task #39): the ~2% arm64 `--smp 1` boot wedge is an EL1 fault
  loop at a user PC, and the loop is now closed — the jumper is not yet found.** A 300-round
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
- **Three upstream win32k `ERR`s fire on every clean i386 boot — NEWLY VISIBLE, OPEN**
  (2026-09-05). The no-op-debug-macro burndown made win32k's always-on `ERR` real, and a green
  boot to `shell ready` now prints 640 lines from exactly three sites: `msgqueue.split2.c`'s
  `"Sending to Suspended Thread Msg %lx"` **333x** (mostly `Msg 121` = `WM_UNKNOWN`-range),
  `clipboard.c`'s `"Cannot open winsta"` **229x**, and `class.c`'s
  `"SYSTEMCUR(ARROW) == NULL, should not happen!!"` **78x** — upstream's own wording on that
  last one. All three are genuine ReactOS `ERR()` calls that a stock DBG build would also print,
  so this is not a regression we introduced and not noise to suppress; it is three real defects
  that were invisible for as long as the macros were silent. Boot time is unaffected (32s).
  Nobody owns these yet. Field manual: the no-op-macro entry under "An arch-gated no-op trace fn
  discards the diagnostics on the arch that needs them".
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
- **127 constants are INVENTED and say so — OPEN** (2026-09-05). Found while chasing
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
    2, so 4 assertions stop EXECUTING rather than start failing — same shape as the arm64
    `KeEvent` 132-pass drop below. A loop terminating early, not four broken assertions.

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
  This is the §9.98 physical-page-reuse cluster, **OWNED BY the smp-bringup lane** (task #19/#23)
  — the instrumentation and the corpus are the contribution from here, not a fix.
- **arm64 UP:=smp1 collapse FAILS on `KeEvent`, deterministically — NEW, OPEN** (2026-07-29,
  first three rounds of the instrumented soak). The MP kernel at one CPU is *not* equivalent
  to the UP kernel on arm64, and the flip is gated on exactly that equivalence:
  - `KeEvent: pass 42733 -> 42601 (dropped)` — **132 fewer passes, in every round so far**
    (3/3). Deterministic, so it is a real MP-vs-UP semantic difference, not the flake.
  - `ExPools: pass 14 -> 13, fail 4 -> 5` — present in rounds 1 and 3, absent in round 2, so
    this one IS flaky, at one test.
  Note the sweep launched **91** tests against an 89-test baseline, so the baseline also wants
  re-seeding once the deltas are understood. Separating "deterministic" from "flaky" here is
  only possible because rounds are repeated on one frozen media and recorded — see
  `conformance --repeat` and `~/vibeos-flake-farm/conformance.jsonl`. **Next:** diff the
  per-subtest KeEvent results between the UP baseline and an `--smp 1` round to name which 132
  cases changed; `KeEvent` at 42733 cases is dominated by a few big loops, so the drop is
  likely one loop terminating early rather than 132 unrelated failures.
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
  result**: it eliminates that site. The one to read first is
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
