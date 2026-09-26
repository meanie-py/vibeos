# rustconv at million-line scale: measurement and direction

Measured 2026-09-22 in `rustconv-speedrun`, base `580948f50b2`.
The initial measurement below made no paid API calls, production translations, bank edits,
default changes, commits or pushes. The subsequent implementation and paid host pilots are
tracked in [pilot results](rustconv-speedrun-pilots.md). The immediate deliverable is a reproducible historical ledger. The
architecture below is a proposal, not a claim that the implementation already provides it.

## Task 0: what the bank actually tells us

Run from the repository root, without model credentials:

```sh
PYTHONPATH=tools/rustconv python3 -m rustconv.cost_ledger \
  --worktrees . --output ~/vibeos-rust-staging/rustconv-speedrun/ledger.json
```

The command discovers registered **local** worktrees and their `.responses*` banks,
including backups. Explicit bank paths can supplement discovery for other collected banks.
This run found five banks across four worktrees, with 1,123 `meta.json` files. The backup
contained no call metadata. This is not a census of inaccessible machines or deleted history.
The JSON includes input paths and SHA-256 digests, per-call evidence, model/target/namespace
breakdowns, missing evidence, and discovery scope.

| Measurement | Observed |
|---|---:|
| Outcome events | 20,931 |
| Archived call records | 7,919 |
| Archive calls matched to outcome events | 4,945 |
| Reconciled API outcome / archive records | 8,386 |
| Recorded estimated cost | $79.47427721 |
| Rejected/API-error outcome estimated cost included above | $6.48010473 |
| API outcome records with elapsed time | 5,412 |
| Sum of their API elapsed times | 237,898.768 seconds |
| Cache-hit events | 12,634 |
| Deterministic events | 2,756 |

**The last two counts are events, not unique functions. API elapsed sums are service time,
not module wall time:** concurrent calls overlap. Cost fields are configured token-price
estimates (`sink.RunArchive.commit`, `__main__._result_fields`), not provider invoices.
No savings or model-quality ranking follows from these unbalanced historical cohorts.

Normalization: deduplicate byte-identical outcome JSON; identify copied archive records
by run, target, model, call index, metadata and current artifact hash. Join archive records
to outcomes by run/target/model/artifact basename plus input/output usage. Prefer the
higher-precision outcome estimate, never add both. Ambiguous joins fail. Missing seal flags
remain unknown, not false. Independent Decimal arithmetic over the original files reproduced
`72.9932 - 47.1245 + 53.60557721 = 79.47427721` (archives minus overlap plus outcomes).
The first two historical stores contain no provider request IDs, so arbitrary rewritten
copies cannot be conclusively deduplicated. This run found no exact copies.

Among **5,697 observed non-preamble source versions**, 751 (13.18%) have at least one
recorded deterministic outcome. Identity is target + source path + unit + exact source hash +
arch + C++ class; repeated cache hits do not inflate that denominator. Shell32: 521/2,779
(18.75%); msvcrt_static: 15/232 (6.47%); netcfgx: 38/231 (16.45%). These figures measure
historically observed routing, not the current lowerer's ceiling or repository-wide coverage.
The handoff's 9%/41% claims are not established by this dataset.

There are 440 subsequent API outcomes for an unchanged observed source identity and
namespace. Of 269 attempts following an audit rejection, 134 produced `generated`, 85 had
exactly the same rejection message, and 3 produced an API error. Checker, prompt and substrate
revisions were not recorded: **this is neither a controlled retry success rate nor proof that
85 calls were avoidable.** `generated` also does not mean integrated or semantically correct.

### Evidence missing from Task 0

* The sequential `--retry-rejects` loop records only its terminal outcome. Earlier rejects
  increment a running counter but are absent from both the outcome history and archive call
  list. Its partial path is reused. Complete historical retry costs cannot be recovered here.
* Directory runs reuse artifact names. For example, the `20260718-013424-netapi32` run in
  the research bank has 13 different source-file preambles pointing to `00_preamble.rs`.
  The ledger found 45 joined records sharing an artifact path with another event. It records
  their usage but refuses to attribute the remaining bytes to each earlier call.
