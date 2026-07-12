# rustconv: cost-efficient C→Rust translation (read before spending API $)

> **ARCHIVED 2026-07-10** — the playbook folded into `rustconv-pipeline.md` (§Cost-efficient translation). Kept for the full narrative + rtl-specific recipe.


This is the playbook + reasoning behind the function-by-function `rustconv
translate` flow with prompt-caching and a resume cache. It was validated on the
win32k frontier: **18 files (all of ntuser + DirectDraw) for $18.98, with
caching saving ~$18** (i.e. it roughly halved the bill). Use it for any paid
bulk translation (rtl included).

## TL;DR

```bash
source .venv/bin/activate   # or use the venv that has tree_sitter_c + anthropic
export ANTHROPIC_API_KEY=$(grep '^ANTHROPIC_API_KEY=' .env | cut -d= -f2-)

# Dry-run first (FREE) — see function count + first-prompt size, set your cap.
python -m rustconv --arch i386 translate path/to/file.c --dry-run

# Real run: opus, banked to staging, hard cost brake.
python -m rustconv --arch i386 translate path/to/file.c \
    -m opus -o ~/vibeos-rust-staging/<area>/file_out.rs --cost-cap 2.00
```

Read the per-file summary line, not the streaming spam. Re-run the SAME command
to resume for $0 if it caps or dies. Never Ctrl-C mid-call.

## Why function-by-function (not whole-file `batch`)

`rustconv batch` sends a whole file in one request. On files >~1000 LOC the model
**silently stubs "lazy" functions** — you get an incomplete translation that
looks done but is missing bodies, and you discover it at link/smoke time after
paying. `translate` makes **one request per function**, which forces each
function to be fully translated. The cost of that is re-sending the file's shared
context (imports, gap functions, globals) on every call — which is exactly what
the caching below neutralizes.

## Why prompt-caching matters (the big lever)

The per-file shared context is **byte-identical across all N functions** of a
file. We put it in a *cached system block*, so function 1 writes it (~1.25×) and
functions 2..N read it from cache (~0.1×) instead of re-sending it at full price.

This scales with function count, so high-function-count files win enormously:

| File | Fns | Cost | Caching saved |
|------|-----|------|---------------|
| sysparams.c | 173 | $2.89 | **$6.91** (≈70%) |
| menu.c | 121 | $4.16 | $5.54 |
| window.c | 68 | $2.51 | $2.60 |

The callee context is also trimmed to **signatures only** (the calling
contract) instead of full bodies — more uncached bulk removed.

(These changes are live on `continue-rust-port`, commit `d20231896a8`.)

## Why NOT the Batch API (counterintuitive)

Batch is 50% off list — but it **breaks our caching**. Our caching relies on
*sequential* calls warming a 5-minute-TTL cache; batch submits all N requests at
once with no ordering guarantee, so the shared-context block is unreliably warm.
The math, per 1,000 LOC (Opus):

- **Sync + caching (this flow): ~$0.69**
- No-cache list price: ~$1.77
- Batch (50% off) but caching missing: ~$0.89

→ **Sync+caching already beats batch-without-caching.** Our caching cuts
61–70% of input; batch's flat 50% can't reliably stack on top. The only way
batch wins is *whole-file* batch — which has the lazy-stub quality problem above.
**Conclusion: stay on sync + caching.** Batch only pays off for a large batch of
*small* files (low function count → little caching benefit, low stub risk).

## Cost discipline

- **`--cost-cap N`** is a hard brake: it stops before the next paid call once
  cumulative estimated cost ≥ N. Set it from the dry-run (function count × a few
  cents). It's per-invocation.
- **Watch the live `[running $X.XX]` counter** and the per-file summary:
  `Cost ≈ $X (caching saved ~$Y)`. If it ever says **"NO cache hits — caching
  not working!"** on a run that made API calls, STOP — something's wrong with the
  cached blocks (don't burn money uncached).
- **Estimate** with the measured rate: **~$0.69 per 1,000 LOC** (sync+cache,
  Opus), higher on output-dense files. Dry-run gives the function count.

## Durability — never lose paid work

- Every function persists to the **resume cache**
  (`tools/rustconv/.responses/<subsystem>__<stem>/`) the instant it returns. A
  killed / credit-exhausted / capped run **resumes for $0** on re-run: same
  command, unchanged C bodies → `CACHED ($0)`. A kill costs at most the one
  in-flight function.
- **Never Ctrl-C mid-call** — you're billed for in-flight output regardless, and
  you only save the function that was streaming.
- **Always `-o` into a durable in-repo or staging path**
  (`~/vibeos-rust-staging/...`), **never `/tmp`** (wiped on reboot). Banked
  `_out.rs` survive independently of the resume cache.
- The cache keys on a **sha256 of the C function body** — edit the C and that
  function re-translates; the rest still resume. `--no-resume` forces fresh.

## Gotchas

- **Split-TU files** (`foo.c` ends with `#include "foo.split2.c"`): the scanner
  **auto-inlines** the split chunks (`_expand_split_includes`), so a single
  `translate foo.c` covers every chunk. Do NOT translate the `.splitN.c`
  separately — that re-pays and risks duplicate symbols.
- **Dual-cfg fastcall pattern:** an i386-fastcall function is emitted as THREE
  pieces — `#[cfg(target_arch="x86")] fn NAME` (fastcall) + `#[cfg(not)] fn NAME`
  (C) + `fn NAME_impl` (body). Consequence: you **cannot reliably pre-seed the
  resume cache from a stitched `_out.rs`** (the 1:1 function-hash cache only
  captures one of the three pieces). If you have an incomplete old banked file,
  re-translate it *fresh* rather than trying to splice — clean output beats
  saving a couple dollars on a Frankenstein that won't compile.
- **`translate`/`submit` SPEND REAL MONEY per function.** Don't re-translate a
  file that already has output without approval. Don't kill mid-run to "adjust."
- **Cargo incremental can mask regressions** — verify integrations with a cold
  build / cold smoke, not `--skip-clean-build`.

## Integrate later (the free half)

Translation output is just *banked* until wired. Integration (free, no API):
add `pub mod X;`, comment the `.c` out of CMake (cohabitation), add the `.rs` to
the build DEPENDS, then run the gate: crate-green ×3 → flip-link
(`ninja <module>`, 0 undefined symbols) → `lint_kernel.py` → `scan_ud2 --check`
→ cold-smoke ×3. The nm ABI oracle (`llvm-nm --defined-only file.c.obj`) tells
you each export's convention: `@Name@N`=fastcall (dual-cfg), `_Name@N`=stdcall
(`extern "system"`), `_Name`=cdecl (`extern "C"`), no hit=FORCEINLINE→inline it.

## rtl-specific

rtl files are mostly **pure-algorithm** with clean FFI surfaces — good
candidates for careful per-function review. Watch ABI: NTAPI = **stdcall
(`_Name@N`) on i386**, **cdecl on amd64/arm64**; `extern "system"` resolves
correctly per arch. Many rtl symbols are **ntdll exports** too (RtlIpv4*, etc.),
which hits the cold-link blocker class — wire those only after that's resolved.
Use `tools/rustconv/rustconv/verify_abi.py` / `verify_ffi.py` to catch
`extern "C"` vs `extern "system"` mismatches that link silently on amd64/arm64
but break i386.
