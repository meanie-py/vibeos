# Tier 2 ucrt difftesting: grade in the guest, rename with objcopy

Status: **RUNNING IN THE GUEST, and it found real bugs.** First run 2026-09-22:
3013 cases, `dll_fail=0` (the positive control), **`rust_fail=14`** — seven
msvcrt Rust bodies that are NOT UCRT-correct. See "First results" below.

Earlier status: **reference set BUILT and linking; comparison harness not written.**
`tools/ucrt_tier2_refset.py` emits `build-<arch>/libucrt_ref.a` on all three
arches, and a probe exe linking it against the shipping ucrtbase import
resolves both sides correctly (2026-09-22). What remains is the comparison
harness and gate wiring.

## The problem

`ucrt-difftest` (landed) grades our Rust against the genuine Microsoft UCRT C on
the HOST. It reaches Tier 1 only — TUs that include nothing but public UCRT
headers. Everything interesting reaches `corecrt_internal.h`, which includes
`<windows.h>` outright, and that is most of the remaining ~999 exports.

## Why the obvious fix was abandoned

The plan was a types-only shim for the Windows header chain. Two findings killed
it, and the second is decisive:

1. **The chain is generated and cascades.** `winnt.h` is not in the source tree
   at all — it is generated per-arch into `build-<arch>/sdk/include/psdk/` — and
   `windef` → `minwindef`, with `ioapiset` and `threadpoolapiset` behind them.
   That is a grind, but only a grind.

2. **`long` is the wrong width, so the answers would be wrong.** Windows `long`
   is 32 bits; the host's is 64. `labs` and `ldiv` are already excluded from
   `ucrt-difftest` for exactly this reason. Measuring the blast radius: ~188
   public UCRT declarations mention bare `long`, concentrated in the families we
   most want — `atol`, `_atol_l`, `strtol`, `_byteswap_ulong`, `__timezone`,
   `__doserrno`, `__threadid`. A host harness must either exclude all of them or
   grade them at the wrong width, silently.

A harness that cannot grade `strtol` is not a harness for this quest.

## The design

Run the same trick in the GUEST, against binaries the normal build already
produces. No header hacks, because nothing is recompiled. No ABI mismatch,
because both sides are Windows.

```
test exe = [ ucrt objects, exports renamed to ref_* ]   <- genuine Microsoft C
         + [ ucrtbase import lib ]                      <- OUR shipping DLL
```

`ref_strtol(...)` is Microsoft's body. `strtol(...)` resolves through the import
to whatever ucrtbase.dll actually ships — Rust once a family is taken over, C
before. So it grades **the shipping artifact**, which is stronger than grading
an archive.

## Why objcopy and not a rename prelude

`ucrt-difftest` renames with a generated `-include` header, which is right for
its four small TUs. At UCRT scale it breaks: a global `#define abs ref_abs`
rewrites HEADER DECLARATIONS too, and collapses C++ overloads that differ only
in return type onto one name. Verified — all four Tier-2 TUs tried failed
identically at `corecrt_math.h:525`.

`llvm-objcopy --redefine-syms` renames the compiled objects instead:

* no preprocessor involved, so no header or overload breakage;
* it operates on the objects the real build already produced, with the real
  flags for the real target;
* **it renames undefined references as well as definitions**, which is the
  property the whole design rests on.

That last point, verified on `convert/strtox.cpp.obj`:

| symbol | kind | after rename |
|---|---|---|
| `_errno` | EXPORT | `_ref__errno` |
| `__acrt_locale_changed_data` | internal | unchanged |
| `__acrt_update_locale_info_explicit` | internal | unchanged |

Exactly the required semantics. The reference set calls its own renamed copies,
so it is self-consistent and isolated from ucrtbase.dll; internals keep their
names and resolve among the privately-linked objects; and nothing collides with
the import, because ucrtbase does not export its internals.

Renaming only EXPORTS (from `ucrtbase.spec`, 2,614 on i386) rather than every
symbol is what makes that work. Rename the internals too and the private copy
still links, but it stops sharing state with itself in the ways its own code
assumes.

## What is verified, and what is not

Verified: the rename mechanism, its definition/reference semantics, and that
Tier-2 TUs (`convert/strtox`, `mbstring/mbctype`, `time/ctime`,
`misc/invalid_parameter`) rename cleanly — 32, 5, 8 and 8 definitions.

