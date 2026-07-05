# vibeos Roadmap — Future Enhancements

*"An operating system for nobody"* — built by AI, verified by compiler, stress-tested at kernel level.

Items below are drawn from ChatGPT planning sessions and ongoing work. Scoped with t-shirt sizes: **S** (days), **M** (1-2 weeks), **L** (weeks-months), **XL** (quarter+).

---

## Phase 0 — Active Translation Work (before this roadmap)

Everything below in this document is longer-term. The immediate translation campaign is tracked in these documents and should be substantially complete first.

Completed items: see [`vibeos-roadmap-complete.md`](vibeos-roadmap-complete.md)

### 0a. IO Subsystem Activation — [`status-update.md`](status-update.md)

IO subsystem C→Rust activation is **substantially complete**. All iomgr/ and pnpmgr/ modules now active on all 3 arches. Remaining: ARM64 desktop rendering (SMSS not starting), amd64 taskbar regression (explorer shell UI not rendering). See status-update.md and engineering docs.

### 0b. Finish ntoskrnl Translation — [`../engineering/rust-port-status.md`](../engineering/rust-port-status.md) (per-file history: [`archive/ntoskrnl-translation-audit.md`](../engineering/archive/ntoskrnl-translation-audit.md))

**~52K lines of C remaining** across 70 files in 6 subsystems. Current kernel is ~344K lines Rust (~80%). After this phase: ~98% Rust.

| Subsystem | Files | Lines | Priority | Notes |
|-----------|-------|-------|----------|-------|
| **config/** (Registry CM) | 24 | 18.8K | HIGH | Last major untranslated subsystem. Complex hive B-tree + cell allocator. Depends on cmlib (8.8K shared lib). Would bring kernel to ~95% Rust. |
| **fstub/** (Disk partitioning) | 4 | 5.8K | MEDIUM | MBR/GPT partition support, HAL stubs. Straightforward. |
| **kd64/** (WinDbg protocol) | 11 | 6.8K | DEFERRED | Already translated to Rust but **disabled** — recursive exception flood on amd64 (INT3 → KdpTrap → infinite nesting). Needs debugging, not retranslation. |
| **kdbg/** (Interactive debugger) | 9 | 12.3K | LOW | ReactOS-specific debugger. Dev tool only, not boot-critical. |
| **kd/** (Low-level debug I/O) | 9 | 2.2K | LOW | Serial/keyboard/terminal for debugger. |
| **cache/** (NewCC) | 13 | 6K | SKIP | NEWCC-only code path, not active. |

**Approach:** rustconv batch pipeline for config/ and fstub/ (~$0.65 API cost). KD64 needs the recursive exception bug diagnosed (possibly another Codex debugging session). KDBG/KD are low priority — debugger code that doesn't affect boot or runtime.

**Size: M-L** — config/ is the hard one (complex data structures, locking). fstub/ is a quick win. Total ~2-3 weeks.

### 0c. Beyond ntoskrnl — [`../engineering/beyond-ntoskrnl-targets.md`](../engineering/beyond-ntoskrnl-targets.md)

After ntoskrnl is fully Rust, translate the rest of the kernel-adjacent stack. **~500K+ lines across multiple components**, spanning months of work.

**Phase 1 — Kernel-mode (reuses existing rustconv infra):**

| Target | Lines | Notes |
|--------|-------|-------|
| HAL/halarm (ARM64) | 2K | Immediate — same build system, tiny |
| ksecdd.sys | 1.4K | Tiny boot driver, perfect starter |
| HAL/halx86 | 23K | Core HAL, heavy inline asm (port I/O, APIC) |

**Phase 2 — win32k (same kernel-mode infra, ~134K lines):**

| Target | Lines | Notes |
|--------|-------|-------|
| gdi/eng/ | 19K | Graphics primitives, pure computation — good entry point |
| ntgdi/ selective | ~10K | Bitmap/brush/pen ops |
| ntuser/ selective | ~15K | Input, cursor, hooks |
| Full win32k | 114K | Long campaign. Window manager is one giant mutable state machine. FreeType (7.9K) probably stays in C. |

win32k is **comparable in size to ntoskrnl** (~134K lines kernel core, ~338K total win32ss). Easier than ke/ (less asm) but harder than io/ (286 global mutable statics in ntuser alone, deep re-entrancy in message loop/hooks). Best entry: gdi/eng/ primitives.

**Phase 3 — User-mode foundation (needs new Rust target + `#![no_std]`):**

| Target | Lines | Notes |
|--------|-------|-------|
| SMSS | 6.5K | First user-mode Rust process. Pure Nt* API, no Win32 deps. |
| NTDLL ldr/ | 9.2K | PE loader. Runs before heap init. High security value. |
| CSRSRV | 7K | CSRSS infrastructure. |

User-mode Rust requires new infrastructure — can't use Rust std, need `#![no_std]` with manual allocation. The syscall dispatchers are arch-specific ASM (stay in ASM).

**Phase 4 — Security stack + drivers:**

| Target | Lines | Notes |
|--------|-------|-------|
| lsasrv + samsrv + msv1_0 | 34.7K | Full auth chain. ReactOS-native (not Wine). |
| advapi32 | 32.1K | ReactOS-native advanced API. |
| fastfat.sys | 68.8K | Primary filesystem driver. |
| sdk/lib/rtl | 57.1K | Shared runtime — benefits ALL components. |

**Size: XL-XXL total** — Months of work across multiple phases. Each phase is independently valuable (HAL in Rust = safer interrupts, win32k in Rust = safer graphics, etc.).

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

| # | Quest | Doc | Size | Depends on |
|---|-------|-----|------|-----------|
| 10a | **USB HID input** (the beachhead; also fixes arm64's monolith miniport) | [`../engineering/input-architecture-and-unification.md`](../engineering/input-architecture-and-unification.md) | M | USB host stack enumerating |
| 10b | **USB mass storage / real storage** (kill the arm64 RAM disk) | [`../engineering/usb-mass-storage-real-storage-quest.md`](../engineering/usb-mass-storage-real-storage-quest.md) | M | same USB host stack as 10a — one probe gates both |
| 10c | **arm64 GIC interrupts** (real routing + dispatch + hardware-backed IRQL) — foundational; ends polling everywhere | *(doc TBD — after USB HID)* | L | timer path in `hal-rust/arm64/clock.rs` is a working template |
| 10d | **Real NIC** (replace virtio-net; interrupt-driven miniport → netstack) | [`../engineering/real-nic-networking-quest.md`](../engineering/real-nic-networking-quest.md) | M–L | amd64 e1000 sooner (real PIC/APIC); arm64 downstream of 10c |

Not trash, leave alone: the GOP linear-framebuffer display driver (`arm64vid`, shared all
arches) and the PL011 UART (a real ARM peripheral).

---

## Priority Order (suggested)

| # | Item | Size | Blocked By | Impact |
|---|------|------|-----------|--------|
| 1 | Testing infrastructure (4a-c) | M | Nothing | Enables everything else safely |
| 2 | SEH idiomation Stage 3 (3) | L | Nothing | Correctness + code quality |
| 3 | Pipeline auto-repair (2b) | M | Nothing | Translation velocity |
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
| 15 | Wine DLL re-implementation (7b) | XXL | Pipeline + Rosetta | Full Rust userland |
| 16 | Linux-origin subsystem refresh (7c) | XL | Pipeline maturity | Remove stale ports |