* Interrupted calls can report zero usage without proving that the provider charged zero.
* `profile_derive`, crate repair, compiler stages, end-to-end makespan, human repair effort,
  and integrated correctness are not captured in these historical records.

Task 0 therefore exists as a **partial historical measurement with explicit unknowns**.
A whole-pipeline cost/time conclusion is still owed. Increasing concurrency or changing
models before obtaining that evidence would repeat the original mistake.

## Re-derived status of the handoff's four defects

1. **Seal namespace separation exists, but a refusal guard already exists too.**
   `_open_response_cache` calls `require_no_namespace_rebuy` before a non-dry run. It refuses
   conflicting banks rather than silently migrating or buying. All three seal flags already
   default to true in this checkout. The missing capability is audited recovery across
   policies, not changing those defaults. This guard alone is not a fleet-wide index.
2. **Semantic placement remains incomplete.** `require_argument_facts` already binds simple
   constant arguments to sealed call occurrences and catches some permutations. It explicitly
   excludes several argument shapes. Outside it, a synthetic source `a = 10; b = 20;`, sealed
   with `seal_facts(..., dialect="c")`, accepts the two placeholder tokens swapped and
   unseals to `a = 20; b = 10;`. No model was involved. `unseal_facts` also retains by-spelling
   recovery and a last-attempt relaxation. The specific historical `_tzset`/`MSVCRT_exp`
   artifacts have not been re-audited by this investigation; their anecdotes are not new findings.
3. **Checker false positives are a plausible class, not a completed audit.** The historical
   ledger contains the `_sopen` va_arg arity rejection. Repeated rejection messages are useful
   triage evidence, but do not establish whether the checker or candidate is wrong. A sweep
   of every checker against positive and negative fixtures remains owed.
4. **A deterministic checker does not imply a deterministic outcome for new candidates.**
   The retry loop changes its prompt using `_reject_correction`. The history includes both
   repeated rejects and later generated bodies. Stop/route on actual candidate, checker and
   context identity; do not ban retries merely because the error text repeats.

## Proposed architecture: preserve behavior, spend on residual decisions

The unit of success should be an **integrated, behavior-checked module at a recorded cost**.
Lines emitted and compile success remain useful intermediate metrics. They cannot serve as
semantic acceptance, especially for library code a desktop smoke never exercises.

Build on the existing substrate closure, c2rust anchor, deterministic lowerers, candidate
oracle and differential harnesses. Avoid replacing them wholesale. The crucial change is to
make their evidence explicit and stop granting an opaque worker body more authority than its
checks establish.

1. **Immutable purchase and evidence store.** Give every invocation and every attempt a
   unique identity. Save raw request/response, source/configuration identity, provider usage,
   finish state and each subsequent audit verdict separately. Rejected and interrupted
   attempts are first-class assets. Store candidates by content; record validation as
   versioned edges, never as permission to overwrite a purchase. Index banks across worktrees.
   A policy change requests revalidation; it does not erase discoverability or authorize
   payment. Source-body equality alone is insufficient: headers, macros, target layout,
   dependency bindings and preparation context can change behavior.
2. **Compiler-derived behavior as the default input to lowering.** Own preprocessing,
   types, constants, calling conventions, layouts, integer semantics and initializer data
   deterministically. Test whether the existing C anchor can be adapted into a buildable
   candidate through bounded transformations, instead of always asking a model to rewrite
   the whole function. Treat anchor correctness and target support as obligations to test,
   not a theorem supplied by its name. For C++, keep frontend-specific layout, lifetime,
   exception and COM authorities; do not pretend a C anchor covers them.
3. **Bound the model's job.** Let it propose transformations for an explicitly identified
   unsupported region, mapping, or control-flow problem. Require source correspondence for
   the output. Preserve reference implementations and banked candidates. Maintain a readable
   backlog when a construct cannot be lowered or checked. No unsupported construct becomes a
   successful empty body, and no audit relaxation silently becomes semantic acceptance.