VERIFIED 2026-09-22, the risk this design rested on: **a full renamed set
links.** All 448 i386 ucrt objects rename and archive, and a probe exe linking
`libucrt_ref.a` + `libucrtbase.a` + kernel32/ntdll/vcstartup/vcruntime/
crtmath/chkstk/pseh/compiler-rt produces exactly the intended shape:

```
401021: calll 0x401080 <_ref_strtol>     ; a real body, stack frame and all
401044: calll 0x41045a <_strtol>         ; jmpl *0x49ead8 -- IAT into ucrtbase.dll
```

Two DLL imports only: `ucrtbase.dll` and `kernel32.dll`. amd64 links identically
(`ref_strtol` / `strtol` at distinct targets). No export was referenced through
asm or a table that objcopy could not reach -- the count of unresolved `ref_*`
symbols across the whole renamed set is **zero**.

Three things the build taught us that the paper design did not have:

* **Archive, not loose objects.** `llvm-ar` into one `.a` and let the link pull
  lazily: 629 KB instead of 5.4 MB, and it sidesteps `.CRT$XIC` entirely. The
  loose-object form drags in `sse2_initializer.cpp`, whose `__acrt_initialize_sse2`
  is behind `_M_HYBRID_X86_ARM64` on i386 and is really supplied by ucrtbase's
  own `stubs.c` -- which the reference side does not link.
* **The object list must come from the LINK LINE, never a glob.**
  `sdk/lib/ucrt_reactos/` replaces four `sdk/lib/ucrt/` files (matherr,
  GetStringTypeW, MultiByteToWideChar, WideCharToMultiByte) and ninja never
  deletes the objects of the copies it stopped using. Globbing `ucrt.dir` picks
  up both and the link dies on duplicate symbols. `ninja -t commands ucrtbase`
  is the authority; `compile_commands.json` independently agrees at 448.
* **Settle the decoration once per arch, never per symbol.** `tolower` and
  `_tolower` are BOTH exports, so on an undecorated arch the symbol `_tolower`
  is export `_tolower`'s body. Probing `_name` first and falling back hands it
  to export `tolower` instead -- stealing one body and leaving the other
  unrenamed, so it would resolve to the DLL and grade against itself. That cost
  9 amd64 and 9 arm64 exports before the map was fixed.

Reference surface: **1013 exports on i386 and amd64, 1010 on arm64** have a
genuine Microsoft body here to grade against. The rest of the 2,614 come from
vcruntime, crtmath, ucrtbase's stubs or our Rust.

## Three sides, not two

A two-sided harness cannot answer the reuse question, and the first guest run
showed exactly why: all 17 families it graded are still MICROSOFT, so `ref_foo`
and the DLL's `foo` were *the same Microsoft code*. 3013 cases passed and proved
nothing about us. It is a genuine positive control — the rename does not corrupt
bodies, across the full 0..255 ctype range and 23x6 strtol cases — but it is not
a verdict.

So the harness links a THIRD side: `tools/ucrt_tier2_refset.py --side msvcrt-rust`
renames the msvcrt Rust archive's ucrtbase exports to `rust_*` (935 on i386).
Each row now grades twice:

* `dll` vs `ref` — what we ship today. The control while a family is vendored C.
* `rust` vs `ref` — **is our existing Rust body UCRT-correct?** Meaningful
  whether or not the family has been taken over, and it needs no flip.

llvm-objcopy renames a static archive in place, so this side costs one call.

**The two private sides must share NOTHING.** Both carry internals that are not
ucrtbase exports, so no export-driven rule covers them, and where the names
coincide the link silently keeps one under `--allow-multiple-definition` — the
reference side would then call a Rust helper and grade our code against itself.
Four collided on i386; two were real (`__invoke_matherr`, `__p___mb_cur_max`)
and are now renamed on the Rust side, leaving Microsoft's objects untouched.

The Rust archive is also not self-contained: 15 symbols had no provider. Nine
were Win32 (kernel32_vista, user32, advapi32); six are msvcrt internals
ucrtbase does not export (`_scprintf`, `_cprintf`, `fwprintf`, `call_dtor`,
`call_copy_ctor`, `__wine_RtlUnwind`). Those six are stubbed in the harness and
abort loudly if reached, rather than linking `libmsvcrt_static.a` — that archive
defines `strtol` and friends too, and would have been allowed to answer a graded
name silently.

