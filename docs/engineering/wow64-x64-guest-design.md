# Running x86-64 guests on arm64: ARM64EC vs. our own BTCpu64 contract

**Status: SCOPING ONLY. Nothing here is built, and this deliberately stops short of
recommending which fork to take.** Written 2026-09-13 at the owner's request, after an audit
established that FEX's x64 support is real and every blocker is on our side
(`docs/product/vibeos-roadmap.md`, and the audit summary below).

The question this answers is not "can FEX run x64" — it can, and does, and the recompiler is
already compiled into the `libwow64fex.dll` we ship today. The question is **which OS-side
contract we implement to reach it**, because we have inherited Microsoft's answer without ever
deciding to.

---

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
did not. Measured in `wow64_manual.c`: **4 enforcement statements** (2 forced-`ZeroBits`, 2
ceiling clamps) against **~25 references** across code and the comments explaining them. A
64-bit guest has the host's own address space and none of this applies. (Credit to the research
lane for spotting this class; they put it at 36 sites, I count ~25 references / 4 enforcement
statements — same substantive point, and I am quoting the number I can defend from the file.)

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

**The blocking constraint, and it is the thing to resolve first:** `external/fex` is VENDORED and
must stay unmodified (AGENTS.md). Fork B needs a Windows plugin that is neither `wow64fex` nor
`arm64ecfex`, so it cannot simply be a patch to `Source/Windows/`. Three sub-options, in
increasing order of how much they cost us:

1. **A new plugin in OUR tree that links vendored FEXCore as a library.** FEXCore is already
   consumed exactly this way by both existing plugins, so this modifies nothing vendored. This is
   the option to price first.
2. **Upstream a third plugin to FEX-Emu.** Cleanest long-term, slowest, and puts us on someone
   else's schedule.
3. **Carry a patch series against `external/fex`.** Violates the vendoring rule as written; would
   need an explicit owner decision to change that rule, not a quiet exception.

**Cost:** unknown until sub-option 1 is priced — that is the single highest-value next
measurement in this document. **Benefit:** no hybrid-PE loader, no ARM64X dispatch vector, no EC
context format; reuses the BTCpu-shaped wiring in `dll/win32/wow64/wow64.c` that we have already
debugged.

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
- **Sub-option B1's real cost.** Asserted as plausible from the fact that FEXCore is already
  linked as a library twice. Not priced.
- **Anything about performance.** A 64-bit guest recompiles more code, and our current 32-bit
  path already faults inside FEX's code cache on images >500K (roadmap: `LookupCache::FindBlock`).
  That bug is bitness-independent and would follow us to either fork. **Fix it before pricing
  either route on a benchmark.**

## Sequencing

The two cheap items — the arm64ec-triple configure and the llvm-mingw pin bump — serve **Fork A
only**. Do not spend on them until the contract question is settled, or they become sunk cost
arguing for a fork nobody chose.
