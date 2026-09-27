# Lane: rustconv-speedrun — make the next 100 translations cheap, fast and trustworthy

**2026-09-22 investigation:** see [measurement and scaling proposal](rustconv-speedrun-scale.md).
It re-derives the claims below; seal defaults and namespace refusal already exist in this
checkout, and historical retry/stage timing evidence is incomplete.

**Worktree** `/home/curi/Projects/os-playground/vibeos-rustconv-speedrun` · **branch**
`rustconv-speedrun` · written 2026-09-22 by the control plane · **owner: the user**

rustconv is 64,319 lines across ~30 modules with 112 test files. It works. This lane is about
the three things that decide how much the *remaining* port costs: **money per function, wall
clock per module, and how often the output is silently wrong.**

The timing is not arbitrary. The upstream merge just surfaced the two largest translation
opportunities in the tree — upstream's AHCI/SATA stack (a 16-file chipset quirks framework
plus a full software AHCI implementation, all on disk, none of it built) and job objects
(`ntoskrnl/ps/job.c` gained 2,198 lines upstream; ours are stubs). Both are on
`upstream-sync-merge-decisions.md`'s owed list. **Whatever this lane saves per function gets
multiplied by those.**

---

**2026-09-23 implementation:** [offline cohort and JScript cutover](rustconv-speedrun-cohort.md)
records the 12-source benchmark, compiler/ABI replay mechanism, scoped quality results,
and three-architecture integration evidence. No additional model spend.

**2026-09-23 grading automation:** `rustconv difftest-contract` adds generated,
compiler-checked stateless scalar/byte-buffer harnesses; see the
[contract guide](../../../tools/rustconv/data/difftest/README.md). Genuine C versus
existing Rust: `tolower` core 270 inputs, `_swab` 326 inputs, zero mismatches;
all five sensitivity mutants detected. Artifacts:
`~/vibeos-rust-staging/rustconv-speedrun/difftest-contract-verified-{tolower,swab}/report.json`.
These are sampled host results, not whole-crate/guest correctness claims. No model spend.

**2026-09-23 buffer grading expansion:** explicit shared-arena layouts, normalized pointer
results, and Linux protected-page placements now work through the same contract command.
Fresh four-pilot reports at
`~/vibeos-rust-staging/rustconv-speedrun/difftest-buffers-final-{tolower,swab,memmove,memchr}/report.json`
record 3,352 executions with zero mismatches and all ten mutation controls detected.
The new `memmove` pilot contributes 1,428 executions across six layouts; `memchr` contributes
1,328, including both allocation-edge placements. Guarded cases count left and right
placements separately. Baseline crashes invalidate grading; only explicit memory-fault
controls may count a reproducible SIGSEGV/SIGBUS during the native call. Scope remains
stateless host grading, not parser/stateful/guest coverage. No model spend.

