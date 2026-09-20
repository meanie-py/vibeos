# Lane: upstream-sync — ReactOS as a bug oracle

**Worktree** `/home/curi/Projects/os-playground/vibeos-upstream-sync` · **branch** `upstream-sync`
· written 2026-09-20 by the control plane · **owner: the user**

We forked ReactOS at `5213cf717cf` (2026-02-11). They have moved 1,263 commits since. This
lane makes their work reachable without treating it as a patch queue we owe.

**The framing that matters, and the one thing not to lose:** upstream is not a source of
patches for us to apply. It is an **oracle that tells us where the bugs are.** When they fix
`ntoskrnl/ex/sysinfo.c`, that is evidence our `sysinfo.rs` — translated from the *pre-fix* C —
contains that bug, faithfully reproduced. We did not merely diverge from their code; **we
froze their bugs in Rust at the fork point.** Every upstream fix touching a file we translated
is a lead.

---

## Measured, 2026-09-20 (re-derive before trusting — these will drift)

```
fork point            5213cf717cf   2026-02-11  [BOOT][ENVIRON] Remove Environ (#8667)
upstream since fork   1,263 commits            through 2026-09-19
ours since fork       5,308 commits            we are moving ~4x their rate
```

Upstream touched **3,958 files / 6,209 edits**, which split three ways:

| | files | edits | what it means for us |
|---|---|---|---|
| Still C/C++ here | 2,480 | 4,035 | directly mergeable |
| We have translated | 668 | 1,149 | merge the dead C, then review the Rust |
| Not in our tree at all | 810 | 1,025 | new upstream work, or files we deleted on purpose |

**Dry-run full merge** (`git merge-tree --write-tree --name-only HEAD upstream/master`,
read-only, touches nothing):

```
244 real conflicts
   85  on files we have translated
   79  in BUILD FILES              <- the landmine, see below
    5  in modules/rostests         <- essentially clean
```

Two caveats on these numbers, both of which bit me while producing them:

- **The 668/787 "translated" count is an UPPER BOUND.** I matched `<stem>.c` to any
  `<stem>.rs`, so `shlwapi/utils.cpp` matches *any* `utils.rs` in the tree. Phase 0 exists to
  replace this with the real cohab registry. Do not quote 668 as fact.
- `git merge-tree` prints conflicted paths, a blank line, **then informational chatter**. Parse
  up to the blank line. Reading past it reports 795 conflicts instead of 244 — I published
  795 before catching it.

---

## The reframe: merge the dead C too, everywhere

A cohab-flipped `.c` is **still on disk and commented out in CMake** — e.g. `ntos.cmake:216`
reads `#${REACTOS_SOURCE_DIR}/ntoskrnl/lpc/reply.c`, and `reply.c`, `utils.cpp`, `sysinfo.c`
are all present. Nothing compiles them.

So merging upstream's change into a flipped `.c` **cannot break the build**, and it buys the
thing that makes this whole burndown affordable: **the C becomes a live oracle again.** Review
stops being archaeology and becomes an ordinary diff against text already in our tree:

```bash
git diff <watermark> HEAD -- ntoskrnl/lpc/reply.c     # what upstream changed
# ...then read sdk/lib/rust/lpc-rust/src/reply.rs against it
```

That is why this is not "merge the C parts and hand-port the Rust parts". It is **merge
everything, then review the Rust against the refreshed C.**

**Conflict resolution is a JUDGEMENT, not a default.** The owner's instruction: for each
conflict the agent reads the upstream diff *and* our corresponding Rust, and writes the patch.
Do not blanket-resolve "ours" because we are 5,308 ahead, and do not blanket-resolve "theirs"
in flipped files because the C is dead. Both are shortcuts that discard the reason the
conflict exists. Note the asymmetry though, because it changes what a correct resolution looks
like: in a file we **compile**, ours is the living code; in a file we have **flipped**, ours is
stale dead text and theirs is the fresher oracle.

---

## THE LANDMINE — read this before merging anything

**79 of the 244 conflicts are in build files**, exactly where our cohab comment-outs live:
`dll/ntdll/CMakeLists.txt`, `dll/win32/CMakeLists.txt`, `base/shell/explorer/CMakeLists.txt`,
`boot/freeldr/freeldr/*.cmake` and more. Upstream touched **405 build files** in total.

A merge that resolves one of those wrongly **re-enables a commented-out `.c`**. That is a
partial cohab flip — the exact failure that froze the guest on 2026-09-19, where 58 live C++
symbols were silently captured, the taskbar hung inside `OnCreate`, `ProcessStartupItems` never
ran, and the conformance sweep launched **0 of 91** tests. **Smoke passed on all three arches
while the guest was dead**, because the freeze is after the `[WIN32K] shell ready` marker.

Arriving via a merge it would be worse: git is the author, nobody reviewed it as a flip, and
there is no diagnostic.

