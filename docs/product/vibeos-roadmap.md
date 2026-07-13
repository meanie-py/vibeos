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

- **⭐ Exported-symbol trampoline refactor** *(new; Size: M, mechanical)* — split every exported symbol into a snake_case **native `nt_foo()`** (plain Rust ABI, holds the logic) + a generated **`NtFoo` trampoline** that is the *only* place `extern "system"/"fastcall"/stdcall` lives. Done mechanically in Python across all ~1,700+ monolith exports (thousands fleet-wide). **This is the capstone of the ghost-tax "two copies of the truth" program**: internal callers hit the native fn, so there is no hand-written `extern` decl left to drift — it kills the ABI-mismatch bug class the crate-split flushes cross-crate, `scan_cc_mismatch` catches same-crate (316 baseline), and `verify_ffi` guards (161). Also the natural substrate for idiomation (§3) and for **generating** crate-split's cross-crate decls instead of hand-writing them — so do it *aware of* the crate-split.
- **msvcrt / ucrtbase from pieces** *(new; Size: M-L)* — the CRT floor is still ~581 `.c`. Rather than translate it from scratch, **assemble** the Rust CRT from what already exists — `cnt-rust` (libcntpr kernel CRT subset) + the 39-crate `rtl-rust` umbrella — and NT-ify the gaps. Displaces the last big userland C floor.
- **ntoskrnl crate-split continuation** *(Size: M)* — debug + land **ex** (x86 hang, branch `ex-crate-split-x86-hang`), then split **ke** and **mm/ARM3**; then **win32k** (flat 257K single crate, textbook USER/GDI seam, `freetype` a free leaf — one coordinated pass in a quiet window). Detail + recipe: the crate-split plan.

### Tier 3 — Resume translation (the through-line)

Genuine remnants only — the boot path is already Rust. Measured 2026-07-07 from `media/doc/WINESYNC.txt` provenance + `wine/`-subdir + the live `*-rust` crate set.

**ROS-native userland — ~140 components, ~240K native LOC (the priority queue).** Batched by shape for rustconv (a name-collision caveat: a few like `smss`→`smss-csrss` are already done; the count is a mild over-estimate):

| Batch | Top targets (native LOC) |
|---|---|
| **Winsock / net client** | ws2_32 (14K), dhcpcsvc (6.4K), dnsapi (3.2K), mswsock, ws2help, wshtcpip |
| **Setup / PnP / device** | devmgr (10K), netcfgx (9K), syssetup (9K), umpnpmgr (7K), newdev, msports |
| **Control panels (`dll/cpl`)** | desk (11.5K), sysdm (7.9K), intl (5.2K), mmsys (3.8K), timedate, access, main, usrmgr, powercfg, input — ~15 CPLs, uniform shape, very batchable |
| **Services (`base/services`)** | eventlog (3.5K), netshell (4.5K), svchost, wkssvc, dnsrslvr, audiosrv, wlansvc |
| **Security / auth** | msv1_0 (3.3K), samlib (2.4K), ntmarta (1.8K), aclui (5.7K), authz — completes the LSA/SAM chain already in flight |
| **Shell exts / apps / audio** | zipfldr, shimgvw, fontext, ntobjshex, stobject; console (3K), diskpart (9.3K); wdmaud.drv, mmdrv, the `*.acm` codecs |
| **misc system DLLs** | userenv (6.5K), psapi (1.7K), framedyn, hhctrl.ocx (5.2K), fmifs |

**Wine-derived userland — ~154 DLLs, ~1.05M LOC (lower priority than Tier 2).** kernelbase (41K, `kernelbase/wine/`), comctl32, comdlg32, ole32, oleaut32, gdiplus, riched20, urlmon, wininet, mshtml, jscript/vbscript, msxml*, windowscodecs, crypt32, wldap32, rpcrt4, the whole `dll/directx/wine/` stack. Re-implementing these from *current* Wine via an LLM pipeline (§7b) beats translating the frozen fork — that's why they sit below the native queue.

**Already translated (frontier boundary):** kernel/win32k, kernel32, ntdll, user32, gdi32, advapi32, secur32, shell32, msafd, mlang, mscms, version, cabinet + the boot/server spine (winsrv, smss-csrss, lsasrv, samsrv, msgina, services, winlogon, userinit, explorer) + the driver fleet.

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

---

## 4. Testing Infrastructure Modernization

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
  writeup `../engineering/active-lanes/stable-toolchain-plan.md`). All 4 nightly deps killed:
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

- **arm64 0x1E / `[WAITDEP]` ALL-ONES residual** (~2 hits / 2 days organic gate) — the
  dominant netstack-TDI source is FIXED (§9.36) but a residual arm64 dispatcher/stack
  corruption with the same ALL-ONES `bad-thread-ptr` face still recurs (07-09 gate
  `ntoskrnl+0x451B0`, evidence in `~/vibeos-flake-farm/waitdep-evidence/`; 07-10 overnight
  `+0x1CD8C` retried green). `ke/waitdep.rs` always-on captures each hit — a `[WAITDEP]`
  0x109 in a gate log IS the live capture. §9.36 addendum. **2026-07-11 §9.54 reframe:
  all recorded arm64 `+0xNNNN` top-symbols were MASKS (every arm64 bugcheck nested through
  a KeGetBugMessageText fault; harness logged the reporter's crash PC). Mask fixed + one
  true face root-caused & fixed (KiUserModeCallout callback-ERET IRQ-clobber, dropped-`cli`
  translation bug). Re-triage the residual from fresh evidence only — symbolicate the FIRST
  banner in a log, never the last.**
- **winlogon 0xC000021A→0x4C init race** (~3/20, all arches) — kernel32 `%wZ` races basesrv's
  CSR shared-section population. MITIGATED (snapshot validation + loud `snapshot CORRUPT`
  print); real fix owed: publish-last ordering in csrsrv/basesrv. §9.36.
- **win32k `DialogBoxParamW` dialog-creation race** (amd64) — WM_INITDIALOG occasionally never
  dispatched on the winlogon desktop; wedge defensively fixed (§9.12), underlying race open
  (would bite real SAS/logon dialogs outside autologon).
- **amd64 start-menu render residue** (§9.41) — Programs/Documents/Settings render without label
  text + Run/Shut Down dropped (i386 renders full). Root-caused on the `screen-painting` lane
  (GDI TEB batch never flushed — `GdiBatchCount` read at 0x74 vs real 0x1740, `ee3c81e348c`);
  strike this when that lane lands on master.
- **amd64 CLIENTINFO-on-Ctrl+Esc** (§9.41) — NOT reproducing since the §9.40 DesktopHeapReAlloc
  fix (4/4 start-menu opens; `test_start_menu_opens` gates it). Watch item only.
- **i386 "session-pool gate" `BAD gate Type=0x80`** (§9.24) — REFRAMED to pushlock-class (stack
  `_EX_PUSH_LOCK_WAIT_BLOCK.WakeGate`, not session pool); not reproducing, likely closed by the
  §9.13 WS-pushlock fixes. Re-open only on a live recurrence → GDB watchpoint on the WakeGate byte.
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