**2026-09-26 difftest on uncovered code, and what is owed next.** Ten contracts over code no
`*-difftest` crate grades (MD4/MD5/SHA1, CRC32, RtlRunEncode, LZNT1, version.dll's
`VerQueryValueW` from the whole Wine TU via the `reactos-amd64` host profile) found shipping
bugs in 4 of them, all fixed (`c70ae28f033`, `ba512e2f47a`): compress's unconditional
out-param store, sha1's misaligned cast-store, version.dll's `Is16` offset, byte-vs-WCHAR
pointer difference and truncated wide literal, plus kernel32 `vdm.rs`'s byte-vs-WCHAR length
found by grepping that class tree-wide. None was visible to smoke. The lesson for the MAIN
pass: difftest discovers bug CLASSES; the leverage is a $0 static check per class run on
every translated body. **Owed, in order:**
1. ~~Three pattern checkers~~ DONE 2026-09-26: `scan_ptr_byte_diff`, `scan_misaligned_cast`,
   `scan_dropped_null_guard`, all gated with EMPTY baselines (agent-reference, "Differential
   grading"). Each re-finds its seed bug in the pre-fix source.
   - First runs found 3 / 18 / 5 real sites in the tree: 3 byte-count bugs (vdm.rs:540,
     move.rs:315, shell32 `wildcard_to_file`), 18 byte-array casts, and 5 dropped guards
     (ole32 ×3, mlang, fastfat). One more finding, msvcrt `_wfopen_s`, was a scanner miss
     of a cast-then-compare test, now recognised.
   - Riding along: an umpnpmgr UUID formatter that printed bytes 0..8 twice, and mlang's
     dropped `if (!old_font) return E_FAIL`.
   - All fixed. The 6 reviewed-intentional sites carry `nolint` with their reason.
   - The "store only at the `out:` label" half of (c) is DONE too, as
     `scan_store_on_failure` (0 in the tree, re-fires on compress). Its false positives
     turned up two real bugs of other classes, both fixed: kernel32
     `GetConsoleInputExeNameW` returned 0 on success, so the A export never filled its
     buffer; ntoskrnl `MiMapViewOfDataSection` had dropped the view-size range checks and
     aligned the offset after sizing from it, and ARM3's `RtlLongLongSub` could not fail.
2. ~~`scan_bool_width` triage~~ DONE 2026-09-26, and now GATED (empty baseline). The 202
   findings were 25 Rust DEFINITIONS returning u8/BOOLEAN where C says BOOL (the decls were
   right). A new definition-vs-C-twin mode found 31 more that no Rust decl pointed at —
   exports only C calls. All 56 widened to BOOL (61 bodies: win32k's REGION_* and
   GreSetBrushOrg have per-arch cfg twins, which the scanner now also compares);
   userenv's local decls narrowed `OpenEventW`/`CreateMutexW`'s BOOL params to u8 too.
3. ~~`scan_sibling_drift` / `scan_arch_cfg` pass~~ DONE 2026-09-26.
   - sibling_drift: still the same 65 candidates hand-read on 09-20 (`WSPSelect` stays
     listed after its fix, by design). The 19 in files changed since were re-read: no bug.
   - arch_cfg: 2,825 sites vs 2,453 at the July audit; path-free diff against that tree
     (`git archive 2607142dce2`) left 507 new, of which the 133 in the risky shapes
     (single-64-bit-arch gates, `not(aarch64)`, smp-per-arch) and the 27 that exclude only
     arm64 were read. Three bugs:
     `ex/init.rs` SharedUserData Reserved1/3 (amd64 stored truncated 64-bit constants, 0
     for the WOW64 system range start; elsewhere Reserved1 got SystemRangeStart, not
     HighestUserAddress); user32 `GetClassLongA/W` `_WIN64` gated amd64-only (new gated
     class — now the general `scan_cfg_vs_c`, see below); and a fourth, stale copy of arm64 SMP `SYNCH_LEVEL` = 2 in
     `ke-inlines` that the July-25 fix missed (dead under smp today, now 12).
   - Then mechanized: `scan_cfg_vs_c` (gated, empty baseline) evaluates every in-body arch
     cfg and every C `#if` in the twin (arch macros, `_WIN64`, `_MI_PAGING_LEVELS`) to arch
     sets and flags a Rust set that splits arches the C keeps together. Of 1,648 in-body
     arch cfgs: 438 agree with the C mechanically, 548 are ABI/layout spellings or
     per-arch file twins, 77 mix in non-arch features, and ~650 have no C arch `#if` to
     compare against — the Rust-only cfgs, still a reading job. First run flagged 54; the
     rule refinements (arm64-unaware C, paging levels, file twins, bindgen layout) left 15,
     all Rust-only diagnostics / bring-up allowances, now `nolint`ed with reasons. Only
     real hit: GetClassLongA/W, already fixed. Supersedes the one-day `scan_win64_cfg`.
   - The ~650 Rust-only in-body cfgs the tool cannot decide (`--worklist`) went to three
     read-only Sonnet reviewers (664 sites after removing the 37 July had cleared); every
     claimed bug was re-verified against the C before any edit. 596 LEGIT. Fixed:
     win32k `TEXT_NextLineW` measured no text on ANY arch (four stacked cfgs AND to
     false) — which led to 8 more dead stacked-cfg items (orphans of deleted fastcall
     twins), now a second gated check in `scan_cfg_vs_c`; smlib `SmConnectToSm` (an
     `NTDDI_VERSION < VISTA` test mistranslated as pointer width); `PsLocateSystemDll`
     (arm64 skipped four bugchecks the C makes on every arch); `KeSetSystemAffinityThread`
     (`CONFIG_SMP` translated as `not(x86)`); two `MiResolveTransitionFault` diagnostics
     widened. Rejected after checking: `MmRequestPageMemoryConsumer`'s amd64 yield is the
     deliberate 97d678e72ef boot-stall fix (now commented at the site).
   - Owed from that audit: `KeSetTimeIncrement`'s amd64-only SharedUserData zeroing (does
     arm64 carry the same dormant boot hang? instrument before changing); arm64
     shortcuts `MiGetPfnEntry` (skips the PFN-bitmap NULL check), generictable's
     arm64-only splay-walk loop guard, `MiBalancerThreadInner` (arm64 `break`s where x86
     bugchecks NO_PAGES_AVAILABLE); 29 win32k x86/not(x86) arms with identical calls
     (cosmetic). Tool gap: map C `CONFIG_SMP` to `feature = "smp"` so the thrdobj class
     is mechanical too. Reports: scratchpad `cfgaudit/shard_{A,B,C}.report.tsv` (not kept).
   - Correction (conformance lane caught it): two of the "revived" dead items were dead ON
     PURPOSE. i386 `RtlPrefetchMemoryNonTemporal` is owned by rtlk's prefetch_asm.rs (a
     `ret` that kiinit patches to NOP); the revived empty stub won the link under
     `--allow-multiple-definition` and i386 kmtest RtlMemoryKM bugchecked 0x1E, EIP=0
     (e50515abfab rejected). arm64 `RtlUshortByteSwap` duplicated the byteswap crate. Both
     deleted instead; dead-cfg output now says to look for another provider first.
   - DONE, tooling: `scan_duplicate_symbols` rebuilt on `tools/rust_link_model.py`. It is
     per binary, per arch and feature-resolved, and it sees `global_asm!` labels, so the
     prefetch shape is a positive control. The first honest run had 230 pairs; model fixes
     (build.rs cfgs, per-binary features, `if arch` link blocks, i386 decoration classes,
     `#[cfg(any())] // comment`) brought that to 21. All 21 were confirmed with `llvm-nm` in
     the built archives. Fixed:
     classpnp's four Ke/Io shims duplicated ntoskrnl-vista-rust, and its
     `KeSetCoalescableTimer` was a `0` stub. The kernel exported a CAS spin stub over
     rtl-sync's real SRW locks on amd64/arm64 (callers: `RtlAddFunctionTable` & co.).
     i386's local-only asm `KeInvalidateTlbEntry` shadowed the SMP-shootdown one by archive
     order alone. rtl-heapuser (user-mode PEB heap list) is now a `heaplist` feature the
     kernels leave off. rtlk's `KeGetPcr` and ntoskrnl's `RtlFillMemoryUlong` duplicates are
     gone. Deliberate, `nolint`ed with reasons: arbiter's amd64 `KeGetCurrentIrql` (same
     `mov cr8` body) and freeldr's six CRT overrides (as freeldr.c overrides libcntpr).
     Baseline empty.
   - Not read yet, and still owed: the remaining ~350 new sites — 249 are `x86` /
     `not(x86)` calling-convention twins (sibling_drift's arch-gate mode diffs those
     body-to-body), 73 are the inclusive `any(x86_64, aarch64)` shape.