4. **Attach verification to each transformation.** Start with typed relationships:
   constant-to-expression, argument-to-call, branch-to-condition, load/store width, arithmetic
   signedness and conversion, cleanup/effect ordering. Token multisets and source order are
   supporting checks, not equivalence proofs. Structural transformations may legitimately
   reorder or duplicate code; checks must describe what they prove and return unsupported
   when the relationship cannot be established. Retain candidate failures for checker replay.
5. **Exercise actual behavior at the module boundary.** Use genuine C references and
   existing library conformance tests, add differential/property/fuzz cases for uncovered
   behavior, and compare returns, buffers, externally visible effects and cleanup paths.
   Generate inputs and harness plumbing, not expected answers copied from the candidate.
   Pin architecture/configuration and isolate undefined/unspecified C behavior so mismatches
   remain interpretable. Three-arch compile/link/ABI/export gates and OS smoke still apply;
   a green host test is not a claim about untested guest ABI or concurrency.
6. **Schedule independent work under explicit resource and spending limits.** Separate
   frontend/substrate preparation, candidate production and expensive integration. Cache each
   stage by its actual inputs; group compiler failures by shared root before model escalation.
   Measure critical-path time and marginal acceptance yield before tuning batching, model
   selection or concurrency. Budget reservation must include in-flight calls and possible
   retries, not just charges from already completed work.

Compiler-generated Rust may remain unsafe and C-shaped initially. Behavioral compatibility,
Rust implementation coverage and reduction of unsafe surface are separate milestones.
Ownership/RAII improvements should be checked transformations after a faithful baseline,
not an unmeasured rewrite bundled into the first port.

## Execution order and decision gates

This measurement changes the handoff's priority order. Namespace refusal/default protection
already landed; the first work is evidence preservation and an honest acceptance boundary.

| Next increment | Done when |
|---|---|
| Preserve and account for every attempt | Concurrent calls, directory runs, retries, crashes and replay have distinct durable artifacts; every billed outcome has one accounting record; stage timers include parent spans |
| Separate candidate storage from validation | A stricter audit can discover and recheck existing bodies for $0; stale context is explicit; no automatic cross-policy acceptance or purchase |
| Build a banked regression corpus | Stratified C/C++ bodies, known false rejects, wrong-but-compiling mutations and ABI variants run offline; checker changes report false positives, escapes and unsupported cases separately |
| Trial deterministic anchor-to-candidate lowering | Offline replay measures unsupported constructs, buildability, behavioral evidence and repair effort; compare with banked model output on the same units |
| Run a complete module pilot | Record stage critical path, total estimated/provider cost, acceptance coverage and repairs for several distinct module shapes; obtain any paid experiment budget separately |
| Scale admission | Only qualified module shapes enter bulk work; unknown shapes return to the corpus and frontend backlog |

Include pointer-heavy code, tables, macros, floating point, callback APIs and C++ lifetimes in
that corpus. Split evaluation by module/family, so near-duplicate functions do not make the
held-out results look better. Seed known behavioral mutations to test whether the acceptance
pipeline can detect wrong code that compiles. Report the tested boundary; zero observed
mismatches is not proof of zero defects across millions of lines.

The next 100 translations should be a measured cohort with acceptance evidence, not a throughput
race. No credible dollar or wall-clock forecast for several million lines follows from the
current bank alone. The missing quantities are semantic acceptance yield and repair cost,
not merely average API price per function.

## Validation and remaining gates

The ledger's 10 regression tests pass, including copied banks, reused preamble paths,
ambiguous joins, conflicting usage, repeated source versions and unknown flags. Ruff and
pyright pass. Full `check-tooling` returns exit 1 with 57/59 checks passing: the deterministic
translation test lacks this worktree's compilation database, and the ARM64 record/replay
fixture lacks `qemu/boot.img`. The strict source-deferred check returns `SCANNER_ERROR` from
`verify_signatures` for missing generated headers; its diagnostic requests an amd64 build.
These gates remain owed before landing. No builds, smokes or landing were performed.

Full test output and the source-deferred summary/log are preserved beside `ledger.json` in
`~/vibeos-rust-staging/rustconv-speedrun/` as `check-tooling.log`,
`source-deferred-summary.json` and `source-deferred-full.log`.
