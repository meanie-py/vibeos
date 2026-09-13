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

## Why a 64-bit guest is structurally CHEAPER than the 32-bit one we already ship

This is the part that is easy to get backwards, so state it plainly: **x86-64 Windows and ARM64
Windows are both LLP64.** Same 8-byte pointers, same `long` = 32 bits, same struct alignment
rules. A 64-bit x86 guest running on a 64-bit ARM host is not a *bitness* transition at all.

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

What remains is genuinely smaller and uniform:

- **Register-ABI mapping.** Win64 x86-64 passes args in `rcx, rdx, r8, r9` + stack; ARM64 uses
  `x0-x7`. This is a mechanical, *per-arity* shuffle, not a per-struct rewrite — one generic
  dispatcher, not 13,678 lines.
- **Guest context save/restore** in whatever format we choose to define.
- **Memory/invalidation notifications** — the `BTCpuNotifyMemory*` family, which we already
  implement for the 32-bit plugin and which is bitness-independent.
- **Exception dispatch** from guest faults back into guest-visible SEH.

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
  decision that nobody has made.
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