**So phase 1 is a census gate, not a merge.** Snapshot every commented-out source line across
every CMake file; after any upstream merge, assert that set is unchanged except where a human
said otherwise. Model it on the scanners in `tools/` (`scan_orphan_modules`,
`scan_dropped_seh_pairs`) and wire it into `vibeos-cli check`. **Do not run the big merge
before this exists.**

Related, from the shell32 lane the same night: a stale `ORPHAN-BY-DESIGN:` banner switches
**four** checks off — `scan_extern_arity`, `scan_cc_mismatch`, `scan_silent_failures` and
`verify_lost_exports` all skip a file on sight of the token. If a merge wires a module, that
banner must come off in the same commit or four scanners go quiet on live code.

---

## Provenance: a per-FILE watermark

The unit is the file, not the commit, because the question worth answering is *"is our Rust
reconciled with the current C?"* — and that is per-file.

```json
"ntoskrnl/lpc/reply.c": {
  "upstream": "<sha>",
  "rust": "sdk/lib/rust/lpc-rust/src/reply.rs",
  "reviewed": "2026-09-20"
}
```

One structure, three jobs:

- **Burndown** — count of translated files whose watermark ≠ upstream HEAD. One number, it
  goes to zero, and a scanner can gate it.
- **Review queue** — `git log <watermark>..upstream/master -- <file.c>`, exact and
  re-derivable by anyone, never a typed table. (A typed table is a claim whose age the next
  reader cannot see; `recyclebin-rust`'s Cargo.toml note said "PARKED at 25 errors" and it was
  at 204.)
- **Provenance** — the watermark, plus an `Upstream-Commit: <sha>` trailer on every backport
  commit so `git log --grep` finds them forever.

The big merge sets **every** watermark at once. That is most of why it is worth doing as one
merge rather than a trickle.

---

## Phases

| # | what | why in this order |
|---|---|---|
| **0** | Build the real cohab registry: which `.c`/`.cpp` are commented out in CMake **and** have a `#[no_mangle]` Rust owner | Replaces the stem-match guess. Every later number depends on it, and the guess is an upper bound. |
| **1** | The comment-out census gate, wired into `vibeos-cli check` | Nothing else is safe without it |
| **2** | Merge `modules/rostests` alone — **5 conflicts** | Proves the pipeline on the piece the owner explicitly wants (kmtest and friends are still C/C++ here and we want them), at near-zero risk |
| **3** | Full merge, 244 conflicts, agent-resolved per the rule above; set all watermarks | One-time. Expect the build-file conflicts to dominate the care budget. |
| **4** | Burn down the review queue: refreshed C diff vs our Rust | Now a diff, not a port |
| **5** | `vibeos-cli upstream-sync` — fetch, merge, report the new queue | Only meaningful once the watermark is at parity; the owner will run it ~weekly |

---

## Rules that apply with extra force here

- **`git merge` only. NEVER `reset --hard`, never force-update `master`.** A `reset --hard` to
  a stale `origin/master` once rewound mainline by ~81 commits to the 2024 fork. A
  `reference-transaction` hook blocks non-fast-forward `master` updates.
- **Never delete a translated `.rs`, never revert Rust to C, never hide a translation from the
  build** to make a merge resolve cleanly. A broken Rust translation, integrated and failing,
  is the correct state. These outrank the smoke-test imperative.
- **`./vibeos-cli land`** — the merge queue. Do not hand-roll gate-then-push. A docs-only batch
  now skips the gate entirely (`tools/docs_only.py`), so documentation lands in seconds.
- **Never broad-kill QEMUs** (`pkill -f qemu` kills your own process tree). Use
  `./vibeos-cli kill`.
- **Path discipline**: resolve `$W=/home/curi/Projects/os-playground/vibeos-upstream-sync` once
  and prefix every path. An absolute path to the main checkout writes to the main checkout.
- **First thing you will hit in this fresh worktree:** `vibeos-cli check` reports
  `verify_signatures SCANNER_ERROR — scanner did not complete`. That is correct, not broken.
  It needs generated headers from **this** worktree and refuses to borrow a sibling's, because
  a verdict that depends on which siblings happen to exist is not a gate (it once moved a
  mismatch count 987 -> 9 with no source change). Fix: `./vibeos-cli build --arch i386`. The
  `land` preflight classifies it as SCANNER_ERROR and warns rather than blocking, so it will
  not wedge a landing.
- The 810 not-in-our-tree files are `modify/delete` conflicts and **each needs a reason, not a
  default**. Keep them out of any automated pass.

## Open questions for the owner — do not decide these alone

1. **The 810 absent files** — which were deleted on purpose and must stay deleted?
2. **How much of `base/applications`, `rosapps`, `base/setup` do we even want?** They are large,
   upstream-active, and barely touched here.
3. **Whether the weekly sync is allowed to land unattended** once phases 0–4 are done, or always
   produces a queue for review.