## First results (i386, 2026-09-22)

```
cases=3013  dll_pass=3013 dll_fail=0  rust_pass=2999 rust_fail=14
```

Two families, seven bodies, and both are the SEMANTIC gap this quest is about —
ucrtbase is `_MSVCR_VER` 140, msvcrt is 0:

* **`atoi`/`atol` do not saturate.** UCRT clamps overflow to INT_MAX/INT_MIN;
  our Rust wraps. `atoi("2147483648")` gives -2147483648 where UCRT gives
  2147483647; `atoi("999999999999999")` gives -1530494977.
* **`strtoul` at bases 16 and 36** returns 1 for `"-2147483648"` where UCRT
  returns 4294967295. Base 10 is correct, so this is base-specific overflow
  handling, not sign handling.

These bodies may well be right for msvcrt and wrong for ucrt, which is precisely
the question the harness exists to settle. Reusing them blind would have shipped
both bugs.

## Fault isolation, and why Tier 2 must run on arm64

Without it, ONE faulting body ends the run and hides every export after it, so
blockers cost one four-minute boot each and arrive strictly in order. Six boots
were spent that way learning six names. The guard turns a fault into a FAULT row
and carries on, so one run reports every fault, DIFF and MATCH together.

**VEH, not `__try`.** clang's `__try` does not catch a HARDWARE fault under mingw
at all -- it never emits async-exception handler liveness. `dll/ntdll/seh_poc_user.c`
records this having proved it, and reading that file first would have saved
building the `__try` path. Its 32-bit x86 SEH codegen is broken here besides
(undefined `L__ehtable$` / `$parent_frame_offset`).

**And VEH only fires on arm64.** `RtlDispatchException`'s
`RtlCallVectoredExceptionHandlers` block is `#ifndef _NTOSKRNL_`, and librtl always
builds with `-D_NTOSKRNL_`, so first-chance VEH dispatch is DEAD CODE on i386 and
amd64 -- `rtl-except/src/{i386,amd64}/except.rs` say so outright. arm64 has it wired
and proved: `[SEH-POC-U] VEH on hardware AV: fired=1 resumed=1 (PASS)`. arm64 also
boots in ~40s against i386's ~80s.

**No `setjmp`/`longjmp`.** On arm64 `setjmp` resolves to the SEH-based
`__intrinsic_setjmpex`, which collides with the renamed reference set and would need
valid unwind data to return into from a context we redirected -- the very machinery
being graded. The handler instead advances the PC past the faulting instruction
(arm64 is a fixed 4 bytes, so that is exact) and raises a flag every generated loop
tests, so a faulting body limps at most to the next case boundary.

The guard has a **positive control** that earned its place on the first run: a
synthetic body faults on purpose, was NOT caught on i386, and the harness refused to
grade rather than produce a table that silently stopped partway.

## The wall, and why the sweep is now sharded (2026-09-23)

A single boot used to grade all 220 exports in one process. It never finished, and
the interesting part is what the stopping point did NOT depend on.

| run | exports graded | cases graded |
|---|---|---|
| forward | 114 | ~5,263 |
| reversed (`UCRTDIFF_REVERSE=1`) | 53 | ~2,182 |

The case figures are computed statically, not measured: `generated.inc` records which
`GRADE_*` macro each export uses, each macro's loop nest is a product of known corpus
sizes, so the cumulative case count at any prefix is arithmetic. That killed the last
cheap theory. The wall is **not a fixed export count** (114 vs 53), **not one bad body**
(it would sit at reverse index ~106, not 53), and **not a fixed case budget** either.

Two real defects were found in the harness while chasing it, both in the same place --
`run_guarded` abandons a hung body's thread rather than terminating it, which is the
right call (terminating mid-CRT leaks locks), but nothing stopped the abandoned thread
from continuing to act:

* **it kept writing the shared accounting.** `g_ex_cases`, `g_ex_dll_fail`,
  `g_ex_rust_fail`, `g_ex_first`, `g_cases` and the pass/fail totals are global and
  `export_begin` resets them for the *next* export. So every row after the first HANG
  was racing a zombie. The old comment claimed the worst case was "one spurious FAULT,
  never a silent wrong verdict"; that was wrong, and it is the instability that made
  successive tables disagree.
