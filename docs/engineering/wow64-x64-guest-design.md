# Running x86-64 guests on arm64: ARM64EC vs. our own BTCpu64 contract

**Status: DECIDED 2026-10-05 — Fork B first. M1 DONE 2026-10-05; M2 DONE 2026-10-06 — the x86-64
ladder is at parity with the 32-bit one (see "Status" below).** Written 2026-09-13 at the
owner's request as scoping, after an audit established that FEX's x64 support is real and every
blocker is on our side (`docs/product/vibeos-roadmap.md`, and the audit summary below).

**The decision (owner, 2026-10-05): build Fork B (our own BTCpu64-shaped contract) first.** Most
of what B needs, A needs too — a 64-bit FEX plugin, x64 CONTEXT conversion, the guest
exception and syscall path — and B runs x64 apps on the amd64 system DLLs we already build,
fully emulated, the same shape as today's 32-bit path. Fork A (ARM64EC) stays the later
speed-up: it runs system DLL code natively, but only once our userland is compiled ARM64EC /
ARM64X. (Not "ARM64EC is opt-in on Windows": on Windows 11 ARM every x64 app runs in an
ARM64EC process. What is opt-in there is a developer porting their OWN code to ARM64EC.)
First milestone: a `hello64` guest that enters FEX in 64-bit mode and makes one syscall.

The question this answers is not "can FEX run x64" — it can, and does, and the recompiler is
already compiled into the `libwow64fex.dll` we ship today. The question is **which OS-side
contract we implement to reach it**, because we have inherited Microsoft's answer without ever
deciding to.

---

## Status — what exists, and what is owed

**M2 (2026-10-06): parity with x86-on-arm64.** The 32-bit ladder's ten guest sources, rebuilt
for x86-64 (`gui64 win64 file64 async64 marshal64 fault64 thread64 selfmod64 reg64 spawn64`,
plus M1's `hello64`), all exit `0x600D` under FEX on arm64 AND natively on amd64, with the
32-bit ladder green on the same boots: arm64 `/tmp/vibeos-cli/20261006-020111-v3xu401s-smoke-arm64`
(24 guests at 0x600D, gui32 at its graded 0x600E), amd64 run dirs in the commit messages. They
run through the guest's OWN amd64 ntdll, loader, kernel32/user32/gdi32/advapi32/ws2_32 — real
Win32, not raw syscalls — so this covers: process init, the import closure, file and overlapped
I/O, IOCP, the registry (incl. the 64-bit view), win32k drawing, window creation with
callbacks into the guest and WM_PAINT, guest SEH through `.pdata` (hardware fault, software
raise, debug print, Get/SetThreadContext), threads (TLS, per-thread SEH, 4 concurrent), runtime
code generation incl. kernel-side writes into code, and process creation incl.
CreateRemoteThread into a running x86-64 child.

They are graded where the media carries them (`tests/qemu/cli/ladder.py` `STAGED_LADDER`): they
need build-amd64 to exist at all, so an arm64-only tree does not owe them. `hello64` (no
imports) stays in the always-owed `LADDER`.

| piece | where | what |
|---|---|---|
| contract (v2) | `sdk/include/reactos/wowx64/btcpu64.h` (+ `wowx64-rust/src/contract.rs`) | BTCpu-shaped verbs, x86-64 `WOWX64_CONTEXT`, a host-services table. v2: the CPU layer passes the 4 register args + guest rsp; the OS side reads exactly each service's stack args (v1 copied a fixed 11 words and read off the top of a fresh thread's stack) |
| CPU layer | `tools/fex/wowx64fex/` -> `libwowx64fex.dll` | upstream `WOW64/Module.cpp`'s shape; adds the ReadFile/MemoryDirty notifications for kernel-side writes |
| OS layer | `sdk/lib/rust/wowx64-rust` -> `wowx64.dll` | `host.rs` entry + guest-entry path; `image.rs` maps/rebases the guest ntdll; `syscall.rs` the forwarder and its exceptions; `except.rs` guest exception delivery; `callback.rs` win32k callbacks; `thread.rs` thread creation + stack ownership |
| guest DLL set | `qemu/mkramdisk.sh` -> `<SystemRoot>\sysamd64` | build-amd64's full import closure of a stock GUI app (`wow64-closure.py --include-prestaged`) + `lpk.dll` |
| native ntdll | `ntdll-rust` `LdrpLdrData`, `dispatch/mod.rs` `WowX64UserCallbackHook` | the loader reads its OWN module list/lock (two ntdlls share one PEB); a callback hook slot |
| guests | `tools/fex/hello-compute/*32.c` built twice by `build-hello-compute.sh` | the width-specific parts (`fs:[0]` SEH vs `.pdata`, WoW64-view expectations) are `#ifdef _WIN64`; the i386 builds are byte-identical to before |