## TASK 0, AND DO NOT SKIP IT: measure where the money and the time actually go

Every number below is an anecdote from a lane that happened to notice something. Nobody has
ever measured the pipeline as a whole, and that is now possible: **`meta.json` in each run
archive carries `cost_usd` per call** (found by the shell32 lane on 2026-09-19 while pricing
a re-buy question — it converted a three-round argument about counts into a one-line answer).

So the first deliverable is a ledger across the whole `.responses/` bank, fleet-wide, and it
should answer:

- Cost per function by module, by model, by seal-flag namespace.
- What fraction of calls are **retries**, and what fraction of retries ever change the verdict.
- What fraction of functions are lowered deterministically for $0, and how that varies by
  module shape. (AGENTS.md says ~9% repo-wide, 41% on stub-heavy modules, ~0% on real driver
  code — **verify that; it is the kind of remembered number that rots.**)
- Wall clock per stage: how much is API latency vs `profile_derive` vs `crate-repair` vs
  cargo check.

**Optimise nothing before this exists.** The whole reason this lane is worth running is that
the fleet has strong opinions about rustconv's costs and approximately no measurements, and
that is exactly the shape that produced `0 → 327 → 60 → 65` on a different question last week.

Recent landings to calibrate against (shell32 lane, all on master):
`75bd42035f4` recyclebin — 6 TUs at **$0** · `633d827cd39` CFolder+CFolderItems 62 fns **$0.35**
· `98559324e0f` CRegFolder **$0.73** · `1d73029720c` CExtractIcon **$0.12**. The pipeline is
already cheap on some shapes. Find out which, and why.

---

## The four known defects, with provenance

I did not find most of these; the lanes did. Provenance is marked because you should
re-derive before building on any of them — every one is a claim, not a measurement you made.

