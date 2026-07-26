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
- **msvcrt / ucrtbase from pieces** *(new; Size: M-L)* — the CRT floor is still ~581 `.c`. Rather than translate it from scratch, **assemble** the Rust CRT from what already exists — `cnt-rust` (libcntpr kernel CRT subset) + the 39-crate `rtl-rust` umbrella — and NT-ify the gaps. Displaces the last big userland C floor.
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
  `Protect`/`Dirty`.

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
  against a kernel with no probe in it.

  **THE HEADLINE FROM 9 INSTRUMENTED SOAK ROUNDS (2026-07-26): this is NOT one bug.** Two soaks
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

  **FIRST REPRODUCTION UNDER THE PROBES (2026-07-26), and it points hard at the STACK.**
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
  **Owed:** widen the census to report ANY thread stuck >N seconds with its `State` and
  `WaitReason`, not only `GateWait`. Until then C and D can only be told apart by the presence
  or absence of a STUCK line, which is a weak discriminator resting on the watchdog's coverage.

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
- **net suite `test_tdi_selftest_roundtrip` — flake rate measured 2026-07-24, and it is BAD
  under host load.** 1 pass / 6 attempts in one evening while a sibling lane ran a full arm64
  ninja (host load 4–6). The gate's `NET_FLAKE` auto-retry is 3 attempts, so at this rate a
  green gate is roughly a coin flip. **Timing signature is the useful discriminator: a passing
  run finishes in ~36s, a failing one burns ~122s and dies on the marker wait** — i.e. it is a
  timeout, not a wrong result. TDI CONNECT reaches `ESTABLISHED` and the *engine-level* echo to
  the very same guestfwd endpoint (`10.0.2.100:9999`) succeeds in the same boot, so the host
  echo is alive and only the TDI pended-IRP completion misses its window. Ruled out as a code
  regression: the merge of 2026-07-24 touched **no** netstack source (only `afd-rust`, which
  this self-test bypasses by being the TDI client itself), and the self-test is kernel-mode on
  its own system thread with no dependency on the usermode components. Owed: either give the
  marker wait a load-proportional budget or make the poll worker's completion deterministic —
  and stop treating 3 retries as sufficient cover.
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