**The design, in the places it is not obvious:**

- **Shared PEB and TEB.** The guest's TEB is the thread's TEB; its PEB is the process's. That is
  what makes every kernel-written per-thread field (win32k's `Win32ClientInfo`, last-error) just
  work, and what WoW64 cannot have. The cost is a short list of things each side must keep to
  itself, each found the hard way: the loader's module list and lock (native unwinding looked up
  FEX's `.pdata` in the guest's list), the TEB stack bounds (below), the `InDbgPrint` flag (a
  guest DbgPrint silenced every native trace from the VEH), and the heap block layout (below).
- **One heap format for every 64-bit target.** Both ntdlls reach the process heap through
  `PEB->ProcessHeap`, so their heap code must agree on what a block is. rtl-heap's aarch64
  `HEAP_ENTRY` had its own field order that heap.h never had; it now uses heap.h's `_WIN64`
  layout, as x86_64 always did.
- **TEB stack bounds describe the GUEST stack.** The guest's amd64 `RtlDispatchException` rejects
  any frame outside them; the native stack the CPU layer runs on is fully committed instead,
  so it never needs guard-page growth (which checks those bounds).
- **The system32 redirect is conditional.** A name that exists in `sysamd64` is redirected there;
  any other name stays in system32 (the guest's own exe, data files) — unless system32 holds an
  image for another machine there, which the guest gets as "not found", as WoW64 would give it.
  (Unconditional, `spawn64` could not relaunch itself; with no foreign-image check, gdi32's
  `LoadLibrary("lpk.dll")` mapped the arm64 one.)
- **Threads** (`NtCreateThread`): the guest CONTEXT is parked in the target and the kernel gets a
  native one starting `WowX64GuestThreadStart`; that works into a running x86-64 child too
  (wowx64.dll sits at the same base in every guest process). The initial thread of a NEW x86-64
  process needs no context at all — its native loader diverts to wowx64.

**Owed:**

1. **User-mode APCs into a guest thread** (`NtQueueApcThread` with a guest routine) — the native
   `KiUserApcDispatcher` would call x86-64 code. Nothing in the ladder queues one.
2. **`NtCreateThreadEx`** is not in our service table, so a guest that calls it gets
   `STATUS_INVALID_SYSTEM_SERVICE`; it needs the same treatment as `NtCreateThread` when it lands.
3. **Thread creation into a non-x86-64 process** (an x86-64 guest starting a NATIVE arm64 or
   32-bit child) is refused with `STATUS_NOT_SUPPORTED`: the guest's CONTEXT names a start
   routine for the wrong machine, and we cannot name the right one from here.
4. **Per-thread CPU-layer state on thread exit** (FEX's `Threads` map, the parked frames) is
   not freed.
5. **A native kernel32 still loads** in an x86-64 guest process (something in the CPU layer's
   init LoadLibrary's it; libwowx64fex.dll's own imports are ntdll alone). Harmless today; worth
   knowing what wants it.

## The audit result, in one paragraph

Vendored FEX defines two Windows targets: `wow64fex` (32-bit guests, the one we build) and
`arm64ecfex` (64-bit guests, which we do not). They are mutually exclusive by construction —
`Source/Windows/CMakeLists.txt:26` is `if (ARCHITECTURE_arm64ec) ... elseif (ARCHITECTURE_arm64)
...` — and `tools/fex/build-fex.sh` passes an `aarch64-w64-mingw32` triple, which selects the
32-bit one. FEXCore itself is bitness-agnostic at compile time: `Is64BitMode` is
`FEX_CONFIG_OPT(Is64BitMode, IS64BIT_MODE)` (`FEXCore/Source/Interface/Context/Context.h:334`), a
**runtime** option, with no `#ifdef` gating the 64-bit paths in `Core.cpp`. Nothing upstream
restricts us.

## The knob, and it really is one line

`ARM64EC/Module.cpp:589`:

```cpp
FEXCore::Config::Set(FEXCore::Config::CONFIG_IS64BIT_MODE, "1");
```

`WOW64/Module.cpp` simply never sets it, so it defaults to 32-bit. Placed side by side, the two
modules' core-init sequences are otherwise near-identical; they differ in exactly four places:

| | WOW64 | ARM64EC |
|---|---|---|
| bitness | (unset → 32-bit) | `CONFIG_IS64BIT_MODE = "1"` |
| host features | `HostTypeEnum::Wow64` | `HostTypeEnum::Arm64ec` |
| syscall handler | `WowSyscallHandler` | `Exception::ECSyscallHandler` |
| handler config | `Context::HandlerConfig` | `Exception::HandlerConfig` |

**So the recompiler half of "run x64 guests" is a config flag and a syscall handler.** Everything
expensive about ARM64EC lives *outside* this init, in the ABI plumbing.

## Why the MARSHALLER side gets cheaper — and why that is only half the ledger

**Read this section and the next one together. Quoting either alone decides the fork by
itself, and neither is the whole comparison.**

**x86-64 Windows and ARM64 Windows are both LLP64.** Same 8-byte pointers, same `long` = 32
bits, same struct alignment rules. So a 64-bit x86 guest on a 64-bit ARM host has no *width*
transition.

It does still have an **arch** transition. That transition does not disappear; it MOVES, out of
the marshaller and entirely into the CPU layer — see "What moves onto the CPU-layer side" below.
The correct claim is that **the WIDTH class closes**, not that the bug class closes. The parent
class is "a structure whose layout depends on the guest's ARCH", and width is only one way a
layout can so depend.

WoW64 is hard because it is 32-on-64: every pointer, handle and struct crossing the boundary has
to be narrowed or widened, and every one of those is a chance to get a field offset wrong. That
is what the last several weeks of this lane were. The machinery it took:

| | lines |
|---|---|
| `dll/win32/wow64/wow64_manual.c` (hand-written marshallers) | 6,552 |
| `wow64_thunks.gen.c` (generated Nt thunks) | 1,784 |
| `wow64_win32k_thunks.gen.c` (generated win32k thunks) | 5,342 |
| **total width-thunking surface** | **13,678** |

**A 64-bit guest needs none of it.** Arguments are already the right width; structures are
already the right shape; `PORT_MESSAGE`, the CSR capture buffer, `BASE_STATIC_SERVER_DATA` and
every other layout that cost this lane a boot apiece are byte-identical between guest and host.
The five width bugs that took the ladder from 0/9 to 9/9 are, for a 64-bit guest, not bugs that
can exist.

**A second class closes too, and it is not width either: address-space CONFINEMENT.** A 32-bit
guest must be kept inside the low 4GB, so the marshaller forces
`ZeroBits = WOW64_ZEROBITS_BELOW_2G` on `NtAllocateVirtualMemory` and `NtMapViewOfSection`,
clamps to `WOW64_USER_CEILING32`, and checks that csrss's shared section, `StaticServerData` and
heap pointers actually landed below 4GB — one of which asserted and dereferenced NULL when they
did not. Measured in `wow64_manual.c`: **14 distinct guarded confinement sites** — 11 `if` tests on a
confinement condition (`ZeroBits == 0`, `Max > WOW64_USER_CEILING32`, `>> 32` guest-addressability
checks, the `> 0xFFFFFFFFULL` client-info test) plus 3 assignments forcing a ceiling. A 64-bit
guest has the host's own address space and none of it applies.

The number took three tries across two lanes, and the history is worth one line because the
failure mode is generic: the research lane (who spotted the class — it is theirs) first reported
36 from `grep -c "4GB\|ZeroBits"`, of which 22 hits are comments and `DbgPrint` strings; I then
reported ~25 textual references, which was the same mistake with a wider pattern, and 4
"enforcement statements", which was too NARROW because it counted only the forced assignments and
missed the 11 guarded tests. **All three numbers were produced by grepping for a token instead of
deciding what a "site" is.** 14 is what you get once the method is stated first, and it is
verified here against the file.

What remains on the marshaller side is genuinely smaller and uniform:

- **Register-ABI mapping.** Win64 x86-64 passes args in `rcx, rdx, r8, r9` + stack; ARM64 uses
  `x0-x7`. A mechanical, *per-arity* shuffle, not a per-struct rewrite.
- **Memory/invalidation notifications** — the `BTCpuNotifyMemory*` family, which we already
  implement for the 32-bit plugin and which is bitness-independent.

## What MOVES onto the CPU-layer side — the other half of the ledger

The marshaller gets to be pure width work only because it already **delegates** everything
arch-shaped. Verified in our own tree: `wow64.c` resolves and calls `BTCpuGetContext` /
`BTCpuSetContext` (`wow64.c:294-297`, `763-764`) rather than thunking `CONTEXT` itself, because
a guest `CONTEXT` is arch-shaped, not width-shaped. `NtGetContextThread`, `NtSetContextThread`,
`NtContinue` and the exception path all take that route.

So a 64-bit guest does not make that work vanish — it inherits it, unreduced and in a harder
form:

- **CONTEXT conversion.** Someone must still produce an x86-64 `CONTEXT` from an AArch64 one and
  back. x64's context is materially larger and more complex than i386's, so this gets *bigger*,
  not smaller.
- **Exception ENTRY is a different mechanism, not a narrower one.** i386 guests use the `fs:[0]`
  SEH chain that the 32-bit dispatcher walks. x64 guests use table-based `.pdata` unwind.
  Neither is saved by LLP64, and the second is not a simplification of the first.

**The load-bearing point for choosing a fork: the saving and the new cost land on OPPOSITE SIDES
of the BTCpu contract.** ~13,678 lines plus the confinement class come OFF the marshaller side;
an unpriced amount goes ON to the CPU-layer side — the same side whose "plugin in our tree
linking vendored FEXCore" option is flagged below as not yet priced. **The comparison must be
made across the contract, not on one side of it.**

This does not weaken Fork B: the saving is large and real, and **Fork A needs the same CONTEXT
and unwind work anyway**, on top of the hybrid-PE ABI. It means the CONTEXT conversion and
exception-entry mechanism must be priced WITH the plugin, not treated as extras — they are
precisely the part the marshaller structurally refuses to do.

## The two forks

### Fork A — implement ARM64EC (Microsoft's shape)

Build `arm64ecfex` and give it the OS contract it expects. From `ARM64EC/Module.cpp` and
`libarm64ecfex.def`, that means our loader, ntdll and exception path must grow:

- `CHPE_V2` / `IMAGE_ARM64EC_METADATA` / `CHPEMetadataPointer` — compiled-hybrid PE metadata the
  **loader** must parse and honour
- `ARM64EC_REDIRECTION_ENTRY` tables
- `__os_arm64x_dispatch_call`, `__os_arm64x_dispatch_icall`, `__os_arm64x_dispatch_icall_cfg` —
  the ARM64X dispatch vector **ntdll must export and populate**
- `ARM64EC_NT_CONTEXT` — a distinct context format, so context conversion *and* exception
  dispatch both become EC-aware
- the `DispatchJump` / `RetToEntryThunk` / `ExitToX64` / `BeginSimulation` DATA-export protocol

Plus the two cheap prerequisites: a second FEX configure with an arm64ec triple, and an
llvm-mingw pin bump (our pinned toolchain has `aarch64-`, `armv7-`, `i686-`, `x86_64-` and
`generic-w64-mingw32`, and **no** `arm64ec-w64-mingw32`).

**Cost:** large. Hybrid-PE loading and an EC-aware exception path are each a real subsystem.
**Benefit:** it is the shape real Windows uses, so real ARM64EC binaries would run, and we would
inherit upstream's maintained plugin as-is.

### Fork B — define our own 64-bit guest contract

We are the OS. Nothing obliges us to implement Microsoft's hybrid-PE ABI to run an x86-64 binary;
that ABI exists to let x64 and native ARM64 code interoperate *within one process*, which is a
goal we do not have. A plain "64-bit guest in its own process, syscalls thunked out" model is the
one we already have working for 32-bit — and it is the model the 13,678-line table above says is
*easier* at 64 bits, not harder.

Note that upstream is already partly thinking this way: `libarm64ecfex.def` exports
`BTCpu64FlushInstructionCache`, `BTCpu64IsProcessorFeaturePresent`, `BTCpu64NotifyMemoryDirty`
and `BTCpu64NotifyReadFile` — BTCpu-shaped names, alongside the EC-shaped DATA exports.

**The blocking constraint, and it is the thing to resolve first:** `vendor/fex` is VENDORED and
must stay unmodified (AGENTS.md). Fork B needs a Windows plugin that is neither `wow64fex` nor
`arm64ecfex`, so it cannot simply be a patch to `Source/Windows/`. Three sub-options, in
increasing order of how much they cost us:

1. **A new plugin in OUR tree that links vendored FEXCore as a library.** FEXCore is already
   consumed exactly this way by both existing plugins, so this modifies nothing vendored. This is
   the option to price first.
2. **Upstream a third plugin to FEX-Emu.** Cleanest long-term, slowest, and puts us on someone
   else's schedule.
3. **Carry a patch series against `vendor/fex`.** Violates the vendoring rule as written; would
   need an explicit owner decision to change that rule, not a quiet exception.

**Cost:** unknown until sub-option 1 is priced — that is the single highest-value next
measurement in this document. **Benefit:** no hybrid-PE loader, no ARM64X dispatch vector, no EC
context format; reuses the BTCpu-shaped wiring in `dll/win32/wow64/wow64.c` that we have already
debugged.

## First-pass pricing of Fork B's plugin option (including CONTEXT, as it must be)

Asked for after the correction above, on the principle that pricing the plugin without CONTEXT
conversion and exception entry understates Fork B exactly as quoting 13,678 alone overstates it.
This is a FIRST PASS from reading upstream, not an estimate anybody should schedule against.

**Shape of the thing to build.** The two existing Windows plugins are ~1,050 lines each
(`WOW64/Module.cpp` 1,066; `ARM64EC/Module.cpp` 1,041 + 155 of `Module.S`) over ~1,290 lines of
shared `Source/Windows/Common/` (allocator hooks, CPU features, image and invalidation trackers,
logging). So a third plugin is a ~1,000-line module that reuses Common — not a subsystem.

**The CONTEXT conversion is already written, and is NOT ARM64EC-specific.** This is the
load-bearing finding of the pricing pass, and it is checkable in one function:
`ARM64EC/Module.cpp:317` `LoadStateFromECContext` maps a standard x64 `CONTEXT` straight into
FEXCore state — `State.gregs[REG_RAX] = Context.Rax;` and so on. There is nothing hybrid-PE
about it. It is "x86-64 `CONTEXT` <-> FEXCore state", which is precisely what ANY x64 guest
contract needs, EC or not. What is EC-specific is how the context is *delivered* (the
`ARM64EC_NT_CONTEXT` overlay on the native frame), not the mapping itself.

Measured surface in the x64 plugin:

| piece | lines | reusable outside ARM64EC? |
|---|---|---|
| state <-> x64 `CONTEXT` (`LoadStateFromECContext`, `ReconstructThreadState`) | 162 | **yes, in substance** — plain register mapping |
| exception path (`RethrowGuestException`, `ResetToConsistentState*`) | 267 | partly — entangled with EC delivery and `ARM64_NT_CONTEXT` |
| *(for contrast)* the 32-bit equivalents we already drive today | 97 | — |

So the CONTEXT half of "the unpriced cost on the CPU-layer side" is ~162 lines whose logic
already exists upstream and is not EC-shaped; the exception half is ~267 lines that genuinely is
entangled and is the part to scope properly. **That is materially cheaper than "someone still
has to produce an x86-64 CONTEXT from an AArch64 one" sounds** — that someone already did, for
the x64 case, in the plugin we are not building.

### The category question is ANSWERED: yes, with `vendor/fex` untouched

This was the one yes/no that could have taken Fork B off the table. It does not. **Measured
2026-09-13** by building a probe module outside the vendored tree:

    forkbfex.dll   4,320,768 bytes
    exports:       ForkBProbeInit
    FEXCore symbols linked in: 3,715
    git status --porcelain vendor/fex:  (empty -- the build DIRTIED NOTHING)

The probe calls the same entry points both real plugins use —
`FEXCore::Config::Set(CONFIG_IS64BIT_MODE, "1")`, `Context::CreateNewContext(HostFeatures)` —
compiled with our own `aarch64-w64-mingw32-clang++` and linked against the static libraries a
standalone `tools/fex/build-fex.sh` already produces (`libFEXCore.a`, `libFEXCore_Base.a`,
`libCommonWindows.a`, `libCommonWindowsRuntime.a`, `libCommon.a`, `libCommonTools.a`,
`libwow64_ex.a`, `libntdll_ex.a`, plus the External deps and compiler-rt builtins). 3,715 FEXCore
symbols is the real recompiler, not an empty shell.

**Correction 2026-10-03 — that `git status` line proved less than it was read as proving.** It
shows the build left no *uncommitted* change, which is the relevant fact here and the conclusion
above stands: a Fork B probe links the real recompiler without the build touching the vendored
tree. But it is NOT evidence that the tree matches upstream, and it was later cited that way. It
does not: `6afc128553f` (2026-07-14) carries a committed NULL-thread guard in
`Source/Windows/WOW64/Module.cpp`, and `git status` is clean for a committed patch. The oracle
for divergence from upstream is `vendor-patches/fex.md` and `git log -- 'vendor/fex/*'`; the
record itself claimed "Unmodified" for nearly three months and is now fixed.

**Four practical constraints found while doing it**, all surmountable, all worth knowing before
anyone wires this into our CMake for real:

1. **Do NOT embed FEX with `add_subdirectory`.** 25 references to `${CMAKE_SOURCE_DIR}` inside
   the vendored tree would rebind to OUR project root and break. The working arrangement is the
   one we already have: configure and build FEX standalone, then link its artifacts.
2. **FEX exports nothing for consumers** — no `install(TARGETS)` for FEXCore, no `export()`. So
   there is no `find_package` route; paths to the build tree are explicit and ours to maintain.
3. **FEXCore's public headers need GENERATED headers**, so a consumer needs the FEX *build* tree
   and not just the source: `<build>/include/FEXCore/Config/ConfigValues.inl` and
   `ConfigOptions.inl`. Our build therefore depends on a FEX build having run — which is already
   true today for staging `libwow64fex.dll`, so this adds no new ordering constraint.
4. **`$<TARGET_OBJECTS:FEXCore_object>` cannot cross projects.** The real plugins link FEXCore as
   an object library; from outside, link `libFEXCore.a` instead, and wrap the archives in
   `-Wl,--start-group ... --end-group` — without the group, static-archive ordering leaves
   `strlen`/`memcpy`/`memmove` undefined even though FEX's own CRT provides them.

**What this does NOT establish**: that such a plugin WORKS. It links and it contains the
recompiler; it has never been loaded, entered, or run a single guest instruction. The experiment
was scoped to the category question — can this exist without touching vendored FEX — and that is
all it answers. The exception path (267 lines, EC-entangled) remains the real unpriced work.

## What is NOT established here

Listed because a scoping doc that reads as confident is worse than useless:

- **Whether `BTCpuSimulate`'s entry protocol generalises to a 64-bit guest.** Our
  `wow64.c` drives it through 32-bit-shaped context structures. Fork B needs a context format
  decision that nobody has made — and per the section above, that decision is not a detail, it
  is where the arch transition now lives.
- **Whether `Is64BitMode` alone is sufficient at the FEXCore level.** It is clearly *the* knob,
  and the table above shows only four init differences — but "four differences" is not the same
  as "no other runtime assumptions", and the syscall-handler difference in particular hides
  unmeasured work.
- **Sub-option B1's real cost.** First-pass priced above for the CONTEXT and exception surface.
  The UNPRICED part is now narrower and sharper: whether our CMake can link vendored FEXCore and
  reuse `Source/Windows/Common/` without modifying `vendor/fex`. Nobody has tried it, and it is
  the one question that could change Fork B's cost by category rather than degree.
- **Anything about performance.** A 64-bit guest recompiles more code. (This bullet used to
  say the 32-bit path faults inside FEX's code cache on images >500K and must be fixed first.
  That bug was RETRACTED 2026-10-03 — a harness drain race plus FEX's by-design overcommit
  faults misread as crashes; see the roadmap. The 32-bit ladder is 11/11 on QEMU and on a
  real Pi 4.)
## Sequencing

The two cheap items — the arm64ec-triple configure and the llvm-mingw pin bump — serve **Fork A
only**. Do not spend on them until the contract question is settled, or they become sunk cost
arguing for a fork nobody chose.
