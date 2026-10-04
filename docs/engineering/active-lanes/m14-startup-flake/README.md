# M14 WoW64/FEX startup flake — recovered work, NOT applied

**These patches are preserved, not applied, and must not be applied blindly.** One of them
modifies `vendor/fex`, which is **vendored and must stay unmodified** (AGENTS.md). They are
here because the analysis in them is worth more than the diffs, and because the worktree that
held them (`vibeos-fex/.claude/worktrees/fex-m4`, branch `fex-arm-m4`, HEAD `87ee3f7fa09`,
last committed 2026-07-17) was removed to unblock the host-wide build policy. Its branch
survives and was fully merged into master; only these uncommitted edits were at risk.

## Why this matters now

This is a July root-cause investigation into the same failure family the arm64 WoW64 lane is
working in 2026-09, and it arrived at the mechanism **independently and earlier**. See the
roadmap entry "arm64 WoW64 guests fault INSIDE FEX's code cache".

The claim, from `02-wow64-force-resident.patch`'s own comment:

> FEX's block compiler reads the guest code at a call target INSIDE a signal-deferring section
> (FEXCore `Core.cpp` `CompileBlock` → `GuardSignalDeferringSection`); if that target page is
> **not yet resident** the compile yields an empty block whose entrypoint is 0, and
> `ExitFunctionLink` (`JIT.cpp`) branches straight to it.

with a measured repro: **~3 of 6 guests died under a 6-VM soak**, victims varying
(gui32/file32/hello32/doom) "because it hits whichever guest is mid-actctx when starved". Every
WoW64 process's `LdrInitializeThunk` runs `RtlCreateActivationContext`, which calls `__alloca` /
`get_manifest_in_module` on ntdll32 pages *other* than the caller's — so the demand-fault of the
target page loses the race under host contention.

**This converges with the 2026-09-13 control experiment.** A 505,344-byte guest with all of Lua
linked but `main()` returning immediately faulted identically to the real guest, at
`LookupCache::FindBlock` / `AddBlockMapping`, while all nine parity guests passed. That falsified
"volume of translated code" and left image/address-range — and the July analysis says why the
image is what matters: **it is about PAGES MAPPED AND RESIDENT, not code executed.** A larger
image spans more pages, so more of them can be non-resident when `CompileBlock` reads them. The
never-executed control has the same page count as the real guest, which is exactly why it failed
the same way.

It also independently states the Linux-vs-Windows assumption the 2026-09 leads reached from the
other direction: *"Upstream FEX assumes CompileBlock never fails for a mapped RIP (true on Linux,
where the frontend read faults the page in); the Windows/WOW64 compile path can observe the
transient."*

## The three patches

| file | touches | what it does |
|---|---|---|
| `01-fexcore-jit-null-block.patch` | **`vendor/fex` (VENDORED)** | `ExitFunctionLink` returns `DispatcherLoopTop` instead of a null host block, so a failed compile re-enters the dispatcher and recompiles rather than `br 0` → host PC=0 → process killed |
| `02-wow64-force-resident.patch` | `dll/win32/wow64/wow64.c` | `NtLockVirtualMemory` over a freshly-mapped guest image, outside FEX's deferred section, to make pages resident before the guest runs |
| `03-kernel-guest-context-dump.patch` | `ntoskrnl-rust` arm64 `stubs.rs` | on a fatal fault in a JIT block the arm64 TrapFrame holds HOST state and says nothing about the GUEST address; this digs the guest x86 CONTEXT out of the WoW64 CPU area (`TEB64.TlsSlots[WOW64_TLS_CPURESERVED=1]` @ `TEB+0x1480` → CpuArea, guest `WOW64_CONTEXT` at CpuArea+16; i386 `Eip@+0xB8`, `Esp@+0xC4`) so a PC=0 fault can name the guest address whose block failed to compile |

`soak.sh` is the 6-VM soak harness the measurement came from.

## Do they still apply? (re-measured 2026-10-03, `git apply --check`)

| patch | against current source |
|---|---|
| `01-fexcore-jit-null-block` | **applies cleanly** — vendored FEX has not moved under it |
| `02-wow64-force-resident` | **does NOT apply** — `wow64.c` has drifted (the CSR/LPC work landed since) |
| `03-kernel-guest-context-dump` | **applies cleanly** |

Re-check before relying on this table; it is a snapshot, and 01 applying cleanly is a statement
about the vendored tree not having moved, not a licence to apply it.

