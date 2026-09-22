# Tier 2 ucrt difftesting: grade in the guest, rename with objcopy

Status: **design proven, not built**. Every risky step below was verified
against the tree on 2026-09-21; what remains is CMake wiring and a comparison
harness.

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

NOT verified, and the next things to find out:

* that a full renamed set LINKS into a test exe (some TUs may reference exports
  through asm or tables that objcopy does not reach);
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