* **it shared the fault budget.** `g_fault_hits` is one global counter. A zombie
  faulting in a loop spent the allowance of the export actually being graded, and at
  4096 the handler returns `CONTINUE_SEARCH` -- unhandled exception, process dies, and
  the serial log simply STOPS. No FAULT row, no HANG row, no `done`. That is exactly
  the signature of the wall, though it has not been confirmed as its cause.

Fix: one thread OWNS the accounting at a time (`g_owner_tid`), and writes from any
other thread are dropped and counted (`dropped_writes`, `zombie_faults` on the `done`
line). The abandon design is untouched; a zombie simply loses the ability to corrupt a
verdict or end the run.

**Sharding makes the remaining question moot rather than answering it.**
`UCRTDIFF_SHARD=k/N` builds a harness that grades exports `k, k+N, k+2N...`, so a shard
finishes well inside any observed wall and N boots cover the set. The interleave is
deliberate: a contiguous block would put a whole family, and so a whole failure mode, in
one shard. `tools/ucrt_tier2_sweep.py` drives the shards and merges the tables.

The other half is **fail-loud**: the harness now owns a deadline (`UCRTDIFF_BUDGET_MS`)
and prints a `done` line carrying `partial=<n>` when it runs out. Previously a truncated
run and a wedged one were the same observation -- a log that stopped. A shard that did
not finish is now reported INCOMPLETE and its exports listed as ungraded, because the
failure this whole lane must never have is a short run reading as a clean one.

### Still blocked (2026-09-23)

The sharded harness builds clean on all three arches and links, but **has not yet
produced a graded table**: on arm64 the boot reaches `shell ready` and
`[EXPLORER] _tWinMain entered`, then serial goes silent during the kmtest `ExPools`
subtest at ~2,840 lines and stays silent for the whole drain, with ZERO `[UCRTDIFF]`
output. Staging is confirmed correct (`ucrtdiff-arm64.exe` into All Users Startup, and
`ucrtbase.dll` into system32 per `mkramdisk.sh`), and the exe imports all four new
kernel32 symbols, which our kernel32 exports. So this is not the payload failing to
load. It reproduces across runs. Next: establish whether kmtest running concurrently is
what ends the boot -- it contributes thousands of `[PS]` lines to a shared serial
channel and is unrelated to Tier 2.

## Where this stands (2026-09-22, end of session, SUPERSEDED above)

**The arm64 payload does not launch, and the cause is not yet known.** Ruled out:
the exe is staged (`ucrtdiff-arm64.exe -> .../StartUp/ucrtdiff.exe`), the boot
reaches `shell ready`, the payload gate reports `boot ok / payload fail` with ZERO
`[UCRTDIFF]` output, the subsystem is GUI, and every imported symbol across all six
DLLs resolves (the 16 kernel32 names that first looked missing are forwarders to
ntdll, which the objdump parse could not see). kmtest's payload demonstrably runs
from the same Startup folder on arm64.

Next experiment, one boot: stage a MINIMAL arm64 payload that only DbgPrints its
marker, to separate "the exe never loads" from "the harness dies before its first
print". Note that concurrent arm64 builds from the merge queue repeatedly invalidated
the media between build and smoke (`staleness: fail`), so check the VERDICT before
reading any serial log -- three conclusions were drawn tonight from runs that had
never executed.

STILL not verified:

* state isolation in practice. Each side has its own locale and ptd, so a row
  that sets a locale must set it on BOTH — `ref_setlocale` and `setlocale`. This
  is how msvcrt-difftest already works, so the pattern exists;
* whether the gate can run it. `ucrtbase_winetest` only became linkable on
  2026-09-20 (84f063dd416: `-lgcc` was gated on `NOT MSVC`, and the pinned
  llvm-mingw ships no libgcc), and nothing in `tests/qemu/cli/` currently runs
  winetests.

## Why this is worth the wiring

It answers the question the whole reuse plan rests on. 809 of the owed exports
already have a Rust body written and graded against MSVCRT at `_MSVCR_VER=0`,
where ucrtbase is 140. Whether each is reusable is guesswork today — and the
surveys that guessed have contradicted each other twice. This turns 809
judgement calls into one harness run and a table.