**1. The resume cache is keyed by SEAL-FLAG NAMESPACE, so turning a flag on later re-buys the
whole file.** *(msvcrt lane, 2026-09-19, and it is why AGENTS.md says "pass them from the
START".)* An earlier translation bought under different flags reports "0 banked, N to buy" and
is indistinguishable from virgin work — the old namespace survives untouched, it is just not
the one you are now asking for. **This is the highest-leverage item on the list**, because it
converts a policy rule that agents must remember into money they silently spend. If a body's C
text is unchanged, a flag that only tightens *auditing* should be able to replay it.

**2. Sealing is defeated by RECALL, and the guard only fires when NO token survived.**
*(msvcrt lane; AGENTS.md records both cases.)* `_tzset` came back with all 35 constants
PERMUTED into the wrong slots and the seal reported success, because its invariant is a closed
SET and a permutation satisfies it. Worse, `MSVCRT_exp` shipped `math_error(0, …)` where the C
says `math_error(_DOMAIN, …)` — kept some tokens, permuted the rest, **never flagged**. This is
the pipeline's largest correctness hole: it ships silently-wrong bodies that compile and link.
A positional check (token i lands in slot i) rather than a set check is the obvious direction.

**3. A wrong CHECKER bills on every retry and can never pass.** *(CDefView: six workers
rejected across three paid runs by an arity check that miscounted a C++ receiver; fixing the
checker recovered five at $0. va_arg: two rejects on an alias the macro scan could not see —
I fixed that one on 2026-09-19, `4c611bdf611`.)* Two instances in one family is a class. **The
systematic version of this task is a sweep: for each deterministic audit, can it be wrong in
the direction where the MODEL was right?** Each one that can should say so in its own reject
message, because the tell — an identical reject across independent samplings — is currently
something the agent has to notice rather than something the tool reports.

**4. `--retry-rejects` re-buys a verdict that cannot change.** A deterministic reject is
deterministic; retrying a body the checker will reject again pays full price for the same
answer. The harness already holds both attempts: **diff the signature, and an identical one
means deterministic, not flaky.** (Same shape as the `CONFORMANCE_FLAKE` misclassification
that blocked the merge queue for 75 minutes — the label is the expensive part, not the retry
count.)

---

## Where the speed probably is — hypotheses, not findings

Ordered by my guess at leverage. **Task 0 should reorder this list**, and if it does, say so.

- **Deterministic lowering is the only free win.** Every function `trivial_c.py` handles is a
  function nobody pays for. What is the actual ceiling, and what shapes are one rule away from
  qualifying?
- **Parallelism.** How much of a module's wall clock is serialized on the API, and does the
  worklist exploit independence between functions as well as it could?
- **Prompt/context size.** AGENTS.md: `translate` re-sends context per function, which is why
  the real bill is 2–5× the `estimate` floor. Cached system blocks exist; how much of the
  per-function payload is actually re-sent, and how much could be shared?
- **`crate-repair` before payment.** It already repairs the whole-crate `cargo check`
  deterministically. What does it still miss that a human then pays the model to fix?

---

## Rules with teeth here — you are working on the thing that spends money

- **Never re-translate a file that already has output** to test a change. Use `--dry-run`,
  `--prompt`, and the banked `.responses/` bodies. A benchmark that re-buys is a benchmark
  that costs more than it saves.
- **Never delete a `.responses/` dir**, and never delete a translated `.rs`. Removing paid
  artifact needs the owner directly — a relay is not approval.
- **When a deterministic audit rejects a body, suspect the AUDIT first.** That is the whole of
  defect 3 above, and it is in AGENTS.md because it has been paid for twice.
- **Never Ctrl-C a `translate`/`submit` mid-call** — killing at function 35/40 re-pays all 35.
- Changes here affect every lane's spend. `check-tooling` runs rustconv's 112 test files;
  keep them green and add to them.
- **`verify_signatures` will report `SCANNER_ERROR` in this fresh worktree.** Correct, not
  broken: it needs generated headers from *this* worktree and refuses to borrow a sibling's.
  Fix with `./vibeos-cli build --arch i386`. The `land` preflight warns rather than blocks.
- A docs-only batch now skips the gate entirely, so write-ups land in seconds.

## Open questions for the owner

1. **Is there a budget for measurement runs?** Task 0 should be answerable from banked
   artifacts at $0, but a controlled A/B of a prompt change is not.
2. **Which of the four defects first?** My order is 1, 2, 3, 4 — cache key first because it is
   pure money, recall second because it is the only one that ships wrong code. Defect 2 may
   well deserve to be first on correctness grounds alone.
3. **Does this lane get to change the default flags?** Several fixes here would make
   `--seal-facts --seal-calls --seal-wide` unnecessary to pass explicitly, which is a
   behaviour change every other lane inherits.