**The vendored tree MOVED, and the table silently lied for a week.** `external/fex` became
`vendor/fex` in the 2026-09-26..10-03 relocation, so `01`'s diff headers addressed a path git no
longer knew; `git apply --check` failed on a missing file while the table above still said
"applies cleanly". Nothing warned, because a stale path and a drifted hunk fail the same way and
only the second one is interesting. Patch 01's headers are rebased to `vendor/fex` as of this
re-measure, and the verdicts above are the post-rebase ones. **An applicability table is a
measurement with an expiry date; re-run the check, do not read the row.**

## Before reusing any of it

- **Patch 1 cannot land as-is.** Its own comment says "Local fix, not for upstream
  (`vendor/fex` is vendored)" — which is presumably why it sat uncommitted for two months. If
  the diagnosis holds, the landable forms are (a) fix the residency on OUR side so `CompileBlock`
  never sees a non-resident page (patch 2's approach), or (b) upstream it to FEX-Emu. Not a
  quiet local edit to a vendored tree.
- **Patches 2 and 3 touch our own code and are landable in principle**, but they are two months
  old, unbuilt since, and predate a lot of movement in `wow64.c`. Treat them as a design, not a
  diff — expect to rewrite against current source.
- **None of this has been re-verified on the current box.** The artifacts, and the machine, died
  in the 2026-09-13 disk failure. The measured 3/6 rate is a July number.


## BOTH halves of the July mechanism are unsupported in the build we ship (checked 2026-10-03)

Preserved here as evidence, but **do not build on this analysis without re-deriving it.** It
was questioned on the grounds that "host contention" is doing unearned work in it — a guest's
page residency is decided inside the VM, and host load reaches it only through narrow channels
(timer-interrupt placement, host memory pressure), none of which obviously makes a *specific*
page non-resident. Following that up falsified not the contention framing but the two concrete
mechanisms underneath it:

**1. "inside a signal-deferring section" — a NO-OP on Windows.**
`FEXCore/include/FEXCore/Utils/SignalScopeGuards.h` splits at `#ifndef _WIN32` (line 18) with
`#else` at 234 and `#endif` at 269. **We compile the 234-269 block**, where

        template<template<typename> class LockType = std::unique_lock, typename MutexType>
        static auto GuardSignalDeferringSection(MutexType& mutex, InternalThreadState*, uint64_t = ~0ULL) {
          return LockType<MutexType> {mutex};     // that is the whole body
        }

is a bare lock with **no deferred-signal refcount**. The POSIX branch (line 211) wraps a
`DeferredSignalRefCountGuard`; the Windows branch does not — Windows has no POSIX signals, FEX
uses a VEH. So `CompileBlock`'s `GuardSignalDeferringSection(CodeInvalidationMutex, Thread)`
(`Core.cpp:852`, `:954`) defers nothing, and a page fault taken inside it is delivered to the
VEH like any other. That is consistent with what we now measure: 539 access violations inside
`LookupCache` across a 10-boot farm, every one recovered.

**2. "the pager evicted the ntdll32 code pages" — we have no pager that does that.**
`MmWorkingSetManager` is fully implemented (`ARM3/wslist.rs:504`, it really does trim under
`MmAvailablePages < MmMinimumFreePages`) and has **no live caller**. The balance-set manager's
two calls are commented out — `ke/balmgr.rs:404,415`, `// MmWorkingSetManager();` under
`// NO WS Management Yet!`, inherited from upstream ReactOS — and `git log -S` over those lines
shows them only ever touched by the Rust translation. Nothing in this kernel trims a working
set, so pages cannot be evicted between the patch's prefault and `RtlCreateActivationContext`.

**What survives.** The *empirical* numbers may still be real — 3/6 guests dying, improving to
1/6 with a one-time touch and to 0 with `NtLockVirtualMemory` — and if prefaulting genuinely
helped then something about a non-resident page genuinely was fatal. But that tree is gone
(2026-09-13 disk failure), the numbers have never been re-derived, and the *explanation*
attached to them is now known to name two things that do not operate here. Treat the patches as
an untested empirical lead and the prose as refuted.

**The lesson, which is the reusable part.** This analysis was preserved, cross-linked from the
roadmap, and cited as independent corroboration of a 2026-09 reading — on the strength of its
confident mechanism prose, which nobody checked against the `#ifdef` it depends on. Two greps
settled it. A second source agreeing with you is worth exactly what its evidence is worth, and
"arrived at independently and two months earlier" is not evidence.
